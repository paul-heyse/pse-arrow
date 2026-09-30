// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admission and immutable lookup of physical contracts (blueprint §6.2, §8).
//!
//! Registry construction checks the actual declarations and their references. No hash
//! participates in validity, type selection, or the proof that a conversion applies.
use crate::basis::Basis;
use crate::conversion::ConversionRule;
use crate::kind::{DerivedKind, KindDefinition, KindFactor, QuantityKind};
use crate::operation::QuantityOperation;
use crate::quantity_type::{QuantityType, QuantityTypeKey};
use crate::reference_state::ReferenceState;
use crate::unit::{DefinedUnit, Unit, UnitFactor, canonical_factors, unit_product_id};
use crate::unit_product::UnitProduct;
use crate::unit_set::UnitSet;
use crate::{
    BasisId, BasisKind, BasisRule, ConversionId, ConversionKind, DimensionVector, EntityKind,
    EntityKindId, Opcode, OperationId, QuantityAdditionKind, QuantityError, QuantityKindCategory,
    QuantityKindId, QuantityScaleRule, QuantityShapeRule, QuantityTypeId, ReferenceRule,
    ReferenceStateId, ScaleKind, SubjectRule, UnitId, UnitSetId,
};
use pse_ids::SemanticId;
use std::collections::{BTreeMap, BTreeSet};

/// Unadmitted declarations. Duplicate identities are retained until `build` rejects them.
#[derive(Clone, Debug, Default)]
pub struct QuantityRegistryBuilder {
    entity_kinds: Vec<EntityKind>,
    units: Vec<Unit>,
    defined_units: Vec<DefinedUnit>,
    kinds: Vec<QuantityKind>,
    derived_kinds: Vec<DerivedKind>,
    bases: Vec<Basis>,
    reference_states: Vec<ReferenceState>,
    quantity_types: Vec<QuantityType>,
    conversions: Vec<ConversionRule>,
    operations: Vec<QuantityOperation>,
    reduction_domains: Vec<(OperationId, EntityKindId)>,
    unit_sets: Vec<UnitSet>,
    neutral: Vec<QuantityTypeId>,
}
macro_rules! add_declaration {
    ($method:ident, $field:ident, $ty:ty) => {
        #[doc = concat!("Add a `", stringify!($ty), "` declaration; admission occurs in `build`.")]
        pub fn $method(&mut self, value: $ty) -> &mut Self {
            self.$field.push(value);
            self
        }
    };
}
impl QuantityRegistryBuilder {
    /// Start an empty declaration set.
    pub fn new() -> Self {
        Self::default()
    }
    add_declaration!(entity_kind, entity_kinds, EntityKind);
    add_declaration!(unit, units, Unit);
    add_declaration!(defined_unit, defined_units, DefinedUnit);
    add_declaration!(kind, kinds, QuantityKind);
    add_declaration!(derived_kind, derived_kinds, DerivedKind);
    add_declaration!(basis, bases, Basis);
    add_declaration!(reference_state, reference_states, ReferenceState);
    add_declaration!(quantity_type, quantity_types, QuantityType);
    add_declaration!(conversion, conversions, ConversionRule);
    add_declaration!(operation, operations, QuantityOperation);
    /// Declare the exact domain kind consumed by a registered index reduction.
    pub fn reduction_domain(&mut self, operation: OperationId, domain: EntityKindId) -> &mut Self {
        self.reduction_domains.push((operation, domain));
        self
    }
    add_declaration!(unit_set, unit_sets, UnitSet);
    /// Explicitly bind the package's neutral scalar type; dimension equality cannot pick it.
    pub fn neutral_dimensionless(&mut self, id: QuantityTypeId) -> &mut Self {
        self.neutral.push(id);
        self
    }
    /// Admit the complete registry atomically.
    ///
    /// # Errors
    /// Rejects duplicate identities/keys/symbols, unresolved references, invalid unit,
    /// type, basis, conversion and operation declarations, or an invalid neutral binding.
    pub fn build(self) -> Result<QuantityRegistry, QuantityError> {
        let neutral = match self.neutral.as_slice() {
            [] => None,
            [id] => Some(*id),
            _ => {
                return Err(error(
                    "quantity_type.neutral_singleton",
                    SemanticId::NIL,
                    "neutral scalar was designated more than once",
                ));
            }
        };
        let mut units = index(self.units, |x| x.id, "unit")?;
        for unit in units.values() {
            require(
                unit.definition.is_none(),
                "unit.atomic_declaration",
                unit.id.as_id(),
                "a unit with a composition is declared as a defined unit",
            )?;
        }
        for unit in admit_defined(&units, self.defined_units)? {
            units.insert(unit.id, unit);
        }
        let mut kinds = index(self.kinds, |x| x.id, "quantity_kind")?;
        for kind in kinds.values() {
            require(
                kind.definition.is_none(),
                "quantity_kind.base_declaration",
                kind.id.as_id(),
                "a kind with a monomial is declared as a derived kind",
            )?;
        }
        for kind in admit_derived_kinds(&kinds, self.derived_kinds)? {
            kinds.insert(kind.id, kind);
        }
        let mut registry = QuantityRegistry {
            entity_kinds: index(self.entity_kinds, |x| x.id, "entity_kind")?,
            units,
            kinds,
            bases: index(self.bases, |x| x.id, "basis")?,
            reference_states: index(self.reference_states, |x| x.id, "reference_state")?,
            quantity_types: index(self.quantity_types, |x| x.id, "quantity_type")?,
            conversions: index(self.conversions, |x| x.id, "conversion")?,
            operations: index(self.operations, |x| x.id, "operation")?,
            reduction_domains: index(
                self.reduction_domains,
                |x| x.0,
                "operation.reduction_domain",
            )?
            .into_iter()
            .map(|(id, (_, kind))| (id, kind))
            .collect(),
            unit_sets: index(self.unit_sets, |x| x.id, "unit_set")?,
            neutral,
            by_key: BTreeMap::new(),
            by_symbol: BTreeMap::new(),
            by_opcode: BTreeMap::new(),
            by_conversion: BTreeMap::new(),
            by_monomial: BTreeMap::new(),
            by_name: BTreeMap::new(),
        };
        registry.validate()?;
        registry.by_name = registry.names()?;
        registry.by_monomial = registry
            .kinds
            .values()
            .filter_map(|kind| Some((kind.definition.as_ref()?.monomial.clone(), kind.id)))
            .collect();
        registry.by_key = registry
            .quantity_types
            .values()
            .map(|v| (v.key.clone(), v.id))
            .collect();
        // Literal factors name atomic units only; a defined unit is spelled by its
        // composition (ADR-0124).
        registry.by_symbol = registry
            .units
            .values()
            .filter(|v| v.definition.is_none())
            .map(|v| (v.symbol.clone(), v.id))
            .collect();
        for v in registry.operations.values() {
            registry.by_opcode.entry(v.opcode).or_default().push(v.id);
        }
        for v in registry.conversions.values() {
            registry
                .by_conversion
                .entry((v.from, v.to))
                .or_default()
                .push(v.id);
        }
        Ok(registry)
    }
}
/// An immutable collection of admitted physical declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantityRegistry {
    entity_kinds: BTreeMap<EntityKindId, EntityKind>,
    units: BTreeMap<UnitId, Unit>,
    kinds: BTreeMap<QuantityKindId, QuantityKind>,
    bases: BTreeMap<BasisId, Basis>,
    reference_states: BTreeMap<ReferenceStateId, ReferenceState>,
    quantity_types: BTreeMap<QuantityTypeId, QuantityType>,
    conversions: BTreeMap<ConversionId, ConversionRule>,
    operations: BTreeMap<OperationId, QuantityOperation>,
    reduction_domains: BTreeMap<OperationId, EntityKindId>,
    unit_sets: BTreeMap<UnitSetId, UnitSet>,
    neutral: Option<QuantityTypeId>,
    by_key: BTreeMap<QuantityTypeKey, QuantityTypeId>,
    by_symbol: BTreeMap<String, UnitId>,
    by_opcode: BTreeMap<Opcode, Vec<OperationId>>,
    by_conversion: BTreeMap<(QuantityTypeId, QuantityTypeId), Vec<ConversionId>>,
    by_monomial: BTreeMap<Vec<KindFactor>, QuantityKindId>,
    by_name: BTreeMap<String, PhysicalName>,
}
/// What a physical name declared in the physical document denotes (ADR-0123 Outcome 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhysicalName {
    /// A named quantity type.
    QuantityType(QuantityTypeId),
    /// A named reference state.
    ReferenceState(ReferenceStateId),
}
macro_rules! lookup {
    ($method:ident, $field:ident, $id:ty, $ty:ty) => {
        #[doc = concat!("Resolve a declared `", stringify!($ty), "`.")]
        ///
        /// # Errors
        /// Rejects an identity absent from this admitted registry.
        pub fn $method(&self, id: $id) -> Result<&$ty, QuantityError> {
            self.$field.get(&id).ok_or(QuantityError::UnknownId {
                kind: stringify!($method),
                id: id.as_id(),
            })
        }
    };
}
impl QuantityRegistry {
    /// Conservative owned collection and string extent, excluding allocator overhead.
    /// Saturation makes an overflow exceed every finite admission allowance.
    pub fn allocation_extent(&self) -> usize {
        let mut bytes = [
            self.entity_kinds.len(),
            self.units.len(),
            self.kinds.len(),
            self.bases.len(),
            self.reference_states.len(),
            self.quantity_types.len(),
            self.conversions.len(),
            self.operations.len(),
            self.reduction_domains.len(),
            self.unit_sets.len(),
            self.by_key.len(),
            self.by_symbol.len(),
            self.by_opcode.len(),
            self.by_conversion.len(),
            self.by_monomial.len(),
            self.by_name.len(),
        ]
        .into_iter()
        .fold(size_of::<Self>(), |n, count| {
            n.saturating_add(count.saturating_mul(512))
        });
        for kind in self.entity_kinds.values() {
            bytes = bytes.saturating_add(kind.name.capacity());
        }
        for unit in self.units.values() {
            bytes = bytes.saturating_add(unit.symbol.capacity()).saturating_add(
                unit.definition
                    .as_ref()
                    .map_or(0, |d| d.capacity().saturating_mul(size_of::<UnitFactor>())),
            );
        }
        for symbol in self.by_symbol.keys() {
            bytes = bytes.saturating_add(symbol.capacity());
        }
        for name in self.by_name.keys() {
            bytes = bytes.saturating_add(name.capacity().saturating_mul(3));
        }
        for kind in self.kinds.values() {
            if let Some(definition) = &kind.definition {
                bytes = bytes.saturating_add(
                    definition
                        .monomial
                        .capacity()
                        .saturating_mul(2 * size_of::<KindFactor>()),
                );
            }
        }
        for quantity in self.quantity_types.values() {
            bytes = bytes.saturating_add(quantity.key.shape.capacity().saturating_mul(32));
        }
        for key in self.by_key.keys() {
            bytes = bytes.saturating_add(key.shape.capacity().saturating_mul(32));
        }
        for conversion in self.conversions.values() {
            bytes = bytes.saturating_add(
                conversion
                    .required_parameters
                    .capacity()
                    .saturating_mul(size_of::<String>()),
            );
            for name in &conversion.required_parameters {
                bytes = bytes.saturating_add(name.capacity());
            }
        }
        for operation in self.operations.values() {
            bytes = bytes
                .saturating_add(operation.input_kinds.capacity().saturating_mul(32))
                .saturating_add(operation.input_conversions.capacity().saturating_mul(64))
                .saturating_add(
                    operation
                        .precondition_invariants
                        .capacity()
                        .saturating_mul(32),
                );
        }
        for indices in self.by_opcode.values() {
            bytes = bytes.saturating_add(indices.capacity().saturating_mul(32));
        }
        for indices in self.by_conversion.values() {
            bytes = bytes.saturating_add(indices.capacity().saturating_mul(32));
        }
        bytes
    }
    /// Copy admitted declarations into a builder for an explicitly extended package.
    /// The original registry remains immutable; `build` revalidates every declaration and
    /// catches duplicate identities/keys in the extended candidate.
    pub fn to_builder(&self) -> QuantityRegistryBuilder {
        QuantityRegistryBuilder {
            entity_kinds: self.entity_kinds.values().cloned().collect(),
            units: self
                .units
                .values()
                .filter(|unit| unit.definition.is_none())
                .cloned()
                .collect(),
            defined_units: self
                .units
                .values()
                .filter_map(|unit| {
                    Some(DefinedUnit {
                        id: unit.id,
                        symbol: unit.symbol.clone(),
                        composition: unit.definition.clone()?,
                    })
                })
                .collect(),
            kinds: self
                .kinds
                .values()
                .filter(|kind| kind.definition.is_none())
                .cloned()
                .collect(),
            derived_kinds: self
                .kinds
                .values()
                .filter_map(|kind| {
                    Some(DerivedKind {
                        id: kind.id,
                        extensive: kind.extensive,
                        addition_kind: kind.addition_kind,
                        definition: kind.definition.clone()?,
                    })
                })
                .collect(),
            bases: self.bases.values().cloned().collect(),
            reference_states: self.reference_states.values().cloned().collect(),
            quantity_types: self.quantity_types.values().cloned().collect(),
            conversions: self.conversions.values().cloned().collect(),
            operations: self.operations.values().cloned().collect(),
            reduction_domains: self
                .reduction_domains
                .iter()
                .map(|(id, kind)| (*id, *kind))
                .collect(),
            unit_sets: self.unit_sets.values().cloned().collect(),
            neutral: self.neutral.into_iter().collect(),
        }
    }
    lookup!(entity_kind, entity_kinds, EntityKindId, EntityKind);
    /// Declared generic kinds in semantic identity order.
    pub fn entity_kinds(&self) -> impl ExactSizeIterator<Item = &EntityKind> {
        self.entity_kinds.values()
    }
    /// Resolve an unambiguous qualified package name at a legacy source boundary.
    pub fn entity_kind_named(&self, name: &str) -> Option<EntityKindId> {
        let mut found = self.entity_kinds.values().filter(|k| k.name == name);
        let first = found.next()?.id;
        if found.next().is_some() {
            None
        } else {
            Some(first)
        }
    }
    lookup!(unit, units, UnitId, Unit);
    lookup!(kind, kinds, QuantityKindId, QuantityKind);
    lookup!(basis, bases, BasisId, Basis);
    lookup!(
        reference_state,
        reference_states,
        ReferenceStateId,
        ReferenceState
    );
    lookup!(quantity_type, quantity_types, QuantityTypeId, QuantityType);
    lookup!(conversion, conversions, ConversionId, ConversionRule);
    lookup!(operation, operations, OperationId, QuantityOperation);
    /// Exact admitted reduction dispatch contract, separate from applicability preconditions.
    pub fn reduction_domain(&self, operation: OperationId) -> Option<EntityKindId> {
        self.reduction_domains.get(&operation).copied()
    }
    /// Every exact operation/domain dispatch declaration in stable operation order.
    pub fn reduction_domains(
        &self,
    ) -> impl ExactSizeIterator<Item = (OperationId, EntityKindId)> + '_ {
        self.reduction_domains.iter().map(|(id, kind)| (*id, *kind))
    }
    lookup!(unit_set, unit_sets, UnitSetId, UnitSet);
    /// Compose a unit literal (ADR-0124). Each factor symbol names an atomic unit; the
    /// result's identity, dimension and scale follow from the canonical atomic factors, so
    /// a composite literal needs no registered whole unit. A product that is one atomic
    /// unit with exponent one is that unit, including an affine or datum-restricted one.
    ///
    /// # Errors
    /// An unknown atomic symbol, an affine or datum-restricted unit that is not the sole
    /// factor with exponent one, or a product whose exponents or scale overflow.
    pub fn compose(&self, product: &UnitProduct) -> Result<Unit, QuantityError> {
        let factors = canonical_factors(
            product
                .factors()
                .iter()
                .map(|(symbol, exponent)| {
                    let unit = self.by_symbol.get(symbol).copied().ok_or_else(|| {
                        QuantityError::UnknownUnitSymbol {
                            symbol: symbol.clone(),
                        }
                    })?;
                    Ok(UnitFactor {
                        unit,
                        exponent: *exponent,
                    })
                })
                .collect::<Result<Vec<_>, QuantityError>>()?,
        )?;
        if let [only] = factors.as_slice()
            && only.exponent == crate::Ratio::ONE
        {
            return self.unit(only.unit).cloned();
        }
        for factor in &factors {
            let unit = self.unit(factor.unit)?;
            if unit.is_affine || unit.reference_state.is_some() {
                return Err(QuantityError::AffineUnitFactor {
                    unit: unit.id,
                    symbol: unit.symbol.clone(),
                });
            }
        }
        let id = unit_product_id(&factors);
        if let Some(defined) = self.units.get(&id) {
            return Ok(defined.clone());
        }
        let (dimension, scale_to_canonical) =
            crate::unit::derived_measure(&factors, |id| self.unit(id))?;
        Ok(Unit {
            id,
            symbol: product.to_string(),
            dimension,
            scale_to_canonical,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
            definition: Some(factors),
        })
    }
    /// The canonical literal spelling of an admitted unit: its own symbol when atomic,
    /// otherwise its canonical atomic factors. Composing it returns the same unit.
    ///
    /// # Errors
    /// An identity absent from this admitted registry.
    pub fn unit_product(&self, id: UnitId) -> Result<UnitProduct, QuantityError> {
        let unit = self.unit(id)?;
        let Some(factors) = &unit.definition else {
            return Ok(UnitProduct::symbol(unit.symbol.clone()));
        };
        Ok(UnitProduct::from_factors(
            factors
                .iter()
                .map(|factor| Ok((self.unit(factor.unit)?.symbol.clone(), factor.exponent)))
                .collect::<Result<Vec<_>, QuantityError>>()?,
        )?)
    }
    /// All quantity types in stable identity order.
    pub fn quantity_types(&self) -> impl Iterator<Item = &QuantityType> {
        self.quantity_types.values()
    }
    /// All admitted units in stable identity order.
    pub fn units(&self) -> impl ExactSizeIterator<Item = &Unit> {
        self.units.values()
    }
    /// All admitted quantity kinds in stable identity order.
    pub fn kinds(&self) -> impl ExactSizeIterator<Item = &QuantityKind> {
        self.kinds.values()
    }
    /// A kind's canonical monomial over base kinds (ADR-0124): a derived kind's expanded
    /// definition, or the base kind itself with exponent one.
    ///
    /// # Errors
    /// Rejects a kind absent from this admitted registry.
    pub fn kind_monomial(&self, id: QuantityKindId) -> Result<Vec<KindFactor>, QuantityError> {
        let kind = self.kind(id)?;
        Ok(kind.definition.as_ref().map_or_else(
            || {
                vec![KindFactor {
                    kind: id,
                    exponent: crate::Ratio::ONE,
                }]
            },
            |definition| definition.monomial.clone(),
        ))
    }
    /// The declared kind whose canonical monomial this is: a base kind for itself with
    /// exponent one, otherwise the unique derived kind declaring it. Never synthesized.
    pub fn kind_by_monomial(&self, monomial: &[KindFactor]) -> Option<QuantityKindId> {
        if let [only] = monomial
            && only.exponent == crate::Ratio::ONE
            && self
                .kinds
                .get(&only.kind)
                .is_some_and(|kind| kind.definition.is_none())
        {
            return Some(only.kind);
        }
        self.by_monomial.get(monomial).copied()
    }
    /// The declared count or indicator category of a quantity type's kind (ADR-0103).
    /// A measured kind has none.
    ///
    /// # Errors
    /// Rejects a type or kind absent from this admitted registry.
    pub fn discrete_category(
        &self,
        id: QuantityTypeId,
    ) -> Result<Option<QuantityKindCategory>, QuantityError> {
        Ok(self.kind(self.quantity_type(id)?.key.kind)?.category)
    }
    /// All admitted bases in stable identity order.
    pub fn bases(&self) -> impl ExactSizeIterator<Item = &Basis> {
        self.bases.values()
    }
    /// All admitted reference states in stable identity order.
    pub fn reference_states(&self) -> impl ExactSizeIterator<Item = &ReferenceState> {
        self.reference_states.values()
    }
    /// All admitted conversions in stable identity order.
    pub fn conversions(&self) -> impl ExactSizeIterator<Item = &ConversionRule> {
        self.conversions.values()
    }
    /// All admitted operation contracts in stable identity order.
    pub fn operations(&self) -> impl ExactSizeIterator<Item = &QuantityOperation> {
        self.operations.values()
    }
    /// All admitted unit sets in stable identity order.
    pub fn unit_sets(&self) -> impl ExactSizeIterator<Item = &UnitSet> {
        self.unit_sets.values()
    }
    /// Resolve by exact semantic components, never by dimension or a digest.
    ///
    /// # Errors
    /// Rejects an unregistered key.
    pub fn resolve_key(&self, key: &QuantityTypeKey) -> Result<QuantityTypeId, QuantityError> {
        self.by_key.get(key).copied().ok_or_else(|| {
            error(
                "quantity_type.registered_key",
                key.kind.as_id(),
                "no type has the requested complete key",
            )
        })
    }
    /// Candidate operation declarations in stable identity order; inference must match them.
    pub fn operations_for(&self, opcode: Opcode) -> impl Iterator<Item = &QuantityOperation> {
        self.by_opcode
            .get(&opcode)
            .into_iter()
            .flatten()
            .filter_map(|id| self.operations.get(id))
    }
    /// Explicit directed conversions; a reverse conversion is never inferred.
    pub fn conversions_between(
        &self,
        from: QuantityTypeId,
        to: QuantityTypeId,
    ) -> impl Iterator<Item = &ConversionRule> {
        self.by_conversion
            .get(&(from, to))
            .into_iter()
            .flatten()
            .filter_map(|id| self.conversions.get(id))
    }
    /// The explicitly designated neutral scalar, if the package provides one.
    pub fn neutral_dimensionless(&self) -> Option<QuantityTypeId> {
        self.neutral
    }
    /// What a name declared in the physical document denotes (ADR-0123 Outcome 6).
    pub fn physical_name(&self, name: &str) -> Option<PhysicalName> {
        self.by_name.get(name).copied()
    }
    /// Every declared physical name, in name order.
    pub fn physical_names(&self) -> impl ExactSizeIterator<Item = (&str, PhysicalName)> {
        self.by_name
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
    }
    /// A typed reference condition's value in its quantity type's canonical unit.
    ///
    /// # Errors
    /// An unknown type or unit, or a unit that does not convert to the type's.
    pub fn reference_condition(
        &self,
        condition: &crate::ReferenceCondition,
    ) -> Result<f64, QuantityError> {
        Ok(crate::CanonicalConversionPlan::registered(
            self,
            condition.quantity_type,
            condition.unit,
        )?
        .apply(condition.value)?
        .value())
    }
    /// The physical names, checked: identifiers, one namespace across quantity types and
    /// reference states.
    fn names(&self) -> Result<BTreeMap<String, PhysicalName>, QuantityError> {
        let mut names = BTreeMap::new();
        let declared = self
            .quantity_types
            .values()
            .filter_map(|ty| {
                Some((
                    ty.name.as_deref()?,
                    PhysicalName::QuantityType(ty.id),
                    ty.id.as_id(),
                ))
            })
            .chain(self.reference_states.values().map(|state| {
                (
                    state.name.as_str(),
                    PhysicalName::ReferenceState(state.id),
                    state.id.as_id(),
                )
            }));
        for (name, value, subject) in declared {
            let mut chars = name.chars();
            require(
                chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
                    && chars.all(|c| c == '_' || c.is_alphanumeric()),
                "physical_name.identifier",
                subject,
                "a physical name is an identifier",
            )?;
            require(
                names.insert(name.to_owned(), value).is_none(),
                "physical_name.unique",
                subject,
                &format!("the physical name {name} is declared more than once"),
            )?;
        }
        Ok(names)
    }
    /// A typed condition states the dimension it names, in a point type without a datum,
    /// and a unit of that type, with a finite positive value.
    fn validate_condition(
        &self,
        state: ReferenceStateId,
        condition: &crate::ReferenceCondition,
        dimension: &DimensionVector,
        rule: &'static str,
    ) -> Result<(), QuantityError> {
        let ty = self.quantity_type(condition.quantity_type)?;
        require(
            &self.kind(ty.key.kind)?.dimension == dimension
                && ty.key.scale_kind == ScaleKind::Point
                && ty.key.reference_state.is_none(),
            rule,
            state.as_id(),
            "the condition's quantity type is an absolute point of its dimension",
        )?;
        let value = self.reference_condition(condition)?;
        require(
            value.is_finite() && value > 0.0,
            "reference_state.conditions",
            state.as_id(),
            "reference conditions must be finite and positive",
        )
    }
    fn validate(&self) -> Result<(), QuantityError> {
        let mut symbols = BTreeSet::new();
        for kind in self.entity_kinds.values() {
            require(
                kind.id.as_id() != SemanticId::NIL && !kind.name.is_empty(),
                "entity_kind.identity",
                kind.id.as_id(),
                "kind requires an identity and name",
            )?;
        }
        for ty in self.quantity_types.values() {
            for kind in ty.key.shape.iter().chain(ty.key.subject_kind.iter()) {
                self.entity_kind(*kind)?;
            }
        }
        for kind in self.reduction_domains.values() {
            self.entity_kind(*kind)?;
        }
        for operation in self.operations.values() {
            if let Some(kind) = operation.result_subject_kind {
                self.entity_kind(kind)?;
            }
        }
        for unit in self.units.values() {
            unit.validate()?;
            if let Some(reference) = unit.reference_state {
                self.reference_state(reference)?;
            }
            require(
                symbols.insert(&unit.symbol),
                "unit.unique_symbol",
                unit.id.as_id(),
                "duplicate resolved unit symbol",
            )?;
            require(
                unit.definition.is_some() || unit.symbol != "1",
                "unit.symbol",
                unit.id.as_id(),
                "the numeral 1 spells the empty product; declare it as a defined unit",
            )?;
        }
        let temperature = DimensionVector::base(crate::BaseDimension::Temperature);
        let pressure = DimensionVector::base(crate::BaseDimension::Mass)
            .div(&DimensionVector::base(crate::BaseDimension::Length))
            .and_then(|d| {
                d.div(
                    &DimensionVector::base(crate::BaseDimension::Time)
                        .pow(crate::Ratio::new(2, 1)?)?,
                )
            })
            .map_err(QuantityError::from)?;
        for reference in self.reference_states.values() {
            if let Some(condition) = &reference.temperature {
                self.validate_condition(
                    reference.id,
                    condition,
                    &temperature,
                    "reference_state.temperature",
                )?;
            }
            if let Some(condition) = &reference.pressure {
                self.validate_condition(
                    reference.id,
                    condition,
                    &pressure,
                    "reference_state.pressure",
                )?;
            }
        }
        for kind in self.kinds.values() {
            require(
                kind.category.is_none() || kind.dimension == DimensionVector::DIMENSIONLESS,
                "quantity_kind.category_dimensionless",
                kind.id.as_id(),
                "a count or indicator kind is a dimensionless pure number",
            )?;
        }
        for basis in self.bases.values() {
            if let Some(id) = basis.reference_conditions {
                self.reference_state(id)?;
            }
            require(
                basis.kind != BasisKind::StandardVolume || basis.reference_conditions.is_some(),
                "basis.standard_conditions",
                basis.id.as_id(),
                "standard volume requires reference conditions",
            )?;
        }
        let mut keys = BTreeSet::new();
        for ty in self.quantity_types.values() {
            require(
                keys.insert(&ty.key),
                "quantity_type.unique_key",
                ty.id.as_id(),
                "duplicate complete quantity key",
            )?;
            self.validate_type(ty)?;
        }
        self.validate_derived_kinds()?;
        for conversion in self.conversions.values() {
            self.validate_conversion(conversion)?;
        }
        for operation in self.operations.values() {
            self.validate_operation(operation)?;
            require(
                operation.opcode != Opcode::SumOver
                    || self.reduction_domains.contains_key(&operation.id),
                "operation.reduction_domain",
                operation.id.as_id(),
                "registered SumOver requires its explicit consumed domain kind",
            )?;
        }
        for operation in self.reduction_domains.keys() {
            let declared = self.operation(*operation)?;
            require(
                declared.shape_rule == QuantityShapeRule::ReduceBoundIndex
                    && matches!(declared.opcode, Opcode::SumOver | Opcode::Integral),
                "operation.reduction_domain",
                operation.as_id(),
                "domain dispatch requires an explicit indexed reduction operation",
            )?;
        }
        for set in self.unit_sets.values() {
            set.validate(self)?;
        }
        if let Some(id) = self.neutral {
            let ty = self.quantity_type(id)?;
            let kind = self.kind(ty.key.kind)?;
            require(
                kind.dimension.is_dimensionless()
                    && kind.addition_kind == QuantityAdditionKind::Additive
                    && ty.key.basis.is_none()
                    && ty.key.reference_state.is_none()
                    && ty.key.shape.is_empty()
                    && ty.key.subject_kind.is_none()
                    && ty.key.scale_kind == ScaleKind::Point,
                "quantity_type.neutral_scalar",
                id.as_id(),
                "neutral scalar has physical obligations",
            )?;
        }
        Ok(())
    }
    /// Derived kinds (ADR-0124): unique monomials, a canonical unit of the derived
    /// dimension that every type of the kind stores in, existing result components, and a
    /// registered result type.
    fn validate_derived_kinds(&self) -> Result<(), QuantityError> {
        let mut monomials = BTreeSet::new();
        for kind in self.kinds.values() {
            let Some(definition) = &kind.definition else {
                continue;
            };
            let id = kind.id.as_id();
            require(
                monomials.insert(&definition.monomial),
                "quantity_kind.monomial_unique",
                id,
                "two derived kinds declare one monomial",
            )?;
            // A chain keeps dimensionless kinds such as a mole fraction in its monomial,
            // but the neutral scalar and count or indicator factors are pure numbers
            // outside it, so a definition over one could never resolve.
            let neutral = self
                .neutral
                .map(|ty| self.quantity_type(ty).map(|ty| ty.key.kind))
                .transpose()?;
            for factor in &definition.monomial {
                require(
                    Some(factor.kind) != neutral && self.kind(factor.kind)?.category.is_none(),
                    "quantity_kind.pure_number_factor",
                    id,
                    "a derived kind's factor is not a pure number",
                )?;
            }
            let unit = self.unit(definition.canonical_unit)?;
            require(
                unit.dimension == kind.dimension
                    && !unit.is_affine
                    && unit.offset_to_canonical == 0.0
                    && (unit.reference_state.is_none()
                        || unit.reference_state == definition.reference_state),
                "quantity_kind.canonical_unit",
                id,
                "the declared canonical unit does not store the derived dimension",
            )?;
            if let Some(basis) = definition.basis {
                self.basis(basis)?;
            }
            if let Some(reference) = definition.reference_state {
                self.reference_state(reference)?;
            }
            if let Some(subject) = definition.subject_kind {
                self.entity_kind(subject)?;
            }
            require(
                kind.addition_kind == QuantityAdditionKind::OriginSensitive
                    || definition.scale_kind == ScaleKind::Point,
                "quantity_kind.result_scale",
                id,
                "difference is reserved for origin-sensitive kinds",
            )?;
            let mut result = false;
            for ty in self
                .quantity_types
                .values()
                .filter(|ty| ty.key.kind == kind.id)
            {
                require(
                    ty.canonical_unit == definition.canonical_unit,
                    "quantity_kind.canonical_unit",
                    ty.id.as_id(),
                    "a type of a derived kind stores in the kind's declared canonical unit",
                )?;
                result |= ty.key.reference_state == definition.reference_state
                    && ty.key.scale_kind == definition.scale_kind
                    && ty.key.shape.is_empty()
                    && ty.key.subject_kind == definition.subject_kind
                    && definition
                        .basis
                        .is_none_or(|basis| ty.key.basis == Some(basis));
            }
            require(
                result,
                "quantity_kind.result_type",
                id,
                "no registered type has the declared result",
            )?;
        }
        Ok(())
    }
    fn validate_type(&self, ty: &QuantityType) -> Result<(), QuantityError> {
        let kind = self.kind(ty.key.kind)?;
        let unit = self.unit(ty.canonical_unit)?;
        require(
            kind.dimension == unit.dimension,
            "quantity_type.dimension",
            ty.id.as_id(),
            "kind and canonical unit dimensions disagree",
        )?;
        require(
            !unit.is_affine && unit.offset_to_canonical == 0.0,
            "quantity_type.canonical_unit",
            ty.id.as_id(),
            "canonical storage unit is affine",
        )?;
        require(
            unit.reference_state.is_none() || unit.reference_state == ty.key.reference_state,
            "quantity_type.unit_reference",
            ty.id.as_id(),
            "canonical unit restriction disagrees with quantity datum",
        )?;
        if let Some(id) = ty.key.basis {
            self.basis(id)?;
        }
        if let Some(id) = ty.key.reference_state {
            self.reference_state(id)?;
        }
        if let Some(value) = ty.nominal_magnitude {
            require(
                value.is_finite() && value > 0.0,
                "quantity_type.nominal",
                ty.id.as_id(),
                "nominal magnitude must be finite and positive",
            )?;
        }
        require(
            kind.addition_kind == QuantityAdditionKind::OriginSensitive
                || ty.key.scale_kind == ScaleKind::Point,
            "quantity_type.scale_kind",
            ty.id.as_id(),
            "difference is reserved for origin-sensitive kinds",
        )
    }
    fn validate_conversion(&self, rule: &ConversionRule) -> Result<(), QuantityError> {
        let from = self.quantity_type(rule.from)?;
        let to = self.quantity_type(rule.to)?;
        let id = rule.id.as_id();
        let mut parameters = BTreeSet::new();
        for parameter in &rule.required_parameters {
            require(
                !parameter.is_empty() && parameters.insert(parameter),
                "conversion.parameters",
                id,
                "parameter names must be nonempty and unique",
            )?;
        }
        match rule.kind {
            ConversionKind::Kernel => require(
                rule.kernel.is_some() && rule.scale.is_none() && rule.offset.is_none(),
                "conversion.kernel",
                id,
                "kernel conversion requires a binding and no scalar coefficients",
            )?,
            ConversionKind::Scale | ConversionKind::Affine => {
                require(
                    rule.kernel.is_none() && rule.required_parameters.is_empty(),
                    "conversion.scalar",
                    id,
                    "scalar conversion cannot hide a kernel or parameter dependency",
                )?;
                require(
                    rule.scale.is_some_and(|x| x.is_finite() && x > 0.0)
                        && rule.offset.is_none_or(f64::is_finite),
                    "conversion.coefficients",
                    id,
                    "scalar conversion requires finite coefficients and a positive scale",
                )?;
                require(
                    self.kind(from.key.kind)?.dimension == self.kind(to.key.kind)?.dimension,
                    "conversion.dimension",
                    id,
                    "scalar conversion changes physical dimension",
                )?;
                require(
                    from.key.kind == to.key.kind
                        && from.key.basis == to.key.basis
                        && from.key.shape == to.key.shape
                        && from.key.subject_kind == to.key.subject_kind
                        && from.key.scale_kind == to.key.scale_kind,
                    "conversion.contract",
                    id,
                    "scalar conversion changes kind, basis, scale, shape or subject",
                )?;
                if rule.kind == ConversionKind::Scale {
                    let expected = crate::convert_spec_for_type(
                        self.unit(from.canonical_unit)?,
                        self.unit(to.canonical_unit)?,
                        &from.key,
                    )?;
                    require(
                        rule.scale.is_some_and(|scale| {
                            pse_ids::canonical_f64_bits(scale)
                                == pse_ids::canonical_f64_bits(expected.scale)
                        }),
                        "conversion.unit_scale",
                        id,
                        "declared scale disagrees with the actual canonical-unit conversion",
                    )?;
                    require(
                        rule.offset.is_none_or(|offset| offset == 0.0)
                            && from.key.reference_state == to.key.reference_state,
                        "conversion.scale",
                        id,
                        "scale conversion has an offset or changes datum",
                    )?;
                } else {
                    require(
                        from.key.scale_kind == ScaleKind::Point && rule.offset.is_some(),
                        "conversion.affine_point",
                        id,
                        "affine conversion cannot act on differences",
                    )?;
                    let (scale, offset) = if from.key.reference_state != to.key.reference_state {
                        let admitted = crate::DatumConversionPlan::admit(self, rule.from, rule.to)?;
                        (admitted.scale(), admitted.offset())
                    } else {
                        let admitted = crate::convert_spec_for_type(
                            self.unit(from.canonical_unit)?,
                            self.unit(to.canonical_unit)?,
                            &from.key,
                        )?;
                        (admitted.scale, admitted.offset)
                    };
                    require(
                        rule.scale.map(f64::to_bits) == Some(scale.to_bits())
                            && rule.offset.map(f64::to_bits) == Some(offset.to_bits()),
                        "conversion.datum_origin",
                        id,
                        "affine coefficients must be derived from the selected typed datum and unit representations",
                    )?;
                }
            }
        }
        Ok(())
    }
    fn validate_operation(&self, rule: &QuantityOperation) -> Result<(), QuantityError> {
        let id = rule.id.as_id();
        self.kind(rule.result_kind)?;
        for kind in &rule.input_kinds {
            self.kind(*kind)?;
        }
        require(
            !rule.input_kinds.is_empty(),
            "operation.operands",
            id,
            "registered composition requires operands",
        )?;
        validate_sources(rule)?;
        if let Some(basis) = rule.result_basis {
            self.basis(basis)?;
        }
        if let Some(reference) = rule.result_reference_state {
            self.reference_state(reference)?;
        }
        require(
            rule.basis_rule == BasisRule::DeclaredResult || rule.result_basis.is_none(),
            "operation.result_basis",
            id,
            "result basis requires declared_result",
        )?;
        require(
            rule.reference_rule == ReferenceRule::DeclaredResult
                || rule.result_reference_state.is_none(),
            "operation.result_reference",
            id,
            "result datum requires declared_result",
        )?;
        require(
            rule.subject_rule == SubjectRule::DeclaredResult || rule.result_subject_kind.is_none(),
            "operation.result_subject",
            id,
            "result subject requires declared_result",
        )?;
        let requires_invariant = matches!(
            rule.basis_rule,
            BasisRule::Cancel | BasisRule::DeclaredResult
        ) || matches!(
            rule.reference_rule,
            ReferenceRule::Cancel | ReferenceRule::DeclaredResult
        ) || rule.subject_rule == SubjectRule::DeclaredResult;
        require(
            !requires_invariant || !rule.precondition_invariants.is_empty(),
            "operation.precondition",
            id,
            "cancel/declared_result requires an explicit invariant",
        )?;
        let mut invariants = BTreeSet::new();
        require(
            rule.precondition_invariants
                .iter()
                .all(|x| invariants.insert(*x)),
            "operation.unique_invariant",
            id,
            "duplicate invariant dependency",
        )?;
        let mut converted = BTreeSet::new();
        for conversion in &rule.input_conversions {
            let declaration = self.conversion(conversion.conversion)?;
            let position = usize::from(conversion.operand);
            require(
                position < rule.input_kinds.len() && converted.insert(position),
                "operation.input_conversion",
                id,
                "conversion operand is out of range or repeated",
            )?;
            require(
                self.quantity_type(declaration.to)?.key.kind == rule.input_kinds[position],
                "operation.converted_kind",
                id,
                "conversion output kind does not match the declared operand kind",
            )?;
        }
        require(
            !(rule.basis_rule == BasisRule::RegisteredConversion
                || rule.reference_rule == ReferenceRule::RegisteredConversion)
                || !rule.input_conversions.is_empty(),
            "operation.registered_conversion",
            id,
            "conversion policy requires explicit operand conversions",
        )?;
        Ok(())
    }
}
fn validate_sources(rule: &QuantityOperation) -> Result<(), QuantityError> {
    let id = rule.id.as_id();
    for (source, preserve) in [
        (rule.basis_source, rule.basis_rule == BasisRule::Preserve),
        (
            rule.reference_source,
            rule.reference_rule == ReferenceRule::Preserve,
        ),
        (
            rule.scale_source,
            rule.scale_rule == QuantityScaleRule::Preserve,
        ),
        (
            rule.shape_source,
            rule.shape_rule == QuantityShapeRule::Preserve,
        ),
        (
            rule.subject_source,
            rule.subject_rule == SubjectRule::Preserve,
        ),
    ] {
        require(
            source.is_none_or(|ordinal| usize::from(ordinal) < rule.input_kinds.len())
                && (!preserve || source.is_some()),
            "operation.source",
            id,
            "preserve needs an in-range operand source",
        )?;
        require(
            preserve || source.is_none(),
            "operation.unused_source",
            id,
            "source supplied for a non-preserve policy",
        )?;
    }
    Ok(())
}

/// Derive each defined unit from its composition (ADR-0124): expand defined factors into
/// canonical atomic factors, refuse cycles, aliases and affine or datum-restricted
/// factors, require the declared identity to be the product identity, and derive the
/// dimension and scale.
fn admit_defined(
    atomic: &BTreeMap<UnitId, Unit>,
    defined: Vec<DefinedUnit>,
) -> Result<Vec<Unit>, QuantityError> {
    let declared = index(defined, |x| x.id, "unit")?;
    for id in declared.keys() {
        require(
            !atomic.contains_key(id),
            "registry.unique_id",
            id.as_id(),
            "duplicate unit identity",
        )?;
    }
    let mut expanded = BTreeMap::new();
    for id in declared.keys() {
        expand(*id, atomic, &declared, &mut expanded, &mut BTreeSet::new())?;
    }
    declared
        .into_values()
        .map(|unit| {
            let factors = expanded.remove(&unit.id).unwrap_or_default();
            require(
                !matches!(factors.as_slice(), [only] if only.exponent == crate::Ratio::ONE),
                "unit.defined_alias",
                unit.id.as_id(),
                "a defined unit is not another unit's alias",
            )?;
            let derived = unit_product_id(&factors);
            require(
                derived == unit.id,
                "unit.defined_identity",
                unit.id.as_id(),
                &format!("a defined unit's identity is its product identity {derived}"),
            )?;
            let (dimension, scale_to_canonical) = crate::unit::derived_measure(&factors, |id| {
                atomic.get(&id).ok_or(QuantityError::UnknownId {
                    kind: "unit",
                    id: id.as_id(),
                })
            })?;
            Ok(Unit {
                id: unit.id,
                symbol: unit.symbol,
                dimension,
                scale_to_canonical,
                offset_to_canonical: 0.0,
                is_affine: false,
                reference_state: None,
                definition: Some(factors),
            })
        })
        .collect()
}
fn expand(
    id: UnitId,
    atomic: &BTreeMap<UnitId, Unit>,
    declared: &BTreeMap<UnitId, DefinedUnit>,
    expanded: &mut BTreeMap<UnitId, Vec<UnitFactor>>,
    visiting: &mut BTreeSet<UnitId>,
) -> Result<Vec<UnitFactor>, QuantityError> {
    if let Some(unit) = atomic.get(&id) {
        require(
            !unit.is_affine && unit.reference_state.is_none(),
            "unit.affine_factor",
            id.as_id(),
            "an affine or datum-restricted unit cannot be a defined unit's factor",
        )?;
        return Ok(vec![UnitFactor {
            unit: id,
            exponent: crate::Ratio::ONE,
        }]);
    }
    if let Some(done) = expanded.get(&id) {
        return Ok(done.clone());
    }
    let unit = declared.get(&id).ok_or(QuantityError::UnknownId {
        kind: "unit",
        id: id.as_id(),
    })?;
    require(
        visiting.insert(id),
        "unit.defined_cycle",
        id.as_id(),
        "a defined unit's composition refers back to itself",
    )?;
    let mut factors = Vec::new();
    for factor in &unit.composition {
        for inner in expand(factor.unit, atomic, declared, expanded, visiting)? {
            factors.push(UnitFactor {
                unit: inner.unit,
                exponent: inner.exponent.checked_mul(factor.exponent)?,
            });
        }
    }
    let factors = canonical_factors(factors)?;
    visiting.remove(&id);
    expanded.insert(id, factors.clone());
    Ok(factors)
}

/// Expand each derived kind's monomial into canonical base-kind factors (ADR-0124),
/// refusing cycles and aliases, and derive its dimension.
fn admit_derived_kinds(
    base: &BTreeMap<QuantityKindId, QuantityKind>,
    derived: Vec<DerivedKind>,
) -> Result<Vec<QuantityKind>, QuantityError> {
    let declared = index(derived, |x| x.id, "quantity_kind")?;
    for id in declared.keys() {
        require(
            !base.contains_key(id),
            "registry.unique_id",
            id.as_id(),
            "duplicate quantity_kind identity",
        )?;
    }
    let mut expanded = BTreeMap::new();
    for id in declared.keys() {
        expand_kind(*id, base, &declared, &mut expanded, &mut BTreeSet::new())?;
    }
    declared
        .into_values()
        .map(|kind| {
            let monomial = expanded.remove(&kind.id).unwrap_or_default();
            require(
                !monomial.is_empty()
                    && !matches!(monomial.as_slice(), [only] if only.exponent == crate::Ratio::ONE),
                "quantity_kind.definition_alias",
                kind.id.as_id(),
                "a derived kind's monomial is neither empty nor another kind",
            )?;
            let mut dimension = DimensionVector::DIMENSIONLESS;
            for factor in &monomial {
                let factor_kind = base.get(&factor.kind).ok_or(QuantityError::UnknownId {
                    kind: "kind",
                    id: factor.kind.as_id(),
                })?;
                dimension = dimension.mul(&factor_kind.dimension.pow(factor.exponent)?)?;
            }
            Ok(QuantityKind {
                id: kind.id,
                dimension,
                extensive: kind.extensive,
                addition_kind: kind.addition_kind,
                category: None,
                definition: Some(KindDefinition {
                    monomial,
                    ..kind.definition
                }),
            })
        })
        .collect()
}
fn expand_kind(
    id: QuantityKindId,
    base: &BTreeMap<QuantityKindId, QuantityKind>,
    declared: &BTreeMap<QuantityKindId, DerivedKind>,
    expanded: &mut BTreeMap<QuantityKindId, Vec<KindFactor>>,
    visiting: &mut BTreeSet<QuantityKindId>,
) -> Result<Vec<KindFactor>, QuantityError> {
    if base.contains_key(&id) {
        return Ok(vec![KindFactor {
            kind: id,
            exponent: crate::Ratio::ONE,
        }]);
    }
    if let Some(done) = expanded.get(&id) {
        return Ok(done.clone());
    }
    let kind = declared.get(&id).ok_or(QuantityError::UnknownId {
        kind: "kind",
        id: id.as_id(),
    })?;
    require(
        visiting.insert(id),
        "quantity_kind.definition_cycle",
        id.as_id(),
        "a derived kind's monomial refers back to itself",
    )?;
    let mut merged = BTreeMap::<QuantityKindId, crate::Ratio>::new();
    for factor in &kind.definition.monomial {
        for inner in expand_kind(factor.kind, base, declared, expanded, visiting)? {
            let slot = merged.entry(inner.kind).or_insert(crate::Ratio::ZERO);
            *slot = slot.checked_add(inner.exponent.checked_mul(factor.exponent)?)?;
        }
    }
    let factors: Vec<_> = merged
        .into_iter()
        .filter(|(_, exponent)| !exponent.is_zero())
        .map(|(kind, exponent)| KindFactor { kind, exponent })
        .collect();
    visiting.remove(&id);
    expanded.insert(id, factors.clone());
    Ok(factors)
}

fn index<K: Ord + Copy + Into<SemanticId>, V>(
    values: Vec<V>,
    key: impl Fn(&V) -> K,
    kind: &'static str,
) -> Result<BTreeMap<K, V>, QuantityError> {
    let mut result = BTreeMap::new();
    for value in values {
        let id = key(&value);
        if result.insert(id, value).is_some() {
            return Err(error(
                "registry.unique_id",
                id.into(),
                &format!("duplicate {kind} identity"),
            ));
        }
    }
    Ok(result)
}
fn error(rule: &'static str, subject: SemanticId, detail: &str) -> QuantityError {
    QuantityError::Registry {
        rule,
        subject,
        detail: detail.to_owned(),
    }
}
fn require(
    condition: bool,
    rule: &'static str,
    subject: SemanticId,
    detail: &str,
) -> Result<(), QuantityError> {
    if condition {
        Ok(())
    } else {
        Err(error(rule, subject, detail))
    }
}

#[cfg(test)]
mod generic_kind_tests {
    use super::*;
    use crate::{EntityKind, EntityKindId};
    #[test]
    fn authored_kind_identity_controls_axes_and_subjects() {
        use crate::{DimensionVector, QuantityAdditionKind, QuantityTypeKey};
        let raw = SemanticId::from_bytes([1; 16]);
        let mut seed = QuantityRegistryBuilder::new();
        seed.unit(Unit {
            id: raw.into(),
            symbol: "one".into(),
            dimension: DimensionVector::DIMENSIONLESS,
            scale_to_canonical: 1.0,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
            definition: None,
        });
        seed.kind(QuantityKind {
            id: raw.into(),
            dimension: DimensionVector::DIMENSIONLESS,
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
            category: None,
            definition: None,
        });
        seed.quantity_type(QuantityType {
            id: raw.into(),
            name: None,
            key: QuantityTypeKey {
                kind: raw.into(),
                basis: None,
                reference_state: None,
                scale_kind: ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: raw.into(),
            nominal_magnitude: None,
        });
        seed.neutral_dimensionless(raw.into());
        let registry = seed.build().unwrap();
        let membrane = EntityKindId::from_id(SemanticId::from_bytes([91; 16]));
        let mut quantity = registry
            .quantity_type(registry.neutral_dimensionless().unwrap())
            .unwrap()
            .clone();
        quantity.id = QuantityTypeId::from_id(SemanticId::from_bytes([92; 16]));
        quantity.key.shape = vec![membrane];
        quantity.key.subject_kind = Some(membrane);
        let mut missing = registry.to_builder();
        missing.quantity_type(quantity.clone());
        assert!(missing.build().is_err());
        let mut builder = registry.to_builder();
        builder.entity_kind(EntityKind {
            id: membrane,
            name: "synthetic.membrane".into(),
        });
        builder.quantity_type(quantity.clone());
        let admitted = builder.build().unwrap();
        assert_eq!(
            admitted.quantity_type(quantity.id).unwrap().key.shape,
            [membrane]
        );
        assert_eq!(
            admitted
                .quantity_type(quantity.id)
                .unwrap()
                .key
                .subject_kind,
            Some(membrane)
        );
        assert_eq!(
            admitted.entity_kind_named("synthetic.membrane"),
            Some(membrane)
        );
        let mut renamed = admitted.to_builder();
        // A second identity is a second kind even when its display spelling is equal.
        renamed.entity_kind(EntityKind {
            id: EntityKindId::from_id(SemanticId::from_bytes([93; 16])),
            name: "synthetic.membrane".into(),
        });
        assert_eq!(
            renamed
                .build()
                .unwrap()
                .entity_kind_named("synthetic.membrane"),
            None
        );
    }
}
