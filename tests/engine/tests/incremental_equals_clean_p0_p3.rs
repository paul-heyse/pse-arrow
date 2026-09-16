// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A real species source edit traverses commit admission and invalidates actual P3 inputs.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixed integration fixture assertions"
)]

#[path = "../../support/workflow_budget.rs"]
mod workflow_budget;

use object_store::memory::InMemory;
use pse_authoring::{
    ParseBudget,
    change_set::AuthoredReader,
    document::{DocumentBundle, load_package_texts},
};
use pse_catalog::{
    Catalog, ExecutionSettings, FixedClock, RefName, Snapshot, ThreadBudget, TrustLevel,
    session::{SessionFactory, native_engine_profile},
};
use pse_compiler::{
    PassStatus, PolicySet,
    driver::{CommitBase, CommitReport, CommitRequest, Driver, PipelineRequest, StageResult},
};
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_relations::generated::authored;
use pse_runtime::{ResourceBudget, SharedRuntime};
use pse_schema::{Registry, model::Cell};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

const ADDED_SPECIES: &str = "  - id: '01991d6a-13a0-7000-8000-000000000003'\n    name: oxygen\n    formula: O2\n    mw: 0.031998\n    component_type: Component\n    charge: 0\n    doc: Molecular oxygen.\n";

fn source(registry: &Registry, changed: bool) -> DocumentBundle {
    let mut species =
        include_str!("../../fixtures/packages/minimal_explicit/materials/species.yaml").to_owned();
    if changed {
        species.push_str(ADDED_SPECIES);
    }
    // This is a complete model package. Cases are absent so the only source edit is
    // the added species; a case revision binding would introduce a second mutation.
    load_package_texts(
        BTreeMap::from([
            (
                "package.toml".to_owned(),
                include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
            ),
            ("materials/species.yaml".to_owned(), species),
            (
                "templates/model.yaml".to_owned(),
                include_str!("../../fixtures/packages/minimal_explicit/templates/model.yaml")
                    .to_owned(),
            ),
        ]),
        registry,
        ParseBudget::default(),
    )
    .unwrap()
}

struct Fixture {
    registry: Arc<Registry>,
    catalog: Arc<Catalog>,
    driver: Driver,
    _spill: tempfile::TempDir,
}
fn fixture() -> Fixture {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spill = tempfile::tempdir().unwrap();
    let one = NonZeroUsize::new(1).unwrap();
    let threads = ThreadBudget {
        pool_threads: one,
        target_partitions: one,
    };
    let runtime = SharedRuntime::build(ResourceBudget {
        memory_limit_bytes: NonZeroUsize::new(workflow_budget::MEMORY_LIMIT_BYTES).unwrap(),
        spill_dir: spill.path().to_path_buf(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: NonZeroUsize::new(16).unwrap(),
        threads,
        execution: ExecutionSettings::default(),
        hashing_may_use_pool: false,
    })
    .unwrap();
    let reserver: Arc<dyn MemoryReserver> = runtime.reserver();
    let sessions = Arc::new(
        SessionFactory::new(
            runtime.runtime_env(),
            Arc::clone(&reserver),
            ExecutionSettings::default(),
            threads,
            native_engine_profile(),
        )
        .unwrap(),
    );

    let validator = pse_rules::validator::InvariantValidator::new(Arc::clone(&registry));
    let catalog = Arc::new(
        Catalog::open(
            Arc::new(InMemory::new()),
            Arc::clone(&registry),
            TrustLevel::Owned,
            Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
            Arc::clone(&sessions),
        )
        .with_semantic_validator(Arc::new(
            pse_compiler::validator::CompilerValidator::new(Arc::new(validator), &registry)
                .unwrap()
                .with_sessions(Arc::clone(&sessions)),
        )),
    );
    let driver = Driver::new(Arc::clone(&catalog)).unwrap();
    Fixture {
        registry,
        catalog,
        driver,
        _spill: spill,
    }
}

async fn commit(
    fixture: &mut Fixture,
    reference: &RefName,
    base: Option<CommitBase>,
    source: DocumentBundle,
) -> CommitReport {
    let cancel = CancellationToken::new();
    let base_id = if let Some(base) = &base {
        fixture
            .driver
            .base_reader(base, &cancel)
            .await
            .unwrap()
            .revision_id()
    } else {
        SemanticId::NIL
    };
    let result = fixture
        .driver
        .commit(
            CommitRequest {
                revision_ids: None,
                reference: reference.clone(),
                base,
                documents: vec![source],
                changes: None,
                header: authored::change_sets::Row {
                    change_set_id: pse_authoring::ids::uuid_v7(),
                    base_revision_id: base_id,
                    author: "incremental oracle".to_owned(),
                    message: "actual complete source commit".to_owned(),
                    created_at: 1_000_000_000,
                },
            },
            &cancel,
        )
        .await
        .expect("real P0/P1/P2 commit");
    assert_eq!(
        result.validation.error_count(),
        0,
        "{:?}",
        result.validation.findings()
    );
    assert!(result.model.is_some() && result.tip.is_some());
    result
}
async fn base(fixture: &Fixture, reference: &RefName, tip: Arc<Snapshot>) -> CommitBase {
    CommitBase {
        snapshot: tip,
        observed: fixture
            .catalog
            .read_ref(reference, &CancellationToken::new())
            .await
            .unwrap()
            .unwrap(),
    }
}
async fn normalize(fixture: &mut Fixture, tip: Arc<Snapshot>, reuse: bool) -> StageResult {
    let result = fixture
        .driver
        .run(
            PipelineRequest {
                through: "P3".to_owned(),
                snapshot: tip,
                policies: PolicySet::default(),
                reuse,
            },
            &CancellationToken::new(),
        )
        .await
        .expect("actual P3 stage");
    assert_eq!(result.stages.len(), 1);
    result.stages.into_iter().next().unwrap()
}
fn rows(registry: &Registry, snapshot: &Snapshot) -> BTreeMap<String, Vec<String>> {
    snapshot
        .relations()
        .iter()
        .map(|(port, relation)| {
            let values = pse_relations::cells::decode_columns(registry, relation.batch()).unwrap();
            let mut lossless = values
                .into_iter()
                .map(|row| Cell::Struct(row).literal_spec())
                .collect::<Vec<_>>();
            lossless.sort();
            (port.clone(), lossless)
        })
        .collect()
}
fn species_count(snapshot: &Snapshot) -> usize {
    snapshot
        .relations()
        .values()
        .find(|relation| relation.contract().name == "species")
        .unwrap()
        .rows()
}
async fn sources(
    fixture: &Fixture,
    reference: &RefName,
    tip: Arc<Snapshot>,
) -> BTreeMap<SemanticId, String> {
    fixture
        .driver
        .base_reader(
            &base(fixture, reference, tip).await,
            &CancellationToken::new(),
        )
        .await
        .unwrap()
        .source_documents()
        .unwrap()
}

#[tokio::test]
async fn changed_species_commit_invalidates_p3_and_matches_a_clean_complete_run() {
    let mut incremental = fixture();
    let reference = RefName::parse("incremental-species").unwrap();
    let original = source(&incremental.registry, false);
    let changed = source(&incremental.registry, true);
    let original_texts = original
        .documents
        .iter()
        .map(|document| (document.id, document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let changed_texts = changed
        .documents
        .iter()
        .map(|document| (document.id, document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        original_texts.keys().collect::<Vec<_>>(),
        changed_texts.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        original_texts
            .iter()
            .filter(|(id, text)| changed_texts[*id] != **text)
            .count(),
        1
    );

    let first = commit(&mut incremental, &reference, None, original).await;
    assert_eq!(species_count(first.model.as_ref().unwrap()), 1);
    let first_tip = first.tip.unwrap();
    let before = normalize(&mut incremental, Arc::clone(&first_tip), true).await;
    assert_eq!(before.status, PassStatus::Ok);
    assert_eq!(species_count(&before.snapshot), 1);
    let previous = base(&incremental, &reference, first_tip).await;
    let second = commit(
        &mut incremental,
        &reference,
        Some(previous),
        changed.clone(),
    )
    .await;
    assert_eq!(species_count(second.model.as_ref().unwrap()), 2);
    let second_tip = second.tip.unwrap();
    let after = normalize(&mut incremental, Arc::clone(&second_tip), true).await;
    assert_eq!(
        after.status,
        PassStatus::Ok,
        "actual changed species input must miss"
    );
    assert_eq!(species_count(&after.snapshot), 2);
    let repeated = normalize(&mut incremental, Arc::clone(&second_tip), true).await;
    assert_eq!(
        repeated.status,
        PassStatus::Reused,
        "exact unchanged complete inputs must reuse"
    );
    assert_eq!(
        rows(&incremental.registry, &after.snapshot),
        rows(&incremental.registry, &repeated.snapshot)
    );
    assert_eq!(
        sources(&incremental, &reference, second_tip).await,
        changed_texts
    );

    let mut clean = fixture();
    let clean_ref = RefName::parse("clean-species").unwrap();
    let clean_commit = commit(&mut clean, &clean_ref, None, changed).await;
    assert_eq!(
        rows(&incremental.registry, second.model.as_ref().unwrap()),
        rows(&clean.registry, clean_commit.model.as_ref().unwrap())
    );
    let clean_tip = clean_commit.tip.unwrap();
    assert_eq!(
        sources(&clean, &clean_ref, Arc::clone(&clean_tip)).await,
        changed_texts
    );
    let clean_stage = normalize(&mut clean, clean_tip, false).await;
    assert_eq!(clean_stage.status, PassStatus::Ok);
    assert_eq!(
        rows(&incremental.registry, &after.snapshot),
        rows(&clean.registry, &clean_stage.snapshot)
    );
    assert_ne!(
        rows(&incremental.registry, &before.snapshot),
        rows(&incremental.registry, &after.snapshot)
    );
}
