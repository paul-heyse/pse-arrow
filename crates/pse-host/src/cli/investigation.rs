// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public investigation intents lower to one validated query model.
use super::{HostError, OutputOptions};
use crate::{
    envelope::{CommandOutcome, Completeness, Envelope, OutputFormat},
    investigation as query,
};
use clap::{Args, Subcommand, ValueEnum};
use serde_json::{Value, json};
use std::io::IsTerminal as _;
use std::path::Path;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
mod delivery;
mod evidence;
mod human;
mod proposal;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Order {
    Arrival,
    Deterministic,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum HistoryMode {
    Log,
    Blame,
    Pickaxe,
    PickaxeLines,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum CountBy {
    Matches,
    Files,
    Lines,
    Entities,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Matching {
    Contains,
    Exact,
    Prefix,
    Fuzzy,
}

#[derive(Debug, Args)]
pub(super) struct Search {
    /// Read a complete typed JSON request; explicit flags override selected fields.
    #[arg(long)]
    pub request: Option<PathBuf>,
    /// Explicit named-query configuration; otherwise use the optional repository file.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Select a saved typed query from configuration.
    #[arg(long)]
    pub query: Option<String>,
    /// Restrict source paths without requiring Cargo or analyzer preparation.
    #[arg(long = "path", global = true)]
    pub paths: Vec<PathBuf>,
    #[arg(long, global = true)]
    pub package: Vec<String>,
    #[arg(long, global = true)]
    pub glob: Vec<String>,
    #[arg(long = "exclude", global = true)]
    pub exclude: Vec<String>,
    #[arg(long = "type", global = true)]
    pub types: Vec<String>,
    #[arg(long, global = true, conflicts_with = "untracked")]
    pub tracked: bool,
    #[arg(long, global = true)]
    pub untracked: bool,
    #[arg(long, global = true)]
    pub changed: bool,
    #[arg(long, global = true)]
    pub dependencies: bool,
    #[arg(long, global = true)]
    pub include_ignored: bool,
    #[arg(long, global = true)]
    pub hidden: bool,
    #[arg(long, global = true)]
    pub generated: bool,
    #[arg(long, global = true, conflicts_with = "revision")]
    pub index: bool,
    #[arg(long, global = true)]
    pub revision: Option<String>,
    /// Stop after this many useful results; the default is twenty.
    #[arg(long, global = true)]
    pub limit: Option<usize>,
    /// Examine the complete selected scope instead of using the default result limit.
    #[arg(long, global = true, conflicts_with_all = ["limit", "exists"])]
    pub all: bool,
    /// Stop after a positive witness; a negative answer requires complete examination.
    #[arg(long, global = true, conflicts_with = "limit")]
    pub exists: bool,
    #[arg(long, global = true, conflicts_with_all = ["exists", "limit"])]
    pub count: bool,
    /// Select the population counted by an explicit count demand.
    #[arg(long, global = true, value_enum)]
    pub count_unit: Option<CountBy>,
    #[arg(long, global = true, value_enum)]
    pub order: Option<Order>,
    /// Continue a deterministic page under its original query/source guards.
    #[arg(long, global = true)]
    pub cursor: Option<String>,
    /// Select result detail fields; source identity and coverage remain mandatory.
    #[arg(long, global = true, value_delimiter = ',')]
    pub result_fields: Vec<String>,
    /// Group returned witnesses, without claiming exhaustive group counts.
    #[arg(long, global=true, value_parser=["source","kind","name"])]
    pub group_by: Option<String>,
    /// Explain the effective validated query without invoking its providers.
    #[arg(long, global = true)]
    pub explain: bool,
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Mode {
    /// Search declarations in an explicitly selected complete descriptor set.
    Schema {
        query: String,
        #[arg(long)]
        descriptor: PathBuf,
        #[arg(long = "kind")]
        kinds: Vec<String>,
        #[arg(long)]
        owner: Option<String>,
    },
    /// Discover selected files and their source kinds.
    Files,
    /// Search native text, preserving byte and encoding policies.
    Text {
        pattern: String,
        #[arg(long)]
        literal: bool,
        #[arg(long)]
        ignore_case: bool,
        #[arg(long)]
        word: bool,
        #[arg(long)]
        line: bool,
        #[arg(long)]
        multiline: bool,
        #[arg(long)]
        invert: bool,
        #[arg(long)]
        context: Option<usize>,
    },
    /// Match a native Rust syntax pattern or explicit Tree-sitter extraction query.
    Syntax {
        pattern: Option<String>,
        #[arg(long, conflicts_with = "pattern")]
        tree_sitter: Option<String>,
        #[arg(long, conflicts_with_all = ["pattern", "tree_sitter"])]
        rule: Option<PathBuf>,
    },
    /// Find written declarations or explicitly selected semantic workspace symbols.
    Symbol {
        name: String,
        #[arg(long)]
        semantic: bool,
        #[arg(long, value_enum)]
        matching: Option<Matching>,
        #[arg(long = "kind")]
        kinds: Vec<String>,
        #[arg(long)]
        module: Option<String>,
        #[arg(long)]
        visibility: Option<String>,
        /// Written signature filter; no inferred type equivalence is claimed.
        #[arg(long)]
        signature: Option<String>,
    },
    /// Search an explicitly acquired native API artifact without acquisition.
    Api {
        name: String,
        #[arg(long)]
        artifact: Option<PathBuf>,
        #[arg(long)]
        dependency: Option<String>,
        #[arg(long)]
        input: Option<String>,
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        bound: Option<String>,
    },
    /// Select a structured field using a typed JSON Pointer.
    Structured {
        pointer: String,
        #[arg(long)]
        equals: Option<String>,
    },
    /// Investigate native Git history without changing the checkout.
    History {
        pattern: Option<String>,
        #[arg(long = "at-revision")]
        revision: Option<String>,
        #[arg(long, value_enum)]
        operation: Option<HistoryMode>,
    },
}

#[derive(Debug, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
#[command(group(clap::ArgGroup::new("inspection_operand").args(["result", "source"]).required(true)))]
pub(super) struct Inspect {
    #[command(subcommand)]
    action: Option<evidence::Action>,
    /// Original result reference: returned JSON object or its portable encoded form.
    #[arg(long, conflicts_with = "source")]
    pub result: Option<String>,
    /// Direct known source: no preliminary search is required.
    #[arg(long, conflicts_with = "result")]
    pub source: Option<PathBuf>,
    #[arg(long, conflicts_with = "revision")]
    pub index: bool,
    #[arg(long)]
    pub revision: Option<String>,
    #[arg(long)]
    pub range_start: Option<usize>,
    #[arg(long)]
    pub range_end: Option<usize>,
    #[arg(long, conflicts_with_all = ["head", "tail", "range_start", "range_end"])]
    pub lines: Option<String>,
    #[arg(long, conflicts_with_all = ["tail", "range_start", "range_end"])]
    pub head: Option<usize>,
    #[arg(long, conflicts_with_all = ["range_start", "range_end"])]
    pub tail: Option<usize>,
    /// Expand one selected semantic fact through its original native provider.
    #[arg(long, value_enum)]
    pub view: Option<View>,
    #[arg(long)]
    pub select: Option<usize>,
    #[arg(long)]
    pub predicate: Option<String>,
    /// Passively associate an original type-sizes acquisition with --view layout.
    #[arg(long, requires = "view")]
    pub compiler_layout: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum View {
    Declaration,
    /// Inspect fallible native layout under the selected semantic world and target.
    Layout,
    References,
    Calls,
    Implementations,
    Macro,
    Tests,
    Candidates,
    Hints,
    Hint,
    Candidate,
    Predicate,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(untagged)]
pub(super) enum Inspection {
    Schema(Box<crate::schema::Report>),
    Source(Box<query::SearchResult>),
    Semantic(Box<query::rust::SemanticInspection>),
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(untagged)]
pub(super) enum Preview {
    Syntax(Box<crate::source_proposal::ChangeProposal>),
    Semantic(Box<query::rust::SemanticInspection>),
}

#[derive(Debug, Args)]
#[command(subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub(super) struct Propose {
    #[command(subcommand)]
    action: Option<proposal::Action>,
    /// Original result reference as returned JSON or its portable encoded form.
    #[arg(long)]
    pub result: Option<String>,
    /// Exact replacement preview; source and index remain untouched.
    #[arg(long)]
    pub replacement: Option<String>,
    /// Native semantic rename preview, requiring the original provider context.
    #[arg(long, conflicts_with = "replacement")]
    pub rename: Option<String>,
    #[arg(long, conflicts_with_all = ["rename", "replacement", "ssr", "assist"])]
    pub file_rename: Option<PathBuf>,
    #[arg(long, conflicts_with_all = ["rename", "replacement", "assist"])]
    pub ssr: Option<String>,
    #[arg(long, conflicts_with_all = ["rename", "replacement"])]
    pub assist: Option<usize>,
    #[arg(long)]
    pub assists: bool,
}

impl Search {
    fn resolve(self, root: &Path) -> Result<query::config::Resolved, query::Error> {
        let mut overrides = json!({});
        if !self.paths.is_empty() {
            overrides["sources"]["paths"] = json!(
                self.paths
                    .into_iter()
                    .map(query::NativePath::from)
                    .collect::<Vec<_>>()
            );
        }
        if !self.package.is_empty() {
            overrides["sources"]["packages"] = json!(self.package);
        }
        if !self.glob.is_empty() {
            overrides["sources"]["globs"] = json!(self.glob);
        }
        if !self.exclude.is_empty() {
            overrides["sources"]["exclude"] = json!(self.exclude);
        }
        if !self.types.is_empty() {
            overrides["sources"]["types"] = json!(self.types);
        }
        if self.tracked {
            overrides["sources"]["tracked"] = json!(true);
        }
        if self.untracked {
            overrides["sources"]["tracked"] = json!(false);
        }
        if self.changed {
            overrides["sources"]["changed"] = json!(true);
        }
        if self.dependencies {
            overrides["sources"]["include_dependencies"] = json!(true);
        }
        if self.include_ignored {
            overrides["sources"]["include_ignored"] = json!(true);
        }
        if self.hidden {
            overrides["sources"]["include_hidden"] = json!(true);
        }
        if self.generated {
            overrides["sources"]["include_generated"] = json!(true);
        }
        if self.index {
            overrides["sources"]["kind"] = json!({"kind":"index"});
        }
        if let Some(revision) = self.revision {
            overrides["sources"]["kind"] = json!({"kind":"revision","revision":revision});
        }
        if let Some(limit) = self.limit {
            overrides["answer"]["limit"] = json!(limit);
        }
        if self.all {
            overrides["answer"]["limit"] = Value::Null;
        }
        if self.exists {
            overrides["answer"]["kind"] = json!("exists");
            overrides["answer"]["limit"] = json!(1);
        }
        if self.count {
            overrides["answer"]["kind"] = json!("count");
            overrides["answer"]["limit"] = Value::Null;
        }
        if let Some(unit) = self.count_unit {
            overrides["answer"]["count_unit"] = json!(match unit {
                CountBy::Matches => "matches",
                CountBy::Files => "files",
                CountBy::Lines => "lines",
                CountBy::Entities => "entities",
            });
        }
        if let Some(order) = self.order {
            overrides["answer"]["order"] = json!(match order {
                Order::Arrival => "arrival",
                Order::Deterministic => "stable",
            });
        }
        if let Some(cursor) = self.cursor {
            overrides["answer"]["cursor"] = json!(cursor);
        }
        if !self.result_fields.is_empty() {
            overrides["presentation"]["fields"] = json!(self.result_fields);
        }
        if let Some(group) = self.group_by {
            overrides["presentation"]["group_by"] = json!(group);
        }
        if let Some(mode) = self.mode {
            let predicate = match mode {
                Mode::Schema {
                    query,
                    descriptor,
                    kinds,
                    owner,
                } => {
                    json!({"kind":"schema","query":query,"descriptor":query::NativePath::from(descriptor),"kinds":kinds,"owner":owner})
                }
                Mode::Files => json!({"kind":"files"}),
                Mode::Text {
                    pattern,
                    literal,
                    ignore_case,
                    word,
                    line,
                    multiline,
                    invert,
                    context,
                } => {
                    if let Some(context) = context {
                        overrides["presentation"]["context"] = json!(context);
                    }
                    let mut value = json!({"kind":"text","patterns":[pattern]});
                    for (key, selected) in [
                        ("literal", literal),
                        ("word", word),
                        ("whole_line", line),
                        ("multiline", multiline),
                        ("invert", invert),
                    ] {
                        if selected {
                            value[key] = json!(true);
                        }
                    }
                    if ignore_case {
                        value["case"] = json!("insensitive");
                    }
                    value
                }
                Mode::Syntax {
                    pattern,
                    tree_sitter,
                    rule,
                } => {
                    let mut value = json!({"kind":"syntax"});
                    if let Some(pattern) = pattern {
                        value["pattern"] = json!(pattern);
                    }
                    if let Some(tree_sitter) = tree_sitter {
                        value["tree_sitter"] = json!(tree_sitter);
                    }
                    if let Some(rule) = rule {
                        value["rule"] = serde_json::from_slice(&std::fs::read(root.join(rule))?)?;
                    }
                    value
                }
                Mode::Symbol {
                    name,
                    semantic,
                    matching,
                    kinds,
                    module,
                    visibility,
                    signature,
                } => {
                    let mut value = json!({"kind":"symbol","name":name});
                    if semantic {
                        value["provider"] = json!("semantic");
                    }
                    if let Some(matching) = matching {
                        value["matching"] = json!(match matching {
                            Matching::Contains => "contains",
                            Matching::Exact => "exact",
                            Matching::Prefix => "prefix",
                            Matching::Fuzzy => "fuzzy",
                        });
                    }
                    if !kinds.is_empty() {
                        value["kinds"] = json!(kinds);
                    }
                    for (key, selected) in [
                        ("module", module),
                        ("visibility", visibility),
                        ("signature", signature),
                    ] {
                        if let Some(selected) = selected {
                            value[key] = json!(selected);
                        }
                    }
                    value
                }
                Mode::Api {
                    name,
                    artifact,
                    dependency,
                    input,
                    output,
                    bound,
                } => {
                    let mut value = json!({"kind":"api","query":name});
                    if let Some(artifact) = artifact {
                        value["artifact"] = json!(artifact);
                    }
                    for (key, selected) in [
                        ("dependency", dependency),
                        ("input", input),
                        ("output", output),
                        ("bound", bound),
                    ] {
                        if let Some(selected) = selected {
                            value[key] = json!(selected);
                        }
                    }
                    value
                }
                Mode::Structured { pointer, equals } => {
                    let mut value = json!({"kind":"structured","pointer":pointer});
                    if let Some(value_text) = equals {
                        value["equals"] = serde_json::from_str::<Value>(&value_text)?;
                    }
                    value
                }
                Mode::History {
                    pattern,
                    revision,
                    operation,
                } => {
                    let mut value = json!({"kind":"history"});
                    if let Some(pattern) = pattern {
                        value["query"] = json!(pattern);
                    }
                    if let Some(revision) = revision {
                        value["revision"] = json!(revision);
                    }
                    if let Some(operation) = operation {
                        value["operation"] = json!(match operation {
                            HistoryMode::Log => "log",
                            HistoryMode::Blame => "blame",
                            HistoryMode::Pickaxe => "pickaxe",
                            HistoryMode::PickaxeLines => "pickaxe_lines",
                        });
                    }
                    value
                }
            };
            overrides["predicate"] = predicate;
        } else if self.request.is_none() && self.query.is_none() {
            return Err(query::Error::Invalid(
                "select a search mode, --request or --query".into(),
            ));
        }
        query::config::explain(
            root,
            self.config.as_deref(),
            self.query.as_deref(),
            self.request.as_deref(),
            overrides,
        )
    }
}

pub(super) fn search(
    options: Search,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    let explain = options.explain;
    let mut resolved = options.resolve(&root)?;
    if let Some(budget) = output.presentation.budget {
        resolved.request.presentation.budget = Some(budget);
        resolved
            .origins
            .insert("presentation.budget".into(), "explicit CLI --budget".into());
    }
    resolved.request.validate()?;
    if explain {
        return output
            .emit_page(
                Envelope::response(
                    "search explain",
                    "effective validated query; no providers invoked",
                    resolved,
                ),
                crate::envelope::DetailView::default(),
            )
            .map_err(HostError::from);
    }
    let request = resolved.request;
    let rt = super::runtime()?;
    let handle = rt.handle().clone();
    let cancellation = CancellationToken::new();
    let task_token = cancellation.clone();
    let mut output = output.clone();
    if output.format == OutputFormat::Ndjson && output.presentation.fields.is_some() {
        return Err(query::Error::Invalid("--fields selects envelope payload fields; use --result-fields to select streamed search detail".into()).into());
    }
    output.presentation = query::presentation::envelope_view(&output.presentation);
    let (sender, receiver) = tokio::sync::mpsc::channel(8);
    let mut task = crate::source_command::batch::spawn_cpu(task_token, move |token| {
        execute(
            root,
            request,
            output,
            token,
            handle,
            delivery::Frames(sender),
        )
    });
    let result = rt.block_on(async {
        let delivery = tokio::select! {
            result = delivery::stdout(receiver) => result,
            signal = tokio::signal::ctrl_c() => signal.and_then(|()| Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"search output interrupted"))),
        };
        // Dropping the receiver releases a blocked producer before awaiting its actual exit.
        if delivery.is_err() { task.cancel(); }
        let settled = task.settle().await?;
        Ok::<_,crate::source_command::SourceCommandError>(match delivery {
            Ok(())=>settled,
            // Output is unavailable; rendering another stdout error would block
            // on the same consumer. Exit only after the producer has settled.
            Err(error)=>Ok(if error.kind()==std::io::ErrorKind::Interrupted {130}else{1}),
        })
    })?;
    result.map_err(HostError::from)
}

fn execute(
    root: PathBuf,
    request: query::Request,
    output: OutputOptions,
    token: CancellationToken,
    handle: tokio::runtime::Handle,
    frames: delivery::Frames,
) -> Result<i32, query::Error> {
    let streaming = output.format == OutputFormat::Ndjson;
    let mut writer = query::stream::StreamWriter::new(frames, token.clone());
    let mut report = query::Report::default();
    if !streaming {
        report.query = Some(query::PerformedQuery {
            root: root.clone().into(),
            request: request.clone(),
        });
    }
    let mut groups = std::collections::BTreeMap::<String, Vec<u64>>::new();
    let mut ordinal = 0u64;
    let observed = std::sync::Mutex::new(query::Summary::default());
    let progress = indicatif::ProgressBar::with_draw_target(
        None,
        if std::io::stderr().is_terminal() {
            indicatif::ProgressDrawTarget::stderr()
        } else {
            indicatif::ProgressDrawTarget::hidden()
        },
    );
    progress.set_message("searching selected sources");
    let mut emit = |mut event: query::Event| -> Result<(), query::Error> {
        if let query::Event::Progress {
            examined_files,
            examined_bytes,
            matches,
        } = event
        {
            let mut facts = observed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            facts.examined_files = facts.examined_files.max(examined_files);
            facts.examined_bytes = facts.examined_bytes.max(examined_bytes);
            facts.matches = facts.matches.max(matches);
            progress.set_message(format!(
                "search: {examined_files} files, {examined_bytes} bytes, {matches} matches"
            ));
            progress.tick();
            return Ok(());
        }
        if let query::Event::Result(row) = &mut event {
            if let Some(key) = query::presentation::project(row, &request.presentation) {
                groups.entry(key).or_default().push(ordinal);
            }
            ordinal = ordinal.saturating_add(1);
            let mut facts = observed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            facts.emitted = ordinal;
            facts.matches = facts.matches.max(ordinal);
        }
        if let query::Event::Terminal(summary) = &mut event {
            summary.groups = groups.clone();
            summary.omitted_fields = query::presentation::omitted(&request.presentation);
            progress.finish_and_clear();
        }
        if streaming {
            return writer.emit(&event);
        }
        match event {
            query::Event::Source(source) => report.sources.push(*source),
            query::Event::Result(row) => report.results.push(*row),
            query::Event::Diagnostic {
                phase,
                code,
                message,
                source,
            } => report.diagnostics.push(query::QueryDiagnostic {
                phase,
                code,
                message,
                source,
            }),
            query::Event::NativeFailure { failure } => report.native_failures.push(*failure),
            query::Event::Terminal(summary) => report.summary = summary,
            query::Event::Begin { .. } => {}
            query::Event::Progress { .. } => {}
        }
        Ok(())
    };
    emit(query::Event::Begin {
        invocation_id: output.invocation_id.as_str().into(),
        root: root.clone().into(),
        request: Box::new(request.clone()),
    })?;
    let performed = if query::composition::required(&request.predicate) {
        query::composition::run(&root, &request, &token, &mut emit, &handle)
    } else {
        match &request.predicate {
            query::Predicate::Schema(schema) => {
                crate::schema::search::run(&root, &request, schema, &token, &mut emit)
            }
            query::Predicate::Api(_)
            | query::Predicate::Symbol(query::SymbolQuery {
                provider: query::DiscoveryProvider::Semantic | query::DiscoveryProvider::Api,
                ..
            }) => handle.block_on(query::rust::global_search(
                &root, &request, &token, &mut emit,
            )),
            query::Predicate::History(history) => handle.block_on(query::providers::history(
                &root, &request, history, &token, &mut emit,
            )),
            _ => query::providers::run(&root, &request, &token, &mut emit),
        }
    };
    let summary = match performed {
        Ok(summary) => summary,
        Err(error) => {
            if matches!(error, query::Error::Delivery(_)) && token.is_cancelled() {
                return Err(error);
            }
            let disposition = error.disposition();
            if let Some(failure) = error.native_refusal() {
                emit(query::Event::NativeFailure {
                    failure: Box::new(failure),
                })?;
            }
            let unavailable = disposition == query::FailureDisposition::Unavailable;
            let cancelled =
                disposition == query::FailureDisposition::Cancelled || token.is_cancelled();
            emit(query::Event::Diagnostic {
                phase: error.phase(),
                code: if unavailable {
                    "investigation.unavailable"
                } else {
                    "investigation.provider"
                }
                .into(),
                message: error.to_string(),
                source: None,
            })?;
            let mut summary = observed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if unavailable {
                summary.unknown = summary.unknown.saturating_add(1);
            } else if !cancelled {
                summary.failures = summary.failures.saturating_add(1);
            }
            summary.complete = false;
            summary.cancellation_requested = cancelled;
            summary.stop = if cancelled {
                query::StopReason::Cancelled
            } else {
                query::StopReason::Complete
            };
            summary.count = query::Count::AtLeast(summary.matches);
            summary
        }
    };
    // One positive witness settles existence without proving every alternative.
    // Unknown coverage and original provider diagnostics remain in the response.
    let exists_proven = request.answer.kind == query::AnswerKind::Exists && summary.matches > 0;
    let failed = summary.failures > 0
        || (summary.unknown > 0 && !exists_proven)
        || summary.cancellation_requested;
    let cancelled = summary.cancellation_requested;
    emit(query::Event::Terminal(summary))?;
    drop(emit);
    if streaming {
        return Ok(if cancelled { 130 } else { i32::from(failed) });
    }
    // The streaming writer never owns full-answer retention, and releases stdout before rendering.
    let mut frames = writer.finish();
    let complete = report.summary.complete;
    let subject = if cancelled {
        crate::envelope::SubjectState::Cancelled
    } else if report.summary.failures > 0 || (report.summary.unknown > 0 && !exists_proven) {
        crate::envelope::SubjectState::Unknown
    } else if complete && report.summary.matches == 0 {
        crate::envelope::SubjectState::NoMatch
    } else {
        crate::envelope::SubjectState::Available
    };
    let mut response = Envelope::response("search", "requested investigation work settled", report);
    response.subject_state = subject;
    if cancelled {
        response.outcome = CommandOutcome::Interrupted;
    } else if failed {
        response.outcome = CommandOutcome::Failure;
    }
    response.observation.completeness = if complete {
        Completeness::Complete
    } else {
        Completeness::Partial
    };
    response.observation.scope = "selected_query_source_universe".into();
    response.observation.required = false;
    response.invocation_id = output.invocation_id;
    if output.format == OutputFormat::Human && output.presentation.fields.is_none() {
        use std::io::Write as _;
        let rendered = human::report(&response)?;
        frames.write_all(rendered.as_bytes())?;
        return Ok(response.exit_code());
    }
    let projection = crate::envelope::Presentation {
        fields: output.presentation.fields,
        budget: None,
    };
    crate::envelope::write(frames, response, output.format, &projection).map_err(query::Error::from)
}

pub(super) fn inspect(
    options: Inspect,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    if options.compiler_layout.is_some() && !matches!(options.view, Some(View::Layout)) {
        return Err(
            query::Error::Invalid("--compiler-layout requires --view layout".into()).into(),
        );
    }
    if let Some(action) = options.action {
        return evidence::inspect(action, selected, output);
    }
    let mut root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    let mut locator = if let Some(reference) = options.result {
        query::ResultReference::decode(&reference)?.locator
    } else {
        let path = options
            .source
            .ok_or_else(|| query::Error::Invalid("inspect requires --result or --source".into()))?;
        let source = if options.index {
            query::SourceKind::Index
        } else if let Some(revision) = options.revision {
            query::SourceKind::Revision { revision }
        } else {
            query::SourceKind::Working
        };
        let range = match (options.range_start, options.range_end) {
            (Some(start), Some(end)) if end >= start => Some(query::MatchRange {
                byte_start: start,
                byte_end: end,
                line_start: 1,
                line_end: 1,
            }),
            (None, None) => None,
            _ => {
                return Err(query::Error::Invalid(
                    "byte range requires ordered --range-start and --range-end".into(),
                )
                .into());
            }
        };
        query::Locator::Source {
            path: path.into(),
            source,
            range,
            guard: None,
        }
    };
    if selected.is_none()
        && let query::Locator::Source {
            guard:
                Some(query::SourceGuard {
                    root: Some(original),
                    ..
                }),
            ..
        } = &locator
    {
        root = original.to_path_buf();
    }
    if matches!(locator, query::Locator::Evidence { .. }) {
        return evidence::inspect_locator(&locator, selected, output);
    }
    if options.lines.is_some() || options.head.is_some() || options.tail.is_some() {
        let query::Locator::Source {
            path,
            source,
            range,
            guard,
        } = &mut locator
        else {
            return Err(query::Error::Invalid(
                "line/head/tail selection requires a source locator".into(),
            )
            .into());
        };
        let request = match source {
            query::SourceKind::Working => crate::source_read::SourceRequest::Working {
                path: root.join(path.to_path_buf()),
            },
            query::SourceKind::Index => crate::source_read::SourceRequest::Index {
                repository: root.clone(),
                path: path.to_path_buf(),
            },
            query::SourceKind::Revision { revision } => crate::source_read::SourceRequest::Blob {
                repository: root.clone(),
                path: path.to_path_buf(),
                revision: revision.clone(),
            },
        };
        let input = query::providers::acquire(&root, &request)?;
        let mut starts = vec![0];
        for (index, byte) in input.bytes.iter().enumerate() {
            if *byte == b'\n' && index + 1 < input.bytes.len() {
                starts.push(index + 1);
            }
        }
        let (first, last) = if let Some(lines) = options.lines {
            let (first, last) = lines.split_once(':').unwrap_or((&lines, &lines));
            let parse = |s: &str| {
                s.parse::<usize>()
                    .map_err(|_| query::Error::Invalid("lines must be positive FIRST:LAST".into()))
            };
            (parse(first)?, parse(last)?)
        } else if let Some(head) = options.head {
            (1, head.min(starts.len()))
        } else {
            (
                starts
                    .len()
                    .saturating_sub(options.tail.unwrap_or(0))
                    .saturating_add(1),
                starts.len(),
            )
        };
        if first == 0 || last < first || first > starts.len() {
            return Err(query::Error::Invalid(
                "selected line range is empty or outside the source".into(),
            )
            .into());
        }
        *range = Some(query::MatchRange {
            byte_start: starts[first - 1],
            byte_end: starts.get(last).copied().unwrap_or(input.bytes.len()),
            line_start: first,
            line_end: last.min(starts.len()),
        });
        if guard.is_none() {
            *guard = input.guard;
        }
    }
    if options.view.is_some() && matches!(locator, query::Locator::Source { .. }) {
        locator = super::runtime()?.block_on(query::rust::source_semantic(
            &root,
            &locator,
            &CancellationToken::new(),
        ))?;
    }
    if options.view.is_some() && !matches!(locator, query::Locator::Semantic { .. }) {
        return Err(query::Error::Unavailable(
            "requested native semantic view is unavailable for this evidence context".into(),
        )
        .into());
    }
    let mut compiler_evidence = None;
    let result = if matches!(locator, query::Locator::Semantic { .. }) {
        use crate::semantic::Kind;
        let kind = match options.view.unwrap_or(View::Declaration) {
            View::Declaration => Kind::Explain,
            View::Layout => Kind::Layout,
            View::References => Kind::Refs,
            View::Calls => Kind::Calls,
            View::Implementations => Kind::Implementations,
            View::Macro => Kind::Macro,
            View::Tests => Kind::Tests,
            View::Candidates => Kind::Candidates,
            View::Hints => Kind::Hints,
            View::Hint => Kind::Hint {
                index: options.select.ok_or_else(|| {
                    query::Error::Invalid("hint resolution requires --select".into())
                })?,
            },
            View::Candidate => Kind::Candidate {
                index: options.select.ok_or_else(|| {
                    query::Error::Invalid("candidate resolution requires --select".into())
                })?,
            },
            View::Predicate => Kind::Predicate {
                text: options.predicate.ok_or_else(|| {
                    query::Error::Invalid("predicate inspection requires --predicate".into())
                })?,
            },
        };
        let mut inspected =
            super::runtime()?.block_on(query::rust::semantic_view(&root, &locator, kind))?;
        if let Some(selector) = &options.compiler_layout {
            let state =
                crate::paths::CheckoutState::resolve(&root, &std::env::vars_os().collect())?;
            let original =
                crate::compiler_investigation::artifact::observe(&root, &state, selector)?;
            inspected.compiler_layout = Some(crate::semantic::associate_layout(
                &inspected.native,
                &original.payload,
            ));
            compiler_evidence = Some(original);
        }
        Inspection::Semantic(Box::new(inspected))
    } else if matches!(locator, query::Locator::Schema { .. }) {
        Inspection::Schema(Box::new(crate::schema::search::inspect(&root, &locator)?))
    } else if matches!(locator, query::Locator::Api { .. }) {
        Inspection::Source(Box::new(query::rust::inspect_api(&root, &locator)?))
    } else {
        Inspection::Source(Box::new(query::providers::inspect(&root, &locator)?))
    };
    let mut response =
        Envelope::response("inspect", "addressed original source inspection", result);
    if let Some(original) = compiler_evidence {
        response.providers.extend(original.providers);
        response.artifacts.extend(original.artifacts);
        response.next.extend(original.next);
    }
    if matches!(options.view, Some(View::Tests))
        && let Inspection::Semantic(answer) = &response.payload
    {
        response.next = super::code::semantic::test_routes(
            &root,
            &answer.native,
            &answer.native.session.configuration,
        );
    }
    if output.format == OutputFormat::Human
        && output.presentation.fields.is_none()
        && output.presentation.budget.is_none()
    {
        let text = match &response.payload {
            Inspection::Schema(answer) => {
                serde_json::to_string_pretty(answer).map_err(query::Error::from)?
            }
            Inspection::Source(row) => {
                let path = row
                    .reference
                    .as_ref()
                    .and_then(|reference| match &reference.locator {
                        query::Locator::Source { path, .. } => Some(path),
                        _ => None,
                    });
                human::row(row, path)?
            }
            Inspection::Semantic(answer) => {
                serde_json::to_string_pretty(answer).map_err(query::Error::from)?
            }
        };
        return super::source_emit(output, response, move |_| text);
    }
    output
        .emit_page(response, crate::envelope::DetailView::default())
        .map_err(HostError::from)
}

pub(super) fn propose(
    options: Propose,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    if let Some(action) = options.action {
        return proposal::dispatch(action, selected, output);
    }
    let mut root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    let mut reference = query::ResultReference::decode(&options.result.ok_or_else(|| {
        query::Error::Invalid("propose requires --result or a native preview operation".into())
    })?)?;
    if selected.is_none()
        && let query::Locator::Source {
            guard:
                Some(query::SourceGuard {
                    root: Some(original),
                    ..
                }),
            ..
        } = &reference.locator
    {
        root = original.to_path_buf();
    }
    if options.rename.is_some()
        || options.file_rename.is_some()
        || options.ssr.is_some()
        || options.assist.is_some()
        || options.assists
    {
        reference.locator = super::runtime()?.block_on(query::rust::source_semantic(
            &root,
            &reference.locator,
            &CancellationToken::new(),
        ))?;
    }
    let proposal = if matches!(reference.locator, query::Locator::Semantic { .. }) {
        use crate::semantic::Kind;
        let kind = if let Some(name) = options.rename {
            Kind::Rename { name }
        } else if let Some(destination) = options.file_rename {
            Kind::FileRename { destination }
        } else if let Some(rule) = options.ssr {
            Kind::Ssr { rule }
        } else if let Some(index) = options.assist {
            Kind::Assist { index }
        } else if options.assists {
            Kind::Assists
        } else {
            return Err(
                query::Error::Invalid("select a native semantic preview operation".into()).into(),
            );
        };
        Preview::Semantic(Box::new(
            super::runtime()?.block_on(query::rust::semantic_view(
                &root,
                &reference.locator,
                kind,
            ))?,
        ))
    } else {
        Preview::Syntax(Box::new(query::rust::propose(
            &root,
            &reference.locator,
            options.replacement,
            options.rename,
        )?))
    };
    output
        .emit_page(
            Envelope::response(
                "propose",
                "checked immutable preview; no source or index writes",
                proposal,
            ),
            crate::envelope::DetailView::default(),
        )
        .map_err(HostError::from)
}
