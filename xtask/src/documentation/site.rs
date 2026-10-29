// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit Markdown publication and third-party mdBook/Pagefind execution.
#![allow(
    clippy::print_stdout,
    clippy::disallowed_types,
    reason = "xtask publication command owns terminal diagnostics and dynamic tooling errors"
)]
use super::{
    adr,
    headings::markdown_headings,
    metadata::{self, Value},
};
use anyhow::{Context, Result, ensure};
use clap::ValueEnum;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// Publication choices owned by docs/site.toml.
#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    /// Exact independently provisioned tool versions.
    #[serde(default)]
    pub tools: BTreeMap<String, String>,
    /// Explicit entrypoints, assets and inclusion rules.
    pub publication: Publication,
    /// Ordered published Markdown collections.
    pub collections: Vec<Collection>,
    /// Later matching scope overrides take precedence.
    #[serde(default)]
    pub overrides: Vec<ScopeOverride>,
    /// Named finite attachment sets.
    #[serde(default)]
    pub asset_bundles: Vec<AssetBundle>,
}
/// Selected publication content; retaining a file never implicitly publishes it.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Publication {
    /// Excluded docs-relative patterns.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Current work docs-relative patterns.
    #[serde(default)]
    pub current_work: Vec<String>,
    /// Required included entrypoints.
    #[serde(default)]
    pub entrypoints: Vec<String>,
    /// Individual finite attachment paths.
    #[serde(default)]
    pub assets: Vec<String>,
    /// Names of explicitly selected finite bundles.
    #[serde(default)]
    pub asset_bundles: Vec<String>,
}
fn yes() -> bool {
    true
}
/// One independently owned Markdown collection.
#[derive(Clone, Debug, Deserialize)]
pub struct Collection {
    /// Navigation label.
    pub title: String,
    /// Relative directory under docs.
    pub root: String,
    /// Include descendants rather than only immediate files.
    #[serde(default = "yes")]
    pub recursive: bool,
    /// Default search scope.
    pub scope: String,
}
/// Pattern-based search-scope override.
#[derive(Clone, Debug, Deserialize)]
pub struct ScopeOverride {
    /// Docs-relative matching patterns.
    pub patterns: Vec<String>,
    /// Chosen search scope.
    pub scope: String,
}
/// Named finite source attachment bundle.
#[derive(Clone, Debug, Deserialize)]
pub struct AssetBundle {
    /// Unique bundle identity.
    pub name: String,
    /// Docs-relative containment root.
    pub root: String,
    /// Explicit bundle-relative files.
    pub files: Vec<String>,
}
/// A selected source page after strict frontmatter removal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    /// Docs-relative delivery path.
    pub path: PathBuf,
    /// Authored title or first unfenced H1.
    pub title: String,
    /// Original Markdown body.
    pub body: String,
    /// Collection navigation label.
    pub collection: String,
    /// Current/Reference/History filter.
    pub scope: String,
    /// Native authored status, without inferred promotion.
    pub status: String,
}
/// Third-party publication operations, never ordinary implicit preparation.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SiteCommand {
    /// Build and atomically publish the complete site.
    Build,
    /// Build and run Pagefind's server.
    Serve,
    /// Explicitly install exact tool versions.
    Install,
    /// Print exact standalone tool specifications.
    ToolSpecs,
}

/// Read the sole site declaration without executing or preparing a tool.
/// # Errors
/// Invalid configuration or inaccessible file.
pub fn configuration(root: &Path) -> Result<Configuration> {
    Ok(toml::from_str(&fs::read_to_string(
        root.join("docs/site.toml"),
    )?)?)
}

/// Match Python fnmatch-style patterns with wildcard separators and literal braces.
/// # Errors
/// Malformed wildcard character classes.
pub fn matches(path: &str, patterns: &[String]) -> Result<bool> {
    for pattern in patterns {
        // Python fnmatch has no globstar or alternation operator. Normalize those
        // two supplier extensions before using its maintained matching engine.
        let mut normalized = String::new();
        let mut star = false;
        let mut class = false;
        for ch in pattern.chars() {
            if ch == '*' && !class {
                if !star {
                    normalized.push(ch);
                }
                star = true;
                continue;
            }
            star = false;
            match ch {
                '[' => {
                    class = true;
                    normalized.push(ch);
                }
                ']' => {
                    class = false;
                    normalized.push(ch);
                }
                '{' | '}' if !class => {
                    normalized.push('[');
                    normalized.push(ch);
                    normalized.push(']');
                }
                _ => normalized.push(ch),
            }
        }
        let matcher = globset::GlobBuilder::new(&normalized)
            .literal_separator(false)
            .backslash_escape(false)
            .allow_unclosed_class(true)
            .build()?
            .compile_matcher();
        if matcher.is_match(path) {
            return Ok(true);
        }
    }
    Ok(false)
}
/// Resolve existing symlinks while normalizing missing destination components.
/// # Errors
/// Current-directory or symlink resolution failures.
pub(crate) fn resolved(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut result = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => {
                result.push(component.as_os_str());
                if fs::symlink_metadata(&result).is_ok_and(|m| m.file_type().is_symlink()) {
                    result = fs::canonicalize(&result)?;
                }
            }
        }
    }
    Ok(result)
}
/// Resolve a docs-relative path and enforce containment through symlink ancestors.
/// # Errors
/// The path escapes its selected root or cannot be resolved.
pub fn checked_path(docs: &Path, relative: &Path) -> Result<PathBuf> {
    let path = docs.join(relative);
    ensure!(
        resolved(&path)?.starts_with(resolved(docs)?),
        "publication path escapes docs: {}",
        relative.display()
    );
    Ok(path)
}
pub(crate) fn walk_files(
    dir: &Path,
    recursive: bool,
    extension: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            if recursive {
                paths.extend(walk_files(&path, true, extension)?);
            }
        } else if extension.is_none_or(|ext| path.extension().is_some_and(|value| value == ext)) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}
fn valid_scope(scope: &str) -> bool {
    ["Current", "Reference", "History"].contains(&scope)
}
/// Determine native publication scope without inferring authority from arbitrary status.
/// # Errors
/// Ambiguous collections, malformed patterns or invalid selected scope.
pub fn document_scope(
    relative: &Path,
    status: &str,
    config: &Configuration,
    collection: Option<&Collection>,
) -> Result<Option<String>> {
    let relative_text = relative.to_string_lossy().replace('\\', "/");
    if matches(&relative_text, &config.publication.exclude)? {
        return Ok(None);
    }
    let collection = if let Some(collection) = collection {
        collection
    } else {
        let mut selected = Vec::new();
        for candidate in &config.collections {
            let base = Path::new(&candidate.root);
            ensure!(
                !base.is_absolute()
                    && !base.components().any(|c| matches!(c, Component::ParentDir)),
                "publication collection escapes docs"
            );
            let base = if candidate.root == "." {
                Path::new("")
            } else {
                base
            };
            if let Ok(within) = relative.strip_prefix(base)
                && (candidate.recursive || within.components().count() == 1)
            {
                selected.push(candidate);
            }
        }
        ensure!(
            selected.len() <= 1,
            "duplicate publication path: {relative_text}"
        );
        let Some(&collection) = selected.first() else {
            return Ok(None);
        };
        collection
    };
    let mut scope = collection.scope.clone();
    if relative
        .components()
        .next()
        .is_some_and(|part| part.as_os_str() == "adr")
        && relative.file_name().is_some_and(|name| {
            Regex::new(r"^[0-9]{4}-").is_ok_and(|re| re.is_match(&name.to_string_lossy()))
        })
    {
        scope = match status {
            "accepted" => "Current",
            "superseded" | "deprecated" | "rejected" => "History",
            _ => "Reference",
        }
        .to_owned();
    }
    if matches(&relative_text, &config.publication.current_work)?
        || config.publication.entrypoints.contains(&relative_text)
    {
        scope = "Current".to_owned();
    }
    for override_ in &config.overrides {
        if matches(&relative_text, &override_.patterns)? {
            scope = override_.scope.clone();
        }
    }
    ensure!(
        valid_scope(&scope),
        "{relative_text}: invalid search scope {scope:?}"
    );
    Ok(Some(scope))
}
/// Discover selected Markdown without running subprocesses or editing navigation.
/// # Errors
/// Invalid paths, missing titles, conflicting collections or missing entrypoints.
pub fn discover(root: &Path, config: &Configuration) -> Result<Vec<Page>> {
    let docs = root.join("docs");
    let mut pages = Vec::new();
    let mut seen = BTreeSet::new();
    for collection in &config.collections {
        let base = checked_path(&docs, Path::new(&collection.root))?;
        ensure!(base.is_dir(), "missing collection: {}", base.display());
        let mut paths = walk_files(&base, collection.recursive, Some("md"))?;
        paths.sort_by_key(|path| {
            (
                path.file_name().is_none_or(|name| name != "README.md"),
                path.clone(),
            )
        });
        for path in paths {
            let relative = path.strip_prefix(&docs)?.to_owned();
            let relative_text = relative.to_string_lossy().replace('\\', "/");
            checked_path(&docs, &relative)?;
            if matches(&relative_text, &config.publication.exclude)? {
                continue;
            }
            ensure!(
                seen.insert(relative.clone()),
                "duplicate publication path: {}",
                relative.display()
            );
            let raw = fs::read(&path)?;
            let identity = format!("docs/{relative_text}");
            let document = metadata::read(&raw, &identity, false, Some(&identity), root)?;
            let body = std::str::from_utf8(document.body)?.to_owned();
            let heading = markdown_headings(&body)
                .into_iter()
                .find_map(|heading| heading.source.strip_prefix("# ").map(str::trim))
                .unwrap_or("");
            let title = match document.metadata.get("title") {
                None | Some(Value::Null) => heading,
                Some(Value::String(value)) if value.is_empty() => heading,
                Some(Value::Bool(false)) => heading,
                Some(Value::Integer(0) | Value::Unsigned(0)) => heading,
                Some(value) => value.as_str().context(format!(
                    "{relative_text}: supply a scalar title or first H1"
                ))?,
            };
            ensure!(
                !title.trim().is_empty(),
                "{relative_text}: supply a scalar title or first H1"
            );
            let status = match document.metadata.get("status") {
                None => "",
                Some(value) => value
                    .as_str()
                    .context(format!("{relative_text}: status must be a string"))?,
            };
            let scope = document_scope(&relative, status, config, Some(collection))?
                .context("selected publication has no scope")?;
            pages.push(Page {
                path: relative,
                title: title.to_owned(),
                body,
                collection: collection.title.clone(),
                scope,
                status: status.to_owned(),
            });
        }
    }
    for entry in &config.publication.entrypoints {
        ensure!(
            seen.contains(Path::new(entry)),
            "entrypoint is not published: {entry}"
        );
    }
    for pattern in &config.publication.current_work {
        let mut found = false;
        for page in &pages {
            found |= matches(&page.path.to_string_lossy(), std::slice::from_ref(pattern))?;
        }
        ensure!(found, "current-work pattern matches no page: {pattern}");
    }
    Ok(pages)
}
/// Select only explicitly declared finite attachments and validate their ownership.
/// # Errors
/// Missing, duplicate, symlinked, excluded, reserved or escaping attachments.
pub fn selected_assets(root: &Path, config: &Configuration) -> Result<Vec<PathBuf>> {
    let docs = root.join("docs");
    let mut selected = config.publication.assets.clone();
    let mut names = BTreeSet::new();
    for bundle in &config.asset_bundles {
        ensure!(
            names.insert(&bundle.name),
            "duplicate publication asset bundle"
        );
    }
    for name in &config.publication.asset_bundles {
        let bundle = config
            .asset_bundles
            .iter()
            .find(|bundle| &bundle.name == name)
            .with_context(|| format!("unknown publication asset bundle: {name}"))?;
        let base = checked_path(&docs, Path::new(&bundle.root))?;
        for item in &bundle.files {
            let path = base.join(item);
            ensure!(
                resolved(&path)?.starts_with(resolved(&base)?),
                "asset escapes bundle {name}: {item}"
            );
            selected.push(
                path.strip_prefix(&docs)?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    let mut assets = BTreeSet::new();
    for relative in selected {
        let relative = PathBuf::from(relative);
        ensure!(
            !relative.is_absolute()
                && !relative
                    .components()
                    .any(|part| matches!(part, Component::ParentDir)),
            "noncanonical publication asset: {}",
            relative.display()
        );
        let path = checked_path(&docs, &relative)?;
        ensure!(
            path.is_file() && !fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "missing or symlink publication asset: {}",
            relative.display()
        );
        ensure!(
            path.extension().is_none_or(|ext| ext != "md")
                && !path
                    .file_name()
                    .is_some_and(|name| name == "book.toml" || name == "SUMMARY.md"),
            "reserved publication asset: {}",
            relative.display()
        );
        let resolved_path = resolved(&path)?;
        let mut contained = false;
        for collection in &config.collections {
            let base = resolved(&checked_path(&docs, Path::new(&collection.root))?)?;
            contained |= resolved_path.starts_with(&base)
                && (collection.recursive || resolved_path.parent() == Some(base.as_path()));
        }
        ensure!(
            contained,
            "asset outside publication collections: {}",
            relative.display()
        );
        ensure!(
            !matches(&relative.to_string_lossy(), &config.publication.exclude)?,
            "selected asset is excluded: {}",
            relative.display()
        );
        ensure!(
            assets.insert(relative),
            "duplicate selected publication asset"
        );
    }
    Ok(assets.into_iter().collect())
}
/// Validate parsed Markdown links/images to explicitly selected non-Markdown assets.
/// # Errors
/// A local delivery attachment was not selected.
pub fn validate_attachments(root: &Path, pages: &[Page], assets: &[PathBuf]) -> Result<()> {
    let docs = resolved(&root.join("docs"))?;
    let selected: BTreeSet<_> = assets.iter().collect();
    let scheme = Regex::new(r"^[A-Za-z][A-Za-z0-9+.-]*:")?;
    for page in pages {
        for event in Parser::new_ext(&page.body, Options::ENABLE_TABLES) {
            let target = match event {
                Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => dest_url,
                _ => continue,
            };
            let target = target.as_ref();
            if target.is_empty() || target.starts_with('/') || scheme.is_match(target) {
                continue;
            }
            let path = target.split(['?', '#']).next().unwrap_or("");
            if path.is_empty() {
                continue;
            }
            let decoded = percent_encoding::percent_decode_str(path).decode_utf8()?;
            let path = resolved(
                &docs
                    .join(page.path.parent().unwrap_or(Path::new("")))
                    .join(decoded.as_ref()),
            )?;
            if path.starts_with(&docs) && path.extension().is_none_or(|extension| extension != "md")
            {
                let relative = path.strip_prefix(&docs)?;
                if relative.starts_with("theme") || relative.starts_with("generated/rustdoc") {
                    continue;
                }
                ensure!(
                    selected.contains(&relative.to_owned()),
                    "{}: attachment is not explicitly selected: {}",
                    page.path.display(),
                    relative.display()
                );
            }
        }
    }
    Ok(())
}
type SectionAnnotations = BTreeMap<PathBuf, BTreeMap<String, String>>;

fn architecture_index(root: &Path, pages: &[Page]) -> Result<(Page, SectionAnnotations)> {
    let owners = adr::section_owners(&root.join("docs/authoritative_design/blueprint.md"))?;
    let mut annotations = BTreeMap::<PathBuf, BTreeMap<String, String>>::new();
    let mut rows=vec!["# Architecture sections".to_owned(),String::new(),"Generated from authoritative numbered headings. See the [reading guide](authoritative_design/README.md) for current contract precedence.".to_owned(),String::new()];
    for (id, (path, heading)) in owners {
        let relative = path.strip_prefix(root.join("docs"))?.to_owned();
        let anchor = format!("section-{}", id.to_lowercase().replace('.', "-"));
        annotations
            .entry(relative.clone())
            .or_default()
            .insert(heading.clone(), anchor.clone());
        rows.push(format!(
            "- [§{id} — {}]({}#{anchor})",
            heading.trim_start_matches(['#', ' ']),
            relative.display()
        ));
    }
    ensure!(
        !pages
            .iter()
            .any(|page| page.path == Path::new("architecture-sections.md")),
        "architecture-sections.md is reserved for the derived section directory"
    );
    Ok((
        Page {
            path: "architecture-sections.md".into(),
            title: "Architecture sections".into(),
            body: rows.join("\n") + "\n",
            collection: "Architecture".into(),
            scope: "Reference".into(),
            status: String::new(),
        },
        annotations,
    ))
}
/// Materialize selected chapters/attachments without modifying authored source.
/// # Errors
/// Invalid ownership, attachment selection or staging destination.
pub fn stage(
    root: &Path,
    source: &Path,
    pages: &[Page],
    config: &Configuration,
) -> Result<Vec<Page>> {
    ensure!(
        !source.exists(),
        "staging source already exists: {}",
        source.display()
    );
    fs::create_dir_all(source)?;
    let assets = selected_assets(root, config)?;
    validate_attachments(root, pages, &assets)?;
    for relative in assets {
        let target = source.join(&relative);
        ensure!(
            resolved(&target)?.starts_with(resolved(source)?),
            "asset destination escapes staging source: {}",
            relative.display()
        );
        fs::create_dir_all(target.parent().context("asset parent absent")?)?;
        fs::copy(root.join("docs").join(relative), target)?;
    }
    let (directory, annotations) = architecture_index(root, pages)?;
    let mut pages = pages.to_vec();
    pages.push(directory);
    let mut summary = vec!["# Summary".to_owned(), String::new()];
    for collection in &config.collections {
        let mut members: Vec<_> = pages
            .iter()
            .filter(|page| page.collection == collection.title)
            .collect();
        let landing = Path::new(&collection.root).join("README.md");
        let landing = if collection.root == "." {
            PathBuf::from("README.md")
        } else {
            landing
        };
        let has_landing = members.iter().any(|page| page.path == landing);
        members.sort_by_key(|page| page.path != landing);
        summary.extend([format!("# {}", collection.title), String::new()]);
        for page in members {
            let title = page.title.replace('[', "\\[").replace(']', "\\]");
            summary.push(format!(
                "{}- [{title}]({})",
                if has_landing && page.path != landing {
                    "  "
                } else {
                    ""
                },
                page.path.display()
            ));
        }
    }
    fs::write(source.join("SUMMARY.md"), summary.join("\n") + "\n")?;
    for page in &pages {
        let mut lines: Vec<_> = page.body.split_inclusive('\n').map(str::to_owned).collect();
        for heading in markdown_headings(&page.body) {
            if let Some(anchor) = annotations
                .get(&page.path)
                .and_then(|values| values.get(heading.source))
            {
                lines[heading.line] = format!("<a id=\"{anchor}\"></a>\n\n{}", lines[heading.line]);
            }
        }
        let target = checked_path(source, &page.path)?;
        fs::create_dir_all(target.parent().context("page parent absent")?)?;
        fs::write(target, lines.concat())?;
    }
    fs::copy(root.join("docs/book.toml"), source.join("book.toml"))?;
    Ok(pages)
}
fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for path in walk_files(from, true, None)? {
        let relative = path.strip_prefix(from)?;
        let target = checked_path(to, relative)?;
        fs::create_dir_all(target.parent().context("copy parent absent")?)?;
        fs::copy(path, target)?;
    }
    Ok(())
}
/// Add scoped search metadata to already rendered mdBook pages.
/// # Errors
/// Unsupported generated HTML or failed delivery writes.
pub fn annotate(output: &Path, pages: &[Page], root: &Path) -> Result<()> {
    copy_tree(&root.join("docs/theme"), &output.join("theme"))?;
    let suggest_edit = Regex::new(r#"(?s)<a href="[^"]+"[^>]*title="Suggest an edit".*?</a>"#)?;
    for page in pages {
        let path = output.join(page.path.with_extension("html"));
        let mut text = fs::read_to_string(&path)?;
        ensure!(
            text.matches("<main>").count() == 1 && text.matches("</head>").count() == 1,
            "{}: unsupported mdBook HTML structure",
            page.path.display()
        );
        let prefix = "../".repeat(page.path.components().count().saturating_sub(1));
        let title = html_escape(&page.title);
        text=text.replacen("<main>",&format!("<main data-pagefind-body data-pagefind-filter=\"scope:{}\"><span hidden data-pagefind-ignore data-pagefind-meta=\"title\">{title}</span>",page.scope),1);
        text=text.replacen("</head>",&format!("<link rel=\"stylesheet\" href=\"{prefix}pagefind/pagefind-component-ui.css\">\n<link rel=\"stylesheet\" href=\"{prefix}theme/search.css\">\n<script type=\"module\" src=\"{prefix}theme/search.js\"></script>\n</head>"),1);
        let label = html_escape(&format!(
            "{} · {}{}",
            page.scope,
            page.collection,
            if page.status.is_empty() {
                String::new()
            } else {
                format!(" · {}", page.status)
            }
        ));
        text = text.replacen(
            "</main>",
            &format!("<p class=\"document-scope\" data-pagefind-ignore>{label}</p></main>"),
            1,
        );
        if page.path == Path::new("architecture-sections.md") {
            text = suggest_edit.replace_all(&text, "").into_owned();
        }
        fs::write(path, text)?;
    }
    if let Some(first) = pages.first().map(|page| page.path.with_extension("html"))
        && first != Path::new("index.html")
    {
        fs::write(
            output.join("index.html"),
            fs::read_to_string(output.join(first))?
                .replace("data-pagefind-body", "data-pagefind-ignore"),
        )?;
    }
    Ok(())
}
/// Require the exact declared standalone tools; never install during a build/read.
/// # Errors
/// Missing tool, failed version command or mismatched release.
pub fn check_tools(config: &Configuration) -> Result<()> {
    for (name, version) in &config.tools {
        let output = Command::new(name)
            .arg("--version")
            .output()
            .with_context(|| format!("missing {name}; run just bootstrap-docs"))?;
        ensure!(output.status.success(), "{name} --version failed");
        let text = String::from_utf8(output.stdout)?;
        let pattern = Regex::new(&format!(r"(?:^|\s)v?{}(?:\s|$)", regex::escape(version)))?;
        ensure!(
            pattern.is_match(&text),
            "expected {name} {version}, got {}; run just bootstrap-docs",
            text.trim()
        );
    }
    Ok(())
}
/// Publish only a complete candidate, restoring the previous site if rename fails.
/// # Errors
/// Rename fails; a failed restoration is retained in the error context.
pub fn publish(candidate: &Path, destination: &Path) -> Result<()> {
    let backup = candidate
        .parent()
        .context("candidate parent absent")?
        .join("previous");
    let existed = destination.exists();
    if existed {
        fs::rename(destination, &backup)?;
    }
    if let Err(error) = fs::rename(candidate, destination) {
        if existed {
            fs::rename(&backup, destination)
                .context("publication failed and previous site restoration failed")?;
        }
        return Err(error.into());
    }
    Ok(())
}
fn checked_command(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    ensure!(
        status.success(),
        "documentation tool failed: {command:?}: {status}"
    );
    Ok(())
}
/// Build the selected source once, then publish only after both tools succeed.
/// # Errors
/// Declaration/tool/staging/render/search/publication failures.
pub fn build(root: &Path) -> Result<PathBuf> {
    let config = configuration(root)?;
    check_tools(&config)?;
    let pages = discover(root, &config)?;
    let work = root.join("build/docs");
    fs::create_dir_all(&work)?;
    let scratch = tempfile::Builder::new()
        .prefix("publish-")
        .tempdir_in(work)?;
    let source = scratch.path().join("source");
    let output = scratch.path().join("site");
    let pages = stage(root, &source, &pages, &config)?;
    checked_command(
        Command::new("mdbook")
            .arg("build")
            .arg(&source)
            .arg("--dest-dir")
            .arg(&output),
    )?;
    annotate(&output, &pages, root)?;
    checked_command(Command::new("pagefind").arg("--site").arg(&output))?;
    let rustdoc = root.join("docs/generated/rustdoc");
    if rustdoc.is_dir() {
        copy_tree(&rustdoc, &output.join("generated/rustdoc"))?;
    }
    let destination = root.join("docs/book");
    publish(&output, &destination)?;
    println!(
        "docs: published {} chapters with scoped search",
        pages.len()
    );
    Ok(destination)
}
/// Execute an explicit documentation operation.
/// # Errors
/// Tool preparation, build or serving fails.
pub fn execute(root: &Path, command: SiteCommand) -> Result<()> {
    match command {
        SiteCommand::ToolSpecs => println!(
            "{}",
            configuration(root)?
                .tools
                .iter()
                .map(|(name, version)| format!("{name}@{version}"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        SiteCommand::Install => {
            if Command::new("cargo-binstall")
                .arg("--version")
                .output()
                .is_err()
            {
                checked_command(Command::new("cargo").args([
                    "install",
                    "cargo-binstall",
                    "--locked",
                ]))?;
            }
            let mut command = Command::new("cargo");
            command.args(["binstall", "--no-confirm"]);
            for (name, version) in configuration(root)?.tools {
                command.arg(format!("{name}@{version}"));
            }
            checked_command(&mut command)?;
        }
        SiteCommand::Build => {
            build(root)?;
        }
        SiteCommand::Serve => {
            let output = build(root)?;
            println!("Rebuild with just docs after edits; refresh your browser.");
            checked_command(
                Command::new("pagefind")
                    .arg("--site")
                    .arg(output)
                    .arg("--serve"),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "site_tests.rs"]
mod tests;
