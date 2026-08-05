import { XIcon } from "lucide-react";
import { useEffect, useId, useRef, type ReactNode } from "react";

interface DialogProps {
  title: string;
  onClose: () => void;
  children: ReactNode;
}

export function Dialog({ title, onClose, children }: DialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;
    const invoker =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    if (!dialog) return;
    dialog.showModal();
    dialog.querySelector<HTMLElement>("button, [href], input, select")?.focus();
    return () => {
      dialog.close();
      invoker?.focus();
    };
  }, []);

  return (
    <dialog
      ref={dialogRef}
      aria-labelledby={titleId}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      className="m-auto max-h-[min(48rem,calc(100dvh-2rem))] w-[min(44rem,calc(100%-2rem))] overflow-auto rounded-2xl border border-[var(--border)] bg-[var(--surface-raised)] p-0 text-[var(--text)] shadow-2xl backdrop:bg-black/45"
    >
      <div className="sticky top-0 z-10 flex items-center gap-4 border-b border-[var(--border)] bg-[var(--surface-raised)] px-5 py-4">
        <h2
          id={titleId}
          className="min-w-0 flex-1 truncate text-lg font-semibold"
        >
          {title}
        </h2>
        <button
          type="button"
          aria-label="Close dialog"
          onClick={onClose}
          className="grid size-10 shrink-0 place-items-center rounded-lg text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)] focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] focus-visible:outline-none"
        >
          <XIcon aria-hidden="true" className="size-5" />
        </button>
      </div>
      <div className="p-5">{children}</div>
    </dialog>
  );
}
