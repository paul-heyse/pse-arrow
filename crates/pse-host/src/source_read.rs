// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-only structural reading. No compiler, Cargo discovery or analyzer session is started.

use ra_ap_syntax::{AstNode, SourceFile, SyntaxKind, SyntaxNode, ast, ast::HasName};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ops::Range;

fn source_digest(source: &str) -> String {
    Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Explicit parsing policy; this is never presented as an inferred Cargo configuration.
#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, JsonSchema)]
pub enum RustEdition {
    /// Rust 2015 syntax.
    #[value(name = "2015")]
    E2015,
    /// Rust 2018 syntax.
    #[value(name = "2018")]
    E2018,
    /// Rust 2021 syntax.
    #[value(name = "2021")]
    E2021,
    /// Rust 2024 syntax.
    #[value(name = "2024")]
    E2024,
}
impl RustEdition {
    fn native(self) -> ra_ap_syntax::Edition {
        use ra_ap_syntax::Edition;
        match self {
            Self::E2015 => Edition::Edition2015,
            Self::E2018 => Edition::Edition2018,
            Self::E2021 => Edition::Edition2021,
            Self::E2024 => Edition::Edition2024,
        }
    }
}

/// Exact half-open UTF-8 byte range and one-based inclusive source lines.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct SourceRange {
    /// First byte in the source snapshot.
    pub byte_start: usize,
    /// First byte after the selected text.
    pub byte_end: usize,
    /// First line, starting at one.
    pub line_start: usize,
    /// Last line containing selected bytes, starting at one.
    pub line_end: usize,
}
fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        )
        .collect()
}
fn location(lines: &[usize], range: Range<usize>) -> SourceRange {
    let line_at = |offset: usize| lines.partition_point(|&start| start <= offset);
    SourceRange {
        byte_start: range.start,
        byte_end: range.end,
        line_start: line_at(range.start),
        line_end: line_at(if range.is_empty() {
            range.end
        } else {
            range.end - 1
        }),
    }
}

/// Written declaration/heading, not a resolved symbol or an active cfg assertion.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Entry {
    /// Syntactic ancestor names followed by the selected name.
    pub path: Vec<String>,
    /// Native Rust syntax kind, or `heading` for Markdown.
    pub kind: String,
    /// Exact declaration range, including attached docs/attributes where the parser associates them.
    pub range: SourceRange,
    /// Original declaration header; no token regeneration or formatting.
    pub signature: String,
}
/// Parser error; useful recovered entries remain available alongside it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ParseIssue {
    /// Native parser/validation message.
    pub message: String,
    /// Original source coordinates.
    pub range: SourceRange,
}
/// A structure reading of one immutable UTF-8 string.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Outline {
    /// Digest of every original source byte, including malformed regions and trivia.
    pub source_sha256: String,
    /// Exact selected Rust edition; Markdown has none.
    pub rust_edition: Option<RustEdition>,
    /// Parsing backend/version, independently of source semantics.
    pub backend: String,
    /// Entries in source order. Duplicate paths are retained, not silently collapsed.
    pub entries: Vec<Entry>,
    /// Original syntax errors; their absence does not establish compiler acceptance.
    pub parse_issues: Vec<ParseIssue>,
}

fn declaration(node: &SyntaxNode) -> Option<String> {
    if let Some(item) = ast::Item::cast(node.clone()) {
        use ast::Item;
        return match item {
            Item::Fn(n) => n.name().map(|x| x.text().to_string()),
            Item::Struct(n) => n.name().map(|x| x.text().to_string()),
            Item::Enum(n) => n.name().map(|x| x.text().to_string()),
            Item::Union(n) => n.name().map(|x| x.text().to_string()),
            Item::Trait(n) => n.name().map(|x| x.text().to_string()),
            Item::Module(n) => n.name().map(|x| x.text().to_string()),
            Item::TypeAlias(n) => n.name().map(|x| x.text().to_string()),
            Item::Const(n) => n.name().map(|x| x.text().to_string()),
            Item::Static(n) => n.name().map(|x| x.text().to_string()),
            Item::MacroRules(n) => n.name().map(|x| x.text().to_string()),
            Item::MacroDef(n) => n.name().map(|x| x.text().to_string()),
            Item::Impl(n) => n.self_ty().map(|x| x.syntax().text().to_string()),
            _ => None,
        };
    }
    None
}

/// Parse a Rust snapshot losslessly even when some syntax is incomplete.
pub fn rust(source: &str, edition: RustEdition) -> Outline {
    let lines = line_starts(source);
    let parsed = SourceFile::parse(source, edition.native());
    let entries = parsed
        .tree()
        .syntax()
        .descendants()
        .filter_map(|node| {
            let name = declaration(&node)?;
            let mut path: Vec<_> = node
                .ancestors()
                .skip(1)
                .filter_map(|n| declaration(&n))
                .collect();
            path.reverse();
            path.push(name);
            let native_range = node.text_range();
            let start = usize::from(native_range.start());
            let end = usize::from(native_range.end());
            // A direct body begins after the header; attributes may themselves contain braces.
            let body = node.children().find(|n| {
                matches!(
                    n.kind(),
                    SyntaxKind::BLOCK_EXPR
                        | SyntaxKind::ASSOC_ITEM_LIST
                        | SyntaxKind::ITEM_LIST
                        | SyntaxKind::RECORD_FIELD_LIST
                        | SyntaxKind::VARIANT_LIST
                )
            });
            let header_end = body.map_or(end, |n| usize::from(n.text_range().start()));
            Some(Entry {
                path,
                kind: format!("{:?}", node.kind()),
                range: location(&lines, start..end),
                signature: source[start..header_end].trim_end().to_owned(),
            })
        })
        .collect();
    let parse_issues = parsed
        .errors()
        .into_iter()
        .map(|error| ParseIssue {
            message: error.to_string(),
            range: location(
                &lines,
                usize::from(error.range().start())..usize::from(error.range().end()),
            ),
        })
        .collect();
    Outline {
        source_sha256: source_digest(source),
        rust_edition: Some(edition),
        backend: "ra_ap_syntax/0.0.360".into(),
        entries,
        parse_issues,
    }
}

/// Parse CommonMark headings, including setext headings and excluding fenced-code pseudo-headings.
pub fn markdown(source: &str) -> Outline {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
    let lines = line_starts(source);
    let mut headings = Vec::<(usize, String, Range<usize>)>::new();
    let mut current: Option<(usize, String, usize)> = None;
    // Repository documents use frontmatter; parsing it as a setext heading would
    // misidentify authored metadata as navigable content. Library metadata parsing
    // retains original source coordinates without a separate stripping pass.
    let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some((level as usize, String::new(), range.start))
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, title, _)) = &mut current {
                    title.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, title, _)) = &mut current {
                    title.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, title, start)) = current.take() {
                    headings.push((level, title, start..range.end));
                }
            }
            _ => {}
        }
    }
    let mut stack: Vec<(usize, String)> = Vec::new();
    let entries = headings
        .iter()
        .enumerate()
        .map(|(index, (level, title, range))| {
            while stack.last().is_some_and(|(parent, _)| parent >= level) {
                stack.pop();
            }
            stack.push((*level, title.clone()));
            let section_end = headings[index + 1..]
                .iter()
                .find(|(next_level, _, _)| next_level <= level)
                .map_or(source.len(), |(_, _, next)| next.start);
            Entry {
                path: stack.iter().map(|(_, name)| name.clone()).collect(),
                kind: "heading".into(),
                range: location(&lines, range.start..section_end),
                signature: source[range.clone()].trim_end().to_owned(),
            }
        })
        .collect();
    Outline {
        source_sha256: source_digest(source),
        rust_edition: None,
        backend: "pulldown-cmark/0.13.5".into(),
        entries,
        parse_issues: Vec::new(),
    }
}

/// A source selection cannot be guessed or reused against a changed snapshot.
#[derive(Debug, thiserror::Error)]
pub enum SourceReadError {
    /// The supplied source is no longer the outline's source.
    #[error("source changed since the structural reading; read it again")]
    StaleSource,
    /// No exact syntactic path matches.
    #[error("no declaration or heading has the requested path")]
    Missing,
    /// Multiple declarations share a path (for example, separate trait implementations).
    #[error("{0} declarations share this syntactic path; select an exact byte position")]
    Ambiguous(usize),
    /// A serialized location is invalid for the selected snapshot.
    #[error("source range is outside the snapshot or splits a UTF-8 character")]
    InvalidRange,
}
pse_diagnostics::impl_diagnostic! {
    SourceReadError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
/// Exact original source, with numbering supplied separately from bytes.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SourceSelection {
    /// Source generation to which the selected text belongs.
    pub source_sha256: String,
    /// Exact byte and line extent.
    pub range: SourceRange,
    /// Original source text, including comments and formatting.
    pub text: String,
}
/// Resolve an exact ancestor path or byte position within that path, retaining ambiguity.
pub fn show(
    source: &str,
    outline: &Outline,
    path: &[String],
    at_byte: Option<usize>,
) -> Result<SourceSelection, SourceReadError> {
    if outline.source_sha256 != source_digest(source) {
        return Err(SourceReadError::StaleSource);
    }
    let matches: Vec<_> = outline
        .entries
        .iter()
        .filter(|entry| {
            entry.path == path
                && at_byte
                    .is_none_or(|at| entry.range.byte_start <= at && at < entry.range.byte_end)
        })
        .collect();
    let entry = match matches.as_slice() {
        [] => return Err(SourceReadError::Missing),
        [one] => *one,
        several => return Err(SourceReadError::Ambiguous(several.len())),
    };
    let text = source
        .get(entry.range.byte_start..entry.range.byte_end)
        .ok_or(SourceReadError::InvalidRange)?;
    Ok(SourceSelection {
        source_sha256: outline.source_sha256.clone(),
        range: entry.range.clone(),
        text: text.into(),
    })
}

#[cfg(test)]
mod tests;
