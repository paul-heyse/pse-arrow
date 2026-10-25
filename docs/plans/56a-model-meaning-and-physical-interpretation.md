---
title: "Plan 56a: Model meaning and physical interpretation"
status: draft
date: 2026-10-25
adrs: []
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, preparation, numerical-execution]
---

# Plan 56a: Model meaning and physical interpretation

## Purpose and consumed basis

**Proposed:** modeling and compiler products govern expression occurrences, lexical binding,
mode targets and readable physical interpretation. This companion develops DA-003/005/006;
[Plan 56](56-design-principle-alignment.md) owns finding disposition, shared decisions and scope.
It uses the source and standard recorded there. Source inspection establishes the present seams,
not a numerical or performance result.

The integral foundation assessment reuses `SpecializedModel::resolved`, `pse-modeling::derived`,
compiler `Projection`/`NumericalAnnotations`, and `BindingQuantity`/`CanonicalConversionPlan`.
These already own the needed semantic distinctions. The gap is incomplete authoritative products
and consumers reconstructing them, not a missing universal model or unit library. Keep physical
and numerical admission, original guards, immutable preparation and value-only rebinding.

## Target and alternatives

### Exact occurrences and one lexical rule

The registry's expression field facets declare grammar and role. Derive the selected occurrence
visitor from `catalog/modeling_expression.rs` and `model/field_facets.rs`, retaining declaration,
field role and ordinal rather than reconciling by expression text. Two equal strings at distinct
occurrences stay distinct. One modeling-owned lexical function handles function and scope
parameters, coordinate-map, response, reconstruction and reference-translation binders; both the
checker and runtime source index consume it. Preserve nearer-scope shadowing and
`Lookup::{Pending, Absent, Found}`. Pending is not absence or permission to bind an outer name.

`realization.argument` consumes its declared expression grammar. Where a formulation requires a
constant scalar (`derived_big_m`, `hull`), checked resolution admits that constant or gives a typed
unsupported/nonconstant diagnostic. Delete the hidden independent `f64` text parse. This does not
introduce arithmetic evaluation outside the existing library/modeling route.

Keeping the manual selected-source inventory would leave every new DSL field dependent on two
edits; a new runtime parser/registry would add a competing authority. Select the existing registry
facets plus modeling lexical owner instead.

### Compiler-owned mode inventory, runtime value binding

Modeling owns resolved roles and derived identity recipes. The compiler combines those facts with
selected projection structure into immutable per-mode numerical target inventories. Each entry
identifies its resolved target/source, role, physical quantity, observation location and resolved
support, with explicit supported/not-applicable/refused meaning. Runtime supplies current case
values, provider observations and admitted profile overlays; evaluated values never enter Salsa.

Steady variables/observables, dynamic states/algebraic rows/closure and inventory targets,
time/terminal/integral observations and nested residual positions are genuinely different. Encode
those differences once using the existing specialized integrals, closures, balances and initial
conditions. Do not clone one classification into all modes and silently drop the unmatched targets.
An authored annotation the chosen mode cannot honor is diagnosed, not skipped.
Compiler-owned view/role policy also decides inner-unknown filtering and sensitivity/covariance
role admission; runtime must consume that policy rather than restate it inline.

Frozen-expression checks consume resolved transitive support. Unknown or unresolved support
refuses distinctly from an expression known to contain no changing coordinate. Delete synthesized
`s_<hex>` name matching, whole-symbol scans and fail-open unmatched-path branches. Do not assume
current direct `NumericalAnnotation::reads` covers transitive derived symbols without checking it.
Use the existing modeling derivation catalogue for requirement identities; place the
reconstructed-original-bound recipe in an existing shared owner below backend and runtime.

A runtime-owned second inventory is unselected: it preserves the compiler/runtime disagreement.
A compiler inventory containing evaluated provider values is also unselected: it breaks pure
semantic reuse. Preserve steady outer filtering, dynamic start precedence, scaling, guards and
selected/nested demand while moving structural interpretation.

### Readable physical context from the existing authority

Preserve I20: project the existing selected physical registry and conversion owners. Typed
transport distinguishes quantity, unit, basis and reference-state identity domains. Readable
context carries exact ids plus names/symbols, dimension, quantity kind, basis/subject, datum or
reference state and convention where relevant. Value units and difference/tolerance units remain
distinct for affine quantities. Completeness and unavailable original closure are explicit.

Resolve named input addresses through the selected physical registry, refusing ambiguity. Do not
add a global alias table or per-host YAML interpreter. Python, native result inspection and MCP
consume the same projection, keyed by the exact original physical-context identity. Canonical
readers obtain that retained closure through C; they never substitute the latest registry.
Executable exchange already preserves authored physical documents and continues to do so.

This corrects direct reader usability without asserting all exported meaning is lost. Existing
opaque identities remain exact; source-level recoverability is different from a usable result view.

## Packages and implementation visions

### A01 — Occurrence and binder authority

Generate the occurrence visitor, move both lexical consumers and constant-form admission to the
same checked modeling operations, and delete `selected_source.rs::requirements`' parallel text
inventory and lossy reconciliation. Migrate `check.rs::CheckedLexical` and
`workflow/modeling/canonical.rs::SourceIndex` together. Adding one expression field then changes
its declaration and focused semantic expectations, not another handwritten inventory.

### A02 — Resolved per-mode numerical products

Publish inventories/support from `pse-compiler/.../executable/policy.rs`; migrate steady
`cases.rs`, dynamic `dynamics.rs`, nested/fit/shooting consumers of these preparations. Remove
`hints`' lossy target-kind tuple, target-set reconstruction, repeated dynamic row lookup and
independent derivation recipes. Migrate `cases.rs::resolve_prepared_case_with_current_view`
inner-demand filtering and sensitivity/covariance role admission to the compiler-owned view
policy, covering DA-031's residue within DA-006. Include the compiler products and all current consuming modes in
the same integrated delivery; a new field nobody reads does not complete A02.

### A03 — Physical identity and interpretation projection

Extend declared identity domains and Rust/Python documents; expose the existing physical owner
projection and named resolution. Migrate result views, inspection context/cursors and host
presentations with C03's retained-context locator and B01's permission fact. An immutable cursor
binds original physical context and cannot resolve through another context with the same symbol.
Regenerate through codegen and delete fixture-style host parsing as a required user path.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| A01 | planned | 56a | Generated occurrences and shared lexical/constant admission consumed by both paths; old inventories deleted |
| A02 | planned | 56a | All current mode consumers use resolved inventory/support; skips/scans/re-mints removed |
| A03 | planned | 56a | Exact original physical context readable through Rust/Python/MCP, with typed domains and ambiguity refusal |

A01 enables A02's source attribution. A03's owner projection can be implemented independently,
but persisted/public integration requires C03 and F01's boundary emission. Do not wait for all of
C or F to design or test the pure physical projection. Registry, identity and document changes
coordinate at the shared declaration owner; C05 handles the selected fresh-store cutover.

## Acceptance and preservation

**Planned evidence:** independently named occurrence/role cases include equal text at different
ordinals, each binder form, shadowing, pending nearer scope, malformed DSL and constant/nonconstant
form arguments. Numerical inventory cases cover steady, nested and dynamic targets, observable
nominals, closure/inventory targets, unsupported annotations and transitive coordinate dependence.
Include inner unknowns with outer/inner demand changes and sensitive/covariance views whose role
admission differs; no duplicate runtime branch may silently alter the compiler selection.
Compare original physical values, guards, starts and tolerances; generated expectations alone are
not an oracle for semantic roles.

Physical controls use swapped unit/quantity ids, identical numbers with different bases/references,
affine values versus differences, repeated symbols in different contexts, missing original closure
and result inspection without out-of-band YAML parsing. Exercise both executable bundles and
bounded result views with their distinct contracts. Targeted validated package tests accompany
implementation; G owns composed installed/native journeys. No tests ran to establish this proposal.
