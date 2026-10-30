// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `run -- <argv…>`: exec adapter; exits with the child's status or 128+signal.
use super::Context;
use crate::{
    HostError,
    foreground::{self, ForegroundRequest},
};
use clap::Args;
use std::{
    ffi::OsString,
    time::{Duration, Instant},
};

/// Pass-through command.
#[derive(Debug, Args)]
pub(crate) struct RunArgs {
    /// Absolute executable and its arguments, after `--`, passed through unchanged.
    #[arg(
        last = true,
        required = true,
        num_args = 1..,
        value_name = "ARGV",
        value_parser = clap::value_parser!(OsString)
    )]
    pub(crate) command: Vec<OsString>,
}

pub(crate) fn run(ctx: &Context, args: RunArgs) -> Result<i32, HostError> {
    ctx.runtime()?.block_on(foreground::run(ForegroundRequest {
        command: args.command,
        environment: ctx.environment().clone(),
        cwd: std::env::current_dir()?,
        require_enclosing: false,
        startup_deadline: Instant::now() + Duration::from_secs(30),
        drain_timeout: Duration::from_secs(30),
    }))
}
