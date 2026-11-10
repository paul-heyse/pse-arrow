// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admit a prepared campaign image and forward to its native campaign owner.
use super::{
    GenerationIntent, HostError, ManagedCommand, NativeToolSelection, OutputOptions, checkout_root,
};
use crate::{
    envelope::{Envelope, SubjectState},
    generation::EntryGuard,
    prepared_xtask::{self, NativeToolTarget, PreparedNativeTool, PreparedNativeToolError},
};
use clap::{Args, Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(super) enum Provider {
    #[default]
    None,
    #[value(name = "coolprop8-water")]
    CoolpropWater,
}
impl Provider {
    fn argument(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CoolpropWater => "coolprop8-water",
        }
    }
    fn target(self) -> NativeToolTarget {
        match self {
            Self::None => NativeToolTarget::Conformance,
            Self::CoolpropWater => NativeToolTarget::ConformanceCoolprop,
        }
    }
    fn environment(
        self,
        root: &Path,
        inherited: &crate::environment::Environment,
    ) -> Result<crate::environment::Environment, HostError> {
        let capabilities = match self {
            Self::None => "solver,klu,isolation,uno,petsc",
            Self::CoolpropWater => "solver,klu,isolation,uno,petsc,coolprop",
        };
        crate::native::select_capabilities(root, inherited, capabilities)
    }
}

#[derive(Debug, Args)]
pub(super) struct CampaignArgs {
    #[arg(long)]
    manifest: PathBuf,
    #[arg(long)]
    run: Option<String>,
    #[arg(long = "fixture", requires = "run")]
    fixtures: Vec<String>,
    #[arg(long, value_enum, default_value = "none")]
    provider: Provider,
    #[arg(long)]
    selection: Option<String>,
}
impl CampaignArgs {
    fn native_arguments(&self) -> Vec<OsString> {
        let mut result = vec![
            "conformance".into(),
            "--manifest".into(),
            self.manifest.as_os_str().into(),
            "--provider".into(),
            self.provider.argument().into(),
        ];
        if let Some(run) = &self.run {
            result.extend([OsString::from("--run"), run.as_str().into()]);
        }
        for fixture in &self.fixtures {
            result.extend([OsString::from("--fixture"), fixture.as_str().into()]);
        }
        result
    }
    fn open(
        &self,
        root: &Path,
        environment: &crate::environment::Environment,
    ) -> Result<PreparedNativeTool, PreparedNativeToolError> {
        match &self.selection {
            Some(selection) => {
                PreparedNativeTool::open_for(root, environment, selection, self.provider.target())
            }
            None => PreparedNativeTool::current_for(root, environment, self.provider.target()),
        }
    }
}

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Resolve declared campaign inputs without building, starting providers or writing outputs.
    Readiness(CampaignArgs),
    /// Explicitly prepare the selected native conformance image.
    Prepare {
        #[arg(long, value_enum, default_value = "none")]
        provider: Provider,
    },
    /// Execute the selected campaign with the original managed owner.
    Run {
        #[command(flatten)]
        campaign: CampaignArgs,
        #[arg(long, requires = "selection")]
        historical: bool,
        #[arg(long)]
        report_dir: Option<PathBuf>,
        #[arg(long)]
        detach: bool,
        #[arg(long)]
        session: Option<String>,
    },
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Readiness(_) => GenerationIntent::Passive,
            Self::Prepare { .. } | Self::Run { .. } => GenerationIntent::NewWork,
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct SelectionView {
    selection: String,
    source_identity: String,
    executable_sha256: String,
    profile: String,
}
fn selection_view(tool: &PreparedNativeTool) -> Result<SelectionView, HostError> {
    Ok(SelectionView {
        selection: tool.id().into(),
        source_identity: tool.source_identity().map_err(tool_error)?,
        executable_sha256: tool.executable().sha256.clone(),
        profile: tool.target().name().into(),
    })
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct ReadinessView {
    profile: String,
    selection: Option<SelectionView>,
    campaign: Option<serde_json::Value>,
    preparation: Option<String>,
}

fn tool_error(error: PreparedNativeToolError) -> HostError {
    HostError::PreparedNativeTool(Box::new(error))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressSnapshot {
    version: u8,
    historical_image: bool,
    updated_at_ms: u64,
    requested_fixtures: Vec<String>,
    selected_runs: usize,
    omitted_requested_fixtures: usize,
    runs: BTreeMap<String, ProgressRun>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressRun {
    runtime_run: Option<String>,
    phase: Option<String>,
    cases: BTreeMap<String, CaseProgress>,
    total_fixtures: Option<usize>,
    passed: Option<bool>,
    coverage_complete: Option<bool>,
    execution_error: Option<String>,
    settlement_error: Option<String>,
    export_error: Option<String>,
    settlement: Option<String>,
    export: Option<String>,
    published_outputs: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CaseProgress {
    phase: Option<String>,
    status: Option<String>,
    finding: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct CaseView {
    run: String,
    fixture: String,
    #[serde(flatten)]
    progress: CaseProgress,
}

#[derive(Debug, Serialize, JsonSchema)]
struct RunView {
    run: String,
    runtime_run: Option<String>,
    phase: Option<String>,
    total_fixtures: Option<usize>,
    omitted_fixtures: Option<usize>,
    passed: Option<bool>,
    coverage_complete: Option<bool>,
    execution_error: Option<String>,
    settlement_error: Option<String>,
    export_error: Option<String>,
    settlement: Option<String>,
    export: Option<String>,
    published_outputs: Vec<String>,
}

/// A bounded page from the most recent atomically published campaign checkpoint.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ConformancePage {
    original_invocation: crate::envelope::CliInvocationId,
    committed_by_terminal: bool,
    historical_image: bool,
    updated_at_ms: u64,
    selected_runs: usize,
    requested_fixture_count: usize,
    runs: Vec<RunView>,
    cases: Vec<CaseView>,
    next_offset: Option<usize>,
}

/// Read one original owner checkpoint without starting any native tool or service.
pub(crate) fn read_progress(
    directory: &crate::invocation::InvocationDirectory,
    terminal: Option<&[crate::envelope::ArtifactReference]>,
    offset: usize,
) -> Result<Option<ConformancePage>, HostError> {
    const PAGE_ROWS: usize = 20;
    let Some(bytes) = directory.conformance_checkpoint(terminal)? else {
        return Ok(None);
    };
    let snapshot: ProgressSnapshot =
        serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
    if snapshot.version != 1 {
        return Err(HostError::Ownership(
            "unsupported conformance checkpoint version".into(),
        ));
    }
    let mut runs = Vec::new();
    let mut cases = Vec::new();
    let mut index = 0usize;
    for (name, run) in snapshot.runs {
        let count = run.cases.len();
        for (fixture, progress) in run.cases {
            if index >= offset && cases.len() < PAGE_ROWS {
                cases.push(CaseView {
                    run: name.clone(),
                    fixture,
                    progress,
                });
            }
            index += 1;
        }
        runs.push(RunView {
            run: name,
            runtime_run: run.runtime_run,
            phase: run.phase,
            total_fixtures: run.total_fixtures,
            omitted_fixtures: run.total_fixtures.map(|total| total.saturating_sub(count)),
            passed: run.passed,
            coverage_complete: run.coverage_complete,
            execution_error: run.execution_error,
            settlement_error: run.settlement_error,
            export_error: run.export_error,
            settlement: run.settlement,
            export: run.export,
            published_outputs: run.published_outputs,
        });
    }
    if offset > index {
        return Err(HostError::Ownership(
            "case page offset exceeds the observed snapshot".into(),
        ));
    }
    let end = offset + cases.len();
    Ok(Some(ConformancePage {
        original_invocation: directory.id().clone(),
        committed_by_terminal: terminal.is_some(),
        historical_image: snapshot.historical_image,
        updated_at_ms: snapshot.updated_at_ms,
        selected_runs: snapshot.selected_runs,
        requested_fixture_count: snapshot
            .requested_fixtures
            .len()
            .saturating_add(snapshot.omitted_requested_fixtures),
        runs,
        cases,
        next_offset: (end < index).then_some(end),
    }))
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
        Action::Prepare { provider } => {
            let environment = provider.environment(&root, &environment)?;
            let tool = prepared_xtask::prepare_for(&root, &environment, provider.target())
                .map_err(tool_error)?;
            Ok(output.emit(Envelope::response(
                "conformance prepare",
                "selected native campaign image prepared",
                selection_view(&tool)?,
            ))?)
        }
        Action::Readiness(campaign) => {
            let profile = campaign.provider.target().name().into();
            let tool = match campaign.open(&root, &environment) {
                Ok(tool) => tool,
                Err(PreparedNativeToolError::CurrentSelection { source, .. }) if matches!(*source, PreparedNativeToolError::Io(ref io) if io.kind() == ErrorKind::NotFound) =>
                {
                    let mut envelope = Envelope::response(
                        "conformance readiness",
                        "campaign source readiness requires the prepared native owner",
                        ReadinessView {
                            profile,
                            selection: None,
                            campaign: None,
                            preparation: Some(format!(
                                "pse conformance prepare --provider {}",
                                campaign.provider.argument()
                            )),
                        },
                    );
                    envelope.subject_state = SubjectState::Unprepared;
                    return Ok(output.emit_observation(envelope)?);
                }
                Err(error) => return Err(tool_error(error)),
            };
            let environment = campaign.provider.environment(&root, &environment)?;
            let readiness = tool
                .conformance_readiness(campaign.native_arguments(), environment)
                .map_err(tool_error)?;
            Ok(output.emit_observation(Envelope::response(
                "conformance readiness",
                "declared campaign inputs resolved; scientific admission remains execution-time",
                ReadinessView {
                    profile,
                    selection: Some(selection_view(&tool)?),
                    campaign: Some(readiness),
                    preparation: None,
                },
            ))?)
        }
        Action::Run {
            campaign,
            historical,
            report_dir,
            detach,
            session,
        } => {
            let tool = campaign.open(&root, &environment).map_err(tool_error)?;
            let environment = campaign.provider.environment(&root, &environment)?;
            let mut args = campaign.native_arguments();
            if historical {
                args.push("--historical-image".into());
            }
            if let Some(directory) = report_dir {
                args.extend([OsString::from("--report-dir"), directory.into()]);
            }
            let selection = selection_view(&tool)?;
            let payload = tool
                .launch_conformance(args, environment, historical)
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
                    detach,
                    session,
                    resource_class: None,
                    verb: "conformance run",
                    tool: Some(NativeToolSelection {
                        kind: tool.target().name(),
                        selection: selection.selection,
                        source_identity: selection.source_identity,
                        executable_sha256: selection.executable_sha256,
                    }),
                    workflow: None,
                },
                entry,
                output,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        envelope::CliInvocationId,
        generation::PreparedGeneration,
        invocation::{InvocationSpec, InvocationStore},
        paths::CheckoutState,
    };
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::unix::fs::{OpenOptionsExt as _, symlink},
    };

    #[test]
    fn campaign_checkpoint_pages_original_cases_and_refuses_replaced_member() {
        let root = tempfile::tempdir().unwrap();
        let environment = BTreeMap::from([(
            "XDG_STATE_HOME".into(),
            root.path().join("state").into_os_string(),
        )]);
        let state = CheckoutState::from_paths(
            &root.path().join("checkout"),
            &root.path().join("common.git"),
            &environment,
        )
        .unwrap();
        let directory = InvocationStore::new(state)
            .begin(InvocationSpec {
                invocation_id: CliInvocationId::new(),
                command: "conformance run".into(),
                session: None,
                parent: None,
                operation: "fixture",
                control: PreparedGeneration {
                    generation: "generation.fixture".into(),
                    protocol: 1,
                    manifest_digest: "a".repeat(64),
                },
                environment_digest: "b".repeat(64),
                scientific: Vec::new(),
            })
            .unwrap();
        assert!(read_progress(&directory, None, 0).unwrap().is_none());
        let path = directory.path().join("conformance-progress.json");
        let cases = (0..21)
            .map(|index| {
                (
                    format!("fixture-{index:02}"),
                    serde_json::json!({"phase":"returned","status":"Failed","finding":"original"}),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let snapshot = serde_json::json!({
            "version": 1, "historical_image": false, "updated_at_ms": 123,
            "requested_fixtures": [], "selected_runs": 1, "omitted_requested_fixtures": 0,
            "runs": {"first": {"cases": cases, "total_fixtures": 25, "passed": false, "published_outputs":[]}},
        });
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        writeln!(file, "{snapshot}").unwrap();
        let page = read_progress(&directory, None, 0).unwrap().unwrap();
        assert!(!page.committed_by_terminal);
        assert_eq!(page.cases.len(), 20);
        assert_eq!(page.runs[0].omitted_fixtures, Some(4));
        let commitments = directory.conformance_artifacts().unwrap();
        let last = read_progress(&directory, Some(&commitments), page.next_offset.unwrap())
            .unwrap()
            .unwrap();
        assert!(last.committed_by_terminal);
        assert_eq!(last.cases[0].fixture, "fixture-20");
        assert!(last.next_offset.is_none());
        drop(file);
        let replacement = directory.path().join("replacement.json");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&replacement)
            .unwrap();
        writeln!(file, "{snapshot}").unwrap();
        drop(file);
        fs::rename(replacement, &path).unwrap();
        assert!(read_progress(&directory, Some(&commitments), 0).is_err());
        fs::remove_file(&path).unwrap();
        let foreign = root.path().join("foreign.json");
        fs::write(&foreign, snapshot.to_string()).unwrap();
        symlink(&foreign, path).unwrap();
        assert!(read_progress(&directory, None, 0).is_err());
    }
}
