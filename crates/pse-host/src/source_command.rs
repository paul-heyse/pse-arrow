// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive one-file source command views over the shared structural reader.
/// Shared-pool CPU batches with actual completion ownership.
pub mod batch;
use crate::{
    envelope::{Completeness, Envelope},
    source_read::{
        self, Outline, PageRequest, PageState, PositionSelector, RustEdition, SourceIdentity,
        SourceLocator, SourceRequest, SourceSelection,
    },
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
    /// Exact original source kind, context and content generation.
    pub source: SourceIdentity,
    /// Declaration coverage and exact-source continuation.
    pub page: PageState,
    /// Structure, edition, source digest and parse errors from the same read.
    pub outline: Outline,
}
/// Exact selection plus the source context needed to interpret its limitations.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FileSelection {
    /// Native file operand, including non-UTF8 filename bytes.
    pub file: OsString,
    /// Reusable exact-source location, without resolved-symbol authority.
    pub locator: SourceLocator,
    /// Actual selected parser/backend.
    pub backend: String,
    /// Selected parsing edition, never an inferred Cargo configuration.
    pub rust_edition: Option<RustEdition>,
    /// Same-read parser errors, including those outside the selected declaration.
    pub parse_issues: Vec<source_read::ParseIssue>,
    /// Exact original UTF-8 text; line numbering is a separate human presentation.
    pub selection: SourceSelection,
    /// Interpreted observations inside the exact selected source extent.
    pub facts: Vec<source_read::SourceFact>,
}
/// Source reading refuses unsupported or ambiguous selectors without guessing.
#[derive(Debug, thiserror::Error)]
pub enum SourceCommandError {
    /// Explicit generated protobuf association retains its own original refusal.
    #[error(transparent)]
    Schema(#[from] crate::schema::SchemaError),
    /// Guarded result construction retains the shared investigation refusal.
    #[error(transparent)]
    Investigation(#[from] crate::investigation::Error),
    /// Existing generated ownership evidence remains independently fallible.
    #[error(transparent)]
    Ownership(#[from] crate::ownership::OwnershipError),
    /// Mandatory provenance projection preserves the original serialization cause.
    #[error("source provenance projection: {0}")]
    Projection(#[from] serde_json::Error),
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
    /// A task's result has already been consumed.
    #[error("source batch completion was already consumed")]
    AlreadySettled,
    /// The bridge propagated an unwinding CPU task panic after its tasks returned.
    #[error("source batch CPU task panicked: {0}; this message is not a native error chain")]
    WorkerPanic(String),
    /// A panic carried an opaque non-string payload rather than a diagnostic message.
    #[error(
        "source batch CPU task panicked with an opaque non-string payload; no native error chain is available"
    )]
    OpaqueWorkerPanic,
}
/// A generated hit retains its exact source independently from declaration-map evidence.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GeneratedOrigin {
    /// Exact byte-safe source and original selected range.
    pub hit: SourceLocator,
    /// Exact original selected bytes.
    pub text: String,
    /// Existing map/producer sidecar observations, without a second provenance index.
    pub owner: crate::ownership::OwnerQuery,
    /// Exact producer associations for the actual enclosing generated Rust declaration.
    pub protobuf_associations: Vec<crate::schema::compare::Association>,
}
/// Inspect a generated hit through the existing ownership owner; never infer an authored
/// source location from a declaration name or execute the declared regeneration command.
pub fn generated_origin(
    root: &Path,
    request: &SourceRequest,
    range: std::ops::Range<usize>,
    if_source: Option<&str>,
) -> Result<Envelope<GeneratedOrigin>, SourceCommandError> {
    let source = request.acquire()?;
    if if_source.is_some_and(|digest| digest != source.identity().source_sha256()) {
        return Err(source_read::SourceReadError::StaleSource.into());
    }
    let text = source
        .text()
        .get(range.clone())
        .ok_or(source_read::SourceReadError::InvalidRange)?
        .to_owned();
    let protobuf_associations =
        crate::schema::compare::origin_associations(root, &source, range.clone())?;
    let hit = SourceLocator {
        source: source.identity().clone(),
        range: source_read::location(&source_read::line_starts(source.text()), range),
    };
    let mut owner = crate::ownership::query(root, request.path())?;
    let failures = std::mem::take(&mut owner.failures);
    let mut response = Envelope::response(
        "inspect origin",
        "exact generated hit and independently declared contributor/regeneration routes",
        GeneratedOrigin {
            hit,
            text,
            owner,
            protobuf_associations,
        },
    );
    response.providers.push(provider_context(
        source.identity(),
        "raw-source-acquisition;generated-output-map/v1",
        std::slice::from_ref(&response.payload.hit.range),
    )?);
    response.observation.scope = "exact_source_hit_and_existing_generator_declarations".into();
    response.observation.limitations = response.payload.owner.coverage_limits.clone();
    response.observation.limitations.push("routing/lineage comes from existing producer declarations, independently of the hit's byte generation; unknown authored source locations remain unknown".into());
    if !matches!(
        source.identity().origin(),
        source_read::SourceOrigin::Working
    ) {
        response.observation.limitations.push("Current protobuf producer manifests do not qualify historical/index generation. Original manifest acquisition is unavailable; identical generated bytes do not establish current authored origin.".into());
    }
    for association in &response.payload.protobuf_associations {
        let authored_path = root.join(&association.source);
        if association.state == "verified" {
            let authored = SourceRequest::Working {
                path: authored_path.clone(),
            }
            .acquire()?;
            if authored.identity().source_sha256() != association.source_sha256 {
                response.observation.limitations.push(format!("Authored source {} changed during association acquisition; no follow action is offered",association.source));
                continue;
            }
            let range = source_read::location(
                &source_read::line_starts(authored.text()),
                0..authored.text().len(),
            );
            let reference =
                crate::investigation::ResultReference::new(crate::investigation::Locator::Source {
                    path: authored_path.into(),
                    source: crate::investigation::SourceKind::Working,
                    range: Some(crate::investigation::MatchRange {
                        byte_start: range.byte_start,
                        byte_end: range.byte_end,
                        line_start: range.line_start,
                        line_end: range.line_end,
                    }),
                    guard: Some(crate::investigation::SourceGuard {
                        root: Some(root.to_path_buf().into()),
                        sha256: Some(association.source_sha256.clone()),
                        length: Some(authored.text().len() as u64),
                        ..Default::default()
                    }),
                });
            response.next.push(crate::envelope::NextAction::Command {
                argv:vec!["pse".into(),"inspect".into(),"--result".into(),reference.encode()?],
                reason:format!("read hash-verified authored schema {}; {} is an explicit generated association, not a native Rust consumer identity",association.source,association.declaration),
            });
        }
    }
    if let crate::ownership::Contributors::Available { records, .. } =
        &response.payload.owner.contributors
    {
        let mut emitters = std::collections::BTreeSet::new();
        for record in records {
            if emitters.insert(&record.generator_path) {
                if let Some(root) = root.to_str() {
                    response.next.push(crate::envelope::NextAction::Command {
                        argv: vec![
                            "pse".into(),
                            "-C".into(),
                            root.into(),
                            "inspect".into(),
                            "outline".into(),
                            record.generator_path.clone(),
                        ],
                        reason: format!(
                            "inspect declared emitter {} ({}) without claiming resolved identity",
                            record.generator_symbol, record.declaration_key
                        ),
                    });
                } else {
                    response.observation.limitations.push("declared emitter next action cannot encode the non-UTF8 checkout in Unicode argv; exact byte-safe ownership context remains available".into());
                }
            }
        }
    }
    let policy = crate::envelope::RedactionPolicy::from_environment();
    response.failures = failures
        .iter()
        .map(|error| crate::envelope::ErrorRecord::from_typed(error, &policy))
        .collect();
    if !response.failures.is_empty() {
        response.observation.completeness = Completeness::Partial;
    }
    Ok(response)
}

/// Project an exact locator into a literal native command only when every argument is
/// Unicode-representable. The locator itself always retains original native path bytes.
pub(crate) fn origin_action(
    root: &Path,
    locator: &SourceLocator,
) -> Option<crate::envelope::NextAction> {
    let mut argv = vec![
        "pse".into(),
        "-C".into(),
        root.to_str()?.into(),
        "inspect".into(),
        "origin".into(),
        locator.source.path().to_str()?.into(),
        "--range-start".into(),
        locator.range.byte_start.to_string(),
        "--range-end".into(),
        locator.range.byte_end.to_string(),
        "--if-source".into(),
        locator.source.source_sha256().into(),
    ];
    match locator.source.origin() {
        source_read::SourceOrigin::Working => {}
        source_read::SourceOrigin::Index { .. } => argv.push("--index".into()),
        source_read::SourceOrigin::Blob { revision, .. } => {
            argv.extend(["--revision".into(), revision.clone()])
        }
    }
    Some(crate::envelope::NextAction::Command {argv,reason:"inspect the hit through existing generated-output contributors and regeneration declarations".into()})
}
/// Project the shared exact source identity into mandatory envelope provenance. This adapter
/// owns presentation only; it neither acquires sources nor declares a second identity format.
pub(crate) fn provider_context(
    source: &SourceIdentity,
    backend: &str,
    ranges: &[source_read::SourceRange],
) -> Result<crate::envelope::ProviderContext, serde_json::Error> {
    let address = serde_json::to_string(source)?;
    Ok(crate::envelope::ProviderContext {
        provider: backend.into(),
        context: Some(address.clone()),
        analysis_generation: Some(source.source_sha256().into()),
        availability: crate::envelope::SubjectState::Available,
        coordinates: ranges
            .iter()
            .map(|range| crate::envelope::SourceCoordinates::Utf8Bytes {
                source: address.clone(),
                revision: Some(source.source_sha256().into()),
                start: range.byte_start,
                end: range.byte_end,
            })
            .collect(),
    })
}
pse_diagnostics::impl_diagnostic! {
    SourceCommandError,
    code(this) { match this {Self::Selection(error)=>pse_diagnostics::TypedDiagnostic::diagnostic_code(error), Self::WorkerPanic(_)|Self::OpaqueWorkerPanic=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),_=>Some(pse_diagnostics::DiagnosticCode::HostOwnership)} },
    forward(this) {match this {Self::Selection(error)=>Some(error),_=>None}},
    help(_this) {None}, related(_this) {None},source(_this) {None}
}
fn language(file: &Path, language: Option<Language>) -> Result<Language, SourceCommandError> {
    language
        .or_else(|| match file.extension().and_then(OsStr::to_str) {
            Some("rs") => Some(Language::Rust),
            Some("md" | "markdown") => Some(Language::Markdown),
            _ => None,
        })
        .ok_or_else(|| SourceCommandError::UnknownLanguage(file.into()))
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
    outline_page(
        &SourceRequest::Working { path: file.into() },
        language,
        edition,
        PageRequest::default(),
    )
}
/// Acquire one addressed source and build only its selected declaration page.
pub fn outline_page(
    request: &SourceRequest,
    selected_language: Option<Language>,
    edition: RustEdition,
    page: PageRequest,
) -> Result<Envelope<FileOutline>, SourceCommandError> {
    let language = language(request.path(), selected_language)?;
    let source = request.acquire()?;
    let (outline, page) = source_read::outline_page(
        &source,
        match language {
            Language::Rust => Some(edition),
            Language::Markdown => None,
        },
        page,
    )?;
    let mut response = Envelope::response(
        "inspect outline",
        format!(
            "{} syntactic entries; {} parse issues",
            outline.entries.len(),
            outline.parse_issues.len()
        ),
        FileOutline {
            file: request.path().as_os_str().to_owned(),
            source: source.identity().clone(),
            page,
            outline,
        },
    );
    let has_issues = !response.payload.outline.parse_issues.is_empty();
    response.providers.push(provider_context(
        source.identity(),
        &format!(
            "{};edition={:?}",
            response.payload.outline.backend, response.payload.outline.rust_edition
        ),
        &[source_read::location(
            &source_read::line_starts(source.text()),
            0..source.text().len(),
        )],
    )?);
    coverage(&mut response, has_issues);
    if response.payload.page.omitted_records > 0 {
        response.observation.completeness = Completeness::Partial;
        response.observation.required = false;
        response.observation.limitations.push(
            "declaration page omitted further entries; continuation binds the same exact source"
                .into(),
        );
    }
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
    show_addressed(
        &SourceRequest::Working { path: file.into() },
        items,
        &at_byte.map_or(PositionSelector::Path, |offset| PositionSelector::Byte {
            offset,
        }),
        language,
        edition,
    )
}
/// Read a declaration or exact range from working, index or committed source bytes.
pub fn show_addressed(
    request: &SourceRequest,
    items: &[String],
    position: &PositionSelector,
    selected_language: Option<Language>,
    edition: RustEdition,
) -> Result<Envelope<FileSelection>, SourceCommandError> {
    let language = language(request.path(), selected_language)?;
    let source = request.acquire()?;
    let outline = match language {
        Language::Rust => source_read::rust(source.text(), edition),
        Language::Markdown => source_read::markdown(source.text()),
    };
    let selection = position.select(source.text(), &outline, items)?;
    let locator = SourceLocator {
        source: source.identity().clone(),
        range: selection.range.clone(),
    };
    let mut response = Envelope::response(
        "inspect declaration",
        format!(
            "source lines {}–{}; original text",
            selection.range.line_start, selection.range.line_end
        ),
        FileSelection {
            file: request.path().as_os_str().to_owned(),
            locator,
            backend: outline.backend,
            rust_edition: outline.rust_edition,
            facts: outline.facts.into_iter().filter(|fact| fact.range.byte_start >= selection.range.byte_start && fact.range.byte_end <= selection.range.byte_end).collect(),
            parse_issues: outline.parse_issues,
            selection,
        },
    );
    let has_issues = !response.payload.parse_issues.is_empty();
    response.providers.push(provider_context(
        source.identity(),
        &format!(
            "{};edition={:?}",
            response.payload.backend, response.payload.rust_edition
        ),
        std::slice::from_ref(&response.payload.selection.range),
    )?);
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
    if let Some(cursor) = &view.page.next_cursor {
        text.push_str(&format!(
            "{} further declarations; continuation={cursor}\n",
            view.page.omitted_records
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
