import { type ReactNode, useEffect, useRef } from "react";

interface ConfirmDialogProps {
  title: string;
  children: ReactNode;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
  danger?: boolean;
}

/** A small modal question. Esc or a click outside cancels; Cancel has the focus at first. */
export function ConfirmDialog({
  title,
  children,
  confirmLabel,
  onConfirm,
  onCancel,
  danger = false,
}: ConfirmDialogProps) {
  const cancel = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    cancel.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [onCancel]);

  return (
    <div
      className="dialog-backdrop"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) onCancel();
      }}
    >
      <div className="dialog" role="alertdialog" aria-modal="true" aria-labelledby="dialog-title">
        <h2 id="dialog-title" className="dialog-title">
          {title}
        </h2>
        <div className="dialog-body">{children}</div>
        <div className="dialog-actions">
          <button ref={cancel} type="button" className="btn" onClick={onCancel}>
            Cancel
          </button>
          <button
            type="button"
            className={danger ? "btn btn-danger-solid" : "btn btn-primary"}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
