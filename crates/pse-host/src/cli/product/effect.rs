// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit project effects and exact acknowledgement lookup through the performing owner.
use super::{ProjectSource, product_error, request_file};
use crate::{
    HostError,
    cli::GenerationIntent,
    envelope::{
        CommandOutcome, Completeness, Envelope, ErrorRecord, NextAction, OutputOptions,
        RedactionPolicy, Settlement, SubjectState,
    },
    environment::Environment,
    investigation::NativePath,
    product_reader::{
        self, ProductReaderSelection, ProjectEffectAcknowledgement, ProjectEffectOutcome,
        ProjectEffectTarget,
    },
};
use clap::{Args, Subcommand};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{
    io::{self, Read as _},
    num::NonZeroU64,
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub(in crate::cli) enum Action {
    /// Create a project under an explicit original operation identity.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Create {
        #[command(flatten)]
        target: Selection,
        /// Canonical project key.
        project: String,
        /// Human-readable project title.
        title: String,
    },
    /// Apply an authored navigation intent, read from stdin by default.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Navigate {
        #[command(flatten)]
        target: Selection,
        /// Authored JSON containing the original project descriptor and typed request.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Reconcile an original operation identity without submitting another effect.
    #[command(add(crate::catalog::description::CommandEffect::Recover))]
    Acknowledgement {
        #[command(flatten)]
        target: Selection,
    },
}
#[derive(Debug, Args)]
pub(in crate::cli) struct Selection {
    #[command(flatten)]
    source: ProjectSource,
    /// Expected issuing store reported by the original project read.
    #[arg(long)]
    issuing_store: String,
    /// Original operation identity, retained across acknowledgement queries.
    #[arg(long)]
    operation: String,
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Acknowledgement { .. } => GenerationIntent::Passive,
            _ => GenerationIntent::NewWork,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RequestAction {
    Create {
        project: String,
        title: String,
    },
    Navigate {
        project: Box<RawValue>,
        request: Box<RawValue>,
    },
    Acknowledgement,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NavigationIntent {
    project: Box<RawValue>,
    request: Box<RawValue>,
}
#[derive(Serialize)]
struct Request<'a> {
    protocol: &'static str,
    version: u32,
    operation: &'a str,
    target: Target<'a>,
    timeout_ms: Option<NonZeroU64>,
    action: RequestAction,
}
#[derive(Serialize)]
struct Target<'a> {
    state: NativePath,
    database: Option<&'a str>,
    issuing_store: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Disposition {
    Unprepared,
    Acknowledged,
    Absent,
    Refused,
    Unresolved,
}
#[derive(Serialize, JsonSchema)]
pub(in crate::cli) struct ProjectEffectView {
    reader: String,
    operation: String,
    target: Option<ProjectEffectTarget>,
    disposition: Disposition,
    acknowledgement: Option<ProjectEffectAcknowledgement>,
}

fn navigation_intent(path: Option<&Path>) -> Result<NavigationIntent, HostError> {
    let decode = |text: &str| {
        serde_json::from_str(text).map_err(|error| {
            HostError::Ownership(format!(
                "invalid project navigation intent ({:?}, line {}, column {})",
                error.classify(),
                error.line(),
                error.column()
            ))
        })
    };
    if let Some(path) = path {
        return decode(request_file(path)?.get());
    }
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(product_reader::PROJECT_PERFORM_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > product_reader::PROJECT_PERFORM_REQUEST_BYTES {
        return Err(HostError::Ownership(
            "project navigation intent exceeds its byte bound".into(),
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| HostError::Ownership("project navigation intent is not UTF-8 JSON".into()))?;
    decode(text)
}

fn recovery(
    root: &Path,
    state: &Path,
    target: &Selection,
    policy: &RedactionPolicy,
) -> Option<NextAction> {
    let mut argv = vec![
        root.join("scripts/pse").to_str()?.to_owned(),
        "-C".into(),
        root.to_str()?.into(),
        "project".into(),
        "acknowledgement".into(),
        format!("--reader={}", target.source.reader),
        format!("--state={}", state.to_str()?),
        format!("--issuing-store={}", target.issuing_store),
        format!("--operation={}", target.operation),
    ];
    if let Some(database) = &target.source.database {
        argv.push(format!("--database={database}"));
    }
    if let Some(timeout) = target.source.timeout_ms {
        argv.push(format!("--timeout-ms={timeout}"));
    }
    argv.iter()
        .all(|word| policy.redact(word) == *word)
        .then_some(NextAction::Command {
        argv,
        reason:
            "look up the exact original project acknowledgement before considering another effect"
                .into(),
    })
}

pub(super) fn dispatch(
    action: Action,
    root: &Path,
    selected: Option<&Path>,
    output: &OutputOptions,
    environment: &Environment,
) -> Result<i32, HostError> {
    let root = root.canonicalize()?;
    let (command, mut target, request) = match action {
        Action::Create {
            target,
            project,
            title,
        } => (
            "project create",
            target,
            RequestAction::Create { project, title },
        ),
        Action::Navigate { target, intent } => {
            let path = intent
                .map(|path| super::super::source_path(selected, path))
                .transpose()?;
            let NavigationIntent { project, request } = navigation_intent(path.as_deref())?;
            (
                "project navigate",
                target,
                RequestAction::Navigate { project, request },
            )
        }
        Action::Acknowledgement { target } => (
            "project acknowledgement",
            target,
            RequestAction::Acknowledgement,
        ),
    };
    output.validate_for::<ProjectEffectView>(command)?;
    let reader = ProductReaderSelection::open(&root, environment, &target.source.reader)
        .map_err(product_error)?;
    let state = std::path::absolute(super::super::source_path(
        selected,
        target.source.state.clone(),
    )?)?;
    let policy = RedactionPolicy::from_inputs(environment);
    let mut response = Envelope::response(
        command,
        "original project operation observed",
        ProjectEffectView {
            reader: reader.id().to_owned(),
            operation: policy.redact(&target.operation),
            target: None,
            disposition: Disposition::Unprepared,
            acknowledgement: None,
        },
    );
    response.observation.scope = "project_operation".into();
    if !reader.supports_project_perform().map_err(product_error)? {
        response.summary = "selected capture does not support project effects".into();
        response.outcome = CommandOutcome::BoundaryFailure;
        response.subject_state = SubjectState::Unprepared;
        if let (Some(program), Some(checkout)) = (root.join("scripts/pse").to_str(), root.to_str())
            && [program, checkout]
                .iter()
                .all(|word| policy.redact(word) == *word)
        {
            response.next.push(NextAction::Command {
                argv: vec![
                    program.into(),
                    "-C".into(),
                    checkout.into(),
                    "observe".into(),
                    "prepare".into(),
                ],
                reason: "prepare a captured product image with the original effect protocol".into(),
            });
        }
        return Ok(output.emit(response)?);
    }
    let request = serde_json::value::to_raw_value(&Request {
        protocol: product_reader::PROJECT_PERFORM_PROTOCOL,
        version: 1,
        operation: &target.operation,
        target: Target {
            state: state.clone().into(),
            database: target.source.database.as_deref(),
            issuing_store: &target.issuing_store,
        },
        timeout_ms: target.source.timeout_ms,
        action: request,
    })
    .map_err(io::Error::other)?;
    if request.get().len() > product_reader::PROJECT_PERFORM_REQUEST_BYTES {
        return Err(HostError::Ownership(
            "project operation exceeds its request byte bound".into(),
        ));
    }
    let reply = reader.perform_project(
        &request,
        &target.operation,
        &target.issuing_store,
        target.source.database.as_deref(),
        environment,
    );
    match reply {
        Ok(reply) => {
            response.payload.target = reply.target.map(|mut observed| {
                if target.source.database.is_none() {
                    target.source.database = Some(observed.database.clone());
                }
                observed.issuing_store = policy.redact(&observed.issuing_store);
                observed.namespace = policy.redact(&observed.namespace);
                observed.database = policy.redact(&observed.database);
                observed
            });
            response.settlement = Settlement::Complete;
            match reply.outcome {
                ProjectEffectOutcome::Acknowledged {
                    mut acknowledgement,
                } => {
                    response.subject_state = SubjectState::Succeeded;
                    response.payload.disposition = Disposition::Acknowledged;
                    acknowledgement.key = policy.redact(&acknowledgement.key);
                    acknowledgement.kind = policy.redact(&acknowledgement.kind);
                    acknowledgement.project = policy.redact(&acknowledgement.project);
                    acknowledgement.branch =
                        acknowledgement.branch.map(|value| policy.redact(&value));
                    acknowledgement.revision =
                        acknowledgement.revision.map(|value| policy.redact(&value));
                    response.payload.acknowledgement = Some(acknowledgement);
                }
                ProjectEffectOutcome::Absent => {
                    response.summary =
                        "original acknowledgement absent; effect remains unproved".into();
                    response.payload.disposition = Disposition::Absent;
                    response.subject_state = SubjectState::Unknown;
                    response.settlement = Settlement::Unknown;
                }
                ProjectEffectOutcome::Refused { failure } => {
                    response.summary = "original project operation refused".into();
                    response.payload.disposition = Disposition::Refused;
                    response.subject_state = SubjectState::Failed;
                    response.outcome = CommandOutcome::Failure;
                    response.failures.push(*failure);
                }
                ProjectEffectOutcome::Unresolved { failure } => {
                    response.payload.disposition = Disposition::Unresolved;
                    response.failures.push(*failure);
                }
            }
            response.child_exit = Some(
                if reply.cleanup.is_some()
                    || matches!(
                        response.payload.disposition,
                        Disposition::Refused | Disposition::Unresolved
                    )
                {
                    2
                } else {
                    0
                },
            );
            if let Some(cleanup) = reply.cleanup {
                response.settlement = Settlement::Unknown;
                response.outcome = CommandOutcome::BoundaryFailure;
                response.failures.push(cleanup);
            }
        }
        Err(error) => {
            response.payload.disposition = Disposition::Unresolved;
            response
                .failures
                .push(ErrorRecord::from_host(&product_error(error), &policy));
        }
    }
    if matches!(response.payload.disposition, Disposition::Unresolved) {
        response.summary = "original project operation settlement is unresolved".into();
        response.outcome = CommandOutcome::BoundaryFailure;
        response.subject_state = SubjectState::Unknown;
        response.settlement = Settlement::Unknown;
        response.observation.completeness = Completeness::Partial;
    }
    if matches!(
        response.payload.disposition,
        Disposition::Absent | Disposition::Unresolved
    ) {
        response
            .next
            .extend(recovery(&root, &state, &target, &policy));
    }
    Ok(output.emit(response)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    #[test]
    fn project_effect_selects_original_recovery_without_replaying_the_mutation() {
        let action = crate::cli::Cli::try_parse_from([
            "pse",
            "project",
            "create",
            "--reader=reader",
            "--state=/tmp/canonical state",
            "--issuing-store=issuer",
            "--operation=original",
            "--database=selected",
            "p",
            "title",
        ])
        .unwrap();
        let Some(crate::cli::Command::Project {
            action: super::super::ProjectAction::Effect(Action::Create { target, .. }),
        }) = action.command
        else {
            panic!("expected an explicit project effect");
        };
        let root = Path::new("/tmp/project root");
        let state = &target.source.state;
        let Some(NextAction::Command { argv, .. }) =
            recovery(root, state, &target, &Default::default())
        else {
            panic!("expected an exact recovery action");
        };
        assert!(argv.contains(&"acknowledgement".into()));
        assert!(argv.contains(&"--operation=original".into()));
        assert!(argv.contains(&"--state=/tmp/canonical state".into()));
        assert!(!argv.contains(&"create".into()));
        assert!(crate::cli::Cli::try_parse_from(argv).is_ok());
        let secret = RedactionPolicy::new([("TOKEN".into(), "original".into())]);
        assert!(recovery(root, state, &target, &secret).is_none());
    }
}
