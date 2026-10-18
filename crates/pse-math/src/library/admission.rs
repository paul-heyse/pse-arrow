// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Construction bounds for the pinned Numerica Dualizer (Taylor orders zero to two).
//! This counts its convolution and primitive expansion, not a differentiation engine.
use super::*;
use crate::jets::{EvaluationLimits, JetLayout};
use std::sync::{Arc, atomic::AtomicBool};
use symbolica::atom::AtomView;
#[cfg(test)]
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
    let first = if layout.order >= pse_kernels::DerivativeOrder::First {
        mul(2, layout.coordinates.len())?
    } else {
        0
    };
    let products = layout.pairs.iter().try_fold(add(1, first)?, |n, &(i, j)| {
        add(n, if i == j { 3 } else { 4 })
    })?;
    expansion_with_products(c, layout.order, layout.width(), products)
}
fn expansion_with_products(
    c: OperationCount,
    order: pse_kernels::DerivativeOrder,
    w: usize,
    products: usize,
) -> Result<usize, MathError> {
    let convolution = products
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
    evaluator_construction_shape(
        expressions,
        parameters,
        order,
        add(add(1, first)?, second)?,
        products_for(coordinates, order)?,
    )
}
pub(crate) fn evaluator_construction_shape(
    expressions: &[Atom],
    parameters: usize,
    order: pse_kernels::DerivativeOrder,
    width: usize,
    products: usize,
) -> Result<EvaluatorConstruction, MathError> {
    let source = expressions
        .iter()
        .try_fold(0, |bytes, atom| add(bytes, atom.as_view().get_byte_size()))?;
    let operations = source_operations(expressions)?;
    construction_shape(source, operations, expressions.len(), parameters, order, width, products)
}
fn construction_shape(
    source: usize, operations: usize, outputs: usize, parameters: usize,
    order: pse_kernels::DerivativeOrder, width: usize, products: usize,
) -> Result<EvaluatorConstruction, MathError> {
    let mut per_operation = 0;
    for count in [
        OperationCount::new(1, 0, 0, 0),
        OperationCount::new(0, 1, 0, 0),
        OperationCount::new(0, 0, 1, 0),
        OperationCount::new(0, 0, 0, 1),
    ] {
        per_operation = per_operation.max(expansion_with_products(count, order, width, products)?);
    }
    let expanded = mul(operations, per_operation)?;
    let scalar_instructions = add(add(source, operations)?, outputs)?;
    let vectors = add(
        add(mul(2, parameters)?, outputs)?,
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
fn reserve_facts(facts: &EvaluatorFacts, initial: EvaluatorConstruction) -> Result<(), MathError> {
    // Immutable evaluator and private construction/worker copy retain independent charge.
    let extra = add(
        mul(facts.storage.numeric_entries.saturating_sub(initial.entries), 2 * size_of::<f64>())?,
        facts.storage.instruction_bytes.saturating_sub(initial.instructions),
    )?;
    crate::construction::additional(extra)
}

type ScalarProductKey = (Vec<Atom>, Vec<Atom>, Optimization);

/// Immutable exact scalar products. Every entry retains the allocation owner that
/// first published it; borrowing a bundle never transfers that charge to a worker.
#[derive(Clone, Default)]
pub struct ScalarProducts {
    evaluators: Arc<std::collections::HashMap<Arc<ScalarProductKey>, Arc<ScalarProduct>>>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
    foreign_bytes: usize,
}
struct ScalarProduct {
    evaluator: ExpressionEvaluator<Complex<Rational>>,
    bytes: usize,
    facts: EvaluatorFacts,
    source_bytes: usize,
    source_operations: usize,
    source_atom_operations: usize,
    owner: std::sync::OnceLock<Arc<dyn crate::AllocationOwner>>,
}
impl std::fmt::Debug for ScalarProducts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScalarProducts")
            .field("entries", &self.len())
            .finish()
    }
}
impl ScalarProducts {
    /// Number of exact preparations, excluding derivative conversions and workers.
    pub fn len(&self) -> usize {
        self.evaluators.len()
    }
    /// Whether this bundle contains no scalar preparation.
    pub fn is_empty(&self) -> bool {
        self.evaluators.is_empty()
    }
    /// Complete reachable scalar storage, including exact keys and descriptors.
    pub fn retained_bytes(&self) -> usize {
        self.evaluators.values().fold(
            self.descriptor_bytes().saturating_add(self.foreign_bytes),
            |bytes, product| bytes.saturating_add(product.bytes),
        )
    }
    pub(crate) fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + self.evaluators.capacity()
                * (size_of::<Arc<ScalarProductKey>>() + size_of::<Arc<ScalarProduct>>() + 1)
    }
    pub(crate) fn unowned_bytes(&self) -> usize {
        self.evaluators
            .values()
            .filter(|product| product.owner.get().is_none())
            .fold(0usize, |bytes, product| bytes.saturating_add(product.bytes))
    }
    /// Whether this bundle contains newly constructed entries needing an owner.
    pub fn has_unowned(&self) -> bool {
        self.evaluators
            .values()
            .any(|product| product.owner.get().is_none())
    }
    /// Conservative foreign capacity retained from the original constructing jobs,
    /// counted once per constructor, not once per exact entry or borrower.
    pub fn foreign_allowance_bytes(&self) -> usize {
        self.foreign_bytes
    }
    /// Anchor only newly constructed entries. Inherited entries keep their original owner.
    pub fn with_owner(self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.with_owner_and_foreign_allowance(owner, 0)
    }
    /// Transfer an already admitted conservative foreign allowance with newly built
    /// exact entries. It covers unknown arbitrary-precision/library heaps, not exact RSS.
    pub fn with_owner_and_foreign_allowance(
        mut self,
        owner: Arc<dyn crate::AllocationOwner>,
        foreign_bytes: usize,
    ) -> Self {
        let mut attached = false;
        for product in self.evaluators.values() {
            attached |= product.owner.set(owner.clone()).is_ok();
        }
        if attached {
            self.foreign_bytes = self.foreign_bytes.saturating_add(foreign_bytes);
        }
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
}

/// Preserve the two existing source-work policies: scalar/provider preparation
/// counts builtins explicitly, while Taylor admission uses Atom's operation count.
#[derive(Clone, Copy)]
enum SourceOperationLimit {
    Scalar(usize),
    Taylor(usize),
}
impl SourceOperationLimit {
    fn check(self, scalar: usize, taylor: usize) -> Result<(), MathError> {
        let (operations, limit) = match self {
            Self::Scalar(limit) => (scalar, limit),
            Self::Taylor(limit) => (taylor, limit),
        };
        if operations > limit { return Err(MathError::Limit("scalar construction operations")); }
        Ok(())
    }
}

/// Construction-local index borrowing immutable retained exact products. Neither
/// mutable evaluator state nor cancellation controls enter the retained bundle.
#[derive(Default)]
pub(crate) struct ScalarCache {
    evaluators: std::collections::HashMap<Arc<ScalarProductKey>, Arc<ScalarProduct>>,
    builds: usize,
    foreign_bytes: usize,
}
impl ScalarCache {
    pub(crate) fn from_products(products: &ScalarProducts) -> Self {
        Self {
            evaluators: products.evaluators.as_ref().clone(),
            builds: 0,
            foreign_bytes: products.foreign_bytes,
        }
    }
    pub(crate) fn freeze(self) -> ScalarProducts {
        ScalarProducts {
            evaluators: Arc::new(self.evaluators),
            owner: None,
            foreign_bytes: self.foreign_bytes,
        }
    }
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.evaluators.len()
    }
    #[cfg(test)]
    pub(crate) fn builds(&self) -> usize {
        self.builds
    }

    pub(crate) fn admit_prepare(
        &mut self,
        expressions: &[Atom],
        parameters: &[Atom],
        options: Optimization,
        cancelled: &Arc<AtomicBool>,
        limits: EvaluationLimits,
    ) -> Result<(), MathError> {
        let product = self.prepare(expressions, parameters, options, cancelled, SourceOperationLimit::Scalar(limits.operations))?;
        if total(product.facts.operations)? > limits.operations {
            return Err(MathError::Limit("scalar evaluator operations"));
        }
        limits.allocation(product.facts.storage.numeric_entries)
    }

    fn prepare(
        &mut self,
        expressions: &[Atom],
        parameters: &[Atom],
        options: Optimization,
        cancelled: &Arc<AtomicBool>,
        source_limit: SourceOperationLimit,
    ) -> Result<&ScalarProduct, MathError> {
        let _phase = pse_diagnostics::observation::phase("math.scalar_prepare");
        if cancelled.load(std::sync::atomic::Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        // Scalar optimization does not consume the later execution selector.
        // Share exact scalar preparation across interpreted/native Value routes.
        let scalar_options = Optimization {
            evaluator: Evaluator::Interpreted,
            ..options
        };
        let key = (expressions.to_vec(), parameters.to_vec(), scalar_options);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.scalar_cache_lookups", "count", 1);
        }
        if !self.evaluators.contains_key(&key) {
            let _construction = pse_diagnostics::observation::current().map(|context| {
                context.count("math.scalar_constructions", "count", 1);
                context.phase("math.scalar_construction")
            });
            let source_operations = source_operations(expressions)?;
            let source_atom_operations = expressions.iter().try_fold(0, |sum, expression| add(sum, total(expression.count_operations())?))?;
            source_limit.check(source_operations, source_atom_operations)?;
            let source_bytes = expressions.iter().try_fold(0, |bytes, atom| add(bytes, atom.as_view().get_byte_size()))?;
            let initial = construction_shape(source_bytes, source_operations, expressions.len(), parameters.len(), pse_kernels::DerivativeOrder::Value, 1, 1)?;
            let atoms = expressions
                .iter()
                .chain(parameters)
                .try_fold(0, |bytes, atom| add(bytes, atom.as_view().get_byte_size()))?;
            crate::construction::additional(add(
                mul(initial.entries, 2 * size_of::<Complex<Rational>>())?,
                add(initial.instructions, atoms)?,
            )?)?;
            let evaluator = exact_evaluator(expressions, parameters, options, cancelled)?;
            let export = evaluator.export_instructions();
            let facts = EvaluatorFacts::inspect(&export, evaluator.count_operations(), "math.rational_representation_inspections")?;
            reserve_facts(&facts, initial)?;
            let bytes = add(size_of::<ScalarProduct>(), add(atoms, add(
                mul(facts.storage.numeric_entries, 2 * size_of::<Complex<Rational>>())?,
                facts.storage.instruction_bytes,
            )?)?)?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            self.evaluators.insert(
                Arc::new(key.clone()),
                Arc::new(ScalarProduct {
                    evaluator,
                    bytes,
                    facts,
                    source_bytes,
                    source_operations,
                    source_atom_operations,
                    owner: std::sync::OnceLock::new(),
                }),
            );
            self.builds += 1;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("math.scalar_constructions_completed", "count", 1);
            }
        } else if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.scalar_cache_reuses", "count", 1);
        }
        let product = self.evaluators.get(&key).map(Arc::as_ref)
            .ok_or_else(|| MathError::Contract("missing prepared scalar evaluator".into()))?;
        source_limit.check(product.source_operations, product.source_atom_operations)?;
        Ok(product)
    }
}

/// Scalar-only provider composition: repeated identical lifts share exact
/// preparation, while every mutable numeric/order product remains private.
pub(crate) fn bounded_scalar_evaluator(
    expressions: &[Atom],
    parameters: &[Atom],
    options: Optimization,
    cancelled: &Arc<AtomicBool>,
    limits: EvaluationLimits,
    scalars: &mut ScalarCache,
) -> Result<FloatingEvaluator, MathError> {
    let product = scalars.prepare(expressions, parameters, options, cancelled, SourceOperationLimit::Scalar(limits.operations))?;
    let initial = construction_shape(product.source_bytes, product.source_operations, expressions.len(), parameters.len(), pse_kernels::DerivativeOrder::Value, 1, 1)?;
    if total(product.facts.operations)? > limits.operations {
        return Err(MathError::Limit("scalar evaluator operations"));
    }
    limits.allocation(product.facts.storage.numeric_entries)?;
    reserve_facts(&product.facts, initial)?;
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    FloatingEvaluator::new(product.evaluator.clone().map_coeff(&|c| c.re.to_f64()))
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
    scalars: &mut ScalarCache,
) -> Result<FloatingEvaluator, MathError> {
    if zero_components.iter().any(|&(parameter, component)| {
        parameter >= parameters.len() || component == 0 || component >= layout.width()
    }) {
        return Err(MathError::Contract(
            "invalid structural Taylor zero component".into(),
        ));
    }
    let first = if layout.order >= pse_kernels::DerivativeOrder::First {
        mul(2, layout.coordinates.len())?
    } else {
        0
    };
    let products = layout.pairs.iter().try_fold(add(1, first)?, |n, &(i, j)| {
        add(n, if i == j { 3 } else { 4 })
    })?;
    let product = scalars.prepare(expressions, parameters, options, cancelled, SourceOperationLimit::Taylor(remaining))?;
    let construction = construction_shape(product.source_bytes, product.source_operations, expressions.len(), parameters.len(), layout.order, layout.width(), products)?;
    let evaluator = &product.evaluator;
    let scalar = product.facts.operations;
    if total(scalar)? > remaining {
        return Err(MathError::Limit("scalar evaluator operations"));
    }
    limits.allocation(add(occupied_entries, product.facts.storage.numeric_entries)?)?;
    if layout.width() == 1 {
        reserve_facts(&product.facts, construction)?;
        return FloatingEvaluator::new(evaluator.clone().map_coeff(&|c| c.re.to_f64()));
    }
    if !product.facts.taylor_supported {
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
            product.facts.constants,
            add(
                mul(3, product.facts.instructions)?,
                add(mul(7, scalar.function_calls)?, mul(3, scalar.inversions)?)?,
            )?,
        )?,
    )?;
    limits.allocation(add(
        occupied_entries,
        add(bound, mul(layout.width(), vectors)?)?,
    )?)?;
    let dual = symbolica::prelude::HyperDual::<Complex<Rational>>::new(layout.shape.clone());
    let evaluator = scoped_abort(cancelled, || {
        evaluator
            .clone()
            .vectorize(&symbolica::prelude::Dualizer::new(
                dual,
                zero_components.to_vec(),
            ))
    })
    .map_err(|error| {
        if cancelled.load(std::sync::atomic::Ordering::Acquire) {
            MathError::Cancelled
        } else {
            MathError::Library(error)
        }
    })?;
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    // Expanded exact Taylor and floating programs are distinct representations.
    let expanded_export = evaluator.export_instructions();
    let facts = EvaluatorFacts::inspect(&expanded_export, evaluator.count_operations(), "math.taylor_representation_inspections")?;
    if total(facts.operations)? > bound {
        return Err(MathError::Contract("library Taylor expansion exceeded admitted bound".into()));
    }
    reserve_facts(&facts, construction)?;
    limits.allocation(add(occupied_entries, facts.storage.numeric_entries)?)?;
    drop(expanded_export);
    FloatingEvaluator::new(evaluator.map_coeff(&|c| c.re.to_f64()))
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
            &mut Default::default(),
        )
        .unwrap();
        reserve_facts(&compiled.facts, second).unwrap();
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
    fn large_rational_scalar_products_keep_one_foreign_allowance_per_constructor() {
        #[derive(Debug)]
        struct Owner;
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let huge = Rational::from(i64::MAX).pow(16);
        let expressions = vec![&x * Atom::num(huge) + 1];
        let cancel = Arc::new(AtomicBool::new(false));
        let mut cache = ScalarCache::default();
        cache
            .admit_prepare(
                &expressions,
                std::slice::from_ref(&x),
                Optimization::default(),
                &cancel,
                EvaluationLimits::default(),
            )
            .unwrap();
        assert!(
            cache.evaluators.values().any(|product| product
                .evaluator
                .export_instructions()
                .constants
                .iter()
                .any(|constant| constant.re.to_string().len() > 100)),
            "test requires an actual arbitrary-precision exact constant"
        );
        let owner = Arc::new(Owner);
        let weak = Arc::downgrade(&owner);
        let known = cache.freeze();
        let known_bytes = known.retained_bytes();
        let retained = known.with_owner_and_foreign_allowance(owner.clone(), 64 << 20);
        assert_eq!(retained.retained_bytes(), known_bytes + (64 << 20));
        let borrowed = ScalarCache::from_products(&retained)
            .freeze()
            .with_owner_and_foreign_allowance(owner.clone(), 64 << 20);
        assert_eq!(
            borrowed.foreign_allowance_bytes(),
            64 << 20,
            "borrowing must not add another foreign population"
        );
        drop(owner);
        drop(retained);
        assert!(weak.upgrade().is_some());
        drop(borrowed);
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn scalar_preparation_refusal_and_cancellation_leave_no_cache_product() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let expressions = [x.clone() * &x + 3];
        let cancel = Arc::new(AtomicBool::new(false));
        let mut scalars = ScalarCache::default();
        assert!(matches!(
            crate::construction::scoped(Arc::new(Refuse), || {
                bounded_scalar_evaluator(
                    &expressions,
                    std::slice::from_ref(&x),
                    Optimization::default(),
                    &cancel,
                    EvaluationLimits::default(),
                    &mut scalars,
                )
            }),
            Err(MathError::Limit("test pool full"))
        ));
        assert_eq!(scalars.len(), 0);
        let mut first = bounded_scalar_evaluator(
            &expressions,
            std::slice::from_ref(&x),
            Optimization::default(),
            &cancel,
            EvaluationLimits::default(),
            &mut scalars,
        )
        .unwrap();
        let mut second = bounded_scalar_evaluator(
            &expressions,
            std::slice::from_ref(&x),
            Optimization::default(),
            &cancel,
            EvaluationLimits::default(),
            &mut scalars,
        )
        .unwrap();
        assert_eq!(scalars.len(), 1);
        let mut result = [0.];
        first.evaluator.try_evaluate(&[2.], &mut result).unwrap();
        assert_eq!(result, [7.]);
        second.evaluator.try_evaluate(&[4.], &mut result).unwrap();
        assert_eq!(result, [19.]);
        first.evaluator.try_evaluate(&[2.], &mut result).unwrap();
        assert_eq!(result, [7.]);
        for limits in [
            EvaluationLimits { operations: 1, ..Default::default() },
            EvaluationLimits { scratch_bytes: 1, ..Default::default() },
        ] {
            assert!(matches!(bounded_scalar_evaluator(&expressions, std::slice::from_ref(&x), Optimization::default(), &cancel, limits, &mut scalars), Err(MathError::Limit(_))));
            assert_eq!(scalars.builds(), 1, "a retained exact representation is checked under the new policy");
        }
        cancel.store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            bounded_scalar_evaluator(
                &expressions,
                std::slice::from_ref(&x),
                Optimization::default(),
                &cancel,
                EvaluationLimits::default(),
                &mut scalars
            ),
            Err(MathError::Cancelled)
        ));
        assert_eq!(
            scalars.len(),
            1,
            "cancelled cache hits do not mint another mutable product"
        );
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
        crate::construction::scoped(owner.clone(), || reserve_facts(&EvaluatorFacts::inspect(&export, OperationCount::new(0, 0, 0, 0), "math.test_representation_inspections")?, initial)).unwrap();
        assert_eq!(
            owner.0.load(std::sync::atomic::Ordering::SeqCst),
            2 * size_of::<f64>()
        );
        assert!(matches!(
            crate::construction::scoped(Arc::new(Refuse), || reserve_facts(&EvaluatorFacts::inspect(&export, OperationCount::new(0, 0, 0, 0), "math.test_representation_inspections")?, initial)),
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
            instructions: instruction_description(&export).unwrap().bytes,
        };
        let owner = Arc::new(Grow(std::sync::atomic::AtomicUsize::new(0)));
        crate::construction::scoped(owner.clone(), || reserve_facts(&EvaluatorFacts::inspect(&export, OperationCount::new(0, 0, 0, 0), "math.test_representation_inspections")?, initial)).unwrap();
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
            &mut Default::default(),
        )
        .unwrap();
        let mut compiled = compiled;
        let mut output = [0.0];
        compiled.evaluator.evaluate(&[2.0, 3.0, 4.0], &mut output);
        assert_eq!(output, [30.0]);
    }
}
