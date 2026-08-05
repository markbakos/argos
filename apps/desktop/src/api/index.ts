import { createCoreApi } from "./core";
import { createTaskManagerApi } from "./taskManager";
import { tauriTransport, type Transport } from "./transport/tauri";

export function createApi(transport: Transport) {
  return {
    core: createCoreApi(transport),
    taskManager: createTaskManagerApi(transport),
  };
}

export type Api = ReturnType<typeof createApi>;

export const api: Api = createApi(tauriTransport);
