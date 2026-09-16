// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(clippy::unwrap_used, reason = "native candidate-admission assertions")]

use super::*;
use pse_ids::SemanticId;
use pse_relations::generated::enums::PublicationKind;
use pse_schema::{RegistryBuilder, model::*};

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    let relation = |name| {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Nested reference admission fixture.",
        )
        .pk(&["id"])
    };
    builder.declare_relation(relation("targets").columns(vec![FieldContract::key(
        "id",
        FieldContract::id(),
        "Target identity.",
    )]));
    builder.declare_relation(relation("sources").columns(vec![
        FieldContract::key("id", FieldContract::id(), "Source identity."),
        FieldContract::list(
            FieldContract::structure(vec![
                FieldContract::id()
                    .with_name("quoted.ref")
                    .with_fk("authored.targets", "id")
                    .optional(),
            ])
            .optional(),
        )
        .with_name("nested.list")
        .optional(),
    ]));
    Arc::new(builder.build().unwrap())
}

fn record() -> publications::Row {
    publications::Row {
        workspace_id: identity(1),
        publication_id: identity(2),
        parent_publication_id: None,
        attempt_id: identity(3),
        kind: PublicationKind::Source,
        inputs: vec![],
        members: vec![],
    }
}

fn bind(
    context: &SessionContext,
    record: &mut publications::Row,
    registry: &Registry,
    relation: &str,
    rows: &[Vec<Cell>],
    catalog: &str,
) {
    let spec = registry.relation(relation).unwrap();
    let reference = TableReference::full(catalog.to_owned(), "public", spec.key.name);
    context.deregister_table(reference.clone()).unwrap();
    context
        .register_batch(
            reference,
            pse_relations::cells::batch_from_cells(registry, spec, rows).unwrap(),
        )
        .unwrap();
    record
        .members
        .push(publications::RuntimePublicationsFieldMembersItem {
            catalog_name: catalog.into(),
            schema_name: "public".into(),
            table_name: spec.key.name.into(),
            table_uri: format!("memory://{catalog}/{relation}"),
            delta_version: 1,
            relation_id: spec.id,
            relation_version: i64::from(spec.key.version),
            contract_fingerprint: spec.fingerprint,
            selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
        });
}

fn identity(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn reference(value: u8) -> Cell {
    Cell::List(vec![Cell::Struct(vec![Cell::Id(identity(value))])])
}

#[tokio::test]
async fn nested_references_are_native_and_mask_aware() {
    let registry = registry();
    for (value, include_target, valid) in [
        (reference(10), true, true),
        (reference(11), true, false),
        (reference(10), false, false),
        (Cell::Null, false, true),
        (Cell::List(vec![]), false, true),
        (Cell::List(vec![Cell::Null]), false, true),
        (
            Cell::List(vec![Cell::Struct(vec![Cell::Null])]),
            false,
            true,
        ),
    ] {
        let context = SessionContext::new();
        let mut record = record();
        bind(
            &context,
            &mut record,
            &registry,
            "authored.sources",
            &[vec![Cell::Id(identity(1)), value]],
            "datafusion",
        );
        if include_target {
            bind(
                &context,
                &mut record,
                &registry,
                "authored.targets",
                &[vec![Cell::Id(identity(10))]],
                "datafusion",
            );
        }
        let state = context.state();
        let plans = violation_plans(&record, Arc::clone(&registry), &state)
            .await
            .unwrap();
        let explain = plans
            .iter()
            .map(|plan| plan.display_indent().to_string())
            .collect::<String>();
        assert!(explain.contains("Unnest"));
        if include_target {
            assert!(explain.contains("LeftAnti"));
        }
        assert_eq!(
            admit(&record, Arc::clone(&registry), &state).await.is_ok(),
            valid
        );
    }
}

#[tokio::test]
async fn nested_references_cannot_borrow_another_catalogs_identity() {
    use datafusion::catalog::{MemoryCatalogProvider, MemorySchemaProvider};
    let registry = registry();
    let context = SessionContext::new();
    let other = Arc::new(MemoryCatalogProvider::new());
    datafusion::catalog::CatalogProvider::register_schema(
        other.as_ref(),
        "public",
        Arc::new(MemorySchemaProvider::new()),
    )
    .unwrap();
    context.register_catalog("other", other);
    let mut record = record();
    bind(
        &context,
        &mut record,
        &registry,
        "authored.sources",
        &[vec![Cell::Id(identity(1)), reference(10)]],
        "datafusion",
    );
    bind(
        &context,
        &mut record,
        &registry,
        "authored.targets",
        &[vec![Cell::Id(identity(10))]],
        "other",
    );
    assert!(admit(&record, registry, &context.state()).await.is_err());
}

#[tokio::test]
async fn conflicting_exact_target_selections_refuse_before_execution() {
    let registry = registry();
    let context = SessionContext::new();
    let mut record = record();
    bind(
        &context,
        &mut record,
        &registry,
        "authored.sources",
        &[vec![Cell::Id(identity(1)), reference(10)]],
        "datafusion",
    );
    bind(
        &context,
        &mut record,
        &registry,
        "authored.targets",
        &[vec![Cell::Id(identity(10))]],
        "datafusion",
    );
    let mut another = record.members.last().unwrap().clone();
    another.delta_version += 1;
    record.members.push(another);
    let error = violation_plans(&record, registry, &context.state())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("ambiguous selected revisions"));
}
