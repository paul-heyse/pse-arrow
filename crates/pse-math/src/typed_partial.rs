// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Symbolica partials over shared bindings. No arithmetic differentiation rules live here.
use super::*;
use symbolica::atom::Indeterminate;

struct Binding {
    value: Atom,
    call: Atom,
    partial: Atom,
    derivative: Atom,
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
                Indeterminate::try_from(argument.atom.clone()).map_err(MathError::Library)?;
            let stages = self.stages[scope.0..].to_vec();
            let mut bindings = Vec::new();
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
                            let atom = shared_derivative(expression, &bindings, &variable);
                            let derivative = self.partial_binding(atom, source, &mut work)?;
                            let call =
                                library::function(slot, std::slice::from_ref(&argument.atom))?;
                            let partial = call.derivative(variable.clone());
                            next.push(Binding {
                                value: library::formal(slot)?,
                                call,
                                partial,
                                derivative,
                            });
                        }
                        bindings.extend(next);
                    }
                    Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
                    _ => {
                        return Err(MathError::Contract(
                            "shared partial requires a smooth pure function body".into(),
                        ));
                    }
                }
            }
            let atom = shared_derivative(&value, &bindings, &variable);
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
        library::formal(self.materialize(atom, source)?)
    }
}

fn shared_derivative(expression: &Atom, bindings: &[Binding], variable: &Indeterminate) -> Atom {
    let mut atom = expression.clone();
    for binding in bindings {
        if atom.contains(binding.value.as_view()) {
            atom = atom
                .replace(binding.value.clone())
                .with(binding.call.clone());
        }
    }
    atom = atom.derivative(variable.clone());
    for binding in bindings {
        atom = atom
            .replace(binding.partial.clone())
            .with(binding.derivative.clone());
    }
    for binding in bindings {
        atom = atom
            .replace(binding.call.clone())
            .with(binding.value.clone());
    }
    atom
}
