// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable scientific replay levels. Reruns are new executions, never occurrence recovery.
use super::{
    ModelingPackage, PhysicalContext, RunHandle, RunReport, RunRequest, RunResult, Runtime,
    WorkflowError, contract,
};
use pse_backend_native::{
    execution::BackendSettings,
    solve::{Backend, Controls, SolveIntent, SolverSelection},
};
use pse_ids::{ContentHash, SemanticId};
use pse_model::{
    diagnostic::Observation,
    generated::{enums::ExternalDerivativeSource, identities::RunId},
    strategy::{CompositionRequest, NumericalStrategy, StartOrigin},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn rebind_retained_sources<'a>(
    sources: impl Iterator<Item = &'a ModelingPackage>,
    runtime: &Runtime,
) -> Result<Vec<ModelingPackage>, WorkflowError> {
    let activity = runtime.admit_activity()?;
    activity.sync_scope(|| {
        let count = sources
            .size_hint()
            .1
            .ok_or_else(|| contract("retained replay positional extent absent"))?;
        let source_extent = count
            .checked_mul(size_of::<&ModelingPackage>() + 1024)
            .and_then(|bytes| bytes.checked_add(256))
            .ok_or_else(|| contract("retained replay source view extent"))?;
        let _sources_owner = runtime
            .shared
            .math()
            .reserve("replay:receiver-source-view", source_extent)?;
        let sources = sources.collect::<Vec<_>>();
        let mut seen: BTreeMap<_, Vec<&ModelingPackage>> = BTreeMap::new();
        let bytes = sources
            .iter()
            .try_fold(1024usize, |bytes, source| {
                let candidates = seen.entry(source.replay_source_owner_key()).or_default();
                let duplicate = candidates
                    .iter()
                    .any(|original| source.same_replay_source(original));
                if !duplicate {
                    candidates.push(source);
                }
                bytes
                    .checked_add(
                        size_of::<ModelingPackage>()
                            .checked_mul(2)?
                            .checked_add(1024)?,
                    )?
                    .checked_add(if duplicate {
                        source.physical.metadata_bytes()
                    } else {
                        0
                    })
            })
            .ok_or_else(|| contract("retained replay positional extent overflow"))?;
        drop(seen);
        let owner = runtime
            .shared
            .math()
            .reserve("replay:receiver-positions", bytes)?;
        let mut packages = map_retained_sources(sources.into_iter(), |source| {
            source.reconstruct_replay_source(runtime)
        })?;
        for package in &mut packages {
            package.replay_owner = Some(owner.clone());
        }
        Ok(packages)
    })
}

/// The memo lives only for this operation; borrowed originals keep pointer candidates alive.
fn map_retained_sources<'a, T: Clone>(
    sources: impl Iterator<Item = &'a ModelingPackage>,
    mut convert: impl FnMut(&ModelingPackage) -> Result<T, WorkflowError>,
) -> Result<Vec<T>, WorkflowError> {
    let mut memo: BTreeMap<_, Vec<(&ModelingPackage, usize)>> = BTreeMap::new();
    let mut output: Vec<T> = Vec::with_capacity(sources.size_hint().0);
    for source in sources {
        let candidates = memo.entry(source.replay_source_owner_key()).or_default();
        if let Some((_, position)) = candidates
            .iter()
            .find(|(original, _)| source.same_replay_source(original))
        {
            let value = output[*position].clone();
            output.push(value);
        } else {
            let value = convert(source)?;
            candidates.push((source, output.len()));
            output.push(value);
        }
    }
    Ok(output)
}

/// One complete canonical source object, including actual payload and lexical references.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaySourceObject {
    /// Original logical identity.
    pub logical: String,
    /// Original lexical scope.
    pub scope: String,
    /// Original lexical name.
    pub name: String,
    /// Checked content/reference version.
    pub version: String,
    /// Exact producer interpretation.
    pub kind: String,
    /// Complete immutable input bytes.
    pub payload: Vec<u8>,
    /// Complete declared lexical dependencies.
    pub references: Vec<(String, String)>,
}
/// Actual admitted physical source bytes, retaining empty membership distinctly from omission.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReplayPhysicalSource {
    /// Registry relation identity.
    pub relation: SemanticId,
    /// Complete Arrow IPC stream, including schema and actual admitted rows.
    pub ipc: Vec<u8>,
}
pub(in crate::workflow) fn physical_sources(
    physical: &PhysicalContext,
) -> Result<Vec<ReplayPhysicalSource>, WorkflowError> {
    use datafusion::arrow::ipc::writer::StreamWriter;
    physical
        .sources
        .values()
        .map(|batch| {
            let mut ipc = Vec::new();
            {
                let mut writer =
                    StreamWriter::try_new(&mut ipc, &batch.batch().schema()).map_err(|error| {
                        contract(format!("replay physical source encoding: {error}"))
                    })?;
                writer.write(batch.batch()).map_err(|error| {
                    contract(format!("replay physical source encoding: {error}"))
                })?;
                writer.finish().map_err(|error| {
                    contract(format!("replay physical source encoding: {error}"))
                })?;
            }
            Ok(ReplayPhysicalSource {
                relation: batch.relation_id(),
                ipc,
            })
        })
        .collect()
}
/// Complete ordered scalar ABI coordinate of a required executable provider.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReplayProviderPort {
    /// Exact scalar port identity.
    pub id: SemanticId,
    /// Physical quantity, including basis and reference.
    pub quantity: SemanticId,
    /// Actual representation unit.
    pub unit: SemanticId,
}
/// Explicit logical array layout over a provider's scalar ABI.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReplayProviderShape {
    /// Logical argument/result identity.
    pub id: SemanticId,
    /// Ordered index axes.
    pub axes: Vec<SemanticId>,
    /// Exact sparse/ragged coordinates.
    pub coordinates: Vec<Vec<SemanticId>>,
    /// Ordered scalar ABI identities.
    pub cells: Vec<SemanticId>,
}
/// Required real executable registration; a descriptor alone cannot supply its factory.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReplayProvider {
    /// Authored callable name.
    pub name: String,
    /// Implementation identity.
    pub implementation: SemanticId,
    /// Algorithm and physical revision.
    pub revision: ContentHash,
    /// Actual parameter-data identity.
    pub data: ContentHash,
    /// Actual factory settings, envelope and starts identity.
    pub configuration: ContentHash,
    /// Ordered full input ABI.
    pub inputs: Vec<ReplayProviderPort>,
    /// Ordered full output ABI.
    pub outputs: Vec<ReplayProviderPort>,
    /// Full logical input shape.
    pub input_shapes: Vec<ReplayProviderShape>,
    /// Full logical output shape.
    pub output_shapes: Vec<ReplayProviderShape>,
    /// Provenance of actual partials.
    pub derivative_source: ExternalDerivativeSource,
    /// Actual derivative order: 0 values, 1 first, 2 second.
    pub derivatives: u8,
    /// Established smoothness order, separately from implemented derivatives.
    pub smoothness: u8,
}
impl ReplayProvider {
    pub(in crate::workflow) fn new(name: &str, registration: &pse_kernels::Registration) -> Self {
        let descriptor = super::review_context::ReviewProviderDescriptor::new(name, registration);
        Self {
            name: descriptor.name,
            implementation: descriptor.implementation,
            revision: descriptor.revision,
            data: descriptor.data,
            configuration: registration.configuration_key(),
            inputs: descriptor.inputs,
            outputs: descriptor.outputs,
            input_shapes: descriptor.input_shapes,
            output_shapes: descriptor.output_shapes,
            derivative_source: descriptor.derivative_source,
            derivatives: descriptor.derivatives,
            smoothness: descriptor.smoothness,
        }
    }
}
/// Exportable source closure. Reconstruction requires matching actual admitted physical/provider premises.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaySourceDocument {
    /// Source identity, checked against the complete context object.
    pub identity: ContentHash,
    /// Exact generated registry interpretation.
    pub interpretation: String,
    /// Required linked native build.
    pub build: ContentHash,
    /// Required admitted physical interpretation.
    pub physical_identity: ContentHash,
    /// Complete original physical source streams.
    pub physical_sources: Vec<ReplayPhysicalSource>,
    /// Full required callable descriptor bodies; actual executable factories are supplied separately.
    pub providers: Vec<ReplayProvider>,
    /// Complete authored/data/reference object inventory.
    pub objects: Vec<ReplaySourceObject>,
}
impl ReplaySourceDocument {
    /// Re-admit actual source bytes through the production pipeline, without a store/history lookup.
    pub fn reconstruct(
        &self,
        runtime: &Runtime,
        physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>,
    ) -> Result<ModelingPackage, WorkflowError> {
        let activity = runtime.admit_activity()?;
        activity.sync_scope(|| runtime.admit_replay_source(self, physical, providers))
    }
}
/// Stream the modeling owner's exact JSON codec as bytes without an unadmitted
/// intermediate string/vector. Bindings retain their native typed owner in Rust;
/// the external closure carries its lossless body rather than a second value taxonomy.
fn serialize_bindings<S: serde::Serializer>(
    value: &pse_modeling::Bindings,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::{Error as _, SerializeSeq as _};
    struct JsonBytes<Q>(Q);
    impl<Q: serde::ser::SerializeSeq> std::io::Write for JsonBytes<Q> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            for byte in bytes {
                self.0
                    .serialize_element(byte)
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut bytes = JsonBytes(serializer.serialize_seq(None)?);
    serde_json::to_writer(&mut bytes, value).map_err(S::Error::custom)?;
    bytes.0.end()
}
/// Full actual requested solver controls; native settings remain their original typed authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaySolverSettings {
    /// Stable original structural block overrides, including their complete controls.
    pub block_overrides:
        BTreeMap<pse_structural::incidence::BlockId, crate::math::settings::BlockSolveOverride>,
    /// Original explicitly admitted independent block width.
    pub block_workers: usize,
    /// Original scientific analysis goal.
    pub intent: SolveIntent,
    /// Explicit native selection; absent means Auto.
    pub backend: Option<Backend>,
    /// Full materialized backend settings.
    pub settings: Option<BackendSettings>,
    /// Original finite controls, including wall allowance, options, starts and reuse.
    pub controls: Controls,
    /// Complete original composition, frozen advice, branch and finite work limits.
    pub composition: CompositionRequest,
    /// Full ID-keyed scientific numerical requirements.
    pub numerics: pse_model::numerics::NumericalPolicy,
    /// Requested preprocessing mode.
    pub presolve: pse_backend_native::presolve::PolicyKind,
    /// Actual materialized library options, including explicit native defaults.
    pub presolve_options: pse_backend_native::solve::Options,
    /// Passes which cannot silently disappear.
    pub required_passes: BTreeSet<pse_backend_native::presolve::Pass>,
    /// Optional absolute convexity assessment budget.
    pub convexity_absolute: Option<f64>,
    /// Optional relative convexity assessment budget.
    pub convexity_relative: Option<f64>,
    /// Original optional reconstruction accuracy class and finite allowances.
    pub reconstruction: Option<pse_backend_native::derived::ReconstructionAccuracy>,
    /// Original requested physical sensitivities and propagation inputs.
    pub sensitivity: Option<crate::math::settings::SensitivityRequest>,
}
impl ReplaySolverSettings {
    /// Project once from actual admitted profile bodies, never from a hash or current defaults.
    pub fn from_profile(profile: &crate::math::solves::SolverProfile) -> Self {
        let (convexity_absolute, convexity_relative) = match profile.convexity {
            pse_math::convexity::ConvexityPolicy::Exact => (None, None),
            pse_math::convexity::ConvexityPolicy::Numerical { absolute, relative } => {
                (Some(absolute), Some(relative))
            }
        };
        Self {
            block_overrides: profile.block_overrides.clone(),
            block_workers: profile.block_workers,
            intent: profile.intent,
            backend: match profile.selection {
                SolverSelection::Auto => None,
                SolverSelection::Explicit(backend) => Some(backend),
            },
            settings: profile.backend.document().cloned(),
            controls: profile.controls.clone(),
            composition: profile.composition.clone(),
            numerics: profile.numerics.clone(),
            presolve: profile.presolve.kind(),
            presolve_options: profile.presolve.document_options(),
            required_passes: match &profile.presolve {
                pse_backend_native::presolve::Policy::Explicit { required, .. } => required.clone(),
                _ => BTreeSet::new(),
            },
            convexity_absolute,
            convexity_relative,
            reconstruction: profile.reconstruction,
            sensitivity: profile.sensitivity.clone(),
        }
    }
}
/// Full retained governing declaration and synchronous decisions for one original step.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayDecisions {
    /// Original step ordinal.
    pub step: usize,
    /// Full effective numerical declaration, including ordering and all limits.
    pub declaration: NumericalStrategy,
    /// Original scientific problem identity.
    pub original: ContentHash,
    /// Actual effective profile identity.
    pub profile: ContentHash,
    /// Actual original start provenance.
    pub start: StartOrigin,
    /// Actual per-rung starts in their consumed order.
    pub starts: Vec<StartOrigin>,
    /// Every retained synchronous decision, independent of ordinary progress drops.
    pub decisions: Vec<super::attempt_documents::AttemptDecision>,
}
impl ReplayDecisions {
    pub(super) fn new(step: usize, trace: &crate::math::strategy::Trace) -> Self {
        Self {
            step,
            declaration: trace.declaration.clone(),
            original: trace.original,
            profile: trace.profile,
            start: trace.start,
            starts: trace.starts.clone(),
            decisions: trace
                .events
                .iter()
                .map(super::attempt_documents::AttemptDecision::from)
                .collect(),
        }
    }
}
/// Recorded coverage, without promoting an interrupted prefix into a full trajectory.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayCoverage {
    /// Original requested modeling steps or dynamic samples/horizon periods.
    pub requested: usize,
    /// Actual retained completed steps/samples/periods.
    pub observed: usize,
    /// Actual last completed physical time when available.
    pub completed_time: Option<Observation>,
    /// Original completion permits the requested scientific result.
    pub usable: bool,
    /// Actual terminal cause projection, when retained.
    pub terminal: Option<pse_model::generated::enums::NumericalAttemptObservation>,
}
/// Complete export of available scientific premises. Compiled products/factories remain in-process capabilities.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayDocument {
    /// Explicit current document version.
    pub version: pse_model::document::Version<3>,
    /// Historical source execution; a numerical rerun always gets another identity.
    pub source_run: RunId,
    /// Exact original independent-sequence failure policy.
    pub continue_independent: bool,
    /// Actual immutable build/runtime observations used by constructor admission.
    pub runtime_premises: Vec<pse_backend_native::execution::Snapshot>,
    /// Exact required binary build.
    pub build: ContentHash,
    /// Full immutable authored/provider/physical source closures.
    pub sources: Vec<ReplaySourceDocument>,
    /// Full actual solver settings for all original solver owners.
    pub solver_settings: Vec<ReplaySolverSettings>,
    /// Original portable case constructors, retaining all supplied structural and numerical inputs.
    pub case_constructors: Vec<ReplayCaseConstructor>,
    /// Complete original simulation constructors across all linked methods.
    pub simulation_constructors: Vec<ReplaySimulationConstructor>,
    /// Complete original fit and experiment constructor.
    pub fit_constructor: Option<ReplayFitConstructor>,
    /// Complete original closed-loop plant and stage constructors.
    pub horizon_constructor: Option<ReplayHorizonConstructor>,
    /// Complete original shooting and simulation constructor.
    pub shooting_constructor: Option<ReplayShootingConstructor>,
    /// Original physical mappings and full constructor configurations, using existing producer documents.
    pub contexts: Vec<pse_model::generated::runtime::scientific_contexts::Row>,
    /// Full original horizon provenance, including output demand, period, stages and mappings.
    pub horizon: Option<Vec<u8>>,
    /// Full retained decisions with their actual effective declaration.
    pub decisions: Vec<ReplayDecisions>,
    /// Full actual retained adapter settings, submitted starts and original coordinate witnesses.
    pub native_attempts: Vec<ReplayNativeAttempt>,
    /// Native record coverage is explicit for integrated trajectories.
    pub native_record_coverage: ReplayNativeRecordCoverage,
    /// Explicit actual scientific observation coverage.
    pub coverage: ReplayCoverage,
    /// Native continuation has no library checkpoint in the current source owner.
    pub native_continuation: bool,
    #[serde(skip)]
    #[schemars(skip)]
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Actual immutable input closure, independent of mutable history and occurrence recovery.
#[derive(Clone, Debug)]
pub struct ScientificReplay {
    source_run: RunId,
    result: Arc<RunResult>,
    coverage: ReplayCoverage,
}
impl ScientificReplay {
    /// Full immutable production inputs with actual physical/provider/compiled owners.
    pub fn inputs(&self) -> RunRequest<'_> {
        self.result.replay_request()
    }
    fn traces(&self) -> Box<dyn Iterator<Item = (usize, &Arc<crate::math::strategy::Trace>)> + '_> {
        self.result.replay_traces()
    }
    fn decision_extent(&self) -> Result<usize, WorkflowError> {
        self.traces().try_fold(512usize, |bytes, (_, trace)| {
            bytes
                .checked_add(
                    trace
                        .retained_bytes()
                        .map_err(crate::math::MathRuntimeError::from)?
                        .checked_mul(4)
                        .ok_or_else(|| contract("replay decision extent"))?,
                )
                .ok_or_else(|| contract("replay decision extent"))
        })
    }
    fn decision_rows(&self) -> Vec<ReplayDecisions> {
        self.traces()
            .map(|(step, trace)| ReplayDecisions::new(step, trace))
            .collect()
    }
    /// Inspect exact original effective declarations/decisions under finite admission.
    pub fn recorded_decisions(&self) -> Result<ReplayDecisionDocument, WorkflowError> {
        let bytes = self.decision_extent()?;
        if bytes > EXPORT_MAX {
            return Err(contract(
                "replay decision body exceeds finite export capacity",
            ));
        }
        let owner = self
            .result
            .runtime
            .shared
            .math()
            .reserve("replay:recorded-decisions", bytes)?;
        Ok(ReplayDecisionDocument {
            decisions: self.decision_rows(),
            _owner: owner,
        })
    }
    fn sources(&self) -> Box<dyn Iterator<Item = &ModelingPackage> + '_> {
        match self.inputs() {
            RunRequest::Modeling(steps) => Box::new(steps.iter().map(|step| &step.source)),
            RunRequest::Simulation(simulation) => Box::new(std::iter::once(&simulation.source)),
            RunRequest::Fit(fit) => Box::new(std::iter::once(&fit.source)),
            RunRequest::Horizon(horizon) => Box::new(horizon.sources.iter()),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, .. } => {
                Box::new(std::iter::once(&problem.simulation.source))
            }
        }
    }
    /// Reconstruct actual locally usable source packages into a receiving runtime.
    pub fn reconstruct_sources(
        &self,
        runtime: &Runtime,
    ) -> Result<Vec<ModelingPackage>, WorkflowError> {
        rebind_retained_sources(self.sources(), runtime)
    }
    /// Default bounded complete export. Oversized closure refuses instead of truncating required inputs.
    pub fn document(&self) -> Result<ReplayDocument, WorkflowError> {
        self.document_with_limit(16 * 1024 * 1024)
    }
    /// Explicit finite export allocation. Caller receives either the complete closure or refusal.
    pub fn document_with_limit(&self, bytes: usize) -> Result<ReplayDocument, WorkflowError> {
        if bytes == 0 || bytes > 256 * 1024 * 1024 {
            return Err(contract(
                "replay export requires a finite allocation up to 256 MiB",
            ));
        }
        let native_extent =
            self.native_attempts()
                .try_fold(0usize, |bytes, (_, _, _, native)| {
                    bytes
                        .checked_add(
                            encoded_extent(native.as_ref(), EXPORT_MAX)?
                                .saturating_mul(8)
                                .saturating_add(256),
                        )
                        .ok_or_else(|| contract("replay native extent overflow"))
                })?;
        let extent = self
            .sources()
            .try_fold(
                self.constructor_extent()?
                    .saturating_add(self.decision_extent()?)
                    .saturating_add(native_extent),
                |total, source| total.checked_add(source.replay_source_extent()),
            )
            .ok_or_else(|| contract("replay export extent overflow"))?;
        if extent > bytes {
            return Err(contract(
                "complete replay closure exceeds requested export allocation",
            ));
        }
        let owner = self
            .result
            .runtime
            .shared
            .math()
            .reserve("replay:complete-export", bytes)?;
        // Wire positions and every final duplicate byte remain part of the unchanged
        // format and extent. Only repeated conversion of the same retained input is skipped.
        let sources = map_retained_sources(self.sources(), ModelingPackage::replay_source)?;
        let mut runtime_premises = Vec::new();
        let mut solver_settings = Vec::new();
        let mut case_constructors = Vec::new();
        let mut simulation_constructors = Vec::new();
        let mut fit_constructor = None;
        let mut horizon_constructor = None;
        #[cfg(feature = "solver-diffsol")]
        let mut shooting_constructor = None;
        #[cfg(not(feature = "solver-diffsol"))]
        let shooting_constructor = None;
        let mut contexts = Vec::new();
        let mut horizon = None;
        match self.inputs() {
            RunRequest::Modeling(steps) => {
                for (index, step) in steps.iter().enumerate() {
                    runtime_premises.push(step.solve.execution_snapshot().clone());
                    solver_settings.push(ReplaySolverSettings::from_profile(&step.profile));
                    let mut constructor = step.original_inputs.document(index)?;
                    constructor.prepared_graph = Some(ReplayPreparedGraph::new(&step.solve)?);
                    constructor.requested_start = step
                        .solve
                        .requested_start()
                        .map(pse_backend_native::solve::WarmStart::snapshot);
                    constructor.requested_screened_start = step
                        .solve
                        .requested_screened_start()
                        .map(ReplayScreenedStart::new);
                    case_constructors.push(constructor);
                    contexts.push(super::artifacts::solve_context(
                        step,
                        self.source_run,
                        i64::try_from(index).map_err(|_| contract("replay step ordinal"))?,
                    )?);
                }
            }
            RunRequest::Simulation(simulation) => {
                runtime_premises.push(simulation.snapshot().clone());
                simulation_constructors.push(
                    simulation
                        .original_inputs
                        .as_ref()
                        .ok_or_else(|| contract("original simulation constructor absent"))?
                        .document(0),
                );
                contexts.push(
                    simulation.scientific_context(
                        self.source_run,
                        super::dynamics::profile_identity(simulation.profile())
                            .map_err(crate::math::MathRuntimeError::from)?
                            .as_id(),
                    )?,
                );
            }
            RunRequest::Fit(fit) => {
                runtime_premises.push(fit.execution_snapshot().clone());
                fit_constructor = Some(fit.original_inputs.document(0));
                solver_settings.push(ReplaySolverSettings::from_profile(
                    &fit.problem.profile.solver,
                ));
                contexts.push(fit.scientific_context(self.source_run)?);
            }
            RunRequest::Horizon(request) => {
                horizon = Some(request.provenance.clone());
                let original = &request.original;
                runtime_premises.push(original.plant.snapshot().clone());
                horizon_constructor = Some(ReplayHorizonConstructor::new(original)?);
                contexts.push(
                    original.plant.scientific_context(
                        self.source_run,
                        super::dynamics::profile_identity(original.plant.profile())
                            .map_err(crate::math::MathRuntimeError::from)?
                            .as_id(),
                    )?,
                );
                if let Some(estimator) = &original.estimator {
                    solver_settings.push(ReplaySolverSettings::from_profile(
                        &estimator.analysis.solver,
                    ));
                }
                if let Some(controller) = &original.controller {
                    solver_settings.push(ReplaySolverSettings::from_profile(
                        &controller.analysis.solver,
                    ));
                }
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => {
                runtime_premises.push(problem.execution_snapshot().clone());
                shooting_constructor = Some(ReplayShootingConstructor::new(problem, initial)?);
                solver_settings.push(ReplaySolverSettings::from_profile(&problem.solver));
                let mut context = problem
                    .simulation
                    .scientific_context(self.source_run, problem.profile_key.as_id())?;
                // The exact caller initial is part of input reconstruction, not a native checkpoint.
                context.binding = Some(
                    serde_json::to_vec(initial)
                        .map_err(|error| contract(error.to_string()))?
                        .into(),
                );
                contexts.push(context);
            }
        }
        let document = ReplayDocument {
            version: pse_model::document::Version,
            source_run: self.source_run,
            continue_independent: self.result.modeling_continue_independent,
            runtime_premises,
            build: pse_backend_native::execution::LINKED.build_identity(),
            sources,
            solver_settings,
            case_constructors,
            simulation_constructors,
            fit_constructor,
            horizon_constructor,
            shooting_constructor,
            contexts,
            horizon,
            decisions: self.decision_rows(),
            native_attempts: self
                .native_attempts()
                .map(|(step, mechanism, component, native)| ReplayNativeAttempt {
                    step,
                    mechanism,
                    component,
                    record: native.clone(),
                })
                .collect(),
            native_record_coverage: match self.inputs() {
                RunRequest::Simulation(_) | RunRequest::Horizon(_) => {
                    ReplayNativeRecordCoverage::IntegrationReportOnly
                }
                _ => ReplayNativeRecordCoverage::StrategyAttempts,
            },
            coverage: self.coverage.clone(),
            native_continuation: false,
            _owner: owner,
        };
        encoded_extent(&document, bytes)?;
        Ok(document)
    }
    /// Start a new tolerant execution under fresh identity/allowance through the original supervisor.
    /// Frozen controls/inputs remain exact; scheduling, floating-point/native paths may differ.
    pub async fn rerun(&self) -> Result<RunHandle, WorkflowError> {
        self.rerun_with_cancel(&crate::CancelSource::new()).await
    }
    /// Fresh replay with caller cancellation shared by pending preparation/admission.
    pub async fn rerun_with_cancel(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        match self.inputs() {
            RunRequest::Modeling(steps) => {
                let mut fresh = Vec::with_capacity(steps.len());
                for step in steps {
                    let mut source = step.source.clone();
                    source.held_preparation = None;
                    let mut prepared = step
                        .original_inputs
                        .reconstruct_in_process(&source, cancel)
                        .await?;
                    // Postconstructor declarations are scientific input. Original-only graphs
                    // can detach recursively; scoped opaque products require their producer.
                    if step.solve.replay_declaration() != prepared.solve.replay_declaration()
                        || !same_inventory(
                            step.solve.replay_automatic_products(),
                            prepared.solve.replay_automatic_products(),
                        )?
                    {
                        let bytes = step
                            .solve
                            .replay_metadata_extent()
                            .map_err(crate::math::MathRuntimeError::from)?
                            .checked_add(ReplayPreparedGraph::extent(&step.solve, 0, &mut 0)?)
                            .ok_or_else(|| contract("fresh replay original graph copy extent"))?;
                        let owner = self
                            .result
                            .runtime
                            .shared
                            .math()
                            .reserve("replay:fresh-original-graph", bytes)?;
                        prepared.solve = step
                            .solve
                            .fresh_replay_copy()
                            .map_err(crate::math::MathRuntimeError::from)?
                            .retaining_replay_metadata(owner);
                    }
                    if step.solve.requested_screened_start().is_some() {
                        return Err(contract(
                            "replay of a requested screened start requires producer re-screening under the fresh task; historical screening does not grant new permission",
                        ));
                    }
                    if let Some(start) = step.solve.requested_start() {
                        let owner = self
                            .result
                            .runtime
                            .shared
                            .math()
                            .reserve("replay:requested-start", warm_start_extent(start))?;
                        prepared.solve = prepared
                            .solve
                            .with_start_owned(start.clone(), owner)
                            .map_err(crate::math::MathRuntimeError::from)?;
                        prepared.profile.controls.start =
                            pse_backend_native::solve::StartPolicy::Explicit;
                    }
                    fresh.push(prepared);
                }
                self.result
                    .runtime
                    .start_modeling(fresh, self.result.modeling_continue_independent, cancel)
                    .await
            }
            RunRequest::Simulation(simulation) => simulation.start_with_cancel(cancel),
            RunRequest::Fit(fit) => fit.start_with_cancel(cancel),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Horizon(request) => {
                let mut horizon = request.original.as_ref().clone();
                // The old engagement has drained. A new execution may retain immutable
                // science, but must establish its own preparation lane.
                horizon.plant.source.held_preparation = None;
                if let Some(estimator) = &mut horizon.estimator {
                    estimator.package.held_preparation = None;
                }
                if let Some(controller) = &mut horizon.controller {
                    controller.package.held_preparation = None;
                }
                self.result.runtime.start_horizon(horizon, cancel).await
            }
            #[cfg(not(feature = "solver-diffsol"))]
            RunRequest::Horizon(_) => Err(contract(
                "horizon replay requires the original linked dynamic supervisor",
            )),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => {
                problem.start_replay_with_cancel(initial.clone(), cancel)
            }
        }
    }
    /// Numerical points are not library continuation checkpoints.
    pub fn native_continuation(&self) -> Result<RunHandle, WorkflowError> {
        Err(contract(
            "native continuation is unsupported: no library-owned checkpoint was exported",
        ))
    }
}
impl RunResult {
    pub(super) fn replay_request(&self) -> RunRequest<'_> {
        self.original_request()
    }
    pub(super) fn replay_traces(
        &self,
    ) -> Box<dyn Iterator<Item = (usize, &Arc<crate::math::strategy::Trace>)> + '_> {
        let retained: Box<dyn Iterator<Item = (usize, &Arc<crate::math::strategy::Trace>)> + '_> =
            match self.request() {
                RunRequest::Modeling(steps) => Box::new((0..steps.len()).filter_map(|index| {
                    self.modeling_result(index)?
                        .strategy
                        .as_ref()
                        .map(|trace| (index, trace))
                })),
                _ => match self.report() {
                    Ok(RunReport::Fit(report)) => {
                        Box::new(report.strategy.iter().map(|trace| (0, trace)))
                    }
                    #[cfg(feature = "solver-diffsol")]
                    Ok(RunReport::Shooting(report)) => {
                        Box::new(report.strategy.iter().map(|trace| (0, trace)))
                    }
                    _ => Box::new(std::iter::empty()),
                },
            };
        let failed = self
            .error()
            .and_then(|error| error.strategy_trace())
            .map(|trace| {
                (
                    self.modeling_failure
                        .as_ref()
                        .and_then(|failure| failure.step)
                        .unwrap_or(0),
                    trace,
                )
            });
        Box::new(retained.chain(failed))
    }
    /// Borrow retained compact native facts without cloning a request or constructing replay.
    pub(super) fn native_attempt_records(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            usize,
            Option<usize>,
            &Arc<pse_backend_native::solve::AttemptRecord>,
        ),
    > {
        self.replay_traces().flat_map(|(step, trace)| {
            trace
                .products
                .iter()
                .enumerate()
                .flat_map(move |(mechanism, product)| {
                    product
                        .native_attempt
                        .iter()
                        .map(move |record| (step, mechanism, None, record))
                        .chain(
                            product
                                .native_components
                                .iter()
                                .flat_map(|components| components.iter().enumerate())
                                .map(move |(component, record)| {
                                    (step, mechanism, Some(component), record)
                                }),
                        )
                })
        })
    }
    /// Retain the complete immutable input/decision closure, without numerical work or history lookup.
    pub fn replay(self: &Arc<Self>) -> ScientificReplay {
        ScientificReplay {
            source_run: self.run_id,
            result: self.clone(),
            coverage: self.recorded_coverage(),
        }
    }
    /// Shared retained factual coverage; construction confers no rerun admission.
    pub(super) fn recorded_coverage(&self) -> ReplayCoverage {
        let (requested, observed, completed_time) = match self.replay_request() {
            RunRequest::Modeling(steps) => {
                let mut count = 0;
                for index in 0..steps.len() {
                    if self.modeling_result(index).is_some() {
                        count += 1;
                    }
                }
                (steps.len(), count, None)
            }
            RunRequest::Simulation(simulation) => match self.report() {
                Ok(RunReport::Simulation(trajectory)) => (
                    simulation.profile().samples.len(),
                    trajectory.report().coverage.produced(),
                    Some(Observation::number(trajectory.report().completed_time)),
                ),
                _ => (simulation.profile().samples.len(), 0, None),
            },
            RunRequest::Fit(_) => (1, usize::from(self.report().is_ok()), None),
            RunRequest::Horizon(request) => (
                request.original.steps,
                self.horizon
                    .as_ref()
                    .map_or(0, |report| report.completed_steps),
                None,
            ),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { .. } => (1, usize::from(self.report().is_ok()), None),
        };
        let terminal = match self.report() {
            Err(error) => Some(crate::math::strategy::workflow_failure(error)),
            Ok(RunReport::Simulation(trajectory)) => {
                Some(trajectory_observation(trajectory.report()))
            }
            Ok(RunReport::Fit(report)) => report
                .validation_error
                .as_ref()
                .map(|diagnostic| crate::math::strategy::boundary_failure(diagnostic.class))
                .or_else(|| {
                    report
                        .solve
                        .as_ref()
                        .map(crate::math::strategy::observe_native)
                }),
            #[cfg(feature = "solver-diffsol")]
            Ok(RunReport::Shooting(report)) => report
                .validation_error
                .as_ref()
                .map(|diagnostic| crate::math::strategy::boundary_failure(diagnostic.class))
                .or_else(|| {
                    report
                        .trajectory
                        .as_ref()
                        .map(|trajectory| trajectory_observation(trajectory))
                })
                .or_else(|| {
                    report
                        .solve
                        .as_ref()
                        .map(crate::math::strategy::observe_native)
                }),
            Ok(RunReport::Modeling(results)) => results
                .last()
                .map(|result| crate::math::strategy::observe(&result.outcome)),
            Ok(RunReport::Horizon(report)) => report
                .summary_error()
                .map(crate::math::strategy::workflow_failure)
                .or_else(|| {
                    report
                        .detail_error()
                        .map(crate::math::strategy::workflow_failure)
                }),
        };
        ReplayCoverage {
            requested,
            observed,
            completed_time,
            usable: self.usable(),
            terminal,
        }
    }
}

/// Original case constructor retained before start resolution and derived numerical input creation.
#[derive(Debug)]
pub(crate) struct OriginalCaseInputs {
    specialization: Option<Arc<super::modeling::engagement::OriginalPreparationInputs>>,
    case: pse_compiler::workspace::ModelingCaseBindings,
    order: pse_kernels::DerivativeOrder,
    compiler: pse_compiler::workspace::Profile,
    solver: crate::math::solves::SolverProfile,
    numerical: crate::math::solves::NumericalInputs,
    overrides: super::modeling::cases::CaseOverrides,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalCaseInputs {
    #[expect(
        clippy::too_many_arguments,
        reason = "retain the existing actual constructor arguments without inventing another request language"
    )]
    pub(in crate::workflow) fn capture(
        source: &ModelingPackage,
        model: &crate::math::modeling::ModelingPreparation,
        case: &pse_compiler::workspace::ModelingCaseBindings,
        order: pse_kernels::DerivativeOrder,
        compiler: pse_compiler::workspace::Profile,
        solver: &crate::math::solves::SolverProfile,
        numerical: &crate::math::solves::NumericalInputs,
        overrides: &super::modeling::cases::CaseOverrides,
    ) -> Result<Arc<Self>, WorkflowError> {
        let bytes = size_of::<Self>()
            .saturating_add(512)
            .saturating_add(case.values.len().saturating_mul(192))
            .saturating_add(case.members.len().saturating_mul(192))
            .saturating_add(case.variables.len().saturating_mul(256))
            .saturating_add(
                case.values
                    .keys()
                    .chain(case.variables.keys())
                    .map(String::len)
                    .sum::<usize>(),
            )
            .saturating_add(
                encoded_extent(&(&numerical.declarations, &numerical.targets), EXPORT_MAX)?
                    .saturating_mul(8),
            )
            .saturating_add(profile_extent(solver)?)
            .saturating_add(
                numerical
                    .implicit
                    .values()
                    .map(|implicit| {
                        implicit
                            .unknowns
                            .len()
                            .saturating_mul(32)
                            .saturating_add(implicit.bounds.as_ref().map_or(0, |bounds| {
                                (bounds.lower.len() + bounds.upper.len()).saturating_mul(32)
                            }))
                            .saturating_add(match &implicit.selection {
                                pse_math::factorable::SelectedGraph::Unestablished { meaning } => {
                                    meaning.len()
                                }
                                _ => 0,
                            })
                            .saturating_add(256)
                    })
                    .sum::<usize>(),
            )
            .saturating_add(
                (overrides.seed.len() + overrides.parameters.len() + overrides.fixes.len())
                    .saturating_mul(192),
            );
        let owner = source
            .runtime
            .shared
            .math()
            .reserve("replay:original-case-constructor", bytes)?;
        Ok(Arc::new(Self {
            specialization: model.original.clone(),
            case: case.clone(),
            order,
            compiler,
            solver: solver.clone(),
            numerical: numerical.clone(),
            overrides: overrides.clone(),
            _owner: owner,
        }))
    }
    async fn reconstruct_in_process(
        &self,
        source: &ModelingPackage,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingSolvePreparation, WorkflowError> {
        let original = self
            .specialization
            .as_ref()
            .ok_or_else(|| contract("original specialization constructor was not retained"))?;
        source
            .prepare_solve_seed(
                original.root,
                original.instance,
                original.bindings.clone(),
                original.limits,
                self.case.clone(),
                self.order,
                self.compiler,
                self.solver.clone(),
                self.numerical.clone(),
                self.overrides.clone(),
                cancel,
            )
            .await
    }
    fn document(&self, source: usize) -> Result<ReplayCaseConstructor, WorkflowError> {
        // Gather retained facts independently of the executable replay predicate.
        let document = self.descriptive_document(source)?;
        if !self.numerical.implicit.is_empty() {
            return Err(contract(
                "portable case replay requires library-owned export of caller-supplied implicit PreparedBody capabilities; retained in-process rerun remains available",
            ));
        }
        Ok(document)
    }
    pub(super) fn opaque_descriptors(&self) -> Vec<String> {
        self.numerical
            .implicit
            .keys()
            .map(|key| format!("{key:?}"))
            .collect()
    }
    pub(super) fn descriptive_extent(&self) -> usize {
        self._owner
            .size()
            .saturating_add(
                self.specialization
                    .as_ref()
                    .map_or(0, |original| original.retained_bytes()),
            )
            .saturating_mul(8)
            .saturating_add(65536)
    }
    pub(super) fn descriptive_case_inputs(&self) -> ReplayCaseInputs {
        ReplayCaseInputs::new(&self.case)
    }
    pub(super) fn descriptive_document(
        &self,
        source: usize,
    ) -> Result<ReplayCaseConstructor, WorkflowError> {
        let original = self
            .specialization
            .as_ref()
            .ok_or_else(|| contract("original specialization constructor was not retained"))?;
        Ok(ReplayCaseConstructor {
            source,
            root: original.root,
            instance: original.instance,
            bindings: original.bindings.clone(),
            preparation: super::PreparationSettings {
                compiler: self.compiler,
                limits: original.limits,
            },
            case: ReplayCaseInputs::new(&self.case),
            order: derivative_order(self.order),
            settings: ReplaySolverSettings::from_profile(&self.solver),
            declarations: self.numerical.declarations.clone(),
            targets: self.numerical.targets.clone(),
            seed: bits(&self.overrides.seed),
            parameters: bits(&self.overrides.parameters),
            fixes: bits(&self.overrides.fixes),
            requested_start: None,
            requested_screened_start: None,
            prepared_graph: None,
        })
    }
}
fn warm_start_extent(seed: &pse_backend_native::solve::WarmStart) -> usize {
    use pse_backend_native::solve::WarmPayload as W;
    let cells = match &seed.payload {
        W::Root(values) => values.len(),
        W::Nlp {
            primal,
            bounds,
            rows,
            working,
            ..
        } => {
            #[cfg(feature = "solver-pounce")]
            let active = working.as_ref().map_or(0, |set| {
                set.active.bounds.len() + set.active.constraints.len()
            });
            #[cfg(not(feature = "solver-pounce"))]
            let active = {
                let _ = working;
                0
            };
            primal
                .len()
                .saturating_add(
                    bounds
                        .as_ref()
                        .map_or(0, |(a, b)| a.len().saturating_add(b.len())),
                )
                .saturating_add(rows.as_ref().map_or(0, Vec::len))
                .saturating_add(active)
        }
        W::Highs {
            primal,
            dual,
            basis,
        } => primal
            .as_ref()
            .map_or(0, Vec::len)
            .saturating_add(
                dual.as_ref()
                    .map_or(0, |(a, b)| a.len().saturating_add(b.len())),
            )
            .saturating_add(basis.as_ref().map_or(0, |basis| {
                basis.columns.len().saturating_add(basis.rows.len())
            })),
    };
    cells.saturating_mul(256).saturating_add(4096)
}
fn derivative_order(order: pse_kernels::DerivativeOrder) -> u8 {
    match order {
        pse_kernels::DerivativeOrder::Value => 0,
        pse_kernels::DerivativeOrder::First => 1,
        pse_kernels::DerivativeOrder::Second => 2,
    }
}
fn bits(values: &BTreeMap<SemanticId, f64>) -> BTreeMap<SemanticId, u64> {
    values
        .iter()
        .map(|(id, value)| (*id, value.to_bits()))
        .collect()
}
fn unbits(values: &BTreeMap<SemanticId, u64>) -> BTreeMap<SemanticId, f64> {
    values
        .iter()
        .map(|(id, value)| (*id, f64::from_bits(*value)))
        .collect()
}
/// Explicit structural bound override; infinity is preserved in its original IEEE bits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "bits", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum ReplayBoundOverride {
    /// Preserve the authored bound.
    Keep,
    /// Explicitly remove the authored endpoint.
    Remove,
    /// Exact original endpoint, including an explicit infinity.
    Value(u64),
}
impl ReplayBoundOverride {
    fn new(value: Option<Option<f64>>) -> Self {
        match value {
            None => Self::Keep,
            Some(None) => Self::Remove,
            Some(Some(value)) => Self::Value(value.to_bits()),
        }
    }
    fn restore(&self) -> Option<Option<f64>> {
        match self {
            Self::Keep => None,
            Self::Remove => Some(None),
            Self::Value(bits) => Some(Some(f64::from_bits(*bits))),
        }
    }
}
/// Complete original fixed/free and endpoint specification for one authored path.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayVariableState {
    /// Original fixed/free override.
    pub fixed: Option<bool>,
    /// Original lower endpoint override.
    pub lower: ReplayBoundOverride,
    /// Original upper endpoint override.
    pub upper: ReplayBoundOverride,
}
/// Exact original case input bits. These are constructor values, not observations or checkpoints.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayCaseInputs {
    /// Values supplied by authored path.
    pub values: BTreeMap<String, u64>,
    /// Values supplied by admitted semantic member identity.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub members: BTreeMap<SemanticId, u64>,
    /// All structural fixed/free and bound overrides.
    pub variables: BTreeMap<String, ReplayVariableState>,
}
impl ReplayCaseInputs {
    fn new(case: &pse_compiler::workspace::ModelingCaseBindings) -> Self {
        Self {
            values: case
                .values
                .iter()
                .map(|(path, value)| (path.clone(), value.to_bits()))
                .collect(),
            members: bits(&case.members),
            variables: case
                .variables
                .iter()
                .map(|(path, value)| {
                    (
                        path.clone(),
                        ReplayVariableState {
                            fixed: value.fixed,
                            lower: ReplayBoundOverride::new(value.lower),
                            upper: ReplayBoundOverride::new(value.upper),
                        },
                    )
                })
                .collect(),
        }
    }
    fn restore(&self) -> pse_compiler::workspace::ModelingCaseBindings {
        pse_compiler::workspace::ModelingCaseBindings {
            values: self
                .values
                .iter()
                .map(|(path, value)| (path.clone(), f64::from_bits(*value)))
                .collect(),
            members: unbits(&self.members),
            variables: self
                .variables
                .iter()
                .map(|(path, value)| {
                    (
                        path.clone(),
                        pse_compiler::workspace::ModelingVariableState {
                            fixed: value.fixed,
                            lower: value.lower.restore(),
                            upper: value.upper.restore(),
                        },
                    )
                })
                .collect(),
        }
    }
}
/// Exact producer required to recreate an opaque compiled product in another runtime.
#[derive(Clone, Copy, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplayProducerPremise {
    /// Original structural block schedule preparation.
    Blocks,
    /// Authored causal/recycle supplier preparation.
    Causal,
    /// Multistart specification proposal and fresh screening.
    Multistart,
    /// Exact derived request and original correspondence preparation.
    Derived,
    /// Fidelity/evaluator input and fresh model-management actor.
    Surrogate,
    /// Exact flow/block request and mass/domain supplier.
    Petsc,
    /// Exact path request, oriented origin and curvature premises.
    Path,
}
/// One ordered reference in the original immutable prepared graph.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayPreparedProduct {
    /// Postorder node for an original-only reusable scientific preparation.
    pub node: Option<usize>,
    /// Required fresh producer when the product is an opaque compiled capability.
    pub required_producer: Option<ReplayProducerPremise>,
    /// Exact original scientific binding identity, retained separately from capability.
    pub original: ContentHash,
    /// Exact effective product profile identity; a digest does not recreate the capability.
    pub profile: ContentHash,
}
/// One original input and its exact full requested policy, before any native dispatch.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayPreparedNode {
    /// Exact original scientific binding identity.
    pub original: ContentHash,
    /// Exact compiler artifacts, evaluator admission and composed preparation identity.
    pub preparation: ContentHash,
    /// Actual build/runtime observation of this profile's admission.
    pub runtime: pse_backend_native::execution::Snapshot,
    /// Full actual immutable scientific/native profile body.
    pub settings: ReplaySolverSettings,
    /// Exact effective profile identity, verified again after reconstruction.
    pub profile: ContentHash,
    /// Full requested explicit start before dispatch.
    pub requested_start: Option<pse_backend_native::solve::WarmStartSnapshot>,
    /// Full requested screened proposal; fresh screening is an independent premise.
    pub requested_screened_start: Option<ReplayScreenedStart>,
    /// Exact actual postconstructor ordering, starts, branch and limits.
    pub declaration: Option<NumericalStrategy>,
    /// Exact ordered automatic inventory prepared in advance, including every Original profile.
    pub automatic: Vec<ReplayPreparedProduct>,
    /// Exact ordered declared product inventory.
    pub rungs: Vec<ReplayPreparedProduct>,
}
/// Full bounded postorder inventory. Opaque capabilities are explicit producer premises.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayPreparedGraph {
    /// Actual original nodes, children before parents; no mutable preparation history lookup.
    pub nodes: Vec<ReplayPreparedNode>,
    /// Exact root original node.
    pub root: usize,
}
#[derive(Clone, Copy)]
struct ReplayGraphBound {
    depth: usize,
    occurrences: usize,
    bytes: usize,
}
impl ReplayPreparedGraph {
    fn reconstruction_extent(&self) -> Result<usize, WorkflowError> {
        let mut bounds: Vec<ReplayGraphBound> = Vec::with_capacity(self.nodes.len());
        let mut occurrences = 0usize;
        let mut bytes = self
            .nodes
            .len()
            .checked_mul(size_of::<super::ModelingSolvePreparation>())
            .ok_or_else(|| contract("replay original graph table extent overflow"))?;
        for (index, node) in self.nodes.iter().enumerate() {
            let mut bound = ReplayGraphBound {
                depth: 1,
                occurrences: 1,
                bytes: 0,
            };
            for product in node.automatic.iter().chain(&node.rungs) {
                if product.required_producer.is_some() {
                    return Err(contract(format!(
                        "portable prepared product requires fresh {:?} producer rebind",
                        product.required_producer
                    )));
                }
                let child = product.node.filter(|child| *child < index).ok_or_else(|| {
                    contract("replay original graph has a missing, cyclic or nonpostorder product")
                })?;
                let child = bounds[child];
                bound.depth = bound.depth.max(
                    child
                        .depth
                        .checked_add(1)
                        .ok_or_else(|| contract("replay original graph depth overflow"))?,
                );
                bound.occurrences = bound
                    .occurrences
                    .checked_add(child.occurrences)
                    .ok_or_else(|| {
                        contract("replay original graph occurrence expansion overflow")
                    })?;
            }
            occurrences = occurrences
                .checked_add(bound.occurrences)
                .filter(|total| *total <= 4096)
                .ok_or_else(|| {
                    contract("replay original graph exceeds 4096 expanded preparation occurrences")
                })?;
            if bound.depth > 64 {
                return Err(contract("replay original graph exceeds depth 64"));
            }
            bounds.push(bound);
        }
        // Reject structurally excessive input before traversing any owned profile body.
        for (index, node) in self.nodes.iter().enumerate() {
            let mut copied = encoded_extent(node, EXPORT_MAX)?
                .checked_mul(8)
                .and_then(|bytes| bytes.checked_add(65536))
                .ok_or_else(|| contract("replay original node copy extent overflow"))?;
            for product in node.automatic.iter().chain(&node.rungs) {
                let child = product
                    .node
                    .ok_or_else(|| contract("replay original child absent"))?;
                copied = copied
                    .checked_add(bounds[child].bytes)
                    .ok_or_else(|| contract("replay original graph copy expansion overflow"))?;
            }
            bounds[index].bytes = copied;
            bytes = bytes
                .checked_add(copied)
                .filter(|bytes| *bytes <= EXPORT_MAX)
                .ok_or_else(|| {
                    contract(
                        "replay original graph expanded copy allowance exceeds finite capacity",
                    )
                })?;
        }
        Ok(bytes)
    }
    fn extent(
        input: &crate::math::solves::PreparedSolve,
        depth: usize,
        count: &mut usize,
    ) -> Result<usize, WorkflowError> {
        *count += 1;
        if depth >= 64 || *count > 4096 {
            return Err(contract(
                "replay prepared graph exceeds finite depth/node allowance",
            ));
        }
        let mut bytes = profile_extent(input.replay_profile())?
            .saturating_add(
                encoded_extent(input.execution_snapshot(), EXPORT_MAX)?.saturating_mul(8),
            )
            .saturating_add(input.requested_start().map_or(0, warm_start_extent))
            .saturating_add(input.requested_screened_start().map_or(0, |start| {
                start.point().len().saturating_mul(256).saturating_add(2048)
            }))
            .saturating_add(
                encoded_extent(&input.replay_declaration(), EXPORT_MAX)?.saturating_mul(8),
            );
        for product in input
            .replay_automatic_products()
            .iter()
            .chain(input.replay_rungs().unwrap_or_default())
        {
            bytes = bytes.saturating_add(512);
            if let crate::math::solves::PreparedRung::Original(input) = product {
                bytes = bytes.saturating_add(Self::extent(input, depth + 1, count)?);
            }
        }
        Ok(bytes)
    }
    fn new(input: &crate::math::solves::PreparedSolve) -> Result<Self, WorkflowError> {
        let mut graph = Self {
            nodes: Vec::new(),
            root: 0,
        };
        graph.root = graph.push(input)?;
        Ok(graph)
    }
    fn product(
        &mut self,
        product: &crate::math::solves::PreparedRung,
    ) -> Result<ReplayPreparedProduct, WorkflowError> {
        use crate::math::solves::PreparedRung as R;
        let (node, required_producer) = match product {
            R::Original(input) => (Some(self.push(input)?), None),
            #[cfg(feature = "solver-kinsol")]
            R::Blocks(_) => (None, Some(ReplayProducerPremise::Blocks)),
            #[cfg(feature = "solver-kinsol")]
            R::Causal(_) => (None, Some(ReplayProducerPremise::Causal)),
            R::Multistart(_) => (None, Some(ReplayProducerPremise::Multistart)),
            R::Derived(_) => (None, Some(ReplayProducerPremise::Derived)),
            R::Surrogate(_) => (None, Some(ReplayProducerPremise::Surrogate)),
            #[cfg(feature = "solver-petsc")]
            R::Petsc(_) => (None, Some(ReplayProducerPremise::Petsc)),
            R::Path { .. } => (None, Some(ReplayProducerPremise::Path)),
        };
        Ok(ReplayPreparedProduct {
            node,
            required_producer,
            original: product
                .original_identity()
                .map_err(crate::math::MathRuntimeError::from)?,
            profile: product
                .strategy_profile()
                .map_err(crate::math::MathRuntimeError::from)?,
        })
    }
    fn push(&mut self, input: &crate::math::solves::PreparedSolve) -> Result<usize, WorkflowError> {
        let automatic = input
            .replay_automatic_products()
            .iter()
            .map(|product| self.product(product))
            .collect::<Result<Vec<_>, _>>()?;
        let rungs = input
            .replay_rungs()
            .unwrap_or_default()
            .iter()
            .map(|product| self.product(product))
            .collect::<Result<Vec<_>, _>>()?;
        let node = self.nodes.len();
        self.nodes.push(ReplayPreparedNode {
            original: input
                .original_identity()
                .map_err(crate::math::MathRuntimeError::from)?,
            preparation: input
                .preparation_identity()
                .map_err(crate::math::MathRuntimeError::from)?,
            runtime: input.execution_snapshot().clone(),
            settings: ReplaySolverSettings::from_profile(input.replay_profile()),
            profile: input
                .strategy_profile()
                .map_err(crate::math::MathRuntimeError::from)?,
            requested_start: input
                .requested_start()
                .map(pse_backend_native::solve::WarmStart::snapshot),
            requested_screened_start: input
                .requested_screened_start()
                .map(ReplayScreenedStart::new),
            declaration: input.replay_declaration().cloned(),
            automatic,
            rungs,
        });
        Ok(node)
    }
}
fn same_inventory(
    left: &[crate::math::solves::PreparedRung],
    right: &[crate::math::solves::PreparedRung],
) -> Result<bool, WorkflowError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        if std::mem::discriminant(left) != std::mem::discriminant(right)
            || left
                .original_identity()
                .map_err(crate::math::MathRuntimeError::from)?
                != right
                    .original_identity()
                    .map_err(crate::math::MathRuntimeError::from)?
            || left
                .strategy_profile()
                .map_err(crate::math::MathRuntimeError::from)?
                != right
                    .strategy_profile()
                    .map_err(crate::math::MathRuntimeError::from)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}
/// Complete original proposal input; historical screening is not renewed permission.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayScreenedStart {
    /// Full original source dependencies.
    pub source: pse_model::strategy::SemanticProductKey,
    /// Original bound target.
    pub target: ContentHash,
    /// Original scientific branch requirement.
    pub branch: pse_model::strategy::BranchPolicy,
    /// Actual input producer provenance.
    pub origin: StartOrigin,
    /// All physical source coordinates in exact IEEE bits.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub values: BTreeMap<SemanticId, u64>,
}
impl ReplayScreenedStart {
    pub(super) fn new(screened: &crate::math::prediction::Screened) -> Self {
        let proposal = screened.proposal();
        Self {
            source: proposal.source(),
            target: proposal.target(),
            branch: proposal.branch(),
            origin: proposal.origin(),
            values: proposal
                .values()
                .map(|(id, value)| (id, value.to_bits()))
                .collect(),
        }
    }
}
/// Portable original case constructor, independent of mutable history or compiled native products.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayCaseConstructor {
    /// Index of the immutable source closure in ReplayDocument.sources.
    pub source: usize,
    /// Original selected authored definition.
    pub root: pse_modeling::DeclarationId,
    /// Original instance identity.
    pub instance: pse_modeling::InstanceId,
    /// Full immutable structural arguments/facts/omissions/elastic declarations.
    #[serde(serialize_with = "serialize_bindings")]
    #[schemars(with = "Vec<u8>")]
    pub bindings: pse_modeling::Bindings,
    /// Actual compiler controls and original expansion limits.
    pub preparation: super::PreparationSettings,
    /// Exact supplied numeric values and structural variable specifications.
    pub case: ReplayCaseInputs,
    /// Requested derivative order.
    pub order: u8,
    /// Exact original requested scientific/native settings.
    pub settings: ReplaySolverSettings,
    /// Original user-supplied numerical declarations, before derived resolution.
    pub declarations: Vec<pse_math::numerics::SourcedRequirement>,
    /// Original user-supplied target definitions, before derived resolution.
    pub targets: Vec<pse_math::numerics::TargetSpec>,
    /// Explicit predecessor starts in exact original bits.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub seed: BTreeMap<SemanticId, u64>,
    /// Explicit continuation parameters in exact original bits.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub parameters: BTreeMap<SemanticId, u64>,
    /// Explicit temporary fixes in exact original bits.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub fixes: BTreeMap<SemanticId, u64>,
    /// Exact explicit input requested after constructor admission, even before dispatch.
    pub requested_start: Option<pse_backend_native::solve::WarmStartSnapshot>,
    /// Exact requested screened input; restoring requires fresh producer screening.
    pub requested_screened_start: Option<ReplayScreenedStart>,
    /// Full original-only profile graph; opaque products explicitly require producer rebind.
    pub prepared_graph: Option<ReplayPreparedGraph>,
}
impl ReplayCaseConstructor {
    /// Prepare the original typed request through its actual source/numerical admission pipeline.
    pub async fn reconstruct(
        &self,
        package: &ModelingPackage,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingSolvePreparation, WorkflowError> {
        let Some(graph) = &self.prepared_graph else {
            return self
                .reconstruct_body(
                    package,
                    &self.settings,
                    self.requested_start.as_ref(),
                    self.requested_screened_start.as_ref(),
                    cancel,
                )
                .await;
        };
        if graph.nodes.is_empty() || graph.nodes.len() > 4096 || graph.root != graph.nodes.len() - 1
        {
            return Err(contract(
                "replay original graph has an invalid root/node allowance",
            ));
        }
        let _shape_owner = package.runtime.shared.math().reserve(
            "replay:graph-structural-validation",
            graph
                .nodes
                .len()
                .checked_mul(size_of::<ReplayGraphBound>())
                .and_then(|bytes| bytes.checked_add(128))
                .ok_or_else(|| contract("replay graph validation extent overflow"))?,
        )?;
        let expanded_extent = graph.reconstruction_extent()?;
        for (index, node) in graph.nodes.iter().enumerate() {
            for product in node.automatic.iter().chain(&node.rungs) {
                if product.required_producer.is_some() {
                    return Err(contract(format!(
                        "portable prepared product requires fresh {:?} producer rebind; a profile/hash cannot reconstruct its opaque compiled capability",
                        product.required_producer
                    )));
                }
                if product.node.is_none_or(|child| child >= index) {
                    return Err(contract(
                        "replay original graph has a missing, cyclic or nonpostorder product",
                    ));
                }
            }
            if node
                .declaration
                .as_ref()
                .is_none_or(|declaration| declaration.mechanisms.len() != node.rungs.len())
                && !node.rungs.is_empty()
            {
                return Err(contract(
                    "replay declaration does not bind its complete ordered products",
                ));
            }
        }
        let owner = package
            .runtime
            .shared
            .math()
            .reserve("replay:original-graph-reconstruction", expanded_extent)?;
        let mut built: Vec<super::ModelingSolvePreparation> = Vec::with_capacity(graph.nodes.len());
        for node in &graph.nodes {
            let mut prepared = self
                .reconstruct_body(
                    package,
                    &node.settings,
                    node.requested_start.as_ref(),
                    node.requested_screened_start.as_ref(),
                    cancel,
                )
                .await?;
            let products = |references: &[ReplayPreparedProduct]| -> Result<Vec<crate::math::solves::PreparedRung>, WorkflowError> {
                references.iter().map(|product| {
                    let input = &built[product.node.ok_or_else(|| contract("missing replay original node"))?].solve;
                    if input.original_identity().map_err(crate::math::MathRuntimeError::from)? != product.original
                        || input.strategy_profile().map_err(crate::math::MathRuntimeError::from)? != product.profile {
                        return Err(contract("reconstructed original product identity/profile differs from its exact retained input"));
                    }
                    let extent = input.replay_metadata_extent().map_err(crate::math::MathRuntimeError::from)?
                        .checked_add(ReplayPreparedGraph::extent(input, 0, &mut 0)?)
                        .filter(|bytes| *bytes <= EXPORT_MAX).ok_or_else(|| contract("replay original metadata copy exceeds finite capacity"))?;
                    let owner = package.runtime.shared.math().reserve("replay:retained-original-metadata-copy", extent)?;
                    Ok(input.clone().retaining_replay_metadata(owner).into())
                }).collect()
            };
            if !node.automatic.is_empty() {
                prepared.solve = prepared
                    .solve
                    .with_automatic_products(products(&node.automatic)?)
                    .map_err(crate::math::MathRuntimeError::from)?;
            }
            if let Some(declaration) = &node.declaration {
                prepared.solve = prepared
                    .solve
                    .with_strategy(declaration.clone(), products(&node.rungs)?)
                    .map_err(crate::math::MathRuntimeError::from)?;
            }
            if prepared
                .solve
                .preparation_identity()
                .map_err(crate::math::MathRuntimeError::from)?
                != node.preparation
            {
                return Err(contract(
                    "exact replay preparation differs; the recorded leaf compiler/evaluator/artifact producer must rebind its original constructor",
                ));
            }
            if prepared.solve.execution_snapshot() != &node.runtime {
                return Err(contract(
                    "reconstructed original profile requires its exact recorded build/runtime admission premises",
                ));
            }
            if prepared
                .solve
                .original_identity()
                .map_err(crate::math::MathRuntimeError::from)?
                != node.original
                || prepared
                    .solve
                    .strategy_profile()
                    .map_err(crate::math::MathRuntimeError::from)?
                    != node.profile
            {
                return Err(contract(
                    "reconstructed exact original profile differs; effective operational precision requires its producer rebind",
                ));
            }
            built.push(prepared);
        }
        // All node bodies own their admitted preparation. The scratch lease covers only
        // this temporary postorder table and its bounded reconstruction copies.
        let result = built
            .pop()
            .ok_or_else(|| contract("replay original root absent"))?;
        drop(owner);
        Ok(result)
    }
    async fn reconstruct_body(
        &self,
        package: &ModelingPackage,
        settings: &ReplaySolverSettings,
        start: Option<&pse_backend_native::solve::WarmStartSnapshot>,
        screened: Option<&ReplayScreenedStart>,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingSolvePreparation, WorkflowError> {
        let order = match self.order {
            0 => pse_kernels::DerivativeOrder::Value,
            1 => pse_kernels::DerivativeOrder::First,
            2 => pse_kernels::DerivativeOrder::Second,
            _ => {
                return Err(contract(
                    "replay derivative order outside actual library contract",
                ));
            }
        };
        if screened.is_some() {
            return Err(contract(
                "portable requested screened input requires fresh original-owner screening; historical screening is not reusable permission",
            ));
        }
        let start_owner = start
            .map(|start| {
                let bytes = start
                    .restore_extent()
                    .map_err(crate::math::MathRuntimeError::from)?;
                package
                    .runtime
                    .shared
                    .math()
                    .reserve("replay:restored-requested-start", bytes)
                    .map_err(WorkflowError::from)
            })
            .transpose()?;
        let mut prepared = package
            .prepare_solve_seed(
                self.root,
                self.instance,
                self.bindings.clone(),
                self.preparation.limits,
                self.case.restore(),
                order,
                self.preparation.compiler,
                settings.profile()?,
                crate::math::solves::NumericalInputs {
                    declarations: self.declarations.clone(),
                    targets: self.targets.clone(),
                    implicit: BTreeMap::new(),
                },
                super::modeling::cases::CaseOverrides {
                    seed: unbits(&self.seed),
                    parameters: unbits(&self.parameters),
                    fixes: unbits(&self.fixes),
                },
                cancel,
            )
            .await?;
        if let (Some(start), Some(owner)) = (start, start_owner) {
            prepared.solve = prepared
                .solve
                .with_start_owned(
                    start
                        .restore()
                        .map_err(crate::math::MathRuntimeError::from)?,
                    owner,
                )
                .map_err(crate::math::MathRuntimeError::from)?;
            prepared.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        }
        Ok(prepared)
    }
}

impl ReplaySolverSettings {
    /// Re-admit the frozen original settings using their existing production contract.
    pub fn profile(&self) -> Result<crate::math::solves::SolverProfile, WorkflowError> {
        let mut presolve_options = self.presolve_options.clone();
        let certificate_tolerance = presolve_options.remove("certify_tol");
        let mut profile = crate::math::settings::SolveSettings {
            block_overrides: self.block_overrides.clone(),
            block_workers: self.block_workers,
            version: pse_model::document::Version,
            composition: self.composition.clone(),
            reconstruction: self.reconstruction,
            intent: self.intent,
            backend: self.backend,
            settings: self.settings.clone(),
            presolve: self.presolve,
            presolve_options,
            required_passes: self.required_passes.clone(),
            controls: self.controls.clone(),
            numerics: self.numerics.clone(),
            convexity_absolute: self.convexity_absolute,
            convexity_relative: self.convexity_relative,
            sensitivity: self.sensitivity.clone(),
        }
        .profile()?;
        if let Some(value) = certificate_tolerance {
            match (&mut profile.presolve, value) {
                (
                    pse_backend_native::presolve::Policy::Explicit { options, .. },
                    pse_backend_native::solve::OptionValue::Real(value),
                ) if value.is_finite() && value >= 0.0 => options.certify_tol = value,
                _ => return Err(contract("invalid actual presolve certification tolerance")),
            }
        }
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn replay_bindings_transport_preserves_modeling_owner_codec() {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            #[serde(serialize_with = "super::serialize_bindings")]
            bindings: &'a pse_modeling::Bindings,
        }
        let mut bindings = pse_modeling::Bindings::default();
        bindings.arguments.insert(
            "enabled".into(),
            pse_modeling::specialize::Value::Boolean(true),
        );
        bindings
            .arguments
            .insert("count".into(), pse_modeling::specialize::Value::Integer(7));
        bindings.demand.push("actual.path".into());
        let exported = serde_json::to_value(Body {
            bindings: &bindings,
        })
        .unwrap();
        let bytes: Vec<u8> = serde_json::from_value(exported["bindings"].clone()).unwrap();
        assert_eq!(bytes, serde_json::to_vec(&bindings).unwrap());
        let restored: pse_modeling::Bindings = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, bindings);
    }
    use super::*;
    fn rows() -> Vec<pse_authoring::language::Declaration> {
        pse_authoring::language::parse(
            "package p {def Root {param a:Scalar=1;var x:Scalar;eq e:x==a;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    #[tokio::test]
    async fn retained_replay_positions_bind_exact_owners_once_and_keep_receiver_allocations() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let a = runtime
            .modeling_package(rows(), physical.clone())
            .await
            .unwrap();
        let b = runtime
            .modeling_package(rows(), physical.clone())
            .await
            .unwrap();
        assert_eq!(a.revision.identity(), b.revision.identity());
        let positions = [&a, &a, &b, &a];
        let mut conversions = 0;
        let wire = map_retained_sources(positions.into_iter(), |source| {
            conversions += 1;
            source.replay_source()
        })
        .unwrap();
        assert_eq!(
            conversions, 2,
            "same scientific hash cannot merge independent admitted owners"
        );
        let alternate = a
            .clone()
            .with_accelerators(pse_math::implicit::accelerators::Accelerators::standard());
        assert!(!a.same_replay_source(&alternate));
        let mut distinct_conversions = 0;
        map_retained_sources([&a, &alternate].into_iter(), |source| {
            distinct_conversions += 1;
            source.replay_source()
        })
        .unwrap();
        assert_eq!(
            distinct_conversions, 2,
            "an explicit accelerator owner cannot be replaced by another position"
        );
        assert_eq!(wire.len(), 4);
        assert_eq!(
            serde_json::to_value(&wire[0]).unwrap(),
            serde_json::to_value(&wire[3]).unwrap()
        );
        let receiver =
            Runtime::from_shared_local(runtime.shared.clone(), runtime.model.clone()).unwrap();
        let result = rebind_retained_sources(positions.into_iter(), &receiver).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].revision_id(), result[1].revision_id());
        assert_eq!(result[0].revision_id(), result[3].revision_id());
        assert_ne!(result[0].revision_id(), result[2].revision_id());
        assert_ne!(result[0].revision_id(), a.revision_id());
        let again = rebind_retained_sources(positions.into_iter(), &receiver).unwrap();
        assert_ne!(result[0].revision_id(), again[0].revision_id());
        let retained = Arc::downgrade(result[0].replay_owner.as_ref().unwrap());
        let borrower = result[3].clone();
        drop(result);
        assert!(retained.upgrade().is_some());
        drop(borrower);
        assert!(retained.upgrade().is_none());
        receiver.close();
        assert!(matches!(
            rebind_retained_sources(positions.into_iter(), &receiver),
            Err(WorkflowError::CallerClosed)
        ));
    }
    #[tokio::test]
    async fn supplied_replay_refuses_closed_receiver_and_duplicate_identity() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let source = runtime
            .modeling_package(rows(), physical.clone())
            .await
            .unwrap();
        let mut document = source.replay_source().unwrap();
        document.objects.push(document.objects[0].clone());
        assert!(
            document
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .unwrap_err()
                .to_string()
                .contains("duplicate logical or lexical identity")
        );
        runtime.close();
        assert!(matches!(
            document.reconstruct(&runtime, physical, BTreeMap::new()),
            Err(WorkflowError::CallerClosed)
        ));
    }
    #[tokio::test]
    async fn source_document_roundtrip_reconstructs_actual_production_package_without_store() {
        let runtime = crate::workflow::tests::runtime();
        let rows = rows();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let physical = crate::workflow::tests::physical();
        let package = runtime
            .modeling_package(rows.clone(), physical.clone())
            .await
            .unwrap();
        let document = package.replay_source().unwrap();
        let restored: ReplaySourceDocument =
            serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        let reconstructed = restored
            .reconstruct(&runtime, physical, BTreeMap::new())
            .unwrap();
        assert_eq!(
            &*reconstructed.declarations().await.unwrap(),
            rows.as_slice()
        );
        assert_ne!(reconstructed.revision_id(), package.revision_id());
        let mut bindings = pse_modeling::Bindings::default();
        bindings.demand.push("a".into());
        let model = reconstructed
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                bindings,
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(model.solved().model(), root);
        assert!(model.original.is_some());
    }
    #[tokio::test]
    async fn source_reconstruction_refuses_missing_reference_changed_body_and_build() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let package = runtime
            .modeling_package(rows(), physical.clone())
            .await
            .unwrap();
        let original = package.replay_source().unwrap();
        let mut document = original.clone();
        document.build = pse_ids::encoding_checksum(b"different build").content_hash();
        assert!(
            document
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .is_err()
        );
        let mut document = original.clone();
        document.objects[0].payload.push(0);
        assert!(
            document
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .is_err()
        );
        let mut document = original.clone();
        let edge_owner = document
            .objects
            .iter_mut()
            .find(|object| !object.references.is_empty())
            .unwrap();
        edge_owner.references.push(edge_owner.references[0].clone());
        assert!(
            document
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .is_err()
        );
        let mut document = original.clone();
        document.providers.push(ReplayProvider {
            name: "required-executable-provider".into(),
            implementation: SemanticId::NIL,
            revision: document.identity,
            data: document.identity,
            configuration: document.identity,
            inputs: vec![],
            outputs: vec![],
            input_shapes: vec![],
            output_shapes: vec![],
            derivative_source: ExternalDerivativeSource::Supplied,
            derivatives: 0,
            smoothness: 0,
        });
        assert!(
            document
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .is_err()
        );
        let mut document = original.clone();
        document.physical_identity =
            pse_ids::encoding_checksum(b"missing physical premise").content_hash();
        assert!(
            document
                .reconstruct(&runtime, physical.clone(), BTreeMap::new())
                .is_err()
        );
        let mut document = original;
        let referenced = document
            .objects
            .iter()
            .find_map(|object| object.references.first())
            .unwrap()
            .clone();
        document
            .objects
            .retain(|object| (object.scope.clone(), object.name.clone()) != referenced);
        assert!(
            document
                .reconstruct(&runtime, physical, BTreeMap::new())
                .is_err()
        );
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn plan38b_requested_seed_survives_cancellation_before_dispatch_and_portable_reconstruction()
     {
        use pse_backend_native::solve::{HessianMode, WarmPayload, WarmStart};
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse("package p {def Root {var x:Scalar; annotation bounds x(0,2); annotation start x(1); let f:Scalar=x; annotation objective f(minimize); eq floor:x>=0.5;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::First,
                crate::workflow::tests::compiler_profile(),
                crate::math::solves::SolverProfile {
                    intent: SolveIntent::Optimize,
                    selection: SolverSelection::Explicit(Backend::Ipopt),
                    controls: Controls {
                        hessian: HessianMode::LimitedMemory,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let seed = WarmStart {
            origin: None,
            compatibility: prepared.solve.compatibility().unwrap().clone(),
            payload: WarmPayload::primal(vec![1.25]),
        };
        let identity = seed.content_key();
        let mut prepared = prepared.with_start(seed).unwrap();
        let mut alternate_input = prepared.original_inputs.document(0).unwrap();
        alternate_input.settings.controls.iterations = 17;
        let alternate = alternate_input
            .reconstruct(&package, &crate::CancelSource::new())
            .await
            .unwrap()
            .with_start(prepared.solve.requested_start().unwrap().clone())
            .unwrap();
        let rungs = vec![prepared.solve.clone().into(), alternate.solve.into()];
        let mut declaration = prepared.solve.numerical_strategy();
        declaration.limits.attempts = 2;
        declaration.mechanisms = rungs
            .iter()
            .map(|rung: &crate::math::solves::PreparedRung| {
                let mut mechanism = declaration.mechanisms[0].clone();
                mechanism.limits = declaration.limits;
                mechanism.operation = rung.operation_contract().unwrap();
                mechanism.profile = Some(pse_model::strategy::ProfileRef {
                    method: pse_model::strategy::NumericalMethod::Backend(rung.backend().unwrap()),
                    key: rung.strategy_profile().unwrap(),
                });
                mechanism
            })
            .collect();
        prepared.solve = prepared
            .solve
            .with_strategy(declaration.clone(), rungs)
            .unwrap();
        let permits = runtime.shared.compiler_cpu();
        let width = permits.available_permits();
        assert!(width > 0);
        let held = permits.acquire_many_owned(width as u32).await.unwrap();
        let parent = crate::CancelSource::new();
        let handle = runtime
            .start_modeling(vec![prepared], true, &parent)
            .await
            .unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), handle.wait())
                .await
                .is_err()
        );
        parent.cancel();
        let stopped = tokio::time::timeout(std::time::Duration::from_secs(2), handle.wait())
            .await
            .unwrap()
            .unwrap();
        let replay = stopped.replay();
        let document = replay.document_with_limit(128 << 20).unwrap();
        assert_eq!(serde_json::to_value(&document).unwrap()["version"], 3);
        assert!(document.continue_independent);
        assert!(
            document.native_attempts.is_empty(),
            "no native report was dispatched while all CPU permits were held"
        );
        let constructor = &document.case_constructors[0];
        let graph = constructor.prepared_graph.as_ref().unwrap();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(
            graph.nodes[graph.root].declaration.as_ref(),
            Some(&declaration)
        );
        assert_eq!(graph.nodes[1].settings.controls.iterations, 17);
        assert_eq!(
            constructor
                .requested_start
                .as_ref()
                .unwrap()
                .restore()
                .unwrap()
                .content_key(),
            identity
        );
        let mut malformed = constructor.clone();
        malformed.prepared_graph.as_mut().unwrap().root = usize::MAX;
        assert!(
            malformed
                .reconstruct(&package, &crate::CancelSource::new())
                .await
                .is_err()
        );
        let mut missing = constructor.clone();
        let missing_graph = missing.prepared_graph.as_mut().unwrap();
        missing_graph.nodes[missing_graph.root].rungs[0].required_producer =
            Some(ReplayProducerPremise::Derived);
        assert!(
            missing
                .reconstruct(&package, &crate::CancelSource::new())
                .await
                .is_err()
        );
        for repeated in [false, true] {
            let mut excessive = constructor.clone();
            let excessive_graph = excessive.prepared_graph.as_mut().unwrap();
            let mut leaf = excessive_graph.nodes[0].clone();
            leaf.automatic.clear();
            leaf.rungs.clear();
            leaf.declaration = None;
            let template = graph.nodes[graph.root].rungs[0].clone();
            excessive_graph.nodes.clear();
            let length = if repeated { 20 } else { 65 };
            for index in 0..length {
                let mut node = leaf.clone();
                if index > 0 {
                    let mut product = template.clone();
                    product.node = Some(index - 1);
                    node.automatic.push(product.clone());
                    if repeated {
                        node.automatic.push(product);
                    }
                }
                excessive_graph.nodes.push(node);
            }
            excessive_graph.root = length - 1;
            let before = runtime.native().preparation_effects();
            let refusal = excessive
                .reconstruct(&package, &crate::CancelSource::new())
                .await
                .unwrap_err();
            let effects = runtime.native().preparation_effects().since(before);
            assert_eq!(
                (effects.workspaces, effects.frontiers, effects.completions),
                (0, 0, 0)
            );
            assert!(
                refusal.to_string().contains(if repeated {
                    "expanded preparation occurrences"
                } else {
                    "depth 64"
                }),
                "{refusal}"
            );
        }
        drop(held);
        let sources = replay.reconstruct_sources(&runtime).unwrap();
        let mut alternate_compiler = alternate_input.clone();
        alternate_compiler
            .preparation
            .compiler
            .optimization
            .horner_iterations += 1;
        let different = alternate_compiler
            .reconstruct(&sources[0], &crate::CancelSource::new())
            .await
            .unwrap()
            .with_start(
                constructor
                    .requested_start
                    .as_ref()
                    .unwrap()
                    .restore()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            different.solve.strategy_profile().unwrap(),
            graph.nodes[1].profile
        );
        assert_eq!(
            different.solve.original_identity().unwrap(),
            graph.nodes[1].original
        );
        assert_ne!(
            different.solve.preparation_identity().unwrap(),
            graph.nodes[1].preparation
        );
        let mut wrong_compiler = constructor.clone();
        wrong_compiler.prepared_graph.as_mut().unwrap().nodes[1].preparation =
            different.solve.preparation_identity().unwrap();
        let refusal = wrong_compiler
            .reconstruct(&sources[0], &crate::CancelSource::new())
            .await
            .unwrap_err();
        assert!(
            refusal
                .to_string()
                .contains("compiler/evaluator/artifact producer"),
            "{refusal}"
        );
        let restored = constructor
            .reconstruct(&sources[0], &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            restored.solve.requested_start().unwrap().content_key(),
            identity
        );
        assert_eq!(restored.solve.replay_declaration(), Some(&declaration));
        assert_eq!(
            restored.solve.replay_rungs().unwrap()[1]
                .strategy_profile()
                .unwrap(),
            constructor.prepared_graph.as_ref().unwrap().nodes[1].profile
        );
        let fresh = replay.rerun().await.unwrap().wait().await.unwrap();
        assert_ne!(fresh.run_id, stopped.run_id);
        assert!(fresh.report().is_ok(), "{:?}", fresh.report());
        let exported = fresh.replay().document_with_limit(128 << 20).unwrap();
        assert!(exported.continue_independent);
        let summary = fresh.attempt_summary().unwrap();
        assert_eq!(
            summary.native_attempts.len(),
            exported.native_attempts.len()
        );
        for (summary, replay) in summary
            .native_attempts
            .iter()
            .zip(&exported.native_attempts)
        {
            assert!(Arc::ptr_eq(&summary.record, &replay.record.document()));
        }
        let omitted = fresh
            .attempt_summary_with_limits(crate::workflow::attempt_documents::AttemptSummaryLimits {
                steps: 1,
                events: 0,
                findings: 0,
            })
            .unwrap();
        assert!(omitted.native_attempts.is_empty());
        assert_eq!(
            omitted.omitted_native_attempts,
            exported.native_attempts.len()
        );
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn plan38b_fit_replay_reconstructs_original_constructor_and_new_allowance() {
        use crate::workflow::fitting::regression;
        let package = regression::package(regression::declared(), [1.; 4]).await;
        let profile =
            regression::profile(pse_backend_native::solve::HessianMode::LimitedMemory, None);
        let original = regression::fit(&package, profile.clone()).await;
        assert!(original.report().is_ok());
        let replay = original.replay();
        let document = replay.document_with_limit(128 << 20).unwrap();
        let permits = package.runtime.shared.compiler_cpu();
        let width = permits.available_permits();
        assert!(width > 0);
        let held = permits.acquire_many_owned(width as u32).await.unwrap();
        let parent = crate::CancelSource::new();
        let pending = replay.rerun_with_cancel(&parent).await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), pending.wait())
                .await
                .is_err()
        );
        parent.cancel();
        let stopped = tokio::time::timeout(std::time::Duration::from_secs(2), pending.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(stopped.report().is_err());
        assert_eq!(
            stopped
                .replay()
                .document_with_limit(128 << 20)
                .unwrap()
                .fit_constructor
                .as_ref()
                .unwrap()
                .max_cells,
            profile.max_cells
        );
        drop(held);
        let constructor = document.fit_constructor.as_ref().unwrap();
        assert_eq!(constructor.max_cells, profile.max_cells);
        assert_eq!(constructor.derivatives, profile.derivatives);
        assert_eq!(
            constructor.rank_tolerance.to_bits(),
            profile.rank_tolerance.to_bits()
        );
        assert_eq!(
            crate::math::solves::profile_key(&constructor.solver.profile().unwrap()).unwrap(),
            crate::math::solves::profile_key(&profile.solver).unwrap()
        );
        let sources = replay.reconstruct_sources(&package.runtime).unwrap();
        let reconstructed = constructor
            .reconstruct(&sources[0], &crate::CancelSource::new())
            .await
            .unwrap();
        let restored = reconstructed.start().unwrap().wait().await.unwrap();
        assert!(restored.report().is_ok());
        let fresh = replay.rerun().await.unwrap().wait().await.unwrap();
        assert_ne!(fresh.run_id, original.run_id);
        assert!(fresh.report().is_ok(), "{:?}", fresh.report());
        assert!(
            !fresh
                .replay()
                .document_with_limit(128 << 20)
                .unwrap()
                .native_attempts
                .is_empty()
        );
    }
    #[tokio::test]
    async fn original_simulation_capture_refuses_long_paths_and_large_sets_before_copy() {
        let runtime = crate::workflow::tests::runtime_on(
            16 << 20,
            crate::math::MathPolicy {
                worker_bytes: 1 << 20,
                workspace_bytes: 1 << 20,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let rows = rows();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let pool = runtime.shared.pool();
        let remaining = runtime.shared.budget().memory_limit_bytes.get() - pool.reserved();
        let held = runtime
            .shared
            .math()
            .reserve("test:replay-input-headroom", remaining - (128 << 10))
            .unwrap();
        let baseline = pool.reserved();
        let mut case = pse_compiler::workspace::ModelingCaseBindings::default();
        case.values.insert("x".repeat(1 << 20), 1.0);
        let capture = |case: &pse_compiler::workspace::ModelingCaseBindings,
                       exact: &BTreeSet<SemanticId>,
                       quadratures: &BTreeSet<SemanticId>| {
            OriginalSimulationInputs::capture(
                &package,
                root,
                pse_modeling::specialize::root_instance(root),
                &Default::default(),
                Default::default(),
                case,
                crate::workflow::tests::compiler_profile(),
                &Default::default(),
                pse_kernels::DerivativeOrder::First,
                exact,
                None,
                quadratures,
                None,
            )
        };
        assert!(capture(&case, &BTreeSet::new(), &BTreeSet::new()).is_err());
        assert_eq!(pool.reserved(), baseline);
        let demand = (0u128..4096)
            .map(|id| SemanticId::from_bytes(id.to_le_bytes()))
            .collect();
        assert!(capture(&Default::default(), &demand, &demand).is_err());
        assert_eq!(pool.reserved(), baseline);
        drop(held);
    }
    #[test]
    fn original_case_bits_preserve_removed_bounds_infinity_and_negative_zero() {
        let mut case = pse_compiler::workspace::ModelingCaseBindings::default();
        case.values.insert("x".into(), -0.0);
        case.variables.insert(
            "x".into(),
            pse_compiler::workspace::ModelingVariableState {
                fixed: Some(false),
                lower: Some(None),
                upper: Some(Some(f64::INFINITY)),
            },
        );
        let document = ReplayCaseInputs::new(&case);
        let restored: ReplayCaseInputs =
            serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        let restored = restored.restore();
        assert_eq!(restored.values["x"].to_bits(), (-0.0_f64).to_bits());
        assert_eq!(restored.variables["x"].lower, Some(None));
        assert_eq!(restored.variables["x"].upper, Some(Some(f64::INFINITY)));
    }
    #[test]
    fn full_profile_and_typed_structural_facts_survive_recorded_input_reconstruction() {
        let mut profile = crate::math::solves::SolverProfile::default();
        profile.controls.iterations = 17;
        profile.controls.options.insert(
            "actual_option".into(),
            pse_backend_native::solve::OptionValue::Integer(4),
        );
        let document = ReplaySolverSettings::from_profile(&profile);
        assert!(document.settings.is_none());
        let bytes = serde_json::to_vec(&document).unwrap();
        let document: ReplaySolverSettings = serde_json::from_slice(&bytes).unwrap();
        let restored = document.profile().unwrap();
        assert_eq!(
            crate::math::solves::profile_key(&profile).unwrap(),
            crate::math::solves::profile_key(&restored).unwrap()
        );
        let mut bindings = pse_modeling::Bindings::default();
        bindings.facts.insert(
            pse_modeling::analysis::Fact::Stage("stage1".into()),
            pse_modeling::specialize::Value::Boolean(true),
        );
        let restored: pse_modeling::Bindings =
            serde_json::from_slice(&serde_json::to_vec(&bindings).unwrap()).unwrap();
        assert_eq!(bindings, restored);
    }
    #[tokio::test]
    async fn review_context_failed_run_retains_exact_inputs_starts_settings_and_unavailable_products()
     {
        use crate::workflow::ReviewContextRequest;
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p {def Root {param a:Scalar=1.25;var x:Scalar;eq e:x==a;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let mut solver = crate::workflow::tests::profile();
        solver.controls.iterations = 17;
        let mut prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                pse_compiler::workspace::ModelingCaseBindings {
                    values: BTreeMap::from([("x".into(), 1.25)]),
                    variables: BTreeMap::from([(
                        "x".into(),
                        pse_compiler::workspace::ModelingVariableState {
                            fixed: Some(true),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
                pse_kernels::DerivativeOrder::First,
                crate::workflow::tests::compiler_profile(),
                solver,
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let opaque_body = prepared
            .model
            .model
            .compiled()
            .admitted
            .bodies
            .values()
            .next()
            .unwrap()
            .math()
            .clone();
        let original = Arc::get_mut(&mut prepared.original_inputs).unwrap();
        original.numerical.implicit.insert(
            pse_kernels::ProviderKey(ContentHash::from_bytes([41; 32])),
            pse_math::factorable::ImplicitDefinition {
                residual: opaque_body,
                unknowns: vec![(f64::NEG_INFINITY, f64::INFINITY)],
                bounds: None,
                selection: pse_math::factorable::SelectedGraph::Unestablished {
                    meaning: "caller supplied opaque implicit body".into(),
                },
            },
        );
        assert!(
            original
                .document(0)
                .unwrap_err()
                .to_string()
                .contains("caller-supplied implicit")
        );
        assert!(original.descriptive_document(0).is_ok());
        let expected_inputs = prepared
            .model
            .values
            .scalars
            .iter()
            .map(|(id, value)| (*id, value.to_bits()))
            .collect::<BTreeMap<_, _>>();
        let expected_starts = serde_json::to_vec(&prepared.starts).unwrap();
        let result = Arc::new(RunResult::joined(
            RunId::from_bytes([91; 16]),
            runtime,
            crate::workflow::RunOutcome::Modeling {
                request: vec![prepared],
                result: Err(Arc::new(contract("failed before dispatch"))),
            },
            None,
        ));
        assert!(
            result
                .replay()
                .document()
                .unwrap_err()
                .to_string()
                .contains("caller-supplied implicit")
        );
        let (context, counts) =
            crate::math::counted(async { result.review_context(&ReviewContextRequest::default()) })
                .await;
        let context = context.unwrap();
        let document = context.document();
        assert!(
            document
                .unavailable
                .iter()
                .any(|item| item.path == "/steps/0/implicit")
        );
        assert_eq!(counts, crate::math::PreparationCounts::default());
        assert_eq!(document.solver_settings[0].controls.iterations, 17);
        let fact = |path: &str| {
            document
                .facts
                .iter()
                .find(|fact| fact.path == path)
                .unwrap()
        };
        let inputs: BTreeMap<SemanticId, u64> =
            serde_json::from_slice(&fact("/steps/0/physical_values").payload).unwrap();
        assert_eq!(inputs, expected_inputs);
        assert_eq!(fact("/steps/0/starts").payload, expected_starts);
        let submitted: ReplayCaseInputs =
            serde_json::from_slice(&fact("/steps/0/submitted_case").payload).unwrap();
        assert_eq!(submitted.values["x"], 1.25_f64.to_bits());
        assert!(!document.coverage.as_ref().unwrap().usable);
        let invocation: serde_json::Value =
            serde_json::from_slice(&fact("/invocation").payload).unwrap();
        assert_eq!(invocation["products"][0]["availability"], "unavailable");
        document.validate().unwrap();
    }
}

/// One complete original integrated constructor retained before mode preparation.
#[derive(Debug)]
pub(crate) struct OriginalSimulationInputs {
    document: ReplaySimulationConstructor,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalSimulationInputs {
    #[expect(
        clippy::too_many_arguments,
        reason = "retain the actual integrated constructor without changing its scientific request contract"
    )]
    pub(in crate::workflow) fn capture(
        package: &ModelingPackage,
        root: pse_modeling::DeclarationId,
        instance: pse_modeling::InstanceId,
        bindings: &pse_modeling::Bindings,
        limits: pse_modeling::Limits,
        case: &pse_compiler::workspace::ModelingCaseBindings,
        compiler: pse_compiler::workspace::Profile,
        profile: &super::SimulationProfile,
        order: pse_kernels::DerivativeOrder,
        exact_parameters: &BTreeSet<SemanticId>,
        forward_parameters: Option<&BTreeSet<SemanticId>>,
        provisional_quadratures: &BTreeSet<SemanticId>,
        binding: Option<&super::AdmittedBinding>,
    ) -> Result<Arc<Self>, WorkflowError> {
        if forward_parameters.is_some() && profile.forward_axes.is_some() {
            return Err(contract(
                "dynamic constructor has competing forward selections",
            ));
        }
        let binding_bytes = encoded_extent(&binding, EXPORT_MAX)?.saturating_mul(8);
        let bytes = size_of::<Self>()
            .saturating_add(binding_bytes)
            .saturating_add(1024)
            .saturating_add(crate::math::modeling::bindings_bytes(bindings))
            .saturating_add(encoded_extent(profile, EXPORT_MAX)?.saturating_mul(8))
            .saturating_add(
                (case.values.len() + case.members.len() + case.variables.len()).saturating_mul(512),
            )
            .saturating_add(
                case.values
                    .keys()
                    .chain(case.variables.keys())
                    .map(String::len)
                    .sum::<usize>(),
            )
            .saturating_add(
                (exact_parameters.len() + provisional_quadratures.len()).saturating_mul(128),
            )
            .saturating_add(
                forward_parameters.map_or(0, |parameters| parameters.len().saturating_mul(128)),
            )
            .saturating_add(
                profile
                    .forward_axes
                    .as_ref()
                    .map_or(0, |axes| axes.len().saturating_mul(size_of::<usize>())),
            );
        let owner = package
            .runtime
            .shared
            .math()
            .reserve("replay:original-simulation-constructor", bytes)?;
        let mut original = profile.clone();
        let forward_selection = if let Some(parameters) = forward_parameters {
            Some(ReplayForwardSelection::SemanticParameters(
                parameters.clone(),
            ))
        } else {
            original
                .forward_axes
                .take()
                .map(ReplayForwardSelection::PhysicalAxes)
        };
        Ok(Arc::new(Self {
            document: ReplaySimulationConstructor {
                source: 0,
                root,
                instance,
                bindings: bindings.clone(),
                preparation: super::PreparationSettings { compiler, limits },
                case: ReplayCaseInputs::new(case),
                profile: original,
                order: derivative_order(order),
                exact_parameters: exact_parameters.clone(),
                forward_selection,
                provisional_quadratures: provisional_quadratures.clone(),
                binding: binding.cloned(),
            },
            _owner: owner,
        }))
    }
    pub(in crate::workflow) fn binding(&self) -> Option<&super::AdmittedBinding> {
        self.document.binding.as_ref()
    }
    pub(in crate::workflow) fn document(&self, source: usize) -> ReplaySimulationConstructor {
        ReplaySimulationConstructor {
            source,
            ..self.document.clone()
        }
    }
}
/// Replay retains the original internal request without teaching Profile's public
/// serialization a general selection API. Exactly one coordinate form is authoritative.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "coordinates",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ReplayForwardSelection {
    /// Consumer inputs resolved to their start-interval axes during preparation.
    SemanticParameters(BTreeSet<SemanticId>),
    /// An already selected physical integration request, including scheduled intervals.
    PhysicalAxes(Vec<usize>),
}
/// Portable original integrated constructor; all methods still enter the existing supervisor.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaySimulationConstructor {
    /// Source closure index.
    pub source: usize,
    /// Original authored root.
    pub root: pse_modeling::DeclarationId,
    /// Original instance identity.
    pub instance: pse_modeling::InstanceId,
    /// Full structural arguments, facts and demand.
    #[serde(serialize_with = "serialize_bindings")]
    #[schemars(with = "Vec<u8>")]
    pub bindings: pse_modeling::Bindings,
    /// Original compiler and specialization allowances.
    pub preparation: super::PreparationSettings,
    /// Full original case values/fixed-state overrides.
    pub case: ReplayCaseInputs,
    /// Full native dynamic settings, horizon, start, sensitivities and allowance.
    pub profile: super::SimulationProfile,
    /// Original requested partial order.
    pub order: u8,
    /// Explicit exact derivative parameter demand.
    pub exact_parameters: BTreeSet<SemanticId>,
    /// Original internal forward selection, absent for ordinary full Forward.
    pub forward_selection: Option<ReplayForwardSelection>,
    /// Explicit provisional quadrature demands.
    pub provisional_quadratures: BTreeSet<SemanticId>,
    /// Original physical point binding; values are reconstructed before fresh IC preparation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<super::AdmittedBinding>,
}
impl ReplaySimulationConstructor {
    /// Reconstruct original dynamics from source through actual compiler/physical admission.
    pub async fn reconstruct(
        &self,
        package: &ModelingPackage,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingSimulation, WorkflowError> {
        let order = match self.order {
            0 => pse_kernels::DerivativeOrder::Value,
            1 => pse_kernels::DerivativeOrder::First,
            2 => pse_kernels::DerivativeOrder::Second,
            _ => return Err(contract("replay dynamic derivative order invalid")),
        };
        if self.profile.forward_axes.is_some() && self.forward_selection.is_some() {
            return Err(contract(
                "replay dynamic constructor has competing forward selections",
            ));
        }
        let mut profile = self.profile.clone();
        let forward_parameters = match &self.forward_selection {
            Some(ReplayForwardSelection::SemanticParameters(parameters)) => Some(parameters),
            Some(ReplayForwardSelection::PhysicalAxes(axes)) => {
                profile.forward_axes = Some(axes.clone());
                None
            }
            None => None,
        };
        package
            .prepare_simulation_with_controls(
                self.root,
                self.instance,
                self.bindings.clone(),
                self.preparation.limits,
                self.case.restore(),
                self.preparation.compiler,
                profile,
                order,
                &self.exact_parameters,
                forward_parameters,
                &self.provisional_quadratures,
                None,
                self.binding.as_ref(),
                None,
                cancel,
            )
            .await
    }
}
/// Original fit preparation, retained before derivative/profile resolution.
#[derive(Debug)]
pub(crate) struct OriginalFitInputs {
    document: ReplayFitConstructor,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalFitInputs {
    pub(in crate::workflow) fn capture(
        package: &ModelingPackage,
        id: pse_model::generated::identities::FitId,
        profile: &super::FitProfile,
        compiler: pse_compiler::workspace::Profile,
        limits: pse_modeling::Limits,
        binding: Option<&super::AdmittedBinding>,
        residual_lm_criteria: super::ResidualLmCriteria,
    ) -> Result<Arc<Self>, WorkflowError> {
        let extent = size_of::<Self>()
            .saturating_add(encoded_extent(&binding, EXPORT_MAX)?.saturating_mul(8))
            .saturating_add(1024)
            .saturating_add(profile_extent(&profile.solver)?)
            .saturating_add(
                encoded_extent(&(&profile.simulations, &profile.uncertainty), EXPORT_MAX)?
                    .saturating_mul(8),
            );
        let owner = package
            .runtime
            .shared
            .math()
            .reserve("replay:original-fit-constructor", extent)?;
        Ok(Arc::new(Self {
            document: ReplayFitConstructor {
                optimizer: profile.optimizer,
                residual_lm_criteria,
                source: 0,
                id,
                preparation: super::PreparationSettings { compiler, limits },
                solver: ReplaySolverSettings::from_profile(&profile.solver),
                simulations: profile.simulations.clone(),
                rank_tolerance: profile.rank_tolerance,
                max_cells: profile.max_cells,
                derivatives: profile.derivatives,
                experiment_workers: profile.experiment_workers,
                uncertainty: profile.uncertainty.clone(),
                binding: binding.cloned(),
            },
            _owner: owner,
        }))
    }
    pub(in crate::workflow) fn binding(&self) -> Option<&super::AdmittedBinding> {
        self.document.binding.as_ref()
    }
    pub(super) fn document(&self, source: usize) -> ReplayFitConstructor {
        ReplayFitConstructor {
            source,
            ..self.document.clone()
        }
    }
}
/// Portable fit constructor. Exact fit/experiment declarations are in its source objects.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayFitConstructor {
    /// Exact original residual algorithm policy, independent of physical stationarity.
    pub residual_lm_criteria: super::ResidualLmCriteria,
    /// Requested fitting method, preserved before contextual resolution.
    pub optimizer: super::fitting::FitOptimizer,
    /// Source closure index.
    pub source: usize,
    /// Exact authored fit declaration.
    pub id: pse_model::generated::identities::FitId,
    /// Original compiler and expansion controls.
    pub preparation: super::PreparationSettings,
    /// Complete original native/scientific policy.
    pub solver: ReplaySolverSettings,
    /// Complete original integration controls by experiment identity.
    #[schemars(
        with = "pse_ids::document::KeyedMap<pse_modeling::InstanceId, super::SimulationProfile>"
    )]
    pub simulations: BTreeMap<pse_modeling::InstanceId, super::SimulationProfile>,
    /// Original response rank cutoff.
    pub rank_tolerance: f64,
    /// Original finite derivative/rank allowance.
    pub max_cells: usize,
    /// Original derivative evidence policy.
    pub derivatives: super::FitDerivatives,
    /// Finite independent experiment lanes.
    pub experiment_workers: pse_model::scalars::PositiveCount,
    /// Complete requested uncertainty/propagation controls.
    pub uncertainty: Option<super::FitUncertainty>,
    /// Original global parameter values, admitted before fitting and experiment preparation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<super::AdmittedBinding>,
}
impl ReplayFitConstructor {
    /// Rebuild the authored fit/experiments through existing production admission.
    pub async fn reconstruct(
        &self,
        package: &ModelingPackage,
        cancel: &crate::CancelSource,
    ) -> Result<super::PreparedFit, WorkflowError> {
        if self.residual_lm_criteria != super::ResidualLmCriteria::current() {
            return Err(contract(
                "recorded residual LM criteria are unsupported by this build",
            ));
        }
        package
            .prepare_fit_bound(
                self.id,
                super::FitProfile {
                    optimizer: self.optimizer,
                    solver: self.solver.profile()?,
                    simulations: self.simulations.clone(),
                    rank_tolerance: self.rank_tolerance,
                    max_cells: self.max_cells,
                    derivatives: self.derivatives,
                    experiment_workers: self.experiment_workers,
                    uncertainty: self.uncertainty.clone(),
                },
                self.preparation.compiler,
                self.preparation.limits,
                self.binding.as_ref(),
                cancel,
            )
            .await
    }
}
/// Original source-bound output/control bound in exact IEEE bits.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayShootingBound {
    /// Exact contract coordinate.
    pub id: SemanticId,
    /// Original optional lower bound.
    pub lower: Option<u64>,
    /// Original optional upper bound.
    pub upper: Option<u64>,
}
/// Original shooting request over its complete original simulation constructor.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayShootingConstructor {
    /// Complete original integrated constructor.
    pub simulation: ReplaySimulationConstructor,
    /// Single or multiple shooting.
    pub method: pse_model::generated::enums::ShootingMethod,
    /// Exact requested inner node times.
    pub nodes: Vec<u64>,
    /// Original scheduled control coordinates and bounds.
    pub controls: Vec<ReplayShootingBound>,
    /// Original path output coordinates and bounds.
    pub path: Vec<ReplayShootingBound>,
    /// Original terminal objective weights.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub terminal: BTreeMap<SemanticId, u64>,
    /// Original integral objective weights.
    #[schemars(with = "pse_ids::document::KeyedMap<SemanticId, u64>")]
    pub integral: BTreeMap<SemanticId, u64>,
    /// Original callback derivative demand policy.
    pub eager_first: bool,
    /// Complete original scientific/native controls.
    pub solver: ReplaySolverSettings,
    /// Original caller starting coordinate vector, distinct from native continuation.
    pub initial: Option<Vec<u64>>,
}
#[cfg(feature = "solver-diffsol")]
impl ReplayShootingConstructor {
    pub(super) fn new(
        problem: &super::ShootingProblem,
        initial: &Option<Vec<f64>>,
    ) -> Result<Self, WorkflowError> {
        let request = &problem.original_inputs;
        Ok(Self {
            simulation: problem
                .simulation
                .original_inputs
                .as_ref()
                .ok_or_else(|| contract("shooting original simulation constructor absent"))?
                .document(0),
            method: request.method,
            nodes: request.nodes.iter().map(|v| v.to_bits()).collect(),
            controls: request
                .controls
                .iter()
                .map(|v| ReplayShootingBound {
                    id: v.input,
                    lower: v.lower.map(f64::to_bits),
                    upper: v.upper.map(f64::to_bits),
                })
                .collect(),
            path: request
                .path
                .iter()
                .map(|v| ReplayShootingBound {
                    id: v.output,
                    lower: v.lower.map(f64::to_bits),
                    upper: v.upper.map(f64::to_bits),
                })
                .collect(),
            terminal: bits(&request.objective.terminal),
            integral: bits(&request.objective.integral),
            eager_first: request.evaluation == super::ShootingEvaluationPolicy::EagerFirst,
            solver: ReplaySolverSettings::from_profile(&request.solver),
            initial: initial
                .as_ref()
                .map(|v| v.iter().map(|v| v.to_bits()).collect()),
        })
    }
    /// Rebuild original shooting through the existing simulation and shooting admission owners.
    pub async fn reconstruct(
        &self,
        package: &ModelingPackage,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<super::ShootingProblem>, WorkflowError> {
        let simulation = self.simulation.reconstruct(package, cancel).await?;
        Ok(Arc::new(
            simulation.shooting(super::ShootingProfile {
                method: self.method,
                nodes: self.nodes.iter().map(|v| f64::from_bits(*v)).collect(),
                controls: self
                    .controls
                    .iter()
                    .map(|v| super::ShootingControl {
                        input: v.id,
                        lower: v.lower.map(f64::from_bits),
                        upper: v.upper.map(f64::from_bits),
                    })
                    .collect(),
                path: self
                    .path
                    .iter()
                    .map(|v| super::PathBound {
                        output: v.id,
                        lower: v.lower.map(f64::from_bits),
                        upper: v.upper.map(f64::from_bits),
                    })
                    .collect(),
                objective: super::ShootingObjective {
                    terminal: unbits(&self.terminal),
                    integral: unbits(&self.integral),
                },
                evaluation: if self.eager_first {
                    super::ShootingEvaluationPolicy::EagerFirst
                } else {
                    super::ShootingEvaluationPolicy::Demanded
                },
                solver: self.solver.profile()?,
            })?,
        ))
    }
}

impl ReplayCaseConstructor {
    fn from_analysis(
        source: usize,
        analysis: &super::ModelingAnalysis,
    ) -> Result<Self, WorkflowError> {
        if !analysis.numerical.implicit.is_empty() {
            return Err(contract(
                "portable analysis replay requires library-owned export of caller-supplied implicit capabilities",
            ));
        }
        Self::descriptive_analysis(source, analysis)
    }
    fn descriptive_analysis(
        source: usize,
        analysis: &super::ModelingAnalysis,
    ) -> Result<Self, WorkflowError> {
        Ok(Self {
            source,
            root: analysis.root,
            instance: analysis.instance,
            bindings: analysis.bindings.clone(),
            preparation: super::PreparationSettings {
                compiler: analysis.compiler,
                limits: analysis.limits,
            },
            case: ReplayCaseInputs::new(&analysis.case),
            order: derivative_order(analysis.order),
            settings: ReplaySolverSettings::from_profile(&analysis.solver),
            declarations: analysis.numerical.declarations.clone(),
            targets: analysis.numerical.targets.clone(),
            seed: BTreeMap::new(),
            parameters: BTreeMap::new(),
            fixes: BTreeMap::new(),
            requested_start: None,
            requested_screened_start: None,
            prepared_graph: None,
        })
    }
    fn analysis(&self) -> Result<super::ModelingAnalysis, WorkflowError> {
        let order = match self.order {
            0 => pse_kernels::DerivativeOrder::Value,
            1 => pse_kernels::DerivativeOrder::First,
            2 => pse_kernels::DerivativeOrder::Second,
            _ => return Err(contract("replay analysis derivative order invalid")),
        };
        Ok(super::ModelingAnalysis {
            root: self.root,
            instance: self.instance,
            bindings: self.bindings.clone(),
            limits: self.preparation.limits,
            case: self.case.restore(),
            order,
            compiler: self.preparation.compiler,
            solver: self.settings.profile()?,
            numerical: crate::math::solves::NumericalInputs {
                declarations: self.declarations.clone(),
                targets: self.targets.clone(),
                implicit: BTreeMap::new(),
            },
        })
    }
}
/// Exact original horizon signal, preserving all trajectory value bits.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ReplayHorizonSignal {
    /// Plant contract output.
    Measured(SemanticId),
    /// Named estimator output.
    Estimated(String),
    /// Driven input ordinal.
    Applied(usize),
    /// Complete declared trajectory in exact original bits.
    Trajectory(Vec<u64>),
}
impl ReplayHorizonSignal {
    fn new(value: &super::HorizonSignal) -> Self {
        match value {
            super::HorizonSignal::Measured(id) => Self::Measured(*id),
            super::HorizonSignal::Estimated(path) => Self::Estimated(path.clone()),
            super::HorizonSignal::Applied(input) => Self::Applied(*input),
            super::HorizonSignal::Trajectory(values) => {
                Self::Trajectory(values.iter().map(|v| v.to_bits()).collect())
            }
        }
    }
    fn restore(&self) -> super::HorizonSignal {
        match self {
            Self::Measured(id) => super::HorizonSignal::Measured(*id),
            Self::Estimated(path) => super::HorizonSignal::Estimated(path.clone()),
            Self::Applied(input) => super::HorizonSignal::Applied(*input),
            Self::Trajectory(values) => super::HorizonSignal::Trajectory(
                values.iter().map(|v| f64::from_bits(*v)).collect(),
            ),
        }
    }
}
/// Original NMPC stage, including its complete source, analysis and mapping.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayHorizonController {
    /// Full source-qualified original analysis.
    pub analysis: ReplayCaseConstructor,
    /// Exact original case-signal bindings.
    pub bindings: Vec<(String, ReplayHorizonSignal)>,
    /// Original first-move mappings.
    pub moves: Vec<(String, usize)>,
    /// Original optional advanced-step predictions.
    pub predictions: Option<Vec<(String, String)>>,
}
/// Original MHE stage, including exact window and prior meaning.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayHorizonEstimator {
    /// Full source-qualified original analysis.
    pub analysis: ReplayCaseConstructor,
    /// Original finite estimation window.
    pub window: usize,
    /// Exact ordered measured-output mappings.
    pub measurements: Vec<(SemanticId, Vec<String>)>,
    /// Original driven-input window bindings; false is Constant, true is Periods.
    pub inputs: Vec<(usize, bool, Vec<String>)>,
    /// Full prior/next-state mapping and exact original prior bits.
    pub arrival: Vec<(String, String, u64)>,
}
/// Full original horizon constructor; an interrupted observation never replaces its requested horizon.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayHorizonConstructor {
    /// Original output demand, separate from observed coverage.
    pub output: super::StudyOutputDemand,
    /// Complete original plant constructor and source index.
    pub plant: ReplaySimulationConstructor,
    /// Original period in exact IEEE bits.
    pub period: u64,
    /// Original requested finite periods.
    pub steps: usize,
    /// Exact driven plant identities and initial input bits.
    pub inputs: Vec<(SemanticId, u64)>,
    /// Full optional original estimator.
    pub estimator: Option<ReplayHorizonEstimator>,
    /// Full optional original controller.
    pub controller: Option<ReplayHorizonController>,
}
impl ReplayHorizonConstructor {
    pub(in crate::workflow) fn new(horizon: &super::Horizon) -> Result<Self, WorkflowError> {
        Self::from_retained(horizon, true)
    }
    pub(super) fn descriptive(horizon: &super::Horizon) -> Result<Self, WorkflowError> {
        Self::from_retained(horizon, false)
    }
    fn from_retained(horizon: &super::Horizon, executable: bool) -> Result<Self, WorkflowError> {
        let analysis = |source, body| {
            if executable {
                ReplayCaseConstructor::from_analysis(source, body)
            } else {
                ReplayCaseConstructor::descriptive_analysis(source, body)
            }
        };
        let mut next = 1;
        let estimator = if let Some(estimator) = &horizon.estimator {
            let analysis = analysis(next, &estimator.analysis)?;
            next += 1;
            Some(ReplayHorizonEstimator {
                analysis,
                window: estimator.window,
                measurements: estimator.measurements.clone(),
                inputs: estimator
                    .inputs
                    .iter()
                    .map(|(input, binding)| match binding {
                        super::WindowInput::Constant(path) => (*input, false, vec![path.clone()]),
                        super::WindowInput::Periods(paths) => (*input, true, paths.clone()),
                    })
                    .collect(),
                arrival: estimator
                    .arrival
                    .iter()
                    .map(|v| (v.prior.clone(), v.next.clone(), v.initial.to_bits()))
                    .collect(),
            })
        } else {
            None
        };
        let controller = if let Some(controller) = &horizon.controller {
            Some(ReplayHorizonController {
                analysis: analysis(next, &controller.analysis)?,
                bindings: controller
                    .bindings
                    .iter()
                    .map(|(path, value)| (path.clone(), ReplayHorizonSignal::new(value)))
                    .collect(),
                moves: controller.moves.clone(),
                predictions: controller.advanced.as_ref().map(|v| v.predictions.clone()),
            })
        } else {
            None
        };
        Ok(Self {
            output: horizon.output.clone(),
            plant: horizon
                .plant
                .original_inputs
                .as_ref()
                .ok_or_else(|| contract("original horizon plant constructor absent"))?
                .document(0),
            period: horizon.period.to_bits(),
            steps: horizon.steps,
            inputs: horizon
                .inputs
                .iter()
                .map(|v| (v.parameter, v.initial.to_bits()))
                .collect(),
            estimator,
            controller,
        })
    }
    /// Rebuild the exact original plant and stages from admitted source packages.
    pub async fn reconstruct(
        &self,
        sources: &[ModelingPackage],
        cancel: &crate::CancelSource,
    ) -> Result<super::Horizon, WorkflowError> {
        let source = |index| {
            sources
                .get(index)
                .ok_or_else(|| contract("replay horizon source closure absent"))
        };
        let plant = self
            .plant
            .reconstruct(source(self.plant.source)?, cancel)
            .await?;
        let estimator = if let Some(estimator) = &self.estimator {
            let mut inputs = Vec::new();
            for (input, periods, paths) in &estimator.inputs {
                let binding = if *periods {
                    super::WindowInput::Periods(paths.clone())
                } else {
                    if paths.len() != 1 {
                        return Err(contract("replay constant window input extent"));
                    }
                    super::WindowInput::Constant(paths[0].clone())
                };
                inputs.push((*input, binding));
            }
            Some(super::HorizonEstimator {
                package: source(estimator.analysis.source)?.clone(),
                analysis: estimator.analysis.analysis()?,
                window: estimator.window,
                measurements: estimator.measurements.clone(),
                inputs,
                arrival: estimator
                    .arrival
                    .iter()
                    .map(|(prior, next, initial)| super::Arrival {
                        prior: prior.clone(),
                        next: next.clone(),
                        initial: f64::from_bits(*initial),
                    })
                    .collect(),
            })
        } else {
            None
        };
        let controller = if let Some(controller) = &self.controller {
            Some(super::HorizonController {
                package: source(controller.analysis.source)?.clone(),
                analysis: controller.analysis.analysis()?,
                bindings: controller
                    .bindings
                    .iter()
                    .map(|(path, value)| (path.clone(), value.restore()))
                    .collect(),
                moves: controller.moves.clone(),
                advanced: controller
                    .predictions
                    .as_ref()
                    .map(|predictions| super::AdvancedStep {
                        predictions: predictions.clone(),
                    }),
            })
        } else {
            None
        };
        Ok(super::Horizon {
            output: self.output.clone(),
            plant,
            period: f64::from_bits(self.period),
            steps: self.steps,
            inputs: self
                .inputs
                .iter()
                .map(|(parameter, initial)| super::HorizonInput {
                    parameter: *parameter,
                    initial: f64::from_bits(*initial),
                })
                .collect(),
            estimator,
            controller,
        })
    }
}

const EXPORT_MAX: usize = 256 * 1024 * 1024;
/// Counting serialization admits copies without allocating a second serialized document.
struct ExtentWriter {
    written: usize,
    limit: usize,
}
impl std::io::Write for ExtentWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let total = self
            .written
            .checked_add(bytes.len())
            .filter(|total| *total <= self.limit)
            .ok_or_else(|| {
                std::io::Error::other("complete replay extent exceeds finite allocation")
            })?;
        self.written = total;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn encoded_extent(
    value: &impl serde::Serialize,
    limit: usize,
) -> Result<usize, WorkflowError> {
    let mut writer = ExtentWriter { written: 0, limit };
    serde_json::to_writer(&mut writer, value)
        .map_err(|error| contract(format!("replay encoding extent: {error}")))?;
    Ok(writer.written)
}
pub(in crate::workflow) fn profile_extent(
    profile: &crate::math::solves::SolverProfile,
) -> Result<usize, WorkflowError> {
    // Native option export is fixed-size (21 entries); variable scientific/advice/settings
    // bodies are borrowed during this count, before any clone/allocation.
    let options = profile.presolve.document_options();
    let required = match &profile.presolve {
        pse_backend_native::presolve::Policy::Explicit { required, .. } => Some(required),
        _ => None,
    };
    encoded_extent(
        &(
            &profile.controls,
            &profile.composition,
            &profile.numerics,
            profile.backend.document(),
            &profile.sensitivity,
            &profile.reconstruction,
            options,
            required,
        ),
        EXPORT_MAX,
    )
    .map(|bytes| {
        bytes
            .saturating_mul(8)
            .saturating_add(profile.controls.options.len().saturating_mul(512))
            .saturating_add(size_of::<ReplaySolverSettings>() + 1024)
    })
}

fn trajectory_observation(
    report: &pse_backend_native::dynamics::Report,
) -> pse_model::generated::enums::NumericalAttemptObservation {
    use pse_backend_native::dynamics::Termination as T;
    use pse_model::generated::enums::NumericalAttemptObservation as O;
    if let Some(error) = &report.error {
        return crate::math::strategy::failure(error);
    }
    match report.termination {
        T::Completed | T::Event => O::Converged,
        T::Cancelled => O::Cancelled,
        T::TimeLimit => O::ResourceExhausted,
        T::StepLimit | T::EventLimit => O::Limited,
        T::Failed => O::NumericalFailure,
        T::Panic => O::Panic,
    }
}

/// Complete retained recorded decisions with their allocation owner.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayDecisionDocument {
    /// Full immutable governing declarations and decision observations.
    pub decisions: Vec<ReplayDecisions>,
    #[serde(skip)]
    #[schemars(skip)]
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl ScientificReplay {
    fn constructor_extent(&self) -> Result<usize, WorkflowError> {
        let original = match self.inputs() {
            RunRequest::Modeling(steps) => steps.iter().try_fold(1024usize, |bytes, step| {
                let inputs = &step.original_inputs;
                let specialization = inputs
                    .specialization
                    .as_ref()
                    .map_or(0, |original| original.retained_bytes());
                let requested_start = step
                    .solve
                    .requested_start()
                    .map_or(0, warm_start_extent)
                    .saturating_add(ReplayPreparedGraph::extent(&step.solve, 0, &mut 0)?);
                let screened = step.solve.requested_screened_start().map_or(0, |start| {
                    start.point().len().saturating_mul(256).saturating_add(2048)
                });
                bytes
                    .checked_add(inputs._owner.size())
                    .and_then(|bytes| bytes.checked_add(specialization))
                    .and_then(|bytes| {
                        bytes
                            .checked_add(requested_start.saturating_mul(8).saturating_add(screened))
                    })
                    .and_then(|bytes| bytes.checked_add(profile_extent(&step.profile).ok()?))
                    .and_then(|bytes| {
                        bytes.checked_add(
                            encoded_extent(&(&step.starts, &step.model.values.scalars), EXPORT_MAX)
                                .ok()?
                                .saturating_mul(8),
                        )
                    })
                    .and_then(|bytes| {
                        bytes.checked_add(step.bound_source.as_ref().map_or(
                            Some(0),
                            |binding| {
                                encoded_extent(&***binding, EXPORT_MAX)
                                    .ok()
                                    .map(|bytes| bytes.saturating_mul(8))
                            },
                        )?)
                    })
                    .ok_or_else(|| contract("complete case replay extent overflow"))
            })?,
            RunRequest::Simulation(simulation) => simulation
                .original_inputs
                .as_ref()
                .map_or(1024, |inputs| inputs._owner.size()),
            RunRequest::Fit(fit) => fit
                .original_inputs
                ._owner
                .size()
                .saturating_add(profile_extent(&fit.problem.profile.solver)?),
            RunRequest::Horizon(request) => {
                let horizon = &request.original;
                let mut bytes = horizon
                    .plant
                    .original_inputs
                    .as_ref()
                    .map_or(1024, |inputs| inputs._owner.size())
                    .saturating_add(request.provenance.len())
                    .saturating_add(horizon.inputs.len().saturating_mul(128));
                for analysis in horizon
                    .estimator
                    .iter()
                    .map(|stage| &stage.analysis)
                    .chain(horizon.controller.iter().map(|stage| &stage.analysis))
                {
                    bytes = bytes
                        .saturating_add(crate::math::modeling::bindings_bytes(&analysis.bindings))
                        .saturating_add(profile_extent(&analysis.solver)?)
                        .saturating_add(
                            encoded_extent(
                                &(
                                    &analysis.numerical.declarations,
                                    &analysis.numerical.targets,
                                ),
                                EXPORT_MAX,
                            )?
                            .saturating_mul(8),
                        )
                        .saturating_add(
                            (analysis.case.values.len()
                                + analysis.case.members.len()
                                + analysis.case.variables.len())
                            .saturating_mul(512),
                        )
                        .saturating_add(
                            analysis
                                .case
                                .values
                                .keys()
                                .chain(analysis.case.variables.keys())
                                .map(String::len)
                                .sum::<usize>(),
                        );
                }
                if let Some(estimator) = &horizon.estimator {
                    bytes = bytes
                        .saturating_add(
                            estimator
                                .measurements
                                .iter()
                                .map(|(_, paths)| {
                                    paths.iter().map(|path| path.len() + 128).sum::<usize>()
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(
                            estimator
                                .inputs
                                .iter()
                                .map(|(_, input)| match input {
                                    super::WindowInput::Constant(path) => path.len() + 128,
                                    super::WindowInput::Periods(paths) => {
                                        paths.iter().map(|path| path.len() + 128).sum()
                                    }
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(
                            estimator
                                .arrival
                                .iter()
                                .map(|arrival| arrival.prior.len() + arrival.next.len() + 256)
                                .sum::<usize>(),
                        );
                }
                if let Some(controller) = &horizon.controller {
                    bytes = bytes
                        .saturating_add(
                            controller
                                .bindings
                                .iter()
                                .map(|(path, signal)| {
                                    path.len()
                                        + 128
                                        + match signal {
                                            super::HorizonSignal::Estimated(path) => path.len(),
                                            super::HorizonSignal::Trajectory(values) => {
                                                values.len().saturating_mul(16)
                                            }
                                            _ => 0,
                                        }
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(
                            controller
                                .moves
                                .iter()
                                .map(|(path, _)| path.len() + 128)
                                .sum::<usize>(),
                        )
                        .saturating_add(controller.advanced.as_ref().map_or(0, |advanced| {
                            advanced
                                .predictions
                                .iter()
                                .map(|(a, b)| a.len() + b.len() + 256)
                                .sum()
                        }));
                }
                bytes
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => problem
                .replay_constructor_extent()
                .saturating_add(
                    problem
                        .simulation
                        .original_inputs
                        .as_ref()
                        .map_or(1024, |inputs| inputs._owner.size()),
                )
                .saturating_add(
                    initial
                        .as_ref()
                        .map_or(0, |values| values.len().saturating_mul(16)),
                )
                .saturating_add(profile_extent(&problem.solver)?),
        };
        original
            .checked_mul(8)
            .and_then(|bytes| bytes.checked_add(65536))
            .ok_or_else(|| contract("complete replay constructor extent overflow"))
    }
}

pub(in crate::workflow) fn source_reconstruction_extent(
    document: &ReplaySourceDocument,
) -> Result<usize, WorkflowError> {
    encoded_extent(document, EXPORT_MAX)?
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(65536))
        .filter(|bytes| *bytes <= EXPORT_MAX)
        .ok_or_else(|| contract("complete replay source reconstruction exceeds finite capacity"))
}

/// Native record availability, separate from physical sample/period coverage.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplayNativeRecordCoverage {
    /// All native reports produced by retained strategy operations have compact records.
    StrategyAttempts,
    /// Integration retains its trajectory report and partial coverage; per-attempt adapter
    /// option/start/counter records are not supplied by that producer and remain unavailable.
    IntegrationReportOnly,
}
/// Workflow attribution over the native owner's immutable compact attempt facts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayNativeAttempt<T = pse_backend_native::solve::AttemptRecord> {
    /// Original modeling step, or zero for fit/shooting.
    pub step: usize,
    /// Actual operation ordinal in the retained governing declaration.
    pub mechanism: usize,
    /// Actual ordered block/causal component ordinal; absent for a whole-problem native report.
    pub component: Option<usize>,
    /// Full actual native options, termination, counters, start receipt and raw evidence.
    pub record: Arc<T>,
}
impl ScientificReplay {
    pub(super) fn native_attempts(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            usize,
            Option<usize>,
            &Arc<pse_backend_native::solve::AttemptRecord>,
        ),
    > {
        self.result.native_attempt_records()
    }
}
