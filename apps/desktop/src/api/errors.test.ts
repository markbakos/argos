import { describe, expect, it } from "vitest";

import { normalizeAppError } from "./errors";

describe("normalizeAppError", () => {
  it("preserves a typed backend error and allowlisted details", () => {
    const normalized = normalizeAppError({
      code: "SYSTEMD_TIMEOUT",
      message: "The systemd request timed out.",
      details: { scope: "user", unknown: "discarded" },
      retryable: true,
      correlation_id: "38326a30-09d2-45c1-b96a-65d5104f161e",
    });

    expect(normalized).toBeInstanceOf(Error);
    expect(normalized.code).toBe("SYSTEMD_TIMEOUT");
    expect(normalized.message).toBe("The systemd request timed out.");
    expect(normalized.details).toEqual({ scope: "user" });
    expect(normalized.retryable).toBe(true);
    expect(normalized.correlation_id).toBe(
      "38326a30-09d2-45c1-b96a-65d5104f161e",
    );
  });

  it("replaces an unknown rejection without disclosing it", () => {
    const normalized = normalizeAppError(
      new Error("database=/home/user/private.db token=secret"),
    );

    expect(normalized.code).toBe("CORE_INTERNAL");
    expect(JSON.stringify(normalized)).not.toContain("private.db");
    expect(JSON.stringify(normalized)).not.toContain("secret");
  });
});
