# Security

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD §9](../01_prd.md), [System Architecture](../02_system_architecture.md) S8, PR #2 리뷰(위협 모델·문서), PR #4 위협 모델 리뷰(학습 부품·미디어), 2026-11-08~09 실측(§5) |

> **이 문서의 역할**: 앱이 책임지는 보안 범위, AI가 주입당했을 때 쓸 수 있는 통로와 대응, 수용한 위험을 정한다. 각 규칙의 구현 상세는 강제하는 레이어 문서에 있고 여기서는 링크한다.

---

## 1. 책임 범위

| # | 막는다 | 예 |
|---|---|---|
| R1 | **외부에서** 앱에 있는 사용자 정보(비밀·학교 데이터)를 탈취하는 것 | 같은 사용자 권한으로 도는 악성 프로그램이 키체인·IPC로 비밀번호·쿠키·성적을 꺼내 감 |
| R2 | 사용자가 **다른 학생의** 데이터를 읽기·쓰기·삭제하는 것 | 학번 매개변수를 바꿔 학생지원시스템에 조회 요청(IDOR) |
| R3 | **다른 사람이 쓴 글의 프롬프트 주입**으로 AI가 위험 행동을 하는 것 | 학생정보 수정·신청·퇴학 요청 쿼리 전송, 데이터를 외부 커뮤니티로 유출 |
| R4 | **악성 학습 입력**(학교 서버에서 받은 동영상·자료 파일)이 파서를 통해 로컬 파일을 읽거나 코드를 실행하는 것 | `.mp4` 이름을 단 HLS 재생목록이 홈 폴더 파일을 프레임으로 그려 OCR → PDF → AI 대화로 흘림(§5.3에서 재현). 동영상을 올리는 쪽(교수·학교, 또는 그 계정을 탈취한 사람)이 공격자다 |
| R5 | **부품·학습 결과물 변조**: 배포 경로나 같은 사용자 권한 프로세스(주입당한 AI의 명령 포함)가 실행 파일·모델·결과 파일, 그리고 **앱 데이터의 무결성 기록·작업 폴더**를 바꿔치기하는 것 | 바꾼 ffmpeg가 앱 권한으로 실행됨, 스크립트 파일에 지시문을 심어 공식 자료처럼 AI에 들어가게 함 |

**범위 밖**: 사용자 본인이 자기 정보를 자기 의도로 외부에 보내거나 앱에서 꺼내는 것. 사용자의 책임이다.

**방어 수준**: AI 제공자가 주입 프롬프트 방어를 제공하므로, R3은 제공자 방어 위에 **최소한의 장치**만 더한다. 사용자의 일반 AI 사용(외부 리서치, 스킬 등)을 제한하지 않는다(S7).

## 2. 주입당한 AI가 쓸 수 있는 통로

| 통로 | 할 수 있는 것 | 대응 | 판정 |
|---|---|---|---|
| A. `cdx` → IPC → 코어 | 본인 데이터 조회, 개인 캘린더·플래너 쓰기 | 학번 인자 없음, 포털 조회 허용 목록, 학사 쓰기 경로 없음, 쓰기는 앱 화면 확인(§3) | **R2·학사 쓰기 불가** |
| B. 학교 서버 직접 요청(Bash `curl` 등) | 세션이 있어야 가능 | 비밀은 **Data Protection 키체인**(앱 접근 그룹)에만 둔다. 다른 프로그램은 읽지 못한다(§5.2) | **막힘** |
| C. 사용자 브라우저 조작(Computer Use, 브라우저 MCP) | 사용자가 로그인한 브라우저로 학생지원시스템에 접속해 학번 변경·폼 제출 | 시점 경고(§4) | **수용한 위험**(§6) |
| D. 외부 전송(WebFetch, `curl`, 사용자 MCP) | `cdx`로 읽은 데이터를 밖으로 보냄 | 격리 요약과 시점 경고(§4) | **수용한 위험**(§6) |
| E. IPC 직접 호출 | 토큰으로 `cdx`를 거치지 않고 요청 | IPC 서버가 `cdx`와 같은 명령 허용 목록을 적용 | A와 같음 |
| F. 학습 결과물 파일 변조(Bash `echo >>` 등) | 사용자 폴더의 스크립트·슬라이드 PDF에 지시문을 심어 다음 학습 기능에서 AI에 넣음 | 결과물 등급은 앱이 남긴 해시와 맞을 때만 `Official`, 아니면 `Peer`(§4.1) | **R3 장치로 처리** |

## 3. 지킬 것과 강제하는 곳

| 지킬 것 | 범위 | 강제하는 곳 |
|---|---|---|
| 비밀(비밀번호·쿠키·토큰)은 코어 밖으로 나가지 않는다. 앱·`cdx`에 비밀을 출력하는 경로가 없다 | R1 | [data_access §2.5](../layers/data_access/README.md), [entrypoints §3](../layers/04_entrypoints.md) `ipc/wire` 타입 |
| 비밀은 Data Protection 키체인(앱 접근 그룹)에만 저장한다. 배포 빌드는 Hardened Runtime을 켜고 `get-task-allow`를 넣지 않는다 | R1, B | [data_access §2.5](../layers/data_access/README.md), [platform §3](02_platform.md) |
| IPC는 대화 단위 토큰 + 호출자 계보 확인으로 받는다 | R1, E | [entrypoints §3](../layers/04_entrypoints.md) |
| 본인 데이터만 조회한다(세션 학번만) | R2 | [07_student_portal §3.5](../layers/data_access/07_student_portal.md), [04_smart_attendance](../layers/data_access/04_smart_attendance.md), [entrypoints §4](../layers/04_entrypoints.md) |
| 학사에 영향을 주는 쓰기를 하지 않는다 | R2, R3 | [data_access §1.3](../layers/data_access/README.md), 포털 조회 허용 목록 |
| 개인 데이터 쓰기는 **코어가 만든** 확인 내용으로 앱 화면에서 확인받고, 확인한 요청만 실행한다. 격리 모드에서는 쓰기를 거부한다 | R3 | [application §4.4](../layers/02_application.md), [entrypoints §4](../layers/04_entrypoints.md) |
| 허용되지 않은 호스트·경로로 요청하지 않는다 | R1 | [data_access §2.5](../layers/data_access/README.md) HTTP 허용 목록 |
| OTP를 AI가 받지 않고 우회하지 않는다 | — | 로그인은 앱 화면 전용, `cdx`에 로그인 명령 없음 |
| 앱은 AI 자격 증명을 다루지 않는다 | R1 | [ai_adapters §2](../layers/05_ai_adapters.md) |
| 다른 사람이 쓴 글은 원문 대신 격리 요약을 AI에 준다. 읽은 직후와 위험 행동 직전에 경고한다 | R3 | §4, [entrypoints §5·§6](../layers/04_entrypoints.md) |
| 학교 HTML은 정화해서 그리고, WebView는 엄격한 CSP와 Tauri capabilities로 제한한다 | R1, R3 | [presentation §2.3](../layers/03_presentation.md) |
| 기록·크래시 보고에 비밀과 학교 데이터 본문을 남기지 않는다 | R1 | [data_access §2.5](../layers/data_access/README.md) 기록 규칙, [platform §3](02_platform.md) |
| 동영상 디코딩은 형식 고정(`-f mp4 -i fd:`), 필요한 기능만 켠 빌드(`file` 프로토콜 없음), 읽기 전용 stdin 핸들(stdio 외 상속 없음), **App Sandbox**(파일·네트워크 없음)로 한다. 실행 전 서명 요구 조건과 샌드박스 권한을 확인한다. Windows는 격리가 확인되기 전 학습 기능을 끈다 | R4 | [learning_adapters §3.1](../layers/06_learning_adapters.md), §5.3 |
| 입력 크기·길이·해상도와 후보·묶음 수, 자식 프로세스 시간에 상한을 둔다 | R4 | [learning_adapters §3.4](../layers/06_learning_adapters.md) |
| 학교 데이터(과목명·제목)로 만드는 경로는 이름 정리 규칙을 거친다 | R4 | [domain §4.2](../layers/01_domain.md) |
| 부품은 관리자만 쓰는 공용 경로에 두고, 해시(실행 파일은 코드 서명도)를 확인한 뒤에만 쓴다. 설치 프로그램의 받기 절차는 링크·임시 파일 공격을 막는다 | R5 | [learning_adapters §4](../layers/06_learning_adapters.md), [platform §3](02_platform.md) |
| 학습 결과물은 만든 직후 해시를 기록하고 기록에 HMAC(키는 Data Protection 키체인)을 붙인다. HMAC이나 해시가 맞지 않으면 `Peer` 등급으로 다룬다. 다시 만들기는 동영상에서 다시 뽑는다 | R3, R5 | §4.1, [application §4.3](../layers/02_application.md) `LearningOutputRepository` |

## 4. 프롬프트 주입 대응 (R3)

### 4.1 신뢰 등급

DS가 응답을 만들 때 **구조로** 판단한다(내용을 해석하지 않는다). 출력 DTO에 등급을 붙인다.

| 등급 | 판단 기준 | 예 |
|---|---|---|
| `Own` | 본인 데이터, 시스템이 만든 값 | 성적, 출결, 시간표, 과제 마감 |
| `Official` | 학교·교수가 쓴 글 | 학교·학과 공지, 과목 공지, 과제 설명, 강의계획서 |
| `Peer` | 다른 사람이 쓴 글. 경로 종류(토론·게시판·메시지·그룹) 또는 작성자 ID ≠ 로그인 ID | 토론 글·댓글, LearningX 게시판, 메시지 |

**학습 결과물**(슬라이드 PDF의 OCR 글자, 스크립트)은 강의 동영상에서 앱이 만든 것이라 `Official`이다. 단 사용자 폴더에 있어 앱 밖에서 바뀔 수 있으므로, `LearningOutputRepository`가 만든 직후 기록한 해시와 **읽을 때 해시가 맞을 때만** `Official`로 주고, 맞지 않거나 기록이 없으면 `Peer`로 다룬다(격리 요약·시점 경고).
- 기록 자체도 같은 사용자 권한의 앱 데이터에 있으므로, 기록마다 HMAC-SHA256을 붙인다. HMAC 키는 Data Protection 키체인(앱 접근 그룹)에만 두어 다른 프로그램(주입당한 AI의 명령 포함)이 읽지 못한다(§5.2 실측). 결과물과 기록을 함께 바꿔도 HMAC이 맞지 않아 `Peer`가 된다.
- 작업 폴더(판정 기록)도 같은 HMAC으로 확인한다. 후보 장면 이미지는 남기지 않으므로 다시 만들기의 OCR 입력은 언제나 동영상에서 새로 뽑은 장면이다.

### 4.2 장치

| 장치 | 내용 | 일반 사용에 주는 영향 |
|---|---|---|
| 격리 요약 | `Peer` 원문은 대화 AI에 주지 않는다. 도구 없는 격리 세션이 요약한 결과만 `cdx`가 돌려준다. 원문은 앱 화면에서 사용자가 직접 본다 | `Peer` 원문 인용만 제한 |
| 읽은 직후 경고 | `cdx`가 `Peer` 데이터를 돌려준 직후 PostToolUse hook이 경고를 컨텍스트로 넣는다 | 없음 |
| 위험 행동 직전 경고 | `Peer`를 읽은 대화에서 위험 도구(외부 전송, 브라우저, Computer Use, `cdx` 쓰기)를 처음 부를 때 PreToolUse hook이 **한 번 거부하며 이유로 경고**한다. 같은 호출을 다시 하면 허용한다 | 그 대화의 첫 위험 호출에 한 번 더 호출 |

- 경고는 시스템 프롬프트가 아니라 **위험 시점**에 넣는다. 대화가 길어져도 비중이 줄지 않고, AI가 판단 직전에 정책을 다시 검토한다.
- 도구를 막지 않는다. 사용자의 외부 리서치·스킬 사용은 그대로다. `Peer`를 읽지 않은 대화에는 아무것도 넣지 않는다.
- 상태(대화별 `Peer` 읽음 여부, 경고 여부)는 코어가 갖고, hook은 `cdx hook`으로 코어에 묻는다([entrypoints §7](../layers/04_entrypoints.md)).

## 5. 실측 근거 (2026-11-08)

### 5.1 hook (Claude Code 2.1.319, 사용자 환경 그대로)

| 조건 | 결과 |
|---|---|
| `--settings`로 넣은 PreToolUse hook, 기본·`bypassPermissions` 모드 | 내장 도구, 사용자 MCP 도구, Skill, ToolSearch가 모두 hook을 거쳤다 |
| 프로젝트 설정 `disableAllHooks: true` | hook이 꺼졌다 |
| 위 + 앱 `--settings`에 `disableAllHooks: false` | hook이 동작했다. 앱 설정이 프로젝트 설정보다 우선한다 |
| PostToolUse `additionalContext` | 도구 결과 바로 뒤에 AI에게 전달됐다 |
| PreToolUse `additionalContext` | **도구 실행 뒤** 결과와 함께 전달됐다. 실행 전 경고로 쓸 수 없다 |
| PreToolUse 한 번 `deny` + 이유 | 이유가 실행 전에 전달됐고, AI가 사용자 요청임을 확인한 뒤 같은 호출을 다시 해 허용됐다 |

- hook이 넣은 메시지는 stream-json 출력에 이벤트로 나오지 않는다. 경고 전달은 `cdx hook`이 코어에 남기는 기록으로 추적한다.
- 세 번 모두 AI는 노골적인 주입 지시를 따르지 않았다(표본이 작아 방어 근거로 쓰지 않는다).

### 5.2 키체인 (macOS 27.0)

| 저장 방식 | 다른 프로그램(`security find-generic-password -w`)이 읽기 |
|---|---|
| 로그인 키체인, `security`로 생성 | 허용 창 없이 읽힘 |
| 로그인 키체인, Developer ID(팀) 서명 앱이 `SecItemAdd` | **허용 창 없이 읽힘**. 기본 ACL에 `/usr/bin/security`, 파티션에 `apple-tool:`이 들어간다 |
| entitlement 없이 Data Protection 키체인 | 저장 실패(`-34018`) |
| **Data Protection 키체인 + 앱 접근 그룹**(`468HW42V9P.io.github.seongyul-lee.canvasdongseo`) | `security`: 항목을 찾지 못함. 같은 팀 서명의 다른 프로그램: 찾지 못함(`-25300`). 앱은 재실행 후에도 읽음 |

- **허용 창**(2026-11-09): Data Protection 키체인에는 ACL 허용 창이 없다. 서명·프로파일이 어긋나면 창 대신 실행 즉시 종료되거나 `-34018`이 난다. 화면 잠금 중에는 `WhenUnlocked` 항목 읽기가 `-25308`로 실패하고, `AfterFirstUnlockThisDeviceOnly`는 성공한다. 남는 허용 창은 앱이 실행하는 `claude` CLI의 로그인 키체인 항목에서 생긴다. 측정과 결정은 [키체인 측정](../measurements/2026-11_keychain_prompts.md)에 둔다.

### 5.3 ffmpeg 샌드박스 (2026-11-09, macOS 27.0, Apple M5)

ffmpeg를 App Sandbox 권한(`com.apple.security.app-sandbox`, 파일 접근 권한 없음)으로 Developer ID 서명하고, 강의 동영상을 stdin으로 넘겨 시험했다. 대조군은 같은 버전의 일반 ffmpeg다.

| 시험 | 일반 ffmpeg | 샌드박스 + Hardened Runtime + 정적 빌드 |
|---|---|---|
| 정상: stdin 동영상 → 1fps 회색조 프레임(1,661장) | 성공 11.2초 | **성공 11.2~12.0초**(차이 없음) |
| 정상: 원본 해상도 컬러 장면(rgb24), 16kHz wav 음성 | 성공 | **성공** |
| 홈 폴더 파일 직접 열기 | 열림 | **거부**(`Operation not permitted`) |
| HLS 재생목록(stdin) → 홈 폴더 파일 | **비밀 파일 3프레임을 읽어 냄** | **거부**(샌드박스로도, 빌드에 hls 없음으로도) |
| 형식 자동 판별 + 재생목록 입력 | — | 거부(빌드에 hls·concat 없음) |
| 네트워크(`https://`) | 접속됨 | **거부**(DNS 차단, 빌드에 프로토콜 없음) |
| 홈 폴더에 파일 쓰기 | 써짐 | **거부** |
| `moov`가 끝에 있는 mp4: `-i pipe:0` | 실패(0프레임) | — |
| 같은 파일: `-i fd:`(같은 stdin 핸들) | 성공 | **성공**(전체 프레임, `-ss` 장면 추출) |
| `fd,pipe`만 켠 빌드에서 경로로 열기(`file:` 포함) | — | **거부**(`Protocol not found`) |

- 위 표의 정상 경로 수치는 `-i pipe:0`(앞쪽 `moov` 영상)으로 잰 값이고, `fd:`로 바꾼 빌드에서도 1,661프레임 11.2초·음성 1,660초로 같았다.
- 서명 요구 조건(`codesign -dr -`)은 `identifier "io.github.seongyul-lee.canvasdongseo.ffmpeg" and anchor apple generic and … certificate leaf[subject.OU] = "468HW42V9P"`였다. 앱이 실행 전에 이 조건과 `app-sandbox` 권한을 확인한다.
- 명령줄 실행 파일에 권한만 넣으면 실행 즉시 `_libsecinit_appsandbox`에서 SIGTRAP으로 죽는다. **번들 식별자가 있는 최소 `.app` 번들**(`Contents/MacOS/ffmpeg` + `Info.plist`)로 감싸야 한다.
- Hardened Runtime(공증 조건)에서는 라이브러리 검증 때문에 Homebrew처럼 다른 서명의 dylib을 쓰는 빌드가 뜨지 않는다. 시스템 프레임워크에만 의존하는 **정적 빌드**는 뜬다.
- 공증(notarization) 제출은 하지 않았다(§7).

## 6. 수용한 위험

| 위험 | 이유 |
|---|---|
| 주입당한 AI가 사용자 브라우저를 조작해(C) 학생지원시스템에 IDOR 요청 | 화면 단위 조작이라 코어가 판정할 수 없다. 제공자 방어 + 격리 요약 + 시점 경고로 확률만 낮춘다 |
| 주입당한 AI가 조회 결과를 외부로 보냄(D) | 도구를 막으면 일반 사용 가치가 크게 준다. 같은 장치로 확률만 낮춘다 |
| 같은 사용자 권한의 악성 프로그램이 화면·브라우저 등 앱 밖 경로로 정보를 얻음 | OS 수준 문제로 앱이 막을 수 없다. 앱 안의 비밀과 IPC만 지킨다(R1) |
| 사용자가 대화 모드 권한을 넓게 열어 둠 | 사용자 선택 |
| Windows에서 Credential Manager의 같은 사용자 접근 | 미확인. Windows 단계에서 검토([platform §4](02_platform.md)) |
| 강의 동영상 속 슬라이드·음성에 올린 쪽(교수·학교)이 지시문을 넣음 | 학습 결과물은 `Official`이다. 학교·교수가 쓴 공지를 `Official`로 믿는 판단(§4.1)과 같다. 계정 탈취로 올린 경우는 R4의 파서 공격만 막는다 |

**출시 관문**: Windows는 ffmpeg 격리(§7 수용 기준)를 통과하기 전까지 학습 기능(슬라이드 PDF·전사)을 켜지 않는다. 데이터 조회·대화 등 나머지 기능은 영향이 없다. 디코더 메모리 버그를 통한 코드 실행(R4)은 샌드박스만 막을 수 있어서다.

- 학교 시스템의 IDOR 결함(학교 측 문제)은 [07_student_portal §3.4](../layers/data_access/07_student_portal.md), 데이터 접근 경로별 위험은 [data_access §4](../layers/data_access/README.md).

## 7. 미확인 항목

- 사용자 설정(`~/.claude/settings.json`)의 `disableAllHooks`도 앱 `--settings`가 덮는지, 하위 에이전트(Task) 안의 도구 호출에도 hook이 적용되는지
- 호출자 계보 확인(피어 PID → 부모 체인)의 macOS·Windows 구현 방법
- 위험 도구 목록을 어떤 기준으로 고르는지(도구 이름 규칙, 사용자 MCP 분류)
- ffmpeg 샌드박스 번들의 공증: 수용 기준은 공증을 통과한 `.pkg` 안의 번들이 §5.3 표를 그대로 재현하는 것
- Windows ffmpeg 격리: 수용 기준은 AppContainer(또는 낮은 무결성 수준 + Job 객체)에서 §5.3 표와 같은 결과(정상 성공, 홈 파일·네트워크·쓰기 거부)
