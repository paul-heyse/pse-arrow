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
    canonical_projects::{Project, ProjectAcknowledgement, ProjectNavigationRequest},
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const REQUEST_BYTES: usize = 1024 * 1024;
const RESPONSE_BYTES: usize = 256 * 1024;
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
    Acknowledged { acknowledgement: Acknowledgement },
    Absent,
    Refused { failure: Box<ErrorRecord> },
    Unresolved { failure: Box<ErrorRecord> },
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
            && matches!(self.outcome, Outcome::Acknowledged { .. } | Outcome::Absent)
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
            if let Outcome::Refused { failure } | Outcome::Unresolved { failure } =
                &mut self.outcome
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
) -> Outcome {
    let project = match &action {
        Action::Create { project, .. } => Some(project.as_str()),
        Action::Navigate { project, .. } => Some(project.key.as_str()),
        Action::Acknowledgement => None,
    };
    let result = match &action {
        Action::Create { project, title } => store
            .create_project(operation, project, title)
            .await
            .map(Some),
        Action::Navigate { project, request } => store
            .project_navigate(operation, project, &request.action)
            .await
            .map(Some),
        Action::Acknowledgement => store.project_acknowledgement(operation).await,
    };
    observed_outcome(result, operation, project, policy)
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
        },
        Ok(None) => Outcome::Absent,
        Err(error) => {
            let failure = canonical_diagnostic(&error, policy);
            if project.is_none() || error.effect_is_unknown() {
                Outcome::Unresolved {
                    failure: Box::new(failure),
                }
            } else {
                refused(failure)
            }
        }
    }
}

async fn perform(request: Request, cancel: &pse_columnar::CancellationToken) -> Response {
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
    } else if cancel.is_cancelled() {
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
        let mut environment: pse_host::environment::Environment = std::env::vars_os().collect();
        for (name, value) in [
            ("PSE_PROJECT_CANONICAL_PASSWORD", &options.password),
            (
                "PSE_PROJECT_CANONICAL_SELECTION_PASSWORD",
                &options.selection_password,
            ),
        ] {
            let mut name = std::ffi::OsString::from(name);
            while environment.contains_key(&name) {
                name.push("_");
            }
            environment.insert(name, value.as_str().into());
        }
        let policy = RedactionPolicy::from_inputs(&environment);
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
        let (selected, outcome) = match selection {
            Ok(issuing_store) if issuing_store == target.issuing_store => {
                let selected = Target {
                    issuing_store: issuing_store.to_owned(),
                    namespace: store.namespace().to_owned(),
                    database: store.database().to_owned(),
                };
                let outcome = if cancel.is_cancelled() {
                    refused(local_diagnostic(
                        &HandoffError::StoppedBeforeSubmission,
                        &policy,
                    ))
                } else {
                    // Once submitted, the original request keeps its task and deadline
                    // through canonical settlement and the subsequent connection drain.
                    execute(&store, &operation, action, &policy).await
                };
                (Some(selected), outcome)
            }
            Ok(_) => (
                None,
                refused(local_diagnostic(&HandoffError::WrongIssuer, &policy)),
            ),
            Err(error) => (None, refused(canonical_diagnostic(&error, &policy))),
        };
        let cleanup = store
            .drain_owned_connection_until(tokio::time::Instant::from_std(deadline))
            .await
            .err()
            .map(|error| canonical_diagnostic(&error, &policy));
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

async fn transfer(
    input: Reader,
    output: Writer,
    cancel: &pse_columnar::CancellationToken,
) -> Result<i32> {
    let bytes = read_request(&input, cancel).await?;
    let policy = RedactionPolicy::from_environment();
    let mut response = match decode(&bytes) {
        Ok(request) => perform(request, cancel).await,
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
        let stop = pse_runtime::CancelSource::new();
        let signal = super::signals::ProcessInterrupt::new(stop.clone())?;
        let result = async {
            let input = Reader::new(std::io::stdin())?;
            let output = Writer::new(std::io::stdout())?;
            transfer(input, output, &stop.token()).await
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
        let mut response = perform(original, &pse_columnar::CancellationToken::new()).await;
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
        let cancel = pse_columnar::CancellationToken::new();
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
    }
}
