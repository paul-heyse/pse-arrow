// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual source-to-ref commit, including unchanged-content revision identity and stale CAS.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions report exact failures"
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
    Catalog, ExecutionSettings, FixedClock, RefName, ThreadBudget, TrustLevel,
    session::{SessionFactory, phase0_reference_profile},
};
use pse_compiler::{
    ExternalInputs, PassStatus, PolicySet,
    driver::{CommitBase, CommitRequest, Driver, PipelineRequest},
};
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_relations::generated::authored;
use pse_runtime::{ResourceBudget, SharedRuntime};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

fn document(registry: &pse_schema::Registry) -> DocumentBundle {
    load_package_texts(
        BTreeMap::from([(
            "package.toml".to_owned(),
            include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
        )]),
        registry,
        ParseBudget::default(),
    )
    .unwrap()
}
fn header(base: SemanticId) -> authored::change_sets::Row {
    authored::change_sets::Row {
        change_set_id: pse_authoring::ids::uuid_v7(),
        base_revision_id: base,
        author: "test".to_owned(),
        message: "exact source commit".to_owned(),
        created_at: 1_000_000_000,
    }
}

fn case_source(registry: &pse_schema::Registry, model: SemanticId) -> DocumentBundle {
    load_package_texts(BTreeMap::from([
        ("package.toml".to_owned(), include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned()),
        ("cases/design.yaml".to_owned(), format!("cases:\n  - id: '01010101010101010101010101010101'\n    model_revision_id: '{model}'\n    name: design\n    kind: base\n    doc: ''\n")),
    ]), registry, ParseBudget::default()).unwrap()
}
struct Fixture {
    registry: Arc<pse_schema::Registry>,
    catalog: Arc<Catalog>,
    sessions: Arc<SessionFactory>,
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
    let validator = pse_rules::validator::InvariantValidator::new(
        Arc::clone(&registry),
        runtime.runtime_env(),
        Arc::clone(&reserver),
        ExecutionSettings::default(),
        threads,
        phase0_reference_profile(),
    );
    let catalog = Arc::new(
        Catalog::open(
            Arc::new(InMemory::new()),
            Arc::clone(&registry),
            TrustLevel::Owned,
            Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
            Arc::clone(&reserver),
        )
        .with_semantic_validator(Arc::new(
            pse_compiler::validator::CompilerValidator::new(Arc::new(validator)),
        )),
    );
    let sessions = Arc::new(
        SessionFactory::new(
            runtime.runtime_env(),
            reserver,
            ExecutionSettings::default(),
            threads,
            phase0_reference_profile(),
        )
        .unwrap(),
    );
    Fixture {
        registry,
        catalog,
        sessions,
        _spill: spill,
    }
}

#[tokio::test]
async fn actual_commit_preserves_source_and_distinguishes_revisions_with_equal_content() {
    let Fixture {
        registry,
        catalog,
        sessions,
        _spill,
    } = fixture();
    let mut driver = Driver::new(Arc::clone(&catalog), Arc::clone(&sessions)).unwrap();
    let reference = RefName::parse("commit-test").unwrap();
    let cancel = CancellationToken::default();
    let source = document(&registry);
    let first = driver
        .commit(
            CommitRequest {
                revision_ids: None,
                reference: reference.clone(),
                base: None,
                documents: vec![source.clone()],
                header: header(SemanticId::NIL),
                changes: None,
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        first.validation.error_count, 0,
        "{:?}",
        first.validation.findings
    );
    let tip = first.tip.unwrap();
    let model = first.model.unwrap();
    let observed = catalog
        .read_ref(&reference, &cancel)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        observed.revision_ref().unwrap().revision_id,
        first.revision_id.unwrap()
    );
    let base = CommitBase {
        observed,
        snapshot: tip,
    };
    let reader = driver.base_reader(&base, &cancel).await.unwrap();
    assert_eq!(
        reader.source_documents().unwrap()[&source.documents[0].id],
        source.documents[0].text
    );
    let second = driver
        .commit(
            CommitRequest {
                revision_ids: None,
                reference: reference.clone(),
                base: Some(base.clone()),
                documents: vec![source.clone()],
                header: header(reader.revision_id()),
                changes: None,
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(second.validation.error_count, 0);
    assert_eq!(
        second.model.as_ref().unwrap().snapshot_id(),
        model.snapshot_id()
    );
    assert_ne!(second.revision_id, first.revision_id);
    let before = catalog
        .read_ref(&reference, &cancel)
        .await
        .unwrap()
        .unwrap();
    let stale = driver
        .commit(
            CommitRequest {
                revision_ids: None,
                reference: reference.clone(),
                base: Some(base),
                documents: vec![source],
                header: header(reader.revision_id()),
                changes: None,
            },
            &cancel,
        )
        .await;
    assert!(stale.is_err());
    let after = catalog
        .read_ref(&reference, &cancel)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(before.manifest_ref(), after.manifest_ref());
    assert_eq!(before.revision_ref(), after.revision_ref());
}

#[tokio::test]
async fn actual_p3_outputs_match_in_process_durable_and_uncached_execution() {
    let Fixture {
        registry,
        catalog,
        sessions,
        _spill,
    } = fixture();
    let mut driver = Driver::new(Arc::clone(&catalog), Arc::clone(&sessions)).unwrap();
    let reference = RefName::parse("commit-test").unwrap();
    let cancel = CancellationToken::default();
    let source = document(&registry);
    let first = driver
        .commit(
            CommitRequest {
                revision_ids: None,
                reference: reference.clone(),
                base: None,
                documents: vec![source.clone()],
                header: header(SemanticId::NIL),
                changes: None,
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        first.validation.error_count, 0,
        "{:?}",
        first.validation.findings
    );
    let tip = first.tip.unwrap();
    let pipeline = PipelineRequest {
        through: "P3".to_owned(),
        snapshot: Arc::clone(&tip),
        policies: PolicySet::default(),
        external_bindings: ExternalInputs::default(),
        fixture_mode: false,
        reuse: true,
    };
    let normalized = driver
        .run(pipeline.clone(), &cancel)
        .await
        .expect("uncached P3");
    assert_eq!(normalized.stages.len(), 1);
    assert_eq!(normalized.stages[0].status, PassStatus::Ok);
    let reused = driver
        .run(pipeline.clone(), &cancel)
        .await
        .expect("in-process P3 reuse");
    assert_eq!(reused.stages[0].status, PassStatus::Reused);
    let mut reopened_driver =
        Driver::new(Arc::clone(&catalog), Arc::clone(&sessions)).expect("fresh driver");
    let durable = reopened_driver
        .run(pipeline.clone(), &cancel)
        .await
        .expect("durable P3 reuse");
    assert_eq!(durable.stages[0].status, PassStatus::Reused);
    let mut uncached_request = pipeline;
    uncached_request.reuse = false;
    let uncached = driver
        .run(uncached_request, &cancel)
        .await
        .expect("forced uncached P3");
    for (port, expected) in normalized.stages[0].snapshot.relations() {
        assert_eq!(
            expected.batch(),
            uncached.stages[0].snapshot.relations()[port].batch(),
            "{port:?}"
        );
        assert_eq!(
            expected.batch(),
            durable.stages[0].snapshot.relations()[port].batch(),
            "{port:?}"
        );
    }
    assert_changed_normalization_refused(&catalog, &normalized.stages[0].snapshot).await;
}

async fn assert_changed_normalization_refused(catalog: &Catalog, snapshot: &pse_catalog::Snapshot) {
    use pse_catalog::store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    };
    let context = AdmissionContext {
        parents: snapshot.parents().clone(),
        stage_pass: snapshot.stage_pass(),
    };
    let manifest = catalog
        .manifest_template(pse_ids::SnapshotKind::Stage, &context)
        .unwrap();
    let registry = catalog.registry();
    let mut changed = false;
    let relations = snapshot
        .relations()
        .values()
        .map(|relation| {
            let mut batch = relation.batch().clone();
            if relation.contract().name == "package_graph" {
                let spec = registry.relation("normalized.package_graph").unwrap();
                let mut rows =
                    pse_relations::cells::cells_from_batch(registry, spec, &batch).unwrap();
                assert_eq!(rows.len(), 1);
                rows[0][1] = pse_schema::model::Cell::text("9.9.9");
                batch = pse_relations::cells::batch_from_cells(registry, spec, &rows).unwrap();
                changed = true;
            }
            (
                relation.member().port.clone(),
                RelationDraft {
                    contract: Arc::clone(relation.contract()),
                    batches: vec![batch],
                },
            )
        })
        .collect();
    assert!(changed);
    let error = catalog
        .publish_bundle(
            BundleDraft {
                manifest,
                context,
                relations,
            },
            &CancellationToken::default(),
        )
        .await
        .unwrap_err();
    assert!(
        format!("{error:?}").contains("actual stage rows differ from executed producer"),
        "{error:?}"
    );
}

#[tokio::test]
async fn case_publication_requires_the_actual_selected_model_revision() {
    let Fixture {
        registry,
        catalog,
        sessions,
        _spill,
    } = fixture();
    let mut driver = Driver::new(Arc::clone(&catalog), Arc::clone(&sessions)).unwrap();
    let cancel = CancellationToken::default();
    // Cases bind an explicitly selected revision; the compiler never rewrites authored IDs.
    let case_ref = RefName::parse("nonempty-case").unwrap();
    let ids = pse_compiler::driver::CommitRevisionIds {
        model: pse_authoring::ids::uuid_v7(),
        case: pse_authoring::ids::uuid_v7(),
    };
    let case_document = case_source(&registry, ids.model);
    let case_commit = driver
        .commit(
            CommitRequest {
                revision_ids: Some(ids),
                reference: case_ref.clone(),
                base: None,
                documents: vec![case_document.clone()],
                header: header(SemanticId::NIL),
                changes: None,
            },
            &cancel,
        )
        .await
        .expect("nonempty admitted case");
    assert_eq!(
        case_commit.validation.error_count, 0,
        "{:?}",
        case_commit.validation.findings
    );
    let tip = case_commit.tip.expect("published case");
    let observed = catalog.read_ref(&case_ref, &cancel).await.unwrap().unwrap();
    let base = CommitBase {
        observed: observed.clone(),
        snapshot: tip,
    };
    let reader = driver.base_reader(&base, &cancel).await.unwrap();
    let spec = registry.relation("authored.cases").unwrap();
    let actual = pse_relations::cells::cells_from_batch(
        &registry,
        spec,
        &reader.relations().unwrap()[&spec.id],
    )
    .unwrap();
    assert_eq!(actual[0][1], pse_schema::model::Cell::Id(ids.model));
    let next_ids = pse_compiler::driver::CommitRevisionIds {
        model: pse_authoring::ids::uuid_v7(),
        case: pse_authoring::ids::uuid_v7(),
    };
    for selected in [ids, next_ids] {
        let refused = driver
            .commit(
                CommitRequest {
                    revision_ids: Some(selected),
                    reference: case_ref.clone(),
                    base: Some(base.clone()),
                    documents: vec![case_document.clone()],
                    header: header(reader.revision_id()),
                    changes: None,
                },
                &cancel,
            )
            .await;
        assert!(
            refused.is_err(),
            "reuse of a revision ID or stale authored model binding must fail"
        );
        let after = catalog.read_ref(&case_ref, &cancel).await.unwrap().unwrap();
        assert_eq!(after.revision_ref(), observed.revision_ref());
    }
    let next = driver
        .commit(
            CommitRequest {
                revision_ids: Some(next_ids),
                reference: case_ref.clone(),
                base: Some(base),
                documents: vec![case_source(&registry, next_ids.model)],
                header: header(reader.revision_id()),
                changes: None,
            },
            &cancel,
        )
        .await
        .expect("case with exact next model binding");
    assert_eq!(
        next.validation.error_count, 0,
        "{:?}",
        next.validation.findings
    );
    assert_eq!(next.model_revision_id, Some(next_ids.model));
    assert_eq!(next.revision_id, Some(next_ids.case));
}

fn rename_source(registry: &pse_schema::Registry, model: SemanticId) -> DocumentBundle {
    let physical = pse_compiler::passes::p10::fixture::physical_package(registry).unwrap();
    let mut texts: BTreeMap<_, _> = physical
        .documents
        .into_iter()
        .map(|document| (document.path, document.text))
        .collect();
    let sid = |value| SemanticId::from_bytes([value; 16]);
    texts.insert("templates/model.yaml".to_owned(),format!(
        "templates:\n  - id: '{}'\n    name: heater\n    version: '1.0.0'\n    kind: unit\n    doc: ''\ntemplate_symbols:\n  - id: '{}'\n    template_id: '{}'\n    name: x\n    role: variable\n    quantity_type_id: '{}'\n    indexed_by: []\n    doc: ''\ntemplate_equations:\n  - id: '{}'\n    template_id: '{}'\n    name: equation\n    indexed_by: []\n    expression: 'x == (x + 1 where x = 3)'\n    sense: eq\n    doc: ''\n",
        sid(102),sid(103),sid(102),sid(31),sid(105),sid(102)));
    texts.insert("cases/design.yaml".to_owned(),format!(
        "instances:\n  - id: '{}'\n    template_id: '{}'\n    name: H\n    param_values: []\n    feature_values: []\n    doc: ''\ncases:\n  - id: '{}'\n    model_revision_id: '{model}'\n    name: design\n    kind: base\n    doc: ''\ncase_specs:\n  - id: '{}'\n    case_id: '{}'\n    target: H.x\n    priority: 0\n",
        sid(106),sid(102),sid(107),sid(109),sid(107)));
    load_package_texts(texts, registry, ParseBudget::default()).unwrap()
}

fn rename_with_case_revision(
    source: &DocumentBundle,
    reader: &pse_compiler::driver::BaseReader,
    registry: &pse_schema::Registry,
    old_model: SemanticId,
    new_model: SemanticId,
) -> pse_authoring::change_set::ChangeSet {
    use pse_authoring::document::{DocumentEdit, amend_rename_sources, rename};
    let changes = rename(
        std::slice::from_ref(source),
        reader,
        header(reader.revision_id()),
        SemanticId::from_bytes([103; 16]),
        "temperature",
        registry,
    )
    .unwrap();
    let renamed_case = changes
        .document_edits()
        .iter()
        .find(|edit| edit.path == "cases/design.yaml")
        .unwrap();
    let additional = DocumentEdit {
        document_id: renamed_case.document_id,
        path: renamed_case.path.clone(),
        before: renamed_case.after.clone(),
        after: renamed_case
            .after
            .replace(&old_model.to_string(), &new_model.to_string()),
    };
    assert_ne!(
        additional.before, additional.after,
        "the next model identity is explicitly authored"
    );
    amend_rename_sources(
        &changes,
        std::slice::from_ref(source),
        reader,
        &[additional],
        registry,
    )
    .unwrap()
}

#[tokio::test]
async fn full_rename_publishes_exact_sources_targets_and_explicit_case_revision_atomically() {
    use pse_compiler::driver::CommitRevisionIds;
    let Fixture {
        registry,
        catalog,
        sessions,
        _spill,
    } = fixture();
    let mut driver = Driver::new(Arc::clone(&catalog), sessions).unwrap();
    let cancel = CancellationToken::default();
    let reference = RefName::parse("full-rename").unwrap();
    let ids = CommitRevisionIds {
        model: pse_authoring::ids::uuid_v7(),
        case: pse_authoring::ids::uuid_v7(),
    };
    let source = rename_source(&registry, ids.model);
    let first = commit_initial_source(&mut driver, &reference, ids, source.clone(), &cancel).await;
    assert_eq!(
        first.validation.error_count, 0,
        "{:?}",
        first.validation.findings
    );
    let observed = catalog
        .read_ref(&reference, &cancel)
        .await
        .unwrap()
        .unwrap();
    let base = CommitBase {
        observed: observed.clone(),
        snapshot: first.tip.unwrap(),
    };
    let reader = driver.base_reader(&base, &cancel).await.unwrap();
    let next_ids = CommitRevisionIds {
        model: pse_authoring::ids::uuid_v7(),
        case: pse_authoring::ids::uuid_v7(),
    };
    let changes = rename_with_case_revision(&source, &reader, &registry, ids.model, next_ids.model);
    assert_eq!(
        catalog
            .read_ref(&reference, &cancel)
            .await
            .unwrap()
            .unwrap()
            .revision_ref(),
        observed.revision_ref()
    );
    let targets = registry.relation("authored.case_spec_targets").unwrap();
    let before = pse_relations::cells::cells_from_batch(
        &registry,
        targets,
        &reader.relations().unwrap()[&targets.id],
    )
    .unwrap();
    let edits = changes.document_edits().to_vec();
    let second = driver
        .commit(
            CommitRequest {
                reference: reference.clone(),
                revision_ids: Some(next_ids),
                base: Some(base.clone()),
                documents: vec![source],
                header: changes.header.clone(),
                changes: Some(changes),
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        second.validation.error_count, 0,
        "{:?}",
        second.validation.findings
    );
    let tip = second.tip.unwrap();
    let observed = catalog
        .read_ref(&reference, &cancel)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(observed.revision_ref().unwrap().revision_id, next_ids.case);
    let current = driver
        .base_reader(
            &CommitBase {
                observed,
                snapshot: tip,
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_rename_projection(&registry, &current, &reader, &before, &edits);
    let retained = driver.base_reader(&base, &cancel).await.unwrap();
    assert_eq!(
        retained.source_documents().unwrap(),
        reader.source_documents().unwrap()
    );
}

async fn commit_initial_source(
    driver: &mut Driver,
    reference: &RefName,
    ids: pse_compiler::driver::CommitRevisionIds,
    source: DocumentBundle,
    cancel: &CancellationToken,
) -> pse_compiler::driver::CommitReport {
    driver
        .commit(
            CommitRequest {
                reference: reference.clone(),
                revision_ids: Some(ids),
                base: None,
                documents: vec![source],
                header: header(SemanticId::NIL),
                changes: None,
            },
            cancel,
        )
        .await
        .unwrap()
}

fn assert_rename_projection(
    registry: &pse_schema::Registry,
    current: &pse_compiler::driver::BaseReader,
    previous: &pse_compiler::driver::BaseReader,
    before: &[Vec<pse_schema::model::Cell>],
    edits: &[pse_authoring::document::DocumentEdit],
) {
    let targets = registry.relation("authored.case_spec_targets").unwrap();
    let after = pse_relations::cells::cells_from_batch(
        registry,
        targets,
        &current.relations().unwrap()[&targets.id],
    )
    .unwrap();
    assert_eq!(before, after, "stable target identities survive rename");
    let sources = current.source_documents().unwrap();
    let originals = previous.source_documents().unwrap();
    for edit in edits {
        assert_eq!(sources[&edit.document_id], edit.after);
        assert_eq!(originals[&edit.document_id], edit.before);
    }
    assert!(sources.values().any(|text| text.contains("H.temperature")));
    assert!(sources.values().any(|text| text.contains("where x = 3")));
}
