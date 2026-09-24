// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registered P2 unnest/anti-join/distinct behavior against an independent set oracle.
use pse_columnar::CancellationToken;
use pse_engine::session::{ExecutionSettings, ThreadBudget};
use pse_ids::SemanticId;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    sync::Arc,
};
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn phase_row(phase: u8) -> Vec<serde_json::Value> {
    vec![
        serde_json::json!(["id", (id(phase)).to_hex()]),
        serde_json::json!(["id", (id(90)).to_hex()]),
        serde_json::json!(["text", format!("phase{phase}")]),
        serde_json::json!(["enum", pse_material::PhaseType::Liquid.as_str()]),
        serde_json::json!(["bool", false]),
        serde_json::json!(["text", "declared phase"]),
    ]
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "complete target fixture construction and its independent assertions are kept in execution order"
)]
async fn relational_expansion_reference_matches_complete_actual_p2_keys() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry.relation("authored.material_systems").unwrap();
    let phases = registry.relation("authored.phases").unwrap();
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.name == "closure:material_system_phases_exist")
        .unwrap();
    let systems = [
        (100, vec![]),
        (101, vec![1, 2]),
        (102, vec![3, 3]),
        (103, vec![2, 3]),
        (104, vec![2]),
    ];
    for (known, reverse) in [(vec![1, 2], false), (vec![2, 1], true), (vec![], false)] {
        let present: BTreeSet<_> = known.iter().copied().collect();
        let expected: Vec<_> = systems
            .iter()
            .filter(|(_, members)| members.iter().any(|member| !present.contains(member)))
            .map(|(system, _)| vec![serde_json::json!(["id", (id(*system)).to_hex()])])
            .collect();
        let mut system_rows: Vec<_> = systems
            .iter()
            .map(|(system, members)| {
                vec![
                    serde_json::json!(["id", (id(*system)).to_hex()]),
                    serde_json::json!(["id", (id(90)).to_hex()]),
                    serde_json::json!(["text", format!("system{system}")]),
                    serde_json::json!(["list", []]),
                    serde_json::json!([
                        "list",
                        members
                            .iter()
                            .map(|member| serde_json::json!(["id", (id(*member)).to_hex()]))
                            .collect::<Vec<serde_json::Value>>()
                    ]),
                    serde_json::json!(["text", "actual declared membership"]),
                ]
            })
            .collect();
        if reverse {
            system_rows.reverse();
        }
        let phase_rows: Vec<_> = known.into_iter().map(phase_row).collect();
        let rows = BTreeMap::from([
            (
                spec.key,
                pse_relations::testing::batch_from_literals(&registry, spec, &system_rows).unwrap(),
            ),
            (
                phases.key,
                pse_relations::testing::batch_from_literals(&registry, phases, &phase_rows)
                    .unwrap(),
            ),
        ]);
        let thread = NonZeroUsize::new(1).unwrap();
        let session = pse_testkit::factory(
            Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: thread,
                target_partitions: thread,
            },
        )
        .and_then(|factory| {
            factory.candidate(rows, Arc::clone(&registry), &CancellationToken::new())
        })
        .unwrap();
        let cancel = CancellationToken::default();
        let inputs = invariant
            .inputs
            .iter()
            .map(|name| registry.relation(name).unwrap().key)
            .collect::<Vec<_>>();
        let plan = session
            .bind_declared_query(&invariant.query, &inputs, &cancel)
            .await
            .unwrap();
        let plan = datafusion::logical_expr::LogicalPlanBuilder::from(plan)
            .sort([datafusion::logical_expr::col("material_system_id").sort(true, false)])
            .unwrap()
            .build()
            .unwrap();
        let result = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let actual: Vec<_> = result
            .batches()
            .iter()
            .flat_map(|batch| pse_relations::testing::literal_rows(batch).unwrap())
            .collect();
        assert_eq!(
            actual, expected,
            "every missing phase is reflected once per actual system key"
        );
        assert!(
            !actual.is_empty(),
            "an over-pruning empty implementation would fail the oracle"
        );
        assert!(
            result
                .prepared()
                .optimized_plan()
                .schema()
                .has_column_with_unqualified_name("material_system_id")
        );
    }
}
