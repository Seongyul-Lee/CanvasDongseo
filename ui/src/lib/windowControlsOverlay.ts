// T3 Code의 apps/web/src/lib/windowControlsOverlay.ts를 Tauri용으로 바꿨다(MIT, ui/LICENSE-t3code). 기록은 ui/UPSTREAM.md
// 원본은 Electron의 Window Controls Overlay API와 macOS 신호등 영역 높이를 읽는다.
// Tauri의 WebView에는 그 API가 없고, 타이틀바가 화면을 덮는지는 창 설정(crates/bin/app/tauri.conf.json)이 정한다.

// 네이티브 타이틀바가 WebView 위에 겹치는 높이(px). 지금 창은 기본 장식이라 타이틀바가 WebView 밖에 있다.
// 창 설정을 titleBarStyle: "Overlay"로 바꾸면 타이틀바가 겹치므로 이 값도 화면 상단 바 높이로 바꾼다.
// 둘이 어긋나면 test/lib.test.js가 실패한다
const NATIVE_TITLEBAR_HEIGHT = 0;

/** 가져온 부품(tooltip, command)이 팝업을 타이틀바 아래로 밀어낼 때 쓴다. 이름은 원본을 따른다 */
export function useNativeTitlebarHeight(): number {
  return NATIVE_TITLEBAR_HEIGHT;
}
