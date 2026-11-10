// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pinned SurrealKit suite on a fresh native service and a separate namespace.

use super::{NativeServiceError, current_fixture};
use pse_diagnostics::{SupervisedChild, SupervisionError};
use pse_operations::{
    canonical::{CanonicalOptions, MESSAGE_BYTES},
    generated::surreal::{SCHEMA, SCHEMA_DIGEST},
    transport::{
        bounded_query,
        client::{Client, RequestContext, Root},
        request,
    },
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    io::Read as _,
    net::SocketAddr,
    os::unix::{fs::OpenOptionsExt as _, process::ExitStatusExt as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const SUITE: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/canonical.toml");
const SEED_MUTATION_CASES: [&str; 2] = ["authored_seed_is_visible", "native_u64_exact"];
const MAX_REPORT_BYTES: u64 = 1024 * 1024;
const SEED: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/seed.surql");
const HTTP_FIXTURE: &str =
    include_str!("../../../../scripts/tests/fixtures/surrealkit/control-transaction.surql");
const ROLLOUT_SUITE: &str =
    include_str!("../../../../scripts/tests/fixtures/surrealkit/rollout.toml");
const ROLLOUT_SCHEMA: &str =
    include_str!("../../../../scripts/tests/fixtures/surrealkit/rollout-schema.surql");
const ROLLOUT_MANIFEST: &str =
    include_str!("../../../../scripts/tests/fixtures/surrealkit/rollout-manifest.toml");
const ROLLOUT_ID: &str = "20261107215752__canonical_v4_fixture";
const PARITY_CASE: &str = "replaying every rollout defines what sync defines";
const DRIFT_TABLE: &str = "pse_surreal_kit_parity_drift";
const NAMESPACE: &str = "surrealkit_contract";
const SETUP_USER: &str = "suite_owner";
const INSPECTOR: &str = "suite_viewer";
const SUITE_NAME: &str = "production_generated_contract";
const ROLLOUT_SUITE_NAME: &str = "authored_rollout_contract";
const DATABASE_PREFIX: &str = "contract";
const SETUP_REFUSAL: &str = "pse_surrealkit_fixture_setup_failed";
const SEED_REFUSAL: &str = "pse_surrealkit_fixture_seed_failed";

fn prepared_tool() -> Result<PathBuf, NativeServiceError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let environment = std::env::vars_os().collect();
    Ok(pse_host::surrealkit::verify(&root, &environment)?)
}

fn owned_address(endpoint: &str, live_endpoint: &str) -> Result<SocketAddr, NativeServiceError> {
    if endpoint == live_endpoint
        && let Some(SocketAddr::V4(address)) = endpoint
            .strip_prefix("grpc://")
            .and_then(|address| address.parse::<SocketAddr>().ok())
        && address.ip().is_loopback()
        && address.port() != 0
    {
        return Ok(address.into());
    }
    Err(NativeServiceError::Refused(
        "SurrealKit endpoint differs from the live test-owned gRPC endpoint",
    ))
}

async fn provision_namespace(
    address: SocketAddr,
    options: &CanonicalOptions,
    setup_password: &str,
    inspection_password: &str,
    deadline: Instant,
) -> Result<(), NativeServiceError> {
    let client = Client::connect(address, MESSAGE_BYTES)
        .await
        .map_err(|_| NativeServiceError::Refused("SurrealKit namespace control did not connect"))?;
    let outcome = async {
        request(
            client
                .signin(Root {
                    username: options.username.clone(),
                    password: options.password.clone(),
                })
                .request_context(RequestContext::control(deadline)),
        )
        .await?;
        // Both generated passwords are simple UUIDs, containing only hexadecimal digits.
        bounded_query(
            client
                .query(format!(
                    "DEFINE NAMESPACE {NAMESPACE}; USE NS {NAMESPACE}; \
             DEFINE USER {SETUP_USER} ON NAMESPACE PASSWORD '{setup_password}' ROLES OWNER; \
             DEFINE USER {INSPECTOR} ON NAMESPACE PASSWORD '{inspection_password}' ROLES VIEWER;"
                ))
                .request_context(RequestContext::control(deadline)),
        )
        .await
    }
    .await;
    // Upstream query diagnostics can reflect credential DDL. Only classify these failures.
    let drained = client.disconnect().await;
    match (outcome.is_ok(), drained.is_ok()) {
        (true, true) => Ok(()),
        (false, true) => Err(NativeServiceError::Refused(
            "SurrealKit namespace setup failed",
        )),
        (true, false) => Err(NativeServiceError::Refused(
            "SurrealKit namespace control drain failed",
        )),
        (false, false) => Err(NativeServiceError::Refused(
            "SurrealKit namespace setup and control drain failed",
        )),
    }
}

fn stage_project(
    folder: &Path,
    schema: &str,
    seed: Option<&str>,
) -> Result<PathBuf, NativeServiceError> {
    for directory in [
        "schema",
        "tests/suites",
        "tests/fixtures",
        "seed",
        "snapshots",
    ] {
        fs::create_dir_all(folder.join(directory))?;
    }
    fs::write(folder.join("schema/canonical.surql"), schema)?;
    fs::write(folder.join("tests/suites/canonical.toml"), SUITE)?;
    fs::write(
        folder.join("tests/fixtures/control-transaction.surql"),
        HTTP_FIXTURE,
    )?;
    let seed_path = folder.join("seed/seed.surql");
    if let Some(seed) = seed {
        fs::write(&seed_path, seed)?;
    } else {
        stage_fifo(&seed_path)?;
    }
    write_empty_snapshots(folder)?;
    Ok(seed_path)
}

fn write_empty_snapshots(folder: &Path) -> Result<(), NativeServiceError> {
    fs::write(
        folder.join("snapshots/schema_snapshot.json"),
        r#"{"version":1,"files":[]}"#,
    )?;
    fs::write(
        folder.join("snapshots/catalog_snapshot.json"),
        r#"{"version":2,"entities":[]}"#,
    )?;
    Ok(())
}

fn stage_fifo(path: &Path) -> Result<(), NativeServiceError> {
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        path,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .map_err(std::io::Error::from)?;
    Ok(())
}

struct SuiteTool<'a> {
    binary: &'a Path,
    endpoint: &'a str,
    setup_password: &'a str,
    inspection_password: &'a str,
    work_end: Instant,
    settlement_end: Instant,
    child_pending: Arc<AtomicBool>,
}

#[derive(Clone, Copy)]
enum SuiteSelection {
    CurrentSchema,
    SeedMutation,
    AuthoredRollout,
}

impl SuiteSelection {
    fn cli(self) -> (&'static str, &'static str) {
        match self {
            Self::CurrentSchema | Self::SeedMutation => ("sync", SUITE_NAME),
            Self::AuthoredRollout => ("both", ROLLOUT_SUITE_NAME),
        }
    }
}

impl SuiteTool<'_> {
    async fn run(
        &self,
        folder: &Path,
        selection: SuiteSelection,
        interrupt_at: Option<PathBuf>,
    ) -> Result<(ExitStatus, Vec<u8>), NativeServiceError> {
        let (schema_source, suite) = selection.cli();
        let allowance = match selection {
            SuiteSelection::CurrentSchema | SuiteSelection::SeedMutation => Duration::from_secs(30),
            SuiteSelection::AuthoredRollout => Duration::from_secs(120),
        };
        let mut command = Command::new(self.binary);
        let http_endpoint = self.endpoint.replacen("grpc://", "http://", 1);
        command
            .env_clear()
            .env("HOME", folder)
            .current_dir(folder)
            .stdin(Stdio::null())
            .args(["test", "--schema-from", schema_source, "--suite", suite])
            .args([
                "--host",
                self.endpoint,
                "--ns",
                NAMESPACE,
                "--db",
                DATABASE_PREFIX,
                "--base-url",
                &http_endpoint,
            ])
            .args(["--auth-level", "namespace", "--connect-timeout-secs", "10"])
            .arg("--folder")
            .arg(folder)
            .arg("--json-out")
            .arg(folder.join("result.json"))
            .env("SURREALDB_USER", SETUP_USER)
            .env("SURREALDB_PASSWORD", self.setup_password)
            .env("PSE_SURREALKIT_INSPECTION_USER", INSPECTOR)
            .env(
                "PSE_SURREALKIT_INSPECTION_PASSWORD",
                self.inspection_password,
            );
        if matches!(selection, SuiteSelection::AuthoredRollout) {
            command.arg("--no-seed");
        }
        if matches!(selection, SuiteSelection::SeedMutation) {
            command.args(["--tag", "seed-mutation"]);
        }
        let deadline = (Instant::now() + allowance).min(self.work_end);
        if deadline <= Instant::now() {
            return Err(NativeServiceError::Refused(
                "SurrealKit work cutoff elapsed",
            ));
        }
        let child_end =
            (deadline + Duration::from_secs(10)).min(self.settlement_end - Duration::from_secs(30));
        let child_pending = self.child_pending.clone();
        let state = folder.to_owned();
        tokio::task::spawn_blocking(move || {
            child_pending.store(true, Ordering::Release);
            let mut child =
                SupervisedChild::spawn_captured(&mut command, 64 * 1024, 64 * 1024, child_end)?;
            let observed = if let Some(input) = interrupt_at {
                wait_for_project_reader(&mut child, &input, deadline).map(Some)
            } else {
                child
                    .wait_captured(None, deadline)
                    .map_err(NativeServiceError::from)
                    .map(|()| None)
            };
            // Hold the test FIFO writer open until the child is physically drained.
            let settlement = child.cancel();
            if child.retained_status().is_some() && settlement.is_ok() {
                child_pending.store(false, Ordering::Release);
            }
            let status = match (observed, settlement) {
                (Ok(_writer), Ok(status)) => status,
                (Err(work), Ok(_)) => return Err(work),
                (Ok(_writer), Err(drain)) => return Err(drain.into()),
                (Err(work), Err(drain)) => {
                    return Err(NativeServiceError::Settlement {
                        state,
                        failures: vec![work, drain.into()],
                    });
                }
            };
            let output = child.captured_output().ok_or(NativeServiceError::Refused(
                "SurrealKit capture absent after settlement",
            ))?;
            if output.stdout.overflow || output.stderr.overflow {
                return Err(NativeServiceError::Refused(
                    "SurrealKit diagnostics exceeded capture bounds",
                ));
            }
            Ok((status, output.stderr.prefix.clone()))
        })
        .await
        .map_err(|_| NativeServiceError::Refused("SurrealKit child owner task failed"))?
    }
}

fn wait_for_project_reader(
    child: &mut SupervisedChild,
    path: &Path,
    deadline: Instant,
) -> Result<fs::File, NativeServiceError> {
    while Instant::now() < deadline {
        // A nonblocking writer opens only once the real project input reader is waiting.
        match fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(writer) => return Ok(writer),
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {}
            Err(error) => return Err(error.into()),
        }
        match child.wait_captured(
            None,
            (Instant::now() + Duration::from_millis(15)).min(deadline),
        ) {
            Ok(()) => break,
            Err(SupervisionError::Deadline) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err(NativeServiceError::Refused(
        "SurrealKit did not reach the selected interrupted input",
    ))
}

#[derive(Deserialize)]
struct CaseIdentity {
    name: String,
}

#[derive(Deserialize)]
struct SuiteIdentity {
    name: String,
    cases: Vec<CaseIdentity>,
}

#[derive(Deserialize)]
struct CaseReport {
    name: String,
    kind: String,
    passed: bool,
    assertions: Vec<AssertionReport>,
}

#[derive(Deserialize)]
struct AssertionReport {
    name: String,
    passed: bool,
    message: String,
}

#[derive(Deserialize)]
struct SuiteReport {
    suite_name: String,
    namespace: String,
    database: String,
    cases: Vec<CaseReport>,
}

#[derive(Deserialize)]
struct Report {
    suites_total: usize,
    suites_failed: usize,
    cases_total: usize,
    cases_passed: usize,
    cases_failed: usize,
    suites: Vec<SuiteReport>,
}

fn verify_report(
    path: &Path,
    status: ExitStatus,
    expected_failure: Option<&str>,
) -> Result<usize, NativeServiceError> {
    let source: SuiteIdentity = toml::from_str(SUITE)
        .map_err(|_| NativeServiceError::Refused("invalid authored SurrealKit suite"))?;
    let report = read_report(path)?;
    let Some(suite) = report.suites.first() else {
        return Err(NativeServiceError::Refused(
            "missing SurrealKit suite result",
        ));
    };
    let authored: BTreeSet<_> = source.cases.iter().map(|case| case.name.as_str()).collect();
    let names = if expected_failure.is_some() {
        BTreeSet::from(SEED_MUTATION_CASES)
    } else {
        authored.clone()
    };
    let observed: BTreeSet<_> = suite.cases.iter().map(|case| case.name.as_str()).collect();
    if report.suites_total != 1
        || report.suites.len() != 1
        || suite.suite_name != source.name
        || suite.namespace != NAMESPACE
        || !suite
            .database
            .starts_with(&format!("{DATABASE_PREFIX}_sk_test_"))
        || !names.is_subset(&authored)
        || report.cases_total != names.len()
        || suite.cases.len() != names.len()
        || observed != names
    {
        return Err(NativeServiceError::Refused(
            "partial or unexpected SurrealKit suite report",
        ));
    }
    let failures: Vec<_> = suite
        .cases
        .iter()
        .filter(|case| !case.passed)
        .map(|case| case.name.as_str())
        .collect();
    if status.code() != Some(i32::from(expected_failure.is_some()))
        || report.suites_failed != usize::from(expected_failure.is_some())
        || report.cases_failed != failures.len()
        || report.cases_passed != names.len() - failures.len()
        || failures.iter().copied().ne(expected_failure)
    {
        eprintln!("Unexpected SurrealKit assertion failures: {failures:?}");
        return Err(NativeServiceError::Command {
            stage: "SurrealKit suite assertions",
            status,
        });
    }
    Ok(report.cases_passed)
}

fn read_report(path: &Path) -> Result<Report, NativeServiceError> {
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                NativeServiceError::Refused(
                    "SurrealKit stopped before writing its original suite report",
                )
            } else {
                error.into()
            }
        })?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_REPORT_BYTES {
        return Err(NativeServiceError::Refused(
            "SurrealKit suite report is not a bounded regular file",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_REPORT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_REPORT_BYTES {
        return Err(NativeServiceError::Refused(
            "SurrealKit suite report exceeds its byte limit",
        ));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| NativeServiceError::Refused("invalid SurrealKit suite report"))
}

fn verify_pre_report_refusal(
    project: &Path,
    status: ExitStatus,
    stderr: &[u8],
    refusal: &str,
) -> Result<(), NativeServiceError> {
    if status.code() != Some(1)
        || project.join("result.json").try_exists()?
        || !String::from_utf8_lossy(stderr).contains(refusal)
    {
        return Err(NativeServiceError::Refused(
            "SurrealKit did not observe the expected authored refusal before report publication",
        ));
    }
    Ok(())
}

fn stage_rollout_project(folder: &Path, drift: bool) -> Result<(), NativeServiceError> {
    let destination = folder.join("rollouts").join(ROLLOUT_ID);
    for path in [
        folder.join("schema"),
        folder.join("tests/suites"),
        destination.join("schema"),
        destination.join("snapshots"),
    ] {
        fs::create_dir_all(path)?;
    }
    fs::write(destination.with_extension("toml"), ROLLOUT_MANIFEST)?;
    fs::write(destination.join("schema/canonical.surql"), ROLLOUT_SCHEMA)?;
    write_empty_snapshots(&destination)?;
    let mut schema = ROLLOUT_SCHEMA.to_owned();
    if drift {
        use std::fmt::Write as _;
        writeln!(schema, "\nDEFINE TABLE {DRIFT_TABLE} SCHEMAFULL;")
            .expect("writing to a String succeeds");
    }
    fs::write(folder.join("schema/canonical.surql"), schema)?;
    fs::write(folder.join("tests/suites/rollout.toml"), ROLLOUT_SUITE)?;
    fs::write(
        folder.join("tests/config.toml"),
        "[rollouts]\nparity = true\n",
    )?;
    Ok(())
}

fn verify_rollout_report(
    path: &Path,
    status: ExitStatus,
    drift: bool,
) -> Result<(), NativeServiceError> {
    let report = read_report(path)?;
    let refused = || NativeServiceError::Refused("unexpected SurrealKit rollout parity report");
    let [parity, sync, replay] = report.suites.as_slice() else {
        return Err(refused());
    };
    let [case] = parity.cases.as_slice() else {
        return Err(refused());
    };
    let failed = usize::from(drift);
    if status.code() != Some(failed as i32)
        || report.suites_total != 3
        || report.suites_failed != failed
        || report.cases_total != 3
        || report.cases_failed != failed
        || report.cases_passed != 3 - failed
        || parity.suite_name != "rollout parity"
        || case.name != PARITY_CASE
        || case.kind != "rollout_parity"
        || case.passed == drift
        || sync.database == replay.database
    {
        return Err(refused());
    }
    for (suite, source) in [(sync, "sync"), (replay, "rollouts")] {
        if suite.suite_name != format!("{ROLLOUT_SUITE_NAME} [{source}]")
            || suite.namespace != NAMESPACE
            || !suite
                .database
                .starts_with(&format!("{DATABASE_PREFIX}_sk_test_"))
            || !matches!(suite.cases.as_slice(), [case] if case.name == "definitions_do_not_activate" && case.passed)
        {
            return Err(refused());
        }
    }
    let disagreements: Vec<_> = case.assertions.iter().filter(|a| !a.passed).collect();
    if !case.assertions.iter().any(|assertion| {
        assertion.name == "matching definitions"
            && assertion.passed
            && assertion
                .message
                .split_once(' ')
                .and_then(|(count, _)| count.parse::<usize>().ok())
                .is_some_and(|count| count > 0)
    }) || if drift {
        disagreements.is_empty()
            || disagreements.iter().any(|assertion| {
                !assertion.name.contains(DRIFT_TABLE)
                    || assertion.message != "defined by sync but not by the rollouts"
            })
    } else {
        !disagreements.is_empty()
    } {
        eprintln!(
            "SurrealKit parity disagreements: {:?}",
            disagreements.iter().map(|a| &a.name).collect::<Vec<_>>()
        );
        return Err(refused());
    }
    Ok(())
}

async fn exercise_suite(tool: &SuiteTool<'_>, root: &Path) -> Result<usize, NativeServiceError> {
    let passing = root.join("passing");
    stage_project(&passing, SCHEMA, Some(SEED))?;
    let (status, _) = tool
        .run(&passing, SuiteSelection::CurrentSchema, None)
        .await?;
    let passed = verify_report(&passing.join("result.json"), status, None)?;

    let assertion_failure = root.join("assertion-failure");
    stage_project(
        &assertion_failure,
        SCHEMA,
        Some(&SEED.replace("42dec", "43dec")),
    )?;
    let (status, _) = tool
        .run(&assertion_failure, SuiteSelection::SeedMutation, None)
        .await?;
    verify_report(
        &assertion_failure.join("result.json"),
        status,
        Some("authored_seed_is_visible"),
    )?;

    let setup_failure = root.join("setup-failure");
    stage_project(&setup_failure, ROLLOUT_SCHEMA, Some(SEED))?;
    fs::write(
        setup_failure.join("setup.surql"),
        format!("THROW '{SETUP_REFUSAL}';"),
    )?;
    let (status, stderr) = tool
        .run(&setup_failure, SuiteSelection::CurrentSchema, None)
        .await?;
    verify_pre_report_refusal(&setup_failure, status, &stderr, SETUP_REFUSAL)?;

    let seed_failure = root.join("seed-failure");
    stage_project(
        &seed_failure,
        ROLLOUT_SCHEMA,
        Some(&format!("THROW '{SEED_REFUSAL}';")),
    )?;
    let (status, stderr) = tool
        .run(&seed_failure, SuiteSelection::CurrentSchema, None)
        .await?;
    verify_pre_report_refusal(&seed_failure, status, &stderr, SEED_REFUSAL)?;

    let setup_interrupted = root.join("setup-interrupted");
    stage_project(&setup_interrupted, ROLLOUT_SCHEMA, Some(SEED))?;
    let setup = setup_interrupted.join("setup.surql");
    stage_fifo(&setup)?;
    let seed_interrupted = root.join("seed-interrupted");
    let seed = stage_project(&seed_interrupted, ROLLOUT_SCHEMA, None)?;
    for (project, input) in [(setup_interrupted, setup), (seed_interrupted, seed)] {
        let (status, _) = tool
            .run(&project, SuiteSelection::CurrentSchema, Some(input))
            .await?;
        if status.signal().is_none() || project.join("result.json").try_exists()? {
            return Err(NativeServiceError::Refused(
                "SurrealKit interruption control had unexpected finality",
            ));
        }
    }
    for (name, drift) in [("rollout", false), ("rollout-drift", true)] {
        let project = root.join(name);
        stage_rollout_project(&project, drift)?;
        let (status, _) = tool
            .run(&project, SuiteSelection::AuthoredRollout, None)
            .await?;
        verify_rollout_report(&project.join("result.json"), status, drift)?;
    }
    Ok(passed)
}

fn verify_rejected_reports() {
    let root = tempfile::tempdir().unwrap();
    let report = root.path().join("result.json");
    let failure = ExitStatus::from_raw(1 << 8);
    let verify = || verify_report(&report, failure, Some("authored_seed_is_visible"));
    assert!(verify().unwrap_err().to_string().contains("before writing"));
    fs::write(&report, b"{").unwrap();
    assert!(verify().unwrap_err().to_string().contains("invalid"));
    fs::OpenOptions::new()
        .write(true)
        .open(&report)
        .unwrap()
        .set_len(MAX_REPORT_BYTES + 1)
        .unwrap();
    assert!(
        verify()
            .unwrap_err()
            .to_string()
            .contains("bounded regular file")
    );
    let case = |name, passed| {
        serde_json::json!({
            "name": name, "kind": "sql_expect", "passed": passed, "assertions": []
        })
    };
    let mut document = serde_json::json!({
        "suites_total": 1, "suites_failed": 1, "cases_total": 2,
        "cases_passed": 1, "cases_failed": 1,
        "suites": [{
            "suite_name": SUITE_NAME, "namespace": NAMESPACE,
            "database": format!("{DATABASE_PREFIX}_sk_test_selected"),
            "cases": [case(SEED_MUTATION_CASES[0], false), case(SEED_MUTATION_CASES[1], true)]
        }]
    });
    let write = |document: &serde_json::Value| {
        fs::write(&report, serde_json::to_vec(document).unwrap()).unwrap();
    };
    write(&document);
    assert_eq!(verify().unwrap(), 1);
    document["suites"][0]["cases"][1]["name"] = serde_json::json!("immutable_bytes_exact");
    write(&document);
    assert!(
        verify()
            .unwrap_err()
            .to_string()
            .contains("partial or unexpected")
    );
    document["suites"][0]["cases"] = serde_json::json!([case(SEED_MUTATION_CASES[0], false)]);
    write(&document);
    assert!(
        verify()
            .unwrap_err()
            .to_string()
            .contains("partial or unexpected")
    );
}

#[tokio::test]
#[ignore = "requires the explicitly prepared pinned SurrealKit release"]
async fn surrealkit_generated_contract_uses_owned_service_and_disposes() {
    verify_rejected_reports();
    let binary = prepared_tool().unwrap();
    let mut service = current_fixture().unwrap();
    let state = service.state().to_path_buf();
    let work_end = Instant::now() + Duration::from_secs(600);
    let settlement_end = work_end + Duration::from_secs(60);
    let child_pending = Arc::new(AtomicBool::new(false));
    let pending = child_pending.clone();
    let outcome = service
        .run_until(work_end, settlement_end, |service| {
            Box::pin(async move {
                service
                    .launch((Instant::now() + Duration::from_secs(90)).min(work_end))
                    .await?;
                let options = CanonicalOptions::from_state(service.state())?;
                let live_endpoint = service.ready()?.store().endpoint();
                if owned_address("grpc://127.0.0.1:1", live_endpoint).is_ok() {
                    return Err(NativeServiceError::Refused(
                        "SurrealKit accepted a foreign endpoint",
                    ));
                }
                let address = owned_address(&options.endpoint, live_endpoint)?;
                let setup_password = uuid::Uuid::new_v4().simple().to_string();
                let inspection_password = uuid::Uuid::new_v4().simple().to_string();
                provision_namespace(
                    address,
                    &options,
                    &setup_password,
                    &inspection_password,
                    work_end,
                )
                .await?;
                let tool = SuiteTool {
                    binary: &binary,
                    endpoint: &options.endpoint,
                    setup_password: &setup_password,
                    inspection_password: &inspection_password,
                    work_end,
                    settlement_end,
                    child_pending: pending,
                };
                exercise_suite(&tool, &service.state().join("surrealkit")).await
            })
        })
        .await;
    assert!(
        !child_pending.load(Ordering::Acquire),
        "unsettled SurrealKit child retains {state:?}: {outcome:?}"
    );
    let disposal = service.dispose();
    let (Ok(cases), Ok(())) = (&outcome, &disposal) else {
        panic!(
            "SurrealKit suite or its original endpoint failed: suite={outcome:?}, disposal={disposal:?}"
        );
    };
    assert!(!state.try_exists().unwrap());
    eprintln!(
        "SurrealKit generated schema {SCHEMA_DIGEST}: {cases} cases passed; authored rollout parity and drift control exercised; owned endpoint disposed"
    );
}
