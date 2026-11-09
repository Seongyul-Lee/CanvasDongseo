//! 허용 표 대조. 순수 함수만 둔다(I/O 없음). 입력은 `metadata`가 만든 모델, 규칙은 `rules`의 데이터다.

use std::collections::BTreeSet;
use std::fmt;

/// 검사 대상인 워크스페이스 crate 하나와 그 crate가 선언한 의존.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateDeps {
    pub name: String,
    pub deps: Vec<Dep>,
}

/// 선언된 의존 하나. `kind`·`target`이 달라도(dev, build, cfg) 모두 검사한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dep {
    pub name: String,
    /// 워크스페이스 crate면 true, 레지스트리·git 등 외부 crate면 false
    pub workspace: bool,
}

/// 워크스페이스 crate 사이의 허용 대상.
#[derive(Debug, Clone, Copy)]
pub enum Allowed {
    /// 이름이 정확히 같은 crate
    Crate(&'static str),
    /// 이름이 이 접두사로 시작하는 crate (예: `ports-` = `application/ports/*`)
    Prefix(&'static str),
    /// `xtask`를 뺀 모든 워크스페이스 crate (composition root)
    AnyExceptXtask,
}

/// 외부 crate 제한. §3.2는 `domain/*`의 외부 crate만 정한다.
#[derive(Debug, Clone, Copy)]
pub enum External {
    /// 제한하지 않는다(§3.2 표 밖, platform §1에서 정한다)
    Any,
    /// 나열한 것만 허용한다(빈 목록이면 외부 crate 없음)
    Only(&'static [&'static str]),
}

/// crate 하나의 허용 규칙. `doc`은 문서의 어느 행에서 왔는지 보여 준다.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub krate: &'static str,
    pub internal: &'static [Allowed],
    pub external: External,
    pub doc: &'static str,
}

pub const XTASK: &str = "xtask";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Violation {
    /// 허용 표에 행이 없는 워크스페이스 crate
    UnknownCrate { krate: String },
    /// 허용 표에 없는 워크스페이스 crate 의존
    Internal {
        from: String,
        to: String,
        doc: &'static str,
    },
    /// 허용되지 않은 외부 crate 의존
    External {
        from: String,
        to: String,
        doc: &'static str,
    },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::UnknownCrate { krate } => write!(
                f,
                "{krate}: 허용 표에 행이 없다. §3.2 표와 rules.rs에 이 crate의 행을 더한다"
            ),
            Violation::Internal { from, to, doc } => {
                write!(f, "{from} → {to}: 허용되지 않은 워크스페이스 의존 ({doc})")
            }
            Violation::External { from, to, doc } => {
                write!(f, "{from} → {to}: 허용되지 않은 외부 crate ({doc})")
            }
        }
    }
}

/// 모든 crate의 선언된 의존을 허용 표와 대조해 위반을 돌려준다. 위반이 없으면 빈 Vec.
pub fn check(crates: &[CrateDeps], rules: &[Rule]) -> Vec<Violation> {
    // BTreeSet: 같은 의존이 normal·dev·build·cfg로 여러 번 선언돼도 한 번만, 정렬된 순서로 보고한다
    let mut violations = BTreeSet::new();
    for krate in crates {
        let Some(rule) = rules.iter().find(|r| r.krate == krate.name) else {
            violations.insert(Violation::UnknownCrate {
                krate: krate.name.clone(),
            });
            continue;
        };
        for dep in &krate.deps {
            if dep.workspace {
                if !rule.internal.iter().any(|a| a.allows(&dep.name)) {
                    violations.insert(Violation::Internal {
                        from: krate.name.clone(),
                        to: dep.name.clone(),
                        doc: rule.doc,
                    });
                }
            } else if !rule.external.allows(&dep.name) {
                violations.insert(Violation::External {
                    from: krate.name.clone(),
                    to: dep.name.clone(),
                    doc: rule.doc,
                });
            }
        }
    }
    violations.into_iter().collect()
}

impl Allowed {
    fn allows(self, dep: &str) -> bool {
        match self {
            Allowed::Crate(name) => dep == name,
            Allowed::Prefix(prefix) => dep.starts_with(prefix),
            Allowed::AnyExceptXtask => dep != XTASK,
        }
    }
}

impl External {
    fn allows(self, dep: &str) -> bool {
        match self {
            External::Any => true,
            External::Only(names) => names.contains(&dep),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn krate(name: &str, internal: &[&str], external: &[&str]) -> CrateDeps {
        let mut deps: Vec<Dep> = internal
            .iter()
            .map(|n| Dep {
                name: (*n).to_owned(),
                workspace: true,
            })
            .collect();
        deps.extend(external.iter().map(|n| Dep {
            name: (*n).to_owned(),
            workspace: false,
        }));
        CrateDeps {
            name: name.to_owned(),
            deps,
        }
    }

    const TEST_RULES: &[Rule] = &[
        Rule {
            krate: "domain-core",
            internal: &[],
            external: External::Only(&["jiff"]),
            doc: "domain/core",
        },
        Rule {
            krate: "app-kernel",
            internal: &[],
            external: External::Any,
            doc: "application/kernel",
        },
        Rule {
            krate: "ports-school",
            internal: &[Allowed::Crate("app-kernel"), Allowed::Crate("domain-core")],
            external: External::Any,
            doc: "application/ports/*",
        },
        Rule {
            krate: "ports-ai",
            internal: &[Allowed::Crate("app-kernel"), Allowed::Crate("domain-core")],
            external: External::Any,
            doc: "application/ports/*",
        },
        Rule {
            krate: "usecases-school",
            internal: &[
                Allowed::Crate("app-kernel"),
                Allowed::Crate("domain-core"),
                Allowed::Prefix("ports-"),
            ],
            external: External::Any,
            doc: "그 밖의 application/usecases/*",
        },
        Rule {
            krate: "app",
            internal: &[Allowed::AnyExceptXtask],
            external: External::Any,
            doc: "bin/app",
        },
        Rule {
            krate: XTASK,
            internal: &[],
            external: External::Any,
            doc: "xtask",
        },
    ];

    #[test]
    fn 허용된_의존만_있으면_위반이_없다() {
        let crates = [
            krate("domain-core", &[], &["jiff"]),
            krate("app-kernel", &[], &["thiserror"]),
            krate("ports-school", &["app-kernel", "domain-core"], &[]),
            krate(
                "usecases-school",
                &["app-kernel", "ports-school", "ports-ai"],
                &[],
            ),
            krate(
                "app",
                &["usecases-school", "ports-school", "domain-core"],
                &["tauri"],
            ),
            krate(XTASK, &[], &["serde_json"]),
        ];
        assert_eq!(check(&crates, TEST_RULES), vec![]);
    }

    #[test]
    fn 표에_없는_워크스페이스_의존을_찾는다() {
        // 위반을 일부러 넣는다: 포트 crate끼리 의존(§3.2 "포트 crate끼리 서로 의존하지 않는다")
        let crates = [krate("ports-school", &["app-kernel", "ports-ai"], &[])];
        assert_eq!(
            check(&crates, TEST_RULES),
            vec![Violation::Internal {
                from: "ports-school".into(),
                to: "ports-ai".into(),
                doc: "application/ports/*",
            }]
        );
    }

    #[test]
    fn 안쪽_레이어가_바깥을_알면_위반이다() {
        let crates = [krate("domain-core", &["usecases-school"], &[])];
        assert_eq!(
            check(&crates, TEST_RULES),
            vec![Violation::Internal {
                from: "domain-core".into(),
                to: "usecases-school".into(),
                doc: "domain/core",
            }]
        );
    }

    #[test]
    fn 허용되지_않은_외부_crate를_찾는다() {
        let crates = [krate("domain-core", &[], &["jiff", "tokio"])];
        assert_eq!(
            check(&crates, TEST_RULES),
            vec![Violation::External {
                from: "domain-core".into(),
                to: "tokio".into(),
                doc: "domain/core",
            }]
        );
    }

    #[test]
    fn 외부_제한이_없는_crate는_외부_의존을_보지_않는다() {
        let crates = [krate("ports-school", &["app-kernel"], &["tokio", "tauri"])];
        assert_eq!(check(&crates, TEST_RULES), vec![]);
    }

    #[test]
    fn 허용_표에_행이_없는_crate는_위반이다() {
        let crates = [krate("brand-new", &[], &[])];
        assert_eq!(
            check(&crates, TEST_RULES),
            vec![Violation::UnknownCrate {
                krate: "brand-new".into()
            }]
        );
    }

    #[test]
    fn composition_root도_xtask에는_의존하지_못한다() {
        let crates = [krate("app", &["domain-core", XTASK], &[])];
        assert_eq!(
            check(&crates, TEST_RULES),
            vec![Violation::Internal {
                from: "app".into(),
                to: XTASK.into(),
                doc: "bin/app",
            }]
        );
    }

    #[test]
    fn xtask는_앱_crate에_의존하지_못한다() {
        let crates = [krate(XTASK, &["domain-core"], &[])];
        assert_eq!(
            check(&crates, TEST_RULES),
            vec![Violation::Internal {
                from: XTASK.into(),
                to: "domain-core".into(),
                doc: "xtask",
            }]
        );
    }

    #[test]
    fn 같은_의존이_여러_kind로_선언돼도_한_번만_보고한다() {
        // normal과 dev에 같은 crate를 선언하면 metadata에 두 번 나온다
        let crates = [krate("domain-core", &["app-kernel", "app-kernel"], &[])];
        assert_eq!(check(&crates, TEST_RULES).len(), 1);
    }
}
