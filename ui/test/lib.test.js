// T3 Code 원본을 그대로 쓸 수 없어 우리가 쓴 보조 모듈(UPSTREAM.md "고친 파일")을 확인한다.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { cn } from "../src/lib/utils.ts";
import { useNativeTitlebarHeight } from "../src/lib/windowControlsOverlay.ts";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");

test("cn: 같은 종류의 클래스는 뒤에 온 것이 이긴다", () => {
  assert.equal(cn("px-2", "px-4"), "px-4");
});

test("cn: 거짓 값은 빼고 배열과 객체를 펼친다", () => {
  assert.equal(cn("a", false, undefined, null, ["b", { c: true, d: false }]), "a b c");
});

// 가져온 부품이 쓰는 T3 Code 토큰(t3code.css의 --text-2xs 등)을 tailwind-merge가 색으로 보면 글자 색이 지워진다
test("cn: T3 Code의 작은 글자 크기를 글자 색과 따로 본다", () => {
  for (const size of ["text-2xs", "text-3xs", "text-4xs", "text-5xs"]) {
    assert.equal(cn("text-red-500", size), `text-red-500 ${size}`);
    assert.equal(cn("text-xs", size), size);
  }
});

test("cn: T3 Code의 배경 이미지 클래스를 배경색과 따로 본다", () => {
  for (const image of ["bg-composer-seam-below", "bg-composer-seam-above"]) {
    assert.equal(cn("bg-red-500", image), `bg-red-500 ${image}`);
  }
});

// 타이틀바가 WebView 위에 겹치는지는 창 설정이 정한다(Tauri에서는 titleBarStyle: "Overlay"일 때만 겹친다).
// 설정만 바꾸고 화면 쪽 값을 잊으면 여기서 걸린다
test("타이틀바 높이: Tauri 창 설정과 맞는다", () => {
  const conf = JSON.parse(readFileSync(path.join(repoRoot, "crates/bin/app/tauri.conf.json"), "utf8"));
  /** @type {{ titleBarStyle?: string }[]} */
  const windows = conf.app.windows;
  assert.ok(windows.length > 0, "창 설정이 없다");
  const overlaps = windows.some((w) => w.titleBarStyle === "Overlay");

  const height = useNativeTitlebarHeight();
  if (overlaps) {
    assert.ok(height > 0, "타이틀바가 WebView 위에 겹치는데 높이가 0이다");
  } else {
    assert.equal(height, 0, "타이틀바가 WebView 밖에 있는데 높이가 0이 아니다");
  }
});
