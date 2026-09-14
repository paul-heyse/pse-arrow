// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Unit-only packages are admitted from actual rows without guessed physical types.
use pse_compiler::{
    passes::p10::fixture::{physical_rows, unit_rows},
    quantity_relations::{decode_unit_sets, decode_units},
};
use pse_ids::SemanticId;
use pse_quantity::{BaseDimension, DimensionVector, UnitId};
use pse_schema::model::Cell;
fn sid(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
#[test]
fn unit_only_rows_resolve_base_dimensions_without_any_neutral_quantity() {
    let registry = pse_schema::catalog::assemble().unwrap();
    let rows = unit_rows(&registry).unwrap();
    let decoded = decode_units(&rows, &registry).unwrap();
    let sets = decode_unit_sets(&rows, &registry, &decoded).unwrap();
    assert_eq!(decoded.len(), 8);
    assert_eq!(sets.len(), 1);
    let length = DimensionVector::base(BaseDimension::Length);
    let derived = sets
        .values()
        .next()
        .unwrap()
        .derived_unit_from_units(length, &decoded)
        .unwrap();
    assert_eq!(derived.dimension, length);
    assert_eq!(derived.scale_to_canonical.to_bits(), 1.0_f64.to_bits());
    let mut changed = decoded.clone();
    changed
        .get_mut(&UnitId::from_id(sid(1)))
        .unwrap()
        .scale_to_canonical = 100.;
    assert!(decode_unit_sets(&rows, &registry, &changed).is_err());
}
#[test]
fn altered_values_duplicate_ids_and_wrong_base_axes_are_rejected_from_actual_rows() {
    let registry = pse_schema::catalog::assemble().unwrap();
    let mut rows = unit_rows(&registry).unwrap();
    let spec = registry.relation("reference.units").unwrap();
    let initial =
        pse_relations::cells::cells_from_batch(&registry, spec, &rows[&spec.key]).unwrap();
    let mut bad = initial.clone();
    bad[0][4] = Cell::F64(0.);
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &bad).unwrap(),
    );
    assert!(decode_units(&rows, &registry).is_err());
    let mut duplicate = initial.clone();
    duplicate.push(initial[0].clone());
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &duplicate).unwrap(),
    );
    assert!(decode_units(&rows, &registry).is_err());
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &initial).unwrap(),
    );
    let decoded = decode_units(&rows, &registry).unwrap();
    let sets = registry.relation("reference.unit_sets").unwrap();
    let mut values =
        pse_relations::cells::cells_from_batch(&registry, sets, &rows[&sets.key]).unwrap();
    values[0][2] = Cell::Id(sid(1));
    rows.insert(
        sets.key,
        pse_relations::cells::batch_from_cells(&registry, sets, &values).unwrap(),
    );
    assert!(decode_unit_sets(&rows, &registry, &decoded).is_err());
}

#[test]
fn complete_registry_uses_explicit_neutral_and_revalidates_actual_contracts() {
    use pse_compiler::quantity_relations::decode_quantity_registry;
    use pse_quantity::QuantityTypeId;
    let registry = pse_schema::catalog::assemble().unwrap();
    let mut rows = physical_rows(&registry).unwrap();
    let neutral = QuantityTypeId::from_id(sid(31));
    let decoded = decode_quantity_registry(&rows, &registry, Some(neutral)).unwrap();
    assert_eq!(decoded.neutral_dimensionless(), Some(neutral));
    assert!(
        decode_quantity_registry(&rows, &registry, None)
            .unwrap()
            .neutral_dimensionless()
            .is_none()
    );
    assert!(
        decode_quantity_registry(&rows, &registry, Some(QuantityTypeId::from_id(sid(30)))).is_err()
    );
    let spec = registry.relation("reference.quantity_types").unwrap();
    let mut values =
        pse_relations::cells::cells_from_batch(&registry, spec, &rows[&spec.key]).unwrap();
    values[0][7] = Cell::Id(sid(2));
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &values).unwrap(),
    );
    assert!(decode_quantity_registry(&rows, &registry, Some(neutral)).is_err());
}

#[test]
fn p10_fixture_binds_complete_actual_inputs_and_rejects_stale_type_claims() {
    use pse_compiler::passes::p10::{Context, canonicalize_rows, fixture};
    use pse_ids::{CancellationToken, FixedBudget};
    let registry = fixture::registry().unwrap();
    let (mut rows, context) = fixture::arithmetic_inputs(&registry).unwrap();
    let root = context.roots[0];
    let cancel = CancellationToken::default();
    let budget = FixedBudget::new(512 << 20);
    let output = canonicalize_rows(&rows, &context, &registry, budget.as_ref(), &cancel).unwrap();
    let spec = registry.relation("compiled.math_int_constants").unwrap();
    let values =
        pse_relations::cells::cells_from_batch(&registry, spec, &output.rows[&spec.key]).unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0][1], Cell::I64(3));
    assert_eq!(output.roots.len(), 1);
    let no_neutral = Context {
        neutral: None,
        ..context.clone()
    };
    assert!(canonicalize_rows(&rows, &no_neutral, &registry, budget.as_ref(), &cancel).is_err());
    let spec = registry.relation("compiled.math_expr_nodes").unwrap();
    let mut values =
        pse_relations::cells::cells_from_batch(&registry, spec, &rows[&spec.key]).unwrap();
    let quantity = spec
        .columns
        .iter()
        .position(|column| column.name == "quantity_type_id")
        .unwrap();
    let node = spec
        .columns
        .iter()
        .position(|column| column.name == "node_id")
        .unwrap();
    values
        .iter_mut()
        .find(|row| row[node] == Cell::U64(root.0))
        .unwrap()[quantity] = Cell::Id(sid(30));
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(&registry, spec, &values).unwrap(),
    );
    assert!(canonicalize_rows(&rows, &context, &registry, budget.as_ref(), &cancel).is_err());
    assert!(
        canonicalize_rows(
            &rows,
            &context,
            &registry,
            FixedBudget::new(1).as_ref(),
            &cancel
        )
        .is_err()
    );
    cancel.cancel();
    assert!(canonicalize_rows(&rows, &context, &registry, budget.as_ref(), &cancel).is_err());
}

#[test]
fn full_fixture_model_uses_actual_source_loader_and_complete_registry_membership() {
    use pse_compiler::passes::p10::fixture;
    use pse_schema::model::SnapshotClass;
    let registry = fixture::registry().unwrap();
    let rows = fixture::model_inputs(&registry).unwrap();
    let package = fixture::physical_package(&registry).unwrap();
    assert_eq!(
        rows.len(),
        registry
            .relations()
            .iter()
            .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
            .count()
    );
    for (id, expected) in &package.rows {
        let spec = registry.relation_by_id(*id).unwrap();
        assert_eq!(
            pse_relations::cells::cells_from_batch(&registry, spec, &rows[&spec.key]).unwrap(),
            *expected
        );
    }
    let entities = registry.relation("authored.entities").unwrap();
    assert_eq!(
        rows[&entities.key].num_rows(),
        12,
        "nine units, one unit set and two quantity kinds"
    );
    let context = fixture::context(&rows, &registry).unwrap();
    let physical = pse_compiler::quantity_relations::decode_quantity_registry(
        &rows,
        &registry,
        context.neutral,
    )
    .unwrap();
    assert_eq!(physical.neutral_dimensionless(), context.neutral);
}
