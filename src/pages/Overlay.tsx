import { LevelMeter } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { useStatus } from "../hooks/useStatus";

/** Плаващият прозорец, който се показва по време на диктовка. */
export function Overlay() {
  const { t } = useAppState();
  const { status, level } = useStatus();

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

  return (
    <div className={`overlay overlay--${status.state}`}>
      <div className="overlay__row">
        <span className="overlay__icon" aria-hidden="true">
          {status.state === "error" ? "⚠" : status.state === "done" ? "✓" : "🎙"}
        </span>
        <span className="overlay__title">{title}</span>
      </div>
      {status.state === "listening" ? (
        <LevelMeter level={level} bars={16} />
      ) : status.state === "processing" ? (
        <div className="overlay__progress" aria-hidden="true">
          <span />
        </div>
      ) : (
        detail && <p className="overlay__detail">{detail}</p>
      )}
    </div>
  );
}
