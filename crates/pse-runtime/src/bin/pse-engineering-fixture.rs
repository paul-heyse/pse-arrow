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
    workflow::{NativeAllocation, NativeStudyWorker, NativeStudyWorkerSpec, WorkerSettings},
};
use std::{
    io::Read as _,
    panic::AssertUnwindSafe,
    path::PathBuf,
    time::{Duration, Instant},
};
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
    long_about = "Run through scripts/pse-env from an application scope and retain that foreground lifetime. The original canonical owner launches and drains its bound service; a detached managed payload is not the required scope. This keeper never grants engineering-service peer authentication."
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

#[derive(serde::Serialize)]
struct Ready<'a> {
    version: pse_model::document::Version<1>,
    state: &'a std::path::Path,
    database: String,
    fixture: pse_operations::native_fixture::FixtureSelection,
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
        let receipt = Ready { version: Default::default(), state: service.state(), database: options.database, fixture: service.fixture_selection()? };
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
                _ = tokio::time::sleep(Duration::from_millis(25)), if pending_worker.is_some() => {
                    if let Some(gate) = &args.study_worker_start && start_requested(gate)? {
                        let spec = pending_worker.take().ok_or("original pending worker configuration absent")?;
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
    signal.finish().await;
    match action {
        Ok(Ok(())) => {}
        Ok(Err(error)) => failures.insert(0, error),
        Err(panic) => {
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
    if failures.is_empty() {
        if let Some(worker) = &mut worker {
            if let Err(error) = worker.dispose() {
                failures.push(Box::new(error));
            }
        }
        if let Err(error) = service.dispose() {
            failures.push(Box::new(error));
        }
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
        let args = Args::parse();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        runtime.block_on(run(args))
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pse-engineering-fixture: {}", error_report(error.as_ref()));
            std::process::ExitCode::FAILURE
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
