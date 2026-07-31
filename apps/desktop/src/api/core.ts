import type {
  BoundaryProof,
  EventEnvelope,
  SystemIdentity,
} from "../generated";
import { normalizeAppError } from "./errors";
import type { Transport, Unlisten } from "./transport/tauri";

const BOUNDARY_PROOF_COMMAND = "core_boundary_proof";
const BOUNDARY_PROOF_EVENT = "core://boundary-proof";
const SYSTEM_IDENTITY_COMMAND = "core_get_system_identity";
const HOSTNAME_MAX_BYTES = 64;

function containsControlCharacter(value: string) {
  return Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0);
    return (
      codePoint !== undefined &&
      (codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f))
    );
  });
}

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

function decodeSystemIdentity(value: unknown): SystemIdentity {
  if (
    !isRecord(value) ||
    typeof value["hostname"] !== "string" ||
    value["hostname"].length === 0 ||
    new TextEncoder().encode(value["hostname"]).length > HOSTNAME_MAX_BYTES ||
    containsControlCharacter(value["hostname"])
  ) {
    throw new Error("The system identity response is invalid.");
  }

  return { hostname: value["hostname"] };
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
  getSystemIdentity(): Promise<SystemIdentity>;
  proveBoundary(): Promise<BoundaryProof>;
  onBoundaryProof(
    handler: (event: EventEnvelope<BoundaryProof>) => void,
  ): Promise<Unlisten>;
}

export function createCoreApi(transport: Transport): CoreApi {
  return {
    getSystemIdentity() {
      return normalizeFailure(
        transport.invoke(SYSTEM_IDENTITY_COMMAND, decodeSystemIdentity),
      );
    },
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
