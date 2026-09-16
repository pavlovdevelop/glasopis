import { describe, expect, it } from "vitest";
import { acceleratorFrom } from "./HotkeyInput";

function key(partial: Partial<Parameters<typeof acceleratorFrom>[0]>) {
  return acceleratorFrom({
    key: "a",
    code: "KeyA",
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    ...partial,
  });
}

describe("acceleratorFrom", () => {
  it("съставя комбинация с модификатори", () => {
    expect(key({ key: " ", code: "Space", ctrlKey: true, altKey: true })).toBe("Ctrl+Alt+Space");
  });

  it("използва буквата от кода на клавиша", () => {
    expect(key({ ctrlKey: true, altKey: true })).toBe("Ctrl+Alt+A");
  });

  it("използва цифрата от кода на клавиша", () => {
    expect(key({ key: "1", code: "Digit1", ctrlKey: true })).toBe("Ctrl+1");
  });

  it("игнорира самостоятелен модификатор", () => {
    expect(key({ key: "Control", code: "ControlLeft", ctrlKey: true })).toBeNull();
  });
});
