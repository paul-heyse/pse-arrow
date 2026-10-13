// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable scientific replay levels. Reruns are new executions, never occurrence recovery.
use super::{ModelingPackage, PhysicalContext, RunHandle, RunReport, RunRequest, RunResult, Runtime,
    WorkflowError, contract};
use pse_backend_native::{execution::BackendSettings, solve::{Backend, Controls, SolveIntent, SolverSelection}};
use pse_ids::{ContentHash, SemanticId};
use pse_model::{diagnostic::Observation, generated::{enums::ExternalDerivativeSource, identities::RunId},
    strategy::{CompositionRequest, NumericalStrategy, StartOrigin}};
use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayPhysicalSource {
    /// Registry relation identity.
    pub relation: SemanticId,
    /// Complete Arrow IPC stream, including schema and actual admitted rows.
    pub ipc: Vec<u8>,
}
pub(in crate::workflow) fn physical_sources(physical: &PhysicalContext) -> Result<Vec<ReplayPhysicalSource>, WorkflowError> {
    use datafusion::arrow::ipc::writer::StreamWriter;
    physical.sources.values().map(|batch| {
        let mut ipc = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut ipc, &batch.batch().schema())
                .map_err(|error| contract(format!("replay physical source encoding: {error}")))?;
            writer.write(batch.batch()).map_err(|error| contract(format!("replay physical source encoding: {error}")))?;
            writer.finish().map_err(|error| contract(format!("replay physical source encoding: {error}")))?;
        }
        Ok(ReplayPhysicalSource { relation: batch.relation_id(), ipc })
    }).collect()
}
/// Complete ordered scalar ABI coordinate of a required executable provider.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ReplayProviderPort {
    /// Exact scalar port identity.
    pub id: SemanticId,
    /// Physical quantity, including basis and reference.
    pub quantity: SemanticId,
    /// Actual representation unit.
    pub unit: SemanticId,
}
/// Explicit logical array layout over a provider's scalar ABI.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
        let spec = registration.spec();
        let ports = |ports: &[pse_kernels::Port]| ports.iter().map(|port| ReplayProviderPort {
            id: port.id, quantity: port.quantity.as_id(), unit: port.unit.as_id() }).collect();
        let shapes = |shapes: &[pse_kernels::ProviderShape]| shapes.iter().map(|shape| ReplayProviderShape {
            id: shape.id, axes: shape.axes.clone(), coordinates: shape.coordinates.clone(), cells: shape.cells.clone() }).collect();
        let order = |order| match order { pse_kernels::DerivativeOrder::Value => 0,
            pse_kernels::DerivativeOrder::First => 1, pse_kernels::DerivativeOrder::Second => 2 };
        Self { name: name.into(), implementation: spec.id, revision: spec.revision, data: spec.data,
            configuration: registration.configuration_key(), inputs: ports(&spec.inputs), outputs: ports(&spec.outputs),
            input_shapes: shapes(&spec.shapes.inputs), output_shapes: shapes(&spec.shapes.outputs),
            derivative_source: spec.derivative_source, derivatives: order(spec.derivatives), smoothness: order(spec.smoothness) }
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
    pub fn reconstruct(&self, runtime: &Runtime, physical: PhysicalContext,
        providers: BTreeMap<String, pse_kernels::Registration>) -> Result<ModelingPackage, WorkflowError> {
        runtime.admit_replay_source(self, physical, providers)
    }
}
/// Full actual requested solver controls; native settings remain their original typed authority.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ReplaySolverSettings {
    /// Original scientific analysis goal.
    pub intent: SolveIntent,
    /// Explicit native selection; absent means Auto.
    pub backend: Option<Backend>,
    /// Full materialized backend settings.
    pub settings: BackendSettings,
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
            pse_math::convexity::ConvexityPolicy::Numerical { absolute, relative } => (Some(absolute), Some(relative)),
        };
        Self { intent: profile.intent, backend: match profile.selection { SolverSelection::Auto => None,
                SolverSelection::Explicit(backend) => Some(backend) },
            settings: profile.backend.clone(), controls: profile.controls.clone(), composition: profile.composition.clone(),
            numerics: profile.numerics.clone(), presolve: profile.presolve.kind(),
            presolve_options: profile.presolve.document_options(),
            required_passes: match &profile.presolve { pse_backend_native::presolve::Policy::Explicit { required, .. } => required.clone(), _ => BTreeSet::new() },
            convexity_absolute, convexity_relative, reconstruction: profile.reconstruction,
            sensitivity: profile.sensitivity.clone() }
    }
}
/// Full retained governing declaration and synchronous decisions for one original step.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
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
    fn new(step: usize, trace: &crate::math::strategy::Trace) -> Self {
        Self { step, declaration: trace.declaration.clone(), original: trace.original,
            profile: trace.profile, start: trace.start, starts: trace.starts.clone(),
            decisions: trace.events.iter().map(super::attempt_documents::AttemptDecision::from).collect() }
    }
}
/// Recorded coverage, without promoting an interrupted prefix into a full trajectory.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
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
pub struct ReplayDocument {
    /// Explicit current document version.
    pub version: pse_model::document::Version<1>,
    /// Historical source execution; a numerical rerun always gets another identity.
    pub source_run: RunId,
    /// Exact required binary build.
    pub build: ContentHash,
    /// Full immutable authored/provider/physical source closures.
    pub sources: Vec<ReplaySourceDocument>,
    /// Full actual solver settings for all original solver owners.
    pub solver_settings: Vec<ReplaySolverSettings>,
    /// Original portable case constructors, retaining all supplied structural and numerical inputs.
    pub case_constructors: Vec<ReplayCaseConstructor>,
    /// Original physical mappings and full constructor configurations, using existing producer documents.
    pub contexts: Vec<pse_model::generated::runtime::scientific_contexts::Row>,
    /// Full original horizon provenance, including output demand, period, stages and mappings.
    pub horizon: Option<Vec<u8>>,
    /// Full retained decisions with their actual effective declaration.
    pub decisions: Vec<ReplayDecisions>,
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
    runtime: Runtime,
    request: RunRequest,
    traces: Vec<(usize, Arc<crate::math::strategy::Trace>)>,
    coverage: ReplayCoverage,
}
impl ScientificReplay {
    /// Full immutable production inputs with actual physical/provider/compiled owners.
    pub fn inputs(&self) -> &RunRequest { &self.request }
    /// Inspect exact original effective declarations/decisions without running numerical work.
    pub fn recorded_decisions(&self) -> Vec<ReplayDecisions> {
        self.traces.iter().map(|(step, trace)| ReplayDecisions::new(*step, trace)).collect()
    }
    fn sources(&self) -> Vec<&ModelingPackage> {
        match &self.request {
            RunRequest::Modeling(steps) => steps.iter().map(|step| &step.source).collect(),
            RunRequest::Simulation(simulation) => vec![&simulation.source],
            RunRequest::Fit(fit) => vec![&fit.source],
            RunRequest::Horizon(horizon) => horizon.sources.iter().collect(),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, .. } => vec![&problem.simulation.source],
        }
    }
    /// Reconstruct actual locally usable source packages into a receiving runtime.
    pub fn reconstruct_sources(&self, runtime: &Runtime) -> Result<Vec<ModelingPackage>, WorkflowError> {
        self.sources().into_iter().map(|source| source.reconstruct_replay_source(runtime)).collect()
    }
    /// Default bounded complete export. Oversized closure refuses instead of truncating required inputs.
    pub fn document(&self) -> Result<ReplayDocument, WorkflowError> { self.document_with_limit(16 * 1024 * 1024) }
    /// Explicit finite export allocation. Caller receives either the complete closure or refusal.
    pub fn document_with_limit(&self, bytes: usize) -> Result<ReplayDocument, WorkflowError> {
        if bytes == 0 || bytes > 256 * 1024 * 1024 { return Err(contract("replay export requires a finite allocation up to 256 MiB")); }
        let sources = self.sources();
        let extent = sources.iter().try_fold(65536usize, |total, source| {
            total.checked_add(source.revision.admitted.retained_bytes().checked_mul(16)?)
                .and_then(|total| total.checked_add(source.physical.retained_bytes().checked_mul(16)?))
        }).ok_or_else(|| contract("replay export extent overflow"))?;
        if extent > bytes { return Err(contract("complete replay closure exceeds requested export allocation")); }
        let owner = self.runtime.shared.math().reserve("replay:complete-export", bytes)?;
        let sources = sources.into_iter().map(ModelingPackage::replay_source).collect::<Result<Vec<_>, _>>()?;
        let mut solver_settings = Vec::new();
        let mut case_constructors = Vec::new();
        let mut contexts = Vec::new();
        let mut horizon = None;
        match &self.request {
            RunRequest::Modeling(steps) => for (index, step) in steps.iter().enumerate() {
                solver_settings.push(ReplaySolverSettings::from_profile(&step.profile));
                case_constructors.push(step.original_inputs.document(index)?);
                contexts.push(super::artifacts::solve_context(step, self.source_run,
                    i64::try_from(index).map_err(|_| contract("replay step ordinal"))?)?);
            },
            RunRequest::Simulation(simulation) => contexts.push(simulation.scientific_context(self.source_run, super::dynamics::profile_identity(simulation.profile()).map_err(crate::math::MathRuntimeError::from)?.as_id())?),
            RunRequest::Fit(fit) => {
                solver_settings.push(ReplaySolverSettings::from_profile(&fit.problem.profile.solver));
                contexts.push(fit.scientific_context(self.source_run)?);
            }
            RunRequest::Horizon(request) => {
                horizon = Some(request.provenance.clone());
                let original = &request.original;
                contexts.push(original.plant.scientific_context(self.source_run, super::dynamics::profile_identity(original.plant.profile()).map_err(crate::math::MathRuntimeError::from)?.as_id())?);
                if let Some(estimator) = &original.estimator { solver_settings.push(ReplaySolverSettings::from_profile(&estimator.analysis.solver)); }
                if let Some(controller) = &original.controller { solver_settings.push(ReplaySolverSettings::from_profile(&controller.analysis.solver)); }
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => {
                solver_settings.push(ReplaySolverSettings::from_profile(&problem.solver));
                let mut context = problem.simulation.scientific_context(self.source_run, problem.profile_key.as_id())?;
                // The exact caller initial is part of input reconstruction, not a native checkpoint.
                context.binding = Some(serde_json::to_vec(initial).map_err(|error| contract(error.to_string()))?.into());
                contexts.push(context);
            }
        }
        let document = ReplayDocument { version: pse_model::document::Version, source_run: self.source_run,
            build: pse_backend_native::execution::LINKED.build_identity(), sources, solver_settings, case_constructors, contexts, horizon,
            decisions: self.recorded_decisions(), coverage: self.coverage.clone(), native_continuation: false, _owner: owner };
        let encoded = serde_json::to_vec(&document).map_err(|error| contract(error.to_string()))?;
        if encoded.len().saturating_mul(2) > bytes { return Err(contract("complete replay document exceeds export allocation")); }
        Ok(document)
    }
    /// Start a new tolerant execution under fresh identity/allowance through the original supervisor.
    /// Frozen controls/inputs remain exact; scheduling, floating-point/native paths may differ.
    pub async fn rerun(&self) -> Result<RunHandle, WorkflowError> {
        let cancel = crate::CancelSource::new();
        match &self.request {
            RunRequest::Modeling(steps) => self.runtime.start_modeling(steps.clone(), false, &cancel).await,
            RunRequest::Simulation(simulation) => simulation.start(),
            RunRequest::Fit(fit) => fit.start(),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Horizon(request) => self.runtime.start_horizon(request.original.as_ref().clone(), &cancel).await,
            #[cfg(not(feature = "solver-diffsol"))]
            RunRequest::Horizon(_) => Err(contract("horizon replay requires the original linked dynamic supervisor")),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => match initial { Some(initial) => problem.start_with_initial(initial.clone()), None => problem.start() },
        }
    }
    /// Numerical points are not library continuation checkpoints.
    pub fn native_continuation(&self) -> Result<RunHandle, WorkflowError> {
        Err(contract("native continuation is unsupported: no library-owned checkpoint was exported"))
    }
}
impl RunResult {
    /// Retain the complete immutable input/decision closure, without numerical work or history lookup.
    pub fn replay(&self) -> ScientificReplay {
        let mut traces = Vec::new();
        let (requested, observed, completed_time) = match &self.request {
            RunRequest::Modeling(steps) => {
                let mut count = 0;
                for index in 0..steps.len() { if let Some(result) = self.modeling_result(index) {
                    count += 1;
                    if let Some(trace) = &result.strategy { traces.push((index, trace.clone())); }
                }}
                (steps.len(), count, None)
            }
            RunRequest::Simulation(simulation) => match &self.report {
                Ok(RunReport::Simulation(trajectory)) => (simulation.profile().samples.len(), trajectory.report().samples.len(), Some(Observation::number(trajectory.report().completed_time))),
                _ => (simulation.profile().samples.len(), 0, None),
            },
            RunRequest::Fit(_) => { if let Ok(RunReport::Fit(report)) = &self.report
                && let Some(trace) = &report.strategy { traces.push((0, trace.clone())); }
                (1, usize::from(self.report.is_ok()), None) },
            RunRequest::Horizon(request) => (request.original.steps,
                self.horizon.as_ref().map_or(0, |report| report.completed_steps), None),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { .. } => { if let Ok(RunReport::Shooting(report)) = &self.report
                && let Some(trace) = &report.strategy { traces.push((0, trace.clone())); }
                (1, usize::from(self.report.is_ok()), None) },
        };
        if let Err(error) = &self.report && let Some(trace) = error.strategy_trace() {
            traces.push((observed, trace.clone()));
        }
        ScientificReplay { source_run: self.run_id, runtime: self.runtime.clone(), request: self.request.clone(), traces,
            coverage: ReplayCoverage { requested, observed, completed_time, usable: self.usable(),
                terminal: self.report.as_ref().err().map(|error| crate::math::strategy::workflow_failure(error)) } }
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
    #[expect(clippy::too_many_arguments, reason = "retain the existing actual constructor arguments without inventing another request language")]
    pub(in crate::workflow) fn capture(source: &ModelingPackage,
        model: &crate::math::modeling::ModelingPreparation,
        case: &pse_compiler::workspace::ModelingCaseBindings,
        order: pse_kernels::DerivativeOrder, compiler: pse_compiler::workspace::Profile,
        solver: &crate::math::solves::SolverProfile, numerical: &crate::math::solves::NumericalInputs,
        overrides: &super::modeling::cases::CaseOverrides) -> Result<Arc<Self>, WorkflowError> {
        let bytes = 65536usize.saturating_add(case.values.len().saturating_mul(128))
            .saturating_add(case.members.len().saturating_mul(128))
            .saturating_add(case.variables.len().saturating_mul(256))
            .saturating_add(case.values.keys().chain(case.variables.keys()).map(String::len).sum::<usize>())
            .saturating_add(numerical.declarations.len().saturating_mul(2048))
            .saturating_add(numerical.targets.len().saturating_mul(256))
            .saturating_add((overrides.seed.len() + overrides.parameters.len() + overrides.fixes.len()).saturating_mul(128));
        Ok(Arc::new(Self { specialization: model.original.clone(), case: case.clone(), order, compiler,
            solver: solver.clone(), numerical: numerical.clone(), overrides: overrides.clone(),
            _owner: source.runtime.shared.math().reserve("replay:original-case-constructor", bytes)? }))
    }
    fn document(&self, source: usize) -> Result<ReplayCaseConstructor, WorkflowError> {
        let original = self.specialization.as_ref().ok_or_else(|| contract("original specialization constructor was not retained"))?;
        if !self.numerical.implicit.is_empty() {
            return Err(contract("portable case replay requires library-owned export of caller-supplied implicit PreparedBody capabilities; retained in-process rerun remains available"));
        }
        Ok(ReplayCaseConstructor { source, root: original.root, instance: original.instance,
            bindings: original.bindings.clone(), preparation: super::PreparationSettings { compiler: self.compiler, limits: original.limits },
            case: ReplayCaseInputs::new(&self.case), order: derivative_order(self.order),
            settings: ReplaySolverSettings::from_profile(&self.solver), declarations: self.numerical.declarations.clone(),
            targets: self.numerical.targets.clone(), seed: bits(&self.overrides.seed),
            parameters: bits(&self.overrides.parameters), fixes: bits(&self.overrides.fixes) })
    }
}
fn derivative_order(order: pse_kernels::DerivativeOrder) -> u8 {
    match order { pse_kernels::DerivativeOrder::Value => 0, pse_kernels::DerivativeOrder::First => 1,
        pse_kernels::DerivativeOrder::Second => 2 }
}
fn bits(values: &BTreeMap<SemanticId, f64>) -> BTreeMap<SemanticId, u64> {
    values.iter().map(|(id, value)| (*id, value.to_bits())).collect()
}
fn unbits(values: &BTreeMap<SemanticId, u64>) -> BTreeMap<SemanticId, f64> {
    values.iter().map(|(id, value)| (*id, f64::from_bits(*value))).collect()
}
/// Explicit structural bound override; infinity is preserved in its original IEEE bits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "bits", rename_all = "snake_case")]
pub enum ReplayBoundOverride {
    /// Preserve the authored bound.
    Keep,
    /// Explicitly remove the authored endpoint.
    Remove,
    /// Exact original endpoint, including an explicit infinity.
    Value(u64),
}
impl ReplayBoundOverride {
    fn new(value: Option<Option<f64>>) -> Self { match value {
        None => Self::Keep, Some(None) => Self::Remove, Some(Some(value)) => Self::Value(value.to_bits()) } }
    fn restore(&self) -> Option<Option<f64>> { match self {
        Self::Keep => None, Self::Remove => Some(None), Self::Value(bits) => Some(Some(f64::from_bits(*bits))) } }
}
/// Complete original fixed/free and endpoint specification for one authored path.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
pub struct ReplayCaseInputs {
    /// Values supplied by authored path.
    pub values: BTreeMap<String, u64>,
    /// Values supplied by admitted semantic member identity.
    pub members: BTreeMap<SemanticId, u64>,
    /// All structural fixed/free and bound overrides.
    pub variables: BTreeMap<String, ReplayVariableState>,
}
impl ReplayCaseInputs {
    fn new(case: &pse_compiler::workspace::ModelingCaseBindings) -> Self {
        Self { values: case.values.iter().map(|(path, value)| (path.clone(), value.to_bits())).collect(),
            members: bits(&case.members), variables: case.variables.iter().map(|(path, value)| (path.clone(), ReplayVariableState {
                fixed: value.fixed, lower: ReplayBoundOverride::new(value.lower), upper: ReplayBoundOverride::new(value.upper) })).collect() }
    }
    fn restore(&self) -> pse_compiler::workspace::ModelingCaseBindings {
        pse_compiler::workspace::ModelingCaseBindings {
            values: self.values.iter().map(|(path, value)| (path.clone(), f64::from_bits(*value))).collect(),
            members: unbits(&self.members), variables: self.variables.iter().map(|(path, value)| (path.clone(), pse_compiler::workspace::ModelingVariableState {
                fixed: value.fixed, lower: value.lower.restore(), upper: value.upper.restore() })).collect() }
    }
}
/// Portable original case constructor, independent of mutable history or compiled native products.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ReplayCaseConstructor {
    /// Index of the immutable source closure in ReplayDocument.sources.
    pub source: usize,
    /// Original selected authored definition.
    pub root: pse_modeling::DeclarationId,
    /// Original instance identity.
    pub instance: pse_modeling::InstanceId,
    /// Full immutable structural arguments/facts/omissions/elastic declarations.
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
    pub seed: BTreeMap<SemanticId, u64>,
    /// Explicit continuation parameters in exact original bits.
    pub parameters: BTreeMap<SemanticId, u64>,
    /// Explicit temporary fixes in exact original bits.
    pub fixes: BTreeMap<SemanticId, u64>,
}
impl ReplayCaseConstructor {
    /// Prepare the original typed request through its actual source/numerical admission pipeline.
    pub async fn reconstruct(&self, package: &ModelingPackage, cancel: &crate::CancelSource)
        -> Result<super::ModelingSolvePreparation, WorkflowError> {
        let order = match self.order { 0 => pse_kernels::DerivativeOrder::Value,
            1 => pse_kernels::DerivativeOrder::First, 2 => pse_kernels::DerivativeOrder::Second,
            _ => return Err(contract("replay derivative order outside actual library contract")) };
        package.prepare_solve_seed(self.root, self.instance, self.bindings.clone(), self.preparation.limits,
            self.case.restore(), order, self.preparation.compiler, self.settings.profile()?,
            crate::math::solves::NumericalInputs { declarations: self.declarations.clone(), targets: self.targets.clone(), implicit: BTreeMap::new() },
            super::modeling::cases::CaseOverrides { seed: unbits(&self.seed), parameters: unbits(&self.parameters), fixes: unbits(&self.fixes) }, cancel).await
    }
}
impl ReplaySolverSettings {
    /// Re-admit the frozen original settings using their existing production contract.
    pub fn profile(&self) -> Result<crate::math::solves::SolverProfile, WorkflowError> {
        let mut presolve_options = self.presolve_options.clone();
        let certificate_tolerance = presolve_options.remove("certify_tol");
        let mut profile = crate::math::settings::SolveSettings {
            version: pse_model::document::Version, composition: self.composition.clone(), reconstruction: self.reconstruction,
            intent: self.intent, backend: self.backend, settings: self.settings.document().cloned(), presolve: self.presolve,
            presolve_options, required_passes: self.required_passes.clone(), controls: self.controls.clone(), numerics: self.numerics.clone(),
            convexity_absolute: self.convexity_absolute, convexity_relative: self.convexity_relative, sensitivity: self.sensitivity.clone(),
        }.profile()?;
        if let Some(value) = certificate_tolerance {
            match (&mut profile.presolve, value) {
                (pse_backend_native::presolve::Policy::Explicit { options, .. }, pse_backend_native::solve::OptionValue::Real(value)) if value.is_finite() && value >= 0.0 => options.certify_tol = value,
                _ => return Err(contract("invalid actual presolve certification tolerance")),
            }
        }
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rows() -> Vec<pse_authoring::language::Declaration> {
        pse_authoring::language::parse("package p {param a:Scalar=1;def Root {var x:Scalar;eq e:x==a;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap()
    }
    #[tokio::test]
    async fn source_document_roundtrip_reconstructs_actual_production_package_without_store() {
        let runtime = crate::workflow::tests::runtime();
        let rows = rows();
        let root = rows.iter().find(|row| row.name == "Root").unwrap().declaration_id;
        let physical = crate::workflow::tests::physical();
        let package = runtime.modeling_package(rows.clone(), physical.clone()).await.unwrap();
        let document = package.replay_source().unwrap();
        let restored: ReplaySourceDocument = serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        let reconstructed = restored.reconstruct(&runtime, physical, BTreeMap::new()).unwrap();
        assert_eq!(&*reconstructed.declarations().await.unwrap(), rows.as_slice());
        assert_ne!(reconstructed.revision_id(), package.revision_id());
        let model = reconstructed.prepare(root, pse_modeling::specialize::root_instance(root),
            Default::default(), Default::default(), &crate::CancelSource::new()).await.unwrap();
        assert_eq!(model.solved().model(), root);
        assert!(model.original.is_some());
    }
    #[tokio::test]
    async fn source_reconstruction_refuses_missing_reference_changed_body_and_build() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let package = runtime.modeling_package(rows(), physical.clone()).await.unwrap();
        let original = package.replay_source().unwrap();
        let mut document = original.clone();
        document.build = pse_ids::encoding_checksum(b"different build").content_hash();
        assert!(document.reconstruct(&runtime, physical.clone(), BTreeMap::new()).is_err());
        let mut document = original.clone();
        document.objects[0].payload.push(0);
        assert!(document.reconstruct(&runtime, physical.clone(), BTreeMap::new()).is_err());
        let mut document = original;
        let referenced = document.objects.iter().find_map(|object| object.references.first()).unwrap().clone();
        document.objects.retain(|object| (object.scope.clone(), object.name.clone()) != referenced);
        assert!(document.reconstruct(&runtime, physical, BTreeMap::new()).is_err());
    }
    #[test]
    fn original_case_bits_preserve_removed_bounds_infinity_and_negative_zero() {
        let mut case = pse_compiler::workspace::ModelingCaseBindings::default();
        case.values.insert("x".into(), -0.0);
        case.variables.insert("x".into(), pse_compiler::workspace::ModelingVariableState {
            fixed: Some(false), lower: Some(None), upper: Some(Some(f64::INFINITY)) });
        let document = ReplayCaseInputs::new(&case);
        let restored: ReplayCaseInputs = serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        let restored = restored.restore();
        assert_eq!(restored.values["x"].to_bits(), (-0.0_f64).to_bits());
        assert_eq!(restored.variables["x"].lower, Some(None));
        assert_eq!(restored.variables["x"].upper, Some(Some(f64::INFINITY)));
    }
    #[test]
    fn full_profile_and_typed_structural_facts_survive_recorded_input_reconstruction() {
        let mut profile = crate::math::solves::SolverProfile::default();
        profile.controls.iterations = 17;
        profile.controls.options.insert("actual_option".into(), pse_backend_native::solve::OptionValue::Integer(4));
        let document = ReplaySolverSettings::from_profile(&profile);
        let restored = document.profile().unwrap();
        assert_eq!(crate::math::solves::profile_key(&profile).unwrap(), crate::math::solves::profile_key(&restored).unwrap());
        let mut bindings = pse_modeling::Bindings::default();
        bindings.facts.insert(pse_modeling::analysis::Fact::Stage("stage1".into()), pse_modeling::specialize::Value::Boolean(true));
        let restored: pse_modeling::Bindings = serde_json::from_slice(&serde_json::to_vec(&bindings).unwrap()).unwrap();
        assert_eq!(bindings, restored);
    }
}
