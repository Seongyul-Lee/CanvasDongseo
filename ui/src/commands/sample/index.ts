// 샘플 구현의 조립(commands/sample): Vite(import.meta.glob)와 브라우저(주소, DOM)에 묶인 부분만 둔다.
// 판단은 순수 모듈(catalog, selection, sampleCore)이 한다. commands/core.ts가 샘플 모드에서만 동적 import로 부른다
import type { Core } from "../types.ts";
import { buildCatalog } from "./catalog.ts";
import { mountPicker, mountProblem } from "./picker.ts";
import { sampleResponses } from "./responses.ts";
import { createSampleCore } from "./sampleCore.ts";
import { selectStates, withSelection } from "./selection.ts";

// 샘플 파일이나 샘플 구현이 바뀌면 화면을 통째로 다시 불러온다. 고른 상태는 주소에 있어 그대로 남는다.
// 이렇게 받지 않으면 변경이 화면 모듈까지 번져 store가 빈 채로 다시 만들어진다
import.meta.hot?.accept(() => {
  window.location.reload();
});

export function startSampleCore(): Core {
  // design/samples/<기능>/<상태>.json과 <상태>.events.json
  const files = import.meta.glob("../../../../design/samples/*/*.json", { eager: true, import: "default" });
  try {
    const catalog = buildCatalog(files);
    const selected = selectStates(catalog, window.location.search);
    mountPicker(document, catalog, selected, (feature, state) => {
      // 고른 상태를 주소에 담고 새로 불러온다. 상태마다 주소가 생겨 캡처하고 다시 열기 쉽다
      window.location.search = withSelection(window.location.search, feature, state);
    });
    return createSampleCore({ catalog, selected, responses: sampleResponses });
  } catch (error) {
    mountProblem(document, error instanceof Error ? error.message : String(error));
    throw new Error("샘플 모드를 시작하지 못했다", { cause: error });
  }
}
