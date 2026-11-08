# Domain

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../01_system_architecture.md), [PRD](../00_prd.md) |

> **이 문서의 역할**: domain 레이어의 엔티티와 업무 규칙을 정한다. 구체 목록은 Functional Spec과 유스케이스를 정할 때 채운다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Entities |
| crate | `domain` |
| 의존할 수 있는 것 | 기초 crate만. 외부 crate는 `jiff`(시각 값 타입)만 쓴다 |

## 2. 원칙

- UI·프레임워크·HTTP·키체인·AI·OS API를 모른다.
- **직렬화를 하지 않는다.** `serde`를 쓰지 않는다. 직렬화는 DS DTO, 출력 DTO, ViewModel, IPC 타입이 맡는다.
- 학교·학습에 관한 업무 규칙만 둔다. 학교 세션 갱신 시점 같은 인프라 정책은 [data_access/shared](05_data_access/README.md)에 둔다.
- 순수 함수와 값 타입으로 쓰고, 단위 테스트로 고정한다.

## 3. 엔티티 (후보)

| 엔티티 | 출처 | 비고 |
|---|---|---|
| 과목, 학기 | Canvas | |
| 주차·차시, 학습 자료 | LearningX, Commons | 같은 `content_id`가 여러 주차에 나올 수 있다 |
| 과제, 제출 | Canvas | 미제출 표현은 REST·GraphQL 차이를 DS에서 맞춘 뒤 들어온다 |
| 출결 | LearningX, 스마트출결, 학생지원시스템 | 최종 기준은 학생지원시스템 |
| 공지 | Canvas, LearningX, 학교·학과 홈페이지 | |
| 학적·성적·졸업 요건 | 학생지원시스템 | |

## 4. 업무 규칙 (후보)

- 이번 주 할 일 판정(마감, 미완료 차시, 미제출 과제)
- 차시 완료 판정(`completion_determinable`이 참일 때만 `completed`를 믿는다)
- 출결 집계(출석·지각·결석·미결)

## 5. 이웃 레이어와의 계약

- application은 엔티티와 규칙을 쓰고, 바깥으로는 출력 DTO로 바꿔 내보낸다(S10).
- 출력 포트 구현(data_access)이 DS DTO를 엔티티로 바꿔 넘긴다.

## 6. 미확인 항목

- 엔티티·규칙의 확정 목록(Functional Spec 이후)
