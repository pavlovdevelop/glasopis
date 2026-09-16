import { useCallback, useEffect, useState } from "react";
import { Button, Card } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { api, errorMessage, type HistoryEntry } from "../services/api";

export function HistoryPage() {
  const { settings, t, setError } = useAppState();
  const [entries, setEntries] = useState<HistoryEntry[]>([]);

  const refresh = useCallback(async () => {
    try {
      setEntries((await api.getHistory()).entries);
    } catch (err) {
      setError(errorMessage(err));
    }
  }, [setError]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const clear = async () => {
    if (!window.confirm(t("history.clearConfirm"))) return;
    try {
      await api.clearHistory();
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  if (!settings) return null;

  return (
    <Card title={t("history.title")}>
      {!settings.privacy.keep_history && <p className="hint">{t("history.disabled")}</p>}
      {entries.length === 0 ? (
        <p className="hint">{t("history.empty")}</p>
      ) : (
        <ul className="history">
          {entries.map((entry) => (
            <li key={`${entry.timestamp}-${entry.text.slice(0, 12)}`} className="history__item">
              <time className="history__time">
                {new Date(entry.timestamp * 1000).toLocaleString()}
              </time>
              <p className="history__text">{entry.text}</p>
            </li>
          ))}
        </ul>
      )}
      <div className="actions">
        <Button variant="danger" onClick={clear} disabled={entries.length === 0}>
          {t("history.clear")}
        </Button>
      </div>
    </Card>
  );
}
