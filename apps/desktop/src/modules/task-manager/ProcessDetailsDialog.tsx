import { useEffect, useState } from "react";

import { api } from "../../api";
import { Dialog } from "../../components/Dialog";
import { StateMessage } from "../../components/StateMessage";
import type {
  TaskManagerProcessDetails,
  TaskManagerProcessIdentity,
} from "../../generated";
import { formatBytes, formatCount, formatState } from "./format";

function Value({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-lg bg-[var(--background)] p-3">
      <dt className="text-xs font-medium text-[var(--text-muted)]">{label}</dt>
      <dd className="mt-1 break-words text-sm">{value}</dd>
    </div>
  );
}

function Details({ details }: { details: TaskManagerProcessDetails }) {
  const memory = details.memory;
  return (
    <div className="space-y-6">
      {details.restricted_fields.length ? (
        <StateMessage tone="warning" title="Some fields are restricted">
          Linux did not permit access to: {details.restricted_fields.join(", ")}
          .
        </StateMessage>
      ) : null}

      <section aria-labelledby="process-identity-heading">
        <h3 id="process-identity-heading" className="font-semibold">
          Identity and scheduling
        </h3>
        <dl className="mt-3 grid gap-2 sm:grid-cols-2">
          <Value label="PID" value={formatCount(details.identity.pid)} />
          <Value label="Parent PID" value={formatCount(details.parent_pid)} />
          <Value label="State" value={formatState(details.state.kind)} />
          <Value
            label="User ID"
            value={details.uid?.toString() ?? "Unavailable"}
          />
          <Value label="Threads" value={formatCount(details.thread_count)} />
          <Value label="Nice value" value={details.nice.toString()} />
          <Value
            label="Voluntary context switches"
            value={
              details.voluntary_context_switches?.toLocaleString() ??
              "Unavailable"
            }
          />
          <Value
            label="Involuntary context switches"
            value={
              details.involuntary_context_switches?.toLocaleString() ??
              "Unavailable"
            }
          />
        </dl>
      </section>

      <section aria-labelledby="process-memory-heading">
        <h3 id="process-memory-heading" className="font-semibold">
          Memory
        </h3>
        <dl className="mt-3 grid gap-2 sm:grid-cols-2">
          <Value label="Resident" value={formatBytes(memory.resident_bytes)} />
          <Value
            label="Peak resident"
            value={formatBytes(memory.peak_resident_bytes)}
          />
          <Value
            label="Anonymous resident"
            value={formatBytes(memory.resident_anonymous_bytes)}
          />
          <Value
            label="File resident"
            value={formatBytes(memory.resident_file_bytes)}
          />
          <Value
            label="Shared resident"
            value={formatBytes(memory.resident_shared_bytes)}
          />
          <Value label="Swap" value={formatBytes(memory.swap_bytes)} />
          <Value label="Virtual" value={formatBytes(memory.virtual_bytes)} />
          <Value
            label="Peak virtual"
            value={formatBytes(memory.peak_virtual_bytes)}
          />
        </dl>
      </section>

      <section aria-labelledby="process-command-heading">
        <h3 id="process-command-heading" className="font-semibold">
          Command and placement
        </h3>
        <dl className="mt-3 grid gap-2">
          <Value
            label="Executable"
            value={details.executable ?? "Unavailable"}
          />
          <Value
            label="Command line"
            value={details.command_line ?? "Unavailable"}
          />
          <Value
            label="Control groups"
            value={
              details.cgroups.length
                ? details.cgroups.join(" · ")
                : "Unavailable"
            }
          />
        </dl>
      </section>

      <section aria-labelledby="process-io-heading">
        <h3 id="process-io-heading" className="font-semibold">
          Cumulative I/O
        </h3>
        {details.io ? (
          <dl className="mt-3 grid gap-2 sm:grid-cols-2">
            <Value
              label="Physical read"
              value={formatBytes(details.io.read_bytes)}
            />
            <Value
              label="Physical written"
              value={formatBytes(details.io.write_bytes)}
            />
            <Value
              label="Characters read"
              value={formatBytes(details.io.characters_read)}
            />
            <Value
              label="Characters written"
              value={formatBytes(details.io.characters_written)}
            />
            <Value
              label="Read syscalls"
              value={formatCount(details.io.read_syscalls)}
            />
            <Value
              label="Write syscalls"
              value={formatCount(details.io.write_syscalls)}
            />
          </dl>
        ) : (
          <p className="mt-2 text-sm text-[var(--text-muted)]">
            Unavailable for this process.
          </p>
        )}
      </section>
    </div>
  );
}

export function ProcessDetailsDialog({
  identity,
  name,
  onClose,
}: {
  identity: TaskManagerProcessIdentity;
  name: string;
  onClose: () => void;
}) {
  const [details, setDetails] = useState<TaskManagerProcessDetails>();
  const [error, setError] = useState(false);

  useEffect(() => {
    let active = true;
    void api.taskManager
      .processDetails(identity)
      .then((value) => {
        if (active) setDetails(value);
      })
      .catch(() => {
        if (active) setError(true);
      });
    return () => {
      active = false;
    };
  }, [identity]);

  return (
    <Dialog title={`${name} details`} onClose={onClose}>
      {error ? (
        <StateMessage tone="error" title="Process details are unavailable">
          The process may have exited, changed identity, or restricted access.
          Close this panel and refresh the list.
        </StateMessage>
      ) : details ? (
        <Details details={details} />
      ) : (
        <StateMessage tone="loading" title="Reading process details…" />
      )}
    </Dialog>
  );
}
