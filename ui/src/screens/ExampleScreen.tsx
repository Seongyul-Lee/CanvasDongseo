// 예시 화면(screens): store를 구독해 컴포넌트에 넘기기만 한다.
// 첫 기능이 들어오면 지운다(design/samples/example/README.md)
import { useEffect, useSyncExternalStore } from "react";
import { ExampleView } from "../components/ExampleView.tsx";
import { exampleStore } from "../stores/example.ts";

export function ExampleScreen() {
  const { viewModel, progress } = useSyncExternalStore(exampleStore.subscribe, exampleStore.getState);
  useEffect(() => {
    void exampleStore.start();
  }, []);
  return viewModel && <ExampleView viewModel={viewModel} progress={progress} />;
}
