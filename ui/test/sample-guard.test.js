// 배포 빌드에 샘플 구현·샘플 JSON이 들어가면 빌드가 실패하는지(sample-guard.js) 실제 번들러로 확인한다.
// 픽스처는 저장소 모양(ui/src/commands/sample, design/samples)을 흉내 낸 test/fixtures/guard에 있다.
import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { build } from "vite";
import { forbidSamplesInBuild } from "../sample-guard.js";

const repoRoot = path.join(import.meta.dirname, "fixtures", "guard");
const uiDir = path.join(repoRoot, "ui");

/**
 * 픽스처의 파일 하나를 엔트리로 배포 빌드를 한다. 결과는 디스크에 쓰지 않는다
 * @param {string} entry src/ 기준 경로
 */
async function buildEntry(entry) {
  const result = await build({
    configFile: false,
    root: uiDir,
    logLevel: "silent",
    plugins: [forbidSamplesInBuild({ repoRoot, uiDir })],
    build: { write: false, rollupOptions: { input: path.join(uiDir, "src", entry) } },
  });
  return code(result);
}

/**
 * 빌드 결과의 코드 조각들
 * @param {Awaited<ReturnType<typeof build>>} result
 */
function chunks(result) {
  return [result]
    .flat()
    .flatMap((r) => ("output" in r ? r.output : []))
    .flatMap((output) => (output.type === "chunk" ? [output] : []));
}

/**
 * 빌드 결과 조각의 코드를 모두 이어 붙인다
 * @param {Awaited<ReturnType<typeof build>>} result
 */
function code(result) {
  return chunks(result)
    .map((chunk) => chunk.code)
    .join("\n");
}

/**
 * 환경 변수를 켠 채로 빌드해도 샘플이 들어가지 않아야 한다
 * @param {import("node:test").TestContext} t
 */
function withSampleModeEnv(t) {
  const before = process.env.VITE_SAMPLE_MODE;
  process.env.VITE_SAMPLE_MODE = "1";
  t.after(() => {
    if (before === undefined) delete process.env.VITE_SAMPLE_MODE;
    else process.env.VITE_SAMPLE_MODE = before;
  });
}

test("샘플 구현을 정적으로 불러오면 빌드가 실패한다", async () => {
  await assert.rejects(buildEntry("static-impl.ts"), /src\/commands\/sample\/demo\.ts/);
});

test("샘플 JSON을 불러오면 빌드가 실패한다", async () => {
  await assert.rejects(buildEntry("static-json.ts"), /design\/samples\/demo\/ready\.json/);
});

test("조건 없는 동적 import로 불러도 빌드가 실패한다", async () => {
  await assert.rejects(buildEntry("dynamic-always.ts"), /src\/commands\/sample\/demo\.ts/);
});

test("개발 서버에서만 실행되는 동적 import는 배포 빌드에서 빠진다", async (t) => {
  withSampleModeEnv(t);
  const output = await buildEntry("dev-only.ts");
  assert.ok(output.includes("real"), "엔트리가 빌드되지 않았다");
  assert.ok(!output.includes("GUARD_SAMPLE_IMPL_MARKER"), "샘플 구현이 결과에 들어갔다");
});

test("실제 프로젝트: 코어 통로(commands/core.ts)를 넣어 배포 빌드를 해도 샘플이 없다", async (t) => {
  withSampleModeEnv(t);
  const realUiDir = path.resolve(import.meta.dirname, "..");
  // 지금은 예시 엔트리만 코어 통로를 부른다. 예시를 지우면 index.html만 남긴다(design/samples/example/README.md)
  const result = await build({
    root: realUiDir,
    configFile: path.join(realUiDir, "vite.config.ts"),
    logLevel: "silent",
    build: {
      write: false,
      rollupOptions: { input: ["index.html", "example.html"].map((file) => path.join(realUiDir, file)) },
    },
  });

  // vite.config.ts에 가드가 걸려 있으므로, 빌드가 끝났다는 것이 곧 샘플 모듈이 결과에 없다는 뜻이다
  const moduleIds = chunks(result).flatMap((chunk) => chunk.moduleIds);
  assert.ok(
    moduleIds.some((id) => id.endsWith("/src/commands/core.ts")),
    "코어 통로가 빌드에 들어가지 않아 아무것도 확인하지 못했다",
  );
  assert.ok(!code(result).includes("샘플 모드에 없는 command"), "샘플 구현이 결과에 들어갔다");
});
