// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admission and immutable lookup of physical contracts (blueprint §6.2, §8).
//!
//! Registry construction checks the actual declarations and their references. No hash
//! participates in validity, type selection, or the proof that a conversion applies.
use crate::basis::Basis;
use crate::conversion::ConversionRule;
use crate::kind::QuantityKind;
use crate::operation::QuantityOperation;
use crate::quantity_type::{QuantityType, QuantityTypeKey};
use crate::reference_state::ReferenceState;
use crate::unit::Unit;
use crate::unit_set::UnitSet;
use crate::{
    BasisId, BasisKind, BasisRule, ConversionId, ConversionKind, DomainKind, Opcode, OperationId,
    QuantityAdditionKind, QuantityError, QuantityKindId, QuantityScaleRule, QuantityShapeRule,
    QuantityTypeId, ReferenceRule, ReferenceStateId, ScaleKind, SubjectKind, SubjectRule, UnitId,
    UnitSetId,
};
use pse_ids::SemanticId;
use std::collections::{BTreeMap, BTreeSet};

/// Unadmitted declarations. Duplicate identities are retained until `build` rejects them.
#[derive(Clone, Debug, Default)]
pub struct QuantityRegistryBuilder {
    units: Vec<Unit>,
    kinds: Vec<QuantityKind>,
    bases: Vec<Basis>,
    reference_states: Vec<ReferenceState>,
    quantity_types: Vec<QuantityType>,
    conversions: Vec<ConversionRule>,
    operations: Vec<QuantityOperation>,
    reduction_domains: Vec<(OperationId, DomainKind)>,
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
    add_declaration!(unit, units, Unit);
    add_declaration!(kind, kinds, QuantityKind);
    add_declaration!(basis, bases, Basis);
    add_declaration!(reference_state, reference_states, ReferenceState);
    add_declaration!(quantity_type, quantity_types, QuantityType);
    add_declaration!(conversion, conversions, ConversionRule);
    add_declaration!(operation, operations, QuantityOperation);
    /// Declare the exact domain kind consumed by a registered index reduction.
    pub fn reduction_domain(&mut self, operation: OperationId, domain: DomainKind) -> &mut Self {
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
        let registry = QuantityRegistry {
            units: index(self.units, |x| x.id, "unit")?,
            kinds: index(self.kinds, |x| x.id, "quantity_kind")?,
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
        };
        registry.validate()?;
        Ok(registry)
    }
}
/// An immutable collection of admitted physical declarations.
#[derive(Clone, Debug)]
pub struct QuantityRegistry {
    units: BTreeMap<UnitId, Unit>,
    kinds: BTreeMap<QuantityKindId, QuantityKind>,
    bases: BTreeMap<BasisId, Basis>,
    reference_states: BTreeMap<ReferenceStateId, ReferenceState>,
    quantity_types: BTreeMap<QuantityTypeId, QuantityType>,
    conversions: BTreeMap<ConversionId, ConversionRule>,
    operations: BTreeMap<OperationId, QuantityOperation>,
    reduction_domains: BTreeMap<OperationId, DomainKind>,
    unit_sets: BTreeMap<UnitSetId, UnitSet>,
    neutral: Option<QuantityTypeId>,
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
    /// Copy admitted declarations into a builder for an explicitly extended package.
    /// The original registry remains immutable; `build` revalidates every declaration and
    /// catches duplicate identities/keys in the extended candidate.
    pub fn to_builder(&self) -> QuantityRegistryBuilder {
        QuantityRegistryBuilder {
            units: self.units.values().cloned().collect(),
            kinds: self.kinds.values().cloned().collect(),
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
    pub fn reduction_domain(&self, operation: OperationId) -> Option<DomainKind> {
        self.reduction_domains.get(&operation).copied()
    }
    /// Every exact operation/domain dispatch declaration in stable operation order.
    pub fn reduction_domains(
        &self,
    ) -> impl ExactSizeIterator<Item = (OperationId, DomainKind)> + '_ {
        self.reduction_domains.iter().map(|(id, kind)| (*id, *kind))
    }
    lookup!(unit_set, unit_sets, UnitSetId, UnitSet);
    /// Lookup by the unique resolved unit symbol.
    pub fn unit_by_symbol(&self, symbol: &str) -> Option<&Unit> {
        self.units.values().find(|x| x.symbol == symbol)
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
        self.quantity_types
            .values()
            .find(|x| x.key == *key)
            .map(|x| x.id)
            .ok_or_else(|| {
                error(
                    "quantity_type.registered_key",
                    key.kind.as_id(),
                    "no type has the requested complete key",
                )
            })
    }
    /// Candidate operation declarations in stable identity order; inference must match them.
    pub fn operations_for(&self, opcode: Opcode) -> impl Iterator<Item = &QuantityOperation> {
        self.operations.values().filter(move |x| x.opcode == opcode)
    }
    /// Explicit directed conversions; a reverse conversion is never inferred.
    pub fn conversions_between(
        &self,
        from: QuantityTypeId,
        to: QuantityTypeId,
    ) -> impl Iterator<Item = &ConversionRule> {
        self.conversions
            .values()
            .filter(move |x| x.from == from && x.to == to)
    }
    /// The explicitly designated neutral scalar, if the package provides one.
    pub fn neutral_dimensionless(&self) -> Option<QuantityTypeId> {
        self.neutral
    }
    fn validate(&self) -> Result<(), QuantityError> {
        let mut symbols = BTreeSet::new();
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
        }
        for reference in self.reference_states.values() {
            for value in [reference.temperature, reference.pressure]
                .into_iter()
                .flatten()
            {
                require(
                    value.is_finite() && value > 0.0,
                    "reference_state.conditions",
                    reference.id.as_id(),
                    "reference conditions must be finite and positive",
                )?;
            }
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
                    && matches!(ty.key.subject_kind, None | Some(SubjectKind::None))
                    && ty.key.scale_kind == ScaleKind::Point,
                "quantity_type.neutral_scalar",
                id.as_id(),
                "neutral scalar has physical obligations",
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
