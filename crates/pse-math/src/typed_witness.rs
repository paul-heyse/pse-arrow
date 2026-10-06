// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Abstract scientific calls for a disposable, physically admitted witness pass.
use super::*;
use std::collections::BTreeMap;
use symbolica::atom::Symbol;

/// Abstract scientific function/partial identities. Implementations are deliberately
/// absent: an ideal or zero residual potential remains a lawful scientific witness.
#[derive(Clone, Debug, Default)]
pub struct ScientificWitness {
    functions: BTreeMap<(SemanticId, Vec<usize>), usize>,
    symbols: Vec<Symbol>,
}

impl BodyBuilder<'_> {
    /// Start an isolated scientific pass without copying earlier numerical programs.
    /// Formal-slot identities stay disjoint from the caller's already admitted values.
    /// # Errors
    /// Invalid existing construction limits.
    pub fn scientific_pass(&self) -> Result<BodyBuilder<'_>, MathError> {
        let mut pass = BodyBuilder::new(
            self.context,
            self.registry,
            self.checker,
            self.inputs,
            self.limits,
        )?;
        pass.input_quantities.clone_from(&self.input_quantities);
        pass.next_slot = self.next_slot;
        pass.formula_authority.clone_from(&self.formula_authority);
        Ok(pass)
    }

    /// Preserve a checked formal's physical contract while making its shadow value
    /// independent of numerical arguments supplied at a particular evaluation point.
    /// # Errors
    /// The bounded shadow pass has no remaining formal slot.
    pub fn scientific_argument(&mut self, mut value: TypedValue) -> Result<TypedValue, MathError> {
        self.tick()?;
        value.atom = library::formal(self.slot()?)?;
        value.effects.clear();
        Ok(value)
    }

    /// Replace one already checked scientific call with its abstract function value.
    /// The same selected function and partial coordinates use the same library symbol;
    /// applied arguments remain explicit so identical calls cancel under normalization.
    /// # Errors
    /// Invalid partial coordinates, physical derivative contract or construction bound.
    pub fn scientific_placeholder(
        &mut self,
        witness: &mut ScientificWitness,
        function: SemanticId,
        result: ResolvedPhysicalContract,
        arguments: &[TypedValue],
        partial: &[usize],
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        self.tick()?;
        if partial.len() > 8 || partial.iter().any(|index| *index >= arguments.len()) {
            return Err(MathError::Contract(
                "abstract scientific partial coordinates".into(),
            ));
        }
        let mut coordinates = partial.to_vec();
        // These are partials of an admitted smooth scientific potential.
        coordinates.sort_unstable();
        let key = (function, coordinates);
        let slot = if let Some(slot) = witness.functions.get(&key) {
            *slot
        } else {
            let slot = self.slot()?;
            witness.functions.insert(key, slot);
            witness.symbols.push(self.context.pool.function(slot)?);
            slot
        };
        let quantity = if partial.is_empty() {
            result
        } else {
            let contracts = partial
                .iter()
                .map(|index| arguments[*index].quantity.clone())
                .collect::<Vec<_>>();
            let admission = pse_quantity::resolved::infer_partial(
                &result,
                &contracts,
                self.registry,
                self.checker,
            )?;
            self.admissions.push(admission.clone());
            admission.result
        };
        Ok(TypedValue {
            atom: library::function(
                slot,
                &arguments
                    .iter()
                    .map(|value| value.atom.clone())
                    .collect::<Vec<_>>(),
            )?,
            indices: quantity.indices().clone(),
            quantity,
            effects: BTreeSet::new(),
            source,
        })
    }

    /// Test mathematical dependence after resolving this shadow pass's shared bindings.
    /// Symbolica owns expansion, common-denominator normalization and cancellation.
    /// No numerical potential implementation is consulted and no executable is produced.
    /// # Errors
    /// Unsupported barrier composition or the bounded witness construction exceeds its budget.
    pub fn retains_scientific_witness(
        &self,
        value: &TypedValue,
        witness: &ScientificWitness,
    ) -> Result<bool, MathError> {
        receipts::require_admission()?;
        let mut bindings = BTreeMap::new();
        witness_bindings(&self.stages, &mut bindings, self.limits.occurrences, 0)?;
        let atom = witness_resolve(&value.atom, &bindings, self.limits.occurrences)?;
        witness_expansion_budget(atom.as_view(), self.limits.occurrences, 0)?;
        let atom = atom.expand().together().expand();
        witness_bound(&atom, self.limits.occurrences)?;
        Ok(witness
            .symbols
            .iter()
            .any(|symbol| atom.contains_symbol(*symbol)))
    }
}

// Conservative term-count bounds for the library's rational expansion. This only
// refuses oversized construction; Symbolica alone computes the normalized formula.
fn witness_expansion_budget(
    atom: symbolica::atom::AtomView<'_>,
    limit: usize,
    depth: usize,
) -> Result<(usize, usize), MathError> {
    use symbolica::{atom::AtomView, coefficient::CoefficientView};
    if depth > 128 {
        return Err(MathError::Limit("scientific witness expansion depth"));
    }
    let bound = match atom {
        AtomView::Num(_) | AtomView::Var(_) => (1, 1),
        AtomView::Fun(function) => {
            for argument in function.iter() {
                witness_expansion_budget(argument, limit, depth + 1)?;
            }
            (1, 1)
        }
        AtomView::Add(sum) => sum.iter().try_fold((0usize, 1usize), |(p, q), term| {
            let (a, b) = witness_expansion_budget(term, limit, depth + 1)?;
            bounded_terms(
                (
                    p.saturating_mul(b).saturating_add(a.saturating_mul(q)),
                    q.saturating_mul(b),
                ),
                limit,
            )
        })?,
        AtomView::Mul(product) => product
            .iter()
            .try_fold((1usize, 1usize), |(p, q), factor| {
                let (a, b) = witness_expansion_budget(factor, limit, depth + 1)?;
                bounded_terms((p.saturating_mul(a), q.saturating_mul(b)), limit)
            })?,
        AtomView::Pow(power) => {
            let (p, q) = witness_expansion_budget(power.get_base(), limit, depth + 1)?;
            match power.get_exp() {
                AtomView::Num(number) => match number.get_coeff_view() {
                    CoefficientView::Natural(exponent, 1, 0, 1) => {
                        let exponent_abs = u32::try_from(exponent.unsigned_abs())
                            .map_err(|_| MathError::Limit("scientific witness power"))?;
                        let terms = (
                            p.saturating_pow(exponent_abs),
                            q.saturating_pow(exponent_abs),
                        );
                        if exponent < 0 {
                            (terms.1, terms.0)
                        } else {
                            terms
                        }
                    }
                    _ => (p, q),
                },
                exponent => {
                    witness_expansion_budget(exponent, limit, depth + 1)?;
                    (p, q)
                }
            }
        }
    };
    bounded_terms(bound, limit)
}

fn bounded_terms(terms: (usize, usize), limit: usize) -> Result<(usize, usize), MathError> {
    if terms.0 > limit || terms.1 > limit {
        return Err(MathError::Limit("scientific witness rational expansion"));
    }
    Ok(terms)
}

fn witness_bound(atom: &Atom, limit: usize) -> Result<(), MathError> {
    let count = atom.count_operations();
    if count
        .additions
        .saturating_add(count.multiplications)
        .saturating_add(count.inversions)
        .saturating_add(count.function_calls)
        > limit
    {
        return Err(MathError::Limit("normalized scientific witness"));
    }
    Ok(())
}

fn witness_resolve(
    atom: &Atom,
    bindings: &BTreeMap<usize, Atom>,
    limit: usize,
) -> Result<Atom, MathError> {
    let mut atom = atom.clone();
    for (slot, expression) in bindings.iter().rev() {
        let formal = library::formal(*slot)?;
        if atom.contains(formal.as_view()) {
            atom = atom.replace(formal).with(expression.clone());
            witness_bound(&atom, limit)?;
        }
    }
    Ok(atom)
}

fn witness_bindings(
    stages: &[Stage],
    bindings: &mut BTreeMap<usize, Atom>,
    limit: usize,
    depth: usize,
) -> Result<(), MathError> {
    if depth > 128 || stages.len() > limit {
        return Err(MathError::Limit("scientific witness bindings"));
    }
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                let values = expressions
                    .iter()
                    .map(|expression| witness_resolve(expression, bindings, limit))
                    .collect::<Result<Vec<_>, _>>()?;
                bindings.extend(outputs.iter().copied().zip(values));
            }
            Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
            Stage::Branch {
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                let mut yes = bindings.clone();
                let mut no = bindings.clone();
                witness_bindings(then, &mut yes, limit, depth + 1)?;
                witness_bindings(otherwise, &mut no, limit, depth + 1)?;
                let left = witness_resolve(&library::formal(*left)?, bindings, limit)?;
                let right = witness_resolve(&library::formal(*right)?, bindings, limit)?;
                for (slot, a) in &yes {
                    if bindings.get(slot) == Some(a) {
                        continue;
                    }
                    // Branch-local temporaries need not exist in the other branch;
                    // only jointly written result slots escape the branch boundary.
                    let Some(b) = no.get(slot) else {
                        continue;
                    };
                    let atom = if a == b {
                        a.clone()
                    } else {
                        library::function(
                            *slot,
                            &[left.clone(), right.clone(), a.clone(), b.clone()],
                        )?
                    };
                    bindings.insert(*slot, atom);
                }
            }
            Stage::Provider {
                inputs, outputs, ..
            } => {
                let arguments = inputs
                    .iter()
                    .map(|slot| witness_resolve(&library::formal(*slot)?, bindings, limit))
                    .collect::<Result<Vec<_>, _>>()?;
                for slot in outputs {
                    bindings.insert(*slot, library::function(*slot, &arguments)?);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};

    #[test]
    fn normalized_scientific_witness_rejects_zero_and_algebraic_cancellation() {
        let registry = standard_registry().unwrap();
        let source = SemanticId::from_bytes([73; 16]);
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let input = builder
            .input(0, ids::quantity("neutral"), IndexSet::new(), source)
            .unwrap();
        let mut witness = ScientificWitness::default();
        let call = builder
            .scientific_placeholder(
                &mut witness,
                source,
                input.quantity.clone(),
                std::slice::from_ref(&input),
                &[],
                source,
            )
            .unwrap();
        assert!(builder.retains_scientific_witness(&call, &witness).unwrap());
        let mut zero = call.clone();
        zero.atom *= Atom::num(0);
        assert!(!builder.retains_scientific_witness(&zero, &witness).unwrap());
        let bound = builder.independent(call.clone()).unwrap();
        let mut cancelled = call.clone();
        cancelled.atom = bound.atom - call.atom.clone();
        assert!(
            !builder
                .retains_scientific_witness(&cancelled, &witness)
                .unwrap()
        );
        let mut distributed = call.clone();
        distributed.atom =
            &call.atom * (&input.atom + Atom::num(1)) - &call.atom * &input.atom - &call.atom;
        assert!(
            !builder
                .retains_scientific_witness(&distributed, &witness)
                .unwrap()
        );
        let mut quotient = call.clone();
        quotient.atom = &call.atom / (&input.atom + Atom::num(1))
            + &call.atom * &input.atom / (&input.atom + Atom::num(1))
            - &call.atom;
        assert!(
            !builder
                .retains_scientific_witness(&quotient, &witness)
                .unwrap()
        );
    }

    #[test]
    fn normalized_scientific_witness_keeps_partials_abstract_and_resolves_repeated_arguments() {
        let registry = standard_registry().unwrap();
        let source = SemanticId::from_bytes([74; 16]);
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let input = builder
            .input(0, ids::quantity("neutral"), IndexSet::new(), source)
            .unwrap();
        let independent = builder.independent(input.clone()).unwrap();
        let mut witness = ScientificWitness::default();
        let first = builder
            .scientific_placeholder(
                &mut witness,
                source,
                input.quantity.clone(),
                std::slice::from_ref(&input),
                &[0],
                source,
            )
            .unwrap();
        let same = builder
            .scientific_placeholder(
                &mut witness,
                source,
                input.quantity.clone(),
                std::slice::from_ref(&independent),
                &[0],
                source,
            )
            .unwrap();
        assert!(
            builder
                .retains_scientific_witness(&first, &witness)
                .unwrap()
        );
        let mut cancelled = first.clone();
        cancelled.atom -= same.atom;
        assert!(
            !builder
                .retains_scientific_witness(&cancelled, &witness)
                .unwrap()
        );
        let second = builder
            .scientific_placeholder(
                &mut witness,
                source,
                input.quantity.clone(),
                std::slice::from_ref(&input),
                &[0, 0],
                source,
            )
            .unwrap();
        let mut distinct = first;
        distinct.atom -= second.atom;
        assert!(
            builder
                .retains_scientific_witness(&distinct, &witness)
                .unwrap()
        );
    }
}
