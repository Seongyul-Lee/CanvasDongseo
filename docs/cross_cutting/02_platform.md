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
| 비밀 저장 | `keyring` + `apple-native-keyring-store`(`protected` 기능) | 4.2.0 (2026-08-29), 1.0.2 (2026-08-06) | data_access/shared | macOS는 Data Protection 키체인(앱 접근 그룹). 로그인 키체인은 쓰지 않는다([security §5.2](01_security.md)). `keychain` 기능은 켜지 않는다(로그인 키체인으로 돌아가지 않게). 접근 그룹은 저장소 설정 키 `access-group`으로 지정한다([키체인 측정](../measurements/2026-11_keychain_prompts.md)) |
| JSON | `serde`, `serde_json` | — | DTO·ViewModel·IPC 타입 | domain에는 쓰지 않는다 |
| HTML | `scraper` | 0.29.0 (2026-10-21) | data_access | 공지, SSO·포털 화면 |
| XML | `quick-xml` | 0.42.0 (2026-08-22) | data_access/commons | 엔티티를 펼치지 않음. 크기 상한·`DOCTYPE` 거부는 직접 검사 |
| 시간 | `jiff` | 0.2.40 (2026-11-07) | domain 포함 전체(domain의 유일한 외부 crate) | 시간대 있는 값으로 통일 |
| 오류·기록 | `thiserror`, `tracing` | — | domain 제외 전체 | 실패 단계 이름만 기록 |
| IPC | `interprocess` | 2.4.7 (2026-10-23) | entrypoints, bin/cdx | Unix 소켓·named pipe |
| CLI | `clap` | 4.6.7 (2026-09-14) | bin/cdx | `ToolGuide` 생성 |
| 테스트 | `cargo test`, `wiremock` | 0.6.5 (2025-08-24) | 전체 | DS 계약 테스트, 실측 응답 픽스처 |
| 의존성 검사 | `cargo-deny` | — | CI | 라이선스·취약점·금지 crate |
| 프런트엔드 | `react` + Vite (SPA) | 19.3.1 (2026-10-14) | presentation | 대화 UI 라이브러리가 풍부하고 AI 에이전트의 코드 정확도가 높다 |
| 학습 | `whisper-rs`(whisper.cpp 1.9.5로 실측), ffmpeg 9.0.2(LGPL 정적 빌드 사이드카), `objc2-vision`, `windows`(OCR) | [learning_adapters](../layers/06_learning_adapters.md) | learning | 확장 단계. 전사 모델 `large-v3-turbo-q5_0` + Silero VAD |

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

**학습 부품** (목록·크기·해시·확인 규칙은 [learning_adapters §4](../layers/06_learning_adapters.md))

- 설치 패키지에 넣지 않는다(모델 제공자 정책상 재배포 문제를 피한다).
- **설치 과정에서 사용자 조작 없이 자동으로 내려받는다.** 앱 안에 따로 설치하는 단계를 두지 않는다.
  - macOS: **`.pkg` 설치 프로그램**으로 배포하고, postinstall 스크립트가 부품을 받는다(2026-11-09 결정). DMG는 설치 단계가 없어 쓰지 않는다. `.pkg`는 Developer ID Installer 인증서로 서명하고 공증한다.
  - Windows: 설치 프로그램(NSIS 또는 MSI, 착수 때 선정)의 설치 단계에서 받는다.
- 받는 주소는 허용 목록(https)에 두고, 받은 직후 예상 크기와 고정 해시(실행 파일은 서명도)를 확인한다. 실패하면 반쯤 받은 파일을 지운다.
- **받기 절차**(postinstall은 root로 실행되므로 링크·임시 파일 공격을 막는다):
  1. `/tmp`처럼 다른 사용자가 쓸 수 있는 곳을 쓰지 않는다. 대상 경로 아래 root 전용(0700) 임시 폴더를 새로 만들고, 그 안에 받는다.
  2. 경로의 각 단계가 심볼릭 링크가 아닌지 확인하고(링크면 중단), 파일은 링크를 따라가지 않게(`O_NOFOLLOW`) 연다.
  3. 해시·서명을 확인한 뒤 같은 볼륨 안에서 원자적으로 이름을 바꿔 제자리에 둔다.
  4. 최종 권한: 폴더 root:wheel 0755, 파일 0644(실행 파일 0755). 사용자는 읽기만 한다.
- 위치: 설치 프로그램이 관리자 권한으로 쓰고 사용자는 읽기만 하는 공용 경로. macOS `/Library/Application Support/CanvasDongseo/components/<부품>/<버전>-<해시 앞 12자리>/`(postinstall은 root로 실행되므로 사용자별 앱 데이터에 쓰지 않는다). 사용자 프로세스(AI가 실행한 명령 포함)가 부품을 바꿔치기하지 못하게 하는 효과도 있다. 앱은 이 경로에 쓰지 않고 읽기만 한다.
- **받기와 다시 받기는 설치 프로그램만 한다.** 받기에 실패해도 설치는 끝내고, 앱이 `ComponentMissing`(복구: 설치 프로그램)을 받으면 화면은 "설치 프로그램을 다시 실행"을 안내한다. 앱은 부품을 스스로 받지 않는다.

**앱 식별자와 서명** (2026-11-08 등록)

| 항목 | 값 |
|---|---|
| 앱 이름(`productName`) | `CanvasDongseo` |
| 번들 ID | `io.github.seongyul-lee.canvasdongseo` (학교 도메인은 소유하지 않으므로 쓰지 않는다) |
| 팀 | `468HW42V9P` |
| App ID | `468HW42V9P.io.github.seongyul-lee.canvasdongseo` (Apple Developer 계정에 등록됨) |
| 키체인 접근 그룹 | `468HW42V9P.io.github.seongyul-lee.canvasdongseo` |
| 프로비저닝 | 개발: Mac App Development(Xcode 자동 관리). 배포: Developer ID 프로파일(배포 단계에서 생성). 키체인 접근 그룹 entitlement 때문에 앱 번들에 프로파일을 포함한다. 서명 인증서와 프로파일 종류가 맞아야 한다: Developer ID 서명에 개발 프로파일을 넣으면 앱이 실행 즉시 종료된다([키체인 측정 §3.2](../measurements/2026-11_keychain_prompts.md)). 서명 후 검사에서 인증서·프로파일·entitlement 조합을 확인한다 |
| 배포 빌드 조건 | Hardened Runtime 켬, `get-task-allow` 없음, 앱 샌드박스 끔(AI CLI 실행·로컬 소켓 사용) |

**기록·크래시 보고**: 로그와 크래시 보고에 비밀(쿠키·토큰·비밀번호)과 학교 데이터 본문을 남기지 않는다. 오류는 실패 단계 이름만 기록한다. 크래시 보고를 외부로 보내는 기능은 두지 않는다(두게 되면 이 규칙을 다시 검토한다).

## 4. 미확인 항목

- Windows 비밀 저장: Credential Manager는 같은 사용자 프로세스가 읽을 수 있을 것으로 예상한다. 대안(DPAPI 범위, 앱 전용 보호) 검토 필요
- Windows에서 Claude Code 공식 설치 스크립트의 실행 정책 처리
- macOS 서명·공증, 자동 업데이트 방식
- Windows 설치 프로그램 형식(NSIS·MSI)과 공용 부품 경로. 수용 기준: `%ProgramData%` 아래 폴더의 ACL이 Administrators·SYSTEM만 쓰기, Users는 읽기·실행만이며, 일반 사용자 프로세스의 쓰기·이름 바꾸기·삭제가 거부되는 것을 확인한다
- 자동 업데이트 때 부품 버전이 바뀌면 누가 받는지. 수용 기준: 부품을 받는 주체가 여전히 관리자 권한의 설치 프로그램뿐이고(앱은 받지 않음), 새 해시가 업데이트된 앱 안에 고정되어 있다
