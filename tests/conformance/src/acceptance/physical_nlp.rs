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
    if !report.accepted || !result.usable() {
        use pse_relations::columnar::RelationRow;
        if let Ok(events) = result.table("runtime.solve_strategy_events") {
            let events =
                pse_relations::generated::runtime::solve_strategy_events::Row::rows(&events)
                    .expect("retained strategy observations");
            for event in events.iter().rev().take(20) {
                eprintln!(
                    "physical strategy: mechanism={:?}, kind={:?}, phase={:?}, observation={:?}, original={:?}, permission={:?}, transition={:?}, failures={:?}",
                    event.mechanism,
                    event.kind,
                    event.phase,
                    event.observation,
                    event.original_conclusion,
                    event.permission,
                    event.transition,
                    event.failures
                );
            }
        }
    }
    assert!(
        report.accepted && result.usable(),
        "case={case} backend={backend:?} presolve={presolve:?}; diagnostic={:?}; native/outcome={:?}; assessments={:?}",
        report.diagnostic(),
        report.outcome,
        result.assessments(),
    );
    if let Outcome::Native(native) = &report.outcome {
        assert_eq!(
            native.backend, backend,
            "case={case} presolve={presolve:?}; {native:?}"
        );
    }
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
        // These package declarations govern production admission as well as
        // historical output comparisons. A missing engineering-rule marker
        // must not silently replace the physical floor with a unit fallback.
        for rule in [
            "13874d4b57684720bb42a48164325812", // temperature
            "cff3158e8b394cdcacb4e10c4ead6a39", // density
            "8fa5f567f8c34ee5a144058621620751", // pressure
            "4b8c1108211445f180f66c602f10b982", // bound duty carrier
            "395ce3cb36004439af06430d7aea3000", // energy-balance power
        ] {
            let rule = id(rule);
            let contexts = prepared
                .solve
                .numerics()
                .targets
                .iter()
                .filter_map(|target| target.engineering.as_ref().map(|context| (target, context)))
                .filter(|(_, context)| {
                    context
                        .rule_id
                        .is_some_and(|selected| selected.as_id() == rule)
                })
                .collect::<Vec<_>>();
            assert!(
                !contexts.is_empty(),
                "shared engineering rule {rule} was not admitted"
            );
            for (target, context) in contexts {
                let floor = context.physical_allowance.expect("authored physical floor");
                assert!(floor.is_finite() && floor > 0.);
                assert!(!context.canonical_fallback, "{context:?}");
                assert!(target.budget >= floor, "{target:?}");
            }
        }
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
