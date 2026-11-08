// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Receiving controls for campaign selection and owned report publication.
use super::*;

#[path = "conformance_export_tests.rs"]
mod export;

#[test]
fn native_provider_selection_is_explicit_and_unprepared_builds_refuse() {
    assert_eq!(NativeProviders::default(), NativeProviders::None);
    NativeProviders::None.validate().unwrap();
    assert_eq!(
        NativeProviders::CoolpropWater.validate().is_ok(),
        cfg!(feature = "property-coolprop")
    );
}

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
    let manifest = ConformanceManifest::load(&path)?;
    let physical = &manifest.runs[0].physical;
    let manifest = std::fs::read_to_string(&path)?.replace(
        "execution=\"pure\"",
        &format!("dependencies=[{physical:?}]\nexecution=\"native\""),
    );
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
        let outcome = AssertUnwindSafe(campaign.run_native(
            run,
            &runtime,
            NativeProviders::None,
            &CancelSource::new(),
        ))
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
fn native_campaign_shares_exact_physical_custody_only_for_the_resolved_root() -> Result<()> {
    use std::{cell::Cell, sync::Arc};

    let (directory, path) = fixture()?;
    let mut run = ConformanceManifest::load(&path)?.runs.remove(0);
    let original_root = directory.path().join(&run.physical);
    let registry = pse_schema::shared_registry()?;
    let original = capture_package(&original_root, &registry, Default::default())?;
    for root in ["physical", "same-label"] {
        for (relative, bytes) in &original.files {
            let path = directory.path().join(root).join(relative);
            std::fs::create_dir_all(path.parent().context("fixture document parent")?)?;
            std::fs::write(path, bytes)?;
        }
    }
    run.physical = "physical".into();
    run.dependencies = vec!["same-label".into(), "./physical".into()];
    let selection = NativeSourceSelection::resolve(directory.path(), &run)?;
    assert_eq!(selection.modeling[2], selection.physical);
    let physical_document = "materials/time.yaml";
    let path = selection.physical.join(physical_document);
    let alternate = [
        original.files[physical_document].as_slice(),
        b"\n# changed bytes\n",
    ]
    .concat();
    std::fs::write(selection.modeling[1].join(physical_document), &alternate)?;
    let captured = selection.capture_physical(&registry)?;
    assert_eq!(captured.header.content_hash, original.header.content_hash);
    std::fs::remove_file(&path)?;
    assert!(captured.verify(&selection.physical, &registry).is_err());
    std::fs::write(&path, &alternate)?;
    assert!(captured.verify(&selection.physical, &registry).is_err());

    let validation = pse_relations::validate::ValidationContext::new(
        &registry,
        pse_engine::validation::NativeValidation(
            datafusion::prelude::SessionContext::new().state(),
        ),
    );
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20));
    let token = pse_columnar::CancellationToken::new();
    let admissions = Cell::new(0);
    let load = |files: &BTreeMap<String, Vec<u8>>| -> Result<_> {
        admissions.set(admissions.get() + 1);
        Ok(load_package_documents_owned(
            files,
            &registry,
            Default::default(),
            &pool,
            &token,
            &validation,
        )?)
    };
    let physical = load(&captured.files)?;
    let modeling = selection.modeling_bundles(&physical, &registry, load)?;
    assert_eq!(
        admissions.get(),
        3,
        "physical, modeling root and distinct dependency"
    );
    assert!(std::ptr::eq(physical.bundle(), modeling[2].bundle()));
    assert_eq!(
        physical.bundle().package.package_id,
        modeling[1].bundle().package.package_id
    );
    assert_eq!(
        physical.bundle().package.version,
        modeling[1].bundle().package.version
    );
    assert_ne!(
        physical.bundle().package.content_hash,
        modeling[1].bundle().package.content_hash
    );

    let physical = OwnedDocumentSet::try_from_bundles(vec![physical], &pool, &token)?;
    let modeling = OwnedDocumentSet::try_from_bundles(modeling, &pool, &token)?;
    let original = &physical.bundles()[0];
    let shared = &modeling.bundles()[2];
    assert!(std::ptr::eq(
        original.documents.as_ptr(),
        shared.documents.as_ptr()
    ));
    assert_eq!(original.package.content_hash, captured.header.content_hash);
    let document = shared
        .documents
        .iter()
        .find(|doc| doc.path == physical_document)
        .context("selected physical document")?;
    assert_eq!(document.bytes(), captured.files[physical_document]);
    assert_eq!(
        document.id,
        pse_ids::named_id(captured.header.package_id.as_id(), physical_document)
    );
    assert_ne!(
        selection.capture_physical(&registry)?.header.content_hash,
        captured.header.content_hash
    );
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
