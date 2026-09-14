// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Forced lookup collisions never replace complete actual dependency comparison.
#![allow(
    clippy::expect_used,
    reason = "bounded fixtures report exact failure boundaries"
)]
use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, Snapshot, TrustLevel,
    session::{
        ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
        phase0_reference_profile,
    },
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_compiler::{
    BoundInput, ExternalInputs, InputBundle, PassContext, PolicySet, StageKey,
    memo::{Dependencies, Memo},
    passes::PolicyBinding,
};
use pse_ids::{
    CancellationToken, ContentHash, FixedBudget, MemoryReserver, SemanticId, SnapshotKind,
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, EnumDecl, LogicalType, Namespace, RelationDecl, SnapshotClass,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    sync::Arc,
};

struct Fixture {
    registry: Arc<Registry>,
    reserver: Arc<dyn MemoryReserver>,
}
fn fixture() -> Fixture {
    let mut builder = RegistryBuilder::new();
    declare_package_document(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "inputs",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "memo fixture",
        )
        .pk(&["id"])
        .columns(vec![
            ColumnSpec::key("id", LogicalType::id(), "key"),
            ColumnSpec::payload("value", LogicalType::I64, "actual dependency"),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(128 << 20);
    Fixture { registry, reserver }
}

fn declare_package_document(builder: &mut RegistryBuilder) {
    let original = pse_schema::catalog::assemble().expect("document authority");
    let document = original
        .documents()
        .iter()
        .find(|document| document.path_glob == "package.toml")
        .expect("package document");
    let mut pending = document
        .sections
        .iter()
        .map(|section| section.relation)
        .collect::<Vec<_>>();
    pending.extend(["authored.entities", "authored.documents"]);
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name) {
            continue;
        }
        let relation = original.relation(name).expect("declared dependency");
        pending.extend(
            relation
                .columns
                .iter()
                .filter_map(|column| column.fk.map(|fk| fk.relation)),
        );
        builder.declare_relation(RelationDecl {
            key: relation.key,
            authority: relation.authority,
            snapshot_class: relation.snapshot_class,
            derivation_granularity: relation.derivation_granularity,
            stability: relation.stability,
            primary_key: relation.primary_key.clone(),
            columns: relation.columns.clone(),
            doc: relation.doc,
        });
    }
    for enumeration in original.enums() {
        builder.declare_enum(EnumDecl {
            name: enumeration.name,
            idaes_source: enumeration.idaes_source,
            members: enumeration.members.clone(),
        });
    }
    builder.declare_document(document.clone());
}

async fn snapshot(fixture: &Fixture, value: Option<i64>, timestamp: &str) -> Arc<Snapshot> {
    let spec = fixture.registry.relation("authored.inputs").expect("spec");
    let rows = value
        .map(|value| {
            vec![vec![
                Cell::Id(SemanticId::from_bytes([1; 16])),
                Cell::I64(value),
            ]]
        })
        .unwrap_or_default();
    let relations = fixture
        .registry
        .relations()
        .iter()
        .filter(|relation| relation.snapshot_class == SnapshotClass::Model)
        .map(|relation| {
            let values = if relation.id == spec.id {
                rows.as_slice()
            } else {
                &[]
            };
            let batch = pse_relations::cells::batch_from_cells(&fixture.registry, relation, values)
                .expect("complete explicit member");
            (
                pse_ids::model_port_name(relation.key.namespace.as_str(), relation.id),
                RelationDraft {
                    contract: Arc::new(
                        RelationContract::from_spec(
                            &fixture.registry,
                            relation,
                            EncodingPolicy::IpcFile,
                        )
                        .expect("contract"),
                    ),
                    batches: vec![batch],
                },
            )
        })
        .collect();
    let context = AdmissionContext::default();
    let catalog = Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        Arc::clone(&fixture.registry),
        TrustLevel::Untrusted,
        Arc::new(FixedClock(timestamp.to_owned())),
        Arc::clone(&fixture.reserver),
    );
    let manifest = catalog
        .manifest_template(SnapshotKind::Model, &context)
        .expect("manifest");
    catalog
        .publish_bundle(
            BundleDraft {
                manifest,
                context,
                relations,
            },
            &CancellationToken::default(),
        )
        .await
        .expect("snapshot")
}
fn inputs(fixture: &Fixture, snapshot: Arc<Snapshot>) -> InputBundle {
    let key = fixture
        .registry
        .relation("authored.inputs")
        .expect("spec")
        .key;
    InputBundle {
        ports: BTreeMap::from([(
            "input",
            Some(BoundInput::bind(snapshot, key, &fixture.registry).expect("actual input")),
        )]),
    }
}
fn capture(
    fixture: &Fixture,
    inputs: &InputBundle,
    policies: &PolicySet,
    documents: &[DocumentBundle],
    session: Option<&SnapshotSession>,
) -> Dependencies {
    let documents = pse_authoring::document::load_bundles_owned(
        documents,
        &fixture.registry,
        fixture.reserver.as_ref(),
        &CancellationToken::new(),
    )
    .expect("actual source owners");
    Dependencies::capture(
        inputs,
        &PassContext {
            registry: &fixture.registry,
            documents: &documents,
            policies,
            external: &ExternalInputs::default(),
            cancel: &CancellationToken::default(),
            reserver: fixture.reserver.as_ref(),
            session,
        },
        session.is_some(),
    )
    .expect("complete retained inputs")
}
fn session(fixture: &Fixture, input: &InputBundle, version: &str) -> SnapshotSession {
    let runtime = Arc::new(
        datafusion::execution::runtime_env::RuntimeEnvBuilder::new()
            .build()
            .expect("runtime"),
    );
    let one = NonZeroUsize::new(1).expect("one");
    let mut profile = phase0_reference_profile();
    version.clone_into(&mut profile.version);
    build_candidate_session(
        input.rows(&fixture.registry).expect("rows"),
        Arc::clone(&fixture.registry),
        runtime,
        Arc::clone(&fixture.reserver),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        profile,
    )
    .expect("sealed engine semantics")
}
#[tokio::test]
async fn forced_bucket_collision_checks_values_absence_policy_and_exact_lineage() {
    let fixture = fixture();
    let original = snapshot(&fixture, Some(7), "2026-09-14T00:00:00Z").await;
    let changed = snapshot(&fixture, Some(8), "2026-09-14T00:00:00Z").await;
    let new_lineage = snapshot(&fixture, Some(7), "2026-09-14T01:00:00Z").await;
    assert_eq!(original.snapshot_id(), new_lineage.snapshot_id());
    assert_ne!(original.manifest_ref(), new_lineage.manifest_ref());
    let present = inputs(&fixture, Arc::clone(&original));
    let policies = PolicySet::default();
    let key = StageKey(ContentHash::from_bytes([0; 32]));
    let mut memo = Memo::new(4);
    memo.insert(
        key,
        capture(&fixture, &present, &policies, &[], None),
        Arc::clone(&original),
    );
    assert!(
        memo.lookup(key, &capture(&fixture, &present, &policies, &[], None))
            .expect("lookup")
            .is_some()
    );
    for changed in [
        inputs(&fixture, changed),
        inputs(&fixture, new_lineage),
        InputBundle {
            ports: BTreeMap::from([("input", None)]),
        },
    ] {
        assert!(
            memo.lookup(key, &capture(&fixture, &changed, &policies, &[], None))
                .expect("collision")
                .is_none()
        );
    }
    let empty = inputs(
        &fixture,
        snapshot(&fixture, None, "2026-09-14T00:00:00Z").await,
    );
    assert!(
        memo.lookup(key, &capture(&fixture, &empty, &policies, &[], None))
            .expect("empty input")
            .is_none()
    );
    let policy = PolicySet(BTreeMap::from([(
        "selection".to_owned(),
        PolicyBinding {
            policy_id: SemanticId::from_bytes([1; 16]),
            input: present.ports["input"].clone().expect("present"),
        },
    )]));
    assert!(
        memo.lookup(key, &capture(&fixture, &present, &policy, &[], None))
            .expect("policy")
            .is_none()
    );
}
#[tokio::test]
async fn source_bytes_and_actual_engine_profile_change_even_when_the_lookup_key_is_forced_equal() {
    let fixture = fixture();
    let snapshot = snapshot(&fixture, Some(7), "2026-09-14T00:00:00Z").await;
    let input = inputs(&fixture, Arc::clone(&snapshot));
    let document = load_package_texts(
        BTreeMap::from([(
            "package.toml".to_owned(),
            include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
        )]),
        &fixture.registry,
        ParseBudget::default(),
    )
    .expect("real source");
    let mut changed = document.clone();
    changed.documents[0]
        .text
        .push_str("\n# changed actual source bytes\n");
    assert_eq!(
        document.documents[0].content_hash, changed.documents[0].content_hash,
        "unchanged public hash claim is deliberate"
    );
    let policies = PolicySet::default();
    let original_engine = session(&fixture, &input, "one");
    let changed_engine = session(&fixture, &input, "two");
    let key = StageKey(ContentHash::from_bytes([0; 32]));
    let mut memo = Memo::new(4);
    memo.insert(
        key,
        capture(
            &fixture,
            &input,
            &policies,
            std::slice::from_ref(&document),
            Some(&original_engine),
        ),
        snapshot,
    );
    assert!(
        memo.lookup(
            key,
            &capture(
                &fixture,
                &input,
                &policies,
                &[changed],
                Some(&original_engine)
            )
        )
        .expect("source bytes")
        .is_none()
    );
    assert!(
        memo.lookup(
            key,
            &capture(
                &fixture,
                &input,
                &policies,
                &[document],
                Some(&changed_engine)
            )
        )
        .expect("engine semantics")
        .is_none()
    );
}
#[test]
fn production_graph_has_no_synthetic_p10_predecessors() {
    let registry = pse_schema::catalog::assemble().expect("production registry");
    let graph = pse_compiler::passes::dag::StageDag::build(&registry).expect("closed graph");
    assert_eq!(
        graph
            .through("P3")
            .expect("P3")
            .iter()
            .map(|spec| spec.name)
            .collect::<Vec<_>>(),
        ["P3"]
    );
    assert!(graph.through("P10").is_err());
}
