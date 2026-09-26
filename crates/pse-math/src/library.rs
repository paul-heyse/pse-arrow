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
    crate::context()?
        .formals
        .get(slot)
        .cloned()
        .ok_or(MathError::Limit("formal symbols"))
}

pub(crate) fn register_symbols() -> Result<crate::SymbolicContext, String> {
    let register = |name: String| {
        let name = NamespacedSymbol::try_from(name.as_str())?;
        SymbolBuilder::new(name)
            .build()
            .map_err(|error| error.to_string())
    };
    let mut formals = Vec::with_capacity(MAX_FORMAL_SYMBOLS);
    let mut functions = Vec::with_capacity(MAX_FORMAL_SYMBOLS);
    // Symbolica registers built-ins as part of its global State initialization.
    // Complete each family in a fixed order, independent of model arrival order.
    for slot in 0..MAX_FORMAL_SYMBOLS {
        formals.push(Atom::var(register(format!("pse_math::slot_{slot}"))?));
    }
    for slot in 0..MAX_FORMAL_SYMBOLS {
        functions.push(register(format!("pse_math::function_{slot}"))?);
    }
    Ok(crate::SymbolicContext {
        formals,
        functions,
        environment: crate::linked_environment()?,
    })
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
    crate::context()?;
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

mod admission;
pub(crate) use admission::bounded_evaluator;

/// Bounded reusable function symbols for Symbolica-generated provider lifts.
pub(crate) fn function(slot: usize, arguments: &[Atom]) -> Result<Atom, MathError> {
    use symbolica::atom::FunctionBuilder;
    let symbol = *crate::context()?
        .functions
        .get(slot)
        .ok_or(MathError::Limit("provider function symbols"))?;
    Ok(FunctionBuilder::new(symbol)
        .add_args(arguments.iter())
        .finish())
}

/// Owned scalar stack slots exposed by the portable library representation.
/// Instruction storage and allocator overhead remain a separate foreign allowance.
pub(crate) fn numeric_entries(evaluator: &ExpressionEvaluator<f64>) -> Result<usize, MathError> {
    let export = evaluator.export_instructions();
    export
        .input_count
        .checked_add(export.constants.len())
        .and_then(|n| n.checked_add(export.temporary_count))
        .ok_or(MathError::Limit("evaluator stack extent"))
}
