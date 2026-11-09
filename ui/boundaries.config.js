// ui/src 폴더 의존 규칙(presentation §2.1 표)을 eslint-plugin-boundaries 설정으로 만든다.
// 실제 설정(eslint.config.js)과 위반 픽스처 테스트(test/boundaries.test.js)가 같은 함수를 쓴다.
import boundaries from "eslint-plugin-boundaries";

/**
 * @param {{ repoRoot: string, uiDir: string }} paths 저장소 루트와 ui/ 폴더의 절대 경로
 * @returns {import("eslint").Linter.Config}
 */
export function boundariesConfig({ repoRoot, uiDir }) {
  // 패턴은 저장소 루트 기준이다. ui/ 밖은 design/samples/만 요소로 둔다
  /** @param {string} p */
  const ui = (p) => `ui/src/${p}`;
  /** @param {string | string[]} type */
  const to = (type) => ({ to: { element: { types: [type].flat() } } });

  return {
    files: ["src/**/*.{ts,tsx,js,jsx}"],
    plugins: { boundaries },
    settings: {
      "boundaries/root-path": repoRoot,
      "boundaries/elements": [
        { type: "theme", pattern: ui("theme/**"), partialMatch: false },
        { type: "viewmodels", pattern: ui("viewmodels/**"), partialMatch: false },
        { type: "components", pattern: ui("components/**"), partialMatch: false },
        { type: "screens", pattern: ui("screens/**"), partialMatch: false },
        { type: "stores", pattern: ui("stores/**"), partialMatch: false },
        // 샘플 구현(presentation §2.4). commands보다 먼저 둬야 이 타입으로 분류된다
        { type: "commands-sample", pattern: ui("commands/sample/**"), partialMatch: false },
        { type: "commands", pattern: ui("commands/**"), partialMatch: false },
        { type: "samples", pattern: "design/samples/**", partialMatch: false },
      ],
      // 엔트리(main.tsx)는 폴더가 아니라 파일 하나라서 파일 분류로 둔다
      "boundaries/files": [{ pattern: ui("main.tsx"), category: "entry" }],
      "import/resolver": {
        typescript: { project: `${uiDir}/tsconfig.json` },
      },
    },
    rules: {
      // src 안의 파일은 모두 위 폴더 중 하나에 있어야 한다
      "boundaries/no-unknown-files": 2,
      "boundaries/dependencies": [
        2,
        {
          default: "disallow",
          // 요소에 속하지 않는 로컬 파일(ui/ 밖, ui/의 설정 파일 등)로 가는 의존도 검사해 막는다
          checkUnknownLocals: true,
          checkAllOrigins: true,
          policies: [
            // presentation §2.1 표. 같은 폴더 안의 의존은 허용한다
            { from: { file: { categories: "entry" } }, allow: to(["screens", "theme"]) },
            { from: { element: { type: "theme" } }, allow: to("theme") },
            { from: { element: { type: "viewmodels" } }, allow: to("viewmodels") },
            { from: { element: { type: "components" } }, allow: to(["components", "theme", "viewmodels"]) },
            { from: { element: { type: "screens" } }, allow: to(["screens", "components", "stores", "theme"]) },
            { from: { element: { type: "stores" } }, allow: to(["stores", "commands", "viewmodels"]) },
            { from: { element: { type: "commands" } }, allow: to(["commands", "viewmodels"]) },
            // ui/ 밖 참조는 샘플 구현이 design/samples/를 읽는 경우 하나뿐이다
            {
              from: { element: { type: "commands-sample" } },
              allow: to(["commands-sample", "commands", "viewmodels", "samples"]),
            },
            // 외부 패키지는 허용하되, Tauri API는 commands/의 실제 구현만 쓴다
            { allow: { to: { module: { origin: "external" } } } },
            {
              disallow: { to: { module: { origin: "external", source: "@tauri-apps/**" } } },
              message: "Tauri API는 commands/의 실제 구현만 부른다(presentation §2.1)",
            },
            {
              from: { element: { type: "commands" } },
              allow: { to: { module: { origin: "external", source: "@tauri-apps/**" } } },
            },
          ],
        },
      ],
    },
  };
}
