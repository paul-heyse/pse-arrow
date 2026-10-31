// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scoped native metadata/index checks and the selected pinned offline link-check owner.
#![allow(
    clippy::disallowed_types,
    reason = "xtask preserves dynamic external-tool causes at its existing command boundary"
)]
use super::{adr, lifecycle, metadata, site};
use anyhow::{Context, Result, ensure};
use clap::Args;
use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild};
use pulldown_cmark::{Event, Parser, Tag};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use url::Url;

const TOOL_BYTES: usize = 16 << 20;
const SOURCE_BYTES: u64 = 16 << 20;
/// The actual native check grammar; schema/help do not select or execute a tool.
#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("docs-check-selection").required(true).multiple(false).args(["files","changed","schema"])))]
pub struct Query {
    /// Exact current Markdown files; no implicit recursive collection scan.
    pub files: Vec<PathBuf>,
    /// Current tracked/index/untracked Markdown changes relative to HEAD.
    #[arg(long)]
    pub changed: bool,
    /// Emit only the native report schema.
    #[arg(long)]
    pub schema: bool,
}
/// Classification describes the selected source's question, not permission to edit its target.
#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// A defect in a selected current source or its selected local reference.
    Scoped,
    /// External URL deliberately not visited by an offline check.
    External,
    /// A link to immutable Git evidence or natively classified historical documentation.
    Historical,
}
/// Error findings fail this check; unavailable external validation remains informational.
#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// A selected local check failed.
    Error,
    /// An explicitly unexamined external/historical transport.
    Information,
}
/// One original source location with a native or pinned-tool explanation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Diagnostic {
    /// Selected repository-relative source.
    pub file: PathBuf,
    /// One-based source line.
    pub line: usize,
    /// One-based source column.
    pub column: usize,
    /// Native stable check family.
    pub code: String,
    /// Scoped, external or historical reference context.
    pub category: Category,
    /// Errors determine the selected check result.
    pub severity: Severity,
    /// Original link target when this is a link finding.
    pub target: Option<String>,
    /// Original native/pinned-tool explanation.
    pub message: String,
}
/// Scope and actual tool evidence of one selected documentation check.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Native report format.
    pub version: u32,
    /// Exact current sources checked; an empty changed set remains empty.
    pub files: Vec<PathBuf>,
    /// Original selected lychee version output; absent if no tool executed.
    pub tool_version: Option<String>,
    /// Original normally settled checker status; absent for empty/unavailable selection.
    pub tool_exit: Option<i32>,
    /// All selected observations, including offline exclusions.
    pub diagnostics: Vec<Diagnostic>,
    /// No error in this bounded selection; not a whole-repository qualification.
    pub passed: bool,
    /// Explicit coverage and interpretation limits.
    pub limitations: Vec<String>,
}
struct Captured {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    code: i32,
}
fn bounded(mut input: impl Read, maximum: usize) -> std::io::Result<(Vec<u8>, usize)> {
    let mut prefix = Vec::new();
    let mut total = 0usize;
    let mut bytes = [0u8; 8192];
    loop {
        let count = input.read(&mut bytes)?;
        if count == 0 {
            return Ok((prefix, total));
        }
        total = total.saturating_add(count);
        let keep = count.min(maximum.saturating_sub(prefix.len()));
        prefix.extend_from_slice(&bytes[..keep]);
    }
}
fn capture(mut command: Command, maximum: usize) -> Result<Captured> {
    command.stdin(Stdio::null()).stdout(Stdio::piped());
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Park,
        Some(StderrCapture {
            bytes: 65536,
            stack_bytes: 256 << 10,
        }),
    )?;
    let stdout = child
        .take_stdout()
        .context("documentation tool stdout absent")?;
    let reader = std::thread::Builder::new()
        .name("pse-docs-tool-output".into())
        .spawn(move || bounded(stdout, maximum))?;
    let wait = child.wait_ready(None, Some(Instant::now() + Duration::from_secs(30)));
    let settled = child.cancel();
    let output = reader
        .join()
        .map_err(|_| anyhow::anyhow!("documentation tool output reader panicked"));
    let stderr = child.take_stderr_prefix().unwrap_or_default();
    let mut failures = Vec::new();
    if let Err(error) = wait {
        failures.push(format!("original tool wait: {error}"));
    }
    let status = match settled {
        Ok(status) => Some(status),
        Err(error) => {
            failures.push(format!("original tool settlement: {error}"));
            None
        }
    };
    let stdout = match output {
        Ok(Ok((bytes, total))) => {
            if total > maximum {
                failures.push(format!("tool output exceeded {maximum} bytes"));
            }
            bytes
        }
        Ok(Err(error)) => {
            failures.push(format!("original tool output: {error}"));
            Vec::new()
        }
        Err(error) => {
            failures.push(error.to_string());
            Vec::new()
        }
    };
    ensure!(
        failures.is_empty(),
        "{}; original stdout prefix: {}; original stderr prefix: {}",
        failures.join("; "),
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    let status = status.context("settled documentation tool status absent")?;
    use std::os::unix::process::ExitStatusExt as _;
    Ok(Captured {
        stdout,
        stderr,
        code: status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(1)),
    })
}
fn select_tool(root: &Path) -> Result<PathBuf> {
    let mut command = Command::new(root.join("scripts/pse-host"));
    command
        .current_dir(root)
        .args(["-C"])
        .arg(root)
        .args(["diagnostic-tools", "verify", "lychee"]);
    let selected = capture(command, 65536)?;
    ensure!(
        selected.code == 0,
        "pinned lychee is unavailable; explicitly run scripts/pse-host diagnostic-tools install lychee: {}",
        String::from_utf8_lossy(&selected.stderr)
    );
    let path = PathBuf::from(String::from_utf8(selected.stdout)?.trim_end_matches(['\r', '\n']));
    ensure!(
        path.is_absolute() && path.is_file(),
        "native lychee owner returned no absolute selected executable"
    );
    Ok(path)
}
fn selected(root: &Path, query: &Query) -> Result<Vec<PathBuf>> {
    let mut paths: BTreeSet<PathBuf> = query.files.iter().cloned().collect();
    if query.changed {
        for arguments in [
            vec!["diff", "--name-only", "--diff-filter=ACMR", "-z", "HEAD"],
            vec![
                "diff",
                "--cached",
                "--name-only",
                "--diff-filter=ACMR",
                "-z",
            ],
            vec!["ls-files", "--others", "--exclude-standard", "-z"],
        ] {
            let mut command = Command::new("git");
            command.arg("-C").arg(root).args(arguments);
            let observed = capture(command, TOOL_BYTES)?;
            ensure!(
                observed.code == 0,
                "Git source selection failed: {}",
                String::from_utf8_lossy(&observed.stderr)
            );
            for raw in observed
                .stdout
                .split(|byte| *byte == 0)
                .filter(|raw| !raw.is_empty())
            {
                use std::os::unix::ffi::OsStringExt as _;
                let path = PathBuf::from(std::ffi::OsString::from_vec(raw.to_vec()));
                if path.extension().is_some_and(|extension| extension == "md")
                    && root.join(&path).is_file()
                {
                    paths.insert(path);
                }
            }
        }
    }
    paths
        .into_iter()
        .map(|path| {
            let path = site::resolved(&root.join(path))?;
            ensure!(
                path.is_file() && path.extension().is_some_and(|extension| extension == "md"),
                "scoped docs check requires an existing Markdown file: {}",
                path.display()
            );
            ensure!(
                fs::metadata(&path)?.len() <= SOURCE_BYTES,
                "selected document exceeds16MiB: {}",
                path.display()
            );
            Ok(path
                .strip_prefix(root)
                .context("selected document escapes checkout")?
                .to_owned())
        })
        .collect()
}
#[derive(Deserialize)]
struct Status {
    text: String,
}
#[derive(Deserialize)]
struct Span {
    line: usize,
    column: usize,
}
#[derive(Deserialize)]
struct Link {
    url: String,
    status: Status,
    span: Option<Span>,
}
#[derive(Deserialize)]
struct Lychee {
    errors: usize,
    #[serde(default)]
    error_map: BTreeMap<PathBuf, Vec<Link>>,
    #[serde(default)]
    excluded_map: BTreeMap<PathBuf, Vec<Link>>,
}
fn diagnostic(
    file: &Path,
    line: usize,
    column: usize,
    code: &str,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        file: file.to_owned(),
        line,
        column,
        code: code.into(),
        category: Category::Scoped,
        severity: Severity::Error,
        target: None,
        message: message.into(),
    }
}
fn target(root: &Path, source: &Path, value: &str) -> Option<(PathBuf, Option<String>)> {
    let source = Url::from_file_path(root.join(source)).ok()?;
    let url = source.join(value).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    Some((url.to_file_path().ok()?, url.fragment().map(str::to_owned)))
}
fn historical(root: &Path, path: &Path, config: Option<&site::Configuration>) -> bool {
    let Some(config) = config else {
        return false;
    };
    let Ok(relative) = path.strip_prefix(root.join("docs")) else {
        return false;
    };
    let status = fs::read(path)
        .ok()
        .and_then(|bytes| {
            metadata::read(&bytes, "link target", false, None, root)
                .ok()
                .and_then(|document| {
                    document
                        .metadata
                        .get("status")
                        .and_then(metadata::Value::as_str)
                        .map(str::to_owned)
                })
        })
        .unwrap_or_default();
    site::document_scope(relative, &status, config, None)
        .ok()
        .flatten()
        .is_some_and(|scope| scope == "History")
}
fn native_checks(root: &Path, files: &[PathBuf], report: &mut Report) -> Result<()> {
    let pattern = regex::Regex::new(r"^#{1,6}\s+(O[0-9]{2,})\b")?;
    for file in files {
        let bytes = fs::read(root.join(file))?;
        let identity = file.to_string_lossy();
        let required = [
            "docs/plans",
            "docs/adr",
            "docs/capability-maps",
            "docs/authoritative_design",
        ]
        .iter()
        .any(|prefix| file.starts_with(prefix));
        let document = match metadata::read(&bytes, &identity, required, Some(&identity), root) {
            Ok(document) => document,
            Err(error) => {
                report
                    .diagnostics
                    .push(diagnostic(file, 1, 1, "docs.metadata", error.to_string()));
                continue;
            }
        };
        let text = std::str::from_utf8(&bytes)?;
        for (event, range) in Parser::new(text).into_offset_iter() {
            if let Event::Start(Tag::Link { dest_url, .. }) = event
                && dest_url.starts_with("git:")
            {
                let line = text[..range.start]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1;
                let valid = lifecycle::reference(root, &dest_url, None);
                report.diagnostics.push(Diagnostic {
                    file: file.clone(),
                    line,
                    column: 1,
                    code: "docs.historical_reference".into(),
                    category: Category::Historical,
                    severity: if valid.is_ok() {
                        Severity::Information
                    } else {
                        Severity::Error
                    },
                    target: Some(dest_url.to_string()),
                    message: valid.err().map_or_else(
                        || "immutable Git object was not fetched by this offline check".into(),
                        |error| error.to_string(),
                    ),
                });
            }
        }
        let mut opportunity_ids = BTreeSet::new();
        for heading in super::headings::markdown_headings(text) {
            if let Some(captures) = pattern.captures(heading.source) {
                let id = &captures[1];
                if !opportunity_ids.insert(id.to_owned())
                    || lifecycle::reference(root, &format!("#{id}"), Some(&root.join(file)))
                        .is_err()
                {
                    report.diagnostics.push(diagnostic(
                        file,
                        heading.line + 1,
                        1,
                        "docs.opportunity_anchor",
                        format!("{id} requires one usable explicit owner anchor"),
                    ));
                }
            }
        }
        let indexed = document
            .metadata
            .get("doc_role")
            .and_then(metadata::Value::as_str)
            .is_some_and(|role| ["plan", "review", "decision"].contains(&role));
        if indexed && file.file_name().is_none_or(|name| name != "README.md") {
            let index = file
                .parent()
                .context("document parent absent")?
                .join("README.md");
            let mut found = false;
            if let Ok(text) = fs::read_to_string(root.join(&index)) {
                for event in Parser::new(&text) {
                    if let Event::Start(Tag::Link { dest_url, .. }) = event
                        && target(root, &index, &dest_url).is_some_and(|(path, _)| {
                            site::resolved(&path).is_ok_and(|path| path == root.join(file))
                        })
                    {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                report.diagnostics.push(diagnostic(
                    file,
                    1,
                    1,
                    "docs.readme_index",
                    format!(
                        "selected document is absent from its immediate index {}",
                        index.display()
                    ),
                ));
            }
        }
    }
    Ok(())
}
fn merge_links(root: &Path, files: &[PathBuf], bytes: &[u8], report: &mut Report) -> Result<()> {
    let links: Lychee = serde_json::from_slice(bytes).context("pinned lychee JSON response")?;
    let config = site::configuration(root).ok();
    let blueprint = root.join("docs/authoritative_design/blueprint.md");
    let sections = if blueprint.is_file() {
        Some(adr::section_owners(&blueprint)?)
    } else {
        None
    };
    let count: usize = links.error_map.values().map(Vec::len).sum();
    ensure!(
        count == links.errors,
        "pinned lychee error count does not match original source records"
    );
    for (source, records) in links.error_map {
        let file = site::resolved(&source)?.strip_prefix(root)?.to_owned();
        ensure!(
            files.contains(&file),
            "lychee returned an unselected source"
        );
        for link in records {
            let local = target(root, &file, &link.url);
            let mut code = "docs.link";
            let mut category = Category::Scoped;
            if let Some((path, fragment)) = &local {
                if historical(root, path, config.as_ref()) {
                    category = Category::Historical;
                }
                if let Some(fragment) = fragment {
                    code = if fragment.starts_with("section-") {
                        "docs.section_anchor"
                    } else {
                        "docs.fragment"
                    };
                    if fragment.starts_with("section-")
                        && link.status.text == "Cannot find fragment"
                        && sections.as_ref().is_some_and(|owners| {
                            owners.iter().any(|(id, (owner, _))| {
                                format!("section-{}", id.to_lowercase().replace('.', "-"))
                                    == *fragment
                                    && owner == path
                            })
                        })
                    {
                        continue;
                    }
                }
            }
            let span = link.span.unwrap_or(Span { line: 1, column: 1 });
            let mut finding = diagnostic(&file, span.line, span.column, code, link.status.text);
            finding.category = category;
            finding.target = Some(link.url);
            report.diagnostics.push(finding);
        }
    }
    for (source, records) in links.excluded_map {
        let file = site::resolved(&source)?.strip_prefix(root)?.to_owned();
        ensure!(
            files.contains(&file),
            "lychee excluded an unselected source"
        );
        for link in records {
            let span = link.span.unwrap_or(Span { line: 1, column: 1 });
            let category = if (link.url.starts_with("git:") || link.url.contains("/blob/"))
                && lifecycle::reference(root, &link.url, None).is_ok()
            {
                Category::Historical
            } else {
                Category::External
            };
            report.diagnostics.push(Diagnostic {
                file: file.clone(),
                line: span.line,
                column: span.column,
                code: "docs.offline_reference".into(),
                category,
                severity: Severity::Information,
                target: Some(link.url),
                message: "external/historical transport was not visited by this offline check"
                    .into(),
            });
        }
    }
    Ok(())
}
/// Check the exact selected documents through native metadata and one already selected tool.
/// # Errors
/// Source selection, tool settlement/encoding or inconsistent pinned tool output.
pub fn check(root: &Path, query: &Query) -> Result<Report> {
    let root = site::resolved(root)?;
    let files = selected(&root, query)?;
    let mut report=Report {version:1,files:files.clone(),tool_version:None,tool_exit:None,diagnostics:Vec::new(),passed:false,limitations:vec!["Only named/changed current Markdown sources were checked; linked targets are read only to validate those references.".into(),"External network references and historical Git objects were not fetched; no historical cleanup or repository-wide qualification is implied.".into()]};
    native_checks(&root, &files, &mut report)?;
    if !files.is_empty() {
        let operation = (|| {
            let tool = select_tool(&root)?;
            let mut version = Command::new(&tool);
            version.arg("--version").env_clear();
            let version = capture(version, 4096)?;
            ensure!(version.code == 0, "selected lychee version query failed");
            report.tool_version = Some(String::from_utf8(version.stdout)?.trim().into());
            let config = tempfile::NamedTempFile::new()?;
            let mut command = Command::new(&tool);
            command
                .current_dir(&root)
                .env_clear()
                .args([
                    "--offline",
                    "--include-fragments",
                    "--no-progress",
                    "--no-ignore",
                    "--hidden",
                    "--format",
                    "json",
                    "--config",
                ])
                .arg(config.path())
                .arg("--")
                .args(files.iter().map(|path| root.join(path)));
            let output = capture(command, TOOL_BYTES)?;
            report.tool_exit = Some(output.code);
            ensure!(
                [0, 2].contains(&output.code),
                "selected lychee exited{}: {}",
                output.code,
                String::from_utf8_lossy(&output.stderr)
            );
            merge_links(&root, &files, &output.stdout, &mut report)
        })();
        if let Err(error) = operation {
            report.diagnostics.push(diagnostic(
                &files[0],
                1,
                1,
                "docs.tool_unavailable",
                format!("{error:#}"),
            ));
        }
    }
    report.diagnostics.sort_by(|left, right| {
        (&left.file, left.line, left.column, &left.code, &left.target).cmp(&(
            &right.file,
            right.line,
            right.column,
            &right.code,
            &right.target,
        ))
    });
    report.passed = !report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    Ok(report)
}
/// Emit the real response schema or one source-bounded report, then preserve its verdict.
/// # Errors
/// Source/tool/terminal failures or a report containing selected error diagnostics.
pub fn execute(root: &Path, query: Query) -> Result<()> {
    if query.schema {
        let mut output = std::io::stdout().lock();
        serde_json::to_writer_pretty(&mut output, &schemars::schema_for!(Report))?;
        writeln!(output)?;
        return Ok(());
    }
    let report = check(root, &query)?;
    let mut output = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &report)?;
    writeln!(output)?;
    ensure!(report.passed, "selected documentation checks failed");
    Ok(())
}

#[cfg(test)]
mod tests;
