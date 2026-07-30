import type { BoundaryProof, EventEnvelope } from "../generated";
import { normalizeAppError } from "./errors";
import type { Transport, Unlisten } from "./transport/tauri";

const BOUNDARY_PROOF_COMMAND = "core_boundary_proof";
const BOUNDARY_PROOF_EVENT = "core://boundary-proof";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function decodeBoundaryProof(value: unknown): BoundaryProof {
  if (
    !isRecord(value) ||
    typeof value["message"] !== "string" ||
    typeof value["correlation_id"] !== "string"
  ) {
    throw new Error("The boundary proof response is invalid.");
  }

  return {
    message: value["message"],
    correlation_id: value["correlation_id"],
  };
}

function decodeBoundaryProofEvent(
  value: unknown,
): EventEnvelope<BoundaryProof> {
  if (
    !isRecord(value) ||
    typeof value["schema_version"] !== "number" ||
    typeof value["correlation_id"] !== "string"
  ) {
    throw new Error("The boundary proof event is invalid.");
  }

  return {
    schema_version: value["schema_version"],
    correlation_id: value["correlation_id"],
    payload: decodeBoundaryProof(value["payload"]),
  };
}

function normalizeFailure<T>(promise: Promise<T>): Promise<T> {
  return promise.catch((error: unknown) =>
    Promise.reject(normalizeAppError(error)),
  );
}

export interface CoreApi {
  proveBoundary(): Promise<BoundaryProof>;
  onBoundaryProof(
    handler: (event: EventEnvelope<BoundaryProof>) => void,
  ): Promise<Unlisten>;
}

export function createCoreApi(transport: Transport): CoreApi {
  return {
    proveBoundary() {
      return normalizeFailure(
        transport.invoke(BOUNDARY_PROOF_COMMAND, decodeBoundaryProof),
      );
    },
    onBoundaryProof(handler) {
      return normalizeFailure(
        transport.listen<EventEnvelope<BoundaryProof>>(
          BOUNDARY_PROOF_EVENT,
          decodeBoundaryProofEvent,
          handler,
        ),
      );
    },
  };
}
