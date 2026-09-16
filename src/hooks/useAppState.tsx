import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { api, errorMessage, type Settings } from "../services/api";
import { translate, type Language } from "../i18n";

interface AppStateValue {
  settings: Settings | null;
  language: Language;
  t: (key: string, params?: Record<string, string | number>) => string;
  /** Записва настройките в бекенда и обновява локалното състояние. */
  save: (next: Settings) => Promise<void>;
  /** Презарежда настройките (например след избор на модел). */
  reload: () => Promise<void>;
  error: string | null;
  setError: (message: string | null) => void;
  saving: boolean;
}

const AppStateContext = createContext<AppStateValue | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const reload = useCallback(async () => {
    try {
      setSettings(await api.getSettings());
    } catch (err) {
      setError(errorMessage(err));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const save = useCallback(async (next: Settings) => {
    setSaving(true);
    try {
      setSettings(await api.saveSettings(next));
      setError(null);
    } catch (err) {
      setError(errorMessage(err));
      // Настройките в бекенда не са променени — връщаме показаните.
      try {
        setSettings(await api.getSettings());
      } catch {
        /* вече показваме грешка */
      }
    } finally {
      setSaving(false);
    }
  }, []);

  const language: Language = settings?.general.ui_language === "en" ? "en" : "bg";

  const value = useMemo<AppStateValue>(
    () => ({
      settings,
      language,
      t: (key, params) => translate(language, key, params),
      save,
      reload,
      error,
      setError,
      saving,
    }),
    [settings, language, save, reload, error, saving],
  );

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  return <AppStateContext.Provider value={value}>{children}</AppStateContext.Provider>;
}

export function useAppState(): AppStateValue {
  const value = useContext(AppStateContext);
  if (!value) throw new Error("useAppState трябва да е вътре в AppStateProvider");
  return value;
}
