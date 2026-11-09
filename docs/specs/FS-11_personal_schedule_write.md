# FS-11 개인 일정 쓰기

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 초안 |
| 시나리오 | S8-1 ([UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)) |
| 화면 | 쓰기 확인 창(`UserConfirmation`), 대화 안 앱 메시지, 사이드바 대화별 상태 "Approval" ([UX Strategy §7.2](../06_ux_strategy.md#72-mvp-화면), [§7.4](../06_ux_strategy.md#74-대화별-상태)) |
| 유스케이스 | 개인 데이터 쓰기 묶음 ([application §3](../layers/02_application.md#3-유스케이스-후보)), `UserConfirmation` ([application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장)) |
| 근거 | [PRD §9.1](../01_prd.md#91-비목표-non-goals), [data_access §1.3](../layers/data_access/README.md#13-지키는-원칙), [02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로), [security §3](../cross_cutting/01_security.md#3-지킬-것과-강제하는-곳), [entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령), [presentation §2.3](../layers/03_presentation.md#23-신뢰하지-않는-html과-webview-보안) |

> **이 문서의 역할**: 학생이 대화로 요청한 개인 캘린더 일정·플래너 메모·완료 표시를, 앱이 만든 확인 창에서 승인받은 뒤에만 Canvas에 쓰는 기능의 동작과 수용 기준을 정한다.
> 쓰기 확인 규칙의 원본은 [application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장), 쓰기 경로는 [02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로)가 맡는다. 일정·플래너를 학생에게 보이는 조회는 이 문서와 FS-08 어디에도 없다(§6).

---

## 1. 범위

| 하는 것 | 하지 않는 것 |
|---|---|
| 개인 캘린더 일정 만들기·바꾸기·지우기(`context_code=user_{id}`) | 과목 일정 만들기·바꾸기([02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로) "과목 일정은 만들지 않는다") |
| 플래너 메모 만들기·바꾸기·지우기 | 과제·퀴즈 제출, 출결·진도 조작, 게시판·토론 글쓰기, 메시지 보내기([data_access §1.3](../layers/data_access/README.md#13-지키는-원칙)) |
| 플래너 항목 완료 표시와 해제(`planner/overrides`) | 확인 없이 쓰기, AI가 쓴 설명을 확인 창에 그리기 |
| 쓰기마다 앱 화면에서 확인받기 | 격리 모드 대화의 쓰기 |
| 쓰기 결과를 대화와 AI에 돌려주기 | 외부 캘린더(Google·Apple 등)에 쓰기. 아키텍처에 없다 |

## 2. 기능 요구사항

### F1 쓰기 요청 받기

- **입력**: AI가 `cdx`의 쓰기 명령으로 보낸 값. 종류(일정·메모·완료 표시), 동작(만들기·바꾸기·지우기), 그리고 종류별 값(제목, 시작·끝 시각, 내용, 연결 과목, 대상 항목 ID).
- 학번·사용자 ID 인자는 없다. `user_{id}`의 ID는 코어가 로그인 세션(`GET /users/self`)에서 정한다([entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령), [PRD §9.2](../01_prd.md#92-제약과-위험)).
- 시각은 시간대가 있는 값으로 받는다. 끝이 시작보다 앞이거나 필수 값이 빠지면 확인 창을 띄우지 않고 거부한다. 이 검사는 `cdx` 인자 검사와 유스케이스 입력 검사에서 하며 application 오류가 아니다(추론, §6).
- 바꾸기·지우기의 대상은 본인 개인 일정(`user_{id}` 문맥)과 본인 플래너 메모·항목뿐이다. 유스케이스가 대상 항목을 먼저 읽어 확인하고, 아니면 `Rejected(PolicyDenied)`로 끝낸다(추론, §6).

### F2 격리 모드 거부

- 요청을 일으킨 대화가 격리 모드면 확인 창을 띄우지 않고 바로 `Rejected(PolicyDenied)`로 끝낸다. 지켜보는 사람이 없기 때문이다([application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장)).
- 대화 모드는 코어가 IPC 토큰(대화 단위)으로 안다. AI가 넘긴 값을 쓰지 않는다([entrypoints §3](../layers/04_entrypoints.md#3-ipc)).
- 이 거부는 제공자 CLI 옵션이 아니라 코어가 강제한다([ai_adapters §3](../layers/05_ai_adapters.md#3-실행-모드)).

### F3 확인 내용 만들기

- 유스케이스가 **실제로 보낼 HTTP 요청**(메서드, 대상, 본문 필드)을 먼저 만들고, 확인 내용은 그 요청에서 만든다. AI가 쓴 설명·요약은 쓰지 않는다([application §4.4](../layers/02_application.md#44-사용자-상호작용과-로컬-저장)).
- 확인 내용에는 종류, 동작, 그리고 요청에 실리는 값(제목, 시각, 내용, 연결 과목)을 모두 보인다. 바꾸기는 지금 값과 바뀔 값을, 지우기는 지울 항목의 지금 값을 보인다(추론, §6).
- 확인 요청에는 요청 ID, 내용 해시, 요청을 일으킨 대화 ID를 붙인다. 내용 해시는 실제로 보낼 요청 전체에서 만든다(추론, §6).

### F4 확인 창

- `UserConfirmation`이 확인 창을 띄운다. 창은 코어가 만든 값만 그린다. 앱의 말로 보이게 하고, AI 메시지와 겉모습부터 구별한다(P2, [presentation §2.3](../layers/03_presentation.md#23-신뢰하지-않는-html과-webview-보안)).
- 창이 떠 있는 동안 그 대화의 상태는 "Approval"이다([UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)).
- 버튼은 승인과 거부 둘이다. 창을 닫는 것은 거부로 다룬다.
- 승인 command는 요청 ID와 내용 해시를 함께 보낸다([presentation §2.3](../layers/03_presentation.md#23-신뢰하지-않는-html과-webview-보안)).
- 요청마다 창이 따로 뜬다. 승인 하나로 여러 요청을 실행하지 않는다(추론, §6).

### F5 승인 뒤 실행

- 승인 응답의 요청 ID·내용 해시가 확인 요청과 같을 때만 그 요청을 보낸다. 다르면 보내지 않고 `Rejected(PolicyDenied)`로 끝낸다(추론: 사유 열거형 중 가장 가까운 값, §6).
- 한 요청 ID는 한 번만 실행한다. 같은 ID의 승인이 다시 오면 무시한다(추론, §6).
- 요청은 Canvas 쿠키 세션과 `X-CSRF-Token`으로 보낸다([02_canvas §1](../layers/data_access/02_canvas.md#1-공통)).
- 학교 세션이 만료돼 있으면 FS-03 F5처럼 `OtpPrompt`로 OTP를 받은 뒤 **승인된 같은 요청**을 다시 보낸다. 내용이 같으므로 확인을 다시 묻지 않는다(추론, §6).

### F6 결과 알리기

- 성공하면 대화 안에 반영 완료 앱 메시지를 남기고(S8-1 "반영 완료가 보인다"), AI에는 만든·바꾼 항목의 ID와 값을 돌려준다. 위치는 [UX Strategy §8](../06_ux_strategy.md#8-상태-오류-긴-작업) "대화 중에 생긴 상태"를 따른다(추론, §6).
- 거부되면 AI에 거부 사유만 돌려준다. AI는 거부를 받은 뒤 같은 쓰기를 스스로 다시 요청하지 않는다는 문장을 `ToolGuide`에 둔다(추론, §6).

### F7 `Peer`를 읽은 대화

- `cdx` 쓰기는 위험 도구다. `Peer`를 읽은 대화에서 처음 부르면 hook이 한 번 거부하며 경고한다([security §4.2](../cross_cutting/01_security.md#42-장치)). 판정은 FS-09가 정한다([README §3](README.md#3-목록)). 그 뒤의 쓰기도 F2~F5를 그대로 거친다.

## 3. 예외

| 상황 | 판정 | application 오류 | 다음 행동(P5) |
|---|---|---|---|
| 확인 창에서 거부하거나 창을 닫음 | — | `Rejected(UserDeclined)` | 없음. AI는 거부됐다고 답한다 |
| 격리 모드 대화에서 온 쓰기 | — | `Rejected(PolicyDenied)` | 없음(사람이 보지 않는 대화) |
| 승인 응답의 ID·해시가 확인 요청과 다름 | — | `Rejected(PolicyDenied)`(추론) | 없음. 요청을 보내지 않는다 |
| 바꾸기·지우기 대상이 개인 항목이 아님 | — | `Rejected(PolicyDenied)`(추론) | 개인 일정만 바꿀 수 있다는 안내 |
| 학교 세션 만료 | `SessionExpired` | 없음. 유스케이스가 창 안에서 처리하고, 창을 닫거나 대기 상한을 넘기면 `NeedsLogin` | OTP 입력 창(FS-03 F5). 성공하면 승인된 요청을 다시 보낸다 |
| 학생 권한으로 막힘(401 헤더 없음, 404) | `NotAvailable` | `NotAvailable` | 그 기능은 쓸 수 없다는 안내 |
| 응답 형식이 다름(필수 필드 없음) | `SourceChanged` | `SourceChanged` | 앱 업데이트가 필요하다는 안내 |
| 요청 제한 | `RateLimited` | `Temporary` | 다시 시도. 제한 응답은 서버가 처리하지 않았으므로 코어가 백오프 재시도한다([data_access README](../layers/data_access/README.md) HTTP 정책) |
| 요청을 보낸 뒤 연결이 끊김(결과 모름) | `Network` | `Temporary` | 코어는 다시 보내지 않는다. 학생에게 캘린더에서 확인하라고 안내(추론, §6) |

- 화면은 `Rejected`의 사유([application §4.5](../layers/02_application.md#45-application-오류))만 보고 안내를 고른다.

## 4. 수용 기준

| # | 기준 | 확인 방법 | 시나리오 |
|---|---|---|---|
| AC1 | "이 과제 마감을 내 캘린더에 넣어 줘"에 확인 창이 뜨고, 승인하면 Canvas 개인 캘린더에 일정이 생기며 대화에 반영 완료가 보인다 | 실제 계정으로 요청하고 Canvas 웹 캘린더에서 확인한다 | S8-1 ★ |
| AC2 | 확인 창에서 거부하면(창 닫기 포함) 일정이 생기지 않고 결과가 `Rejected(UserDeclined)`다 | `UserConfirmation` 메모리 구현으로 거부를 돌려주는 테스트 + 실제 계정 | S8-1 ★ |
| AC3 | 확인 창의 값이 실제로 보낸 요청 본문과 같다. AI가 다른 설명을 붙여도 창에는 요청 값만 보인다 | 요청을 기록하는 메모리 Gateway로 창 내용과 요청을 비교하는 테스트 | S8-1 |
| AC4 | 승인 응답의 요청 ID나 내용 해시가 하나라도 다르면 요청이 나가지 않는다 | 다른 ID·해시로 승인하는 테스트 | — |
| AC5 | 같은 요청 ID로 승인이 두 번 와도 요청은 한 번만 나간다 | 승인 두 번을 보내는 테스트 | — |
| AC6 | 격리 모드 대화의 쓰기 요청은 확인 창 없이 `Rejected(PolicyDenied)`로 끝나고 요청이 나가지 않는다 | 격리 모드 IPC 토큰으로 쓰기를 부르는 테스트 | — |
| AC7 | 확인 창이 떠 있는 동안 사이드바의 그 대화가 "Approval"이고, 창이 닫히면 원래 상태로 돌아간다 | `presenters/shell` 테스트 + 화면 확인 | S8-1 |
| AC8 | `cdx` 쓰기 명령에 사용자 ID·학번 인자가 없고, `user_{id}`는 세션의 ID로 채워진다 | `cdx --help`와 요청 기록 테스트 | — |
| AC9 | 과목 일정(`course_{id}` 문맥)을 바꾸거나 지우는 요청은 확인 창 없이 거부된다 | 대상 항목 문맥을 고정한 테스트 | — |
| AC10 | 승인 뒤 세션이 만료돼 있으면 OTP 입력 뒤 같은 요청이 한 번 나가고, 확인 창은 다시 뜨지 않는다 | 메모리 `AuthGateway`로 만료를 재현한 테스트 | S2-1 |
| AC11 | 요청을 보낸 뒤 연결이 끊기면 코어가 다시 보내지 않는다(중복 일정 없음) | 응답 전에 연결을 끊는 테스트 | — |
| AC12 | 플래너 메모 만들기·바꾸기·지우기와 완료 표시·해제가 각각 확인 창을 거쳐 Canvas에 반영된다 | 실제 계정으로 하나씩 실행하고 Canvas 플래너에서 확인한다 | S8-1 ★ |

★ 실제 학교 계정과 서버가 있어야 확인할 수 있다(실제 대화로 확인하면 유료 Claude 계정도 필요하다).

## 5. User Flow (UF-11)

이 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 그릴 갈래: 확인 뒤 쓰기(F1·F3~F6), 거부(창 닫기 포함), 격리 모드 거부(F2), 세션 만료 뒤 승인된 요청 다시 보내기(F5), `Peer` 읽은 대화의 첫 쓰기(F7).

## 6. 미확인 항목

- **개인 일정·플래너 조회가 아키텍처에 없다**: F1은 바꾸기·지우기 전에 대상 항목을 읽어 확인하는데, 그 읽기와 "내 일정 보여 줘" 같은 조회를 맡는 유스케이스가 [application §3](../layers/02_application.md#3-유스케이스-후보)에 없다("조회" 행은 할 일·과목·차시·과제·공지·출결·성적뿐). 바꾸기·지우기의 대상 확인은 쓰기 유스케이스 안의 읽기로 두고(추론), 학생에게 보이는 일정·플래너 조회는 UX Strategy §12 후보로 올린다. 이것이 정해지기 전에는 바꾸기·지우기의 대상 ID를 AI가 얻을 경로도 없다
- 쓰기 경로 6개(`POST·PUT·DELETE /calendar_events`, `POST·PUT·DELETE /planner_notes`, `POST·PUT /planner/overrides`)는 모두 "미확인"이다([02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로)). 학생 권한으로 만들고 지울 수 있는지, 응답 필드가 무엇인지 실제 계정으로 기록한다. 완료 표시 해제가 `PUT`으로 되는지도 함께 본다
- **확인 대기 상한**: 확인 창을 기다리는 동안 `cdx` 요청이 열려 있다. AI 제공자의 도구 호출 제한 시간을 넘기면 어떻게 끝낼지(오류 종류, 창을 닫을지) 정해야 한다. FS-03 §6의 같은 항목과 함께 정한다
- **ID·해시 불일치와 대상이 개인 항목이 아닌 경우의 사유**: [application §4.5](../layers/02_application.md#45-application-오류)의 `Rejected` 사유 중 `PolicyDenied`가 가장 가깝다고 보고 썼다(추론). 따로 사유를 둘지 정한다
- **바꾸기·지우기의 대상 제한**: [02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로)는 "과목 일정은 만들지 않는다"만 적는다. 바꾸기·지우기도 개인 항목만으로 제한한다는 F1 규칙은 추론이다. Canvas가 학생의 과목 일정 수정을 어차피 막는지 확인하고, 앱에서도 막을지 정한다
- **S8-1과 범위**: S8-1은 "과제 마감을 넣는다"(만들기)만 다룬다. [data_access §1.3](../layers/data_access/README.md#13-지키는-원칙)은 `PUT·DELETE`도 허용하므로 F1에 바꾸기·지우기를 넣었다. 바꾸기·지우기의 시나리오가 필요하면 UX Strategy에 더한다
- 확인 창에 바꾸기 전·뒤 값을 함께 보일지, 지울 항목의 값을 보일지(F3, 추론). 문구와 모양은 Design Process에서 정한다
- 내용 해시를 만드는 범위(메서드·경로·본문 전체)와 해시 함수, 요청 ID의 재사용 금지 기간(F3·F5, 추론)
- 반영 완료를 앱 메시지로 보일지 확인 창 안에서 보일지(F6, 추론). [UX Strategy §8](../06_ux_strategy.md#8-상태-오류-긴-작업)의 원칙으로 앱 메시지라고 봤다
- 거부 뒤 AI가 같은 쓰기를 다시 요청하지 않게 하는 문장을 `ToolGuide`에 둘지(F6, 추론). [entrypoints §6](../layers/04_entrypoints.md#6-toolguide)에 없는 문장이다
- 요청 뒤 연결이 끊겨 결과를 모를 때 코어가 Canvas를 조회해 생겼는지 확인할지, 학생에게 확인을 맡길지(§3, 추론)
- 입력 검사(끝 < 시작, 필수 값 없음)의 실패를 어떤 형태로 AI에 돌려줄지(F1, 추론). application 오류에 해당하는 것이 없다
