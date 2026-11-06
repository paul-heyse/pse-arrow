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

#[derive(Debug, Default, clap::Args)]
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
    let environment = selected_environment(&root, &std::env::vars_os().collect())?;
    let command = workflow::command(&root, &input.invocation, control, environment)?;
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

// Workload launch resolves the same checkout selections as pse-env before admission.
// Passive reads and command discovery deliberately do not evaluate personal shell code.
fn selected_environment(
    root: &Path,
    caller: &crate::environment::Environment,
) -> Result<crate::environment::Environment, HostError> {
    crate::admission::validate_request(caller)?;
    Ok(crate::environment::activate(root, caller, &crate::environment::Environment::new())?.values)
}

#[cfg(test)]
mod tests {
    use clap::Parser as _;

    fn checkout() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        for path in [
            ".python-version",
            ".cargo/config.toml",
            ".config/build.toml",
            ".config/diagnostic-tools.toml",
            ".config/sccache.toml",
            ".config/agent-capacity.toml",
        ] {
            let target = root.path().join(path);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(source.join(path), target).unwrap();
        }
        std::fs::write(
            root.path().join(".envrc.local"),
            "export PSE_BUILD_MODE=unbounded\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn workflow_launch_activates_local_unbounded_mode_before_admission() {
        let root = checkout();
        let caller = [("HOME".into(), root.path().as_os_str().to_owned())].into();
        let environment = super::selected_environment(root.path(), &caller).unwrap();
        let selection =
            crate::admission::Selection::load(root.path(), &environment, None, None).unwrap();
        assert!(selection.profile().ordinary());
        let controls = selection.profile().controls().unwrap();
        assert!(controls.memory_max.is_none());
        assert!(controls.cpu_quota_per_sec_usec.is_none());
        assert!(controls.allowed_cpus.is_empty());
        assert_eq!(selection.profile().slots, 0);
        assert_eq!(
            environment
                .get(std::ffi::OsStr::new("CARGO_BUILD_JOBS"))
                .unwrap(),
            "32"
        );
    }

    #[test]
    fn workflow_activation_preserves_caller_bounded_and_explicit_finite_selections() {
        let root = checkout();
        for (key, value, name, memory) in [
            ("PSE_BUILD_MODE", "bounded", "functional", 40_u64 << 30),
            ("PSE_RESOURCE_CLASS", "compile", "compile", 24_u64 << 30),
            ("PSE_MEMORY_MAX", "64G", "wide", 64_u64 << 30),
        ] {
            let caller = [
                ("HOME".into(), root.path().as_os_str().to_owned()),
                (key.into(), value.into()),
            ]
            .into();
            let environment = super::selected_environment(root.path(), &caller).unwrap();
            let selection =
                crate::admission::Selection::load(root.path(), &environment, None, None).unwrap();
            assert_eq!(selection.profile().name, name);
            assert_eq!(selection.profile().memory, memory);
            assert_eq!(
                selection.profile().controls().unwrap().memory_max,
                Some(memory)
            );
            assert!(selection.profile().aggregate >= selection.profile().allocation);
        }
    }

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
            super::super::Cli::try_parse_from([
                "pse",
                "inspect",
                "test-selection",
                "pse-host",
                "--detach",
            ])
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
