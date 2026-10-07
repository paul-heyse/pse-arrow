// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Integrated time consumes generated coordinates and the existing function oracle.
#[path = "dynamics_accuracy.rs"]
mod accuracy;
#[path = "dynamics_checks.rs"]
pub(in crate::workflow) mod checks;
#[cfg(feature = "solver-idas")]
#[path = "consistent.rs"]
pub(super) mod consistent;
#[path = "dynamics_events.rs"]
mod events;
#[path = "trajectory.rs"]
mod trajectory;
use super::*;
use crate::workflow::dynamics::{
    CoordinateBinding, DynamicCoordinates, DynamicMode, DynamicWorker, FunctionProgram,
    GuardProgram, RangeCheck, RangeValue,
};
use crate::workflow::math;
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function},
};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingHint, ModelingOutput, Profile};
use pse_ids::{ContentHash, FramedHasher};
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use pse_model::generated::enums::{NumericalSource, NumericalTarget};
use pse_model::generated::identities::RunId;
use std::{collections::BTreeSet, sync::Arc};

/// Authored integration controls retain the factory's provisional physical slots until
/// bound engineering context is frozen. Explicit profiles never acquire these slots.
pub(in crate::workflow) struct AuthoredIntegrationControls {
    pub(in crate::workflow) profile: native::Profile,
    pub(in crate::workflow) provisional_quadratures: BTreeSet<SemanticId>,
}
/// Immutable generated simulation. No authored state/RHS declaration is introduced.
#[derive(Clone, Debug)]
pub struct ModelingSimulation {
    /// The model, case and instance it simulates (`pse_model::lineage`).
    pub(in crate::workflow) solved: pse_model::lineage::Solved,
    quantities: Arc<pse_quantity::QuantityRegistry>,
    modes: Vec<SimulationMode>,
    contract: native::Contract,
    #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
    state_positions: BTreeMap<SemanticId, usize>,
    #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
    output_positions: BTreeMap<SemanticId, usize>,
    profile: native::Profile,
    snapshot: pse_backend_native::execution::Snapshot,
    pub(in crate::workflow) runtime: Runtime,
    pub(in crate::workflow) source: ModelingPackage,
    programs: Arc<[FunctionProgram]>,
    coordinates: DynamicCoordinates,
    pub(in crate::workflow) parameters: Vec<f64>,
    key: ContentHash,
    pub(in crate::workflow) bytes: usize,
}
#[derive(Clone, Debug)]
struct SimulationMode {
    name: String,
    model: ModelingPreparation,
    numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
    context: DynamicMode,
    check_program: Option<Arc<crate::math::ExecutableCase>>,
    terminal_check_program: Option<Arc<crate::math::ExecutableCase>>,
    sample_scope: Option<results::AssessmentScope>,
    original_initial_conditions: Vec<OriginalInitialCondition>,
}
#[derive(Clone, Debug)]
struct OriginalInitialCondition {
    coordinate: usize,
    row: SemanticId,
    source: DeclarationId,
    expected: f64,
    tolerance: f64,
}
/// Clone-shared immutable scientific completion and lazy transport.
#[derive(Clone, Debug)]
pub struct ModelingTrajectory {
    inner: Arc<TrajectorySnapshot>,
}
#[derive(Debug)]
struct TrajectorySnapshot {
    run_id: RunId,
    report: Arc<native::Report>,
    prepared: ModelingSimulation,
    checks: Vec<ModelingCheck>,
    reports: Vec<ModelingReport>,
    checks_complete: bool,
    completion: crate::workflow::numerics::Completed,
    validation_error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    header: pse_model::generated::runtime::computation_runs::Row,
    coverage: native::EndpointAssessment,
    assessment: pse_model::generated::runtime::candidate_assessments::Row,
    _owner: Arc<pse_columnar::AllocationLease>,
    tables: Mutex<Option<Arc<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>>>>,
}
impl ModelingTrajectory {
    /// Unique identity of this completed attempt.
    pub fn run_id(&self) -> RunId {
        self.inner.run_id
    }
    /// Borrow the native outcome without permitting mutation beneath completion.
    pub fn report(&self) -> &native::Report {
        &self.inner.report
    }
    /// Borrow the preparation that produced this outcome.
    pub fn prepared(&self) -> &ModelingSimulation {
        &self.inner.prepared
    }
    /// Sample obligations retained by completion.
    pub fn checks(&self) -> &[ModelingCheck] {
        &self.inner.checks
    }
    /// Whole-domain reports retained by completion.
    pub fn reports(&self) -> &[ModelingReport] {
        &self.inner.reports
    }
    /// Whether requested sample checks were evaluated.
    pub fn checks_complete(&self) -> bool {
        self.inner.checks_complete
    }
    /// Permission derived exclusively from the joined completion.
    pub fn accepted(&self) -> bool {
        self.inner.completion.permits_use()
    }
    /// Retained typed validation cause, if any.
    pub fn validation_error(&self) -> Option<&pse_model::diagnostic::BoundaryDiagnostic> {
        self.inner.validation_error.as_ref()
    }
    pub(in crate::workflow) fn completion(&self) -> &crate::workflow::numerics::Completed {
        &self.inner.completion
    }
    pub(in crate::workflow) fn header(
        &self,
    ) -> &pse_model::generated::runtime::computation_runs::Row {
        &self.inner.header
    }

    pub(in crate::workflow) fn assessment(
        &self,
    ) -> &pse_model::generated::runtime::candidate_assessments::Row {
        &self.inner.assessment
    }

    pub(super) fn sample_context(
        &self,
        sample: &native::Sample,
    ) -> Result<
        (
            ModelingPreparation,
            CaseValues,
            BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        ),
        WorkflowError,
    > {
        let mode = self
            .inner
            .prepared
            .modes
            .get(sample.mode)
            .ok_or_else(|| contract("trajectory sample mode absent"))?;
        let mut values = mode.context.values.clone();
        self.inner.prepared.update_sample_values(
            sample,
            &self.inner.prepared.parameters,
            &mut values,
        )?;
        Ok((mode.model.clone(), values, mode.context.providers.clone()))
    }
    pub(super) async fn derivative_samples(
        &self,
        policy: pse_backend_native::derivative_diagnostics::Policy,
        controls: pse_backend_native::solve::Controls,
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            Vec<(i64, f64, bool, bool, String)>,
            Arc<pse_columnar::AllocationLease>,
        ),
        WorkflowError,
    > {
        let trajectory = self.clone();
        let bytes = policy
            .allowance()
            .map_err(crate::math::MathRuntimeError::from)?
            .checked_add(
                self.inner
                    .report
                    .samples
                    .len()
                    .checked_mul(1024)
                    .ok_or_else(|| contract("derivative sample extent"))?,
            )
            .ok_or_else(|| contract("derivative sample extent"))?;
        let handle = self.inner.prepared.runtime.shared.math().submit(1, bytes, move |flag, _| {
            let execution = pse_backend_native::solve::Execution::new(flag.clone(), &controls);
            let scope = execution.scope()?;
            let mut outcomes = Vec::new();
            for (index, sample) in trajectory.inner.report.samples.iter().enumerate() {
                let p = &trajectory.inner.prepared;
                let parameters = p.profile.parameters_at(&p.parameters, sample.time);
                // The box the mode's range obligations admit, so a difference step stays
                // inside it: one-sided at an input or state held at its bound.
                let bounds = p.worker(scope.clone())?.coordinate_box(sample.mode, sample.time, &sample.state, &parameters)?;
                let mut functions = vec![(Function::Rhs, p.contract.states.len()), (Function::Output, p.contract.outputs.len())];
                if !p.contract.quadratures.is_empty() { functions.push((Function::QuadratureFlux, p.contract.quadratures.len())); }
                if !p.contract.balances.is_empty() { functions.push((Function::Inventory, p.contract.balances.len())); }
                let mut complete = true; let mut passed = true; let mut checked = 0; let mut suspicious = 0; let mut missing = 0; let mut details = Vec::new();
                for (function, rows) in functions {
                    let mut normalization = pse_math::normalization::Normalization::identity(p.contract.states.len() + p.contract.parameters.len(), rows);
                    normalization.variables[p.contract.states.len()..].copy_from_slice(&p.profile.parameter_scales);
                    let result = pse_backend_native::derivative_diagnostics::analyze_dynamic(
                        Box::new(p.worker(scope.clone())?),
                        pse_backend_native::derivative_diagnostics::DynamicSample { mode: sample.mode, function, time: sample.time, state: sample.state.clone(), parameters: parameters.clone(), bounds: bounds.clone() },
                        normalization, policy, execution.clone(),
                    )?;
                    if !result.passed() { details.push(format!("{function:?}: {}", result.summary())); }
                    complete &= result.complete; passed &= result.passed();
                    if let Some(sample) = &result.sample { checked += sample.checked; suspicious += sample.suspicious; missing += sample.missing_structure; }
                }
                outcomes.push((index as i64, sample.time, complete, passed, format!("{checked} comparisons; {suspicious} suspicious entries; {missing} missing sparsity entries; fixed time and mode\n{}", details.join("\n"))));
            }
            let retained = outcomes.capacity() * size_of::<(i64, f64, bool, bool, String)>() + outcomes.iter().map(|o| o.4.capacity()).sum::<usize>();
            Ok((outcomes, retained))
        })?;
        let control = handle.cancellation();
        let finish = handle.finish();
        tokio::pin!(finish);
        Ok(
            tokio::select! { result = &mut finish => result?, () = cancel.cancelled() => { control.cancel(); finish.await? } },
        )
    }
    /// The typed failed qualification or native interruption, without changing termination.
    pub fn diagnostic(&self) -> Option<pse_model::diagnostic::BoundaryDiagnostic> {
        use pse_model::diagnostic::{BoundaryClass as C, BoundaryDiagnostic as D, Observation};
        if self.accepted() {
            return None;
        }
        if let Some(error) = &self.inner.validation_error {
            return Some(error.clone());
        }
        if let Some(error) = &self.inner.report.error {
            return Some(super::super::diagnostics::observed(
                error,
                pse_diagnostics::DiagnosticStage::ModelingTrajectory,
            ));
        }
        let class = match self.inner.report.termination {
            native::Termination::Cancelled => C::Cancelled,
            native::Termination::StepLimit | native::Termination::TimeLimit => C::ResourceLimit,
            _ => C::TrialRejected,
        };
        let mut error = D::new(
            class,
            pse_diagnostics::DiagnosticStage::ModelingTrajectory,
            self.inner
                .checks
                .iter()
                .filter(|c| !c.satisfied)
                .map(|c| c.source_id.as_id()),
            pse_diagnostics::DiagnosticRule::ModelingTrajectoryRejected,
        );
        error.observations.insert(
            "termination".into(),
            Observation::Text(self.inner.report.termination.as_str().into()),
        );
        error.observations.insert(
            "completed_time".into(),
            Observation::number(self.inner.report.completed_time),
        );
        Some(error)
    }
}
/// A submitted integration: its native report and sample checks, with the admission
/// lease that holds its memory until the result is consumed.
type DynamicsHandle = crate::math::solves::SolveHandle<(
    (
        native::Report,
        checks::SampleChecks,
        Vec<pse_math::engineering_accuracy::GoalResult>,
    ),
    Arc<pse_columnar::AllocationLease>,
)>;
impl ModelingSimulation {
    /// Project a coherent joined shooting product, retaining its final optimizer header.
    #[cfg(feature = "solver-diffsol")]
    pub(in crate::workflow) fn completed_trajectory(
        &self,
        report: &crate::workflow::ShootingReport,
        header: pse_model::generated::runtime::computation_runs::Row,
        assessment: pse_model::generated::runtime::candidate_assessments::Row,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        let trajectory = report
            .trajectory
            .as_ref()
            .ok_or_else(|| contract("shooting trajectory absent"))?;
        let coverage = report
            .endpoint_assessment
            .clone()
            .ok_or_else(|| contract("shooting endpoint assessment absent"))?;
        Ok(ModelingTrajectory {
            inner: Arc::new(TrajectorySnapshot {
                run_id: header.run_id,
                report: trajectory.clone(),
                prepared: self.clone(),
                checks: report.checks.clone(),
                reports: report.reports.clone(),
                checks_complete: report.checks_complete,
                completion: report.completion.clone(),
                validation_error: report.validation_error.clone(),
                header,
                coverage,
                assessment,
                _owner: owner,
                tables: Mutex::new(None),
            }),
        })
    }
    /// Required physical obligations from the admitted model, even when evaluation is absent.
    pub(in crate::workflow) fn required_closure_checks(&self) -> usize {
        self.contract.balances.len()
            + self
                .modes
                .iter()
                .map(|mode| mode.model.compiled().model.required_closure_checks())
                .max()
                .unwrap_or(0)
    }
    /// Content identity of the simulation's contract, programs and modes.
    pub fn identity(&self) -> ContentHash {
        self.key
    }
    /// The initial mode's model view.
    pub fn model(&self) -> &ModelingPreparation {
        &self.modes[0].model
    }
    /// Declared mode names in native execution order; callers never persist native indices.
    pub fn mode_names(&self) -> impl Iterator<Item = &str> {
        self.modes.iter().map(|m| m.name.as_str())
    }
    /// Native dynamic contract: states, parameters, outputs and events.
    pub fn contract(&self) -> &native::Contract {
        &self.contract
    }
    /// The integration parameter values: the contract's unscheduled parameters in
    /// declared order, then every interval of every scheduled input
    /// ([`native::Profile::parameters_at`] gives the contract values at a time).
    pub fn parameters(&self) -> &[f64] {
        &self.parameters
    }
    /// Native integration profile.
    pub fn profile(&self) -> &native::Profile {
        &self.profile
    }
    /// Resolved numerical policy of the initial mode.
    pub fn numerics(&self) -> &pse_model::numerics::ResolvedNumericalPolicy {
        &self.modes[0].numerics
    }
    pub(crate) fn worker(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<DynamicWorker, ProblemError> {
        self.program().worker(scope)
    }
    pub(crate) fn snapshot(&self) -> &pse_backend_native::execution::Snapshot {
        &self.snapshot
    }
    pub(crate) fn program(&self) -> crate::workflow::dynamics::DynamicProgram {
        crate::workflow::dynamics::DynamicProgram {
            contract: self.contract.clone(),
            programs: self.programs.clone(),
            coordinates: self.coordinates.clone(),
            modes: self
                .modes
                .iter()
                .map(|m| m.context.clone())
                .collect::<Vec<_>>(),
            max_cells: self.profile.max_cells,
        }
    }
    /// The shooting request an authored `procedure shooting` fixture of `instance` declares
    /// (ADR-0110 Outcome 5): its method and nodes, its schedules held free as the
    /// controls, and the model's one objective level, minimized. A member that names a
    /// declared integral weighs that quadrature over the horizon; any other member is an
    /// observed output weighed at the horizon's end. Path bounds are not authored: the
    /// model's bounds remain the guard's.
    #[cfg(feature = "solver-diffsol")]
    pub(in crate::workflow) fn authored_shooting(
        &self,
        instance: InstanceId,
        solver: crate::math::solves::SolverProfile,
    ) -> Result<crate::workflow::ShootingProfile, WorkflowError> {
        use crate::workflow::{ShootingControl, ShootingObjective, ShootingProfile};
        let product = self.model().compiled();
        let fixture = product
            .model
            .fixtures
            .get(&instance)
            .ok_or_else(|| contract("shooting fixture absent"))?;
        let (Some(shooting), Some(integration)) = (&fixture.shooting, &fixture.integration) else {
            return Err(contract("the fixture declares no shooting"));
        };
        let controls = integration
            .schedules
            .iter()
            .filter_map(|s| {
                s.control.map(|bounds| ShootingControl {
                    input: s.target,
                    lower: bounds.lower,
                    upper: bounds.upper,
                })
            })
            .collect();
        let objectives = &product.model.objectives;
        let mut objective = ShootingObjective::default();
        match objectives.levels.as_slice() {
            [] => {}
            [level] => {
                let sign = if level.sense == pse_modeling::annotation::ObjectiveSense::Maximize {
                    -1.
                } else {
                    1.
                };
                for member in objectives.members_of(level) {
                    let mut weight = sign * member.scale;
                    if let Some(normalization) = member.normalization {
                        let value = self.modes[0]
                            .context
                            .values
                            .scalars
                            .get(&normalization)
                            .copied()
                            .filter(|v| v.is_finite() && *v > 0.)
                            .ok_or_else(|| contract("objective normalization value"))?;
                        weight /= value;
                    }
                    if let Some(quadrature) = product.model.integral_of(member.target) {
                        *objective.integral.entry(quadrature).or_default() += weight;
                    } else {
                        let output = ModelingOutput::Member(member.target).row_id();
                        if !self.contract.outputs.contains(&output) {
                            return Err(contract(
                                "a shooting objective member is a declared integral or an observed output",
                            ));
                        }
                        *objective.terminal.entry(output).or_default() += weight;
                    }
                }
            }
            _ => {
                return Err(contract(
                    "shooting minimizes one objective level, not lexicographic levels",
                ));
            }
        }
        Ok(ShootingProfile {
            method: shooting.method,
            nodes: shooting
                .nodes
                .iter()
                .map(|t| t * self.coordinates.time_scale)
                .collect(),
            controls,
            path: Vec::new(),
            objective,
            solver,
        })
    }
    pub(in crate::workflow) fn submit(
        &self,
        run_id: RunId,
        mut submission: crate::math::Submission,
    ) -> Result<DynamicsHandle, WorkflowError> {
        let local = std::time::Instant::now()
            .checked_add(self.profile.time_limit)
            .ok_or_else(|| contract("simulation submission deadline extent"))?;
        let deadline = submission.deadline.map_or(local, |outer| outer.min(local));
        submission.deadline = Some(deadline);
        let service = self.runtime.shared.math();
        let prepared = self.clone();
        let handle = service.submit_with(1, self.bytes, submission, move |flag, progress| {
            #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
            {
                let scope = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline));
                let accuracy_admission = (!prepared.numerics().policy.goals.is_empty())
                    .then(|| accuracy::task_admission(scope.clone()));
                if let Some(admission) = &accuracy_admission {
                    admission.reserve_attempt()?;
                }
                let mut worker = prepared.worker(scope.clone())?;
                let native_result = native::integrate_with_progress_observed(
                    &mut worker,
                    &prepared.profile,
                    &prepared.parameters,
                    flag.clone(),
                    progress.clone(),
                    &prepared.snapshot,
                );
                drop(worker);
                let mut accuracy_charges = Vec::new();
                if let Some(admission) = &accuracy_admission {
                    accuracy_charges.push(accuracy::complete_dynamic_attempt(
                        admission,
                        prepared.key,
                        0,
                    )?);
                }
                let report = native_result?;
                let checks = prepared.check_samples(run_id, &report, &prepared.parameters, &scope);
                let (mut report, checks, accuracy) = prepared.assess_dynamic_accuracy(
                    report,
                    checks,
                    accuracy::DynamicAttemptContext {
                        run_id,
                        scope: &scope,
                        flag,
                        progress: progress.clone(),
                        admission: accuracy_admission,
                        charges: &mut accuracy_charges,
                    },
                )?;
                if !prepared.numerics().policy.goals.is_empty() {
                    (report.progress, report.dropped_progress) = progress.snapshot();
                }
                let bytes = report
                    .numeric_bytes()
                    .checked_add(checks.rows.capacity() * size_of::<ModelingCheck>())
                    .and_then(|n| {
                        n.checked_add(
                            checks
                                .reports
                                .iter()
                                .map(pse_model::HeapUsage::owned_bytes)
                                .sum::<usize>(),
                        )
                    })
                    .and_then(|n| n.checked_add(4 << 20))
                    .and_then(|n| n.checked_add(accuracy::retained_bytes(&accuracy)))
                    .and_then(|n| {
                        n.checked_add(
                            report
                                .statistics
                                .iter()
                                .map(|entry| {
                                    serde_json::to_vec(entry).map_or(0, |bytes| bytes.len())
                                })
                                .sum::<usize>(),
                        )
                    })
                    .ok_or(crate::math::MathRuntimeError::Limit(
                        "modeling trajectory extent",
                    ))?;
                Ok(((report, checks, accuracy), bytes))
            }
            #[cfg(not(any(feature = "solver-diffsol", feature = "solver-idas")))]
            {
                let _ = (prepared, flag, progress, run_id);
                Err(crate::math::MathRuntimeError::Solve(
                    ProblemError::unsupported("dynamic backend is not linked"),
                ))
            }
        })?;
        Ok(handle)
    }
    pub(in crate::workflow) fn finish(
        &self,
        run_id: RunId,
        report: native::Report,
        checks: checks::SampleChecks,
        accuracy: Vec<pse_math::engineering_accuracy::GoalResult>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        let coverage = report.assess_endpoint(&self.profile);
        let required_closure = self.required_closure_checks();
        let completion = crate::workflow::numerics::complete(
            crate::workflow::numerics::trajectory_use(&report, coverage.satisfied),
            crate::workflow::numerics::CompletionEvidence {
                checks: &checks.rows,
                checks_complete: checks.complete && checks.error.is_none(),
                required_closure,
                endpoint_satisfied: Some(coverage.satisfied),
                coverage_complete: coverage.prefix_complete,
                accuracy: &accuracy,
            },
            &self.numerics().policy,
        )
        .with_context(self.numerics());
        let header = crate::workflow::completion::simulation_header(
            run_id,
            self,
            &report,
            &checks,
            &completion,
        )?;
        let assessment =
            completion.assessment_row(run_id, 0, &self.numerics().policy, None, None, false);
        Ok(ModelingTrajectory {
            inner: Arc::new(TrajectorySnapshot {
                run_id,
                completion,
                checks: checks.rows,
                reports: checks.reports,
                checks_complete: checks.complete,
                validation_error: checks.error,
                report: Arc::new(report),
                prepared: self.clone(),
                header,
                coverage,
                assessment,
                _owner: owner,
                tables: Mutex::new(None),
            }),
        })
    }

    /// Await one joined run. Cancellation still waits for native teardown.
    pub async fn run(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        let handle = self.start()?;
        let wait = handle.wait();
        tokio::pin!(wait);
        let result = tokio::select! { r=&mut wait=>r?, ()=cancel.cancelled()=>{handle.cancel();wait.await?} };
        match &result.report {
            Ok(crate::workflow::RunReport::Simulation(trajectory)) => {
                Ok(trajectory.as_ref().clone())
            }
            Err(error) => Err(WorkflowError::Shared(error.clone())),
            _ => Err(contract("simulation report mismatch")),
        }
    }
}
/// One owner for integrated state/parameter layout, shared with data-authored fixtures.
pub(super) fn dynamic_ports(
    product: &pse_compiler::workspace::PreparedModeling,
    case: &ModelingCaseBindings,
) -> Result<(Vec<pse_kernels::Port>, Vec<SemanticId>), WorkflowError> {
    let axis = product
        .model
        .integrated
        .values()
        .next()
        .ok_or_else(|| contract("missing integrated axis"))?;
    let integral_ids = product
        .model
        .integrals
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let rates = product
        .model
        .derivatives
        .values()
        .map(|d| d.rate)
        .collect::<BTreeSet<_>>();
    let fixed = case
        .variables
        .iter()
        .map(|(p, s)| {
            let id = *product
                .model
                .paths
                .get(p)
                .ok_or_else(|| contract(format!("unknown dynamic case path {p}")))?;
            if !product
                .admitted
                .case()
                .variables()
                .iter()
                .any(|v| v.port.id == id)
            {
                return Err(contract(
                    "dynamic fixedness requires an independent variable",
                ));
            }
            Ok((id, s.fixed))
        })
        .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
    let state = product
        .admitted
        .case()
        .variables()
        .iter()
        .filter(|v| {
            !rates.contains(&v.port.id)
                && !fixed.get(&v.port.id).copied().flatten().unwrap_or(v.fixed)
        })
        .map(|v| v.port.clone())
        .collect::<Vec<_>>();
    let states = state.iter().map(|p| p.id).collect::<Vec<_>>();
    // ADR-0103 item 6: integration holds discrete variables fixed per segment.
    product
        .model
        .require_fixed_discrete(
            states.iter().copied(),
            pse_modeling::DomainAnalysis::IntegratedDynamics,
        )
        .map_err(crate::workflow::modeling_error)?;
    if product
        .model
        .derivatives
        .keys()
        .any(|id| !states.contains(id))
    {
        return Err(contract(
            "a differentiated state cannot be fixed for integration",
        ));
    }
    let parameters = product
        .admitted
        .inputs
        .iter()
        .filter(|id| {
            !rates.contains(id)
                && !integral_ids.contains(id)
                && **id != axis.time
                && !states.contains(id)
        })
        .copied()
        .collect::<Vec<_>>();
    Ok((state, parameters))
}
impl ModelingPackage {
    /// Resolve authored integration controls against the generated coordinate layout.
    pub(in crate::workflow) fn integration_profile(
        &self,
        model: &ModelingPreparation,
        data: &pse_modeling::specialize::Fixture,
        numerics: &pse_model::numerics::NumericalPolicy,
    ) -> Result<AuthoredIntegrationControls, WorkflowError> {
        let integration = data
            .integration
            .as_ref()
            .ok_or_else(|| contract("integration controls absent"))?;
        let product = model.compiled();
        let axis = product
            .model
            .integrated
            .values()
            .next()
            .ok_or_else(|| contract("integrated fixture axis absent"))?;
        let time_scale = self
            .quantities
            .unit(
                self.quantities
                    .quantity_type(axis.quantity)
                    .map_err(math)?
                    .canonical_unit,
            )
            .map_err(math)?
            .scale_to_canonical;
        let case = ModelingCaseBindings::from(data);
        let (states, parameters) = dynamic_ports(product, &case)?;
        // Generated flux quadratures inherit the declared physical closure allowance
        // without another automatic precision multiplier. Explicit quadrature controls
        // override this slot below. Local integration error is estimated; independently
        // checked cumulative closure remains the acceptance authority, not a proof claim.
        let mut conserved_tolerances = BTreeMap::<SemanticId, f64>::new();
        for balance in product.model.inventory_balances.values() {
            // Factory controls precede bound context evaluation. This source floor is
            // replaced by the frozen resolved inventory budget before native admission.
            let value = match &balance.requirement.tolerance {
                pse_modeling::specialize::ClosureTolerance::Explicit(value) => value,
                pse_modeling::specialize::ClosureTolerance::EngineeringRule { rule_id, .. } => {
                    &product
                        .model
                        .engineering_rules
                        .iter()
                        .find(|rule| rule.id == *rule_id)
                        .ok_or_else(|| contract("inventory engineering rule unavailable"))?
                        .value
                }
            };
            let tolerance = match value {
                pse_modeling::specialize::Value::Number { bits, .. }
                | pse_modeling::specialize::Value::Coordinate { bits, .. } => f64::from_bits(*bits),
                pse_modeling::specialize::Value::Integer(value) => *value as f64,
                _ => {
                    return Err(contract(
                        "inventory tolerance has no static physical magnitude",
                    ));
                }
            };
            conserved_tolerances
                .entry(balance.flux_id)
                .and_modify(|current| *current = current.min(tolerance))
                .or_insert(tolerance);
        }
        let profile = pse_backend_native::dynamics::Profile {
            endpoint: match data.endpoint {
                pse_modeling::specialize::FixtureEndpoint::FixedHorizon => {
                    native::EndpointRequirement::default()
                }
                pse_modeling::specialize::FixtureEndpoint::DeclaredTerminalEvent(event) => {
                    native::EndpointRequirement {
                        kind: pse_model::generated::enums::EndpointPolicy::DeclaredTerminalEvent,
                        event: Some(event),
                    }
                }
            },
            start: axis.lower * time_scale,
            end: integration
                .samples
                .last()
                .copied()
                .ok_or_else(|| contract("integration samples empty"))?
                * time_scale,
            samples: integration.samples.iter().map(|t| t * time_scale).collect(),
            rtol: integration.relative_tolerance,
            out_rtol: integration.quadrature_relative_tolerance.or_else(|| {
                (!conserved_tolerances.is_empty()).then_some(integration.relative_tolerance)
            }),
            out_atol: product
                .model
                .integrals
                .keys()
                .map(|id| {
                    integration
                        .quadratures
                        .get(id)
                        .copied()
                        .or_else(|| conserved_tolerances.get(id).copied())
                        .ok_or_else(|| contract("integral tolerance absent"))
                })
                .collect::<Result<_, _>>()?,
            atol: vec![integration.normalized_absolute_tolerance; states.len()],
            initial_step: integration.initial_step * time_scale,
            parameter_scales: vec![1.; parameters.len()],
            schedule: authored_schedule(integration, &parameters, time_scale)?
                .into_iter()
                .map(|(input, _)| input)
                .collect(),
            numerics: numerics.clone(),
            ..pse_backend_native::dynamics::Profile::default()
        };
        let provisional_quadratures = product
            .model
            .inventory_balances
            .values()
            .filter(|balance| {
                !integration.quadratures.contains_key(&balance.flux_id)
                    && matches!(
                        balance.requirement.tolerance,
                        pse_modeling::specialize::ClosureTolerance::EngineeringRule { .. }
                    )
            })
            .map(|balance| balance.flux_id)
            .collect();
        Ok(AuthoredIntegrationControls {
            profile,
            provisional_quadratures,
        })
    }
}
/// The fixture's scheduled inputs on the integration layout (ADR-0119 Outcome 2): each
/// input's position among the contract parameters with its change times in seconds, and
/// its authored interval values.
fn authored_schedule<'f>(
    integration: &'f pse_modeling::specialize::IntegrationFixture,
    parameters: &[SemanticId],
    time_scale: f64,
) -> Result<Vec<(native::ScheduledInput, &'f [f64])>, WorkflowError> {
    integration
        .schedules
        .iter()
        .map(|s| {
            let parameter = parameters
                .iter()
                .position(|p| *p == s.target)
                .ok_or_else(|| {
                    contract("a scheduled input must be an integration parameter, not a state")
                })?;
            Ok((
                native::ScheduledInput {
                    parameter,
                    times: s.times.iter().map(|t| t * time_scale).collect(),
                },
                s.values.as_slice(),
            ))
        })
        .collect()
}
impl ModelingPackage {
    /// Bind an authored case directly to the integrated route and its integration controls.
    pub async fn declared_simulation(
        &self,
        root: DeclarationId,
        compiler: Profile,
        profile: Option<native::Profile>,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        let execution = self
            .declared_execution(
                root,
                compiler,
                Default::default(),
                Default::default(),
                limits,
                cancel,
            )
            .await?;
        if !matches!(execution.procedure, DeclaredProcedure::Integrate(_)) {
            return Err(contract(
                "declared simulation requires the authored integration procedure",
            ));
        }
        self.simulation_for_declared(&execution, profile, cancel)
            .await
    }
    /// Internal integration projection; the declared operation owner retains its procedure.
    pub(in crate::workflow) async fn simulation_for_declared(
        &self,
        execution: &DeclaredExecution,
        profile: Option<native::Profile>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        if !matches!(
            execution.procedure,
            DeclaredProcedure::Integrate(_) | DeclaredProcedure::Shooting { .. }
        ) {
            return Err(contract(
                "integration projection requires an admitted integrated procedure",
            ));
        }
        let analysis = &execution.analysis;
        let (profile, provisional_quadratures) = if let Some(profile) = profile {
            (profile, BTreeSet::new())
        } else {
            let fixture = execution
                .model
                .compiled()
                .model
                .fixtures
                .get(&analysis.instance)
                .ok_or_else(|| contract("authored integration controls absent"))?;
            let controls =
                self.integration_profile(&execution.model, fixture, &analysis.solver.numerics)?;
            (controls.profile, controls.provisional_quadratures)
        };
        self.prepare_simulation_with_controls(
            analysis.root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            analysis.case.clone(),
            analysis.compiler,
            profile,
            DerivativeOrder::First,
            &BTreeSet::new(),
            &provisional_quadratures,
            cancel,
        )
        .await
    }
    /// Prepare integrated dynamics from one specialized definition. Rates are compiler
    /// projections; all original equations, hints and source identities remain inspectable.
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    async fn prepare_simulation_mode(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        mut profile: native::Profile,
        derivatives: DerivativeOrder,
        mode: usize,
        mode_names: &[String],
        exact_parameters: &BTreeSet<SemanticId>,
        provisional_quadratures: &BTreeSet<SemanticId>,
        snapshot: &pse_backend_native::execution::Snapshot,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        if derivatives < DerivativeOrder::First {
            return Err(contract(
                "integrated dynamics needs at least first-order partials",
            ));
        }
        let model = self
            .prepare(root, instance, bindings, limits, cancel)
            .await?;
        let product = model.compiled();
        if let Some(fixture) = product.model.fixtures.get(&instance) {
            let required = match fixture.endpoint {
                pse_modeling::specialize::FixtureEndpoint::FixedHorizon => {
                    native::EndpointRequirement::default()
                }
                pse_modeling::specialize::FixtureEndpoint::DeclaredTerminalEvent(event) => {
                    native::EndpointRequirement {
                        kind: pse_model::generated::enums::EndpointPolicy::DeclaredTerminalEvent,
                        event: Some(event),
                    }
                }
            };
            if profile.endpoint != required {
                return Err(contract(
                    "integration profile contradicts the authored endpoint requirement",
                ));
            }
        }
        let axis = product
            .model
            .integrated
            .values()
            .next()
            .ok_or_else(|| contract("simulation requires an integrated time axis"))?;
        // An authored objective needs an optimizing consumer: a shooting fixture, whose
        // shooting problem minimizes it (ADR-0110 Outcome 5).
        let shooting = product.model.fixtures.get(&instance).is_some_and(|f| {
            f.procedure == pse_model::generated::enums::ModelingProcedure::Shooting
        });
        if product.model.integrated.len() != 1
            || (product.admitted.case().objective().is_some() && !shooting)
        {
            return Err(contract(
                "integrated consumer requires one time axis; an optimization objective needs a shooting fixture",
            ));
        }
        let time_unit = self
            .quantities
            .unit(
                self.quantities
                    .quantity_type(axis.quantity)
                    .map_err(math)?
                    .canonical_unit,
            )
            .map_err(math)?;
        let time_scale = time_unit.scale_to_canonical;
        if profile.start != axis.lower * time_scale || profile.end > axis.upper * time_scale {
            return Err(contract(
                "integration horizon must start at the domain lower bound and remain inside the domain",
            ));
        }
        let integral_ids = product
            .model
            .integrals
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        let fixed_integrals = integral_ids.iter().any(|id| {
            !product
                .model
                .inventory_balances
                .values()
                .any(|b| b.flux_id == *id)
        });
        if fixed_integrals
            && (profile.end != axis.upper * time_scale
                || profile.samples.last().copied() != Some(profile.end))
        {
            return Err(contract(
                "definite integrals require the complete domain and an upper-endpoint sample",
            ));
        }
        // Quadratures coexist with state/output sensitivities. Terminal
        // expressions are excluded from native outputs below, so this does
        // not claim derivatives of the integral itself.
        let terminal_rows = product
            .admitted
            .case()
            .instances()
            .iter()
            .filter(|i| i.slots.iter().any(|s| integral_ids.contains(&s.source())))
            .map(|i| i.instance)
            .collect::<BTreeSet<_>>();
        let terminal_targets = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| match o {
                ModelingOutput::Member(id) if terminal_rows.contains(&o.row_id()) => Some(*id),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        if product.model.annotations.iter().any(|a| {
            terminal_targets.contains(&a.target)
                && matches!(
                    a.value,
                    pse_modeling::annotation::AnnotationValue::Bounds(..)
                        | pse_modeling::annotation::AnnotationValue::Start(..)
                )
        }) {
            return Err(contract(
                "terminal integral bounds or starts require a terminal check, not an integration constraint",
            ));
        }
        let (state, parameters) = dynamic_ports(product, &case)?;
        // The fixture's schedules are the authority (ADR-0119 Outcome 2): a profile may
        // repeat an authored schedule, but a different one for the same input is refused.
        let authored = match product
            .model
            .fixtures
            .get(&instance)
            .and_then(|f| f.integration.as_ref())
        {
            Some(integration) => authored_schedule(integration, &parameters, time_scale)?,
            None => Vec::new(),
        };
        for (input, _) in &authored {
            match profile
                .schedule
                .iter()
                .find(|s| s.parameter == input.parameter)
            {
                Some(declared) if declared == input => {}
                Some(_) => {
                    return Err(contract(
                        "the profile schedules an input differently from its authored schedule",
                    ));
                }
                None => profile.schedule.push(input.clone()),
            }
        }
        let rates = product
            .model
            .derivatives
            .values()
            .map(|d| d.rate)
            .collect::<BTreeSet<_>>();
        let states = state.iter().map(|p| p.id).collect::<Vec<_>>();
        let (mut values, starts) = self
            .resolve_starts(&model, &case, &Default::default(), compiler, false, cancel)
            .await?;
        // Terminal placeholders are excluded from every trial program below. Their
        // real values are supplied only by completed native whole-domain quadrature.
        for id in &integral_ids {
            values.scalars.insert(*id, 0.);
        }
        let initial_outputs = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| {
                if let ModelingOutput::InitialState { state, equation } = o {
                    Some((*state, *equation, o.row_id()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if initial_outputs.len() != product.model.derivatives.len()
            || initial_outputs.len() != product.model.initial_equations.len()
        {
            return Err(contract(
                "every differential state requires one explicit isolated lower-endpoint initial condition",
            ));
        }
        let initial_values = self
            .observe_registered(
                model.clone(),
                initial_outputs.iter().map(|(_, _, row)| *row).collect(),
                values.clone(),
                compiler,
                self.registrations(),
                cancel,
            )
            .await?;
        for (state, _, row) in &initial_outputs {
            values.scalars.insert(*state, initial_values[row]);
        }
        // Evaluate the original isolated RHS projection, independently of case Start overrides.
        let original_initial_rows = product
            .model
            .inventory_initial_conditions
            .iter()
            .map(|(target, obligation)| {
                ModelingOutput::Hint {
                    target: *target,
                    declaration: obligation.lineage.declaration,
                    kind: ModelingHint::Start,
                }
                .row_id()
            })
            .collect::<Vec<_>>();
        let original_initial_values = self
            .observe_registered(
                model.clone(),
                original_initial_rows.iter().copied().collect(),
                values.clone(),
                compiler,
                self.registrations(),
                cancel,
            )
            .await?;
        if let Some(id) = states
            .iter()
            .chain(&parameters)
            .find(|id| !values.scalars.contains_key(id))
        {
            return Err(contract(format!(
                "dynamic coordinate {id} requires a value or deterministic start"
            )));
        }
        let providers = self
            .inner_registrations(
                model.clone(),
                &case,
                &crate::math::solves::NumericalInputs::default(),
                &profile.numerics,
                &pse_backend_native::solve::Controls::default(),
                derivatives,
                compiler,
                cancel,
                implicit::ProviderDemand::Observations(None),
            )
            .await?;
        let mut differential = Vec::new();
        let mut rhs = Vec::new();
        let mut used = product.model.initial_equations.clone();
        for id in &states {
            if product.model.derivatives.contains_key(id) {
                let (output, equations) = product
                    .admitted
                    .outputs
                    .iter()
                    .find_map(|o| {
                        if let ModelingOutput::DynamicRate { state, equations } = o {
                            (*state == *id).then_some((o.row_id(), equations))
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| contract("integrated derivative system is absent"))?;
                differential.push(true);
                rhs.push(output);
                used.extend(equations);
            } else {
                differential.push(false);
                rhs.push(SemanticId::NIL);
            }
        }
        let algebraic = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| match o {
                ModelingOutput::Equation { id, sense } if !used.contains(id) => Some((*id, *sense)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if algebraic.len() != differential.iter().filter(|v| !**v).count()
            || algebraic
                .iter()
                .any(|(_, s)| *s != pse_authoring::dsl::EquationSense::Eq)
        {
            return Err(contract(
                "dynamic algebraic partition must be square and contain equalities only",
            ));
        }
        for (row, (id, _)) in rhs
            .iter_mut()
            .filter(|r| **r == SemanticId::NIL)
            .zip(&algebraic)
        {
            *row = *id;
        }
        self.runtime
            .shared
            .math()
            .validate_modeling_partition(
                self.numerical_workspace()?,
                model.clone(),
                algebraic.iter().map(|(id, _)| *id).collect(),
                states
                    .iter()
                    .zip(&differential)
                    .filter_map(|(id, d)| (!d).then_some(*id))
                    .collect(),
                compiler,
                cancel,
            )
            .await?;
        let mut initial = Vec::new();
        let mut constants = BTreeMap::new();
        for (i, id) in states.iter().enumerate() {
            if let Some((_, _, row)) = initial_outputs.iter().find(|(state, _, _)| state == id) {
                initial.push(*row);
                continue;
            }
            match start_row(&product.admitted.outputs, *id, starts.get(id))? {
                Some(row) => initial.push(row),
                None => {
                    initial.push(SemanticId::NIL);
                    constants.insert(i, values.scalars[id]);
                }
            }
        }
        let mut outputs = states
            .iter()
            .map(|id| ModelingOutput::Member(*id).row_id())
            .collect::<Vec<_>>();
        for a in &product.model.annotations {
            // A shooting fixture observes its objective members, too.
            if matches!(
                a.value,
                pse_modeling::annotation::AnnotationValue::Report(_)
            ) || matches!(
                a.value,
                pse_modeling::annotation::AnnotationValue::AccuracyGoal(_)
            ) || (shooting
                && matches!(
                    a.value,
                    pse_modeling::annotation::AnnotationValue::Objective(_)
                ))
            {
                let id = ModelingOutput::Member(a.target).row_id();
                if !outputs.contains(&id) {
                    outputs.push(id);
                }
            }
        }
        for goal in &profile.numerics.goals {
            if !integral_ids.contains(&goal.target_id) {
                let id = ModelingOutput::Member(goal.target_id).row_id();
                if !outputs.contains(&id) {
                    outputs.push(id);
                }
            }
        }
        outputs.retain(|row| {
            !product
                .model
                .integrals
                .keys()
                .any(|id| ModelingOutput::Member(*id).row_id() == *row)
        });
        outputs.retain(|id| !terminal_rows.contains(id));
        let nominal_rows = product
            .admitted
            .outputs
            .iter()
            .filter(|o| {
                matches!(
                    o,
                    ModelingOutput::Hint {
                        kind: ModelingHint::Nominal,
                        ..
                    }
                )
            })
            .map(ModelingOutput::row_id)
            .collect();
        let observed = self
            .observe_registered(
                model.clone(),
                nominal_rows,
                values.clone(),
                compiler,
                providers.clone(),
                cancel,
            )
            .await?;
        let mut targets = state
            .iter()
            .map(|p| pse_math::numerics::TargetSpec {
                id: p.id,
                kind: NumericalTarget::Variable,
                quantity: p.quantity,
                unit: p.unit,
                integer: false,
                declared_tolerance: None,
            })
            .collect::<Vec<_>>();
        for (id, _) in &algebraic {
            let row = product
                .admitted
                .case()
                .rows()
                .iter()
                .find(|r| r.id == *id)
                .ok_or_else(|| contract("algebraic row missing"))?;
            targets.push(pse_math::numerics::TargetSpec {
                id: *id,
                kind: NumericalTarget::Row,
                quantity: row.quantity,
                unit: self
                    .quantities
                    .quantity_type(row.quantity)
                    .map_err(math)?
                    .canonical_unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        let extra_targets = profile
            .numerics
            .goals
            .iter()
            .map(|g| (g.target_id, g.target_kind))
            .chain(product.model.annotations.iter().filter_map(|a| {
                matches!(
                    a.value,
                    pse_modeling::annotation::AnnotationValue::AccuracyGoal(_)
                        | pse_modeling::annotation::AnnotationValue::EngineeringScale(_)
                        | pse_modeling::annotation::AnnotationValue::EngineeringDefault { .. }
                )
                .then_some((
                    a.target,
                    if states.contains(&a.target) {
                        NumericalTarget::Variable
                    } else {
                        if product.model.closures.contains_key(&a.target)
                            || product.model.inventory_balances.contains_key(&a.target)
                        {
                            NumericalTarget::Closure
                        } else {
                            NumericalTarget::Observable
                        }
                    },
                ))
            }))
            .collect::<BTreeSet<_>>();
        for (id, kind) in extra_targets {
            if targets.iter().any(|t| t.id == id && t.kind == kind) {
                continue;
            }
            let ty = product
                .model
                .symbols
                .get(&id)
                .map(|symbol| &symbol.ty)
                .or_else(|| product.model.closures.get(&id).map(|closure| &closure.ty))
                .or_else(|| {
                    product
                        .model
                        .inventory_balances
                        .get(&id)
                        .map(|balance| &balance.ty)
                })
                .ok_or_else(|| contract("dynamic accuracy target absent"))?;
            let pse_modeling::Type::Quantity(q) = ty else {
                return Err(contract("dynamic accuracy target physical type"));
            };
            let quantity = q
                .resolve(&self.quantities, &BTreeMap::new())
                .map_err(|e| contract(e.to_string()))?;
            let unit = self
                .quantities
                .quantity_type(quantity)
                .map_err(|e| contract(e.to_string()))?
                .canonical_unit;
            targets.push(pse_math::numerics::TargetSpec {
                id,
                kind,
                quantity,
                unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        let mut declarations = Vec::new();
        for o in &product.admitted.outputs {
            if let ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::Nominal,
            } = o
                && let Some(t) = targets.iter().find(|t| t.id == *target)
            {
                declarations.push(cases::requirement(
                    model.solved().lineage(),
                    *target,
                    t.kind,
                    *declaration,
                    NumericalSource::ModelHint,
                    Some(observed[&o.row_id()]),
                    None,
                ));
            }
        }
        let frozen = states
            .iter()
            .chain(&integral_ids)
            .copied()
            .chain(std::iter::once(axis.time))
            .chain(
                profile
                    .schedule
                    .iter()
                    .filter_map(|s| parameters.get(s.parameter).copied()),
            )
            .collect::<BTreeSet<_>>();
        for annotation in &product.model.annotations {
            match &annotation.value {
                pse_modeling::annotation::AnnotationValue::AccuracyGoal(goal) => {
                    for expression in [
                        goal.time.as_ref(),
                        goal.resolution.as_ref(),
                        goal.criterion_lower.as_ref(),
                        goal.criterion_upper.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        cases::require_frozen_expression(expression, &product.model, &frozen)?;
                    }
                }
                pse_modeling::annotation::AnnotationValue::EngineeringScale(scale) => {
                    cases::require_frozen_expression(&scale.value, &product.model, &frozen)?;
                }
                _ => {}
            }
        }
        let accuracy_rows = product
            .admitted
            .outputs
            .iter()
            .filter(|o| {
                matches!(
                    o,
                    ModelingOutput::Hint {
                        kind: ModelingHint::AccuracyGoalTime
                            | ModelingHint::AccuracyGoalResolution
                            | ModelingHint::AccuracyGoalLower
                            | ModelingHint::AccuracyGoalUpper
                            | ModelingHint::EngineeringScaleValue,
                        ..
                    }
                )
            })
            .map(ModelingOutput::row_id)
            .collect::<BTreeSet<_>>();
        let accuracy_values = if accuracy_rows.is_empty() {
            BTreeMap::new()
        } else {
            let observed = self
                .observe_registered(
                    model.clone(),
                    accuracy_rows,
                    values.clone(),
                    compiler,
                    providers.clone(),
                    cancel,
                )
                .await?;
            observed.iter().map(|(id, value)| (*id, *value)).collect()
        };
        let mut numerical_inputs = crate::math::solves::NumericalInputs::default();
        cases::lower_authored_accuracy(
            product,
            model.solved().lineage(),
            &self.quantities,
            &mut numerical_inputs,
            &mut profile.numerics,
            &mut targets,
            &accuracy_values,
        )?;
        declarations.extend(numerical_inputs.declarations);
        let closure_requirements = cases::lower_closure_requirements(
            product,
            &mut targets,
            model.solved().lineage(),
            &self.quantities,
            &mut declarations,
            &mut profile.numerics,
        )?;
        declarations.extend(closure_requirements);
        declarations.extend(cases::state_reconstruction_requirements(
            product,
            &targets,
            model.solved().lineage(),
            &self.quantities,
        )?);
        declarations.extend(cases::conservation_row_requirements(
            product,
            &targets,
            model.solved().lineage(),
            &self.quantities,
        )?);
        let numerics = Arc::new(
            pse_math::numerics::resolve(
                &self.quantities,
                &targets,
                &declarations,
                &profile.numerics,
            )
            .map_err(math)?,
        );
        let closure_budgets = cases::closure_budgets(&numerics);
        for (position, integral) in product.model.integrals.keys().enumerate() {
            if !provisional_quadratures.contains(integral) {
                continue;
            }
            let budgets = product
                .model
                .inventory_balances
                .values()
                .filter(|balance| balance.flux_id == *integral)
                .map(|balance| {
                    closure_budgets
                        .get(&balance.id)
                        .copied()
                        .ok_or_else(|| contract("resolved inventory closure budget absent"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if budgets.is_empty() {
                return Err(contract(
                    "provisional quadrature has no physical inventory owner",
                ));
            }
            *profile
                .out_atol
                .get_mut(position)
                .ok_or_else(|| contract("integrated flux control absent"))? =
                budgets.into_iter().fold(f64::INFINITY, f64::min);
        }
        let target_access = pse_math::numerics::TargetAccess::new(&numerics).map_err(math)?;
        let scale = |id, kind| {
            target_access
                .get(kind, id)
                .map(|target| target.coordinate_scale)
                .map_err(math)
        };
        let coordinates = DynamicCoordinates {
            time: axis.time,
            time_scale,
            time_origin: 0.,
            state: states
                .iter()
                .map(|id| {
                    Ok(CoordinateBinding {
                        id: *id,
                        scale: scale(*id, NumericalTarget::Variable)?,
                        offset: 0.,
                    })
                })
                .collect::<Result<_, WorkflowError>>()?,
            parameters: parameters
                .iter()
                .map(|id| CoordinateBinding {
                    id: *id,
                    scale: 1.,
                    offset: 0.,
                })
                .collect(),
        };
        let derivative_coordinates = states
            .iter()
            .chain(&parameters)
            .copied()
            .collect::<Vec<_>>();
        let original_initial_conditions = product
            .model
            .inventory_initial_conditions
            .iter()
            .zip(&original_initial_rows)
            .map(|((target, obligation), row)| {
                let coordinate = coordinates
                    .state
                    .iter()
                    .position(|c| c.id == *target)
                    .ok_or_else(|| contract("original initial coordinate absent"))?;
                let binding = &coordinates.state[coordinate];
                let expected = *original_initial_values
                    .get(row)
                    .ok_or_else(|| contract("original initial RHS projection absent"))?;
                let normalized = (expected - binding.offset) / binding.scale;
                let absolute = *profile
                    .atol
                    .get(coordinate)
                    .ok_or_else(|| contract("original initial absolute tolerance absent"))?;
                let tolerance = binding.scale.abs() * (absolute + profile.rtol * normalized.abs());
                if !expected.is_finite() || !tolerance.is_finite() || tolerance <= 0. {
                    return Err(contract(
                        "original initial physical value or tolerance invalid",
                    ));
                }
                Ok(OriginalInitialCondition {
                    coordinate,
                    row: obligation.row,
                    source: obligation.lineage.declaration,
                    expected,
                    tolerance,
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        // Every interval of a scheduled input starts at the model's value; the authored
        // interval values then replace it (I6).
        let mut parameter_values = profile.integration_parameters(
            &parameters
                .iter()
                .map(|id| values.scalars[id])
                .collect::<Vec<_>>(),
        );
        let first = profile.columns_at(parameters.len(), profile.start);
        for (input, authored) in &authored {
            parameter_values
                .get_mut(first[input.parameter]..first[input.parameter] + authored.len())
                .ok_or_else(|| contract("scheduled input interval layout"))?
                .copy_from_slice(authored);
        }
        let event_layout = (0..mode_names.len())
            .map(|index| {
                events::resolve_events(product, instance, index, &states).map(|(events, _)| events)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fixture_events = product
            .model
            .fixtures
            .get(&instance)
            .map(|f| f.modes.iter().flat_map(|m| &m.events).collect::<Vec<_>>())
            .unwrap_or_default();
        let balances = product
            .model
            .inventory_balances
            .values()
            .map(|balance| {
                let tolerance = closure_budgets.get(&balance.id).copied()
                    .ok_or_else(|| contract_error("resolved physical inventory closure budget absent"))?;
                let mut transfers = BTreeSet::new();
                for guard in balance.transfers.keys() {
                    let matching = fixture_events
                        .iter()
                        .filter(|event| event.guard == *guard)
                        .collect::<Vec<_>>();
                    if matching.is_empty() || matching.iter().any(|e| e.next.is_none()) {
                        let mut refusal = pse_model::diagnostic::BoundaryDiagnostic::new(
                            pse_model::diagnostic::BoundaryClass::Unsupported,
                            pse_diagnostics::DiagnosticStage::ModelingConservationTransfer,
                            [balance.id, balance.lineage.declaration.into(), *guard],
                            pse_diagnostics::DiagnosticRule::ModelingDynamicInventoryTransferEventUnsupported,
                        );
                        refusal.observations.insert("capability".into(), pse_model::diagnostic::Observation::Text(
                            "a permitted inventory transfer requires the same resolved guard occurrence as a nonterminal authored event".into(),
                        ));
                        return Err(Box::new(refusal).into());
                    }
                    for event in matching {
                        transfers.insert(event.guard);
                    }
                }
                Ok(native::Balance {
                    id: balance.id,
                    inventory: balance.inventory_id,
                    flux: balance.flux_id,
                    tolerance,
                    transfers,
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        let signs = self
            .state_signs(
                &model, &case, &states, &values, &providers, compiler, cancel,
            )
            .await?;
        let mut dynamic_contract = native::Contract {
            identity: ContentHash::from_bytes([0; 32]),
            states: states.clone(),
            differential: differential.clone(),
            parameters: parameters.clone(),
            outputs: outputs.clone(),
            events: event_layout,
            quadratures: product.model.integrals.keys().copied().collect(),
            balances: balances.clone(),
            signs: signs.clone(),
            derivatives,
        };
        let directions = exact_parameters
            .iter()
            .map(|id| {
                parameters
                    .iter()
                    .position(|p| p == id)
                    .map(|position| first[position])
                    .ok_or_else(|| {
                        contract("exact transient parameter is not an integration parameter")
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let demand = if directions.is_empty() {
            native::DynamicDemand::Base
        } else {
            native::DynamicDemand::ExactHessian
        };
        let mut consumer = profile.clone();
        if demand == native::DynamicDemand::ExactHessian {
            consumer.sensitivity = native::DynamicSensitivity::Adjoint;
        }
        profile.method = consumer
            .resolve_for(
                &dynamic_contract,
                &parameter_values,
                snapshot,
                demand,
                derivatives,
                &directions,
            )
            .map_err(crate::math::MathRuntimeError::from)?
            .method;
        let mut programs = Vec::new();
        let (event_contracts, event_roles) =
            events::resolve_events(product, instance, mode, &states)?;
        let quadrature_rows = product
            .model
            .integrals
            .values()
            .map(|q| ModelingOutput::Member(q.integrand).row_id())
            .collect::<Vec<_>>();
        let mut roles = vec![
            (Function::Rhs, rhs.clone(), BTreeMap::new()),
            (Function::Initial, initial, constants),
            (Function::Output, outputs.clone(), BTreeMap::new()),
        ];
        if !quadrature_rows.is_empty() {
            roles.push((Function::QuadratureFlux, quadrature_rows, BTreeMap::new()));
        }
        if !product.model.inventory_balances.is_empty() {
            roles.push((
                Function::Inventory,
                product
                    .model
                    .inventory_balances
                    .keys()
                    .map(|id| ModelingOutput::Inventory(*id).row_id())
                    .collect(),
                BTreeMap::new(),
            ));
        }
        roles.extend(event_roles);
        for (function, rows, mut constants) in roles {
            let selected = rows
                .iter()
                .enumerate()
                .filter(|(i, _)| !constants.contains_key(i))
                .map(|(_, r)| *r)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let compiled = self
                .runtime
                .shared
                .math()
                .prepare_modeling_functions(
                    self.numerical_workspace()?,
                    model.clone(),
                    selected,
                    derivative_coordinates.clone(),
                    derivatives,
                    compiler,
                    cancel,
                )
                .await?;
            // No derivative-coordinate feedback may leak into an explicit RHS; an
            // implicit DAE would need a residual adapter rather than a guessed rate.
            for i in compiled.assembly.structure().instances() {
                if i.slots.iter().any(|s| integral_ids.contains(&s.source())) {
                    return Err(contract(
                        "definite integral has a noncausal dependency in an integrated trial function; use simultaneous realization",
                    ));
                }
                if i.slots.iter().any(|s| rates.contains(&s.source())) {
                    return Err(contract(
                        "dynamic function depends on an unresolved derivative coordinate",
                    ));
                }
                if function == Function::Initial
                    && i.slots.iter().any(|s| states.contains(&s.source()))
                {
                    return Err(contract(
                        "dynamic initial values must be independent of state guesses",
                    ));
                }
            }
            let row_indices = rows
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    if constants.contains_key(&i) {
                        Ok(0)
                    } else {
                        compiled
                            .assembly
                            .structure()
                            .rows()
                            .iter()
                            .position(|r| r.id == *id)
                            .ok_or_else(|| contract("dynamic output missing"))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut scales = vec![1.; rows.len()];
            if matches!(
                function,
                Function::Rhs | Function::Initial | Function::Reset(_)
            ) {
                for (i, id) in states.iter().enumerate() {
                    let state_scale = coordinates.state[i].scale;
                    scales[i] = if matches!(function, Function::Initial | Function::Reset(_)) {
                        1. / state_scale
                    } else if differential[i] {
                        let row = compiled
                            .assembly
                            .structure()
                            .rows()
                            .iter()
                            .find(|r| r.id == rows[i])
                            .ok_or_else(|| contract("rate row missing"))?;
                        let rate = self.quantities.quantity_type(row.quantity).map_err(math)?;
                        self.quantities
                            .unit(rate.canonical_unit)
                            .map_err(math)?
                            .scale_to_canonical
                            / self
                                .quantities
                                .unit(state[i].unit)
                                .map_err(math)?
                                .scale_to_canonical
                            / state_scale
                    } else {
                        1. / scale(rows[i], NumericalTarget::Row)?
                    };
                    if let Some(v) = constants.get_mut(&i) {
                        *v /= state_scale;
                    }
                    let _ = id;
                }
            }
            if function == Function::QuadratureFlux {
                for (i, integral) in product.model.integrals.values().enumerate() {
                    let quantity = |id| -> Result<_, WorkflowError> {
                        let pse_modeling::Type::Quantity(q) = &product.model.symbols[&id].ty else {
                            return Err(contract("integral physical type absent"));
                        };
                        let q = q
                            .resolve(&self.quantities, &BTreeMap::new())
                            .map_err(|e| contract(e.to_string()))?;
                        self.quantities
                            .quantity_type(q)
                            .map_err(|e| contract(e.to_string()))
                    };
                    scales[i] = self
                        .quantities
                        .unit(quantity(integral.integrand)?.canonical_unit)
                        .map_err(math)?
                        .scale_to_canonical
                        / self
                            .quantities
                            .unit(quantity(integral.result)?.canonical_unit)
                            .map_err(math)?
                            .scale_to_canonical;
                }
            }
            programs.push(FunctionProgram {
                mode: 0,
                function,
                case: compiled,
                rows: row_indices,
                offsets: vec![0.; scales.len()],
                scales,
                constants,
            });
        }
        let guard = self
            .prepare_dynamic_guards(&model, &case, &terminal_targets, compiler, cancel)
            .await?;
        let units = results::assessment_units(product);
        let sample_scope = (!integral_ids.is_empty()).then(|| {
            units
                .iter()
                .filter(|(_, rows)| rows.is_disjoint(&terminal_rows))
                .map(|(key, _)| *key)
                .collect::<results::AssessmentScope>()
        });
        let mut all_check_rows = units.values().flatten().copied().collect::<BTreeSet<_>>();
        all_check_rows.extend(algebraic.iter().map(|(id, _)| *id));
        let mut sample_rows = units
            .iter()
            .filter(|(key, _)| sample_scope.as_ref().is_none_or(|s| s.contains(key)))
            .flat_map(|(_, rows)| rows.iter().copied())
            .collect::<BTreeSet<_>>();
        sample_rows.extend(algebraic.iter().map(|(id, _)| *id));
        let check_program = if sample_rows.is_empty() {
            None
        } else {
            Some(
                self.runtime
                    .shared
                    .math()
                    .prepare_modeling_functions(
                        self.numerical_workspace()?,
                        model.clone(),
                        sample_rows.into_iter().collect(),
                        derivative_coordinates.clone(),
                        DerivativeOrder::Value,
                        compiler,
                        cancel,
                    )
                    .await?,
            )
        };
        let terminal_check_program = if integral_ids.is_empty() || all_check_rows.is_empty() {
            None
        } else {
            Some(
                self.runtime
                    .shared
                    .math()
                    .prepare_modeling_functions(
                        self.numerical_workspace()?,
                        model.clone(),
                        all_check_rows.into_iter().collect(),
                        derivative_coordinates.clone(),
                        DerivativeOrder::Value,
                        compiler,
                        cancel,
                    )
                    .await?,
            )
        };
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingDynamicV3);
        hash.hash(&snapshot.identity())
            .id(&root.as_id())
            .id(&instance.as_id())
            .hash(&numerics.key)
            .hash(
                &super::super::dynamics::profile_identity(&profile)
                    .map_err(crate::math::MathRuntimeError::from)?
                    .as_id(),
            )
            .u64(match derivatives {
                DerivativeOrder::Value => 0,
                DerivativeOrder::First => 1,
                DerivativeOrder::Second => 2,
            });
        for p in &programs {
            for id in p.case.assembly.bodies().keys() {
                hash.hash(id);
            }
            for row in &p.rows {
                hash.u64(*row as u64);
            }
            for (i, value) in &p.constants {
                hash.u64(*i as u64).u64(value.to_bits());
            }
        }
        for (id, v) in &values.scalars {
            hash.id(id).u64(v.to_bits());
        }
        for v in &parameter_values {
            hash.u64(v.to_bits());
        }
        for r in providers.values() {
            hash.hash(&r.configuration_key());
        }
        if let Some(guard) = &guard {
            for id in guard.case.assembly.bodies().keys() {
                hash.hash(id);
            }
            for check in &guard.checks {
                hash.id(&check.source)
                    .bool(check.lower.is_some())
                    .bool(check.upper.is_some());
                for v in [&check.value]
                    .into_iter()
                    .chain(check.lower.as_ref())
                    .chain(check.upper.as_ref())
                {
                    match v {
                        RangeValue::Input(id) => {
                            hash.str("input").id(id);
                        }
                        RangeValue::Output(id) => {
                            hash.str("output").id(id);
                        }
                        RangeValue::Constant(v) => {
                            hash.str("constant").u64(v.to_bits());
                        }
                    }
                }
            }
        }
        for program in check_program.iter().chain(&terminal_check_program) {
            for id in program.assembly.bodies().keys() {
                hash.hash(id);
            }
            for row in program.assembly.structure().rows() {
                hash.id(&row.id);
            }
        }
        hash.str(&mode_names[mode]);
        for obligation in &original_initial_conditions {
            hash.str("original-coordinate-initial")
                .id(&obligation.row)
                .id(&obligation.source.into())
                .u64(obligation.coordinate as u64)
                .u64(obligation.expected.to_bits())
                .u64(obligation.tolerance.to_bits());
        }
        for event in &event_contracts {
            hash.id(&event.id)
                .bool(event.terminal)
                .u64(event.next_mode as u64)
                .u64(event.tolerance.to_bits())
                .str(event.direction.as_str());
        }
        for sign in &signs {
            hash.str(sign.as_str());
        }
        hash.u64(balances.len() as u64);
        for balance in &balances {
            hash.id(&balance.id)
                .id(&balance.inventory)
                .id(&balance.flux)
                .u64(balance.tolerance.to_bits())
                .u64(balance.transfers.len() as u64);
            for event in &balance.transfers {
                hash.id(event);
            }
        }
        let key = hash.finish_hash();
        dynamic_contract.identity = key;
        let contract = dynamic_contract;
        let cells = profile
            .validate(&contract, &parameter_values)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let bytes = programs.iter().try_fold(
            cells
                .checked_mul(64)
                .and_then(|n| n.checked_add(4 << 20))
                .ok_or_else(|| contract_error("dynamic result extent"))?,
            |n, p| {
                n.checked_add(p.case.assembly.numeric_worker_bytes())
                    .ok_or_else(|| contract_error("dynamic worker extent"))
            },
        )?;
        let bytes = bytes
            .checked_add(guard.as_ref().map_or(0, |g| {
                g.case.assembly.numeric_worker_bytes() + g.checks.len() * 256 + g.rows.len() * 128
            }))
            .ok_or_else(|| contract_error("dynamic guard extent"))?;
        let maximum_checks = product
            .admitted
            .outputs
            .len()
            .checked_add(product.model.annotations.len())
            .and_then(|n| n.checked_add(product.model.closures.len()))
            .and_then(|n| n.checked_mul(profile.samples.len()))
            .and_then(|n| n.checked_add(original_initial_conditions.len()))
            .and_then(|n| {
                if contract.balances.is_empty() {
                    return Some(n);
                }
                profile
                    .max_events
                    .checked_mul(3)?
                    .checked_add(profile.samples.len())?
                    .checked_add(2)?
                    .checked_mul(contract.balances.len())?
                    .checked_add(n)
            })
            .ok_or_else(|| contract_error("dynamic check extent"))?;
        if maximum_checks > profile.max_cells {
            return Err(contract_error("dynamic check cell budget"));
        }
        let bytes = bytes
            .checked_add(
                original_initial_conditions
                    .len()
                    .checked_mul(size_of::<OriginalInitialCondition>())
                    .ok_or_else(|| contract_error("dynamic original initial storage"))?,
            )
            .ok_or_else(|| contract_error("dynamic original initial storage"))?;
        let bytes = bytes
            .checked_add(
                maximum_checks
                    .checked_mul(size_of::<ModelingCheck>() * 2)
                    .ok_or_else(|| contract_error("dynamic check storage"))?,
            )
            .and_then(|n| {
                n.checked_add(
                    check_program
                        .iter()
                        .chain(&terminal_check_program)
                        .map(|p| p.assembly.numeric_worker_bytes())
                        .sum::<usize>(),
                )
            })
            .and_then(|n| {
                n.checked_add(product.admitted.outputs.len() * 256 + values.scalars.len() * 128)
            })
            .and_then(|n| {
                n.checked_add(
                    product
                        .model
                        .annotations
                        .iter()
                        .map(|a| match &a.value {
                            pse_modeling::annotation::AnnotationValue::Report(label) => {
                                label.len() * 2
                                    + product.model.symbols[&a.target].lineage.path.len() * 2
                                    + 256
                            }
                            _ => 0,
                        })
                        .sum::<usize>(),
                )
            })
            .ok_or_else(|| contract_error("dynamic check storage"))?;
        let accuracy_bytes = profile
            .numerics
            .goals
            .iter()
            .try_fold(0usize, |bytes, goal| {
                bytes
                    .checked_add(size_of::<pse_math::engineering_accuracy::GoalResult>() * 2)
                    .and_then(|n| n.checked_add(goal.provenance.len().checked_mul(2)?))
                    .and_then(|n| n.checked_add(2048))
            })
            .ok_or_else(|| contract_error("dynamic accuracy goal storage"))?;
        let bytes = bytes
            .checked_add(accuracy_bytes)
            .ok_or_else(|| contract_error("dynamic accuracy goal storage"))?;
        #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
        let state_positions = pse_math::index::CheckedInventory::new(&contract.states, |id| *id)
            .map_err(math)?
            .into_positions();
        #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
        let output_positions = pse_math::index::CheckedInventory::new(&contract.outputs, |id| *id)
            .map_err(math)?
            .into_positions();
        #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
        let bytes = bytes
            .checked_add(
                (state_positions.len() + output_positions.len())
                    .checked_mul(size_of::<(SemanticId, usize)>() + 512)
                    .ok_or_else(|| contract_error("dynamic projection storage"))?,
            )
            .ok_or_else(|| contract_error("dynamic projection storage"))?;
        Ok(ModelingSimulation {
            solved: model.solved(),
            quantities: self.quantities.clone(),
            modes: vec![SimulationMode {
                name: mode_names[mode].clone(),
                model,
                numerics,
                context: DynamicMode {
                    values,
                    providers,
                    guard,
                },
                check_program,
                terminal_check_program,
                sample_scope,
                original_initial_conditions,
            }],
            contract,
            #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
            state_positions,
            #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
            output_positions,
            profile,
            snapshot: snapshot.clone(),
            runtime: self.runtime.clone(),
            source: self.clone(),
            programs: programs.into(),
            coordinates,
            parameters: parameter_values,
            key,
            bytes,
        })
    }
    /// The sign each state keeps during integration (ADR-0119 Outcome 4, review F03): a
    /// constant-zero endpoint of the state's effective authored bound, the bound annotation
    /// with any fixture override of an endpoint (the same bound the guard checks). A zero
    /// lower endpoint keeps the state non-negative, a zero upper one non-positive; with no
    /// zero endpoint, or both, the state is free. Empty when every state is free. The
    /// guard remains the validity authority; coordinates scale without offset, so a sign
    /// holds in them too.
    #[expect(
        clippy::too_many_arguments,
        reason = "the prepared model, case, state layout, values and providers are independent inputs of one observation"
    )]
    async fn state_signs(
        &self,
        model: &ModelingPreparation,
        case: &ModelingCaseBindings,
        states: &[SemanticId],
        values: &CaseValues,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Vec<native::StateSign>, WorkflowError> {
        use pse_modeling::annotation::AnnotationValue;
        #[derive(Clone, Copy)]
        enum Endpoint {
            Constant(f64),
            Row(SemanticId),
        }
        let product = model.compiled();
        let mut endpoints = vec![[None::<Endpoint>; 2]; states.len()];
        for a in &product.model.annotations {
            let AnnotationValue::Bounds(lower, upper) = &a.value else {
                continue;
            };
            let Some(i) = states.iter().position(|id| *id == a.target) else {
                continue;
            };
            for (k, (bound, kind)) in [(lower, ModelingHint::Lower), (upper, ModelingHint::Upper)]
                .into_iter()
                .enumerate()
            {
                // Only an endpoint that references no member is a constant.
                if pse_modeling::expression::references(bound).is_empty() {
                    endpoints[i][k] = Some(Endpoint::Row(
                        ModelingOutput::Hint {
                            target: a.target,
                            declaration: a.lineage.declaration,
                            kind,
                        }
                        .row_id(),
                    ));
                }
            }
        }
        for (path, state) in &case.variables {
            let Some(i) = product
                .model
                .paths
                .get(path)
                .and_then(|id| states.iter().position(|s| s == id))
            else {
                continue;
            };
            for (k, endpoint) in [state.lower, state.upper].into_iter().enumerate() {
                if let Some(endpoint) = endpoint {
                    endpoints[i][k] = endpoint.map(Endpoint::Constant);
                }
            }
        }
        let rows = endpoints
            .iter()
            .flatten()
            .filter_map(|e| match e {
                Some(Endpoint::Row(id)) => Some(*id),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let observed: BTreeMap<SemanticId, f64> = if rows.is_empty() {
            BTreeMap::new()
        } else {
            let observed = self
                .observe_registered(
                    model.clone(),
                    rows,
                    values.clone(),
                    compiler,
                    providers.clone(),
                    cancel,
                )
                .await?;
            (*observed).clone()
        };
        let zero = |endpoint: Option<Endpoint>| match endpoint {
            Some(Endpoint::Constant(v)) => v == 0.,
            Some(Endpoint::Row(id)) => observed.get(&id) == Some(&0.),
            None => false,
        };
        let signs = endpoints
            .into_iter()
            .map(|[lower, upper]| match (zero(lower), zero(upper)) {
                (true, false) => native::StateSign::NonNegative,
                (false, true) => native::StateSign::NonPositive,
                _ => native::StateSign::Free,
            })
            .collect::<Vec<_>>();
        Ok(if signs.iter().all(|s| *s == native::StateSign::Free) {
            Vec::new()
        } else {
            signs
        })
    }
    async fn prepare_dynamic_guards(
        &self,
        model: &ModelingPreparation,
        case: &ModelingCaseBindings,
        terminal_targets: &BTreeSet<SemanticId>,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Option<GuardProgram>, WorkflowError> {
        use pse_modeling::annotation::AnnotationValue;
        let product = model.compiled();
        let mut checks = Vec::new();
        let mut bounded = BTreeSet::new();
        let mut valid = BTreeSet::new();
        let overrides = case
            .variables
            .iter()
            .map(|(path, state)| (product.model.paths[path], state))
            .collect::<BTreeMap<_, _>>();
        let endpoint = |target, source, kind| {
            RangeValue::Output(
                ModelingOutput::Hint {
                    target,
                    declaration: source,
                    kind,
                }
                .row_id(),
            )
        };
        for a in &product.model.annotations {
            if terminal_targets.contains(&a.target) {
                continue;
            }
            let value = if product.admitted.inputs.contains(&a.target) {
                RangeValue::Input(a.target)
            } else {
                RangeValue::Output(ModelingOutput::Member(a.target).row_id())
            };
            let (mut lower, mut upper) = match &a.value {
                AnnotationValue::Bounds(..) => {
                    if !bounded.insert(a.target) {
                        return Err(contract("competing dynamic bounds"));
                    }
                    if !product
                        .admitted
                        .case()
                        .variables()
                        .iter()
                        .any(|v| v.port.id == a.target)
                    {
                        return Err(contract("bounds require an independent dynamic variable"));
                    }
                    (
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::Lower,
                        )),
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::Upper,
                        )),
                    )
                }
                AnnotationValue::Valid { .. } => {
                    if !valid.insert(a.target) {
                        return Err(contract("competing dynamic validity ranges"));
                    }
                    (
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::ValidLower,
                        )),
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::ValidUpper,
                        )),
                    )
                }
                _ => continue,
            };
            if matches!(a.value, AnnotationValue::Bounds(..))
                && let Some(state) = overrides.get(&a.target)
            {
                if let Some(v) = state.lower {
                    lower = v.map(RangeValue::Constant);
                }
                if let Some(v) = state.upper {
                    upper = v.map(RangeValue::Constant);
                }
            }
            checks.push(RangeCheck {
                source: a.lineage.declaration.into(),
                target: a.target,
                value,
                lower,
                upper,
            });
        }
        for (id, state) in overrides {
            if !bounded.contains(&id) && (state.lower.is_some() || state.upper.is_some()) {
                checks.push(RangeCheck {
                    source: id,
                    target: id,
                    value: RangeValue::Input(id),
                    lower: state.lower.flatten().map(RangeValue::Constant),
                    upper: state.upper.flatten().map(RangeValue::Constant),
                });
            }
        }
        if checks.is_empty() {
            return Ok(None);
        }
        let rows = checks
            .iter()
            .flat_map(|c| [Some(&c.value), c.lower.as_ref(), c.upper.as_ref()])
            .flatten()
            .filter_map(|v| {
                if let RangeValue::Output(id) = v {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        let compiled = self
            .runtime
            .shared
            .math()
            .prepare_modeling_functions(
                self.numerical_workspace()?,
                model.clone(),
                rows.into_iter().collect(),
                vec![],
                DerivativeOrder::Value,
                compiler,
                cancel,
            )
            .await?;
        let rows = compiled
            .assembly
            .structure()
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();
        Ok(Some(GuardProgram {
            case: compiled,
            rows,
            checks,
        }))
    }
}
/// The lower-endpoint start of a state without an isolated initial equation: an annotation
/// start is evaluated through the model at the endpoint (its row); any other source is the
/// resolved constant. The typed source decides; no label is parsed (F14).
fn start_row(
    outputs: &[ModelingOutput],
    state: SemanticId,
    source: Option<&StartSource>,
) -> Result<Option<SemanticId>, WorkflowError> {
    let Some(StartSource::Annotation { declaration }) = source else {
        return Ok(None);
    };
    outputs
        .iter()
        .find(|o| {
            matches!(o, ModelingOutput::Hint { target, declaration: d, kind: ModelingHint::Start }
                if *target == state && d == declaration)
        })
        .map(|o| Some(o.row_id()))
        .ok_or_else(|| contract("initial annotation output missing"))
}
fn contract_error(message: &str) -> WorkflowError {
    contract(message)
}

#[cfg(test)]
#[cfg(feature = "solver-diffsol")]
mod tests {
    use super::*;
    use pse_backend_native::dynamics::Oracle;
    #[test]
    fn start_source_drives_initial_conditions() {
        let state = SemanticId::from_bytes([1; 16]);
        let [declaration, other] = [2, 3].map(|n| DeclarationId::from_bytes([n; 16]));
        let outputs = vec![
            ModelingOutput::Hint {
                target: state,
                declaration,
                kind: ModelingHint::Start,
            },
            ModelingOutput::Hint {
                target: state,
                declaration: other,
                kind: ModelingHint::Lower,
            },
        ];
        // An annotation start is evaluated through the model: its own hint row.
        assert_eq!(
            start_row(
                &outputs,
                state,
                Some(&StartSource::Annotation { declaration })
            )
            .unwrap(),
            Some(outputs[0].row_id())
        );
        // A start annotation without its model output is refused, not frozen.
        assert!(
            start_row(
                &outputs,
                state,
                Some(&StartSource::Annotation { declaration: other })
            )
            .is_err()
        );
        // Every other source is the resolved constant, whatever its case path says.
        for source in [
            StartSource::ModelDefault,
            StartSource::Case {
                path: "annotation:x".into(),
            },
            StartSource::Predecessor,
            StartSource::Continuation,
            StartSource::Stored {
                solution: SemanticId::NIL,
            },
        ] {
            assert_eq!(start_row(&outputs, state, Some(&source)).unwrap(), None);
        }
        assert_eq!(start_row(&outputs, state, None).unwrap(), None);
    }
    fn physical() -> PhysicalContext {
        let mut physical = super::super::super::tests::physical();
        physical.preconditions = Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        );
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        physical
    }
    #[tokio::test]
    async fn goal_accuracy_dynamic_shares_one_real_comparator_for_samples_endpoint_and_integral() {
        use pse_model::generated::enums::{
            AccuracyGoalSubject, AccuracyGoalUse, AccuracyObservation, NumericalAccuracyClass,
        };
        let runtime = super::super::super::tests::runtime();
        let cancel = crate::CancelSource::new();
        let rows = pse_authoring::language::parse(
            "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == -x[i]/1{s}; eq initial: x[0{s}] == 1{s}; let clock[i in t]: Scalar = abs(i/1{s}-0.5); let state_view[i in t]: Time = 2*x[i]; let area:Time=integral(i in t | x[i]/1{s}); annotation report clock(\"point output\"); annotation report state_view(\"state-derived output\"); annotation report area(\"integral\"); } }",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical()).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let profile = native::Profile {
            method: native::Method::Diffsol,
            samples: vec![0., 0.5, 1.],
            rtol: 0.02,
            atol: vec![0.001],
            out_rtol: Some(0.02),
            out_atol: vec![0.001],
            ..Default::default()
        };
        let prepare = |profile| {
            package.prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile,
                DerivativeOrder::First,
                &cancel,
            )
        };
        let base = prepare(profile.clone()).await.unwrap();
        let ordinary = base.run(&cancel).await.unwrap();
        assert!(ordinary.completion().accuracy.is_empty());
        assert!(
            !ordinary
                .report()
                .progress
                .iter()
                .any(|e| e.phase == "accuracy.dynamic.comparator")
        );
        let state = base.contract.states[0];
        let integral = base.contract.quadratures[0];
        let symbol_named = |suffix: &str| {
            base.model()
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(suffix))
                .map(|symbol| symbol.id)
                .unwrap_or_else(|| panic!("missing dynamic output {suffix}"))
        };
        let clock = symbol_named(".clock");
        let state_view = symbol_named(".state_view");
        let mut requested = profile;
        for (index, target, observation) in [
            (1, clock, AccuracyObservation::Sample),
            (2, clock, AccuracyObservation::Endpoint),
            (3, state, AccuracyObservation::Sample),
            (4, state, AccuracyObservation::Endpoint),
            (5, state_view, AccuracyObservation::Sample),
            (6, integral, AccuracyObservation::Integrated),
        ] {
            let pse_modeling::Type::Quantity(q) =
                &base.model().compiled().model.symbols[&target].ty
            else {
                panic!("physical target");
            };
            let quantity = q.resolve(&base.quantities, &BTreeMap::new()).unwrap();
            let unit = base
                .quantities
                .quantity_type(quantity)
                .unwrap()
                .canonical_unit;
            requested
                .numerics
                .goals
                .push(pse_model::engineering_accuracy::AccuracyGoal {
                    goal_id: pse_ids::named_id(target, &format!("engineering-observation-{index}"))
                        .into(),
                    model_id: None,
                    case_id: None,
                    instance_id: None,
                    fit_id: None,
                    target_id: target,
                    target_kind: if target == state {
                        NumericalTarget::Variable
                    } else {
                        NumericalTarget::Observable
                    },
                    quantity_id: quantity.as_id(),
                    unit_id: unit.as_id(),
                    subject: AccuracyGoalSubject::SelectedOutput,
                    observation,
                    time: (observation == AccuracyObservation::Sample).then_some(0.5),
                    resolution: Some(1.),
                    criterion_lower: None,
                    criterion_upper: Some(2.),
                    required_class: NumericalAccuracyClass::Estimated,
                    use_policy: if target == clock {
                        AccuracyGoalUse::RequireSatisfied
                    } else {
                        AccuracyGoalUse::Assess
                    },
                    refine: false,
                    source: NumericalSource::Analysis,
                    priority: 0,
                    provenance: "Process-scale time output decision".into(),
                });
        }
        let prepared = prepare(requested).await.unwrap();
        let joined = prepared.run(&cancel).await.unwrap();
        assert!(
            !joined.accepted(),
            "the unresolved integrated goal must remain visible"
        );
        assert_eq!(
            joined.report().termination,
            native::Termination::Completed,
            "the successful base trajectory remains available despite unresolved Assess evidence"
        );
        assert_eq!(joined.completion().accuracy.len(), 6);
        for observation in [AccuracyObservation::Sample, AccuracyObservation::Endpoint] {
            let selected = joined
                .completion()
                .accuracy
                .iter()
                .find(|goal| goal.goal.target_id == clock && goal.goal.observation == observation)
                .unwrap();
            assert_eq!(
                selected.classification.status,
                pse_model::generated::enums::AccuracyGoalStatus::Satisfied,
                "state-independent dynamic {observation:?} goal did not satisfy; completion={:#?}",
                joined.completion()
            );
            assert!(
                selected
                    .evidence
                    .as_ref()
                    .is_some_and(|e| e.accuracy.class == NumericalAccuracyClass::Estimated)
            );
        }
        for (target, observation) in [
            (state, AccuracyObservation::Sample),
            (state, AccuracyObservation::Endpoint),
            (state_view, AccuracyObservation::Sample),
        ] {
            let selected = joined
                .completion()
                .accuracy
                .iter()
                .find(|goal| goal.goal.target_id == target && goal.goal.observation == observation)
                .unwrap();
            assert_eq!(
                selected.classification.unavailable,
                Some(pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty),
                "state-derived {target} {observation:?} must retain generated affine-rate uncertainty"
            );
        }
        let integrated = joined
            .completion()
            .accuracy
            .iter()
            .find(|goal| goal.goal.observation == AccuracyObservation::Integrated)
            .unwrap();
        assert_eq!(
            integrated.classification.unavailable,
            Some(pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty)
        );
        assert_eq!(
            joined
                .report()
                .progress
                .iter()
                .filter(|e| e.phase == "accuracy.dynamic.comparator")
                .count(),
            1
        );
        assert_eq!(
            joined
                .report()
                .statistics
                .iter()
                .filter(|s| s["accuracy_occurrence"] == 1)
                .count(),
            1
        );
        assert_eq!(prepared.profile.numerics.goals[0].resolution, Some(1.));
        assert_eq!(prepared.profile.rtol, 0.02);
        assert_eq!(
            joined
                .report()
                .samples
                .iter()
                .map(|s| s.time)
                .collect::<Vec<_>>(),
            vec![0., 0.5, 1.]
        );
    }
    #[tokio::test]
    async fn trajectory_transport_shares_completion_retries_budget_and_retains_escaped_batches() {
        use pse_columnar::MemoryConsumer;
        let (batch, pool) = {
            let runtime = super::super::super::tests::runtime();
            let cancel = crate::CancelSource::new();
            let rows = pse_authoring::language::parse(
                "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 0.5; var x[i in t]: Time; conserve stock[i in t]: Time on t inventory x[i] flux p tolerance 1e-6{s}; eq initial: x[0{s}] == 1{s}; } }",
                SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            ).unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime.modeling_package(rows, physical()).await.unwrap();
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    super::super::super::tests::compiler_profile(),
                    native::Profile {
                        method: native::Method::Diffsol,
                        samples: vec![0., 0.5, 1.],
                        parameter_scales: vec![1.],
                        out_rtol: Some(1e-8),
                        out_atol: vec![1e-10],
                        ..Default::default()
                    },
                    DerivativeOrder::First,
                    &cancel,
                )
                .await
                .unwrap();
            let result = prepared.start().unwrap().wait().await.unwrap();
            let crate::workflow::RunReport::Simulation(trajectory) = result.report().unwrap()
            else {
                panic!("simulation expected");
            };
            let trajectory = trajectory.as_ref();
            assert_eq!(
                result.completion().unwrap().computation.as_ref().unwrap(),
                trajectory.header()
            );
            let cloned = trajectory.clone();
            assert!(Arc::ptr_eq(&trajectory.inner, &cloned.inner));
            assert!(trajectory.accepted());
            assert_eq!(
                trajectory.header().qualification,
                pse_model::generated::enums::NativeQualification::Feasible
            );
            assert!(trajectory.diagnostic().is_none());
            assert!(trajectory.inner.tables.lock().unwrap().is_none());
            let pool = runtime.shared.pool();
            let before = pool.reserved();
            let pressure = MemoryConsumer::new("test:trajectory-pressure").register(&pool);
            pressure
                .try_grow(runtime.shared.budget().memory_limit_bytes.get() - before)
                .unwrap();
            assert!(trajectory.tables().is_err());
            assert!(result.tables().is_err());
            assert!(trajectory.inner.tables.lock().unwrap().is_none());
            drop(pressure);
            assert_eq!(pool.reserved(), before);
            let maps = std::thread::scope(|scope| {
                (0..4)
                    .map(|_| scope.spawn(|| trajectory.tables().unwrap()))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|thread| thread.join().unwrap())
                    .collect::<Vec<_>>()
            });
            assert!(maps.iter().all(|m| Arc::ptr_eq(m, &maps[0])));
            assert!(Arc::ptr_eq(&maps[0], &cloned.tables().unwrap()));
            assert!(
                result.tables().is_err(),
                "outer RunResult retains its failed encoding"
            );
            // Equal supplied identities do not make distinct attempts share mutable transport.
            let ((other_report, other_checks, other_accuracy), other_owner) = prepared
                .submit(trajectory.run_id(), crate::math::Submission::ephemeral())
                .unwrap()
                .finish()
                .await
                .unwrap();
            let other = prepared
                .finish(
                    trajectory.run_id(),
                    other_report,
                    other_checks,
                    other_accuracy,
                    other_owner,
                )
                .unwrap();
            assert_eq!(other.run_id(), trajectory.run_id());
            assert!(!Arc::ptr_eq(&other.inner, &trajectory.inner));
            assert!(!Arc::ptr_eq(&other.tables().unwrap(), &maps[0]));
            use pse_relations::{columnar::RelationRow, generated::runtime::candidate_assessments};
            assert_eq!(
                candidate_assessments::Row::rows(&maps[0][&candidate_assessments::RELATION_ID])
                    .unwrap(),
                vec![trajectory.assessment().clone()]
            );
            assert_eq!(result.assessments, vec![trajectory.assessment().clone()]);
            let batch = trajectory.table("runtime.simulation_samples").unwrap();
            assert!(batch.batch().num_rows() > 0);
            assert!(trajectory.table("runtime.solve_variables").is_err());
            assert!(trajectory.table("missing.table").is_err());
            assert_eq!(
                trajectory.header().trajectory_termination,
                Some(trajectory.report().termination)
            );
            (batch, pool)
        };
        assert!(batch.batch().num_rows() > 0);
        assert!(pool.reserved() > 0);
        drop(batch);
        assert_eq!(pool.reserved(), 0);
    }

    #[tokio::test]
    async fn contextual_closure_inventory_controls_preserve_explicit_profile_origin() {
        use super::super::super::tests as fixture;
        let source = r#"package p {
            entity kind source provenance {attribute title:Text;}
            enum role {given}
            entity source s {title="inventory control"}
            constant allowance:Time=1{s} provenance(s,role.given);
            annotation engineering_rule p.allowance;
            def Root {domain t:Time from 0{s} to 1{s};
                discretize grid on t using integrated(elements=1,order=1);
                param rate:Scalar=0.5;var x[i in t]:Time;
                annotation engineering_scale x[0{s}](kind=magnitude,value=10000{s});
                conserve stock[i in t]:Time on t inventory x[i] flux rate tolerance p.allowance;
                eq initial:x[0{s}]==1{s};
            }
            test dynamic fixture {dof 0;route integrated;procedure integrate;
                integrate samples(0{s},0.5{s},1{s}) relative(global) normalized_absolute(global) step(1e-4{s});
            } {child root:Root=Root();}
        }"#;
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "dynamic")
            .unwrap()
            .declaration_id;
        let package = fixture::runtime()
            .modeling_package(rows, physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let automatic = package
            .declared_simulation(
                root,
                fixture::compiler_profile(),
                None,
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(automatic.profile().out_atol, vec![10.0]);
        assert!(
            automatic
                .contract
                .balances
                .iter()
                .all(|balance| balance.tolerance == 10.0)
        );
        let mut explicit = automatic.profile().clone();
        explicit.out_atol = vec![1.0];
        let declared = package
            .declared_simulation(
                root,
                fixture::compiler_profile(),
                Some(explicit.clone()),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(declared.profile().out_atol, vec![1.0]);
        assert!(
            declared
                .contract
                .balances
                .iter()
                .all(|balance| balance.tolerance == 10.0)
        );
        let direct = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                fixture::compiler_profile(),
                explicit,
                DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(direct.profile().out_atol, vec![1.0]);
        assert!(
            direct
                .contract
                .balances
                .iter()
                .all(|balance| balance.tolerance == 10.0)
        );
    }
    #[tokio::test]
    async fn authored_conservation_derives_only_its_generated_flux_quadrature_controls() {
        let runtime = super::super::super::tests::runtime();
        let cancel = crate::CancelSource::new();
        for unrelated_integral in [false, true] {
            let extra = if unrelated_integral {
                "let area:Time=integral(i in t | 2); annotation check area(area>1.9{s});"
            } else {
                ""
            };
            let source = format!(
                "package p {{ def Root {{ domain t: Time from 0{{s}} to 1{{s}}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 0.5; var x[i in t]: Time; conserve stock[i in t]: Time on t inventory x[i] flux p tolerance 1e-6{{s}}; eq initial: x[0{{s}}] == 1{{s}}; {extra} }} test dynamic fixture {{ dof 0; route integrated; procedure integrate; integrate samples(0{{s}},0.5{{s}},1{{s}}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{{s}}); }} {{ child root: Root = Root(); }} }}"
            );
            let rows = pse_authoring::language::parse(
                &source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "dynamic")
                .unwrap()
                .declaration_id;
            let package = runtime.modeling_package(rows, physical()).await.unwrap();
            let prepared = package
                .declared_simulation(
                    root,
                    super::super::super::tests::compiler_profile(),
                    None,
                    Limits::default(),
                    &cancel,
                )
                .await;
            if unrelated_integral {
                let error = prepared.err().unwrap();
                assert!(
                    error.to_string().contains(
                        "each integrated quadrature requires an explicit physical tolerance"
                    ),
                    "{error:?}"
                );
            } else {
                let prepared = prepared.unwrap();
                assert_eq!(prepared.profile.out_rtol, Some(1e-8));
                assert_eq!(prepared.profile.out_atol, vec![1e-6]);
                let trajectory = prepared.run(&cancel).await.unwrap();
                assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
            }
        }
    }
    #[tokio::test]
    async fn declared_terminal_endpoint_qualifies_conservation_prefix_without_fabricating_integrals()
     {
        let runtime = super::super::super::tests::runtime();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let allowance =
            pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            for (endpoint, integral, accepted) in [
                (
                    "endpoint declared_terminal_event(root.hit[0{s}]);",
                    "",
                    true,
                ),
                ("endpoint fixed_horizon;", "", false),
                (
                    "endpoint declared_terminal_event(root.hit[0{s}]);",
                    "let total: Time = integral(i in t | p); annotation report total(\"whole-domain\");",
                    false,
                ),
            ] {
                let quadrature = if integral.is_empty() {
                    String::new()
                } else {
                    format!(
                        "quadrature_relative({allowance}) quadrature_absolute(root.total={allowance}{{s}})"
                    )
                };
                let source = format!(
                    "package p {{ def Root {{ domain t:Time from 0{{s}} to 1{{s}}; discretize grid on t using integrated(elements=1,order=1); param p:Scalar=0.5; var x[i in t]:Time; conserve stock[i in t]:Time on t inventory x[i] flux p tolerance {allowance}{{s}}; eq initial:x[0{{s}}]==1{{s}}; let hit[i in t]:Time=x[i]-1.25{{s}}; {integral} }} test stopped fixture {{ dof 0; route integrated; procedure integrate; {endpoint} integrate samples(0{{s}},0.2{{s}},0.8{{s}},1{{s}}) relative({allowance}) normalized_absolute({allowance}) step(1e-4{{s}}) {quadrature}; mode arc; event root.hit[0{{s}}] direction(either) tolerance({allowance}{{s}}) terminal; }} {{ child root:Root=Root(); }} }}"
                );
                let rows = pse_authoring::language::parse(
                    &source,
                    SemanticId::NIL,
                    pse_authoring::language::IdentityPolicy::Named,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap();
                let root = rows
                    .iter()
                    .find(|r| r.name == "stopped")
                    .unwrap()
                    .declaration_id;
                let package = runtime.modeling_package(rows, physical()).await.unwrap();
                let prepared = package
                    .declared_simulation(root, compiler, None, Limits::default(), &cancel)
                    .await
                    .unwrap();
                if accepted {
                    let resetting = source.replace(
                        &format!("tolerance({allowance}{{s}}) terminal;"),
                        &format!("tolerance({allowance}{{s}}) reset(root.x[0{{s}}]=root.x[0{{s}}]) terminal;"),
                    );
                    let rows = pse_authoring::language::parse(
                        &resetting,
                        SemanticId::NIL,
                        pse_authoring::language::IdentityPolicy::Named,
                        pse_authoring::ParseBudget::default(),
                    )
                    .unwrap();
                    let resetting = runtime.modeling_package(rows, physical()).await.unwrap();
                    let error = resetting
                        .declared_simulation(root, compiler, None, Limits::default(), &cancel)
                        .await
                        .unwrap_err();
                    assert!(
                        error.to_string().contains("terminal without resets"),
                        "{error}"
                    );
                }
                let mut profile = prepared.profile().clone();
                profile.method = method;
                assert_eq!(profile.rtol, allowance);
                assert!(profile.atol.iter().all(|value| *value == allowance));
                assert_eq!(prepared.contract.balances[0].tolerance, allowance);
                assert!(profile.out_atol.iter().all(|value| *value == allowance));
                let prepared = package
                    .declared_simulation(
                        root,
                        compiler,
                        Some(profile.clone()),
                        Limits::default(),
                        &cancel,
                    )
                    .await
                    .unwrap();
                if accepted {
                    profile.endpoint = native::EndpointRequirement::default();
                    let error = package
                        .declared_simulation(
                            root,
                            compiler,
                            Some(profile),
                            Limits::default(),
                            &cancel,
                        )
                        .await
                        .unwrap_err();
                    assert!(error.to_string().contains("authored endpoint"), "{error}");
                }
                let trajectory = prepared.run(&cancel).await.unwrap();
                assert_eq!(
                    trajectory.report().termination,
                    native::Termination::Event,
                    "{:?}",
                    trajectory.report().error
                );
                let inventory = prepared.contract.states[0];
                let budget = super::super::super::tests::engineering_target(
                    prepared.numerics(),
                    NumericalTarget::Variable,
                    inventory,
                )
                .budget;
                assert_eq!(budget, allowance);
                assert!((trajectory.report().completed_time - 0.5).abs() <= budget / 0.5);
                assert_eq!(trajectory.report().samples.len(), 2);
                assert_eq!(
                    trajectory.accepted(),
                    accepted,
                    "{:?}; {:?}",
                    trajectory.completion(),
                    trajectory.validation_error()
                );
                assert_eq!(trajectory.checks_complete(), accepted);
                assert_eq!(
                    trajectory.header().qualification
                        == pse_model::generated::enums::NativeQualification::Feasible,
                    accepted
                );
                assert_eq!(trajectory.diagnostic().is_none(), accepted);
                assert!(
                    trajectory
                        .reports()
                        .iter()
                        .all(|r| r.label != "whole-domain")
                );
                assert_eq!(
                    trajectory.report().conservation.last().unwrap().time,
                    trajectory.report().completed_time
                );
                let endpoint = trajectory.report().endpoint.as_ref().unwrap();
                assert!((endpoint.point.outputs[0] - 1.25).abs() <= budget);
                let tables = trajectory.tables().unwrap();
                assert!(tables.contains_key(
                    &pse_relations::generated::runtime::trajectory_endpoints::RELATION_ID
                ));
                if accepted {
                    assert_eq!(
                        trajectory.completion().closure,
                        pse_model::generated::enums::ClosureAssessment::Closed
                    );
                }
            }
        }
    }
    #[tokio::test]
    async fn authored_conservation_inventory_and_original_flux_reach_native_closure_reports() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            let source = "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 0.5; var x[i in t]: Time; conserve stock[i in t]: Time on t inventory x[i] flux p tolerance 1e-6{s}; eq initial: x[0{s}] == 1{s}; } }";
            let rows = pse_authoring::language::parse(
                source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(rows, physical.clone())
                .await
                .unwrap();
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    native::Profile {
                        method,
                        samples: vec![0.0, 0.5, 1.0],
                        parameter_scales: vec![1.0],
                        rtol: 1e-9,
                        atol: vec![1e-11],
                        out_rtol: Some(1e-9),
                        out_atol: vec![1e-11],
                        ..Default::default()
                    },
                    DerivativeOrder::First,
                    &cancel,
                )
                .await
                .unwrap();
            let descriptor = prepared
                .model()
                .compiled()
                .model
                .inventory_balances
                .values()
                .next()
                .unwrap();
            assert_eq!(prepared.contract.balances.len(), 1);
            assert_eq!(prepared.contract.balances[0].id, descriptor.id);
            assert_eq!(
                prepared.contract.balances[0].inventory,
                descriptor.inventory_id
            );
            assert_eq!(prepared.contract.balances[0].flux, descriptor.flux_id);
            assert!(
                prepared
                    .model()
                    .compiled()
                    .admitted
                    .outputs
                    .contains(&ModelingOutput::Inventory(descriptor.id))
            );
            let trajectory = prepared.run(&cancel).await.unwrap();
            assert_eq!(
                trajectory.report().termination,
                native::Termination::Completed,
                "{:?}",
                trajectory.report().error
            );
            assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
            assert!((trajectory.report().samples.last().unwrap().state[0] - 1.5).abs() < 1e-7);
            let checks = trajectory
                .checks()
                .iter()
                .filter(|c| c.kind == pse_model::generated::enums::ModelingCheckKind::Closure)
                .collect::<Vec<_>>();
            assert!(!checks.is_empty());
            assert!(checks.iter().all(|c| c.target_id == descriptor.id
                && c.satisfied
                && c.time.is_some()
                && c.tolerance == Some(1e-6)));
            assert!(
                trajectory
                    .report()
                    .conservation
                    .iter()
                    .all(|p| p.defects[0].abs() < 1e-6)
            );
        }
    }
    #[tokio::test]
    async fn authored_conservation_general_inventory_audits_original_coordinates_and_exposes_flux_drift()
     {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            for (flux, closes) in [("p*exp(x[i]/1{s})", true), ("0", false)] {
                // Diffsol's original nonlinear flux quadrature has about 3.6e-6 s error
                // under these fixed controls; the physical 1e-5 s budget admits that
                // independently observed error while zero authored flux misses by 0.649 s.
                let source = format!(
                    "package p {{ def Root {{ domain t: Time from 0{{s}} to 1{{s}}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 0.5; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[0{{s}}] == 0{{s}}; conserve stock[i in t]: Time on t inventory 1{{s}}*exp(x[i]/1{{s}}) flux {flux} tolerance 1e-5{{s}}; }} }}"
                );
                let rows = pse_authoring::language::parse(
                    &source,
                    SemanticId::NIL,
                    pse_authoring::language::IdentityPolicy::Named,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap();
                let root = rows
                    .iter()
                    .find(|r| r.name == "Root")
                    .unwrap()
                    .declaration_id;
                let package = runtime
                    .modeling_package(rows, physical.clone())
                    .await
                    .unwrap();
                let prepared = package
                    .prepare_simulation(
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Bindings::default(),
                        Limits::default(),
                        ModelingCaseBindings::default(),
                        compiler,
                        native::Profile {
                            method,
                            samples: vec![0.0, 0.5, 1.0],
                            parameter_scales: vec![1.0],
                            initial_step: 1e-3,
                            rtol: 1e-9,
                            atol: vec![1e-11],
                            out_rtol: Some(1e-9),
                            out_atol: vec![1e-11],
                            ..Default::default()
                        },
                        DerivativeOrder::First,
                        &cancel,
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    prepared.contract.states.len(),
                    1,
                    "original independent differential coordinate remains"
                );
                let trajectory = prepared.run(&cancel).await.unwrap();
                assert_eq!(
                    trajectory.report().termination,
                    native::Termination::Completed,
                    "{:?}",
                    trajectory.report().error
                );
                assert_eq!(
                    trajectory.accepted(),
                    closes,
                    "{:?} complete={} conservation={:?}",
                    trajectory.validation_error(),
                    trajectory.checks_complete(),
                    trajectory.report().conservation
                );
                let assessment = trajectory.completion();
                assert_eq!(
                    assessment.closure,
                    if closes {
                        pse_model::generated::enums::ClosureAssessment::Closed
                    } else {
                        pse_model::generated::enums::ClosureAssessment::Unclosed
                    }
                );
                assert!((trajectory.report().conservation[0].inventories[0] - 1.0).abs() < 1e-8);
                let last = trajectory.report().conservation.last().unwrap();
                assert!((last.inventories[0] - 0.5_f64.exp()).abs() < 1e-7);
                if closes {
                    assert!(last.defects[0].abs() < 1e-5);
                } else {
                    assert!((last.defects[0] - (0.5_f64.exp() - 1.0)).abs() < 1e-7);
                }
                assert!(
                    trajectory
                        .checks()
                        .iter()
                        .any(|c| c.kind == pse_model::generated::enums::ModelingCheckKind::Closure)
                );
            }
        }
    }
    #[tokio::test]
    async fn authored_conservation_auxiliary_inventory_initializes_original_algebraic_coordinates()
    {
        let runtime = super::super::super::tests::runtime();
        let cancel = crate::CancelSource::new();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            for initial_y_value in [Some(2), Some(4), None] {
                let initial_y = initial_y_value
                    .map(|value| format!("eq initial_y:y[0{{s}}]=={value}{{s}};"))
                    .unwrap_or_default();
                let source = format!(
                    "package p {{ def Root {{ domain t:Time from 0{{s}} to 1{{s}}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; var y[i in t]:Time; eq definition[i in t]:y[i]==2*x[i]; eq initial_x:x[0{{s}}]==1{{s}}; {initial_y} conserve stock[i in t]:Time on t inventory x[i]+y[i] flux 3 tolerance 1e-6{{s}}; }} }}"
                );
                let rows = pse_authoring::language::parse(
                    &source,
                    SemanticId::NIL,
                    pse_authoring::language::IdentityPolicy::Named,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap();
                let root = rows
                    .iter()
                    .find(|r| r.name == "Root")
                    .unwrap()
                    .declaration_id;
                let package = runtime.modeling_package(rows, physical()).await.unwrap();
                let prepared = package
                    .prepare_simulation(
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Bindings::default(),
                        Limits::default(),
                        ModelingCaseBindings::default(),
                        super::super::super::tests::compiler_profile(),
                        native::Profile {
                            method,
                            samples: vec![0., 0.5, 1.],
                            rtol: 1e-9,
                            atol: vec![1e-11; 3],
                            out_rtol: Some(1e-9),
                            out_atol: vec![1e-11],
                            ..Default::default()
                        },
                        DerivativeOrder::First,
                        &cancel,
                    )
                    .await;
                if initial_y_value.is_none() {
                    assert!(
                        prepared
                            .err()
                            .unwrap()
                            .to_string()
                            .contains("explicit initial conditions")
                    );
                    continue;
                }
                let prepared = prepared.unwrap();
                assert_eq!(prepared.contract.states.len(), 3);
                assert_eq!(
                    prepared
                        .contract
                        .differential
                        .iter()
                        .filter(|value| **value)
                        .count(),
                    1
                );
                let descriptor = prepared
                    .model()
                    .compiled()
                    .model
                    .inventory_balances
                    .values()
                    .next()
                    .unwrap();
                assert!(descriptor.state.is_some());
                let trajectory = prepared.run(&cancel).await.unwrap();
                assert_eq!(
                    trajectory.report().termination,
                    native::Termination::Completed,
                    "{:?}",
                    trajectory.report().error
                );
                if initial_y_value == Some(4) {
                    assert!(
                        !trajectory.accepted(),
                        "inconsistent original coordinate initial rows must refuse result use"
                    );
                    assert!(
                        trajectory.checks_complete() && trajectory.validation_error().is_none()
                    );
                    let original_sources = prepared
                        .model()
                        .compiled()
                        .model
                        .inventory_initial_conditions
                        .values()
                        .map(|condition| condition.lineage.declaration)
                        .collect::<BTreeSet<_>>();
                    assert!(trajectory.checks().iter().any(|check| {
                        check.kind
                            == pse_model::generated::enums::ModelingCheckKind::OriginalEquation
                            && original_sources.contains(&check.source_id)
                            && !check.satisfied
                            && check.time == Some(0.)
                            && check.tolerance.is_some()
                    }));
                    assert!(
                        trajectory
                            .report()
                            .conservation
                            .iter()
                            .all(|point| point.defects[0].abs() < 1e-6)
                    );
                    continue;
                }
                assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
                assert!((trajectory.report().conservation[0].inventories[0] - 3.).abs() < 1e-7);
                assert!(
                    (trajectory.report().conservation.last().unwrap().inventories[0] - 6.).abs()
                        < 1e-7
                );
                assert!(
                    trajectory
                        .report()
                        .conservation
                        .iter()
                        .all(|point| point.defects[0].abs() < 1e-6)
                );
            }
        }
    }
    #[tokio::test]
    async fn authored_conservation_shared_coordinates_keep_each_original_initial_source() {
        let runtime = super::super::super::tests::runtime();
        let cancel = crate::CancelSource::new();
        let source = "package p { def Root { domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; var y[i in t]:Time; eq initial_x:x[0{s}]==1{s}; eq initial_y:y[0{s}]==2{s}; conserve sum[i in t]:Time on t inventory x[i]+y[i] flux 3 tolerance 1e-6{s}; conserve difference[i in t]:Time on t inventory x[i]-y[i] flux -1 tolerance 1e-6{s}; } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let initial_sources = rows
            .iter()
            .filter(|row| row.name.starts_with("initial_"))
            .map(|row| row.declaration_id)
            .collect::<BTreeSet<_>>();
        let package = runtime.modeling_package(rows, physical()).await.unwrap();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    super::super::super::tests::compiler_profile(),
                    native::Profile {
                        method,
                        samples: vec![0., 0.5, 1.],
                        rtol: 1e-9,
                        atol: vec![1e-11; 4],
                        out_rtol: Some(1e-9),
                        out_atol: vec![1e-11; 2],
                        ..Default::default()
                    },
                    DerivativeOrder::First,
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(prepared.contract.balances.len(), 2);
            assert_eq!(
                prepared
                    .contract
                    .differential
                    .iter()
                    .filter(|value| **value)
                    .count(),
                2
            );
            assert_eq!(
                prepared
                    .model()
                    .compiled()
                    .model
                    .annotations
                    .iter()
                    .filter(|hint| initial_sources.contains(&hint.lineage.declaration)
                        && matches!(
                            hint.value,
                            pse_modeling::annotation::AnnotationValue::Start(_)
                        ))
                    .count(),
                2
            );
            let trajectory = prepared.run(&cancel).await.unwrap();
            assert_eq!(
                trajectory.report().termination,
                native::Termination::Completed,
                "{:?}",
                trajectory.report().error
            );
            assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
            let mut initial = trajectory.report().conservation[0].inventories.clone();
            let mut last = trajectory
                .report()
                .conservation
                .last()
                .unwrap()
                .inventories
                .clone();
            initial.sort_by(f64::total_cmp);
            last.sort_by(f64::total_cmp);
            for (observed, expected) in initial.iter().zip([-1., 3.]) {
                assert!((observed - expected).abs() < 1e-7);
            }
            for (observed, expected) in last.iter().zip([-2., 6.]) {
                assert!((observed - expected).abs() < 1e-7);
            }
            assert!(
                trajectory
                    .report()
                    .conservation
                    .iter()
                    .all(|point| point.defects.iter().all(|defect| defect.abs() < 1e-6))
            );
        }
    }
    #[tokio::test]
    async fn authored_conservation_transfers_resolve_guard_names_and_preserve_state_dependence() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            for (expression, matches, guard_matches) in [
                ("x[i]", true, true),
                ("0.5*x[i]", false, true),
                ("x[i]", true, false),
            ] {
                let event_guard = if guard_matches { "hit" } else { "other" };
                let source = format!(
                    "package p {{ def Root {{ domain t: Time from 0{{s}} to 1{{s}}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 1; var x[i in t]: Time; conserve stock[i in t]: Time on t inventory x[i] flux p tolerance 1e-6{{s}} transfers(hit[0{{s}}] = {expression}); eq initial: x[0{{s}}] == 1{{s}}; let hit[i in t]: Time = x[i]-1.5{{s}}; let other[i in t]: Time = x[i]-1.5{{s}}; let jump[i in t]: Time = 2*x[i]; }} def Assembly {{ child renamed: Root = Root(); }} test evented fixture {{ dof 0; route integrated; procedure integrate; integrate samples(0{{s}},0.25{{s}},0.75{{s}},1{{s}}) relative(1e-9) normalized_absolute(1e-11) step(1e-4{{s}}); mode before; event plant.renamed.{event_guard}[0{{s}}] direction(either) tolerance(1e-8{{s}}) reset(plant.renamed.x[0{{s}}] = plant.renamed.jump[0{{s}}]) next(after); mode after; }} {{ child plant: Assembly = Assembly(); }} }}"
                );
                let rows = pse_authoring::language::parse(
                    &source,
                    SemanticId::NIL,
                    pse_authoring::language::IdentityPolicy::Named,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap();
                let root = rows
                    .iter()
                    .find(|r| r.name == "evented")
                    .unwrap()
                    .declaration_id;
                let stock_source = rows
                    .iter()
                    .find(|row| row.name == "stock")
                    .unwrap()
                    .declaration_id;
                let package = runtime
                    .modeling_package(rows, physical.clone())
                    .await
                    .unwrap();
                let prepared = package
                    .declared_simulation(
                        root,
                        compiler,
                        Some(native::Profile {
                            method,
                            samples: vec![0.0, 0.25, 0.75, 1.0],
                            parameter_scales: vec![1.0],
                            rtol: 1e-9,
                            atol: vec![1e-11],
                            out_rtol: Some(1e-9),
                            out_atol: vec![1e-11],
                            ..Default::default()
                        }),
                        Limits::default(),
                        &cancel,
                    )
                    .await;
                if !guard_matches {
                    let WorkflowError::Boundary(refusal) = prepared.unwrap_err() else {
                        panic!("a foreign guard permission must retain an attributable refusal");
                    };
                    assert_eq!(
                        refusal.rule,
                        pse_diagnostics::DiagnosticRule::ModelingDynamicInventoryTransferEventUnsupported
                    );
                    assert_eq!(refusal.sources.len(), 3);
                    assert!(refusal.sources.contains(&stock_source.into()));
                    continue;
                }
                let prepared = prepared.unwrap();
                assert_eq!(prepared.contract.balances[0].transfers.len(), 1);
                assert!(
                    prepared.contract.balances[0]
                        .transfers
                        .contains(&prepared.contract.events[0][0].id)
                );
                let descriptor = prepared
                    .model()
                    .compiled()
                    .model
                    .inventory_balances
                    .values()
                    .next()
                    .unwrap();
                assert_eq!(
                    descriptor
                        .transfers
                        .keys()
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    prepared.contract.balances[0].transfers
                );
                assert!(
                    descriptor.lineage.path.contains("plant")
                        && descriptor.lineage.path.contains("renamed")
                );
                assert!(
                    prepared
                        .programs
                        .iter()
                        .any(|p| p.function == Function::Transfer(0))
                );
                let trajectory = prepared.run(&cancel).await.unwrap();
                assert_eq!(
                    trajectory.accepted(),
                    matches,
                    "{:?} {:?}",
                    trajectory.report().error,
                    trajectory.validation_error()
                );
                if matches {
                    assert_eq!(
                        trajectory.report().termination,
                        native::Termination::Completed
                    );
                    assert!(
                        (trajectory.report().conservation.last().unwrap().transfers[0] - 1.5).abs()
                            < 1e-7
                    );
                    assert!(
                        trajectory
                            .report()
                            .conservation
                            .iter()
                            .all(|p| p.defects[0].abs() < 1e-6)
                    );
                } else {
                    assert_eq!(trajectory.report().termination, native::Termination::Failed);
                    assert!(
                        trajectory
                            .report()
                            .error
                            .as_ref()
                            .unwrap()
                            .to_string()
                            .contains("permitted event transfer")
                    );
                    assert!(trajectory.checks().iter().any(|c| c.kind
                        == pse_model::generated::enums::ModelingCheckKind::Closure
                        && c.source_id == descriptor.lineage.declaration
                        && c.target_id == descriptor.id
                        && !c.satisfied));
                }
            }
        }
    }
    #[tokio::test]
    async fn authored_conservation_point_inventory_uses_difference_transfer_convention() {
        let runtime = super::super::super::tests::runtime();
        let cancel = crate::CancelSource::new();
        let mut physical = physical();
        // A named physical rate boundary is required independently of conservation.
        // Extend the fixture registry through ordinary atomic admission, preserving its rules.
        let delta = physical
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some("DeltaTemperature"))
            .unwrap()
            .clone();
        let time = physical
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some("Time"))
            .unwrap()
            .clone();
        let reciprocal = pse_quantity::Ratio::new(-1, 1).unwrap();
        let mut builder = physical.quantities.to_builder();
        builder.unit(pse_quantity::Unit {
            id: pse_quantity::UnitId::from_id(pse_ids::named_id(
                SemanticId::NIL,
                "fixture-minute-unit",
            )),
            symbol: "fixture_min".into(),
            dimension: physical
                .quantities
                .unit(time.canonical_unit)
                .unwrap()
                .dimension,
            scale_to_canonical: 60.,
            offset_to_canonical: 0.,
            is_affine: false,
            reference_state: None,
            definition: None,
        });
        let with_minute = builder.build().unwrap();
        let rate_unit = with_minute
            .compose(
                &pse_quantity::UnitProduct::from_factors([
                    ("K".into(), pse_quantity::Ratio::ONE),
                    ("fixture_min".into(), reciprocal),
                ])
                .unwrap(),
            )
            .unwrap();
        let mut builder = with_minute.to_builder();
        if with_minute.unit(rate_unit.id).is_err() {
            builder.defined_unit(pse_quantity::unit::DefinedUnit {
                id: rate_unit.id,
                symbol: "K/fixture_min".into(),
                composition: rate_unit.definition.clone().unwrap(),
            });
        }
        let rate_kind = pse_quantity::QuantityKindId::from_id(pse_ids::named_id(
            SemanticId::NIL,
            "fixture-temperature-rate-kind",
        ));
        let rate_type = pse_quantity::QuantityTypeId::from_id(pse_ids::named_id(
            SemanticId::NIL,
            "fixture-temperature-rate-type",
        ));
        let rate_contract = pse_quantity::InvariantId::from_id(pse_ids::named_id(
            SemanticId::NIL,
            "fixture-temperature-rate-operand",
        ));
        let interval_contract = pse_quantity::InvariantId::from_id(pse_ids::named_id(
            SemanticId::NIL,
            "fixture-temperature-interval-operand",
        ));
        builder
            .derived_kind(pse_quantity::DerivedKind {
                id: rate_kind,
                extensive: false,
                addition_kind: pse_quantity::QuantityAdditionKind::Additive,
                definition: pse_quantity::KindDefinition {
                    monomial: vec![
                        pse_quantity::KindFactor {
                            kind: delta.key.kind,
                            exponent: pse_quantity::Ratio::ONE,
                        },
                        pse_quantity::KindFactor {
                            kind: time.key.kind,
                            exponent: reciprocal,
                        },
                    ],
                    canonical_unit: rate_unit.id,
                    basis: None,
                    reference_state: None,
                    scale_kind: pse_quantity::ScaleKind::Point,
                    subject_kind: None,
                },
            })
            .quantity_type(pse_quantity::QuantityType {
                id: rate_type,
                name: Some("TemperatureRate".into()),
                key: pse_quantity::QuantityTypeKey {
                    kind: rate_kind,
                    scale_kind: pse_quantity::ScaleKind::Point,
                    ..delta.key.clone()
                },
                canonical_unit: rate_unit.id,
                nominal_magnitude: None,
            });
        // An origin-sensitive temperature increment requires an explicitly admitted
        // inverse operation, with both complete operand contracts retained.
        builder.operation(pse_quantity::QuantityOperation {
            id: pse_quantity::OperationId::from_id(pse_ids::named_id(
                SemanticId::NIL,
                "fixture-temperature-rate-integral",
            )),
            opcode: pse_quantity::Opcode::Mul,
            input_kinds: vec![rate_kind, time.key.kind],
            result_kind: delta.key.kind,
            basis_rule: pse_quantity::BasisRule::DeclaredResult,
            reference_rule: pse_quantity::ReferenceRule::DeclaredResult,
            scale_rule: pse_quantity::QuantityScaleRule::Difference,
            shape_rule: pse_quantity::QuantityShapeRule::SameIndices,
            basis_source: None,
            reference_source: None,
            scale_source: None,
            shape_source: None,
            subject_rule: pse_quantity::SubjectRule::DeclaredResult,
            subject_source: None,
            result_subject_kind: delta.key.subject_kind,
            result_basis: delta.key.basis,
            result_reference_state: delta.key.reference_state,
            input_conversions: vec![],
            precondition_invariants: vec![rate_contract, interval_contract],
        });
        physical.quantities = Arc::new(builder.build().unwrap());
        let mut preconditions = physical.preconditions.declarations().to_vec();
        for (id, position, required) in [
            (rate_contract, 0, rate_type),
            (interval_contract, 1, time.id),
        ] {
            preconditions.push(pse_quantity::PhysicalPrecondition {
                id,
                operand_positions: vec![position],
                requirement: pse_quantity::PhysicalRequirement::OperandQuantityContract {
                    required,
                    match_shape: false,
                },
            });
        }
        preconditions.sort_by_key(|declaration| declaration.id);
        physical.preconditions =
            Arc::new(pse_quantity::PhysicalPreconditions::new(preconditions).unwrap());
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let source = "package p { def Root { domain t:Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param base:Temperature=300{K}; param step:DeltaTemperature=1{K}; var x[i in t]:Temperature; let rate[i in t]:TemperatureRate=0.5*step/1{s}; conserve stock[i in t]:Temperature on t inventory x[i] flux rate[i] tolerance 1e-6{K} transfers(hit[0{s}]=x[i]-base); eq initial:x[0{s}]==base; let hit[i in t]:DeltaTemperature=x[i]-base-0.5*step; let jump[i in t]:Temperature=x[i]+(x[i]-base); } test evented fixture { dof 0; route integrated; procedure integrate; integrate samples(0{s},0.5{s},1.5{s},2{s}) relative(1e-9) normalized_absolute(1e-10) step(1e-4{s}); mode before; event root.hit[0{s}] direction(either) tolerance(1e-8{K}) reset(root.x[0{s}]=root.jump[0{s}]) next(after); mode after; } { child root:Root=Root(); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "evented")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            let prepared = package
                .declared_simulation(
                    root,
                    super::super::super::tests::compiler_profile(),
                    Some(native::Profile {
                        method,
                        end: 2.,
                        samples: vec![0., 0.5, 1.5, 2.],
                        parameter_scales: vec![1.; 2],
                        rtol: 1e-9,
                        atol: vec![1e-10],
                        out_rtol: Some(1e-9),
                        out_atol: vec![1e-11],
                        ..Default::default()
                    }),
                    Limits::default(),
                    &cancel,
                )
                .await
                .unwrap();
            let trajectory = prepared.run(&cancel).await.unwrap();
            assert_eq!(
                trajectory.report().termination,
                native::Termination::Completed,
                "{:?}",
                trajectory.report().error
            );
            assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
            let last = trajectory.report().conservation.last().unwrap();
            assert!((last.inventories[0] - 301.5).abs() < 1e-7);
            assert!((last.transfers[0] - 0.5).abs() < 1e-7);
            assert!(last.defects[0].abs() < 1e-6);
        }
    }
    #[tokio::test]
    async fn authored_conservation_refuses_algebraic_resets_before_native_consistency() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 1; var x[i in t]: Time; var y[i in t]: Time; conserve stock[i in t]: Time on t inventory x[i] flux p tolerance 1e-6{s}; eq initial: x[0{s}] == 1{s}; eq definition[i in t]: y[i] == 2*x[i]; annotation start y(2{s}); let hit[i in t]: Time = x[i]-1.5{s}; let jump[i in t]: Time = 2*y[i]; } test evented fixture { dof 0; route integrated; procedure integrate; integrate samples(0{s},1{s}) relative(1e-9) normalized_absolute(1e-11) step(1e-4{s}); mode before; event root.hit[0{s}] direction(either) tolerance(1e-8{s}) reset(root.y[0{s}] = root.jump[0{s}]) next(after); mode after; } { child root: Root = Root(); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "evented")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let error = package
            .declared_simulation(
                root,
                super::super::super::tests::compiler_profile(),
                Some(native::Profile {
                    samples: vec![0.0, 1.0],
                    parameter_scales: vec![1.0],
                    atol: vec![1e-11; 2],
                    out_rtol: Some(1e-9),
                    out_atol: vec![1e-11],
                    ..Default::default()
                }),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::Unsupported,
            "{error}"
        );
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::ModelingDynamicAlgebraicResetUnsupported
        );
        assert_eq!(diagnostic.sources.len(), 2);
    }
    #[tokio::test]
    async fn kernel_conformance_integrates_authored_samples_and_retains_each_check() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let text = "package p { def Root {domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==2; eq initial:x[0{s}]==1{s}; annotation check x(x[i]<=4{s}); annotation valid x(0{s},4{s}); let area:Time=integral(i in t | 2); annotation check area(area>1.9{s});} test dynamic fixture {dof 0; route integrated; procedure integrate; integrate samples(0{s},0.5{s},1{s}) relative(1e-6) normalized_absolute(1e-8) step(1e-4{s}) quadrature_relative(1e-6) quadrature_absolute(root.area=1e-7{s});} {child root:Root=Root();} }";
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let rendered = pse_authoring::language::render(&rows).unwrap();
        let reparsed = pse_authoring::language::parse(
            &rendered,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            reparsed.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let policy = ModelingConformancePolicy {
            compiler: super::super::super::tests::compiler_profile(),
            solver: super::super::super::tests::profile(),
            numerical: Default::default(),
            limits: Limits::default(),
            maximum_fixtures: 4,
            maximum_checks: 64,
            fixtures: Default::default(),
            diagnostics: None,
            derivatives: pse_backend_native::derivative_diagnostics::Policy {
                perturbation: 1e-6,
                relative_tolerance: 1e-4,
                maximum_cells: 100,
            },
        };
        let report = package
            .conform(policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.trajectories.len(), 1, "{:?}", report.checks);
        let trajectory = report.trajectories.values().next().unwrap();
        assert!(trajectory.accepted(), "{:?}", trajectory.diagnostic());
        assert_eq!(trajectory.report().samples.len(), 3);
        let checks = report
            .checks
            .iter()
            .filter(|c| c.kind == pse_model::generated::enums::ModelingConformanceKind::Check)
            .collect::<Vec<_>>();
        assert_eq!(checks.len(), 4);
        let state_target = checks
            .iter()
            .find(|c| c.sample_index == 0)
            .unwrap()
            .target_id;
        let checks = checks
            .into_iter()
            .filter(|c| c.target_id == state_target)
            .collect::<Vec<_>>();
        assert_eq!(checks.len(), 3);
        assert_eq!(
            checks.iter().map(|c| c.sample_index).collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!(
            checks.iter().map(|c| c.time).collect::<Vec<_>>(),
            [Some(0.), Some(0.5), Some(1.)]
        );
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(
            report
                .checks
                .iter()
                .filter(
                    |c| c.kind == pse_model::generated::enums::ModelingConformanceKind::Derivatives
                )
                .count(),
            3
        );
        report.table().unwrap();
    }
    /// ADR-0119 Outcome 2 (I6): an authored fixture holds an input piecewise constant.
    /// Each interval is its own integration parameter carrying its authored value, so the
    /// trajectory follows the schedule and the forward sensitivities of both intervals stay
    /// live across the change. The fixture is the schedule's authority.
    #[tokio::test]
    async fn kernel_fixture_schedules_inputs() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let text = "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param u: Scalar = 5; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == u; eq initial: x[0{s}] == 1{s}; } test scheduled fixture {dof 0; route integrated; procedure integrate; integrate samples(0{s}, 0.5{s}, 1{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{s}); schedule root.u at(0.5{s}) values(2, -1);} {child root: Root = Root();} test on_state fixture {dof 0; route integrated; procedure integrate; integrate samples(0{s}, 1{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{s}); schedule root.x[0{s}] at(0.5{s}) values(1{s}, 2{s});} {child root: Root = Root();} }";
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let rendered = pse_authoring::language::render(&rows).unwrap();
        assert!(
            rendered.contains("schedule root.u at(0.5{s}) values(2, -1);"),
            "{rendered}"
        );
        let reparsed = pse_authoring::language::parse(
            &rendered,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            reparsed.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
        let (scheduled, on_state) = (root("scheduled"), root("on_state"));
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let prepared = package
            .declared_simulation(scheduled, compiler, None, Limits::default(), &cancel)
            .await
            .unwrap();
        // One contract parameter, two intervals, each with its authored value.
        assert_eq!(prepared.contract().parameters.len(), 1);
        assert_eq!(
            prepared.profile().schedule,
            vec![native::ScheduledInput {
                parameter: 0,
                times: vec![0.5]
            }]
        );
        assert_eq!(prepared.parameters(), [2., -1.]);
        let mut profile = prepared.profile().clone();
        profile.sensitivity = native::DynamicSensitivity::Forward;
        let sensitive = package
            .declared_simulation(
                scheduled,
                compiler,
                Some(profile.clone()),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_ne!(sensitive.identity(), prepared.identity());
        let result = sensitive.run(&cancel).await.unwrap();
        assert!(result.accepted(), "{:?}", result.diagnostic());
        let samples = &result.report().samples;
        // x = 1 + 2·t before the change and 2 − (t − 0.5) after; the unscheduled model
        // value 5 would give 6 at the end.
        for (sample, expected) in samples.iter().zip([1., 2., 1.5]) {
            assert!((sample.outputs[0] - expected).abs() < 1e-7, "{sample:?}");
        }
        // dx/du₀ and dx/du₁, output-major over the two interval parameters.
        let sensitivities = |i: usize| &samples[i].output_sensitivities[..2];
        for (i, expected) in [(1, [0.5, 0.]), (2, [0.5, 0.5])] {
            for (actual, expected) in sensitivities(i).iter().zip(expected) {
                assert!((actual - expected).abs() < 1e-7, "{:?}", samples[i]);
            }
        }
        // A profile repeating the authored schedule is accepted; one scheduling the same
        // input differently is refused.
        profile.schedule[0].times = vec![0.25];
        let error = package
            .declared_simulation(
                scheduled,
                compiler,
                Some(profile),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("authored schedule"), "{error}");
        // A state is not a scheduled input.
        let error = package
            .declared_simulation(on_state, compiler, None, Limits::default(), &cancel)
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("integration parameter"),
            "{error}"
        );
    }
    /// ADR-0119 Outcome 3: the case's fixture declares the modes (the facts selecting
    /// each) and the events with their resets and successors; the simulation reads them,
    /// with no runtime mode input.
    #[tokio::test]
    async fn kernel_integrated_events_bind_source_resets_and_same_layout_modes() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let integrate = "dof 0; route integrated; procedure integrate; integrate samples(0{s}, 0.25{s}, 0.75{s}, 2{s}) relative(1e-8) normalized_absolute(1e-8) step(1e-4{s});";
        let source = format!(
            "package p {{ def Root {{ domain t: Time from 0{{s}} to 2{{s}}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[0{{s}}] == 1{{s}}; let hit[i in t]: Time = x[i]-2{{s}}; let jump[i in t]: Time = 2*x[i]; let wrong: Scalar = 0; stage coast {{ override eq rate[i in t]: d(x[i])/di == 0; }} annotation check x(x[i] <= 4.01{{s}}); }} \
             test evented fixture {{ {integrate} mode rise; event root.hit[0{{s}}] direction(either) tolerance(1e-8{{s}}) reset(root.x[0{{s}}] = root.jump[0{{s}}]) next(coast); mode coast facts(stage.coast = true); }} {{ child root: Root = Root(); }} \
             test mistyped fixture {{ {integrate} mode rise; event root.hit[0{{s}}] direction(either) tolerance(1e-8{{s}}) reset(root.x[0{{s}}] = root.wrong) next(coast); mode coast facts(stage.coast = true); }} {{ child root: Root = Root(); }} \
             test stopped fixture {{ {integrate} mode stop; event root.hit[0{{s}}] direction(either) tolerance(1e-8{{s}}) terminal; }} {{ child root: Root = Root(); }} }}"
        );
        let rows = pse_authoring::language::parse(
            &source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let rendered = pse_authoring::language::render(&rows).unwrap();
        assert!(
            rendered.contains("mode coast facts(stage.coast = true);")
                && rendered.contains("event root.hit[0{s}] direction(either) tolerance(1e-8{s}) reset(root.x[0{s}] = root.jump[0{s}]) next(coast);")
                && rendered.contains("event root.hit[0{s}] direction(either) tolerance(1e-8{s}) terminal;"),
            "{rendered}"
        );
        let reparsed = pse_authoring::language::parse(
            &rendered,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            reparsed.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
        let (evented, mistyped, stopped) = (root("evented"), root("mistyped"), root("stopped"));
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let cancel = crate::CancelSource::new();
        let profile = native::Profile {
            end: 2.,
            samples: vec![0., 0.25, 0.75, 2.],
            parameter_scales: vec![1.],
            sensitivity: native::DynamicSensitivity::Forward,
            ..Default::default()
        };
        let compiler = super::super::super::tests::compiler_profile();
        let simulate = |root, profile| {
            package.declared_simulation(root, compiler, Some(profile), Limits::default(), &cancel)
        };
        let prepared = simulate(evented, profile.clone()).await.unwrap();
        assert_eq!(prepared.mode_names().collect::<Vec<_>>(), ["rise", "coast"]);
        let result = prepared.run(&cancel).await.unwrap();
        assert!(
            result.accepted(),
            "{:?} {:?}",
            result.report().error,
            result.validation_error()
        );
        assert_eq!(result.report().events.len(), 1);
        assert!((result.report().events[0].time - 0.5).abs() < 1e-5);
        assert_eq!(
            result
                .report()
                .samples
                .iter()
                .map(|s| s.mode)
                .collect::<Vec<_>>(),
            vec![0, 0, 1, 1]
        );
        assert!((result.report().samples[3].outputs[0] - 4.).abs() < 1e-6);
        assert!(
            result.report().samples[3].output_sensitivities[0].abs() < 1e-5,
            "moving root sensitivity must cancel at the threshold: {:?}",
            result.report().samples[3]
        );
        assert_eq!(result.checks().len(), 4);
        let tables = result.tables().unwrap();
        assert!(
            tables.contains_key(&pse_relations::generated::runtime::simulation_events::RELATION_ID)
        );
        let error = simulate(mistyped, profile.clone()).await.unwrap_err();
        assert!(error.to_string().contains("physical type"), "{error}");
        let mut terminal_profile = profile;
        terminal_profile.sensitivity = native::DynamicSensitivity::None;
        let stopped = simulate(stopped, terminal_profile)
            .await
            .unwrap()
            .run(&cancel)
            .await
            .unwrap();
        assert_eq!(stopped.report().termination, native::Termination::Event);
        assert!(!stopped.accepted());
        assert!(!stopped.checks_complete());
        assert_eq!(stopped.report().samples.len(), 2);
    }
    /// ADR-0119 Outcome 3 within ADR-0110's routes: an authored directional event routes an
    /// automatic method to IDAS, which honours the direction (`IDASetRootDirection`); an
    /// undirected one stays on Diffsol; an explicit Diffsol request is refused. The guard
    /// x − 1.25 s of x = 1 + 2t − 2t² rises through zero at (1 − 1/√2)/2 s and falls at
    /// (1 + 1/√2)/2 s.
    #[cfg(feature = "solver-idas")]
    #[tokio::test]
    async fn authored_directional_event_routes_to_idas() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let fixture = |direction: &str| {
            format!(
                "test {direction} fixture {{ dof 0; route integrated; procedure integrate; integrate samples(0{{s}}, 0.5{{s}}, 1{{s}}) relative(1e-9) normalized_absolute(1e-10) step(1e-4{{s}}); mode arc; event root.g[0{{s}}] direction({direction}) tolerance(1e-8{{s}}) terminal; }} {{ child root: Arc = Arc(); }}"
            )
        };
        let source = format!(
            "package p {{ def Arc {{ domain t: Time from 0{{s}} to 1{{s}}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == 2 - 4*i/1{{s}}; eq initial: x[0{{s}}] == 1{{s}}; let g[i in t]: Time = x[i] - 1.25{{s}}; }} {} {} {} }}",
            fixture("either"),
            fixture("rising"),
            fixture("falling")
        );
        let rows = pse_authoring::language::parse(
            &source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
        let tests = ["either", "rising", "falling"].map(root);
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let (rise, fall) = (0.5 - 0.5f64.sqrt() / 2., 0.5 + 0.5f64.sqrt() / 2.);
        for (root, method, time) in [
            (tests[0], native::Method::Diffsol, rise),
            (tests[1], native::Method::Idas, rise),
            (tests[2], native::Method::Idas, fall),
        ] {
            let prepared = package
                .declared_simulation(root, compiler, None, Limits::default(), &cancel)
                .await
                .unwrap();
            assert_eq!(prepared.profile().resolved_method().unwrap(), method);
            let result = prepared.run(&cancel).await.unwrap();
            assert_eq!(
                result.report().termination,
                native::Termination::Event,
                "{:?}",
                result.report().error
            );
            assert_eq!(result.report().events.len(), 1);
            assert!(
                (result.report().events[0].time - time).abs() < 1e-6,
                "{method:?}: {:?}",
                result.report().events
            );
            assert!((result.report().completed_time - time).abs() < 1e-6);
        }
        // Diffsol detects every sign change, so it refuses a directional event.
        let mut diffsol = package
            .declared_simulation(tests[2], compiler, None, Limits::default(), &cancel)
            .await
            .unwrap()
            .profile()
            .clone();
        diffsol.method = native::Method::Diffsol;
        let error = package
            .declared_simulation(
                tests[2],
                compiler,
                Some(diffsol),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("needs IDAS"), "{error}");
    }
    /// ADR-0119 Outcome 4 (review F03): the IDAS sign constraints derive from the authored
    /// bounds, never from a runtime setting. A constant-zero lower endpoint keeps a state
    /// non-negative and a zero upper one non-positive; a bound without a zero endpoint, or
    /// with a fixture override of the zero endpoint, leaves it free. The constraint keeps a
    /// stiff decay non-negative where the unconstrained loose-tolerance run dips below
    /// zero (inside its relaxed bound, which the guard still checks).
    #[cfg(feature = "solver-idas")]
    #[tokio::test]
    async fn idas_sign_constraints_from_authored_bounds() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let integrate = "dof 0; route integrated; procedure integrate; integrate samples(0{s}, 10{s}) relative(1e-2) normalized_absolute(1e-2) step(1e-4{s});";
        let decay = |state: &str, start: &str, rate: &str, bounds: &str| {
            format!(
                "var {state}[i in t]: Time; eq rate_{state}[i in t]: d({state}[i])/di == -{rate}*{state}[i]/1{{s}}; eq initial_{state}: {state}[0{{s}}] == {start}{{s}}; annotation bounds {state}({bounds});"
            )
        };
        let axis = "domain t: Time from 0{s} to 10{s}; discretize grid on t using integrated(elements=1,order=1);";
        let source = format!(
            "package p {{ def Decay {{ {axis} {} }} def Signs {{ {axis} {} {} {} }} \
             test decay fixture {{ {integrate} }} {{ child root: Decay = Decay(); }} \
             test relaxed fixture {{ {integrate} lower root.x[0{{s}}] = -1{{s}}; }} {{ child root: Decay = Decay(); }} \
             test signs fixture {{ {integrate} }} {{ child root: Signs = Signs(); }} }}",
            decay("x", "1", "1000", "0{s}, 2{s}"),
            decay("x", "1", "1", "0{s}, 2{s}"),
            decay("y", "-1", "1", "-2{s}, 0{s}"),
            decay("z", "0.5", "1", "-1{s}, 2{s}"),
        );
        let rows = pse_authoring::language::parse(
            &source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
        let [decay, relaxed, signs] = ["decay", "relaxed", "signs"].map(root);
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let prepare = |root, profile| {
            package.declared_simulation(root, compiler, profile, Limits::default(), &cancel)
        };
        // One sign per state, in state order: x ≥ 0, y ≤ 0, z free.
        let prepared = prepare(signs, None).await.unwrap();
        let symbols = &prepared.model().compiled().model.symbols;
        let expected = prepared
            .contract()
            .states
            .iter()
            .map(|id| match symbols[id].lineage.path.rsplit('.').next() {
                Some("x") => native::StateSign::NonNegative,
                Some("y") => native::StateSign::NonPositive,
                _ => native::StateSign::Free,
            })
            .collect::<Vec<_>>();
        for sign in [
            native::StateSign::NonNegative,
            native::StateSign::NonPositive,
            native::StateSign::Free,
        ] {
            assert!(expected.contains(&sign), "{expected:?}");
        }
        assert_eq!(prepared.contract().signs, expected);
        // The stiff decay on IDAS with recoverable trials and loose tolerances.
        let mut profile = prepare(decay, None).await.unwrap().profile().clone();
        profile.method = native::Method::Idas;
        profile.trial_failures = native::TrialPolicy::Recoverable;
        profile.samples = (0..=400).map(|i| f64::from(i) * 0.025).collect();
        let lowest = |result: &ModelingTrajectory| {
            result
                .report()
                .samples
                .iter()
                .map(|s| s.outputs[0])
                .fold(f64::INFINITY, f64::min)
        };
        let constrained = prepare(decay, Some(profile.clone())).await.unwrap();
        assert_eq!(
            constrained.contract().signs,
            [native::StateSign::NonNegative]
        );
        let kept = constrained.run(&cancel).await.unwrap();
        assert!(kept.accepted(), "{:?}", kept.diagnostic());
        assert!(lowest(&kept) >= 0., "constrained minimum {}", lowest(&kept));
        let free = prepare(relaxed, Some(profile)).await.unwrap();
        assert!(free.contract().signs.is_empty());
        assert_ne!(free.identity(), constrained.identity());
        let dipped = free.run(&cancel).await.unwrap();
        assert!(dipped.accepted(), "{:?}", dipped.diagnostic());
        assert!(
            lowest(&dipped) < 0.,
            "the unconstrained control run stayed non-negative ({})",
            lowest(&dipped)
        );
    }
    /// ADR-0119 Outcome 3: authored events select the integrated route. Specializing their
    /// case on the simultaneous route is refused, and a fixture that declares both is
    /// refused when the package is admitted.
    #[tokio::test]
    async fn simultaneous_route_refuses_authored_events() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let def = "def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == 2; eq initial: x[0{s}] == 1{s}; let hit[i in t]: Time = x[i]-2{s}; }";
        let events =
            "mode run; event root.hit[0{s}] direction(rising) tolerance(1e-8{s}) terminal;";
        let parse = |text: &str| {
            pse_authoring::language::parse(
                text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap()
        };
        let rows = parse(&format!(
            "package p {{ {def} test evented fixture {{ dof 0; route integrated; procedure integrate; integrate samples(0{{s}}, 1{{s}}) relative(1e-8) normalized_absolute(1e-8) step(1e-4{{s}}); {events} }} {{ child root: Root = Root(); }} }}"
        ));
        let evented = rows
            .iter()
            .find(|r| r.name == "evented")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, physical.clone())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let error = package
            .prepare(
                evented,
                pse_modeling::specialize::root_instance(evented),
                Bindings::default().with_analysis(pse_modeling::analysis::Route::Simultaneous),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("the simultaneous route refuses them"),
            "{error}"
        );
        // The integrated route admits the same case.
        package
            .prepare(
                evented,
                pse_modeling::specialize::root_instance(evented),
                Bindings::default().with_analysis(pse_modeling::analysis::Route::Integrated),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let rows = parse(&format!(
            "package p {{ {def} test declared fixture {{ dof 0; route simultaneous; procedure solve; {events} }} {{ child root: Root = Root(); }} }}"
        ));
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let declared = package
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "declared")
            .unwrap()
            .declaration_id;
        let error = package
            .declared_execution(
                declared,
                super::super::super::tests::compiler_profile(),
                super::super::super::tests::profile(),
                Default::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("integrated route"), "{error}");
    }
    #[tokio::test]
    async fn kernel_integrated_definite_integrals_are_terminal_native_quadratures() {
        terminal_quadratures(native::Method::Diffsol).await;
        #[cfg(feature = "solver-idas")]
        terminal_quadratures(native::Method::Idas).await;
    }
    async fn terminal_quadratures(method: native::Method) {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[0{s}] == 1{s}; let area: Time = integral(i in t | p); let score: Scalar = log(area/1{s}); annotation report area(\"integral\"); annotation report score(\"log-integral\"); annotation check area(area > 3{s}); annotation check x(x[i] <= 6{s}); } }";
        let cancel = crate::CancelSource::new();
        let compiler = super::super::super::tests::compiler_profile();
        for (text, noncausal) in [
            (source.to_string(), false),
            (
                source.replace("d(x[i])/di == p", "d(x[i])/di == area/1{s}"),
                true,
            ),
        ] {
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(rows, physical.clone())
                .await
                .unwrap();
            let profile = native::Profile {
                method,
                end: 2.,
                samples: vec![0., 1., 2.],
                parameter_scales: vec![1.],
                out_rtol: Some(1e-9),
                out_atol: vec![1e-10],
                ..Default::default()
            };
            let preparation = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    DerivativeOrder::First,
                    &cancel,
                )
                .await;
            if noncausal {
                let error = preparation.unwrap_err().to_string();
                assert!(error.contains("noncausal"), "{error}");
                continue;
            }
            let prepared = preparation.unwrap();
            assert_eq!(prepared.contract.quadratures.len(), 1);
            assert!(prepared.contract.balances.is_empty());
            assert_eq!(
                prepared.contract.outputs.len(),
                1,
                "terminal outputs must not be prefix samples"
            );
            let result = prepared.run(&cancel).await.unwrap();
            assert!(
                result.accepted(),
                "{:?} {:?}",
                result.report().error,
                result.validation_error()
            );
            assert_eq!(result.checks().len(), 4);
            assert_eq!(
                result
                    .checks()
                    .iter()
                    .filter(|c| c.sample_index == 2)
                    .count(),
                2
            );
            assert!(
                (result
                    .reports()
                    .iter()
                    .find(|r| r.label == "integral")
                    .unwrap()
                    .value
                    - 4.)
                    .abs()
                    < 1e-6
            );
            assert!(
                (result
                    .reports()
                    .iter()
                    .find(|r| r.label == "log-integral")
                    .unwrap()
                    .value
                    - 4_f64.ln())
                .abs()
                    < 1e-6
            );
            assert!((result.report().samples[1].integrals[0] - 2.).abs() < 1e-6);
            let tables = result.tables().unwrap();
            use pse_relations::{columnar::RelationRow, generated::runtime::modeling_reports};
            assert_eq!(
                modeling_reports::Row::rows(&tables[&modeling_reports::RELATION_ID])
                    .unwrap()
                    .len(),
                2
            );
            let mut short = profile.clone();
            short.end = 1.;
            short.samples = vec![0., 1.];
            assert!(
                package
                    .prepare_simulation(
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Bindings::default(),
                        Limits::default(),
                        ModelingCaseBindings::default(),
                        compiler,
                        short,
                        DerivativeOrder::First,
                        &cancel
                    )
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("complete domain")
            );
            let mut derivatives = profile;
            derivatives.sensitivity = native::DynamicSensitivity::Forward;
            let sensitive = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    derivatives,
                    DerivativeOrder::First,
                    &cancel,
                )
                .await
                .unwrap();
            let sensitive = sensitive.run(&cancel).await.unwrap();
            assert!(sensitive.accepted(), "{:?}", sensitive.diagnostic());
            for sample in &sensitive.report().samples {
                assert_eq!(sample.output_sensitivities.len(), 1);
                assert!((sample.output_sensitivities[0] - sample.time).abs() < 1e-6);
                assert!((sample.integrals[0] - 2. * sample.time).abs() < 1e-6);
            }
        }
    }
    #[tokio::test]
    async fn temporal_composition_refuses_missing_or_competing_endpoint_initialization() {
        let runtime = super::super::super::tests::runtime();
        let source = "package p {def Cell(times:Set<Time>={}) {var x:Time;eq rate[i in times]:d(x)/di==1;} def Root {domain t:Time from 0{s} to 2{s};discretize grid on t using integrated(elements=1,order=1);child cell:Cell=Cell();evolve time_state on cell using t bind times;eq initial:cell[0{s}].x==0{s};}}";
        for text in [
            source.replace("eq initial:cell[0{s}].x==0{s};", ""),
            source.replace("eq initial:", "eq competing:cell[0{s}].x==1{s};eq initial:"),
        ] {
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|row| row.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime.modeling_package(rows, physical()).await.unwrap();
            let error = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    super::super::super::tests::compiler_profile(),
                    native::Profile {
                        end: 2.,
                        samples: vec![0., 2.],
                        ..Default::default()
                    },
                    DerivativeOrder::First,
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap_err();
            assert!(error.to_string().contains("initial"), "{error}");
        }
    }

    #[tokio::test]
    async fn kernel_integrated_sample_checks_share_model_semantics_and_owned_rows() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == 2; eq initial: x[0{s}] == 1{s}; annotation check x(x[i] <= 4{s}); } }";
        for (text, accepted) in [
            (source.to_string(), false),
            (source.replace("x[i] <= 4{s}", "x[i] <= 6{s}"), true),
            (source.replace("x[i] <= 4{s}", "(i!=0{s} or x[i]==1{s}) and (i!=1{s} or abs(x[i]-3{s})<0.00001{s}) and (i!=2{s} or abs(x[i]-5{s})<0.00001{s})"),true),
        ] {
            let declarations = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = declarations
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(declarations, physical.clone()).await
                .unwrap();
            let cancel = crate::CancelSource::new();
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    super::super::super::tests::compiler_profile(),
                    native::Profile {
                        end: 2.,
                        samples: vec![0., 1., 2.],
                        ..Default::default()
                    },
                    DerivativeOrder::First,
                    &cancel,
                )
                .await
                .unwrap();
            let result = prepared.run(&cancel).await.unwrap();
            assert_eq!(result.report().termination, native::Termination::Completed);
            assert!(result.checks_complete(), "{:?}", result.validation_error());
            assert_eq!(result.accepted(), accepted);
            assert_eq!(result.checks().len(), 3);
            let expected_checks = result.checks().to_vec();
            let tables = result.tables().unwrap();
            use pse_relations::{
                columnar::RelationRow,
                generated::runtime::{computation_runs, modeling_checks},
            };
            let checks = tables[&modeling_checks::RELATION_ID].clone();
            let header =
                computation_runs::Row::rows(&tables[&computation_runs::RELATION_ID]).unwrap();
            assert_eq!(header[0].feasible, Some(accepted));
            assert_eq!(
                header[0].qualification,
                if accepted {
                    pse_model::generated::enums::NativeQualification::Feasible
                } else {
                    pse_model::generated::enums::NativeQualification::Unqualified
                }
            );
            drop(result);
            drop(prepared);
            drop(package);
            drop(tables);
            let rows = modeling_checks::Row::rows(&checks).unwrap();
            assert_eq!(rows, expected_checks);
            assert_eq!(
                rows.iter().map(|row| (row.sample_index, row.time)).collect::<Vec<_>>(),
                [(0, Some(0.)), (1, Some(1.)), (2, Some(2.))]
            );
            assert_eq!(
                rows.iter().map(|row| row.satisfied).collect::<Vec<_>>(),
                [true, true, accepted]
            );
        }
    }
    #[tokio::test]
    async fn kernel_integrated_coupled_rate_matrix_uses_original_residual_and_parameter_jets() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; var y[i in t]: Time; eq first[i in t]: p*d(x[i])/di+d(y[i])/di == p; eq second[i in t]: d(x[i])/di+2*d(y[i])/di == 0; eq ix: x[0{s}] == 1{s}; eq iy: y[0{s}] == 3{s}; } }";
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        for (text, nonlinear, singular) in [
            (source.to_string(), false, false),
            (
                source.replace("p*d(x[i])/di", "d(x[i])/di*d(x[i])/di"),
                true,
                false,
            ),
            (
                source.replace("param p: Scalar = 2", "param p: Scalar = 0.5"),
                false,
                true,
            ),
        ] {
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(rows, physical.clone())
                .await
                .unwrap();
            let result = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    native::Profile {
                        end: 2.,
                        samples: vec![0., 1., 2.],
                        atol: vec![1e-8; 2],
                        parameter_scales: vec![1.],
                        sensitivity: native::DynamicSensitivity::Forward,
                        ..Default::default()
                    },
                    DerivativeOrder::First,
                    &cancel,
                )
                .await;
            if nonlinear {
                let error = result.unwrap_err().to_string();
                assert!(error.contains("affine residual"), "{error}");
                continue;
            }
            let prepared = result.unwrap();
            let product = prepared.model().compiled();
            let inner = product.admitted.implicit_systems().next().unwrap();
            assert_eq!(
                inner.algorithm,
                pse_compiler::workspace::ImplicitAlgorithm::AffineRates
            );
            assert_eq!(inner.residuals[0].rows.len(), 2);
            assert!(
                product
                    .admitted
                    .outputs
                    .iter()
                    .filter_map(|o| match o {
                        ModelingOutput::DynamicRate { equations, .. } => Some(equations),
                        _ => None,
                    })
                    .all(|rows| rows.len() == 2)
            );
            let x = product
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(".x"))
                .unwrap()
                .id;
            let y = product
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(".y"))
                .unwrap()
                .id;
            let xi = prepared
                .contract
                .states
                .iter()
                .position(|s| *s == x)
                .unwrap();
            let yi = prepared
                .contract
                .states
                .iter()
                .position(|s| *s == y)
                .unwrap();
            let result = prepared.run(&cancel).await.unwrap();
            if singular {
                assert_ne!(result.report().termination, native::Termination::Completed);
                assert!(result.report().error.is_some());
                continue;
            }
            assert_eq!(
                result.report().termination,
                native::Termination::Completed,
                "{:?}",
                result.report().error
            );
            for sample in &result.report().samples {
                assert!((sample.outputs[xi] - (1. + 4. * sample.time / 3.)).abs() < 1e-6);
                assert!((sample.outputs[yi] - (3. - 2. * sample.time / 3.)).abs() < 1e-6);
                assert!((sample.output_sensitivities[xi] + 2. * sample.time / 9.).abs() < 1e-6);
                assert!((sample.output_sensitivities[yi] - sample.time / 9.).abs() < 1e-6);
            }
        }
    }
    #[tokio::test]
    async fn dynamics_refuses_free_integer() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize mesh on t using integrated(elements=1,order=1); var x[i in t]: Time; var units: Count in integer; eq ode[i in t]: d(x[i])/di == 2; eq initial: x[0{s}] == 1{s}; annotation start x(0{s}); annotation start units(1{1}); annotation bounds units(0{1}, 3{1}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let profile = native::Profile {
            end: 1.,
            samples: vec![0., 1.],
            parameter_scales: vec![1.],
            ..Default::default()
        };
        let cancel = crate::CancelSource::new();
        let simulate = |case| {
            package.prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case,
                super::super::super::tests::compiler_profile(),
                profile.clone(),
                DerivativeOrder::First,
                &cancel,
            )
        };
        let error = simulate(ModelingCaseBindings::default())
            .await
            .err()
            .unwrap();
        assert_eq!(
            super::super::super::tests::free_discrete_refusal(&error),
            ("units".into(), "integrated_dynamics".into())
        );
        // Fixed for the segment, the discrete input is an ordinary parameter of integration.
        let fixed = ModelingCaseBindings {
            members: BTreeMap::new(),
            values: BTreeMap::from([("units".into(), 1.)]),
            variables: BTreeMap::from([(
                "units".into(),
                pse_compiler::workspace::ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            )]),
        };
        let prepared = simulate(fixed).await.unwrap();
        assert_eq!(prepared.contract.states.len(), 1);
    }
    #[tokio::test]
    async fn kernel_integrated_time_uses_generated_rates_native_solver_and_initial_sensitivities() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; param offset: Time = 1{s}; var x[i in t]: Time; eq ode[i in t]: d(x[i])/di == p; eq initial: x[0{s}] == offset; annotation start x(0{s}); annotation nominal x(10{s}); annotation check x(abs(x[i] - offset - p*i) < 1e-5{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let profile = native::Profile {
            end: 2.,
            samples: vec![0., 0.5, 1., 2.],
            parameter_scales: vec![1., 1.],
            sensitivity: native::DynamicSensitivity::Forward,
            ..Default::default()
        };
        let prepared = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile,
                DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(prepared.contract.states.len(), 1);
        assert_eq!(prepared.coordinates.state[0].scale, 10.);
        let named = |name: &str| {
            prepared
                .model()
                .compiled()
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let p = named("p");
        let offset = named("offset");
        let pi = prepared
            .contract
            .parameters
            .iter()
            .position(|id| *id == p)
            .unwrap();
        let oi = prepared
            .contract
            .parameters
            .iter()
            .position(|id| *id == offset)
            .unwrap();
        let result = prepared.run(&cancel).await.unwrap();
        assert_eq!(
            result.report().termination,
            native::Termination::Completed,
            "{:?}",
            result.report().error
        );
        for sample in &result.report().samples {
            assert!((sample.outputs[0] - (1. + 2. * sample.time)).abs() < 1e-6);
            assert!((sample.state[0] - (1. + 2. * sample.time) / 10.).abs() < 1e-6);
            assert!((sample.output_sensitivities[pi] - sample.time).abs() < 1e-6);
            assert!((sample.output_sensitivities[oi] - 1.).abs() < 1e-6);
        }
        // @start is only a guess: the original endpoint equation determines x(0).
        assert!((result.report().requested_initial[0] - 0.1).abs() < 1e-12);
        let tables = result.tables().unwrap();
        use pse_relations::{
            columnar::RelationRow,
            generated::runtime::{computation_runs, response_sensitivities, simulation_samples},
        };
        let samples = tables[&simulation_samples::RELATION_ID].clone();
        let header = computation_runs::Row::rows(&tables[&computation_runs::RELATION_ID]).unwrap();
        assert_eq!(
            header[0].trajectory_termination,
            Some(native::Termination::Completed)
        );
        assert_eq!(header[0].completed_samples, Some(4));
        assert_eq!(
            response_sensitivities::Row::rows(&tables[&response_sensitivities::RELATION_ID])
                .unwrap()
                .len(),
            8
        );
        drop(result);
        let rows = simulation_samples::Row::rows(&samples).unwrap();
        assert_eq!(rows.len(), 4);
        assert!((rows.last().unwrap().value - 5.).abs() < 1e-6);
        let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut worker = prepared
            .worker(pse_kernels::ExecutionScope::new(flag.clone(), None))
            .unwrap();
        assert!(
            worker
                .evaluate(0, Function::Rhs, 0., &[0.1], &prepared.parameters, true)
                .is_ok()
        );
        // Fitting trials reuse compilation, but their checks must consume the
        // actual trial parameters, including the initial-condition parameter.
        let mut trial_parameters = prepared.parameters.clone();
        trial_parameters[pi] = 3.;
        trial_parameters[oi] = 4.;
        let trial = native::integrate(
            &mut worker,
            &prepared.profile,
            &trial_parameters,
            flag.clone(),
        )
        .unwrap();
        assert_eq!(trial.termination, native::Termination::Completed);
        let run: RunId = pse_operations::mint_id();
        let checks = prepared.check_samples(
            run,
            &trial,
            &trial_parameters,
            &pse_kernels::ExecutionScope::new(flag.clone(), None),
        );
        assert!(checks.complete && checks.error.is_none());
        assert!(checks.rows.iter().all(|c| c.satisfied));
        let wrong = prepared.check_samples(
            run,
            &trial,
            &prepared.parameters,
            &pse_kernels::ExecutionScope::new(flag.clone(), None),
        );
        assert!(wrong.rows.iter().any(|c| !c.satisfied));
        // A final trajectory assessment consumes its enclosing deadline even
        // when the simulation's own local allowance would be longer.
        let expired = prepared.check_samples(
            run,
            &trial,
            &trial_parameters,
            &pse_kernels::ExecutionScope::new(flag.clone(), Some(std::time::Instant::now())),
        );
        assert!(!expired.complete);
        assert!(expired.error.is_some());
        assert!(expired.rows.is_empty());
        assert!(!flag.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(prepared.parameters[pi], 2.);
        assert_eq!(prepared.parameters[oi], 1.);
        flag.store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            worker.evaluate(0, Function::Rhs, 0., &[0.1], &prepared.parameters, true),
            Err(ProblemError::Provider(
                pse_kernels::ProviderError::Cancelled
            ))
        ));
    }
    #[tokio::test]
    async fn kernel_nonlinear_dae_initial_parameter_sensitivities_and_quadrature_agree() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t:Time from 0{s} to 1{s}; discretize mesh on t using integrated(elements=1,order=1); param p:Scalar=2; param offset:Time=1{s}; var x[i in t]:Time; var y[i in t]:Scalar; eq rate[i in t]:d(x[i])/di==p; eq initial:x[0{s}]==offset; eq algebraic[i in t]:y[i]*y[i]==x[i]/1{s}; annotation start x(1{s}); annotation start y(1); let area:Time=integral(i in t | p); annotation check area(abs(area-2{s})<1e-5{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let y_declaration = rows.iter().find(|r| r.name == "y").unwrap().declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let cancel = crate::CancelSource::new();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            let profile = native::Profile {
                method,
                end: 1.,
                samples: vec![0., 0.5, 1.],
                rtol: 1e-8,
                atol: vec![1e-10; 2],
                parameter_scales: vec![1.; 2],
                sensitivity: native::DynamicSensitivity::Forward,
                out_rtol: Some(1e-8),
                out_atol: vec![1e-9],
                ..Default::default()
            };
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Default::default(),
                    Default::default(),
                    super::super::super::tests::compiler_profile(),
                    profile,
                    DerivativeOrder::First,
                    &cancel,
                )
                .await
                .unwrap();
            let y = prepared
                .model()
                .compiled()
                .model
                .symbols
                .values()
                .find(|s| s.lineage.declaration == y_declaration)
                .unwrap()
                .id;
            let yi = prepared
                .contract
                .states
                .iter()
                .position(|id| *id == y)
                .unwrap();
            let parameter = |name: &str| {
                prepared
                    .model()
                    .compiled()
                    .model
                    .symbols
                    .values()
                    .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
                    .unwrap()
                    .id
            };
            let pi = prepared
                .contract
                .parameters
                .iter()
                .position(|id| *id == parameter("p"))
                .unwrap();
            let oi = prepared
                .contract
                .parameters
                .iter()
                .position(|id| *id == parameter("offset"))
                .unwrap();
            let result = prepared.run(&cancel).await.unwrap();
            assert!(result.accepted(), "{method:?}: {:?}", result.diagnostic());
            for sample in &result.report().samples {
                let y = (1. + 2. * sample.time).sqrt();
                assert!((sample.state[yi] - y).abs() < 1e-6);
                assert!(
                    (sample.state_sensitivities[yi * 2 + pi] - sample.time / (2. * y)).abs() < 1e-5
                );
                assert!((sample.state_sensitivities[yi * 2 + oi] - 1. / (2. * y)).abs() < 1e-5);
            }
        }
    }
    #[tokio::test]
    async fn kernel_integrated_dae_checks_partition_and_retains_completed_samples_on_failure() {
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; param offset: Time = 1{s}; var x[i in t]: Time; var y: Scalar; eq ode[i in t]: d(x[i])/di == y; eq initial: x[0{s}] == offset; eq algebraic: y == 2*p; annotation start x(0{s}); annotation start y(999); } }";
        let parse = |source: &str| {
            pse_authoring::language::parse(
                source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap()
        };
        let rows = parse(source);
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let profile = native::Profile {
            end: 1.,
            samples: vec![0., 0.1, 0.5, 1.],
            rtol: pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction,
            atol: vec![
                pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
                2
            ],
            parameter_scales: vec![1., 1.],
            sensitivity: native::DynamicSensitivity::Forward,
            ..Default::default()
        };
        let prepared = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile.clone(),
                DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        let xi = prepared
            .contract
            .differential
            .iter()
            .position(|d| *d)
            .unwrap();
        let yi = 1 - xi;
        let result = prepared.run(&cancel).await.unwrap();
        assert_eq!(
            result.report().termination,
            native::Termination::Completed,
            "{:?}",
            result.report().error
        );
        let state_budget = |index| {
            super::super::super::tests::engineering_target(
                prepared.numerics(),
                NumericalTarget::Variable,
                prepared.contract.states[index],
            )
            .budget
        };
        assert!((result.report().consistent_initial[yi] - 4.).abs() <= state_budget(yi));
        for sample in &result.report().samples {
            assert!((sample.outputs[xi] - (1. + 4. * sample.time)).abs() <= state_budget(xi));
            assert!((sample.outputs[yi] - 4.).abs() <= state_budget(yi));
        }
        let singular = package
            .with_declarations(parse(&source.replace("y == 2*p", "p == 2")))
            .await
            .unwrap();
        assert!(
            singular
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    DerivativeOrder::First,
                    &cancel
                )
                .await
                .is_err()
        );
        let no_initial = singular
            .with_declarations(parse(&source.replace("eq initial: x[0{s}] == offset;", "")))
            .await
            .unwrap();
        assert!(
            no_initial
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    DerivativeOrder::First,
                    &cancel
                )
                .await
                .unwrap_err()
                .to_string()
                .contains("initial condition")
        );
        let guarded = no_initial
            .with_declarations(parse(&source.replace(
                "annotation start x(0{s});",
                "annotation start x(0{s}); annotation valid x(0{s},2{s});",
            )))
            .await
            .unwrap();
        let prepared = guarded
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile,
                DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        let mut worker = prepared
            .worker(pse_kernels::ExecutionScope::new(
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
                None,
            ))
            .unwrap();
        let mut outside = vec![4.; 2];
        outside[xi] = 3.;
        let error = worker
            .evaluate(0, Function::Rhs, 0., &outside, &prepared.parameters, false)
            .unwrap_err();
        assert!(matches!(
            error,
            ProblemError::Math(pse_math::MathError::OutsideRange {
                value: 3.,
                lower: Some(0.),
                upper: Some(2.),
                ..
            })
        ));
        assert_eq!(
            pse_backend_native::callback::classify(&error),
            pse_backend_native::callback::Failure::Trial
        );
        let diagnostic = crate::workflow::diagnostics::observed(
            &error,
            pse_diagnostics::DiagnosticStage::DynamicTest,
        );
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::TrialRejected
        );
        assert_eq!(diagnostic.rule, pse_diagnostics::DiagnosticRule::MathRange);
        assert_eq!(diagnostic.sources.len(), 2);
        for (name, value) in [("value", 3_f64), ("lower", 0.), ("upper", 2.)] {
            assert!(
                matches!(diagnostic.observations[name], pse_model::diagnostic::Observation::Real(actual) if actual.to_bits() == value.to_bits())
            );
        }
        let result = prepared.run(&cancel).await.unwrap();
        assert_ne!(result.report().termination, native::Termination::Completed);
        assert!(result.report().error.is_some());
        assert!(!result.report().samples.is_empty());
        assert!(result.report().samples.iter().all(|s| s.outputs[xi] <= 2.));
        let bounded = guarded
            .with_declarations(parse(&source.replace(
                "annotation start x(0{s});",
                "annotation start x(0{s}); annotation bounds x(0{s},2{s});",
            )))
            .await
            .unwrap();
        let profile = prepared.profile.clone();
        let case = ModelingCaseBindings {
            members: BTreeMap::new(),
            values: BTreeMap::new(),
            variables: BTreeMap::from([(
                "x[0{s}]".into(),
                pse_compiler::workspace::ModelingVariableState {
                    upper: Some(Some(10.)),
                    ..Default::default()
                },
            )]),
        };
        let override_bounds = bounded
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case,
                compiler,
                profile,
                DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        let result = override_bounds.run(&cancel).await.unwrap();
        assert_eq!(
            result.report().termination,
            native::Termination::Completed,
            "{:?}",
            result.report().error
        );
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn kernel_same_definition_selects_steady_integrated_and_simultaneous() {
        use pse_modeling::analysis::Route;
        let runtime = super::super::super::tests::runtime();
        let physical = physical();
        let source = "package p { difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0.5,0.5); def Root { domain t: Time from 0{s} to 1{s}; when analysis.route == analysis.integrated { discretize native on t using integrated(elements=1,order=1); } when analysis.route != analysis.integrated { discretize grid on t using backward(elements=4,order=1); } param target: Time = 2{s}; param tau: Time = 1{s}; var x[i in t]: Time; when analysis.dynamic { eq transient[i in t]: d(x[i])/di == (target-x[i])/tau; eq initial: x[0{s}] == 0{s}; } when not analysis.dynamic { eq stationary[i in t]: 0 == (target-x[i])/tau; } annotation start x(0{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let simulation = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                native::Profile {
                    parameter_scales: vec![1.; 2],
                    ..Default::default()
                },
                DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        let trajectory = simulation.run(&cancel).await.unwrap();
        assert_eq!(
            trajectory.report().termination,
            native::Termination::Completed,
            "{:?}",
            trajectory.report().error
        );
        assert!(
            (trajectory.report().samples.last().unwrap().outputs[0] - 2. * (1. - (-1.0f64).exp()))
                .abs()
                < 1e-5
        );
        for route in [Route::Steady, Route::Simultaneous] {
            let mut solver = super::super::super::tests::profile();
            solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
                pse_backend_native::solve::Backend::Kinsol,
            );
            let prepared = package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default().with_analysis(route),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    DerivativeOrder::First,
                    compiler,
                    solver,
                    crate::math::solves::NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            let ids = prepared
                .model
                .model
                .compiled()
                .admitted
                .case()
                .variables()
                .iter()
                .map(|v| v.port.id)
                .collect::<Vec<_>>();
            let result = package
                .solve_case(prepared, compiler, &cancel)
                .await
                .unwrap();
            assert!(result.accepted, "{result:?}");
            let values = ids
                .iter()
                .map(|id| result.values.scalars[id])
                .collect::<Vec<_>>();
            if route == Route::Steady {
                assert!(values.iter().all(|v| (*v - 2.).abs() < 1e-7));
            } else {
                let end = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                assert!((end - 2. * (1. - 1.25f64.powi(-4))).abs() < 1e-7);
            }
        }
    }
}
