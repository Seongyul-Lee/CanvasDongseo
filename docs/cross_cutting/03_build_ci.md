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
| 작업 | 오래 걸리는 작업의 상태·진행률 | `usecases/jobs`, `presenters/jobs` |
| 화면 | 기능 단위 디자인([Design Process §1](../03_design_process.md#1-흐름)) | `presenters/*`, `tauri_commands`, `ui/` |
| AI 연동 통로 | `cdx` 명령·프로토콜 | `ipc/wire`, `ipc/cli`, `ipc_server`, `bin/cdx` |

### 2.2 나누는 경우와 나누지 않는 경우

- **나눈다**: Actor가 다르거나, 같은 Actor 안에서도 변경 빈도가 크게 다를 때. 대표적인 경우가 **포트(안정)와 유스케이스(자주 바뀜)**다. 어댑터는 포트만 필요하므로 유스케이스를 고쳐도 어댑터가 다시 빌드되지 않아야 한다.
- **나누지 않는다**: Actor와 변경 빈도가 같은데 파일 수만 많을 때. crate마다 rustc 호출·링크 비용이 붙는다. 크기만으로는 나누지 않는다.
- 새 crate를 만들면 [System Architecture §3.1·§3.2](../02_system_architecture.md#3-레이어-지도)에 행을 더하고, §2.3의 표가 달라지면 고친다. §2.3의 표는 §3.2 의존 규칙에서 역의존을 따라가 얻는다. 규칙을 바꾸면 표를 다시 따라가 확인한다.

### 2.3 변경 하나가 다시 빌드시키는 범위 (목표)

[System Architecture §3.2](../02_system_architecture.md#32-의존-규칙-cargo-dependencies로-강제) 의존 규칙에서 역의존을 따라간 결과다. 구현 후 `cargo build --timings`로 확인한다(§6).

| 바뀐 곳 | 다시 빌드되는 crate |
|---|---|
| 유스케이스 하나(예: `usecases/school`) | 그 crate, 같은 묶음 `presenters`, (`school`·`ai`·`jobs`이면) `presenters/shell`, `tauri_commands`, `ipc_server`, `bin/app` |
| `usecases/jobs` | 그 crate, 그것을 쓰는 유스케이스 crate(현재 `usecases/learning`), 그 묶음의 `presenters`와 `presenters/jobs`·`presenters/shell`, `tauri_commands`, `ipc_server`, `bin/app` |
| 출력 DTO 필드 하나 | 위와 같다(출력 DTO는 유스케이스 crate에 있다) |
| Source 하나(예: `canvas`) | 그 crate, `bin/app` |
| DS 계약 하나(예: `contracts/notice`) | 그 crate, 그 계약을 구현하는 Source, `repositories`, `bin/app` |
| `domain/slides` 기준값 | 그 crate, `usecases/learning`, `presenters/learning`, `tauri_commands`, `ipc_server`, `bin/app`. 포트와 어댑터는 `domain/slides`를 모른다 |
| AI 제공자 어댑터 | 그 crate, `bin/app` |
| `ipc/cli`(명령 설명) | 그 crate, `bin/cdx`, `bin/app` |
| `platform/keychain` | 그 crate, `data_access/auth`, `local_store/learning`, `bin/app` |
| 포트(예: `ports/learning`) | 그 포트를 쓰는 유스케이스와 구현 어댑터, 그 위. 포트는 드물게 바뀐다 |
| `application/kernel`, `domain/core` | 거의 전부. 그래서 이 두 crate에는 안정된 것만 둔다 |

## 3. 로컬 개발

| 항목 | 규칙 |
|---|---|
| 편집 직후 훅 | `cargo check -p <바뀐 crate>`(전체 빌드 대신). 의존 그래프 검사(§5.4)도 여기서 돈다 |
| 테스트 | `cargo nextest run -p <바뀐 crate>`와 `cargo test --doc -p <바뀐 crate>`(nextest는 doctest를 돌리지 않는다). 역의존 crate까지 돌릴 때는 `cargo xtask affected`(§5.2)를 쓴다 |
| 개발 프로필 | `[profile.dev] debug = "line-tables-only"`. 의존 crate는 `[profile.dev.package."*"] opt-level = 1`처럼 최적화해 실행 속도를 얻는다(착수 때 측정해 정함) |
| 학습 기능 끄기 | `bin/app`에 `learning` cargo feature(기본 켬)를 둔다. 개발 중 학습 기능을 만지지 않을 때는 `--no-default-features`로 whisper.cpp·ffmpeg 관련 crate를 빌드·링크하지 않는다. **배포 빌드는 항상 기본 feature로 만든다** |
| Tauri CLI | Tauri 앱은 관례 위치(`src-tauri/`)가 아니라 `crates/bin/app`에 있다. CLI는 현재 폴더와 `src-tauri/`를 먼저 보고, 없으면 폴더를 깊이 3까지 뒤져 `tauri.conf.json`을 찾는다(tauri-cli 2.12.4 소스 확인). 탐색에 기대지 않도록 `TAURI_APP_PATH=crates/bin/app`, `TAURI_FRONTEND_PATH=ui`를 지정해 실행한다(`ui/package.json`의 스크립트에 둔다). `tauri.conf.json`의 `frontendDist`는 그 파일 기준 상대 경로(`../../../ui/dist`)다 |
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
| develop 푸시 | 푸시 전 커밋(`github.event.before`) 대비로 계산한다. 이 저장소는 Squash Merge 외에 rebase merge·merge commit도 허용하므로 한 번에 여러 커밋이 푸시될 수 있고, `HEAD~1` 대비면 앞 커밋의 변경이 빠진다. `before`가 없거나 `0000…`(새 브랜치)이거나 그 커밋이 저장소에 없으면(force push 등) 전체를 돌린다. 연속 머지에서 앞 커밋의 검사가 취소되지 않도록 develop 푸시는 실행을 취소하지 않는다(`cancel-in-progress`는 PR에서만) |
| 배포 빌드 | 영향 범위를 쓰지 않고 전체를 빌드·테스트한다(배포 흐름은 미확인, §7) |

### 5.2 영향 범위 계산 (`cargo xtask affected`)

1. 바뀐 파일을 아래 경로 표로 crate 또는 작업에 대응시킨다.
2. `cargo metadata`의 의존 그래프에서 그 crate들의 역의존 crate를 모두 모은다.
3. 모은 crate만 `cargo check`, `cargo clippy`, `cargo nextest run -p …`, `cargo test --doc -p …`로 돌린다. 모은 crate에 `presenters/*`가 있으면 TypeScript 타입을 다시 생성해 `ui/src/viewmodels/`와 차이가 없는지 확인한다.

- 의존이 Cargo `[dependencies]`로 강제되므로 역의존 계산을 믿을 수 있다. 같은 `cargo metadata`를 의존 그래프 검사(§5.4)와 함께 쓴다.
- `xtask`는 워크스페이스 안의 도구 crate다. 앱 crate에 의존하지 않고, 앱 crate도 `xtask`에 의존하지 않는다.

**경로 표** — 파일마다 **위에서부터 처음 맞는 행**을 쓴다. 전체를 돌리는 행을 먼저 둔다.

| 바뀐 경로 | 대응 |
|---|---|
| `Cargo.toml`(워크스페이스 루트), `Cargo.lock`, `rust-toolchain.toml`, `.cargo/**`, `deny.toml` | 전체 |
| `.github/workflows/**`, `crates/xtask/**` | 전체 |
| `.github/**`(위 행 밖: PR·이슈 템플릿 등) | 빌드 없음 |
| `design/samples/**` | `presenters/*` 전부(Presenter 테스트의 기대값, [presentation §2.1](../layers/03_presentation.md#21-프런트엔드-spa-내부-구조)). Presenter crate는 순수 로직이라 전부 돌려도 싸다. 샘플 모드의 입력이기도 하므로 프런트엔드 타입 검사와 `eslint-plugin-boundaries`도 돌린다([presentation §2.4](../layers/03_presentation.md#24-샘플-모드)) |
| `ui/**` | 프런트엔드 작업(타입 검사, `eslint-plugin-boundaries`, 테스트, **배포용 빌드** `vite build`). 배포용 빌드는 샘플 구현이나 `design/samples/`가 결과에 들어가면 실패한다([presentation §2.4](../layers/03_presentation.md#24-샘플-모드)). PR에서는 Rust crate를 돌리지 않는다. develop 푸시에서는 `bin/app` 빌드도 돌린다(앱이 `ui/dist`를 번들에 넣으므로) |
| `crates/bin/app/tauri.conf.json` | `bin/app`(아래 crate 행과 같다)과 프런트엔드 작업(위 `ui/**` 행과 같다). 프런트엔드 테스트가 창 설정을 읽어 화면 쪽 값(타이틀바 높이, `ui/src/lib/windowControlsOverlay.ts`)과 맞는지 확인하므로, 창 설정만 바뀐 PR에서도 테스트가 돌아야 한다 |
| `crates/<crate>/**` | 그 crate |
| `docs/**`, `design/**`(samples 밖) | 빌드 없음 |
| 그 밖의 경로 | 전체(대응표에 없는 경로는 안전한 쪽으로 처리한다) |

### 5.3 러너

| 러너 | 돌리는 것 |
|---|---|
| Linux | `learning/macos`·`learning/windows`·`tauri_commands`·`bin/app`을 뺀 모든 crate: `domain/*`, `application/*`, `data_access/*`(`auth` 포함), `ai/*`, `learning/media`·`transcribe`·`pdf`, `local_store/*`, `platform/fs`, `platform/keychain`(공통·메모리 구현), `presenters/*`, `ipc/*`, `ipc_server`, `bin/cdx`, `xtask`, 프런트엔드 |
| macOS | `platform/keychain`(macOS 구현), `learning/macos`, `tauri_commands`, `bin/app` 빌드, 서명·entitlement 조합 검사([platform §3](02_platform.md#3-설치배포)) |
| Windows | Windows 구현 착수 후 `platform/keychain`(Windows 구현), `learning/windows`, 앱 빌드 |

- `platform/keychain`이 모든 OS에서 컴파일되게 하는 것이 Linux 배정의 전제다. 이 crate는 `SecretStore <I>`와 메모리 구현을 OS와 무관하게 두고, OS 키체인 구현만 `#[cfg(target_os = ...)]`와 `[target.'cfg(...)'.dependencies]`(`apple-native-keyring-store` 등) 뒤에 둔다. `auth`와 `local_store/learning`은 `Arc<dyn SecretStore>`를 받으므로 테스트에서 메모리 구현을 쓴다.
- `tauri_commands`는 `tauri`를 쓰므로 Linux에서도 컴파일할 수 있지만 시스템 웹뷰 라이브러리가 필요하다. 착수 때 Linux 러너에 둘지 정한다(§7).
- macOS 러너 작업은 영향 범위에 위 macOS 열의 crate가 있을 때만 돈다.

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
- 합칠 후보를 미리 정해 둔다: `ports/common`·`presenters/common`·`presenters/jobs`(작을 것으로 예상), `local_store/settings`. 측정에서 이점이 없으면 이웃 crate와 합친다.
- 도구·검사를 더하는 기준은 [Design Process §3](../03_design_process.md#3-규칙을-늘리는-기준)과 같다. 예: feature 통합으로 `-p` 단위 빌드가 서로 캐시를 깨는 일이 실제로 측정되면 그때 `cargo-hakari`(workspace-hack)를 검토한다.

## 7. 미확인 항목

- ~~trait의 `async fn`을 dyn으로 쓸 수 있는지~~ **확인함(2026-11-09, Rust 1.99.1, `rust-toolchain.toml`)**. trait의 `async fn`은 dyn으로 쓸 수 없다(`E0038: the trait is not dyn compatible ... because method is async`). §4의 대안대로 `application/kernel`에 `BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>`를 두고, 포트 메서드는 `async fn` 대신 이를 돌려준다. proc-macro와 tokio 없이 `Arc<dyn Port>`로 주입되고 future가 `Send`임을 `crates/application/kernel/tests/main.rs`(테스트 전용 포트)로 확인했다. 같은 crate의 `compile_fail,E0038` doctest가 `async fn` 포트의 dyn 사용이 막혀 있음을 고정하므로, 툴체인을 올려 이 doctest가 실패하면 `async fn`으로 바꿀 수 있는지 다시 본다. `BoxStream`은 스트림 trait(`futures-core` 등)를 정해야 하므로 스트리밍 포트를 만들 때 더한다
- 개발 프로필의 의존 crate 최적화 수준(착수 때 측정)
- 배포 빌드 흐름(태그·브랜치, 서명·공증 단계를 CI에서 할지)
- macOS 러너 비용과 실행 빈도
- `tauri_commands`를 Linux 러너에서 돌릴지(웹뷰 개발 패키지 설치 비용)
- ~~`crates/bin/app` 위치에서 `cargo tauri dev`·`build`가 동작하는지~~ **확인함(2026-11-09, tauri 2.12.5, tauri-cli 2.12.4, macOS arm64)**. 저장소 루트에서 §3의 환경 변수(`TAURI_APP_PATH=crates/bin/app TAURI_FRONTEND_PATH=ui`)로 `cargo tauri dev`가 창을 띄우고, `cargo tauri build`가 `target/release/bundle/macos/CanvasDongseo.app`을 만든다(워크스페이스 `target/`을 쓴다). `ui/`가 아직 없어 `TAURI_FRONTEND_PATH`는 지금 효과가 없다(경로를 찾지 못하면 CLI가 `crates/bin`을 프런트엔드 폴더로 쓰고, `beforeDevCommand`가 없어 동작에 영향이 없다). 그동안 `tauri.conf.json`의 `build.frontendDist`는 `crates/bin/app/frontend`의 정적 HTML을 가리킨다. 프런트엔드 SPA를 붙일 때(#19) `frontendDist`를 `../../../ui/dist`로 바꾸고 `devUrl`·`beforeDevCommand`·`beforeBuildCommand`를 더한 뒤 `frontend/`를 지운다. **#19에서 반영함(2026-11-09)**: `frontendDist`는 `../../../ui/dist`, `devUrl`은 `http://localhost:5173`이고, `beforeDevCommand`·`beforeBuildCommand`(`npm run dev`·`npm run build`)는 `cwd`를 `../../../ui`로 적어 `TAURI_FRONTEND_PATH` 없이도 `ui/`에서 돈다(CLI는 훅을 `crates/bin/app` 기준으로 실행한다, tauri-cli 2.12.4 소스 확인). `frontend/`는 지웠다. `npm --prefix ui run tauri dev`·`build`로 창에 SPA가 뜨는 것을 확인했다. 번들 대상은 `app`만 둔다(DMG는 쓰지 않는다, [platform §3](02_platform.md#3-설치배포)). 서명·공증과 `.pkg`는 확인 범위 밖이다
