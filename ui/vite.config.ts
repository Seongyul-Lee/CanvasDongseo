import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";
import { forbidSamplesInBuild } from "./sample-guard.js";

const uiDir = import.meta.dirname;
const repoRoot = path.resolve(uiDir, "..");

const samplesDir = path.join(repoRoot, "design", "samples");

// 샘플 모드(presentation §2.4): design/samples/는 ui/ 밖이라 개발 서버가 폴더를 지켜보지 않는다.
// 지켜보게 해야 상태 파일을 더하거나 지웠을 때 다시 불러온다
const watchSamples: Plugin = {
  name: "watch-samples",
  apply: "serve",
  configureServer(server) {
    server.watcher.add(samplesDir);
  },
};

// Tauri가 devUrl(http://localhost:5173)로 개발 서버를 띄우므로 포트를 고정한다
export default defineConfig({
  plugins: [react(), tailwindcss(), watchSamples, forbidSamplesInBuild({ repoRoot, uiDir })],
  // T3 Code에서 가져온 부품이 ~/lib/utils처럼 부른다. 가져온 파일을 고치지 않으려고 같은 별칭을 둔다(tsconfig.json의 paths와 같다)
  resolve: {
    alias: { "~": path.join(uiDir, "src") },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    // 샘플 모드(presentation §2.4)가 ui/ 밖의 design/samples/를 읽는다
    fs: { allow: [uiDir, samplesDir] },
  },
});
