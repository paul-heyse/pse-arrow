// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary published-source case journeys, separate from historical oracle fixtures.
use super::*;
use pse_backend_native::solve::{Controls, SolveIntent};
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn ordinary_delivery_admits_complete_source_and_process_definitions() -> TestResult {
    let (runtime, package, _spill, _) = pr_jacobian_tests::ordinary_delivery_package().await?;
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

async fn execute(name: &str, expected: bool) -> TestResult {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package().await?;
    let services = runtime.execution_services();
    let rows = package.declarations(services.preparation()).await?;
    let selected = rows
        .iter()
        .find(|row| row.name == name)
        .ok_or("ordinary case absent")?;
    let case = selected.declaration_id;
    let original_specification = selected.value.clone();
    let cancel = crate::CancelSource::new();
    let declared = package
        .declared_execution(
            services.preparation(),
            case,
            preparation.compiler,
            crate::math::solves::SolverProfile {
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
    match package
        .execute_declared(&services, &declared, &cancel)
        .await?
    {
        DeclaredAlgebraicOutcome::Initialization(report) => {
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
            assert_eq!(
                result.accepted,
                expected,
                "{name}: {:?}",
                result.diagnostic()
            );
            assert_original_checks(&result)?;
            check_exchanger_reports(&result.reports, name == "ordinary_arithmetic_closure")?;
            if name == "ordinary_rating" {
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

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn ordinary_delivery_scientific_intake_publishes_and_reopens_exact_source() -> TestResult {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package().await?;
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
        .declarations(&runtime.preparation_services())
        .await?;
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
        .declarations(&runtime.preparation_services())
        .await?;
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
