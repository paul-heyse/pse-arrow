// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored recycle initialization keeps physical checks and the original specification.
use super::*;
use crate::math::solves::SolverProfile;
use pse_backend_native::solve::{Controls, SolveIntent};
use std::time::Duration;

#[tokio::test]
async fn reference_recycle_initialization_preserves_scientific_obligations()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    reference_recycle_initialization("01a0f053c2807589b9b24cf0b26a07c9", true).await
}

#[tokio::test]
async fn reference_recycle_initialization_rejects_overheated_stage_without_commit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    reference_recycle_initialization("01a0f053f157702a825ee1acc5a50f11", false).await
}

async fn reference_recycle_initialization(
    fixture: &str,
    expected_success: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (package, _spill, preparation) = pr_jacobian_tests::reference_seed_package().await?;
    let cancel = crate::CancelSource::new();
    let fixture = DeclarationId::from(SemanticId::parse_hex(fixture)?);
    let declared = package
        .declared_execution(
            fixture,
            preparation.compiler,
            SolverProfile {
                intent: SolveIntent::Root,
                controls: Controls {
                    time_limit: Duration::from_secs(600),
                    ..Default::default()
                },
                ..Default::default()
            },
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await?;
    let initialized = package
        .initialize_declared(&declared, InitializationOverrides::default(), &cancel)
        .await?;
    if expected_success {
        assert!(initialized.completed, "{:?}", initialized.failure);
        assert!(initialized.committed.is_some());
        assert!(
            initialized
                .attempts
                .iter()
                .all(|attempt| attempt.accepted())
        );
    } else {
        assert!(!initialized.completed);
        assert!(initialized.committed.is_none());
        let failure = initialized
            .failure
            .as_ref()
            .ok_or("rejected recycle initialization omitted its failure")?;
        assert_eq!(
            failure.class,
            pse_model::diagnostic::BoundaryClass::TrialRejected
        );
        let original = package
            .prepare_analysis(&declared.analysis, &cancel)
            .await?;
        let expected = original
            .model
            .model
            .compiled()
            .model
            .fixtures
            .get(&declared.analysis.instance)
            .ok_or("prepared recycle initialization omitted its fixture")?
            .expected_failure
            .as_ref()
            .ok_or("rejected recycle fixture omitted its expected failure")?;
        assert!(expected.matches(failure), "{failure:?}");
    }
    Ok(())
}

#[tokio::test]
async fn reference_cstr_steady_precision() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (package, _spill, preparation) = pr_jacobian_tests::reference_seed_package().await?;
    let cancel = crate::CancelSource::new();
    let fixture = DeclarationId::from(SemanticId::parse_hex("01a0ee3de15373109faad8251c7d3424")?);
    let declared = package
        .declared_execution(
            fixture,
            preparation.compiler,
            SolverProfile {
                intent: SolveIntent::Root,
                controls: Controls {
                    time_limit: Duration::from_secs(600),
                    ..Default::default()
                },
                ..Default::default()
            },
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await?;
    let prepared = package.prepare_declared(&declared, &cancel).await?;
    eprintln!("CSTR actual profile: {:?}", prepared.profile);
    let result = package
        .solve_case(prepared, preparation.compiler, &cancel)
        .await?;
    eprintln!(
        "CSTR outcome={:?}, completion={:?}",
        crate::math::strategy::observe(&result.outcome),
        result.completion
    );
    let model = &result.prepared.model.model.compiled().model;
    let show = |native: &pse_backend_native::solve::SolveReport| {
        eprintln!(
            "CSTR native: backend={:?}, termination={:?}, qualification={:?}",
            native.backend, native.termination, native.qualification
        );
        if let Some(quality) = &native.quality {
            eprintln!("CSTR quality normalized_max={}", quality.normalized_max);
            for violation in quality
                .rows
                .iter()
                .chain(&quality.bounds)
                .filter(|v| v.physical > 0.)
                .take(20)
            {
                eprintln!(
                    "CSTR physical residual: id={}, residual={}, tolerance={}",
                    violation.id, violation.physical, violation.tolerance
                );
            }
        }
    };
    match &result.outcome {
        crate::math::solves::Outcome::Native(native) => show(native),
        crate::math::solves::Outcome::Constant(report) => {
            eprintln!(
                "CSTR complete reconstruction: components={}, normalized_max={}",
                report.component_reports().len(),
                report.quality.normalized_max
            );
            for native in report.component_reports() {
                show(native);
            }
        }
        crate::math::solves::Outcome::Rejected(cause) => eprintln!("CSTR rejected: {cause}"),
    }
    if let Some(trace) = &result.strategy {
        eprintln!(
            "CSTR strategy: mechanisms={}, events={}",
            trace.declaration.mechanisms.len(),
            trace.events.len()
        );
    }
    for check in result
        .checks
        .iter()
        .filter(|check| !check.satisfied)
        .take(30)
    {
        let path = model
            .symbols
            .get(&check.target_id)
            .map(|symbol| symbol.lineage.path.as_str());
        eprintln!(
            "CSTR failed {:?}: target={} source={} path={path:?} value={} tolerance={:?}",
            check.kind, check.target_id, check.source_id, check.value, check.tolerance
        );
    }
    assert!(result.accepted, "{:?}", result.diagnostic());
    assert!(result.checks.iter().all(|check| check.satisfied));
    Ok(())
}
