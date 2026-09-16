import { describe, expect, it } from "vitest";
import { errorMessage, formatBytes } from "./api";

describe("formatBytes", () => {
  it("форматира мегабайти", () => {
    expect(formatBytes(190_085_487)).toBe("181 MB");
  });

  it("форматира гигабайти", () => {
    expect(formatBytes(1_533_763_059)).toBe("1.4 GB");
  });
});

describe("errorMessage", () => {
  it("връща текста на грешката от бекенда", () => {
    expect(errorMessage("Не е открит микрофон.")).toBe("Не е открит микрофон.");
    expect(errorMessage(new Error("Грешка"))).toBe("Грешка");
  });
});
