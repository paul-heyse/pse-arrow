// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Coordinate transport only: reductions, derivative transforms and recovery belong to POUNCE.
use super::{Policy, Report, Scaling};
use crate::{
    NlpOracle, OracleContract, ProblemError,
    callback::CallbackState,
    nlp_pattern::Pattern,
    quality::{self, Tolerances},
    solve::{
        Assurance, Candidate, Compatibility, Execution, Metric, NativeTermination, SolveReport,
        Termination, WarmPayload, WarmStart,
    },
    tnlp::Adapter,
};
use pounce_nlp::{
    SolverReturn,
    expression_provider::ExpressionProvider,
    tnlp::{
        BoundsInfo, IndexStyle, IpoptCq, IpoptData, Solution, SparsityRequest, StartingPoint, TNLP,
    },
};
use pounce_presolve::{LinearEqElimTnlp, PresolveMap, PresolveTnlp};
use pse_math::{binding::ObjectiveSense, sparse::AssemblyMatrix};
use std::{cell::RefCell, rc::Rc};

/// Attempt-owned wrapper stack and original worker. Never stored in Salsa or sent across workers.
pub struct Pipeline {
    original: Rc<RefCell<Adapter>>,
    outer: Rc<RefCell<dyn TNLP>>,
    report: Report,
    compatibility: Compatibility,
    native: Compatibility,
    initial: Vec<f64>,
    warm: Option<WarmStart>,
    transport: Option<Transport>,
}
impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresolvePipeline")
            .field("report", &self.report)
            .finish_non_exhaustive()
    }
}
impl Pipeline {
    /// Inspect qualified passes and retained proof without consuming callback ownership.
    pub fn report(&self) -> &Report {
        &self.report
    }
    /// A library certificate or zero-dimensional projection needs no native solve.
    /// The result names that fact explicitly and is still independently observed.
    pub fn terminal_report(
        &self,
        sense: ObjectiveSense,
    ) -> Result<Option<SolveReport>, ProblemError> {
        if self.report.proof.is_none() && !self.initial.is_empty() {
            return Ok(None);
        }
        let original = self.original.borrow();
        let certified = self.report.proof.is_some();
        let mut report = SolveReport::new(
            self.compatibility.backend,
            original.oracle.contract(),
            NativeTermination {
                code: 0,
                name: if certified {
                    "PresolveCertifiedInfeasible"
                } else {
                    "PresolveZeroDimensional"
                }
                .into(),
                message: None,
                category: if certified {
                    Termination::Infeasible
                } else {
                    Termination::FeasibleOnly
                },
                assurance: Assurance::None,
            },
            &original.state.execution,
        );
        drop(original);
        report
            .metrics
            .insert("native.attempted".into(), Metric::Bool(false));
        if !certified {
            let objective = self
                .outer
                .borrow_mut()
                .eval_f(&[], true)
                .ok_or_else(|| failure(&self.original))?;
            report.candidate = Some(Candidate {
                kind: crate::solve::CandidateKind::FinalIterate,
                primal: vec![],
                objective: Some(objective * sense.sign()),
                row_dual: None,
                bound_dual: None,
                reduced_costs: None,
                slacks: None,
            });
        }
        Ok(Some(report))
    }
    /// Qualify, build and project one source start using native wrapper interfaces.
    #[allow(
        clippy::too_many_arguments,
        reason = "Presolve admission needs the independent solver, scaling, warm-start, and resource contracts"
    )]
    pub fn new(
        oracle: Box<dyn NlpOracle>,
        initial: &[f64],
        policy: &Policy,
        tolerance: &Tolerances,
        scaling: Option<&Scaling>,
        execution: Execution,
        warm: Option<&WarmStart>,
        compatibility: Compatibility,
        limit: usize,
    ) -> Result<Self, ProblemError> {
        let normalization = oracle.normalization().cloned().unwrap_or_else(|| {
            pse_math::normalization::Normalization::identity(
                oracle.contract().variables.len(),
                oracle.contract().rows.len(),
            )
        });
        normalization.validate(
            oracle.contract().variables.len(),
            oracle.contract().rows.len(),
        )?;
        let mut report = policy.qualify(oracle.as_ref(), tolerance)?;
        let (n, m, _, _) = report.dimensions;
        if n > limit || m > limit || initial.len() != n || initial.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "presolve dimensions/start/cap".into(),
            ));
        }
        if let Some(s) = scaling {
            s.validate(n, m)?;
        }
        let mut start = initial.to_vec();
        let mut duals = None;
        if let Some(w) = warm {
            w.validate(&compatibility)?;
            match &w.payload {
                WarmPayload::Nlp{primal,bounds,rows}=>{
                    if primal.len()!=n || primal.iter().any(|v|!v.is_finite()) {return Err(ProblemError::Contract("original warm primal dimensions/values".into()));}
                    start.clone_from(primal);
                    match (bounds,rows) {
                        (Some((l,u)),Some(r)) if l.len()==n && u.len()==n && r.len()==m &&
                            l.iter().chain(u).all(|v|v.is_finite() && *v>=0.0) && r.iter().all(|v|v.is_finite())=>{duals=Some((l.clone(),u.clone(),r.clone()));},
                        (None,None)=>{},
                        _=>return Err(ProblemError::Contract("original warm dual dimensions/sign/values".into())),
                    }
                },
                _=>return Err(ProblemError::Contract("native basis/working set requires its exact native coordinate owner; supply an original NLP start".into())),
            }
        }
        let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
        let hess = oracle
            .hessian_pattern()
            .map(|p| Pattern::new(p, true))
            .transpose()?
            .unwrap_or(Pattern {
                rows: vec![],
                columns: vec![],
            });
        let source_contract = oracle.contract().clone();
        let want_duals = duals.is_some();
        let original = Rc::new(RefCell::new(Adapter {
            normalization,
            certification_budget: None,
            oracle,
            state: CallbackState::new(execution),
            jac,
            hess,
            initial: start,
            duals,
            solution: None,
            normalize_affine: report.effective.enabled,
        }));
        let mut outer: Rc<RefCell<dyn TNLP>> = original.clone();
        let row_wrapper = if report.effective.enabled {
            let provider: Rc<RefCell<dyn ExpressionProvider>> = original.clone();
            let p = Rc::new(RefCell::new(PresolveTnlp::with_expression_provider(
                outer,
                provider,
                report.effective,
            )));
            outer = p.clone();
            Some(p)
        } else {
            None
        };
        let affine = if report.effective.linear_eq_reduction {
            let p = Rc::new(RefCell::new(LinearEqElimTnlp::new(outer, report.effective)));
            outer = p.clone();
            Some(p)
        } else {
            None
        };
        let info = outer
            .borrow_mut()
            .get_nlp_info()
            .ok_or_else(|| failure(&original))?;
        if info.n < 0
            || info.m < 0
            || info.nnz_jac_g < 0
            || info.nnz_h_lag < 0
            || [info.n, info.m, info.nnz_jac_g, info.nnz_h_lag]
                .iter()
                .any(|v| *v as usize > limit)
            || info.index_style != IndexStyle::C
        {
            return Err(ProblemError::Contract(
                "presolve native dimensions/index style/cap".into(),
            ));
        }
        let detected = row_wrapper.as_ref().is_some_and(|p| {
            let p = p.borrow();
            p.certified_infeasible().is_some()
                || p.tighten_report().infeasible
                || p.fbbt_report()
                    .is_some_and(|r| r.infeasibility_witness.is_some())
        });
        if detected {
            let budgets = tolerance.normalized(&original.borrow().normalization)?;
            let can_confirm = original
                .borrow()
                .oracle
                .presolve_facts()
                .is_some_and(|f| !f.has_guards);
            let proof = if can_confirm {
                original.borrow_mut().certification_budget = Some(budgets.clone());
                let mut confirmation = PresolveTnlp::with_expression_provider(
                    original.clone(),
                    original.clone(),
                    pounce_presolve::PresolveOptions {
                        auxiliary: false,
                        linear_eq_reduction: false,
                        redundant_constraint_removal: false,
                        licq_check: false,
                        ..report.effective
                    },
                );
                let ok = confirmation.get_nlp_info().is_some();
                let proof = ok.then(|| confirmation.certified_infeasible()).flatten();
                drop(confirmation);
                original.borrow_mut().certification_budget = None;
                proof
            } else {
                None
            };
            if let Some(native) = proof {
                let witness_row = match native {
                    pounce_nlp::tnlp::InfeasibilityProof::IntervalArithmetic { witness } => {
                        source_contract.rows.get(witness).copied()
                    }
                    pounce_nlp::tnlp::InfeasibilityProof::BoundPropagation => None,
                };
                report.proof = Some(super::PresolveProof {
                    contributions: original
                        .borrow()
                        .oracle
                        .presolve_facts()
                        .map(|f| {
                            source_contract
                                .rows
                                .iter()
                                .zip(&f.row_sources)
                                .flat_map(|(r, s)| s.iter().map(move |(i, o)| (*r, *i, *o)))
                                .collect()
                        })
                        .unwrap_or_default(),
                    native,
                    witness_row,
                    rows: source_contract.rows.clone(),
                    columns: source_contract.variables.iter().map(|v| v.id).collect(),
                    normalization: original.borrow().normalization.key(),
                    budgets,
                });
            } else {
                if matches!(policy, Policy::Explicit { required, .. } if !required.is_empty()) {
                    return Err(ProblemError::Contract("required presolve reduction lacks a tolerance-compatible infeasibility proof".into()));
                }
                drop(outer);
                drop(row_wrapper);
                drop(affine);
                let adapter = Rc::try_unwrap(original)
                    .map_err(|_| {
                        ProblemError::Contract(
                            "presolve confirmation retained callback owner".into(),
                        )
                    })?
                    .into_inner();
                let execution = adapter.state.execution.clone();
                let mut fallback = Self::new(
                    adapter.oracle,
                    initial,
                    &Policy::Off,
                    tolerance,
                    scaling,
                    execution,
                    warm,
                    compatibility,
                    limit,
                )?;
                fallback.report.requested = policy.clone();
                fallback.report.diagnostics.insert("infeasibility.confirmation".into(),
                    "not established after per-bound acceptance expansion; unreduced normalized problem retained".into());
                return Ok(fallback);
            }
        }
        let nr = info.n as usize;
        let mr = info.m as usize;
        let map = match &row_wrapper {
            Some(p) => p
                .borrow_mut()
                .transformation()
                .ok_or_else(|| failure(&original))?,
            None => PresolveMap::identity(
                n,
                m,
                source_contract.variables.iter().map(|v| v.lower).collect(),
                source_contract.variables.iter().map(|v| v.upper).collect(),
            ),
        };
        let elim = affine
            .as_ref()
            .and_then(|p| p.borrow_mut().elimination_plan());
        report.columns = elim
            .as_ref()
            .map_or_else(|| (0..n).collect(), |p| p.vars_kept.clone());
        report.rows = elim.as_ref().map_or_else(
            || map.rows_kept.clone(),
            |p| p.rows_kept.iter().map(|i| map.rows_kept[*i]).collect(),
        );
        if report.columns.len() != nr || report.rows.len() != mr {
            return Err(ProblemError::Contract(
                "presolve semantic maps disagree with native shape".into(),
            ));
        }
        report.dimensions = (n, m, nr, mr);
        if let Some(p) = &row_wrapper {
            let p = p.borrow();

            report
                .diagnostics
                .insert("bounds".into(), format!("{:?}", p.tighten_report()));
            report
                .diagnostics
                .insert("fbbt".into(), format!("{:?}", p.fbbt_report()));
            report
                .diagnostics
                .insert("rank".into(), format!("{:?}", p.licq_verdict()));
            report.diagnostics.insert(
                "auxiliary".into(),
                format!("{:?}", p.auxiliary_diagnostics()),
            );
        }
        if let Some(p) = &affine {
            report
                .diagnostics
                .insert("affine".into(), format!("{:?}", p.borrow().report()));
        }
        let (mut xl, mut xu, mut gl, mut gu) =
            (vec![0.0; nr], vec![0.0; nr], vec![0.0; mr], vec![0.0; mr]);
        if !outer.borrow_mut().get_bounds_info(BoundsInfo {
            x_l: &mut xl,
            x_u: &mut xu,
            g_l: &mut gl,
            g_u: &mut gu,
        }) {
            return Err(failure(&original));
        }
        let (mut x, mut zl, mut zu, mut lambda) =
            (vec![0.0; nr], vec![0.0; nr], vec![0.0; nr], vec![0.0; mr]);
        if !outer.borrow_mut().get_starting_point(StartingPoint {
            init_x: true,
            x: &mut x,
            init_z: want_duals,
            z_l: &mut zl,
            z_u: &mut zu,
            init_lambda: want_duals,
            lambda: &mut lambda,
        }) {
            return Err(failure(&original));
        }
        if let Some(p) = &row_wrapper {
            report.diagnostics.insert(
                "warm.rows".into(),
                format!("{:?}", p.borrow().starting_point_projection_report()),
            );
        }
        if let Some(p) = &affine {
            report.diagnostics.insert(
                "warm.affine".into(),
                format!("{:?}", p.borrow().starting_point_projection_report()),
            );
        }
        let projected_scaling = scaling.map(|s| Scaling {
            objective: s.objective,
            variables: report.columns.iter().map(|i| s.variables[*i]).collect(),
            constraints: report.rows.iter().map(|i| s.constraints[*i]).collect(),
        });
        let mut h = pse_ids::FramedHasher::new("pse.presolve.transformation.v2");
        h.hash(&original.borrow().normalization.key());
        h.hash(&compatibility.layout)
            .hash(&policy.key())
            .u64(nr as u64)
            .u64(mr as u64);
        if let Some(f) = report.facts {
            h.hash(&f);
        }
        for i in report.columns.iter().chain(&report.rows) {
            h.u64(*i as u64);
        }
        for v in xl.iter().chain(&xu).chain(&gl).chain(&gu) {
            h.u64(v.to_bits());
        }
        // Recovery coefficients and consumed data are part of native reuse identity,
        // even when the retained row/column indices happen to be unchanged.
        for (i, v) in map.fixed_vars.iter().zip(&map.fixed_values) {
            h.u64(*i as u64).u64(v.to_bits());
        }
        if let Some(p) = &elim {
            use pounce_presolve::VarRecovery;
            for r in &p.recovery {
                match r {
                    VarRecovery::Kept(i) => {
                        h.u64(0).u64(*i as u64);
                    }
                    VarRecovery::Constant(v) => {
                        h.u64(1).u64(v.to_bits());
                    }
                    VarRecovery::Affine { rep, coeff, offset } => {
                        h.u64(2)
                            .u64(*rep as u64)
                            .u64(coeff.to_bits())
                            .u64(offset.to_bits());
                    }
                }
            }
            for s in &p.steps {
                h.u64(s.row as u64).u64(s.var as u64).u64(s.pivot.to_bits());
            }
            for i in p.x_l_src.iter().chain(&p.x_u_src) {
                h.u64(*i as u64);
            }
        }
        if let Some(s) = &projected_scaling {
            h.hash(&s.key());
        }
        report.transformation = h.finish_hash();
        let native = Compatibility {
            layout: report.transformation,
            data: compatibility.data,
            backend: compatibility.backend,
        };
        let warm = want_duals.then(|| WarmStart {
            origin: None,
            compatibility: native.clone(),
            payload: WarmPayload::Nlp {
                primal: x.clone(),
                bounds: Some((zl, zu)),
                rows: Some(lambda),
            },
        });
        let mut contract = source_contract;
        contract.identity = report.transformation;
        contract.variables = report
            .columns
            .iter()
            .enumerate()
            .map(|(j, i)| {
                let mut v = contract.variables[*i].clone();
                v.lower = xl[j];
                v.upper = xu[j];
                v
            })
            .collect();
        contract.rows = report.rows.iter().map(|i| contract.rows[*i]).collect();
        let (jac, nj) = matrix(&outer, mr, nr, info.nnz_jac_g as usize, false, limit)?;
        let (hess, nh) = matrix(&outer, nr, nr, info.nnz_h_lag as usize, true, limit)?;
        let transport = Transport {
            outer: outer.clone(),
            original: original.clone(),
            contract,
            bounds: gl.into_iter().zip(gu).collect(),
            jac,
            hess,
            nj,
            nh,
            scaling: projected_scaling,
        };
        Ok(Self {
            original,
            outer,
            report,
            compatibility,
            native,
            initial: x,
            warm,
            transport: Some(transport),
        })
    }
    /// Projected initial coordinates; library projection already completed once.
    pub fn initial(&self) -> &[f64] {
        &self.initial
    }
    /// Projected complete dual seed, when available.
    pub fn warm(&self) -> Option<&WarmStart> {
        self.warm.as_ref()
    }
    /// Exact transformed native reuse identity.
    pub fn native_compatibility(&self) -> &Compatibility {
        &self.native
    }
    /// Project only the selected physical acceptance scales for native diagnostics.
    pub fn tolerances(&self, t: &Tolerances) -> Tolerances {
        Tolerances {
            variables: self
                .report
                .columns
                .iter()
                .map(|i| t.variables[*i] / self.original.borrow().normalization.variables[*i])
                .collect(),
            rows: self
                .report
                .rows
                .iter()
                .map(|i| t.rows[*i] / self.original.borrow().normalization.rows[*i])
                .collect(),
            integrality: t.integrality,
        }
    }
    /// Transfer callback ownership to one solver, once.
    pub fn take_oracle(&mut self) -> Result<Transport, ProblemError> {
        self.transport
            .take()
            .ok_or_else(|| ProblemError::Contract("presolve transport already consumed".into()))
    }
    /// Full recovery traverses the library stack exactly once, then independently observes the original model.
    pub fn finish(
        mut self,
        mut report: SolveReport,
        tolerance: &Tolerances,
        sense: ObjectiveSense,
    ) -> SolveReport {
        report.variables = self
            .original
            .borrow()
            .oracle
            .contract()
            .variables
            .iter()
            .map(|v| v.id)
            .collect();
        report.rows = self.original.borrow().oracle.contract().rows.clone();
        if let Some(c) = report.candidate.take() {
            let n = c.primal.len();
            let m = self.report.rows.len();
            let has_duals = c.bound_dual.is_some() && c.row_dual.is_some();
            let (zl, zu) = c.bound_dual.unwrap_or_else(|| (vec![0.0; n], vec![0.0; n]));
            let lambda = c.row_dual.unwrap_or_else(|| vec![0.0; m]);
            let status = match report.termination.category {
                Termination::Success => SolverReturn::Success,
                Termination::Acceptable => SolverReturn::StopAtAcceptablePoint,
                Termination::Cancelled => SolverReturn::UserRequestedStop,
                Termination::Limit | Termination::IterationLimit => SolverReturn::MaxiterExceeded,
                Termination::TimeLimit => SolverReturn::WallTimeExceeded,
                _ => SolverReturn::Unassigned,
            };
            let result = quality::contained(|| {
                if zl.len() != n || zu.len() != n || lambda.len() != m {
                    return Err(ProblemError::Contract(
                        "native final multiplier dimensions".into(),
                    ));
                }
                // g is transport-only. Original observations below never trust it.
                let mut g = vec![0.0; m];
                if !self.outer.borrow_mut().eval_g(&c.primal, true, &mut g) {
                    g.fill(f64::NAN);
                }
                self.outer.borrow_mut().finalize_solution(
                    Solution {
                        status,
                        x: &c.primal,
                        z_l: &zl,
                        z_u: &zu,
                        g: &g,
                        lambda: &lambda,
                        obj_value: c.objective.unwrap_or(f64::NAN) * sense.sign(),
                    },
                    &IpoptData::default(),
                    &IpoptCq::default(),
                );
                Ok(())
            });
            if let Err(e) = result {
                report.validation_error = Some(e.to_string());
                report.termination.assurance = Assurance::None;
            }
            if let Some(mut original) = self.original.borrow_mut().solution.take() {
                original.objective = original.objective.map(|v| v * sense.sign());
                if !has_duals {
                    original.bound_dual = None;
                    original.row_dual = None;
                }
                report.candidate = Some(original);
            }
        }
        report.warm_start = None;
        {
            let mut original = self.original.borrow_mut();
            if original.state.execution.stopped().is_none() {
                quality::attach_nlp(&mut report, original.oracle.as_mut(), tolerance, sense);
            } else {
                report.quality = None;
                report.observation = None;
                report.validation_error =
                    Some("final observation cancelled or deadline exhausted".into());
                report.termination.assurance = Assurance::None;
            }
            original.state.finish(&mut report);
        }
        if let Some(c) = &report.candidate {
            // Recovered multipliers must have valid dimensions and signs to be seeds.
            // Their numerical KKT qualification remains a separate completion decision.
            let qualified = report
                .observation
                .as_ref()
                .is_some_and(|o| o.dual_error.is_none());
            report.warm_start = Some(WarmStart {
                origin: None,
                compatibility: self.compatibility,
                payload: WarmPayload::Nlp {
                    primal: c.primal.clone(),
                    bounds: qualified.then(|| c.bound_dual.clone()).flatten(),
                    rows: qualified.then(|| c.row_dual.clone()).flatten(),
                },
            });
        } else {
            report.quality = None;
            report.observation = None;
            report.termination.assurance = Assurance::None;
        }
        self.report.diagnostics.insert("recovery".into(),"library finalize traversal; independently observed original model; duals use minimization convention".into());
        report.preprocessing = Some(self.report);
        report
    }
}
fn failure(original: &Rc<RefCell<Adapter>>) -> ProblemError {
    let a = original.borrow();
    if let Some((kind, message)) = &a.state.terminal {
        if *kind == Termination::Cancelled {
            return pse_math::MathError::Cancelled.into();
        }
        return ProblemError::Contract(format!("presolve callback: {message}"));
    }
    ProblemError::Contract(
        "presolve evaluation failed without an attributable source witness".into(),
    )
}
fn matrix(
    outer: &Rc<RefCell<dyn TNLP>>,
    rows: usize,
    cols: usize,
    nnz: usize,
    hessian: bool,
    limit: usize,
) -> Result<(AssemblyMatrix, usize), ProblemError> {
    let (mut r, mut c) = (vec![0; nnz], vec![0; nnz]);
    let mode = SparsityRequest::Structure {
        irow: &mut r,
        jcol: &mut c,
    };
    let ok = if hessian {
        outer
            .borrow_mut()
            .eval_h(None, false, 0.0, None, false, mode)
    } else {
        outer.borrow_mut().eval_jac_g(None, false, mode)
    };
    if !ok && nnz != 0 {
        return Err(ProblemError::Contract(
            "presolve sparse structure callback failed".into(),
        ));
    }
    let pairs = r
        .into_iter()
        .zip(c)
        .map(|(r, c)| {
            Ok((
                usize::try_from(r)
                    .map_err(|_| ProblemError::Contract("negative native row".into()))?,
                usize::try_from(c)
                    .map_err(|_| ProblemError::Contract("negative native column".into()))?,
            ))
        })
        .collect::<Result<Vec<_>, ProblemError>>()?;
    Ok((AssemblyMatrix::new(rows, cols, &pairs, limit)?, nnz))
}
/// Faer canonicalizes native COO output. This type never implements a reduction algorithm.
pub struct Transport {
    outer: Rc<RefCell<dyn TNLP>>,
    original: Rc<RefCell<Adapter>>,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    jac: AssemblyMatrix,
    hess: AssemblyMatrix,
    nj: usize,
    nh: usize,
    scaling: Option<Scaling>,
}
impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeCoordinateTransport")
            .field("contract", &self.contract)
            .finish_non_exhaustive()
    }
}
impl NlpOracle for Transport {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn scaling(&self) -> Option<&Scaling> {
        self.scaling.as_ref()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.jac.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        (self.contract.derivatives >= pse_kernels::DerivativeOrder::Second)
            .then(|| self.hess.matrix().symbolic())
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        let v = self.outer.borrow_mut().eval_f(x, true);
        v.ok_or_else(|| failure(&self.original))
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if self.outer.borrow_mut().eval_grad_f(x, true, out) {
            Ok(())
        } else {
            Err(failure(&self.original))
        }
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if self.outer.borrow_mut().eval_g(x, true, out) {
            Ok(())
        } else {
            Err(failure(&self.original))
        }
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let mut v = vec![0.0; self.nj];
        if !self.outer.borrow_mut().eval_jac_g(
            Some(x),
            true,
            SparsityRequest::Values { values: &mut v },
        ) {
            return Err(failure(&self.original));
        }
        refill(&mut self.jac, &v, out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        w: f64,
        l: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let mut v = vec![0.0; self.nh];
        if !self.outer.borrow_mut().eval_h(
            Some(x),
            true,
            w,
            Some(l),
            true,
            SparsityRequest::Values { values: &mut v },
        ) {
            return Err(failure(&self.original));
        }
        refill(&mut self.hess, &v, out)
    }
}
fn refill(matrix: &mut AssemblyMatrix, v: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
    matrix.clear();
    for (i, v) in v.iter().enumerate() {
        matrix.add(i, *v)?;
    }
    crate::tnlp::copy(matrix.matrix().val(), out)
}
