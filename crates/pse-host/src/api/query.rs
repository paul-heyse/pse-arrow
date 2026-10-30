// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed declaration queries, usable public paths and authored examples over one checked artifact.
use super::{ApiError, Artifact};
use rustdoc_types::{Crate, Id, Item, ItemEnum, Type};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Structural kind selected without interpreting rendered documentation.
#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, clap::ValueEnum, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A free or associated function.
    Function,
    /// A struct declaration.
    Struct,
    /// An enum declaration.
    Enum,
    /// A union declaration.
    Union,
    /// A trait or trait alias.
    Trait,
    /// A type alias.
    TypeAlias,
    /// A constant or associated constant.
    Constant,
    /// A static declaration.
    Static,
    /// A module declaration.
    Module,
    /// A declarative or procedural macro.
    Macro,
    /// Another rustdoc declaration, retained without guessing its kind.
    Other,
}
fn kind(item: &Item) -> Kind {
    match &item.inner {
        ItemEnum::Function(_) => Kind::Function,
        ItemEnum::Struct(_) => Kind::Struct,
        ItemEnum::Enum(_) => Kind::Enum,
        ItemEnum::Union(_) => Kind::Union,
        ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => Kind::Trait,
        ItemEnum::TypeAlias(_) => Kind::TypeAlias,
        ItemEnum::Constant { .. } | ItemEnum::AssocConst { .. } => Kind::Constant,
        ItemEnum::Static(_) => Kind::Static,
        ItemEnum::Module(_) => Kind::Module,
        ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => Kind::Macro,
        _ => Kind::Other,
    }
}
/// One selected query; unspecified qualification filters impose no negative claim.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Query {
    /// Exact public crate-relative path.
    pub path: Option<String>,
    /// Case-insensitive name/docs substring.
    pub search: Option<String>,
    /// Exact structural kind.
    pub kind: Option<Kind>,
    /// Input type spelling within structured function inputs.
    pub input: Option<String>,
    /// Output type spelling within a structured function output.
    pub output: Option<String>,
    /// Declared trait path substring, never a trait-solver query.
    pub trait_name: Option<String>,
    /// Require declared unsafe qualification.
    pub unsafe_only: bool,
    /// Require declared async qualification.
    pub async_only: bool,
    /// Require declared const qualification.
    pub const_only: bool,
    /// Expand authored docs on each returned candidate.
    pub docs: bool,
    /// Expand authored examples, retaining hidden setup.
    pub examples: bool,
    /// Facade artifact identities traversed before this declaration; binds continuations.
    pub surface_identity: Option<String>,
}
/// Trait provenance is a declaration fact, independent of applicability.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct TraitProvenance {
    /// Trait declaration path when rustdoc exposes one.
    pub path: String,
    /// Required import when a public path is known.
    pub required_import: Option<String>,
    /// Negative impl declaration.
    pub negative: bool,
    /// Compiler-synthetic impl declaration.
    pub synthetic: bool,
    /// Blanket implementation type spelling when present.
    pub blanket: Option<String>,
}
/// Original source location; absent spans remain unknown.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Location {
    /// Original rustdoc source path.
    pub filename: std::path::PathBuf,
    /// Original one-based line and zero-based column.
    pub begin: (usize, usize),
    /// Original exclusive end location.
    pub end: (usize, usize),
}
/// Authored example expectation; reading never executes the example.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Expectation {
    /// Ordinary Rust example expectation.
    CompileAndRun,
    /// `no_run` example expectation.
    CompileOnly,
    /// `compile_fail` example expectation, without attributing a particular failure.
    CompileFail,
    /// `should_panic` example expectation.
    ShouldPanic,
    /// Globally or target-conditionally ignored example.
    Ignored,
}
/// One CommonMark code fence with independently preserved hidden setup and expectation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Example {
    /// Authored rustdoc expectation, independent of any execution result.
    pub expectation: Expectation,
    /// Original fence flags, including edition and conditional ignore settings.
    pub attributes: Vec<String>,
    /// Original code-fence content.
    pub authored: String,
    /// Visible code with rustdoc hidden setup omitted.
    pub displayed: String,
    /// Code with hidden setup revealed and escaped hashes normalized.
    pub expanded: String,
    /// Hidden setup lines in original order.
    pub hidden_setup: Vec<String>,
    /// Fence start in original documentation UTF-8 bytes.
    pub documentation_byte_start: usize,
    /// Fence end in original documentation UTF-8 bytes.
    pub documentation_byte_end: usize,
    /// Reading this example always reports `not_run`.
    pub execution: &'static str,
}
/// One whole producer page unit. The original typed declaration remains in the addressed artifact.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Candidate {
    /// Reachable import or associated-item path in this addressed crate.
    pub public_path: String,
    /// Artifact-local rustdoc item identity; never compared across artifacts.
    pub item_id: u32,
    /// Typed declaration kind.
    pub kind: Kind,
    /// Compact declaration presentation, with explicit omissions.
    pub signature: String,
    /// Whether this presentation includes every signature component.
    pub signature_complete: bool,
    /// Expanded authored docs only when requested.
    pub docs: Option<String>,
    /// Declared rustdoc attributes; absent cfg information stays unknown.
    pub attributes: Vec<String>,
    /// Authored deprecation release and note, when rustdoc supplies them.
    pub deprecation: Option<Deprecation>,
    /// Original definition location when supplied by rustdoc.
    pub source: Option<Location>,
    /// Required trait import and implementation facts where known.
    pub trait_provenance: Option<TraitProvenance>,
    /// Whether rustdoc explicitly stripped part of this declaration.
    pub incomplete_declaration: bool,
    /// Documentation extraction supplies no ordinary-build availability verdict.
    pub ordinary_build_availability: &'static str,
    /// Whole authored examples only when explicitly requested.
    pub examples: Vec<Example>,
}
/// Retained declaration deprecation, independent of its availability in a chosen build.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Deprecation {
    /// Upstream's declared release marker.
    pub since: Option<String>,
    /// Upstream's authored migration note.
    pub note: Option<String>,
}
/// Page cursor binds both immutable evidence and exact query/extraction policy.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u32,
    artifact: String,
    query: String,
    offset: usize,
}
/// Whole-candidate page with examined scope distinct from its returned size.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Page {
    /// Exact locked source-qualified package.
    pub package_id: String,
    /// Source/target/toolchain/features/extraction identity.
    pub context_key: String,
    /// Original immutable rustdoc bytes.
    pub artifact_sha256: String,
    /// Addressed retained original JSON.
    pub artifact: std::path::PathBuf,
    /// Actual documentation compilation target.
    pub target: String,
    /// Cargo's selected documentation-root unit features.
    pub features: BTreeSet<String>,
    /// Broader Cargo metadata package features, separate from selected doc units.
    pub metadata_features: BTreeSet<String>,
    /// Checked rustdoc JSON schema version.
    pub rustdoc_format: u32,
    /// All typed items examined, independently of this page size.
    pub examined_items: usize,
    /// All discovered declaration paths examined.
    pub examined_public_paths: usize,
    /// Exact number of matches within this artifact and query policy.
    pub matched_candidates: usize,
    /// Matches not returned on this page.
    pub omitted_candidates: usize,
    /// Complete selected candidate units.
    pub candidates: Vec<Candidate>,
    /// Bound continuation for this exact artifact and query.
    pub continuation: Option<String>,
    /// Known extraction/presentation limits retained on every page.
    pub limitations: Vec<&'static str>,
    /// Addressed facade artifacts traversed to the defining declaration artifact.
    pub facades: Vec<Facade>,
}
/// One native public reexport edge, preserving both immutable artifact contexts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Facade {
    /// Source-qualified package containing the reexport.
    pub package_id: String,
    /// Exact source/feature/target/toolchain context of that facade.
    pub context_key: String,
    /// Original facade JSON location.
    pub artifact: std::path::PathBuf,
    /// Original facade JSON content identity.
    pub artifact_sha256: String,
    /// Public prefix selected through the facade.
    pub public_prefix: String,
    /// Defining artifact's corresponding usable public prefix.
    pub definition_prefix: String,
}
/// Typed external reexport lead; Cargo supplies its exact source-qualified package edge.
#[derive(Debug)]
pub(crate) struct ExternalUse {
    pub(crate) public_prefix: String,
    pub(crate) crate_name: String,
    pub(crate) definition_path: Vec<String>,
    pub(crate) suffix: String,
}
/// Typed availability shares one schema between unavailable preparation and addressed pages.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub enum Report {
    /// A finalized artifact in the exact currently observed context was selected.
    Prepared {
        /// Whole producer-selected page over that artifact.
        page: Page,
    },
    /// No finalized artifact exists for the selected context; no build was attempted.
    Unprepared {
        /// Original locked package query.
        package_query: String,
        /// Explicit target request, or the observed host default when absent.
        requested_target: Option<String>,
    },
}
#[derive(Clone)]
struct Address {
    path: String,
    id: Id,
    trait_provenance: Option<TraitProvenance>,
}
fn public(item: &Item) -> bool {
    matches!(item.visibility, rustdoc_types::Visibility::Public)
}
fn walk(
    document: &Crate,
    id: Id,
    path: String,
    ancestors: &mut BTreeSet<Id>,
    output: &mut BTreeMap<String, Id>,
) {
    if !ancestors.insert(id) {
        return;
    }
    if let Some(item) = document.index.get(&id) {
        if let ItemEnum::Use(import) = &item.inner {
            if let Some(target) = import.id
                && document.index.contains_key(&target)
            {
                walk(document, target, path, ancestors, output);
            } else {
                output.insert(path, id);
            }
        } else {
            output.entry(path.clone()).or_insert(id);
            if let ItemEnum::Module(module) = &item.inner {
                for child in &module.items {
                    let Some(child_item) = document.index.get(child) else {
                        continue;
                    };
                    if !public(child_item) {
                        continue;
                    }
                    if let ItemEnum::Use(import) = &child_item.inner {
                        if import.is_glob {
                            if let Some(target) = import.id
                                && document.index.contains_key(&target)
                            {
                                walk(document, target, path.clone(), ancestors, output);
                            } else {
                                output.insert(format!("{path}::*"), *child);
                            }
                        } else {
                            walk(
                                document,
                                *child,
                                format!("{path}::{}", import.name),
                                ancestors,
                                output,
                            );
                        }
                    } else if let Some(name) = &child_item.name {
                        walk(
                            document,
                            *child,
                            format!("{path}::{name}"),
                            ancestors,
                            output,
                        );
                    }
                }
            }
        }
    }
    ancestors.remove(&id);
}
pub(crate) fn external_uses(document: &Crate, path: Option<&str>) -> Vec<ExternalUse> {
    let root = document
        .index
        .get(&document.root)
        .and_then(|item| item.name.as_deref())
        .unwrap_or("");
    let selected = path.map(|path| {
        if path == root || path.starts_with(&format!("{root}::")) {
            path.to_owned()
        } else {
            format!("{root}::{path}")
        }
    });
    let mut result = Vec::new();
    for address in addresses(document) {
        let Some(Item {
            inner: ItemEnum::Use(import),
            ..
        }) = document.index.get(&address.id)
        else {
            continue;
        };
        let Some(summary) = import.id.and_then(|id| document.paths.get(&id)) else {
            continue;
        };
        let Some(external) = document.external_crates.get(&summary.crate_id) else {
            continue;
        };
        let prefix = address.path.strip_suffix("::*").unwrap_or(&address.path);
        let suffix = match &selected {
            Some(selected) if selected == prefix => String::new(),
            Some(selected) => match selected.strip_prefix(&format!("{prefix}::")) {
                Some(suffix) => suffix.into(),
                None => continue,
            },
            None => String::new(),
        };
        result.push(ExternalUse {
            public_prefix: prefix.into(),
            crate_name: external.name.clone(),
            definition_path: summary.path.clone(),
            suffix,
        });
    }
    result.sort_by_key(|route| std::cmp::Reverse(route.public_prefix.len()));
    result
}
pub(crate) fn definition_prefix(document: &Crate, canonical: &[String]) -> Option<String> {
    let root = document
        .index
        .get(&document.root)
        .and_then(|item| item.name.as_deref())?;
    if canonical.len() == 1 {
        return Some(root.into());
    }
    let ids: BTreeSet<_> = document
        .paths
        .iter()
        .filter_map(|(id, summary)| (summary.path == canonical).then_some(*id))
        .collect();
    addresses(document)
        .into_iter()
        .filter(|address| ids.contains(&address.id))
        .map(|address| address.path)
        .min_by_key(String::len)
}
fn addresses(document: &Crate) -> Vec<Address> {
    let root = document
        .index
        .get(&document.root)
        .and_then(|item| item.name.clone())
        .unwrap_or_default();
    let mut paths = BTreeMap::new();
    walk(
        document,
        document.root,
        root,
        &mut BTreeSet::new(),
        &mut paths,
    );
    let mut result = Vec::new();
    for (path, id) in &paths {
        result.push(Address {
            path: path.clone(),
            id: *id,
            trait_provenance: None,
        });
        let Some(item) = document.index.get(id) else {
            continue;
        };
        let impls = match &item.inner {
            ItemEnum::Struct(value) => &value.impls,
            ItemEnum::Enum(value) => &value.impls,
            ItemEnum::Union(value) => &value.impls,
            ItemEnum::Primitive(value) => &value.impls,
            ItemEnum::Trait(value) => {
                for associated in &value.items {
                    if let Some(associated_item) = document.index.get(associated)
                        && let Some(name) = &associated_item.name
                    {
                        result.push(Address {
                            path: format!("{path}::{name}"),
                            id: *associated,
                            trait_provenance: None,
                        });
                    }
                }
                continue;
            }
            _ => continue,
        };
        for impl_id in impls {
            let Some(Item {
                inner: ItemEnum::Impl(implementation),
                ..
            }) = document.index.get(impl_id)
            else {
                continue;
            };
            let provenance = implementation.trait_.as_ref().map(|trait_| {
                let public_path = paths
                    .iter()
                    .find_map(|(path, id)| (*id == trait_.id).then(|| path.clone()))
                    .or_else(|| {
                        document
                            .paths
                            .get(&trait_.id)
                            .map(|summary| summary.path.join("::"))
                    });
                TraitProvenance {
                    path: public_path.clone().unwrap_or_else(|| trait_.path.clone()),
                    required_import: public_path,
                    negative: implementation.is_negative,
                    synthetic: implementation.is_synthetic,
                    blanket: implementation.blanket_impl.as_ref().map(type_name),
                }
            });
            let mut associated = implementation.items.clone();
            if !implementation.is_negative
                && let Some(trait_) = &implementation.trait_
                && let Some(Item {
                    inner: ItemEnum::Trait(value),
                    ..
                }) = document.index.get(&trait_.id)
            {
                for default in &value.items {
                    if let Some(item) = document.index.get(default)
                        && matches!(&item.inner, ItemEnum::Function(function) if function.has_body)
                        && !associated
                            .iter()
                            .filter_map(|id| document.index.get(id))
                            .any(|override_| override_.name == item.name)
                    {
                        associated.push(*default);
                    }
                }
            }
            for associated_id in associated {
                if let Some(associated_item) = document.index.get(&associated_id)
                    && let Some(name) = &associated_item.name
                    && (provenance.is_some() || public(associated_item))
                {
                    result.push(Address {
                        path: format!("{path}::{name}"),
                        id: associated_id,
                        trait_provenance: provenance.clone(),
                    });
                }
            }
        }
    }
    result.sort_by(|left, right| (&left.path, left.id.0).cmp(&(&right.path, right.id.0)));
    result.dedup_by(|left, right| {
        left.path == right.path
            && left.id == right.id
            && left.trait_provenance.as_ref().map(|value| &value.path)
                == right.trait_provenance.as_ref().map(|value| &value.path)
    });
    result
}
#[cfg(test)]
pub(super) fn test_public_paths(document: &Crate) -> Vec<(String, Option<String>)> {
    addresses(document)
        .into_iter()
        .map(|address| {
            (
                address.path,
                address
                    .trait_provenance
                    .and_then(|value| value.required_import),
            )
        })
        .collect()
}
/// Presentation of a typed Rust type. Unhandled complexity is explicit, never a guessed type.
pub fn type_name(value: &Type) -> String {
    match value {
        Type::ResolvedPath(path) => format!(
            "{}{}",
            path.path,
            path.args.as_deref().map(arguments).unwrap_or_default()
        ),
        Type::Generic(value) | Type::Primitive(value) => value.clone(),
        Type::Infer => "_".into(),
        Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => format!(
            "&{}{}{}",
            lifetime
                .as_ref()
                .map(|value| format!("{value} "))
                .unwrap_or_default(),
            if *is_mutable { "mut " } else { "" },
            type_name(type_)
        ),
        Type::RawPointer { is_mutable, type_ } => format!(
            "*{} {}",
            if *is_mutable { "mut" } else { "const" },
            type_name(type_)
        ),
        Type::Slice(value) => format!("[{}]", type_name(value)),
        Type::Array { type_, len } => format!("[{}; {len}]", type_name(type_)),
        Type::Tuple(values) => format!(
            "({}{})",
            values.iter().map(type_name).collect::<Vec<_>>().join(", "),
            if values.len() == 1 { "," } else { "" }
        ),
        Type::QualifiedPath {
            name,
            args,
            self_type,
            trait_,
        } => format!(
            "<{}{}>::{name}{}",
            type_name(self_type),
            trait_
                .as_ref()
                .map(|value| format!(
                    " as {}{}",
                    value.path,
                    value.args.as_deref().map(arguments).unwrap_or_default()
                ))
                .unwrap_or_default(),
            args.as_deref().map(arguments).unwrap_or_default()
        ),
        Type::FunctionPointer(function) => format!(
            "fn({}){} [full function pointer declaration in artifact]",
            function
                .sig
                .inputs
                .iter()
                .map(|(_, value)| type_name(value))
                .collect::<Vec<_>>()
                .join(", "),
            function
                .sig
                .output
                .as_ref()
                .map(|value| format!(" -> {}", type_name(value)))
                .unwrap_or_default()
        ),
        Type::ImplTrait(_) => "impl <bounds in artifact>".into(),
        Type::DynTrait(_) => "dyn <bounds in artifact>".into(),
        Type::Pat { type_, .. } => format!("{} is <pattern in artifact>", type_name(type_)),
    }
}
fn arguments(value: &rustdoc_types::GenericArgs) -> String {
    match value {
        rustdoc_types::GenericArgs::AngleBracketed { args, constraints } => {
            let mut values: Vec<_> = args
                .iter()
                .map(|arg| match arg {
                    rustdoc_types::GenericArg::Lifetime(value) => value.clone(),
                    rustdoc_types::GenericArg::Type(value) => type_name(value),
                    rustdoc_types::GenericArg::Const(value) => value.expr.clone(),
                    rustdoc_types::GenericArg::Infer => "_".into(),
                })
                .collect();
            if !constraints.is_empty() {
                values.push("<associated constraints in artifact>".into());
            }
            if values.is_empty() {
                String::new()
            } else {
                format!("<{}>", values.join(", "))
            }
        }
        rustdoc_types::GenericArgs::Parenthesized { inputs, output } => format!(
            "({}){}",
            inputs.iter().map(type_name).collect::<Vec<_>>().join(", "),
            output
                .as_ref()
                .map(|value| format!(" -> {}", type_name(value)))
                .unwrap_or_default()
        ),
        _ => "<arguments in artifact>".into(),
    }
}
fn signature(item: &Item) -> (String, bool) {
    let name = item.name.as_deref().unwrap_or("<unnamed>");
    if let ItemEnum::Function(function) = &item.inner {
        let generics = if function.generics.params.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                function
                    .generics
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let mut signature = format!(
            "{}{}{}fn {name}{generics}({}){}",
            if function.header.is_const {
                "const "
            } else {
                ""
            },
            if function.header.is_async {
                "async "
            } else {
                ""
            },
            if function.header.is_unsafe {
                "unsafe "
            } else {
                ""
            },
            function
                .sig
                .inputs
                .iter()
                .map(|(name, type_)| format!("{name}: {}", type_name(type_)))
                .collect::<Vec<_>>()
                .join(", "),
            function
                .sig
                .output
                .as_ref()
                .map(|value| format!(" -> {}", type_name(value)))
                .unwrap_or_default()
        );
        let complete = function.generics.params.is_empty()
            && function.generics.where_predicates.is_empty()
            && !signature.contains("in artifact")
            && !function.sig.is_c_variadic
            && matches!(function.header.abi, rustdoc_types::Abi::Rust);
        if !complete {
            signature.push_str(" [full declaration and constraints in artifact]");
        }
        (signature, complete)
    } else {
        (format!("{:?} {name}", kind(item)), false)
    }
}
fn matches(item: &Item, address: &Address, query: &Query) -> bool {
    if let Some(kind_) = query.kind
        && kind(item) != kind_
    {
        return false;
    }
    if let Some(path) = &query.path
        && address.path != *path
        && !address.path.ends_with(&format!("::{path}"))
    {
        return false;
    }
    if let Some(search) = &query.search {
        let search = search.to_lowercase();
        if !address.path.to_lowercase().contains(&search)
            && !item
                .docs
                .as_ref()
                .is_some_and(|docs| docs.to_lowercase().contains(&search))
        {
            return false;
        }
    }
    if let Some(trait_) = &query.trait_name
        && !address
            .trait_provenance
            .as_ref()
            .is_some_and(|value| value.path.contains(trait_))
    {
        return false;
    }
    if query.input.is_some()
        || query.output.is_some()
        || query.unsafe_only
        || query.async_only
        || query.const_only
    {
        let ItemEnum::Function(function) = &item.inner else {
            return false;
        };
        if query.unsafe_only && !function.header.is_unsafe
            || query.async_only && !function.header.is_async
            || query.const_only && !function.header.is_const
        {
            return false;
        }
        if let Some(input) = &query.input
            && !function
                .sig
                .inputs
                .iter()
                .any(|(_, value)| type_name(value).contains(input))
        {
            return false;
        }
        if let Some(output) = &query.output
            && !function
                .sig
                .output
                .as_ref()
                .is_some_and(|value| type_name(value).contains(output))
        {
            return false;
        }
    }
    true
}
/// Extract authored Rust fences without rustdoc transformation or code execution.
pub fn examples(docs: &str) -> Vec<Example> {
    use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
    let mut result = Vec::new();
    let mut active: Option<(Vec<String>, usize, String)> = None;
    for (event, range) in pulldown_cmark::Parser::new(docs).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(block)) => {
                let info = match block {
                    CodeBlockKind::Fenced(info) => info.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let attrs: Vec<String> = info
                    .split([',', ' ', '\t'])
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .collect();
                let rust = attrs.first().is_none_or(|value| {
                    matches!(
                        value.as_str(),
                        "rust" | "no_run" | "compile_fail" | "should_panic" | "ignore"
                    ) || value.starts_with("edition")
                        || value.starts_with("ignore-")
                        || value.starts_with("E0")
                });
                if rust {
                    active = Some((attrs, range.start, String::new()));
                }
            }
            Event::Text(text) => {
                if let Some((_, _, text_)) = &mut active {
                    text_.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((attributes, start, authored)) = active.take() {
                    let expectation = if attributes
                        .iter()
                        .any(|value| value == "ignore" || value.starts_with("ignore-"))
                    {
                        Expectation::Ignored
                    } else if attributes.iter().any(|value| value == "compile_fail") {
                        Expectation::CompileFail
                    } else if attributes.iter().any(|value| value == "no_run") {
                        Expectation::CompileOnly
                    } else if attributes.iter().any(|value| value == "should_panic") {
                        Expectation::ShouldPanic
                    } else {
                        Expectation::CompileAndRun
                    };
                    let mut displayed = String::new();
                    let mut expanded = String::new();
                    let mut hidden_setup = Vec::new();
                    for line in authored.lines() {
                        let trimmed = line.trim_start();
                        if trimmed == "#" || trimmed.starts_with("# ") {
                            let hidden = trimmed.strip_prefix("# ").unwrap_or("");
                            hidden_setup.push(hidden.into());
                            expanded.push_str(hidden);
                            expanded.push('\n');
                        } else {
                            let line = if trimmed.starts_with("##") {
                                line.replacen("##", "#", 1)
                            } else {
                                line.to_owned()
                            };
                            displayed.push_str(&line);
                            displayed.push('\n');
                            expanded.push_str(&line);
                            expanded.push('\n');
                        }
                    }
                    result.push(Example {
                        expectation,
                        attributes,
                        authored,
                        displayed,
                        expanded,
                        hidden_setup,
                        documentation_byte_start: start,
                        documentation_byte_end: range.end,
                        execution: "not_run",
                    });
                }
            }
            _ => {}
        }
    }
    result
}
/// Select whole candidates against one immutable artifact; never cut recursive declarations.
pub fn inspect(
    artifact: &Artifact,
    query: &Query,
    limit: usize,
    cursor: Option<&str>,
) -> Result<Page, ApiError> {
    use base64::Engine as _;
    if limit == 0 {
        return Err(super::invalid("API page limit must be positive"));
    }
    let query_digest = crate::probe::digest(&serde_json::to_vec(query)?);
    let offset = if let Some(cursor) = cursor {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|_| super::invalid("invalid API continuation encoding"))?;
        let cursor: Cursor = serde_json::from_slice(&bytes)?;
        if cursor.version != 1 || cursor.artifact != artifact.sha256 || cursor.query != query_digest
        {
            return Err(super::invalid(
                "API continuation source or query changed; select the retained original artifact or restart",
            ));
        }
        cursor.offset
    } else {
        0
    };
    let addresses = addresses(&artifact.document);
    let examined_public_paths = addresses.len();
    let selected: Vec<_> = addresses
        .into_iter()
        .filter(|address| {
            artifact
                .document
                .index
                .get(&address.id)
                .is_some_and(|item| matches(item, address, query))
        })
        .collect();
    if offset > selected.len() {
        return Err(super::invalid(
            "API continuation offset is outside its selected artifact",
        ));
    }
    let total = selected.len();
    let candidates=selected.into_iter().skip(offset).take(limit).filter_map(|address|artifact.document.index.get(&address.id).map(|item|{
        let (signature,signature_complete)=signature(item);
        let incomplete_declaration=match &item.inner{ItemEnum::Module(value)=>value.is_stripped,ItemEnum::Struct(value)=>match &value.kind{rustdoc_types::StructKind::Plain{has_stripped_fields,..}=>*has_stripped_fields,rustdoc_types::StructKind::Tuple(fields)=>fields.iter().any(Option::is_none),rustdoc_types::StructKind::Unit=>false},ItemEnum::Union(value)=>value.has_stripped_fields,ItemEnum::Enum(value)=>value.has_stripped_variants,ItemEnum::Use(_)=>true,_=>false};
        Candidate{public_path:address.path,item_id:item.id.0,kind:kind(item),signature,signature_complete,docs:query.docs.then(||item.docs.clone()).flatten(),attributes:item.attrs.iter().map(|attr|format!("{attr:?}")).collect(),deprecation:item.deprecation.as_ref().map(|value|Deprecation{since:value.since.clone(),note:value.note.clone()}),source:item.span.as_ref().map(|span|Location{filename:span.filename.clone(),begin:span.begin,end:span.end}),trait_provenance:address.trait_provenance,incomplete_declaration,ordinary_build_availability:"unknown; rustdoc cfg(doc) is a documentation context",examples:if query.examples{item.docs.as_deref().map(examples).unwrap_or_default()}else{Vec::new()}}
    })).collect::<Vec<_>>();
    let next = offset + candidates.len();
    let continuation = if next < total {
        Some(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Cursor {
                version: 1,
                artifact: artifact.sha256.clone(),
                query: query_digest,
                offset: next,
            })?),
        )
    } else {
        None
    };
    Ok(Page {
        package_id: artifact.context.package_id.clone(),
        context_key: artifact.context.key()?,
        artifact_sha256: artifact.sha256.clone(),
        artifact: artifact.path.clone(),
        target: artifact.context.target.clone(),
        features: artifact.context.features.clone(),
        metadata_features: artifact.context.metadata_features.clone(),
        rustdoc_format: artifact.context.rustdoc_format,
        examined_items: artifact.document.index.len(),
        examined_public_paths,
        matched_candidates: total,
        omitted_candidates: total.saturating_sub(candidates.len()),
        candidates,
        continuation,
        facades: Vec::new(),
        limitations: vec![
            "declarations and trait implementations do not establish arbitrary trait applicability or compiler success",
            "external definitions absent from this artifact require explicitly preparing the addressed crate",
            "signatures marked incomplete retain complete typed declarations in the original artifact",
            "known rustdoc attributes are declaration facts; absent ordinary-build cfg gates remain unknown",
        ],
    })
}
