// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source renames preserve identities, lexical binding and Arrow ownership.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]
use pse_authoring::ParseBudget;
use pse_columnar::CancellationToken;
use pse_engine::session::EngineSession;
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_runtime::authoring_driver::document::{
    DocumentBundle, DocumentEdit, OwnedDocumentSet, load_bundles_owned, load_package_texts,
    rename_documents,
};
use std::{collections::BTreeMap, sync::Arc};
#[path = "authoring_support/mod.rs"]
mod support;
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn bundle() -> DocumentBundle {
    let package = format!(
        "[package]\nid='{}'\nname='example'\nversion='1.0.0'\nkind='model'\nid_policy='explicit'\ndependencies=[]\ndoc=''\n",
        id(1)
    );
    let template = format!(
        "templates:\n  - id: '{}'\n    name: heater\n    version: '1.0.0'\n    kind: unit\n    doc: ''\ntemplate_symbols:\n  - id: '{}'\n    template_id: '{}'\n    name: x\n    role: variable\n    quantity_type_id: '{}'\n    indexed_by: []\n    doc: ''\ntemplate_equations:\n  - id: '{}'\n    template_id: '{}'\n    name: equation\n    indexed_by: []\n    expression: 'x == (x + 1 where x = 3)'\n    sense: eq\n    doc: ''\n",
        id(2),
        id(3),
        id(2),
        id(4),
        id(5),
        id(2)
    );
    let case = format!(
        "instances:\n  - id: '{}'\n    template_id: '{}'\n    name: H\n    param_values: []\n    feature_values: []\n    doc: ''\ncases:\n  - id: '{}'\n    model_revision_id: '{}'\n    name: design\n    kind: base\n    doc: ''\ncase_specs:\n  - id: '{}'\n    case_id: '{}'\n    target: H.x\n    priority: 0\n",
        id(6),
        id(2),
        id(7),
        id(8),
        id(9),
        id(7)
    );
    load_package_texts(
        BTreeMap::from([
            ("package.toml".to_owned(), package),
            ("templates/model.yaml".to_owned(), template),
            ("cases/design.yaml".to_owned(), case),
        ]),
        pse_engine::validation::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap()
}
struct Fixture {
    sources: OwnedDocumentSet,
    session: EngineSession,
    budget: Arc<dyn pse_columnar::MemoryPool>,
    cancel: CancellationToken,
}
fn fixture() -> Fixture {
    let registry = support::registry();
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20));
    let session = support::session(Arc::clone(&registry), Arc::clone(&budget));
    let cancel = CancellationToken::new();
    let sources = load_bundles_owned(&[bundle()], &registry, &budget, &cancel).unwrap();
    Fixture {
        sources,
        session,
        budget,
        cancel,
    }
}
async fn renamed(fixture: &Fixture) -> OwnedDocumentSet {
    rename_documents(
        &fixture.sources,
        id(3),
        "x",
        "temperature",
        &fixture.session,
        &fixture.cancel,
    )
    .await
    .unwrap()
}
fn text(sources: &OwnedDocumentSet, path: &str) -> String {
    sources
        .bundles()
        .iter()
        .flat_map(|b| &b.documents)
        .find(|d| d.path == path)
        .unwrap()
        .text
        .clone()
}
#[tokio::test]
async fn complete_rename_preserves_bound_targets_and_lexical_shadowing() {
    let fixture = fixture();
    let updated = renamed(&fixture).await;
    let before = pse_runtime::authoring_driver::p1::project(
        &fixture.sources,
        &fixture.session,
        &fixture.cancel,
    )
    .await
    .unwrap();
    let after =
        pse_runtime::authoring_driver::p1::project(&updated, &fixture.session, &fixture.cancel)
            .await
            .unwrap();
    let relation = authored::case_spec_targets::RELATION_ID;
    assert_eq!(before[&relation].batch(), after[&relation].batch());
    let template = text(&updated, "templates/model.yaml");
    assert!(template.contains("temperature"));
    assert!(template.contains("where x = 3"));
    assert!(text(&updated, "cases/design.yaml").contains("H.temperature"));
    assert!(text(&fixture.sources, "cases/design.yaml").contains("target: H.x"));
}
#[tokio::test]
async fn source_edits_compose_with_rename_and_require_exact_before_images() {
    let fixture = fixture();
    let updated = renamed(&fixture).await;
    let source = updated
        .bundles()
        .iter()
        .flat_map(|b| &b.documents)
        .find(|d| d.path == "cases/design.yaml")
        .unwrap();
    let edit = DocumentEdit {
        document_id: source.id,
        path: source.path.clone(),
        before: source.text.clone(),
        after: source.text.replace(&id(8).to_string(), &id(30).to_string()),
    };
    let amended = updated
        .edit(
            std::slice::from_ref(&edit),
            fixture.session.registry(),
            ParseBudget::default(),
            &fixture.budget,
            &fixture.cancel,
        )
        .unwrap();
    let projected =
        pse_runtime::authoring_driver::p1::project(&amended, &fixture.session, &fixture.cancel)
            .await
            .unwrap();
    let cases =
        authored::cases::View::from_checked(&projected[&authored::cases::RELATION_ID]).unwrap();
    assert_eq!(cases.row(0).unwrap().model_revision_id, id(30));
    let mut stale = edit.clone();
    stale.before.push(' ');
    assert!(
        updated
            .edit(
                &[stale],
                fixture.session.registry(),
                ParseBudget::default(),
                &fixture.budget,
                &fixture.cancel
            )
            .is_err()
    );
    assert!(
        updated
            .edit(
                &[edit.clone(), edit],
                fixture.session.registry(),
                ParseBudget::default(),
                &fixture.budget,
                &fixture.cancel
            )
            .is_err()
    );
    assert!(
        rename_documents(
            &updated,
            id(3),
            "x",
            "other",
            &fixture.session,
            &fixture.cancel
        )
        .await
        .is_err()
    );
    assert!(
        rename_documents(
            &updated,
            id(3),
            "temperature",
            "equation",
            &fixture.session,
            &fixture.cancel
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn named_policy_rejects_explicit_document_identities_and_rename_obeys_cancellation() {
    let fixture = fixture();
    let source = &fixture.sources.bundles()[0]
        .documents
        .iter()
        .find(|d| d.path == "package.toml")
        .unwrap();
    let edit = DocumentEdit {
        document_id: source.id,
        path: source.path.clone(),
        before: source.text.clone(),
        after: source
            .text
            .replace("id_policy='explicit'", "id_policy='named'"),
    };
    // The target parser enforces named identities when admitting the edit;
    // changing only the policy cannot produce a valid named document bundle.
    assert!(
        fixture
            .sources
            .edit(
                &[edit],
                fixture.session.registry(),
                ParseBudget::default(),
                &fixture.budget,
                &fixture.cancel
            )
            .is_err()
    );
    fixture.cancel.cancel();
    assert!(
        rename_documents(
            &fixture.sources,
            id(3),
            "x",
            "temperature",
            &fixture.session,
            &fixture.cancel
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn detached_source_columns_keep_reservations_until_last_owner_drop() {
    let fixture = fixture();
    let baseline = fixture.budget.reserved();
    let updated = renamed(&fixture).await;
    let shared = updated.clone();
    let retained = fixture.budget.reserved();
    assert!(retained > baseline);
    drop(updated);
    assert_eq!(fixture.budget.reserved(), retained);
    let detached = Arc::clone(
        shared.bundles()[0].batches[&authored::entities::RELATION_ID]
            .batch()
            .column(0),
    );
    drop(shared);
    let budget = Arc::clone(&fixture.budget);
    drop(fixture);
    assert!(budget.reserved() > 0);
    drop(detached);
    assert_eq!(budget.reserved(), 0);
}
