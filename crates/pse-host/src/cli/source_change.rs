// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Receive the exact source generator preview and original publication state.
use super::{GenerationIntent, checkout_root, private_state};
use crate::{
    HostError,
    envelope::{
        CommandOutcome, Envelope, ErrorRecord, NextAction, OutputOptions, RedactionPolicy,
        Settlement, SubjectState,
    },
    environment::Environment,
    paths::CheckoutState,
};
use clap::{Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest as _, Sha256};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

const STATUS_BYTES: usize = 4 * 1024 * 1024;
const RECORD_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Generate consequences in isolation from authored checkout inputs.
    #[command(add(crate::catalog::description::CommandEffect::Prepare))]
    Preview {
        /// Supported model change shape.
        #[arg(long, value_enum)]
        case: Case,
        /// Additional authored checkout input.
        #[arg(long)]
        input: Vec<PathBuf>,
        /// Compatibility intent for the original preview.
        #[arg(long)]
        compatibility: Option<String>,
    },
    /// Read a retained preview or publication without starting Cargo.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Status {
        #[arg(value_parser = parse_id)]
        id: String,
        /// Include the bounded diagnostic tail of the recorded failed stage.
        #[arg(long)]
        diagnostic: bool,
    },
    /// Publish the generated files for the exact preview.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Apply {
        #[arg(value_parser = parse_id)]
        id: String,
    },
    /// Reconcile an original interrupted preview or publication.
    #[command(add(crate::catalog::description::CommandEffect::Recover))]
    Recover {
        #[arg(value_parser = parse_id)]
        id: String,
    },
    /// Release an identified preview whose publication is settled.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Discard {
        #[arg(value_parser = parse_id)]
        id: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Case {
    OptionalProjectDescription,
    RecordedResultFieldRename,
}

impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Status { .. } => GenerationIntent::Passive,
            Self::Recover { .. } | Self::Discard { .. } => GenerationIntent::ExistingControl,
            Self::Preview { .. } | Self::Apply { .. } => GenerationIntent::NewWork,
        }
    }
}

fn parse_id(value: &str) -> Result<String, &'static str> {
    if value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(value.into())
    } else {
        Err("expected the original 32-character lowercase preview identity")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum State {
    Generating,
    GenerationFailed,
    Ready,
    Applying,
    Applied,
    Discarded,
}

#[derive(Deserialize)]
struct OwnerFailure {
    code: String,
    stage: String,
    original_log_sha256: Option<String>,
    recovery: Option<Vec<String>>,
}
#[derive(Deserialize)]
struct Status {
    version: u32,
    preview_id: String,
    state: State,
    failure: Option<OwnerFailure>,
}

#[derive(Serialize, JsonSchema)]
pub(super) struct View {
    preview_id: String,
    state: State,
    receipt: serde_json::Value,
    diagnostic: Option<String>,
}

fn passive_status(root: &Path, state: &Path, id: &str) -> Result<Box<RawValue>, HostError> {
    #[derive(Deserialize)]
    struct Record {
        checkout: PathBuf,
        status: Box<RawValue>,
    }
    private_state::directory(state)?;
    let directory = state.join(id);
    private_state::directory(&directory)?;
    let bytes = private_state::read(&directory.join("record.json"), RECORD_BYTES)?;
    let record: Record = serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
    if record.checkout != root {
        return Err(HostError::Ownership(
            "source-change receipt belongs to a different checkout".into(),
        ));
    }
    Ok(record.status)
}

fn decode(raw: &RawValue, expected: Option<&str>) -> Result<Status, HostError> {
    if raw.get().len() > STATUS_BYTES {
        return Err(HostError::Ownership(
            "source-change receipt exceeds its extent".into(),
        ));
    }
    let status: Status = serde_json::from_str(raw.get()).map_err(std::io::Error::other)?;
    if status.version != 1
        || parse_id(&status.preview_id).is_err()
        || expected.is_some_and(|id| id != status.preview_id)
    {
        return Err(HostError::Ownership(
            "source-change receipt identity mismatch".into(),
        ));
    }
    Ok(status)
}

pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?.canonicalize()?;
    let environment: Environment = std::env::vars_os().collect();
    let state = CheckoutState::resolve(&root, &environment)?
        .root
        .join("source-change");
    let policy = RedactionPolicy::from_inputs(&environment);
    let (verb, id, extra, diagnostic) = match action {
        Action::Preview {
            case,
            input,
            compatibility,
        } => {
            let case = case.to_possible_value().ok_or_else(|| {
                HostError::Ownership("source-change case has no command representation".into())
            })?;
            let mut args = vec![OsString::from("--case"), case.get_name().into()];
            for path in input {
                args.extend([
                    OsString::from("--input"),
                    super::source_path(selected, path)?.into_os_string(),
                ]);
            }
            if let Some(note) = compatibility {
                args.push(format!("--compatibility={note}").into());
            }
            ("preview", None, args, false)
        }
        Action::Status { id, diagnostic } => ("status", Some(id), Vec::new(), diagnostic),
        Action::Apply { id } => ("apply", Some(id), Vec::new(), false),
        Action::Recover { id } => ("recover", Some(id), Vec::new(), false),
        Action::Discard { id } => ("discard", Some(id), Vec::new(), false),
    };
    let command_name = format!("source-change {verb}");
    output.validate_for::<View>(&command_name)?;
    let (raw, exit) = if verb == "status" {
        let id = id
            .as_deref()
            .ok_or_else(|| HostError::Ownership("source-change status requires an ID".into()))?;
        (passive_status(&root, &state, id)?, None)
    } else {
        let mut command = Command::new(root.join("scripts/pse-env"));
        command
            .current_dir(&root)
            .env_clear()
            .envs(&environment)
            .args([
                "--",
                "scripts/cargo-bin/cargo-pse-correctness",
                "run",
                "--quiet",
                "--bin",
                "xtask",
                "-p",
                "xtask",
                "--no-default-features",
                "--locked",
                "--",
                "source-change",
                "--state-root",
            ])
            .arg(&state)
            .arg(verb)
            .args(id.iter())
            .args(extra);
        let child = crate::product_reader::execute(command, Vec::new(), STATUS_BYTES, None)
            .map_err(super::product::product_error)?;
        if child.stdout_bytes > STATUS_BYTES as u64 {
            return Err(HostError::Ownership(
                "source-change child response exceeds its complete extent".into(),
            ));
        }
        let raw = serde_json::from_slice(&child.stdout).map_err(|_| {
            HostError::Ownership(format!(
                "source-change owner returned exit {} without a receipt: {}",
                child.status,
                policy.redact(&String::from_utf8_lossy(&child.stderr))
            ))
        })?;
        (raw, Some(child.status))
    };
    let status = decode(&raw, id.as_deref())?;
    if exit.is_some_and(|exit| exit != if status.failure.is_some() { 1 } else { 0 }) {
        return Err(HostError::Ownership(
            "source-change child exit differs from its original receipt".into(),
        ));
    }
    // Project string values after JSON decoding to preserve redaction across JSON escapes.
    let receipt =
        serde_json::from_str::<serde_json::Value>(raw.get()).map_err(std::io::Error::other)?;
    let mut response = Envelope::response(
        &command_name,
        format!("source-change {verb} receipt"),
        View {
            preview_id: status.preview_id,
            state: status.state,
            receipt: redact(receipt, &policy),
            diagnostic: None,
        },
    );
    response.child_exit = exit;
    response.observation.scope = "source_change".into();
    response.subject_state = match status.state {
        State::Ready | State::Applied | State::Discarded => SubjectState::Succeeded,
        State::GenerationFailed => SubjectState::Failed,
        State::Generating | State::Applying => SubjectState::Unknown,
    };
    response.settlement = match status.state {
        State::Generating | State::Applying => Settlement::Incomplete,
        _ => Settlement::Complete,
    };
    if let Some(failure) = status.failure {
        if matches!(
            failure.stage.as_str(),
            "release-snapshot" | "release-postimages"
        ) {
            response.settlement = Settlement::Incomplete;
        }
        let error = HostError::Ownership(format!(
            "source-change {} failed at {}",
            failure.code, failure.stage
        ));
        response
            .failures
            .push(ErrorRecord::from_host(&error, &policy));
        if exit.is_some() {
            response.outcome = CommandOutcome::Failure;
        }
        let id = &response.payload.preview_id;
        if diagnostic && let Some(expected) = &failure.original_log_sha256 {
            let detail = private_state::read(&state.join(id).join("failed-stage.log"), 68 * 1024)
                .and_then(|bytes| {
                    let observed: String = Sha256::digest(&bytes)
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect();
                    if *expected != observed {
                        return Err(HostError::Ownership(
                            "source-change diagnostic no longer matches its receipt".into(),
                        ));
                    }
                    Ok(policy.redact(&String::from_utf8_lossy(&bytes)))
                });
            match detail {
                Ok(detail) => response.payload.diagnostic = Some(detail),
                Err(error) => {
                    response.observation.completeness = crate::envelope::Completeness::Partial;
                    response
                        .failures
                        .push(ErrorRecord::from_host(&error, &policy));
                }
            }
        }
        if failure
            .recovery
            .as_deref()
            .is_some_and(|argv| argv == ["pse", "source-change", "recover", id])
            && let (Some(program), Some(checkout)) =
                (root.join("scripts/pse").to_str(), root.to_str())
            && [program, checkout]
                .into_iter()
                .all(|word| policy.redact(word) == word)
        {
            let argv = [program, "-C", checkout, "source-change", "recover", id]
                .map(ToOwned::to_owned)
                .to_vec();
            response.next.push(NextAction::Command {
                argv,
                reason: "reconcile the original preview or publication".into(),
            });
        }
    }
    Ok(output.emit(response)?)
}

fn redact(value: serde_json::Value, policy: &RedactionPolicy) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::String(value) => Value::String(policy.redact(&value)),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|value| redact(value, policy))
                .collect(),
        ),
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (policy.redact(&key), redact(value, policy)))
                .collect(),
        ),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _},
    };

    #[test]
    fn passive_source_status_requires_the_original_private_checkout_receipt() {
        let directory = tempfile::tempdir().unwrap();
        let state = directory.path().join("state");
        let id = "1234567890abcdef1234567890abcdef";
        assert!(passive_status(directory.path(), &state, id).is_err());
        assert!(!state.exists());
        let operation = state.join(id);
        fs::DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&operation)
            .unwrap();
        let file = operation.join("record.json");
        let record = serde_json::json!({
            "checkout": directory.path(),
            "status": { "version": 1, "preview_id": id, "state": "applying", "failure": null },
        });
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(file)
            .unwrap()
            .write_all(&serde_json::to_vec(&record).unwrap())
            .unwrap();
        assert!(passive_status(&state, &state, id).is_err());
        let receipt = passive_status(directory.path(), &state, id).unwrap();
        assert!(matches!(
            decode(&receipt, Some(id)).unwrap().state,
            State::Applying
        ));
        assert!(decode(&receipt, Some("another")).is_err());
    }
}
