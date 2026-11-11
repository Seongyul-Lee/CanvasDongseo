// 예시 store(stores): 받은 ViewModel과 이벤트를 보관하고 화면에 알린다. 값을 새로 만들지 않는다.
// 첫 기능이 들어오면 지운다(design/samples/example/README.md)
import { getExample, onExampleProgress } from "../commands/example.ts";
import type { ExampleProgressEvent, ExampleViewModel } from "../viewmodels/example.ts";

export interface ExampleState {
  viewModel: ExampleViewModel | null;
  /** 마지막으로 받은 진행 이벤트 */
  progress: ExampleProgressEvent | null;
}

let state: ExampleState = { viewModel: null, progress: null };
const subscribers = new Set<() => void>();
let started: Promise<void> | null = null;

function set(next: ExampleState): void {
  state = next;
  for (const notify of subscribers) {
    notify();
  }
}

export const exampleStore = {
  subscribe(notify: () => void): () => void {
    subscribers.add(notify);
    return () => {
      subscribers.delete(notify);
    };
  },

  getState: (): ExampleState => state,

  /** 진행 이벤트를 구독하고 ViewModel을 받아 온다. 여러 번 불러도 한 번만 시작한다 */
  start(): Promise<void> {
    started ??= (async () => {
      await onExampleProgress((progress) => set({ ...state, progress }));
      const viewModel = await getExample();
      set({ ...state, viewModel });
    })();
    return started;
  },
};
