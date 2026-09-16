// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated columns retain reserve-before-growth ownership across detached readers.

use pse_ids::{CancellationToken, ContentHash, FixedBudget, SemanticId};
use pse_relations::columnar::Collection;
use pse_relations::generated::{
    authored::packages::{self, AuthoredPackagesFieldDependenciesItem},
    compiled::symbol_expressions,
    enums::{IdPolicy, PackageKind},
    reference::units,
};

fn package() -> packages::Row {
    packages::Row {
        package_id: SemanticId::from_bytes([1; 16]),
        name: "actual package".to_owned(),
        version: "1.0.0".to_owned(),
        kind: PackageKind::Model,
        id_policy: IdPolicy::Explicit,
        dependencies: vec![AuthoredPackagesFieldDependenciesItem {
            package_id: SemanticId::from_bytes([2; 16]),
            version_req: "=2.3.4".to_owned(),
        }],
        content_hash: ContentHash::NIL,
        doc: "nested payload".to_owned(),
    }
}

#[test]
fn typed_collection_keeps_distinct_schemas_and_explicit_empty_relations()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = pse_schema::registry()?;
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::new();
    let mut collection = Collection::new(registry, budget.as_ref(), &cancel);
    collection.ensure::<units::Row>()?;
    collection.push(package())?;
    let expression = symbol_expressions::Row {
        symbol_id: SemanticId::from_bytes([3; 16]),
        node_id: 17,
        derivation_id: SemanticId::from_bytes([4; 16]),
    };
    collection.push(expression.clone())?;
    let batches = collection.finish()?;
    assert_eq!(batches.len(), 3);
    assert_eq!(
        packages::View::from_checked(&batches[&packages::spec(registry)?.key])?.rows()?,
        vec![package()],
    );
    assert_eq!(
        symbol_expressions::View::from_checked(&batches[&symbol_expressions::spec(registry)?.key])?
            .rows()?,
        vec![expression],
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
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::new();
    let mut collection = Collection::new(registry, budget.as_ref(), &cancel);
    collection.push(package())?;
    let batches = collection.finish()?;
    let buffer = batches[&packages::spec(registry)?.key]
        .batch()
        .column(5)
        .to_data()
        .child_data()[0]
        .child_data()[1]
        .buffers()[1]
        .clone();
    let held = budget.reserved();
    drop(batches);
    assert_eq!(budget.reserved(), held);
    assert_eq!(buffer.as_slice(), b"=2.3.4");
    drop(buffer);
    assert_eq!(budget.reserved(), 0);
    Ok(())
}

#[test]
fn checked_reuse_and_single_input_concat_share_the_existing_allocation_claim()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = pse_schema::registry()?;
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::new();
    let mut collection = Collection::new(registry, budget.as_ref(), &cancel);
    collection.push(package())?;
    let batches = collection.finish()?;
    let spec = packages::spec(registry)?;
    let original = &batches[&spec.key];
    let held = budget.reserved();
    let reused = original.retained(budget.as_ref(), &cancel)?;
    let sliced = reused.slice(0, 1)?.retained(budget.as_ref(), &cancel)?;
    let concatenated = pse_relations::columnar::FieldCheckedBatch::concat_reserved(
        registry,
        spec,
        &[sliced],
        budget.as_ref(),
        &cancel,
    )?;
    assert_eq!(budget.reserved(), held);
    drop(batches);
    drop(reused);
    assert_eq!(budget.reserved(), held);
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
    let cancel = CancellationToken::new();
    let denied = FixedBudget::new(0);
    let mut collection = Collection::new(registry, denied.as_ref(), &cancel);
    assert!(collection.push(package()).is_err());
    drop(collection);
    assert_eq!(denied.reserved(), 0);

    let budget = FixedBudget::new(64 << 20);
    let mut collection = Collection::new(registry, budget.as_ref(), &cancel);
    collection.push(package())?;
    cancel.cancel();
    assert!(collection.finish().is_err());
    assert_eq!(budget.reserved(), 0);
    Ok(())
}
