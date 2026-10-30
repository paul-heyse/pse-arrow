// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `telemetry-service <action>`: one pretty JSON result. Inspection never installs, loads,
//! repairs or starts anything.
use super::Context;
use crate::{
    HostError,
    telemetry_service::{self, SetupOptions},
};
use clap::{Args, Subcommand};
use std::{io::Write as _, path::PathBuf};

/// Telemetry service action.
#[derive(Debug, Args)]
#[command(
    subcommand_required = true,
    arg_required_else_help = true,
    after_help = "Existing enabled deployments must be explicitly disabled before reconfiguration."
)]
pub(crate) struct TelemetryServiceArgs {
    /// The action.
    #[command(subcommand)]
    pub(crate) action: TelemetryAction,
}

/// An explicit state directory instead of the selected default.
#[derive(Debug, Args)]
pub(crate) struct StateArgs {
    /// Telemetry state directory.
    #[arg(long, value_name = "PATH")]
    pub(crate) state: Option<PathBuf>,
}

/// Every stateful action is selected by name.
#[derive(Debug, Subcommand)]
pub(crate) enum TelemetryAction {
    /// Install the pinned collector and backend releases.
    Install,
    /// Configure (never start) the local collector and backend.
    Setup(SetupArgs),
    /// Passive observation of the configured deployment.
    Inspect(StateArgs),
    /// Start the configured services.
    Start(StateArgs),
    /// Stop the configured services.
    Stop(StateArgs),
    /// Apply a changed configuration.
    Reconfigure(StateArgs),
    /// Restore the configuration preceding an interrupted reconfiguration.
    #[command(name = "recover-configuration")]
    RecoverConfiguration(StateArgs),
}

/// Setup selections; omitted listeners keep the established local defaults.
#[derive(Debug, Args)]
pub(crate) struct SetupArgs {
    #[command(flatten)]
    pub(crate) state: StateArgs,
    /// Listener override `NAME=PORT`; repeatable, each name once.
    #[arg(long = "port", value_name = "NAME=PORT", action = clap::ArgAction::Append, value_parser = port)]
    pub(crate) ports: Vec<(String, u16)>,
    /// Local backend organization identifier.
    #[arg(long, overrides_with = "organization")]
    pub(crate) organization: Option<String>,
    /// Stream name `SIGNAL=NAME` for traces, metrics or logs; repeatable, last wins.
    #[arg(long = "stream", value_name = "SIGNAL=NAME", action = clap::ArgAction::Append, value_parser = stream)]
    pub(crate) streams: Vec<(String, String)>,
    /// Backend retention in days (at least three).
    #[arg(long, value_name = "DAYS", overrides_with = "retention_days")]
    pub(crate) retention_days: Option<u32>,
}

fn invalid(message: &str) -> HostError {
    HostError::Ownership(message.into())
}
fn port(value: &str) -> Result<(String, u16), HostError> {
    let (name, port) = value
        .split_once('=')
        .ok_or_else(|| invalid("telemetry port must be NAME=PORT"))?;
    let port = port
        .parse::<u16>()
        .map_err(|_| invalid("telemetry port must be an integer from 1 through 65535"))?;
    Ok((name.into(), port))
}
fn stream(value: &str) -> Result<(String, String), HostError> {
    let (name, stream) = value
        .split_once('=')
        .ok_or_else(|| invalid("telemetry stream must be SIGNAL=NAME"))?;
    Ok((name.into(), stream.into()))
}

fn setup_options(args: SetupArgs) -> Result<SetupOptions, HostError> {
    let mut options = SetupOptions::default();
    for (name, port) in args.ports {
        if options.ports.insert(name, port).is_some() {
            return Err(invalid("duplicate telemetry listener selection"));
        }
    }
    if let Some(organization) = args.organization {
        options.organization = organization;
    }
    options.streams.extend(args.streams);
    if let Some(days) = args.retention_days {
        options.retention_days = days;
    }
    Ok(options)
}

pub(crate) fn run(ctx: &Context, args: TelemetryServiceArgs) -> Result<i32, HostError> {
    let environment = ctx.environment();
    let runtime = ctx.runtime()?;
    let result = match args.action {
        TelemetryAction::Install => telemetry_service::install(&ctx.checkout_root()?, environment)?,
        TelemetryAction::Setup(setup) => {
            let state = setup.state.state.clone();
            let options = setup_options(setup)?;
            runtime.block_on(telemetry_service::setup(
                &ctx.checkout_root()?,
                environment,
                state.as_deref(),
                &options,
            ))?
        }
        TelemetryAction::Inspect(state) => runtime.block_on(telemetry_service::inspect(
            environment,
            state.state.as_deref(),
        ))?,
        TelemetryAction::Start(state) => runtime.block_on(telemetry_service::start(
            environment,
            state.state.as_deref(),
        ))?,
        TelemetryAction::Stop(state) => {
            runtime.block_on(telemetry_service::stop(environment, state.state.as_deref()))?
        }
        TelemetryAction::Reconfigure(state) => runtime.block_on(telemetry_service::reconfigure(
            environment,
            state.state.as_deref(),
        ))?,
        TelemetryAction::RecoverConfiguration(state) => runtime.block_on(
            telemetry_service::recover_previous_configuration(environment, state.state.as_deref()),
        )?,
    };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &result).map_err(std::io::Error::other)?;
    writeln!(stdout)?;
    Ok(0)
}
