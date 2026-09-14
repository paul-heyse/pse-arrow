// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Snapshot identity follows complete admitted membership and explicit actual parents.
#![allow(
    clippy::expect_used,
    reason = "test factories use fixed valid declarations and fixture objects"
)]

use pse_catalog::store::{
    membership::AdmissionContext,
    open::Catalog,
    publish::{BundleDraft, RelationDraft},
};
use pse_catalog::{EncodingPolicy, FixedClock, RelationContract, Snapshot, TrustLevel};
use pse_ids::{CancellationToken, FixedBudget, SnapshotKind};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, Cell, ColumnSpec, LogicalType, Namespace, RelationDecl, SnapshotClass},
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn fixture(version: u32) -> (Catalog, Arc<Registry>) {
    let mut builder = RegistryBuilder::new();
    for (name, class) in [
        ("values", SnapshotClass::Model),
        ("empty", SnapshotClass::Model),
        ("cases", SnapshotClass::Case),
        ("notes", SnapshotClass::Sidecar),
    ] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                version,
                Authority::Authored,
                class,
                "membership fixture",
            )
            .pk(&["id"])
            .columns(vec![ColumnSpec::key("id", LogicalType::U64, "Key")]),
        );
    }
    let reg = Arc::new(builder.build().expect("registry"));
    let catalog = Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        Arc::clone(&reg),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        FixedBudget::new(64 << 20),
    );
    (catalog, reg)
}
fn relation(reg: &Registry, name: &str, rows: &[Vec<Cell>]) -> RelationDraft {
    let spec = reg.relation(name).expect("relation");
    RelationDraft {
        contract: Arc::new(
            RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).expect("contract"),
        ),
        batches: vec![pse_relations::cells::batch_from_cells(reg, spec, rows).expect("typed rows")],
    }
}
fn model(catalog: &Catalog, reg: &Registry, id: u64) -> BundleDraft {
    let relations = ["values", "empty"]
        .into_iter()
        .map(|name| {
            let qualified = format!("authored.{name}");
            let spec = reg.relation(&qualified).expect("relation");
            (
                pse_ids::model_port_name("authored", spec.id),
                relation(
                    reg,
                    &qualified,
                    &if name == "values" {
                        vec![vec![Cell::U64(id)]]
                    } else {
                        vec![]
                    },
                ),
            )
        })
        .collect();
    BundleDraft {
        manifest: catalog
            .manifest_template(SnapshotKind::Model, &AdmissionContext::default())
            .expect("manifest"),
        relations,
        context: AdmissionContext::default(),
    }
}
fn case(catalog: &Catalog, reg: &Registry, parent: Arc<Snapshot>) -> BundleDraft {
    let context = AdmissionContext {
        parents: BTreeMap::from([("model".to_owned(), parent)]),
        stage_pass: None,
    };
    let spec = reg.relation("authored.cases").expect("case");
    BundleDraft {
        manifest: catalog
            .manifest_template(SnapshotKind::Case, &context)
            .expect("manifest"),
        context,
        relations: BTreeMap::from([(
            pse_ids::model_port_name("authored", spec.id),
            relation(reg, "authored.cases", &[]),
        )]),
    }
}

#[tokio::test]
async fn required_empty_members_and_explicit_parent_values_determine_admitted_membership() {
    let (catalog, reg) = fixture(1);
    let cancel = CancellationToken::default();
    let base = catalog
        .publish_bundle(model(&catalog, &reg, 1), &cancel)
        .await
        .expect("model");
    assert_eq!(base.relations().len(), 2);
    assert_eq!(
        base.relation("authored", "empty")
            .expect("explicit empty")
            .rows(),
        0
    );
    let mut missing = model(&catalog, &reg, 1);
    missing
        .relations
        .retain(|_, draft| draft.contract.name != "empty");
    assert!(catalog.publish_bundle(missing, &cancel).await.is_err());
    let old_case = catalog
        .publish_bundle(case(&catalog, &reg, Arc::clone(&base)), &cancel)
        .await
        .expect("case");
    let next = catalog
        .publish_bundle(model(&catalog, &reg, 2), &cancel)
        .await
        .expect("different actual model rows");
    let new_case = catalog
        .publish_bundle(case(&catalog, &reg, next), &cancel)
        .await
        .expect("case bound to actual next parent");
    assert_ne!(old_case.snapshot_id(), new_case.snapshot_id());
    let mut duplicate = base.manifest().frame();
    duplicate.members.push(duplicate.members[0].clone());
    assert!(
        pse_ids::snapshot_id(&duplicate).is_err(),
        "duplicate ports are rejected before framing"
    );
}

#[tokio::test]
async fn sidecar_publication_cannot_change_model_identity_and_changed_contracts_do() {
    let (catalog, reg) = fixture(1);
    let cancel = CancellationToken::default();
    let base = catalog
        .publish_bundle(model(&catalog, &reg, 1), &cancel)
        .await
        .expect("model");
    let artifact = catalog
        .publish_sidecar(
            relation(&reg, "authored.notes", &[vec![Cell::U64(7)]]),
            &cancel,
        )
        .await
        .expect("sidecar");
    assert_eq!(artifact.relation().rows(), 1);
    let reopened = catalog
        .read_manifest(base.manifest_ref(), &AdmissionContext::default(), &cancel)
        .await
        .expect("same actual admitted model");
    assert_eq!(reopened.snapshot_id(), base.snapshot_id());
    assert!(reopened.relation("authored", "notes").is_none());
    let (other, other_reg) = fixture(2);
    let changed = other
        .publish_bundle(model(&other, &other_reg, 1), &cancel)
        .await
        .expect("new actual schema version");
    assert_ne!(changed.snapshot_id(), base.snapshot_id());
}
