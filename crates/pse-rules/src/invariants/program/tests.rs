// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated query binding and tiny in-memory invariant execution; no store or compiler.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "unit test assertions"
)]

use datafusion::{arrow::array::RecordBatch, execution::runtime_env::RuntimeEnv};
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineSession, ExecutionSettings, ThreadBudget, native_engine_profile};
use pse_ids::{ContentHash, SemanticId};
use pse_schema::{Registry, model::RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    sync::Arc,
};

fn session(registry: &Arc<Registry>, rows: BTreeMap<RelationKey, RecordBatch>) -> EngineSession {
    let threads = NonZeroUsize::new(1).unwrap();
    pse_engine::EngineFactory::new(
        Arc::new(RuntimeEnv::default()),
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: threads,
            target_partitions: threads,
        },
        native_engine_profile(),
    )
    .and_then(|factory| factory.candidate(rows, Arc::clone(registry), &CancellationToken::new()))
    .unwrap()
}

#[tokio::test]
async fn every_declared_query_binds_against_its_exact_native_inputs() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rows = registry
        .relations()
        .iter()
        .map(|spec| {
            (
                spec.key,
                RecordBatch::new_empty(Arc::new(
                    pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
                )),
            )
        })
        .collect();
    let session = session(&registry, rows);
    let cancel = CancellationToken::default();
    let mut failures = Vec::new();
    for invariant in registry.invariants() {
        let inputs = invariant
            .inputs
            .iter()
            .map(|name| registry.relation(name).unwrap().key)
            .collect::<Vec<_>>();
        match session
            .bind_declared_query(&invariant.query, &inputs, &cancel)
            .await
        {
            Ok(plan) => {
                for name in &invariant.key_columns {
                    if let Err(error) = plan.schema().field_with_unqualified_name(name) {
                        failures.push(format!("{}: {error}", invariant.qualified_name()));
                    }
                }
            }
            Err(error) => failures.push(format!(
                "{}: {error:?}\n{}",
                invariant.qualified_name(),
                invariant.query
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} native query binding failures:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
    let inputs = registry
        .invariants()
        .iter()
        .map(|invariant| {
            invariant
                .inputs
                .iter()
                .map(|name| registry.relation(name).unwrap().key)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let queries = registry
        .invariants()
        .iter()
        .zip(&inputs)
        .map(|(invariant, inputs)| (invariant.query.as_str(), inputs.as_slice()))
        .collect::<Vec<_>>();
    let plans = session
        .bind_declared_queries(&queries, &cancel)
        .await
        .unwrap();
    assert_eq!(plans.len(), queries.len());
    for (invariant, plan) in registry.invariants().iter().zip(plans) {
        for key in &invariant.key_columns {
            plan.schema().field_with_unqualified_name(key).unwrap();
        }
    }
}

#[tokio::test]
async fn native_query_binding_rejects_hidden_missing_and_unused_inputs() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry.relation("authored.packages").unwrap();
    let rows = BTreeMap::from([(
        spec.key,
        RecordBatch::new_empty(Arc::new(
            pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
        )),
    )]);
    let session = session(&registry, rows);
    let cancel = CancellationToken::default();
    for (sql, inputs) in [
        ("SELECT package_id FROM authored.packages", vec![]),
        ("SELECT 1", vec![spec.key]),
        ("DELETE FROM authored.packages", vec![spec.key]),
        (
            "SELECT * FROM authored.species",
            vec![registry.relation("authored.species").unwrap().key],
        ),
    ] {
        assert!(
            session
                .bind_declared_query(sql, &inputs, &cancel)
                .await
                .is_err(),
            "{sql}"
        );
        assert!(
            session
                .bind_declared_queries(
                    &[
                        ("SELECT package_id FROM authored.packages", &[spec.key]),
                        (sql, &inputs),
                    ],
                    &cancel
                )
                .await
                .is_err(),
            "a valid sibling cannot admit {sql}"
        );
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one duplicate-key fixture compares separate and batched registry/native obligations"
)]
async fn native_duplicate_keys_produce_one_typed_finding() {
    use pse_engine::session::policy::RequirementPlanner;
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let spec = registry.relation("authored.packages").unwrap();
    let row = vec![
        serde_json::json!(["id", (SemanticId::from_bytes([7; 16])).to_hex()]),
        serde_json::json!(["text", "fixture"]),
        serde_json::json!(["text", "1.0.0"]),
        serde_json::json!(["enum", "model"]),
        serde_json::json!(["enum", "explicit"]),
        serde_json::json!(["list", []]),
        serde_json::json!(["hash", (ContentHash::NIL).to_hex()]),
        serde_json::json!(["text", "Fixture"]),
    ];
    let rows = BTreeMap::from([(
        spec.key,
        pse_relations::testing::batch_from_literals(&registry, spec, &[row.clone(), row]).unwrap(),
    )]);
    let session = session(&registry, rows.clone());
    let invariant = registry
        .invariants()
        .iter()
        .find(|invariant| {
            invariant.relation == "authored.packages" && invariant.name == "unique:pk"
        })
        .unwrap();
    let report = crate::invariants::run_invariants(
        &rows,
        &session,
        &registry,
        crate::invariants::InvariantScope::Required(&BTreeSet::from([invariant.id])),
        &CancellationToken::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.check_count(), 1);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report
            .findings()
            .iter()
            .map(|batch| batch.batch().num_rows())
            .sum::<usize>(),
        1
    );
    // Mix a registry query with a native row check. Their plans share admission,
    // but the empty relation must not inherit the duplicate-key finding.
    let native = registry
        .relations()
        .iter()
        .find(|spec| !spec.checks.is_empty())
        .unwrap();
    let native_id = native
        .row_check_id(native.checks.keys().next().unwrap())
        .unwrap();
    let mut rows = rows;
    rows.insert(
        native.key,
        RecordBatch::new_empty(Arc::new(
            pse_schema::arrow::relation_schema(&registry, native).unwrap(),
        )),
    );
    let mixed = self::session(&registry, rows.clone());
    let selected = BTreeSet::from([invariant.id, native_id]);
    let cancel = CancellationToken::new();
    let plans = super::compile_individual(
        &rows.keys().copied().collect(),
        &mixed,
        &registry,
        crate::invariants::InvariantScope::Required(&selected),
        &cancel,
    )
    .await
    .unwrap();
    assert_eq!(
        plans.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>(),
        selected
    );
    for (id, plan) in plans {
        let result = mixed
            .prepare(plan, &cancel)
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
            usize::from(id == invariant.id)
        );
    }
    let plan = crate::invariants::RegistryRequirementPlanner
        .plan(&mixed, &selected, &cancel)
        .await
        .unwrap();
    let error = mixed
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap_err();
    assert!(
        pse_diagnostics::diagnostic_leaves(&error)
            .iter()
            .any(|leaf| {
                leaf.diagnostic_code() == Some(pse_diagnostics::DiagnosticCode::SchemaAdmission)
            })
    );
    let mut missing = selected;
    missing.insert(SemanticId::NIL);
    assert!(
        crate::invariants::RegistryRequirementPlanner
            .plan(&mixed, &missing, &cancel)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn stoichiometry_phase_policy_preserves_defaults_restrictions_and_missing_members() {
    let registry = pse_schema::catalog::assemble().unwrap();
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.name == "closure:stoichiometry_species_in_phase")
        .unwrap();
    let context = datafusion::prelude::SessionContext::new();
    for sql in [
        "CREATE SCHEMA authored",
        "CREATE VIEW authored.species AS SELECT * FROM (VALUES (1, CAST(NULL AS VARCHAR[])), (2, ['liquid']), (3, CAST([] AS VARCHAR[]))) AS t(species_id, valid_phase_types)",
        "CREATE VIEW authored.phases AS SELECT * FROM (VALUES (10, 'liquid'), (20, 'aqueous'), (30, 'vapor')) AS t(phase_id, phase_type)",
        "CREATE VIEW authored.phase_species AS SELECT * FROM (VALUES (10, 2)) AS t(phase_id, species_id)",
        "CREATE VIEW authored.stoichiometry AS SELECT * FROM (VALUES (1,10,1), (2,10,2), (3,20,1), (4,30,1), (5,30,2), (6,10,3), (7,99,1), (8,10,99), (9,20,2)) AS t(reaction_id, phase_id, species_id)",
    ] {
        context.sql(sql).await.unwrap().collect().await.unwrap();
    }
    let result = context
        .sql(&invariant.query)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let mut actual = Vec::new();
    for batch in result {
        actual.extend(
            batch
                .column(0)
                .as_any()
                .downcast_ref::<datafusion::arrow::array::Int64Array>()
                .unwrap()
                .values()
                .iter()
                .copied(),
        );
    }
    actual.sort_unstable();
    assert_eq!(actual, [1, 3, 5, 6, 7, 8, 9]);
}
