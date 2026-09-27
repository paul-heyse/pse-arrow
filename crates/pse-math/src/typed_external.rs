// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reify Symbolica's derivatives of opaque functions as supplied provider partials.
use super::*;
use std::collections::BTreeMap;
use symbolica::atom::Indeterminate;

fn bounded(atom: &Atom, limit: usize) -> Result<(), MathError> {
    let count = atom.count_operations();
    if count
        .additions
        .saturating_add(count.multiplications)
        .saturating_add(count.inversions)
        .saturating_add(count.function_calls)
        > limit
    {
        return Err(MathError::Limit("external partial symbolic extent"));
    }
    Ok(())
}
fn expand(mut atom: Atom, values: &BTreeMap<usize, Atom>, limit: usize) -> Result<Atom, MathError> {
    for (slot, value) in values.iter().rev() {
        if atom
            .as_view()
            .get_byte_size()
            .checked_mul(value.as_view().get_byte_size().saturating_add(1))
            .is_none_or(|n| n > 1 << 20)
        {
            return Err(MathError::Limit("external partial expansion"));
        }
        atom = atom.replace(library::formal(*slot)?).with(value.clone());
        bounded(&atom, limit)?;
    }
    Ok(atom)
}
impl BodyBuilder<'_> {
    pub(super) fn external_partial(
        &mut self,
        scope: FunctionScope,
        value: &Atom,
        arguments: &[TypedValue],
        source: SemanticId,
    ) -> Result<Atom, MathError> {
        let stages = self.stages[scope.0..].to_vec();
        let mut values = BTreeMap::new();
        let mut calls = Vec::new();
        for stage in &stages {
            match stage {
                Stage::Block {
                    expressions,
                    outputs,
                    ..
                } => {
                    for (e, output) in expressions.iter().zip(outputs) {
                        values.insert(
                            *output,
                            expand(e.clone(), &values, self.limits.occurrences)?,
                        );
                    }
                }
                Stage::Require { .. } | Stage::Domain { .. } => {}
                Stage::Branch { .. } => {
                    return Err(MathError::Contract(
                        "external partial across a branch requires a proved derivative path".into(),
                    ));
                }
                Stage::Provider {
                    spec,
                    partial,
                    inputs,
                    outputs,
                    ..
                } => {
                    let inputs_expanded = inputs
                        .iter()
                        .map(|i| expand(library::formal(*i)?, &values, self.limits.occurrences))
                        .collect::<Result<Vec<_>, _>>()?;
                    let bytes = inputs_expanded
                        .iter()
                        .try_fold(0usize, |n, e| n.checked_add(e.as_view().get_byte_size()));
                    if inputs
                        .len()
                        .checked_mul(outputs.len())
                        .is_none_or(|n| n > self.limits.occurrences)
                        || bytes
                            .and_then(|n| n.checked_mul(outputs.len()))
                            .is_none_or(|n| n > 1 << 20)
                    {
                        return Err(MathError::Limit("external partial call expansion"));
                    }
                    for (ordinal, output) in outputs.iter().enumerate() {
                        let composed = library::function(*output, &inputs_expanded)?;
                        values.insert(*output, composed.clone());
                        calls.push((
                            spec,
                            partial,
                            inputs,
                            outputs,
                            ordinal,
                            *output,
                            inputs_expanded.clone(),
                            composed,
                        ));
                    }
                }
            }
        }
        let mut atom = expand(value.clone(), &values, self.limits.occurrences)?;
        for arg in arguments {
            atom = atom
                .derivative(Indeterminate::try_from(arg.atom.clone()).map_err(MathError::Library)?);
            bounded(&atom, self.limits.occurrences)?;
        }
        // Generic local coordinates must not alias any expression already in this scope.
        let local_start = self.next_slot;
        let mut supplied = BTreeMap::new();
        for (spec, partial, inputs, outputs, ordinal, output, expanded, _) in &calls {
            if local_start
                .checked_add(inputs.len())
                .is_none_or(|n| n > self.limits.slots)
            {
                return Err(MathError::Limit("external partial local coordinates"));
            }
            let local = (local_start..local_start + inputs.len())
                .map(library::formal)
                .collect::<Result<Vec<_>, _>>()?;
            let generic = library::function(*output, &local)?;
            let remaining =
                (spec.derivatives.min(spec.smoothness) as usize).saturating_sub(partial.len());
            let mut selections = Vec::new();
            if remaining >= 1 {
                for i in 0..inputs.len() {
                    selections.push(vec![i]);
                }
            }
            if remaining >= 2 && arguments.len() >= 2 {
                if inputs
                    .len()
                    .checked_mul(inputs.len())
                    .is_none_or(|n| n > self.limits.occurrences)
                {
                    return Err(MathError::Limit("external mixed partial extent"));
                }
                for i in 0..inputs.len() {
                    for j in i..inputs.len() {
                        selections.push(vec![i, j]);
                    }
                }
            }
            // Replace higher partials before lower ones; ordinary values are rebound last.
            for selected in selections.into_iter().rev() {
                let mut pattern = generic.clone();
                for i in &selected {
                    pattern = pattern.derivative(
                        Indeterminate::try_from(local[*i].clone()).map_err(MathError::Library)?,
                    );
                }
                for (v, e) in local.iter().zip(expanded) {
                    pattern = pattern.replace(v.clone()).with(e.clone());
                }
                if !atom.contains(pattern.as_view()) {
                    continue;
                }
                self.tick()?;
                let mut full = partial.to_vec();
                full.extend(selected.iter().copied());
                let key = (spec.key(), inputs.to_vec(), full.clone());
                if !supplied.contains_key(&key) {
                    let destinations = outputs
                        .iter()
                        .map(|_| self.slot())
                        .collect::<Result<Vec<_>, _>>()?;
                    self.stages.push(Stage::Provider {
                        spec: (*spec).clone(),
                        partial: full,
                        inputs: inputs.to_vec(),
                        outputs: destinations.clone(),
                        source,
                    });
                    supplied.insert(key.clone(), destinations);
                }
                let slot = supplied[&key][*ordinal];
                let replaced = atom.replace(pattern).with(library::formal(slot)?);
                self.provider_order = self.provider_order.min(match remaining - selected.len() {
                    0 => DerivativeOrder::Value,
                    1 => DerivativeOrder::First,
                    _ => DerivativeOrder::Second,
                });
                atom = replaced;
            }
        }
        for (_, _, _, _, _, output, _, composed) in calls.iter().rev() {
            atom = atom
                .replace(composed.clone())
                .with(library::formal(*output)?);
        }
        if calls.iter().any(|(_, _, _, _, _, output, _, _)| {
            atom.contains_symbol(self.context.functions[*output])
        }) {
            return Err(MathError::Contract(
                "pure function partial exceeds supplied external derivative order".into(),
            ));
        }
        piecewise::strengthen_guards(&mut self.stages[scope.0..], arguments.len());
        Ok(atom)
    }
}
