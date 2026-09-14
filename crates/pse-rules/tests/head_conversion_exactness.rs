// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Rule heads admit exact values and declared meanings, not merely castable storage.
#![allow(clippy::unwrap_used, reason = "fixed rule boundary fixture assertions")]

use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    profile::phase0_reference_profile,
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId};
use pse_rules::{
    RuleError,
    exec::execute,
    plan::{PortBinding, compile},
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, DerivationGranularity, EnumDecl, EnumMember, LogicalType as T,
        Namespace, RelationDecl, RuleDecl, RuleExpr, RuleHead, RulePlan, RuleSpec, SnapshotClass,
    },
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

fn fixture(
    source: ColumnSpec,
    target: ColumnSpec,
    value: Cell,
) -> (Arc<Registry>, SnapshotSession, PortBinding) {
    let mut builder = RegistryBuilder::new();
    for (name, member) in [
        ("SourceChoice", "source_only"),
        ("TargetChoice", "target_only"),
    ] {
        builder.declare_enum(EnumDecl::platform(
            name,
            vec![
                EnumMember::new("shared", "same spelling"),
                EnumMember::new(member, "distinct meaning"),
            ],
        ));
    }
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "input",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "exact conversion input",
        )
        .pk(&["id"])
        .columns(vec![ColumnSpec::key("id", T::U32, "identity"), source]),
    );
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Inferred,
            "output",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "exact conversion output",
        )
        .granularity(DerivationGranularity::Row)
        .pk(&["id"])
        .columns(vec![ColumnSpec::key("id", T::U32, "identity"), target]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let spec = registry.relation("authored.input").unwrap();
    let key = spec.key;
    let batch =
        pse_relations::cells::batch_from_cells(&registry, spec, &[vec![Cell::U64(1), value]])
            .unwrap();
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(64 << 20);
    let session = build_candidate_session(
        BTreeMap::from([(key, batch)]),
        Arc::clone(&registry),
        Arc::new(RuntimeEnv::default()),
        reserver,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: NonZeroUsize::new(1).unwrap(),
            target_partitions: NonZeroUsize::new(1).unwrap(),
        },
        phase0_reference_profile(),
    )
    .unwrap();
    (
        registry,
        session,
        PortBinding {
            ports: BTreeMap::from([("input".to_owned(), key)]),
        },
    )
}

fn column(ty: T) -> ColumnSpec {
    ColumnSpec::payload("value", ty, "value contract")
}
fn rule(value: RuleExpr) -> RuleSpec {
    let declaration = RuleDecl::new(
        "head_exactness",
        "1",
        1,
        RuleHead::Relation("inferred.output".to_owned()),
        RulePlan::Project {
            input: Box::new(RulePlan::Scan {
                relation: "authored.input".to_owned(),
                port: "input",
            }),
            columns: vec![("id", RuleExpr::Col("id")), ("value", value)],
        },
    );
    RuleSpec {
        id: SemanticId::from_bytes([42; 16]),
        name: declaration.name,
        version: declaration.version,
        stratum: declaration.stratum,
        head: declaration.head,
        plan: declaration.plan,
        negation: declaration.negation,
        monotonic: declaration.monotonic,
        conflict_policy: declaration.conflict_policy,
    }
}
async fn admitted(target: ColumnSpec, literal: Cell, expected: Cell) {
    let (registry, session, ports) = fixture(column(T::U32), target, Cell::U64(0));
    let compiled = compile(&rule(RuleExpr::Lit(literal)), &ports, &session, &registry).unwrap();
    let outcome = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    let rows = outcome
        .head
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(&registry, batch).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![vec![Cell::U64(1), expected]]);
}
fn refused(target: ColumnSpec, literal: Cell) {
    let (registry, session, ports) = fixture(column(T::U32), target, Cell::U64(0));
    assert!(matches!(
        compile(&rule(RuleExpr::Lit(literal)), &ports, &session, &registry),
        Err(RuleError::HeadSchemaMismatch { .. })
    ));
}

#[tokio::test]
async fn integer_to_float_requires_exact_representability_at_the_binary_boundary() {
    for (integer, float) in [
        (0, 0.0),
        (-1, -1.0),
        (9_007_199_254_740_992, 9_007_199_254_740_992.0),
        (9_007_199_254_740_994, 9_007_199_254_740_994.0),
        (i64::MIN, -9_223_372_036_854_775_808.0),
    ] {
        admitted(column(T::F64), Cell::I64(integer), Cell::F64(float)).await;
    }
    for value in [9_007_199_254_740_993, -9_007_199_254_740_993, i64::MAX] {
        refused(column(T::F64), Cell::I64(value));
    }
}

#[tokio::test]
async fn float_to_integer_requires_integrality_and_the_actual_signed_range() {
    for (float, integer) in [
        (0.0, 0),
        (42.0, 42),
        (-9_223_372_036_854_775_808.0, i64::MIN),
        (9_223_372_036_854_774_784.0, 9_223_372_036_854_774_784),
    ] {
        admitted(column(T::I64), Cell::F64(float), Cell::I64(integer)).await;
    }
    for value in [
        42.5,
        -0.5,
        9_223_372_036_854_775_808.0,
        -18_446_744_073_709_551_616.0,
    ] {
        refused(column(T::I64), Cell::F64(value));
    }
}

#[tokio::test]
async fn nullable_heads_accept_typed_null_without_allowing_nullability_narrowing() {
    admitted(column(T::I64).optional(), Cell::Null, Cell::Null).await;
    refused(column(T::I64), Cell::Null);
    let (registry, session, ports) =
        fixture(column(T::I64).optional(), column(T::I64), Cell::I64(3));
    assert!(matches!(
        compile(&rule(RuleExpr::Col("value")), &ports, &session, &registry),
        Err(RuleError::HeadSchemaMismatch { .. })
    ));
}

#[tokio::test]
async fn quantity_and_enum_contracts_cannot_be_obtained_from_storage_agreement() {
    refused(column(T::F64).with_quantity("temperature"), Cell::F64(3.0));
    let (registry, session, ports) = fixture(
        column(T::F64).with_quantity("temperature"),
        column(T::F64).with_quantity("pressure"),
        Cell::F64(3.0),
    );
    assert!(matches!(
        compile(&rule(RuleExpr::Col("value")), &ports, &session, &registry),
        Err(RuleError::HeadSchemaMismatch { .. })
    ));
    let (registry, session, ports) = fixture(
        column(T::enumeration("SourceChoice")),
        column(T::enumeration("TargetChoice")),
        Cell::Enum("shared"),
    );
    assert!(matches!(
        compile(&rule(RuleExpr::Col("value")), &ports, &session, &registry),
        Err(RuleError::HeadSchemaMismatch { .. })
    ));
    refused(
        column(T::enumeration("TargetChoice")),
        Cell::Enum("source_only"),
    );
    admitted(
        column(T::enumeration("TargetChoice")),
        Cell::Enum("target_only"),
        Cell::Enum("target_only"),
    )
    .await;
}
