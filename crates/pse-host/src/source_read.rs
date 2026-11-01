// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-only structural reading. No compiler, Cargo discovery or analyzer session is started.

use ra_ap_syntax::{AstNode, SourceFile, SyntaxKind, SyntaxNode, ast, ast::HasName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ops::Range;

mod acquisition;
pub use acquisition::{SourceIdentity, SourceOrigin, SourceRequest, SourceText};
mod facts;
mod navigation;
pub use facts::{AttributeMeta, InterpretedValue, SourceFact, ValueMapping};
pub use navigation::byte_offset;
pub(crate) use navigation::outline as outline_page;
pub use navigation::{OutlineCursor, PageRequest, PageState, PositionSelector, SourceLocator};

pub(crate) fn source_digest(source: &str) -> String {
    Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Choose an actual declaration/callee name inside a guarded source selection.
/// Ambiguous text is not silently rebound to the beginning of comments or attributes.
pub(crate) fn semantic_anchor(source: &str, range: Range<usize>) -> Option<usize> {
    if range.start > range.end || source.get(range.clone()).is_none() {
        return None;
    }
    let parsed = SourceFile::parse(source, RustEdition::E2024.native());
    let root = parsed.syntax_node();
    let start = ra_ap_syntax::TextSize::from(u32::try_from(range.start).ok()?);
    let end = ra_ap_syntax::TextSize::from(u32::try_from(range.end).ok()?);
    let selected = ra_ap_syntax::TextRange::new(start, end);
    let node = root
        .descendants()
        .filter(|node| node.text_range() == selected)
        .last();
    if let Some(node) = node
        && let Some(anchor) = semantic_node_anchor(&node)
    {
        return Some(anchor);
    }
    // Rustdoc may exclude attached docs/attrs from an otherwise complete item
    // span. Match that native item and its real name, never a textual prefix.
    let mut items = root
        .descendants()
        .filter(|node| {
            ast::Item::can_cast(node.kind())
                && node.text_range().end() == selected.end()
                && node.text_range().contains_range(selected)
        })
        .filter_map(|node| semantic_node_anchor(&node))
        .filter(|offset| range.contains(offset));
    if let Some(anchor) = items.next()
        && items.next().is_none()
    {
        return Some(anchor);
    }
    let mut names = root.descendants().filter(|node| {
        matches!(node.kind(), SyntaxKind::NAME | SyntaxKind::NAME_REF)
            && node.text_range().contains_range(selected)
    });
    let name = names.next()?;
    if names.next().is_some() {
        return None;
    }
    Some(usize::from(name.text_range().start()))
}
fn semantic_node_anchor(node: &SyntaxNode) -> Option<usize> {
    if let Some(method) = ast::MethodCallExpr::cast(node.clone()) {
        return method
            .name_ref()
            .map(|name| usize::from(name.syntax().text_range().start()));
    }
    if let Some(call) = ast::CallExpr::cast(node.clone()) {
        return call
            .expr()?
            .syntax()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::NAME_REF)
            .last()
            .map(|name| usize::from(name.text_range().start()));
    }
    if let Some(use_) = ast::Use::cast(node.clone()) {
        let tree = use_.use_tree()?;
        if tree.use_tree_list().is_some() || tree.star_token().is_some() {
            return None;
        }
        return tree
            .path()?
            .syntax()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::NAME_REF)
            .last()
            .map(|name| usize::from(name.text_range().start()));
    }
    if let Some(name) = node.children().find(|node| node.kind() == SyntaxKind::NAME) {
        return Some(usize::from(name.text_range().start()));
    }
    if matches!(node.kind(), SyntaxKind::NAME | SyntaxKind::NAME_REF) {
        return Some(usize::from(node.text_range().start()));
    }
    None
}

/// Explicit parsing policy; this is never presented as an inferred Cargo configuration.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize, JsonSchema,
)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub(crate) fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        )
        .collect()
}
pub(crate) fn location(lines: &[usize], range: Range<usize>) -> SourceRange {
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
    /// Native written visibility, not a semantic accessibility verdict.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) written_visibility: Option<String>,
    /// Actual enclosing module declarations, excluding impl/trait names.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) modules: Vec<String>,
    /// The native declaration is nested under a trait/impl item list.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) associated: bool,
    /// Native self receiver, used by declaration-kind filters rather than text guessing.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) method: bool,
    /// Native written header after attached docs/attributes, without token regeneration.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) written_header: Option<SourceRange>,
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
    /// Lossless literal/doc/attribute observations from this exact parse, including root docs.
    pub facts: Vec<SourceFact>,
}

/// Logical Rust identifier from an actual parsed name; raw syntax is retained in source bytes.
pub(crate) fn identifier_name(name: ast::Name) -> String {
    let text = name.text();
    text.strip_prefix("r#").unwrap_or(text).to_owned()
}
fn declaration(node: &SyntaxNode) -> Option<String> {
    if let Some(item) = ast::Item::cast(node.clone()) {
        use ast::Item;
        return match item {
            Item::Fn(n) => n.name().map(identifier_name),
            Item::Struct(n) => n.name().map(identifier_name),
            Item::Enum(n) => n.name().map(identifier_name),
            Item::Union(n) => n.name().map(identifier_name),
            Item::Trait(n) => n.name().map(identifier_name),
            Item::Module(n) => n.name().map(identifier_name),
            Item::TypeAlias(n) => n.name().map(identifier_name),
            Item::Const(n) => n.name().map(identifier_name),
            Item::Static(n) => n.name().map(identifier_name),
            Item::MacroRules(n) => n.name().map(identifier_name),
            Item::MacroDef(n) => n.name().map(identifier_name),
            Item::Impl(n) => n.self_ty().map(|x| x.syntax().text().to_string()),
            _ => None,
        };
    }
    None
}

/// Parse a Rust snapshot losslessly even when some syntax is incomplete.
pub fn rust(source: &str, edition: RustEdition) -> Outline {
    rust_bounded(source, edition, 0, usize::MAX).0
}
pub(super) fn rust_bounded(
    source: &str,
    edition: RustEdition,
    offset: usize,
    limit: usize,
) -> (Outline, usize) {
    let parsed = SourceFile::parse(source, edition.native());
    rust_parsed(source, edition, offset, limit, &parsed)
}
pub(crate) fn rust_with_syntax(source: &str, edition: RustEdition) -> (Outline, SyntaxNode) {
    let parsed = SourceFile::parse(source, edition.native());
    (
        rust_parsed(source, edition, 0, usize::MAX, &parsed).0,
        parsed.syntax_node(),
    )
}
pub(crate) fn rust_parse_issues(source: &str, edition: RustEdition) -> Vec<ParseIssue> {
    let lines = line_starts(source);
    SourceFile::parse(source, edition.native())
        .errors()
        .into_iter()
        .map(|error| ParseIssue {
            message: error.to_string(),
            range: location(
                &lines,
                usize::from(error.range().start())..usize::from(error.range().end()),
            ),
        })
        .collect()
}
fn rust_parsed(
    source: &str,
    edition: RustEdition,
    offset: usize,
    limit: usize,
    parsed: &ra_ap_syntax::Parse<SourceFile>,
) -> (Outline, usize) {
    let lines = line_starts(source);
    let mut total = 0;
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
            let header_start = node
                .children_with_tokens()
                .find(|element| {
                    !ast::AnyAttr::can_cast(element.kind()) && !element.kind().is_trivia()
                })
                .map_or(start, |element| usize::from(element.text_range().start()));
            Some(Entry {
                path,
                kind: format!("{:?}", node.kind()),
                range: location(&lines, start..end),
                signature: source[start..header_end].trim_end().to_owned(),
                written_header: Some(location(&lines, header_start..header_end)),
                written_visibility: node
                    .children()
                    .find(|child| child.kind() == SyntaxKind::VISIBILITY)
                    .map(|visibility| visibility.text().to_string()),
                modules: {
                    let mut modules = node
                        .ancestors()
                        .skip(1)
                        .filter_map(ast::Module::cast)
                        .filter_map(|module| module.name().map(identifier_name))
                        .collect::<Vec<_>>();
                    modules.reverse();
                    modules
                },
                associated: node.ancestors().skip(1).any(|ancestor| {
                    matches!(ancestor.kind(), SyntaxKind::TRAIT | SyntaxKind::IMPL)
                }),
                method: ast::Fn::cast(node.clone())
                    .and_then(|function| function.param_list())
                    .is_some_and(|parameters| parameters.self_param().is_some()),
            })
        })
        .filter_map(|entry| {
            let index = total;
            total += 1;
            (index >= offset && index - offset < limit).then_some(entry)
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
    (
        Outline {
            source_sha256: source_digest(source),
            rust_edition: Some(edition),
            backend: "ra_ap_syntax/0.0.360".into(),
            entries,
            parse_issues,
            facts: facts::collect(parsed.tree().syntax(), &lines),
        },
        total,
    )
}

/// Parse CommonMark headings, including setext headings and excluding fenced-code pseudo-headings.
pub fn markdown(source: &str) -> Outline {
    markdown_bounded(source, 0, usize::MAX).0
}
pub(super) fn markdown_bounded(source: &str, offset: usize, limit: usize) -> (Outline, usize) {
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
                written_visibility: None,
                modules: Vec::new(),
                associated: false,
                method: false,
                written_header: None,
            }
        })
        .enumerate()
        .filter_map(|(index, entry)| (index >= offset && index - offset < limit).then_some(entry))
        .collect();
    (
        Outline {
            source_sha256: source_digest(source),
            rust_edition: None,
            backend: "pulldown-cmark/0.13.5".into(),
            entries,
            parse_issues: Vec::new(),
            facts: Vec::new(),
        },
        headings.len(),
    )
}

/// A source selection cannot be guessed or reused against a changed snapshot.
#[derive(Debug, thiserror::Error)]
pub enum SourceReadError {
    /// Preserve acquisition failure rather than inventing an empty source.
    #[error("source input/output: {0}")]
    Io(#[from] std::io::Error),
    /// Preserve the repository owner's typed cause.
    #[error(transparent)]
    Repository(#[from] crate::repository::RepositoryError),
    /// Explicit supplied working bytes need an absolute operand.
    #[error("working source path must be absolute: {0:?}")]
    RelativePath(std::path::PathBuf),
    /// An acquisition result was internally inconsistent.
    #[error("source acquisition did not establish a complete identity")]
    InvalidIdentity,
    /// Invalid continuation or pagination request.
    #[error("invalid source continuation or page allowance")]
    InvalidCursor,
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
    code(this) { Some(match this {Self::Io(_)=>pse_diagnostics::DiagnosticCode::HostIo,Self::Repository(error)=>pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),_=>pse_diagnostics::DiagnosticCode::HostOwnership}) },
    forward(this) { match this {Self::Repository(error)=>Some(error),_=>None} }, help(_this) { None }, related(_this) { None }, source(_this) { None }
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
            (path.is_empty() || entry.path == path)
                && at_byte
                    .is_none_or(|at| entry.range.byte_start <= at && at < entry.range.byte_end)
        })
        .collect();
    if path.is_empty() {
        if let Some(at) = at_byte {
            if !source.is_char_boundary(at) {
                return Err(SourceReadError::InvalidRange);
            }
        } else {
            return Err(SourceReadError::Missing);
        }
    }
    let entry = if path.is_empty() {
        matches
            .iter()
            .min_by_key(|entry| entry.range.byte_end - entry.range.byte_start)
            .copied()
            .ok_or(SourceReadError::Missing)?
    } else {
        match matches.as_slice() {
            [] => return Err(SourceReadError::Missing),
            [one] => *one,
            several => return Err(SourceReadError::Ambiguous(several.len())),
        }
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
