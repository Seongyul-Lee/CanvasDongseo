// 샘플 구현(commands/sample): 기능마다 고른 상태를 주소의 쿼리(sample.<기능>=<상태>)로 읽고 쓴다. 순수 함수만 둔다
import type { Catalog } from "./catalog.ts";

const PARAM_PREFIX = "sample.";

/** 기능 → 고른 상태. 주소에 없거나 없는 이름이면 이름 순 첫 상태다 */
export function selectStates(catalog: Catalog, search: string): Record<string, string> {
  const params = new URLSearchParams(search);
  return Object.fromEntries(
    catalog.flatMap((feature) => {
      const wanted = params.get(PARAM_PREFIX + feature.name);
      const state = feature.states.find((s) => s.name === wanted) ?? feature.states[0];
      return state === undefined ? [] : [[feature.name, state.name]];
    }),
  );
}

/** 고른 상태를 넣은 새 쿼리. 다른 값은 그대로 둔다 */
export function withSelection(search: string, feature: string, state: string): string {
  const params = new URLSearchParams(search);
  params.set(PARAM_PREFIX + feature, state);
  return `?${params.toString()}`;
}
