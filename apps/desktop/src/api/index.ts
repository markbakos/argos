import { createCoreApi } from "./core";
import { tauriTransport, type Transport } from "./transport/tauri";

export function createApi(transport: Transport) {
  return {
    core: createCoreApi(transport),
  };
}

export type Api = ReturnType<typeof createApi>;

export const api: Api = createApi(tauriTransport);
