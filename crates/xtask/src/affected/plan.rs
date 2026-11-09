//! 영향 범위 계획: 바뀐 파일 → 돌릴 crate·작업, 러너별 crate, 실행할 명령. 순수 함수만 둔다(I/O 없음).

use std::collections::BTreeSet;

use super::graph::{Member, Workspace};
use super::paths::{Effect, classify};

/// 계산 기준이 되는 CI 이벤트(build_ci §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// develop으로 가는 PR: develop과의 merge-base 대비
    PullRequest,
    /// develop 푸시: 푸시 전 커밋(`github.event.before`) 대비
    Push,
}

/// 바뀐 파일 목록을 얻을 비교 기준.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffBase {
    /// `git merge-base <ref> HEAD`의 결과와 비교한다
    MergeBase(String),
    /// 이 리비전과 바로 비교한다
    Rev(String),
    /// 비교할 커밋이 없다: 전체를 돌린다
    All,
}

/// 이벤트와 인자로 비교 기준을 정한다. `explicit`(`--base`)이 있으면 그것을 쓴다.
pub fn diff_base(
    event: Event,
    base_ref: &str,
    before: Option<&str>,
    explicit: Option<&str>,
) -> DiffBase {
    if let Some(rev) = explicit {
        return DiffBase::Rev(rev.to_owned());
    }
    match event {
        Event::PullRequest => DiffBase::MergeBase(base_ref.to_owned()),
        // 한 번에 여러 커밋이 푸시될 수 있으므로(rebase merge·merge commit) HEAD~1이 아니라 푸시 전 커밋과 비교한다.
        // 새 브랜치 푸시는 before가 0000…이다
        Event::Push => match before {
            Some(sha) if !sha.is_empty() && !sha.bytes().all(|b| b == b'0') => {
                DiffBase::Rev(sha.to_owned())
            }
            _ => DiffBase::All,
        },
    }
}

/// 작업을 돌릴 러너(build_ci §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runner {
    Linux,
    Macos,
    /// 개발자 컴퓨터: 어느 러너에도 배정되지 않은 crate(Windows 전용)만 뺀다
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    /// 전체를 돌리는 경로가 하나라도 있었다
    pub all: bool,
    /// 파일이 직접 바뀐 crate(`cargo fmt --check` 대상)
    pub changed: BTreeSet<String>,
    /// check·clippy·nextest·doc test를 돌릴 crate(역의존 포함, 러너 구분 전)
    pub crates: BTreeSet<String>,
    /// 프런트엔드 작업(타입 검사, eslint-plugin-boundaries)을 돌린다
    pub frontend: bool,
    /// 프런트엔드 테스트와 배포용 빌드(`vite build`)도 돌린다(`ui/**` 대응)
    pub frontend_full: bool,
    /// 파일별 대응(출력용)
    pub files: Vec<(String, Effect)>,
}

/// 전체를 돌리는 계획(경로 표의 "전체" 행, 비교 기준을 못 구했을 때).
pub fn full(workspace: &Workspace) -> Plan {
    Plan {
        all: true,
        changed: workspace.names(),
        crates: workspace.names(),
        frontend: true,
        frontend_full: true,
        files: Vec::new(),
    }
}

/// 바뀐 파일 목록으로 계획을 세운다(build_ci §5.2).
pub fn plan(files: &[String], workspace: &Workspace, event: Event) -> Plan {
    let mut plan = Plan::default();
    // 역의존을 따라갈 crate와, 그 crate만 돌릴 crate(Rust 코드가 바뀌지 않은 경우)를 나눈다
    let mut seeds = BTreeSet::new();
    let mut direct = BTreeSet::new();
    for file in files {
        let effect = classify(file, workspace);
        match &effect {
            Effect::All => plan.all = true,
            Effect::None => {}
            Effect::PresenterSamples => {
                // 샘플은 Presenter 테스트의 기대값이다. Rust 코드가 바뀐 것이 아니므로 역의존은 따라가지 않는다
                direct.extend(
                    workspace
                        .members
                        .iter()
                        .filter(|m| m.dir.starts_with(PRESENTERS_DIR))
                        .map(|m| m.name.clone()),
                );
                plan.frontend = true;
            }
            Effect::Frontend => {
                plan.frontend = true;
                plan.frontend_full = true;
                // 앱이 ui/dist를 번들에 넣으므로 develop 푸시에서는 bin/app도 빌드한다
                if event == Event::Push
                    && let Some(app) = workspace.by_dir(APP_DIR)
                {
                    direct.insert(app.name.clone());
                }
            }
            Effect::Crate(name) => {
                seeds.insert(name.clone());
            }
        }
        plan.files.push((file.clone(), effect));
    }

    if plan.all {
        return Plan {
            files: plan.files,
            ..full(workspace)
        };
    }
    plan.crates = workspace.reverse_closure(&seeds);
    plan.crates.extend(direct);
    plan.changed = seeds;
    plan
}

const PRESENTERS_DIR: &str = "crates/presentation/presenters/";
const APP_DIR: &str = "crates/bin/app";

/// macOS 러너에서만 도는 crate(build_ci §5.3)
const MACOS_ONLY: &[&str] = &[
    "crates/learning/macos",
    "crates/presentation/tauri_commands",
    "crates/bin/app",
];
/// Linux와 macOS에서 모두 도는 crate(공통·메모리 구현은 Linux, macOS 구현은 macOS)
const LINUX_AND_MACOS: &[&str] = &["crates/platform/keychain"];
/// Windows 러너(Windows 구현 착수 후)에서 돌 crate. 지금은 어느 러너도 돌리지 않는다
const WINDOWS_ONLY: &[&str] = &["crates/learning/windows"];

/// crate 폴더가 배정된 러너(build_ci §5.3). 빈 목록이면 지금은 어느 러너도 돌리지 않는다.
pub fn runners_of(dir: &str) -> &'static [Runner] {
    if MACOS_ONLY.contains(&dir) {
        &[Runner::Macos]
    } else if LINUX_AND_MACOS.contains(&dir) {
        &[Runner::Linux, Runner::Macos]
    } else if WINDOWS_ONLY.contains(&dir) {
        &[]
    } else {
        &[Runner::Linux]
    }
}

/// 계획의 crate 중 `runner`에서 돌릴 것(이름 순).
pub fn crates_for<'a>(plan: &Plan, workspace: &'a Workspace, runner: Runner) -> Vec<&'a Member> {
    let mut members: Vec<&Member> = plan
        .crates
        .iter()
        .filter_map(|name| workspace.get(name))
        .filter(|m| {
            let runners = runners_of(&m.dir);
            match runner {
                Runner::Local => !runners.is_empty(),
                _ => runners.contains(&runner),
            }
        })
        .collect();
    members.sort_by(|a, b| a.name.cmp(&b.name));
    members
}

/// crate 목록에 돌릴 cargo 명령(check, clippy, nextest, doc test). 빈 목록이면 명령도 없다.
pub fn cargo_commands(members: &[&Member]) -> Vec<Vec<String>> {
    if members.is_empty() {
        return Vec::new();
    }
    let packages = |only_lib: bool| -> Vec<String> {
        members
            .iter()
            .filter(|m| !only_lib || m.has_lib)
            .flat_map(|m| ["-p".to_owned(), m.name.clone()])
            .collect()
    };
    let command = |head: &[&str], only_lib: bool, tail: &[&str]| -> Vec<String> {
        head.iter()
            .map(|s| (*s).to_owned())
            .chain(packages(only_lib))
            .chain(tail.iter().map(|s| (*s).to_owned()))
            .collect()
    };

    let mut commands = vec![
        command(&["cargo", "check", "--all-targets"], false, &[]),
        command(
            &["cargo", "clippy", "--all-targets"],
            false,
            &["--", "-D", "warnings"],
        ),
        // 테스트가 아직 없는 crate(골격)가 있어도 실패하지 않게 한다
        command(&["cargo", "nextest", "run", "--no-tests=pass"], false, &[]),
    ];
    // nextest는 doctest를 돌리지 않는다(build_ci §3). doctest는 lib 대상에만 있다
    if members.iter().any(|m| m.has_lib) {
        commands.push(command(&["cargo", "test", "--doc"], true, &[]));
    }
    commands
}

/// 바뀐 crate의 `cargo fmt --check`(build_ci §5.4). 돌릴 것이 없으면 `None`.
pub fn fmt_command(plan: &Plan) -> Option<Vec<String>> {
    let to_vec = |items: &[&str]| items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    if plan.all {
        return Some(to_vec(&["cargo", "fmt", "--all", "--check"]));
    }
    if plan.changed.is_empty() {
        return None;
    }
    let mut command = to_vec(&["cargo", "fmt", "--check"]);
    for name in &plan.changed {
        command.push("-p".to_owned());
        command.push(name.clone());
    }
    Some(command)
}

/// 프런트엔드 작업(`ui/package.json`의 스크립트). 돌릴 것이 없으면 빈 목록.
pub fn frontend_commands(plan: &Plan) -> Vec<Vec<String>> {
    let npm = |args: &[&str]| {
        ["npm", "--prefix", "ui"]
            .iter()
            .chain(args)
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
    };
    let mut commands = Vec::new();
    if plan.frontend {
        commands.push(npm(&["ci"]));
        commands.push(npm(&["run", "typecheck"]));
        // eslint-plugin-boundaries 포함
        commands.push(npm(&["run", "lint"]));
    }
    if plan.frontend_full {
        // 폴더 의존 규칙의 위반 픽스처 테스트 포함
        commands.push(npm(&["run", "test"]));
        // 배포용 빌드
        commands.push(npm(&["run", "build"]));
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::super::graph::tests::sample;
    use super::*;

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| (*p).into()).collect()
    }

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).into()).collect()
    }

    fn joined(commands: Vec<Vec<String>>) -> Vec<String> {
        commands.iter().map(|c| c.join(" ")).collect()
    }

    fn names(members: Vec<&Member>) -> Vec<&str> {
        members.iter().map(|m| m.name.as_str()).collect()
    }

    const BEFORE: &str = "1111111111111111111111111111111111111111";

    #[test]
    fn pr은_merge_base와_비교한다() {
        assert_eq!(
            diff_base(Event::PullRequest, "origin/develop", None, None),
            DiffBase::MergeBase("origin/develop".into())
        );
        // PR에서는 before를 쓰지 않는다
        assert_eq!(
            diff_base(Event::PullRequest, "origin/develop", Some(BEFORE), None),
            DiffBase::MergeBase("origin/develop".into())
        );
    }

    #[test]
    fn 푸시는_푸시_전_커밋과_비교한다() {
        // 한 번에 여러 커밋이 푸시돼도(rebase merge·merge commit) 모두 들어간다
        assert_eq!(
            diff_base(Event::Push, "origin/develop", Some(BEFORE), None),
            DiffBase::Rev(BEFORE.into())
        );
    }

    #[test]
    fn 푸시_전_커밋이_없으면_전체로_폴백한다() {
        // 새 브랜치 푸시는 before가 0000…이다
        let zero = "0".repeat(40);
        assert_eq!(
            diff_base(Event::Push, "origin/develop", Some(&zero), None),
            DiffBase::All
        );
        assert_eq!(
            diff_base(Event::Push, "origin/develop", None, None),
            DiffBase::All
        );
        assert_eq!(
            diff_base(Event::Push, "origin/develop", Some(""), None),
            DiffBase::All
        );
    }

    #[test]
    fn 명시한_기준이_우선한다() {
        assert_eq!(
            diff_base(Event::PullRequest, "origin/develop", None, Some("abc123")),
            DiffBase::Rev("abc123".into())
        );
        assert_eq!(
            diff_base(Event::Push, "origin/develop", Some(BEFORE), Some("abc123")),
            DiffBase::Rev("abc123".into())
        );
    }

    #[test]
    fn 비교_기준을_못_구하면_전체_계획() {
        // before 객체가 없을 때(force push 등) 쓰는 계획: 표의 "전체" 행과 같다
        let ws = sample();
        let mut expected = plan(&files(&["Cargo.lock"]), &ws, Event::Push);
        expected.files.clear();
        assert_eq!(full(&ws), expected);
    }

    #[test]
    fn 문서만_바뀌면_아무것도_돌리지_않는다() {
        let p = plan(
            &files(&["docs/a.md", "docs/b/c.md"]),
            &sample(),
            Event::PullRequest,
        );
        assert!(!p.all);
        assert!(p.crates.is_empty());
        assert!(p.changed.is_empty());
        assert!(!p.frontend);
        assert!(!p.frontend_full);
        assert!(frontend_commands(&p).is_empty());
        assert_eq!(fmt_command(&p), None);
        assert!(cargo_commands(&crates_for(&p, &sample(), Runner::Linux)).is_empty());
    }

    #[test]
    fn 바뀐_파일이_없으면_아무것도_돌리지_않는다() {
        let p = plan(&[], &sample(), Event::Push);
        assert_eq!(p, Plan::default());
    }

    #[test]
    fn 디자인_샘플만_바뀌면_presenter_전부와_프런트엔드() {
        let p = plan(
            &files(&["design/samples/school/todo.json"]),
            &sample(),
            Event::PullRequest,
        );
        assert!(!p.all);
        assert_eq!(p.crates, set(&["presenters-common", "presenters-school"]));
        // 타입 검사와 eslint-plugin-boundaries만(테스트·vite build는 ui/** 대응)
        assert!(p.frontend);
        assert!(!p.frontend_full);
        assert_eq!(
            joined(frontend_commands(&p)),
            [
                "npm --prefix ui ci",
                "npm --prefix ui run typecheck",
                "npm --prefix ui run lint"
            ]
        );
        // Rust 파일이 바뀌지 않았으므로 fmt 대상은 없다
        assert_eq!(fmt_command(&p), None);
        assert_eq!(
            names(crates_for(&p, &sample(), Runner::Linux)),
            ["presenters-common", "presenters-school"]
        );
        assert!(crates_for(&p, &sample(), Runner::Macos).is_empty());
    }

    #[test]
    fn crate가_바뀌면_역의존까지_돌린다() {
        let p = plan(
            &files(&["crates/application/usecases/school/src/lib.rs"]),
            &sample(),
            Event::PullRequest,
        );
        assert_eq!(p.changed, set(&["usecases-school"]));
        assert_eq!(
            p.crates,
            set(&[
                "usecases-school",
                "presenters-school",
                "tauri-commands",
                "app"
            ])
        );
        assert!(!p.frontend);
        assert_eq!(
            names(crates_for(&p, &sample(), Runner::Linux)),
            ["presenters-school", "usecases-school"]
        );
        assert_eq!(
            names(crates_for(&p, &sample(), Runner::Macos)),
            ["app", "tauri-commands"]
        );
    }

    #[test]
    fn 표에_없는_경로가_하나라도_있으면_전체() {
        let p = plan(
            &files(&["docs/a.md", "README.md"]),
            &sample(),
            Event::PullRequest,
        );
        assert!(p.all);
        assert_eq!(p.crates, sample().names());
        assert!(p.frontend && p.frontend_full);
        assert_eq!(
            fmt_command(&p).unwrap(),
            ["cargo", "fmt", "--all", "--check"]
        );
    }

    #[test]
    fn ui는_pr에서_rust를_돌리지_않고_푸시에서_앱을_빌드한다() {
        let pr = plan(&files(&["ui/src/main.tsx"]), &sample(), Event::PullRequest);
        assert!(pr.frontend && pr.frontend_full);
        assert!(pr.crates.is_empty());
        // ui/** 대응: 타입 검사, eslint-plugin-boundaries, 테스트, 배포용 빌드
        assert_eq!(
            joined(frontend_commands(&pr)),
            [
                "npm --prefix ui ci",
                "npm --prefix ui run typecheck",
                "npm --prefix ui run lint",
                "npm --prefix ui run test",
                "npm --prefix ui run build"
            ]
        );

        let push = plan(&files(&["ui/src/main.tsx"]), &sample(), Event::Push);
        assert!(push.frontend);
        assert_eq!(push.crates, set(&["app"]));
        assert!(push.changed.is_empty());
    }

    #[test]
    fn 러너_배정은_build_ci_5_3을_따른다() {
        assert_eq!(runners_of("crates/domain/core"), [Runner::Linux]);
        assert_eq!(runners_of("crates/xtask"), [Runner::Linux]);
        assert_eq!(
            runners_of("crates/platform/keychain"),
            [Runner::Linux, Runner::Macos]
        );
        for dir in [
            "crates/learning/macos",
            "crates/presentation/tauri_commands",
            "crates/bin/app",
        ] {
            assert_eq!(runners_of(dir), [Runner::Macos], "{dir}");
        }
        assert!(runners_of("crates/learning/windows").is_empty());
    }

    #[test]
    fn 로컬은_windows_전용만_뺀다() {
        let ws = sample();
        let p = plan(&files(&["Cargo.lock"]), &ws, Event::PullRequest);
        let local = names(crates_for(&p, &ws, Runner::Local));
        assert!(local.contains(&"app"));
        assert!(local.contains(&"domain-core"));
        assert!(!local.contains(&"learning-windows"));
        // Linux와 macOS에 모두 배정된 crate는 각 러너에 한 번씩 나온다
        assert!(names(crates_for(&p, &ws, Runner::Linux)).contains(&"platform-keychain"));
        assert!(names(crates_for(&p, &ws, Runner::Macos)).contains(&"platform-keychain"));
    }

    #[test]
    fn cargo_명령은_check_clippy_nextest_doctest_순서() {
        let ws = sample();
        let mut cdx = ws.get("cdx").unwrap().clone();
        cdx.has_lib = false;
        let core = ws.get("domain-core").unwrap();
        let commands = cargo_commands(&[core, &cdx]);
        let joined: Vec<String> = commands.iter().map(|c| c.join(" ")).collect();
        assert_eq!(
            joined,
            [
                "cargo check --all-targets -p domain-core -p cdx",
                "cargo clippy --all-targets -p domain-core -p cdx -- -D warnings",
                "cargo nextest run --no-tests=pass -p domain-core -p cdx",
                // doc test는 lib 대상이 있는 crate만
                "cargo test --doc -p domain-core",
            ]
        );
    }

    #[test]
    fn lib가_없으면_doc_test를_빼고_crate가_없으면_명령도_없다() {
        let mut cdx = sample().get("cdx").unwrap().clone();
        cdx.has_lib = false;
        assert_eq!(cargo_commands(&[&cdx]).len(), 3);
        assert!(cargo_commands(&[]).is_empty());
    }

    #[test]
    fn fmt는_바뀐_crate만_검사한다() {
        let p = plan(
            &files(&["crates/ipc/cli/src/lib.rs", "crates/domain/core/src/lib.rs"]),
            &sample(),
            Event::PullRequest,
        );
        assert_eq!(
            fmt_command(&p).unwrap(),
            [
                "cargo",
                "fmt",
                "--check",
                "-p",
                "domain-core",
                "-p",
                "ipc-cli"
            ]
        );
    }
}
