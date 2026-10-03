// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin experiment composition; compiled library derivatives own local mathematics.
use super::*;
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
use std::{sync::atomic::AtomicBool, time::Instant};
#[derive(Debug)]
struct Point {
    x: Vec<f64>,
    predictions: Vec<f64>,
    responses: AssemblyMatrix,
    constraints: Vec<f64>,
    jacobian: AssemblyMatrix,
    blocks: Vec<Option<SparseColMat<usize, f64>>>,
    trajectories: BTreeMap<InstanceId, Arc<native::dynamics::Report>>,
    /// The transient experiments' adjoint part of the objective gradient, once computed.
    adjoint: Option<Vec<f64>>,
}
#[derive(Debug)]
pub(super) struct FitOracle {
    prepared: Arc<FitProblem>,
    workers: Vec<Option<CaseWorker>>,
    execution: Execution,
    /// The derivative source: the prepared profile's, until the final assessment reruns
    /// the forward sensitivities for rank.
    derivatives: FitDerivatives,
    hessian: Option<AssemblyMatrix>,
    gram: Option<sparse::GramWorker>,
    point: Option<Point>,
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
        let p = p.into();
        let workers = p
            .experiments
            .iter()
            .map(|e| match e {
                Experiment::Steady(s) => {
                    let providers = crate::math::attempt_providers(&s.providers, &execution.cancel)
                        .map_err(ProblemError::Provider)?;
                    Ok(Some(
                        s.case.assembly.worker(providers, execution.cancel.clone()),
                    ))
                }
                Experiment::Transient(_) => Ok(None),
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
            derivatives: p.profile.derivatives,
            prepared: p,
            workers,
            execution,
            hessian,
            gram,
            point: None,
        })
    }
    fn evaluate(&mut self, x: &[f64]) -> Result<&Point, ProblemError> {
        if let Some(stop) = self.execution.stopped() {
            return Err(ProblemError::stopped(stop, "fit evaluation deadline"));
        }
        let p = &self.prepared;
        let n = p.contract.variables.len();
        if x.len() != n || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("fit trial coordinates"));
        }
        if self
            .point
            .as_ref()
            .is_some_and(|v| v.x.iter().zip(x).all(|(a, b)| a.to_bits() == b.to_bits()))
        {
            return self
                .point
                .as_ref()
                .ok_or_else(|| ProblemError::internal("fit point cache"));
        }
        self.point = None;
        let mut point = Point {
            x: x.to_vec(),
            predictions: vec![0.0; p.measurements.len()],
            responses: p.layout.responses.clone(),
            constraints: vec![0.0; p.contract.rows.len()],
            jacobian: p.layout.constraints.clone(),
            blocks: Vec::new(),
            trajectories: BTreeMap::new(),
            adjoint: None,
        };
        for (ei, e) in p.experiments.iter().enumerate() {
            match e {
                Experiment::Steady(s) => {
                    let worker = self.workers[ei]
                        .as_mut()
                        .ok_or_else(|| ProblemError::internal("missing steady worker"))?;
                    let mut values = s.values.clone();
                    for &(id, col) in &s.coordinates {
                        values.scalars.insert(id, x[col.get()]);
                    }
                    let outputs = worker.constraints(&values)?;
                    let j = worker.jacobian(&values)?;
                    for &(row, global) in &s.constraints {
                        point.constraints[global.get()] = outputs[row.get()];
                    }
                    let mapping = &p.layout.mappings[ei];
                    for &(source, target) in &mapping.constraints {
                        point.jacobian.add(target, j.val()[source])?;
                    }
                    for term in &mapping.responses {
                        debug_assert!(p.measurements[term.observation].included);
                        point
                            .responses
                            .add(term.contribution, j.val()[term.local])?;
                    }
                    for (i, o) in p
                        .measurements
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| o.experiment == ei && o.included)
                    {
                        point.predictions[i] = outputs[o.row];
                    }
                    point.blocks.push(Some(j.to_owned()));
                }
                Experiment::Transient(s) => {
                    if !p
                        .measurements
                        .iter()
                        .any(|o| o.experiment == ei && o.included)
                    {
                        point.blocks.push(None);
                        continue;
                    }
                    #[cfg(feature = "solver-diffsol")]
                    {
                        // A gradient-only fit integrates without sensitivities here; its
                        // gradient is the adjoint product of `FitOracle::adjoint`.
                        let forward = self.derivatives == FitDerivatives::Responses
                            && s.profile.sensitivity != native::dynamics::DynamicSensitivity::None;
                        let report = s.integrate(
                            &|k| Self::value(p, x, k),
                            &self.execution,
                            if forward {
                                native::dynamics::DynamicSensitivity::Forward
                            } else {
                                native::dynamics::DynamicSensitivity::None
                            },
                        )?;
                        for (i, o) in p
                            .measurements
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| o.experiment == ei && o.included)
                        {
                            let sample = o
                                .sample_index
                                .and_then(|i| report.samples.get(i))
                                .ok_or_else(|| {
                                    ProblemError::internal("missing prepared transient sample")
                                })?;
                            point.predictions[i] = sample.outputs[o.row];
                            if !forward {
                                continue;
                            }
                            for term in p.layout.mappings[ei]
                                .responses
                                .iter()
                                .filter(|t| t.observation == i)
                            {
                                let binding = s
                                    .bindings
                                    .iter()
                                    .find(|b| b.local == term.local)
                                    .ok_or_else(|| {
                                        ProblemError::internal(
                                            "transient response parameter binding",
                                        )
                                    })?;
                                point
                                    .responses
                                    .add(term.contribution, s.response(sample, o.row, binding)?)?;
                            }
                        }
                        point.trajectories.insert(
                            p.declaration.experiments[ei].experiment_id,
                            Arc::new(report),
                        );
                        point.blocks.push(None);
                    }
                    #[cfg(not(feature = "solver-diffsol"))]
                    {
                        let _ = s;
                        return Err(ProblemError::unsupported("Diffsol not linked"));
                    }
                }
            }
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
            phase: "fit.evaluation".into(),
            elapsed: self.execution.started.elapsed(),
            values: BTreeMap::from([(
                "experiments".into(),
                native::solve::Metric::Integer(p.experiments.len() as i64),
            )]),
            incumbent: None,
        });
        self.point = Some(point);
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
    /// The transient experiments' part of the objective gradient by adjoint sensitivities
    /// (ADR-0110 item 3), once per trial point.
    fn adjoint(&mut self, x: &[f64]) -> Result<Vec<f64>, ProblemError> {
        if let Some(gradient) = self.point.as_ref().and_then(|p| p.adjoint.clone()) {
            return Ok(gradient);
        }
        let p = self.prepared.clone();
        let mut total = vec![0.0; x.len()];
        for (ei, e) in p.experiments.iter().enumerate() {
            let Experiment::Transient(s) = e else {
                continue;
            };
            if !p
                .measurements
                .iter()
                .any(|o| o.experiment == ei && o.included)
                || !s
                    .bindings
                    .iter()
                    .any(|b| p.parameter_columns[b.parameter].is_some())
            {
                continue;
            }
            for (parameter, value) in self.transient_gradient(&p, ei, s, x)? {
                if let Some(column) = p.parameter_columns[parameter] {
                    total[column.get()] += value;
                }
            }
        }
        if let Some(point) = self.point.as_mut() {
            point.adjoint = Some(total.clone());
        }
        Ok(total)
    }
    /// One transient experiment's gradient contributions from one forward and one backward
    /// pass, whose cotangent is the weighted residual of each included observation.
    #[cfg(feature = "solver-diffsol")]
    fn transient_gradient(
        &self,
        p: &FitProblem,
        ei: usize,
        s: &IntegratedExperiment,
        x: &[f64],
    ) -> Result<Vec<(usize, f64)>, ProblemError> {
        let outputs = s.program.contract.outputs.len();
        let mut cotangent =
            |report: &native::dynamics::Report| Self::cotangent(p, ei, outputs, report);
        s.gradient(&|k| Self::value(p, x, k), &self.execution, &mut cotangent)
            .map(|(_, contributions)| contributions)
    }
    #[cfg(not(feature = "solver-diffsol"))]
    fn transient_gradient(
        &self,
        _: &FitProblem,
        _: usize,
        _: &IntegratedExperiment,
        _: &[f64],
    ) -> Result<Vec<(usize, f64)>, ProblemError> {
        Err(ProblemError::unsupported("Diffsol not linked"))
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
        for o in p
            .measurements
            .iter()
            .filter(|o| o.experiment == ei && o.included)
        {
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
    fn transient_curvatures(&self, x: &[f64]) -> Result<Vec<Option<Vec<f64>>>, ProblemError> {
        let p = self.prepared.clone();
        p.experiments
            .iter()
            .enumerate()
            .map(|(ei, e)| {
                let Experiment::Transient(s) = e else {
                    return Ok(None);
                };
                let free = s
                    .bindings
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| p.parameter_columns[b.parameter].is_some())
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();
                if free.is_empty()
                    || !p
                        .measurements
                        .iter()
                        .any(|o| o.experiment == ei && o.included)
                {
                    return Ok(None);
                }
                self.transient_curvature(&p, ei, s, x, &free).map(Some)
            })
            .collect()
    }
    #[cfg(feature = "solver-idas")]
    fn transient_curvature(
        &self,
        p: &FitProblem,
        ei: usize,
        s: &IntegratedExperiment,
        x: &[f64],
        free: &[usize],
    ) -> Result<Vec<f64>, ProblemError> {
        let outputs = s.program.contract.outputs.len();
        let mut cotangent =
            |report: &native::dynamics::Report| Self::cotangent(p, ei, outputs, report);
        s.hessian(
            &|k| Self::value(p, x, k),
            &self.execution,
            free,
            &mut cotangent,
        )
        .map(|(_, _, curvature)| curvature)
    }
    #[cfg(not(feature = "solver-idas"))]
    fn transient_curvature(
        &self,
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
        self.evaluate(x)?;
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
        let p = self.evaluate(x)?;
        if out.len() != p.constraints.len() {
            return Err(ProblemError::internal("fit constraint extent"));
        }
        out.copy_from_slice(&p.constraints);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.evaluate(x)?;
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
            self.adjoint(x)?
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
        self.evaluate(x)?;
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
            HessianMode::LimitedMemory => {
                return Err(ProblemError::internal("limited-memory fit Hessian demand"));
            }
        };
        // The exact Hessian's transient curvature Σ rᵢwᵢ∇²yᵢ, by second-order adjoint
        // sensitivities of each transient experiment (ADR-0110 item 4).
        let curvatures = if exact {
            self.transient_curvatures(x)?
        } else {
            vec![None; self.prepared.experiments.len()]
        };
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
                    if let Some(curvature) = &curvatures[ei] {
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
                for (i, o) in p
                    .measurements
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| o.experiment == ei && o.included)
                {
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
    pub(crate) fn execute(
        self: &Arc<Self>,
        route: native::routing::Route,
        flag: Arc<AtomicBool>,
        progress: Arc<native::solve::Progress>,
        workers: usize,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let adapters: Vec<&dyn native::execution::BackendExecution> = match route {
            native::routing::Route::Native(backend) => vec![native::execution::adapter(backend)],
            native::routing::Route::Constant => vec![],
        };
        native::execution::scoped(
            &adapters,
            self.profile.solver.controls.threads,
            self.runtime.native().stack_bytes(),
            || self.execute_inner(route, flag, progress, workers),
        )
    }
    fn execute_inner(
        self: &Arc<Self>,
        route: native::routing::Route,
        flag: Arc<AtomicBool>,
        progress: Arc<native::solve::Progress>,
        workers: usize,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let execution = Execution {
            cancel: flag,
            started: Instant::now(),
            time_limit: self.profile.solver.controls.time_limit,
            progress,
            // Adjoint checkpoints are foreign allocations charged to the fit's allowance.
            memory: Some(
                self.runtime
                    .shared
                    .budget()
                    .math
                    .foreign_allowance(&self.profile.solver.controls),
            ),
        };
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
                    settings: &self.profile.solver.backend,
                    controls: &self.profile.solver.controls,
                    accuracy: &self.accuracy,
                    execution: execution.clone(),
                    tolerances: &self.tolerances,
                    normalization: &self.normalization,
                    compatibility: Compatibility {
                        layout: self.key,
                        profile: self.profile_key.as_id(),
                        data: self.source_identity,
                        backend,
                    },
                    warm: None,
                },
                &mut native::execution::Retained::default(),
                native::execution::Nlp {
                    oracle: Box::new(oracle),
                    initial: &self.initial,
                    presolve: &self.profile.solver.presolve,
                    intent: self.profile.solver.intent,
                    sense: pse_math::binding::ObjectiveSense::Minimize,
                    limit: self.profile.max_cells,
                    // An exact-Hessian fit reads its covariance from its own KKT analysis:
                    // the inverse reduced Hessian over its parameter columns (ADR-0118 item 8).
                    analysis: native::execution::Analysis {
                        inverse_reduced_hessian: (self.profile.solver.controls.hessian
                            == HessianMode::Exact)
                            .then(|| self.free().map(|(_, col)| col).collect::<Vec<_>>())
                            .filter(|columns| !columns.is_empty()),
                        ..native::execution::Analysis::for_intent(self.profile.solver.intent)
                    },
                },
            )?;
            let candidate = report.candidate.as_ref().map(|c| c.primal.clone());
            (Some(report), candidate)
        };
        let mut report = FitReport {
            checks: vec![],
            reports: vec![],
            checks_complete: false,
            validation_error: None,
            solve,
            hessian: self.profile.solver.controls.hessian,
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
        self.derive(&mut report, route, &execution, workers);
        Ok(report)
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
        if let Some(x) = report.candidate.as_ref() {
            // Fresh final evaluation is independent of native callback cache and candidate status.
            let mut final_oracle = FitOracle::new(self.clone(), execution)?;
            match final_oracle.evaluate(x) {
                Err(e) => report.diagnostic = Some(FitDiagnostic::new(FitRule::FinalEvaluation, e)),
                Ok(point) => {
                    report.trajectories = point.trajectories.clone();
                    report.constraint_values = point.constraints.clone();
                    let residuals = self
                        .measurements
                        .iter()
                        .zip(&point.predictions)
                        .filter(|(o, _)| o.included)
                        .map(|(o, v)| FitOracle::residual(o, *v).map(|v| v.0))
                        .collect::<Result<Vec<_>, _>>()?;
                    let objective =
                        0.5 * faer::col::ColRef::from_slice(&residuals).squared_norm_l2();
                    if !objective.is_finite() {
                        report.diagnostic = Some(FitDiagnostic::new(
                            FitRule::ObjectiveOverflow,
                            ProblemError::numerical("fresh fitting objective overflow"),
                        ));
                        return Ok(());
                    }
                    report.objective = Some(objective);
                    report.quality = Some(native::quality::observed(
                        &self.contract,
                        &self.bounds,
                        x,
                        &point.constraints,
                        &self.tolerances,
                    )?);
                    report.predictions = point
                        .predictions
                        .iter()
                        .zip(&self.measurements)
                        .map(|(v, o)| o.included.then_some(*v))
                        .collect();
                    match final_oracle.response_rank(x) {
                        Ok(diagnostic) => {
                            report.responses = Some(diagnostic.responses);
                            report.singular_values = diagnostic.singular_values;
                            report.directions = Some(diagnostic.directions);
                            report.rank = Some(diagnostic.rank);
                        }
                        Err(e) => {
                            report.diagnostic = Some(FitDiagnostic::new(FitRule::ResponseRank, e));
                        }
                    }
                }
            }
        }
        Ok(())
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
impl FitOracle {
    fn response_rank(&mut self, x: &[f64]) -> Result<RankDiagnostic, ProblemError> {
        // Rank and responses need the response Jacobian: a gradient-only fit reruns the
        // forward sensitivities once, at the candidate (PS-12).
        if self.derivatives != FitDerivatives::Responses {
            self.derivatives = FitDerivatives::Responses;
            self.point = None;
        }
        self.evaluate(x)?;
        let p = &self.prepared;
        let point = self
            .point
            .as_ref()
            .ok_or_else(|| ProblemError::internal("fit point"))?;
        let np = p.parameter_columns.iter().filter(|v| v.is_some()).count();
        let local_dense = p
            .experiments
            .iter()
            .try_fold(0usize, |cells, experiment| {
                let Experiment::Steady(s) = experiment else {
                    return Some(cells);
                };
                cells.checked_add(s.local_states.checked_mul(s.local_states + np)?)
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
                    .and_then(|b| b.checked_add(n.checked_mul(n)?.checked_mul(128)?));
                scratch = scratch.max(rank_scratch(n, n, false).size_bytes());
            }
        }
        // Vector bookkeeping and opaque library metadata, separate from numeric
        // matrices and library-declared scratch; this is not an RSS measurement.
        let bytes = bytes
            .and_then(|b| b.checked_add(scratch))
            .and_then(|b| b.checked_add((rows + np).checked_mul(64)?))
            .and_then(|b| b.checked_add(4 << 20))
            .filter(|b| *b <= limit)
            .ok_or_else(|| {
                ProblemError::memory("local dense response diagnostic memory allowance")
            })?;
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new("fit:response-diagnostic")
                .register(&p.runtime.shared.pool());
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
        reservation
            .try_grow(
                bytes
                    .checked_add(matching_stack)
                    .ok_or_else(|| ProblemError::memory("response structural stack extent"))?,
            )
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
                        .map(|i| p.tolerances.variables[s.coordinates[GlobalCol::new(i)].1.get()])
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
                let qualified_response = native::square_response::response(
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
                    || {
                        Ok((
                            Mat::from_fn(nx, nx, |i, j| {
                                jac.get(s.constraints[i].0.get(), j).copied().unwrap_or(0.)
                            }),
                            Mat::from_fn(nx, np, |i, j| {
                                point
                                    .jacobian
                                    .matrix()
                                    .get(s.constraints[i].1.get(), free[j].1.get())
                                    .copied()
                                    .unwrap_or(0.)
                            }),
                        ))
                    },
                )
                .map_err(|e| ProblemError::numerical(e.to_string()))?;
                let dx = qualified_response.values;
                for (i, o) in p
                    .measurements
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| o.experiment == ei && o.included)
                {
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
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::tests::{compiler_profile, id, physical, runtime};
    fn source(fixed: bool) -> crate::workflow::ModelingPackage {
        source_body(
            fixed,
            "param p: Scalar = 2; let y: Scalar = p*p; annotation check p(p > 0);",
        )
    }
    fn source_body(fixed: bool, body: &str) -> crate::workflow::ModelingPackage {
        source_text(fixed, &format!("package p {{ def Root {{ {body} }} }}"))
    }
    fn source_text(fixed: bool, text: &str) -> crate::workflow::ModelingPackage {
        source_measured(fixed, text, 3., 2.)
    }
    fn source_measured(
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
            .unwrap()
            .with_fit_declarations(data)
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
            },
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100000,
            derivatives: FitDerivatives::Responses,
            uncertainty: None,
        }
    }
    #[tokio::test]
    async fn prepared_fit_clones_share_the_original_reservation() {
        // Ownership is independent of native backend availability.
        let revision = source(true);
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

        let pressure = datafusion::execution::memory_pool::MemoryConsumer::new("test:fit-pressure")
            .register(&pool);
        let limit = revision.runtime.shared.budget().memory_limit_bytes.get();
        pressure.try_grow(limit - pool.reserved()).unwrap();
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
        let ex = Execution::new(Arc::new(AtomicBool::new(false)), &p.profile.solver.controls);
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
            let ex = Execution::new(Arc::new(AtomicBool::new(false)), &p.profile.solver.controls);
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
        let ex = Execution::new(Arc::new(AtomicBool::new(false)), &p.profile.solver.controls);
        let mut oracle = FitOracle::new(p, ex).unwrap();
        assert!(oracle.response_rank(&[2.0]).is_err());
        assert!((oracle.objective(&[2.0]).unwrap() - 0.5).abs() < 1e-12);
        let mut gradient = [0.0];
        oracle.gradient(&[2.0], &mut gradient).unwrap();
        assert!((gradient[0] - 4.0).abs() < 1e-12);
    }
    #[tokio::test]
    async fn all_fixed_fit_uses_joined_direct_evaluation_and_retained_sources() {
        let p = source(true)
            .prepare_fit(
                id(32).into(),
                profile(true),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(p.route(), native::routing::Route::Constant);
        let handle = p.start().unwrap();
        let a = handle.wait().await.unwrap();
        let b = handle.wait().await.unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        let crate::workflow::RunReport::Fit(r) = a.report().unwrap() else {
            panic!()
        };
        assert!(r.solve.is_none());
        assert_eq!(r.predictions, vec![Some(4.0)]);
        let table = a.table("runtime.fit_observations").unwrap();
        assert_eq!(table.batch().num_rows(), 1);
        assert_eq!(a.table("authored.fit_cases").unwrap().batch().num_rows(), 1);
        drop((a, b, handle, p));
        assert_eq!(table.batch().num_rows(), 1);
    }
    #[tokio::test]
    async fn all_fixed_required_presolve_is_refused_and_scales_are_checked() {
        let revision = source(true);
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
    #[tokio::test]
    async fn variable_fit_requires_a_linked_adapter() {
        let error = source(false)
            .prepare_fit(
                id(32).into(),
                profile(false),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(
                error,
                WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                    ProblemError::Unavailable {
                        backend: Backend::Ipopt,
                        ..
                    }
                ))
            ),
            "{error:?}"
        );
    }
    #[tokio::test]
    async fn invalid_uncertainty_and_unbound_observations_fail_admission() {
        let b = source_measured(
            false,
            "package p {def Root {param p:Scalar=2;let y:Scalar=p*p;}}",
            3.,
            0.,
        );
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
        let mut b = source(false);
        Arc::make_mut(&mut b.fit_declarations).fits[0].observations[0].experiment_id =
            id(99).into();
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
        let b = source_body(false, body);
        let profile = profile(false);
        let fixed = source_body(true, body);
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
            assert!(
                matches!(
                    error,
                    WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                        ProblemError::Unavailable {
                            backend: Backend::Ipopt,
                            ..
                        }
                    ))
                ),
                "{error:?}"
            );
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
        let ex = Execution::new(Arc::new(AtomicBool::new(false)), &p.profile.solver.controls);
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
        );
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
        let mut package = source(true);
        Arc::make_mut(&mut package.fit_declarations).fits[0].observations[0].output_path =
            "p".into();
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
    #[tokio::test]
    async fn authored_fit_checks_can_reject_a_numerically_feasible_candidate() {
        let package = source_body(
            true,
            "param p: Scalar = 2; let y: Scalar = p*p; annotation check p(p > 3);",
        );
        let prepared = package
            .prepare_fit(
                id(32).into(),
                profile(true),
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
        let package = source(true);
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
        );
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
                .revision
                .checked()
                .measurement(
                    package.fit_declarations.fits[0].experiments[0].case_id,
                    id(31).into(),
                    "value",
                    Some("sigma")
                )
                .unwrap()
                .value,
            Some(3.)
        );
        let mut data = (*package.fit_declarations).clone();
        data.fits[0].parameters[0].value = 2.5;
        let changed = package.clone().with_fit_declarations(data).unwrap();
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
        let mut data = (*package.fit_declarations).clone();
        let binding = data.fits[0].experiments[0].bindings[0].clone();
        data.fits[0].experiments[0].bindings.push(binding);
        let invalid = package.with_fit_declarations(data).unwrap();
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
        let package = runtime().modeling_package(rows, physical).unwrap();
        let ipopt = |intent| SolverProfile {
            presolve: Default::default(),
            numerics: Default::default(),
            convexity: Default::default(),
            intent,
            selection: SolverSelection::Explicit(Backend::Ipopt),
            controls: Controls::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
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
