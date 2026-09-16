//! Downloading and managing the local speech models.
//!
//! Models are plain files in `%LOCALAPPDATA%\Glasopis\models\`. They are
//! downloaded from the whisper.cpp model repository on Hugging Face — no
//! account, no API key, no payment — and every download is verified against the
//! SHA-256 checksum stored in the catalogue.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use glasopis_core::models::{self, ModelInfo};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};

use crate::errors::{GlasopisError, Result};
use crate::paths;
use crate::state::{AppState, MODEL_EVENT};

/// What the settings window shows for one catalogue entry.
#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub id: String,
    pub label: String,
    pub technical_name: String,
    pub size_bytes: u64,
    pub size_mb: u64,
    pub ram_mb: u32,
    pub recommended: bool,
    pub downloaded: bool,
    pub active: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ModelEvent {
    Downloading {
        id: String,
        downloaded: u64,
        total: u64,
        percent: f64,
    },
    Verifying {
        id: String,
    },
    Ready {
        id: String,
    },
    Cancelled {
        id: String,
    },
    Failed {
        id: String,
        message: String,
    },
    Deleted {
        id: String,
    },
}

pub fn model_path(app: &AppHandle, info: &ModelInfo) -> Result<PathBuf> {
    Ok(paths::models_dir(app)?.join(info.file_name))
}

/// The file of the model the user selected, if it is present on disk.
pub fn active_model_path(app: &AppHandle, model_id: Option<&str>) -> Result<PathBuf> {
    let id = model_id.ok_or(GlasopisError::NoModelSelected)?;
    let info = models::find(id).ok_or(GlasopisError::NoModelSelected)?;
    let path = model_path(app, info)?;
    if !path.is_file() {
        return Err(GlasopisError::ModelMissingOrCorrupted);
    }
    Ok(path)
}

pub fn list(app: &AppHandle) -> Result<Vec<ModelStatus>> {
    let dir = paths::models_dir(app)?;
    let state = app.state::<AppState>();
    let active_id = state.settings().voice.model_id;

    Ok(models::CATALOG
        .iter()
        .map(|info| {
            let path = dir.join(info.file_name);
            ModelStatus {
                id: info.id.to_string(),
                label: info.tier.label_bg().to_string(),
                technical_name: info.technical_name.to_string(),
                size_bytes: info.size_bytes,
                size_mb: info.size_mb(),
                ram_mb: info.ram_mb,
                recommended: info.recommended,
                downloaded: path.is_file(),
                active: active_id.as_deref() == Some(info.id),
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect())
}

pub fn delete(app: &AppHandle, id: &str) -> Result<()> {
    let info = models::find(id).ok_or(GlasopisError::NoModelSelected)?;
    let path = model_path(app, info)?;
    let state = app.state::<AppState>();
    // The model may be loaded in memory; release it before deleting the file.
    if state.settings().voice.model_id.as_deref() == Some(id) {
        state.engine.unload();
    }
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    let _ = app.emit(MODEL_EVENT, ModelEvent::Deleted { id: id.to_string() });
    Ok(())
}

/// Asks a running download to stop.
pub fn cancel(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    let flag = state.cancel_flags.lock().get(id).cloned();
    if let Some(flag) = flag {
        flag.store(true, Ordering::SeqCst);
    }
}

/// Starts the download on a background thread and reports progress through
/// [`MODEL_EVENT`].
pub fn download(app: &AppHandle, id: &str) -> Result<()> {
    let info =
        models::find(id).ok_or_else(|| GlasopisError::other(format!("Непознат модел: {id}")))?;
    if !models::is_trusted_url(info.url) {
        return Err(GlasopisError::other(
            "Адресът за изтегляне на модела не е разпознат.".to_string(),
        ));
    }
    let target = model_path(app, info)?;
    if target.is_file() {
        let _ = app.emit(MODEL_EVENT, ModelEvent::Ready { id: id.to_string() });
        return Ok(());
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let state = app.state::<AppState>();
        let mut flags = state.cancel_flags.lock();
        if flags.contains_key(id) {
            return Err(GlasopisError::other("Моделът вече се изтегля.".to_string()));
        }
        flags.insert(id.to_string(), Arc::clone(&cancel_flag));
    }

    let app = app.clone();
    let id = id.to_string();
    std::thread::Builder::new()
        .name("glasopis-model-download".into())
        .spawn(move || {
            let info = models::find(&id).expect("моделът е проверен по-горе");
            let event = match run_download(&app, info, &target, &cancel_flag) {
                Ok(true) => ModelEvent::Ready { id: id.clone() },
                Ok(false) => ModelEvent::Cancelled { id: id.clone() },
                Err(err) => {
                    log::error!("изтеглянето на модел {id} не успя: {err}");
                    ModelEvent::Failed {
                        id: id.clone(),
                        message: err.to_string(),
                    }
                }
            };
            let state = app.state::<AppState>();
            state.cancel_flags.lock().remove(&id);
            let _ = app.emit(MODEL_EVENT, event);
        })
        .map_err(|e| GlasopisError::other(format!("Изтеглянето не може да започне: {e}")))?;
    Ok(())
}

/// Returns `Ok(true)` when the model was downloaded and verified,
/// `Ok(false)` when the user cancelled.
fn run_download(
    app: &AppHandle,
    info: &ModelInfo,
    target: &PathBuf,
    cancel_flag: &AtomicBool,
) -> Result<bool> {
    let part = target.with_extension("part");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("Glasopis/", env!("CARGO_PKG_VERSION")))
        .timeout(None)
        .connect_timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| GlasopisError::other(format!("Неуспешна мрежова заявка: {e}")))?;

    let mut response = client.get(info.url).send().map_err(|err| {
        log::error!("мрежова грешка: {err}");
        GlasopisError::NetworkUnavailable
    })?;
    if !response.status().is_success() {
        return Err(GlasopisError::other(format!(
            "Сървърът върна грешка {} при изтеглянето на модела.",
            response.status()
        )));
    }

    let total = response.content_length().unwrap_or(info.size_bytes);
    let mut file = std::fs::File::create(&part)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_report = std::time::Instant::now();

    loop {
        if cancel_flag.load(Ordering::SeqCst) {
            drop(file);
            let _ = std::fs::remove_file(&part);
            return Ok(false);
        }
        let read = response
            .read(&mut buffer)
            .map_err(|e| GlasopisError::other(format!("Прекъснато изтегляне: {e}")))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        downloaded += read as u64;

        if last_report.elapsed() >= std::time::Duration::from_millis(200) {
            last_report = std::time::Instant::now();
            let percent = if total > 0 {
                (downloaded as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let _ = app.emit(
                MODEL_EVENT,
                ModelEvent::Downloading {
                    id: info.id.to_string(),
                    downloaded,
                    total,
                    percent,
                },
            );
        }
    }
    file.flush()?;
    drop(file);

    let _ = app.emit(
        MODEL_EVENT,
        ModelEvent::Verifying {
            id: info.id.to_string(),
        },
    );
    let digest = format!("{:x}", hasher.finalize());
    if digest != info.sha256 {
        let _ = std::fs::remove_file(&part);
        return Err(GlasopisError::other(
            "Изтегленият файл е повреден (несъответстваща контролна сума). Опитайте отново."
                .to_string(),
        ));
    }

    std::fs::rename(&part, target)?;
    log::info!("моделът {} е готов: {}", info.id, target.display());
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_events_serialize_for_the_ui() {
        let json = serde_json::to_string(&ModelEvent::Downloading {
            id: "medium".into(),
            downloaded: 50,
            total: 100,
            percent: 50.0,
        })
        .unwrap();
        assert!(json.contains("\"state\":\"downloading\""));
        assert!(json.contains("\"percent\":50.0"));
    }

    #[test]
    fn every_catalog_url_is_trusted() {
        for info in models::CATALOG {
            assert!(models::is_trusted_url(info.url));
        }
    }
}
