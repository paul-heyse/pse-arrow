// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Construction bounds for the pinned Numerica Dualizer (Taylor orders zero to two).
//! This counts its convolution and primitive expansion, not a differentiation engine.
use super::*;
use crate::jets::{EvaluationLimits, JetLayout};
use std::sync::{Arc, atomic::AtomicBool};
use symbolica::atom::Symbol;
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
pub(crate) fn products(layout: &JetLayout) -> Result<usize, MathError> {
    let n = layout.coordinates.len();
    match layout.order {
        pse_kernels::DerivativeOrder::Value => Ok(1),
        pse_kernels::DerivativeOrder::First => add(1, mul(2, n)?),
        pse_kernels::DerivativeOrder::Second => add(add(1, mul(3, n)?)?, mul(2, mul(n, n)?)?),
    }
}

/// Scalar work envelope from Dualizer::map_instruction in Symbolica 3.0.0.
/// A multiply has W initial products and (K-W) product/add pairs. Each unary
/// Taylor loop has depth <=2 and one convolution, rescale and addition per step.
/// Setup/final scaling costs <=2W+2. Powf is log, multiplication, then exp.
fn expansion(c: OperationCount, layout: &JetLayout) -> Result<usize, MathError> {
    let w = layout.width();
    let convolution = products(layout)?
        .checked_mul(2)
        .and_then(|v| v.checked_sub(w))
        .ok_or(MathError::Limit("Taylor convolution"))?;
    let depth = layout.order as usize;
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

#[allow(
    clippy::too_many_arguments,
    reason = "Distinct compilation contexts and remaining body allowances"
)]
pub(crate) fn bounded_evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    layout: &JetLayout,
    options: Optimization,
    cancelled: &Arc<AtomicBool>,
    limits: EvaluationLimits,
    remaining: usize,
    occupied_entries: usize,
) -> Result<ExpressionEvaluator<f64>, MathError> {
    let source_work = expressions
        .iter()
        .try_fold(0, |sum, e| add(sum, total(e.count_operations())?))?;
    if source_work > remaining {
        return Err(MathError::Limit("scalar construction operations"));
    }
    let evaluator = exact_evaluator(expressions, parameters, options, cancelled)?;
    let scalar = evaluator.count_operations();
    if total(scalar)? > remaining {
        return Err(MathError::Limit("scalar evaluator operations"));
    }
    if layout.width() == 1 {
        return Ok(evaluator.map_coeff(&|c| c.re.to_f64()));
    }
    let exported = evaluator.export_instructions();
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
        return Err(MathError::Limit("derivative expansion"));
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
        .vectorize(&symbolica::prelude::Dualizer::new(dual, vec![]))
        .map_err(MathError::Library)?;
    if total(evaluator.count_operations())? > bound {
        return Err(MathError::Contract(
            "library Taylor expansion exceeded admitted bound".into(),
        ));
    }
    // Pinned vectorize's stack optimizer is unsupported; retain its own slot layout.
    Ok(evaluator.map_coeff(&|c| c.re.to_f64()))
}
