// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked native construction, retained exact scalar products and scientific
//! source/final-operation limits. Supplier controls own native growth and copies.
use super::*;
use crate::jets::{EvaluationLimits, JetLayout};
use hashbrown::{Equivalent, HashMap, hash_map::EntryRef};
use std::collections::hash_map::RandomState;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::AtomicBool;
use symbolica::atom::AtomView;
#[cfg(test)]
use symbolica::atom::Symbol;
use symbolica::evaluate::OperationCount;

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

/// Distinct source populations: encoded payload bytes are not numeric slots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SourceShape {
    bytes: usize,
    nodes: usize,
    operands: usize,
    literal_bytes: usize,
    operations: usize,
}
fn source_shape<'a>(
    expressions: impl IntoIterator<Item = &'a Atom>,
) -> Result<SourceShape, MathError> {
    let mut shape = SourceShape::default();
    for expression in expressions {
        shape.bytes = add(shape.bytes, expression.as_view().get_byte_size())?;
        let mut failed = false;
        expression.as_view().visitor(&mut |node| {
            let mut record = || -> Result<(), MathError> {
                shape.nodes = add(shape.nodes, 1)?;
                let (operands, operations) = match node {
                    AtomView::Add(a) => (a.get_nargs(), a.get_nargs().saturating_sub(1)),
                    AtomView::Mul(a) => (a.get_nargs(), a.get_nargs().saturating_sub(1)),
                    AtomView::Pow(p) => (
                        2,
                        match isize::try_from(p.get_exp()) {
                            Ok(exponent) => add(
                                exponent.unsigned_abs().saturating_sub(1),
                                usize::from(exponent < 0),
                            )?,
                            Err(_) => 1,
                        },
                    ),
                    // Function nodes retain their operands and the evaluator call
                    // independently of the encoding of their symbol or arguments.
                    AtomView::Fun(f) => (f.get_nargs(), 1),
                    AtomView::Num(_) => {
                        shape.literal_bytes = add(shape.literal_bytes, node.get_byte_size())?;
                        (0, 0)
                    }
                    _ => (0, 0),
                };
                shape.operands = add(shape.operands, operands)?;
                shape.operations = add(shape.operations, operations)?;
                Ok(())
            };
            failed |= record().is_err();
            !failed
        });
        if failed {
            return Err(MathError::Limit("source evaluator population"));
        }
    }
    Ok(shape)
}
type ScalarProductKey = (Vec<Atom>, Vec<Atom>, Optimization, Vec<(Atom, Atom)>);
type ScalarProductMap = HashMap<Arc<ScalarProductKey>, Arc<ScalarProduct>, RandomState>;

/// Borrow the exact ordered scalar identity. Both hashing and owned construction
/// use the same tuple fields; only the later execution selector is irrelevant.
struct ScalarProductQuery<'a> {
    expressions: &'a [Atom],
    parameters: &'a [Atom],
    options: Optimization,
    definitions: &'a [(Atom, Atom)],
}
impl<'a> ScalarProductQuery<'a> {
    fn new(
        expressions: &'a [Atom],
        parameters: &'a [Atom],
        options: Optimization,
        definitions: &'a [(Atom, Atom)],
    ) -> Self {
        Self {
            expressions,
            parameters,
            definitions,
            options: Optimization {
                evaluator: Evaluator::Interpreted,
                ..options
            },
        }
    }
    fn owned_key(&self) -> Arc<ScalarProductKey> {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.scalar_key_constructions", "count", 1);
            context.count(
                "math.scalar_key_vector_allocations",
                "count",
                u64::from(!self.expressions.is_empty())
                    + u64::from(!self.parameters.is_empty())
                    + u64::from(!self.definitions.is_empty()),
            );
        }
        Arc::new((
            self.expressions.to_vec(),
            self.parameters.to_vec(),
            self.options,
            self.definitions.to_vec(),
        ))
    }
}
impl Hash for ScalarProductQuery<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (
            self.expressions,
            self.parameters,
            self.options,
            self.definitions,
        )
            .hash(state);
    }
}
impl Equivalent<Arc<ScalarProductKey>> for ScalarProductQuery<'_> {
    fn equivalent(&self, key: &Arc<ScalarProductKey>) -> bool {
        self.expressions == key.0
            && self.parameters == key.1
            && self.options == key.2
            && self.definitions == key.3
    }
}

/// Immutable exact scalar products. Every entry retains the allocation owner that
/// first published it; borrowing a bundle never transfers that charge to a worker.
#[derive(Clone, Default)]
pub struct ScalarProducts {
    evaluators: Arc<ScalarProductMap>,
    partials: Arc<HashMap<Arc<crate::execution::LocalPartialsKey>, Arc<PartialProduct>>>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
    foreign_bytes: usize,
}
struct ScalarProduct {
    evaluator: ExpressionEvaluator<Complex<Rational>>,
    bytes: usize,
    facts: EvaluatorFacts,
    source: SourceShape,
    source_atom_operations: usize,
    owner: std::sync::OnceLock<Arc<dyn crate::AllocationOwner>>,
}
struct PartialProduct {
    partials: Arc<crate::execution::LocalPartials>,
    bytes: usize,
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
    /// Number of exact scalar evaluators, excluding named derivatives and workers.
    pub fn len(&self) -> usize {
        self.evaluators.len()
    }
    /// Whether this bundle contains no exact evaluator or named derivative preparation.
    pub fn is_empty(&self) -> bool {
        self.evaluators.is_empty() && self.partials.is_empty()
    }
    /// Retained exact named derivative preparations, excluding numerical workers.
    pub fn derivative_preparations(&self) -> usize {
        self.partials.len()
    }
    /// Whether this bundle retains every named derivative allocation of a nonempty
    /// seed. Additional preparations are allowed. This observes shared backing,
    /// not mathematical or scientific equivalence.
    pub fn contains_same_derivative_preparations(&self, seed: &Self) -> bool {
        !seed.partials.is_empty()
            && seed.partials.iter().all(|(key, product)| {
                self.partials
                    .get(key)
                    .is_some_and(|other| Arc::ptr_eq(&product.partials, &other.partials))
            })
    }
    /// Complete reachable scalar storage, including exact keys and descriptors.
    pub fn retained_bytes(&self) -> usize {
        self.partials.values().fold(
            self.evaluators.values().fold(
                self.descriptor_bytes().saturating_add(self.foreign_bytes),
                |bytes, product| bytes.saturating_add(product.bytes),
            ),
            |bytes, product| bytes.saturating_add(product.bytes),
        )
    }
    pub(crate) fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + self.evaluators.capacity()
                * (size_of::<Arc<ScalarProductKey>>() + size_of::<Arc<ScalarProduct>>() + 1)
            + self.partials.capacity()
                * (size_of::<Arc<crate::execution::LocalPartialsKey>>()
                    + size_of::<Arc<PartialProduct>>()
                    + 1)
    }
    pub(crate) fn unowned_bytes(&self) -> usize {
        self.partials
            .values()
            .filter(|product| product.owner.get().is_none())
            .fold(
                self.evaluators
                    .values()
                    .filter(|product| product.owner.get().is_none())
                    .fold(0usize, |bytes, product| bytes.saturating_add(product.bytes)),
                |bytes, product| bytes.saturating_add(product.bytes),
            )
    }
    /// Whether this bundle contains newly constructed entries needing an owner.
    pub fn has_unowned(&self) -> bool {
        self.evaluators
            .values()
            .any(|product| product.owner.get().is_none())
            || self
                .partials
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
        for product in self.partials.values() {
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
        if operations > limit {
            return Err(MathError::Limit("scalar construction operations"));
        }
        Ok(())
    }
}

/// Construction-local index borrowing immutable retained exact products. Neither
/// mutable evaluator state nor cancellation controls enter the retained bundle.
#[derive(Default)]
pub(crate) struct ScalarCache {
    evaluators: ScalarProductMap,
    partials: HashMap<Arc<crate::execution::LocalPartialsKey>, Arc<PartialProduct>>,
    builds: usize,
    foreign_bytes: usize,
}
impl ScalarCache {
    pub(crate) fn from_products(products: &ScalarProducts) -> Self {
        Self {
            evaluators: products.evaluators.as_ref().clone(),
            partials: products.partials.as_ref().clone(),
            builds: 0,
            foreign_bytes: products.foreign_bytes,
        }
    }
    pub(crate) fn freeze(self) -> ScalarProducts {
        ScalarProducts {
            evaluators: Arc::new(self.evaluators),
            partials: Arc::new(self.partials),
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
    pub(crate) fn local_partials(
        &self,
        key: &crate::execution::LocalPartialsKey,
    ) -> Option<Arc<crate::execution::LocalPartials>> {
        self.partials
            .get(key)
            .map(|product| Arc::clone(&product.partials))
    }
    pub(crate) fn retain_local_partials(
        &mut self,
        key: crate::execution::LocalPartialsKey,
        partials: Arc<crate::execution::LocalPartials>,
        bytes: usize,
    ) {
        self.partials.insert(
            Arc::new(key),
            Arc::new(PartialProduct {
                partials,
                bytes,
                owner: Default::default(),
            }),
        );
    }

    pub(crate) fn admit_prepare<'a>(
        &mut self,
        expressions: &[Atom],
        parameters: &[Atom],
        definitions: &[(Atom, Atom)],
        options: Optimization,
        stop: impl Into<ConstructionStop<'a>>,
        limits: EvaluationLimits,
    ) -> Result<(), MathError> {
        let stop = stop.into();
        stop.check()?;
        let product = self.prepare(
            expressions,
            parameters,
            definitions,
            options,
            stop,
            limits,
            0,
            SourceOperationLimit::Scalar(limits.operations),
        )?;
        if total(product.facts.operations)? > limits.operations {
            return Err(MathError::Limit("scalar evaluator operations"));
        }
        limits.allocation(product.facts.storage.numeric_entries)
    }

    fn prepare(
        &mut self,
        expressions: &[Atom],
        parameters: &[Atom],
        definitions: &[(Atom, Atom)],
        options: Optimization,
        stop: ConstructionStop<'_>,
        limits: EvaluationLimits,
        occupied_entries: usize,
        source_limit: SourceOperationLimit,
    ) -> Result<&ScalarProduct, MathError> {
        let _phase = pse_diagnostics::observation::phase("math.scalar_prepare");
        stop.check()?;
        // Scalar optimization does not consume the later execution selector.
        // Share exact scalar preparation across interpreted/native Value routes.
        let query = ScalarProductQuery::new(expressions, parameters, options, definitions);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.scalar_cache_lookups", "count", 1);
        }
        let product: &Arc<ScalarProduct> = match self.evaluators.entry_ref(&query) {
            EntryRef::Occupied(entry) => {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("math.scalar_cache_reuses", "count", 1);
                }
                entry.into_mut()
            }
            EntryRef::Vacant(entry) => {
                let _construction = pse_diagnostics::observation::current().map(|context| {
                    context.count("math.scalar_constructions", "count", 1);
                    context.phase("math.scalar_construction")
                });
                let source = source_shape(
                    expressions
                        .iter()
                        .chain(definitions.iter().map(|(_, rhs)| rhs)),
                )?;
                let source_atom_operations = expressions
                    .iter()
                    .chain(definitions.iter().map(|(_, rhs)| rhs))
                    .try_fold(0, |sum, expression| {
                        add(sum, total(expression.count_operations())?)
                    })?;
                source_limit.check(source.operations, source_atom_operations)?;
                let atoms = expressions
                    .iter()
                    .chain(parameters)
                    .chain(definitions.iter().flat_map(|(lhs, rhs)| [lhs, rhs]))
                    .try_fold(0, |bytes, atom| add(bytes, atom.as_view().get_byte_size()))?;
                let key_bytes = add(
                    add(size_of::<ScalarProductKey>(), 2 * size_of::<usize>())?,
                    mul(
                        add(
                            add(expressions.len(), parameters.len())?,
                            mul(2, definitions.len())?,
                        )?,
                        size_of::<Atom>(),
                    )?,
                )?;
                // This owner retains the source key. Native input/alias copies and
                // algorithmic constructor growth are admitted by the supplier.
                let source_admission = add(atoms, key_bytes)?;
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("math.scalar_source_nodes", "count", source.nodes as u64);
                    context.count("math.scalar_source_bytes", "bytes", source.bytes as u64);
                    context.count(
                        "math.scalar_literal_bytes",
                        "bytes",
                        source.literal_bytes as u64,
                    );
                    context.count(
                        "math.scalar_source_admission_bytes",
                        "bytes",
                        source_admission as u64,
                    );
                }
                crate::construction::additional(source_admission)?;
                let mut control = SupplierControl::new(stop, limits, occupied_entries);
                let evaluator =
                    exact_evaluator(expressions, parameters, definitions, options, &mut control)?;
                let exact_bytes = control.output_bytes;
                let export = evaluator
                    .try_export_instructions_with(&mut control)
                    .map_err(supplier_error)?;
                let facts = EvaluatorFacts::inspect(
                    &export,
                    evaluator.count_operations(),
                    "math.rational_representation_inspections",
                )?;
                let bytes = add(
                    size_of::<ScalarProduct>(),
                    add(exact_bytes, add(atoms, key_bytes)?)?,
                )?;
                stop.check()?;
                let product = entry.insert_with_key(
                    symbolica::domains::backend::integer::IntegerAllocationContext::fresh()
                        .run(|| query.owned_key()),
                    Arc::new(ScalarProduct {
                        evaluator,
                        bytes,
                        facts,
                        source,
                        source_atom_operations,
                        owner: std::sync::OnceLock::new(),
                    }),
                );
                self.builds += 1;
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("math.scalar_constructions_completed", "count", 1);
                }
                product
            }
        };
        source_limit.check(product.source.operations, product.source_atom_operations)?;
        Ok(product.as_ref())
    }
}

/// Scalar-only provider composition: repeated identical lifts share exact
/// preparation, while every mutable numeric/order product remains private.
pub(crate) fn bounded_scalar_evaluator<'a>(
    expressions: &[Atom],
    parameters: &[Atom],
    options: Optimization,
    stop: impl Into<ConstructionStop<'a>>,
    limits: EvaluationLimits,
    scalars: &mut ScalarCache,
) -> Result<FloatingEvaluator, MathError> {
    let stop = stop.into();
    stop.check()?;
    bounded_defined_scalar_evaluator(expressions, parameters, &[], options, stop, limits, scalars)
}

pub(crate) fn bounded_defined_scalar_evaluator<'a>(
    expressions: &[Atom],
    parameters: &[Atom],
    definitions: &[(Atom, Atom)],
    options: Optimization,
    stop: impl Into<ConstructionStop<'a>>,
    limits: EvaluationLimits,
    scalars: &mut ScalarCache,
) -> Result<FloatingEvaluator, MathError> {
    let stop = stop.into();
    stop.check()?;
    let product = scalars.prepare(
        expressions,
        parameters,
        definitions,
        options,
        stop,
        limits,
        0,
        SourceOperationLimit::Scalar(limits.operations),
    )?;
    if total(product.facts.operations)? > limits.operations {
        return Err(MathError::Limit("scalar evaluator operations"));
    }
    limits.allocation(product.facts.storage.numeric_entries)?;
    FloatingEvaluator::from_exact(&product.evaluator, stop, limits, 0)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Distinct compilation contexts and remaining body allowances"
)]
pub(crate) fn bounded_evaluator<'a>(
    source_id: pse_ids::SemanticId,
    expressions: &[Atom],
    parameters: &[Atom],
    layout: &JetLayout,
    zero_components: &[(usize, usize)],
    options: Optimization,
    stop: impl Into<ConstructionStop<'a>>,
    limits: EvaluationLimits,
    remaining: usize,
    occupied_entries: usize,
    scalars: &mut ScalarCache,
) -> Result<FloatingEvaluator, MathError> {
    let stop = stop.into();
    stop.check()?;
    if zero_components.iter().any(|&(parameter, component)| {
        parameter >= parameters.len() || component == 0 || component >= layout.width()
    }) {
        return Err(MathError::Contract(
            "invalid structural Taylor zero component".into(),
        ));
    }
    let product = scalars.prepare(
        expressions,
        parameters,
        &[],
        options,
        stop,
        limits,
        occupied_entries,
        SourceOperationLimit::Taylor(remaining),
    )?;
    let evaluator = &product.evaluator;
    let scalar = product.facts.operations;
    if total(scalar)? > remaining {
        return Err(MathError::Limit("scalar evaluator operations"));
    }
    limits.allocation(add(
        occupied_entries,
        product.facts.storage.numeric_entries,
    )?)?;
    if layout.width() == 1 {
        return FloatingEvaluator::from_exact(evaluator, stop, limits, occupied_entries);
    }
    if !product.facts.taylor_supported {
        return Err(MathError::Contract("unsupported Taylor primitive".into()));
    }
    let mut admission = SupplierControl::new(stop, limits, occupied_entries);
    let heads = crate::stable::known_heads()?;
    let evaluator = evaluator
        .try_vectorize_shape_known_with(
            &layout.shape,
            zero_components,
            heads,
            &mut |extent: symbolica::evaluate::OperationExtent| {
                stop.check()?;
                // Native shape construction reports pair/coordinate scans;
                // later stages report emitted Taylor scalar arithmetic.
                if extent.stage != symbolica::evaluate::OperationStage::Construction
                    && extent.work > remaining
                {
                    return Err(MathError::WorkLimit {
                        source_id,
                        resource: "derivative expansion",
                        required: extent.work,
                        available: remaining,
                        components: layout.width(),
                    });
                }
                limits.allocation(add(occupied_entries, extent.output_entries)?)?;
                admission.check_extent(extent)
            },
        )
        .map_err(|error| match error {
            symbolica::evaluate::OperationError::Control(error) => error,
            symbolica::evaluate::OperationError::Overflow => {
                MathError::Limit("supplier construction extent")
            }
            symbolica::evaluate::OperationError::Allocation => {
                MathError::Limit("supplier construction allocation")
            }
            symbolica::evaluate::OperationError::Unsupported(profile) => {
                MathError::Contract(format!("unsupported checked Taylor profile: {profile}"))
            }
            symbolica::evaluate::OperationError::InvalidProgram(message) => {
                MathError::Library(message)
            }
            symbolica::evaluate::OperationError::Interrupted => MathError::Cancelled,
        })?;
    if total(evaluator.count_operations())? > remaining {
        return Err(MathError::Contract(
            "library Taylor construction exceeded admitted work".into(),
        ));
    }
    FloatingEvaluator::from_exact(&evaluator, stop, limits, occupied_entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_shared_alias_dag_fits_finite_owner_and_refuses_one_byte_short_without_publication() {
        use crate::construction::ConstructionAdmission;
        const WIDTH: usize = 128;
        const LAYERS: usize = 20;
        const CAPACITY: usize = 512 << 20;
        const REFUSAL: &str = "large shared DAG construction owner full";
        #[derive(Debug, Default)]
        struct Usage {
            used: usize,
            required: usize,
            calls: usize,
            refusals: usize,
        }
        #[derive(Debug)]
        struct FiniteOwner {
            capacity: usize,
            usage: std::sync::Mutex<Usage>,
        }
        impl FiniteOwner {
            fn new(capacity: usize) -> Self {
                Self {
                    capacity,
                    usage: Default::default(),
                }
            }
        }
        impl ConstructionAdmission for FiniteOwner {
            fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
                let mut usage = self.usage.lock().unwrap();
                usage.calls += 1;
                usage.required = usage
                    .used
                    .checked_add(bytes)
                    .ok_or(MathError::Limit(REFUSAL))?;
                if usage.required > self.capacity {
                    usage.refusals += 1;
                    return Err(MathError::Limit(REFUSAL));
                }
                usage.used = usage.required;
                Ok(())
            }
        }

        crate::initialize().unwrap();
        let parameters = (0..WIDTH)
            .map(formal)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let mut previous = parameters.clone();
        let mut definitions = Vec::with_capacity(WIDTH * LAYERS);
        for _ in 0..LAYERS {
            let mut next = Vec::with_capacity(WIDTH);
            for column in 0..WIDTH {
                let alias = formal(WIDTH + definitions.len()).unwrap();
                let value = (&previous[column] + &previous[(column + 1) % WIDTH]) / Atom::num(2);
                definitions.push((alias.clone(), value));
                next.push(alias);
            }
            previous = next;
        }
        // The mean keeps every shared node live; the endpoints distinguish the
        // local cyclic binomial convolution from a shortcut to the global mean.
        let mean =
            previous.iter().fold(Atom::num(0), |sum, value| sum + value) / Atom::num(WIDTH as u64);
        let expressions = [previous[0].clone(), previous[WIDTH - 1].clone(), mean];
        let options = Optimization::default();
        let limits = EvaluationLimits::default();
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let owner = Arc::new(FiniteOwner::new(CAPACITY));
        let mut cache = ScalarCache::default();
        assert_eq!(cache.len(), 0);
        crate::construction::scoped(owner.clone(), || {
            cache.admit_prepare(
                &expressions,
                &parameters,
                &definitions,
                options,
                &scope,
                limits,
            )
        })
        .unwrap_or_else(|error| {
            panic!("cold shared DAG construction failed: {error:?}; owner={owner:?}")
        });
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.builds(), 1);
        let product = cache.evaluators.values().next().unwrap();
        assert!(
            (5_000..=8_000).contains(&product.facts.storage.instruction_count),
            "actual shared native instruction count {}",
            product.facts.storage.instruction_count
        );
        let required = owner.usage.lock().unwrap().used;
        assert!((1..CAPACITY).contains(&required));
        assert_eq!(
            Arc::strong_count(&owner),
            1,
            "native products retain no construction control"
        );

        // This is the actual cold scalar-preparation boundary, not a warm-cache
        // acceptance substitute. Its observed finite admission defines the exact
        // adjacent refusal case while all optimizer and scientific limits stay fixed.
        let short = Arc::new(FiniteOwner::new(required - 1));
        let mut refused = ScalarCache::default();
        let result = crate::construction::scoped(short.clone(), || {
            refused.admit_prepare(
                &expressions,
                &parameters,
                &definitions,
                options,
                &scope,
                limits,
            )
        });
        assert!(matches!(result, Err(MathError::Limit(REFUSAL))));
        assert_eq!((refused.len(), refused.builds()), (0, 0));
        let usage = short.usage.lock().unwrap();
        assert_eq!(usage.refusals, 1);
        assert_eq!(
            usage.required,
            short.capacity + 1,
            "refusal is exactly one byte short"
        );
        assert!(
            usage.calls > 1,
            "the native operation follows source/key admission"
        );
        assert!(
            usage.used < usage.required,
            "refused growth was never committed"
        );
        drop(usage);
        assert_eq!(Arc::strong_count(&short), 1);

        // Ordinary numerical realization continues under the same finite owner
        // after the cold constructor passed; it does not renew that admission.
        let mut floating = crate::construction::scoped(owner.clone(), || {
            bounded_defined_scalar_evaluator(
                &expressions,
                &parameters,
                &definitions,
                options,
                &scope,
                limits,
                &mut cache,
            )
        })
        .unwrap();
        assert_eq!(cache.builds(), 1);
        for (offset, scale) in [(0_u64, 1_u64), (7, 3)] {
            let inputs: Vec<_> = (0..WIDTH)
                .map(|j| (offset + scale * (j * j) as u64) as f64)
                .collect();
            let endpoint = |column: usize| {
                let mut coefficient = 1_u64;
                let mut numerator = 0_u64;
                for shift in 0..=LAYERS {
                    let j = (column + shift) % WIDTH;
                    numerator += coefficient * (offset + scale * (j * j) as u64);
                    if shift < LAYERS {
                        coefficient = coefficient * (LAYERS - shift) as u64 / (shift + 1) as u64;
                    }
                }
                numerator as f64 / (1_u64 << LAYERS) as f64
            };
            let expected_mean =
                offset as f64 + scale as f64 * ((WIDTH - 1) * (2 * WIDTH - 1)) as f64 / 6.;
            let mut values = [0.; 3];
            floating
                .evaluator
                .try_evaluate(&inputs, &mut values)
                .unwrap();
            assert_eq!(values, [endpoint(0), endpoint(WIDTH - 1), expected_mean]);
        }
        assert_eq!(owner.usage.lock().unwrap().refusals, 0);

        let cancellation = pse_diagnostics::Cancellation::default();
        let expired =
            pse_kernels::ExecutionScope::new(cancellation.clone(), Some(std::time::Instant::now()));
        let stopped_owner = Arc::new(FiniteOwner::new(CAPACITY));
        let mut stopped = ScalarCache::default();
        let result = crate::construction::scoped(stopped_owner.clone(), || {
            stopped.admit_prepare(
                &expressions,
                &parameters,
                &definitions,
                options,
                &expired,
                limits,
            )
        });
        assert!(matches!(
            result,
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
        assert_eq!((stopped.len(), stopped.builds()), (0, 0));
        assert_eq!(stopped_owner.usage.lock().unwrap().calls, 0);
        assert!(
            !cancellation.is_cancelled(),
            "the original deadline is not rewritten as cancellation"
        );
    }

    #[test]
    fn checked_floating_constant_call_payload_is_in_auxiliary_and_worker_backing() {
        use crate::typed::{BodyBuilder, BodyLimits, TypedValue};
        use pse_quantity::{
            IndexSet, ResolvedPhysicalContract,
            standard::{StandardInvariantChecker, ids, standard_registry},
        };
        let context = crate::initialize().unwrap();
        let power = Rational::from(2).pow(4096);
        let denominator = &power + &Rational::from(1);
        let ratio = &power / &denominator;
        let argument = Atom::num(ratio);
        // Numerator and denominator each need at least 4096 bits. The tiny
        // f64 result does not erase these exact retained constant-call arguments.
        let payload_lower_bound = 2 * (4096 / 8);
        let expression =
            crate::stable::call(crate::stable::Operation::Log1p, &[&argument]).unwrap();
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let limits = EvaluationLimits::default();
        let mut floating = bounded_scalar_evaluator(
            &[expression],
            &[],
            Optimization::default(),
            &scope,
            limits,
            &mut ScalarCache::default(),
        )
        .unwrap();
        assert_eq!(floating.export.constant_functions.len(), 1);
        assert!(
            floating.facts.resettable,
            "a closed known constant remains pure"
        );
        floating.export.constant_functions[0]
            .tags
            .push("unqualified".into());
        assert!(
            !instruction_description(&floating.export)
                .unwrap()
                .resettable
        );
        floating.export.constant_functions[0].tags.clear();
        let argument_copy = floating.export.constant_functions[0]
            .fixed_args
            .pop()
            .unwrap();
        assert!(
            !instruction_description(&floating.export)
                .unwrap()
                .resettable
        );
        floating.export.constant_functions[0]
            .fixed_args
            .push(argument_copy);
        assert!(floating.facts.storage.auxiliary_bytes >= payload_lower_bound);
        assert!(floating.facts.storage.numeric_entries < payload_lower_bound / size_of::<f64>());
        let mut result = [0.];
        floating.evaluator.try_evaluate(&[], &mut result).unwrap();
        assert_eq!(result[0], std::f64::consts::LN_2);

        let registry = standard_registry().unwrap();
        let quantity =
            ResolvedPhysicalContract::named(ids::quantity("neutral"), IndexSet::new(), &registry)
                .unwrap();
        let source = pse_ids::SemanticId::from_bytes([42; 16]);
        let literal = TypedValue::symbolic_fixture(argument, quantity, source);
        let mut builder = BodyBuilder::new(
            context,
            &registry,
            &StandardInvariantChecker,
            0,
            BodyLimits::default(),
        )
        .unwrap();
        let output = builder
            .unary(crate::Function::Log1p, literal, source)
            .unwrap();
        let prepared = builder.prepare(&[output]).unwrap();
        let compiled = prepared
            .compile(
                &[0],
                &[],
                pse_kernels::DerivativeOrder::Value,
                Optimization::default(),
                limits,
                &scope,
            )
            .unwrap();
        let census = compiled
            .census()
            .find(|census| census.order == pse_kernels::DerivativeOrder::Value)
            .unwrap();
        assert!(census.instruction_storage_bytes >= payload_lower_bound);
        assert!(census.worker_bytes >= payload_lower_bound);
        assert!(census.frame_entries < payload_lower_bound / size_of::<f64>());
        let mut worker = compiled.worker();
        for _ in 0..2 {
            let result = worker
                .evaluate(
                    &[],
                    pse_kernels::DerivativeOrder::Value,
                    &mut pse_kernels::ProviderBindings::new(),
                    scope.cancellation(),
                )
                .unwrap();
            assert_eq!(result.values, [std::f64::consts::LN_2]);
        }
    }

    #[test]
    fn checked_taylor_metadata_work_is_distinct_from_emitted_scalar_work() {
        crate::initialize().unwrap();
        let limits = EvaluationLimits {
            operations: 10_000,
            ..Default::default()
        };
        let parameters = (0..64).map(formal).collect::<Result<Vec<_>, _>>().unwrap();
        let layout = JetLayout::new(
            (0..64).collect(),
            pse_kernels::DerivativeOrder::First,
            limits,
        )
        .unwrap();
        let native_scan = layout.width() * layout.width() * (3 * parameters.len() + 1);
        assert!(native_scan > limits.operations);
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let mut compiled = bounded_evaluator(
            pse_ids::SemanticId::NIL,
            &[parameters[0].clone() + 1],
            &parameters,
            &layout,
            &[],
            Optimization {
                horner_iterations: 0,
                ..Default::default()
            },
            &scope,
            limits,
            limits.operations,
            0,
            &mut ScalarCache::default(),
        )
        .unwrap();
        assert!(total(compiled.facts.operations).unwrap() < limits.operations);
        let mut input = vec![0.; parameters.len() * layout.width()];
        input[0] = 2.;
        input[1] = 1.;
        let mut output = vec![0.; layout.width()];
        compiled
            .evaluator
            .try_evaluate(&input, &mut output)
            .unwrap();
        assert_eq!(&output[..2], &[3., 1.]);
        assert!(output[2..].iter().all(|&value| value == 0.));
    }

    #[test]
    fn checked_scalar_constructor_uses_captured_owner_and_preserves_typed_refusal() {
        crate::initialize().unwrap();
        #[derive(Debug)]
        struct Original;
        impl crate::construction::ConstructionAdmission for Original {
            fn try_grow(&self, _: usize) -> Result<(), MathError> {
                Err(MathError::Limit("original supplier owner full"))
            }
        }
        let current =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let mut control = crate::construction::scoped(Arc::new(Original), || {
            SupplierControl::new((&current).into(), EvaluationLimits::default(), 0)
        });
        let unrelated = Arc::new(Grow(std::sync::atomic::AtomicUsize::new(0)));
        let x = formal(0).unwrap();
        let result = crate::construction::scoped(unrelated.clone(), || {
            exact_evaluator(
                &[(&x + 1).pow(12)],
                &[x],
                &[],
                Optimization {
                    cores: 2,
                    ..Default::default()
                },
                &mut control,
            )
        });
        assert!(matches!(
            result,
            Err(MathError::Limit("original supplier owner full"))
        ));
        assert_eq!(
            unrelated.0.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "the supplier operation must use its captured original owner, including native workers"
        );
    }

    #[test]
    fn scalar_cache_observes_current_scope_deadline_without_retaining_producer_stop() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let expressions = [x.clone() + Atom::num(1)];
        let parameters = [x];
        let producer = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(producer.clone(), None);
        let mut cache = ScalarCache::default();
        cache
            .admit_prepare(
                &expressions,
                &parameters,
                &[],
                Optimization::default(),
                &scope,
                EvaluationLimits::default(),
            )
            .unwrap();
        assert_eq!(cache.builds, 1);
        producer.cancel();
        let receiver =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        cache
            .admit_prepare(
                &expressions,
                &parameters,
                &[],
                Optimization::default(),
                &receiver,
                EvaluationLimits::default(),
            )
            .unwrap();
        assert_eq!(
            cache.builds, 1,
            "an old producer stop cannot poison an immutable exact product"
        );
        let current = pse_diagnostics::Cancellation::default();
        let expired =
            pse_kernels::ExecutionScope::new(current.clone(), Some(std::time::Instant::now()));
        let error = cache
            .admit_prepare(
                &expressions,
                &parameters,
                &[],
                Optimization::default(),
                &expired,
                EvaluationLimits::default(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            MathError::Scope(pse_kernels::ProviderError::Deadline)
        ));
        assert!(
            !current.is_cancelled(),
            "expiry must not mutate the original cancellation owner"
        );
        assert_eq!(cache.builds, 1);
    }

    #[test]
    fn checked_taylor_admits_before_growth_and_preserves_original_refusal() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let producer = Arc::new(AtomicBool::new(false));
        let source = exact_evaluator(
            &[x.clone().pow(2) + 1],
            &[x],
            &[],
            Optimization::default(),
            &mut SupplierControl::new((&producer).into(), EvaluationLimits::default(), 0),
        )
        .unwrap();
        let dual = symbolica::prelude::HyperDual::<Complex<Rational>>::new(vec![
            vec![0],
            vec![1],
            vec![2],
        ]);
        let dualizer = symbolica::prelude::Dualizer::new(dual, vec![]);
        let mut admitted = 0;
        let mut required = 0;
        let complete = source
            .try_vectorize_with(
                &dualizer,
                &mut |extent: symbolica::evaluate::OperationExtent| {
                    required = required.max(extent.peak_bytes().unwrap());
                    admitted += extent.work;
                    Ok::<_, MathError>(())
                },
            )
            .unwrap();
        assert!(admitted > 0);
        assert!(required > 0);
        assert!(total(complete.count_operations()).unwrap() <= admitted);
        let mut attempts = 0;
        let error = source.try_vectorize_with(
            &dualizer,
            &mut |extent: symbolica::evaluate::OperationExtent| {
                attempts += usize::from(extent.peak_bytes().unwrap() > 0);
                if extent.peak_bytes().unwrap() > required - 1 {
                    Err(MathError::Limit("original supplier admission"))
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(
            error,
            Err(symbolica::evaluate::OperationError::Control(
                MathError::Limit("original supplier admission")
            ))
        ));
        assert_eq!(
            attempts, 1,
            "refusal precedes cloning and every expansion stage"
        );
        let complete = source
            .try_vectorize_with(
                &dualizer,
                &mut |extent: symbolica::evaluate::OperationExtent| {
                    assert!(extent.peak_bytes().unwrap() <= required);
                    Ok::<_, MathError>(())
                },
            )
            .unwrap();
        let mut evaluator = complete.map_coeff(&|c| c.re.to_f64());
        let mut output = [0.; 3];
        evaluator.try_evaluate(&[3., 1., 0.], &mut output).unwrap();
        assert_eq!(output, [10., 6., 1.]);
    }

    #[test]
    fn checked_taylor_stop_between_stages_and_nested_controls_are_private() {
        use symbolica::evaluate::{OperationError, OperationExtent, OperationStage};
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let producer = Arc::new(AtomicBool::new(false));
        let source = exact_evaluator(
            &[x.clone().pow(2)],
            &[x],
            &[],
            Optimization::default(),
            &mut SupplierControl::new((&producer).into(), EvaluationLimits::default(), 0),
        )
        .unwrap();
        producer.store(true, std::sync::atomic::Ordering::Release);
        let dual = symbolica::prelude::HyperDual::<Complex<Rational>>::new(vec![
            vec![0],
            vec![1],
            vec![2],
        ]);
        let dualizer = symbolica::prelude::Dualizer::new(dual, vec![(0, 2)]);
        let mut nested = false;
        let result = source.try_vectorize_with(&dualizer, &mut |extent: OperationExtent| {
            if !nested {
                nested = true;
                let child = source.try_vectorize_with(&dualizer, &mut |_| Err("child stop"));
                assert!(matches!(child, Err(OperationError::Control("child stop"))));
            }
            if extent.stage == OperationStage::Optimization {
                Err("current stop")
            } else {
                Ok(())
            }
        });
        assert!(matches!(
            result,
            Err(OperationError::Control("current stop"))
        ));
        let complete = source
            .try_vectorize_with(&dualizer, &mut |_| Ok::<_, &'static str>(()))
            .unwrap();
        let mut evaluator = complete.map_coeff(&|c| c.re.to_f64());
        let mut output = [0.; 3];
        evaluator.try_evaluate(&[4., 1., 0.], &mut output).unwrap();
        assert_eq!(output, [16., 8., 1.]);
        assert_eq!(
            Arc::strong_count(&producer),
            1,
            "retained source and transformed products do not retain producer scope"
        );
    }

    #[test]
    fn checked_taylor_extent_overflow_refuses_before_control_or_growth() {
        struct Overflow;
        impl symbolica::domains::dual::DualNumberStructure for Overflow {
            fn get_len(&self) -> usize {
                usize::MAX
            }
            fn get_shape(&self) -> Vec<&[usize]> {
                vec![]
            }
            fn get_multiplication_table(&self) -> &[(usize, usize, usize)] {
                &[]
            }
            fn get_max_depth(&self) -> usize {
                2
            }
        }
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let source = exact_evaluator(
            &[x.clone().pow(2)],
            &[x],
            &[],
            Optimization::default(),
            &mut SupplierControl::new(
                (&Arc::new(AtomicBool::new(false))).into(),
                EvaluationLimits::default(),
                0,
            ),
        )
        .unwrap();
        let mut calls = 0;
        let result = source.try_vectorize_with(
            &symbolica::prelude::Dualizer::new(Overflow, vec![]),
            &mut |_| {
                calls += 1;
                Ok::<_, MathError>(())
            },
        );
        assert!(matches!(
            result,
            Err(symbolica::evaluate::OperationError::Overflow)
        ));
        assert_eq!(calls, 1, "only the allocation-free initial stop check ran");
    }

    #[test]
    fn source_encoding_and_literal_payload_do_not_become_numeric_slot_populations() {
        crate::initialize().unwrap();
        let small = formal(0).unwrap();
        let wide = formal(FORMAL_CHUNK).unwrap();
        let entries = |parameter: Atom| {
            bounded_scalar_evaluator(
                std::slice::from_ref(&parameter),
                std::slice::from_ref(&parameter),
                Optimization::default(),
                &Arc::new(AtomicBool::new(false)),
                EvaluationLimits::default(),
                &mut ScalarCache::default(),
            )
            .unwrap()
            .facts
            .storage
            .numeric_entries
        };
        assert_eq!(
            entries(small),
            entries(wide),
            "formal encoding width does not add numeric slots"
        );
        let small = Atom::num(1);
        let large = symbolica::parse!(
            "123456789012345678901234567890123456789/100000000000000000000000000000000000003"
        );
        let small_shape = source_shape([&small]).unwrap();
        let large_shape = source_shape([&large]).unwrap();
        assert!(large_shape.bytes > small_shape.bytes);
        assert!(large_shape.literal_bytes > small_shape.literal_bytes);
        let prepare = |literal: Atom| {
            let reports = Arc::new(Grow(std::sync::atomic::AtomicUsize::new(0)));
            let prepared = crate::construction::scoped(reports.clone(), || {
                bounded_scalar_evaluator(
                    &[literal],
                    &[],
                    Optimization::default(),
                    &Arc::new(AtomicBool::new(false)),
                    EvaluationLimits::default(),
                    &mut ScalarCache::default(),
                )
                .unwrap()
            });
            (
                prepared.facts.storage.numeric_entries,
                reports.0.load(std::sync::atomic::Ordering::SeqCst),
            )
        };
        let small = prepare(small);
        let large = prepare(large);
        assert_eq!(
            small.0, large.0,
            "literal digits are exact payload, not f64 slots"
        );
        assert!(
            large.1 > small.1,
            "actual native admission includes larger exact payload"
        );
    }

    #[test]
    fn source_census_includes_nested_builtin_operands_and_every_alias_definition() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let a = formal(1).unwrap();
        let b = formal(2).unwrap();
        let sine = symbolica::atom::FunctionBuilder::new(Symbol::SIN)
            .add_arg(&a)
            .finish();
        let exponential = symbolica::atom::FunctionBuilder::new(Symbol::EXP)
            .add_arg(&a)
            .finish();
        let outputs = [b.clone()];
        let definitions = [(a, x.pow(2)), (b, sine + exponential)];
        let shape =
            source_shape(outputs.iter().chain(definitions.iter().map(|(_, rhs)| rhs))).unwrap();
        // Symbolica normalizes exp(a) to E^a: one additional leaf and
        // operand beyond the sin(a) function node. Both alias bodies count.
        assert_eq!(shape.nodes, 10);
        assert_eq!(shape.operands, 7);
        assert_eq!(shape.operations, 4);
        assert!(shape.literal_bytes > 0);
        let mut prepared = bounded_defined_scalar_evaluator(
            &outputs,
            &[x],
            &definitions,
            Optimization::default(),
            &Arc::new(AtomicBool::new(false)),
            EvaluationLimits::default(),
            &mut ScalarCache::default(),
        )
        .unwrap();
        let expected = 4_f64.sin() + 4_f64.exp();
        assert!((prepared.evaluator.evaluate_single(&[2.]) - expected).abs() < 1e-12);
    }

    #[test]
    fn scalar_definition_map_participates_in_exact_identity_and_refusal() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let a = formal(1).unwrap();
        let outputs = [a.clone()];
        let parameters = [x.clone()];
        let first = [(a.clone(), x.clone() * &x)];
        let second = [(a, x + 1)];
        let cancel = Arc::new(AtomicBool::new(false));
        let mut cache = ScalarCache::default();
        let mut one = bounded_defined_scalar_evaluator(
            &outputs,
            &parameters,
            &first,
            Optimization::default(),
            &cancel,
            EvaluationLimits::default(),
            &mut cache,
        )
        .unwrap();
        let mut two = bounded_defined_scalar_evaluator(
            &outputs,
            &parameters,
            &second,
            Optimization::default(),
            &cancel,
            EvaluationLimits::default(),
            &mut cache,
        )
        .unwrap();
        assert_eq!(
            cache.len(),
            2,
            "equal outputs with different definitions are distinct programs"
        );
        let mut value = [0.];
        one.evaluator.try_evaluate(&[3.], &mut value).unwrap();
        assert_eq!(value, [9.]);
        two.evaluator.try_evaluate(&[3.], &mut value).unwrap();
        assert_eq!(value, [4.]);
        bounded_defined_scalar_evaluator(
            &outputs,
            &parameters,
            &first,
            Optimization::default(),
            &cancel,
            EvaluationLimits::default(),
            &mut cache,
        )
        .unwrap();
        assert_eq!(cache.builds(), 2);
        let mut refused = ScalarCache::default();
        assert!(matches!(
            crate::construction::scoped(Arc::new(Refuse), || bounded_defined_scalar_evaluator(
                &outputs,
                &parameters,
                &first,
                Optimization::default(),
                &cancel,
                EvaluationLimits::default(),
                &mut refused
            )),
            Err(MathError::Limit("test pool full"))
        ));
        assert!(refused.freeze().is_empty());
    }

    fn scalar_count(context: &pse_diagnostics::observation::ObservationContext, name: &str) -> u64 {
        context
            .reader()
            .snapshot(std::time::Duration::from_secs(60))
            .metrics
            .iter()
            .find_map(|metric| match metric.cumulative {
                pse_diagnostics::observation::Value::Count(count) if metric.name == name => {
                    Some(count)
                }
                _ => None,
            })
            .unwrap_or(0)
    }

    #[test]
    fn scalar_borrowed_key_hash_and_equivalence_preserve_order_and_options() {
        use std::hash::BuildHasher;
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let y = formal(1).unwrap();
        let options = Optimization::default();
        let expressions = [x.clone() + &y, x.clone() * &y];
        let parameters = [x.clone(), y.clone()];
        let baseline = ScalarProductQuery::new(&expressions, &parameters, options, &[]).owned_key();
        let cases = [
            (expressions.to_vec(), parameters.to_vec(), options, true),
            (
                expressions.to_vec(),
                parameters.to_vec(),
                Optimization {
                    evaluator: Evaluator::CompiledScalar { module_bytes: 4096 },
                    ..options
                },
                true,
            ),
            (vec![], vec![], options, false),
            (
                vec![expressions[1].clone(), expressions[0].clone()],
                parameters.to_vec(),
                options,
                false,
            ),
            (expressions.to_vec(), vec![y, x], options, false),
            (
                expressions.to_vec(),
                parameters.to_vec(),
                Optimization {
                    cores: 2,
                    ..options
                },
                false,
            ),
            (
                expressions.to_vec(),
                parameters.to_vec(),
                Optimization {
                    horner_iterations: 0,
                    ..options
                },
                false,
            ),
            (
                expressions.to_vec(),
                parameters.to_vec(),
                Optimization {
                    cpe_iterations: 0,
                    ..options
                },
                false,
            ),
        ];
        for (expressions, parameters, options, equivalent) in cases {
            let query = ScalarProductQuery::new(&expressions, &parameters, options, &[]);
            let key = query.owned_key();
            assert!(query.equivalent(&key));
            assert_eq!(query.equivalent(&baseline), equivalent);
            for _ in 0..4 {
                let hasher = RandomState::new();
                assert_eq!(hasher.hash_one(&query), hasher.hash_one(&key));
            }
        }
    }

    #[test]
    fn scalar_borrowed_lookup_owns_keys_only_on_successful_misses() {
        crate::initialize().unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("scalar.borrowed_lookup");
        let context = operation.context();
        let _entered = context.enter();
        let x = formal(0).unwrap();
        let y = formal(1).unwrap();
        let expressions = [x.clone() * &x + &y];
        let parameters = [x.clone(), y.clone()];
        let options = Optimization::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut cache = ScalarCache::default();
        assert!(matches!(
            crate::construction::scoped(Arc::new(Refuse), || {
                cache.admit_prepare(
                    &expressions,
                    &parameters,
                    &[],
                    options,
                    &cancel,
                    EvaluationLimits::default(),
                )
            }),
            Err(MathError::Limit("test pool full"))
        ));
        assert_eq!(cache.len(), 0);
        assert_eq!(scalar_count(&context, "math.scalar_key_constructions"), 0);
        assert_eq!(
            scalar_count(&context, "math.scalar_key_vector_allocations"),
            0
        );
        assert_eq!(scalar_count(&context, "math.scalar_constructions"), 1);

        cache
            .admit_prepare(
                &expressions,
                &parameters,
                &[],
                options,
                &cancel,
                EvaluationLimits::default(),
            )
            .unwrap();
        let product = cache.evaluators.values().next().unwrap().clone();
        let retained = cache.freeze();
        let mut cache = ScalarCache::from_products(&retained);
        // Equal contents at different addresses and a different execution route
        // borrow the same exact preparation without retaining this attempt.
        let other_cancel = Arc::new(AtomicBool::new(false));
        cache
            .admit_prepare(
                expressions.as_ref(),
                parameters.as_ref(),
                &[],
                Optimization {
                    evaluator: Evaluator::CompiledScalar { module_bytes: 4096 },
                    ..options
                },
                &other_cancel,
                EvaluationLimits::default(),
            )
            .unwrap();
        assert_eq!(Arc::strong_count(&other_cancel), 1);
        assert!(Arc::ptr_eq(
            &product,
            cache.evaluators.values().next().unwrap()
        ));
        assert_eq!(cache.builds(), 0);
        assert_eq!(scalar_count(&context, "math.scalar_key_constructions"), 1);
        assert_eq!(
            scalar_count(&context, "math.scalar_key_vector_allocations"),
            2
        );
        assert_eq!(scalar_count(&context, "math.scalar_constructions"), 2);
        assert_eq!(scalar_count(&context, "math.scalar_cache_reuses"), 1);

        for limits in [
            EvaluationLimits {
                operations: 0,
                ..Default::default()
            },
            EvaluationLimits {
                scratch_bytes: 0,
                ..Default::default()
            },
        ] {
            assert!(matches!(
                cache.admit_prepare(&expressions, &parameters, &[], options, &cancel, limits),
                Err(MathError::Limit(_))
            ));
        }
        assert_eq!(cache.builds(), 0);
        assert_eq!(scalar_count(&context, "math.scalar_key_constructions"), 1);
        for (expressions, parameters, options) in [
            (vec![x.clone() * &x + &y + 1], parameters.to_vec(), options),
            (expressions.to_vec(), vec![y, x], options),
            (
                expressions.to_vec(),
                parameters.to_vec(),
                Optimization {
                    horner_iterations: 0,
                    ..options
                },
            ),
        ] {
            cache
                .admit_prepare(
                    &expressions,
                    &parameters,
                    &[],
                    options,
                    &cancel,
                    EvaluationLimits::default(),
                )
                .unwrap();
        }
        assert_eq!(cache.len(), 4);
        assert_eq!(cache.builds(), 3);
        assert_eq!(scalar_count(&context, "math.scalar_key_constructions"), 4);
        assert_eq!(
            scalar_count(&context, "math.scalar_key_vector_allocations"),
            8
        );
        assert_eq!(
            scalar_count(&context, "math.scalar_constructions_completed"),
            4
        );
        assert_eq!(retained.len(), 1);
    }

    #[test]
    fn scalar_borrowed_entry_failure_and_eviction_preserve_escaped_lease() {
        #[derive(Debug)]
        struct Owner;
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let expressions = [x.clone() * &x + 1];
        let parameters = [x];
        let cancel = Arc::new(AtomicBool::new(false));
        let options = Optimization::default();
        let mut cache = ScalarCache::default();
        assert!(matches!(
            cache.admit_prepare(
                &expressions,
                &parameters,
                &[],
                Optimization {
                    cores: 0,
                    ..options
                },
                &cancel,
                EvaluationLimits::default()
            ),
            Err(MathError::Contract(_))
        ));
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.builds(), 0);
        cache
            .admit_prepare(
                &expressions,
                &parameters,
                &[],
                options,
                &cancel,
                EvaluationLimits::default(),
            )
            .unwrap();
        let owner = Arc::new(Owner);
        let weak_owner = Arc::downgrade(&owner);
        let retained = cache.freeze().with_owner(owner.clone());
        let mut cache = ScalarCache::from_products(&retained);
        let query = ScalarProductQuery::new(&expressions, &parameters, options, &[]);
        let escaped = cache.evaluators.remove(&query).unwrap();
        assert_eq!(cache.len(), 0);
        drop(retained);
        drop(owner);
        assert!(weak_owner.upgrade().is_some());
        let mut evaluator = escaped.evaluator.clone().map_coeff(&|c| c.re.to_f64());
        assert_eq!(evaluator.evaluate_single(&[3.]), 10.);
        cache
            .admit_prepare(
                &expressions,
                &parameters,
                &[],
                options,
                &cancel,
                EvaluationLimits::default(),
            )
            .unwrap();
        assert_eq!(cache.builds(), 1);
        assert!(!Arc::ptr_eq(
            &escaped,
            cache.evaluators.get(&query).unwrap()
        ));
        drop(escaped);
        assert!(weak_owner.upgrade().is_none());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn retained_symbolica_subprogram_refuses_demanded_taylor_while_inlining_expands_it() {
        use symbolica::evaluate::{FunctionRegistrationOptions, InliningPolicy};
        use symbolica::{parse, symbol};
        crate::initialize().unwrap();
        let parameters = [parse!("p45_subprogram_x")];
        let expression = parse!("p45_subprogram_f(p45_subprogram_x)");
        let function = parse!("p45_subprogram_x^2+1");
        let layout = JetLayout::new(
            vec![0],
            pse_kernels::DerivativeOrder::Second,
            EvaluationLimits::default(),
        )
        .unwrap();
        for (policy, expected_coefficients) in [
            (InliningPolicy::Never, None),
            (InliningPolicy::Always, Some([5., 4., 1.])),
        ] {
            let evaluator = expression
                .evaluator(&parameters)
                .add_function_with_options(
                    symbol!("p45_subprogram_f"),
                    vec![symbol!("p45_subprogram_x")],
                    function.clone(),
                    FunctionRegistrationOptions::default().inlining(policy),
                )
                .unwrap()
                .build()
                .unwrap();
            let mut scalar = evaluator
                .clone()
                .map_coeff(&|coefficient| coefficient.re.to_f64());
            assert_eq!(
                scalar.evaluate_single(&[2.]),
                5.,
                "both policies compute the same scalar function"
            );
            let expanded = evaluator.vectorize(&symbolica::prelude::Dualizer::new(
                symbolica::prelude::HyperDual::<Complex<Rational>>::new(layout.shape.clone()),
                vec![],
            ));
            if let Some(expected) = expected_coefficients {
                let mut floating = expanded
                    .unwrap()
                    .map_coeff(&|coefficient| coefficient.re.to_f64());
                let mut output = [f64::NAN; 3];
                floating.evaluate(&[2., 1., 0.], &mut output);
                assert_eq!(
                    output, expected,
                    "the inlined route emits Second Taylor coefficients"
                );
            } else {
                let refusal = expanded.unwrap_err();
                assert!(
                    refusal.contains("Sub-evaluator") && refusal.contains("cannot be vectorized"),
                    "{refusal}"
                );
            }
        }
    }
    #[test]
    fn builtin_source_operations_and_taylor_final_operations_remain_checked() {
        crate::initialize().unwrap();
        let x = formal(0).unwrap();
        let expression = symbolica::atom::FunctionBuilder::new(Symbol::EXP)
            .add_arg(&x)
            .finish();
        assert_eq!(source_shape([&expression]).unwrap().operations, 1);
        let limits = EvaluationLimits::default();
        let layout = JetLayout::new(vec![0], pse_kernels::DerivativeOrder::Second, limits).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut compiled = bounded_evaluator(
            pse_ids::SemanticId::NIL,
            std::slice::from_ref(&expression),
            std::slice::from_ref(&x),
            &layout,
            &[],
            Optimization::default(),
            &cancel,
            limits,
            limits.operations,
            0,
            &mut ScalarCache::default(),
        )
        .unwrap();
        let mut output = [0.; 3];
        compiled
            .evaluator
            .try_evaluate(&[0., 1., 0.], &mut output)
            .unwrap();
        assert_eq!(output, [1., 1., 0.5]);
        let result = bounded_evaluator(
            pse_ids::SemanticId::NIL,
            &[expression],
            &[x],
            &layout,
            &[],
            Optimization::default(),
            &cancel,
            limits,
            1,
            0,
            &mut ScalarCache::default(),
        );
        assert!(matches!(
            result,
            Err(MathError::WorkLimit {
                source_id: pse_ids::SemanticId::NIL,
                ..
            })
        ));
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
                &[],
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
            EvaluationLimits {
                operations: 1,
                ..Default::default()
            },
            EvaluationLimits {
                scratch_bytes: 1,
                ..Default::default()
            },
        ] {
            assert!(matches!(
                bounded_scalar_evaluator(
                    &expressions,
                    std::slice::from_ref(&x),
                    Optimization::default(),
                    &cancel,
                    limits,
                    &mut scalars
                ),
                Err(MathError::Limit(_))
            ));
            assert_eq!(
                scalars.builds(),
                1,
                "a retained exact representation is checked under the new policy"
            );
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
