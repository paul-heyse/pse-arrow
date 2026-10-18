// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronous resolver-owned read projections. These are never caller dependency lists.
use crate::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Read {
    Entity(EntityKindId),
    Unit(UnitId),
    Kind(QuantityKindId),
    Basis(BasisId),
    Reference(ReferenceStateId),
    Quantity(QuantityTypeId),
    Conversion(ConversionId),
    Operation(OperationId),
    UnitSet(UnitSetId),
    Reduction(OperationId),
    Name(String),
    Symbol(String),
    EntityName(String),
    Key(QuantityTypeKey),
    Literal(DimensionVector, Option<ReferenceStateId>),
    Monomial(Vec<KindFactor>),
    Contract(Vec<KindFactor>, Vec<EntityKindId>),
    Opcode(Opcode),
    Between(QuantityTypeId, QuantityTypeId),
    Neutral,
    Entities,
    Units,
    Kinds,
    Bases,
    References,
    Quantities,
    Conversions,
    Operations,
    UnitSets,
    Reductions,
    Names,
    Whole,
}
#[derive(Debug)]
struct Capture {
    registry: usize,
    facts: usize,
    reads: BTreeSet<Read>,
    invariants: BTreeSet<InvariantId>,
    all_facts: bool,
    unsupported: bool,
    suppressed: usize,
}
thread_local! { static ACTIVE: RefCell<Vec<Capture>> = const { RefCell::new(Vec::new()) }; }
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.borrow_mut().pop();
        });
    }
}
/// Opaque complete values of actual resolver reads for one successful product.
/// It can only be issued by a synchronous capture with the actual immutable owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalPremises {
    reads: BTreeSet<Read>,
    invariants: BTreeSet<InvariantId>,
    all_facts: bool,
    registry: Arc<QuantityRegistry>,
    facts: Arc<PhysicalPreconditions>,
}
/// Capture actual reads. An unwind or failed check discards the partial closure.
/// A foreign registry/checker makes the product ineligible for narrowed reuse.
pub fn capture<T, E>(
    registry: &QuantityRegistry,
    facts: &PhysicalPreconditions,
    check: impl FnOnce() -> Result<T, E>,
) -> Result<(T, Option<PhysicalPremises>), E> {
    ACTIVE.with(|active| {
        active.borrow_mut().push(Capture {
            registry: std::ptr::from_ref(registry) as usize,
            facts: std::ptr::from_ref(facts) as usize,
            reads: BTreeSet::new(),
            invariants: BTreeSet::new(),
            all_facts: false,
            unsupported: false,
            suppressed: 0,
        })
    });
    let reset = Reset;
    let value = check()?;
    // Snapshot construction reads underlying maps directly, never recording accessors. It copies
    // only the observed rows and queried index outcomes, never the full context first.
    let state = ACTIVE.with(|active| active.borrow_mut().pop());
    std::mem::forget(reset);
    let Some(state) = state else {
        return Ok((value, None));
    };
    if state.unsupported {
        return Ok((value, None));
    }
    let projected = project(registry, &state.reads);
    let selected = selected_facts(facts, &state.invariants, state.all_facts);
    Ok((
        value,
        Some(PhysicalPremises {
            reads: state.reads,
            invariants: state.invariants,
            all_facts: state.all_facts,
            registry: Arc::new(projected),
            facts: Arc::new(selected),
        }),
    ))
}
/// Whether retained concrete admissions must be resolved again for this capture.
pub fn observing(registry: &QuantityRegistry, checker: &dyn infer::InvariantChecker) -> bool {
    observe_checker(registry, checker);
    ACTIVE.with(|active| {
        active.borrow().last().is_some_and(|capture| {
            capture.suppressed == 0
                && capture.registry == std::ptr::from_ref(registry) as usize
                && checker
                    .physical_premise_owner(infer::CheckerSeal)
                    .is_some_and(|owner| std::ptr::from_ref(owner) as usize == capture.facts)
        })
    })
}
struct Resume(usize);
impl Drop for Resume {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            for capture in active.borrow_mut().iter_mut().take(self.0) {
                capture.suppressed -= 1;
            }
        });
    }
}
/// Administrative framing reads are not resolution dependencies. This guard is
/// synchronous and unwind safe, including inside nested captures.
pub fn without_recording<T>(encode: impl FnOnce() -> T) -> T {
    let count = ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        for capture in active.iter_mut() {
            capture.suppressed += 1;
        }
        active.len()
    });
    let _resume = Resume(count);
    encode()
}
/// Mark a route whose complete dependencies are not issued by this interpreter.
pub fn unsupported() {
    ACTIVE.with(|active| {
        for capture in active
            .borrow_mut()
            .iter_mut()
            .filter(|capture| capture.suppressed == 0)
        {
            capture.unsupported = true;
        }
    });
}
pub(crate) fn observe(registry: &QuantityRegistry, read: Read) {
    ACTIVE.with(|active| {
        for capture in active
            .borrow_mut()
            .iter_mut()
            .filter(|capture| capture.suppressed == 0)
        {
            if capture.registry == std::ptr::from_ref(registry) as usize {
                capture.reads.insert(read.clone());
            } else {
                capture.unsupported = true;
            }
        }
    });
}
pub(crate) fn observe_checker(registry: &QuantityRegistry, checker: &dyn infer::InvariantChecker) {
    ACTIVE.with(|active| {
        for capture in active
            .borrow_mut()
            .iter_mut()
            .filter(|capture| capture.suppressed == 0)
        {
            if capture.registry != std::ptr::from_ref(registry) as usize
                || !checker
                    .physical_premise_owner(infer::CheckerSeal)
                    .is_some_and(|owner| std::ptr::from_ref(owner) as usize == capture.facts)
            {
                capture.unsupported = true;
            }
        }
    });
}
pub(crate) fn observe_facts(facts: &PhysicalPreconditions, invariant: Option<InvariantId>) {
    ACTIVE.with(|active| {
        for capture in active
            .borrow_mut()
            .iter_mut()
            .filter(|capture| capture.suppressed == 0)
        {
            if capture.facts != std::ptr::from_ref(facts) as usize {
                capture.unsupported = true;
            } else if let Some(invariant) = invariant {
                capture.invariants.insert(invariant);
            } else {
                capture.all_facts = true;
            }
        }
    });
}
fn selected_facts(
    facts: &PhysicalPreconditions,
    invariants: &BTreeSet<InvariantId>,
    all: bool,
) -> PhysicalPreconditions {
    PhysicalPreconditions {
        declarations: if all {
            facts.declarations.clone()
        } else {
            invariants
                .iter()
                .filter_map(|id| {
                    facts
                        .declarations
                        .binary_search_by_key(id, |fact| fact.id)
                        .ok()
                        .map(|position| facts.declarations[position].clone())
                })
                .collect()
        },
    }
}
impl PhysicalPremises {
    /// Revalidate every positive, absent and population lookup against new owners.
    pub fn matches(&self, registry: &QuantityRegistry, facts: &PhysicalPreconditions) -> bool {
        self.registry.as_ref() == &project(registry, &self.reads)
            && self.facts.as_ref() == &selected_facts(facts, &self.invariants, self.all_facts)
    }
    /// Import an already issued complete product into an enclosing capture only
    /// after revalidating it against that capture's actual immutable owners.
    pub fn observe(&self, registry: &QuantityRegistry, facts: &PhysicalPreconditions) -> bool {
        if !self.matches(registry, facts) {
            unsupported();
            return false;
        }
        for read in &self.reads {
            observe(registry, read.clone());
        }
        if self.all_facts {
            observe_facts(facts, None);
        } else {
            for invariant in &self.invariants {
                observe_facts(facts, Some(*invariant));
            }
        }
        true
    }
    /// Require that a repeated numerical construction issued no question outside
    /// its preflight closure. Missing coverage refuses retention rather than guessing.
    pub fn covers(&self, actual: &Self) -> bool {
        (self.reads.contains(&Read::Whole) || actual.reads.is_subset(&self.reads))
            && (self.all_facts
                || !actual.all_facts && actual.invariants.is_subset(&self.invariants))
    }
    /// Resolver-issued sparse context, valid only for repeating the captured operation.
    pub fn context(&self) -> (&QuantityRegistry, &PhysicalPreconditions) {
        (&self.registry, &self.facts)
    }
    /// Conservative complete retained extent, including sparse access paths.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.invariants
                    .len()
                    .saturating_mul(size_of::<InvariantId>() + 128),
            )
            .saturating_add(self.registry.allocation_extent())
            .saturating_add(self.facts.allocation_extent())
            .saturating_add(
                self.reads
                    .iter()
                    .map(|read| {
                        size_of::<Read>()
                            + 128
                            + match read {
                                Read::Name(s) | Read::Symbol(s) | Read::EntityName(s) => {
                                    s.capacity()
                                }
                                Read::Key(key) => key.shape.capacity() * size_of::<EntityKindId>(),
                                Read::Monomial(factors) => {
                                    factors.capacity() * size_of::<KindFactor>()
                                }
                                Read::Contract(factors, axes) => {
                                    factors.capacity() * size_of::<KindFactor>()
                                        + axes.capacity() * size_of::<EntityKindId>()
                                }
                                _ => 0,
                            }
                    })
                    .sum::<usize>(),
            )
    }
    /// Frame lookup questions as well as answers. The caller frames the sparse values
    /// with its inventory encoder; this method never enumerates a captured registry.
    pub fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.str("builtin-physical-preconditions-v1")
            .u64(self.reads.len() as u64);
        for read in &self.reads {
            macro_rules! row {
                ($tag:literal,$id:expr) => {{
                    hash.str($tag).id(&$id.as_id());
                }};
            }
            match read {
                Read::Entity(id) => row!("entity", id),
                Read::Unit(id) => row!("unit", id),
                Read::Kind(id) => row!("kind", id),
                Read::Basis(id) => row!("basis", id),
                Read::Reference(id) => row!("reference", id),
                Read::Quantity(id) => row!("quantity", id),
                Read::Conversion(id) => row!("conversion", id),
                Read::Operation(id) => row!("operation", id),
                Read::UnitSet(id) => row!("unit-set", id),
                Read::Reduction(id) => row!("reduction", id),
                Read::Name(name) => {
                    hash.str("name").str(name);
                }
                Read::Symbol(name) => {
                    hash.str("symbol").str(name);
                }
                Read::EntityName(name) => {
                    hash.str("entity-name").str(name);
                }
                Read::Opcode(opcode) => {
                    hash.str("opcode").str(opcode.as_str());
                }
                Read::Between(from, to) => {
                    hash.str("between").id(&from.as_id()).id(&to.as_id());
                }
                Read::Key(key) => {
                    hash.str("key")
                        .id(&key.kind.as_id())
                        .str(key.scale_kind.as_str());
                    for id in [
                        key.basis.map(BasisId::as_id),
                        key.reference_state.map(ReferenceStateId::as_id),
                        key.subject_kind.map(EntityKindId::as_id),
                    ] {
                        hash.bool(id.is_some());
                        if let Some(id) = id {
                            hash.id(&id);
                        }
                    }
                    hash.u64(key.shape.len() as u64);
                    for kind in &key.shape {
                        hash.id(&kind.as_id());
                    }
                }
                Read::Monomial(factors) => {
                    hash.str("monomial").u64(factors.len() as u64);
                    for factor in factors {
                        hash.id(&factor.kind.as_id())
                            .part(&factor.exponent.num().to_le_bytes())
                            .part(&factor.exponent.den().to_le_bytes());
                    }
                }
                Read::Contract(factors, axes) => {
                    hash.str("contract-candidates").u64(factors.len() as u64);
                    for factor in factors {
                        hash.id(&factor.kind.as_id())
                            .part(&factor.exponent.num().to_le_bytes())
                            .part(&factor.exponent.den().to_le_bytes());
                    }
                    hash.u64(axes.len() as u64);
                    for axis in axes {
                        hash.id(&axis.as_id());
                    }
                }
                Read::Literal(dimension, reference) => {
                    hash.str("literal-candidates")
                        .part(&dimension.canonical_bytes())
                        .bool(reference.is_some());
                    if let Some(reference) = reference {
                        hash.id(&reference.as_id());
                    }
                }
                Read::Neutral => {
                    hash.str("neutral");
                }
                Read::Entities => {
                    hash.str("entities");
                }
                Read::Units => {
                    hash.str("units");
                }
                Read::Kinds => {
                    hash.str("kinds");
                }
                Read::Bases => {
                    hash.str("bases");
                }
                Read::References => {
                    hash.str("references");
                }
                Read::Quantities => {
                    hash.str("quantities");
                }
                Read::Conversions => {
                    hash.str("conversions");
                }
                Read::Operations => {
                    hash.str("operations");
                }
                Read::UnitSets => {
                    hash.str("unit-sets");
                }
                Read::Reductions => {
                    hash.str("reductions");
                }
                Read::Names => {
                    hash.str("names");
                }
                Read::Whole => {
                    hash.str("whole");
                }
            }
            // Answers are indexed by the exact question: the union of positive
            // declarations alone does not encode a negative or membership result.
            let ids: Vec<pse_ids::SemanticId> = match read {
                Read::Name(name) => self
                    .registry
                    .by_name
                    .get(name)
                    .map(|value| match value {
                        PhysicalName::QuantityType(id) => id.as_id(),
                        PhysicalName::ReferenceState(id) => id.as_id(),
                    })
                    .into_iter()
                    .collect(),
                Read::Symbol(symbol) => self
                    .registry
                    .by_symbol
                    .get(symbol)
                    .map(|id| id.as_id())
                    .into_iter()
                    .collect(),
                Read::Key(key) => self
                    .registry
                    .by_key
                    .get(key)
                    .map(|id| id.as_id())
                    .into_iter()
                    .collect(),
                Read::Monomial(factors) => self
                    .registry
                    .by_monomial
                    .get(factors)
                    .map(|id| id.as_id())
                    .into_iter()
                    .collect(),
                Read::Literal(dimension, reference) => self
                    .registry
                    .by_literal
                    .get(&(*dimension, *reference))
                    .into_iter()
                    .flatten()
                    .map(|id| id.as_id())
                    .collect(),
                Read::Contract(factors, axes) => self
                    .registry
                    .by_contract
                    .get(&(factors.clone(), axes.clone()))
                    .into_iter()
                    .flatten()
                    .map(|id| id.as_id())
                    .collect(),
                Read::Opcode(opcode) => self
                    .registry
                    .by_opcode
                    .get(opcode)
                    .into_iter()
                    .flatten()
                    .map(|id| id.as_id())
                    .collect(),
                Read::Between(from, to) => self
                    .registry
                    .by_conversion
                    .get(&(*from, *to))
                    .into_iter()
                    .flatten()
                    .map(|id| id.as_id())
                    .collect(),
                _ => Vec::new(),
            };
            hash.u64(ids.len() as u64);
            for id in ids {
                hash.id(&id);
            }
        }
        hash.bool(self.all_facts).u64(self.invariants.len() as u64);
        for invariant in &self.invariants {
            hash.id(&invariant.as_id());
        }
        // The complete inventory identity predates entity name lookup. Preserve that
        // identity and include these consumed spelling values in the new frame only.
        for entity in self.registry.entity_kinds.values() {
            hash.id(&entity.id.as_id()).str(&entity.name);
        }
    }
}
fn project(registry: &QuantityRegistry, reads: &BTreeSet<Read>) -> QuantityRegistry {
    macro_rules! selected {
        ($field:ident,$variant:ident,$all:ident) => {
            if reads.contains(&Read::$all) {
                registry.$field.clone()
            } else {
                reads
                    .iter()
                    .filter_map(|read| match read {
                        Read::$variant(id) => {
                            registry.$field.get(id).map(|value| (*id, value.clone()))
                        }
                        _ => None,
                    })
                    .collect::<BTreeMap<_, _>>()
            }
        };
    }
    if reads.contains(&Read::Whole) {
        return registry.clone();
    }
    let mut result = QuantityRegistry {
        entity_kinds: selected!(entity_kinds, Entity, Entities),
        units: selected!(units, Unit, Units),
        kinds: selected!(kinds, Kind, Kinds),
        bases: selected!(bases, Basis, Bases),
        reference_states: selected!(reference_states, Reference, References),
        quantity_types: selected!(quantity_types, Quantity, Quantities),
        conversions: selected!(conversions, Conversion, Conversions),
        operations: selected!(operations, Operation, Operations),
        unit_sets: selected!(unit_sets, UnitSet, UnitSets),
        reduction_domains: selected!(reduction_domains, Reduction, Reductions),
        neutral: None,
        by_key: BTreeMap::new(),
        by_symbol: BTreeMap::new(),
        by_opcode: BTreeMap::new(),
        by_conversion: BTreeMap::new(),
        by_monomial: BTreeMap::new(),
        by_name: BTreeMap::new(),
        by_contract: BTreeMap::new(),
        by_entity_name: BTreeMap::new(),
        by_literal: BTreeMap::new(),
    };
    for read in reads {
        match read {
            Read::Whole => return registry.clone(),
            Read::Neutral => result.neutral = registry.neutral,
            Read::EntityName(name) => {
                if let Some(ids) = registry.by_entity_name.get(name) {
                    result.by_entity_name.insert(name.clone(), ids.clone());
                    for id in ids {
                        if let Some(value) = registry.entity_kinds.get(id) {
                            result.entity_kinds.insert(*id, value.clone());
                        }
                    }
                }
            }
            Read::Name(name) => {
                if let Some(value) = registry.by_name.get(name) {
                    result.by_name.insert(name.clone(), *value);
                    match value {
                        PhysicalName::QuantityType(id) => {
                            if let Some(v) = registry.quantity_types.get(id) {
                                result.quantity_types.insert(*id, v.clone());
                            }
                        }
                        PhysicalName::ReferenceState(id) => {
                            if let Some(v) = registry.reference_states.get(id) {
                                result.reference_states.insert(*id, v.clone());
                            }
                        }
                    }
                }
            }
            Read::Symbol(symbol) => {
                if let Some(id) = registry.by_symbol.get(symbol) {
                    result.by_symbol.insert(symbol.clone(), *id);
                    if let Some(v) = registry.units.get(id) {
                        result.units.insert(*id, v.clone());
                    }
                }
            }
            Read::Key(key) => {
                if let Some(id) = registry.by_key.get(key) {
                    result.by_key.insert(key.clone(), *id);
                    if let Some(v) = registry.quantity_types.get(id) {
                        result.quantity_types.insert(*id, v.clone());
                    }
                }
            }
            Read::Monomial(factors) => {
                let id = match factors.as_slice() {
                    [only]
                        if only.exponent == Ratio::ONE
                            && registry
                                .kinds
                                .get(&only.kind)
                                .is_some_and(|v| v.definition.is_none()) =>
                    {
                        Some(only.kind)
                    }
                    _ => registry.by_monomial.get(factors).copied(),
                };
                if let Some(id) = id {
                    result.by_monomial.insert(factors.clone(), id);
                    if let Some(v) = registry.kinds.get(&id) {
                        result.kinds.insert(id, v.clone());
                    }
                }
            }
            Read::Opcode(opcode) => {
                if let Some(ids) = registry.by_opcode.get(opcode) {
                    result.by_opcode.insert(*opcode, ids.clone());
                    for id in ids {
                        if let Some(v) = registry.operations.get(id) {
                            result.operations.insert(*id, v.clone());
                        }
                    }
                }
            }
            Read::Between(from, to) => {
                if let Some(ids) = registry.by_conversion.get(&(*from, *to)) {
                    result.by_conversion.insert((*from, *to), ids.clone());
                    for id in ids {
                        if let Some(v) = registry.conversions.get(id) {
                            result.conversions.insert(*id, v.clone());
                        }
                    }
                }
            }
            Read::Contract(factors, axes) => {
                if let Some(ids) = registry.by_contract.get(&(factors.clone(), axes.clone())) {
                    result
                        .by_contract
                        .insert((factors.clone(), axes.clone()), ids.clone());
                    for id in ids {
                        if let Some(value) = registry.quantity_types.get(id) {
                            result.quantity_types.insert(*id, value.clone());
                        }
                    }
                }
            }
            Read::Literal(dimension, reference) => {
                if let Some(ids) = registry.by_literal.get(&(*dimension, *reference)) {
                    result
                        .by_literal
                        .insert((*dimension, *reference), ids.clone());
                    for id in ids {
                        if let Some(value) = registry.quantity_types.get(id) {
                            result.quantity_types.insert(*id, value.clone());
                        }
                    }
                }
            }
            Read::Names => {
                result.by_name = registry.by_name.clone();
                result.quantity_types.extend(
                    registry
                        .quantity_types
                        .iter()
                        .filter(|(_, v)| v.name.is_some())
                        .map(|(id, v)| (*id, v.clone())),
                );
                result
                    .reference_states
                    .extend(registry.reference_states.clone());
            }
            _ => {}
        }
    }
    result
}

#[cfg(all(test, feature = "fixtures"))]
mod tests {
    use super::*;
    use crate::standard::{StandardInvariantChecker, standard_registry};
    fn temperature(registry: &QuantityRegistry) -> QuantityTypeId {
        match registry.physical_name("Temperature").unwrap() {
            PhysicalName::QuantityType(id) => id,
            _ => panic!("quantity"),
        }
    }
    fn facts() -> PhysicalPreconditions {
        PhysicalPreconditions::new(generated::standard_preconditions()).unwrap()
    }
    fn entity(registry: &QuantityRegistry, byte: u8, name: &str) -> QuantityRegistry {
        let mut builder = registry.to_builder();
        builder.entity_kind(EntityKind {
            id: EntityKindId::from_id(pse_ids::SemanticId::from_bytes([byte; 16])),
            name: name.into(),
        });
        builder.build().unwrap()
    }
    #[test]
    fn consumed_direct_transitive_and_absent_reads_preserve_unrelated_declarations() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let (_, premises) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            let ty = registry.quantity_type(temperature(&registry))?;
            registry.unit(ty.canonical_unit)?;
            assert!(registry.physical_name("NeverDeclared").is_none());
            Ok(())
        })
        .unwrap();
        let premises = premises.unwrap();
        assert!(premises.matches(&entity(&registry, 250, "unrelated"), &facts));
        let mut builder = registry.to_builder();
        let mut changed = registry
            .quantity_type(temperature(&registry))
            .unwrap()
            .clone();
        changed.id = QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([249; 16]));
        changed.name = Some("NeverDeclared".into());
        changed
            .key
            .shape
            .push(EntityKindId::from_id(pse_ids::SemanticId::from_bytes(
                [248; 16],
            )));
        builder
            .entity_kind(EntityKind {
                id: changed.key.shape[0],
                name: "synthetic".into(),
            })
            .quantity_type(changed);
        assert!(!premises.matches(&builder.build().unwrap(), &facts));
        let mut changed = registry.clone();
        let unit = registry
            .quantity_type(temperature(&registry))
            .unwrap()
            .canonical_unit;
        changed.units.get_mut(&unit).unwrap().scale_to_canonical *= 2.0;
        assert!(!premises.matches(&changed, &facts));
    }
    #[test]
    fn complete_candidate_buckets_capture_ambiguity_before_short_circuit() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let (operation, premises) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            Ok(registry.operations_for(Opcode::Mul).next().unwrap().clone())
        })
        .unwrap();
        let mut second = operation;
        second.id = OperationId::from_id(pse_ids::SemanticId::from_bytes([247; 16]));
        let mut builder = registry.to_builder();
        builder.operation(second);
        assert!(!premises.unwrap().matches(&builder.build().unwrap(), &facts));
        let (_, premises) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            Ok(registry.entity_kind_named("new-kind"))
        })
        .unwrap();
        let first = entity(&registry, 246, "new-kind");
        let second = entity(&first, 245, "new-kind");
        assert!(!premises.unwrap().matches(&second, &facts));
    }
    #[test]
    fn negative_complete_key_and_composed_unit_registration_are_dependencies() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let mut key = registry
            .quantity_type(temperature(&registry))
            .unwrap()
            .key
            .clone();
        let axis = EntityKindId::from_id(pse_ids::SemanticId::from_bytes([244; 16]));
        key.shape.push(axis);
        let (_, premises) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            assert!(registry.resolve_key(&key).is_err());
            Ok(())
        })
        .unwrap();
        let mut builder = registry.to_builder();
        builder.entity_kind(EntityKind {
            id: axis,
            name: "axis".into(),
        });
        let mut ty = registry
            .quantity_type(temperature(&registry))
            .unwrap()
            .clone();
        ty.id = QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([243; 16]));
        ty.name = None;
        ty.key = key;
        builder.quantity_type(ty);
        assert!(!premises.unwrap().matches(&builder.build().unwrap(), &facts));
        let product = UnitProduct::from_factors([("m".into(), Ratio::new(7, 1).unwrap())]).unwrap();
        let (unit, premises) = capture(&registry, &facts, || registry.compose(&product)).unwrap();
        if registry.unit(unit.id).is_err() {
            let mut builder = registry.to_builder();
            builder.defined_unit(DefinedUnit {
                id: unit.id,
                symbol: "synthetic-square".into(),
                composition: unit.definition.unwrap(),
            });
            assert!(!premises.unwrap().matches(&builder.build().unwrap(), &facts));
        }
    }
    #[test]
    fn caught_failures_nested_owners_custom_checker_and_unwind_are_conservative() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let (_, outer) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            let inner = capture(&registry, &facts, || -> Result<(), QuantityError> {
                registry.quantity_type(QuantityTypeId::from_id(
                    pse_ids::SemanticId::from_bytes([242; 16]),
                ))?;
                Ok(())
            });
            assert!(inner.is_err());
            Ok(())
        })
        .unwrap();
        assert!(
            outer
                .unwrap()
                .reads
                .iter()
                .any(|r| matches!(r, Read::Quantity(_)))
        );
        let foreign = registry.clone();
        let (_, outer) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            let _ = capture(&foreign, &facts, || {
                foreign.quantity_type(temperature(&registry))
            })?;
            Ok(())
        })
        .unwrap();
        assert!(outer.is_none());
        let (_, custom) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            observe_checker(&registry, &StandardInvariantChecker);
            Ok(())
        })
        .unwrap();
        assert!(custom.is_none());
        let _ = std::panic::catch_unwind(|| {
            let _ = capture(&registry, &facts, || -> Result<(), QuantityError> {
                panic!("cancelled capture")
            });
        });
        let (_, clean) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            registry.quantity_type(temperature(&registry))?;
            Ok(())
        })
        .unwrap();
        assert!(clean.is_some());
    }
    #[test]
    fn indexed_literal_candidates_preserve_vectors_and_capture_new_scalar_candidate() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let scalar = registry.neutral_dimensionless().unwrap();
        let source = registry
            .unit(registry.quantity_type(scalar).unwrap().canonical_unit)
            .unwrap();
        let (_, premises) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            let _ = literal::resolve_literal(source, literal::LiteralContext::Free, &registry);
            Ok(())
        })
        .unwrap();
        let premises = premises.unwrap();
        let mut builder = registry.to_builder();
        let axis = EntityKindId::from_id(pse_ids::SemanticId::from_bytes([237; 16]));
        builder.entity_kind(EntityKind {
            id: axis,
            name: "literal-unrelated-axis".into(),
        });
        let mut ty = registry.quantity_type(scalar).unwrap().clone();
        ty.id = QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([238; 16]));
        ty.name = Some("unrelated_vector".into());
        ty.key.shape = vec![axis];
        builder.quantity_type(ty);
        let vector = builder.build().unwrap();
        assert!(premises.matches(&vector, &facts));
        let mut builder = registry.to_builder();
        let mut ty = registry
            .quantity_types()
            .find(|ty| ty.key.shape.is_empty() && ty.id != scalar)
            .unwrap()
            .clone();
        // A new basis gives a distinct complete scalar contract without inventing an admitted mutation.
        let basis = BasisId::from_id(pse_ids::SemanticId::from_bytes([239; 16]));
        let prototype = registry.bases().next().unwrap().clone();
        builder.basis(Basis {
            id: basis,
            ..prototype
        });
        ty.id = QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([240; 16]));
        ty.name = Some("new_literal_candidate".into());
        ty.key.basis = Some(basis);
        // Use the scalar dimension/representation, with a distinct admitted basis.
        let original = registry.quantity_type(scalar).unwrap();
        ty.key.kind = original.key.kind;
        ty.key.scale_kind = original.key.scale_kind;
        ty.key.reference_state = original.key.reference_state;
        ty.canonical_unit = original.canonical_unit;
        builder.quantity_type(ty);
        assert!(!premises.matches(&builder.build().unwrap(), &facts));
    }
    #[test]
    fn sparse_declaration_names_are_charged_independently_of_name_indexes() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let mut builder = registry.to_builder();
        let scalar = registry.neutral_dimensionless().unwrap();
        let axis = EntityKindId::from_id(pse_ids::SemanticId::from_bytes([233; 16]));
        builder.entity_kind(EntityKind {
            id: axis,
            name: "long-name-axis".into(),
        });
        let mut quantity = registry.quantity_type(scalar).unwrap().clone();
        quantity.id = QuantityTypeId::from_id(pse_ids::SemanticId::from_bytes([234; 16]));
        quantity.key.shape = vec![axis];
        quantity.name = Some("x".repeat(65536));
        let id = quantity.id;
        let quantity_bytes = quantity.name.as_ref().unwrap().len();
        builder.quantity_type(quantity);
        let registry = builder.build().unwrap();
        let (_, premises) = capture(&registry, &facts, || registry.quantity_type(id)).unwrap();
        let premises = premises.unwrap();
        assert!(premises.registry.by_name.is_empty());
        assert!(premises.retained_bytes() >= quantity_bytes);
        let mut reference = registry.reference_states().next().unwrap().clone();
        reference.name = "r".repeat(65536);
        reference.id = ReferenceStateId::from_id(pse_ids::SemanticId::from_bytes([235; 16]));
        let id = reference.id;
        let reference_bytes = reference.name.len();
        let mut builder = registry.to_builder();
        builder.reference_state(reference);
        let registry = builder.build().unwrap();
        let (_, premises) = capture(&registry, &facts, || registry.reference_state(id)).unwrap();
        let premises = premises.unwrap();
        assert!(premises.registry.by_name.is_empty());
        assert!(premises.retained_bytes() >= reference_bytes);
    }
    #[test]
    fn administrative_suspension_keeps_new_nested_audits_active() {
        let registry = standard_registry().unwrap();
        let facts = facts();
        let (_, outer) = capture(&registry, &facts, || -> Result<_, QuantityError> {
            without_recording(|| {
                let (_, inner) = capture(&registry, &facts, || {
                    registry.quantity_type(temperature(&registry))
                })
                .unwrap();
                assert!(!inner.unwrap().reads.is_empty());
            });
            Ok(())
        })
        .unwrap();
        assert!(outer.unwrap().reads.is_empty());
    }
}
