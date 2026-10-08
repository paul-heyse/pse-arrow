// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Construction bounds for the pinned Numerica Dualizer (Taylor orders zero to two).
//! This counts its convolution and primitive expansion, not a differentiation engine.
use super::*;
use crate::jets::{EvaluationLimits, JetLayout};
use std::sync::{Arc, atomic::AtomicBool};
use symbolica::atom::{AtomView, Symbol};
use symbolica::evaluate::{Instruction, OperationCount};

fn add(a: usize, b: usize) -> Result<usize, MathError> {
    a.checked_add(b)
        .ok_or(MathError::Limit("derivative expansion"))
}
fn mul(a: usize, b: usize) -> Result<usize, MathError> {
    a.checked_mul(b)
        .ok_or(MathError::Limit("derivative expansion"))
}
fn total(c: OperationCount) -> Result<usize, MathError> {
    add(
        add(c.additions, c.multiplications)?,
        add(c.inversions, c.function_calls)?,
    )
}

/// Number of ordered Taylor convolution products, including each a_j * b_0.
/// Orders 0/1/2 have 1, 1+2n, and 1+3n+2n² products respectively.
fn products_for(n: usize, order: pse_kernels::DerivativeOrder) -> Result<usize, MathError> {
    match order {
        pse_kernels::DerivativeOrder::Value => Ok(1),
        pse_kernels::DerivativeOrder::First => add(1, mul(2, n)?),
        pse_kernels::DerivativeOrder::Second => add(add(1, mul(3, n)?)?, mul(2, mul(n, n)?)?),
    }
}

/// Scalar work envelope from Dualizer::map_instruction in Symbolica 3.0.1.
/// A multiply has W initial products and (K-W) product/add pairs. Each unary
/// Taylor loop has depth <=2 and one convolution, rescale and addition per step.
/// Setup/final scaling costs <=2W+2. Powf is log, multiplication, then exp.
fn expansion(c: OperationCount, layout: &JetLayout) -> Result<usize, MathError> {
    expansion_for(c, layout.coordinates.len(), layout.order, layout.width())
}
fn expansion_for(
    c: OperationCount,
    n: usize,
    order: pse_kernels::DerivativeOrder,
    w: usize,
) -> Result<usize, MathError> {
    let convolution = products_for(n, order)?
        .checked_mul(2)
        .and_then(|v| v.checked_sub(w))
        .ok_or(MathError::Limit("Taylor convolution"))?;
    let depth = order as usize;
    let unary = add(
        add(
            mul(depth, add(add(convolution, mul(2, w)?)?, 1)?)?,
            mul(2, w)?,
        )?,
        2,
    )?;
    let power = add(mul(2, unary)?, convolution)?;
    add(
        add(mul(c.additions, w)?, mul(c.multiplications, convolution)?)?,
        add(mul(c.inversions, unary)?, mul(c.function_calls, power)?)?,
    )
}

/// Initial source-issued reservation for known floating payload. Optimization can
/// increase its populations; the enclosing owner reserves that positive excess
/// before `map_coeff` allocates. Rational construction retains foreign admission.
#[derive(Clone, Copy)]
pub(crate) struct EvaluatorConstruction {
    pub(crate) entries: usize,
    pub(crate) instructions: usize,
}
fn source_operations(expressions: &[Atom]) -> Result<usize, MathError> {
    let mut operations = 0usize;
    let mut failed = false;
    for expression in expressions {
        expression.as_view().visitor(&mut |node| {
            let next = match node {
                AtomView::Add(a) => Some(a.get_nargs().saturating_sub(1)),
                AtomView::Mul(a) => Some(a.get_nargs().saturating_sub(1)),
                AtomView::Pow(p) => match isize::try_from(p.get_exp()) {
                    Ok(exponent) => exponent
                        .unsigned_abs()
                        .saturating_sub(1)
                        .checked_add(usize::from(exponent < 0)),
                    Err(_) => Some(1),
                },
                // Atom::count_operations in 3.0.1 omits Fun nodes. Count them here
                // because the exact evaluator and Dualizer do allocate for them.
                AtomView::Fun(_) => Some(1),
                _ => Some(0),
            };
            if let Some(next) = next.and_then(|next| operations.checked_add(next)) {
                operations = next;
            } else {
                failed = true;
            }
            !failed
        });
        if failed {
            return Err(MathError::Limit("source evaluator operations"));
        }
    }
    Ok(operations)
}
/// One source atom byte bounds one encoded node/constant/operand. Integer powers
/// additionally contribute their explicit arithmetic expansion. The maximum of
/// Dualizer's four operation costs allows Horner optimization to redistribute work;
/// positive actual export excess is reserved before any floating conversion.
pub(crate) fn evaluator_construction(
    expressions: &[Atom],
    parameters: usize,
    coordinates: usize,
    order: pse_kernels::DerivativeOrder,
) -> Result<EvaluatorConstruction, MathError> {
    let source = expressions
        .iter()
        .try_fold(0, |bytes, atom| add(bytes, atom.as_view().get_byte_size()))?;
    let operations = source_operations(expressions)?;
    let first = if order >= pse_kernels::DerivativeOrder::First {
        coordinates
    } else {
        0
    };
    let second = if order >= pse_kernels::DerivativeOrder::Second {
        mul(coordinates, add(coordinates, 1)?)? / 2
    } else {
        0
    };
    let width = add(add(1, first)?, second)?;
    let mut per_operation = 0;
    for count in [
        OperationCount::new(1, 0, 0, 0),
        OperationCount::new(0, 1, 0, 0),
        OperationCount::new(0, 0, 1, 0),
        OperationCount::new(0, 0, 0, 1),
    ] {
        per_operation = per_operation.max(expansion_for(count, coordinates, order, width)?);
    }
    let expanded = mul(operations, per_operation)?;
    let scalar_instructions = add(add(source, operations)?, expressions.len())?;
    let vectors = add(
        add(mul(2, parameters)?, expressions.len())?,
        add(
            source,
            add(mul(3, scalar_instructions)?, mul(10, operations)?)?,
        )?,
    )?;
    let entries = add(expanded, mul(width, vectors)?)?;
    // Instructions contain indices, not coefficient payloads. Numeric operation
    // results/assignment vectors bound instruction count and operand slots. Tags
    // and original literal payloads have the admitted source-byte population.
    let instructions = add(
        mul(
            entries,
            add(
                size_of::<Instruction>(),
                mul(4, size_of::<symbolica::evaluate::Slot>())?,
            )?,
        )?,
        source,
    )?;
    Ok(EvaluatorConstruction {
        entries,
        instructions,
    })
}
fn reserve_export<T>(
    export: &ExportedInstructions<T>,
    initial: EvaluatorConstruction,
) -> Result<(), MathError> {
    let entries = add(
        add(export.input_count, export.constants.len())?,
        export.temporary_count,
    )?;
    let instructions =
        instruction_bytes(export).ok_or(MathError::Limit("evaluator instruction extent"))?;
    // The initial body reservation charges numeric payload twice: the immutable
    // evaluator and its worker/construction copy. Grow that same population for
    // any optimizer excess, without imposing a new scientific operation cutoff.
    let extra = add(
        mul(
            entries.saturating_sub(initial.entries),
            2 * size_of::<f64>(),
        )?,
        instructions.saturating_sub(initial.instructions),
    )?;
    crate::construction::additional(extra)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Distinct compilation contexts and remaining body allowances"
)]
pub(crate) fn bounded_evaluator(
    source_id: pse_ids::SemanticId,
    expressions: &[Atom],
    parameters: &[Atom],
    layout: &JetLayout,
    zero_components: &[(usize, usize)],
    options: Optimization,
    cancelled: &Arc<AtomicBool>,
    limits: EvaluationLimits,
    remaining: usize,
    occupied_entries: usize,
) -> Result<ExpressionEvaluator<f64>, MathError> {
    if zero_components.iter().any(|&(parameter, component)| {
        parameter >= parameters.len() || component == 0 || component >= layout.width()
    }) {
        return Err(MathError::Contract(
            "invalid structural Taylor zero component".into(),
        ));
    }
    let construction = evaluator_construction(
        expressions,
        parameters.len(),
        layout.coordinates.len(),
        layout.order,
    )?;
    let source_work = expressions.iter().try_fold(0, |sum, expression| {
        add(sum, total(expression.count_operations())?)
    })?;
    if source_work > remaining {
        return Err(MathError::Limit("scalar construction operations"));
    }
    let evaluator = exact_evaluator(expressions, parameters, options, cancelled)?;
    let scalar = evaluator.count_operations();
    if total(scalar)? > remaining {
        return Err(MathError::Limit("scalar evaluator operations"));
    }
    let exported = evaluator.export_instructions();
    limits.allocation(add(
        occupied_entries,
        add(
            add(exported.input_count, exported.constants.len())?,
            exported.temporary_count,
        )?,
    )?)?;
    if layout.width() == 1 {
        reserve_export(&exported, construction)?;
        return Ok(evaluator.map_coeff(&|c| c.re.to_f64()));
    }
    if !exported.sub_evaluators.is_empty()
        || exported.instructions.iter().any(|i| match i {
            Instruction::Fun(_, f, _) => ![
                Symbol::SQRT,
                Symbol::EXP,
                Symbol::LOG,
                Symbol::SIN,
                Symbol::COS,
                Symbol::ABS,
                Symbol::CONJ,
            ]
            .contains(&f.0),
            _ => false,
        })
    {
        return Err(MathError::Contract("unsupported Taylor primitive".into()));
    }
    let bound = expansion(scalar, layout)?;
    if bound > remaining {
        return Err(MathError::WorkLimit {
            source_id,
            resource: "derivative expansion",
            required: bound,
            available: remaining,
            components: layout.width(),
        });
    }
    // Numeric slots: original parameters/constants, external input/output buffers,
    // one slot per arithmetic operation, <=3 assignment vectors per instruction
    // (Powf's log result, adjacent multiplication, exp result), and <=7 new constant
    // vectors per function (log+exp); inversions introduce <=3 constant vectors.
    let vectors = add(
        add(mul(2, parameters.len())?, expressions.len())?,
        add(
            exported.constants.len(),
            add(
                mul(3, exported.instructions.len())?,
                add(mul(7, scalar.function_calls)?, mul(3, scalar.inversions)?)?,
            )?,
        )?,
    )?;
    limits.allocation(add(
        occupied_entries,
        add(bound, mul(layout.width(), vectors)?)?,
    )?)?;
    let dual = symbolica::prelude::HyperDual::<Complex<Rational>>::new(layout.shape.clone());
    let evaluator = evaluator
        .vectorize(&symbolica::prelude::Dualizer::new(
            dual,
            zero_components.to_vec(),
        ))
        .map_err(MathError::Library)?;
    if total(evaluator.count_operations())? > bound {
        return Err(MathError::Contract(
            "library Taylor expansion exceeded admitted bound".into(),
        ));
    }
    // Check the actual expanded payload before floating constants/stack are allocated.
    let expanded_export = evaluator.export_instructions();
    reserve_export(&expanded_export, construction)?;
    limits.allocation(add(
        occupied_entries,
        add(
            add(expanded_export.input_count, expanded_export.constants.len())?,
            expanded_export.temporary_count,
        )?,
    )?)?;
    // Pinned vectorize already optimizes its stack and common pairs.
    Ok(evaluator.map_coeff(&|c| c.re.to_f64()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_construction_counts_builtin_functions_and_checks_taylor_payload() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let expression = symbolica::atom::FunctionBuilder::new(Symbol::EXP)
            .add_arg(&x)
            .finish();
        assert_eq!(
            source_operations(std::slice::from_ref(&expression)).unwrap(),
            1
        );
        let value = evaluator_construction(
            std::slice::from_ref(&expression),
            1,
            1,
            pse_kernels::DerivativeOrder::Value,
        )
        .unwrap();
        let second = evaluator_construction(
            std::slice::from_ref(&expression),
            1,
            1,
            pse_kernels::DerivativeOrder::Second,
        )
        .unwrap();
        assert!(second.entries > value.entries);
        let layout = JetLayout::new(
            vec![0],
            pse_kernels::DerivativeOrder::Second,
            EvaluationLimits::default(),
        )
        .unwrap();
        let compiled = bounded_evaluator(
            pse_ids::SemanticId::NIL,
            &[expression],
            &[x],
            &layout,
            &[],
            Optimization::default(),
            &Arc::new(AtomicBool::new(false)),
            EvaluationLimits::default(),
            1_000_000,
            0,
        )
        .unwrap();
        reserve_export(&compiled.export_instructions(), second).unwrap();
    }
    #[derive(Debug)]
    struct Grow(std::sync::atomic::AtomicUsize);
    impl crate::construction::ConstructionAdmission for Grow {
        fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
            self.0.fetch_add(bytes, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }
    #[derive(Debug)]
    struct Refuse;
    impl crate::construction::ConstructionAdmission for Refuse {
        fn try_grow(&self, _: usize) -> Result<(), MathError> {
            Err(MathError::Limit("test pool full"))
        }
    }
    #[test]
    fn floating_conversion_reserves_excess_before_allocation_and_propagates_refusal() {
        let initial = EvaluatorConstruction {
            entries: 4,
            instructions: 1024,
        };
        let export = ExportedInstructions::<f64> {
            input_count: 1,
            output_count: 1,
            instructions: vec![],
            temporary_count: 4,
            constants: vec![],
            constant_functions: vec![],
            sub_evaluators: vec![],
        };
        let owner = Arc::new(Grow(std::sync::atomic::AtomicUsize::new(0)));
        crate::construction::scoped(owner.clone(), || reserve_export(&export, initial)).unwrap();
        assert_eq!(
            owner.0.load(std::sync::atomic::Ordering::SeqCst),
            2 * size_of::<f64>()
        );
        assert!(matches!(
            crate::construction::scoped(Arc::new(Refuse), || reserve_export(&export, initial)),
            Err(MathError::Limit("test pool full"))
        ));
    }
    #[test]
    fn horner_growth_preserves_original_scientific_limits_and_eligibility() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let y = formal(1).unwrap();
        let z = formal(2).unwrap();
        let expression = (&x + &y) * (&x + &z);
        let parameters = [x, y, z];
        let options = Optimization {
            horner_iterations: 1,
            ..Default::default()
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let exact = exact_evaluator(
            std::slice::from_ref(&expression),
            &parameters,
            options,
            &cancel,
        )
        .unwrap();
        let export = exact.export_instructions();
        let entries = export.input_count + export.constants.len() + export.temporary_count;
        // Horner's multiplication-first search does not promise a monotonic total.
        // Admit this actual optimized payload against a smaller initial reservation
        // and verify growth succeeds instead of treating excess as a contract error.
        let initial = EvaluatorConstruction {
            entries: entries - 1,
            instructions: instruction_bytes(&export).unwrap(),
        };
        let owner = Arc::new(Grow(std::sync::atomic::AtomicUsize::new(0)));
        crate::construction::scoped(owner.clone(), || reserve_export(&export, initial)).unwrap();
        assert_eq!(
            owner.0.load(std::sync::atomic::Ordering::SeqCst),
            2 * size_of::<f64>()
        );
        let limits = EvaluationLimits::default();
        let layout = JetLayout::new(vec![], pse_kernels::DerivativeOrder::Value, limits).unwrap();
        let compiled = bounded_evaluator(
            pse_ids::SemanticId::NIL,
            &[expression],
            &parameters,
            &layout,
            &[],
            options,
            &cancel,
            limits,
            limits.operations,
            0,
        )
        .unwrap();
        let mut compiled = compiled;
        let mut output = [0.0];
        compiled.evaluate(&[2.0, 3.0, 4.0], &mut output);
        assert_eq!(output, [30.0]);
    }
}
