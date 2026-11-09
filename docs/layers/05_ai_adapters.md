# AI Adapters

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-10-26 Claude Code 2.1.310·codex-cli 0.160.0 로그인 실측, 2026-11-08 Claude Code 2.1.319 실측(§6) |

> **이 문서의 역할**: AI 포트(`AiInstallation`·`AiAccount`·`AiSession`, [application §4.2](02_application.md))의 제공자별 구현을 정한다. MVP는 Claude Code다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Interface Adapters(이벤트 변환) + Frameworks & Drivers(AI CLI 프로세스) |
| crate | `ai/claude_code` (추가 제공자는 `ai/<제공자>`) |
| 의존할 수 있는 것 | `application/ports/ai`, `application/kernel`, `domain/core` (유스케이스 crate는 모른다) |

- 새 제공자 = crate 하나 추가 + composition root 등록. 기존 crate는 고치지 않는다.
- 제공자 형식(stream-json, app-server JSON 등)은 어댑터 안에서 중립 이벤트로 바꾼다.

## 2. Claude 로그인

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
| 앱 전용 `CLAUDE_CONFIG_DIR` | 로그인을 따로 해야 하고, 대화 재개 기록이 계정 폴더에 묶여 대화마다 계정을 추적해야 한다 |
| `--bare` | 구독 OAuth·키체인을 읽지 않는다(API 키 전용) |

## 3. 실행 모드

| 모드 | 쓰는 곳 | Claude 실행 옵션 |
|---|---|---|
| **대화** (기본) | 사람이 보는 채팅 | `-p --input-format stream-json --output-format stream-json --append-system-prompt <ToolGuide> --allowedTools "Bash(cdx *)" --settings <앱 hook 설정>`. 사용자 설정·스킬·플러그인·MCP·커맨드를 그대로 쓴다. 나머지 도구의 권한 확인은 앱 화면으로 중계한다(`--permission-prompts host`) |
| **격리** | 사람이 지켜보지 않는 작업(`Peer` 원문 요약, §5) | 위에 더해 `--setting-sources "" --strict-mcp-config --safe-mode --tools Bash --permission-mode dontAsk --permission-prompts none`. 요약 세션은 `--tools ""`(도구 없음) |

- 대화 모드에서 `cdx`만 미리 허용하는 것은 확인 창을 줄이는 편의다. 보안 경계가 아니다([security](../cross_cutting/01_security.md)).
- **키체인 허용 창 줄이기**: `claude`는 로그인 키체인의 자기 자격 증명을 프로세스 시작 때만 읽는다(측정: `auth status` 4회, 첫 턴 5회, 이후 턴 0회). 토큰 갱신이나 Claude Desktop이 항목을 다시 쓰면 허용이 풀려 읽기마다 창이 뜰 수 있다. 그래서 대화마다 프로세스 하나를 오래 쓰고, CLI의 **시작 구간**(실행부터 첫 `system/init` 이벤트나 종료까지, 상태 확인·정기 갱신·격리 요약 세션 포함)만 한 줄로 세운다. 실행 전체를 한 줄로 세우지 않는다: 살아 있는 대화 프로세스가 `cdx` 응답을 기다리는 동안 코어가 격리 요약 세션(§5)을 띄워야 하므로, 실행 전체를 세우면 서로를 기다린다. 시작 구간이 시간 초과되거나 로그아웃으로 나오면 대기 중인 시작을 멈추고, 화면에서 "항상 허용"과 키체인 잠금 해제를 안내한다. 앱은 이 항목을 읽거나 고치지 않는다([키체인 측정](../measurements/2026-11_keychain_prompts.md))
- 격리 모드는 **제공자 CLI의 옵션**에 기댄다(코어가 강제하는 경계가 아니다). 격리 모드에서 쓰기를 거부하는 것은 코어가 강제한다([entrypoints §4](04_entrypoints.md)).
- `ToolGuide` 주입: `--append-system-prompt`.
- 모델 선택(`/model`)은 `session_spec`의 모델 값으로 `--model`에 넘긴다.

## 4. 슬래시 커맨드 전달

- `capabilities()`에 커맨드 분류(앱 처리·전달·지원 안 함)를 담는다. 분류 기준은 [presentation §3](03_presentation.md).
- 사용할 수 있는 커맨드·스킬 목록은 `system/init`의 `slash_commands`·`skills`에서 읽는다.
- 전달 대상 커맨드는 stream-json 사용자 메시지로 그대로 보낸다.

## 5. hook 설정과 격리 요약

**앱 hook 설정** (`--settings`로 넘김, 대화 모드)

```json
{
  "disableAllHooks": false,
  "hooks": {
    "PostToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "command": "cdx hook post --provider claude" }] }],
    "PreToolUse":  [{ "matcher": "<위험 도구 패턴>", "hooks": [{ "type": "command", "command": "cdx hook pre --provider claude" }] }]
  }
}
```

- `disableAllHooks: false`를 항상 넣는다. 프로젝트 설정의 `disableAllHooks: true`를 덮는다(§6).
- `cdx hook`은 Claude hook JSON(stdin)을 코어의 중립 판정으로 바꾸고, 판정을 Claude 형식으로 돌려준다.
  - `warn(context)` → PostToolUse `additionalContext`
  - `deny_once(reason)` → PreToolUse `permissionDecision: "deny"` + `permissionDecisionReason`
- PreToolUse에서 `additionalContext`는 쓰지 않는다. 도구 실행 뒤에 전달되기 때문이다(§6).

**격리 요약**: `Peer` 원문을 도구 없는 격리 세션(`--tools ""`)에 넣어 요약만 받는다. 이 세션은 코어 유스케이스가 직접 띄우고, 대화 AI와 문맥을 공유하지 않는다.

## 6. 실측 기록 (2026-11-08, Claude Code 2.1.319, macOS)

가짜 `cdx`(고정 JSON 출력)와, 실행되면 파일이 생기는 검사용 명령으로 시험했다. 끝난 뒤 `ls`로 검사용 파일이 없는 것을 확인했다.

**격리 모드 권한** (`--setting-sources "" --strict-mcp-config --safe-mode --tools Bash --allowedTools "Bash(cdx *)" --permission-mode dontAsk --permission-prompts none`)

| 명령 | 결과 |
|---|---|
| `cdx courses` | 실행됨 |
| `cdx courses; touch …`, `cdx courses && touch …`, `cdx $(touch …)`, `touch …`, `cdx courses > …` | 모두 거부(`permission_denials`에 기록), 파일 생성 없음 |
| `cdx discussion 2>&1 \| head -100` (대화 모드, 별도 실행) | **확인 없이 실행됨**. 읽기 전용 명령으로의 파이프는 `Bash(cdx *)` 허용에 포함된다 |

- 결론은 시험한 형태로 한정한다: `;`, `&&`, `$(…)`, `>`는 거부, 읽기 전용 명령 파이프는 허용. 다른 형태(백틱, `|` 뒤 쓰기 명령 등)는 시험하지 않았다.

- 사용자 설정을 끈 상태에서도 구독 로그인으로 응답했다(`apiKeySource: none`).
- 사용자 설정을 켠 상태(`--safe-mode`만)에서도 같은 결과였다. 다만 `--safe-mode`는 권한 설정을 그대로 두므로, 사용자 허용 규칙이 넓으면 결과가 달라질 수 있다. 그래서 격리 모드는 `--setting-sources ""`를 쓴다.

**대화 모드와 격리 모드의 로드 차이** (`system/init`)

| 항목 | 대화(사용자 환경) | 격리 |
|---|---|---|
| 슬래시 커맨드 | 141 (`/clear`, `/compact` 포함) | 55 |
| 스킬 | 97 (`pdf`, `pptx`, `docx`, `computer-use`, `design:*` 등) | 19 (내장) |
| 플러그인 / MCP 서버 / 도구 | 16 / 38 / 387 | 5 / 0 / 1 |

- 이 수치는 개발자 기기 기준이다. 학생 기기마다 다르다.
- 격리 모드에도 내장 플러그인 5개가 로드됐다. 어떤 플러그인인지는 확인하지 않았다(도구는 Bash 1개뿐이었다).
- `-p`로 `/clear`를 보내면 오류 없이 빈 결과가 왔다. 문맥이 실제로 지워졌는지는 확인하지 않았다(앱 처리로 정함). `/compact`는 시험하지 않았다.

**hook과 컨텍스트 주입**: 결과는 [security §5.1](../cross_cutting/01_security.md)에 둔다.

## 7. 추가 제공자 메모

- Codex: `codex app-server`(한 줄 JSON). 로그인은 `account/login/start`가 주는 `authUrl`을 앱이 연다. 포트 1455 충돌 때 기기 코드 방식. 지시 주입·도구 제한 방법은 착수 때 확인한다.

## 8. 미확인 항목

- 대화 모드에서 권한 확인 요청을 stream-json으로 중계하는 메시지 형식(`--permission-prompts host`, CLI 제어 메시지로 예상)
- `/clear`·`/compact`를 stream-json 입력으로 보냈을 때의 정확한 동작
