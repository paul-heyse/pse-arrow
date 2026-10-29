// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-preserving ATX heading selection using CommonMark block recognition.
use pulldown_cmark::{Event, Parser, Tag};

/// An authored column-zero ATX heading, retaining its exact source line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heading<'a> {
    /// Zero-based line index in the complete input.
    pub line: usize,
    /// Original line without its CR/LF terminator.
    pub source: &'a str,
}

/// Select the existing ATX/line contract while excluding fenced and indented examples.
/// Setext, blockquote/list and indented headings do not widen the metadata vocabulary.
pub fn markdown_headings(text: &str) -> Vec<Heading<'_>> {
    let lines: Vec<_> = text.split_inclusive('\n').scan(0usize, |offset, line| {
        let start = *offset;
        *offset += line.len();
        Some((start, line.trim_end_matches(['\r', '\n'])))
    }).collect();
    Parser::new(text).into_offset_iter().filter_map(|(event, range)| {
        if !matches!(event, Event::Start(Tag::Heading { .. })) { return None; }
        let index = lines.partition_point(|(offset, _)| *offset <= range.start).checked_sub(1)?;
        let (offset, source) = lines[index];
        if offset != range.start { return None; }
        let hashes = source.bytes().take_while(|byte| *byte == b'#').count();
        if !(1..=6).contains(&hashes) || !source[hashes..].starts_with(char::is_whitespace) {
            return None;
        }
        Some(Heading { line: index, source })
    }).collect()
}
