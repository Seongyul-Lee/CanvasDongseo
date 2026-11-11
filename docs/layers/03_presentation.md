# Presentation

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../02_system_architecture.md) |

> **이 문서의 역할**: 화면(WebView)과 코어 사이의 View · ViewModel · Presenter · Controller 역할과 위치를 정한다. 화면 구성과 기준 디자인은 [UX Strategy](../06_ux_strategy.md), 흐름은 [Functional Spec](../specs/README.md)의 User Flow, 디자인 절차는 [Design Process](../03_design_process.md)가 맡는다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Interface Adapters (View는 Frameworks & Drivers) |
| crate | `presentation/presenters/{common, school, ai, learning, jobs, shell}`, `presentation/tauri_commands` (+ 프런트엔드 SPA) |
| 의존할 수 있는 것 | `presenters/common`: `application/kernel`. `presenters/<묶음>`: 같은 묶음의 `application/usecases`, `presenters/common`, `kernel`. `tauri_commands`: `usecases/*`, `presenters/*`, `ports/common`(구현), `kernel`, `tauri` |

- Presenter·ViewModel은 순수 로직이고 테스트를 먼저 쓰므로 `tauri`에 의존하지 않는 `presenters/*`에 둔다. Presenter 테스트를 돌릴 때 Tauri를 컴파일하지 않는다. Tauri command는 로직 없는 `tauri_commands`에 둔다(Humble Object).
- `presenters`는 `usecases`와 같은 묶음으로 나눈다. 한 묶음의 유스케이스를 고쳐도 다른 묶음의 Presenter는 다시 빌드되지 않는다([build_ci §2](../cross_cutting/03_build_ci.md#2-crate를-나누는-기준)).
- **여러 묶음에 걸친 표시**는 `presenters/shell`이 판정한다. Presenter는 유스케이스를 부르지 않으므로, `tauri_commands`가 필요한 유스케이스를 각각 불러 받은 출력 DTO를 `shell`에 넘긴다.
  - **연결 상태 모음**: 학교 계정(`GetAuthState`), 학생지원시스템 세션, AI(설치·계정 상태)의 출력 DTO를 같은 연결 행 ViewModel(상태 종류, 점, 행동)로 바꾸고, 문제 상태의 개수(사이드바 연결 알약)를 낸다. 온보딩 카드, 설정 행, 알약, 대화 안 안내가 모두 이 결과를 쓴다([UX Strategy §7.5](../06_ux_strategy.md#75-설정)).
  - **대화별 상태**: 대화의 AI 진행(응답 중, 사용 한도, 실패, 보지 않은 완료), 그 대화에서 시작한 작업, 그 대화에 걸린 확인 요청(`UserConfirmation`, `OtpPrompt`, 권한 확인)을 받아 상태 하나를 우선순위대로 고른다([UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)).
  - 대화 ID는 IPC 토큰이 대화 단위이므로 코어가 안다([entrypoints §3](04_entrypoints.md#3-ipc)). 작업과 확인 요청은 시작한 대화 ID를 함께 가진다([application §4.4](02_application.md#44-사용자-상호작용과-로컬-저장), [§4.6](02_application.md#46-작업-오래-걸리는-유스케이스)).

## 2. 구조

```
View(SPA) ─command─▶ Controller(tauri command) ─▶ 유스케이스
   ▲                                                  │ 출력 DTO
   └──── ViewModel(JSON) ◀── Presenter ◀──────────────┘
```

| 역할 | 위치 | 하는 일 |
|---|---|---|
| View | 프런트엔드 SPA 컴포넌트 | ViewModel을 그리고 입력을 command로 보낸다. 조건 분기·계산·변환을 하지 않는다 |
| ViewModel 상태 보관 | 프런트엔드의 얇은 store | 받은 ViewModel·이벤트를 보관하고 View에 연결한다. 값을 새로 만들지 않는다 |
| ViewModel 타입 | `presentation/presenters/*` | 화면용 DTO(과목 카드, 배너 종류, 버튼 활성 여부, 표시용 마감 문구 등) |
| Presenter | `presentation/presenters/*` | 출력 DTO·오류를 ViewModel로 바꾼다. 어떤 배너·안내 행동을 줄지 같은 판단을 한다 |
| Controller | `presentation/tauri_commands` | 입력을 받아 유스케이스를 부르고 Presenter에 넘긴다. 로직 없음 |

- ViewModel은 Clean Architecture의 뜻으로, Presenter가 만든 동작 없는 데이터다. MVVM(Android·iOS)의 ViewModel처럼 상태 변경이나 동작을 담지 않는다.
- 판단이 들어간 표시 문구는 Presenter에서, 순수 서식(로캘 숫자·날짜 포맷)은 View에서 만든다.
- ViewModel 타입은 Rust에서 TypeScript로 생성해 어긋나지 않게 한다(`ts-rs`·`specta` 등, 미조사).
- Tauri capabilities로 WebView가 부를 수 있는 command를 제한한다.
- AI용 Presenter는 이 레이어가 아니라 [entrypoints](04_entrypoints.md)에 있다.

### 2.1 프런트엔드 SPA 내부 구조

화면을 그리는 계층을 theme · components · screens로 나누고, ViewModel 타입, ViewModel 보관, command 호출을 따로 둔다.

```
ui/src/
  theme/       CSS 변수 원본. t3code.css(T3 Code에서 가져옴, 고치지 않음), app.css(우리 것)
  components/  재사용 UI. props는 ViewModel 조각이다
    ui/        T3 Code에서 가져온 기본 부품(Base UI 래퍼). 고치지 않는다
  lib/         가져온 부품이 부르는 화면 보조 모듈(클래스 합치기, 크기 관찰, 타이틀바 높이 등)
  hooks/       가져온 부품이 부르는 화면 보조 훅
  screens/     컴포넌트 조립과 store 연결만 한다
  stores/      ViewModel 보관. 값을 새로 만들지 않는다
  viewmodels/  Rust에서 생성한 ViewModel·이벤트 타입. 손으로 고치지 않는다
  commands/    command 래퍼. 실제 구현(Tauri)과 샘플 구현(§2.4)
```

| 폴더 | 의존할 수 있는 것 | 하지 않는 것 |
|---|---|---|
| `theme` | 없음 | 컴포넌트·화면 코드 |
| `viewmodels` | 없음(생성 코드) | 손으로 고치기 |
| `components` | `theme`, `viewmodels`, `lib`, `hooks` | store·command 호출, 조건 분기로 표시 판단 |
| `lib` | 없음 | store·command 호출, ViewModel 값에 따른 표시 판단 |
| `hooks` | 없음 | `lib`과 같다 |
| `screens` | `components`, `stores`, `theme` | command 직접 호출, ViewModel 가공 |
| `stores` | `commands`, `viewmodels` | 표시 판단, 값 계산, 파생 값(selector 계산) |
| `commands` | `viewmodels`, Tauri API, 샘플 구현(`commands/sample/`)만 `design/samples/`. 샘플 구현을 부르는 것은 `commands/core.ts` 하나이고, 샘플 구현이 `commands/`에서 부를 수 있는 것은 `types.ts`뿐이다(§2.4) | 상태 보관 |

- `lib/`와 `hooks/`는 T3 Code 부품이 `~/lib/utils`, `~/hooks/useCommitOnBlur`처럼 부르는 원본 경로를 그대로 둔 것이다. 가져온 부품을 고치지 않으려고 별칭 `~`(`ui/src`)를 `tsconfig.json`과 `vite.config.ts`에 둔다. 우리 코드는 상대 경로를 쓴다. 가져온 파일과 고친 파일은 `ui/UPSTREAM.md`에 있다.
- 화면 상태에 따른 표시 판단(배너 종류, 버튼 활성 여부)은 ViewModel 필드로 받는다. 컴포넌트는 필드 값을 그대로 그린다(`banner.kind`에 맞는 변형을 고르는 것까지는 View의 일이다).
- 사용자 동작은 `screens` → store 액션 → `commands` 래퍼 순서로 코어에 간다. 응답과 코어 이벤트는 store에 들어가고 `screens`가 store를 구독해 그린다.
- `viewmodels/`는 화면과 코어 사이의 계약이다. command 응답뿐 아니라 이벤트(AI 스트림, 진행률)의 타입도 여기에 있다.
- `commands/`는 WebView ↔ 코어의 Tauri command 경로다. `cdx` ↔ 코어의 IPC([entrypoints](04_entrypoints.md))와 다르다.
- 구현 순서: ViewModel 타입·Presenter(Rust, 테스트 먼저. 기대값은 `design/samples/`의 샘플 JSON이다 — [Design Process §2.3](../03_design_process.md#23-목업)) → TypeScript 타입 생성(`viewmodels/`) → `theme` → `components` → `screens`. 로직을 먼저 만들고 화면은 테스트가 필요 없을 만큼 얇게 얹는다(Humble Object).
- **Presenter 테스트가 샘플 JSON을 읽는 방법**: `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/<저장소 루트까지의 상대 경로>/design/samples/<기능>/<상태>.json"))`로 컴파일 때 넣는다. cargo가 이 파일을 의존 정보에 기록하므로 샘플을 고치면 테스트가 다시 빌드된다. 실행 중에 파일을 읽지 않는다(cargo가 변경을 모른다). 비교는 문자열이 아니라 JSON 값으로 한다(`serde_json::Value`, 키 순서·공백 무시). `include_str!`은 파일을 하나씩 적어야 하므로 **새 상태를 더하면 테스트 케이스도 함께 더한다**(입력 DTO는 상태마다 다르므로 어차피 손으로 쓴다). 빠뜨림을 막기 위해 crate마다 테스트 하나가 샘플 폴더의 파일 목록과 등록된 케이스 목록이 같은지 확인한다. CI에서는 `design/samples/**` 변경이 `presenters/*` 테스트를 돌린다([build_ci §5.2](../cross_cutting/03_build_ci.md#52-영향-범위-계산-cargo-xtask-affected)).
- 위 의존 방향은 `eslint-plugin-boundaries`로 검사해 편집 직후 훅과 CI에 건다. `ui/` 밖을 참조하는 것은 `commands/`의 샘플 구현이 `design/samples/`를 읽는 경우 하나뿐이며, 이 예외만 규칙에 둔다(§2.4).

### 2.2 디자인 토큰

- 원본은 **CSS 변수**다. T3 Code 기준 버전의 `apps/web/src/index.css`(Tailwind v4 `@theme`, `:root`, `.dark` 변형)를 `ui/src/theme/t3code.css`로 그대로 가져온다. 기준 버전과 가져온 파일은 `ui/UPSTREAM.md`에 기록한다([UX Strategy §3](../06_ux_strategy.md#3-기준-디자인-t3-code), [Design Process §2.1](../03_design_process.md#21-foundation)).
- 우리 값(한글 글꼴, 우리 앱만의 상태 색 등)은 `ui/src/theme/app.css`에서 더하거나 덮어쓴다. 가져온 파일은 고치지 않는다. 기준 버전을 올릴 때 가져온 파일을 통째로 바꾸고 차이를 확인한다.
- 컴포넌트는 CSS 변수와 Tailwind 유틸리티만 쓰고 값을 직접 적지 않는다.
- 테마는 시스템(기본, `prefers-color-scheme`)·라이트·다크 중에서 설정으로 고른다. 루트 요소의 `.dark` 클래스로 적용한다. 테마는 코어의 판단이 필요 없는 화면 전용 값이므로 코어 설정(`SettingsRepository`)에 두지 않고 프런트엔드 로컬 저장(WebView `localStorage`)에 둔다. 읽고 쓰는 일은 `stores/`의 테마 store 하나만 하며, ViewModel 보관 규칙의 유일한 예외다. T3 Code의 실행 중 색 계산(`themePalette.ts` 등)은 가져오지 않고 기본 테마 값만 쓴다.
- 기반 컴포넌트는 Base UI다. T3 Code의 `components/ui/`(Base UI 래퍼) 중 T3 Code 내부(앱 상태, 내부 패키지, 브랜드 자산)에 묶이지 않은 부품을 그대로 가져와 `ui/src/components/ui/`에 둔다. 가져오지 않은 부품과 이유는 `ui/UPSTREAM.md`에 있다. 아이콘은 `lucide-react`를 쓴다.
- 가져온 기본 부품 안의 상호작용 처리(열림 상태, 포커스 이동 등)는 View의 일이다. ViewModel 값에 따른 표시 판단은 여전히 Presenter가 한다.

### 2.3 신뢰하지 않는 HTML과 WebView 보안

공지·과제 설명·게시판 본문 같은 학교 HTML은 다른 사람이 쓴 내용이다. WebView에서 스크립트가 실행되면 같은 WebView의 command(쓰기 승인 포함)를 부를 수 있으므로 다음을 지킨다.

| 규칙 | 내용 |
|---|---|
| 정화 | 학교 HTML은 Presenter(Rust)에서 허용 목록 방식으로 정화한 뒤 ViewModel에 넣는다. 스크립트, 이벤트 속성, `javascript:` 링크, iframe, form을 지운다 |
| 렌더링 | 정화된 HTML은 스크립트가 없는 샌드박스 iframe(`sandbox` 속성, `allow-scripts` 없음)에 그린다 |
| CSP | `script-src 'self'`(인라인 스크립트 금지), `connect-src`는 Tauri IPC만, 외부 이미지는 허용 목록 호스트만. `data:` 이미지는 허용한다(`img-src 'self' data:`). 가져온 테마의 바탕 결 무늬가 `data:` URL이기 때문이다(`ui/UPSTREAM.md` §7) |
| capabilities | WebView가 부를 수 있는 command를 Tauri capabilities로 제한한다 |
| 확인 화면 | `UserConfirmation`의 내용은 코어가 만든 값만 그린다. 승인 command는 요청 ID·내용 해시를 함께 보낸다([application §4.4](02_application.md)) |
| 링크 | 외부 링크는 OS 브라우저로 연다. 앱 WebView 안에서 이동하지 않는다 |

### 2.4 샘플 모드

[Design Process ③](../03_design_process.md#23-목업)에서 코어 없이 화면을 그리기 위한 실행 방식이다.

| 항목 | 내용 |
|---|---|
| 구조 | 코어를 부르는 통로의 모양은 `commands/types.ts`의 `Core`(`invoke`, `listen`)다. 실제 구현(`commands/tauri.ts`: Tauri `invoke`, 코어 이벤트 구독)과 샘플 구현(`commands/sample/`)이 이 모양을 지킨다. 기능의 command 래퍼(`commands/<기능>.ts`)는 한 번만 쓰고 `commands/core.ts`의 `invoke`·`listen`을 부른다. 어느 구현을 붙일지는 `core.ts`만 고르므로 command 래퍼·`stores`·`screens`는 어느 쪽이 붙었는지 모른다 |
| 고르는 방법 | 환경 변수 `VITE_SAMPLE_MODE=1`로 고른다. `npm --prefix ui run dev:sample`이 이 값을 넣어 Vite 개발 서버를 띄운다. 개발 서버(`import.meta.env.DEV`)에서만 켜지고, 배포 빌드에서는 값을 넣어도 켜지지 않는다 |
| 배포 빌드와의 경계 | `core.ts`가 샘플 구현을 샘플 모드에서만 실행되는 동적 import로 부른다. 배포 빌드에서는 그 갈래가 지워져 샘플 구현과 `design/samples/`가 결과에 들어가지 않는다. 들어가면 Vite 플러그인(`ui/sample-guard.js`)이 `vite build`를 실패시키므로, 배포 빌드를 돌리는 CI와 배포가 모두 막힌다. 플러그인이 실제로 막는지는 `ui/test/sample-guard.test.js`가 확인한다 |
| 샘플 구현 | 고른 상태의 `design/samples/<기능>/<상태>.json`을 command 응답으로 돌려준다. 어느 command가 어느 기능의 샘플을 받는지는 `commands/sample/responses.ts`의 표에 적고, 표에 없는 command는 실패한다 |
| 상태 고르기 | 화면 오른쪽 아래 패널에서 기능마다 상태를 고른다. 고른 상태는 주소에 담기고(`?sample.<기능>=<상태>`) 화면을 새로 불러온다. 주소에 없으면 이름 순 첫 상태다. 패널은 샘플 구현이 React 화면 밖에 DOM으로 직접 그린다(`components`·`screens`가 샘플 모드를 모르게 하기 위해서다) |
| 이벤트 | 시간에 따라 바뀌는 것(AI 스트림, 작업 진행)은 `<상태>.events.json`에 `[{ "afterMs": 300, "event": "<이벤트 이름>", "payload": {} }]` 모양으로 적는다. `afterMs`는 앞 이벤트 뒤에 기다릴 시간(ms)이다. 화면이 그 이벤트를 처음 구독할 때부터 순서대로 한 번 흘려보낸다. 파일에 이벤트 이름이 여럿이면 그중 하나를 처음 구독할 때 파일 전체가 흐르기 시작한다. 시작한 뒤에는 멈출 수 없고, 구독을 끊으면 받지만 않는다 |
| 파일을 고쳤을 때 | 샘플 파일을 고치거나 더하거나 지우면 화면을 통째로 새로 불러온다. 파일 모양이 틀리면 패널에 파일과 이유를 보여 준다 |
| 한계 | 브라우저에서 보므로 WKWebView와 다를 수 있고, Tauri 전용 기능은 동작하지 않는다. 실제 앱 확인은 [Design Process ⑤](../03_design_process.md#24-실제-앱-확인) |

## 3. 대화 화면의 슬래시 커맨드

| 분류 | 예 | 처리 |
|---|---|---|
| 앱이 대신 처리 | `/clear`, `/login`, `/model` | 새 대화, 제공자 로그인 화면, 모델 선택(`session_spec`의 모델 값)으로 연결. 대화 기록의 원본은 앱이다 |
| 그대로 전달 | `/compact`, 사용자 커스텀 커맨드, 스킬·플러그인 커맨드 | 제공자 CLI에 그대로 보낸다 |
| 대화형 전용 | `/config`, `/mcp` 화면, `/resume` 선택기 | "앱에서 지원하지 않음" 안내 |

- 분류는 제공자마다 다르므로 `AiSession.capabilities()`로 받는다([application §4.2](02_application.md)). 사용할 수 있는 커맨드·스킬 목록은 대화 시작 이벤트에서 받아 입력창 자동 완성에 쓴다.
- AI가 요청한 권한 확인(`PermissionRequest`)과 개인 데이터 쓰기 확인(`UserConfirmation`)은 앱 화면에서 받는다.

## 4. 이웃 레이어와의 계약

| 이웃 | 계약 |
|---|---|
| application | `usecases/*`의 유스케이스만 호출한다. 출력 DTO를 받는다. `tauri_commands`가 `UserConfirmation`·`OtpPrompt`·`ProgressSink` 포트를 구현한다 |
| 프런트엔드 | command 이름과 ViewModel 타입(생성된 TypeScript)이 계약이다 |

## 5. 미확인 항목

- Rust → TypeScript 타입 생성 도구
- 샘플 JSON과 이벤트 파일(이벤트 이름, payload)이 ViewModel·이벤트 타입과 맞는지 검사하는 방법. 샘플 구현은 JSON을 타입 확인 없이 그대로 돌려주므로, 값이 타입과 어긋나도 타입 검사·lint·테스트가 모두 통과하고 화면만 깨진다. 실제 기능은 Presenter 테스트(샘플 JSON = 기대값)가 모양을 확인하지만, Presenter가 없는 ③ 단계와 이벤트 파일은 아무도 확인하지 않는다. 타입 생성 도구와 함께 정한다
