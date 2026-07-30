import type { AppError, AppErrorCode, AppErrorDetails } from "../generated";

const FRONTEND_FALLBACK_CORRELATION_ID = "00000000-0000-4000-8000-000000000000";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function optionalString(
  record: Record<string, unknown>,
  key: string,
): string | undefined {
  const value = record[key];
  return typeof value === "string" ? value : undefined;
}

function normalizeDetails(value: unknown): AppErrorDetails | undefined {
  if (!isRecord(value)) {
    return undefined;
  }

  const field = optionalString(value, "field");
  const scope = optionalString(value, "scope");
  const moduleId = optionalString(value, "module_id");
  const targetDisplay = optionalString(value, "target_display");
  const sideEffect = value["side_effect_may_have_occurred"];

  return {
    ...(field === undefined ? {} : { field }),
    ...(scope === undefined ? {} : { scope }),
    ...(moduleId === undefined ? {} : { module_id: moduleId }),
    ...(targetDisplay === undefined ? {} : { target_display: targetDisplay }),
    ...(typeof sideEffect === "boolean"
      ? { side_effect_may_have_occurred: sideEffect }
      : {}),
  };
}

/** Error object that preserves the generated contract fields for feature handling. */
export class ApiError extends Error implements AppError {
  readonly code: AppErrorCode;
  readonly details?: AppErrorDetails | null;
  readonly retryable: boolean;
  readonly correlation_id: string;

  constructor(contract: AppError) {
    super(contract.message);
    this.name = "ApiError";
    this.code = contract.code;
    this.retryable = contract.retryable;
    this.correlation_id = contract.correlation_id;
    if (contract.details !== undefined) {
      this.details = contract.details;
    }
  }
}

/** Converts a trusted backend rejection to the generated shape without exposing unknown values. */
export function normalizeAppError(value: unknown): ApiError {
  if (
    isRecord(value) &&
    typeof value["code"] === "string" &&
    typeof value["message"] === "string" &&
    typeof value["retryable"] === "boolean" &&
    typeof value["correlation_id"] === "string"
  ) {
    const details = normalizeDetails(value["details"]);
    return new ApiError({
      code: value["code"] as AppErrorCode,
      message: value["message"],
      retryable: value["retryable"],
      correlation_id: value["correlation_id"],
      ...(details === undefined ? {} : { details }),
    });
  }

  return new ApiError({
    code: "CORE_INTERNAL",
    message: "Argos could not complete the request.",
    retryable: false,
    correlation_id: FRONTEND_FALLBACK_CORRELATION_ID,
  });
}
