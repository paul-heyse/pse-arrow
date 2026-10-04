// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Single and multiple shooting (ADR-0110 Outcome 5) over the integrated experiment (I8).
//!
//! The controls are scheduled inputs held free: every interval value of a scheduled input
//! becomes an NLP variable with the control's bounds. Single shooting integrates one window
//! over the horizon; multiple shooting integrates one window per node interval, each
//! anchored at its start state ([`native::dynamics::Anchored`]), with the differential
//! states at the inner nodes as variables closed by continuity rows. Path bounds on outputs
//! are rows at every sample of the horizon. A terminal objective weighs the outputs at the
//! end and takes its gradient from the forward sensitivities that the rows need anyway;
//! an integral objective weighs the declared quadratures and takes its gradient from the
//! adjoint, with each window's quadratures observed as outputs. The NLP runs through the
//! one NLP runner (`native::execution::nlp`) with the limited-memory Hessian. The route
//! exists where an integrator is linked.
use super::integrated::{Binding, IntegratedExperiment, Window, WindowColumn};
use super::{WorkflowError, contract, math};
use crate::math::solves::SolverProfile;
use faer::sparse::{SymbolicSparseColMat, SymbolicSparseColMatRef};
use pse_backend_native::{
    self as native, NlpOracle, OracleContract, ProblemError, Variable,
    dynamics::DynamicSensitivity,
    solve::{Compatibility, Execution, HessianMode},
};
#[cfg(test)]
use pse_ids::ContentHash;
use pse_ids::{FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, Port};
use pse_math::{normalization::Normalization, numerics::TargetSpec};
use pse_model::generated::{enums::NumericalTarget, identities::RunId};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

/// Registry vocabulary of the shooting routes (ADR-0110 Outcome 5).
pub use pse_model::generated::enums::ShootingMethod;

/// A scheduled input held free: each of its interval values is a control variable within
/// these bounds (the control grid is the input's schedule).
#[derive(Clone, Debug, PartialEq)]
pub struct ShootingControl {
    /// The scheduled contract parameter.
    pub input: SemanticId,
    /// Lower bound of every interval value; unbounded when absent.
    pub lower: Option<f64>,
    /// Upper bound of every interval value; unbounded when absent.
    pub upper: Option<f64>,
}
/// Bounds on an output at every sample of the horizon.
#[derive(Clone, Debug, PartialEq)]
pub struct PathBound {
    /// The bounded output.
    pub output: SemanticId,
    /// Lower bound; unbounded when absent.
    pub lower: Option<f64>,
    /// Upper bound; unbounded when absent.
    pub upper: Option<f64>,
}
/// The minimized objective: a weighted sum of outputs at the end of the horizon (Mayer)
/// and of declared quadratures over the horizon (Lagrange). A caller maximizing negates the
/// weights.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShootingObjective {
    /// Output weights at the end of the horizon.
    pub terminal: BTreeMap<SemanticId, f64>,
    /// Declared quadrature weights over the horizon.
    pub integral: BTreeMap<SemanticId, f64>,
}
/// A shooting request over one prepared simulation.
#[derive(Clone, Debug)]
pub struct ShootingProfile {
    /// Single or multiple shooting.
    pub method: ShootingMethod,
    /// Multiple shooting's inner node times, strictly increasing inside the horizon; empty
    /// for single shooting.
    pub nodes: Vec<f64>,
    /// The scheduled inputs held free.
    pub controls: Vec<ShootingControl>,
    /// Output bounds at every sample.
    pub path: Vec<PathBound>,
    /// The minimized objective.
    pub objective: ShootingObjective,
    /// The NLP's native controls; shooting supplies first derivatives, so the Hessian is
    /// limited memory.
    pub solver: SolverProfile,
}
/// One control variable: its input, integration column, bounds and identity.
#[derive(Clone, Debug)]
struct Control {
    input: SemanticId,
    column: usize,
    variable: Variable,
}
/// One row of the NLP: a continuity row closes window `window`'s end state `state` onto
/// the next node; a path row bounds output `output` at horizon sample `sample`.
#[derive(Clone, Copy, Debug)]
enum Row {
    Continuity { window: usize, anchor: usize },
    Path { sample: usize, output: usize },
}
/// A prepared shooting problem: the NLP over the controls and the inner nodes' states.
#[derive(Debug)]
pub struct ShootingProblem {
    snapshot: native::execution::Snapshot,
    structural_assessment: Option<native::structural::Assessment>,
    pub(super) simulation: super::ModelingSimulation,
    experiment: IntegratedExperiment,
    method: ShootingMethod,
    pub(super) runtime: super::Runtime,
    pub(super) solver: SolverProfile,
    /// The windows in time order and, for every horizon sample, its window and index there.
    windows: Vec<Window>,
    samples: Vec<(usize, usize)>,
    controls: Vec<Control>,
    /// The anchored (differential) states and the start's anchors, fixed.
    differential: Vec<usize>,
    start: Vec<f64>,
    rows: Vec<Row>,
    bounds: Vec<(f64, f64)>,
    terminal: Vec<(usize, f64)>,
    integral: Vec<(usize, f64)>,
    contract: OracleContract,
    pattern: SymbolicSparseColMat<usize>,
    normalization: Normalization,
    tolerances: native::quality::Tolerances,
    accuracy: native::solve::ResolvedAccuracy,
    route: native::routing::Route,
    pub(super) profile_key: pse_ids::roles::ProfileHash,
    numerics: pse_model::numerics::ResolvedNumericalPolicy,
}
/// The outcome of a shooting solve.
#[derive(Debug)]
pub struct ShootingReport {
    /// Actual shared numerical-driver events after original trajectory assessment.
    pub strategy: Option<Arc<crate::math::strategy::Trace>>,
    /// Single or multiple shooting.
    pub method: ShootingMethod,
    /// The NLP runner's report; absent when there is no variable to solve for.
    pub solve: Option<native::solve::SolveReport>,
    /// Fresh original-coordinate shooting constraint and bound quality.
    pub quality: Option<native::quality::Quality>,
    pub(crate) completion: super::numerics::Completed,
    /// The NLP candidate: the controls, then every inner node's differential states.
    pub candidate: Option<Vec<f64>>,
    /// Fresh original-coordinate constraint values at the candidate.
    pub constraint_values: Option<Vec<f64>>,
    /// Each control input's interval values at the candidate.
    pub controls: BTreeMap<SemanticId, Vec<f64>>,
    /// The anchored (differential) states at every node, the fixed start first, in
    /// normalized coordinates.
    pub nodes: Vec<Vec<f64>>,
    /// The objective at the candidate.
    pub objective: Option<f64>,
    /// The largest continuity residual at the candidate, in normalized coordinates; zero
    /// for single shooting.
    pub continuity: Option<f64>,
    /// The integration vector at the candidate: the experiment's values with the controls.
    pub parameters: Option<Vec<f64>>,
    /// The candidate's trajectory at the horizon's samples, stitched from the windows,
    /// with every quadrature accumulated from the horizon's start.
    pub trajectory: Option<Arc<native::dynamics::Report>>,
    /// The model's checks on the stitched trajectory at the candidate.
    pub checks: Vec<super::ModelingCheck>,
    /// Authored reports evaluated with the candidate trajectory.
    pub reports: Vec<super::ModelingReport>,
    /// Whether every sample check was evaluated.
    pub checks_complete: bool,
    /// Why the sample checks could not be evaluated, if they could not.
    pub validation_error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
}

impl super::ModelingPackage {
    /// Prepare the shooting problem an authored shooting procedure declares (ADR-0110
    /// Outcome 5, ADR-0119): the case's simulation, its schedules held free as the controls
    /// from their authored values, its shooting method and nodes, and the model's objective
    /// level, minimized. A given `profile` replaces the fixture's integration controls.
    pub async fn declared_shooting(
        &self,
        root: pse_modeling::DeclarationId,
        compiler: pse_compiler::workspace::Profile,
        profile: Option<native::dynamics::Profile>,
        solver: SolverProfile,
        limits: pse_modeling::Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ShootingProblem, WorkflowError> {
        let execution = self
            .declared_execution(root, compiler, solver, Default::default(), limits, cancel)
            .await?;
        self.shooting_for_declared(&execution, profile, cancel)
            .await
    }
    /// Prepare shooting from the operation's already admitted authored execution.
    pub(in crate::workflow) async fn shooting_for_declared(
        &self,
        execution: &super::modeling::DeclaredExecution,
        profile: Option<native::dynamics::Profile>,
        cancel: &crate::CancelSource,
    ) -> Result<ShootingProblem, WorkflowError> {
        if !matches!(
            execution.procedure,
            super::modeling::DeclaredProcedure::Shooting { .. }
        ) {
            return Err(contract(
                "declared shooting requires the authored shooting procedure",
            ));
        }
        let simulation = self
            .simulation_for_declared(execution, profile, cancel)
            .await?;
        let request = simulation.authored_shooting(
            execution.analysis.instance,
            execution.analysis.solver.clone(),
        )?;
        let control = pse_columnar::flight::FlightCancellation::default();
        let operation = self
            .runtime
            .shared
            .math()
            .job(1, 0, control.clone(), move |flag| {
                Ok(ShootingProblem::new(&simulation, request, &flag))
            });
        tokio::pin!(operation);
        tokio::select! {
            result = &mut operation => result.map_err(WorkflowError::from)?,
            () = cancel.cancelled() => {
                control.cancel();
                let _ = operation.await;
                Err(crate::math::MathRuntimeError::Cancelled.into())
            }
        }
    }
}

impl super::ModelingSimulation {
    /// Prepare single or multiple shooting over this simulation (ADR-0110 Outcome 5). The
    /// controls are scheduled inputs of the simulation's profile; path bounds and terminal
    /// weights name outputs; integral weights name declared quadratures.
    pub fn shooting(&self, request: ShootingProfile) -> Result<ShootingProblem, WorkflowError> {
        ShootingProblem::new(self, request, &AtomicBool::new(false))
    }
}

impl ShootingProblem {
    fn new(
        simulation: &super::ModelingSimulation,
        request: ShootingProfile,
        cancel: &AtomicBool,
    ) -> Result<Self, WorkflowError> {
        let profile = simulation.profile();
        let c = simulation.contract();
        if request.solver.controls.hessian != HessianMode::LimitedMemory {
            return Err(contract(
                "shooting supplies first derivatives; request the limited-memory Hessian",
            ));
        }
        request
            .solver
            .controls
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        match request.method {
            ShootingMethod::Single if !request.nodes.is_empty() => {
                return Err(contract("single shooting has no inner nodes"));
            }
            ShootingMethod::Multiple
                if request.nodes.is_empty()
                    || request.nodes.windows(2).any(|w| w[0] >= w[1])
                    || request
                        .nodes
                        .iter()
                        .any(|t| !t.is_finite() || *t <= profile.start || *t >= profile.end) =>
            {
                return Err(contract(
                    "multiple shooting nodes are increasing times strictly inside the horizon",
                ));
            }
            _ => {}
        }
        let np = c.parameters.len();
        // Controls: every interval of a scheduled input, in schedule order.
        let mut controls = Vec::new();
        let mut ports = Vec::new();
        let parameter_ports = simulation
            .model()
            .compiled()
            .admitted
            .case()
            .parameters()
            .iter()
            .map(|p| (p.id, (p.quantity, p.unit)))
            .collect::<BTreeMap<_, _>>();
        for control in &request.controls {
            let parameter = c
                .parameters
                .iter()
                .position(|id| *id == control.input)
                .ok_or_else(|| contract("a shooting control is not a dynamic parameter"))?;
            let schedule = profile
                .schedule
                .iter()
                .find(|s| s.parameter == parameter)
                .ok_or_else(|| contract("a shooting control needs a scheduled input"))?;
            let (lower, upper) = (
                control.lower.unwrap_or(f64::NEG_INFINITY),
                control.upper.unwrap_or(f64::INFINITY),
            );
            if lower.is_nan()
                || upper.is_nan()
                || lower > upper
                || controls
                    .iter()
                    .any(|c: &Control| profile.columns_at(np, profile.start)[parameter] == c.column)
            {
                return Err(contract("shooting control bounds or duplicate control"));
            }
            let port = parameter_ports
                .get(&control.input)
                .copied()
                .ok_or_else(|| contract("shooting control physical port absent"))?;
            let times = std::iter::once(profile.start).chain(schedule.times.iter().copied());
            for (interval, t) in times.enumerate() {
                controls.push(Control {
                    input: control.input,
                    column: profile.columns_at(np, t)[parameter],
                    variable: Variable {
                        id: pse_ids::named_id(
                            control.input,
                            &format!("shooting.control.{interval}"),
                        ),
                        lower,
                        upper,
                    },
                });
                ports.push(port);
            }
        }
        let output_ports = c
            .outputs
            .iter()
            .map(|id| {
                let row = simulation
                    .model()
                    .compiled()
                    .admitted
                    .case()
                    .rows()
                    .iter()
                    .find(|r| r.id == *id)
                    .ok_or_else(|| contract("dynamic output contract absent"))?;
                let unit = simulation
                    .source
                    .quantities
                    .quantity_type(row.quantity)
                    .map_err(math)?
                    .canonical_unit;
                Ok(Port {
                    id: *id,
                    quantity: row.quantity,
                    unit,
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        // Windows over the node intervals, each sampled at the horizon's samples inside it
        // and at its end.
        let times = std::iter::once(profile.start)
            .chain(request.nodes.iter().copied())
            .chain(std::iter::once(profile.end))
            .collect::<Vec<_>>();
        let experiment = IntegratedExperiment {
            snapshot: simulation.snapshot().clone(),
            program: simulation.program(),
            profile: profile.clone(),
            parameters: simulation.parameters.clone(),
            output_ports,
            bindings: controls
                .iter()
                .enumerate()
                .map(|(i, control)| Binding {
                    local: control.column,
                    parameter: i,
                    conversion: pse_quantity::UnitConvertSpec {
                        from: ports[i].1,
                        to: ports[i].1,
                        scale: 1.,
                        offset: 0.,
                    },
                })
                .collect(),
        };
        let mut windows = Vec::new();
        let mut samples = vec![(0, 0); profile.samples.len()];
        for (k, bounds) in times.windows(2).enumerate() {
            let (start, end) = (bounds[0], bounds[1]);
            let last = k + 2 == times.len();
            let mut local = Vec::new();
            for (i, t) in profile.samples.iter().enumerate() {
                if *t >= start && (*t < end || (last && *t <= end)) {
                    samples[i] = (k, local.len());
                    local.push(*t);
                }
            }
            if local.last() != Some(&end) {
                local.push(end);
            }
            windows.push(
                experiment
                    .window(start, end, local)
                    .map_err(|e| WorkflowError::Math(e.into()))?,
            );
        }
        let differential = (0..c.states.len())
            .filter(|i| c.differential[*i])
            .collect::<Vec<_>>();
        // The fixed start: the requested differential initial values at the experiment's
        // parameters.
        let start = {
            use native::dynamics::Oracle;
            let mut worker = simulation
                .worker(pse_kernels::ExecutionScope::new(
                    Arc::new(AtomicBool::new(false)),
                    None,
                ))
                .map_err(|e| WorkflowError::Math(e.into()))?;
            let initial = worker
                .evaluate(
                    0,
                    native::dynamics::Function::Initial,
                    profile.start,
                    &vec![0.; c.states.len()],
                    &profile.parameters_at(&simulation.parameters, profile.start),
                    false,
                )
                .map_err(|e| WorkflowError::Math(e.into()))?;
            differential
                .iter()
                .map(|i| initial.values[*i])
                .collect::<Vec<_>>()
        };
        let position = |ids: &[SemanticId], id: &SemanticId, what: &str| {
            ids.iter()
                .position(|o| o == id)
                .ok_or_else(|| contract(format!("shooting {what} is not declared")))
        };
        let terminal = request
            .objective
            .terminal
            .iter()
            .map(|(id, w)| Ok((position(&c.outputs, id, "terminal output")?, *w)))
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        let integral = request
            .objective
            .integral
            .iter()
            .map(|(id, w)| Ok((position(&c.quadratures, id, "integral quadrature")?, *w)))
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        if terminal
            .iter()
            .chain(&integral)
            .any(|(_, w)| !w.is_finite())
        {
            return Err(contract("shooting objective weights are finite"));
        }
        // Rows: continuity at every inner node, then every path bound at every sample.
        let mut rows = Vec::new();
        let mut bounds = Vec::new();
        let mut row_ids = Vec::new();
        let mut row_ports = Vec::new();
        let neutral = simulation
            .source
            .quantities
            .neutral_dimensionless()
            .ok_or_else(|| contract("shooting requires the bound neutral quantity"))?;
        let neutral_unit = simulation
            .source
            .quantities
            .quantity_type(neutral)
            .map_err(math)?
            .canonical_unit;
        for window in 0..windows.len() - 1 {
            for (anchor, state) in differential.iter().enumerate() {
                rows.push(Row::Continuity { window, anchor });
                bounds.push((0., 0.));
                row_ids.push(pse_ids::named_id(
                    c.states[*state],
                    &format!("shooting.continuity.{window}"),
                ));
                row_ports.push((neutral, neutral_unit));
            }
        }
        for bound in &request.path {
            let output = position(&c.outputs, &bound.output, "path output")?;
            let (lower, upper) = (
                bound.lower.unwrap_or(f64::NEG_INFINITY),
                bound.upper.unwrap_or(f64::INFINITY),
            );
            if lower.is_nan() || upper.is_nan() || lower > upper {
                return Err(contract("shooting path bounds"));
            }
            for sample in 0..profile.samples.len() {
                rows.push(Row::Path { sample, output });
                bounds.push((lower, upper));
                row_ids.push(pse_ids::named_id(
                    bound.output,
                    &format!("shooting.path.{sample}"),
                ));
                let port = &experiment.output_ports[output];
                row_ports.push((port.quantity, port.unit));
            }
        }
        let mut variables = controls
            .iter()
            .map(|c| c.variable.clone())
            .collect::<Vec<_>>();
        let mut targets = controls
            .iter()
            .zip(&ports)
            .map(|(c, (quantity, unit))| {
                target(c.variable.id, NumericalTarget::Variable, *quantity, *unit)
            })
            .collect::<Vec<_>>();
        for window in 1..windows.len() {
            for state in &differential {
                let id = pse_ids::named_id(c.states[*state], &format!("shooting.node.{window}"));
                variables.push(Variable {
                    id,
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                });
                targets.push(target(id, NumericalTarget::Variable, neutral, neutral_unit));
            }
        }
        targets.extend(
            row_ids
                .iter()
                .zip(&row_ports)
                .map(|(id, (quantity, unit))| target(*id, NumericalTarget::Row, *quantity, *unit)),
        );
        targets.push(target(
            SemanticId::NIL,
            NumericalTarget::Objective,
            neutral,
            neutral_unit,
        ));
        let numerics = pse_math::numerics::resolve(
            &simulation.source.quantities,
            &targets,
            &[],
            &request.solver.numerics,
        )
        .map_err(math)?;
        let columns = variables.iter().map(|v| v.id).collect::<Vec<_>>();
        let normalization =
            Normalization::from_policy(&numerics, &columns, &row_ids).map_err(math)?;
        let tolerances = native::quality::Tolerances::from_policy(&numerics, &columns, &row_ids)
            .map_err(crate::math::MathRuntimeError::from)?;
        let accuracy =
            native::solve::ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)
                .map_err(crate::math::MathRuntimeError::from)?;
        let mut session = FramedHasher::new(pse_ids::Frame::ShootingProblemV1);
        session
            .str("profile")
            .hash(
                &crate::math::solves::profile_key(&request.solver)
                    .map_err(crate::math::MathRuntimeError::from)?
                    .as_id(),
            )
            .hash(&numerics.key);
        let mut identity = FramedHasher::new(pse_ids::Frame::ShootingProblemV1);
        identity
            .hash(&simulation.identity())
            .str(request.method.as_str());
        for t in &request.nodes {
            identity.u64(t.to_bits());
        }
        for (v, c) in variables.iter().zip(&controls) {
            identity.id(&v.id).u64(c.column as u64);
        }
        for id in &row_ids {
            identity.id(id);
        }
        for (o, w) in terminal.iter().chain(&integral) {
            identity.u64(*o as u64).u64(w.to_bits());
        }
        let oracle_contract = OracleContract {
            identity: identity.finish_hash(),
            variables,
            rows: row_ids,
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        let mut problem = Self {
            snapshot: native::execution::Snapshot::observe(&native::execution::LINKED),
            structural_assessment: None,
            simulation: simulation.clone(),
            experiment,
            method: request.method,
            runtime: simulation.runtime.clone(),
            solver: request.solver,
            windows,
            samples,
            controls,
            differential,
            start,
            rows,
            bounds,
            terminal,
            integral,
            contract: oracle_contract,
            // Filled below from the rows and the windows.
            pattern: SymbolicSparseColMat::new_checked(0, 0, vec![0], None, vec![]),
            normalization,
            tolerances,
            accuracy,
            route: native::routing::Route::Constant,
            profile_key: pse_ids::roles::ProfileHash::from_id(session.finish_hash()),
            numerics,
        };
        problem.pattern = problem.jacobian_structure()?;
        problem.admit_windows(simulation)?;
        if !problem.contract.variables.is_empty() {
            let structural_extent =
                native::structural::construction_bytes(&problem.contract, problem.pattern.as_ref())
                    .map_err(crate::math::MathRuntimeError::from)?;
            let reservation =
                datafusion::execution::memory_pool::MemoryConsumer::new("shooting:structure")
                    .register(&problem.runtime.shared.pool());
            reservation
                .try_grow(structural_extent)
                .map_err(crate::math::MathRuntimeError::from)?;
            let structure = native::structural::oracle_structure_with_cancel(
                &problem.contract,
                problem.pattern.as_ref(),
                &problem.bounds,
                true,
                cancel,
            )
            .map_err(crate::math::MathRuntimeError::from)?;
            let facts = native::routing::oracle_facts(&problem.contract, true, false);
            let decision = native::routing::Requirements {
                table: &native::execution::LINKED,
                facts: &facts,
                intent: problem.solver.intent,
                numerical_psd: false,
                least_squares: false,
                controls: &problem.solver.controls,
                settings: &problem.solver.backend,
                sensitivity: false,
                context: native::routing::Context {
                    snapshot: problem.snapshot.clone(),
                    pending_classes: &[],
                    structure: Some(structure),
                    oracle: Some(&problem.contract),
                    guards: &BTreeMap::new(),
                    budgets: Some(native::execution::Budgets {
                        tolerances: &problem.tolerances,
                        normalization: &problem.normalization,
                        accuracy: &problem.accuracy,
                    }),
                    coefficients: None,
                    cone: None,
                    factorable: None,
                    certificate: None,
                    prepared: &[native::routing::ArtifactDemand::Representation(
                        native::execution::Representation::Nlp,
                    )],
                    refusals: &BTreeMap::new(),
                },
            }
            .decision(problem.solver.selection);
            problem.route = decision
                .route()
                .map_err(crate::math::MathRuntimeError::from)?;
            problem.structural_assessment = decision.structure;
            if let Some(assessment) = &mut problem.structural_assessment {
                let retained = assessment.retained_bytes();
                if retained > reservation.size() {
                    return Err(contract("shooting structural retained allowance"));
                }
                reservation.shrink(reservation.size() - retained);
                assessment.witness = assessment
                    .witness
                    .clone()
                    .with_owner(pse_columnar::AllocationLease::new(reservation));
            }
            crate::math::solves::admit_profile(&problem.solver, problem.route, &problem.snapshot)
                .map_err(crate::math::MathRuntimeError::from)?;
        }
        Ok(problem)
    }
    /// The NLP's variables, rows and bounds.
    pub fn contract(&self) -> &OracleContract {
        &self.contract
    }
    /// The fixed start of the first window: the anchored (differential) states in
    /// normalized coordinates, in state order.
    pub fn initial_state(&self) -> &[f64] {
        &self.start
    }
    /// Start the horizon from another state (a receding horizon's measured or estimated
    /// state), in normalized coordinates of the anchored states.
    pub fn set_start(&mut self, start: Vec<f64>) -> Result<(), WorkflowError> {
        if start.len() != self.differential.len() || start.iter().any(|v| !v.is_finite()) {
            return Err(contract("shooting start extent or value"));
        }
        self.start = start;
        Ok(())
    }
    /// The number of control variables; the inner nodes' states follow them.
    pub fn controls(&self) -> usize {
        self.controls.len()
    }
    /// The anchors of window `k` at the variables `x`.
    fn anchors<'x>(&'x self, k: usize, x: &'x [f64]) -> &'x [f64] {
        if k == 0 {
            &self.start
        } else {
            let nd = self.differential.len();
            let first = self.controls.len() + (k - 1) * nd;
            &x[first..first + nd]
        }
    }
    /// The experiment's integration vector with the controls from `x`: every control is
    /// a binding of the experiment.
    fn integration(&self, x: &[f64]) -> Vec<f64> {
        self.experiment.bind(&|i| x[i])
    }
    /// Every window's anchored profile is admitted before native work: with forward
    /// sensitivities when rows or a terminal objective need them, and on the adjoint route
    /// with observed quadratures when the objective has an integral part.
    fn admit_windows(&self, simulation: &super::ModelingSimulation) -> Result<(), WorkflowError> {
        let n = self.contract.variables.len();
        let forward = n > 0 && (!self.rows.is_empty() || !self.terminal.is_empty());
        let adjoint = n > 0 && !self.integral.is_empty();
        let problem = |e: ProblemError| WorkflowError::Math(e.into());
        let worker = |observed| {
            native::dynamics::Anchored::new(
                simulation.worker(pse_kernels::ExecutionScope::new(
                    Arc::new(AtomicBool::new(false)),
                    None,
                ))?,
                observed,
            )
        };
        let (plain, observed) = (
            worker(false).map_err(problem)?,
            worker(true).map_err(problem)?,
        );
        let integration = self.experiment.parameters.clone();
        for window in &self.windows {
            let parameters = IntegratedExperiment::window_parameters(
                window,
                &integration,
                &vec![0.; self.differential.len()],
            )
            .map_err(problem)?;
            let mut profile = plain.profile(&window.profile);
            profile.sensitivity = if forward {
                DynamicSensitivity::Forward
            } else {
                DynamicSensitivity::None
            };
            profile
                .validate(native::dynamics::Oracle::contract(&plain), &parameters)
                .map_err(problem)?;
            if adjoint {
                let mut profile = observed.profile(&window.profile);
                profile.sensitivity = DynamicSensitivity::Adjoint;
                profile
                    .validate(native::dynamics::Oracle::contract(&observed), &parameters)
                    .map_err(problem)?;
            }
        }
        Ok(())
    }
    /// The NLP variable a window's integration column moves, if any.
    fn variable(&self, window: usize, column: &WindowColumn) -> Option<usize> {
        match column {
            WindowColumn::Integration(g) => self.controls.iter().position(|c| c.column == *g),
            WindowColumn::Anchor(a) if window > 0 => {
                Some(self.controls.len() + (window - 1) * self.differential.len() + a)
            }
            WindowColumn::Anchor(_) => None,
        }
    }
    /// The window a row reads and, for a continuity row, the next node's variable.
    fn row_window(&self, row: &Row) -> (usize, Option<usize>) {
        match row {
            Row::Continuity { window, anchor } => (
                *window,
                Some(self.controls.len() + window * self.differential.len() + anchor),
            ),
            Row::Path { sample, .. } => (self.samples[*sample].0, None),
        }
    }
    fn jacobian_structure(&self) -> Result<SymbolicSparseColMat<usize>, WorkflowError> {
        let mut pairs = Vec::new();
        for (r, row) in self.rows.iter().enumerate() {
            let (window, next) = self.row_window(row);
            let mut columns = self.windows[window]
                .columns
                .iter()
                .filter_map(|c| self.variable(window, c))
                .chain(next)
                .collect::<Vec<_>>();
            columns.sort_unstable();
            columns.dedup();
            pairs.extend(columns.into_iter().map(|c| faer::sparse::Pair::new(r, c)));
        }
        SymbolicSparseColMat::try_new_from_indices(
            self.rows.len(),
            self.contract.variables.len(),
            &pairs,
        )
        .map(|(pattern, _)| pattern)
        .map_err(|e| contract(format!("shooting Jacobian pattern: {e:?}")))
    }
    /// The initial point: the experiment's control values and, for multiple shooting, the
    /// nodes of one pass that chains the windows from the start at those controls, so the
    /// continuity rows hold there.
    pub fn initial_point(&self, execution: &Execution) -> Result<Vec<f64>, ProblemError> {
        let mut x = self
            .controls
            .iter()
            .map(|c| self.experiment.parameters[c.column])
            .collect::<Vec<_>>();
        let integration = self.integration(&x);
        let mut anchors = self.start.clone();
        for window in self.windows.iter().take(self.windows.len() - 1) {
            let report = self.experiment.integrate_window(
                window,
                &integration,
                &anchors,
                execution,
                DynamicSensitivity::None,
            )?;
            let end = report
                .samples
                .last()
                .ok_or_else(|| ProblemError::internal("shooting window without its end sample"))?;
            anchors = self.differential.iter().map(|i| end.state[*i]).collect();
            x.extend(&anchors);
        }
        Ok(x)
    }
}

/// A numerical target of the shooting NLP.
fn target(
    id: SemanticId,
    kind: NumericalTarget,
    quantity: pse_quantity::QuantityTypeId,
    unit: pse_quantity::UnitId,
) -> TargetSpec {
    TargetSpec {
        id,
        kind,
        quantity,
        unit,
        integer: false,
        declared_tolerance: None,
    }
}

/// One evaluated point: every window's report and the rows, objective and derivatives.
#[derive(Debug)]
struct Point {
    x: Vec<f64>,
    reports: Vec<native::dynamics::Report>,
    constraints: Vec<f64>,
    jacobian: Vec<f64>,
    objective: f64,
    /// The terminal part of the objective gradient, from forward sensitivities.
    gradient: Vec<f64>,
    /// The integral part, from the windows' adjoints, once computed.
    adjoint: Option<Vec<f64>>,
}
/// The shooting NLP oracle over one attempt.
#[derive(Debug)]
struct ShootingOracle {
    problem: Arc<ShootingProblem>,
    execution: Execution,
    point: Option<Point>,
}
impl ShootingOracle {
    fn evaluate(&mut self, x: &[f64]) -> Result<&Point, ProblemError> {
        if let Some(stop) = self.execution.stopped() {
            return Err(ProblemError::stopped(stop, "shooting evaluation deadline"));
        }
        let p = self.problem.clone();
        let n = p.contract.variables.len();
        if x.len() != n || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("shooting trial coordinates"));
        }
        if self
            .point
            .as_ref()
            .is_some_and(|v| v.x.iter().zip(x).all(|(a, b)| a.to_bits() == b.to_bits()))
        {
            return self
                .point
                .as_ref()
                .ok_or_else(|| ProblemError::internal("shooting point cache"));
        }
        self.point = None;
        // Forward sensitivities serve the rows and the terminal objective.
        let forward = n > 0 && (!p.rows.is_empty() || !p.terminal.is_empty());
        let sensitivity = if forward {
            DynamicSensitivity::Forward
        } else {
            DynamicSensitivity::None
        };
        let integration = p.integration(x);
        let reports = p
            .windows
            .iter()
            .enumerate()
            .map(|(k, window)| {
                p.experiment.integrate_window(
                    window,
                    &integration,
                    p.anchors(k, x),
                    &self.execution,
                    sensitivity,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let end = |k: usize| {
            reports[k]
                .samples
                .last()
                .ok_or_else(|| ProblemError::internal("shooting window without its end sample"))
        };
        let width = |k: usize| p.windows[k].columns.len();
        let mut constraints = Vec::with_capacity(p.rows.len());
        let mut jacobian = vec![0.; p.pattern.row_idx().len()];
        let mut entries = BTreeMap::<(usize, usize), f64>::new();
        for (r, row) in p.rows.iter().enumerate() {
            let (k, next) = p.row_window(row);
            let (value, derivatives) = match row {
                Row::Continuity { anchor, .. } => {
                    let sample = end(k)?;
                    let state = p.differential[*anchor];
                    let value = sample.state[state] - x[next.unwrap_or_default()];
                    let derivatives = forward.then(|| {
                        &sample.state_sensitivities[state * width(k)..(state + 1) * width(k)]
                    });
                    (value, derivatives)
                }
                Row::Path { sample, output } => {
                    let (_, local) = p.samples[*sample];
                    let sample = reports[k]
                        .samples
                        .get(local)
                        .ok_or_else(|| ProblemError::internal("shooting path sample"))?;
                    let derivatives = forward.then(|| {
                        &sample.output_sensitivities[output * width(k)..(output + 1) * width(k)]
                    });
                    (sample.outputs[*output], derivatives)
                }
            };
            constraints.push(value);
            if let Some(derivatives) = derivatives {
                for (column, d) in p.windows[k].columns.iter().zip(derivatives) {
                    if let Some(v) = p.variable(k, column) {
                        *entries.entry((r, v)).or_default() += d;
                    }
                }
            }
            if let Some(v) = next {
                *entries.entry((r, v)).or_default() -= 1.;
            }
        }
        for col in 0..p.pattern.ncols() {
            for k in p.pattern.col_range(col) {
                jacobian[k] = entries
                    .get(&(p.pattern.row_idx()[k], col))
                    .copied()
                    .unwrap_or_default();
            }
        }
        // The objective: terminal outputs at the end, and each window's quadratures.
        let last = p.windows.len() - 1;
        let finish = end(last)?;
        let mut objective = p
            .terminal
            .iter()
            .map(|(o, w)| w * finish.outputs[*o])
            .sum::<f64>();
        for k in 0..p.windows.len() {
            let sample = end(k)?;
            objective += p
                .integral
                .iter()
                .map(|(q, w)| w * sample.integrals[*q])
                .sum::<f64>();
        }
        let mut gradient = vec![0.; n];
        if forward {
            for (o, w) in &p.terminal {
                let sensitivities =
                    &finish.output_sensitivities[o * width(last)..(o + 1) * width(last)];
                for (column, d) in p.windows[last].columns.iter().zip(sensitivities) {
                    if let Some(v) = p.variable(last, column) {
                        gradient[v] += w * d;
                    }
                }
            }
        }
        if !objective.is_finite()
            || constraints
                .iter()
                .chain(&jacobian)
                .chain(&gradient)
                .any(|v| !v.is_finite())
        {
            return Err(ProblemError::numerical(
                "nonfinite shooting row or objective",
            ));
        }
        self.execution.progress.push(native::solve::Event {
            phase: "shooting.evaluation".into(),
            elapsed: self.execution.started.elapsed(),
            values: BTreeMap::from([(
                "windows".into(),
                native::solve::Metric::Integer(p.windows.len() as i64),
            )]),
            incumbent: None,
        });
        self.point = Some(Point {
            x: x.to_vec(),
            reports,
            constraints,
            jacobian,
            objective,
            gradient,
            adjoint: None,
        });
        self.point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("shooting point publication"))
    }
    /// The integral objective's gradient: one adjoint per window over its observed
    /// quadratures, whose outputs follow the model's own at the window's end.
    fn adjoint(&mut self, x: &[f64]) -> Result<Vec<f64>, ProblemError> {
        if let Some(gradient) = self.point.as_ref().and_then(|p| p.adjoint.clone()) {
            return Ok(gradient);
        }
        let p = self.problem.clone();
        let mut total = vec![0.; x.len()];
        if !p.integral.is_empty() && !x.is_empty() {
            let integration = p.integration(x);
            let m = p.experiment.program.contract.outputs.len();
            let nq = p.experiment.program.contract.quadratures.len();
            for (k, window) in p.windows.iter().enumerate() {
                let mut cotangent = |r: &native::dynamics::Report| {
                    let mut w = vec![0.; r.samples.len() * (m + nq)];
                    let last = (r.samples.len() - 1) * (m + nq);
                    for (q, weight) in &p.integral {
                        w[last + m + q] += weight;
                    }
                    Ok(w)
                };
                let (_, gradient) = p.experiment.window_gradient(
                    window,
                    &integration,
                    p.anchors(k, x),
                    &self.execution,
                    &mut cotangent,
                )?;
                for (column, d) in window.columns.iter().zip(&gradient) {
                    if let Some(v) = p.variable(k, column) {
                        total[v] += d;
                    }
                }
            }
        }
        if let Some(point) = self.point.as_mut() {
            point.adjoint = Some(total.clone());
        }
        Ok(total)
    }
}
impl NlpOracle for ShootingOracle {
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.problem.normalization)
    }
    fn contract(&self) -> &OracleContract {
        &self.problem.contract
    }
    fn jacobian_pattern(&self) -> SymbolicSparseColMatRef<'_, usize> {
        self.problem.pattern.as_ref()
    }
    fn hessian_pattern(&self) -> Option<SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.problem.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.evaluate(x).map(|p| p.objective)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let p = self.evaluate(x)?;
        if out.len() != p.constraints.len() {
            return Err(ProblemError::internal("shooting constraint extent"));
        }
        out.copy_from_slice(&p.constraints);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let terminal = self.evaluate(x)?.gradient.clone();
        if out.len() != terminal.len() {
            return Err(ProblemError::internal("shooting gradient extent"));
        }
        let integral = self.adjoint(x)?;
        for ((o, t), i) in out.iter_mut().zip(terminal).zip(integral) {
            *o = t + i;
            if !o.is_finite() {
                return Err(ProblemError::numerical("nonfinite shooting gradient"));
            }
        }
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let p = self.evaluate(x)?;
        if out.len() != p.jacobian.len() {
            return Err(ProblemError::internal("shooting Jacobian extent"));
        }
        out.copy_from_slice(&p.jacobian);
        Ok(())
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(ProblemError::internal(
            "limited-memory shooting Hessian demand",
        ))
    }
}

impl ShootingProblem {
    /// Solve from `initial` (or [`Self::initial_point`]) on the admitted worker: the NLP
    /// through the one NLP runner, then a fresh evaluation of the candidate for its
    /// objective, continuity residual and stitched trajectory.
    pub(crate) fn solve(
        self: &Arc<Self>,
        run_id: RunId,
        scope: pse_kernels::ExecutionScope,
        progress: Arc<native::solve::Progress>,
        initial: Option<&[f64]>,
        solver: &SolverProfile,
    ) -> Result<ShootingReport, crate::math::MathRuntimeError> {
        let operation_profile = crate::math::solves::profile_key(solver)?;
        let adapters: Vec<&dyn native::execution::BackendExecution> = match self.route {
            native::routing::Route::Native(backend) => vec![native::execution::adapter(backend)],
            native::routing::Route::Constant => vec![],
        };
        native::execution::scoped(
            &adapters,
            solver.controls.threads,
            self.runtime.native().stack_bytes(),
            || {
                let backend = match self.route {
                    native::routing::Route::Native(backend) => Some(backend),
                    native::routing::Route::Constant => None,
                };
                let (mut report, trace) = crate::math::opaque_strategy::direct(
                    self.runtime.shared.math(),
                    crate::math::opaque_strategy::Source {
                        original: self.contract.identity,
                        preparation: self.contract.identity,
                        profile: operation_profile.as_id(),
                        backend,
                        controls: &solver.controls,
                        start: if initial.is_some() {
                            pse_model::strategy::StartOrigin::Explicit
                        } else {
                            pse_model::strategy::StartOrigin::Specification
                        },
                        start_identity: initial.map(|point| {
                            crate::math::opaque_strategy::point_identity(
                                self.contract.identity,
                                point,
                            )
                        }),
                    },
                    &scope,
                    || self.solve_inner(run_id, scope.clone(), progress, initial, solver),
                    |report| report.solve.as_ref(),
                    |report| report.completion.decision.clone(),
                    |report| {
                        report
                            .validation_error
                            .as_ref()
                            .map(crate::math::opaque_strategy::assessment_failure)
                    },
                )?;
                report.strategy = Some(trace);
                Ok(report)
            },
        )
    }
    fn solve_inner(
        self: &Arc<Self>,
        run_id: RunId,
        scope: pse_kernels::ExecutionScope,
        progress: Arc<native::solve::Progress>,
        initial: Option<&[f64]>,
        solver: &SolverProfile,
    ) -> Result<ShootingReport, crate::math::MathRuntimeError> {
        let operation_profile = crate::math::solves::profile_key(solver)?;
        let mut execution =
            Execution::within(scope.cancellation().clone(), &solver.controls, scope)?;
        execution.progress = progress;
        execution.memory = Some(
            self.runtime
                .shared
                .budget()
                .math
                .foreign_allowance(&solver.controls),
        );
        let initial = match initial {
            Some(x) => x.to_vec(),
            None => self.initial_point(&execution)?,
        };
        if initial.len() != self.contract.variables.len() {
            return Err(ProblemError::Contract("shooting initial point extent".into()).into());
        }
        let (solve, candidate) = if initial.is_empty() {
            (None, Some(vec![]))
        } else {
            let native::routing::Route::Native(backend) = self.route else {
                return Err(ProblemError::unsupported("shooting NLP has no native route").into());
            };
            let mut h = FramedHasher::new(pse_ids::Frame::ShootingProblemV1);
            for v in &self.experiment.parameters {
                h.u64(v.to_bits());
            }
            for v in &self.start {
                h.u64(v.to_bits());
            }
            let report = native::execution::nlp(
                native::execution::Step {
                    adapter: native::execution::adapter(backend),
                    snapshot: &self.snapshot,
                    structure: self.structural_assessment.as_ref(),
                    settings: &solver.backend,
                    controls: &solver.controls,
                    accuracy: &self.accuracy,
                    execution: execution.clone(),
                    tolerances: &self.tolerances,
                    normalization: &self.normalization,
                    compatibility: Compatibility {
                        layout: self.contract.identity,
                        profile: operation_profile.as_id(),
                        data: h.finish_hash(),
                        backend,
                    },
                    warm: None,
                },
                &mut native::execution::Retained::default(),
                native::execution::Nlp {
                    oracle: Box::new(ShootingOracle {
                        problem: self.clone(),
                        execution: execution.clone(),
                        point: None,
                    }),
                    initial: &initial,
                    presolve: &solver.presolve,
                    intent: solver.intent,
                    sense: pse_math::binding::ObjectiveSense::Minimize,
                    limit: self.experiment.profile.max_cells,
                    analysis: native::execution::Analysis::for_intent(solver.intent),
                },
            )?;
            let candidate = report.candidate.as_ref().map(|c| c.primal.clone());
            (Some(report), candidate)
        };
        let mut report = ShootingReport {
            strategy: None,
            method: self.method,
            solve,
            candidate,
            quality: None,
            constraint_values: None,
            completion: super::numerics::Completed {
                closure: pse_model::generated::enums::ClosureAssessment::Unavailable,
                decision: super::numerics::refused(
                    pse_model::generated::enums::CandidateRefusal::NoCandidate,
                ),
            },
            controls: BTreeMap::new(),
            nodes: Vec::new(),
            objective: None,
            continuity: None,
            parameters: None,
            trajectory: None,
            checks: Vec::new(),
            reports: Vec::new(),
            checks_complete: false,
            validation_error: None,
        };
        if let Some(x) = report.candidate.clone() {
            // A fresh evaluation, independent of the native callbacks' cache.
            let mut oracle = ShootingOracle {
                problem: self.clone(),
                execution: execution.clone(),
                point: None,
            };
            let point = match oracle.evaluate(&x) {
                Ok(point) => point,
                Err(error) => {
                    let diagnostic = super::diagnostics::observed(
                        &error,
                        pse_diagnostics::DiagnosticStage::Shooting,
                    );
                    if let Some(native) = report.solve.as_mut() {
                        native.record_validation_failure(error);
                    }
                    report.validation_error = Some(diagnostic);
                    self.assess_completion(&mut report);
                    return Ok(report);
                }
            };
            report.quality = Some(native::quality::observed(
                &self.contract,
                &self.bounds,
                &x,
                &point.constraints,
                &self.tolerances,
            )?);
            report.constraint_values = Some(point.constraints.clone());
            report.objective = Some(point.objective);
            report.continuity = Some(
                self.rows
                    .iter()
                    .zip(&point.constraints)
                    .filter(|(r, _)| matches!(r, Row::Continuity { .. }))
                    .map(|(_, v)| v.abs())
                    .fold(0., f64::max),
            );
            let integration = self.integration(&x);
            for (control, value) in self.controls.iter().zip(&x) {
                report
                    .controls
                    .entry(control.input)
                    .or_default()
                    .push(*value);
            }
            report.nodes = (0..self.windows.len())
                .map(|k| self.anchors(k, &x).to_vec())
                .collect();
            let mut stitched = native::dynamics::Report {
                termination: native::dynamics::Termination::Completed,
                completed_time: self.experiment.profile.end,
                requested_initial: point.reports[0].requested_initial.clone(),
                consistent_initial: point.reports[0].consistent_initial.clone(),
                samples: Vec::with_capacity(self.samples.len()),
                endpoint: point.reports.last().and_then(|r| r.endpoint.clone()),
                conservation: Vec::new(),
                events: point
                    .reports
                    .iter()
                    .flat_map(|r| r.events.clone())
                    .collect(),
                statistics: point
                    .reports
                    .iter()
                    .flat_map(|r| r.statistics.clone())
                    .collect(),
                error: None,
                progress: Vec::new(),
                dropped_progress: 0,
            };
            // Each window integrates its quadratures from its own start.
            let mut before = vec![vec![0.; self.experiment.program.contract.quadratures.len()]];
            for window in &point.reports {
                let end = window.samples.last().ok_or_else(|| {
                    ProblemError::internal("shooting window without its end sample")
                })?;
                let previous = before.last().cloned().unwrap_or_default();
                before.push(
                    previous
                        .iter()
                        .zip(&end.integrals)
                        .map(|(a, b)| a + b)
                        .collect(),
                );
            }
            if let Some(endpoint) = stitched.endpoint.as_mut() {
                // Window anchors are execution coordinates, not physical model inputs.
                // Preserve the final window's actual left-side values while projecting its
                // column map back onto the original experiment integration vector.
                let parameters = self.experiment.program.contract.parameters.len();
                let window = self
                    .windows
                    .last()
                    .ok_or_else(|| ProblemError::internal("shooting endpoint without window"))?;
                if endpoint.inputs.len() < parameters || endpoint.input_columns.len() < parameters {
                    return Err(
                        ProblemError::internal("shooting endpoint physical input extent").into(),
                    );
                }
                endpoint.inputs.truncate(parameters);
                endpoint.input_columns.truncate(parameters);
                for column in &mut endpoint.input_columns {
                    *column = match window.columns.get(*column) {
                        Some(WindowColumn::Integration(original)) => *original,
                        _ => {
                            return Err(ProblemError::internal(
                                "shooting endpoint physical input map",
                            )
                            .into());
                        }
                    };
                }
                let offset = &before[point.reports.len() - 1];
                for (value, offset) in endpoint.point.integrals.iter_mut().zip(offset) {
                    *value += offset;
                }
                endpoint.point.state_sensitivities.clear();
                endpoint.point.output_sensitivities.clear();
            }
            stitched.conservation = stitch_conservation(
                &self.experiment.program.contract,
                &point.reports,
                &before,
                self.experiment.profile.max_cells,
            )?;
            for (k, local) in &self.samples {
                let mut sample = point.reports[*k].samples[*local].clone();
                sample.state_sensitivities.clear();
                sample.output_sensitivities.clear();
                for (value, offset) in sample.integrals.iter_mut().zip(&before[*k]) {
                    *value += offset;
                }
                stitched.samples.push(sample);
            }
            let checks =
                self.simulation
                    .check_samples(run_id, &stitched, &integration, &execution.scope()?);
            report.checks_complete = checks.complete && checks.error.is_none();
            report.checks = checks.rows;
            report.reports = checks.reports;
            report.validation_error = checks.error;
            report.trajectory = Some(Arc::new(stitched));
            report.parameters = Some(integration);
        }
        self.assess_completion(&mut report);
        Ok(report)
    }
    fn assess_completion(&self, report: &mut ShootingReport) {
        use super::numerics::{CompletionEvidence, complete, constant_use, native_use, refused};
        use pse_model::generated::enums::CandidateRefusal;
        let fresh = report
            .quality
            .as_ref()
            .map_or_else(|| refused(CandidateRefusal::Infeasible), constant_use);
        let native = match report.solve.as_ref() {
            Some(report) => {
                let mut native = native_use(report, &self.numerics.policy);
                if !fresh.permits_use() {
                    native.refuse(CandidateRefusal::Infeasible);
                }
                native
            }
            None => fresh,
        };
        let coverage_complete = report.trajectory.as_ref().is_some_and(|trajectory| {
            self.experiment.profile.samples.iter().all(|required| {
                trajectory
                    .samples
                    .iter()
                    .any(|sample| sample.time == *required)
            })
        });
        let endpoint_satisfied = report.trajectory.as_ref().is_some_and(|trajectory| {
            trajectory
                .assess_endpoint(&self.experiment.profile)
                .satisfied
                && trajectory.error.is_none()
        });
        report.completion = complete(
            native,
            CompletionEvidence {
                checks: &report.checks,
                checks_complete: report.checks_complete && report.validation_error.is_none(),
                required_closure: self.simulation.required_closure_checks(),
                endpoint_satisfied: Some(endpoint_satisfied),
                coverage_complete,
            },
            &self.numerics.policy,
        );
    }
    pub(super) fn encoding_bytes(
        &self,
        report: Option<&ShootingReport>,
    ) -> Result<usize, crate::math::MathRuntimeError> {
        let variables = self
            .contract
            .variables
            .len()
            .checked_mul(size_of::<pse_model::generated::runtime::solve_variables::Row>());
        let rows = self.contract.rows.len().checked_mul(size_of::<
            pse_model::generated::runtime::solve_constraints::Row,
        >());
        variables
            .and_then(|n| rows.and_then(|r| n.checked_add(r)))
            .and_then(|n| n.checked_add(report.map_or(0, ShootingReport::numeric_bytes)))
            .and_then(|n| {
                self.solver
                    .controls
                    .report_allowance()
                    .ok()
                    .and_then(|a| n.checked_add(a))
            })
            .ok_or(crate::math::MathRuntimeError::Limit(
                "shooting encoding extent",
            ))
    }
    pub(super) fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    pub(super) fn tolerances(&self) -> &native::quality::Tolerances {
        &self.tolerances
    }
    pub(super) fn initial_extent(&self) -> usize {
        self.contract.variables.len()
    }
    pub(super) fn numerics(&self) -> &pse_model::numerics::ResolvedNumericalPolicy {
        &self.numerics
    }
    pub(super) fn request_identity(
        &self,
        initial: Option<&[f64]>,
    ) -> pse_ids::roles::LineageRequestHash {
        let mut identity = FramedHasher::new(pse_ids::Frame::ShootingProblemV1);
        identity
            .hash(&self.contract.identity)
            .hash(&self.profile_key.as_id())
            .bool(initial.is_some());
        if let Some(initial) = initial {
            identity.u64(initial.len() as u64);
            for value in initial {
                identity.u64(pse_ids::canonical_f64_bits(*value));
            }
        }
        pse_ids::roles::LineageRequestHash::from(identity.finish_hash())
    }
    pub(super) fn job_bytes(&self) -> Result<usize, crate::math::MathRuntimeError> {
        self.simulation
            .bytes
            .checked_mul(self.windows.len() + 1)
            .and_then(|n| n.checked_add(self.solver.controls.foreign_bytes.unwrap_or(0)))
            .ok_or(crate::math::MathRuntimeError::Limit("shooting extent"))
    }
}
impl ShootingReport {
    pub(super) fn numeric_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        size_of::<Self>()
            + self
                .trajectory
                .as_ref()
                .map_or(0, |report| report.numeric_bytes())
            + self
                .checks
                .iter()
                .map(HeapUsage::owned_bytes)
                .sum::<usize>()
            + self
                .reports
                .iter()
                .map(HeapUsage::owned_bytes)
                .sum::<usize>()
            + self
                .validation_error
                .as_ref()
                .map_or(0, HeapUsage::owned_bytes)
            + self.quality.as_ref().map_or(0, |q| {
                (q.rows.capacity() + q.bounds.capacity() + q.integrality.capacity())
                    * size_of::<native::quality::Violation>()
            })
            + self.completion.decision.qualifiers.capacity()
                * size_of::<pse_model::generated::enums::CandidateQualifier>()
            + self.completion.decision.refusals.capacity()
                * size_of::<pse_model::generated::enums::CandidateRefusal>()
            + (self.candidate.as_ref().map_or(0, Vec::capacity)
                + self.constraint_values.as_ref().map_or(0, Vec::capacity)
                + self.parameters.as_ref().map_or(0, Vec::capacity)
                + self.nodes.iter().map(Vec::capacity).sum::<usize>()
                + self.controls.values().map(Vec::capacity).sum::<usize>())
                * size_of::<f64>()
    }
}

/// Rebase independent window facts onto the horizon's original inventory baseline.
/// Keeping both facts at a shooting node exposes its physical continuity jump.
fn stitch_conservation(
    layout: &native::dynamics::Contract,
    reports: &[native::dynamics::Report],
    integral_offsets: &[Vec<f64>],
    max_cells: usize,
) -> Result<Vec<native::dynamics::ConservationPoint>, ProblemError> {
    let n = layout.balances.len();
    if n == 0 {
        if reports.iter().any(|r| !r.conservation.is_empty()) {
            return Err(ProblemError::internal("shooting conserved subject layout"));
        }
        return Ok(vec![]);
    }
    if reports.is_empty() {
        return Err(ProblemError::internal(
            "shooting conservation window coverage",
        ));
    }
    let extent = reports
        .iter()
        .try_fold(0usize, |count, report| {
            count.checked_add(report.conservation.len())
        })
        .ok_or_else(|| ProblemError::Contract("shooting conservation extent overflow".into()))?;
    let cells = n
        .checked_mul(3)
        .and_then(|width| width.checked_add(layout.quadratures.len()))
        .and_then(|width| width.checked_mul(extent));
    if cells.is_none_or(|cells| cells > max_cells) {
        return Err(ProblemError::Contract(
            "shooting conservation cell allowance".into(),
        ));
    }
    if integral_offsets.len() != reports.len() + 1 {
        return Err(ProblemError::internal(
            "shooting conservation integral offsets",
        ));
    }
    let fluxes = layout
        .balances
        .iter()
        .map(|balance| {
            layout
                .quadratures
                .iter()
                .position(|id| *id == balance.flux)
                .ok_or_else(|| ProblemError::internal("shooting conservation flux identity"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut baseline = None::<Vec<f64>>;
    let mut transfers = vec![0.; n];
    let mut points = Vec::with_capacity(extent);
    for (window, report) in reports.iter().enumerate() {
        if report.conservation.is_empty()
            || integral_offsets[window].len() != layout.quadratures.len()
        {
            return Err(ProblemError::internal(
                "shooting conservation window coverage",
            ));
        }
        for local in &report.conservation {
            if local.mode >= layout.events.len()
                || local.inventories.len() != n
                || local.transfers.len() != n
                || local.defects.len() != n
                || local.integrals.len() != layout.quadratures.len()
            {
                return Err(ProblemError::internal(
                    "shooting conservation subject or mode layout",
                ));
            }
            let initial = baseline.get_or_insert_with(|| local.inventories.clone());
            let mut point = local.clone();
            for (integral, offset) in point.integrals.iter_mut().zip(&integral_offsets[window]) {
                *integral += offset;
            }
            for (transfer, offset) in point.transfers.iter_mut().zip(&transfers) {
                *transfer += offset;
            }
            for (index, flux) in fluxes.iter().enumerate() {
                point.defects[index] = point.inventories[index]
                    - initial[index]
                    - point.integrals[*flux]
                    - point.transfers[index];
            }
            if !point.time.is_finite()
                || point
                    .inventories
                    .iter()
                    .chain(&point.integrals)
                    .chain(&point.transfers)
                    .chain(&point.defects)
                    .any(|v| !v.is_finite())
            {
                return Err(ProblemError::numerical(
                    "nonfinite shooting conservation fact",
                ));
            }
            points.push(point);
        }
        let last = report
            .conservation
            .last()
            .ok_or_else(|| ProblemError::internal("shooting conservation window coverage"))?;
        for (total, local) in transfers.iter_mut().zip(&last.transfers) {
            *total += local;
        }
    }
    Ok(points)
}

#[cfg(test)]
mod conservation_tests {
    use super::*;
    fn window(start: f64, inventory: f64) -> native::dynamics::Report {
        let fact = |time, inventory, integral, transfer| native::dynamics::ConservationPoint {
            time,
            mode: 0,
            inventories: vec![inventory],
            integrals: vec![integral],
            transfers: vec![transfer],
            defects: vec![0.],
        };
        native::dynamics::Report {
            termination: native::dynamics::Termination::Completed,
            completed_time: start + 1.,
            requested_initial: vec![],
            consistent_initial: vec![],
            samples: vec![],
            endpoint: None,
            conservation: vec![
                fact(start, inventory, 0., 0.),
                fact(start + 1., inventory + 3., 1., 2.),
            ],
            events: vec![],
            statistics: vec![],
            error: None,
            progress: vec![],
            dropped_progress: 0,
        }
    }
    fn layout() -> native::dynamics::Contract {
        let flux = pse_ids::named_id(SemanticId::NIL, "flux");
        native::dynamics::Contract {
            identity: ContentHash::from_bytes([0; 32]),
            states: vec![],
            differential: vec![],
            parameters: vec![],
            outputs: vec![],
            events: vec![vec![]],
            quadratures: vec![flux],
            balances: vec![native::dynamics::Balance {
                id: pse_ids::named_id(SemanticId::NIL, "subject"),
                inventory: pse_ids::named_id(SemanticId::NIL, "inventory"),
                flux,
                tolerance: 1e-6,
                transfers: Default::default(),
            }],
            signs: vec![],
            derivatives: DerivativeOrder::First,
        }
    }
    #[test]
    fn conservation_stitch_preserves_flux_transfers_and_exposes_node_jump() {
        let offsets = vec![vec![0.], vec![1.], vec![2.]];
        for (jump, expected) in [(0., 0.), (0.25, 0.25)] {
            let points = stitch_conservation(
                &layout(),
                &[window(0., 2.), window(1., 5. + jump)],
                &offsets,
                100,
            )
            .unwrap();
            assert_eq!(points.len(), 4);
            assert_eq!(points[2].inventories, vec![5. + jump]);
            assert_eq!(points[2].defects, vec![expected]);
            assert_eq!(points[3].integrals, vec![2.]);
            assert_eq!(points[3].transfers, vec![4.]);
            assert_eq!(points[3].defects, vec![expected]);
        }
    }
    #[test]
    fn conservation_stitch_refuses_missing_subject_or_mode_facts_and_unbounded_extent() {
        let offsets = vec![vec![0.], vec![1.]];
        let mut report = window(0., 2.);
        assert!(
            stitch_conservation(&layout(), std::slice::from_ref(&report), &offsets, 0).is_err()
        );
        report.conservation[0].mode = 1;
        assert!(
            stitch_conservation(&layout(), std::slice::from_ref(&report), &offsets, 100).is_err()
        );
        report.conservation[0].mode = 0;
        report.conservation[0].inventories.clear();
        assert!(stitch_conservation(&layout(), &[report], &offsets, 100).is_err());
    }
}

#[cfg(test)]
#[cfg(feature = "solver-ipopt")]
#[path = "shooting_tests.rs"]
mod tests;
