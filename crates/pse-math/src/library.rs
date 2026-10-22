// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact public Symbolica integration. No project arithmetic instructions.
use crate::MathError;
use std::sync::Arc;
use symbolica::domains::{float::Complex, rational::Rational};
use symbolica::{
    atom::{Atom, AtomCore, NamespacedSymbol, Symbol, SymbolAttribute, SymbolBuilder},
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
    /// once all of its symbols registered. Compatible existing symbols retain their
    /// identity on retry; incompatible attributes or custom metadata fail registration.
    fn cover(&mut self, slots: usize) -> Result<(), String> {
        let register = |name: String| {
            let namespaced = NamespacedSymbol::try_from(name.as_str())?;
            let symbol = SymbolBuilder::new(namespaced)
                // An explicit empty list selects Symbolica's checked registration path;
                // a bare builder accepts any existing same-name symbol unchanged.
                .with_attributes(Vec::<SymbolAttribute>::new())
                .build()
                .map_err(|error| error.to_string())?;
            // Symbolica treats omitted callbacks as unspecified when re-registering.
            // Pool symbols require none, including numerical evaluation callbacks.
            if !symbol.is_exportable() {
                return Err(format!("Symbol {name} redefined with custom callbacks"));
            }
            Ok(symbol)
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
/// Explicit evaluator implementation. Scalar C++ compilation is opt-in for eligible
/// demanded arithmetic products; provider calls and ordered guards remain host-owned.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Evaluator {
    /// Portable library instruction interpretation.
    #[default]
    Interpreted,
    /// Fixed conservative Linux x86-64 C++ profile. The finite allowance covers
    /// each module's export temporaries, named compiler outputs, loaded code,
    /// template metadata and bounded diagnostic reader; worker numeric scratch
    /// remains independently charged from the exported stack.
    CompiledScalar {
        /// Prepaid complete native preparation/module capacity for each arithmetic block.
        module_bytes: usize,
    },
}
impl Evaluator {
    /// Stable compilation choices, included in artifact identities. This is a
    /// supported profile identity, not environment/provenance attestation.
    pub fn identity(self) -> &'static str {
        match self {
            Self::Interpreted => "interpreted-f64-v1",
            Self::CompiledScalar { .. } => {
                "symbolica-3.0.1-cpp-scalar-value-v3;named-modules-8;g++;/usr/bin/g++;O2;c++17;pipe;two-phase;page-4096;noasm;no-fast-math;no-unsafe-math;fp-contract-off;x86-64-generic"
            }
        }
    }
    /// Per-module physical capacity carried by the compilation owner.
    pub fn module_bytes(self) -> usize {
        match self {
            Self::Interpreted => 0,
            Self::CompiledScalar { module_bytes } => module_bytes,
        }
    }
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
    /// Explicit implementation for scalar Value arithmetic blocks.
    pub evaluator: Evaluator,
}
impl Optimization {
    pub(crate) fn validate(self) -> Result<(), MathError> {
        if self.cores == 0 || self.cores > 64 || self.horner_iterations > 1000 || self.cpe_iterations > 32 {
            return Err(MathError::Contract("optimizer budget".into()));
        }
        Ok(())
    }
}
impl Default for Optimization {
    fn default() -> Self {
        Self {
            cores: 1,
            horner_iterations: 10,
            cpe_iterations: 1,
            evaluator: Evaluator::Interpreted,
        }
    }
}
// Symbolica 3.0.1 stores the builder's abort callback in its evaluator settings.
// Its public API has no settings setter. Detach the producer flag before retaining
// an exact evaluator; later serial vectorization reads only its constructor scope.
thread_local! {
    static CURRENT_ABORT: std::cell::RefCell<Option<Arc<std::sync::atomic::AtomicBool>>> = const { std::cell::RefCell::new(None) };
}
struct RestoreAbort(Option<Arc<std::sync::atomic::AtomicBool>>);
impl Drop for RestoreAbort {
    fn drop(&mut self) {
        CURRENT_ABORT.with(|current| {
            current.replace(self.0.take());
        });
    }
}
fn scoped_abort<T>(flag: &Arc<std::sync::atomic::AtomicBool>, work: impl FnOnce() -> T) -> T {
    let restore = RestoreAbort(CURRENT_ABORT.with(|current| current.replace(Some(flag.clone()))));
    let result = work();
    drop(restore);
    result
}
struct AbortBinding {
    producer: std::sync::Mutex<Option<Arc<std::sync::atomic::AtomicBool>>>,
    observed: std::sync::atomic::AtomicBool,
}
impl AbortBinding {
    fn new(producer: Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self { producer: std::sync::Mutex::new(Some(producer)), observed: std::sync::atomic::AtomicBool::new(false) }
    }
    fn finish<T>(&self, result: T, producer: &std::sync::atomic::AtomicBool) -> Result<T, MathError> {
        if self.observed.load(std::sync::atomic::Ordering::Acquire)
            || producer.load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(MathError::Cancelled);
        }
        Ok(result)
    }
    fn cancelled(&self) -> bool {
        let producer = self
            .producer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        producer.map_or_else(
            || {
                CURRENT_ABORT.with(|current| {
                    current
                        .borrow()
                        .as_ref()
                        .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Acquire))
                })
            },
            |flag| {
                let cancelled = flag.load(std::sync::atomic::Ordering::Acquire);
                if cancelled { self.observed.store(true, std::sync::atomic::Ordering::Release); }
                cancelled
            },
        )
    }
}
struct DetachAbort(Arc<AbortBinding>);
impl Drop for DetachAbort {
    fn drop(&mut self) {
        self.0
            .producer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}
fn exact_evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    definitions: &[(Atom, Atom)],
    options: Optimization,
    cancelled: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<ExpressionEvaluator<Complex<Rational>>, MathError> {
    if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err(MathError::Cancelled); }
    options.validate()?;
    crate::context()?;
    let abort = Arc::new(AbortBinding::new(cancelled.clone()));
    let detach = DetachAbort(abort.clone());
    let mut functions = symbolica::evaluate::FunctionMap::new();
    functions.add_aliases(definitions.iter().cloned()).map_err(|error| MathError::Library(error.to_string()))?;
    let ev = Atom::evaluator_multiple(expressions, parameters)
        .function_map(functions)
        // The direct translator resolves aliases as a shared instruction DAG;
        // the tree translator would recursively expand each output expression.
        .direct_translation(true)
        .abort_check(Some(Box::new(move || abort.cancelled())))
        .cores(options.cores)
        .horner_iterations(options.horner_iterations)
        .cpe_iterations(Some(options.cpe_iterations))
        .max_common_pair_cache_entries(65536)
        .max_common_pair_distance(256)
        .max_horner_scheme_variables(HORNER_SCHEME_VARIABLES)
        .verbose(false)
        .build()
        .map_err(|e| {
            if detach.0.observed.load(std::sync::atomic::Ordering::Acquire)
                || cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                MathError::Cancelled
            } else {
                MathError::Library(e.to_string())
            }
        })?;
    // Symbolica may return a successfully built, truncated optimizer result after
    // its abort callback fired. It must never reach export, ScalarCache or retention.
    let ev = detach.0.finish(ev, cancelled)?;
    drop(detach);
    if !ev.is_real() {
        return Err(MathError::Contract(
            "complex arithmetic is outside this profile".into(),
        ));
    }
    Ok(ev)
}

mod admission;
pub use admission::ScalarProducts;
pub(crate) use admission::{
    ScalarCache, bounded_evaluator, bounded_scalar_evaluator, bounded_defined_scalar_evaluator, evaluator_construction,
    evaluator_construction_shape,
};

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
#[derive(Clone, Copy)]
pub(crate) struct Storage {
    /// Owned scalar stack slots: inputs, constants and temporaries.
    pub(crate) numeric_entries: usize,
    /// Instruction stream, argument lists, output indices and sub-evaluators, in bytes. The
    /// exported instructions are at least as wide as the library's own, so this bounds them.
    pub(crate) instruction_bytes: usize,
    /// Exported instruction population, including exported sub-evaluator occurrences.
    pub(crate) instruction_count: usize,
}
/// Facts inspected once for one immutable library representation. A policy change
/// rechecks these premises; it never reuses an earlier admission decision.
#[derive(Clone, Copy)]
pub(crate) struct EvaluatorFacts {
    pub(crate) storage: Storage,
    pub(crate) operations: symbolica::evaluate::OperationCount,
    pub(crate) constants: usize,
    pub(crate) instructions: usize,
    pub(crate) taylor_supported: bool,
    pub(crate) resettable: bool,
}
impl EvaluatorFacts {
    pub(crate) fn inspect<T>(
        export: &ExportedInstructions<T>,
        operations: symbolica::evaluate::OperationCount,
        representation: &'static str,
    ) -> Result<Self, MathError> {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(representation, "count", 1);
        }
        let description = instruction_description(export)
            .ok_or(MathError::Limit("evaluator instruction extent"))?;
        Ok(Self {
            storage: Storage {
                numeric_entries: export
                    .input_count
                    .checked_add(export.constants.len())
                    .and_then(|n| n.checked_add(export.temporary_count))
                    .ok_or(MathError::Limit("evaluator stack extent"))?,
                instruction_bytes: description.bytes,
                instruction_count: description.count,
            },
            operations,
            constants: export.constants.len(),
            instructions: export.instructions.len(),
            taylor_supported: description.taylor_supported,
            resettable: description.resettable,
        })
    }
}
/// Final floating representation and the single export shared by all construction
/// consumers. Mutable workers receive only the evaluator, never this inspection copy.
pub(crate) struct FloatingEvaluator {
    pub(crate) evaluator: ExpressionEvaluator<f64>,
    pub(crate) facts: EvaluatorFacts,
    pub(crate) export: ExportedInstructions<f64>,
}
impl FloatingEvaluator {
    fn new(evaluator: ExpressionEvaluator<f64>) -> Result<Self, MathError> {
        let export = evaluator.export_instructions();
        let facts = EvaluatorFacts::inspect(
            &export,
            evaluator.count_operations(),
            "math.floating_representation_inspections",
        )?;
        Ok(Self {
            evaluator,
            facts,
            export,
        })
    }
}
struct InstructionDescription {
    bytes: usize,
    count: usize,
    taylor_supported: bool,
    resettable: bool,
}
/// One recursive census supplies storage and eligibility for this export.
fn instruction_description<T>(export: &ExportedInstructions<T>) -> Option<InstructionDescription> {
    let mut result = InstructionDescription {
        bytes: size_of_val(export.instructions.as_slice())
            .checked_add(export.output_count.checked_mul(size_of::<usize>())?)?,
        count: export.instructions.len(),
        taylor_supported: export.sub_evaluators.is_empty(),
        resettable: export.constant_functions.is_empty() && export.sub_evaluators.is_empty(),
    };
    for instruction in &export.instructions {
        let bytes = match instruction {
            Instruction::Add(_, arguments, _) | Instruction::Mul(_, arguments, _) => {
                size_of_val(arguments.as_slice())
            }
            Instruction::Fun(_, call, _) => {
                let (symbol, tags, arguments) = call.as_ref();
                result.resettable = false;
                result.taylor_supported &= [
                    Symbol::SQRT,
                    Symbol::EXP,
                    Symbol::LOG,
                    Symbol::SIN,
                    Symbol::COS,
                    Symbol::ABS,
                    Symbol::CONJ,
                ]
                .contains(symbol);
                size_of_val(call.as_ref())
                    + size_of_val(tags.as_slice())
                    + tags.iter().map(String::len).sum::<usize>()
                    + size_of_val(arguments.as_slice())
            }
            _ => 0,
        };
        result.bytes = result.bytes.checked_add(bytes)?;
    }
    for sub in &export.sub_evaluators {
        let own = &sub.instructions;
        let numeric = own
            .input_count
            .checked_add(own.constants.len())?
            .checked_add(own.temporary_count)?
            .checked_mul(size_of::<f64>())?;
        let nested = instruction_description(own)?;
        result.bytes = result
            .bytes
            .checked_add(numeric)?
            .checked_add(nested.bytes)?;
        result.count = result.count.checked_add(nested.count)?;
    }
    Some(result)
}

#[cfg(test)]
mod abort_binding_tests {
    use super::*;
    #[test]
    fn observed_constructor_abort_refuses_a_successful_truncated_result() {
        let producer = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let binding = AbortBinding::new(producer.clone());
        assert!(binding.cancelled());
        // Even a subsequently cleared caller flag cannot rehabilitate an optimizer
        // result whose construction already observed an abort.
        producer.store(false, std::sync::atomic::Ordering::Release);
        assert!(matches!(binding.finish("successful truncated result", &producer), Err(MathError::Cancelled)));
        let fresh = AbortBinding::new(producer.clone());
        assert_eq!(fresh.finish("complete result", &producer).unwrap(), "complete result");
    }
    #[test]
    fn constructor_abort_binding_is_fresh_nested_and_restored_after_unwind() {
        let producer = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let binding = Arc::new(AbortBinding::new(producer.clone()));
        let detach = DetachAbort(binding.clone());
        producer.store(true, std::sync::atomic::Ordering::Release);
        assert!(
            binding.cancelled(),
            "initial optimizer threads read the producer flag"
        );
        drop(detach);
        assert_eq!(Arc::strong_count(&producer), 1);
        assert!(!binding.cancelled());
        let current = Arc::new(std::sync::atomic::AtomicBool::new(true));
        scoped_abort(&current, || {
            assert!(binding.cancelled());
            let nested = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                scoped_abort(&nested, || {
                    assert!(!binding.cancelled());
                    panic!("test nested unwind");
                })
            }));
            assert!(binding.cancelled());
            assert_eq!(Arc::strong_count(&nested), 1);
        });
        assert!(!binding.cancelled());
        assert_eq!(Arc::strong_count(&current), 1);
    }
}
