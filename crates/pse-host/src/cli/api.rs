// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native acquisition and passive queries over source/context-checked rustdoc artifacts.
use super::{GenerationIntent, HostError, ManagedCommand, OutputOptions};
use crate::{
    api::{
        self,
        query::{Kind, Query},
    },
    envelope::{self, Envelope},
    generation::EntryGuard,
    paths::CheckoutState,
};
use clap::Args;
use std::path::Path;

#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("api_effect").args(["prepare","compatibility","check_docs", "extract_examples", "retire_build"]).required(true).multiple(false)))]
pub(super) struct Arguments {
    #[command(flatten)]
    lookup: InspectArgs,
    /// Explicitly acquire documentation; passive reads use inspect api.
    #[arg(long,conflicts_with_all=["path","search","cursor","compare_with","native_examples"])]
    prepare: bool,
    /// Explicitly execute separately prepared compatibility rules.
    #[arg(long, requires = "compare_with", conflicts_with_all = ["cursor", "native_examples"])]
    compatibility: bool,
    /// Explicitly check workspace library documentation through native rustdoc.
    #[arg(long,conflicts_with_all=["compare_with","path","search","cursor","native_examples"])]
    check_docs: bool,
    /// Explicitly preprocess native doctests to format2; no examples compile or run.
    #[arg(long, conflicts_with_all=["path", "search", "cursor", "compare_with", "native_examples"])]
    extract_examples: bool,
    /// Preview the exact original acquisition's regenerable Cargo build output after drain.
    #[arg(long, value_name = "INVOCATION", conflicts_with_all=["path", "search", "cursor", "compare_with", "native_examples", "export_jsonl"])]
    retire_build: Option<String>,
    /// Release the Cargo output identity from the preview.
    #[arg(long, requires = "retire_build", value_name = "IDENTITY")]
    release: Option<String>,
}
#[derive(Debug, Args)]
pub(in crate::cli) struct InspectArgs {
    /// Export deterministic JSONL facts from an already acquired artifact into a new directory.
    #[arg(long, conflicts_with_all=["path", "search", "cursor", "compare_with", "native_examples", "adjacency", "kind", "input", "output_type", "where_shapes", "trait_name", "unsafe_only", "async_only", "const_only", "docs", "examples", "declaration"])]
    export_jsonl: Option<std::path::PathBuf>,
    /// Exact locked dependency name, name@version or source-qualified Cargo package ID.
    crate_query: String,
    /// Public crate-relative item path; omission lists candidates.
    path: Option<String>,
    /// Case-insensitive name/documentation search.
    #[arg(long, conflicts_with = "path")]
    search: Option<String>,
    /// Explicit target triple; default is the pinned rustc's observed host.
    #[arg(long)]
    target: Option<String>,
    #[arg(long, value_enum)]
    kind: Option<Kind>,
    #[arg(long)]
    input: Option<String>,
    /// Structural declared where predicate; may be repeated.
    #[arg(long = "where")]
    where_shapes: Vec<String>,
    #[arg(long)]
    output_type: Option<String>,
    #[arg(long)]
    trait_name: Option<String>,
    #[arg(long)]
    unsafe_only: bool,
    #[arg(long)]
    async_only: bool,
    #[arg(long)]
    const_only: bool,
    /// Expand authored documentation for returned candidates.
    #[arg(long)]
    docs: bool,
    /// Expand authored Rust examples and hidden setup; never execute them.
    #[arg(long)]
    examples: bool,
    /// Inspect an explicitly acquired doctests.json, preserving native transformations.
    #[arg(long, conflicts_with_all=["path", "search", "cursor", "compare_with"])]
    native_examples: Option<std::path::PathBuf>,
    /// Expand the original format-qualified rustdoc item, including complex bounds/associated data.
    #[arg(long)]
    declaration: bool,
    /// Examine selected declared adjacency; never acquires an analyzer or another artifact.
    #[arg(long, value_enum, requires = "path", conflicts_with_all=["cursor","compare_with","native_examples"])]
    adjacency: Vec<api::query::adjacency::Relation>,
    /// Read structural differences from an explicitly selected acquired document.json;
    /// --compatibility separately executes native rules. Neither generates an input.
    #[arg(long, conflicts_with_all=["path","search"])]
    compare_with: Option<std::path::PathBuf>,
    /// Continue the same immutable artifact/query page; stale contexts refuse.
    #[arg(long)]
    cursor: Option<String>,
}
impl Arguments {
    pub(super) fn intent(&self) -> GenerationIntent {
        if self.prepare
            || self.compatibility
            || self.check_docs
            || self.extract_examples
            || self.release.is_some()
        {
            GenerationIntent::NewWork
        } else {
            GenerationIntent::Passive
        }
    }
}
fn error(error: api::ApiError) -> HostError {
    HostError::Api(Box::new(error))
}
fn provider(
    context: &api::Context,
    artifact: Option<&api::Artifact>,
) -> Result<envelope::ProviderContext, HostError> {
    let mut provider = envelope::ProviderContext {
        provider: format!(
            "rustdoc-json/{}; {}",
            context.rustdoc_format,
            context
                .rustdoc_identity
                .lines()
                .next()
                .unwrap_or("version unknown")
        ),
        context: Some(format!(
            "{}; context={}; target={}; doc_features={:?}",
            context.package_id,
            context.key().map_err(error)?,
            context.target,
            context.features
        )),
        analysis_generation: artifact.map(|artifact| artifact.sha256.clone()),
        availability: if artifact.is_some() {
            envelope::SubjectState::Available
        } else {
            envelope::SubjectState::Unprepared
        },
        coordinates: Vec::new(),
    };
    if let Some(artifact) = artifact {
        provider
            .coordinates
            .push(envelope::SourceCoordinates::Utf8Bytes {
                source: artifact.path.display().to_string(),
                revision: Some(artifact.sha256.clone()),
                start: 0,
                end: usize::try_from(std::fs::metadata(&artifact.path)?.len())
                    .map_err(std::io::Error::other)?,
            });
    }
    Ok(provider)
}
fn unprepared(
    package_query: String,
    target: Option<String>,
    providers: Vec<envelope::ProviderContext>,
    artifacts: Vec<envelope::ArtifactReference>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let mut response = Envelope::response(
        "inspect api",
        if providers
            .iter()
            .any(|provider| provider.availability == envelope::SubjectState::Available)
        {
            "facade reused; defining crate needs explicit preparation"
        } else {
            "API context is unprepared; explicitly acquire this crate"
        },
        api::query::Report::Unprepared {
            package_query: package_query.clone(),
            requested_target: target.clone(),
        },
    );
    response.subject_state = envelope::SubjectState::Unprepared;
    response.providers = providers;
    response.artifacts = artifacts;
    let mut argv = vec![
        "pse".into(),
        "api".into(),
        package_query,
        "--prepare".into(),
    ];
    if let Some(target) = target {
        argv.extend(["--target".into(), target]);
    }
    response.next.push(envelope::NextAction::Command{argv,reason:"explicit compilation acquires this exact defining crate in its isolated documentation context; lookup itself does not compile".into()});
    response.observation.limitations.push(
        "unprepared external definitions are not absent APIs or empty successful searches".into(),
    );
    Ok(output.emit_page(response, envelope::DetailView::default())?)
}
pub(super) fn dispatch(
    arguments: Arguments,
    root: &Path,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let Arguments {
        lookup,
        prepare,
        compatibility,
        check_docs,
        extract_examples,
        retire_build,
        release,
    } = arguments;
    if let Some(invocation) = retire_build {
        let invocation = invocation.try_into().map_err(|_| {
            HostError::Ownership("API build retirement requires an exact invocation UUID".into())
        })?;
        let environment = std::env::vars_os().collect();
        let result = super::runtime()?.block_on(api::build_output::inspect_or_release(
            root,
            &environment,
            &lookup.crate_query,
            &invocation,
            release.as_deref(),
        ))?;
        return Ok(output.emit(Envelope::response(
            "api retire-build",
            "observed regenerable Cargo output under the original API acquisition lease",
            result,
        ))?);
    }
    if lookup.export_jsonl.is_some() {
        return Err(error(api::ApiError::Invalid(
            "--export-jsonl reads an acquired artifact: use inspect api separately from acquisition/check effects".into(),
        )));
    }
    let operation = if extract_examples {
        Operation::ExtractExamples
    } else if prepare {
        Operation::Prepare
    } else if compatibility {
        Operation::Compatibility
    } else if check_docs {
        Operation::CheckDocs
    } else {
        return Err(HostError::Ownership("API lookup is passive: use inspect api; explicit acquisition/checks select an operation".into()));
    };
    perform(lookup, root, output, Some((operation, entry)))
}
pub(in crate::cli) fn inspect(
    arguments: InspectArgs,
    root: &Path,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    perform(arguments, root, output, None)
}
#[derive(Clone, Copy, Debug)]
enum Operation {
    ExtractExamples,
    Prepare,
    Compatibility,
    CheckDocs,
}
fn perform(
    arguments: InspectArgs,
    root: &Path,
    output: &OutputOptions,
    effect: Option<(Operation, &EntryGuard)>,
) -> Result<i32, HostError> {
    // Validate the complete shape program before context lookup or explicit effects.
    let input_shape = arguments
        .input
        .as_deref()
        .map(api::query::TypePattern::parse)
        .transpose()
        .map_err(error)?;
    let output_shape = arguments
        .output_type
        .as_deref()
        .map(api::query::TypePattern::parse)
        .transpose()
        .map_err(error)?;
    let where_shapes = arguments
        .where_shapes
        .iter()
        .map(|v| api::query::WherePattern::parse(v))
        .collect::<Result<Vec<_>, _>>()
        .map_err(error)?;
    if let Some(path) = &arguments.native_examples {
        let page =
            api::doctests::inspect(&root.join(path), output.presentation.budget.unwrap_or(20))
                .map_err(error)?;
        let omitted = page.omitted_examples;
        let mut response = Envelope::response(
            "inspect api",
            "native doctest preprocessing; compilation and execution not run",
            api::query::Report::NativeExamples { page },
        );
        response.observation.scope = "addressed_native_doctest_format2_original_evidence".into();
        return Ok(output.emit_page(
            response,
            envelope::DetailView {
                omitted_records: omitted,
                truncated: omitted > 0,
                ..Default::default()
            },
        )?);
    }
    let environment = std::env::vars_os().collect();
    let store = CheckoutState::resolve(root, &environment)?.root;
    let snapshot = crate::dependency::capture(root, &environment)?;
    if let Some((Operation::CheckDocs, entry)) = effect {
        let package = crate::probe::select(&snapshot.metadata, &arguments.crate_query)
            .map_err(|error| HostError::Api(Box::new(error.into())))?;
        if !snapshot.metadata.workspace_members.contains(&package.id) {
            return Err(HostError::Ownership("documentation checking selects a workspace package; external API acquisition remains a separate explicit operation".into()));
        }
        return super::testing::start(
            crate::test_workflow::execute::Action::Rustdoc,
            crate::selection::Request {
                operation: crate::selection::Operation::Rustdoc,
                scope: crate::selection::Scope::Targets {
                    scope: Box::new(crate::selection::Scope::Packages(vec![
                        crate::selection::PackageIntent {
                            package: package.id.to_string(),
                            target: None,
                            features: crate::selection::Features::default(),
                        },
                    ])),
                    selector: crate::selection::CargoTargets::Library,
                },
                platform: arguments.target,
                filters: crate::selection::Filters::default(),
                non_incremental: false,
                ignored: crate::selection::IgnoredPolicy::Default,
            },
            "default".into(),
            super::testing::RunOptions::default(),
            Some(root),
            entry,
            output,
        );
    }
    if let Some((operation @ (Operation::Prepare | Operation::ExtractExamples), entry)) = effect {
        let native_examples = matches!(operation, Operation::ExtractExamples);
        let acquisition = api::prepare_mode(
            root,
            &store,
            &snapshot,
            &arguments.crate_query,
            arguments.target.as_deref(),
            &environment,
            native_examples,
        )
        .map_err(error)?;
        let build_target = api::build_output::Target::prepare(&acquisition, &output.invocation_id)?;
        let context_path = acquisition.directory.join("context.json");
        let context_bytes = std::fs::read(&context_path)?;
        let artifacts = vec![envelope::ArtifactReference {
            kind: "api_acquisition_context".into(),
            location: context_path.display().to_string(),
            digest: Some(crate::probe::digest(&context_bytes)),
        }];
        let providers = vec![envelope::ProviderContext {
            provider: format!(
                "rustdoc-{}/{}; {}",
                if native_examples { "doctest" } else { "json" },
                if native_examples {
                    api::doctests::FORMAT_VERSION
                } else {
                    acquisition.context.rustdoc_format
                },
                acquisition
                    .context
                    .rustdoc_identity
                    .lines()
                    .next()
                    .unwrap_or("version unknown")
            ),
            context: Some(format!(
                "{}; context={}; target={}; doc_features={:?}",
                acquisition.context.package_id,
                acquisition.context.key().map_err(error)?,
                acquisition.context.target,
                acquisition.context.features
            )),
            analysis_generation: None,
            availability: envelope::SubjectState::Unknown,
            coordinates: vec![envelope::SourceCoordinates::Utf8Bytes {
                source: context_path.display().to_string(),
                revision: Some(crate::probe::digest(&context_bytes)),
                start: 0,
                end: context_bytes.len(),
            }],
        }];
        return super::launch_managed_finalize(
            ManagedCommand {
                parent: None,
                root: root.to_owned(),
                cwd: acquisition.cwd.clone(),
                environment: acquisition.environment.clone(),
                command: acquisition.command.clone(),
                detach: false,
                session: None,
                resource_class: None,
                verb: if native_examples {
                    "api extract-examples"
                } else {
                    "api prepare"
                },
                tool: None,
                workflow: None,
            },
            entry,
            output,
            artifacts,
            providers,
            || {
                if native_examples {
                    use std::io::Read as _;
                    let state = CheckoutState::resolve(root, &acquisition.environment)?;
                    let invocation = crate::invocation::InvocationStore::new(state)
                        .directory(&output.invocation_id)?;
                    let mut bytes = Vec::new();
                    invocation
                        .stream(crate::extraction::OutputStream::Stdout)?
                        .read_to_end(&mut bytes)?;
                    let path =
                        api::doctests::finish(&acquisition, &snapshot, &bytes).map_err(error)?;
                    let artifact = envelope::ArtifactReference {
                        kind: "native_doctest_format2".into(),
                        location: path.display().to_string(),
                        digest: Some(crate::probe::digest(&bytes)),
                    };
                    super::runtime()?.block_on(
                        build_target.release_if_drained(root, &acquisition.environment),
                    )?;
                    return Ok(vec![artifact]);
                }
                let artifact = api::finish(&acquisition, &snapshot).map_err(error)?;
                super::runtime()?
                    .block_on(build_target.release_if_drained(root, &acquisition.environment))?;
                Ok(vec![envelope::ArtifactReference {
                    kind: "rustdoc_json".into(),
                    location: artifact.path.display().to_string(),
                    digest: Some(artifact.sha256),
                }])
            },
        );
    }
    let selected = api::locate(
        root,
        &store,
        &snapshot,
        &arguments.crate_query,
        arguments.target.as_deref(),
        &environment,
    )
    .map_err(error)?;
    let mut providers = vec![provider(&selected.context, selected.artifact.as_ref())?];
    let Some(mut artifact) = selected.artifact else {
        if arguments.export_jsonl.is_some() {
            return Err(error(api::ApiError::Invalid(
                "API export requires a prepared artifact; explicitly acquire it with api PACKAGE --prepare".into(),
            )));
        }
        if arguments.cursor.is_some() {
            return Err(error(api::ApiError::Invalid("API continuation source/context is unavailable; restart against the current context or select retained original evidence".into())));
        }
        return unprepared(
            arguments.crate_query,
            arguments.target,
            providers,
            Vec::new(),
            output,
        );
    };
    if let Some(directory) = arguments.export_jsonl {
        let manifest = api::export::write(&artifact, &root.join(directory)).map_err(error)?;
        let mut response = Envelope::response(
            "inspect api",
            "complete selected-artifact facts export; no acquisition or execution",
            api::query::Report::Export { manifest },
        );
        response.subject_state = envelope::SubjectState::Available;
        response.providers = providers;
        response.observation.scope = "one_acquired_rustdoc_artifact_export".into();
        return Ok(output.emit(response)?);
    }
    if let Some(baseline) = arguments.compare_with {
        let baseline = api::retained(&root.join(baseline)).map_err(error)?;
        if !matches!(effect, Some((Operation::Compatibility, _))) {
            let page = api::comparison::inspect(
                &baseline,
                &artifact,
                output.presentation.budget.unwrap_or(20),
                arguments.cursor.as_deref(),
            )
            .map_err(error)?;
            let detail = envelope::DetailView {
                budget: Some(output.presentation.budget.unwrap_or(20)),
                omitted_records: page.omitted_differences,
                truncated: page.omitted_differences > 0,
                ..envelope::DetailView::default()
            };
            let incomplete = page.missing_items != [0, 0];
            let mut response = Envelope::response(
                "inspect api",
                "native structural API differences; no compatibility rules executed",
                api::query::Report::Comparison { page },
            );
            response.subject_state = envelope::SubjectState::Available;
            response.providers = providers;
            response
                .providers
                .push(provider(&baseline.context, Some(&baseline))?);
            response.observation.scope =
                "two_addressed_rustdoc_artifacts_native_public_path_normalization".into();
            if incomplete {
                response.observation.completeness = envelope::Completeness::Partial;
            }
            return Ok(output.emit_page(response, detail)?);
        }
        let Some((Operation::Compatibility, entry)) = effect else {
            return Err(HostError::Ownership(
                "native compatibility requires an explicit performing operation".into(),
            ));
        };
        let comparison =
            api::comparison::prepare(root, &store, &baseline, &artifact, &environment)?;
        providers.push(provider(&baseline.context, Some(&baseline))?);
        return super::launch_managed_finalize(
            ManagedCommand {
                parent: None,
                root: root.to_owned(),
                cwd: root.to_owned(),
                environment,
                command: std::iter::once(comparison.executable.into_os_string())
                    .chain(comparison.arguments)
                    .collect(),
                detach: false,
                session: None,
                resource_class: None,
                verb: "api compare",
                tool: Some(super::NativeToolSelection {
                    kind: "cargo-semver-checks",
                    selection: comparison.version,
                    source_identity: format!("{}..{}", baseline.sha256, artifact.sha256),
                    executable_sha256: comparison.executable_sha256,
                }),
                workflow: None,
            },
            entry,
            output,
            comparison.artifacts,
            providers,
            || Ok(Vec::new()),
        );
    }
    let mut query = Query {
        input_shape,
        output_shape,
        where_shapes,
        path: arguments.path,
        search: arguments.search,
        kind: arguments.kind,
        trait_name: arguments.trait_name,
        unsafe_only: arguments.unsafe_only,
        async_only: arguments.async_only,
        const_only: arguments.const_only,
        docs: arguments.docs,
        examples: arguments.examples,
        declaration: arguments.declaration,
        surface_identity: None,
        request_identity: None,
        ..Query::default()
    };
    api::query::bind_request(&mut query, arguments.cursor.as_deref()).map_err(error)?;
    if !arguments.adjacency.is_empty() {
        let page = api::query::inspect(&artifact, &query, 1, None).map_err(error)?;
        let seed = page.candidates.first().ok_or_else(|| {
            error(api::ApiError::Invalid(
                "API adjacency requires one addressed declaration".into(),
            ))
        })?;
        let graph = api::query::adjacency::inspect(
            &artifact,
            rustdoc_types::Id(seed.item_id),
            &arguments.adjacency,
            output.presentation.budget.unwrap_or(20),
        )
        .map_err(error)?;
        let complete = graph.complete;
        let mut response = Envelope::response(
            "inspect api",
            "declared acquired API adjacency; contextual applicability remains separate",
            api::query::Report::Adjacency { graph },
        );
        response.providers = providers;
        response.observation.scope = "bounded_acquired_artifact_adjacency".into();
        if !complete {
            response.observation.completeness = envelope::Completeness::Partial;
        }
        return Ok(output.emit_page(response, envelope::DetailView::default())?);
    }
    let mut artifacts = Vec::new();
    let mut facades = Vec::new();
    let mut visited = std::collections::BTreeSet::from([artifact.context.package_id.clone()]);
    loop {
        let first = api::query::inspect(&artifact, &query, 1, None).map_err(error)?;
        if !first.candidates.is_empty() {
            break;
        }
        let Some(route) =
            api::reexport(&snapshot, &artifact, query.path.as_deref()).map_err(error)?
        else {
            break;
        };
        if !visited.insert(route.package_id.clone()) {
            return Err(error(api::ApiError::Invalid(
                "cyclic external reexport route; select the addressed defining crate".into(),
            )));
        }
        artifacts.push(envelope::ArtifactReference {
            kind: "rustdoc_facade".into(),
            location: artifact.path.display().to_string(),
            digest: Some(artifact.sha256.clone()),
        });
        let selected = api::locate(
            root,
            &store,
            &snapshot,
            &route.package_id,
            arguments.target.as_deref(),
            &environment,
        )
        .map_err(error)?;
        providers.push(provider(&selected.context, selected.artifact.as_ref())?);
        let Some(definition) = selected.artifact else {
            if arguments.cursor.is_some() {
                return Err(error(api::ApiError::Invalid("API continuation's defining artifact/context is unavailable; restart the query".into())));
            }
            return unprepared(
                route.package_id,
                arguments.target,
                providers,
                artifacts,
                output,
            );
        };
        let prefix=api::query::definition_prefix(&definition.document,&route.definition_path).ok_or_else(||error(api::ApiError::Invalid("defining artifact does not expose the reexport target through a known usable public path".into())))?;
        if query.path.is_some() {
            query.path = Some(if route.suffix.is_empty() {
                prefix.clone()
            } else {
                format!("{prefix}::{}", route.suffix)
            });
        }
        let identity = format!(
            "{}:{}",
            query.surface_identity.as_deref().unwrap_or(""),
            artifact.sha256
        );
        query.surface_identity = Some(crate::probe::digest(identity.as_bytes()));
        facades.push(api::query::Facade {
            package_id: artifact.context.package_id.clone(),
            context_key: artifact.context.key().map_err(error)?,
            artifact: artifact.path.clone(),
            artifact_sha256: artifact.sha256.clone(),
            public_prefix: route.public_prefix,
            definition_prefix: prefix,
        });
        artifact = definition;
    }
    let limit = output.presentation.budget.unwrap_or(20);
    let mut page = api::query::inspect(&artifact, &query, limit, arguments.cursor.as_deref())
        .map_err(error)?;
    for facade in facades.iter().rev() {
        for candidate in &mut page.candidates {
            if let Some(suffix) = candidate
                .public_path
                .strip_prefix(&facade.definition_prefix)
            {
                candidate.public_path = format!("{}{suffix}", facade.public_prefix);
            }
        }
    }
    page.facades = facades;
    let omitted = page.omitted_candidates;
    let no_match = page.candidates.is_empty();
    let mut response = Envelope::response(
        "inspect api",
        if no_match {
            "no matching declarations in the addressed API artifact"
        } else {
            "declared API candidates; ordinary-build applicability remains unknown"
        },
        api::query::Report::Prepared { page },
    );
    if no_match {
        response.subject_state = envelope::SubjectState::NoMatch;
    }
    response.observation.scope = "one_checked_rustdoc_artifact".into();
    response.providers = providers;
    response.artifacts = artifacts;
    response.artifacts.push(envelope::ArtifactReference {
        kind: "rustdoc_json".into(),
        location: artifact.path.display().to_string(),
        digest: Some(artifact.sha256),
    });
    Ok(output.emit_page(
        response,
        envelope::DetailView {
            budget: Some(limit),
            omitted_records: omitted,
            truncated: omitted > 0,
            ..Default::default()
        },
    )?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn bulk_export_is_a_whole_artifact_read_without_query_filters() {
        use clap::Parser as _;
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "inspect",
                "api",
                "dep",
                "--export-jsonl",
                "facts"
            ])
            .is_ok()
        );
        for filter in ["--docs", "--unsafe-only", "--declaration"] {
            assert!(
                super::super::Cli::try_parse_from([
                    "pse",
                    "inspect",
                    "api",
                    "dep",
                    "--export-jsonl",
                    "facts",
                    filter
                ])
                .is_err()
            );
        }
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "inspect",
                "api",
                "dep",
                "--export-jsonl",
                "facts",
                "--search",
                "Foo"
            ])
            .is_err()
        );
    }
    use clap::Parser as _;
    #[test]
    fn native_example_extraction_and_retained_inspection_have_separate_effect_controls() {
        assert!(
            super::super::Cli::try_parse_from(["pse", "api", "dep", "--extract-examples"]).is_ok()
        );
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "inspect",
                "api",
                "dep",
                "--native-examples",
                "doctests.json"
            ])
            .is_ok()
        );
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "api",
                "dep",
                "--extract-examples",
                "--prepare"
            ])
            .is_err()
        );
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "api",
                "dep",
                "--prepare",
                "--native-examples",
                "doctests.json"
            ])
            .is_err()
        );
    }
}
