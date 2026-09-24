// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual package columns enter one native physical inventory and conversion plan.
#![allow(clippy::unwrap_used, reason = "explicit native physical fixtures")]
use datafusion::{
    arrow::array::Float64Array,
    logical_expr::{LogicalPlanBuilder, lit},
};
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineSession, ExecutionSettings, ThreadBudget};
use pse_ids::SemanticId;
use pse_quantity::standard::ids;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{normalized, reference},
};
use pse_runtime::physical::{PhysicalInventory, input_keys};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, num::NonZeroUsize, path::Path, sync::Arc};

fn inputs(registry: &Registry) -> BTreeMap<RelationKey, FieldCheckedBatch> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let bundles = ["elements", "physical", "fixture-currency"].map(|name| {
        pse_runtime::authoring_driver::document::load_package(
            &root.join("packages/reference").join(name),
            registry,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap()
    });
    input_keys(registry)
        .into_iter()
        .filter_map(|key| {
            let spec = registry.relation(&key.qualified_name()).unwrap();
            let parts = bundles
                .iter()
                .filter_map(|bundle| bundle.batches.get(&spec.id))
                .cloned()
                .collect::<Vec<_>>();
            (!parts.is_empty()).then(|| {
                (
                    key,
                    FieldCheckedBatch::concat(registry, spec, &parts).unwrap(),
                )
            })
        })
        .collect()
}
fn session(
    registry: &Arc<Registry>,
    rows: BTreeMap<RelationKey, FieldCheckedBatch>,
    cancel: &CancellationToken,
) -> EngineSession {
    let one = NonZeroUsize::new(1).unwrap();
    pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(8usize << 30)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
    )
    .unwrap()
    .candidate_checked(rows, Arc::clone(registry), cancel)
    .unwrap()
}

#[tokio::test]
async fn actual_reference_inventory_retains_explicit_context_and_full_physical_types() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let session = session(&registry, inputs(&registry), &cancel);
    let actual = PhysicalInventory::load(&session, &registry, &cancel)
        .await
        .unwrap();
    assert_eq!(actual.neutral(), Some(ids::quantity("neutral")));
    let expected = pse_quantity::standard::standard_registry().unwrap();
    for quantity in expected.quantity_types() {
        let found = actual.quantities().quantity_type(quantity.id).unwrap();
        assert_eq!(found.key, quantity.key);
        assert_eq!(found.canonical_unit, quantity.canonical_unit);
    }
    assert!(actual.elements().elements().len() > 0);
    assert!(!actual.preconditions().is_empty());
}

#[tokio::test]
async fn native_conversion_uses_point_or_difference_and_exact_selected_units() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let session = session(&registry, inputs(&registry), &cancel);
    let physical = PhysicalInventory::load(&session, &registry, &cancel)
        .await
        .unwrap();
    let mut difference_key = physical
        .quantities()
        .quantity_type(ids::quantity("temperature.point"))
        .unwrap()
        .key
        .clone();
    difference_key.scale_kind = pse_quantity::ScaleKind::Difference;
    let difference_type = physical.quantities().resolve_key(&difference_key).unwrap();
    let plan = LogicalPlanBuilder::empty(true)
        .project([
            physical
                .conversion_expr(
                    lit(0_f64),
                    ids::unit("degC"),
                    ids::unit("K"),
                    ids::quantity("temperature.point"),
                )
                .unwrap()
                .alias("point"),
            physical
                .conversion_expr(
                    lit(18_f64),
                    ids::unit("degF"),
                    ids::unit("K"),
                    difference_type,
                )
                .unwrap()
                .alias("difference"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let result = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let batch = &result.batches()[0];
    let point = batch
        .column(0)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap()
        .value(0);
    let difference = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap()
        .value(0);
    // NIST SP 811 temperature conversion: t/K = t/degC + 273.15;
    // an 18 Fahrenheit-degree interval is 10 kelvin, with no point offset.
    assert_eq!(point.to_bits(), 273.15_f64.to_bits());
    assert!((difference - 10.).abs() < 1e-13);
    assert!(
        physical
            .conversion_expr(
                lit(1_f64),
                ids::unit("degC"),
                ids::unit("K"),
                ids::quantity("pressure.absolute")
            )
            .is_err()
    );
}

#[tokio::test]
async fn mixture_normalization_preserves_differences_and_requires_actual_fraction_contracts() {
    use pse_quantity::{
        IndexSet, QuantityTypeId,
        infer::{NoInvariantFacts, OpRequest, Operand, infer_with_evidence},
    };

    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let session = session(&registry, inputs(&registry), &cancel);
    let physical = PhysicalInventory::load(&session, &registry, &cancel)
        .await
        .unwrap();
    let quantity = |id| QuantityTypeId::from_id(SemanticId::parse_hex(id).unwrap());
    let difference = quantity("d5bb3d48b9804f2f8d5a6f0a7cadaee8");
    let fraction = quantity("4f842bb63da53a798dca2764c2c64ece");
    let indices = IndexSet::new();
    let infer = |left, right| {
        infer_with_evidence(
            &OpRequest::Div,
            &[
                Operand {
                    quantity_type: left,
                    indices: &indices,
                },
                Operand {
                    quantity_type: right,
                    indices: &indices,
                },
            ],
            physical.quantities(),
            physical.precondition_checker(),
        )
    };
    let weighted = infer_with_evidence(
        &OpRequest::Mul,
        &[fraction, quantity("0f83c0a2bffb4cd09c7f727ae6c42d20")].map(|quantity_type| Operand {
            quantity_type,
            indices: &indices,
        }),
        physical.quantities(),
        physical.precondition_checker(),
    )
    .unwrap()
    .result;
    assert_eq!(weighted, difference);
    assert_eq!(infer(weighted, fraction).unwrap().result, difference);
    assert_eq!(
        infer(fraction, quantity("11a07d5c43da49afbf360f3b404ce9ca"))
            .unwrap()
            .result,
        quantity("b6edcb08fc072d81a9f99b45f7f9c90a")
    );
    // Normalization divides a weighted enthalpy difference. Dividing an
    // origin-sensitive point directly would move its thermodynamic datum.
    assert!(infer(quantity("1831d0d72dc74b299ba8ecb6d4da6f53"), fraction).is_err());
    assert!(infer(quantity("94b88c5bead5458629c2afc11ffcff20"), fraction).is_err());
    assert!(infer(quantity("0f83c0a2bffb4cd09c7f727ae6c42d20"), fraction).is_err());
    assert!(infer(quantity("899354057d474a1ab027860e1b3a18ea"), fraction).is_err());
    assert!(infer(fraction, difference).is_err());
    assert!(
        infer_with_evidence(
            &OpRequest::Div,
            &[difference, fraction].map(|quantity_type| Operand {
                quantity_type,
                indices: &indices,
            }),
            physical.quantities(),
            &NoInvariantFacts,
        )
        .is_err()
    );
}

#[tokio::test]
async fn energy_density_cancels_molar_basis_and_preserves_thermochemical_datum() {
    use pse_quantity::{
        IndexSet, QuantityTypeId,
        infer::{OpRequest, Operand, infer_with_evidence},
    };
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let session = session(&registry, inputs(&registry), &cancel);
    let physical = PhysicalInventory::load(&session, &registry, &cancel)
        .await
        .unwrap();
    let quantity = |id| QuantityTypeId::from_id(SemanticId::parse_hex(id).unwrap());
    let indices = IndexSet::new();
    let density = quantity("b6edcb08fc072d81a9f99b45f7f9c90a");
    let enthalpy = quantity("1831d0d72dc74b299ba8ecb6d4da6f53");
    let infer = |left, right| {
        infer_with_evidence(
            &OpRequest::Mul,
            &[left, right].map(|quantity_type| Operand {
                quantity_type,
                indices: &indices,
            }),
            physical.quantities(),
            physical.precondition_checker(),
        )
    };
    let result = infer(density, enthalpy).unwrap().result;
    assert_eq!(result, quantity("e4af547ff9774ed9b71a903c86b43aea"));
    assert_eq!(infer(enthalpy, density).unwrap().result, result);
    let actual = &physical.quantities().quantity_type(result).unwrap().key;
    assert_eq!(actual.basis, None);
    assert_eq!(
        actual.reference_state,
        physical
            .quantities()
            .quantity_type(enthalpy)
            .unwrap()
            .key
            .reference_state
    );
    assert!(infer(density, quantity("553e33a36c6245619508d88d585c148b")).is_err());
    assert!(infer(density, quantity("d5bb3d48b9804f2f8d5a6f0a7cadaee8")).is_err());
}

#[tokio::test]
async fn inlet_pressure_equality_has_an_indexed_absolute_difference_contract() {
    use pse_quantity::{
        BoundIndexId, BoundIndexRef, DomainId, DomainKind, IndexSet, QuantityTypeId, ScaleKind,
        infer::{OpRequest, Operand, infer},
    };
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let session = session(&registry, inputs(&registry), &cancel);
    let physical = PhysicalInventory::load(&session, &registry, &cancel)
        .await
        .unwrap();
    let quantity = |id| QuantityTypeId::from_id(SemanticId::parse_hex(id).unwrap());
    let pressure = quantity("53279f3ced4148358959c5cfcc1c2843");
    let index = BoundIndexRef::new(
        BoundIndexId::from_id(SemanticId::from_bytes([1; 16])),
        DomainId::from_id(SemanticId::from_bytes([2; 16])),
        DomainKind::PortSet,
    );
    let indices = IndexSet::try_from_iter([index]).unwrap();
    let operand = Operand {
        quantity_type: pressure,
        indices: &indices,
    };
    let residual = infer(&OpRequest::Sub, &[operand, operand], physical.quantities()).unwrap();
    let key = &physical
        .quantities()
        .quantity_type(residual.result)
        .unwrap()
        .key;
    assert_eq!(
        residual.result,
        quantity("4b2f172f8eaf440e983e739f9d22e0ae")
    );
    assert_eq!(residual.indices, indices);
    assert_eq!(key.reference_state, None);
    assert_eq!(key.scale_kind, ScaleKind::Difference);
    assert_eq!(key.shape, [DomainKind::PortSet]);
    let other = IndexSet::try_from_iter([BoundIndexRef {
        bound_index: BoundIndexId::from_id(SemanticId::from_bytes([3; 16])),
        ..index
    }])
    .unwrap();
    assert!(
        infer(
            &OpRequest::Sub,
            &[
                operand,
                Operand {
                    indices: &other,
                    ..operand
                }
            ],
            physical.quantities()
        )
        .is_err()
    );
}

#[tokio::test]
async fn normalized_unit_union_requires_complete_exact_definitions() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let original = inputs(&registry);
    let unit_spec = reference::units::spec(&registry).unwrap();
    let unit = reference::units::View::from_checked(&original[&unit_spec.key])
        .unwrap()
        .row(0)
        .unwrap();
    for changed in [false, true] {
        let mut rows = original.clone();
        let mut builder = normalized::units::Builder::with_registry(&registry, 1).unwrap();
        builder
            .push(normalized::units::Row {
                unit_id: unit.unit_id,
                symbol: unit.symbol.clone(),
                name: unit.name.clone(),
                dimension: unit.dimension.clone(),
                scale_to_canonical: if changed {
                    unit.scale_to_canonical * 2.
                } else {
                    unit.scale_to_canonical
                },
                offset_to_canonical: unit.offset_to_canonical,
                is_affine: unit.is_affine,
                reference_state_id: unit.reference_state_id,
                system: unit.system.clone(),
                doc: unit.doc.clone(),
                package_id: SemanticId::NIL,
                unit_set_id: SemanticId::NIL,
            })
            .unwrap();
        let spec = normalized::units::spec(&registry).unwrap();
        rows.insert(spec.key, builder.finish().unwrap());
        let session = session(&registry, rows, &cancel);
        let result = PhysicalInventory::load(&session, &registry, &cancel).await;
        assert_eq!(result.is_ok(), !changed);
    }
}

#[tokio::test]
async fn missing_context_is_not_inferred_and_cancellation_propagates() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let mut rows = inputs(&registry);
    rows.remove(&reference::math_context::spec(&registry).unwrap().key);
    let session = session(&registry, rows, &cancel);
    let physical = PhysicalInventory::load(&session, &registry, &cancel)
        .await
        .unwrap();
    assert!(physical.neutral().is_none());
    assert!(physical.boolean().is_none());
    cancel.cancel();
    assert!(
        PhysicalInventory::load(&session, &registry, &cancel)
            .await
            .is_err()
    );
}
