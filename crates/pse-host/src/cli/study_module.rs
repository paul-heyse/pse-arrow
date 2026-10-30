// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `study-module prepare|select`: prints the receipt as one JSON line. Preparation uses the
//! already selected native server CLI; selection never builds or hashes source. No database
//! is started or changed.
use super::Context;
use crate::{
    HostError,
    study_module::{self, Variant},
};
use clap::{Args, Subcommand};
use std::{io::Write as _, path::PathBuf};

/// Study-module action.
#[derive(Debug, Args)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub(crate) struct StudyModuleArgs {
    /// The action.
    #[command(subcommand)]
    pub(crate) action: StudyAction,
}

/// Module variant; production unless qualification is selected.
#[derive(Debug, Args)]
pub(crate) struct VariantArgs {
    /// The qualification module (rollback, timeout, memory and trap controls).
    #[arg(long)]
    pub(crate) qualification: bool,
    /// The long-hold qualification module, built from an owned source snapshot.
    #[arg(long, requires = "qualification")]
    pub(crate) long_hold: bool,
}
impl VariantArgs {
    fn variant(&self) -> Variant {
        if self.long_hold {
            Variant::LongHold
        } else if self.qualification {
            Variant::Qualification
        } else {
            Variant::Production
        }
    }
}

/// Study-module actions.
#[derive(Debug, Subcommand)]
pub(crate) enum StudyAction {
    /// Build the module with the selected server CLI.
    Prepare {
        /// Surreal tool root (default: `PSE_SURREAL_TOOLS` or the user data directory).
        #[arg(long, value_name = "PATH")]
        tool_root: Option<PathBuf>,
        #[command(flatten)]
        variant: VariantArgs,
    },
    /// Select a prepared module by its exact receipt.
    Select {
        /// The module receipt.
        #[arg(long, required = true, value_name = "PATH")]
        receipt: PathBuf,
        #[command(flatten)]
        variant: VariantArgs,
    },
}

pub(crate) fn run(ctx: &Context, args: StudyModuleArgs) -> Result<i32, HostError> {
    let receipt = match args.action {
        StudyAction::Select { receipt, variant } => {
            study_module::select(&receipt, variant.variant())?
        }
        StudyAction::Prepare { tool_root, variant } => {
            let root = ctx.checkout_root()?;
            let environment = ctx.environment();
            let tools = match tool_root {
                Some(tools) => tools,
                None => study_module::tool_root(environment)?,
            };
            ctx.runtime()?.block_on(study_module::prepare(
                &root,
                environment,
                &tools,
                variant.variant(),
            ))?
        }
    };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &receipt)
        .map_err(|error| HostError::Ownership(error.to_string()))?;
    writeln!(stdout)?;
    Ok(0)
}
