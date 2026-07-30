import { useEffect, useState } from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { BoundaryProof, EventEnvelope } from "../generated";
import { createApi } from ".";
import type { Transport } from "./transport/tauri";

const proof: BoundaryProof = {
  message: "Argos typed boundary is ready.",
  correlation_id: "38326a30-09d2-45c1-b96a-65d5104f161e",
};

function ProofConsumer({ transport }: { transport: Transport }) {
  const [message, setMessage] = useState("Waiting");

  useEffect(() => {
    let active = true;
    void createApi(transport)
      .core.proveBoundary()
      .then((result) => {
        if (active) {
          setMessage(result.message);
        }
      });
    return () => {
      active = false;
    };
  }, [transport]);

  return <output>{message}</output>;
}

describe("core API", () => {
  it("lets a component reach the typed command only through the facade", async () => {
    const commands: string[] = [];
    const transport: Transport = {
      invoke<T>(command: string, decode: (value: unknown) => T) {
        commands.push(command);
        return Promise.resolve(decode(proof));
      },
      listen() {
        return Promise.resolve(() => undefined);
      },
    };

    render(<ProofConsumer transport={transport} />);

    expect(
      await screen.findByText("Argos typed boundary is ready."),
    ).toBeTruthy();
    expect(commands).toEqual(["core_boundary_proof"]);
  });

  it("unwraps a typed event and returns its teardown", async () => {
    const events: string[] = [];
    let stopped = false;
    const envelope: EventEnvelope<BoundaryProof> = {
      schema_version: 1,
      correlation_id: proof.correlation_id,
      payload: proof,
    };
    const transport: Transport = {
      invoke<T>(_command: string, decode: (value: unknown) => T) {
        return Promise.resolve(decode(proof));
      },
      listen<T>(
        event: string,
        decode: (value: unknown) => T,
        handler: (payload: T) => void,
      ) {
        events.push(event);
        handler(decode(envelope));
        return Promise.resolve(() => {
          stopped = true;
        });
      },
    };
    const received: EventEnvelope<BoundaryProof>[] = [];
    const unlisten = await createApi(transport).core.onBoundaryProof(
      (event) => {
        received.push(event);
      },
    );

    expect(events).toEqual(["core://boundary-proof"]);
    expect(received).toEqual([envelope]);
    unlisten();
    expect(stopped).toBe(true);
  });
});
