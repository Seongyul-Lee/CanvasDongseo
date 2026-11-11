// 가져온 테마(t3code.css)가 앱 설정과 맞는지 확인한다.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

const uiDir = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(uiDir, "..");

// t3code.css의 바탕 결 무늬(--surface-grain)는 data: URL 이미지다. CSP가 막으면 무늬가 그려지지 않는데,
// 눈으로는 거의 구분되지 않아(UPSTREAM.md §7) CSP를 고치다 조용히 빠지기 쉽다
test("CSP: 테마가 쓰는 data: 이미지를 허용한다", () => {
  const css = readFileSync(path.join(uiDir, "src/theme/t3code.css"), "utf8");
  assert.match(css, /url\(\s*["']?data:image\//, "t3code.css가 data: 이미지를 쓰지 않는다. 이 테스트와 CSP의 data: 허용을 함께 지운다");

  const conf = JSON.parse(readFileSync(path.join(repoRoot, "crates/bin/app/tauri.conf.json"), "utf8"));
  /** @type {unknown} */
  const csp = conf.app.security.csp;
  assert.equal(typeof csp, "string", "CSP가 문자열이 아니다. 이 테스트가 읽는 방법을 고친다");

  /** @type {Map<string, string[]>} */
  const directives = new Map();
  for (const directive of String(csp).split(";")) {
    const [name, ...sources] = directive.trim().split(/\s+/);
    if (name) directives.set(name, sources);
  }
  // img-src가 없으면 브라우저는 default-src를 쓴다
  const imageSources = directives.get("img-src") ?? directives.get("default-src") ?? [];
  assert.ok(imageSources.includes("data:"), `CSP가 data: 이미지를 막는다(이미지 출처: ${imageSources.join(" ")})`);
});
