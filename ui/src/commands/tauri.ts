// 실제 구현(commands): Tauri의 command 호출과 이벤트 구독을 그대로 잇는다. 로직을 두지 않는다
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Core } from "./types.ts";

export const tauriCore: Core = {
  invoke: <T>(command: string, args?: Record<string, unknown>) => invoke<T>(command, args),
  listen: <T>(event: string, handler: (payload: T) => void) => listen<T>(event, (e) => handler(e.payload)),
};
