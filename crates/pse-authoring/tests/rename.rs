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
    change_set::{AuthoredReader, apply},
    document::{DocumentBundle, DocumentEdit, amend_rename_sources, load_package_texts, rename},
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId};
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
fn base(bundle: &DocumentBundle) -> Base {
    let registry = pse_schema::registry().unwrap();
    let empty = Base {
        revision: id(10),
        rows: BTreeMap::new(),
        documents: BTreeMap::new(),
    };
    let changes = pse_authoring::p1::stage(
        std::slice::from_ref(bundle),
        &empty,
        header(empty.revision),
        registry,
    )
    .unwrap();
    Base {
        revision: id(11),
        rows: apply(&empty, &changes, registry).unwrap().relations,
        documents: bundle
            .documents
            .iter()
            .map(|document| (document.id, document.text.clone()))
            .collect(),
    }
}
#[test]
fn complete_rename_preserves_bound_targets_and_lexical_shadowing() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle();
    let base = base(&bundle);
    let changes = rename(
        std::slice::from_ref(&bundle),
        &base,
        header(base.revision),
        id(3),
        "temperature",
        registry,
    )
    .unwrap();
    assert_eq!(
        changes
            .ops
            .iter()
            .filter(|operation| operation.op == ChangeOpKind::Rename)
            .count(),
        1
    );
    assert_eq!(changes.document_edits().len(), 2);
    let candidate = apply(&base, &changes, registry).unwrap();
    let spec = registry.relation("authored.case_spec_targets").unwrap();
    assert_eq!(
        pse_relations::cells::cells_from_batch(registry, spec, &base.rows[&spec.id]).unwrap(),
        pse_relations::cells::cells_from_batch(registry, spec, &candidate.relations[&spec.id])
            .unwrap()
    );
    let template = changes
        .document_edits()
        .iter()
        .find(|edit| edit.path == "templates/model.yaml")
        .unwrap();
    assert!(template.after.contains("temperature"));
    assert!(template.after.contains("where x = 3"));
    let target = changes
        .document_edits()
        .iter()
        .find(|edit| edit.path == "cases/design.yaml")
        .unwrap();
    assert!(target.after.contains("H.temperature"));
    assert!(
        base.documents
            .values()
            .any(|text| text.contains("target: H.x"))
    );
}

#[test]
fn owned_apply_releases_proof_scratch_and_keeps_exact_remaining_owners() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle();
    let mut base = base(&bundle);
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::new();
    // A real complete registry inventory reproduces the large verification
    // expansion of the full commit. Its input buffers remain charged throughout.
    for (key, rows) in registry.schema_rows_ref() {
        let spec = registry.relation(&key.qualified_name()).unwrap();
        base.rows.insert(
            spec.id,
            pse_relations::cells::batch_from_cells_owned(
                registry,
                spec,
                rows,
                budget.as_ref(),
                &cancel,
            )
            .unwrap(),
        );
    }
    let baseline = budget.reserved();
    assert!(baseline > 0);
    let changes = rename(
        std::slice::from_ref(&bundle),
        &base,
        header(base.revision),
        id(3),
        "temperature",
        registry,
    )
    .unwrap();
    let candidate =
        pse_authoring::change_set::apply_owned(&base, &changes, registry, budget.as_ref(), &cancel)
            .unwrap();
    let retained = budget.reserved();
    assert!(retained > baseline);
    assert!(
        retained - baseline < 64 << 20,
        "only complete proof/metadata copies and output buffers remain: {} bytes",
        retained - baseline
    );
    assert_eq!(candidate.changes.document_edits(), changes.document_edits());
    assert_eq!(candidate.changes.ops, changes.ops);
    let shared = candidate.clone();
    assert!(std::ptr::eq(&raw const *candidate, &raw const *shared));
    drop(candidate);
    assert_eq!(budget.reserved(), retained);
    let mut subsequent = budget.open("subsequent-session-work");
    subsequent.try_grow(64 << 20).unwrap();
    drop(subsequent);
    assert_eq!(budget.reserved(), retained);
    let entities = registry.relation("authored.entities").unwrap();
    let detached = std::sync::Arc::clone(shared.relations[&entities.id].column(0));
    drop(shared);
    assert!(
        budget.reserved() > baseline,
        "detached output retains its buffer lease"
    );
    assert!(
        budget.reserved() < retained,
        "candidate metadata/proof lease is released"
    );
    drop(detached);
    assert_eq!(budget.reserved(), baseline);
    let tiny = FixedBudget::new(1024);
    assert!(matches!(
        pse_authoring::change_set::apply_owned(&base, &changes, registry, tiny.as_ref(), &cancel),
        Err(AuthoringError::Resource(_))
    ));
    assert_eq!(tiny.reserved(), 0);
    drop(base);
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn explicit_case_revision_edits_revalidate_the_complete_rename() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle();
    let base = base(&bundle);
    let bundles = std::slice::from_ref(&bundle);
    let changes = rename(
        bundles,
        &base,
        header(base.revision),
        id(3),
        "temperature",
        registry,
    )
    .unwrap();
    let document = changes
        .document_edits()
        .iter()
        .find(|edit| edit.path == "cases/design.yaml")
        .unwrap();
    let additional = DocumentEdit {
        document_id: document.document_id,
        path: document.path.clone(),
        before: document.after.clone(),
        after: document
            .after
            .replace(&id(8).to_string(), &id(30).to_string()),
    };
    let amended = amend_rename_sources(
        &changes,
        bundles,
        &base,
        std::slice::from_ref(&additional),
        registry,
    )
    .unwrap();
    let candidate = apply(&base, &amended, registry).unwrap();
    let spec = registry.relation("authored.cases").unwrap();
    let rows =
        pse_relations::cells::cells_from_batch(registry, spec, &candidate.relations[&spec.id])
            .unwrap();
    let case = authored::cases::Row::from_cells(rows[0].clone()).unwrap();
    assert_eq!(case.model_revision_id, id(30));
    assert!(
        amended
            .ops
            .iter()
            .any(|operation| operation.op == ChangeOpKind::Rename)
    );
    let mut stale = additional.clone();
    stale.before.push(' ');
    assert!(amend_rename_sources(&changes, bundles, &base, &[stale], registry).is_err());
    let mut rebound = additional;
    rebound.after = rebound.after.replace("H.temperature", "H.equation");
    assert!(amend_rename_sources(&changes, bundles, &base, &[rebound], registry).is_err());
    let mut changed_base = base;
    changed_base
        .documents
        .values_mut()
        .next()
        .unwrap()
        .push(' ');
    assert!(amend_rename_sources(&changes, bundles, &changed_base, &[], registry).is_err());
}
#[test]
fn incomplete_sources_and_mutated_verified_envelopes_are_refused() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle();
    let base = base(&bundle);
    let mut incomplete = bundle.clone();
    incomplete.documents.pop();
    assert!(
        rename(
            &[incomplete],
            &base,
            header(base.revision),
            id(3),
            "temperature",
            registry
        )
        .is_err()
    );
    let mut changes = rename(
        &[bundle],
        &base,
        header(base.revision),
        id(3),
        "temperature",
        registry,
    )
    .unwrap();
    changes.header.message = "changed after proof".to_owned();
    assert!(apply(&base, &changes, registry).is_err());
}

#[test]
fn owned_rename_and_amend_retain_allocations_and_refuse_stale_sources() {
    use pse_authoring::document::{amend_rename_sources_owned, load_bundles_owned, rename_owned};
    use pse_ids::{CancellationToken, FixedBudget};
    let registry = pse_schema::registry().unwrap();
    let source = bundle();
    let base = base(&source);
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::default();
    let sources = load_bundles_owned(&[source], registry, budget.as_ref(), &cancel).unwrap();
    let initial = budget.reserved();
    let changes = rename_owned(
        &sources,
        &base,
        header(base.revision),
        id(3),
        "temperature",
        registry,
        (budget.as_ref(), &cancel),
    )
    .unwrap();
    assert!(budget.reserved() > initial);
    let alias = changes.clone();
    let held = budget.reserved();
    drop(changes);
    assert_eq!(budget.reserved(), held);
    let edit = next_model_edit(&alias);
    let amended = amend_rename_sources_owned(
        &alias,
        &sources,
        &base,
        std::slice::from_ref(&edit),
        registry,
        (budget.as_ref(), &cancel),
    )
    .unwrap();
    let candidate =
        pse_authoring::change_set::apply_owned(&base, &amended, registry, budget.as_ref(), &cancel)
            .unwrap();
    let spec = registry.relation("authored.cases").unwrap();
    let rows =
        pse_relations::cells::cells_from_batch(registry, spec, &candidate.relations[&spec.id])
            .unwrap();
    assert_eq!(
        authored::cases::Row::from_cells(rows[0].clone())
            .unwrap()
            .model_revision_id,
        id(30)
    );
    let mut stale = edit.clone();
    stale.before.push(' ');
    assert!(
        amend_rename_sources_owned(
            &alias,
            &sources,
            &base,
            &[stale],
            registry,
            (budget.as_ref(), &cancel)
        )
        .is_err()
    );
    let mut rebound = edit;
    rebound.after = rebound.after.replace("H.temperature", "H.equation");
    assert!(
        amend_rename_sources_owned(
            &alias,
            &sources,
            &base,
            &[rebound],
            registry,
            (budget.as_ref(), &cancel)
        )
        .is_err()
    );
    drop(candidate);
    drop(amended);
    drop(alias);
    assert_eq!(budget.reserved(), initial);
    let tiny = FixedBudget::new(1);
    assert!(
        rename_owned(
            &sources,
            &base,
            header(base.revision),
            id(3),
            "temperature",
            registry,
            (tiny.as_ref(), &cancel)
        )
        .is_err()
    );
    assert_eq!(tiny.reserved(), 0);
    drop(sources);
    assert_eq!(budget.reserved(), 0);
}

fn next_model_edit(envelope: &pse_authoring::change_set::ChangeSet) -> DocumentEdit {
    let source = envelope
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
