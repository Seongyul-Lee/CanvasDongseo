// 코어를 부르는 통로(commands). 실제 구현(tauri.ts)과 샘플 구현(sample/) 중 어느 쪽을 붙일지 여기서만 고른다.
// command 래퍼·stores·screens는 어느 쪽이 붙었는지 모른다(presentation §2.4)
import { tauriCore } from "./tauri.ts";
import type { Core, Unlisten } from "./types.ts";

// 샘플 구현은 개발 서버에서 VITE_SAMPLE_MODE=1일 때만 동적 import로 부른다. 배포 빌드에서는 이 갈래가 지워져
// 샘플 구현과 design/samples/가 결과에 들어가지 않는다(들어가면 sample-guard.js가 빌드를 실패시킨다)
const core: Promise<Core> =
  import.meta.env.DEV && import.meta.env.VITE_SAMPLE_MODE === "1"
    ? import("./sample/index.ts").then((sample) => sample.startSampleCore())
    : Promise.resolve(tauriCore);

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return (await core).invoke<T>(command, args);
}

export async function listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
  return (await core).listen<T>(event, handler);
}
