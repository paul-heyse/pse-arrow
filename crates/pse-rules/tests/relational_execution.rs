// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent expected rows exercise the actual sealed-session relational compiler.
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    profile::phase0_reference_profile,
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId};
use pse_rules::{
    exec::execute,
    plan::{PortBinding, compile},
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, DerivationGranularity, LogicalType as T, Namespace,
        RelationDecl, RuleDecl, RuleExpr, RuleHead, RulePlan, SnapshotClass,
    },
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[expect(
    clippy::unwrap_used,
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
            ColumnSpec::key("id", T::U32, "key"),
            ColumnSpec::payload("flag", T::Bool, "predicate").optional(),
            ColumnSpec::payload("value", T::F64, "measure"),
            ColumnSpec::payload("members", T::list(T::U32), "orderedmembers").optional(),
            ColumnSpec::reference("parent", T::U32, "graph parent").optional(),
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
            ColumnSpec::key("origin", T::U32, "start"),
            ColumnSpec::key("reached", T::U32, "end"),
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
        .columns(vec![ColumnSpec::key("id", T::U32, "key")]),
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
fn rule(plan: RulePlan) -> pse_schema::model::RuleSpec {
    let declaration = RuleDecl::new(
        "fixture",
        "1",
        1,
        RuleHead::Relation("inferred.output".to_owned()),
        plan,
    );
    pse_schema::model::RuleSpec {
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
fn scan() -> RulePlan {
    RulePlan::Scan {
        relation: "authored.input".to_owned(),
        port: "input",
    }
}
fn keys(input: RulePlan) -> RulePlan {
    RulePlan::Project {
        input: Box::new(input),
        columns: vec![("id", RuleExpr::Col("id"))],
    }
}
#[tokio::test]
async fn root_predicate_preserves_unknown_candidates_and_orders_actual_keys() {
    let (registry, session, binding) = fixture();
    let rule = rule(keys(RulePlan::Filter {
        input: Box::new(scan()),
        predicate: RuleExpr::Col("flag"),
    }));
    let compiled = compile(&rule, &binding, &session, &registry).unwrap();
    let outcome = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    let actual: Vec<_> = outcome
        .head
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(&registry, batch).unwrap())
        .collect();
    let unknown: Vec<_> = outcome
        .undecided
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(&registry, batch).unwrap())
        .collect();
    assert_eq!(actual, vec![vec![Cell::U64(1)]]);
    assert_eq!(unknown, vec![vec![Cell::U64(2)]]);
    assert_eq!(outcome.rules_fired[0].rows, 1);
    assert!(outcome.explain_pgjson.contains("Plan"));
}
#[tokio::test]
async fn duplicate_input_rows_are_visible_to_count_and_float_keys_are_rejected() {
    let (registry, session, binding) = fixture();
    let mut duplicate = keys(scan());
    duplicate = RulePlan::Union(vec![duplicate.clone(), duplicate]);
    let count = RulePlan::Aggregate {
        input: Box::new(duplicate),
        group: vec!["id"],
        aggregates: vec![pse_schema::model::RuleAggregate {
            function: pse_schema::model::RuleAggregateFn::Count,
            input: None,
            output_name: "n",
            order_by: vec![],
            null_policy: pse_schema::model::AggregateNullPolicy::Reject,
            empty_policy: pse_schema::model::AggregateEmptyPolicy::Zero,
        }],
    };
    let source = rule(keys(RulePlan::Filter {
        input: Box::new(count),
        predicate: RuleExpr::cmp(
            pse_schema::model::CmpOp::Gt,
            RuleExpr::Col("n"),
            RuleExpr::Lit(Cell::U64(1)),
        ),
    }));
    let compiled = compile(&source, &binding, &session, &registry).unwrap();
    let result = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    let actual: Vec<_> = result
        .head
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(&registry, batch).unwrap())
        .collect();
    assert_eq!(
        actual,
        vec![vec![Cell::U64(1)], vec![Cell::U64(2)], vec![Cell::U64(3)]]
    );
    let bad = rule(keys(RulePlan::Distinct(Box::new(scan()))));
    assert!(matches!(
        compile(&bad, &binding, &session, &registry),
        Err(pse_rules::RuleError::FloatKey { .. })
    ));
}
#[test]
fn reversible_key_codec_checks_every_name_type_and_value_without_hashes() {
    let (registry, _, _) = fixture();
    let rule = rule(keys(scan()));
    let text = pse_rules::derivations::encode_key(&rule, &registry, &[Cell::U64(42)]).unwrap();
    assert_eq!(
        pse_rules::derivations::decode_key(&rule, &registry, &text).unwrap(),
        vec![Cell::U64(42)]
    );
    for altered in [
        text.replace("\"id\"", "\"other\""),
        text.replace("42", "4294967296"),
        text.replace("u64", "text"),
    ] {
        assert!(pse_rules::derivations::decode_key(&rule, &registry, &altered).is_err());
    }
}

#[tokio::test]
async fn unnest_retains_parent_keys_and_obeys_explicit_null_policy() {
    let (registry, session, binding) = fixture();
    let expansion = RulePlan::Unnest {
        input: Box::new(scan()),
        column: "members",
        value_name: "member",
        null_list: pse_schema::model::NullListPolicy::NoMembers,
        empty_list: pse_schema::model::EmptyListPolicy::NoMembers,
    };
    let source = rule(RulePlan::Distinct(Box::new(keys(expansion.clone()))));
    let compiled = compile(&source, &binding, &session, &registry).unwrap();
    let result = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    let actual: Vec<_> = result
        .head
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(&registry, batch).unwrap())
        .collect();
    assert_eq!(actual, vec![vec![Cell::U64(1)]]);
    let mut reject = expansion;
    if let RulePlan::Unnest { null_list, .. } = &mut reject {
        *null_list = pse_schema::model::NullListPolicy::Reject;
    }
    let compiled = compile(&rule(keys(reject)), &binding, &session, &registry).unwrap();
    assert!(
        execute(&compiled, &session, &registry, &CancellationToken::new())
            .await
            .is_err()
    );
}
#[tokio::test]
async fn recursive_closure_matches_independent_edges_and_rejects_unfinished_work() {
    let (registry, session, binding) = fixture();
    let edges = RulePlan::Filter {
        input: Box::new(scan()),
        predicate: RuleExpr::IsNotNull(Box::new(RuleExpr::Col("parent"))),
    };
    let seed = RulePlan::Project {
        input: Box::new(edges.clone()),
        columns: vec![
            ("origin", RuleExpr::Col("id")),
            ("reached", RuleExpr::Col("parent")),
        ],
    };
    let next = RulePlan::Project {
        input: Box::new(edges),
        columns: vec![
            ("from", RuleExpr::Col("id")),
            ("next", RuleExpr::Col("parent")),
        ],
    };
    let step = RulePlan::Project {
        input: Box::new(RulePlan::EquiJoin {
            left: Box::new(RulePlan::RecursiveRef { name: "reach" }),
            right: Box::new(next),
            keys: vec![("reached", "from")],
            null_equality: pse_schema::model::NullEquality::NullEqualsNothing,
        }),
        columns: vec![
            ("origin", RuleExpr::Col("origin")),
            ("reached", RuleExpr::Col("next")),
        ],
    };
    let recursive = RulePlan::Recursive {
        name: "reach",
        seed: Box::new(seed),
        step: Box::new(step),
        is_distinct: false,
        depth_bound: pse_schema::model::DepthBound::SeedRows,
    };
    let mut source = rule(recursive);
    source.head = RuleHead::Relation("inferred.closure".to_owned());
    let compiled = compile(&source, &binding, &session, &registry).unwrap();
    let outcome = execute(&compiled, &session, &registry, &CancellationToken::new())
        .await
        .unwrap();
    let actual: Vec<_> = outcome
        .head
        .iter()
        .flat_map(|batch| pse_relations::cells::decode_columns(&registry, batch).unwrap())
        .collect();
    assert_eq!(
        actual,
        vec![
            vec![Cell::U64(1), Cell::U64(2)],
            vec![Cell::U64(1), Cell::U64(3)],
            vec![Cell::U64(2), Cell::U64(3)]
        ]
    );
    if let RulePlan::Recursive { step, .. } = &mut source.plan {
        **step = RulePlan::RecursiveRef { name: "reach" };
    }
    let compiled = compile(&source, &binding, &session, &registry).unwrap();
    assert!(
        execute(&compiled, &session, &registry, &CancellationToken::new())
            .await
            .is_err()
    );
}
