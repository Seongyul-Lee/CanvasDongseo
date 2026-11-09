# 스마트출결 (전자출결)

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-11-08 확인(별도 표시가 없으면) |

> **이 문서의 역할**: LearningX와 별도인 스마트출결 서비스(LTI 도구 15)의 접근 경로를 정의한다. Source `smart_attendance`.
> 구조(레이어, `<I>`·DS 배치, 공통 오류, HTTP 정책)는 [README](README.md)를 따른다.

---

## 1. 접근 경로

LearningX와 다른 서비스(`/smart-attendance/*`)이고 인증 방식도 다르다.

| 단계 | 요청 | 결과 |
|---|---|---|
| 1 | 쿠키로 `GET /courses/{id}/external_tools/15` → LTI 폼 | [01_auth §3](01_auth.md) 단계 1과 같은 방식 |
| 2 | 폼을 `POST /smart-attendance/lti/attendances` | 스마트출결 자체 세션(쿠키) |
| 3 | `GET /smart-attendance/courses/{id}/attendances?locale=ko` | 화면 HTML. 여기서 CSRF 값을 얻는다 |
| 4 | `GET /smart-attendance/api/v1/attendances?user_login=<학번>&user_name=<이름>&course_id={id}&includes[]=attendees_count` | 수업 일시별 출결(출석·지각·결석) |
| 5 | `GET /smart-attendance/api/v1/reserved_seats?user_login=<학번>&course_id={id}` | 지정 좌석 |

- API는 쿠키 세션과 `X-CSRF-TOKEN`·`X-XSRF-TOKEN` 헤더로 인증한다. `xn_api_token`을 쓰지 않는다. 그래서 `auth`에 스마트출결용 세션(`RequestAuthenticator`·`SessionPrincipal`·`SessionStore` 구현)을 따로 둔다([README §2.5](README.md#25-http--auth--인증과-http-정책)).
- 쿼리에 학번(`user_login`)·이름(`user_name`)을 넣어 보낸다. **학생지원시스템과 같은 IDOR 구조로 의심된다**([07 §3.4](07_student_portal.md)에서 매개변수 학번 미검증 확인). 학번 매개변수 규칙은 [07 §3.5](07_student_portal.md#35-앱이-지킬-것)를 따른다. 두 값은 스마트출결 세션의 `SessionPrincipal`에서 읽고, 응답 대조는 `smart_attendance` Source가 한다. 실존 타인 학번으로 추가 확인은 하지 않는다.
- 화면 안내에 따르면 전자출결은 생성 다음 날 새벽 5시에 학사행정서비스로 옮겨진다. 최종 출결의 기준은 학사행정서비스다([07_student_portal](07_student_portal.md)에서 확인).
- 읽기만 한다. 출결 등록 요청은 보내지 않는다([README §1.3](README.md)).

---

## 미확인 항목

- 스마트출결 세션의 유효 시간과 갱신 방법
- 출결·좌석 응답에 학번 필드가 있는지(응답 대조에 쓸 필드). 없으면 1차 방어(세션 학번만 주입)만 남는다
- `user_name`에 넣을 이름의 출처(LTI 폼 필드 또는 Canvas 프로필). 스마트출결 세션 구현이 로그인 때 얻어 `SessionPrincipal`로 내준다
