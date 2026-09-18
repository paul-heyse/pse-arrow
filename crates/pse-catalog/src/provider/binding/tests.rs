// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    reason = "unit fixtures assert exact native ownership"
)]

use super::*;
use datafusion::{arrow::datatypes::Schema, datasource::MemTable};

fn table() -> Arc<dyn TableProvider> {
    Arc::new(MemTable::try_new(Arc::new(Schema::empty()), vec![vec![]]).unwrap())
}
fn reference() -> TableReference {
    TableReference::full("a.b", "s.q", "t.r")
}
fn role(name: &str) -> BindingKey {
    BindingKey::Input(name.to_owned())
}

#[tokio::test]
async fn roles_resolve_the_native_schema_owner_and_forks_isolate_replacement() {
    let mut bindings = Bindings::default();
    let first = table();
    let binding = TableBinding::new(reference(), Arc::clone(&first), None, None);
    bindings.insert(role("left"), binding.clone()).unwrap();
    bindings.insert(role("right"), binding).unwrap();
    assert!(Arc::ptr_eq(
        &bindings.get(&role("left")).unwrap(),
        &bindings.get(&role("right")).unwrap()
    ));
    let earlier = bindings.clone();
    let catalogs = bindings.catalogs("unused", "public").unwrap();
    let schema = catalogs.catalog("a.b").unwrap().schema("s.q").unwrap();
    assert!(Arc::ptr_eq(
        &schema.table("t.r").await.unwrap().unwrap(),
        &first
    ));
    let replacement = table();
    bindings
        .replace_table(
            &reference(),
            &TableBinding::new(reference(), Arc::clone(&replacement), None, None),
        )
        .unwrap();
    for name in ["left", "right"] {
        assert!(Arc::ptr_eq(
            &bindings.get(&role(name)).unwrap().provider,
            &replacement
        ));
        assert!(Arc::ptr_eq(
            &earlier.get(&role(name)).unwrap().provider,
            &first
        ));
    }
    assert!(Arc::ptr_eq(
        &schema.table("t.r").await.unwrap().unwrap(),
        &first
    ));
    assert_eq!(catalogs.generation(), 0);
}

#[tokio::test]
async fn removing_the_last_role_removes_the_native_table_only_in_that_fork() {
    let mut bindings = Bindings::default();
    let binding = TableBinding::new(reference(), table(), None, None);
    bindings.insert(role("left"), binding.clone()).unwrap();
    bindings.insert(role("right"), binding).unwrap();
    let earlier = bindings.clone();
    bindings.retain(|key| key != &role("left"));
    assert!(bindings.lookup(&reference()).is_some());
    bindings.retain(|_| false);
    assert!(bindings.lookup(&reference()).is_none());
    let catalogs = bindings.catalogs("unused", "public").unwrap();
    assert!(
        catalogs
            .catalog("a.b")
            .unwrap()
            .schema("s.q")
            .unwrap()
            .table("t.r")
            .await
            .unwrap()
            .is_none()
    );
    assert!(earlier.lookup(&reference()).is_some());
    assert!(
        bindings
            .scopes()
            .any(|scope| scope == ("a.b".into(), Some("s.q".into())))
    );
}

#[test]
fn aliases_cannot_override_established_semantic_facts() {
    let mut bindings = Bindings::default();
    let binding = TableBinding::new(reference(), table(), None, None);
    bindings.insert(role("first"), binding.clone()).unwrap();
    assert!(bindings.insert(role("second"), binding.observed()).is_err());
    assert!(bindings.get(&role("second")).is_none());
    assert!(
        !bindings
            .get(&role("first"))
            .unwrap()
            .effects
            .contains(&pse_schema::model::provider::OperationEffect::Observe)
    );
}

#[tokio::test]
async fn native_namespace_mutation_is_private_to_the_execution_fork() {
    let mut bindings = Bindings::default();
    bindings.namespace("a.b", Some("s.q")).unwrap();
    let execution = bindings.catalogs("a.b", "s.q").unwrap();
    let schema = execution.catalog("a.b").unwrap().schema("s.q").unwrap();
    schema.register_table("new".into(), table()).unwrap();
    assert_eq!(execution.generation(), 1);
    let separate = bindings.catalogs("a.b", "s.q").unwrap();
    assert!(
        separate
            .catalog("a.b")
            .unwrap()
            .schema("s.q")
            .unwrap()
            .table_names()
            .is_empty()
    );
    assert_eq!(separate.generation(), 0);
}
