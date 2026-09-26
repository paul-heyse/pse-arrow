// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin experiment composition; compiled library derivatives own local mathematics.
use super::*;
use faer::{
    Mat,
    sparse::{SparseColMat, SymbolicSparseColMatRef},
};
#[cfg(any(test, feature = "solver-ipopt", feature = "solver-pounce"))]
use native::solve::Backend;
use native::{
    NlpOracle, ProblemError,
    solve::{Compatibility, Execution},
};
use pse_math::{assembly::CaseWorker, sparse::AssemblyMatrix};
use std::{sync::atomic::AtomicBool, time::Instant};
#[derive(Debug)]
struct Point {
    physical: Vec<super::super::BalanceCheck>,
    x: Vec<f64>,
    predictions: Vec<f64>,
    responses: AssemblyMatrix,
    constraints: Vec<f64>,
    jacobian: AssemblyMatrix,
    blocks: Vec<Option<SparseColMat<usize, f64>>>,
}
#[derive(Debug)]
struct FitOracle {
    prepared: Arc<FitProblem>,
    workers: Vec<Option<CaseWorker>>,
    execution: Execution,
    hessian: Option<AssemblyMatrix>,
    gram: Option<sparse::GramWorker>,
    point: Option<Point>,
}
struct RankDiagnostic {
    responses: pse_columnar::Leased<Mat<f64>>,
    singular_values: Vec<f64>,
    rank: usize,
}
fn error(message: impl Into<String>) -> ProblemError {
    ProblemError::Contract(message.into())
}
impl FitOracle {
    fn new(p: impl Into<Arc<FitProblem>>, execution: Execution) -> Result<Self, ProblemError> {
        let p = p.into();
        let workers = p
            .experiments
            .iter()
            .map(|e| match e {
                Experiment::Steady(s) => {
                    let providers = p
                        .revision
                        .0
                        .providers
                        .values()
                        .map(|p| p.registration.clone())
                        .map(|r| {
                            r.worker()
                                .map(|w| (r.spec().key(), w))
                                .map_err(|e| error(e.to_string()))
                        })
                        .collect::<Result<_, _>>()?;
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
            prepared: p,
            workers,
            execution,
            hessian,
            gram,
            point: None,
        })
    }
    fn evaluate(&mut self, x: &[f64]) -> Result<&Point, ProblemError> {
        if self.execution.stopped().is_some() {
            return Err(error("fitting cancelled or deadline exceeded"));
        }
        let p = &self.prepared;
        let n = p.contract.variables.len();
        if x.len() != n || x.iter().any(|v| !v.is_finite()) {
            return Err(error("fit trial coordinates"));
        }
        if self
            .point
            .as_ref()
            .is_some_and(|v| v.x.iter().zip(x).all(|(a, b)| a.to_bits() == b.to_bits()))
        {
            return self.point.as_ref().ok_or_else(|| error("fit point cache"));
        }
        self.point = None;
        let mut point = Point {
            physical: vec![],
            x: x.to_vec(),
            predictions: vec![0.0; p.measurements.len()],
            responses: p.layout.responses.clone(),
            constraints: vec![0.0; p.contract.rows.len()],
            jacobian: p.layout.constraints.clone(),
            blocks: Vec::new(),
        };
        for (ei, e) in p.experiments.iter().enumerate() {
            match e {
                Experiment::Steady(s) => {
                    let worker = self.workers[ei]
                        .as_mut()
                        .ok_or_else(|| error("missing steady worker"))?;
                    let mut values = s.values.clone();
                    for &(id, col) in &s.coordinates {
                        values.scalars.insert(id, x[col]);
                    }
                    let outputs = worker.constraints(&values)?;
                    let sources = worker.constraint_sources()?;
                    let case = p.declaration.experiments[ei].case_id;
                    for b in p
                        .revision
                        .0
                        .sources
                        .balances
                        .iter()
                        .filter(|b| b.case_id == case)
                    {
                        point.physical.push(super::super::BalanceCheck {
                            balance: b.balance_id,
                            experiment: ei,
                            sample: 0,
                            time: None,
                            value: super::super::balances::closure(b, &sources)
                                .map_err(|e| e.to_string()),
                        });
                    }
                    let j = worker.jacobian(&values)?;
                    for &(row, global) in &s.constraints {
                        point.constraints[global] = outputs[row];
                    }
                    let mapping = &p.layout.mappings[ei];
                    for &(local, global) in &mapping.constraints {
                        point.jacobian.add(global, j.val()[local])?;
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
                        let mut params = s.parameters.clone();
                        for (j, id) in s.declaration.parameters.iter().enumerate() {
                            if let Some(k) = p
                                .declaration
                                .parameters
                                .iter()
                                .position(|v| v.symbol_id == *id)
                            {
                                let v = p.parameter_columns[k]
                                    .map_or(p.declaration.parameters[k].value, |c| x[c]);
                                let conversion = s.conversions[s.state_ports.len() + j];
                                params[j] = v * conversion.scale + conversion.offset;
                            }
                        }
                        // Integrations borrow the admitted outer worker; never enqueue nested native jobs.
                        let mut profile = s.profile.clone();
                        profile.time_limit = profile.time_limit.min(
                            self.execution
                                .time_limit
                                .saturating_sub(self.execution.started.elapsed()),
                        );
                        let mut worker = s.worker(self.execution.cancel.clone())?;
                        let report = native::dynamics::integrate(
                            &mut worker,
                            &profile,
                            &params,
                            self.execution.cancel.clone(),
                        )?;
                        if report.termination != native::dynamics::Termination::Completed {
                            return Err(report.error.unwrap_or_else(|| {
                                error(format!(
                                    "incomplete fit integration: {:?}",
                                    report.termination
                                ))
                            }));
                        }
                        for (j, b) in s.contract.balances.iter().enumerate() {
                            for (i, sample) in report.samples.iter().enumerate() {
                                point.physical.push(super::super::BalanceCheck {
                                    balance: b.id,
                                    experiment: ei,
                                    sample: i,
                                    time: Some(sample.time),
                                    value: super::super::balances::dynamic_closure(
                                        b, j, &report, sample,
                                    )
                                    .map_err(|e| e.to_string()),
                                });
                            }
                        }
                        for (i, o) in p
                            .measurements
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| o.experiment == ei && o.included)
                        {
                            let sample = o
                                .sample_index
                                .and_then(|i| report.samples.get(i))
                                .ok_or_else(|| error("missing prepared transient sample"))?;
                            point.predictions[i] = sample.outputs[o.row];
                            for term in p.layout.mappings[ei]
                                .responses
                                .iter()
                                .filter(|t| t.observation == i)
                            {
                                let scale = s.conversions[s.state_ports.len() + term.local].scale;
                                point.responses.add(
                                    term.contribution,
                                    sample.output_sensitivities[o.row * params.len() + term.local]
                                        * scale,
                                )?;
                            }
                        }
                        point.blocks.push(None);
                    }
                    #[cfg(not(feature = "solver-diffsol"))]
                    {
                        let _ = s;
                        return Err(error("Diffsol not linked"));
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
            return Err(error("nonfinite fit output or response"));
        }
        self.execution.progress.push(native::solve::Event {
            phase: "fit.evaluation".into(),
            elapsed: self.execution.started.elapsed(),
            values: BTreeMap::from([(
                "experiments".into(),
                native::solve::Metric::Integer(p.experiments.len() as i64),
            )]),
        });
        self.point = Some(point);
        self.point
            .as_ref()
            .ok_or_else(|| error("fit point publication"))
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
        let point = self.point.as_ref().ok_or_else(|| error("fit point"))?;
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
            return Err(error("fit objective overflow"));
        }
        Ok(sum)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let p = self.evaluate(x)?;
        if out.len() != p.constraints.len() {
            return Err(error("fit constraint extent"));
        }
        out.copy_from_slice(&p.constraints);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.evaluate(x)?;
        if out.len() != x.len() {
            return Err(error("fit gradient extent"));
        }
        let point = self.point.as_ref().ok_or_else(|| error("fit point"))?;
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
        for (j, v) in out.iter_mut().enumerate() {
            let value = result[(j, 0)];
            if !value.is_finite() {
                return Err(error("nonfinite loss gradient"));
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
            .ok_or_else(|| error("fit point"))?
            .jacobian
            .matrix()
            .val();
        if out.len() != values.len() {
            return Err(error("fit sparse Jacobian extent"));
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
            return Err(error("fit Lagrangian demand"));
        }
        let p = &self.prepared;
        let point = self.point.as_ref().ok_or_else(|| error("fit point"))?;
        let h = self
            .hessian
            .as_mut()
            .ok_or_else(|| error("Hessian not prepared"))?;
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
        self.gram
            .as_mut()
            .ok_or_else(|| error("Gram not prepared"))?
            .refill(&point.responses, &weights, objective_weight, h)?;
        for (ei, e) in p.experiments.iter().enumerate() {
            let Experiment::Steady(s) = e else {
                return Err(error("transient exact Hessian unavailable"));
            };
            let mut lambda = vec![0.0; s.case.assembly.structure().rows().len()];
            for &(local, global) in &s.constraints {
                lambda[local] += multipliers[global];
            }
            for (i, o) in p
                .measurements
                .iter()
                .enumerate()
                .filter(|(_, o)| o.experiment == ei && o.included)
            {
                let (r, w) = Self::residual(o, point.predictions[i])?;
                lambda[o.row] += objective_weight * r * w;
            }
            let mut values = s.values.clone();
            for &(id, c) in &s.coordinates {
                values.scalars.insert(id, x[c]);
            }
            let local = self.workers[ei]
                .as_mut()
                .ok_or_else(|| error("steady Hessian worker"))?
                .hessian(&values, 0.0, &lambda)?;
            for &(source, target) in &p.layout.mappings[ei].hessian {
                h.add(target, local.val()[source])?;
            }
        }
        if out.len() != h.matrix().val().len() {
            return Err(error("fit sparse Hessian extent"));
        }
        out.copy_from_slice(h.matrix().val());
        Ok(())
    }
}
impl PreparedFit {
    pub(crate) fn execute(
        &self,
        flag: Arc<AtomicBool>,
        progress: Arc<native::solve::Progress>,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        #[cfg(feature = "solver-pounce")]
        if self.route == native::routing::Route::Native(Backend::Pounce) {
            return native::pounce::with_threads(
                self.problem.profile.solver.controls.threads,
                self.problem.revision.0.runtime.native().stack_bytes(),
                || self.execute_inner(flag, progress),
            );
        }
        self.execute_inner(flag, progress)
    }
    fn execute_inner(
        &self,
        flag: Arc<AtomicBool>,
        progress: Arc<native::solve::Progress>,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let execution = Execution {
            cancel: flag,
            started: Instant::now(),
            time_limit: self.problem.profile.solver.controls.time_limit,
            progress,
        };
        let mut oracle = FitOracle::new(self.problem.clone(), execution.clone())?;
        let (solve, candidate) = if self.problem.initial.is_empty() {
            oracle.objective(&[])?;
            (None, Some(vec![]))
        } else {
            let native::routing::Route::Native(backend) = self.route else {
                return Err(error("nonempty fit has no native route").into());
            };
            let stamp = Compatibility {
                layout: self.problem.key,
                data: self.problem.revision.identity(),
                backend,
            };
            let pipeline = native::presolve::Pipeline::new(
                Box::new(oracle),
                &self.problem.initial,
                &self.problem.profile.solver.presolve,
                &self.problem.tolerances,
                None,
                execution.clone(),
                None,
                stamp,
                self.problem.profile.max_cells,
            )?;
            #[cfg(any(feature = "solver-ipopt", feature = "solver-pounce"))]
            let mut pipeline = pipeline;
            let report = if let Some(r) =
                pipeline.terminal_report(pse_math::binding::ObjectiveSense::Minimize)?
            {
                r
            } else {
                match backend {
                    #[cfg(feature = "solver-ipopt")]
                    Backend::Ipopt => {
                        let mut transport = pipeline.take_oracle()?;
                        let controls = &self.problem.profile.solver.controls;
                        let scales = NlpOracle::scaling(&transport).cloned();
                        native::ipopt::Session::new().solve(
                            &mut transport,
                            pipeline.initial(),
                            pse_math::binding::ObjectiveSense::Minimize,
                            controls,
                            execution.clone(),
                            &pipeline.tolerances(&self.problem.tolerances),
                            scales.as_ref(),
                            None,
                            pipeline.native_compatibility().clone(),
                        )?
                    }
                    #[cfg(feature = "solver-pounce")]
                    Backend::Pounce => {
                        let transport = pipeline.take_oracle()?;
                        let controls = &self.problem.profile.solver.controls;
                        let (method, linear) = match self.problem.profile.solver.backend.clone() {
                            crate::math::solves::BackendSettings::Default => (
                                native::pounce::Method::InteriorPoint,
                                native::pounce::LinearSettings::default(),
                            ),
                            crate::math::solves::BackendSettings::Pounce { method, linear } => {
                                (method, linear)
                            }
                            _ => return Err(error("wrong fitting POUNCE settings").into()),
                        };
                        native::pounce::Session::new().solve(
                            Box::new(transport),
                            pipeline.initial(),
                            pse_math::binding::ObjectiveSense::Minimize,
                            controls,
                            method,
                            linear,
                            execution.clone(),
                            &pipeline.tolerances(&self.problem.tolerances),
                            None,
                            pipeline.native_compatibility().clone(),
                        )?
                    }
                    _ => return Err(error("native fitting backend unavailable").into()),
                }
            };
            let mut report = pipeline.finish(
                report,
                &self.problem.tolerances,
                pse_math::binding::ObjectiveSense::Minimize,
            );
            native::quality::record_kkt(
                &mut report,
                &self.problem.normalization,
                &self.problem.profile.solver.controls.accuracy,
            );
            native::quality::qualify(&mut report, &self.problem.profile.solver.controls.accuracy);
            let candidate = report.candidate.as_ref().map(|c| c.primal.clone());
            (Some(report), candidate)
        };
        let mut report = FitReport {
            physical: vec![],
            solve,
            candidate,
            quality: None,
            constraint_values: vec![],
            objective: None,
            predictions: vec![None; self.problem.measurements.len()],
            responses: None,
            singular_values: vec![],
            rank: None,
            diagnostic: None,
        };
        if let Some(x) = report.candidate.as_ref() {
            // Fresh final evaluation is independent of native callback cache and candidate status.
            let mut final_oracle = FitOracle::new(self.problem.clone(), execution)?;
            match final_oracle.evaluate(x) {
                Err(e) => report.diagnostic = Some(e.to_string()),
                Ok(point) => {
                    report.physical = point.physical.clone();
                    report.constraint_values = point.constraints.clone();
                    let residuals = self
                        .problem
                        .measurements
                        .iter()
                        .zip(&point.predictions)
                        .filter(|(o, _)| o.included)
                        .map(|(o, v)| FitOracle::residual(o, *v).map(|v| v.0))
                        .collect::<Result<Vec<_>, _>>()?;
                    let objective =
                        0.5 * faer::col::ColRef::from_slice(&residuals).squared_norm_l2();
                    if !objective.is_finite() {
                        report.diagnostic = Some("fresh fitting objective overflow".into());
                        return Ok(report);
                    }
                    report.objective = Some(objective);
                    report.quality = Some(native::quality::observed(
                        &self.problem.contract,
                        &self.problem.bounds,
                        x,
                        &point.constraints,
                        &self.problem.tolerances,
                    )?);
                    report.predictions = point
                        .predictions
                        .iter()
                        .zip(&self.problem.measurements)
                        .map(|(v, o)| o.included.then_some(*v))
                        .collect();
                    match final_oracle.response_rank(x) {
                        Ok(diagnostic) => {
                            report.responses = Some(diagnostic.responses);
                            report.singular_values = diagnostic.singular_values;
                            report.rank = Some(diagnostic.rank);
                        }
                        Err(e) => report.diagnostic = Some(e.to_string()),
                    }
                }
            }
        }
        Ok(report)
    }
}
fn singular_values(a: &Mat<f64>, limit: usize) -> Result<Vec<f64>, ProblemError> {
    if a.as_ref()
        .col_iter()
        .any(|c| c.iter().any(|v| !v.is_finite()))
    {
        return Err(error("nonfinite local rank input"));
    }
    use faer::{
        Par,
        diag::Diag,
        dyn_stack::{MemBuffer, MemStack},
        linalg::svd,
    };
    let req = rank_scratch(a.nrows(), a.ncols());
    if req.size_bytes() > limit {
        return Err(error("rank scratch allowance"));
    }
    let mut buffer = MemBuffer::try_new(req).map_err(|e| error(e.to_string()))?;
    let mut s = Diag::<f64>::zeros(a.nrows().min(a.ncols()));
    svd::svd(
        a.as_ref(),
        s.as_mut(),
        None,
        None,
        Par::Seq,
        MemStack::new(&mut buffer),
        Default::default(),
    )
    .map_err(|e| error(format!("{e:?}")))?;
    Ok(s.column_vector().iter().copied().collect())
}
fn rank_scratch(rows: usize, cols: usize) -> faer::dyn_stack::StackReq {
    use faer::linalg::svd::{self, ComputeSvdVectors};
    svd::svd_scratch::<f64>(
        rows,
        cols,
        ComputeSvdVectors::No,
        ComputeSvdVectors::No,
        faer::Par::Seq,
        Default::default(),
    )
}
fn response_scratch(n: usize, np: usize) -> faer::dyn_stack::StackReq {
    use faer::{
        Par,
        linalg::lu::partial_pivoting::{factor, solve},
    };
    factor::lu_in_place_scratch::<usize, f64>(n, n, Par::Seq, Default::default())
        .or(solve::solve_in_place_scratch::<usize, f64>(n, np, Par::Seq))
}
fn solve_regular(a: &Mat<f64>, mut rhs: Mat<f64>, limit: usize) -> Result<Mat<f64>, ProblemError> {
    use faer::{
        Par,
        dyn_stack::{MemBuffer, MemStack},
        linalg::lu::partial_pivoting::{factor, solve},
    };
    let n = a.nrows();
    let mut lu = a.clone();
    let mut perm = vec![0usize; n];
    let mut inverse = vec![0usize; n];
    let req = response_scratch(n, rhs.ncols());
    if req.size_bytes() > limit {
        return Err(error("response solve scratch allowance"));
    }
    let mut memory = MemBuffer::try_new(req).map_err(|e| error(e.to_string()))?;
    let (_, permutation) = factor::lu_in_place(
        lu.as_mut(),
        &mut perm,
        &mut inverse,
        Par::Seq,
        MemStack::new(&mut memory),
        Default::default(),
    );
    solve::solve_in_place(
        lu.as_ref(),
        lu.as_ref(),
        permutation,
        rhs.as_mut(),
        Par::Seq,
        MemStack::new(&mut memory),
    );
    if rhs
        .as_ref()
        .col_iter()
        .any(|c| c.iter().any(|v| !v.is_finite()))
    {
        return Err(error("nonfinite implicit response solve"));
    }
    Ok(rhs)
}
/// Normwise backward error in the same scaled coordinates used for rank admission.
fn check_response(a: &Mat<f64>, x: &Mat<f64>, b: &Mat<f64>) -> Result<(), ProblemError> {
    let mut residual = -b;
    faer::linalg::matmul::matmul(
        residual.as_mut(),
        faer::Accum::Add,
        a.as_ref(),
        x.as_ref(),
        1.0,
        faer::Par::Seq,
    );
    let numerator = residual.as_ref().norm_l2();
    let denominator = a.as_ref().norm_l2() * x.as_ref().norm_l2() + b.as_ref().norm_l2();
    let error_bound = 64.0 * a.nrows().max(1) as f64 * f64::EPSILON;
    let backward_error = if denominator == 0.0 {
        numerator
    } else {
        numerator / denominator
    };
    if !denominator.is_finite() || !backward_error.is_finite() || backward_error > error_bound {
        return Err(error(format!(
            "implicit response backward error {backward_error} exceeds {error_bound}"
        )));
    }
    Ok(())
}
impl FitOracle {
    fn response_rank(&mut self, x: &[f64]) -> Result<RankDiagnostic, ProblemError> {
        self.evaluate(x)?;
        let p = &self.prepared;
        let point = self.point.as_ref().ok_or_else(|| error("fit point"))?;
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
            .ok_or_else(|| error("local response diagnostic extent"))?;
        if local_dense > p.profile.max_cells {
            return Err(error(
                "local dense response diagnostic allowance; fit candidate remains available",
            ));
        }
        // Dense diagnostics are optional: reserve independently from the sparse
        // solve, before allocating any matrix. Failure does not lose the candidate.
        let limit = p.revision.0.runtime.shared.budget().math.worker_bytes;
        if local_dense.checked_mul(8).is_none_or(|bytes| bytes > limit) {
            return Err(error("local dense response diagnostic memory allowance"));
        }
        use faer::linalg::temp_mat_scratch;
        let rows = p.measurements.len();
        let mut bytes = temp_mat_scratch::<f64>(rows, np)
            .size_bytes()
            .checked_mul(2);
        let mut scratch = rank_scratch(rows, np).size_bytes();
        for experiment in &p.experiments {
            if let Experiment::Steady(s) = experiment {
                let n = s.local_states;
                // fx, scaled closure, LU; rhs, solve, residual, physical response.
                // temp_mat_scratch uses the same public faer alignment policy as
                // fresh Mat allocations. Each matrix here is created at final size.
                bytes = bytes
                    .and_then(|b| {
                        b.checked_add(temp_mat_scratch::<f64>(n, n).size_bytes().checked_mul(3)?)
                    })
                    .and_then(|b| {
                        b.checked_add(temp_mat_scratch::<f64>(n, np).size_bytes().checked_mul(4)?)
                    })
                    .and_then(|b| b.checked_add(n.checked_mul(4 * size_of::<usize>())?));
                scratch = scratch
                    .max(rank_scratch(n, n).size_bytes())
                    .max(response_scratch(n, np).size_bytes());
            }
        }
        // Vector bookkeeping and opaque library metadata, separate from numeric
        // matrices and library-declared scratch; this is not an RSS measurement.
        let bytes = bytes
            .and_then(|b| b.checked_add(scratch))
            .and_then(|b| b.checked_add((rows + np).checked_mul(64)?))
            .and_then(|b| b.checked_add(4 << 20))
            .filter(|b| *b <= limit)
            .ok_or_else(|| error("local dense response diagnostic memory allowance"))?;
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new("fit:response-diagnostic")
                .register(&p.revision.0.runtime.shared.pool());
        reservation
            .try_grow(bytes)
            .map_err(|e| error(e.to_string()))?;
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
                    .get(i, *col)
                    .copied()
                    .unwrap_or(0.0);
            }
        }
        for (ei, e) in p.experiments.iter().enumerate() {
            if let Experiment::Steady(s) = e {
                let nx = s.local_states;
                if s.constraints.len() != nx
                    || s.constraints
                        .iter()
                        .any(|(_, g)| p.bounds[*g].0 != p.bounds[*g].1)
                {
                    return Err(error(
                        "steady response needs square equality closure; fit NLP remains valid",
                    ));
                }
                if s.constraints.iter().any(|(_, g)| {
                    (point.constraints[*g] - p.bounds[*g].0).abs() > p.tolerances.rows[*g]
                }) {
                    return Err(error(
                        "steady response requires a feasible physical closure",
                    ));
                }
                if nx == 0 {
                    continue;
                }
                let jac = point.blocks[ei]
                    .as_ref()
                    .ok_or_else(|| error("steady response partials"))?;
                let fx = Mat::from_fn(nx, nx, |i, j| {
                    jac.get(s.constraints[i].0, j).copied().unwrap_or(0.0)
                });
                let scaled = Mat::from_fn(nx, nx, |i, j| {
                    fx[(i, j)] * p.tolerances.variables[s.coordinates[j].1]
                        / p.tolerances.rows[s.constraints[i].1]
                });
                let spectrum = singular_values(&scaled, bytes)?;
                if spectrum
                    .last()
                    .is_none_or(|last| *last <= spectrum[0] * p.profile.rank_tolerance)
                {
                    return Err(error(
                        "steady closure is locally rank deficient at the stated scaling/cutoff",
                    ));
                }
                let rhs = Mat::from_fn(nx, np, |i, j| {
                    -point
                        .jacobian
                        .matrix()
                        .get(s.constraints[i].1, free[j].1)
                        .copied()
                        .unwrap_or(0.0)
                        / p.tolerances.rows[s.constraints[i].1]
                });
                let scaled_dx = solve_regular(&scaled, rhs.clone(), bytes)?;
                check_response(&scaled, &scaled_dx, &rhs)?;
                let dx = Mat::from_fn(nx, np, |i, j| {
                    scaled_dx[(i, j)] * p.tolerances.variables[s.coordinates[i].1]
                });
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
        let spectrum = singular_values(&weighted, bytes)?;
        let cutoff = spectrum.first().copied().unwrap_or(0.0) * p.profile.rank_tolerance;
        let rank = spectrum.iter().filter(|s| **s > cutoff).count();
        let retained = temp_mat_scratch::<f64>(rows, np).size_bytes();
        let owner = pse_columnar::AllocationLease::new(reservation.split(retained));
        let response = pse_columnar::Leased::new(Arc::new(response), owner);
        Ok(RankDiagnostic {
            responses: response,
            singular_values: spectrum,
            rank,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::tests::{compiler_profile, declaration, id, physical, runtime};
    #[test]
    fn implicit_response_checks_scaled_backward_error() {
        let a = faer::mat![[2.0, 1.0], [1.0, 3.0]];
        let b = faer::mat![[4.0], [7.0]];
        let x = solve_regular(&a, b.clone(), 1024 * 1024).unwrap();
        check_response(&a, &x, &b).unwrap();
        assert!(check_response(&a, &faer::mat![[1.0], [1.0]], &b).is_err());
        check_response(&Mat::zeros(2, 2), &Mat::zeros(2, 1), &Mat::zeros(2, 1)).unwrap();
        assert!(check_response(&a, &faer::mat![[f64::NAN], [1.0]], &b).is_err());
    }
    fn source(fixed: bool) -> crate::workflow::ModelBuilder {
        let mut d = declaration();
        let v = d.cases[0].variables.remove(0);
        let unit = v.port.unit_id;
        d.cases[0].parameters.push(serde_json::from_value(serde_json::json!({"symbol_id":v.port.symbol_id,"quantity_id":v.port.quantity_id,"unit_id":unit})).unwrap());
        d.cases[0].rows[0].lower = None;
        d.cases[0].rows[0].upper = None;
        let mut b = crate::workflow::ModelBuilder::from_declaration(runtime(), d, physical());
        b.dataset(serde_json::from_value(serde_json::json!({"dataset_id":id(30),"name":"synthetic","source":"unit","content_hash":ContentHash::from_bytes([1;32])})).unwrap());
        b.observation(serde_json::from_value(serde_json::json!({"observation_id":id(31),"dataset_id":id(30),"target":"x squared","value":3.0,"unit_id":unit,"std_dev":2.0,"timestamp":null,"tag":null,"source_span":{"document_id":id(30),"start":0,"end":0}})).unwrap());
        b.fit(serde_json::from_value(serde_json::json!({"fit_id":id(32),"model_id":id(20),"parameters":[{"symbol_id":id(1),"fixed":fixed,"value":2.0,"lower":0.1,"upper":10.0,"scale":2.0}],"experiments":[{"experiment_id":id(33),"case_id":id(5),"dynamic_id":null}],"observations":[{"observation_id":id(31),"experiment_id":id(33),"output_id":id(4),"time":null,"included":true,"importance":4.0}]})).unwrap());
        b
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
                backend: crate::math::solves::BackendSettings::Default,
            },
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100000,
        }
    }
    #[tokio::test]
    async fn prepared_fit_clones_share_the_original_reservation() {
        // Ownership is independent of native backend availability.
        let revision = source(true).freeze().unwrap();
        let prepared = revision
            .prepare_fit(
                id(32),
                profile(false),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let pool = revision.0.runtime.shared.pool();
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
        let limit = revision.0.runtime.shared.budget().memory_limit_bytes.get();
        pressure.try_grow(limit - pool.reserved()).unwrap();
        assert!(
            revision
                .prepare_fit(
                    id(32),
                    profile(false),
                    compiler_profile(),
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
            .freeze()
            .unwrap()
            .prepare_fit_problem(
                id(32),
                profile(false),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
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
        let pool = o.prepared.revision.0.runtime.shared.pool();
        let before = pool.reserved();
        let RankDiagnostic {
            responses: j,
            singular_values: s,
            rank: r,
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
        let limit = o
            .prepared
            .revision
            .0
            .runtime
            .shared
            .budget()
            .memory_limit_bytes
            .get();
        pressure.try_grow(limit - pool.reserved()).unwrap();
        assert!(o.response_rank(&[2.0]).is_err());
        assert!((o.objective(&[2.0]).unwrap() - 0.5).abs() < 1e-12);
        drop(pressure);
        assert!(o.response_rank(&[2.0]).is_ok());
    }
    #[tokio::test]
    async fn sparse_fit_admission_tracks_support_and_refills_duplicates() {
        let p = source(false)
            .freeze()
            .unwrap()
            .prepare_fit_problem(
                id(32),
                profile(false),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        // One thousand independent coordinates need linear derivative storage.
        let Experiment::Steady(base) = &p.experiments[0] else {
            panic!()
        };
        let experiments = (0..1000)
            .map(|column| {
                let mut s = base.clone();
                s.coordinates[0].1 = column;
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
            &[id(1)],
            &[Some(0)],
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
                &[id(1)],
                &[Some(0)],
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
            &[id(1)],
            &[Some(0)],
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
            response.add(0, value).unwrap();
            response.add(0, 1.0).unwrap();
            worker.refill(&response, &[2.0], 0.5, &mut hessian).unwrap();
            assert!((hessian.matrix().val()[0] - 2.0 * (value + 1.0).powi(2)).abs() < 1e-12);
        }
    }
    #[tokio::test]
    async fn bounded_rank_diagnostic_does_not_disable_sparse_candidate_evaluation() {
        let mut p = source(false)
            .freeze()
            .unwrap()
            .prepare_fit_problem(
                id(32),
                profile(false),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
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
            .freeze()
            .unwrap()
            .prepare_fit(
                id(32),
                profile(true),
                compiler_profile(),
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
        let revision = source(true).freeze().unwrap();
        let mut controls = profile(true);
        controls.solver.presolve = native::presolve::Policy::from_native_options(
            &Default::default(),
            [native::presolve::Pass::AffineElimination].into(),
        )
        .unwrap();
        let error = revision
            .prepare_fit(
                id(32),
                controls,
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, WorkflowError::Contract(ref message) if message == "all-fixed fitting evaluates directly and cannot apply required native presolve passes")
        );
    }
    #[cfg(not(feature = "solver-ipopt"))]
    #[tokio::test]
    async fn variable_fit_requires_a_linked_adapter() {
        let error = source(false)
            .freeze()
            .unwrap()
            .prepare_fit(
                id(32),
                profile(false),
                compiler_profile(),
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
        let mut b = source(false);
        b.sources.observations[0].std_dev = Some(0.0);
        let r = b.freeze().unwrap();
        let error = r
            .prepare_fit_problem(
                id(32),
                profile(false),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, WorkflowError::Contract(ref message) if message == "included observations require finite values, positive difference-unit standard deviations and importance")
        );
        let mut b = source(false);
        b.sources.fits[0].observations[0].experiment_id = id(99);
        let r = b.freeze().unwrap();
        let error = r
            .prepare_fit_problem(
                id(32),
                profile(false),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, WorkflowError::Contract(ref message) if message == "fit observations or integration profile ownership")
        );
    }
    #[tokio::test]
    async fn steady_response_solves_the_compiled_implicit_closure() {
        let mut b = source(false);
        let d = b.declaration_mut();
        let qty = d.cases[0].parameters[0].quantity_id;
        let unit = d.cases[0].parameters[0].unit_id;
        d.definitions[0].sources = vec!["state-parameter".into(), "state*state".into()];
        d.definitions[0].formals=serde_json::from_value(serde_json::json!([{"path":"state","quantity_id":qty},{"path":"parameter","quantity_id":qty}])).unwrap();
        d.cases[0].variables=serde_json::from_value(serde_json::json!([{"port":{"symbol_id":id(8),"quantity_id":qty,"unit_id":unit},"fixed":false,"domain":"continuous","lower":null,"upper":null}])).unwrap();
        d.cases[0].values.push(
            serde_json::from_value(serde_json::json!({"symbol_id":id(8),"value":2.0})).unwrap(),
        );
        d.cases[0].rows.push(
            serde_json::from_value(
                serde_json::json!({"row_id":id(7),"quantity_id":qty,"lower":0.0,"upper":0.0}),
            )
            .unwrap(),
        );
        d.cases[0].instances[0].slots=serde_json::from_value(serde_json::json!([{"source_id":id(8),"formal_quantity_id":qty,"formal_unit_id":unit},{"source_id":id(1),"formal_quantity_id":qty,"formal_unit_id":unit}])).unwrap();
        d.cases[0].instances[0].contributions=serde_json::from_value(serde_json::json!([{"output":0,"row_id":id(7),"scale":1.0},{"output":1,"row_id":id(4),"scale":1.0}])).unwrap();
        let profile = profile(false);
        let mut fixed = source(true);
        *fixed.declaration_mut() = b.declaration_mut().clone();
        let fixed = fixed.freeze().unwrap();
        let fixed_profile = profile.clone();
        let fixed_problem = fixed
            .prepare_fit_problem(
                id(32),
                fixed_profile.clone(),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        // Fixed fit parameters do not eliminate free experiment-local states.
        assert_eq!(fixed_problem.contract.variables.len(), 1);
        #[cfg(not(feature = "solver-ipopt"))]
        {
            let error = fixed
                .prepare_fit(
                    id(32),
                    fixed_profile,
                    compiler_profile(),
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
            .freeze()
            .unwrap()
            .prepare_fit_problem(
                id(32),
                profile,
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
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

    #[test]
    fn library_rank_and_regular_solve_have_independent_controls() {
        let a = Mat::from_fn(2, 2, |i, j| if i == j { 2.0 } else { 0.0 });
        let rhs = Mat::from_fn(2, 1, |i, _| 2.0 * (i + 1) as f64);
        let solved = solve_regular(&a, rhs, 1 << 20).unwrap();
        assert_eq!(solved[(0, 0)], 1.0);
        assert_eq!(solved[(1, 0)], 2.0);
        let singular = Mat::from_fn(2, 2, |_, _| 1.0);
        let s = singular_values(&singular, 1 << 20).unwrap();
        assert!(s[1] < 1e-12);
        assert!(singular_values(&a, 0).is_err());
    }
}
#[cfg(all(test, feature = "solver-diffsol"))]
mod composition_tests {
    use super::*;
    use crate::workflow::{
        dynamics::tests as dynamic,
        tests::{compiler_profile, id},
    };
    #[tokio::test]
    async fn mixed_shared_parameter_gradient_uses_inline_forward_sensitivities() {
        let mut b = dynamic::source();
        let mut steady = b.declaration_mut().cases[0].clone();
        steady.case_id = id(6);
        steady.variables[0].fixed = true;
        b.declaration_mut().cases.push(steady);
        let time_unit = b
            .physical_context()
            .quantities
            .quantity_type(pse_quantity::QuantityTypeId::from_id(id(63)))
            .unwrap()
            .canonical_unit
            .as_id();
        let neutral_unit = b
            .physical_context()
            .quantities
            .quantity_type(pse_quantity::standard::ids::quantity("neutral"))
            .unwrap()
            .canonical_unit
            .as_id();
        b.dataset(serde_json::from_value(serde_json::json!({"dataset_id":id(70),"name":"mixed","source":"unit","content_hash":ContentHash::from_bytes([1;32])})).unwrap());
        for (obs, value, unit) in [(71, 12.0, time_unit), (72, 1.0, neutral_unit)] {
            b.observation(serde_json::from_value(serde_json::json!({"observation_id":id(obs),"dataset_id":id(70),"target":"response","value":value,"unit_id":unit,"std_dev":1.0,"timestamp":null,"tag":null,"source_span":{"document_id":id(70),"start":0,"end":0}})).unwrap());
        }
        b.fit(serde_json::from_value(serde_json::json!({"fit_id":id(73),"model_id":id(20),"parameters":[{"symbol_id":id(3),"fixed":false,"value":2.0,"lower":0.1,"upper":10.0,"scale":1.0}],"experiments":[{"experiment_id":id(74),"case_id":id(5),"dynamic_id":id(50)},{"experiment_id":id(75),"case_id":id(6),"dynamic_id":null}],"observations":[{"observation_id":id(71),"experiment_id":id(74),"output_id":id(42),"time":1.0,"included":true,"importance":1.0},{"observation_id":id(72),"experiment_id":id(75),"output_id":id(40),"time":null,"included":true,"importance":1.0}]})).unwrap());
        let profile = FitProfile {
            solver: SolverProfile {
                presolve: Default::default(),
                numerics: Default::default(),
                convexity: Default::default(),
                intent: SolveIntent::Optimize,
                selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
                controls: native::solve::Controls {
                    hessian: HessianMode::LimitedMemory,
                    ..Default::default()
                },
                backend: crate::math::solves::BackendSettings::Default,
            },
            simulations: BTreeMap::from([(id(74), dynamic::profile())]),
            rank_tolerance: 1e-8,
            max_cells: 100000,
        };
        let revision = b.freeze().unwrap();
        let mut exact = profile.clone();
        exact.solver.controls.hessian = HessianMode::Exact;
        assert!(
            revision
                .prepare_fit_problem(
                    id(73),
                    exact,
                    compiler_profile(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
        let prepared = revision
            .prepare_fit_problem(
                id(73),
                profile,
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(prepared.contract.variables.len(), 1);
        let execution = Execution::new(
            Arc::new(AtomicBool::new(false)),
            &prepared.profile.solver.controls,
        );
        let mut o = FitOracle::new(prepared, execution).unwrap();
        assert!((o.objective(&[2.0]).unwrap() - 1.0).abs() < 1e-5);
        let mut g = [0.0];
        o.gradient(&[2.0], &mut g).unwrap();
        assert!((g[0] - 2.0).abs() < 1e-5);
        let RankDiagnostic {
            responses: j,
            singular_values: s,
            rank,
        } = o.response_rank(&[2.0]).unwrap();
        assert_eq!(rank, 1);
        assert!((j[(0, 0)] - 1.0).abs() < 1e-6);
        assert!((j[(1, 0)] - 1.0).abs() < 1e-6);
        assert!((s[0] - 2.0f64.sqrt()).abs() < 1e-6);
    }
}

#[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
#[path = "p09_tests.rs"]
mod p09_tests;
