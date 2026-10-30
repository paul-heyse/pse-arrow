// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `pse-env` grammar (`pse-host boundary`), the exec adapter's contract.
//!
//! The first non-option word starts the command and `--` ends options; everything after
//! is data. The command positional is `trailing_var_arg` without `allow_hyphen_values`, so
//! an unknown option before the command is refused rather than run as the command.
use super::Context;
use crate::{
    HostError, admission,
    boundary::{self, Launch},
    environment::{self, CacheMode, EnvironmentRead},
    native,
};
use clap::Args;
use std::{ffi::OsString, io::Write as _};

fn resource_class(value: &str) -> Result<String, HostError> {
    admission::validate_class(value)?;
    Ok(value.to_owned())
}

/// Boundary options; repeated options take their last value.
#[derive(Debug, Args)]
#[command(
    args_override_self = true,
    arg_required_else_help = true,
    override_usage = "pse-env [--docs] [--cache auto|on|off] [--native[=CAPABILITIES]] [--resource-class CLASS] -- command [args...]\n       pse-env --print | --explain [--json]",
    help_template = "{usage}\n\n{about-with-newline}\n{all-args}{after-help}",
    after_help = "Native providers require explicit preparation (scripts/bootstrap-host, native prepare); store setup remains unavailable."
)]
pub(crate) struct BoundaryArgs {
    /// Print the shell activation program.
    #[arg(long, conflicts_with_all = ["explain", "native", "no_scope", "resource_class", "recipe_default_class", "command"])]
    pub(crate) print: bool,
    /// Explain the selected environment without evaluating personal code.
    #[arg(long, conflicts_with_all = ["no_scope", "resource_class", "recipe_default_class", "command"])]
    pub(crate) explain: bool,
    /// Explanation output is JSON.
    #[arg(long, requires = "explain", conflicts_with_all = ["print", "command"])]
    pub(crate) json: bool,
    /// Select the documentation environment.
    #[arg(long, conflicts_with = "native")]
    pub(crate) docs: bool,
    /// Compiler-cache intent.
    #[arg(long, value_enum, default_value_t = CacheMode::Auto)]
    pub(crate) cache: CacheMode,
    /// Select native providers: bare (or `auto`) uses `$PSE_NATIVE_CAPABILITIES` or the full
    /// profile; `--native=` selects none.
    #[arg(
        long,
        value_name = "CAPABILITIES",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "auto",
        value_parser = native::Request::parse
    )]
    pub(crate) native: Option<native::Request>,
    /// Require authenticated enclosing containment instead of creating a scope.
    #[arg(long)]
    pub(crate) no_scope: bool,
    /// Explicit resource class.
    #[arg(long, value_name = "CLASS", value_parser = resource_class)]
    pub(crate) resource_class: Option<String>,
    /// Recipe default resource class, below caller and explicit selections.
    #[arg(long, value_name = "CLASS", value_parser = resource_class)]
    pub(crate) recipe_default_class: Option<String>,
    /// Store setup (refused).
    #[arg(long, hide = true)]
    pub(crate) store: bool,
    /// Allocation binding (refused).
    #[arg(long, hide = true)]
    pub(crate) bind_allocation: bool,
    /// The command and its arguments, passed through unchanged.
    #[arg(
        trailing_var_arg = true,
        num_args = 1..,
        value_name = "COMMAND",
        required_unless_present_any = ["print", "explain"],
        value_parser = clap::value_parser!(OsString)
    )]
    pub(crate) command: Vec<OsString>,
}

pub(crate) fn run(ctx: &Context, args: BoundaryArgs) -> Result<i32, HostError> {
    if args.store {
        return Err(boundary::unsupported("--store"));
    }
    if args.bind_allocation {
        return Err(boundary::unsupported("--bind-allocation"));
    }
    if args.explain
        && let Some(request) = &args.native
    {
        let explanation = boundary::explain_native(&request.resolve(ctx.environment())?);
        writeln!(std::io::stdout(), "{explanation}")?;
        return Ok(0);
    }
    if args.print || args.explain {
        let operation = if args.print {
            EnvironmentRead::Print
        } else {
            EnvironmentRead::Explain
        };
        let output = environment::read(
            &ctx.checkout_root()?,
            ctx.environment(),
            operation,
            args.docs,
            args.cache,
        )?;
        std::io::stdout().lock().write_all(&output)?;
        return Ok(0);
    }
    boundary::launch(
        &ctx.checkout_root()?,
        ctx.environment(),
        Launch {
            command: args.command,
            docs: args.docs,
            cache: args.cache,
            native: args.native,
            require_enclosing: args.no_scope,
            class: args.resource_class,
            recipe_class: args.recipe_default_class,
        },
    )
}
