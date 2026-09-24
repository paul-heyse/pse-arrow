// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(clippy::unwrap_used, reason = "native candidate-admission assertions")]

use super::*;
use pse_ids::SemanticId;
use pse_relations::generated::enums::PublicationKind;
use pse_schema::{RegistryBuilder, model::*};

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    builder.declare_artifact_profile("relations", std::collections::BTreeSet::new());
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
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
    }
}

fn bind(
    context: &SessionContext,
    record: &mut publications::Row,
    registry: &Registry,
    relation: &str,
    rows: &[Vec<serde_json::Value>],
    catalog: &str,
) {
    let spec = registry.relation(relation).unwrap();
    let reference = TableReference::full(catalog.to_owned(), "public", spec.key.name);
    context.deregister_table(reference.clone()).unwrap();
    context
        .register_batch(
            reference,
            pse_relations::testing::untrusted_batch_from_literals(registry, spec, rows).unwrap(),
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

#[test]
fn complete_artifact_profiles_refuse_missing_members_but_allow_explicit_partial_collections() {
    let registry = pse_engine::validation::registry().unwrap();
    let mut request = record();
    admit_profile(&request, registry).unwrap();
    request.kind = PublicationKind::Source;
    assert!(admit_profile(&request, registry).is_err());
    request.members = registry
        .artifact_profile("source")
        .unwrap()
        .iter()
        .map(|id| {
            let spec = registry.relation_by_id(*id).unwrap();
            publications::RuntimePublicationsFieldMembersItem {
                catalog_name: "artifact".into(),
                schema_name: spec.key.namespace.as_str().into(),
                table_name: spec.key.name.into(),
                relation_id: spec.id,
                relation_version: i64::from(spec.key.version),
                contract_fingerprint: spec.fingerprint,
                table_uri: format!("memory:///source/{id}"),
                delta_version: 1,
                selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
            }
        })
        .collect();
    admit_profile(&request, registry).unwrap();
    request.members.pop();
    assert!(admit_profile(&request, registry).is_err());
}

fn reference(value: u8) -> serde_json::Value {
    serde_json::json!([
        "list",
        vec![serde_json::json!([
            "struct",
            vec![serde_json::json!(["id", (identity(value)).to_hex()])]
        ])]
    ])
}

#[tokio::test]
async fn an_explicit_singleton_admits_zero_or_one_row_and_refuses_two() {
    let mut builder = RegistryBuilder::new();
    builder.declare_artifact_profile("relations", std::collections::BTreeSet::new());
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "singleton",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Singleton cardinality fixture.",
        )
        .pk(&[])
        .columns(vec![
            FieldContract::native(datafusion::arrow::datatypes::DataType::Int64).with_name("value"),
        ]),
    );
    let registry = Arc::new(builder.build().unwrap());
    for count in 0..=2 {
        let context = SessionContext::new();
        let mut record = record();
        let rows = vec![vec![serde_json::json!(["i64", 7])]; count];
        bind(
            &context,
            &mut record,
            &registry,
            "authored.singleton",
            &rows,
            "datafusion",
        );
        assert_eq!(
            admit(&record, Arc::clone(&registry), &context.state())
                .await
                .is_ok(),
            count <= 1
        );
    }
}

#[tokio::test]
async fn nested_references_are_native_and_mask_aware() {
    let registry = registry();
    for (value, include_target, valid) in [
        (reference(10), true, true),
        (reference(11), true, false),
        (reference(10), false, false),
        (serde_json::json!(["null", null]), false, true),
        (serde_json::json!(["list", []]), false, true),
        (
            serde_json::json!(["list", vec![serde_json::json!(["null", null])]]),
            false,
            true,
        ),
        (
            serde_json::json!([
                "list",
                vec![serde_json::json!([
                    "struct",
                    vec![serde_json::json!(["null", null])]
                ])]
            ]),
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
            &[vec![
                serde_json::json!(["id", (identity(1)).to_hex()]),
                value,
            ]],
            "datafusion",
        );
        if include_target {
            bind(
                &context,
                &mut record,
                &registry,
                "authored.targets",
                &[vec![serde_json::json!(["id", (identity(10)).to_hex()])]],
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
        &[vec![
            serde_json::json!(["id", (identity(1)).to_hex()]),
            reference(10),
        ]],
        "datafusion",
    );
    bind(
        &context,
        &mut record,
        &registry,
        "authored.targets",
        &[vec![serde_json::json!(["id", (identity(10)).to_hex()])]],
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
        &[vec![
            serde_json::json!(["id", (identity(1)).to_hex()]),
            reference(10),
        ]],
        "datafusion",
    );
    bind(
        &context,
        &mut record,
        &registry,
        "authored.targets",
        &[vec![serde_json::json!(["id", (identity(10)).to_hex()])]],
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

fn composite_registry(policy: ReferenceNullPolicy, scalar: bool) -> Arc<Registry> {
    let relation = |name| {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Correlated reference fixture.",
        )
    };
    let integer = |name: &str| FieldContract::nonnegative(i64::MAX).with_name(name);
    let mapping = ReferenceContract {
        relation: "authored.targets".into(),
        columns: vec![
            ReferenceColumn {
                source: vec!["tenant.key".into()],
                target: "tenant".into(),
            },
            ReferenceColumn {
                source: vec!["nested".into(), "identity".into()],
                target: "id".into(),
            },
        ],
        null_policy: policy,
    };
    let value = if scalar {
        integer("reference").with_fk("authored.targets", "id")
    } else {
        FieldContract::list(
            FieldContract::structure(vec![
                integer("tenant.key").optional(),
                FieldContract::structure(vec![integer("identity").optional()])
                    .with_name("nested")
                    .optional(),
            ])
            .with_reference(&mapping)
            .unwrap()
            .optional(),
        )
        .with_name("reference")
        .optional()
    };
    let mut builder = RegistryBuilder::new();
    builder.declare_artifact_profile("relations", std::collections::BTreeSet::new());
    builder.declare_relation(
        relation("targets")
            .pk(&["tenant", "id"])
            .columns(vec![integer("tenant"), integer("id")]),
    );
    builder.declare_relation(
        relation("sources")
            .pk(&["id"])
            .columns(vec![integer("id"), value]),
    );
    Arc::new(builder.build().unwrap())
}

fn pair(tenant: Option<i64>, id: Option<i64>) -> serde_json::Value {
    serde_json::json!([
        "struct",
        vec![
            tenant.map_or(
                serde_json::json!(["null", null]),
                |value| serde_json::json!(["i64", value])
            ),
            serde_json::json!([
                "struct",
                vec![id.map_or(
                    serde_json::json!(["null", null]),
                    |value| serde_json::json!(["i64", value])
                )]
            ]),
        ]
    ])
}

#[tokio::test]
async fn composite_references_preserve_occurrence_correlation_and_null_policy() {
    for (value, policy, valid) in [
        (
            serde_json::json!([
                "list",
                vec![pair(Some(1), Some(10)), pair(Some(2), Some(20))]
            ]),
            ReferenceNullPolicy::Required,
            true,
        ),
        (
            serde_json::json!(["list", vec![pair(Some(1), Some(20))]]),
            ReferenceNullPolicy::Required,
            false,
        ),
        (
            serde_json::json!(["list", vec![pair(None, None)]]),
            ReferenceNullPolicy::AllOrNone,
            true,
        ),
        (
            serde_json::json!(["list", vec![pair(None, None)]]),
            ReferenceNullPolicy::Required,
            false,
        ),
        (
            serde_json::json!(["list", vec![pair(Some(1), None)]]),
            ReferenceNullPolicy::AllOrNone,
            false,
        ),
        (
            serde_json::json!(["list", vec![serde_json::json!(["null", null])]]),
            ReferenceNullPolicy::Required,
            true,
        ),
        (
            serde_json::json!(["list", []]),
            ReferenceNullPolicy::Required,
            true,
        ),
        (
            serde_json::json!(["null", null]),
            ReferenceNullPolicy::Required,
            true,
        ),
    ] {
        let registry = composite_registry(policy, false);
        let context = SessionContext::new();
        let mut record = record();
        bind(
            &context,
            &mut record,
            &registry,
            "authored.sources",
            &[vec![serde_json::json!(["i64", 1]), value]],
            "datafusion",
        );
        bind(
            &context,
            &mut record,
            &registry,
            "authored.targets",
            &[
                vec![
                    serde_json::json!(["i64", 1]),
                    serde_json::json!(["i64", 10]),
                ],
                vec![
                    serde_json::json!(["i64", 2]),
                    serde_json::json!(["i64", 20]),
                ],
            ],
            "datafusion",
        );
        assert_eq!(
            admit(&record, registry, &context.state()).await.is_ok(),
            valid
        );
    }
}

#[tokio::test]
async fn a_scalar_reference_cannot_claim_a_nonunique_selected_target_key() {
    let registry = composite_registry(ReferenceNullPolicy::Required, true);
    let context = SessionContext::new();
    let mut record = record();
    bind(
        &context,
        &mut record,
        &registry,
        "authored.sources",
        &[vec![
            serde_json::json!(["i64", 1]),
            serde_json::json!(["i64", 10]),
        ]],
        "datafusion",
    );
    bind(
        &context,
        &mut record,
        &registry,
        "authored.targets",
        &[
            vec![
                serde_json::json!(["i64", 1]),
                serde_json::json!(["i64", 10]),
            ],
            vec![
                serde_json::json!(["i64", 2]),
                serde_json::json!(["i64", 10]),
            ],
        ],
        "datafusion",
    );
    assert!(
        admit(&record, registry, &context.state())
            .await
            .unwrap_err()
            .to_string()
            .contains("ambiguous reference target")
    );
}
