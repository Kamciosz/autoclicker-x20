//! PL: Testy kontraktu metadanych wydania i dokumentacji użytkownika.
//! EN: Release metadata and user-documentation contract tests.

use std::{fs, path::PathBuf};

fn repository_file(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name)
}

#[test]
fn package_metadata_is_ready_for_the_first_release() {
    let manifest = fs::read_to_string(repository_file("Cargo.toml")).unwrap();
    assert!(manifest.contains("version = \"0.1.0\""));
    assert!(manifest.contains("license = \"MIT\""));
    assert!(manifest.contains("gpui-kit = \"=0.7.1\""));
}

#[test]
fn readme_documents_the_safety_shortcuts() {
    let readme = fs::read_to_string(repository_file("README.md")).unwrap();
    for shortcut in ["⌘⌥S", "⌘⌥P", "⌘⌥X", "Escape"] {
        assert!(
            readme.contains(shortcut),
            "missing documented shortcut: {shortcut}"
        );
    }
}

#[test]
fn local_documentation_links_and_module_tags_resolve() {
    fn markdown_links(path: &std::path::Path) {
        let text = fs::read_to_string(path).unwrap();
        for suffix in text.split("](").skip(1) {
            let destination = suffix.split(')').next().unwrap();
            if destination.contains("://") {
                continue;
            }
            let destination = destination.split('#').next().unwrap();
            assert!(
                path.parent().unwrap().join(destination).exists(),
                "{} -> {destination}",
                path.display()
            );
        }
    }
    for file in ["README.md", "CONTRIBUTING.md", "CHANGELOG.md"] {
        markdown_links(&repository_file(file));
    }
    for entry in fs::read_dir(repository_file("docs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "md") {
            markdown_links(&path);
        }
    }
    for entry in fs::read_dir(repository_file("src")).unwrap() {
        let text = fs::read_to_string(entry.unwrap().path()).unwrap();
        for line in text
            .lines()
            .filter(|line| line.contains("@uses ") || line.contains("@used_by "))
        {
            let target = line.split_whitespace().last().unwrap();
            let (path, symbol) = target.split_once("::").unwrap();
            let source = fs::read_to_string(repository_file(path)).unwrap();
            assert!(
                source.contains(&format!("struct {symbol}"))
                    || source.contains(&format!("enum {symbol}"))
                    || source.contains(&format!("trait {symbol}"))
                    || source.contains(&format!("fn {symbol}(")),
                "unresolved module tag: {target}"
            );
        }
    }
}
