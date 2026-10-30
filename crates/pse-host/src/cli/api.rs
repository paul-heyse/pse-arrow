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
pub(super) struct Arguments {
    /// Exact locked dependency name, name@version or source-qualified Cargo package ID.
    crate_query: String,
    /// Public crate-relative item path; omission lists candidates.
    path: Option<String>,
    /// Case-insensitive name/documentation search.
    #[arg(long, conflicts_with = "path")]
    search: Option<String>,
    /// Explicitly compile documentation in an isolated context; ordinary lookup never generates it.
    #[arg(long,conflicts_with_all=["path","search","cursor"])]
    prepare: bool,
    /// Explicit target triple; default is the pinned rustc's observed host.
    #[arg(long)]
    target: Option<String>,
    #[arg(long, value_enum)]
    kind: Option<Kind>,
    #[arg(long)]
    input: Option<String>,
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
    /// Expand the original format-qualified rustdoc item, including complex bounds/associated data.
    #[arg(long)]
    declaration: bool,
    /// Read structural differences from an explicitly selected acquired document.json;
    /// --compatibility separately executes native rules. Neither generates an input.
    #[arg(long, conflicts_with_all=["prepare","path","search"])]
    compare_with: Option<std::path::PathBuf>,
    /// Explicitly execute separately prepared native compatibility rules. Without
    /// this flag comparison only reads library-owned structural differences.
    #[arg(long, requires = "compare_with", conflicts_with = "cursor")]
    compatibility: bool,
    /// Explicitly check this workspace library's authored documentation with native
    /// rustdoc diagnostics; no API artifact or analyzer session is required.
    #[arg(long, conflicts_with_all=["prepare","compare_with","path","search","cursor"])]
    check_docs: bool,
    /// Continue the same immutable artifact/query page; stale contexts refuse.
    #[arg(long)]
    cursor: Option<String>,
}
impl Arguments {
    pub(super) fn intent(&self) -> GenerationIntent {
        if self.prepare || self.compatibility || self.check_docs {
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
        "api",
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
    let environment = std::env::vars_os().collect();
    let store = CheckoutState::resolve(root, &environment)?.root;
    let snapshot = crate::dependency::capture(root, &environment)?;
    if arguments.check_docs {
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
                ignored: crate::selection::IgnoredPolicy::Default,
            },
            "default".into(),
            super::testing::RunOptions::default(),
            Some(root),
            entry,
            output,
        );
    }
    if arguments.prepare {
        let acquisition = api::prepare(
            root,
            &store,
            &snapshot,
            &arguments.crate_query,
            arguments.target.as_deref(),
            &environment,
        )
        .map_err(error)?;
        let context_path = acquisition.directory.join("context.json");
        let context_bytes = std::fs::read(&context_path)?;
        let artifacts = vec![envelope::ArtifactReference {
            kind: "api_acquisition_context".into(),
            location: context_path.display().to_string(),
            digest: Some(crate::probe::digest(&context_bytes)),
        }];
        let providers = vec![envelope::ProviderContext {
            provider: format!(
                "rustdoc-json/{}; {}",
                acquisition.context.rustdoc_format,
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
                verb: "api prepare",
                tool: None,
                workflow: None,
            },
            entry,
            output,
            artifacts,
            providers,
            || {
                let artifact = api::finish(&acquisition, &snapshot).map_err(error)?;
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
    if let Some(baseline) = arguments.compare_with {
        let baseline = api::retained(&root.join(baseline)).map_err(error)?;
        if !arguments.compatibility {
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
                "api",
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
        path: arguments.path,
        search: arguments.search,
        kind: arguments.kind,
        input: arguments.input,
        output: arguments.output_type,
        trait_name: arguments.trait_name,
        unsafe_only: arguments.unsafe_only,
        async_only: arguments.async_only,
        const_only: arguments.const_only,
        docs: arguments.docs,
        examples: arguments.examples,
        declaration: arguments.declaration,
        surface_identity: None,
        request_identity: None,
    };
    api::query::bind_request(&mut query, arguments.cursor.as_deref()).map_err(error)?;
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
        "api",
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
