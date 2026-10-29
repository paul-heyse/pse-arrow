// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded source-only knowledge construction. Captured bytes are provenance, never authority.
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result, bail, ensure};
use pse_codegen::codegen::provenance::SourceRelationship;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::knowledge::{
    DocumentRole, KnowledgeCorpus, KnowledgeEdge, KnowledgeExcerpt, KnowledgeProduction,
    KnowledgeSource, KnowledgeSubject, SubjectKind, SubjectSource, SubjectTopic,
};
use crate::documentation::{adr, headings::markdown_headings, lifecycle, metadata, site};

const POLICY: &str = "docs/lifecycle.toml";
const SITE: &str = "docs/site.toml";
const BLUEPRINT: &str = "docs/authoritative_design/blueprint.md";
const PRODUCERS: &str = "docs/generated/source-relationships";
const EXCLUDED: &[&str] = &[
    "external", "build", "target", ".git", "book", "node_modules", "__pycache__",
    ".store", ".cache", "secrets", "credentials",
];
const SUFFIXES: &[&str] = &[
    "md", "rs", "py", "pyi", "toml", "surql", "json", "yml", "yaml", "lock", "patch",
];

/// Explicit finite limits; these bound this read, not host workload capacity.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SourceBounds {
    /// All selected and interpretation contributors, at most 256.
    pub max_sources: usize,
    /// Aggregate captured bytes, at most 32 MiB.
    pub max_bytes: usize,
    /// One admitted source, at most 4 MiB.
    pub max_source_bytes: usize,
    /// Closed native subjects, at most 256.
    pub max_subjects: usize,
    /// Directory entries examined for finite native owner/producer indexes.
    pub max_discovery_entries: usize,
    /// Capture and interpretation deadline in milliseconds, at most 60 seconds.
    pub milliseconds: u64,
}
impl Default for SourceBounds {
    fn default() -> Self {
        Self { max_sources: 256, max_bytes: 32 << 20, max_source_bytes: 4 << 20,
            max_subjects: 256, max_discovery_entries: 4096, milliseconds: 15000 }
    }
}
/// One exact one-based physical source extent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLineRange {
    /// Inclusive original first line.
    pub start_line: usize,
    /// Inclusive original last line.
    pub end_line: usize,
}
/// An explicit source selection; no Git, database, compiler or dependency resolver is run.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SourceSelection {
    /// One to 64 distinct checkout-relative code/document inputs.
    pub paths: Vec<String>,
    /// Optional exact hashes for any selected or transitive contributor.
    pub expected_digests: BTreeMap<String, String>,
    /// Uninterpreted activation precondition carried into the typed corpus.
    pub expected_active: Option<String>,
    /// Optional source excerpt extents, at most 640 lines each.
    pub ranges: BTreeMap<String, SourceLineRange>,
    /// Finite selection/read limits.
    pub bounds: SourceBounds,
}
/// Original native subject attribution, independent of excerpt truncation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SubjectRange {
    /// Closed subject identity.
    pub subject_id: String,
    /// Exact source identity.
    pub source_id: String,
    /// Native section identity or full native scope identity.
    pub native_id: String,
    /// Physical original extent.
    #[serde(flatten)]
    pub range: SourceLineRange,
}
/// A validated read cut. Every interpretation read is served from these retained bytes.
#[derive(Clone, Debug)]
pub struct SourceCut {
    root: PathBuf,
    selection: SourceSelection,
    layout: Layout,
    files: BTreeMap<String, Vec<u8>>,
    unavailable: BTreeMap<String, String>,
}
impl SourceCut {
    /// Exact captured contributor bytes, with no live path or access implication.
    pub fn sources(&self) -> &BTreeMap<String, Vec<u8>> { &self.files }
}
/// Source-only projection and its explicit scope limits; import still validates the corpus.
#[derive(Clone, Debug, Serialize)]
pub struct SourceCorpusBuild {
    /// Existing closed v2 knowledge representation.
    pub corpus: KnowledgeCorpus,
    /// Unsupported or undeclared meaning; never inferred from textual proximity.
    pub unknowns: Vec<String>,
    /// Unavailable contributors or finite-scope omissions; prevent a completeness claim.
    pub omissions: Vec<String>,
    /// Exact native section and package source extents.
    pub ranges: Vec<SubjectRange>,
}

fn digest(raw: &[u8]) -> String {
    Sha256::digest(raw).iter().map(|byte| format!("{byte:02x}")).collect()
}
fn parsed<T: std::str::FromStr>(text: &str) -> Result<T> {
    text.parse().map_err(|_| anyhow::anyhow!("unknown native vocabulary: {text}"))
}
fn validate_selection(selection: &SourceSelection) -> Result<()> {
    let bounds = &selection.bounds;
    ensure!(!selection.paths.is_empty() && selection.paths.len() <= 64,
        "select 1..64 explicit source paths");
    let unique: BTreeSet<_> = selection.paths.iter().collect();
    ensure!(unique.len() == selection.paths.len(), "duplicate selected source");
    ensure!((1..=256).contains(&bounds.max_sources) && (1..=256).contains(&bounds.max_subjects)
        && (1..=32 << 20).contains(&bounds.max_bytes)
        && (1..=4 << 20).contains(&bounds.max_source_bytes)
        && (1..=4096).contains(&bounds.max_discovery_entries)
        && (1..=60000).contains(&bounds.milliseconds), "invalid finite source bounds");
    for path in selection.paths.iter().chain(selection.expected_digests.keys()).chain(selection.ranges.keys()) {
        admitted_relative(path)?;
    }
    ensure!(selection.expected_digests.values().all(|value| value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))),
        "expected source digests must be lowercase SHA256");
    Ok(())
}
fn admitted_relative(relative: &str) -> Result<&Path> {
    let path = Path::new(relative);
    ensure!(!relative.is_empty() && relative.len() <= 4096 && !relative.contains(['\\', '\0'])
        && path.components().all(|part| matches!(part, Component::Normal(_)))
        && path.components().collect::<PathBuf>().to_str() == Some(relative),
        "source must be a literal repository-relative path: {relative:?}");
    ensure!(!path.components().any(|part| {
        let name = part.as_os_str().to_string_lossy();
        EXCLUDED.contains(&name.as_ref()) || name.starts_with(".venv")
    }), "excluded source environment/storage path: {relative}");
    let name = path.file_name().context("source filename absent")?.to_string_lossy();
    ensure!(name != "history.txt" && !name.starts_with(".env")
        && path.extension().and_then(|value| value.to_str()).is_some_and(|suffix| SUFFIXES.contains(&suffix)),
        "source is not an admitted code/document input: {relative}");
    Ok(path)
}
fn regular_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let relative = admitted_relative(relative)?;
    let mut candidate = root.to_owned();
    for part in relative.components() {
        candidate.push(part.as_os_str());
        ensure!(!fs::symlink_metadata(&candidate)?.file_type().is_symlink(),
            "source cannot traverse symbolic links: {}", candidate.display());
    }
    ensure!(fs::symlink_metadata(&candidate)?.is_file(), "source must be a regular file");
    Ok(candidate)
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)] device: u64,
    #[cfg(unix)] inode: u64,
    #[cfg(unix)] mode: u32,
    #[cfg(unix)] ctime: (i64, i64),
    #[cfg(unix)] mtime: (i64, i64),
}
impl Stamp {
    fn of(info: &Metadata) -> Self {
        #[cfg(unix)] use std::os::unix::fs::MetadataExt;
        Self { len: info.len(), modified: info.modified().ok(),
            #[cfg(unix)] device: info.dev(), #[cfg(unix)] inode: info.ino(),
            #[cfg(unix)] mode: info.mode(), #[cfg(unix)] ctime: (info.ctime(), info.ctime_nsec()),
            #[cfg(unix)] mtime: (info.mtime(), info.mtime_nsec()) }
    }
}
struct Capture<'a> {
    root: &'a Path,
    selection: &'a SourceSelection,
    deadline: Instant,
    files: RefCell<BTreeMap<String, Vec<u8>>>,
    stamps: RefCell<BTreeMap<String, Stamp>>,
    unavailable: RefCell<BTreeMap<String, String>>,
}
impl Capture<'_> {
    fn read(&self, relative: &str) -> Result<Vec<u8>> {
        if let Some(raw) = self.files.borrow().get(relative) { return Ok(raw.clone()); }
        if let Some(error) = self.unavailable.borrow().get(relative) { bail!("{error}"); }
        let result = self.capture(relative);
        if let Err(error) = &result { self.unavailable.borrow_mut().insert(relative.into(), error.to_string()); }
        result
    }
    fn capture(&self, relative: &str) -> Result<Vec<u8>> {
        ensure!(Instant::now() < self.deadline, "source capture deadline exhausted");
        let bounds = &self.selection.bounds;
        ensure!(self.files.borrow().len() < bounds.max_sources, "source contributor bound exhausted");
        let used: usize = self.files.borrow().values().map(Vec::len).sum();
        let allowance = bounds.max_source_bytes.min(bounds.max_bytes.saturating_sub(used));
        ensure!(allowance > 0, "aggregate source byte bound exhausted");
        let path = regular_path(self.root, relative)?;
        let before = Stamp::of(&fs::symlink_metadata(&path)?);
        ensure!(before.len <= allowance as u64, "selected source exceeds byte bound: {relative}");
        let mut file = File::open(&path)?;
        ensure!(file.metadata()?.is_file() && Stamp::of(&file.metadata()?) == before,
            "selected source replaced before read: {relative}");
        let mut raw = Vec::new();
        (&mut file).take(allowance as u64 + 1).read_to_end(&mut raw)?;
        ensure!(raw.len() <= allowance && Stamp::of(&file.metadata()?) == before
            && Stamp::of(&fs::symlink_metadata(regular_path(self.root, relative)?)?) == before,
            "selected source changed during read: {relative}");
        ensure!(Instant::now() < self.deadline, "source capture deadline exhausted");
        if let Some(expected) = self.selection.expected_digests.get(relative) {
            ensure!(digest(&raw) == *expected, "selected source digest mismatch: {relative}");
        }
        self.stamps.borrow_mut().insert(relative.into(), before);
        self.files.borrow_mut().insert(relative.into(), raw.clone());
        Ok(raw)
    }
    fn verify(&self, layout: &Layout) -> Result<()> {
        ensure!(Instant::now() < self.deadline, "source capture deadline exhausted");
        for (relative, expected) in self.stamps.borrow().iter() {
            ensure!(Stamp::of(&fs::symlink_metadata(regular_path(self.root, relative)?)?) == *expected,
                "source cut changed before closure: {relative}");
        }
        for (directory, expected) in &layout.directories {
            ensure!(Stamp::of(&fs::symlink_metadata(self.root.join(directory))?) == *expected,
                "source index membership changed before closure: {directory}");
        }
        for path in &layout.missing {
            ensure!(!self.root.join(path).try_exists()?, "absent source policy/index appeared before closure: {path}");
        }
        ensure!(self.selection.expected_digests.keys().all(|path| self.files.borrow().contains_key(path)),
            "expected digest names an uncaptured contributor");
        ensure!(Instant::now() < self.deadline, "source capture deadline exhausted");
        Ok(())
    }
}
#[derive(Clone, Debug, Default)]
struct Layout {
    policy: bool,
    site: bool,
    sections: Vec<String>,
    producers: Vec<String>,
    directories: BTreeMap<String, Stamp>,
    missing: BTreeSet<String>,
}
fn discover(root: &Path, selection: &SourceSelection, deadline: Instant) -> Result<Layout> {
    let mut layout = Layout { policy: root.join(POLICY).try_exists()?, site: root.join(SITE).try_exists()?, ..Layout::default() };
    if !layout.policy { layout.missing.insert(POLICY.into()); }
    if !layout.site { layout.missing.insert(SITE.into()); }
    let mut remaining = selection.bounds.max_discovery_entries;
    if root.join(BLUEPRINT).try_exists()? {
        layout.sections.push(BLUEPRINT.into());
        discover_directory(root, "docs/authoritative_design/sections", "md", deadline, &mut remaining,
            &mut layout.sections, &mut layout.directories, &mut layout.missing)?;
    } else { layout.missing.insert(BLUEPRINT.into()); }
    if selection.paths.iter().any(|path| !path.ends_with(".md")) {
        discover_directory(root, PRODUCERS, "json", deadline, &mut remaining,
            &mut layout.producers, &mut layout.directories, &mut layout.missing)?;
        layout.producers.retain(|path| path.ends_with("/relationships.json"));
    }
    layout.sections.sort(); layout.producers.sort();
    Ok(layout)
}
fn discover_directory(root: &Path, relative: &str, suffix: &str, deadline: Instant,
    remaining: &mut usize, output: &mut Vec<String>, directories: &mut BTreeMap<String, Stamp>, missing: &mut BTreeSet<String>) -> Result<()> {
    let path = root.join(relative);
    if !path.try_exists()? { missing.insert(relative.into()); return Ok(()); }
    let mut ancestor = root.to_owned();
    for part in Path::new(relative).components() {
        ensure!(matches!(part, Component::Normal(_)), "source index must be repository-relative");
        ancestor.push(part.as_os_str());
        ensure!(!fs::symlink_metadata(&ancestor)?.file_type().is_symlink(), "source index cannot traverse symbolic links");
    }
    let info = fs::symlink_metadata(&path)?;
    ensure!(info.is_dir() && !info.file_type().is_symlink(), "source index cannot traverse a symbolic directory");
    directories.insert(relative.into(), Stamp::of(&info));
    for entry in fs::read_dir(path)? {
        ensure!(Instant::now() < deadline && *remaining > 0, "source index discovery bound exhausted");
        *remaining -= 1;
        let entry = entry?;
        let kind = entry.file_type()?;
        ensure!(!kind.is_symlink(), "source index cannot include symbolic links");
        let name = entry.file_name().into_string().map_err(|_| anyhow::anyhow!("non-UTF8 source index path"))?;
        let child = format!("{relative}/{name}");
        if EXCLUDED.contains(&name.as_str()) || name.starts_with(".venv") { continue; }
        if kind.is_dir() {
            discover_directory(root, &child, suffix, deadline, remaining, output, directories, missing)?;
        } else if kind.is_file() && entry.path().extension().is_some_and(|value| value == suffix) {
            admitted_relative(&child)?; output.push(child);
        }
    }
    Ok(())
}

/// Capture one finite original byte cut and verify every read's stamp again at closure.
/// Interpretation itself uses the same adapters as the subsequent offline build.
/// # Errors
/// Invalid selection, bounds, changing bytes/membership, or invalid owner declarations.
pub fn bounded_source_cut(root: &Path, selection: &SourceSelection) -> Result<SourceCut> {
    validate_selection(selection)?;
    let root = fs::canonicalize(root)?;
    let deadline = Instant::now() + Duration::from_millis(selection.bounds.milliseconds);
    let layout = discover(&root, selection, deadline)?;
    let capture = Capture { root: &root, selection, deadline, files: RefCell::new(BTreeMap::new()),
        stamps: RefCell::new(BTreeMap::new()), unavailable: RefCell::new(BTreeMap::new()) };
    for path in &selection.paths { capture.read(path)?; }
    let reader = |path: &str| capture.read(path);
    let _ = assemble(&root, selection, &layout, &reader)?;
    capture.verify(&layout)?;
    let files = capture.files.into_inner();
    let unavailable = capture.unavailable.into_inner();
    Ok(SourceCut { root, selection: selection.clone(), layout, files, unavailable })
}
/// Construct owner-native concepts exclusively from a completed cut; no live files are opened.
/// # Errors
/// Conflicting native meanings, malformed owner data, or a typed corpus extent violation.
pub fn build_corpus(cut: &SourceCut) -> Result<SourceCorpusBuild> {
    let reader = |path: &str| cut.files.get(path).cloned().with_context(|| {
        cut.unavailable.get(path).cloned().unwrap_or_else(|| format!("source outside captured cut: {path}"))
    });
    let mut result = assemble(&cut.root, &cut.selection, &cut.layout, &reader)?;
    result.corpus.sources = cut.files.iter().map(|(path, raw)| KnowledgeSource {
        id: path.clone(), kind: "source-file".into(), locator: format!("repo:{path}"),
        version_digest: digest(raw), basis: "declared".into(), title: {
            let lines = std::str::from_utf8(raw).map_or(0, |text| text.lines().count());
            let range = cut.selection.ranges.get(path).copied().unwrap_or(SourceLineRange { start_line: 1, end_line: lines.min(240) });
            format!("{path} [selected lines {}-{}/{}; chunks at most 16KiB]", range.start_line, range.end_line, lines)
        }, status: "unknown".into(),
    }).collect();
    for source in &mut result.corpus.sources {
        let scope = result.corpus.subjects.iter().find(|subject| {
            subject.sources.iter().any(|link| link.source_id == source.id && link.role.as_str() == "metadata")
        }).and_then(|subject| subject.publication_scope.as_deref());
        source.status = match scope { Some("Current") => "current", Some("History") => "historical", _ => "unknown" }.into();
    }
    excerpts(&mut result, cut)?;
    if result.corpus.subjects.is_empty() {
        result.unknowns.push("no owner-native development concept declared in the selected source population".into());
    } else {
        result.corpus.validate().map_err(|error| anyhow::anyhow!(error.to_string()))?;
    }
    result.unknowns.sort(); result.unknowns.dedup(); result.omissions.sort(); result.omissions.dedup();
    Ok(result)
}

#[derive(Clone)]
struct DocumentFact {
    metadata: BTreeMap<String, metadata::Value>,
    role: DocumentRole,
    owner: String,
    scope: Option<String>,
    digest: String,
    topics: Vec<String>,
}
struct Builder<'a> {
    root: &'a Path,
    selection: &'a SourceSelection,
    layout: &'a Layout,
    reader: &'a dyn Fn(&str) -> Result<Vec<u8>>,
    policy: lifecycle::Policy,
    context_loaded: bool,
    policy_text: String,
    site: site::Configuration,
    facts: BTreeMap<String, DocumentFact>,
    subjects: BTreeMap<String, KnowledgeSubject>,
    edges: BTreeMap<String, KnowledgeEdge>,
    productions: BTreeMap<String, KnowledgeProduction>,
    governance: BTreeSet<(String, String)>,
    ranges: Vec<SubjectRange>,
    unknowns: Vec<String>,
    omissions: Vec<String>,
}
fn new_subject(kind: SubjectKind, native: &str, owner: &str, label: &str) -> KnowledgeSubject {
    KnowledgeSubject { id: format!("{}:{native}", kind.as_str()), kind, owner: owner.into(), native_id: native.into(),
        label: label.into(), basis: "native".into(), native_state: None, state_basis: None,
        publication_scope: None, document_role: None, native_version: None, evidence: None,
        sources: Vec::new(), topics: Vec::new() }
}
fn contribute(subject: &mut KnowledgeSubject, source: &str, role: &str) -> Result<()> {
    let link = SubjectSource { source_id: source.into(), role: parsed(role)? };
    if !subject.sources.contains(&link) { subject.sources.push(link); }
    ensure!(subject.sources.len() <= 8, "native contributor bound exceeded: {}", subject.id);
    Ok(())
}
impl Builder<'_> {
    fn document_context(&mut self) -> Result<()> {
        if self.context_loaded { return Ok(()); }
        if self.layout.site { self.site = toml::from_str(&self.text(SITE)?)?; }
        if self.layout.policy {
            ensure!(self.layout.site, "lifecycle interpretation requires the captured site policy");
            self.policy_text = self.text(POLICY)?;
            self.policy = lifecycle::load_policy(self.root, Some(self.reader))?;
        }
        self.context_loaded = true;
        Ok(())
    }
    fn read(&self, path: &str) -> Result<Vec<u8>> { (self.reader)(path) }
    fn text(&self, path: &str) -> Result<String> { Ok(String::from_utf8(self.read(path)?)?) }
    fn add(&mut self, mut subject: KnowledgeSubject) -> Result<String> {
        for value in [&subject.id, &subject.native_id, &subject.owner, &subject.label] {
            ensure!(!value.trim().is_empty() && value.len() <= 4096, "native subject string bound exceeded");
        }
        if let Some(old) = self.subjects.get_mut(&subject.id) {
            ensure!(old.kind == subject.kind && old.owner == subject.owner && old.native_id == subject.native_id
                && old.native_state == subject.native_state && old.native_version == subject.native_version,
                "conflicting owner-native subject: {}", subject.id);
            for link in subject.sources { contribute(old, &link.source_id, link.role.as_str())?; }
            return Ok(old.id.clone());
        }
        ensure!(self.subjects.len() < self.selection.bounds.max_subjects, "native subject bound exhausted");
        subject.sources.sort_by(|left, right| (&left.source_id, left.role.as_str()).cmp(&(&right.source_id, right.role.as_str())));
        let id = subject.id.clone(); self.subjects.insert(id.clone(), subject); Ok(id)
    }
    fn policies(&self, subject: &mut KnowledgeSubject) -> Result<()> {
        if self.layout.policy { contribute(subject, POLICY, "interpretation_policy")?; }
        if self.layout.site { contribute(subject, SITE, "interpretation_policy")?; }
        Ok(())
    }
    fn edge(&mut self, from: &str, to: &str, relation: &str) -> Result<()> {
        if from == to { return Ok(()); }
        if !self.subjects.contains_key(from) || !self.subjects.contains_key(to) {
            self.unknowns.push(format!("declared {relation} endpoint unavailable: {from} -> {to}"));
            return Ok(());
        }
        let id = serde_json::to_string(&(relation, from, to))?;
        self.edges.insert(id.clone(), KnowledgeEdge { id, from: from.into(), to: to.into(), relation: parsed(relation)?, basis: "declared".into() });
        ensure!(self.edges.len() <= 2048, "typed relationship bound exhausted"); Ok(())
    }
    fn document(&mut self, path: &str) -> Result<Option<String>> {
        if !path.ends_with(".md") { return Ok(None); }
        if let Some(fact) = self.facts.get(path) {
            return Ok(Some(if fact.role.as_str() == "decision" {
                format!("decision:{}", string(&fact.metadata, "id").unwrap_or(path))
            } else { format!("document:{path}") }));
        }
        self.document_context()?;
        let raw = self.read(path)?;
        let document = metadata::read_with_policy(&raw, path, false, Some(path), self.root, Some(&self.policy_text))?;
        let interpreted = lifecycle::interpreted_with_reader(self.root, &self.root.join(path), &document,
            &self.policy, &self.site, Some(self.reader))?;
        let mut values = document.metadata.clone(); values.extend(interpreted);
        for key in ["status", "title"] {
            ensure!(values.get(key).is_none_or(|value| value.as_str().is_some()), "{path}: {key} must be a string");
        }
        let role: DocumentRole = parsed(string(&values, "doc_role").unwrap_or("reference"))?;
        let owner = string(&values, "doc_owner").unwrap_or(path).to_owned();
        if !values.contains_key("doc_owner") {
            self.unknowns.push(format!("document semantic owner unavailable: {path}"));
        }
        let scope = if self.layout.site && path.starts_with("docs/") {
            site::document_scope(Path::new(&path[5..]), string(&values, "status").unwrap_or(""), &self.site, None)?
        } else { None };
        let topics = strings(&values, "doc_topics")?;
        ensure!(topics.len() <= 16, "native topic bound exceeded: {path}");
        let fact = DocumentFact { metadata: values, role, owner, scope, digest: digest(&raw), topics };
        self.facts.insert(path.into(), fact.clone());
        let kind = if role.as_str() == "decision" { parsed("decision")? } else { parsed("document")? };
        let native = if role.as_str() == "decision" {
            string(&fact.metadata, "id").context("decision has no authored native ID")?
        } else { path };
        if role.as_str() == "decision" { ensure!(string(&fact.metadata, "status").is_some(), "decision has no native status"); }
        let subject = self.document_subject(&fact, kind, native, string(&fact.metadata, "title").unwrap_or(native), path)?;
        let id = self.add(subject)?;
        let (owner, fragment) = fact.owner.split_once('#').map_or((fact.owner.as_str(), None), |(owner, fragment)| (owner, Some(fragment)));
        if owner != path && !owner.starts_with("git:") && !owner.starts_with("https:") {
            match self.document(owner) {
                Ok(Some(owner_id)) if fragment.is_none() => self.edge(&owner_id, &id, "owns")?,
                Ok(_) => self.unknowns.push(format!("document owner fragment retained; typed fragment relationship undeclared: {}", fact.owner)),
                Err(error) => self.omissions.push(format!("document owner unavailable: {owner}: {error}")),
            }
        }
        Ok(Some(id))
    }
    fn document_subject(&self, fact: &DocumentFact, kind: SubjectKind, native: &str, label: &str, path: &str) -> Result<KnowledgeSubject> {
        let mut subject = new_subject(kind, native, &fact.owner, label);
        subject.native_version = Some(fact.digest.clone());
        subject.native_state = string(&fact.metadata, "status").map(str::to_owned);
        subject.state_basis = subject.native_state.as_ref().map(|_| "authored document metadata".into());
        subject.publication_scope = fact.scope.clone(); subject.document_role = Some(fact.role);
        subject.topics = fact.topics.iter().map(|topic| SubjectTopic { topic: topic.clone(), source_id: path.into() }).collect();
        contribute(&mut subject, path, "metadata")?; self.policies(&mut subject)?;
        Ok(subject)
    }
}
fn string<'a>(values: &'a BTreeMap<String, metadata::Value>, key: &str) -> Option<&'a str> {
    values.get(key).and_then(metadata::Value::as_str)
}
fn strings(values: &BTreeMap<String, metadata::Value>, key: &str) -> Result<Vec<String>> {
    let Some(value) = values.get(key) else { return Ok(Vec::new()); };
    let metadata::Value::Sequence(values) = value else { bail!("{key} must be a string sequence"); };
    values.iter().map(|value| value.as_str().map(str::to_owned).context("metadata sequence member must be a string")).collect()
}

impl Builder<'_> {
    fn decisions(&mut self) -> Result<()> {
        let decisions: Vec<_> = self.facts.values().filter(|fact| fact.role.as_str() == "decision").cloned().collect();
        for fact in decisions {
            let native = string(&fact.metadata, "id").context("decision ID absent")?;
            let subject = format!("decision:{native}");
            for previous in strings(&fact.metadata, "supersedes")? {
                self.edge(&format!("decision:{previous}"), &subject, "superseded_by")?;
            }
            if let Some(next) = string(&fact.metadata, "superseded-by").filter(|next| !next.is_empty()) {
                self.edge(&subject, &format!("decision:{next}"), "superseded_by")?;
            }
        }
        Ok(())
    }
    fn sections(&mut self, selected: &BTreeSet<String>, identities: &BTreeSet<String>) -> Result<()> {
        if selected.is_empty() && identities.is_empty() { return Ok(()); }
        if self.layout.sections.is_empty() {
            self.unknowns.push("stable contract-section owner index unavailable".into()); return Ok(());
        }
        let paths: Vec<_> = self.layout.sections.iter().map(|path| self.root.join(path)).collect();
        let reader = |path: &Path| -> Result<String> {
            let relative = path.strip_prefix(self.root)?.to_str().context("non-UTF8 section path")?;
            self.text(relative)
        };
        let owners = adr::section_owners_from_sources(&paths, &reader)?;
        let numbered = Regex::new(r"^#{2,4}\s+(?:([0-9]+(?:\.[0-9]+)*)|([A-Z][0-9]+))\.?\s")?;
        for identity in identities {
            if !owners.contains_key(identity) { self.unknowns.push(format!("stable contract-section owner unavailable: {identity}")); }
        }
        for (native, (path, heading)) in owners {
            let relative = path.strip_prefix(self.root)?.to_str().context("non-UTF8 owner path")?.replace('\\', "/");
            if !selected.contains(&relative) && !identities.contains(&native) { continue; }
            self.document(&relative)?;
            let fact = self.facts.get(&relative).context("section document unavailable")?.clone();
            if fact.role.as_str() != "contract" {
                self.unknowns.push(format!("stable section owner is not classified as a contract: {relative}")); continue;
            }
            let mut subject = self.document_subject(&fact, parsed("contract")?, &native, heading.trim_start_matches(['#', ' ']), &relative)?;
            contribute(&mut subject, BLUEPRINT, "interpretation_policy")?;
            let id = self.add(subject)?;
            self.edge(&format!("document:{relative}"), &id, "contains")?;
            let text = self.text(&relative)?;
            let headings: Vec<_> = markdown_headings(&text).into_iter().filter(|heading| numbered.is_match(heading.source)).collect();
            let matches: Vec<_> = headings.iter().enumerate().filter(|(_, candidate)| candidate.source == heading).collect();
            ensure!(matches.len() == 1, "section heading source cut differs from owner index: {native}");
            let (index, heading) = matches[0];
            let range = SubjectRange { subject_id: id, source_id: relative, native_id: native,
                range: SourceLineRange { start_line: heading.line + 1,
                    end_line: headings.get(index + 1).map_or_else(|| text.lines().count(), |next| next.line) } };
            if !self.ranges.contains(&range) { self.ranges.push(range); }
        }
        Ok(())
    }
    fn packages(&mut self, documents: &BTreeSet<String>) -> Result<()> {
        if documents.is_empty() { return Ok(()); }
        if !self.layout.policy {
            self.unknowns.push("native plan package bindings unavailable; no state inferred from prose or headings".into());
            return Ok(());
        }
        let rows = lifecycle::aggregate(self.root, &self.policy, Some(documents), Some(self.reader))?;
        let by_id: BTreeMap<_, _> = rows.iter().map(|row| (row.id.as_str(), row)).collect();
        let packages: Vec<_> = rows.iter().filter(|row| ["package", "packet"].contains(&row.kind.as_str())).collect();
        if packages.is_empty() {
            self.unknowns.push("no bound native package rows; no state inferred from prose or headings".into());
        }
        for row in &packages {
            self.document(&row.document)?;
            let fact = self.facts.get(&row.document).context("native package metadata absent")?.clone();
            ensure!(fact.role.as_str() == "plan", "native package source must retain plan role");
            let mut subject = self.document_subject(&fact, parsed("plan_package")?, &row.id,
                row.label.as_deref().or(row.native_id.as_deref()).unwrap_or(&row.id), &row.document)?;
            subject.owner = row.owner.as_deref().unwrap_or(&row.document).to_owned();
            subject.native_state = Some(row.state.clone()); subject.state_basis = Some(row.status_basis.clone());
            contribute(&mut subject, &row.document, "native_scope")?;
            let mut chain = BTreeSet::new(); let mut current = row.id.as_str();
            loop {
                ensure!(chain.insert(current), "cyclic native status owner chain");
                let owner = by_id.get(current).context("canonical native status owner chain unavailable")?;
                self.read(&owner.document)?;
                contribute(&mut subject, &owner.document, "native_scope")?;
                let next = owner.relationships.iter().find(|edge| edge.kind == "status-owned-by");
                if let Some(next) = next { current = &next.target; }
                else { ensure!(row.status_owner.as_deref().unwrap_or(&row.id) == current, "native canonical status owner differs from chain"); break; }
            }
            let id = self.add(subject)?;
            self.edge(&format!("document:{}", row.document), &id, "contains")?;
            if let Some(line) = row.source_line {
                self.ranges.push(SubjectRange { subject_id: id, source_id: row.document.clone(), native_id: row.id.clone(),
                    range: SourceLineRange { start_line: line, end_line: line } });
            }
        }
        for row in packages {
            for edge in &row.relationships {
                let relation = match edge.kind.as_str() {
                    "depends-on" => "depends_on", "status-owned-by" => "status_owned_by", "superseded-by" => "superseded_by", _ => continue,
                };
                if by_id.get(edge.target.as_str()).is_some_and(|target| !["package", "packet"].contains(&target.kind.as_str())) {
                    self.omissions.push(format!("native package endpoint kind has no typed adapter: {}", edge.target));
                }
                self.edge(&format!("plan_package:{}", row.id), &format!("plan_package:{}", edge.target), relation)?;
            }
        }
        Ok(())
    }
    fn cargo_declarations(&mut self, path: &str) -> Result<()> {
        if Path::new(path).file_name().is_none_or(|name| name != "Cargo.toml") { return Ok(()); }
        let raw = self.read(path)?;
        let document: toml::Value = toml::from_str(std::str::from_utf8(&raw)?)?;
        let mut declarations = Vec::new();
        collect_dependencies(&document, "", &mut declarations);
        for (location, name, value) in declarations {
            let native = format!("cargo:{path}#{location}.{name}");
            let mut subject = new_subject(parsed("declaration")?, &native, path, &format!("{name} = {}", value.to_string().trim()));
            subject.basis = "declared".into(); subject.native_version = Some(digest(&raw));
            contribute(&mut subject, path, "definition")?;
            self.add(subject)?;
        }
        self.unknowns.push(format!("Cargo declarations at {path} do not establish resolved features, target activation or actual API use; no source-bound owner artifact supplied"));
        Ok(())
    }
    fn productions(&mut self) -> Result<BTreeSet<String>> {
        let mut sections = BTreeSet::new();
        let selected: BTreeSet<_> = self.selection.paths.iter().map(String::as_str).collect();
        for carrier in &self.layout.producers {
            let document: Producer = match self.text(carrier).and_then(|text| Ok(serde_json::from_str(&text)?)) {
                Ok(document) => document,
                Err(error) => { self.omissions.push(format!("producer declarations unavailable: {carrier}: {error}")); continue; }
            };
            ensure!(document.schema_version == 1 && !document.producer.trim().is_empty() && document.producer.len() <= 4096,
                "unknown producer relationship version or identity: {carrier}");
            for row in document.relationships {
                if !selected.contains(carrier.as_str()) && !row.source_path.as_deref().is_some_and(|path| selected.contains(path))
                    && !selected.contains(row.generator_path.as_str()) && !selected.contains(row.output_path.as_str()) { continue; }
                ensure!(row.contract_sections.len() <= 16 && row.contract_sections.iter().all(|section| !section.is_empty() && section.len() <= 4096),
                    "producer contract sections require finite stable identities");
                ensure!(row.basis == "declared", "producer relationship basis is not declared");
                let declaration_native = format!("{}:{}", row.declaration_kind, row.declaration_key);
                let mut declaration = new_subject(parsed("declaration")?, &declaration_native,
                    row.source_path.as_deref().unwrap_or(carrier), &row.declaration_key);
                declaration.basis = "declared".into(); declaration.native_version = row.source_identity.clone();
                let generator_native = format!("{}#{}", row.generator_path, row.generator_symbol);
                let mut generator = new_subject(parsed("generator")?, &generator_native, &row.generator_path, &generator_native);
                let mut artifact = new_subject(parsed("generated_artifact")?, &row.output_path, "generated-artifact", &row.output_path);
                for subject in [&mut declaration, &mut generator, &mut artifact] {
                    subject.basis = "declared".into(); contribute(subject, carrier, "producer_declaration")?;
                }
                for (subject, path) in [(&mut generator, &row.generator_path), (&mut artifact, &row.output_path)] {
                    match self.read(path) {
                        Ok(raw) => { subject.native_version = Some(digest(&raw)); contribute(subject, path, "definition")?; }
                        Err(error) => self.omissions.push(format!("producer contributor unavailable: {path}: {error}")),
                    }
                }
                if let Some(path) = &row.source_path {
                    match self.read(path) {
                        Ok(_) => contribute(&mut declaration, path, match row.source_role.as_str() {
                            "catalogue_registration" | "module_registration" => "catalogue_registration", "type_definition" | "definition" => "definition", _ => "producer_declaration",
                        })?,
                        Err(error) => self.omissions.push(format!("declaration contributor unavailable: {path}: {error}")),
                    }
                } else {
                    self.unknowns.push(format!("semantic declaration source location unavailable: {declaration_native}"));
                }
                let declaration_id = self.add(declaration)?;
                let generator_id = self.add(generator)?;
                let artifact_id = self.add(artifact)?;
                let direction = row.direction.as_deref().map(parsed).transpose()?;
                let id = digest(&serde_json::to_vec(&(&declaration_id, &generator_id, &artifact_id, &row.direction))?);
                self.productions.insert(id.clone(), KnowledgeProduction { id, declaration_id, generator_id,
                    artifact_id: artifact_id.clone(), direction, basis: "declared".into() });
                ensure!(self.productions.len() <= 1024, "production triple bound exhausted");
                self.governance.extend(row.contract_sections.iter().map(|section| (section.clone(), artifact_id.clone())));
                sections.extend(row.contract_sections);
            }
            self.unknowns.push(format!("producer declaration fingerprint freshness against current registry is unverified: {}", document.producer));
        }
        Ok(sections)
    }
}
fn collect_dependencies(document: &toml::Value, prefix: &str, rows: &mut Vec<(String, String, toml::Value)>) {
    let Some(table) = document.as_table() else { return; };
    for (key, value) in table {
        let location = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
        if ["dependencies", "dev-dependencies", "build-dependencies"].contains(&key.as_str()) {
            if let Some(dependencies) = value.as_table() {
                rows.extend(dependencies.iter().map(|(name, value)| (location.clone(), name.clone(), value.clone())));
            }
        } else if (prefix.is_empty() && ["workspace", "target"].contains(&key.as_str())) || prefix == "target" {
            collect_dependencies(value, &location, rows);
        }
    }
}
#[derive(Deserialize)]
struct Producer {
    schema_version: u32,
    producer: String,
    relationships: Vec<SourceRelationship>,
}
fn assemble(root: &Path, selection: &SourceSelection, layout: &Layout,
    reader: &dyn Fn(&str) -> Result<Vec<u8>>) -> Result<SourceCorpusBuild> {
    let mut builder = Builder { root, selection, layout, reader, policy: lifecycle::Policy::default(),
        context_loaded: false, policy_text: String::new(),
        site: site::Configuration { tools: BTreeMap::new(), publication: site::Publication::default(),
            collections: Vec::new(), overrides: Vec::new(), asset_bundles: Vec::new() },
        facts: BTreeMap::new(), subjects: BTreeMap::new(), edges: BTreeMap::new(), productions: BTreeMap::new(), governance: BTreeSet::new(),
        ranges: Vec::new(), unknowns: Vec::new(), omissions: Vec::new() };
    for path in &selection.paths { builder.document(path)?; builder.cargo_declarations(path)?; }
    let sections = builder.productions()?;
    let contracts = builder.facts.iter().filter(|(path, fact)| fact.role.as_str() == "contract" && selection.paths.contains(path))
        .map(|(path, _)| path.clone()).collect();
    builder.sections(&contracts, &sections)?;
    // Preserve the exact selected producer row's output correspondence.
    for (section, artifact) in std::mem::take(&mut builder.governance) {
        builder.edge(&format!("contract:{section}"), &artifact, "governs")?;
    }
    let plans = builder.facts.iter().filter(|(_, fact)| fact.role.as_str() == "plan").map(|(path, _)| path.clone()).collect();
    builder.packages(&plans)?;
    builder.decisions()?;
    Ok(SourceCorpusBuild { corpus: KnowledgeCorpus { version: 2, expected_active: selection.expected_active.clone(),
        sources: Vec::new(), subjects: builder.subjects.into_values().collect(), excerpts: Vec::new(),
        edges: builder.edges.into_values().collect(), productions: builder.productions.into_values().collect() },
        unknowns: builder.unknowns, omissions: builder.omissions, ranges: builder.ranges })
}
fn excerpts(result: &mut SourceCorpusBuild, cut: &SourceCut) -> Result<()> {
    ensure!(cut.selection.ranges.keys().all(|path| cut.files.contains_key(path)), "excerpt range names an uncaptured contributor");
    for (path, raw) in &cut.files {
        let text = match std::str::from_utf8(raw) { Ok(text) => text, Err(_) => {
            result.omissions.push(format!("source excerpt is not UTF8: {path}")); continue;
        } };
        let lines: Vec<_> = text.lines().collect();
        if lines.is_empty() { continue; }
        let range = cut.selection.ranges.get(path).copied().unwrap_or(SourceLineRange { start_line: 1, end_line: lines.len().min(240) });
        ensure!(range.start_line > 0 && range.start_line <= range.end_line && range.end_line <= lines.len()
            && range.end_line - range.start_line < 640, "invalid selected excerpt range: {path}");
        let document = result.corpus.subjects.iter().find(|subject| ["document", "decision"].contains(&subject.kind.as_str())
            && subject.sources.iter().any(|source| source.source_id == *path && source.role.as_str() == "metadata"));
        let mut selections = vec![(document.map(|subject| subject.id.clone()), range)];
        for native in &result.ranges {
            if native.source_id != *path { continue; }
            let intersection = SourceLineRange { start_line: range.start_line.max(native.range.start_line),
                end_line: range.end_line.min(native.range.end_line) };
            if intersection.start_line <= intersection.end_line { selections.push((Some(native.subject_id.clone()), intersection)); }
        }
        for (subject_id, range) in selections {
            for start in (range.start_line - 1..range.end_line).step_by(40) {
                let chunk = lines[start..(start + 40).min(range.end_line)].join("\n");
                if chunk.trim().is_empty() { continue; }
                let mut end = chunk.len().min(16000);
                while !chunk.is_char_boundary(end) { end -= 1; }
                if end < chunk.len() { result.omissions.push(format!("source excerpt truncated at 16KiB: {path}:L{}", start + 1)); }
                let id = format!("{path}#{}:L{}", subject_id.as_deref().unwrap_or("unattributed"), start + 1);
                result.corpus.excerpts.push(KnowledgeExcerpt { id, source_id: path.clone(), subject_id: subject_id.clone(),
                    text: chunk[..end].into(), locator: format!("repo:{path}#L{}", start + 1), basis: "text".into() });
                ensure!(result.corpus.excerpts.len() <= 1024, "source excerpt bound exhausted");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "workspace_sources_tests.rs"]
mod tests;
