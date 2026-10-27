# strum

Realizations considered: installed `strum` / `strum_macros` =0.28.0 (latest stable, 2026-02-22).
Research date 2026-10-27. Sources: strum 0.28.0 `src/lib.rs` (`EnumMessage`), repository
`pse-vocabulary/src/lib.rs` and `pse-diagnostics/src/vocabulary.rs` at `c2f8b485a`.

## Types

### strum/closed-vocabulary-derives
- **Need it serves:** closed string vocabularies (enum ↔ registry spelling) with parsing, display,
  variant lists and per-variant descriptive text.
- **Recognizable code shapes:** `macro_rules!` that stamps an enum plus `ALL`, `as_str`, `parse`,
  `description()` match arms.
- **Mechanisms and APIs:** `EnumString` (`parse_err_ty`, `parse_err_fn`), `Display`,
  `IntoStaticStr` (`const_into_str`), `VariantArray` (`VARIANTS`), `EnumMessage`
  (`get_message`, `get_detailed_message`, `get_documentation` from doc comments,
  `get_serializations`), `EnumProperty` (`#[strum(props(k = "v"))]`).
- **Obligations discharged:** spelling, parsing with a custom error, const string conversion, variant
  arrays, doc-comment-derived descriptions.
- **Residual caller obligations:** serde spelling (separate `#[serde(rename)]` or `into/try_from`
  String), JSON Schema, typed per-variant metadata (props are strings).
- **Semantic conditions:** `EnumMessage` getters are not `const fn`.
- **Limits and negative results:** no typed property values.
- **Evidence:** Interface-checked: strum 0.28.0 `EnumMessage` trait signatures.

## Current applicability

### [CBO-045](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-045) — Diagnostics implemented through a forwarding macro beside the miette derive (vocabulary half)
- **Types:** strum/closed-vocabulary-derives.
- **Realization:** installed 0.28.0 — already used by both `vocabulary!` copies.
- **Conclusion:** no further library route — strum is already the mechanism; the duplication is
  between two local macros. `EnumMessage::get_documentation` could replace the `description()` match
  (descriptions are already emitted as `#[doc = $description]`), at the cost of losing `const fn`.
- **What it would discharge / what remains local:** at most the `description()` arms; a single leaf
  crate owning one `vocabulary!` is local consolidation.
- **Semantic gaps against the CBO's preservation constraints:** none.
- **Unsettled premises and what would settle them:** **premise sharpening:** the copies differ by
  more than the `description` parameter — the `pse-diagnostics` copy also emits a hand `JsonSchema`
  impl, `From<_> for String` / `TryFrom<String>`, and `ALL` as a slice, while the `pse-vocabulary`
  copy emits per-variant `#[serde(rename)]` and `ALL` as a fixed array; a merged macro must carry
  those options (or the schema/serde route must be unified).
- **Evidence:** Interface-checked. Application Proposed.
