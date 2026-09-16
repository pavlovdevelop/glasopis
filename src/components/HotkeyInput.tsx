import { useState } from "react";

/** Клавиши, които не могат да бъдат самостоятелна комбинация. */
const MODIFIERS = ["Control", "Alt", "Shift", "Meta"];

export interface KeyCombination {
  key: string;
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

/** Превръща събитие от клавиатурата в акселератор като `Ctrl+Alt+Space`. */
export function acceleratorFrom(event: KeyCombination): string | null {
  if (MODIFIERS.includes(event.key)) return null;
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Super");

  const key = event.code.startsWith("Key")
    ? event.code.slice(3)
    : event.code.startsWith("Digit")
      ? event.code.slice(5)
      : event.code;
  parts.push(key);
  return parts.join("+");
}

export function HotkeyInput({
  value,
  onChange,
  placeholder,
  disabled,
}: {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  disabled?: boolean;
}) {
  const [recording, setRecording] = useState(false);

  return (
    <button
      type="button"
      className={`hotkey ${recording ? "hotkey--recording" : ""}`}
      disabled={disabled}
      onClick={() => setRecording(true)}
      onBlur={() => setRecording(false)}
      onKeyDown={(event) => {
        if (!recording) return;
        event.preventDefault();
        const accelerator = acceleratorFrom(event);
        if (accelerator) {
          onChange(accelerator);
          setRecording(false);
        }
      }}
    >
      {recording ? placeholder : value}
    </button>
  );
}
