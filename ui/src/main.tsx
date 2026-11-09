// 엔트리: 테마를 불러오고 첫 화면을 붙인다. 로직을 두지 않는다
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./theme/app.css";
import { HomeScreen } from "./screens/HomeScreen.tsx";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("#root 요소가 없다");
}

createRoot(root).render(
  <StrictMode>
    <HomeScreen />
  </StrictMode>,
);
