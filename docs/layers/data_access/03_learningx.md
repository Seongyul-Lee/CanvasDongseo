# LearningX

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-11-08 확인(별도 표시가 없으면). 레거시 `CanvasDongseo-legacy` 문서·코드 |

> **이 문서의 역할**: LearningX API로 접근하는 데이터(주차학습·출결·수업자료실·게시판)와 도구별 학생 권한을 정의한다. Source `learningx`.
> 구조(레이어, `<I>`·DS 배치, 공통 오류, HTTP 정책)는 [README](README.md)를 따른다.

---

## 1. 공통

- 기준 주소: `https://canvas.dongseo.ac.kr/learningx/api/v1`
- 인증: `Authorization: Bearer <xn_api_token>`([01_auth](01_auth.md)). 공개 API가 아니라 학교가 LearningX를 업데이트하면 바뀔 수 있다.

## 2. 경로

| 데이터 | 경로 | 주요 필드 (레거시 기준) |
|---|---|---|
| 주차학습(차시 목록) | `GET /courses/{id}/modules?include_detail=true` | 항목 `content_type`(`attendance_item` = 차시), `content_data.week_position`, `lesson_position`, `item_id`, `use_attendance`, `unlock_at`, `due_at`, `late_at`, `lock_at`, `item_content_data.content_id`, `content_type`(PDF·VIDEO 등), `duration`, `total_file_size`, `updated_at`, `completed`, `completion_determinable` |
| 출결 | `GET /courses/{id}/attendance_items/summary` | `attendance_summaries[*].item_id`, `attendance_status`(`attendance`·`none` 등) |
| 수업자료실 | `GET /courses/{id}/resources` | `commons_content`(없으면 안내문만 있는 항목), `content_type`, `content_id`, `updated_at` |

**출결현황 도구(68)가 쓰는 경로** (2026-11-08 확인, 모두 `xn_api_token`으로 200)

| 데이터 | 경로 | 주요 필드 |
|---|---|---|
| 출결 대상 항목 | `GET /courses/{id}/attendance_items?include_detail=true` | `attendance_items[]`: `item_id`, `use_attendance`, `item_content_type`, `item_content_id`, `week_position`, `lesson_position`, `title`, `lecture_period_status`, `unlock_at`, `late_at`, `due_at`, `lock_at`, `item_content_data`, `opened` / `counts`(종류별 개수) |
| 출결 요약 | `GET /courses/{id}/attendance_items/summary?only_use_attendance=true` | §2 위 표와 같음 |
| 주차·차시 구성 | `GET /courses/{id}/lessons` | 주차별 `week_position`, `lessons[].lesson_position`, `attendance_calc_type`, 기간 |
| 차시별 출결 | `GET /courses/{id}/lessons/attendances` | `week_position`, `lesson_position`, `attendance_calc_type`, `attendance_status` |
| 과목 설정 | `GET /courses/{id}/settings?role=1` | 배속 제한 등 과목 설정. 필요할 때만 쓴다 |

- 출결현황 화면의 집계(총 학습요소, 출석·지각·결석·미결)는 위 경로로 만들 수 있다. `modules?include_detail=true`보다 출결에 맞춘 경로이므로, 출결 DS는 `attendance_items`와 `lessons/attendances`를 쓴다.

**게시판 도구(64)가 쓰는 경로**

| 데이터 | 경로 | 주요 필드 |
|---|---|---|
| 과목 게시판 목록 | `GET /learningx_board/courses/{id}/boards` | `id`, `slug`, `type`, `title`, `description`, 사용 기능(`use_reply`, `use_comment`, `use_notice`, `use_secret_post` …), `is_public` |

- 게시글 목록·본문 경로는 확인하지 못했다(확인한 과목의 게시판에 글이 없음).

- 차시 완료 여부는 `completion_determinable`이 참일 때만 `completed`를 믿는다.
- 같은 `content_id`가 여러 주차에 나올 수 있다(예: 운영체제 1주차 PDF).

## 3. 인증 관련 경로

[01_auth §2](01_auth.md)~3.4의 `/learningx/login`, `/learningx/google2fa/*`, `/learningx/api/v1/google2fa/authenticate`, `/learningx/lti/modulebuilder`.

## 4. LearningX 도구별 학생 권한 확인 결과

2026-11-08, 운영체제 과목(44947)에서 학생 계정으로 도구 화면을 열기만 하고(아무것도 누르지 않음) 화면 내용과 도구가 부르는 경로를 기록했다.

| 번호 | 이름 | 결과 | 데이터 접근 |
|---|---|---|---|
| 64 | Learningx Board (게시판) | ✅ 열림 | `xn_api_token`으로 게시판 목록 API 사용 가능(§2). 게시글 경로는 미확인 |
| 66 | Learningx Board Global | ⛔ 과목 홈으로 이동 | 과목에서 쓰지 않는 도구. 대상에서 뺀다 |
| 68 | Lecture Attendance (출결현황) | ✅ 열림 | `xn_api_token`으로 사용 가능(§2). **출결 DS의 주 경로** |
| 62 | Learning Activity | ⛔ "권한이 없습니다" | 학생 사용 불가 |
| 8 | LearnStatus | ⛔ "이 과목 유형에서는 지원하지 않습니다" | 이 과목에서는 사용 불가. 다른 유형 과목에서 다시 확인할 수 있다 |
| 15 | 스마트출결 (전자출결) | ✅ 열림 | 아래 [04_smart_attendance](04_smart_attendance.md). 별도 세션 필요 |
| 90 | AI 튜터 | ⛔ "AI 튜터가 지원되지 않는 과목" | 일부 과목만 제공. 지원 과목에서 다시 확인해야 한다 |

관리자용 도구(`/learningx/lti/admin/*`)는 대상이 아니다.


---

## 미확인 항목

- LearningX 게시판의 게시글 목록·본문 경로(§2, 글이 있는 게시판에서 확인)
- LearnStatus(8)와 AI 튜터(90)를 다른 유형·지원 과목에서 쓸 수 있는지(§4)
