// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed project effect transport; the canonical store owns admission and settlement.
use pse_host::{
    envelope::{ErrorRecord, ObservationValue, RedactionPolicy},
    investigation::NativePath,
    observer_io::{Reader, Writer},
};
use pse_model::document::Version;
use pse_operations::{
    canonical::{CanonicalError, CanonicalOptions, CanonicalStore, operation_scope},
    canonical_artifacts::Artifact,
    canonical_projects::{Project, ProjectAcknowledgement, ProjectNavigationRequest},
};
use pse_runtime::{
    CancelSource,
    host_operations::project::{
        DEFAULT_BRANCH, DEFAULT_DOCUMENT_BYTES,
        exchange::{self, ExchangeRequest, PublicationStatus},
        handler,
    },
    tooling::reader::canonical_redaction_policy,
    workflow::{
        ProjectImportIntent, ProjectImportRecoveryReport, ProjectImportReport, WorkflowError,
    },
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const REQUEST_BYTES: usize = 1024 * 1024;
const RESPONSE_BYTES: usize = REQUEST_BYTES;
const MAX_TIMEOUT_MS: u64 = 25_000;
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_OPERATION_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum Protocol {
    #[serde(rename = "pse.project-perform")]
    ProjectPerform,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    #[serde(rename = "protocol")]
    _protocol: Protocol,
    #[serde(rename = "version")]
    _version: Version<1>,
    operation: String,
    target: Selection,
    timeout_ms: Option<u64>,
    action: Action,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    state: NativePath,
    database: Option<String>,
    issuing_store: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    Create {
        project: String,
        title: String,
    },
    Navigate {
        project: Project,
        request: ProjectNavigationRequest,
    },
    Acknowledgement,
    ImportIssue {
        input: NativePath,
        title: String,
        branch_name: Option<String>,
        maximum_document_bytes: Option<usize>,
    },
    Import {
        input: NativePath,
        intent: Box<ProjectImportIntent>,
        maximum_document_bytes: Option<usize>,
    },
    ImportRecover {
        intent: Box<ProjectImportIntent>,
        original: Box<Artifact>,
    },
    ImportOpen {
        intent: Box<ProjectImportIntent>,
    },
    Export {
        destination: NativePath,
        request: Box<ExchangeRequest>,
    },
    ExportRecover {
        destination: NativePath,
    },
}

#[derive(Debug, Serialize)]
struct Target {
    issuing_store: String,
    namespace: String,
    database: String,
}

#[derive(Debug, Serialize)]
struct Acknowledgement {
    key: String,
    kind: String,
    project: String,
    branch: Option<String>,
    revision: Option<String>,
    generation: u64,
}
impl From<ProjectAcknowledgement> for Acknowledgement {
    fn from(original: ProjectAcknowledgement) -> Self {
        Self {
            key: original.key,
            kind: original.kind,
            project: original.project,
            branch: original.branch,
            revision: original.revision,
            generation: original.generation,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Acknowledged {
        acknowledgement: Acknowledgement,
    },
    Absent,
    ImportIssued {
        intent: Box<ProjectImportIntent>,
    },
    Imported {
        report: Box<ProjectImportReport>,
    },
    ImportRecovered {
        report: Box<ProjectImportRecoveryReport>,
    },
    ImportOpened {
        source: Box<handler::OpenedProjectImport>,
    },
    Export {
        status: Box<PublicationStatus>,
    },
    Refused {
        failure: Box<ErrorRecord>,
    },
    Unresolved {
        failure: Box<ErrorRecord>,
        #[serde(skip_serializing_if = "Option::is_none")]
        import_status: Option<Box<ProjectImportReport>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        publication_status: Option<Box<PublicationStatus>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        readback_failure: Option<Box<ErrorRecord>>,
    },
}

#[derive(Debug, Serialize)]
struct Response {
    protocol: Protocol,
    version: Version<1>,
    operation: Option<String>,
    target: Option<Target>,
    outcome: Outcome,
    cleanup: Option<ErrorRecord>,
}
impl Response {
    fn new(operation: Option<String>, outcome: Outcome) -> Self {
        Self {
            protocol: Protocol::ProjectPerform,
            version: Version,
            operation,
            target: None,
            outcome,
            cleanup: None,
        }
    }

    fn exit_code(&self) -> i32 {
        if self.cleanup.is_none()
            && matches!(
                self.outcome,
                Outcome::Acknowledged { .. }
                    | Outcome::Absent
                    | Outcome::ImportIssued { .. }
                    | Outcome::Imported { .. }
                    | Outcome::ImportRecovered { .. }
                    | Outcome::ImportOpened { .. }
                    | Outcome::Export { .. }
            )
        {
            0
        } else {
            2
        }
    }

    fn encode(&mut self, policy: &RedactionPolicy) -> Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec(self)?;
        if bytes.len() > RESPONSE_BYTES {
            let reduced = |error: &mut ErrorRecord| {
                let mut limited = local_diagnostic(&HandoffError::ResponseExtent, policy);
                limited.code.clone_from(&error.code);
                limited.facts.clone_from(&error.facts);
                *error = limited;
            };
            if let Outcome::Refused { failure } | Outcome::Unresolved { failure, .. } =
                &mut self.outcome
            {
                reduced(failure);
            }
            if let Outcome::Unresolved {
                readback_failure: Some(failure),
                ..
            } = &mut self.outcome
            {
                reduced(failure);
            }
            if let Some(cleanup) = &mut self.cleanup {
                reduced(cleanup);
            }
            bytes = serde_json::to_vec(self)?;
        }
        if bytes.len() > RESPONSE_BYTES {
            return Err(HandoffError::ResponseExtent.into());
        }
        Ok(bytes)
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
enum HandoffError {
    #[error("project effect request exceeds 1 MiB")]
    #[diagnostic(code(validation::invariant))]
    RequestExtent,
    #[error("invalid project effect request at line {line}, column {column}")]
    #[diagnostic(code(validation::invariant))]
    InvalidRequest { line: usize, column: usize },
    #[error("project effect requires a nonempty operation key of at most 4096 bytes")]
    #[diagnostic(code(validation::invariant))]
    Operation,
    #[error("project effect requires an absolute deployment path and a bounded store identity")]
    #[diagnostic(code(validation::invariant))]
    Target,
    #[error("project effect requires a 1..=25000ms original allowance")]
    #[diagnostic(code(validation::invariant))]
    Allowance,
    #[error("the selected store identity differs from the original target")]
    #[diagnostic(code(validation::invariant))]
    WrongIssuer,
    #[error("project effect stopped before submission")]
    #[diagnostic(code(runtime::infrastructure))]
    StoppedBeforeSubmission,
    #[error("project acknowledgement coordinates differ from the original request")]
    #[diagnostic(code(runtime::infrastructure))]
    AcknowledgementIdentity,
    #[error("project import requires an absolute original pack path")]
    #[diagnostic(code(validation::invariant))]
    ImportPath,
    #[error("project import intent differs from the requested operation or receiving issuer")]
    #[diagnostic(code(validation::invariant))]
    ImportIdentity,
    #[error("project import recovery requires the unchanged original artifact intent")]
    #[diagnostic(code(validation::invariant))]
    ImportArtifact,
    #[error("the project effect response exceeded its transport extent")]
    #[diagnostic(code(runtime::infrastructure))]
    ResponseExtent,
}

fn local_diagnostic(error: &HandoffError, policy: &RedactionPolicy) -> ErrorRecord {
    ErrorRecord::from_diagnostic(error, None, policy)
}

fn canonical_diagnostic(error: &CanonicalError, policy: &RedactionPolicy) -> ErrorRecord {
    let mut record = ErrorRecord::from_typed(error, policy);
    let observations = &mut record.facts.observations;
    observations.insert(
        "effect_unknown".into(),
        ObservationValue::Boolean(error.effect_is_unknown()),
    );
    match error {
        CanonicalError::Refused {
            code, class, stage, ..
        } => {
            observations.insert(
                "refusal_code".into(),
                ObservationValue::Text(code.to_string()),
            );
            observations.insert(
                "refusal_class".into(),
                ObservationValue::Text(format!("{class:?}")),
            );
            observations.insert(
                "refusal_stage".into(),
                ObservationValue::Text((*stage).into()),
            );
        }
        CanonicalError::OperationReused => {
            observations.insert("operation_reused".into(), ObservationValue::Boolean(true));
        }
        CanonicalError::SettlementUnresolved {
            source, readback, ..
        } => {
            record.diagnostic_source = Some(Box::new(canonical_diagnostic(source, policy)));
            if let Some(readback) = readback {
                record.related.push(canonical_diagnostic(readback, policy));
            }
        }
        _ => {}
    }
    record
}

fn refused(failure: ErrorRecord) -> Outcome {
    Outcome::Refused {
        failure: Box::new(failure),
    }
}

async fn execute(
    store: &CanonicalStore,
    operation: &str,
    action: Action,
    policy: &RedactionPolicy,
    cancel: &CancelSource,
) -> (Outcome, Option<ErrorRecord>) {
    let outcome = match action {
        Action::Create { project, title } => observed_outcome(
            store
                .create_project(operation, &project, &title)
                .await
                .map(Some),
            operation,
            Some(&project),
            policy,
        ),
        Action::Navigate { project, request } => observed_outcome(
            store
                .project_navigate(operation, &project, &request.action)
                .await
                .map(Some),
            operation,
            Some(project.key.as_str()),
            policy,
        ),
        Action::Acknowledgement => observed_outcome(
            store.project_acknowledgement(operation).await,
            operation,
            None,
            policy,
        ),
        Action::ImportIssue {
            input,
            title,
            branch_name,
            maximum_document_bytes,
        } => {
            let input = input.to_path_buf();
            if !input.is_absolute() {
                return (
                    refused(local_diagnostic(&HandoffError::ImportPath, policy)),
                    None,
                );
            }
            let prior =
                store
                    .project_acknowledgement(operation)
                    .await
                    .and_then(|acknowledgement| {
                        if acknowledgement.is_some() {
                            Err(CanonicalError::OperationReused)
                        } else {
                            Ok(())
                        }
                    });
            if let Err(error) = prior {
                return (refused(canonical_diagnostic(&error, policy)), None);
            }
            match handler::issue_project_import(
                store,
                operation,
                &input,
                &title,
                branch_name.as_deref().unwrap_or(DEFAULT_BRANCH),
                maximum_document_bytes.unwrap_or(DEFAULT_DOCUMENT_BYTES),
            ) {
                Ok(intent) => Outcome::ImportIssued {
                    intent: Box::new(intent),
                },
                Err(error) => refused(workflow_diagnostic(&error, policy)),
            }
        }
        Action::Import {
            input,
            intent,
            maximum_document_bytes,
        } => {
            let input = input.to_path_buf();
            if !input.is_absolute() {
                return (
                    refused(local_diagnostic(&HandoffError::ImportPath, policy)),
                    None,
                );
            }
            if let Err(failure) = validate_import_identity(store, operation, &intent, policy) {
                return (Outcome::Refused { failure }, None);
            }
            if let Err(error) = handler::observe_project_import(store, &intent).await {
                // The exact acknowledgement lookup authenticates a previously used
                // key before this invocation can start a new source/artifact effect.
                return (refused(workflow_diagnostic(&error, policy)), None);
            }
            return match handler::apply_project_import(
                store,
                &input,
                &intent,
                maximum_document_bytes.unwrap_or(DEFAULT_DOCUMENT_BYTES),
                cancel,
            )
            .await
            {
                Ok(applied) => {
                    let outcome = match applied.result {
                        Ok(report) => Outcome::Imported {
                            report: Box::new(report),
                        },
                        Err(error) => unresolved_import(store, &intent, &error, policy).await,
                    };
                    let cleanup = applied
                        .cleanup
                        .map(|error| ErrorRecord::from_typed(&error, policy));
                    (outcome, cleanup)
                }
                Err(error) => (refused(ErrorRecord::from_typed(&error, policy)), None),
            };
        }
        Action::ImportRecover { intent, original } => {
            if let Err(failure) = validate_import_identity(store, operation, &intent, policy) {
                return (Outcome::Refused { failure }, None);
            }
            match handler::observe_project_import(store, &intent).await {
                Ok(Some(report))
                    if report.artifact_intent.as_ref() == Some(original.as_ref())
                        || report
                            .retired_originals
                            .iter()
                            .any(|header| header == original.as_ref()) =>
                {
                    match handler::recover_project_import(
                        store,
                        &intent,
                        &original,
                        &cancel.token(),
                    )
                    .await
                    {
                        Ok(report) => Outcome::ImportRecovered {
                            report: Box::new(report),
                        },
                        Err(error) => unresolved_import(store, &intent, &error, policy).await,
                    }
                }
                Ok(_) => refused(local_diagnostic(&HandoffError::ImportArtifact, policy)),
                Err(error) => refused(workflow_diagnostic(&error, policy)),
            }
        }
        Action::ImportOpen { intent } => {
            if let Err(failure) = validate_import_identity(store, operation, &intent, policy) {
                return (Outcome::Refused { failure }, None);
            }
            return match handler::open_project_import(store, &intent, cancel).await {
                Ok(opened) => {
                    let outcome = match opened.result {
                        Ok(source) => Outcome::ImportOpened {
                            source: Box::new(source),
                        },
                        Err(error) => refused(workflow_diagnostic(&error, policy)),
                    };
                    (
                        outcome,
                        opened
                            .cleanup
                            .map(|error| ErrorRecord::from_typed(&error, policy)),
                    )
                }
                Err(error) => (refused(ErrorRecord::from_typed(&error, policy)), None),
            };
        }
        Action::Export {
            destination,
            request,
        } => {
            return match exchange::export(store, operation, destination.clone(), *request, cancel)
                .await
            {
                Ok(exported) => {
                    let outcome = match exported.result {
                        Ok(status) => Outcome::Export {
                            status: Box::new(status),
                        },
                        Err(error) => unresolved_publication(
                            store,
                            operation,
                            destination,
                            ErrorRecord::from_typed(&error, policy),
                            policy,
                            &cancel.token(),
                        ),
                    };
                    let cleanup = exported
                        .cleanup
                        .map(|error| ErrorRecord::from_typed(&error, policy))
                        .or_else(|| {
                            exported
                                .publication_cleanup
                                .map(|error| workflow_diagnostic(&error, policy))
                        });
                    (outcome, cleanup)
                }
                Err(error) => (refused(ErrorRecord::from_typed(&error, policy)), None),
            };
        }
        Action::ExportRecover { destination } => {
            let issuer = match store.issuing_store() {
                Ok(issuer) => issuer,
                Err(error) => return (refused(canonical_diagnostic(&error, policy)), None),
            };
            match exchange::recover_publication(
                store.deployment_state(),
                destination.clone(),
                operation,
                issuer,
                &cancel.token(),
            ) {
                Ok(status) => Outcome::Export {
                    status: Box::new(status),
                },
                Err(error) => unresolved_publication(
                    store,
                    operation,
                    destination,
                    workflow_diagnostic(&error, policy),
                    policy,
                    &cancel.token(),
                ),
            }
        }
    };
    (outcome, None)
}

fn unresolved_publication(
    store: &CanonicalStore,
    operation: &str,
    destination: NativePath,
    failure: ErrorRecord,
    policy: &RedactionPolicy,
    cancel: &pse_columnar::CancellationToken,
) -> Outcome {
    let readback = store
        .issuing_store()
        .map_err(WorkflowError::from)
        .and_then(|issuer| {
            exchange::publication_status(
                store.deployment_state(),
                destination,
                operation,
                issuer,
                cancel,
            )
        });
    let (publication_status, readback_failure) = match readback {
        Ok(status) => (Some(Box::new(status)), None),
        Err(error) => (None, Some(Box::new(workflow_diagnostic(&error, policy)))),
    };
    Outcome::Unresolved {
        failure: Box::new(failure),
        import_status: None,
        publication_status,
        readback_failure,
    }
}

fn workflow_diagnostic(error: &WorkflowError, policy: &RedactionPolicy) -> ErrorRecord {
    match error {
        WorkflowError::Canonical(error) => canonical_diagnostic(error, policy),
        _ => ErrorRecord::from_typed(error, policy),
    }
}

fn validate_import_identity(
    store: &CanonicalStore,
    operation: &str,
    intent: &ProjectImportIntent,
    policy: &RedactionPolicy,
) -> std::result::Result<(), Box<ErrorRecord>> {
    intent
        .validate()
        .map_err(|error| Box::new(canonical_diagnostic(&error, policy)))?;
    let issuing_store = store
        .issuing_store()
        .map_err(|error| Box::new(canonical_diagnostic(&error, policy)))?;
    if intent.operation() != operation || intent.issuing_store() != issuing_store {
        return Err(Box::new(local_diagnostic(
            &HandoffError::ImportIdentity,
            policy,
        )));
    }
    Ok(())
}

async fn unresolved_import(
    store: &CanonicalStore,
    intent: &ProjectImportIntent,
    error: &WorkflowError,
    policy: &RedactionPolicy,
) -> Outcome {
    let (import_status, readback_failure) =
        match handler::observe_project_import(store, intent).await {
            Ok(report) => (report.map(Box::new), None),
            Err(error) => (None, Some(Box::new(workflow_diagnostic(&error, policy)))),
        };
    Outcome::Unresolved {
        failure: Box::new(workflow_diagnostic(error, policy)),
        import_status,
        publication_status: None,
        readback_failure,
    }
}

fn observed_outcome(
    result: std::result::Result<Option<ProjectAcknowledgement>, CanonicalError>,
    operation: &str,
    project: Option<&str>,
    policy: &RedactionPolicy,
) -> Outcome {
    match result {
        Ok(Some(ack))
            if ack.key == operation && project.is_none_or(|project| ack.project == project) =>
        {
            Outcome::Acknowledged {
                acknowledgement: ack.into(),
            }
        }
        Ok(Some(_)) => Outcome::Unresolved {
            failure: Box::new(local_diagnostic(
                &HandoffError::AcknowledgementIdentity,
                policy,
            )),
            import_status: None,
            publication_status: None,
            readback_failure: None,
        },
        Ok(None) => Outcome::Absent,
        Err(error) => {
            let failure = canonical_diagnostic(&error, policy);
            if project.is_none() || error.effect_is_unknown() {
                Outcome::Unresolved {
                    failure: Box::new(failure),
                    import_status: None,
                    publication_status: None,
                    readback_failure: None,
                }
            } else {
                refused(failure)
            }
        }
    }
}

async fn perform(request: Request, cancel: &CancelSource) -> Response {
    let Request {
        operation,
        target,
        timeout_ms,
        action,
        ..
    } = request;
    let policy = RedactionPolicy::from_environment();
    let state = target.state.to_path_buf();
    let timeout_ms = timeout_ms.unwrap_or(MAX_TIMEOUT_MS);
    let safe_operation = (operation.len() <= MAX_OPERATION_BYTES).then_some(operation.as_str());
    let invalid = if safe_operation.is_none() || operation.is_empty() {
        Some(HandoffError::Operation)
    } else if !state.is_absolute()
        || !(1..=MAX_OPERATION_BYTES).contains(&target.issuing_store.len())
    {
        Some(HandoffError::Target)
    } else if !(1..=MAX_TIMEOUT_MS).contains(&timeout_ms) {
        Some(HandoffError::Allowance)
    } else if cancel.token().is_cancelled() {
        Some(HandoffError::StoppedBeforeSubmission)
    } else {
        None
    };
    if let Some(error) = invalid {
        return Response::new(
            safe_operation.map(str::to_owned),
            refused(local_diagnostic(&error, &policy)),
        );
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let cancel = cancel.with_deadline(Some(deadline));
    operation_scope(deadline, async {
        let options =
            match CanonicalOptions::from_state_for_database(&state, target.database.as_deref()) {
                Ok(options) => options,
                Err(error) => {
                    return Response::new(
                        Some(operation),
                        refused(canonical_diagnostic(&error, &policy)),
                    );
                }
            };
        let policy = canonical_redaction_policy(&options);
        let store = match CanonicalStore::connect(&options).await {
            Ok(store) => store,
            Err(error) => {
                return Response::new(
                    Some(operation),
                    refused(canonical_diagnostic(&error, &policy)),
                );
            }
        };
        let selection = store.open().await.and_then(|()| store.issuing_store());
        let (selected, outcome, native_cleanup) = match selection {
            Ok(issuing_store) if issuing_store == target.issuing_store => {
                let selected = Target {
                    issuing_store: issuing_store.to_owned(),
                    namespace: store.namespace().to_owned(),
                    database: store.database().to_owned(),
                };
                let (outcome, cleanup) = if cancel.token().is_cancelled() {
                    (
                        refused(local_diagnostic(
                            &HandoffError::StoppedBeforeSubmission,
                            &policy,
                        )),
                        None,
                    )
                } else {
                    // Once submitted, the original request keeps its task and deadline
                    // through canonical settlement and the subsequent connection drain.
                    execute(&store, &operation, action, &policy, &cancel).await
                };
                (Some(selected), outcome, cleanup)
            }
            Ok(_) => (
                None,
                refused(local_diagnostic(&HandoffError::WrongIssuer, &policy)),
                None,
            ),
            Err(error) => (None, refused(canonical_diagnostic(&error, &policy)), None),
        };
        let canonical_cleanup = store
            .drain_owned_connection_until(tokio::time::Instant::from_std(deadline))
            .await
            .err()
            .map(|error| canonical_diagnostic(&error, &policy));
        let cleanup = match (native_cleanup, canonical_cleanup) {
            (Some(mut native), Some(canonical)) => {
                native.related.push(canonical);
                Some(native)
            }
            (native, canonical) => native.or(canonical),
        };
        Response {
            target: selected,
            cleanup,
            ..Response::new(Some(operation), outcome)
        }
    })
    .await
}

async fn read_request(input: &Reader, cancel: &pse_columnar::CancellationToken) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 16 << 10];
    while bytes.len() <= REQUEST_BYTES {
        let remaining = (REQUEST_BYTES + 1 - bytes.len()).min(chunk.len());
        let count = cancel
            .until_cancelled(input.read(&mut chunk[..remaining]))
            .await??;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
        tokio::task::yield_now().await;
    }
    Ok(bytes)
}

fn decode(bytes: &[u8]) -> std::result::Result<Request, HandoffError> {
    if bytes.len() > REQUEST_BYTES {
        return Err(HandoffError::RequestExtent);
    }
    serde_json::from_slice(bytes).map_err(|error| HandoffError::InvalidRequest {
        line: error.line(),
        column: error.column(),
    })
}

async fn transfer(input: Reader, output: Writer, cancel: &CancelSource) -> Result<i32> {
    let bytes = read_request(&input, &cancel.token()).await?;
    let policy = RedactionPolicy::from_environment();
    let mut response = match decode(&bytes) {
        Ok(request) => {
            let cancel = cancel.clone();
            // Pack verification and archive transfer can use synchronous contained
            // I/O. Keep the process signal listener runnable and join the owner.
            tokio::task::spawn_blocking(move || {
                let executor = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                Ok::<_, std::io::Error>(executor.block_on(perform(request, &cancel)))
            })
            .await??
        }
        Err(error) => Response::new(None, refused(local_diagnostic(&error, &policy))),
    };
    let bytes = response.encode(&policy)?;
    // A signal requests no fresh effect. A listener still awaiting the original
    // result can receive it, even when that signal arrived after submission.
    tokio::time::timeout(DELIVERY_TIMEOUT, output.write(&bytes)).await??;
    Ok(response.exit_code())
}

pub(super) fn run() -> Result<i32> {
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(async {
        let stop = CancelSource::new();
        let signal = super::signals::ProcessInterrupt::new(stop.clone())?;
        let result = async {
            let input = Reader::new(std::io::stdin())?;
            let output = Writer::new(std::io::stdout())?;
            transfer(input, output, &stop).await
        }
        .await;
        signal.finish().await;
        result
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        reason = "the bounded protocol and isolated store controls require exact fixture outcomes"
    )]
    use super::*;
    use serde_json::json;

    fn request(
        state: &NativePath,
        database: Option<&str>,
        issuer: &str,
        operation: &str,
        action: serde_json::Value,
    ) -> Request {
        let bytes = serde_json::to_vec(&json!({
            "protocol": "pse.project-perform",
            "version": 1,
            "operation": operation,
            "target": {"state": state, "database": database, "issuing_store": issuer},
            "action": action,
        }))
        .unwrap();
        decode(&bytes).unwrap()
    }

    #[tokio::test]
    async fn project_perform_refuses_invalid_selection_without_opening_it() {
        let original = request(
            &NativePath::Utf8("relative".into()),
            None,
            "074e38e9-7afe-4832-9770-3c98a1c0435b",
            "original",
            json!({"kind": "create", "project": "target", "title": "title"}),
        );
        let mut response = perform(original, &CancelSource::new()).await;
        let bytes = response.encode(&RedactionPolicy::default()).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response.exit_code(), 2);
        assert_eq!(payload["protocol"], "pse.project-perform");
        assert_eq!(payload["version"], 1);
        assert_eq!(payload["operation"], "original");
        assert!(payload["target"].is_null());
        assert_eq!(payload["outcome"]["kind"], "refused");
        assert_eq!(
            payload["outcome"]["failure"]["code"],
            "validation::invariant"
        );
        assert!(matches!(
            decode(&vec![b' '; REQUEST_BYTES + 1]),
            Err(HandoffError::RequestExtent)
        ));

        let root = tempfile::tempdir().unwrap();
        let absent = root.path().join("no-configured-service");
        let issue = request(
            &NativePath::from(absent.clone()),
            None,
            "074e38e9-7afe-4832-9770-3c98a1c0435b",
            "exact-import",
            json!({"kind": "import_issue", "input": NativePath::from(root.path().join("pack")), "title": "Original"}),
        );
        assert!(matches!(
            &issue.action,
            Action::ImportIssue {
                maximum_document_bytes: None,
                ..
            }
        ));
        let response = perform(issue, &CancelSource::new()).await;
        assert!(matches!(response.outcome, Outcome::Refused { .. }));
        assert!(response.target.is_none());
        assert!(!absent.exists());
    }

    #[test]
    fn project_perform_keeps_uncertain_settlement_and_omits_opaque_request() {
        let error = CanonicalError::SettlementUnresolved {
            owner: "project",
            operation: "original".into(),
            request: b"private-original-bytes".to_vec(),
            source: Box::new(CanonicalError::Timeout),
            readback: Some(Box::new(CanonicalError::ConnectionUnavailable)),
        };
        let policy = RedactionPolicy::default();
        let outcome = observed_outcome(Err(error), "original", Some("project"), &policy);
        let mut response = Response::new(Some("original".into()), outcome);
        let bytes = response.encode(&policy).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["outcome"]["kind"], "unresolved");
        assert_eq!(
            value["outcome"]["failure"]["facts"]["observations"]["effect_unknown"]["value"],
            true
        );
        assert!(value["outcome"]["failure"]["diagnostic_source"].is_object());
        assert_eq!(
            value["outcome"]["failure"]["related"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            !std::str::from_utf8(&bytes)
                .unwrap()
                .contains("private-original-bytes")
        );
        assert_eq!(response.exit_code(), 2);
    }

    #[tokio::test]
    async fn project_perform_keeps_original_issuer_operation_and_receipt() {
        use pse_operations::native_fixture::{FixtureSelection, canonical_fixture_store};
        let fixture = FixtureSelection::selected().unwrap();
        let store = canonical_fixture_store().unwrap();
        store.open().await.unwrap();
        let state = NativePath::from(fixture.state().to_path_buf());
        let issuer = store.issuing_store().unwrap();
        let database = Some(store.database());
        let cancel = CancelSource::new();
        let create =
            json!({"kind": "create", "project": "original-project", "title": "original title"});

        let wrong_issuer = uuid::Uuid::new_v4().to_string();
        let response = perform(
            request(
                &state,
                database,
                &wrong_issuer,
                "original-create",
                create.clone(),
            ),
            &cancel,
        )
        .await;
        assert!(matches!(response.outcome, Outcome::Refused { .. }));
        assert!(response.target.is_none());
        assert!(store.project("original-project").await.unwrap().is_none());
        assert!(
            store
                .project_acknowledgement("original-create")
                .await
                .unwrap()
                .is_none()
        );

        let response = perform(
            request(&state, database, issuer, "original-create", create.clone()),
            &cancel,
        )
        .await;
        let Outcome::Acknowledged { acknowledgement } = &response.outcome else {
            panic!("original create: {response:?}");
        };
        assert_eq!(acknowledgement.key, "original-create");
        assert_eq!(acknowledgement.project, "original-project");
        assert_eq!(response.target.as_ref().unwrap().issuing_store, issuer);
        let mut original = response;
        let policy = RedactionPolicy::default();
        let original_receipt: serde_json::Value =
            serde_json::from_slice(&original.encode(&policy).unwrap()).unwrap();
        assert!(
            original_receipt["outcome"]["acknowledgement"]
                .get("request")
                .is_none()
        );

        let replay = perform(
            request(&state, database, issuer, "original-create", create),
            &cancel,
        )
        .await;
        assert!(matches!(replay.outcome, Outcome::Acknowledged { .. }));
        let changed =
            json!({"kind": "create", "project": "original-project", "title": "changed title"});
        let duplicate = perform(
            request(&state, database, issuer, "original-create", changed),
            &cancel,
        )
        .await;
        let Outcome::Refused { failure } = duplicate.outcome else {
            panic!("changed duplicate: {duplicate:?}");
        };
        assert!(matches!(
            failure.facts.observations.get("operation_reused"),
            Some(ObservationValue::Boolean(true))
        ));

        let project = store.project("original-project").await.unwrap().unwrap();
        let navigate = json!({
            "kind": "navigate",
            "project": project,
            "request": {"version": 1, "action": {"kind": "project_title", "name": "updated title"}},
        });
        let response = perform(
            request(
                &state,
                database,
                issuer,
                "original-navigate",
                navigate.clone(),
            ),
            &cancel,
        )
        .await;
        assert!(matches!(response.outcome, Outcome::Acknowledged { .. }));
        let response = perform(
            request(&state, database, issuer, "stale-navigate", navigate),
            &cancel,
        )
        .await;
        let Outcome::Refused { failure } = response.outcome else {
            panic!("stale navigation: {response:?}");
        };
        assert!(matches!(
            failure.facts.observations.get("refusal_code"),
            Some(ObservationValue::Text(code)) if code == "PSE_REFUSAL:project_generation_changed"
        ));
        assert_eq!(
            store
                .project("original-project")
                .await
                .unwrap()
                .unwrap()
                .title,
            "updated title"
        );

        let mut recovered = perform(
            request(
                &state,
                database,
                issuer,
                "original-create",
                json!({"kind": "acknowledgement"}),
            ),
            &cancel,
        )
        .await;
        let recovered_receipt: serde_json::Value =
            serde_json::from_slice(&recovered.encode(&policy).unwrap()).unwrap();
        assert_eq!(recovered_receipt["outcome"], original_receipt["outcome"]);
        let absent = perform(
            request(
                &state,
                database,
                issuer,
                "stale-navigate",
                json!({"kind": "acknowledgement"}),
            ),
            &cancel,
        )
        .await;
        assert!(matches!(absent.outcome, Outcome::Absent));

        let original_import = store
            .project_import_intent(
                "original-import",
                "receiving-project",
                "receiving-branch",
                "receiving-problem",
                &"a".repeat(64),
                b"{}".to_vec(),
                "Original",
                DEFAULT_BRANCH,
            )
            .unwrap();
        let original_intent = serde_json::to_value(&original_import).unwrap();
        assert_eq!(
            original_intent["publication"]["operation"],
            "original-import"
        );
        assert_eq!(original_intent["publication"]["issuing_store"], issuer);
        let input = tempfile::tempdir().unwrap();
        let missing_pack = input.path().join("absent");
        let reused_issue = perform(
            request(&state, database, issuer, "original-create", json!({
                "kind": "import_issue", "input": NativePath::from(missing_pack.clone()), "title": "Changed",
            })), &cancel,
        ).await;
        let Outcome::Refused { failure } = reused_issue.outcome else {
            panic!("reused import issue key: {reused_issue:?}");
        };
        assert!(matches!(
            failure.facts.observations.get("operation_reused"),
            Some(ObservationValue::Boolean(true))
        ));
        let import = json!({
            "kind": "import", "input": NativePath::from(missing_pack.clone()),
            "intent": original_intent,
        });
        let wrong_identity = perform(
            request(&state, database, issuer, "different-import", import.clone()),
            &cancel,
        )
        .await;
        let Outcome::Refused { failure } = wrong_identity.outcome else {
            panic!("wrong import identity: {wrong_identity:?}");
        };
        assert!(failure.message.contains("intent differs"));
        let missing_input = perform(
            request(&state, database, issuer, "original-import", import),
            &cancel,
        )
        .await;
        assert!(matches!(missing_input.outcome, Outcome::Refused { .. }));
        assert!(!missing_pack.exists());
        assert!(
            store
                .project_import_record(&original_import)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.project("receiving-project").await.unwrap().is_none());
    }
}
