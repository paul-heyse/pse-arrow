// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `activity [UNIT]`: one JSON observation; exit 125 when it is partial.
use super::Context;
use crate::{HostError, activity::ActivityObservation};
use clap::Args;
use std::io::Write as _;

/// Unit selection.
#[derive(Debug, Args)]
pub(crate) struct ActivityArgs {
    /// A `.service` or `.scope` unit (absent: this process's unit).
    #[arg(value_name = "UNIT")]
    pub(crate) unit: Option<String>,
}

pub(crate) fn run(ctx: &Context, args: ActivityArgs) -> Result<i32, HostError> {
    ctx.runtime()?.block_on(async {
        let observation = ActivityObservation::read(args.unit.as_deref()).await;
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, &observation).map_err(std::io::Error::other)?;
        writeln!(stdout)?;
        Ok(if observation.complete() { 0 } else { 125 })
    })
}
