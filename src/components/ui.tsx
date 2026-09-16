import type { ChangeEvent, ReactNode } from "react";

export function Card({
  title,
  description,
  children,
  footer,
}: {
  title?: string;
  description?: string;
  children: ReactNode;
  footer?: ReactNode;
}) {
  return (
    <section className="card">
      {title && <h2 className="card__title">{title}</h2>}
      {description && <p className="card__description">{description}</p>}
      <div className="card__body">{children}</div>
      {footer && <div className="card__footer">{footer}</div>}
    </section>
  );
}

export function Row({
  label,
  hint,
  htmlFor,
  children,
}: {
  label: string;
  hint?: string;
  htmlFor?: string;
  children: ReactNode;
}) {
  return (
    <div className="row">
      <div className="row__text">
        <label className="row__label" htmlFor={htmlFor}>
          {label}
        </label>
        {hint && <p className="row__hint">{hint}</p>}
      </div>
      <div className="row__control">{children}</div>
    </div>
  );
}

export function Toggle({
  id,
  checked,
  onChange,
  label,
  hint,
  disabled,
}: {
  id: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
  hint?: string;
  disabled?: boolean;
}) {
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <label className="switch">
        <input
          id={id}
          type="checkbox"
          checked={checked}
          disabled={disabled}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.checked)}
        />
        <span className="switch__track" aria-hidden="true" />
      </label>
    </Row>
  );
}

export function Select<T extends string>({
  id,
  value,
  options,
  onChange,
  label,
  hint,
  disabled,
}: {
  id: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
  label: string;
  hint?: string;
  disabled?: boolean;
}) {
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <select
        id={id}
        className="select"
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value as T)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </Row>
  );
}

export function NumberField({
  id,
  value,
  onChange,
  label,
  hint,
  min,
  max,
}: {
  id: string;
  value: number;
  onChange: (value: number) => void;
  label: string;
  hint?: string;
  min?: number;
  max?: number;
}) {
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <input
        id={id}
        className="input input--number"
        type="number"
        value={value}
        min={min}
        max={max}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </Row>
  );
}

export function Button({
  children,
  onClick,
  variant = "secondary",
  disabled,
  type = "button",
}: {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "secondary" | "danger" | "ghost";
  disabled?: boolean;
  type?: "button" | "submit";
}) {
  return (
    <button
      type={type}
      className={`button button--${variant}`}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}

export function Banner({ kind, children }: { kind: "info" | "error" | "success"; children: ReactNode }) {
  return (
    <div className={`banner banner--${kind}`} role={kind === "error" ? "alert" : "status"}>
      {children}
    </div>
  );
}

/** Индикатор за нивото на микрофона. */
export function LevelMeter({ level, bars = 12 }: { level: number; bars?: number }) {
  const active = Math.round(level * bars);
  return (
    <div className="meter" role="img" aria-label={`${Math.round(level * 100)}%`}>
      {Array.from({ length: bars }, (_, index) => (
        <span
          key={index}
          className={`meter__bar ${index < active ? "meter__bar--on" : ""}`}
          style={{ height: `${20 + (index / bars) * 80}%` }}
        />
      ))}
    </div>
  );
}
