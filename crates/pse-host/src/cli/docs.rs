// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact native docs forwarding; the selected xtask owns its grammar and diagnostics.
use super::{GenerationIntent, HostError, OutputOptions, checkout_root, tools};
use crate::{
    generation::EntryGuard,
    prepared_xtask::{self, PreparedXtask},
};
use clap::Args as ClapArgs;
use std::{
    ffi::OsString,
    io::{self, Write},
    path::Path,
};

#[derive(Debug, ClapArgs)]
pub(super) struct Args {
    /// Explicit retained native selection; absence uses the last successful explicit prepare.
    #[arg(long)]
    pub selection: Option<String>,
    /// Exact native docs arguments, forwarded without a host grammar or diagnostic decoder.
    #[arg(trailing_var_arg=true,allow_hyphen_values=true,num_args=1..)]
    pub arguments: Vec<OsString>,
}
impl Args {
    pub(super) fn intent(&self) -> GenerationIntent {
        // These explicit native read flags never prepare a source tool or refresh the host.
        if prepared_xtask::passive_read(&self.arguments) {
            GenerationIntent::Passive
        } else {
            GenerationIntent::NewWork
        }
    }
}
pub(super) fn dispatch(
    args: Args,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let passive = args.intent() == GenerationIntent::Passive;
    let selection = match args.selection {
        Some(selection) => selection,
        None => PreparedXtask::current(&checkout_root(selected)?, &std::env::vars_os().collect())
            .map_err(|error| HostError::PreparedXtask(Box::new(error)))?
            .id()
            .to_owned(),
    };
    let arguments = std::iter::once(OsString::from("docs"))
        .chain(args.arguments)
        .collect();
    if passive {
        let root = checkout_root(selected)?;
        let environment = std::env::vars_os().collect();
        let tool = PreparedXtask::open(&root, &environment, &selection)
            .map_err(|error| HostError::PreparedXtask(Box::new(error)))?;
        let reply = tool
            .read_passive_docs(arguments, &environment)
            .map_err(|error| HostError::PreparedXtask(Box::new(error)))?;
        io::stdout().lock().write_all(&reply.stdout)?;
        io::stderr().lock().write_all(&reply.stderr)?;
        return Ok(reply.status);
    }
    tools::run(
        tools::Run {
            selection,
            detach: false,
            session: None,
            arguments,
            verb: "docs",
        },
        selected,
        output,
        entry,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_native_read_flags_are_passive_only_before_literal_arguments() {
        let args = |tail: &[&str]| Args {
            selection: None,
            arguments: tail.iter().map(OsString::from).collect(),
        };
        assert_eq!(
            args(&["check", "--schema"]).intent(),
            GenerationIntent::Passive
        );
        assert_eq!(
            args(&["check", "--help"]).intent(),
            GenerationIntent::Passive
        );
        assert_eq!(
            args(&["check", "--", "--schema"]).intent(),
            GenerationIntent::NewWork
        );
        assert_eq!(
            args(&["check", "docs/README.md"]).intent(),
            GenerationIntent::NewWork
        );
    }
}
