// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Receiving controls for campaign selection and owned report publication.
use super::*;

fn fixture() -> Result<(tempfile::TempDir, PathBuf)> {
    let directory = tempfile::tempdir()?;
    let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/packages");
    let package = directory.path().join("model");
    std::fs::create_dir(&package)?;
    std::fs::create_dir(package.join("models"))?;
    let manifest = std::fs::read_to_string(packages.join("minimal_explicit/package.toml"))?
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
    std::fs::write(package.join("package.toml"), manifest)?;
    std::fs::write(
        package.join("models/pure.pse"),
        r#"package native_campaign {
        fn square(x:Scalar)->Scalar=x*x;
        test positive fixture { dof 0; route steady; procedure check; } {
            expect square(2)==4 tolerance 1e-12;
        }
    }"#,
    )?;
    let path = directory.path().join("conformance.toml");
    let physical = packages.join("physical-primitives");
    std::fs::write(
        &path,
        format!(
            r#"version=1
[settings]
version=1
memory_limit_bytes=1073741824
[[runs]]
version=1
name="first"
package="model"
physical={physical:?}
execution="pure"
[[runs]]
version=1
name="second"
package="model"
physical={physical:?}
execution="pure"
"#,
            physical = physical.to_string_lossy()
        ),
    )?;
    Ok((directory, path))
}

#[test]
fn campaign_selection_precedes_execution_and_retains_authored_order() -> Result<()> {
    let (_directory, path) = fixture()?;
    let campaign = Campaign::load(&path, None, None)?;
    assert_eq!(
        campaign
            .runs()
            .map(|run| run.name.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert!(Campaign::load(&path, Some("absent"), None).is_err());
    assert!(Campaign::load(&path, None, Some(Vec::new())).is_err());
    let selected = Campaign::load(&path, Some("second"), None)?;
    assert_eq!(
        selected
            .runs()
            .map(|run| run.name.as_str())
            .collect::<Vec<_>>(),
        ["second"]
    );
    Ok(())
}

#[tokio::test]
async fn pure_campaign_report_retains_science_and_publishes_complete_files_without_overwrite()
-> Result<()> {
    let (directory, path) = fixture()?;
    let campaign = Campaign::load(&path, Some("first"), None)?;
    let run = campaign.runs().next().context("first run")?;
    let report = campaign
        .run_pure(run, directory.path(), &CancelSource::new())
        .await?;
    assert!(report.passed_completely(), "{:?}", report.report);
    assert_eq!(report.report.fixture_statuses.len(), 1);
    let target = directory.path().join("first.arrow");
    let files = report.export(&target)?;
    assert_eq!(files.len(), 6);
    let before = std::fs::read(&target)?;
    let reader =
        datafusion::arrow::ipc::reader::FileReader::try_new(std::fs::File::open(&target)?, None)?;
    let rows = reader
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .map(|batch| batch.num_rows())
        .sum::<usize>();
    assert_eq!(rows, report.report.checks.len());
    assert!(rows > 0);
    assert!(report.export(&target).is_err());
    assert_eq!(std::fs::read(&target)?, before);
    // Run-name identity is not sufficient to substitute a different campaign owner.
    let other = Campaign::load(&path, Some("first"), None)?;
    assert!(
        other
            .run_pure(run, directory.path(), &CancelSource::new())
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn pure_campaign_unknown_fixture_refuses_without_fabricated_coverage() -> Result<()> {
    let (directory, path) = fixture()?;
    let id = DeclarationId::from_bytes([199; 16]);
    let campaign = Campaign::load(&path, Some("first"), Some(vec![id]))?;
    let run = campaign.runs().next().context("first run")?;
    assert!(
        campaign
            .run_pure(run, directory.path(), &CancelSource::new())
            .await
            .is_err()
    );
    assert!(!directory.path().join("first.arrow").exists());
    Ok(())
}

#[tokio::test]
async fn native_campaign_caller_settles_success_and_refusal_before_report_export() -> Result<()> {
    use futures_util::FutureExt;
    use std::{panic::AssertUnwindSafe, sync::Arc};
    let (directory, path) = fixture()?;
    let manifest =
        std::fs::read_to_string(&path)?.replace("execution=\"pure\"", "execution=\"native\"");
    std::fs::write(&path, manifest)?;
    let unknown = DeclarationId::from_bytes([199; 16]);
    for fixtures in [None, Some(vec![unknown])] {
        let expected_pass = fixtures.is_none();
        let campaign = Campaign::load(&path, Some("first"), fixtures)?;
        let run = campaign.runs().next().context("selected native run")?;
        let shared =
            pse_runtime::SharedRuntime::build(budget(campaign.settings(), directory.path())?)?;
        let factory =
            Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
        let runtime = Runtime::from_shared_local(
            shared.clone(),
            factory.bind_model(pse_schema::shared_registry()?)?,
        )?;
        let outcome = AssertUnwindSafe(campaign.run_native(run, &runtime, &CancelSource::new()))
            .catch_unwind()
            .await;
        runtime.close();
        runtime.clear_program_cache();
        let cleanup = shared.math().shutdown_native_lanes().await;
        let result = match outcome {
            Ok(result) => result,
            Err(panic) => {
                if let Err(error) = cleanup {
                    // Both original panic and independent cleanup failure remain owned.
                    std::panic::resume_unwind(Box::new((panic, error)));
                }
                std::panic::resume_unwind(panic);
            }
        };
        match (result, cleanup) {
            (Ok(report), Ok(())) => {
                assert!(
                    expected_pass,
                    "unknown fixture must refuse before a report exists"
                );
                assert!(report.passed_completely(), "{:?}", report.report);
                assert_eq!(report.report.fixture_statuses.len(), 1);
                // The producer and its native lanes have closed. The report still owns
                // its exact tables and can publish them without reviving that producer.
                let files = report.export(&directory.path().join("native.arrow"))?;
                assert_eq!(files.len(), 6);
            }
            (Err(error), Ok(())) => {
                assert!(
                    !expected_pass,
                    "native composition unexpectedly refused: {error:#}"
                );
                assert!(
                    error.to_string().contains("selected declaration absent"),
                    "unrelated refusal: {error:#}"
                );
                assert!(
                    error.to_string().contains(&unknown.to_string()),
                    "original refused declaration missing: {error:#}"
                );
            }
            (Err(error), Err(cleanup)) => {
                return Err(
                    error.context(format!("native conformance cleanup also failed: {cleanup}"))
                );
            }
            (Ok(_), Err(cleanup)) => return Err(cleanup.into()),
        }
    }
    Ok(())
}

#[test]
fn campaign_budget_preserves_default_native_population_and_explicit_override() -> Result<()> {
    let (directory, path) = fixture()?;
    let campaign = Campaign::load(&path, Some("first"), None)?;
    let native = pse_runtime::math::MathPolicy::for_pool(campaign.settings().memory_limit_bytes);
    assert_eq!(
        budget(campaign.settings(), directory.path())?.math.jobs,
        native.jobs
    );
    let mut settings = campaign.settings().clone();
    settings.math_jobs = Some(3);
    assert_eq!(budget(&settings, directory.path())?.math.jobs, 3);
    Ok(())
}
