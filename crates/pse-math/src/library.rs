// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact public Symbolica integration. No project arithmetic instructions.
use crate::MathError;
use symbolica::domains::{float::Complex, rational::Rational};
use symbolica::{
    atom::{Atom, AtomCore, NamespacedSymbol, SymbolBuilder},
    evaluate::ExpressionEvaluator,
};
/// Hard ceiling for process-global reusable formal symbols. Instances do not register symbols.
pub const MAX_FORMAL_SYMBOLS: usize = 4096;
/// A reusable formal input or guarded-stage result.
/// # Errors
/// Exceeds the process-wide symbol bound or a symbol has incompatible registration.
pub fn formal(slot: usize) -> Result<Atom, MathError> {
    crate::initialize()?;
    if slot >= MAX_FORMAL_SYMBOLS {
        return Err(MathError::Limit("formal symbols"));
    }
    let text = format!("pse_math::slot_{slot}");
    let name =
        NamespacedSymbol::try_from(text.as_str()).map_err(|e| MathError::Library(e.clone()))?;
    SymbolBuilder::new(name)
        .build()
        .map(Atom::var)
        .map_err(|e| MathError::Library(e.to_string()))
}
/// Explicit optimizer controls; no target-dependent or unbounded defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Optimization {
    /// Positive bounded optimizer concurrency.
    pub cores: usize,
    /// Horner search budget.
    pub horner_iterations: usize,
    /// Common-pair elimination rounds.
    pub cpe_iterations: usize,
}
impl Default for Optimization {
    fn default() -> Self {
        Self {
            cores: 1,
            horner_iterations: 10,
            cpe_iterations: 1,
        }
    }
}
/// Build one multi-output block. All control/domain barriers are outside this block.
/// # Errors
/// Invalid capacities or an unsupported library expression.
pub(crate) fn evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    options: Optimization,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<ExpressionEvaluator<f64>, MathError> {
    Ok(exact_evaluator(expressions, parameters, options, cancelled)?.map_coeff(&|c| c.re.to_f64()))
}

fn exact_evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    options: Optimization,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<ExpressionEvaluator<Complex<Rational>>, MathError> {
    crate::initialize()?;
    if options.cores == 0
        || options.cores > 64
        || options.horner_iterations > 1000
        || options.cpe_iterations > 32
    {
        return Err(MathError::Contract("optimizer budget".into()));
    }
    let abort = std::sync::Arc::clone(cancelled);
    let ev = Atom::evaluator_multiple(expressions, parameters)
        .abort_check(Some(Box::new(move || {
            abort.load(std::sync::atomic::Ordering::Relaxed)
        })))
        .cores(options.cores)
        .horner_iterations(options.horner_iterations)
        .cpe_iterations(Some(options.cpe_iterations))
        .max_common_pair_cache_entries(65536)
        .max_common_pair_distance(256)
        .max_horner_scheme_variables(MAX_FORMAL_SYMBOLS)
        .verbose(false)
        .build()
        .map_err(|e| {
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                MathError::Cancelled
            } else {
                MathError::Library(e.to_string())
            }
        })?;
    if !ev.is_real() {
        return Err(MathError::Contract(
            "complex arithmetic is outside this profile".into(),
        ));
    }
    Ok(ev)
}

/// Numerica generates the derivative arithmetic; no project AD rules are involved.
pub(crate) fn jet_evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    shape: Vec<Vec<usize>>,
    options: Optimization,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<ExpressionEvaluator<f64>, MathError> {
    use symbolica::prelude::{Dualizer, HyperDual};
    let evaluator = exact_evaluator(expressions, parameters, options, cancelled)?;
    let dual = HyperDual::<Complex<Rational>>::new(shape);
    let evaluator = evaluator
        .vectorize(&Dualizer::new(dual, vec![]))
        .map_err(MathError::Library)?;
    // Do not optimize_stack after vectorize: the pinned profile's unchecked slice aborts.
    Ok(evaluator.map_coeff(&|c| c.re.to_f64()))
}

/// Bounded reusable function symbols for Symbolica-generated provider lifts.
pub(crate) fn function(slot: usize, arguments: &[Atom]) -> Result<Atom, MathError> {
    use symbolica::atom::FunctionBuilder;
    if slot >= MAX_FORMAL_SYMBOLS {
        return Err(MathError::Limit("provider function symbols"));
    }
    let text = format!("pse_math::function_{slot}");
    let name =
        NamespacedSymbol::try_from(text.as_str()).map_err(|e| MathError::Library(e.clone()))?;
    let symbol = SymbolBuilder::new(name)
        .build()
        .map_err(|e| MathError::Library(e.to_string()))?;
    Ok(FunctionBuilder::new(symbol)
        .add_args(arguments.iter())
        .finish())
}
