import { useCallback, useEffect, useState } from "react";
import { Banner, Button, Card } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import {
  api,
  errorMessage,
  events,
  formatBytes,
  type ModelEvent,
  type ModelStatus,
} from "../services/api";

interface Progress {
  percent: number;
  label: string;
}

export function ModelsPage() {
  const { t, reload, setError, error } = useAppState();
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [progress, setProgress] = useState<Record<string, Progress>>({});
  const [modelsDir, setModelsDir] = useState("");

  const refresh = useCallback(async () => {
    try {
      setModels(await api.listModels());
      setModelsDir((await api.getAppInfo()).models_dir);
    } catch (err) {
      setError(errorMessage(err));
    }
  }, [setError]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void events
      .onModel((event: ModelEvent) => {
        setProgress((current) => {
          const next = { ...current };
          switch (event.state) {
            case "downloading":
              next[event.id] = {
                percent: event.percent,
                label: `${t("models.downloading")} ${Math.round(event.percent)}%`,
              };
              break;
            case "verifying":
              next[event.id] = { percent: 100, label: t("models.verifying") };
              break;
            default:
              delete next[event.id];
          }
          return next;
        });
        if (event.state === "failed") setError(event.message);
        if (event.state !== "downloading" && event.state !== "verifying") {
          void refresh();
          void reload();
        }
      })
      .then((un) => {
        unlisten = un;
      });
    return () => unlisten?.();
  }, [refresh, reload, setError, t]);

  const download = async (id: string) => {
    setError(null);
    setProgress((current) => ({
      ...current,
      [id]: { percent: 0, label: t("models.downloading") },
    }));
    try {
      await api.downloadModel(id);
    } catch (err) {
      setError(errorMessage(err));
      setProgress((current) => {
        const next = { ...current };
        delete next[id];
        return next;
      });
    }
  };

  const remove = async (id: string) => {
    if (!window.confirm(t("models.deleteConfirm"))) return;
    try {
      await api.deleteModel(id);
      await refresh();
      await reload();
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  const activate = async (id: string) => {
    try {
      await api.selectModel(id);
      await refresh();
      await reload();
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  return (
    <Card title={t("models.title")} description={t("models.intro")}>
      {error && <Banner kind="error">{error}</Banner>}
      <ul className="models">
        {models.map((model) => {
          const running = progress[model.id];
          return (
            <li key={model.id} className={`model ${model.active ? "model--active" : ""}`}>
              <div className="model__head">
                <div>
                  <h3 className="model__name">
                    {model.label}
                    {model.recommended && <span className="tag">{t("models.recommended")}</span>}
                    {model.active && <span className="tag tag--active">{t("models.active")}</span>}
                  </h3>
                  <p className="model__meta">
                    {model.technical_name} · {t("models.size")}: {formatBytes(model.size_bytes)} ·{" "}
                    {t("models.ram")}: ~{model.ram_mb} MB
                  </p>
                </div>
                <div className="model__actions">
                  {running ? (
                    <Button
                      variant="ghost"
                      onClick={() => {
                        api
                          .cancelModelDownload(model.id)
                          .catch((err) => setError(errorMessage(err)));
                      }}
                    >
                      {t("common.cancel")}
                    </Button>
                  ) : model.downloaded ? (
                    <>
                      {!model.active && (
                        <Button variant="primary" onClick={() => activate(model.id)}>
                          {t("models.activate")}
                        </Button>
                      )}
                      <Button variant="danger" onClick={() => remove(model.id)}>
                        {t("common.delete")}
                      </Button>
                    </>
                  ) : (
                    <Button variant="primary" onClick={() => download(model.id)}>
                      {t("common.download")}
                    </Button>
                  )}
                </div>
              </div>
              {running && (
                <div className="progress" aria-label={running.label}>
                  <div className="progress__bar" style={{ width: `${running.percent}%` }} />
                  <span className="progress__label">{running.label}</span>
                </div>
              )}
              {!running && (
                <p className="model__status">
                  {model.downloaded ? t("models.downloaded") : t("models.notDownloaded")}
                </p>
              )}
            </li>
          );
        })}
      </ul>
      <p className="hint">
        {t("models.storedIn")}: <code>{modelsDir}</code>{" "}
        <Button
          variant="ghost"
          onClick={() => {
            api.openFolder("models").catch((err) => setError(errorMessage(err)));
          }}
        >
          {t("common.openFolder")}
        </Button>
      </p>
    </Card>
  );
}
