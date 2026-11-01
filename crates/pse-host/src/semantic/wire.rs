// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native domain mappings. Only original provider documents use opaque JSON bytes.
mod graph;
mod investigation;
use super::*;
use crate::{
    code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb,
    source_read::{SourceIdentity, SourceOrigin, SourceText},
};
pub(super) use graph::answer_from_pb as graph_answer_from_pb;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    os::unix::ffi::{OsStrExt as _, OsStringExt as _},
    path::{Path, PathBuf},
};

fn malformed(field: &str) -> Error {
    Error::Schema(format!("invalid native wire field: {field}"))
}
fn required<T>(value: Option<T>, field: &str) -> Result<T> {
    value.ok_or_else(|| malformed(field))
}
fn size(value: u64, field: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| malformed(field))
}
fn number(value: usize) -> u64 {
    value as u64
}
fn path(value: &Path) -> Vec<u8> {
    value.as_os_str().as_bytes().to_vec()
}
fn from_path(value: Vec<u8>) -> PathBuf {
    OsString::from_vec(value).into()
}
fn opaque(value: &serde_json::Value) -> Result<pb::OriginalEvidence> {
    // Provider identity belongs to the enclosing session/status. A bare native document
    // supplies no separate identity that this representation could truthfully manufacture.
    Ok(pb::OriginalEvidence {
        data: serde_json::to_vec(value)?,
        media_type: "application/json".into(),
        ..Default::default()
    })
}
fn from_opaque(value: pb::OriginalEvidence) -> Result<serde_json::Value> {
    if value.media_type != "application/json" {
        return Err(malformed("original.media_type"));
    }
    Ok(serde_json::from_slice(&value.data)?)
}
fn configuration_to_pb(value: Configuration) -> pb::NativeConfiguration {
    pb::NativeConfiguration {
        backend: match value.backend {
            Backend::Lsp => pb::native_configuration::Backend::LiveLsp,
            Backend::NativeSnapshot => pb::native_configuration::Backend::NativeSnapshot,
        }
        .into(),
        project: path(&value.project),
        features: value.features,
        no_default_features: value.no_default_features,
        target: value.target,
        build_macros: value.build_macros,
        test_context: value.test_context,
        ..Default::default()
    }
}
fn configuration_from_pb(value: pb::NativeConfiguration) -> Result<Configuration> {
    Ok(Configuration {
        backend: match value
            .backend
            .as_known()
            .ok_or_else(|| malformed("configuration.backend"))?
        {
            pb::native_configuration::Backend::LiveLsp => Backend::Lsp,
            pb::native_configuration::Backend::NativeSnapshot => Backend::NativeSnapshot,
        },
        project: from_path(value.project),
        features: value.features,
        no_default_features: value.no_default_features,
        target: value.target,
        build_macros: value.build_macros,
        test_context: value.test_context,
    })
}
fn analysis_to_pb(value: AnalysisContext) -> Result<pb::NativeAnalysisContext> {
    Ok(pb::NativeAnalysisContext {
        backend: value.backend,
        configuration: configuration_to_pb(value.configuration).into(),
        position_encoding: value.position_encoding,
        backend_status: value
            .backend_status
            .as_ref()
            .map(opaque)
            .transpose()?
            .into(),
        preparation: value.preparation,
        ..Default::default()
    })
}
fn analysis_from_pb(mut value: pb::NativeAnalysisContext) -> Result<AnalysisContext> {
    Ok(AnalysisContext {
        backend: value.backend,
        configuration: configuration_from_pb(required(
            value.configuration.take(),
            "session.configuration",
        )?)?,
        position_encoding: value.position_encoding,
        backend_status: value.backend_status.take().map(from_opaque).transpose()?,
        preparation: value.preparation,
    })
}
pub(super) fn status_to_pb(value: SessionStatus) -> Result<pb::NativeSessionStatus> {
    Ok(pb::NativeSessionStatus {
        configuration: configuration_to_pb(value.configuration).into(),
        backend: value.backend,
        position_encoding: value.position_encoding,
        generation: value.generation,
        server_capabilities: opaque(&value.server_capabilities)?.into(),
        backend_status: value
            .backend_status
            .as_ref()
            .map(opaque)
            .transpose()?
            .into(),
        active_requests: number(value.active_requests),
        context_current: value.context_current,
        context_issue: value.context_issue,
        automatic_reload_disabled: value.automatic_reload_disabled,
        preparation_issue: value.preparation_issue,
        preparation: value.preparation,
        ..Default::default()
    })
}
pub(super) fn status_from_pb(mut value: pb::NativeSessionStatus) -> Result<SessionStatus> {
    Ok(SessionStatus {
        configuration: configuration_from_pb(required(
            value.configuration.take(),
            "status.configuration",
        )?)?,
        backend: value.backend,
        position_encoding: value.position_encoding,
        generation: value.generation,
        server_capabilities: from_opaque(required(
            value.server_capabilities.take(),
            "status.server_capabilities",
        )?)?,
        backend_status: value.backend_status.take().map(from_opaque).transpose()?,
        active_requests: size(value.active_requests, "status.active_requests")?,
        context_current: value.context_current,
        context_issue: value.context_issue,
        automatic_reload_disabled: value.automatic_reload_disabled,
        preparation_issue: value.preparation_issue,
        preparation: value.preparation,
    })
}
pub(super) fn request_to_pb(value: &Request) -> Result<pb::NativeRequest> {
    use pb::native_query::Kind as K;
    let operation = match value {
        Request::Graph(value) => {
            pb::native_request::Operation::Graph(Box::new(graph::query_to_pb(value)?))
        }
        Request::Query(value) => pb::native_request::Operation::Query(Box::new(pb::NativeQuery {
            file: path(&value.file),
            line: number(value.line),
            column: number(value.column),
            if_source: value.if_source.clone(),
            if_generation: value.if_generation,
            if_candidate: value.if_candidate.clone(),
            limit: number(value.limit),
            end_line: value.end_line.map(number),
            kind: Some(match &value.kind {
                Kind::Body => K::Body(Default::default()),
                Kind::Origins { kind } => K::Origins(investigation::origin_to_pb(*kind).into()),
                Kind::Compatibility {
                    against,
                    against_generation,
                    operation,
                } => K::Compatibility(Box::new(pb::NativeCompatibilityQuery {
                    against: investigation::target_to_pb(against).into(),
                    operation: investigation::operation_to_pb(*operation).into(),
                    against_generation: *against_generation,
                    ..Default::default()
                })),
                Kind::Visibility {
                    destination,
                    destination_generation,
                } => K::Visibility(Box::new(pb::NativeVisibilityQuery {
                    destination: investigation::target_to_pb(destination).into(),
                    destination_generation: *destination_generation,
                    ..Default::default()
                })),

                Kind::Explain => K::Explain(Default::default()),
                Kind::Layout => K::Layout(Default::default()),
                Kind::Refs => K::Refs(Default::default()),
                Kind::Calls => K::Calls(Default::default()),
                Kind::Implementations => K::Implementations(Default::default()),
                Kind::Macro => K::Macro(Default::default()),
                Kind::Tests => K::Tests(Default::default()),
                Kind::Candidates => K::Candidates(Default::default()),
                Kind::Symbols => K::Symbols(Default::default()),
                Kind::Hints => K::Hints(Default::default()),
                Kind::Hint { index } => K::Hint(number(*index)),
                Kind::Ssr { rule } => K::Ssr(rule.clone()),
                Kind::Importable { target } => K::Importable(Box::new(pb::NativeImportTarget {
                    file: path(&target.file),
                    byte_start: number(target.byte_start),
                    byte_end: number(target.byte_end),
                    source_sha256: target.source_sha256.clone(),
                    ..Default::default()
                })),
                Kind::Predicate { text } => K::Predicate(text.clone()),
                Kind::Candidate { index } => K::Candidate(number(*index)),
                Kind::Rename { name } => K::Rename(name.clone()),
                Kind::FileRename { destination } => K::FileRename(path(destination)),
                Kind::Assists => K::Assists(Default::default()),
                Kind::Assist { index } => K::Assist(number(*index)),
            }),
            ..Default::default()
        })),
        Request::WorkspaceSymbols(value) => {
            pb::native_request::Operation::Workspace(Box::new(pb::NativeWorkspaceQuery {
                query: value.query.clone(),
                matching: match value.matching {
                    NativeNameMode::Fuzzy => pb::native_workspace_query::Matching::FuzzyMatch,
                    NativeNameMode::Exact => pb::native_workspace_query::Matching::ExactMatch,
                    NativeNameMode::Prefix => pb::native_workspace_query::Matching::PrefixMatch,
                }
                .into(),
                limit: number(value.limit),
                include_dependencies: value.include_dependencies,
                if_generation: value.if_generation,
                ..Default::default()
            }))
        }
        Request::Status {} | Request::Stop {} | Request::PackageContext { .. } => {
            return Err(malformed("Run.operation requires query or workspace"));
        }
    };
    Ok(pb::NativeRequest {
        operation: Some(operation),
        ..Default::default()
    })
}
pub(super) fn request_from_pb(value: pb::NativeRequest) -> Result<Request> {
    use pb::native_query::Kind as K;
    Ok(match required(value.operation, "request.operation")? {
        pb::native_request::Operation::Graph(value) => {
            Request::Graph(graph::query_from_pb(*value)?)
        }
        pb::native_request::Operation::Query(value) => {
            if value.file.is_empty() || value.line == 0 || value.column == 0 || value.limit == 0 {
                return Err(malformed("query coordinates/file/limit"));
            }
            Request::Query(Query {
                kind: match required(value.kind, "query.kind")? {
                    K::Body(_) => Kind::Body,
                    K::Origins(kind) => Kind::Origins {
                        kind: investigation::origin_from_pb(kind)?,
                    },
                    K::Compatibility(mut q) => Kind::Compatibility {
                        against: investigation::target_from_pb(required(
                            q.against.take(),
                            "compatibility.against",
                        )?)?,
                        against_generation: q.against_generation,
                        operation: investigation::operation_from_pb(q.operation)?,
                    },
                    K::Visibility(mut q) => Kind::Visibility {
                        destination: investigation::target_from_pb(required(
                            q.destination.take(),
                            "visibility.destination",
                        )?)?,
                        destination_generation: q.destination_generation,
                    },

                    K::Explain(_) => Kind::Explain,
                    K::Layout(_) => Kind::Layout,
                    K::Refs(_) => Kind::Refs,
                    K::Calls(_) => Kind::Calls,
                    K::Implementations(_) => Kind::Implementations,
                    K::Macro(_) => Kind::Macro,
                    K::Tests(_) => Kind::Tests,
                    K::Candidates(_) => Kind::Candidates,
                    K::Symbols(_) => Kind::Symbols,
                    K::Hints(_) => Kind::Hints,
                    K::Hint(index) => Kind::Hint {
                        index: size(index, "hint.index")?,
                    },
                    K::Ssr(rule) => Kind::Ssr { rule },
                    K::Importable(target) => {
                        if target.byte_end < target.byte_start {
                            return Err(malformed("importable.range"));
                        }
                        Kind::Importable {
                            target: ImportTarget {
                                file: from_path(target.file),
                                byte_start: size(target.byte_start, "importable.byte_start")?,
                                byte_end: size(target.byte_end, "importable.byte_end")?,
                                source_sha256: target.source_sha256,
                            },
                        }
                    }
                    K::Predicate(text) => Kind::Predicate { text },
                    K::Candidate(index) => Kind::Candidate {
                        index: size(index, "candidate.index")?,
                    },
                    K::Rename(name) => Kind::Rename { name },
                    K::FileRename(destination) => Kind::FileRename {
                        destination: from_path(destination),
                    },
                    K::Assists(_) => Kind::Assists,
                    K::Assist(index) => Kind::Assist {
                        index: size(index, "assist.index")?,
                    },
                },
                file: from_path(value.file),
                line: size(value.line, "query.line")?,
                column: size(value.column, "query.column")?,
                if_source: value.if_source,
                if_generation: value.if_generation,
                if_candidate: value.if_candidate,
                limit: size(value.limit, "query.limit")?,
                end_line: value
                    .end_line
                    .map(|value| size(value, "query.end_line"))
                    .transpose()?,
            })
        }
        pb::native_request::Operation::Workspace(value) => {
            if value.limit == 0 {
                return Err(malformed("workspace.limit"));
            }
            Request::WorkspaceSymbols(WorkspaceQuery {
                query: value.query,
                matching: match value
                    .matching
                    .as_known()
                    .ok_or_else(|| malformed("workspace.matching"))?
                {
                    pb::native_workspace_query::Matching::FuzzyMatch => NativeNameMode::Fuzzy,
                    pb::native_workspace_query::Matching::ExactMatch => NativeNameMode::Exact,
                    pb::native_workspace_query::Matching::PrefixMatch => NativeNameMode::Prefix,
                },
                limit: size(value.limit, "workspace.limit")?,
                include_dependencies: value.include_dependencies,
                if_generation: value.if_generation,
            })
        }
    })
}

fn library_to_pb(value: NativeLibraryProfile) -> pb::NativeLibraryProfile {
    pb::NativeLibraryProfile {
        package_id: value.package_id,
        manifest: path(&value.manifest),
        source_root: path(&value.source_root),
        library_root: path(&value.library_root),
        features: value.features.into_iter().collect(),
        target: value.target,
        role: value.role,
        ..Default::default()
    }
}
fn library_from_pb(value: pb::NativeLibraryProfile) -> NativeLibraryProfile {
    NativeLibraryProfile {
        package_id: value.package_id,
        manifest: from_path(value.manifest),
        source_root: from_path(value.source_root),
        library_root: from_path(value.library_root),
        features: value.features.into_iter().collect(),
        target: value.target,
        role: value.role,
    }
}
fn package_to_pb(value: NativePackageContext) -> pb::NativePackageContext {
    pb::NativePackageContext {
        edition: value.edition,
        cfg: value.cfg,
        package_id: value.package_id,
        manifest: path(&value.manifest),
        source_root: path(&value.source_root),
        workspace_root: path(&value.workspace_root),
        library_root: path(&value.library_root),
        features: value.features.into_iter().collect(),
        target: value.target,
        role: value.role,
        source_hashes: value
            .source_hashes
            .into_iter()
            .map(|(source, digest)| pb::NativeSourceHash {
                path: path(&source),
                source_sha256: digest,
                ..Default::default()
            })
            .collect(),
        dependencies: value.dependencies.into_iter().map(library_to_pb).collect(),
        dependency_issues: value.dependency_issues,
        dependency_exclusions: value
            .dependency_exclusions
            .into_iter()
            .map(|value| match value {
                NativeDependencyExclusion::Sysroot => {
                    pb::native_package_context::Exclusion::Sysroot.into()
                }
            })
            .collect(),
        ..Default::default()
    }
}
fn package_from_pb(value: pb::NativePackageContext) -> Result<NativePackageContext> {
    let mut source_hashes = BTreeMap::new();
    for source in value.source_hashes {
        if source_hashes
            .insert(from_path(source.path), source.source_sha256)
            .is_some()
        {
            return Err(malformed("package duplicate source_hashes"));
        }
    }
    Ok(NativePackageContext {
        edition: value.edition,
        cfg: value.cfg,
        package_id: value.package_id,
        manifest: from_path(value.manifest),
        source_root: from_path(value.source_root),
        workspace_root: from_path(value.workspace_root),
        library_root: from_path(value.library_root),
        features: value.features.into_iter().collect(),
        target: value.target,
        role: value.role,
        source_hashes,
        dependencies: value
            .dependencies
            .into_iter()
            .map(library_from_pb)
            .collect(),
        dependency_issues: value.dependency_issues,
        dependency_exclusions: value
            .dependency_exclusions
            .into_iter()
            .map(|value| match value.as_known() {
                Some(pb::native_package_context::Exclusion::Sysroot) => {
                    Ok(NativeDependencyExclusion::Sysroot)
                }
                None => Err(malformed("package.dependency_exclusions")),
            })
            .collect::<Result<BTreeSet<_>>>()?,
    })
}
pub(super) fn package_answer_to_pb(
    value: PackageContextAnswer,
) -> Result<pb::NativePackageContextAnswer> {
    Ok(pb::NativePackageContextAnswer {
        session: analysis_to_pb(value.session)?.into(),
        generation: value.generation,
        packages: value.packages.into_iter().map(package_to_pb).collect(),
        ..Default::default()
    })
}
pub(super) fn package_answer_from_pb(
    mut value: pb::NativePackageContextAnswer,
) -> Result<PackageContextAnswer> {
    Ok(PackageContextAnswer {
        session: analysis_from_pb(required(value.session.take(), "package_answer.session")?)?,
        generation: value.generation,
        packages: value
            .packages
            .into_iter()
            .map(package_from_pb)
            .collect::<Result<_>>()?,
    })
}
fn position_to_pb(value: ProtocolPosition) -> pb::NativePosition {
    pb::NativePosition {
        line: value.line,
        character: value.character,
        ..Default::default()
    }
}
fn position_from_pb(value: pb::NativePosition) -> ProtocolPosition {
    ProtocolPosition {
        line: value.line,
        character: value.character,
    }
}
fn location_to_pb(value: FactLocation) -> pb::NativeLocation {
    pb::NativeLocation {
        coordinates: Some(match value {
            FactLocation::Bytes {
                path: source,
                start,
                end,
            } => pb::native_location::Coordinates::Bytes(Box::new(pb::NativeByteLocation {
                path: path(&source),
                start: number(start),
                end: number(end),
                ..Default::default()
            })),
            FactLocation::Protocol { uri, range } => {
                pb::native_location::Coordinates::Protocol(Box::new(pb::NativeProtocolLocation {
                    uri,
                    range: pb::NativeRange {
                        start: position_to_pb(range.start).into(),
                        end: position_to_pb(range.end).into(),
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }))
            }
        }),
        ..Default::default()
    }
}
fn location_from_pb(value: pb::NativeLocation) -> Result<FactLocation> {
    Ok(match required(value.coordinates, "location.coordinates")? {
        pb::native_location::Coordinates::Bytes(value) => {
            if value.end < value.start || value.path.is_empty() {
                return Err(malformed("location.byte_range"));
            }
            FactLocation::Bytes {
                path: from_path(value.path),
                start: size(value.start, "location.start")?,
                end: size(value.end, "location.end")?,
            }
        }
        pb::native_location::Coordinates::Protocol(mut value) => {
            let mut range = required(value.range.take(), "location.range")?;
            let start = position_from_pb(required(range.start.take(), "location.range.start")?);
            let end = position_from_pb(required(range.end.take(), "location.range.end")?);
            if (end.line, end.character) < (start.line, start.character) {
                return Err(malformed("location.protocol_range"));
            }
            FactLocation::Protocol {
                uri: value.uri,
                range: ProtocolRange { start, end },
            }
        }
    })
}
fn symbol_to_pb(value: SymbolFact) -> pb::NativeSymbolFact {
    pb::NativeSymbolFact {
        name: value.name,
        kind: value.kind,
        container: value.container,
        location: value.location.map(location_to_pb).into(),
        source_sha256: value.source_sha256,
        ..Default::default()
    }
}
fn symbol_from_pb(mut value: pb::NativeSymbolFact) -> Result<SymbolFact> {
    Ok(SymbolFact {
        name: value.name,
        kind: value.kind,
        container: value.container,
        location: value.location.take().map(location_from_pb).transpose()?,
        source_sha256: value.source_sha256,
    })
}
fn relation_to_pb(value: FactRelation) -> pb::NativeRelation {
    match value {
        FactRelation::Definition => pb::NativeRelation::NativeDefinition,
        FactRelation::TypeDefinition => pb::NativeRelation::NativeTypeDefinition,
        FactRelation::References => pb::NativeRelation::NativeReferences,
        FactRelation::Calls => pb::NativeRelation::NativeCalls,
        FactRelation::Implementations => pb::NativeRelation::NativeImplementations,
    }
}
fn relation_from_pb(value: buffa::EnumValue<pb::NativeRelation>) -> Result<FactRelation> {
    Ok(
        match value
            .as_known()
            .ok_or_else(|| malformed("relationship.relation"))?
        {
            pb::NativeRelation::NativeDefinition => FactRelation::Definition,
            pb::NativeRelation::NativeTypeDefinition => FactRelation::TypeDefinition,
            pb::NativeRelation::NativeReferences => FactRelation::References,
            pb::NativeRelation::NativeCalls => FactRelation::Calls,
            pb::NativeRelation::NativeImplementations => FactRelation::Implementations,
        },
    )
}
fn observed_to_pb(value: ObservedFacts) -> Result<pb::NativeObservedFacts> {
    Ok(pb::NativeObservedFacts {
        investigation: value
            .investigation
            .map(investigation::to_pb)
            .transpose()?
            .into(),
        symbols: value.symbols.into_iter().map(symbol_to_pb).collect(),
        relationships: value
            .relationships
            .into_iter()
            .map(|value| pb::NativeRelationshipFact {
                relation: relation_to_pb(value.relation).into(),
                direction: match value.direction {
                    FactDirection::Outgoing => pb::Direction::Outgoing,
                    FactDirection::Incoming => pb::Direction::Incoming,
                }
                .into(),
                location: location_to_pb(value.location).into(),
                name: value.name,
                read: value.read,
                write: value.write,
                import: value.import,
                test: value.test,
                ..Default::default()
            })
            .collect(),
        predicate: value
            .predicate
            .map(|value| pb::NativePredicateFact {
                outcome: match value.outcome {
                    PredicateOutcome::Holds => pb::NativePredicateOutcome::Holds,
                    PredicateOutcome::NotProven => pb::NativePredicateOutcome::NotProven,
                    PredicateOutcome::Invalid => pb::NativePredicateOutcome::InvalidPredicate,
                    PredicateOutcome::Unsupported => {
                        pb::NativePredicateOutcome::UnsupportedPredicate
                    }
                }
                .into(),
                cause: value.cause,
                ..Default::default()
            })
            .into(),
        importability: value
            .importability
            .map(|value| pb::NativeImportabilityFact {
                visibility: value.visibility,
                importable: value.importable,
                path: value.path,
                limitations: value.limitations,
                ..Default::default()
            })
            .into(),
        candidates: value
            .candidates
            .into_iter()
            .map(|value| pb::NativeLazyCandidate {
                kind: match value.kind {
                    CandidateKind::Hint => pb::NativeCandidateKind::NativeHint,
                    CandidateKind::Completion => pb::NativeCandidateKind::NativeCompletion,
                    CandidateKind::Assist => pb::NativeCandidateKind::NativeAssist,
                }
                .into(),
                index: number(value.index),
                fingerprint: value.fingerprint,
                label: value.label,
                location: value.location.map(location_to_pb).into(),
                ..Default::default()
            })
            .collect(),
        tests: value
            .tests
            .into_iter()
            .map(|value| {
                Ok(pb::NativeTestCandidate {
                    test_id: value.test_id,
                    cfg: value.cfg,
                    relatedness: value.relatedness,
                    eligible: value.eligible,
                    cfg_eligible: value.cfg_eligible,
                    cargo_targets: value
                        .cargo_targets
                        .into_iter()
                        .map(|target| pb::NativeRunnableTarget {
                            package_id: target.package_id,
                            manifest: path(&target.manifest),
                            root: path(&target.root),
                            name: target.name,
                            kind: target.kind,
                            required_features: target.required_features,
                            active_features: target.active_features.into_iter().collect(),
                            required_features_satisfied: target.required_features_satisfied,
                            ..Default::default()
                        })
                        .collect(),
                    snapshot_update: value
                        .snapshot_update
                        .map(|intent| pb::NativeSnapshotUpdate {
                            expect_test: intent.expect_test,
                            insta: intent.insta,
                            snapbox: intent.snapbox,
                            ..Default::default()
                        })
                        .into(),
                    label: value.label,
                    location: value.location.map(location_to_pb).into(),
                    runnable_kind: value.runnable_kind,
                    cargo_args: value.cargo_args,
                    executable_args: value.executable_args,
                    original: opaque(&value.original)?.into(),
                    ..Default::default()
                })
            })
            .collect::<Result<_>>()?,
        ..Default::default()
    })
}
fn observed_from_pb(mut value: pb::NativeObservedFacts) -> Result<ObservedFacts> {
    Ok(ObservedFacts {
        investigation: value
            .investigation
            .take()
            .map(investigation::from_pb)
            .transpose()?,
        symbols: value
            .symbols
            .into_iter()
            .map(symbol_from_pb)
            .collect::<Result<_>>()?,
        relationships: value
            .relationships
            .into_iter()
            .map(|mut value| {
                Ok(RelationshipFact {
                    relation: relation_from_pb(value.relation)?,
                    direction: match value
                        .direction
                        .as_known()
                        .ok_or_else(|| malformed("relationship.direction"))?
                    {
                        pb::Direction::Outgoing => FactDirection::Outgoing,
                        pb::Direction::Incoming => FactDirection::Incoming,
                    },
                    location: location_from_pb(required(
                        value.location.take(),
                        "relationship.location",
                    )?)?,
                    name: value.name,
                    read: value.read,
                    write: value.write,
                    import: value.import,
                    test: value.test,
                })
            })
            .collect::<Result<_>>()?,
        predicate: value
            .predicate
            .take()
            .map(|value| -> Result<PredicateFact> {
                Ok(PredicateFact {
                    outcome: match value
                        .outcome
                        .as_known()
                        .ok_or_else(|| malformed("predicate.outcome"))?
                    {
                        pb::NativePredicateOutcome::Holds => PredicateOutcome::Holds,
                        pb::NativePredicateOutcome::NotProven => PredicateOutcome::NotProven,
                        pb::NativePredicateOutcome::InvalidPredicate => PredicateOutcome::Invalid,
                        pb::NativePredicateOutcome::UnsupportedPredicate => {
                            PredicateOutcome::Unsupported
                        }
                    },
                    cause: value.cause,
                })
            })
            .transpose()?,
        importability: value.importability.take().map(|value| ImportabilityFact {
            visibility: value.visibility,
            importable: value.importable,
            path: value.path,
            limitations: value.limitations,
        }),
        candidates: value
            .candidates
            .into_iter()
            .map(|mut value| {
                Ok(LazyCandidateFact {
                    kind: match value
                        .kind
                        .as_known()
                        .ok_or_else(|| malformed("candidate.kind"))?
                    {
                        pb::NativeCandidateKind::NativeHint => CandidateKind::Hint,
                        pb::NativeCandidateKind::NativeCompletion => CandidateKind::Completion,
                        pb::NativeCandidateKind::NativeAssist => CandidateKind::Assist,
                    },
                    index: size(value.index, "candidate.index")?,
                    fingerprint: value.fingerprint,
                    label: value.label,
                    location: value.location.take().map(location_from_pb).transpose()?,
                })
            })
            .collect::<Result<_>>()?,
        tests: value
            .tests
            .into_iter()
            .map(|mut value| {
                Ok(TestCandidateFact {
                    test_id: value.test_id,
                    cfg: value.cfg,
                    relatedness: value.relatedness,
                    eligible: value.eligible,
                    cfg_eligible: value.cfg_eligible,
                    cargo_targets: value
                        .cargo_targets
                        .into_iter()
                        .map(|target| RunnableTargetFact {
                            package_id: target.package_id,
                            manifest: from_path(target.manifest),
                            root: from_path(target.root),
                            name: target.name,
                            kind: target.kind,
                            required_features: target.required_features,
                            active_features: target.active_features.into_iter().collect(),
                            required_features_satisfied: target.required_features_satisfied,
                        })
                        .collect(),
                    snapshot_update: value.snapshot_update.take().map(|intent| {
                        SnapshotUpdateFact {
                            expect_test: intent.expect_test,
                            insta: intent.insta,
                            snapbox: intent.snapbox,
                        }
                    }),
                    label: value.label,
                    location: value.location.take().map(location_from_pb).transpose()?,
                    runnable_kind: value.runnable_kind,
                    cargo_args: value.cargo_args,
                    executable_args: value.executable_args,
                    original: from_opaque(required(value.original.take(), "test.original")?)?,
                })
            })
            .collect::<Result<_>>()?,
    })
}
fn section_to_pb(value: Section) -> Result<pb::NativeSection> {
    Ok(pb::NativeSection {
        category: value.category,
        evidence: opaque(&value.evidence)?.into(),
        omitted: number(value.omitted),
        handles: value
            .handles
            .into_iter()
            .map(|value| pb::NativeCandidateHandle {
                index: number(value.index),
                fingerprint: value.fingerprint,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    })
}
fn section_from_pb(mut value: pb::NativeSection) -> Result<Section> {
    Ok(Section {
        category: value.category,
        evidence: from_opaque(required(value.evidence.take(), "section.evidence")?)?,
        omitted: size(value.omitted, "section.omitted")?,
        handles: value
            .handles
            .into_iter()
            .map(|value| {
                Ok(CandidateHandle {
                    index: size(value.index, "section.handle.index")?,
                    fingerprint: value.fingerprint,
                })
            })
            .collect::<Result<_>>()?,
    })
}
fn source_to_pb(value: &SourceIdentity) -> pb::NativeSourceIdentity {
    let mut output = pb::NativeSourceIdentity {
        path: path(value.path()),
        source_sha256: value.source_sha256().into(),
        ..Default::default()
    };
    match value.origin() {
        SourceOrigin::Working => {
            output.origin = pb::native_source_identity::Origin::WorkingSource.into()
        }
        SourceOrigin::Index {
            git_dir,
            index_path,
            blob,
        } => {
            output.origin = pb::native_source_identity::Origin::IndexSource.into();
            output.git_dir = Some(git_dir.as_bytes().to_vec());
            output.index_path = Some(index_path.as_bytes().to_vec());
            output.blob = Some(blob.clone());
        }
        SourceOrigin::Blob {
            git_dir,
            revision,
            blob,
        } => {
            output.origin = pb::native_source_identity::Origin::BlobSource.into();
            output.git_dir = Some(git_dir.as_bytes().to_vec());
            output.revision = Some(revision.clone());
            output.blob = Some(blob.clone());
        }
    }
    output
}
fn source_from_pb(value: pb::NativeSourceIdentity) -> Result<SourceIdentity> {
    let origin = match value
        .origin
        .as_known()
        .ok_or_else(|| malformed("source.origin"))?
    {
        pb::native_source_identity::Origin::WorkingSource => {
            if value.git_dir.is_some()
                || value.index_path.is_some()
                || value.revision.is_some()
                || value.blob.is_some()
            {
                return Err(malformed("source.working context"));
            }
            SourceOrigin::Working
        }
        pb::native_source_identity::Origin::IndexSource => {
            if value.revision.is_some() {
                return Err(malformed("source.index revision"));
            }
            SourceOrigin::Index {
                git_dir: crate::repository::GitName::new(&required(
                    value.git_dir,
                    "source.git_dir",
                )?),
                index_path: crate::repository::GitName::new(&required(
                    value.index_path,
                    "source.index_path",
                )?),
                blob: required(value.blob, "source.blob")?,
            }
        }
        pb::native_source_identity::Origin::BlobSource => {
            if value.index_path.is_some() {
                return Err(malformed("source.blob index_path"));
            }
            SourceOrigin::Blob {
                git_dir: crate::repository::GitName::new(&required(
                    value.git_dir,
                    "source.git_dir",
                )?),
                revision: required(value.revision, "source.revision")?,
                blob: required(value.blob, "source.blob")?,
            }
        }
    };
    Ok(SourceIdentity::from_parts(
        from_path(value.path),
        value.source_sha256,
        origin,
    )?)
}

fn proposal_to_pb(value: ChangeProposal) -> Result<pb::NativePreview> {
    use crate::source_proposal::{FileOperation, ProducerContext};
    let producer = match value.producer() {
        ProducerContext::Syntax {
            backend,
            configuration,
        } => pb::NativeProducerContext {
            backend: backend.clone(),
            configuration: configuration.clone(),
            synchronized_generation: None,
            ..Default::default()
        },
        ProducerContext::Semantic {
            backend,
            configuration,
            synchronized_generation,
        } => pb::NativeProducerContext {
            backend: backend.clone(),
            configuration: configuration.clone(),
            synchronized_generation: Some(synchronized_generation.clone()),
            ..Default::default()
        },
    };
    Ok(pb::NativePreview {
        producer: producer.into(),
        sources: value
            .sources()
            .iter()
            .map(|source| source_to_pb(source.identity()))
            .collect(),
        file_operations: value
            .file_operations()
            .iter()
            .map(|value| match value {
                FileOperation::Create {
                    path: destination,
                    text,
                } => pb::NativeFileOperation {
                    kind: pb::native_file_operation::Kind::Create.into(),
                    path: destination.as_bytes().to_vec(),
                    text: text.clone(),
                    ..Default::default()
                },
                FileOperation::Delete { source } => pb::NativeFileOperation {
                    kind: pb::native_file_operation::Kind::Delete.into(),
                    source: source_to_pb(source).into(),
                    ..Default::default()
                },
                FileOperation::Rename {
                    source,
                    destination,
                } => pb::NativeFileOperation {
                    kind: pb::native_file_operation::Kind::Rename.into(),
                    source: source_to_pb(source).into(),
                    destination: destination.as_bytes().to_vec(),
                    ..Default::default()
                },
            })
            .collect(),
        candidates: value
            .candidates()
            .iter()
            .map(|value| pb::NativeEditCandidate {
                source: source_to_pb(value.source()).into(),
                start: number(value.range().start),
                end: number(value.range().end),
                replacement: value.replacement().into(),
                ..Default::default()
            })
            .collect(),
        selected: value.selected().iter().copied().map(number).collect(),
        conflicts: value
            .conflicts()
            .iter()
            .map(|value| pb::NativeConflict {
                first: number(value.first),
                second: number(value.second),
                ..Default::default()
            })
            .collect(),
        limitations: value.limitations().to_vec(),
        unresolved_placeholders: value
            .unresolved_placeholders()
            .iter()
            .map(|value| pb::NativePlaceholder {
                candidate: number(value.candidate),
                description: value.description.clone(),
                ..Default::default()
            })
            .collect(),
        originals: value
            .sources()
            .iter()
            .map(|source| pb::NativeSourceText {
                source: source_to_pb(source.identity()).into(),
                text: source.text().into(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    })
}
fn proposal_from_pb(mut value: pb::NativePreview) -> Result<ChangeProposal> {
    use crate::source_proposal::{
        CandidateConflict, ChangeProposal, EditCandidate, FileOperation, ProducerContext,
        UnresolvedPlaceholder,
    };
    let producer = required(value.producer.take(), "proposal.producer")?;
    let producer = match producer.synchronized_generation {
        Some(synchronized_generation) => ProducerContext::Semantic {
            backend: producer.backend,
            configuration: producer.configuration,
            synchronized_generation,
        },
        None => ProducerContext::Syntax {
            backend: producer.backend,
            configuration: producer.configuration,
        },
    };
    let identities = value
        .sources
        .into_iter()
        .map(source_from_pb)
        .collect::<Result<Vec<_>>>()?;
    let originals = value
        .originals
        .into_iter()
        .map(|mut value| {
            Ok(SourceText::from_identity(
                source_from_pb(required(value.source.take(), "proposal.original.source")?)?,
                value.text,
            )?)
        })
        .collect::<Result<Vec<_>>>()?;
    if identities
        != originals
            .iter()
            .map(|source| source.identity().clone())
            .collect::<Vec<_>>()
    {
        return Err(malformed("proposal.original identities"));
    }
    let candidates = value
        .candidates
        .into_iter()
        .map(|mut value| {
            if value.end < value.start {
                return Err(malformed("proposal.candidate range"));
            }
            Ok(EditCandidate::new(
                source_from_pb(required(value.source.take(), "proposal.candidate.source")?)?,
                size(value.start, "proposal.candidate.start")?
                    ..size(value.end, "proposal.candidate.end")?,
                value.replacement,
            ))
        })
        .collect::<Result<_>>()?;
    let selected = value
        .selected
        .into_iter()
        .map(|value| size(value, "proposal.selected"))
        .collect::<Result<_>>()?;
    let operations = value
        .file_operations
        .into_iter()
        .map(|mut value| {
            Ok(
                match value
                    .kind
                    .as_known()
                    .ok_or_else(|| malformed("proposal.file_operation.kind"))?
                {
                    pb::native_file_operation::Kind::Create => {
                        if value.source.is_set() || !value.destination.is_empty() {
                            return Err(malformed("proposal.create fields"));
                        }
                        FileOperation::Create {
                            path: OsString::from_vec(value.path),
                            text: value.text,
                        }
                    }
                    pb::native_file_operation::Kind::Delete => {
                        if !value.path.is_empty()
                            || !value.text.is_empty()
                            || !value.destination.is_empty()
                        {
                            return Err(malformed("proposal.delete fields"));
                        }
                        FileOperation::Delete {
                            source: source_from_pb(required(
                                value.source.take(),
                                "proposal.delete.source",
                            )?)?,
                        }
                    }
                    pb::native_file_operation::Kind::Rename => {
                        if !value.path.is_empty() || !value.text.is_empty() {
                            return Err(malformed("proposal.rename fields"));
                        }
                        FileOperation::Rename {
                            source: source_from_pb(required(
                                value.source.take(),
                                "proposal.rename.source",
                            )?)?,
                            destination: OsString::from_vec(value.destination),
                        }
                    }
                },
            )
        })
        .collect::<Result<_>>()?;
    let placeholders = value
        .unresolved_placeholders
        .into_iter()
        .map(|value| {
            Ok(UnresolvedPlaceholder {
                candidate: size(value.candidate, "proposal.placeholder.candidate")?,
                description: value.description,
            })
        })
        .collect::<Result<_>>()?;
    let conflicts = value
        .conflicts
        .into_iter()
        .map(|value| {
            Ok(CandidateConflict {
                first: size(value.first, "proposal.conflict.first")?,
                second: size(value.second, "proposal.conflict.second")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let proposal = ChangeProposal::with_file_operations(
        producer,
        originals,
        candidates,
        selected,
        operations,
        value.limitations,
        placeholders,
    )?;
    if proposal.conflicts() != conflicts {
        return Err(malformed("proposal.conflicts"));
    }
    Ok(proposal)
}

fn native_location_to_pb(value: NativeLocation) -> pb::NativeByteLocation {
    pb::NativeByteLocation {
        path: path(&value.path),
        start: number(value.start),
        end: number(value.end),
        ..Default::default()
    }
}
fn native_location_from_pb(value: pb::NativeByteLocation) -> Result<NativeLocation> {
    if value.end < value.start || value.path.is_empty() {
        return Err(malformed("native_types.location"));
    }
    Ok(NativeLocation {
        path: from_path(value.path),
        start: size(value.start, "native_types.start")?,
        end: size(value.end, "native_types.end")?,
    })
}
fn type_to_pb(value: TypeFact) -> pb::NativeTypeFact {
    pb::NativeTypeFact {
        display: value.display,
        is_reference: value.is_reference,
        is_mutable_reference: value.is_mutable_reference,
        contains_unknown: value.contains_unknown,
        ..Default::default()
    }
}
fn type_from_pb(value: pb::NativeTypeFact) -> TypeFact {
    TypeFact {
        display: value.display,
        is_reference: value.is_reference,
        is_mutable_reference: value.is_mutable_reference,
        contains_unknown: value.contains_unknown,
    }
}
fn layout_to_pb(value: LayoutFact) -> pb::NativeLayoutFact {
    pb::NativeLayoutFact {
        ty: value.ty,
        size: value.size,
        align: value.align,
        sized: outcome_to_pb(value.sized).into(),
        prefix_size: value.prefix_size,
        prefix_align: value.prefix_align,
        error: value.error,
        niches: value.niches,
        fields: value
            .fields
            .into_iter()
            .map(|field| pb::NativeLayoutFieldFact {
                name: field.name,
                offset: field.offset,
                layout: layout_to_pb(*field.layout).into(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}
fn layout_from_pb(value: pb::NativeLayoutFact) -> Result<LayoutFact> {
    let sized = outcome_from_pb(value.sized)?;
    if sized != PredicateOutcome::Holds
        && (value.size.is_some() || value.align.is_some() || value.niches.is_some())
    {
        return Err(malformed("layout.full_extent_without_sized_proof"));
    }
    Ok(LayoutFact {
        ty: value.ty,
        size: value.size,
        align: value.align,
        sized,
        prefix_size: value.prefix_size,
        prefix_align: value.prefix_align,
        error: value.error,
        niches: value.niches,
        fields: value
            .fields
            .into_iter()
            .map(|mut field| {
                Ok(LayoutFieldFact {
                    name: field.name,
                    offset: field.offset,
                    layout: Box::new(layout_from_pb(required(
                        field.layout.take(),
                        "layout.field",
                    )?)?),
                })
            })
            .collect::<Result<_>>()?,
    })
}
fn target_to_pb(value: TargetFact) -> pb::NativeTargetFact {
    pb::NativeTargetFact {
        relationship: value.relationship,
        name: value.name,
        location: value.location.map(native_location_to_pb).into(),
        precision: value.precision,
        ..Default::default()
    }
}
fn target_from_pb(mut value: pb::NativeTargetFact) -> Result<TargetFact> {
    Ok(TargetFact {
        relationship: value.relationship,
        name: value.name,
        location: value
            .location
            .take()
            .map(native_location_from_pb)
            .transpose()?,
        precision: value.precision,
    })
}
fn outcome_to_pb(value: PredicateOutcome) -> pb::NativePredicateOutcome {
    match value {
        PredicateOutcome::Holds => pb::NativePredicateOutcome::Holds,
        PredicateOutcome::NotProven => pb::NativePredicateOutcome::NotProven,
        PredicateOutcome::Invalid => pb::NativePredicateOutcome::InvalidPredicate,
        PredicateOutcome::Unsupported => pb::NativePredicateOutcome::UnsupportedPredicate,
    }
}
fn outcome_from_pb(
    value: buffa::EnumValue<pb::NativePredicateOutcome>,
) -> Result<PredicateOutcome> {
    Ok(
        match value
            .as_known()
            .ok_or_else(|| malformed("predicate.outcome"))?
        {
            pb::NativePredicateOutcome::Holds => PredicateOutcome::Holds,
            pb::NativePredicateOutcome::NotProven => PredicateOutcome::NotProven,
            pb::NativePredicateOutcome::InvalidPredicate => PredicateOutcome::Invalid,
            pb::NativePredicateOutcome::UnsupportedPredicate => PredicateOutcome::Unsupported,
        },
    )
}
fn callable_context_to_pb(c: CallableContextFact) -> pb::NativeCallableContextFact {
    pb::NativeCallableContextFact {
        callsite: c.callsite.map(native_location_to_pb).into(),
        callable_target: target_to_pb(c.callable_target).into(),
        substitution_owner: target_to_pb(c.substitution_owner).into(),
        receiver_parameter: c.receiver_parameter.map(type_to_pb).into(),
        parameters: c.parameters.into_iter().map(type_to_pb).collect(),
        result: type_to_pb(c.result).into(),
        adjusted_result: c.adjusted_result.map(type_to_pb).into(),
        result_coerces_to_adjusted: c.result_coerces_to_adjusted,
        substitutions: c
            .substitutions
            .map(|arguments| pb::NativeCallableSubstitutions {
                arguments: arguments
                    .into_iter()
                    .map(|s| pb::NativeCallableSubstitutionFact {
                        parameter_index: number(s.parameter_index),
                        parameter: s.parameter,
                        argument: type_to_pb(s.argument).into(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            })
            .into(),
        receiver_adjustments: c
            .receiver_adjustments
            .map(|steps| pb::NativeReceiverAdjustments {
                steps: steps
                    .into_iter()
                    .map(|s| pb::NativeAdjustmentFact {
                        kind: s.kind,
                        source: type_to_pb(s.source).into(),
                        target: type_to_pb(s.target).into(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            })
            .into(),
        ..Default::default()
    }
}
fn callable_context_from_pb(mut c: pb::NativeCallableContextFact) -> Result<CallableContextFact> {
    Ok(CallableContextFact {
        callsite: c.callsite.take().map(native_location_from_pb).transpose()?,
        callable_target: target_from_pb(required(
            c.callable_target.take(),
            "callable_context.callable_target",
        )?)?,
        substitution_owner: target_from_pb(required(
            c.substitution_owner.take(),
            "callable_context.substitution_owner",
        )?)?,
        receiver_parameter: c.receiver_parameter.take().map(type_from_pb),
        parameters: c.parameters.into_iter().map(type_from_pb).collect(),
        result: type_from_pb(required(c.result.take(), "callable_context.result")?),
        adjusted_result: c.adjusted_result.take().map(type_from_pb),
        result_coerces_to_adjusted: c.result_coerces_to_adjusted,
        substitutions: c
            .substitutions
            .take()
            .map(|s| {
                s.arguments
                    .into_iter()
                    .map(|mut s| {
                        Ok(CallableSubstitutionFact {
                            parameter_index: size(
                                s.parameter_index,
                                "substitution.parameter_index",
                            )?,
                            parameter: s.parameter,
                            argument: type_from_pb(required(
                                s.argument.take(),
                                "substitution.argument",
                            )?),
                        })
                    })
                    .collect::<Result<_>>()
            })
            .transpose()?,
        receiver_adjustments: c
            .receiver_adjustments
            .take()
            .map(|s| {
                s.steps
                    .into_iter()
                    .map(|mut s| {
                        Ok(AdjustmentFact {
                            kind: s.kind,
                            source: type_from_pb(required(s.source.take(), "adjustment.source")?),
                            target: type_from_pb(required(s.target.take(), "adjustment.target")?),
                        })
                    })
                    .collect::<Result<_>>()
            })
            .transpose()?,
    })
}
fn typed_to_pb(value: TypedFacts) -> pb::NativeTypedFacts {
    pb::NativeTypedFacts {
        callable_candidates: value
            .callable_candidates
            .into_iter()
            .map(|c| pb::NativeCallableCandidateFact {
                target: target_to_pb(c.target).into(),
                signature: c.signature,
                receiver: type_to_pb(c.receiver).into(),
                applicability: outcome_to_pb(c.applicability).into(),
                required_import: c.required_import,
                context: c.context.map(callable_context_to_pb).into(),
                obligations: c
                    .obligations
                    .into_iter()
                    .map(|o| pb::NativeCallableObligationFact {
                        declared_trait: target_to_pb(o.declared_trait).into(),
                        parameter: o.parameter,
                        subject: o.subject.map(type_to_pb).into(),
                        outcome: outcome_to_pb(o.outcome).into(),
                        reason: o.reason,
                        ..Default::default()
                    })
                    .collect(),
                limitations: c.limitations,
                reasons: c.reasons,
                ..Default::default()
            })
            .collect(),
        context: value
            .context
            .map(|c| pb::NativeTypeContext {
                crate_root: path(&c.crate_root),
                crate_source_sha256: c.crate_source_sha256,
                features: c.features.into_iter().collect(),
                cfg: c.cfg,
                edition: c.edition,
                target: c.target,
                compiler_identity: c.compiler_identity,
                ..Default::default()
            })
            .into(),
        capture_owner: value.capture_owner.map(native_location_to_pb).into(),
        callsite: value.callsite.map(native_location_to_pb).into(),
        callable_kind: value.callable_kind,
        layout: value.layout.map(layout_to_pb).into(),
        adjustments: value
            .adjustments
            .into_iter()
            .map(|a| pb::NativeAdjustmentFact {
                kind: a.kind,
                source: type_to_pb(a.source).into(),
                target: type_to_pb(a.target).into(),
                ..Default::default()
            })
            .collect(),
        captures: value
            .captures
            .into_iter()
            .map(|c| pb::NativeCaptureFact {
                local: c.local.map(native_location_to_pb).into(),
                usages: c
                    .usages
                    .map(|uses| pb::NativeCaptureUsages {
                        uses: uses
                            .into_iter()
                            .map(|u| pb::NativeCaptureUseFact {
                                location: u.location.map(native_location_to_pb).into(),
                                is_ref: u.is_ref,
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    })
                    .into(),
                place: c.place,
                has_field_projections: c.has_field_projections,
                mode: c.mode,
                place_type: type_to_pb(c.place_type).into(),
                stored_type: type_to_pb(c.stored_type).into(),
                ..Default::default()
            })
            .collect(),
        expression: value.expression.map(native_location_to_pb).into(),
        original: value.original.map(type_to_pb).into(),
        adjusted: value.adjusted.map(type_to_pb).into(),
        original_equals_adjusted: value.original_equals_adjusted,
        original_coerces_to_adjusted: value.original_coerces_to_adjusted,
        target: value
            .target
            .map(|value| pb::NativeTargetFact {
                relationship: value.relationship,
                name: value.name,
                location: value.location.map(native_location_to_pb).into(),
                precision: value.precision,
                ..Default::default()
            })
            .into(),
        bounds: value
            .bounds
            .into_iter()
            .map(|value| pb::NativeBoundFact {
                parameter: value.parameter,
                declared_traits: value.declared_traits,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}
fn typed_from_pb(mut value: pb::NativeTypedFacts) -> Result<TypedFacts> {
    Ok(TypedFacts {
        callable_candidates: value
            .callable_candidates
            .into_iter()
            .map(|mut c| {
                Ok(CallableCandidateFact {
                    target: target_from_pb(required(c.target.take(), "candidate.target")?)?,
                    signature: c.signature,
                    receiver: type_from_pb(required(c.receiver.take(), "candidate.receiver")?),
                    applicability: outcome_from_pb(c.applicability)?,
                    required_import: c.required_import,
                    context: c.context.take().map(callable_context_from_pb).transpose()?,
                    obligations: c
                        .obligations
                        .into_iter()
                        .map(|mut o| {
                            Ok(CallableObligationFact {
                                declared_trait: target_from_pb(required(
                                    o.declared_trait.take(),
                                    "obligation.declared_trait",
                                )?)?,
                                parameter: o.parameter,
                                subject: o.subject.take().map(type_from_pb),
                                outcome: outcome_from_pb(o.outcome)?,
                                reason: o.reason,
                            })
                        })
                        .collect::<Result<_>>()?,
                    limitations: c.limitations,
                    reasons: c.reasons,
                })
            })
            .collect::<Result<_>>()?,
        context: value.context.take().map(|c| TypeContext {
            crate_root: from_path(c.crate_root),
            crate_source_sha256: c.crate_source_sha256,
            features: c.features.into_iter().collect(),
            cfg: c.cfg,
            edition: c.edition,
            target: c.target,
            compiler_identity: c.compiler_identity,
        }),
        capture_owner: value
            .capture_owner
            .take()
            .map(native_location_from_pb)
            .transpose()?,
        callsite: value
            .callsite
            .take()
            .map(native_location_from_pb)
            .transpose()?,
        callable_kind: value.callable_kind,
        layout: value.layout.take().map(layout_from_pb).transpose()?,
        adjustments: value
            .adjustments
            .into_iter()
            .map(|mut a| {
                Ok(AdjustmentFact {
                    kind: a.kind,
                    source: type_from_pb(required(a.source.take(), "adjustment.source")?),
                    target: type_from_pb(required(a.target.take(), "adjustment.target")?),
                })
            })
            .collect::<Result<_>>()?,
        captures: value
            .captures
            .into_iter()
            .map(|mut c| {
                Ok(CaptureFact {
                    local: c.local.take().map(native_location_from_pb).transpose()?,
                    usages: c
                        .usages
                        .take()
                        .map(|uses| {
                            uses.uses
                                .into_iter()
                                .map(|mut u| {
                                    Ok(CaptureUseFact {
                                        location: u
                                            .location
                                            .take()
                                            .map(native_location_from_pb)
                                            .transpose()?,
                                        is_ref: u.is_ref,
                                    })
                                })
                                .collect::<Result<_>>()
                        })
                        .transpose()?,
                    place: c.place,
                    has_field_projections: c.has_field_projections,
                    mode: c.mode,
                    place_type: type_from_pb(required(c.place_type.take(), "capture.place_type")?),
                    stored_type: type_from_pb(required(
                        c.stored_type.take(),
                        "capture.stored_type",
                    )?),
                })
            })
            .collect::<Result<_>>()?,
        expression: value
            .expression
            .take()
            .map(native_location_from_pb)
            .transpose()?,
        original: value.original.take().map(type_from_pb),
        adjusted: value.adjusted.take().map(type_from_pb),
        original_equals_adjusted: value.original_equals_adjusted,
        original_coerces_to_adjusted: value.original_coerces_to_adjusted,
        target: value
            .target
            .take()
            .map(|mut value| -> Result<TargetFact> {
                Ok(TargetFact {
                    relationship: value.relationship,
                    name: value.name,
                    location: value
                        .location
                        .take()
                        .map(native_location_from_pb)
                        .transpose()?,
                    precision: value.precision,
                })
            })
            .transpose()?,
        bounds: value
            .bounds
            .into_iter()
            .map(|value| BoundFact {
                parameter: value.parameter,
                declared_traits: value.declared_traits,
            })
            .collect(),
    })
}
pub(super) fn answer_to_pb(value: Answer) -> Result<pb::NativeAnswer> {
    Ok(pb::NativeAnswer {
        session: analysis_to_pb(value.session)?.into(),
        source: source_to_pb(&value.source).into(),
        source_uri: value.source_uri,
        source_byte_offset: number(value.source_byte_offset),
        available: matches!(value.availability, Availability::Available),
        generation: value.generation,
        position: position_to_pb(value.position).into(),
        sections: value
            .sections
            .into_iter()
            .map(section_to_pb)
            .collect::<Result<_>>()?,
        location_sources: value
            .location_sources
            .into_iter()
            .map(|value| pb::NativeSourceHash {
                path: path(&value.path),
                source_sha256: value.source_sha256,
                ..Default::default()
            })
            .collect(),
        proposal: value.proposal.map(proposal_to_pb).transpose()?.into(),
        limitations: value.limitations,
        observed: observed_to_pb(value.observed)?.into(),
        omitted_relationships: value
            .omitted_relationships
            .into_iter()
            .map(|(relation, count)| pb::NativeOmittedRelationship {
                relation: relation_to_pb(relation).into(),
                count: number(count),
                ..Default::default()
            })
            .collect(),
        native_types: value.native_types.map(typed_to_pb).into(),
        ..Default::default()
    })
}
pub(super) fn answer_from_pb(mut value: pb::NativeAnswer) -> Result<Answer> {
    let mut omitted_relationships = BTreeMap::new();
    for omitted in value.omitted_relationships {
        if omitted_relationships
            .insert(
                relation_from_pb(omitted.relation)?,
                size(omitted.count, "answer.omitted_relationships.count")?,
            )
            .is_some()
        {
            return Err(malformed("answer duplicate omitted_relationships"));
        }
    }
    Ok(Answer {
        session: analysis_from_pb(required(value.session.take(), "answer.session")?)?,
        source: source_from_pb(required(value.source.take(), "answer.source")?)?,
        source_uri: value.source_uri,
        source_byte_offset: size(value.source_byte_offset, "answer.source_byte_offset")?,
        availability: if value.available {
            Availability::Available
        } else {
            Availability::NoMatch
        },
        generation: value.generation,
        position: position_from_pb(required(value.position.take(), "answer.position")?),
        observed: observed_from_pb(required(value.observed.take(), "answer.observed")?)?,
        omitted_relationships,
        native_types: value.native_types.take().map(typed_from_pb).transpose()?,
        sections: value
            .sections
            .into_iter()
            .map(section_from_pb)
            .collect::<Result<_>>()?,
        location_sources: value
            .location_sources
            .into_iter()
            .map(|value| LocationSource {
                path: from_path(value.path),
                source_sha256: value.source_sha256,
            })
            .collect(),
        proposal: value.proposal.take().map(proposal_from_pb).transpose()?,
        limitations: value.limitations,
    })
}
pub(super) fn workspace_to_pb(value: WorkspaceAnswer) -> Result<pb::NativeWorkspaceAnswer> {
    Ok(pb::NativeWorkspaceAnswer {
        session: analysis_to_pb(value.session)?.into(),
        generation: value.generation,
        symbols: section_to_pb(value.symbols)?.into(),
        limitations: value.limitations,
        observed: value.observed.into_iter().map(symbol_to_pb).collect(),
        ..Default::default()
    })
}
pub(super) fn workspace_from_pb(mut value: pb::NativeWorkspaceAnswer) -> Result<WorkspaceAnswer> {
    Ok(WorkspaceAnswer {
        session: analysis_from_pb(required(value.session.take(), "workspace.session")?)?,
        generation: value.generation,
        symbols: section_from_pb(required(value.symbols.take(), "workspace.symbols")?)?,
        limitations: value.limitations,
        observed: value
            .observed
            .into_iter()
            .map(symbol_from_pb)
            .collect::<Result<_>>()?,
    })
}
pub(super) fn failure_to_pb(value: Failure) -> Result<pb::Failure> {
    Ok(pb::Failure {
        state: match value.state {
            FailureState::Unavailable => pb::failure::State::Unavailable,
            FailureState::Unsupported => pb::failure::State::Unsupported,
            FailureState::Cancelled => pb::failure::State::Cancelled,
            FailureState::Stale => pb::failure::State::Stale,
            FailureState::Failed => pb::failure::State::Failed,
        }
        .into(),
        phase: match value.phase {
            FailurePhase::Schema => pb::failure::Phase::Schema,
            FailurePhase::Constraint => pb::failure::Phase::Constraint,
            FailurePhase::Context => pb::failure::Phase::Context,
            FailurePhase::Validator => pb::failure::Phase::Validator,
            FailurePhase::Provider => pb::failure::Phase::Provider,
            FailurePhase::Transport => pb::failure::Phase::Transport,
            FailurePhase::Settlement => pb::failure::Phase::Settlement,
        }
        .into(),
        code: "semantic.native_refusal".into(),
        message: value.message,
        backend_code: value.backend_code,
        original: value.backend_data.as_ref().map(opaque).transpose()?.into(),
        observed_cancellation: value
            .observed_cancellation
            .map(|value| cancellation_to_pb(*value))
            .transpose()?
            .into(),
        ..Default::default()
    })
}
pub(super) fn failure_from_pb(mut value: pb::Failure) -> Result<Failure> {
    Ok(Failure {
        state: match value
            .state
            .as_known()
            .ok_or_else(|| malformed("failure.state"))?
        {
            pb::failure::State::Unavailable => FailureState::Unavailable,
            pb::failure::State::Unsupported => FailureState::Unsupported,
            pb::failure::State::Cancelled => FailureState::Cancelled,
            pb::failure::State::Stale => FailureState::Stale,
            pb::failure::State::Failed => FailureState::Failed,
        },
        phase: match value
            .phase
            .as_known()
            .ok_or_else(|| malformed("failure.phase"))?
        {
            pb::failure::Phase::Schema => FailurePhase::Schema,
            pb::failure::Phase::Constraint => FailurePhase::Constraint,
            pb::failure::Phase::Context => FailurePhase::Context,
            pb::failure::Phase::Validator => FailurePhase::Validator,
            pb::failure::Phase::Provider => FailurePhase::Provider,
            pb::failure::Phase::Transport => FailurePhase::Transport,
            pb::failure::Phase::Settlement => FailurePhase::Settlement,
        },
        message: value.message,
        backend_code: value.backend_code,
        backend_data: value.original.take().map(from_opaque).transpose()?,
        observed_cancellation: value
            .observed_cancellation
            .take()
            .map(cancellation_from_pb)
            .transpose()?
            .map(Box::new),
    })
}
fn cancellation_to_pb(value: CancellationObservation) -> Result<pb::NativeCancellationObservation> {
    Ok(pb::NativeCancellationObservation {
        completion: match value.completion {
            Settlement::CpuSettled => pb::terminal::Completion::CpuSettled,
            Settlement::LspDetached => pb::terminal::Completion::LspDetached,
            Settlement::LspReturned => pb::terminal::Completion::LspReturned,
        }
        .into(),
        original_outcome: Some(match value.original {
            OriginalOutcome::Succeeded => {
                pb::native_cancellation_observation::OriginalOutcome::Succeeded(Default::default())
            }
            OriginalOutcome::Failed(value) => {
                pb::native_cancellation_observation::OriginalOutcome::Failed(Box::new(
                    failure_to_pb(value)?,
                ))
            }
        }),
        ..Default::default()
    })
}
fn cancellation_from_pb(
    value: pb::NativeCancellationObservation,
) -> Result<CancellationObservation> {
    Ok(CancellationObservation {
        completion: match value
            .completion
            .as_known()
            .ok_or_else(|| malformed("cancellation.completion"))?
        {
            pb::terminal::Completion::CpuSettled => Settlement::CpuSettled,
            pb::terminal::Completion::LspDetached => Settlement::LspDetached,
            pb::terminal::Completion::LspReturned => Settlement::LspReturned,
            pb::terminal::Completion::Unobserved => {
                return Err(malformed("cancellation observed completion"));
            }
        },
        original: match required(value.original_outcome, "cancellation.original_outcome")? {
            pb::native_cancellation_observation::OriginalOutcome::Succeeded(_) => {
                OriginalOutcome::Succeeded
            }
            pb::native_cancellation_observation::OriginalOutcome::Failed(value) => {
                OriginalOutcome::Failed(failure_from_pb(*value)?)
            }
        },
    })
}
pub(super) fn response_event(value: Response) -> Result<pb::NativeEvent> {
    Ok(pb::NativeEvent {
        item: Some(match value {
            Response::Graph(value) => {
                pb::native_event::Item::Graph(Box::new(graph::answer_to_pb(value)?))
            }
            Response::Query(value) => {
                pb::native_event::Item::Query(Box::new(answer_to_pb(*value)?))
            }
            Response::WorkspaceSymbols(value) => {
                pb::native_event::Item::Workspace(Box::new(workspace_to_pb(value)?))
            }
            Response::Status(_) | Response::PackageContext(_) | Response::Stopped => {
                return Err(malformed("Run.response"));
            }
        }),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use serde_json::json;

    fn context() -> AnalysisContext {
        AnalysisContext {
            backend: "original-native-provider".into(),
            configuration: Configuration {
                backend: Backend::NativeSnapshot,
                project: "/selected/project".into(),
                features: vec!["actual-feature".into()],
                no_default_features: true,
                target: Some("selected-target".into()),
                build_macros: false,
                test_context: false,
            },
            position_encoding: "utf-8".into(),
            backend_status: Some(json!({"native":"original health"})),
            preparation: "original immutable world".into(),
        }
    }
    fn raw_path() -> PathBuf {
        from_path(b"/selected/\xff.rs".to_vec())
    }
    fn location() -> FactLocation {
        FactLocation::Bytes {
            path: raw_path(),
            start: 3,
            end: 9,
        }
    }

    #[test]
    fn all_native_operations_preserve_explicit_guards_and_raw_paths_over_binary_wire() {
        let operations = [
            Kind::Explain,
            Kind::Refs,
            Kind::Calls,
            Kind::Implementations,
            Kind::Macro,
            Kind::Tests,
            Kind::Candidates,
            Kind::Symbols,
            Kind::Hints,
            Kind::Hint { index: 29 },
            Kind::Ssr {
                rule: "$a + $b ==>> $b + $a".into(),
            },
            Kind::Importable {
                target: ImportTarget {
                    file: raw_path(),
                    byte_start: 3,
                    byte_end: 9,
                    source_sha256: "a".repeat(64),
                },
            },
            Kind::Predicate {
                text: "Clone".into(),
            },
            Kind::Candidate { index: 27 },
            Kind::Rename {
                name: "actual_name".into(),
            },
            Kind::FileRename {
                destination: raw_path(),
            },
            Kind::Assists,
            Kind::Assist { index: 31 },
        ];
        for kind in operations {
            let request = Request::Query(Query {
                kind,
                file: raw_path(),
                line: 17,
                column: 23,
                if_source: Some("b".repeat(64)),
                if_generation: Some(7),
                if_candidate: Some("original-fingerprint".into()),
                limit: 43,
                end_line: Some(19),
            });
            let original = request_to_pb(&request).unwrap();
            let binary = original.encode_to_vec();
            let received = pb::NativeRequest::decode_from_slice(&binary).unwrap();
            let domain = request_from_pb(received).unwrap();
            assert_eq!(request_to_pb(&domain).unwrap(), original);
            let Request::Query(restored) = domain else {
                panic!("changed operation")
            };
            assert_eq!(restored.file, raw_path());
            assert_eq!(restored.if_generation, Some(7));
            assert_eq!(
                restored.if_candidate.as_deref(),
                Some("original-fingerprint")
            );
        }
        for matching in [
            NativeNameMode::Fuzzy,
            NativeNameMode::Exact,
            NativeNameMode::Prefix,
        ] {
            let request = Request::WorkspaceSymbols(WorkspaceQuery {
                query: "Target".into(),
                matching,
                limit: 30,
                include_dependencies: true,
                if_generation: Some(11),
            });
            let original = request_to_pb(&request).unwrap();
            let received = request_from_pb(
                pb::NativeRequest::decode_from_slice(&original.encode_to_vec()).unwrap(),
            )
            .unwrap();
            assert_eq!(request_to_pb(&received).unwrap(), original);
        }
        assert!(request_to_pb(&Request::Status {}).is_err());
        assert!(request_from_pb(pb::NativeRequest::default()).is_err());
    }

    #[test]
    fn typed_native_observations_remain_distinct_from_opaque_original_documents() {
        let source = SourceText::new(raw_path(), "fn target() {}\n".into()).unwrap();
        let answer = Answer {
            session: context(),
            source: source.identity().clone(),
            source_uri: "file:///selected/%FF.rs".into(),
            source_byte_offset: 3,
            availability: Availability::Available,
            generation: 7,
            position: ProtocolPosition {
                line: 0,
                character: 3,
            },
            observed: ObservedFacts {
                investigation: None,
                symbols: vec![SymbolFact {
                    name: "target".into(),
                    kind: Some("function".into()),
                    container: Some("display_only".into()),
                    location: Some(location()),
                    source_sha256: Some(source.identity().source_sha256().into()),
                }],
                relationships: vec![RelationshipFact {
                    relation: FactRelation::Calls,
                    direction: FactDirection::Incoming,
                    location: location(),
                    name: Some("caller".into()),
                    read: None,
                    write: Some(false),
                    import: Some(false),
                    test: Some(true),
                }],
                predicate: Some(PredicateFact {
                    outcome: PredicateOutcome::NotProven,
                    cause: Some("native proof not established".into()),
                }),
                importability: Some(ImportabilityFact {
                    visibility: Some(true),
                    importable: None,
                    path: None,
                    limitations: vec!["native proof unavailable".into()],
                }),
                candidates: vec![LazyCandidateFact {
                    kind: CandidateKind::Assist,
                    index: 31,
                    fingerprint: "original-candidate".into(),
                    label: Some("actual action".into()),
                    location: Some(location()),
                }],
                tests: vec![TestCandidateFact {
                    cfg_eligible: None,
                    cargo_targets: Vec::new(),
                    snapshot_update: None,
                    test_id: Some("module::test".into()),
                    cfg: None,
                    relatedness: None,
                    eligible: None,
                    label: Some("actual test".into()),
                    location: Some(location()),
                    runnable_kind: Some("cargo".into()),
                    cargo_args: vec!["test".into()],
                    executable_args: vec!["actual_test".into()],
                    original: json!({"opaque":"not executable authority"}),
                }],
            },
            omitted_relationships: BTreeMap::from([(FactRelation::Calls, 27)]),
            native_types: None,
            sections: vec![Section {
                category: "predicate_evaluation".into(),
                evidence: json!({"holds":true,"display":"contradicts typed unknown deliberately"}),
                omitted: 2,
                handles: vec![CandidateHandle {
                    index: 31,
                    fingerprint: "original-candidate".into(),
                }],
            }],
            location_sources: vec![LocationSource {
                path: raw_path(),
                source_sha256: source.identity().source_sha256().into(),
            }],
            proposal: None,
            limitations: vec!["selected native world only".into()],
        };
        let original = answer_to_pb(answer).unwrap();
        let domain =
            answer_from_pb(pb::NativeAnswer::decode_from_slice(&original.encode_to_vec()).unwrap())
                .unwrap();
        assert_eq!(
            domain.observed.predicate.as_ref().unwrap().outcome,
            PredicateOutcome::NotProven
        );
        assert_eq!(
            domain.observed.importability.as_ref().unwrap().importable,
            None
        );
        assert_eq!(domain.source.path(), raw_path());
        assert_eq!(domain.omitted_relationships[&FactRelation::Calls], 27);
        assert_eq!(domain.sections[0].evidence["holds"], true);
        assert_eq!(answer_to_pb(domain).unwrap(), original);
        let mut missing = original.clone();
        missing.observed = Default::default();
        assert!(answer_from_pb(missing).is_err());
        let mut unknown = original;
        unknown
            .observed
            .modify(|facts| facts.predicate.modify(|fact| fact.outcome = 99.into()));
        assert!(answer_from_pb(unknown).is_err());
    }

    #[test]
    fn checked_preview_transports_originals_and_rejects_changed_bytes_or_conflict_claims() {
        use crate::source_proposal::{
            ChangeProposal, EditCandidate, FileOperation, ProducerContext, UnresolvedPlaceholder,
        };
        let source = SourceText::new(raw_path(), "fn original() {}\n".into()).unwrap();
        let proposal = ChangeProposal::with_file_operations(
            ProducerContext::Semantic {
                backend: "actual-provider".into(),
                configuration: "actual-world".into(),
                synchronized_generation: "7".into(),
            },
            vec![source.clone()],
            vec![
                EditCandidate::new(source.identity().clone(), 3..11, "replacement".into()),
                EditCandidate::new(source.identity().clone(), 3..11, "alternative".into()),
            ],
            vec![0],
            vec![FileOperation::Rename {
                source: source.identity().clone(),
                destination: OsString::from_vec(b"/selected/renamed-\xfe.rs".to_vec()),
            }],
            vec!["preview grants no writes".into()],
            vec![UnresolvedPlaceholder {
                candidate: 1,
                description: "choose alternate explicitly".into(),
            }],
        )
        .unwrap();
        let original = proposal_to_pb(proposal).unwrap();
        let restored = proposal_from_pb(
            pb::NativePreview::decode_from_slice(&original.encode_to_vec()).unwrap(),
        )
        .unwrap();
        assert_eq!(restored.sources()[0].text(), source.text());
        assert_eq!(restored.sources()[0].identity(), source.identity());
        assert_eq!(restored.selected(), &[0]);
        assert_eq!(restored.conflicts().len(), 1);
        assert_eq!(proposal_to_pb(restored).unwrap(), original);
        let mut stale = original.clone();
        stale.originals[0].text.push('!');
        assert!(proposal_from_pb(stale).is_err());
        let mut false_conflicts = original;
        false_conflicts.conflicts.clear();
        assert!(proposal_from_pb(false_conflicts).is_err());
    }

    #[test]
    fn source_origins_and_backend_refusals_keep_their_original_boundaries() {
        let git_dir = crate::repository::GitName::new(b"/original/\xff.git");
        for origin in [
            SourceOrigin::Working,
            SourceOrigin::Index {
                git_dir: git_dir.clone(),
                index_path: crate::repository::GitName::new(b"/original/index"),
                blob: "original-blob".into(),
            },
            SourceOrigin::Blob {
                git_dir,
                revision: "resolved-revision".into(),
                blob: "original-blob".into(),
            },
        ] {
            let source_path = if matches!(origin, SourceOrigin::Working) {
                raw_path()
            } else {
                from_path(b"source-\xff.rs".to_vec())
            };
            let identity = SourceIdentity::from_parts(source_path, "a".repeat(64), origin).unwrap();
            assert_eq!(source_from_pb(source_to_pb(&identity)).unwrap(), identity);
        }
        for state in [
            FailureState::Unavailable,
            FailureState::Unsupported,
            FailureState::Cancelled,
            FailureState::Stale,
            FailureState::Failed,
        ] {
            let failure = Failure {
                state,
                phase: FailurePhase::Provider,
                message: "original native refusal".into(),
                backend_code: Some(-32800),
                backend_data: Some(
                    json!({"cpu_work_completed":true,"original_cause":{"native":"exact"}}),
                ),
                observed_cancellation: None,
            };
            let expected = serde_json::to_value(&failure).unwrap();
            let wire = failure_to_pb(failure).unwrap();
            let received =
                failure_from_pb(pb::Failure::decode_from_slice(&wire.encode_to_vec()).unwrap())
                    .unwrap();
            assert_eq!(serde_json::to_value(received).unwrap(), expected);
        }
        let unknown = pb::Failure {
            phase: 99.into(),
            ..Default::default()
        };
        assert!(failure_from_pb(unknown).is_err());
        assert!(source_from_pb(pb::NativeSourceIdentity::default()).is_err());
    }

    #[test]
    fn failure_phases_and_observed_settlement_do_not_depend_on_backend_json() {
        for phase in [
            FailurePhase::Schema,
            FailurePhase::Constraint,
            FailurePhase::Context,
            FailurePhase::Validator,
            FailurePhase::Provider,
            FailurePhase::Transport,
            FailurePhase::Settlement,
        ] {
            for completion in [
                Settlement::CpuSettled,
                Settlement::LspDetached,
                Settlement::LspReturned,
            ] {
                for original in [
                    OriginalOutcome::Succeeded,
                    OriginalOutcome::Failed(Failure {
                        state: FailureState::Stale,
                        phase: FailurePhase::Context,
                        message: "original changed source".into(),
                        backend_code: Some(-32801),
                        backend_data: Some(json!({"original":"native-data"})),
                        observed_cancellation: None,
                    }),
                ] {
                    let failure = Failure {
                        state: FailureState::Cancelled,
                        phase,
                        message: "cancellation observed".into(),
                        backend_code: None,
                        backend_data: Some(
                            json!({"cpu_work_completed":false,"completion":"deliberately misleading provider data"}),
                        ),
                        observed_cancellation: Some(Box::new(CancellationObservation {
                            completion,
                            original,
                        })),
                    };
                    let expected = serde_json::to_value(&failure).unwrap();
                    let wire = failure_to_pb(failure).unwrap();
                    let restored = failure_from_pb(
                        pb::Failure::decode_from_slice(&wire.encode_to_vec()).unwrap(),
                    )
                    .unwrap();
                    assert_eq!(serde_json::to_value(restored).unwrap(), expected);
                }
            }
        }
        assert!(cancellation_from_pb(pb::NativeCancellationObservation::default()).is_err());
        let unknown = pb::Failure {
            state: 99.into(),
            ..Default::default()
        };
        assert!(failure_from_pb(unknown).is_err());
    }

    #[test]
    fn status_package_and_workspace_preserve_one_selected_world() {
        let status = SessionStatus {
            configuration: context().configuration,
            backend: "original-native".into(),
            position_encoding: "utf-8".into(),
            generation: 7,
            server_capabilities: json!({"original":"capabilities"}),
            backend_status: Some(json!({"actual":"health"})),
            active_requests: 3,
            context_current: false,
            context_issue: Some("original context changed".into()),
            automatic_reload_disabled: true,
            preparation_issue: Some("original preparation issue".into()),
            preparation: "explicit".into(),
        };
        let expected = serde_json::to_value(&status).unwrap();
        assert_eq!(
            serde_json::to_value(status_from_pb(status_to_pb(status).unwrap()).unwrap()).unwrap(),
            expected
        );
        let package = PackageContextAnswer {
            session: context(),
            generation: 7,
            packages: vec![NativePackageContext {
                edition: Some("2024".into()),
                cfg: vec!["feature=\"actual\"".into()],
                package_id: "path+file:///selected#actual@0.1.0".into(),
                manifest: raw_path(),
                source_root: "/selected".into(),
                workspace_root: "/selected".into(),
                library_root: raw_path(),
                features: BTreeSet::from(["actual".into()]),
                target: Some("actual-target".into()),
                role: "lib".into(),
                source_hashes: BTreeMap::from([(raw_path(), "b".repeat(64))]),
                dependencies: vec![NativeLibraryProfile {
                    package_id: "actual-dependency".into(),
                    manifest: raw_path(),
                    source_root: "/dependency".into(),
                    library_root: raw_path(),
                    features: BTreeSet::new(),
                    target: None,
                    role: "lib".into(),
                }],
                dependency_issues: vec!["unmapped native context remains unproved".into()],
                dependency_exclusions: BTreeSet::from([NativeDependencyExclusion::Sysroot]),
            }],
        };
        let wire = package_answer_to_pb(package).unwrap();
        let restored = package_answer_from_pb(
            pb::NativePackageContextAnswer::decode_from_slice(&wire.encode_to_vec()).unwrap(),
        )
        .unwrap();
        assert_eq!(restored.packages[0].manifest, raw_path());
        assert_eq!(package_answer_to_pb(restored).unwrap(), wire);
        let workspace = WorkspaceAnswer {
            session: context(),
            generation: 7,
            symbols: Section {
                category: "original symbols".into(),
                evidence: json!({"opaque":true}),
                omitted: 23,
                handles: Vec::new(),
            },
            observed: vec![SymbolFact {
                name: "same_spelled".into(),
                kind: None,
                container: None,
                location: Some(location()),
                source_sha256: Some("b".repeat(64)),
            }],
            limitations: vec!["original scope".into()],
        };
        let wire = workspace_to_pb(workspace).unwrap();
        let restored = workspace_from_pb(
            pb::NativeWorkspaceAnswer::decode_from_slice(&wire.encode_to_vec()).unwrap(),
        )
        .unwrap();
        assert_eq!(restored.observed[0].location.as_ref().map(|location| matches!(location, FactLocation::Bytes { path, .. } if path == &raw_path())), Some(true));
        assert_eq!(workspace_to_pb(restored).unwrap(), wire);
    }

    #[test]
    fn protocol_locations_reject_missing_coordinates_and_unrecognized_request_modes() {
        let actual = FactLocation::Protocol {
            uri: "file:///selected/%FF.rs".into(),
            range: ProtocolRange {
                start: ProtocolPosition {
                    line: 2,
                    character: 3,
                },
                end: ProtocolPosition {
                    line: 5,
                    character: 7,
                },
            },
        };
        let wire = location_to_pb(actual);
        assert_eq!(
            location_to_pb(location_from_pb(wire.clone()).unwrap()),
            wire
        );
        let backwards = pb::NativeLocation {
            coordinates: Some(pb::native_location::Coordinates::Protocol(Box::new(
                pb::NativeProtocolLocation {
                    uri: "file:///selected.rs".into(),
                    range: pb::NativeRange {
                        start: pb::NativePosition {
                            line: 8,
                            character: 3,
                            ..Default::default()
                        }
                        .into(),
                        end: pb::NativePosition {
                            line: 2,
                            character: 0,
                            ..Default::default()
                        }
                        .into(),
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                },
            ))),
            ..Default::default()
        };
        assert!(location_from_pb(backwards).is_err());
        assert!(location_from_pb(pb::NativeLocation::default()).is_err());
        let unknown = pb::NativeRequest {
            operation: Some(pb::native_request::Operation::Workspace(Box::new(
                pb::NativeWorkspaceQuery {
                    limit: 1,
                    matching: 99.into(),
                    ..Default::default()
                },
            ))),
            ..Default::default()
        };
        assert!(request_from_pb(unknown).is_err());
    }

    #[test]
    fn original_and_adjusted_native_types_and_call_targets_have_explicit_wire_fields() {
        let facts = TypedFacts {
            capture_owner: None,
            context: None,
            callable_candidates: Vec::new(),
            callsite: None,
            adjustments: Vec::new(),
            captures: Vec::new(),
            callable_kind: None,
            layout: None,
            expression: Some(NativeLocation {
                path: raw_path(),
                start: 3,
                end: 9,
            }),
            original: Some(TypeFact {
                display: "&mut [Actual; 3]".into(),
                is_reference: true,
                is_mutable_reference: true,
                contains_unknown: false,
            }),
            adjusted: Some(TypeFact {
                display: "&[Actual]".into(),
                is_reference: true,
                is_mutable_reference: false,
                contains_unknown: false,
            }),
            original_equals_adjusted: Some(false),
            original_coerces_to_adjusted: Some(true),
            target: Some(TargetFact {
                relationship: "resolved_call".into(),
                name: "actual_target".into(),
                location: Some(NativeLocation {
                    path: raw_path(),
                    start: 17,
                    end: 29,
                }),
                precision: "original_source".into(),
            }),
            bounds: vec![BoundFact {
                parameter: "T".into(),
                declared_traits: vec!["ActualTrait".into()],
            }],
        };
        let wire = typed_to_pb(facts);
        let restored =
            typed_from_pb(pb::NativeTypedFacts::decode_from_slice(&wire.encode_to_vec()).unwrap())
                .unwrap();
        assert_eq!(restored.original_equals_adjusted, Some(false));
        assert_eq!(restored.original_coerces_to_adjusted, Some(true));
        assert!(restored.original.as_ref().unwrap().is_mutable_reference);
        assert!(!restored.adjusted.as_ref().unwrap().is_mutable_reference);
        assert_eq!(
            restored
                .target
                .as_ref()
                .unwrap()
                .location
                .as_ref()
                .unwrap()
                .path,
            raw_path()
        );
        assert_eq!(typed_to_pb(restored), wire);
    }

    #[test]
    fn static_prefix_is_never_decoded_as_proven_full_layout() {
        let prefix = LayoutFact {
            ty: "[u8]".into(),
            size: None,
            align: None,
            sized: PredicateOutcome::NotProven,
            prefix_size: Some(0),
            prefix_align: Some(1),
            error: Some("full extent not established".into()),
            niches: None,
            fields: Vec::new(),
        };
        let expected = serde_json::to_value(&prefix).unwrap();
        assert_eq!(
            serde_json::to_value(layout_from_pb(layout_to_pb(prefix)).unwrap()).unwrap(),
            expected
        );
        assert!(
            layout_from_pb(pb::NativeLayoutFact {
                size: Some(0),
                align: Some(1),
                sized: pb::NativePredicateOutcome::NotProven.into(),
                ..Default::default()
            })
            .is_err()
        );
    }
}
