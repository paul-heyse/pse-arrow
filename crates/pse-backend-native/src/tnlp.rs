// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared TNLP boundary for library presolve and native POUNCE execution.
use crate::{
    NlpOracle, ProblemError,
    callback::CallbackState,
    nlp_pattern::Pattern,
    solve::{Candidate, Event, Metric},
};
use pounce_nlp::tnlp::{
    BoundsInfo, IDX_NAMES, IndexStyle, IpoptCq, IpoptData, IterStats, MetaData, NlpInfo, Solution,
    SparsityRequest, StartingPoint, TNLP,
};
pub(crate) struct Adapter {
    pub(crate) normalization: pse_math::normalization::Normalization,
    pub(crate) certification_budget: Option<crate::quality::Tolerances>,
    pub(crate) normalize_affine: bool,
    pub(crate) oracle: Box<dyn NlpOracle>,
    pub(crate) state: CallbackState,
    pub(crate) jac: Pattern,
    pub(crate) hess: Pattern,
    pub(crate) initial: Vec<f64>,
    pub(crate) duals: Option<(Vec<f64>, Vec<f64>, Vec<f64>)>,
    pub(crate) solution: Option<Candidate>,
}
impl pounce_nlp::expression_provider::ExpressionProvider for Adapter {
    fn constraint_expression(&self, i: usize) -> Option<pounce_nlp::expression_provider::FbbtTape> {
        use pounce_nlp::expression_provider::FbbtOp;
        let f = self.oracle.presolve_facts()?;
        let mut tape = f.tapes.get(i)?.clone();
        if self.normalize_affine
            && let Some(row) = f.affine.get(i)?.as_ref()
        {
            let n = tape.len();
            if n == 0 {
                return None;
            }
            tape.ops.push(FbbtOp::Const(row.constant));
            tape.ops.push(FbbtOp::Sub(n - 1, n));
        }
        self.normalization.tape(&tape, i, 1_000_000).ok()
    }
}
pub(crate) fn copy<T: Copy>(from: &[T], to: &mut [T]) -> Result<(), ProblemError> {
    if from.len() != to.len() {
        return Err(ProblemError::Contract("POUNCE callback dimensions".into()));
    }
    to.copy_from_slice(from);
    Ok(())
}
pub(crate) fn finite(x: &[f64]) -> Result<(), ProblemError> {
    if x.iter().any(|v| !v.is_finite()) {
        Err(ProblemError::Contract(
            "nonfinite POUNCE callback values".into(),
        ))
    } else {
        Ok(())
    }
}
fn point(x: Option<&[f64]>) -> Result<&[f64], ProblemError> {
    x.ok_or_else(|| ProblemError::Contract("missing POUNCE trial".into()))
}
impl TNLP for Adapter {
    fn get_scaling_parameters(&mut self, r: pounce_nlp::tnlp::ScalingRequest<'_>) -> bool {
        let Some(s) = self.oracle.scaling() else {
            return false;
        };
        if s.validate(r.x_scaling.len(), r.g_scaling.len()).is_err() {
            return false;
        }
        *r.obj_scaling = s.objective;
        *r.use_x_scaling = true;
        *r.use_g_scaling = true;
        r.x_scaling.copy_from_slice(&s.variables);
        r.g_scaling.copy_from_slice(&s.constraints);
        true
    }
    fn get_variables_linearity(&mut self, types: &mut [pounce_nlp::tnlp::Linearity]) -> bool {
        let Some(f) = self.oracle.presolve_facts() else {
            return false;
        };
        if types.len() != f.objective_linear.len() {
            return false;
        }
        for (t, linear) in types.iter_mut().zip(&f.objective_linear) {
            *t = if *linear {
                pounce_nlp::tnlp::Linearity::Linear
            } else {
                pounce_nlp::tnlp::Linearity::NonLinear
            };
        }
        true
    }
    fn derivative_proofs(&mut self) -> pounce_nlp::constant_derivatives::DerivativeProofs {
        use pounce_nlp::constant_derivatives::{DerivativeProof as P, DerivativeProofs};
        let f = self.oracle.derivative_facts();
        let proof = |v| if v { P::Constant } else { P::Unknown };
        DerivativeProofs {
            grad_f: proof(f.gradient_constant),
            hessian: proof(f.hessian_constant),
            jac: self.oracle.presolve_facts().map_or_else(
                || vec![proof(f.jacobian_constant); self.oracle.contract().rows.len()],
                |p| p.affine.iter().map(|r| proof(r.is_some())).collect(),
            ),
        }
    }
    fn get_constraints_linearity(&mut self, types: &mut [pounce_nlp::tnlp::Linearity]) -> bool {
        if let Some(f) = self.oracle.presolve_facts() {
            if types.len() != f.affine.len() {
                return false;
            }
            for (t, row) in types.iter_mut().zip(&f.affine) {
                *t = if row.is_some() {
                    pounce_nlp::tnlp::Linearity::Linear
                } else {
                    pounce_nlp::tnlp::Linearity::NonLinear
                };
            }
            return true;
        }
        if self.oracle.derivative_facts().jacobian_constant
            && types.len() == self.oracle.contract().rows.len()
        {
            types.fill(pounce_nlp::tnlp::Linearity::Linear);
            true
        } else {
            false
        }
    }

    fn get_nlp_info(&mut self) -> Option<NlpInfo> {
        Some(NlpInfo {
            n: i32::try_from(self.initial.len()).ok()?,
            m: i32::try_from(self.oracle.contract().rows.len()).ok()?,
            nnz_jac_g: i32::try_from(self.jac.rows.len()).ok()?,
            nnz_h_lag: i32::try_from(self.hess.rows.len()).ok()?,
            index_style: IndexStyle::C,
        })
    }
    fn get_bounds_info(&mut self, b: BoundsInfo<'_>) -> bool {
        self.state
            .evaluate("bounds", || {
                if b.x_l.len() != self.initial.len()
                    || b.x_u.len() != self.initial.len()
                    || b.g_l.len() != self.oracle.contract().rows.len()
                    || b.g_u.len() != self.oracle.contract().rows.len()
                {
                    return Err(ProblemError::Contract(
                        "POUNCE bound output dimensions".into(),
                    ));
                }
                let lower: Vec<_> = self
                    .oracle
                    .contract()
                    .variables
                    .iter()
                    .map(|v| v.lower)
                    .collect();
                let upper: Vec<_> = self
                    .oracle
                    .contract()
                    .variables
                    .iter()
                    .map(|v| v.upper)
                    .collect();
                let gl: Vec<_> = self
                    .oracle
                    .constraint_bounds()
                    .iter()
                    .map(|v| v.0)
                    .collect();
                let gu: Vec<_> = self
                    .oracle
                    .constraint_bounds()
                    .iter()
                    .map(|v| v.1)
                    .collect();
                copy(&lower, b.x_l)?;
                copy(&upper, b.x_u)?;
                copy(&gl, b.g_l)?;
                copy(&gu, b.g_u)?;
                if self.normalize_affine
                    && let Some(f) = self.oracle.presolve_facts()
                {
                    for (r, row) in f.affine.iter().enumerate() {
                        if let Some(row) = row {
                            for bound in [&mut b.g_l[r], &mut b.g_u[r]] {
                                let shifted = *bound - row.constant;
                                if bound.is_finite() && !shifted.is_finite() {
                                    return Err(ProblemError::Contract(
                                        "affine bound transport overflow".into(),
                                    ));
                                }
                                *bound = shifted;
                            }
                        }
                    }
                }
                for (i, (l, u)) in b.x_l.iter_mut().zip(b.x_u.iter_mut()).enumerate() {
                    for value in [l, u] {
                        if value.is_finite() {
                            *value = pse_math::normalization::checked_ratio(
                                *value,
                                self.normalization.variables[i],
                            )?;
                        }
                    }
                }
                for (i, (l, u)) in b.g_l.iter_mut().zip(b.g_u.iter_mut()).enumerate() {
                    for value in [l, u] {
                        if value.is_finite() {
                            *value = pse_math::normalization::checked_ratio(
                                *value,
                                self.normalization.rows[i],
                            )?;
                        }
                    }
                }
                if b.x_l
                    .iter()
                    .chain(&*b.x_u)
                    .chain(&*b.g_l)
                    .chain(&*b.g_u)
                    .any(|v| v.is_finite() && v.abs() >= 1e19)
                {
                    return Err(ProblemError::Contract(
                        "normalized finite bound reaches native infinity sentinel".into(),
                    ));
                }
                if let Some(t) = &self.certification_budget {
                    for (lower, upper, budgets) in [
                        (&mut *b.x_l, &mut *b.x_u, &t.variables),
                        (&mut *b.g_l, &mut *b.g_u, &t.rows),
                    ] {
                        for ((l, u), budget) in lower.iter_mut().zip(upper).zip(budgets) {
                            for (bound, delta) in [(l, -budget), (u, *budget)] {
                                if bound.is_finite() {
                                    let expanded = *bound + delta;
                                    if !expanded.is_finite() || expanded.abs() >= 1e19 {
                                        return Err(ProblemError::Contract(
                                            "tolerance expansion reaches native infinity sentinel"
                                                .into(),
                                        ));
                                    }
                                    *bound = expanded;
                                }
                            }
                        }
                    }
                }
                Ok(())
            })
            .is_some()
    }
    fn get_starting_point(&mut self, s: StartingPoint<'_>) -> bool {
        self.state
            .evaluate("start", || {
                if s.init_x && s.x.len() != self.initial.len()
                    || s.init_z
                        && (s.z_l.len() != self.initial.len() || s.z_u.len() != self.initial.len())
                    || s.init_lambda && s.lambda.len() != self.oracle.contract().rows.len()
                {
                    return Err(ProblemError::Contract(
                        "POUNCE start output dimensions".into(),
                    ));
                }
                if s.init_x {
                    copy(&self.normalization.normalized_point(&self.initial)?, s.x)?;
                }
                if s.init_z || s.init_lambda {
                    let (l, u, r) = self.duals.as_ref().ok_or_else(|| {
                        ProblemError::Contract("POUNCE requested absent dual seed".into())
                    })?;
                    if s.init_z {
                        copy(l, s.z_l)?;
                        copy(u, s.z_u)?;
                        for (i, (l, u)) in s.z_l.iter_mut().zip(s.z_u.iter_mut()).enumerate() {
                            *l = pse_math::normalization::checked_ratio(
                                pse_math::normalization::checked_product(
                                    *l,
                                    self.normalization.variables[i],
                                )?,
                                self.normalization.objective,
                            )?;
                            *u = pse_math::normalization::checked_ratio(
                                pse_math::normalization::checked_product(
                                    *u,
                                    self.normalization.variables[i],
                                )?,
                                self.normalization.objective,
                            )?;
                        }
                    }
                    if s.init_lambda {
                        copy(r, s.lambda)?;
                        for (i, v) in s.lambda.iter_mut().enumerate() {
                            *v = pse_math::normalization::checked_ratio(
                                pse_math::normalization::checked_product(
                                    *v,
                                    self.normalization.rows[i],
                                )?,
                                self.normalization.objective,
                            )?;
                        }
                    }
                }
                Ok(())
            })
            .is_some()
    }
    fn eval_f(&mut self, x: &[f64], _: bool) -> Option<f64> {
        self.state.evaluate("objective", || {
            let point = self.normalization.physical_point(x)?;
            let v = pse_math::normalization::checked_ratio(
                self.oracle.objective(&point)?,
                self.normalization.objective,
            )?;
            finite(&[v])?;
            Ok(v)
        })
    }
    fn eval_grad_f(&mut self, x: &[f64], _: bool, out: &mut [f64]) -> bool {
        self.state
            .evaluate("gradient", || {
                let mut v = vec![0.0; out.len()];
                self.oracle
                    .gradient(&self.normalization.physical_point(x)?, &mut v)?;
                for (i, value) in v.iter_mut().enumerate() {
                    *value = pse_math::normalization::checked_ratio(
                        pse_math::normalization::checked_product(
                            *value,
                            self.normalization.variables[i],
                        )?,
                        self.normalization.objective,
                    )?;
                }
                finite(&v)?;
                copy(&v, out)
            })
            .is_some()
    }
    fn eval_g(&mut self, x: &[f64], _: bool, out: &mut [f64]) -> bool {
        self.state
            .evaluate("constraints", || {
                let mut v = vec![0.0; out.len()];
                self.oracle
                    .constraints(&self.normalization.physical_point(x)?, &mut v)?;
                if self.normalize_affine
                    && let Some(f) = self.oracle.presolve_facts()
                {
                    for (v, row) in v.iter_mut().zip(&f.affine) {
                        if let Some(row) = row {
                            *v -= row.constant;
                        }
                    }
                }
                for (i, value) in v.iter_mut().enumerate() {
                    *value =
                        pse_math::normalization::checked_ratio(*value, self.normalization.rows[i])?;
                }
                finite(&v)?;
                copy(&v, out)
            })
            .is_some()
    }
    fn eval_jac_g(&mut self, x: Option<&[f64]>, _: bool, mode: SparsityRequest<'_>) -> bool {
        self.state
            .evaluate("jacobian", || match mode {
                SparsityRequest::Structure { irow, jcol } => {
                    if irow.len() != self.jac.rows.len() || jcol.len() != self.jac.columns.len() {
                        return Err(ProblemError::Contract(
                            "POUNCE Jacobian structure dimensions".into(),
                        ));
                    }
                    copy(&self.jac.rows, irow)?;
                    copy(&self.jac.columns, jcol)
                }
                SparsityRequest::Values { values } => {
                    let mut v = vec![0.0; values.len()];
                    self.oracle
                        .jacobian(&self.normalization.physical_point(point(x)?)?, &mut v)?;
                    for (i, value) in v.iter_mut().enumerate() {
                        *value = pse_math::normalization::checked_ratio(
                            pse_math::normalization::checked_product(
                                *value,
                                self.normalization.variables[self.jac.columns[i] as usize],
                            )?,
                            self.normalization.rows[self.jac.rows[i] as usize],
                        )?;
                    }
                    finite(&v)?;
                    copy(&v, values)
                }
            })
            .is_some()
    }
    fn eval_h(
        &mut self,
        x: Option<&[f64]>,
        _: bool,
        weight: f64,
        lambda: Option<&[f64]>,
        _: bool,
        mode: SparsityRequest<'_>,
    ) -> bool {
        self.state
            .evaluate("hessian", || match mode {
                SparsityRequest::Structure { irow, jcol } => {
                    if irow.len() != self.hess.rows.len() || jcol.len() != self.hess.columns.len() {
                        return Err(ProblemError::Contract(
                            "POUNCE Hessian structure dimensions".into(),
                        ));
                    }
                    copy(&self.hess.rows, irow)?;
                    copy(&self.hess.columns, jcol)
                }
                SparsityRequest::Values { values } => {
                    let mut v = vec![0.0; values.len()];
                    let lambda = if self.oracle.contract().rows.is_empty() {
                        &[][..]
                    } else {
                        point(lambda)?
                    };
                    if lambda.len() != self.normalization.rows.len() {
                        return Err(ProblemError::Contract("normalized multipliers".into()));
                    }
                    let lambda = lambda
                        .iter()
                        .zip(&self.normalization.rows)
                        .map(|(v, s)| pse_math::normalization::checked_ratio(*v, *s))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.oracle.hessian(
                        &self.normalization.physical_point(point(x)?)?,
                        pse_math::normalization::checked_ratio(
                            weight,
                            self.normalization.objective,
                        )?,
                        &lambda,
                        &mut v,
                    )?;
                    for (i, value) in v.iter_mut().enumerate() {
                        *value = pse_math::normalization::checked_product(
                            pse_math::normalization::checked_product(
                                *value,
                                self.normalization.variables[self.hess.columns[i] as usize],
                            )?,
                            self.normalization.variables[self.hess.rows[i] as usize],
                        )?;
                    }
                    finite(&v)?;
                    copy(&v, values)
                }
            })
            .is_some()
    }
    fn finalize_solution(&mut self, s: Solution<'_>, _: &IpoptData, _: &IpoptCq) {
        self.solution = self.state.evaluate("original coordinate recovery", || {
            if s.lambda.len() != self.normalization.rows.len()
                || s.z_l.len() != self.normalization.variables.len()
                || s.z_u.len() != self.normalization.variables.len()
            {
                return Err(ProblemError::Contract(
                    "native multiplier recovery dimensions".into(),
                ));
            }
            let row_dual = s
                .lambda
                .iter()
                .zip(&self.normalization.rows)
                .map(|(v, scale)| {
                    pse_math::normalization::checked_ratio(
                        pse_math::normalization::checked_product(*v, self.normalization.objective)?,
                        *scale,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let bounds = |values: &[f64]| {
                values
                    .iter()
                    .zip(&self.normalization.variables)
                    .map(|(v, scale)| {
                        pse_math::normalization::checked_ratio(
                            pse_math::normalization::checked_product(
                                *v,
                                self.normalization.objective,
                            )?,
                            *scale,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            };
            Ok(Candidate {
                kind: crate::solve::CandidateKind::FinalIterate,
                primal: self.normalization.physical_point(s.x)?,
                objective: s
                    .obj_value
                    .is_finite()
                    .then(|| {
                        pse_math::normalization::checked_product(
                            s.obj_value,
                            self.normalization.objective,
                        )
                    })
                    .transpose()?,
                row_dual: Some(row_dual),
                bound_dual: Some((bounds(s.z_l)?, bounds(s.z_u)?)),
                reduced_costs: None,
                slacks: None,
            })
        });
    }
    fn get_var_con_metadata(&mut self, v: &mut MetaData, r: &mut MetaData) -> bool {
        v.strings.insert(
            IDX_NAMES.into(),
            self.oracle
                .contract()
                .variables
                .iter()
                .map(|v| v.id.to_string())
                .collect(),
        );
        r.strings.insert(
            IDX_NAMES.into(),
            self.oracle
                .contract()
                .rows
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        true
    }
    fn intermediate_callback(&mut self, s: IterStats, _: &IpoptData, _: &IpoptCq) -> bool {
        let execution = self.state.execution.clone();
        self.state.evaluate("intermediate",||{let mut values=std::collections::BTreeMap::new();
            macro_rules! real{($($f:ident),*)=>{$(values.insert(stringify!($f).into(),Metric::Real(s.$f));)*}}
            real!(obj_value,inf_pr,inf_du,mu,d_norm,regularization_size,alpha_du,alpha_pr);
            values.insert("iteration".into(),Metric::Integer(i64::from(s.iter)));values.insert("line_search.trials".into(),Metric::Integer(i64::from(s.ls_trials)));values.insert("mode".into(),Metric::Text(format!("{:?}",s.mode)));
            execution.progress.push(Event{phase:"pounce.iteration".into(),elapsed:execution.started.elapsed(),values});Ok(())}).is_some()
    }
}
