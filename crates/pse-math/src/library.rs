// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact public Symbolica integration. No project arithmetic instructions.
use crate::MathError;
use symbolica::domains::{float::Complex, rational::Rational};
use symbolica::{
    atom::{Atom, AtomCore, NamespacedSymbol, Symbol, SymbolBuilder},
    evaluate::{ExportedInstructions, ExpressionEvaluator, Instruction},
};
/// Formal and function symbols register in chunks of this many slots. Initialization
/// registers the first chunk; a body whose explicit slot allowance (`BodyLimits::slots`)
/// exceeds it extends the pool one chunk at a time when it first needs a slot there. The
/// pool has no ceiling of its own: each body's allowance bounds it. Measured on
/// 2026-09-29: the PC-SAFT tangent-plane body of three components needs 5510 slots, and a
/// chunk's 16 384 symbols register in 5-6 ms.
pub const FORMAL_CHUNK: usize = 8192;
/// Symbolica's Horner-scheme variable budget. It equals the former fixed pool so that
/// evaluator construction is unchanged; it is an optimizer budget, not a symbol bound.
const HORNER_SCHEME_VARIABLES: usize = 8192;
/// A reusable formal input or guarded-stage result. The first request for a slot beyond
/// the registered pool extends it; the same slot always yields the same symbol.
/// # Errors
/// A symbol has incompatible registration.
pub fn formal(slot: usize) -> Result<Atom, MathError> {
    crate::context()?.pool.formal(slot)
}
/// The number of registered formal slots, a whole number of [`FORMAL_CHUNK`]s.
/// # Errors
/// The symbolic runtime is not initialized.
pub fn formal_pool_len() -> Result<usize, MathError> {
    crate::context()?.pool.len()
}

/// Process-global formal and function symbols. Chunk `k` registers the formals of slots
/// `k·C..(k+1)·C`, then their functions, so Symbolica's registration order — its canonical
/// term order — of any two pool symbols is fixed, whenever and wherever a chunk registers.
/// Growth is serialized by the write lock; readers never observe a partial chunk.
#[derive(Debug)]
pub(crate) struct Pool(std::sync::RwLock<Symbols>);
#[derive(Debug, Default)]
struct Symbols {
    formals: Vec<Atom>,
    functions: Vec<Symbol>,
}
impl Pool {
    /// Register the first chunk before any model is admitted.
    pub(crate) fn new() -> Result<Self, String> {
        let mut symbols = Symbols::default();
        symbols.cover(FORMAL_CHUNK)?;
        Ok(Self(std::sync::RwLock::new(symbols)))
    }
    fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, Symbols>, MathError> {
        self.0
            .read()
            .map_err(|_| MathError::Library("formal symbol pool lock poisoned".into()))
    }
    /// Extend the pool to cover `slot` and read it under the same lock.
    fn extended<T>(
        &self,
        slot: usize,
        read: impl Fn(&Symbols) -> Option<T>,
    ) -> Result<T, MathError> {
        let mut symbols = self
            .0
            .write()
            .map_err(|_| MathError::Library("formal symbol pool lock poisoned".into()))?;
        let slots = slot
            .checked_add(1)
            .ok_or(MathError::Limit("formal symbol index"))?;
        symbols.cover(slots).map_err(MathError::Library)?;
        read(&symbols).ok_or(MathError::Limit("formal symbol index"))
    }
    fn len(&self) -> Result<usize, MathError> {
        Ok(self.read()?.formals.len())
    }
    fn formal(&self, slot: usize) -> Result<Atom, MathError> {
        if let Some(atom) = self.read()?.formals.get(slot) {
            return Ok(atom.clone());
        }
        self.extended(slot, |symbols| symbols.formals.get(slot).cloned())
    }
    pub(crate) fn function(&self, slot: usize) -> Result<Symbol, MathError> {
        if let Some(symbol) = self.read()?.functions.get(slot) {
            return Ok(*symbol);
        }
        self.extended(slot, |symbols| symbols.functions.get(slot).copied())
    }
}
impl Symbols {
    /// Register whole chunks until at least `slots` formals exist. A chunk is appended only
    /// once all of its symbols registered; Symbolica returns an already registered symbol
    /// unchanged, so a retried chunk keeps its registration order.
    fn cover(&mut self, slots: usize) -> Result<(), String> {
        let register = |name: String| {
            let name = NamespacedSymbol::try_from(name.as_str())?;
            SymbolBuilder::new(name)
                .build()
                .map_err(|error| error.to_string())
        };
        while self.formals.len() < slots {
            let start = self.formals.len();
            let end = start
                .checked_add(FORMAL_CHUNK)
                .ok_or("formal symbol pool extent")?;
            let formals = (start..end)
                .map(|slot| Ok(Atom::var(register(format!("pse_math::slot_{slot}"))?)))
                .collect::<Result<Vec<_>, String>>()?;
            let functions = (start..end)
                .map(|slot| register(format!("pse_math::function_{slot}")))
                .collect::<Result<Vec<_>, String>>()?;
            self.formals.extend(formals);
            self.functions.extend(functions);
        }
        Ok(())
    }
}

pub(crate) fn register_symbols() -> Result<crate::SymbolicContext, String> {
    // Symbolica registers built-ins as part of its global State initialization; the
    // first chunk follows in a fixed order, independent of model arrival order.
    Ok(crate::SymbolicContext {
        pool: Pool::new()?,
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
        .max_horner_scheme_variables(HORNER_SCHEME_VARIABLES)
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

/// Reusable function symbols for Symbolica-generated provider lifts, one per formal slot.
pub(crate) fn function(slot: usize, arguments: &[Atom]) -> Result<Atom, MathError> {
    use symbolica::atom::FunctionBuilder;
    let symbol = crate::context()?.pool.function(slot)?;
    Ok(FunctionBuilder::new(symbol)
        .add_args(arguments.iter())
        .finish())
}

/// What one immutable evaluator retains, measured from the library's exported
/// representation.
pub(crate) struct Storage {
    /// Owned scalar stack slots: inputs, constants and temporaries.
    pub(crate) numeric_entries: usize,
    /// Instruction stream, argument lists, output indices and sub-evaluators, in bytes. The
    /// exported instructions are at least as wide as the library's own, so this bounds them.
    pub(crate) instruction_bytes: usize,
}
/// The retained storage of `evaluator`: its numeric stack and its instruction stream.
pub(crate) fn storage(evaluator: &ExpressionEvaluator<f64>) -> Result<Storage, MathError> {
    let export = evaluator.export_instructions();
    Ok(Storage {
        numeric_entries: export
            .input_count
            .checked_add(export.constants.len())
            .and_then(|n| n.checked_add(export.temporary_count))
            .ok_or(MathError::Limit("evaluator stack extent"))?,
        instruction_bytes: instruction_bytes(&export)
            .ok_or(MathError::Limit("evaluator instruction extent"))?,
    })
}
fn instruction_bytes(export: &ExportedInstructions<f64>) -> Option<usize> {
    let listed = size_of_val(export.instructions.as_slice())
        .checked_add(export.output_count.checked_mul(size_of::<usize>())?)?;
    let arguments = export.instructions.iter().try_fold(listed, |bytes, i| {
        bytes.checked_add(match i {
            Instruction::Add(_, arguments, _) | Instruction::Mul(_, arguments, _) => {
                size_of_val(arguments.as_slice())
            }
            Instruction::Fun(_, call, _) => {
                let (_, tags, arguments) = call.as_ref();
                size_of_val(call.as_ref())
                    + size_of_val(tags.as_slice())
                    + tags.iter().map(String::len).sum::<usize>()
                    + size_of_val(arguments.as_slice())
            }
            _ => 0,
        })
    })?;
    export
        .sub_evaluators
        .iter()
        .try_fold(arguments, |bytes, sub| {
            let own = &sub.instructions;
            let numeric = own
                .input_count
                .checked_add(own.constants.len())?
                .checked_add(own.temporary_count)?
                .checked_mul(size_of::<f64>())?;
            bytes
                .checked_add(numeric)?
                .checked_add(instruction_bytes(own)?)
        })
}
