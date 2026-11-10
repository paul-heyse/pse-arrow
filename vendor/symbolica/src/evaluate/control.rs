//! Borrowed admission for a single evaluator construction operation.
//!
//! Extents describe mathematical backing, including vector capacity overlap, rather
//! than allocator bookkeeping or an operating-system memory limit. A control is
//! never installed in an evaluator or shared through thread-local state.

/// A safe-point stage within the checked operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OperationStage {
    /// Scalar instruction expansion and its representation conversion.
    #[default]
    Vectorization,
    /// Common-pair elimination and stack optimization.
    Optimization,
    /// Finite scalar native Taylor bodies for qualified local-partial heads.
    KnownHeadGeneration,
    /// Allocation-free source counting or admitted source emission.
    Export,
    /// Complete product, before returning to the caller.
    Finalization,
}

/// An extent that must be accepted before the corresponding construction stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OperationExtent {
    /// The stage whose growth or completion is being checked.
    pub stage: OperationStage,
    /// Incremental work: scalar arithmetic for Taylor, program records for export.
    pub work: usize,
    /// Mathematical backing of the immutable input, which remains live.
    pub input_bytes: usize,
    /// Peak temporary backing, including old/new capacity and representation overlap.
    pub temporary_bytes: usize,
    /// Upper bound for the returned representation's backing.
    pub output_bytes: usize,
    /// Upper bound for numeric slots in the returned representation.
    pub output_entries: usize,
}

impl OperationExtent {
    /// Combined live extent; overflow is a refusal, never a saturated allowance.
    pub fn peak_bytes(self) -> Option<usize> {
        self.input_bytes
            .checked_add(self.temporary_bytes)?
            .checked_add(self.output_bytes)
    }
}

/// Current cancellation/deadline and backing admission, borrowed for one call.
pub trait OperationControl {
    /// The caller's original typed refusal, preserved unchanged.
    type Error;
    /// Check the original operation and admit the extent before growth.
    fn check(&mut self, extent: OperationExtent) -> Result<(), Self::Error>;
}

impl<E, F: FnMut(OperationExtent) -> Result<(), E>> OperationControl for F {
    type Error = E;
    fn check(&mut self, extent: OperationExtent) -> Result<(), E> {
        self(extent)
    }
}

/// Failure of a checked evaluator operation. No variant carries a partial product.
#[derive(Debug, PartialEq, Eq)]
pub enum OperationError<E> {
    /// The caller refused admission or observed its original stop condition.
    Control(E),
    /// An extent or shape could not be represented.
    Overflow,
    /// A fallible destination reservation failed after admission.
    Allocation,
    /// This operation has no maintained growth contract for the requested profile.
    Unsupported(&'static str),
    /// The supplied evaluator or vectorizer has inconsistent structure.
    InvalidProgram(String),
    /// An optimizer observed interruption; a truncated result is not a product.
    Interrupted,
}

impl<E: std::fmt::Display> std::fmt::Display for OperationError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Control(error) => write!(f, "operation refused: {error}"),
            Self::Overflow => f.write_str("evaluator operation extent overflow"),
            Self::Allocation => f.write_str("evaluator operation allocation failed"),
            Self::Unsupported(profile) => write!(f, "unsupported checked operation: {profile}"),
            Self::InvalidProgram(error) => f.write_str(error),
            Self::Interrupted => f.write_str("evaluator operation interrupted"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for OperationError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Control(error) => Some(error),
            _ => None,
        }
    }
}

use super::*;

/// Closed finite local-partial description for a qualified numerical head.
/// The caller owns the mathematical meaning of these symbols. The supplier owns
/// Taylor composition, including mixed components and factorial conventions.
#[derive(Clone, Copy, Debug)]
pub struct KnownHead {
    /// The value head.
    pub symbol: Symbol,
    /// Number of ordinary arguments; the maintained profile accepts one or two.
    pub arity: usize,
    /// First local partials, in argument order. Missing entries refuse demand.
    pub first: [Option<Symbol>; 2],
    /// Symmetric second local partials: 00, 01, 11 (only 00 for arity one).
    pub second: [Option<Symbol>; 3],
}

fn add<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_add(b).ok_or(OperationError::Overflow)
}

fn mul<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_mul(b).ok_or(OperationError::Overflow)
}

// Count the selected coefficient representation without cloning or formatting it.
// GMP is the maintained exact profile; a different backend needs its own backing contract.
pub(super) fn payload<E>(value: &Complex<Rational>) -> Result<usize, OperationError<E>> {
    let mut bytes = 0;
    for value in [&value.re, &value.im] {
        for integer in [value.numerator_ref(), value.denominator_ref()] {
            if let Integer::Large(integer) = integer {
                #[cfg(feature = "integer-gmp")]
                {
                    let bits = usize::try_from(integer.as_raw().capacity())
                        .map_err(|_| OperationError::Overflow)?;
                    bytes = add(bytes, mul(add(bits, 63)? / 64, 8)?)?;
                }
                #[cfg(not(feature = "integer-gmp"))]
                {
                    let _ = integer;
                    return Err(OperationError::Unsupported("exact coefficient backend"));
                }
            }
        }
    }
    Ok(bytes)
}

// Ordinary Numerica Clone can adopt a cleared cached GMP allocation much
// larger than its source. Copy canonical limbs through the backend's fresh-copy
// API instead; no arithmetic, normalization or cache-capacity allowance is needed.
fn copy_integer(value: &Integer) -> Integer {
    match value {
        Integer::Large(value) => Integer::Large(
            crate::domains::integer::MultiPrecisionInteger::from_raw(value.to_raw()),
        ),
        _ => value.clone(),
    }
}
fn copy_rational(value: &Rational) -> Rational {
    Rational::from_unchecked(
        copy_integer(value.numerator_ref()),
        copy_integer(value.denominator_ref()),
    )
}
fn copy_coefficient(value: &Complex<Rational>) -> Complex<Rational> {
    Complex::new(copy_rational(&value.re), copy_rational(&value.im))
}
fn copy_external(
    external: &ExternalFunctionContainer<Complex<Rational>>,
) -> ExternalFunctionContainer<Complex<Rational>> {
    ExternalFunctionContainer {
        export_name: external.export_name.clone(),
        symbol: external.symbol,
        tags: Vec::new(),
        fixed_args: external.fixed_args.iter().map(copy_coefficient).collect(),
        imp: None,
        constant_index: external.constant_index,
        stack: Vec::new(),
        body: None,
    }
}

#[cfg(all(test, feature = "integer-gmp"))]
mod tests {
    use super::*;
    use crate::domains::{
        dual::HyperDual,
        integer::{MultiPrecisionInteger, RawMultiPrecisionInteger},
    };
    use std::convert::Infallible;

    #[test]
    fn checked_known_head_admits_generated_names_before_their_allocation() {
        let symbol = crate::symbol!("checked_name::value");
        let derivative = crate::symbol!(format!("checked_name::{}", "d".repeat(32_768)));
        let source = ExpressionEvaluator {
            stack: vec![Complex::default(); 2],
            param_count: 1,
            reserved_indices: 1,
            instructions: vec![(Instr::ExternalFun(1, 0, vec![0]), ComplexPhase::Any)],
            result_indices: vec![1],
            external_fns: vec![ExternalFunctionContainer::new(symbol, vec![], vec![])],
            settings: OptimizationSettings::default(),
        };
        let heads = [
            KnownHead {
                symbol,
                arity: 1,
                first: [Some(derivative), None],
                second: [None; 3],
            },
            KnownHead {
                symbol: derivative,
                arity: 1,
                first: [None; 2],
                second: [None; 3],
            },
        ];
        let dual = Dualizer::new(
            HyperDual::<Complex<Rational>>::new(vec![vec![0], vec![1]]),
            vec![],
        );
        let mut admitted = 0;
        let product = source
            .try_vectorize_known_with(&dual, &heads, &mut |extent: OperationExtent| {
                if extent.stage == OperationStage::KnownHeadGeneration {
                    assert!(extent.output_bytes >= derivative.get_name().len());
                }
                admitted = admitted.max(extent.output_bytes);
                Ok::<_, Infallible>(())
            })
            .unwrap();
        let retained_names = product
            .external_fns
            .iter()
            .map(|external| external.export_name.capacity())
            .sum::<usize>();
        assert!(retained_names > 32_768);
        assert!(retained_names <= admitted);
    }

    #[test]
    fn checked_taylor_copies_do_not_adopt_unadmitted_cached_limb_capacity() {
        let integer = Integer::Large(MultiPrecisionInteger::from_raw(
            RawMultiPrecisionInteger::from(1) << 129_u32,
        ));
        let coefficient = Complex::from(Rational::from_unchecked(integer, Integer::one()));
        let source = ExpressionEvaluator {
            stack: vec![Complex::default(), coefficient, Complex::default()],
            param_count: 1,
            reserved_indices: 2,
            instructions: vec![(Instr::Add(2, vec![0, 1]), ComplexPhase::Any)],
            result_indices: vec![2],
            external_fns: vec![],
            settings: OptimizationSettings::default(),
        };
        let source_payload = payload::<Infallible>(&source.stack[1]).unwrap();
        // Exercise the selected backend's real cache, including a witness that
        // ordinary Clone adopts its larger capacity. The source remains small.
        let cache = (0..32)
            .map(|_| {
                MultiPrecisionInteger::from_raw(RawMultiPrecisionInteger::from(1) << 1_000_000_u32)
            })
            .collect::<Vec<_>>();
        drop(cache);
        let witness = source.stack[1].clone();
        assert!(payload::<Infallible>(&witness).unwrap() > source_payload * 100);
        drop(witness);
        let dual = Dualizer::new(
            HyperDual::<Complex<Rational>>::new(vec![vec![0], vec![1], vec![2]]),
            vec![],
        );
        let mut extent = OperationExtent::default();
        let product = source
            .try_vectorize_with(&dual, &mut |reported: OperationExtent| {
                if reported.output_bytes > 0 {
                    extent = reported;
                }
                Ok::<_, Infallible>(())
            })
            .unwrap();
        assert!(payload::<Infallible>(&product.get_constants()[0]).unwrap() <= source_payload);
        let numeric = product.stack.capacity() * size_of::<Complex<Rational>>()
            + product
                .stack
                .iter()
                .map(|value| payload::<Infallible>(value).unwrap())
                .sum::<usize>();
        let instruction_bytes = product.instructions.capacity()
            * size_of::<(Instr, ComplexPhase)>()
            + product
                .instructions
                .iter()
                .map(|(instruction, _)| match instruction {
                    Instr::Add(_, args) | Instr::Mul(_, args) => {
                        args.capacity() * size_of::<usize>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
            + product.result_indices.capacity() * size_of::<usize>();
        assert!(
            numeric + instruction_bytes <= extent.output_bytes,
            "all realized returned backing fits the pre-growth extent"
        );
    }
}

impl ExpressionEvaluator<Complex<Rational>> {
    /// Vectorize built-in instructions with a borrowed current-operation control.
    ///
    /// The maintained profile is Taylor depth at most two, with no external
    /// definitions or retained sub-evaluators. Unsupported inputs refuse before
    /// cloning or expanding the input. A passive scan admits the entire finite
    /// expansion and optimizer overlap before growth. Safe points surround each
    /// scalar instruction expansion and each optimizer pass; there is no promise
    /// of interruption inside a scalar exact-arithmetic call or optimizer pass.
    /// The returned evaluator has no inherited or current abort callback.
    pub fn try_vectorize_with<D: DualNumberStructure, C: OperationControl>(
        &self,
        dualizer: &Dualizer<D>,
        control: &mut C,
    ) -> Result<Self, OperationError<C::Error>> {
        self.try_vectorize_known_with(dualizer, &[], control)
    }

    /// Vectorize a closed set of qualified local-partial heads. Unknown external
    /// definitions, tags, callback derivative generation and retained bodies refuse.
    /// Local Taylor bodies are emitted directly as scalar native instructions.
    pub fn try_vectorize_known_with<D: DualNumberStructure, C: OperationControl>(
        &self,
        dualizer: &Dualizer<D>,
        heads: &[KnownHead],
        control: &mut C,
    ) -> Result<Self, OperationError<C::Error>> {
        numerica::domains::backend::integer::IntegerAllocationContext::fresh().run(|| {
            control
                .check(OperationExtent::default())
                .map_err(OperationError::Control)?;
            let extent = self.vectorization_extent(dualizer, heads)?;
            control.check(extent).map_err(OperationError::Control)?;
            let mut generated = HashMap::default();
            for external in &self.external_fns {
                if external.constant_index.is_some() || generated.contains_key(&external.symbol) {
                    continue;
                }
                control
                    .check(OperationExtent {
                        work: 0,
                        stage: OperationStage::KnownHeadGeneration,
                        ..extent
                    })
                    .map_err(OperationError::Control)?;
                let head = KnownHead::find(heads, external.symbol)?;
                generated.insert(
                    external.symbol,
                    super::known_head::generate_known(head, dualizer, control, extent)?,
                );
            }
            let vectorizer = super::known_head::KnownDualizer {
                dualizer,
                generated,
            };
            // Producer callbacks never remain installed in the new product.
            let settings = OptimizationSettings {
                abort_check: None,
                abort_level: 0,
                hot_start: None,
                horner_iterations: self.settings.horner_iterations,
                n_cores: self.settings.n_cores,
                cpe_iterations: self.settings.cpe_iterations,
                max_horner_scheme_variables: self.settings.max_horner_scheme_variables,
                max_common_pair_cache_entries: self.settings.max_common_pair_cache_entries,
                max_common_pair_distance: self.settings.max_common_pair_distance,
                verbose: self.settings.verbose,
                direct_translation: self.settings.direct_translation,
            };
            let evaluator = Self {
                stack: self.stack.iter().map(copy_coefficient).collect(),
                param_count: self.param_count,
                reserved_indices: self.reserved_indices,
                instructions: self.instructions.clone(),
                result_indices: self.result_indices.clone(),
                external_fns: self.external_fns.iter().map(copy_external).collect(),
                settings,
            };
            let mut error = None;
            let stopped_extent = OperationExtent { work: 0, ..extent };
            let result = evaluator.vectorize_impl(
                &vectorizer,
                &mut |stage| match control.check(OperationExtent {
                    stage,
                    ..stopped_extent
                }) {
                    Ok(()) => true,
                    Err(refusal) => {
                        error = Some(refusal);
                        false
                    }
                },
                &copy_coefficient,
                &copy_external,
            );
            if let Some(error) = error {
                return Err(OperationError::Control(error));
            }
            let result = result.map_err(OperationError::InvalidProgram)?;
            if result.settings.abort_level != 0 {
                return Err(OperationError::Interrupted);
            }
            Ok(result)
        })
    }

    fn vectorization_extent<E, D: DualNumberStructure>(
        &self,
        dualizer: &Dualizer<D>,
        heads: &[KnownHead],
    ) -> Result<OperationExtent, OperationError<E>> {
        if self.param_count > self.reserved_indices || self.reserved_indices > self.stack.len() {
            return Err(OperationError::Unsupported("invalid evaluator stack"));
        }
        for external in &self.external_fns {
            if external.body.is_some()
                || !external.tags.is_empty()
                || external.imp.is_some()
                || !external.stack.is_empty()
                || (external.constant_index.is_none() && !external.fixed_args.is_empty())
            {
                return Err(OperationError::Unsupported(
                    "retained body or tagged external head",
                ));
            }
            let head = KnownHead::find::<E>(heads, external.symbol)?;
            if external.constant_index.is_some() && external.fixed_args.len() != head.arity {
                return Err(OperationError::Unsupported("constant head arity"));
            }
        }
        if self.settings.hot_start.is_some() {
            return Err(OperationError::Unsupported("retained Horner hot start"));
        }
        let width = dualizer.dual.get_len();
        let depth = dualizer.dual.get_max_depth();
        if width == 0 || depth > 2 {
            return Err(OperationError::Unsupported(
                "Taylor depth greater than two or empty shape",
            ));
        }
        if dualizer
            .zero_components
            .iter()
            .any(|&(parameter, component)| {
                parameter >= self.param_count || component >= width || component == 0
            })
        {
            return Err(OperationError::Unsupported(
                "invalid structural zero component",
            ));
        }
        let table = dualizer.dual.get_multiplication_table();
        if table
            .iter()
            .any(|&(a, b, c)| a >= width || b >= width || c >= width)
        {
            return Err(OperationError::Unsupported(
                "invalid Taylor multiplication table",
            ));
        }
        let mut generated_names = 0;
        for external in &self.external_fns {
            if external.constant_index.is_none() {
                generated_names = add(
                    generated_names,
                    KnownHead::find::<E>(heads, external.symbol)?.name_bytes(depth, heads)?,
                )?;
            }
        }
        // mul emits W initial products and a product/add pair for each table entry.
        // Unary kernels add at most two W-sized setup/final maps and, per Taylor
        // depth, a convolution, rescale, addition and one scalar setup operation.
        // Powf is log, a convolution, adjacent assignments and exp. Structural
        // zeros only remove work. These are the shared map_instruction kernels.
        let convolution = add(width, mul(2, table.len())?)?;
        let unary = add(
            add(
                mul(depth, add(add(convolution, mul(2, width)?)?, 1)?)?,
                mul(2, width)?,
            )?,
            2,
        )?;
        let power = add(mul(2, unary)?, convolution)?;
        let mut work = 0;
        let mut unfolded = 0;
        let mut constant_vectors = self
            .reserved_indices
            .checked_sub(self.param_count)
            .ok_or_else(|| OperationError::InvalidProgram("invalid reserved indices".into()))?;
        let mut operand_capacity = 0;
        for (instruction, _) in &self.instructions {
            let (operations, count, constants) = match instruction {
                Instr::Add(_, args) | Instr::Mul(_, args) => {
                    if args.is_empty() {
                        return Err(OperationError::Unsupported("empty arithmetic instruction"));
                    }
                    operand_capacity = add(operand_capacity, args.capacity())?;
                    let count = args.len().saturating_sub(1);
                    (
                        mul(
                            count,
                            if matches!(instruction, Instr::Add(..)) {
                                width
                            } else {
                                convolution
                            },
                        )?,
                        count.max(1),
                        0,
                    )
                }
                Instr::Pow(_, _, -1) => (unary, 1, 3),
                Instr::Pow(..) => {
                    return Err(OperationError::Unsupported(
                        "integer power other than inversion",
                    ));
                }
                Instr::Powf(..) => (power, 1, 7),
                Instr::BuiltinFun(_, symbol, _)
                    if [
                        Symbol::EXP_ID,
                        Symbol::LOG_ID,
                        Symbol::SQRT_ID,
                        Symbol::SIN_ID,
                        Symbol::COS_ID,
                        Symbol::ABS_ID,
                        Symbol::CONJ_ID,
                    ]
                    .contains(&symbol.get_id()) =>
                {
                    (unary, 1, 7)
                }
                Instr::BuiltinFun(..) => {
                    return Err(OperationError::Unsupported("unknown builtin"));
                }
                Instr::ExternalFun(_, index, args) => {
                    let external = self
                        .external_fns
                        .get(*index)
                        .ok_or(OperationError::Unsupported("external function index"))?;
                    let head = KnownHead::find::<E>(heads, external.symbol)?;
                    if args.len() != head.arity {
                        return Err(OperationError::Unsupported("qualified head arity"));
                    }
                    operand_capacity = add(operand_capacity, args.capacity())?;
                    (head.instruction_bound(width, table.len())?, 1, 1)
                }
                Instr::Join(..) | Instr::IfElse(..) | Instr::Goto(..) | Instr::Label(..) => {
                    (0, 1, 0)
                }
            };
            work = add(work, operations)?;
            unfolded = add(unfolded, count)?;
            constant_vectors = add(constant_vectors, constants)?;
        }
        // Assignments are retained instructions but are not mathematical arithmetic.
        let emitted = add(work, mul(mul(3, width)?, unfolded)?)?;
        let constants = mul(constant_vectors, width)?;
        // Common-pair extraction of the binary expanded program can introduce at
        // most one new binary instruction per original binary instruction: every
        // extraction consumes at least two operand occurrences. Stack unfolding
        // and final pair extraction therefore fit the doubled instruction extent.
        let instructions = mul(2, emitted)?;
        let entries = add(add(mul(self.param_count, width)?, constants)?, instructions)?;
        let mut coefficients = self
            .stack
            .iter()
            .try_fold(0, |bytes, value| add(bytes, payload(value)?))?;
        let mut external_bytes = mul(
            self.external_fns.capacity(),
            size_of::<ExternalFunctionContainer<Complex<Rational>>>(),
        )?;
        for external in &self.external_fns {
            external_bytes = add(external_bytes, external.export_name.capacity())?;
            external_bytes = add(
                external_bytes,
                mul(
                    add(external.fixed_args.capacity(), external.stack.capacity())?,
                    size_of::<Complex<Rational>>(),
                )?,
            )?;
            if let Some(implementation) = &external.imp {
                external_bytes = add(external_bytes, size_of_val(implementation.as_ref()))?;
            }
            coefficients = external
                .stack
                .iter()
                .try_fold(coefficients, |bytes, value| add(bytes, payload(value)?))?;
            coefficients = external
                .fixed_args
                .iter()
                .try_fold(coefficients, |bytes, value| add(bytes, payload(value)?))?;
        }
        let input_bytes = add(
            external_bytes,
            add(
                add(
                    mul(self.stack.capacity(), size_of::<Complex<Rational>>())?,
                    coefficients,
                )?,
                add(
                    mul(
                        self.instructions.capacity(),
                        size_of::<(Instr, ComplexPhase)>(),
                    )?,
                    mul(
                        add(operand_capacity, self.result_indices.capacity())?,
                        size_of::<usize>(),
                    )?,
                )?,
            )?,
        )?;
        let outputs = mul(self.result_indices.len(), width)?;
        let head_bytes = add(
            generated_names,
            mul(
                mul(6, self.external_fns.len())?,
                add(
                    size_of::<ExternalFunctionContainer<Complex<Rational>>>(),
                    2 * size_of::<usize>(),
                )?,
            )?,
        )?;
        let output_bytes = add(
            add(external_bytes, head_bytes)?,
            add(
                add(mul(entries, size_of::<Complex<Rational>>())?, coefficients)?,
                add(
                    mul(
                        instructions,
                        add(size_of::<(Instr, ComplexPhase)>(), 2 * size_of::<usize>())?,
                    )?,
                    mul(outputs, size_of::<usize>())?,
                )?,
            )?,
        )?;
        // Capacity doubling, simultaneous VectorInstruction/Instr representations,
        // slot maps, unfold maps, branch/label maps and pair-extraction maps/lists.
        // Binary instructions have at most one pair: all optimizer records are
        // linear in emitted instruction count, not in an application-side source
        // predictor. 64 machine words per instruction covers the simultaneous
        // maps, per-key table metadata, vectors and geometric capacity overlap.
        let temporary_bytes = add(
            generated_names,
            add(
                add(input_bytes, mul(2, output_bytes)?)?,
                mul(
                    add(add(emitted, self.reserved_indices)?, outputs)?,
                    64 * size_of::<usize>(),
                )?,
            )?,
        )?;
        let extent = OperationExtent {
            stage: OperationStage::Vectorization,
            work,
            input_bytes,
            temporary_bytes,
            output_bytes,
            output_entries: entries,
        };
        extent.peak_bytes().ok_or(OperationError::Overflow)?;
        Ok(extent)
    }
}
