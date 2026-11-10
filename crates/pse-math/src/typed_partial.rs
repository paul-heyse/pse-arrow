// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Symbolica partials over shared bindings. No arithmetic differentiation rules live here.
use super::*;
use std::collections::BTreeMap;
use symbolica::atom::{AtomView, Indeterminate, Symbol};

#[derive(Default)]
struct Bindings {
    calls: BTreeMap<Symbol, Atom>,
    materialized: BTreeMap<Symbol, (Atom, Atom)>,
}
impl Bindings {
    fn insert(&mut self, value: Atom, call: Atom, derivative: Atom) -> Result<(), MathError> {
        let formal = value
            .get_symbol()
            .ok_or_else(|| MathError::Contract("shared binding formal symbol".into()))?;
        let function = call
            .get_symbol()
            .ok_or_else(|| MathError::Contract("shared binding function symbol".into()))?;
        self.calls.insert(formal, call);
        self.materialized.insert(function, (value, derivative));
        Ok(())
    }

    fn derivative(&self, expression: &Atom, variable: &Indeterminate, argument: &Atom) -> Atom {
        if self.calls.is_empty() {
            return expression.derivative(variable.clone());
        }
        let lifted = expression.replace_map(|node, _, result| {
            if let AtomView::Var(formal) = node
                && let Some(call) = self.calls.get(&formal.get_symbol())
            {
                result.set_from_view(&call.as_view());
            }
        });
        lifted
            .derivative(variable.clone())
            .replace_map(|node, _, result| {
                let AtomView::Fun(call) = node else {
                    return;
                };
                let mut args = call.iter();
                // Symbolica's unary derivative is der(depth, function symbol, argument).
                // The function identity and arguments are stable across nesting; the
                // standalone atom's packed representation need not match a nested view.
                let (symbol, arg, derivative) = if call.get_symbol() == Symbol::DERIVATIVE {
                    let (Some(depth), Some(AtomView::Var(function)), Some(arg), None) =
                        (args.next(), args.next(), args.next(), args.next())
                    else {
                        return;
                    };
                    if !matches!(i64::try_from(depth), Ok(1)) {
                        return;
                    }
                    (function.get_symbol(), arg, true)
                } else {
                    let (Some(arg), None) = (args.next(), args.next()) else {
                        return;
                    };
                    (call.get_symbol(), arg, false)
                };
                let same_argument = match (arg, argument.as_view()) {
                    (AtomView::Var(left), AtomView::Var(right)) => {
                        left.get_symbol() == right.get_symbol()
                    }
                    (left, right) => left == right,
                };
                if same_argument && let Some((value, partial)) = self.materialized.get(&symbol) {
                    let replacement = if derivative { partial } else { value };
                    result.set_from_view(&replacement.as_view());
                }
            })
    }
}

impl BodyBuilder<'_> {
    /// Reify each earlier binding as a unary function of the current differentiation
    /// coordinate. Symbolica differentiates the expression and its function calls;
    /// their partials are rebound to earlier materialized outputs. Repeating this
    /// operation handles mixed and repeated partials without expanding the source DAG.
    pub(super) fn shared_partial(
        &mut self,
        scope: FunctionScope,
        value: &Atom,
        arguments: &[TypedValue],
        source: SemanticId,
    ) -> Result<Atom, MathError> {
        let mut value = value.clone();
        let mut work = 0usize;
        for argument in arguments {
            let variable =
                Indeterminate::try_from(argument.atom()?.clone()).map_err(MathError::Library)?;
            let stages = self.stages[scope.0..].to_vec();
            // Guard-only blocks are evaluated by the original source schedule,
            // but are not mathematical dependencies of this selected partial.
            // Determine the value DAG before asking Symbolica to differentiate;
            // otherwise an opaque predicate acquires an unrequested derivative.
            let mut needed = value.get_all_symbols(false);
            let mut selected = BTreeSet::new();
            for stage in stages.iter().rev() {
                if let Stage::Block {
                    expressions,
                    outputs,
                    ..
                } = stage
                {
                    for (expression, &slot) in expressions.iter().zip(outputs) {
                        let formal = library::formal(slot)?
                            .get_symbol()
                            .ok_or_else(|| MathError::Contract("shared partial formal".into()))?;
                        if needed.remove(&formal) {
                            selected.insert(slot);
                            needed.extend(expression.get_all_symbols(false));
                        }
                    }
                }
            }
            let mut bindings = Bindings::default();
            for stage in stages {
                match stage {
                    Stage::Block {
                        expressions,
                        outputs,
                        ..
                    } => {
                        // A block's outputs become visible together, after its inputs.
                        let mut next = Vec::new();
                        for (expression, slot) in expressions.iter().zip(outputs) {
                            if !selected.contains(&slot) {
                                continue;
                            }
                            let atom = bindings.derivative(expression, &variable, argument.atom()?);
                            let derivative = self.partial_binding(atom, source, &mut work)?;
                            let call =
                                library::function(slot, std::slice::from_ref(argument.atom()?))?;
                            next.push((library::formal(slot)?, call, derivative));
                        }
                        for (value, call, derivative) in next {
                            bindings.insert(value, call, derivative)?;
                        }
                    }
                    Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
                    _ => {
                        return Err(MathError::Contract(
                            "shared partial requires a smooth pure function body".into(),
                        ));
                    }
                }
            }
            let atom = bindings.derivative(&value, &variable, argument.atom()?);
            value = self.partial_binding(atom, source, &mut work)?;
        }
        Ok(value)
    }

    fn partial_binding(
        &mut self,
        atom: Atom,
        source: SemanticId,
        work: &mut usize,
    ) -> Result<Atom, MathError> {
        let count = atom.count_operations();
        *work = work
            .saturating_add(count.additions)
            .saturating_add(count.multiplications)
            .saturating_add(count.inversions)
            .saturating_add(count.function_calls);
        if *work > self.limits.occurrences {
            return Err(MathError::Limit("shared partial construction"));
        }
        if atom.is_zero() {
            return Ok(atom);
        }
        self.tick()?;
        library::formal(self.materialize(Some(atom), source)?)
    }
}
