// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored recycle initialization keeps physical checks and the original specification.
use super::*;
use crate::math::solves::SolverProfile;
use pse_backend_native::solve::{Controls, SolveIntent};
use std::time::Duration;

#[tokio::test]
async fn reference_recycle_context_reaches_original_transport_obligations()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use pse_model::generated::enums::NumericalTarget;
    let (package, _spill, preparation) = pr_jacobian_tests::reference_seed_package().await?;
    let cancel = crate::CancelSource::new();
    let execution = package
        .declared_execution(
            SemanticId::parse_hex("01a0f053c2807589b9b24cf0b26a07c9")?.into(),
            preparation.compiler,
            SolverProfile {
                intent: SolveIntent::Root,
                ..Default::default()
            },
            Default::default(),
            preparation.limits,
            &cancel,
        )
        .await?;
    let prepared = package
        .prepare_analysis(&execution.analysis, &cancel)
        .await?;
    let model = &prepared.model.model.compiled().model;
    let policy = prepared.solve.numerics();
    let power_rule = SemanticId::parse_hex("395ce3cb36004439af06430d7aea3000")?;
    let names = ["fresh_feed", "mixed", "heated", "liquid", "recycle"];
    let mut counts = BTreeMap::new();
    for closure in model.closures.values().filter(|closure| {
        closure.requirements.len() == 2
            && closure.requirements.iter().all(|requirement| {
                matches!(requirement.tolerance,
                pse_modeling::specialize::ClosureTolerance::EngineeringRule { rule_id, .. }
                if rule_id == power_rule)
            })
    }) {
        let name = names
            .iter()
            .find(|name| closure.lineage.path.ends_with(**name))
            .expect("original energy connection");
        *counts.entry(*name).or_insert(0usize) += 1;
        let resolved = policy
            .targets
            .iter()
            .find(|target| target.id == closure.id && target.kind == NumericalTarget::Closure)
            .expect("resolved original closure");
        let mut budgets = Vec::new();
        for (index, requirement) in closure.requirements.iter().enumerate() {
            let context = requirement
                .context
                .expect("original named transport context");
            assert!(model.symbols.contains_key(&context));
            let endpoint = pse_ids::named_id(closure.id, &format!("physical-endpoint:{index}"));
            let target = policy
                .targets
                .iter()
                .find(|target| target.id == endpoint && target.kind == NumericalTarget::Closure)
                .expect("separately resolved original endpoint");
            assert_eq!(
                (target.quantity, target.unit),
                (resolved.quantity, resolved.unit)
            );
            let engineering = target
                .engineering
                .as_ref()
                .expect("shared engineering interpretation");
            assert_eq!(engineering.rule_id.map(|id| id.as_id()), Some(power_rule));
            assert!(!engineering.canonical_fallback);
            assert!(
                engineering
                    .characteristic
                    .is_some_and(|value| value.is_finite() && value > 0.)
            );
            assert!(policy.policy.engineering_scales.iter().any(|scale| {
                scale.target_id == context && scale.provenance.contains("engineering scale")
            }));
            budgets.push(target.budget);
        }
        assert_eq!(
            resolved.budget,
            budgets.into_iter().reduce(f64::min).unwrap()
        );
    }
    for name in names {
        assert_eq!(
            counts.get(name),
            Some(&4),
            "each feed owns its original {name} energy closure"
        );
    }
    Ok(())
}

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
            preparation.limits,
            &cancel,
        )
        .await?;
    let initialized = package
        .initialize_declared(&declared, InitializationOverrides::default(), &cancel)
        .await?;
    if !initialized.completed {
        eprintln!(
            "recycle initialization failed: attempts={}, failure={:?}",
            initialized.attempts.len(),
            initialized.failure
        );
        for (index, attempt) in initialized
            .attempts
            .iter()
            .enumerate()
            .filter(|(_, attempt)| !attempt.accepted())
            .take(20)
        {
            eprintln!(
                "recycle attempt={index}, step={:?}, interruption={:?}",
                attempt.step, attempt.interruption
            );
            let result = match &attempt.result {
                Ok(result) => result,
                Err(cause) => {
                    eprintln!("recycle preparation/execution failed: {cause}");
                    continue;
                }
            };
            let model = &result.prepared.model.model.compiled().model;
            let path = |id: &SemanticId| {
                model
                    .symbols
                    .get(id)
                    .map(|symbol| symbol.lineage.path.as_str())
                    .or_else(|| {
                        model
                            .equations
                            .iter()
                            .find(|row| row.id == *id)
                            .map(|row| row.lineage.path.as_str())
                    })
                    .or_else(|| {
                        model
                            .closures
                            .get(id)
                            .map(|closure| closure.lineage.path.as_str())
                    })
            };
            eprintln!(
                "recycle outcome={:?}, completion={:?}, validation_error={:?}",
                crate::math::strategy::observe(&result.outcome),
                result.completion,
                result.validation_error
            );
            let show_quality = |quality: &pse_backend_native::quality::Quality| {
                eprintln!("recycle quality normalized_max={}", quality.normalized_max);
                for (kind, violation) in quality
                    .rows
                    .iter()
                    .map(|v| ("row", v))
                    .chain(quality.bounds.iter().map(|v| ("bound", v)))
                    .filter(|(_, v)| v.physical > v.tolerance || !v.physical.is_finite())
                    .take(20)
                {
                    eprintln!(
                        "recycle violated {kind}: id={}, path={:?}, residual={}, tolerance={}",
                        violation.id,
                        path(&violation.id),
                        violation.physical,
                        violation.tolerance
                    );
                }
            };
            let show_native = |native: &pse_backend_native::solve::SolveReport| {
                eprintln!(
                    "recycle native: backend={:?}, code={}, name={}, message={:?}, category={:?}, assurance={:?}, qualification={:?}",
                    native.backend,
                    native.termination.code,
                    native.termination.name,
                    native.termination.message,
                    native.termination.category,
                    native.termination.assurance,
                    native.qualification
                );
                if let Some(candidate) = &native.candidate {
                    eprintln!(
                        "recycle candidate: kind={:?}, variables={}, primal={}",
                        candidate.kind,
                        native.variables.len(),
                        candidate.primal.len()
                    );
                    for (id, value) in native.variables.iter().zip(&candidate.primal) {
                        eprintln!(
                            "recycle primal: id={id}, path={:?}, value={value}",
                            path(id)
                        );
                    }
                }
                if let Some(quality) = &native.quality {
                    show_quality(quality);
                }
                eprintln!(
                    "recycle native events={}, dropped={}; newest retained events first",
                    native.events.len(),
                    native.dropped_events
                );
                for event in native.events.iter().rev().take(5) {
                    eprintln!(
                        "recycle native event: phase={}, elapsed={:?}",
                        event.phase, event.elapsed
                    );
                    for (key, value) in event.values.iter().take(4) {
                        eprintln!("recycle native event value: {key}={value:?}");
                    }
                }
            };
            match &result.outcome {
                crate::math::solves::Outcome::Native(native) => show_native(native),
                crate::math::solves::Outcome::Constant(report) => {
                    eprintln!(
                        "recycle reconstructed components={}",
                        report.component_reports().len()
                    );
                    show_quality(&report.quality);
                    for native in report.component_reports().take(20) {
                        show_native(native);
                    }
                }
                crate::math::solves::Outcome::Rejected(cause) => {
                    eprintln!("recycle rejected: {cause}")
                }
            }
            for check in result
                .checks
                .iter()
                .filter(|check| !check.satisfied)
                .take(20)
            {
                eprintln!(
                    "recycle failed {:?}: source={}, target={}, path={:?}, value={}, tolerance={:?}",
                    check.kind,
                    check.source_id,
                    check.target_id,
                    path(&check.target_id),
                    check.value,
                    check.tolerance
                );
            }
            if let Some(trace) = &result.strategy {
                eprintln!(
                    "recycle strategy: mechanisms={}, events={}; newest relevant events first",
                    trace.declaration.mechanisms.len(),
                    trace.events.len()
                );
                for event in trace
                    .events
                    .iter()
                    .rev()
                    .filter(|event| {
                        event.observation.is_some()
                            || event.permission.is_some()
                            || event.cause.is_some()
                            || event.phase == pse_model::strategy::Phase::Native
                    })
                    .take(20)
                {
                    eprintln!(
                        "recycle strategy event: mechanism={}, kind={:?}, phase={:?}, observation={:?}, original={:?}, permission={:?}, transition={:?}",
                        event.mechanism,
                        event.kind,
                        event.phase,
                        event.observation,
                        event.original,
                        event.permission,
                        event.transition
                    );
                    if let Some(cause) = &event.cause {
                        eprintln!("recycle strategy cause: {cause}");
                    }
                }
            }
        }
    }
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
            preparation.limits,
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
