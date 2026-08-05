import { CircleAlertIcon } from "lucide-react";

import type { ModulePageProps } from "../registry";

export default function UnavailableModulePage({ module }: ModulePageProps) {
  const reason =
    module.health_reason && "message" in module.health_reason
      ? module.health_reason.message
      : "This module is not available in the current build.";

  return (
    <section className="max-w-3xl" aria-labelledby="module-title">
      <p className="text-xs font-semibold tracking-[0.18em] text-[var(--text-muted)] uppercase">
        Module
      </p>
      <h1
        id="module-title"
        className="mt-3 text-3xl font-semibold tracking-tight text-[var(--text)]"
      >
        {module.manifest.display_name}
      </h1>
      <div
        role="status"
        className="mt-8 flex gap-3 rounded-xl border border-[var(--border)] bg-[var(--surface)] p-5 text-sm text-[var(--text-muted)]"
      >
        <CircleAlertIcon
          aria-hidden="true"
          className="mt-0.5 size-5 shrink-0"
        />
        <div>
          <p className="font-semibold text-[var(--text)]">Module unavailable</p>
          <p className="mt-1 leading-6">{reason}</p>
        </div>
      </div>
    </section>
  );
}
