// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Full rename preserves actual reference identities and rejects incomplete or changed inputs.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use arrow_array::RecordBatch;
use pse_authoring::{
    AuthoringError, ParseBudget,
    change_set::{AuthoredReader, ChangeSet, apply_owned},
    document::{
        DocumentBundle, DocumentEdit, OwnedDocumentSet, amend_rename_sources_owned,
        load_bundles_owned, load_package_texts, rename_owned,
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use std::sync::Arc;
mod support;
use pse_relations::generated::{authored, enums::ChangeOpKind};
use std::collections::BTreeMap;

struct Base {
    revision: SemanticId,
    rows: BTreeMap<SemanticId, RecordBatch>,
    documents: BTreeMap<SemanticId, String>,
}
impl AuthoredReader for Base {
    fn revision_id(&self) -> SemanticId {
        self.revision
    }
    fn relations(&self) -> Result<BTreeMap<SemanticId, RecordBatch>, AuthoringError> {
        Ok(self.rows.clone())
    }
    fn source_documents(&self) -> Result<BTreeMap<SemanticId, String>, AuthoringError> {
        Ok(self.documents.clone())
    }
}
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn header(base: SemanticId) -> authored::change_sets::Row {
    authored::change_sets::Row {
        change_set_id: id(20),
        base_revision_id: base,
        author: "fixture".to_owned(),
        message: "rename".to_owned(),
        created_at: 0,
    }
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
        pse_schema::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap()
}
struct Fixture {
    base: Base,
    sources: OwnedDocumentSet,
    session: SnapshotSession,
    budget: Arc<FixedBudget>,
    cancel: CancellationToken,
}
async fn fixture() -> Fixture {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let budget = FixedBudget::new(512 << 20);
    let session = support::session(Arc::clone(&registry_owner), Arc::clone(&budget));
    let cancel = CancellationToken::new();
    let sources = load_bundles_owned(&[bundle()], registry, budget.as_ref(), &cancel).unwrap();
    let empty = Base {
        revision: id(10),
        rows: BTreeMap::new(),
        documents: BTreeMap::new(),
    };
    let candidate =
        pse_authoring::p1::construct(&sources, &empty, header(empty.revision), &session, &cancel)
            .await
            .unwrap();
    let base = Base {
        revision: id(11),
        rows: candidate.relations.clone(),
        documents: sources
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .map(|document| (document.id, document.text.clone()))
            .collect(),
    };
    Fixture {
        base,
        sources,
        session,
        budget,
        cancel,
    }
}
async fn renamed(fixture: &Fixture) -> pse_authoring::change_set::OwnedChangeSet {
    rename_owned(
        &fixture.sources,
        &fixture.base,
        header(fixture.base.revision),
        id(3),
        "temperature",
        &fixture.session,
        &fixture.cancel,
    )
    .await
    .unwrap()
}
fn next_model_edit(changes: &ChangeSet) -> DocumentEdit {
    let source = changes
        .document_edits()
        .iter()
        .find(|edit| edit.path == "cases/design.yaml")
        .unwrap();
    DocumentEdit {
        document_id: source.document_id,
        path: source.path.clone(),
        before: source.after.clone(),
        after: source
            .after
            .replace(&id(8).to_string(), &id(30).to_string()),
    }
}
#[tokio::test]
async fn complete_rename_preserves_bound_targets_and_lexical_shadowing() {
    let fixture = fixture().await;
    let changes = renamed(&fixture).await;
    assert_eq!(
        changes
            .ops
            .iter()
            .filter(|operation| operation.op == ChangeOpKind::Rename)
            .count(),
        1
    );
    assert_eq!(changes.document_edits().len(), 2);
    let candidate = apply_owned(&fixture.base, &changes, &fixture.session, &fixture.cancel)
        .await
        .unwrap();
    let relation = authored::case_spec_targets::RELATION_ID;
    assert_eq!(fixture.base.rows[&relation], candidate.relations[&relation]);
    let template = changes
        .document_edits()
        .iter()
        .find(|edit| edit.path == "templates/model.yaml")
        .unwrap();
    assert!(template.after.contains("temperature"));
    assert!(template.after.contains("where x = 3"));
    assert!(
        changes
            .document_edits()
            .iter()
            .find(|edit| edit.path == "cases/design.yaml")
            .unwrap()
            .after
            .contains("H.temperature")
    );
    assert!(
        fixture
            .base
            .documents
            .values()
            .any(|text| text.contains("target: H.x"))
    );
}
#[tokio::test]
async fn additional_case_revision_edit_preserves_identity_and_refuses_stale_or_rebound_sources() {
    let fixture = fixture().await;
    let changes = renamed(&fixture).await;
    let edit = next_model_edit(&changes);
    let amended = amend_rename_sources_owned(
        &changes,
        &fixture.sources,
        &fixture.base,
        std::slice::from_ref(&edit),
        &fixture.session,
        &fixture.cancel,
    )
    .await
    .unwrap();
    let candidate = apply_owned(&fixture.base, &amended, &fixture.session, &fixture.cancel)
        .await
        .unwrap();
    let registry = fixture.session.registry();
    let spec = registry.relation("authored.cases").unwrap();
    let checked = pse_relations::columnar::FieldCheckedBatch::admit(
        registry,
        spec,
        candidate.relations[&spec.id].clone(),
    )
    .unwrap();
    assert_eq!(
        authored::cases::View::from_checked(&checked)
            .unwrap()
            .row(0)
            .unwrap()
            .model_revision_id,
        id(30)
    );
    let mut stale = edit.clone();
    stale.before.push(' ');
    assert!(
        amend_rename_sources_owned(
            &changes,
            &fixture.sources,
            &fixture.base,
            &[stale],
            &fixture.session,
            &fixture.cancel
        )
        .await
        .is_err()
    );
    let mut rebound = edit;
    rebound.after = rebound.after.replace("H.temperature", "H.equation");
    assert!(
        amend_rename_sources_owned(
            &changes,
            &fixture.sources,
            &fixture.base,
            &[rebound],
            &fixture.session,
            &fixture.cancel
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn copied_envelope_cannot_mint_private_source_completion() {
    let fixture = fixture().await;
    let changes = renamed(&fixture).await;
    let copied = ChangeSet::from_staged(
        changes.header.clone(),
        changes.ops.clone(),
        changes.staged.clone(),
        fixture.session.registry(),
    );
    if let Ok(copied) = copied {
        assert!(copied.documents().is_none());
        assert!(
            apply_owned(&fixture.base, &copied, &fixture.session, &fixture.cancel)
                .await
                .is_err()
        );
    }
    let mut stale = Base {
        revision: fixture.base.revision,
        rows: fixture.base.rows.clone(),
        documents: fixture.base.documents.clone(),
    };
    stale.documents.values_mut().next().unwrap().push(' ');
    assert!(
        rename_owned(
            &fixture.sources,
            &stale,
            header(stale.revision),
            id(3),
            "temperature",
            &fixture.session,
            &fixture.cancel
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn detached_source_columns_keep_their_reservations_until_last_owner_drop() {
    let fixture = fixture().await;
    let baseline = fixture.budget.reserved();
    let changes = renamed(&fixture).await;
    let shared = changes.clone();
    let envelope = (*changes).clone();
    let retained = fixture.budget.reserved();
    assert!(retained > baseline);
    drop(changes);
    assert_eq!(fixture.budget.reserved(), retained);
    drop(shared);
    assert_eq!(
        fixture.budget.reserved(),
        retained,
        "the candidate's envelope clone retains staging and source metadata allocations"
    );
    let detached = Arc::clone(
        envelope.documents().unwrap().bundles()[0].batches[&authored::entities::RELATION_ID]
            .batch()
            .column(0),
    );
    drop(envelope);
    assert!(fixture.budget.reserved() > baseline);
    assert!(fixture.budget.reserved() < retained);
    let budget = Arc::clone(&fixture.budget);
    // Session-owned native execution observations may outlive the change set. Drop
    // that independent owner before measuring the final detached buffer's release.
    drop(fixture);
    assert!(budget.reserved() > 0);
    drop(detached);
    assert_eq!(budget.reserved(), 0);
}
