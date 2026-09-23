import type { CSSProperties, MouseEvent as ReactMouseEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppState } from "../hooks/useAppState";
import { useOverlayDrag } from "../hooks/useOverlayDrag";
import { useStatus } from "../hooks/useStatus";

function MicIcon() {
  return (
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <rect x="9" y="2" width="6" height="12" rx="3" fill="currentColor" />
      <path
        d="M5 10a7 7 0 0 0 14 0M12 19v3"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
      />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <polyline
        points="5 13 10 18 19 7"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function QuestionIcon() {
  return (
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <path
        d="M9 9a3 3 0 1 1 4.5 2.6c-.9.5-1.5 1-1.5 2.1"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="12" cy="18" r="1" fill="currentColor" />
    </svg>
  );
}

function WarningIcon() {
  return (
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <path
        d="M12 3 22 20H2Z"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinejoin="round"
      />
      <line x1="12" y1="9.5" x2="12" y2="14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <circle cx="12" cy="17" r="1" fill="currentColor" />
    </svg>
  );
}

/** Мини еквалайзер вместо статична икона, докато Glasopis слуша - реагира на
 * нивото на звука и никога не спира да мърда, дори в пълна тишина. */
function Waveform({ level }: { level: number }) {
  const bars = [0, 1, 2, 3];
  return (
    <span className="overlay-ball__wave" aria-hidden="true">
      {bars.map((i) => (
        <span
          key={i}
          className="overlay-ball__wave-bar"
          style={
            {
              "--level": level,
              animationDelay: `${i * 130}ms`,
            } as CSSProperties
          }
        />
      ))}
    </span>
  );
}

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
        : status.state === "confirming"
          ? t("overlay.confirming")
          : status.state === "done"
            ? t("overlay.done")
            : status.state === "error"
              ? t("overlay.error")
              : t("status.idle");

  const detail =
    status.state === "error"
      ? status.message
      : status.state === "confirming"
        ? status.question
        : status.state === "done" && status.clipboard_only
          ? t("overlay.clipboardOnly")
          : status.state === "done"
            ? status.text
            : null;

  const icon =
    status.state === "error" ? (
      <WarningIcon />
    ) : status.state === "confirming" ? (
      <QuestionIcon />
    ) : status.state === "done" ? (
      <CheckIcon />
    ) : status.state === "listening" ? (
      <Waveform level={level} />
    ) : (
      <MicIcon />
    );

  const style =
    status.state === "listening"
      ? ({ "--level": level } as CSSProperties)
      : undefined;

  const startDrag = (event: ReactMouseEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    void getCurrentWindow().startDragging();
  };

  // Спрян асистент чака гласов отговор ("да"/"не") на въпроса си - ако
  // единственият начин да го прочетете е да задържите мишката върху топката,
  // това не е разговор. Затова докато чака, въпросът стои изписан открито.
  const showQuestionBubble = status.state === "confirming";

  return (
    <div className="overlay-root">
      <div
        className={`overlay-ball overlay-ball--${status.state}`}
        style={style}
        title={detail ? `${title} - ${detail}` : title}
        onMouseDown={startDrag}
      >
        {status.state === "processing" && <span className="overlay-ball__spinner" aria-hidden="true" />}
        <span className="overlay-ball__icon">{icon}</span>
        <span className="sr-only">{detail ? `${title} - ${detail}` : title}</span>
      </div>
      {showQuestionBubble && (
        <div className="overlay-question" onMouseDown={startDrag}>
          {status.question}
        </div>
      )}
    </div>
  );
}
