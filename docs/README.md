# CanvasDongseo 문서

동서대학교 학생의 학교 데이터를 아는 AI 비서 데스크톱 앱(Tauri 2 + Rust + React)의 기획·설계 문서다.

## 읽는 순서

| 문서 | 내용 |
|---|---|
| [01_prd](01_prd.md) | 제품 정의, 문제, 범위, 확장 방향, 제약·위험 |
| [02_system_architecture](02_system_architecture.md) | 결정 요약, 프로세스 구조, 레이어 지도, 용어, 전체 문서 색인 |
| [03_design_process](03_design_process.md) | 디자인 단계와 산출물 |
| [04_apple_developer_account](04_apple_developer_account.md) | Apple 계정·Developer Program, 인증서, 프로파일, 번들 ID, 공증 자격 증명 현황 |
| [05_scrum_board](05_scrum_board.md) | 스프린트 운영 규칙, GitHub Projects 필드·View, 이슈 템플릿 |
| [06_ux_strategy](06_ux_strategy.md) | Actor·시나리오, 경험 원칙, 기준 디자인(T3 Code), 화면 구성, 문구 원칙 |

## 폴더

| 폴더 | 내용 |
|---|---|
| [layers/](layers/) | 레이어별 문서(domain, application, presentation, entrypoints, data_access, ai_adapters, learning_adapters) |
| [specs/](specs/README.md) | 기능별 Functional Spec과 User Flow(`FS-NN_<기능>.md`) |
| [cross_cutting/](cross_cutting/) | 레이어를 가로지르는 주제(security, platform) |
| [measurements/](measurements/) | 설계 결정의 근거가 된 실측 보고서 |

## 쓰는 규칙

- 한 가지 사실은 한 문서에만 적고, 다른 곳에서는 링크한다.
- 레이어 문서는 개요(링, crate, 의존) → 주요 구성 → 이웃 레이어와의 계약 → 미확인 항목 순서로 쓴다.
- 새 문서를 더하면 [02_system_architecture §5](02_system_architecture.md#5-문서-색인)의 색인에 행을 더한다.
