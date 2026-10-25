// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite readable projections of the admitted physical owner.
use super::{PhysicalContext, WorkflowError, contract};
use pse_quantity::{QuantityTypeId, ReferenceStateId, UnitId};
use serde::{Deserialize, Serialize};

/// A bounded query against one immutable physical context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalInspectionRequest {
    /// Exact declared identity/name selection.
    pub query: PhysicalInspectionQuery,
    /// Original declaration offset in semantic identity order.
    pub offset: usize,
    /// One through sixteen declarations; encoded output is at most 64 KiB.
    #[schemars(range(min = 1, max = 16))]
    pub limit: usize,
}
impl PhysicalInspectionRequest {
    /// Shared finite query admission, before lookup, cloning or allocation.
    pub(super) fn valid_bounds(&self) -> bool {
        (1..=16).contains(&self.limit) && match &self.query {
            PhysicalInspectionQuery::QuantityNamed { name } | PhysicalInspectionQuery::ReferenceStateNamed { name } => name.len() <= 1024,
            _ => true,
        }
    }
}
/// Physical names are package-declared names, never global aliases.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PhysicalInspectionQuery {
    /// All quantities in the selected context, paged without cloning its inventory.
    Quantities,
    /// One exact quantity and optionally its admitted representation conversion.
    Quantity {
        /// Complete physical quantity identity.
        id: QuantityTypeId,
        /// Representation unit; absent selects the canonical unit.
        unit: Option<UnitId>,
    },
    /// One exact declared quantity name.
    QuantityNamed {
        /// Declared physical-document name, case sensitive.
        #[schemars(length(max = 1024))]
        name: String,
    },
    /// One exact declared reference-state name.
    ReferenceStateNamed {
        /// Declared physical-document name, case sensitive.
        #[schemars(length(max = 1024))]
        name: String,
    },
}
/// Original selected quantity and its complete physical interpretation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalQuantityDescription {
    /// Exact original type including basis, datum, scale, shape and subject.
    pub quantity: pse_quantity::QuantityType,
    /// What is measured, including exact rational dimension and arithmetic convention.
    pub quantity_kind: pse_quantity::QuantityKind,
    /// Original canonical representation.
    pub canonical_unit: pse_quantity::Unit,
    /// Selected representation; raw result values are never changed by inspection.
    pub representation_unit: pse_quantity::Unit,
    /// Admitted conversion multiplier to canonical magnitude.
    pub scale_to_canonical: f64,
    /// Admitted conversion offset; difference quantities retain zero here.
    pub offset_to_canonical: f64,
    /// Exact original basis and its composition/rate/reference conventions.
    pub basis: Option<pse_quantity::Basis>,
    /// Actual reference conditions required by a standard-volume or similar basis.
    pub basis_reference_state: Option<pse_quantity::ReferenceState>,
    /// Exact original datum, including typed reference conditions.
    pub reference_state: Option<pse_quantity::ReferenceState>,
    /// Declared ordered axis kinds.
    pub shape: Vec<pse_quantity::EntityKind>,
    /// Declared subject kind.
    pub subject: Option<pse_quantity::EntityKind>,
}
/// One immutable context page. An extent refusal never returns truncated meaning.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PhysicalInspectionDocument {
    /// Complete selected declarations with their actual immutable context identity.
    Page {
        /// Exact physical context that interpreted every record.
        context: pse_ids::ContentHash,
        /// Complete selected population, independent of the current page.
        count: usize,
        /// Original offset.
        offset: usize,
        /// Selected quantity records, in stable identity order.
        quantities: Vec<PhysicalQuantityDescription>,
        /// Selected named reference-state records.
        reference_states: Vec<pse_quantity::ReferenceState>,
        /// Next original offset; each call reacquires any canonical protection.
        next: Option<usize>,
        /// Native allocation retained through serialization of this bounded page.
        #[serde(skip)]
        #[schemars(skip)]
        owner: Option<std::sync::Arc<pse_columnar::MemoryReservation>>,
    },
    /// Required complete records do not fit the fixed document allowance.
    Unavailable {
        /// Exact context, retained even when no record can be copied.
        context: pse_ids::ContentHash,
        /// Explicit reason; narrowing the selection may make it readable.
        reason: String,
    },
}
impl PhysicalContext {
    /// Resolve only a selected registry's declared quantity name.
    /// # Errors
    /// Unknown or ambiguous names refuse instead of selecting an arbitrary identity.
    pub fn resolve_quantity(&self, name: &str) -> Result<QuantityTypeId, WorkflowError> {
        if name.len() > 1024 { return Err(contract("physical name exceeds 1024-byte query allowance")); }
        match self.quantities().physical_name(name) {
            Some(pse_quantity::PhysicalName::QuantityType(id)) => Ok(id),
            _ => Err(contract("unknown declared physical quantity name or wrong name domain")),
        }
    }
    /// Resolve one exact declared representation symbol within this context.
    /// # Errors
    /// Unknown or ambiguous symbols refuse; no global aliases are consulted.
    pub fn resolve_unit(&self, symbol: &str) -> Result<UnitId, WorkflowError> {
        if symbol.len() > 1024 { return Err(contract("physical symbol exceeds 1024-byte query allowance")); }
        let mut matches = self.quantities().units().filter(|unit| unit.symbol == symbol);
        let id = matches.next().ok_or_else(|| contract("unknown declared physical unit symbol"))?.id;
        if matches.next().is_some() { return Err(contract("ambiguous declared physical unit symbol")); }
        Ok(id)
    }
    /// Resolve only a selected registry's declared reference-state name.
    /// # Errors
    /// Unknown or ambiguous names refuse.
    pub fn resolve_reference_state(&self, name: &str) -> Result<ReferenceStateId, WorkflowError> {
        if name.len() > 1024 { return Err(contract("physical name exceeds 1024-byte query allowance")); }
        match self.quantities().physical_name(name) {
            Some(pse_quantity::PhysicalName::ReferenceState(id)) => Ok(id),
            _ => Err(contract("unknown declared reference-state name or wrong name domain")),
        }
    }
    fn describe_quantity(&self, id: QuantityTypeId, unit: Option<UnitId>) -> Result<PhysicalQuantityDescription, WorkflowError> {
        let registry = self.quantities();
        let quantity = registry.quantity_type(id).map_err(quantity_error)?;
        // Count borrowed originals before cloning any potentially large nested field.
        let mut extent = Extent(0);
        serde_json::to_writer(&mut extent, quantity).map_err(|e| contract(e.to_string()))?;
        serde_json::to_writer(&mut extent, registry.kind(quantity.key.kind).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?;
        serde_json::to_writer(&mut extent, registry.unit(quantity.canonical_unit).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?;
        serde_json::to_writer(&mut extent, registry.unit(unit.unwrap_or(quantity.canonical_unit)).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?;
        for id in &quantity.key.shape { serde_json::to_writer(&mut extent, registry.entity_kind(*id).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?; }
        if let Some(id) = quantity.key.basis { serde_json::to_writer(&mut extent, registry.basis(id).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?; }
        let basis_reference = quantity.key.basis.map(|id| registry.basis(id)).transpose().map_err(quantity_error)?.and_then(|basis| basis.reference_conditions);
        if let Some(id) = basis_reference { serde_json::to_writer(&mut extent, registry.reference_state(id).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?; }
        if let Some(id) = quantity.key.reference_state { serde_json::to_writer(&mut extent, registry.reference_state(id).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?; }
        if let Some(id) = quantity.key.subject_kind { serde_json::to_writer(&mut extent, registry.entity_kind(id).map_err(quantity_error)?).map_err(|e| contract(e.to_string()))?; }
        if extent.0 > 60 << 10 { return Err(contract("complete physical record exceeds bounded projection allowance")); }
        let quantity = quantity.clone();
        let representation = unit.unwrap_or(quantity.canonical_unit);
        let plan = pse_quantity::CanonicalConversionPlan::registered(registry, id, representation).map_err(quantity_error)?;
        Ok(PhysicalQuantityDescription {
            quantity_kind: registry.kind(quantity.key.kind).map_err(quantity_error)?.clone(),
            canonical_unit: registry.unit(quantity.canonical_unit).map_err(quantity_error)?.clone(),
            representation_unit: registry.unit(representation).map_err(quantity_error)?.clone(),
            scale_to_canonical: plan.scale(),
            offset_to_canonical: plan.offset(),
            basis: quantity.key.basis.map(|id| registry.basis(id).cloned()).transpose().map_err(quantity_error)?,
            basis_reference_state: basis_reference.map(|id| registry.reference_state(id).cloned()).transpose().map_err(quantity_error)?,
            reference_state: quantity.key.reference_state.map(|id| registry.reference_state(id).cloned()).transpose().map_err(quantity_error)?,
            shape: quantity.key.shape.iter().map(|id| registry.entity_kind(*id).cloned()).collect::<Result<_, _>>().map_err(quantity_error)?,
            subject: quantity.key.subject_kind.map(|id| registry.entity_kind(id).cloned()).transpose().map_err(quantity_error)?,
            quantity,
        })
    }
    /// Describe a finite original-context page through the actual physical registry.
    /// # Errors
    /// Invalid bounds, names, identity domains or incompatible unit/quantity contracts refuse.
    pub fn describe(&self, request: &PhysicalInspectionRequest) -> Result<PhysicalInspectionDocument, WorkflowError> {
        if !request.valid_bounds() { return Err(contract("physical inspection requires a 1..=16 record page and at most 1024-byte names")); }
        let reservation = pse_columnar::MemoryConsumer::new("physical:readable-projection").register(&self.pool);
        reservation.try_grow(16 * (64 << 10)).map_err(pse_engine::EngineError::from)?;
        let owner = std::sync::Arc::new(reservation);
        let mut quantities = Vec::new();
        let mut reference_states = Vec::new();
        let count = match &request.query {
            PhysicalInspectionQuery::Quantities => {
                let count = self.quantities().quantity_types().count();
                for quantity in self.quantities().quantity_types().skip(request.offset).take(request.limit) {
                    quantities.push(self.describe_quantity(quantity.id, None)?);
                }
                count
            }
            PhysicalInspectionQuery::Quantity { id, unit } => {
                let quantity = self.describe_quantity(*id, *unit)?;
                if request.offset == 0 { quantities.push(quantity); }
                1
            }
            PhysicalInspectionQuery::QuantityNamed { name } => {
                let quantity = self.describe_quantity(self.resolve_quantity(name)?, None)?;
                if request.offset == 0 { quantities.push(quantity); }
                1
            }
            PhysicalInspectionQuery::ReferenceStateNamed { name } => {
                let reference = self.quantities().reference_state(self.resolve_reference_state(name)?).map_err(quantity_error)?;
                let mut extent = Extent(0);
                serde_json::to_writer(&mut extent, reference).map_err(|e| contract(e.to_string()))?;
                if extent.0 > 60 << 10 { return Err(contract("complete physical record exceeds bounded projection allowance")); }
                if request.offset == 0 { reference_states.push(reference.clone()); }
                1
            }
        };
        if request.offset > count { return Err(contract("physical inspection offset outside selection")); }
        let end = request.offset.saturating_add(request.limit).min(count);
        let document = PhysicalInspectionDocument::Page { context: self.identity(), count, offset: request.offset, quantities, reference_states, next: (end < count).then_some(end), owner: Some(owner) };
        let mut extent = Extent(0);
        serde_json::to_writer(&mut extent, &document).map_err(|e| contract(e.to_string()))?;
        if extent.0 > 64 << 10 {
            return Ok(PhysicalInspectionDocument::Unavailable { context: self.identity(), reason: "complete physical records exceed the 64 KiB document allowance".into() });
        }
        Ok(document)
    }
}
struct Extent(usize);
impl std::io::Write for Extent {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> { self.0 = self.0.saturating_add(bytes.len()); Ok(bytes.len()) }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn quantity_error(error: pse_quantity::QuantityError) -> WorkflowError { super::math(pse_math::MathError::Quantity(error)) }

#[cfg(test)]
mod physical_inspection_unit {
    use super::*;
    use std::sync::Arc;
    fn request(query: PhysicalInspectionQuery) -> PhysicalInspectionRequest { PhysicalInspectionRequest { query, offset: 0, limit: 1 } }
    fn one(context: &PhysicalContext, id: QuantityTypeId, unit: Option<UnitId>) -> PhysicalQuantityDescription {
        let PhysicalInspectionDocument::Page { mut quantities, .. } = context.describe(&request(PhysicalInspectionQuery::Quantity { id, unit })).unwrap() else { panic!("complete fixture record") };
        quantities.remove(0)
    }
    #[test]
    fn physical_projection_affine_points_and_differences_keep_owner_conversion_and_domains() {
        let context = super::super::tests::physical();
        let celsius = context.resolve_unit("degC").unwrap();
        let point = one(&context, context.resolve_quantity("Temperature").unwrap(), Some(celsius));
        let difference = one(&context, context.resolve_quantity("DeltaTemperature").unwrap(), Some(celsius));
        assert_eq!(point.offset_to_canonical, 273.15);
        assert_eq!(difference.offset_to_canonical, 0.0);
        assert_eq!(point.quantity.key.scale_kind, pse_quantity::ScaleKind::Point);
        assert_eq!(difference.quantity.key.scale_kind, pse_quantity::ScaleKind::Difference);
        assert_eq!(point.quantity_kind.dimension, difference.quantity_kind.dimension);
        assert!(context.describe(&request(PhysicalInspectionQuery::Quantity { id: QuantityTypeId::from_id(celsius.as_id()), unit: None })).is_err());
        assert!(context.describe(&request(PhysicalInspectionQuery::Quantity { id: point.quantity.id, unit: Some(context.resolve_unit("s").unwrap()) })).is_err());
    }
    #[test]
    fn physical_projection_preserves_distinct_basis_and_datum_with_equal_dimensions() {
        let context = super::super::tests::physical();
        let quantities = context.quantities().quantity_types().collect::<Vec<_>>();
        let (left, right) = quantities.iter().flat_map(|a| quantities.iter().map(move |b| (*a, *b))).find(|(a, b)| a.key.kind == b.key.kind && (a.key.basis != b.key.basis || a.key.reference_state != b.key.reference_state)).unwrap();
        let a = one(&context, left.id, None);
        let b = one(&context, right.id, None);
        assert_eq!(a.quantity_kind.dimension, b.quantity_kind.dimension);
        assert_ne!((a.quantity.key.basis, a.quantity.key.reference_state), (b.quantity.key.basis, b.quantity.key.reference_state));
        assert_eq!(a.basis.as_ref().map(|basis| basis.id), left.key.basis);
        assert_eq!(b.reference_state.as_ref().map(|state| state.id), right.key.reference_state);
    }
    #[test]
    fn physical_projection_pages_retain_pool_custody_and_do_not_clone_the_registry() {
        let context = super::super::tests::physical();
        let baseline = context.pool.reserved();
        let mut query = PhysicalInspectionRequest { query: PhysicalInspectionQuery::Quantities, offset: 0, limit: 16 };
        let page = context.describe(&query).unwrap();
        assert!(context.pool.reserved() > baseline);
        let PhysicalInspectionDocument::Page { quantities, next, count, .. } = &page else { panic!("bounded page") };
        assert_eq!(quantities.len(), 16);
        assert!(*count > 16);
        query.offset = next.unwrap();
        let PhysicalInspectionDocument::Page { quantities: more, .. } = context.describe(&query).unwrap() else { panic!("next bounded page") };
        assert!(quantities.iter().all(|first| more.iter().all(|second| first.quantity.id != second.quantity.id)));
        drop(more);
        drop(page);
        assert_eq!(context.pool.reserved(), baseline);
        query.limit = 17;
        assert!(context.describe(&query).is_err());
    }
    fn synthetic(seed: u8, entity_name: &str) -> PhysicalContext {
        use pse_quantity::*;
        let id = pse_ids::SemanticId::from_bytes([seed; 16]);
        let mut builder = QuantityRegistryBuilder::new();
        builder.entity_kind(EntityKind { id: id.into(), name: entity_name.into() });
        builder.unit(Unit { id: id.into(), symbol: "shared".into(), dimension: DimensionVector::DIMENSIONLESS, scale_to_canonical: 1., offset_to_canonical: 0., is_affine: false, reference_state: None, definition: None });
        builder.kind(QuantityKind { id: id.into(), dimension: DimensionVector::DIMENSIONLESS, extensive: false, addition_kind: QuantityAdditionKind::Additive, category: None, definition: None });
        builder.quantity_type(QuantityType { id: id.into(), name: Some("Shared".into()), key: QuantityTypeKey { kind: id.into(), basis: None, reference_state: None, scale_kind: ScaleKind::Point, shape: vec![id.into()], subject_kind: Some(id.into()) }, canonical_unit: id.into(), nominal_magnitude: None });
        let mut context = super::super::tests::physical();
        context.quantities = Arc::new(builder.build().unwrap());
        context.preconditions = Arc::new(PhysicalPreconditions::new(vec![]).unwrap());
        context.key = pse_compiler::workspace::physical_identity(&context.quantities, &context.preconditions);
        context
    }
    #[test]
    fn physical_projection_names_are_selected_context_local_and_ambiguous_admission_refuses() {
        let left = synthetic(71, "left.axis");
        let right = synthetic(72, "right.axis");
        assert_ne!(left.identity(), right.identity());
        assert_ne!(left.resolve_quantity("Shared").unwrap(), right.resolve_quantity("Shared").unwrap());
        assert_ne!(left.resolve_unit("shared").unwrap(), right.resolve_unit("shared").unwrap());
        assert!(left.describe(&request(PhysicalInspectionQuery::Quantity { id: right.resolve_quantity("Shared").unwrap(), unit: None })).is_err());
        let mut builder = left.quantities().to_builder();
        let mut duplicate = left.quantities().quantity_types().next().unwrap().clone();
        duplicate.id = pse_ids::SemanticId::from_bytes([73; 16]).into();
        duplicate.key.scale_kind = pse_quantity::ScaleKind::Difference;
        builder.quantity_type(duplicate);
        assert!(builder.build().is_err(), "duplicate declared names fail admission before resolution");
    }
    #[test]
    fn physical_projection_refuses_oversized_nested_record_before_copy() {
        let context = synthetic(74, &"a".repeat(70_000));
        let baseline = context.pool.reserved();
        assert!(context.describe(&request(PhysicalInspectionQuery::QuantityNamed { name: "Shared".into() })).is_err());
        assert_eq!(context.pool.reserved(), baseline);
    }
    #[test]
    fn physical_projection_refuses_oversized_names_before_query_copy_or_pool_admission() {
        let context = super::super::tests::physical();
        let baseline = context.pool.reserved();
        for query in [PhysicalInspectionQuery::QuantityNamed { name: "x".repeat(1025) }, PhysicalInspectionQuery::ReferenceStateNamed { name: "x".repeat(1025) }] {
            let request = request(query);
            assert!(!request.valid_bounds());
            assert!(context.describe(&request).is_err());
            assert_eq!(context.pool.reserved(), baseline);
        }
        assert!(context.resolve_quantity(&"x".repeat(1025)).is_err());
        assert!(context.resolve_reference_state(&"x".repeat(1025)).is_err());
    }
}
