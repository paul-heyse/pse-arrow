// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Description of the owning clap parser, shared by native host and xtask via `#[path]`.
//! Descriptions are discovery evidence; the original parser remains execution authority.
use clap::{Arg, Command};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;

/// Whether a source actually supplied the requested domain.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    /// Source supplied the requested information.
    Available,
    /// The selected source is absent or cannot currently be inspected.
    Unavailable,
    /// The parser/source does not expose this semantic information.
    Unknown,
}
/// Whether enumeration describes the accepted domain, independently of pagination.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Exhaustiveness {
    /// Complete finite domain under the stated source/context.
    Exhaustive,
    /// Suggestions from an open domain; other accepted values may exist.
    Open,
    /// Domain coverage could not be established.
    Unknown,
}
/// Source freshness is never inferred from an empty result.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    /// Owning parser/declaration in this executing generation.
    CurrentGeneration,
    /// Explicit retained snapshot, without asserting current-source equivalence.
    RetainedSnapshot,
    /// Source owner established a stale association.
    Stale,
    /// Currentness has not been established.
    Unknown,
}
/// Attribution accompanying every page, including unknown/unavailable pages.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Source {
    /// Actual parser or provider owner.
    pub owner: String,
    /// Actual generation/snapshot identity, when supplied by the owner.
    pub generation: Option<String>,
    /// Established freshness only.
    pub freshness: Freshness,
}
impl Source {
    /// The actual in-memory authoritative parser.
    pub fn parser() -> Self {
        Self {
            owner: "clap parser".into(),
            generation: None,
            freshness: Freshness::CurrentGeneration,
        }
    }
}
/// Public metadata may be absent even though the parser enforces a rule.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "availability", content = "value", rename_all = "snake_case")]
pub enum Information<T> {
    /// Read from the authoritative declaration or a documented public introspector.
    Available(T),
    /// The owner does not expose a stable introspection contract for this fact.
    Unavailable {
        /// Why absence cannot be interpreted as an empty rule set.
        reason: String,
    },
}
/// OS-native values retain their original encoded bytes when UTF-8 cannot represent them.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "encoding", content = "value", rename_all = "snake_case")]
pub enum OsValue {
    /// Exact UTF-8 text.
    Utf8(String),
    /// Rust's platform-specific OS encoded bytes; not a negotiated source/LSP encoding.
    PlatformBytes(Vec<u8>),
}
impl OsValue {
    fn capture(value: &OsStr) -> Self {
        value.to_str().map_or_else(
            || Self::PlatformBytes(value.as_encoded_bytes().to_vec()),
            |value| Self::Utf8(value.into()),
        )
    }
}
/// One parser/provider suggestion; canonical values and aliases remain distinct.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Candidate {
    /// Canonical spelling suggested to callers.
    pub value: String,
    /// Actual parser-owned aliases, when exposed.
    pub aliases: Vec<String>,
    /// Owner-provided explanation, if any.
    pub help: Option<String>,
    /// Hidden values may remain accepted even when not advertised.
    pub hidden: bool,
}
/// Captured candidate domain before prefix/pagination; no serialized byte budgets.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CandidateSet {
    /// Actual domain source/context.
    pub source: Source,
    /// Availability of the selected source.
    pub availability: Availability,
    /// Finite/open/unknown accepted-domain coverage.
    pub exhaustiveness: Exhaustiveness,
    /// Actual enumerated candidates, never fabricated from a custom parser's debug text.
    pub values: Vec<Candidate>,
    /// Unavailable or intentionally unexposed facts.
    pub limitations: Vec<String>,
}
impl CandidateSet {
    /// An absent provider is distinct from an available finite empty domain.
    pub fn unavailable(owner: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            source: Source {
                owner: owner.into(),
                generation: None,
                freshness: Freshness::Unknown,
            },
            availability: Availability::Unavailable,
            exhaustiveness: Exhaustiveness::Unknown,
            values: Vec::new(),
            limitations: vec![detail.into()],
        }
    }
}
/// Record-based paging shared by command descriptions and argument candidates.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Page<T> {
    /// Actual owning source, even when the requested domain is unavailable.
    pub source: Source,
    /// Whether the query found an available source.
    pub availability: Availability,
    /// Coverage of the source's domain, independently of the visible page length.
    pub exhaustiveness: Exhaustiveness,
    /// Returned records in stable owner-defined order.
    pub records: Vec<T>,
    /// Prefix applied before paging.
    pub prefix: String,
    /// Zero-based offset within the matching source snapshot.
    pub offset: usize,
    /// Requested record limit; null means all matching records.
    pub limit: Option<usize>,
    /// Next offset in the same source snapshot, if more known records remain.
    pub continuation: Option<usize>,
    /// Number of matching enumerated records, not a count of an unknown/open universe.
    pub known_matches: Option<usize>,
    /// Coverage/currentness limitations; missing metadata is explicit.
    pub limitations: Vec<String>,
}
/// Prefix/page options. Providers receive no demand to scan unrelated domains.
#[derive(Clone, Debug, Default)]
pub struct Window {
    /// Literal canonical prefix, not a glob or parser replacement.
    pub prefix: String,
    /// Offset within the source snapshot.
    pub offset: usize,
    /// Maximum records, independently of JSON byte size.
    pub limit: Option<usize>,
}
fn page<T>(
    source: Source,
    availability: Availability,
    exhaustiveness: Exhaustiveness,
    values: Vec<T>,
    window: &Window,
    limitations: Vec<String>,
) -> Page<T> {
    let count = values.len();
    let length = window.limit.unwrap_or(usize::MAX);
    let records = values
        .into_iter()
        .skip(window.offset)
        .take(length)
        .collect::<Vec<_>>();
    let end = window.offset.saturating_add(records.len());
    let continuation = (end < count && !records.is_empty()).then_some(end);
    Page {
        source,
        availability,
        exhaustiveness,
        records,
        prefix: window.prefix.clone(),
        offset: window.offset,
        limit: window.limit,
        continuation,
        known_matches: (availability == Availability::Available).then_some(count),
        limitations,
    }
}
fn invalid_window<T>(window: &Window) -> Option<Page<T>> {
    (window.limit == Some(0)).then(|| {
        page(
            Source::parser(),
            Availability::Unavailable,
            Exhaustiveness::Unknown,
            Vec::new(),
            window,
            vec!["page limit must be positive; no domain provider was evaluated".into()],
        )
    })
}
/// Descriptive extensions add associations/examples, never copies of validation rules.
#[derive(Clone, Debug, Default)]
pub struct CommandMetadata {
    /// Literal example argument vectors, excluding the executable.
    pub examples: Vec<Vec<String>>,
    /// Serialization-schema IDs supplied by the response owner.
    pub output_schemas: Vec<String>,
    /// Exact invocation route, supplied by the prepared owner.
    pub invocation_route: Vec<String>,
    /// Features required by that owner association.
    pub features: Vec<String>,
}
impl clap::builder::CommandExt for CommandMetadata {}
/// Association with a lazy domain provider; it does not alter the argument's parser.
#[derive(Clone, Debug)]
pub struct CandidateProvider {
    /// Provider identity, resolved by the application owner.
    pub name: String,
}
impl clap::builder::ArgExt for CandidateProvider {}
/// Whether declared defaults are safe to expose; environment values are never read here.
#[derive(Clone, Debug)]
pub struct SecretDefaults;
impl clap::builder::ArgExt for SecretDefaults {}
/// Actual public argument metadata, including explicit introspection limitations.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Argument {
    /// Owning clap argument ID, independent of its long flag.
    pub id: String,
    /// Primary long option, when present.
    pub long: Option<String>,
    /// Primary short option, when present.
    pub short: Option<char>,
    /// Positional index, when applicable.
    pub index: Option<usize>,
    /// Canonical parser action, not guessed Rust/domain semantics.
    pub action: String,
    /// Owner help text.
    pub help: Option<String>,
    /// Owner long help text, if separately supplied.
    pub long_help: Option<String>,
    /// Minimum values consumed by an occurrence.
    pub minimum_values: usize,
    /// Maximum values, with null for an unbounded occurrence.
    pub maximum_values: Option<usize>,
    /// Unconditional required flag; group/conditional semantics remain independent.
    pub required: bool,
    /// Whether only this explicitly supplied argument may be present.
    pub exclusive: bool,
    /// Actual global propagation declaration.
    pub global: bool,
    /// Trailing literal argv consumption.
    pub trailing_var_arg: bool,
    /// Values require the explicit `--` separator.
    pub last: bool,
    /// Environment mirror name; actual captured values are not exposed here.
    pub environment: Option<OsValue>,
    /// Exact declared defaults, or explicit sensitivity refusal.
    pub defaults: Information<Vec<OsValue>>,
    /// Canonical accepted finite values when the parser exposes them.
    pub possible_values: Information<Vec<Candidate>>,
    /// Lazy external suggestions; not proof of parser acceptance.
    pub candidate_provider: Option<String>,
    /// Conflicts exposed by clap, expanding argument-group targets through its own API.
    pub conflicts: Vec<String>,
    /// Requires/conditional rules are not publicly inspectable in this pinned clap release.
    pub requires: Information<Vec<String>>,
    /// Overrides/conditional-default rules are not publicly inspectable here.
    pub conditional_rules: Information<Vec<String>>,
}
/// Group cardinality is kept separate from its member arguments' required flags.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Group {
    /// Original clap group ID.
    pub id: String,
    /// Original member IDs.
    pub members: Vec<String>,
    /// Whether at least one member is required.
    pub required: bool,
    /// Whether several members may be present together.
    pub multiple: bool,
    /// Group requirements/conflicts have no public getter in this release.
    pub relationships: Information<Vec<String>>,
}
/// A command description; it cannot reconstruct private typed parsers or validators.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Description {
    /// Canonical path below the executable root.
    pub path: Vec<String>,
    /// Actual clap command name.
    pub name: String,
    /// Owner-provided description, if present.
    pub about: Option<String>,
    /// Native short help/usage, hiding captured environment values and sensitive defaults.
    pub short_help: String,
    /// Native long help with the same explicit sensitivity policy.
    pub long_help: String,
    /// Public arguments from the actual initialized parser.
    pub arguments: Vec<Argument>,
    /// Actual group cardinality declarations.
    pub groups: Vec<Group>,
    /// Immediate visible child command names; no provider hydration.
    pub subcommands: Vec<String>,
    /// Descriptive examples from the owner extension.
    pub examples: Vec<Vec<String>>,
    /// Owner-supplied output schemas; absence is explicit.
    pub output_schemas: Information<Vec<String>>,
    /// Owner-supplied prepared invocation route; never dispatch authority.
    pub invocation_route: Information<Vec<String>>,
    /// Feature requirements supplied by that owner.
    pub features: Vec<String>,
    /// Introspection limitations that remain enforced by the owning parser.
    pub limitations: Vec<String>,
}
fn unavailable<T>(reason: &str) -> Information<T> {
    Information::Unavailable {
        reason: reason.into(),
    }
}
fn possible(arg: &Arg) -> Option<Vec<Candidate>> {
    // SetTrue/Count parsers carry internal typed defaults, but a zero-value flag
    // does not accept those values as literal argv tokens.
    if arg
        .get_num_args()
        .is_some_and(|range| !range.takes_values())
    {
        return Some(Vec::new());
    }
    arg.get_value_parser().possible_values().map(|values| {
        values
            .map(|value| Candidate {
                value: value.get_name().into(),
                aliases: value
                    .get_name_and_aliases()
                    .filter(|alias| *alias != value.get_name())
                    .map(str::to_owned)
                    .collect(),
                help: value.get_help().map(ToString::to_string),
                hidden: value.is_hide_set(),
            })
            .collect()
    })
}
fn describe_argument(command: &Command, arg: &Arg) -> Argument {
    let range = arg.get_num_args();
    Argument {
        id: arg.get_id().to_string(),
        long: arg.get_long().map(str::to_owned),
        short: arg.get_short(),
        index: arg.get_index(),
        action: format!("{:?}", arg.get_action()),
        help: arg.get_help().map(ToString::to_string),
        long_help: arg.get_long_help().map(ToString::to_string),
        minimum_values: range.map_or(0, |range| range.min_values()),
        maximum_values: range.map_or(Some(0), |range| {
            (range.max_values() != usize::MAX).then_some(range.max_values())
        }),
        required: arg.is_required_set(),
        exclusive: arg.is_exclusive_set(),
        global: arg.is_global_set(),
        trailing_var_arg: arg.is_trailing_var_arg_set(),
        last: arg.is_last_set(),
        environment: arg.get_env().map(OsValue::capture),
        defaults: if arg.get::<SecretDefaults>().is_some() {
            unavailable("owner classified declared defaults as secret")
        } else {
            Information::Available(
                arg.get_default_values()
                    .iter()
                    .map(|value| OsValue::capture(value))
                    .collect(),
            )
        },
        possible_values: possible(arg).map_or_else(
            || unavailable("typed parser does not expose a finite possible-values domain"),
            Information::Available,
        ),
        candidate_provider: arg
            .get::<CandidateProvider>()
            .map(|provider| provider.name.clone()),
        conflicts: command
            .get_arg_conflicts_with(arg)
            .into_iter()
            .map(|arg| arg.get_id().to_string())
            .collect(),
        requires: unavailable(
            "clap 4.6.7 public introspection does not expose requires/conditional requirements",
        ),
        conditional_rules: unavailable(
            "opaque typed validation, overrides and conditional defaults remain owned by the parser",
        ),
    }
}
fn describe(mut command: Command, path: Vec<String>) -> Description {
    command = command.mut_args(|arg| {
        if arg.get::<SecretDefaults>().is_some() {
            arg.hide_default_value(true).hide_env_values(true)
        } else {
            arg.hide_env_values(true)
        }
    });
    let short_help = command.render_help().to_string();
    let long_help = command.render_long_help().to_string();
    let metadata = command
        .get::<CommandMetadata>()
        .cloned()
        .unwrap_or_default();
    Description{path,name:command.get_name().into(),about:command.get_about().map(ToString::to_string),short_help,long_help,arguments:command.get_arguments().filter(|arg|!arg.is_hide_set()).map(|arg|describe_argument(&command,arg)).collect(),groups:command.get_groups().map(|group|Group{id:group.get_id().to_string(),members:group.get_args().map(ToString::to_string).collect(),required:group.is_required_set(),multiple:group.clone().is_multiple(),relationships:unavailable("group requires/conflicts have no public introspection getter in clap 4.6.7")}).collect(),subcommands:command.get_subcommands().filter(|child|!child.is_hide_set()).map(|child|child.get_name().into()).collect(),examples:metadata.examples,output_schemas:if metadata.output_schemas.is_empty(){unavailable("response owner did not declare a serialization-schema association")}else{Information::Available(metadata.output_schemas)},invocation_route:if metadata.invocation_route.is_empty(){unavailable("prepared owner invocation route is not attached to this parser")}else{Information::Available(metadata.invocation_route)},features:metadata.features,limitations:vec!["public descriptions do not reconstruct opaque typed parsers or private conditional validation".into()]}
}
fn scoped(mut root: Command, path: &[String]) -> Option<(Command, Vec<String>)> {
    let mut canonical = Vec::new();
    root = root.disable_help_subcommand(true);
    for name in path {
        // render_help builds only this node, unlike build()'s intentional full recursion.
        let _ = root.render_help();
        let child = root.find_subcommand(name)?.clone();
        if child.is_hide_set() {
            return None;
        }
        canonical.push(child.get_name().to_owned());
        root = child;
    }
    let _ = root.render_help();
    Some((root, canonical))
}
/// Command-path export. Scoped traversal does not initialize unrelated deferred subtrees.
pub fn commands(
    root: Command,
    path: &[String],
    recursive: bool,
    window: &Window,
) -> Page<Description> {
    if let Some(refusal) = invalid_window(window) {
        return refusal;
    }
    let Some((mut selected, path)) = scoped(root, path) else {
        return page(
            Source::parser(),
            Availability::Unavailable,
            Exhaustiveness::Unknown,
            Vec::new(),
            window,
            vec!["selected visible command path is not declared by this parser".into()],
        );
    };
    if recursive {
        selected.build();
    }
    fn collect(
        command: Command,
        path: Vec<String>,
        recursive: bool,
        values: &mut Vec<Description>,
    ) {
        let children = if recursive {
            command
                .get_subcommands()
                .filter(|child| !child.is_hide_set())
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        values.push(describe(command, path.clone()));
        for child in children {
            let mut next = path.clone();
            next.push(child.get_name().into());
            collect(child, next, true, values);
        }
    }
    let mut values = Vec::new();
    collect(selected, path, recursive, &mut values);
    values.retain(|value| value.path.join(" ").starts_with(&window.prefix));
    values.sort_by(|left, right| left.path.cmp(&right.path));
    page(
        Source::parser(),
        Availability::Available,
        Exhaustiveness::Exhaustive,
        values,
        window,
        Vec::new(),
    )
}
/// Resolve one argument's suggestions lazily. The provider is never called for other domains.
pub fn candidates<E>(
    root: Command,
    path: &[String],
    argument: &str,
    window: &Window,
    mut provider: impl FnMut(&str) -> Result<CandidateSet, E>,
) -> Result<Page<Candidate>, E> {
    if let Some(refusal) = invalid_window(window) {
        return Ok(refusal);
    }
    let Some((command, _)) = scoped(root, path) else {
        return Ok(page(
            Source::parser(),
            Availability::Unavailable,
            Exhaustiveness::Unknown,
            Vec::new(),
            window,
            vec!["selected visible command path is not declared".into()],
        ));
    };
    let Some(arg) = command
        .get_arguments()
        .find(|arg| arg.get_id() == argument || arg.get_long() == Some(argument))
    else {
        return Ok(page(
            Source::parser(),
            Availability::Unavailable,
            Exhaustiveness::Unknown,
            Vec::new(),
            window,
            vec!["selected argument is not declared by this command".into()],
        ));
    };
    let mut set = if let Some(values) = possible(arg) {
        CandidateSet {
            source: Source::parser(),
            availability: Availability::Available,
            exhaustiveness: Exhaustiveness::Exhaustive,
            values,
            limitations: Vec::new(),
        }
    } else if let Some(association) = arg.get::<CandidateProvider>() {
        provider(&association.name)?
    } else {
        CandidateSet{source:Source::parser(),availability:Availability::Unknown,exhaustiveness:Exhaustiveness::Unknown,values:Vec::new(),limitations:vec!["custom parser exposes no finite domain or associated provider; absence is not an empty accepted domain".into()]}
    };
    set.values
        .retain(|value| value.value.starts_with(&window.prefix));
    set.values
        .sort_by(|left, right| left.value.cmp(&right.value));
    Ok(page(
        set.source,
        set.availability,
        set.exhaustiveness,
        set.values,
        window,
        set.limitations,
    ))
}
