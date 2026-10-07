// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scientific equations and oracle expectations are owned by the authored seed.
use super::fixtures::*;
use pse_backend_native::solve::{Assurance, Backend, Termination};
use pse_runtime::workflow::{RunReport, RunResult};
use pse_runtime::{CancelSource, math::solves::Outcome};

fn native_failure_context(
    result: &RunResult,
    case: pse_ids::SemanticId,
    backend: Backend,
    presolve: &pse_backend_native::presolve::Policy,
) {
    let report = match result.report() {
        Ok(RunReport::Modeling(reports)) => reports.first().expect("one authored physical case"),
        other => panic!("case={case} backend={backend:?} presolve={presolve:?}; report={other:?}"),
    };
    assert!(
        report.accepted && result.usable(),
        "case={case} backend={backend:?} presolve={presolve:?}; diagnostic={:?}; native/outcome={:?}; assessments={:?}",
        report.diagnostic(),
        report.outcome,
        result.assessments(),
    );
}

#[tokio::test]
async fn authored_physical_nlp_preserves_native_routes_and_original_qualification() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let id = |s| pse_ids::SemanticId::parse_hex(s).unwrap();
    let heater = id("68ba8dc2d6b05d9a9fe1b1a3625d8015");
    let optimization = id("079a378ba3ce46728b32c87f4fe6a3df");
    let flash = id("040af20814bc57abb565c3c7f680be05");
    for backend in [Backend::Ipopt, Backend::Pounce, Backend::Kinsol] {
        let prepared = seed_prepare(
            &package,
            heater,
            profile(backend, false),
            &CancelSource::new(),
        )
        .await
        .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        native_failure_context(&result, heater, backend, &Default::default());
        let report = authored_success(&result);
        assert!(
            report
                .checks
                .iter()
                .filter(|r| r.kind == pse_relations::generated::enums::ModelingCheckKind::Closure)
                .count()
                >= 2
        );
        assert!(!package.declarations().await.unwrap().is_empty());
        assert!(result.table("authored.modeling_declarations").is_err());
        assert!(result.table("authored.computation_models").is_err());
    }
    for (backend, presolve, assurance) in [
        (
            Backend::Ipopt,
            pse_backend_native::presolve::Policy::Off,
            Assurance::LocalStationary,
        ),
        (
            Backend::Pounce,
            pse_backend_native::presolve::Policy::Off,
            Assurance::LocalStationary,
        ),
        (
            Backend::Ipopt,
            pse_backend_native::presolve::Policy::Auto,
            Assurance::LocalStationary,
        ),
        (
            Backend::Pounce,
            pse_backend_native::presolve::Policy::Auto,
            Assurance::LocalStationary,
        ),
    ] {
        let mut settings = profile(backend, true);
        settings.presolve = presolve.clone();
        let result = seed_prepare(&package, optimization, settings, &CancelSource::new())
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        native_failure_context(&result, optimization, backend, &presolve);
        let report = authored_success(&result);
        let Outcome::Native(native) = &report.outcome else {
            panic!("expected native optimization")
        };
        assert!(matches!(
            native.termination.category,
            Termination::Success | Termination::Acceptable
        ));
        assert_eq!(native.termination.assurance, assurance, "{native:?}");
        assert_eq!(
            native.qualification,
            pse_backend_native::solve::Qualification::Stationary
        );
        assert!(
            native
                .quality
                .as_ref()
                .is_some_and(|quality| quality.feasible())
        );
        let kkt = native.evidence.kkt.as_ref().unwrap();
        assert_eq!(kkt.stationarity, Some(true));
        assert_eq!(kkt.complementarity, Some(true));
    }
    for backend in [Backend::Ipopt, Backend::Pounce] {
        let result = seed_prepare(
            &package,
            flash,
            profile(backend, false),
            &CancelSource::new(),
        )
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
        native_failure_context(&result, flash, backend, &Default::default());
        authored_success(&result);
    }
    for case in [
        "fc52409793e44e61adb3eff88946fdb6",
        "efcd1d0ad288438daf6764b4ab25a2a6",
        "8c22c4a4f87141b083bfc0d9442d382c",
        "d84e844726e64a2c9b23d96a4039b4f9",
    ] {
        let result = seed_prepare(
            &package,
            id(case),
            profile(Backend::Ipopt, false),
            &CancelSource::new(),
        )
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
        native_failure_context(&result, id(case), Backend::Ipopt, &Default::default());
        authored_success(&result);
    }
    // Root-only strategies must retain objective and inequality admission refusals.
    assert!(
        seed_prepare(
            &package,
            optimization,
            profile(Backend::Kinsol, false),
            &CancelSource::new()
        )
        .await
        .is_err()
    );
}
