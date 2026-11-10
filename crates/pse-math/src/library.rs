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
    #[cfg(test)]
    pub(crate) fn with_existing_formals<T>(
        &self,
        slots: usize,
        use_formals: impl FnOnce(&[Atom]) -> Result<T, MathError>,
    ) -> Result<T, MathError> {
        let symbols = self.read()?;
        let formals = symbols.formals.get(..slots).ok_or_else(|| {
            MathError::Contract("formal pool has not admitted the sealed slots".into())
        })?;
        use_formals(formals)
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
    crate::stable::initialize()?;
    Ok(crate::SymbolicContext {
        pool: Pool::new()?,
        environment: crate::linked_environment()?,
    })
}
pse_contract::projection_enum! {
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
    projection: { name: "MathEvaluator", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Portable library instruction interpretation.
    #[default]
    Interpreted => 1;
    /// Fixed conservative Linux x86-64 C++ profile. The finite allowance covers
    /// each module's export temporaries, named compiler outputs, loaded code,
    /// template metadata and bounded diagnostic reader; worker numeric scratch
    /// remains independently charged from the exported stack.
    CompiledScalar {
        /// Prepaid complete native preparation/module capacity for each arithmetic block.
        module_bytes: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Explicit branch-free AVX2 x4 profile with an independently callable
    /// compatible scalar tail in the same owned native module.
    CompiledX4 {
        /// Complete native module, pinned headers and preparation allowance.
        module_bytes: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
}
}
impl Evaluator {
    pub(crate) fn numeric_stack_factor(self) -> usize {
        if matches!(self, Self::CompiledX4 { .. }) {
            5
        } else {
            1
        }
    }
    /// Stable compilation choices, included in artifact identities. This is a
    /// supported profile identity, not environment/provenance attestation.
    pub fn identity(self) -> &'static str {
        match self {
            Self::Interpreted => "interpreted-f64-v1",
            Self::CompiledScalar { .. } => {
                "symbolica-3.0.1-cpp-scalar-value-v3;named-modules-32;g++;/usr/bin/g++;O2;c++17;pipe;two-phase;page-4096;noasm;no-fast-math;no-unsafe-math;fp-contract-off;x86-64-generic"
            }
            Self::CompiledX4 { .. } => {
                "symbolica-3.0.1-cpp-x4-f64-v1;authenticated-pinned-xsimd;avx2;no-fma;scalar-tail;named-modules-32;g++;/usr/bin/g++;O2;c++17;noasm;no-fast-math;no-unsafe-math;fp-contract-off"
            }
        }
    }
    /// Per-module physical capacity carried by the compilation owner.
    pub fn module_bytes(self) -> usize {
        match self {
            Self::Interpreted => 0,
            Self::CompiledScalar { module_bytes } | Self::CompiledX4 { module_bytes } => {
                module_bytes
            }
        }
    }
}
const MAX_OPTIMIZER_CORES: usize = 64;
const MAX_HORNER_ITERATIONS: usize = 1000;
const MAX_CPE_ITERATIONS: usize = 32;
pse_contract::projection_struct! {
/// Explicit optimizer controls; no target-dependent or unbounded defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Optimization {
    projection: { name: "MathOptimization", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Positive bounded optimizer concurrency.
    pub cores: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: MAX_OPTIMIZER_CORES as u64 }] };
    /// Horner search budget.
    pub horner_iterations: usize => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 0, max: MAX_HORNER_ITERATIONS as u64 }] };
    /// Common-pair elimination rounds.
    pub cpe_iterations: usize => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 0, max: MAX_CPE_ITERATIONS as u64 }] };
    /// Explicit implementation for scalar Value arithmetic blocks.
    pub evaluator: Evaluator => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl Optimization {
    pub(crate) fn validate(self) -> Result<(), MathError> {
        if self.cores == 0
            || self.cores > MAX_OPTIMIZER_CORES
            || self.horner_iterations > MAX_HORNER_ITERATIONS
            || self.cpe_iterations > MAX_CPE_ITERATIONS
        {
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
/// Borrow the actual stop contract of one constructor. Runtime/compiled-body
/// callers carry their full scope; the public compiler accelerator boundary has
/// only its original flag and deliberately makes no deadline claim.
#[derive(Clone, Copy)]
pub(crate) enum ConstructionStop<'a> {
    Scope(&'a pse_kernels::ExecutionScope),
    Flag(&'a Arc<std::sync::atomic::AtomicBool>),
}
impl<'a> From<&'a pse_kernels::ExecutionScope> for ConstructionStop<'a> {
    fn from(scope: &'a pse_kernels::ExecutionScope) -> Self {
        Self::Scope(scope)
    }
}
impl<'a> From<&'a Arc<std::sync::atomic::AtomicBool>> for ConstructionStop<'a> {
    fn from(flag: &'a Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self::Flag(flag)
    }
}
impl<'a> From<&'a pse_diagnostics::Cancellation> for ConstructionStop<'a> {
    fn from(control: &'a pse_diagnostics::Cancellation) -> Self {
        Self::Flag(control)
    }
}
impl<'a> ConstructionStop<'a> {
    pub(crate) fn check(self) -> Result<(), MathError> {
        match self {
            Self::Scope(scope) => scope.check().map_err(crate::error::scope_error),
            Self::Flag(flag) if flag.load(std::sync::atomic::Ordering::Acquire) => {
                Err(MathError::Cancelled)
            }
            Self::Flag(_) => Ok(()),
        }
    }
}
/// One borrowed supplier operation. Capture the original allocation owner before
/// supplier workers run; their thread-local construction owner is not authority.
struct SupplierControl<'a> {
    stop: ConstructionStop<'a>,
    owner: Option<Arc<dyn crate::construction::ConstructionAdmission>>,
    limits: crate::jets::EvaluationLimits,
    occupied_entries: usize,
    peak: usize,
    output_bytes: usize,
}
impl<'a> SupplierControl<'a> {
    fn new(
        stop: ConstructionStop<'a>,
        limits: crate::jets::EvaluationLimits,
        occupied_entries: usize,
    ) -> Self {
        Self {
            stop,
            owner: crate::construction::current(),
            limits,
            occupied_entries,
            peak: 0,
            output_bytes: 0,
        }
    }
    fn check_extent(
        &mut self,
        extent: symbolica::evaluate::OperationExtent,
    ) -> Result<(), MathError> {
        self.stop.check()?;
        // Constructor envelopes describe allocation, not an observed final
        // mathematical slot population. Export/finalization report actual slots.
        if matches!(
            extent.stage,
            symbolica::evaluate::OperationStage::Export
                | symbolica::evaluate::OperationStage::Finalization
        ) {
            self.limits.allocation(
                self.occupied_entries
                    .checked_add(extent.output_entries)
                    .ok_or(MathError::Limit("supplier numeric entries"))?,
            )?;
        }
        let required = extent
            .peak_bytes()
            .ok_or(MathError::Limit("supplier construction extent"))?;
        if required > self.peak {
            if let Some(owner) = &self.owner {
                owner.try_grow(required - self.peak)?;
            }
            self.peak = required;
        }
        if extent.stage == symbolica::evaluate::OperationStage::Finalization {
            self.output_bytes = extent.output_bytes;
        }
        Ok(())
    }
}
impl symbolica::evaluate::OperationControl for SupplierControl<'_> {
    type Error = MathError;
    fn check(&mut self, extent: symbolica::evaluate::OperationExtent) -> Result<(), MathError> {
        self.check_extent(extent)
    }
}
fn supplier_error(error: symbolica::evaluate::OperationError<MathError>) -> MathError {
    use symbolica::evaluate::OperationError;
    match error {
        OperationError::Control(error) => error,
        OperationError::Overflow => MathError::Limit("supplier construction extent"),
        OperationError::Allocation => MathError::Limit("supplier construction allocation"),
        OperationError::Unsupported(profile) => {
            MathError::Contract(format!("unsupported checked evaluator profile: {profile}"))
        }
        OperationError::InvalidProgram(message) => MathError::Library(message),
        OperationError::Interrupted => MathError::Cancelled,
    }
}
fn exact_evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    definitions: &[(Atom, Atom)],
    options: Optimization,
    control: &mut SupplierControl<'_>,
) -> Result<ExpressionEvaluator<Complex<Rational>>, MathError> {
    control.stop.check()?;
    options.validate()?;
    crate::context()?;
    let heads = crate::stable::known_heads()?;
    let evaluator = symbolica::evaluate::EvaluatorBuilder::try_new_aliases_with(
        expressions, parameters, definitions, control,
    ).map_err(supplier_error)?
        // Keep the shared alias DAG and the original selected optimization policy.
        .direct_translation(true)
        .cores(options.cores)
        .horner_iterations(options.horner_iterations)
        .cpe_iterations(Some(options.cpe_iterations))
        .max_common_pair_cache_entries(65536)
        .max_common_pair_distance(256)
        .max_horner_scheme_variables(HORNER_SCHEME_VARIABLES)
        .verbose(false)
        .try_build_known_with(heads,control)
        .map_err(supplier_error)?;
    control.stop.check()?;
    if !evaluator.is_real() {
        return Err(MathError::Contract(
            "complex arithmetic is outside this profile".into(),
        ));
    }
    Ok(evaluator)
}

mod admission;
pub use admission::ScalarProducts;
pub(crate) use admission::{
    ScalarCache, bounded_defined_scalar_evaluator, bounded_evaluator, bounded_scalar_evaluator,
};

/// Reusable function symbols for Symbolica-generated provider lifts, one per formal slot.
pub(crate) fn function(slot: usize, arguments: &[Atom]) -> Result<Atom, MathError> {
    use symbolica::atom::FunctionBuilder;
    let symbol = crate::context()?.pool.function(slot)?;
    Ok(FunctionBuilder::new(symbol)
        .add_args(arguments.iter())
        .finish())
}

/// Private evaluator backing reported by the supplier, with its separately
/// consumed public instruction description as a lower bound.
#[derive(Clone, Copy)]
pub(crate) struct Storage {
    /// Owned scalar stack slots: inputs, constants and temporaries.
    pub(crate) numeric_entries: usize,
    /// Backing beyond the scientific scalar slots: instruction operands, call
    /// metadata, exact fixed arguments, names and native capacity slack. The
    /// public description also bounds the separately consumed instruction view.
    pub(crate) auxiliary_bytes: usize,
    /// Exported instruction population, including exported sub-evaluator occurrences.
    pub(crate) instruction_count: usize,
}
/// Facts inspected once for one immutable library representation. A policy change
/// rechecks these premises; it never reuses an earlier admission decision.
#[derive(Clone, Copy)]
pub(crate) struct EvaluatorFacts {
    pub(crate) storage: Storage,
    pub(crate) operations: symbolica::evaluate::OperationCount,
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
                auxiliary_bytes: description.bytes,
                instruction_count: description.count,
            },
            operations,
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
    fn from_exact(
        exact: &ExpressionEvaluator<Complex<Rational>>,
        stop: ConstructionStop<'_>,
        limits: crate::jets::EvaluationLimits,
        occupied_entries: usize,
    ) -> Result<Self, MathError> {
        let mut control = SupplierControl::new(stop, limits, occupied_entries);
        let evaluator = exact
            .try_map_f64_known_with(crate::stable::transfer_heads()?, &mut control)
            .map_err(supplier_error)?;
        // Save the actual private heap before public export reports its distinct
        // returned representation. No exact payload becomes scientific f64 slots.
        let retained_bytes = control.output_bytes;
        let export = evaluator
            .try_export_instructions_with(&mut control)
            .map_err(supplier_error)?;
        let mut facts = EvaluatorFacts::inspect(
            &export,
            evaluator.count_operations(),
            "math.floating_representation_inspections",
        )?;
        let numeric_bytes = facts
            .storage
            .numeric_entries
            .checked_mul(size_of::<f64>())
            .ok_or(MathError::Limit("evaluator scalar stack extent"))?;
        let auxiliary = retained_bytes.checked_sub(numeric_bytes).ok_or_else(|| {
            MathError::Contract("native evaluator heap is smaller than its scalar stack".into())
        })?;
        facts.storage.auxiliary_bytes = facts.storage.auxiliary_bytes.max(auxiliary);
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
fn pure_scalar_head(symbol: Symbol, arity: usize) -> bool {
    crate::stable::identify(symbol).is_some_and(|operation| arity == operation.arity())
        || (arity == 1
            && [
                Symbol::SQRT,
                Symbol::EXP,
                Symbol::LOG,
                Symbol::SIN,
                Symbol::COS,
                Symbol::ABS,
                Symbol::CONJ,
            ]
            .contains(&symbol))
}

/// One recursive census supplies storage and eligibility for this export.
fn instruction_description<T>(export: &ExportedInstructions<T>) -> Option<InstructionDescription> {
    let pure_constants = export.constant_functions.iter().all(|constant| {
        constant.tags.is_empty()
            && constant.fixed_args.iter().all(|value| value.im.is_zero())
            && (pure_scalar_head(constant.symbol, constant.fixed_args.len())
                || (constant.fixed_args.is_empty()
                    && [Symbol::PI, Symbol::E].contains(&constant.symbol)))
    });
    let mut result = InstructionDescription {
        bytes: size_of_val(export.instructions.as_slice())
            .checked_add(export.output_count.checked_mul(size_of::<usize>())?)?,
        count: export.instructions.len(),
        taylor_supported: pure_constants && export.sub_evaluators.is_empty(),
        resettable: pure_constants && export.sub_evaluators.is_empty(),
    };
    for instruction in &export.instructions {
        let bytes = match instruction {
            Instruction::Add(_, arguments, _) | Instruction::Mul(_, arguments, _) => {
                size_of_val(arguments.as_slice())
            }
            Instruction::Fun(_, call, _) => {
                let (symbol, tags, arguments) = call.as_ref();
                let pure = tags.is_empty() && pure_scalar_head(*symbol, arguments.len());
                result.resettable &= pure;
                result.taylor_supported &= pure;
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
