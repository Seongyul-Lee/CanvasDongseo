# Build · CI

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../02_system_architecture.md) S2·S11, [Design Process](../03_design_process.md), [PR 템플릿](../../.github/pull_request_template.md) |

> **이 문서의 역할**: crate를 나누는 기준, 로컬 빌드 설정, 제네릭 사용 정책, CI에서 바뀐 crate만 빌드·테스트하는 방법을 정한다. crate 목록과 의존 규칙 자체는 [System Architecture §3](../02_system_architecture.md#3-레이어-지도)에 있다.

---

## 1. 왜 crate 경계가 빌드 문제인가

- Rust는 crate가 컴파일 단위다. 한 crate가 바뀌면 그 crate와 **그 crate에 의존하는 모든 crate**가 다시 컴파일된다. 증분 컴파일은 crate 안의 일을 줄일 뿐, 의존하는 crate를 다시 부르는 것은 막지 못한다.
- 코드가 커질수록 전체 빌드 시간은 선형보다 빠르게 는다. 개발 중 체감은 "한 번 고칠 때 몇 개 crate가 다시 빌드되는가"로 정해진다.
- 그래서 **자주 바뀌는 코드를 많이 의존받는 crate에 두지 않는다**(안정된 쪽으로 의존한다, Stable Dependencies Principle). 단일 책임 원칙의 Actor 기준과 같은 경계가 나온다: 바뀌는 이유(Actor)가 다른 코드는 바뀌는 때도 다르다.
- 이 앱은 Tauri 번들 하나로 배포된다. 따로 배포되는 단위는 앱 번들, `cdx`, ffmpeg 사이드카, 모델 부품뿐이다. 따라서 crate 분할의 이점은 **독립 빌드·테스트**다.

## 2. crate를 나누는 기준

### 2.1 Actor

| Actor | 바뀌는 이유 | 해당 crate 묶음 |
|---|---|---|
| 학교 데이터 | 학교 시스템 형식 변경, 새 출처, 조회 기능 | `ports/school`, `usecases/school`, data_access 전부, `presenters/school` |
| AI 제공자 | 제공자 CLI 변경, 새 제공자, 대화 기능 | `ports/ai`, `usecases/ai`, `ai/*`, `local_store/conversations`, `presenters/ai` |
| 학습 기능 | 처리 방식·엔진 변경, 슬라이드 판정 기준 조정 | `domain/slides`, `ports/learning`, `usecases/learning`, `learning/*`, `local_store/learning`, `presenters/learning` |
| 화면 | 기능 단위 디자인([Design Process §1](../03_design_process.md#1-흐름)) | `presenters/*`, `tauri_commands`, `ui/` |
| AI 연동 통로 | `cdx` 명령·프로토콜 | `ipc/wire`, `ipc/cli`, `ipc_server`, `bin/cdx` |

### 2.2 나누는 경우와 나누지 않는 경우

- **나눈다**: Actor가 다르거나, 같은 Actor 안에서도 변경 빈도가 크게 다를 때. 대표적인 경우가 **포트(안정)와 유스케이스(자주 바뀜)**다. 어댑터는 포트만 필요하므로 유스케이스를 고쳐도 어댑터가 다시 빌드되지 않아야 한다.
- **나누지 않는다**: Actor와 변경 빈도가 같은데 파일 수만 많을 때. crate마다 rustc 호출·링크 비용이 붙는다. 크기만으로는 나누지 않는다.
- 새 crate를 만들면 [System Architecture §3.1·§3.2](../02_system_architecture.md#3-레이어-지도)에 행을 더하고, §5의 영향 표가 달라지면 고친다.

### 2.3 변경 하나가 다시 빌드시키는 범위 (목표)

설계 기준의 예상치다. 구현 후 `cargo build --timings`로 확인한다(§6).

| 바뀐 곳 | 다시 빌드되는 crate |
|---|---|
| 유스케이스 하나(예: `usecases/school`) | 그 crate, 같은 묶음 `presenters`, `tauri_commands`, `ipc_server`, `bin/app` |
| 출력 DTO 필드 하나 | 위와 같다(출력 DTO는 유스케이스 crate에 있다) |
| Source 하나(예: `canvas`) | 그 crate, `bin/app` |
| DS 계약 하나(예: `contracts/notice`) | 그 crate, 그 계약을 구현하는 Source, `repositories`, `bin/app` |
| `domain/slides` 기준값 | 그 crate, `usecases/learning`과 그 위(`presenters/learning`, `tauri_commands`, `ipc_server`, `bin/app`) |
| AI 제공자 어댑터 | 그 crate, `bin/app` |
| `ipc/cli`(명령 설명) | 그 crate, `bin/cdx`, `bin/app` |
| 포트(예: `ports/learning`) | 그 포트를 쓰는 유스케이스와 구현 어댑터, 그 위. 포트는 드물게 바뀐다 |
| `application/kernel`, `domain/core` | 거의 전부. 그래서 이 두 crate에는 안정된 것만 둔다 |

## 3. 로컬 개발

| 항목 | 규칙 |
|---|---|
| 편집 직후 훅 | `cargo check -p <바뀐 crate>`(전체 빌드 대신). 의존 그래프 검사(§5.4)도 여기서 돈다 |
| 테스트 | `cargo nextest run -p <바뀐 crate>`. 역의존 crate까지 돌릴 때는 `cargo xtask affected`(§5.2)를 쓴다 |
| 개발 프로필 | `[profile.dev] debug = "line-tables-only"`. 의존 crate는 `[profile.dev.package."*"] opt-level = 1`처럼 최적화해 실행 속도를 얻는다(착수 때 측정해 정함) |
| 학습 기능 끄기 | `bin/app`에 `learning` cargo feature(기본 켬)를 둔다. 개발 중 학습 기능을 만지지 않을 때는 `--no-default-features`로 whisper.cpp·ffmpeg 관련 crate를 빌드·링크하지 않는다. **배포 빌드는 항상 기본 feature로 만든다** |
| 링커 | macOS는 Xcode 기본 링커(ld-prime)를 쓴다. Linux CI는 툴체인 기본(x86_64는 rust-lld)을 쓴다. 링크 시간이 문제로 측정되면 다시 본다 |
| 통합 테스트 | crate마다 `tests/main.rs` 하나에 모듈로 모은다. 파일마다 테스트 바이너리가 생겨 링크가 늘어나는 것을 막는다 |
| 의존 버전 | `[workspace.dependencies]`에서 버전과 feature를 한 번에 정하고, 각 crate는 `workspace = true`로 쓴다 |

## 4. 제네릭과 동적 디스패치

- **포트는 `Arc<dyn Port>`로 주입한다.** 유스케이스를 포트 타입에 대한 제네릭(`fn run<R: CourseRepository>`)으로 쓰지 않는다. 제네릭이면 단형화가 모두 `bin/app`에서 일어나 composition root가 무거워지고, 유스케이스를 고칠 때마다 다시 생성된다.
- 이미 정한 `Vec<Box<dyn TodoDataSource>>`([data_access §2.3](../layers/data_access/README.md#23-출력-포트-구현-data_accessrepositories))와 같은 방향이다.
- 제네릭은 crate 안에서 끝나는 작은 도우미(파싱, 변환)에만 쓴다. 공개 API에 제네릭을 두면 그 단형화는 호출하는 crate에서 일어난다는 점을 고려한다.
- 비동기 포트: 포트는 dyn으로 써야 하므로 trait의 `async fn`을 그대로 쓸 수 있는지는 착수 때 툴체인에서 확인한다. 안 되면 `application/kernel`에 `BoxFuture`·`BoxStream` 별칭을 두고 포트가 이를 돌려준다(proc-macro 없음). application은 여전히 tokio에 의존하지 않는다([application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)).

## 5. CI

### 5.1 언제 도는가

| 이벤트 | 범위 |
|---|---|
| develop으로 가는 PR | develop과의 merge-base 대비 바뀐 파일로 영향 범위를 계산한다(§5.2) |
| develop 푸시 | Squash Merge라 커밋 하나가 PR 하나다. `HEAD~1` 대비로 계산한다 |
| 배포 빌드 | 영향 범위를 쓰지 않고 전체를 빌드·테스트한다(배포 흐름은 미확인, §7) |

### 5.2 영향 범위 계산 (`cargo xtask affected`)

1. 바뀐 파일을 아래 경로 표로 crate 또는 작업에 대응시킨다.
2. `cargo metadata`의 의존 그래프에서 그 crate들의 역의존 crate를 모두 모은다.
3. 모은 crate만 `cargo check`, `cargo clippy`, `cargo nextest run -p …`로 돌린다.

- 의존이 Cargo `[dependencies]`로 강제되므로 역의존 계산을 믿을 수 있다. 같은 `cargo metadata`를 의존 그래프 검사(§5.4)와 함께 쓴다.
- `xtask`는 워크스페이스 안의 도구 crate다. 앱 crate에 의존하지 않고, 앱 crate도 `xtask`에 의존하지 않는다.

**경로 표**

| 바뀐 경로 | 대응 |
|---|---|
| `crates/<crate>/**` | 그 crate |
| `design/samples/**` | `presenters/*` 전부(Presenter 테스트의 기대값, [presentation §2.1](../layers/03_presentation.md#21-프런트엔드-spa-내부-구조)). Presenter crate는 순수 로직이라 전부 돌려도 싸다 |
| `ui/**` | 프런트엔드 작업(타입 검사, `eslint-plugin-boundaries`, 테스트). Rust crate는 돌리지 않는다 |
| `crates/presentation/presenters/**` 중 ViewModel 타입 | 위에 더해 TypeScript 타입 생성 후 `ui/viewmodels/`와 차이가 없는지 확인 |
| `Cargo.toml`(워크스페이스), `Cargo.lock`, `rust-toolchain.toml`, `.cargo/**`, `deny.toml` | 전체 |
| `.github/workflows/**`, `crates/xtask/**` | 전체 |
| `docs/**`, `design/` 중 samples 밖, `.github/ISSUE_TEMPLATE/**` | 빌드 없음 |
| 그 밖의 경로 | 전체(대응표에 없는 경로는 안전한 쪽으로 처리한다) |

### 5.3 러너

| 러너 | 돌리는 것 |
|---|---|
| Linux | 플랫폼과 무관한 crate 전부: `domain/*`, `application/*`, data_access(키체인 제외), `ai/*`, `learning/media`·`transcribe`·`pdf`, `local_store/*`, `presenters/*`, `ipc/*`, `ipc_server`, `bin/cdx`, 프런트엔드 |
| macOS | `platform/keychain`, `learning/macos`, `tauri_commands`, `bin/app` 빌드, 서명·entitlement 조합 검사([platform §3](02_platform.md#3-설치배포)) |
| Windows | Windows 구현 착수 후 `learning/windows`와 앱 빌드 |

- 키체인 구현을 `platform/keychain`으로 분리한 이유가 이것이다. 그 위의 crate는 테스트에서 메모리 구현을 쓰므로 Linux에서 돈다.
- macOS 러너 작업은 영향 범위에 macOS 전용 crate가 있을 때만 돈다.

### 5.4 항상 도는 검사

| 검사 | 내용 |
|---|---|
| 의존 그래프 | `cargo xtask deps-check`: `cargo metadata`를 [System Architecture §3.2](../02_system_architecture.md#32-의존-규칙-cargo-dependencies로-강제) 표와 대조한다 |
| `cargo deny` | 라이선스·취약점·금지 crate |
| `cargo fmt --check` | 바뀐 crate만 |

### 5.5 캐시

- `target/`은 러너별·`Cargo.lock` 해시별로 캐시한다(예: `Swatinem/rust-cache`). develop 푸시에서 저장하고 PR에서는 읽기만 한다.
- whisper.cpp 네이티브 빌드(`learning/transcribe`)는 캐시가 없으면 가장 비싼 항목이다. 이 crate가 영향 범위에 없으면 빌드하지 않는다.

## 6. 측정

- 스프린트에 한 번, 클린 빌드와 대표 변경(§2.3 표의 유스케이스·Source·DTO 한 줄 수정)의 증분 빌드 시간을 `cargo build --timings`로 잰다.
- 표의 예상과 다르게 다시 빌드되는 crate가 보이면 crate 경계를 고친다.
- 도구·검사를 더하는 기준은 [Design Process §3](../03_design_process.md#3-규칙을-늘리는-기준)과 같다. 예: feature 통합으로 `-p` 단위 빌드가 서로 캐시를 깨는 일이 실제로 측정되면 그때 `cargo-hakari`(workspace-hack)를 검토한다.

## 7. 미확인 항목

- trait의 `async fn`을 dyn으로 쓸 수 있는지(착수 때 툴체인 기준). 수용 기준: 포트가 proc-macro 없이 `Arc<dyn Port>`로 주입된다
- 개발 프로필의 의존 crate 최적화 수준(착수 때 측정)
- 배포 빌드 흐름(태그·브랜치, 서명·공증 단계를 CI에서 할지)
- macOS 러너 비용과 실행 빈도
