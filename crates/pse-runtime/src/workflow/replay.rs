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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaySolverSettings {
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
            pse_math::convexity::ConvexityPolicy::Numerical { absolute, relative } => (Some(absolute), Some(relative)),
        };
        Self { intent: profile.intent, backend: match profile.selection { SolverSelection::Auto => None,
                SolverSelection::Explicit(backend) => Some(backend) },
            settings: profile.backend.document().cloned(), controls: profile.controls.clone(), composition: profile.composition.clone(),
            numerics: profile.numerics.clone(), presolve: profile.presolve.kind(),
            presolve_options: profile.presolve.document_options(),
            required_passes: match &profile.presolve { pse_backend_native::presolve::Policy::Explicit { required, .. } => required.clone(), _ => BTreeSet::new() },
            convexity_absolute, convexity_relative, reconstruction: profile.reconstruction,
            sensitivity: profile.sensitivity.clone() }
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
    fn new(step: usize, trace: &crate::math::strategy::Trace) -> Self {
        Self { step, declaration: trace.declaration.clone(), original: trace.original,
            profile: trace.profile, start: trace.start, starts: trace.starts.clone(),
            decisions: trace.events.iter().map(super::attempt_documents::AttemptDecision::from).collect() }
    }
}
/// Recorded coverage, without promoting an interrupted prefix into a full trajectory.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
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
    pub version: pse_model::document::Version<1>,
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
    #[cfg(feature = "solver-diffsol")]
    pub shooting_constructor: Option<ReplayShootingConstructor>,
    /// Original physical mappings and full constructor configurations, using existing producer documents.
    pub contexts: Vec<pse_model::generated::runtime::scientific_contexts::Row>,
    /// Full original horizon provenance, including output demand, period, stages and mappings.
    pub horizon: Option<Vec<u8>>,
    /// Full retained decisions with their actual effective declaration.
    pub decisions: Vec<ReplayDecisions>,
    /// Full actual retained adapter settings, submitted starts and original coordinate witnesses.
    pub native_attempts: Vec<ReplayNativeAttempt>,
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
    pub fn inputs(&self) -> &RunRequest { &self.result.request }
    fn traces(&self) -> Box<dyn Iterator<Item = (usize, &Arc<crate::math::strategy::Trace>)> + '_> {
        let retained: Box<dyn Iterator<Item = (usize, &Arc<crate::math::strategy::Trace>)> + '_> = match &self.result.request {
            RunRequest::Modeling(steps) => Box::new((0..steps.len()).filter_map(|index| {
                self.result.modeling_result(index)?.strategy.as_ref().map(|trace| (index, trace))
            })),
            _ => match &self.result.report {
                Ok(RunReport::Fit(report)) => Box::new(report.strategy.iter().map(|trace| (0, trace))),
                #[cfg(feature = "solver-diffsol")]
                Ok(RunReport::Shooting(report)) => Box::new(report.strategy.iter().map(|trace| (0, trace))),
                _ => Box::new(std::iter::empty()),
            },
        };
        let failed = self.result.report.as_ref().err().and_then(|error| error.strategy_trace())
            .map(|trace| (self.coverage.observed, trace));
        Box::new(retained.chain(failed))
    }
    fn decision_extent(&self) -> Result<usize, WorkflowError> {
        self.traces().try_fold(512usize, |bytes, (_, trace)| {
            bytes.checked_add(trace.retained_bytes().map_err(crate::math::MathRuntimeError::from)?
                .checked_mul(4).ok_or_else(|| contract("replay decision extent"))?)
                .ok_or_else(|| contract("replay decision extent"))
        })
    }
    fn decision_rows(&self) -> Vec<ReplayDecisions> {
        self.traces().map(|(step, trace)| ReplayDecisions::new(step, trace)).collect()
    }
    /// Inspect exact original effective declarations/decisions under finite admission.
    pub fn recorded_decisions(&self) -> Result<ReplayDecisionDocument, WorkflowError> {
        let bytes = self.decision_extent()?;
        if bytes > EXPORT_MAX { return Err(contract("replay decision body exceeds finite export capacity")); }
        let owner = self.result.runtime.shared.math().reserve("replay:recorded-decisions", bytes)?;
        Ok(ReplayDecisionDocument { decisions: self.decision_rows(), _owner: owner })
    }
    fn sources(&self) -> Box<dyn Iterator<Item = &ModelingPackage> + '_> {
        match &self.result.request {
            RunRequest::Modeling(steps) => Box::new(steps.iter().map(|step| &step.source)),
            RunRequest::Simulation(simulation) => Box::new(std::iter::once(&simulation.source)),
            RunRequest::Fit(fit) => Box::new(std::iter::once(&fit.source)),
            RunRequest::Horizon(horizon) => Box::new(horizon.sources.iter()),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, .. } => Box::new(std::iter::once(&problem.simulation.source)),
        }
    }
    /// Reconstruct actual locally usable source packages into a receiving runtime.
    pub fn reconstruct_sources(&self, runtime: &Runtime) -> Result<Vec<ModelingPackage>, WorkflowError> {
        self.sources().map(|source| source.reconstruct_replay_source(runtime)).collect()
    }
    /// Default bounded complete export. Oversized closure refuses instead of truncating required inputs.
    pub fn document(&self) -> Result<ReplayDocument, WorkflowError> { self.document_with_limit(16 * 1024 * 1024) }
    /// Explicit finite export allocation. Caller receives either the complete closure or refusal.
    pub fn document_with_limit(&self, bytes: usize) -> Result<ReplayDocument, WorkflowError> {
        if bytes == 0 || bytes > 256 * 1024 * 1024 { return Err(contract("replay export requires a finite allocation up to 256 MiB")); }
        let native_extent = self.native_attempts().try_fold(0usize, |bytes, (_, native)| {
            bytes.checked_add(ReplayNativeAttempt::extent(native)?).ok_or_else(|| contract("replay native extent overflow"))
        })?;
        let extent = self.sources().try_fold(self.constructor_extent()?.saturating_add(self.decision_extent()?).saturating_add(native_extent), |total, source| {
            total.checked_add(source.replay_source_extent())
        }).ok_or_else(|| contract("replay export extent overflow"))?;
        if extent > bytes { return Err(contract("complete replay closure exceeds requested export allocation")); }
        let owner = self.result.runtime.shared.math().reserve("replay:complete-export", bytes)?;
        let sources = self.sources().map(ModelingPackage::replay_source).collect::<Result<Vec<_>, _>>()?;
        let mut runtime_premises = Vec::new();
        let mut solver_settings = Vec::new();
        let mut case_constructors = Vec::new();
        let mut simulation_constructors = Vec::new();
        let mut fit_constructor = None;
        let mut horizon_constructor = None;
        #[cfg(feature = "solver-diffsol")]
        let mut shooting_constructor = None;
        let mut contexts = Vec::new();
        let mut horizon = None;
        match &self.result.request {
            RunRequest::Modeling(steps) => for (index, step) in steps.iter().enumerate() {
                runtime_premises.push(step.solve.execution_snapshot().clone());
                solver_settings.push(ReplaySolverSettings::from_profile(&step.profile));
                case_constructors.push(step.original_inputs.document(index)?);
                contexts.push(super::artifacts::solve_context(step, self.source_run,
                    i64::try_from(index).map_err(|_| contract("replay step ordinal"))?)?);
            },
            RunRequest::Simulation(simulation) => {
                runtime_premises.push(simulation.snapshot().clone());
                simulation_constructors.push(simulation.original_inputs.as_ref().ok_or_else(|| contract("original simulation constructor absent"))?.document(0));
                contexts.push(simulation.scientific_context(self.source_run, super::dynamics::profile_identity(simulation.profile()).map_err(crate::math::MathRuntimeError::from)?.as_id())?);
            }
            RunRequest::Fit(fit) => {
                runtime_premises.push(fit.execution_snapshot().clone());
                fit_constructor = Some(fit.original_inputs.document(0));
                solver_settings.push(ReplaySolverSettings::from_profile(&fit.problem.profile.solver));
                contexts.push(fit.scientific_context(self.source_run)?);
            }
            RunRequest::Horizon(request) => {
                horizon = Some(request.provenance.clone());
                let original = &request.original;
                runtime_premises.push(original.plant.snapshot().clone());
                horizon_constructor = Some(ReplayHorizonConstructor::new(original)?);
                contexts.push(original.plant.scientific_context(self.source_run, super::dynamics::profile_identity(original.plant.profile()).map_err(crate::math::MathRuntimeError::from)?.as_id())?);
                if let Some(estimator) = &original.estimator { solver_settings.push(ReplaySolverSettings::from_profile(&estimator.analysis.solver)); }
                if let Some(controller) = &original.controller { solver_settings.push(ReplaySolverSettings::from_profile(&controller.analysis.solver)); }
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => {
                runtime_premises.push(problem.execution_snapshot().clone());
                shooting_constructor = Some(ReplayShootingConstructor::new(problem, initial)?);
                solver_settings.push(ReplaySolverSettings::from_profile(&problem.solver));
                let mut context = problem.simulation.scientific_context(self.source_run, problem.profile_key.as_id())?;
                // The exact caller initial is part of input reconstruction, not a native checkpoint.
                context.binding = Some(serde_json::to_vec(initial).map_err(|error| contract(error.to_string()))?.into());
                contexts.push(context);
            }
        }
        let document = ReplayDocument { version: pse_model::document::Version, source_run: self.source_run, continue_independent: self.result.modeling_continue_independent, runtime_premises,
            build: pse_backend_native::execution::LINKED.build_identity(), sources, solver_settings, case_constructors,
            simulation_constructors, fit_constructor, horizon_constructor,
            #[cfg(feature = "solver-diffsol")]
            shooting_constructor,
            contexts, horizon,
            decisions: self.decision_rows(), native_attempts: self.native_attempts().map(|(step, native)| ReplayNativeAttempt::new(step, native)).collect(), coverage: self.coverage.clone(), native_continuation: false, _owner: owner };
        encoded_extent(&document, bytes)?;
        Ok(document)
    }
    /// Start a new tolerant execution under fresh identity/allowance through the original supervisor.
    /// Frozen controls/inputs remain exact; scheduling, floating-point/native paths may differ.
    pub async fn rerun(&self) -> Result<RunHandle, WorkflowError> {
        self.rerun_with_cancel(&crate::CancelSource::new()).await
    }
    /// Fresh replay with caller cancellation shared by pending preparation/admission.
    pub async fn rerun_with_cancel(&self, cancel: &crate::CancelSource) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() { return Err(crate::math::MathRuntimeError::Cancelled.into()); }
        match &self.result.request {
            RunRequest::Modeling(steps) => {
                let mut fresh = Vec::with_capacity(steps.len());
                for step in steps {
                    let mut source = step.source.clone();
                    source.held_preparation = None;
                    let mut prepared = step.original_inputs.reconstruct_in_process(&source, cancel).await?;
                    // Postconstructor declarations are scientific input. Original-only graphs
                    // can detach recursively; scoped opaque products require their producer.
                    if step.solve.replay_declaration() != prepared.solve.replay_declaration() {
                        prepared.solve = step.solve.fresh_replay_copy().map_err(crate::math::MathRuntimeError::from)?;
                    }
                    if step.solve.requested_screened_start().is_some() {
                        return Err(contract("replay of a requested screened start requires producer re-screening under the fresh task; historical screening does not grant new permission"));
                    }
                    if let Some(start) = step.solve.requested_start() {
                        prepared = prepared.with_start(start.clone())?;
                    }
                    fresh.push(prepared);
                }
                self.result.runtime.start_modeling(fresh, self.result.modeling_continue_independent, cancel).await
            },
            RunRequest::Simulation(simulation) => simulation.start(),
            RunRequest::Fit(fit) => fit.start(),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Horizon(request) => {
                let mut horizon = request.original.as_ref().clone();
                // The old engagement has drained. A new execution may retain immutable
                // science, but must establish its own preparation lane.
                horizon.plant.source.held_preparation = None;
                if let Some(estimator) = &mut horizon.estimator { estimator.package.held_preparation = None; }
                if let Some(controller) = &mut horizon.controller { controller.package.held_preparation = None; }
                self.result.runtime.start_horizon(horizon, cancel).await
            },
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
    pub fn replay(self: &Arc<Self>) -> ScientificReplay {
        let (requested, observed, completed_time) = match &self.request {
            RunRequest::Modeling(steps) => {
                let mut count = 0;
                for index in 0..steps.len() { if self.modeling_result(index).is_some() {
                    count += 1;
                }}
                (steps.len(), count, None)
            }
            RunRequest::Simulation(simulation) => match &self.report {
                Ok(RunReport::Simulation(trajectory)) => (simulation.profile().samples.len(), trajectory.report().samples.len(), Some(Observation::number(trajectory.report().completed_time))),
                _ => (simulation.profile().samples.len(), 0, None),
            },
            RunRequest::Fit(_) => (1, usize::from(self.report.is_ok()), None),
            RunRequest::Horizon(request) => (request.original.steps,
                self.horizon.as_ref().map_or(0, |report| report.completed_steps), None),
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { .. } => (1, usize::from(self.report.is_ok()), None),
        };
        let terminal = match &self.report {
            Err(error) => Some(crate::math::strategy::workflow_failure(error)),
            Ok(RunReport::Simulation(trajectory)) => Some(trajectory_observation(trajectory.report())),
            Ok(RunReport::Fit(report)) => report.validation_error.as_ref().map(|diagnostic| crate::math::strategy::boundary_failure(diagnostic.class))
                .or_else(|| report.solve.as_ref().map(crate::math::strategy::observe_native)),
            #[cfg(feature = "solver-diffsol")]
            Ok(RunReport::Shooting(report)) => report.validation_error.as_ref().map(|diagnostic| crate::math::strategy::boundary_failure(diagnostic.class))
                .or_else(|| report.trajectory.as_ref().map(|trajectory| trajectory_observation(trajectory)))
                .or_else(|| report.solve.as_ref().map(crate::math::strategy::observe_native)),
            Ok(RunReport::Modeling(results)) => results.last().map(|result| crate::math::strategy::observe(&result.outcome)),
            Ok(RunReport::Horizon(report)) => report.summary_error().map(crate::math::strategy::workflow_failure)
                .or_else(|| report.detail_error().map(crate::math::strategy::workflow_failure)),
        };
        ScientificReplay { source_run: self.run_id, result: self.clone(),
            coverage: ReplayCoverage { requested, observed, completed_time, usable: self.usable(), terminal } }
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
        let bytes = size_of::<Self>().saturating_add(512)
            .saturating_add(case.values.len().saturating_mul(192))
            .saturating_add(case.members.len().saturating_mul(192))
            .saturating_add(case.variables.len().saturating_mul(256))
            .saturating_add(case.values.keys().chain(case.variables.keys()).map(String::len).sum::<usize>())
            .saturating_add(encoded_extent(&(&numerical.declarations, &numerical.targets), EXPORT_MAX)?.saturating_mul(8))
            .saturating_add(profile_extent(solver)?)
            .saturating_add(numerical.implicit.values().map(|implicit| {
                implicit.unknowns.len().saturating_mul(32)
                    .saturating_add(implicit.bounds.as_ref().map_or(0, |bounds| (bounds.lower.len() + bounds.upper.len()).saturating_mul(32)))
                    .saturating_add(match &implicit.selection { pse_math::factorable::SelectedGraph::Unestablished { meaning } => meaning.len(), _ => 0 })
                    .saturating_add(256)
            }).sum::<usize>())
            .saturating_add((overrides.seed.len() + overrides.parameters.len() + overrides.fixes.len()).saturating_mul(192));
        let owner = source.runtime.shared.math().reserve("replay:original-case-constructor", bytes)?;
        Ok(Arc::new(Self { specialization: model.original.clone(), case: case.clone(), order, compiler,
            solver: solver.clone(), numerical: numerical.clone(), overrides: overrides.clone(), _owner: owner }))
    }
    async fn reconstruct_in_process(&self, source: &ModelingPackage, cancel: &crate::CancelSource)
        -> Result<super::ModelingSolvePreparation, WorkflowError> {
        let original = self.specialization.as_ref().ok_or_else(|| contract("original specialization constructor was not retained"))?;
        source.prepare_solve_seed(original.root, original.instance, original.bindings.clone(), original.limits,
            self.case.clone(), self.order, self.compiler, self.solver.clone(), self.numerical.clone(),
            self.overrides.clone(), cancel).await
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
    fn new(value: Option<Option<f64>>) -> Self { match value {
        None => Self::Keep, Some(None) => Self::Remove, Some(Some(value)) => Self::Value(value.to_bits()) } }
    fn restore(&self) -> Option<Option<f64>> { match self {
        Self::Keep => None, Self::Remove => Some(None), Self::Value(bits) => Some(Some(f64::from_bits(*bits))) } }
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
#[serde(deny_unknown_fields)]
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
            intent: self.intent, backend: self.backend, settings: self.settings.clone(), presolve: self.presolve,
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
        let mut bindings = pse_modeling::Bindings::default();
        bindings.demand.push("a".into());
        let model = reconstructed.prepare(root, pse_modeling::specialize::root_instance(root),
            bindings, Default::default(), &crate::CancelSource::new()).await.unwrap();
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
        let mut document = original.clone();
        let edge_owner = document.objects.iter_mut().find(|object| !object.references.is_empty()).unwrap();
        edge_owner.references.push(edge_owner.references[0].clone());
        assert!(document.reconstruct(&runtime, physical.clone(), BTreeMap::new()).is_err());
        let mut document = original.clone();
        document.physical_sources.clear();
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
        assert!(document.settings.is_none());
        let bytes = serde_json::to_vec(&document).unwrap();
        let document: ReplaySolverSettings = serde_json::from_slice(&bytes).unwrap();
        let restored = document.profile().unwrap();
        assert_eq!(crate::math::solves::profile_key(&profile).unwrap(), crate::math::solves::profile_key(&restored).unwrap());
        let mut bindings = pse_modeling::Bindings::default();
        bindings.facts.insert(pse_modeling::analysis::Fact::Stage("stage1".into()), pse_modeling::specialize::Value::Boolean(true));
        let restored: pse_modeling::Bindings = serde_json::from_slice(&serde_json::to_vec(&bindings).unwrap()).unwrap();
        assert_eq!(bindings, restored);
    }
}

/// One complete original integrated constructor retained before mode preparation.
#[derive(Debug)]
pub(crate) struct OriginalSimulationInputs {
    document: ReplaySimulationConstructor,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalSimulationInputs {
    #[expect(clippy::too_many_arguments, reason = "retain the actual integrated constructor without changing its scientific request contract")]
    pub(in crate::workflow) fn capture(package: &ModelingPackage,
        root: pse_modeling::DeclarationId, instance: pse_modeling::InstanceId, bindings: &pse_modeling::Bindings,
        limits: pse_modeling::Limits, case: &pse_compiler::workspace::ModelingCaseBindings,
        compiler: pse_compiler::workspace::Profile, profile: &super::SimulationProfile,
        order: pse_kernels::DerivativeOrder, exact_parameters: &BTreeSet<SemanticId>,
        provisional_quadratures: &BTreeSet<SemanticId>) -> Result<Arc<Self>, WorkflowError> {
        let bytes = size_of::<Self>().saturating_add(1024).saturating_add(crate::math::modeling::bindings_bytes(bindings))
            .saturating_add(encoded_extent(profile, EXPORT_MAX)?.saturating_mul(8))
            .saturating_add((case.values.len() + case.members.len() + case.variables.len()).saturating_mul(512))
            .saturating_add(case.values.keys().chain(case.variables.keys()).map(String::len).sum::<usize>())
            .saturating_add((exact_parameters.len() + provisional_quadratures.len()).saturating_mul(128));
        let owner = package.runtime.shared.math().reserve("replay:original-simulation-constructor", bytes)?;
        Ok(Arc::new(Self { document: ReplaySimulationConstructor { source: 0, root, instance,
            bindings: bindings.clone(), preparation: super::PreparationSettings { compiler, limits },
            case: ReplayCaseInputs::new(case), profile: profile.clone(), order: derivative_order(order),
            exact_parameters: exact_parameters.clone(), provisional_quadratures: provisional_quadratures.clone() }, _owner: owner }))
    }
    fn document(&self, source: usize) -> ReplaySimulationConstructor {
        ReplaySimulationConstructor { source, ..self.document.clone() }
    }
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
    /// Explicit provisional quadrature demands.
    pub provisional_quadratures: BTreeSet<SemanticId>,
}
impl ReplaySimulationConstructor {
    /// Reconstruct original dynamics from source through actual compiler/physical admission.
    pub async fn reconstruct(&self, package: &ModelingPackage, cancel: &crate::CancelSource)
        -> Result<super::ModelingSimulation, WorkflowError> {
        let order = match self.order { 0 => pse_kernels::DerivativeOrder::Value, 1 => pse_kernels::DerivativeOrder::First,
            2 => pse_kernels::DerivativeOrder::Second, _ => return Err(contract("replay dynamic derivative order invalid")) };
        package.prepare_simulation_with_controls(self.root, self.instance, self.bindings.clone(),
            self.preparation.limits, self.case.restore(), self.preparation.compiler, self.profile.clone(), order,
            &self.exact_parameters, &self.provisional_quadratures, None, cancel).await
    }
}
/// Original fit preparation, retained before derivative/profile resolution.
#[derive(Debug)]
pub(crate) struct OriginalFitInputs {
    document: ReplayFitConstructor,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalFitInputs {
    pub(in crate::workflow) fn capture(package: &ModelingPackage,
        id: pse_model::generated::identities::FitId, profile: &super::FitProfile,
        compiler: pse_compiler::workspace::Profile, limits: pse_modeling::Limits) -> Result<Arc<Self>, WorkflowError> {
        let extent = size_of::<Self>().saturating_add(1024).saturating_add(profile_extent(&profile.solver)?)
            .saturating_add(encoded_extent(&(&profile.simulations, &profile.uncertainty), EXPORT_MAX)?.saturating_mul(8));
        let owner = package.runtime.shared.math().reserve("replay:original-fit-constructor", extent)?;
        Ok(Arc::new(Self { document: ReplayFitConstructor { source: 0, id,
            preparation: super::PreparationSettings { compiler, limits }, solver: ReplaySolverSettings::from_profile(&profile.solver),
            simulations: profile.simulations.clone(), rank_tolerance: profile.rank_tolerance,
            max_cells: profile.max_cells, derivatives: profile.derivatives, uncertainty: profile.uncertainty.clone() }, _owner: owner }))
    }
    fn document(&self, source: usize) -> ReplayFitConstructor { ReplayFitConstructor { source, ..self.document.clone() } }
}
/// Portable fit constructor. Exact fit/experiment declarations are in its source objects.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayFitConstructor {
    /// Source closure index.
    pub source: usize,
    /// Exact authored fit declaration.
    pub id: pse_model::generated::identities::FitId,
    /// Original compiler and expansion controls.
    pub preparation: super::PreparationSettings,
    /// Complete original native/scientific policy.
    pub solver: ReplaySolverSettings,
    /// Complete original integration controls by experiment identity.
    pub simulations: BTreeMap<pse_modeling::InstanceId, super::SimulationProfile>,
    /// Original response rank cutoff.
    pub rank_tolerance: f64,
    /// Original finite derivative/rank allowance.
    pub max_cells: usize,
    /// Original derivative evidence policy.
    pub derivatives: super::FitDerivatives,
    /// Complete requested uncertainty/propagation controls.
    pub uncertainty: Option<super::FitUncertainty>,
}
impl ReplayFitConstructor {
    /// Rebuild the authored fit/experiments through existing production admission.
    pub async fn reconstruct(&self, package: &ModelingPackage, cancel: &crate::CancelSource) -> Result<super::PreparedFit, WorkflowError> {
        package.prepare_fit(self.id, super::FitProfile { solver: self.solver.profile()?, simulations: self.simulations.clone(),
            rank_tolerance: self.rank_tolerance, max_cells: self.max_cells, derivatives: self.derivatives,
            uncertainty: self.uncertainty.clone() }, self.preparation.compiler, self.preparation.limits, cancel).await
    }
}
/// Original source-bound output/control bound in exact IEEE bits.
#[cfg(feature = "solver-diffsol")]
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
#[cfg(feature = "solver-diffsol")]
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayShootingConstructor {
    /// Complete original integrated constructor.
    pub simulation: ReplaySimulationConstructor,
    /// Single or multiple shooting.
    pub method: super::ShootingMethod,
    /// Exact requested inner node times.
    pub nodes: Vec<u64>,
    /// Original scheduled control coordinates and bounds.
    pub controls: Vec<ReplayShootingBound>,
    /// Original path output coordinates and bounds.
    pub path: Vec<ReplayShootingBound>,
    /// Original terminal objective weights.
    pub terminal: BTreeMap<SemanticId, u64>,
    /// Original integral objective weights.
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
    fn new(problem: &super::ShootingProblem, initial: &Option<Vec<f64>>) -> Result<Self, WorkflowError> {
        let request = &problem.original_inputs;
        Ok(Self { simulation: problem.simulation.original_inputs.as_ref().ok_or_else(|| contract("shooting original simulation constructor absent"))?.document(0),
            method: request.method, nodes: request.nodes.iter().map(|v| v.to_bits()).collect(),
            controls: request.controls.iter().map(|v| ReplayShootingBound { id: v.input, lower: v.lower.map(f64::to_bits), upper: v.upper.map(f64::to_bits) }).collect(),
            path: request.path.iter().map(|v| ReplayShootingBound { id: v.output, lower: v.lower.map(f64::to_bits), upper: v.upper.map(f64::to_bits) }).collect(),
            terminal: bits(&request.objective.terminal), integral: bits(&request.objective.integral),
            eager_first: request.evaluation == super::ShootingEvaluationPolicy::EagerFirst,
            solver: ReplaySolverSettings::from_profile(&request.solver), initial: initial.as_ref().map(|v| v.iter().map(|v| v.to_bits()).collect()) })
    }
    /// Rebuild original shooting through the existing simulation and shooting admission owners.
    pub async fn reconstruct(&self, package: &ModelingPackage, cancel: &crate::CancelSource) -> Result<Arc<super::ShootingProblem>, WorkflowError> {
        let simulation = self.simulation.reconstruct(package, cancel).await?;
        Ok(Arc::new(simulation.shooting(super::ShootingProfile { method: self.method,
            nodes: self.nodes.iter().map(|v| f64::from_bits(*v)).collect(),
            controls: self.controls.iter().map(|v| super::ShootingControl { input: v.id, lower: v.lower.map(f64::from_bits), upper: v.upper.map(f64::from_bits) }).collect(),
            path: self.path.iter().map(|v| super::PathBound { output: v.id, lower: v.lower.map(f64::from_bits), upper: v.upper.map(f64::from_bits) }).collect(),
            objective: super::ShootingObjective { terminal: unbits(&self.terminal), integral: unbits(&self.integral) },
            evaluation: if self.eager_first { super::ShootingEvaluationPolicy::EagerFirst } else { super::ShootingEvaluationPolicy::Demanded },
            solver: self.solver.profile()? })?))
    }
}

impl ReplayCaseConstructor {
    fn from_analysis(source: usize, analysis: &super::ModelingAnalysis) -> Result<Self, WorkflowError> {
        if !analysis.numerical.implicit.is_empty() { return Err(contract("portable analysis replay requires library-owned export of caller-supplied implicit capabilities")); }
        Ok(Self { source, root: analysis.root, instance: analysis.instance, bindings: analysis.bindings.clone(),
            preparation: super::PreparationSettings { compiler: analysis.compiler, limits: analysis.limits },
            case: ReplayCaseInputs::new(&analysis.case), order: derivative_order(analysis.order),
            settings: ReplaySolverSettings::from_profile(&analysis.solver), declarations: analysis.numerical.declarations.clone(),
            targets: analysis.numerical.targets.clone(), seed: BTreeMap::new(), parameters: BTreeMap::new(), fixes: BTreeMap::new() })
    }
    fn analysis(&self) -> Result<super::ModelingAnalysis, WorkflowError> {
        let order = match self.order { 0 => pse_kernels::DerivativeOrder::Value, 1 => pse_kernels::DerivativeOrder::First,
            2 => pse_kernels::DerivativeOrder::Second, _ => return Err(contract("replay analysis derivative order invalid")) };
        Ok(super::ModelingAnalysis { root: self.root, instance: self.instance, bindings: self.bindings.clone(),
            limits: self.preparation.limits, case: self.case.restore(), order, compiler: self.preparation.compiler,
            solver: self.settings.profile()?, numerical: crate::math::solves::NumericalInputs {
                declarations: self.declarations.clone(), targets: self.targets.clone(), implicit: BTreeMap::new() } })
    }
}
/// Exact original horizon signal, preserving all trajectory value bits.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
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
    fn new(value: &super::HorizonSignal) -> Self { match value {
        super::HorizonSignal::Measured(id) => Self::Measured(*id), super::HorizonSignal::Estimated(path) => Self::Estimated(path.clone()),
        super::HorizonSignal::Applied(input) => Self::Applied(*input), super::HorizonSignal::Trajectory(values) => Self::Trajectory(values.iter().map(|v| v.to_bits()).collect()) } }
    fn restore(&self) -> super::HorizonSignal { match self {
        Self::Measured(id) => super::HorizonSignal::Measured(*id), Self::Estimated(path) => super::HorizonSignal::Estimated(path.clone()),
        Self::Applied(input) => super::HorizonSignal::Applied(*input), Self::Trajectory(values) => super::HorizonSignal::Trajectory(values.iter().map(|v| f64::from_bits(*v)).collect()) } }
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
        let mut next = 1;
        let estimator = if let Some(estimator) = &horizon.estimator {
            let analysis = ReplayCaseConstructor::from_analysis(next, &estimator.analysis)?; next += 1;
            Some(ReplayHorizonEstimator { analysis, window: estimator.window, measurements: estimator.measurements.clone(),
                inputs: estimator.inputs.iter().map(|(input, binding)| match binding { super::WindowInput::Constant(path) => (*input, false, vec![path.clone()]),
                    super::WindowInput::Periods(paths) => (*input, true, paths.clone()) }).collect(),
                arrival: estimator.arrival.iter().map(|v| (v.prior.clone(), v.next.clone(), v.initial.to_bits())).collect() })
        } else { None };
        let controller = if let Some(controller) = &horizon.controller {
            Some(ReplayHorizonController { analysis: ReplayCaseConstructor::from_analysis(next, &controller.analysis)?,
                bindings: controller.bindings.iter().map(|(path, value)| (path.clone(), ReplayHorizonSignal::new(value))).collect(),
                moves: controller.moves.clone(), predictions: controller.advanced.as_ref().map(|v| v.predictions.clone()) })
        } else { None };
        Ok(Self { output: horizon.output.clone(), plant: horizon.plant.original_inputs.as_ref()
            .ok_or_else(|| contract("original horizon plant constructor absent"))?.document(0),
            period: horizon.period.to_bits(), steps: horizon.steps, inputs: horizon.inputs.iter().map(|v| (v.parameter, v.initial.to_bits())).collect(),
            estimator, controller })
    }
    /// Rebuild the exact original plant and stages from admitted source packages.
    pub async fn reconstruct(&self, sources: &[ModelingPackage], cancel: &crate::CancelSource) -> Result<super::Horizon, WorkflowError> {
        let source = |index| sources.get(index).ok_or_else(|| contract("replay horizon source closure absent"));
        let plant = self.plant.reconstruct(source(self.plant.source)?, cancel).await?;
        let estimator = if let Some(estimator) = &self.estimator {
            let mut inputs = Vec::new();
            for (input, periods, paths) in &estimator.inputs {
                let binding = if *periods { super::WindowInput::Periods(paths.clone()) } else {
                    if paths.len() != 1 { return Err(contract("replay constant window input extent")); }
                    super::WindowInput::Constant(paths[0].clone()) };
                inputs.push((*input, binding));
            }
            Some(super::HorizonEstimator { package: source(estimator.analysis.source)?.clone(), analysis: estimator.analysis.analysis()?,
                window: estimator.window, measurements: estimator.measurements.clone(), inputs,
                arrival: estimator.arrival.iter().map(|(prior, next, initial)| super::Arrival { prior: prior.clone(), next: next.clone(), initial: f64::from_bits(*initial) }).collect() })
        } else { None };
        let controller = if let Some(controller) = &self.controller {
            Some(super::HorizonController { package: source(controller.analysis.source)?.clone(), analysis: controller.analysis.analysis()?,
                bindings: controller.bindings.iter().map(|(path, value)| (path.clone(), value.restore())).collect(), moves: controller.moves.clone(),
                advanced: controller.predictions.as_ref().map(|predictions| super::AdvancedStep { predictions: predictions.clone() }) })
        } else { None };
        Ok(super::Horizon { output: self.output.clone(), plant, period: f64::from_bits(self.period), steps: self.steps,
            inputs: self.inputs.iter().map(|(parameter, initial)| super::HorizonInput { parameter: *parameter, initial: f64::from_bits(*initial) }).collect(), estimator, controller })
    }
}

const EXPORT_MAX: usize = 256 * 1024 * 1024;
/// Counting serialization admits copies without allocating a second serialized document.
struct ExtentWriter { written: usize, limit: usize }
impl std::io::Write for ExtentWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let total = self.written.checked_add(bytes.len()).filter(|total| *total <= self.limit)
            .ok_or_else(|| std::io::Error::other("complete replay extent exceeds finite allocation"))?;
        self.written = total;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}
fn encoded_extent(value: &impl serde::Serialize, limit: usize) -> Result<usize, WorkflowError> {
    let mut writer = ExtentWriter { written: 0, limit };
    serde_json::to_writer(&mut writer, value).map_err(|error| contract(format!("replay encoding extent: {error}")))?;
    Ok(writer.written)
}
pub(in crate::workflow) fn profile_extent(profile: &crate::math::solves::SolverProfile) -> Result<usize, WorkflowError> {
    // Native option export is fixed-size (21 entries); variable scientific/advice/settings
    // bodies are borrowed during this count, before any clone/allocation.
    let options = profile.presolve.document_options();
    let required = match &profile.presolve { pse_backend_native::presolve::Policy::Explicit { required, .. } => Some(required), _ => None };
    encoded_extent(&(&profile.controls, &profile.composition, &profile.numerics,
        profile.backend.document(), &profile.sensitivity, &profile.reconstruction, options, required), EXPORT_MAX)
        .map(|bytes| bytes.saturating_mul(8).saturating_add(size_of::<ReplaySolverSettings>() + 1024))
}

fn trajectory_observation(report: &pse_backend_native::dynamics::Report) -> pse_model::generated::enums::NumericalAttemptObservation {
    use pse_backend_native::dynamics::Termination as T;
    use pse_model::generated::enums::NumericalAttemptObservation as O;
    if let Some(error) = &report.error { return crate::math::strategy::failure(error); }
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
        let original = match &self.result.request {
            RunRequest::Modeling(steps) => steps.iter().try_fold(1024usize, |bytes, step| {
                let inputs = &step.original_inputs;
                let specialization = inputs.specialization.as_ref().map_or(0, |original| original.retained_bytes());
                bytes.checked_add(inputs._owner.size()).and_then(|bytes| bytes.checked_add(specialization))
                    .and_then(|bytes| bytes.checked_add(profile_extent(&step.profile).ok()?))
                    .and_then(|bytes| bytes.checked_add(encoded_extent(&(&step.starts, &step.model.values.scalars), EXPORT_MAX).ok()?.saturating_mul(8)))
                    .and_then(|bytes| bytes.checked_add(step.bound_source.as_ref().map_or(Some(0), |binding| encoded_extent(&***binding, EXPORT_MAX).ok().map(|bytes| bytes.saturating_mul(8)))?))
                    .ok_or_else(|| contract("complete case replay extent overflow"))
            })?,
            RunRequest::Simulation(simulation) => simulation.original_inputs.as_ref().map_or(1024, |inputs| inputs._owner.size()),
            RunRequest::Fit(fit) => fit.original_inputs._owner.size().saturating_add(profile_extent(&fit.problem.profile.solver)?),
            RunRequest::Horizon(request) => {
                let horizon = &request.original;
                let mut bytes = horizon.plant.original_inputs.as_ref().map_or(1024, |inputs| inputs._owner.size())
                    .saturating_add(request.provenance.len()).saturating_add(horizon.inputs.len().saturating_mul(128));
                for analysis in horizon.estimator.iter().map(|stage| &stage.analysis)
                    .chain(horizon.controller.iter().map(|stage| &stage.analysis)) {
                    bytes = bytes.saturating_add(crate::math::modeling::bindings_bytes(&analysis.bindings))
                        .saturating_add(profile_extent(&analysis.solver)?)
                        .saturating_add(encoded_extent(&(&analysis.numerical.declarations, &analysis.numerical.targets), EXPORT_MAX)?.saturating_mul(8))
                        .saturating_add((analysis.case.values.len() + analysis.case.members.len() + analysis.case.variables.len()).saturating_mul(512))
                        .saturating_add(analysis.case.values.keys().chain(analysis.case.variables.keys()).map(String::len).sum::<usize>());
                }
                if let Some(estimator) = &horizon.estimator {
                    bytes = bytes.saturating_add(estimator.measurements.iter().map(|(_, paths)| paths.iter().map(|path| path.len() + 128).sum::<usize>()).sum::<usize>())
                        .saturating_add(estimator.inputs.iter().map(|(_, input)| match input { super::WindowInput::Constant(path) => path.len() + 128,
                            super::WindowInput::Periods(paths) => paths.iter().map(|path| path.len() + 128).sum() }).sum::<usize>())
                        .saturating_add(estimator.arrival.iter().map(|arrival| arrival.prior.len() + arrival.next.len() + 256).sum::<usize>());
                }
                if let Some(controller) = &horizon.controller {
                    bytes = bytes.saturating_add(controller.bindings.iter().map(|(path, signal)| path.len() + 128 + match signal {
                        super::HorizonSignal::Estimated(path) => path.len(), super::HorizonSignal::Trajectory(values) => values.len().saturating_mul(16), _ => 0 }).sum::<usize>())
                        .saturating_add(controller.moves.iter().map(|(path, _)| path.len() + 128).sum::<usize>())
                        .saturating_add(controller.advanced.as_ref().map_or(0, |advanced| advanced.predictions.iter().map(|(a, b)| a.len() + b.len() + 256).sum()));
                }
                bytes
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, initial } => problem.replay_constructor_extent()
                .saturating_add(problem.simulation.original_inputs.as_ref().map_or(1024, |inputs| inputs._owner.size()))
                .saturating_add(initial.as_ref().map_or(0, |values| values.len().saturating_mul(16)))
                .saturating_add(profile_extent(&problem.solver)?),
        };
        original.checked_mul(8).and_then(|bytes| bytes.checked_add(65536))
            .ok_or_else(|| contract("complete replay constructor extent overflow"))
    }
}

pub(in crate::workflow) fn source_reconstruction_extent(document: &ReplaySourceDocument) -> Result<usize, WorkflowError> {
    encoded_extent(document, EXPORT_MAX)?.checked_mul(8).and_then(|bytes| bytes.checked_add(65536))
        .filter(|bytes| *bytes <= EXPORT_MAX)
        .ok_or_else(|| contract("complete replay source reconstruction exceeds finite capacity"))
}

/// Actual submitted original-space start, distinct from an output seed or a native checkpoint.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaySubmittedStart {
    /// Original predecessor attempt selection.
    pub previous_attempt: Option<usize>,
    /// Full actual submitted source-coordinate seed.
    pub seed: Option<pse_backend_native::solve::WarmStartSnapshot>,
    /// Full actual partial coordinate start in exact original bits.
    pub sparse_seed: Option<BTreeMap<SemanticId, u64>>,
    /// Complete transformations actually applied before API submission.
    pub transformations: Vec<pse_backend_native::solve::SeedTransformation>,
    /// Actual API submission; native internal use remains separately observed.
    pub submitted: bool,
}
/// Actual adapter choices/options/start and producer evidence for one retained native attempt.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayNativeAttempt {
    /// Original modeling step, or zero for fit/shooting.
    pub step: usize,
    /// Actual executed backend.
    pub backend: Backend,
    /// Full original variable correspondence.
    pub variables: Vec<SemanticId>,
    /// Full original row correspondence.
    pub rows: Vec<SemanticId>,
    /// Complete actual adapter options, separate from queried defaults.
    pub options: pse_backend_native::solve::Options,
    /// Defaults only when supplied by a native getter.
    pub native_defaults: pse_backend_native::solve::Options,
    /// Actual owner-emitted build/runtime/numerical provenance, preserved without parsing.
    pub provenance: BTreeMap<String, String>,
    /// Actual submitted source-coordinate start and transforms.
    pub start: Option<ReplaySubmittedStart>,
    /// Actual bounded native counters/phase/stop coverage.
    pub observations: Option<Arc<pse_backend_native::solve::NativeObservations>>,
    /// Full retained bounded raw map history, with its exact omissions/sample roles.
    pub raw_map: Option<Arc<pse_backend_native::solve::RawMapHistory>>,
}
impl ReplayNativeAttempt {
    fn new(step: usize, native: &pse_backend_native::solve::SolveReport) -> Self {
        Self { step, backend: native.backend, variables: native.variables.clone(), rows: native.rows.clone(),
            options: native.options.clone(), native_defaults: native.native_defaults.clone(), provenance: native.provenance.clone(),
            start: native.start_receipt.as_ref().map(|start| ReplaySubmittedStart {
                previous_attempt: start.previous_attempt, seed: start.seed.as_ref().map(pse_backend_native::solve::WarmStart::snapshot),
                sparse_seed: start.sparse_seed.as_ref().map(bits), transformations: start.transformations.clone(), submitted: start.submitted }),
            observations: native.evidence.native_observations.clone(), raw_map: native.evidence.raw_map.clone() }
    }
    fn extent(native: &pse_backend_native::solve::SolveReport) -> Result<usize, WorkflowError> {
        let mut bytes = encoded_extent(&(&native.options, &native.native_defaults, &native.provenance), EXPORT_MAX)?.saturating_mul(8)
            .saturating_add((native.variables.len() + native.rows.len()).saturating_mul(1024)).saturating_add(16384);
        if let Some(start) = &native.start_receipt {
            bytes = bytes.saturating_add(start.sparse_seed.as_ref().map_or(0, |seed| seed.len().saturating_mul(192)))
                .saturating_add(encoded_extent(&start.transformations, EXPORT_MAX)?.saturating_mul(8));
            if let Some(seed) = &start.seed {
                use pse_backend_native::solve::WarmPayload as W;
                let count = match &seed.payload {
                    W::Root(values) => values.len(),
                    W::Nlp { primal, bounds, rows, .. } => primal.len() + bounds.as_ref().map_or(0, |(a, b)| a.len() + b.len()) + rows.as_ref().map_or(0, Vec::len),
                    W::Highs { primal, dual, .. } => primal.as_ref().map_or(0, Vec::len) + dual.as_ref().map_or(0, |(a, b)| a.len() + b.len()),
                };
                bytes = bytes.saturating_add(count.saturating_mul(128));
            }
        }
        if let Some(raw) = &native.evidence.raw_map {
            bytes = bytes.saturating_add(raw.samples.iter().chain(raw.last_successful.iter()).chain(raw.last_evaluation.iter())
                .map(|sample| sample.values.len().saturating_mul(256).saturating_add(512)).sum::<usize>());
        }
        Ok(bytes)
    }
}
impl ScientificReplay {
    fn native_attempts(&self) -> Box<dyn Iterator<Item = (usize, &pse_backend_native::solve::SolveReport)> + '_> {
        match &self.result.request {
            RunRequest::Modeling(steps) => Box::new((0..steps.len()).filter_map(|index| {
                let result = self.result.modeling_result(index)?;
                match &result.outcome { crate::math::solves::Outcome::Native(native) => Some((index, native.as_ref())), _ => None }
            })),
            _ => match &self.result.report {
                Ok(RunReport::Fit(report)) => Box::new(report.solve.iter().map(|native| (0, native))),
                #[cfg(feature = "solver-diffsol")]
                Ok(RunReport::Shooting(report)) => Box::new(report.solve.iter().map(|native| (0, native))),
                _ => Box::new(std::iter::empty()),
            },
        }
    }
}
