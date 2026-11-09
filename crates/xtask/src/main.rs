//! [tool] 의존 그래프 검사, 영향 범위 계산. 앱 crate에 의존하지 않는다
//!
//! 레이어와 의존 규칙: docs/02_system_architecture.md §3.1·§3.2
//! 진입점: 서브커맨드 이름을 읽어 해당 모듈의 `run`으로 넘기기만 한다.

mod affected;
mod deps_check;

use std::process::ExitCode;

const USAGE: &str = "사용법: cargo xtask <서브커맨드>\n\n서브커맨드:\n  deps-check   cargo metadata를 System Architecture §3.2 의존 표와 대조한다\n  affected     바뀐 파일의 영향 범위(역의존 crate·작업)만 계산해 돌린다(cargo xtask affected --help)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("deps-check") => deps_check::run(),
        Some("affected") => affected::run(&args[1..]),
        Some(other) => {
            eprintln!("알 수 없는 서브커맨드: {other}\n\n{USAGE}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
