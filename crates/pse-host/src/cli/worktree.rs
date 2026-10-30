// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit worktree mutations and passive native inventory through the original owners.
use super::{GenerationIntent, HostError, OutputOptions, checkout_root};
use crate::worktree::{self, NewRequest};
use clap::Subcommand;
use std::{ffi::OsString, path::Path};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Create a sibling checkout, copy local configuration and explicitly prepare native tools.
    New {
        /// One sibling directory component; no existing path is overwritten.
        name: OsString,
        #[arg(long, default_value = "HEAD")]
        base: OsString,
        /// New branch name; default matches the native checkout name.
        #[arg(long)]
        branch: Option<OsString>,
        /// Submit the existing detached workspace check after preparation succeeds.
        #[arg(long)]
        warm: bool,
    },
    /// Observe main/linked worktrees, direct ancestry, dirty paths and original native users.
    List {
        /// Direct ancestry comparison base, resolved before walking worktree commits.
        #[arg(long, default_value = "HEAD")]
        base: OsString,
    },
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::New { .. } => GenerationIntent::NewWork,
            Self::List { .. } => GenerationIntent::Passive,
        }
    }
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    match action {
        Action::New {
            name,
            base,
            branch,
            warm,
        } => {
            let response = worktree::create(
                &root,
                NewRequest {
                    name,
                    base,
                    branch,
                    warm,
                },
                &environment,
            )?;
            Ok(output.emit(response)?)
        }
        Action::List { base } => {
            let response =
                super::runtime()?.block_on(worktree::list(&root, &base, &environment))?;
            Ok(output.emit(response)?)
        }
    }
}
