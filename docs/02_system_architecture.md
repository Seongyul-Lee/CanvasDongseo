# CanvasDongseo — System Architecture

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD](01_prd.md), 레이어별 문서(§5) |

> **이 문서의 역할**: 여러 레이어에 걸치는 것만 정한다. 결정 요약, 프로세스 구조, 레이어 지도와 의존 규칙, 용어, 문서 색인.
> 레이어 안의 상세는 [레이어별 문서](#5-문서-색인)에, 레이어를 가로지르는 주제(보안, 플랫폼, 빌드·CI)는 공통 문서에 둔다. 한 가지 사실은 한 문서에만 적고 나머지는 링크한다.

---

## 1. 결정 요약

| # | 결정 | 근거 | 상세 |
|---|---|---|---|
| S1 | 데스크톱 셸은 **Tauri 2**, 코어는 **Rust**로 한다 | 크로스플랫폼(macOS·Windows), OS 웹뷰라 메모리·설치 용량이 작다. 코어가 Rust 그대로라 연결층이 없다. 타입으로 오류 계약을 강제하기 쉽다 | [platform](cross_cutting/02_platform.md) |
| S2 | 레이어와 그 안의 책임 단위를 **Cargo crate**로 나눠 의존 방향을 컴파일러가 강제한다. 나누는 기준은 S11 | 금지된 의존은 빌드가 되지 않는다 | §3 |
| S3 | **MCP 서버를 두지 않는다.** AI는 앱이 제공하는 CLI `cdx`를 Bash 도구로 호출하고, `cdx`는 실행 중인 코어에 로컬 IPC로 요청한다 | 학교 세션(쿠키가 쓸 때마다 바뀜)은 코어 하나만 가져야 한다. `cdx`는 세션을 갖지 않는다 | [entrypoints](layers/04_entrypoints.md) |
| S4 | `cdx` 사용법은 **시스템 프롬프트**로 알려 준다(스킬 문서가 아님) | 제공자와 무관하게 같은 텍스트(`ToolGuide`)를 주입할 수 있다 | [entrypoints](layers/04_entrypoints.md) |
| S5 | AI 제공자는 **역할별 포트(`AiInstallation`·`AiAccount`·`AiSession`) + 제공자별 어댑터 crate**로 둔다. MVP는 Claude Code | 다른 AI 구독을 기존 코드 수정 없이 추가한다([PRD §8.1](01_prd.md)) | [application](layers/02_application.md), [ai_adapters](layers/05_ai_adapters.md) |
| S6 | AI 로그인은 **제공자 CLI에 위임**한다. 앱은 AI 자격 증명을 저장·중계하지 않는다 | 약관(자격 증명 수집·중계 금지), 터미널 로그인 재사용 | [ai_adapters](layers/05_ai_adapters.md) |
| S7 | 대화는 **사용자 환경 그대로**(스킬·플러그인·커맨드·MCP) 실행하고, 무인 작업만 격리한다 | 학생은 앱 고유 기능 밖의 CLI 기능(PDF·PPTX 스킬, Computer Use, `/compact` 등)도 쓰려 한다 | [ai_adapters](layers/05_ai_adapters.md) |
| S8 | **학교 데이터·비밀·쓰기의 경계는 코어에 둔다.** AI의 도구 사용은 막지 않고, 다른 사람이 쓴 글을 읽은 대화에만 격리 요약과 시점 경고를 더한다. 비밀은 Data Protection 키체인(앱 접근 그룹)에 둔다 | S7로 대화 중 AI가 임의 명령을 실행할 수 있다. 제공자마다 권한 제어가 다르다. 책임 범위는 외부 탈취, 다른 학생 데이터 접근, 주입에 의한 위험 행동 | [security](cross_cutting/01_security.md) |
| S9 | 학습 기능은 **OCR만 OS별 crate**로 나누고, 디코딩(ffmpeg)·전사(whisper.cpp)·PDF는 공통 crate로 둔다. 슬라이드 판정은 domain 규칙으로 자동 확정한다(AI 검토 없음). macOS를 먼저 구현한다 | OS 고유 API가 필요한 것은 OCR뿐이다. 2026-11-09 실측으로 디코딩·전사 방식을 정했다 | [learning_adapters](layers/06_learning_adapters.md) |
| S10 | 유스케이스는 domain 엔티티가 아니라 **유스케이스 출력 DTO**를 돌려준다. Presenter가 이를 화면용 ViewModel·AI용 출력으로 바꾼다 | 소비자(화면, AI)가 둘이라 엔티티 변경이 바깥으로 번지지 않게 한다(Clean Architecture의 Response Model) | [application](layers/02_application.md), [presentation](layers/03_presentation.md) |
| S11 | crate는 **Actor(바뀌는 이유)와 변경 빈도**로 나눈다. 포트(안정)와 유스케이스(자주 바뀜)를 다른 crate에 두고, 어댑터는 포트에만 의존한다. CI는 바뀐 crate와 그 역의존 crate만 빌드·테스트한다 | 한 crate가 바뀌면 의존하는 crate가 모두 다시 빌드된다. 자주 바뀌는 코드가 많이 의존받는 crate에 있으면 작은 변경이 전체 빌드가 된다 | [build_ci](cross_cutting/03_build_ci.md) |

## 2. 프로세스 구조

```
┌──────────────────────── Tauri 앱 ────────────────────────┐
│  WebView (채팅 UI)  ──Tauri command──▶  Rust 코어          │
│                                       · 학교 세션 유일 소유자 │
│                                       · 유스케이스·data_access│
│                                       · IPC 서버            │
└────────────────────────────▲──────────────────────────────┘
                             │ Unix 소켓(macOS) / named pipe(Windows)
                             │ + 일회용 토큰
                      ┌──────┴──────┐
                      │  cdx (CLI)  │  세션 없음, JSON 출력
                      └──────▲──────┘
                             │ Bash 도구 호출
                      ┌──────┴──────────────┐
                      │ AI 제공자 CLI 프로세스 │  코어가 자식 프로세스로 실행
                      │ (MVP: claude -p)    │  stream-json 입출력
                      └─────────────────────┘
```

- 코어가 꺼져 있으면 `cdx`는 "앱을 실행하세요"라는 오류만 낸다. 스스로 로그인하거나 세션을 만들지 않는다.
- AI 프로세스는 대화마다 코어가 띄우고 정리한다. 앱이 닫히면 자식 프로세스를 모두 끝낸다.

## 3. 레이어 지도

### 3.1 crate 구성

나누는 기준(Actor, 변경 빈도)과 변경별 재빌드 범위는 [build_ci §2](cross_cutting/03_build_ci.md#2-crate를-나누는-기준)에 있다.

```
crates/
  domain/
    core/                 엔티티, 학교 업무 규칙, 학습 결과물 이름 규칙
    slides/               슬라이드 판정 규칙 (domain §4.1)
  application/
    kernel/               application 오류, CancelToken, 공통 값 타입. 안정된 것만 둔다
    ports/
      school/             학교 데이터 Repository·Gateway, AuthGateway
      ai/                 AiInstallation·AiAccount·AiSession, 중립 이벤트, ConversationRepository
      learning/           FrameDecoder·AudioDecoder·Transcriber·TextRecognizer·SlidePdfWriter, LearningOutputRepository, ComponentStore
      common/             UserConfirmation, OtpPrompt, ProgressSink, JobSpawner, SettingsRepository
    usecases/
      school/ ai/ learning/   유스케이스와 출력 DTO (Actor별)
      jobs/               JobRegistry, 작업 상태 조회·취소
  data_access/
    ds_core/              data_access 공통 오류
    contracts/<묶음>/      DS <I>와 DS DTO, 리소스 묶음별 (예: 과목·차시, 과제, 출결, 공지, 학적·성적, 자료)
    repositories/         ports/school 구현
    http/                 HTTP 정책, RequestAuthenticator <I>
    auth/                 인증 구현(로그인·갱신·세션 갱신 정책), AuthGateway 구현, SessionStore
    canvas/ learningx/ smart_attendance/ commons/ school_site/ student_portal/   (Source)
  ai/
    claude_code/          ports/ai 구현 (MVP)
    (codex/ gemini/ …)    추가 제공자
  learning/
    media/                FrameDecoder·AudioDecoder (ffmpeg 사이드카, 공통)
    transcribe/           Transcriber (whisper.cpp, 공통)
    pdf/                  SlidePdfWriter (공통)
    macos/  (windows/)    TextRecognizer (OS OCR)
  local_store/
    conversations/        ConversationRepository 구현
    learning/             LearningOutputRepository·ComponentStore 구현
    settings/             SettingsRepository 구현
  platform/
    keychain/             SecretStore <I>, OS 키체인 구현(cfg로 OS별), 메모리 구현. OS 의존을 여기에 가둔다
    fs/                   원자적 쓰기, 앱 데이터 경로
  presentation/
    presenters/
      common/             application 오류 → 공통 ViewModel 조각
      school/ ai/ learning/ jobs/   Presenter·ViewModel (usecases와 같은 묶음)
    tauri_commands/       Tauri command(Controller), UserConfirmation·OtpPrompt·ProgressSink 구현
  entrypoints/
    ipc_server/           cdx 요청 ↔ 유스케이스, AI용 Presenter
  ipc/
    wire/                 cdx ↔ ipc_server 요청·응답 타입 (serde, 비밀 필드 없음)
    cli/                  cdx 명령 정의 (clap, ToolGuide 원본)
  bin/
    app/                  Tauri 앱 + composition root. tauri.conf.json·capabilities·아이콘도 여기에 둔다
    cdx/                  CLI
  xtask/                  의존 그래프 검사, 영향 범위 계산 (build_ci §5)
ui/                       프런트엔드 SPA(React). View와 ViewModel 보관 (presentation §2.1)
design/                   디자인 목업·샘플 JSON (Design Process)
```

### 3.2 의존 규칙 (Cargo `[dependencies]`로 강제)

| crate | 의존할 수 있는 것 |
|---|---|
| `domain/core` | 기초 crate만. 외부 crate는 `jiff`만 |
| `domain/slides` | 기초 crate만. 외부 crate 없음. `domain/core`도 모른다 |
| `application/kernel` | 기초 crate만 |
| `application/ports/*` | `kernel`, `domain/core`. 포트 crate끼리 서로 의존하지 않는다 |
| `application/usecases/learning` | `kernel`, `domain/core`, `domain/slides`, 필요한 `ports/*`, `usecases/jobs` |
| `application/usecases/jobs` | `kernel`, `domain/core`, `ports/common` |
| 그 밖의 `application/usecases/*` | `kernel`, `domain/core`, 필요한 `ports/*`. 유스케이스 crate끼리 서로 의존하지 않는다. `usecases/jobs`는 오래 걸리는 작업이 있는 묶음(현재 `usecases/learning`)만 쓴다 |
| `data_access/*` | `ports/school`, `kernel`, `domain/core`. `auth`만 `platform/keychain`도 쓴다. 유스케이스는 모른다. 내부 규칙은 [data_access §2.6](layers/data_access/README.md#26-의존-규칙) |
| `ai/*` | `ports/ai`, `kernel`, `domain/core` |
| `learning/*` | `ports/learning`, `kernel`, `domain/core` |
| `local_store/*` | 구현하는 포트 crate, `kernel`, `domain/core`, `platform/*` |
| `platform/*` | 기초 crate와 OS 연동 crate만. 앱 crate를 모른다 |
| `presentation/presenters/common` | `kernel` |
| `presentation/presenters/<묶음>` | 같은 묶음의 `usecases`, `presenters/common`, `kernel` |
| `presentation/tauri_commands` | `usecases/*`, `presenters/*`, `ports/common`(구현), `kernel` |
| `entrypoints/ipc_server` | `usecases/*`, `ipc/wire`, `kernel` |
| `ipc/wire` | 기초 crate만(`serde`) |
| `ipc/cli` | `ipc/wire`, `clap` |
| `bin/app` | 전부 (composition root) |
| `bin/cdx` | 워크스페이스 crate는 `ipc/wire`, `ipc/cli`만. 코어 로직·세션·학교 서버 접근이 없다 |
| `xtask` | 앱 crate에 의존하지 않는다. 앱 crate도 `xtask`에 의존하지 않는다 |

- 이 표는 **워크스페이스 crate 사이의** 의존을 정한다. 외부 crate(`tokio`, `interprocess` 등)를 어느 crate가 쓰는지는 [platform §1](cross_cutting/02_platform.md#1-기술-스택)에 있다. `domain/*`의 외부 crate 제한만 이 표에 둔다.
- 어댑터(`data_access`, `ai`, `learning`, `local_store`)는 **포트에만** 의존한다. 유스케이스를 고쳐도 어댑터는 다시 빌드되지 않는다.
- `domain/slides`는 `usecases/learning`만 쓴다. 포트와 어댑터는 `domain/core`까지만 안다. 그래서 슬라이드 판정 기준을 바꿔도 학습 유스케이스와 그 위만 다시 빌드된다.
- 포트는 `Arc<dyn Port>`로 주입한다. 유스케이스를 포트 타입의 제네릭으로 쓰지 않는다([build_ci §4](cross_cutting/03_build_ci.md#4-제네릭과-동적-디스패치)).
- 의존 그래프(`cargo metadata`)를 위 표와 대조하는 검사(`cargo xtask deps-check`)를 편집 직후 훅과 CI에 둔다. `cargo deny`도 함께 돌린다([build_ci §5.4](cross_cutting/03_build_ci.md#54-항상-도는-검사)).
- 확장 근거: AI 제공자·데이터 출처·학습 기능이 늘어날 계획이 PRD에 있으므로, 처음부터 crate 경계를 둔다.

### 3.3 Clean Architecture 링 대응

| 링 | 이 프로젝트 |
|---|---|
| Entities | `domain/core`, `domain/slides` |
| Use Cases | `application/kernel`, `application/ports/*`(출력 포트), `application/usecases/*`(유스케이스, 출력 DTO) |
| Interface Adapters | `presentation/presenters/*`(Presenter·ViewModel), `presentation/tauri_commands`(Controller), `entrypoints/ipc_server`(AI용 Presenter), `data_access/repositories`(출력 포트 구현), `data_access/contracts/*`(DS 계약), AI 어댑터의 이벤트 변환 |
| Frameworks & Drivers | Tauri, WebView SPA(View), `data_access`의 Source와 `http`·`auth`, `platform/*`(키체인·파일), AI CLI 프로세스, OS 학습 API, `local_store/*` |
| Main | `bin/app` (composition root) |

## 4. 용어

| 용어 | 뜻 |
|---|---|
| 출력 포트 | 유스케이스가 바깥을 부르는 인터페이스. 엔티티 컬렉션은 **Repository**, 그 밖의 외부 동작은 **Gateway** |
| 출력 DTO | 유스케이스가 돌려주는 값. 엔티티를 그대로 내보내지 않는다 |
| Source | data_access의 접근 구역별 출처 어댑터(`canvas`, `learningx` …). DS 구현을 담는다 |
| DS (DataSource) | Source가 구현하는 리소스 묶음 단위 인터페이스. 소유는 `data_access/contracts/<묶음>` |
| Presenter | 출력 DTO를 소비자(화면, AI)에 맞는 형태로 바꾸는 구성 |
| `ToolGuide` | AI에게 주입하는 `cdx` 사용법 텍스트 |
| `<I>` | 인터페이스(Rust trait) |
| Actor | 두 가지 뜻으로 쓴다. **변경 Actor**: 코드가 바뀌는 이유, crate를 나누는 기준([build_ci §2.1](cross_cutting/03_build_ci.md#21-actor)). **사용자 Actor**: 같은 목적을 가진 사용자 집단, 시나리오의 기준([UX Strategy §4](06_ux_strategy.md#4-actor와-시나리오)). 둘이 함께 나오는 곳에서는 앞말을 붙인다 |

## 5. 문서 색인

**레이어** (`layers/`) — 각 문서는 역할, 링, crate, 의존, 주요 구성, 이웃 레이어와의 계약, 미확인 항목 순서로 쓴다.

| 문서 | 레이어 |
|---|---|
| [01_domain](layers/01_domain.md) | 엔티티, 업무 규칙 |
| [02_application](layers/02_application.md) | 유스케이스, 출력 DTO, 출력 포트 카탈로그 |
| [03_presentation](layers/03_presentation.md) | View · ViewModel · Presenter · Controller |
| [04_entrypoints](layers/04_entrypoints.md) | `cdx`, IPC, `ToolGuide`, AI용 Presenter |
| [data_access](layers/data_access/README.md) | 학교 데이터 접근(출처별 문서 포함) |
| [05_ai_adapters](layers/05_ai_adapters.md) | AI 제공자 어댑터(로그인, 실행 모드, 커맨드) |
| [06_learning_adapters](layers/06_learning_adapters.md) | OS별 전사·OCR·슬라이드 추출 |

**공통** (`cross_cutting/`)

| 문서 | 주제 |
|---|---|
| [01_security](cross_cutting/01_security.md) | 보안 경계와 남는 위험 |
| [02_platform](cross_cutting/02_platform.md) | 기술 스택, OS별 구성, 설치·배포 |
| [03_build_ci](cross_cutting/03_build_ci.md) | crate 분할 기준, 로컬 빌드 설정, 제네릭 정책, 바뀐 crate만 도는 CI |

**절차**

| 문서 | 주제 |
|---|---|
| [UX Strategy](06_ux_strategy.md) | Actor·시나리오, 경험 원칙, 기준 디자인(T3 Code), 화면 구성, 문구 원칙 |
| [Design Process](03_design_process.md) | Foundation과 기능 단위 디자인 단계(User Flow → 상태 목록 → 목업 → 구현 → 실제 앱 확인 → 시나리오 점검)와 산출물 |
| [Apple Developer 계정](04_apple_developer_account.md) | 멤버십, 인증서, 프로파일, 번들 ID, 공증 자격 증명 현황 |
| [스크럼 보드](05_scrum_board.md) | 스프린트 운영 규칙, GitHub Projects 필드·View, 이슈 템플릿 |

**측정** (`measurements/`)

| 문서 | 주제 |
|---|---|
| [2026-11 강의 동영상 처리](measurements/2026-11_lecture_media_processing.md) | 키프레임 디코딩, 전사 엔진·모델 비교, 최종 결정 |
| [2026-11 키체인 허용 창](measurements/2026-11_keychain_prompts.md) | 접근 그룹·정책 적용, 서명 변형, 화면 잠금 중 읽기, `claude` CLI 키체인 읽기 |
