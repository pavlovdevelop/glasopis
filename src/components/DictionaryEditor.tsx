import { useState } from "react";
import { Button, Card } from "./ui";
import { useAppState } from "../hooks/useAppState";
import type { DictionaryEntry } from "../services/api";

/** Личен речник: „както го казвам“ → „както да се изпише“. */
export function DictionaryEditor() {
  const { settings, save, t } = useAppState();
  const [spoken, setSpoken] = useState("");
  const [written, setWritten] = useState("");
  if (!settings) return null;

  const entries = settings.dictionary.entries;
  const persist = (next: DictionaryEntry[]) =>
    void save({ ...settings, dictionary: { entries: next } });

  const add = () => {
    const trimmedSpoken = spoken.trim();
    const trimmedWritten = written.trim();
    if (!trimmedSpoken || !trimmedWritten) return;
    persist([
      ...entries.filter((entry) => entry.spoken.toLowerCase() !== trimmedSpoken.toLowerCase()),
      { spoken: trimmedSpoken, written: trimmedWritten },
    ]);
    setSpoken("");
    setWritten("");
  };

  return (
    <Card title={t("dictionary.title")} description={t("dictionary.hint")}>
      {entries.length === 0 ? (
        <p className="hint">{t("dictionary.empty")}</p>
      ) : (
        <ul className="dictionary">
          {entries.map((entry) => (
            <li key={entry.spoken} className="dictionary__item">
              <span className="dictionary__spoken">{entry.spoken}</span>
              <span aria-hidden="true">→</span>
              <span className="dictionary__written">{entry.written}</span>
              <Button
                variant="ghost"
                onClick={() => persist(entries.filter((item) => item.spoken !== entry.spoken))}
              >
                {t("common.delete")}
              </Button>
            </li>
          ))}
        </ul>
      )}
      <form
        className="actions"
        onSubmit={(event) => {
          event.preventDefault();
          add();
        }}
      >
        <label className="sr-only" htmlFor="dictionary-spoken">
          {t("dictionary.spoken")}
        </label>
        <input
          id="dictionary-spoken"
          className="input"
          value={spoken}
          placeholder={t("dictionary.spokenPlaceholder")}
          onChange={(event) => setSpoken(event.target.value)}
        />
        <span aria-hidden="true">→</span>
        <label className="sr-only" htmlFor="dictionary-written">
          {t("dictionary.written")}
        </label>
        <input
          id="dictionary-written"
          className="input"
          value={written}
          placeholder={t("dictionary.writtenPlaceholder")}
          onChange={(event) => setWritten(event.target.value)}
        />
        <Button type="submit" variant="primary" disabled={!spoken.trim() || !written.trim()}>
          {t("dictionary.add")}
        </Button>
      </form>
    </Card>
  );
}
