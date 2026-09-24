// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::*;
use pse_backend_native::solve::{Backend, Termination};
use pse_runtime::{CancelSource, math::solves::Outcome, workflow::RunReport};
#[tokio::test]
async fn publication_resource() {
    let owner = WorkflowRuntime::new().unwrap();
    let rt = runtime(&owner);
    let f = json("bindings.json");
    let revision = builder(&owner).await.freeze().unwrap();
    let case = sid(&f["root_case"]);
    let cancelled = CancelSource::new();
    cancelled.cancel();
    assert!(
        revision
            .prepare(
                case,
                profile(Backend::Ipopt, 3, 3, false),
                compiler(),
                false,
                &cancelled
            )
            .await
            .is_err()
    );
    let prepared = revision
        .prepare(
            case,
            profile(Backend::Ipopt, 3, 3, false),
            compiler(),
            false,
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let a = prepared.start().unwrap();
    let b = prepared.start().unwrap();
    let (a, b) = tokio::join!(a.wait(), b.wait());
    let a = a.unwrap();
    let b = b.unwrap();
    success(&a);
    success(&b);
    assert_ne!(a.run_id, b.run_id);
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let publication = a
        .prepare_publication(base.clone(), case, None, &owner.cancel)
        .unwrap();
    let root = publication.commit(&owner.cancel).await.unwrap();
    let reopened = rt.open(root.clone(), &owner.cancel).await.unwrap();
    assert_eq!(reopened.root().version, root.version);
    let name = datafusion::common::ResolvedTableReference {
        catalog: "artifact".into(),
        schema: "authored".into(),
        table: "computation_models".into(),
    };
    assert!(reopened.member(&name).is_ok());
    let mut wrong = root.clone();
    wrong.version += 1;
    assert!(rt.open(wrong, &owner.cancel).await.is_err());
    let cancelled = pse_columnar::CancellationToken::new();
    cancelled.cancel();
    let interrupted = b
        .prepare_publication(
            base,
            case,
            Some(reopened.record().publication_id),
            &owner.cancel,
        )
        .unwrap();
    assert!(interrupted.commit(&cancelled).await.is_err());
    assert!(rt.open(root, &owner.cancel).await.is_ok());
    // Cancel only after an actual native object write; the control transaction remains absent.
    let faults = pse_testkit::fault_store::FaultStore::new(std::sync::Arc::new(
        object_store::memory::InMemory::new(),
    ));
    let fault_base = url::Url::parse("memory://plan14-interrupted/").unwrap();
    owner
        .runtime
        .runtime_env()
        .object_store_registry
        .register_store(&fault_base, faults.clone());
    let interrupted = pse_columnar::CancellationToken::new();
    faults.arm(pse_testkit::fault_store::FaultPlan {
        operation: "put",
        prefix: String::new(),
        call: 1,
        fault: pse_testkit::fault_store::Fault::Cancel(interrupted.clone()),
    });
    let command = a
        .prepare_publication(fault_base, case, None, &owner.cancel)
        .unwrap();
    assert!(command.commit(&interrupted).await.is_err());
    assert_eq!(faults.fired(), 1);
    let retained = a.table("runtime.solve_variables").unwrap();
    let arrays = retained.batch().columns().to_vec();
    let reserved = owner.runtime.pool().reserved();
    drop(retained);
    drop(a);
    drop(b);
    drop(prepared);
    drop(revision);
    assert!(owner.runtime.pool().reserved() > 0);
    assert!(!arrays.is_empty());
    drop(arrays);
    assert!(owner.runtime.pool().reserved() <= reserved);
    // A limited or cancelled attempt remains an attempt, never an optimum certificate.
    let revision = builder(&owner).await.freeze().unwrap();
    let mut limited = profile(Backend::Ipopt, 3, 3, false);
    limited.controls.iterations = 1;
    let p = revision
        .prepare(case, limited, compiler(), false, &CancelSource::new())
        .await
        .unwrap();
    let result = p.start().unwrap().wait().await.unwrap();
    let RunReport::Solves(report) = result.report().unwrap() else {
        panic!()
    };
    assert!(
        matches!(&report.outcomes[0],Outcome::Native(r) if r.termination.category != Termination::Success)
    );
    assert_eq!(
        result
            .table("runtime.solve_runs")
            .unwrap()
            .batch()
            .num_rows(),
        1
    );
    let p = revision
        .prepare(
            case,
            profile(Backend::Ipopt, 3, 3, false),
            compiler(),
            false,
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let handle = p.start().unwrap();
    handle.cancel();
    let result = handle.wait().await.unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &result,
        &handle.wait().await.unwrap()
    ));
    assert!(result.table("runtime.solve_runs").is_ok());
}
