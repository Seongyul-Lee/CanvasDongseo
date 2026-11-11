# CanvasDongseo — Design Process

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD](01_prd.md), [UX Strategy](06_ux_strategy.md), [Presentation](layers/03_presentation.md), [스크럼 보드](05_scrum_board.md) |

> **이 문서의 역할**: 화면 디자인을 *어떤 순서로, 어떤 산출물로* 진행하는지 정한다. Actor·시나리오, UX 원칙, 기준 디자인(T3 Code), 화면 구성, 문구 원칙은 [UX Strategy](06_ux_strategy.md)가, 앱 안의 구현 구조(theme·components·screens, 토큰 import)는 [Presentation](layers/03_presentation.md)이, 스프린트 운영은 [스크럼 보드](05_scrum_board.md)가 맡는다.

---

## 1. 흐름

```
[입력]        UX Strategy (Actor → 시나리오, 기준 디자인, 화면 구성, 문구 원칙)
[처음 한 번]  Foundation
[기능마다]    ① User Flow → ② 상태 목록 → ③ 목업 → ④ 구현 → ⑤ 실제 앱 확인 → ⑥ 시나리오 점검
```

- 앱 전체를 단계별로 한 번씩 훑지 않는다. **기능 하나를 ①~⑥까지 끝내고** 다음 기능으로 간다. 한 스프린트에 여러 기능을 끝낼 수 있다.
- 기능 하나는 백로그 항목 하나다. 항목의 "화면 작업 단계" 체크 목록이 ①~⑥이다([스크럼 보드 §4](05_scrum_board.md#4-템플릿)).
- Foundation은 첫 기능의 ③ 전에 한 번 한다. 이후 바꿀 일이 생기면 그 기능에서 고친다.

| 단계 | 입력 | 산출물 | 위치 |
|---|---|---|---|
| Foundation | UX Strategy §3, [Presentation §2.2](layers/03_presentation.md#22-디자인-토큰) | T3 Code에서 가져온 토큰·기본 부품과 출처 기록, 한글 글꼴, 문구·용어·한글 표기 규칙 | `ui/src/theme/`, `ui/src/components/ui/`, `ui/UPSTREAM.md`, `design/writing.md` |
| ① User Flow | UX Strategy의 시나리오, 그 기능의 Functional Spec | 시작 → 성공까지의 흐름과 실패·중단 갈래 | 해당 [Functional Spec](specs/README.md)의 User Flow 절(`UF-NN`, Mermaid) |
| ② 상태 목록 | User Flow, §2.2의 상태 출처 | 이 기능에서 생길 수 있는 상태와 각 상태의 ViewModel 샘플 | `design/samples/<기능>/` |
| ③ 목업 | 상태 목록, 샘플 | SPA 샘플 모드로 그린 상태별 화면 | `ui/`(샘플 모드), 캡처는 PR |
| ④ 구현 | 샘플, 목업 | [Presentation §2.1](layers/03_presentation.md#21-프런트엔드-spa-내부-구조) 구현 순서를 따른다 | `ui/`, `crates/presentation/*` |
| ⑤ 실제 앱 확인 | 상태 목록 | 상태별 캡처, PR의 before/after | PR(레포에 커밋하지 않는다) |
| ⑥ 시나리오 점검 | 시나리오, User Flow | 시나리오별 통과·막힘 기록 | PR 본문 |

## 2. 단계별 기준

### 2.1 Foundation

기준 디자인은 T3 Code다([UX Strategy §3](06_ux_strategy.md#3-기준-디자인-t3-code)). Foundation은 디자인을 새로 설계하지 않고, 기준 버전에서 가져와 우리 것을 더한다.

| 순서 | 할 일 | 산출물 |
|---|---|---|
| 1 | 기준 버전과 가져온 파일 목록을 기록한다. 가져온 파일에 브랜드 자산(T3 Code·다른 회사 로고)이 섞였는지 점검해 기록한다 | `ui/UPSTREAM.md`: 기준 버전·커밋, 가져온 파일마다 원본 경로, 고친 파일과 이유, 브랜드 자산 점검 결과 |
| 2 | 그대로 가져올 것을 복사한다: `apps/web/src/index.css`, `apps/web/src/components/ui/`(Base UI 래퍼) 중 T3 Code 내부(앱 상태, 내부 패키지, 브랜드 자산)에 묶이지 않은 부품, 그 부품이 부르는 보조 모듈, lucide 아이콘. 원본을 그대로 쓸 수 없는 보조 모듈은 필요한 부분만 다시 쓰고 `UPSTREAM.md`의 고친 파일에 적는다 | `ui/src/theme/t3code.css`, `ui/src/components/ui/`, `ui/src/lib/`, `ui/src/hooks/` |
| 3 | 기본 테마 값만 쓴다. 실행 중에 색을 계산해 변수를 덮어쓰는 코드(`themePalette.ts`, `appearanceContrast.ts` 등)는 가져오지 않는다 | |
| 4 | 한글 글꼴을 고르고 `--font-sans` 앞에 넣는다. 크기와 색은 그대로 둔다. 화면이 깨질 때만 줄 간격을 고친다 | `ui/src/theme/app.css`, 글꼴 파일과 라이선스 |
| 5 | 문구, 용어, 한글 표기 규칙을 정한다. [UX Strategy §10](06_ux_strategy.md#10-문구-원칙)의 문구 원칙을 따른다 | `design/writing.md` |

- **가져온 파일은 고치지 않는다.** 우리 것은 별도 파일(`theme/app.css`, `components/ui/` 밖의 컴포넌트)에 둔다. 기준 버전을 올릴 때 가져온 파일을 통째로 바꾸고 차이만 확인하기 위해서다. 꼭 고쳐야 하면 `UPSTREAM.md`에 이유를 적는다.
- 기능 화면(사이드바, 설정, 온보딩, 입력창)은 복사하지 않는다. 판정 로직이 섞여 있으므로 구조와 클래스를 본떠 기능의 ③·④에서 다시 쓴다.
- 데스크톱에서 함께 확인한다: 라이트·다크(설정에서 시스템·라이트·다크), 창 크기 변화에 따른 배치, 키보드 조작과 포커스 표시. 기준 디자인에 있는 동작을 따른다.
- 문구, 용어, 한글 표기 규칙은 Foundation이 정하고 이후 기능에서 고쳐 간다. UX Strategy는 원칙만 두므로 규칙을 바꿀 때 UX Strategy를 고치지 않는다. 원칙과 어긋나게 바꿔야 하면 UX Strategy를 먼저 고친다.

### 2.2 User Flow와 상태 목록

- 대상 기능과 화면은 [UX Strategy](06_ux_strategy.md)의 시나리오와 화면 구성을 따른다. 이 문서에서는 화면을 정하지 않는다.
- User Flow에는 성공 경로뿐 아니라 실패·중단 갈래를 함께 그린다. 갈래는 그 기능 Functional Spec의 예외(§3)에서 가져온다. ⑥에서 갈래마다 끝까지 가 볼 수 있어야 한다.
- 상태 목록은 화면 단위가 아니라 **그 기능에서 화면에 나타날 수 있는 상태** 단위로 만든다. 빠뜨리지 않도록 아래 출처를 확인한다.

| 출처 | 확인할 것 |
|---|---|
| AI 중립 이벤트([application §4.2](layers/02_application.md#42-ai)) | 스트리밍 중, 도구 사용, 권한 확인, 사용량 한도, 완료, 오류 |
| 확인 요청([application §4.4](layers/02_application.md#44-사용자-상호작용과-로컬-저장)) | `UserConfirmation`, `OtpPrompt` |
| 작업 상태([application §4.6](layers/02_application.md#46-작업-오래-걸리는-유스케이스)) | `queued`, `running`(진행률), `done`, `failed`, `cancelled` |
| application 오류([application §4.5](layers/02_application.md#45-application-오류)) | 오류 종류마다 화면에 보일 모습 |
| 비어 있음·불러오는 중 | 첫 사용, 데이터가 아직 없음 |

- 상태 하나가 ViewModel의 `kind` 값 하나와 샘플 JSON 하나가 된다. `design/samples/<기능>/README.md`에 상태 목록을 두고, 같은 폴더에 상태마다 `<상태>.json`을 둔다.

### 2.3 목업

- 목업은 따로 그리지 않고 **SPA 샘플 모드**로 그린다. SPA의 command 래퍼를 샘플 구현으로 바꿔 끼우면, 코어 없이 `design/samples/<기능>/<상태>.json`을 받아 실제 컴포넌트로 화면을 그린다. Vite 개발 서버로 띄우므로(`npm --prefix ui run dev:sample`) Rust·Tauri 빌드가 없고, 컴포넌트나 샘플을 저장하면 바로 반영된다. 구조와 상태를 고르는 방법은 [Presentation §2.4](layers/03_presentation.md#24-샘플-모드).
- 샘플 데이터는 **ViewModel 모양의 JSON**이다. 목업이 곧 ViewModel 타입 초안이 되고, 화면이 조건 분기 없이 그려지는지 이 단계에서 확인한다(예: 배너는 `banner.kind`로 받는다).
- **샘플 JSON은 Presenter 테스트의 기대값이다**(읽는 방법은 [Presentation §2.1](layers/03_presentation.md#21-프런트엔드-spa-내부-구조)). ③에서 샘플 모드가 이 파일을 그리고, ④에서 Presenter 테스트가 Presenter 출력과 같은 파일을 비교한다. ViewModel이 바뀌면 테스트가 실패하므로 목업과 앱이 어긋나지 않는다.
- 화면 구조와 배치는 T3 Code의 해당 화면을 본뜬다([UX Strategy §7](06_ux_strategy.md#7-화면-구성)). 값을 직접 적지 않고 CSS 변수와 Tailwind 유틸리티만 쓴다.
- 샘플 모드는 브라우저에서 보므로 macOS WKWebView와 글꼴·CSS가 다를 수 있고, Tauri 전용 기능(창, OS 브라우저 열기, 파일 대화상자)은 확인할 수 없다. 이 차이는 ⑤에서 확인한다.
- 같은 스프린트 안에서 순서를 고를 수 있으면 컴포넌트를 많이 쓰는 기능부터 한다. 앞 기능에서 만든 컴포넌트를 뒤 기능이 다시 쓰게 된다.

### 2.4 실제 앱 확인

| 항목 | 기준 |
|---|---|
| 상태 | 상태 목록의 상태마다 실제 앱에서 캡처한다 |
| 환경 | 라이트·다크, 좁은 창·넓은 창, 키보드만으로 조작 |
| 접근성 | 글자와 배경의 대비, 포커스 표시 |
| PR 증거 | 화면이 바뀌면 before/after 캡처를 PR에 붙인다. 움직임·타이밍이 있으면 짧은 영상을 붙인다. 캡처는 레포에 커밋하지 않는다 |

### 2.5 시나리오 점검

실제 사용자 대상 확인 대신, UX Strategy의 시나리오와 User Flow로 확인한다.

- 이 기능에 걸린 시나리오를 하나씩 실제 앱에서 따라간다. User Flow의 갈래(성공·실패·중단)마다 그 상태에 도달하고, 빠져나오거나 끝낼 수 있는지 본다.
- 막힌 곳은 이 기능 안에서 고친다. 이 기능의 범위를 넘으면 백로그 항목으로 올린다.
- 결과는 PR 본문에 시나리오 번호별로 "통과" 또는 "막힘(이유)"으로 적는다.
- 실제 학생 대상 사용성 확인은 MVP가 끝난 뒤 검토한다.

## 3. 규칙을 늘리는 기준

- 린트·검사 규칙은 **같은 실수가 두 번 나왔을 때** 더한다. 일어나지 않은 실수를 예상해 미리 만들지 않는다.
- 규칙을 더하면 그 규칙이 지키는 내용을 적은 문서에 한 줄로 근거를 남긴다.

## 4. 폴더 구성

```
design/
  writing.md           Foundation의 문구·용어·한글 표기 규칙
  samples/<기능>/
    README.md          ② 상태 목록
    <상태>.json        ViewModel 샘플 = Presenter 테스트 기대값 = ③ 샘플 모드 입력
    <상태>.events.json (있을 때만) 그 상태에서 순서대로 흘려보낼 이벤트
ui/
  UPSTREAM.md          T3 Code 기준 버전과 가져온 파일 기록
  LICENSE-t3code       T3 Code의 라이선스 전문(MIT)
  src/theme/           t3code.css(가져온 원본), app.css(우리 것)
  src/components/ui/   T3 Code에서 가져온 기본 부품
  src/lib/, src/hooks/ 가져온 부품이 부르는 보조 모듈
```

## 5. 미확인 항목

- 한글 글꼴(무엇을 쓸지, 앱에 포함할지)
- 샘플 모드 화면을 상태별로 캡처하는 도구. Tauri 창 자체의 캡처는 우선 직접 한다

## 6. 확인한 항목

### 6.1 `components/ui/`를 복사만 해서 컴파일되는지 (#20, 2026-11-11)

기준 태그 `v0.1.8-nightly.20261109.4399`(커밋 `357fb754`)의 `apps/web/src/components/ui/` 52개 파일(테스트 4개 제외)을 빈 프로젝트에 복사해 `tsc --noEmit`으로 확인했다. `ui/tsconfig.json`과 같은 옵션(`strict`, `noUnused*`, `noUncheckedIndexedAccess`, `verbatimModuleSyntax`)에 `~/*` 별칭만 더했다. **복사만으로는 통과하지 않는다.**

| 단계 | 더한 것 | 결과 |
|---|---|---|
| A | 복사만 | 거의 모든 파일 실패. 부품 41곳이 `~/lib/utils`의 `cn`을 쓴다 |
| B | `lib/utils.ts`(`cn`만, 원본과 같은 tailwind-merge 설정) | `components/ui/` 밖 의존이 있는 14개 파일만 실패 |
| C | 자체 완결 보조 모듈 4개 원본 복사: `lib/visibleAnimation`, `lib/observeResize`, `lib/color`, `hooks/useCommitOnBlur` | 아래 표의 파일만 실패. 우리 엄격 옵션 때문에 새로 생긴 오류는 없다 |

C 뒤에도 실패하는 파일과 원인:

| 파일 | 걸리는 의존 | 성격 |
|---|---|---|
| `tooltip.tsx`, `command.tsx` (+ `tooltip`을 쓰는 `panel-tab-close-button.tsx`) | `~/lib/windowControlsOverlay` → `~/env`(`isElectron`), `utils`의 `isMacPlatform`(`@t3tools/shared/keybindings`) | Electron 창 제어 영역. 기본 부품인데 걸린다 |
| `toast.tsx`, `toast.logic.ts`, `toastHelpers.ts`, `anchoredCopyToast.ts` | `@t3tools/contracts`, `~/composerDraftStore`, `~/hooks/useThreadRouteTarget`, `~/hooks/useCopyToClipboard`, `~/components/MorphIcon`, `lucide` | 앱 상태·라우팅에 묶임 |
| `sidebar.tsx` | `effect/Schema`, `~/hooks/useLocalStorage`·`useMediaQuery`·`useResizeDrag`, `~/lib/accessibilityAnnouncements`, `~/components/MorphIcon`, `lucide` | 기능 화면 성격(원래 복사 대상 아님) |
| `calendar.tsx` | `@daypicker/react` | 외부 npm 의존 |
| `qr-code.tsx` | `@t3tools/shared/qrCode` | T3 내부 패키지 |
| `standalone-page.tsx`, `wizard.tsx` | `../T3Wordmark`(브랜드 자산), `../AnimatedHeight` | 브랜드·앱 컴포넌트 |

나머지 약 40개 파일(button, input, dialog, menu, popover, select, checkbox, switch 등)은 B·C의 보조 파일만으로 통과한다.

필요한 npm 의존(통과한 부품 기준, 원본 `pnpm-lock.yaml`이 해석한 버전): `@base-ui/react` 1.9.0, `lucide-react` 0.564.0, `class-variance-authority` 0.7.1, `tailwind-merge` 3.6.0(원본 `package.json`의 범위는 `^3.4.0`), `react`·`react-dom` 19.3.0, `@types/react`·`@types/react-dom` 19.3.0, `tailwindcss` 4.3.3. 이 중 `class-variance-authority`, `tailwind-merge`는 아직 `ui/package.json`에 없다.

Foundation(#21)에 주는 결론:

- 2단계 "그대로 복사"는 `components/ui/` 전체가 아니라 **통과하는 부품만** 고르는 방식으로 바꾼다. 위 표의 파일은 가져오지 않거나, 가져오면 `UPSTREAM.md`에 고친 이유를 적는다.
- `components/ui/` 밖에서 가져와야 하는 파일이 생긴다: `lib/utils.ts`는 원본을 그대로 쓸 수 없어(`@t3tools/contracts`, `effect`, `composerDraftStore` 의존) `cn`만 둔 우리 파일로 둔다. 보조 모듈 4개는 원본 그대로 가져온다.
- 원본 import를 고치지 않으려면 `~/*` 별칭을 `ui/tsconfig.json`과 `vite.config.ts`에 더해야 한다. 별칭이 폴더 의존 검사(`boundaries.config.js`)와 맞는지 함께 확인한다.
- `tooltip`은 자주 쓰는 기본 부품이라 빼지 않는다. `tooltip.tsx`·`command.tsx`가 `windowControlsOverlay`에서 쓰는 것은 `useNativeTitlebarHeight` 하나뿐이므로 두 부품은 고치지 않고, `lib/windowControlsOverlay.ts`만 Tauri용으로 바꿔 둔다(2026-11-11 결정). 고친 이유는 `UPSTREAM.md`에 적는다.
