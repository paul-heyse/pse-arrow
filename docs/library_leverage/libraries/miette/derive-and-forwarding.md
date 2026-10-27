# miette (derive)

Realizations considered: installed `miette` / `miette-derive` =7.6.0 (latest stable 7.6.0,
2025-04-27).
Research date 2026-10-27. Sources: miette-derive 7.6.0 source (`diagnostic_arg.rs`, `diagnostic.rs`,
`code.rs`, `forward.rs`); miette 7.6.0 `README.md` (`#[diagnostic_source]`); upstream
`tests/derive.rs` (main) for `forward(...)` cases.

## Types

### miette/derive-forwarding-and-overrides
- **Need it serves:** implement `miette::Diagnostic` declaratively per error variant, including
  delegation to an inner diagnostic with selective overrides.
- **Recognizable code shapes:** a `macro_rules!` that hand-implements every `Diagnostic` method as
  "own value or forward to child"; per-variant code tables; `{ None }` placeholder clauses.
- **Mechanisms and APIs:** `#[derive(Diagnostic)]` with container/variant
  `#[diagnostic(code(path) | code = "..", severity(..), help(".. {field}"), url(".."),
  forward(field))]`, `#[diagnostic(transparent)]` (forward everything to the single field),
  field attributes `#[diagnostic_source]`, `#[related]` (iterator of diagnostics), `#[source_code]`,
  `#[label]`. `forward(field)` delegates every method not overridden on that variant
  (`forward_or_override_enum`): e.g. `#[diagnostic(code(foo::bar::overridden), help("{1}"),
  forward(0))]` is an upstream test case.
- **Obligations discharged:** all miette methods (code display, help, url, severity, labels,
  source_code, related, diagnostic_source) with forwarding and format-string help.
- **Residual caller obligations:** `code` is a **static string** (a `syn::Path` or string literal,
  stored as `Code(String)`); it cannot be an expression or a typed enum value. Any typed projection
  (typed code enum, structured facts, child lists of a different trait) needs its own impl or derive.
- **Semantic conditions:** `forward` delegates per variant; overrides win; `transparent` excludes
  other args.
- **Version / build conditions:** stable in 7.x; `forward(...)` is implemented and tested upstream but
  little documented in the README.
- **Limits and negative results:** no dynamic/typed code; no hook to emit a second trait.
- **Evidence:** Interface-checked: miette-derive 7.6.0 source (`DiagnosticArg::{Transparent, Code,
  Severity, Help, Url, Forward}`, `Code` parse of `LitStr`/`Path`), upstream `tests/derive.rs`
  lines using `forward(span)` / `forward(0)`.

## Current applicability

### [CBO-045](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-045) — Diagnostics implemented through a forwarding macro beside the miette derive
- **Types:** miette/derive-forwarding-and-overrides.
- **Realization:** installed 7.6.0.
- **Conclusion:** fits with conditions for the miette half only — `forward(field)` + per-variant
  `help(...)` + `#[related]` reproduce the macro's miette methods, but the typed half
  (`TypedDiagnostic::diagnostic_code` → `DiagnosticCode`, `diagnostic_facts`, `diagnostic_children`,
  typed forwarding) has no derive, and miette's string code would become a second declaration of each
  code. Single authority needs a local `#[derive(TypedDiagnostic)]` that emits both traits (or emits
  `#[diagnostic(code = "...")]` from the typed code's spelling).
- **What it would discharge / what remains local:** the hand miette impl in `impl_diagnostic!`;
  local: the typed projection, facts, and the `disposition()` class map.
- **Prerequisites, configuration or extension:** a small proc-macro crate (workspace already depends
  on `syn` 3 for codegen). **Premise sharpening:** existing derive sites already show the double
  declaration — `pse-operations/src/canonical.rs` has `#[diagnostic(code(runtime::infrastructure))]`
  *and* a separate `impl TypedDiagnostic for CanonicalError`; 10 non-generated files derive
  `miette::Diagnostic`. Of the 41 `impl_diagnostic!` sites (count confirmed), an approximate
  clause scan finds non-`None` `forward` in ~13, `facts` in ~9, `help` in ~5, `related` in ~2,
  `source` in none — most sites pass the placeholders.
- **Semantic gaps against the CBO's preservation constraints:** typed `DiagnosticCode` and the facts
  projection are not expressible in miette derive; forwarding rules match.
- **Unsettled premises and what would settle them:** whether a string code derived from the typed
  code (`DiagnosticCode` `Display`) is acceptable as the miette code everywhere (it is what the macro
  emits today).
- **Evidence:** Interface-checked. Application Proposed.
