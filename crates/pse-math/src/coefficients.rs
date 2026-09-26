// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Symbolica coefficient projection with explicit assumptions and retained domain admission.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, Target},
    library,
    sparse::AssemblyMatrix,
};
use pse_ids::{ContentHash, FramedHasher};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::atom::{Atom, AtomCore};

/// A parameter-sensitive coefficient snapshot; it cannot stand in for future case values.
#[derive(Clone, Debug)]
pub struct Coefficients {
    structure: ContentHash,
    values: BTreeMap<pse_ids::SemanticId, u64>,
    /// Complete structure plus consumed fixed/parameter values.
    pub assumptions: ContentHash,
    /// Authored constant objective offset.
    pub objective_constant: f64,
    /// Authored linear objective coefficients in free-variable order.
    pub objective: Vec<f64>,
    /// Full symmetric Hessian in the authored objective sense.
    pub hessian: faer::sparse::SparseColMat<usize, f64>,
    /// Affine constraint coefficients in canonical sparse order.
    pub constraints: faer::sparse::SparseColMat<usize, f64>,
    /// Constant offset per constraint, kept for bound shifting.
    pub row_constants: Vec<f64>,
}
impl Coefficients {
    /// Known coefficient buffers, excluding map/allocator overhead.
    pub fn retained_bytes(&self) -> usize {
        let sparse = |m: &faer::sparse::SparseColMat<usize, f64>| {
            size_of_val(m.val()) + size_of_val(m.row_idx()) + size_of_val(m.symbolic().col_ptr())
        };
        size_of::<Self>()
            + sparse(&self.hessian)
            + sparse(&self.constraints)
            + (self.objective.capacity() + self.row_constants.capacity()) * size_of::<f64>()
            + self.values.len() * size_of::<(pse_ids::SemanticId, u64)>()
    }
    /// Reject a shared classification established from different consumed values.
    pub fn matches_facts(&self, facts: &crate::presolve::Facts) -> bool {
        self.structure == facts.structure && self.values == facts.values
    }
    /// Exact fixed/parameter assumptions consumed by this library-derived projection.
    pub fn matches_values(&self, values: &CaseValues) -> bool {
        self.values
            .iter()
            .all(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) == Some(*bits))
    }
    /// Structure against which this coefficient snapshot was derived.
    pub fn structure(&self) -> ContentHash {
        self.structure
    }
}
impl CasePlan {
    /// Derive degree <=2 objective and affine rows, rejecting opaque/non-polynomial terms.
    /// Every erased denominator/domain obligation must hold over the selected bounds.
    pub fn coefficients(
        &self,
        values: &CaseValues,
        term_limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Coefficients, MathError> {
        let facts = self.presolve_facts(values, term_limit, cancel)?;
        self.coefficients_with_facts(values, &facts, term_limit, cancel)
    }
    /// Consume the current shared bound facts rather than reclassifying affine rows.
    pub fn coefficients_with_facts(
        &self,
        values: &CaseValues,
        facts: &crate::presolve::Facts,
        term_limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Coefficients, MathError> {
        if !facts.matches(self, values)
            || facts.affine.iter().any(Option::is_none)
            || facts.objective_degree.is_none_or(|d| d > 2)
            || facts
                .obligations
                .values()
                .any(|s| *s != crate::presolve::ObligationStatus::Discharged)
        {
            return Err(MathError::Contract(
                "coefficient projection requires current affine and domain facts".into(),
            ));
        }
        if values.scalars.values().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "nonfinite coefficient assumption".into(),
            ));
        }
        for variable in self.structure().variables().iter().filter(|v| v.fixed) {
            if let Some(&value) = values.scalars.get(&variable.port.id)
                && !variable.domain.contains(
                    value,
                    variable.lower.unwrap_or(f64::NEG_INFINITY),
                    variable.upper.unwrap_or(f64::INFINITY),
                )
            {
                return Err(MathError::Contract(
                    "fixed coefficient value outside declared domain".into(),
                ));
            }
        }
        if term_limit == 0 {
            return Err(MathError::Limit("polynomial terms"));
        }
        let n = self.columns().len();
        let m = self.structure().rows().len();
        let columns: BTreeMap<_, _> = self
            .columns()
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();
        let mut identity = FramedHasher::new("pse.math.coefficient-assumptions.v1");
        identity.hash(&self.structure().key());
        let mut objective = vec![0.0; n];
        let mut constant = 0.0;
        let mut row_constants = Vec::with_capacity(m);
        let mut jp = vec![];
        let mut jv = vec![];
        for (row, affine) in facts.affine.iter().enumerate() {
            let affine = affine
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing affine row".into()))?;
            row_constants.push(affine.constant);
            for (&column, &value) in &affine.entries {
                jp.push((row, column));
                jv.push(value);
            }
        }
        if jp.len() > term_limit {
            return Err(MathError::Limit("affine coefficient entries"));
        }
        let mut hp = vec![];
        let mut hv = vec![];
        for b in self.structure().instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            identity.hash(&b.body);
            let body = &self.bodies()[&b.body];
            let mut replacements = vec![];
            let mut formals = vec![];
            let mut local_columns = vec![];
            for (i, slot) in b.slots.iter().enumerate() {
                let replacement = if let Some(&col) = columns.get(&slot.source()) {
                    let atom = library::formal(i)?;
                    formals.push(atom.clone());
                    local_columns.push(col);
                    atom * Atom::num(slot.scale()) + Atom::num(slot.offset())
                } else {
                    let value = *values.scalars.get(&slot.source()).ok_or_else(|| {
                        MathError::Contract("missing coefficient parameter".into())
                    })?;
                    identity.u64(value.to_bits());
                    Atom::num(slot.scale() * value + slot.offset())
                };
                replacements.push((library::formal(i)?, replacement));
            }
            // Substitution cannot introduce another slot: each formal keeps its own index.
            let replace = |atom: &Atom| {
                replacements.iter().fold(atom.clone(), |a, (from, to)| {
                    a.replace(from.clone()).with(to.clone())
                })
            };
            for c in &b.contributions {
                if c.target != Target::Objective {
                    continue;
                }
                let expression = replace(body.expression(c.output).ok_or_else(|| {
                    MathError::Contract(
                        "opaque or switching output is not a coefficient model".into(),
                    )
                })?);
                // Prove degree before expansion: constant second partials bound the
                // expanded quadratic term count. Symbolica owns every derivative and expansion.
                let width = formals.len();
                if width
                    .checked_add(1)
                    .and_then(|w| w.checked_mul(w + 1))
                    .map(|v| v / 2)
                    .is_none_or(|v| v > term_limit)
                {
                    return Err(MathError::Limit("quadratic coefficient capacity"));
                }
                let polynomial = expression
                    .expand()
                    .to_polynomial_in_vars::<u16>(formals.clone());
                if polynomial.nterms() > term_limit {
                    return Err(MathError::Limit("polynomial terms"));
                }
                for (term, coefficient) in polynomial.coefficients.iter().enumerate() {
                    if !coefficient.is_constant() {
                        return Err(MathError::Contract("non-polynomial coefficient".into()));
                    }
                    let exponents = polynomial.exponents(term);
                    let degree: usize = exponents.iter().map(|&e| usize::from(e)).sum();
                    if degree > 2 {
                        return Err(MathError::Contract("unsupported coefficient degree".into()));
                    }
                    let v = number(coefficient, cancel)? * c.scale;
                    let factors: Vec<_> = exponents
                        .iter()
                        .enumerate()
                        .flat_map(|(i, &e)| std::iter::repeat_n(local_columns[i], usize::from(e)))
                        .collect();
                    match (c.target, factors.as_slice()) {
                        (Target::Objective, []) => constant += v,
                        (Target::Objective, [i]) => objective[*i] += v,
                        (Target::Objective, [i, j]) => {
                            hp.push((*i, *j));
                            hv.push(v);
                            hp.push((*j, *i));
                            hv.push(v);
                        }
                        _ => return Err(MathError::Contract("coefficient degree mapping".into())),
                    }
                    if jp
                        .len()
                        .checked_add(hp.len())
                        .is_none_or(|k| k > term_limit)
                    {
                        return Err(MathError::Limit("global coefficient terms"));
                    }
                }
            }
        }
        let index_limit = term_limit.max(n).max(m);
        let mut j = AssemblyMatrix::new(m, n, &jp, index_limit)?;
        for (i, v) in jv.into_iter().enumerate() {
            j.add(i, v)?;
        }
        let mut h = AssemblyMatrix::new(n, n, &hp, index_limit)?;
        for (i, v) in hv.into_iter().enumerate() {
            h.add(i, v)?;
        }
        if !constant.is_finite()
            || objective
                .iter()
                .chain(&row_constants)
                .any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "nonfinite coefficient projection".into(),
            ));
        }
        Ok(Coefficients {
            structure: self.structure().key(),
            values: self
                .structure()
                .instances()
                .iter()
                .flat_map(|b| b.slots.iter())
                .filter(|s| !columns.contains_key(&s.source()))
                .map(|s| (s.source(), values.scalars[&s.source()].to_bits()))
                .collect(),
            assumptions: identity.finish_hash(),
            objective_constant: constant,
            objective,
            hessian: h.matrix().clone(),
            constraints: j.matrix().clone(),
            row_constants,
        })
    }
}
pub(crate) fn number(atom: &Atom, cancel: &Arc<AtomicBool>) -> Result<f64, MathError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(MathError::Cancelled);
    }
    let output = atom
        .evaluate(&std::collections::HashMap::<Atom, f64>::new())
        .map_err(|e| MathError::Library(e.to_string()))?;
    if cancel.load(Ordering::Relaxed) {
        return Err(MathError::Cancelled);
    }
    if !output.is_finite() {
        return Err(MathError::Contract("nonfinite library coefficient".into()));
    }
    Ok(output)
}

/// Exact represented-rational verification of Q = sign * Rᵀ diag(w) R, w >= 0.
/// The certificate is tied to every numeric matrix entry and the objective orientation.
#[derive(Clone, Debug)]
pub struct GramCertificate {
    identity: ContentHash,
}
impl GramCertificate {
    /// Verify a supplied sum-of-squares witness through Symbolica/Numerica rational arithmetic.
    pub fn new(
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
        factors: &faer::Mat<f64>,
        weights: &[f64],
        operation_limit: usize,
    ) -> Result<Self, MathError> {
        let n = q.ncols();
        let k = factors.nrows();
        if q.nrows() != n
            || factors.ncols() != n
            || weights.len() != k
            || !matches!(sign, -1.0 | 1.0)
            || weights.iter().any(|w| !w.is_finite() || *w < 0.0)
            || n.checked_mul(n)
                .and_then(|v| v.checked_mul(k.max(1)))
                .is_none_or(|v| v > operation_limit)
        {
            return Err(MathError::Contract(
                "Gram witness shape, weight, or budget".into(),
            ));
        }
        let rational = |v: f64| -> Result<Atom, MathError> {
            if !v.is_finite() {
                return Err(MathError::Contract("nonfinite Gram entry".into()));
            }
            let f = symbolica::domains::float::Float::from(v);
            let r = symbolica::domains::rational::Rational::try_from(f)
                .map_err(|e| MathError::Library(e.to_string()))?;
            Ok(Atom::num(r))
        };
        let dense = q.to_dense();
        for i in 0..n {
            for j in 0..n {
                let mut gram = Atom::num(0);
                for t in 0..k {
                    gram += rational(weights[t])?
                        * rational(factors[(t, i)])?
                        * rational(factors[(t, j)])?;
                }
                if gram != rational(sign * dense[(i, j)])? {
                    return Err(MathError::Contract(
                        "quadratic matrix differs from exact Gram witness".into(),
                    ));
                }
            }
        }
        Ok(Self {
            identity: quadratic_identity(q, sign),
        })
    }
    /// Reject a stale witness after any numeric matrix or sense change.
    pub fn validate(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError> {
        if quadratic_identity(q, sign) != self.identity {
            return Err(MathError::Contract("stale convexity evidence".into()));
        }
        Ok(())
    }
}
pub(crate) fn quadratic_identity(
    q: &faer::sparse::SparseColMat<usize, f64>,
    sign: f64,
) -> ContentHash {
    let mut h = FramedHasher::new("pse.math.gram.v1");
    h.u64(q.nrows() as u64)
        .u64(q.ncols() as u64)
        .u64(sign.to_bits());
    for c in 0..q.ncols() {
        for (&r, &v) in q
            .symbolic()
            .row_idx_of_col_raw(c)
            .iter()
            .zip(q.val_of_col(c))
        {
            if v != 0.0 {
                h.u64(r as u64).u64(c as u64).u64(v.to_bits());
            }
        }
    }
    h.finish_hash()
}
