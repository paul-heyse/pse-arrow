// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native composition consumes real children and owns no predecessor after completion.
#[path = "support/strata_fixture.rs"]
mod fixture;
use datafusion::{
    arrow::array::{Array, StructArray, UInt32Array},
    common::Column,
    logical_expr::{Expr, Extension, LogicalPlan, LogicalPlanBuilder, lit},
};
use fixture::{Fixture, builder, declare, head, id, input};
use pse_ids::CancellationToken;
use pse_rules::strata::{StratumLimits, native::plan_strata};
use pse_schema::model::{Cell, RuleDecl, RuleHead, RulePlan};
use std::sync::Arc;

fn fixture() -> Fixture {
    let mut b = builder();
    input(&mut b, vec![id("id")], &["id"]);
    head(&mut b, "facts", "fact_assertions", &["id"], vec![id("id")]);
    declare(
        &mut b,
        RuleDecl::new(
            "copy",
            "1",
            0,
            RuleHead::Relation("inferred.facts".into()),
            RulePlan::Scan {
                relation: "authored.input".into(),
                port: "input",
            },
        )
        .assertions("provenance.fact_assertions"),
    );
    Fixture::new(b, vec![vec![Cell::U64(17)], vec![Cell::U64(29)]], 2)
}

#[tokio::test]
async fn native_plan_composes_with_unnest_and_releases_input_owners() {
    let fixture = fixture();
    let weak = Arc::downgrade(&fixture.facts);
    let cancel = CancellationToken::new();
    let (session, plan) = plan_strata(
        &fixture.rules,
        &fixture.bindings,
        &fixture.session,
        &fixture.registry,
        &fixture.outputs,
        StratumLimits {
            max_rounds: 4.try_into().unwrap(),
        },
        &cancel,
    )
    .unwrap();
    assert!(plan.display_pg_json().to_string().len() < 16 * 1024);
    let column = Column::new_unqualified("inferred.facts");
    let plan = LogicalPlanBuilder::from(plan)
        .project(vec![Expr::Column(column.clone())])
        .unwrap()
        .unnest_column(column)
        .unwrap()
        .build()
        .unwrap();
    let prepared = session.prepare_rule_plan(plan, &cancel).unwrap();
    assert!(format!("{prepared:?}").len() < 4096);
    // Preparation observes no executed fixed-point loop.
    assert!(session.execution_observations().unwrap().is_empty());
    let done = prepared.execute(&cancel).await.unwrap();
    let observations = session.execution_observations().unwrap();
    assert!(observations.iter().any(|o| {
        o.physical_plan()
            .is_some_and(|p| p.contains("PseFixedPointExec") && p.contains("DataSourceExec"))
    }));
    let batches = done.into_batches();
    drop(session);
    drop(fixture);
    assert!(weak.upgrade().is_none());
    let mut values = Vec::new();
    for batch in batches {
        let facts = batch
            .column(0)
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let ids = facts
            .column(0)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        values.extend((0..ids.len()).map(|row| ids.value(row)));
    }
    values.sort_unstable();
    assert_eq!(values, vec![17, 29]);
}

#[tokio::test]
async fn replaced_native_child_is_executed_and_different_evidence_refused() {
    let fixture = fixture();
    let cancel = CancellationToken::new();
    let (session, plan) = plan_strata(
        &fixture.rules,
        &fixture.bindings,
        &fixture.session,
        &fixture.registry,
        &fixture.outputs,
        StratumLimits {
            max_rounds: 4.try_into().unwrap(),
        },
        &cancel,
    )
    .unwrap();
    let LogicalPlan::Extension(extension) = plan else {
        panic!("native extension required");
    };
    assert_eq!(extension.node.inputs().len(), 1);
    let child = LogicalPlanBuilder::from(extension.node.inputs()[0].clone())
        .filter(lit(false))
        .unwrap()
        .build()
        .unwrap();
    let node = extension
        .node
        .with_exprs_and_inputs(Vec::new(), vec![child])
        .unwrap();
    let error = session
        .prepare_rule_plan(LogicalPlan::Extension(Extension { node }), &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("located input differs from its actual bound owner"),
        "{error}"
    );
}
