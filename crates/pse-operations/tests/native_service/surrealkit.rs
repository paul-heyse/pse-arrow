// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pinned SurrealKit suite on a fresh native service and a separate namespace.

use super::{NativeServiceError, fixture_with_schema};
use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
use pse_host::paths::{HostPaths, UserDirectory};
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
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    net::SocketAddr,
    os::unix::{fs::OpenOptionsExt as _, process::ExitStatusExt as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

const PIN: &str = include_str!("../../../../.config/surrealkit.toml");
const SUITE: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/canonical.toml");
const SEED: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/seed.surql");
const NAMESPACE: &str = "surrealkit_contract";
const SETUP_USER: &str = "suite_owner";
const INSPECTOR: &str = "suite_viewer";
const SUITE_NAME: &str = "production_generated_contract";
const DATABASE_PREFIX: &str = "contract";

#[derive(Deserialize, PartialEq)]
struct ToolPin {
    version: String,
    repository: String,
    source_revision: String,
}

#[derive(Deserialize)]
struct ToolReceipt {
    #[serde(flatten)]
    pin: ToolPin,
    source_commit: String,
    source_build: bool,
    release_build_locked: bool,
    binary_sha256: String,
}

fn prepared_tool() -> Result<PathBuf, NativeServiceError> {
    let pin: ToolPin =
        toml::from_str(PIN).map_err(|_| NativeServiceError::Refused("invalid SurrealKit pin"))?;
    let environment = std::env::vars_os().collect();
    let paths = HostPaths::new(&environment);
    let binary = match paths.selected("PSE_SURREALKIT_BIN") {
        Some(binary) => PathBuf::from(binary),
        None => paths
            .base(UserDirectory::Data)
            .map_err(|_| NativeServiceError::Refused("no prepared SurrealKit data root"))?
            .join("pse-arrow/tools/surrealkit")
            .join(format!("v{}", pin.version))
            .join(format!("grpc-{}", std::env::consts::ARCH))
            .join("surrealkit"),
    };
    let invalid = || {
        NativeServiceError::Refused(
            "SurrealKit requires its prepared pinned binary and release receipt",
        )
    };
    let receipt = binary.with_file_name("release.json");
    if !binary.is_absolute()
        || !binary.is_file()
        || binary.canonicalize().map_err(|_| invalid())? != binary
        || !receipt.symlink_metadata().map_err(|_| invalid())?.is_file()
    {
        return Err(invalid());
    }
    let receipt: ToolReceipt =
        serde_json::from_slice(&fs::read(receipt)?).map_err(|_| invalid())?;
    if receipt.pin != pin
        || receipt.source_commit != pin.source_revision
        || !receipt.source_build
        || !receipt.release_build_locked
        || receipt.binary_sha256 != hex::encode(Sha256::digest(fs::read(&binary)?))
    {
        return Err(invalid());
    }
    Ok(binary)
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

impl SuiteTool<'_> {
    async fn run(
        &self,
        folder: &Path,
        interrupt_at: Option<PathBuf>,
    ) -> Result<ExitStatus, NativeServiceError> {
        let mut command = Command::new(self.binary);
        command
            .env_clear()
            .env("HOME", folder)
            .current_dir(folder)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(["test", "--schema-from", "sync", "--suite", SUITE_NAME])
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
        tokio::task::spawn_blocking(move || {
            let mut child = SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, None)
                .map_err(|_| NativeServiceError::Refused("SurrealKit child admission failed"))?;
            let deadline = Instant::now() + Duration::from_secs(30);
            let observed = if let Some(seed) = interrupt_at {
                wait_for_seed_reader(&mut child, &seed, deadline).map(Some)
            } else {
                child
                    .wait_ready(None, Some(deadline))
                    .map_err(|_| NativeServiceError::Refused("SurrealKit child observation failed"))
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
    passed: bool,
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
    let report: Report = serde_json::from_slice(&fs::read(path).map_err(|_| {
        NativeServiceError::Refused("SurrealKit stopped before writing its original suite report")
    })?)
    .map_err(|_| NativeServiceError::Refused("invalid SurrealKit suite report"))?;
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

async fn exercise_suite(tool: &SuiteTool<'_>, root: &Path) -> Result<usize, NativeServiceError> {
    let passing = root.join("passing");
    stage_project(&passing, Some(SEED))?;
    let status = tool.run(&passing, None).await?;
    let passed = verify_report(&passing.join("result.json"), status, None)?;

    let assertion_failure = root.join("assertion-failure");
    stage_project(&assertion_failure, Some(&SEED.replace("42dec", "43dec")))?;
    let status = tool.run(&assertion_failure, None).await?;
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
    let status = tool.run(&setup_failure, None).await?;
    if status.code() != Some(1) || setup_failure.join("result.json").try_exists()? {
        return Err(NativeServiceError::Refused(
            "SurrealKit setup control had unexpected finality",
        ));
    }

    let interrupted = root.join("interrupted");
    let seed = stage_project(&interrupted, None)?;
    let status = tool.run(&interrupted, Some(seed)).await?;
    if status.signal().is_none() || interrupted.join("result.json").try_exists()? {
        return Err(NativeServiceError::Refused(
            "SurrealKit interruption control had unexpected finality",
        ));
    }
    Ok(passed)
}

#[tokio::test]
#[ignore = "requires the explicitly prepared pinned SurrealKit release"]
async fn surrealkit_generated_contract_uses_owned_service_and_disposes() {
    let binary = prepared_tool().unwrap();
    let mut service = fixture_with_schema(false).unwrap();
    let state = service.state().to_path_buf();
    let deadline = Instant::now() + Duration::from_secs(240);
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
        "SurrealKit generated schema {SCHEMA_DIGEST}: {cases} cases passed; owned endpoint disposed"
    );
}
