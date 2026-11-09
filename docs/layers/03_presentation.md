# Presentation

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../02_system_architecture.md) |

> **이 문서의 역할**: 화면(WebView)과 코어 사이의 View · ViewModel · Presenter · Controller 역할과 위치를 정한다. 화면 구성과 흐름은 UX Strategy·User Flow가 맡는다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Interface Adapters (View는 Frameworks & Drivers) |
| crate | `presentation/presenters/{common, school, ai, learning}`, `presentation/tauri_commands` (+ 프런트엔드 SPA) |
| 의존할 수 있는 것 | `presenters/common`: `application/kernel`. `presenters/<묶음>`: 같은 묶음의 `application/usecases`, `presenters/common`, `kernel`. `tauri_commands`: `usecases/*`, `presenters/*`, `ports/common`·`ports/school`(구현), `kernel`, `tauri` |

- Presenter·ViewModel은 순수 로직이고 테스트를 먼저 쓰므로 `tauri`에 의존하지 않는 `presenters/*`에 둔다. Presenter 테스트를 돌릴 때 Tauri를 컴파일하지 않는다. Tauri command는 로직 없는 `tauri_commands`에 둔다(Humble Object).
- `presenters`는 `usecases`와 같은 묶음으로 나눈다. 한 묶음의 유스케이스를 고쳐도 다른 묶음의 Presenter는 다시 빌드되지 않는다([build_ci §2](../cross_cutting/03_build_ci.md#2-crate를-나누는-기준)).

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
  theme/       디자인 토큰(tokens.json 원본)과 생성된 CSS 변수
  components/  재사용 UI. props는 ViewModel 조각이다
  screens/     컴포넌트 조립과 store 연결만 한다
  stores/      ViewModel 보관. 값을 새로 만들지 않는다
  viewmodels/  Rust에서 생성한 ViewModel·이벤트 타입. 손으로 고치지 않는다
  commands/    Tauri command 래퍼
```

| 폴더 | 의존할 수 있는 것 | 하지 않는 것 |
|---|---|---|
| `theme` | 없음 | 컴포넌트·화면 코드 |
| `viewmodels` | 없음(생성 코드) | 손으로 고치기 |
| `components` | `theme`, `viewmodels` | store·command 호출, 조건 분기로 표시 판단 |
| `screens` | `components`, `stores`, `theme` | command 직접 호출, ViewModel 가공 |
| `stores` | `commands`, `viewmodels` | 표시 판단, 값 계산, 파생 값(selector 계산) |
| `commands` | `viewmodels`, Tauri API | 상태 보관 |

- 화면 상태에 따른 표시 판단(배너 종류, 버튼 활성 여부)은 ViewModel 필드로 받는다. 컴포넌트는 필드 값을 그대로 그린다(`banner.kind`에 맞는 변형을 고르는 것까지는 View의 일이다).
- 사용자 동작은 `screens` → store 액션 → `commands` 래퍼 순서로 코어에 간다. 응답과 코어 이벤트는 store에 들어가고 `screens`가 store를 구독해 그린다.
- `viewmodels/`는 화면과 코어 사이의 계약이다. command 응답뿐 아니라 이벤트(AI 스트림, 진행률)의 타입도 여기에 있다.
- `commands/`는 WebView ↔ 코어의 Tauri command 경로다. `cdx` ↔ 코어의 IPC([entrypoints](04_entrypoints.md))와 다르다.
- 구현 순서: ViewModel 타입·Presenter(Rust, 테스트 먼저. 기대값은 `design/samples/`의 샘플 JSON이다 — [Design Process §2.3](../03_design_process.md#23-목업)) → TypeScript 타입 생성(`viewmodels/`) → `theme` → `components` → `screens`. 로직을 먼저 만들고 화면은 테스트가 필요 없을 만큼 얇게 얹는다(Humble Object).
- **Presenter 테스트가 샘플 JSON을 읽는 방법**: `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/<저장소 루트까지의 상대 경로>/design/samples/<기능>/<상태>.json"))`로 컴파일 때 넣는다. cargo가 이 파일을 의존 정보에 기록하므로 샘플을 고치면 테스트가 다시 빌드된다. 실행 중에 파일을 읽지 않는다(cargo가 변경을 모른다). 비교는 문자열이 아니라 JSON 값으로 한다(`serde_json::Value`, 키 순서·공백 무시). CI에서는 `design/samples/**` 변경이 `presenters/*` 테스트를 돌린다([build_ci §5.2](../cross_cutting/03_build_ci.md#52-영향-범위-계산-cargo-xtask-affected)).
- 위 의존 방향은 `eslint-plugin-boundaries`로 검사해 편집 직후 훅과 CI에 건다.

### 2.2 디자인 토큰

- 원본은 `ui/src/theme/tokens.json`(DTCG 형식) 하나다. 앱이 import하므로 앱 안에 둔다.
- 토큰은 원시 토큰(색 단계, 간격 단계)과 이를 가리키는 의미 토큰(`surface`, `textMuted`, `warningSurface` 등) 2단으로 둔다. 컴포넌트는 의미 토큰만 쓴다.
- 빌드 도구(Style Dictionary 등, 미정)가 원본에서 `tokens.css`(CSS 변수)를 만든다. 컴포넌트는 CSS 변수만 쓰고 값을 직접 적지 않는다.
- `design/`의 HTML 목업도 같은 `tokens.css`를 불러온다. 목업에서 고친 값이 앱에 그대로 반영된다. 디자인 절차는 [Design Process](../03_design_process.md)에 있다.
- 라이트·다크 모드는 의미 토큰이 가리키는 원시 토큰을 두 벌로 두고 OS 설정(`prefers-color-scheme`)을 따른다.
- `components`는 키보드 조작·포커스·스크린 리더 지원을 갖춘 스타일 없는 컴포넌트 라이브러리 위에 만들고, 겉모습만 토큰으로 입힌다(라이브러리는 미확인).

### 2.3 신뢰하지 않는 HTML과 WebView 보안

공지·과제 설명·게시판 본문 같은 학교 HTML은 다른 사람이 쓴 내용이다. WebView에서 스크립트가 실행되면 같은 WebView의 command(쓰기 승인 포함)를 부를 수 있으므로 다음을 지킨다.

| 규칙 | 내용 |
|---|---|
| 정화 | 학교 HTML은 Presenter(Rust)에서 허용 목록 방식으로 정화한 뒤 ViewModel에 넣는다. 스크립트, 이벤트 속성, `javascript:` 링크, iframe, form을 지운다 |
| 렌더링 | 정화된 HTML은 스크립트가 없는 샌드박스 iframe(`sandbox` 속성, `allow-scripts` 없음)에 그린다 |
| CSP | `script-src 'self'`(인라인 스크립트 금지), `connect-src`는 Tauri IPC만, 외부 이미지는 허용 목록 호스트만 |
| capabilities | WebView가 부를 수 있는 command를 Tauri capabilities로 제한한다 |
| 확인 화면 | `UserConfirmation`의 내용은 코어가 만든 값만 그린다. 승인 command는 요청 ID·내용 해시를 함께 보낸다([application §4.4](02_application.md)) |
| 링크 | 외부 링크는 OS 브라우저로 연다. 앱 WebView 안에서 이동하지 않는다 |

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
- 토큰 빌드 도구(Style Dictionary 등)와 `tokens.css` 생성물의 커밋 여부
- 기반 컴포넌트 라이브러리(스타일 없는 접근성 컴포넌트)
