# CanvasDongseo — System Architecture

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD](01_prd.md), 레이어별 문서(§5) |

> **이 문서의 역할**: 여러 레이어에 걸치는 것만 정한다. 결정 요약, 프로세스 구조, 레이어 지도와 의존 규칙, 용어, 문서 색인.
> 레이어 안의 상세는 [레이어별 문서](#5-문서-색인)에, 레이어를 가로지르는 주제(보안, 플랫폼)는 공통 문서에 둔다. 한 가지 사실은 한 문서에만 적고 나머지는 링크한다.

---

## 1. 결정 요약

| # | 결정 | 근거 | 상세 |
|---|---|---|---|
| S1 | 데스크톱 셸은 **Tauri 2**, 코어는 **Rust**로 한다 | 크로스플랫폼(macOS·Windows), OS 웹뷰라 메모리·설치 용량이 작다. 코어가 Rust 그대로라 연결층이 없다. 타입으로 오류 계약을 강제하기 쉽다 | [platform](cross_cutting/02_platform.md) |
| S2 | 레이어마다 **Cargo crate**를 나눠 의존 방향을 컴파일러가 강제한다 | 금지된 의존은 빌드가 되지 않는다 | §3 |
| S3 | **MCP 서버를 두지 않는다.** AI는 앱이 제공하는 CLI `cdx`를 Bash 도구로 호출하고, `cdx`는 실행 중인 코어에 로컬 IPC로 요청한다 | 학교 세션(쿠키가 쓸 때마다 바뀜)은 코어 하나만 가져야 한다. `cdx`는 세션을 갖지 않는다 | [entrypoints](layers/04_entrypoints.md) |
| S4 | `cdx` 사용법은 **시스템 프롬프트**로 알려 준다(스킬 문서가 아님) | 제공자와 무관하게 같은 텍스트(`ToolGuide`)를 주입할 수 있다 | [entrypoints](layers/04_entrypoints.md) |
| S5 | AI 제공자는 **역할별 포트(`AiInstallation`·`AiAccount`·`AiSession`) + 제공자별 어댑터 crate**로 둔다. MVP는 Claude Code | 다른 AI 구독을 기존 코드 수정 없이 추가한다([PRD §8.1](01_prd.md)) | [application](layers/02_application.md), [ai_adapters](layers/05_ai_adapters.md) |
| S6 | AI 로그인은 **제공자 CLI에 위임**한다. 앱은 AI 자격 증명을 저장·중계하지 않는다 | 약관(자격 증명 수집·중계 금지), 터미널 로그인 재사용 | [ai_adapters](layers/05_ai_adapters.md) |
| S7 | 대화는 **사용자 환경 그대로**(스킬·플러그인·커맨드·MCP) 실행하고, 무인 작업만 격리한다 | 학생은 앱 고유 기능 밖의 CLI 기능(PDF·PPTX 스킬, Computer Use, `/compact` 등)도 쓰려 한다 | [ai_adapters](layers/05_ai_adapters.md) |
| S8 | **학교 데이터·비밀·쓰기의 경계는 코어에 둔다.** AI의 도구 사용은 막지 않고, 다른 사람이 쓴 글을 읽은 대화에만 격리 요약과 시점 경고를 더한다. 비밀은 Data Protection 키체인(앱 접근 그룹)에 둔다 | S7로 대화 중 AI가 임의 명령을 실행할 수 있다. 제공자마다 권한 제어가 다르다. 책임 범위는 외부 탈취, 다른 학생 데이터 접근, 주입에 의한 위험 행동 | [security](cross_cutting/01_security.md) |
| S9 | 학습 기능(전사·OCR·슬라이드 추출)은 **OS별 crate**로 나누고 macOS를 먼저 구현한다 | OS 고유 API·가속 의존성을 경계 뒤에 가둔다 | [learning_adapters](layers/06_learning_adapters.md) |
| S10 | 유스케이스는 domain 엔티티가 아니라 **유스케이스 출력 DTO**를 돌려준다. Presenter가 이를 화면용 ViewModel·AI용 출력으로 바꾼다 | 소비자(화면, AI)가 둘이라 엔티티 변경이 바깥으로 번지지 않게 한다(Clean Architecture의 Response Model) | [application](layers/02_application.md), [presentation](layers/03_presentation.md) |

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

```
crates/
  domain/                 엔티티, 업무 규칙
  application/            유스케이스, 출력 DTO, 출력 포트 <I>
  data_access/
    repositories/         출력 포트 구현, DS <I>와 계약, 공통 오류
    shared/               인증·세션·HTTP 정책
    canvas/ learningx/ smart_attendance/ commons/ school_site/ student_portal/   (Source)
  ai/
    claude_code/          AI 포트 구현 (MVP)
    (codex/ gemini/ …)    추가 제공자
  learning/
    macos/  (windows/)    학습 포트 구현
  local_store/            앱 로컬 저장(대화 기록, 설정) 포트 구현
  presentation/           Tauri command(Controller), 화면용 Presenter·ViewModel
  entrypoints/
    ipc_server/           cdx 요청 ↔ 유스케이스, AI용 Presenter
  ipc_protocol/           cdx ↔ ipc_server 요청·응답 타입 (비밀 필드 없음)
  bin/
    app/                  Tauri 앱 + composition root
    cdx/                  CLI
ui/                       프런트엔드 SPA(React). View와 ViewModel 보관 (presentation §2.1)
design/                   디자인 목업·샘플 JSON (Design Process)
```

### 3.2 의존 규칙 (Cargo `[dependencies]`로 강제)

| crate | 의존할 수 있는 것 |
|---|---|
| `domain` | 기초 crate만. 외부 crate는 `jiff`만 |
| `application` | `domain` |
| `data_access/*`, `ai/*`, `learning/*`, `local_store` | `application`, `domain`. data_access 내부(`repositories`, `shared`, Source) 규칙은 [data_access §2.6](layers/data_access/README.md) |
| `presentation` | `application` |
| `entrypoints/*` | `application`, `ipc_protocol` |
| `ipc_protocol` | 기초 crate만(`serde`, `clap`) |
| `bin/app` | 전부 (composition root) |
| `bin/cdx` | `ipc_protocol`만. 코어 로직·세션·학교 서버 접근이 없다 |

- 의존 그래프(`cargo metadata`)를 위 표와 대조하는 검사를 편집 직후 훅과 CI에 둔다. `cargo deny`도 함께 돌린다.
- 확장 근거: AI 제공자·데이터 출처·학습 기능이 늘어날 계획이 PRD에 있으므로, 처음부터 crate 경계를 둔다.

### 3.3 Clean Architecture 링 대응

| 링 | 이 프로젝트 |
|---|---|
| Entities | `domain` |
| Use Cases | `application` (유스케이스, 출력 DTO, 출력 포트) |
| Interface Adapters | `presentation`(Controller·Presenter·ViewModel), `entrypoints/ipc_server`(AI용 Presenter), `data_access/repositories`(출력 포트 구현), AI 어댑터의 이벤트 변환 |
| Frameworks & Drivers | Tauri, WebView SPA(View), `data_access`의 Source와 `shared`(HTTP·키체인), AI CLI 프로세스, OS 학습 API, `local_store` |
| Main | `bin/app` (composition root) |

## 4. 용어

| 용어 | 뜻 |
|---|---|
| 출력 포트 | 유스케이스가 바깥을 부르는 인터페이스. 엔티티 컬렉션은 **Repository**, 그 밖의 외부 동작은 **Gateway** |
| 출력 DTO | 유스케이스가 돌려주는 값. 엔티티를 그대로 내보내지 않는다 |
| Source | data_access의 접근 구역별 출처 어댑터(`canvas`, `learningx` …). DS 구현을 담는다 |
| DS (DataSource) | Source가 구현하는 리소스 묶음 단위 인터페이스. 소유는 `data_access/repositories` |
| Presenter | 출력 DTO를 소비자(화면, AI)에 맞는 형태로 바꾸는 구성 |
| `ToolGuide` | AI에게 주입하는 `cdx` 사용법 텍스트 |
| `<I>` | 인터페이스(Rust trait) |

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

**절차**

| 문서 | 주제 |
|---|---|
| [Design Process](03_design_process.md) | 와이어프레임부터 화면별 토큰까지 디자인 단계와 산출물 |
