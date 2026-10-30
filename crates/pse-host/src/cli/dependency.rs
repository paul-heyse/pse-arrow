// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected dependency units reuse the same C07 intent and admission owner as execution.
use super::{HostError, OutputOptions};
use clap::Subcommand;
use schemars::JsonSchema;
use serde::Serialize;
use std::{ffi::OsString, path::Path};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explain dependency instances in actual admitted check contexts without compiling.
    Units {
        /// Exact locked dependency, distinct from the following selected workspace roots.
        #[arg(id = "dependency")]
        dependency: String,
        #[command(flatten)]
        selection: super::selection::SelectionArgs,
    },
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct UnitContext {
    /// Original selected/admitted compiler arguments, including feature absence policy outcomes.
    arguments: Vec<OsString>,
    /// Exact roots sharing this compatible context; incompatible roots remain separate.
    roots: Vec<crate::correctness::CompilerUnit>,
    /// Only source-qualified instances of the selected dependency; absence is not test coverage.
    dependency_units: Vec<crate::correctness::CompilerUnit>,
    /// Every unit examined in this original admitted context.
    examined_units: usize,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Units {
    /// Original exact metadata dependency facts.
    dependency: crate::dependency::Report,
    /// Original normalized C07 selection, preserving default/negative feature policies.
    selection: Option<crate::selection::Plan>,
    /// Original native compiler-unit contexts; none has executed.
    contexts: Option<Vec<UnitContext>>,
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let Action::Units {
        dependency,
        selection,
    } = action;
    let root = super::checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    let snapshot = crate::dependency::capture(&root, &environment)?;
    let facts = crate::dependency::inspect(&snapshot, &dependency, "locked_offline_metadata");
    let package = match &facts.payload.selected {
        crate::dependency::Selection::Exact { facts } => facts.package.id.clone(),
        _ => {
            let mut response = facts.map_payload(|dependency| Units {
                dependency,
                selection: None,
                contexts: None,
            });
            response.command = "dep units".into();
            return Ok(output.emit(response)?);
        }
    };
    let admitted = crate::test_workflow::CheckIntent::new(
        snapshot.metadata,
        selection.request(crate::selection::Operation::Check)?,
        environment,
    )
    .and_then(crate::test_workflow::CheckIntent::admit)
    .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let contexts = admitted
        .resolved()
        .admitted()
        .iter()
        .map(|context| UnitContext {
            arguments: context.arguments.clone(),
            roots: context
                .units
                .iter()
                .filter(|unit| unit.root)
                .cloned()
                .collect(),
            dependency_units: context
                .units
                .iter()
                .filter(|unit| unit.package_id == package)
                .cloned()
                .collect(),
            examined_units: context.units.len(),
        })
        .collect();
    let mut response = crate::envelope::Envelope::response(
        "dep units",
        "selected dependency compiler contexts; no build or test listing performed",
        Units {
            dependency: facts.payload,
            selection: Some(admitted.resolved().plan().clone()),
            contexts: Some(contexts),
        },
    );
    response.observation.scope = "original_shared_check_intent_and_correctness_admission".into();
    response.observation.limitations = vec![
        "Cargo unit graphs describe planned compiler actions, not captured artifacts, actual compilation, runnable tests or behavioral impact".into(),
        "only selected check-context dependency instances are returned; original normalized roots/defaults/negative features and host/target identities remain separate".into(),
        "native provider readiness, fixture preparation and original physical settlement are not granted by this read".into(),
    ];
    Ok(output.emit(response)?)
}
