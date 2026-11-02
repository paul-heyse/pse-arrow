// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact locations and source-bound producer pagination.
use super::{
    Outline, RustEdition, SourceIdentity, SourceRange, SourceReadError, SourceSelection,
    SourceText, line_starts, location, show,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Reusable exact original source location; it makes no resolved-symbol claim.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SourceLocator {
    /// Exact source kind, path and content generation.
    pub source: SourceIdentity,
    /// Half-open original UTF-8 coordinates.
    pub range: SourceRange,
}
impl SourceLocator {
    /// Recover original text only from the same source identity and validated coordinates.
    pub fn select<'a>(&self, source: &'a SourceText) -> Result<&'a str, SourceReadError> {
        if &self.source != source.identity() {
            return Err(SourceReadError::StaleSource);
        }
        let selected = source
            .text()
            .get(self.range.byte_start..self.range.byte_end)
            .ok_or(SourceReadError::InvalidRange)?;
        if location(
            &line_starts(source.text()),
            self.range.byte_start..self.range.byte_end,
        ) != self.range
        {
            return Err(SourceReadError::InvalidRange);
        }
        Ok(selected)
    }
}
/// Explicit navigation coordinates, independent of synthetic declaration labels.
#[derive(Clone, Debug)]
pub enum PositionSelector {
    /// Exact written declaration path supplied separately.
    Path,
    /// Enclosing declaration at a zero-based UTF-8 byte offset.
    Byte {
        /// Zero-based original UTF-8 byte position.
        offset: usize,
    },
    /// Enclosing declaration at one-based line and Unicode scalar column.
    /// A line's newline is excluded; the column immediately after its final scalar is valid.
    LineColumn {
        /// One-based source line.
        line: usize,
        /// One-based Unicode scalar column, excluding the newline.
        column: usize,
    },
    /// Exact half-open byte slice, without requiring a declaration.
    Range {
        /// Inclusive original UTF-8 byte offset.
        start: usize,
        /// Exclusive original UTF-8 byte offset.
        end: usize,
    },
}
impl PositionSelector {
    /// Read against the same parsed source. Path constraints remain useful at duplicate labels.
    pub fn select(
        &self,
        source: &str,
        outline: &Outline,
        path: &[String],
    ) -> Result<SourceSelection, SourceReadError> {
        match self {
            Self::Path => show(source, outline, path, None),
            Self::Byte { offset } => show(source, outline, path, Some(*offset)),
            Self::LineColumn { line, column } => show(
                source,
                outline,
                path,
                Some(byte_offset(source, *line, *column)?),
            ),
            Self::Range { start, end } => {
                if !path.is_empty() {
                    return Err(SourceReadError::InvalidRange);
                }
                if outline.source_sha256 != super::source_digest(source) {
                    return Err(SourceReadError::StaleSource);
                }
                let text = source
                    .get(*start..*end)
                    .ok_or(SourceReadError::InvalidRange)?;
                Ok(SourceSelection {
                    source_sha256: outline.source_sha256.clone(),
                    range: location(&line_starts(source), *start..*end),
                    text: text.into(),
                })
            }
        }
    }
}
/// Convert one-based Unicode scalar coordinates to original UTF-8 bytes.
/// CRLF is a single line ending; standalone CR remains an ordinary scalar.
pub fn byte_offset(source: &str, line: usize, column: usize) -> Result<usize, SourceReadError> {
    let starts = line_starts(source);
    let start = *starts
        .get(line.checked_sub(1).ok_or(SourceReadError::InvalidRange)?)
        .ok_or(SourceReadError::InvalidRange)?;
    let next = starts.get(line);
    let end = next.map_or(source.len(), |end| end - 1);
    let mut text = &source[start..end];
    if next.is_some() {
        text = text.strip_suffix('\r').unwrap_or(text);
    }
    let ordinal = column.checked_sub(1).ok_or(SourceReadError::InvalidRange)?;
    let offset = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(text.len()))
        .nth(ordinal)
        .ok_or(SourceReadError::InvalidRange)?;
    Ok(start + offset)
}

#[cfg(test)]
mod coordinate_tests {
    use super::*;
    #[test]
    fn original_utf8_coordinates_normalize_only_crlf_endings() {
        let text = "α\r\nβ\rγ\n";
        assert_eq!(byte_offset(text, 1, 2).unwrap(), 2);
        assert!(byte_offset(text, 1, 3).is_err());
        assert_eq!(byte_offset(text, 2, 1).unwrap(), 4);
        assert_eq!(byte_offset(text, 2, 2).unwrap(), 6);
        assert_eq!(byte_offset(text, 2, 3).unwrap(), 7);
        assert_eq!(byte_offset(text, 2, 4).unwrap(), 9);
        assert_eq!(byte_offset("\r\r\n", 1, 2).unwrap(), 1);
        assert_eq!(byte_offset("α\r", 1, 3).unwrap(), 3);
        assert!(byte_offset(text, 0, 1).is_err());
        assert!(byte_offset(text, 1, 0).is_err());
    }
}
/// Producer allowance applies to every declaration, including nested outline entries.
#[derive(Clone, Debug, Default)]
pub struct PageRequest {
    /// Interpretation demand, independent of declaration record allowance.
    pub detail: super::SourceDetail,
    /// Maximum declaration records; None selects all. Zero refuses.
    pub max_records: Option<usize>,
    /// Opaque continuation returned by this same producer.
    pub cursor: Option<String>,
}
/// Source/backend-bound continuation. Consumers should pass the encoded value unchanged.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OutlineCursor {
    source: SourceIdentity,
    backend: String,
    edition: Option<RustEdition>,
    detail: super::SourceDetail,
    next: usize,
}
/// Coverage and continuation for a bounded declaration collection.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PageState {
    /// Total declarations examined, including nested entries.
    pub total_records: usize,
    /// Number of declarations before the returned page.
    pub offset: usize,
    /// Returned collection size.
    pub returned_records: usize,
    /// Remaining declarations following the returned page.
    pub omitted_records: usize,
    /// Exact-source continuation, never an offset into fresh working bytes.
    pub next_cursor: Option<String>,
}

/// Parse and retain only the requested declaration page; ASTs stay operation-local.
pub(crate) fn outline(
    source: &SourceText,
    edition: Option<RustEdition>,
    request: PageRequest,
) -> Result<(Outline, PageState), SourceReadError> {
    let backend = if edition.is_some() {
        "ra_ap_syntax/0.0.360"
    } else {
        "pulldown-cmark/0.13.5"
    };
    let limit = request.max_records.unwrap_or(usize::MAX);
    if limit == 0 {
        return Err(SourceReadError::InvalidCursor);
    }
    let offset = if let Some(cursor) = request.cursor {
        let cursor: OutlineCursor =
            serde_json::from_str(&cursor).map_err(|_| SourceReadError::InvalidCursor)?;
        if cursor.source != *source.identity() {
            return Err(SourceReadError::StaleSource);
        }
        if cursor.backend != backend || cursor.edition != edition || cursor.detail != request.detail
        {
            return Err(SourceReadError::InvalidCursor);
        }
        cursor.next
    } else {
        0
    };
    let (outline, total) = match edition {
        Some(edition) => super::rust_bounded(source.text(), edition, offset, limit, request.detail),
        None => super::markdown_bounded(source.text(), offset, limit),
    };
    if offset > total {
        return Err(SourceReadError::InvalidCursor);
    }
    let next = offset + outline.entries.len();
    let next_cursor = if next < total {
        Some(
            serde_json::to_string(&OutlineCursor {
                source: source.identity().clone(),
                backend: backend.into(),
                edition,
                detail: request.detail,
                next,
            })
            .map_err(|_| SourceReadError::InvalidCursor)?,
        )
    } else {
        None
    };
    let state = PageState {
        total_records: total,
        offset,
        returned_records: outline.entries.len(),
        omitted_records: total - next,
        next_cursor,
    };
    Ok((outline, state))
}
