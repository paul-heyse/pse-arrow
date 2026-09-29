// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Symbolica coefficient projection with explicit assumptions and retained domain admission.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, Target},
    index::{Addend, Entry, GlobalCol, GlobalRow},
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
    /// The linear coefficients and constant of each later lexicographic objective, in
    /// optimization order after the primary one (ADR-0111); empty with one objective. A
    /// later objective with a quadratic term has no coefficient projection.
    pub lexicographic: Vec<(Vec<f64>, f64)>,
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
            + self
                .lexicographic
                .iter()
                .map(|(c, _)| (c.capacity() + 1) * size_of::<f64>())
                .sum::<usize>()
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
        if !facts.matches(self, values) || !facts.coefficient_eligible() {
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
            .map(|(i, &id)| (id, GlobalCol::new(i)))
            .collect();
        let mut identity = FramedHasher::new(pse_ids::Frame::MathCoefficientAssumptionsV1);
        identity.hash(&self.structure().key());
        let mut objective = vec![0.0; n];
        let mut constant = 0.0;
        let mut lexicographic =
            vec![(vec![0.0; n], 0.0); self.structure().objectives().len().saturating_sub(1)];
        let mut row_constants = Vec::with_capacity(m);
        let mut jp = Vec::<Entry<GlobalRow, GlobalCol>>::new();
        let mut jv = vec![];
        for (row, affine) in facts.affine.iter().enumerate() {
            let affine = affine
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing affine row".into()))?;
            row_constants.push(affine.constant);
            for (&column, &value) in &affine.entries {
                jp.push(Entry::new(GlobalRow::new(row), GlobalCol::new(column)));
                jv.push(value);
            }
        }
        if jp.len() > term_limit {
            return Err(MathError::Limit("affine coefficient entries"));
        }
        let mut hp = Vec::<Entry<GlobalCol, GlobalCol>>::new();
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
                let Target::Objective(level) = c.target else {
                    continue;
                };
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
                    match (level, factors.as_slice()) {
                        (0, []) => constant += v,
                        (0, [i]) => objective[i.get()] += v,
                        (0, [i, j]) => {
                            hp.push(Entry::new(*i, *j));
                            hv.push(v);
                            hp.push(Entry::new(*j, *i));
                            hv.push(v);
                        }
                        (later, []) => lexicographic[later - 1].1 += v,
                        (later, [i]) => lexicographic[later - 1].0[i.get()] += v,
                        (_, [_, _]) => {
                            return Err(MathError::Contract(
                                "a later lexicographic objective is not linear".into(),
                            ));
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
            j.add(Addend::new(i), v)?;
        }
        let mut h = AssemblyMatrix::hessian(n, &hp, index_limit)?;
        for (i, v) in hv.into_iter().enumerate() {
            h.add(Addend::new(i), v)?;
        }
        if !constant.is_finite()
            || objective
                .iter()
                .chain(&row_constants)
                .chain(lexicographic.iter().flat_map(|(c, k)| c.iter().chain([k])))
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
            lexicographic,
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
        return Err(MathError::CoefficientRange);
    }
    Ok(output)
}
