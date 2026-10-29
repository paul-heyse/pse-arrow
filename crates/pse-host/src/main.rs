// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Narrow host composition root, independent of scientific initialization.
use pse_host::{HostError, HostManager};
use std::{
    ffi::OsString,
    io::{self, Write},
    process::ExitCode,
    time::{Duration, Instant},
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
        Err(error) => {
            let prefix = if std::env::args_os()
                .nth(1)
                .is_some_and(|value| value == "boundary")
            {
                "pse-env"
            } else {
                "pse-host"
            };
            let _ = writeln!(io::stderr(), "{prefix}: {error}");
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
                "pse-host: native host capabilities\n  env --print|--explain [--json]\n  activity [UNIT.service|UNIT.scope]\n  run /absolute/executable [args...]\n  --version\nConstruction/update is explicit: scripts/bootstrap-host. Reads never build."
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
        Some("boundary") => pse_host::boundary::run_cli(&arguments[1..]),
        Some("env") => Ok(pse_host::environment::run_cli(&arguments[1..])?),
        Some("activity") => {
            if arguments.len() > 2 {
                return Err(HostError::Ownership(
                    "activity accepts an optional unit name".into(),
                ));
            }
            let name = arguments
                .get(1)
                .map(|name| {
                    name.to_str()
                        .ok_or_else(|| HostError::Ownership("unit name must be UTF-8".into()))
                })
                .transpose()?;
            runtime()?.block_on(async {
                let manager = HostManager::connect_user(None).await?;
                let observation = match name {
                    Some(name) => manager.borrow(name).observe().await?,
                    None => manager.current_unit().await?,
                };
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
            runtime()?.block_on(pse_host::foreground::run(
                pse_host::foreground::ForegroundRequest {
                    command: arguments[1..].to_vec(),
                    environment: std::env::vars_os().collect(),
                    cwd: std::env::current_dir()?,
                    require_enclosing: false,
                    startup_deadline: Instant::now() + Duration::from_secs(30),
                    drain_timeout: Duration::from_secs(30),
                },
            ))
        }
        Some("__foreground-exec") => pse_host::foreground::trampoline(&arguments[1..]),
        Some(_) => Err(HostError::Ownership("unknown operation; use --help".into())),
    }
}
fn runtime() -> Result<tokio::runtime::Runtime, HostError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}
