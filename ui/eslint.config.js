import path from "node:path";
import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import { boundariesConfig } from "./boundaries.config.js";

const uiDir = import.meta.dirname;
const repoRoot = path.resolve(uiDir, "..");

export default tseslint.config(
  { ignores: ["dist/**", "test/fixtures/**"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
  },
  {
    files: ["*.{js,ts}", "test/**/*.js"],
    languageOptions: { globals: globals.node },
  },
  {
    // T3 Code에서 가져온 부품은 고치지 않는다(UPSTREAM.md). 원본은 `...props`에서 속성을 빼려고
    // 쓰지 않는 변수를 밑줄로 시작하는 이름으로 둔다
    files: ["src/components/ui/**/*.{ts,tsx}"],
    rules: {
      "@typescript-eslint/no-unused-vars": ["error", { ignoreRestSiblings: true, argsIgnorePattern: "^_" }],
    },
  },
  boundariesConfig({ repoRoot, uiDir }),
);
