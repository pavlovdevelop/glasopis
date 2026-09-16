import bg from "./bg.json";
import en from "./en.json";

export type Language = "bg" | "en";

/** Всички преводи. Български е езикът по подразбиране. */
export const translations: Record<Language, unknown> = { bg, en };

export const languages: { value: Language; labelKey: string }[] = [
  { value: "bg", labelKey: "general.languageBg" },
  { value: "en", labelKey: "general.languageEn" },
];

function lookup(source: unknown, path: string): string | undefined {
  const value = path
    .split(".")
    .reduce<unknown>(
      (acc, part) =>
        acc && typeof acc === "object"
          ? (acc as Record<string, unknown>)[part]
          : undefined,
      source,
    );
  return typeof value === "string" ? value : undefined;
}

/**
 * Превежда ключ като `settings.general.title`.
 * Липсващ превод пада обратно към българския, за да не остане празен текст.
 */
export function translate(
  language: Language,
  key: string,
  params?: Record<string, string | number>,
): string {
  const text = lookup(translations[language], key) ?? lookup(bg, key) ?? key;
  if (!params) return text;
  return Object.entries(params).reduce(
    (acc, [name, value]) => acc.replaceAll(`{${name}}`, String(value)),
    text,
  );
}
