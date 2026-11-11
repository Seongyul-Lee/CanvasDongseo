// 폴더 의존 규칙(presentation §2.1)이 위반을 실제로 잡는지 픽스처로 확인한다.
// 픽스처는 저장소 모양(ui/src, design/samples, ui/ 밖 폴더)을 흉내 낸 test/fixtures/repo에 있다.
import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { ESLint } from "eslint";
import tseslint from "typescript-eslint";
import { boundariesConfig } from "../boundaries.config.js";

const repoRoot = path.join(import.meta.dirname, "fixtures", "repo");
const uiDir = path.join(repoRoot, "ui");

const eslint = new ESLint({
  cwd: uiDir,
  overrideConfigFile: true,
  overrideConfig: [
    { files: ["**/*.{ts,tsx}"], languageOptions: { parser: tseslint.parser } },
    boundariesConfig({ repoRoot, uiDir }),
  ],
});

/** @param {string} file ui/ 기준 경로 */
async function lint(file) {
  const [result] = await eslint.lintFiles([path.join(uiDir, file)]);
  assert.ok(result, `${file}을 검사하지 못했다`);
  return result.messages.map((m) => `${m.ruleId}: ${m.message}`);
}

const allowed = [
  "src/main.tsx",
  "src/viewmodels/course.ts",
  "src/viewmodels/banner.ts",
  "src/components/Card.tsx",
  "src/components/ui/Tip.tsx",
  "src/lib/cn.ts",
  "src/lib/merge.ts",
  "src/hooks/useThing.ts",
  "src/screens/Home.tsx",
  "src/stores/course.ts",
  "src/commands/course.ts",
  "src/commands/sample/course.ts",
];

for (const file of allowed) {
  test(`허용: ${file}`, async () => {
    assert.deepEqual(await lint(file), []);
  });
}

/** 파일 → 기대하는 오류(규칙 이름과 메시지 일부) */
const violations = {
  "src/stray.ts": "boundaries/no-unknown-files",
  "src/theme/bad-imports-screen.ts": 'from elements of type "theme" to elements of type "screens"',
  "src/viewmodels/bad-imports-components.ts": 'from elements of type "viewmodels" to elements of type "components"',
  "src/components/BadStore.tsx": 'from elements of type "components" to elements of type "stores"',
  "src/components/BadCommand.tsx": 'from elements of type "components" to elements of type "commands"',
  "src/components/BadTauri.tsx": "Tauri API는 commands/의 실제 구현만 부른다",
  "src/components/BadUiConfig.tsx": 'from elements of type "components" to entities of module with origin "local"',
  "src/components/BadAliasStore.tsx": 'from elements of type "components" to elements of type "stores"',
  "src/lib/BadStore.ts": 'from elements of type "lib" to elements of type "stores"',
  "src/lib/BadHook.ts": 'from elements of type "lib" to elements of type "hooks"',
  "src/hooks/BadComponent.ts": 'from elements of type "hooks" to elements of type "components"',
  "src/screens/BadCommand.tsx": 'from elements of type "screens" to elements of type "commands"',
  "src/screens/BadViewModel.tsx": 'from elements of type "screens" to elements of type "viewmodels"',
  "src/stores/BadComponent.ts": 'from elements of type "stores" to elements of type "components"',
  "src/stores/BadLib.ts": 'from elements of type "stores" to elements of type "lib"',
  "src/commands/BadStore.ts": 'from elements of type "commands" to elements of type "stores"',
  "src/commands/BadSamples.ts": 'from elements of type "commands" to elements of type "samples"',
  "src/commands/BadOutside.ts": 'from elements of type "commands" to entities of module with origin "local"',
  "src/commands/sample/BadTauri.ts": "Tauri API는 commands/의 실제 구현만 부른다",
};

for (const [file, expected] of Object.entries(violations)) {
  test(`위반: ${file}`, async () => {
    const messages = await lint(file);
    assert.equal(messages.length, 1, `오류가 하나여야 한다: ${JSON.stringify(messages)}`);
    assert.ok(messages[0]?.includes(expected), `기대: ${expected}\n실제: ${messages[0]}`);
  });
}
