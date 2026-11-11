// 샘플 모드 예시의 엔트리(example.html). 배포 빌드의 입력(index.html)이 아니라 결과에 들어가지 않는다.
// 첫 기능이 들어오면 지운다(design/samples/example/README.md)
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./theme/app.css";
import { ExampleScreen } from "./screens/ExampleScreen.tsx";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("#root 요소가 없다");
}

createRoot(root).render(
  <StrictMode>
    <ExampleScreen />
  </StrictMode>,
);
