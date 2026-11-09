# FS-06 연결 상태와 출처 장애

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 초안 |
| 시나리오 | S2-2, S2-7 ([UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)) |
| 화면 | 좌측 사이드바의 연결 알약과 대화별 상태, 설정 > 연결(학교 계정·학생지원시스템·AI 행), 온보딩 카드, 대화 안 앱 메시지 ([UX Strategy §7.1](../06_ux_strategy.md#71-내비게이션-골격), [§7.4](../06_ux_strategy.md#74-대화별-상태), [§7.5](../06_ux_strategy.md#75-설정)) |
| 유스케이스 | 판정은 `presenters/shell`([presentation §1](../layers/03_presentation.md#1-개요)). 입력은 `GetAuthState`(FS-03), 학생지원시스템 세션 상태(FS-04, 이름 없음), AI 설치·계정 상태(FS-02, [application §4.2](../layers/02_application.md#42-ai) `AiInstallation`·`AiAccount`), 대화별 AI 진행·작업·확인 요청 |
| 근거 | [presentation §1](../layers/03_presentation.md#1-개요), [UX Strategy §7.4·§7.5·§8](../06_ux_strategy.md#8-상태-오류-긴-작업), [application §4.5](../layers/02_application.md#45-application-오류), [data_access §2.4](../layers/data_access/README.md#24-datasource-i와-계약-data_accesscontracts), [PRD §9.2](../01_prd.md#92-제약과-위험) |

> **이 문서의 역할**: 학교 계정·학생지원시스템·AI의 연결 상태를 한 곳에서 같은 체계로 판정해 모든 표시가 같은 결과를 쓰게 하고, 학교 출처 하나가 막혀도 나머지 기능이 동작하게 하는 동작과 수용 기준을 정한다.
> 각 연결의 로그인·로그아웃 동작은 [FS-02](FS-02_ai_connection.md)(AI), [FS-03](FS-03_school_account.md)(학교 계정), [FS-04](FS-04_student_portal_login.md)(학생지원시스템)가 맡는다. 상태 체계와 행의 모양은 [UX Strategy §7.5](../06_ux_strategy.md#75-설정), 대화별 상태의 우선순위는 [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)가 정한다.

---

## 1. 범위

| 하는 것 | 하지 않는 것 |
|---|---|
| 세 종류 연결의 상태를 같은 상태 체계로 판정하기 | 각 연결의 로그인·로그아웃·갱신 동작(FS-02·FS-03·FS-04) |
| 문제 상태인 연결의 개수를 연결 알약에 보이기 | 사이드바에 출처별·제공자별 상태 두기([UX Strategy §7.1](../06_ux_strategy.md#71-내비게이션-골격)) |
| 온보딩 카드·설정 행·알약·대화 안내가 같은 판정 결과를 쓰게 하기 | 화면(View)마다 상태를 따로 판정하기([presentation §2.1](../layers/03_presentation.md#21-프런트엔드-spa-내부-구조)) |
| 대화마다 상태 하나를 우선순위대로 고르기 | 동영상별 진행률, 앱이 뒤에 있을 때의 OS 알림([UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)) |
| 출처별 장애·구조 변경을 그 출처를 쓰는 기능에만 알리고, 나머지 기능은 그대로 쓰게 하기 | 출처 하나의 장애로 앱 전체나 다른 출처의 기능을 멈추기([PRD §9.2](../01_prd.md#92-제약과-위험)) |

## 2. 기능 요구사항

### F1 연결 행 판정

- **위치**: `presenters/shell`. Presenter는 유스케이스를 부르지 않는다. `tauri_commands`가 세 유스케이스를 각각 불러 받은 출력 DTO를 `shell`에 넘긴다([presentation §1](../layers/03_presentation.md#1-개요)).
- **출력**: 연결마다 연결 행 ViewModel 하나(상태 종류, headline, 점, 행동 버튼). 상태 종류와 점·버튼의 대응은 [UX Strategy §7.5](../06_ux_strategy.md#75-설정) 표를 그대로 쓴다. 문구는 Foundation이 정한다.
- **입력과 상태의 대응**:

| 연결 | 입력 | 확인 중 | 설치 안 됨 | 로그인 필요 | 곧 만료 | 사용할 수 없음 | 연결됨 |
|---|---|---|---|---|---|---|---|
| 학교 계정 | `GetAuthState`([FS-03 F7](FS-03_school_account.md)) | 확인 중(키체인 `Locked` 포함) | 해당 없음 | 로그아웃, 자동 갱신 실패 뒤 | 로그인 유지 쿠키 만료가 2일 안 | 학교 계정 출처(Canvas·LearningX·스마트출결)의 장애(F5, §6) | 연결됨. 학생 이름 |
| 학생지원시스템 | 세션 상태([FS-04 F7](FS-04_student_portal_login.md)) | 확인 중 | 해당 없음 | 로그인하지 않은 상태의 표시는 미정(§6) | 쓰지 않는다(추론, §6) | 학생지원시스템 장애(추론, §6) | 연결됨 |
| AI | 설치·계정 상태(FS-02) | 확인 중 | `NotInstalled` | `LoggedOut` | 해당 없음 | `PlanNotSupported`, `UsageLimit`(입력 경로는 §6) | 연결됨 |

- 판정은 출력 DTO의 값만 본다. 같은 입력이면 항상 같은 ViewModel이 나온다(순수 함수, 샘플 JSON으로 테스트, [presentation §2.1](../layers/03_presentation.md#21-프런트엔드-spa-내부-구조)).
- 입력 하나를 얻지 못해도(예: AI 설치 감지가 끝나지 않음) 그 행만 "확인 중"이 된다. 다른 행의 판정에는 영향이 없다(추론).

### F2 연결 알약

- 점이 있는 상태(설치 안 됨, 로그인 필요, 곧 만료, 사용할 수 없음)인 행의 개수를 보인다. 0이면 알약을 보이지 않는다([UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)).
- 개수는 F1의 결과에서만 센다. 알약이 따로 판정하지 않는다.
- 알약은 어느 연결이 문제인지 나누어 보이지 않는다. 누르면 설정 > 연결로 간다.

### F3 같은 판정 결과 쓰기

- 온보딩 카드, 설정 > 연결의 행, 연결 알약, 대화 안 안내가 **모두 F1의 같은 ViewModel**을 쓴다([UX Strategy §7.5](../06_ux_strategy.md#75-설정)). View는 받은 상태 종류에 맞는 모양만 고른다.
- 대화 안에서 연결 문제가 나면 앱 메시지에 해당 연결 행으로 바로 가는 버튼을 둔다.
- 상태가 바뀌면(로그인, 로그아웃, 만료 예고, 장애, 회복) 위 네 곳이 함께 바뀐다. 한 곳만 옛 상태로 남지 않는다.

### F4 대화별 상태

- `shell`은 대화의 AI 진행(응답 중, 사용 한도, 실패, 보지 않은 완료), 그 대화에서 시작한 작업, 그 대화에 걸린 확인 요청(쓰기 확인, 권한 확인, OTP 입력, 학생지원시스템 로그인)을 받아 **상태 하나**를 고른다([presentation §1](../layers/03_presentation.md#1-개요)). 우리 상태와 T3 Code 상태의 대응, 우선순위는 [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태) 표를 따른다.
- 겹치면 우선순위가 높은 것 하나만 보인다. 높은 상태가 끝나면(확인 응답, 창 닫기, 대기 상한) 남은 것 중 가장 높은 상태로 돌아간다.
- 확인 요청과 작업이 가진 대화 ID는 코어가 IPC 토큰에서 정한 값이다. AI가 넘긴 값을 쓰지 않는다([application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장)).
- 연결 행과 대화별 상태는 서로 다른 것을 보인다. 예: 학교 세션이 끊겨 OTP를 기다리면 그 대화는 "Input"이고, 학교 계정 행은 FS-03이 돌려준 상태를 따른다.

### F5 출처별 장애와 구조 변경 (S2-7)

- 조회 유스케이스가 출처 하나에서 오류를 받으면 **그 출처가 필요한 요청만** 실패한다. 다른 출처를 쓰는 요청과 다른 기능(대화, 학습, 설정)은 그대로 동작한다([PRD §9.2](../01_prd.md#92-제약과-위험): 경로별 저하).
- 대화 안에서는 오류별 다음 행동([UX Strategy §8](../06_ux_strategy.md#8-상태-오류-긴-작업))을 앱 메시지로 보인다.
  - `NotAvailable`: 안 되는 출처·기능과, 나머지는 쓸 수 있다는 안내
  - `SourceChanged`: 학교 사이트가 바뀌어 앱 업데이트가 필요하다는 안내. 실패 단계 이름만 쓰고, 로그인 문제로 안내하지 않는다
  - `Temporary`: 다시 시도
- AI에는 `cdx`가 같은 오류를 AI용 코드(`not_available`, `source_changed` 등)로 준다([entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령)). AI는 학교 데이터를 추측해 채우지 않는다(R2).
- 출처 장애는 로그인 상태를 바꾸지 않는다. 장애 때문에 연결 행이 "로그인 필요"나 로그아웃이 되지 않는다.
- 설정 행과 알약에는 출처 장애가 학교 계정 행의 "사용할 수 없음" 원인으로 보인다([UX Strategy §7.5](../06_ux_strategy.md#75-설정)). 다만 어느 출처가 언제부터 안 되는지 기억하고 회복을 판정하는 곳이 아키텍처에 없다(§6). 그 전까지 이 표시는 구현하지 않는다.
- 여러 출처를 합치는 조회(예: 이번 주 할 일 = Canvas + LearningX, [data_access §2.3](../layers/data_access/README.md#23-출력-포트-구현-data_accessrepositories))에서 출처 하나만 실패할 때 나머지로 부분 결과를 낼지는 아키텍처가 정하지 않았다(§6).

## 3. 예외

| 상황 | 판정 | application 오류 | 다음 행동(P5) |
|---|---|---|---|
| 한 출처가 응답하지 않음, 5xx | `Network` | `Temporary` | 다시 시도. 다른 출처는 그대로 |
| 요청 제한 | `RateLimited` | `Temporary` | 다시 시도 |
| 권한 없음, 그 과목에서 쓰지 않는 기능 | `NotAvailable` | `NotAvailable` | 안 되는 출처·기능과 나머지는 쓸 수 있다는 안내 |
| 학교 쪽 화면·형식이 바뀜 | `SourceChanged`(실패 단계 이름) | `SourceChanged` | 앱 업데이트가 필요하다는 안내 |
| 학교 세션 만료 | `SessionExpired` | `NeedsLogin` | OTP 창([FS-03 F5](FS-03_school_account.md)), 대화는 "Input" |
| 학생지원시스템 세션 만료 | `SessionExpired` | `NeedsLogin` | 학생지원시스템 로그인 창([FS-04 F4](FS-04_student_portal_login.md)), 대화는 "Input" |
| AI를 쓸 수 없음 | — | `AiUnavailable(NotInstalled)`, `AiUnavailable(LoggedOut)`, `AiUnavailable(PlanNotSupported)`, `AiUnavailable(UsageLimit)` | 사유별: 설치 / 로그인 / 유료 플랜 안내 / 한도가 풀리는 시각 |
| 상태 입력 하나를 얻지 못함 | — | — | 그 행만 "확인 중". 다른 행은 그대로(추론) |

## 4. 수용 기준

| # | 기준 | 확인 방법 | 시나리오 |
|---|---|---|---|
| AC1 | 학교 계정·학생지원시스템·AI의 출력 DTO 조합마다 `shell`이 내는 연결 행 ViewModel이 샘플 JSON과 같다. 세 행이 같은 상태 종류 집합을 쓴다 | Presenter 테스트(`design/samples/`의 상태별 샘플) | S2-2, S2-7 |
| AC2 | 알약의 개수는 점이 있는 행의 수와 같고, 0이면 알약이 보이지 않는다 | Presenter 테스트(상태 조합 전부) | S2-2 |
| AC3 | 같은 입력에서 온보딩 카드, 설정 행, 알약, 대화 안내가 같은 상태 종류를 보인다 | 샘플 모드에서 네 곳을 차례로 열어 본다 | S2-2 |
| AC4 | 로그인 유지 쿠키 만료가 2일 안이면 알약 개수가 오르고 학교 계정 행이 "곧 만료"(만료 예정 시각, 갱신 버튼)가 된다. 갱신 뒤 알약과 행에서 사라진다 | 만료 시각을 고정한 테스트 + 실제 갱신 ★([FS-03 AC10](FS-03_school_account.md)과 같은 확인) | S2-2 |
| AC5 | 한 대화에 상태가 겹치면 [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태) 우선순위의 가장 위 상태 하나만 보인다. 예: 쓰기 확인 대기 + AI 응답 중이면 "Approval", OTP 대기 + 작업 진행이면 "Input" | Presenter 테스트(우선순위 조합 전부) | — |
| AC6 | 높은 상태가 끝나면 남은 상태 중 가장 높은 것으로 돌아간다(예: OTP 입력 뒤 "Working") | Presenter 테스트 | S2-1 |
| AC7 | 대화 안의 연결 문제 앱 메시지에 연결 행으로 가는 버튼이 있고, 누르면 설정 > 연결의 해당 행이 열린다 | 화면 확인 | S2-7 |
| AC8 | 한 출처(예: LearningX)를 막으면 그 출처를 쓰는 질문에는 안 되는 출처의 안내가, 다른 출처만 쓰는 질문(예: 학교 공지)에는 답이 나온다 | 메모리 DS에서 한 출처만 실패시키는 테스트 + 실제 앱에서 그 호스트를 막고 질문 ★ | S2-7 |
| AC9 | 학생지원시스템을 막아도 Canvas·LearningX·공지 조회가 동작하고, 반대로 학교 계정 출처를 막아도 학생지원시스템 조회가 동작한다 | 메모리 DS 테스트 | S2-7 |
| AC10 | `SourceChanged`가 나면 앱 업데이트 안내가 나오고, 비밀번호·로그인 문제로 안내하지 않으며 연결 행의 로그인 상태가 바뀌지 않는다 | 응답을 고정한 테스트 | S2-7 |
| AC11 | 출처 장애(`Temporary`, `NotAvailable`)가 연결 행을 "로그인 필요"나 로그아웃으로 바꾸지 않는다 | Presenter 테스트 | S2-7 |
| AC12 | AI가 로그아웃된 경우와 사용 한도에 걸린 경우에 AI 행이 서로 다른 상태와 행동을 보인다 | Presenter 테스트 | — |
| AC13 | 사이드바에 출처별·제공자별 로그인 상태가 없고, 연결 문제는 알약 하나로만 보인다 | 화면 확인 | S2-2 |

★ 실제 학교 계정과 서버가 있어야 확인할 수 있다.

## 5. User Flow (UF-06)

이 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 그릴 갈래: 연결 상태 변화 → 행·알약·카드·대화 안내 갱신(F1~F3), 대화별 상태 전이(F4), 출처 하나가 막힌 상태에서 그 출처를 쓰는 질문과 다른 출처를 쓰는 질문(F5).

## 6. 미확인 항목

- **아키텍처에 없음 → [UX Strategy §12](../06_ux_strategy.md#12-아키텍처에-올릴-것) 후보: 출처별 장애 상태의 보관과 회복 판정.** [UX Strategy §7.5](../06_ux_strategy.md#75-설정)는 "출처별 장애(LearningX만 안 됨 등)는 … 학교 계정 행의 '사용할 수 없음' 원인으로 보인다"고 하고, [§11](../06_ux_strategy.md#11-대응표)은 출처별 장애를 "설정 > 연결 > 학교 계정, 연결 알약"에 대응시킨다. 그러나 [presentation §1](../layers/03_presentation.md#1-개요)의 `shell` 입력은 `GetAuthState`, 학생지원시스템 세션, AI 상태뿐이고, 최근 실패한 출처를 기억하는 유스케이스·포트와 언제 정상으로 되돌릴지의 규칙이 없다
- **`NotAvailable`의 뜻이 문서마다 다르다.** [application §4.5](../layers/02_application.md#45-application-오류)는 "권한이 없거나 그 과목에서 쓰지 않는 기능", [data_access §2.4](../layers/data_access/README.md#24-datasource-i와-계약-data_accesscontracts)는 예로 "Canvas 401(헤더 없음), 404 '사용 안 함'"을 든다. 학교 시스템 장애(연결 실패, 5xx)는 `Network` → `Temporary`다. 한편 [UX Strategy §11](../06_ux_strategy.md#11-대응표)은 "출처별 장애·구조 변경(`NotAvailable`, `SourceChanged`)", [§7.5](../06_ux_strategy.md#75-설정)는 "사용할 수 없음 … 학교 시스템 장애"로 쓴다. 장애가 `Temporary`로만 오면 "다시 시도"만 보여 S2-7의 "안 되는 출처와 기능이 보이고"를 채우지 못한다. `Temporary`에도 출처를 담을지, 장애를 따로 둘지 정한다
- 여러 출처를 합치는 조회에서 한 출처가 실패할 때 부분 결과를 낼지, 낸다면 빠진 출처를 어떻게 표시할지(F5). [data_access §2.3](../layers/data_access/README.md#23-출력-포트-구현-data_accessrepositories)은 합치는 방식만 정한다
- **학생지원시스템 행의 "로그인 안 됨" 표시.** 학생지원시스템은 필요할 때만 로그인하므로([UX Strategy §7.2](../06_ux_strategy.md#72-mvp-화면)) 대부분의 시간에 로그인되지 않은 상태다. 이를 [§7.5](../06_ux_strategy.md#75-설정)의 "로그인 필요"(빨강 점)로 두면 알약이 거의 늘 보인다. 점 없는 상태를 새로 둘지(§7.5 표 변경), "로그인 필요"로 두되 알약 개수에서 뺄지 정한다
- 학생지원시스템 행에서 "곧 만료"를 쓸지. 60분 세션이라 쓰지 않는 것으로 추론했다
- 학생지원시스템 장애를 어느 행에 보일지. [UX Strategy §7.5](../06_ux_strategy.md#75-설정)는 학교 계정 행만 말한다. 학생지원시스템 행으로 보이는 것은 추론이다. 로그인이 없는 출처(학교·학과 공지, Commons)의 장애는 어느 행에도 속하지 않는다
- **AI 행의 "연결됨" headline.** [UX Strategy §7.5](../06_ux_strategy.md#75-설정)는 "이름 또는 이메일로 연결됨"이라 쓰고, [application §4.2](../layers/02_application.md#42-ai)의 `AiAccount.account()`는 "이메일·조직 ID는 버린다". AI 행에 무엇을 보일지 정한다
- **AI 사용 한도를 행에 올리는 경로 → [UX Strategy §12](../06_ux_strategy.md#12-아키텍처에-올릴-것) 후보.** 한도는 대화 중 `UsageLimit` 이벤트로만 온다([application §4.2](../layers/02_application.md#42-ai)). `AiAccount.account()`에는 한도가 없어, `shell`이 AI 행을 "사용할 수 없음(사용 한도)"으로 판정할 입력이 없다
- 상태가 바뀔 때 화면에 알리는 방식(코어 이벤트를 보낼지, 화면이 다시 조회할지). [presentation §2.1](../layers/03_presentation.md#21-프런트엔드-spa-내부-구조)은 "코어 이벤트는 store에 들어간다"고만 쓴다
- 키체인 `Misconfigured`일 때 학교 계정 행의 상태([FS-03 §6](FS-03_school_account.md#6-미확인-항목)과 같은 항목)
