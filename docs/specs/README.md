# Functional Spec

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD](../01_prd.md), [UX Strategy](../06_ux_strategy.md), [application](../layers/02_application.md)과 레이어별 문서 |

> **이 폴더의 역할**: 기능마다 *정확히 무엇을 하고*, *입력·출력·예외는 무엇이며*, *언제 성공으로 보는지*(수용 기준)를 정한다. 기능의 단계와 분기(User Flow)도 같은 문서에 둔다.
> 왜 만드는가는 [PRD](../01_prd.md), 어떤 경험과 화면인가는 [UX Strategy](../06_ux_strategy.md), 코드에서 어디에 두는가는 [레이어 문서](../02_system_architecture.md#5-문서-색인)가 맡는다.

---

## 1. 규칙

- **기능 하나에 문서 하나**를 둔다. 파일 이름은 `FS-NN_<기능>.md`이고, 번호는 두 자리다.
- FS 번호는 백로그 이슈, PR, QA 시나리오가 참조하므로 바꾸지 않는다. 없애면 번호를 비워 두고, 더하면 다음 번호를 쓴다.
- 모든 FS는 [UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)의 시나리오(S…)에 연결한다. 시나리오가 없는 FS는 근거를 한 줄로 적는다(예: 법적 고지 의무).
- 범위는 UX Strategy와 같은 규칙을 따른다. **아키텍처에 없는 기능을 FS가 새로 만들지 않는다.** 필요한 것이 아키텍처에 없으면 [UX Strategy §12](../06_ux_strategy.md#12-아키텍처에-올릴-것)에 올린다.
- 다른 문서가 정한 값(오류 종류, 세션 수명, 화면 구성 등)은 옮겨 적지 않고 링크한다. FS에는 그 값으로 기능이 *어떻게 동작해야 하는지*만 쓴다.
- FS를 확정하면 [application §3](../layers/02_application.md#3-유스케이스-후보)의 유스케이스·출력 DTO와 [domain](../layers/01_domain.md)의 엔티티·규칙을 그에 맞춰 확정한다.

## 2. 문서 형식

```
# FS-NN 기능 이름

| 항목 | 내용 |   ← 상태(초안·확정), 시나리오, 화면, 유스케이스, 근거
> 이 문서의 역할

## 1. 범위          하는 것 / 하지 않는 것
## 2. 기능 요구사항  F1, F2 … — 동작, 입력, 출력, 규칙
## 3. 예외           상황 → 판정 → application 오류 → 다음 행동
## 4. 수용 기준      AC1, AC2 … — 확인할 수 있는 문장, 확인 방법, 연결 시나리오
## 5. User Flow      UF-NN — Design Process ①에서 Mermaid로 그린다
## 6. 미확인 항목
```

- 요구사항과 수용 기준의 번호는 문서 안에서만 쓴다. 다른 곳에서는 `FS-03 F2`, `FS-03 AC4`처럼 FS 번호를 붙여 부른다.
- 수용 기준은 [Design Process ⑥ 시나리오 점검](../03_design_process.md)과 백로그 이슈의 "끝났다고 보는 기준"이 그대로 가져다 쓸 수 있게 쓴다. 실제 학교 계정·서버나 유료 Claude 계정이 있어야 확인할 수 있는 기준은 수용 기준 표의 시나리오 칸 끝에 ★을 붙이고, 표 아래 각주에 무엇이 필요한지 적는다.
- **User Flow는 `UF-NN`이고 FS 번호와 같다.** FS 초안 단계에서는 비워 두고, 그 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 성공 경로와 함께 §3 예외의 실패·중단 갈래를 모두 그린다.

## 3. 목록

| FS | 기능 | 시나리오 | 상태 |
|---|---|---|---|
| [FS-01](FS-01_onboarding.md) | 온보딩 | S1-1, S1-3, S1-4, S1-7 | 초안 |
| [FS-02](FS-02_ai_connection.md) | AI 연결 | S1-1~S1-3, S2-4, S2-5, S10-3 | 초안 |
| [FS-03](FS-03_school_account.md) | 학교 계정 연결 | S1-4~S1-6, S2-1, S2-2, S10-3 | 초안 |
| [FS-04](FS-04_student_portal_login.md) | 학생지원시스템 로그인 | S2-3, S6-1 | 초안 |
| [FS-05](FS-05_departments.md) | 학과 설정 | S1-7, S5-3 | 초안 |
| [FS-06](FS-06_connection_status.md) | 연결 상태와 출처 장애 | S2-2, S2-7 | 초안 |
| [FS-07](FS-07_conversation.md) | 대화 | S9-1~S9-3, S10-3 | 초안 |
| [FS-08](FS-08_todo_and_courses.md) | 할 일·과목 조회 | S3-1, S3-2, S4-1 | 초안 |
| [FS-09](FS-09_notices_and_peer_safety.md) | 공지 조회와 `Peer` 안전 | S4-3, S5-1~S5-3 | 초안 |
| [FS-10](FS-10_academic_records.md) | 학적·성적·출결 조회 | S6-1, S6-2 | 초안 |
| [FS-11](FS-11_personal_schedule_write.md) | 개인 일정 쓰기 | S8-1 | 초안 |
| [FS-12](FS-12_material_download.md) | 자료·동영상 내려받기 | S4-2, S7-4, S10-3 | 초안 |
| [FS-13](FS-13_slide_pdf_and_script_jobs.md) | 슬라이드 PDF·스크립트 작업 | S7-1, S7-2, S2-6 | 초안 |
| [FS-14](FS-14_study_with_script.md) | 스크립트로 공부하기 | S7-3 | 초안 |
| [FS-15](FS-15_settings_policy_about.md) | 설정·정책·앱 정보 | S10-1, S10-2 | 초안 |

- FS-01 온보딩은 FS-02·FS-03·FS-05를 순서대로 잇는 흐름이다. 각 단계의 동작은 해당 FS가 정하고, FS-01은 순서·건너뛰기·재개만 정한다.
- FS-06은 FS-02·FS-03·FS-04의 상태를 한 곳에서 판정해 연결 알약·설정 행·온보딩 카드·대화 안내에 쓴다([presentation §1](../layers/03_presentation.md#1-개요) `presenters/shell`).
- 위 표는 [UX Strategy §11](../06_ux_strategy.md#11-대응표)의 행을 기능 단위로 묶은 것이다. 대응표에 행이 늘면 이 표도 확인한다.
