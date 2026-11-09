# FS-08 할 일·과목 조회

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 초안 |
| 시나리오 | S3-1, S3-2, S4-1 ([UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)) |
| 화면 | 대화만([UX Strategy §7.3](../06_ux_strategy.md#73-mvp-이후-화면): A3는 대화로만 다룬다). 원문 링크는 OS 브라우저 |
| 유스케이스 | 조회 묶음의 이번 주 할 일, 과목·차시·과제 조회 ([application §3](../layers/02_application.md#3-유스케이스-후보)). 이름은 미정 |
| 근거 | [02_canvas](../layers/data_access/02_canvas.md), [03_learningx](../layers/data_access/03_learningx.md), [domain §4](../layers/01_domain.md#4-업무-규칙-후보), [entrypoints §4·§5](../layers/04_entrypoints.md#4-cdx-명령), [UX Strategy §9](../06_ux_strategy.md#9-ai-응답-경험-원칙) |

> **이 문서의 역할**: 학생이 대화로 이번 주 할 일, 특정 과목의 남은 일, 과제의 요구 사항을 물을 때 코어가 무엇을 조회하고 무엇을 돌려주는지와 수용 기준을 정한다.
> 경로와 응답 형식은 [02_canvas](../layers/data_access/02_canvas.md)·[03_learningx](../layers/data_access/03_learningx.md), 세션이 끊겼을 때의 동작은 [FS-03](FS-03_school_account.md) F5, 출처 장애 표시는 [FS-06](FS-06_connection_status.md), 과제 첨부·자료 내려받기는 [FS-12](FS-12_material_download.md)가 맡는다.

---

## 1. 범위

| 하는 것 | 하지 않는 것 |
|---|---|
| 이번 주 할 일: 모든 수강 과목의 미제출 과제와 시청하지 않은 차시를 마감 순으로 모으기 | 과제 제출, 진도·출석 처리 등 학교에 쓰는 요청([data_access §1.3](../layers/data_access/README.md)) |
| 과목 하나의 남은 과제·미시청 차시·마감 | 할 일 전용 화면(MVP 이후, [UX Strategy §7.3](../06_ux_strategy.md#73-mvp-이후-화면)) |
| 과제 하나의 요구 사항(설명, 배점, 제출 방식, 마감, 루브릭)과 원문 링크 | 과제 첨부·강의 자료 내려받기(FS-12) |
| 조회한 과목 범위와 조회 시각을 결과에 붙이기(P3) | 출결 집계(FS-10), 성적(FS-10) |
| 출처 하나가 실패해도 나머지 결과를 돌려주기(S2-7) | 과제 마감을 캘린더·플래너에 넣기(FS-11) |

## 2. 기능 요구사항

### F1 수강 과목 범위

- 조회 대상은 Canvas의 활성 수강 과목이다(`courses?enrollment_state=active&include[]=term`, [02_canvas §4](../layers/data_access/02_canvas.md#4-graphql) 결정).
- 모든 조회 결과에 **조회한 과목 목록**(과목 ID, 과목명)과 **조회 시각**을 붙인다. 할 일이 0개인 과목도 목록에 남긴다. S3-1 점검("수강 과목 수와 응답의 과목 수가 같다")이 이 값을 쓴다.
- 활성 과목에 수업이 아닌 과목(비교과 등)이 섞일 수 있는지는 확인하지 않았다(§6).

### F2 이번 주 할 일 (S3-1)

- **입력**: 없음(기간은 domain 규칙으로 코어가 정한다, §6).
- **동작**: 모든 수강 과목에서 아래 항목을 모아 마감 시각 순으로 정렬한다. 판정은 domain 규칙(이번 주 할 일 판정, [domain §4](../layers/01_domain.md#4-업무-규칙-후보))으로 한다.

| 항목 | 출처 | 포함 조건 |
|---|---|---|
| 과제 | Canvas 과제 + 내 제출 | 마감이 이번 주 안이고 미제출 |
| 차시 | LearningX 주차학습(`content_type` = `attendance_item`) | 마감(`due_at`)이 이번 주 안이고 완료가 아님 |

- **차시 완료 판정**: `completion_determinable`이 참일 때만 `completed`를 믿는다([03_learningx §2](../layers/data_access/03_learningx.md#2-경로)). 거짓이면 완료로 보지 않고 "완료 여부 확인 불가"로 표시해 목록에 남긴다(추론, §6).
- **미제출 판정**: REST의 `unsubmitted`와 GraphQL의 빈 제출 노드는 DS에서 같은 "미제출"로 바뀌어 들어온다([02_canvas §4](../layers/data_access/02_canvas.md#4-graphql)). 유스케이스는 둘을 구별하지 않는다.
- **시각**: REST(UTC)와 GraphQL(`+09:00`) 시각은 시간대가 있는 값으로 들어온다. 비교·정렬은 시각 값으로 하고, 표시는 한국 시각으로 한다.
- **출력**(항목마다): 종류(과제·차시), 과목명, 제목, 마감 시각, 차시면 주차·차시 번호와 지각 마감·잠금 시각(있으면), 원문 링크, 신뢰 등급 `Own`([security §4.1](../cross_cutting/01_security.md#41-신뢰-등급)). 결과 전체에 F1의 과목 목록과 조회 시각, F5의 출처별 상태.

### F3 과목 하나의 남은 일 (S3-2)

- **입력**: 과목 ID. 과목명을 ID로 바꾸는 일은 AI가 과목 목록(F1)을 보고 한다.
- **출력**: 그 과목의 남은 과제(미제출), 시청하지 않은 차시, 각 마감을 한 결과에 담는다. F2와 같은 판정·시각 규칙을 쓴다. 기간은 F2와 같다(S3-2 상황은 "이번 주").
- 과목이 LearningX 주차학습을 쓰지 않으면(도구 없음, `NotAvailable`) 차시 칸을 "이 과목은 주차학습을 쓰지 않음"으로 두고 과제는 그대로 돌려준다. 오류로 다루지 않는다([02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로) 아래 규칙).
- 결과 값은 Canvas·LearningX 웹 화면에서 학생이 보는 값과 같아야 한다(S3-2 점검).

### F4 과제 요구 사항 (S4-1)

- **입력**: 과목 ID, 과제 ID.
- **출력**: 과제 상세([02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로) `assignments/{assignment_id}`)의 설명, 배점, 채점 방식, 제출 방식, 마감·잠금 시각, 루브릭(과제 상세의 `rubric` 필드), 원문 링크(과제 페이지 주소).
- 설명은 교수가 쓴 글이라 `Official`이다. 대화 AI에 그대로 주고, AI가 요약한다(R1·R2). `Peer` 격리 요약 대상이 아니다.
- 설명은 HTML이다. AI에게는 글자로 바꿔 준다(추론, §6). 설명 안의 링크·첨부는 주소만 남기고 따라가지 않는다.
- 원문 링크는 대화에 링크로 보이고, 누르면 **OS 기본 브라우저**로 연다. 앱 WebView 안에서 이동하지 않는다([presentation §2.3](../layers/03_presentation.md#23-신뢰하지-않는-html과-webview-보안)).

### F5 출처별 부분 실패

- 과제(Canvas)와 차시(LearningX)는 따로 조회한다. 한 출처가 실패해도 다른 출처의 결과를 돌려준다(S2-7). 근거는 "전면 장애가 아니라 경로별로 저하되도록 설계한다"([PRD §9.2](../01_prd.md#92-제약과-위험))다. 여러 출처를 합치는 다른 조회도 이 규칙을 따른다.
- 결과에 출처별 상태(성공, 실패와 그 application 오류)와 실패한 과목을 붙인다. AI는 이것으로 "LearningX 차시는 지금 가져오지 못했다"처럼 빠진 범위를 말한다. 어떤 출처 이름을 보일지는 §6.
- 과목 하나의 `NotAvailable`은 실패가 아니라 정상 결과다(F3).
- 세션 만료는 부분 실패로 넘기지 않는다. [FS-03](FS-03_school_account.md) F5대로 OTP를 받아 원래 요청 전체를 다시 실행한다. 창을 닫거나 대기 상한을 넘겨 `NeedsLogin`이 되면 부분 결과 없이 요청 전체가 `NeedsLogin`으로 끝난다.

### F6 AI에게 주는 형태

- 결과는 `cdx` 조회 명령으로 AI에 간다. 명령 하나가 F2·F3·F4 유스케이스 하나에 대응한다([entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령)). 명령 이름은 미정이다.
- AI용 Presenter는 항목마다 신뢰 등급을 붙이고, 내부 URL 쿼리와 요청 단계 정보는 내보내지 않는다([entrypoints §5](../layers/04_entrypoints.md#5-ai용-presenter)). 원문 링크는 학생이 여는 주소이므로 내보낸다.
- 응답 형식(목록에 출처·조회 시각·원문 링크)은 `ToolGuide`의 R1이 정한다. 코어는 그 재료를 빠짐없이 준다.

## 3. 예외

| 상황 | 판정 | application 오류 | 다음 행동(P5) |
|---|---|---|---|
| 학교 세션 만료 | `SessionExpired` | 없음. 유스케이스가 창 안에서 처리하고, 창을 닫거나 대기 상한을 넘기면 `NeedsLogin` | OTP 입력 창, 입력 뒤 원래 요청 다시 실행([FS-03](FS-03_school_account.md) F5) |
| 과목이 주차학습·과제 기능을 쓰지 않음 | `NotAvailable` | `NotAvailable` | 그 칸만 "쓰지 않음". 나머지는 그대로(F3) |
| LearningX 또는 Canvas 응답 형식이 바뀜 | `SourceChanged` | `SourceChanged` | 그 출처의 결과만 빠지고, 앱 업데이트가 필요하다는 안내. 다른 출처 결과는 돌려준다(F5) |
| 과제 GraphQL 묶음 조회 실패 | `SourceChanged`·`Network` | — | REST 구현으로 바꿔 끼울 수 있다([02_canvas §4](../layers/data_access/02_canvas.md#4-graphql)). 실행 중 자동 전환할지는 §6 |
| 네트워크 오류·요청 제한 | `Network`·`RateLimited` | `Temporary` | 다시 시도. 부분 실패면 F5 |
| 과제 ID가 그 과목에 없음 | Canvas 404 | `NotAvailable`(추론, §6) | AI가 과목의 과제 목록을 다시 조회 |
| 학교 계정 로그아웃 상태 | — | `NeedsLogin` | 연결 행으로 가는 버튼(S10-3) |

## 4. 수용 기준

| # | 기준 | 확인 방법 | 시나리오 |
|---|---|---|---|
| AC1 | "이번 주 뭐 해야 돼?"에 마감 순 목록과 함께 조회한 과목 범위와 조회 시각이 보이고, 과목 수가 수강 과목 수와 같다 | 실제 대화에서 질문하고 Canvas 수강 과목 수와 비교한다 ★ | S3-1 |
| AC2 | 할 일 결과는 마감 시각 순이고, 할 일이 없는 과목도 조회한 과목 목록에 있다 | 고정 데이터 유스케이스 테스트 | S3-1 |
| AC3 | 제출한 과제와 완료한 차시는 할 일에 없다. REST의 `unsubmitted`와 GraphQL의 빈 제출 노드는 같은 결과를 낸다 | 고정 응답 테스트(두 DS 구현에 같은 계약 테스트) | S3-1 |
| AC4 | `completion_determinable`이 거짓인 차시는 완료로 판정되지 않는다 | domain 단위 테스트 | S3-2 |
| AC5 | 같은 시각을 UTC(`…Z`)와 `+09:00`으로 받아도 정렬과 이번 주 포함 여부가 같고, 표시는 한국 시각이다 | domain·DS 단위 테스트 | S3-1 |
| AC6 | 특정 과목을 물으면 남은 과제, 시청하지 않은 차시, 각 마감이 한 응답에 나오고, 값이 Canvas·LearningX 웹 화면과 같다 | 실제 과목으로 질문하고 웹 화면과 비교한다 ★ | S3-2 |
| AC7 | 주차학습을 쓰지 않는 과목을 물으면 과제는 나오고 차시 칸만 "쓰지 않음"이다. 오류 안내가 나오지 않는다 | `NotAvailable`을 내는 DS로 유스케이스 테스트 | S3-2 |
| AC8 | 과제 요구 사항을 물으면 요약과 원문 링크가 보이고, 링크를 누르면 OS 기본 브라우저에서 과제 페이지가 열린다. 앱 창 안에서 이동하지 않는다 | 실제 대화에서 링크를 누른다 ★ | S4-1 |
| AC9 | LearningX가 실패해도 이번 주 할 일에 Canvas 과제가 나오고, 차시를 가져오지 못했다는 것이 함께 보인다 | LearningX DS가 `SourceChanged`를 내는 테스트 + 응답 확인 | S3-1, S2-7 |
| AC10 | 조회 중 세션이 만료되면 OTP 창이 뜨고, 입력 뒤 같은 대화에 할 일 답이 이어진다 | 쿠키를 지워 만료를 재현한 뒤 질문한다 ★ | S2-1 |
| AC11 | `cdx` 출력에 쿠키·토큰, 내부 URL 쿼리, 요청 단계 정보가 없고, 항목마다 신뢰 등급이 있다 | AI용 Presenter 테스트(출력 JSON 검사) | — |

★ 실제 학교 계정과 서버가 있어야 확인할 수 있다.

## 5. User Flow (UF-08)

이 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 그릴 갈래: 이번 주 할 일(성공, 출처 하나 실패, 세션 만료 → OTP → 이어 가기), 과목 하나의 남은 일(주차학습 쓰지 않음 포함), 과제 요구 사항과 원문 링크 열기.

## 6. 미확인 항목

- **"이번 주"의 범위가 정해지지 않았다.** 월~일(한국 시각)인지 오늘부터 7일인지, 이미 마감이 지났지만 늦은 제출·지각 시청이 가능한 항목(`late_at`, `lock_at` 전)을 넣을지. [domain §4](../layers/01_domain.md#4-업무-규칙-후보)는 "이번 주 할 일 판정(마감, 미완료 차시, 미제출 과제)"만 쓴다. domain 규칙으로 정하고 고정 테스트를 둔다. `cdx`가 기간 인자를 받을지도 함께 정한다
- **응답에 출처 이름을 붙이는 방법.** [UX Strategy §5](../06_ux_strategy.md#5-경험-원칙) P3와 [§9](../06_ux_strategy.md#9-ai-응답-경험-원칙) R1은 "출처·조회 시각·원문 링크를 붙인다"고 쓰지만, [application §2](../layers/02_application.md#2-원칙)는 "출처 이름(Canvas, LearningX …)은 이 레이어에 나타나지 않는다"고 쓴다. 출력 DTO에 중립 출처 구분(예: 과제·차시 종류, 원문 링크의 호스트)만 두고 AI가 이름을 붙이는지, AI용 Presenter가 이름을 붙이는지 정해야 한다. F5의 "어느 출처가 실패했는지" 표시도 같다
- **이번 주 할 일의 Canvas 쪽 경로.** [data_access §2.3](../layers/data_access/README.md)은 "이번 주 할 일 = Canvas planner + LearningX 미완료 차시"를 예로 들고, [02_canvas §4](../layers/data_access/02_canvas.md#4-graphql) 결정은 "과목별 과제 + 내 제출은 GraphQL"이다. planner(`/planner/items`)는 과제 외에 공지·토론도 날짜순으로 준다. 할 일에 과제만 넣을지 planner의 다른 항목(토론 마감 등)도 넣을지, 어느 경로를 쓸지 Repository 설계 때 정한다. 포트 이름(`TodoRepository`)도 [application §4.1](../layers/02_application.md#4-출력-포트) 예 목록에 없다
- `completion_determinable`이 거짓인 차시를 "확인 불가"로 남길지 목록에서 뺄지(F2, 추론)
- 차시의 원문 링크 주소. LearningX 주차학습 화면이나 차시 항목으로 바로 가는 주소 형식을 확인하지 않았다([03_learningx §2](../layers/data_access/03_learningx.md#2-경로)에 없음). 확인 전에는 과목의 주차학습 탭(LTI) 주소를 쓸 수 있는지 본다
- 차시의 "마감"으로 `due_at`(출석 인정)을 쓰고 `late_at`·`lock_at`을 함께 보일지. 화면 값과 비교하는 S3-2 점검 때 어느 값을 기준으로 할지 학교 화면을 보고 정한다
- 활성 수강 과목(`enrollment_state=active`)에 지난 학기·비교과 과목이 섞이는지. S3-1 점검의 "수강 과목 수"를 어디서 셀지(Canvas 대시보드, 학생지원시스템 수강 내역)와 함께 정한다
- 과제 설명 HTML을 AI에 줄 때 글자로 바꾸는 위치(AI용 Presenter로 추론, F4)와 이미지·표 처리
- 과제 ID가 과목에 없을 때의 application 오류(`NotAvailable`로 추론, §3)
- GraphQL 과제 조회가 실패했을 때 실행 중 REST로 자동 전환할지, 조립할 때만 고를지([02_canvas §4](../layers/data_access/02_canvas.md#4-graphql)는 "바꿔 끼울 수 있다"만 정한다). GraphQL 별칭 묶음의 크기 상한도 미확인([02_canvas 미확인](../layers/data_access/02_canvas.md#미확인-항목))
- 수강 과목 12개 기준 LearningX 차시 조회는 출처당 동시 1개 규칙([data_access §2.5](../layers/data_access/README.md))이라 과목 수만큼 순차 요청이 된다. 응답 시간 목표를 둘지, 결과를 잠깐 기억해 둘지 정하지 않았다
