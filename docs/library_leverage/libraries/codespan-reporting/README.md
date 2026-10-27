# codespan-reporting

Realizations considered: not a dependency. Upgrade route `codespan-reporting` 0.13.1
(released 2025-10-22, MSRV 1.67).
Research date 2026-10-27. Sources: docs.rs `codespan_reporting::files::Files` 0.13.1
(fetched 2026-10-27); crates.io API. `Diagnostic` and `Label` were not re-read at 0.13.1.

## Types

### codespan-reporting/id-keyed-multi-file-rendering
- **Need it serves:** Render diagnostics whose labels point into source files identified by
  a caller-chosen id, from a caller-owned file database.
- **Recognizable code shapes:**
  - Spans keyed by an opaque document id rather than a path.
  - Several documents involved in one error.
- **Mechanisms and APIs:**
  - The `files::Files<'a>` trait has associated types `FileId: Copy + PartialEq`,
    `Name: Display` and `Source: AsRef<str>`.
  - Required methods are `name`, `source`, `line_index` and `line_range`. `line_number`,
    `column_number` and `location` are provided.
  - Built-ins: `SimpleFile` (`FileId = ()`) and `SimpleFiles` (`FileId = usize`).
  - `diagnostic::{Diagnostic<FileId>, Label::primary(file_id, range)}` and
    `term::emit(writer, &config, &files, &diagnostic)`. These are documentation knowledge
    and were not re-read at 0.13.1.
- **Obligations discharged:** Excerpt rendering with line indexes supplied by the file
  database, and multiple labels per diagnostic.
- **Residual caller obligations:**
  - Implementing `Files` over the document store.
  - Converting from the error type. It is a separate diagnostic model, not a `std::error`
    protocol.
- **Semantic conditions:**
  - `FileId` must be `Copy`. A 16-byte id type qualifies if it is `Copy`.
  - Ranges are byte ranges.
- **Version / build conditions:** Rendering uses `termcolor`-style writers. Color can be
  disabled.
- **Limits and negative results:**
  - It is a second diagnostic model beside an existing `Diagnostic` protocol.
  - The fetched trait page does not state whether one diagnostic can label several files.
    Multi-file labels are commonly documented elsewhere but were not verified here.
- **Evidence:** Interface-checked: `Files` trait page at 0.13.1. Other items come from
  documentation knowledge.

## Current applicability

### [CBO-038](../../../library_leverage/codebase/authoring-and-identity.md#cbo-038) — Author-facing diagnostics do not render source context
- **Types:** `codespan-reporting/id-keyed-multi-file-rendering`.
- **Realization:** Upgrade route, a new dependency.
- **Conclusion:** Ruled out here as the primary route. It would add a second diagnostic
  model beside the adopted miette protocol, which already forwards codes and labels. It is
  worth retaining only if one excerpt must label several documents, which miette expresses
  through related diagnostics.
- **What it would discharge / what remains local:**
  - A `Files` implementation over a `SemanticId` map would serve the id-keyed source map
    natively.
  - The conversion from domain errors to codespan diagnostics remains local, and would be
    duplicated relative to miette.
- **Prerequisites, configuration or extension:** A document store implementing `Files`.
- **Semantic gaps against the CBO's preservation constraints:** None found for byte spans
  and id keys.
- **Unsettled premises and what would settle them:** Whether multi-document excerpts are
  needed. Checking how often errors carry spans from more than one document would settle
  it.
- **Evidence:** Interface-checked (trait page); application Proposed.
