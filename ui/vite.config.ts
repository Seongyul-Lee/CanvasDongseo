import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tauri가 devUrl(http://localhost:5173)로 개발 서버를 띄우므로 포트를 고정한다
export default defineConfig({
  plugins: [react(), tailwindcss()],
  // T3 Code에서 가져온 부품이 ~/lib/utils처럼 부른다. 가져온 파일을 고치지 않으려고 같은 별칭을 둔다(tsconfig.json의 paths와 같다)
  resolve: {
    alias: { "~": path.join(import.meta.dirname, "src") },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
});
