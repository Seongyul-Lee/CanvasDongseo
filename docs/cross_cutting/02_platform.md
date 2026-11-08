# Platform

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../02_system_architecture.md) S1, 2026-11-08 crates.io 조회 |

> **이 문서의 역할**: 기술 스택(crate·버전), OS별 구성, 설치·배포를 정한다. 각 crate를 어느 레이어에서 어떻게 쓰는지는 레이어 문서에 있다.

---

## 1. 기술 스택

버전은 2026-11-08 crates.io 기준이다. 착수할 때 다시 확인한다.

| 영역 | 선택 | 버전(최근 갱신일) | 쓰는 레이어 | 비고 |
|---|---|---|---|---|
| 셸 | `tauri` | 2.12.5 (2026-11-06) | presentation, bin/app | capabilities로 command 제한 |
| 비동기 | `tokio` | — | 전체(domain 제외) | 갱신 스케줄러, 스트리밍 다운로드, 자식 프로세스 |
| HTTP | `reqwest` (rustls) | 0.13.5 (2026-09-08) | data_access | 리디렉션 정책은 끄고 직접 따른다. 쿠키 저장소는 직접 관리 |
| 인증 보조 | `rsa` | 0.9.10 (2026-11-07) | data_access/shared | LearningX 로그인의 PKCS#1 v1.5 복호화 |
| 비밀 저장 | `keyring` | 4.2.0 (2026-08-29) | data_access/shared | 메이저 4라 API 확인 필요 |
| JSON | `serde`, `serde_json` | — | DTO·ViewModel·IPC 타입 | domain에는 쓰지 않는다 |
| HTML | `scraper` | 0.29.0 (2026-10-21) | data_access | 공지, SSO·포털 화면 |
| XML | `quick-xml` | 0.42.0 (2026-08-22) | data_access/commons | 엔티티를 펼치지 않음. 크기 상한·`DOCTYPE` 거부는 직접 검사 |
| 시간 | `jiff` | 0.2.40 (2026-11-07) | domain 포함 전체 | 시간대 있는 값으로 통일 |
| 오류·기록 | `thiserror`, `tracing` | — | 전체 | 실패 단계 이름만 기록 |
| IPC | `interprocess` | 2.4.7 (2026-10-23) | entrypoints, bin/cdx | Unix 소켓·named pipe |
| CLI | `clap` | 4.6.7 (2026-09-14) | bin/cdx | `ToolGuide` 생성 |
| 테스트 | `cargo test`, `wiremock` | 0.6.5 (2025-08-24) | 전체 | DS 계약 테스트, 실측 응답 픽스처 |
| 의존성 검사 | `cargo-deny` | — | CI | 라이선스·취약점·금지 crate |
| 프런트엔드 | `react` + Vite (SPA) | 19.3.1 (2026-11-06) | presentation | 대화 UI 라이브러리가 풍부하고 AI 에이전트의 코드 정확도가 높다 |
| 학습 | `whisper-rs`, `objc2-vision`, ffmpeg | [learning_adapters](../layers/06_learning_adapters.md) | learning | 확장 단계 |

- GraphQL은 전용 클라이언트 없이 JSON POST로 보낸다([02_canvas §4](../layers/data_access/02_canvas.md)).
- 레지스트리(학과 게시판 코드, 포털 조회 허용 목록·학번 매개변수)는 앱에 포함하는 데이터 파일(TOML)로 두고, 읽을 때 타입으로 검증한다.

## 2. OS별 구성

| 항목 | macOS (먼저) | Windows (다음) |
|---|---|---|
| 웹뷰 | WKWebView | WebView2 |
| 비밀 저장 | Keychain | Credential Manager |
| IPC | Unix 소켓 | named pipe |
| 학습 기능 | [learning_adapters](../layers/06_learning_adapters.md) | 같음 |
| AI CLI 설치 | 공식 셸 스크립트 | 공식 PowerShell 스크립트(실행 정책 확인 필요) |

## 3. 설치·배포

- macOS를 먼저 개발·배포하고 Windows를 잇는다([PRD §7.1](../01_prd.md)).
- AI 제공자 CLI는 재배포하지 않고, 사용자 동의 뒤 공식 설치 프로그램을 실행한다.
- 서명·공증, 자동 업데이트, 설치 패키지 형식은 착수 때 정한다.

## 4. 미확인 항목

- `keyring` 4.x API와 Windows 동작
- Windows에서 Claude Code 공식 설치 스크립트의 실행 정책 처리(레거시 Q-S2)
- macOS 서명·공증, 자동 업데이트 방식
