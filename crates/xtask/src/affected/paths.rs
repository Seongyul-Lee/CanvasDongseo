//! build_ci §5.2 경로 표(데이터)와 매칭. 순수 함수만 둔다(I/O 없음).
//! 표를 바꾸면 docs/cross_cutting/03_build_ci.md §5.2와 이 파일의 `ROWS`를 함께 고친다.

use super::graph::Workspace;

/// 바뀐 파일 하나가 일으키는 일.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// 전체를 돌린다
    All,
    /// 빌드 없음
    None,
    /// `presenters/*` 전부 + 프런트엔드 타입 검사·`eslint-plugin-boundaries`
    PresenterSamples,
    /// 프런트엔드 작업. develop 푸시면 `bin/app` 빌드도 돈다
    Frontend,
    /// 그 crate(패키지 이름)와 역의존
    Crate(String),
    /// 그 crate(패키지 이름)와 역의존 + 프런트엔드 작업
    CrateAndFrontend(String),
}

/// 경로 표의 "바뀐 경로" 열.
#[derive(Debug, Clone, Copy)]
enum Pattern {
    /// 경로가 정확히 같다(워크스페이스 루트 파일)
    Exact(&'static str),
    /// 이 폴더 아래(`<폴더>/**`). 값은 `/`로 끝난다
    Under(&'static str),
    /// 워크스페이스 crate 폴더 아래(`crates/<crate>/**`)
    WorkspaceCrate,
    /// 그 밖의 경로
    Anything,
}

/// 경로 표의 "대응" 열. `Crate`는 매칭된 crate 이름을 채운다.
#[derive(Debug, Clone, Copy)]
enum Action {
    All,
    None,
    PresenterSamples,
    Frontend,
    Crate,
    CrateAndFrontend,
}

/// build_ci §5.2 경로 표. 파일마다 위에서부터 처음 맞는 행을 쓴다.
const ROWS: &[(&[Pattern], Action)] = &[
    (
        &[
            Pattern::Exact("Cargo.toml"),
            Pattern::Exact("Cargo.lock"),
            Pattern::Exact("rust-toolchain.toml"),
            Pattern::Under(".cargo/"),
            Pattern::Exact("deny.toml"),
        ],
        Action::All,
    ),
    (
        &[
            Pattern::Under(".github/workflows/"),
            Pattern::Under("crates/xtask/"),
        ],
        Action::All,
    ),
    (&[Pattern::Under(".github/")], Action::None),
    (
        &[Pattern::Under("design/samples/")],
        Action::PresenterSamples,
    ),
    (&[Pattern::Under("ui/")], Action::Frontend),
    // 프런트엔드 테스트(ui/test/lib.test.js)가 창 설정을 읽는다
    (
        &[Pattern::Exact("crates/bin/app/tauri.conf.json")],
        Action::CrateAndFrontend,
    ),
    (&[Pattern::WorkspaceCrate], Action::Crate),
    (
        &[Pattern::Under("docs/"), Pattern::Under("design/")],
        Action::None,
    ),
    (&[Pattern::Anything], Action::All),
];

/// 바뀐 파일 하나(워크스페이스 루트 기준, `/` 구분)를 경로 표로 대응시킨다.
pub fn classify(path: &str, workspace: &Workspace) -> Effect {
    for (patterns, action) in ROWS {
        for pattern in *patterns {
            let crate_name = match pattern {
                Pattern::Exact(p) if path == *p => None,
                Pattern::Under(p) if path.starts_with(p) => None,
                Pattern::WorkspaceCrate if path.starts_with("crates/") => {
                    match workspace.owner_of(path) {
                        Some(member) => Some(member.name.clone()),
                        // crates/ 아래지만 어느 crate 폴더도 아니면 다음 행으로 넘어간다
                        None => continue,
                    }
                }
                Pattern::Anything => None,
                _ => continue,
            };
            return match action {
                Action::All => Effect::All,
                Action::None => Effect::None,
                Action::PresenterSamples => Effect::PresenterSamples,
                Action::Frontend => Effect::Frontend,
                Action::Crate => {
                    Effect::Crate(crate_name.expect("WorkspaceCrate 행만 Crate를 쓴다"))
                }
                Action::CrateAndFrontend => match workspace.owner_of(path) {
                    Some(member) => Effect::CrateAndFrontend(member.name.clone()),
                    // 그 crate가 워크스페이스에 없으면 다음 행으로 넘어간다
                    None => continue,
                },
            };
        }
    }
    unreachable!("마지막 행(Anything)이 모든 경로에 맞는다")
}

#[cfg(test)]
mod tests {
    use super::super::graph::tests::sample;
    use super::*;

    fn c(path: &str) -> Effect {
        classify(path, &sample())
    }

    #[test]
    fn 워크스페이스_루트_설정은_전체() {
        for path in [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            "deny.toml",
        ] {
            assert_eq!(c(path), Effect::All, "{path}");
        }
    }

    #[test]
    fn 워크플로와_xtask는_전체() {
        assert_eq!(c(".github/workflows/checks.yml"), Effect::All);
        assert_eq!(c("crates/xtask/src/main.rs"), Effect::All);
        assert_eq!(c("crates/xtask/Cargo.toml"), Effect::All);
    }

    #[test]
    fn 워크플로_밖의_github는_빌드_없음() {
        assert_eq!(c(".github/pull_request_template.md"), Effect::None);
        assert_eq!(c(".github/ISSUE_TEMPLATE/feature.yml"), Effect::None);
    }

    #[test]
    fn 디자인_샘플은_presenter_샘플_작업() {
        assert_eq!(
            c("design/samples/school/todo.json"),
            Effect::PresenterSamples
        );
    }

    #[test]
    fn ui는_프런트엔드_작업() {
        assert_eq!(c("ui/src/main.tsx"), Effect::Frontend);
        assert_eq!(c("ui/package.json"), Effect::Frontend);
    }

    #[test]
    fn tauri_창_설정은_그_crate와_프런트엔드_작업() {
        // 프런트엔드 테스트(ui/test/lib.test.js)가 창 설정을 읽는다
        assert_eq!(
            c("crates/bin/app/tauri.conf.json"),
            Effect::CrateAndFrontend("app".into())
        );
        // 같은 crate의 다른 파일은 crate 행이다
        assert_eq!(c("crates/bin/app/build.rs"), Effect::Crate("app".into()));
        assert_eq!(
            c("crates/bin/app/capabilities/default.json"),
            Effect::Crate("app".into())
        );
    }

    #[test]
    fn 앱_crate가_없으면_tauri_창_설정은_전체() {
        // 그 crate가 워크스페이스에 없으면 아래 행으로 넘어가 안전하게 전체를 돌린다
        let mut ws = sample();
        ws.members.retain(|m| m.name != "app");
        assert_eq!(classify("crates/bin/app/tauri.conf.json", &ws), Effect::All);
    }

    #[test]
    fn crate_폴더는_그_crate() {
        assert_eq!(
            c("crates/application/usecases/school/src/lib.rs"),
            Effect::Crate("usecases-school".into())
        );
        // crate의 Cargo.toml은 워크스페이스 루트 행이 아니라 crate 행이다
        assert_eq!(c("crates/bin/app/Cargo.toml"), Effect::Crate("app".into()));
    }

    #[test]
    fn 문서와_샘플_밖_디자인은_빌드_없음() {
        assert_eq!(c("docs/cross_cutting/03_build_ci.md"), Effect::None);
        assert_eq!(c("design/mockups/home.html"), Effect::None);
    }

    #[test]
    fn 표에_없는_경로는_전체() {
        assert_eq!(c("README.md"), Effect::All);
        assert_eq!(c("scripts/setup.sh"), Effect::All);
        // crates/ 아래지만 crate 폴더가 아닌 곳(지운 crate 포함)도 안전하게 전체
        assert_eq!(c("crates/README.md"), Effect::All);
        assert_eq!(c("crates/removed/src/lib.rs"), Effect::All);
    }

    #[test]
    fn 위에서부터_처음_맞는_행을_쓴다() {
        // `design/samples/`는 아래 `design/**`(빌드 없음)보다 먼저 맞는다
        assert_eq!(c("design/samples/a.json"), Effect::PresenterSamples);
        // `.github/workflows/`는 아래 `.github/**`(빌드 없음)보다 먼저 맞는다
        assert_eq!(c(".github/workflows/affected.yml"), Effect::All);
        // `crates/xtask/`는 아래 crate 행보다 먼저 맞는다
        assert_eq!(c("crates/xtask/src/affected/mod.rs"), Effect::All);
    }
}
