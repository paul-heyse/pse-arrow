// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive investigation adapters retain the existing scientific and execution authorities.
use crate::{
    HostError,
    envelope::{Completeness, Envelope, ErrorRecord, OutputOptions, RedactionPolicy},
    investigation::{EvidenceKind, Locator, ResultReference},
    test_workflow::{CandidateLane, SourceCandidate},
};
use clap::Subcommand;
use std::{
    num::{NonZeroU64, NonZeroUsize},
    path::{Path, PathBuf},
};
#[path = "dependency_usage.rs"]
mod dependency_usage;
#[path = "schema.rs"]
mod schema;
#[path = "source.rs"]
mod source;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(super) enum EvidenceLayout {
    Compact,
    Directory,
}

/// Explicit evidence kinds never guess whether an operand is a file or invocation.
#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explain effective CLI settings without preparing or connecting.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Settings {
        #[command(flatten)]
        options: crate::cli::product::context::Options,
        /// Resolve this configured target alias.
        #[arg(long)]
        target: Option<String>,
    },
    /// Observe written references to exact Cargo package IDs in a prepared native world.
    DependencyUsage(dependency_usage::Args),
    /// Compare explicitly addressed source, API, retained context or native world evidence.
    Comparison(super::comparison::Args),
    /// Navigate an explicitly supplied complete descriptor set, or compare schema worlds.
    Schema(schema::Args),
    /// Explain selected binary, JSON or textproto payloads without invoking an RPC.
    Payload(crate::schema::payload::Args),
    /// Export source-generated observations from original private workflow evidence.
    DeveloperEvidence {
        #[arg(long)]
        from_result: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        output: PathBuf,
        /// Full original dependencies, inventory selection, or report-only facts.
        #[arg(long, value_enum, default_value_t = crate::test_workflow::execute::EvidenceScope::Full)]
        scope: crate::test_workflow::execute::EvidenceScope,
        /// Compact bounded protobuf, or an explicit staged external-file directory.
        #[arg(long, value_enum, default_value_t = EvidenceLayout::Compact)]
        layout: EvidenceLayout,
    },
    /// Verify a portable directory bundle and optionally return a bounded member range.
    DeveloperEvidenceBundle {
        input: PathBuf,
        #[arg(long)]
        member: Option<usize>,
        #[arg(long, default_value = "0", requires = "member")]
        offset: u64,
        #[arg(long, default_value = "4096", requires = "member")]
        length: NonZeroU64,
    },
    /// Publish one whole verified original directory member through bounded sequential I/O.
    DeveloperEvidenceMember {
        input: PathBuf,
        #[arg(long)]
        member: usize,
        #[arg(long)]
        output: PathBuf,
    },
    /// Read written declarations or document headings without semantic preparation.
    Outline(source::Outline),
    /// Read a known declaration, literal item, position or exact original range.
    Declaration(source::Declaration),
    /// Inspect the declared authored owner of an exact generated-source hit.
    Origin(source::Origin),
    /// Inspect a known Rust position through its explicitly prepared native semantic owner.
    Entity(source::Entity),
    /// Inspect an exact locked API, original docs/examples, reexports or structural differences.
    Api(crate::cli::api::InspectArgs),
    /// Inspect native exact dependency identity, features and inclusion explanations.
    Dependency {
        package: String,
        #[arg(long)]
        metadata: Option<PathBuf>,
        #[arg(long)]
        why: bool,
        #[arg(long)]
        feature_member: Option<String>,
    },
    /// Inspect planned compiler units without building or listing tests.
    DependencyUnits {
        dependency: String,
        /// Explain qualified correspondence to the explicitly prepared native world.
        #[arg(long)]
        native_world: bool,
        /// Include the original captured metadata alongside the planned-unit explanation.
        #[arg(long)]
        metadata_original: bool,
        #[command(flatten)]
        selection: super::super::selection::SelectionArgs,
    },
    /// Inspect existing source, generated and document ownership independently.
    Owner {
        path: PathBuf,
        #[arg(long)]
        metadata: Option<PathBuf>,
        #[arg(long)]
        xtask_selection: Option<String>,
    },
    /// Read original compiler failures or one addressed original record.
    #[command(group(clap::ArgGroup::new("diagnostic_operand").args(["invocation", "address"]).required(true).multiple(false)))]
    Diagnostic {
        invocation: Option<String>,
        #[arg(long)]
        address: Option<String>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        raw: bool,
        #[arg(long)]
        source: bool,
        #[arg(long, default_value = "4194304")]
        max_record_bytes: NonZeroUsize,
    },
    /// Read exact artifact ranges with native validators; this does not decode a protocol.
    Artifact {
        file: PathBuf,
        #[arg(long, default_value = "0")]
        offset: u64,
        #[arg(long)]
        length: Option<NonZeroU64>,
        #[arg(long, conflicts_with = "length")]
        all: bool,
        #[arg(long)]
        if_match: Option<String>,
    },
    /// Read an original native compiler artifact after its existing whole-file verification.
    CompilerArtifact {
        invocation: String,
        file: usize,
        #[arg(long, default_value = "0")]
        offset: u64,
        #[arg(long, default_value = "4096")]
        length: NonZeroU64,
    },
    /// Inspect compiler artifacts, or original admitted check/test/compiler contexts with --metadata.
    CompilerArtifacts {
        invocation: String,
        /// Explicit retained metadata to address alongside original admitted units.
        #[arg(long)]
        metadata: Option<PathBuf>,
    },
    /// Read compiler-verified original source bytes from an acquired facts artifact.
    CompilerSource {
        invocation: String,
        source_file: u32,
        #[arg(long, default_value = "0")]
        offset: u64,
        #[arg(long, default_value = "4096")]
        length: NonZeroU64,
    },
    /// Explain original typed capture, suspension and cleanup observations without rerunning rustc.
    CompilerFacts {
        invocation: String,
        /// Exact acquired body path; omitted selects all originally acquired bodies.
        #[arg(long)]
        body: Option<String>,
        /// Whole-body JSON payload budget; provenance is always retained.
        #[arg(long, default_value = "16384")]
        max_bytes: NonZeroUsize,
        /// Select acquired detail before whole-body output budgeting; never acquires missing facts.
        #[arg(long, value_enum, default_value = "all")]
        fact_focus: crate::compiler_investigation::artifact::FactFocus,
    },
    /// Read an already captured original test population or its original failures.
    Tests {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        failures: bool,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, default_value = "20")]
        limit: NonZeroUsize,
        #[arg(long, default_value = "262144")]
        page_bytes: NonZeroUsize,
    },
    /// Explain real native test selection; no binary listing or test execution occurs.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    #[command(add(crate::catalog::Examples(&[&[], &["pse-host", "--lib"]])))]
    TestSelection {
        #[command(flatten)]
        tests: super::super::selection::TestArgs,
        /// Validate a source witness and explain the current package or its explicitly selected target.
        #[arg(long)]
        from_result: Option<String>,
        #[arg(long, hide = true, requires = "from_result")]
        target_witness: Option<String>,
        #[arg(long, hide = true, requires = "from_result", conflicts_with = "admitted")]
        doctest_candidate: bool,
        #[arg(long)]
        admitted: bool,
        #[arg(long)]
        full: bool,
    },
    /// Explain selected native check declarations or admitted compiler units without compiling.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    CheckSelection {
        #[command(flatten)]
        selection: super::super::selection::SelectionArgs,
        #[arg(long)]
        admitted: bool,
        #[arg(long)]
        full: bool,
    },
    /// Read the original managed result without waiting, cancellation or resubmission.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    #[command(add(crate::catalog::Examples(&[&["latest"]])))]
    Invocation {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        /// Inspect a bounded detail retained by the original owner.
        #[arg(long, value_enum)]
        detail: Option<crate::invocation_view::ResultDetail>,
    },
    /// Read original retained stdout/stderr byte ranges.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    #[command(add(crate::catalog::Examples(&[&["latest", "--stderr", "--tail"]])))]
    Log {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        stderr: bool,
        #[arg(long, default_value = "0", conflicts_with = "tail")]
        offset: u64,
        /// Select at most 262144 original bytes in this window.
        #[arg(long, default_value = "4096")]
        length: NonZeroU64,
        #[arg(long)]
        tail: bool,
    },
    /// Explain conservative affected-test reach through the existing native owner.
    Affected {
        #[command(flatten)]
        selection: super::super::affected::Selection,
    },
    /// Inspect native Git layers and source/test ownership without compiling.
    Changes {
        #[command(flatten)]
        selection: super::super::changes::Selection,
    },
}

// Signal observation is independent of the original copy/publication result.
// A completed receipt must survive a listener failure; two failures retain both causes.
fn transfer_completion(
    operation: Result<crate::developer_evidence::MemberExport, HostError>,
    control: std::io::Result<()>,
) -> Result<(crate::developer_evidence::MemberExport, Option<HostError>), HostError> {
    match (operation, control) {
        (Ok(receipt), control) => Ok((receipt, control.err().map(HostError::from))),
        (Err(operation), Ok(())) => Err(operation),
        (Err(operation), Err(control)) => Err(HostError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(control.into()),
        }),
    }
}

/// Dispatch only passive owner reads. Explicit preparation/execution stay separate.
pub(super) fn inspect(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let source_reference = if let Action::TestSelection {
        from_result: Some(reference),
        ..
    } = &action
    {
        Some(ResultReference::decode(reference)?)
    } else {
        None
    };
    let original_root = source_reference
        .as_ref()
        .and_then(|reference| match &reference.locator {
            Locator::Source {
                guard:
                    Some(crate::investigation::SourceGuard {
                        root: Some(root), ..
                    }),
                ..
            } => Some(root.to_path_buf()),
            _ => None,
        });
    let selected = selected.or(original_root.as_deref());
    let plain_source = matches!(
        &action,
        Action::Comparison(_)
            | Action::Outline(_)
            | Action::Declaration(_)
            | Action::Origin(_)
            | Action::Artifact { .. }
            | Action::Schema(_)
            | Action::Payload(_)
            | Action::DeveloperEvidenceBundle { .. }
            | Action::DeveloperEvidenceMember { .. }
    );
    let root = if plain_source {
        selected.map_or_else(std::env::current_dir, std::path::absolute)?
    } else {
        super::super::checkout_root(selected)?
    };
    match action {
        Action::Settings { options, target } => {
            crate::cli::product::context::explain(&root, &options, target, output)
        }
        Action::Comparison(args) => super::comparison::dispatch(args, selected, output),
        Action::Schema(args) => schema::inspect(args, &root, output),
        Action::Payload(mut args) => {
            args.descriptor = root.join(args.descriptor);
            args.input = root.join(args.input);
            output
                .emit(Envelope::response(
                    "inspect payload",
                    "passive schema-bound payload explanation",
                    crate::schema::payload::inspect(&args)?,
                ))
                .map_err(Into::into)
        }
        Action::DeveloperEvidence {
            from_result,
            session,
            output: destination,
            scope,
            layout,
        } => {
            if layout == EvidenceLayout::Directory
                && scope != crate::test_workflow::execute::EvidenceScope::Full
            {
                return Err(HostError::Ownership(
                    "directory export requires explicit full dependency scope".into(),
                ));
            }
            let store = super::super::invocation_store(selected)?;
            let session =
                crate::invocation::select_session(session, &std::env::vars_os().collect())?;
            if layout == EvidenceLayout::Directory {
                let payload = crate::developer_evidence::export_directory(
                    &store,
                    &from_result,
                    session.as_deref(),
                    &root.join(destination),
                )?;
                return output
                    .emit(Envelope::response(
                        "inspect developer-evidence",
                        "exported original native directory observations",
                        payload,
                    ))
                    .map_err(Into::into);
            }
            let payload = crate::developer_evidence::export(
                &store,
                &from_result,
                session.as_deref(),
                &root.join(destination),
                scope,
            )?;
            output
                .emit(Envelope::response(
                    "inspect developer-evidence",
                    "exported original native observations",
                    payload,
                ))
                .map_err(Into::into)
        }
        Action::DeveloperEvidenceMember {
            input,
            member,
            output: destination,
        } => {
            let input = root.join(input);
            let destination = root.join(destination);
            let (payload, control_failure) = super::super::runtime()?.block_on(async {
                let cancel = tokio_util::sync::CancellationToken::new();
                let worker_cancel = cancel.clone();
                let mut operation = tokio::task::spawn_blocking(move ||
                    crate::developer_evidence::export_directory_member(
                        &input, member, &destination, &worker_cancel,
                    ));
                tokio::select! {
                    result = &mut operation => transfer_completion(
                        result.map_err(HostError::from).and_then(std::convert::identity), Ok(()),
                    ),
                    signal = tokio::signal::ctrl_c() => {
                        cancel.cancel();
                        let settled = operation.await.map_err(HostError::from).and_then(std::convert::identity);
                        transfer_completion(settled, signal)
                    }
                }
            })?;
            let mut response = Envelope::response(
                "inspect developer-evidence-member",
                "published one verified original member",
                payload,
            );
            if let Some(error) = control_failure {
                response.failures.push(ErrorRecord::from_typed(
                    &error,
                    &RedactionPolicy::from_environment(),
                ));
                response.outcome = crate::envelope::CommandOutcome::Failure;
                response.observation.completeness = Completeness::Partial;
                response.observation.limitations.push("Original member publication completed; signal observation failed. The published receipt remains valid; do not replay the publication merely because control observation failed.".into());
            }
            output.emit(response).map_err(Into::into)
        }
        Action::DeveloperEvidenceBundle {
            input,
            member,
            offset,
            length,
        } => {
            let payload = crate::developer_evidence::inspect_directory(
                &root.join(input),
                member,
                offset,
                length,
            )?;
            output
                .emit(Envelope::response(
                    "inspect developer-evidence-bundle",
                    "verified original offline directory observations",
                    payload,
                ))
                .map_err(Into::into)
        }
        Action::Outline(options) => source::outline(options, selected, output),
        Action::Declaration(options) => source::declaration(options, selected, output),
        Action::Origin(options) => source::origin(options, selected, output),
        Action::Entity(options) => source::entity(options, selected, output),
        Action::DependencyUsage(options) => dependency_usage::inspect(options, &root, output),
        Action::Api(options) => crate::cli::api::inspect(options, &root, output),
        Action::Dependency {
            package,
            metadata,
            why,
            feature_member,
        } => {
            let supplied = metadata.is_some();
            let snapshot = if let Some(path) = metadata {
                crate::catalog::MetadataSnapshot::read(&super::super::source_path(selected, path)?)?
            } else {
                crate::dependency::capture(&root, &std::env::vars_os().collect())?
            };
            let mut response = crate::dependency::inspect_details(
                &snapshot,
                &package,
                if supplied {
                    "explicit_retained_snapshot"
                } else {
                    "locked_offline_metadata"
                },
                why,
                feature_member.as_deref(),
            );
            response.command = "inspect dependency".into();
            Ok(output.emit(response)?)
        }
        Action::DependencyUnits {
            dependency,
            selection,
            native_world,
            metadata_original,
        } => super::super::dependency::dispatch(
            super::super::dependency::Action::Units {
                dependency,
                selection,
                native_world,
                metadata_original,
            },
            selected,
            output,
        ),
        Action::Owner {
            path,
            metadata,
            xtask_selection,
        } => owner(&root, path, metadata, xtask_selection, selected, output),
        Action::Diagnostic {
            invocation,
            address,
            cursor,
            session,
            raw,
            source,
            max_record_bytes,
        } => {
            let store = super::super::invocation_store(selected)?;
            let limit =
                NonZeroUsize::new(output.presentation.budget.unwrap_or(20)).ok_or_else(|| {
                    HostError::Ownership("diagnostic result allowance must be positive".into())
                })?;
            if let Some(address) = address {
                let address = serde_json::from_str(&address).map_err(std::io::Error::other)?;
                let cursor = cursor
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()
                    .map_err(std::io::Error::other)?;
                let evidence =
                    crate::compiler_investigation::diagnostic(&store, &address, max_record_bytes)?;
                let mut response = evidence.page(&root, cursor.as_ref(), limit, raw, source)?;
                response.command = "inspect diagnostic detail".into();
                let omitted = response
                    .payload
                    .total_related_records
                    .saturating_sub(response.payload.records.len());
                Ok(output.emit_page(
                    response,
                    crate::envelope::DetailView {
                        budget: Some(limit.get()),
                        omitted_records: omitted,
                        truncated: omitted > 0,
                        ..Default::default()
                    },
                )?)
            } else {
                let invocation = invocation.ok_or_else(|| {
                    HostError::Ownership("select an invocation or exact diagnostic address".into())
                })?;
                let session =
                    crate::invocation::select_session(session, &std::env::vars_os().collect())?;
                let cursor = cursor
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()
                    .map_err(std::io::Error::other)?;
                let mut response = crate::compiler_investigation::diagnostics(
                    &store,
                    &invocation,
                    session.as_deref(),
                    cursor.as_ref(),
                    limit,
                    &root,
                )?;
                response.command = "inspect diagnostic".into();
                let omitted = response
                    .payload
                    .retained_references
                    .saturating_sub(response.payload.records.len());
                Ok(output.emit_page(
                    response,
                    crate::envelope::DetailView {
                        budget: Some(limit.get()),
                        omitted_records: omitted,
                        truncated: omitted > 0,
                        ..Default::default()
                    },
                )?)
            }
        }
        Action::Artifact {
            file,
            offset,
            length,
            all,
            if_match,
        } => {
            let file = super::super::source_path(selected, file)?;
            let request = crate::artifacts::ReadRequest {
                offset,
                length: if all {
                    None
                } else {
                    length.or_else(|| crate::artifacts::ReadRequest::default().length)
                },
                expected_etag: if_match,
            };
            let payload =
                super::super::runtime()?.block_on(crate::artifacts::read_file(&file, &request))?;
            let mut response = Envelope::response(
                "inspect artifact",
                "original selected artifact byte range",
                payload,
            );
            response.observation.scope = "selected_artifact_byte_range".into();
            response.observation.limitations.push("ETag is an opaque native validator; a selected range does not establish immutable whole-object identity".into());
            if let Some(offset) = response.payload.next_offset {
                let file = std::path::absolute(file)?;
                let path = file.to_str().ok_or_else(|| {
                    HostError::Ownership(
                        "native object-store continuation requires its Unicode filesystem key"
                            .into(),
                    )
                })?;
                let mut argv = vec![
                    "pse".into(),
                    "inspect".into(),
                    "artifact".into(),
                    path.into(),
                    "--offset".into(),
                    offset.to_string(),
                ];
                if let Some(length) = request.length {
                    argv.extend(["--length".into(), length.to_string()]);
                } else {
                    argv.push("--all".into());
                }
                if let Some(etag) = &response.payload.etag {
                    argv.extend(["--if-match".into(), etag.clone()]);
                }
                response.next.push(crate::envelope::NextAction::Command {
                    argv,
                    reason: "continue the selected native byte range under its original validator"
                        .into(),
                });
            }
            super::super::source_emit(output, response, super::super::artifact_text)
        }
        Action::CompilerArtifact {
            invocation,
            file,
            offset,
            length,
        } => {
            let state =
                crate::paths::CheckoutState::resolve(&root, &std::env::vars_os().collect())?;
            let mut response = crate::compiler_investigation::artifact::read_file(
                &root,
                &state,
                &invocation,
                file,
                offset,
                length,
            )?;
            response.command = "inspect compiler-artifact".into();
            Ok(output.emit(response)?)
        }
        Action::CompilerArtifacts {
            invocation,
            metadata,
        } => {
            if let Some(metadata) = metadata {
                output.validate_for::<super::comparison::ContextInspection>(
                    "inspect compiler-artifacts metadata",
                )?;
                let report = super::comparison::context_inspection(
                    &root,
                    &invocation,
                    &root.join(metadata),
                )?;
                let address = report.reference.encode()?;
                let mut response = Envelope::response(
                    "inspect compiler-artifacts metadata",
                    "explicit retained metadata and original admitted context; association remains qualified",
                    report,
                );
                response.observation.reference = Some(address);
                return Ok(output.emit_page(
                    response,
                    crate::envelope::DetailView {
                        budget: output.presentation.budget,
                        ..Default::default()
                    },
                )?);
            }
            let state =
                crate::paths::CheckoutState::resolve(&root, &std::env::vars_os().collect())?;
            let mut response =
                crate::compiler_investigation::artifact::observe(&root, &state, &invocation)?;
            response.command = "inspect compiler-artifacts".into();
            Ok(output.emit(response)?)
        }
        Action::CompilerSource {
            invocation,
            source_file,
            offset,
            length,
        } => {
            let state =
                crate::paths::CheckoutState::resolve(&root, &std::env::vars_os().collect())?;
            Ok(
                output.emit(crate::compiler_investigation::facts::read_source(
                    &root,
                    &state,
                    &invocation,
                    source_file,
                    offset,
                    length,
                )?)?,
            )
        }
        Action::CompilerFacts {
            invocation,
            body,
            max_bytes,
            fact_focus,
        } => {
            let state =
                crate::paths::CheckoutState::resolve(&root, &std::env::vars_os().collect())?;
            let response = crate::compiler_investigation::facts::observe(
                &root,
                &state,
                &invocation,
                body.as_deref(),
                max_bytes,
                fact_focus,
            )?;
            let omitted = response.payload.omitted_bodies;
            Ok(output.emit_page(
                response,
                crate::envelope::DetailView {
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..Default::default()
                },
            )?)
        }
        Action::Tests {
            invocation,
            session,
            failures,
            cursor,
            limit,
            page_bytes,
        } => {
            let store = super::super::invocation_store(selected)?;
            let session =
                crate::invocation::select_session(session, &std::env::vars_os().collect())?;
            if failures {
                let cursor = cursor
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()
                    .map_err(std::io::Error::other)?;
                let mut response = crate::test_inventory::failures(
                    &store,
                    &invocation,
                    session.as_deref(),
                    cursor.as_ref(),
                    limit,
                    page_bytes,
                )?;
                response.command = "inspect tests failures".into();
                Ok(output.emit(response)?)
            } else {
                let cursor = cursor
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()
                    .map_err(std::io::Error::other)?;
                let mut response = crate::test_inventory::read(
                    &store,
                    &invocation,
                    session.as_deref(),
                    cursor.as_ref(),
                    limit,
                    page_bytes,
                )?;
                response.command = "inspect tests".into();
                Ok(output.emit(response)?)
            }
        }
        Action::TestSelection {
            mut tests,
            from_result: _,
            target_witness,
            doctest_candidate,
            admitted,
            full,
        } => {
            let inputs = std::env::vars_os().collect();
            let mut metadata_root = root.clone();
            let candidate_witness = if let Some(reference) = &source_reference {
                match &reference.locator {
                    Locator::Source {
                        path,
                        source: crate::investigation::SourceKind::Working,
                        ..
                    } => {
                        let witness =
                            crate::investigation::providers::inspect(&root, &reference.locator)?;
                        let reference = witness.reference.ok_or_else(|| {
                            HostError::Ownership(
                                "source inspection did not return a guarded current witness".into(),
                            )
                        })?;
                        Some((path.to_path_buf(), reference))
                    }
                    Locator::Semantic { .. } => {
                        let token = tokio_util::sync::CancellationToken::new();
                        let resolved = crate::cli::runtime()?.block_on(async {
                            let operation = crate::investigation::rust::resolve_entity(&root, &reference.locator, &token);
                            tokio::pin!(operation);
                            tokio::select! {
                                result = &mut operation => result,
                                signal = tokio::signal::ctrl_c() => {
                                    token.cancel();
                                    let result = operation.await;
                                    signal?;
                                    result
                                }
                            }
                        })?;
                        let Locator::Semantic { project, path, .. } = &resolved.locator else {
                            return Err(HostError::Ownership(
                                "native source refinement did not retain its original project/source locator".into(),
                            ));
                        };
                        metadata_root = project.clone();
                        Some((path.to_path_buf(), ResultReference::new(resolved.locator)))
                    }
                    _=>return Err(HostError::Ownership("--from-result requires a guarded working-source/native entity witness; retained revisions and evidence do not establish a current test candidate".into())),
                }
            } else {
                None
            };
            let snapshot = crate::dependency::capture(&metadata_root, &inputs)?;
            let candidate = if let Some((path, reference)) = candidate_witness {
                let workspace_root = snapshot.metadata.workspace_root.as_std_path();
                let owner = crate::ownership::source_owner(workspace_root, &path, Some(&snapshot))?;
                let crate::ownership::PackageOwnership::WorkspacePackage { package_id, .. } =
                    owner.package
                else {
                    return Err(HostError::Ownership("source witness has no current containing workspace package; select tests explicitly".into()));
                };
                if let Some(witness) = &target_witness {
                    tests.verify_source_target(
                        &snapshot.metadata,
                        &package_id,
                        witness,
                        doctest_candidate,
                    )?;
                }
                tests.select_source_package(package_id.clone())?;
                let lane = if doctest_candidate {
                    CandidateLane::Rustdoc
                } else {
                    CandidateLane::Nextest
                };
                Some(SourceCandidate {
                    reference,
                    package_id,
                    lane,
                })
            } else {
                None
            };
            let metadata = snapshot.metadata;
            let request = tests.request()?;
            let next = if candidate.is_some() && admitted {
                tests
                    .admitted_source_argv(&metadata_root, &request)
                    .map(|argv| crate::envelope::NextAction::Command {
                        argv,
                        reason: "run the admitted Cargo target with the selected Nextest filters"
                            .into(),
                    })
            } else {
                None
            };
            let intent =
                crate::test_workflow::Intent::new(metadata, request, tests.profile, inputs)
                    .map_err(|error| HostError::Workflow(Box::new(error)))?;
            if admitted {
                let admitted = intent
                    .admit()
                    .map_err(|error| HostError::Workflow(Box::new(error)))?;
                selection_emit(output, "test", full, admitted.explain(), candidate, next)
            } else {
                let explanation = intent.explain();
                let next = metadata_root.join("justfile").to_str().filter(|_| {
                    doctest_candidate && matches!(
                        explanation.evidence.doctest_route,
                        crate::catalog::description::Information::Available(_)
                    )
                }).map(|justfile| crate::envelope::NextAction::Command {
                    argv: vec!["just".into(), "--justfile".into(), justfile.into(), "doctest".into()],
                    reason: "run the workspace rustdoc doctest lane for eligible declarations; rustdoc selects the examples".into(),
                });
                selection_emit(output, "test", full, explanation, candidate, next)
            }
        }
        Action::CheckSelection {
            selection,
            admitted,
            full,
        } => {
            let inputs = std::env::vars_os().collect();
            let metadata = crate::dependency::capture(&root, &inputs)?.metadata;
            let intent = crate::test_workflow::CheckIntent::new(
                metadata,
                selection.request(crate::selection::Operation::Check)?,
                inputs,
            )
            .map_err(|error| HostError::Workflow(Box::new(error)))?;
            if admitted {
                let admitted = intent
                    .admit()
                    .map_err(|error| HostError::Workflow(Box::new(error)))?;
                selection_emit(output, "check", full, admitted.explain(), None, None)
            } else {
                selection_emit(output, "check", full, intent.explain(), None, None)
            }
        }
        Action::Invocation {
            invocation,
            session,
            detail,
        } => {
            let store = super::super::invocation_store(selected)?;
            let session =
                crate::invocation::select_session(session, &std::env::vars_os().collect())?;
            let response = crate::invocation_view::inspect(
                &store,
                &invocation,
                session.as_deref(),
                detail,
                &RedactionPolicy::from_environment(),
            )?;
            Ok(output.emit_observation(response)?)
        }
        Action::Log {
            invocation,
            session,
            stderr,
            offset,
            length,
            tail,
        } => {
            let store = super::super::invocation_store(selected)?;
            let session =
                crate::invocation::select_session(session, &std::env::vars_os().collect())?;
            let stream = if stderr {
                crate::extraction::OutputStream::Stderr
            } else {
                crate::extraction::OutputStream::Stdout
            };
            let mut response = crate::invocation_view::log(
                &store,
                &invocation,
                session.as_deref(),
                stream,
                offset,
                length,
                tail,
            )?;
            response.command = "inspect log".into();
            Ok(output.emit_observation(response)?)
        }
        Action::Affected { selection } => {
            super::super::affected::dispatch(selection, selected, output)
        }
        Action::Changes { selection } => {
            super::super::changes::dispatch(selection, selected, output)
        }
    }
}

fn selection_emit(
    output: &OutputOptions,
    kind: &str,
    full: bool,
    mut explanation: crate::test_workflow::Explanation<'_>,
    candidate: Option<SourceCandidate>,
    next: Option<crate::envelope::NextAction>,
) -> Result<i32, HostError> {
    let command = format!(
        "inspect {kind}-selection{}",
        if full { " full" } else { "" }
    );
    let summary = format!(
        "native {kind} {:?} intent; no source build, runtime preparation or test listing performed",
        explanation.stage
    );
    let from_source = candidate.is_some();
    if let Some(candidate) = candidate {
        use crate::catalog::description::Information;
        if matches!(candidate.lane, CandidateLane::Rustdoc) {
            let reason = "rustdoc owns doctest execution through just doctest";
            explanation.evidence.nextest_inventory = Information::Unavailable {
                reason: reason.into(),
            };
            explanation.selected_tests = Information::Unavailable {
                reason: reason.into(),
            };
            explanation.runner = None;
            explanation.fixture_requirements = Information::Unavailable {
                reason: reason.into(),
            };
            explanation.pending = vec!["rustdoc_doctest_execution"];
        }
        explanation.evidence.source_candidate = Information::Available(candidate);
    }
    fn mark<T>(
        mut response: Envelope<T>,
        from_source: bool,
        next: Option<crate::envelope::NextAction>,
    ) -> Envelope<T> {
        if from_source {
            response.observation.limitations.push("A validated current source witness supports the candidate. Dependency reach, exact inventory, coverage and execution verdict each require their own evidence.".into());
        }
        response.next.extend(next);
        response
    }
    if full {
        Ok(output.emit(mark(
            Envelope::response(command, summary, explanation),
            from_source,
            next,
        ))?)
    } else {
        Ok(output.emit(mark(
            Envelope::response(command, summary, explanation.brief()),
            from_source,
            next,
        ))?)
    }
}

fn owner(
    root: &Path,
    path: PathBuf,
    metadata: Option<PathBuf>,
    xtask_selection: Option<String>,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let metadata = metadata
        .map(|path| {
            super::super::source_path(selected, path).and_then(|path| {
                crate::catalog::MetadataSnapshot::read(&path).map_err(HostError::from)
            })
        })
        .transpose()?;
    let source = crate::ownership::source_owner(root, &path, metadata.as_ref())?;
    let mut failures = Vec::new();
    let mut document_failure = None;
    let document = match xtask_selection {
        Some(id) => {
            let inputs = std::env::vars_os().collect();
            match crate::prepared_xtask::PreparedNativeTool::open(root,&inputs,&id).and_then(|tool| tool.document_owner(&path,&inputs)) {
                Ok(document) => crate::catalog::description::Information::Available(document),
                Err(error) => { let reason = error.to_string(); document_failure = Some(error); crate::catalog::description::Information::Unavailable {reason} }
            }
        }
        None => crate::catalog::description::Information::Unavailable {reason:"no prepared native document interpreter selected; explicit tools prepare xtask is available".into()},
    };
    let generated = match crate::ownership::query(root, &path) {
        Ok(mut query) => {
            failures.append(&mut query.failures);
            crate::catalog::description::Information::Available(query)
        }
        Err(error) => {
            let reason = error.to_string();
            failures.push(error);
            crate::catalog::description::Information::Unavailable { reason }
        }
    };
    let mut response = Envelope::response(
        "inspect owner",
        "source and generated-output ownership observed independently",
        super::super::OwnerView {
            source,
            generated,
            document,
        },
    );
    response.observation.scope = "source_package_and_generated_output_routing".into();
    response.observation.limitations = response.payload.source.coverage_limits.clone();
    let policy = RedactionPolicy::from_environment();
    if let Some(error) = document_failure {
        response
            .failures
            .push(ErrorRecord::from_typed(&error, &policy));
    }
    response.failures.extend(
        failures
            .iter()
            .map(|error| ErrorRecord::from_typed(error, &policy)),
    );
    if !response.failures.is_empty() {
        response.observation.completeness = Completeness::Partial;
    }
    Ok(output.emit(response)?)
}

/// Decode an explicitly typed evidence locator through its existing original owner.
pub(super) fn inspect_locator(
    locator: &Locator,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let Locator::Evidence {
        invocation,
        kind,
        selector,
    } = locator
    else {
        return Err(HostError::Ownership(
            "expected a typed original-evidence locator".into(),
        ));
    };
    let action = match kind {
        EvidenceKind::Diagnostic => {
            if let Some(address) = selector {
                let address: crate::compiler_investigation::DiagnosticAddress =
                    serde_json::from_str(address).map_err(std::io::Error::other)?;
                if address.invocation.as_str() != invocation {
                    return Err(HostError::Ownership(
                        "diagnostic locator and original address identify different invocations"
                            .into(),
                    ));
                }
            }
            Action::Diagnostic {
                invocation: if selector.is_none() {
                    Some(invocation.clone())
                } else {
                    None
                },
                address: selector.clone(),
                cursor: None,
                session: None,
                raw: true,
                source: false,
                max_record_bytes: NonZeroUsize::new(4194304)
                    .ok_or_else(|| HostError::Ownership("invalid record allowance".into()))?,
            }
        }
        EvidenceKind::TestPopulation => Action::Tests {
            invocation: invocation.clone(),
            session: None,
            failures: false,
            cursor: selector.clone(),
            limit: NonZeroUsize::new(20)
                .ok_or_else(|| HostError::Ownership("invalid page allowance".into()))?,
            page_bytes: NonZeroUsize::new(262144)
                .ok_or_else(|| HostError::Ownership("invalid byte allowance".into()))?,
        },
        EvidenceKind::Log => {
            let stream = selector.as_deref().unwrap_or("stdout");
            if !matches!(stream, "stdout" | "stderr") {
                return Err(HostError::Ownership(
                    "log locator selector must be stdout or stderr".into(),
                ));
            }
            Action::Log {
                invocation: invocation.clone(),
                session: None,
                stderr: stream == "stderr",
                offset: 0,
                length: NonZeroU64::new(4096)
                    .ok_or_else(|| HostError::Ownership("invalid range allowance".into()))?,
                tail: false,
            }
        }
        EvidenceKind::Artifact => {
            let file = selector.as_deref().and_then(|value|value.strip_prefix("compiler:")).and_then(|value|value.parse().ok()).ok_or_else(||HostError::Ownership("original compiler artifact locator requires compiler:ORDINAL; explicit files use inspect artifact".into()))?;
            Action::CompilerArtifact {
                invocation: invocation.clone(),
                file,
                offset: 0,
                length: NonZeroU64::new(4096)
                    .ok_or_else(|| HostError::Ownership("invalid range allowance".into()))?,
            }
        }
    };
    inspect(action, selected, output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[derive(Debug, Parser)]
    struct Parsed {
        #[command(subcommand)]
        action: Action,
    }
    #[test]
    fn developer_export_scope_is_explicit_with_full_default_and_no_report_only_shortcut() {
        use crate::test_workflow::execute::EvidenceScope;
        let base = [
            "inspect",
            "developer-evidence",
            "--from-result",
            "original",
            "--output",
            "portable.pb",
        ];
        let Action::DeveloperEvidence { scope, .. } = Parsed::try_parse_from(base).unwrap().action
        else {
            panic!("export action")
        };
        assert_eq!(scope, EvidenceScope::Full);
        for (name, expected) in [
            ("report", EvidenceScope::Report),
            ("selection", EvidenceScope::Selection),
            ("full", EvidenceScope::Full),
        ] {
            let args: Vec<_> = base.into_iter().chain(["--scope", name]).collect();
            let Action::DeveloperEvidence { scope, .. } =
                Parsed::try_parse_from(args).unwrap().action
            else {
                panic!("export action")
            };
            assert_eq!(scope, expected);
        }
        assert!(Parsed::try_parse_from(base.into_iter().chain(["--report-only"])).is_err());
        assert!(Parsed::try_parse_from(base.into_iter().chain(["--scope", "complete"])).is_err());
    }
    #[test]
    fn directory_profile_and_offline_range_are_explicit_without_compact_fallback() {
        let parsed = Parsed::try_parse_from([
            "inspect",
            "developer-evidence",
            "--from-result",
            "original",
            "--output",
            "portable",
            "--layout",
            "directory",
        ])
        .unwrap();
        assert!(matches!(
            parsed.action,
            Action::DeveloperEvidence {
                layout: EvidenceLayout::Directory,
                scope: crate::test_workflow::execute::EvidenceScope::Full,
                ..
            }
        ));
        assert!(
            Parsed::try_parse_from(["inspect", "developer-evidence-bundle", "portable"]).is_ok()
        );
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "developer-evidence-bundle",
                "portable",
                "--member",
                "1",
                "--offset",
                "8",
                "--length",
                "4"
            ])
            .is_ok()
        );
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "developer-evidence-bundle",
                "portable",
                "--offset",
                "8"
            ])
            .is_err()
        );
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "developer-evidence-bundle",
                "portable",
                "--member",
                "1",
                "--length",
                "0"
            ])
            .is_err()
        );
    }
    #[test]
    fn directory_cli_dispatch_reads_a_moved_bundle_without_a_checkout() {
        let scratch = tempfile::tempdir().unwrap();
        let (bundle, _) =
            crate::developer_evidence::tests::directory::exported(scratch.path(), false);
        let moved = scratch.path().join("recipient");
        std::fs::rename(bundle, &moved).unwrap();
        let action = Parsed::try_parse_from([
            "inspect",
            "developer-evidence-bundle",
            "recipient",
            "--member",
            "1",
            "--offset",
            "4",
            "--length",
            "3",
        ])
        .unwrap()
        .action;
        let output = OutputOptions {
            format: crate::envelope::OutputFormat::Json,
            presentation: Default::default(),
            invocation_id: crate::envelope::CliInvocationId::new(),
            artifact_directory: None,
        };
        assert_eq!(inspect(action, Some(scratch.path()), &output).unwrap(), 0);
    }
    #[test]
    fn directory_sequential_member_cli_publishes_exact_original_without_checkout() {
        let scratch = tempfile::tempdir().unwrap();
        let (bundle, _) =
            crate::developer_evidence::tests::directory::exported(scratch.path(), false);
        let moved = scratch.path().join("recipient");
        std::fs::rename(bundle, &moved).unwrap();
        let action = Parsed::try_parse_from([
            "inspect",
            "developer-evidence-member",
            "recipient",
            "--member",
            "1",
            "--output",
            "copied.stdout",
        ])
        .unwrap()
        .action;
        let output = OutputOptions {
            format: crate::envelope::OutputFormat::Json,
            presentation: Default::default(),
            invocation_id: crate::envelope::CliInvocationId::new(),
            artifact_directory: None,
        };
        assert_eq!(inspect(action, Some(scratch.path()), &output).unwrap(), 0);
        assert_eq!(
            std::fs::read(scratch.path().join("copied.stdout")).unwrap(),
            [0; 16]
        );
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "developer-evidence-member",
                "recipient",
                "--output",
                "other"
            ])
            .is_err()
        );
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "developer-evidence-member",
                "recipient",
                "--member",
                "1"
            ])
            .is_err()
        );
    }
    #[test]
    fn directory_sequential_member_control_failure_keeps_success_and_independent_failures() {
        let scratch = tempfile::tempdir().unwrap();
        let (bundle, _) =
            crate::developer_evidence::tests::directory::exported(scratch.path(), false);
        let output = scratch.path().join("published");
        let receipt = crate::developer_evidence::export_directory_member(
            &bundle,
            1,
            &output,
            &tokio_util::sync::CancellationToken::new(),
        )
        .unwrap();
        let (receipt, control) = transfer_completion(
            Ok(receipt),
            Err(std::io::Error::other("signal listener failed")),
        )
        .unwrap();
        assert_eq!(receipt.copied.content.byte_length, 16);
        assert_eq!(std::fs::read(&output).unwrap(), [0; 16]);
        assert!(
            matches!(control, Some(HostError::Io(ref error)) if error.to_string() == "signal listener failed")
        );
        let error = transfer_completion(
            Err(HostError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "original sink",
            ))),
            Err(std::io::Error::other("original listener")),
        )
        .unwrap_err();
        let HostError::Settlement { operation, cleanup } = error else {
            panic!("lost independent outcomes")
        };
        assert!(
            matches!(*operation, HostError::Io(ref error) if error.kind() == std::io::ErrorKind::BrokenPipe)
        );
        assert!(
            matches!(*cleanup, HostError::Io(ref error) if error.to_string() == "original listener")
        );
    }
    #[test]
    fn test_intent_preserves_native_selection_and_exact_filter_authority() {
        let parsed = Parsed::try_parse_from([
            "inspect",
            "test-selection",
            "pse-host",
            "-E",
            "test(search)",
            "--without-feature",
            "native-solvers",
            "--",
            "literal",
        ])
        .unwrap();
        let Action::TestSelection { tests, .. } = parsed.action else {
            panic!("wrong query");
        };
        let request = tests.request().unwrap();
        assert_eq!(request.filters.words, ["literal"]);
        assert_eq!(request.filters.expressions, ["test(search)"]);
        let crate::selection::Scope::Packages(packages) = request.scope else {
            panic!("lost selected packages");
        };
        assert_eq!(packages[0].package, "pse-host");
        assert!(packages[0].features.absent.contains("native-solvers"));
    }
    #[test]
    fn doctest_candidate_requires_current_doctest_target_before_routing() {
        let witness = serde_json::json!({
            "package_id":"fixture","manifest":"/workspace/Cargo.toml","root":"/workspace/src/lib.rs",
            "name":"fixture","kind":"library","required_features":[],"active_features":[],
            "required_features_satisfied":true
        }).to_string();
        let arguments = [
            "inspect", "test-selection", "--from-result", "guarded", "--lib",
            "--target-witness", &witness, "--doctest-candidate",
        ];
        let parsed = Parsed::try_parse_from(arguments).unwrap();
        assert!(Parsed::try_parse_from(arguments.into_iter().chain(["--admitted"])).is_err());
        let Action::TestSelection { tests, doctest_candidate, .. } = parsed.action else {
            panic!("wrong query");
        };
        assert!(doctest_candidate);
        let mut metadata: cargo_metadata::Metadata = serde_json::from_value(serde_json::json!({
            "packages":[{"name":"fixture","version":"0.0.0","id":"fixture","manifest_path":"/workspace/Cargo.toml","dependencies":[],"targets":[{"name":"fixture","kind":["lib"],"crate_types":["lib"],"src_path":"/workspace/src/lib.rs","edition":"2024","test":false,"doctest":true}],"features":{}}],
            "workspace_members":["fixture"],"workspace_default_members":["fixture"],
            "workspace_root":"/workspace","target_directory":"/workspace/target","version":1
        })).unwrap();
        tests.verify_source_target(&metadata, "fixture", &witness, true).unwrap();
        metadata.packages[0].targets[0].doctest = false;
        assert!(tests.verify_source_target(&metadata, "fixture", &witness, true).is_err());
    }
    #[test]
    fn rerun_retains_native_policy_and_refuses_feature_override() {
        let parsed =
            Parsed::try_parse_from(["inspect", "test-selection", "--rerun-failed", "original"])
                .unwrap();
        let Action::TestSelection { tests, .. } = parsed.action else {
            panic!("wrong query");
        };
        assert!(
            matches!(tests.request().unwrap().scope,crate::selection::Scope::RerunFailed {invocation} if invocation=="original")
        );
        let parsed = Parsed::try_parse_from([
            "inspect",
            "test-selection",
            "--rerun-failed",
            "original",
            "--features",
            "native-solvers",
        ])
        .unwrap();
        let Action::TestSelection { tests, .. } = parsed.action else {
            panic!("wrong query");
        };
        assert!(tests.request().is_err());
    }
    #[test]
    fn diagnostic_selection_requires_an_original_identity_and_never_generates() {
        assert!(Parsed::try_parse_from(["inspect", "diagnostic"]).is_err());
        assert!(
            Parsed::try_parse_from(["inspect", "diagnostic", "original", "--address", "{}"])
                .is_err()
        );
        assert!(Parsed::try_parse_from(["inspect", "diagnostic", "original", "--raw"]).is_ok());
        assert!(
            Parsed::try_parse_from(["inspect", "diagnostic", "--address", "{}", "--source"])
                .is_ok()
        );
        assert!(Parsed::try_parse_from(["inspect", "generate", "pse-host"]).is_err());
    }
    #[test]
    fn passive_api_reads_have_no_acquisition_or_execution_switch() {
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "api",
                "tokio",
                "--search",
                "spawn",
                "--docs",
                "--examples",
                "--declaration"
            ])
            .is_ok()
        );
        assert!(Parsed::try_parse_from(["inspect", "api", "tokio", "--prepare"]).is_err());
        assert!(Parsed::try_parse_from(["inspect", "api", "tokio", "--check-docs"]).is_err());
        assert!(
            Parsed::try_parse_from([
                "inspect",
                "api",
                "tokio",
                "--compare-with",
                "previous.json",
                "--compatibility"
            ])
            .is_err()
        );
    }
    #[test]
    fn public_investigation_intents_replace_legacy_passive_routes() {
        use crate::cli::{Cli, GenerationIntent};
        for arguments in [
            vec!["pse", "inspect", "owner", "source.rs"],
            vec!["pse", "inspect", "dependency", "tokio"],
            vec![
                "pse",
                "inspect",
                "dependency-units",
                "tokio",
                "pse-host",
                "--lib",
            ],
            vec!["pse", "inspect", "invocation", "original"],
            vec!["pse", "inspect", "log", "original"],
            vec!["pse", "inspect", "compiler-artifacts", "original"],
            vec!["pse", "inspect", "compiler-artifact", "original", "0"],
            vec!["pse", "inspect", "changes"],
            vec!["pse", "inspect", "affected"],
            vec!["pse", "inspect", "entity", "source.rs", "--view", "hints"],
            vec!["pse", "propose", "rename", "source.rs", "--name", "renamed"],
            vec![
                "pse",
                "propose",
                "syntax",
                "source.rs",
                "--pattern",
                "old($ARG)",
                "--replacement",
                "new($ARG)",
            ],
        ] {
            let parsed = Cli::try_parse_from(arguments.clone())
                .unwrap_or_else(|error| panic!("{arguments:?}: {error}"));
            assert_eq!(
                parsed.command.unwrap().generation_intent(),
                GenerationIntent::Passive
            );
        }
        for arguments in [
            vec!["pse", "owner", "source.rs"],
            vec!["pse", "dep", "tokio"],
            vec!["pse", "result", "original"],
            vec!["pse", "log", "original"],
            vec!["pse", "explain", "test", "pse-host"],
            vec!["pse", "changes"],
            vec!["pse", "affected"],
            vec![
                "pse",
                "code",
                "query",
                "source.rs",
                "--mode",
                "syntax",
                "--pattern",
                "old($ARG)",
            ],
            vec!["pse", "compiler", "diagnostics", "original"],
            vec!["pse", "api", "tokio"],
        ] {
            assert!(
                Cli::try_parse_from(arguments.clone()).is_err(),
                "legacy route survived: {arguments:?}"
            );
        }
    }
}
