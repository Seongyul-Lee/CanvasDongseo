//! `cargo metadata --no-deps --format-version 1` JSON → 검사용 모델(`CrateDeps`) 변환.
//! 경계에서 외부 형식을 한 번만 해석하고, 대조 로직(`check`)은 JSON을 모르게 한다.

use std::collections::BTreeSet;

use serde_json::Value;

use super::check::{CrateDeps, Dep};

/// 워크스페이스 멤버마다 선언된 의존(normal·dev·build, cfg 대상 포함)을 모은다.
/// 의존 이름은 패키지 이름이다(`package = "..."`로 바꿔 부른 경우에도 metadata의 `name`은 패키지 이름).
pub fn parse(json: &str) -> Result<Vec<CrateDeps>, String> {
    let root: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let members: BTreeSet<&str> = root["workspace_members"]
        .as_array()
        .ok_or("workspace_members가 없다")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let packages = root["packages"].as_array().ok_or("packages가 없다")?;

    let workspace: Vec<&Value> = packages
        .iter()
        .filter(|p| p["id"].as_str().is_some_and(|id| members.contains(id)))
        .collect();
    let names: BTreeSet<&str> = workspace
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();

    workspace
        .iter()
        .map(|p| {
            let name = p["name"].as_str().ok_or("패키지 name이 없다")?;
            let deps = p["dependencies"]
                .as_array()
                .ok_or_else(|| format!("{name}: dependencies가 없다"))?
                .iter()
                .map(|d| {
                    let dep = d["name"]
                        .as_str()
                        .ok_or_else(|| format!("{name}: 의존 name이 없다"))?;
                    // 레지스트리·git 의존은 source가 있다. 경로 의존 중 워크스페이스 멤버만 내부 의존이다
                    let workspace = d["source"].is_null() && names.contains(dep);
                    Ok(Dep {
                        name: dep.to_owned(),
                        workspace,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(CrateDeps {
                name: name.to_owned(),
                deps,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "workspace_members": ["path+file:///w/a#0.1.0", "path+file:///w/b#0.1.0"],
      "packages": [
        {
          "name": "a",
          "id": "path+file:///w/a#0.1.0",
          "dependencies": [
            {"name": "b", "source": null, "kind": null, "target": null, "path": "/w/b"},
            {"name": "jiff", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": null, "target": null},
            {"name": "b", "source": null, "kind": "dev", "target": "cfg(unix)", "path": "/w/b"}
          ]
        },
        {"name": "b", "id": "path+file:///w/b#0.1.0", "dependencies": []}
      ]
    }"#;

    #[test]
    fn 워크스페이스_의존과_외부_의존을_구분한다() {
        let crates = parse(SAMPLE).unwrap();
        assert_eq!(
            crates,
            vec![
                CrateDeps {
                    name: "a".into(),
                    deps: vec![
                        Dep {
                            name: "b".into(),
                            workspace: true
                        },
                        Dep {
                            name: "jiff".into(),
                            workspace: false
                        },
                        Dep {
                            name: "b".into(),
                            workspace: true
                        },
                    ],
                },
                CrateDeps {
                    name: "b".into(),
                    deps: vec![]
                },
            ]
        );
    }

    #[test]
    fn 형식이_다르면_오류를_돌려준다() {
        assert!(parse("{}").is_err());
        assert!(parse("not json").is_err());
    }
}
