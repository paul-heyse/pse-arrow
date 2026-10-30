// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One fixed Rust helper owns source selection and native interpreters inside managed custody.
use super::{HostError, ManagedCommand, OutputOptions, checkout_root};
use crate::{
    generation::EntryGuard,
    managed::{payload::PayloadPolicy, workflow},
    test_workflow::execute::{Action, Bounds, Input},
};
use std::{path::Path, time::Duration};

#[derive(Debug, clap::Args)]
pub(super) struct Tests {
    #[command(flatten)]
    pub(super) selection: super::selection::TestArgs,
    #[command(flatten)]
    pub(super) run: RunOptions,
}

#[derive(Debug, clap::Args)]
pub(super) struct Check {
    #[command(flatten)]
    pub(super) selection: super::selection::SelectionArgs,
    #[command(flatten)]
    pub(super) run: RunOptions,
}

#[derive(Debug, clap::Args)]
pub(super) struct RunOptions {
    /// Ask native Cargo for timing diagnostics (shared mutable output, not retained evidence).
    #[arg(long)]
    timings: bool,
    /// Return the original managed launch identity and leave its owner running.
    #[arg(long)]
    detach: bool,
    /// Correlate this operation with an agent session without granting control authority.
    #[arg(long)]
    session: Option<String>,
}

pub(super) fn start(
    action: Action,
    request: crate::selection::Request,
    profile: String,
    run: RunOptions,
    selected: Option<&Path>,
    entry: &EntryGuard,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let control = entry.prepared().ok_or_else(|| {
        HostError::Ownership(
            "native test/check requires prepared host tools; run scripts/bootstrap-host".into(),
        )
    })?;
    let input = Input {
        version: 1,
        invocation: output.invocation_id.clone(),
        action,
        request,
        profile,
        timings: run.timings,
        policy: PayloadPolicy {
            startup: Duration::from_secs(30),
            drain: Duration::from_secs(30),
            write: Duration::from_secs(5),
        },
        bounds: Bounds {
            record_bytes: 1024 * 1024,
            grouping_capacity: 2048,
            failure_records: 20,
        },
    };
    let encoded = workflow::encode_input(&input)?;
    let command = workflow::command(
        &root,
        &input.invocation,
        control,
        std::env::vars_os().collect(),
    )?;
    let arguments = std::iter::once(command.program.into_os_string())
        .chain(command.arguments)
        .collect();
    super::launch_managed(
        ManagedCommand {
            parent: None,
            root,
            cwd: command.cwd,
            environment: command.environment,
            command: arguments,
            detach: run.detach,
            session: run.session,
            resource_class: None,
            verb: action.command(),
            tool: None,
            workflow: Some(encoded),
        },
        entry,
        output,
    )
}

#[cfg(test)]
mod tests {
    use clap::Parser as _;

    #[test]
    fn managed_run_options_preserve_selection_and_do_not_leak_into_explain() {
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "test",
            "pse-host",
            "--detach",
            "--session",
            "agent-session",
            "-E",
            "test(exact_case)",
        ])
        .unwrap();
        let Some(super::super::Command::Test(args)) = cli.command else {
            panic!("test command lost")
        };
        assert!(args.run.detach);
        assert_eq!(args.run.session.as_deref(), Some("agent-session"));
        let request = args.selection.request().unwrap();
        assert_eq!(request.filters.expressions, ["test(exact_case)"]);
        assert!(
            super::super::Cli::try_parse_from(["pse", "explain", "test", "pse-host", "--detach",])
                .is_err()
        );
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "check",
            "pse-ids",
            "--detach",
            "--session",
            "agent-session",
        ])
        .unwrap();
        let Some(super::super::Command::Check(args)) = cli.command else {
            panic!("check command lost")
        };
        assert!(args.run.detach);
        assert_eq!(args.run.session.as_deref(), Some("agent-session"));
        assert_eq!(
            args.selection
                .request(crate::selection::Operation::Check)
                .unwrap()
                .operation,
            crate::selection::Operation::Check
        );
    }
}
