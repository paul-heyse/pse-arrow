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
        /// Choose one semantic world owner; native snapshots remain explicitly prepared.
        #[arg(long, value_enum, default_value = "lsp")]
        backend: semantic::Backend,
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,
        #[arg(long)]
        no_default_features: bool,
        #[arg(long)]
        target: Option<String>,
        /// Explicitly run check --workspace --all-targets for build scripts/proc macros.
        /// Package/input changes require stop and explicit preparation; never checks on save.
        #[arg(long)]
        build_macros: bool,
        /// Select the analyzer's explicit cfg(test) world; no tests are executed.
        #[arg(long)]
        test_context: bool,
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
impl Subject {
    pub(in crate::cli) fn lifetime_locator(
        &self,
        root: &Path,
    ) -> Result<crate::investigation::Locator, HostError> {
        let env = std::env::vars_os().collect();
        let record = semantic::load(root, &env).map_err(error)?.ok_or_else(|| {
            HostError::Ownership(
                "explicitly prepare a native session before semantic inspection".into(),
            )
        })?;
        if self
            .configuration
            .as_ref()
            .is_some_and(|expected| record.configuration.identity().as_ref().ok() != Some(expected))
        {
            return Err(HostError::Ownership(
                "selected native configuration changed".into(),
            ));
        }
        Ok(crate::investigation::Locator::Semantic {
            project: record.configuration.project,
            path: root.join(&self.file).canonicalize()?.into(),
            line: self.line,
            column: self.column,
            generation: self.if_generation.map(|g| g.to_string()),
            source_sha256: self.if_source.clone(),
            candidate_fingerprint: self.if_candidate.clone(),
            session_invocation: Some(record.invocation.as_str().into()),
        })
    }
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
    HostError::Semantic(Box::new(error))
}
#[derive(serde::Serialize, schemars::JsonSchema)]
pub(in crate::cli) struct AnswerView {
    #[serde(flatten)]
    value: semantic::Answer,
    #[serde(skip_serializing_if = "Option::is_none")]
    compiler_layout: Option<semantic::LayoutAssociation>,
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
            backend,
            features,
            no_default_features,
            target,
            build_macros,
            test_context,
        } => {
            let configuration = Configuration {
                backend,
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
                test_context,
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
                    let status = value.into_status().map_err(error)?;
                    if !status.context_current {
                        return Err(HostError::Ownership("captured semantic Cargo/toolchain or preparation inputs changed; stop it explicitly before preparing its replacement".into()));
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
            let record = SessionRecord {
                version: semantic::SESSION_PROTOCOL_VERSION,
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
            let command = match record.configuration.backend {
                semantic::Backend::Lsp => crate::managed::payload::PayloadCommand {
                    program: control.executable(&root, crate::generation::HostTool::Host)?,
                    arguments,
                    cwd: root.clone(),
                    environment: environment.clone(),
                },
                semantic::Backend::NativeSnapshot => {
                    let helper = crate::prepared_xtask::PreparedNativeTool::current_for(
                        &root,
                        &environment,
                        crate::prepared_xtask::NativeToolTarget::NativeAnalysis,
                    )
                    .map_err(|error| HostError::PreparedNativeTool(Box::new(error)))?;
                    helper
                        .launch(arguments, environment.clone())
                        .map_err(|error| HostError::PreparedNativeTool(Box::new(error)))?
                }
            };
            let selection = crate::admission::Selection::load(&root, &environment, None, None)?;
            let store = crate::invocation::InvocationStore::new(
                crate::paths::CheckoutState::resolve(&root, &environment)?,
            );
            let directory = Arc::new(store.begin(crate::invocation::InvocationSpec {
                invocation_id: output.invocation_id.clone(),
                command: "code session prepare".into(),
                session: None,
                parent: None,
                operation: record.configuration.clone(),
                control: control.clone(),
                environment_digest: super::super::environment_digest(&environment),
                scientific: vec![],
            })?);
            let mut launched = runtime.block_on(crate::managed::supervisor::launch_generic(
                crate::managed::supervisor::ManagedRequest {
                    root: root.clone(),
                    directory,
                    command,
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
                    let status = value.into_status().map_err(error)?;
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
                    Some(Ok(value)) => match value.into_status() {
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
pub(in crate::cli) fn query(
    kind: Kind,
    subject: Subject,
    selected: Option<&Path>,
    output: &OutputOptions,
    compiler_layout: Option<&str>,
) -> Result<i32, HostError> {
    if compiler_layout.is_some() && !matches!(&kind, Kind::Layout) {
        return Err(HostError::Ownership(
            "--compiler-layout requires --view layout".into(),
        ));
    }
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    let record = semantic::load(&root,&environment).map_err(error)?.ok_or_else(|| HostError::Ownership("semantic preparation required: pse code session prepare; source-only search/inspect remain available".into()))?;
    let command = if matches!(
        &kind,
        Kind::Rename { .. } | Kind::FileRename { .. } | Kind::Ssr { .. } | Kind::Assist { .. }
    ) {
        "propose entity"
    } else {
        "inspect entity"
    };
    let identity = record.configuration.identity().map_err(error)?;
    let handoff = if matches!(&kind, Kind::Rename { .. } | Kind::Ssr { .. }) {
        proposal_checkout(&root, &record.configuration.project, &environment)
    } else {
        None
    };
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
        occurrence_scope: None,
    });
    let value = super::super::runtime()?.block_on(async {
        let cancellation = tokio_util::sync::CancellationToken::new();
        let answer =
            semantic::client_cancellable(&root, &environment, &record, &request, &cancellation);
        tokio::pin!(answer);
        tokio::select! {
            result = &mut answer => result,
            signal = tokio::signal::ctrl_c() => {
                signal?;
                cancellation.cancel();
                // Keep the receive half through the original settled Terminal.
                answer.await
            },
        }
    });
    let mut value = match value {
        Ok(value) => value.into_query().map_err(error)?,
        Err(error) => {
            let failure = semantic::Failure::from_error(&error);
            let mut response = Envelope::response(
                format!("{command} refusal"),
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
    if let Some(binding) = handoff
        && proposal_checkout(&root, &record.configuration.project, &environment).as_ref()
            == Some(&binding)
        && let Some(proposal) = value.proposal.as_mut()
    {
        let (checkout, id, revision) = binding;
        proposal.bind_handoff(&checkout, id, revision);
    }
    let omitted = value.sections.iter().map(|section| section.omitted).sum();
    let layout_evidence = compiler_layout
        .map(|selector| {
            let state = crate::paths::CheckoutState::resolve(&root, &environment)?;
            crate::compiler_investigation::artifact::observe(&root, &state, selector)
        })
        .transpose()?;
    let compiler_layout = layout_evidence
        .as_ref()
        .map(|evidence| semantic::associate_layout(&value, &evidence.payload));
    let mut response = Envelope::response(
        command,
        "selected analyzer evidence; no compiler acceptance or source writes",
        value,
    );
    if let Some(evidence) = layout_evidence {
        response.providers.extend(evidence.providers);
        response.artifacts.extend(evidence.artifacts);
        response.next.extend(evidence.next);
    }
    let uri = response.payload.source_uri.clone();
    let offset = response.payload.source_byte_offset;
    response.providers.push(envelope::ProviderContext {
        provider: response.payload.session.backend.clone(),
        context: Some(identity),
        analysis_generation: Some(response.payload.generation.to_string()),
        availability: envelope::SubjectState::Available,
        coordinates: vec![
            envelope::SourceCoordinates::Lsp {
                uri: uri.clone(),
                position_encoding: response.payload.session.position_encoding.clone(),
                line: response.payload.position.line,
                character: response.payload.position.character,
            },
            envelope::SourceCoordinates::Utf8Bytes {
                source: uri,
                revision: Some(response.payload.source.source_sha256().into()),
                start: offset,
                end: offset,
            },
        ],
    });
    response.subject_state = if matches!(
        response.payload.availability,
        semantic::Availability::NoMatch
    ) {
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
    response
        .next
        .extend(test_routes(&root, &response.payload, &record.configuration));
    Ok(output.emit_page(
        response.map_payload(|value| AnswerView {
            value,
            compiler_layout,
        }),
        envelope::DetailView {
            budget: output.presentation.budget,
            omitted_records: omitted,
            truncated: omitted > 0,
            ..Default::default()
        },
    )?)
}
pub(in crate::cli) fn edit(
    action: Edit,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    match action {
        Edit::Rename { subject, name } => {
            query(Kind::Rename { name }, subject, selected, output, None)
        }
        Edit::FileRename { subject, to } => query(
            Kind::FileRename { destination: to },
            subject,
            selected,
            output,
            None,
        ),
        Edit::Assists(subject) => query(Kind::Assists, subject, selected, output, None),
        Edit::Assist { subject, select } => query(
            Kind::Assist { index: select },
            subject,
            selected,
            output,
            None,
        ),
        Edit::Ssr { subject, rule } => query(Kind::Ssr { rule }, subject, selected, output, None),
    }
}

fn proposal_checkout(
    root: &Path,
    project: &Path,
    environment: &crate::environment::Environment,
) -> Option<(PathBuf, String, String)> {
    let root = root.canonicalize().ok()?;
    if project.canonicalize().ok()? != root {
        return None;
    }
    let repository = gix::open_opts(&root, gix::open::Options::isolated()).ok()?;
    let common = repository.common_dir().canonicalize().ok()?;
    let state = crate::paths::CheckoutState::from_paths(&root, &common, environment).ok()?;
    let revision = repository.head_id().ok()?.to_string();
    Some((root, state.checkout_id.as_str().to_owned(), revision))
}
pub(in crate::cli) fn test_routes(
    root: &Path,
    answer: &semantic::Answer,
    configuration: &Configuration,
) -> Vec<envelope::NextAction> {
    if configuration.project != root
        || !matches!(
            answer.source.origin(),
            crate::source_read::SourceOrigin::Working
        )
    {
        return Vec::new();
    }
    let mut next = Vec::new();
    for candidate in &answer.observed.tests {
        if candidate.eligible == Some(false) {
            continue;
        }
        let Some(location) = &candidate.location else {
            continue;
        };
        let path = match location {
            semantic::FactLocation::Bytes { path, .. } => Some(path.clone()),
            semantic::FactLocation::Protocol { uri, .. } => url::Url::parse(uri)
                .ok()
                .and_then(|uri| uri.to_file_path().ok()),
        };
        let Some(path) = path.and_then(|path| path.canonicalize().ok()) else {
            continue;
        };
        let original = url::Url::parse(&answer.source_uri)
            .ok()
            .and_then(|uri| uri.to_file_path().ok());
        let digest = if original.as_ref() == Some(&path) {
            Some(answer.source.source_sha256())
        } else {
            answer
                .location_sources
                .iter()
                .find(|source| source.path == path)
                .map(|source| source.source_sha256.as_str())
        };
        let Some(digest) = digest else { continue };
        let Ok(source) =
            crate::source_read::SourceRequest::Working { path: path.clone() }.acquire()
        else {
            continue;
        };
        if source.identity().source_sha256() != digest {
            continue;
        }
        let reference =
            crate::investigation::ResultReference::new(crate::investigation::Locator::Source {
                path: path.clone().into(),
                source: crate::investigation::SourceKind::Working,
                range: None,
                guard: Some(crate::investigation::SourceGuard {
                    root: Some(root.to_path_buf().into()),
                    sha256: Some(digest.into()),
                    length: Some(source.text().len() as u64),
                    modified_nanos: None,
                    observed: None,
                    git_blob: None,
                }),
            });
        let Ok(reference) = reference.encode() else {
            continue;
        };
        let Some(root) = root.to_str() else { continue };
        let mut argv = vec![
            "pse".into(),
            "-C".into(),
            root.into(),
            "inspect".into(),
            "test-selection".into(),
            "--from-result".into(),
            reference,
        ];
        if !configuration.features.is_empty() {
            argv.extend(["--features".into(), configuration.features.join(",")]);
        }
        if configuration.no_default_features {
            argv.push("--no-default-features".into());
        }
        if let Some(target) = &configuration.target {
            argv.extend(["--target".into(), target.clone()]);
        }
        let exact = if candidate.runnable_kind.as_deref() == Some("test")
            && candidate.eligible == Some(true)
            && let [target] = candidate.cargo_targets.as_slice()
            && target.required_features_satisfied
            && target.manifest.parent().is_some_and(|owner| {
                owner.starts_with(root) && path.starts_with(owner) && target.root.starts_with(owner)
            })
            && let Some(test) = &candidate.test_id
            && !test.is_empty()
            && test
                .chars()
                .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == ':')
        {
            let selector = match target.kind.as_str() {
                "lib" | "rlib" | "proc-macro" => Some(vec!["--lib".into()]),
                "test" | "bin" => Some(vec![format!("--{}", target.kind), target.name.clone()]),
                _ => None,
            };
            if let Some(selector) = selector {
                argv.extend(selector);
                argv.extend(["-E".into(), format!("test(={test})")]);
                true
            } else {
                false
            }
        } else {
            false
        };
        let scope = if exact {
            "a unique snapshot target and exact test name; current Cargo admission checks the selection"
        } else {
            "the containing package because the snapshot could not establish an eligible exact target and test name"
        };
        next.push(envelope::NextAction::Command{argv,reason:format!("{} runnable ({}) nominates {scope}; this candidate supplies no coverage or verdict", candidate.runnable_kind.as_deref().unwrap_or("unclassified"), candidate.relatedness.as_deref().unwrap_or("native relationship"))});
    }
    next
}

#[cfg(test)]
mod runnable_route_tests {
    use super::*;
    #[test]
    fn runnable_routes_bind_current_source_and_ignore_analyzer_argv() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("lib.rs");
        std::fs::write(&path, "#[test] fn selected() {}\n").unwrap();
        let source = crate::source_read::SourceRequest::Working { path: path.clone() }
            .acquire()
            .unwrap();
        let configuration = Configuration {
            backend: semantic::Backend::NativeSnapshot,
            project: temporary.path().to_owned(),
            features: vec!["chosen".into()],
            no_default_features: true,
            target: None,
            build_macros: false,
            test_context: false,
        };
        let mut answer = semantic::Answer {
            session: semantic::AnalysisContext {
                backend: "native-snapshot".into(),
                configuration: configuration.clone(),
                position_encoding: "utf-8".into(),
                backend_status: None,
                preparation: "restricted".into(),
            },
            source: source.identity().clone(),
            source_uri: url::Url::from_file_path(&path).unwrap().into(),
            source_byte_offset: 0,
            availability: semantic::Availability::Available,
            generation: 1,
            position: semantic::ProtocolPosition::default(),
            observed: semantic::ObservedFacts {
                tests: vec![semantic::TestCandidateFact {
                    test_id: Some("selected".into()),
                    cfg: None,
                    relatedness: Some("native relationship".into()),
                    eligible: Some(true),
                    cfg_eligible: Some(true),
                    cargo_targets: vec![semantic::RunnableTargetFact {
                        package_id: "fixture".into(),
                        manifest: temporary.path().join("Cargo.toml"),
                        root: path.clone(),
                        name: "fixture".into(),
                        kind: "lib".into(),
                        required_features: Vec::new(),
                        active_features: Default::default(),
                        required_features_satisfied: true,
                    }],
                    snapshot_update: None,
                    label: Some("display only".into()),
                    location: Some(semantic::FactLocation::Bytes {
                        path: path.clone(),
                        start: 0,
                        end: 24,
                    }),
                    runnable_kind: Some("test".into()),
                    cargo_args: vec!["rm".into(), "--not-a-selection".into()],
                    executable_args: vec!["untrusted-argv".into()],
                    original: serde_json::json!({"snapshot_update":"intent only"}),
                }],
                ..Default::default()
            },
            omitted_relationships: Default::default(),
            native_types: None,
            sections: vec![],
            location_sources: vec![],
            proposal: None,
            limitations: vec![],
        };
        let routes = test_routes(temporary.path(), &answer, &configuration);
        let [envelope::NextAction::Command { argv, .. }] = routes.as_slice() else {
            panic!("guarded route missing")
        };
        assert!(
            argv.windows(2)
                .any(|pair| pair == ["inspect", "test-selection"])
        );
        assert!(argv.iter().any(|arg| arg == "test(=selected)"));
        assert!(argv.iter().any(|arg| arg == "--lib"));
        assert!(
            !argv
                .iter()
                .any(|arg| arg == "rm" || arg == "untrusted-argv")
        );
        let index = argv.iter().position(|arg| arg == "--from-result").unwrap();
        let reference = crate::investigation::ResultReference::decode(&argv[index + 1]).unwrap();
        assert!(matches!(
            reference.locator,
            crate::investigation::Locator::Source { guard: Some(_), .. }
        ));
        let mut external = configuration.clone();
        external.project = temporary.path().join("external-project");
        assert!(test_routes(temporary.path(), &answer, &external).is_empty());
        answer.observed.tests[0].eligible = Some(false);
        assert!(test_routes(temporary.path(), &answer, &configuration).is_empty());
        answer.observed.tests[0].eligible = None;
        let routes = test_routes(temporary.path(), &answer, &configuration);
        let [envelope::NextAction::Command { argv, .. }] = routes.as_slice() else {
            panic!("conservative route missing")
        };
        assert!(!argv.iter().any(|arg| arg == "--lib" || arg == "-E"));
        std::fs::write(path, "changed historical witness\n").unwrap();
        assert!(test_routes(temporary.path(), &answer, &configuration).is_empty());
    }
}
