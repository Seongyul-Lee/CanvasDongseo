// 샘플 구현(commands/sample): 실제 구현과 같은 모양(Core)으로, 고른 상태의 샘플을 command 응답으로 돌려주고
// 이벤트 파일을 순서대로 흘려보낸다. Vite·DOM을 모른다(조립은 index.ts)
import type { Core, Unlisten } from "../types.ts";
import type { Catalog, SampleState } from "./catalog.ts";

export interface SampleCoreOptions {
  catalog: Catalog;
  /** 기능 → 고른 상태 */
  selected: Record<string, string>;
  /** command 이름 → 응답으로 돌려줄 ViewModel의 기능 */
  responses: Record<string, string>;
}

type Handler = (payload: unknown) => void;

export function createSampleCore({ catalog, selected, responses }: SampleCoreOptions): Core {
  const states = new Map<string, SampleState>();
  for (const feature of catalog) {
    const state = feature.states.find((s) => s.name === selected[feature.name]);
    if (state !== undefined) {
      states.set(feature.name, state);
    }
  }
  const handlers = new Map<string, Set<Handler>>();
  /** 이벤트를 이미 흘리기 시작한 기능. 다시 구독해도 처음부터 다시 흘리지 않는다 */
  const playing = new Set<string>();

  function play(state: SampleState): void {
    let at = 0;
    for (const step of state.script) {
      at += step.afterMs;
      setTimeout(() => {
        for (const handler of handlers.get(step.event) ?? []) {
          handler(structuredClone(step.payload));
        }
      }, at);
    }
  }

  return {
    async invoke<T>(command: string): Promise<T> {
      const feature = Object.hasOwn(responses, command) ? responses[command] : undefined;
      if (feature === undefined) {
        throw new Error(`샘플 모드에 없는 command: ${command}`);
      }
      const state = states.get(feature);
      if (state === undefined) {
        throw new Error(`command ${command}의 샘플(design/samples/${feature}/)이 없다`);
      }
      // 실제 응답처럼 부를 때마다 새 값을 준다
      return structuredClone(state.viewModel) as T;
    },

    async listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
      const listeners = handlers.get(event) ?? new Set<Handler>();
      handlers.set(event, listeners);
      listeners.add(handler as Handler);
      // 받을 쪽이 생긴 뒤에 흘리기 시작한다. 먼저 흘리면 구독 전에 지나간 이벤트를 놓친다
      for (const [feature, state] of states) {
        if (!playing.has(feature) && state.script.some((step) => step.event === event)) {
          playing.add(feature);
          play(state);
        }
      }
      return () => {
        listeners.delete(handler as Handler);
      };
    },
  };
}
