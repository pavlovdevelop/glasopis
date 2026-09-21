import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppState } from "../hooks/useAppState";
import { useOverlayDrag } from "../hooks/useOverlayDrag";
import { useStatus } from "../hooks/useStatus";

/**
 * Малка плаваща „топка“, която се показва по време на диктовка. Влачи се
 * навсякъде по екрана (позицията се запомня) и не отнема клавиатурния фокус.
 */
export function Overlay() {
  const { t } = useAppState();
  const { status, level } = useStatus();
  useOverlayDrag();

  const title =
    status.state === "listening"
      ? t("overlay.listening")
      : status.state === "processing"
        ? t("overlay.processing")
        : status.state === "done"
          ? t("overlay.done")
          : status.state === "error"
            ? t("overlay.error")
            : t("status.idle");

  const detail =
    status.state === "error"
      ? status.message
      : status.state === "done" && status.clipboard_only
        ? t("overlay.clipboardOnly")
        : status.state === "done"
          ? status.text
          : null;

  const icon = status.state === "error" ? "⚠" : status.state === "done" ? "✓" : "🎙";

  return (
    <div
      className={`overlay-ball overlay-ball--${status.state}`}
      title={detail ? `${title} — ${detail}` : title}
      onMouseDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        void getCurrentWindow().startDragging();
      }}
    >
      {status.state === "listening" && (
        <span
          className="overlay-ball__ring"
          style={{ transform: `scale(${1 + level * 0.35})` }}
          aria-hidden="true"
        />
      )}
      {status.state === "processing" && <span className="overlay-ball__spinner" aria-hidden="true" />}
      <span className="overlay-ball__icon" aria-hidden="true">
        {icon}
      </span>
      <span className="sr-only">{detail ? `${title} — ${detail}` : title}</span>
    </div>
  );
}
