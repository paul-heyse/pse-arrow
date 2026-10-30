// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Argument structs shared by every verb; one definition each, flattened where used.
use clap::{ArgAction, Args, ValueEnum};
use std::path::PathBuf;

/// Requested output rendering. The A03 envelope gives these their effect.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Human output on a terminal, a compact summary otherwise.
    #[default]
    Auto,
    /// Human-oriented text.
    Human,
    /// One versioned JSON envelope.
    Json,
    /// A newline-delimited JSON event stream ending with a summary record.
    Ndjson,
}

/// Global options, accepted before or after any verb (but never inside pass-through argv).
#[derive(Debug, Clone, Default, Args)]
pub struct GlobalArgs {
    /// Use the checkout containing DIR instead of locating it from the host executable or
    /// the current directory.
    #[arg(short = 'C', long = "checkout", global = true, value_name = "DIR")]
    pub checkout: Option<PathBuf>,
    /// Output format.
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = Format::Auto,
        env = "PSE_FORMAT"
    )]
    pub format: Format,
    /// Cap the rendered summary at N lines; the rest is replaced by counts and paths.
    #[arg(long, global = true, value_name = "N", env = "PSE_BUDGET", value_parser = clap::value_parser!(u64).range(1..))]
    pub budget: Option<u64>,
    /// More diagnostic output; repeatable.
    #[arg(short = 'v', long = "verbose", global = true, action = ArgAction::Count, conflicts_with = "quiet")]
    pub verbose: u8,
    /// Less diagnostic output; repeatable.
    #[arg(short = 'q', long = "quiet", global = true, action = ArgAction::Count)]
    pub quiet: u8,
}

/// Envelope projection, flattened into verbs whose output is an envelope.
#[derive(Debug, Clone, Default, Args)]
pub struct OutputArgs {
    /// Print only these envelope fields (absent: the default projection).
    #[arg(long, value_delimiter = ',', action = ArgAction::Append, value_name = "FIELD")]
    pub fields: Option<Vec<String>>,
}
