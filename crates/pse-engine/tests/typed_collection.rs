// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated columns retain reserve-before-growth ownership across detached readers.

use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, SemanticId};
use pse_relations::columnar::Collection;
use pse_relations::generated::{
    authored::package_unit_sets as unit_sets,
    authored::packages::{self, AuthoredPackagesFieldDependenciesItem},
    enums::{IdPolicy, PackageKind},
    reference::units,
};

fn package() -> packages::Row {
    packages::Row {
        package_id: SemanticId::from_bytes([1; 16]).into(),
        name: "actual package".to_owned(),
        version: "1.0.0".to_owned(),
        kind: PackageKind::Model,
        id_policy: IdPolicy::Explicit,
        dependencies: vec![AuthoredPackagesFieldDependenciesItem {
            package_id: SemanticId::from_bytes([2; 16]).into(),
            version_req: pse_model::generated::structures::VersionRequirement {
                operator: pse_model::generated::enums::ModelingVersionOperator::Exact,
                major: 2,
                minor: 3,
                patch: 4,
            },
        }],
        content_hash: ContentHash::NIL,
        doc: "nested payload".to_owned(),
    }
}

#[test]
fn typed_phase_roundtrip_preserves_occurrences_empty_membership_and_budget_ownership()
-> Result<(), Box<dyn std::error::Error>> {
    use pse_model::generated::facts::FactBatch;
    use pse_relations::generated::facts::{decode, encode};
    let registry = pse_schema::registry()?;
    let validation = fixture_validation(registry);
    let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let cancel = CancellationToken::new();
    let mut second = package();
    second.name = "second occurrence".into();
    let values = FactBatch::AuthoredPackages(vec![second, package(), package()]);
    let checked = encode(&values, registry, &validation, &pool, &cancel)?;
    assert_eq!(decode(&checked)?, values);
    let detached = checked.batch().column(1).clone();
    drop(checked);
    assert!(pool.reserved() > 0);
    drop(detached);
    assert_eq!(pool.reserved(), 0);
    let empty = FactBatch::ReferenceUnits(vec![]);
    let checked = encode(&empty, registry, &validation, &pool, &cancel)?;
    assert_eq!(decode(&checked)?, empty);
    drop(checked);
    assert_eq!(pool.reserved(), 0);
    let denied: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(0));
    assert!(encode(&values, registry, &validation, &denied, &cancel).is_err());
    assert_eq!(denied.reserved(), 0);
    cancel.cancel();
    assert!(encode(&values, registry, &validation, &pool, &cancel).is_err());
    assert_eq!(pool.reserved(), 0);
    Ok(())
}

#[test]
fn typed_collection_keeps_distinct_schemas_and_explicit_empty_relations()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = pse_schema::registry()?;
    let validation = fixture_validation(registry);
    let budget: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let cancel = CancellationToken::new();
    let mut collection = Collection::new(registry, &budget, &cancel, &validation);
    collection.ensure::<units::Row>()?;
    collection.push(package())?;
    let selection = unit_sets::Row {
        package_id: SemanticId::from_bytes([1; 16]).into(),
        unit_set_id: SemanticId::from_bytes([4; 16]),
    };
    collection.push(selection.clone())?;
    let batches = collection.finish()?;
    assert_eq!(batches.len(), 3);
    assert_eq!(
        packages::View::from_checked(&batches[&packages::spec(registry)?.key])?.rows()?,
        vec![package()],
    );
    assert_eq!(
        unit_sets::View::from_checked(&batches[&unit_sets::spec(registry)?.key])?.rows()?,
        vec![selection],
    );
    assert_eq!(batches[&units::spec(registry)?.key].batch().num_rows(), 0);
    assert!(budget.reserved() > 0);
    drop(batches);
    assert_eq!(budget.reserved(), 0);
    Ok(())
}

#[test]
fn detached_nested_buffer_retains_the_construction_claim() -> Result<(), Box<dyn std::error::Error>>
{
    let registry = pse_schema::registry()?;
    let validation = fixture_validation(registry);
    let budget: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let cancel = CancellationToken::new();
    let mut collection = Collection::new(registry, &budget, &cancel, &validation);
    collection.push(package())?;
    let batches = collection.finish()?;
    let buffer = batches[&packages::spec(registry)?.key]
        .batch()
        .column(5)
        .to_data()
        .child_data()[0]
        .child_data()[1]
        // The typed requirement's operator, a string enum column (ADR-0123 Outcome 7).
        .child_data()[0]
        .buffers()[1]
        .clone();
    let held = budget.reserved();
    drop(batches);
    assert!(budget.reserved() >= buffer.len() && budget.reserved() < held);
    assert_eq!(buffer.as_slice(), b"exact");
    drop(buffer);
    assert_eq!(budget.reserved(), 0);
    Ok(())
}

#[test]
fn checked_reuse_and_single_input_concat_share_the_existing_allocation_claim()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = pse_schema::registry()?;
    let validation = fixture_validation(registry);
    let budget: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let cancel = CancellationToken::new();
    let mut collection = Collection::new(registry, &budget, &cancel, &validation);
    collection.push(package())?;
    let batches = collection.finish()?;
    let spec = packages::spec(registry)?;
    let original = &batches[&spec.key];
    let held = budget.reserved();
    let reused = original.retained(&budget, &cancel)?;
    assert_eq!(budget.reserved(), held);
    let slice = reused.slice(0, 1)?;
    let slice_held = budget.reserved();
    assert!(slice_held > held, "the new slice owns its metadata");
    let sliced = slice.retained(&budget, &cancel)?;
    assert_eq!(budget.reserved(), slice_held);
    drop(slice);
    let concatenated = pse_relations::columnar::FieldCheckedBatch::concat_reserved(
        registry,
        spec,
        std::slice::from_ref(&sliced),
        &budget,
        &cancel,
    )?;
    assert_eq!(budget.reserved(), slice_held);
    for (source, result) in sliced
        .batch()
        .columns()
        .iter()
        .zip(concatenated.batch().columns())
    {
        assert!(std::sync::Arc::ptr_eq(source, result));
    }
    drop(batches);
    drop(reused);
    drop(sliced);
    assert!(budget.reserved() > 0 && budget.reserved() < slice_held);
    assert_eq!(
        packages::View::from_checked(&concatenated)?.rows()?,
        vec![package()]
    );
    drop(concatenated);
    assert_eq!(budget.reserved(), 0);
    Ok(())
}

#[test]
fn denied_growth_and_cancellation_release_all_construction_claims()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = pse_schema::registry()?;
    let validation = fixture_validation(registry);
    let cancel = CancellationToken::new();
    let denied: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(0));
    let mut collection = Collection::new(registry, &denied, &cancel, &validation);
    assert!(collection.push(package()).is_err());
    drop(collection);
    assert_eq!(denied.reserved(), 0);

    let budget: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let mut collection = Collection::new(registry, &budget, &cancel, &validation);
    collection.push(package())?;
    cancel.cancel();
    assert!(collection.finish().is_err());
    assert_eq!(budget.reserved(), 0);
    Ok(())
}

// Deliberate native context for these isolated generated-contract fixtures.
fn fixture_validation(
    registry: &pse_schema::Registry,
) -> std::sync::Arc<pse_relations::validate::ValidationContext> {
    std::sync::Arc::new(pse_relations::validate::ValidationContext::new(
        registry,
        pse_engine::validation::NativeValidation(
            datafusion::prelude::SessionContext::new().state(),
        ),
    ))
}
