# AI Adapters

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 레거시 `docs/14_ports_and_rpc.md` §2.1.2·§2.1.3, `docs/16_ai_runtime.md`, `src/canvas_dongseo/infrastructure/ai/claude_cli.py`, 2026-11-08 Claude Code 2.1.319 실측(§5) |

> **이 문서의 역할**: AI 포트(`AiInstallation`·`AiAccount`·`AiSession`, [application §4.2](02_application.md))의 제공자별 구현을 정한다. MVP는 Claude Code다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Interface Adapters(이벤트 변환) + Frameworks & Drivers(AI CLI 프로세스) |
| crate | `ai/claude_code` (추가 제공자는 `ai/<제공자>`) |
| 의존할 수 있는 것 | `application`, `domain` |

- 새 제공자 = crate 하나 추가 + composition root 등록. 기존 crate는 고치지 않는다.
- 제공자 형식(stream-json, app-server JSON 등)은 어댑터 안에서 중립 이벤트로 바꾼다.

## 2. Claude 로그인 (레거시 방식 유지)

| 항목 | 방식 |
|---|---|
| 설치 | 공식 설치 스크립트를 사용자 동의 뒤 실행한다. 앱이 CLI를 재배포하지 않는다 |
| 상태 | `claude auth status` → `loggedIn`, `authMethod`, `subscriptionType`만 읽는다 |
| 로그인 | `claude auth login`을 자식 프로세스로 실행. CLI가 OS 브라우저를 열고 localhost 콜백으로 끝낸다. 완료 = 종료 코드 0 + `loggedIn: true` |
| 브라우저가 안 열림 | stdout에서 수동 주소를 뽑아(이스케이프 제거, `https://claude.com/` 접두사 검사) 버튼으로 준다. **코드 붙여 넣기는 앱이 받지 않고** 터미널 실행을 안내한다 |
| 계정 폴더 | 기본 `~/.claude` 그대로. 터미널에서 로그인돼 있으면 바로 완료 |
| 로그아웃 | `claude auth logout`(`affects_terminal: true`, 동의 후). 앱에서만 끄기는 설정으로 따로 둔다 |

- stdout 형식은 공식 계약이 아니므로 CLI 버전별 픽스처로 테스트한다.

검토하고 쓰지 않는 방식:

| 방식 | 쓰지 않는 이유 |
|---|---|
| `claude setup-token` → 앱 키체인 → `CLAUDE_CODE_OAUTH_TOKEN` | 앱이 장기 AI 토큰을 보관·중계한다(S6 위반) |
| 앱 전용 `CLAUDE_CONFIG_DIR` | 로그인을 따로 해야 하고, 재개 기록이 계정 폴더에 묶여 복잡해진다. 레거시에서 만들었다 되돌림 |
| `--bare` | 구독 OAuth·키체인을 읽지 않는다(API 키 전용) |

## 3. 실행 모드

| 모드 | 쓰는 곳 | Claude 실행 옵션 |
|---|---|---|
| **대화** (기본) | 사람이 보는 채팅 | `-p --input-format stream-json --output-format stream-json --append-system-prompt <ToolGuide> --allowedTools "Bash(cdx *)"`. 사용자 설정·스킬·플러그인·MCP·커맨드를 그대로 쓴다. 나머지 도구의 권한 확인은 앱 화면으로 중계한다(`--permission-prompts host`) |
| **격리** | 사람이 지켜보지 않는 작업(예: 정기 브리핑) | 위에 더해 `--setting-sources "" --strict-mcp-config --safe-mode --tools Bash --permission-mode dontAsk --permission-prompts none` |

- 대화 모드에서 `cdx`만 미리 허용하는 것은 확인 창을 줄이는 편의다. 보안 경계가 아니다([security](../03_cross_cutting/01_security.md)).
- 격리 모드에서 `Bash(cdx *)`가 명령 연결·치환·리디렉션을 막는 것은 실측했다(§5).
- `ToolGuide` 주입: `--append-system-prompt`.

## 4. 슬래시 커맨드 전달

- `capabilities()`에 커맨드 분류(앱 처리·전달·지원 안 함)를 담는다. 분류 기준은 [presentation §3](03_presentation.md).
- 사용할 수 있는 커맨드·스킬 목록은 `system/init`의 `slash_commands`·`skills`에서 읽는다.
- 전달 대상 커맨드는 stream-json 사용자 메시지로 그대로 보낸다.

## 5. 실측 기록 (2026-11-08, Claude Code 2.1.319, macOS)

가짜 `cdx`(고정 JSON 출력)와, 실행되면 파일이 생기는 검사용 명령으로 시험했다. 끝난 뒤 `ls`로 검사용 파일이 없는 것을 확인했다.

**격리 모드 권한** (`--setting-sources "" --strict-mcp-config --safe-mode --tools Bash --allowedTools "Bash(cdx *)" --permission-mode dontAsk --permission-prompts none`)

| 명령 | 결과 |
|---|---|
| `cdx courses` | 실행됨 |
| `cdx courses; touch …`, `cdx courses && touch …`, `cdx $(touch …)`, `touch …`, `cdx courses > …` | 모두 거부(`permission_denials`에 기록), 파일 생성 없음 |

- 사용자 설정을 끈 상태에서도 구독 로그인으로 응답했다(`apiKeySource: none`).
- 사용자 설정을 켠 상태(`--safe-mode`만)에서도 같은 결과였다. 다만 `--safe-mode`는 권한 설정을 그대로 두므로, 사용자 허용 규칙이 넓으면 결과가 달라질 수 있다. 그래서 격리 모드는 `--setting-sources ""`를 쓴다.

**대화 모드와 격리 모드의 로드 차이** (`system/init`)

| 항목 | 대화(사용자 환경) | 격리 |
|---|---|---|
| 슬래시 커맨드 | 141 (`/clear`, `/compact` 포함) | 55 |
| 스킬 | 97 (`pdf`, `pptx`, `docx`, `computer-use`, `design:*` 등) | 19 (내장) |
| 플러그인 / MCP 서버 / 도구 | 16 / 38 / 387 | 5 / 0 / 1 |

- 이 수치는 개발자 기기 기준이다. 학생 기기마다 다르다.
- `-p`로 `/clear`를 보내면 오류 없이 빈 결과가 왔다. 문맥이 실제로 지워졌는지는 확인하지 않았다(앱 처리로 정함).

## 6. 추가 제공자 메모

- Codex: `codex app-server`(한 줄 JSON). 로그인은 `account/login/start`가 주는 `authUrl`을 앱이 연다. 포트 1455 충돌 때 기기 코드 방식. 지시 주입·도구 제한 방법은 착수 때 확인한다(레거시 `docs/14` §2.1.3).

## 7. 미확인 항목

- 대화 모드에서 권한 확인 요청을 stream-json으로 중계하는 메시지 형식(`--permission-prompts host`). 레거시는 CLI 제어 메시지로 처리했다(레거시 `docs/16` §3.2.2)
- `/clear`·`/compact`를 stream-json 입력으로 보냈을 때의 정확한 동작
