import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Unlisten = () => void;

export interface Transport {
  invoke<T>(
    command: string,
    decode: (value: unknown) => T,
    arguments_?: Record<string, unknown>,
  ): Promise<T>;
  listen<T>(
    event: string,
    decode: (value: unknown) => T,
    handler: (payload: T) => void,
  ): Promise<Unlisten>;
}

export const tauriTransport: Transport = {
  invoke<T>(
    command: string,
    decode: (value: unknown) => T,
    arguments_?: Record<string, unknown>,
  ) {
    return invoke<unknown>(command, arguments_).then(decode);
  },
  listen<T>(
    event: string,
    decode: (value: unknown) => T,
    handler: (payload: T) => void,
  ) {
    return listen<unknown>(event, ({ payload }) => {
      handler(decode(payload));
    });
  },
};
