// 샘플 구현(commands/sample): design/samples/의 파일 목록을 기능·상태 목록으로 바꾼다. 순수 함수만 둔다

/** 이벤트 파일(<상태>.events.json)의 한 줄: 앞 이벤트 뒤 afterMs만큼 기다렸다가 흘려보낸다 */
export interface ScriptStep {
  afterMs: number;
  event: string;
  payload: unknown;
}

export interface SampleState {
  name: string;
  /** <상태>.json: command 응답으로 돌려줄 ViewModel */
  viewModel: unknown;
  /** <상태>.events.json: 순서대로 흘려보낼 이벤트. 파일이 없으면 빈 목록 */
  script: ScriptStep[];
}

export interface SampleFeature {
  name: string;
  /** 이름 순 */
  states: SampleState[];
}

/** 이름 순 */
export type Catalog = SampleFeature[];

const SAMPLE_PATH = /\/samples\/([^/]+)\/([^/]+)\.json$/;
const EVENTS_SUFFIX = ".events";

const byName = (a: { name: string }, b: { name: string }) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0);

/** `files`는 import.meta.glob이 주는 모양(경로 → JSON 값)이다 */
export function buildCatalog(files: Record<string, unknown>): Catalog {
  const features = new Map<string, Map<string, SampleState>>();
  const scripts: { path: string; feature: string; state: string; value: unknown }[] = [];

  for (const [globPath, value] of Object.entries(files)) {
    // 오류 문구에는 앞의 ../를 뗀 경로(design/samples/…)를 보여 준다
    const path = globPath.replace(/^(?:\.\.\/)+/, "");
    const match = SAMPLE_PATH.exec(globPath);
    const [feature, file] = [match?.[1], match?.[2]];
    if (feature === undefined || file === undefined) {
      throw new Error(`${path}: design/samples/<기능>/<상태>.json 모양이 아니다`);
    }
    if (file.endsWith(EVENTS_SUFFIX)) {
      scripts.push({ path, feature, state: file.slice(0, -EVENTS_SUFFIX.length), value });
      continue;
    }
    const states = features.get(feature) ?? new Map<string, SampleState>();
    states.set(file, { name: file, viewModel: value, script: [] });
    features.set(feature, states);
  }

  for (const { path, feature, state, value } of scripts) {
    const target = features.get(feature)?.get(state);
    if (target === undefined) {
      throw new Error(`${path}: 짝이 되는 상태 파일(${state}.json)이 없다`);
    }
    target.script = parseScript(path, value);
  }

  return [...features]
    .map(([name, states]) => ({ name, states: [...states.values()].sort(byName) }))
    .sort(byName);
}

/** 이벤트 파일은 사람이 손으로 쓰므로 모양을 확인하고, 틀리면 파일과 위치를 알려 준다 */
function parseScript(path: string, value: unknown): ScriptStep[] {
  if (!Array.isArray(value)) {
    throw new Error(`${path}: 이벤트 목록은 배열이어야 한다`);
  }
  return value.map((item: unknown, index) => {
    const where = `${path}: ${index + 1}번째 이벤트`;
    if (typeof item !== "object" || item === null) {
      throw new Error(`${where}가 객체가 아니다`);
    }
    const { afterMs, event } = item as Record<string, unknown>;
    if (typeof afterMs !== "number" || !Number.isFinite(afterMs) || afterMs < 0) {
      throw new Error(`${where}의 afterMs는 0 이상의 숫자여야 한다`);
    }
    if (typeof event !== "string" || event === "") {
      throw new Error(`${where}의 event는 빈 문자열이 아닌 문자열이어야 한다`);
    }
    if (!("payload" in item)) {
      throw new Error(`${where}에 payload가 없다`);
    }
    return { afterMs, event, payload: item.payload };
  });
}
