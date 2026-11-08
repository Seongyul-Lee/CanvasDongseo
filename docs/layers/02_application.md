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
| crate | `application` |
| 의존할 수 있는 것 | `domain` |

## 2. 원칙

- 인터페이스는 **쓰는 쪽이 소유**한다. 유스케이스가 쓰는 출력 포트는 여기에 둔다.
- 유스케이스는 **출력 DTO**를 돌려준다. 엔티티를 그대로 내보내지 않는다(S10).
- 출처 이름(Canvas, LearningX …)과 제공자 이름(Claude …)은 이 레이어에 나타나지 않는다.
- 사용자와 주고받는 흐름(OTP 입력, 쓰기 확인, AI 로그인 대기)은 유스케이스가 조율한다.

## 3. 유스케이스 (후보)

| 묶음 | 예 |
|---|---|
| 학교 로그인 | `LoginWithOtp`, `Logout`, `GetAuthState` |
| 조회 | 이번 주 할 일, 과목·차시·과제·공지·출결·성적 조회 |
| 개인 데이터 쓰기 | 개인 캘린더 일정, 플래너 메모·완료 표시(사용자 확인 뒤) |
| 자료 | 강의 자료·동영상 내려받기 |
| AI | 설치 감지, 로그인·로그아웃, 대화 시작·전송·취소, 권한 확인 응답 |
| 학습(확장) | 전사, 슬라이드 추출, 요약·퀴즈 생성 |

구체 목록과 출력 DTO는 Functional Spec 이후 확정한다.

## 4. 출력 포트

### 4.1 데이터 (Repository · Gateway)

| 종류 | 뜻 | 예 |
|---|---|---|
| Repository | 엔티티 모음을 조회(필요하면 저장)하는 컬렉션 | `CourseRepository`, `LectureRepository`, `AssignmentRepository`, `AttendanceRepository`, `NoticeRepository` |
| Gateway | 컬렉션이 아닌 외부 동작 | `MaterialDownloadGateway`, `MediaDownloadGateway`, `AuthGateway` |

- 유스케이스가 필요한 단위로 나눈다. 출처 단위(`CanvasRepository`)로 만들지 않는다.
- 구현과 계약은 [data_access §2](data_access/README.md).

### 4.2 AI

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
- `session_spec`: 실행 모드(대화·격리), 작업 폴더, `ToolGuide`, 재개할 세션 ID.
- 제공자 어댑터 crate 하나가 세 포트를 모두 구현하고, composition root에서 제공자 ID로 묶어 등록한다. 구현은 [ai_adapters](05_ai_adapters.md).

### 4.3 학습

| 포트 | 하는 일 |
|---|---|
| `Transcriber` | 강의 동영상 음성을 텍스트로 |
| `TextRecognizer` | 이미지 글자 인식 |
| `SlideExtractor` | 동영상에서 슬라이드 장면 추출 |

구현은 [learning_adapters](06_learning_adapters.md).

## 5. 이웃 레이어와의 계약

| 이웃 | 계약 |
|---|---|
| presentation, entrypoints | 유스케이스를 부르고 출력 DTO·application 오류를 받는다 |
| data_access, ai, learning | 위 출력 포트를 구현한다. 공통 오류는 application 오류로 바꿔 넘긴다 |

## 6. 미확인 항목

- 유스케이스·출력 DTO의 확정 목록(Functional Spec 이후)
