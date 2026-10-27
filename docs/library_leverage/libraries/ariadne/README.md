# ariadne

Realizations considered: not a dependency. Upgrade route `ariadne` 0.6.0 (released
2025-10-28).
Research date 2026-10-27. Sources: docs.rs `ariadne::Span` and `ariadne::Config` 0.6.0
(fetched 2026-10-27); crates.io API.

## Types

### ariadne/source-id-span-rendering
- **Need it serves:** Render rich multi-label reports where each span names its source by a
  caller-chosen id.
- **Recognizable code shapes:** The same shapes as id-keyed diagnostics elsewhere: spans of
  `(document id, range)`.
- **Mechanisms and APIs:**
  - The `Span` trait has `type SourceId: PartialEq + ToOwned + ?Sized`, plus `source()`,
    `start()` and `end()`.
  - Built-in impls cover `Range<usize>` (`SourceId = ()`) and `(Id, Range<usize>)`, so spans
    carry their source id.
  - `Config::with_index_type(IndexType)` selects byte or char spans. The default is char
    spans.
  - Report building and source caches (`Report::build`, `Cache`, `sources(..)`) come from
    documentation knowledge and were not verified at 0.6.0.
- **Obligations discharged:** Multi-source excerpt rendering with id-keyed spans.
- **Residual caller obligations:**
  - The cache over the document store.
  - Conversion from domain errors.
  - Choosing byte indexing explicitly.
- **Semantic conditions:**
  - Spans default to character offsets. Byte-offset callers must set the index type, or
    excerpts misalign on non-ASCII text.
  - The authored language admits `°`, `∂` and `±`.
- **Version / build conditions:** The 0.x API has changed across minor versions.
- **Limits and negative results:** It is a separate diagnostic model with no `std::error`
  protocol.
- **Evidence:** Interface-checked: `Span` and `Config` pages at 0.6.0. Cache and Report APIs
  are not verified.

## Current applicability

### [CBO-038](../../../library_leverage/codebase/authoring-and-identity.md#cbo-038) — Author-facing diagnostics do not render source context
- **Types:** `ariadne/source-id-span-rendering`.
- **Realization:** Upgrade route, a new dependency.
- **Conclusion:** Ruled out here as the primary route, for the same reason as the other
  alternative renderer: it is a second diagnostic model beside the adopted protocol. There
  is also a char-offset default trap for a byte-span language containing non-ASCII symbols.
- **What it would discharge / what remains local:** Id-keyed multi-source rendering. The
  conversion from domain errors stays local.
- **Prerequisites, configuration or extension:** Byte index type and a cache over documents.
- **Semantic gaps against the CBO's preservation constraints:** Byte vs char offsets, as
  above.
- **Unsettled premises and what would settle them:** None beyond multi-document need.
- **Evidence:** Interface-checked; application Proposed.
