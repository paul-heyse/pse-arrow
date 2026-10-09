// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical scientific execution and exact historical reopening.
#![allow(
    clippy::unwrap_used,
    clippy::unreachable,
    reason = "durable execution fixtures require valid authored setup and durable operation ownership"
)]
use super::tests::{compiler_profile, physical, profile};
use super::*;
use pse_backend_native::solve::{Backend, ReusePolicy, SolverSelection};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_ids::SemanticId;
use std::time::Duration;

pub(super) const LINEAR: &str = "package p { def Root { param t: Scalar = 1; var x: Scalar; eq e: x == 2+t; annotation start x(2+t); annotation check x(x > 0); } }";

/// A package over `runtime` and its root analysis, solved by KINSOL.
pub(super) async fn package_on(
    runtime: &Runtime,
    source: &str,
) -> (ModelingPackage, ModelingAnalysis) {
    let physical = physical();
    let rows = pse_authoring::language::parse(
        source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    let mut solver = profile();
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    solver.controls.reuse = ReusePolicy::AllowRebuild;
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(),
        limits: Default::default(),
        case: ModelingCaseBindings::default(),
        order: pse_kernels::DerivativeOrder::Second,
        compiler: compiler_profile(),
        solver,
        numerical: Default::default(),
    };
    (package, analysis)
}

/// A lease short enough for tests to observe expiry and heartbeats.
pub(super) fn quick() -> LeasePolicy {
    LeasePolicy {
        lease: Duration::from_secs(5),
        heartbeat: Duration::from_millis(100),
        flush: Duration::from_millis(20),
        ..LeasePolicy::default()
    }
}

/// Attach the same isolated canonical source store and accounted pool to the worker.
pub(super) fn durable_runtime() -> Runtime {
    let runtime = tests::runtime_with_workspace(32 << 20);
    durable_runtime_on(runtime)
}

/// Attach durable operations to an existing worker runtime without changing its budget.
pub(super) fn durable_runtime_on(runtime: Runtime) -> Runtime {
    let operations = Operations::from_store(
        runtime.canonical().store().clone(),
        "native-fixture",
        quick(),
        runtime.shared.pool(),
    );
    runtime.with_durability(Durability::Durable(operations))
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_durable_constant_completion_reopens_exact_manifest() {
    let runtime = durable_runtime();
    let (package,analysis)=package_on(&runtime,"package p { def Root { param x:Scalar=3; annotation report x(\"constant\"); annotation check x(x>0); } }").await;
    let prepared = package
        .prepare_analysis(&analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = prepared.start().unwrap();
    let run = handle.canonical_run_key().unwrap().to_owned();
    let attempt = handle.canonical_attempt_key().unwrap().to_owned();
    assert_eq!(attempt.len(), 64);
    let result = handle.wait().await.unwrap();
    assert!(result.usable(), "{:?}", result.report());
    assert_eq!(result.canonical_run_key(), Some(run.as_str()));
    assert_eq!(result.canonical_attempt_key(), Some(attempt.as_str()));
    let RunDurability::Durable(record) = result.durability() else {
        panic!("ephemeral result")
    };
    assert_eq!(
        record.attempt.as_ref().unwrap().outcome.as_deref(),
        Some("succeeded")
    );
    let Durability::Durable(operations) = runtime.durability() else {
        unreachable!()
    };
    let reopened = operations.record(&run, &attempt).await.unwrap();
    assert_eq!(reopened.manifest, record.manifest);
    assert_eq!(
        reopened.completion.as_ref().unwrap().state,
        pse_model::generated::enums::AttemptState::Completed
    );
    assert!(
        reopened
            .completion
            .as_ref()
            .unwrap()
            .completion
            .as_ref()
            .unwrap()
            .assessments
            .iter()
            .all(|a| a.permits_result)
    );
    let selection = operations
        .store()
        .read_results(&run, &attempt, Duration::from_secs(60))
        .await
        .unwrap();
    let tables = result.tables().unwrap();
    assert!(!tables.is_empty());
    for (relation, table) in tables {
        let descriptor = selection
            .sets()
            .iter()
            .find(|d| d.name == relation.to_string())
            .unwrap();
        assert_eq!(descriptor.row_count, table.batch().num_rows() as u64);
        assert!(descriptor.batch_count > 0);
        let payload = operations
            .store()
            .result_payload(&selection, &descriptor.key, 0)
            .await
            .unwrap();
        assert!(
            payload.batch.payload.len() <= pse_operations::canonical_execution::RESULT_BATCH_BYTES
        );
    }
    drop(selection);
    drop(result);
    drop(handle);
    drop(package);
    runtime
        .canonical()
        .store()
        .remove_isolated_fixture()
        .await
        .unwrap();
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_registration_refusal_never_returns_success_or_fabricates_header() {
    let runtime = durable_runtime();
    let empty = pse_operations::testing::canonical_fixture_store().unwrap();
    let operations = Operations::from_store(
        empty.clone(),
        "refused-fixture",
        quick(),
        runtime.shared.pool(),
    );
    let runtime = runtime.with_durability(Durability::Durable(operations));
    let (package, analysis) = package_on(
        &runtime,
        "package p { def Root { param x:Scalar=3; annotation report x(\"constant\"); } }",
    )
    .await;
    let prepared = package
        .prepare_analysis(&analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = prepared.start().unwrap();
    let run = handle.canonical_run_key().unwrap().to_owned();
    let result = handle.wait().await.unwrap();
    assert!(!result.usable());
    assert!(result.report().is_err());
    assert!(result.completion().is_err());
    assert!(
        matches!(result.report(), Err(WorkflowError::Canonical(_))),
        "original registration cause: {:?}",
        result.report()
    );
    let RunDurability::Durable(record) = result.durability() else {
        panic!("ephemeral result")
    };
    assert!(record.attempt.is_err());
    assert!(record.run.is_none());
    assert!(record.manifest.is_none());
    assert!(
        matches!(&record.attempt,Err(error) if matches!(error.as_ref(),WorkflowError::Canonical(_))),
        "registration receipt must keep the original cause"
    );
    assert!(empty.canonical_run(&run).await.unwrap().is_none());
    drop(result);
    drop(handle);
    drop(package);
    empty.remove_isolated_fixture().await.unwrap();
    runtime
        .canonical()
        .store()
        .remove_isolated_fixture()
        .await
        .unwrap();
}
