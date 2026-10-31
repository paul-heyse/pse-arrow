// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded, best-effort suggestions. Never a permission or execution boundary.
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::Path,
};

const LIMIT: usize = 256 * 1024;

fn literal(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    let value = node.utf8_text(source.as_bytes()).ok()?;
    if value.contains(['$', '`', '\\']) {
        return None;
    }
    Some(value.trim_matches(['\'', '"']).to_owned())
}

fn route(words: &[String]) -> Option<&'static str> {
    let (program, args) = words.split_first()?;
    let program = Path::new(program).file_name()?.to_str()?;
    match program {
        "env" => {
            let mut start = 0;
            while let Some(arg) = args.get(start) {
                if arg == "--" || (!arg.starts_with('-') && arg.contains('=')) {
                    start += 1;
                } else if arg == "-u" || arg == "--unset" {
                    start += 2;
                } else {
                    break;
                }
            }
            route(args.get(start..)?)
        }
        "command" | "exec" => route(args.strip_prefix(&["--".into()]).unwrap_or(args)),
        "timeout" if args.first().is_some_and(|arg| !arg.starts_with('-')) => route(args.get(1..)?),
        "pse-env" => route(args.get(args.iter().position(|arg| arg == "--")? + 1..)?),
        "cargo" | "cargo-pse-correctness" => match args.first()?.as_str() {
            "test" | "t" => Some("pse test"),
            "check" | "c" => Some("pse check"),
            "nextest" => match args.get(1)?.as_str() {
                "run" => Some("pse test"),
                "list" => Some("pse tests"),
                _ => None,
            },
            "clippy" => Some("pse run-all clippy-default clippy-no-default (at scope end)"),
            "fmt" => Some("pse run-all adr-index fmt (at turn end)"),
            "doc" => Some(
                "pse api for API lookup, or pse run-all docs-rust for workspace documentation checks",
            ),
            "rustc" => Some(
                "pse compiler generate or pse compiler probe for explicit compiler investigations",
            ),
            "build" => {
                Some("pse run -- cargo build for managed builds; pse check for compilation checks")
            }
            "run" if args.windows(2).any(|pair| pair == ["-p", "xtask"]) => Some(
                "pse owner / pse docs check for those workflows, or pse run for other managed xtask work",
            ),
            _ => None,
        },
        "rustc" | "rustdoc" => Some(
            "pse compiler / pse api for investigations, or pse run -- TOOL for managed execution",
        ),
        "rust-analyzer" => Some("pse code session prepare for an owned reusable semantic session"),
        "xtask" => Some(
            "pse owner / pse docs check for those workflows, or pse run -- xtask for other managed work",
        ),
        _ => None,
    }
}

fn collect(
    node: tree_sitter::Node<'_>,
    source: &str,
    root: &Path,
    routes: &mut BTreeSet<&'static str>,
) -> Option<()> {
    // Definitions and unevaluated script data are not executed commands.
    if node.kind() == "function_definition" {
        return None;
    }
    if matches!(node.kind(), "heredoc_body" | "comment") {
        return Some(());
    }
    if node.kind() == "command" {
        let name = node.child_by_field_name("name")?;
        let Some(program) = literal(name, source) else {
            return Some(());
        };
        // A directory-changing script requires shell execution knowledge we do not infer.
        if matches!(
            program.as_str(),
            "cd" | "pushd" | "popd" | "eval" | "alias" | "unalias" | "source" | "." | "enable"
        ) {
            return None;
        }
        let mut words = vec![program];
        let mut cursor = node.walk();
        for argument in node.children_by_field_name("argument", &mut cursor) {
            words.push(literal(argument, source)?);
        }
        if matches!(words[0].as_str(), "command" | "builtin" | "exec")
            && route(&words).is_none()
        { return None; }
        if words
            .iter()
            .any(|w| matches!(w.as_str(), "--help" | "-h" | "--version" | "-V"))
        {
            return Some(());
        }
        for (index, word) in words.iter().enumerate() {
            if word == "--manifest-path" || word.starts_with("--manifest-path=") {
                let value = if word == "--manifest-path" {
                    words.get(index + 1)?.as_str()
                } else {
                    word.strip_prefix("--manifest-path=")?
                };
                if value.is_empty() {
                    return None;
                }
                let path = Path::new(value);
                // Relative manifests depend on shell cwd; leave their custom selection alone.
                if !path.is_absolute() || !path.canonicalize().ok()?.starts_with(root) {
                    return None;
                }
            }
        }
        if let Some(route) = route(&words) {
            routes.insert(route);
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect(child, source, root, routes)?;
    }
    Some(())
}

fn advice(source: &str, root: &Path) -> Option<String> {
    if source.len() > LIMIT {
        return None;
    }
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_bash::LANGUAGE.into())
        .ok()?;
    let tree = parser.parse(source, None)?;
    if tree.root_node().has_error() {
        return None;
    }
    let mut routes = BTreeSet::new();
    collect(tree.root_node(), source, root, &mut routes)?;
    if routes.is_empty() {
        return None;
    }
    let selected = routes.into_iter().take(3).collect::<Vec<_>>().join("; ");
    Some(format!(
        "Prefer the native CLI for repository tooling: {selected}. It supplies the applicable execution ownership, correctness/fixture selection and retained results. Use the pse-cli skill and pse commands for the appropriate route. This direct invocation is proceeding unchanged."
    ))
}

pub(super) fn execute() -> Result<i32, crate::HostError> {
    // Advice must remain quiet on malformed, oversized or unavailable input.
    let result = (|| -> Option<()> {
        let mut bytes = Vec::new();
        std::io::stdin()
            .take((LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .ok()?;
        if bytes.len() > LIMIT {
            return None;
        }
        let payload: Value = serde_json::from_slice(&bytes).ok()?;
        let source = payload
            .pointer("/tool_input/command")
            .or_else(|| payload.pointer("/tool_input/cmd"))?
            .as_str()?;
        let cwd = payload
            .pointer("/tool_input/workdir")
            .and_then(Value::as_str)
            .or_else(|| payload.get("cwd").and_then(Value::as_str))?;
        let cwd = Path::new(cwd).canonicalize().ok()?;
        let root = super::session_root(&cwd, &serde_json::json!({"cwd":cwd})).ok()?;
        if !root.join("scripts/pse").is_file() || !cwd.starts_with(&root) {
            return None;
        }
        let context = advice(source, &root)?;
        let output = serde_json::json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":context}});
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, &output).ok()?;
        writeln!(stdout).ok()?;
        Some(())
    })();
    let _ = result;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn executable_structure_drives_advice_without_matching_data() {
        for source in [
            "cargo test -p pse-host",
            "X=y /usr/bin/cargo check",
            "env X=y cargo nextest list",
            "scripts/pse-env -- cargo clippy",
            "cargo check && cargo test | cat",
        ] {
            assert!(advice(source, Path::new("/repo")).is_some(), "{source}");
        }
        for source in [
            "echo 'cargo test'",
            "# cargo test",
            "cat <<'EOF'\ncargo test\nEOF",
            "pse test pse-host",
            "just hygiene",
            "cargo --version",
            "cargo test --help",
            "rg 'cargo test' .",
            "cd /other && cargo test",
            "cargo test --manifest-path /other/Cargo.toml",
            "cargo test --manifest-path=/other/Cargo.toml",
            "env --chdir=/other cargo test",
            "cargo test --manifest-path=\"$OTHER_MANIFEST\"",
            "command cd /other && cargo test",
            "builtin cd /other && cargo test",
            "cargo() { :; }; cargo test",
            "function later() { cargo test; }",
            "cargo test '",
        ] {
            assert!(advice(source, Path::new("/repo")).is_none(), "{source}");
        }
    }
}
