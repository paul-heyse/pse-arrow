// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Characterization probes for the 2026-09-15 provider design review.
//! These assert observed gaps, not target acceptance criteria.

use datafusion::{
    arrow::array::{RecordBatch, UInt64Array},
    catalog::{CatalogProvider, CatalogProviderList, MemoryCatalogProvider},
    execution::runtime_env::RuntimeEnv,
};
use pse_catalog::{
    provider::list::SnapshotCatalogList,
    session::{
        ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
    },
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    RegistryBuilder,
    model::{Authority, ColumnSpec, LogicalType, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc};

fn session() -> SnapshotSession {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Review probe values.",
        )
        .pk(&["id"])
        .columns(vec![ColumnSpec::key("id", LogicalType::U64, "Value.")]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let spec = registry.relation("authored.values").unwrap();
    let batch = RecordBatch::try_new(
        Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
        vec![Arc::new(UInt64Array::from(vec![7]))],
    )
    .unwrap();
    let checked = FieldCheckedBatch::admit(&registry, spec, batch).unwrap();
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
    .candidate_checked_ports(
        BTreeMap::from([("before".to_owned(), checked)]),
        registry,
        &CancellationToken::new(),
    )
    .unwrap()
}

#[tokio::test]
async fn direct_role_scan_works_but_sql_hierarchy_cannot_resolve_it() {
    let session = session();
    let cancel = CancellationToken::new();
    let result = session
        .prepare(session.scan_role("before").unwrap(), &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert_eq!(
        result
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        1
    );
    let error = session
        .sql("SELECT id FROM roles.inputs.before", &cancel)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("not found"), "{error}");
}

#[tokio::test]
async fn ordinary_information_schema_is_outside_the_admitted_inventory() {
    let session = session();
    let error = session
        .sql(
            "SELECT table_name FROM information_schema.tables",
            &CancellationToken::new(),
        )
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("outside the pinned session inventory"),
        "{error}"
    );
}

#[test]
fn sealed_catalog_list_registration_returns_none_but_does_not_register() {
    let list = SnapshotCatalogList::new(BTreeMap::new());
    let provider: Arc<dyn CatalogProvider> = Arc::new(MemoryCatalogProvider::new());
    assert!(
        list.register_catalog("requested".to_owned(), provider)
            .is_none()
    );
    assert!(list.catalog("requested").is_none());
}

#[test]
fn view_resolution_can_inline_without_a_table_scan() {
    use datafusion::{
        datasource::provider_as_source,
        logical_expr::{LogicalPlan, LogicalPlanBuilder},
    };
    use datafusion_catalog::view::ViewTable;
    let inner = LogicalPlanBuilder::empty(true).build().unwrap();
    let provider = provider_as_source(Arc::new(ViewTable::new(inner, None)));
    let plan = LogicalPlanBuilder::scan("review_view", provider, None)
        .unwrap()
        .build()
        .unwrap();
    let LogicalPlan::SubqueryAlias(alias) = plan else {
        panic!("expected an inlined view alias");
    };
    assert!(matches!(
        alias.input.as_ref(),
        LogicalPlan::EmptyRelation(_)
    ));
}

#[tokio::test]
async fn native_sql_ddl_changes_the_namespace_before_dataframe_collection() {
    let context = datafusion::execution::context::SessionContext::new();
    let frame = context
        .sql("CREATE TABLE review_ddl (id BIGINT)")
        .await
        .unwrap();
    assert!(context.table_exist("review_ddl").unwrap());
    drop(frame);
}
