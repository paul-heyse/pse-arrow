// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary published-source case journeys, separate from historical oracle fixtures.
use super::*;
use pse_backend_native::solve::{Controls, SolveIntent};
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn ordinary_delivery_admits_complete_source_and_process_definitions() -> TestResult {
    let (runtime, package, _spill, _) = pr_jacobian_tests::ordinary_delivery_package(false).await?;
    let rows = package
        .declarations(&runtime.preparation_services())
        .await?;
    for name in [
        "ordinary_recycle",
        "ordinary_overheated",
        "ordinary_exchanger",
        "ordinary_rating",
        "ordinary_arithmetic_closure",
    ] {
        let case = rows
            .iter()
            .find(|row| row.name == name)
            .ok_or("ordinary case absent")?;
        assert_eq!(
            case.value.kind,
            pse_model::generated::enums::ModelingDeclarationKind::Case,
            "ordinary execution uses the production reader, not an oracle test root"
        );
        assert!(
            case.value
                .scope
                .as_ref()
                .and_then(|scope| scope.fixture.as_ref())
                .is_some(),
            "ordinary authored case keeps its actual specification"
        );
    }
    Ok(())
}

#[allow(
    clippy::print_stderr,
    reason = "optional recording setup and finalization report test evidence independently of science"
)]
async fn execute(name: &str, expected: bool) -> TestResult {
    if std::env::var_os(crate::tooling::capture::SOURCE_RECORDING_REQUEST_ENV).is_none() {
        return execute_journey(name, expected, &mut None, false).await;
    }
    let root = crate::observation::process_diagnostics()
        .hub()
        .start("ordinary.delivery.recording");
    let mut recording = match crate::tooling::capture::source_recording_from_environment(
        root.context(),
        recycle_tests::SOLVE_RECORDING_FAMILIES,
    ) {
        Ok(recording) => recording,
        Err(error) => {
            eprintln!("source-recording diagnostic setup failed: {error}");
            None
        }
    };
    let context = recording
        .as_ref()
        .map_or_else(|| root.context(), |recording| recording.context());
    let result = pse_diagnostics::observation::scope(
        Some(context),
        execute_journey(name, expected, &mut recording, false),
    )
    .await;
    root.finish(if result.is_ok() {
        pse_diagnostics::observation::Outcome::Completed
    } else {
        pse_diagnostics::observation::Outcome::Failed
    });
    if let Some(recording) = recording {
        match recording.finish() {
            Ok(document) => eprintln!("source-recording terminal detail={document:?}"),
            Err(error) => eprintln!("source-recording diagnostic finish failed: {error}"),
        }
    }
    result
}

async fn execute_journey(
    name: &str,
    expected: bool,
    recording: &mut Option<crate::tooling::capture::SourceRecordingSession>,
    independent_reference: bool,
) -> TestResult {
    let (runtime, package, _spill, preparation, reference_inputs) = if independent_reference {
        let (runtime, package, spill, preparation, inputs) =
            pr_jacobian_tests::ordinary_delivery_reference_package().await?;
        (runtime, package, spill, preparation, Some(inputs))
    } else {
        let (runtime, package, spill, preparation) =
            pr_jacobian_tests::ordinary_delivery_package(false).await?;
        (runtime, package, spill, preparation, None)
    };
    let services = runtime.execution_services();
    let rows = package.declarations(services.preparation()).await?;
    let interval_claim = rows
        .iter()
        .find(|row| row.name == "feos_interval_unknown")
        .ok_or("whole caloric interval claim absent")?
        .declaration_id
        .as_id();
    let selected = rows
        .iter()
        .find(|row| row.name == name)
        .ok_or("ordinary case absent")?;
    let case = selected.declaration_id;
    let original_specification = selected.value.clone();
    let perry_permission = rows
        .iter()
        .find(|row| row.name == "benzene_liquid_cp_extrapolation" && row.parent_id == Some(case));
    assert_eq!(
        perry_permission.is_some(),
        matches!(name, "ordinary_recycle" | "ordinary_overheated"),
        "only the ordinary recycle scenarios authorize this exact liquid-Cp extrapolation"
    );
    if let Some(permission) = perry_permission {
        let policy = permission
            .value
            .permission
            .as_ref()
            .ok_or("record allowance absent")?;
        assert_eq!(
            policy.target_kind,
            pse_model::generated::enums::ModelingPermissionTarget::Records
        );
        assert_eq!(policy.targets.len(), 1);
        assert!(!policy.allow_unknown && policy.allow_extrapolation);
    }
    let cancel = crate::CancelSource::new();
    let declared = package
        .declared_execution(
            services.preparation(),
            case,
            preparation.compiler,
            pr_jacobian_tests::numerical_solver(crate::math::solves::SolverProfile {
                intent: SolveIntent::Root,
                numerics: pse_model::numerics::NumericalPolicy {
                    strict_engineering_context: matches!(
                        name,
                        "ordinary_recycle"
                            | "ordinary_overheated"
                            | "ordinary_exchanger"
                            | "ordinary_rating"
                            | "ordinary_arithmetic_closure"
                    ),
                    ..Default::default()
                },
                controls: Controls {
                    time_limit: Duration::from_secs(600),
                    ..Default::default()
                },
                ..Default::default()
            })?,
            Default::default(),
            preparation.limits,
            &cancel,
        )
        .await?;
    let outcome = if name == "ordinary_rating" {
        DeclaredAlgebraicOutcome::Solve(
            rating_from_accepted_design(&package, &services, &declared, interval_claim, &cancel)
                .await?,
        )
    } else {
        package
            .execute_declared(&services, &declared, &cancel)
            .await?
    };
    if independent_reference {
        ordinary_reference::retain_native_context(
            name,
            &package,
            services.preparation(),
            &declared,
            &outcome,
            &cancel,
        )
        .await?;
    }
    recycle_tests::retain_declared(
        recording,
        &package,
        services.preparation(),
        &declared,
        Some(&outcome),
    );
    match outcome {
        DeclaredAlgebraicOutcome::Initialization(report) => {
            initialization_failure_diagnostics(name, &report);
            assert_eq!(report.completed, expected, "{name}: {:?}", report.failure);
            assert!(
                !report.attempts.is_empty(),
                "initialization retains actual attempts"
            );
            assert_eq!(report.committed.is_some(), expected);
            if expected {
                assert!(
                    report.attempts.len() >= 2,
                    "the stage and restored original are distinct attempts"
                );
                let original = report
                    .attempts
                    .last()
                    .ok_or("original initialization attempt absent")?;
                assert_eq!(original.step, ModelingInitializationStep::Original);
                let result = original
                    .result
                    .as_ref()
                    .map_err(|error| error.to_string())?;
                assert!(result.accepted);
                check_recycle_reports(&result.reports)?;
                assert_original_checks(result)?;
                assert_unknown_caloric_interval(result, interval_claim)?;
                let claim = rows
                    .iter()
                    .find(|row| row.name == "correlation_range")
                    .ok_or("published temperature-correlation range claim absent")?
                    .declaration_id
                    .as_id();
                assert_perry_extrapolation(
                    result,
                    claim,
                    case.as_id(),
                    perry_permission
                        .ok_or("ordinary recycle record allowance absent")?
                        .declaration_id
                        .as_id(),
                )?;
                if independent_reference {
                    ordinary_reference::check(
                        name,
                        &package,
                        &declared.analysis,
                        result,
                        false,
                        services.preparation(),
                        reference_inputs
                            .as_ref()
                            .ok_or("original reference snapshot absent")?,
                    )?;
                }
            } else {
                let failure = report
                    .failure
                    .as_ref()
                    .ok_or("intended stage failure absent")?;
                assert_eq!(
                    failure.class,
                    pse_model::diagnostic::BoundaryClass::TrialRejected
                );
                let attempt = report
                    .attempts
                    .iter()
                    .find(|attempt| {
                        attempt.step == ModelingInitializationStep::Stage("overheated_tear".into())
                    })
                    .ok_or("intended overheated stage was not attempted")?;
                assert!(!attempt.accepted());
                assert!(
                    attempt.result.is_ok(),
                    "setup failure cannot satisfy a scientific rejection control"
                );
                assert_overheated_contradiction(
                    attempt.result.as_ref().map_err(|cause| cause.to_string())?,
                )?;
                if independent_reference {
                    ordinary_reference::check(
                        name,
                        &package,
                        &declared.analysis,
                        attempt.result.as_ref().map_err(|cause| cause.to_string())?,
                        true,
                        services.preparation(),
                        reference_inputs
                            .as_ref()
                            .ok_or("original reference snapshot absent")?,
                    )?;
                }
                let original = package
                    .prepare_analysis(services.preparation(), &declared.analysis, &cancel)
                    .await?;
                let expected_failure = original
                    .model
                    .model
                    .compiled()
                    .model
                    .fixtures
                    .get(&declared.analysis.instance)
                    .and_then(|fixture| fixture.expected_failure.as_ref())
                    .ok_or("authored intended-stage failure correspondence absent")?;
                assert!(expected_failure.matches(failure), "{failure:?}");
            }
            assert_eq!(report.table()?.batch().num_rows(), 1);
        }
        DeclaredAlgebraicOutcome::Solve(result) => {
            failure_diagnostics(name, &result);
            assert_eq!(
                result.accepted,
                expected,
                "{name}: {:?}",
                result.diagnostic()
            );
            assert_original_checks(&result)?;
            assert_unknown_caloric_interval(&result, interval_claim)?;
            check_exchanger_reports(&result.reports, name == "ordinary_arithmetic_closure")?;
            if independent_reference {
                ordinary_reference::check(
                    name,
                    &package,
                    &declared.analysis,
                    &result,
                    false,
                    services.preparation(),
                    reference_inputs
                        .as_ref()
                        .ok_or("original reference snapshot absent")?,
                )?;
            }
            if name == "ordinary_rating" {
                let model = &result.prepared.model.model.compiled().model;
                let symbol = |suffix: &str| {
                    model
                        .symbols
                        .values()
                        .find(|symbol| symbol.lineage.path.ends_with(suffix))
                        .map(|symbol| symbol.id)
                        .ok_or("rating specification symbol absent")
                };
                let columns = result.prepared.model.case.compiled().plan.columns();
                assert!(
                    columns.contains(&symbol("exchanger.cold.outlet.T")?),
                    "rating cold outlet remains a free physical variable"
                );
                assert!(
                    !columns.contains(&symbol("exchanger.area")?),
                    "rating area remains fixed"
                );
                near(
                    report_label(&result.reports, "solved heat-transfer area")?,
                    1.,
                    1e-12,
                )?;
            } else {
                near(
                    report_path(&result.reports, "exchanger.cold.outlet.T")?,
                    331.6111111111111,
                    0.1,
                )?;
            }
            // A missing required output or wrong physical flow must fail this same
            // independent comparator, rather than merely shrinking its population.
            let without_energy = result
                .reports
                .iter()
                .filter(|row| !row.path.ends_with("exchanger.cold.outlet.energy_flow"))
                .cloned()
                .collect::<Vec<_>>();
            assert!(
                check_exchanger_reports(&without_energy, name == "ordinary_arithmetic_closure")
                    .is_err()
            );
            let mut wrong_flow = result.reports.clone();
            wrong_flow
                .iter_mut()
                .find(|row| row.path.ends_with("exchanger.hot.outlet.flow"))
                .ok_or("hot outlet flow report absent")?
                .value += 0.1;
            assert!(
                check_exchanger_reports(&wrong_flow, name == "ordinary_arithmetic_closure")
                    .is_err()
            );
        }
    }
    let after = package.declarations(services.preparation()).await?;
    assert_eq!(
        after
            .iter()
            .find(|row| row.declaration_id == case)
            .ok_or("original case absent after execution")?
            .value,
        original_specification,
        "temporary stages and solver outcomes cannot mutate the user's specification"
    );
    Ok(())
}

/// Client composition of public Rust operations. One immutable case supplies both
/// questions; an accepted design point supplies only a primal input to the rating.
async fn rating_from_accepted_design(
    package: &ModelingPackage,
    services: &crate::workflow::ExecutionServices,
    declared: &DeclaredExecution,
    interval_claim: SemanticId,
    cancel: &crate::CancelSource,
) -> Result<ModelingResult, Box<dyn std::error::Error + Send + Sync>> {
    use pse_backend_native::solve::{Backend, SolverSelection, StartPolicy, WarmPayload};
    let revision = package.revision_id().to_owned();
    let original = declared.analysis.clone();
    assert_eq!(
        original.solver.selection,
        SolverSelection::Explicit(Backend::Ipopt)
    );
    let area_path = original
        .case
        .variables
        .keys()
        .find(|path| path.ends_with("root.exchanger.area"))
        .ok_or("admitted rating area path absent")?
        .clone();
    let cold_path = original
        .case
        .values
        .keys()
        .find(|path| path.ends_with("root.exchanger.cold.outlet.T"))
        .ok_or("admitted rating cold outlet path absent")?
        .clone();
    let mut design = original.clone();
    design
        .case
        .variables
        .get_mut(&area_path)
        .ok_or("rating area state absent")?
        .fixed = Some(false);
    design
        .case
        .variables
        .entry(cold_path.clone())
        .or_default()
        .fixed = Some(true);
    design
        .case
        .values
        .insert(cold_path.clone(), 331.6111111111111);
    assert_eq!(
        (design.root, design.instance),
        (original.root, original.instance)
    );
    assert_eq!(design.bindings, original.bindings);
    let prepared = package
        .prepare_analysis(services.preparation(), &design, cancel)
        .await?;
    let accepted = package
        .solve_case(services, prepared, design.compiler, cancel)
        .await?;
    failure_diagnostics("ordinary_rating_design_start", &accepted);
    if !accepted.accepted {
        return Err("the auxiliary design did not establish an original-permitted point".into());
    }
    assert_original_checks(&accepted)?;
    assert_unknown_caloric_interval(&accepted, interval_claim)?;
    check_exchanger_reports(&accepted.reports, false)?;
    near(
        report_path(&accepted.reports, "exchanger.cold.outlet.T")?,
        331.6111111111111,
        0.1,
    )?;
    let mut target = package
        .prepare_analysis(services.preparation(), &original, cancel)
        .await?;
    let target_model = &target.model.model.compiled().model;
    let area = target_model
        .paths
        .get(&area_path)
        .copied()
        .ok_or("actual area binding absent")?;
    let cold = target_model
        .paths
        .get(&cold_path)
        .copied()
        .ok_or("actual cold outlet binding absent")?;
    let source_variables = accepted
        .prepared
        .model
        .case
        .compiled()
        .plan
        .structure()
        .variables();
    let target_variables = target.model.case.compiled().plan.structure().variables();
    assert!(
        source_variables
            .iter()
            .find(|variable| variable.port.id == cold)
            .ok_or("design cold outlet variable absent")?
            .fixed
    );
    assert!(
        !source_variables
            .iter()
            .find(|variable| variable.port.id == area)
            .ok_or("design area variable absent")?
            .fixed
    );
    assert!(
        !target_variables
            .iter()
            .find(|variable| variable.port.id == cold)
            .ok_or("rating cold outlet variable absent")?
            .fixed
    );
    assert!(
        target_variables
            .iter()
            .find(|variable| variable.port.id == area)
            .ok_or("rating area variable absent")?
            .fixed
    );
    let mut values = BTreeMap::new();
    for id in target.model.case.compiled().plan.columns() {
        let source = source_variables
            .iter()
            .find(|variable| variable.port.id == *id)
            .ok_or("accepted source coordinate absent")?;
        let destination = target_variables
            .iter()
            .find(|variable| variable.port.id == *id)
            .ok_or("original rating coordinate absent")?;
        assert_eq!(
            (source.port.quantity, source.port.unit),
            (destination.port.quantity, destination.port.unit)
        );
        let value = accepted
            .values
            .scalars
            .get(id)
            .copied()
            .filter(|value| value.is_finite())
            .ok_or("accepted design does not cover a finite target coordinate")?;
        assert!(values.insert(*id, value).is_none());
    }
    assert_eq!(
        values.len(),
        target.model.case.compiled().plan.columns().len()
    );
    near(
        *values
            .get(&cold)
            .ok_or("newly free cold outlet was dropped from accepted-state transfer")?,
        331.6111111111111,
        1e-12,
    )?;
    assert!(
        !values.contains_key(&area),
        "a source free-area solution cannot overwrite the target's fixed area"
    );
    let mut missing_cold = values.clone();
    missing_cold.remove(&cold);
    assert!(
        target
            .solve
            .clone()
            .with_primal_start(missing_cold)
            .is_err(),
        "partial auxiliary-state coverage must refuse"
    );
    let seeded = target
        .solve
        .clone()
        .with_primal_start(values)
        .map_err(crate::math::MathRuntimeError::from)?;
    let mut seed = seeded
        .requested_start()
        .ok_or("explicit primal start absent")?
        .clone();
    let WarmPayload::Nlp {
        bounds,
        rows,
        barrier,
        working,
        ..
    } = &seed.payload
    else {
        return Err("the selected rating input must be a primal-only NLP start".into());
    };
    assert!(bounds.is_none() && rows.is_none() && barrier.is_none() && working.is_none());
    if let crate::math::solves::Outcome::Native(source) = &accepted.outcome {
        seed.origin = source
            .warm_start
            .as_ref()
            .and_then(|start| start.origin.clone());
    }
    target = target.with_start(seed)?;
    assert_eq!(target.profile.controls.start, StartPolicy::Explicit);
    assert_eq!(
        target.solve.replay_profile().controls.start,
        StartPolicy::Explicit
    );
    assert_eq!(
        original.case, declared.analysis.case,
        "preparation and start transfer cannot mutate the original specification"
    );
    assert_eq!(package.revision_id(), revision);
    let handle = services.start_modeling(vec![target], false, cancel).await?;
    let run = handle.wait().await?;
    let crate::workflow::RunReport::Modeling(results) =
        run.report().map_err(|cause| cause.to_string())?
    else {
        return Err("restored rating did not return its actual modeling report".into());
    };
    let [result] = results.as_slice() else {
        return Err("restored rating must produce one original result".into());
    };
    failure_diagnostics("ordinary_rating_from_design", result);
    assert!(
        result.accepted && run.usable(),
        "an accepted start cannot establish target success without fresh original qualification"
    );
    let permission = run
        .completion()
        .map_err(|cause| cause.to_string())?
        .step_permission(0)
        .ok_or("fresh final rating permission absent")?;
    assert!(permission.permits_result());
    assert_eq!(permission.reference_point, Some(result.run_id.as_id()));
    assert_ne!(
        result.run_id, accepted.run_id,
        "design and rating retain separate original point identities"
    );
    Ok(result.clone())
}

#[allow(
    clippy::print_stderr,
    reason = "failed ordinary scientific controls retain bounded native and physical diagnostics without dumping prepared source or model owners"
)]
fn failure_diagnostics(name: &str, result: &ModelingResult) {
    if result.accepted {
        return;
    }
    let model = &result.prepared.model.model.compiled().model;
    let path = |id: &SemanticId| {
        model
            .symbols
            .get(id)
            .map(|row| row.lineage.path.as_str())
            .or_else(|| {
                model
                    .equations
                    .iter()
                    .find(|row| row.id == *id)
                    .map(|row| row.lineage.path.as_str())
            })
            .or_else(|| model.closures.get(id).map(|row| row.lineage.path.as_str()))
    };
    eprintln!(
        "{name}: original={:?}; validation={:?}",
        result.diagnostic(),
        result.validation_error
    );
    let quality = |quality: &pse_backend_native::quality::Quality| {
        eprintln!(
            "{name}: original normalized maximum={}",
            quality.normalized_max
        );
        for (kind, row) in quality
            .rows
            .iter()
            .map(|row| ("row", row))
            .chain(quality.bounds.iter().map(|row| ("bound", row)))
            .filter(|(_, row)| !row.physical.is_finite() || row.physical > row.tolerance)
            .take(8)
        {
            eprintln!(
                "{name}: violated {kind} path={:?}, value={}, tolerance={}",
                path(&row.id),
                row.physical,
                row.tolerance
            );
        }
    };
    let native = |report: &pse_backend_native::solve::SolveReport| {
        eprintln!(
            "{name}: native={:?}, code={}, name={}, category={:?}, qualification={:?}, message={:?}",
            report.backend,
            report.termination.code,
            report.termination.name,
            report.termination.category,
            report.qualification,
            report.termination.message
        );
        if let Some(candidate) = &report.candidate {
            for (id, value) in report
                .variables
                .iter()
                .zip(&candidate.primal)
                .filter(|(id, _)| {
                    path(id).is_some_and(|path| {
                        [
                            "exchanger.hot.outlet.T",
                            "exchanger.cold.outlet.T",
                            "exchanger.area",
                            "exchanger.delta.first",
                            "exchanger.delta.second",
                            "exchanger.hot.heat",
                            "exchanger.cold.heat",
                        ]
                        .iter()
                        .any(|suffix| path.ends_with(suffix))
                    })
                })
            {
                eprintln!("{name}: retained candidate {:?}={value}", path(id));
            }
        }
        if let Some(observed) = &report.quality {
            quality(observed);
        }
    };
    match &result.outcome {
        crate::math::solves::Outcome::Native(report) => native(report),
        crate::math::solves::Outcome::Constant(report) => {
            quality(&report.quality);
            for component in report.component_reports().take(4) {
                native(component);
            }
        }
        crate::math::solves::Outcome::Rejected(cause) => {
            eprintln!("{name}: execution rejected: {cause}")
        }
    }
    for check in result
        .checks
        .iter()
        .filter(|check| !check.satisfied)
        .take(6)
    {
        eprintln!(
            "{name}: failed {:?}, path={:?}, value={}, tolerance={:?}, applicability={:?}",
            check.kind,
            path(&check.target_id),
            check.value,
            check.tolerance,
            check.applicability_outcome
        );
    }
    for row in result
        .reports
        .iter()
        .filter(|row| {
            row.path.contains("exchanger.")
                && (row.path.ends_with("outlet.T")
                    || row.path.ends_with("energy_flow")
                    || row.path.ends_with(".area"))
        })
        .take(10)
    {
        eprintln!("{name}: physical report {}={}", row.path, row.value);
    }
}

#[allow(
    clippy::print_stderr,
    reason = "initialization failure controls retain bounded actual stage and native causes instead of disguising a numerical stop as the intended physical rejection"
)]
fn initialization_failure_diagnostics(name: &str, report: &ModelingInitializationReport) {
    if report.completed {
        return;
    }
    if let Some(failure) = &report.failure {
        eprintln!(
            "{name}: initialization failure class={:?}, stage={:?}, rule={:?}, sources={:?}, observations={:?}",
            failure.class,
            failure.stage,
            failure.rule,
            failure.sources.iter().take(8).collect::<Vec<_>>(),
            failure.observations.iter().take(4).collect::<Vec<_>>()
        );
    }
    for (ordinal, attempt) in report.attempts.iter().enumerate().take(6) {
        eprintln!(
            "{name}: attempt={ordinal}, step={:?}, accepted={}, interruption={:?}",
            attempt.step,
            attempt.accepted(),
            attempt
                .interruption
                .as_ref()
                .map(|cause| (cause.class, cause.rule))
        );
        match &attempt.result {
            Ok(result) => failure_diagnostics(name, result),
            Err(cause) => {
                let cause = cause.boundary_diagnostic();
                eprintln!(
                    "{name}: attempt={ordinal} produced no result; class={:?}, stage={:?}, rule={:?}, observations={:?}",
                    cause.class,
                    cause.stage,
                    cause.rule,
                    cause.observations.iter().take(4).collect::<Vec<_>>()
                );
            }
        }
    }
}

fn near(actual: f64, expected: f64, tolerance: f64) -> TestResult {
    if actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= tolerance {
        Ok(())
    } else {
        Err(
            format!("physical comparison: {actual} differs from {expected}, tolerance {tolerance}")
                .into(),
        )
    }
}

fn report_value(
    reports: &[ModelingReport],
    matches: impl Fn(&ModelingReport) -> bool,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let mut selected = reports.iter().filter(|row| matches(row));
    let row = selected.next().ok_or("required physical report absent")?;
    if selected.next().is_some() || !row.value.is_finite() {
        return Err("required physical report is ambiguous or nonfinite".into());
    }
    Ok(row.value)
}
fn report_label(
    reports: &[ModelingReport],
    label: &str,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    report_value(reports, |row| row.label == label)
}
fn report_path(
    reports: &[ModelingReport],
    path: &str,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    report_value(reports, |row| row.path.ends_with(path))
}
fn assert_original_checks(result: &ModelingResult) -> TestResult {
    if result.checks.is_empty() || result.checks.iter().any(|check| !check.satisfied) {
        return Err("accepted physical result lacks complete original checks".into());
    }
    if !result
        .checks
        .iter()
        .any(|check| check.kind == pse_model::generated::enums::ModelingCheckKind::Closure)
    {
        return Err("original conservation/transport checks absent".into());
    }
    Ok(())
}
pub(super) fn assert_unknown_caloric_interval(
    result: &ModelingResult,
    claim: SemanticId,
) -> TestResult {
    let checks = result
        .checks
        .iter()
        .filter(|check| check.claim_id == Some(claim))
        .collect::<Vec<_>>();
    if checks.is_empty() {
        return Err("whole caloric interval evidence was not retained".into());
    }
    for check in checks {
        if check.applicability_outcome
            != Some(pse_model::generated::enums::ModelingApplicabilityOutcome::UnknownEvidence)
            || check.unknown_allowed != Some(true)
            || check.permission_ids.is_empty()
            || check.applicability_basis.is_some()
            || check.coverage_id.is_some()
            || !check.input_values.iter().any(|input| input.name == "T0")
            || !check.input_values.iter().any(|input| input.name == "T")
        {
            return Err("caloric integration must retain both physical endpoints and explicitly permitted unknown evidence".into());
        }
    }
    Ok(())
}
fn assert_overheated_contradiction(result: &ModelingResult) -> TestResult {
    use pse_backend_native::solve::{Backend, Qualification, Termination};
    let crate::math::solves::Outcome::Native(native) = &result.outcome else {
        return Err("the overheated control must retain its actual native attempt".into());
    };
    assert_eq!(native.backend, Backend::Ipopt);
    assert_eq!(native.termination.category, Termination::Infeasible);
    assert_eq!(native.qualification, Qualification::Unqualified);
    assert_eq!(native.termination.code, 2);
    assert_eq!(native.termination.name, "Infeasible_Problem_Detected");
    let model = &result.prepared.model.model.compiled().model;
    let connection = model
        .connections
        .values()
        .find(|connection| connection.lineage.path.ends_with("root.recycle"))
        .ok_or("actual overheated recycle connection absent")?;
    let temperature = |port: SemanticId| {
        model
            .material_ports
            .get(&port)
            .and_then(|port| {
                port.coordinates
                    .iter()
                    .find(|(slot, _)| slot.name == "temperature" && slot.indices.is_empty())
            })
            .map(|(_, id)| *id)
            .ok_or("actual material-port temperature coordinate absent")
    };
    let source_coordinate = temperature(connection.from)?;
    let receiver_coordinate = temperature(connection.to)?;
    let source = model
        .ports
        .get(&source_coordinate)
        .ok_or("source coordinate alias absent")?
        .symbol;
    let receiver = model
        .ports
        .get(&receiver_coordinate)
        .ok_or("receiver coordinate alias absent")?
        .symbol;
    let symbol = model
        .symbols
        .get(&source)
        .ok_or("source temperature meaning absent")?;
    let definition = symbol
        .definition
        .and_then(|root| model.resolved.nodes.get(root))
        .ok_or("source temperature definition absent")?;
    let pse_modeling::resolved::Node::Number(number) = definition else {
        return Err(
            "overheated control's temperature must retain its selected literal meaning".into(),
        );
    };
    near(number.value, 500., 0.)?;
    let variable = result
        .prepared
        .model
        .case
        .compiled()
        .plan
        .structure()
        .variables()
        .iter()
        .find(|variable| variable.port.id == receiver)
        .ok_or("receiver temperature variable absent")?;
    let upper = variable
        .upper
        .ok_or("receiver physical temperature upper bound absent")?;
    near(upper, 450., 0.)?;
    let row = connection
        .bindings
        .iter()
        .zip(&connection.rows)
        .find(|((from, to), _)| *from == source_coordinate && *to == receiver_coordinate)
        .map(|(_, row)| *row)
        .ok_or("actual temperature connection row absent")?;
    let violation = native
        .quality
        .as_ref()
        .and_then(|quality| quality.rows.iter().find(|violation| violation.id == row))
        .ok_or("native result omitted original temperature-connection assessment")?;
    near(
        violation.physical,
        number.value - upper,
        2. * violation.tolerance,
    )?;
    assert!(
        violation.physical > violation.tolerance,
        "the native stop must retain the actual50K physical contradiction"
    );
    assert!(
        result.checks.iter().any(|check| check.kind
            == pse_model::generated::enums::ModelingCheckKind::Closure
            && model
                .closures
                .get(&check.target_id)
                .is_some_and(|closure| closure.lineage.declaration
                    == connection.lineage.declaration
                    && closure.lineage.instance == connection.lineage.instance)
            && !check.satisfied),
        "the actual recycle transport closure must fail"
    );
    Ok(())
}
pub(super) fn assert_perry_extrapolation(
    result: &ModelingResult,
    claim: SemanticId,
    scope: SemanticId,
    permission: SemanticId,
) -> TestResult {
    use pse_model::generated::enums::{ModelingApplicabilityOutcome, ModelingPermissionTarget};
    let checks = result
        .checks
        .iter()
        .filter(|check| {
            check.claim_id == Some(claim)
                && check.applicability_outcome == Some(ModelingApplicabilityOutcome::OutsideRegion)
                && check.permission_ids.contains(&permission)
        })
        .collect::<Vec<_>>();
    if checks.is_empty() {
        return Err("published benzene liquid-Cp extrapolation was not retained".into());
    }
    if !checks.iter().any(|check| {
        check
            .input_values
            .iter()
            .any(|input| input.name == "T" && (input.value - 368.).abs() <= 0.1)
    }) {
        return Err(
            "the 368K liquid-Cp use must retain its actual outside-range observation".into(),
        );
    }
    for check in checks {
        let authorization = check
            .applicability_permissions
            .iter()
            .find(|entry| entry.permission_id == permission)
            .ok_or("exact record extrapolation authorization absent")?;
        if check.selected_records.len() != 1
            || authorization.targets != check.selected_records
            || authorization.scope != scope
            || authorization.target_kind != ModelingPermissionTarget::Records
            || authorization.allow_unknown
            || !authorization.allow_extrapolation
            || check.extrapolation_allowed != Some(true)
            || !check.satisfied
        {
            return Err("liquid-Cp extrapolation cannot broaden its exact record, lexical scope or independent flags".into());
        }
    }
    Ok(())
}
fn check_recycle_reports(reports: &[ModelingReport]) -> TestResult {
    for component in ["benzene", "toluene"] {
        let feed = report_label(reports, &format!("ordinary feed {component}"))?;
        let vapor = report_label(reports, &format!("ordinary vapor {component}"))?;
        let purge = report_label(reports, &format!("ordinary purge {component}"))?;
        near(feed, 0.5, 1e-12)?;
        near(vapor + purge, feed, 0.002)?;
    }
    near(
        report_label(reports, "ordinary feed flow")?,
        report_label(reports, "vapor product")? + report_label(reports, "ordinary purge flow")?,
        0.004,
    )?;
    // Cancel internal recycle transport. Only fresh feed, product/purge and heater
    // energy cross the boundary; sum the five 1 W native closure allowances.
    near(
        report_label(reports, "ordinary feed energy")?
            + report_label(reports, "ordinary heater duty")?,
        report_label(reports, "ordinary vapor energy")?
            + report_label(reports, "ordinary purge energy")?,
        5.,
    )
}
fn check_exchanger_reports(reports: &[ModelingReport], arithmetic: bool) -> TestResult {
    for (side, flow) in [("hot", 5.), ("cold", 1.)] {
        near(
            report_path(reports, &format!("exchanger.{side}.inlet.flow"))?,
            flow,
            1e-12,
        )?;
        near(
            report_path(reports, &format!("exchanger.{side}.outlet.flow"))?,
            flow,
            0.002,
        )?;
    }
    let hot_gain = report_path(reports, "exchanger.hot.outlet.energy_flow")?
        - report_path(reports, "exchanger.hot.inlet.energy_flow")?;
    let cold_gain = report_path(reports, "exchanger.cold.outlet.energy_flow")?
        - report_path(reports, "exchanger.cold.inlet.energy_flow")?;
    let hot_duty = report_label(reports, "ordinary hot duty")?;
    let cold_duty = report_label(reports, "ordinary cold duty")?;
    near(hot_gain, hot_duty, 1.)?;
    near(cold_gain, cold_duty, 1.)?;
    near(hot_gain + cold_gain, 0., 3.)?;
    if hot_gain >= 0. || cold_gain <= 0. {
        return Err("selected exchanger must transfer energy from hot to cold side".into());
    }
    let first = report_path(reports, "exchanger.hot.inlet.T")?
        - report_path(reports, "exchanger.cold.inlet.T")?;
    let second = report_path(reports, "exchanger.hot.outlet.T")?
        - report_path(reports, "exchanger.cold.outlet.T")?;
    if first <= 0. || second <= 0. {
        return Err(
            "selected co-current exchanger crosses its positive driving-force domain".into(),
        );
    }
    let mean = if arithmetic {
        (first + second) / 2.
    } else if first == second {
        first
    } else {
        (first - second) / (first / second).ln()
    };
    let area = report_label(reports, "solved heat-transfer area")?;
    let coefficient = report_label(reports, "ordinary heat transfer coefficient")?;
    if area <= 0. {
        return Err("selected exchanger area must be positive".into());
    }
    near(coefficient, 100., 1e-12)?;
    // Two 0.1 K temperature-difference allowances plus the 1 W duty allowance.
    near(
        cold_duty,
        area * coefficient * mean,
        1. + area * coefficient * 0.2,
    )?;
    // Independent dimensional conversion and published SSLW u-tube/stainless
    // arithmetic at the actual area. This retains the limited cost claim.
    let square_feet = area / 0.09290304;
    let log_oversized_area = (square_feet * 1.1).ln();
    let base = (11.3852 - 0.9186 * log_oversized_area + 0.0979 * log_oversized_area.powi(2)).exp();
    let material = 2.7 + (square_feet / 100.).powf(0.07);
    near(
        report_label(reports, "area in square feet")?,
        square_feet,
        1e-8,
    )?;
    near(report_label(reports, "base purchase cost")?, base, 0.01)?;
    near(
        report_label(reports, "ordinary material factor")?,
        material,
        1e-8,
    )?;
    near(
        report_label(reports, "ordinary pressure factor")?,
        0.9803,
        1e-8,
    )?;
    near(
        report_label(reports, "capital cost at CEPCI 500")?,
        base * material * 0.9803 * 1.12,
        0.01,
    )
}

#[tokio::test]
async fn ordinary_delivery_recycle_initialization_retains_original_acceptance() -> TestResult {
    execute("ordinary_recycle", true).await
}

#[tokio::test]
async fn ordinary_delivery_overheated_stage_retains_failure_without_commit() -> TestResult {
    execute("ordinary_overheated", false).await
}

#[tokio::test]
async fn ordinary_delivery_exchanger_design_retains_scoped_costing_result() -> TestResult {
    execute("ordinary_exchanger", true).await
}

#[tokio::test]
async fn ordinary_delivery_exchanger_rating_reuses_physical_definition() -> TestResult {
    execute("ordinary_rating", true).await
}

#[tokio::test]
async fn ordinary_delivery_compatible_closure_preserves_side_balances() -> TestResult {
    execute("ordinary_arithmetic_closure", true).await
}

#[tokio::test]
async fn ordinary_reference_exchanger_design_binary_comparison() -> TestResult {
    execute_journey("ordinary_exchanger", true, &mut None, true).await
}
#[tokio::test]
async fn ordinary_reference_exchanger_rating_binary_comparison() -> TestResult {
    execute_journey("ordinary_rating", true, &mut None, true).await
}
#[tokio::test]
async fn ordinary_reference_exchanger_arithmetic_binary_comparison() -> TestResult {
    execute_journey("ordinary_arithmetic_closure", true, &mut None, true).await
}
#[tokio::test]
async fn ordinary_reference_ideal_recycle_binary_comparison() -> TestResult {
    execute_journey("ordinary_recycle", true, &mut None, true).await
}
#[tokio::test]
async fn ordinary_reference_overheated_recycle_binary_comparison() -> TestResult {
    execute_journey("ordinary_overheated", false, &mut None, true).await
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
#[allow(
    clippy::print_stderr,
    reason = "optional canonical recording reports its independent setup and finalization evidence"
)]
async fn ordinary_delivery_scientific_intake_publishes_and_reopens_exact_source() -> TestResult {
    if std::env::var_os(crate::tooling::capture::SOURCE_RECORDING_REQUEST_ENV).is_none() {
        return ordinary_delivery_scientific_intake_journey(&mut None).await;
    }
    let root = crate::observation::process_diagnostics()
        .hub()
        .start("ordinary.canonical.recording");
    let mut recording = match crate::tooling::capture::source_recording_from_environment(
        root.context(),
        recycle_tests::SOLVE_RECORDING_FAMILIES,
    ) {
        Ok(recording) => recording,
        Err(error) => {
            eprintln!("source-recording diagnostic setup failed: {error}");
            None
        }
    };
    let context = recording
        .as_ref()
        .map_or_else(|| root.context(), |recording| recording.context());
    let result = pse_diagnostics::observation::scope(
        Some(context),
        ordinary_delivery_scientific_intake_journey(&mut recording),
    )
    .await;
    root.finish(if result.is_ok() {
        pse_diagnostics::observation::Outcome::Completed
    } else {
        pse_diagnostics::observation::Outcome::Failed
    });
    if let Some(recording) = recording {
        match recording.finish() {
            Ok(document) => eprintln!("source-recording terminal detail={document:?}"),
            Err(error) => eprintln!("source-recording diagnostic finish failed: {error}"),
        }
    }
    result
}

#[cfg(feature = "canonical-tests")]
#[allow(
    clippy::print_stderr,
    reason = "optional canonical handoff failure remains visible independently of the scientific result"
)]
async fn ordinary_delivery_scientific_intake_journey(
    recording: &mut Option<crate::tooling::capture::SourceRecordingSession>,
) -> TestResult {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(true).await?;
    let physical = package.physical_context().clone();
    let runtime = super::super::durable_tests::durable_runtime_on(runtime);
    let cancel = crate::CancelSource::new();
    let store = runtime.canonical_store()?;
    let project_key = "ordinary-scientific-intake";
    let branch_key = "ordinary-scientific-intake-main";
    let draft_key = "ordinary-scientific-intake-draft";
    store
        .create_project(
            "ordinary-intake-create",
            project_key,
            "Ordinary scientific source intake",
        )
        .await?;
    let project = store
        .project(project_key)
        .await?
        .ok_or("created source project absent")?;
    let base = Box::pin(runtime.project_fork(
        &project,
        branch_key,
        "ordinary-intake-problem",
        "ordinary-intake-fork",
        &package,
    ))
    .await?;
    let original =
        Box::pin(runtime.modeling_revision(base.clone(), physical.clone(), BTreeMap::new()))
            .await?;
    let original_rows = original
        .declaration_path(
            &runtime.preparation_services(),
            &["ordinary_bt", "bt_vapor_caloric"],
            &cancel,
        )
        .await?
        .ok_or("ordinary source bank absent")?;
    let target = original_rows
        .iter()
        .find(|row| row.name == "bt_vapor_caloric")
        .ok_or("ordinary source bank absent")?;
    let artifact = include_bytes!(
        "../../../../../packages/reference/data/ordinary-bt/source-artifacts/feos-poling2000.json"
    );

    // Raw custody is not scientific admission or branch activation. Malformed input
    // must leave the original source available through its original revision.
    let project = store
        .project(project_key)
        .await?
        .ok_or("source project absent after fork")?;
    let branch = store
        .project_branch(branch_key)
        .await?
        .ok_or("source branch absent")?;
    let invalid = b"invalid [[";
    let invalid_intent = Box::pin(runtime.project_draft_save_intent(
        "ordinary-intake-malformed",
        &project,
        &branch,
        draft_key,
        &base.key,
        None,
        invalid,
    ))
    .await?;
    Box::pin(runtime.project_save_draft_intent(&invalid_intent, invalid)).await?;
    assert!(
        Box::pin(runtime.project_admit_draft(
            "ordinary-intake-admit-malformed",
            draft_key,
            physical.clone(),
            &cancel
        ))
        .await
        .is_err()
    );
    assert_eq!(
        store
            .project_branch_revision(branch_key)
            .await?
            .ok_or("branch revision absent")?
            .key,
        base.key
    );
    let mut wrong_parent = scientific_intake::feos_poling2000_bt_caloric_delta(
        original.revision_id(),
        target,
        artifact,
    )?;
    wrong_parent.parent = "another-local-parent".into();
    assert!(
        runtime
            .project_draft_delta(&original, wrong_parent)
            .is_err()
    );

    let delta = scientific_intake::feos_poling2000_bt_caloric_delta(
        original.revision_id(),
        target,
        artifact,
    )?;
    let typed = runtime.project_draft_delta(&original, delta)?;
    let body = serde_json::to_vec(&typed)?;
    let project = store
        .project(project_key)
        .await?
        .ok_or("source project absent after raw custody")?;
    let branch = store
        .project_branch(branch_key)
        .await?
        .ok_or("source branch absent after raw custody")?;
    let draft = store
        .project_draft(draft_key)
        .await?
        .ok_or("saved malformed draft absent")?;
    let intent = Box::pin(runtime.project_draft_save_intent(
        "ordinary-intake-save",
        &project,
        &branch,
        draft_key,
        &base.key,
        Some(draft.generation),
        &body,
    ))
    .await?;
    let first = Box::pin(runtime.project_save_draft_intent(&intent, &body)).await?;
    let replay = Box::pin(runtime.project_save_draft_intent(&intent, &body)).await?;
    assert_eq!(
        first, replay,
        "lost acknowledgement replay retains the same exact raw-source effect"
    );
    let mut different = body.clone();
    different.push(b' ');
    assert!(
        Box::pin(runtime.project_save_draft_intent(&intent, &different))
            .await
            .is_err(),
        "an exact-custody identity cannot accept different bytes"
    );
    let admitted = Box::pin(runtime.project_admit_draft(
        "ordinary-intake-admit",
        draft_key,
        physical.clone(),
        &cancel,
    ))
    .await?;
    let admitted_rows = admitted
        .declaration_path(
            &runtime.preparation_services(),
            &["ordinary_bt", "bt_vapor_caloric"],
            &cancel,
        )
        .await?
        .ok_or("admitted source bank absent")?;
    let admitted_bank = admitted_rows
        .iter()
        .find(|row| row.declaration_id == target.declaration_id)
        .ok_or("admitted source bank absent")?;
    assert_eq!(
        admitted_bank.value.dataset, target.value.dataset,
        "unchanged input preserves the scientific cells and source role"
    );
    assert_eq!(
        store
            .project_branch_revision(branch_key)
            .await?
            .ok_or("branch revision absent")?
            .key,
        base.key,
        "local admission does not publish the branch"
    );

    let current_project = store
        .project(project_key)
        .await?
        .ok_or("source project absent")?;
    let current_branch = store
        .project_branch(branch_key)
        .await?
        .ok_or("source branch absent")?;
    let current_draft = store
        .project_draft(draft_key)
        .await?
        .ok_or("source draft absent")?;
    let publication = store.project_publication_intent(
        &current_project,
        &current_branch,
        &base.key,
        Some(&current_draft),
        "ordinary-intake-publish",
    )?;
    let published =
        Box::pin(runtime.project_publish_draft(&publication, physical.clone(), &cancel)).await?;
    let revision = published
        .canonical_revision()
        .ok_or("published scientific revision absent")?
        .clone();
    let replayed =
        Box::pin(runtime.project_publish_draft(&publication, physical.clone(), &cancel)).await?;
    assert_eq!(
        replayed
            .canonical_revision()
            .ok_or("replayed revision absent")?
            .key,
        revision.key
    );
    assert_eq!(
        store
            .project_acknowledgement("ordinary-intake-publish")
            .await?
            .ok_or("publication acknowledgement absent")?
            .revision
            .as_deref(),
        Some(revision.key.as_str())
    );
    drop((published, replayed, admitted));
    let reopened =
        Box::pin(runtime.modeling_revision(revision, physical.clone(), BTreeMap::new())).await?;
    let rows = reopened
        .declarations(&runtime.preparation_services())
        .await?;
    let bank = rows
        .iter()
        .find(|row| row.declaration_id == target.declaration_id)
        .ok_or("reopened source bank absent")?;
    assert_eq!(
        bank.value.dataset, target.value.dataset,
        "fresh protected read recovers original units, coefficients and provenance"
    );
    let case = rows
        .iter()
        .find(|row| row.name == "ordinary_exchanger")
        .ok_or("ordinary consuming case absent")?
        .declaration_id;
    let services = runtime.execution_services();
    let operation = Box::pin(reopened.prepare_operation(
        services.preparation(),
        &crate::workflow::OperationSource::of(&reopened),
        &crate::workflow::PreparationSettings {
            compiler: preparation.compiler,
            limits: preparation.limits,
        },
        crate::workflow::RuntimeOperationRequest::DeclaredCase {
            case,
            settings: Box::new(crate::math::settings::SolveSettings {
                intent: SolveIntent::Root,
                numerics: pse_model::numerics::NumericalPolicy {
                    strict_engineering_context: true,
                    ..Default::default()
                },
                controls: Controls {
                    time_limit: Duration::from_secs(600),
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
        None,
        &cancel,
    ))
    .await?;
    let handle = operation.start(&services, &cancel).await?;
    let run_key = handle
        .canonical_run_key()
        .ok_or("ordinary consuming run was not recorded")?
        .to_owned();
    let attempt_key = handle
        .canonical_attempt_key()
        .ok_or("ordinary consuming attempt was not recorded")?
        .to_owned();
    let run = handle.wait().await?;
    if let Some(recording) = recording
        && let Err(error) = recording.retain_result(&run)
    {
        eprintln!("source-recording review handoff failed: {error}");
    }
    assert!(
        run.usable(),
        "ordinary published-source consuming run must retain original permission"
    );
    let crate::workflow::RunReport::Modeling(results) =
        run.report().map_err(|error| error.to_string())?
    else {
        return Err("ordinary exchanger did not produce its original modeling result".into());
    };
    assert_eq!(results.len(), 1);
    assert_original_checks(&results[0])?;
    assert_unknown_caloric_interval(
        &results[0],
        rows.iter()
            .find(|row| row.name == "feos_interval_unknown")
            .ok_or("reopened whole caloric interval claim absent")?
            .declaration_id
            .as_id(),
    )?;
    check_exchanger_reports(&results[0].reports, false)?;
    let expected_reports = run
        .table("runtime.modeling_reports")
        .map_err(|error| error.to_string())?
        .batch()
        .clone();
    assert!(
        expected_reports.num_rows() > 0,
        "recorded readback has required physical outputs"
    );
    drop((run, handle, operation));
    let mut readback = runtime
        .results(
            &run_key,
            &attempt_key,
            "runtime.modeling_reports",
            0,
            expected_reports.num_rows() as u64,
            cancel.token().clone(),
        )
        .await?;
    let recovered = readback
        .next_relation_batch()
        .await?
        .ok_or("recorded ordinary reports absent")?;
    assert_eq!(
        recovered.batch(),
        &expected_reports,
        "fresh protected readback preserves the complete ordinary physical outputs and coordinates"
    );
    assert!(readback.next_relation_batch().await?.is_none());
    let historical = Box::pin(runtime.modeling_revision(base, physical, BTreeMap::new())).await?;
    let historical_rows = historical
        .declarations(&runtime.preparation_services())
        .await?;
    assert_eq!(
        historical_rows
            .iter()
            .find(|row| row.declaration_id == target.declaration_id)
            .ok_or("historical source bank absent")?
            .value
            .dataset,
        target.value.dataset,
        "failed and successful later work cannot overwrite the original source revision"
    );
    Ok(())
}
