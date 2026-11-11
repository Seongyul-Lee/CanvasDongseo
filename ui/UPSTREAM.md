# UPSTREAM — T3 Code에서 가져온 것

기준 디자인 T3 Code에서 가져온 파일의 기록이다. 절차는 [Design Process §2.1](../docs/03_design_process.md#21-foundation), 근거는 [UX Strategy §3](../docs/06_ux_strategy.md#3-기준-디자인-t3-code)에 있다.

| 항목 | 값 |
|---|---|
| 저장소 | `pingdotgg/t3code` (https://github.com/pingdotgg/t3code) |
| 기준 태그 | `v0.1.8-nightly.20261109.4399` |
| 커밋 | `357fb754fa9dae4111f0945fb8a13d30b012a62a` |
| 원본 기준 폴더 | `apps/web/src/` |
| 라이선스 | MIT, Copyright (c) 2026 T3 Tools Inc. 전문은 [`LICENSE-t3code`](LICENSE-t3code) |
| 가져온 날 | 2026-11-11 (#21) |

T3 Code의 소스 파일에는 파일별 라이선스 문구가 없다. 저작권·라이선스 문구는 `LICENSE-t3code`(원본 `LICENSE` 그대로)로 남긴다.

## 1. 그대로 가져온 파일

원본과 바이트 단위로 같다(2026-11-11 비교, 47개). **고치지 않는다.** 기준 버전을 올릴 때 통째로 바꾼다.

| 우리 경로 (`ui/src/`) | 원본 경로 (`apps/web/src/`) | 부르는 부품 |
|---|---|---|
| `theme/t3code.css` | `index.css` | |
| `components/ui/<파일>` (아래 42개) | `components/ui/<같은 이름>` | |
| `lib/visibleAnimation.ts` | `lib/visibleAnimation.ts` | `spinner`, `refresh-icon` |
| `lib/observeResize.ts` | `lib/observeResize.ts` | `middle-truncate` |
| `lib/color.ts` | `lib/color.ts` | `color-picker` |
| `hooks/useCommitOnBlur.ts` | `hooks/useCommitOnBlur.ts` | `draft-input` |

`components/ui/` 42개: `alert-dialog.tsx`, `alert.tsx`, `autocomplete.tsx`, `badge.tsx`, `button.tsx`, `checkbox.tsx`, `closing-positioner.tsx`, `collapsible-section-header.tsx`, `collapsible.tsx`, `color-picker.tsx`, `combobox.tsx`, `command.tsx`, `dialog-styles.ts`, `dialog.tsx`, `discovery-list.tsx`, `draft-input.tsx`, `empty.tsx`, `group.tsx`, `input-group.tsx`, `input.tsx`, `kbd.tsx`, `label.tsx`, `menu.tsx`, `middle-truncate.tsx`, `number-field.tsx`, `panel-tab-close-button.tsx`, `popover.tsx`, `preview-card.tsx`, `radio-group.tsx`, `refresh-icon.tsx`, `scroll-area.tsx`, `select.tsx`, `separator.tsx`, `sheet.tsx`, `skeleton.tsx`, `spinner.tsx`, `switch.tsx`, `table.tsx`, `textarea.tsx`, `toggle-group.tsx`, `toggle.tsx`, `tooltip.tsx`

보조 모듈 4개는 `components/ui/` 밖의 파일이지만 부품이 불러서 원본 경로 그대로 함께 가져왔다. 다른 것에 의존하지 않는다.

## 2. 고친 파일

원본을 그대로 쓸 수 없어 필요한 부분만 남기거나 바꿔 썼다. 기준 버전을 올릴 때 원본의 바뀐 점을 보고 손으로 맞춘다.

| 우리 경로 (`ui/src/`) | 원본 경로 | 고친 내용 | 이유 |
|---|---|---|---|
| `lib/utils.ts` | `lib/utils.ts` | `cn`과 tailwind-merge 설정만 남겼다. ID 생성, 플랫폼 판별 함수는 뺐다 | 원본은 `@t3tools/contracts`, `@t3tools/shared`, `effect`, `composerDraftStore`를 import한다. 가져온 부품이 쓰는 것은 `cn` 하나다 |
| `lib/windowControlsOverlay.ts` | `lib/windowControlsOverlay.ts` | `useNativeTitlebarHeight`만 남기고 Tauri 창 설정에 맞는 값을 돌려주게 바꿨다 | 원본은 Electron(`isElectron`, Window Controls Overlay API, macOS 신호등 영역 52px)을 읽는다. Tauri의 WebView에는 그 API가 없다. `tooltip.tsx`와 `command.tsx`가 쓰는 것은 이 훅 하나라서 두 부품은 고치지 않았다(2026-11-11 결정) |

- 타이틀바 높이는 창 설정(`crates/bin/app/tauri.conf.json`)과 맞아야 한다. 지금 창은 기본 장식이라 타이틀바가 WebView 밖에 있고 값은 0이다. 창을 `titleBarStyle: "Overlay"`로 바꾸면 타이틀바가 WebView 위에 겹치므로 이 값도 바꾼다. 어긋나면 `test/lib.test.js`가 실패한다.
- `cn`의 tailwind-merge 설정(`text-2xs`~`text-5xs`, `bg-composer-seam-*`)은 `t3code.css`의 토큰과 짝이다. 배경 이미지 클래스(`bg-composer-seam-*`) 설정은 `test/lib.test.js`가 확인한다. 글자 크기 설정은 테스트가 지켜 주지 않는다. tailwind-merge 3.6.0은 설정 없이도 `text-2xs`를 글자 크기로 알아봐서, 설정을 빼도 테스트가 통과한다.

## 3. 가져오지 않은 파일 (`components/ui/`)

| 파일 | 이유 | 필요해지면 |
|---|---|---|
| `toast.tsx`, `toast.logic.ts`, `toastHelpers.ts`, `anchoredCopyToast.ts` | T3 Code의 앱 상태와 라우팅에 묶여 있다(`@t3tools/contracts`, `composerDraftStore`, `useThreadRouteTarget`, `useCopyToClipboard`, `MorphIcon`) | 구조와 클래스를 본떠 다시 쓴다 |
| `sidebar.tsx`, `sidebarState.ts` | `effect/Schema`, 앱 훅 3개(`useLocalStorage`, `useMediaQuery`, `useResizeDrag`), `accessibilityAnnouncements`, `MorphIcon`에 의존한다. 사이드바는 기능 화면이라 다시 쓰기로 했다(Design Process §2.1) | 구조와 클래스를 본떠 다시 쓴다 |
| `calendar.tsx` | npm 패키지 `@daypicker/react`(원본 lock 10.0.1)가 필요하다. 쓰는 화면이 아직 없다 | 패키지를 더하고 그대로 복사한다 |
| `wizard.tsx` | `components/AnimatedHeight.tsx`가 필요하다. `components/ui/` 밖의 `components/`는 우리 컴포넌트를 두는 곳이다 | `AnimatedHeight.tsx`(`lib/observeResize`만 의존)를 둘 곳을 정하고 함께 가져온다 |
| `qr-code.tsx` | T3 Code 내부 패키지 `@t3tools/shared/qrCode`에 의존한다 | 쓰지 않는다 |
| `standalone-page.tsx` | `T3Wordmark`(T3 Code 워드마크)를 그린다 | 쓰지 않는다 |
| `middle-truncate.test.ts`, `middle-truncate.browser.test.tsx`, `sidebar.browser.test.tsx`, `toast.browser.test.tsx` | `vite-plus/test`로 쓴 원본의 테스트다. 우리 테스트 도구와 다르다 | 쓰지 않는다 |

확인 과정은 [Design Process §6.1](../docs/03_design_process.md)에 있다(#20).

## 4. npm 의존

원본 `pnpm-lock.yaml`이 해석한 버전을 따른다([platform §1](../docs/cross_cutting/02_platform.md#1-기술-스택)).

| 패키지 | 버전 | 비고 |
|---|---|---|
| `@base-ui/react` | 1.9.0 | |
| `lucide-react` | 0.564.0 | 아이콘. ISC |
| `class-variance-authority` | 0.7.1 | |
| `tailwind-merge` | 3.6.0 | 원본 `package.json`의 범위는 `^3.4.0` |
| `tailwindcss`, `@tailwindcss/vite` | 4.3.3 | |
| `react`, `react-dom` | 19.3.0 | |

- T3 Code가 `sidebar`·`toast`에서 쓰는 `lucide`(React용이 아닌 패키지)는 가져온 부품이 쓰지 않아 넣지 않았다.
- 원본은 `main.tsx`에서 `@fontsource-variable/inter`(Inter Variable 글꼴)를 불러온다. `--font-sans`는 `-apple-system`이 먼저라 macOS에서는 쓰이지 않으므로 넣지 않았다. Windows에서는 원본이 Inter로 그려진다. 한글 글꼴을 정할 때(Foundation 4단계) 함께 정한다.

## 5. 브랜드 자산 점검 (2026-11-11)

- 가져온 부품과 보조 모듈에는 T3 Code나 다른 회사의 로고, 워드마크, 앱 아이콘이 없다. 부품 안에 직접 그린 SVG는 체크 표시(`checkbox.tsx`)와 메뉴의 표시(`menu.tsx`)뿐이다.
- `t3code.css`는 이미지 파일을 참조하지 않는다. `url()`은 바탕 결 무늬(`--surface-grain`, SVG 노이즈 필터) 2개뿐이다.
- `t3code.css`에는 내장 테마의 색 프리셋 5개(`data-theme-id`가 `t3-chat`, `ember`, `grove`, `iris`, `ocean`)와 배경 그림용 색 변수(`--stage-art-*`, `--stage-night-*`)가 들어 있다. 색 값일 뿐 로고나 그림은 아니다. 우리는 `data-theme-id`를 붙이지 않으므로 기본 테마만 적용된다. `t3-chat`은 T3의 다른 제품 색이므로 켜지 않는다.
- `T3Wordmark`를 그리는 `standalone-page.tsx`는 가져오지 않았다.
- 실행 중에 색을 계산하는 코드(`themePalette.ts`, `appearanceContrast.ts` 등)는 가져오지 않았다.

## 6. 우리 쪽에서 맞춘 설정

- **별칭 `~`**: `ui/src`를 가리킨다(`tsconfig.json`의 `paths`, `vite.config.ts`의 `resolve.alias`). 가져온 부품이 `~/lib/utils`처럼 부르기 때문이다. 우리 코드는 상대 경로를 쓴다. 별칭으로 불러도 폴더 의존 검사는 똑같이 적용된다(`test/boundaries.test.js`).
- **폴더**: `lib/`, `hooks/`는 원본 경로를 그대로 둔 것이다. 의존 규칙은 [Presentation §2.1](../docs/layers/03_presentation.md#21-프런트엔드-spa-내부-구조)에 있다.
- **ESLint**: `src/components/ui/**`에 한해, `...props`에서 속성을 빼려고 둔 쓰지 않는 변수(`_hideIndicator` 등)를 허용한다. 다른 규칙과 폴더 의존 규칙은 그대로다.
- **테마 연결**: `theme/app.css`가 `t3code.css`를 불러온다. Tailwind는 `t3code.css` 안의 `@import "tailwindcss"`로 들어온다.

## 7. 원본과 다르게 보이거나 동작하는 것

- **바탕 결 무늬**: `--surface-grain`은 `data:` URL 이미지다. `default-src 'self'`만 있는 CSP에서는 막혀서 그려지지 않으므로, 앱의 CSP에 `img-src 'self' data:`를 더했다(2026-11-12 결정). 무늬가 있고 없고의 차이는 눈으로 거의 구분되지 않는다(배포 앱의 빈 바탕 기준 라이트 252 → 평균 250.8, 브라우저 다크 0 → 평균 3.3). 그래서 CSP가 `data:` 이미지를 막지 않는지 `test/theme.test.js`가 확인한다.
- **빌드 경고**: Tailwind가 CSS를 줄일 때 `::highlight()` 경고 4개를 낸다. `t3code.css`의 인용 강조 규칙에서 나오고 우리는 그 규칙을 쓰지 않는다. 원본 그대로 두려고 고치지 않았다.
- **React Compiler**: 원본의 린트 설정과 부품 주석(`tooltip.tsx`)은 React Compiler가 메모이제이션을 맡는다고 전제한다. 우리 빌드는 React Compiler를 쓰지 않는다. 다시 그리는 횟수가 다를 수 있다(측정하지 않았다).

## 8. 기준 버전을 올릴 때

1. 새 태그를 받아 §1의 파일을 통째로 덮어쓴다.
2. `components/ui/`에 새로 생기거나 없어진 파일, 새로 생긴 import를 확인하고 §1·§3을 고친다.
3. §2의 두 파일은 원본의 바뀐 점을 보고 손으로 맞춘다.
4. §4를 원본 lock의 버전에 맞춘다.
5. `npm run typecheck`, `npm run lint`, `npm test`, `npm run build`를 돌리고, 라이트·다크에서 부품을 띄워 본다.
6. 이 문서의 기준 태그, 커밋, 날짜를 고친다. [UX Strategy §3](../docs/06_ux_strategy.md#3-기준-디자인-t3-code)의 기준 버전도 함께 고친다.
