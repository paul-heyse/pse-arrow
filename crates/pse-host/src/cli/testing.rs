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

pub(super) fn start(
    action: Action,
    request: crate::selection::Request,
    profile: String,
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
            root,
            cwd: command.cwd,
            environment: command.environment,
            command: arguments,
            detach: false,
            session: None,
            resource_class: None,
            verb: action.command(),
            tool: None,
            workflow: Some(encoded),
        },
        entry,
        output,
    )
}
