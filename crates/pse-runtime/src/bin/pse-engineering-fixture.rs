// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit retained disposable canonical owner for independent engineering-client qualification.
#![allow(
    clippy::print_stderr,
    reason = "qualification owner reports terminal failure and retained state"
)]

use clap::Parser;
use futures_util::FutureExt as _;
use pse_operations::{
    canonical::CanonicalOptions,
    native_service::{
        NativeCanonicalService, NativeModuleSource, NativeServiceError, NativeServiceSpec,
    },
};
use pse_runtime::{
    CancelSource,
    host_operations::engineering::EngineeringFixtureReady,
    workflow::{NativeAllocation, NativeStudyWorker, NativeStudyWorkerSpec, WorkerSettings},
};
use std::{
    io::Read as _,
    panic::AssertUnwindSafe,
    path::PathBuf,
    time::{Duration, Instant},
};
#[path = "support/engineering-study-qualification.rs"]
mod qualification;
#[path = "support/signals.rs"]
mod signals;

type Error = Box<dyn std::error::Error + Send + Sync>;

fn error_report(error: &(dyn std::error::Error + 'static)) -> String {
    let Some(error) = error.downcast_ref::<NativeServiceError>() else {
        return error.to_string();
    };
    let mut rendered = String::new();
    // The native owner exposes every independent settlement through Diagnostic::related.
    // Its privileged bootstrap cause intentionally ends at its credential-safe Display;
    // never use Debug or reopen the private original SQL-bearing bootstrap failure.
    if miette::NarratableReportHandler::new()
        .render_report(&mut rendered, error)
        .is_err()
    {
        return format!("{error}\nnative diagnostic rendering failed");
    }
    rendered
}

#[derive(Debug, Parser)]
#[command(
    about = "Retain a disposable canonical fixture in a foreground native application scope",
    long_about = "Run through scripts/pse-env from an application scope and retain that foreground lifetime. The original canonical owner launches and drains its bound service; a detached managed payload is not the required scope. This keeper never grants engineering-service peer authentication.",
    after_help = "For Rust-owned three-phase client coordination, use: pse-engineering-fixture qualify-study --help. That coordination mode runs outside an enclosing native workload."
)]
struct Args {
    /// Explicit already-built canonical process helper.
    #[arg(long)]
    service_helper: PathBuf,
    /// Explicit already-built scientific worker; this foreground root retains its owner.
    #[arg(long, requires = "worker_allocation")]
    study_worker: Option<PathBuf>,
    /// Explicit original NativeAllocation document for the separately placed worker.
    #[arg(long, requires = "study_worker")]
    worker_allocation: Option<PathBuf>,
    /// Optional original-owner gate; publish {"version":1} atomically to start the worker.
    #[arg(long, requires_all = ["study_worker", "study_worker_started"])]
    study_worker_start: Option<PathBuf>,
    /// Fresh atomic receipt published only after the original worker launch completes.
    #[arg(long, requires = "study_worker_start")]
    study_worker_started: Option<PathBuf>,
    /// Fresh final disposition, published only after the original owners have drained.
    #[arg(long, requires_all = ["drain_request", "drained"])]
    finish_request: Option<PathBuf>,
    /// Fresh request for independent normal worker/service shutdown, granting no disposal.
    #[arg(long, requires = "finish_request")]
    drain_request: Option<PathBuf>,
    /// Fresh observation published only after actual independent owner drain and join checks.
    #[arg(long, requires = "finish_request")]
    drained: Option<PathBuf>,
    /// Explicit already-prepared production module receipt.
    #[arg(
        long,
        conflicts_with = "module_state",
        required_unless_present = "module_state"
    )]
    module_receipt: Option<PathBuf>,
    /// Explicit existing state used only to select its immutable prepared module artifact.
    #[arg(
        long,
        conflicts_with = "module_receipt",
        required_unless_present = "module_receipt"
    )]
    module_state: Option<PathBuf>,
    /// Existing parent in which this owner creates one fresh disposable state.
    #[arg(long)]
    state_parent: PathBuf,
    /// Fresh atomic routing receipt; contains no database credentials.
    #[arg(long)]
    ready: PathBuf,
    #[arg(long, default_value_t = 120_000)]
    startup_ms: u64,
    /// Maximum retained qualification lifetime before orderly shutdown.
    #[arg(long, default_value_t = 600_000)]
    lifetime_ms: u64,
    #[arg(long, default_value_t = 30_000)]
    shutdown_ms: u64,
}

struct Failures(Vec<Error>);
impl std::fmt::Debug for Failures {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Failures")
            .field("count", &self.0.len())
            .finish_non_exhaustive()
    }
}
impl std::fmt::Display for Failures {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index != 0 {
                writeln!(formatter)?;
            }
            write!(formatter, "{}", error_report(error.as_ref()))?;
        }
        Ok(())
    }
}
impl std::error::Error for Failures {}

fn worker_allocation(path: &std::path::Path) -> Result<NativeAllocation, Error> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((64 << 10) + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 << 10 {
        return Err("worker allocation exceeds64KiB".into());
    }
    let allocation: NativeAllocation = serde_json::from_slice(&bytes)?;
    allocation.validate()?;
    Ok(allocation)
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct WorkerStart {
    version: pse_model::document::Version<1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum FixtureDisposition {
    Preserve,
    Dispose,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct FinishRequest {
    version: pse_model::document::Version<1>,
    disposition: FixtureDisposition,
}
fn finish_requested(path: &std::path::Path) -> Result<Option<FinishRequest>, Error> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !file.metadata()?.is_file() {
        return Err("finish request must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(1025).read_to_end(&mut bytes)?;
    if bytes.len() > 1024 {
        return Err("finish request exceeds1024bytes".into());
    }
    Ok(Some(serde_json::from_slice(&bytes)?))
}
fn publish_finish_request(
    path: &std::path::Path,
    disposition: FixtureDisposition,
) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or("finish request needs an existing parent")?;
    let mut receipt = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(
        receipt.as_file_mut(),
        &FinishRequest {
            version: Default::default(),
            disposition,
        },
    )?;
    receipt.as_file().sync_all()?;
    receipt
        .persist_noclobber(path)
        .map_err(|error| error.error)?;
    Ok(())
}
fn complete_fixture_cleanup(
    disposition: FixtureDisposition,
    stop: Option<&CancelSource>,
    failures: &mut Vec<Error>,
    dispose: impl FnOnce() -> Vec<Error>,
) -> FixtureDisposition {
    if disposition == FixtureDisposition::Dispose && failures.is_empty() {
        if stop.is_some_and(|stop| stop.token().is_cancelled()) {
            failures
                .push("keeper stopped before cleanup acceptance; stopped fixture retained".into());
        } else {
            // This synchronous acceptance follows actual drain and final disposition.
            // A later stop cannot revoke an irreversible cleanup already accepted here.
            failures.extend(dispose());
            return FixtureDisposition::Dispose;
        }
    }
    FixtureDisposition::Preserve
}

fn start_requested(path: &std::path::Path) -> Result<bool, Error> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    if !file.metadata()?.is_file() {
        return Err("worker start request must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(1025).read_to_end(&mut bytes)?;
    if bytes.len() > 1024 {
        return Err("worker start request exceeds1024bytes".into());
    }
    let _: WorkerStart = serde_json::from_slice(&bytes)?;
    Ok(true)
}

fn take_requested_worker<T>(
    pending: &mut Option<T>,
    gate: Option<&std::path::Path>,
) -> Result<Option<T>, Error> {
    if pending.is_some()
        && let Some(gate) = gate
        && start_requested(gate)?
    {
        return Ok(pending.take());
    }
    Ok(None)
}

fn publish_worker_started(path: &std::path::Path) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or("worker started receipt needs an existing parent")?;
    let mut receipt = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(
        receipt.as_file_mut(),
        &WorkerStart {
            version: Default::default(),
        },
    )?;
    receipt.as_file().sync_all()?;
    receipt.persist_noclobber(path)?;
    Ok(())
}

async fn run(args: Args) -> Result<(), Error> {
    let environment = std::env::vars_os().collect();
    let module_source = match (args.module_receipt, args.module_state) {
        (Some(receipt), None) => NativeModuleSource::Prepared(pse_host::study_module::select(
            &receipt,
            pse_host::study_module::Variant::Production,
        )?),
        (None, Some(state)) => NativeModuleSource::RecordedState(state),
        _ => return Err("select exactly one original module receipt or recorded state".into()),
    };
    // Parse original deployment meaning before any service effect. This does not
    // prepare a provider or turn a routing receipt into a performing owner.
    let allocation = args
        .worker_allocation
        .as_deref()
        .map(worker_allocation)
        .transpose()?;
    if let Some(finish) = &args.finish_request {
        if !finish.is_absolute()
            || finish.try_exists()?
            || args.study_worker_start.as_ref() == Some(finish)
            || args.study_worker_started.as_ref() == Some(finish)
            || &args.ready == finish
        {
            return Err("keeper finish request requires a fresh distinct absolute path".into());
        }
    }
    for request in [&args.drain_request, &args.drained].into_iter().flatten() {
        if !request.is_absolute()
            || request.try_exists()?
            || args.finish_request.as_ref() == Some(request)
            || args.study_worker_start.as_ref() == Some(request)
            || args.study_worker_started.as_ref() == Some(request)
            || &args.ready == request
            || args.drain_request.as_ref() == Some(request)
                && args.drained.as_ref() == Some(request)
        {
            return Err(
                "keeper drain request/observation requires fresh distinct absolute paths".into(),
            );
        }
    }
    if let Some(gate) = &args.study_worker_start {
        let started = args
            .study_worker_started
            .as_ref()
            .ok_or("worker started receipt absent")?;
        if !gate.is_absolute() || !started.is_absolute() || gate == started || started.exists() {
            return Err(
                "worker gate and fresh started receipt require distinct absolute paths".into(),
            );
        }
    }
    let mut service = NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment)?,
        helper: args.service_helper,
        module_source,
        state_parent: args.state_parent,
    })?;
    let mut worker: Option<NativeStudyWorker> = None;
    let stop = CancelSource::new();
    let signal = signals::ProcessInterrupt::new(stop.clone())?;
    let clock = Instant::now();
    let startup = clock
        .checked_add(Duration::from_millis(args.startup_ms))
        .ok_or("startup allowance exceeds clock range")?;
    let lifetime = args
        .startup_ms
        .checked_add(args.lifetime_ms)
        .and_then(|value| value.checked_add(args.shutdown_ms))
        .ok_or("keeper allowance exceeds clock range")?;
    let terminal = clock
        .checked_add(Duration::from_millis(lifetime))
        .ok_or("terminal allowance exceeds clock range")?;
    let action = AssertUnwindSafe(pse_operations::transport::operation_scope(terminal, async {
        service.launch(startup).await?;
        service.connect().await?;
        let mut pending_worker = if let (Some(executable), Some(allocation)) = (args.study_worker, allocation) {
            Some(NativeStudyWorkerSpec {
                ready: service.ready()?, executable, allocation,
                environment: std::env::vars().filter(|(key, _)| !key.starts_with("NEXTEST_") && !key.starts_with("CARGO_BIN_EXE_")).collect(),
                settings: WorkerSettings::default(),
            })
        } else { None };
        if args.study_worker_start.is_none() && let Some(spec) = pending_worker.take() {
            worker = Some(NativeStudyWorker::new(spec)?);
            worker.as_mut().ok_or("original worker owner unavailable")?.launch(startup).await?;
        }
        let options = CanonicalOptions::from_state(service.state())?;
        let receipt = EngineeringFixtureReady { version: Default::default(), state: service.state().to_owned(), database: options.database, fixture: service.fixture_selection()? };
        let parent = args.ready.parent().ok_or(NativeServiceError::Refused("ready receipt needs an existing parent"))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer(temporary.as_file_mut(), &receipt).map_err(|_| NativeServiceError::Refused("ready receipt encoding failed"))?;
        temporary.as_file().sync_all()?;
        temporary.persist_noclobber(&args.ready).map_err(|error| NativeServiceError::from(error.error))?;
        let active_until = Instant::now().checked_add(Duration::from_millis(args.lifetime_ms))
            .ok_or("keeper active allowance exceeds clock range")?;
        loop {
            tokio::select! {
                _ = stop.cancelled() => break,
                _ = tokio::time::sleep_until(active_until.into()) => break,
                _ = tokio::time::sleep(Duration::from_millis(25)), if pending_worker.is_some() || args.finish_request.is_some() => {
                    if let Some(drain) = &args.drain_request && start_requested(drain)? {
                        break;
                    }
                    if let Some(finish) = &args.finish_request && finish_requested(finish)?.is_some() {
                        break;
                    }
                    if let Some(spec) = take_requested_worker(&mut pending_worker, args.study_worker_start.as_deref())? {
                        worker = Some(NativeStudyWorker::new(spec)?);
                        // A late gate receives the original startup allowance only
                        // within this keeper's predeclared active/terminal window.
                        let deadline = Instant::now().checked_add(Duration::from_millis(args.startup_ms))
                            .ok_or("worker startup allowance exceeds clock range")?.min(active_until);
                        worker.as_mut().ok_or("original worker owner unavailable")?.launch(deadline).await?;
                        publish_worker_started(args.study_worker_started.as_deref().ok_or("worker started receipt absent")?)?;
                    }
                }
            }
        }
        Ok::<(), Error>(())
    })).catch_unwind().await;
    // Early finish gets only the declared settlement allowance, bounded by the
    // original lifetime. A late request cannot restart or extend that clock.
    let terminal = Instant::now()
        .checked_add(Duration::from_millis(args.shutdown_ms))
        .map_or(terminal, |deadline| deadline.min(terminal));
    // Every independent teardown is attempted even after launch ambiguity, body
    // failure or panic. The frontend never receives this performing worker owner.
    let mut failures: Vec<Error> = Vec::new();
    if let Some(worker) = &mut worker {
        if let Err(error) = worker.shutdown(terminal).await {
            failures.push(Box::new(error));
        }
    }
    if let Err(error) = service.shutdown(terminal).await {
        failures.push(Box::new(error));
    }
    match action {
        Ok(Ok(())) => {}
        Ok(Err(error)) => failures.insert(0, error),
        Err(panic) => {
            signal.finish().await;
            if let Some(worker) = &worker {
                eprintln!(
                    "engineering scientific owner retained at {}",
                    worker.directory().display()
                );
            }
            eprintln!(
                "engineering fixture retained at {}; independent teardown failures: {}",
                service.state().display(),
                Failures(failures)
            );
            std::panic::resume_unwind(panic);
        }
    }
    if failures.is_empty()
        && let Some(worker) = &worker
    {
        match worker.terminal() {
            Ok(Some(receipt)) if receipt.failure.is_some() => failures.push("original scientific worker reported failure; its original terminal and diagnostics are retained".into()),
            Ok(Some(receipt)) if receipt.native_joined => {}
            Ok(_) => failures.push("original scientific worker application join is unestablished".into()),
            Err(error) => failures.push(Box::new(error)),
        }
    }
    let mut disposition = FixtureDisposition::Dispose;
    if let Some(finish) = &args.finish_request {
        disposition = FixtureDisposition::Preserve;
        if failures.is_empty()
            && let Some(drained) = &args.drained
            && let Err(error) = publish_worker_started(drained)
        {
            failures.push(error);
        }
        loop {
            if stop.token().is_cancelled() {
                failures.push(
                    "keeper stopped before final cleanup acceptance; stopped fixture retained"
                        .into(),
                );
                break;
            }
            match finish_requested(finish) {
                Ok(Some(request)) => {
                    disposition = request.disposition;
                    break;
                }
                Ok(None) => {}
                Err(error) => {
                    failures.push(error);
                    break;
                }
            }
            if Instant::now() >= terminal {
                failures.push(
                    "final finish disposition absent at deadline; stopped fixture retained".into(),
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    let accepted = complete_fixture_cleanup(
        disposition,
        args.finish_request.as_ref().map(|_| &stop),
        &mut failures,
        || {
            let mut causes: Vec<Error> = Vec::new();
            if let Some(worker) = &mut worker {
                if let Err(error) = worker.dispose() {
                    causes.push(Box::new(error));
                }
            }
            if let Err(error) = service.dispose() {
                causes.push(Box::new(error));
            }
            causes
        },
    );
    signal.finish().await;
    if accepted == FixtureDisposition::Preserve && failures.is_empty() {
        eprintln!(
            "engineering fixture stopped and retained at {}",
            service.state().display()
        );
    }
    if failures.is_empty() {
        Ok(())
    } else {
        if let Some(worker) = &worker {
            eprintln!(
                "engineering scientific owner retained at {}",
                worker.directory().display()
            );
        }
        eprintln!(
            "engineering fixture retained at {}",
            service.state().display()
        );
        Err(Box::new(Failures(failures)))
    }
}
fn main() -> std::process::ExitCode {
    let result: Result<(), Error> = (|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("qualify-study")) {
            let args = qualification::Args::parse_from(
                std::iter::once(std::ffi::OsString::from(
                    "pse-engineering-fixture qualify-study",
                ))
                .chain(std::env::args_os().skip(2)),
            );
            runtime.block_on(qualification::run(args))
        } else {
            runtime.block_on(run(Args::parse()))
        }
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pse-engineering-fixture: {}", error_report(error.as_ref()));
            if error
                .downcast_ref::<qualification::Failure>()
                .is_some_and(|failure| failure.interrupted)
            {
                std::process::ExitCode::from(130)
            } else {
                std::process::ExitCode::FAILURE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "credential-safe original diagnostic controls"
    )]
    use super::*;

    #[test]
    fn scientific_keeper_arguments_require_original_image_and_allocation_together() {
        let base = [
            "fixture",
            "--service-helper",
            "prepared-canonical",
            "--module-receipt",
            "prepared-module",
            "--state-parent",
            "existing-parent",
            "--ready",
            "fresh-ready",
        ];
        assert!(Args::try_parse_from(base).is_ok());
        assert!(
            Args::try_parse_from(base.into_iter().chain(["--finish-request", "finish.json"]))
                .is_err()
        );
        assert!(
            Args::try_parse_from(base.into_iter().chain(["--drain-request", "drain.json"]))
                .is_err()
        );
        assert!(
            Args::try_parse_from(base.into_iter().chain([
                "--finish-request",
                "finish.json",
                "--drain-request",
                "drain.json",
                "--drained",
                "drained.json",
            ]))
            .is_ok()
        );
        let worker_only = base
            .into_iter()
            .chain(["--study-worker", "prepared-pse-worker"]);
        assert!(Args::try_parse_from(worker_only).is_err());
        let allocation_only = base
            .into_iter()
            .chain(["--worker-allocation", "original-allocation.json"]);
        assert!(Args::try_parse_from(allocation_only).is_err());
        assert!(
            Args::try_parse_from(base.into_iter().chain([
                "--study-worker",
                "prepared-pse-worker",
                "--worker-allocation",
                "original-allocation.json"
            ]))
            .is_ok()
        );
        let worker = base.into_iter().chain([
            "--study-worker",
            "prepared-pse-worker",
            "--worker-allocation",
            "original-allocation.json",
        ]);
        assert!(
            Args::try_parse_from(worker.clone().chain(["--study-worker-start", "gate.json"]))
                .is_err()
        );
        assert!(
            Args::try_parse_from(
                worker
                    .clone()
                    .chain(["--study-worker-started", "started.json"])
            )
            .is_err()
        );
        assert!(
            Args::try_parse_from(worker.chain([
                "--study-worker-start",
                "gate.json",
                "--study-worker-started",
                "started.json"
            ]))
            .is_ok()
        );
    }

    #[test]
    fn scientific_keeper_gate_is_strict_bounded_and_started_receipt_is_exclusive() {
        let directory = tempfile::tempdir().unwrap();
        let gate = directory.path().join("start.json");
        let started = directory.path().join("started.json");
        assert!(!start_requested(&gate).unwrap());
        assert!(!started.exists());
        for invalid in [
            br#"{}"#.as_slice(),
            br#"{"version":2}"#,
            br#"{"version":1,"unexpected":true}"#,
        ] {
            std::fs::write(&gate, invalid).unwrap();
            assert!(start_requested(&gate).is_err());
            assert!(!started.exists());
        }
        std::fs::write(&gate, [b' '; 1025]).unwrap();
        assert!(
            start_requested(&gate)
                .unwrap_err()
                .to_string()
                .contains("exceeds1024bytes")
        );
        std::fs::write(&gate, br#"{"version":1}"#).unwrap();
        assert!(start_requested(&gate).unwrap());
        publish_worker_started(&started).unwrap();
        let original = std::fs::read(&started).unwrap();
        assert_eq!(original, br#"{"version":1}"#);
        assert!(publish_worker_started(&started).is_err());
        assert_eq!(std::fs::read(&started).unwrap(), original);
    }

    #[test]
    fn scientific_keeper_persistent_gate_hands_off_original_worker_only_once() {
        let directory = tempfile::tempdir().unwrap();
        let gate = directory.path().join("start.json");
        let mut pending = Some("original worker configuration");
        assert!(
            take_requested_worker(&mut pending, Some(&gate))
                .unwrap()
                .is_none()
        );
        assert!(pending.is_some());
        publish_worker_started(&gate).unwrap();
        let original = std::fs::read(&gate).unwrap();
        assert_eq!(
            take_requested_worker(&mut pending, Some(&gate)).unwrap(),
            Some("original worker configuration")
        );
        // Finish polling continues after the successful launch. Its next ticks
        // must neither relaunch nor consume the persistent original gate.
        for _ in 0..3 {
            assert!(
                take_requested_worker(&mut pending, Some(&gate))
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(std::fs::read(&gate).unwrap(), original);
    }

    #[test]
    fn scientific_keeper_final_disposition_is_one_strict_atomic_request() {
        let directory = tempfile::tempdir().unwrap();
        let finish = directory.path().join("finish.json");
        assert!(finish_requested(&finish).unwrap().is_none());
        for invalid in [
            br#"{"version":1}"#.as_slice(),
            br#"{"version":2,"disposition":"dispose"}"#,
            br#"{"version":1,"disposition":"dispose","extra":true}"#,
            br#"{"version":1,"disposition":"unknown"}"#,
        ] {
            std::fs::write(&finish, invalid).unwrap();
            assert!(finish_requested(&finish).is_err());
        }
        std::fs::write(&finish, [b' '; 1025]).unwrap();
        assert!(finish_requested(&finish).is_err());
        std::fs::remove_file(&finish).unwrap();
        publish_finish_request(&finish, FixtureDisposition::Preserve).unwrap();
        let original = std::fs::read(&finish).unwrap();
        assert_eq!(original, br#"{"version":1,"disposition":"preserve"}"#);
        assert_eq!(
            finish_requested(&finish).unwrap().unwrap().disposition,
            FixtureDisposition::Preserve
        );
        assert!(publish_finish_request(&finish, FixtureDisposition::Dispose).is_err());
        assert_eq!(std::fs::read(&finish).unwrap(), original);
        let link = directory.path().join("linked.json");
        std::os::unix::fs::symlink(&finish, &link).unwrap();
        assert!(finish_requested(&link).is_err());
    }

    #[test]
    fn scientific_keeper_cleanup_acceptance_preserves_on_stop_or_drain_failure_and_disposes_success()
     {
        for condition in ["stopped", "failed-drain", "success", "late-stop"] {
            let directory = tempfile::tempdir().unwrap();
            let fixture = directory.path().join("original-stopped-fixture");
            std::fs::write(&fixture, b"original state").unwrap();
            let stop = CancelSource::new();
            if condition == "stopped" {
                stop.cancel();
            }
            let mut failures: Vec<Error> = if condition == "failed-drain" {
                vec!["original independent drain failed".into()]
            } else {
                Vec::new()
            };
            let accepted = complete_fixture_cleanup(
                FixtureDisposition::Dispose,
                Some(&stop),
                &mut failures,
                || {
                    if condition == "late-stop" {
                        stop.cancel();
                    }
                    std::fs::remove_file(&fixture).unwrap();
                    Vec::new()
                },
            );
            let disposed = matches!(condition, "success" | "late-stop");
            assert_eq!(accepted == FixtureDisposition::Dispose, disposed);
            assert_eq!(fixture.exists(), !disposed);
            assert_eq!(failures.is_empty(), disposed);
        }
    }

    #[test]
    fn scientific_keeper_allocation_preserves_profile_and_refuses_extent_before_decode() {
        let allocation =
            NativeAllocation::materialize(8 << 30, 512 << 20, 1, 6 << 30, None).unwrap();
        let mut document = tempfile::NamedTempFile::new().unwrap();
        serde_json::to_writer(document.as_file_mut(), &allocation).unwrap();
        assert_eq!(worker_allocation(document.path()).unwrap(), allocation);
        std::fs::write(document.path(), vec![b' '; (64 << 10) + 1]).unwrap();
        assert!(
            worker_allocation(document.path())
                .unwrap_err()
                .to_string()
                .contains("exceeds64KiB")
        );
        let mut invalid = allocation;
        invalid.native_workers = 0;
        std::fs::write(document.path(), serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(worker_allocation(document.path()).is_err());
    }

    #[test]
    fn fixture_diagnostic_preserves_nested_independent_settlement_failures() {
        let error = NativeServiceError::Settlement {
            state: PathBuf::from("retained-state"),
            failures: vec![
                NativeServiceError::Host(pse_host::HostError::Ownership(
                    "bound service requires this process's original native workload scope".into(),
                )),
                NativeServiceError::Settlement {
                    state: PathBuf::from("retained-state"),
                    failures: vec![NativeServiceError::Refused(
                        "original launch remains uncertain; no physical release was established",
                    )],
                },
            ],
        };
        let rendered = error_report(&error);
        assert!(
            rendered.contains("after 2 settlement failures"),
            "{rendered}"
        );
        assert!(
            rendered.contains("requires this process's original native workload scope"),
            "{rendered}"
        );
        assert!(
            rendered.contains("no physical release was established"),
            "{rendered}"
        );
        assert!(rendered.contains("runtime::infrastructure"), "{rendered}");
    }

    #[test]
    fn fixture_diagnostic_uses_public_display_without_debugging_private_causes() {
        struct Private;
        impl std::fmt::Debug for Private {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("private-credential-and-SQL")
            }
        }
        impl std::fmt::Display for Private {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("credential-safe original failure")
            }
        }
        impl std::error::Error for Private {}
        let rendered = error_report(&Private);
        assert_eq!(rendered, "credential-safe original failure");
        assert!(!rendered.contains("private-credential-and-SQL"));
    }
}
