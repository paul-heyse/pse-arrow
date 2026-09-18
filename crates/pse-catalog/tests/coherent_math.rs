// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent native admission and cold-write oracles for coherent expression rows.
#![allow(
    clippy::unwrap_used,
    reason = "contract fixtures and independent assertions"
)]

#[path = "support/native_execution.rs"]
mod native_execution;

use datafusion::{
    common::TableReference,
    execution::{context::SessionContext, session_state::SessionStateBuilder},
    physical_plan::collect,
};
use deltalake::{
    DeltaTableBuilder, delta_datafusion::SessionFallbackPolicy,
    kernel::transaction::CommitProperties, protocol::SaveMode,
};
use pse_catalog::{
    delta::{admission::violation_plans, contract::DeclaredCheck, write::DeltaWrite},
    session::planner::UnifiedPlanner,
};
use pse_ids::{ContentHash, SemanticId};
use pse_relations::generated::{
    compiled::math_expr_nodes as nodes,
    enums::{Opcode, PublicationKind},
    runtime::publications,
};
use std::sync::Arc;

fn context() -> SessionContext {
    SessionContext::new_with_state(
        SessionStateBuilder::new()
            .with_default_features()
            .with_query_planner(Arc::new(UnifiedPlanner::default()))
            .build(),
    )
}

fn integer(node_id: i64) -> nodes::Row {
    nodes::Row {
        node_id,
        opcode: Opcode::Const,
        children: vec![],
        payload: nodes::CompiledMathExprNodesFieldPayload::from_integer(
            nodes::CompiledMathExprNodesFieldPayloadInteger { value: 7 },
        ),
        quantity_type_id: None,
        scope_instance_id: None,
        subtree_hash: ContentHash::from_bytes([0; 32]),
    }
}

fn batch(rows: Vec<nodes::Row>) -> datafusion::arrow::record_batch::RecordBatch {
    let registry = pse_schema::registry().unwrap();
    let mut builder = nodes::Builder::with_registry(registry, rows.len()).unwrap();
    for row in rows {
        builder.push(row).unwrap();
    }
    builder.finish().unwrap().into_batch()
}

fn candidate() -> publications::Row {
    let registry = pse_schema::registry().unwrap();
    let spec = nodes::spec(registry).unwrap();
    publications::Row {
        workspace_id: SemanticId::NIL,
        publication_id: SemanticId::NIL,
        parent_publication_id: None,
        attempt_id: SemanticId::NIL,
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![publications::RuntimePublicationsFieldMembersItem {
            catalog_name: "datafusion".into(),
            schema_name: "public".into(),
            table_name: "nodes".into(),
            relation_id: spec.id,
            relation_version: i64::from(spec.key.version),
            contract_fingerprint: spec.fingerprint,
            table_uri: "memory://nodes".into(),
            delta_version: 1,
            selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
        }],
    }
}

#[tokio::test]
async fn native_rows_enforce_arity_payload_and_selected_child_closure() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let context = context();
    let reference = TableReference::full("datafusion", "public", "nodes");
    for (children, payload_free, valid) in [
        (vec![0, 0], true, true),
        (vec![0], true, false),
        (vec![0, 0], false, false),
        (vec![0, 9], true, false),
    ] {
        let mut node = integer(1);
        node.opcode = Opcode::Add;
        node.children = children;
        if payload_free {
            node.payload = nodes::CompiledMathExprNodesFieldPayload::from_none();
        }
        context.deregister_table(reference.clone()).unwrap();
        context
            .register_batch(reference.clone(), batch(vec![integer(0), node]))
            .unwrap();
        let state = context.state();
        let plans = violation_plans(&candidate(), Arc::clone(&registry), &state)
            .await
            .unwrap();
        let mut violations = 0;
        for plan in plans {
            let output = collect(
                state.create_physical_plan(&plan).await.unwrap(),
                state.task_ctx(),
            )
            .await
            .unwrap();
            violations += output
                .iter()
                .map(datafusion::arrow::record_batch::RecordBatch::num_rows)
                .sum::<usize>();
        }
        assert_eq!(violations == 0, valid);
    }
}

#[tokio::test]
async fn cold_delta_write_rejects_a_validly_tagged_payload_for_the_wrong_opcode() {
    let registry = pse_schema::registry().unwrap();
    let contract = DeclaredCheck::new(registry, nodes::RELATION_ID).unwrap();
    let context = context();
    let directory = tempfile::tempdir().unwrap();
    let uri = url::Url::from_directory_path(directory.path()).unwrap();
    let input = context
        .read_batch(batch(vec![integer(0)]))
        .unwrap()
        .into_unoptimized_plan();
    let write = DeltaWrite::declared(
        DeltaTableBuilder::from_url(uri.clone())
            .unwrap()
            .build()
            .unwrap(),
        input,
        SaveMode::Append,
        CommitProperties::default(),
        contract,
    )
    .unwrap();
    let state = context.state();
    native_execution::run(&state, pse_schema::shared_registry().unwrap(), &write)
        .await
        .unwrap();
    drop(context);
    drop(state);
    let state = self::context().state();
    let table = DeltaTableBuilder::from_url(uri.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    let version = table.version();
    let state = DeclaredCheck::open(&table, &state)
        .unwrap()
        .bind(&state)
        .unwrap();
    let mut invalid = integer(1);
    invalid.payload = nodes::CompiledMathExprNodesFieldPayload::from_none();
    let result = table
        .write(vec![batch(vec![invalid])])
        .with_save_mode(SaveMode::Append)
        .with_session_state(Arc::new(state))
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .await;
    assert!(result.is_err());
    assert_eq!(
        DeltaTableBuilder::from_url(uri)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        version
    );
}

#[tokio::test]
async fn declared_delta_projections_measure_the_one_coherent_layout() {
    use datafusion::execution::{
        memory_pool::{GreedyMemoryPool, MemoryPool, PeakRecordingPool},
        runtime_env::RuntimeEnvBuilder,
    };
    use std::time::Instant;
    let (_directory, uri, contract, version) = written_nodes().await;
    for (name, query) in [
        ("opcode", "SELECT opcode FROM nodes"),
        (
            "integer_arm",
            "SELECT payload['integer']['value'] AS value FROM nodes",
        ),
        ("whole_node", "SELECT * FROM nodes"),
    ] {
        let pool = Arc::new(PeakRecordingPool::new(Arc::new(GreedyMemoryPool::new(
            128 << 20,
        ))));
        let runtime = Arc::new(
            RuntimeEnvBuilder::new()
                .with_memory_pool(pool.clone())
                .build()
                .unwrap(),
        );
        let state = SessionStateBuilder::new_with_default_features()
            .with_runtime_env(runtime)
            .build();
        let context = SessionContext::new_with_state(state.clone());
        let view = pse_catalog::delta::provider::open_declared_view(
            uri.clone(),
            version,
            &contract,
            Arc::new(state.clone()),
        )
        .await
        .unwrap();
        context.register_table("nodes", Arc::new(view)).unwrap();
        let started = Instant::now();
        let logical = context
            .sql(query)
            .await
            .unwrap()
            .into_optimized_plan()
            .unwrap();
        let physical = state.create_physical_plan(&logical).await.unwrap();
        let planning = started.elapsed();
        println!(
            "coherent_scan_plan name={name} {}",
            datafusion::physical_plan::displayable(physical.as_ref()).indent(false)
        );
        let started = Instant::now();
        let rows = collect(Arc::clone(&physical), state.task_ctx())
            .await
            .unwrap();
        let execution = started.elapsed();
        assert_eq!(
            rows.iter()
                .map(datafusion::arrow::array::RecordBatch::num_rows)
                .sum::<usize>(),
            512
        );
        if name == "integer_arm" {
            assert!(rows.iter().all(|batch| {
                batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::Int64Array>()
                    .unwrap()
                    .values()
                    .iter()
                    .all(|value| *value == 7)
            }));
        }
        let retained = rows
            .iter()
            .map(|row| pse_ids::owned_buffer::retained_buffer_bytes(row).unwrap())
            .sum::<usize>();
        println!(
            "coherent_scan name={name} rows=512 planning_us={} execution_us={} parquet_bytes_scanned={} operator_peak_bytes={} result_buffer_bytes={retained}",
            planning.as_micros(),
            execution.as_micros(),
            scanned_bytes(physical.as_ref()),
            pool.max_reserved()
        );
        drop((rows, physical, logical, context, state));
        assert_eq!(pool.reserved(), 0);
    }
}
fn scanned_bytes(plan: &dyn datafusion::physical_plan::ExecutionPlan) -> usize {
    plan.metrics()
        .and_then(|metrics| metrics.sum_by_name("bytes_scanned"))
        .map_or(0, |metric| metric.as_usize())
        + plan
            .children()
            .into_iter()
            .map(|child| scanned_bytes(child.as_ref()))
            .sum::<usize>()
}

async fn written_nodes() -> (tempfile::TempDir, url::Url, DeclaredCheck, i64) {
    let registry = pse_schema::registry().unwrap();
    let contract = DeclaredCheck::new(registry, nodes::RELATION_ID).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let uri = url::Url::from_directory_path(directory.path()).unwrap();
    let context = context();
    let input = context
        .read_batch(batch((0..512).map(integer).collect()))
        .unwrap()
        .into_unoptimized_plan();
    let write = DeltaWrite::declared(
        DeltaTableBuilder::from_url(uri.clone())
            .unwrap()
            .build()
            .unwrap(),
        input,
        SaveMode::Append,
        CommitProperties::default(),
        contract.clone(),
    )
    .unwrap();
    let state = context.state();
    native_execution::run(&state, pse_schema::shared_registry().unwrap(), &write)
        .await
        .unwrap();
    drop((context, state));
    let version = i64::try_from(
        DeltaTableBuilder::from_url(uri.clone())
            .unwrap()
            .load()
            .await
            .unwrap()
            .version()
            .unwrap(),
    )
    .unwrap();
    (directory, uri, contract, version)
}
