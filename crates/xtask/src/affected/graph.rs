//! 워크스페이스 모델과 역의존 계산. 순수 함수만 둔다(I/O 없음). 입력은 `metadata`가 만든다.

use std::collections::BTreeSet;

/// 워크스페이스 crate 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// 패키지 이름(`cargo -p`에 넘기는 이름)
    pub name: String,
    /// 워크스페이스 루트 기준 crate 폴더(`/` 구분, 끝 `/` 없음). 예: `crates/application/ports/school`
    pub dir: String,
    /// lib 대상이 있으면 true(doc test는 lib 대상만 돈다)
    pub has_lib: bool,
    /// 선언한 워크스페이스 내부 의존(normal·dev·build 모두, 패키지 이름)
    pub deps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Workspace {
    pub members: Vec<Member>,
}

impl Workspace {
    pub fn get(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    pub fn by_dir(&self, dir: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.dir == dir)
    }

    /// `path`를 담은 crate. crate 폴더가 겹치면 가장 깊은(긴) 폴더를 고른다.
    pub fn owner_of(&self, path: &str) -> Option<&Member> {
        self.members
            .iter()
            .filter(|m| {
                path.strip_prefix(m.dir.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
            })
            .max_by_key(|m| m.dir.len())
    }

    pub fn names(&self) -> BTreeSet<String> {
        self.members.iter().map(|m| m.name.clone()).collect()
    }

    /// `seeds`와 그것에 (간접으로라도) 의존하는 crate 전부. dev·build 의존도 따라간다
    /// (의존하는 쪽의 테스트·빌드 스크립트가 다시 컴파일되므로).
    pub fn reverse_closure(&self, seeds: &BTreeSet<String>) -> BTreeSet<String> {
        let mut found: BTreeSet<String> = seeds.clone();
        let mut frontier: Vec<String> = seeds.iter().cloned().collect();
        while let Some(current) = frontier.pop() {
            for member in &self.members {
                if !found.contains(&member.name) && member.deps.contains(&current) {
                    found.insert(member.name.clone());
                    frontier.push(member.name.clone());
                }
            }
        }
        found
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn member(name: &str, dir: &str, deps: &[&str]) -> Member {
        Member {
            name: name.into(),
            dir: dir.into(),
            has_lib: true,
            deps: deps.iter().map(|d| (*d).into()).collect(),
        }
    }

    /// 실제 구조를 줄인 표본: core ← kernel ← usecases ← presenters ← tauri ← app, cdx는 따로
    pub fn sample() -> Workspace {
        Workspace {
            members: vec![
                member("domain-core", "crates/domain/core", &[]),
                member("app-kernel", "crates/application/kernel", &["domain-core"]),
                member(
                    "usecases-school",
                    "crates/application/usecases/school",
                    &["app-kernel"],
                ),
                member(
                    "presenters-school",
                    "crates/presentation/presenters/school",
                    &["usecases-school"],
                ),
                member(
                    "presenters-common",
                    "crates/presentation/presenters/common",
                    &[],
                ),
                member(
                    "tauri-commands",
                    "crates/presentation/tauri_commands",
                    &["presenters-school"],
                ),
                member("platform-keychain", "crates/platform/keychain", &[]),
                member("learning-macos", "crates/learning/macos", &[]),
                member("learning-windows", "crates/learning/windows", &[]),
                member("ipc-cli", "crates/ipc/cli", &[]),
                member("cdx", "crates/bin/cdx", &["ipc-cli"]),
                member(
                    "app",
                    "crates/bin/app",
                    &["tauri-commands", "ipc-cli", "platform-keychain"],
                ),
                member("xtask", "crates/xtask", &[]),
            ],
        }
    }

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).into()).collect()
    }

    #[test]
    fn 역의존을_간접까지_모은다() {
        let ws = sample();
        assert_eq!(
            ws.reverse_closure(&set(&["usecases-school"])),
            set(&[
                "usecases-school",
                "presenters-school",
                "tauri-commands",
                "app"
            ])
        );
    }

    #[test]
    fn 아무도_의존하지_않는_crate는_자기만_돈다() {
        assert_eq!(
            sample().reverse_closure(&set(&["presenters-common"])),
            set(&["presenters-common"])
        );
    }

    #[test]
    fn 여러_crate의_역의존을_합친다() {
        assert_eq!(
            sample().reverse_closure(&set(&["ipc-cli", "platform-keychain"])),
            set(&["ipc-cli", "cdx", "app", "platform-keychain"])
        );
    }

    #[test]
    fn 경로의_crate는_가장_깊은_폴더로_고른다() {
        let mut ws = sample();
        ws.members
            .push(member("nested", "crates/bin/app/nested", &[]));
        assert_eq!(
            ws.owner_of("crates/bin/app/nested/src/lib.rs")
                .unwrap()
                .name,
            "nested"
        );
        assert_eq!(
            ws.owner_of("crates/bin/app/src/main.rs").unwrap().name,
            "app"
        );
        // 폴더 이름의 접두사만 같은 경로는 그 crate가 아니다
        assert!(ws.owner_of("crates/bin/application/x.rs").is_none());
        assert!(ws.owner_of("crates/bin/app").is_none());
    }
}
