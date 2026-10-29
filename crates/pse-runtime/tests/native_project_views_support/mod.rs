// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual project operator wire consumers over the caller's private canonical service.
//! Included only by the product-operators branch of the native repeated-analysis target.

use datafusion::arrow::{
    array::{Array, FixedSizeBinaryArray, Float64Array, Int64Array},
    ipc::reader::{FileReader, StreamReader},
};
use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
use pse_relations::{generated::runtime::solve_variables, validate::ValidationContext};
use pse_runtime::{
    host_operations::project::{ProjectAction, ProjectOperationRequest},
    workflow::{StudyCohortExportRecord, StudyCohortMember, StudyCohortReceipt,
        StudyCohortRequest, StudyKpiRequest, StudyObservationContinuity, StudyObservationPage,
        StudyObservationRequest, StudyWorkbenchPage, StudyWorkbenchRequest},
};
use serde_json::Value;
use std::{fs::File, panic::AssertUnwindSafe, path::{Path, PathBuf},
    process::{Command, Stdio}, sync::Arc, time::{Duration, Instant}};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct Operator<'a> {
    state: &'a Path,
    database: &'a str,
    scratch: &'a Path,
}
impl Operator<'_> {
    fn invoke(&self, name: &str, action: ProjectAction, success: bool) -> Result<Option<Value>> {
        let input = self.scratch.join(format!("{name}.request.json"));
        let stdout = self.scratch.join(format!("{name}.stdout.json"));
        let stderr = self.scratch.join(format!("{name}.stderr.log"));
        std::fs::write(&input, serde_json::to_vec_pretty(&ProjectOperationRequest {
            version: Default::default(), physical: None, action,
        })?)?;
        let spill = self.scratch.join(format!("{name}.spill"));
        std::fs::create_dir(&spill)?;
        let mut command = Command::new(env!("CARGO_BIN_EXE_pse-project"));
        command.arg("project").arg("--state").arg(self.state)
            .arg("--database").arg(self.database).arg("--timeout-ms").arg("90000")
            .arg("runtime").arg("--request").arg(&input)
            .arg("--spill-dir").arg(&spill)
            .stdin(Stdio::null()).stdout(Stdio::from(File::create(&stdout)?))
            .stderr(Stdio::from(File::create(&stderr)?));
        let mut child = SupervisedChild::spawn(&mut command,
            DrainFailurePolicy::Refuse(Duration::from_secs(30)), None)?;
        if let Err(error) = child.wait_ready(None, Some(Instant::now()+Duration::from_secs(90))) {
            // Attempt the owned drain even on wait failure and retain both causes.
            let drained = child.cancel();
            return Err(format!("{name} wait failed: {error}; drain={drained:?}; stderr={}",
                std::fs::read_to_string(&stderr)?).into());
        }
        let status = child.wait()?;
        let diagnostics = std::fs::read_to_string(&stderr)?;
        if !success {
            assert!(!status.success(), "{name} unexpectedly succeeded");
            assert!(diagnostics.contains("review export output allowance exhausted"),
                "{name} must retain the actual bounded-export refusal: {diagnostics}");
            return Ok(None);
        }
        assert!(status.success(), "{name} failed ({status}): {diagnostics}");
        let envelope: Value = serde_json::from_slice(&std::fs::read(&stdout)?)?;
        assert_eq!(envelope["version"], 1, "{name} operator wire version");
        assert!(envelope.get("result").is_some(), "{name} missing original result");
        Ok(Some(envelope["result"].clone()))
    }
}

fn check_detail(
    path: &Path,
    member: &StudyCohortMember,
    original_manifest_digest: &str,
    expected: &[solve_variables::Row],
    validation: &ValidationContext,
) -> Result<()> {
    assert!(member.manifest.is_some(), "capture retains the original manifest key");
    assert!(!original_manifest_digest.is_empty(), "original closed manifest digest required");
    let stream = StreamReader::try_new(File::open(path)?, None)?;
    let mut position = 0;
    for batch in stream {
        let batch = batch?;
        let schema = batch.schema();
        for (key, original) in [
            ("pse.canonical.run", member.run.as_str()),
            ("pse.canonical.attempt", member.attempt.as_deref().expect("captured original attempt")),
            ("pse.canonical.manifest", original_manifest_digest),
            ("pse.canonical.relation", "runtime.solve_variables"),
        ] {
            assert_eq!(schema.metadata().get(key).map(String::as_str), Some(original));
        }
        let end = position+batch.num_rows();
        let mut builder = solve_variables::Builder::new(validation)?;
        for row in expected.get(position..end).expect("CLI export cannot add original rows") {
            builder.push(row.clone())?;
        }
        let original = builder.finish()?;
        // Portable IPC includes canonical attribution beyond the registry schema.
        // Compare the original declared fields and arrays without readmitting it.
        assert_eq!(schema.fields(), original.batch().schema().fields());
        assert_eq!(batch.columns(), original.batch().columns(),
            "CLI must preserve exact original values, tolerances and qualification");
        position = end;
    }
    assert_eq!(position, expected.len());
    Ok(())
}

fn exercise(
    operator: &Operator<'_>,
    study: &str,
    cohort: StudyCohortRequest,
    kpi: StudyKpiRequest,
    original_manifest_digest: &str,
    expected: &[solve_variables::Row],
    validation: &ValidationContext,
) -> Result<()> {
    let page: StudyWorkbenchPage = serde_json::from_value(operator.invoke("workbench",
        ProjectAction::StudyWorkbench { request: StudyWorkbenchRequest {
            filter: cohort.filter.clone(), maximum: 1, cursor: None,
        } }, true)?.unwrap())?;
    let [header] = page.headers.as_slice() else { panic!("one exact CLI study header") };
    assert_eq!(header.study, study);
    assert_eq!(header.point_count, 2);
    assert!(page.next.is_none(), "exact selected study has no continuation");

    let pages = operator.scratch.join("cohort.ndjson");
    let exported = operator.invoke("cohort", ProjectAction::StudyCohort {
        request: cohort.clone(), output: Some(pages.clone()), maximum_bytes: 1 << 20,
    }, true)?.unwrap();
    assert_eq!(exported["bytes"].as_u64(), Some(std::fs::metadata(&pages)?.len()));
    let receipt: StudyCohortReceipt = serde_json::from_value(exported["receipt"].clone())?;
    assert!(receipt.complete);
    assert_eq!((receipt.header_count, receipt.member_count), (1,2));
    let records = std::fs::read_to_string(&pages)?.lines()
        .map(serde_json::from_str::<StudyCohortExportRecord>).collect::<std::result::Result<Vec<_>,_>>()?;
    let Some(StudyCohortExportRecord::Receipt(final_receipt)) = records.last()
        else { panic!("completed NDJSON must end in its original receipt") };
    assert_eq!(serde_json::to_value(final_receipt)?, exported["receipt"]);
    let headers: usize = records.iter().map(|record| match record {
        StudyCohortExportRecord::Headers(page) => page.headers.len(), _ => 0,
    }).sum();
    assert_eq!(headers, 1);
    let members = records.iter().flat_map(|record| match record {
        StudyCohortExportRecord::Members(page) => page.members.as_slice(), _ => &[],
    }).collect::<Vec<_>>();
    assert_eq!(members.len(), 2);
    assert_eq!(members.iter().map(|member| member.position).collect::<Vec<_>>(), [0,1]);
    let first = &members[0].value;
    assert_eq!(first.study, study);
    assert!(first.settled);

    let refused = operator.scratch.join("refused.ndjson");
    operator.invoke("cohort-refused", ProjectAction::StudyCohort {
        request: cohort.clone(), output: Some(refused.clone()), maximum_bytes: 1,
    }, false)?;
    assert!(!refused.exists());
    assert!(refused.with_extension("ndjson.incomplete").is_file());

    let aggregate = operator.scratch.join("aggregate.arrow");
    let analysis = operator.invoke("analyze", ProjectAction::StudyAnalyze {
        request: cohort.clone(), kpi: kpi.clone(), output: aggregate.clone(),
    }, true)?.unwrap();
    assert_eq!(analysis["included"], 2);
    assert_eq!(analysis["unavailable"], 0);
    let coordinate = expected.iter().find(|row| row.symbol_id == kpi.owner)
        .expect("expected original rows include the selected scientific KPI");
    let mut rows = 0;
    for batch in FileReader::try_new(File::open(&aggregate)?, None)? {
        let batch = batch?;
        let count = batch.column_by_name("count").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
        let mean = batch.column_by_name("mean").unwrap().as_any().downcast_ref::<Float64Array>().unwrap();
        for row in 0..batch.num_rows() {
            assert!(!count.is_null(row) && !mean.is_null(row));
            assert_eq!(count.value(row), 2);
            assert!((mean.value(row)-2.5).abs() <= 1e-7,
                "native CLI aggregates the original roots 2 and 3");
            for (field, identity) in [("quantity_id", coordinate.quantity_id.unwrap()),
                ("unit_id", coordinate.unit_id.unwrap())] {
                let ids = batch.column_by_name(field).unwrap().as_any()
                    .downcast_ref::<FixedSizeBinaryArray>().unwrap();
                assert!(!ids.is_null(row));
                assert_eq!(ids.value(row), identity.as_bytes());
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 1);

    let detail = operator.scratch.join("detail.arrow");
    let exported = operator.invoke("result", ProjectAction::StudyResult {
        request: cohort, member: 0, relation: "runtime.solve_variables".into(),
        start: 0, end: u64::MAX, output: detail.clone(), maximum_bytes: 1 << 20,
    }, true)?.unwrap();
    assert_eq!(exported["bytes"].as_u64(), Some(std::fs::metadata(&detail)?.len()));
    assert_eq!(exported["member"], 0);
    check_detail(&detail, first, original_manifest_digest, expected, validation)?;

    let observation: StudyObservationPage = serde_json::from_value(operator.invoke("observe",
        ProjectAction::StudyObserve {
            request: StudyObservationRequest { study: study.into(), maximum_members: 2, maximum_events: 512 },
            cursor: None, delay_ms: None,
        }, true)?.unwrap())?;
    assert_eq!(observation.status.header.study, study);
    assert!(observation.status.header.terminal && !observation.status.header.cancelled);
    assert_eq!(observation.status.members.members.len(), 2);
    let parents = observation.sources.iter().filter(|source|
        source.position.run == observation.status.header.run).collect::<Vec<_>>();
    let [parent] = parents.as_slice() else { panic!("one original CLI parent observation") };
    assert!(parent.position.history_complete);
    assert_eq!(parent.continuity, StudyObservationContinuity::HistoryUnavailable);
    assert!(parent.events.is_empty());
    assert!(matches!(parent.coverage,
        Some(pse_runtime::workflow::ProgressCoverage::Unknown {})));
    Ok(())
}

/// Run real child commands without starving the SDK/service driver's async runtime.
/// On any error or assertion, retain the complete private request/output/log directory.
#[allow(clippy::print_stderr, reason = "retain and locate failed external-wire receipts before resuming the original assertion")]
pub(crate) async fn check(
    state: PathBuf,
    database: String,
    study: String,
    cohort: StudyCohortRequest,
    kpi: StudyKpiRequest,
    expected: Vec<solve_variables::Row>,
    validation: Arc<ValidationContext>,
    original_manifest_digest: String,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let scratch = tempfile::tempdir()?;
        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| exercise(&Operator {
            state: &state, database: &database, scratch: scratch.path(),
        }, &study, cohort, kpi, &original_manifest_digest, &expected, &validation)));
        match outcome {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => {
                let retained = scratch.keep();
                Err(format!("{error}; native project wire receipts retained at {}", retained.display()).into())
            }
            Err(panic) => {
                let retained = scratch.keep();
                eprintln!("native project wire receipts retained at {}", retained.display());
                std::panic::resume_unwind(panic)
            }
        }
    }).await?
}
