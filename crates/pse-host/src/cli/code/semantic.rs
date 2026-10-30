// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed semantic command consumers; syntax/query reads remain server-free.
use super::super::{EntryGuard, GenerationIntent, HostError, OutputOptions, checkout_root};
use crate::{
    envelope::{self, Envelope},
    semantic::{self, Configuration, Kind, Request, SessionRecord},
};
use clap::{Args, Subcommand};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Subcommand)]
pub(in crate::cli) enum Session {
    /// Explicitly prepare/reuse a restricted native analyzer model. Never runs check-on-save.
    Prepare {
        /// Cargo project root; defaults to the owning checkout.
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,
        #[arg(long)]
        no_default_features: bool,
        #[arg(long)]
        target: Option<String>,
        /// Explicitly prepare build scripts/proc macros through the original correctness adapter.
        #[arg(long)]
        build_macros: bool,
    },
    /// Observe the original native service; never prepares or rebuilds it.
    Status,
    /// Stop and prove drain of the original analyzer service and its descendants.
    Stop,
}
impl Session {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Prepare { .. } => GenerationIntent::NewWork,
            Self::Stop => GenerationIntent::ExistingControl,
            Self::Status => GenerationIntent::Passive,
        }
    }
}
#[derive(Debug, Args)]
pub(in crate::cli) struct Subject {
    /// Exact working Rust file; no editor attachment or unsaved buffer is assumed.
    file: PathBuf,
    /// One-based source line.
    #[arg(long, default_value_t = 1)]
    line: usize,
    /// One-based Unicode scalar column, excluding CR/LF; converted to negotiated encoding.
    #[arg(long, default_value_t = 1)]
    column: usize,
    /// Refuse if the original source digest differs.
    #[arg(long)]
    if_source: Option<String>,
    /// Refuse stale generation-bound action selection.
    #[arg(long)]
    if_generation: Option<u64>,
    /// Discovery fingerprint for a selected hint/completion/assist; indexes are not identities.
    #[arg(long)]
    if_candidate: Option<String>,
    /// Require this explicit session configuration identity.
    #[arg(long)]
    configuration: Option<String>,
    /// Inclusive final line for selected inlay/SSR range; defaults to the selected line.
    #[arg(long)]
    end_line: Option<usize>,
}
#[derive(Debug, Subcommand)]
pub(in crate::cli) enum Edit {
    /// Preview only the selected entity's native semantic rename; no source writes.
    Rename {
        #[command(flatten)]
        subject: Subject,
        #[arg(long)]
        name: String,
    },
    /// Preview native file-rename follow-up edits and the exact filesystem operation.
    FileRename {
        #[command(flatten)]
        subject: Subject,
        #[arg(long)]
        to: PathBuf,
    },
    /// Enumerate native assists without eagerly resolving every edit.
    Assists(Subject),
    /// Resolve one source/generation-bound native assist into an immutable proposal.
    Assist {
        #[command(flatten)]
        subject: Subject,
        #[arg(long)]
        select: usize,
    },
    /// Preview the native semantic SSR rule (PATTERN ==>> REPLACEMENT), scoped to selected lines.
    Ssr {
        #[command(flatten)]
        subject: Subject,
        #[arg(long)]
        rule: String,
    },
}
fn error(error: semantic::Error) -> HostError {
    HostError::Ownership(error.to_string())
}
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub(in crate::cli) struct AnswerView {
    #[schemars(with = "semantic::Answer")]
    value: Box<serde_json::value::RawValue>,
}
pub(super) fn session(
    action: Session,
    selected: Option<&Path>,
    entry: &EntryGuard,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    match action {
        Session::Prepare {
            project,
            features,
            no_default_features,
            target,
            build_macros,
        } => {
            let configuration = Configuration {
                project: project.map_or_else(
                    || root.clone(),
                    |path| {
                        if path.is_absolute() {
                            path
                        } else {
                            root.join(path)
                        }
                    },
                ),
                features,
                no_default_features,
                target,
                build_macros,
            }
            .resolve()
            .map_err(error)?;
            let _lock = semantic::preparation_lock(&root, &environment).map_err(error)?;
            let runtime = super::super::runtime()?;
            if let Some(previous) = semantic::load(&root, &environment).map_err(error)? {
                let original = runtime.block_on(crate::managed::supervisor::observe_recorded(
                    &root,
                    &previous.invocation,
                    &environment,
                ))?;
                if !original.native_drained {
                    if previous.configuration != configuration {
                        return Err(HostError::Ownership("a differing semantic configuration is active; stop it explicitly before preparing another".into()));
                    }
                    let value = runtime
                        .block_on(async {
                            tokio::time::timeout(
                                Duration::from_secs(3),
                                semantic::client(
                                    &root,
                                    &environment,
                                    &previous,
                                    &Request::Status {},
                                ),
                            )
                            .await
                        })
                        .map_err(|e| {
                            HostError::Ownership(format!("semantic status deadline: {e}"))
                        })?
                        .map_err(error)?;
                    let status: semantic::SessionStatus = serde_json::from_value(value)
                        .map_err(|e| HostError::Ownership(e.to_string()))?;
                    if !status.context_current {
                        return Err(HostError::Ownership("captured semantic Cargo/toolchain context changed; stop it explicitly before preparing its replacement".into()));
                    }
                    return Ok(output.emit(Envelope::response(
                        "code session prepare",
                        "reused the original native analyzer session",
                        status,
                    ))?);
                }
            }
            let control = entry.prepared().cloned().ok_or_else(|| {
                HostError::Ownership(
                    "semantic preparation requires explicitly prepared host tools".into(),
                )
            })?;
            let store = crate::invocation::InvocationStore::new(
                crate::paths::CheckoutState::resolve(&root, &environment)?,
            );
            let directory = Arc::new(store.begin(crate::invocation::InvocationSpec {
                invocation_id: output.invocation_id.clone(),
                command: "code session prepare".into(),
                session: None,
                parent: None,
                operation: configuration.clone(),
                control: control.clone(),
                environment_digest: super::super::environment_digest(&environment),
                scientific: vec![],
            })?);
            let record = SessionRecord {
                version: 1,
                invocation: output.invocation_id.clone(),
                configuration,
            };
            let arguments = vec![
                "-C".into(),
                root.as_os_str().to_owned(),
                "__semantic-service".into(),
                "--invocation".into(),
                record.invocation.as_str().into(),
                "--configuration".into(),
                serde_json::to_string(&record.configuration)
                    .map_err(|e| HostError::Ownership(e.to_string()))?
                    .into(),
            ];
            let selection = crate::admission::Selection::load(&root, &environment, None, None)?;
            let mut launched = runtime.block_on(crate::managed::supervisor::launch_generic(
                crate::managed::supervisor::ManagedRequest {
                    root: root.clone(),
                    directory,
                    command: crate::managed::payload::PayloadCommand {
                        program: control.executable(&root, crate::generation::HostTool::Host)?,
                        arguments,
                        cwd: root.clone(),
                        environment: environment.clone(),
                    },
                    selection,
                    policy: crate::managed::payload::PayloadPolicy {
                        startup: Duration::from_secs(30),
                        drain: Duration::from_secs(30),
                        write: Duration::from_secs(5),
                    },
                },
            ))?;
            if let Err(operation) =
                semantic::prepare_record(&root, &environment, &record).map_err(error)
            {
                return match runtime
                    .block_on(launched.cancel(Instant::now() + Duration::from_secs(30)))
                {
                    Ok(_) => Err(operation),
                    Err(cleanup) => Err(HostError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(cleanup),
                    }),
                };
            }
            let prepared = runtime.block_on(async {
                let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
                loop {
                    match tokio::time::timeout(Duration::from_secs(3), semantic::client(&root,&environment,&record,&Request::Status {})).await {
                        Ok(Ok(value)) => break Ok(value),
                        failure => {
                            if tokio::time::Instant::now() >= deadline { break Err(HostError::Ownership(format!("semantic preparation did not become reachable: {failure:?}"))); }
                            if launched.wait(Some(Instant::now())).await?.is_some() { break Err(HostError::Ownership("original semantic service ended before readiness; inspect its original invocation result/log".into())); }
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            });
            match prepared {
                Ok(value) => {
                    let status: semantic::SessionStatus = serde_json::from_value(value)
                        .map_err(|e| HostError::Ownership(e.to_string()))?;
                    Ok(output.emit(Envelope::response("code session prepare","original native analyzer session initialized; Cargo model preparation remains visible",status))?)
                }
                Err(operation) => match runtime
                    .block_on(launched.cancel(Instant::now() + Duration::from_secs(30)))
                {
                    Ok(_) => Err(operation),
                    Err(cleanup) => Err(HostError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(cleanup),
                    }),
                },
            }
        }
        Session::Status => {
            let record = semantic::load(&root, &environment).map_err(error)?;
            let mut response = Envelope::response(
                "code session status",
                "semantic preparation is absent",
                semantic::SessionOverview {
                    prepared: false,
                    record: None,
                    original_native: None,
                    analyzer: None,
                    unavailable: None,
                },
            );
            if let Some(record) = record {
                let runtime = super::super::runtime()?;
                let original = runtime.block_on(crate::managed::supervisor::observe_recorded(
                    &root,
                    &record.invocation,
                    &environment,
                ))?;
                let live = !original.native_drained;
                let state = if live {
                    Some(runtime.block_on(async {
                        tokio::time::timeout(
                            Duration::from_secs(3),
                            semantic::client(&root, &environment, &record, &Request::Status {}),
                        )
                        .await
                        .map_err(|e| {
                            HostError::Ownership(format!("semantic readiness unavailable: {e}"))
                        })?
                        .map_err(error)
                    }))
                } else {
                    None
                };
                response.summary = if live {
                    "original native semantic session observed"
                } else {
                    "original semantic session is drained"
                }
                .into();
                let (analyzer, unavailable) = match state {
                    Some(Ok(value)) => match serde_json::from_value(value) {
                        Ok(value) => (Some(value), None),
                        Err(error) => (
                            None,
                            Some(format!(
                                "semantic status response differs from declared protocol: {error}"
                            )),
                        ),
                    },
                    Some(Err(error)) => (None, Some(error.to_string())),
                    None => (None, None),
                };
                response.payload = semantic::SessionOverview {
                    prepared: true,
                    record: Some(record),
                    original_native: Some(original),
                    analyzer,
                    unavailable,
                };
            } else {
                response.subject_state = envelope::SubjectState::Unprepared;
            }
            Ok(output.emit(response)?)
        }
        Session::Stop => {
            let record = semantic::load(&root, &environment)
                .map_err(error)?
                .ok_or_else(|| HostError::Ownership("semantic preparation is absent".into()))?;
            let original = super::super::runtime()?
                .block_on(semantic::stop(&root, &environment, &record))
                .map_err(error)?;
            let store = crate::invocation::InvocationStore::new(
                crate::paths::CheckoutState::resolve(&root, &environment)?,
            );
            let response = super::super::control::settled_response(
                &store,
                record.invocation,
                original,
                "code session stop",
                "original analyzer service drained; original performing-owner evidence inspected",
                true,
            )?;
            Ok(output.emit(response)?)
        }
    }
}
pub(super) fn query(
    kind: Kind,
    subject: Subject,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    let record = semantic::load(&root,&environment).map_err(error)?.ok_or_else(|| HostError::Ownership("semantic preparation required: pse code session prepare; source-only outline/show/query remain available".into()))?;
    let identity = record.configuration.identity().map_err(error)?;
    if subject
        .configuration
        .as_ref()
        .is_some_and(|expected| expected != &identity)
    {
        return Err(HostError::Ownership(
            "selected semantic configuration differs from active session".into(),
        ));
    }
    let file = if subject.file.is_absolute() {
        subject.file
    } else {
        root.join(subject.file)
    };
    let request = Request::Query(semantic::Query {
        kind,
        file,
        line: subject.line,
        column: subject.column,
        if_source: subject.if_source,
        if_generation: subject.if_generation,
        if_candidate: subject.if_candidate,
        limit: output.presentation.budget.unwrap_or(12),
        end_line: subject.end_line,
    });
    let value = super::super::runtime()?.block_on(async {
        tokio::select! {
            answer = semantic::client(&root,&environment,&record,&request) => answer,
            signal = tokio::signal::ctrl_c() => { signal?; Err(semantic::Error::Cancelled) },
        }
    });
    let value = match value {
        Ok(value) => value,
        Err(error) => {
            let failure = semantic::Failure::from_error(&error);
            let mut response = Envelope::response(
                "code semantic refusal",
                "semantic request did not produce a current answer; original session retains its independent lifetime",
                failure,
            );
            response.subject_state = match response.payload.state {
                semantic::FailureState::Stale => envelope::SubjectState::Stale,
                semantic::FailureState::Cancelled => envelope::SubjectState::Cancelled,
                semantic::FailureState::Unavailable | semantic::FailureState::Unsupported => {
                    envelope::SubjectState::Unavailable
                }
                semantic::FailureState::Failed => envelope::SubjectState::Unknown,
            };
            response.outcome =
                if matches!(response.payload.state, semantic::FailureState::Cancelled) {
                    envelope::CommandOutcome::Interrupted
                } else {
                    envelope::CommandOutcome::Failure
                };
            response.failures.push(envelope::ErrorRecord::from_typed(
                &error,
                &envelope::RedactionPolicy::from_environment(),
            ));
            return Ok(output.emit_page(response, envelope::DetailView::default())?);
        }
    };
    let omitted = value
        .get("sections")
        .and_then(serde_json::Value::as_array)
        .map_or(0, |sections| {
            sections
                .iter()
                .filter_map(|s| s["omitted"].as_u64())
                .sum::<u64>() as usize
        });
    let mut response = Envelope::response(
        "code semantic",
        "selected analyzer evidence; no compiler acceptance or source writes",
        value,
    );
    let uri = response.payload["source_uri"]
        .as_str()
        .ok_or_else(|| {
            HostError::Ownership("semantic producer omitted its actual source URI".into())
        })?
        .to_string();
    let offset = response.payload["source_byte_offset"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            HostError::Ownership("semantic producer omitted its source byte offset".into())
        })?;
    response.providers.push(envelope::ProviderContext {
        provider: response.payload["session"]["backend"]
            .as_str()
            .unwrap_or("rust-analyzer")
            .into(),
        context: Some(identity),
        analysis_generation: response.payload["generation"]
            .as_u64()
            .map(|value| value.to_string()),
        availability: envelope::SubjectState::Available,
        coordinates: vec![
            envelope::SourceCoordinates::Lsp {
                uri: uri.clone(),
                position_encoding: response.payload["session"]["position_encoding"]
                    .as_str()
                    .unwrap_or("unknown")
                    .into(),
                line: response.payload["position"]["line"]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .unwrap_or(0),
                character: response.payload["position"]["character"]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .unwrap_or(0),
            },
            envelope::SourceCoordinates::Utf8Bytes {
                source: uri,
                revision: response.payload["source"]["source_sha256"]
                    .as_str()
                    .map(str::to_owned),
                start: offset,
                end: offset,
            },
        ],
    });
    response.subject_state = if response.payload["availability"] == "no_match" {
        envelope::SubjectState::NoMatch
    } else {
        envelope::SubjectState::Available
    };
    response.observation.scope = "selected rust-analyzer model and synchronized source".into();
    response.observation.completeness = envelope::Completeness::Partial;
    response.observation.required = false;
    response.observation.limitations.push("Backend-selected Cargo contexts can be ambiguous; restricted macro/build-script preparation is incomplete. Native evidence is retained without parsing debug/hover strings into typed authority.".into());
    response.next.push(envelope::NextAction::Command {
        argv: vec![
            "pse".into(),
            "-C".into(),
            root.to_string_lossy().into(),
            "code".into(),
            "session".into(),
            "status".into(),
        ],
        reason: "inspect actual session context, negotiated capabilities and pending backend state"
            .into(),
    });
    if record.configuration.project == root {
        response
            .next
            .extend(test_routes(&root, &response.payload, &record.configuration));
    }
    let typed = AnswerView {
        value: serde_json::value::to_raw_value(&response.payload)
            .map_err(|e| HostError::Ownership(e.to_string()))?,
    };
    Ok(output.emit_page(
        response.map_payload(|_| typed),
        envelope::DetailView {
            budget: output.presentation.budget,
            omitted_records: omitted,
            truncated: omitted > 0,
            ..Default::default()
        },
    )?)
}
pub(super) fn edit(
    action: Edit,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    match action {
        Edit::Rename { subject, name } => query(Kind::Rename { name }, subject, selected, output),
        Edit::FileRename { subject, to } => query(
            Kind::FileRename { destination: to },
            subject,
            selected,
            output,
        ),
        Edit::Assists(subject) => query(Kind::Assists, subject, selected, output),
        Edit::Assist { subject, select } => {
            query(Kind::Assist { index: select }, subject, selected, output)
        }
        Edit::Ssr { subject, rule } => query(Kind::Ssr { rule }, subject, selected, output),
    }
}
pub(super) fn hint(
    subject: Subject,
    index: Option<usize>,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    query(
        index.map_or(Kind::Hints, |index| Kind::Hint { index }),
        subject,
        selected,
        output,
    )
}
fn test_routes(
    root: &Path,
    answer: &serde_json::Value,
    configuration: &Configuration,
) -> Vec<envelope::NextAction> {
    let mut next = Vec::new();
    let Some(sections) = answer["sections"].as_array() else {
        return next;
    };
    for section in sections
        .iter()
        .filter(|section| section["category"] == "related_test_candidates")
    {
        let Some(candidates) = section["evidence"].as_array() else {
            continue;
        };
        for candidate in candidates {
            let runnable = &candidate["runnable"];
            if runnable["kind"] != "cargo" {
                continue;
            }
            let Some(arguments) = runnable["args"]["cargoArgs"].as_array() else {
                continue;
            };
            let package = arguments
                .windows(2)
                .find(|pair| pair[0] == "-p" || pair[0] == "--package")
                .and_then(|pair| pair[1].as_str());
            let test = runnable["args"]["executableArgs"]
                .as_array()
                .and_then(|args| args.first())
                .and_then(serde_json::Value::as_str);
            let (Some(package), Some(test)) = (package, test) else {
                continue;
            };
            if test.is_empty()
                || !test
                    .chars()
                    .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == ':')
            {
                continue;
            }
            let mut argv = vec![
                "pse".into(),
                "-C".into(),
                root.to_string_lossy().into(),
                "tests".into(),
                "-p".into(),
                package.into(),
            ];
            if arguments.iter().any(|arg| arg == "--lib") {
                argv.push("--lib".into());
            } else if let Some(target) = arguments
                .windows(2)
                .find(|pair| pair[0] == "--test")
                .and_then(|pair| pair[1].as_str())
            {
                argv.extend(["--test".into(), target.into()]);
            } else {
                continue;
            }
            if !configuration.features.is_empty() {
                argv.extend(["--features".into(), configuration.features.join(",")]);
            }
            if configuration.no_default_features {
                argv.push("--no-default-features".into());
            }
            if let Some(target) = &configuration.target {
                argv.extend(["--target".into(), target.clone()]);
            }
            argv.extend(["-E".into(), format!("test(={test})")]);
            next.push(envelope::NextAction::Command{argv,reason:"ask the original test owner to enumerate this semantic candidate; candidate generation is not affected selection or test execution".into()});
        }
    }
    next
}
