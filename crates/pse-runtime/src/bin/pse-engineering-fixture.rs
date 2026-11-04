// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit retained disposable canonical owner for independent engineering-client qualification.
#![allow(
    clippy::print_stderr,
    reason = "qualification owner reports terminal failure and retained state"
)]

use clap::Parser;
use pse_operations::{
    canonical::CanonicalOptions,
    native_service::{
        NativeCanonicalService, NativeModuleSource, NativeServiceError, NativeServiceSpec,
    },
};
use pse_runtime::CancelSource;
use std::{
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
    let mut service = NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment)?,
        helper: args.service_helper,
        module_source,
        state_parent: args.state_parent,
    })?;
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
    let action = service.run(terminal, |service| Box::pin(async move {
        service.launch(startup).await?;
        service.connect().await?;
        let options = CanonicalOptions::from_state(service.state())?;
        let receipt = Ready { version: Default::default(), state: service.state(), database: options.database, fixture: service.fixture_selection()? };
        let parent = args.ready.parent().ok_or(NativeServiceError::Refused("ready receipt needs an existing parent"))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer(temporary.as_file_mut(), &receipt).map_err(|_| NativeServiceError::Refused("ready receipt encoding failed"))?;
        temporary.as_file().sync_all()?;
        temporary.persist_noclobber(&args.ready).map_err(|error| NativeServiceError::from(error.error))?;
        tokio::select! { _ = stop.cancelled() => {}, _ = tokio::time::sleep(Duration::from_millis(args.lifetime_ms)) => {} }
        Ok(())
    })).await;
    signal.finish().await;
    match action {
        Ok(()) => {
            service.dispose()?;
            Ok(())
        }
        Err(error) => {
            eprintln!(
                "engineering fixture retained at {}",
                service.state().display()
            );
            Err(Box::new(error))
        }
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
