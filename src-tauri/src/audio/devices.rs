//! Enumeration of Windows audio input devices.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

use crate::errors::{GlasopisError, Result};

#[derive(Debug, Clone, Serialize)]
pub struct InputDevice {
    /// Device name as reported by Windows; also the value stored in settings.
    pub name: String,
    pub is_default: bool,
}

/// Human readable name of a device, as Windows reports it.
fn device_name(device: &cpal::Device) -> Option<String> {
    device
        .description()
        .ok()
        .map(|description| description.name().to_string())
}

/// Lists every usable input device.
pub fn list_input_devices() -> Result<Vec<InputDevice>> {
    let host = cpal::default_host();
    let default_name = host.default_input_device().and_then(|d| device_name(&d));

    let devices = host.input_devices().map_err(|e| {
        GlasopisError::other(format!("Неуспешно четене на аудио устройствата: {e}"))
    })?;

    let mut out = Vec::new();
    for device in devices {
        let Some(name) = device_name(&device) else {
            continue;
        };
        // Devices without a usable input configuration only confuse the user.
        if device.default_input_config().is_err() {
            continue;
        }
        let is_default = default_name.as_deref() == Some(name.as_str());
        out.push(InputDevice { name, is_default });
    }

    if out.is_empty() {
        return Err(GlasopisError::NoMicrophone);
    }
    out.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.cmp(&b.name)));
    Ok(out)
}

/// Resolves the device that should be used for recording.
pub fn resolve_device(preferred: Option<&str>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    if let Some(name) = preferred {
        let devices = host.input_devices().map_err(|e| {
            GlasopisError::other(format!("Неуспешно четене на аудио устройствата: {e}"))
        })?;
        for device in devices {
            if device_name(&device).as_deref() == Some(name) {
                return Ok(device);
            }
        }
        log::warn!("избраният микрофон не е намерен, използвам стандартния");
    }
    host.default_input_device()
        .ok_or(GlasopisError::NoMicrophone)
}
