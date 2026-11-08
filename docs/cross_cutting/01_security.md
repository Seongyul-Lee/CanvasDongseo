# Security

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD §9](../01_prd.md), [System Architecture](../02_system_architecture.md) S8, PR #2 리뷰(위협 모델·문서), 2026-11-08 실측(§5) |

> **이 문서의 역할**: 앱이 책임지는 보안 범위, AI가 주입당했을 때 쓸 수 있는 통로와 대응, 수용한 위험을 정한다. 각 규칙의 구현 상세는 강제하는 레이어 문서에 있고 여기서는 링크한다.

---

## 1. 책임 범위

| # | 막는다 | 예 |
|---|---|---|
| R1 | **외부에서** 앱에 있는 사용자 정보(비밀·학교 데이터)를 탈취하는 것 | 같은 사용자 권한으로 도는 악성 프로그램이 키체인·IPC로 비밀번호·쿠키·성적을 꺼내 감 |
| R2 | 사용자가 **다른 학생의** 데이터를 읽기·쓰기·삭제하는 것 | 학번 매개변수를 바꿔 학생지원시스템에 조회 요청(IDOR) |
| R3 | **다른 사람이 쓴 글의 프롬프트 주입**으로 AI가 위험 행동을 하는 것 | 학생정보 수정·신청·퇴학 요청 쿼리 전송, 데이터를 외부 커뮤니티로 유출 |

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

## 3. 지킬 것과 강제하는 곳

| 지킬 것 | 범위 | 강제하는 곳 |
|---|---|---|
| 비밀(비밀번호·쿠키·토큰)은 코어 밖으로 나가지 않는다. 앱·`cdx`에 비밀을 출력하는 경로가 없다 | R1 | [data_access §2.5](../layers/data_access/README.md), [entrypoints §3](../layers/04_entrypoints.md) `ipc_protocol` 타입 |
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

## 4. 프롬프트 주입 대응 (R3)

### 4.1 신뢰 등급

DS가 응답을 만들 때 **구조로** 판단한다(내용을 해석하지 않는다). 출력 DTO에 등급을 붙인다.

| 등급 | 판단 기준 | 예 |
|---|---|---|
| `Own` | 본인 데이터, 시스템이 만든 값 | 성적, 출결, 시간표, 과제 마감 |
| `Official` | 학교·교수가 쓴 글 | 학교·학과 공지, 과목 공지, 과제 설명, 강의계획서 |
| `Peer` | 다른 사람이 쓴 글. 경로 종류(토론·게시판·메시지·그룹) 또는 작성자 ID ≠ 로그인 ID | 토론 글·댓글, LearningX 게시판, 메시지 |

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

## 6. 수용한 위험

| 위험 | 이유 |
|---|---|
| 주입당한 AI가 사용자 브라우저를 조작해(C) 학생지원시스템에 IDOR 요청 | 화면 단위 조작이라 코어가 판정할 수 없다. 제공자 방어 + 격리 요약 + 시점 경고로 확률만 낮춘다 |
| 주입당한 AI가 조회 결과를 외부로 보냄(D) | 도구를 막으면 일반 사용 가치가 크게 준다. 같은 장치로 확률만 낮춘다 |
| 같은 사용자 권한의 악성 프로그램이 화면·브라우저 등 앱 밖 경로로 정보를 얻음 | OS 수준 문제로 앱이 막을 수 없다. 앱 안의 비밀과 IPC만 지킨다(R1) |
| 사용자가 대화 모드 권한을 넓게 열어 둠 | 사용자 선택 |
| Windows에서 Credential Manager의 같은 사용자 접근 | 미확인. Windows 단계에서 검토([platform §4](02_platform.md)) |

- 학교 시스템의 IDOR 결함(학교 측 문제)은 [07_student_portal §3.4](../layers/data_access/07_student_portal.md), 데이터 접근 경로별 위험은 [data_access §4](../layers/data_access/README.md).

## 7. 미확인 항목

- 사용자 설정(`~/.claude/settings.json`)의 `disableAllHooks`도 앱 `--settings`가 덮는지, 하위 에이전트(Task) 안의 도구 호출에도 hook이 적용되는지
- 호출자 계보 확인(피어 PID → 부모 체인)의 macOS·Windows 구현 방법
- 위험 도구 목록을 어떤 기준으로 고르는지(도구 이름 규칙, 사용자 MCP 분류)
