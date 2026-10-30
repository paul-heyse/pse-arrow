// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive one-file source command views over the shared structural reader.
use crate::{
    envelope::{Completeness, Envelope},
    source_read::{self, Outline, RustEdition, SourceSelection},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt as _, OsStringExt as _},
    path::{Path, PathBuf},
};

/// Explicit format override; filename suffixes are otherwise the only inference.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Language {
    /// Lossless Rust syntax, without Cargo/cfg/macro expansion.
    Rust,
    /// CommonMark headings and their source sections.
    Markdown,
}
/// One exact file operand and its immutable parsed source reading.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FileOutline {
    /// Native file path; platform bytes are retained by Serde's OsString representation.
    pub file: OsString,
    /// Structure, edition, source digest and parse errors from the same read.
    pub outline: Outline,
}
/// Exact selection plus the source context needed to interpret its limitations.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FileSelection {
    /// Native file operand, including non-UTF8 filename bytes.
    pub file: OsString,
    /// Actual selected parser/backend.
    pub backend: String,
    /// Selected parsing edition, never an inferred Cargo configuration.
    pub rust_edition: Option<RustEdition>,
    /// Same-read parser errors, including those outside the selected declaration.
    pub parse_issues: Vec<source_read::ParseIssue>,
    /// Exact original UTF-8 text; line numbering is a separate human presentation.
    pub selection: SourceSelection,
}
/// Source reading refuses unsupported or ambiguous selectors without guessing.
#[derive(Debug, thiserror::Error)]
pub enum SourceCommandError {
    /// File could not be read as UTF-8.
    #[error("cannot read source {file:?}: {source}")]
    Read {
        /// Original selected path, without lossy filename conversion.
        file: PathBuf,
        /// Native I/O/UTF-8 cause.
        #[source]
        source: std::io::Error,
    },
    /// Unknown formats need an explicit choice.
    #[error("source format is unknown for {0:?}; select --language rust or markdown")]
    UnknownLanguage(PathBuf),
    /// Native filenames may be opaque; item/heading names must be Unicode.
    #[error(
        "source selection must be FILE::ITEM[::ITEM] or FILE --item ITEM; use --item for filenames containing ::"
    )]
    InvalidSelector,
    /// Preserve the reader's typed absence/ambiguity/source-generation cause.
    #[error(transparent)]
    Selection(#[from] source_read::SourceReadError),
}
pse_diagnostics::impl_diagnostic! {
    SourceCommandError,
    code(this) { match this {Self::Selection(error)=>pse_diagnostics::TypedDiagnostic::diagnostic_code(error), Self::Read{..}=>Some(pse_diagnostics::DiagnosticCode::HostIo),_=>Some(pse_diagnostics::DiagnosticCode::HostOwnership)} },
    forward(this) {match this {Self::Selection(error)=>Some(error),_=>None}},
    help(_this) {None}, related(_this) {None},source(_this) {None}
}
fn read(
    file: &Path,
    language: Option<Language>,
    edition: RustEdition,
) -> Result<(String, Outline), SourceCommandError> {
    let language = language
        .or_else(|| match file.extension().and_then(OsStr::to_str) {
            Some("rs") => Some(Language::Rust),
            Some("md" | "markdown") => Some(Language::Markdown),
            _ => None,
        })
        .ok_or_else(|| SourceCommandError::UnknownLanguage(file.into()))?;
    let source = std::fs::read_to_string(file).map_err(|source| SourceCommandError::Read {
        file: file.into(),
        source,
    })?;
    let outline = match language {
        Language::Rust => source_read::rust(&source, edition),
        Language::Markdown => source_read::markdown(&source),
    };
    Ok((source, outline))
}
fn coverage<T>(response: &mut Envelope<T>, has_issues: bool) {
    response.observation.scope = "one_file_syntactic_reading".into();
    response.observation.limitations.push("written syntax only; no active cfg, macro expansion, name resolution or compiler acceptance established".into());
    if has_issues {
        response.observation.completeness = Completeness::Partial;
        response.observation.required = false;
        response
            .observation
            .limitations
            .push("recovered declarations coexist with original source parse issues".into());
    }
}
/// Read once, then present that exact parse without requiring a checkout or tools.
pub fn outline(
    file: &Path,
    language: Option<Language>,
    edition: RustEdition,
) -> Result<Envelope<FileOutline>, SourceCommandError> {
    let (_, outline) = read(file, language, edition)?;
    let mut response = Envelope::response(
        "outline",
        format!(
            "{} syntactic entries; {} parse issues",
            outline.entries.len(),
            outline.parse_issues.len()
        ),
        FileOutline {
            file: file.as_os_str().to_owned(),
            outline,
        },
    );
    let has_issues = !response.payload.outline.parse_issues.is_empty();
    coverage(&mut response, has_issues);
    Ok(response)
}
/// Split the shorthand at its first separator, or accept an explicit native file operand.
pub fn selector(
    operand: &OsStr,
    items: Vec<String>,
) -> Result<(PathBuf, Vec<String>), SourceCommandError> {
    if !items.is_empty() {
        return Ok((operand.into(), items));
    }
    let bytes = operand.as_bytes();
    let boundary = bytes
        .windows(2)
        .position(|part| part == b"::")
        .ok_or(SourceCommandError::InvalidSelector)?;
    let names = std::str::from_utf8(&bytes[boundary + 2..])
        .map_err(|_| SourceCommandError::InvalidSelector)?
        .split("::")
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if boundary == 0 || names.iter().any(String::is_empty) {
        return Err(SourceCommandError::InvalidSelector);
    }
    Ok((
        PathBuf::from(OsString::from_vec(bytes[..boundary].to_vec())),
        names,
    ))
}
/// Select from one same-read outline; duplicate paths remain actionable errors.
pub fn show(
    file: &Path,
    items: &[String],
    at_byte: Option<usize>,
    language: Option<Language>,
    edition: RustEdition,
) -> Result<Envelope<FileSelection>, SourceCommandError> {
    let (source, outline) = read(file, language, edition)?;
    let selection = source_read::show(&source, &outline, items, at_byte)?;
    let mut response = Envelope::response(
        "show",
        format!(
            "source lines {}–{}; original text",
            selection.range.line_start, selection.range.line_end
        ),
        FileSelection {
            file: file.as_os_str().to_owned(),
            backend: outline.backend,
            rust_edition: outline.rust_edition,
            parse_issues: outline.parse_issues,
            selection,
        },
    );
    let has_issues = !response.payload.parse_issues.is_empty();
    coverage(&mut response, has_issues);
    Ok(response)
}
/// Human structure lines, with parse context visible rather than hidden in a status code.
pub fn outline_text(view: &FileOutline) -> String {
    let mut text = context(
        &view.file,
        &view.outline.source_sha256,
        &view.outline.backend,
        view.outline.rust_edition,
        &view.outline.parse_issues,
    );
    for entry in &view.outline.entries {
        text.push_str(&format!(
            "{}-{} {} [{}]\n",
            entry.range.line_start,
            entry.range.line_end,
            entry.path.join("::"),
            entry.kind
        ));
    }
    text
}
/// Number display lines separately; the structured selection retains exact source bytes.
pub fn selection_text(view: &FileSelection) -> String {
    let mut text = context(
        &view.file,
        &view.selection.source_sha256,
        &view.backend,
        view.rust_edition,
        &view.parse_issues,
    );
    for (offset, line) in view.selection.text.split_inclusive('\n').enumerate() {
        text.push_str(&format!(
            "{} | {}",
            view.selection.range.line_start + offset,
            line
        ));
        if !line.ends_with('\n') {
            text.push('\n');
        }
    }
    text
}
fn context(
    file: &OsStr,
    digest: &str,
    backend: &str,
    edition: Option<RustEdition>,
    issues: &[source_read::ParseIssue],
) -> String {
    let mut text = format!(
        "{file:?}; source={digest}; backend={backend}; edition={edition:?}; syntactic only\n"
    );
    for issue in issues {
        text.push_str(&format!(
            "parse issue {}:{}-{}: {}\n",
            issue.range.line_start, issue.range.byte_start, issue.range.byte_end, issue.message
        ));
    }
    text
}
