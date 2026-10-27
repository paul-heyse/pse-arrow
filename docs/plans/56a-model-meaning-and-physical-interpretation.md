---
title: "Plan 56a: Model meaning and physical interpretation"
status: complete
date: 2026-10-25
adrs: [ADR-0226]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, preparation, numerical-execution]
---

# Plan 56a: Model meaning and physical interpretation

## Completion audit — 2026-10-27

**Interface-checked:** independent review of A01–A03 at
`57cce1756318357a5797f99e83f1ad9115661368` supports completion within the accepted scope.
The checked occurrence/shared lexical and constant-admission products reach compiler/runtime
consumers; selected mode/support/view-role products reach steady, implicit and dynamic paths;
physical identities resolve through original retained result context and public inspection.
The recorded source/native/installed/MCP/CLI evidence was inspected at its original conditions;
no new compilation, scientific run or broad qualification was performed. No additional material
gap was established in A's accepted obligations. This does not certify unfinished B/C/F or the
whole series; the [coordinator audit](56-design-principle-alignment.md#completion-audit--2026-10-27)
owns that judgment. Benefits remain structural and unmeasured.

## Purpose and consumed basis

**Implemented:** modeling and compiler products govern expression occurrences, lexical binding,
mode targets and readable physical interpretation. This companion develops DA-003/005/006;
[Plan 56](56-design-principle-alignment.md) owns finding disposition, shared decisions and scope.
It uses the source and standard recorded there. Source inspection establishes the present seams,
not a numerical or performance result.

The integral foundation assessment reuses `SpecializedModel::resolved`, `pse-modeling::derived`,
compiler `Projection`/`NumericalAnnotations`, and `BindingQuantity`/`CanonicalConversionPlan`.
These already own the needed semantic distinctions. The addressed gap was incomplete authoritative products
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
| A01 | complete | 56a | Generated occurrences and shared lexical/constant admission consumed by both paths; old inventories deleted |
| A02 | complete | 56a | All current mode consumers use resolved inventory/support; skips/scans/re-mints removed |
| A03 | complete | 56a | Exact original physical context readable through Rust/Python/MCP, with typed domains and ambiguity refusal |

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

## Execution checkpoint — 2026-10-25

The operator authorized A01–A03 plus minimal working boundary-emission, governing-permission and
retained-context prerequisites. Source-owned physical identity aliases were explicitly selected;
ADR-0226 and its scoped review precede that dependent implementation. RC01–RC06 are not adopted.

A01 reuses the existing generated visitor. A02 consumes already-transitive resolved numeric/guard
support rather than inventing another walker. A03 uses the original run's retained physical rows
and receipt; the governing aggregate fact remains separate from selected-product usability.
Current stored Assessment facts suffice for this reader slice without a completion schema change.
No store reset, inline payload redesign, broad B01 migration or full F01 partition is selected.

A01–A03 and their authorized minimal prerequisites are implemented and accepted at the local
Linux scope below. Independent review corrections and functional qualification are complete.
Plan56 owns the resolved finding dispositions; supporting companions record only the consumed
prerequisite slices. Their remaining packages and decisions are not claimed complete.

## Verification and Outcome — 2026-10-25

**Implemented:** selected-source discovery and checked admission share generated occurrences and
one lexical binding owner; constant formulation arguments use the existing checked evaluator.
Compiler inventories own all current steady/nested/dynamic targets, transitive support, physical
contracts, view roles, closure endpoints and difference-valued inventory budgets. Runtime binds
values and consumes these facts. The parallel expression inventory, rendered-name dependency
searches, lossy target tuples, runtime closure target reconstruction and duplicate bound identity
recipes are removed. Current starts, physical guards, scientific comparators and thresholds remain.

**Implemented:** six physical identity domains re-export their native owners through generated
Rust/Arrow/Python boundaries. Bounded physical descriptions and named resolution serve Rust,
Python, actual MCP and derived CLI callers. Protected result pages reopen the exact original
physical rows and context receipt, resolve the persisted relation version exactly, bind cursors
to that closure, and retain protection through response construction. Missing or invalid receipt
refuses physical interpretation while raw diagnostics remain readable. Original governing permission
is copied separately from selected-product usability. Host and campaign YAML meaning parsers
were replaced by admitted physical-owner resolution.

**Tested — baseline zero, local Linux:** the following are distinct scopes, not an additive total.
Rust unit rows use `scripts/pse-env -- python3 -m scripts.arrow_validation nextest run`,
`--lib --locked`, and the stated package/filter; the resolver explicitly verifies Arrow
force-validation for each consuming closure. Native rows additionally use the recorded native
capabilities or the composed `just native-test` controller.

| Command and selected scope | Result and conditions |
|---|---|
| `just check`, plus `just check-package pse-compiler -p pse-runtime` after the late consumer correction | Workspace/all-target compile and changed-package compile passed; no warnings/errors |
| `-p pse-modeling -E 'test(occurrence_argument_tests) \| test(selected_source::) \| test(expression::occurrences::tests) \| test(occurrence_lexical_tests)'`; separate `test(check::selection_tests)` | 33 passed and 6 passed respectively; exact occurrences, binders, pending/shadowing and constant grammar |
| `-p pse-compiler -E 'test(workspace::modeling::executable::policy::tests) \| test(kernel_conservation_scatter_preserves_mixed_contracts_and_homogeneous_control)'` | 9 passed, 0 failed; repeated after the complete closure inventory correction |
| Physical alias/base/schema controls in `pse-codegen`/`pse-quantity`; `-p pse-relations -E 'test(source_owned_physical_alias_roundtrips)'`; `-p pse-codegen -E 'test(evaluated_array_alias_follows_its_record_definition)'` | 3, 1 and 1 passed respectively; actual Arrow metadata/bytes/null/refusal boundary and evaluated alias ordering |
| Native runtime filter: owned `contextual_closure` controls, `authored_conservation_point_inventory_uses_difference_transfer_convention`, `goal_accuracy_dynamic_shares_one_real_comparator_for_samples_endpoint_and_integral`, `dynamic_refined_goal_consumes_the_compiler_physical_target_and_role`, `dynamic_observable_nominal_is_bound_from_the_compiler_inventory`, and `contextual_closure_fit_factory_controls_preserve_explicit_experiment_profiles` | 11 passed, 0 failed; `--native=solver,klu,isolation`, Diffsol/root-isolation/KINSOL features, scientific inputs `build/scientific-inputs/1792963727547520285.json`; Nextest `f69746f3-4a04-43bc-89d6-575fcde48da4` |
| `-p pse-runtime -E 'test(physical_relation_locator)'` | 2 passed, 0 failed; actual producer `RelationKey::to_string()` with declared versions and malformed/missing/unknown-version refusals |
| `PSE_SURREAL_STATE=<owned> just native-test -p pse-runtime --lib -E 'test(canonical_result_physical_) \| test(canonical_result_missing_physical_) \| test(canonical_complete_export_cancelled_at_seal_preserves_original_completion) \| test(authored_shooting_fixture_solves) \| test(source_index_lexical_adapter) \| test(source_index_references_keep) \| test(indexed_imports_keep_pending) \| test(complete_namespace_inventory) \| test(physical_projection) \| test(physical_relation_locator)'` | 16 passed, 1 fixture failure; Nextest `f5e04ca4-c35a-487a-b664-6388f56afb82`. The fixture failed before its adverse oracle at head-CAS, not at physical admission |
| Same native command, only `test(canonical_result_physical_context_requires_the_original_receipt_payload_not_just_its_hash)` after fixture repair | 1 passed, 0 failed, both empty and wrong-kind branches exercised; Nextest `f65e5642-8b2e-42c1-851a-5e60773236cd`. The selected canonical scope has no remaining failed obligation; this is not a claim of one clean 17-test invocation |
| `scripts/pse-env -- python3 -m unittest scripts.tests.test_capability_native scripts.tests.test_capability_execution_basis` | 41 passed; host contracts, separate from actual receiver evidence |
| `just py-unit python/pse/tests/test_capability_adapters.py::test_physical_context_python_mcp_and_cli_use_the_same_native_projection --basetemp <owned>` | 1 passed, 0 failed; actual correctness extension, MCP and derived CLI projection equality |
| `PSE_SURREAL_STATE=<owned> just native-python <output> <stored-reader node> <residual-LM node> <fixed-fitting node> -m 'component or integration' --basetemp <owned> --tb=short` | 2 fitting journeys passed; stored-reader assertion failed on versioned relation lookup. Nodes are `test_capability_stored_result_preserves_native_arrow_window`, `test_residual_lm_public_result_exposes_actual_stopping_and_original_qualification`, and `test_fixed_fitting_sources_round_trip_and_use_shared_result_lifecycle` |
| Same native Python route, only the stored-reader node, `-m integration`, after locator repair and sole observer acquisition | 1 passed, 0 failed; original physical context available, governing permission true, product usability unknown, exact Arrow and actual MCP document equality. Receipt: `build/qualification/56a-reader-WKtsjO/native-python.xml` |
| `just codegen`; generated candidate checks and actual compiled-API stub preparation | All 10 schema targets published/compared successfully; no hand-edited generated outputs. Evaluated Python aliases import correctly |
| `just hygiene --live` and `just governance` | Initial bundles respectively failed typecheck/generated capability reference and generated capability reference; all other leaves passed, including 25 governance tests, default/no-default Clippy and Rust docs. `just typecheck` and `just codegen-capabilities-check` passed after repair; no baseline waiver |
| `just lint-native-contracts`, `just lint-native-data`, `just clippy-no-default` | Passed at scope end with zero warnings/errors; workspace native-data lint repeated after the reader fix |
| `just doctor` | Passed, zero warnings; current correctness extension admitted. Installed profile is native/dev with force-validation, native-solvers and solver-highs |

**Tested — final closeout:** `just turn-end --live` passed; `just docs` published 677 chapters;
`scripts/pse-env --docs -- python3 -m scripts.document_lifecycle validate` validated 1,483 native
scope items. Final `just py-sync-native-correctness` admitted the refreshed compiled API and
provenance; the actual Python/MCP/CLI physical-projection test was repeated on that installed
image and passed 1/1. These are distinct from the earlier stored-reader and scientific journeys.

**Tested:** separate canonical deployment `build/qualification/56a-canonical-7wt3jw_a` used the
existing functional profile, ordinary host configuration and a no-resident lifecycle. The existing
selected global store was preserved. Same-state reconfiguration/readmission after the receiver
rebuild retained its database; no storage reset or payload/schema migration occurred. Final
quiesce, authenticated drain, stop and status all exited zero: inactive service, writes closed,
no owned listener/process and all registered scopes/pending launches drained. Database and
receipts remain retained. A broader activity inventory encountered unrelated disappearing units;
the selected invocations' independent terminal and drain evidence was complete.

**Independent assessment:** the scoped source-owned identity design review accepted the selected
RC07 direction before dependent code. Fresh implementation review found and verified repairs for
refined targets, nested nominal ownership, sensitivity roles, repeated scans, pre-copy bounds,
receipt admission and the later closure/inventory/integral residue. The fitting derivative-selection
seam, exact versioned locator and adverse fixture were independently reread. These are source
assessments, separate from the actual functional evidence above. ADR-0226 remains proposed.

### Mistakes made and corrected

The first implementation left closure targets partly reconstructed at runtime and could give an
affine inventory a point-valued compiler budget. Independent review exposed that missed consumer;
compiler-issued difference contracts/endpoints and runtime consumption now replace it. A reused
resolved fitting profile also retained positional sensitivity axes alongside the fitting-owned
semantic selection; the caller now clears those inherited axes without changing numeric controls.

Actual stored-reader qualification exposed a name-only lookup of the producer's versioned relation
locator. Exact registered-version selection fixed it; malformed spellings still refuse. The
adverse-receipt fixture initially failed CAS before reaching its oracle. It now uses the actual
parent and explicitly removes the inherited context object for the empty case. Assertions and
production CAS remain unchanged. Native-only fixture type/API errors were also corrected.

Concurrent preparation initially overwrote a mutable feature-specific `xtask` output, so the guarded
initializer refused it. A serialized helper handoff completed initialization. Observer contention
correctly refused one simultaneous Python run before tests; the successful rerun acquired the
existing exclusive allocation. A component-only selector selected no tests for an integration
node and was corrected. None of those attempts establishes a product pass. The earlier
reduced-feature dynamic accuracy refusal likewise remains distinct from its passing intended
root-isolation configuration; no scientific comparator or threshold was relaxed.

### Deliberate deviations and limits

The occurrence visitor already existed and was reused. Native physical wrappers remain the one
Rust authority through generated aliases under selected RC07. Minimal B01/C03/F01 prerequisites
use the existing Assessment and retained source closure, with the existing directional emitter;
Completion versioning, full covariance migration, general operation generation and worker reuse
are outside this delivery. RC01–RC06 are not adopted. Related Plan56 records are reconciled; other plan series retain
their independent scope and status.

**Not run / not claimed:** a fresh exhaustive feature matrix (no dependency or feature declarations
changed), full parity/solver-fleet qualification, distribution or other platforms, and performance
campaigns. Structural work removal is implemented; no unmeasured speed, capacity or memory benefit
is asserted. The active series and prerequisite readers still depend on this accepted scope and
its receipts, so this completed companion is retained rather than prematurely retired.
