// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared native edit protection and nonblocking session environment activation.
use crate::HostError;
use serde_json::Value;
use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};

/// A visible checkout-local authorization; committed markers never grant authority.
pub const DESIGN_EDIT_MARKER: &str = ".design-edit";
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}

/// Read every edited path, including both original and destination rename paths.
/// Unknown or malformed payloads refuse rather than permitting an unchecked edit.
pub fn edit_paths(payload: &Value) -> Result<Vec<String>, HostError> {
    let payload = payload
        .as_object()
        .ok_or_else(|| invalid("edit payload must be an object"))?;
    let inputs = payload
        .get("tool_input")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("tool_input must be an object"))?;
    for key in ["file_path", "notebook_path"] {
        if let Some(value) = inputs.get(key) {
            return value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(|value| vec![value.into()])
                .ok_or_else(|| invalid(format!("{key} must be a nonempty string")));
        }
    }
    if let Some(command) = inputs.get("command").and_then(Value::as_str)
        && command.starts_with("*** Begin Patch\n")
        && command.contains("*** End Patch")
    {
        let paths = command
            .lines()
            .filter_map(|line| {
                [
                    "*** Add File: ",
                    "*** Update File: ",
                    "*** Delete File: ",
                    "*** Move to: ",
                ]
                .into_iter()
                .find_map(|prefix| line.strip_prefix(prefix))
            })
            .filter(|path| !path.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if !paths.is_empty() {
            return Ok(paths);
        }
    }
    Err(invalid(
        "unrecognized edit payload; refusing an unchecked file edit",
    ))
}

// Resolve existing links even when their final target has not been created yet.
// Walking path components never traverses directory contents, and a link cycle refuses.
fn resolve(path: &Path) -> Result<PathBuf, HostError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut pending = absolute
        .components()
        .map(|component| component.as_os_str().to_owned())
        .collect::<VecDeque<_>>();
    let mut resolved = PathBuf::new();
    let mut links = 0;
    while let Some(part) = pending.pop_front() {
        match Path::new(&part).components().next() {
            Some(Component::CurDir) => {}
            Some(Component::ParentDir) => {
                resolved.pop();
            }
            Some(Component::RootDir | Component::Prefix(_)) => resolved.push(part),
            Some(Component::Normal(_)) => {
                let next = resolved.join(&part);
                match fs::symlink_metadata(&next) {
                    Ok(meta) if meta.is_symlink() => {
                        links += 1;
                        if links > 40 {
                            return Err(invalid(
                                "edit path has a symlink cycle or excessive indirection",
                            ));
                        }
                        let target = fs::read_link(next)?;
                        if target.is_absolute() {
                            resolved.clear();
                        }
                        for component in target.components().rev() {
                            pending.push_front(component.as_os_str().to_owned());
                        }
                    }
                    Ok(_) => resolved.push(part),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        resolved.push(part)
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            None => {}
        }
    }
    Ok(resolved)
}

/// Return the original protection reason, after resolving relative paths and symlinks.
/// The design exception never authorizes VCS, generated or foreign build output edits.
pub fn protected(root: &Path, path: &Path, design_edit: bool) -> Result<Option<String>, HostError> {
    let root = resolve(root)?;
    let target = resolve(&root.join(path))?;
    let Ok(relative) = target.strip_prefix(&root) else {
        return Ok(None);
    };
    let parts = relative
        .components()
        .map(|part| part.as_os_str())
        .collect::<Vec<_>>();
    if parts.is_empty()
        || parts.contains(&OsStr::new(".git"))
        || ["target", "build", "external"]
            .iter()
            .any(|name| parts[0] == OsStr::new(name))
    {
        return Ok(Some(
            "VCS state, build output and reading copies are protected".into(),
        ));
    }
    let name = relative.to_string_lossy().replace('\\', "/");
    if [
        "docs/generated/",
        "scripts/generated/",
        "xtask/src/generated/",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
        || parts[0] == OsStr::new("crates") && parts.contains(&OsStr::new("generated"))
        || name == "crates/pse-ipopt-sys/src/bindings.rs"
    {
        return Ok(Some(
            "generator output is protected; fix the generator".into(),
        ));
    }
    if !design_edit {
        if name.starts_with("docs/authoritative_design/") {
            return Ok(Some(format!(
                "blueprint edits require the authorized design workflow (PSE_DESIGN_EDIT=1 or an untracked {DESIGN_EDIT_MARKER} file)"
            )));
        }
        if let Some(filename) = name.strip_prefix("docs/adr/")
            && filename
                .as_bytes()
                .get(..4)
                .is_some_and(|prefix| prefix.iter().all(u8::is_ascii_digit))
            && filename.as_bytes().get(4) == Some(&b'-')
            && filename.ends_with(".md")
            && !filename.contains('/')
            && target.is_file()
        {
            let text = fs::read_to_string(target)?;
            let front = text
                .strip_prefix("---\n")
                .and_then(|text| text.split_once("---"))
                .map_or("", |(front, _)| front);
            let status = front
                .lines()
                .find_map(|line| line.strip_prefix("status:"))
                .and_then(|value| value.split_whitespace().next());
            if status != Some("proposed") {
                return Ok(Some(
                    "accepted ADRs are immutable; use the ADR supersession workflow".into(),
                ));
            }
        }
    }
    Ok(None)
}

/// Accept the explicit launch-time selection or a currently untracked visible marker.
pub fn design_edit_authorized(
    root: &Path,
    launch_value: Option<&OsStr>,
) -> Result<bool, HostError> {
    if launch_value == Some(OsStr::new("1")) {
        return Ok(true);
    }
    if !root.join(DESIGN_EDIT_MARKER).is_file() {
        return Ok(false);
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--error-unmatch", DESIGN_EDIT_MARKER])
        .output()?;
    Ok(!tracked.status.success())
}

/// Prefer the payload's actual Git checkout for SessionStart, including worktrees.
pub fn session_root(default: &Path, payload: &Value) -> Result<PathBuf, HostError> {
    let Some(cwd) = payload
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    else {
        return Ok(default.to_owned());
    };
    let output = Command::new("git")
        .args(["-C", cwd, "rev-parse", "--show-toplevel"])
        .output()?;
    if output.status.success() {
        let root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        if root.join("scripts/pse-env").is_file() {
            return Ok(root);
        }
    }
    Ok(default.to_owned())
}

/// Append only the existing native boundary's activation program, preserving its
/// private value transfer. The CLI reports any failure without blocking the session.
pub fn session_env(root: &Path, destination: Option<&Path>) -> Result<(), HostError> {
    let Some(destination) = destination else {
        return Ok(());
    };
    let result = Command::new(root.join("scripts/pse-env"))
        .arg("--print")
        .current_dir(root)
        .output()?;
    if !result.status.success() {
        return Err(invalid(format!(
            "session environment activation exited {}",
            result.status
        )));
    }
    let mut output = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(destination)?;
    output.write_all(&result.stdout)?;
    Ok(())
}

fn default_root() -> Result<PathBuf, HostError> {
    if let Some(root) = std::env::var_os("CLAUDE_PROJECT_DIR").map(PathBuf::from)
        && root.join("scripts/pse-env").is_file()
    {
        return Ok(root);
    }
    let mut root = std::env::current_dir()?;
    while !root.join(".config/diagnostic-tools.toml").is_file() {
        if !root.pop() {
            return Err(invalid("run agent hooks inside a pse-arrow checkout"));
        }
    }
    Ok(root)
}

/// The lean host hook endpoint. Help remains offline and never reads standard input.
pub fn run_cli(arguments: &[OsString]) -> Result<i32, HostError> {
    if arguments.is_empty()
        || arguments == [OsString::from("--help")]
        || arguments == [OsString::from("-h")]
    {
        writeln!(std::io::stdout(), "agent-hook guard|session-env")?;
        return Ok(0);
    }
    let action = arguments.first().and_then(|value| value.to_str());
    if arguments.len() != 1 || !matches!(action, Some("guard" | "session-env")) {
        writeln!(
            std::io::stderr(),
            "BLOCKED: expected agent-hook guard|session-env"
        )?;
        return Ok(2);
    }
    let mut input = String::new();
    let read = std::io::stdin().lock().read_to_string(&mut input);
    if action == Some("session-env") {
        let result: Result<(), HostError> = (|| {
            let default = default_root()?;
            let payload = serde_json::from_str(&input).unwrap_or(Value::Null);
            let root = session_root(&default, &payload)?;
            session_env(
                &root,
                std::env::var_os("CLAUDE_ENV_FILE")
                    .as_deref()
                    .map(Path::new),
            )
        })();
        if let Err(error) = result {
            writeln!(
                std::io::stderr(),
                "pse-env: session environment unavailable: {error}"
            )?;
        }
        return Ok(0);
    }
    let result: Result<(), HostError> = (|| {
        read?;
        let payload =
            serde_json::from_str::<Value>(&input).map_err(|error| invalid(error.to_string()))?;
        let paths = edit_paths(&payload)?;
        let root = default_root()?;
        let design_edit =
            design_edit_authorized(&root, std::env::var_os("PSE_DESIGN_EDIT").as_deref())?;
        for path in paths {
            if let Some(reason) = protected(&root, Path::new(&path), design_edit)? {
                return Err(invalid(format!("{path}: {reason}")));
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(0),
        Err(error) => {
            writeln!(std::io::stderr(), "BLOCKED: {error}")?;
            Ok(2)
        }
    }
}

#[cfg(test)]
#[path = "agent_hooks_tests.rs"]
mod tests;
