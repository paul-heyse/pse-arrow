// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Retained descriptive facts. Reading this owner never admits numerical reconstruction.
use super::replay::{
    ReplayCoverage, ReplayDecisions, ReplayPhysicalSource, ReplayProviderPort, ReplayProviderShape,
    ReplaySolverSettings, ReplaySourceObject,
};
use super::{ModelingPackage, RunRequest, RunResult, WorkflowError, contract};
use pse_diagnostics::observation::OperationId;
use pse_model::{
    document::Version,
    generated::identities::{AttemptId, RunId},
};
use std::sync::Arc;
mod attribution;
mod identity;
pub use attribution::{ReviewSourceAttribution, ReviewSourceOccurrence, ReviewSourceTerm};

/// Exact explanatory selection, independent of executable replay eligibility.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewContextRequest {
    /// Empty means all retained original steps; otherwise exact zero-based steps.
    pub steps: Vec<usize>,
    /// Include actual authored and physical input bytes.
    pub include_sources: bool,
    /// Include retained native records; never run a native query to fill gaps.
    pub include_native_records: bool,
    /// Finite allocation for the complete selected document, up to 256 MiB.
    pub maximum_bytes: usize,
}
impl Default for ReviewContextRequest {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            include_sources: true,
            include_native_records: true,
            maximum_bytes: 16 << 20,
        }
    }
}
impl ReviewContextRequest {
    fn validate(&self) -> Result<(), WorkflowError> {
        if self.maximum_bytes == 0 || self.maximum_bytes > 256 << 20 || self.steps.len() > 4096 {
            return Err(contract(
                "review context requires finite bytes and at most 4096 selected steps",
            ));
        }
        let mut steps = self.steps.clone();
        steps.sort_unstable();
        steps.dedup();
        if steps.len() != self.steps.len() {
            return Err(contract(
                "review context contains duplicate step identities",
            ));
        }
        Ok(())
    }
    fn selects(&self, step: usize) -> bool {
        self.steps.is_empty() || self.steps.contains(&step)
    }
}
/// An existing producer's exact JSON document, with its codec and source coordinates.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewContextFact {
    /// JSON-pointer-shaped coordinate in this retained scientific selection.
    pub path: String,
    /// Named existing owner codec. No schema is inferred from these bytes.
    pub codec: String,
    /// Exact existing owner JSON encoding; integers and IEEE-bit inputs remain lossless.
    pub payload: Vec<u8>,
}
/// Missing explanatory material is independent of scientific product availability.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewContextUnavailable {
    /// Exact missing field or capability.
    pub path: String,
    /// Observed reason; never inferred executable behavior.
    pub reason: String,
    /// Retained opaque descriptor, when available.
    pub descriptor: Option<String>,
}
/// Recorded callable descriptor, independent of opaque factory configuration callbacks.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewProviderDescriptor {
    /// Authored registration name.
    pub name: String,
    /// Recorded implementation identity, not its executable code.
    pub implementation: pse_ids::SemanticId,
    /// Actual algorithm/physical revision.
    pub revision: pse_ids::ContentHash,
    /// Actual parameter-data identity.
    pub data: pse_ids::ContentHash,
    /// Configuration is unavailable unless recorded separately; export never queries a factory.
    pub configuration: Option<pse_ids::ContentHash>,
    /// Exact ordered scalar input ABI.
    pub inputs: Vec<ReplayProviderPort>,
    /// Exact ordered scalar output ABI.
    pub outputs: Vec<ReplayProviderPort>,
    /// Full logical input shape.
    pub input_shapes: Vec<ReplayProviderShape>,
    /// Full logical output shape.
    pub output_shapes: Vec<ReplayProviderShape>,
    /// Actual derivative provenance.
    pub derivative_source: pse_model::generated::enums::ExternalDerivativeSource,
    /// Recorded implemented derivative order.
    pub derivatives: u8,
    /// Recorded established smoothness order.
    pub smoothness: u8,
}
impl ReviewProviderDescriptor {
    pub(super) fn new(name: &str, registration: &pse_kernels::Registration) -> Self {
        let spec = registration.spec();
        let ports = |ports: &[pse_kernels::Port]| {
            ports
                .iter()
                .map(|port| ReplayProviderPort {
                    id: port.id,
                    quantity: port.quantity.as_id(),
                    unit: port.unit.as_id(),
                })
                .collect()
        };
        let shapes = |shapes: &[pse_kernels::ProviderShape]| {
            shapes
                .iter()
                .map(|shape| ReplayProviderShape {
                    id: shape.id,
                    axes: shape.axes.clone(),
                    coordinates: shape.coordinates.clone(),
                    cells: shape.cells.clone(),
                })
                .collect()
        };
        let order = |order| match order {
            pse_kernels::DerivativeOrder::Value => 0,
            pse_kernels::DerivativeOrder::First => 1,
            pse_kernels::DerivativeOrder::Second => 2,
        };
        Self {
            name: name.into(),
            implementation: spec.id,
            revision: spec.revision,
            data: spec.data,
            configuration: None,
            inputs: ports(&spec.inputs),
            outputs: ports(&spec.outputs),
            input_shapes: shapes(&spec.shapes.inputs),
            output_shapes: shapes(&spec.shapes.outputs),
            derivative_source: spec.derivative_source,
            derivatives: order(spec.derivatives),
            smoothness: order(spec.smoothness),
        }
    }
}
/// Original source facts, with opaque descriptors and no executable factory predicates.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewSourceDocument {
    /// Original research source attribution, reauthenticated before durable recording.
    pub research: Option<super::ResearchSource>,
    /// Exact historical engineering attribution, when selected before execution.
    pub context: Option<pse_operations::canonical_contexts::ContextSelection>,
    /// Exact original source identity.
    pub identity: pse_ids::ContentHash,
    /// Registry interpretation of these source facts.
    pub interpretation: String,
    /// Source image build premise; does not prove an executed native version.
    pub build: pse_ids::ContentHash,
    /// Original admitted physical meaning.
    pub physical_identity: pse_ids::ContentHash,
    /// Actual admitted physical source streams, including units/bases/reference definitions.
    pub physical_sources: Vec<ReplayPhysicalSource>,
    /// Recorded provider ABI descriptors; opaque implementations stay unavailable.
    pub providers: Vec<ReviewProviderDescriptor>,
    /// Original authored/data/reference objects.
    pub objects: Vec<ReplaySourceObject>,
}
/// Retained scientific coordinates before executable preparation identity predicates.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewScientificCoordinates {
    /// Original run identity.
    pub run_id: RunId,
    /// Original step ordinal.
    pub step: i64,
    /// Original model identity.
    pub model_id: Option<pse_model::generated::identities::DeclarationId>,
    /// Original fit identity.
    pub fit_id: Option<pse_model::generated::identities::FitId>,
    /// Original authored case.
    pub case_id: Option<pse_model::generated::identities::DeclarationId>,
    /// Original instantiated model.
    pub instance_id: Option<pse_model::generated::identities::InstanceId>,
    /// Exact retained source identity.
    pub source_identity: pse_ids::ContentHash,
    /// Exact retained physical identity.
    pub physical_identity: pse_ids::ContentHash,
    /// Independently retained preparation identity, absent when it would require opaque callbacks.
    pub preparation_identity: Option<pse_ids::ContentHash>,
    /// Independently retained profile identity, absent when not separately recorded.
    pub profile_identity: Option<pse_ids::ContentHash>,
    /// Existing scientific-coordinate configuration codec, copied from retained inputs.
    pub configuration: pse_model::Bytes,
    /// Original admitted binding codec when retained.
    pub binding: Option<pse_model::Bytes>,
    /// Actual retained state coordinates with units and affine transforms.
    pub state: Vec<pse_model::generated::structures::ScientificArtifactCoordinate>,
    /// Actual retained input coordinates.
    pub inputs: Vec<pse_model::generated::structures::ScientificArtifactCoordinate>,
    /// Actual retained output coordinates.
    pub outputs: Vec<pse_model::generated::structures::ScientificArtifactCoordinate>,
    /// Actual retained integral coordinates.
    pub integrals: Vec<pse_model::generated::structures::ScientificArtifactCoordinate>,
    /// Actual physical time coordinate when retained.
    pub time: Option<pse_model::generated::structures::ScientificArtifactCoordinate>,
    /// Actual source-owned mode names.
    pub mode_names: Vec<String>,
    /// Actual recorded mode identities.
    pub mode_identities: Vec<pse_ids::ContentHash>,
}
/// Data-only descriptive context. Deserialization grants no factory or execution capability.
#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewContextDocument {
    /// Current descriptive codec version.
    pub version: Version<2>,
    /// Original execution identity; absent for a pre-handle failure.
    pub run_id: Option<RunId>,
    /// Actual durable attempt, only when retained explicitly.
    pub attempt_id: Option<AttemptId>,
    /// Original operation, including operations which failed before a run existed.
    pub operation: Option<OperationId>,
    /// Exact selection and its finite allocation.
    pub selection: ReviewContextRequest,
    /// Selected original source closures. Build hashes describe source image premises only.
    pub sources: Vec<ReviewSourceDocument>,
    /// Exact original source slots corresponding to sources; selected subsets never renumber constructor references.
    pub source_positions: Vec<usize>,
    /// Actual materialized settings, in selected original step order.
    pub solver_settings: Vec<ReplaySolverSettings>,
    /// Existing owner codecs for constructors, contexts, decisions and native coordinate witnesses.
    pub facts: Vec<ReviewContextFact>,
    /// Original scientific coverage and permission; no permission is inferred from context availability.
    pub coverage: Option<ReplayCoverage>,
    /// Original earlier failure, when present.
    pub failure: Option<String>,
    /// Explicit omissions and unavailable capabilities.
    pub unavailable: Vec<ReviewContextUnavailable>,
}
impl ReviewContextDocument {
    /// Copy typed facts only under the handoff's existing admitted working reservation.
    /// The caller keeps that reservation alive until the copied document is discarded.
    pub(crate) fn copy_for_handoff(
        &self,
        owner: &pse_columnar::MemoryReservation,
    ) -> Result<Self, WorkflowError> {
        self.validate()?;
        let limit = owner.size() / 8;
        super::replay::encoded_extent(
            self,
            limit.min(crate::tooling::frozen_inspection::MAXIMUM_DOCUMENT_BYTES),
        )?;
        Ok(Self {
            version: self.version,
            run_id: self.run_id,
            attempt_id: self.attempt_id,
            operation: self.operation,
            selection: self.selection.clone(),
            sources: self.sources.clone(),
            source_positions: self.source_positions.clone(),
            solver_settings: self.solver_settings.clone(),
            facts: self.facts.clone(),
            coverage: self.coverage.clone(),
            failure: self.failure.clone(),
            unavailable: self.unavailable.clone(),
        })
    }
    /// Describe an earlier source/admission failure from explicitly supplied copied facts.
    /// This constructor needs neither a runtime, an admitted package nor a run handle.
    pub fn pre_handle_failure(
        selection: ReviewContextRequest,
        operation: Option<OperationId>,
        failure: String,
        facts: Vec<ReviewContextFact>,
    ) -> Result<Self, WorkflowError> {
        let mut document = builder(&selection, None, operation).document;
        document.failure = Some(failure);
        document.facts = facts;
        document.unavailable.push(ReviewContextUnavailable {
            path: "/run_id".into(),
            reason: "failure preceded establishment of run and attempt identities".into(),
            descriptor: None,
        });
        document.unavailable.push(ReviewContextUnavailable { path: "/sources".into(), reason: "no admitted source closure exists; supplied original source/request facts retain only their recorded meaning".into(), descriptor: None });
        document.validate()?;
        Ok(document)
    }
    /// Preserve optional actual physical-session receipt facts after admission failed,
    /// without requiring a package, runtime lookup or scientific success.
    pub fn physical_admission_failure(
        selection: ReviewContextRequest,
        operation: Option<OperationId>,
        failure: String,
        receipt: &super::PhysicalAdmissionReceipt,
    ) -> Result<Self, WorkflowError> {
        let facts = receipt
            .observations()
            .map(|observations| {
                serde_json::to_vec(observations)
                    .map(|payload| ReviewContextFact {
                        path: "/physical_admission/engine".into(),
                        codec: "engine-execution-observations.v2".into(),
                        payload,
                    })
                    .map_err(|error| contract(error.to_string()))
            })
            .transpose()?
            .into_iter()
            .collect();
        let mut document = Self::pre_handle_failure(selection, operation, failure, facts)?;
        if let Some(reason) = receipt.unavailable() {
            document.unavailable.push(ReviewContextUnavailable {
                path: "/physical_admission/engine".into(),
                reason: reason.into(),
                descriptor: None,
            });
        }
        document.validate()?;
        Ok(document)
    }
    /// Preflight a supplied descriptive selection against an independently admitted handoff.
    pub(crate) fn admit_handoff(
        &self,
        request: &ReviewContextRequest,
    ) -> Result<(), WorkflowError> {
        if (!request.include_sources
            && (self.selection.include_sources || !self.sources.is_empty()))
            || (!request.include_native_records
                && (self.selection.include_native_records
                    || self.facts.iter().any(|fact| {
                        matches!(
                            fact.codec.as_str(),
                            "native-attempt-record.v1" | "dynamic-native-statistics.v1"
                        )
                    })))
            || (!request.steps.is_empty()
                && (self.selection.steps.is_empty()
                    || self
                        .selection
                        .steps
                        .iter()
                        .any(|step| !request.steps.contains(step))))
        {
            return Err(contract(
                "supplied review context exceeds admitted handoff selection",
            ));
        }
        super::replay::encoded_extent(self, request.maximum_bytes)?;
        Ok(())
    }
    /// Validate supplied factual membership only; never reconstruct sources or executable bodies.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.selection.validate()?;
        if self.source_positions.len() != self.sources.len()
            || self
                .source_positions
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.source_positions.len()
        {
            return Err(contract(
                "review source positions are missing or conflicting",
            ));
        }
        let mut paths = std::collections::BTreeSet::new();
        for fact in &self.facts {
            if !fact.path.starts_with('/')
                || !paths.insert(fact.path.as_str())
                || !matches!(
                    fact.codec.as_str(),
                    "replay-case-constructor.v2"
                        | "replay-case-inputs.v2"
                        | "replay-simulation-constructor.v2"
                        | "replay-fit-constructor.v3"
                        | "replay-horizon-constructor.v2"
                        | "replay-shooting-constructor.v2"
                        | "native-execution-snapshot.v1"
                        | "scientific-context.v1"
                        | "scientific-coordinates.v1"
                        | "replay-decisions.v2"
                        | "native-attempt-record.v1"
                        | "horizon-provenance.v1"
                        | "invocation-evidence.v2"
                        | "modeling-start-sources.v1"
                        | "physical-input-bits.v1"
                        | "admitted-binding.v1"
                        | "authored-source.v1"
                        | "submitted-request.v1"
                        | "modeling-source-map.v1"
                        | "engine-execution-observations.v2"
                        | "initialization-request.v1"
                        | "initialization-outcome.v1"
                        | "initialization-attempt.v1"
                        | "declared-recycle-request.v1"
                        | "declared-recycle-outcome.v1"
                        | "declared-recycle-settings.v1"
                        | "dynamic-native-statistics.v1"
                        | "modeling-direct-preparation.v1"
                        | "modeling-preparation-settings.v1"
                        | "modeling-direct-result.v1"
                        | "modeling-direct-initialization.v1"
                        | "modeling-direct-attempt.v1"
                        | "modeling-direct-settings.v1"
                        | "modeling-direct-numerics.v1"
                )
            {
                return Err(contract(
                    "review context has conflicting coordinates or unsupported factual codec",
                ));
            }
            // Existing JSON owner codecs are factual attachments, never executable admission.
            let _: serde::de::IgnoredAny = serde_json::from_slice(&fact.payload)
                .map_err(|error| contract(format!("review fact JSON: {error}")))?;
        }
        for source in &self.sources {
            let mut identities = std::collections::BTreeSet::new();
            for object in &source.objects {
                if !identities.insert(object.logical.as_str()) {
                    return Err(contract(
                        "review source contains conflicting logical identity",
                    ));
                }
            }
        }
        identity::validate(self)?;
        super::replay::encoded_extent(self, self.selection.maximum_bytes)?;
        Ok(())
    }
}
/// Admitted export owner; immutable ingress uses the separate data-only document.
#[derive(Debug)]
pub struct ReviewContext {
    document: ReviewContextDocument,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl serde::Serialize for ReviewContext {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.document, serializer)
    }
}
impl ReviewContext {
    /// Move joined facts into a descriptive handoff without encoding and decoding a copy.
    pub(crate) fn into_handoff_parts(
        self,
    ) -> (ReviewContextDocument, Arc<pse_columnar::AllocationLease>) {
        (self.document, self._owner)
    }
    /// Borrow the complete selected factual document.
    pub fn document(&self) -> &ReviewContextDocument {
        &self.document
    }
}
struct Builder {
    document: ReviewContextDocument,
    used: usize,
    limit: usize,
}
impl Builder {
    fn fact(
        &mut self,
        path: String,
        codec: &str,
        body: &impl serde::Serialize,
    ) -> Result<(), WorkflowError> {
        let bytes = super::replay::encoded_extent(body, self.limit.saturating_sub(self.used))?;
        self.used = self
            .used
            .checked_add(
                bytes
                    .saturating_add(path.len())
                    .saturating_add(codec.len())
                    .saturating_add(128),
            )
            .filter(|bytes| *bytes <= self.limit)
            .ok_or_else(|| contract("selected review context exceeds finite allocation"))?;
        let payload = serde_json::to_vec(body)
            .map_err(|error| contract(format!("review context codec: {error}")))?;
        self.document.facts.push(ReviewContextFact {
            path,
            codec: codec.into(),
            payload,
        });
        Ok(())
    }
    fn unavailable(
        &mut self,
        path: impl Into<String>,
        reason: impl Into<String>,
        descriptor: Option<String>,
    ) {
        self.document.unavailable.push(ReviewContextUnavailable {
            path: path.into(),
            reason: reason.into(),
            descriptor,
        });
    }
    fn settings(
        &mut self,
        profile: &crate::math::solves::SolverProfile,
    ) -> Result<(), WorkflowError> {
        let extent = super::replay::profile_extent(profile)?;
        if extent > self.limit.saturating_sub(self.used) {
            return Err(contract(
                "selected review settings exceed finite allocation",
            ));
        }
        let settings = ReplaySolverSettings::from_profile(profile);
        self.used = self.used.saturating_add(super::replay::encoded_extent(
            &settings,
            self.limit.saturating_sub(self.used),
        )?);
        self.document.solver_settings.push(settings);
        Ok(())
    }
    fn result_settings(
        &mut self,
        prefix: &str,
        profile: &crate::math::solves::SolverProfile,
    ) -> Result<(), WorkflowError> {
        // profile_extent already bounds the retained projection and its clone.
        let extent = super::replay::profile_extent(profile)?;
        let remaining = self.limit.saturating_sub(self.used);
        if extent > remaining {
            return Err(contract(format!(
                "selected result settings require {extent} bounded bytes; {remaining} remain"
            )));
        }
        self.fact(
            format!("{prefix}/settings"),
            "modeling-direct-settings.v1",
            &ReplaySolverSettings::from_profile(profile),
        )
    }
    fn source(&mut self, source: &ModelingPackage, position: usize) -> Result<(), WorkflowError> {
        if let Some(receipt) = source.physical.engine_receipt() {
            if let Some(facts) = receipt.observations() {
                self.fact(
                    format!("/physical_admission/{position}/engine"),
                    "engine-execution-observations.v2",
                    facts,
                )?;
            } else if let Some(reason) = receipt.unavailable() {
                self.unavailable(
                    format!("/physical_admission/{position}/engine"),
                    reason,
                    None,
                );
            }
        }
        if !self.document.selection.include_sources {
            return Ok(());
        }
        let extent = source.review_source_extent(self.limit.saturating_sub(self.used))?;
        if self.used.saturating_add(extent) > self.limit {
            return Err(contract("selected review source exceeds finite allocation"));
        }
        // This operation copies retained facts only: no admission, providers or source lookup.
        let document = source.review_source()?;
        if document.physical_sources.is_empty() {
            self.unavailable(format!("/sources/{}/physical_sources", self.document.sources.len()), "no physical source streams retained; physical identity and coordinate descriptors do not supply unrecorded full definitions", None);
        }
        for provider in &document.providers {
            self.unavailable(format!("/sources/{}/providers/{}/configuration", self.document.sources.len(), provider.name), "opaque factory configuration was not recorded; no configuration callback was queried", None);
            self.unavailable(format!("/sources/{}/providers/{}/implementation", self.document.sources.len(), provider.name),
                "retained descriptor describes an opaque executable registration; its factory implementation is not recorded", Some(format!("{:?}", provider.implementation)));
        }
        self.document.sources.push(document);
        self.document.source_positions.push(position);
        self.used = self.used.saturating_add(extent);
        Ok(())
    }
}
fn builder(
    request: &ReviewContextRequest,
    run_id: Option<RunId>,
    operation: Option<OperationId>,
) -> Builder {
    Builder {
        used: 4096,
        limit: request.maximum_bytes,
        document: ReviewContextDocument {
            version: Version,
            run_id,
            attempt_id: None,
            operation,
            selection: request.clone(),
            sources: Vec::new(),
            source_positions: Vec::new(),
            solver_settings: Vec::new(),
            facts: Vec::new(),
            coverage: None,
            failure: None,
            unavailable: Vec::new(),
        },
    }
}
impl RunResult {
    /// Export available original facts without executable Replay's all-or-refuse predicates.
    pub fn review_context(
        self: &Arc<Self>,
        request: &ReviewContextRequest,
    ) -> Result<ReviewContext, WorkflowError> {
        request.validate()?;
        let owner = self
            .services
            .preparation()
            .math()
            .reserve("review:descriptive-context", request.maximum_bytes)?;
        let mut out = builder(
            request,
            Some(self.run_id),
            self.invocation
                .as_ref()
                .and_then(|invocation| invocation.operation()),
        );
        // Shared factual projection precedes every executable Replay predicate.
        out.document.coverage = Some(self.recorded_coverage());
        out.document.attempt_id = match &self.durability {
            super::RunDurability::Ephemeral => None,
            super::RunDurability::Durable(record) => Some(record.attempt_id),
        };
        out.fact(
            "/invocation".into(),
            "invocation-evidence.v2",
            &self.invocation_evidence()?,
        )?;
        out.document.failure = self.error().map(ToString::to_string);
        match self.replay_request() {
            RunRequest::Expectation(expectation) => {
                select_single(request)?;
                out.source(&expectation.source,0)?;
                out.fact("/expectation/constructor".into(),"explicit-measure-expectation.v1",&super::replay::expectation_request_facts(expectation)?.constructor)?;
                if let Ok(super::RunReport::Expectation(product)) = self.report() { out.fact("/expectation/report".into(),"expectation-report.v1",&product.report)?; }
            }
            RunRequest::Mathematical(mathematical) => {
                select_single(request)?;
                out.source(&mathematical.source, 0)?;
                out.fact("/mathematical/constructor".into(), "selected-mathematical.v1", &super::replay::mathematical_request_facts(mathematical)?.constructor)?;
                if let Ok(super::RunReport::Mathematical(product)) = self.report() {
                    out.fact("/mathematical/product".into(), "mathematical-product.v1", product)?;
                }
            }
            RunRequest::SelectedObservation(observation) => {
                select_single(request)?;
                out.source(&observation.source, 0)?;
                out.fact("/observation/constructor".into(), "selected-observation.v1", &super::replay::selected_observation_request_facts(observation)?.constructor)?;
                if let Ok(super::RunReport::SelectedObservation(values)) = self.report() {
                    out.fact("/observation/values".into(), "selected-observation-values.v1", values)?;
                }
            }
            RunRequest::Initialization(initialization) => {
                if request.steps.iter().any(|step| *step != 0) {
                    return Err(contract(
                        "initialization review context selects the original task scope",
                    ));
                }
                out.source(&initialization.source, 0)?;
                out.settings(&initialization.execution.analysis.solver)?;
                out.fact(
                    "/initialization/policy".into(),
                    "initialization-policy.v1",
                    initialization.policy()?,
                )?;
                out.fact(
                    "/initialization/constructor".into(),
                    "replay-case-constructor.v2",
                    &super::ReplayCaseConstructor::from_analysis(
                        0,
                        &initialization.execution.analysis,
                    )?,
                )?;
            }
            RunRequest::Modeling(steps) => {
                if request.steps.iter().any(|step| *step >= steps.len()) {
                    return Err(contract("review context selected step absent"));
                }
                for (index, step) in steps
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| request.selects(*index))
                {
                    out.source(&step.source, index)?;
                    out.settings(&step.profile)?;
                    if step.original_inputs.descriptive_extent()
                        > out.limit.saturating_sub(out.used)
                    {
                        return Err(contract("review constructor exceeds finite allocation"));
                    }
                    out.fact(
                        format!("/steps/{index}/submitted_case"),
                        "replay-case-inputs.v2",
                        &step.original_inputs.descriptive_case_inputs(),
                    )?;
                    match step.original_inputs.descriptive_document(index) {
                        Ok(mut constructor) => {
                            constructor.requested_start = step
                                .solve
                                .requested_start()
                                .map(pse_backend_native::solve::WarmStart::snapshot);
                            constructor.requested_screened_start = step
                                .solve
                                .requested_screened_start()
                                .map(super::replay::ReplayScreenedStart::new);
                            out.fact(
                                format!("/steps/{index}/constructor"),
                                "replay-case-constructor.v2",
                                &constructor,
                            )?;
                        }
                        Err(error) => out.unavailable(
                            format!("/steps/{index}/constructor"),
                            error.to_string(),
                            None,
                        ),
                    }
                    for descriptor in step.original_inputs.opaque_descriptors() {
                        out.unavailable(
                            format!("/steps/{index}/implicit"),
                            "caller-supplied executable body is not a portable implementation",
                            Some(descriptor),
                        );
                    }
                    out.fact(
                        format!("/steps/{index}/starts"),
                        "modeling-start-sources.v1",
                        &step.starts,
                    )?;
                    if step.model.values.scalars.len().saturating_mul(256)
                        > out.limit.saturating_sub(out.used)
                    {
                        return Err(contract(
                            "selected review physical values exceed finite allocation",
                        ));
                    }
                    let values = step
                        .model
                        .values
                        .scalars
                        .iter()
                        .map(|(id, value)| (*id, value.to_bits()))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    out.fact(
                        format!("/steps/{index}/physical_values"),
                        "physical-input-bits.v1",
                        &values,
                    )?;
                    if let Some(binding) = &step.bound_source {
                        out.fact(
                            format!("/steps/{index}/binding"),
                            "admitted-binding.v1",
                            &***binding,
                        )?;
                    }
                    out.fact(
                        format!("/steps/{index}/runtime"),
                        "native-execution-snapshot.v1",
                        step.solve.execution_snapshot(),
                    )?;
                    out.fact(
                        format!("/steps/{index}/context"),
                        "scientific-coordinates.v1",
                        &super::artifacts::solve_coordinates(
                            step,
                            self.run_id,
                            i64::try_from(index).map_err(|_| contract("review step ordinal"))?,
                        )?,
                    )?;
                    out.unavailable(format!("/steps/{index}/context/preparation_identity"), "preparation identity was not retained independently; descriptive export never queries provider configuration callbacks", None);
                    out.unavailable(format!("/steps/{index}/context/profile_identity"), "effective settings are retained directly; descriptive export does not generate a fresh profile identity", None);
                }
            }
            RunRequest::Simulation(simulation) => {
                select_single(request)?;
                out.source(&simulation.source, 0)?;
                if let Some(original) = &simulation.original_inputs {
                    out.fact(
                        "/simulation/constructor".into(),
                        "replay-simulation-constructor.v2",
                        &original.document(0),
                    )?;
                } else {
                    out.unavailable(
                        "/simulation/constructor",
                        "original simulation constructor absent",
                        None,
                    );
                }
                match simulation.review_coordinates(self.run_id) {
                    Ok(coordinates) => out.fact(
                        "/simulation/context".into(),
                        "scientific-coordinates.v1",
                        &coordinates,
                    )?,
                    Err(error) => out.unavailable("/simulation/context", error.to_string(), None),
                }
                out.fact(
                    "/simulation/runtime".into(),
                    "native-execution-snapshot.v1",
                    simulation.snapshot(),
                )?;
                if request.include_native_records {
                    if let Ok(super::RunReport::Simulation(trajectory)) = self.report() {
                        #[derive(serde::Serialize)]
                        struct Statistics<'a> {
                            run_id: RunId,
                            segments: &'a [serde_json::Value],
                        }
                        out.fact(
                            "/simulation/native_statistics".into(),
                            "dynamic-native-statistics.v1",
                            &Statistics {
                                run_id: self.run_id,
                                segments: &trajectory.report().statistics,
                            },
                        )?;
                    } else {
                        out.unavailable(
                            "/simulation/native_statistics",
                            "original joined native segment statistics unavailable",
                            None,
                        );
                    }
                }
            }
            RunRequest::Fit(fit) => {
                select_single(request)?;
                out.source(&fit.source, 0)?;
                out.settings(&fit.problem.profile.solver)?;
                out.fact(
                    "/fit/constructor".into(),
                    "replay-fit-constructor.v3",
                    &fit.original_inputs.document(0),
                )?;
                out.fact(
                    "/fit/context".into(),
                    "scientific-context.v1",
                    &fit.scientific_context(self.run_id)?,
                )?;
                out.fact(
                    "/fit/runtime".into(),
                    "native-execution-snapshot.v1",
                    fit.execution_snapshot(),
                )?;
            }
            RunRequest::Horizon(horizon) => {
                select_single(request)?;
                for (position, source) in horizon.sources.iter().enumerate() {
                    out.source(source, position)?;
                }
                out.fact(
                    "/horizon/provenance".into(),
                    "horizon-provenance.v1",
                    &horizon.provenance,
                )?;
                match horizon.original.plant.review_coordinates(self.run_id) {
                    Ok(coordinates) => out.fact(
                        "/horizon/plant/context".into(),
                        "scientific-coordinates.v1",
                        &coordinates,
                    )?,
                    Err(error) => {
                        out.unavailable("/horizon/plant/context", error.to_string(), None)
                    }
                }
                if let Some(estimator) = &horizon.original.estimator {
                    out.settings(&estimator.analysis.solver)?;
                    for descriptor in estimator.analysis.numerical.implicit.keys() {
                        out.unavailable(
                            "/horizon/estimator/implicit",
                            "caller-supplied executable body is not a portable implementation",
                            Some(format!("{descriptor:?}")),
                        );
                    }
                }
                if let Some(controller) = &horizon.original.controller {
                    out.settings(&controller.analysis.solver)?;
                    for descriptor in controller.analysis.numerical.implicit.keys() {
                        out.unavailable(
                            "/horizon/controller/implicit",
                            "caller-supplied executable body is not a portable implementation",
                            Some(format!("{descriptor:?}")),
                        );
                    }
                }
                match super::replay::ReplayHorizonConstructor::descriptive(&horizon.original) {
                    Ok(body) => out.fact(
                        "/horizon/constructor".into(),
                        "replay-horizon-constructor.v2",
                        &body,
                    )?,
                    Err(error) => out.unavailable("/horizon/constructor", error.to_string(), None),
                }
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => {
                select_single(request)?;
                out.source(&problem.simulation.source, 0)?;
                out.settings(&problem.solver)?;
                match super::replay::ReplayShootingConstructor::new(problem, initial) {
                    Ok(body) => out.fact(
                        "/shooting/constructor".into(),
                        "replay-shooting-constructor.v2",
                        &body,
                    )?,
                    Err(error) => out.unavailable("/shooting/constructor", error.to_string(), None),
                }
                match problem.simulation.review_coordinates(self.run_id) {
                    Ok(coordinates) => out.fact(
                        "/shooting/simulation/context".into(),
                        "scientific-coordinates.v1",
                        &coordinates,
                    )?,
                    Err(error) => {
                        out.unavailable("/shooting/simulation/context", error.to_string(), None)
                    }
                }
                out.fact(
                    "/shooting/runtime".into(),
                    "native-execution-snapshot.v1",
                    problem.execution_snapshot(),
                )?;
            }
        }
        for (step, trace) in self
            .replay_traces()
            .filter(|(step, _)| request.selects(*step))
        {
            out.fact(
                format!("/steps/{step}/decisions"),
                "replay-decisions.v2",
                &ReplayDecisions::new(step, trace),
            )?;
        }
        if request.include_native_records {
            for (step, mechanism, component, record) in self
                .native_attempt_records()
                .filter(|(step, _, _, _)| request.selects(*step))
            {
                out.fact(
                    format!("/steps/{step}/native/{mechanism}/{component:?}"),
                    "native-attempt-record.v1",
                    record.as_ref(),
                )?;
            }
        }
        out.unavailable("/source_map", "no static source map is automatically prepared; callers may attach an already retained modeling-source-map.v1 fact for its exact original step/component", None);
        out.unavailable("/backend/executed_version", "source image pins are not evidence of the executing native binary; only retained runtime/native records are supplied", None);
        out.document.validate()?;
        Ok(ReviewContext {
            document: out.document,
            _owner: owner,
        })
    }
}
fn select_single(request: &ReviewContextRequest) -> Result<(), WorkflowError> {
    if request.steps.iter().any(|step| *step != 0) {
        Err(contract("review context selected step absent"))
    } else {
        Ok(())
    }
}
/// The standalone strategy adapters copy only on an explicit request. Their sources
/// are immutable shared revision references; no executable provider is consulted.
#[cfg(feature = "solver-kinsol")]
fn strategy_builder(
    services: &super::PreparationServices,
    source: &ModelingPackage,
    request: &ReviewContextRequest,
    operation: Option<OperationId>,
    run_id: RunId,
) -> Result<(Builder, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    request.validate()?;
    select_single(request)?;
    let owner = services
        .math()
        .reserve("review:strategy-context", request.maximum_bytes)?;
    let mut out = builder(request, Some(run_id), operation);
    out.source(source, 0)?;
    out.unavailable("/invocation", "standalone strategy has no RunResult invocation; actual strategy run and operation are retained directly", None);
    out.unavailable(
        "/attempt_id",
        "standalone strategy was not durably recorded as a run attempt",
        None,
    );
    out.unavailable(
        "/source_map",
        "static source attribution was not requested by this standalone strategy",
        None,
    );
    Ok((out, owner))
}
fn strategy_records(
    out: &mut Builder,
    prefix: &str,
    trace: &crate::math::strategy::Trace,
) -> Result<(), WorkflowError> {
    for (mechanism, product) in trace.products.iter().enumerate() {
        if let Some(record) = &product.native_attempt {
            out.fact(
                format!("{prefix}/native/{mechanism}/None"),
                "native-attempt-record.v1",
                record.as_ref(),
            )?;
        }
        for (component, record) in product
            .native_components
            .iter()
            .flat_map(|records| records.iter())
            .enumerate()
        {
            out.fact(
                format!("{prefix}/native/{mechanism}/Some({component})"),
                "native-attempt-record.v1",
                record.as_ref(),
            )?;
        }
    }
    Ok(())
}

impl ModelingPackage {
    /// Copy the admitted declared preparation and any original direct outcome while
    /// their owners are alive. This descriptive selection does not create a run handle.
    pub fn declared_review_context(
        &self,
        services: &super::PreparationServices,
        declared: &super::DeclaredExecution,
        outcome: Option<&super::DeclaredAlgebraicOutcome>,
        request: &ReviewContextRequest,
        operation: Option<OperationId>,
    ) -> Result<ReviewContext, WorkflowError> {
        declared
            .retained_source()
            .check_service_bindings(services)?;
        if !self.shares_retained_source(declared.retained_source()) {
            return Err(contract(
                "declared review source differs from the admitted original owner",
            ));
        }
        request.validate()?;
        select_single(request)?;
        let owner = services
            .math()
            .reserve("review:declared-context", request.maximum_bytes)?;
        let run = outcome.map(|outcome| match outcome {
            super::DeclaredAlgebraicOutcome::Solve(result) => result.run_id,
            super::DeclaredAlgebraicOutcome::Initialization(report) => report.run_id,
        });
        let mut out = builder(request, run, operation);
        out.source(declared.retained_source(), 0)?;
        out.settings(&declared.analysis.solver)?;
        out.fact(
            "/declared/preparation".into(),
            "modeling-direct-preparation.v1",
            &serde_json::json!({
                "root": declared.analysis.root,
                "instance": declared.analysis.instance,
                "route": declared.route,
                "procedure": declared.procedure.kind(),
            }),
        )?;
        out.fact(
            "/declared/preparation/controls".into(),
            "modeling-preparation-settings.v1",
            &super::PreparationSettings {
                compiler: declared.analysis.compiler,
                limits: declared.analysis.limits,
            },
        )?;
        out.unavailable(
            "/attempt_id",
            "direct declared execution has no durable attempt identity",
            None,
        );
        out.unavailable(
            "/invocation",
            "direct declared execution has no RunResult invocation",
            None,
        );
        match outcome {
            None => out.unavailable(
                "/outcome",
                "preparation did not execute a numerical operation",
                None,
            ),
            Some(super::DeclaredAlgebraicOutcome::Solve(result)) => {
                direct_result(
                    &mut out,
                    declared.retained_source(),
                    "/declared/result",
                    result,
                    request.include_native_records,
                )?;
            }
            Some(super::DeclaredAlgebraicOutcome::Initialization(report)) => {
                #[derive(serde::Serialize)]
                struct Initialization<'a> {
                    run_id: RunId,
                    completed: bool,
                    failure: &'a Option<pse_model::diagnostic::BoundaryDiagnostic>,
                    committed: Option<DirectValues<'a>>,
                    discrete: pse_model::generated::enums::ModelingDiscreteInitialization,
                    discrete_assignment: DirectValues<'a>,
                }
                out.fact(
                    "/declared/initialization".into(),
                    "modeling-direct-initialization.v1",
                    &Initialization {
                        run_id: report.run_id,
                        completed: report.completed,
                        failure: &report.failure,
                        committed: report.committed.as_ref().map(DirectValues),
                        discrete: report.discrete,
                        discrete_assignment: DirectValues(&report.discrete_assignment),
                    },
                )?;
                for (ordinal, attempt) in report.attempts.iter().enumerate() {
                    let (stage, fraction) = match &attempt.step {
                        super::ModelingInitializationStep::Stage(name) => {
                            (Some(name.as_str()), None)
                        }
                        super::ModelingInitializationStep::Homotopy(value) => (None, Some(*value)),
                        super::ModelingInitializationStep::Original => (None, None),
                    };
                    let prefix = format!("/declared/attempts/{ordinal}");
                    out.fact(format!("{prefix}/attempt"), "modeling-direct-attempt.v1", &serde_json::json!({
                        "run_id": report.run_id, "ordinal": ordinal, "kind": attempt.step.kind(),
                        "stage": stage, "fraction_bits": fraction.map(f64::to_bits), "accepted": attempt.accepted(),
                        "interruption": attempt.interruption, "diagnostic": attempt.diagnostic(),
                        "error": attempt.result.as_ref().err().map(ToString::to_string),
                    }))?;
                    if let Ok(result) = &attempt.result {
                        direct_result(
                            &mut out,
                            declared.retained_source(),
                            &format!("{prefix}/result"),
                            result,
                            request.include_native_records,
                        )?;
                    }
                }
            }
        }
        out.document.validate()?;
        Ok(ReviewContext {
            document: out.document,
            _owner: owner,
        })
    }
}

fn direct_result(
    out: &mut Builder,
    source: &ModelingPackage,
    prefix: &str,
    result: &super::ModelingResult,
    include_native: bool,
) -> Result<(), WorkflowError> {
    if !source.shares_retained_source(&result.prepared.source) {
        return Err(contract(
            "direct result source differs from its declared preparation owner",
        ));
    }
    #[derive(serde::Serialize)]
    struct ResultFacts<'a> {
        run_id: RunId,
        attempt: usize,
        accepted: bool,
        diagnostic: Option<pse_model::diagnostic::BoundaryDiagnostic>,
        permission: &'a super::numerics::CandidateDecision,
        values: DirectValues<'a>,
        checks: DirectChecks<'a>,
        reports: DirectReports<'a>,
        quality_origin: &'static str,
        original_quality: Option<DirectQuality<'a>>,
        component_qualities: DirectComponentQualities<'a>,
    }
    let (quality_origin, original_quality, components) = match &result.outcome {
        crate::math::solves::Outcome::Native(report) => ("native", report.quality.as_ref(), None),
        crate::math::solves::Outcome::Constant(report) => (
            "constant_reconstruction",
            Some(&report.quality),
            Some(report.as_ref()),
        ),
        crate::math::solves::Outcome::Rejected(_) => ("no_native_report", None, None),
    };
    out.fact(
        prefix.into(),
        "modeling-direct-result.v1",
        &ResultFacts {
            run_id: result.run_id,
            attempt: result.original_attempt,
            accepted: result.accepted,
            diagnostic: result.diagnostic(),
            permission: &result.completion.decision,
            values: DirectValues(&result.values.scalars),
            checks: DirectChecks(&result.checks),
            reports: DirectReports(&result.reports),
            quality_origin,
            original_quality: original_quality.map(DirectQuality),
            component_qualities: DirectComponentQualities(components),
        },
    )?;
    out.result_settings(prefix, &result.prepared.profile)?;
    out.fact(
        format!("{prefix}/numerics"),
        "modeling-direct-numerics.v1",
        &DirectNumerics {
            policy: result.prepared.solve.numerics(),
            native_accuracy: result.prepared.solve.accuracy(),
            accuracy: &result.completion.accuracy,
        },
    )?;
    if result
        .completion
        .accuracy
        .iter()
        .any(|goal| goal.failure.is_some())
    {
        out.unavailable(format!("{prefix}/numerics/accuracy/failure"),
            "opaque typed cause is not serialized; its original description and producer-reported extent remain in the factual accuracy record", None);
    }
    if include_native && let Some(trace) = &result.strategy {
        strategy_records(out, prefix, trace)?;
    }
    Ok(())
}

// Registry rows are not serde codecs. Borrow their original fields while serializing,
// so Builder's extent pass precedes the bounded output allocation.
struct DirectChecks<'a>(&'a [super::ModelingCheck]);
struct DirectReports<'a>(&'a [super::ModelingReport]);
struct DirectValues<'a>(&'a std::collections::BTreeMap<pse_ids::SemanticId, f64>);
/// Original native/reconstructed quality remains distinct from modeling checks.
struct DirectQuality<'a>(&'a pse_backend_native::quality::Quality);
impl serde::Serialize for DirectQuality<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OriginalQuality", 4)?;
        state.serialize_field("rows", &DirectViolations(&self.0.rows))?;
        state.serialize_field("bounds", &DirectViolations(&self.0.bounds))?;
        state.serialize_field("integrality", &DirectViolations(&self.0.integrality))?;
        state.serialize_field("normalized_max_bits", &self.0.normalized_max.to_bits())?;
        state.end()
    }
}
struct DirectViolations<'a>(&'a [pse_backend_native::quality::Violation]);
impl serde::Serialize for DirectViolations<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeSeq, SerializeStruct};
        struct Violation<'a>(&'a pse_backend_native::quality::Violation);
        impl serde::Serialize for Violation<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut state = serializer.serialize_struct("OriginalViolation", 3)?;
                state.serialize_field("id", &self.0.id)?;
                state.serialize_field("physical_bits", &self.0.physical.to_bits())?;
                state.serialize_field("tolerance_bits", &self.0.tolerance.to_bits())?;
                state.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for violation in self.0 {
            seq.serialize_element(&Violation(violation))?;
        }
        seq.end()
    }
}
struct DirectComponentQualities<'a>(Option<&'a crate::math::solves::ConstantReport>);
impl serde::Serialize for DirectComponentQualities<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        #[derive(serde::Serialize)]
        struct Component<'a> {
            ordinal: usize,
            backend: pse_backend_native::solve::Backend,
            quality: Option<DirectQuality<'a>>,
        }
        let mut seq =
            serializer.serialize_seq(self.0.map(|report| report.component_reports().len()))?;
        if let Some(report) = self.0 {
            for (ordinal, component) in report.component_reports().enumerate() {
                seq.serialize_element(&Component {
                    ordinal,
                    backend: component.backend,
                    quality: component.quality.as_ref().map(DirectQuality),
                })?;
            }
        }
        seq.end()
    }
}
impl serde::Serialize for DirectValues<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for (id, value) in self.0 {
            seq.serialize_element(&(id, value.to_bits()))?;
        }
        seq.end()
    }
}
macro_rules! fields {
    ($state:ident, $row:ident, $($field:ident),+ $(,)?) => {
        $(serde::ser::SerializeStruct::serialize_field(&mut $state, stringify!($field), &$row.$field)?;)+
    };
}

/// Borrow the actual resolved inventory and original accuracy outcomes; the Builder
/// sizes the selected document before allocating its serialized factual attachment.
struct DirectNumerics<'a> {
    policy: &'a pse_model::numerics::ResolvedNumericalPolicy,
    native_accuracy: &'a pse_backend_native::solve::ResolvedAccuracy,
    accuracy: &'a [pse_math::engineering_accuracy::GoalResult],
}
impl serde::Serialize for DirectNumerics<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DirectNumerics", 5)?;
        state.serialize_field("key", &self.policy.key)?;
        state.serialize_field("policy", &self.policy.policy)?;
        state.serialize_field("targets", &DirectTargets(&self.policy.targets))?;
        state.serialize_field("native_accuracy", self.native_accuracy)?;
        state.serialize_field("accuracy", &DirectAccuracy(self.accuracy))?;
        state.end()
    }
}
struct DirectTargets<'a>(&'a [pse_model::numerics::ResolvedTarget]);
impl serde::Serialize for DirectTargets<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeSeq, SerializeStruct};
        struct Target<'a>(&'a pse_model::numerics::ResolvedTarget);
        impl serde::Serialize for Target<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let row = self.0;
                let mut state = serializer.serialize_struct("ResolvedTarget", 12)?;
                fields!(
                    state,
                    row,
                    id,
                    kind,
                    quantity,
                    unit,
                    nominal,
                    coordinate_scale,
                    required_scale,
                    absolute,
                    relative,
                    budget,
                    engineering
                );
                struct Provenance<'a>(&'a [pse_model::numerics::NumericalProvenance]);
                impl serde::Serialize for Provenance<'_> {
                    fn serialize<S: serde::Serializer>(
                        &self,
                        serializer: S,
                    ) -> Result<S::Ok, S::Error> {
                        struct Entry<'a>(&'a pse_model::numerics::NumericalProvenance);
                        impl serde::Serialize for Entry<'_> {
                            fn serialize<S: serde::Serializer>(
                                &self,
                                serializer: S,
                            ) -> Result<S::Ok, S::Error> {
                                let row = self.0;
                                let mut state =
                                    serializer.serialize_struct("NumericalProvenance", 6)?;
                                fields!(
                                    state,
                                    row,
                                    declaration,
                                    source,
                                    field,
                                    selected,
                                    value,
                                    description
                                );
                                state.end()
                            }
                        }
                        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                        for row in self.0 {
                            seq.serialize_element(&Entry(row))?;
                        }
                        seq.end()
                    }
                }
                state.serialize_field("provenance", &Provenance(&row.provenance))?;
                state.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for row in self.0 {
            seq.serialize_element(&Target(row))?;
        }
        seq.end()
    }
}
struct DirectAccuracy<'a>(&'a [pse_math::engineering_accuracy::GoalResult]);
impl serde::Serialize for DirectAccuracy<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeSeq, SerializeStruct};
        struct Goal<'a>(&'a pse_math::engineering_accuracy::GoalResult);
        impl serde::Serialize for Goal<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let row = self.0;
                let mut state = serializer.serialize_struct("GoalResult", 6)?;
                state.serialize_field("goal", &row.goal)?;
                let c = &row.classification;
                state.serialize_field(
                    "classification",
                    &(
                        c.status,
                        c.resolution,
                        c.criterion,
                        c.interval.map(|(a, b)| (a.to_bits(), b.to_bits())),
                        c.error.map(f64::to_bits),
                        c.unavailable,
                    ),
                )?;
                struct Evidence<'a>(&'a pse_model::engineering_accuracy::OutputEvidence);
                impl serde::Serialize for Evidence<'_> {
                    fn serialize<S: serde::Serializer>(
                        &self,
                        serializer: S,
                    ) -> Result<S::Ok, S::Error> {
                        let row = self.0;
                        let mut state = serializer.serialize_struct("OutputEvidence", 14)?;
                        fields!(
                            state,
                            row,
                            target,
                            target_kind,
                            quantity,
                            unit,
                            observation,
                            accuracy,
                            source,
                            validity,
                            interpretation,
                            method,
                            limitation
                        );
                        state.serialize_field("time_bits", &row.time.map(f64::to_bits))?;
                        state.serialize_field("value_bits", &row.value.map(f64::to_bits))?;
                        state.serialize_field(
                            "interval_bits",
                            &row.interval.map(|(a, b)| (a.to_bits(), b.to_bits())),
                        )?;
                        state.end()
                    }
                }
                state.serialize_field("evidence", &row.evidence.as_ref().map(Evidence))?;
                struct Demand<'a>(&'a pse_math::engineering_accuracy::GoalWorkDemand);
                impl serde::Serialize for Demand<'_> {
                    fn serialize<S: serde::Serializer>(
                        &self,
                        serializer: S,
                    ) -> Result<S::Ok, S::Error> {
                        let row = self.0;
                        let mut state = serializer.serialize_struct("GoalWorkDemand", 5)?;
                        fields!(state, row, product, source);
                        state.serialize_field(
                            "physical_output_allowance_bits",
                            &row.physical_output_allowance.to_bits(),
                        )?;
                        state.serialize_field(
                            "adjoint_backward_error_bits",
                            &row.adjoint_backward_error.to_bits(),
                        )?;
                        struct Rows<'a>(&'a [pse_math::engineering_accuracy::ResidualRowDemand]);
                        impl serde::Serialize for Rows<'_> {
                            fn serialize<S: serde::Serializer>(
                                &self,
                                serializer: S,
                            ) -> Result<S::Ok, S::Error> {
                                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                                for row in self.0 {
                                    seq.serialize_element(&(
                                        row.row,
                                        row.physical_allowance.to_bits(),
                                    ))?;
                                }
                                seq.end()
                            }
                        }
                        state.serialize_field("rows", &Rows(&row.rows))?;
                        state.end()
                    }
                }
                state.serialize_field("work_demand", &row.work_demand.as_ref().map(Demand))?;
                struct Cause<'a>(&'a pse_math::engineering_accuracy::GoalFailure);
                impl serde::Serialize for Cause<'_> {
                    fn serialize<S: serde::Serializer>(
                        &self,
                        serializer: S,
                    ) -> Result<S::Ok, S::Error> {
                        serializer.collect_str(&self.0.cause)
                    }
                }
                state.serialize_field("failure_description", &row.failure.as_ref().map(Cause))?;
                state.serialize_field(
                    "failure_owned_bytes",
                    &row.failure.as_ref().map(|failure| failure.bytes),
                )?;
                state.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for row in self.0 {
            seq.serialize_element(&Goal(row))?;
        }
        seq.end()
    }
}
impl serde::Serialize for DirectChecks<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        struct Row<'a>(&'a super::ModelingCheck);
        impl serde::Serialize for Row<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeStruct;
                let row = self.0;
                let mut state = serializer.serialize_struct("ModelingCheck", 33)?;
                fields!(
                    state,
                    row,
                    run_id,
                    step,
                    sample_index,
                    target_id,
                    source_id,
                    kind,
                    satisfied,
                    within_validity,
                    extrapolation_allowed,
                    basis,
                    layer,
                    claim_id,
                    claim_owner,
                    claim_owner_lineage,
                    coverage_id,
                    evidence_id,
                    form_id,
                    call_id,
                    selected_records,
                    dependencies,
                    applicability_outcome,
                    applicability_basis,
                    permission_ids,
                    unknown_allowed,
                    observation_instance,
                    applicability_required,
                    applicability_reason
                );
                state.serialize_field("time_bits", &row.time.map(f64::to_bits))?;
                state.serialize_field("value_bits", &row.value.to_bits())?;
                state.serialize_field("tolerance_bits", &row.tolerance.map(f64::to_bits))?;
                struct Inputs<'a>(&'a [pse_model::generated::runtime::modeling_checks::RuntimeModelingChecksFieldInputValuesItem]);
                impl serde::Serialize for Inputs<'_> {
                    fn serialize<S: serde::Serializer>(
                        &self,
                        serializer: S,
                    ) -> Result<S::Ok, S::Error> {
                        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                        for item in self.0 {
                            seq.serialize_element(&(
                                &item.name,
                                item.value.to_bits(),
                                item.quantity_type,
                            ))?;
                        }
                        seq.end()
                    }
                }
                struct Permissions<'a>(&'a [pse_model::generated::runtime::modeling_checks::RuntimeModelingChecksFieldApplicabilityPermissionsItem]);
                impl serde::Serialize for Permissions<'_> {
                    fn serialize<S: serde::Serializer>(
                        &self,
                        serializer: S,
                    ) -> Result<S::Ok, S::Error> {
                        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                        for item in self.0 {
                            seq.serialize_element(&(
                                item.permission_id,
                                item.scope,
                                item.target_kind,
                                &item.targets,
                                item.allow_unknown,
                                item.allow_extrapolation,
                            ))?;
                        }
                        seq.end()
                    }
                }
                state.serialize_field("input_values", &Inputs(&row.input_values))?;
                state.serialize_field(
                    "applicability_permissions",
                    &Permissions(&row.applicability_permissions),
                )?;
                state.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for row in self.0 {
            seq.serialize_element(&Row(row))?;
        }
        seq.end()
    }
}
impl serde::Serialize for DirectReports<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeSeq, SerializeStruct};
        struct Row<'a>(&'a super::ModelingReport);
        impl serde::Serialize for Row<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let row = self.0;
                let mut state = serializer.serialize_struct("ModelingReport", 10)?;
                fields!(
                    state,
                    row,
                    run_id,
                    step,
                    target_id,
                    source_id,
                    label,
                    path,
                    quantity_id,
                    unit_id,
                    transfer_context
                );
                state.serialize_field("value_bits", &row.value.to_bits())?;
                state.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for row in self.0 {
            seq.serialize_element(&Row(row))?;
        }
        seq.end()
    }
}
#[cfg(feature = "solver-kinsol")]
pub(in crate::workflow) fn initialization_context(
    services: &super::PreparationServices,
    source: &ModelingPackage,
    profile: &crate::math::initialization::InitializationProfile,
    request: &ReviewContextRequest,
    operation: Option<OperationId>,
    report: &crate::math::initialization::InitializationReport,
) -> Result<ReviewContext, WorkflowError> {
    let (mut out, owner) = strategy_builder(services, source, request, operation, report.run_id)?;
    out.settings(&profile.solver)?;
    #[derive(serde::Serialize)]
    struct Submitted<'a> {
        source: usize,
        stages: &'a [std::collections::BTreeMap<pse_ids::SemanticId, f64>],
    }
    out.fact(
        "/initialization/request".into(),
        "initialization-request.v1",
        &Submitted {
            source: 0,
            stages: &profile.stages,
        },
    )?;
    // Bound temporary projection before cloning the existing InitializationDocument.
    let entries = report
        .stages
        .iter()
        .try_fold(
            report
                .original
                .scalars
                .len()
                .saturating_add(report.values.scalars.len()),
            |count, stage| {
                count
                    .checked_add(stage.overlay.len())?
                    .checked_add(stage.candidate.scalars.len())
            },
        )
        .ok_or_else(|| contract("initialization context extent overflow"))?;
    if entries.saturating_mul(256) > out.limit.saturating_sub(out.used) {
        return Err(contract(
            "initialization outcome exceeds selected review allocation",
        ));
    }
    #[derive(serde::Serialize)]
    struct Outcome {
        run_id: RunId,
        initialization: super::InitializationDocument,
    }
    out.fact(
        "/initialization/outcome".into(),
        "initialization-outcome.v1",
        &Outcome {
            run_id: report.run_id,
            initialization: super::InitializationDocument::from(report),
        },
    )?;
    #[derive(serde::Serialize)]
    struct Attempt<'a> {
        run_id: RunId,
        ordinal: usize,
        stage: usize,
        route: super::RouteDocument,
        committed: bool,
        failure: Option<String>,
        decisions: Option<ReplayDecisions>,
        boundary_id: pse_structural::incidence::BlockId,
        boundary_inputs: &'a [pse_ids::SemanticId],
        boundary_rows: &'a [pse_ids::SemanticId],
        boundary_columns: &'a [pse_ids::SemanticId],
    }
    for (ordinal, attempt) in report.attempts.iter().enumerate() {
        out.fact(
            format!("/initialization/attempts/{ordinal}"),
            "initialization-attempt.v1",
            &Attempt {
                run_id: report.run_id,
                ordinal,
                stage: attempt.stage,
                route: attempt.strategy.into(),
                committed: attempt.committed,
                failure: attempt.result.as_ref().err().map(ToString::to_string),
                decisions: attempt
                    .trace
                    .as_ref()
                    .map(|trace| ReplayDecisions::new(ordinal, trace)),
                boundary_id: attempt.boundary.id,
                boundary_inputs: &attempt.boundary.inputs,
                boundary_rows: &attempt.boundary.members.rows,
                boundary_columns: &attempt.boundary.members.columns,
            },
        )?;
        if request.include_native_records
            && let Some(trace) = &attempt.trace
        {
            strategy_records(
                &mut out,
                &format!("/initialization/attempts/{ordinal}"),
                trace,
            )?;
        }
    }
    out.document.validate()?;
    Ok(ReviewContext {
        document: out.document,
        _owner: owner,
    })
}
#[cfg(feature = "solver-kinsol")]
#[allow(
    clippy::too_many_arguments,
    reason = "borrowed original strategy owners avoid an eager aggregate context allocation"
)]
pub(in crate::workflow) fn recycle_context(
    services: &super::PreparationServices,
    source: &ModelingPackage,
    submitted: &super::RecycleRequest,
    initial: &[f64],
    fixed: &std::collections::BTreeMap<pse_ids::SemanticId, f64>,
    settings: &impl serde::Serialize,
    request: &ReviewContextRequest,
    operation: Option<OperationId>,
    report: &crate::math::initialization::DeclaredRootReport,
) -> Result<ReviewContext, WorkflowError> {
    let (mut out, owner) = strategy_builder(services, source, request, operation, report.run_id)?;
    #[derive(serde::Serialize)]
    struct Submitted<'a> {
        source: usize,
        request: &'a super::RecycleRequest,
        initial: &'a [f64],
        fixed: &'a std::collections::BTreeMap<pse_ids::SemanticId, f64>,
    }
    out.fact(
        "/recycle/request".into(),
        "declared-recycle-request.v1",
        &Submitted {
            source: 0,
            request: submitted,
            initial,
            fixed,
        },
    )?;
    out.fact(
        "/recycle/settings".into(),
        "declared-recycle-settings.v1",
        settings,
    )?;
    #[derive(serde::Serialize)]
    struct Outcome<'a> {
        run_id: RunId,
        permission: pse_model::generated::enums::CandidateUse,
        original_candidate: Option<&'a std::collections::BTreeMap<pse_ids::SemanticId, f64>>,
    }
    out.fact(
        "/recycle/outcome".into(),
        "declared-recycle-outcome.v1",
        &Outcome {
            run_id: report.run_id,
            permission: report.candidate_use(),
            original_candidate: report
                .original_candidate
                .as_ref()
                .map(|values| &values.scalars),
        },
    )?;
    out.fact(
        "/steps/0/decisions".into(),
        "replay-decisions.v2",
        &ReplayDecisions::new(0, &report.strategy),
    )?;
    if request.include_native_records {
        strategy_records(&mut out, "/steps/0", &report.strategy)?;
    }
    out.document.validate()?;
    Ok(ReviewContext {
        document: out.document,
        _owner: owner,
    })
}

impl ModelingPackage {
    /// Retain a reviewable source/preparation failure even before run or attempt identities exist.
    pub fn review_context_failure(
        &self,
        services: &super::PreparationServices,
        request: &ReviewContextRequest,
        operation: Option<OperationId>,
        error: &WorkflowError,
    ) -> Result<ReviewContext, WorkflowError> {
        self.check_service_bindings(services)?;
        request.validate()?;
        select_single(request)?;
        let owner = services
            .math()
            .reserve("review:pre-handle-context", request.maximum_bytes)?;
        let mut out = builder(request, None, operation);
        out.source(self, 0)?;
        out.document.failure = Some(error.to_string());
        out.unavailable(
            "/run_id",
            "failure occurred before a run handle was established",
            None,
        );
        out.unavailable(
            "/constructor",
            "failed request constructor was not retained by this source owner",
            None,
        );
        out.unavailable("/outcome", "no numerical execution was established", None);
        out.document.validate()?;
        Ok(ReviewContext {
            document: out.document,
            _owner: owner,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_quality_bounded_readback_preserves_original_violations_and_ieee_bits() {
        use pse_backend_native::quality::{Quality, Violation};
        let id = pse_ids::SemanticId::from_bytes([17; 16]);
        let physical = 50.000006501572955_f64;
        let quality = Quality {
            rows: vec![Violation {
                id,
                physical,
                tolerance: 0.1,
            }],
            bounds: vec![Violation {
                id,
                physical: f64::INFINITY,
                tolerance: 1.0,
            }],
            integrality: vec![Violation {
                id,
                physical: f64::from_bits(0x7ff8_0000_0000_0001),
                tolerance: 1.0,
            }],
            normalized_max: f64::INFINITY,
        };
        let mut request = ReviewContextRequest {
            maximum_bytes: 512,
            ..Default::default()
        };
        assert!(
            builder(&request, None, None)
                .fact(
                    "/quality".into(),
                    "modeling-direct-result.v1",
                    &DirectQuality(&quality)
                )
                .is_err()
        );
        request.maximum_bytes = 1 << 20;
        let mut out = builder(&request, None, None);
        out.fact(
            "/quality".into(),
            "modeling-direct-result.v1",
            &DirectQuality(&quality),
        )
        .unwrap();
        out.document.validate().unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&out.document.facts[0].payload).unwrap();
        assert_eq!(json["rows"][0]["id"], serde_json::to_value(id).unwrap());
        assert_eq!(
            json["rows"][0]["physical_bits"].as_u64(),
            Some(physical.to_bits())
        );
        assert_eq!(
            json["rows"][0]["tolerance_bits"].as_u64(),
            Some(0.1_f64.to_bits())
        );
        assert_eq!(
            json["bounds"][0]["physical_bits"].as_u64(),
            Some(f64::INFINITY.to_bits())
        );
        assert_eq!(
            json["integrality"][0]["physical_bits"].as_u64(),
            Some(0x7ff8_0000_0000_0001)
        );
        assert_eq!(
            json["normalized_max_bits"].as_u64(),
            Some(f64::INFINITY.to_bits())
        );
    }
    #[test]
    fn direct_settings_admit_one_conservative_profile_bound_and_refuse_less() {
        let profile = crate::math::solves::SolverProfile::default();
        let extent = super::super::replay::profile_extent(&profile).unwrap();
        assert!(extent > 0);
        let request = ReviewContextRequest {
            maximum_bytes: extent + 4096,
            ..Default::default()
        };
        let mut admitted = builder(&request, None, None);
        admitted.result_settings("/result", &profile).unwrap();
        assert_eq!(
            admitted.document.facts[0].codec,
            "modeling-direct-settings.v1"
        );
        let mut refused = builder(&request, None, None);
        refused.used = request.maximum_bytes - extent + 1;
        assert!(refused.result_settings("/result", &profile).is_err());
        let mut declaration = builder(&request, None, None);
        declaration.settings(&profile).unwrap();
    }
    #[test]
    fn direct_preparation_controls_remain_readable_without_a_numerical_outcome() {
        let settings = super::super::PreparationSettings::default();
        let request = ReviewContextRequest {
            include_sources: false,
            ..Default::default()
        };
        let mut out = builder(&request, None, None);
        out.fact(
            "/declared/preparation/controls".into(),
            "modeling-preparation-settings.v1",
            &settings,
        )
        .unwrap();
        out.document.validate().unwrap();
        let restored: ReviewContextDocument =
            serde_json::from_slice(&serde_json::to_vec(&out.document).unwrap()).unwrap();
        restored.validate().unwrap();
        let actual: super::super::PreparationSettings =
            serde_json::from_slice(&restored.facts[0].payload).unwrap();
        assert_eq!(actual.compiler, settings.compiler);
        assert!(restored.run_id.is_none());
        assert!(restored.failure.is_none());
    }
    #[test]
    fn direct_numerics_preflight_preserves_caloric_contributors_and_actual_accuracy() {
        use pse_model::generated::enums::*;
        let id = pse_ids::SemanticId::from_bytes([4; 16]);
        let contributor = serde_json::json!({
            "declaration_id":id, "actual_flow_id":id, "science_revision":"rev-a", "science_records":[id],
            "power_rule_id":id, "exposure_quantity_id":id, "exposure_unit_id":id,
            "aggregate_exposure":2.0, "power_quantity_id":id, "power_unit_id":id, "power_allowance":3.0
        });
        let engineering = serde_json::from_value(serde_json::json!({
            "rule_id":null, "scale_id":null, "physical_allowance":1.0, "relative_fraction":0.1,
            "characteristic":2.0, "scale_kind":null, "source":NumericalSource::Model, "budget":1.0,
            "canonical_fallback":false, "limitation":"original context", "caloric":{
                "requirement_id":id, "instance_id":null, "source":NumericalSource::Model,
                "science_revision":"rev-a", "science_records":[id], "response_quantity_id":null,
                "response_unit_id":null, "response":null, "thermal_quantity_id":null,
                "thermal_unit_id":null, "thermal_resolution":null, "limiting_consumer":id, "allowance":1.0,
                "consumers":[{"target_id":id,"declaration_id":id,"power_rule_id":id,"actual_flow_id":id,
                    "exposure_expression":"feed","power_quantity_id":id,"power_unit_id":id,"power_allowance":3.0,
                    "exposure_quantity_id":id,"exposure_unit_id":id,"aggregate_exposure":2.0,
                    "combined_exposure":2.0,"combined_power_allowance":3.0,"contributors":[contributor]}]
            }
        })).unwrap();
        let goal = pse_model::engineering_accuracy::AccuracyGoal {
            goal_id: id.into(),
            model_id: None,
            case_id: None,
            instance_id: None,
            fit_id: None,
            target_id: id,
            target_kind: NumericalTarget::Variable,
            quantity_id: id,
            unit_id: id,
            subject: AccuracyGoalSubject::SelectedOutput,
            observation: AccuracyObservation::Steady,
            time: None,
            resolution: None,
            criterion_lower: None,
            criterion_upper: Some(10.0),
            required_class: NumericalAccuracyClass::Estimated,
            use_policy: AccuracyGoalUse::Assess,
            refine: false,
            source: NumericalSource::Analysis,
            priority: 0,
            provenance: "original".into(),
        };
        let mut policy = pse_model::numerics::NumericalPolicy::default();
        policy.goals.push(goal.clone());
        let resolved = pse_model::numerics::ResolvedNumericalPolicy {
            key: policy.key(),
            policy,
            targets: vec![pse_model::numerics::ResolvedTarget {
                id,
                kind: NumericalTarget::Row,
                quantity: id,
                unit: id,
                nominal: 1.0,
                coordinate_scale: 2.0,
                required_scale: true,
                absolute: 1.0,
                relative: 0.0,
                budget: 1.0,
                engineering: Some(engineering),
                provenance: vec![],
            }],
        };
        let accuracy = [pse_math::engineering_accuracy::GoalResult::unavailable(
            goal,
            AccuracyUnavailableReason::MissingEvidence,
        )];
        let native_accuracy = pse_backend_native::solve::ResolvedAccuracy {
            feasibility: 1e-5,
            stationarity: 2e-6,
            complementarity: 3e-7,
            integrality: 1e-8,
            gap_absolute: 1e-9,
            gap_relative: 1e-10,
            mip_absolute_gap: 1e-11,
            mip_relative_gap: 1e-12,
            acceptable: Some(pse_model::numerics::KktTolerances {
                stationarity: 0.02,
                complementarity: 0.03,
            }),
            native_scaling: true,
        };
        let facts = DirectNumerics {
            policy: &resolved,
            native_accuracy: &native_accuracy,
            accuracy: &accuracy,
        };
        let mut request = ReviewContextRequest {
            maximum_bytes: 512,
            ..Default::default()
        };
        assert!(
            builder(&request, None, None)
                .fact("/numerics".into(), "modeling-direct-numerics.v1", &facts)
                .is_err()
        );
        request.maximum_bytes = 1 << 20;
        let mut out = builder(&request, None, None);
        out.fact("/numerics".into(), "modeling-direct-numerics.v1", &facts)
            .unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&out.document.facts[0].payload).unwrap();
        assert_eq!(json["key"], serde_json::to_value(resolved.key).unwrap());
        assert_eq!(json["targets"][0]["required_scale"], true);
        assert_eq!(json["native_accuracy"]["feasibility"], 1e-5);
        assert_eq!(json["native_accuracy"]["stationarity"], 2e-6);
        assert_eq!(json["native_accuracy"]["complementarity"], 3e-7);
        assert_eq!(json["native_accuracy"]["acceptable"]["stationarity"], 0.02);
        assert_eq!(json["native_accuracy"]["native_scaling"], true);
        assert_eq!(
            json["targets"][0]["engineering"]["caloric"]["consumers"][0]["contributors"][0]["science_revision"],
            "rev-a"
        );
        assert_eq!(
            json["accuracy"][0]["goal"]["goal_id"],
            serde_json::to_value(id).unwrap()
        );
        assert_eq!(
            json["accuracy"][0]["classification"][5],
            serde_json::to_value(AccuracyUnavailableReason::MissingEvidence).unwrap()
        );
    }
    #[test]
    fn direct_report_serialization_keeps_original_nonfinite_bits_and_identity() {
        let run = RunId::from_bytes([1; 16]);
        let target = pse_ids::SemanticId::from_bytes([2; 16]);
        let bits = 0x7ff8_0000_0000_0001;
        let row = super::super::ModelingReport {
            run_id: run,
            step: 4,
            target_id: target,
            source_id: pse_model::generated::identities::DeclarationId::from(target),
            label: "original".into(),
            path: "model.output".into(),
            quantity_id: target,
            unit_id: target,
            transfer_context: None,
            value: f64::from_bits(bits),
        };
        let json = serde_json::to_value(DirectReports(&[row])).unwrap();
        assert_eq!(json[0]["run_id"], serde_json::to_value(run).unwrap());
        assert_eq!(json[0]["target_id"], serde_json::to_value(target).unwrap());
        assert_eq!(json[0]["value_bits"].as_u64(), Some(bits));
        let mut values = std::collections::BTreeMap::new();
        values.insert(target, f64::from_bits(bits));
        let json = serde_json::to_value(DirectValues(&values)).unwrap();
        assert_eq!(json[0][1].as_u64(), Some(bits));
    }
    #[test]
    fn review_context_ingress_keeps_large_integer_payload_and_rejects_unknown_version() {
        let request = ReviewContextRequest::default();
        let mut out = builder(&request, None, None);
        out.fact(
            "/exact-start".into(),
            "replay-case-inputs.v2",
            &vec![u64::MAX, (-0.0_f64).to_bits()],
        )
        .unwrap();
        let bytes = serde_json::to_vec(&out.document).unwrap();
        let restored: ReviewContextDocument = serde_json::from_slice(&bytes).unwrap();
        let exact: Vec<u64> = serde_json::from_slice(&restored.facts[0].payload).unwrap();
        assert_eq!(exact, [u64::MAX, (-0.0_f64).to_bits()]);
        let mut value = serde_json::to_value(&restored).unwrap();
        let current_version = value["version"].as_u64().unwrap();
        value["version"] = (current_version + 1).into();
        assert!(serde_json::from_value::<ReviewContextDocument>(value).is_err());
    }
    #[test]
    fn review_context_selection_refuses_unbounded_duplicate_and_oversized_facts() {
        let mut request = ReviewContextRequest {
            maximum_bytes: 128,
            ..Default::default()
        };
        assert!(request.validate().is_ok());
        assert!(
            builder(&request, None, None)
                .fact("/large".into(), "retained.v1", &vec![1; 1024])
                .is_err()
        );
        request.maximum_bytes = 0;
        assert!(request.validate().is_err());
        request.maximum_bytes = 1024;
        request.steps = vec![1, 1];
        assert!(request.validate().is_err());
    }
    #[tokio::test]
    async fn review_context_prehandle_source_failure_does_no_numerical_work() {
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p {def Root {param a:Scalar=1;var x:Scalar;eq e:x==a;}}",
            pse_ids::SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let error = contract("original pre-handle failure");
        let (result, counts) = crate::math::counted(async {
            package.review_context_failure(
                &runtime.preparation_services(),
                &ReviewContextRequest::default(),
                None,
                &error,
            )
        })
        .await;
        let context = result.unwrap();
        assert!(context.document().run_id.is_none());
        assert!(
            context
                .document()
                .failure
                .as_deref()
                .unwrap()
                .contains("original pre-handle failure")
        );
        assert!(!context.document().sources[0].objects.is_empty());
        assert_eq!(counts, Default::default());
    }
    #[derive(Debug)]
    struct UnavailableFactory {
        spec: pse_kernels::ProviderSpec,
        calls: Arc<std::sync::atomic::AtomicUsize>,
        configuration_calls: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl pse_kernels::ProviderFactory for UnavailableFactory {
        fn spec(&self) -> &pse_kernels::ProviderSpec {
            &self.spec
        }
        fn configuration_key(&self) -> pse_ids::ContentHash {
            self.configuration_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.spec.revision
        }
        fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(pse_kernels::ProviderError::Contract(
                "opaque factory has no portable implementation".into(),
            ))
        }
    }
    #[tokio::test]
    async fn review_context_opaque_factory_remains_unavailable_without_instantiation() {
        use pse_kernels::{
            AdmittedProvider, DerivativeOrder, DerivativeSource, ProviderShapes, ProviderSpec,
            Registration,
        };
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let configuration_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let quantity = physical.quantities.neutral_dimensionless().unwrap();
        let unit = physical
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let port = |n| pse_kernels::Port {
            id: pse_ids::SemanticId::from_bytes([n; 16]),
            quantity,
            unit,
        };
        let spec = ProviderSpec {
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            id: pse_ids::SemanticId::from_bytes([91; 16]),
            revision: pse_ids::ContentHash::from_bytes([92; 32]),
            data: pse_ids::ContentHash::from_bytes([93; 32]),
            inputs: vec![port(94)],
            outputs: vec![port(95)],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        let registration = Registration::bind(
            AdmittedProvider::new(spec.clone(), &physical.quantities).unwrap(),
            Arc::new(UnavailableFactory {
                spec,
                calls: calls.clone(),
                configuration_calls: configuration_calls.clone(),
            }),
        )
        .unwrap();
        let rows = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar;eq e:x==1;}}",
            pse_ids::SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package_registered(
                rows,
                physical,
                std::collections::BTreeMap::from([("opaque".into(), registration.clone())]),
            )
            .await
            .unwrap();
        let configurations_before = configuration_calls.load(std::sync::atomic::Ordering::SeqCst);
        let context = package
            .review_context_failure(
                &runtime.preparation_services(),
                &ReviewContextRequest::default(),
                None,
                &contract("original failed request"),
            )
            .unwrap();
        assert_eq!(
            configuration_calls.load(std::sync::atomic::Ordering::SeqCst),
            configurations_before
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(context.document().sources[0].providers.len(), 1);
        assert!(
            context.document().sources[0].providers[0]
                .configuration
                .is_none()
        );
        assert!(
            context
                .document()
                .unavailable
                .iter()
                .any(|item| item.path.contains("opaque/implementation"))
        );
        context.document().validate().unwrap();
        let restored: ReviewContextDocument =
            serde_json::from_slice(&serde_json::to_vec(context.document()).unwrap()).unwrap();
        let root = package
            .declarations(&runtime.preparation_services())
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let mut prepared = package
            .prepare_solve(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                pse_compiler::workspace::ModelingCaseBindings {
                    values: std::collections::BTreeMap::from([("x".into(), 1.)]),
                    variables: std::collections::BTreeMap::from([(
                        "x".into(),
                        pse_compiler::workspace::ModelingVariableState {
                            fixed: Some(true),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
                DerivativeOrder::First,
                crate::workflow::tests::compiler_profile(),
                crate::workflow::tests::profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        prepared.solve = prepared
            .solve
            .with_providers(std::collections::BTreeMap::from([(
                registration.spec().key(),
                registration,
            )]))
            .unwrap();
        let before_identity = configuration_calls.load(std::sync::atomic::Ordering::SeqCst);
        prepared.solve.preparation_identity().unwrap();
        assert!(
            configuration_calls.load(std::sync::atomic::Ordering::SeqCst) > before_identity,
            "control reaches the real preparation-identity factory callback path"
        );
        let configurations_before = configuration_calls.load(std::sync::atomic::Ordering::SeqCst);
        let result = Arc::new(RunResult::joined(
            RunId::from_bytes([92; 16]),
            runtime.execution_services(),
            crate::workflow::RunOutcome::Modeling {
                request: vec![prepared],
                result: Err(Arc::new(contract("retained pre-dispatch failure"))),
            },
            None,
        ));
        let run_context = result
            .review_context(&ReviewContextRequest::default())
            .unwrap();
        assert_eq!(
            configuration_calls.load(std::sync::atomic::Ordering::SeqCst),
            configurations_before
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(
            run_context
                .document()
                .unavailable
                .iter()
                .any(|item| item.path.ends_with("/preparation_identity"))
        );
        let coordinates = run_context
            .document()
            .facts
            .iter()
            .find(|fact| fact.path == "/steps/0/context")
            .unwrap();
        assert_eq!(coordinates.codec, "scientific-coordinates.v1");
        let decoded: ReviewScientificCoordinates =
            serde_json::from_slice(&coordinates.payload).unwrap();
        assert_eq!(decoded.run_id, result.run_id);
        assert_eq!(decoded.step, 0);
        drop(run_context);
        drop(result);
        drop(context);
        drop(package);
        drop(runtime);
        restored.validate().unwrap();
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
    #[test]
    fn review_context_pre_source_admission_failure_preserves_original_request() {
        let fact = ReviewContextFact {
            path: "/submitted/source".into(),
            codec: "authored-source.v1".into(),
            payload: serde_json::to_vec("original malformed source").unwrap(),
        };
        let context = ReviewContextDocument::pre_handle_failure(
            ReviewContextRequest::default(),
            None,
            "original parse error".into(),
            vec![fact],
        )
        .unwrap();
        assert!(context.run_id.is_none());
        assert!(context.sources.is_empty());
        assert_eq!(context.failure.as_deref(), Some("original parse error"));
        assert_eq!(
            serde_json::from_slice::<String>(&context.facts[0].payload).unwrap(),
            "original malformed source"
        );
    }
}
