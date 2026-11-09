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
  boundariesConfig({ repoRoot, uiDir }),
);
