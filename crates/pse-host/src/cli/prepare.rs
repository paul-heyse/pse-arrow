// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit host preparation uses the fixed bootstrap owner even when selected tools are stale.
use super::{HostError, checkout_root};
use crate::envelope::{
    CommandOutcome, Envelope, ErrorRecord, NextAction, OutputOptions, RedactionPolicy,
};
use std::{
    ffi::OsString,
    os::unix::process::ExitStatusExt as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, clap::Args)]
pub(super) struct PrepareArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, clap::Subcommand)]
enum Action {
    /// Prepare native tools, or copy a matching prepared source generation.
    Host {
        /// Checkout with an explicitly prepared, equivalent source generation.
        #[arg(long, value_name = "CHECKOUT")]
        from: Option<PathBuf>,
    },
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Route {
    VerifiedCurrent,
    MatchingSource,
    LocalSeed,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub(super) struct HostPreparation {
    route: Option<Route>,
}

pub(super) fn dispatch(
    args: PrepareArgs,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let verb = "prepare host";
    output.validate_for::<HostPreparation>(verb)?;
    let root = checkout_root(selected)?.canonicalize()?;
    let Action::Host { from } = args.action;
    let mut bootstrap = Command::new(root.join("scripts/bootstrap-host"));
    bootstrap.arg("--report-route");
    let from = from
        .map(|from| -> std::io::Result<_> {
            if from.is_absolute() {
                Ok(from)
            } else {
                Ok(std::env::current_dir()?.join(from))
            }
        })
        .transpose()?;
    if let Some(from) = &from {
        let mut argument = OsString::from("--from=");
        argument.push(from);
        bootstrap.arg(argument);
    }
    let result = bootstrap.stderr(Stdio::inherit()).output()?;
    let status = result.status;
    let code = status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(125);
    if status.success() {
        let route = match result.stdout.as_slice() {
            b"verified-current\n" => Route::VerifiedCurrent,
            b"matching-source\n" => Route::MatchingSource,
            b"local-seed\n" => Route::LocalSeed,
            _ => {
                return Err(HostError::Ownership(
                    "invalid host preparation receipt".into(),
                ));
            }
        };
        return Ok(output.emit(Envelope::response(
            verb,
            "native host preparation completed",
            HostPreparation { route: Some(route) },
        ))?);
    }
    let policy = RedactionPolicy::from_environment();
    let error =
        HostError::Ownership(format!("host preparation refused or failed with status {code}"));
    let mut response = Envelope::response(
        verb,
        "native host preparation did not complete",
        HostPreparation { route: None },
    );
    response.outcome = CommandOutcome::BoundaryFailure;
    response.child_exit = Some(code);
    response
        .failures
        .push(ErrorRecord::from_host(&error, &policy));
    let launcher = root.join("scripts/pse");
    let mut retry = vec![launcher.into_os_string(), "prepare".into(), "host".into()];
    if let Some(from) = from {
        let mut argument = OsString::from("--from=");
        argument.push(from);
        retry.push(argument);
    }
    if let Some(argv) = retry
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .ok()
                .filter(|value| policy.redact(value) == *value)
        })
        .collect::<Option<Vec<_>>>()
    {
        response.next.push(NextAction::Command {
            argv,
            reason: "retry preparation after resolving the original failure".into(),
        });
    }
    output.emit(response)?;
    Ok(code)
}
