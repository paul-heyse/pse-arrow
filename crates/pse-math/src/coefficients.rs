// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Symbolica coefficient projection with explicit assumptions and retained domain admission.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::CaseValues,
    class_evidence::{ExactObjectives, aggregate_objectives, shape_facts_with_objectives},
    index::{Addend, Entry, GlobalCol, GlobalRow},
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
        let base = self.presolve_domain_facts(values, term_limit, cancel)?;
        let (facts, objectives) =
            shape_facts_with_objectives(self, values, &base, term_limit, cancel, true)?;
        self.project_coefficients(values, &facts, term_limit, cancel, objectives)
            .map(|(coefficients, _)| coefficients)
    }
    /// Consume the current shared bound facts rather than reclassifying affine rows.
    pub fn coefficients_with_facts(
        &self,
        values: &CaseValues,
        facts: &crate::presolve::Facts,
        term_limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Coefficients, MathError> {
        self.coefficient_projection(values, facts, term_limit, cancel)
            .map(|(coefficients, _)| coefficients)
    }
    pub(crate) fn coefficient_projection(
        &self,
        values: &CaseValues,
        facts: &crate::presolve::Facts,
        term_limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(Coefficients, usize), MathError> {
        self.project_coefficients(values, facts, term_limit, cancel, None)
    }

    pub(super) fn project_coefficients(
        &self,
        values: &CaseValues,
        facts: &crate::presolve::Facts,
        term_limit: usize,
        cancel: &Arc<AtomicBool>,
        objectives: Option<ExactObjectives>,
    ) -> Result<(Coefficients, usize), MathError> {
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
        let mut allowance = crate::class_evidence::Allowance {
            remaining: term_limit.min(facts.proof_remaining),
        };
        let n = self.columns().len();
        let m = self.structure().rows().len();
        let levels = self.structure().objectives().len().max(1);
        allowance.charge(
            n.checked_mul(levels)
                .and_then(|v| v.checked_add(m))
                .ok_or(MathError::Limit("coefficient buffers"))?,
        )?;
        allowance.charge(
            facts
                .affine
                .iter()
                .flatten()
                .map(|row| row.entries.len())
                .sum(),
        )?;
        let mut identity = FramedHasher::new(pse_ids::Frame::MathCoefficientAssumptionsV2);
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
        let mut hp = Vec::<Entry<GlobalCol, GlobalCol>>::new();
        let mut hv = vec![];
        for binding in self.structure().instances() {
            identity.hash(&binding.body);
            for slot in binding.slots.iter() {
                if !self.columns().contains(&slot.source()) {
                    let value = values.scalars.get(&slot.source()).ok_or_else(|| {
                        MathError::Contract("missing coefficient parameter".into())
                    })?;
                    identity.u64(value.to_bits());
                }
            }
        }
        let ExactObjectives {
            formals,
            expressions,
        } = if let Some(objectives) = objectives {
            objectives
        } else {
            // A separately requested projection can consume retained facts, which
            // deliberately carry no objective expressions across requests.
            let (objectives, remaining) =
                aggregate_objectives(self, values, allowance.remaining, cancel)?;
            allowance.remaining = remaining;
            objectives
        };
        let width = formals.len();
        let capacity = width
            .checked_add(1)
            .and_then(|w| w.checked_mul(w + 1))
            .map(|v| v / 2)
            .ok_or(MathError::Limit("quadratic coefficient capacity"))?;
        for (level, objective_view) in expressions.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            allowance.charge(capacity)?;
            let polynomial = objective_view.polynomial;
            if polynomial.nterms() > capacity {
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
                let v = number(coefficient, cancel)?;
                allowance.charge(degree)?;
                let factors: Vec<_> = exponents
                    .iter()
                    .enumerate()
                    .flat_map(|(i, &e)| std::iter::repeat_n(GlobalCol::new(i), usize::from(e)))
                    .collect();
                match (level, factors.as_slice()) {
                    (0, []) => constant += v,
                    (0, [i]) => objective[i.get()] += v,
                    (0, [i, j]) => {
                        allowance.charge(2)?;
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
            }
        }
        let index_limit = term_limit.max(n).max(m);
        allowance.charge(
            jp.len()
                .checked_add(hp.len())
                .and_then(|v| v.checked_add(n.checked_mul(2)?))
                .ok_or(MathError::Limit("coefficient sparse buffers"))?,
        )?;
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
        allowance.charge(
            self.structure()
                .instances()
                .iter()
                .flat_map(|b| b.slots.iter())
                .filter(|s| !self.columns().contains(&s.source()))
                .count(),
        )?;
        Ok((
            Coefficients {
                structure: self.structure().key(),
                values: self
                    .structure()
                    .instances()
                    .iter()
                    .flat_map(|b| b.slots.iter())
                    .filter(|s| !self.columns().contains(&s.source()))
                    .map(|s| (s.source(), values.scalars[&s.source()].to_bits()))
                    .collect(),
                assumptions: identity.finish_hash(),
                objective_constant: constant,
                objective,
                lexicographic,
                hessian: h.matrix().clone(),
                constraints: j.matrix().clone(),
                row_constants,
            },
            allowance.remaining,
        ))
    }
}
pub(crate) fn number(atom: &Atom, cancel: &Arc<AtomicBool>) -> Result<f64, MathError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(MathError::Cancelled);
    }
    let rational = match atom.as_view() {
        symbolica::atom::AtomView::Num(number) => match number.get_coeff_view() {
            symbolica::coefficient::CoefficientView::Natural(n, d, 0, _) if d != 0 => {
                Some(symbolica::domains::rational::Rational::new(n, d))
            }
            symbolica::coefficient::CoefficientView::Large(re, im) if im.is_zero() => {
                Some(re.to_rat())
            }
            _ => None,
        },
        _ => None,
    };
    let output = if let Some(rational) = rational {
        crate::rational_binary64::nearest(&rational)?
    } else {
        atom.evaluate(&std::collections::HashMap::<Atom, f64>::new())
            .map_err(|e| MathError::Library(e.to_string()))?
    };
    if cancel.load(Ordering::Relaxed) {
        return Err(MathError::Cancelled);
    }
    // Numeric projection may round coefficients, but may not erase an exact
    // source dependency or offset. In particular an underflowed nonzero must
    // never become an absent affine entry or a proved zero constant.
    if !output.is_finite() || (output == 0.0 && !atom.is_zero()) {
        return Err(MathError::CoefficientRange);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficient_projection_refuses_nonzero_underflow_without_erasing_source_dependence() {
        crate::initialize().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let small = Atom::num(f64::MIN_POSITIVE);
        for expression in [&small * &small, -(&small * &small)] {
            assert!(!expression.is_zero());
            assert!(matches!(
                number(&expression, &cancel),
                Err(MathError::CoefficientRange)
            ));
        }
        assert_eq!(number(&Atom::num(0), &cancel).unwrap(), 0.0);
        for value in [f64::MIN_POSITIVE, -f64::MIN_POSITIVE, 0.5, -0.5] {
            assert_eq!(
                number(&Atom::num(value), &cancel).unwrap().to_bits(),
                value.to_bits()
            );
        }
    }
}
