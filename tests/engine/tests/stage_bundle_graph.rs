// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact producer ports remain distinct even when their relation schemas are identical.
#![allow(clippy::expect_used, reason = "complete stage-port fixture")]
use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, TrustLevel,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_compiler::{
    BoundInput, ExternalInputs, InputBundle, PassContext, PolicySet, memo::Dependencies,
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SnapshotKind};
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, DerivationGranularity, Determinism, LogicalType, Namespace,
        OutputPort, PassDecl, RelationDecl, SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

fn registry() -> Arc<pse_schema::Registry> {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Normalized,
            "values",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "stage fixture",
        )
        .granularity(DerivationGranularity::Rule)
        .pk(&["id"])
        .columns(vec![ColumnSpec::key("id", LogicalType::U64, "actual key")]),
    );
    builder.declare_pass(
        PassDecl::new("fixture", "1", Determinism::Deterministic).outputs(vec![
            OutputPort {
                port: "left",
                relation: "normalized.values".to_owned(),
            },
            OutputPort {
                port: "right",
                relation: "normalized.values".to_owned(),
            },
        ]),
    );
    Arc::new(builder.build().expect("complete declared producer"))
}

#[tokio::test]
async fn binding_requires_actual_named_output_port_when_schema_is_repeated() {
    let registry = registry();
    let spec = registry.relation("normalized.values").expect("relation");
    let pass = registry.pass("fixture@1").expect("producer");
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(64 << 20);
    let catalog = Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        Arc::clone(&registry),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        Arc::clone(&reserver),
    );
    let context = AdmissionContext {
        parents: BTreeMap::new(),
        stage_pass: Some(pass.id),
    };
    let manifest = catalog
        .manifest_template(SnapshotKind::Stage, &context)
        .expect("stage manifest");
    let relations = [("left", 1_u64), ("right", 2)]
        .into_iter()
        .map(|(port, value)| {
            (
                port.to_owned(),
                RelationDraft {
                    contract: Arc::new(
                        RelationContract::from_spec(&registry, spec, EncodingPolicy::IpcFile)
                            .expect("contract"),
                    ),
                    batches: vec![
                        pse_relations::cells::batch_from_cells(
                            &registry,
                            spec,
                            &[vec![Cell::U64(value)]],
                        )
                        .expect("actual values"),
                    ],
                },
            )
        })
        .collect();
    let snapshot = catalog
        .publish_bundle(
            BundleDraft {
                manifest,
                context,
                relations,
            },
            &CancellationToken::default(),
        )
        .await
        .expect("actual stage");
    assert!(BoundInput::bind(Arc::clone(&snapshot), spec.key, &registry).is_err());
    let mut dependencies = Vec::new();
    for (port, value) in [("left", 1_u64), ("right", 2)] {
        let input = BoundInput::bind_port(Arc::clone(&snapshot), spec.key, port, &registry)
            .expect("exact port");
        assert_eq!(
            pse_relations::cells::cells_from_batch(&registry, spec, input.relation().batch())
                .expect("actual output"),
            vec![vec![Cell::U64(value)]]
        );
        dependencies.push(
            Dependencies::capture(
                &InputBundle {
                    ports: BTreeMap::from([("input", Some(input))]),
                },
                &PassContext {
                    registry: &registry,
                    documents: &pse_authoring::document::OwnedDocumentSet::default(),
                    policies: &PolicySet::default(),
                    external: &ExternalInputs::default(),
                    cancel: &CancellationToken::default(),
                    reserver: reserver.as_ref(),
                    session: None,
                },
                false,
            )
            .expect("actual selected output dependency"),
        );
    }
    assert!(
        !dependencies[0]
            .equivalent(&dependencies[1])
            .expect("compare actual ports")
    );
    assert!(BoundInput::bind_port(snapshot, spec.key, "missing", &registry).is_err());
}
