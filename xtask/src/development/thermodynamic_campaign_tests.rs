// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual feed admission and local report custody, independently of full measurement.
use super::*;
use datafusion::arrow::{
    array::ArrayRef,
    datatypes::{DataType, Field, Schema},
};
use pse_runtime::workflow::PreparationSettings;
use std::{collections::HashMap, sync::Arc};

fn feeds(path: &Path, count: usize, reorder: bool, unit: &str) -> Result<()> {
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("index", DataType::Int64, false),
            Field::new("temperature", DataType::Float64, false)
                .with_metadata(HashMap::from([("unit".into(), unit.into())])),
            Field::new("pressure", DataType::Float64, false)
                .with_metadata(HashMap::from([("unit".into(), "Pa".into())])),
            Field::new("benzene", DataType::Float64, false)
                .with_metadata(HashMap::from([("unit".into(), "1".into())])),
        ],
        HashMap::from([("generator".into(), "frozen-test-population".into())]),
    ));
    let indices: ArrayRef = Arc::new(Int64Array::from_iter_values((0..count).map(|index| {
        if reorder && index < 2 {
            1 - index as i64
        } else {
            index as i64
        }
    })));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            indices,
            Arc::new(Float64Array::from_iter_values(
                (0..count).map(|index| 350.0 + 50.0 * index as f64 / FEED_COUNT as f64),
            )),
            Arc::new(Float64Array::from(vec![100000.0; count])),
            Arc::new(Float64Array::from(vec![0.5; count])),
        ],
    )?;
    let mut writer =
        parquet::arrow::ArrowWriter::try_new(std::fs::File::create(path)?, schema, None)?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

#[test]
fn frozen_feed_admission_rejects_partial_reordered_and_wrong_unit_populations() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("feeds.parquet");
    feeds(&path, FEED_COUNT, false, "K")?;
    let admitted = Feeds::load(&path)?;
    assert_eq!(admitted.rows().len(), FEED_COUNT);
    assert_eq!(admitted.rows()[199].index, 199);
    assert_eq!(
        admitted.sha256,
        Sha256::digest(std::fs::read(&path)?)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert_eq!(admitted.generator, "frozen-test-population");
    for (count, reordered, unit) in [
        (199, false, "K"),
        (200, true, "K"),
        (200, false, "degC"),
        (201, false, "K"),
    ] {
        feeds(&path, count, reordered, unit)?;
        assert!(
            Feeds::load(&path).is_err(),
            "invalid population was admitted"
        );
    }
    Ok(())
}

#[test]
fn selected_campaigns_preserve_paired_order_and_entire_value_sweep() {
    assert_eq!(
        Phase::Flash.cases(),
        ["measurement_smooth", "measurement_nested"]
    );
    assert_eq!(Phase::Flash.count(), FEED_COUNT);
    assert_eq!(Phase::Study.cases(), ["measurement_value_sweep"]);
    assert_eq!(Phase::Study.count(), 1000);
    assert_eq!(sweep_temperature(0), 360.0);
    assert_eq!(sweep_temperature(999), 370.0);
    assert!(sweep_temperature(500) > sweep_temperature(499));
}

fn campaign_fixture() -> Result<(tempfile::TempDir, Campaign)> {
    let directory = tempfile::tempdir()?;
    let package = directory.path().join("model");
    std::fs::create_dir_all(package.join("models"))?;
    let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/packages");
    let header = std::fs::read_to_string(packages.join("minimal_explicit/package.toml"))?
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", &format!("dependencies = [{{ package_id = \"{}\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }}]", "5a".repeat(16)));
    std::fs::write(package.join("package.toml"), header)?;
    std::fs::write(
        package.join("models/fixed.pse"),
        r#"package evidence {
        def Good { var x:Scalar; annotation start x(2); eq fixed:x==2; let cost:Scalar=x*x; annotation objective cost(minimize); annotation report x("value"); annotation check x(x>1.5); }
        def Bad { var x:Scalar; annotation start x(1); eq fixed:x==1; let cost:Scalar=x*x; annotation objective cost(minimize); annotation report x("value"); annotation check x(x>1.5); }
    }"#,
    )?;
    let physical = packages.join("physical-primitives");
    let manifest = directory.path().join("conformance.toml");
    // Serialize the current native preparation owner; no copied Python document schema.
    let text = format!(
        "version=1\n[settings]\nversion=1\nmemory_limit_bytes=1073741824\n[[runs]]\nversion=1\nname=\"seed\"\npackage=\"model\"\nphysical={physical:?}\nexecution=\"native\"\n",
        physical = physical.to_string_lossy()
    );
    let mut document: toml::Value = toml::from_str(&text)?;
    document
        .get_mut("settings")
        .and_then(toml::Value::as_table_mut)
        .context("native settings table")?
        .insert(
            "preparation".into(),
            toml::Value::try_from(PreparationSettings::default())?,
        );
    std::fs::write(&manifest, toml::to_string(&document)?)?;
    let path = directory.path().join("feeds.parquet");
    feeds(&path, FEED_COUNT, false, "K")?;
    Ok((directory, Campaign::load(&manifest, "seed", &path)?))
}

#[tokio::test]
async fn paired_phase_refusals_retain_both_cases_and_export_without_python_or_store() -> Result<()>
{
    let (directory, campaign) = campaign_fixture()?;
    let shared = pse_runtime::SharedRuntime::build(super::super::conformance::budget(
        &campaign.settings,
        directory.path(),
    )?)?;
    let factory = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
    let runtime = Runtime::from_shared_local(
        shared.clone(),
        factory.bind_model(pse_schema::shared_registry()?)?,
    )?;
    let report = campaign
        .run(
            &runtime,
            Phase::Flash,
            Execution::Local,
            &CancelSource::new(),
        )
        .await;
    runtime.close();
    runtime.clear_program_cache();
    shared.math().shutdown_native_lanes().await?;
    let report = report?.with_resources(shared.report()?);
    assert_eq!(report.phases.len(), 2);
    for (phase, name) in report.phases.iter().zip(Phase::Flash.cases()) {
        assert!(
            matches!(phase,PhaseResult::Refused { name: actual, error, .. } if actual == name && error.to_string().contains(name))
        );
    }
    assert!(!report.complete_success());
    let output = directory.path().join("paired-refusals");
    report.export(&output)?;
    let summary: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("summary.json"))?)?;
    assert_eq!(
        summary["outcomes"]
            .as_array()
            .context("phase inventory")?
            .len(),
        2
    );
    assert_eq!(summary["solve"]["controls"]["threads"], 1);
    assert_eq!(summary["solve"]["controls"]["time_limit"], 600.0);
    assert!(summary["memory"]["pool_peak_bytes"].as_u64().is_some());
    assert!(
        summary["outcomes"][0]["refusal"]
            .as_str()
            .is_some_and(|value| value.contains("measurement_smooth"))
    );
    assert!(
        report.export(&output).is_err(),
        "existing evidence cannot be replaced"
    );
    Ok(())
}

#[tokio::test]
async fn mixed_original_occurrences_keep_failed_science_and_owned_tables_after_shutdown()
-> Result<()> {
    let (directory, campaign) = campaign_fixture()?;
    let shared = pse_runtime::SharedRuntime::build(super::super::conformance::budget(
        &campaign.settings,
        directory.path(),
    )?)?;
    let factory = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
    let runtime = Runtime::from_shared_local(
        shared.clone(),
        factory.bind_model(pse_schema::shared_registry()?)?,
    )?;
    let outcome = async {
        let cancel = CancelSource::new();
        let (_, package) = campaign.package(&runtime, &cancel).await?;
        let services = runtime.execution_services();
        let declarations = package.declarations(services.preparation()).await?;
        let points = ["Good", "Bad"]
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let case = declarations
                    .iter()
                    .find(|row| row.name == *name)
                    .context("fixture declaration")?
                    .declaration_id;
                Ok(StudyPoint {
                    operation: OperationRequest::DeclaredCase(CaseOperation {
                        case,
                        route: ModelingAnalysisRoute::Steady,
                        settings: SolveSettings {
                            backend: Some(NativeBackend::Clarabel),
                            intent: NativeSolveIntent::Optimize,
                            ..Default::default()
                        },
                    }),
                    preparation: PreparationSettings::default(),
                    overlay: PointOverlay::default(),
                    policy: StudyPointPolicy {
                        key: OccurrenceKey(index as u32),
                        dependencies: vec![],
                        start: StartPolicy::Fresh,
                        attempt_limit: 1,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let local = package
            .admit_study(
                &services,
                &StudyInputs::Explicit(points),
                &StudyOutputDemand::Full {},
                &cancel,
            )
            .await?;
        let report = local.execute(&services, 2, &cancel).await?;
        Ok::<_, anyhow::Error>(
            FrozenPhase::capture(
                "measurement_value_sweep",
                report,
                Phase::Study,
                &campaign.feeds,
                0.0,
            )
            .await,
        )
    }
    .await;
    runtime.close();
    runtime.clear_program_cache();
    shared.math().shutdown_native_lanes().await?;
    let frozen = outcome?;
    assert_eq!(frozen.occurrences.len(), 2);
    assert!(frozen.occurrences[0].usable, "{:?}", frozen.report.outcomes);
    assert!(
        !frozen.occurrences[1].usable,
        "false authored check remains failed science"
    );
    assert_eq!(
        frozen
            .table
            .as_ref()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?
            .batch()
            .num_rows(),
        2
    );
    assert!(
        frozen
            .occurrences
            .iter()
            .all(|point| point.completion.is_some())
    );
    let report = CampaignReport {
        phase: Phase::Study,
        execution: Execution::Local,
        source: campaign.run,
        settings: campaign.settings,
        solve: campaign.solve,
        resources: None,
        feed_sha256: campaign.feeds.sha256,
        feed_generator: campaign.feeds.generator,
        phases: vec![PhaseResult::Completed(Box::new(frozen))],
    };
    let output = directory.path().join("mixed-results");
    report.export(&output)?;
    let lines = std::fs::read_to_string(output.join("measurement_value_sweep.jsonl"))?;
    let rows = lines
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["usable"], true);
    assert_eq!(rows[1]["usable"], false);
    assert!(
        !report.complete_success(),
        "a two-occurrence control is not a qualified thousand-point campaign"
    );
    Ok(())
}
