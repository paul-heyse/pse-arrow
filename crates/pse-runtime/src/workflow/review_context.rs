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
    pub version: Version<1>,
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
        let extent = super::replay::profile_extent(profile)?.saturating_mul(8);
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
            .runtime
            .shared
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
    runtime: &super::Runtime,
    source: &ModelingPackage,
    request: &ReviewContextRequest,
    operation: Option<OperationId>,
    run_id: RunId,
) -> Result<(Builder, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    request.validate()?;
    select_single(request)?;
    let owner = runtime
        .shared
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
#[cfg(feature = "solver-kinsol")]
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
#[cfg(feature = "solver-kinsol")]
pub(in crate::workflow) fn initialization_context(
    runtime: &super::Runtime,
    source: &ModelingPackage,
    profile: &crate::math::initialization::InitializationProfile,
    request: &ReviewContextRequest,
    operation: Option<OperationId>,
    report: &crate::math::initialization::InitializationReport,
) -> Result<ReviewContext, WorkflowError> {
    let (mut out, owner) = strategy_builder(runtime, source, request, operation, report.run_id)?;
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
    runtime: &super::Runtime,
    source: &ModelingPackage,
    submitted: &super::RecycleRequest,
    initial: &[f64],
    fixed: &std::collections::BTreeMap<pse_ids::SemanticId, f64>,
    settings: &impl serde::Serialize,
    request: &ReviewContextRequest,
    operation: Option<OperationId>,
    report: &crate::math::initialization::DeclaredRootReport,
) -> Result<ReviewContext, WorkflowError> {
    let (mut out, owner) = strategy_builder(runtime, source, request, operation, report.run_id)?;
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
        request: &ReviewContextRequest,
        operation: Option<OperationId>,
        error: &WorkflowError,
    ) -> Result<ReviewContext, WorkflowError> {
        request.validate()?;
        select_single(request)?;
        let owner = self
            .runtime
            .shared
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
        value["version"] = 2.into();
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
            package.review_context_failure(&ReviewContextRequest::default(), None, &error)
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
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let mut prepared = package
            .prepare_solve(
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
            runtime.clone(),
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
