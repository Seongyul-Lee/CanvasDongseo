// 배포 빌드에 샘플 구현(ui/src/commands/sample/)이나 샘플 JSON(design/samples/)이 들어가면 빌드를 실패시킨다(presentation §2.4).
// 실제 설정(vite.config.ts)과 테스트(test/sample-guard.test.js)가 같은 함수를 쓴다.
import path from "node:path";
import { normalizePath } from "vite";

/**
 * @param {{ repoRoot: string, uiDir: string }} paths 저장소 루트와 ui/ 폴더의 절대 경로
 * @returns {import("vite").Plugin}
 */
export function forbidSamplesInBuild({ repoRoot, uiDir }) {
  const forbidden = [path.join(uiDir, "src", "commands", "sample"), path.join(repoRoot, "design", "samples")].map(
    (dir) => `${normalizePath(dir)}/`,
  );

  return {
    name: "forbid-samples-in-build",
    // 샘플 모드는 개발 서버에서만 쓴다. 빌드에만 건다
    apply: "build",
    generateBundle(_options, bundle) {
      // 결과 조각에 실제로 들어간 모듈만 본다(지워진 갈래의 모듈은 결과에 없다)
      const found = Object.values(bundle)
        .flatMap((output) => (output.type === "chunk" ? output.moduleIds : []))
        // 모듈 ID에는 `?url` 같은 쿼리가 붙을 수 있다
        .map((id) => normalizePath(id).replace(/[?#].*$/, ""))
        .filter((file) => forbidden.some((dir) => file.startsWith(dir)));
      if (found.length > 0) {
        const files = [...new Set(found)].map((file) => `  ${path.relative(repoRoot, file)}`).join("\n");
        this.error(
          `배포 빌드에 샘플이 들어갔다. 샘플 구현은 commands/core.ts의 개발 서버 전용 동적 import로만 부른다(presentation §2.4):\n${files}`,
        );
      }
    },
  };
}
