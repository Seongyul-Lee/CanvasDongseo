# CanvasDongseo — Data Access Architecture

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-10~11 로그인 세션으로 확인한 학교 시스템 응답, Instructure 개발자 문서 |

> **이 문서의 역할**: 앱이 학교 시스템의 데이터를 *어떤 구조로* 가져오는지 정의하고, 접근 구역별 문서를 묶는다. 경로와 인증의 상세는 구역별 문서(§3)에 있다.
> data_access 레이어 안쪽(application 포트 아래)만 다룬다. 전체 레이어 구성, 기술 스택, 앱 command·IPC·`cdx` CLI, AI 연동은 [System Architecture](../../02_system_architecture.md)가 맡는다.

---

## 1. 범위와 전제

### 1.1 데이터 출처

| 출처 | 주소 | 문서 |
|---|---|---|
| Canvas LMS | `canvas.dongseo.ac.kr` | [02_canvas](02_canvas.md) |
| LearningX (Xinics, Canvas에 얹힌 LTI 도구) | `canvas.dongseo.ac.kr/learningx/*` | [03_learningx](03_learningx.md) |
| 스마트출결 (전자출결) | `canvas.dongseo.ac.kr/smart-attendance/*` | [04_smart_attendance](04_smart_attendance.md) |
| 동서대 SSO | `eclass1.dongseo.ac.kr` | [01_auth](01_auth.md) (인증에만 사용) |
| Commons (강의 PDF·동영상 원본) | `dcms.dongseo.ac.kr`, 동영상 CDN `dongseo-cms.edge.naverncp.com` | [05_commons](05_commons.md) |
| 학교 공지 | `www.dongseo.ac.kr/kr/` | [06_school_site](06_school_site.md) |
| 학과·기관 공지 | `uni.dongseo.ac.kr/{사이트 코드}/` | [06_school_site](06_school_site.md) |
| 학생지원시스템 (학사행정웹서비스) | `im.dongseo.ac.kr`, SSO `dsso.dongseo.ac.kr` | [07_student_portal](07_student_portal.md) |

### 1.2 결정 사항

| # | 결정 | 근거 |
|---|---|---|
| D1 | 설계·개발은 **(B) 웹 세션 방식**으로 한다. LearningX 로그인으로 얻는 Canvas 웹 세션(`pseudonym_credentials`) 하나로 Canvas API와 LearningX를 함께 읽는다. 별도 Canvas API 토큰은 쓰지 않는다 | 초기 설정 단순화. 2026-11-08 세션으로 Canvas API GET·GraphQL 동작 확인 |
| D2 | 정식 배포 전에는 **(A) OAuth2 방식**(학교 관리자가 발급한 Developer Key)을 검토한다. 인증은 §2.5의 `RequestAuthenticator`로 교체할 수 있게 둔다 | Canvas API Policy: 여러 사용자가 쓰는 앱은 OAuth로 토큰을 받아야 하고, 사용자에게 토큰을 직접 만들게 하면 안 된다. (B)는 정책상 회색 지대다 |
| D3 | Canvas 토큰 자동 발급(`POST /profile/tokens`)과 사용자에게 토큰을 직접 발급하게 하는 안내는 쓰지 않는다 | D2와 같은 정책 조항 |
| D4 | Commons의 `content_id` 다운로드 경로를 쓴다 | 공개 API는 아니지만 학교 측이 공식 허용한다고 회신(2026-11-08, 사용자 구두 전달, 서면 사본 미확보. 서면 확보는 배포 전 과제). 범위는 `content_id` 다운로드로 한정 ([PRD §9.2](../../01_prd.md)) |
| D5 | 외부 Canvas 클라이언트 패키지에 의존하지 않는다. 필요한 Canvas 경로는 직접 만든 DS로 접근한다 | Canvas 호출이 여러 곳에 흩어지지 않고 DS 계약 하나로 모인다 |
| D6 | 학교 세션과 data_access는 앱 코어 프로세스 하나만 가진다. MCP 서버를 두지 않고, AI는 `cdx` CLI → 로컬 IPC → 코어 유스케이스로 데이터를 얻는다([System Architecture S3](../../02_system_architecture.md)) | 쿠키가 쓸 때마다 바뀌므로 세션 소유자가 둘이면 세션을 잃는다. 세션을 다른 프로세스와 나눠 쓰면 프로세스 간 잠금·키체인 재읽기가 구조의 중심이 된다 |

### 1.3 지키는 원칙

- **접근 가능한 데이터 경로는 지금 쓰지 않아도 모두 설계하고 DS로 구현해 둔다.** 쓰지 않는 구현도 유지한다. 기능을 확장할 때는 기존 DS를 고치지 않고 유스케이스·출력 포트 구현에서 연결만 한다(개방-폐쇄). 그래서 이 문서는 "지금 필요한가"가 아니라 "접근할 수 있는가"를 기준으로 경로를 기록한다.
- 학교로 출석·진도·제출 같은 **쓰기 요청을 보내지 않는다.** 예외는 두 가지다.
  - 인증: 로그인, 로그아웃, 세션 갱신
  - **학생 개인 데이터**: Canvas 개인 캘린더 일정(`POST·PUT·DELETE /calendar_events`), 플래너 메모(`POST·PUT·DELETE /planner_notes`), 플래너 항목 완료 표시(`POST·PUT /planner/overrides`). 본인만 보는 데이터이고 출결·성적·제출에 영향을 주지 않는다. 사용자가 요청했을 때만 보낸다
  - 계속 금지: 출석·진도 조작, 과제·퀴즈 제출, 게시판·토론 글쓰기, 메시지 보내기 등 다른 사람에게 보이거나 학사에 영향을 주는 요청
- OTP를 우회하지 않는다. 학교 서버가 주는 로그인 유지 범위 안에서만 세션을 연장한다. OTP의 목적은 대리출석 방지다.
- 비밀(비밀번호, 쿠키, 토큰)은 OS 자격 증명 저장소에만 둔다. 응답 본문, URL 쿼리, 로그, 오류 메시지에 남기지 않는다.

---

## 2. 데이터 접근 아키텍처

### 2.1 레이어와 의존 방향

```
domain          엔티티(과목·차시·과제·공지…), 업무 규칙
application     유스케이스, 출력 포트 <I>(Repository·Gateway, 유스케이스 단위), 유스케이스 출력 DTO
                      ▲ 구현
data_access/
  repositories/ 출력 포트 구현(Repository·Gateway), DS <I>와 계약, 공통 오류 타입
                      ▲ 구현
  shared/       RequestAuthenticator <I> + (B)/(A) 구현,
                AuthGateway 구현(WebSessionLogin·SessionRefresher·SessionStore),
                세션 갱신 정책(순수 모듈), HTTP 정책
  canvas/  learningx/  smart_attendance/  commons/  school_site/  student_portal/
                DS 구현 (Source = 접근 구역별 출처 어댑터)
presentation    Tauri command(Controller), Presenter, ViewModel — 유스케이스만 호출한다
entrypoints     IPC 서버(`cdx` 요청)와 AI용 Presenter — 유스케이스만 호출한다
```

- 인터페이스는 **쓰는 쪽이 소유**한다. 출력 포트 `<I>`는 유스케이스가 쓰므로 application에, DS `<I>`는 출력 포트 구현이 쓰므로 `repositories/`에 둔다.
- 출력 포트 구현은 Source를 모른다. 어떤 DS 구현을 쓸지는 composition root(조립하는 곳)에서 정한다.
- domain은 data_access를 모른다. application은 data_access의 구체 클래스를 모른다.

### 2.2 출력 포트 `<I>` (application)

출력 포트(Repository·Gateway)의 정의와 이름 규칙은 [application §4.1](../02_application.md)에 있다. 이 문서는 그 구현만 다룬다.

- 출력 포트 구현은 domain 엔티티를 돌려준다. 외부 DTO를 내보내지 않는다.

### 2.3 출력 포트 구현 (data_access/repositories)

- 하는 일: DS를 조합하고, DS 결과(DTO)를 domain 모델로 바꾸고, 공통 오류를 application 오류로 바꾼다.
- 여러 출처를 합치는 Repository(예: "이번 주 할 일" = Canvas planner + LearningX 미완료 차시)는 DS 목록을 주입받아 합친다. 예: `TodoRepositoryImpl { sources: Vec<Box<dyn TodoDataSource>> }`. 출처가 늘어도 이 클래스는 고치지 않는다(개방-폐쇄).

### 2.4 DataSource `<I>`와 계약 (data_access/repositories)

- **리소스 묶음마다 인터페이스 하나, 접근 경로마다 메서드 하나**를 둔다. 예: `CourseModulesDataSource.list_modules(course_id)`, `.list_items(course_id, module_id)`
- 같은 `<I>`의 구현(예: REST 구현과 GraphQL 구현)은 서로 바꿔 끼울 수 있어야 한다(리스코프 치환). 이를 위해 계약을 `<I>`에 명시한다.

| 계약 | 내용 |
|---|---|
| 목록 완전성 | 목록 메서드는 항상 전체를 돌려준다. 페이지 처리(REST `Link` 헤더, GraphQL cursor)는 구현 안에서 끝낸다 |
| 필드 | DTO마다 필수 필드와 선택 필드를 정한다. 필수 필드가 없으면 `SourceChanged`를 던진다 |
| 오류 | 아래 공통 오류 타입만 던진다. HTTP 라이브러리 예외, 상태 코드, 응답 본문을 밖으로 내보내지 않는다 |
| 부작용 | 읽기 DS는 학교 서버의 상태를 바꾸지 않는다 |

**공통 오류 타입**

| 오류 | 의미 | 예 |
|---|---|---|
| `Unauthorized` | 인증이 거부됨(세션·토큰 만료 또는 삭제) | Canvas 401 + `WWW-Authenticate` 헤더 |
| `SessionExpired` | 갱신할 수 없음. 다시 로그인(OTP)해야 함 | 갱신 중 로그인 화면으로 이동 |
| `NotAvailable` | 권한이 없거나 그 과목에서 쓰지 않는 기능 | Canvas 401(헤더 없음), 404 "사용 안 함" |
| `SourceChanged` | 학교 쪽 형식이 예상과 다름. 실패한 단계 이름만 담는다 | 폼·필드·XML 요소를 찾지 못함 |
| `Network` | 연결 실패, 시간 초과, 5xx | — |
| `RateLimited` | 요청 제한에 걸림 | Canvas 403 throttled, 429 |

- 계약 준수는 **계약 테스트**로 확인한다. 같은 테스트 묶음을 `<I>`의 모든 구현에 돌린다.

### 2.5 shared — 인증과 HTTP 정책

책임을 나눠 둔다(단일 책임). 로그인·LTI 실행·토큰 교환·로그아웃·쿠키 저장을 한 클래스에 모으지 않는다.

| 구성 | 종류 | 책임 |
|---|---|---|
| `RequestAuthenticator` | `<I>` (shared 소유, Source가 사용. 여러 Source가 함께 쓰는 공통 계약이라 "쓰는 쪽이 소유"의 예외로 공용 crate에 둔다) | 요청에 인증을 붙인다. 인증이 거부되면 한 번 갱신하고 다시 시도한다 |
| `WebSessionAuthenticator` | 구현 (B) | 쿠키 + `X-CSRF-Token`(쓰기 요청), LearningX에는 `Authorization: Bearer <xn_api_token>` |
| `OAuthAuthenticator` | 구현 (A, 배포 전) | `Authorization: Bearer <access_token>`, 1시간 만료 시 refresh token으로 갱신 |
| `AuthGateway` | `<I>` (application 소유) | 로그인·로그아웃·상태 조회. `LoginWithOtp` 같은 유스케이스가 쓴다 |
| `WebSessionLogin` | `AuthGateway` 구현의 일부 | 학번·비밀번호·OTP → 세션 ([01_auth §2](01_auth.md)) |
| `SessionRefresher` | 내부 | 쿠키 → LTI → 새 `xn_api_token` ([01_auth §3](01_auth.md)) |
| `SessionStore` | `<I>` + 키체인 구현 | 쿠키·토큰·학번·비밀번호 저장. 바뀐 쿠키는 받는 즉시 저장한다. macOS는 **Data Protection 키체인(앱 접근 그룹)**에 저장한다. 로그인 키체인은 같은 사용자의 다른 프로세스가 허용 창 없이 읽을 수 있다([security §5.2](../../cross_cutting/01_security.md)). 저장소는 `access-group`을 명시하고 `cloud-sync`는 끈다. 항목은 `AfterFirstUnlockThisDeviceOnly`로 만든다(화면 잠금 중 갱신이 읽을 수 있게). 정책은 만들 때만 붙으므로, 정책이 다른 기존 항목은 지우고 다시 만든다. 실패는 `Locked`(-25308, 잠시 뒤 재시도), `Misconfigured`(-34018, 서명·entitlement 오류), `NotFound`(-25300)로 나눠 돌려주고 앞의 둘을 로그아웃으로 다루지 않는다. 시작 때 한 번 읽어 메모리에 두고, 값이 실제로 바뀔 때만 쓴다([키체인 측정](../../measurements/2026-11_keychain_prompts.md)) |
| 세션 갱신 정책 | shared 안의 순수 모듈 | 만료 임박, 유휴, 깨어남, 로그아웃 상태를 보고 갱신·대기·안내를 정한다 |
| HTTP 정책 | 공통 클라이언트 | 아래 표 |

- Commons는 인증이 필요 없으므로 `RequestAuthenticator`에 의존하지 않는다.
- OTP 입력은 사용자와 주고받는 흐름이므로 유스케이스(application)가 조율한다. shared는 한 번의 로그인 시도만 수행한다.
- 위 표는 Canvas·LearningX 세션(쿠키·`xn_api_token`) 기준이다. **학사행정웹서비스(별도 SSO `dsso`)와 스마트출결은 세션이 따로**다. 각각 `RequestAuthenticator`·`SessionStore` 구현을 하나씩 더 두고, 로그인 학번도 함께 보관한다(IDOR 방어에 쓴다, [07 §3.5](07_student_portal.md)). 포털은 2차 인증이 없고 60분 세션을 `sessionExtend.do`로 연장한다. 세션 만료 시 재로그인, 로그인 실패 잠김 처리는 그 구현에 둔다. 상세·미확인은 [07_student_portal](07_student_portal.md).

**HTTP 정책**

| 항목 | 규칙 |
|---|---|
| 허용 목록 | Source마다 호스트·메서드·경로 허용 목록을 둔다. 허용되지 않은 곳으로의 요청과 리디렉션은 `SourceChanged`로 막는다 |
| 리디렉션 | 직접 따라간다(최대 횟수 제한). 자격 증명을 담은 POST를 307/308로 다른 곳에 다시 보내지 않는다 |
| 응답 전처리 | Canvas JSON 앞의 `while(1);`를 제거한다 |
| 페이지네이션 | `Link` 헤더를 대소문자 구분 없이 읽고 `rel="next"`를 따라간다. 링크는 불투명한 값으로 다룬다 |
| 요청 제한 | 출처마다 동시 요청 1개를 기본으로 한다. `X-Rate-Limit-Remaining`을 기록하고, 제한에 걸리면 지수 백오프로 재시도한다 |
| 크기 제한 | XML·HTML 응답은 읽을 최대 크기를 둔다(Commons XML 1MB) |
| 기록 | 실패 시 고정된 단계 이름만 남긴다. URL 쿼리, 본문, 쿠키, 토큰을 기록하지 않는다 |

### 2.6 의존 규칙

각 항목을 Cargo crate로 나누고 `[dependencies]`로 강제한다. 의존 그래프 검사는 편집 직후 훅과 CI에서 돌린다([System Architecture §3.2](../../02_system_architecture.md)).

| 규칙 | 내용 |
|---|---|
| 레이어 | `presentation`·`entrypoints` → `application` → `domain`. data_access는 application·domain에 의존할 수 있고, 반대 방향은 금지 |
| 출력 포트 구현 | `data_access.repositories`는 Source(`canvas`, `learningx`, …)에 의존하지 않는다 |
| Source 독립 | Source끼리 서로 의존하지 않는다 |
| Source 의존 | Source는 `data_access.repositories`의 계약(DS `<I>`, DTO, 공통 오류)과 `data_access.shared`에만 의존한다 |
| 조립 | 구체 클래스를 조립하는 곳(composition root)만 모든 data_access crate에 의존할 수 있다 |

---

## 3. 접근 구역 문서

Source(접근 구역) 하나에 문서 하나를 둔다. 경로, 인증, 응답 형식, 구역별 위험·미확인 항목은 각 문서에 있다.

| 문서 | Source | 내용 |
|---|---|---|
| [01_auth](01_auth.md) | `shared` (인증) | SSO 로그인, OTP, Canvas 웹 세션, LearningX 토큰 갱신, 방식 A 전환 |
| [02_canvas](02_canvas.md) | `canvas` | REST 경로, GraphQL, LTI 도구 번호 찾기 |
| [03_learningx](03_learningx.md) | `learningx` | 주차학습, 출결, 수업자료실, 게시판, 도구별 학생 권한 |
| [04_smart_attendance](04_smart_attendance.md) | `smart_attendance` | 전자출결 |
| [05_commons](05_commons.md) | `commons` | 강의 PDF·동영상·첨부 원본 다운로드 |
| [06_school_site](06_school_site.md) | `school_site` | 학교 공지, 학과·기관 공지, 학과 게시판 목록 |
| [07_student_portal](07_student_portal.md) | `student_portal` | 학사행정웹서비스: 별도 로그인, 범용 조회 창구, 메뉴별 서비스 목록 |

새 출처를 더할 때는 Source와 문서를 함께 추가하고 이 표에 행을 더한다.

---

## 4. 공통 위험과 미확인 항목

구역 하나에 속하지 않는 것만 둔다. 구역별 항목은 각 문서 끝에 있다.

### 4.1 위험

| 위험 | 대응 |
|---|---|
| 방식 B의 Canvas API Policy 문제(D1·D2) | 배포 전 (A) 검토. 인증은 `RequestAuthenticator`로 교체 가능하게 둔다 |
| LearningX·SSO·Commons·학교 홈페이지는 공개 API가 아님 | 단계별 형식 검사와 `SourceChanged`. 실패 단계 이름으로 어디가 바뀌었는지 알 수 있게 한다 |
| 학생지원시스템·스마트출결이 요청 매개변수의 학번을 세션과 대조하지 않음(IDOR, 학교 측 결함, [07 §3.4](07_student_portal.md)) | 앱은 학번을 입력받지 않고 **세션 학번으로만** 요청을 만든다. 외부 학번은 덮어쓰고, 응답의 학번이 다르면 폐기한다. 학교에 별도 보고(사용자 진행) |

### 4.2 미확인 항목

- (없음. 구역별 문서 참조)

---

## 5. 관련 문서

| 문서 | 다루는 내용 |
|---|---|
| [PRD](../../01_prd.md) | 제품 정의, 범위, 데이터 출처 확장 방향 |
| [System Architecture](../../02_system_architecture.md) | 결정 요약, 프로세스 구조, 레이어 지도, 용어, 문서 색인 |
| [Application](../02_application.md) | 출력 포트 정의 |
| [Security](../../cross_cutting/01_security.md) | 레이어를 가로지르는 보안 경계 |
