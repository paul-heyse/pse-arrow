// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source spans and the parse budget (blueprint §22.1, §4.4 `pse.source_span`).
//!
//! Every authored row carries the span it came from, and every span names a document by
//! identity rather than by path: a path is an attribute of a document and a span that
//! pointed at one would stop resolving the moment a package was reorganized.

use core::fmt;

use pse_ids::SemanticId;

/// A byte range in an authoring document (blueprint §4.4).
///
/// Parser byte offsets use `u32`; their declared Arrow representation uses bounded
/// nonnegative `Int64` fields. A document larger than 4 GiB is outside this parser's
/// envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceSpan {
    /// The document, by identity.
    pub document_id: SemanticId,
    /// The first byte of the range.
    pub start: u32,
    /// One past the last byte of the range.
    pub end: u32,
}

impl SourceSpan {
    /// A span over `[start, end)` of `document_id`.
    pub const fn new(document_id: SemanticId, start: u32, end: u32) -> Self {
        Self {
            document_id,
            start,
            end,
        }
    }

    /// An empty span at the start of `document_id`, for a fact with no textual source.
    pub const fn head(document_id: SemanticId) -> Self {
        Self::new(document_id, 0, 0)
    }

    /// The number of bytes the span covers, or zero if it is inverted.
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the span covers no bytes.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Display for SourceSpan {
    /// `<document id>:<start>..<end>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}..{}", self.document_id, self.start, self.end)
    }
}

impl TryFrom<pse_relations::generated::extension_values::SourceSpan> for SourceSpan {
    type Error = crate::AuthoringError;

    fn try_from(
        value: pse_relations::generated::extension_values::SourceSpan,
    ) -> Result<Self, Self::Error> {
        let invalid = || crate::AuthoringError::Contract {
            at: None,
            reason: "source span exceeds the parser offset range or is inverted".into(),
        };
        let start = u32::try_from(value.start).map_err(|_| invalid())?;
        let end = u32::try_from(value.end).map_err(|_| invalid())?;
        if end < start {
            return Err(invalid());
        }
        Ok(Self::new(value.document_id, start, end))
    }
}

/// What a parser is allowed to spend on one document (blueprint §22.1).
///
/// `serde-saphyr` and `toml` refuse hostile input without panicking, but "without
/// panicking" is not the same as "within bounds": an alias bomb allocates until the process
/// dies. The budget is checked before allocation, in the same spirit as the §14.3 memory
/// reservation, and exceeding it is a typed [`crate::AuthoringError::Budget`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[expect(
    clippy::struct_field_names,
    reason = "each field is a maximum and says so; dropping the prefix would leave `depth`, `aliases` and `bytes`, none of which reads as a limit at a call site"
)]
pub struct ParseBudget {
    /// The deepest nesting a document may reach.
    pub max_depth: u32,
    /// The most aliases a document may expand.
    pub max_aliases: u32,
    /// The largest document, in bytes.
    pub max_bytes: u64,
}

impl ParseBudget {
    /// The default nesting depth. Deeper than any hand-written document and shallow enough
    /// that the recursive descent cannot exhaust the stack.
    pub const DEFAULT_MAX_DEPTH: u32 = 64;
    /// The default alias count.
    pub const DEFAULT_MAX_ALIASES: u32 = 1024;
    /// The default document size: 16 MiB.
    pub const DEFAULT_MAX_BYTES: u64 = 16 * 1024 * 1024;
}

impl Default for ParseBudget {
    /// The phase-0 defaults. A caller ingesting untrusted documents should tighten them;
    /// nothing may loosen them past what the §5.3 envelope admits downstream.
    fn default() -> Self {
        Self {
            max_depth: Self::DEFAULT_MAX_DEPTH,
            max_aliases: Self::DEFAULT_MAX_ALIASES,
            max_bytes: Self::DEFAULT_MAX_BYTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_inverted_span_is_empty_rather_than_negative() {
        let span = SourceSpan::new(SemanticId::NIL, 10, 4);
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());
    }

    #[test]
    fn the_default_budget_is_the_documented_one() {
        let budget = ParseBudget::default();
        assert_eq!(budget.max_depth, 64);
        assert_eq!(budget.max_aliases, 1024);
        assert_eq!(budget.max_bytes, 16 * 1024 * 1024);
    }

    #[test]
    fn a_span_displays_its_document_and_range() {
        let span = SourceSpan::new(SemanticId::from_bytes([0xab; 16]), 3, 9);
        assert_eq!(span.to_string(), "abababababababababababababababab:3..9");
    }
}
