//! `cargo metadata --no-deps --format-version 1` JSON → 영향 범위 계산용 모델(`Workspace`) 변환.
//! 경계에서 외부 형식을 한 번만 해석하고, 계산 로직(`graph`·`plan`)은 JSON을 모르게 한다.

use std::collections::BTreeSet;

use serde_json::Value;

use super::graph::{Member, Workspace};

/// doc test가 도는 lib 계열 대상 종류
const LIB_KINDS: &[&str] = &["lib", "rlib", "dylib", "proc-macro"];

pub fn parse(json: &str) -> Result<Workspace, String> {
    let root: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let workspace_root = root["workspace_root"]
        .as_str()
        .ok_or("workspace_root가 없다")?
        .replace('\\', "/");
    let members: BTreeSet<&str> = root["workspace_members"]
        .as_array()
        .ok_or("workspace_members가 없다")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let packages: Vec<&Value> = root["packages"]
        .as_array()
        .ok_or("packages가 없다")?
        .iter()
        .filter(|p| p["id"].as_str().is_some_and(|id| members.contains(id)))
        .collect();
    let names: BTreeSet<&str> = packages.iter().filter_map(|p| p["name"].as_str()).collect();

    let members = packages
        .iter()
        .map(|p| {
            let name = p["name"].as_str().ok_or("패키지 name이 없다")?;
            let manifest = p["manifest_path"]
                .as_str()
                .ok_or_else(|| format!("{name}: manifest_path가 없다"))?
                .replace('\\', "/");
            let dir = manifest
                .strip_prefix(&workspace_root)
                .and_then(|rest| rest.strip_suffix("/Cargo.toml"))
                .map(|rest| rest.trim_start_matches('/').to_owned())
                .ok_or_else(|| format!("{name}: {manifest}가 워크스페이스 루트 아래가 아니다"))?;
            let has_lib = p["targets"]
                .as_array()
                .ok_or_else(|| format!("{name}: targets가 없다"))?
                .iter()
                .filter_map(|t| t["kind"].as_array())
                .flatten()
                .filter_map(Value::as_str)
                .any(|kind| LIB_KINDS.contains(&kind));
            let mut deps: Vec<String> = p["dependencies"]
                .as_array()
                .ok_or_else(|| format!("{name}: dependencies가 없다"))?
                .iter()
                // 레지스트리·git 의존은 source가 있다. 경로 의존 중 워크스페이스 멤버만 내부 의존이다
                .filter(|d| d["source"].is_null())
                .filter_map(|d| d["name"].as_str())
                .filter(|dep| names.contains(dep))
                .map(str::to_owned)
                .collect();
            deps.sort();
            deps.dedup();
            Ok(Member {
                name: name.to_owned(),
                dir,
                has_lib,
                deps,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Workspace { members })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "workspace_root": "/w",
      "workspace_members": ["path+file:///w/crates/a#0.1.0", "path+file:///w/crates/bin/b#0.1.0"],
      "packages": [
        {
          "name": "a",
          "id": "path+file:///w/crates/a#0.1.0",
          "manifest_path": "/w/crates/a/Cargo.toml",
          "targets": [{"kind": ["lib"]}, {"kind": ["test"]}],
          "dependencies": [
            {"name": "jiff", "source": "registry+https://github.com/rust-lang/crates.io-index"}
          ]
        },
        {
          "name": "b",
          "id": "path+file:///w/crates/bin/b#0.1.0",
          "manifest_path": "/w/crates/bin/b/Cargo.toml",
          "targets": [{"kind": ["bin"]}, {"kind": ["custom-build"]}],
          "dependencies": [
            {"name": "a", "source": null, "kind": null},
            {"name": "a", "source": null, "kind": "dev"}
          ]
        }
      ]
    }"#;

    #[test]
    fn crate_폴더_lib_여부_내부_의존을_읽는다() {
        assert_eq!(
            parse(SAMPLE).unwrap(),
            Workspace {
                members: vec![
                    Member {
                        name: "a".into(),
                        dir: "crates/a".into(),
                        has_lib: true,
                        deps: vec![],
                    },
                    Member {
                        name: "b".into(),
                        dir: "crates/bin/b".into(),
                        has_lib: false,
                        deps: vec!["a".into()],
                    },
                ]
            }
        );
    }

    #[test]
    fn 형식이_다르면_오류를_돌려준다() {
        assert!(parse("{}").is_err());
        assert!(parse("not json").is_err());
    }
}
