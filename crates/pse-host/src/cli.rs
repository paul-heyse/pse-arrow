// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One command grammar and native dispatch for the host entry points.
use crate::envelope::{
    self, CliInvocationId, CommandOutcome, Completeness, Envelope, ErrorRecord, OutputFormat,
    OutputOptions, Presentation, RedactionPolicy,
};
use crate::generation::{EntryGuard, HostTool, NewWorkDecision};
use crate::{HostError, environment::Environment};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{
    ffi::OsString,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};
mod affected;
mod api;
pub mod bundle;
mod changes;
mod code;
mod compiler;
mod control;
mod dependency;
mod docs;
mod probe;
mod product;
mod selection;
mod testing;
mod tools;
mod worktree;

/// Selects the fixed executable protocol, never inferred from an executable filename.
#[derive(Clone, Copy, Debug)]
pub enum BinKind {
    /// Agent-facing command name.
    Pse,
    /// Native host and hook command name.
    Host,
    /// External Cargo command and raw adapter protocol.
    Correctness,
}

#[derive(Debug, Parser)]
#[command(
    name = "pse",
    about = "Native host capabilities; explicit preparation: scripts/bootstrap-host",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    #[arg(short = 'C', global = true, id = "checkout")]
    pub(crate) root: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        value_enum,
        env = "PSE_FORMAT",
        hide_env_values = true,
        default_value = "auto"
    )]
    /// Human output on terminals, JSON on pipes, or compact NDJSON observations.
    format: OutputFormat,
    #[arg(long, global = true, value_delimiter = ',')]
    /// Select comma-separated payload fields while retaining identity, state and coverage.
    fields: Option<Vec<String>>,
    #[arg(long, global = true)]
    /// Limit whole payload records; producer pages retain their omission and continuation facts.
    budget: Option<usize>,
    #[arg(long)]
    version: bool,
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Debug, Subcommand)]
enum Command {
    /// Read retained native compiler diagnostics without rebuilding.
    Compiler {
        #[command(subcommand)]
        action: compiler::Action,
    },
    /// Create prepared sibling worktrees or inspect native checkout users.
    Worktree {
        #[command(subcommand)]
        action: worktree::Action,
    },
    /// Attempt every independent Just recipe under one native foreground owner.
    RunAll(bundle::Options),
    #[command(name = "__bundle", hide = true)]
    Bundle(bundle::Entry),
    /// Explicitly build/list the actual selected native test population.
    Tests(testing::Tests),
    /// Build, execute and reconcile the actual selected native tests.
    Test(testing::Tests),
    /// Check the selected source through the correctness and compiler observation owners.
    Check(testing::Check),
    /// Prepare or inspect inputs through the existing native fixture owner.
    Fixture {
        #[command(subcommand)]
        action: FixtureAction,
    },
    #[command(name = "__fixture-prepare", hide = true)]
    FixturePrepare,
    #[command(name = "__workflow", hide = true)]
    Workflow(ManagedEntryArgs),
    #[command(name = "__semantic-service", hide = true)]
    SemanticService {
        #[arg(long, value_parser = parse_invocation_id)]
        invocation: CliInvocationId,
        #[arg(long)]
        configuration: String,
    },
    /// Explicit captured development-tool preparation, discovery and managed native forwarding.
    Tools {
        #[command(subcommand)]
        action: tools::Action,
    },
    /// Preview conservative changed-source reach through Gitoxide and captured Cargo declarations.
    Affected(affected::Selection),
    /// Observe one original managed invocation; an observation timeout leaves it running.
    Wait(control::Selection),
    /// Stop one authenticated original invocation and inspect its original settlement evidence.
    Cancel(control::Selection),
    /// Inspect scientific evidence through an explicitly prepared native product reader.
    Observe {
        #[command(subcommand)]
        action: product::Action,
    },
    #[command(name = "__supervise", hide = true)]
    Supervise(ManagedEntryArgs),
    #[command(name = "__finalize", hide = true)]
    Finalize(ManagedEntryArgs),
    /// Resolve test/check intent without building binaries or preparing runtime inputs.
    Explain {
        /// Stop at normalized declarations before querying the actual compiler-unit graph.
        #[arg(long)]
        intent_only: bool,
        /// Include every reached compiler unit, rather than the default selected-root view.
        #[arg(long)]
        full: bool,
        #[command(subcommand)]
        action: ExplainAction,
    },
    /// Inspect declared generated-output ownership and existing source contributors.
    Owner {
        path: PathBuf,
        /// Existing captured Cargo metadata enables exact source-package ownership.
        #[arg(long)]
        metadata: Option<PathBuf>,
        /// Explicit prepared native document interpreter; no read implicitly builds it.
        #[arg(long)]
        xtask_selection: Option<String>,
    },
    /// Inspect exact locked dependency identities, features and direct workspace consumers.
    #[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
    Dep {
        #[arg(required = true)]
        package: Option<String>,
        #[command(subcommand)]
        action: Option<dependency::Action>,
        /// Read this explicit Cargo metadata snapshot instead of collecting locked offline metadata.
        #[arg(long)]
        metadata: Option<PathBuf>,
        /// Show one shortest package inclusion path per reachable workspace member.
        #[arg(long)]
        why: bool,
        /// Find exact authored feature-expansion members without activating them.
        #[arg(long)]
        feature_member: Option<String>,
    },
    /// Inspect changed Git layers, declared source owners and conservative test reach.
    Changes(changes::Selection),
    /// Forward documentation operations to the explicitly prepared native owner.
    Docs(docs::Args),
    /// Read exact-context Rust APIs, or explicitly prepare their documentation artifacts.
    Api(api::Arguments),
    /// Query Rust syntax and inspect source-bound change proposals.
    Code {
        #[command(subcommand)]
        action: code::Action,
    },
    /// Create and run an explicitly isolated authored Rust experiment.
    Probe {
        #[command(subcommand)]
        action: probe::Action,
    },
    /// Read an explicit retained file without interpreting its content as a protocol.
    Artifact {
        #[command(subcommand)]
        action: ArtifactAction,
    },
    /// Read one Rust file's written declarations or a Markdown document's headings.
    Outline {
        #[arg(required=true,num_args=1..)]
        files: Vec<PathBuf>,
        /// Continue an exact-source declaration page; select one file.
        #[arg(long)]
        cursor: Option<String>,
        #[command(flatten)]
        parsing: SourceParsing,
    },
    /// Read exact original source at FILE::ITEM[::ITEM], preserving ambiguity.
    Show {
        source: OsString,
        /// Select literal path components; also supports filenames/headings containing ::.
        #[arg(long)]
        item: Vec<String>,
        /// Disambiguate duplicate paths using an original UTF-8 byte position.
        #[arg(long)]
        at_byte: Option<usize>,
        /// One-based line and Unicode scalar column of an enclosing declaration.
        #[arg(long, requires = "at_column", conflicts_with_all = ["at_byte", "range_start"])]
        at_line: Option<usize>,
        #[arg(long, requires = "at_line")]
        at_column: Option<usize>,
        /// Exact half-open UTF-8 byte range.
        #[arg(long, requires = "range_end", conflicts_with_all = ["at_byte", "at_line", "item"])]
        range_start: Option<usize>,
        #[arg(long, requires = "range_start")]
        range_end: Option<usize>,
        #[command(flatten)]
        parsing: SourceParsing,
    },
    /// Discover actual command declarations or lazily query one argument's candidates.
    Commands(CatalogArgs),
    /// Inspect one command's native help and public parser metadata.
    Help(CatalogArgs),
    /// Read independent native checkout, invocation and prepared-tool observations.
    Status {
        #[arg(long, conflicts_with = "line")]
        brief: bool,
        #[arg(long)]
        line: bool,
        #[arg(long,conflicts_with_all=["brief","line"])]
        all: bool,
    },
    #[command(name = "__host-generation-bootstrap", hide = true)]
    HostGenerationBootstrap {
        #[arg(long)]
        from: Option<PathBuf>,
    },
    /// Print the serialization schema of a command's actual detail view.
    Schema {
        #[arg(required=true,num_args=1..)]
        verb: Vec<String>,
    },
    #[command(
        hide = true,
        before_help = "pse-env — native command and environment boundary"
    )]
    Boundary(BoundaryArgs),
    #[command(hide = true)]
    AgentHook {
        #[command(subcommand)]
        action: HookAction,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    CargoTools {
        names: Vec<String>,
    },
    DiagnosticTools {
        #[command(subcommand)]
        action: ToolAction,
    },
    TelemetryService {
        #[command(subcommand)]
        action: TelemetryAction,
    },
    Native {
        #[command(subcommand)]
        action: NativeAction,
    },
    Server {
        #[command(subcommand)]
        action: ServerAction,
    },
    StudyModule {
        #[command(subcommand)]
        action: StudyAction,
    },
    Env(EnvironmentArgs),
    Activity {
        unit: Option<String>,
    },
    /// Inspect newest invocation metadata without reading historical result payloads.
    History {
        #[arg(long, conflicts_with = "all_sessions")]
        session: Option<String>,
        /// Read checkout-wide history instead of selecting the current agent session.
        #[arg(long)]
        all_sessions: bool,
        #[arg(long, default_value_t = 20, value_parser=clap::value_parser!(u64).range(1..))]
        limit: u64,
    },
    /// Read the original immutable result without starting or rebuilding work.
    #[command(group(clap::ArgGroup::new("result_page").args(["tests", "failures"]).multiple(false)))]
    Result {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        /// Read whole selected-test records from this original workflow's retained inventory.
        #[arg(long)]
        tests: bool,
        /// Read original failed cases and owner diagnostics without running work.
        #[arg(long)]
        failures: bool,
        #[arg(long, requires = "failures", value_parser = failure_cursor)]
        failure_cursor: Option<crate::test_inventory::FailureCursor>,
        /// Continue a test page using its original JSON cursor, refusing changed inventory bytes.
        #[arg(long, requires = "tests", value_parser = test_cursor)]
        cursor: Option<crate::test_inventory::Cursor>,
        #[arg(long, requires = "result_page", default_value = "20")]
        limit: std::num::NonZeroUsize,
        /// Bound serialized whole test rows; source scanning remains explicit.
        #[arg(long, requires = "result_page", default_value = "16384")]
        page_bytes: std::num::NonZeroUsize,
    },
    /// Read a bounded original raw stream with exact byte offsets.
    Log {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        stderr: bool,
        #[arg(long, default_value_t = 0, conflicts_with = "tail")]
        offset: u64,
        #[arg(long, default_value = "4096")]
        length: std::num::NonZeroU64,
        #[arg(long)]
        tail: bool,
    },
    Run {
        /// Keep the owned invocation alive after this launching process returns.
        #[arg(long)]
        detach: bool,
        /// Explicit session association; never used as a filesystem path.
        #[arg(long)]
        session: Option<String>,
        /// Explicit finite workload class; ordinary work keeps the configured host policy.
        #[arg(long, value_parser = resource_class)]
        resource_class: Option<String>,
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<OsString>,
    },
}
impl Command {
    fn raw_protocol(&self) -> bool {
        match self {
            Self::Tools { action } => action.raw_protocol(),
            Self::Docs(_) => true,
            Self::Boundary(options) => options.print || !options.explain,
            Self::AgentHook { .. }
            | Self::Workflow(_)
            | Self::SemanticService { .. }
            | Self::Bundle(_)
            | Self::Supervise(_)
            | Self::Finalize(_)
            | Self::HostGenerationBootstrap { .. }
            | Self::CargoTools { .. }
            | Self::DiagnosticTools { .. }
            | Self::Run { .. } => true,
            Self::Env(options) => options.print || options.activation_data,
            _ => false,
        }
    }

    // Effect/ownership intent belongs to the actual command declaration, separately
    // from its presentation protocol. Control and internal entries never auto-rebuild.
    fn generation_intent(&self) -> GenerationIntent {
        match self {
            Self::Tools { action } => action.intent(),
            Self::Compiler { action } => action.intent(),
            Self::Worktree { action } => action.intent(),
            Self::Wait(_) | Self::Cancel(_) => GenerationIntent::ExistingControl,
            Self::Observe { action } => action.intent(),
            Self::Probe { action } => action.intent(),
            Self::Api(args) => args.intent(),
            Self::Docs(args) => args.intent(),
            Self::Code { action } => action.intent(),
            Self::HostGenerationBootstrap { .. }
            | Self::Supervise(_)
            | Self::Finalize(_)
            | Self::Workflow(_)
            | Self::SemanticService { .. }
            | Self::Bundle(_)
            | Self::FixturePrepare => GenerationIntent::Internal,
            Self::Boundary(options)
                if !options.print && !options.explain && !options.command.is_empty() =>
            {
                GenerationIntent::NewWork
            }
            Self::Boundary(_)
            | Self::Affected(_)
            | Self::Explain { .. }
            | Self::Owner { .. }
            | Self::Dep { .. }
            | Self::Changes(_)
            | Self::Outline { .. }
            | Self::Show { .. }
            | Self::Artifact { .. }
            | Self::Commands(_)
            | Self::Help(_)
            | Self::Status { .. }
            | Self::Fixture {
                action: FixtureAction::Show { .. },
            }
            | Self::Schema { .. }
            | Self::AgentHook { .. }
            | Self::Doctor { .. }
            | Self::CargoTools { .. }
            | Self::Env(_)
            | Self::Activity { .. }
            | Self::History { .. }
            | Self::Result { .. }
            | Self::Log { .. }
            | Self::DiagnosticTools {
                action: ToolAction::Verify { .. },
            }
            | Self::Server {
                action: ServerAction::Select,
            }
            | Self::StudyModule {
                action: StudyAction::Select { .. },
            }
            | Self::TelemetryService {
                action: TelemetryAction::Inspect(_),
            } => GenerationIntent::Passive,
            Self::TelemetryService {
                action: TelemetryAction::Stop(_) | TelemetryAction::RecoverConfiguration(_),
            } => GenerationIntent::ExistingControl,
            Self::Run { .. }
            | Self::RunAll(_)
            | Self::Tests(_)
            | Self::Test(_)
            | Self::Check(_)
            | Self::Fixture {
                action: FixtureAction::Prepare { .. },
            }
            | Self::Native { .. }
            | Self::DiagnosticTools {
                action: ToolAction::Install { .. },
            }
            | Self::Server {
                action: ServerAction::Prepare { .. },
            }
            | Self::StudyModule {
                action: StudyAction::Prepare { .. },
            }
            | Self::TelemetryService { .. } => GenerationIntent::NewWork,
        }
    }
}
#[derive(Debug, Args)]
struct ManagedEntryArgs {
    #[arg(long)]
    checkout_bytes: String,
    #[arg(long, value_parser = parse_invocation_id)]
    invocation: CliInvocationId,
}
fn parse_invocation_id(value: &str) -> Result<CliInvocationId, String> {
    CliInvocationId::try_from(value.to_owned()).map_err(|error| error.to_string())
}
#[derive(Debug, Subcommand)]
enum ExplainAction {
    Test(selection::TestArgs),
    Check(selection::SelectionArgs),
}
#[derive(Debug, Args)]
struct CatalogArgs {
    /// Select a command path below pse; absence selects the root.
    path: Vec<String>,
    #[arg(long)]
    json: bool,
    /// Publish Markdown directly from the owning parser, without candidate acquisition.
    #[arg(long,conflicts_with_all=["json","argument","metadata","prefix","offset","limit"])]
    markdown: bool,
    /// Query candidates for this argument ID or long option, rather than command descriptions.
    #[arg(long)]
    argument: Option<String>,
    #[arg(long, default_value = "")]
    prefix: String,
    #[arg(long, default_value_t = 0)]
    offset: usize,
    #[arg(long,value_parser=clap::value_parser!(u64).range(1..))]
    limit: Option<u64>,
    /// Explicit retained Cargo metadata; only a selected metadata provider reads it.
    #[arg(long)]
    metadata: Option<PathBuf>,
}
#[derive(Debug, Args)]
struct SourceParsing {
    /// Read the raw staging-index blob rather than working bytes.
    #[arg(long, conflicts_with = "revision")]
    index: bool,
    /// Read a committed blob; resolve the revision before returning its identity.
    #[arg(long)]
    revision: Option<String>,
    /// Override the .rs/.md/.markdown filename inference.
    #[arg(long, value_enum)]
    language: Option<crate::source_command::Language>,
    /// Explicit syntax policy; no Cargo edition or cfg discovery is performed.
    #[arg(long, value_enum, default_value = "2024")]
    edition: crate::source_read::RustEdition,
}
#[derive(Debug, Subcommand)]
enum ArtifactAction {
    /// Read a selected byte range with explicit continuation; default maximum 4096 bytes.
    Read {
        file: PathBuf,
        #[arg(long, default_value_t = 0)]
        offset: u64,
        /// Select a positive byte length instead of the default prefix.
        #[arg(long, conflicts_with = "all")]
        length: Option<std::num::NonZeroU64>,
        /// Explicitly select every remaining byte from the offset.
        #[arg(long)]
        all: bool,
        /// Require the opaque native ETag observed previously; this is not a content hash.
        #[arg(long)]
        if_match: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationIntent {
    Passive,
    NewWork,
    ExistingControl,
    Internal,
}

#[derive(Debug, Subcommand)]
pub(crate) enum HookAction {
    Guard,
    SessionEnv,
    /// Suggest native tooling routes without changing permission or execution.
    ToolingAdvice,
}
#[derive(Debug, Subcommand)]
enum FixtureAction {
    Prepare {
        #[arg(value_parser = ["study-module"])]
        fixture: String,
    },
    Show {
        #[arg(value_parser = ["study-module"])]
        fixture: String,
    },
}
#[derive(Debug, Subcommand)]
enum ToolAction {
    Install { tool: String },
    Verify { tool: String },
}
#[derive(Debug, Subcommand)]
enum NativeAction {
    Prepare {
        #[arg(long, required = true, value_parser = capabilities)]
        capabilities: String,
    },
}
fn capabilities(value: &str) -> Result<String, String> {
    crate::native::validate_capabilities(value).map_err(|error| error.to_string())?;
    Ok(value.into())
}
#[derive(Debug, Subcommand)]
pub(crate) enum ServerAction {
    Prepare {
        #[arg(long)]
        from_release: Option<PathBuf>,
    },
    Select,
}
#[derive(Debug, Args)]
pub(crate) struct StudyVariant {
    #[arg(long)]
    pub(crate) qualification: bool,
    #[arg(long, requires = "qualification")]
    pub(crate) long_hold: bool,
}
#[derive(Debug, Subcommand)]
pub(crate) enum StudyAction {
    Prepare {
        #[arg(long)]
        tool_root: Option<PathBuf>,
        #[command(flatten)]
        variant: StudyVariant,
    },
    Select {
        #[arg(long)]
        receipt: PathBuf,
        #[command(flatten)]
        variant: StudyVariant,
    },
}
#[derive(Debug, Subcommand)]
pub(crate) enum TelemetryAction {
    Install,
    Setup(TelemetrySetup),
    Inspect(TelemetryState),
    Start(TelemetryState),
    Stop(TelemetryState),
    Reconfigure(TelemetryState),
    RecoverConfiguration(TelemetryState),
}
#[derive(Debug, Args)]
pub(crate) struct TelemetryState {
    #[arg(long)]
    pub(crate) state: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub(crate) struct TelemetrySetup {
    #[arg(long)]
    pub(crate) state: Option<PathBuf>,
    #[arg(long = "port", value_parser = named_port)]
    pub(crate) ports: Vec<(String, u16)>,
    #[arg(long, default_value = "default", overrides_with = "organization")]
    pub(crate) organization: String,
    #[arg(long = "stream", value_parser = named_value)]
    pub(crate) streams: Vec<(String, String)>,
    #[arg(long, default_value_t = 7, overrides_with = "retention_days")]
    pub(crate) retention_days: u32,
}
fn named_value(value: &str) -> Result<(String, String), String> {
    value
        .split_once('=')
        .map(|(name, value)| (name.into(), value.into()))
        .ok_or_else(|| "expected NAME=VALUE".into())
}
fn named_port(value: &str) -> Result<(String, u16), String> {
    let (name, port) = named_value(value)?;
    Ok((
        name,
        port.parse()
            .map_err(|_| "port must be an integer from 1 through 65535")?,
    ))
}
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub(crate) enum CacheArg {
    #[default]
    Auto,
    On,
    Off,
}
impl From<CacheArg> for crate::environment::CacheMode {
    fn from(value: CacheArg) -> Self {
        match value {
            CacheArg::Auto => Self::Auto,
            CacheArg::On => Self::On,
            CacheArg::Off => Self::Off,
        }
    }
}
#[derive(Debug, Args)]
#[group(skip)]
#[command(args_override_self = true)]
#[command(group(clap::ArgGroup::new("operation").required(true).args(["print", "explain", "activation_data"])))]
pub(crate) struct EnvironmentArgs {
    #[arg(long)]
    pub(crate) print: bool,
    #[arg(long)]
    pub(crate) explain: bool,
    #[arg(long, requires = "explain")]
    pub(crate) json: bool,
    #[arg(long, hide = true)]
    pub(crate) activation_data: bool,
    #[arg(long)]
    pub(crate) docs: bool,
    #[arg(long, requires = "activation_data")]
    pub(crate) root: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "auto")]
    pub(crate) cache: CacheArg,
}
#[derive(Debug, Args)]
#[command(args_override_self = true)]
pub(crate) struct BoundaryArgs {
    #[arg(long)]
    pub(crate) print: bool,
    #[arg(long)]
    pub(crate) explain: bool,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) docs: bool,
    #[arg(long)]
    pub(crate) no_scope: bool,
    #[arg(long, value_parser = resource_class)]
    pub(crate) resource_class: Option<String>,
    #[arg(long, value_parser = resource_class)]
    pub(crate) recipe_default_class: Option<String>,
    #[arg(long, value_enum, default_value = "auto")]
    pub(crate) cache: CacheArg,
    #[arg(long, num_args = 0..=1, require_equals = true)]
    pub(crate) native: Option<Option<String>>,
    #[arg(trailing_var_arg = true)]
    pub(crate) command: Vec<OsString>,
}
fn resource_class(value: &str) -> Result<String, String> {
    crate::admission::validate_class(value).map_err(|error| error.to_string())?;
    Ok(value.into())
}
fn failure_cursor(value: &str) -> Result<crate::test_inventory::FailureCursor, serde_json::Error> {
    serde_json::from_str(value)
}
fn test_cursor(value: &str) -> Result<crate::test_inventory::Cursor, serde_json::Error> {
    serde_json::from_str(value)
}

#[derive(Parser)]
#[command(
    name = "cargo pse-correctness",
    about = "Source-build correctness; Cargo and native nextest arguments remain literal"
)]
struct CorrectnessCli {
    #[command(subcommand)]
    command: CorrectnessCommand,
}
#[derive(Subcommand)]
enum CorrectnessCommand {
    Check(CargoTail),
    Build(CargoTail),
    Test(CargoTail),
    Bench(CargoTail),
    Run(CargoTail),
    Clippy(CargoTail),
    Rustc(CargoTail),
    Rustdoc(CargoTail),
    Nextest {
        #[command(subcommand)]
        action: NextestCommand,
    },
}
#[derive(Args)]
struct CargoTail {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    arguments: Vec<OsString>,
}
#[derive(Subcommand)]
enum NextestCommand {
    Run(CargoTail),
    List(CargoTail),
}
pub(crate) fn validate_correctness(
    arguments: &[OsString],
) -> Result<bool, crate::correctness::CorrectnessError> {
    let mut argv = vec![OsString::from("cargo pse-correctness")];
    argv.extend_from_slice(arguments);
    match CorrectnessCli::try_parse_from(argv) {
        Ok(_) => Ok(true),
        Err(error) if !error.use_stderr() => {
            error.print()?;
            Ok(false)
        }
        Err(error) => Err(crate::correctness::CorrectnessError::Invalid(
            error.to_string(),
        )),
    }
}

fn runtime() -> Result<tokio::runtime::Runtime, HostError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}
/// Find an explicitly selected checkout, its immutable executable owner, or current directory.
pub fn checkout_root(selected: Option<&Path>) -> Result<PathBuf, HostError> {
    fn find(mut path: PathBuf) -> Option<PathBuf> {
        loop {
            if path.join(".config/diagnostic-tools.toml").is_file() {
                return Some(path);
            }
            if !path.pop() {
                return None;
            }
        }
    }
    if let Some(path) = selected {
        return find(path.to_path_buf())
            .ok_or_else(|| HostError::Ownership("-C must select a pse-arrow checkout".into()));
    }
    if let Some(executable) = std::env::current_exe().ok().filter(|path| {
        path.parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .is_some_and(|name| name == "host-tools")
    }) && let Some(root) = find(executable)
    {
        return Ok(root);
    }
    find(std::env::current_dir()?)
        .ok_or_else(|| HostError::Ownership("run inside a pse-arrow checkout".into()))
}
#[derive(serde::Serialize, schemars::JsonSchema)]
struct ServerSelection {
    executable: PathBuf,
    recipe: String,
}
pub(crate) fn response_schema(command: &str, view: &Presentation) -> io::Result<schemars::Schema> {
    match command {
        "run" | "run-all" | "tools run" | "test" | "tests" | "check" => {
            envelope::schema::<ManagedReply>(command, view)
        }
        "tools prepare" => envelope::schema::<tools::PreparedTool>(command, view),
        "tools catalog" => envelope::schema::<
            crate::catalog::description::Page<crate::catalog::description::Description>,
        >(command, view),
        "wait" | "cancel" => envelope::schema::<control::ControlView>(command, view),
        "observe prepare" => envelope::schema::<product::PreparedReader>(command, view),
        "observe query" | "observe discover" | "observe investigate" => {
            envelope::schema::<product::ProductView<'static>>(command, view)
        }
        "observe follow" => envelope::schema::<product::FollowView<'static>>(command, view),
        "artifact read" => envelope::schema::<crate::artifacts::ArtifactRead>(command, view),
        "outline" => envelope::schema::<crate::source_command::FileOutline>(command, view),
        "changes" => envelope::schema::<changes::Report>(command, view),
        "docs" => envelope::schema::<ManagedReply>(command, view),
        "api" => envelope::schema::<crate::api::query::Report>(command, view),
        "code query" => envelope::schema::<crate::source_query::QueryReport>(command, view),
        "code preview" => envelope::schema::<crate::source_query::PreviewReport>(command, view),
        "code origin" => envelope::schema::<crate::source_command::GeneratedOrigin>(command, view),
        "compiler diagnostics" => {
            envelope::schema::<crate::compiler_investigation::DiagnosticPage>(command, view)
        }
        "compiler diagnostic" => {
            envelope::schema::<crate::compiler_investigation::DetailPage<'static>>(command, view)
        }
        "compiler generate" | "compiler probe" => envelope::schema::<ManagedReply>(command, view),
        "compiler artifacts" => {
            envelope::schema::<crate::compiler_investigation::artifact::Observation>(command, view)
        }
        "compiler file" => {
            envelope::schema::<crate::compiler_investigation::artifact::FileRead>(command, view)
        }
        "worktree new" => envelope::schema::<crate::worktree::NewReport>(command, view),
        "worktree list" => envelope::schema::<crate::worktree::ListReport>(command, view),
        "api prepare" | "api compare" | "api check-docs" | "probe run" => {
            envelope::schema::<ManagedReply>(command, view)
        }
        "code semantic"
        | "code explain"
        | "code symbols"
        | "code refs"
        | "code calls"
        | "code implementations"
        | "code macro"
        | "code tests"
        | "code candidates"
        | "code candidate"
        | "code predicate"
        | "code hints"
        | "code hint"
        | "code edit rename"
        | "code edit assists"
        | "code edit assist"
        | "code edit ssr" => envelope::schema::<crate::semantic::Answer>(command, view),
        "code session prepare" => envelope::schema::<crate::semantic::SessionStatus>(command, view),
        "code session status" => {
            envelope::schema::<crate::semantic::SessionOverview>(command, view)
        }
        "code session stop" => envelope::schema::<control::ControlView>(command, view),
        "code semantic refusal" => envelope::schema::<crate::semantic::Failure>(command, view),
        "code edit file-rename" => envelope::schema::<crate::semantic::Answer>(command, view),
        "probe new" => envelope::schema::<crate::probe::Probe>(command, view),
        "probe rm" => envelope::schema::<PathBuf>(command, view),
        "outline batch" => envelope::schema::<crate::source_command::batch::Report>(command, view),
        "show" => envelope::schema::<crate::source_command::FileSelection>(command, view),
        "commands" | "help" => envelope::schema::<
            crate::catalog::description::Page<crate::catalog::description::Description>,
        >(command, view),
        "commands candidates" | "help candidates" => envelope::schema::<
            crate::catalog::description::Page<crate::catalog::description::Candidate>,
        >(command, view),
        "status" => envelope::schema::<crate::status::Report>(command, view),
        "explain test" | "explain check" => {
            envelope::schema::<crate::test_workflow::BriefExplanation<'static>>(command, view)
        }
        "explain test full" | "explain check full" => {
            envelope::schema::<crate::test_workflow::Explanation<'static>>(command, view)
        }
        "owner" => envelope::schema::<OwnerView>(command, view),
        "affected" => envelope::schema::<crate::affected::Preview>(command, view),
        "dep" => envelope::schema::<crate::dependency::Report>(command, view),
        "dep units" => envelope::schema::<dependency::Units>(command, view),
        "history" => envelope::schema::<crate::invocation_view::HistoryPage>(command, view),
        "result" => envelope::schema::<crate::invocation_view::ResultView>(command, view),
        "result tests" => envelope::schema::<crate::test_inventory::TestPage>(command, view),
        "result failures" => envelope::schema::<crate::test_inventory::FailurePage>(command, view),
        "log" => envelope::schema::<crate::invocation_view::LogView>(command, view),
        "status brief" => envelope::schema::<crate::status::Summary>(command, view),
        "doctor" => envelope::schema::<crate::doctor::Report>(command, view),
        "env" | "env explain" => {
            envelope::schema::<crate::environment::EnvironmentExplanation>("env explain", view)
        }
        "boundary explain native" => {
            envelope::schema::<crate::boundary::NativeExplanation>(command, view)
        }
        "activity" => envelope::schema::<crate::activity::ActivityObservation>(command, view),
        "fixture prepare" => envelope::schema::<ManagedReply>(command, view),
        "fixture show" | "study-module" | "study-module prepare" | "study-module select" => {
            envelope::schema::<crate::study_module::StudyModuleReceipt>(command, view)
        }
        "telemetry-service"
        | "telemetry-service inspect"
        | "telemetry-service setup"
        | "telemetry-service install"
        | "telemetry-service start"
        | "telemetry-service stop"
        | "telemetry-service reconfigure"
        | "telemetry-service recover-configuration" => {
            envelope::schema::<serde_json::Value>(command, view)
        }
        "native prepare" => {
            envelope::schema::<std::collections::BTreeMap<String, PathBuf>>(command, view)
        }
        "server select" | "server prepare" => envelope::schema::<ServerSelection>(command, view),
        "error" => envelope::schema::<()>(command, &Presentation::default()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("no response schema for {command}; choose a supported managed command"),
        )),
    }
}
fn dispatch(cli: Cli, presentation: &OutputOptions, entry: &EntryGuard) -> Result<i32, HostError> {
    let selected = cli.root.as_deref();
    let Some(command) = cli.command else {
        return Ok(0);
    };
    match command {
        Command::Fixture {
            action: FixtureAction::Prepare { fixture },
        } => {
            let root = checkout_root(selected)?;
            let control = entry.prepared().ok_or_else(|| {
                HostError::Ownership(
                    "fixture preparation requires explicitly prepared native tools".into(),
                )
            })?;
            let command = vec![
                control.executable(&root, HostTool::Host)?.into_os_string(),
                "-C".into(),
                root.as_os_str().to_owned(),
                "--format".into(),
                "json".into(),
                "__fixture-prepare".into(),
            ];
            debug_assert_eq!(fixture, "study-module");
            launch_managed(
                ManagedCommand {
                    parent: None,
                    cwd: root.clone(),
                    root,
                    environment: std::env::vars_os().collect(),
                    command,
                    detach: false,
                    session: None,
                    resource_class: None,
                    verb: "fixture prepare",
                    tool: None,
                    workflow: None,
                },
                entry,
                presentation,
            )
        }
        Command::Fixture {
            action: FixtureAction::Show { fixture: _fixture },
        } => {
            let root = checkout_root(selected)?;
            let receipt = crate::fixtures::selected_module(&root, &std::env::vars_os().collect())
                .map_err(|error| {
                HostError::Workflow(Box::new(crate::test_workflow::WorkflowError::Fixture(
                    error,
                )))
            })?;
            Ok(presentation.emit(Envelope::response(
                "fixture show",
                "native fixture preparation input",
                receipt,
            ))?)
        }
        Command::RunAll(options) => bundle::start(options, selected, entry, presentation),
        Command::Bundle(args) => bundle::run(args, selected, entry),
        Command::FixturePrepare => {
            let receipt = runtime()?
                .block_on(crate::fixtures::prepare_module(
                    &checkout_root(selected)?,
                    &std::env::vars_os().collect(),
                ))
                .map_err(|error| {
                    HostError::Workflow(Box::new(crate::test_workflow::WorkflowError::Fixture(
                        error,
                    )))
                })?;
            Ok(presentation.emit(Envelope::response(
                "fixture prepare input",
                "native owner prepared the fixture module",
                receipt,
            ))?)
        }
        Command::Workflow(args) => runtime()?.block_on(crate::managed::workflow::run(
            crate::managed::decode_checkout(&args.checkout_bytes)?,
            args.invocation,
        )),
        Command::SemanticService {
            invocation,
            configuration,
        } => {
            let configuration = serde_json::from_str(&configuration).map_err(|error| {
                HostError::Ownership(format!("semantic configuration: {error}"))
            })?;
            runtime()?
                .block_on(crate::semantic::serve(
                    checkout_root(selected)?,
                    invocation,
                    configuration,
                ))
                .map_err(|error| HostError::Ownership(error.to_string()))?;
            Ok(0)
        }
        Command::Tests(args) => testing::start(
            crate::test_workflow::execute::Action::List,
            args.selection.request()?,
            args.selection.profile,
            args.run,
            selected,
            entry,
            presentation,
        ),
        Command::Test(args) => testing::start(
            crate::test_workflow::execute::Action::Test,
            args.selection.request()?,
            args.selection.profile,
            args.run,
            selected,
            entry,
            presentation,
        ),
        Command::Check(args) => testing::start(
            crate::test_workflow::execute::Action::Check,
            args.selection.request(crate::selection::Operation::Check)?,
            "default".into(),
            args.run,
            selected,
            entry,
            presentation,
        ),
        Command::Tools { action } => tools::dispatch(action, selected, presentation, entry),
        Command::Affected(args) => affected::dispatch(args, selected, presentation),
        Command::Wait(args) => control::dispatch(args, false, selected, presentation),
        Command::Cancel(args) => control::dispatch(args, true, selected, presentation),
        Command::Observe { action } => product::dispatch(action, selected, presentation),
        Command::Supervise(args) => runtime()?.block_on(crate::managed::supervisor::supervise(
            crate::managed::decode_checkout(&args.checkout_bytes)?,
            args.invocation,
        )),
        Command::Finalize(args) => {
            runtime()?.block_on(crate::managed::finalizer::finalize(
                crate::managed::decode_checkout(&args.checkout_bytes)?,
                args.invocation,
            ))?;
            Ok(0)
        }
        Command::Explain {
            intent_only,
            full,
            action,
        } => {
            let root = checkout_root(selected)?;
            let inputs = std::env::vars_os().collect();

            let metadata = crate::dependency::capture(&root, &inputs)?.metadata;
            match action {
                ExplainAction::Test(args) => {
                    let intent = crate::test_workflow::Intent::new(
                        metadata,
                        args.request()?,
                        args.profile,
                        inputs,
                    )
                    .map_err(|error| HostError::Workflow(Box::new(error)))?;
                    if intent_only {
                        return explain_emit(presentation, "test", full, intent.explain());
                    }
                    let admitted = intent
                        .admit()
                        .map_err(|error| HostError::Workflow(Box::new(error)))?;
                    explain_emit(presentation, "test", full, admitted.explain())
                }
                ExplainAction::Check(args) => {
                    let intent = crate::test_workflow::CheckIntent::new(
                        metadata,
                        args.request(crate::selection::Operation::Check)?,
                        inputs,
                    )
                    .map_err(|error| HostError::Workflow(Box::new(error)))?;
                    if intent_only {
                        return explain_emit(presentation, "check", full, intent.explain());
                    }
                    let admitted = intent
                        .admit()
                        .map_err(|error| HostError::Workflow(Box::new(error)))?;
                    explain_emit(presentation, "check", full, admitted.explain())
                }
            }
        }
        Command::Owner {
            path,
            metadata,
            xtask_selection,
        } => {
            let root = checkout_root(selected)?;
            let metadata = metadata
                .map(|path| {
                    source_path(selected, path).and_then(|path| {
                        crate::catalog::MetadataSnapshot::read(&path).map_err(HostError::from)
                    })
                })
                .transpose()?;
            let source = crate::ownership::source_owner(&root, &path, metadata.as_ref())?;
            let mut failures = Vec::new();
            let mut document_failure = None;
            let document = match xtask_selection {
                Some(id) => {

                    let inputs = std::env::vars_os().collect();
                    match crate::prepared_xtask::PreparedXtask::open(&root, &inputs, &id).and_then(|tool| tool.document_owner(&path, &inputs)) {
                        Ok(document) => crate::catalog::description::Information::Available(document),
                        Err(error) => { let reason = error.to_string(); document_failure = Some(error); crate::catalog::description::Information::Unavailable { reason } },
                    }
                }
                None => crate::catalog::description::Information::Unavailable { reason: "no prepared native document interpreter selected; use tools prepare xtask, then --xtask-selection ID".into() },
            };
            let generated = match crate::ownership::query(&root, &path) {
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
                "owner",
                "source and generated-output ownership observed independently",
                OwnerView {
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
            Ok(presentation.emit(response)?)
        }
        Command::Probe { action } => {
            probe::dispatch(action, &checkout_root(selected)?, presentation, entry)
        }
        Command::Changes(args) => changes::dispatch(args, selected, presentation),
        Command::Docs(args) => docs::dispatch(args, selected, presentation, entry),
        Command::Api(args) => api::dispatch(args, &checkout_root(selected)?, presentation, entry),
        Command::Code { action } => code::dispatch(action, selected, presentation, entry),
        Command::Compiler { action } => compiler::dispatch(action, selected, presentation, entry),
        Command::Worktree { action } => worktree::dispatch(action, selected, presentation),
        Command::Dep {
            package,
            action,
            metadata,
            why,
            feature_member,
        } => {
            if let Some(action) = action {
                return dependency::dispatch(action, selected, presentation);
            }
            let package = package.ok_or_else(|| {
                HostError::Ownership("select a dependency identity or dep units operation".into())
            })?;
            let snapshot = if let Some(path) = metadata.as_ref() {
                crate::catalog::MetadataSnapshot::read(&source_path(selected, path.clone())?)?
            } else {
                crate::dependency::capture(
                    &checkout_root(selected)?,
                    &std::env::vars_os().collect(),
                )?
            };
            Ok(presentation.emit(crate::dependency::inspect_details(
                &snapshot,
                &package,
                if metadata.is_some() {
                    "explicit_retained_snapshot"
                } else {
                    "locked_offline_metadata"
                },
                why,
                feature_member.as_deref(),
            ))?)
        }
        Command::Artifact {
            action:
                ArtifactAction::Read {
                    file,
                    offset,
                    length,
                    all,
                    if_match,
                },
        } => {
            let file = source_path(selected, file)?;
            let request = crate::artifacts::ReadRequest {
                offset,
                length: if all {
                    None
                } else {
                    length.or_else(|| crate::artifacts::ReadRequest::default().length)
                },
                expected_etag: if_match,
            };
            let payload = runtime()?.block_on(crate::artifacts::read_file(&file, &request))?;
            let mut response = Envelope::response(
                "artifact read",
                format!(
                    "artifact bytes {}..{} of {}; whole_object={}",
                    payload.byte_range.start,
                    payload.byte_range.end,
                    payload.object_bytes,
                    payload.whole_object
                ),
                payload,
            );
            response.observation.scope = "selected_artifact_byte_range".into();
            response.observation.limitations.push("native ETag is an opaque provider validator, not a content hash; local mutable files have no snapshot-isolation guarantee".into());
            if let Some(offset) = response.payload.next_offset {
                let file = std::path::absolute(file)?;
                let file = file.to_str().ok_or_else(|| {
                    HostError::Ownership(
                        "artifact continuation requires the native reader's Unicode filesystem key"
                            .into(),
                    )
                })?;
                let mut argv = vec![
                    "pse".into(),
                    "artifact".into(),
                    "read".into(),
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
                argv.extend(["--".into(), file.into()]);
                response.next.push(envelope::NextAction::Command{argv,reason:"read the next selected byte range under the observed native validator; no whole-file identity is inferred".into()});
            }
            source_emit(presentation, response, artifact_text)
        }
        Command::Outline {
            files,
            cursor,
            parsing,
        } => {
            if cursor.is_some() && files.len() != 1 {
                return Err(HostError::Ownership(
                    "an outline continuation selects exactly one file".into(),
                ));
            }
            let mut files = files
                .into_iter()
                .map(|file| {
                    selected.map_or_else(
                        || Ok(file.clone()),
                        |root| std::path::absolute(root.join(&file)),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if files.len() > 1 {
                if parsing.index || parsing.revision.is_some() {
                    return Err(HostError::Ownership(
                        "select one index/blob source per addressed outline".into(),
                    ));
                }
                let response = runtime()?.block_on(async {
                    let mut task = crate::source_command::batch::start_page(
                        files,
                        parsing.language,
                        parsing.edition,
                        tokio_util::sync::CancellationToken::new(),
                        presentation.presentation.budget,
                    );
                    tokio::select! {
                        response=task.settle()=>response.map_err(HostError::from),
                        signal=tokio::signal::ctrl_c()=>{
                            task.cancel();
                            let response=task.settle().await?;
                            signal?;
                            Ok(response)
                        },
                    }
                })?;
                let omitted_records = response
                    .payload
                    .records
                    .iter()
                    .map(|record| match record {
                        crate::source_command::batch::Record::Read { reading } => {
                            reading.page.omitted_records
                        }
                        _ => 0,
                    })
                    .sum();
                return Ok(presentation.emit_page(
                    response,
                    envelope::DetailView {
                        budget: presentation.presentation.budget,
                        omitted_records,
                        truncated: omitted_records > 0,
                        ..envelope::DetailView::default()
                    },
                )?);
            }
            let file = files.pop().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "select at least one source file",
                )
            })?;
            let request = source_request(selected, file, &parsing)?;
            let response = crate::source_command::outline_page(
                &request,
                parsing.language,
                parsing.edition,
                crate::source_read::PageRequest {
                    max_records: presentation.presentation.budget,
                    cursor,
                },
            )?;
            let detail = envelope::DetailView {
                budget: presentation.presentation.budget,
                omitted_records: response.payload.page.omitted_records,
                truncated: response.payload.page.omitted_records > 0,
                ..envelope::DetailView::default()
            };
            Ok(presentation.emit_page(response, detail)?)
        }
        Command::Show {
            source,
            item,
            at_byte,
            at_line,
            at_column,
            range_start,
            range_end,
            parsing,
        } => {
            let positional = at_byte.is_some() || at_line.is_some() || range_start.is_some();
            let (file, items) = if positional
                && item.is_empty()
                && !source
                    .as_encoded_bytes()
                    .windows(2)
                    .any(|part| part == b"::")
            {
                (PathBuf::from(source), Vec::new())
            } else {
                crate::source_command::selector(&source, item)?
            };
            let file = selected.map_or_else(
                || Ok(file.clone()),
                |root| std::path::absolute(root.join(&file)),
            )?;
            let position = if let Some(start) = range_start {
                crate::source_read::PositionSelector::Range {
                    start,
                    end: range_end
                        .ok_or_else(|| HostError::Ownership("range end is required".into()))?,
                }
            } else if let Some(line) = at_line {
                crate::source_read::PositionSelector::LineColumn {
                    line,
                    column: at_column
                        .ok_or_else(|| HostError::Ownership("column is required".into()))?,
                }
            } else if let Some(offset) = at_byte {
                crate::source_read::PositionSelector::Byte { offset }
            } else {
                crate::source_read::PositionSelector::Path
            };
            let response = crate::source_command::show_addressed(
                &source_request(selected, file, &parsing)?,
                &items,
                &position,
                parsing.language,
                parsing.edition,
            )?;
            source_emit(
                presentation,
                response,
                crate::source_command::selection_text,
            )
        }
        Command::Commands(args) => catalog_dispatch(args, true, "commands", selected, presentation),
        Command::Help(args) => catalog_dispatch(args, false, "help", selected, presentation),
        Command::Status { brief, line, all } => {
            let response = crate::status::inspect(
                &checkout_root(selected)?,
                &std::env::vars_os().collect(),
                all,
                if brief || line {
                    crate::status::InspectionScope::Summary
                } else {
                    crate::status::InspectionScope::Full
                },
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            );
            if line && presentation.format == OutputFormat::Human {
                let code = response.exit_code();
                writeln!(io::stdout(), "{}", crate::status::line(&response))?;
                return Ok(code);
            }
            if brief {
                let summary = response.payload.summary();
                let mut response = response.with_payload(summary);
                response.command = "status brief".into();
                response.observation.scope = "requested_native_status_summary".into();
                return Ok(presentation.emit(response)?);
            }
            Ok(presentation.emit(response)?)
        }
        Command::HostGenerationBootstrap { from } => {
            // These synchronous ownership/path checks do not start construction.

            let mode = crate::generation::bootstrap_mode(entry, from.as_deref())?;
            let root = checkout_root(selected)?;

            let outcome = crate::generation::bootstrap_from(&root, from.as_deref(), mode)?;

            Ok(outcome.exit_code)
        }
        Command::Schema { verb } => {
            let schema = response_schema(&verb.join(" "), &presentation.presentation)?;
            serde_json::to_writer_pretty(io::stdout().lock(), &schema).map_err(io::Error::other)?;
            writeln!(io::stdout())?;
            Ok(0)
        }
        Command::Boundary(options) => crate::boundary::execute(options, selected, presentation),
        Command::AgentHook { action } => crate::agent_hooks::execute(action),
        Command::Doctor { json } => {
            let report = runtime()?.block_on(crate::doctor::inspect(
                &checkout_root(selected)?,
                &std::env::vars_os().collect(),
            ));
            let mut response = Envelope::response(
                "doctor",
                if report.ready() {
                    "required native prerequisites are available"
                } else {
                    "required native prerequisites are unavailable"
                },
                report,
            );
            if !response.payload.ready() {
                response.outcome = CommandOutcome::Failure;
            }
            let mut presentation = presentation.clone();
            if json {
                presentation.format = OutputFormat::Json;
            }
            Ok(presentation.emit(response)?)
        }
        Command::CargoTools { names } => {
            for spec in
                crate::diagnostic_tools::cargo_tool_specs(&checkout_root(selected)?, &names)?
            {
                writeln!(io::stdout(), "{spec}")?;
            }
            Ok(0)
        }
        Command::DiagnosticTools { action } => {
            let (install, name) = match action {
                ToolAction::Install { tool } => (true, tool),
                ToolAction::Verify { tool } => (false, tool),
            };
            let tool = crate::diagnostic_tools::Tool::parse(&name)?;
            let values = std::env::vars_os().collect();
            let root = checkout_root(selected)?;
            let path = if install {
                crate::diagnostic_tools::install(&root, tool, &values)?
            } else {
                crate::diagnostic_tools::verify(&root, tool, &values)?
            };
            writeln!(io::stdout(), "{}", path.display())?;
            Ok(0)
        }
        Command::TelemetryService { action } => {
            let name = match &action {
                TelemetryAction::Install => "install",
                TelemetryAction::Setup(_) => "setup",
                TelemetryAction::Inspect(_) => "inspect",
                TelemetryAction::Start(_) => "start",
                TelemetryAction::Stop(_) => "stop",
                TelemetryAction::Reconfigure(_) => "reconfigure",
                TelemetryAction::RecoverConfiguration(_) => "recover-configuration",
            };
            let value = runtime()?.block_on(crate::telemetry_service::execute(
                &checkout_root(selected)?,
                &std::env::vars_os().collect(),
                action,
            ))?;
            Ok(presentation.emit(Envelope::response(
                format!("telemetry-service {name}"),
                format!("telemetry {name} response; delivery remains independently observed"),
                value,
            ))?)
        }
        Command::Native {
            action: NativeAction::Prepare { capabilities },
        } => {
            let prepared = crate::native::execute(&checkout_root(selected)?, &capabilities)?;
            Ok(presentation.emit(Envelope::response(
                "native prepare",
                "requested native profiles prepared",
                prepared,
            ))?)
        }
        Command::Server { action } => {
            let name = match &action {
                ServerAction::Select => "server select",
                ServerAction::Prepare { .. } => "server prepare",
            };
            let selected = crate::server::execute(&checkout_root(selected)?, action)?;
            Ok(presentation.emit(Envelope::response(
                name,
                "immutable server executable selected",
                ServerSelection {
                    executable: selected.executable(),
                    recipe: selected.recipe().into(),
                },
            ))?)
        }
        Command::StudyModule { action } => {
            let name = match &action {
                StudyAction::Prepare { .. } => "study-module prepare",
                StudyAction::Select { .. } => "study-module select",
            };
            let receipt = crate::study_module::execute(&checkout_root(selected)?, action)?;
            Ok(presentation.emit(Envelope::response(
                name,
                "immutable study module receipt selected",
                receipt,
            ))?)
        }
        Command::Env(options) => Ok(crate::environment::execute(
            options,
            selected,
            presentation,
        )?),
        Command::Activity { unit } => runtime()?.block_on(async {
            let value = crate::activity::ActivityObservation::read(unit.as_deref()).await;
            let complete = value.complete();
            let mut response = Envelope::response(
                "activity",
                if complete {
                    "activity observation complete"
                } else {
                    "activity observation incomplete"
                },
                value,
            );
            response.observation.scope = "manager_and_observer".into();
            if !complete {
                response.observation.completeness = Completeness::Partial;
                response
                    .observation
                    .limitations
                    .extend(response.payload.manager_unavailable.iter().cloned());
                response
                    .observation
                    .limitations
                    .extend(response.payload.observer.unavailable.iter().cloned());
            }
            Ok(presentation.emit(response)?)
        }),
        Command::History {
            session,
            all_sessions,
            limit,
        } => {
            let store = invocation_store(selected)?;
            let session = if all_sessions {
                None
            } else {
                crate::invocation::select_session(session, &std::env::vars_os().collect())?
            };
            let limit = usize::try_from(limit).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "history limit exceeds this platform",
                )
            })?;
            Ok(presentation.emit(crate::invocation_view::history(
                &store,
                session.as_deref(),
                limit,
                &RedactionPolicy::from_environment(),
            )?)?)
        }
        Command::Result {
            invocation,
            session,
            tests,
            failures,
            failure_cursor,
            cursor,
            limit,
            page_bytes,
        } => {
            let store = invocation_store(selected)?;
            let session =
                crate::invocation::select_session(session, &std::env::vars_os().collect())?;
            if failures {
                return Ok(presentation.emit(crate::test_inventory::failures(
                    &store,
                    &invocation,
                    session.as_deref(),
                    failure_cursor.as_ref(),
                    limit,
                    page_bytes,
                )?)?);
            }
            if tests {
                return Ok(presentation.emit(crate::test_inventory::read(
                    &store,
                    &invocation,
                    session.as_deref(),
                    cursor.as_ref(),
                    limit,
                    page_bytes,
                )?)?);
            }
            Ok(presentation.emit(crate::invocation_view::result(
                &store,
                &invocation,
                session.as_deref(),
            )?)?)
        }
        Command::Log {
            invocation,
            session,
            stderr,
            offset,
            length,
            tail,
        } => {
            let store = invocation_store(selected)?;
            let session =
                crate::invocation::select_session(session, &std::env::vars_os().collect())?;
            let stream = if stderr {
                crate::extraction::OutputStream::Stderr
            } else {
                crate::extraction::OutputStream::Stdout
            };
            Ok(presentation.emit(crate::invocation_view::log(
                &store,
                &invocation,
                session.as_deref(),
                stream,
                offset,
                length,
                tail,
            )?)?)
        }
        Command::Run {
            command,
            detach,
            session,
            resource_class,
        } => {
            let root = checkout_root(selected)?;
            let environment: Environment = std::env::vars_os().collect();
            let cwd = if selected.is_some() {
                root.clone()
            } else {
                std::env::current_dir()?
            };
            launch_managed(
                ManagedCommand {
                    parent: None,
                    root,
                    cwd,
                    environment,
                    command,
                    detach,
                    session,
                    resource_class,
                    verb: "run",
                    tool: None,
                    workflow: None,
                },
                entry,
                presentation,
            )
        }
    }
}
struct ManagedCommand {
    parent: Option<CliInvocationId>,
    root: PathBuf,
    cwd: PathBuf,
    environment: Environment,
    command: Vec<OsString>,
    detach: bool,
    session: Option<String>,
    resource_class: Option<String>,
    verb: &'static str,
    tool: Option<NativeToolSelection>,
    workflow: Option<(
        crate::managed::workflow::InputBinding,
        Box<serde_json::value::RawValue>,
    )>,
}
#[derive(serde::Serialize)]
struct NativeToolSelection {
    kind: &'static str,
    selection: String,
    source_identity: String,
    executable_sha256: String,
}
fn launch_managed(
    request: ManagedCommand,
    entry: &EntryGuard,
    presentation: &OutputOptions,
) -> Result<i32, HostError> {
    launch_managed_with_artifacts(request, entry, presentation, Vec::new())
}
fn launch_managed_with_artifacts(
    request: ManagedCommand,
    entry: &EntryGuard,
    presentation: &OutputOptions,
    artifacts: Vec<envelope::ArtifactReference>,
) -> Result<i32, HostError> {
    launch_managed_finalize(request, entry, presentation, artifacts, Vec::new(), || {
        Ok(Vec::new())
    })
}
fn launch_managed_finalize(
    request: ManagedCommand,
    entry: &EntryGuard,
    presentation: &OutputOptions,
    mut artifacts: Vec<envelope::ArtifactReference>,
    providers: Vec<envelope::ProviderContext>,
    finalize: impl FnOnce() -> Result<Vec<envelope::ArtifactReference>, HostError>,
) -> Result<i32, HostError> {
    let ManagedCommand {
        parent,
        root,
        cwd,
        environment,
        command,
        detach,
        session,
        resource_class,
        verb,
        tool,
        workflow,
    } = request;
    let session = crate::invocation::select_session(session, &environment)?;
    let control = entry.prepared().cloned().ok_or_else(|| HostError::Ownership("managed run requires an explicitly prepared native generation; run scripts/bootstrap-host".into()))?;
    let selection =
        crate::admission::Selection::load(&root, &environment, resource_class.as_deref(), None)?;
    let runtime = runtime()?;
    let environment_digest = environment_digest(&environment);
    let native_owner_output = tool.is_some();
    let state = crate::paths::CheckoutState::resolve(&root, &environment)?;
    let store = crate::invocation::InvocationStore::new(state);
    let directory = std::sync::Arc::new(store.begin(crate::invocation::InvocationSpec {
        invocation_id: presentation.invocation_id.clone(),
        command: verb.into(),
        session,
        parent,
        operation: RunSelection {
            command: command.clone(),
            cwd: cwd.as_os_str().to_owned(),
            detached: detach,
            tool,
            workflow: workflow.as_ref().map(|(binding, _)| binding.clone()),
        },
        control,
        environment_digest,
        scientific: Vec::new(),
    })?);
    if let Some((_, input)) = workflow {
        directory.publish_evidence(crate::invocation::InvocationEvidence::WorkflowInput, &input)?;
    }
    if !detach {
        let coordinator = crate::admission::Coordinator::open(&root, &environment)?;
        let policy = RedactionPolicy::from_inputs(&environment);
        let program = command
            .first()
            .ok_or_else(|| HostError::Ownership("empty foreground payload".into()))?;
        let observer = crate::managed::workflow::Observer::for_operation(
            &store,
            &directory,
            &root,
            Path::new(program),
            &command[1..],
        )?;
        let mut report = runtime.block_on(crate::foreground::run_captured(
            crate::foreground::ForegroundRequest {
                command,
                environment,
                cwd,
                require_enclosing: false,
                startup_deadline: Instant::now() + Duration::from_secs(30),
                drain_timeout: Duration::from_secs(30),
            },
            coordinator,
            selection,
            directory.clone(),
            observer,
            Duration::from_secs(5),
        ));
        let child_exit = report.child.as_ref().map(|status| {
            use std::os::unix::process::ExitStatusExt as _;
            status
                .code()
                .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
        });
        let required = crate::test_workflow::execute::Action::from_command(verb).is_some();
        let validation = runtime.block_on(crate::managed::workflow::finish_capture(
            directory.clone(),
            &mut report.capture,
            child_exit,
            required,
        ));
        let finalization_errors =
            if child_exit == Some(0) && report.operation.is_ok() && report.capture.is_ok() {
                match finalize() {
                    Ok(published) => {
                        artifacts.extend(published);
                        Vec::new()
                    }
                    Err(error) => vec![error],
                }
            } else {
                Vec::new()
            };
        let mut terminal = crate::managed::result::TerminalResult::from_foreground(
            directory.id(),
            verb,
            report,
            &policy,
        )
        .with_workflow(validation, &policy)
        .with_bundle(&directory, &policy)
        .with_owner_failures(finalization_errors, &policy)
        .with_providers(providers)
        .with_artifacts(artifacts);
        if native_owner_output {
            terminal = terminal.with_native_streams(&directory);
        }
        directory.publish_evidence(
            crate::invocation::InvocationEvidence::Settlement,
            terminal.response(),
        )?;
        directory.publish_result(&terminal)?;
        let response = store
            .result::<serde::de::IgnoredAny, crate::managed::result::ManagedSummary>(
                directory.id(),
            )?
            .ok_or_else(|| HostError::Ownership("published foreground result is absent".into()))?;
        return Ok(presentation
            .emit(response.map_payload(|payload| ManagedReply::Completed(Box::new(payload))))?);
    }
    let mut arguments = command.into_iter();
    let program = arguments
        .next()
        .ok_or_else(|| HostError::Ownership("empty managed payload".into()))?;
    let launched = runtime.block_on(crate::managed::supervisor::launch_generic(
        crate::managed::supervisor::ManagedRequest {
            root,
            directory,
            command: crate::managed::payload::PayloadCommand {
                program: program.into(),
                arguments: arguments.collect(),
                cwd,
                environment,
            },
            selection,
            policy: crate::managed::payload::PayloadPolicy {
                startup: Duration::from_secs(30),
                drain: Duration::from_secs(30),
                write: Duration::from_secs(5),
            },
        },
    ))?;
    let mut response = Envelope::response(
        verb,
        "original service launch acknowledged; completion remains unobserved",
        ManagedReply::Launched(Box::new(launched.observation)),
    );
    response.subject_state = envelope::SubjectState::Launched;
    response.settlement = envelope::Settlement::Unknown;
    // Launch acknowledgement intentionally does not require terminal settlement.
    response.outcome = CommandOutcome::Success;
    response.artifacts.extend(artifacts);
    response.providers.extend(providers);
    response.observation.required = false;
    response.observation.scope = "native_launch_acknowledgement".into();
    presentation.emit(response)?;
    Ok(0)
}
#[derive(serde::Serialize)]
struct RunSelection {
    command: Vec<OsString>,
    cwd: OsString,
    detached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<NativeToolSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow: Option<crate::managed::workflow::InputBinding>,
}
/// Actual managed command replies; launch acknowledgement and terminal evidence stay distinct.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(untagged)]
enum ManagedReply {
    Launched(Box<crate::UnitObservation>),
    Completed(Box<crate::managed::result::ManagedSummary>),
}
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct OwnerView {
    source: crate::ownership::SourceOwnerQuery,
    generated: crate::catalog::description::Information<crate::ownership::OwnerQuery>,
    document:
        crate::catalog::description::Information<crate::ownership::document::DocumentOwnership>,
}
pub(crate) fn environment_digest(environment: &Environment) -> String {
    use sha2::{Digest as _, Sha256};
    let mut digest = Sha256::new();
    for (name, value) in environment {
        for bytes in [name.as_encoded_bytes(), value.as_encoded_bytes()] {
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
    }
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn explain_emit(
    presentation: &OutputOptions,
    verb: &str,
    full: bool,
    explanation: crate::test_workflow::Explanation<'_>,
) -> Result<i32, HostError> {
    let summary = format!(
        "{verb} {:?} stage; remaining preparation and settlement obligations are explicit",
        explanation.stage
    );
    if full {
        Ok(presentation.emit(Envelope::response(
            format!("explain {verb} full"),
            summary,
            explanation,
        ))?)
    } else {
        Ok(presentation.emit(Envelope::response(
            format!("explain {verb}"),
            summary,
            explanation.brief(),
        ))?)
    }
}
fn invocation_store(
    selected: Option<&Path>,
) -> Result<crate::invocation::InvocationStore, HostError> {
    let root = checkout_root(selected)?;
    Ok(crate::invocation::InvocationStore::new(
        crate::paths::CheckoutState::resolve(&root, &std::env::vars_os().collect())?,
    ))
}
fn source_path(selected: Option<&Path>, file: PathBuf) -> Result<PathBuf, HostError> {
    match selected {
        Some(root) => Ok(checkout_root(Some(root))?.join(file)),
        None => Ok(file),
    }
}
fn artifact_text(view: &crate::artifacts::ArtifactRead) -> String {
    let content = match &view.content {
        crate::artifacts::Content::Utf8(text) => format!("UTF-8 selected bytes:\n{text}"),
        crate::artifacts::Content::Base64(text) => format!("base64 selected bytes:\n{text}"),
    };
    format!(
        "{}; bytes={}..{}; whole_object={}; next_offset={:?}; selected_sha256={}; native_etag={:?}; atomic_snapshot={}\n{content}\n",
        view.object_key,
        view.byte_range.start,
        view.byte_range.end,
        view.whole_object,
        view.next_offset,
        view.selected_sha256,
        view.etag,
        view.atomic_snapshot
    )
}
fn source_request(
    selected: Option<&Path>,
    file: PathBuf,
    parsing: &SourceParsing,
) -> Result<crate::source_read::SourceRequest, HostError> {
    if !parsing.index && parsing.revision.is_none() {
        return Ok(crate::source_read::SourceRequest::Working { path: file });
    }
    let directory = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    let repository = crate::repository::source::root(&directory)
        .map_err(crate::source_read::SourceReadError::from)
        .map_err(crate::source_command::SourceCommandError::from)?;
    let file = std::path::absolute(file)?;
    // Git tree/index paths use lexical repository coordinates even when the working file is
    // absent. Do not canonicalize the file or resolve worktree symlinks to address raw blobs.
    let mut normalized = PathBuf::new();
    for component in file.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    let path = normalized
        .strip_prefix(&repository)
        .map_err(|_| HostError::Ownership("Git source path is outside repository".into()))?
        .to_path_buf();
    Ok(if let Some(revision) = &parsing.revision {
        crate::source_read::SourceRequest::Blob {
            repository,
            revision: revision.clone(),
            path,
        }
    } else {
        crate::source_read::SourceRequest::Index { repository, path }
    })
}
fn source_emit<T: serde::Serialize + schemars::JsonSchema>(
    presentation: &OutputOptions,
    mut response: Envelope<T>,
    render: impl FnOnce(&T) -> String,
) -> Result<i32, HostError> {
    if presentation.format == OutputFormat::Human
        && presentation.presentation.fields.is_none()
        && presentation.presentation.budget.is_none()
    {
        response.invocation_id = presentation.invocation_id.clone();
        let _ = envelope::schema::<T>(&response.command, &presentation.presentation)?;
        let text = render(&response.payload);
        let frame = format!(
            "{}; invocation={}; outcome={:?}; coverage={:?}; subject={:?}; settlement={:?}; failures={}\n{text}",
            response.summary,
            response.invocation_id,
            response.outcome,
            response.observation.completeness,
            response.subject_state,
            response.settlement,
            response.failures.len()
        );
        io::stdout().lock().write_all(frame.as_bytes())?;
        Ok(response.exit_code())
    } else {
        let mut detail = response.view.clone();
        detail.budget = presentation.presentation.budget;
        Ok(presentation.emit_page(response, detail)?)
    }
}
fn catalog_envelope<T: serde::Serialize + schemars::JsonSchema>(
    page: crate::catalog::description::Page<T>,
    command: &str,
    context: String,
) -> Envelope<crate::catalog::description::Page<T>> {
    use crate::catalog::description::{Availability, Exhaustiveness, Freshness};
    let mut response = Envelope::response(
        command,
        format!(
            "{} catalog records; source availability {:?}",
            page.records.len(),
            page.availability
        ),
        page,
    );
    response.observation.scope = "requested_catalog_page".into();
    response.observation.limitations = response.payload.limitations.clone();
    if response.payload.availability != Availability::Available
        || response.payload.exhaustiveness != Exhaustiveness::Exhaustive
    {
        response.observation.completeness = Completeness::Partial;
        response.observation.required = false;
    }
    let availability = if response.payload.source.freshness == Freshness::Stale {
        envelope::SubjectState::Stale
    } else {
        match response.payload.availability {
            Availability::Available => envelope::SubjectState::Available,
            Availability::Unavailable => envelope::SubjectState::Unavailable,
            Availability::Unknown => envelope::SubjectState::Unknown,
        }
    };
    response.providers.push(envelope::ProviderContext {
        provider: response.payload.source.owner.clone(),
        context: Some(context),
        analysis_generation: response.payload.source.generation.clone(),
        availability,
        coordinates: Vec::new(),
    });
    response
}
fn catalog_dispatch(
    args: CatalogArgs,
    recursive: bool,
    command: &str,
    selected: Option<&Path>,
    presentation: &OutputOptions,
) -> Result<i32, HostError> {
    if args.markdown {
        io::stdout()
            .lock()
            .write_all(catalog::markdown::render(&args.path)?.as_bytes())?;
        return Ok(0);
    }
    use crate::catalog::{
        self,
        description::{self, Window},
    };
    let window = Window {
        prefix: args.prefix.clone(),
        offset: args.offset,
        limit: args
            .limit
            .map(|limit| usize::try_from(limit).unwrap_or(usize::MAX)),
    };
    let mut presentation = presentation.clone();
    if args.json {
        presentation.format = OutputFormat::Json;
    }
    if let Some(argument) = &args.argument {
        let mut provider_failures = Vec::new();
        let page = description::candidates(
            catalog::command_tree(),
            &args.path,
            argument,
            &window,
            |name| {
                if name == catalog::Domain::Packages.name()
                    || name == catalog::Domain::Targets.name()
                    || name == catalog::Domain::Dependencies.name()
                {
                    if let Some(path) = &args.metadata {
                        let path = if path.is_relative() {
                            selected
                                .map(|root| root.join(path))
                                .unwrap_or_else(|| path.clone())
                        } else {
                            path.clone()
                        };
                        let snapshot = catalog::MetadataSnapshot::read(&path)?;
                        return catalog::Context {
                            metadata: Some(&snapshot.metadata),
                            metadata_generation: Some(&snapshot.digest),
                            metadata_freshness: Some(description::Freshness::RetainedSnapshot),
                            ..catalog::Context::default()
                        }
                        .provide(name)
                        .map(|result| {
                            provider_failures.extend(result.failures);
                            result.candidates
                        });
                    }
                } else if name == catalog::Domain::Invocations.name() {
                    let state = crate::paths::CheckoutState::resolve(
                        &checkout_root(selected)?,
                        &std::env::vars_os().collect(),
                    )?;
                    let store = crate::invocation::InvocationStore::new(state);
                    return catalog::Context {
                        invocations: Some(&store),
                        ..catalog::Context::default()
                    }
                    .provide(name)
                    .map(|result| {
                        provider_failures.extend(result.failures);
                        result.candidates
                    });
                }
                catalog::Context::default().provide(name).map(|result| {
                    provider_failures.extend(result.failures);
                    result.candidates
                })
            },
        )?;
        let mut response = catalog_envelope(
            page,
            &format!("{command} candidates"),
            format!("{} argument {argument}", args.path.join(" ")),
        );
        if !provider_failures.is_empty() {
            let policy = RedactionPolicy::from_environment();
            response.failures.extend(
                provider_failures
                    .iter()
                    .map(|failure| ErrorRecord::from_typed(failure, &policy)),
            );
            response.observation.completeness = Completeness::Partial;
        }
        Ok(presentation.emit(response)?)
    } else {
        let page = catalog::commands(&args.path, recursive, &window);
        Ok(presentation.emit(catalog_envelope(page, command, args.path.join(" ")))?)
    }
}

/// Run a native host entrypoint. The trampoline and raw Cargo protocol bypass parsing.
pub fn main(kind: BinKind) -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    if matches!(kind, BinKind::Correctness) {
        let _entry = if arguments.as_slice() == [OsString::from("--version")] {
            None
        } else {
            match EntryGuard::acquire_current() {
                Ok(entry) => Some(entry),
                Err(error) => {
                    let _ = writeln!(io::stderr(), "arrow-validation: {error}");
                    return ExitCode::from(2);
                }
            }
        };
        return match crate::correctness::run(arguments) {
            Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
            Err(error) => {
                let _ = writeln!(io::stderr(), "arrow-validation: {error}");
                ExitCode::from(2)
            }
        };
    }
    if arguments
        .first()
        .is_some_and(|arg| arg == "__foreground-exec")
    {
        // Inherited-descriptor initialization precedes presentation and every other owner.
        return match crate::foreground::trampoline(&arguments[1..]) {
            Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
            Err(error) => {
                let _ = writeln!(io::stderr(), "pse-host: {error}");
                ExitCode::from(125)
            }
        };
    }
    if !matches!(kind, BinKind::Correctness) && arguments.len() == 1 && arguments[0] == "--version"
    {
        let name = if matches!(kind, BinKind::Pse) {
            "pse"
        } else {
            "pse-host"
        };
        let _ = writeln!(
            io::stdout(),
            "{name} {} protocol 1",
            env!("CARGO_PKG_VERSION")
        );
        return ExitCode::SUCCESS;
    }
    let policy = RedactionPolicy::from_environment();
    let mut presentation = OutputOptions {
        format: OutputFormat::early(
            presentation_prefix(&arguments),
            std::env::var("PSE_FORMAT").ok().as_deref(),
            io::stdout().is_terminal(),
        ),
        presentation: Presentation::default(),
        invocation_id: CliInvocationId::new(),
        artifact_directory: None,
    };
    let mut raw_protocol = arguments.first().is_some_and(|arg| {
        arg == "boundary"
            || arg == "__host-generation-bootstrap"
            || arg == "agent-hook"
            || arg == "cargo-tools"
            || arg == "diagnostic-tools"
    });
    let result = {
        let mut argv = vec![OsString::from(if matches!(kind, BinKind::Pse) {
            "pse"
        } else {
            "pse-host"
        })];
        argv.extend(arguments.clone());
        match Cli::try_parse_from(argv) {
            Ok(cli) if cli.version => {
                let name = if matches!(kind, BinKind::Pse) {
                    "pse"
                } else {
                    "pse-host"
                };
                let _ = writeln!(
                    io::stdout(),
                    "{name} {} protocol 1",
                    env!("CARGO_PKG_VERSION")
                );
                Ok(0)
            }
            Ok(cli) if cli.command.is_none() => {
                use clap::CommandFactory as _;
                let _ = Cli::command().print_help();
                let _ = writeln!(io::stdout());
                Ok(0)
            }
            Ok(cli) => {
                presentation.format = cli.format.resolve(io::stdout().is_terminal());
                presentation.presentation = Presentation {
                    fields: cli.fields.clone(),
                    budget: cli.budget,
                };
                raw_protocol = cli.command.as_ref().is_some_and(Command::raw_protocol);
                (|| {
                    let entry = EntryGuard::acquire_current()?;
                    let intent = cli
                        .command
                        .as_ref()
                        .map(Command::generation_intent)
                        .unwrap_or(GenerationIntent::Passive);
                    if intent == GenerationIntent::NewWork && entry.prepared().is_some() {
                        let root = checkout_root(cli.root.as_deref())?;
                        let environment = std::env::vars_os().collect();
                        match runtime()?
                            .block_on(crate::generation::new_work_decision(&root, &environment))?
                        {
                            NewWorkDecision::Current | NewWorkDecision::PreserveNested => {}
                            NewWorkDecision::Rebuild => {
                                let tool = match kind {
                                    BinKind::Pse => HostTool::Pse,
                                    BinKind::Host => HostTool::Host,
                                    BinKind::Correctness => HostTool::Correctness,
                                };
                                return crate::generation::rebuild_and_reexec(
                                    &root, tool, &arguments,
                                );
                            }
                        }
                    }
                    if !raw_protocol
                        && !matches!(
                            cli.command.as_ref(),
                            Some(
                                Command::Schema { .. }
                                    | Command::Outline { .. }
                                    | Command::Show { .. }
                                    | Command::Api(_)
                                    | Command::Code { .. }
                                    | Command::Compiler {
                                        action: compiler::Action::Diagnostics { .. }
                                            | compiler::Action::Diagnostic { .. }
                                    }
                            )
                        )
                        && (cli.fields.is_some() || cli.budget.is_some())
                    {
                        let root = checkout_root(cli.root.as_deref())?;
                        presentation.artifact_directory = Some(
                            crate::paths::CheckoutState::resolve(
                                &root,
                                &std::env::vars_os().collect(),
                            )?
                            .root
                            .join("artifacts"),
                        );
                    }
                    dispatch(cli, &presentation, &entry)
                })()
            }
            Err(error) => {
                if error.use_stderr() && arguments.first().is_some_and(|arg| arg == "boundary") {
                    let _ = writeln!(io::stderr(), "pse-env: {error}");
                    Ok(125)
                } else if error.use_stderr()
                    && arguments.first().is_some_and(|arg| arg == "agent-hook")
                {
                    let _ = writeln!(io::stderr(), "BLOCKED: {error}");
                    Ok(2)
                } else if !error.use_stderr() {
                    let code = error.exit_code();
                    let _ = error.print();
                    Ok(code)
                } else {
                    Ok(failure(
                        ErrorRecord::from_clap(&error, &policy),
                        CommandOutcome::Usage,
                        &presentation,
                        None,
                        &policy,
                    ))
                }
            }
        }
    };
    match result {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(125)),
        Err(error) => {
            if !raw_protocol {
                let outcome = if matches!(&error,HostError::Io(error) if error.kind()==io::ErrorKind::InvalidInput)
                {
                    CommandOutcome::Usage
                } else {
                    CommandOutcome::BoundaryFailure
                };
                return ExitCode::from(
                    u8::try_from(failure(
                        ErrorRecord::from_typed(&error, &policy),
                        outcome,
                        &presentation,
                        Some(&error),
                        &policy,
                    ))
                    .unwrap_or(125),
                );
            }
            let prefix = if arguments.first().is_some_and(|arg| arg == "boundary") {
                "pse-env"
            } else {
                "pse-host"
            };
            let _ = writeln!(io::stderr(), "{prefix}: {error}");
            ExitCode::from(125)
        }
    }
}
fn presentation_prefix(arguments: &[OsString]) -> &[OsString] {
    use clap::CommandFactory as _;
    let mut tree = Cli::command();
    tree.build();
    let mut command = &tree;
    let mut index = 0;
    while index < arguments.len() {
        let value = &arguments[index];
        if value == "--" {
            return &arguments[..index];
        }
        if let Some(text) = value.to_str() {
            let flag = if let Some(long) = text.strip_prefix("--") {
                let name = long.split('=').next().unwrap_or(long);
                command
                    .get_arguments()
                    .find(|arg| arg.get_long() == Some(name))
                    .map(|arg| (arg, text.contains('=')))
            } else if let Some(short) = text.strip_prefix('-').and_then(|text| text.chars().next())
            {
                command
                    .get_arguments()
                    .find(|arg| arg.get_short() == Some(short))
                    .map(|arg| (arg, text.chars().count() > 2))
            } else {
                None
            };
            if let Some((arg, inline)) = flag {
                if !inline && !arg.is_require_equals_set() && arg.get_action().takes_values() {
                    index += arg.get_num_args().map_or(1, |range| range.min_values());
                }
                index += 1;
                continue;
            }
            if !text.starts_with('-') {
                if let Some(child) = command
                    .get_subcommands()
                    .find(|child| child.get_name() == text)
                {
                    command = child;
                } else if command
                    .get_arguments()
                    .any(clap::Arg::is_trailing_var_arg_set)
                {
                    return &arguments[..index];
                }
            }
        } else if command
            .get_arguments()
            .any(clap::Arg::is_trailing_var_arg_set)
        {
            return &arguments[..index];
        }
        index += 1;
    }
    arguments
}
fn failure(
    record: ErrorRecord,
    outcome: CommandOutcome,
    presentation: &OutputOptions,
    native: Option<&dyn miette::Diagnostic>,
    policy: &RedactionPolicy,
) -> i32 {
    let mut response = Envelope::response(
        "error",
        "command could not produce the requested response",
        (),
    );
    response.invocation_id = presentation.invocation_id.clone();
    response.outcome = outcome;
    response.failures.push(record);
    let code = response.exit_code();
    if envelope::write(
        io::stderr().lock(),
        response,
        presentation.format,
        &Presentation::default(),
    )
    .is_err()
    {
        return 125;
    }
    if presentation.format == OutputFormat::Human
        && let Some(native) = native
    {
        let mut rendered = String::new();
        let result = if io::stderr().is_terminal() {
            miette::GraphicalReportHandler::new().render_report(&mut rendered, native)
        } else {
            miette::NarratableReportHandler::new().render_report(&mut rendered, native)
        };
        if result.is_ok() {
            let _ = writeln!(io::stderr(), "{}", policy.redact(&rendered));
        }
    }
    code
}

#[cfg(test)]
mod tests {
    #[test]
    fn fixture_routes_preserve_effect_intent_and_schema() {
        use clap::Parser as _;
        for (action, expected) in [
            ("prepare", GenerationIntent::NewWork),
            ("show", GenerationIntent::Passive),
        ] {
            let parsed = Cli::try_parse_from(["pse", "fixture", action, "study-module"]).unwrap();
            assert_eq!(parsed.command.unwrap().generation_intent(), expected);
            assert!(response_schema(&format!("fixture {action}"), &Default::default()).is_ok());
        }
        assert!(Cli::try_parse_from(["pse", "fixture", "show", "unknown"]).is_err());
    }
    #[test]
    fn failure_pages_are_passive_and_exclusive() {
        let cli = Cli::try_parse_from([
            "pse",
            "result",
            "latest",
            "--failures",
            "--limit",
            "2",
            "--page-bytes",
            "2048",
        ])
        .unwrap();
        assert_eq!(
            cli.command.unwrap().generation_intent(),
            GenerationIntent::Passive
        );
        assert!(response_schema("result failures", &Default::default()).is_ok());
        assert!(Cli::try_parse_from(["pse", "result", "latest", "--tests", "--failures"]).is_err());
        assert!(
            Cli::try_parse_from(["pse", "result", "latest", "--failure-cursor", "{}"]).is_err()
        );
        let helper = Cli::try_parse_from(["pse", "__fixture-prepare"]).unwrap();
        assert_eq!(
            helper.command.unwrap().generation_intent(),
            GenerationIntent::Internal
        );
    }
    use super::*;
    #[test]
    fn bootstrap_precondition_refusal_reports_no_construction_effects() {
        let absent = tempfile::tempdir().unwrap();
        let cli = Cli::try_parse_from([
            OsString::from("pse"),
            OsString::from("-C"),
            absent.path().as_os_str().to_owned(),
            OsString::from("__host-generation-bootstrap"),
        ])
        .unwrap();
        let entry = EntryGuard::acquire_current().unwrap();
        let presentation = OutputOptions {
            format: OutputFormat::Json,
            presentation: Presentation::default(),
            invocation_id: CliInvocationId::new(),
            artifact_directory: None,
        };
        assert!(dispatch(cli, &presentation, &entry).is_err());
        assert_eq!(std::fs::read_dir(absent.path()).unwrap().count(), 0);
    }
    use clap::CommandFactory as _;
    #[test]
    fn command_tree_is_consistent() {
        Cli::command().debug_assert();
        CorrectnessCli::command().debug_assert();
        crate::catalog::command_tree().debug_assert();
    }
    #[test]
    fn generation_intent_is_independent_of_output_protocol_and_child_arguments() {
        for (arguments, expected) in [
            (
                vec!["pse", "run", "child", "--help"],
                GenerationIntent::NewWork,
            ),
            (
                vec!["pse", "boundary", "--print"],
                GenerationIntent::Passive,
            ),
            (
                vec!["pse", "boundary", "--explain"],
                GenerationIntent::Passive,
            ),
            (
                vec!["pse", "boundary", "child", "--explain"],
                GenerationIntent::NewWork,
            ),
            (
                vec!["pse", "agent-hook", "guard"],
                GenerationIntent::Passive,
            ),
            (vec!["pse", "schema", "doctor"], GenerationIntent::Passive),
            (
                vec!["pse", "telemetry-service", "stop"],
                GenerationIntent::ExistingControl,
            ),
            (
                vec!["pse", "telemetry-service", "start"],
                GenerationIntent::NewWork,
            ),
            (
                vec!["pse", "__host-generation-bootstrap"],
                GenerationIntent::Internal,
            ),
        ] {
            let cli = Cli::try_parse_from(arguments).unwrap();
            assert_eq!(cli.command.unwrap().generation_intent(), expected);
        }
    }

    #[test]
    fn invocation_observers_select_passive_reads_and_reject_invalid_extents() {
        for arguments in [
            vec!["pse", "history", "--session", "session", "--limit", "3"],
            vec!["pse", "result", "latest"],
            vec![
                "pse", "log", "latest", "--stderr", "--tail", "--length", "4",
            ],
        ] {
            let parsed = Cli::try_parse_from(arguments).unwrap();
            let command = parsed.command.unwrap();
            assert_eq!(command.generation_intent(), GenerationIntent::Passive);
            assert!(!command.raw_protocol());
        }
        for arguments in [
            vec!["pse", "history", "--limit", "0"],
            vec!["pse", "log", "latest", "--length", "0"],
            vec!["pse", "log", "latest", "--offset", "1", "--tail"],
        ] {
            assert!(Cli::try_parse_from(arguments).is_err());
        }
    }

    #[test]
    fn performing_managed_reply_schema_accepts_launch_and_original_terminal_views() {
        let replies = [
            ManagedReply::Launched(Box::new(crate::UnitObservation {
                unit: "fixture.service".into(),
                invocation: vec![1; 16],
                active_state: "active".into(),
                sub_state: "running".into(),
                main_pid: 42,
                exit_code: 0,
                exit_status: 0,
                control_group: "/fixture".into(),
            })),
            ManagedReply::Completed(Box::new(crate::managed::result::ManagedSummary {
                bundle: None,
                native_lifetime_pending: false,
                workflow: None,
                child_exit: Some(125),
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: None,
            })),
        ];
        for reply in replies {
            let mut response = Envelope::response("test", "original managed command", reply);
            if matches!(response.payload, ManagedReply::Completed(_)) {
                response.outcome = CommandOutcome::BoundaryFailure;
                response.subject_state = envelope::SubjectState::Unknown;
                response.child_exit = Some(125);
            }
            let value = serde_json::to_value(&response).unwrap();
            let schema = response_schema("test", &Presentation::default()).unwrap();
            let validator = jsonschema::validator_for(schema.as_value()).unwrap();
            assert!(
                validator.is_valid(&value),
                "{:?}",
                validator
                    .iter_errors(&value)
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
            );
            assert_eq!(value["command"], "test");
            assert!(value["payload"].get("response").is_none());
        }
    }
    #[test]
    fn early_format_prefix_comes_from_the_shared_parser_tail_declaration() {
        let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();
        for (arguments, expected) in [
            (args(&["run", "--format", "json"]), OutputFormat::Json),
            (
                args(&[
                    "run", "--format", "ndjson", "--", "child", "--format", "human",
                ]),
                OutputFormat::Ndjson,
            ),
            (
                args(&["run", "child", "--format", "json", "-h"]),
                OutputFormat::Human,
            ),
            (
                args(&[
                    "boundary",
                    "--cache",
                    "auto",
                    "--format=json",
                    "child",
                    "--format",
                    "human",
                ]),
                OutputFormat::Json,
            ),
        ] {
            assert_eq!(
                OutputFormat::early(presentation_prefix(&arguments), Some("human"), false),
                expected
            );
        }
    }
    #[test]
    fn help_snapshots_cover_the_shared_command_tree_without_running_effects() {
        fn visit(mut command: clap::Command, path: String) {
            // Terminal alignment padding is not a command/help semantic contract.
            let normalize = |text: String| {
                text.lines()
                    .map(str::trim_end)
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let short = normalize(command.render_help().to_string());
            let long = normalize(command.render_long_help().to_string());
            insta::assert_snapshot!(
                format!("help_{}", path.replace([' ', '-'], "_")),
                format!("-h\n{short}\n--help\n{long}")
            );
            for child in command.get_subcommands() {
                visit(child.clone(), format!("{path} {}", child.get_name()));
            }
        }
        let mut command = Cli::command();
        command.build();
        visit(command, "pse".into());
    }
    #[test]
    fn managed_response_snapshots_validate_against_the_command_schema() {
        fn snapshot<T: serde::Serialize + schemars::JsonSchema>(command: &str, payload: T) {
            let mut response = Envelope::response(
                command,
                "fixture response; no scientific outcome asserted",
                payload,
            );
            if command == "activity" {
                response.observation.completeness = Completeness::Partial;
                response
                    .observation
                    .limitations
                    .push("fixture intentionally lacks manager and kernel facts".into());
            }
            if command.starts_with("status") {
                response.observation.completeness = Completeness::Partial;
                response.observation.limitations.push(
                    "fixture intentionally leaves requested source observations unavailable".into(),
                );
            }
            response.invocation_id =
                CliInvocationId::try_from("019a0000-0000-7000-8000-000000000001".to_owned())
                    .unwrap();
            let value = serde_json::to_value(&response).unwrap();
            let schema = response_schema(command, &Presentation::default()).unwrap();
            let validator = jsonschema::validator_for(schema.as_value()).unwrap();
            assert!(
                validator.is_valid(&value),
                "{command}: {:?}",
                validator
                    .iter_errors(&value)
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
            );
            insta::assert_snapshot!(
                format!("response_{}", command.replace([' ', '-'], "_")),
                serde_json::to_string_pretty(&response).unwrap()
            );
        }
        snapshot(
            "doctor",
            crate::doctor::Report {
                version: 1,
                checks: vec![crate::doctor::Check {
                    name: "fixture".into(),
                    ok: true,
                    blocking: true,
                    detail: "read-only observation available".into(),
                    fix: String::new(),
                }],
            },
        );
        snapshot(
            "commands",
            crate::catalog::commands(
                &["native".into()],
                false,
                &crate::catalog::description::Window::default(),
            ),
        );
        snapshot(
            "help",
            crate::catalog::commands(
                &["native".into(), "prepare".into()],
                false,
                &crate::catalog::description::Window::default(),
            ),
        );
        fn absent<T>() -> crate::catalog::description::Information<T> {
            crate::catalog::description::Information::Unavailable {
                reason: "fixture source unavailable".into(),
            }
        }
        let status = crate::status::Report {
            checkout_id: absent(),
            repository_id: absent(),
            repository: absent(),
            plans: absent(),
            invocations: absent(),
            host_tools: absent(),
            worktrees: absent(),
            product_observation: absent(),
            atomic_snapshot: false,
        };
        let summary = status.summary();
        snapshot("status", status);
        snapshot("status brief", summary);
        let source = "/// fixture\nfn observed() {}\n";
        let outline = crate::source_read::rust(source, crate::source_read::RustEdition::E2024);
        let selection =
            crate::source_read::show(source, &outline, &["observed".into()], None).unwrap();
        let source_identity =
            crate::source_read::SourceText::new(PathBuf::from("/fixture.rs"), source.into())
                .unwrap()
                .identity()
                .clone();
        let page = || crate::source_read::PageState {
            total_records: outline.entries.len(),
            offset: 0,
            returned_records: outline.entries.len(),
            omitted_records: 0,
            next_cursor: None,
        };
        snapshot(
            "outline",
            crate::source_command::FileOutline {
                file: "fixture.rs".into(),
                source: source_identity.clone(),
                page: page(),
                outline: outline.clone(),
            },
        );
        snapshot(
            "show",
            crate::source_command::FileSelection {
                file: "fixture.rs".into(),
                locator: crate::source_read::SourceLocator {
                    source: source_identity.clone(),
                    range: selection.range.clone(),
                },
                backend: outline.backend,
                rust_edition: outline.rust_edition,
                parse_issues: outline.parse_issues,
                selection,
            },
        );
        snapshot(
            "outline batch",
            crate::source_command::batch::Report {
                records: vec![crate::source_command::batch::Record::Read {
                    reading: Box::new(crate::source_command::FileOutline {
                        file: "fixture.rs".into(),
                        source: source_identity.clone(),
                        page: crate::source_read::PageState {
                            total_records: 1,
                            offset: 0,
                            returned_records: 1,
                            omitted_records: 0,
                            next_cursor: None,
                        },
                        outline: crate::source_read::rust(
                            source,
                            crate::source_read::RustEdition::E2024,
                        ),
                    }),
                }],
                cancellation_requested: false,
                cpu_work_completed: true,
            },
        );
        snapshot(
            "artifact read",
            crate::artifacts::ArtifactRead {
                object_key: "fixture/evidence".into(),
                object_bytes: 10,
                etag: Some("native-validator".into()),
                version: None,
                byte_range: 0..3,
                selected_sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                    .into(),
                content: crate::artifacts::Content::Utf8("abc".into()),
                whole_object: false,
                next_offset: Some(3),
                atomic_snapshot: false,
            },
        );
        snapshot(
            "env explain",
            crate::environment::EnvironmentExplanation {
                local_unresolved: true,
                selections: [("PSE_BUILD_MODE".into(), Some("unbounded".into()))].into(),
            },
        );
        snapshot(
            "activity",
            crate::activity::ActivityObservation {
                requested_unit: Some("fixture.service".into()),
                manager: None,
                manager_unavailable: Some("fixture bus unavailable".into()),
                observer: crate::activity::ObserverFacts {
                    identity: None,
                    control_group: None,
                    unavailable: vec!["fixture kernel unavailable".into()],
                },
            },
        );
        snapshot(
            "boundary explain native",
            crate::boundary::NativeExplanation {
                native_profile: "compiler".into(),
                readiness: "not_observed".into(),
                personal_selections: "unresolved".into(),
                prepare: Some("explicit preparation required".into()),
            },
        );
        for action in [
            "install",
            "setup",
            "inspect",
            "start",
            "stop",
            "reconfigure",
            "recover-configuration",
        ] {
            snapshot(
                &format!("telemetry-service {action}"),
                serde_json::json!({"status":"fixture","delivery":"not_observed"}),
            );
        }
        snapshot(
            "native prepare",
            std::collections::BTreeMap::from([(
                "compiler".to_owned(),
                PathBuf::from("/fixture/profile"),
            )]),
        );
        for action in ["server select", "server prepare"] {
            snapshot(
                action,
                ServerSelection {
                    executable: "/fixture/surreal".into(),
                    recipe: "fixture-generation".into(),
                },
            );
        }
        for action in ["study-module select", "study-module prepare"] {
            // This is the receipt's declared wire record, not a fabricated prepared capability.
            snapshot(
                action,
                pse_buildinfo::study_module::StudyModuleRecord {
                    kind: "fixture".into(),
                    artifact: "/fixture/study.surli".into(),
                    sha256: "0".repeat(64),
                    bytes: 32,
                    sdk_version: "fixture".into(),
                    abi: 1,
                    inputs: serde_json::Map::new(),
                    limits: pse_buildinfo::study_module::ModuleLimits {
                        max_memory_bytes: 1024,
                        max_pool_size: 1,
                        max_execution_time_ms: 1000,
                        max_kv_entries: 0,
                        max_kv_value_bytes: 0,
                        strict_timeout: true,
                    },
                },
            );
        }
    }
    #[test]
    fn boundary_tail_is_literal_and_native_is_last_wins() {
        let cli = Cli::try_parse_from([
            "pse-host",
            "boundary",
            "--native=compiler,klu",
            "--native",
            "tool",
            "--native=",
            "-h",
        ])
        .unwrap();
        let Some(Command::Boundary(options)) = cli.command else {
            panic!("boundary")
        };
        assert_eq!(options.native, Some(None));
        assert_eq!(
            options.command,
            vec![OsString::from("tool"), "--native=".into(), "-h".into()]
        );
    }
    #[test]
    fn run_preserves_child_options_and_env_rejects_conflicts() {
        let cli = Cli::try_parse_from([
            "pse",
            "run",
            "--",
            "/bin/true",
            "--help",
            "--format",
            "json",
        ])
        .unwrap();
        let Some(Command::Run { command, .. }) = cli.command else {
            panic!("run")
        };
        assert_eq!(
            command,
            vec![
                OsString::from("/bin/true"),
                "--help".into(),
                "--format".into(),
                "json".into()
            ]
        );
        assert!(Cli::try_parse_from(["pse", "env", "--print", "--explain"]).is_err());
        assert!(
            Cli::try_parse_from([
                "pse",
                "env",
                "--explain",
                "--json",
                "--docs",
                "--cache",
                "off"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "pse",
                "env",
                "--activation-data",
                "--root",
                "/checkout",
                "--cache",
                "auto"
            ])
            .is_ok()
        );
        let cli =
            Cli::try_parse_from(["pse-host", "run", "/bin/true", "--help", "--format", "json"])
                .unwrap();
        let Some(Command::Run { command, .. }) = cli.command else {
            panic!("run")
        };
        assert_eq!(
            command,
            vec![
                OsString::from("/bin/true"),
                "--help".into(),
                "--format".into(),
                "json".into()
            ]
        );
    }
}
