import { describe, expect, it } from "vitest";
import bg from "./bg.json";
import en from "./en.json";
import { translate } from "./index";

function keys(value: unknown, prefix = ""): string[] {
  if (typeof value !== "object" || value === null) return [prefix];
  return Object.entries(value as Record<string, unknown>).flatMap(([key, child]) =>
    keys(child, prefix ? `${prefix}.${key}` : key),
  );
}

describe("i18n", () => {
  it("английският превод покрива всички български ключове", () => {
    expect(keys(en).sort()).toEqual(keys(bg).sort());
  });

  it("няма празни преводи", () => {
    for (const language of [bg, en]) {
      for (const key of keys(language)) {
        expect(translate("bg", key).length).toBeGreaterThan(0);
      }
    }
  });

  it("заменя параметри", () => {
    expect(translate("bg", "dictation.hint", { hotkey: "Ctrl+Alt+Space" })).toContain(
      "Ctrl+Alt+Space",
    );
  });

  it("пада обратно към български при липсващ ключ", () => {
    expect(translate("en", "app.name")).toBe("Glasopis");
    expect(translate("en", "няма.такъв.ключ")).toBe("няма.такъв.ключ");
  });
});
