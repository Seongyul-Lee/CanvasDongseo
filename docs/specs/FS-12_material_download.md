# FS-12 자료·동영상 내려받기

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 초안 |
| 시나리오 | S4-2, S7-4, S10-3 ([UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)) |
| 화면 | 대화(앱 전용 화면 없음, [UX Strategy §11](../06_ux_strategy.md#11-대응표)) |
| 유스케이스 | 자료 묶음: 강의 자료·동영상 내려받기 ([application §3](../layers/02_application.md#3-유스케이스-후보)), `MaterialDownloadGateway`, `MediaDownloadGateway` ([application §4.1](../layers/02_application.md#41-데이터-repository--gateway)) |
| 근거 | [03_learningx §2](../layers/data_access/03_learningx.md#2-경로), [05_commons](../layers/data_access/05_commons.md), [domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙), [learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일), [entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령), [PRD §8.3](../01_prd.md#83-학습-지원-기능) |

> **이 문서의 역할**: 학생이 대화로 요청한 강의 자료(PDF·수업자료실 파일)와 주차별 강의 동영상을 앱이 정한 폴더에 내려받고, 저장 위치를 알려 주는 기능의 동작과 수용 기준을 정한다.
> 받는 경로와 다운로드 동작은 [05_commons](../layers/data_access/05_commons.md), 이름 규칙은 [domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙)가 맡는다. 자료 목록 조회는 FS-08, 받은 동영상으로 슬라이드 PDF·스크립트를 만드는 일은 FS-13이다([README §3](README.md#3-목록)).

---

## 1. 범위

| 하는 것 | 하지 않는 것 |
|---|---|
| 주차학습의 강의 자료(PDF 등) 내려받기 | 내 제출물·내 파일(`submission.attachments`, `/users/self/files`) 내려받기. 경로는 있지만 이 유스케이스에 넣지 않는다(§6) |
| 수업자료실 자료(일반 파일 포함) 내려받기 | 서버의 자막·대본 받기([05_commons §3](../layers/data_access/05_commons.md#3-다운로드-동작)) |
| 주차별 강의 동영상 내려받기 | 동영상 시청 기록·진도 바꾸기([PRD §9.1](../01_prd.md#91-비목표-non-goals)) |
| 앱이 정한 학기/과목 폴더에 저장하고 위치 알리기 | 저장 위치를 AI나 학생이 경로로 정하기 |
| 끊긴 다운로드 이어 받기 | 받은 파일 지우기(AI가 권한 확인을 거쳐 한다, F6) |

## 2. 기능 요구사항

### F1 받을 대상 고르기

- **입력**: AI가 `cdx`로 넘기는 과목과 항목 식별자(LearningX의 `item_id` 또는 수업자료실 항목, 그 안의 `content_id`). 목록은 FS-08의 조회 결과에서 온다.
- 파일 경로 인자는 받지 않는다. 저장 위치는 앱이 정한다(F3, [entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령)).
- 대상은 [03_learningx §2](../layers/data_access/03_learningx.md#2-경로)의 주차학습 항목(`item_content_data.content_id`, `content_type`)과 수업자료실 항목(`commons_content`)이다. `commons_content`가 없는 수업자료실 항목은 안내문만 있으므로 받을 파일이 없다고 돌려준다.
- 학교 데이터 요청이므로 학교 계정 연결이 필요하다. 목록 조회(LearningX)에서 세션이 만료되면 FS-03 F5처럼 OTP를 받고 원래 요청을 다시 실행한다.

### F2 내려받기

- 종류에 따라 [05_commons §2](../layers/data_access/05_commons.md#2-경로)의 경로를 고른다: PDF는 `content_download_uri`, 동영상은 기본 미디어의 progressive 주소, `content.php`가 지원하지 않는 파일은 일반 파일 경로.
- 다운로드 동작(크기 확인, `.part`로 받은 뒤 크기 확인 후 이름 바꾸기, 이어 받기, 스트리밍)은 [05_commons §3](../layers/data_access/05_commons.md#3-다운로드-동작)을 따른다.
- HTML 로그인 화면 같은 예상 밖 응답은 완료 파일로 만들지 않고 `SourceChanged`로 끝낸다. 남은 `.part`는 지운다(추론, §6).
- 같은 `content_id`가 여러 주차에 나오면([03_learningx §2](../layers/data_access/03_learningx.md#2-경로)) 요청받은 항목의 주차·차시로 이름을 정한다(추론, §6).

### F3 저장 위치

- 파일은 [domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙)의 키(학기, 과목, 종류, `{NN}주차_{M}차시_{제목}`)로 정한 위치에 둔다. 동영상은 슬라이드 PDF·스크립트와 키에서 종류만 다르므로 짝이 된다. 학습 기능(FS-13·FS-14)이 이 짝으로 주차별 자료를 모은다.
- 위치의 모양은 [learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일)와 같은 `<학습 자료 폴더>/<학기>/<과목>/<종류>/…`로 본다. 동영상·강의 자료의 종류 폴더 이름은 아직 문서에 없다(추론, §6).
- 학기·과목명·제목은 학교 데이터이므로 이름 정리 규칙(경로 구분자, 금지 문자, 예약 이름, 길이, 충돌)을 반드시 거친다([domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙), security R4). 일반 파일의 확장자는 `Content-Disposition`에서 오므로 확장자도 같은 정리를 거친다(추론, §6).
- 파일은 앱이 정한 학습 자료 폴더 밖에 만들지 않는다.

### F4 저장 위치 알리기

- 결과로 저장한 파일의 절대 경로, 파일 크기, 종류를 돌려준다. AI는 이 경로를 답에 그대로 보인다(S7-4 "저장 위치가 보인다", S4-2 "정해진 폴더에 파일이 생긴다").
- 이미 받은 파일을 다시 요청하면 다시 받지 않고 기존 경로를 돌려준다(추론, §6).

### F5 여러 개 받기

- "3주차 자료 다 받아 줘"처럼 여러 항목을 요청하면 항목마다 F2~F4를 한 번씩 하고, 항목별 결과(성공 경로 또는 실패 사유)를 함께 돌려준다. 한 항목이 실패해도 나머지는 계속 받는다(추론, §6).
- 출처마다 동시 요청 1개 원칙을 따른다([data_access README](../layers/data_access/README.md) HTTP 정책).

### F6 받은 파일 지우기와의 경계 (S10-3)

- 앱에는 받은 파일을 지우는 유스케이스와 `cdx` 명령이 없다. "다운로드 받은 동영상 모두 지워 줘"는 AI가 자기 파일 도구로 지우고, 그 도구 호출은 대화 모드의 권한 확인 창(`PermissionRequest`)을 거친다([ai_adapters §3](../layers/05_ai_adapters.md#3-실행-모드)). 권한 확인의 동작은 FS-07이 정한다([README §3](README.md#3-목록)).
- 이 기능이 정하는 것은 AI가 지울 대상을 찾을 수 있게 **저장 위치가 고정돼 있고 F4로 알려진다**는 것까지다.
- 파일이 지워진 뒤 같은 항목을 요청하면 처음부터 다시 받는다.

## 3. 예외

| 상황 | 판정 | application 오류 | 다음 행동(P5) |
|---|---|---|---|
| 목록 조회 중 학교 세션 만료 | `SessionExpired` | `NeedsLogin` | OTP 입력 창(FS-03 F5). 성공하면 원래 요청을 다시 실행 |
| 받을 파일이 없는 항목(안내문만 있음) | — | `NotAvailable` | 받을 파일이 없다는 안내 |
| XML 형식이 다르거나 `DOCTYPE`·`ENTITY`가 있음, 크기 초과 | `SourceChanged` | `SourceChanged` | 앱 업데이트가 필요하다는 안내 |
| 다운로드 응답이 HTML 등 예상 밖 내용 | `SourceChanged` | `SourceChanged` | 위와 같음. 완료 파일을 만들지 않는다 |
| 받은 크기가 예상 크기와 다름 | `SourceChanged`(추론) | `SourceChanged` | 다시 시도 안내(추론, §6) |
| 동영상 CDN 403(`Referer` 문제 등) | `SourceChanged`(추론) | `SourceChanged` | 앱 업데이트 안내 |
| 네트워크 끊김·5xx | `Network` | `Temporary` | 다시 시도. 강한 ETag가 있으면 이어 받는다 |
| 요청 제한 | `RateLimited` | `Temporary` | 다시 시도 |
| 디스크 공간 부족, 폴더에 쓸 수 없음 | — | 정해지지 않음(§6) | 정해지지 않음 |

## 4. 수용 기준

| # | 기준 | 확인 방법 | 시나리오 |
|---|---|---|---|
| AC1 | "운영체제 2주차 강의 자료 받아 줘"에 파일이 학습 자료 폴더의 학기/과목 아래에 생기고, 답에 그 절대 경로가 보인다 | 실제 계정으로 요청하고 경로에서 파일을 연다 | S4-2, S7-4 ★ |
| AC2 | 주차별 강의 동영상을 받으면 같은 규칙의 위치에 재생할 수 있는 `.mp4`가 생기고, 답의 경로와 실제 위치가 같다 | 실제 계정으로 요청하고 재생한다 | S7-4 ★ |
| AC3 | 받은 동영상과, 그 동영상으로 만든 슬라이드 PDF·스크립트의 키가 종류만 다르다 | 이름 규칙 단위 테스트(domain) | S7-1 |
| AC4 | 과목명·제목에 `/`, `..`, `:`, 예약 이름(`CON` 등)이 있어도 파일이 학습 자료 폴더 밖에 생기지 않고 정리된 이름으로 생긴다 | 이름을 고정한 테스트 | — |
| AC5 | 다운로드가 중간에 끊기면 완료 이름의 파일이 생기지 않는다. 다시 요청하면 강한 ETag가 있을 때 이어 받고, 없거나 서버가 Range를 무시하면 처음부터 받는다 | `wiremock`으로 끊김·Range 무시를 재현한 테스트 | — |
| AC6 | 다운로드 응답이 HTML 로그인 화면이면 완료 파일이 생기지 않고 결과가 `SourceChanged`다 | 응답을 고정한 테스트 | — |
| AC7 | 동영상 요청에 `Referer: https://dcms.dongseo.ac.kr/`가 붙고, 동영상 전체가 메모리에 올라가지 않는다 | 요청 기록 테스트 + 큰 응답에서 메모리 사용 확인 | — |
| AC8 | `cdx` 다운로드 명령에 저장 경로 인자가 없다 | `cdx --help`와 IPC 요청 타입 확인 | — |
| AC9 | 받을 파일이 없는 수업자료실 항목은 `NotAvailable`로 끝나고 빈 파일이 생기지 않는다 | 응답을 고정한 테스트 | S4-2 |
| AC10 | 여러 항목을 요청했을 때 한 항목이 실패해도 나머지는 받고, 결과에 항목별 성공·실패가 보인다 | 한 항목만 실패하는 테스트 | S4-2 |
| AC11 | "다운로드 받은 동영상 모두 지워 줘"에 권한 확인 창이 뜨고, 승인하면 동영상 폴더가 비며, 학습 자료 폴더 밖의 파일은 그대로다 | 실제 앱에서 요청한다 | S10-3 |

★ 실제 학교 계정과 서버가 있어야 확인할 수 있다.

## 5. User Flow (UF-12)

이 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 그릴 갈래: 한 항목 받기(F1~F4), 여러 항목 받기(F5), 끊긴 다운로드 이어 받기, 세션 만료 뒤 이어 가기, 받을 파일이 없음·형식 바뀜, 받은 파일 지우기(F6, FS-07의 권한 확인으로 넘어가는 지점까지).

## 6. 미확인 항목

- **학습 자료 폴더의 위치**: [learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일)와 [entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령)는 `<학습 자료 폴더>`를 쓰지만 실제 위치(기본값, OS별 경로)와 학생이 바꿀 수 있는지는 어디에도 없다. [UX Strategy §7.5](../06_ux_strategy.md#75-설정)의 설정에도 이 항목이 없다. 바꿀 수 있게 하려면 아키텍처에 없음 → UX Strategy §12 후보
- **동영상·강의 자료의 종류 폴더**: [domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙)의 종류는 "동영상·슬라이드·스크립트"뿐이고, 강의 자료(PDF·수업자료실 파일)의 종류와 키가 없다. 수업자료실 파일에는 주차·차시가 없을 수 있어 `{NN}주차_{M}차시_{제목}` 키를 쓸 수 없다. 강의 자료의 키와 폴더 이름을 domain에 정한다. [learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일) 표에도 동영상 행이 없다
- **받은 파일을 저장하는 구현의 위치**: `MaterialDownloadGateway`·`MediaDownloadGateway`는 `ports/school`(data_access 구현)이고, 키를 경로로 바꾸는 일은 `LearningOutputRepository`(`local_store/learning`, 슬라이드 PDF·스크립트만)가 맡는다([application §4.1·§4.3](../layers/02_application.md#4-출력-포트)). 내려받은 파일의 키 → 경로 변환과 원자적 저장을 어느 포트·구현이 맡는지 정한다
- **긴 다운로드와 작업**: 동영상은 0.5GB까지 실측됐는데([learning_adapters §3.4](../layers/06_learning_adapters.md#34-상한-security-r4)), [application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)은 작업을 `usecases/learning`에만 둔다. 다운로드가 AI 도구 호출 제한 시간을 넘길 수 있다. 다운로드를 작업으로 실행할지(취소·진행 표시 포함), 아니면 `cdx` 요청 안에서 끝낼지 정한다. 작업으로 하면 `usecases/school`이 `usecases/jobs`에 의존하게 되므로 [System Architecture §3.2](../02_system_architecture.md#32-의존-규칙-cargo-dependencies로-강제)를 고쳐야 한다
- **다운로드 크기 상한**: [learning_adapters §3.4](../layers/06_learning_adapters.md#34-상한-security-r4)의 4GB 상한은 학습 작업의 입력 상한이다. 내려받기 자체에도 상한을 둘지, 넘으면 `Rejected(LimitExceeded)`를 쓸지 정한다
- **디스크 공간 부족·쓰기 실패**: application 오류([§4.5](../layers/02_application.md#45-application-오류))에 맞는 종류가 없다. 아키텍처에 없음 → UX Strategy §12 후보
- 이미 받은 파일을 다시 요청할 때 다시 받지 않을지, 서버의 `updated_at`이 바뀌었으면 다시 받을지(F4, 추론)
- 같은 `content_id`가 여러 주차에 나올 때 파일을 하나만 둘지 주차마다 둘지(F2, 추론)
- 일반 파일의 확장자(`Content-Disposition`)도 이름 정리 규칙을 거친다는 것(F3, 추론). [domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙)의 정리 대상은 "학기, 과목명, 제목"이다
- 받은 크기가 예상과 다를 때·CDN 403일 때의 판정(§3, 추론). [05_commons §3](../layers/data_access/05_commons.md#3-다운로드-동작)은 동작만 적고 오류 종류를 적지 않는다
- 여러 항목 요청에서 일부 실패를 돌려주는 모양(F5, 추론)
- 내 제출물(`submission.attachments[].url`)과 내 파일(`/users/self/files`)은 [02_canvas §2](../layers/data_access/02_canvas.md#2-rest-경로)에 경로가 있지만 [application §3](../layers/02_application.md#3-유스케이스-후보)의 "강의 자료·동영상"에 들지 않는다고 보고 범위에서 뺐다. 넣을지 정한다
- [PRD §8.3](../01_prd.md#83-학습-지원-기능)은 "LearningX 공지"도 내려받는 대상으로 적는다. 공지는 FS-09의 조회로 다루고 파일로 받지 않는다고 봤다. 공지를 파일로 저장해야 하는지 확인한다
- AI가 S10-3에서 지울 폴더를 어떻게 아는지: F4의 경로와 정책 원문(FS-15)의 저장 위치 설명으로 안다고 봤다(추론). 정책 원문에 학습 자료 폴더 위치를 넣을지 FS-15와 함께 정한다
- S7-4의 저장 위치를 AI 답이 아니라 앱 메시지(예: Finder에서 열기 버튼)로 보일지. 아키텍처에 없음 → UX Strategy §12 후보
