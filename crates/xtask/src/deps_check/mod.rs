//! `cargo xtask deps-check`: `cargo metadata`를 System Architecture §3.2 의존 표와 대조한다(build_ci §5.4).
//!
//! - `rules`: §3.2 표를 옮긴 허용 표(데이터)
//! - `check`: 허용 표 대조(순수 함수, I/O 없음)
//! - `metadata`: `cargo metadata` JSON → 검사용 모델 변환
//! - 이 파일의 `run`: `cargo metadata` 실행과 결과 출력(I/O 조율)

mod check;
mod metadata;
mod rules;

use std::process::{Command, ExitCode};

pub fn run() -> ExitCode {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = match Command::new(cargo)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!("deps-check: cargo metadata를 실행하지 못했다: {err}");
            return ExitCode::FAILURE;
        }
    };
    if !output.status.success() {
        eprintln!(
            "deps-check: cargo metadata가 실패했다\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return ExitCode::FAILURE;
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let crates = match metadata::parse(&json) {
        Ok(crates) => crates,
        Err(err) => {
            eprintln!("deps-check: cargo metadata 출력을 읽지 못했다: {err}");
            return ExitCode::FAILURE;
        }
    };

    let violations = check::check(&crates, rules::RULES);
    if violations.is_empty() {
        println!(
            "deps-check: 워크스페이스 crate {}개가 System Architecture §3.2 의존 표를 지킨다",
            crates.len()
        );
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "deps-check: System Architecture §3.2 의존 표 위반 {}건",
        violations.len()
    );
    for violation in &violations {
        eprintln!("  - {violation}");
    }
    eprintln!(
        "\n의존을 레이어에 맞게 고친다. 표 자체가 바뀌어야 하면 docs/02_system_architecture.md §3.2와 crates/xtask/src/deps_check/rules.rs를 함께 고친다."
    );
    ExitCode::FAILURE
}
