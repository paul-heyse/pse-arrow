// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent expected rows exercise the actual sealed-session relational compiler.
#![allow(
    clippy::unwrap_used,
    reason = "independent fixture decoding assertions"
)]
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    profile::native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId};
use pse_rules::{
    exec::execute,
    plan::{PortBinding, compile},
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, DependencyMode, DerivationGranularity, FieldContract, FieldContract as T,
        Namespace, RelationDecl, RuleDecl, RuleInput, RuleQuery, SnapshotClass,
    },
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "fixture helper asserts fixed test setup"
)]
fn fixture() -> (Arc<Registry>, SnapshotSession, PortBinding) {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "input",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Fixture input",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "key",
            ),
            FieldContract::payload(
                "flag",
                T::native(datafusion::arrow::datatypes::DataType::Boolean),
                "predicate",
            )
            .optional(),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::Float64),
                "measure",
            ),
            FieldContract::payload(
                "members",
                T::list(T::native(datafusion::arrow::datatypes::DataType::UInt32)),
                "orderedmembers",
            )
            .optional(),
            FieldContract::reference(
                "parent",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "graph parent",
            )
            .optional(),
        ]),
    );
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Inferred,
            "closure",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "Expected reachability",
        )
        .granularity(DerivationGranularity::Row)
        .pk(&["origin", "reached"])
        .columns(vec![
            FieldContract::key(
                "origin",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "start",
            ),
            FieldContract::key(
                "reached",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "end",
            ),
        ]),
    );
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Inferred,
            "output",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "Fixture output",
        )
        .granularity(DerivationGranularity::Row)
        .pk(&["id"])
        .columns(vec![FieldContract::key(
            "id",
            T::native(datafusion::arrow::datatypes::DataType::UInt32),
            "key",
        )]),
    );
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Inferred,
            "decisions",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "Total null-safe decisions",
        )
        .granularity(DerivationGranularity::Row)
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "key",
            ),
            FieldContract::payload(
                "included",
                T::native(datafusion::arrow::datatypes::DataType::Boolean),
                "Total decision",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let spec = registry.relation("authored.input").unwrap();
    let rows = vec![
        vec![
            Cell::U64(3),
            Cell::Bool(false),
            Cell::F64(3.),
            Cell::List(vec![]),
            Cell::Null,
        ],
        vec![
            Cell::U64(1),
            Cell::Bool(true),
            Cell::F64(1.),
            Cell::List(vec![Cell::U64(10), Cell::U64(11)]),
            Cell::U64(2),
        ],
        vec![
            Cell::U64(2),
            Cell::Null,
            Cell::F64(2.),
            Cell::Null,
            Cell::U64(3),
        ],
    ];
    let batch = pse_relations::cells::batch_from_cells(&registry, spec, &rows).unwrap();
    let key = spec.key;
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
        native_engine_profile(),
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
fn rule(sql: &str) -> pse_schema::model::RuleSpec {
    let declaration = RuleDecl::new(
        "fixture",
        "1",
        1,
        "inferred.output",
        sql,
        vec![RuleInput {
            relation: "authored.input".into(),
            port: "input",
            mode: DependencyMode::Read,
        }],
    );
    pse_schema::model::RuleSpec {
        id: SemanticId::from_bytes([42; 16]),
        name: declaration.name,
        version: declaration.version,
        stratum: declaration.stratum,
        head: declaration.head,
        assertion_relation: None,
        queries: declaration.queries,
        inputs: declaration.inputs,
        negation: declaration.negation,
        monotonic: declaration.monotonic,
        conflict_policy: declaration.conflict_policy,
    }
}
fn rows(registry: &Registry, batches: &[datafusion::arrow::array::RecordBatch]) -> Vec<Vec<Cell>> {
    batches
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(registry, batch).unwrap())
        .collect()
}
#[tokio::test]
async fn null_safe_comparisons_emit_total_boolean_decisions() {
    let (registry, session, binding) = fixture();
    for (rhs, equal) in [
        ("true", [true, false, false]),
        ("NULL", [false, true, false]),
    ] {
        for distinct in [false, true] {
            let comparison = if distinct {
                "IS DISTINCT FROM"
            } else {
                "IS NOT DISTINCT FROM"
            };
            let mut source = rule(&format!(
                "SELECT id, flag {comparison} {rhs} AS included FROM authored.input"
            ));
            source.head = "inferred.decisions".into();
            let compiled = compile(&source, &binding, &session, &registry)
                .await
                .unwrap();
            let result = execute(&compiled, &session, &registry, &CancellationToken::new())
                .await
                .unwrap();
            assert!(result.head.iter().all(|batch| {
                !batch
                    .schema()
                    .field_with_name("included")
                    .unwrap()
                    .is_nullable()
            }));
            let expected = equal
                .into_iter()
                .enumerate()
                .map(|(index, equal)| {
                    vec![Cell::U64(index as u64 + 1), Cell::Bool(equal != distinct)]
                })
                .collect::<Vec<_>>();
            assert_eq!(rows(&registry, &result.head), expected);
        }
    }
}
#[tokio::test]
async fn projected_join_key_keeps_its_binding_beside_a_qualified_key() {
    let (registry, session, mut binding) = fixture();
    binding.ports.insert("other".into(), binding.ports["input"]);
    let mut source = rule(
        "SELECT projected.id FROM (SELECT parent AS id FROM authored.input AS input WHERE parent IS NOT NULL) AS projected JOIN authored.input AS other ON projected.id = other.id",
    );
    source.inputs.push(RuleInput {
        relation: "authored.input".into(),
        port: "other",
        mode: DependencyMode::Read,
    });
    let compiled = compile(&source, &binding, &session, &registry)
        .await
        .unwrap();
    let result = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        rows(&registry, &result.head),
        vec![vec![Cell::U64(2)], vec![Cell::U64(3)]]
    );
}
#[tokio::test]
async fn registered_native_calls_use_actual_coercion_and_return_fields() {
    let (registry, session, binding) = fixture();
    let source = rule("SELECT coalesce(id, CAST(0 AS BIGINT UNSIGNED)) AS id FROM authored.input");
    let compiled = compile(&source, &binding, &session, &registry)
        .await
        .unwrap();
    let result = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        rows(&registry, &result.head),
        vec![vec![Cell::U64(1)], vec![Cell::U64(2)], vec![Cell::U64(3)]]
    );
}
#[tokio::test]
async fn native_binding_refuses_unknown_functions_and_incompatible_result_fields() {
    let (registry, session, binding) = fixture();
    for sql in [
        "SELECT function_not_in_retained_session(id) AS id FROM authored.input",
        "SELECT make_array(id) AS id FROM authored.input",
    ] {
        assert!(
            compile(&rule(sql), &binding, &session, &registry)
                .await
                .is_err()
        );
    }
}
#[tokio::test]
async fn explicit_predicate_preserves_unknown_candidates_and_orders_actual_keys() {
    let (registry, session, binding) = fixture();
    let mut source = rule("SELECT id FROM authored.input WHERE flag IS TRUE");
    source.queries.push(RuleQuery {
        truth: "unknown",
        sql: "SELECT id FROM authored.input WHERE flag IS NULL".into(),
    });
    let compiled = compile(&source, &binding, &session, &registry)
        .await
        .unwrap();
    let result = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(rows(&registry, &result.head), vec![vec![Cell::U64(1)]]);
    assert_eq!(rows(&registry, &result.undecided), vec![vec![Cell::U64(2)]]);
    assert_eq!(result.rules_fired[0].rows, 1);
    assert!(result.plans[0].explain_pgjson().contains("Plan"));
}
#[tokio::test]
async fn native_union_counts_duplicates_and_distinct_can_use_float_payloads() {
    let (registry, session, binding) = fixture();
    for sql in [
        "SELECT id FROM (SELECT id FROM authored.input UNION ALL SELECT id FROM authored.input) GROUP BY id HAVING count(*) > 1",
        "SELECT id FROM (SELECT DISTINCT * FROM authored.input)",
    ] {
        let compiled = compile(&rule(sql), &binding, &session, &registry)
            .await
            .unwrap();
        let result = execute(&compiled, &session, &registry, &CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            rows(&registry, &result.head),
            vec![vec![Cell::U64(1)], vec![Cell::U64(2)], vec![Cell::U64(3)]]
        );
    }
}
#[tokio::test]
async fn unnest_retains_parent_keys_and_obeys_explicit_null_policy() {
    let (registry, session, binding) = fixture();
    let source = rule(
        "SELECT DISTINCT id FROM (SELECT id, unnest(coalesce(members, [])) AS member FROM authored.input)",
    );
    let compiled = compile(&source, &binding, &session, &registry)
        .await
        .unwrap();
    let result = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(rows(&registry, &result.head), vec![vec![Cell::U64(1)]]);
    let source = rule(
        "SELECT id FROM (SELECT id, unnest(pse_require_nonnull(members)) AS member FROM authored.input)",
    );
    let compiled = compile(&source, &binding, &session, &registry)
        .await
        .unwrap();
    assert!(
        execute(&compiled, &session, &registry, &CancellationToken::new())
            .await
            .is_err()
    );
}
#[tokio::test]
async fn native_recursive_cte_matches_independent_edges_and_cancellation_refuses_completion() {
    let (registry, session, binding) = fixture();
    let cancel = CancellationToken::new();
    let inputs = binding.ports.values().copied().collect::<Vec<_>>();
    let plan = session
        .bind_declared_query(
            "WITH RECURSIVE reach(origin, reached) AS (
        SELECT id, parent FROM authored.input WHERE parent IS NOT NULL
        UNION ALL SELECT reach.origin, edges.parent FROM reach JOIN authored.input AS edges
        ON reach.reached = edges.id WHERE edges.parent IS NOT NULL)
        SELECT origin, reached FROM reach ORDER BY origin, reached",
            &inputs,
            &cancel,
        )
        .await
        .unwrap();
    let result = session
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .into_batches();
    assert_eq!(
        rows(&registry, &result),
        vec![
            vec![Cell::U64(1), Cell::U64(2)],
            vec![Cell::U64(1), Cell::U64(3)],
            vec![Cell::U64(2), Cell::U64(3)]
        ]
    );
    let plan = session
        .bind_declared_query(
            "WITH RECURSIVE repeated(id) AS (
        SELECT id FROM authored.input UNION ALL SELECT id FROM repeated)
        SELECT id FROM repeated",
            &inputs,
            &cancel,
        )
        .await
        .unwrap();
    let mut stream = session
        .prepare(plan, &cancel)
        .unwrap()
        .execute_stream(&cancel)
        .await
        .unwrap();
    assert!(stream.next_batch(&cancel).await.unwrap().is_some());
    cancel.cancel();
    assert!(stream.next_batch(&cancel).await.is_err());
    assert!(!stream.is_complete());
}
