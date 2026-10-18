// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin experiment composition; compiled library derivatives own local mathematics.
use super::*;
use faer::linalg::temp_mat_scratch;
use faer::{
    Mat,
    sparse::{SparseColMat, SymbolicSparseColMatRef},
};
#[cfg(test)]
use native::solve::Backend;
use native::{
    NlpOracle, ProblemError,
    solve::{Compatibility, Execution},
};
use pse_math::{assembly::CaseWorker, sparse::AssemblyMatrix};
/// One upgrade's scientific comparison witness, shared only with its joined lanes.
/// A native/analysis failure alone does not contradict an independently complete
/// lower product. An actual same-premise comparison failure does.
#[derive(Clone, Default)]
pub(super) struct UpgradeValidity(Arc<std::sync::atomic::AtomicBool>);
impl UpgradeValidity {
    #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
    fn contradicts(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }
    fn contradicted(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire)
    }
}
/// Products actually requested at an exact trial point. Native integration still
/// computes the Jacobians its numerical method requires, independently of fit output demand.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Demand {
    pub(super) jacobian: bool,
    pub(super) responses: bool,
    pub(super) adjoint: bool,
    pub(super) purpose: crate::invocation::IntegrationPurpose,
}
impl Demand {
    fn includes(self, other: Self) -> bool {
        (!other.jacobian || self.jacobian)
            && (!other.responses || self.responses)
            && (!other.adjoint || self.adjoint || self.responses)
    }
    fn union(self, other: Self) -> Self {
        Self {
            jacobian: self.jacobian || other.jacobian,
            responses: self.responses || other.responses,
            adjoint: !(self.responses || other.responses) && (self.adjoint || other.adjoint),
            purpose: self.purpose,
        }
    }
    fn responses() -> Self {
        Self {
            // Implicit response assessment consumes F_p from the assembled constraint
            // Jacobian as well as output responses. Request both products together.
            jacobian: true,
            responses: true,
            ..Self::default()
        }
    }
}
#[derive(Debug)]
struct Point {
    achieved: Demand,
    x: Vec<f64>,
    predictions: Vec<f64>,
    responses: AssemblyMatrix,
    constraints: Vec<f64>,
    jacobian: AssemblyMatrix,
    blocks: Vec<Option<SparseColMat<usize, f64>>>,
    trajectories: BTreeMap<InstanceId, Arc<native::dynamics::Report>>,
    /// The transient experiments' adjoint part of the objective gradient, once computed.
    adjoint: Option<Vec<f64>>,
    /// Complete residual curvature only; native objective weights are applied later.
    /// The enclosing fit job admits this storage through `FitProblem::bytes`.
    curvatures: Option<Vec<Option<Vec<f64>>>>,
}
/// Additional numeric residency while the installed complete point and a private
/// replacement coexist. Previous trajectory Arcs already coexist during upgrade;
/// curvature is moved only on success and is never duplicated here.
pub(super) fn replacement_point_bytes(
    experiments: &[Experiment],
    layout: &sparse::Layout,
    variables: usize,
    rows: usize,
    observations: usize,
) -> Option<usize> {
    let mut bytes = size_of::<Point>()
        .checked_add(layout.responses.retained_bytes())?
        .checked_add(layout.constraints.retained_bytes())?
        .checked_add(
            variables
                .checked_mul(2)?
                .checked_add(rows)?
                .checked_add(observations)?
                .checked_mul(size_of::<f64>())?,
        )?
        .checked_add(
            experiments
                .len()
                .checked_mul(size_of::<Option<SparseColMat<usize, f64>>>())?,
        )?;
    for experiment in experiments {
        if let Experiment::Steady(steady) = experiment {
            let pattern = steady.case.assembly.jacobian_pattern();
            bytes = bytes
                .checked_add(size_of_val(pattern.col_ptr()))?
                .checked_add(size_of_val(pattern.row_idx()))?
                .checked_add(pattern.row_idx().len().checked_mul(size_of::<f64>())?)?;
        }
    }
    Some(bytes)
}
#[derive(Debug, Default)]
pub(super) struct ExperimentProducts {
    predictions: Vec<(usize, f64)>,
    constraints: Vec<(usize, f64)>,
    responses: Vec<(pse_math::index::Addend, f64)>,
    jacobian: Vec<(pse_math::index::Addend, f64)>,
    block: Option<SparseColMat<usize, f64>>,
    trajectory: Option<native::dynamics::Report>,
    adjoint: Vec<(usize, f64)>,
}
#[derive(Debug)]
pub(super) struct FitOracle {
    observation: Option<pse_diagnostics::observation::Producer>,
    point_generation: u64,
    prepared: Arc<FitProblem>,
    workers: Vec<Option<CaseWorker>>,
    #[cfg(feature = "solver-diffsol")]
    transient_workers: Vec<Option<crate::workflow::dynamics::DynamicWorker>>,
    execution: Execution,
    team: Option<experiments::ExperimentTeam>,
    /// The derivative source: the prepared profile's, until the final assessment reruns
    /// the forward sensitivities for rank.
    derivatives: FitDerivatives,
    hessian: Option<AssemblyMatrix>,
    gram: Option<sparse::GramWorker>,
    point: Option<Point>,
    #[cfg(all(test, feature = "solver-diffsol"))]
    transient_passes: (usize, usize),
    #[cfg(all(test, feature = "solver-idas"))]
    transient_curvature_passes: usize,
}
struct RankDiagnostic {
    responses: pse_columnar::Leased<Mat<f64>>,
    singular_values: Vec<f64>,
    /// The full right singular basis of the weighted, scaled response matrix.
    directions: Mat<f64>,
    rank: usize,
}
fn error(message: impl Into<String>) -> ProblemError {
    ProblemError::Contract(message.into())
}
impl FitOracle {
    pub(super) fn new(
        p: impl Into<Arc<FitProblem>>,
        execution: Execution,
    ) -> Result<Self, ProblemError> {
        let _phase = pse_diagnostics::observation::phase("fit.oracle_construction");
        let p = p.into();
        let team = experiments::ExperimentTeam::new(p.clone(), execution.clone())?;
        let workers = p
            .experiments
            .iter()
            .map(|e| match e {
                Experiment::Steady(_) if team.is_some() => Ok(None),
                Experiment::Steady(s) => {
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("fit.experiment_worker_constructions", "count", 1);
                    }
                    let providers =
                        crate::math::attempt_providers(&s.providers, &execution.scope()?)
                            .map_err(ProblemError::Provider)?;
                    let mut worker = s.case.assembly.worker_scoped(providers, execution.scope()?);
                    worker.prepare_providers()?;
                    Ok(Some(worker))
                }
                Experiment::Transient(_) => Ok(None),
            })
            .collect::<Result<Vec<_>, ProblemError>>()?;
        #[cfg(feature = "solver-diffsol")]
        let transient_workers = p
            .experiments
            .iter()
            .map(|experiment| match experiment {
                Experiment::Steady(_) => Ok(None),
                Experiment::Transient(_) if team.is_some() => Ok(None),
                Experiment::Transient(s) => {
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("fit.experiment_worker_constructions", "count", 1);
                    }
                    s.worker(&execution).map(Some)
                }
            })
            .collect::<Result<Vec<_>, ProblemError>>()?;
        let hessian = p.layout.hessian.clone();
        let gram = p
            .layout
            .gram
            .as_ref()
            .map(|g| sparse::GramWorker::new(g.clone(), &p.layout.responses, p.bytes))
            .transpose()?;
        Ok(Self {
            observation: pse_diagnostics::observation::current().map(|context| {
                context.producer(
                    pse_diagnostics::observation::ProducerRole::Workflow,
                    "fit.demand.products",
                )
            }),
            point_generation: 0,
            derivatives: p.profile.derivatives,
            prepared: p,
            workers,
            #[cfg(feature = "solver-diffsol")]
            transient_workers,
            execution,
            team,
            hessian,
            gram,
            point: None,
            #[cfg(all(test, feature = "solver-diffsol"))]
            transient_passes: (0, 0),
            #[cfg(all(test, feature = "solver-idas"))]
            transient_curvature_passes: 0,
        })
    }
    fn evaluate(&mut self, x: &[f64]) -> Result<&Point, ProblemError> {
        self.evaluate_demand(
            x,
            Demand {
                jacobian: true,
                responses: true,
                adjoint: false,
                purpose: crate::invocation::IntegrationPurpose::PrimalTrial,
            },
        )
    }
    fn evaluate_demand(&mut self, x: &[f64], mut demand: Demand) -> Result<&Point, ProblemError> {
        let _phase = pse_diagnostics::observation::phase("fit.trial");
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("fit.trial_demands", "count", 1);
            context.boolean("fit.requested_jacobian", demand.jacobian);
            context.boolean("fit.requested_responses", demand.responses);
            context.boolean("fit.requested_adjoint", demand.adjoint);
        }
        if let Some(stop) = self.execution.stopped() {
            self.point = None;
            return Err(ProblemError::stopped(stop, "fit evaluation deadline"));
        }
        let p = &self.prepared;
        let n = p.contract.variables.len();
        if x.len() != n || x.iter().any(|v| !v.is_finite()) {
            self.point = None;
            return Err(ProblemError::numerical("fit trial coordinates"));
        }
        let same = self
            .point
            .as_ref()
            .is_some_and(|v| v.x.iter().zip(x).all(|(a, b)| a.to_bits() == b.to_bits()));
        if !same {
            self.point_generation = self.point_generation.saturating_add(1);
        }
        if let Some(producer) = &self.observation {
            use pse_diagnostics::observation::{RecordField, Value};
            let cached = same
                && self
                    .point
                    .as_ref()
                    .is_some_and(|p| p.achieved.includes(demand));
            let record = producer.record(
                "workflow.fit_demand.v1",
                if cached { "cache_reuse" } else { "requested" },
                Some("exact original parameter vector"),
                None,
                vec![
                    RecordField {
                        name: "point_generation".into(),
                        unit: "oracle-local point".into(),
                        value: if self.point_generation == u64::MAX {
                            Value::Overflow
                        } else {
                            Value::Count(self.point_generation)
                        },
                        availability: None,
                    },
                    RecordField {
                        name: "purpose".into(),
                        unit: "workflow reason".into(),
                        value: Value::Text(demand.purpose.as_str().into()),
                        availability: None,
                    },
                    RecordField {
                        name: "same_point".into(),
                        unit: "exact coordinate bits".into(),
                        value: Value::Boolean(same),
                        availability: None,
                    },
                    RecordField {
                        name: "response_jacobian".into(),
                        unit: "requested product".into(),
                        value: Value::Boolean(demand.responses),
                        availability: None,
                    },
                    RecordField {
                        name: "adjoint_gradient".into(),
                        unit: "requested product".into(),
                        value: Value::Boolean(demand.adjoint),
                        availability: None,
                    },
                ],
            );
            producer.publish(&record);
        }
        if same
            && self
                .point
                .as_ref()
                .is_some_and(|p| p.achieved.includes(demand))
        {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("fit.trial_cache_reuses", "count", 1);
            }
            return self
                .point
                .as_ref()
                .ok_or_else(|| ProblemError::internal("fit point cache"));
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(
                if same {
                    "fit.derivative_upgrades"
                } else {
                    "fit.new_trials"
                },
                "count",
                1,
            );
        }
        if same
            && matches!(
                demand.purpose,
                crate::invocation::IntegrationPurpose::PrimalTrial
            )
        {
            demand.purpose = crate::invocation::IntegrationPurpose::DerivativeUpgrade;
        }
        if same && let Some(point) = &self.point {
            demand = demand.union(point.achieved);
        }
        // Keep the complete lower product installed until every replacement product
        // succeeds. Trajectory Arcs are immutable; the comparison witness invalidates
        // the lower product only when new scientific evidence contradicts it.
        if !same {
            self.point = None;
        }
        let validity = UpgradeValidity::default();
        let previous = self.point.as_ref().map(|point| point.trajectories.clone());
        let mut point = Point {
            achieved: demand,
            x: x.to_vec(),
            predictions: vec![0.0; p.measurements.len()],
            responses: p.layout.responses.clone(),
            constraints: vec![0.0; p.contract.rows.len()],
            jacobian: p.layout.constraints.clone(),
            blocks: Vec::new(),
            trajectories: BTreeMap::new(),
            adjoint: demand.adjoint.then(|| vec![0.; x.len()]),
            curvatures: None,
        };
        #[cfg(all(test, feature = "solver-diffsol"))]
        for (ei, experiment) in p.experiments.iter().enumerate() {
            if matches!(experiment, Experiment::Transient(_))
                && !p.layout.mappings[ei].observations.is_empty()
            {
                self.transient_passes.0 += 1;
                self.transient_passes.1 +=
                    usize::from(demand.adjoint && !p.layout.mappings[ei].free_bindings.is_empty());
            }
        }
        let products = if let Some(team) = &mut self.team {
            team.evaluate(x, demand, previous.as_ref(), &validity)
        } else {
            p.experiments
                .iter()
                .enumerate()
                .map(|(ei, _)| {
                    evaluate_experiment(
                        p,
                        ei,
                        x,
                        demand,
                        &self.execution,
                        ExperimentWorkers {
                            steady: &mut self.workers[ei],
                            #[cfg(feature = "solver-diffsol")]
                            transient: &mut self.transient_workers[ei],
                        },
                        previous
                            .as_ref()
                            .and_then(|v| v.get(&p.declaration.experiments[ei].experiment_id))
                            .map(Arc::as_ref),
                        &validity,
                    )
                })
                .collect::<Result<Vec<_>, ProblemError>>()
        };
        let products = match products {
            Ok(products) => products,
            Err(error) => {
                if validity.contradicted() {
                    self.point = None;
                }
                return Err(error);
            }
        };
        // Publication is atomic; reduction order is the authored experiment declaration
        // order, independent of lane scheduling and completion order.
        for (ei, product) in products.into_iter().enumerate() {
            for (i, value) in product.predictions {
                point.predictions[i] = value;
            }
            for (i, value) in product.constraints {
                point.constraints[i] = value;
            }
            for (slot, value) in product.responses {
                point.responses.add(slot, value)?;
            }
            for (slot, value) in product.jacobian {
                point.jacobian.add(slot, value)?;
            }
            if let Some(total) = &mut point.adjoint {
                for (i, value) in product.adjoint {
                    total[i] += value;
                }
            }
            point.blocks.push(product.block);
            if let Some(report) = product.trajectory {
                point.trajectories.insert(
                    p.declaration.experiments[ei].experiment_id,
                    Arc::new(report),
                );
            }
        }
        if point
            .adjoint
            .as_ref()
            .is_some_and(|v| v.iter().any(|v| !v.is_finite()))
        {
            return Err(ProblemError::numerical(
                "nonfinite fit adjoint contribution",
            ));
        }
        if point
            .predictions
            .iter()
            .chain(&point.constraints)
            .any(|v| !v.is_finite())
            || point
                .responses
                .matrix()
                .val()
                .iter()
                .any(|v| !v.is_finite())
        {
            return Err(ProblemError::numerical("nonfinite fit output or response"));
        }
        self.execution.progress.push(native::solve::Event {
            source: Some(
                native::solve::EventSource::new("fitting_trial", None, None)
                    .with_role(pse_diagnostics::observation::ProducerRole::Workflow),
            ),
            phase: "fit.evaluation".into(),
            elapsed: self.execution.started.elapsed(),
            values: BTreeMap::from([(
                "experiments".into(),
                native::solve::Metric::Integer(p.experiments.len() as i64),
            )]),
            incumbent: None,
        });
        // A complete curvature block is independent of the optimizer's subsequent
        // objective multiplier and retains the same immutable exact-point premises.
        point.curvatures = self
            .point
            .as_mut()
            .and_then(|previous| previous.curvatures.take());
        self.point = Some(point);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("fit.trials_completed", "count", 1);
        }
        self.point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("fit point publication"))
    }
    /// A fit parameter's value: the trial coordinate when free, the declared value when
    /// fixed.
    #[cfg(feature = "solver-diffsol")]
    fn value(p: &FitProblem, x: &[f64], parameter: usize) -> f64 {
        p.parameter_columns[parameter]
            .map_or(p.declaration.parameters[parameter].value, |c| x[c.get()])
    }
    #[cfg(feature = "solver-diffsol")]
    fn admit_upgrade(before: f64, after: f64, allowance: f64) -> Result<(), ProblemError> {
        if !before.is_finite()
            || !after.is_finite()
            || !allowance.is_finite()
            || allowance < 0.
            || (before - after).abs() > allowance
        {
            return Err(ProblemError::numerical(
                "fit checkpointed prediction differs beyond its physical allowance",
            ));
        }
        Ok(())
    }
    /// One transient experiment's gradient contributions from one forward and one backward
    /// pass, whose cotangent is the weighted residual of each included observation.
    #[cfg(feature = "solver-diffsol")]
    fn transient_gradient(
        p: &FitProblem,
        ei: usize,
        s: &IntegratedExperiment,
        x: &[f64],
        execution: &Execution,
        worker: &mut crate::workflow::dynamics::DynamicWorker,
    ) -> Result<(native::dynamics::Report, Vec<(usize, f64)>), ProblemError> {
        let outputs = s.program.contract.outputs.len();
        let mut cotangent =
            |report: &native::dynamics::Report| Self::cotangent(p, ei, outputs, report);
        s.gradient(
            worker,
            &|k| Self::value(p, x, k),
            execution,
            &p.layout.mappings[ei].free_bindings,
            &mut cotangent,
        )
    }
    #[cfg(feature = "solver-diffsol")]
    /// The cotangent of the fit objective's transient part: each included observation's
    /// weighted residual `rᵢwᵢ` at its sample and output, from the integration's report.
    fn cotangent(
        p: &FitProblem,
        ei: usize,
        outputs: usize,
        report: &native::dynamics::Report,
    ) -> Result<Vec<f64>, ProblemError> {
        let mut weights = vec![0.0; report.samples.len() * outputs];
        for &i in &p.layout.mappings[ei].observations {
            let o = &p.measurements[i];
            let index = o
                .sample_index
                .ok_or_else(|| ProblemError::internal("missing prepared transient sample"))?;
            let prediction = report
                .samples
                .get(index)
                .and_then(|sample| sample.outputs.get(o.row))
                .ok_or_else(|| ProblemError::internal("missing adjoint transient sample"))?;
            let (r, w) = Self::residual(o, *prediction)?;
            weights[index * outputs + o.row] += r * w;
        }
        Ok(weights)
    }
    /// Every transient experiment's curvature block `Σ rᵢwᵢ∇²yᵢ` over its free parameters'
    /// bindings, as the dense block the layout's transient Hessian sources index; absent
    /// for steady experiments and for transient ones without an included observation or
    /// a free parameter.
    fn transient_curvatures(&mut self, x: &[f64]) -> Result<(), ProblemError> {
        crate::workflow::dynamics::analysis("fit.exact_transient_curvature", || {
            self.execution.check()?;
            let point = self
                .point
                .as_ref()
                .ok_or_else(|| ProblemError::internal("fit curvature point"))?;
            if point.curvatures.is_some() {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("fit.transient_curvature_reuses", "count", 1);
                }
                return Ok(());
            }
            let _phase = pse_diagnostics::observation::phase("fit.transient_curvature");
            if let Some(context) = pse_diagnostics::observation::current() {
                context.text(
                    "integration.purpose",
                    crate::invocation::IntegrationPurpose::Curvature.as_str(),
                );
            }
            let p = self.prepared.clone();
            let mut curvatures = Vec::with_capacity(p.experiments.len());
            for (ei, experiment) in p.experiments.iter().enumerate() {
                let mapping = &p.layout.mappings[ei];
                let curvature = match experiment {
                    Experiment::Transient(s)
                        if !mapping.free_bindings.is_empty()
                            && !mapping.observations.is_empty() =>
                    {
                        Some(self.transient_curvature(&p, ei, s, x, &mapping.free_bindings)?)
                    }
                    _ => None,
                };
                curvatures.push(curvature);
            }
            self.execution.check()?;
            if curvatures
                .iter()
                .flatten()
                .flatten()
                .any(|v| !v.is_finite())
            {
                return Err(ProblemError::numerical("nonfinite fit transient curvature"));
            }
            // A failed/cancelled experiment drops all temporary blocks. Advertise a
            // completed product only after every required experiment has succeeded.
            self.point
                .as_mut()
                .ok_or_else(|| ProblemError::internal("fit curvature publication"))?
                .curvatures = Some(curvatures);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("fit.transient_curvature_completions", "count", 1);
            }
            Ok(())
        })
    }
    #[cfg(feature = "solver-idas")]
    fn transient_curvature(
        &mut self,
        p: &FitProblem,
        ei: usize,
        s: &IntegratedExperiment,
        x: &[f64],
        free: &[usize],
    ) -> Result<Vec<f64>, ProblemError> {
        #[cfg(test)]
        {
            self.transient_curvature_passes += 1;
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("fit.transient_curvature_integrations", "count", 1);
        }
        if self.transient_workers[ei].is_none() {
            self.transient_workers[ei] = Some(s.worker(&self.execution)?);
        }
        let outputs = s.program.contract.outputs.len();
        let mut cotangent =
            |report: &native::dynamics::Report| Self::cotangent(p, ei, outputs, report);
        let producer = self.observation.clone();
        let observation = crate::invocation::IntegrationObservation::begin(
            producer,
            crate::invocation::IntegrationPurpose::Curvature,
            false,
            true,
            true,
            ei as u64,
            None,
        );
        let result = s
            .hessian(
                self.transient_workers[ei]
                    .as_mut()
                    .ok_or_else(|| ProblemError::internal("missing transient worker"))?,
                &|k| Self::value(p, x, k),
                &self.execution,
                free,
                &mut cotangent,
            )
            .map(|(_, _, curvature)| curvature);
        observation.finish(result.is_ok());
        result
    }
    #[cfg(not(feature = "solver-idas"))]
    fn transient_curvature(
        &mut self,
        _: &FitProblem,
        _: usize,
        _: &IntegratedExperiment,
        _: &[f64],
        _: &[usize],
    ) -> Result<Vec<f64>, ProblemError> {
        Err(ProblemError::unsupported("IDAS not linked"))
    }
    fn residual(o: &Measurement, pred: f64) -> Result<(f64, f64), ProblemError> {
        let sigma = o.sigma.ok_or_else(|| error("missing standard deviation"))?;
        let r = (pred - o.value.ok_or_else(|| error("missing observation"))?) / sigma;
        let w = o.importance.sqrt() / sigma;
        Ok((r * o.importance.sqrt(), w))
    }
}
impl NlpOracle for FitOracle {
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.prepared.normalization)
    }
    fn contract(&self) -> &OracleContract {
        &self.prepared.contract
    }
    fn jacobian_pattern(&self) -> SymbolicSparseColMatRef<'_, usize> {
        self.prepared.layout.constraints.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<SymbolicSparseColMatRef<'_, usize>> {
        self.hessian.as_ref().map(|p| p.matrix().symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.prepared.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.evaluate_demand(x, Demand::default())?;
        let point = self
            .point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("fit point"))?;
        let residuals = self
            .prepared
            .measurements
            .iter()
            .zip(&point.predictions)
            .filter(|(o, _)| o.included)
            .map(|(o, p)| Self::residual(o, *p).map(|v| v.0))
            .collect::<Result<Vec<_>, _>>()?;
        let norm = faer::col::ColRef::from_slice(&residuals).squared_norm_l2();
        let sum = 0.5 * norm;
        if !sum.is_finite() {
            return Err(ProblemError::numerical("fit objective overflow"));
        }
        Ok(sum)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let p = self.evaluate_demand(x, Demand::default())?;
        if out.len() != p.constraints.len() {
            return Err(ProblemError::internal("fit constraint extent"));
        }
        out.copy_from_slice(&p.constraints);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.evaluate_demand(
            x,
            match self.derivatives {
                FitDerivatives::Responses => Demand::responses(),
                FitDerivatives::Gradient => Demand {
                    adjoint: true,
                    ..Demand::default()
                },
            },
        )?;
        if out.len() != x.len() {
            return Err(ProblemError::internal("fit gradient extent"));
        }
        let point = self
            .point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("fit point"))?;
        let weights = self
            .prepared
            .measurements
            .iter()
            .zip(&point.predictions)
            .map(|(o, p)| {
                if o.included {
                    Self::residual(o, *p).map(|(r, w)| r * w)
                } else {
                    Ok(0.0)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let rhs = Mat::from_fn(weights.len(), 1, |i, _| weights[i]);
        let mut result = Mat::zeros(x.len(), 1);
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            result.as_mut(),
            faer::Accum::Replace,
            point.responses.matrix().as_ref().transpose(),
            rhs.as_ref(),
            1.0,
            faer::Par::Seq,
        );
        // A gradient-only fit's transient rows carry no responses; their part is the
        // adjoint product.
        let adjoint = if self.derivatives == FitDerivatives::Gradient {
            point.adjoint.clone().unwrap_or_else(|| vec![0.0; x.len()])
        } else {
            vec![0.0; x.len()]
        };
        for (j, v) in out.iter_mut().enumerate() {
            let value = result[(j, 0)] + adjoint[j];
            if !value.is_finite() {
                return Err(ProblemError::numerical("nonfinite loss gradient"));
            }
            *v = value;
        }
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.evaluate_demand(
            x,
            Demand {
                jacobian: true,
                ..Demand::default()
            },
        )?;
        let values = self
            .point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("fit point"))?
            .jacobian
            .matrix()
            .val();
        if out.len() != values.len() {
            return Err(ProblemError::internal("fit sparse Jacobian extent"));
        }
        out.copy_from_slice(values);
        Ok(())
    }

    fn hessian(
        &mut self,
        x: &[f64],
        objective_weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.evaluate(x)?;
        if multipliers.len() != self.prepared.bounds.len()
            || !objective_weight.is_finite()
            || multipliers.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::internal("fit Lagrangian demand"));
        }
        let exact = match self.prepared.profile.solver.controls.hessian {
            HessianMode::Exact => true,
            HessianMode::GaussNewton => false,
            HessianMode::Auto
            | HessianMode::LimitedMemory
            | HessianMode::Partitioned
            | HessianMode::FiniteDifference => {
                return Err(ProblemError::Contract("fitting does not supply Hessians for request-only or native-approximated curvature modes".into()));
            }
        };
        // The exact Hessian's transient curvature Σ rᵢwᵢ∇²yᵢ, by second-order adjoint
        // sensitivities of each transient experiment (ADR-0110 item 4).
        if exact {
            self.transient_curvatures(x)?;
        }
        let p = &self.prepared;
        let point = self
            .point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("fit point"))?;
        let h = self
            .hessian
            .as_mut()
            .ok_or_else(|| ProblemError::internal("Hessian not prepared"))?;
        h.clear();
        let weights = p
            .measurements
            .iter()
            .zip(&point.predictions)
            .map(|(o, p)| {
                if o.included {
                    Self::residual(o, *p).map(|v| v.1)
                } else {
                    Ok(0.0)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        // The weighted response Gram JᵀWJ is the Gauss–Newton part of every supplied
        // Hessian; the exact Hessian adds the residual curvature Σ rᵢwᵢ∇²yᵢ.
        self.gram
            .as_mut()
            .ok_or_else(|| ProblemError::internal("Gram not prepared"))?
            .refill(&point.responses, &weights, objective_weight, h)?;
        for (ei, e) in p.experiments.iter().enumerate() {
            let s = match e {
                Experiment::Steady(s) => s,
                // A transient experiment adds no constraint rows: its Gauss–Newton part is
                // the Gram term alone, and the exact Hessian adds its curvature block.
                Experiment::Transient(_) => {
                    if exact
                        && let Some(curvature) = point
                            .curvatures
                            .as_ref()
                            .and_then(|blocks| blocks[ei].as_ref())
                    {
                        for &(source, target) in &p.layout.mappings[ei].hessian {
                            h.add(target, objective_weight * curvature[source])?;
                        }
                    }
                    continue;
                }
            };
            let mut lambda = vec![0.0; s.case.assembly.structure().rows().len()];
            for &(local, global) in &s.constraints {
                lambda[local.get()] += multipliers[global.get()];
            }
            if exact {
                for &i in &p.layout.mappings[ei].observations {
                    let o = &p.measurements[i];
                    let (r, w) = Self::residual(o, point.predictions[i])?;
                    lambda[o.row] += objective_weight * r * w;
                }
            } else if lambda.iter().all(|v| *v == 0.0) {
                // No constraint curvature enters the Gauss–Newton Hessian.
                continue;
            }
            let mut values = s.values.clone();
            for &(id, c) in &s.coordinates {
                values.scalars.insert(id, x[c.get()]);
            }
            if self.workers[ei].is_none() {
                let providers =
                    crate::math::attempt_providers(&s.providers, &self.execution.scope()?)
                        .map_err(ProblemError::Provider)?;
                let mut worker = s.case.assembly.worker_scoped(providers, self.execution.scope()?);
                worker.prepare_providers()?;
                self.workers[ei] = Some(worker);
            }
            let local = self.workers[ei]
                .as_mut()
                .ok_or_else(|| ProblemError::internal("steady Hessian worker"))?
                .hessian(&values, 0.0, &lambda)?;
            for &(source, target) in &p.layout.mappings[ei].hessian {
                h.add(target, local.val()[source])?;
            }
        }
        if out.len() != h.matrix().val().len() {
            return Err(ProblemError::internal("fit sparse Hessian extent"));
        }
        out.copy_from_slice(h.matrix().val());
        Ok(())
    }
}
impl FitProblem {
    /// Run the fit, then derive its covariance and requested intervals, solving up to
    /// `workers` profile chains at once.
    pub(crate) fn execute_profile(
        self: &Arc<Self>,
        solver: &SolverProfile,
        route: native::routing::Route,
        scope: pse_kernels::ExecutionScope,
        progress: Arc<native::solve::Progress>,
        workers: usize,
        admission: Option<Arc<crate::math::strategy::admission::TaskAdmission>>,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let operation = pse_diagnostics::observation::current().map(|parent| {
            parent.child(
                "fit.execution",
                pse_diagnostics::observation::Relationship::Nested,
            )
        });
        let context = operation.as_ref().map(|operation| operation.context());
        if let Some(context) = &context {
            context.declare_summary("fit responsibility");
        }
        let _entered = context.as_ref().map(|context| context.enter());
        let result = (|| {
            let adapters: Vec<&dyn native::execution::BackendExecution> = match route {
                native::routing::Route::Native(backend) => {
                    vec![native::execution::adapter(backend)]
                }
                native::routing::Route::Constant => vec![],
            };
            let _phase = pse_diagnostics::observation::phase("fit.execute");
            let observation = pse_diagnostics::observation::current();
            let (mut report, execution) = native::execution::scoped(
                &adapters,
                solver.controls.threads,
                self.runtime.native().stack_bytes(),
                || {
                    let _entered = observation.as_ref().map(|context| context.enter());
                    self.execute_inner(solver, route, scope, progress, admission)
                },
            )?;
            // The estimation adapter team and native retained state have joined before
            // profile admission releases the coordinator's active allowance.
            native::execution::original_scoped(&execution, || {
                self.derive(&mut report, route, &execution, workers);
            });
            observe_report(&report);
            Ok(report)
        })();
        if let Some(operation) = operation {
            operation.finish(match &result {
                Ok(_) => pse_diagnostics::observation::Outcome::Completed,
                Err(crate::math::MathRuntimeError::Cancelled) => {
                    pse_diagnostics::observation::Outcome::Cancelled
                }
                Err(_) => pse_diagnostics::observation::Outcome::Failed,
            });
        }
        result
    }
    fn execute_inner(
        self: &Arc<Self>,
        solver: &SolverProfile,
        route: native::routing::Route,
        scope: pse_kernels::ExecutionScope,
        progress: Arc<native::solve::Progress>,
        admission: Option<Arc<crate::math::strategy::admission::TaskAdmission>>,
    ) -> Result<(FitReport, Execution), crate::math::MathRuntimeError> {
        let mut execution = Execution::within(
            scope.cancellation_control().clone(),
            &solver.controls,
            scope,
        )?;
        execution.progress = progress;
        execution.work_admission =
            admission.map(|admission| -> Arc<dyn native::solve::WorkAdmission> { admission });
        execution.memory = Some(
            self.runtime
                .shared
                .budget()
                .math
                .foreign_allowance(&solver.controls),
        );
        let mut oracle = FitOracle::new(self.clone(), execution.clone())?;
        let (solve, candidate) = if self.initial.is_empty() {
            oracle.objective(&[])?;
            (None, Some(vec![]))
        } else {
            let native::routing::Route::Native(backend) = route else {
                return Err(ProblemError::unsupported("nonempty fit has no native route").into());
            };
            // The one NLP runner, shared with solve sequences and initialization.
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
                        layout: self.key,
                        profile: crate::math::solves::profile_key(solver)?.as_id(),
                        data: self.source_identity,
                        backend,
                    },
                    warm: None,
                },
                &mut native::execution::Retained::default(),
                native::execution::Nlp {
                    oracle: Box::new(oracle),
                    initial: &self.initial,
                    presolve: &solver.presolve,
                    intent: solver.intent,
                    sense: pse_math::binding::ObjectiveSense::Minimize,
                    limit: self.profile.max_cells,
                    // An exact-Hessian fit reads its covariance from its own KKT analysis:
                    // the inverse reduced Hessian over its parameter columns (ADR-0118 item 8).
                    analysis: native::execution::Analysis {
                        inverse_reduced_hessian: (solver.controls.hessian == HessianMode::Exact)
                            .then(|| self.free().map(|(_, col)| col).collect::<Vec<_>>())
                            .filter(|columns| !columns.is_empty()),
                        ..native::execution::Analysis::for_intent(solver.intent)
                    },
                },
            )?;
            let candidate = report.candidate.as_ref().map(|c| c.primal.clone());
            (Some(report), candidate)
        };
        let mut report = FitReport {
            completion: None,
            strategy: None,
            checks: vec![],
            reports: vec![],
            checks_complete: false,
            validation_error: None,
            solve,
            hessian: solver.controls.hessian,
            derivatives: self.profile.derivatives,
            candidate,
            quality: None,
            constraint_values: vec![],
            objective: None,
            predictions: vec![None; self.measurements.len()],
            trajectories: BTreeMap::new(),
            responses: None,
            singular_values: vec![],
            directions: None,
            rank: None,
            diagnostic: None,
            covariance: None,
            wald: None,
            profiles: None,
        };
        self.observe(&mut report, execution.clone())?;
        Ok((report, execution))
    }
    /// The free parameters: each one's declaration index and fit column, in fit order.
    pub(super) fn free(&self) -> impl Iterator<Item = (usize, OriginalCol)> + '_ {
        self.parameter_columns
            .iter()
            .enumerate()
            .filter_map(|(k, c)| c.map(|c| (k, c)))
    }
    /// The fresh final evaluation of the candidate, independent of the native callback
    /// cache and candidate status, and the local response diagnostic.
    fn observe(
        self: &Arc<Self>,
        report: &mut FitReport,
        execution: Execution,
    ) -> Result<(), crate::math::MathRuntimeError> {
        let Some(x) = report.candidate.clone() else {
            return Ok(());
        };
        let interval = crate::invocation::stage("fit.original_final_assessment");
        let result: Result<(), crate::math::MathRuntimeError> = (|| {
            let _phase = pse_diagnostics::observation::phase("fit.original_final_assessment");
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("fit.original_final_assessments", "count", 1);
            }
            let x = x.as_slice();
            // One fresh final oracle remains independent of optimizer callback state.
            // Known optional refusal precedes any response IVP; actual rank memory is
            // acquired only after integration so it does not inflate the IVP peak.
            let rank = rank_admission(self);
            let responses = rank.is_ok();
            let mut final_oracle = FitOracle::new(self.clone(), execution)?;
            let demand = Demand {
                purpose: crate::invocation::IntegrationPurpose::OriginalAssessment,
                ..if responses {
                    Demand::responses()
                } else {
                    Demand::default()
                }
            };
            let mut derivative_failure = None;
            let evaluated = match final_oracle.evaluate_demand(x, demand).map(|_| ()) {
                Err(error) if responses && final_oracle.execution.stopped().is_none() => {
                    derivative_failure = Some(error);
                    final_oracle
                        .evaluate_demand(
                            x,
                            Demand {
                                purpose: crate::invocation::IntegrationPurpose::OriginalAssessment,
                                ..Demand::default()
                            },
                        )
                        .map(|_| ())
                }
                evaluated => evaluated,
            };
            match evaluated {
                Err(e) => report.diagnostic = Some(FitDiagnostic::new(FitRule::FinalEvaluation, e)),
                Ok(()) => {
                    let point = final_oracle
                        .point
                        .as_ref()
                        .ok_or_else(|| ProblemError::internal("final fit point absent"))?;
                    if !self.install_final_point(report, x, point)? {
                        return Ok(());
                    }
                    if let Some(error) = derivative_failure {
                        report.diagnostic = Some(FitDiagnostic::new(FitRule::ResponseRank, error));
                        return Ok(());
                    }
                    match rank {
                        Err(error) => {
                            report.diagnostic =
                                Some(FitDiagnostic::new(FitRule::ResponseRank, error))
                        }
                        Ok(plan) => match final_oracle.response_rank_admitted(x, plan) {
                            Ok(diagnostic) => {
                                report.responses = Some(diagnostic.responses);
                                report.singular_values = diagnostic.singular_values;
                                report.directions = Some(diagnostic.directions);
                                report.rank = Some(diagnostic.rank);
                            }
                            Err(error) => {
                                report.diagnostic =
                                    Some(FitDiagnostic::new(FitRule::ResponseRank, error))
                            }
                        },
                    }
                }
            }
            Ok(())
        })();
        interval.finish(
            if result.is_ok()
                && report.objective.is_some()
                && !report.diagnostic.as_ref().is_some_and(|diagnostic| {
                    matches!(
                        diagnostic.rule,
                        FitRule::FinalEvaluation | FitRule::ObjectiveOverflow
                    )
                })
            {
                crate::invocation::InvocationOutcome::Completed
            } else {
                crate::invocation::InvocationOutcome::Failed
            },
        );
        result
    }
    /// Values and derivative products refer to the same physically compared trial.
    /// Preserve a completed value observation when rank/derivatives are unavailable.
    fn install_final_point(
        &self,
        report: &mut FitReport,
        x: &[f64],
        point: &Point,
    ) -> Result<bool, crate::math::MathRuntimeError> {
        let residuals = self
            .measurements
            .iter()
            .zip(&point.predictions)
            .filter(|(o, _)| o.included)
            .map(|(o, v)| FitOracle::residual(o, *v).map(|v| v.0))
            .collect::<Result<Vec<_>, _>>()?;
        let objective = 0.5 * faer::col::ColRef::from_slice(&residuals).squared_norm_l2();
        if !objective.is_finite() {
            report.diagnostic = Some(FitDiagnostic::new(
                FitRule::ObjectiveOverflow,
                ProblemError::numerical("fresh fitting objective overflow"),
            ));
            return Ok(false);
        }
        let quality = native::quality::observed(
            &self.contract,
            &self.bounds,
            x,
            &point.constraints,
            &self.tolerances,
        )?;
        let predictions = point
            .predictions
            .iter()
            .zip(&self.measurements)
            .map(|(v, o)| o.included.then_some(*v))
            .collect();
        report.trajectories = point.trajectories.clone();
        report.constraint_values = point.constraints.clone();
        report.objective = Some(objective);
        report.quality = Some(quality);
        report.predictions = predictions;
        Ok(true)
    }
}
/// The singular values of `a`, decreasing, and on request its full right singular basis:
/// column `k` belongs to singular value `k`, and the columns beyond `min(rows, cols)` span
/// the null space.
fn decompose(
    a: &Mat<f64>,
    limit: usize,
    vectors: bool,
) -> Result<(Vec<f64>, Option<Mat<f64>>), ProblemError> {
    if a.as_ref()
        .col_iter()
        .any(|c| c.iter().any(|v| !v.is_finite()))
    {
        return Err(ProblemError::numerical("nonfinite local rank input"));
    }
    use faer::{
        Par,
        diag::Diag,
        dyn_stack::{MemBuffer, MemStack},
        linalg::svd,
    };
    let req = rank_scratch(a.nrows(), a.ncols(), vectors);
    if req.size_bytes() > limit {
        return Err(ProblemError::memory("rank scratch allowance"));
    }
    let mut buffer = MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
    let mut s = Diag::<f64>::zeros(a.nrows().min(a.ncols()));
    let mut v = vectors.then(|| Mat::<f64>::zeros(a.ncols(), a.ncols()));
    svd::svd(
        a.as_ref(),
        s.as_mut(),
        None,
        v.as_mut().map(Mat::as_mut),
        Par::Seq,
        MemStack::new(&mut buffer),
        Default::default(),
    )
    .map_err(|e| ProblemError::numerical(format!("{e:?}")))?;
    if v.as_ref()
        .is_some_and(|v| v.col_iter().any(|c| c.iter().any(|x| !x.is_finite())))
    {
        return Err(ProblemError::numerical("nonfinite right singular vectors"));
    }
    Ok((s.column_vector().iter().copied().collect(), v))
}
fn rank_scratch(rows: usize, cols: usize, vectors: bool) -> faer::dyn_stack::StackReq {
    use faer::linalg::svd::{self, ComputeSvdVectors};
    svd::svd_scratch::<f64>(
        rows,
        cols,
        ComputeSvdVectors::No,
        if vectors {
            ComputeSvdVectors::Full
        } else {
            ComputeSvdVectors::No
        },
        faer::Par::Seq,
        Default::default(),
    )
}
/// Deterministic optional-product admission, without holding matrix memory over an IVP.
struct RankAdmission {
    rows: usize,
    np: usize,
    bytes: usize,
}
fn rank_admission(p: &FitProblem) -> Result<RankAdmission, ProblemError> {
    let np = p.parameter_columns.iter().filter(|v| v.is_some()).count();
    let local_dense = p
        .experiments
        .iter()
        .try_fold(0usize, |cells, experiment| {
            let Experiment::Steady(s) = experiment else {
                return Some(cells);
            };
            cells.checked_add(
                s.local_states
                    .checked_mul(s.local_states.checked_add(np)?)?,
            )
        })
        .and_then(|cells| cells.checked_add(p.measurements.len().checked_mul(np)?))
        .ok_or_else(|| ProblemError::memory("local response diagnostic extent"))?;
    if local_dense > p.profile.max_cells {
        return Err(ProblemError::memory(
            "local dense response diagnostic allowance; fit candidate remains available",
        ));
    }
    // Dense diagnostics are optional: reserve independently from the sparse
    // solve, before allocating any matrix. Failure does not lose the candidate.
    let limit = p.runtime.shared.budget().math.worker_bytes;
    if local_dense.checked_mul(8).is_none_or(|bytes| bytes > limit) {
        return Err(ProblemError::memory(
            "local dense response diagnostic memory allowance",
        ));
    }
    use faer::linalg::temp_mat_scratch;
    let rows = p.measurements.len();
    // The response and its weighted copy, and the right singular basis.
    let mut bytes = temp_mat_scratch::<f64>(rows, np)
        .size_bytes()
        .checked_mul(2)
        .and_then(|b| b.checked_add(temp_mat_scratch::<f64>(np, np).size_bytes()));
    let mut scratch = rank_scratch(rows, np, true).size_bytes();
    for experiment in &p.experiments {
        if let Experiment::Steady(s) = experiment {
            let n = s.local_states;
            // Shared response owns its peak matrix, rank, identities and scratch
            // accounting. Matching's separate thread stack is reserved below;
            // worker_bytes bounds numeric capacity, not structural recursion.
            bytes = bytes
                .and_then(|b| b.checked_add(native::square_response::workspace_bytes(n, np)?))
                .and_then(|b| b.checked_add(n.checked_add(np)?.checked_mul(256)?))
                .and_then(|b| b.checked_add(n.checked_mul(n)?.checked_mul(128)?))
                .and_then(|b| b.checked_add(n.checked_mul(np)?.checked_mul(64)?));
            scratch = scratch.max(rank_scratch(n, n, false).size_bytes());
        }
    }
    // Vector bookkeeping and opaque library metadata, separate from numeric
    // matrices and library-declared scratch; this is not an RSS measurement.
    let bytes = bytes
        .and_then(|b| b.checked_add(scratch))
        .and_then(|b| b.checked_add(rows.checked_add(np)?.checked_mul(64)?))
        .and_then(|b| b.checked_add(4 << 20))
        .filter(|b| *b <= limit)
        .ok_or_else(|| ProblemError::memory("local dense response diagnostic memory allowance"))?;
    // Structural analyses run sequentially, so one separately declared matching
    // stack covers every steady closure without multiplying its peak by experiment.
    let matching_stack = if p
        .experiments
        .iter()
        .any(|e| matches!(e,Experiment::Steady(s) if s.local_states>0))
    {
        pse_structural::incidence::MATCHING_STACK
    } else {
        0
    };
    let bytes = bytes
        .checked_add(matching_stack)
        .ok_or_else(|| ProblemError::memory("response structural stack extent"))?;
    Ok(RankAdmission { rows, np, bytes })
}
impl FitOracle {
    #[cfg(test)]
    fn response_rank(&mut self, x: &[f64]) -> Result<RankDiagnostic, ProblemError> {
        let plan = rank_admission(&self.prepared)?;
        self.response_rank_admitted(x, plan)
    }
    fn response_rank_admitted(
        &mut self,
        x: &[f64],
        plan: RankAdmission,
    ) -> Result<RankDiagnostic, ProblemError> {
        crate::workflow::dynamics::analysis("fit.forward_response_and_identifiability", || {
            // Rank and responses need the response Jacobian: a gradient-only fit reruns the
            // forward sensitivities once, at the candidate (PS-12).
            if self.derivatives != FitDerivatives::Responses {
                self.derivatives = FitDerivatives::Responses;
            }
            self.evaluate_demand(
                x,
                Demand {
                    purpose: crate::invocation::IntegrationPurpose::Identifiability,
                    ..Demand::responses()
                },
            )?;
            let p = &self.prepared;
            let point = self
                .point
                .as_ref()
                .ok_or_else(|| ProblemError::internal("fit point"))?;
            let RankAdmission { rows, np, bytes } = plan;
            let reservation =
                datafusion::execution::memory_pool::MemoryConsumer::new("fit:response-diagnostic")
                    .register(&p.runtime.shared.pool());
            reservation
                .try_grow(bytes)
                .map_err(|e| ProblemError::memory(e.to_string()))?;
            let mut response = Mat::zeros(p.measurements.len(), np);
            let free = p
                .parameter_columns
                .iter()
                .enumerate()
                .filter_map(|(i, c)| c.map(|c| (i, c)))
                .collect::<Vec<_>>();
            for (i, _) in p.measurements.iter().enumerate() {
                for (j, (_, col)) in free.iter().enumerate() {
                    response[(i, j)] = point
                        .responses
                        .matrix()
                        .get(i, col.get())
                        .copied()
                        .unwrap_or(0.0);
                }
            }
            for (ei, e) in p.experiments.iter().enumerate() {
                if let Experiment::Steady(s) = e {
                    let nx = s.local_states;
                    if nx == 0 {
                        continue;
                    }
                    // Original model support, projected only by fixed parameter roles. Keep
                    // every equality and free state, including isolates; observation rows are
                    // not closure equations. Matching is library-owned, not a count test.
                    let original = native::assembled::contract(&s.case.assembly);
                    let mut contract = original.clone();
                    contract.variables.truncate(nx);
                    contract.rows = s
                        .constraints
                        .iter()
                        .map(|(r, _)| original.rows[r.get()])
                        .collect();
                    let row_map: BTreeMap<_, _> = s
                        .constraints
                        .iter()
                        .enumerate()
                        .map(|(i, (r, _))| (r.get(), i))
                        .collect();
                    let source = s.case.assembly.jacobian_pattern();
                    let row_map = &row_map;
                    let pairs = (0..nx)
                        .flat_map(|j| {
                            source.row_idx_of_col(j).filter_map(move |r| {
                                row_map.get(&r).map(|i| faer::sparse::Pair::new(*i, j))
                            })
                        })
                        .collect::<Vec<_>>();
                    let (pattern, _) =
                        faer::sparse::SymbolicSparseColMat::try_new_from_indices(nx, nx, &pairs)
                            .map_err(|e| ProblemError::numerical(format!("{e:?}")))?;
                    let bounds = s
                        .constraints
                        .iter()
                        .map(|(_, r)| p.bounds[r.get()])
                        .collect::<Vec<_>>();
                    let scope = native::square_response::SquareScope::admit(
                        &contract,
                        pattern.as_ref(),
                        &bounds,
                        None,
                    )
                    .map_err(|e| ProblemError::unsupported(e.to_string()))?;
                    let states = (0..nx)
                        .map(|i| point.x[s.coordinates[GlobalCol::new(i)].1.get()])
                        .collect::<Vec<_>>();
                    let row_values = s
                        .constraints
                        .iter()
                        .map(|(_, r)| point.constraints[r.get()])
                        .collect::<Vec<_>>();
                    let tolerances = native::quality::Tolerances {
                        variables: (0..nx)
                            .map(|i| {
                                p.tolerances.variables[s.coordinates[GlobalCol::new(i)].1.get()]
                            })
                            .collect(),
                        rows: s
                            .constraints
                            .iter()
                            .map(|(_, r)| p.tolerances.rows[r.get()])
                            .collect(),
                        integrality: p.tolerances.integrality,
                    };
                    let normalization = Normalization {
                        variables: (0..nx)
                            .map(|i| {
                                p.normalization.variables[s.coordinates[GlobalCol::new(i)].1.get()]
                            })
                            .collect(),
                        rows: s
                            .constraints
                            .iter()
                            .map(|(_, r)| p.normalization.rows[r.get()])
                            .collect(),
                        objective: 1.,
                    };
                    let parameters = free
                        .iter()
                        .map(|(_, c)| p.contract.variables[c.get()].id)
                        .collect::<Vec<_>>();
                    let parameter_scales = free
                        .iter()
                        .map(|(_, c)| p.normalization.variables[c.get()])
                        .collect::<Vec<_>>();
                    let jac = point.blocks[ei]
                        .as_ref()
                        .ok_or_else(|| ProblemError::internal("steady response partials"))?;
                    let parameter_values = free
                        .iter()
                        .map(|(_, c)| point.x[c.get()])
                        .collect::<Vec<_>>();
                    // This ephemeral key names this original experiment projection only.
                    // It does not authorize reuse at another fitting point or provider branch.
                    let mut context = FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
                    context
                        .str("fresh-fit-steady-response")
                        .hash(&p.key)
                        .hash(&p.source_identity)
                        .hash(&p.profile_key.as_id())
                        .hash(&p.numerics.key)
                        .u64(ei as u64)
                        .f64(p.profile.rank_tolerance)
                        .f64(tolerances.integrality);
                    for value in &point.x {
                        context.f64(*value);
                    }
                    for (i, variable) in contract.variables.iter().enumerate() {
                        context
                            .id(&variable.id)
                            .f64(variable.lower)
                            .f64(variable.upper)
                            .f64(tolerances.variables[i]);
                    }
                    for (i, row) in contract.rows.iter().enumerate() {
                        context
                            .id(row)
                            .f64(bounds[i].0)
                            .f64(bounds[i].1)
                            .f64(tolerances.rows[i])
                            .f64(row_values[i]);
                    }
                    for scale in &parameter_scales {
                        context.f64(*scale);
                    }
                    let context = context.finish_hash();
                    let mut parameter_key =
                        FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
                    parameter_key.str("fresh-fit-free-parameters");
                    for (id, value) in parameters.iter().zip(&parameter_values) {
                        parameter_key.id(id).f64(*value);
                    }
                    let key = pse_model::strategy::SemanticProductKey {
                        structure: contract.identity,
                        binding: p.key,
                        numerical_policy: Some(p.numerics.key),
                        normalization: Some(normalization.key()),
                        point: Some(native::square_response::point_key(&states)),
                        parameters: Some(parameter_key.finish_hash()),
                        derivation: Some(context),
                        branch: None,
                        accuracy: Some(context),
                    };
                    let qualified_response = native::square_response::products(
                        native::square_response::Request {
                            scope: &scope,
                            point: &states,
                            residual_values: &row_values,
                            tolerances: &tolerances,
                            normalization: &normalization,
                            parameters: &parameters,
                            parameter_scales: &parameter_scales,
                            rank_tolerance: p.profile.rank_tolerance,
                            bytes,
                        },
                        key,
                        &parameter_values,
                        false,
                        || {
                            let mut state_values =
                                Vec::with_capacity(pattern.as_ref().compute_nnz());
                            for column in 0..nx {
                                for row in pattern.as_ref().row_idx_of_col(column) {
                                    state_values.push(
                                        jac.get(s.constraints[row].0.get(), column)
                                            .copied()
                                            .unwrap_or(0.),
                                    );
                                }
                            }
                            let mut parameter_entries = Vec::new();
                            for (column, (_, source)) in free.iter().enumerate() {
                                for (row, (_, target)) in s.constraints.iter().enumerate() {
                                    let value = point
                                        .jacobian
                                        .matrix()
                                        .get(target.get(), source.get())
                                        .copied()
                                        .unwrap_or(0.);
                                    parameter_entries
                                        .push(faer::sparse::Triplet::new(row, column, value));
                                }
                            }
                            let parameter_partials =
                                SparseColMat::try_new_from_triplets(nx, np, &parameter_entries)
                                    .map_err(|error| {
                                        ProblemError::numerical(format!(
                                            "fresh fit parameter partials: {error:?}"
                                        ))
                                    })?;
                            Ok((SparseColMat::new(pattern, state_values), parameter_partials))
                        },
                        &self.execution,
                    )
                    .response
                    .map_err(|e| ProblemError::numerical(e.to_string()))?;
                    let dx = qualified_response.values;
                    for &i in &p.layout.mappings[ei].observations {
                        let o = &p.measurements[i];
                        for j in 0..np {
                            for k in 0..nx {
                                response[(i, j)] +=
                                    jac.get(o.row, k).copied().unwrap_or(0.0) * dx[(k, j)];
                            }
                        }
                    }
                }
            }
            let included = p
                .measurements
                .iter()
                .enumerate()
                .filter(|(_, o)| o.included)
                .collect::<Vec<_>>();
            let weighted = Mat::from_fn(included.len(), np, |i, j| {
                let (row, o) = included[i];
                let scale = p.declaration.parameters[free[j].0].scale;
                response[(row, j)] * scale * o.importance.sqrt() / o.sigma.unwrap_or(1.0)
            });
            let (spectrum, directions) = decompose(&weighted, bytes, true)?;
            let directions =
                directions.ok_or_else(|| ProblemError::internal("right singular vectors"))?;
            let cutoff = spectrum.first().copied().unwrap_or(0.0) * p.profile.rank_tolerance;
            let rank = spectrum.iter().filter(|s| **s > cutoff).count();
            let retained = temp_mat_scratch::<f64>(rows, np).size_bytes();
            let owner = pse_columnar::AllocationLease::new(reservation.split(retained));
            let response = pse_columnar::Leased::new(Arc::new(response), owner);
            Ok(RankDiagnostic {
                responses: response,
                singular_values: spectrum,
                directions,
                rank,
            })
        })
    }
}
pub(super) struct ExperimentWorkers<'a> {
    pub(super) steady: &'a mut Option<CaseWorker>,
    #[cfg(feature = "solver-diffsol")]
    pub(super) transient: &'a mut Option<crate::workflow::dynamics::DynamicWorker>,
}
/// One independent experiment's scientific products. Mutable evaluator/provider scratch
/// belongs to the caller's lane; changed coordinates always create a fresh native IVP.
#[expect(
    clippy::too_many_arguments,
    reason = "one experiment binds problem/index/point/demand, original execution, exclusive workers, prior completed trajectory and joined upgrade validity"
)]
pub(super) fn evaluate_experiment(
    p: &FitProblem,
    ei: usize,
    x: &[f64],
    demand: Demand,
    execution: &Execution,
    workers: ExperimentWorkers<'_>,
    previous: Option<&native::dynamics::Report>,
    validity: &UpgradeValidity,
) -> Result<ExperimentProducts, ProblemError> {
    // Only activated transient backends can report upgrade contradictions below.
    let _ = validity;
    let operation = pse_diagnostics::observation::current().map(|parent| {
        parent.count("fit.experiment_attempts", "count", 1);
        parent.child(
            "fit.experiment",
            pse_diagnostics::observation::Relationship::Nested,
        )
    });
    let context = operation.as_ref().map(|operation| operation.context());
    let _entered = context.as_ref().map(|context| context.enter());
    let producer = context.as_ref().map(|context| {
        context.producer(
            pse_diagnostics::observation::ProducerRole::Workflow,
            "fit.experiment.products",
        )
    });
    let selected_axes = match &p.experiments[ei] {
        Experiment::Transient(s)
            if demand.responses
                && s.profile.sensitivity != native::dynamics::DynamicSensitivity::None =>
        {
            Some(
                p.layout.mappings[ei]
                    .free_bindings
                    .iter()
                    .map(|&local| s.bindings[local].local)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
        }
        _ => None,
    };
    let integration = (matches!(p.experiments[ei], Experiment::Transient(_))
        && !p.layout.mappings[ei].observations.is_empty())
    .then(|| {
        crate::invocation::IntegrationObservation::begin(
            producer,
            demand.purpose,
            demand.responses,
            demand.adjoint,
            false,
            ei as u64,
            selected_axes.as_ref().map(Vec::len),
        )
    });
    let result = (|| {
        let _phase = pse_diagnostics::observation::phase("fit.experiment");
        if let Some(context) = pse_diagnostics::observation::current() {
            if let Ok(index) = i64::try_from(ei) {
                context.integer("fit.experiment_index", "index", index);
            } else {
                context.unavailable(
                    "fit.experiment_index",
                    "index exceeds signed diagnostic representation",
                );
            }
            context.source_ref(
                "fit.experiment",
                p.declaration.experiments[ei].experiment_id.to_string(),
            );
        }
        let steady = workers.steady;
        #[cfg(feature = "solver-diffsol")]
        let transient = workers.transient;
        execution.check()?;
        #[cfg(not(feature = "solver-diffsol"))]
        let _ = previous;
        let mut result = ExperimentProducts::default();
        let e = &p.experiments[ei];
        let mapping = &p.layout.mappings[ei];
        match e {
            Experiment::Steady(s) => {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.text("fit.experiment_kind", "steady");
                    context.count(
                        if steady.is_none() {
                            "fit.experiment_worker_constructions"
                        } else {
                            "fit.experiment_worker_reuses"
                        },
                        "count",
                        1,
                    );
                }
                if steady.is_none() {
                    let providers =
                        crate::math::attempt_providers(&s.providers, &execution.scope()?)
                            .map_err(ProblemError::Provider)?;
                    let mut worker = s.case.assembly.worker_scoped(providers, execution.scope()?);
                    worker.prepare_providers()?;
                    *steady = Some(worker);
                }
                let worker = steady
                    .as_mut()
                    .ok_or_else(|| ProblemError::internal("missing steady worker"))?;
                let mut values = s.values.clone();
                for &(id, col) in &s.coordinates {
                    values.scalars.insert(id, x[col.get()]);
                }
                let outputs = worker.constraints(&values)?;
                for &(row, global) in &s.constraints {
                    result.constraints.push((global.get(), outputs[row.get()]));
                }
                for &i in &mapping.observations {
                    let o = &p.measurements[i];
                    result.predictions.push((i, outputs[o.row]));
                }
                let j = if demand.jacobian || demand.responses || demand.adjoint {
                    Some(worker.jacobian(&values)?.to_owned())
                } else {
                    None
                };
                if let Some(j) = &j {
                    if demand.jacobian {
                        for &(source, target) in &mapping.constraints {
                            result.jacobian.push((target, j.val()[source]));
                        }
                    }
                    if demand.responses || demand.adjoint {
                        for term in &mapping.responses {
                            debug_assert!(p.measurements[term.observation].included);
                            result
                                .responses
                                .push((term.contribution, j.val()[term.local]));
                        }
                    }
                }
                result.block = j;
            }
            Experiment::Transient(s) => {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.text("fit.experiment_kind", "transient");
                }
                if mapping.observations.is_empty() {
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("fit.unobserved_experiment_skips", "count", 1);
                    }
                    return Ok(result);
                }
                #[cfg(feature = "solver-diffsol")]
                {
                    let forward = demand.responses
                        && s.profile.sensitivity != native::dynamics::DynamicSensitivity::None;
                    let combined = demand.adjoint && !mapping.free_bindings.is_empty();
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.boolean("fit.transient_forward", forward);
                        context.boolean("fit.transient_combined_adjoint", combined);
                        context.count(
                            if transient.is_none() {
                                "fit.experiment_worker_constructions"
                            } else {
                                "fit.experiment_worker_reuses"
                            },
                            "count",
                            1,
                        );
                    }
                    if transient.is_none() {
                        *transient = Some(s.worker(execution)?);
                    }
                    let report = if combined {
                        let (report, contributions) = FitOracle::transient_gradient(
                            p,
                            ei,
                            s,
                            x,
                            execution,
                            transient.as_mut().ok_or_else(|| {
                                ProblemError::internal("missing transient worker")
                            })?,
                        )?;
                        for (parameter, value) in contributions {
                            if let Some(column) = p.parameter_columns[parameter] {
                                result.adjoint.push((column.get(), value));
                            }
                        }

                        report
                    } else {
                        s.integrate(
                            transient.as_mut().ok_or_else(|| {
                                ProblemError::internal("missing transient worker")
                            })?,
                            &|k| FitOracle::value(p, x, k),
                            execution,
                            if forward {
                                native::dynamics::DynamicSensitivity::Forward
                            } else {
                                native::dynamics::DynamicSensitivity::None
                            },
                            selected_axes.as_deref().unwrap_or_default(),
                        )?
                    };
                    if let Some(prior) = previous {
                        let missing = mapping.observations.iter().any(|&i| {
                            let observation = &p.measurements[i];
                            observation
                                .sample_index
                                .and_then(|sample| prior.samples.get(sample))
                                .and_then(|sample| sample.outputs.get(observation.row))
                                .is_none()
                        });
                        if missing
                            || prior
                                .samples
                                .iter()
                                .any(|sample| sample.outputs.iter().any(|v| !v.is_finite()))
                        {
                            validity.contradicts();
                            return Err(ProblemError::numerical(
                                "nonfinite completed fit prediction",
                            ));
                        }
                        let _upgrade = pse_diagnostics::observation::phase(
                            "fit.derivative_upgrade_validation",
                        );
                        if let Some(context) = pse_diagnostics::observation::current() {
                            context.count("fit.upgrade_validation_attempts", "count", 1);
                        }
                        let before = s.prediction_allowances(
                            transient.as_mut().ok_or_else(|| {
                                ProblemError::internal("missing transient worker")
                            })?,
                            &|k| FitOracle::value(p, x, k),
                            prior,
                        )?;
                        let after = s.prediction_allowances(
                            transient.as_mut().ok_or_else(|| {
                                ProblemError::internal("missing transient worker")
                            })?,
                            &|k| FitOracle::value(p, x, k),
                            &report,
                        )?;
                        for &i in &mapping.observations {
                            let o = &p.measurements[i];
                            let sample = o
                                .sample_index
                                .ok_or_else(|| ProblemError::internal("fit upgrade sample"))?;
                            let old = prior
                                .samples
                                .get(sample)
                                .and_then(|v| v.outputs.get(o.row))
                                .copied()
                                .ok_or_else(|| ProblemError::internal("fit prior prediction"))?;
                            let new = report
                                .samples
                                .get(sample)
                                .and_then(|v| v.outputs.get(o.row))
                                .copied()
                                .ok_or_else(|| ProblemError::internal("fit upgraded prediction"))?;
                            let allowance = before
                                .get(sample)
                                .and_then(|v| v.get(o.row))
                                .zip(after.get(sample).and_then(|v| v.get(o.row)))
                                .map(|(a, b)| a + b)
                                .ok_or_else(|| ProblemError::internal("fit upgrade allowance"))?;
                            if let Err(error) = FitOracle::admit_upgrade(old, new, allowance) {
                                validity.contradicts();
                                return Err(error);
                            }
                        }
                    }
                    for &i in &mapping.observations {
                        let o = &p.measurements[i];
                        let sample = o
                            .sample_index
                            .and_then(|i| report.samples.get(i))
                            .ok_or_else(|| {
                                ProblemError::internal("missing prepared transient sample")
                            })?;
                        result.predictions.push((i, sample.outputs[o.row]));
                    }
                    if forward {
                        // Each term directly names its measurement and local binding;
                        // retain binding-major contribution order without relationship searches.
                        for term in &mapping.responses {
                            let o = &p.measurements[term.observation];
                            let sample = o
                                .sample_index
                                .and_then(|i| report.samples.get(i))
                                .ok_or_else(|| {
                                    ProblemError::internal("missing prepared transient sample")
                                })?;
                            result.responses.push((
                                term.contribution,
                                s.response(&report, sample, o.row, &s.bindings[term.local])?,
                            ));
                        }
                    }
                    result.trajectory = Some(report);
                }
                #[cfg(not(feature = "solver-diffsol"))]
                {
                    let _ = s;
                    return Err(ProblemError::unsupported("Diffsol not linked"));
                }
            }
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("fit.experiments_completed", "count", 1);
        }
        Ok(result)
    })();
    if let Some(context) = &context {
        context.boolean("fit.experiment_failed", result.is_err());
        if let Err(cause) = &result {
            context.text(
                "fit.experiment_failure_class",
                crate::math::strategy::failure(cause).as_str(),
            );
            if let Some(code) = pse_diagnostics::TypedDiagnostic::diagnostic_code(cause) {
                context.text("fit.experiment_failure_code", code.as_str());
            } else {
                context.unavailable(
                    "fit.experiment_failure_code",
                    "aggregate has no single diagnostic code",
                );
            }
        }
    }
    if let Some(integration) = integration {
        integration.finish(result.is_ok());
    }
    if let Some(operation) = operation {
        operation.finish(match &result {
            Ok(_) => pse_diagnostics::observation::Outcome::Completed,
            Err(ProblemError::Cancelled) => pse_diagnostics::observation::Outcome::Cancelled,
            Err(_) => pse_diagnostics::observation::Outcome::Failed,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::tests::{compiler_profile, id, physical, runtime};
    // Materialize the large fixture future outside the test executor's polling
    // frame, so fixture construction does not consume the fitting caller's stack.
    #[inline(never)]
    fn source(fixed: bool) -> impl Future<Output = crate::workflow::ModelingPackage> {
        Box::pin(source_body(
            fixed,
            "param p: Scalar = 2; let y: Scalar = p*p; annotation check p(p > 0);",
        ))
    }
    async fn source_body(fixed: bool, body: &str) -> crate::workflow::ModelingPackage {
        source_text(fixed, &format!("package p {{ def Root {{ {body} }} }}")).await
    }
    async fn source_text(fixed: bool, text: &str) -> crate::workflow::ModelingPackage {
        source_measured(fixed, text, 3., 2.).await
    }
    async fn source_measured(
        fixed: bool,
        text: &str,
        value: f64,
        sigma: f64,
    ) -> crate::workflow::ModelingPackage {
        let mut physical = physical();
        physical.preconditions = Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        );
        let mut rows = pse_authoring::language::parse(
            text,
            id(20),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        rows.extend(measured_rows(&[(
            id(31),
            "Scalar",
            Some(value),
            Some(sigma),
        )]));
        let mut data = FitDeclarations::default();
        data.fits.push(serde_json::from_value(serde_json::json!({"fit_id":id(32),"parameters":[{"symbol_id":id(1),"fixed":fixed,"value":2.0,"lower":0.1,"upper":10.0,"scale":2.0}],"experiments":[{"experiment_id":id(33),"case_id":root,"route":"steady","bindings":[{"parameter_id":id(1),"path":"p"}]}],"observations":[{"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":id(31),"experiment_id":id(33),"output_path":"y","time":null,"included":true,"importance":4.0}]})).unwrap());
        runtime()
            .modeling_package(rows, physical)
            .await
            .unwrap()
            .with_fit_declarations(data)
            .await
            .unwrap()
    }
    fn profile(_fixed: bool) -> FitProfile {
        FitProfile {
            solver: SolverProfile {
                presolve: Default::default(),
                numerics: Default::default(),
                convexity: Default::default(),
                intent: SolveIntent::Optimize,
                selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
                controls: Default::default(),
                backend: native::execution::BackendSettings::Default,
                sensitivity: None,
                composition: Default::default(),
                block_overrides: Default::default(),
                block_workers: 1,
                reconstruction: None,
            },
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100000,
            derivatives: FitDerivatives::Responses,
            experiment_workers: pse_model::scalar!(PositiveCount(1)),
            uncertainty: None,
        }
    }
    #[tokio::test]
    async fn fitting_value_demand_omits_steady_jacobian_and_upgrades_coherently() {
        let source = source(false).await;
        let cancel = crate::CancelSource::new();
        let preparation = source.prepare_fit_problem(
            id(32).into(),
            profile(false),
            compiler_profile(),
            Default::default(),
            &cancel,
        );
        // Exercise the production future directly. Its composed activity/context
        // state must not embed all steady/transient preparation futures inline.
        assert!(size_of_val(&preparation) < 64 * 1024);
        let p = preparation.await.unwrap().0;
        let execution = Execution::new(
            pse_diagnostics::Cancellation::new(false),
            &p.profile.solver.controls,
        );
        let mut oracle = FitOracle::new(p, execution).unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(
            pse_diagnostics::observation::Policy {
                max_metrics: 64,
                ..Default::default()
            },
        );
        let operation = hub.start("fit cache and upgrade");
        let context = operation.context();
        let _entered = context.enter();
        let objective = oracle.objective(&[2.0]).unwrap();
        let point = oracle.point.as_ref().unwrap();
        assert!(!point.achieved.responses && !point.achieved.jacobian);
        assert!(point.blocks.iter().all(Option::is_none));
        let mut gradient = [0.0];
        oracle.gradient(&[2.0], &mut gradient).unwrap();
        assert!((gradient[0] - 4.0).abs() < 1e-12);
        assert!(oracle.point.as_ref().unwrap().achieved.responses);
        assert_eq!(objective, oracle.objective(&[2.0]).unwrap());
        oracle.objective(&[3.0]).unwrap();
        assert!(
            oracle
                .point
                .as_ref()
                .unwrap()
                .blocks
                .iter()
                .all(Option::is_none)
        );
        let snapshot = context
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let count = |name: &str| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .map(|metric| metric.cumulative.clone())
        };
        use pse_diagnostics::observation::Value;
        assert_eq!(count("fit.trial_demands"), Some(Value::Count(4)));
        assert_eq!(count("fit.trial_cache_reuses"), Some(Value::Count(1)));
        assert_eq!(count("fit.derivative_upgrades"), Some(Value::Count(1)));
        assert_eq!(count("fit.new_trials"), Some(Value::Count(2)));
        assert_eq!(count("fit.trials_completed"), Some(Value::Count(3)));
        let experiments = snapshot
            .children
            .iter()
            .filter(|child| child.kind == "fit.experiment")
            .collect::<Vec<_>>();
        assert_eq!(experiments.len(), 3);
        assert!(
            snapshot
                .sources
                .iter()
                .all(|source| source.name != "fit.experiment")
        );
        for child in experiments {
            let experiment = hub
                .reader(child.id)
                .unwrap()
                .snapshot(std::time::Duration::from_secs(60));
            assert!(
                experiment
                    .sources
                    .iter()
                    .any(|source| source.name == "fit.experiment"
                        && source.bytes == id(33).to_string().as_bytes())
            );
            assert!(
                experiment
                    .metrics
                    .iter()
                    .any(|metric| metric.name == "fit.experiment_index"
                        && metric.latest == Value::Integer(0))
            );
            assert_eq!(
                experiment.outcome,
                Some(pse_diagnostics::observation::Outcome::Completed)
            );
        }
        assert!(!snapshot.coverage.curtailed);
    }
    #[tokio::test]
    async fn prepared_fit_clones_share_the_original_reservation() {
        // Ownership is independent of native backend availability.
        let revision = source(true).await;
        let prepared = revision
            .prepare_fit(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let pool = revision.runtime.shared.pool();
        let before = pool.reserved();
        let bytes = prepared.problem._owner.size();
        let owner = Arc::downgrade(&prepared.problem._owner);
        let copy = prepared.clone();
        assert!(Arc::ptr_eq(&prepared.problem, &copy.problem));
        assert_eq!(pool.reserved(), before);
        drop(prepared);
        assert!(owner.upgrade().is_some());
        assert_eq!(pool.reserved(), before);
        drop(copy);
        assert!(owner.upgrade().is_none());
        assert!(pool.reserved() <= before - bytes);

        // Disposable program retention may yield to pressure. Release it first so
        // saturation covers live allocations throughout the refusal check.
        revision.runtime.clear_program_cache();
        let pressure = datafusion::execution::memory_pool::MemoryConsumer::new("test:fit-pressure")
            .register(&pool);
        let limit = revision.runtime.shared.budget().memory_limit_bytes.get();
        pressure.try_grow(limit - pool.reserved()).unwrap();
        assert_eq!(pool.reserved(), limit);
        assert!(
            revision
                .prepare_fit(
                    id(32).into(),
                    profile(false),
                    compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
        assert_eq!(pool.reserved(), limit);
    }
    #[tokio::test]
    async fn compiled_weighted_loss_gradient_and_exact_hessian() {
        let p = source(false)
            .await
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .0;
        let ex = Execution::new(
            pse_diagnostics::Cancellation::new(false),
            &p.profile.solver.controls,
        );
        let mut o = FitOracle::new(p, ex).unwrap();
        assert!((o.objective(&[2.0]).unwrap() - 0.5).abs() < 1e-12);
        let mut g = [0.0];
        o.gradient(&[2.0], &mut g).unwrap();
        assert!((g[0] - 4.0).abs() < 1e-12);
        let mut h = vec![0.0; o.hessian_pattern().unwrap().row_idx().len()];
        o.hessian(&[2.0], 1.0, &[], &mut h).unwrap();
        assert_eq!(h.len(), 1);
        assert!((h[0] - 18.0).abs() < 1e-12);
        let pool = o.prepared.runtime.shared.pool();
        let before = pool.reserved();
        let RankDiagnostic {
            responses: j,
            singular_values: s,
            rank: r,
            ..
        } = o.response_rank(&[2.0]).unwrap();
        assert_eq!(r, 1);
        assert!((j[(0, 0)] - 4.0).abs() < 1e-12);
        assert!((s[0] - 8.0).abs() < 1e-12);
        assert!(pool.reserved() > before);
        drop(j);
        assert_eq!(pool.reserved(), before);

        // A full pool refuses only the optional dense diagnostic. Sparse values
        // and derivatives remain available at the candidate after that refusal.
        let pressure = datafusion::execution::memory_pool::MemoryConsumer::new("test:pressure")
            .register(&pool);
        let limit = o.prepared.runtime.shared.budget().memory_limit_bytes.get();
        pressure.try_grow(limit - pool.reserved()).unwrap();
        assert!(o.response_rank(&[2.0]).is_err());
        assert!((o.objective(&[2.0]).unwrap() - 0.5).abs() < 1e-12);
        drop(pressure);
        assert!(o.response_rank(&[2.0]).is_ok());
    }
    /// The lower-triangle Hessian values as a dense symmetric matrix.
    fn dense_hessian(o: &mut FitOracle, x: &[f64], sigma: f64, lambda: &[f64]) -> Mat<f64> {
        let pattern = o.hessian_pattern().unwrap().to_owned().unwrap();
        let mut values = vec![0.0; pattern.row_idx().len()];
        o.hessian(x, sigma, lambda, &mut values).unwrap();
        let n = x.len();
        let mut dense = Mat::zeros(n, n);
        for c in 0..n {
            for k in pattern.col_range(c) {
                let r = pattern.row_idx()[k];
                dense[(r, c)] += values[k];
                if r != c {
                    dense[(c, r)] += values[k];
                }
            }
        }
        dense
    }
    /// I8: the Gauss–Newton Hessian is the weighted response Gram JᵀWJ plus the
    /// constraint-multiplier Hessians, and differs from the exact Hessian by the residual
    /// curvature alone.
    #[tokio::test]
    async fn gauss_newton_hessian_matches_jtwj() {
        // One free parameter and one experiment state tied by a nonlinear closure; the
        // observed y = state² has curvature 2 in the state.
        let body = "param p: Scalar = 2; var state: Scalar; annotation start state(2); eq closure: state*state == p*p; let y: Scalar = state*state;";
        let oracle = |hessian| async move {
            let mut profile = profile(false);
            profile.solver.controls.hessian = hessian;
            let p = source_body(false, body)
                .await
                .prepare_fit_problem(
                    id(32).into(),
                    profile,
                    compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap()
                .0;
            let ex = Execution::new(
                pse_diagnostics::Cancellation::new(false),
                &p.profile.solver.controls,
            );
            FitOracle::new(p, ex).unwrap()
        };
        let mut gn = oracle(HessianMode::GaussNewton).await;
        let mut exact = oracle(HessianMode::Exact).await;
        let x = [2.0, 2.0];
        for sigma in [1.0, 0.5] {
            // Without multipliers the Hessian is exactly σ·JᵀWJ, formed densely here from
            // the evaluated responses and weights.
            let prepared = gn.prepared.clone();
            let point = gn.evaluate(&x).unwrap();
            let responses = point.responses.matrix().to_dense();
            let weights = prepared
                .measurements
                .iter()
                .zip(&point.predictions)
                .map(|(o, v)| FitOracle::residual(o, *v).unwrap().1)
                .collect::<Vec<_>>();
            let weighted = Mat::from_fn(responses.nrows(), responses.ncols(), |i, j| {
                responses[(i, j)] * weights[i]
            });
            let gram = weighted.transpose() * &weighted * faer::Scale(sigma);
            let h = dense_hessian(&mut gn, &x, sigma, &[0.0]);
            assert!((&h - &gram).norm_max() < 1e-12, "{h:?} vs {gram:?}");
            assert!(gram.norm_max() > 1.0);
            // A multiplier adds the constraint curvature and nothing else.
            let constrained = dense_hessian(&mut gn, &x, sigma, &[0.5]);
            let curvature = &constrained - &h;
            assert!((curvature.norm_max() - 1.0).abs() < 1e-12, "{curvature:?}");
            // The exact Hessian adds σ·r·w·∇²y = σ·2 at the state's diagonal.
            let full = dense_hessian(&mut exact, &x, sigma, &[0.5]);
            let residual = &full - &constrained;
            assert!(
                (residual.norm_max() - 2.0 * sigma).abs() < 1e-12,
                "{residual:?}"
            );
            assert!(
                (residual.norm_l1() - 2.0 * sigma).abs() < 1e-12,
                "{residual:?}"
            );
        }
    }
    #[tokio::test]
    async fn sparse_fit_admission_tracks_support_and_refills_duplicates() {
        let p = source(false)
            .await
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .0;
        // One thousand independent coordinates need linear derivative storage.
        let Experiment::Steady(base) = &p.experiments[0] else {
            panic!()
        };
        let experiments = (0..1000)
            .map(|column| {
                let mut s = base.clone();
                s.coordinates[GlobalCol::new(0)].1 = OriginalCol::new(column);
                Experiment::Steady(s)
            })
            .collect::<Vec<_>>();
        let observations = (0..1000)
            .map(|experiment| {
                let mut o = p.measurements[0].clone();
                o.experiment = experiment;
                o
            })
            .collect::<Vec<_>>();
        let layout = sparse::Layout::new(
            &experiments,
            &observations,
            &[Some(OriginalCol::new(0))],
            0,
            1000,
            DerivativeOrder::Second,
            6000,
        )
        .unwrap();
        assert!(layout.cells < 6000);
        assert_eq!(layout.hessian.as_ref().unwrap().matrix().val().len(), 1000);
        assert!(
            sparse::Layout::new(
                &experiments,
                &observations,
                &[Some(OriginalCol::new(0))],
                0,
                1000,
                DerivativeOrder::Second,
                2000
            )
            .is_err()
        );

        // Repeated contributions to the same response sum before J' W J.
        let duplicate = sparse::Layout::new(
            &[p.experiments[0].clone()],
            &p.measurements,
            &[Some(OriginalCol::new(0))],
            0,
            1,
            DerivativeOrder::Second,
            100,
        )
        .unwrap();
        let mut worker =
            sparse::GramWorker::new(duplicate.gram.unwrap(), &duplicate.responses, 1 << 20)
                .unwrap();
        let mut response = duplicate.responses;
        let mut hessian = duplicate.hessian.unwrap();
        for value in [3.0, 0.0, -2.0, 4.0] {
            response.clear();
            hessian.clear();
            response
                .add(pse_math::index::Addend::new(0), value)
                .unwrap();
            response.add(pse_math::index::Addend::new(0), 1.0).unwrap();
            worker.refill(&response, &[2.0], 0.5, &mut hessian).unwrap();
            assert!((hessian.matrix().val()[0] - 2.0 * (value + 1.0).powi(2)).abs() < 1e-12);
        }
    }
    #[tokio::test]
    async fn bounded_rank_diagnostic_does_not_disable_sparse_candidate_evaluation() {
        let mut p = source(false)
            .await
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .0;
        p.profile.max_cells = 0;
        let ex = Execution::new(
            pse_diagnostics::Cancellation::new(false),
            &p.profile.solver.controls,
        );
        let mut oracle = FitOracle::new(p, ex).unwrap();
        assert!(oracle.response_rank(&[2.0]).is_err());
        assert!(
            oracle.point.is_none(),
            "known rank refusal must not evaluate the point"
        );
        assert!((oracle.objective(&[2.0]).unwrap() - 0.5).abs() < 1e-12);
        let mut gradient = [0.0];
        oracle.gradient(&[2.0], &mut gradient).unwrap();
        assert!((gradient[0] - 4.0).abs() < 1e-12);
    }
    #[tokio::test]
    async fn all_fixed_fit_uses_joined_direct_evaluation_and_explicit_source_export() {
        let mut requested = profile(true);
        requested.solver.selection = native::solve::SolverSelection::Auto;
        let package = source(true).await;
        let source_tables = package.source_tables().await.unwrap();
        let p = package
            .prepare_fit(
                id(32).into(),
                requested,
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(p.problem.profile.solver.intent, SolveIntent::Optimize);
        assert_eq!(
            p.problem.profile.solver.selection,
            native::solve::SolverSelection::Auto
        );
        assert!(p.problem.contract.variables.is_empty());
        assert_eq!(p.route(), native::routing::Route::Constant);
        let handle = p.start().unwrap();
        let a = handle.wait().await.unwrap();
        let b = handle.wait().await.unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        let crate::workflow::RunReport::Fit(r) = a.report().unwrap() else {
            panic!()
        };
        assert!(r.solve.is_none());
        let trace = r.strategy.as_ref().unwrap();
        assert!(trace.events.iter().all(|event| event.decision.is_some()));
        let work = trace.events.last().unwrap().work.unwrap().observed;
        assert_eq!(work.evaluations, None);
        assert_eq!(work.iterations, None);
        assert_eq!(
            trace.events.last().unwrap().transition,
            Some(pse_model::strategy::Transition::Finish)
        );
        let events = a.table("runtime.solve_strategy_events").unwrap();
        assert_eq!(events.batch().num_rows(), trace.events.len());
        assert!(trace.owner.is_some());
        assert_eq!(r.predictions, vec![Some(4.0)]);
        let table = a.table("runtime.fit_observations").unwrap();
        assert_eq!(table.batch().num_rows(), 1);
        assert_eq!(
            source_tables[&pse_relations::generated::authored::fit_cases::RELATION_ID]
                .batch()
                .num_rows(),
            1
        );
        drop((a, b, handle, p, package, source_tables));
        assert_eq!(table.batch().num_rows(), 1);
    }
    #[tokio::test]
    async fn scientific_fit_strict_composed_work_refuses_before_evaluation() {
        for counter in 0..4 {
            let mut requested = profile(true);
            requested.solver.selection = native::solve::SolverSelection::Auto;
            let mut limits = pse_model::strategy::WorkLimits {
                attempts: 1,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            };
            match counter {
                0 => limits.evaluations = Some(1),
                1 => limits.iterations = Some(1),
                2 => limits.factorizations = Some(1),
                _ => limits.proof_steps = Some(1),
            }
            requested.solver.composition.limits = Some(limits);
            let prepared = source(true)
                .await
                .prepare_fit(
                    id(32).into(),
                    requested,
                    compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap();
            let error = prepared
                .execute(
                    id(99).into(),
                    pse_kernels::ExecutionScope::new(
                        pse_diagnostics::Cancellation::default(),
                        None,
                    ),
                    Arc::new(native::solve::Progress::new(16)),
                    1,
                )
                .unwrap_err();
            let trace = error.strategy_trace().unwrap();
            assert_eq!(trace.declaration.limits, limits);
            assert!(trace.events.iter().all(
                |event| event.kind != pse_model::generated::enums::NumericalEventKind::Started
            ));
            assert!(
                trace.events.iter().any(|event| matches!(
                    event.cause.as_deref(),
                    Some(ProblemError::Unsupported(_))
                ))
            );
            assert!(trace.events.iter().all(|event| event.decision.is_some()));
        }
    }
    #[tokio::test]
    async fn all_fixed_required_presolve_is_refused_and_scales_are_checked() {
        let revision = source(true).await;
        let mut controls = profile(true);
        controls.solver.presolve = native::presolve::Policy::from_native_options(
            &Default::default(),
            [native::presolve::Pass::AffineElimination].into(),
        )
        .unwrap();
        let error = revision
            .prepare_fit(
                id(32).into(),
                controls,
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, WorkflowError::Input(ref message) if message == "all-fixed fitting evaluates directly and cannot apply required native presolve passes")
        );
    }
    #[cfg(not(feature = "solver-ipopt"))]
    fn assert_unlinked_ipopt(error: &WorkflowError) {
        let WorkflowError::Math(crate::math::MathRuntimeError::Solve(ProblemError::RouteRefused(
            decision,
        ))) = error
        else {
            panic!("expected a typed routing refusal, got {error:?}");
        };
        assert_eq!(
            decision.selection,
            native::solve::SolverSelection::Explicit(Backend::Ipopt)
        );
        assert!(decision.selected.is_none());
        assert!(matches!(
            decision.refusal.as_ref(),
            Some(native::routing::Refusal::Unavailable(Backend::Ipopt))
        ));
        let eligibility = decision
            .eligibility
            .iter()
            .find(|entry| entry.backend == Backend::Ipopt)
            .unwrap();
        assert!(
            eligibility
                .reasons
                .contains(&native::routing::Ineligible::NotLinked)
        );
        assert!(eligibility.causes.iter().any(|cause| matches!(
            cause.as_ref(),
            ProblemError::Unavailable {
                backend: Backend::Ipopt,
                ..
            }
        )));
    }
    #[cfg(not(feature = "solver-ipopt"))]
    #[tokio::test]
    async fn variable_fit_requires_a_linked_adapter() {
        let error = source(false)
            .await
            .prepare_fit(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert_unlinked_ipopt(&error);
    }
    #[tokio::test]
    async fn invalid_uncertainty_and_unbound_observations_fail_admission() {
        let b = source_measured(
            false,
            "package p {def Root {param p:Scalar=2;let y:Scalar=p*p;}}",
            3.,
            0.,
        )
        .await;
        let r = b;
        let error = r
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, WorkflowError::Input(ref message) if message == "included observations require finite values, positive difference-unit standard deviations and importance")
        );
        let b = source(false).await;
        let mut fits = b.fit_declarations().await.unwrap();
        fits.fits[0].observations[0].experiment_id = id(99).into();
        let b = b.with_fit_declarations(fits).await.unwrap();
        let r = b;
        let error = r
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, WorkflowError::Input(ref message) if message == "fit observation or dynamic profile ownership")
        );
    }
    #[tokio::test]
    async fn steady_response_solves_the_compiled_implicit_closure() {
        let body = "param p: Scalar = 2; var state: Scalar; annotation start state(2); eq closure: state == p; let y: Scalar = state*state;";
        let b = source_body(false, body).await;
        let profile = profile(false);
        let fixed = source_body(true, body).await;
        let fixed_profile = profile.clone();
        let fixed_problem = fixed
            .prepare_fit_problem(
                id(32).into(),
                fixed_profile.clone(),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .0;
        // Fixed fit parameters do not eliminate free experiment-local states.
        assert_eq!(fixed_problem.contract.variables.len(), 1);
        #[cfg(not(feature = "solver-ipopt"))]
        {
            let error = fixed
                .prepare_fit(
                    id(32).into(),
                    fixed_profile,
                    compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap_err();
            assert_unlinked_ipopt(&error);
        }
        let p = b
            .prepare_fit_problem(
                id(32).into(),
                profile,
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .0;
        let ex = Execution::new(
            pse_diagnostics::Cancellation::new(false),
            &p.profile.solver.controls,
        );
        let mut o = FitOracle::new(p, ex).unwrap();
        let RankDiagnostic {
            responses: response,
            rank,
            ..
        } = o.response_rank(&[2.0, 2.0]).unwrap();
        assert_eq!(rank, 1);
        assert!((response[(0, 0)] - 4.0).abs() < 1e-12);
        assert!(o.response_rank(&[2.0, 3.0]).is_err());
    }

    #[tokio::test]
    async fn fit_refuses_free_integer() {
        let body = "param p: Scalar = 2; var n: Count in integer; annotation start n(1{1}); annotation bounds n(0{1}, 5{1}); eq e: n >= 1{1}; let y: Scalar = p*p;";
        let error = source_body(false, body)
            .await
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .err()
            .unwrap();
        assert_eq!(
            crate::workflow::tests::free_discrete_refusal(&error),
            ("n".into(), "fitting".into())
        );
    }
    #[tokio::test]
    async fn authored_fit_retains_fixed_case_values_and_physical_bounds() {
        use pse_relations::columnar::RelationRow;
        let package = source_text(
            true,
            "package p { test Root fixture {dof 0; fix x=2;} {param p:Scalar=2; var x:Scalar; let y:Scalar=x+p; annotation bounds x(1,3); annotation check y(y==4);} }",
        ).await;
        let result = package
            .prepare_fit(
                id(32).into(),
                profile(true),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        let table = result.table("runtime.fit_variables").unwrap();
        let rows = pse_relations::generated::runtime::fit_variables::Row::rows(&table).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].fixed);
        assert_eq!(rows[0].value, Some(2.));
        assert_eq!((rows[0].lower, rows[0].upper), (Some(1.), Some(3.)));
        assert!(result.usable());
    }
    #[tokio::test]
    async fn authored_fit_can_observe_a_parameter_without_an_alias() {
        let package = source(true).await;
        let mut fits = package.fit_declarations().await.unwrap();
        fits.fits[0].observations[0].output_path = "p".into();
        let package = package.with_fit_declarations(fits).await.unwrap();
        let result = package
            .prepare_fit(
                id(32).into(),
                profile(true),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!()
        };
        assert_eq!(report.predictions, vec![Some(2.)]);
        assert!(result.usable());
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn final_fit_derivative_failure_preserves_fresh_original_values() {
        // sqrt is defined at zero; its derivative is not. The optional response
        // product must fail without discarding an independently valid value.
        let package = source_body(false, "param p:Scalar=2; let y:Scalar=sqrt(p);").await;
        let mut declarations = package.fit_declarations().await.unwrap();
        declarations.fits[0].parameters[0].lower = Some(0.0);
        let package = package.with_fit_declarations(declarations).await.unwrap();
        let (problem, _) = package
            .prepare_fit_problem(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        p09_tests::with_admitted_fit_oracle(problem.into(), |oracle| {
            let problem = oracle.prepared.clone();
            let mut report = p09_tests::candidate_fit_report(&problem, vec![0.0]);
            problem
                .observe(&mut report, oracle.execution.clone())
                .unwrap();
            assert_eq!(report.predictions, vec![Some(0.0)]);
            assert_eq!(report.objective, Some(4.5));
            assert!(report.quality.as_ref().unwrap().feasible());
            assert!(matches!(
                report.diagnostic,
                Some(FitDiagnostic {
                    rule: FitRule::ResponseRank,
                    ..
                })
            ));
            assert!(report.responses.is_none() && report.rank.is_none());
        })
        .await;
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn original_fit_failure_withholds_completed_statistical_products_in_invocation() {
        use crate::invocation::ProductAvailability;
        use pse_model::scalars::Fraction;
        let package = source_body(
            false,
            "param p: Scalar = 2; let y: Scalar = p*p; annotation check p(p > 3);",
        )
        .await;
        let mut declarations = package.fit_declarations().await.unwrap();
        declarations.fits[0].observations[0].importance = 1.;
        let package = package.with_fit_declarations(declarations).await.unwrap();
        let mut requested = profile(false);
        requested.solver.controls.hessian = HessianMode::GaussNewton;
        requested.uncertainty = Some(FitUncertainty {
            level: pse_model::scalar!(Fraction(0.95)),
            profile: None,
            predictions: false,
        });
        let prepared = package
            .prepare_fit(
                id(32).into(),
                requested,
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("fit original failure after statistical computation");
        let _entered = operation.context().enter();
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!()
        };
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(report.checks_complete && report.checks.iter().any(|check| !check.satisfied));
        assert!(
            report.covariance.as_ref().unwrap().values.is_ok(),
            "retain completed computation evidence"
        );
        assert!(report.wald.as_ref().unwrap().is_ok());
        assert!(!result.usable() && !report.estimate_qualified());
        let document = result.invocation_evidence().unwrap();
        assert_eq!(document.version, 2);
        assert!(document.timing_unavailable.is_none());
        assert!(
            document
                .intervals
                .iter()
                .any(|interval| interval.stage == "native.worker_execution"
                    && interval.ended_seconds.is_some())
        );
        assert!(
            document
                .intervals
                .iter()
                .any(|interval| interval.stage == "fit.original_final_assessment"
                    && interval.ended_seconds.is_some()),
            "native final assessment must reach the returned pre-dispatch invocation"
        );
        for role in [
            "original_scientific_result",
            "parameter_covariance",
            "wald_intervals",
        ] {
            assert!(
                document.products.iter().any(|product| product.role == role
                    && product.availability == ProductAvailability::Unavailable),
                "{role}"
            );
        }
        assert!(
            document
                .products
                .iter()
                .any(|product| product.role == "profile_intervals"
                    && product.availability == ProductAvailability::NotRequested)
        );
        assert!(document.endpoints.qualification_seconds.is_none());
    }
    #[tokio::test]
    async fn authored_fit_checks_can_reject_a_numerically_feasible_candidate() {
        let package = source_body(
            true,
            "param p: Scalar = 2; let y: Scalar = p*p; annotation check p(p > 3);",
        )
        .await;
        let mut requested = profile(true);
        requested.solver.composition.recovery = vec![pse_model::strategy::StartOrigin::Auxiliary];
        let prepared = package
            .prepare_fit(
                id(32).into(),
                requested,
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!()
        };
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(report.checks_complete);
        assert!(report.checks.iter().any(|c| !c.satisfied));
        assert!(!result.usable());
        let trace = report.strategy.as_ref().unwrap();
        let final_event = trace.events.last().unwrap();
        assert_eq!(
            final_event.transition,
            Some(pse_model::strategy::Transition::Stop)
        );
        assert_eq!(final_event.phase, pse_model::strategy::Phase::Assessment);
        assert_eq!(
            final_event.permission,
            Some(pse_model::generated::enums::CandidateUse::Unusable)
        );
        assert!(final_event.decision.is_some());
        assert!(matches!(
            final_event.original,
            Some(crate::math::strategy::OriginalConclusion::Refused { .. })
        ));
        assert_eq!(
            trace
                .events
                .iter()
                .filter(
                    |event| event.kind == pse_model::generated::enums::NumericalEventKind::Started
                )
                .count(),
            1
        );
        assert!(!report.estimate_qualified());
        assert!(result.export_fit_parameters().is_err());
        assert_eq!(
            result
                .table("runtime.modeling_checks")
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
    }
    #[tokio::test]
    async fn authored_fit_source_profile_and_binding_ownership_are_separate() {
        let package = source(true).await;
        let cancel = crate::CancelSource::new();
        let (first, _) = package
            .prepare_fit_problem(
                id(32).into(),
                profile(true),
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        let mut other = profile(true);
        other.rank_tolerance = 1e-6;
        let (second, _) = package
            .prepare_fit_problem(
                id(32).into(),
                other,
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(first.source_identity, second.source_identity);
        assert_ne!(first.profile_key, second.profile_key);
        assert_ne!(first.key, second.key);
        let edited = source_measured(
            true,
            "package p {def Root {param p: Scalar = 2; let y: Scalar = p*p; annotation check p(p > 0);}}",
            5.,
            2.,
        ).await;
        let (third, _) = edited
            .prepare_fit_problem(
                id(32).into(),
                profile(true),
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_ne!(first.source_identity, third.source_identity);
        assert_eq!(
            package
                .all_revision()
                .await
                .unwrap()
                .checked()
                .measurement(
                    package.fit_declarations().await.unwrap().fits[0].experiments[0].case_id,
                    id(31).into(),
                    "value",
                    Some("sigma")
                )
                .unwrap()
                .value,
            Some(3.)
        );
        let mut data = package.fit_declarations().await.unwrap();
        data.fits[0].parameters[0].value = 2.5;
        let changed = package.clone().with_fit_declarations(data).await.unwrap();
        let (fourth, _) = changed
            .prepare_fit_problem(
                id(32).into(),
                profile(true),
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_ne!(
            first.source_identity, fourth.source_identity,
            "fit declaration belongs to source identity"
        );
        let mut data = changed.fit_declarations().await.unwrap();
        let binding = data.fits[0].experiments[0].bindings[0].clone();
        data.fits[0].experiments[0].bindings.push(binding);
        let invalid = changed.with_fit_declarations(data).await.unwrap();
        assert!(
            invalid
                .prepare_fit_problem(
                    id(32).into(),
                    profile(true),
                    compiler_profile(),
                    Default::default(),
                    &cancel
                )
                .await
                .unwrap_err()
                .to_string()
                .contains("duplicate experiment parameter binding")
        );
    }

    /// Solve sequences, block initialization and fitting all execute NLP routes through
    /// the one shared runner, so their reports carry the same pipeline fields.
    #[cfg(all(feature = "solver-ipopt", feature = "solver-kinsol"))]
    #[tokio::test]
    async fn nlp_runner_serves_solve_initialize_fit() {
        use crate::math::solves::{NumericalInputs, Outcome};
        use native::solve::{Controls, Qualification, SolveReport, SolverSelection};
        use pse_compiler::workspace::ModelingCaseBindings;
        let cancel = crate::CancelSource::new();
        let compiler = compiler_profile();
        let physical = physical();
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3); annotation nominal x(2); } }",
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
        let package = runtime().modeling_package(rows, physical).await.unwrap();
        let ipopt = |intent| SolverProfile {
            presolve: Default::default(),
            numerics: Default::default(),
            convexity: Default::default(),
            intent,
            selection: SolverSelection::Explicit(Backend::Ipopt),
            controls: Controls::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            block_overrides: Default::default(),
            block_workers: 1,
            reconstruction: None,
        };
        // A solve sequence step.
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::Second,
                compiler,
                ipopt(SolveIntent::FeasiblePoint),
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let solved = package
            .solve_case(prepared, compiler, &cancel)
            .await
            .unwrap();
        let Outcome::Native(solve) = &solved.outcome else {
            panic!("{:?}", solved.outcome)
        };
        // A block initialization attempt.
        let analysis = package
            .declared_execution(
                root,
                compiler,
                // Initialization blocks prepare first derivatives only.
                SolverProfile {
                    controls: Controls {
                        hessian: HessianMode::LimitedMemory,
                        ..Controls::default()
                    },
                    ..ipopt(SolveIntent::Initialize)
                },
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap()
            .analysis;
        let initialized = package
            .prepare_block_initialization(&analysis, vec![BTreeMap::new()], &cancel)
            .await
            .unwrap()
            .start()
            .unwrap()
            .finish()
            .await
            .unwrap();
        let initialize = initialized.attempts[0].result.as_ref().unwrap();
        // A parameter fit.
        let fitted = source(false)
            .await
            .prepare_fit(
                id(32).into(),
                profile(false),
                compiler,
                Default::default(),
                &cancel,
            )
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        let crate::workflow::RunReport::Fit(fit) = fitted.report().unwrap() else {
            panic!("fit report")
        };
        let fit = fit.solve.as_ref().unwrap();
        // The runner's pipeline fields: preprocessing receipt with library recovery, the
        // original-coordinate KKT evidence and its budgets, and a numerical qualification.
        let fields = |r: &SolveReport| {
            let preprocessing = r.preprocessing.as_ref().expect("pipeline receipt");
            (
                r.backend,
                preprocessing.diagnostics.get("recovery").cloned(),
                preprocessing.passes.keys().copied().collect::<Vec<_>>(),
                r.evidence.kkt.is_some(),
                [
                    "quality.stationarity.budget",
                    "quality.complementarity.budget",
                ]
                .map(|k| r.metrics.contains_key(k)),
                r.warm_start.as_ref().map(|w| w.compatibility.backend),
            )
        };
        let expected = fields(solve);
        assert_eq!(expected.0, Backend::Ipopt);
        assert!(expected.1.is_some());
        assert!(expected.3);
        assert_eq!(expected.4, [true, true]);
        for (name, report) in [("initialize", initialize.as_ref()), ("fit", fit)] {
            assert_eq!(fields(report), expected, "{name}");
        }
        for report in [solve.as_ref(), initialize.as_ref(), fit] {
            assert_ne!(
                report.qualification,
                Qualification::Unqualified,
                "{report:?}"
            );
        }
        assert!((solve.candidate.as_ref().unwrap().primal[0] - 2.0).abs() < 1e-6);
        assert!((initialized.values.scalars.values().next().unwrap() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn library_parameter_rank_has_independent_controls() {
        let a = Mat::from_fn(2, 2, |i, j| if i == j { 2.0 } else { 0.0 });
        let singular = Mat::from_fn(2, 2, |_, _| 1.0);
        let (s, _) = decompose(&singular, 1 << 20, false).unwrap();
        assert!(s[1] < 1e-12);
        assert!(decompose(&a, 0, false).is_err());
    }
}

#[cfg(test)]
#[cfg(feature = "solver-diffsol")]
#[path = "p09_tests.rs"]
mod p09_tests;
