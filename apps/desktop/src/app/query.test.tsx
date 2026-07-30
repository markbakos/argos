import { QueryClientProvider, useQuery } from "@tanstack/react-query";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { ApiError } from "../api/errors";
import { createAppQueryClient } from "./query";

afterEach(cleanup);

function retryableError() {
  return new ApiError({
    code: "SYSTEMD_TIMEOUT",
    message: "The request timed out.",
    retryable: true,
    correlation_id: "38326a30-09d2-45c1-b96a-65d5104f161e",
  });
}

function CancellationProbe({
  onStart,
}: {
  onStart: (signal: AbortSignal) => void;
}) {
  const query = useQuery({
    queryKey: ["cancellation-probe"],
    queryFn: ({ signal }) => {
      onStart(signal);
      return new Promise<string>((_resolve, reject) => {
        signal.addEventListener("abort", () => {
          reject(new DOMException("Query cancelled", "AbortError"));
        });
      });
    },
  });

  return <output>{query.isPending ? "Loading" : "Finished"}</output>;
}

describe("query defaults", () => {
  it("retries only bounded retryable API errors", () => {
    const client = createAppQueryClient();
    const retry = client.getDefaultOptions().queries?.retry;

    if (typeof retry !== "function") {
      throw new Error("The query retry policy must be a function.");
    }

    expect(retry(0, retryableError())).toBe(true);
    expect(retry(1, retryableError())).toBe(true);
    expect(retry(2, retryableError())).toBe(false);
    expect(
      retry(
        0,
        new ApiError({
          code: "VALIDATION_INVALID_FORMAT",
          message: "The request is invalid.",
          retryable: false,
          correlation_id: "38326a30-09d2-45c1-b96a-65d5104f161e",
        }),
      ),
    ).toBe(false);
    expect(retry(0, new Error("unknown"))).toBe(false);
  });

  it("aborts an observed query when its route consumer unmounts", async () => {
    const client = createAppQueryClient();
    let querySignal: AbortSignal | undefined;
    const result = render(
      <QueryClientProvider client={client}>
        <CancellationProbe
          onStart={(signal) => {
            querySignal = signal;
          }}
        />
      </QueryClientProvider>,
    );

    expect(screen.getByText("Loading")).toBeTruthy();
    await waitFor(() => {
      expect(querySignal).toBeDefined();
    });

    result.unmount();

    await waitFor(() => {
      expect(querySignal?.aborted).toBe(true);
    });
  });
});
