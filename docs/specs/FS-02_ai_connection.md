# FS-02 AI 연결

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 초안 |
| 시나리오 | S1-1, S1-2, S1-3, S2-4, S2-5, S10-3 ([UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)) |
| 화면 | 온보딩의 AI 단계, AI 설치 동의 창, 설정 > 연결 > AI, 연결 알약, 연결 로그아웃 확인 창 ([UX Strategy §7](../06_ux_strategy.md#7-화면-구성)) |
| 유스케이스 | AI 묶음의 설치 감지, 로그인·로그아웃([application §3](../layers/02_application.md#3-유스케이스-후보), 이름 미정). 포트 `AiInstallation`, `AiAccount`([application §4.2](../layers/02_application.md#42-ai)), `UserConfirmation`([application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장)) |
| 근거 | [ai_adapters §2·§3](../layers/05_ai_adapters.md#2-claude-로그인), [키체인 측정](../measurements/2026-11_keychain_prompts.md), [security §3](../cross_cutting/01_security.md#3-지킬-것과-강제하는-곳), [System Architecture](../02_system_architecture.md#1-결정-요약) S5·S6 |

> **이 문서의 역할**: 학생이 이미 구독 중인 AI(MVP는 Claude Code)를 앱에 연결하고, 연결을 확인·해제하며, 쓸 수 없을 때 원인과 다음 행동을 보이는 기능의 동작과 수용 기준을 정한다.
> CLI 실행 옵션과 판정 방법은 [ai_adapters](../layers/05_ai_adapters.md), 연결 행의 모양과 상태 체계는 [UX Strategy §7.5](../06_ux_strategy.md#75-설정)와 [FS-06](FS-06_connection_status.md)이 맡는다. 대화 안에서의 동작은 [FS-07](FS-07_conversation.md)이다.

---

## 1. 범위

| 하는 것 | 하지 않는 것 |
|---|---|
| AI CLI 설치 여부·버전 감지 | CLI를 앱에 넣어 재배포하기([ai_adapters §2](../layers/05_ai_adapters.md#2-claude-로그인)) |
| 동의 뒤 공식 설치 스크립트로 설치하기 | AI 자격 증명을 저장·중계하기(S6, `setup-token`·`CLAUDE_CODE_OAUTH_TOKEN`) |
| CLI에 맡긴 로그인과 그 진행·취소 | 로그인 코드를 앱이 받아 붙여 넣기(터미널 안내만) |
| 플랜 판정(Claude Code를 쓸 수 없는 플랜) | 이메일·조직 ID를 읽거나 보이기 |
| 로그아웃(터미널 로그인도 풀린다는 동의 뒤) | 앱 전용 계정 폴더(`CLAUDE_CONFIG_DIR`) |
| 사용 한도 표시 | 사용량 페이지([UX Strategy §3](../06_ux_strategy.md#3-기준-디자인-t3-code) 가져오지 않는 것) |
| 키체인 허용 창이 뜰 때의 안내 | `Claude Code-credentials` 키체인 항목을 읽거나 고치기([키체인 측정 §5](../measurements/2026-11_keychain_prompts.md#5-앱이-하지-않는-것)) |

## 2. 기능 요구사항

### F1 설치 감지

- `AiInstallation.detect()`로 설치 여부, 버전, 최소 버전 충족을 얻는다.
- 감지하는 때: 앱 시작, 온보딩 AI 단계에 들어갈 때, 설치가 끝난 직후, 설정 > 연결 > AI를 열 때.
- 설치돼 있지 않으면 `AiUnavailable(NotInstalled)`이다. 연결 행은 "설치 안 됨"(설치 버튼)이다(FS-06).
- 최소 버전을 만족하지 않을 때의 사유와 행동은 정하지 않았다(§6).

### F2 설치 (S1-1)

- 설치 버튼 → **AI 설치 동의 창**(C1, C4). 창에는 설치하기 **전에** 다음을 보인다(P4).
  - Claude 유료 플랜이 있어야 쓸 수 있다는 조건.
  - 앱이 Claude의 공식 설치 스크립트를 실행한다는 것.
- 승인하면 공식 설치 스크립트를 실행한다. 거부하면 `Rejected(UserDeclined)`이고 아무것도 실행하지 않는다.
- 설치 중에는 카드·행이 진행 중으로 보이고 다시 누를 수 없다. 끝나면 F1로 다시 감지한다. 감지되면 F3으로 넘어간다.
- 학생은 터미널을 열지 않는다(S1-1 점검 방법).

### F3 로그인 (S1-1, S1-3)

- 먼저 `AiAccount.account()`로 상태를 읽는다. 이미 로그인돼 있으면(터미널 로그인 포함) 로그인 단계를 보이지 않고 F4로 간다(S1-3).
- 로그인 버튼 → `start_login()` → CLI가 OS 브라우저를 열고 localhost 콜백으로 끝낸다. 앱은 `wait_login`으로 끝나기를 기다린다. 완료는 [ai_adapters §2](../layers/05_ai_adapters.md#2-claude-로그인)의 완료 조건으로 판정한다.
- 기다리는 동안 버튼은 "취소"가 되고, 누르면 `cancel_login`으로 멈춘다. 실패하거나 취소하면 "다시 시도"가 된다([UX Strategy §7.5](../06_ux_strategy.md#75-설정)).
- 브라우저가 열리지 않으면 `LoginChallenge.open_url`을 "브라우저에서 열기" 버튼으로 보인다. 주소는 어댑터가 접두사를 검사한 값만 쓴다. 로그인 코드 붙여 넣기 칸은 두지 않고, 코드가 필요한 경우 터미널에서 `claude auth login`을 실행하라고 안내한다.
- 앱은 AI 자격 증명을 받거나 저장하지 않는다. 로그인 결과로 남는 것은 CLI의 계정 폴더(`~/.claude`)뿐이다.

### F4 플랜 판정 (S1-2)

- `account()`의 구독 종류가 Claude Code를 쓸 수 없는 플랜이면 `AiUnavailable(PlanNotSupported)`이다. 판정 기준은 [ai_adapters §2](../layers/05_ai_adapters.md#2-claude-로그인)가 정한다.
- 이 상태에서는 대화를 시작하지 않는다. 연결 행은 "사용할 수 없음 / 플랜 미지원"이고, 행동은 유료 플랜 안내 하나다(P5).
- 안내는 로그아웃(`LoggedOut`)·사용 한도(`UsageLimit`) 안내와 문구와 행동이 다르다(S1-2 점검 방법).
- 쓸 수 있는 플랜이면 연결 행은 "연결됨"이고 headline에 구독 종류를 보인다(예: "Claude Pro로 연결됨"). 이메일·조직 ID는 읽지도 보이지도 않는다([application §4.2](../layers/02_application.md#42-ai)).
- 판정하는 때: 로그인이 끝난 직후, 앱 시작, 대화를 시작하기 전. 판정이 로그인 단계에서 가능한지, 첫 메시지 오류로만 알 수 있는지는 미확인이다(§6).

### F5 로그아웃 (S10-3)

- 설정 > 연결 > AI의 로그아웃 버튼 → **연결 로그아웃 확인 창**. 창에는 "터미널의 Claude Code 로그인도 함께 풀린다"는 것을 보인다([UX Strategy §7.2](../06_ux_strategy.md#72-mvp-화면)).
- 승인하면 `AiAccount.logout()`을 실행한다. 결과의 `affects_terminal`이 참이면 확인 창의 안내와 같다. 거부하면 `Rejected(UserDeclined)`이고 아무것도 하지 않는다.
- 로그아웃 뒤 연결 행은 "로그인 필요"가 되고, 새 대화나 메시지 전송은 `AiUnavailable(LoggedOut)`으로 막혀 로그인 안내가 나온다(FS-07).
- 학교 계정, 학과 설정, 대화 기록에는 영향이 없다.

### F6 쓸 수 없을 때 원인 보이기 (S2-4)

- 대화 중이나 상태 조회에서 AI를 쓸 수 없으면 `AiUnavailable`의 사유로 안내를 고른다([UX Strategy §8](../06_ux_strategy.md#8-상태-오류-긴-작업)).

| 사유 | 연결 행 | 대화 안 앱 메시지의 행동 |
|---|---|---|
| `NotInstalled` | 설치 안 됨 | 설치(연결 행으로) |
| `LoggedOut` | 로그인 필요 | 로그인(연결 행으로) |
| `PlanNotSupported` | 사용할 수 없음 / 플랜 미지원 | 유료 플랜 안내 |
| `UsageLimit` | 사용할 수 없음 / 사용 한도(연결 행에 올리는 입력 경로는 미결, [FS-06](FS-06_connection_status.md) §6) | 풀리는 시각. CLI가 시각을 주지 않으면 시각 없이 한도 안내만 |

- 사용 한도는 중립 이벤트 `UsageLimit`으로 알게 된다. 그 대화의 상태는 "Limited"가 된다(판정은 FS-06, [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)).
- 한도가 풀리는 시각이 지나면 다음 전송을 막지 않는다. 앱이 다시 보내지는 않는다(추론, §6).

### F7 키체인 허용 창 안내 (S2-5)

- `claude`는 프로세스 시작 때 로그인 키체인의 자기 자격 증명을 읽고, 허용이 풀려 있으면 읽기마다 macOS 허용 창이 뜬다([키체인 측정 §3.3](../measurements/2026-11_keychain_prompts.md#33-m3-claude-cli의-키체인-읽기)).
- 앱은 CLI의 **시작 구간**(상태 확인·로그인 확인·대화 시작·격리 요약 세션 포함)을 한 번에 하나씩만 실행한다. 허용 창이 겹쳐 쌓이지 않게 하기 위해서다([ai_adapters §3](../layers/05_ai_adapters.md#3-실행-모드)).
- 시작 구간이 시간 초과되거나 로그아웃으로 나오면, 대기 중인 다른 시작을 멈추고 앱 안내를 보인다.
  - 허용 창에서 "항상 허용"을 고르라는 것.
  - 키체인이 잠겨 있으면 잠금을 해제하라는 것.
- 안내를 본 뒤 학생이 다시 시도하면 멈춘 시작을 다시 한다. 앱은 키체인 항목을 읽거나, 지우거나, 권한을 바꾸지 않는다.
- 안내는 온보딩 AI 카드, 설정 > 연결 > AI, 대화 안 앱 메시지 중 지금 학생이 보는 곳에 나온다(같은 판정 결과, FS-06).

## 3. 예외

| 상황 | 판정 | application 오류 | 다음 행동(P5) |
|---|---|---|---|
| CLI가 없음 | `detect()` 설치 안 됨 | `AiUnavailable(NotInstalled)` | 설치(동의 창) |
| 설치 동의 창에서 거부 | — | `Rejected(UserDeclined)` | 설치 버튼이 남는다 |
| 설치 스크립트 실패(네트워크 등) | — | 정하지 않음(§6) | 다시 시도 |
| 브라우저가 열리지 않음 | `open_url` 있음 | — | "브라우저에서 열기" 버튼 |
| 로그인 코드 입력이 필요함 | — | — | 터미널에서 로그인하라는 안내 |
| 로그인 취소·실패 | — | — | 다시 시도 |
| 로그아웃 상태 | `loggedIn: false` | `AiUnavailable(LoggedOut)` | 로그인 |
| Free 등 미지원 플랜 | 구독 종류 | `AiUnavailable(PlanNotSupported)` | 유료 플랜 안내 |
| 사용 한도 | `UsageLimit` 이벤트 | `AiUnavailable(UsageLimit)` | 풀리는 시각(알 때만) |
| 로그아웃 확인 창에서 거부 | — | `Rejected(UserDeclined)` | 연결 유지 |
| 시작 구간 시간 초과 | 키체인 허용 대기로 봄 | — | "항상 허용"·잠금 해제 안내, 다시 시도 |

- 화면은 `AiUnavailable`·`Rejected`의 사유([application §4.5](../layers/02_application.md#45-application-오류))만 보고 안내를 고른다.

## 4. 수용 기준

| # | 기준 | 확인 방법 | 시나리오 |
|---|---|---|---|
| AC1 | `claude`가 없으면 AI 카드·행이 "설치 안 됨"이고, 설치 버튼을 누르면 설치 전에 "Claude 유료 플랜이 필요하다"는 조건이 보이는 동의 창이 뜬다 | `claude`가 없는 계정에서 온보딩을 연다 | S1-1 |
| AC2 | 동의 창에서 거부하면 설치 스크립트가 실행되지 않는다 | 거부 뒤 `claude` 존재와 프로세스 기록 확인. 테스트(`AiInstallation` 메모리 구현) | S1-1 |
| AC3 | 동의 뒤 설치가 끝나면 다시 감지해 로그인 단계로 넘어가고, 그동안 터미널을 열 필요가 없다 | `claude`가 없는 유료 계정으로 따라간다 | S1-1 ★ |
| AC4 | 로그인 버튼을 누르면 OS 브라우저가 열리고, 브라우저에서 로그인을 마치면 앱의 AI 상태가 연결됨이 된다. 기다리는 동안 버튼은 "취소"다 | 로그아웃 상태에서 로그인한다 | S1-1 ★ |
| AC5 | 브라우저가 열리지 않으면 "브라우저에서 열기" 버튼이 보이고, 앱 안에 로그인 코드 입력 칸이 없다 | CLI 출력 픽스처로 Presenter 테스트 | — |
| AC6 | 터미널에서 이미 로그인된 기기에서는 로그인 화면 없이 AI 단계가 끝난다 | 로그인된 기기에서 첫 실행 | S1-3 |
| AC7 | Free 플랜으로 판정되면 유료 플랜 안내가 보이고 대화가 시작되지 않으며, 이 안내는 로그아웃·사용 한도 안내와 다르다 | `account()` 결과를 고정한 Presenter 테스트(세 사유 비교) + Free 계정 실측(§6) | S1-2 ★ |
| AC8 | AI 로그아웃과 사용 한도를 각각 재현하면 서로 다른 안내가 나온다. 로그아웃은 로그인 버튼, 한도는 풀리는 시각(CLI가 줄 때만)이다 | 이벤트를 고정한 테스트 + 실제 로그아웃 | S2-4 |
| AC9 | 로그아웃 확인 창에 "터미널 로그인도 풀린다"는 안내가 있고, 승인하면 터미널에서 `claude auth status`가 로그아웃으로 나온다 | 설정에서 로그아웃 뒤 터미널에서 확인 | S10-3 ★ |
| AC10 | 로그아웃 뒤 메시지를 보내면 로그인 안내가 나오고, 학교 계정 연결과 대화 기록은 그대로다 | 로그아웃 뒤 대화와 설정 확인 | S10-3 ★ |
| AC11 | `claude` 시작 구간이 동시에 둘 이상 돌지 않는다(상태 확인, 대화 시작, 격리 요약 세션 포함) | 가짜 `claude`(시작 시각 기록)로 여러 대화를 동시에 시작하는 테스트 | S2-5 |
| AC12 | 시작 구간이 시간 초과되면 "항상 허용"과 키체인 잠금 해제 안내가 보이고, "항상 허용"을 고른 뒤 다시 실행하면 허용 창이 뜨지 않는다 | 응답하지 않는 가짜 `claude`로 안내 표시 테스트 + 허용이 풀린 상태 실측(§6) | S2-5 ★ |
| AC13 | 앱 데이터 파일, 로그, 앱 키체인 항목 어디에도 AI 자격 증명·이메일·조직 ID가 없다 | 로그인·로그아웃 뒤 해당 위치에서 값 검색 | — |
| AC14 | AI가 연결되면 AI 연결 행에 구독 종류가 "Claude Pro로 연결됨"처럼 보이고 이메일은 보이지 않는다. 모르는 `subscriptionType` 값이면 "Claude로 연결됨"이다(추론, §6) | `account()` 결과를 고정한 Presenter 테스트 + 실제 계정 | S1-1 ★ |

★ 유료 Claude 계정이 있어야 확인할 수 있다. AC7은 Free 계정, AC12는 허용이 풀린 실제 키체인 상태가 필요하다.

## 5. User Flow (UF-02)

이 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 그릴 갈래: 설치 감지 → 동의 → 설치(F1·F2, 거부·실패 포함), 로그인(F3, 이미 로그인됨·브라우저 안 열림·취소 포함), 플랜 판정(F4), 로그아웃(F5), 쓸 수 없을 때 사유별 안내(F6), 키체인 허용 창 안내(F7).

## 6. 미확인 항목

- **구독 종류의 표시 이름**: 연결됨 행은 `subscriptionType`으로 "Claude Pro로 연결됨"처럼 보인다([UX Strategy §7.5](../06_ux_strategy.md#75-설정)). `subscriptionType`이 플랜마다 어떤 값을 주는지(Pro, Max, Team 등)는 Free 값과 함께 실측하고([ai_adapters §8](../layers/05_ai_adapters.md#8-미확인-항목)), 모르는 값이 오면 플랜 이름 없이 "Claude로 연결됨"으로 보인다(추론)
- **사용 한도를 연결 행에 올리는 입력**: F6 표는 `UsageLimit`이면 AI 연결 행을 "사용할 수 없음 / 사용 한도"로 둔다. 그런데 한도는 대화 중 중립 이벤트 `UsageLimit`으로만 오고 `AiAccount.account()`에는 없어서, `presenters/shell`이 연결 행을 그렇게 판정할 입력이 없다([FS-06](FS-06_connection_status.md) §6과 같은 항목). 정해지기 전에는 대화별 상태 "Limited"만 확실하다
- **설치를 실행하는 포트 동작이 아키텍처에 없음 → UX Strategy §12 후보.** [ai_adapters §2](../layers/05_ai_adapters.md#2-claude-로그인)는 "공식 설치 스크립트를 사용자 동의 뒤 실행한다"고 하지만, [application §4.2](../layers/02_application.md#42-ai)의 `AiInstallation`에는 `detect()`만 있다. `install()`(진행·실패 결과 포함)을 더할지 정한다
- **설치 실패에 대응하는 application 오류가 없다.** `Temporary`로 볼지, 사유를 더할지 정한다. 위 항목과 함께 §12 후보
- **최소 버전 미충족일 때의 사유와 행동이 없다.** `detect()`는 "최소 버전 충족"을 돌려주지만 `AiUnavailable`의 사유 열거형에는 버전 항목이 없고, [UX Strategy §7.5](../06_ux_strategy.md#75-설정) 연결 행 상태에도 "업데이트"가 없다. `NotInstalled`로 다루고 설치 스크립트를 다시 실행할지(추론), 사유를 더할지 정한다. 더하면 §12 후보
- **"앱에서만 끄기" 설정이 UX Strategy에 없다.** [ai_adapters §2](../layers/05_ai_adapters.md#2-claude-로그인)는 로그아웃 행에 "앱에서만 끄기는 설정으로 따로 둔다"고 하지만, [UX Strategy §7.5](../06_ux_strategy.md#75-설정) 설정 항목과 [application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장) `SettingsRepository`에는 없다. 이 문서는 넣지 않았다. 둘 중 어느 쪽을 고칠지 정한다
- **키체인 거부와 실제 로그아웃의 구별.** [ai_adapters §3](../layers/05_ai_adapters.md#3-실행-모드)은 시작 구간이 "로그아웃으로 나오면" 키체인 안내를 보이라 하고, S2-4는 로그아웃이면 로그인 버튼을 보이라 한다. 허용 창을 거부했을 때 `claude auth status`가 무엇을 내는지 실측해, 두 경우를 구별할 수 있으면 나누고 없으면 한 안내에 둘을 함께 둘지 정한다(P5는 행동 하나)
- 시작 구간의 시간 초과 값. 학생이 허용 창을 읽고 누르는 시간보다 길어야 한다
- 허용이 풀린 상태에서 실제로 뜨는 창의 수와 재현 방법([키체인 측정 §6](../measurements/2026-11_keychain_prompts.md#6-측정하지-못한-것)). AC12의 실측은 이것을 재현한 뒤 한다
- Free 계정의 `subscriptionType` 값과 막히는 지점(로그인 단계인지 첫 메시지 오류인지, [ai_adapters §8](../layers/05_ai_adapters.md#8-미확인-항목)). 모르는 구독 종류 값이 오면 막지 않고 첫 메시지 결과로 판정할지(추론) 정한다
- 사용 한도 때 CLI가 풀리는 시각을 주는지와 그 형식([ai_adapters §8](../layers/05_ai_adapters.md#8-미확인-항목)). 한도가 풀린 뒤 상태를 언제 되돌릴지(F6, 추론)
- 로그인 대기(`wait_login`)의 상한과, 앱이 꺼질 때 로그인 프로세스를 정리하는 방법
- 공식 설치 스크립트가 설치하는 경로가 앱의 감지 경로(PATH)에 바로 잡히는지. 앱은 셸 설정을 읽지 않을 수 있다(추론)
- 로그아웃할 때 이미 떠 있는 대화 프로세스를 끝낼지. `claude`는 시작 때만 자격 증명을 읽으므로 떠 있는 프로세스가 계속 응답할 수 있다(추론, [키체인 측정 §3.3](../measurements/2026-11_keychain_prompts.md#33-m3-claude-cli의-키체인-읽기))
