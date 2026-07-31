import { MonitorIcon } from "lucide-react";

import { useSystemIdentity } from "./systemIdentity";

interface PageHeaderProps {
  eyebrow: string;
  title: string;
  description: string;
}

function PageHeader({ eyebrow, title, description }: PageHeaderProps) {
  return (
    <header className="max-w-3xl">
      <p className="text-xs font-semibold tracking-[0.18em] text-[var(--text-muted)] uppercase">
        {eyebrow}
      </p>
      <h1 className="mt-3 text-3xl font-semibold tracking-tight text-[var(--text)]">
        {title}
      </h1>
      <p className="mt-3 text-sm leading-6 text-[var(--text-muted)]">
        {description}
      </p>
    </header>
  );
}

export function DashboardPage() {
  const identity = useSystemIdentity();
  const hostname = identity.data?.hostname ?? "This machine";

  return (
    <div className="max-w-5xl">
      <section
        aria-busy={identity.isPending}
        aria-labelledby="machine-name"
        className="relative isolate flex min-h-[30rem] overflow-hidden rounded-3xl border border-[var(--border)] bg-[var(--surface)] px-7 py-10 shadow-sm sm:px-12 lg:px-16"
      >
        <div
          aria-hidden="true"
          className="absolute -top-32 -right-24 -z-10 size-80 rounded-full bg-[var(--accent)] opacity-[0.08] blur-3xl"
        />
        <div
          aria-hidden="true"
          className="absolute -bottom-44 -left-28 -z-10 size-96 rounded-full bg-[var(--text)] opacity-[0.04] blur-3xl"
        />

        <div className="my-auto max-w-3xl">
          <div
            aria-hidden="true"
            className="mb-8 grid size-12 place-items-center rounded-2xl border border-[var(--border)] bg-[var(--background)] text-[var(--accent)] shadow-sm"
          >
            <MonitorIcon className="size-5" />
          </div>
          <p className="text-xs font-semibold tracking-[0.2em] text-[var(--text-muted)] uppercase">
            Dashboard
          </p>
          <h1
            id="machine-name"
            className="mt-4 break-words text-4xl font-semibold tracking-[-0.04em] text-[var(--text)] sm:text-5xl lg:text-6xl"
          >
            {hostname}
          </h1>
          <p className="mt-5 text-base leading-7 text-[var(--text-muted)] sm:text-lg">
            Your local control center.
          </p>

          {identity.isPending ? (
            <p role="status" className="mt-10 text-sm text-[var(--text-muted)]">
              Reading hostname…
            </p>
          ) : null}
          {identity.isError ? (
            <p role="status" className="mt-10 text-sm text-[var(--text-muted)]">
              Hostname unavailable
            </p>
          ) : null}
        </div>
      </section>
    </div>
  );
}

export function SettingsPage() {
  return (
    <PageHeader
      eyebrow="Core"
      title="Settings"
      description="Application and module preferences will be configured here as their foundation tasks are completed."
    />
  );
}

export function DiagnosticsPage() {
  return (
    <PageHeader
      eyebrow="Core"
      title="Diagnostics"
      description="Runtime health and safe diagnostic details will be available here without blocking the rest of Argos."
    />
  );
}
