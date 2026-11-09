# Application

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../02_system_architecture.md), [Data Access Architecture](data_access/README.md) |

> **이 문서의 역할**: 유스케이스, 출력 DTO, 출력 포트(Repository·Gateway·AI·학습)의 정의를 둔다. **포트의 정의는 이 문서에만 둔다.** 구현은 각 어댑터 문서가 맡는다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Use Cases |
| crate | `application/kernel`, `application/ports/{school, ai, learning, common}`, `application/usecases/{school, ai, learning, jobs}` |
| 의존할 수 있는 것 | `kernel`: 기초 crate만. `ports/*`: `kernel`, `domain/core`. `usecases/*`: `kernel`, `domain/core`, 필요한 `ports/*`(`usecases/learning`만 `domain/slides`와 `usecases/jobs`도). 상세는 [System Architecture §3.2](../02_system_architecture.md#32-의존-규칙-cargo-dependencies로-강제) |

| crate | 담는 것 | 바뀌는 때 |
|---|---|---|
| `kernel` | application 오류(§4.5), `CancelToken`(§4.6), 공통 값 타입(ID 등) | 드물다. 거의 모든 crate가 의존하므로 안정된 것만 둔다 |
| `ports/<묶음>` | 출력 포트(§4.1~4.4). 어느 crate에 두는지는 각 표의 "crate" 열 | 어댑터와의 계약이 바뀔 때 |
| `usecases/<묶음>` | 유스케이스와 그 출력 DTO. `usecases/jobs`는 `JobRegistry`(§4.6)도 둔다 | 기능을 만들 때마다. 가장 자주 바뀐다 |

- 어댑터는 `ports/*`에만 의존하고, presentation·entrypoints는 `usecases/*`에 의존한다. 유스케이스와 출력 DTO를 고쳐도 어댑터는 다시 빌드되지 않는다(S11, [build_ci §2](../cross_cutting/03_build_ci.md#2-crate를-나누는-기준)).
- 포트 crate끼리, 유스케이스 crate끼리 서로 의존하지 않는다. 예외는 `usecases/jobs`로, 오래 걸리는 작업이 있는 묶음만 쓴다. 지금은 `usecases/learning`뿐이다(§4.6). 다른 묶음에 작업이 생기면 그 묶음을 [System Architecture §3.2](../02_system_architecture.md#32-의존-규칙-cargo-dependencies로-강제)에 더하고 [build_ci §2.3](../cross_cutting/03_build_ci.md#23-변경-하나가-다시-빌드시키는-범위-목표) 표를 고친다.
- 유스케이스는 포트를 `Arc<dyn Port>`로 받는다. 포트 타입의 제네릭으로 쓰지 않는다([build_ci §4](../cross_cutting/03_build_ci.md#4-제네릭과-동적-디스패치)).

## 2. 원칙

- 인터페이스는 **쓰는 쪽이 소유**한다. 유스케이스가 쓰는 출력 포트는 이 레이어의 `ports/*`에 둔다.
- 유스케이스는 **출력 DTO**를 돌려준다. 엔티티를 그대로 내보내지 않는다(S10).
- 출처 이름(Canvas, LearningX …)과 제공자 이름(Claude …)은 이 레이어에 나타나지 않는다.
- 사용자와 주고받는 흐름(OTP 입력, 쓰기 확인, AI 로그인 대기)은 유스케이스가 조율한다. 사용자에게 묻는 일은 출력 포트(§4.4)로 하고, presentation이 구현한다.
- 유스케이스는 **application 오류**(§4.5)만 돌려준다. 어댑터의 공통 오류는 출력 포트 구현이 바꿔 넘긴다.

## 3. 유스케이스 (후보)

| 묶음 | 예 | crate |
|---|---|---|
| 학교 로그인 | `LoginWithOtp`, `Logout`, `GetAuthState` | `usecases/school` |
| 조회 | 이번 주 할 일, 과목·차시·과제·공지·출결·성적 조회 | `usecases/school` |
| 개인 데이터 쓰기 | 개인 캘린더 일정, 플래너 메모·완료 표시(사용자 확인 뒤) | `usecases/school` |
| 자료 | 강의 자료·동영상 내려받기 | `usecases/school` |
| AI | 설치 감지, 로그인·로그아웃, 대화 시작·전송·취소, 권한 확인 응답, 대화 기록 관리 | `usecases/ai` |
| 대화 안전 | `Peer` 읽음 기록, hook 판정(`pass` / `warn` / `deny_once`), `Peer` 원문 격리 요약([security §4](../cross_cutting/01_security.md)) | `usecases/ai` |
| 학습(확장) | `ExtractSlidePdf`(동영상 → 슬라이드 PDF), `Transcribe`(동영상 → 스크립트), 요약·퀴즈 생성 | `usecases/learning` |
| 작업 | 작업 상태 조회, 취소 (§4.6) | `usecases/jobs` |

구체 목록과 출력 DTO는 Functional Spec 이후 확정한다.

## 4. 출력 포트

### 4.1 데이터 (Repository · Gateway)

crate: `ports/school`

| 종류 | 뜻 | 예 |
|---|---|---|
| Repository | 엔티티 모음을 조회(필요하면 저장)하는 컬렉션 | `CourseRepository`, `LectureRepository`, `AssignmentRepository`, `AttendanceRepository`, `NoticeRepository` |
| Gateway | 컬렉션이 아닌 외부 동작 | `MaterialDownloadGateway`, `MediaDownloadGateway`, `AuthGateway` |

- 유스케이스가 필요한 단위로 나눈다. 출처 단위(`CanvasRepository`)로 만들지 않는다.
- 구현과 계약은 [data_access §2](data_access/README.md).

### 4.2 AI

crate: `ports/ai`(아래 세 포트, 중립 이벤트, §4.4의 `ConversationRepository`)

쓰는 유스케이스가 다르므로 역할별로 나눈다(인터페이스 분리). 설정 화면 유스케이스는 실행 포트에 의존하지 않는다.

```
AiInstallation
  detect()            → 설치 여부, 버전, 최소 버전 충족

AiAccount
  account()           → 로그인 여부, 구독 종류 (이메일·조직 ID는 버린다)
  start_login()       → LoginChallenge { open_url? , device_code? }
  wait_login(id) / cancel_login(id)
  logout()            → { outcome, affects_terminal }

AiSession
  capabilities()      → 실행 모드, 재개, 커맨드 분류, 권한 중계 가능 여부
  start(session_spec) → 대화 핸들
  send / cancel / respond_permission
  events()            → 중립 이벤트 스트림
```

- **중립 이벤트**: `TextDelta`, `ToolUse`, `ToolResult`, `PermissionRequest`, `UsageLimit`, `Done`, `Error`. 화면과 유스케이스는 제공자 형식을 모른다.
- `session_spec`: 실행 모드(대화·격리·요약), 작업 폴더, `ToolGuide`, 모델, 재개할 세션 ID.
- 제공자 어댑터 crate 하나가 세 포트를 모두 구현하고, composition root에서 제공자 ID로 묶어 등록한다. 구현은 [ai_adapters](05_ai_adapters.md).

### 4.3 학습

crate: `ports/learning`(아래 표 전부)

| 포트 | 종류 | 하는 일 | 구현 |
|---|---|---|---|
| `FrameDecoder` | Gateway | 동영상에서 **호출자가 정한** 간격·폭의 회색조 프레임 스트림, 지정 시각의 원본 해상도 장면 | `learning/media` |
| `AudioDecoder` | Gateway | 동영상에서 **호출자가 정한** 샘플레이트·채널의 PCM | `learning/media` |
| `Transcriber` | Gateway | `required_audio_format()`으로 필요한 음성 형식을 알리고, 음성 → 시각이 붙은 문장 목록을 낸다. 진행률·`CancelToken`을 받는다 | `learning/transcribe` |
| `TextRecognizer` | Gateway | `supports(languages)`로 지원 언어를 알리고, 이미지 → 글자와 글자 상자 위치를 낸다 | `learning/macos`, `learning/windows` |
| `SlidePdfWriter` | Gateway | 확정 장면·쪽 표기·OCR 결과 → PDF 바이트(임시 파일). 경로를 정하지 않는다 | `learning/pdf` |
| `LearningOutputRepository` | Repository | 슬라이드 PDF·스크립트의 저장(원자적 쓰기)·조회·존재 확인. 위치는 domain의 이름 규칙으로 정한 키로만 받는다. 저장 직후 해시를 앱 데이터에 기록하되 기록마다 **HMAC**을 붙이고(키는 Data Protection 키체인), 조회 때 HMAC과 해시가 맞는지와 그에 따른 신뢰 등급(`Official`/`Peer`, [security §4.1](../cross_cutting/01_security.md))을 함께 돌려준다 | `local_store/learning` |
| `ComponentStore` | Gateway | 부품(ffmpeg, 전사 모델, VAD 모델)의 위치 찾기와 SHA-256 확인. 없거나 다르면 `ComponentMissing` | `local_store/learning` |

- 슬라이드 판정(구간·묶음·점수·자동 확정)은 포트 뒤에 두지 않는다. `ExtractSlidePdf`가 [domain §4.1](01_domain.md) 규칙으로 직접 판정한다. AI 검토 단계는 없다.
- 포트에는 특정 엔진의 형식(예: Whisper의 16kHz)을 적지 않는다. 디코딩 간격·폭은 domain 상수를, 음성 형식은 `Transcriber`가 알린 값을 유스케이스가 넘긴다. 엔진을 바꿔도 포트는 그대로다(OCP).
- 부품을 쓰는 어댑터(`learning/media`, `learning/transcribe`)는 `ComponentStore`를 주입받아 위치를 얻는다. 해시 확인을 어댑터마다 따로 하지 않는다.
- `TextRecognizer`의 두 구현은 지원 언어가 없을 때(예: Windows 한국어 OCR 언어 팩 없음) 똑같이 `ComponentMissing`으로 실패한다(LSP).
- 구현은 [learning_adapters](06_learning_adapters.md).

### 4.4 사용자 상호작용과 로컬 저장

| 포트 | 종류 | 하는 일 | crate | 구현 |
|---|---|---|---|---|
| `UserConfirmation` | Gateway | 확인 요청을 화면에 띄우고 승인·거부를 받는다 | `ports/common` | `presentation/tauri_commands` |
| `OtpPrompt` | Gateway | 세션이 만료됐을 때 OTP 입력을 요청한다 | `ports/common` | `presentation/tauri_commands` |
| `ConversationRepository` | Repository | 앱 대화 기록(제공자 세션 ID 색인 포함) | `ports/ai` | `local_store/conversations` |
| `SettingsRepository` | Repository | 앱 설정 | `ports/common` | `local_store/settings` |

**쓰기 확인 규칙** (개인 캘린더·플래너)

- 확인 내용은 유스케이스가 **실제로 보낼 요청에서** 만든다(AI가 쓴 설명을 쓰지 않는다).
- 확인 요청에는 요청 ID와 내용 해시를 붙이고, 승인 응답이 같은 ID·해시일 때만 그 요청을 실행한다.
- 격리 모드 대화에서 온 쓰기 요청은 확인을 묻지 않고 거부한다(지켜보는 사람이 없다).

### 4.5 application 오류

| 오류 | 뜻 |
|---|---|
| `NeedsLogin` | 학교 재로그인(OTP)이 필요하다 |
| `NotAvailable` | 권한이 없거나 그 과목에서 쓰지 않는 기능 |
| `SourceChanged` | 학교 쪽 형식이 바뀌었다(실패 단계 이름만) |
| `Temporary` | 네트워크·요청 제한. 다시 시도할 수 있다 |
| `Rejected` | 사용자가 거부했거나 정책(격리 모드 쓰기 등)으로 거부됐다 |
| `AiUnavailable` | AI 제공자 미설치·로그아웃·한도 초과 |
| `ComponentMissing` | 학습 부품(ffmpeg, 전사 모델, OCR 언어 등)이 없거나 해시·서명 확인에 실패했다. 어떤 부품인지와 **복구 방법**(`Installer`: 설치 프로그램 다시 실행, `OsSettings`: OS 설정에서 설치)을 열거형으로 함께 준다. 화면은 복구 방법만 보고 안내를 고른다 |

- data_access의 공통 오류([data_access §2.4](data_access/README.md))는 출력 포트 구현이 위 오류로 바꾼다.

### 4.6 작업 (오래 걸리는 유스케이스)

전사와 슬라이드 PDF는 수 분이 걸릴 수 있어 작업으로 실행한다(실측 시간은 [learning_adapters §3](06_learning_adapters.md)).

| 구성 | 종류 | 내용 | 위치 | 구현 |
|---|---|---|---|---|
| `JobRegistry` | application 내부 | 작업 ID 발급, 상태 전이(`queued`→`running`→`done`·`failed`·`cancelled`) 규칙, 진행률·결과 보관. **실행하지 않는다** | `usecases/jobs` | `usecases/jobs` |
| `JobSpawner` | 출력 포트 | 작업 본문(유스케이스 호출)을 백그라운드에서 실행한다 | `ports/common` | `bin/app`(tokio) |
| `CancelToken` | 값 타입 | 런타임과 무관한 취소 신호. 유스케이스가 단계 사이에서 확인하고 `Transcriber`에 넘긴다 | `kernel` | `kernel` |
| `ProgressSink` | 출력 포트 | 진행률 이벤트를 화면에 보낸다 | `ports/common` | `presentation/tauri_commands` |

- application은 비동기 런타임(tokio)에 의존하지 않는다. 실행은 `JobSpawner` 뒤에 둔다(의존성 규칙, [System Architecture §3.2](../02_system_architecture.md)).
- 취소되거나 실패하면 반쯤 만든 결과 파일은 남기지 않는다(`LearningOutputRepository`의 원자적 쓰기).
- 화면(Tauri command)과 `cdx`(작업 ID로 상태 조회)가 같은 작업을 본다.

## 5. 이웃 레이어와의 계약

| 이웃 | 계약 |
|---|---|
| presentation, entrypoints | `usecases/*`의 유스케이스를 부르고 출력 DTO·application 오류를 받는다. `presentation/tauri_commands`는 §4.4의 `UserConfirmation`·`OtpPrompt`와 §4.6의 `ProgressSink`를 구현한다 |
| data_access, ai, learning, local_store | `ports/*`의 출력 포트를 구현한다. 유스케이스 crate에는 의존하지 않는다. 공통 오류는 application 오류로 바꿔 넘긴다 |
| bin/app | `JobSpawner`를 구현하고(tokio), 모든 포트 구현을 조립한다 |

## 6. 미확인 항목

- 유스케이스·출력 DTO의 확정 목록(Functional Spec 이후)
- `ConversationRepository`의 제공자 세션 ID 표현. 수용 기준: 포트에는 불투명한 문자열 ID만 있고 제공자별 형식이 드러나지 않는다(중립 이벤트 원칙, §4.2)
- 포트의 비동기 표현(trait `async fn`을 dyn으로 쓸 수 있는지). 판단 기준은 [build_ci §4](../cross_cutting/03_build_ci.md#4-제네릭과-동적-디스패치)
