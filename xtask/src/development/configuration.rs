// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit local agent configuration and delegation to the shared library selector.
use anyhow::{Context, Result, ensure};
use regex::Regex;
use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

const ALIASES: [&str; 2] = [".claude", ".agents"];
const LINK_TARGET: &str = "../.codex/skills";
const ROLES: [&str; 6] = [
    "code-mapper",
    "library-research",
    "design-reviewer",
    "executor",
    "implementation-reviewer",
    "test-agent",
];

fn metadata(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
fn normalized(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            _ => out.push(part),
        }
    }
    out
}
fn link_destination(path: &Path) -> Result<PathBuf> {
    let target = fs::read_link(path)?;
    let target = if target.is_absolute() {
        target
    } else {
        path.parent().context("link parent")?.join(target)
    };
    Ok(target
        .canonicalize()
        .unwrap_or_else(|_| normalized(&target)))
}
fn entries(path: &Path) -> Result<BTreeSet<OsString>> {
    fs::read_dir(path)?
        .map(|entry| Ok(entry?.file_name()))
        .collect()
}
fn same_tree(left: &Path, right: &Path) -> Result<bool> {
    let (Some(left_meta), Some(right_meta)) = (metadata(left)?, metadata(right)?) else {
        return Ok(false);
    };
    if left_meta.is_symlink() || right_meta.is_symlink() {
        return Ok(left_meta.is_symlink()
            && right_meta.is_symlink()
            && link_destination(left)? == link_destination(right)?);
    }
    if left_meta.is_dir() && right_meta.is_dir() {
        let names = entries(left)?;
        if names != entries(right)? {
            return Ok(false);
        }
        for name in names {
            if !same_tree(&left.join(&name), &right.join(name))? {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    Ok(left_meta.is_file() && right_meta.is_file() && fs::read(left)? == fs::read(right)?)
}

// Preflight every destination before any alias changes. Local edits and unrelated
// links require explicit resolution; synchronizing aliases never overwrites them.
fn compatible_copy(source: &Path, destination: &Path) -> Result<()> {
    let Some(existing) = metadata(destination)? else {
        return Ok(());
    };
    let original = fs::symlink_metadata(source)?;
    if original.is_dir() && !original.is_symlink() && existing.is_dir() && !existing.is_symlink() {
        let names = entries(source)?;
        for name in entries(destination)? {
            ensure!(
                names.contains(&name),
                "refusing unrelated alias content: {}",
                destination.join(name).display()
            );
        }
        for name in names {
            compatible_copy(&source.join(&name), &destination.join(name))?;
        }
    } else {
        ensure!(
            same_tree(source, destination)?,
            "refusing conflicting alias content: {}",
            destination.display()
        );
    }
    Ok(())
}
#[cfg(unix)]
fn symlink(target: &Path, link: &Path, _directory: bool) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}
#[cfg(windows)]
fn symlink(target: &Path, link: &Path, directory: bool) -> std::io::Result<()> {
    if directory {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}
fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    let original = fs::symlink_metadata(source)?;
    if original.is_symlink() {
        if metadata(destination)?.is_none() {
            symlink(&fs::read_link(source)?, destination, source.is_dir())?;
        }
    } else if original.is_dir() {
        fs::create_dir_all(destination)?;
        for name in entries(source)? {
            copy_tree(&source.join(&name), &destination.join(name))?;
        }
    } else if metadata(destination)?.is_none() {
        fs::copy(source, destination)?;
    }
    Ok(())
}

/// Synchronize only the two skill aliases, or report drift without writing.
/// Native adapters and shared library bundle contents retain their original owners.
pub fn synchronize(root: &Path, check: bool) -> Result<Vec<String>> {
    let root = root.canonicalize()?;
    let target = root.join(".codex/skills");
    ensure!(
        target.is_dir(),
        ".codex/skills must exist before materializing aliases"
    );
    let mut problems = Vec::new();
    for directory in ALIASES {
        let parent = root.join(directory);
        if let Some(meta) = metadata(&parent)? {
            ensure!(
                meta.is_dir() && !meta.is_symlink(),
                "refusing unexpected alias parent: {}",
                parent.display()
            );
        }
        let alias = parent.join("skills");
        let alias_meta = metadata(&alias)?;
        if check {
            let matches = if alias_meta.as_ref().is_some_and(fs::Metadata::is_symlink) {
                link_destination(&alias)? == target.canonicalize()?
            } else {
                same_tree(&alias, &target)?
            };
            if !matches {
                problems.push(format!(
                    "skill alias drift: {directory}/skills; run just agent-config-sync"
                ));
            }
        } else if let Some(meta) = alias_meta {
            if meta.is_symlink() {
                ensure!(
                    link_destination(&alias)? == target.canonicalize()?,
                    "unexpected skill symlink: {}",
                    alias.display()
                );
            } else if meta.is_file() {
                ensure!(
                    fs::read_to_string(&alias)?.trim() == LINK_TARGET,
                    "refusing to replace {}",
                    alias.display()
                );
            } else {
                compatible_copy(&target, &alias)?;
            }
        }
    }
    if !check {
        for directory in ALIASES {
            let alias = root.join(directory).join("skills");
            fs::create_dir_all(alias.parent().context("alias parent")?)?;
            if metadata(&alias)?.is_some_and(|value| value.is_symlink()) {
                continue;
            }
            if metadata(&alias)?.is_some_and(|value| value.is_file()) {
                fs::remove_file(&alias)?;
            }
            if metadata(&alias)?.is_none() && symlink(Path::new(LINK_TARGET), &alias, true).is_ok()
            {
                continue;
            }
            copy_tree(&target, &alias)?;
        }
    }
    Ok(problems)
}

/// Build the shared tool invocation only for an explicit sync/check request.
/// Its manifest, selection validation and conflict policy remain in manage.py.
pub fn library_skills_command(root: &Path, check: bool) -> Result<Command> {
    let environment = std::env::var_os("UV_PROJECT_ENVIRONMENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".venv"));
    let environment = if environment.is_absolute() {
        environment
    } else {
        root.join(environment)
    };
    library_command(
        root,
        check,
        std::env::var_os("LIBRARY_SKILLS_ROOT"),
        std::env::var_os("HOME"),
        Some(environment.join("bin/python").into_os_string()),
    )
}
fn library_command(
    root: &Path,
    check: bool,
    selected: Option<OsString>,
    home: Option<OsString>,
    python: Option<OsString>,
) -> Result<Command> {
    let root = root.canonicalize()?;
    let store = match selected {
        Some(value) if value == "~" => PathBuf::from(home.context("HOME is required for ~")?),
        Some(value) if value.to_str().is_some_and(|value| value.starts_with("~/")) => {
            PathBuf::from(home.context("HOME is required for ~/ library selection")?)
                .join(Path::new(&value).strip_prefix("~")?)
        }
        Some(value) => PathBuf::from(value),
        None => PathBuf::from(home.context("HOME or LIBRARY_SKILLS_ROOT is required")?)
            .join(".local/share/library-skills"),
    };
    let manager = store.join("manage.py");
    ensure!(
        manager.is_file(),
        "shared library skills are not installed at {}; set LIBRARY_SKILLS_ROOT if moved",
        store.display()
    );
    let mut command =
        Command::new(python.unwrap_or_else(|| root.join(".venv/bin/python").into_os_string()));
    command
        .arg(manager)
        .arg("--repo")
        .arg(&root)
        .current_dir(&root);
    if check {
        command.arg("--check");
    }
    Ok(command)
}

/// Collected independent configuration failures; an unavailable check is a failure.
#[derive(Debug, Default)]
pub struct Check {
    /// Independent failures against the zero-error target.
    pub problems: Vec<String>,
    /// Repository-owned instruction documents inspected.
    pub files: usize,
    /// Named concrete repository paths inspected.
    pub path_references: usize,
    /// Named recipe references inspected against the actual just inventory.
    pub recipe_references: usize,
    /// File-path deny rules inspected.
    pub deny_rules: usize,
}
fn direct_files(directory: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.path().extension() == Some(OsStr::new(extension)) {
            out.push(entry.path());
        }
    }
    out.sort();
    Ok(out)
}
fn local_markdown(directory: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            local_markdown(&entry.path(), out)?;
        } else if kind.is_file() && entry.path().extension() == Some(OsStr::new("md")) {
            out.push(entry.path());
        }
    }
    Ok(())
}
fn scanned_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = ["AGENTS.md", "CLAUDE.md", ".codex/skills/README.md"]
        .map(|name| root.join(name))
        .into_iter()
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    for (directory, extension) in [
        (".claude/rules", "md"),
        (".claude/agents", "md"),
        (".codex/agents", "toml"),
        (".agents/roles", "md"),
    ] {
        files.extend(direct_files(&root.join(directory), extension)?);
    }
    // Discover repository-owned process skills; never enter external bundle links.
    if root.join(".codex/skills").is_dir() {
        local_markdown(&root.join(".codex/skills"), &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}
fn body(text: &str) -> &str {
    if let Some(front) = text.strip_prefix("---\n")
        && let Some((_, rest)) = front.split_once("\n---\n")
    {
        rest
    } else {
        text
    }
}
fn path_references(text: &str) -> Result<BTreeSet<String>> {
    let pattern = Regex::new(
        r"(?:^|[^\w./-])((?:docs|scripts|crates|python|tests|xtask|benches|\.github|\.claude|\.codex|\.agents)/[A-Za-z0-9_.*{}<>?-][A-Za-z0-9_./*{}<>?-]*)",
    )?;
    Ok(text
        .lines()
        .filter(|line| !line.contains("agent-config: ignore"))
        .flat_map(|line| {
            pattern
                .captures_iter(line)
                .map(|capture| {
                    capture[1]
                        .trim_end_matches(['.', ',', ';', ':', ')', ']', '`', '\'', '"', '/'])
                        .to_owned()
                })
                .collect::<Vec<_>>()
        })
        .filter(|path| !path.chars().any(|character| "*{}<>?".contains(character)))
        .collect())
}
fn recipe_references(text: &str) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let ticks = Regex::new(r"`([^`\n]+)`")?;
    for capture in ticks.captures_iter(text) {
        if let Some(command) = capture[1].trim().strip_prefix("just ")
            && let Some(name) = command.split_whitespace().next()
            && !name.starts_with('-')
        {
            names.insert(name.into());
        }
    }
    for line in text.lines() {
        if let Some(command) = line.trim_start().strip_prefix("just ")
            && let Some(name) = command.split_whitespace().next()
            && !name.starts_with('-')
        {
            names.insert(name.into());
        }
    }
    Ok(names)
}
fn check_adapters(root: &Path, problems: &mut Vec<String>) -> Result<()> {
    for role in ROLES {
        for (runtime, suffix, name) in [
            (".codex", "toml", role),
            (
                ".claude",
                "md",
                if role == "executor" {
                    "implementer"
                } else {
                    role
                },
            ),
        ] {
            let relative = format!("{runtime}/agents/{name}.{suffix}");
            let path = root.join(&relative);
            if !path.is_file() {
                problems.push(format!("missing native adapter: {relative}"));
                continue;
            }
            let text = fs::read_to_string(path)?;
            for contract in [
                ".agents/roles/worker.md".to_owned(),
                format!(".agents/roles/{role}.md"),
            ] {
                if !text.contains(&contract) || !root.join(&contract).is_file() {
                    problems.push(format!("{relative}: missing shared contract {contract}"));
                }
            }
            if runtime == ".codex" {
                match text.parse::<toml::Table>() {
                    Ok(table) => {
                        for key in [
                            "name",
                            "description",
                            "model",
                            "model_reasoning_effort",
                            "developer_instructions",
                        ] {
                            if table
                                .get(key)
                                .and_then(toml::Value::as_str)
                                .is_none_or(str::is_empty)
                            {
                                problems.push(format!("{relative}: missing {key}"));
                            }
                        }
                        if table.get("name").and_then(toml::Value::as_str) != Some(role) {
                            problems.push(format!("{relative}: name must be {role}"));
                        }
                    }
                    Err(error) => problems.push(format!("{relative}: {error}")),
                }
                if text.contains("Generated by scripts/agent-config.py") {
                    problems.push(format!("{relative}: retire generated adapter ownership"));
                }
            }
        }
    }
    if root.join(".codex/agents/implementer.toml").exists() {
        problems.push(".codex/agents/implementer.toml is obsolete; Codex uses executor".into());
    }
    Ok(())
}
fn check_hooks(path: &Path, problems: &mut Vec<String>) -> Result<()> {
    let config: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let hooks = config
        .get("hooks")
        .and_then(serde_json::Value::as_object)
        .context("hooks must be an object")?;
    if hooks
        .get("PreToolUse")
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        problems.push(format!("{}: missing PreToolUse hook", path.display()));
    }
    for (event, groups) in hooks {
        for group in groups.as_array().context("hook groups must be arrays")? {
            for hook in group
                .get("hooks")
                .and_then(serde_json::Value::as_array)
                .context("hook group must contain hooks")?
            {
                let command = hook
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .context("hook command must be a string")?;
                if !(command.contains("/scripts/pse-host") && command.contains("agent-hook")) {
                    problems.push(format!(
                        "{}: {event} hook does not use shared native policy",
                        path.display()
                    ));
                }
            }
        }
    }
    Ok(())
}
fn check_deny_rules(path: &Path, problems: &mut Vec<String>) -> Result<usize> {
    let config: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let mut count = 0;
    let Some(rules) = config
        .pointer("/permissions/deny")
        .and_then(serde_json::Value::as_array)
    else {
        return Ok(0);
    };
    for rule in rules {
        let rule = rule.as_str().context("deny rules must be strings")?;
        let Some((tool, target)) = rule.strip_suffix(')').and_then(|body| body.split_once('('))
        else {
            continue;
        };
        if !["Edit", "Read", "Write", "NotebookEdit", "MultiEdit", "Glob"].contains(&tool) {
            continue;
        }
        count += 1;
        if !["Edit", "Read"].contains(&tool) {
            problems.push(format!(
                "{}: deny rule {rule:?} must use Edit or Read",
                path.display()
            ));
        }
        if !(target.starts_with('/') || target.starts_with("~/")) {
            problems.push(format!(
                "{}: deny rule {rule:?} must be anchored at a fixed root",
                path.display()
            ));
        }
    }
    Ok(count)
}

/// Validate the current native agent wiring and instruction references without mutation.
pub fn check(root: &Path) -> Result<Check> {
    let mut report = Check::default();
    match synchronize(root, true) {
        Ok(problems) => report.problems.extend(problems),
        Err(error) => report.problems.push(error.to_string()),
    }
    check_adapters(root, &mut report.problems)?;
    for path in [".codex/hooks.json", ".claude/settings.json"] {
        if let Err(error) = check_hooks(&root.join(path), &mut report.problems) {
            report.problems.push(format!("{path}: {error}"));
        }
    }
    match check_deny_rules(&root.join(".claude/settings.json"), &mut report.problems) {
        Ok(count) => report.deny_rules = count,
        Err(error) => report.problems.push(error.to_string()),
    }
    let claude = root.join("CLAUDE.md");
    if metadata(&claude)?.is_some_and(|meta| meta.is_file() && !meta.is_symlink()) {
        if fs::read_to_string(&claude)?.lines().next() != Some("@AGENTS.md") {
            report
                .problems
                .push("CLAUDE.md must start with @AGENTS.md".into());
        }
    } else {
        report
            .problems
            .push("CLAUDE.md must be a real file importing @AGENTS.md".into());
    }
    let allowlist = root.join(".claude/path-allowlist.txt");
    let allowlist = if allowlist.is_file() {
        fs::read_to_string(allowlist)?
    } else {
        String::new()
    };
    let allowlist = allowlist
        .lines()
        .map(|line| {
            line.split('#')
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches('/')
        })
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut allowed = globset::GlobSetBuilder::new();
    for pattern in &allowlist {
        for pattern in [pattern.clone(), format!("{pattern}/*")] {
            allowed.add(
                globset::GlobBuilder::new(&pattern)
                    .literal_separator(false)
                    .backslash_escape(false)
                    .build()?,
            );
        }
    }
    let allowed = allowed.build()?;
    let recipes = Command::new("just")
        .arg("--summary")
        .current_dir(root)
        .output()
        .context("just --summary is required for agent reference validation")?;
    ensure!(
        recipes.status.success(),
        "just --summary failed while checking agent recipe references"
    );
    let recipes = String::from_utf8(recipes.stdout)?
        .split_whitespace()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for path in scanned_files(root)? {
        report.files += 1;
        let text = fs::read_to_string(&path)?;
        for reference in path_references(body(&text))? {
            report.path_references += 1;
            if !root.join(&reference).exists() && !allowed.is_match(&reference) {
                report.problems.push(format!(
                    "{}: path does not exist: {reference}",
                    path.strip_prefix(root)?.display()
                ));
            }
        }
        for name in recipe_references(body(&text))? {
            report.recipe_references += 1;
            if !recipes.contains(&name) {
                report.problems.push(format!(
                    "{}: no such just recipe: just {name}",
                    path.strip_prefix(root)?.display()
                ));
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
#[path = "configuration_tests.rs"]
mod tests;
