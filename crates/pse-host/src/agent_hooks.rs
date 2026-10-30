// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared native edit protection and nonblocking session environment activation.
use crate::{
    HostError,
    ownership::{self, OwnershipMap},
};
use serde_json::Value;
use std::{
    ffi::OsStr,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

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

/// An enforced output boundary or advisory source workflow, never edit authorization.
#[derive(Debug, PartialEq, Eq)]
pub enum EditRoute {
    /// Generated/VCS/build output cannot be edited directly.
    Blocked {
        /// Actual generator/physical-boundary reason and available next action.
        reason: String,
    },
    /// Ordinary authored edits retain their required design/decision workflow context.
    Allowed {
        /// Route context for the original operator, not permission to bypass the workflow.
        additional_context: Option<String>,
    },
}

/// Evaluate exact path ownership after resolving relative operands and symlinks.
/// Missing/invalid maps retain generated-path protection without blocking ordinary sources.
pub fn route(root: &Path, path: &Path) -> Result<EditRoute, HostError> {
    let root = ownership::resolve(root)?;
    let target = ownership::resolve(&root.join(path))?;
    let Ok(relative) = target.strip_prefix(&root) else {
        return Ok(EditRoute::Allowed {
            additional_context: None,
        });
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
        return Ok(EditRoute::Blocked {
            reason: "VCS state, build output and reading copies are protected".into(),
        });
    }
    let map_error = match OwnershipMap::read(&root) {
        Ok(map) => {
            if let Some((owner, _)) = map.owner(relative)? {
                return Ok(EditRoute::Blocked {
                    reason: format!(
                        "generated output belongs to {}; change its declared inputs and run {}",
                        owner.producer,
                        owner.regenerate.join(" ")
                    ),
                });
            }
            None
        }
        Err(error) => Some(error.to_string()),
    };
    if parts.contains(&OsStr::new("generated"))
        || relative == Path::new(ownership::protocol::IPOPT_BINDINGS)
    {
        return Ok(EditRoute::Blocked {
            reason: format!(
                "generated output owner is unknown{}; run just codegen to refresh the generated ownership map",
                map_error
                    .as_ref()
                    .map_or(String::new(), |error| format!(": {error}"))
            ),
        });
    }
    let mut context = Vec::new();
    if let Some(error) = map_error {
        context.push(format!("Generated ownership map unavailable: {error}. Ordinary authored edits remain available; run just codegen to publish routing."));
    }
    if relative.starts_with("docs/authoritative_design") {
        context.push("Architecture changes require the decision/design workflow and a corresponding blueprint revision row; use docs/README.md and the owning architecture section before editing.".into());
    }
    if let Some(name) = relative.to_str() {
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
                context.push("Accepted ADRs remain immutable. Use the ADR supersession workflow and amend the owning architecture with its revision row; this advisory does not authorize changing the accepted record.".into());
            } else {
                context.push("Proposed ADR edits retain the ADR authoring/review workflow; acceptance remains a separate decision.".into());
            }
        }
    }
    Ok(EditRoute::Allowed {
        additional_context: (!context.is_empty()).then(|| context.join("\n")),
    })
}

/// Enforced protection reason, without conflating advisory design routes with a block.
pub fn protected(root: &Path, path: &Path) -> Result<Option<String>, HostError> {
    Ok(match route(root, path)? {
        EditRoute::Blocked { reason } => Some(reason),
        EditRoute::Allowed { .. } => None,
    })
}

/// Prefer the payload's actual Git checkout for native hooks, including worktrees.
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
    crate::cli::checkout_root(None)
}

/// The lean host hook endpoint. Help remains offline and never reads standard input.
pub(crate) fn execute(action: crate::cli::HookAction) -> Result<i32, HostError> {
    let mut input = String::new();
    let read = std::io::stdin().lock().read_to_string(&mut input);
    if matches!(action, crate::cli::HookAction::SessionEnv) {
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
    let result: Result<Vec<String>, HostError> = (|| {
        read?;
        let payload =
            serde_json::from_str::<Value>(&input).map_err(|error| invalid(error.to_string()))?;
        let paths = edit_paths(&payload)?;
        let root = session_root(&default_root()?, &payload)?;
        let mut context = Vec::new();
        for path in paths {
            match route(&root, Path::new(&path))? {
                EditRoute::Blocked { reason } => return Err(invalid(format!("{path}: {reason}"))),
                EditRoute::Allowed {
                    additional_context: Some(detail),
                } => context.push(detail),
                EditRoute::Allowed {
                    additional_context: None,
                } => {}
            }
        }
        context.sort();
        context.dedup();
        Ok(context)
    })();
    match result {
        Ok(context) => {
            if !context.is_empty() {
                serde_json::to_writer(std::io::stdout().lock(), &serde_json::json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":context.join("\n")}}))
                    .map_err(|error| invalid(error.to_string()))?;
                writeln!(std::io::stdout())?;
            }
            Ok(0)
        }
        Err(error) => {
            writeln!(std::io::stderr(), "BLOCKED: {error}")?;
            Ok(2)
        }
    }
}

#[cfg(test)]
#[path = "agent_hooks_tests.rs"]
mod tests;
