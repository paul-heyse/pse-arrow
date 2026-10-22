// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Offline executable extraction and a separately invoked new-work runner.
//! Extraction reads recorded codecs only; source reconstruction/admission belongs to run().
use super::{
    BoundaryDiagnostic, DiagnosticCommandRequest, DiagnosticExecutionRequest, DiagnosticOperation,
    DiagnosticOrigin, ReplayCaseConstructor, ReplaySourceDocument, ReviewContextDocument, Runtime,
    WorkflowError, contract,
};
use crate::CancelSource;
use futures_util::FutureExt;
use pse_diagnostics::observation::{OperationId, Outcome};
use pse_model::{
    diagnostic::{DiagnosticCode, DiagnosticRule, DiagnosticStage, Observation, ValidityLineage},
    document::Version,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};
const MAXIMUM_BYTES: usize = 64 << 20;

/// Exact typed failure predicate. No message parsing or broad exception matching.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReproductionPredicate {
    /// Relevant detailed diagnostic identity.
    pub code: DiagnosticCode,
    /// Original operation stage.
    pub stage: DiagnosticStage,
    /// Original named contract.
    pub rule: DiagnosticRule,
    /// Exact selected original source population, including explicit absence.
    pub sources: Vec<pse_ids::SemanticId>,
    /// Exact recorded validity lineage, when present.
    pub validity: Option<ValidityLineage>,
    /// Explicit necessary observed conditions. Exact values; no implicit tolerance.
    pub conditions: BTreeMap<String, Observation>,
}
impl ReproductionPredicate {
    /// Derive only from a selected typed original diagnostic, never a failure string.
    pub fn from_diagnostic(original: &BoundaryDiagnostic) -> Self {
        Self {
            code: original.code,
            stage: original.stage,
            rule: original.rule,
            sources: original.sources.clone(),
            validity: original.validity.clone(),
            conditions: original.observations.clone(),
        }
    }
    fn matches(&self, diagnostic: &BoundaryDiagnostic) -> bool {
        fn exact<T: Serialize>(left: &T, right: &T) -> bool {
            match (serde_json::to_vec(left), serde_json::to_vec(right)) {
                (Ok(left), Ok(right)) => left == right,
                _ => false,
            }
        }
        self.code == diagnostic.code
            && self.stage == diagnostic.stage
            && self.rule == diagnostic.rule
            && self.sources == diagnostic.sources
            && exact(&self.validity, &diagnostic.validity)
            && self.conditions.iter().all(|(key, value)| {
                diagnostic
                    .observations
                    .get(key)
                    .is_some_and(|actual| exact(value, actual))
            })
    }
    fn supported_family(&self) -> bool {
        // A numerical point cannot encode cancellation or readiness schedules.
        // Detailed producer codes identify such cases; no failure-message classification.
        !matches!(
            self.code.class(),
            pse_diagnostics::FailureClass::RuntimeCancelled
                | pse_diagnostics::FailureClass::RuntimeTimeout
                | pse_diagnostics::FailureClass::RuntimeResourceLimit
        ) && self.code != DiagnosticCode::StudyDependencyUnusable
    }
}
/// Selected execution family; source text and frozen codecs remain distinct.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "family", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReproducerInput {
    /// Actual original authored documents with the existing diagnostic constructor.
    OriginalText {
        /// Exact existing original-source diagnostic request.
        request: Box<DiagnosticExecutionRequest>,
    },
    /// Complete frozen algebraic constructor and source; runtime factory checks remain live.
    FrozenAlgebraic {
        /// Complete original source codec.
        source: Box<ReplaySourceDocument>,
        /// Exact original requested constructor.
        constructor: Box<ReplayCaseConstructor>,
    },
}
/// Portable new request, independent of original outcome and scientific permission.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReproducer {
    /// Executable request codec.
    pub version: Version<1>,
    /// Original text or exact selected frozen codec.
    pub input: ReproducerInput,
    /// Explicit finite allowance for all reconstruction/admission/execution.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub allowance: Duration,
    /// Original attribution; absent when the original failed before a run existed.
    pub origin: Option<DiagnosticOrigin>,
    /// Original operation, including failures before an original run handle existed.
    pub original_operation: Option<OperationId>,
    /// Independently retained typed original failure, if actually supplied.
    pub original_diagnostic: Option<BoundaryDiagnostic>,
    /// Original descriptive failure text, never promoted to a typed predicate.
    pub original_failure_text: Option<String>,
    /// Selected exact typed predicate; absence permits running but no reproduction claim.
    pub predicate: Option<ReproductionPredicate>,
    /// Explicit runtime premises; descriptors do not locate executable factories.
    pub required_environment: Vec<String>,
}
/// Explicit original recording selected for offline extraction.
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "family", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReproducerExtractionInput {
    /// Actual original text and its existing scientific diagnostic constructor.
    OriginalText {
        /// Complete existing original-source request.
        request: Box<DiagnosticExecutionRequest>,
    },
    /// Complete selected retained source and algebraic constructor.
    FrozenReviewCase {
        /// Descriptive frozen facts; no executable capability is supplied by this type.
        context: Box<ReviewContextDocument>,
        /// Exact original selected step.
        step: usize,
        /// Explicit new-run allowance, not an original measured duration.
        #[schemars(with = "pse_model::document::ClosedDuration")]
        allowance: Duration,
    },
}
/// Offline extraction request. Decoding does not construct a Runtime or touch a store.
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReproducerExtractionRequest {
    /// Extraction codec.
    pub version: Version<1>,
    /// Explicit deployment controls forwarded to the later driver.
    pub runtime: super::DiagnosticRuntimeSettings,
    /// Exact selected recorded input.
    pub input: ReproducerExtractionInput,
    /// Optional selected typed original; failure text is insufficient.
    pub original_diagnostic: Option<BoundaryDiagnostic>,
    /// Explicit original operation, including a pre-handle original failure.
    pub original_operation: Option<OperationId>,
    /// Explicit strict predicate when the typed original is unavailable.
    pub predicate: Option<ReproductionPredicate>,
}
impl ReproducerExtractionRequest {
    /// Transform recorded data only. No reconstruction, admission or scientific work.
    pub fn extract(self) -> Result<ReproducerCommandRequest, WorkflowError> {
        let original_operation = self.original_operation;
        let mut result = match self.input {
            ReproducerExtractionInput::OriginalText { request } => {
                DiagnosticReproducer::extract_text(
                    DiagnosticCommandRequest {
                        version: Version,
                        runtime: self.runtime,
                        operation: *request,
                    },
                    self.original_diagnostic,
                    self.predicate,
                )?
            }
            ReproducerExtractionInput::FrozenReviewCase {
                context,
                step,
                allowance,
            } => ReproducerCommandRequest {
                version: Version,
                runtime: self.runtime,
                reproducer: DiagnosticReproducer::extract_review_case(
                    &context,
                    step,
                    allowance,
                    self.original_diagnostic,
                    self.predicate,
                )?,
            },
        };
        if let Some(operation) = original_operation {
            if result
                .reproducer
                .original_operation
                .is_some_and(|recorded| recorded != operation)
            {
                return Err(contract(
                    "explicit original operation conflicts with recorded original attribution",
                ));
            }
            result.reproducer.original_operation = Some(operation);
        }
        Ok(result)
    }
}
/// Readable deployment/request driver; writing it does not execute it.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReproducerCommandRequest {
    /// Driver codec.
    pub version: Version<1>,
    /// Existing finite deployment controls.
    pub runtime: super::DiagnosticRuntimeSettings,
    /// Explicit new-work request.
    pub reproducer: DiagnosticReproducer,
}
/// Strict comparison outcome, independent of the new run's scientific qualification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReproductionStatus {
    /// The selected exact predicate from the typed original matched.
    OriginalFailureReproduced,
    /// An explicit predicate matched, without a matching typed original.
    RelatedDiagnosticReproduced,
    /// Supported predicate was evaluated and did not match.
    NotReproduced,
    /// Original typing/predicate or actual execution prerequisites are unavailable.
    ReproductionUnavailable,
}
/// New operation identity exists even if reconstruction fails before run creation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReproductionDocument {
    /// Result codec.
    pub version: Version<1>,
    /// Fresh operation, established before any reconstruction/admission.
    pub operation: OperationId,
    /// Effective inherited deadline measured from this new operation establishment.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub effective_allowance: Duration,
    /// Complete request preserves original outcome/predicate independently.
    pub original: DiagnosticReproducer,
    /// Fresh scientific run, absent on pre-handle refusal.
    pub run_id: Option<pse_model::generated::identities::RunId>,
    /// Findings actually returned by this explicit execution.
    pub findings: Vec<BoundaryDiagnostic>,
    /// Actual pre-handle or execution failure.
    pub failure: Option<BoundaryDiagnostic>,
    /// Strict comparison outcome.
    pub status: ReproductionStatus,
    /// Exact availability explanation; a new failure is not automatically the original.
    pub comparison_reason: String,
}
fn allowance(value: Duration) -> Result<(), WorkflowError> {
    if value.is_zero() || value > Duration::from_secs(86_400) {
        Err(contract(
            "reproducer requires a positive allowance up to one day",
        ))
    } else {
        Ok(())
    }
}
fn extent(value: &impl Serialize) -> Result<(), WorkflowError> {
    struct Limit(usize);
    impl std::io::Write for Limit {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .filter(|bytes| *bytes <= MAXIMUM_BYTES)
                .ok_or_else(|| std::io::Error::other("reproducer exceeds finite encoded extent"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Limit(0), value).map_err(|error| contract(error.to_string()))
}
fn predicate(
    original: &Option<BoundaryDiagnostic>,
    explicit: Option<ReproductionPredicate>,
) -> Option<ReproductionPredicate> {
    explicit.or_else(|| {
        original
            .as_ref()
            .map(ReproductionPredicate::from_diagnostic)
    })
}
impl DiagnosticReproducer {
    /// Pure extraction from actual text already supplied by its owner; no parser/admission runs.
    pub fn extract_text(
        command: DiagnosticCommandRequest,
        original: Option<BoundaryDiagnostic>,
        explicit: Option<ReproductionPredicate>,
    ) -> Result<ReproducerCommandRequest, WorkflowError> {
        let request = command.operation;
        allowance(request.allowance)?;
        if request.physical_documents.is_empty()
            || request.model_documents.is_empty()
            || request.maximum_source_bytes == 0
            || request.maximum_source_bytes > MAXIMUM_BYTES
        {
            return Err(contract(
                "reproducer requires complete explicit original text and source allowance",
            ));
        }
        for name in request.physical_documents.keys().chain(
            request
                .model_documents
                .iter()
                .flat_map(|documents| documents.keys()),
        ) {
            if std::path::Path::new(name).is_absolute()
                || std::path::Path::new(name)
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
            {
                return Err(contract(
                    "reproducer source filenames must be local names, not original filesystem paths",
                ));
            }
        }
        extent(&request)?;
        let reproducer = Self { version: Version, allowance: request.allowance, original_operation: request.origin.as_ref().and_then(|origin| origin.operation), origin: request.origin.clone(),
            predicate: predicate(&original,explicit), original_diagnostic: original, original_failure_text: None,
            input: ReproducerInput::OriginalText { request: Box::new(request) }, required_environment: vec!["Existing selected runtime and native diagnostic capabilities; no automatic store hydration or original-path lookup".into()] };
        Ok(ReproducerCommandRequest {
            version: Version,
            runtime: command.runtime,
            reproducer,
        })
    }
    /// Pure selected frozen extraction. Recorded completeness is checked independently of
    /// actual build/provider eligibility, which only the explicit runner may establish.
    pub fn extract_review_case(
        context: &ReviewContextDocument,
        step: usize,
        allowance_value: Duration,
        original: Option<BoundaryDiagnostic>,
        explicit: Option<ReproductionPredicate>,
    ) -> Result<Self, WorkflowError> {
        allowance(allowance_value)?;
        extent(context)?;
        context.validate()?;
        let path = format!("/steps/{step}/constructor");
        let mut facts = context
            .facts
            .iter()
            .filter(|fact| fact.path == path && fact.codec == "replay-case-constructor.v2");
        let fact = facts
            .next()
            .ok_or_else(|| contract("selected complete algebraic constructor not recorded"))?;
        if facts.next().is_some() {
            return Err(contract("selected constructor is ambiguous"));
        }
        let constructor: ReplayCaseConstructor = serde_json::from_slice(&fact.payload)
            .map_err(|error| contract(format!("recorded constructor codec: {error}")))?;
        let index = context
            .source_positions
            .iter()
            .position(|position| *position == constructor.source)
            .ok_or_else(|| contract("selected constructor source was not recorded"))?;
        let recorded = context
            .sources
            .get(index)
            .ok_or_else(|| contract("recorded source slot absent"))?;
        if !recorded.providers.is_empty() {
            return Err(contract(
                "frozen extraction requires explicit executable provider rebinding; descriptive provider configuration is not a factory",
            ));
        }
        let source = ReplaySourceDocument {
            identity: recorded.identity,
            interpretation: recorded.interpretation.clone(),
            build: recorded.build,
            physical_identity: recorded.physical_identity,
            physical_sources: recorded.physical_sources.clone(),
            providers: vec![],
            objects: recorded.objects.clone(),
        };
        recorded_source_complete(&source)?;
        if context.unavailable.iter().any(|missing| {
            missing.path == path || missing.path.starts_with(&format!("/sources/{index}/"))
        }) {
            return Err(contract(
                "selected frozen source/constructor has recorded completeness gaps",
            ));
        }
        let origin = context.run_id.map(|run_id| DiagnosticOrigin {
            run_id,
            attempt_id: context.attempt_id,
            step: Some(step),
            native_attempt: None,
            operation: context.operation,
            failure: original.clone(),
        });
        Ok(Self {
            version: Version,
            input: ReproducerInput::FrozenAlgebraic {
                source: Box::new(source),
                constructor: Box::new(constructor),
            },
            allowance: allowance_value,
            origin,
            original_operation: context.operation,
            predicate: predicate(&original, explicit),
            original_diagnostic: original,
            original_failure_text: context.failure.clone(),
            required_environment: vec![
                format!(
                    "Linked native build {}; registry interpretation {}; executable providers: none (opaque descriptors require explicit rebinding)",
                    recorded.build, recorded.interpretation
                ),
                "Complete embedded physical IPC/source objects; no original paths or store lookup"
                    .into(),
            ],
        })
    }
}
fn recorded_source_complete(source: &ReplaySourceDocument) -> Result<(), WorkflowError> {
    if source.objects.is_empty() || source.physical_sources.is_empty() {
        return Err(contract(
            "complete embedded source objects and physical IPC are required",
        ));
    }
    let mut names = BTreeSet::new();
    let mut logical = BTreeSet::new();
    let mut relations = BTreeSet::new();
    for object in &source.objects {
        if object.payload.is_empty()
            || !logical.insert(object.logical.as_str())
            || !names.insert((object.scope.as_str(), object.name.as_str()))
        {
            return Err(contract("recorded source objects are empty or duplicated"));
        }
    }
    if source
        .objects
        .iter()
        .flat_map(|object| &object.references)
        .any(|(scope, name)| !names.contains(&(scope.as_str(), name.as_str())))
    {
        return Err(contract("recorded immutable source reference is missing"));
    }
    for required in [
        "modeling:context",
        "modeling:physical-scope",
        "source:ingress-frame:v1",
    ] {
        if !logical.contains(required) {
            return Err(contract(
                "required recorded source context/frame is missing",
            ));
        }
    }
    for physical in &source.physical_sources {
        // Stream framing only, without decoding arrays, physical admission or numerical construction.
        if !relations.insert(physical.relation)
            || !physical.ipc.starts_with(&[255, 255, 255, 255])
            || !physical.ipc.ends_with(&[255, 255, 255, 255, 0, 0, 0, 0])
        {
            return Err(contract(
                "recorded physical IPC stream is missing, duplicated or incomplete",
            ));
        }
    }
    Ok(())
}

impl Runtime {
    /// Explicit new execution. Its operation/deadline precede every reconstruction,
    /// physical/source admission and constructor call, including pre-run failures.
    pub async fn run_reproducer(
        &self,
        request: DiagnosticReproducer,
        parent: &CancelSource,
    ) -> Result<ReproductionDocument, WorkflowError> {
        allowance(request.allowance)?;
        let operation =
            DiagnosticOperation::new(parent, request.allowance, request.origin.clone())?;
        let observer = pse_diagnostics::observation::current().map_or_else(
            || self.observations().hub().start("reproducer.explicit_run"),
            |parent| {
                parent.child(
                    "reproducer.explicit_run",
                    pse_diagnostics::Relationship::Nested,
                )
            },
        );
        let context = observer.context();
        let mut document = ReproductionDocument {
            version: Version,
            operation: context.id(),
            effective_allowance: operation
                .driver()
                .deadline()
                .map_or(Duration::ZERO, |deadline| {
                    deadline.saturating_duration_since(std::time::Instant::now())
                }),
            original: request,
            run_id: None,
            findings: vec![],
            failure: None,
            status: ReproductionStatus::ReproductionUnavailable,
            comparison_reason: String::new(),
        };
        let mut premises_established = matches!(
            document.original.input,
            ReproducerInput::OriginalText { .. }
        );
        let mut execution_complete = false;
        let outcome = pse_diagnostics::observation::scope(
            Some(context),
            operation.within(self.scope_activity(async {
                operation.check()?;
                extent(&document.original)?;
                match &document.original.input {
                    ReproducerInput::OriginalText { request } => {
                        // Keep the independent execution family's state on the heap. Nested
                        // preparation futures otherwise multiply the caller-thread poll frame
                        // at the synchronous Python boundary.
                        let result = self
                            .diagnose_documents(request.as_ref().clone(), operation.driver())
                            .boxed()
                            .await?;
                        Ok((
                            result.run_id,
                            result.findings,
                            result.failure,
                            result.complete,
                        ))
                    }
                    ReproducerInput::FrozenAlgebraic {
                        source,
                        constructor,
                    } => {
                        recorded_source_complete(source)?;
                        let physical = self
                            .reproducer_physical(source, operation.driver())
                            .boxed()
                            .await?;
                        operation.check()?;
                        let package = source.reconstruct(
                            self,
                            physical,
                            BTreeMap::new(),
                            operation.driver(),
                        )?;
                        operation.check()?;
                        premises_established = true;
                        let prepared = constructor
                            .reconstruct(&package, operation.driver())
                            .boxed()
                            .await?;
                        operation.check()?;
                        let handle = self
                            .start_modeling(vec![prepared], false, operation.driver())
                            .boxed()
                            .await?;
                        let run_id = handle.run_id();
                        document.run_id = Some(run_id);
                        let result = handle.wait().boxed().await?;
                        operation.check()?;
                        let failure = result
                            .report()
                            .err()
                            .map(|error| error.boundary_diagnostic());
                        let complete = failure.is_none();
                        Ok((
                            Some(run_id),
                            result.capture_diagnostics(),
                            failure,
                            complete,
                        ))
                    }
                }
            })),
        )
        .await;
        match outcome {
            Ok((run_id, findings, failure, complete)) => {
                document.run_id = run_id;
                document.findings = findings;
                document.failure = failure;
                execution_complete = complete;
            }
            Err(error) => {
                document.failure = Some(error.boundary_diagnostic());
            }
        }
        // Original text-only errors and unsupported schedules cannot acquire typed meaning.
        if let Some(predicate) = &document.original.predicate {
            if !premises_established {
                document.comparison_reason = "Runtime prerequisites or the inherited operation clock refused before constructor execution; the typed failure retains the actual cause".into();
            } else if !predicate.supported_family() {
                document.comparison_reason = "Numerical extraction does not encode the original cancellation/resource/readiness schedule".into();
            } else if document
                .failure
                .iter()
                .chain(&document.findings)
                .any(|diagnostic| predicate.matches(diagnostic))
            {
                let original_matches = document.original.original_diagnostic.as_ref().is_some_and(|original| {
                    let strict_original = ReproductionPredicate::from_diagnostic(original);
                    document.failure.iter().chain(&document.findings).any(|actual| strict_original.matches(actual)
                        && matches!((serde_json::to_vec(&original.locations),serde_json::to_vec(&actual.locations)),(Ok(left),Ok(right)) if left == right))
                });
                document.status = if original_matches {
                    ReproductionStatus::OriginalFailureReproduced
                } else {
                    ReproductionStatus::RelatedDiagnosticReproduced
                };
                document.comparison_reason = "Exact typed code/stage/rule/source/validity and necessary observed conditions matched; original outcome remains independent of this new run".into();
            } else if !execution_complete {
                document.comparison_reason = "The new execution did not complete all requested work (refused, capped or budget-limited); retained diagnostics do not match the selected typed predicate, but the incomplete population cannot establish non-reproduction".into();
            } else {
                document.status = ReproductionStatus::NotReproduced;
                document.comparison_reason = "The new execution returned no diagnostic matching the complete selected typed predicate; another failure family is insufficient".into();
            }
        } else {
            document.comparison_reason = "No selected typed original diagnostic or explicit predicate; descriptive failure text cannot establish reproduction".into();
        }
        observer.finish(if operation.driver().token().is_cancelled() {
            Outcome::Cancelled
        } else if document.failure.is_some() {
            Outcome::Failed
        } else {
            Outcome::Completed
        });
        Ok(document)
    }
    async fn reproducer_physical(
        &self,
        source: &ReplaySourceDocument,
        cancel: &CancelSource,
    ) -> Result<super::PhysicalContext, WorkflowError> {
        use datafusion::arrow::ipc::reader::StreamReader;
        let bytes = source
            .physical_sources
            .iter()
            .try_fold(0usize, |bytes, source| bytes.checked_add(source.ipc.len()))
            .filter(|bytes| *bytes <= MAXIMUM_BYTES)
            .ok_or_else(|| contract("embedded physical IPC exceeds finite source allowance"))?;
        let _owner = self.shared.math().reserve(
            "reproducer:physical-decode",
            bytes.saturating_mul(16).saturating_add(65536),
        )?;
        let token = cancel.token();
        let validation = self.admission_context();
        let mut batches = BTreeMap::new();
        for physical in &source.physical_sources {
            token.checkpoint().map_err(pse_engine::EngineError::from)?;
            let spec = self
                .registry()
                .relation_by_id(physical.relation)
                .ok_or_else(|| {
                    contract("recorded physical relation is unavailable in this registry")
                })?;
            let mut reader = StreamReader::try_new(std::io::Cursor::new(&physical.ipc), None)
                .map_err(|error| contract(format!("recorded physical IPC: {error}")))?;
            let batch = reader
                .next()
                .ok_or_else(|| contract("recorded physical IPC has no original batch"))?
                .map_err(|error| contract(format!("recorded physical IPC: {error}")))?;
            if reader.next().is_some() {
                return Err(contract(
                    "physical replay codec requires its one complete original batch",
                ));
            }
            let checked = pse_relations::columnar::FieldCheckedBatch::admit(
                self.registry(),
                spec,
                batch,
                &validation,
                &token,
            )
            .map_err(super::relation)?;
            if batches.insert(spec.key, checked).is_some() {
                return Err(contract("duplicate recorded physical source"));
            }
        }
        let factory = self
            .sessions()
            .as_ref()
            .clone()
            .with_target_partitions(std::num::NonZeroUsize::MIN);
        let model = std::sync::Arc::new(factory).bind_model(self.registry().clone())?;
        let session = model.candidate_checked(batches.clone(), &token)?;
        let inventory =
            pse_admission::physical::PhysicalInventory::load(&session, self.registry(), &token)
                .await
                .map_err(|error| contract(format!("physical replay admission: {error}")))?;
        let mut physical = super::PhysicalContext::admitted(std::sync::Arc::new(inventory));
        physical.sources = batches;
        Ok(physical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn deployment() -> super::super::DiagnosticRuntimeSettings {
        super::super::DiagnosticRuntimeSettings {
            memory_limit_bytes: 256 << 20,
            threads: 2,
            spill_dir: "temporary-runtime-selected-at-run".into(),
            max_spill_bytes: 1 << 30,
            batch_size: 1024,
        }
    }
    fn text_command() -> DiagnosticCommandRequest {
        DiagnosticCommandRequest {
            version: Version,
            runtime: deployment(),
            operation: DiagnosticExecutionRequest {
                version: Version,
                physical_documents: BTreeMap::from([(
                    "package.toml".into(),
                    "[package]\nnot valid manifest".into(),
                )]),
                model_documents: vec![BTreeMap::from([(
                    "model.pse".into(),
                    "package p {def Root {var x:Scalar;eq e:x==1;}}".into(),
                )])],
                case_id: pse_ids::SemanticId::from_bytes([77; 16]),
                settings: Default::default(),
                diagnostics: super::super::ModelingDiagnosticPolicy {
                    profile: "reproducer-fixture".into(),
                    maximum_findings: 8,
                    near_bound_absolute: 0.,
                    near_bound_relative: 0.,
                    bound_violation: 0.,
                    residual: 1e-6,
                    variable_small: 1e-8,
                    variable_large: 1e8,
                    variable_zero: 0.,
                    jacobian_small: 1e-8,
                    jacobian_large: 1e8,
                    matrix: pse_math::diagnostics::MatrixPolicy {
                        dense_entries: 16,
                        findings: 8,
                        parallel_tolerance: 1e-8,
                        rank_absolute: 1e-12,
                        rank_relative: 1e-8,
                        singular_vector: 0.1,
                    },
                    terms: pse_math::diagnostics::TermPolicy {
                        zero: 1e-10,
                        mismatch: 1e6,
                        cancellation: 1e-4,
                        maximum_terms: 4,
                        combinations: 16,
                        findings: 8,
                    },
                },
                origin: None,
                allowance: Duration::from_secs(10),
                maximum_source_bytes: 1 << 20,
            },
        }
    }
    #[tokio::test]
    async fn extraction_is_offline_and_actual_text_retains_typed_prehandle_failure_and_strict_predicate()
     {
        let runtime = crate::workflow::tests::runtime();
        let first = runtime
            .diagnose_documents(text_command().operation, &CancelSource::new())
            .await
            .unwrap();
        assert!(first.run_id.is_none());
        let original = first.failure.unwrap();
        let (extracted, counts) = crate::math::counted(async {
            DiagnosticReproducer::extract_text(text_command(), Some(original.clone()), None)
        })
        .await;
        assert_eq!(
            counts,
            Default::default(),
            "offline extraction must not reconstruct or evaluate"
        );
        let extracted = extracted.unwrap();
        let decoded: ReproducerCommandRequest =
            serde_json::from_slice(&serde_json::to_vec(&extracted).unwrap()).unwrap();
        let result = runtime
            .run_reproducer(decoded.reproducer, &CancelSource::new())
            .await
            .unwrap();
        assert_ne!(result.operation, first.operation);
        assert!(result.run_id.is_none());
        assert!(result.failure.is_some());
        assert_eq!(result.status, ReproductionStatus::OriginalFailureReproduced);
        let mut wrong = extracted.reproducer.clone();
        wrong.predicate.as_mut().unwrap().code = DiagnosticCode::RuntimeCancelled;
        let unsupported = runtime
            .run_reproducer(wrong, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            unsupported.status,
            ReproductionStatus::ReproductionUnavailable
        );
        let mut different = extracted.reproducer.clone();
        different.predicate.as_mut().unwrap().code = DiagnosticCode::CompilerMissing;
        let result = runtime
            .run_reproducer(different, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(result.status, ReproductionStatus::ReproductionUnavailable);
        assert!(result.comparison_reason.contains("incomplete population"));
        let mut text_only = extracted.reproducer.clone();
        text_only.original_diagnostic = None;
        text_only.predicate = None;
        text_only.original_failure_text = Some(original.to_string());
        let result = runtime
            .run_reproducer(text_only, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(result.status, ReproductionStatus::ReproductionUnavailable);
        let mut narrowed = extracted.reproducer.clone();
        narrowed
            .original_diagnostic
            .as_mut()
            .unwrap()
            .observations
            .insert(
                "original_necessary_condition".into(),
                Observation::Boolean(true),
            );
        let result = runtime
            .run_reproducer(narrowed, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            result.status,
            ReproductionStatus::RelatedDiagnosticReproduced,
            "a narrower explicit match cannot silently establish the full typed original"
        );
        let mut explicit = extracted.reproducer;
        explicit.original_diagnostic = None;
        let result = runtime
            .run_reproducer(explicit, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            result.status,
            ReproductionStatus::RelatedDiagnosticReproduced
        );
    }
    #[test]
    fn extraction_refuses_paths_missing_references_incomplete_ipc_and_cannot_invent_typed_errors() {
        let mut command = text_command();
        command.operation.physical_documents =
            BTreeMap::from([("/original/private/source.toml".into(), "bytes".into())]);
        assert!(DiagnosticReproducer::extract_text(command, None, None).is_err());
        let mut command = text_command();
        command.operation.model_documents.clear();
        assert!(DiagnosticReproducer::extract_text(command, None, None).is_err());
        let mut context = ReviewContextDocument::pre_handle_failure(
            Default::default(),
            None,
            "runtime.cancelled: descriptive string".into(),
            vec![],
        )
        .unwrap();
        assert!(
            DiagnosticReproducer::extract_review_case(
                &context,
                0,
                Duration::from_secs(2),
                None,
                None
            )
            .is_err()
        );
        context.failure = None;
        assert!(
            DiagnosticReproducer::extract_review_case(&context, 0, Duration::ZERO, None, None)
                .is_err()
        );
    }
    fn numerical_command(body: &str) -> DiagnosticCommandRequest {
        let mut command = text_command();
        command.operation.physical_documents = BTreeMap::from([
            ("package.toml".into(),include_str!("../../../../tests/fixtures/packages/physical-primitives/package.toml").into()),
            ("materials/physical.yaml".into(),include_str!("../../../../tests/fixtures/packages/physical-primitives/materials/physical.yaml").into()),
            ("materials/time.yaml".into(),include_str!("../../../../tests/fixtures/packages/physical-primitives/materials/time.yaml").into()),
        ]);
        let manifest = include_str!("../../../../tests/fixtures/packages/minimal_explicit/package.toml").replace("id_policy = \"explicit\"","id_policy = \"named\"").replace("dependencies = []","dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
        let case = pse_ids::SemanticId::from_bytes([77; 16]);
        command.operation.case_id = case;
        command.operation.model_documents = vec![BTreeMap::from([
            ("package.toml".into(), manifest),
            (
                "models/original.pse".into(),
                format!(
                    "package reproduction_text {{def Root {{{body}}} @id(\"{case}\") case run {{child root:Root=Root();}}}}"
                ),
            ),
        ])];
        command
    }
    #[tokio::test]
    async fn actual_text_numerical_domain_failure_reproduces_selected_typed_source_diagnostic() {
        let runtime = crate::workflow::tests::runtime();
        let mut command = numerical_command("var x:Scalar;eq e:log(x)==0;annotation start x(-1);");
        command.operation.settings.intent = pse_backend_native::solve::SolveIntent::Root;
        let first = runtime
            .diagnose_documents(command.operation.clone(), &CancelSource::new())
            .await
            .unwrap();
        let original = first.failure.iter().chain(&first.findings)
            .find(|diagnostic| diagnostic.failure_class() == pse_diagnostics::FailureClass::SolveEvaluationError)
            .unwrap_or_else(|| panic!("actual negative log point must retain an evaluation failure, not a preparation refusal: {first:?}")).clone();
        assert!(
            matches!(
                original.code,
                DiagnosticCode::CandidateEvaluationFailed
                    | DiagnosticCode::MathDomain
                    | DiagnosticCode::MathEvaluation
            ),
            "negative log must fail through its numerical evaluation owner: {original:?}"
        );
        let extracted = DiagnosticReproducer::extract_text(command, Some(original), None).unwrap();
        let replayed = runtime
            .run_reproducer(extracted.reproducer, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            replayed.status,
            ReproductionStatus::OriginalFailureReproduced,
            "actual original numerical failure and source attribution must match"
        );
    }
    #[tokio::test]
    async fn unmatched_capped_diagnostics_are_unavailable_but_complete_absence_and_positive_matches_stand()
     {
        let runtime = crate::workflow::tests::runtime();
        let mut command = numerical_command("var x:Scalar;eq e:x*x==1;annotation start x(2);");
        command.operation.settings.intent = pse_backend_native::solve::SolveIntent::Root;
        command.operation.diagnostics.variable_large = 1.;
        let complete = runtime
            .diagnose_documents(command.operation.clone(), &CancelSource::new())
            .await
            .unwrap();
        assert!(
            complete.failure.is_none(),
            "must reach actual diagnostics: {:?}",
            complete.failure
        );
        assert!(
            complete.complete,
            "uncapped control must evaluate the full requested population"
        );
        assert!(
            complete.findings.len() > 1,
            "the actual finding limit must omit findings"
        );
        let mut missing = ReproductionPredicate::from_diagnostic(&complete.findings[0]);
        missing.code = DiagnosticCode::CompilerMissing;
        let extracted = DiagnosticReproducer::extract_text(command, None, Some(missing)).unwrap();
        let absent = runtime
            .run_reproducer(extracted.reproducer.clone(), &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(absent.status, ReproductionStatus::NotReproduced);
        let mut capped = extracted.reproducer;
        assert!(matches!(capped.input, ReproducerInput::OriginalText { .. }));
        if let ReproducerInput::OriginalText { request } = &mut capped.input {
            request.diagnostics.maximum_findings = 1;
        }
        let incomplete = runtime
            .run_reproducer(capped.clone(), &CancelSource::new())
            .await
            .unwrap();
        assert!(incomplete.failure.is_none());
        assert!(incomplete.run_id.is_some());
        assert_eq!(incomplete.findings.len(), 1);
        assert_eq!(
            incomplete.status,
            ReproductionStatus::ReproductionUnavailable
        );
        assert!(
            incomplete
                .comparison_reason
                .contains("incomplete population")
        );
        let retained = incomplete.findings[0].clone();
        capped.predicate = Some(ReproductionPredicate::from_diagnostic(&retained));
        capped.original_diagnostic = Some(retained);
        let matched = runtime
            .run_reproducer(capped, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            matched.status,
            ReproductionStatus::OriginalFailureReproduced
        );
    }
    async fn frozen_fixture(runtime: &Runtime) -> ReviewContextDocument {
        let documents = BTreeMap::from([
            ("package.toml".into(),include_bytes!("../../../../tests/fixtures/packages/physical-primitives/package.toml").to_vec()),
            ("materials/physical.yaml".into(),include_bytes!("../../../../tests/fixtures/packages/physical-primitives/materials/physical.yaml").to_vec()),
            ("materials/time.yaml".into(),include_bytes!("../../../../tests/fixtures/packages/physical-primitives/materials/time.yaml").to_vec()),
        ]);
        let pool = runtime.shared.pool();
        let cancel = CancelSource::new();
        let bundle = pse_admission::authoring_driver::document::load_package_documents_owned(
            &documents,
            runtime.registry(),
            Default::default(),
            &pool,
            &cancel.token(),
            &runtime.admission_context(),
        )
        .unwrap();
        let source = pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
            vec![bundle],
            &pool,
            &cancel.token(),
        )
        .unwrap();
        let physical = runtime
            .physical_from_documents(&source, &cancel.token())
            .await
            .unwrap();
        let rows = pse_authoring::language::parse(
            "package reproduction {def Root {param a:Scalar=1;var x:Scalar;eq e:x==a;}}",
            pse_ids::SemanticId::from_bytes([76; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let replay = package.replay_source().unwrap();
        let analysis = super::super::ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Default::default(),
            limits: Default::default(),
            case: pse_compiler::workspace::ModelingCaseBindings {
                values: BTreeMap::from([("x".into(), 1.)]),
                variables: BTreeMap::from([(
                    "x".into(),
                    pse_compiler::workspace::ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
            order: pse_kernels::DerivativeOrder::First,
            compiler: crate::workflow::tests::compiler_profile(),
            solver: crate::workflow::tests::profile(),
            numerical: Default::default(),
        };
        let constructor = ReplayCaseConstructor::from_analysis(0, &analysis).unwrap();
        ReviewContextDocument {
            version: Version,
            run_id: None,
            attempt_id: None,
            operation: None,
            selection: Default::default(),
            sources: vec![super::super::ReviewSourceDocument {
                identity: replay.identity,
                interpretation: replay.interpretation,
                build: replay.build,
                physical_identity: replay.physical_identity,
                physical_sources: replay.physical_sources,
                providers: vec![],
                objects: replay.objects,
            }],
            source_positions: vec![0],
            solver_settings: vec![],
            facts: vec![super::super::ReviewContextFact {
                path: "/steps/0/constructor".into(),
                codec: "replay-case-constructor.v2".into(),
                payload: serde_json::to_vec(&constructor).unwrap(),
            }],
            coverage: None,
            failure: Some("only original descriptive failure".into()),
            unavailable: vec![],
        }
    }
    #[tokio::test]
    async fn complete_frozen_ipc_objects_reconstruct_and_changed_build_refuses_under_fresh_operation()
     {
        let runtime = crate::workflow::tests::runtime();
        let context = frozen_fixture(&runtime).await;
        let (extracted, counts) = crate::math::counted(async {
            DiagnosticReproducer::extract_review_case(
                &context,
                0,
                Duration::from_secs(30),
                None,
                None,
            )
        })
        .await;
        assert_eq!(counts, Default::default());
        let extracted = extracted.unwrap();
        let decoded: DiagnosticReproducer =
            serde_json::from_slice(&serde_json::to_vec(&extracted).unwrap()).unwrap();
        let result = runtime
            .run_reproducer(decoded, &CancelSource::new())
            .await
            .unwrap();
        assert!(
            result.run_id.is_some(),
            "complete source/constructor must reconstruct through real owners: {:?}",
            result.failure
        );
        assert_eq!(
            result.status,
            ReproductionStatus::ReproductionUnavailable,
            "original descriptive string supplies no typed predicate"
        );
        let expired = CancelSource::new()
            .with_deadline(Some(std::time::Instant::now() - Duration::from_millis(1)));
        let (expired_result, counts) =
            crate::math::counted(runtime.run_reproducer(extracted.clone(), &expired)).await;
        let expired_result = expired_result.unwrap();
        assert!(expired_result.run_id.is_none());
        assert!(expired_result.failure.is_some());
        assert_eq!(expired_result.effective_allowance, Duration::ZERO);
        assert_eq!(
            counts,
            Default::default(),
            "expired inherited clock must refuse before source/constructor preparation"
        );
        let mut changed = extracted;
        if let ReproducerInput::FrozenAlgebraic { source, .. } = &mut changed.input {
            source.build = pse_ids::ContentHash::from_bytes([33; 32]);
        }
        let result = runtime
            .run_reproducer(changed, &CancelSource::new())
            .await
            .unwrap();
        assert!(result.run_id.is_none());
        assert!(result.failure.is_some());
        assert_eq!(result.status, ReproductionStatus::ReproductionUnavailable);
        let owner = runtime
            .observations()
            .reader(result.operation)
            .unwrap()
            .snapshot(Duration::from_secs(60));
        assert!(
            owner.outcome.is_some(),
            "pre-handle refusal still settles its fresh operation"
        );
        let mut provider: ReviewContextDocument =
            serde_json::from_slice(&serde_json::to_vec(&context).unwrap()).unwrap();
        provider.sources[0]
            .providers
            .push(super::super::ReviewProviderDescriptor {
                name: "opaque".into(),
                implementation: pse_ids::SemanticId::NIL,
                revision: pse_ids::ContentHash::from_bytes([1; 32]),
                data: pse_ids::ContentHash::from_bytes([2; 32]),
                configuration: None,
                inputs: vec![],
                outputs: vec![],
                input_shapes: vec![],
                output_shapes: vec![],
                derivative_source: pse_model::generated::enums::ExternalDerivativeSource::Analytic,
                derivatives: 0,
                smoothness: 0,
            });
        assert!(
            DiagnosticReproducer::extract_review_case(
                &provider,
                0,
                Duration::from_secs(30),
                None,
                None
            )
            .is_err(),
            "a descriptor never locates an executable factory"
        );
        let mut missing: ReviewContextDocument =
            serde_json::from_slice(&serde_json::to_vec(&context).unwrap()).unwrap();
        missing.sources[0].physical_sources[0].ipc.pop();
        assert!(
            DiagnosticReproducer::extract_review_case(
                &missing,
                0,
                Duration::from_secs(30),
                None,
                None
            )
            .is_err()
        );
        let mut missing: ReviewContextDocument =
            serde_json::from_slice(&serde_json::to_vec(&context).unwrap()).unwrap();
        missing.sources[0].objects[0]
            .references
            .push(("absent".into(), "source".into()));
        assert!(
            DiagnosticReproducer::extract_review_case(
                &missing,
                0,
                Duration::from_secs(30),
                None,
                None
            )
            .is_err()
        );
    }
}
