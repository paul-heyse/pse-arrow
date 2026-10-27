# miette

Realizations considered:
- Installed `miette` 7.6.0 (released 2025-04-27; still the latest release on 2026-10-27).
  The lock resolves it with default features only (`derive`). Its dependency list is
  `cfg-if`, `miette-derive` and `unicode-width`, so `fancy-base` is not active.
- Configuration route on the same pin: enable `fancy-base` (adds `owo-colors` and
  `textwrap`) or `fancy-no-syscall`. No version change is needed.

Research date 2026-10-27. Sources:
- Installed source `miette-7.6.0`:
  - `Cargo.toml` feature table;
  - `src/handlers/mod.rs` (graphical and theme gated on `fancy-base`; `narratable`, `json`
    and `debug` ungated);
  - `src/handlers/narratable.rs` (`render_report`, `render_snippets`, `render_related`);
  - `src/protocol.rs` (`LabeledSpan::{new_with_span, new_primary_with_span, at, at_offset,
    underline}`);
  - `src/named_source.rs` (`NamedSource::new`, `with_language`);
  - `src/source_impls.rs`.
- Context7 `/zkat/miette` (README `with_source_code`, `MietteDiagnostic::and_label`).

## Types

### miette/late-bound-source-labeled-rendering
- **Need it serves:** Render an error with an annotated excerpt of the source it refers to.
  The error value carries only spans, and the source text is attached later at the boundary
  that holds it.
- **Recognizable code shapes:**
  - Errors carrying `(document id, start, end)` or a byte offset, rendered as plain text.
  - A `Diagnostic` implementation whose `labels()` and `source_code()` only forward to a
    wrapped error.
  - No `LabeledSpan` or `NamedSource` anywhere in the language crates.
- **Mechanisms and APIs:**
  - `Diagnostic::labels() -> Option<Box<dyn Iterator<Item = LabeledSpan>>>`, implemented by
    hand or through `#[label("...")]` on a `miette::SourceSpan` field with
    `#[derive(Diagnostic)]`.
  - `Diagnostic::source_code() -> Option<&dyn SourceCode>`.
  - `Report::with_source_code(impl SourceCode)` attaches source after creation. The
    diagnostic's own source still wins.
  - `NamedSource::new(name, text)` gives the excerpt a display name; `.with_language(..)`
    serves highlighters.
  - `SourceCode` is implemented for `str`, `String`, `Arc<T>`, `Cow`, `Vec<u8>` and `[u8]`.
    A custom `SourceCode` can serve text from any store.
  - Renderers:
    - `NarratableReportHandler::new().render_report(&mut String, &dyn Diagnostic)` is plain
      text and screen-reader-oriented. It is available without fancy features and renders
      snippets.
    - `GraphicalReportHandler` (`new_themed(GraphicalTheme::unicode_nocolor())`) needs
      `fancy-base`.
    - `JSONReportHandler` is always available.
- **Obligations discharged:**
  - Line and column computation.
  - Excerpt selection with context lines and overlapping-label merging.
  - Label rendering and multi-diagnostic (`related`) rendering.
  - Consistent formatting of code, help and URL.
- **Residual caller obligations:**
  - Converting domain spans to `miette::SourceSpan`, which is `(offset, len)` in bytes.
  - Owning a document-id to (name, text) map at the presentation boundary.
  - Choosing text, graphical or JSON per consumer.
  - Keeping source text out of persisted or wire forms.
- **Semantic conditions:**
  - One diagnostic renders against one `SourceCode`. A diagnostic whose spans point into
    several documents must split into `related` diagnostics, each with its own
    `source_code()`. `render_related` uses `rel.source_code().or(parent_src)`.
  - Offsets are bytes. Spans must fall on UTF-8 boundaries.
  - A report whose diagnostic already returns `Some(source_code)` ignores
    `with_source_code`.
- **Version / build conditions:**
  - The graphical handler and themes require the `fancy-base` feature or a feature that
    includes it.
  - Under workspace feature unification, enabling a feature anywhere activates it for all
    consumers of the crate in that build.
- **Limits and negative results:**
  - No built-in multi-file source map keyed by an arbitrary id. Each diagnostic has one
    source.
  - No incremental line index cache across many diagnostics on the same file. The
    renderers compute per render.
- **Evidence:** Interface-checked (installed 7.6.0 source files named above; Context7 README
  excerpts).

## Current applicability

### [CBO-038](../../../library_leverage/codebase/authoring-and-identity.md#cbo-038) — Author-facing diagnostics do not render source context
- **Types:** `miette/late-bound-source-labeled-rendering`.
- **Realization:** Installed pin 7.6.0.
  - With current features, `NarratableReportHandler`, `JSONReportHandler` and
    `Report::with_source_code` are usable.
  - Graphical rendering needs `fancy-base` on the same pin. The workspace pin comment says
    library error projections preserve causes "without a rendering dependency", so this is
    a deliberate premise to revisit. It is not a version move.
- **Conclusion:** Fits with conditions. The protocol pieces exist and are already plumbed
  through the forwarding macro. The work is implementing `labels()` from the domain span and
  attaching a source at the driver or presentation boundary.
- **What it would discharge / what remains local:**
  - Discharged: excerpt and caret rendering, line and column computation, and related
    rendering.
  - Remains local:
    - `labels()` for `AuthoringError` and the check and specialize errors, mapping the
      domain `SourceSpan{document_id, start, end}` to
      `LabeledSpan::new_primary_with_span(Some(expected), (start, end-start))`.
    - The id-to-(path, text) source map.
    - Choosing where rendering happens: CLI text, MCP text or JSON, Python.
- **Prerequisites, configuration or extension:**
  - The shared diagnostic macro gains an optional own-labels hook. Today `labels` and
    `source_code` only forward.
  - Source attachment at the boundary that owns the text. The admission driver's document
    load holds `path`, `text` and the document id at the parse call. A `Report` built there,
    or at the CLI, wraps the error with `NamedSource::new(path, text)` through
    `with_source_code`, or a custom `SourceCode` over a document map.
  - Spans into several documents need related diagnostics per document.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Spans name documents by `SemanticId`:* the map resolves id to path for display only.
    Nothing persists.
  - *Source text not always available at the error site:* satisfied, because source is
    attached late.
  - *Codes:* unchanged; `code()` is already implemented.
  - *Wire and Arrow forms must not carry source text:* satisfied if rendering happens only
    at presentation. The typed `DiagnosticFacts.locations` (document id, start, end)
    already carries what a non-Rust renderer would need.
  - Python and MCP consumers receive facts, not `miette::Report`s. A Python excerpt would
    need either a Rust-side render exposed as a string, or a local Python renderer over
    facts plus text.
- **Unsettled premises and what would settle them:**
  - Where author-facing rendering should live: Rust CLI, the Python boundary, or both.
    Inspecting the CLI entry for `pse authoring` and the Python error surface would decide
    whether a Rust-rendered string crossing PyO3 is acceptable.
  - Whether `fancy-base` is acceptable workspace-wide, or the narratable plain-text form
    suffices.
- **Evidence:** Interface-checked (installed source); application Proposed. Counterevidence
  search: `GraphicalReportHandler`, `NarratableReportHandler`, `JSONReportHandler` and
  `miette::Report` across `crates/` and `xtask/` *.rs found no use. `python/pse` has no
  source-excerpt renderer; its `_render_inspection` prints runtime inspection documents.
  Search was limited to those trees.
