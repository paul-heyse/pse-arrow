// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual producer ports determine parent rows even when output contracts repeat.
#![allow(clippy::unwrap_used, reason = "bounded declared port fixtures")]
use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, Snapshot, TrustLevel,
    provider::catalog::SnapshotCatalog,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SnapshotKind};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, DerivationGranularity, Determinism, InputPort, LogicalType,
        Namespace, OutputPort, PassDecl, PortSource, RelationDecl, SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    for name in ["inputs", "outputs"] {
        let column = ColumnSpec::key("id", LogicalType::U64, "actual member");
        let column = if name == "outputs" {
            column.with_fk("normalized.inputs", "id")
        } else {
            column
        };
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Normalized,
                name,
                1,
                Authority::Derived,
                SnapshotClass::Derived,
                "port fixture",
            )
            .granularity(DerivationGranularity::Rule)
            .pk(&["id"])
            .columns(vec![column]),
        );
    }
    builder.declare_pass(
        PassDecl::new("source", "1", Determinism::Deterministic).outputs(
            ["left", "right"]
                .into_iter()
                .map(|port| OutputPort {
                    port,
                    relation: "normalized.inputs".to_owned(),
                })
                .collect(),
        ),
    );
    for (name, source) in [
        (
            "selected",
            PortSource::Derived {
                pass: "source@1",
                port: "left",
            },
        ),
        ("ambiguous", PortSource::Pinned),
    ] {
        builder.declare_pass(
            PassDecl::new(name, "1", Determinism::Deterministic)
                .inputs(vec![InputPort {
                    port: "input",
                    relation: "normalized.inputs".to_owned(),
                    source,
                    required: true,
                }])
                .outputs(vec![OutputPort {
                    port: "result",
                    relation: "normalized.outputs".to_owned(),
                }]),
        );
    }
    Arc::new(builder.build().unwrap())
}
fn draft(
    catalog: &Catalog,
    pass: &str,
    parent: Option<&Arc<Snapshot>>,
    values: &[(&str, u64)],
) -> Result<BundleDraft, pse_catalog::CatalogError> {
    let reg = catalog.registry();
    let pass = reg.pass(pass).unwrap();
    let context = AdmissionContext {
        stage_pass: Some(pass.id),
        parents: parent.map_or_else(BTreeMap::new, |parent| {
            BTreeMap::from([("input".to_owned(), Arc::clone(parent))])
        }),
    };
    let relations = values
        .iter()
        .map(|(port, value)| {
            let output = pass
                .outputs
                .iter()
                .find(|output| output.port == *port)
                .unwrap();
            let spec = reg.relation(&output.relation).unwrap();
            let relation = RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap(),
                ),
                batches: vec![
                    pse_relations::cells::batch_from_cells(reg, spec, &[vec![Cell::U64(*value)]])
                        .unwrap(),
                ],
            };
            ((*port).to_owned(), relation)
        })
        .collect();
    Ok(BundleDraft {
        manifest: catalog.manifest_template(SnapshotKind::Stage, &context)?,
        context,
        relations,
    })
}
#[tokio::test]
async fn derived_parent_uses_only_the_named_output_and_pinned_ambiguity_is_refused() {
    let reg = registry();
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(128 << 20);
    let catalog = Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        Arc::clone(&reg),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        Arc::clone(&reserver),
    );
    let cancel = CancellationToken::default();
    let source = catalog
        .publish_bundle(
            draft(&catalog, "source@1", None, &[("left", 1), ("right", 2)]).unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(source.relations().len(), 2);
    assert!(source.relation("normalized", "inputs").is_none());
    assert!(SnapshotCatalog::new(&source, &reg, &reserver).is_err());
    let expected = draft(&catalog, "selected@1", Some(&source), &[("result", 1)]).unwrap();
    let context = expected.context.clone();
    let selected = catalog.publish_bundle(expected, &cancel).await.unwrap();
    catalog
        .read_manifest(selected.manifest_ref(), &context, &cancel)
        .await
        .unwrap();
    assert!(
        catalog
            .publish_bundle(
                draft(&catalog, "selected@1", Some(&source), &[("result", 2)]).unwrap(),
                &cancel
            )
            .await
            .is_err(),
        "an unselected sibling output cannot satisfy the selected parent's foreign key"
    );
    assert!(matches!(
        draft(&catalog, "ambiguous@1", Some(&source), &[("result", 1)]),
        Err(pse_catalog::CatalogError::Membership { reason })
            if reason == "input role input lacks one unambiguous declared relation normalized.inputs"
    ));
}
