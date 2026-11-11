# AI Adapters

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-10-26 Claude Code 2.1.310·codex-cli 0.160.0 로그인 실측, 2026-11-08 Claude Code 2.1.319 실측(§6), 2026-11-12 Claude Code 2.1.322 도구 호출 제한 시간 실측(§6.1), 2026-11-12 플랜별 구독 종류 조사(§2.1), 2026-11-12 Claude Code 2.1.322 키체인 읽기 실패 흉내 측정(§6.2) |

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
| 설치 | 공식 설치 스크립트를 사용자 동의 뒤 실행한다(`AiInstallation.install()`). 앱이 CLI를 재배포하지 않는다. 진행률은 읽지 않고 끝난 결과(성공·실패)만 돌려준다. 시간 상한을 넘기면 프로세스를 끝내고 실패로 본다(상한 값은 §8). 끝나면 유스케이스가 다시 감지한다 |
| 업데이트 | 설치된 CLI가 최소 버전보다 낮으면 사용자 동의 뒤 `claude update`를 실행한다(`AiInstallation.update()`). 결과와 시간 상한은 설치와 같다. 2.1.322에 이 명령이 있는 것만 확인했다(§8) |
| 상태 | `claude auth status` → `loggedIn`, `authMethod`, `subscriptionType`만 읽는다. `subscriptionType`은 표시 이름에만 쓰고, 값만으로 `PlanNotSupported`를 판정하지 않는다(§2.1) |
| 로그인 | `claude auth login`을 자식 프로세스로 실행. CLI가 OS 브라우저를 열고 localhost 콜백으로 끝낸다. 완료 = 종료 코드 0 + `loggedIn: true` |
| 브라우저가 안 열림 | stdout에서 수동 주소를 뽑아(이스케이프 제거, `https://claude.com/` 접두사 검사) 버튼으로 준다. **코드 붙여 넣기는 앱이 받지 않고** 터미널 실행을 안내한다 |
| 계정 폴더 | 기본 `~/.claude` 그대로. 터미널에서 로그인돼 있으면 바로 완료 |
| 로그아웃 | `claude auth logout`(`affects_terminal: true`, 동의 후). AI를 앱에서만 끄는 설정은 MVP에 두지 않는다(2026-11-12 결정, [FS-02 §1](../specs/FS-02_ai_connection.md#1-범위)) |

- stdout 형식은 공식 계약이 아니므로 CLI 버전별 픽스처로 테스트한다.

검토하고 쓰지 않는 방식:

| 방식 | 쓰지 않는 이유 |
|---|---|
| `claude setup-token` → 앱 키체인 → `CLAUDE_CODE_OAUTH_TOKEN` | 앱이 장기 AI 토큰을 보관·중계한다(S6 위반) |
| 앱 전용 `CLAUDE_CONFIG_DIR` | 로그인을 따로 해야 하고, 대화 재개 기록이 계정 폴더에 묶여 대화마다 계정을 추적해야 한다 |
| `--bare` | 구독 OAuth·키체인을 읽지 않는다(API 키 전용) |

### 2.1 플랜별 구독 종류와 Free 계정 (2026-11-12 조사)

근거는 세 가지다. 이 기기의 `claude auth status` 출력(Claude Code 2.1.322, Max 계정 하나), 설치된 CLI 2.1.322가 계정의 조직 종류를 구독 종류로 바꾸는 표, 그리고 공식 문서와 공개 이슈다. Pro·Team·Enterprise·Free 계정으로 직접 실행해 보지는 않았다.

| 플랜 | `subscriptionType` | 근거 |
|---|---|---|
| Max | `"max"` | 이 기기에서 확인(`planTier`는 `"max_5x"`). CLI의 표 `claude_max` → `max` |
| Pro | `"pro"` | CLI의 표 `claude_pro` → `pro`. 공개 이슈의 출력 예 |
| Team | `"team"` | CLI의 표 `claude_team` → `team` |
| Enterprise | `"enterprise"` | CLI의 표 `claude_enterprise` → `enterprise` |
| Ultra | 확인하지 못함 | 공식 문서는 Ultra를 쓸 수 있는 플랜으로 적는다. CLI 2.1.322의 표에는 Ultra에 해당하는 항목이 없다 |
| 표에 없는 조직 종류, 확인 실패 | `null` | CLI는 표에 없으면 `null`을 준다 |
| Free | 로그인이 끝나지 않아 값이 없다 | 아래 |

- **Free 계정은 로그인 단계에서 막힌다.** 브라우저의 승인 화면(claude.ai/oauth)이 "Claude Max or Pro is required to connect to Claude Code"를 보이고 localhost 콜백이 오지 않는다(공개 이슈 anthropics/claude-code #50349(2026-04), #54588(2.1.120)의 보고). 그래서 `claude auth login`이 끝나지 않고, 앱은 로그인을 기다리는 상태에 머문다. 이 문구는 브라우저에만 보여서 앱이 읽을 수 없다.
- **`null`은 Free를 뜻하지 않는다.** 유료 계정인데 `subscriptionType: null`이 나온 보고가 있다(#34049: iOS App Store로 결제한 Pro, 2.1.74, 사용하면 `403 Request not allowed`. #36769: Pro, 2.1.80). Ultra의 값도 모른다. 그래서 `null`이나 모르는 값은 막지 않고 플랜 이름 없이 연결된 것으로 보인다.
- 로그인한 뒤에 구독이 끝난 계정이 무엇을 내는지(`claude auth status`의 값, 첫 메시지의 오류)는 확인하지 못했다(§8). `PlanNotSupported`는 이 경우를 위한 판정으로 남긴다.
- `claude auth status`의 다른 필드(`email`, `orgId`, `orgName`, `planTier`, `loginExpiresAt` 등)는 읽지 않는다. `planTier`는 CLI가 `subscriptionType`과 사용 한도 등급으로 만든 값이다(`pro`, `max_5x`, `max_20x`, `max_other`, `other`).
- 공식 문서: 쓸 수 있는 계정은 Pro·Max·Ultra 구독, Team·Enterprise, Console이다([인증 문서](https://code.claude.com/docs/en/authentication), [Pro·Max·Ultra](https://support.claude.com/en/articles/11145838), [Team·Enterprise](https://support.claude.com/en/articles/11845131)). 옛 Enterprise 플랜은 Claude Code가 들어 있는 좌석이어야 한다.

## 3. 실행 모드

| 모드 | 쓰는 곳 | Claude 실행 옵션 |
|---|---|---|
| **대화** (기본) | 사람이 보는 채팅 | `-p --input-format stream-json --output-format stream-json --append-system-prompt <ToolGuide> --allowedTools "Bash(cdx *)" --settings <앱 hook 설정>`. 사용자 설정·스킬·플러그인·MCP·커맨드를 그대로 쓴다. 나머지 도구의 권한 확인은 앱 화면으로 중계한다(`--permission-prompts host`) |
| **격리** | 사람이 지켜보지 않는 작업(`Peer` 원문 요약, §5) | 위에 더해 `--setting-sources "" --strict-mcp-config --safe-mode --tools Bash --permission-mode dontAsk --permission-prompts none`. 요약 세션은 `--tools ""`(도구 없음) |

- 대화 모드에서 `cdx`만 미리 허용하는 것은 확인 창을 줄이는 편의다. 보안 경계가 아니다([security](../cross_cutting/01_security.md)).
- **키체인 허용 창 줄이기**: `claude`는 로그인 키체인의 자기 자격 증명을 프로세스 시작 때만 읽는다(측정: `auth status` 4회, 첫 턴 5회, 이후 턴 0회). 토큰 갱신이나 Claude Desktop이 항목을 다시 쓰면 허용이 풀려 읽기마다 창이 뜰 수 있다. 그래서 대화마다 프로세스 하나를 오래 쓰고, CLI의 **시작 구간**(실행부터 첫 `system/init` 이벤트나 종료까지, 상태 확인·정기 갱신·격리 요약 세션 포함)만 한 줄로 세운다. 실행 전체를 한 줄로 세우지 않는다: 살아 있는 대화 프로세스가 `cdx` 응답을 기다리는 동안 코어가 격리 요약 세션(§5)을 띄워야 하므로, 실행 전체를 세우면 서로를 기다린다. 시작 구간이 시간 초과되거나, **기준 시간보다 오래 걸린 뒤** 로그아웃으로 나오면 자격 증명 읽기가 막힌 것으로 본다([application §4.2](02_application.md#42-ai) `account()`의 "자격 증명 읽기 막힘"). 이때 대기 중인 시작을 멈추고, 화면에서 "항상 허용"과 키체인 잠금 해제를 안내한다. 기준 시간 안에 로그아웃으로 나오면 로그아웃이다. `claude auth status`의 출력만으로는 허용 거부와 로그아웃을 나눌 수 없어서(§6.2) 걸린 시간으로 나눈다. 근거는 "허용 창은 사람이 눌러야 닫히고 로그아웃은 창 없이 바로 끝난다"인데, 실제 허용 창으로 잰 것이 아니라 추론이다(§6.2 한정). 안내에서 다시 시도한 결과가 또 로그아웃이면 걸린 시간과 관계없이 로그아웃으로 본다(안내의 다시 시도로 시작한 그 한 번의 실행에만 적용한다). 느린 기기에서 실제 로그아웃이 기준 시간을 넘겨도 두 번째에는 로그인으로 넘어간다. 반대로 허용 창을 기준 시간 안에 거부하면 로그아웃으로 분류되고, 이 오판은 바로잡지 못한다(알려진 한계). 기준 시간 값은 §8. 앱은 이 항목을 읽거나 고치지 않는다([키체인 측정](../measurements/2026-11_keychain_prompts.md))
- 격리 모드는 **제공자 CLI의 옵션**에 기댄다(코어가 강제하는 경계가 아니다). 격리 모드에서 쓰기를 거부하는 것은 코어가 강제한다([entrypoints §4](04_entrypoints.md)).
- `ToolGuide` 주입: `--append-system-prompt`.
- 모델 선택(`/model`)은 `session_spec`의 모델 값으로 `--model`에 넘긴다.
- **도구 호출 제한 시간**: Claude Code는 Bash 호출이 제한 시간(기본 120초)을 넘기면 명령을 끝내지 않고 백그라운드로 옮긴다(§6.1). AI는 `cdx` 결과 대신 옮겼다는 안내를 받고, 나중에 나온 출력은 그 호출의 PostToolUse hook에 오지 않는다([security §5.4](../cross_cutting/01_security.md)). 그래서 `cdx` 요청은 제한 시간 안에 끝나야 한다. 앱은 `--settings`의 `env`에 `BASH_DEFAULT_TIMEOUT_MS=300000`을 넣어 제한 시간을 300초로 늘리고(§5, 대화·격리 모드 모두), 사용자 입력을 기다리는 유스케이스는 요청을 받은 때부터 240초까지만 기다린다([FS-03 F5](../specs/FS-03_school_account.md#f5-끊긴-연결-이어-가기)). 이 값은 그 프로세스의 모든 Bash 호출에 적용되고, 프로젝트 설정에 같은 변수가 있어도 앱의 값이 쓰인다.

## 4. 슬래시 커맨드 전달

- `capabilities()`에 커맨드 분류(앱 처리·전달·지원 안 함)를 담는다. 분류 기준은 [presentation §3](03_presentation.md).
- 사용할 수 있는 커맨드·스킬 목록은 `system/init`의 `slash_commands`·`skills`에서 읽는다.
- 전달 대상 커맨드는 stream-json 사용자 메시지로 그대로 보낸다.

## 5. hook 설정과 격리 요약

**앱 hook 설정** (`--settings`로 넘김, 대화 모드)

```json
{
  "disableAllHooks": false,
  "env": { "BASH_DEFAULT_TIMEOUT_MS": "300000" },
  "hooks": {
    "PostToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "command": "cdx hook post --provider claude" }] }],
    "PreToolUse":  [{ "matcher": "<위험 도구 패턴>", "hooks": [{ "type": "command", "command": "cdx hook pre --provider claude" }] }]
  }
}
```

- `disableAllHooks: false`를 항상 넣는다. 프로젝트 설정의 `disableAllHooks: true`를 덮는다(§6).
- `env`의 `BASH_DEFAULT_TIMEOUT_MS`는 Bash 도구 호출 제한 시간을 300초로 늘린다(§3). `--settings`의 `env`는 프로세스 환경 변수와 프로젝트 설정의 `env`보다 우선한다(§6.1).
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

### 6.1 Bash 도구 호출 제한 시간 (2026-11-12, Claude Code 2.1.322, macOS)

모델은 Sonnet 5.5다. 시작·끝·받은 신호를 기록하는 가짜 `cdx wait <초>`로 시험했다. 옵션은 `--setting-sources "" --strict-mcp-config --tools Bash --allowedTools "Bash(cdx *)" --permission-mode dontAsk`이고, 조건에 적은 것만 바꿨다. 제한 시간을 넘겼을 때의 동작은 기다리는 시간을 줄이려고 제한 시간을 몇 초로 낮춰 시험했다.

| 조건 | 호출 | 결과 |
|---|---|---|
| 기본 | `cdx wait 130` | 120.3초에 도구 결과로 "Command did not complete within its 120s timeout and was moved to the background"가 왔다. `cdx`는 신호를 받지 않고 130초에 끝났다 |
| 프로세스 환경 변수 `BASH_DEFAULT_TIMEOUT_MS=300000` | `cdx wait 130` | 130초에 `cdx` 출력이 도구 결과로 왔다 |
| AI가 `timeout: 600000`을 지정 | `cdx wait 130` | 130초에 `cdx` 출력이 도구 결과로 왔다 |
| 기본 20초, AI가 `timeout: 4000`을 지정 | `cdx wait 10` | 4초에 백그라운드로 옮겨졌다. AI가 준 더 짧은 값이 쓰인다 |
| 기본 4초·`BASH_MAX_TIMEOUT_MS=8000`, AI가 `timeout: 30000`을 지정 | `cdx wait 20` | 오류 없이 8초로 깎여, 8초에 옮겨졌다 |
| `BASH_DEFAULT_TIMEOUT_MS=20000`·`BASH_MAX_TIMEOUT_MS=4000` | `cdx wait 10` | 10초에 출력이 왔다. 최댓값은 AI가 지정한 값만 깎고 기본값은 깎지 않는다 |
| `--settings`의 `env`로 4초 | `cdx wait 15` | 4초에 옮겨졌다 |
| 프로세스 환경 변수 4초 + `--settings`의 `env` 20초 | `cdx wait 10` | 10초에 출력이 왔다. `--settings`의 `env`가 우선한다 |
| 프로젝트 설정의 `env` 4초(`--setting-sources project`) | `cdx wait 10` | 4초에 옮겨졌다 |
| 위 + `--settings`의 `env` 20초 | `cdx wait 10` | 10초에 출력이 왔다. `--settings`가 프로젝트 설정보다 우선한다 |
| 기본 4초, PreToolUse hook이 `updatedInput`으로 `timeout: 20000`을 넣음 | `cdx wait 10` | 10초에 출력이 왔다. `permissionDecision`을 함께 주지 않아도 적용됐다 |
| 기본 4초 | `cdx wait 700` | 4초에 옮겨졌고, 그로부터 600초 뒤(604초)에 `cdx`가 SIGTERM을 받았다. AI는 "stopped after reaching its background time limit" 알림을 받았다 |
| 기본 4초 + `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` | `cdx wait 15` | 옮겨지지 않았다. 4초에 `cdx`가 SIGTERM을 받았고 도구 결과는 오류 "Exit code 143 / Command timed out after 4s"다. 이 설정에서는 Bash 도구에 `run_in_background` 매개변수가 없다(AI가 없다고 답하고 포그라운드로 실행했다) |
| AI가 `run_in_background: true`를 지정 | `cdx wait 3` | `Bash(cdx *)` 허용에 포함돼 거부 없이 실행됐다. 도구 결과는 바로 "Command running in background"다 |

- 도구 설명에는 "`timeout` is in milliseconds: default 120000, max 600000 for a foreground command"로 적혀 있다. `BASH_DEFAULT_TIMEOUT_MS=300000`·`BASH_MAX_TIMEOUT_MS=900000`을 주면 이 문장이 "default 300000, max 900000"으로 바뀐다. 백그라운드 실행의 한도는 "default 600000, max 7200000"이다.
- 백그라운드로 옮겨진 명령이 끝나면 stream-json 출력에 `system`/`task_notification` 이벤트(`task_id`, `tool_use_id`, `status`, `output_file`)가 나오고, AI는 종료 코드만 담긴 알림으로 새 턴을 시작한다. `-p` 실행은 그 턴이 끝난 뒤에 종료한다. 명령의 출력은 알림에 없고 출력 파일에만 있다.
- 같은 옵션에서 AI가 그 출력 파일을 `cat`으로 읽는 것은 거부 없이 실행됐다. 따로 확인하니 작업 폴더 안의 파일은 `cat`으로 읽혔고, 작업 폴더 밖의 파일(`cat`, `head`, `ls`)은 모두 거부됐다. 11-08에는 쓰기가 들어간 명령만 시험했다.
- 한정: 최댓값 600초는 도구 설명으로 확인했고 600초를 실제로 기다려 보지는 않았다. 사용자 설정(`~/.claude/settings.json`)과 관리형 설정의 `env`는 시험하지 않았다(§8). 사용자 환경을 그대로 쓰는 대화 모드와 `--input-format stream-json`으로 오래 띄운 프로세스에서는 시험하지 않았다.

### 6.2 키체인 읽기가 실패할 때의 `claude auth status` (2026-11-12, Claude Code 2.1.322, macOS)

PATH 맨 앞에 `security` shim을 두고, `find-generic-password`가 정해 둔 종료 코드와 오류 문구로 실패하게 한 뒤 `claude auth status`를 실행했다([키체인 측정 §3.3](../measurements/2026-11_keychain_prompts.md#33-m3-claude-cli의-키체인-읽기)과 같은 shim 방식). shim은 인자만 기록하고, 키체인을 바꾸는 하위 명령(`add-generic-password` 등)은 막았다. 실제 키체인 항목과 허용 상태는 바뀌지 않았다. 자격 증명이 키체인에만 있는 계정 폴더(`.credentials.json` 없음)에서 실행했다.

| 흉내 낸 경우 | `security` 종료 코드 | `loggedIn` | `authMethod` | `claude` 종료 코드 | stderr | 걸린 시간 |
|---|---|---|---|---|---|---|
| 정상(shim이 그대로 넘김) | 0 | `true` | `"claude.ai"` | 0 | 없음 | 약 0.38초 |
| 허용 창에서 거부(`errSecAuthFailed`, 추정) | 51 | `false` | `"none"` | 1 | 없음 | 약 0.09초 |
| 키체인 잠김(`errSecInteractionNotAllowed`) | 36 | `false` | `"none"` | 1 | 없음 | 약 0.09초 |
| 항목 없음 = 실제 로그아웃(`errSecItemNotFound`) | 44 | `false` | `"none"` | 1 | 없음 | 약 0.09초 |
| 창을 닫음(`userCanceledErr`) | 128 | `false` | `"none"` | 1 | 없음 | 약 0.09초 |

- 실패한 네 경우는 출력의 필드까지 같다(`loggedIn: false`, `authMethod: "none"`, 구독 종류와 `storedClaudeAiLogin` 없음). **`claude auth status`의 출력과 종료 코드로는 허용 거부·잠김과 로그아웃을 나눌 수 없다.** 표의 걸린 시간은 shim이 바로 실패하게 한 값이라 실패한 네 경우가 모두 같다. 이 표는 시간으로 나눌 수 있다는 근거가 아니다. §3이 걸린 시간으로 나누는 근거는 아래 한정 항목의 추론이다.
- 한 번 실행에 `security find-generic-password`를 2번 불렀다(서비스 이름 `Claude Code-<해시>`, `Claude Code-credentials-<해시>`). 2.1.319에서 잰 4번([키체인 측정 §3.3](../measurements/2026-11_keychain_prompts.md#33-m3-claude-cli의-키체인-읽기))과 다르다. 버전 차이인지 계정 폴더 차이인지는 확인하지 않았다.
- 한정: 실제 허용 창을 띄운 것이 아니라 `security`의 실패를 흉내 낸 것이다. 허용 창에서 "거부"를 눌렀을 때의 실제 종료 코드는 확인하지 않았다(51은 추정). 실제 허용 창이 떠 있는 동안 `security`가 응답을 기다려 걸린 시간이 늘어난다는 것도 추론이다. 허용이 풀린 상태를 재현하지 못했기 때문이다([키체인 측정 §6](../measurements/2026-11_keychain_prompts.md#6-측정하지-못한-것)).
- 계정 폴더에 `.credentials.json`이 함께 있을 때 CLI가 그 파일로 대신 읽는지는 시험하지 않았다.

## 7. 추가 제공자 메모

- Codex: `codex app-server`(한 줄 JSON). 로그인은 `account/login/start`가 주는 `authUrl`을 앱이 연다. 포트 1455 충돌 때 기기 코드 방식. 지시 주입·도구 제한 방법은 착수 때 확인한다.

## 8. 미확인 항목

- 로그인한 뒤 구독이 끝난 계정의 `claude auth status` 값과 첫 메시지의 오류 형식. `PlanNotSupported`를 무엇으로 판정할지 이것으로 정한다(§2.1)
- Ultra 플랜과 Console 계정의 `subscriptionType` 값, Claude Code가 없는 좌석(옛 Enterprise)의 계정이 막히는 지점(§2.1)
- Free 계정이 막히는 지점은 공개 이슈의 보고로 확인했다. Free 계정으로 직접 재현하지는 않았다(§2.1)
- 설치·업데이트 실행의 시간 상한 값. 공식 설치 스크립트가 설치하는 경로가 앱의 감지 경로에 바로 잡히는지([FS-02 §6](../specs/FS-02_ai_connection.md#6-미확인-항목))
- 최소 버전의 값과 그것을 정하는 기준. `claude update`가 패키지 관리자(npm·Homebrew)로 설치한 CLI에서 어떻게 동작하는지(도움말은 "패키지 관리자로 업데이트한다"고만 적는다)
- 자격 증명 읽기 막힘을 걸린 시간으로 나눌 수 있는지와 기준 시간 값, 허용 창에서 거부했을 때 `security`의 실제 종료 코드(§6.2). 허용이 풀린 상태를 재현해 잰다. 허용 창을 기준 시간 안에 거부하면 로그아웃으로 분류되므로(§3) 사람이 거부를 누르는 데 걸리는 시간도 함께 잰다
- 키체인이 잠겨 있을 때 `security`가 잠금 해제 창을 띄우고 기다리는지, 창 없이 바로 실패하는지(§6.2에서는 바로 실패하게 흉내 냈다). 바로 실패하면 §3의 판정은 잠김을 로그아웃으로 분류한다
- 사용 한도에 걸렸을 때 CLI가 풀리는 시각을 주는지와 그 형식
- 대화 모드에서 권한 확인 요청을 stream-json으로 중계하는 메시지 형식(`--permission-prompts host`, CLI 제어 메시지로 예상)
- `/clear`·`/compact`를 stream-json 입력으로 보냈을 때의 정확한 동작
- 사용자 설정(`~/.claude/settings.json`)이나 관리형 설정에 `BASH_DEFAULT_TIMEOUT_MS`가 있을 때도 앱 `--settings`의 값이 쓰이는지(프로젝트 설정보다 우선하는 것만 확인했다, §6.1)
- AI가 `cdx` 호출에 더 짧은 `timeout`을 주거나 `run_in_background`로 부르면 §3의 전제가 깨진다. PreToolUse hook이 `cdx` 호출의 두 값을 고치게 할지, `ToolGuide`의 안내로 둘지([security §7](../cross_cutting/01_security.md#7-미확인-항목))
- 제한 시간을 넘겨 백그라운드로 옮겨진 `cdx` 호출을 어댑터가 어떻게 다룰지(`task_notification` 이벤트를 화면에 알릴지, 뒤늦은 결과를 대화에 어떻게 보일지)
- 격리 모드에서도 작업 폴더 안의 파일은 읽기 전용 명령으로 읽힌다(§6.1). `session_spec`의 작업 폴더를 격리 모드에서 빈 폴더로 둘지
