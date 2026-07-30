import { AlertTriangleIcon, RotateCcwIcon } from "lucide-react";
import {
  Link,
  isRouteErrorResponse,
  useNavigate,
  useRouteError,
} from "react-router-dom";

function errorTitle(error: unknown) {
  if (isRouteErrorResponse(error) && error.status === 404) {
    return "Page not found";
  }

  return "This page could not be opened";
}

export function RouteErrorPage() {
  const error = useRouteError();
  const navigate = useNavigate();

  return (
    <section
      aria-labelledby="route-error-title"
      className="max-w-xl rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-6 shadow-sm"
    >
      <AlertTriangleIcon
        aria-hidden="true"
        className="size-6 text-[var(--status-warning)]"
      />
      <h1
        id="route-error-title"
        className="mt-4 text-2xl font-semibold text-[var(--text)]"
      >
        {errorTitle(error)}
      </h1>
      <p className="mt-2 text-sm leading-6 text-[var(--text-muted)]">
        Argos kept the rest of the application available. Try this route again,
        or return to the Dashboard.
      </p>
      <div className="mt-6 flex flex-wrap gap-3">
        <button
          type="button"
          onClick={() => {
            void navigate(0);
          }}
          className="inline-flex min-h-10 items-center gap-2 rounded-lg bg-[var(--accent)] px-4 text-sm font-semibold text-[var(--accent-contrast)] outline-none focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] focus-visible:ring-offset-2"
        >
          <RotateCcwIcon aria-hidden="true" className="size-4" />
          Try again
        </button>
        <Link
          to="/"
          className="inline-flex min-h-10 items-center rounded-lg border border-[var(--border)] px-4 text-sm font-semibold text-[var(--text)] outline-none hover:bg-[var(--surface-hover)] focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)]"
        >
          Go to Dashboard
        </Link>
      </div>
    </section>
  );
}

export function NotFoundPage() {
  return (
    <section aria-labelledby="not-found-title" className="max-w-xl">
      <p className="text-xs font-semibold tracking-[0.18em] text-[var(--text-muted)] uppercase">
        Not found
      </p>
      <h1
        id="not-found-title"
        className="mt-3 text-3xl font-semibold text-[var(--text)]"
      >
        That page does not exist
      </h1>
      <Link
        to="/"
        className="mt-6 inline-flex min-h-10 items-center rounded-lg bg-[var(--accent)] px-4 text-sm font-semibold text-[var(--accent-contrast)] outline-none focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] focus-visible:ring-offset-2"
      >
        Go to Dashboard
      </Link>
    </section>
  );
}
