// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked coordinate transport shared by coefficient, cone and root adapters.
use crate::{CoefficientProblem, ConicProblem, NleOracle, OracleContract, ProblemError, solve::*};
use pse_math::{
    convexity::{QuadraticEvidence, TransportedEvidence, normalize_quadratic},
    normalization::{Normalization, checked_product as mul, checked_ratio as div},
};

fn bound(v: f64, scale: f64) -> Result<f64, ProblemError> {
    if v.is_infinite() {
        Ok(v)
    } else {
        Ok(div(v, scale)?)
    }
}
fn contract(c: &OracleContract, n: &Normalization) -> Result<OracleContract, ProblemError> {
    n.validate(c.variables.len(), c.rows.len())?;
    let mut out = c.clone();
    for (v, s) in out.variables.iter_mut().zip(&n.variables) {
        v.lower = bound(v.lower, *s)?;
        v.upper = bound(v.upper, *s)?;
    }
    Ok(out)
}
fn values(v: &mut [f64], s: &[f64], inverse: bool) -> Result<(), ProblemError> {
    if v.len() != s.len() {
        return Err(ProblemError::Contract(
            "coordinate transport dimensions".into(),
        ));
    }
    for (v, s) in v.iter_mut().zip(s) {
        *v = if inverse { div(*v, *s)? } else { mul(*v, *s)? };
    }
    Ok(())
}
fn sparse(
    a: &faer::sparse::SparseColMat<usize, f64>,
    columns: &[f64],
    rows: &[f64],
) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
    let mut entries = Vec::with_capacity(a.val().len());
    for (c, s) in columns.iter().enumerate() {
        for (r, v) in a.row_idx_of_col(c).zip(a.val_of_col(c)) {
            entries.push(faer::sparse::Triplet::new(
                r,
                c,
                div(mul(*v, *s)?, rows[r])?,
            ));
        }
    }
    faer::sparse::SparseColMat::try_new_from_triplets(rows.len(), columns.len(), &entries)
        .map_err(|e| ProblemError::Contract(e.to_string()))
}
/// Project coefficients once; original PSD evidence is checked before congruence transport.
pub fn coefficients(
    p: &CoefficientProblem,
    n: &Normalization,
    evidence: Option<&dyn QuadraticEvidence>,
) -> Result<(CoefficientProblem, Option<TransportedEvidence>), ProblemError> {
    p.validate_convex(evidence)?;
    let contract = contract(&p.contract, n)?;
    if p.domains
        .iter()
        .zip(&n.variables)
        .any(|(d, s)| d.is_integer() && *s != 1.0)
    {
        return Err(ProblemError::Contract(
            "integer coordinate substitution".into(),
        ));
    }
    let (hessian, proof) = match &p.hessian {
        Some(q) if q.val().iter().any(|v| *v != 0.0) => {
            let (q, proof) = normalize_quadratic(
                q,
                p.sense.sign(),
                &n.variables,
                n.objective,
                evidence
                    .ok_or_else(|| ProblemError::Contract("missing quadratic evidence".into()))?,
            )?;
            (Some(q), Some(proof))
        }
        Some(q) => (Some(q.clone()), None),
        None => (None, None),
    };
    Ok((
        CoefficientProblem {
            contract,
            objective: p
                .objective
                .iter()
                .zip(&n.variables)
                .map(|(v, s)| div(mul(*v, *s)?, n.objective))
                .collect::<Result<_, _>>()?,
            objective_constant: div(p.objective_constant, n.objective)?,
            sense: p.sense,
            domains: p.domains.clone(),
            assumptions: p.assumptions,
            constraints: sparse(&p.constraints, &n.variables, &n.rows)?,
            hessian,
            bounds: p
                .bounds
                .iter()
                .zip(&n.rows)
                .map(|((l, u), s)| Ok((bound(*l, *s)?, bound(*u, *s)?)))
                .collect::<Result<_, ProblemError>>()?,
        },
        proof,
    ))
}
/// Cone-preserving positive row factors. Required member scales must agree within a block.
pub fn cone_normalization(
    p: &ConicProblem,
    policy: &pse_model::numerics::ResolvedNumericalPolicy,
) -> Result<Normalization, ProblemError> {
    use clarabel::solver::SupportedConeT::{NonnegativeConeT, ZeroConeT};
    let ids: Vec<_> = p.contract.variables.iter().map(|v| v.id).collect();
    let mut n = Normalization::from_policy(policy, &ids, &p.contract.rows)?;
    let mut start = 0;
    for cone in &p.cones {
        let end = start + crate::conic::dim(cone);
        if end > n.rows.len() {
            return Err(ProblemError::Contract("cone scale inventory".into()));
        }
        if !matches!(cone, ZeroConeT(_) | NonnegativeConeT(_)) {
            let mut required = None;
            let mut physical = None;
            for id in &p.contract.rows[start..end] {
                let t = policy
                    .targets
                    .iter()
                    .find(|t| {
                        t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Row
                    })
                    .ok_or_else(|| ProblemError::Contract("cone physical target absent".into()))?;
                if physical.is_some_and(|p| p != (t.quantity, t.unit)) {
                    return Err(ProblemError::Contract(
                        "cone members require a common physical representation".into(),
                    ));
                }
                physical = Some((t.quantity, t.unit));
            }
            for (id, s) in p.contract.rows[start..end].iter().zip(&n.rows[start..end]) {
                if policy
                    .targets
                    .iter()
                    .find(|t| {
                        t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Row
                    })
                    .is_some_and(|t| t.required_scale)
                {
                    if required.is_some_and(|v: f64| v.to_bits() != s.to_bits()) {
                        return Err(ProblemError::Contract(
                            "conflicting required cone block scales".into(),
                        ));
                    }
                    required = Some(*s);
                }
            }
            let common =
                required.unwrap_or_else(|| n.rows[start..end].iter().copied().fold(0.0, f64::max));
            n.rows[start..end].fill(common);
        }
        start = end;
    }
    if start != n.rows.len() {
        return Err(ProblemError::Contract("cone scale inventory".into()));
    }
    Ok(n)
}
/// Normalize native conic storage while retaining a checked original quadratic admission.
pub fn conic(
    p: &ConicProblem,
    n: &Normalization,
    proof: &dyn QuadraticEvidence,
) -> Result<(ConicProblem, TransportedEvidence), ProblemError> {
    p.validate(proof)?;
    let q = p.full_quadratic()?;
    let (q, proof) = normalize_quadratic(&q, 1.0, &n.variables, n.objective, proof)?;
    let mut quadratic = p.quadratic.clone();
    for c in 0..quadratic.n {
        for k in quadratic.colptr[c]..quadratic.colptr[c + 1] {
            quadratic.nzval[k] = div(
                mul(
                    mul(quadratic.nzval[k], n.variables[quadratic.rowval[k]])?,
                    n.variables[c],
                )?,
                n.objective,
            )?;
        }
    }
    // The checked congruence and upper-triangle storage represent the same unchanged Q.
    proof.validate(&q, 1.0)?;
    let mut constraints = p.constraints.clone();
    for c in 0..constraints.n {
        for k in constraints.colptr[c]..constraints.colptr[c + 1] {
            constraints.nzval[k] = div(
                mul(constraints.nzval[k], n.variables[c])?,
                n.rows[constraints.rowval[k]],
            )?;
        }
    }
    Ok((
        ConicProblem {
            contract: contract(&p.contract, n)?,
            quadratic,
            constraints,
            objective: p
                .objective
                .iter()
                .zip(&n.variables)
                .map(|(v, s)| div(mul(*v, *s)?, n.objective))
                .collect::<Result<_, _>>()?,
            rhs: p
                .rhs
                .iter()
                .zip(&n.rows)
                .map(|(v, s)| div(*v, *s))
                .collect::<Result<_, _>>()?,
            cones: p.cones.clone(),
            objective_constant: div(p.objective_constant, n.objective)?,
        },
        proof,
    ))
}
/// Convert a retained root/HiGHS seed without changing semantic compatibility or basis codes.
pub fn warm(w: &WarmStart, n: &Normalization, to_native: bool) -> Result<WarmStart, ProblemError> {
    let mut out = w.clone();
    match &mut out.payload {
        WarmPayload::Root(x) => values(x, &n.variables, to_native)?,
        WarmPayload::Highs { primal, dual, .. } => {
            if let Some(x) = primal {
                values(x, &n.variables, to_native)?;
            }
            if let Some((columns, rows)) = dual {
                duals(columns, &n.variables, n.objective, to_native)?;
                duals(rows, &n.rows, n.objective, to_native)?;
            }
        }
        _ => {
            return Err(ProblemError::Contract(
                "wrong coordinate transport warm-start kind".into(),
            ));
        }
    }
    Ok(out)
}
fn duals(
    v: &mut [f64],
    scales: &[f64],
    objective: f64,
    to_native: bool,
) -> Result<(), ProblemError> {
    if v.len() != scales.len() {
        return Err(ProblemError::Contract("dual transport dimensions".into()));
    }
    for (v, s) in v.iter_mut().zip(scales) {
        *v = if to_native {
            div(mul(*v, *s)?, objective)?
        } else {
            div(mul(*v, objective)?, *s)?
        };
    }
    Ok(())
}
fn recover_certificate(
    c: &mut Certificate,
    n: &Normalization,
    contract: &OracleContract,
) -> Result<(), ProblemError> {
    if let Some(ray) = &mut c.primal {
        values(ray, &n.variables, false)?;
    }
    if let Some(ray) = &mut c.dual {
        let mut scales = n.rows.clone();
        for (v, s) in contract.variables.iter().zip(&n.variables) {
            if v.lower.is_finite() {
                scales.push(*s);
            }
            if v.upper.is_finite() {
                scales.push(*s);
            }
        }
        values(ray, &scales, true)?;
    }
    Ok(())
}
/// Restore every retained candidate/observation/quality field to original physical coordinates.
pub fn recover(
    report: &mut SolveReport,
    n: &Normalization,
    contract: &OracleContract,
) -> Result<(), ProblemError> {
    if let Some(c) = &mut report.certificate {
        recover_certificate(c, n, contract)?;
        report.provenance.insert("certificate.coordinates".into(),"original physical homogeneous ray; native accuracy qualifier retained, no unit-length claim".into());
    }
    if let Some(c) = &mut report.candidate {
        values(&mut c.primal, &n.variables, false)?;
        if let Some(f) = &mut c.objective {
            *f = mul(*f, n.objective)?;
        }
        if let Some(v) = &mut c.row_dual {
            duals(v, &n.rows, n.objective, false)?;
        }
        if let Some((l, u)) = &mut c.bound_dual {
            duals(l, &n.variables, n.objective, false)?;
            duals(u, &n.variables, n.objective, false)?;
        }
        if let Some(v) = &mut c.reduced_costs {
            duals(v, &n.variables, n.objective, false)?;
        }
        if let Some(v) = &mut c.slacks {
            values(v, &n.rows, false)?;
        }
    }
    if let Some(o) = &mut report.observation {
        if let Some(f) = &mut o.objective {
            *f = mul(*f, n.objective)?;
        }
        values(&mut o.values, &n.rows, false)?;
        for ((l, u), s) in o.bounds.iter_mut().zip(&n.rows) {
            if l.is_finite() {
                *l = mul(*l, *s)?;
            }
            if u.is_finite() {
                *u = mul(*u, *s)?;
            }
        }
        for (v, s) in o.equality_residuals.iter_mut().zip(&n.rows) {
            if let Some(v) = v {
                *v = mul(*v, *s)?;
            }
        }
        values(&mut o.lower_violations, &n.rows, false)?;
        values(&mut o.upper_violations, &n.rows, false)?;
        if let Some(v) = &mut o.stationarity {
            duals(v, &n.variables, n.objective, false)?;
        }
        if let Some(v) = &mut o.complementarity {
            for v in v {
                *v = mul(*v, n.objective)?;
            }
        }
    }
    if let Some(q) = &mut report.quality {
        for v in &mut q.rows {
            let i = contract
                .rows
                .iter()
                .position(|id| *id == v.id)
                .ok_or_else(|| {
                    ProblemError::Contract("quality row absent from coordinate map".into())
                })?;
            let s = n.rows[i];
            v.physical = mul(v.physical, s)?;
            v.tolerance = mul(v.tolerance, s)?;
        }
        for (v, s) in q.bounds.iter_mut().zip(&n.variables) {
            v.physical = mul(v.physical, *s)?;
            v.tolerance = mul(v.tolerance, *s)?;
        }
    }
    if let Some(w) = report.warm_start.take() {
        report.warm_start = Some(warm(&w, n, false)?);
    }
    report.provenance.insert(
        "coordinates".into(),
        "original physical coordinates recovered from model normalization".into(),
    );
    Ok(())
}
/// Root callback projection; library linear solves receive the complete normalized Jacobian.
#[derive(Debug)]
pub struct Roots {
    original: Box<dyn NleOracle>,
    contract: OracleContract,
    normalization: Normalization,
}
impl Roots {
    /// Retain the original oracle and one immutable coordinate map.
    pub fn new(
        original: Box<dyn NleOracle>,
        normalization: Normalization,
    ) -> Result<Self, ProblemError> {
        Ok(Self {
            contract: contract(original.contract(), &normalization)?,
            original,
            normalization,
        })
    }
}
impl NleOracle for Roots {
    fn guard_signs(
        &self,
    ) -> std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign> {
        self.original.guard_signs()
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.original.jacobian_pattern()
    }
    fn residual(&mut self, z: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let x = self.normalization.physical_point(z)?;
        self.original.residual(&x, out)?;
        values(out, &self.normalization.rows, true)
    }
    fn jacobian(&mut self, z: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let x = self.normalization.physical_point(z)?;
        self.original.jacobian(&x, out)?;
        let p = self.original.jacobian_pattern();
        let mut k = 0;
        for c in 0..p.ncols() {
            for r in p.row_idx_of_col(c) {
                out[k] = div(
                    mul(out[k], self.normalization.variables[c])?,
                    self.normalization.rows[r],
                )?;
                k += 1;
            }
        }
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        z: &[f64],
        d: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let x = self.normalization.physical_point(z)?;
        let d = self.normalization.physical_point(d)?;
        self.original.jacobian_product(&x, &d, out)?;
        values(out, &self.normalization.rows, true)
    }
    fn observe(&self, mut residual: Vec<f64>) -> Result<crate::quality::Observation, ProblemError> {
        values(&mut residual, &self.normalization.rows, false)?;
        let o = self.original.observe(residual)?;
        let mut values_ = o.values.clone();
        values(&mut values_, &self.normalization.rows, true)?;
        let bounds = o
            .bounds
            .iter()
            .zip(&self.normalization.rows)
            .map(|((l, u), s)| Ok((bound(*l, *s)?, bound(*u, *s)?)))
            .collect::<Result<_, ProblemError>>()?;
        let mut out = crate::quality::Observation::from_values(None, values_, bounds)?;
        out.sources = o.sources;
        Ok(out)
    }
}

/// Transform complete physical relaxation penalties into the normalized diagnostic model.
#[cfg(feature = "highs")]
pub fn diagnostic_request(
    r: &crate::highs::diagnostics::Request,
    n: &Normalization,
) -> Result<crate::highs::diagnostics::Request, ProblemError> {
    let mut out = r.clone();
    if let Some(p) = &mut out.relaxation {
        p.validate(n.variables.len(), n.rows.len())?;
        let scale = |v: &Option<Vec<f64>>, global: f64, s: &[f64]| {
            s.iter()
                .enumerate()
                .map(|(i, s)| {
                    let v = v.as_ref().map_or(global, |v| v[i]);
                    if v < 0.0 {
                        Ok(v)
                    } else {
                        Ok(div(mul(v, *s)?, n.objective)?)
                    }
                })
                .collect::<Result<Vec<_>, ProblemError>>()
        };
        p.lower = Some(scale(&p.lower, p.global[0], &n.variables)?);
        p.upper = Some(scale(&p.upper, p.global[1], &n.variables)?);
        p.rows = Some(scale(&p.rows, p.global[2], &n.rows)?);
    }
    Ok(out)
}
/// Restore original diagnostic directions, bounds, costs and relaxed points; IIS identities persist.
#[cfg(feature = "highs")]
pub fn recover_diagnostics(
    r: &mut crate::highs::diagnostics::Report,
    n: &Normalization,
    row_constants: &[f64],
) -> Result<(), ProblemError> {
    if let Some(v) = &mut r.primal_ray {
        values(v, &n.variables, false)?;
    }
    if let Some(v) = &mut r.dual_ray {
        duals(v, &n.rows, n.objective, false)?;
    }
    if let Some(v) = r.relaxation.as_mut().and_then(|r| r.primal.as_mut()) {
        values(v, &n.variables, false)?;
    }
    for (name, range) in &mut r.ranging {
        let row = name.starts_with("row_");
        let cost = name.starts_with("column_cost_");
        let scales = if row { &n.rows } else { &n.variables };
        if range.value.len() != scales.len() {
            return Err(ProblemError::Contract("ranging coordinate extent".into()));
        }
        for (i, v) in range.value.iter_mut().enumerate() {
            if v.is_finite() {
                *v = if cost {
                    div(mul(*v, n.objective)?, scales[i])?
                } else {
                    mul(*v, scales[i])?
                };
                if row {
                    *v += row_constants[i];
                    if !v.is_finite() {
                        return Err(ProblemError::Contract("ranging constant overflow".into()));
                    }
                }
            }
        }
        for v in &mut range.objective {
            if v.is_finite() {
                *v = mul(*v, n.objective)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> pse_ids::SemanticId {
        pse_ids::SemanticId::from_bytes([n; 16])
    }
    fn contract() -> OracleContract {
        OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![crate::Variable {
                id: id(1),
                lower: 0.0,
                upper: 10.0,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::Second,
            smoothness: pse_kernels::DerivativeOrder::Second,
        }
    }
    fn matrix(v: f64) -> faer::sparse::SparseColMat<usize, f64> {
        faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, v)],
        )
        .unwrap()
    }
    #[test]
    fn coordinate_transport_roots_preserves_residual_and_jacobian_products() {
        let mut roots = Roots::new(
            Box::new(crate::solver_tests::Polynomial::new()),
            Normalization {
                variables: vec![10.0],
                rows: vec![100.0],
                objective: 1.0,
            },
        )
        .unwrap();
        let mut residual = [0.0];
        roots.residual(&[0.2], &mut residual).unwrap();
        let mut original = crate::solver_tests::Polynomial::new();
        let mut expected = [0.0];
        original.residual(&[2.0], &mut expected).unwrap();
        assert_eq!(residual[0], expected[0] / 100.0);
        let mut jacobian = [0.0];
        roots.jacobian(&[0.2], &mut jacobian).unwrap();
        let mut product = [0.0];
        roots
            .jacobian_product(&[0.2], &[3.0], &mut product)
            .unwrap();
        assert!((product[0] - 3.0 * jacobian[0]).abs() < 1e-14);
        let observation = roots.observe(residual.to_vec()).unwrap();
        assert!(observation.values[0].is_finite());
    }
    #[test]
    fn coordinate_transport_coefficients_warm_and_integer_lattice() {
        let mut p = CoefficientProblem {
            contract: contract(),
            objective: vec![3.0],
            objective_constant: 5.0,
            sense: pse_math::binding::ObjectiveSense::Minimize,
            domains: vec![pse_math::binding::VariableDomain::Continuous],
            assumptions: pse_ids::ContentHash::from_bytes([2; 32]),
            constraints: matrix(4.0),
            hessian: Some(matrix(2.0)),
            bounds: vec![(2.0, 20.0)],
        };
        let proof = crate::GramCertificate::new(
            p.hessian.as_ref().unwrap(),
            1.0,
            &faer::Mat::identity(1, 1),
            &[2.0],
            10,
        )
        .unwrap();
        let n = Normalization {
            variables: vec![2.0],
            rows: vec![4.0],
            objective: 8.0,
        };
        let mut certificate = Certificate {
            kind: "test native proof".into(),
            primal: Some(vec![3.0]),
            dual: Some(vec![8.0, 2.0, 6.0]),
        };
        recover_certificate(&mut certificate, &n, &p.contract).unwrap();
        assert_eq!(certificate.primal.unwrap(), vec![6.0]);
        assert_eq!(certificate.dual.unwrap(), vec![2.0, 1.0, 3.0]);
        let (scaled, evidence) = coefficients(&p, &n, Some(&proof)).unwrap();
        assert_eq!(scaled.objective, vec![0.75]);
        assert_eq!(scaled.constraints.val(), &[2.0]);
        assert_eq!(scaled.hessian.as_ref().unwrap().val(), &[1.0]);
        assert_eq!(scaled.bounds, vec![(0.5, 5.0)]);
        scaled
            .validate_convex(evidence.as_ref().map(|e| -> &dyn QuadraticEvidence { e }))
            .unwrap();
        let seed = WarmStart {
            origin: None,
            compatibility: Compatibility {
                layout: p.contract.identity,
                data: p.assumptions,
                backend: Backend::Highs,
            },
            payload: WarmPayload::Highs {
                primal: Some(vec![6.0]),
                dual: Some((vec![12.0], vec![16.0])),
                basis: None,
            },
        };
        let native = warm(&seed, &n, true).unwrap();
        let WarmPayload::Highs { primal, dual, .. } = &native.payload else {
            panic!()
        };
        assert_eq!(primal.as_ref().unwrap(), &vec![3.0]);
        assert_eq!(dual.as_ref().unwrap(), &(vec![3.0], vec![8.0]));
        let recovered = warm(&native, &n, false).unwrap();
        let WarmPayload::Highs { primal, dual, .. } = recovered.payload else {
            panic!()
        };
        assert_eq!(primal.unwrap(), vec![6.0]);
        assert_eq!(dual.unwrap(), (vec![12.0], vec![16.0]));
        p.domains[0] = pse_math::binding::VariableDomain::Integer;
        assert!(coefficients(&p, &n, Some(&proof)).is_err());
    }
    #[test]
    fn coordinate_transport_cone_blocks_and_underflow() {
        use clarabel::{algebra::CscMatrix, solver::SupportedConeT};
        use pse_model::{generated::enums::NumericalTarget, numerics::NumericalPolicy};
        let q = pse_quantity::standard::standard_registry().unwrap();
        let qty = pse_quantity::standard::ids::quantity("neutral");
        let unit = q.quantity_type(qty).unwrap().canonical_unit;
        let mut c = contract();
        c.rows = vec![id(2), id(3)];
        let p = ConicProblem {
            contract: c,
            quadratic: CscMatrix::zeros((1, 1)),
            constraints: CscMatrix::zeros((2, 1)),
            objective: vec![0.0],
            objective_constant: 0.0,
            rhs: vec![2.0, 1.0],
            cones: vec![SupportedConeT::SecondOrderConeT(2)],
        };
        let targets = vec![
            pse_math::numerics::TargetSpec {
                id: id(1),
                kind: NumericalTarget::Variable,
                quantity: qty,
                unit,
                integer: false,
                declared_tolerance: None,
            },
            pse_math::numerics::TargetSpec {
                id: id(2),
                kind: NumericalTarget::Row,
                quantity: qty,
                unit,
                integer: false,
                declared_tolerance: None,
            },
            pse_math::numerics::TargetSpec {
                id: id(3),
                kind: NumericalTarget::Row,
                quantity: qty,
                unit,
                integer: false,
                declared_tolerance: None,
            },
        ];
        let mut policy =
            pse_math::numerics::resolve(&q, &targets, &[], &NumericalPolicy::default()).unwrap();
        policy.targets[1].coordinate_scale = 2.0;
        policy.targets[2].coordinate_scale = 8.0;
        assert_eq!(
            cone_normalization(&p, &policy).unwrap().rows,
            vec![8.0, 8.0]
        );
        policy.targets[1].required_scale = true;
        policy.targets[2].required_scale = true;
        assert!(cone_normalization(&p, &policy).is_err());
        assert!(div(f64::MIN_POSITIVE, f64::MAX).is_err());
        assert!(mul(f64::MAX, 2.0).is_err());
    }
}
