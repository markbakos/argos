import { QueryClient } from "@tanstack/react-query";

import { ApiError } from "../api/errors";

const QUERY_STALE_TIME_MS = 30_000;
const QUERY_CACHE_TIME_MS = 5 * 60_000;
const MAX_QUERY_RETRIES = 2;

function shouldRetryQuery(failureCount: number, error: Error) {
  return (
    error instanceof ApiError &&
    error.retryable &&
    failureCount < MAX_QUERY_RETRIES
  );
}

export function createAppQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        gcTime: QUERY_CACHE_TIME_MS,
        refetchOnWindowFocus: false,
        retry: shouldRetryQuery,
        staleTime: QUERY_STALE_TIME_MS,
      },
      mutations: {
        retry: false,
      },
    },
  });
}
