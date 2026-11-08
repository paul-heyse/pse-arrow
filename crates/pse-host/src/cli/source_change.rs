// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Receive the exact source generator preview and original publication state.
use super::{GenerationIntent, checkout_root, private_state};
use crate::{
    HostError,
    envelope::{
        CommandOutcome, Envelope, ErrorRecord, LabelRecord, NextAction, ObservationValue,
        OutputOptions, RedactionPolicy, Settlement, SubjectState,
    },
    environment::Environment,
    paths::CheckoutState,
    source_proposal::handoff::{self, ProposalHandoff},
};
use clap::{Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest as _, Sha256};
use std::{
    ffi::OsString,
    io::Read as _,
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
        /// Verify a reviewed semantic proposal's applied file digests from PATH or stdin (-).
        #[arg(long, value_name = "PATH")]
        proposal: Option<PathBuf>,
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
    #[serde(default)]
    completed_stages: Vec<String>,
    #[serde(default)]
    surrealql_validation: Vec<StagedValidation>,
}

#[derive(Deserialize)]
struct StagedValidation {
    input: String,
    sha256: Option<String>,
    error: Option<ValidationError>,
}

#[derive(Deserialize)]
struct ValidationError {
    kind: String,
    message: String,
    #[serde(default)]
    diagnostics: Vec<ParseDiagnostic>,
    #[serde(default)]
    omitted_diagnostics: usize,
}

#[derive(Deserialize)]
struct ParseDiagnostic {
    kind: String,
    message: String,
    primary: LabelRecord,
    secondary: Option<LabelRecord>,
}

impl StagedValidation {
    fn finding(self, policy: &RedactionPolicy) -> Option<ErrorRecord> {
        let error = self.error?;
        let source = policy.redact(&self.input);
        let mut record = ErrorRecord::from_diagnostic(
            &miette::MietteDiagnostic::new(format!("{}: {}", self.input, error.message)),
            None,
            policy,
        );
        record
            .parser_context
            .insert("kind".into(), policy.redact(&error.kind));
        record
            .facts
            .observations
            .insert("path".into(), ObservationValue::Text(source.clone()));
        if let Some(digest) = self.sha256 {
            record.facts.observations.insert(
                "sha256".into(),
                ObservationValue::Text(policy.redact(&digest)),
            );
        }
        if error.omitted_diagnostics > 0 {
            record.parser_context.insert(
                "omitted_diagnostics".into(),
                error.omitted_diagnostics.to_string(),
            );
        }
        record.children = error
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                let mut record = ErrorRecord::from_diagnostic(
                    &miette::MietteDiagnostic::new(diagnostic.message),
                    None,
                    policy,
                );
                record
                    .parser_context
                    .insert("kind".into(), policy.redact(&diagnostic.kind));
                record.labels = std::iter::once(diagnostic.primary)
                    .chain(diagnostic.secondary)
                    .map(|mut label| {
                        label.source = Some(source.clone());
                        label.label = label.label.map(|text| policy.redact(&text));
                        label.unavailable = label.unavailable.map(|text| policy.redact(&text));
                        label
                    })
                    .collect();
                record
            })
            .collect();
        Some(record)
    }
}

#[derive(Serialize, JsonSchema)]
pub(super) struct View {
    preview_id: String,
    state: State,
    /// Unfinished stage recorded by the original owner. This does not establish liveness.
    #[serde(skip_serializing_if = "Option::is_none")]
    recorded_stage: Option<String>,
    /// Last stage whose completion was recorded by the original owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    last_completed_stage: Option<String>,
    receipt: serde_json::Value,
    diagnostic: Option<String>,
}

fn passive_status(
    root: &Path,
    state: &Path,
    id: &str,
) -> Result<(Box<RawValue>, Option<String>), HostError> {
    #[derive(Deserialize)]
    struct Record {
        checkout: PathBuf,
        status: Box<RawValue>,
        inflight_stage: Option<String>,
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
    Ok((record.status, record.inflight_stage))
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

fn proposal_input(mut input: impl std::io::Read, checkout_id: &str) -> Result<Vec<u8>, HostError> {
    #[derive(Deserialize)]
    struct ProposalPayload {
        proposal: ProposalResult,
    }
    #[derive(Deserialize)]
    struct ProposalResult {
        handoff: ProposalHandoff,
    }
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input {
        Projection(ProposalHandoff),
        Envelope {
            record: String,
            command: String,
            outcome: String,
            payload: ProposalPayload,
        },
    }
    let mut bytes = Vec::new();
    input
        .by_ref()
        .take(STATUS_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > STATUS_BYTES {
        return Err(HostError::Ownership(
            "source-change proposal input exceeds its byte limit".into(),
        ));
    }
    let decoded: Input = serde_json::from_slice(&bytes).map_err(|_| {
        HostError::Ownership(
            "expected a reviewed source-change handoff or a semantic proposal containing one"
                .into(),
        )
    })?;
    let projection = match decoded {
        Input::Projection(projection) => projection,
        Input::Envelope {
            record,
            command,
            outcome,
            payload,
        } if record == "response" && command == "propose entity" && outcome == "success" => {
            payload.proposal.handoff
        }
        Input::Envelope { .. } => {
            return Err(HostError::Ownership(
                "source-change proposal envelope is not a successful semantic edit".into(),
            ));
        }
    };
    projection
        .validate()
        .map_err(|message| HostError::Ownership(message.into()))?;
    if projection.checkout_id != checkout_id {
        return Err(HostError::Ownership(
            "source-change proposal belongs to a different checkout".into(),
        ));
    }
    let projected = serde_json::to_vec(&projection).map_err(std::io::Error::other)?;
    if projected.len() > handoff::MAX_BYTES {
        return Err(HostError::Ownership(
            "source-change proposal handoff exceeds its byte limit".into(),
        ));
    }
    Ok(projected)
}

fn read_proposal(
    path: PathBuf,
    selected: Option<&Path>,
    checkout_id: &str,
) -> Result<Vec<u8>, HostError> {
    if path == Path::new("-") {
        return proposal_input(std::io::stdin().lock(), checkout_id);
    }
    use std::os::unix::fs::OpenOptionsExt as _;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32)
        .open(super::source_path(selected, path)?)?;
    if !file.metadata()?.is_file() {
        return Err(HostError::Ownership(
            "source-change proposal PATH must be a regular file".into(),
        ));
    }
    proposal_input(file, checkout_id)
}

pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?.canonicalize()?;
    let environment: Environment = std::env::vars_os().collect();
    let checkout = CheckoutState::resolve(&root, &environment)?;
    let state = checkout.root.join("source-change");
    let policy = RedactionPolicy::from_inputs(&environment);
    let mut proposal_bytes = Vec::new();
    let (verb, id, extra, diagnostic) = match action {
        Action::Preview {
            case,
            input,
            compatibility,
            proposal,
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
            if let Some(path) = proposal {
                proposal_bytes = read_proposal(path, selected, checkout.checkout_id.as_str())?;
                args.push("--proposal-stdin".into());
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
    let (raw, exit, inflight_stage) = if verb == "status" {
        let id = id
            .as_deref()
            .ok_or_else(|| HostError::Ownership("source-change status requires an ID".into()))?;
        let (raw, inflight_stage) = passive_status(&root, &state, id)?;
        (raw, None, inflight_stage)
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
                "--features",
                "surrealql-tools",
                "--locked",
                "--",
                "source-change",
                "--state-root",
            ])
            .arg(&state)
            .arg(verb)
            .args(id.iter())
            .args(extra);
        let child = crate::product_reader::execute(command, proposal_bytes, STATUS_BYTES, None)
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
        (raw, Some(child.status), None)
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
    let recorded_stage = inflight_stage
        .filter(|_| matches!(status.state, State::Generating | State::Applying))
        .map(|stage| policy.redact(&stage));
    let last_completed_stage = (verb == "status")
        .then_some(status.completed_stages.last())
        .flatten()
        .map(|stage| policy.redact(stage));
    let mut response = Envelope::response(
        &command_name,
        format!("source-change {verb} receipt"),
        View {
            preview_id: status.preview_id,
            state: status.state,
            recorded_stage,
            last_completed_stage,
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
        let mut error = ErrorRecord::from_host(&error, &policy);
        error.code = Some(policy.redact(&failure.code));
        if failure.stage == "surrealql-validation" {
            error.related.extend(
                status
                    .surrealql_validation
                    .into_iter()
                    .filter_map(|file| file.finding(&policy)),
            );
        }
        response.failures.push(error);
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
    fn reviewed_proposal_input_projects_only_the_selected_checkout_digest_contract() {
        let id = "a".repeat(64);
        let projection = serde_json::json!({
            "version": 1, "checkout_id": id, "checkout_revision": "b".repeat(40),
            "files": [{"path": "crates/example/src/lib.rs", "before_sha256": "c".repeat(64), "after_sha256": "d".repeat(64)}],
        });
        let projection_bytes = serde_json::to_vec(&projection).unwrap();
        let projected = proposal_input(projection_bytes.as_slice(), &id).unwrap();
        let mut envelope = serde_json::json!({
            "record": "response", "command": "propose entity", "outcome": "success",
            "payload": {"proposal": {"handoff": projection, "candidates": ["private replacement body"]}},
        });
        let bytes = serde_json::to_vec(&envelope).unwrap();
        assert_eq!(proposal_input(bytes.as_slice(), &id).unwrap(), projected);
        assert!(proposal_input(bytes.as_slice(), &"e".repeat(64)).is_err());
        envelope["outcome"] = "failure".into();
        assert!(proposal_input(serde_json::to_vec(&envelope).unwrap().as_slice(), &id).is_err());
        let mut invalid = projection;
        invalid["files"][0]["path"] = "crates/../outside.rs".into();
        assert!(proposal_input(serde_json::to_vec(&invalid).unwrap().as_slice(), &id).is_err());
    }

    #[test]
    fn staged_parser_findings_keep_file_identity_labels_and_disclosure() {
        let labels = |primary| {
            serde_json::json!({
                "label": "secret label",
                "primary": primary,
                "byte_offset": if primary {27} else {20},
                "byte_length": 1,
                "line": 1,
                "column": if primary {14} else {7},
                "column_encoding": "utf8_bytes",
                "unavailable": null
            })
        };
        let digest = "a".repeat(64);
        let status = serde_json::json!({
            "version": 1, "preview_id": "a".repeat(32), "state": "generation_failed",
            "failure": {
                "code": "source_change.stage_failed", "stage": "surrealql-validation",
                "original_log_sha256": null, "recovery": null
            },
            "surrealql_validation": [{
                "input": "generated/secret.surql", "sha256": digest, "error": {
                    "kind": "syntax", "message": "secret message",
                    "diagnostics": [{
                        "kind": "message", "message": "secret delimiter",
                        "primary": labels(true), "secondary": labels(false)
                    }]
                }
            }, {
                "input": "generated/absent.surql", "sha256": null, "error": {
                    "kind": "input", "message": "No such file or directory"
                }
            }]
        });
        let status: Status = serde_json::from_value(status).unwrap();
        let policy = RedactionPolicy::new([("TOKEN".into(), "secret".into())]);
        let findings: Vec<_> = status
            .surrealql_validation
            .into_iter()
            .filter_map(|file| file.finding(&policy))
            .collect();
        let [syntax, input] = findings.as_slice() else {
            panic!("expected separate file findings")
        };
        assert_eq!(syntax.parser_context["kind"], "syntax");
        assert_eq!(input.parser_context["kind"], "input");
        assert!(input.children.is_empty());
        assert!(
            matches!(&syntax.facts.observations["sha256"], ObservationValue::Text(value) if value == &digest)
        );
        let [primary, secondary] = syntax.children[0].labels.as_slice() else {
            panic!("expected parser labels")
        };
        assert!(primary.primary);
        assert!(!secondary.primary);
        assert_eq!(
            (primary.byte_offset, primary.line, primary.column),
            (27, Some(1), Some(14))
        );
        assert_eq!(
            (secondary.byte_offset, secondary.line, secondary.column),
            (20, Some(1), Some(7))
        );
        assert_eq!(
            primary.source.as_deref(),
            Some("generated/<redacted:TOKEN>.surql")
        );
        assert_eq!(secondary.source, primary.source);
        assert!(!serde_json::to_string(&findings).unwrap().contains("secret"));
    }

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
        let (receipt, recorded_stage) = passive_status(directory.path(), &state, id).unwrap();
        assert!(recorded_stage.is_none());
        assert!(matches!(
            decode(&receipt, Some(id)).unwrap().state,
            State::Applying
        ));
        assert!(decode(&receipt, Some("another")).is_err());
    }
}
