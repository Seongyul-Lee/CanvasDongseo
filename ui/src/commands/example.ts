// 예시 command 래퍼(commands): command 이름과 타입만 정하고 코어 통로(core.ts)에 넘긴다.
// 첫 기능이 들어오면 지운다(design/samples/example/README.md)
import type { ExampleProgressEvent, ExampleViewModel } from "../viewmodels/example.ts";
import { invoke, listen } from "./core.ts";
import type { Unlisten } from "./types.ts";

export function getExample(): Promise<ExampleViewModel> {
  return invoke<ExampleViewModel>("get_example");
}

export function onExampleProgress(handler: (event: ExampleProgressEvent) => void): Promise<Unlisten> {
  return listen<ExampleProgressEvent>("example://progress", handler);
}
