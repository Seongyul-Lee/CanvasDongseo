// 샘플 구현(commands/sample): 화면 구석에서 기능별 상태를 고르는 패널.
// components·screens가 샘플 모드를 모르게 하려고, React 화면에 넣지 않고 여기서 DOM으로 직접 그린다
import type { Catalog } from "./catalog.ts";

const PANEL_CLASS =
  "fixed right-3 bottom-3 z-[200] max-h-[60vh] overflow-auto rounded-lg border bg-popover px-3 py-2 text-xs text-popover-foreground shadow-lg";

function panel(doc: Document, title: string): HTMLDetailsElement {
  const details = doc.createElement("details");
  details.open = true;
  details.className = PANEL_CLASS;
  const summary = doc.createElement("summary");
  summary.className = "cursor-pointer font-medium select-none";
  summary.textContent = title;
  details.append(summary);
  doc.body.append(details);
  return details;
}

/** 기능마다 상태 목록을 보여 주고, 고르면 `onSelect`를 부른다 */
export function mountPicker(
  doc: Document,
  catalog: Catalog,
  selected: Record<string, string>,
  onSelect: (feature: string, state: string) => void,
): void {
  const details = panel(doc, "샘플 모드");
  for (const feature of catalog) {
    const label = doc.createElement("label");
    label.className = "mt-2 flex items-center justify-between gap-3";
    const name = doc.createElement("span");
    name.className = "text-muted-foreground";
    name.textContent = feature.name;
    const select = doc.createElement("select");
    select.className = "rounded-md border bg-background px-1.5 py-1 text-foreground";
    for (const state of feature.states) {
      const option = doc.createElement("option");
      option.value = state.name;
      // 이벤트를 흘려보내는 상태는 표시해 둔다
      option.textContent = state.script.length > 0 ? `${state.name} ▶` : state.name;
      option.selected = state.name === selected[feature.name];
      select.append(option);
    }
    select.addEventListener("change", () => onSelect(feature.name, select.value));
    label.append(name, select);
    details.append(label);
  }
}

/** 샘플 폴더를 읽지 못했을 때 이유를 화면에 보여 준다(빈 화면만 보이지 않게) */
export function mountProblem(doc: Document, message: string): void {
  const details = panel(doc, "샘플 모드를 시작하지 못했다");
  const text = doc.createElement("p");
  text.className = "mt-2 max-w-80 break-words text-destructive-foreground";
  text.textContent = message;
  details.append(text);
}
