// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only lifecycle interpretation and native plan-scope aggregation.
#![allow(
    clippy::print_stdout,
    clippy::disallowed_types,
    reason = "xtask lifecycle queries own structured terminal output and dynamic tooling errors"
)]
use super::{
    headings::markdown_headings,
    metadata::{self, Document, Value},
    site,
};
use anyhow::{Context, Result, ensure};
use clap::{Args, ValueEnum};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

type Fields = BTreeMap<String, Value>;
type SourceReader<'a> = Option<&'a dyn Fn(&str) -> Result<Vec<u8>>>;
/// Sole lifecycle interpretation and native source-selector declaration.
#[derive(Clone, Debug, Deserialize)]
pub struct Policy {
    /// Supported declaration version.
    pub version: u32,
    /// Declared metadata topics.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Global document metadata defaults.
    #[serde(default)]
    pub defaults: Fields,
    /// Collection-specific metadata defaults.
    #[serde(default)]
    pub collections: BTreeMap<String, Fields>,
    /// Containment-selected metadata defaults.
    #[serde(default)]
    pub bundles: Vec<Bundle>,
    /// Exact-file metadata exceptions.
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    /// Ordered native status classifiers.
    #[serde(default)]
    pub status_profiles: BTreeMap<String, StatusProfile>,
    /// Explicit authoritative native table bindings.
    #[serde(default)]
    pub scope_tables: Vec<Binding>,
    /// Explicit native-item dependencies and status ownership.
    #[serde(default)]
    pub relationships: Vec<Relationship>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            version: 1,
            topics: Vec::new(),
            defaults: BTreeMap::new(),
            collections: BTreeMap::new(),
            bundles: Vec::new(),
            exceptions: Vec::new(),
            status_profiles: BTreeMap::new(),
            scope_tables: Vec::new(),
            relationships: Vec::new(),
        }
    }
}
/// Metadata defaults selected by the deepest containing source bundle.
#[derive(Clone, Debug, Deserialize)]
pub struct Bundle {
    /// Repository-relative source directory.
    pub root: String,
    /// Controlled metadata values.
    #[serde(default)]
    pub defaults: Fields,
}
/// Explicit same-file metadata exception.
#[derive(Clone, Debug, Deserialize)]
pub struct Exception {
    /// Repository-relative file.
    pub path: String,
    /// Controlled metadata values.
    #[serde(default)]
    pub values: Fields,
}
/// Ordered state patterns; qualification labels do not imply completion.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct StatusProfile {
    /// Earlier matching states take precedence.
    #[serde(default)]
    pub order: Vec<String>,
    /// Patterns keyed by state.
    #[serde(flatten)]
    pub patterns: BTreeMap<String, Vec<String>>,
}
fn default_id_pattern() -> String {
    "^(.+)$".to_owned()
}
/// One exact native table selector, without a second status ledger.
#[derive(Clone, Debug, Deserialize)]
pub struct Binding {
    /// Repository-relative retained plan.
    pub path: String,
    /// Stable binding identity.
    pub name: String,
    /// Parsed native heading.
    pub heading: String,
    /// Exact ordered plain header cells.
    pub headers: Vec<String>,
    /// Native item kind.
    pub kind: String,
    /// Native identity column.
    pub id_column: usize,
    /// Capturing native identity pattern.
    #[serde(default = "default_id_pattern")]
    pub id_pattern: String,
    /// Explicit selector when same-heading tables repeat.
    #[serde(default)]
    pub occurrence: Option<usize>,
    /// Optional native status column.
    #[serde(default)]
    pub status_column: Option<usize>,
    /// Optional native owner column.
    #[serde(default)]
    pub owner_column: Option<usize>,
    /// Selected ordered classification profile.
    #[serde(default)]
    pub status_profile: String,
}
/// Explicit native-scope relationship.
#[derive(Clone, Debug, Deserialize)]
pub struct Relationship {
    /// Full native source identity.
    #[serde(rename = "from")]
    pub source: String,
    /// depends-on/superseded-by/status-owned-by.
    pub kind: String,
    /// Full native target identity.
    #[serde(rename = "to")]
    pub target: String,
}
/// Parser-owned table with original Markdown cells and physical row locations.
#[derive(Clone, Debug)]
pub struct NativeTable {
    /// Most recent parsed heading.
    pub heading: String,
    /// Plain parsed header labels.
    pub headers: Vec<String>,
    /// Original Markdown cells, table pipe escapes decoded.
    pub rows: Vec<Vec<String>>,
    /// One-based source line of the header.
    pub line: usize,
    /// One-based source lines for data rows.
    pub row_lines: Vec<usize>,
    /// Physical rows whose source width differs from the header.
    pub arity_errors: Vec<usize>,
}
/// A derived native-scope item; unbound documents retain explicit unknown state.
#[derive(Clone, Debug, Serialize)]
pub struct ScopeItem {
    /// Full document/binding/native identity.
    pub id: String,
    /// Native table identity, absent for unbound documents.
    pub native_id: Option<String>,
    /// Native source document.
    pub document: String,
    /// Native table binding, absent for unbound documents.
    pub binding: Option<String>,
    /// Binding kind or unbound-document.
    pub kind: String,
    /// Effective explicitly owned status.
    pub state: String,
    /// Unmodified native status cell.
    pub raw_status: String,
    /// Source of effective classification.
    pub status_basis: String,
    /// Explicit relationships.
    pub relationships: Vec<ItemRelationship>,
    /// Original native label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Native heading reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Physical authored line including frontmatter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_line: Option<usize>,
    /// Original native classification before canonical ownership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_state: Option<String>,
    /// Explicit canonical status owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_owner: Option<String>,
    /// Original owner cell or document default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Original Markdown cells keyed by native header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_cells: Option<BTreeMap<String, String>>,
}
/// Relationship emitted with a scope item.
#[derive(Clone, Debug, Serialize)]
pub struct ItemRelationship {
    /// Native relationship kind.
    pub kind: String,
    /// Full target identity.
    pub target: String,
}
/// Read-only lifecycle query operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Operation {
    /// Selected publication metadata inventory.
    Inventory,
    /// Inventory plus all native scope binding checks.
    Validate,
    /// Retirement preflight, without release or deletion authorization.
    RetirePlan,
    /// Derived native table status query.
    Scope,
}
impl Operation {
    fn name(self) -> &'static str {
        match self {
            Self::Inventory => "inventory",
            Self::Validate => "validate",
            Self::RetirePlan => "retire-plan",
            Self::Scope => "scope",
        }
    }
}
/// Public lifecycle command arguments.
#[derive(Debug, Args)]
pub struct Query {
    /// Read-only operation.
    #[arg(value_enum)]
    pub command: Operation,
    /// Case-sensitive repository-relative document pattern.
    #[arg(long, default_value = "*")]
    pub path: String,
    /// Controlled document role selector.
    #[arg(long)]
    pub role: Option<String>,
    /// Declared document topic selector.
    #[arg(long)]
    pub topic: Option<String>,
    /// Publication scope selector.
    #[arg(long,value_parser=["Current","Reference","History"])]
    pub scope: Option<String>,
    /// Derived native state selector, only for scope queries.
    #[arg(long,value_parser=["open","blocked","deferred","complete","unknown"])]
    pub state: Option<String>,
}
fn source_path(path: &Path, reader: SourceReader<'_>) -> Result<std::path::PathBuf> {
    if reader.is_none() {
        return site::resolved(path);
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = std::path::PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(part.as_os_str()),
        }
    }
    Ok(normalized)
}
fn read_source(root: &Path, relative: &str, reader: SourceReader<'_>) -> Result<Vec<u8>> {
    if let Some(reader) = reader {
        reader(relative)
    } else {
        Ok(fs::read(root.join(relative))?)
    }
}
fn field(values: &Fields, key: &str) -> Result<String> {
    values
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .with_context(|| format!("{key} must be a string"))
}
fn historical(value: &str) -> bool {
    value.starts_with("git:") || value.starts_with("https:")
}
/// Canonicalize a local owner/section or validate an immutable historical reference.
/// Historical ownership never establishes current action authority.
/// # Errors
/// Invalid/escaping/missing reference or unknown section anchor.
pub fn reference(root: &Path, value: &str, source: Option<&Path>) -> Result<String> {
    reference_with_reader(root, value, source, None)
}
/// Resolve owner references using exactly the captured bytes supplied by the caller.
/// # Errors
/// The same path, ownership or anchor admission failures as `reference`.
pub fn reference_with_reader(
    root: &Path,
    value: &str,
    source: Option<&Path>,
    reader: SourceReader<'_>,
) -> Result<String> {
    if historical(value) {
        let pattern = Regex::new(
            r"^(?:git:([0-9a-f]{12,40}):|https://github\.com/[\w.-]+/[\w.-]+/blob/([0-9a-f]{12,40})/)([^#\s]+)(#\S+)?$",
        )?;
        let captures = pattern
            .captures(value)
            .with_context(|| format!("invalid immutable owner reference: {value:?}"))?;
        let path = Path::new(&captures[3]);
        ensure!(
            !path.is_absolute()
                && !path
                    .components()
                    .any(|part| matches!(part, Component::ParentDir)),
            "invalid immutable owner reference: {value:?}"
        );
        return Ok(value.to_owned());
    }
    let (path, fragment) = value
        .split_once('#')
        .map_or((value, None), |(path, fragment)| (path, Some(fragment)));
    let candidate = if path.is_empty() {
        source
            .context(format!("reference has no document path: {value:?}"))?
            .to_owned()
    } else {
        let base = if source.is_none() || path.starts_with("docs/") {
            root
        } else {
            source
                .and_then(Path::parent)
                .context("owner reference source parent absent")?
        };
        base.join(path)
    };
    let resolved = source_path(&candidate, reader)?;
    let root = source_path(root, reader)?;
    ensure!(
        resolved.starts_with(&root),
        "owner reference escapes repository: {value:?}"
    );
    let captured = if reader.is_some() {
        Some(read_source(
            &root,
            &resolved
                .strip_prefix(&root)?
                .to_string_lossy()
                .replace('\\', "/"),
            reader,
        )?)
    } else {
        ensure!(
            resolved.is_file(),
            "owner reference does not exist: {value:?}"
        );
        None
    };
    if let Some(fragment) = fragment {
        ensure!(
            !fragment.is_empty(),
            "empty owner section fragment: {value:?}"
        );
        let text = String::from_utf8(if let Some(raw) = captured {
            raw
        } else {
            read_source(
                &root,
                &resolved
                    .strip_prefix(&root)?
                    .to_string_lossy()
                    .replace('\\', "/"),
                reader,
            )?
        })?;
        let mut anchors: BTreeSet<String> = Regex::new(r#"<a\s+[^>]*id=["']([^"']+)"#)?
            .captures_iter(&text)
            .map(|captures| captures[1].to_owned())
            .collect();
        let section = Regex::new(r"^#{2,4}\s+(?:([0-9]+(?:\.[0-9]+)*)|([A-Z][0-9]+))\.?\s")?;
        for heading in markdown_headings(&text) {
            anchors.insert(slug(heading.source.trim_start_matches(['#', ' ']).trim())?);
            if let Some(captures) = section.captures(heading.source) {
                let id = captures
                    .get(1)
                    .or_else(|| captures.get(2))
                    .context("section identity capture absent")?
                    .as_str();
                anchors.insert(format!("section-{}", id.to_lowercase().replace('.', "-")));
            }
        }
        ensure!(
            anchors.contains(fragment),
            "owner section does not exist: {value:?}"
        );
    }
    Ok(format!(
        "{}{}",
        resolved
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/"),
        fragment.map_or(String::new(), |fragment| format!("#{fragment}"))
    ))
}
fn slug(value: &str) -> Result<String> {
    Ok(Regex::new(r"\s+")?
        .replace_all(
            &Regex::new(r"[^\w\s-]")?.replace_all(&value.to_lowercase(), ""),
            "-",
        )
        .into_owned())
}
/// Load and validate the sole lifecycle policy and its actual publication references.
/// # Errors
/// Invalid declarations, defaults, references or native classifiers.
pub fn load_policy(root: &Path, reader: SourceReader<'_>) -> Result<Policy> {
    let policy: Policy = toml::from_str(std::str::from_utf8(&read_source(
        root,
        "docs/lifecycle.toml",
        reader,
    )?)?)?;
    ensure!(
        policy.version == 1,
        "unsupported document lifecycle policy version"
    );
    let topics: BTreeSet<_> = policy.topics.iter().cloned().collect();
    ensure!(
        topics.len() == policy.topics.len() && !topics.contains(""),
        "lifecycle topics must be unique declared strings"
    );
    let config: site::Configuration = toml::from_str(std::str::from_utf8(&read_source(
        root,
        "docs/site.toml",
        reader,
    )?)?)?;
    let collection_roots: BTreeSet<_> = config
        .collections
        .iter()
        .map(|collection| collection.root.as_str())
        .collect();
    ensure!(
        policy
            .collections
            .keys()
            .all(|key| collection_roots.contains(key.as_str())),
        "lifecycle collection defaults must use existing publication collection roots"
    );
    let mut defaults = vec![&policy.defaults];
    defaults.extend(policy.collections.values());
    let mut roots = BTreeSet::new();
    let mut exceptions = BTreeSet::new();
    let root_resolved = source_path(root, reader)?;
    for bundle in &policy.bundles {
        let candidate = root.join(&bundle.root);
        let resolved = source_path(&candidate, reader)?;
        ensure!(
            !Path::new(&bundle.root).is_absolute()
                && resolved.starts_with(&root_resolved)
                && (reader.is_some() || candidate.is_dir()),
            "invalid lifecycle bundle root: {}",
            bundle.root
        );
        ensure!(roots.insert(resolved), "ambiguous bundle defaults");
        defaults.push(&bundle.defaults);
    }
    for exception in &policy.exceptions {
        let candidate = root.join(&exception.path);
        let resolved = source_path(&candidate, reader)?;
        ensure!(
            !Path::new(&exception.path).is_absolute()
                && resolved.starts_with(&root_resolved)
                && (reader.is_some() || candidate.is_file()),
            "invalid exact-file lifecycle exception: {}",
            exception.path
        );
        ensure!(
            exceptions.insert(resolved),
            "duplicate exact-file lifecycle exception"
        );
        defaults.push(&exception.values);
    }
    for values in defaults {
        ensure!(
            values.keys().all(|key| key.starts_with("doc_")),
            "unknown lifecycle default fields"
        );
        metadata::validate_fields(values, Some(&topics))?;
        for key in ["doc_owner", "doc_retention_owner"] {
            if values.contains_key(key) {
                reference_with_reader(root, &field(values, key)?, None, reader)?;
            }
        }
    }
    for profile in policy.status_profiles.values() {
        let states: BTreeSet<_> = profile.order.iter().collect();
        ensure!(
            states.len() == profile.order.len()
                && profile.order.iter().all(|state| [
                    "open", "blocked", "deferred", "complete", "unknown"
                ]
                .contains(&state.as_str())),
            "invalid native status profile ordering"
        );
        for state in &profile.order {
            for pattern in profile.patterns.get(state).into_iter().flatten() {
                Regex::new(pattern)?;
            }
        }
    }
    for binding in &policy.scope_tables {
        ensure!(
            policy.status_profiles.contains_key(&binding.status_profile),
            "unknown native status profile: {}",
            binding.status_profile
        );
    }
    Ok(policy)
}
fn merge(values: &mut Fields, origins: &mut BTreeMap<String, Value>, add: &Fields, origin: &str) {
    values.extend(add.clone());
    origins.extend(
        add.keys()
            .map(|key| (key.clone(), Value::String(origin.to_owned()))),
    );
}
/// Resolve defaults, legacy aliases and exact exceptions with authored provenance.
/// # Errors
/// Ambiguous defaults, conflicting owners/overrides, invalid metadata or references.
pub fn interpreted(
    root: &Path,
    path: &Path,
    document: &Document<'_>,
    policy: &Policy,
    config: &site::Configuration,
) -> Result<Fields> {
    interpreted_with_reader(root, path, document, policy, config, None)
}
/// Interpret metadata and owner anchors entirely through one captured source reader.
/// # Errors
/// The same precedence, metadata and ownership failures as `interpreted`.
pub fn interpreted_with_reader(
    root: &Path,
    path: &Path,
    document: &Document<'_>,
    policy: &Policy,
    config: &site::Configuration,
    reader: SourceReader<'_>,
) -> Result<Fields> {
    let resolved = source_path(path, reader)?;
    let relative = resolved
        .strip_prefix(source_path(root, reader)?)?
        .to_string_lossy()
        .replace('\\', "/");
    let mut values = Fields::new();
    let mut origins = BTreeMap::new();
    merge(
        &mut values,
        &mut origins,
        &policy.defaults,
        "global default",
    );
    let mut collections = Vec::new();
    for collection in &config.collections {
        let base = source_path(&root.join("docs").join(&collection.root), reader)?;
        if resolved.starts_with(&base)
            && (collection.recursive || resolved.parent() == Some(base.as_path()))
        {
            collections.push((base.components().count(), &collection.root));
        }
    }
    if let Some(depth) = collections.iter().map(|(depth, _)| *depth).max() {
        let nearest: Vec<_> = collections
            .iter()
            .filter(|(candidate, _)| *candidate == depth)
            .collect();
        ensure!(
            nearest.len() == 1,
            "{relative}: ambiguous collection defaults"
        );
        if let Some(defaults) = policy.collections.get(nearest[0].1) {
            merge(
                &mut values,
                &mut origins,
                defaults,
                &format!("collection:{}", nearest[0].1),
            );
        }
    }
    let bundles: Vec<_> = policy
        .bundles
        .iter()
        .filter(|bundle| Path::new(&relative).starts_with(&bundle.root))
        .collect();
    if let Some(depth) = bundles
        .iter()
        .map(|bundle| Path::new(&bundle.root).components().count())
        .max()
    {
        let nearest: Vec<_> = bundles
            .iter()
            .filter(|bundle| Path::new(&bundle.root).components().count() == depth)
            .collect();
        ensure!(nearest.len() == 1, "{relative}: ambiguous bundle defaults");
        merge(
            &mut values,
            &mut origins,
            &nearest[0].defaults,
            &format!("bundle:{}", nearest[0].root),
        );
    }
    let mut explicit: Fields = document
        .metadata
        .iter()
        .filter(|(key, _)| key.starts_with("doc_"))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let mut aliases = BTreeSet::new();
    for key in ["disposition_owner", "disposition-owner"] {
        if let Some(value) = document
            .metadata
            .get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            aliases.insert(reference_with_reader(root, value, Some(path), reader)?);
        }
    }
    ensure!(aliases.len() <= 1, "{relative}: conflicting legacy owners");
    if let Some(alias) = aliases.first() {
        if historical(alias) {
            values.insert("historical_owner".into(), Value::String(alias.clone()));
        } else {
            if explicit.contains_key("doc_owner") {
                ensure!(
                    reference_with_reader(root, &field(&explicit, "doc_owner")?, None, reader)?
                        == *alias,
                    "{relative}: conflicting canonical and legacy owners"
                );
            }
            explicit.insert("doc_owner".into(), Value::String(alias.clone()));
        }
    }
    for key in ["doc_owner", "doc_retention_owner"] {
        if explicit.contains_key(key) {
            explicit.insert(
                key.into(),
                Value::String(reference_with_reader(
                    root,
                    &field(&explicit, key)?,
                    None,
                    reader,
                )?),
            );
        }
    }
    let exceptions: Vec<_> = policy
        .exceptions
        .iter()
        .filter(|exception| exception.path == relative)
        .collect();
    ensure!(
        exceptions.len() <= 1,
        "{relative}: duplicate exact-file exception"
    );
    if let Some(exception) = exceptions.first() {
        let mut overrides = exception.values.clone();
        for key in ["doc_owner", "doc_retention_owner"] {
            if overrides.contains_key(key) {
                overrides.insert(
                    key.into(),
                    Value::String(reference_with_reader(
                        root,
                        &field(&overrides, key)?,
                        None,
                        reader,
                    )?),
                );
            }
        }
        for (key, value) in &explicit {
            if let Some(override_) = overrides.get(key) {
                ensure!(value == override_, "{relative}: conflicting explicit {key}");
            }
        }
        merge(
            &mut values,
            &mut origins,
            &overrides,
            "exact-file exception",
        );
    }
    merge(&mut values, &mut origins, &explicit, "authored metadata");
    let topics = policy.topics.iter().cloned().collect();
    metadata::validate_fields(&values, Some(&topics))?;
    for key in ["doc_owner", "doc_retention_owner"] {
        if values.contains_key(key) {
            values.insert(
                key.into(),
                Value::String(reference_with_reader(
                    root,
                    &field(&values, key)?,
                    None,
                    reader,
                )?),
            );
        }
    }
    if values
        .get("doc_owner")
        .and_then(Value::as_str)
        .is_some_and(historical)
        && let Some(owner) = values.remove("doc_owner")
    {
        values.insert("historical_owner".into(), owner);
    }
    if !values.contains_key("doc_owner") {
        values.insert(
            "doc_retention".into(),
            Value::String("protected-unknown".into()),
        );
    }
    values.insert("metadata_origins".into(), Value::Mapping(origins));
    Ok(values)
}

fn plain(content: &str) -> String {
    Parser::new(content)
        .filter_map(|event| match event {
            Event::Text(value) | Event::Code(value) => Some(value.into_string()),
            _ => None,
        })
        .collect()
}
fn line_at(body: &str, offset: usize) -> usize {
    body[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1
}
fn row_arity(row: &str) -> usize {
    // GFM pads/truncates to header width. A deliberately wider probe lets the
    // supplier expose all original cells; its zero-width cells at EOF are pads.
    // No escaped-pipe grammar or Markdown lexer is duplicated here.
    let columns = row.bytes().filter(|byte| *byte == b'|').count() + 2;
    let header = vec!["h"; columns].join("|");
    let separator = vec!["---"; columns].join("|");
    let source = format!("{header}\n{separator}\n{}\n", row.trim());
    let mut data = false;
    let mut count = 0;
    for (event, range) in Parser::new_ext(&source, Options::ENABLE_TABLES).into_offset_iter() {
        match event {
            Event::Start(Tag::TableRow) => data = true,
            Event::End(TagEnd::TableRow) => data = false,
            Event::Start(Tag::TableCell) if data && range.start < source.len() => count += 1,
            _ => {}
        }
    }
    count
}
fn heading_content(raw: &str) -> Result<String> {
    let line = raw.lines().next().unwrap_or("");
    let text = line.trim_start_matches('#').trim();
    Ok(Regex::new(r"\s+#+\s*$")?.replace(text, "").into_owned())
}
/// Parse native GFM tables with exact physical rows and supplier-owned pipe rules.
/// `strict=false` defers row-width errors until an actual binding selects a table.
/// # Errors
/// Strict malformed-row rejection or parser-adapter failures.
pub fn tables(body: &str, strict: bool) -> Result<Vec<NativeTable>> {
    let mut result = Vec::new();
    let mut heading = String::new();
    let mut table: Option<NativeTable> = None;
    let mut cells = Vec::new();
    let mut row_line = 0;
    let mut row_source = String::new();
    let lines: Vec<_> = body.lines().collect();
    for (event, range) in Parser::new_ext(body, Options::ENABLE_TABLES).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { .. }) => heading = heading_content(&body[range])?,
            Event::Start(Tag::Table(_)) => {
                table = Some(NativeTable {
                    heading: heading.clone(),
                    headers: Vec::new(),
                    rows: Vec::new(),
                    line: line_at(body, range.start),
                    row_lines: Vec::new(),
                    arity_errors: Vec::new(),
                })
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => {
                cells.clear();
                row_line = line_at(body, range.start);
                row_source = lines.get(row_line - 1).copied().unwrap_or("").to_owned();
            }
            Event::Start(Tag::TableCell) => cells.push(body[range].trim().replace("\\|", "|")),
            Event::End(TagEnd::TableHead) => {
                if let Some(table) = &mut table {
                    table.headers = cells.iter().map(|cell| plain(cell)).collect();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(table) = &mut table {
                    if row_arity(&row_source) != table.headers.len() {
                        ensure!(
                            !strict,
                            "table at line {row_line}: row arity differs from header; escape literal pipes"
                        );
                        table.arity_errors.push(row_line);
                    }
                    table.rows.push(cells.clone());
                    table.row_lines.push(row_line);
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = table.take() {
                    result.push(table);
                }
            }
            _ => {}
        }
    }
    Ok(result)
}
fn state(raw: &str, profile: Option<&StatusProfile>) -> Result<String> {
    if let Some(profile) = profile {
        for state in &profile.order {
            for pattern in profile.patterns.get(state).into_iter().flatten() {
                if Regex::new(pattern)?.is_match(raw) {
                    return Ok(state.clone());
                }
            }
        }
    }
    Ok("unknown".to_owned())
}
fn endpoint_document(identity: &str) -> &str {
    identity.split("::").next().unwrap_or(identity)
}
/// Aggregate explicitly bound native scope, optionally expanding the connected document set.
/// Captured source readers let evidence consumers reuse this sole interpretation.
/// # Errors
/// Invalid selectors, duplicate IDs, malformed bound rows or ownership graph errors.
pub fn aggregate(
    root: &Path,
    policy: &Policy,
    documents: Option<&BTreeSet<String>>,
    reader: SourceReader<'_>,
) -> Result<Vec<ScopeItem>> {
    let mut selected = documents.cloned().unwrap_or_default();
    if documents.is_some() {
        loop {
            let before = selected.len();
            for edge in &policy.relationships {
                let source = endpoint_document(&edge.source);
                let target = endpoint_document(&edge.target);
                if selected.contains(source) || selected.contains(target) {
                    selected.insert(source.to_owned());
                    selected.insert(target.to_owned());
                }
            }
            if selected.len() == before {
                break;
            }
        }
    }
    let mut result = Vec::<ScopeItem>::new();
    let mut seen = BTreeSet::new();
    let mut cache = BTreeMap::<String, (usize, Vec<NativeTable>)>::new();
    let mut bound_documents = BTreeSet::new();
    for binding in &policy.scope_tables {
        if documents.is_some() && !selected.contains(&binding.path) {
            continue;
        }
        let path = root.join(&binding.path);
        ensure!(
            source_path(&path, reader)?.starts_with(source_path(&root.join("docs/plans"), reader)?),
            "scope table must belong to retained plan content"
        );
        if !cache.contains_key(&binding.path) {
            let raw = read_source(root, &binding.path, reader)?;
            let document = metadata::read_with_policy(
                &raw,
                &binding.path,
                false,
                Some(&binding.path),
                root,
                reader.map(|_| ""),
            )?;
            let offset = document
                .prefix
                .split_inclusive(|byte| *byte == b'\n')
                .count();
            let parsed = tables(std::str::from_utf8(document.body)?, false)?;
            cache.insert(binding.path.clone(), (offset, parsed));
        }
        let (offset, candidates) = cache
            .get(&binding.path)
            .context("native table cache absent")?;
        let matching: Vec<_> = candidates
            .iter()
            .filter(|table| table.heading == binding.heading && table.headers == binding.headers)
            .collect();
        let selected_table = match binding.occurrence {
            None if matching.len() == 1 => Some(matching[0]),
            Some(index) if index >= 1 => matching.get(index - 1).copied(),
            _ => None,
        }
        .with_context(|| {
            format!(
                "{}::{}: scope selector must select exactly one native table",
                binding.path, binding.name
            )
        })?;
        ensure!(
            selected_table.arity_errors.is_empty(),
            "{}::{}: bound table row arity differs at lines {:?}; escape literal pipes",
            binding.path,
            binding.name,
            selected_table.arity_errors
        );
        ensure!(
            binding.id_column < binding.headers.len()
                && binding
                    .status_column
                    .is_none_or(|index| index < binding.headers.len())
                && binding
                    .owner_column
                    .is_none_or(|index| index < binding.headers.len()),
            "{}::{}: native binding column outside table",
            binding.path,
            binding.name
        );
        let identity_pattern = Regex::new(&binding.id_pattern)?;
        ensure!(
            identity_pattern.captures_len() > 1,
            "native ID pattern must capture the identity"
        );
        for (row, line) in selected_table.rows.iter().zip(&selected_table.row_lines) {
            let label = plain(&row[binding.id_column]);
            let captures = identity_pattern.captures(&label).with_context(|| {
                format!("{}:{line}: native ID does not match binding", binding.path)
            })?;
            let native_id = captures
                .get(1)
                .context("native identity capture absent")?
                .as_str()
                .to_owned();
            let identity = format!("{}::{}::{native_id}", binding.path, binding.name);
            ensure!(
                seen.insert(identity.clone()),
                "duplicate native scope ID: {identity}"
            );
            let raw = binding
                .status_column
                .map_or("", |index| row[index].as_str());
            result.push(ScopeItem {
                id: identity,
                native_id: Some(native_id),
                label: Some(label),
                document: binding.path.clone(),
                binding: Some(binding.name.clone()),
                kind: binding.kind.clone(),
                source: Some(format!(
                    "{}#{}",
                    binding.path,
                    slug(&selected_table.heading)?
                )),
                source_line: Some(line + offset),
                raw_status: raw.to_owned(),
                state: state(
                    &plain(raw),
                    policy.status_profiles.get(&binding.status_profile),
                )?,
                native_state: None,
                status_owner: None,
                status_basis: if binding.status_column.is_some() {
                    "bound native column"
                } else {
                    "no authoritative status column"
                }
                .to_owned(),
                owner: Some(
                    binding
                        .owner_column
                        .map_or_else(|| binding.path.clone(), |index| row[index].clone()),
                ),
                native_cells: Some(
                    binding
                        .headers
                        .iter()
                        .cloned()
                        .zip(row.iter().cloned())
                        .collect(),
                ),
                relationships: Vec::new(),
            });
        }
        bound_documents.insert(binding.path.clone());
    }
    let by_id: BTreeMap<_, _> = result
        .iter()
        .enumerate()
        .map(|(index, item)| (item.id.clone(), index))
        .collect();
    let mut status_owners = BTreeMap::<String, String>::new();
    for edge in &policy.relationships {
        if documents.is_some() && !selected.contains(endpoint_document(&edge.source)) {
            continue;
        }
        let source = by_id.get(&edge.source).with_context(|| {
            format!(
                "scope relationship names an unknown native item: {} -> {}",
                edge.source, edge.target
            )
        })?;
        ensure!(
            by_id.contains_key(&edge.target),
            "scope relationship names an unknown native item: {} -> {}",
            edge.source,
            edge.target
        );
        ensure!(
            ["depends-on", "superseded-by", "status-owned-by"].contains(&edge.kind.as_str()),
            "unsupported scope relationship kind"
        );
        ensure!(
            edge.source != edge.target,
            "scope relationship cannot point to itself"
        );
        result[*source].relationships.push(ItemRelationship {
            kind: edge.kind.clone(),
            target: edge.target.clone(),
        });
        if edge.kind == "status-owned-by"
            && let Some(previous) = status_owners.insert(edge.source.clone(), edge.target.clone())
        {
            ensure!(
                previous == edge.target,
                "conflicting canonical scope status owners: {}",
                edge.source
            );
        }
    }
    for item in &mut result {
        item.native_state = Some(item.state.clone());
        let mut identity = item.id.clone();
        let mut visited = BTreeSet::new();
        while let Some(owner) = status_owners.get(&identity) {
            ensure!(
                visited.insert(identity.clone()),
                "cyclic canonical scope status ownership: {identity}"
            );
            identity = owner.clone();
        }
        item.status_owner = Some(identity);
    }
    let states: BTreeMap<_, _> = result
        .iter()
        .map(|item| (item.id.clone(), item.state.clone()))
        .collect();
    for item in &mut result {
        if let Some(owner) = &item.status_owner
            && owner != &item.id
        {
            item.state = states
                .get(owner)
                .context("canonical scope owner absent")?
                .clone();
            item.status_basis =
                "canonical owner through explicit status-owned-by relationship".to_owned();
        }
    }
    let plans = root.join("docs/plans");
    if reader.is_none() && plans.is_dir() {
        let pattern = Regex::new(r"^(?:2[89]|[3-9][0-9])[a-z]?-")?;
        for path in site::walk_files(&plans, false, Some("md"))? {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            if path
                .file_name()
                .is_some_and(|name| pattern.is_match(&name.to_string_lossy()))
                && !bound_documents.contains(&relative)
                && (documents.is_none() || selected.contains(&relative))
            {
                result.push(ScopeItem {
                    id: relative.clone(),
                    document: relative,
                    native_id: None,
                    binding: None,
                    kind: "unbound-document".into(),
                    state: "unknown".into(),
                    raw_status: String::new(),
                    status_basis: "no configured authoritative native table".into(),
                    relationships: Vec::new(),
                    label: None,
                    source: None,
                    source_line: None,
                    native_state: None,
                    status_owner: None,
                    owner: None,
                    native_cells: None,
                });
            }
        }
    }
    result.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(result)
}
/// Inventory only selected publication pages with independent native statuses and ownership.
/// # Errors
/// Invalid publication discovery; individual metadata errors are accumulated.
pub fn inventory(root: &Path, policy: &Policy) -> Result<(Vec<serde_json::Value>, Vec<String>)> {
    let config = site::configuration(root)?;
    let pages = site::discover(root, &config)?;
    let mut records = Vec::new();
    let mut errors = Vec::new();
    for page in pages {
        let path = root.join("docs").join(&page.path);
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        let entry = (|| -> Result<serde_json::Value> {
            let raw = fs::read(&path)?;
            let document = metadata::read(&raw, &relative, false, Some(&relative), root)?;
            let values = interpreted(root, &path, &document, policy, &config)?;
            let mut record = serde_json::to_value(values)?;
            let object = record
                .as_object_mut()
                .context("interpreted metadata is not an object")?;
            object.insert("path".into(), json!(relative));
            object.insert("scope".into(), json!(page.scope));
            object.insert(
                "native_status".into(),
                serde_json::to_value(document.metadata.get("status"))?,
            );
            object.insert(
                "native_decision".into(),
                serde_json::to_value(document.metadata.get("decision"))?,
            );
            object.insert(
                "sha256".into(),
                json!(
                    Sha256::digest(&raw)
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                ),
            );
            object.insert("apparent_bytes".into(), json!(raw.len()));
            Ok(record)
        })();
        match entry {
            Ok(record) => records.push(record),
            Err(error) => errors.push(error.to_string()),
        }
    }
    Ok((records, errors))
}
/// Build a read-only observation; retirement output never grants release/deletion.
/// # Errors
/// Invalid source or policy, apart from accumulated inventory/validation errors.
pub fn snapshot(root: &Path, command: Operation, policy: &Policy) -> Result<serde_json::Value> {
    let observed = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false);
    if command == Operation::Scope {
        return Ok(
            json!({"version":1,"observed_at":observed,"operation":command.name(),"authority":"derived native status; no scheduling or authorization","items":aggregate(root,policy,None,None)?}),
        );
    }
    let (records, mut errors) = inventory(root, policy)?;
    let mut result = json!({"version":1,"observed_at":observed,"operation":command.name(),"documents":records,"exclusions":["production and test bodies",".venv*","build and target payloads","persistent state","shared external resources"],"deletion_authorized":false});
    if command == Operation::Validate {
        match aggregate(root, policy, None, None) {
            Ok(items) => {
                result["scope_items_validated"] = json!(items.len());
            }
            Err(error) => errors.push(error.to_string()),
        }
    }
    if command == Operation::RetirePlan {
        let mut references = BTreeMap::<String, BTreeSet<String>>::new();
        for record in &records {
            let path = record["path"].as_str().context("inventory path absent")?;
            let body = fs::read_to_string(root.join(path))?;
            for event in Parser::new_ext(&body, Options::ENABLE_TABLES) {
                if let Event::Start(Tag::Link { dest_url, .. }) = event {
                    let target = dest_url.as_ref();
                    if target.contains("://") || target.starts_with('#') {
                        continue;
                    }
                    let target = target.split('#').next().unwrap_or(target);
                    let candidate = site::resolved(
                        &root
                            .join(path)
                            .parent()
                            .context("reference parent absent")?
                            .join(target),
                    )?;
                    if let Ok(relative) = candidate.strip_prefix(site::resolved(root)?) {
                        references
                            .entry(relative.to_string_lossy().replace('\\', "/"))
                            .or_default()
                            .insert(path.to_owned());
                    }
                }
            }
        }
        result["candidates"]=json!(records.iter().filter(|record|record["doc_role"]=="plan").map(|record|{let path=record["path"].as_str().unwrap_or("");let mut blockers=vec!["owner release not established"];if record.get("doc_owner").is_none(){blockers.push("unknown canonical owner");}json!({"path":path,"surviving_meaning_owner":record.get("doc_owner"),"references_to_repair":references.get(path).cloned().unwrap_or_default(),"blockers":blockers,"native_status":record["native_status"]})}).collect::<Vec<_>>());
    }
    result["errors"] = json!(errors);
    Ok(result)
}
/// Apply supported query filters and emit a single structured observation.
/// # Errors
/// Invalid selector combinations, source errors, or observed validation failures.
pub fn execute(root: &Path, query: Query) -> Result<()> {
    ensure!(
        query.command != Operation::Scope
            || (query.role.is_none() && query.topic.is_none() && query.scope.is_none()),
        "scope query supports --path/--state; --role/--topic/--scope select documents"
    );
    ensure!(
        query.command == Operation::Scope || query.state.is_none(),
        "--state selects native scope items; use the scope command"
    );
    let policy = load_policy(root, None)?;
    let mut result = snapshot(root, query.command, &policy)?;
    if query.command == Operation::Scope {
        let items = result["items"].as_array().context("scope items absent")?;
        let mut selected = Vec::new();
        for item in items {
            if site::matches(
                item["document"].as_str().context("scope document absent")?,
                std::slice::from_ref(&query.path),
            )? && query
                .state
                .as_deref()
                .is_none_or(|state| item["state"] == state)
            {
                selected.push(item.clone());
            }
        }
        let counts: BTreeMap<_, _> = ["open", "blocked", "deferred", "complete", "unknown"]
            .into_iter()
            .map(|state| {
                (
                    state,
                    selected
                        .iter()
                        .filter(|item| item["state"] == state)
                        .count(),
                )
            })
            .collect();
        result["items"] = json!(selected);
        result["counts"] = json!(counts);
    } else {
        let records = result["documents"]
            .as_array()
            .context("inventory documents absent")?;
        let mut selected = Vec::new();
        for record in records {
            if site::matches(
                record["path"].as_str().context("inventory path absent")?,
                std::slice::from_ref(&query.path),
            )? && query
                .role
                .as_deref()
                .is_none_or(|role| record["doc_role"] == role)
                && query
                    .scope
                    .as_deref()
                    .is_none_or(|scope| record["scope"] == scope)
                && query.topic.as_deref().is_none_or(|topic| {
                    record["doc_topics"]
                        .as_array()
                        .is_some_and(|topics| topics.iter().any(|value| value == topic))
                })
            {
                selected.push(record.clone());
            }
        }
        let paths: BTreeSet<_> = selected
            .iter()
            .filter_map(|record| record["path"].as_str().map(str::to_owned))
            .collect();
        result["documents"] = json!(selected);
        if let Some(candidates) = result
            .get_mut("candidates")
            .and_then(serde_json::Value::as_array_mut)
        {
            candidates.retain(|candidate| {
                candidate["path"]
                    .as_str()
                    .is_some_and(|path| paths.contains(path))
            });
        }
    }
    println!("{}", serde_json::to_string_pretty(&result)?);
    ensure!(
        result["errors"].as_array().is_none_or(Vec::is_empty),
        "document-lifecycle: validation errors"
    );
    Ok(())
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
