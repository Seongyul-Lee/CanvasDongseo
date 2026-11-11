// 샘플 구현(presentation §2.4)의 순수한 부분을 확인한다: 샘플 폴더 읽기, 상태 고르기, command 응답, 이벤트 흘리기.
// Vite·브라우저에 묶인 조립(commands/sample/index.ts, picker.ts)은 화면에서 확인한다.
import assert from "node:assert/strict";
import { mock, test } from "node:test";
import { buildCatalog } from "../src/commands/sample/catalog.ts";
import { createSampleCore } from "../src/commands/sample/sampleCore.ts";
import { selectStates, withSelection } from "../src/commands/sample/selection.ts";

/** import.meta.glob이 주는 모양: 경로 → JSON 값 */
const files = {
  "../../../../design/samples/todo/ready.json": { title: "할 일", items: [{ id: "1" }] },
  "../../../../design/samples/todo/empty.json": { title: "할 일", items: [] },
  "../../../../design/samples/chat/streaming.json": { text: "" },
  "../../../../design/samples/chat/streaming.events.json": [
    { afterMs: 100, event: "chat://delta", payload: { text: "안" } },
    { afterMs: 50, event: "chat://delta", payload: { text: "안녕" } },
    { afterMs: 0, event: "chat://done", payload: { ok: true } },
  ],
  "../../../../design/samples/chat/idle.json": { text: "끝" },
};

test("샘플 폴더: 기능과 상태를 이름 순으로 모으고 이벤트 파일을 상태에 붙인다", () => {
  const catalog = buildCatalog(files);
  assert.deepEqual(
    catalog.map((f) => [f.name, f.states.map((s) => s.name)]),
    [
      ["chat", ["idle", "streaming"]],
      ["todo", ["empty", "ready"]],
    ],
  );
  const [chat, todo] = catalog;
  assert.deepEqual(todo?.states[1]?.viewModel, { title: "할 일", items: [{ id: "1" }] });
  assert.deepEqual(chat?.states[0]?.script, []);
  assert.deepEqual(
    chat?.states[1]?.script.map((s) => [s.afterMs, s.event]),
    [
      [100, "chat://delta"],
      [50, "chat://delta"],
      [0, "chat://done"],
    ],
  );
});

test("샘플 폴더: 짝이 되는 상태 파일이 없는 이벤트 파일은 오류다", () => {
  assert.throws(
    () => buildCatalog({ "../design/samples/chat/streaming.events.json": [] }),
    /chat\/streaming\.events\.json.*streaming\.json/,
  );
});

test("샘플 폴더: 이벤트 파일의 모양이 틀리면 파일과 위치를 알려 준다", () => {
  const state = { "../design/samples/chat/a.json": {} };
  /** @param {unknown} events */
  const broken = (events) => () => buildCatalog({ ...state, "../design/samples/chat/a.events.json": events });
  assert.throws(broken({ afterMs: 0 }), /a\.events\.json.*배열/);
  assert.throws(broken([{ afterMs: -1, event: "x", payload: null }]), /a\.events\.json.*1번째.*afterMs/);
  assert.throws(broken([{ afterMs: "1", event: "x", payload: null }]), /afterMs/);
  assert.throws(broken([{ afterMs: 0, event: "", payload: null }]), /1번째.*event/);
  assert.throws(broken([{ afterMs: 0, event: "x", payload: 1 }, { afterMs: 0, event: "x" }]), /2번째.*payload/);
  assert.throws(broken([null]), /1번째/);
});

test("샘플 폴더: design/samples/<기능>/<상태>.json 모양이 아닌 경로는 오류다", () => {
  assert.throws(() => buildCatalog({ "../design/samples/top.json": {} }), /top\.json/);
});

test("상태 고르기: 주소에 없으면 이름 순 첫 상태, 있으면 그 상태", () => {
  const catalog = buildCatalog(files);
  assert.deepEqual(selectStates(catalog, ""), { chat: "idle", todo: "empty" });
  assert.deepEqual(selectStates(catalog, "?sample.todo=ready"), { chat: "idle", todo: "ready" });
  assert.deepEqual(selectStates(catalog, "?sample.chat=streaming&sample.todo=ready&x=1"), {
    chat: "streaming",
    todo: "ready",
  });
});

test("상태 고르기: 주소의 상태가 없는 이름이면 첫 상태로 돌아간다", () => {
  const catalog = buildCatalog(files);
  assert.deepEqual(selectStates(catalog, "?sample.todo=nope&sample.unknown=x"), { chat: "idle", todo: "empty" });
});

test("상태 고르기: 고른 상태를 주소에 넣고 다른 값은 그대로 둔다", () => {
  assert.equal(withSelection("", "todo", "ready"), "?sample.todo=ready");
  assert.equal(withSelection("?x=1&sample.todo=empty", "todo", "ready"), "?x=1&sample.todo=ready");
  assert.equal(withSelection("?sample.chat=idle", "todo", "ready"), "?sample.chat=idle&sample.todo=ready");
});

/** @param {Record<string, string>} selected 기능 → 고른 상태(나머지는 첫 상태) */
function core(selected) {
  const catalog = buildCatalog(files);
  return createSampleCore({
    catalog,
    selected: { ...selectStates(catalog, ""), ...selected },
    responses: { get_todo: "todo", get_chat: "chat" },
  });
}

test("command: 고른 상태의 샘플을 응답으로 돌려준다", async () => {
  assert.deepEqual(await core({}).invoke("get_todo"), { title: "할 일", items: [] });
  assert.deepEqual(await core({ todo: "ready" }).invoke("get_todo"), { title: "할 일", items: [{ id: "1" }] });
});

test("command: 응답은 부를 때마다 새 값이다(받은 쪽이 고쳐도 샘플이 바뀌지 않는다)", async () => {
  const c = core({ todo: "ready" });
  const first = await c.invoke("get_todo");
  first.items.push({ id: "2" });
  assert.deepEqual(await c.invoke("get_todo"), { title: "할 일", items: [{ id: "1" }] });
});

test("command: 표에 없는 command는 이름을 알려 주며 실패한다", async () => {
  await assert.rejects(core({}).invoke("get_nothing"), /샘플 모드에 없는 command: get_nothing/);
  // 객체가 원래 가진 이름(toString 등)을 command로 착각하지 않는다
  await assert.rejects(core({}).invoke("toString"), /샘플 모드에 없는 command: toString/);
});

test("이벤트: 구독하면 고른 상태의 이벤트를 적힌 간격대로 순서대로 흘려보낸다", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const c = core({ chat: "streaming" });
  /** @type {unknown[][]} */
  const seen = [];
  await c.listen("chat://delta", (payload) => seen.push(["delta", payload.text]));
  await c.listen("chat://done", (payload) => seen.push(["done", payload.ok]));

  assert.deepEqual(seen, []);
  t.mock.timers.tick(99);
  assert.deepEqual(seen, []);
  t.mock.timers.tick(1);
  assert.deepEqual(seen, [["delta", "안"]]);
  t.mock.timers.tick(50);
  assert.deepEqual(seen, [
    ["delta", "안"],
    ["delta", "안녕"],
    ["done", true],
  ]);
});

test("이벤트: 이벤트가 없는 상태에서는 아무것도 오지 않는다", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const handler = mock.fn();
  await core({ chat: "idle" }).listen("chat://delta", handler);
  t.mock.timers.tick(10_000);
  assert.equal(handler.mock.callCount(), 0);
});

test("이벤트: 구독을 끊으면 더 받지 않고, 다시 구독해도 처음부터 다시 흘리지 않는다", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const c = core({ chat: "streaming" });
  const first = mock.fn();
  const second = mock.fn();
  const unlisten = await c.listen("chat://delta", first);
  t.mock.timers.tick(100);
  assert.equal(first.mock.callCount(), 1);

  unlisten();
  await c.listen("chat://delta", second);
  t.mock.timers.tick(10_000);
  assert.equal(first.mock.callCount(), 1);
  // 이어지는 두 번째 delta만 받는다
  assert.deepEqual(
    second.mock.calls.map((call) => call.arguments[0]),
    [{ text: "안녕" }],
  );
});
