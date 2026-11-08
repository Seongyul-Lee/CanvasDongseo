# Security

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD §9](../00_prd.md), [System Architecture](../01_system_architecture.md) S8, 레이어별 문서 |

> **이 문서의 역할**: 레이어마다 흩어진 보안 규칙을 한곳에서 본다. 규칙의 상세는 강제하는 레이어 문서에 있고, 여기서는 링크한다.

---

## 1. 전제

대화 모드의 AI는 사용자 도구로 임의 명령을 실행할 수 있다([ai_adapters §3](../02_layers/06_ai_adapters.md)). 그래서 지킬 것은 모두 **코어 쪽**에서 강제한다. 제공자의 도구 제한은 보조 수단이다.

## 2. 지킬 것과 강제하는 곳

| 지킬 것 | 강제하는 곳 |
|---|---|
| 비밀(비밀번호·쿠키·토큰)은 코어 밖으로 나가지 않는다 | [data_access §1.3·§2.5](../02_layers/05_data_access/README.md) 세션 저장소·기록 규칙, [entrypoints §3](../02_layers/04_entrypoints.md) `ipc_protocol` 타입 |
| 본인 데이터만 조회한다(세션 학번만) | [07_student_portal §3.5](../02_layers/05_data_access/07_student_portal.md), [04_smart_attendance](../02_layers/05_data_access/04_smart_attendance.md), [entrypoints §4](../02_layers/04_entrypoints.md)(학번 인자 없음) |
| 학사에 영향을 주는 쓰기를 하지 않는다. 개인 데이터 쓰기는 앱 화면 확인 뒤에만 | [data_access §1.3](../02_layers/05_data_access/README.md), 포털 조회 허용 목록, [entrypoints §4](../02_layers/04_entrypoints.md) |
| 허용되지 않은 호스트·경로로 요청하지 않는다 | [data_access §2.5](../02_layers/05_data_access/README.md) HTTP 허용 목록 |
| 다른 로컬 프로세스가 학교 데이터를 꺼내지 못한다 | [entrypoints §3](../02_layers/04_entrypoints.md) IPC 토큰 + 소켓 권한 |
| OTP를 AI가 받지 않고, 우회하지 않는다 | 로그인은 앱 화면 전용, `cdx`에 로그인 명령 없음 |
| 앱은 AI 자격 증명을 다루지 않는다 | [ai_adapters §2](../02_layers/06_ai_adapters.md) 제공자 CLI 위임 |
| 학교 데이터 안의 지시를 AI가 따르지 않게 한다 | [entrypoints §5·§6](../02_layers/04_entrypoints.md) 신뢰하지 않는 데이터 표시 |
| 무인 작업은 격리 실행한다 | [ai_adapters §3](../02_layers/06_ai_adapters.md) 격리 모드 |
| WebView는 허용된 command만 부른다 | [presentation §2](../02_layers/03_presentation.md) Tauri capabilities |

## 3. 남는 위험

- 대화 모드에서 AI가 사용자 도구(WebFetch 등)로 조회 결과를 외부로 보낼 수 있다. 학생이 터미널에서 Claude Code를 쓸 때와 같은 수준이며, 학교 데이터 안의 지시(프롬프트 주입)가 계기가 될 수 있다. 신뢰하지 않는 데이터 표시와 쓰기의 앱 확인으로 줄이고, 남는 위험으로 둔다.
- 사용자가 대화 모드 권한 설정을 넓게 열어 두면(예: 모든 Bash 허용) 확인 창 없이 실행된다. 사용자 선택으로 본다.
- 학교 시스템의 IDOR 결함(학교 측 문제)은 [07_student_portal §3.4](../02_layers/05_data_access/07_student_portal.md).
- 데이터 접근 경로별 위험은 [data_access §4](../02_layers/05_data_access/README.md).
