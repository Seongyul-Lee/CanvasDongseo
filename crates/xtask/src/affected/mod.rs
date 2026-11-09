//! `cargo xtask affected`: 바뀐 파일로 영향 범위를 계산해 그 crate·작업만 돌린다(build_ci §5.1~5.2).
//!
//! - `paths`: §5.2 경로 표(데이터)와 매칭(순수 함수)
//! - `graph`: 워크스페이스 모델과 역의존 계산(순수 함수)
//! - `plan`: 계획·러너 배정·명령 만들기(순수 함수)
//! - `metadata`: `cargo metadata` JSON → `graph` 모델 변환
//! - 이 파일: 인자 해석, `git`·`cargo` 실행, 결과 출력(I/O 조율)

mod args;
mod graph;
mod metadata;
mod paths;
mod plan;

use std::path::Path;
use std::process::{Command, ExitCode};

use args::Args;
use plan::{DiffBase, Plan, Runner};

pub fn run(raw: &[String]) -> ExitCode {
    if raw.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", args::USAGE);
        return ExitCode::SUCCESS;
    }
    let args = match args::parse(raw) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("affected: {err}\n\n{}", args::USAGE);
            return ExitCode::FAILURE;
        }
    };
    match execute(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("affected: {err}");
            ExitCode::FAILURE
        }
    }
}

fn execute(args: &Args) -> Result<(), String> {
    let workspace = metadata::parse(&capture(
        &cargo(),
        &["metadata", "--no-deps", "--format-version", "1"],
    )?)?;

    let base = plan::diff_base(
        args.event,
        &args.base_ref,
        args.before.as_deref(),
        args.base.as_deref(),
    );
    let plan = match resolve_base(&base) {
        Ok(None) => {
            println!("affected: 비교할 커밋이 없어({base:?}) 전체를 돌린다");
            plan::full(&workspace)
        }
        Ok(Some(rev)) => {
            let output = capture(
                "git",
                &["diff", "--name-only", "--no-renames", &rev, "HEAD"],
            )?;
            let files: Vec<String> = output
                .lines()
                .filter(|l| !l.is_empty())
                .map(str::to_owned)
                .collect();
            println!(
                "affected: {base:?} = {} 대비 바뀐 파일 {}개",
                short(&rev),
                files.len()
            );
            plan::plan(&files, &workspace, args.event)
        }
        Err(err) => {
            // 비교 기준을 못 구하면(force push로 before 객체가 없음, 첫 커밋, 얕은 clone 등) 안전한 쪽으로 전체를 돌린다
            println!("affected: 비교 기준({base:?})을 구하지 못해 전체를 돌린다: {err}");
            plan::full(&workspace)
        }
    };

    print_plan(&plan, &workspace, args.runner);
    if args.github_output {
        write_github_output(&plan, &workspace)?;
    }

    if args.fmt {
        match plan::fmt_command(&plan) {
            Some(command) => run_command(&command)?,
            None => println!("affected: fmt — 바뀐 crate가 없어 건너뛴다"),
        }
    }
    if args.run {
        let members = plan::crates_for(&plan, &workspace, args.runner);
        let commands = plan::cargo_commands(&members);
        if commands.is_empty() {
            println!("affected: {:?} 러너에서 돌릴 crate가 없다", args.runner);
        }
        for command in &commands {
            run_command(command)?;
        }
        if args.frontend {
            run_frontend(&plan)?;
        }
    }
    Ok(())
}

/// 비교할 커밋을 구한다. `Ok(None)`은 비교할 커밋이 없다는 뜻(전체를 돌린다).
fn resolve_base(base: &DiffBase) -> Result<Option<String>, String> {
    let rev = match base {
        DiffBase::All => return Ok(None),
        DiffBase::MergeBase(reference) => capture("git", &["merge-base", reference, "HEAD"])?,
        DiffBase::Rev(rev) => capture(
            "git",
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{rev}^{{commit}}"),
            ],
        )?,
    };
    Ok(Some(rev.trim().to_owned()))
}

fn print_plan(plan: &Plan, workspace: &graph::Workspace, runner: Runner) {
    for (file, effect) in &plan.files {
        println!("  {file} → {effect:?}");
    }
    let list = |members: Vec<&graph::Member>| {
        if members.is_empty() {
            "(없음)".to_owned()
        } else {
            members
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        }
    };
    println!("affected: 전체 = {}", plan.all);
    println!(
        "affected: fmt 대상(바뀐 crate) = {}",
        if plan.changed.is_empty() {
            "(없음)".to_owned()
        } else {
            plan.changed.iter().cloned().collect::<Vec<_>>().join(" ")
        }
    );
    println!(
        "affected: Linux 러너 crate = {}",
        list(plan::crates_for(plan, workspace, Runner::Linux))
    );
    println!(
        "affected: macOS 러너 crate = {}",
        list(plan::crates_for(plan, workspace, Runner::Macos))
    );
    println!(
        "affected: 프런트엔드 작업 = {} (테스트·배포용 빌드 = {})",
        plan.frontend, plan.frontend_full
    );
    println!("affected: 계획 — {runner:?} 러너에서 돌릴 명령:");
    for command in plan::cargo_commands(&plan::crates_for(plan, workspace, runner)) {
        println!("    {}", command.join(" "));
    }
    for command in plan::frontend_commands(plan) {
        println!("    {}", command.join(" "));
    }
}

fn write_github_output(plan: &Plan, workspace: &graph::Workspace) -> Result<(), String> {
    use std::io::Write;
    let path = std::env::var("GITHUB_OUTPUT")
        .map_err(|_| "--github-output인데 GITHUB_OUTPUT이 없다".to_owned())?;
    let lines = format!(
        "rust_linux={}\nrust_macos={}\nfrontend={}\nfrontend_full={}\n",
        !plan::crates_for(plan, workspace, Runner::Linux).is_empty(),
        !plan::crates_for(plan, workspace, Runner::Macos).is_empty(),
        plan.frontend,
        plan.frontend_full,
    );
    std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .and_then(|mut f| f.write_all(lines.as_bytes()))
        .map_err(|e| format!("{path}에 쓰지 못했다: {e}"))?;
    print!("affected: GITHUB_OUTPUT\n{lines}");
    Ok(())
}

fn run_frontend(plan: &Plan) -> Result<(), String> {
    if !plan.frontend {
        return Ok(());
    }
    if !Path::new("ui/package.json").exists() {
        println!("affected: ui/package.json이 없어 프런트엔드 작업을 건너뛴다(계획만 출력)");
        return Ok(());
    }
    for command in plan::frontend_commands(plan) {
        run_command(&command)?;
    }
    Ok(())
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

fn short(rev: &str) -> &str {
    &rev[..rev.len().min(12)]
}

/// 명령을 실행해 표준 출력을 돌려준다. 실패하면 표준 오류를 담아 돌려준다.
fn capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}를 실행하지 못했다: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {}가 실패했다\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 명령을 출력을 그대로 보이며 실행한다. 실패하면 멈춘다.
fn run_command(command: &[String]) -> Result<(), String> {
    println!("affected: $ {}", command.join(" "));
    let (program, rest) = command.split_first().ok_or("빈 명령")?;
    let program = if program == "cargo" {
        cargo()
    } else {
        program.clone()
    };
    let status = Command::new(&program)
        .args(rest)
        .status()
        .map_err(|e| format!("{program}를 실행하지 못했다: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("명령이 실패했다({status}): {}", command.join(" ")))
    }
}
