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
  return (
    <div>
      <PageHeader
        eyebrow="Overview"
        title="Dashboard"
        description="Your Argos workspace is ready. Foundation modules will appear here as they become available."
      />
      <section
        aria-labelledby="workspace-title"
        className="mt-8 max-w-3xl rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-6 shadow-sm"
      >
        <h2
          id="workspace-title"
          className="text-base font-semibold text-[var(--text)]"
        >
          Foundation workspace
        </h2>
        <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
          Use the sidebar to open core settings and diagnostics. This dashboard
          intentionally starts without loading feature data.
        </p>
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
