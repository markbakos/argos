import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "../../api";
import type {
  TaskManagerSnapshot,
  TaskManagerSnapshotRequest,
} from "../../generated";

const SAMPLE_INTERVAL_MS = 2_000;
const HISTORY_LIMIT = 30;

export interface TaskManagerHistoryPoint {
  cpu: number | null;
  memory: number;
}

export function useTaskManager(request: TaskManagerSnapshotRequest) {
  const [snapshot, setSnapshot] = useState<TaskManagerSnapshot>();
  const [history, setHistory] = useState<TaskManagerHistoryPoint[]>([]);
  const [error, setError] = useState<unknown>();
  const [isPending, setIsPending] = useState(true);
  const inFlight = useRef<Promise<TaskManagerSnapshot> | null>(null);
  const requestRef = useRef(request);
  const sampleNow = useRef<() => void>(() => undefined);

  useEffect(() => {
    let active = true;
    let queued = false;
    let queuedFreshBaseline = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    const clearTimer = () => {
      if (timer !== undefined) clearTimeout(timer);
      timer = undefined;
    };
    const canSample = () => active && document.visibilityState === "visible";

    const sample = async (freshBaseline: boolean) => {
      clearTimer();
      if (!canSample()) return;

      const prior = inFlight.current;
      if (prior) {
        queuedFreshBaseline ||= freshBaseline;
        if (queued) return;
        queued = true;
        await prior.catch(() => undefined);
        const nextFreshBaseline = queuedFreshBaseline;
        queued = false;
        queuedFreshBaseline = false;
        if (canSample()) await sample(nextFreshBaseline);
        return;
      }

      const pending = api.taskManager.snapshot({
        ...requestRef.current,
        fresh_baseline: freshBaseline,
      });
      inFlight.current = pending;
      try {
        const next = await pending;
        if (canSample()) {
          setSnapshot(next);
          setError(undefined);
          setIsPending(false);
          const memory = next.memory.total_bytes
            ? (next.memory.used_bytes / next.memory.total_bytes) * 100
            : 0;
          setHistory((current) => [
            ...current.slice(-(HISTORY_LIMIT - 1)),
            { cpu: next.cpu.total_percent ?? null, memory },
          ]);
        }
      } catch (cause) {
        if (canSample()) {
          setError(cause);
          setIsPending(false);
        }
      } finally {
        if (inFlight.current === pending) inFlight.current = null;
      }

      if (canSample()) {
        timer = setTimeout(() => {
          void sample(false);
        }, SAMPLE_INTERVAL_MS);
      }
    };

    sampleNow.current = () => {
      void sample(false);
    };
    const handleVisibility = () => {
      clearTimer();
      if (document.visibilityState === "visible") void sample(true);
    };

    document.addEventListener("visibilitychange", handleVisibility);
    void sample(true);
    return () => {
      active = false;
      sampleNow.current = () => undefined;
      clearTimer();
      document.removeEventListener("visibilitychange", handleVisibility);
    };
  }, []);

  useEffect(() => {
    if (requestRef.current !== request) {
      requestRef.current = request;
      sampleNow.current();
    }
  }, [request]);

  const refresh = useCallback(() => {
    sampleNow.current();
  }, []);

  return { snapshot, history, error, isPending, refresh };
}
