// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Server-free structural reading of source files with line numbers.
//!
//! Rust files are parsed with `syn` (full syntax, `proc-macro2` span locations) into items,
//! impl blocks with their members, and tests. Markdown files are outlined by their headings,
//! parsed with `pulldown-cmark`, so fenced code and front matter never count as headings. Every
//! entry carries a 1-based inclusive line range. A Rust item's range includes its outer
//! attributes and doc comments; a heading's range runs to the line before the next heading of
//! the same or a higher level, without trailing blank lines.
//!
//! [`show`] returns exactly one entry's source. Selectors are entry paths: `Type::method`,
//! `module::function`, `Trait::method`, or a heading's text (`Parent > Child` for a nested
//! heading). A selector may be a trailing segment suffix of a path; `@LINE` selects the match
//! whose range contains that line when a name is defined more than once.
//!
//! Reading has no side effects beyond reading the named file. Rust parsing runs on a scoped
//! thread so `proc-macro2`'s thread-local source map is released with each call.

use std::fmt;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;
use syn::spanned::Spanned;

#[cfg(test)]
mod tests;

/// A structural reading failure, never a statement about the program's meaning.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum OutlineError {
    /// The source file could not be read.
    #[error("outline: cannot read {path}: {source}")]
    #[diagnostic(code(pse::host::outline::read))]
    Read {
        /// File that was requested.
        path: PathBuf,
        /// Underlying I/O failure.
        #[source]
        source: std::io::Error,
    },
    /// The file extension names no supported language.
    #[error("outline: unsupported file type {0} (expected .rs or .md)")]
    #[diagnostic(code(pse::host::outline::unsupported))]
    Unsupported(PathBuf),
    /// The Rust source does not parse.
    #[error("outline: {origin}:{line}:{column}: {message}")]
    #[diagnostic(code(pse::host::outline::parse))]
    Parse {
        /// File path, or `<source>` for in-memory text.
        origin: String,
        /// 1-based line of the failure.
        line: usize,
        /// 1-based column of the failure.
        column: usize,
        /// Parser message.
        message: String,
    },
    /// No entry matches the selector.
    #[error("outline: no item `{selector}`{}", hint(suggestions))]
    #[diagnostic(code(pse::host::outline::not_found))]
    NotFound {
        /// Requested selector.
        selector: String,
        /// Entries sharing the selector's last segment, as `path (lines a-b)`.
        suggestions: Vec<String>,
    },
    /// Several entries match the selector.
    #[error("outline: `{selector}` is ambiguous; add `@LINE` to choose{}", hint(candidates))]
    #[diagnostic(code(pse::host::outline::ambiguous))]
    Ambiguous {
        /// Requested selector.
        selector: String,
        /// Matching entries, as `path (lines a-b)`.
        candidates: Vec<String>,
    },
    /// A `<file>::<item>` target or selector is malformed.
    #[error("outline: invalid target `{0}`; expected <file.rs|file.md>::<item>[@LINE]")]
    #[diagnostic(code(pse::host::outline::target))]
    Target(String),
    /// The isolated parse thread failed before returning a result.
    #[error("outline: parser thread failed: {0}")]
    #[diagnostic(code(pse::host::outline::internal))]
    Internal(String),
}

fn hint(entries: &[String]) -> String {
    if entries.is_empty() {
        String::new()
    } else {
        format!(": {}", entries.join(", "))
    }
}

/// A language the outline understands, chosen by file extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLanguage {
    /// Rust source (`.rs`).
    Rust,
    /// Markdown (`.md`, `.markdown`).
    Markdown,
}

impl SourceLanguage {
    /// Language for a path's extension, if supported.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// What an outline entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutlineKind {
    /// `mod name { ... }` or `mod name;`.
    Module,
    /// A free function.
    Function,
    /// A function carrying a `#[test]`-style attribute (`test`, `tokio::test`, ...).
    Test,
    /// A struct.
    Struct,
    /// An enum.
    Enum,
    /// A union.
    Union,
    /// A trait.
    Trait,
    /// A trait alias.
    TraitAlias,
    /// A `type` alias.
    TypeAlias,
    /// A `const` item.
    Const,
    /// A `static` item.
    Static,
    /// `macro_rules!` definition or an item-position macro invocation.
    Macro,
    /// An `extern` block.
    ExternBlock,
    /// An `impl` block.
    Impl,
    /// A function inside an `impl` block or a trait.
    Method,
    /// An associated `const`.
    AssociatedConst,
    /// An associated `type`.
    AssociatedType,
    /// A Markdown heading.
    Heading,
}

impl OutlineKind {
    /// Short lowercase label used by the text rendering.
    pub fn label(self) -> &'static str {
        match self {
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Test => "test",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Union => "union",
            Self::Trait => "trait",
            Self::TraitAlias => "trait-alias",
            Self::TypeAlias => "type",
            Self::Const => "const",
            Self::Static => "static",
            Self::Macro => "macro",
            Self::ExternBlock => "extern",
            Self::Impl => "impl",
            Self::Method => "method",
            Self::AssociatedConst => "assoc-const",
            Self::AssociatedType => "assoc-type",
            Self::Heading => "heading",
        }
    }
}

/// One structural entry with its children.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutlineEntry {
    /// Entry kind.
    pub kind: OutlineKind,
    /// Display name: an identifier, `impl Trait for Type`, or heading text.
    pub name: String,
    /// Lookup path: `module::Type::method`, or `Parent > Child` for headings.
    pub path: String,
    /// Markdown heading level (1-6); absent for Rust entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// First line, 1-based, including outer attributes and doc comments.
    pub start_line: usize,
    /// Last line, 1-based and inclusive.
    pub end_line: usize,
    /// Nested entries in source order.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OutlineEntry>,
}

/// The structural outline of one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outline {
    /// Language the file was read as.
    pub language: SourceLanguage,
    /// Total number of lines in the file.
    pub line_count: usize,
    /// Top-level entries in source order.
    pub entries: Vec<OutlineEntry>,
}

impl Outline {
    /// Every entry, depth first in source order.
    pub fn iter(&self) -> impl Iterator<Item = &OutlineEntry> {
        let mut stack: Vec<&OutlineEntry> = self.entries.iter().rev().collect();
        std::iter::from_fn(move || {
            let entry = stack.pop()?;
            stack.extend(entry.children.iter().rev());
            Some(entry)
        })
    }

    /// The single entry a selector names (see the module documentation).
    pub fn find(&self, selector: &str) -> Result<&OutlineEntry, OutlineError> {
        let (name, line) = split_line_hint(selector)?;
        let separator = match self.language {
            SourceLanguage::Rust => "::",
            SourceLanguage::Markdown => " > ",
        };
        let exact: Vec<&OutlineEntry> = self.iter().filter(|entry| entry.path == name).collect();
        let mut matches = if exact.is_empty() {
            let suffix = format!("{separator}{name}");
            self.iter()
                .filter(|entry| entry.path.ends_with(&suffix) || entry.name == name)
                .collect()
        } else {
            exact
        };
        if let Some(line) = line {
            matches.retain(|entry| (entry.start_line..=entry.end_line).contains(&line));
            // The innermost match containing the line is the one meant.
            if let Some(innermost) = matches
                .iter()
                .min_by_key(|entry| entry.end_line - entry.start_line)
            {
                matches = vec![*innermost];
            }
        }
        match matches.as_slice() {
            [entry] => Ok(entry),
            [] => {
                let last = name.rsplit(separator).next().unwrap_or(name);
                let suggestions = self
                    .iter()
                    .filter(|entry| entry.path.rsplit(separator).next() == Some(last))
                    .map(describe)
                    .collect();
                Err(OutlineError::NotFound {
                    selector: selector.to_owned(),
                    suggestions,
                })
            }
            many => Err(OutlineError::Ambiguous {
                selector: selector.to_owned(),
                candidates: many.iter().map(|entry| describe(entry)).collect(),
            }),
        }
    }
}

fn describe(entry: &OutlineEntry) -> String {
    format!(
        "{} ({}, lines {}-{})",
        entry.path,
        entry.kind.label(),
        entry.start_line,
        entry.end_line
    )
}

fn split_line_hint(selector: &str) -> Result<(&str, Option<usize>), OutlineError> {
    let invalid = || OutlineError::Target(selector.to_owned());
    match selector.rsplit_once('@') {
        Some((name, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => {
            let line = line.parse().map_err(|_| invalid())?;
            if name.is_empty() {
                return Err(invalid());
            }
            Ok((name, Some(line)))
        }
        _ if selector.trim().is_empty() => Err(invalid()),
        _ => Ok((selector, None)),
    }
}

impl fmt::Display for Outline {
    /// One line per entry: `start-end  kind  name`, indented by depth.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_entry(
            f: &mut fmt::Formatter<'_>,
            entry: &OutlineEntry,
            depth: usize,
            width: usize,
        ) -> fmt::Result {
            let range = format!("{}-{}", entry.start_line, entry.end_line);
            writeln!(
                f,
                "{range:>width$}  {indent}{kind} {name}",
                indent = "  ".repeat(depth),
                kind = entry.kind.label(),
                name = entry.name,
            )?;
            entry
                .children
                .iter()
                .try_for_each(|child| write_entry(f, child, depth + 1, width))
        }
        let digits = self.line_count.max(1).to_string().len();
        let width = digits * 2 + 1;
        self.entries
            .iter()
            .try_for_each(|entry| write_entry(f, entry, 0, width))
    }
}

/// The exact source of one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemSource {
    /// Entry kind.
    pub kind: OutlineKind,
    /// Entry lookup path.
    pub path: String,
    /// First line, 1-based.
    pub start_line: usize,
    /// Last line, 1-based and inclusive.
    pub end_line: usize,
    /// The entry's lines, joined with `\n`, without a trailing newline.
    pub text: String,
}

impl fmt::Display for ItemSource {
    /// The entry's lines, each prefixed with its right-aligned line number.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = self.end_line.to_string().len();
        for (number, line) in (self.start_line..).zip(self.text.split('\n')) {
            writeln!(f, "{number:>width$} | {line}")?;
        }
        Ok(())
    }
}

/// Read and outline a `.rs` or `.md` file.
pub fn outline_file(path: &Path) -> Result<Outline, OutlineError> {
    let (language, source) = read(path)?;
    outline_with_origin(language, &source, &path.display().to_string())
}

/// Outline in-memory source text.
pub fn outline_source(language: SourceLanguage, source: &str) -> Result<Outline, OutlineError> {
    outline_with_origin(language, source, "<source>")
}

/// Read a file and return exactly the entry `selector` names.
pub fn show(path: &Path, selector: &str) -> Result<ItemSource, OutlineError> {
    let (language, source) = read(path)?;
    let outline = outline_with_origin(language, &source, &path.display().to_string())?;
    extract(&outline, &source, selector)
}

/// Return exactly the entry `selector` names from in-memory source text.
pub fn show_source(
    language: SourceLanguage,
    source: &str,
    selector: &str,
) -> Result<ItemSource, OutlineError> {
    let outline = outline_source(language, source)?;
    extract(&outline, source, selector)
}

/// Split a `<file>::<selector>` target at the first `::` following a `.rs` or `.md` path.
pub fn parse_show_target(target: &str) -> Result<(PathBuf, String), OutlineError> {
    let invalid = || OutlineError::Target(target.to_owned());
    for extension in [".rs::", ".md::", ".markdown::"] {
        if let Some(index) = target.find(extension) {
            let split = index + extension.len() - 2;
            let (file, selector) = (&target[..split], &target[split + 2..]);
            if file.len() == extension.len() - 2 || selector.trim().is_empty() {
                return Err(invalid());
            }
            return Ok((PathBuf::from(file), selector.to_owned()));
        }
    }
    Err(invalid())
}

fn read(path: &Path) -> Result<(SourceLanguage, String), OutlineError> {
    let language =
        SourceLanguage::from_path(path).ok_or_else(|| OutlineError::Unsupported(path.into()))?;
    let source = std::fs::read_to_string(path).map_err(|source| OutlineError::Read {
        path: path.into(),
        source,
    })?;
    Ok((language, source))
}

fn outline_with_origin(
    language: SourceLanguage,
    source: &str,
    origin: &str,
) -> Result<Outline, OutlineError> {
    let entries = match language {
        SourceLanguage::Rust => rust::outline(source, origin)?,
        SourceLanguage::Markdown => markdown::outline(source),
    };
    Ok(Outline {
        language,
        line_count: source.lines().count(),
        entries,
    })
}

fn extract(outline: &Outline, source: &str, selector: &str) -> Result<ItemSource, OutlineError> {
    let entry = outline.find(selector)?;
    let text = source
        .lines()
        .skip(entry.start_line - 1)
        .take(entry.end_line + 1 - entry.start_line)
        .collect::<Vec<_>>()
        .join("\n");
    Ok(ItemSource {
        kind: entry.kind,
        path: entry.path.clone(),
        start_line: entry.start_line,
        end_line: entry.end_line,
        text,
    })
}

mod rust {
    use super::{OutlineEntry, OutlineError, OutlineKind, Span, Spanned};
    use syn::{ImplItem, Item, TraitItem, Type};

    pub(super) fn outline(source: &str, origin: &str) -> Result<Vec<OutlineEntry>, OutlineError> {
        // `syn::parse_file` drops a byte-order mark before parsing; strip it here so byte
        // ranges index the same text.
        let text = source.strip_prefix('\u{feff}').unwrap_or(source);
        // proc-macro2 keeps every parsed file in a thread-local source map that lives as long
        // as the thread. A scoped thread confines it to this call.
        std::thread::scope(|scope| {
            scope
                .spawn(|| parse(text, origin))
                .join()
                .map_err(|_| OutlineError::Internal("Rust outline thread panicked".into()))?
        })
    }

    fn parse(text: &str, origin: &str) -> Result<Vec<OutlineEntry>, OutlineError> {
        let file = syn::parse_file(text).map_err(|error| {
            let start = error.span().start();
            OutlineError::Parse {
                origin: origin.to_owned(),
                line: start.line,
                column: start.column + 1,
                message: error.to_string(),
            }
        })?;
        let reader = Reader { text };
        Ok(reader.items(&file.items, ""))
    }

    struct Reader<'a> {
        text: &'a str,
    }

    impl Reader<'_> {
        fn items(&self, items: &[Item], prefix: &str) -> Vec<OutlineEntry> {
            items
                .iter()
                .filter_map(|item| self.item(item, prefix))
                .collect()
        }

        fn item(&self, item: &Item, prefix: &str) -> Option<OutlineEntry> {
            let (kind, name) = match item {
                Item::Const(item) => (OutlineKind::Const, item.ident.to_string()),
                Item::Enum(item) => (OutlineKind::Enum, item.ident.to_string()),
                Item::Fn(item) => {
                    let kind = if is_test(&item.attrs) {
                        OutlineKind::Test
                    } else {
                        OutlineKind::Function
                    };
                    (kind, item.sig.ident.to_string())
                }
                Item::ForeignMod(item) => {
                    let abi = item.abi.name.as_ref().map_or_else(
                        || "extern".to_owned(),
                        |name| format!("extern \"{}\"", name.value()),
                    );
                    let path = join(prefix, &abi);
                    return Some(entry(OutlineKind::ExternBlock, abi, path, item.span()));
                }
                Item::Impl(item) => return Some(self.impl_block(item, prefix)),
                Item::Macro(item) => {
                    let name = item.ident.as_ref().map_or_else(
                        || format!("{}!", self.slice(item.mac.path.span())),
                        ToString::to_string,
                    );
                    (OutlineKind::Macro, name)
                }
                Item::Mod(item) => {
                    let name = item.ident.to_string();
                    let path = join(prefix, &name);
                    let children = item
                        .content
                        .as_ref()
                        .map(|(_, items)| self.items(items, &path))
                        .unwrap_or_default();
                    let mut module = entry(OutlineKind::Module, name, path, item.span());
                    module.children = children;
                    return Some(module);
                }
                Item::Static(item) => (OutlineKind::Static, item.ident.to_string()),
                Item::Struct(item) => (OutlineKind::Struct, item.ident.to_string()),
                Item::Trait(item) => {
                    let name = item.ident.to_string();
                    let path = join(prefix, &name);
                    let children = item
                        .items
                        .iter()
                        .filter_map(|member| trait_member(member, &path))
                        .collect();
                    let mut trait_entry = entry(OutlineKind::Trait, name, path, item.span());
                    trait_entry.children = children;
                    return Some(trait_entry);
                }
                Item::TraitAlias(item) => (OutlineKind::TraitAlias, item.ident.to_string()),
                Item::Type(item) => (OutlineKind::TypeAlias, item.ident.to_string()),
                Item::Union(item) => (OutlineKind::Union, item.ident.to_string()),
                // Imports, extern crates and unparsed tokens are not structure.
                _ => return None,
            };
            let path = join(prefix, &name);
            Some(entry(kind, name, path, item.span()))
        }

        fn impl_block(&self, item: &syn::ItemImpl, prefix: &str) -> OutlineEntry {
            let self_name = self.type_display(&item.self_ty);
            let display = match &item.trait_ {
                Some((path, _)) => format!(
                    "impl {}{} for {self_name}",
                    if item.modifiers.polarity.is_some() {
                        "!"
                    } else {
                        ""
                    },
                    collapse(self.slice(path.span()))
                ),
                None => format!("impl {self_name}"),
            };
            let owner = join(prefix, &type_key(&item.self_ty).unwrap_or(self_name));
            let children = item
                .items
                .iter()
                .filter_map(|member| impl_member(member, &owner))
                .collect();
            let path = join(prefix, &display);
            let mut block = entry(OutlineKind::Impl, display, path, item.span());
            block.children = children;
            block
        }

        fn type_display(&self, ty: &Type) -> String {
            collapse(self.slice(ty.span()))
        }

        fn slice(&self, span: Span) -> &str {
            self.text.get(span.byte_range()).unwrap_or_default()
        }
    }

    fn impl_member(member: &ImplItem, owner: &str) -> Option<OutlineEntry> {
        let (kind, name) = match member {
            ImplItem::Const(item) => (OutlineKind::AssociatedConst, item.ident.to_string()),
            ImplItem::Fn(item) => {
                let kind = if is_test(&item.attrs) {
                    OutlineKind::Test
                } else {
                    OutlineKind::Method
                };
                (kind, item.sig.ident.to_string())
            }
            ImplItem::Type(item) => (OutlineKind::AssociatedType, item.ident.to_string()),
            ImplItem::Macro(item) => (OutlineKind::Macro, format!("{}!", path_text(&item.mac.path))),
            _ => return None,
        };
        let path = join(owner, &name);
        Some(entry(kind, name, path, member.span()))
    }

    fn trait_member(member: &TraitItem, owner: &str) -> Option<OutlineEntry> {
        let (kind, name) = match member {
            TraitItem::Const(item) => (OutlineKind::AssociatedConst, item.ident.to_string()),
            TraitItem::Fn(item) => (OutlineKind::Method, item.sig.ident.to_string()),
            TraitItem::Type(item) => (OutlineKind::AssociatedType, item.ident.to_string()),
            TraitItem::Macro(item) => (OutlineKind::Macro, format!("{}!", path_text(&item.mac.path))),
            _ => return None,
        };
        let path = join(owner, &name);
        Some(entry(kind, name, path, member.span()))
    }

    fn entry(kind: OutlineKind, name: String, path: String, span: Span) -> OutlineEntry {
        OutlineEntry {
            kind,
            name,
            path,
            level: None,
            start_line: span.start().line,
            end_line: span.end().line,
            children: Vec::new(),
        }
    }

    /// A `#[test]`-style attribute: any attribute path ending in `test`.
    fn is_test(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    /// The path segment an impl's members are addressed under: the self type's last path
    /// segment without generics (`Box<T>` is `Box`, `crate::a::Foo` is `Foo`).
    fn type_key(ty: &Type) -> Option<String> {
        match ty {
            Type::Path(path) if path.qself.is_none() => {
                path.path.segments.last().map(|segment| segment.ident.to_string())
            }
            Type::Group(group) => type_key(&group.elem),
            Type::Paren(paren) => type_key(&paren.elem),
            _ => None,
        }
    }

    fn path_text(path: &syn::Path) -> String {
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    fn join(prefix: &str, name: &str) -> String {
        if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}::{name}")
        }
    }

    /// Authored text with each whitespace run collapsed to one space.
    fn collapse(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

mod markdown {
    use super::{Event, Options, OutlineEntry, OutlineKind, Parser, Tag, TagEnd};

    struct Heading {
        level: u8,
        text: String,
        start_line: usize,
    }

    pub(super) fn outline(source: &str) -> Vec<OutlineEntry> {
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(index, _)| index + 1))
            .collect();
        let line_of = |offset: usize| line_starts.partition_point(|&start| start <= offset);
        let lines: Vec<&str> = source.lines().collect();

        let options = Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH;
        let mut headings = Vec::new();
        let mut open: Option<Heading> = None;
        for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    open = Some(Heading {
                        level: level as u8,
                        text: String::new(),
                        start_line: line_of(range.start),
                    });
                }
                Event::End(TagEnd::Heading(_)) => headings.extend(open.take()),
                Event::Text(text) | Event::Code(text) => {
                    if let Some(heading) = open.as_mut() {
                        heading.text.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if let Some(heading) = open.as_mut() {
                        heading.text.push(' ');
                    }
                }
                _ => {}
            }
        }

        // A section ends before the next heading at the same or a higher level.
        let flat: Vec<OutlineEntry> = headings
            .iter()
            .enumerate()
            .map(|(index, heading)| {
                let next = headings[index + 1..]
                    .iter()
                    .find(|later| later.level <= heading.level)
                    .map_or(lines.len() + 1, |later| later.start_line);
                let mut end_line = (next - 1).max(heading.start_line);
                while end_line > heading.start_line
                    && lines.get(end_line - 1).is_some_and(|line| line.trim().is_empty())
                {
                    end_line -= 1;
                }
                let name = heading.text.trim().to_owned();
                OutlineEntry {
                    kind: OutlineKind::Heading,
                    path: name.clone(),
                    name,
                    level: Some(heading.level),
                    start_line: heading.start_line,
                    end_line,
                    children: Vec::new(),
                }
            })
            .collect();
        nest(flat)
    }

    /// Nest headings under the nearest preceding heading of a lower level.
    fn nest(flat: Vec<OutlineEntry>) -> Vec<OutlineEntry> {
        let mut roots: Vec<OutlineEntry> = Vec::new();
        let mut stack: Vec<OutlineEntry> = Vec::new();
        for mut heading in flat {
            while stack
                .last()
                .is_some_and(|parent| parent.level >= heading.level)
            {
                close(&mut stack, &mut roots);
            }
            if let Some(parent) = stack.last() {
                heading.path = format!("{} > {}", parent.path, heading.name);
            }
            stack.push(heading);
        }
        while !stack.is_empty() {
            close(&mut stack, &mut roots);
        }
        roots
    }

    fn close(stack: &mut Vec<OutlineEntry>, roots: &mut Vec<OutlineEntry>) {
        if let Some(done) = stack.pop() {
            match stack.last_mut() {
                Some(parent) => parent.children.push(done),
                None => roots.push(done),
            }
        }
    }
}
