// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque product delegation; scientific request types remain in the selected product.
pub(super) mod context;
mod effect;
use super::{GenerationIntent, HostError, OutputOptions, checkout_root};
use crate::{
    envelope::{self, Envelope},
    environment::Environment,
    investigation::NativePath,
    product_reader::{self, Contract, ProductReaderSelection},
};
use clap::{Args, Subcommand, ValueEnum};
pub(super) use effect::ProjectEffectView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{
    io::{self, Read, Write},
    num::{NonZeroU64, NonZeroUsize},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explicitly build and capture a product reader; never performed by passive reads.
    Prepare {
        /// Include the normalized installed-schema observer.
        #[arg(long)]
        installed_schema: bool,
    },
    /// Discover the operations and schema associations of one captured reader.
    Discover {
        #[arg(long)]
        reader: String,
    },
    /// Read a directional schema from the selected product, without reconstructing it.
    Schema {
        #[arg(long)]
        reader: String,
        #[arg(value_enum)]
        contract: Direction,
    },
    /// Delegate a product-owned JSON request; discover/schema describe its accepted grammar.
    Query {
        #[arg(long)]
        reader: String,
        /// Existing JSON file containing only the product request, not a transport wrapper.
        #[arg(long)]
        request: PathBuf,
    },
    /// Explicitly admit a product-owned diagnostic capture or selected evidence publication.
    Investigate {
        #[arg(long)]
        reader: String,
        /// Product-owned investigate request; passive query never grants attachment permission.
        #[arg(long)]
        request: PathBuf,
    },
    /// Follow opaque product-issued passive continuations; closing the view only stops observation.
    Follow {
        #[arg(long)]
        reader: String,
        /// Initial product follow request; resume coordinates remain product-owned.
        #[arg(long)]
        request: PathBuf,
        /// Maximum whole snapshots emitted before observation stops.
        #[arg(long, default_value_t = 32, value_parser = clap::value_parser!(u32).range(1..))]
        samples: u32,
        /// Minimum polling interval; native progress never awaits this renderer.
        #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(1..))]
        interval_ms: u64,
        /// Observer allowance; its expiry does not time out the original workload.
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
        timeout: u64,
    },
}
#[derive(Debug, Subcommand)]
pub(super) enum ProjectAction {
    /// Authenticate configured target choices through their connection-only readers.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    #[command(add(crate::catalog::Examples(&[&[]])))]
    Targets {
        #[command(flatten)]
        context: context::Options,
    },
    /// Verify and inspect an existing project pack offline.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    InspectPack {
        /// Exact reader identifier returned by `pse observe prepare`.
        #[arg(long)]
        reader: String,
        /// Directory containing the logical project pack.
        input: PathBuf,
        /// Optional bound on bytes in an individual manifest or document.
        #[arg(long)]
        maximum_document_bytes: Option<NonZeroUsize>,
    },
    /// Read one bounded metadata page from an existing canonical deployment.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Metadata {
        #[command(flatten)]
        source: ProjectSource,
        /// Original continuation JSON from the selected metadata page.
        #[arg(long)]
        cursor: Option<ProjectCursor>,
        #[command(subcommand)]
        collection: ProjectCollection,
    },
    /// Read an exact project header.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Show {
        #[command(flatten)]
        source: ProjectSource,
        /// Canonical project key.
        project: String,
    },
    /// Read one generation-fenced branch page for an exact project.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Branches {
        #[command(flatten)]
        source: ProjectSource,
        /// Canonical project key.
        project: String,
        /// Original continuation JSON from the selected branch page.
        #[arg(long)]
        cursor: Option<ProjectCursor>,
    },
    /// Read an exact branch header.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Branch {
        #[command(flatten)]
        source: ProjectSource,
        /// Canonical branch key.
        branch: String,
    },
    /// Read an exact draft head and version descriptor.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    DraftShow {
        #[command(flatten)]
        source: ProjectSource,
        /// Canonical draft key.
        key: String,
    },
    /// Browse one bounded page of compact study headers.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    StudyWorkbench {
        #[command(flatten)]
        source: ProjectSource,
        #[command(flatten)]
        request: StudyWorkbenchQuery,
    },
    /// Read bounded research metadata without opening protected results.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    Research {
        #[command(flatten)]
        source: ProjectSource,
        #[command(subcommand)]
        selection: ResearchQuery,
    },
    /// Describe the closure of an already-issued project selection.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    DescribeClosure {
        #[command(flatten)]
        source: ProjectSource,
        /// Authored closure request, read from stdin when omitted.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Observe installed canonical schema through the selected Viewer connection.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    InstalledSchema {
        #[command(flatten)]
        source: ProjectSource,
        /// Expected installed-store issuer. Required with direct state selection.
        #[arg(long)]
        issuing_store: Option<uuid::Uuid>,
    },
    #[command(flatten)]
    Effect(effect::Action),
}
#[derive(Debug, Args)]
pub(super) struct ProjectSource {
    /// Exact reader identifier returned by `pse observe prepare`.
    #[arg(long, requires = "state", conflicts_with = "target")]
    reader: Option<String>,
    /// Exact existing private canonical deployment state directory.
    #[arg(long, requires = "reader", conflicts_with_all = ["target", "context_file", "context_profile"])]
    state: Option<PathBuf>,
    /// Exact existing database; defaults to the database configured by the deployment.
    #[arg(long, requires = "state")]
    database: Option<String>,
    /// Exact alias from the checkout's private CLI settings.
    #[arg(long)]
    target: Option<String>,
    #[command(flatten)]
    context: context::Options,
    /// Total reader allowance in milliseconds, from 1 through 25000.
    #[arg(long)]
    timeout_ms: Option<NonZeroU64>,
}

impl ProjectSource {
    fn resolve(
        self,
        root: &Path,
        selected: Option<&Path>,
        environment: &Environment,
        output: &OutputOptions,
    ) -> Result<Result<context::ResolvedSource, i32>, HostError> {
        if self
            .timeout_ms
            .is_some_and(|timeout| timeout.get() > 25_000)
        {
            return Err(HostError::Ownership(
                "project timeout requires 1 through 25000 milliseconds".into(),
            ));
        }
        if let Some(state) = self.state {
            let state = std::path::absolute(super::source_path(selected, state)?)?;
            let reader = self.reader.ok_or_else(|| {
                HostError::Ownership("direct project selection requires a prepared reader".into())
            })?;
            return Ok(Ok(context::ResolvedSource {
                state,
                database: self.database,
                reader,
                issuing_store: None,
                timeout_ms: self.timeout_ms,
            }));
        }
        let context = context::Context::load(
            root,
            environment,
            &self.context,
            self.target,
            self.timeout_ms,
        )?;
        if let Some(selected) = context.selected() {
            return Ok(Ok(selected));
        }
        if context.candidates().next().is_none() {
            return Err(HostError::Ownership(
                "configure a named product target or select --state and --reader".into(),
            ));
        }
        Ok(Err(emit_project_targets(
            root,
            environment,
            &context,
            output,
            true,
        )?))
    }
}
#[derive(Debug, Args, Serialize)]
pub(super) struct StudyWorkbenchQuery {
    #[command(flatten)]
    filter: StudyWorkbenchFilter,
    /// Maximum study headers in one page, from 1 through 64.
    #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u32).range(1..=64))]
    maximum: u32,
    /// Original continuation JSON. Repeat the same filters to continue.
    #[arg(long)]
    cursor: Option<ProjectCursor>,
}
#[derive(Debug, Args, Serialize)]
struct StudyWorkbenchFilter {
    /// Exact study key.
    #[arg(long)]
    study: Option<String>,
    /// Exact owning project key.
    #[arg(long)]
    project: Option<String>,
    /// Exact branch key.
    #[arg(long)]
    branch: Option<String>,
    /// Exact modeling source revision.
    #[arg(long)]
    source: Option<String>,
    /// Exact admitted physical source revision.
    #[arg(long)]
    physical_revision: Option<String>,
    /// Exact immutable engineering context key.
    #[arg(long)]
    context: Option<String>,
    /// Terminal outcome from the selected product's request schema.
    #[arg(long)]
    outcome: Option<String>,
    /// Exact dataset, selection, publication operation or research-reference key.
    #[arg(long)]
    research_reference: Option<String>,
}
#[derive(Debug, Subcommand)]
pub(super) enum ProjectCollection {
    /// Discover available project headers.
    Projects,
    /// Read draft headers by branch key.
    Drafts { branch: String },
    /// Read bookmark headers by project key.
    Bookmarks { project: String },
    /// Read savepoint headers by draft key.
    Savepoints { draft: String },
    /// Read admission diagnostics by draft-version key.
    Diagnostics { version: String },
}
#[derive(Debug, Subcommand, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum ResearchQuery {
    /// Read one indexed page of dataset headers for this project.
    Datasets {
        project: String,
        /// Continue after this immutable dataset key.
        #[arg(long, default_value = "")]
        after: String,
        /// Maximum metadata rows in the page.
        #[arg(long, default_value_t = 128, value_parser = clap::value_parser!(u32).range(1..=128))]
        limit: u32,
    },
    /// Read one original dataset version.
    Dataset { key: String },
    /// Read one original frozen selection.
    Selection { key: String },
}
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub(super) struct ProjectCursor(Box<RawValue>);
impl std::str::FromStr for ProjectCursor {
    type Err = serde_json::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(value).map(Self)
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ProjectQuery {
    Target,
    Metadata {
        request: ProjectMetadataQuery,
    },
    Show {
        project: String,
    },
    Branches {
        project: String,
        cursor: Option<ProjectCursor>,
    },
    Branch {
        branch: String,
    },
    DraftShow {
        key: String,
    },
    StudyWorkbench {
        request: StudyWorkbenchQuery,
    },
    ResearchRead {
        request: ResearchQuery,
    },
    Closure {
        request: Box<RawValue>,
    },
    ImportStatus {
        intent: Box<RawValue>,
    },
}
#[derive(Serialize)]
struct ProjectMetadataQuery {
    kind: &'static str,
    owner: String,
    cursor: Option<ProjectCursor>,
}
impl ProjectCollection {
    fn query(self, cursor: Option<ProjectCursor>) -> ProjectQuery {
        let (kind, owner) = match self {
            Self::Projects => ("projects", String::new()),
            Self::Drafts { branch } => ("drafts", branch),
            Self::Bookmarks { project } => ("bookmarks", project),
            Self::Savepoints { draft } => ("savepoints", draft),
            Self::Diagnostics { version } => ("diagnostics", version),
        };
        ProjectQuery::Metadata {
            request: ProjectMetadataQuery {
                kind,
                owner,
                cursor,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Direction {
    Request,
    Response,
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Prepare { .. } | Self::Investigate { .. } => GenerationIntent::NewWork,
            _ => GenerationIntent::Passive,
        }
    }
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct PreparedReader {
    /// Resolver-owned selection key; never an arbitrary executable path.
    reader: String,
    /// Original source context at preparation, not current-source freshness.
    source_identity: String,
    /// Captured executable content identity, independent of a mutable target directory.
    executable_sha256: String,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct ProductView<'a> {
    /// Exact selected immutable product reader.
    reader: &'a str,
    /// Product transport category, independent of the observed scientific outcome.
    category: &'a str,
    /// Original lexical product payload; the product schema owns its scientific meaning.
    product: &'a RawValue,
    /// Opaque product-issued continuation when the selected operation supports one.
    next_request: Option<&'a RawValue>,
    /// Independently checked native unit association and aggregate facts.
    native: product_reader::native::Association,
}
/// Each whole stream record declares observer lifecycle independently of scientific state.
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct FollowView<'a> {
    reader: &'a str,
    observer_state: &'static str,
    sample: usize,
    product: Option<&'a RawValue>,
    next_request: Option<&'a RawValue>,
    summary: Option<&'a product_reader::follow::Summary>,
    native: Option<product_reader::native::Association>,
}
pub(super) fn product_error(error: product_reader::ProductReaderError) -> HostError {
    HostError::ProductReader(Box::new(error))
}
fn request_file(path: &Path) -> Result<Box<RawValue>, HostError> {
    read_request(std::fs::File::open(path)?)
}
fn read_request(input: impl Read) -> Result<Box<RawValue>, HostError> {
    const MAXIMUM: usize = 1024 * 1024;
    let mut bytes = Vec::new();
    input.take(MAXIMUM as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAXIMUM {
        return Err(HostError::Ownership(
            "product request exceeds 1 MiB transport extent".into(),
        ));
    }
    serde_json::from_slice(&bytes).map_err(|error| {
        HostError::Ownership(format!(
            "product request is invalid JSON ({:?}, line {}, column {})",
            error.classify(),
            error.line(),
            error.column()
        ))
    })
}
fn project_pack_request(
    input: PathBuf,
    maximum_document_bytes: Option<NonZeroUsize>,
) -> Result<Box<RawValue>, HostError> {
    #[derive(Serialize)]
    #[serde(tag = "operation", rename_all = "snake_case")]
    enum Request {
        InspectProjectPack {
            input: NativePath,
            #[serde(skip_serializing_if = "Option::is_none")]
            maximum_document_bytes: Option<NonZeroUsize>,
        },
    }
    serde_json::value::to_raw_value(&Request::InspectProjectPack {
        input: NativePath::from(input),
        maximum_document_bytes,
    })
    .map_err(|error| io::Error::other(error).into())
}
fn online_project_request(
    source: &context::ResolvedSource,
    query: ProjectQuery,
) -> Result<Box<RawValue>, HostError> {
    #[derive(Serialize)]
    struct Target<'a> {
        state: NativePath,
        database: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_issuer: Option<&'a str>,
        timeout_ms: Option<NonZeroU64>,
    }
    #[derive(Serialize)]
    struct Request<'a> {
        operation: &'static str,
        canonical: Target<'a>,
        query: ProjectQuery,
    }
    serde_json::value::to_raw_value(&Request {
        operation: "project",
        canonical: Target {
            state: NativePath::from(source.state.clone()),
            database: source.database.as_deref(),
            expected_issuer: source.issuing_store.as_deref(),
            timeout_ms: source.timeout_ms,
        },
        query,
    })
    .map_err(|error| io::Error::other(error).into())
}
impl ProjectAction {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Effect(action) => action.intent(),
            _ => GenerationIntent::Passive,
        }
    }

    fn request(
        self,
        selected: Option<&Path>,
        root: &Path,
        environment: &Environment,
        output: &OutputOptions,
    ) -> Result<Result<(&'static str, String, Box<RawValue>), i32>, HostError> {
        let resolve =
            |path| Ok::<_, HostError>(std::path::absolute(super::source_path(selected, path)?)?);
        let (command, source, query) = match self {
            Self::InspectPack {
                reader,
                input,
                maximum_document_bytes,
            } => {
                return Ok(Ok((
                    "project inspect-pack",
                    reader,
                    project_pack_request(resolve(input)?, maximum_document_bytes)?,
                )));
            }
            Self::Metadata {
                source,
                cursor,
                collection,
            } => ("project metadata", source, collection.query(cursor)),
            Self::Show { source, project } => {
                ("project show", source, ProjectQuery::Show { project })
            }
            Self::Branches {
                source,
                project,
                cursor,
            } => (
                "project branches",
                source,
                ProjectQuery::Branches { project, cursor },
            ),
            Self::Branch { source, branch } => {
                ("project branch", source, ProjectQuery::Branch { branch })
            }
            Self::DraftShow { source, key } => (
                "project draft-show",
                source,
                ProjectQuery::DraftShow { key },
            ),
            Self::StudyWorkbench { source, request } => (
                "project study-workbench",
                source,
                ProjectQuery::StudyWorkbench { request },
            ),
            Self::Research { source, selection } => (
                "project research",
                source,
                ProjectQuery::ResearchRead { request: selection },
            ),
            Self::DescribeClosure { source, intent } => {
                let source = match source.resolve(root, selected, environment, output)? {
                    Ok(source) => source,
                    Err(code) => return Ok(Err(code)),
                };
                let request = match intent {
                    Some(path) => request_file(&resolve(path)?)?,
                    None => read_request(io::stdin().lock())?,
                };
                let request = online_project_request(&source, ProjectQuery::Closure { request })?;
                return Ok(Ok(("project describe-closure", source.reader, request)));
            }
            Self::InstalledSchema {
                source,
                issuing_store,
            } => {
                #[derive(Serialize)]
                struct Request {
                    operation: &'static str,
                    state: NativePath,
                    database: Option<String>,
                    expected_issuer: String,
                    timeout_ms: Option<NonZeroU64>,
                }
                let source = match source.resolve(root, selected, environment, output)? {
                    Ok(source) => source,
                    Err(code) => return Ok(Err(code)),
                };
                let explicit = issuing_store.map(|issuer| issuer.to_string());
                if let (Some(configured), Some(explicit)) = (&source.issuing_store, &explicit)
                    && configured != explicit
                {
                    return Err(HostError::Ownership(
                        "expected schema issuer differs from the configured target".into(),
                    ));
                }
                let expected_issuer = source.issuing_store.or(explicit).ok_or_else(|| {
                    HostError::Ownership(
                        "direct installed schema observation requires --issuing-store".into(),
                    )
                })?;
                let request = serde_json::value::to_raw_value(&Request {
                    operation: "inspect_canonical_catalog",
                    state: source.state.into(),
                    database: source.database,
                    expected_issuer,
                    timeout_ms: source.timeout_ms,
                })
                .map_err(io::Error::other)?;
                return Ok(Ok(("project installed-schema", source.reader, request)));
            }
            Self::Effect(_) | Self::Targets { .. } => {
                return Err(HostError::Ownership(
                    "project effect requires its performing owner".into(),
                ));
            }
        };
        let source = match source.resolve(root, selected, environment, output)? {
            Ok(source) => source,
            Err(code) => return Ok(Err(code)),
        };
        let request = online_project_request(&source, query)?;
        Ok(Ok((command, source.reader, request)))
    }
}

#[derive(Serialize, JsonSchema)]
pub(super) struct ProjectTargetsView {
    choices: Vec<TargetChoice>,
    unavailable: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
struct TargetChoice {
    selector: String,
    namespace: String,
    database: String,
    issuing_store: String,
}

fn observe_target(
    root: &Path,
    environment: &Environment,
    source: &context::ResolvedSource,
    correlation: &str,
) -> Option<ObservedTarget> {
    #[derive(Deserialize)]
    struct Observation {
        kind: String,
        target: ObservedTarget,
        result: TargetResult,
    }
    #[derive(Deserialize)]
    struct TargetResult {
        kind: String,
    }

    let reader = ProductReaderSelection::open(root, environment, &source.reader).ok()?;
    let request = online_project_request(source, ProjectQuery::Target).ok()?;
    let reply = reader.read(&request, correlation, environment).ok()?;
    if reply.category() != product_reader::Category::Success {
        return None;
    }
    let observation: Observation = serde_json::from_str(reply.payload().get()).ok()?;
    (observation.kind == "project"
        && observation.result.kind == "target"
        && source.database.as_deref() == Some(observation.target.database.as_str())
        && source.issuing_store.as_deref() == Some(observation.target.issuing_store.as_str()))
    .then_some(observation.target)
}

#[derive(Deserialize)]
struct ObservedTarget {
    namespace: String,
    database: String,
    issuing_store: String,
}

fn emit_project_targets(
    root: &Path,
    environment: &Environment,
    context: &context::Context,
    output: &OutputOptions,
    ambiguous: bool,
) -> Result<i32, HostError> {
    let policy = envelope::RedactionPolicy::from_inputs(environment);
    let mut view = ProjectTargetsView {
        choices: Vec::new(),
        unavailable: Vec::new(),
    };
    for (name, source) in context.candidates() {
        let selector = policy.redact(name);
        match observe_target(root, environment, &source, output.invocation_id.as_str()) {
            Some(target) => view.choices.push(TargetChoice {
                selector,
                namespace: policy.redact(&target.namespace),
                database: policy.redact(&target.database),
                issuing_store: policy.redact(&target.issuing_store),
            }),
            None => view.unavailable.push(selector),
        }
    }
    let mut response = Envelope::response(
        "project targets",
        if ambiguous {
            "choose an exact --target before dispatch"
        } else {
            "configured target authentication observed"
        },
        view,
    );
    if ambiguous {
        response.outcome = envelope::CommandOutcome::Usage;
    }
    if !response.payload.unavailable.is_empty() {
        response.observation.completeness = envelope::Completeness::Partial;
    }
    if ambiguous {
        // These are required decision inputs from a different response schema.
        // A field selection for the original operation cannot suppress them.
        let mut decision = output.clone();
        decision.presentation = envelope::Presentation::default();
        return Ok(decision.emit(response)?);
    }
    Ok(output.emit(response)?)
}
pub(super) fn project_dispatch(
    action: ProjectAction,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?.canonicalize()?;
    let environment: Environment = std::env::vars_os().collect();
    if let ProjectAction::Effect(action) = action {
        return effect::dispatch(action, &root, selected, output, &environment);
    }
    if let ProjectAction::Targets { context } = action {
        let context = context::Context::load(&root, &environment, &context, None, None)?;
        return emit_project_targets(&root, &environment, &context, output, false);
    }
    let (command, reader, request) = match action.request(selected, &root, &environment, output)? {
        Ok(request) => request,
        Err(code) => return Ok(code),
    };
    let reader =
        ProductReaderSelection::open(&root, &environment, &reader).map_err(product_error)?;
    emit_project_read(command, &reader, &request, output, &environment)
}

fn emit_project_read(
    command: &str,
    reader: &ProductReaderSelection,
    request: &RawValue,
    output: &OutputOptions,
    environment: &Environment,
) -> Result<i32, HostError> {
    let reply = reader
        .read(request, output.invocation_id.as_str(), environment)
        .map_err(product_error)?;
    if output.format == envelope::OutputFormat::Human && !reply.stderr().is_empty() {
        let policy = envelope::RedactionPolicy::from_inputs(environment);
        io::stderr().lock().write_all(
            policy
                .redact(&String::from_utf8_lossy(reply.stderr()))
                .as_bytes(),
        )?;
    }
    let (category, text) = match (reply.category(), command == "project inspect-pack") {
        (product_reader::Category::Success, true) => ("success", "offline project pack verified"),
        (product_reader::Category::Refused, true) => {
            ("refused", "offline project pack inspection refused")
        }
        (product_reader::Category::Success, false) => {
            ("success", "canonical project data observed")
        }
        (product_reader::Category::Refused, false) => ("refused", "canonical project read refused"),
    };
    let mut response = Envelope::response(
        command,
        text,
        ProductView {
            reader: reader.id(),
            category,
            product: reply.payload(),
            next_request: None,
            native: product_reader::native::Association::Absent,
        },
    );
    response.observation.scope = if command == "project inspect-pack" {
        "offline_project_pack"
    } else if command == "project import-status" {
        "project_import"
    } else if command == "project installed-schema" {
        "canonical_installed_schema"
    } else {
        "canonical_project_metadata"
    }
    .into();
    if category == "refused" {
        #[derive(Deserialize)]
        struct Refusal {
            failure: Failure,
        }
        #[derive(Deserialize)]
        struct Failure {
            native: Option<envelope::ErrorRecord>,
        }
        if let Ok(refusal) = serde_json::from_str::<Refusal>(reply.payload().get())
            && let Some(native) = refusal.failure.native
        {
            response.failures.push(native);
        }
        response.outcome = envelope::CommandOutcome::Usage;
    }
    Ok(output.emit(response)?)
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment: Environment = std::env::vars_os().collect();
    if let Action::Follow {
        reader,
        request,
        samples,
        interval_ms,
        timeout,
    } = &action
    {
        let reader =
            ProductReaderSelection::open(&root, &environment, reader).map_err(product_error)?;
        let request = request_file(&super::source_path(selected, request.clone())?)?;
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(*timeout))
            .ok_or_else(|| {
                HostError::Ownership("observer deadline exceeds monotonic horizon".into())
            })?;
        return follow(
            reader,
            environment,
            request,
            product_reader::follow::Limits {
                samples: *samples as usize,
                interval: Duration::from_millis(*interval_ms),
                deadline,
            },
            output,
            &root,
        );
    }
    if let Action::Prepare { installed_schema } = action {
        let reader = product_reader::prepare_selected(&root, &environment, installed_schema)
            .map_err(product_error)?;
        return Ok(output.emit(Envelope::response(
            "observe prepare",
            "native product reader explicitly prepared",
            PreparedReader {
                reader: reader.id().to_owned(),
                source_identity: reader.source_identity().map_err(product_error)?,
                executable_sha256: reader.executable().sha256.clone(),
            },
        ))?);
    }
    let (reader_id, request, investigation) = match action {
        Action::Prepare { .. } => {
            return Err(HostError::Ownership(
                "unreachable product preparation dispatch".into(),
            ));
        }
        Action::Follow { .. } => {
            return Err(HostError::Ownership("unreachable follow dispatch".into()));
        }
        Action::Schema { reader, contract } => {
            let selection = ProductReaderSelection::open(&root, &environment, &reader)
                .map_err(product_error)?;
            let schema = selection
                .schema(match contract {
                    Direction::Request => Contract::Request,
                    Direction::Response => Contract::Response,
                })
                .map_err(product_error)?;
            // This is the product's actual directional schema, not the host envelope schema.
            writeln!(io::stdout().lock(), "{}", schema.get())?;
            return Ok(0);
        }
        Action::Discover { reader } => (reader, None, false),
        Action::Query { reader, request } => (
            reader,
            Some(request_file(&super::source_path(selected, request)?)?),
            false,
        ),
        Action::Investigate { reader, request } => (
            reader,
            Some(request_file(&super::source_path(selected, request)?)?),
            true,
        ),
    };
    let reader =
        ProductReaderSelection::open(&root, &environment, &reader_id).map_err(product_error)?;
    let (command, payload, category, next_request, native) = match request {
        None => {
            // Captured discovery is a complete transport frame from this same executable.
            #[derive(serde::Deserialize)]
            struct Discovery {
                payload: Box<RawValue>,
            }
            let captured: Discovery =
                serde_json::from_slice(&reader.discovery().map_err(product_error)?)
                    .map_err(io::Error::other)?;
            (
                "observe discover",
                captured.payload,
                "success",
                None,
                product_reader::native::Association::Absent,
            )
        }
        Some(request) => {
            let reply = if investigation {
                reader.investigate(&request, output.invocation_id.as_str(), &environment)
            } else {
                reader.read(&request, output.invocation_id.as_str(), &environment)
            }
            .map_err(product_error)?;
            if !reply.stderr().is_empty() {
                io::stderr().lock().write_all(reply.stderr())?;
            }
            let category = match reply.category() {
                product_reader::Category::Success => "success",
                product_reader::Category::Refused => "refused",
            };
            let native = super::runtime()?.block_on(product_reader::native::inspect(
                &root,
                &environment,
                reply.managed_invocation(),
            ));
            (
                if investigation {
                    "observe investigate"
                } else {
                    "observe query"
                },
                reply.payload().to_owned(),
                category,
                reply.next_request().map(ToOwned::to_owned),
                native,
            )
        }
    };
    let next = native.next_actions();
    let mut response = Envelope::response(
        command,
        "selected native product response observed",
        ProductView {
            reader: reader.id(),
            category,
            product: &payload,
            next_request: next_request.as_deref(),
            native,
        },
    );
    response.observation.scope = "captured_product_reader_transport".into();
    response.next.extend(next);
    response.observation.limitations.push("Scientific identities, outcome, source coverage and validity remain in the original product payload.".into());
    response.observation.limitations.push("Native unit accounting includes its descendants, is independent of process/scientific facts, and is not an atomic snapshot.".into());
    if category == "refused" {
        response.outcome = envelope::CommandOutcome::Usage;
    }
    Ok(output.emit(response)?)
}

fn follow(
    reader: ProductReaderSelection,
    environment: Environment,
    request: Box<RawValue>,
    limits: product_reader::follow::Limits,
    output: &OutputOptions,
    root: &Path,
) -> Result<i32, HostError> {
    if output.presentation.fields.is_some() || output.presentation.budget.is_some() {
        return Err(HostError::Ownership("follow emits producer-owned whole snapshots; select detail in the product request rather than --fields/--budget".into()));
    }
    let reader = Arc::new(reader);
    let environment = Arc::new(environment);
    super::runtime()?.block_on(async {
        let stdout = crate::observer_io::Writer::new(io::stdout())?;
        let stderr = crate::observer_io::Writer::new(io::stderr())?;
        let control = pse_diagnostics::Cancellation::default();
        let count = std::cell::Cell::new(0);
        let refused = std::cell::Cell::new(false);
        let complete_frame = std::cell::Cell::new(true);
        let mut interrupt =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        let observing = product_reader::follow::run(
            reader.clone(),
            environment.clone(),
            output.invocation_id.as_str().into(),
            request,
            limits,
            control.clone(),
            |reply| {
                let count = &count;
                let refused = &refused;
                let complete_frame = &complete_frame;
                let reader = &reader;
                let stdout = &stdout;
                let stderr = &stderr;
                let environment = &environment;
                async move {
                    complete_frame.set(false);
                    refused.set(refused.get() || reply.category() == product_reader::Category::Refused);
                    stderr.write(reply.stderr()).await?;
                    let native = product_reader::native::inspect(root, environment, reply.managed_invocation()).await;
                    let next = native.next_actions();
                    let mut response = Envelope::response(
                        "observe follow", "whole product follow sample", FollowView {
                            reader: reader.id(), observer_state: "observing", sample: count.get() + 1,
                            product: Some(reply.payload()), next_request: reply.next_request(), summary: None,
                            native: Some(native),
                        },
                    );
                    response.observation.scope = "captured_product_reader_transport".into();
                    response.next.extend(next);
                    response.observation.limitations.push("Native unit accounting includes its descendants and is independent of scientific process facts; the combined read is not atomic.".into());
                    if refused.get() { response.outcome = envelope::CommandOutcome::Usage; }
                    let (_, frame) = output.encode_frame(response)?;
                    drop(reply);
                    stdout.write(&frame).await?;
                    count.set(count.get() + 1);
                    complete_frame.set(true);
                    Ok(())
                }
            },
        );
        tokio::pin!(observing);
        let result = tokio::select! {
            result = &mut observing => result,
            _ = interrupt.recv() => { control.cancel(); observing.await },
            _ = terminate.recv() => { control.cancel(); observing.await },
        };
        let summary = match result {
            Ok(summary) => summary,
            Err(error) => {
                let mut failure = Envelope::response("observe follow", "observation failed; original work retains its own owner", ());
                failure.outcome = envelope::CommandOutcome::Failure;
                failure.observation.completeness = envelope::Completeness::Partial;
                failure.failures.push(envelope::ErrorRecord::from_typed(&error, &envelope::RedactionPolicy::from_inputs(&std::env::vars_os().collect())));
                if let Ok((_, frame)) = output.encode_frame(failure) {
                    // A broken/blocked renderer cannot send us through the synchronous
                    // command-level error printer and strand observer signal settlement.
                    let _ = tokio::time::timeout(Duration::from_secs(1), stderr.write(&frame)).await;
                }
                return Ok(1);
            }
        };
        if !complete_frame.get() {
            // A caller interrupted delivery, not a byte budget. Never append a second
            // frame to an incomplete first frame or claim required output completed.
            return Ok(match summary.stop {
                product_reader::follow::Stop::Cancelled => 130,
                product_reader::follow::Stop::Deadline => 124,
                _ => 1,
            });
        }
        let mut response = Envelope::response(
            "observe follow", "observation ended; original work retains its own owner", FollowView {
                reader: reader.id(), observer_state: "stopped", sample: count.get(),
                product: None, next_request: None, summary: Some(&summary),
                native: None,
            },
        );
        response.observation.scope = "captured_product_reader_transport".into();
        response.observation.timed_out = matches!(summary.stop, product_reader::follow::Stop::Deadline);
        if refused.get() { response.outcome = envelope::CommandOutcome::Usage; }
        if matches!(summary.stop, product_reader::follow::Stop::Cancelled) { response.outcome = envelope::CommandOutcome::Interrupted; }
        let (code, frame) = output.encode_frame(response)?;
        match tokio::time::timeout(Duration::from_secs(1), stdout.write(&frame)).await {
            Ok(Ok(())) => Ok(code),
            _ => Ok(if matches!(summary.stop, product_reader::follow::Stop::Cancelled) { 130 } else { 1 }),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    #[derive(clap::Parser)]
    struct ProjectCli {
        #[command(subcommand)]
        action: ProjectAction,
    }
    fn request(action: ProjectAction) -> (&'static str, String, Box<RawValue>) {
        let output = OutputOptions {
            format: envelope::OutputFormat::Json,
            presentation: Default::default(),
            invocation_id: envelope::CliInvocationId::new(),
            artifact_directory: None,
        };
        action
            .request(None, Path::new("/"), &Environment::new(), &output)
            .unwrap()
            .unwrap()
    }

    #[test]
    fn project_online_read_requires_one_exact_state_and_preserves_owner_cursor() {
        assert!(
            ProjectCli::try_parse_from(["project", "show", "--reader", "capture", "p"]).is_err()
        );
        assert!(
            ProjectCli::try_parse_from([
                "project", "show", "--reader", "capture", "--state", "/one", "--state", "/two",
                "p",
            ])
            .is_err()
        );
        let original = r#"{"issuing_store":"a","kind":"drafts","owner":"b","after":"c"}"#;
        let parsed = ProjectCli::try_parse_from([
            "project", "metadata", "--reader", "capture", "--state", "/one", "--cursor", original,
            "drafts", "b",
        ])
        .unwrap();
        let (command, reader, request) = request(parsed.action);
        assert_eq!(command, "project metadata");
        assert_eq!(reader, "capture");
        assert!(request.get().contains(original));
        let value: serde_json::Value = serde_json::from_str(request.get()).unwrap();
        assert_eq!(value["canonical"]["state"], "/one");
        assert!(value["canonical"]["database"].is_null());
        assert_eq!(value["query"]["request"]["kind"], "drafts");
        assert_eq!(value["query"]["request"]["owner"], "b");
    }

    #[test]
    fn project_study_workbench_uses_typed_flags_and_preserves_the_owner_cursor() {
        let arguments = [
            "project",
            "study-workbench",
            "--reader",
            "capture",
            "--state",
            "/one",
        ];
        let parsed = ProjectCli::try_parse_from(arguments).unwrap();
        let (command, reader, request) = request(parsed.action);
        assert_eq!(command, "project study-workbench");
        assert_eq!(reader, "capture");
        let value: serde_json::Value = serde_json::from_str(request.get()).unwrap();
        assert_eq!(value["query"]["kind"], "study_workbench");
        assert_eq!(value["query"]["request"]["maximum"], 64);
        assert!(value["query"]["request"]["filter"]["project"].is_null());
        assert!(value["query"]["request"]["cursor"].is_null());
        for invalid in ["0", "65"] {
            assert!(
                ProjectCli::try_parse_from(arguments.into_iter().chain(["--maximum", invalid,]))
                    .is_err()
            );
        }
        let cursor = r#"{"issuing_store":"store-a", "filter":{"project":"p","outcome":"succeeded"},"after":"s"}"#;
        let parsed = ProjectCli::try_parse_from(arguments.into_iter().chain([
            "--database",
            "existing",
            "--project",
            "p",
            "--outcome",
            "succeeded",
            "--maximum",
            "1",
            "--cursor",
            cursor,
        ]))
        .unwrap();
        let (_, _, request) = self::request(parsed.action);
        assert!(request.get().contains(cursor));
        let value: serde_json::Value = serde_json::from_str(request.get()).unwrap();
        assert_eq!(value["canonical"]["database"], "existing");
        let selected = &value["query"]["request"];
        assert_eq!(selected["maximum"], 1);
        assert_eq!(selected["filter"]["project"], "p");
        assert_eq!(selected["filter"]["outcome"], "succeeded");
    }

    #[test]
    fn project_pack_request_keeps_native_input_and_defers_the_default_to_product() {
        use std::os::unix::ffi::OsStringExt as _;
        let input = PathBuf::from(std::ffi::OsString::from_vec(b"/pack-\xff".to_vec()));
        let raw = project_pack_request(input.clone(), None).unwrap();
        let request: serde_json::Value = serde_json::from_str(raw.get()).unwrap();
        assert_eq!(request["operation"], "inspect_project_pack");
        let path: NativePath = serde_json::from_value(request["input"].clone()).unwrap();
        assert_eq!(path.to_path_buf(), input);
        assert!(request.get("maximum_document_bytes").is_none());
        let raw = project_pack_request(input, NonZeroUsize::new(64)).unwrap();
        let request: serde_json::Value = serde_json::from_str(raw.get()).unwrap();
        assert_eq!(request["maximum_document_bytes"], 64);
    }

    #[test]
    fn request_transport_retains_lexical_numbers_and_duplicate_keys() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("request.json");
        let original = r#"{"operation":"custom","x":1.00000000000000000000001,"x":2}"#;
        std::fs::write(&path, original).unwrap();
        assert_eq!(request_file(&path).unwrap().get(), original);
        std::fs::write(&path, b"invalid").unwrap();
        assert!(request_file(&path).is_err());
        std::fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
        assert!(request_file(&path).is_err());
    }
}
