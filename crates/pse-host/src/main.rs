// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Narrow host composition root, independent of scientific initialization.
use pse_host::HostError;
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
                "pse-host: native host capabilities\n  env --print|--explain [--json]\n  activity [UNIT.service|UNIT.scope]\n  doctor [--json]\n  cargo-tools [NAME...]\n  diagnostic-tools install|verify TOOL\n  telemetry-service --help\n  native prepare --capabilities compiler[,solver,klu,isolation,uno,petsc]\n  server prepare|select\n  study-module prepare|select\n  run /absolute/executable [args...]\n  --version\nConstruction/update is explicit: scripts/bootstrap-host. Reads never build."
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
        Some("agent-hook") => pse_host::agent_hooks::run_cli(&arguments[1..]),
        Some("doctor") => runtime()?.block_on(pse_host::doctor::run_cli(&arguments[1..])),
        Some("cargo-tools") => {
            let names = arguments[1..].iter().map(|name| name.to_str().map(str::to_owned).ok_or_else(|| HostError::Ownership("Cargo tool names must be UTF-8".into()))).collect::<Result<Vec<_>, _>>()?;
            for spec in pse_host::diagnostic_tools::cargo_tool_specs(&checkout_root()?, &names)? {
                writeln!(io::stdout(), "{spec}")?;
            }
            Ok(0)
        },
        Some("diagnostic-tools") => {
            let action = arguments.get(1).and_then(|value| value.to_str()).ok_or_else(|| HostError::Ownership("diagnostic-tools install|verify TOOL".into()))?;
            let tool = arguments.get(2).and_then(|value| value.to_str()).ok_or_else(|| HostError::Ownership("diagnostic-tools requires a declared tool".into()))?;
            if arguments.len() != 3 { return Err(HostError::Ownership("diagnostic-tools install|verify TOOL".into())); }
            let root = checkout_root()?;
            let values = std::env::vars_os().collect();
            let tool = pse_host::diagnostic_tools::Tool::parse(tool)?;
            let path = match action {
                "install" => pse_host::diagnostic_tools::install(&root, tool, &values)?,
                "verify" => pse_host::diagnostic_tools::verify(&root, tool, &values)?,
                _ => return Err(HostError::Ownership("diagnostic-tools accepts install or verify".into())),
            };
            writeln!(io::stdout(), "{}", path.display())?;
            Ok(0)
        },
        Some("telemetry-service") => {
            let result = runtime()?.block_on(pse_host::telemetry_service::command(&checkout_root()?, &std::env::vars_os().collect(), &arguments[1..]))?;
            serde_json::to_writer_pretty(io::stdout(), &result).map_err(io::Error::other)?;
            writeln!(io::stdout())?;
            Ok(0)
        },
        Some("native") => pse_host::native::run_cli(&arguments[1..]),
        Some("server") => pse_host::server::run_cli(&arguments[1..]),
        Some("study-module") => pse_host::study_module::run_cli(&arguments[1..]),
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
                let observation = pse_host::activity::ActivityObservation::read(name).await;
                serde_json::to_writer(io::stdout(), &observation).map_err(io::Error::other)?;
                writeln!(io::stdout())?;
                Ok(if observation.complete() { 0 } else { 125 })
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
fn checkout_root() -> Result<std::path::PathBuf, HostError> {
    let mut root = std::env::current_dir()?;
    while !root.join(".config/diagnostic-tools.toml").is_file() {
        if !root.pop() { return Err(HostError::Ownership("run inside a pse-arrow checkout".into())); }
    }
    Ok(root)
}
