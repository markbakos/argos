import {
  AlertTriangleIcon,
  CircleAlertIcon,
  CircleCheckIcon,
  LoaderCircleIcon,
} from "lucide-react";
import type { ReactNode } from "react";

type StateTone = "loading" | "error" | "warning" | "success" | "empty";

const icons = {
  loading: LoaderCircleIcon,
  error: CircleAlertIcon,
  warning: AlertTriangleIcon,
  success: CircleCheckIcon,
  empty: CircleAlertIcon,
};

export function StateMessage({
  tone,
  title,
  children,
}: {
  tone: StateTone;
  title: string;
  children?: ReactNode;
}) {
  const Icon = icons[tone];
  return (
    <div
      role={tone === "error" ? "alert" : "status"}
      className="flex gap-3 rounded-xl border border-[var(--border)] bg-[var(--surface)] p-5"
    >
      <Icon
        aria-hidden="true"
        className={`mt-0.5 size-5 shrink-0 ${tone === "loading" ? "motion-safe:animate-spin" : ""}`}
      />
      <div className="min-w-0">
        <p className="font-semibold">{title}</p>
        {children ? (
          <div className="mt-1 text-sm leading-6 text-[var(--text-muted)]">
            {children}
          </div>
        ) : null}
      </div>
    </div>
  );
}
