// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Every observed object-write boundary preserves the previous complete visible model.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "bounded test fixture construction reports direct assertion failures"
)]

#[path = "../src/fault_store.rs"]
mod fault_store;
mod support;

use fault_store::{Fault, FaultPlan, FaultStore};
use pse_catalog::store::membership::AdmissionContext;
use pse_catalog::store::publish::{BundleDraft, RelationDraft};
use pse_catalog::{Catalog, EncodingPolicy, RefName, RelationContract};
use pse_ids::{CancellationToken, FixedBudget, SnapshotKind};
use pse_schema::model::{Authority, Cell, FieldContract, Namespace, RelationDecl, SnapshotClass};
use pse_schema::{Registry, RegistryBuilder};
use std::collections::BTreeMap;
use std::sync::Arc;

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    for name in ["first", "second", "third"] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Multi-object publication fixture",
            )
            .pk(&["id"])
            .columns(vec![
                FieldContract::key(
                    "id",
                    FieldContract::native(arrow::datatypes::DataType::UInt64),
                    "Key",
                ),
                FieldContract::payload(
                    "value",
                    FieldContract::native(arrow::datatypes::DataType::UInt64),
                    "Value",
                ),
            ]),
        );
    }
    Arc::new(builder.build().expect("three declared relations"))
}

fn draft(catalog: &Catalog, value: u64, cancel: &CancellationToken) -> BundleDraft {
    let context = AdmissionContext::default();
    let relations = catalog
        .registry()
        .relations()
        .iter()
        .map(|spec| {
            let batch = pse_relations::cells::batch_from_cells_owned(
                catalog.registry(),
                spec,
                &[vec![Cell::U64(1), Cell::U64(value)]],
                catalog.reserver().as_ref(),
                cancel,
            )
            .expect("reserved rows");
            (
                pse_ids::model_port_name("authored", spec.id),
                RelationDraft {
                    contract: Arc::new(
                        RelationContract::from_spec(
                            catalog.registry(),
                            spec,
                            EncodingPolicy::IpcFileAndParquet,
                        )
                        .expect("dual encoding contract"),
                    ),
                    batches: vec![batch],
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    BundleDraft {
        manifest: catalog
            .manifest_template(SnapshotKind::Model, &context)
            .expect("manifest"),
        relations,
        context,
    }
}

async fn successful_trace() -> Vec<String> {
    let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    let budget = FixedBudget::new(64 << 20);
    let catalog = support::catalog(store.clone(), registry(), budget.clone());
    let cancel = CancellationToken::default();
    let name = RefName::parse("main").expect("name");
    let old = catalog
        .publish_bundle(draft(&catalog, 1, &cancel), &cancel)
        .await
        .expect("old");
    catalog
        .compare_and_swap_ref(&name, None, &old, &cancel)
        .await
        .expect("initial head");
    let observed = catalog
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("head");
    store.clear_trace();
    let next = catalog
        .publish_bundle(draft(&catalog, 99, &cancel), &cancel)
        .await
        .expect("next");
    catalog
        .compare_and_swap_ref(&name, Some(&observed), &next, &cancel)
        .await
        .expect("CAS");
    let trace = store.put_trace();
    assert_eq!(
        trace.iter().filter(|p| p.starts_with("relations/")).count(),
        6
    );
    assert_eq!(
        trace.iter().filter(|p| p.starts_with("manifests/")).count(),
        1
    );
    assert_eq!(trace.iter().filter(|p| p.starts_with("refs/")).count(), 1);
    assert_eq!(
        trace.iter().filter(|p| p.starts_with("contexts/")).count(),
        1
    );
    drop((old, next, observed, catalog, store));
    assert_eq!(budget.reserved(), 0);
    trace
}

#[tokio::test]
async fn every_actual_object_write_can_fail_without_publishing_a_mixed_snapshot() {
    let trace = successful_trace().await;
    // Observe successful CAS on InMemory, then replay each exact destination on
    // LocalFileSystem. The latter owns persistent bytes on disk, so abandoned
    // caller buffers must return to the observed-reference reservation baseline.
    for (index, interrupted_path) in trace.iter().enumerate() {
        interrupt_at(index, interrupted_path, &trace).await;
    }
}

async fn interrupt_at(index: usize, interrupted_path: &str, trace: &[String]) {
    let directory = tempfile::tempdir().expect("store");
    let backend = Arc::new(
        object_store::local::LocalFileSystem::new_with_prefix(directory.path()).expect("local"),
    );
    let store = FaultStore::new(backend);
    let budget = FixedBudget::new(64 << 20);
    let catalog = support::catalog(store.clone(), registry(), budget.clone());
    let cancel = CancellationToken::default();
    let name = RefName::parse("main").expect("name");
    let old = catalog
        .publish_bundle(draft(&catalog, 1, &cancel), &cancel)
        .await
        .expect("old");
    catalog
        .compare_and_swap_ref(&name, None, &old, &cancel)
        .await
        .expect("head");
    let reference = old.manifest_ref();
    drop(old);
    let observed = catalog
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("head");
    let baseline = budget.reserved();
    store.arm(FaultPlan {
        operation: "put",
        prefix: String::new(),
        call: index + 1,
        fault: Fault::FailBefore,
    });
    let result = match catalog
        .publish_bundle(draft(&catalog, 99, &cancel), &cancel)
        .await
    {
        Ok(next) => {
            catalog
                .compare_and_swap_ref(&name, Some(&observed), &next, &cancel)
                .await
        }
        Err(error) => Err(error),
    };
    assert!(result.is_err(), "boundary={interrupted_path}");
    assert_eq!(store.fired(), 1);
    assert_eq!(store.put_trace(), trace[..=index]);
    assert_eq!(
        budget.reserved(),
        baseline,
        "all abandoned candidates released"
    );
    let restored = catalog
        .read_snapshot(&name, &AdmissionContext::default(), &cancel)
        .await
        .expect("previous full admission")
        .expect("head");
    assert_eq!(restored.manifest_ref(), reference);
    assert_eq!(restored.relations().len(), 3);
    for spec in catalog.registry().relations() {
        let relation = restored
            .relation("authored", spec.key.name)
            .expect("old member");
        assert_eq!(
            pse_relations::cells::cells_from_batch(catalog.registry(), spec, relation.batch())
                .expect("actual old values"),
            vec![vec![Cell::U64(1), Cell::U64(1)]]
        );
    }
    drop((restored, observed));
    assert_eq!(budget.reserved(), 0);
}
