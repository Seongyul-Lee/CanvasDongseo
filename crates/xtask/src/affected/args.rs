//! `cargo xtask affected`의 인자 해석. 순수 함수만 둔다(I/O 없음).

use super::plan::{Event, Runner};

pub const USAGE: &str = "사용법: cargo xtask affected [옵션]

바뀐 파일을 build_ci §5.2 경로 표로 crate·작업에 대응시키고, 역의존 crate까지의 계획을 출력한다.

옵션:
  --event pr|push        pr(기본): --base-ref와의 merge-base 대비, push: --before 대비
  --base-ref <ref>       pr에서 merge-base를 구할 브랜치(기본 origin/develop)
  --before <sha>         push에서 푸시 전 커밋(github.event.before). 없거나 0000…이거나
                         객체가 없으면(force push 등) 전체를 돌린다
  --base <rev>           이 리비전과 바로 비교한다(--event의 기준보다 우선)
  --runner linux|macos|local
                         명령을 만들 러너(기본 local: Windows 전용 crate만 뺀다)
  --run                  계획한 cargo check·clippy·nextest·doc test를 실행한다.
                         프런트엔드 작업도 실행한다(ui/package.json이 없으면 건너뛴다)
  --skip-frontend        --run에서 프런트엔드 작업을 실행하지 않는다(CI는 따로 돈다)
  --fmt                  바뀐 crate의 cargo fmt --check를 실행한다
  --github-output        rust_linux·rust_macos·frontend를 $GITHUB_OUTPUT에 쓴다";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub event: Event,
    pub base_ref: String,
    pub base: Option<String>,
    pub before: Option<String>,
    pub runner: Runner,
    pub run: bool,
    pub frontend: bool,
    pub fmt: bool,
    pub github_output: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            event: Event::PullRequest,
            base_ref: "origin/develop".to_owned(),
            base: None,
            before: None,
            runner: Runner::Local,
            run: false,
            frontend: true,
            fmt: false,
            github_output: false,
        }
    }
}

pub fn parse(raw: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = raw.iter();
    while let Some(flag) = iter.next() {
        let mut value = || {
            iter.next()
                .cloned()
                .ok_or_else(|| format!("{flag}에 값이 없다"))
        };
        match flag.as_str() {
            "--event" => {
                args.event = match value()?.as_str() {
                    "pr" => Event::PullRequest,
                    "push" => Event::Push,
                    other => return Err(format!("--event는 pr 또는 push다: {other}")),
                }
            }
            "--base-ref" => args.base_ref = value()?,
            "--base" => args.base = Some(value()?),
            "--before" => args.before = Some(value()?),
            "--runner" => {
                args.runner = match value()?.as_str() {
                    "linux" => Runner::Linux,
                    "macos" => Runner::Macos,
                    "local" => Runner::Local,
                    other => {
                        return Err(format!("--runner는 linux, macos, local 중 하나다: {other}"));
                    }
                }
            }
            "--run" => args.run = true,
            "--skip-frontend" => args.frontend = false,
            "--fmt" => args.fmt = true,
            "--github-output" => args.github_output = true,
            other => return Err(format!("알 수 없는 옵션: {other}")),
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(items: &[&str]) -> Result<Args, String> {
        parse(&items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>())
    }

    #[test]
    fn 기본값은_pr_origin_develop_로컬_계획만() {
        assert_eq!(p(&[]).unwrap(), Args::default());
    }

    #[test]
    fn ci_인자를_읽는다() {
        let args = p(&[
            "--event",
            "push",
            "--before",
            "abc",
            "--runner",
            "linux",
            "--run",
            "--skip-frontend",
            "--github-output",
        ])
        .unwrap();
        assert_eq!(args.event, Event::Push);
        assert_eq!(args.before.as_deref(), Some("abc"));
        assert_eq!(args.runner, Runner::Linux);
        assert!(args.run && !args.frontend && args.github_output && !args.fmt);
    }

    #[test]
    fn 비교_기준을_바꾼다() {
        let args = p(&["--base-ref", "origin/main", "--base", "HEAD~3"]).unwrap();
        assert_eq!(args.base_ref, "origin/main");
        assert_eq!(args.base.as_deref(), Some("HEAD~3"));
    }

    #[test]
    fn 잘못된_인자는_오류() {
        assert!(p(&["--event", "tag"]).is_err());
        assert!(p(&["--runner", "windows"]).is_err());
        assert!(p(&["--base"]).is_err());
        assert!(p(&["--before"]).is_err());
        assert!(p(&["--bogus"]).is_err());
    }
}
