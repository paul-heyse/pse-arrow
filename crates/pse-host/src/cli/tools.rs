// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Forward exact captured native parser inputs through the one managed invocation owner.
use super::{
    GenerationIntent, HostError, ManagedCommand, NativeToolSelection, OutputOptions, checkout_root,
};
use crate::{
    envelope::Envelope,
    generation::EntryGuard,
    prepared_xtask::{self, NativeToolTarget, PreparedNativeTool},
};
use clap::Subcommand;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Preview or release one completed checkout-local host tool generation.
    RetireHost {
        generation: String,
        /// Exact release token returned by the current generation preview.
        #[arg(long)]
        release: Option<String>,
    },
    /// Preview or release one settled compact developer result.
    RetireResult {
        #[arg(value_parser = super::parse_invocation_id)]
        invocation: crate::envelope::CliInvocationId,
        /// Select the original campaign checkpoint instead of an ordinary test result.
        #[arg(long)]
        conformance: bool,
        /// Exact token returned by the current owner preview.
        #[arg(long)]
        release: Option<String>,
    },
    /// Export a compact result or one committed workflow file to a fresh destination.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    ExportResult {
        /// An exact invocation ID, unique prefix, or session-scoped latest.
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        /// Include this bounded detail from the original owner.
        #[arg(long, value_enum)]
        detail: Option<crate::invocation_view::ResultDetail>,
        /// Export this exact ordinal from the original workflow-files view.
        #[arg(long, conflicts_with = "detail")]
        workflow_file: Option<usize>,
        /// Fresh destination file, relative to the selected checkout if not absolute.
        #[arg(long)]
        output: PathBuf,
    },
    /// Set a named compact-result hold or clear it by omitting the condition.
    HoldResult {
        #[arg(value_parser = super::parse_invocation_id)]
        invocation: crate::envelope::CliInvocationId,
        /// Select the original campaign checkpoint instead of an ordinary test result.
        #[arg(long)]
        conformance: bool,
        #[arg(long)]
        consumer: String,
        #[arg(long)]
        condition: Option<String>,
    },
    /// Sweep eligible compact results under the selected checkout's owner guards.
    SweepResults {
        /// Sweep the authenticated registered worktrees of this repository.
        #[arg(long)]
        registered: bool,
    },
    /// Prepare the fixed checkout-scoped compact-result expiry timer.
    PrepareResultExpiry {
        /// Also enable the prepared user timer.
        #[arg(long)]
        enable: bool,
    },
    /// Build/capture the declared native tool; passive queries never invoke this operation.
    Prepare {
        #[arg(value_enum)]
        tool: NativeToolTarget,
    },
    /// Read the selected native parser's retained catalogue, with captured-source limits.
    Catalog {
        #[arg(long, value_enum, default_value = "xtask")]
        tool: NativeToolTarget,
        #[arg(long)]
        selection: String,
    },
    /// Read the actual retained native document-owner response schema.
    OwnerSchema {
        #[arg(long, value_enum, default_value = "xtask")]
        tool: NativeToolTarget,
        #[arg(long)]
        selection: String,
    },
    /// Run exact arguments through the selected native parser under managed ownership.
    Run {
        #[arg(long, value_enum, default_value = "xtask")]
        tool: NativeToolTarget,
        #[arg(long)]
        selection: String,
        #[arg(long)]
        detach: bool,
        #[arg(long)]
        session: Option<String>,
        #[arg(last=true, required=true, num_args=1..)]
        arguments: Vec<OsString>,
    },
    /// Read the original fixed semantic protocol schemas from a captured helper.
    Protocol {
        #[arg(long, value_enum, default_value = "native-analysis")]
        tool: NativeToolTarget,
        #[arg(long)]
        selection: String,
    },
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Prepare { .. } | Self::Run { .. } => GenerationIntent::NewWork,
            Self::RetireHost {
                release: Some(_), ..
            }
            | Self::RetireResult {
                release: Some(_), ..
            }
            | Self::ExportResult { .. }
            | Self::HoldResult { .. }
            | Self::SweepResults { .. }
            | Self::PrepareResultExpiry { .. } => GenerationIntent::ExistingControl,
            _ => GenerationIntent::Passive,
        }
    }
    pub(super) fn raw_protocol(&self) -> bool {
        matches!(self, Self::Run { .. })
    }
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct PreparedTool {
    tool: &'static str,
    selection: String,
    source_identity: String,
    executable_sha256: String,
    features: BTreeSet<String>,
}
fn tool_error(error: prepared_xtask::PreparedNativeToolError) -> HostError {
    HostError::PreparedNativeTool(Box::new(error))
}
/// Transport/run controls shared by exact native-owner forwarding adapters.
#[derive(Debug)]
pub(super) struct Run {
    pub tool: NativeToolTarget,
    pub selection: String,
    pub detach: bool,
    pub session: Option<String>,
    pub arguments: Vec<OsString>,
    pub verb: &'static str,
}
pub(super) fn run(
    args: Run,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    if args.tool == NativeToolTarget::CompilerAnalysis {
        return Err(HostError::Ownership("compiler-analysis is a controlled compiler stage; use compiler generate --kind facts with an exact --body selection".into()));
    }
    if args.tool.is_conformance() {
        return Err(HostError::Ownership("use pse conformance run for a prepared campaign image".into()));
    }
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    let tool = PreparedNativeTool::open_for(&root, &environment, &args.selection, args.tool)
        .map_err(tool_error)?;
    let evidence = NativeToolSelection {
        kind: tool.target().name(),
        selection: tool.id().into(),
        source_identity: tool.source_identity().map_err(tool_error)?,
        executable_sha256: tool.executable().sha256.clone(),
    };
    let payload = tool
        .launch(args.arguments, environment)
        .map_err(tool_error)?;
    let command = std::iter::once(payload.program.into_os_string())
        .chain(payload.arguments)
        .collect();
    super::launch_managed(
        ManagedCommand {
            parent: None,
            root,
            cwd: payload.cwd,
            environment: payload.environment,
            command,
            detach: args.detach,
            session: args.session,
            resource_class: None,
            verb: args.verb,
            tool: Some(evidence),
            workflow: None,
        },
        entry,
        output,
    )
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    match action {
        Action::RetireHost {
            generation,
            release,
        } => {
            let report = crate::generation::retirement::retire(
                &root,
                &environment,
                &generation,
                release.as_deref(),
            )?;
            let refused = report.refused();
            let mut envelope = Envelope::response(
                "tools retire-host",
                "checkout-local host generation retirement",
                report,
            );
            if refused {
                envelope.outcome = crate::envelope::CommandOutcome::Failure;
            }
            Ok(output.emit(envelope)?)
        }
        Action::RetireResult {
            invocation,
            release,
            conformance,
        } => {
            let store = super::invocation_store(selected)?;
            if conformance {
                let report = store.retire_conformance(&invocation, release.as_deref())?;
                return Ok(output.emit(Envelope::response(
                    "tools retire-result conformance",
                    "original campaign evidence retirement",
                    report,
                ))?);
            }
            let report = store.retire_compact_result(&invocation, release.as_deref())?;
            Ok(output.emit(Envelope::response(
                "tools retire-result",
                "checkout-local compact result retirement",
                report,
            ))?)
        }
        Action::ExportResult {
            invocation,
            session,
            detail,
            workflow_file,
            output: destination,
        } => {
            output.validate_for::<crate::invocation_view::ResultExport>("tools export-result")?;
            let session = crate::invocation::select_session(session, &environment)?;
            let store = super::invocation_store(selected)?;
            if let Some(ordinal) = workflow_file {
                let receipt = crate::invocation_view::export_workflow_file(
                    &store, &invocation, session.as_deref(), ordinal, &root.join(destination),
                    &crate::envelope::RedactionPolicy::from_inputs(&environment), &output.invocation_id,
                )?;
                return Ok(output.emit_observation(receipt)?);
            }
            let receipt = crate::invocation_view::export(
                &store,
                &invocation,
                session.as_deref(),
                detail,
                &root.join(destination),
                &crate::envelope::RedactionPolicy::from_inputs(&environment),
                &output.invocation_id,
            )?;
            Ok(output.emit_observation(receipt)?)
        }
        Action::HoldResult {
            invocation,
            consumer,
            condition,
            conformance,
        } => {
            let store = super::invocation_store(selected)?;
            let policy = crate::envelope::RedactionPolicy::from_environment();
            if policy.redact(&consumer) != consumer
                || condition
                    .as_ref()
                    .is_some_and(|value| policy.redact(value) != *value)
            {
                return Err(HostError::Ownership(
                    "compact result hold contains a protected value".into(),
                ));
            }
            if conformance {
                store.hold_conformance(&invocation, &consumer, condition.as_deref())?;
            } else {
                store.hold_compact_result(&invocation, &consumer, condition.as_deref())?;
            }
            Ok(output.emit(Envelope::response(
                "tools hold-result",
                "compact result hold updated",
                (),
            ))?)
        }
        Action::SweepResults { registered } => {
            let report = if registered {
                crate::invocation::sweep_registered_compact_results(&root, &environment)?
            } else {
                super::invocation_store(selected)?.sweep_compact_results()?
            };
            Ok(output.emit(Envelope::response(
                "tools sweep-results",
                "compact result sweep",
                report,
            ))?)
        }
        Action::PrepareResultExpiry { enable } => {
            let report = crate::invocation::prepare_compact_timer(&root, &environment, enable)?;
            Ok(output.emit(Envelope::response(
                "tools prepare-result-expiry",
                "compact result expiry timer preparation",
                report,
            ))?)
        }
        Action::Prepare { tool: target } => {
            if target.is_conformance() {
                return Err(HostError::Ownership("use pse conformance prepare for a campaign image".into()));
            }
            let tool =
                prepared_xtask::prepare_for(&root, &environment, target).map_err(tool_error)?;
            Ok(output.emit(Envelope::response(
                "tools prepare",
                "native development tool explicitly captured",
                PreparedTool {
                    tool: tool.target().name(),
                    selection: tool.id().into(),
                    source_identity: tool.source_identity().map_err(tool_error)?,
                    executable_sha256: tool.executable().sha256.clone(),
                    features: tool.features().clone(),
                },
            ))?)
        }
        Action::Catalog {
            selection,
            tool: target,
        } => {
            let tool = PreparedNativeTool::open_for(&root, &environment, &selection, target)
                .map_err(tool_error)?;
            Ok(output.emit(super::catalog_envelope(
                tool.catalog().map_err(tool_error)?,
                "tools catalog",
                format!("captured {} selection {}", tool.target(), tool.id()),
            ))?)
        }
        Action::OwnerSchema {
            selection,
            tool: target,
        } => {
            let tool = PreparedNativeTool::open_for(&root, &environment, &selection, target)
                .map_err(tool_error)?;
            serde_json::to_writer(
                io::stdout().lock(),
                &tool.owner_schema().map_err(tool_error)?,
            )
            .map_err(io::Error::other)?;
            writeln!(io::stdout().lock())?;
            Ok(0)
        }
        Action::Run {
            tool,
            selection,
            detach,
            session,
            arguments,
        } => run(
            Run {
                tool,
                selection,
                detach,
                session,
                arguments,
                verb: "tools run",
            },
            selected,
            output,
            entry,
        ),
        Action::Protocol {
            selection,
            tool: target,
        } => {
            let tool = PreparedNativeTool::open_for(&root, &environment, &selection, target)
                .map_err(tool_error)?;
            serde_json::to_writer(io::stdout().lock(), &tool.protocol().map_err(tool_error)?)
                .map_err(io::Error::other)?;
            writeln!(io::stdout().lock())?;
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn native_analysis_preparation_is_explicit_and_protocol_discovery_is_passive() {
        let parsed =
            super::super::Cli::try_parse_from(["pse", "tools", "prepare", "native-analysis"])
                .unwrap();
        assert!(matches!(
            parsed.command,
            Some(super::super::Command::Tools {
                action: Action::Prepare {
                    tool: NativeToolTarget::NativeAnalysis
                }
            })
        ));
        let parsed = super::super::Cli::try_parse_from([
            "pse",
            "tools",
            "protocol",
            "--selection",
            "original",
        ])
        .unwrap();
        assert_eq!(
            parsed.command.unwrap().generation_intent(),
            GenerationIntent::Passive
        );
    }
    #[test]
    fn tool_parser_tail_stays_literal_and_discovery_remains_passive() {
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "tools",
            "run",
            "--selection",
            "selection",
            "--",
            "owner",
            "--format",
            "human",
        ])
        .unwrap();
        let Some(super::super::Command::Tools {
            action: Action::Run { arguments, .. },
        }) = cli.command
        else {
            panic!("native forwarding selection")
        };
        assert_eq!(arguments, ["owner", "--format", "human"]);
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "tools",
            "catalog",
            "--selection",
            "selection",
        ])
        .unwrap();
        assert_eq!(
            cli.command.unwrap().generation_intent(),
            GenerationIntent::Passive
        );
    }
}
