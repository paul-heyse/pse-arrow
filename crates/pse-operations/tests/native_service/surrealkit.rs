// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pinned SurrealKit suite on a fresh native service and a separate namespace.

use super::{NativeServiceError, fixture_with_schema};
use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
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
    net::SocketAddr,
    os::unix::{fs::OpenOptionsExt as _, process::ExitStatusExt as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

const SUITE: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/canonical.toml");
const SEED: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/seed.surql");
const ROLLOUT_SUITE: &str =
    include_str!("../../../../scripts/tests/fixtures/surrealkit/rollout.toml");
const ROLLOUT_ID: &str = "20261021131104__canonical_28f894ce3c844206";
const PARITY_CASE: &str = "replaying every rollout defines what sync defines";
const DRIFT_TABLE: &str = "pse_surreal_kit_parity_drift";
const NAMESPACE: &str = "surrealkit_contract";
const SETUP_USER: &str = "suite_owner";
const INSPECTOR: &str = "suite_viewer";
const SUITE_NAME: &str = "production_generated_contract";
const ROLLOUT_SUITE_NAME: &str = "authored_rollout_contract";
const DATABASE_PREFIX: &str = "contract";

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

fn stage_project(folder: &Path, seed: Option<&str>) -> Result<PathBuf, NativeServiceError> {
    for directory in ["schema", "tests/suites", "seed", "snapshots"] {
        fs::create_dir_all(folder.join(directory))?;
    }
    fs::write(folder.join("schema/canonical.surql"), SCHEMA)?;
    fs::write(folder.join("tests/suites/canonical.toml"), SUITE)?;
    let seed_path = folder.join("seed/seed.surql");
    if let Some(seed) = seed {
        fs::write(&seed_path, seed)?;
    } else {
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            &seed_path,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .map_err(std::io::Error::from)?;
    }
    // SurrealKit begins with empty local snapshots for this fresh project.
    fs::write(
        folder.join("snapshots/schema_snapshot.json"),
        r#"{"version":1,"files":[]}"#,
    )?;
    fs::write(
        folder.join("snapshots/catalog_snapshot.json"),
        r#"{"version":2,"entities":[]}"#,
    )?;
    Ok(seed_path)
}

struct SuiteTool<'a> {
    binary: &'a Path,
    endpoint: &'a str,
    setup_password: &'a str,
    inspection_password: &'a str,
}

#[derive(Clone, Copy)]
enum SuiteSelection {
    CurrentSchema,
    AuthoredRollout,
}

impl SuiteSelection {
    fn cli(self) -> (&'static str, &'static str) {
        match self {
            Self::CurrentSchema => ("sync", SUITE_NAME),
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
    ) -> Result<ExitStatus, NativeServiceError> {
        let (schema_source, suite) = selection.cli();
        let allowance = match selection {
            SuiteSelection::CurrentSchema => Duration::from_secs(30),
            SuiteSelection::AuthoredRollout => Duration::from_secs(120),
        };
        let mut command = Command::new(self.binary);
        command
            .env_clear()
            .env("HOME", folder)
            .current_dir(folder)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(["test", "--schema-from", schema_source, "--suite", suite])
            .args([
                "--host",
                self.endpoint,
                "--ns",
                NAMESPACE,
                "--db",
                DATABASE_PREFIX,
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
        tokio::task::spawn_blocking(move || {
            let mut child = SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, None)
                .map_err(|_| NativeServiceError::Refused("SurrealKit child admission failed"))?;
            let deadline = Instant::now() + allowance;
            let observed = if let Some(seed) = interrupt_at {
                wait_for_seed_reader(&mut child, &seed, deadline).map(Some)
            } else {
                child
                    .wait_ready(None, Some(deadline))
                    .map_err(|error| {
                        eprintln!(
                            "SurrealKit {suite} ({schema_source}) child observation: {error}"
                        );
                        NativeServiceError::Refused("SurrealKit child observation failed")
                    })
                    .map(|()| None)
            };
            let status = child
                .cancel()
                .map_err(|_| NativeServiceError::Refused("SurrealKit child settlement failed"))?;
            // Hold the test FIFO writer open until the child is physically drained.
            drop(observed?);
            Ok(status)
        })
        .await
        .map_err(|_| NativeServiceError::Refused("SurrealKit child owner task failed"))?
    }
}

fn wait_for_seed_reader(
    child: &mut SupervisedChild,
    path: &Path,
    deadline: Instant,
) -> Result<fs::File, NativeServiceError> {
    while Instant::now() < deadline {
        // A nonblocking writer opens only when the real SurrealKit seed reader is waiting.
        match fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(writer) => return Ok(writer),
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {}
            Err(error) => return Err(error.into()),
        }
        if child
            .try_wait()
            .map_err(|_| NativeServiceError::Refused("SurrealKit child observation failed"))?
            .is_some()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    Err(NativeServiceError::Refused(
        "SurrealKit did not reach the interrupted seed",
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
    let names: BTreeSet<_> = source.cases.iter().map(|case| case.name.as_str()).collect();
    let observed: BTreeSet<_> = suite.cases.iter().map(|case| case.name.as_str()).collect();
    if report.suites_total != 1
        || report.suites.len() != 1
        || suite.suite_name != source.name
        || suite.namespace != NAMESPACE
        || !suite
            .database
            .starts_with(&format!("{DATABASE_PREFIX}_sk_test_"))
        || report.cases_total != source.cases.len()
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
    if status.success() != expected_failure.is_none()
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
    serde_json::from_slice(&fs::read(path).map_err(|_| {
        NativeServiceError::Refused("SurrealKit stopped before writing its original suite report")
    })?)
    .map_err(|_| NativeServiceError::Refused("invalid SurrealKit suite report"))
}

fn stage_rollout_project(folder: &Path, drift: bool) -> Result<(), NativeServiceError> {
    let release =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../database/canonical-schema/rollouts");
    let frozen = release.join(ROLLOUT_ID);
    let destination = folder.join("rollouts").join(ROLLOUT_ID);
    for path in [
        folder.join("schema"),
        folder.join("tests/suites"),
        destination.join("schema"),
        destination.join("snapshots"),
    ] {
        fs::create_dir_all(path)?;
    }
    let manifest = format!("{ROLLOUT_ID}.toml");
    fs::copy(
        release.join(&manifest),
        folder.join("rollouts").join(manifest),
    )?;
    for member in [
        "schema/canonical.surql",
        "snapshots/schema_snapshot.json",
        "snapshots/catalog_snapshot.json",
    ] {
        fs::copy(frozen.join(member), destination.join(member))?;
    }
    let mut schema = fs::read_to_string(frozen.join("target-schema.surql"))?;
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
    stage_project(&passing, Some(SEED))?;
    let status = tool
        .run(&passing, SuiteSelection::CurrentSchema, None)
        .await?;
    let passed = verify_report(&passing.join("result.json"), status, None)?;

    let assertion_failure = root.join("assertion-failure");
    stage_project(&assertion_failure, Some(&SEED.replace("42dec", "43dec")))?;
    let status = tool
        .run(&assertion_failure, SuiteSelection::CurrentSchema, None)
        .await?;
    verify_report(
        &assertion_failure.join("result.json"),
        status,
        Some("authored_seed_is_visible"),
    )?;

    let setup_failure = root.join("setup-failure");
    stage_project(
        &setup_failure,
        Some("THROW 'fixture seed deliberately failed';"),
    )?;
    let status = tool
        .run(&setup_failure, SuiteSelection::CurrentSchema, None)
        .await?;
    if status.code() != Some(1) || setup_failure.join("result.json").try_exists()? {
        return Err(NativeServiceError::Refused(
            "SurrealKit setup control had unexpected finality",
        ));
    }

    let interrupted = root.join("interrupted");
    let seed = stage_project(&interrupted, None)?;
    let status = tool
        .run(&interrupted, SuiteSelection::CurrentSchema, Some(seed))
        .await?;
    if status.signal().is_none() || interrupted.join("result.json").try_exists()? {
        return Err(NativeServiceError::Refused(
            "SurrealKit interruption control had unexpected finality",
        ));
    }
    for (name, drift) in [("rollout", false), ("rollout-drift", true)] {
        let project = root.join(name);
        stage_rollout_project(&project, drift)?;
        let status = tool
            .run(&project, SuiteSelection::AuthoredRollout, None)
            .await?;
        verify_rollout_report(&project.join("result.json"), status, drift)?;
    }
    Ok(passed)
}

#[tokio::test]
#[ignore = "requires the explicitly prepared pinned SurrealKit release"]
async fn surrealkit_generated_contract_uses_owned_service_and_disposes() {
    let binary = prepared_tool().unwrap();
    let mut service = fixture_with_schema(false).unwrap();
    let state = service.state().to_path_buf();
    let deadline = Instant::now() + Duration::from_secs(300);
    let outcome = service
        .run(deadline, |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
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
                    deadline,
                )
                .await?;
                let tool = SuiteTool {
                    binary: &binary,
                    endpoint: &options.endpoint,
                    setup_password: &setup_password,
                    inspection_password: &inspection_password,
                };
                exercise_suite(&tool, &service.state().join("surrealkit")).await
            })
        })
        .await;
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
