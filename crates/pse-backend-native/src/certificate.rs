// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Verification of infeasibility and unboundedness rays against the original problem data
//! (Plan 22 I11), whichever adapter found them. A ray is recomputed over the cone form
//! `Ax + s = b, s ∈ K` with finite variable bounds appended as rows
//! ([`crate::conic::bound_rows`]):
//!
//! - a Farkas ray `y` needs `Aᵀy = 0`, `y ∈ K*` and `bᵀy < 0`; it is verified only when
//!   `bᵀy` stays negative after every row and bound is relaxed by its acceptance budget,
//!   so a problem infeasible by less than its budgets is never certified;
//! - a recession direction `x` needs `Px = 0`, `-Ax ∈ K` and `qᵀx < 0`.
//!
//! Residuals are relative to the magnitudes they sum and cone violations to the ray's
//! largest entry, so the verdict is invariant under the positive row and column scalings
//! of normalization. A column residual beyond the declared tolerance is bounded over the
//! column's finite box; over an unbounded column it defeats the proof.
use crate::{
    CoefficientProblem, ConicProblem,
    conic::{self, Cone, lowering},
    quality::Tolerances,
    solve::{
        CertificateAccuracy, CertificateKind, CertificateVerification, InfeasibilityCertificate,
        RayCoordinate, RayEntry, Termination,
    },
};

/// Verify `certificate` against the original cone form `p`, whose cone rows carry the
/// acceptance budgets `t.rows` and whose variables `t.variables`. A ray that does not
/// match the layout, or whose entries are not finite, is left unverified.
pub(crate) fn verify(
    certificate: &mut InfeasibilityCertificate,
    p: &ConicProblem,
    t: &Tolerances,
    tolerance: f64,
) {
    let values: Vec<f64> = certificate.ray.iter().map(|e| e.value).collect();
    let scale = values.iter().fold(0.0f64, |a, v| a.max(v.abs()));
    certificate.verification = if !(scale > 0.0 && scale.is_finite())
        || t.rows.len() != p.rhs.len()
        || t.variables.len() != p.contract.variables.len()
    {
        None
    } else {
        match certificate.kind {
            CertificateKind::PrimalInfeasible => farkas(p, &values, scale, t, tolerance),
            CertificateKind::DualInfeasible => recession(p, &values, scale, tolerance),
        }
    };
}
/// Verify a certificate of a coefficient problem against the cone form of its original
/// data, the layout [`crate::ConicProblem::from_coefficients`] lowers to.
pub(crate) fn verify_coefficients(
    certificate: &mut InfeasibilityCertificate,
    p: &CoefficientProblem,
    t: &Tolerances,
    tolerance: f64,
) {
    match lowering::data(p) {
        Ok((cone, rows)) => verify(
            certificate,
            &cone,
            &lowering::row_budgets(&rows, t),
            tolerance,
        ),
        Err(_) => certificate.verification = None,
    }
}
/// The dual cone violation of `z`: zero, nonnegative, second-order and PSD cones are
/// self-dual; `(u, v, w) ∈ K*_exp` iff `(u − v, −u, w) ∈ K_exp`; `(u, v, w) ∈ K*_pow(α)` iff
/// `(u/α, v/(1 − α), w) ∈ K_pow(α)`, and likewise for the generalized power cone.
fn dual_violation(cone: &Cone, z: &[f64]) -> Option<f64> {
    let mapped = match cone {
        Cone::Zero { .. } => return Some(0.0),
        Cone::Nonnegative { .. } | Cone::SecondOrder { .. } | Cone::PsdTriangle { .. } => {
            z.to_vec()
        }
        Cone::Exponential => vec![z[0] - z[1], -z[0], z[2]],
        Cone::Power { alpha } => vec![z[0] / alpha, z[1] / (1.0 - alpha), z[2]],
        Cone::GeneralizedPower { alpha, .. } => z
            .iter()
            .enumerate()
            .map(|(i, v)| alpha.get(i).map_or(*v, |a| v / a))
            .collect(),
    };
    conic::cone_violation(cone, &mapped).ok()
}
/// The Farkas conditions of `y` (cone rows then bound rows).
fn farkas(
    p: &ConicProblem,
    y: &[f64],
    scale: f64,
    t: &Tolerances,
    tolerance: f64,
) -> Option<CertificateVerification> {
    let variables = &p.contract.variables;
    let bounds = conic::bound_rows(variables);
    let m = p.rhs.len();
    if y.len() != m + bounds.len() || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let (rows, bound_values) = y.split_at(m);
    // bᵀy and the acceptance budgets the relaxed problem may use.
    let mut objective = 0.0;
    let mut budget = 0.0;
    for ((b, y), t) in p.rhs.iter().zip(rows).zip(&t.rows) {
        objective += b * y;
        budget += y.abs() * t;
    }
    for (&(j, lower), y) in bounds.iter().zip(bound_values) {
        let v = &variables[j];
        objective += if lower { -v.lower } else { v.upper } * y;
        budget += y.abs() * t.variables[j];
    }
    // Aᵀy per column, with the magnitudes it sums.
    let n = variables.len();
    let mut residual = vec![0.0; n];
    let mut magnitude = vec![0.0; n];
    for (c, (r, m)) in residual.iter_mut().zip(&mut magnitude).enumerate() {
        for k in p.constraints.column(c) {
            let term = p.constraints.values[k] * rows[p.constraints.row_indices[k]];
            *r += term;
            *m += term.abs();
        }
    }
    for (&(j, lower), y) in bounds.iter().zip(bound_values) {
        let term = if lower { -y } else { *y };
        residual[j] += term;
        magnitude[j] += term.abs();
    }
    let mut worst = 0.0f64;
    let mut excess = 0.0;
    for ((r, m), v) in residual.iter().zip(&magnitude).zip(variables) {
        let relative = if *m > 0.0 { r.abs() / m } else { r.abs() };
        worst = worst.max(relative);
        let reach = v.lower.abs().max(v.upper.abs());
        if reach.is_finite() {
            // Over a finite box, sup rᵀx is bounded exactly.
            excess += r.abs() * reach;
        } else if relative > tolerance {
            excess = f64::INFINITY;
        }
    }
    let mut cone = 0.0f64;
    let mut start = 0;
    for block in &p.cones {
        let end = start + block.dim();
        cone = cone.max(dual_violation(block, rows.get(start..end)?)?);
        start = end;
    }
    cone = cone.max(bound_values.iter().fold(0.0, |a, v| a.max(-v)));
    let margin = -objective - budget - excess;
    Some(CertificateVerification {
        residual: worst,
        objective: objective / scale,
        cone: cone / scale,
        margin: margin / scale,
        tolerance,
        verified: margin > 0.0 && cone / scale <= tolerance,
    })
}
/// The recession conditions of `x` over the variables.
fn recession(
    p: &ConicProblem,
    x: &[f64],
    scale: f64,
    tolerance: f64,
) -> Option<CertificateVerification> {
    let variables = &p.contract.variables;
    let n = variables.len();
    if x.len() != n || x.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let relative = |value: f64, magnitude: f64| {
        if magnitude > 0.0 {
            value.abs() / magnitude
        } else {
            value.abs()
        }
    };
    // Px from the upper-triangle storage.
    let mut px = vec![0.0; n];
    let mut pm = vec![0.0; n];
    for c in 0..n {
        for k in p.quadratic.column(c) {
            let r = p.quadratic.row_indices[k];
            let v = p.quadratic.values[k];
            px[r] += v * x[c];
            pm[r] += (v * x[c]).abs();
            if r != c {
                px[c] += v * x[r];
                pm[c] += (v * x[r]).abs();
            }
        }
    }
    let mut worst = px
        .iter()
        .zip(&pm)
        .fold(0.0f64, |a, (v, m)| a.max(relative(*v, *m)));
    // s = -Ax must lie in the cone.
    let m = p.rhs.len();
    let mut ax = vec![0.0; m];
    let mut am = vec![0.0; m];
    for (c, x) in x.iter().enumerate() {
        for k in p.constraints.column(c) {
            let r = p.constraints.row_indices[k];
            ax[r] += p.constraints.values[k] * x;
            am[r] += (p.constraints.values[k] * x).abs();
        }
    }
    let mut cone = 0.0f64;
    let mut start = 0;
    for block in &p.cones {
        let end = start + block.dim();
        match block {
            Cone::Zero { .. } => {
                for r in start..end {
                    worst = worst.max(relative(ax[r], am[r]));
                }
            }
            Cone::Nonnegative { .. } => {
                for r in start..end {
                    cone = cone.max(relative(ax[r].max(0.0), am[r]));
                }
            }
            _ => {
                let s: Vec<f64> = ax[start..end].iter().map(|v| -v).collect();
                let magnitude = am[start..end].iter().fold(0.0f64, |a, v| a.max(*v));
                cone = cone.max(relative(conic::cone_violation(block, &s).ok()?, magnitude));
            }
        }
        start = end;
    }
    for (j, lower) in conic::bound_rows(variables) {
        // -x + s = -l: s = x_j >= 0; x + s = u: s = -x_j >= 0.
        let violation = if lower { -x[j] } else { x[j] };
        cone = cone.max(violation.max(0.0) / scale);
    }
    let (objective, magnitude) = p
        .objective
        .iter()
        .zip(x)
        .fold((0.0, 0.0), |(o, m), (q, x)| (o + q * x, m + (q * x).abs()));
    Some(CertificateVerification {
        residual: worst,
        objective: objective / scale,
        cone,
        margin: -objective / scale,
        tolerance,
        verified: objective < -tolerance * magnitude
            && magnitude > 0.0
            && worst <= tolerance
            && cone <= tolerance,
    })
}
/// The cone-form Farkas ray of a row ray `y` over the rows `L <= Ax <= U` of `p` (HiGHS's
/// dual ray): each row's multiplier on the side its sign selects, and the bound multipliers
/// that cancel `Aᵀy` where the bound is finite. HiGHS's sign convention is not assumed: of
/// `y` and `-y`, the orientation with the smaller `bᵀz` is kept; either orientation that
/// verifies is a proof. `None` for a problem the cone form does not represent.
#[cfg_attr(
    not(feature = "highs"),
    expect(dead_code, reason = "only HiGHS exports row rays")
)]
pub(crate) fn farkas_from_rows(p: &CoefficientProblem, y: &[f64]) -> Option<Vec<RayEntry>> {
    let (cone, rows) = lowering::data(p).ok()?;
    if y.len() != p.bounds.len() {
        return None;
    }
    let mut d = vec![0.0; p.contract.variables.len()];
    for (c, d) in d.iter_mut().enumerate() {
        for (r, v) in p
            .constraints
            .row_idx_of_col(c)
            .zip(p.constraints.val_of_col(c))
        {
            *d += v * y[r];
        }
    }
    let bounds = conic::bound_rows(&p.contract.variables);
    let orient = |sigma: f64| -> Vec<f64> {
        let mut z: Vec<f64> = rows
            .iter()
            .map(|r| {
                let y = sigma * y[r.row];
                match r.side {
                    lowering::RowSide::Equal => -y,
                    lowering::RowSide::Upper => (-y).max(0.0),
                    lowering::RowSide::Lower => y.max(0.0),
                }
            })
            .collect();
        // Rows contribute -σAᵀy; the bound rows (-x on a lower, +x on an upper) cancel it.
        z.extend(bounds.iter().map(|&(j, lower)| {
            let d = sigma * d[j];
            if lower { (-d).max(0.0) } else { d.max(0.0) }
        }));
        z
    };
    let b: Vec<f64> = cone
        .rhs
        .iter()
        .copied()
        .chain(bounds.iter().map(|&(j, lower)| {
            let v = &p.contract.variables[j];
            if lower { -v.lower } else { v.upper }
        }))
        .collect();
    let dot = |z: &[f64]| z.iter().zip(&b).map(|(z, b)| z * b).sum::<f64>();
    let (plus, minus) = (orient(1.0), orient(-1.0));
    let z = if dot(&minus) < dot(&plus) { minus } else { plus };
    let coordinates = rows
        .iter()
        .map(|r| (r.side.coordinate(), p.contract.rows[r.row]))
        .chain(bounds.iter().map(|&(j, lower)| {
            (
                if lower {
                    RayCoordinate::VariableLower
                } else {
                    RayCoordinate::VariableUpper
                },
                p.contract.variables[j].id,
            )
        }));
    Some(
        coordinates
            .zip(z)
            .map(|((coordinate, id), value)| RayEntry {
                coordinate,
                id,
                value,
            })
            .collect(),
    )
}
/// The certificate of HiGHS's exported rays for a continuous LP in native coordinates: a
/// dual ray proves infeasibility, a primal ray unboundedness (when the rows admit a point).
#[cfg_attr(
    not(feature = "highs"),
    expect(dead_code, reason = "only HiGHS exports row rays")
)]
pub(crate) fn from_highs_rays(
    p: &CoefficientProblem,
    termination: Termination,
    dual: Option<&[f64]>,
    primal: Option<&[f64]>,
) -> Option<InfeasibilityCertificate> {
    let (kind, ray) = match (termination, dual, primal) {
        (Termination::Infeasible | Termination::InfeasibleOrUnbounded, Some(y), _) => {
            (CertificateKind::PrimalInfeasible, farkas_from_rows(p, y)?)
        }
        (Termination::Unbounded | Termination::InfeasibleOrUnbounded, _, Some(x)) => (
            CertificateKind::DualInfeasible,
            p.contract
                .variables
                .iter()
                .zip(x)
                .map(|(v, x)| RayEntry {
                    coordinate: RayCoordinate::Variable,
                    id: v.id,
                    value: *x,
                })
                .collect(),
        ),
        _ => return None,
    };
    Some(InfeasibilityCertificate {
        kind,
        accuracy: CertificateAccuracy::Full,
        ray,
        verification: None,
    })
}
