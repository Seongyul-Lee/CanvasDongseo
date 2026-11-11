# Canvas

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-11-08 확인(별도 표시가 없으면), Instructure 개발자 문서 |

> **이 문서의 역할**: Canvas REST·GraphQL로 접근하는 데이터와 경로를 정의한다. Source `canvas`.
> 구조(레이어, `<I>`·DS 배치, 공통 오류, HTTP 정책)는 [README](README.md)를 따른다.

---

## 1. 공통

- 기준 주소: `https://canvas.dongseo.ac.kr/api/v1`, GraphQL `POST https://canvas.dongseo.ac.kr/api/graphql`
- 방식 B에서는 쿠키 세션으로 호출한다. JSON 앞의 `while(1);`를 제거한다. GraphQL과 쓰기 요청에는 `X-CSRF-Token`이 필요하다.
- 페이지네이션: 기본 10개, `per_page` 최대값은 정해지지 않았다(100으로 동작 확인). 항상 `Link` 헤더를 따른다.
- 요청 제한: 요청마다 `X-Request-Cost`가 차감되고 `X-Rate-Limit-Remaining`으로 남은 양을 알려 준다. 동시에 한 요청만 보내면 제한에 거의 걸리지 않는다.

## 2. REST 경로

상태: ✅ 200 확인, ⛔ 학생 권한으로 막힘·과목에서 사용 안 함 (2026-11-08 확인).

| 데이터 | 경로 | 상태 | 비고 |
|---|---|---|---|
| 내 정보 | `GET /users/self` | ✅ | 학번 없음 |
| 내 프로필(학번) | `GET /users/self/profile` | ✅ | `login_id`가 학번. 이름·학번 외 필드는 읽지 않는다 |
| 수강 등록 | `GET /users/self/enrollments` | ✅ | |
| 과목 목록 | `GET /courses?enrollment_state=active&include[]=term&per_page=100` | ✅ | 학기 정보 포함 |
| 과목 탭 | `GET /courses/{id}/tabs` | ✅ | LTI 도구 번호 찾기(§3) |
| 과목 LTI 도구 | `GET /courses/{id}/external_tools?include_parents=true` | ✅ | 학교 전체 도구 목록([03_learningx §4](03_learningx.md)) |
| 모듈 | `GET /courses/{id}/modules` | ✅ | Canvas 모듈. 주차학습 상세는 LearningX([03_learningx](03_learningx.md)) |
| 과제·내 제출 | `GET /courses/{id}/assignments?include[]=submission&per_page=100` | ✅ | `submission.attachments[].url`로 내 제출물 다운로드 |
| 과목 공지 | `GET /announcements?context_codes[]=course_{id}&start_date=…&end_date=…` | ✅ | **기간을 주지 않으면 최근 14일만 온다**(운영체제 과목: 기간 없이 0건, 학기 기간으로 6건) |
| 공지·토론 본문 | `GET /courses/{id}/discussion_topics/{topic_id}` | ✅ | 공지도 토론 주제로 조회된다 |
| 토론 글·댓글 | `GET /courses/{id}/discussion_topics/{topic_id}/view` | ✅ | `view`, `participants`, `unread_entries` |
| 과목 상세·강의계획서 | `GET /courses/{id}?include[]=syllabus_body&include[]=term` | ✅ | 강의계획서는 HTML |
| 과목별 성적(총점) | `GET /courses?enrollment_state=active&include[]=total_scores` | ✅ | `enrollments[].computed_current_score` 등. 교수자가 성적을 공개하지 않으면 `null` |
| 수강 등록별 성적 | `GET /users/self/enrollments?type[]=StudentEnrollment` | ✅ | `grades` |
| 과제 그룹·비중 | `GET /courses/{id}/assignment_groups?include[]=assignments` | ✅ | 성적 비중 계산 |
| 과제 상세 | `GET /courses/{id}/assignments/{assignment_id}` | ✅ | 설명(HTML), 배점, 채점 방식 |
| 내 제출 상세 | `GET /courses/{id}/assignments/{assignment_id}/submissions/self?include[]=submission_comments&include[]=rubric_assessment` | ✅ | 점수, 피드백 댓글, 루브릭 평가 |
| 동료 평가 | `GET /courses/{id}/assignments/{assignment_id}/peer_reviews` | ✅ | |
| 루브릭 | `GET /courses/{id}/rubrics` | ⛔ 401 | 과제 상세의 `rubric` 필드로 대신한다 |
| 모듈 항목 | `GET /courses/{id}/modules/{module_id}/items` | ✅ | |
| 캘린더 일정 | `GET /calendar_events?type=event&all_events=true&context_codes[]=course_{id}` | ✅ | |
| 캘린더 과제 | `GET /calendar_events?type=assignment&all_events=true&context_codes[]=course_{id}` | ✅ | `context_codes`가 없으면 0건 |
| 플래너 메모 | `GET /planner_notes` | ✅ | |
| 플래너 메모 쓰기 | `POST /planner_notes`, `PUT·DELETE /planner_notes/{id}` | ✅ (2026-11-12) | 개인 데이터 쓰기 예외([README §1.3](README.md)). 요청·응답은 §2.1 |
| 플래너 완료 표시 | `POST /planner/overrides`, `PUT /planner/overrides/{id}` | ✅ (2026-11-12) | 개인 데이터 쓰기 예외([README §1.3](README.md)). 해제는 `PUT`으로 된다. 요청·응답은 §2.1 |
| 개인 캘린더 일정 쓰기 | `POST /calendar_events`(`context_code=user_{id}`), `PUT·DELETE /calendar_events/{id}` | ✅ (2026-11-12) | 개인 데이터 쓰기 예외([README §1.3](README.md)). 과목 일정은 만들지 않는다. 요청·응답은 §2.1 |
| 메시지 상세 | `GET /conversations/{id}` | ✅ | |
| 안 읽은 메시지 수 | `GET /conversations/unread_count` | ✅ | |
| 활동 요약 | `GET /users/self/activity_stream/summary` | ✅ | 종류별 안 읽은 수 |
| 내 그룹 | `GET /users/self/groups` | ✅ | |
| 과목 그룹 | `GET /courses/{id}/groups` | ✅ | |
| 그룹 파일·구성원 | `GET /groups/{id}/files`, `GET /groups/{id}/users` | 확인 필요 | 그룹이 있는 과목에서 확인 |
| 과목 수강생 | `GET /courses/{id}/users` | ✅ | 이름만 |
| 과목 권한 | `GET /courses/{id}/permissions` | ✅ | 내 권한 확인용 |
| 할 일 | `GET /users/self/todo` | ✅ | |
| 플래너 | `GET /planner/items` | ✅ | 과제·공지·토론을 날짜순으로 한 번에 |
| 활동 스트림 | `GET /users/self/activity_stream` | ✅ | |
| 메시지 | `GET /conversations` | ✅ | |
| 토론 | `GET /courses/{id}/discussion_topics` | ✅ | |
| 과목 폴더 | `GET /courses/{id}/folders` | ✅ | |
| 내 파일 | `GET /users/self/files` | ✅ | |
| 과목 파일 | `GET /courses/{id}/files` | ⛔ 401 | 학생 권한 없음 |
| 페이지 | `GET /courses/{id}/pages` | ⛔ 404 | 확인한 과목에서 사용 안 함 |
| 퀴즈 | `GET /courses/{id}/quizzes` | ⛔ 404 | 확인한 과목에서 사용 안 함 |
| 첫 페이지 | `GET /courses/{id}/front_page` | ⛔ 404 | 설정 안 됨 |
| 분석 | `GET /courses/{id}/analytics/users/self/activity` | ⛔ 401 | |
| 학기 목록 | `GET /accounts/self/terms` | ⛔ 401 | 과목의 `include[]=term`으로 대신한다 |

- 페이지·퀴즈·과목 파일·첫 페이지는 확인한 과목에서 막혔지만 Canvas가 학생에게 여는 경로다. 과목마다 열릴 수 있으므로 DS는 구현해 두고 `NotAvailable`을 정상 결과로 다룬다.
- 404 "사용 안 함"은 과목마다 다를 수 있다. 막힌 경로는 `NotAvailable`로 다루고, 과목마다 다시 확인한다.

### 2.1 개인 데이터 쓰기의 요청과 응답 (2026-11-12 확인)

학생 본인 계정의 웹 세션으로, 시험용 개인 일정 1개와 플래너 메모 1개를 만들고 고치고 지웠다. 완료 표시는 그 시험용 메모에만 걸었다. 끝난 뒤 목록을 다시 읽어 시험용 항목이 남지 않은 것을 확인했다. 과목 일정과 과제에는 요청을 보내지 않았다.

- 쓰기 요청에는 세션 쿠키와 `X-CSRF-Token` 헤더(쿠키 `_csrf_token`을 URL 디코딩한 값)가 필요하다([01_auth §1](01_auth.md#1-자격-증명)). 본문은 JSON으로 보냈다.
- 여덟 경로 모두 학생 권한으로 된다. 만들기는 201, 고치기와 지우기는 200이다.
- **지우기는 항목을 없애지 않고 `workflow_state`를 `deleted`로 바꾼다.** 지우기의 응답도, 지운 뒤 `GET …/{id}`의 응답도 200에 `workflow_state: "deleted"`인 항목이다(404가 아니다). 목록 조회(`GET /planner_notes`, `GET /calendar_events`)에는 나오지 않았다.

| 요청 | 보낸 필드 | 결과 | 응답 필드 |
|---|---|---|---|
| `POST /calendar_events` | `calendar_event[context_code]`(`user_{id}`), `calendar_event[title]`, `calendar_event[start_at]`, `calendar_event[end_at]`, `calendar_event[description]` | 201. `workflow_state: "active"`, `type: "event"`, `all_day: false`, `hidden: false` | 아래 일정 필드 |
| `PUT /calendar_events/{id}` | `calendar_event[title]`, `calendar_event[start_at]`, `calendar_event[end_at]` | 200. 보낸 값으로 바뀐다 | 일정 필드 |
| `DELETE /calendar_events/{id}` | — | 200. `workflow_state: "deleted"` | 일정 필드 |
| `POST /planner_notes` | `title`, `details`, `todo_date` | 201. `workflow_state: "active"`, `course_id: null` | 아래 메모 필드 |
| `PUT /planner_notes/{id}` | `title`, `details`, `todo_date` | 200. 보낸 값으로 바뀐다 | 메모 필드 |
| `DELETE /planner_notes/{id}` | — | 200. `workflow_state: "deleted"` | 메모 필드 |
| `POST /planner/overrides` | `plannable_type`(`planner_note`), `plannable_id`, `marked_complete`(`true`) | 201. `marked_complete: true`, `dismissed: false`, `workflow_state: "active"` | 아래 완료 표시 필드 |
| `PUT /planner/overrides/{id}` | `marked_complete`(`false`) | 200. `marked_complete: false`. **완료 표시 해제가 `PUT`으로 된다** | 완료 표시 필드 |

- 일정 필드(24개): `id`, `title`, `start_at`, `end_at`, `workflow_state`, `created_at`, `updated_at`, `all_day`, `all_day_date`, `comments`, `location_address`, `location_name`, `type`, `description`, `child_events_count`, `all_context_codes`, `context_code`, `context_name`, `parent_event_id`, `hidden`, `child_events`, `url`, `html_url`, `duplicates`
- 메모 필드(9개): `id`, `todo_date`, `title`, `details`, `user_id`, `course_id`, `workflow_state`, `created_at`, `updated_at`
- 완료 표시 필드(11개): `id`, `plannable_type`, `plannable_id`, `user_id`, `workflow_state`, `marked_complete`, `deleted_at`, `created_at`, `updated_at`, `dismissed`, `assignment_id`
- 완료 표시는 `GET /planner/items`의 항목 안 `planner_override`로 읽힌다(필드는 위와 같다). 표시를 걸면 `planner_override.marked_complete`가 `true`, `PUT`으로 풀면 `false`가 됐다. 항목의 필드는 `plannable_id`, `plannable_type`, `plannable`, `plannable_date`, `planner_override`, `new_activity`, `submissions`다.
- `DELETE /planner/overrides/{id}`도 200(`workflow_state: "deleted"`)으로 된다. 시험용 표시를 지우는 데 썼다. 앱이 보내는 요청 목록([README §1.3](README.md))에는 없는 경로이고, 넣지 않는다(2026-11-12 결정). 앱은 완료 표시 해제를 `PUT`으로 한다.
- 확인하지 않은 것: 과제·퀴즈 같은 다른 항목의 완료 표시(`plannable_type`이 `planner_note`가 아닌 경우), 과목에 연결한 메모(`course_id`), 하루 종일 일정과 반복 일정, 잘못된 값을 보냈을 때의 오류 응답, 다른 사람이나 과목의 일정에 쓰려 할 때의 거부 응답.

## 3. LTI 도구 번호 찾기

도구 번호는 과목·학교 설정에 따라 다를 수 있으므로 코드에 고정하지 않고 과목 탭에서 찾는다.

- `GET /courses/{id}/tabs`의 `id`가 `context_external_tool_<번호>` 형식이다.
- 운영체제 과목(44947)의 탭(2026-11-08 확인):

| 탭 이름 | 도구 번호 | LTI 주소 |
|---|---|---|
| 게시판 | 64 | `/learningx/lti/learningx_board/boards` |
| 주차학습 | 70 | `/learningx/lti/modulebuilder` |
| 설문 | 78 | `/learningx/lti/survey/surveys` |
| 수업자료실 | 4 | `/learningx/lti/courseresource` |
| 출결현황 | 68 | `/learningx/lti/lecture_attendance/course_menu` |
| 전자출결 | 15 | `/smart-attendance/lti/attendances` |

- 탭 이름은 과목마다 바뀔 수 있으므로, 탭 이름이 아니라 `external_tools`의 LTI 주소(`url`)로 도구를 식별한다.

## 4. GraphQL

- `POST /api/graphql`이 방식 B 세션으로 동작한다(2026-11-08, `allCourses` 조회 확인).
- 권한은 REST와 같다. REST의 모든 기능이 있지는 않다.
- `id`는 전역 ID이고, REST의 숫자 ID는 `_id`다.
- 스키마는 `https://canvas.dongseo.ac.kr/graphiql`에서 볼 수 있다.

**스키마 (2026-11-08 introspection)**

학교 Canvas의 GraphQL 스키마는 범위가 좁다.

| 타입 | 있는 것 | 없는 것 |
|---|---|---|
| `Query` | `allCourses`, `course(id)`, `assignment`, `submission`, `term`, `moduleItem`, `node`, `legacyNode` | 공지, 플래너, 할 일, 메시지 |
| `Course` | `term`, `assignmentsConnection`, `modulesConnection`, `submissionsConnection`, `externalToolsConnection`, `enrollmentsConnection` | 공지, 토론 목록, 파일 |
| `Assignment` | `dueAt`, `lockAt`, `unlockAt`, `submissionTypes`, `htmlUrl`, `submissionsConnection` | — |
| `Submission` | `state`, `submittedAt`, `late`, `missing`, `attachments`, `score`, `grade` | — |
| `Discussion` | `_id`, `createdAt`, `modules` | 제목·본문 |

**REST와 비교 (2026-11-08, 활성 과목 12개, 과제 61개)**

| 방법 | 요청 수 | 걸린 시간 (3회) | 비고 |
|---|---|---|---|
| REST `courses/{id}/assignments?include[]=submission` 과목마다 | 12 | 2237 / 2563 / 2764 ms | 순차 요청 |
| GraphQL `course(id:)` 12개를 별칭으로 묶어 한 번에 | 1 | 493 / 432 / 508 ms | `X-Request-Cost` 0.33 |
| GraphQL `allCourses` 하나로 | 1 | 7205 ms (1회) | 지난 학기 포함 56과목을 필터 없이 돌려줘 오히려 느림. 쓰지 않는다 |

- 과제 61개의 ID·마감·제출 시각은 REST와 일치했다.
- **형식 차이** (DS 계약에서 맞춰야 함):
  - 시각: REST는 UTC(`…Z`), GraphQL은 `+09:00`. DS는 시간대가 있는 시각으로 바꿔 넘긴다.
  - 미제출: REST는 `submission.workflow_state = "unsubmitted"`, GraphQL은 제출 노드가 비어 있음. DS는 둘 다 "미제출"로 바꾼다.

**결정**

| 데이터 | 방법 | 이유 |
|---|---|---|
| 과목 목록 + 학기 | REST `courses?enrollment_state=active&include[]=term` | 한 번이면 되고, GraphQL `allCourses`는 활성 과목만 거를 수 없다 |
| 과목별 과제 + 내 제출 | **GraphQL** (활성 과목 ID를 별칭으로 묶음) | 요청 수가 과목 수에서 1로 줄고, 약 5배 빠르다 |
| 과목 모듈 | GraphQL 묶음에 포함할 수 있음 | 필요할 때 같은 쿼리에 더한다 |
| 공지, 할 일, 플래너, 메시지, 토론 | REST | GraphQL 스키마에 없다 |

- 과제 DS `<I>`는 REST와 GraphQL 구현을 모두 두고, 같은 계약 테스트를 돌린다([README §2.4](README.md)). GraphQL이 실패하거나 학교가 스키마를 바꾸면 REST 구현으로 바꿔 끼울 수 있다.
- 별칭으로 묶는 과목 수가 많아질 때의 비용 상한은 확인하지 않았다. 묶음 크기를 정해 두고(예: 20과목) 나눠 보낸다.

---

## 미확인 항목

- GraphQL 별칭 묶음의 크기 상한과 요청 비용(§4)
