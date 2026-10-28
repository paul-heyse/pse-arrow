// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Narrow host composition root, independent of scientific initialization.
use pse_host::{HostError, HostManager, LaunchIntent, ServiceSpec};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
        Err(error) => {
            let _ = writeln!(io::stderr(), "pse-host: {error}");
            ExitCode::from(125)
        }
    }
}
fn run() -> Result<i32, HostError> {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let operation = arguments
        .first()
        .map(|value| {
            value
                .to_str()
                .ok_or_else(|| HostError::Ownership("operation name must be UTF-8".into()))
        })
        .transpose()?;
    match operation {
        None | Some("help" | "--help" | "-h") => {
            writeln!(
                io::stdout(),
                "pse-host: native host capabilities\n  env --print|--explain [--json]\n  activity UNIT.service\n  run /absolute/executable [args...]\n  --version\nConstruction/update is explicit: scripts/bootstrap-host. Reads never build."
            )?;
            Ok(0)
        }
        Some("--version") => {
            writeln!(
                io::stdout(),
                "pse-host {} protocol 1",
                env!("CARGO_PKG_VERSION")
            )?;
            Ok(0)
        }
        Some("env") => Ok(pse_host::environment::run_cli(&arguments[1..])?),
        Some("activity") => {
            if arguments.len() != 2 {
                return Err(HostError::Ownership(
                    "activity requires one unit name".into(),
                ));
            }
            let name = arguments[1]
                .to_str()
                .ok_or_else(|| HostError::Ownership("unit name must be UTF-8".into()))?;
            runtime()?.block_on(async {
                let manager = HostManager::connect_user(None).await?;
                let observation = manager.borrow(name).observe().await?;
                serde_json::to_writer(io::stdout(), &observation).map_err(io::Error::other)?;
                writeln!(io::stdout())?;
                Ok(0)
            })
        }
        Some("run") => {
            if arguments.len() < 2 {
                return Err(HostError::Ownership(
                    "run requires an absolute executable".into(),
                ));
            }
            let executable = PathBuf::from(&arguments[1]);
            let args = arguments[2..]
                .iter()
                .map(|arg| {
                    arg.clone()
                        .into_string()
                        .map_err(|_| HostError::Ownership("D-Bus arguments must be UTF-8".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            runtime()?.block_on(async {
                let manager = HostManager::connect_user(None).await?;
                let spec = ServiceSpec {
                    intent: LaunchIntent::fresh()?,
                    executable,
                    arguments: args,
                    environment: std::env::vars_os()
                        .map(|(key, value)| {
                            let key = key.into_string().map_err(|_| {
                                HostError::Ownership("D-Bus environment key must be UTF-8".into())
                            })?;
                            let value = value.into_string().map_err(|_| {
                                HostError::Ownership("D-Bus environment value must be UTF-8".into())
                            })?;
                            Ok(format!("{key}={value}"))
                        })
                        .collect::<Result<Vec<_>, HostError>>()?,
                };
                let mut interrupt =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
                let mut terminate =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
                let mut unit = match manager
                    .start_service(&spec, Instant::now() + Duration::from_secs(30))
                    .await
                {
                    Ok(unit) => unit,
                    Err(operation @ HostError::UnknownLaunch { .. }) => {
                        let cleanup = async {
                            let mut unit = manager
                                .reconcile(&spec.intent, Instant::now() + Duration::from_secs(30))
                                .await?;
                            unit.stop_and_drain(Instant::now() + Duration::from_secs(30))
                                .await
                        }
                        .await;
                        return match cleanup {
                            Ok(_) => Err(operation),
                            Err(cleanup) => Err(HostError::Settlement {
                                operation: Box::new(operation),
                                cleanup: Box::new(cleanup),
                            }),
                        };
                    }
                    Err(error) => return Err(error),
                };
                let outcome: Result<i32, HostError> = async {
                    loop {
                        let observation = unit.observe().await?;
                        if let Some(code) = observation.completed_status() {
                            break Ok(code);
                        }
                        tokio::select! {
                            _ = interrupt.recv() => {
                                break Ok(130);
                            }
                            _ = terminate.recv() => { break Ok(143); }
                            () = tokio::time::sleep(Duration::from_millis(20)) => {}
                        }
                    }
                }
                .await;
                let cleanup = unit
                    .stop_and_drain(Instant::now() + Duration::from_secs(30))
                    .await;
                match (outcome, cleanup) {
                    (Ok(status), Ok(_)) => Ok(status),
                    (Err(operation), Ok(_)) => Err(operation),
                    (Err(operation), Err(cleanup)) => Err(HostError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(cleanup),
                    }),
                    (Ok(status), Err(cleanup)) => Err(HostError::Ownership(format!(
                        "original child status {status}; cleanup failed: {cleanup}"
                    ))),
                }
            })
        }
        Some(_) => Err(HostError::Ownership("unknown operation; use --help".into())),
    }
}
fn runtime() -> Result<tokio::runtime::Runtime, HostError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}
