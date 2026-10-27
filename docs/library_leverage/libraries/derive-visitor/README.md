# derive-visitor

Realizations considered: not a dependency. Upgrade route `derive-visitor` 0.4.0 (released
2024-05-26; no later release as of 2026-10-27).
Research date 2026-10-27. Sources: docs.rs `derive_visitor` 0.4.0 crate page (fetched
2026-10-27); crates.io API.

## Types

### derive-visitor/derived-tree-traversal
- **Need it serves:** Derive recursive traversal (visit and mutate) of a heterogeneous AST,
  instead of hand-writing `walk_*`, `mutate_*`, `strip_*` and `paths_*` functions per node
  type.
- **Recognizable code shapes:**
  - Families of free functions (`walk_expr`, `walk_predicate`, `strip_expr`,
    `paths_expr`) that each repeat the same `match` over every variant.
  - `structural_eq` implemented as "clone, strip spans, compare".
- **Mechanisms and APIs:**
  - `#[derive(Drive, DriveMut)]` on AST types, with `#[drive(skip)]` on fields.
  - `#[derive(Visitor, VisitorMut)]` with `#[visitor(Type(enter), Other(exit))]` attributes
    and `enter_type`/`exit_type` methods.
  - `Event::{Enter, Exit}`.
  - Closure helpers: `visitor_fn`, `visitor_fn_mut`, `visitor_enter_fn`,
    `visitor_enter_fn_mut`.
  - The optional `std-types-drive` feature drives primitives and `String`.
- **Obligations discharged:**
  - Exhaustive child traversal kept in step with type definitions.
  - Enter and exit events.
- **Residual caller obligations:**
  - Fallible traversal with early return: the docs page does not show an error-returning
    visitor, so "return the first error unchanged" needs local state plus a flag.
  - Post-order transform semantics.
  - Collecting borrowed references with a lifetime, as in `paths(&self) -> Vec<&Path>`.
- **Semantic conditions:**
  - Traversal order follows field declaration order. Source order must match field order
    for "source order" guarantees.
  - The dispatch mechanism is not described on the docs page.
- **Version / build conditions:** Last release 2024-05. Maintenance status should be weighed.
- **Limits and negative results:**
  - Early termination and errors are not documented.
  - Primitives and `String` are not driven by default.
- **Evidence:** Interface-checked: docs.rs 0.4.0 crate page only. Source was not read.

### local-alternative/span-insensitive-equality (recorded only as a contrast, no library)
- **Contrast:** A span newtype whose `PartialEq` always returns `true` (or a derive with an
  ignore attribute) removes clone-strip-compare without any traversal library. This is
  noted because it discharges the `structural_eq` and `strip_spans` part of the walker
  more directly than a visitor derive does.

## Current applicability

### [CBO-037](../../../library_leverage/codebase/authoring-and-identity.md#cbo-037) — Grammar facts maintained in several hand-written forms (walker form)
- **Types:** `derive-visitor/derived-tree-traversal`.
- **Realization:** Upgrade route `derive-visitor =0.4.0`, a new dependency with a dated
  release line.
- **Conclusion:** Fits with conditions for the read-only visitors (`walk`,
  `walk_expressions`, `strip_spans`). Unsettled for `try_walk_mut`, which is a fallible
  post-order transform returning the first error, and for `paths()`, which collects borrowed
  references.
- **What it would discharge / what remains local:**
  - Discharged: exhaustive recursion in about half of the 700-line walker.
  - Remains local: fallible transform semantics, borrowed collection and any traversal-order
    guarantee that differs from field order.
- **Prerequisites, configuration or extension:** Derives on every AST node type, including
  `Path`, `PathSegment`, `Predicate` and `Equation`, with `#[drive(skip)]` on spans and
  scalars.
- **Semantic gaps against the CBO's preservation constraints:**
  - "Visit every expression child in source order" depends on field order matching source
    order.
  - Fallible early return is not documented.
- **Unsettled premises and what would settle them:**
  - Error-returning traversal support. Reading the 0.4.0 source (`Visitor` trait
    signatures) would settle it.
  - Whether a dated, single-maintainer crate is acceptable.
- **Evidence:** Interface-checked (docs page); application Proposed.
