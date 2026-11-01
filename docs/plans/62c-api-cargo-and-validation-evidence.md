---
title: "Plan62c: API, Cargo and validation evidence"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Companion to retained Plan62; API, Cargo and native runnable evidence supports subsequent consumers."
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/62-code-intelligence-capability-expansion.md
doc_topics: [current-work, libraries, validation]
review_sources: [docs/functional_reviews/code-intelligence-capability-expansion-2026-11-01.md]
adrs: [ADR-0233, ADR-0235]
---

# API, Cargo and validation evidence

## Target and foundation assessment

**Proposed:** develop O08–O10 and the API contribution to O03 from the
[source review](../functional_reviews/code-intelligence-capability-expansion-2026-11-01.md).
The [coordinator](62-code-intelligence-capability-expansion.md) owns dispositions and shared
contracts; this document owns C01–C04 execution state. Baseline `4b2f8df34` and the operator's
uncommitted library overviews are distinct implementation and research inputs.

The API owner already retains rustdoc artifacts, original declarations, public routes,
source-qualified external definitions and public-structure comparison. Candidate records already
carry completeness, reexports, attributes, docs/examples and unknown ordinary-build availability.
`SourceBridge` checks artifact, package/library/features/target, dependency profiles, environment
and source bytes before nominating semantic correspondence. Extend these owners rather than
create another loader or comparison engine. Matching names/spans alone establish no native identity.

Authored CommonMark fences already preserve hidden setup, displayed/expanded text, attributes,
expectation and `not_run`. Fence-level expansion is useful reading evidence, not the native
compiler's wrapper. Dependency inspection already offers declarations/features/workspace uses,
shortest inclusion paths and inverse feature-member lookup. Existing correctness admission owns
actual planned compiler units. Native related-test facts and guarded source-to-test-selection
routes also exist. Neither unit discovery nor a test runner is missing.

API artifacts own declaration meaning, the native world contextual resolution/obligations,
metadata package observations, correctness admission selected units and validation owners actual
discovery/execution. Structural API work requires no semantic service. Contextual applicability
requires an established API-to-source-to-world bridge. Reuse one compatible artifact/admission
within an inquiry and hydrate only demanded detail. Core3.4 and the selected heuristics/binding
shape this target; no scientific acceptance is inferred from code relationships.

## Package state

| Package | Status | Dependencies | Completion boundary |
| --- | --- | --- | --- |
| C01 | complete | T01; T02 API slice; A02 for contextual refinement only | Rich schema61 structural contracts and native contextual refinement consumed publicly |
| C02 | complete | T01; T02 example slice | Original/native format2 transformed examples separated; unselected scraped/Markdown modes retain triggers |
| C03 | complete | T01; T02 package/unit/world slice | Real Cargo unit/feature/rename/host-target evidence joined with selected world; existing graph owners retained |
| C04 | complete | A02 for semantic runnables; T02 relevance slice; C02/C03 only for their contributors | Qualified native runnable candidates use shared guarded current-checkout test-selection owner on both public surfaces |

Dependencies refer to working slices, not whole documents. C01's structural work does not wait
for A02; C04's ordinary runnables do not wait for native doctest extraction or all C03 comparisons.
Source-qualified package selection remains useful when narrower native candidates are unavailable.

## C01 — Rich contracts and contextual availability

Extend `api/query.rs` and current inspection/comparison consumers using schema61 `Generics`,
`WherePredicate`, `AssocItemConstraint`, `GenericBound::Use` and `PreciseCapturingArg`. Preserve
enclosing declarations needed to classify `use<'a,T,N>`. Expose typed attributes and trait
properties such as dyn compatibility without forcing users to inspect Debug strings or reconstruct
constraints from compact signatures. Keep addressed original declarations as expansion evidence.

Distinguish `Iterator<Item = T>` from `Iterator<Item: Clone>`, higher-ranked from free lifetimes,
ordinary/const/provided-default stability, and authored representation from computed layout.
Missing third-party stability stays unknown. Unknown attribute variants/text remain unmodeled;
JSON format compatibility does not make arbitrary text a stable attribute grammar. Stripped,
hidden and private members retain different coverage. A facade route and defining declaration
can differ in provenance and stability; external IDs are artifact-local.

Refine API candidates through existing `SourceBridge` and A02's native explanations. Compose
HIR method/trait candidate iteration, associated items, callable substitutions, receiver adjustments,
directional coercion and supported predicate evaluation. Keep enumeration, importability, solver
outcome and resolved actual call distinct. `NotProven` is not false. A successful bridge is not
proof of arbitrary blanket applicability or compiler acceptance. `cfg(doc)` artifacts cannot
establish ordinary-build availability, and inactive cfg items cannot be recovered from their JSON.

Use ordinary completion for ordinary completion questions. API-fit inquiry adds reasons, imports,
adjustments and unresolved constraints. A03 owns bounded synthesis; it is not a prerequisite here.
Migrate actual query/expansion/contextual consumers and remove superseded Debug parsing or
signature-based reconstruction; retain display-only text and artifact-only queries where useful.

Acceptance: equality versus bound and HRTB versus free lifetime differ; precise captures resolve
parameter categories; reexports retain both origins; missing metadata is unknown; dereference,
missing extension import and unmet bound yield distinct explanations. Source/environment/dependency
changes or ambiguous profiles refuse correspondence rather than repairing it through spelling.

Mechanisms: [rustdoc](../libraries/rust_analyzers/rustdoc.md) addendum §§2–5,
[rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) generic/attribute/stability sections,
and [HIR](../libraries/rust_analyzers/ra_ap_hir.md) type/candidate APIs. Exact
[schema61 source](https://github.com/rust-lang/rustdoc-types/blob/v0.61.0/src/lib.rs) governs fields.

## C02 — Native example preprocessing

Preserve inexpensive authored-fence reading. Add explicit pinned rustdoc preprocessing through
existing API/compiler acquisition and managed custody, retaining original inputs and outputs.
Doctest format2 is distinct from API JSON61; validate versions separately. Preserve original code,
crate-level attributes, transformed code and wrapper details, result-return behavior, origin and
execution attributes. Missing transformation after fatal syntax is unavailable, not empty success.

Extraction, compilation and execution remain separate observations. `no_run`, `compile_fail`,
`should_panic`, `test_harness`, `standalone_crate`, edition and conditional ignore settings express
intent/native processing, not completed checks. Passing `compile_fail` alone does not prove the
intended diagnostic. `cfg(doctest)`, `cfg(doc)` and ordinary compilation are different worlds;
Markdown collection does not grant access to private items. Native test owners execute separately
selected examples; do not reproduce rustdoc wrapper logic or build another test controller.

Migrate example inspection/expansion so current fence `expanded` text is never labeled native
transformation. Keep authored fences and their meaningful tests. Scraped example usage and
standalone executable Markdown remain conditional acquisition modes: select only for a concrete
example-navigation question beyond existing local calls, retaining analyzed-target coverage.

Acceptance: Result wrapper, hidden setup/escaped hashes, test_harness, standalone_crate,
target-conditional ignore, doctest Markdown carrier, unknown version and absent transformation.
Demonstrate that reading/extraction executes no example binary. Native invocation/fields must
be checked against actual c1070d693 sources before decoder implementation; newer book examples
are leads rather than the local protocol authority. See both rustdoc overviews and
[pinned extraction](https://github.com/rust-lang/rust/blob/c1070d693/src/librustdoc/doctest/extracted.rs).

## C03 — Package, compiler-unit and semantic-world explanations

Compose existing dependency details, original cargo_metadata, `CheckIntent` admission, correctness
`CompilerUnit` records and native package profiles. Return authored requests/aliases/defaults,
resolved package identity, optional/target edges, actual unit features/target/host-build role/mode
and native cfg/edition context. Explain the supported join and unavailable or nonunique dimensions.
Package inclusion is not unit inclusion; a planned unit is not executed compilation; a loaded
analyzer Cargo model is not automatically the exact admitted rustc unit.

Compare explicitly selected contexts using existing admission, without a second feature resolver.
Reuse acquisition across related questions. Original build-script/artifact/diagnostic events can
supplement evidence; metadata/planned units cannot invent execution observations. Typed parsing
is not lossless: unknown/malformed messages can become TextLine and compatibility defaults hide
absence. Retain original bytes where these distinctions matter. Cached BuildScriptExecuted is
not necessarily fresh execution; Artifact.fresh does not establish rebuild causality.

Assess Guppy PackageGraph/set queries only where graph consumers justify replacing bespoke
traversal. CargoSet/options/summaries are modeled counterfactual candidates, not authoritative
unit selection. Compatibility with experimental workspace feature unification is unestablished.
Hakari is not selected: native unification already exists. Determinator-style impact can nominate
scope but never certify exhaustive semantic impact. Resolve each candidate against an actual
consumer or record why no adoption is needed and what would reopen it.

Migrate dependency/unit/world explanations and remove duplicate joins/feature interpretation once
replaced. Keep distinct underlying package/unit views. No build telemetry, cache manager or
execution controller is added. Acceptance: renamed dependency, host/target/test units of one
package, changed default/feature policy, metadata-only package outside selected units, matching and
ambiguous native profiles, and an unknown event retained verbatim.

Sources: [cargo_metadata](../libraries/rust_analyzers/cargo_metadata.md),
[dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md), existing dependency/
correctness/source-bridge owners, [metadata .23.1](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/)
and [Cargo unit graphs](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph).

## C04 — Runnable discovery and relevance

Nominate validation for an inspected item or proposal with explicit reason: native relationship,
enclosing test module, package target or documentation example. Use `Analysis::runnables`, related
tests and actual package target metadata rather than inferring all kinds from `#[test]`. Distinguish
tests/modules/benches/doctests/binaries, cfg eligibility, required features and source provenance.
Snapshot-update metadata (`expect_test`, insta, snapbox) is intent, not update authorization.

Extend the existing candidate/original-runnable projection and guarded selection route. C02 can
contribute doctest candidates in its own world. Historical/API evidence cannot nominate a current
working-source candidate without established freshness/correspondence. Route selected candidates
to existing test-selection, discovery and execution owners; never execute analyzer argv as authority.
Relatedness proves neither coverage nor exhaustive impact; inactive tests are not executable in
the ordinary selected world. Keep conservative containing-package selection when narrower semantic
relevance is unavailable.

Migrate candidate output and selected-result routing together; delete independent heuristic kinds
or command reconstruction that the new consumers replace. Acceptance: related test/module, bench
and binary correctly classified, inactive-cfg test, doctest-only example, passive snapshot-update
metadata and stale historical witness rejected for current selection. See
[IDE](../libraries/rust_analyzers/ra_ap_ide.md) addendum §5, rustdoc and cargo_metadata targets.

## Verification and handoff

The conditional research record below predates production completion. Its open/next-step
statements are historical; the [Outcome](#outcome) owns the delivered boundary.

### Conditional acquisition and graph decisions — 2026-11-01

**Interface-checked:** scraped API-use examples and standalone Markdown have distinct value and
remain separate modes. Pinned rustdoc c1070d693 `scrape_examples.rs` pairs
`--scrape-examples-output-path` with `--scrape-examples-target-crate`; optional `--scrape-tests`
widening typechecks selected targets. It records direct/method call sites keyed by DefPathHash,
skips macro-expanded calls and writes opaque compiler-private encoded data for `--with-examples`.
Select that native acquisition when an investigation needs example usages from explicitly
acquired reverse dependencies outside the loaded local world. No such target population was
selected in this conditional wave: no scraper adapter or stable JSON usage index is justified.
Its output cannot establish exhaustive calls or interchangeable compiler identities.

Pinned rustdoc `lib.rs::main_args` routes Markdown with `--output-format=doctest` through
`doctest::test_markdown`; plain Markdown otherwise renders HTML. Select native transformation
when a concrete README/tutorial fence requires it, retaining format2 and original-source context.
No particular standalone document was selected here, so additional Markdown acquisition is not
implemented. This does not defer C02's selected Rust-source/doctest-carrier preprocessing. Neither
conditional mode compiled or executed examples in this wave. Original fence reading remains useful.

**Interface-checked:** Guppy0.19.1 PackageGraph/query_forward/query_reverse/resolve_with_fn and
PackageSet algebra are credible for richer repeated graph queries. They do not replace a bespoke
shortest-path solver here: `dependency.rs::inclusion_paths` already uses petgraph `astar`.
`affected.rs::WorkspaceGraph` also deliberately includes authored optional/path edges and
uncertainty widening; a resolved-only graph would weaken that meaning. Retain the existing owners.
Revisit Guppy when a concrete package-set/conditional-edge consumer removes substantial additional
traversal and adaptation, not merely to express the current inclusion path differently.

CargoSet/CargoOptions model host/target features with resolver V1/V1Install/V2/V3. Exact0.19.1
defaults are resolverV3, no dev dependencies and PlatformSpec::Any. Its source exposes no matching
experimental workspace feature-unification setting. Do not substitute it for CheckIntent admission
and actual Cargo-selected planned CompilerUnit records. Counterfactual summaries remain candidates
for a deliberately modeled question with its assumptions explicit. Planned actions are not executed
compilation. Hakari remains unselected, as previously settled.

Determinator0.13.0 `new(old,new)`, `add_changed_paths`, `set_rules` and
`compute().affected_set` can nominate cross-revision package impact. Its none/default/all-feature
simulations cannot replace existing generator-aware conservative reach or certify exhaustive impact.
Revisit for an explicit dependency/feature-change comparison needing that nomination. No new graph
or impact dependency is selected for C03's current unit explanations. Exact downloaded release
sources (`graph/query.rs`, `resolve.rs`, `cargo/cargo_api.rs`, Determinator's `determinator.rs`)
settled these choices without a compilation probe; unsuccessful Context7 resolution was not used
as absence evidence. C01–C04 production work remains open.

**Proposed:** compile touched packages with `scripts/pse check <package> --all-targets` and exercise
new behavior with `scripts/pse test <package> --lib -E '<filterset>'`, using the existing selected native
feature/preparation route where needed. T02 integrates each consumed generated declaration with
its producer and real consumer; never edit generated outputs. Original artifacts may coexist with
typed facts for distinct inspection purposes; that is not competing writable authority.

Independent authored/compiler fixtures must expose projection/association mistakes rather than
only snapshot their own output. Q01 owns assembled API-to-native, original-to-transformed example,
package-to-unit-to-world and candidate-to-existing-selection journeys plus scope-end checks.
No implementation, extraction, semantic service, product test or benchmark was run to author
this document. All packages are open; benefits remain unmeasured. Actual closure evidence and
enduring contracts follow their owning coordinator and normal decision routes.

## Outcome

**Implemented:** schema61 contracts retain generics, HRTB/associated bounds, captures, attributes,
stability and dyn compatibility without flattening them to display strings. Rustdoc format2
preprocessing retains original examples separately from native transformed units, including
hidden/wrapper context and explicit not-executed meaning. Actual Cargo plans retain renames,
default-feature policy, host/target distinctions and metadata-only members; no modeled graph
substitutes for actual selected compiler units. Library overview discovery now follows exact
basenames recursively, including moved documentation and ambiguous duplicates without following
symlinks or guessing matches from text.

Native runnable discovery preserves Test/TestMod/Bench/DocTest/Bin, relatedness, cfg eligibility,
actual target feature metadata and passive snapshot-update intent. Unavailable eligibility stays
unknown. Analyzer argv is never execution authority: direct and consolidated consumers route
only guarded current-checkout source through existing test selection. External-project and stale
sources do not receive that action. The graph/acquisition alternatives above remain conditional
with their recorded revisit triggers; they are assessed, not implemented.

**Tested, local Linux, zero-failure target:** API controls passed 21/21
(`01a1961f-f54c-70d8-bbdc-2ef3a723203f`); actual Cargo/compiler controls passed 3/3
(`01a1969d-27c8-73c4-8248-44a282babbe5`); final schema/dependency/source-command controls passed
32/32 (`01a196d2-4aff-7189-8d92-5ce40a3b80eb`). Final native-hir controls passed 14/14,
including contextual candidates, runnables and route guards. An actual prepared current-checkout
session routed native runnables through both direct and consolidated public inspection to
successful guarded test-selection reads and then stopped completely; the coordinator
[Outcome](62-code-intelligence-capability-expansion.md#outcome) owns original invocations and commands.

A mistake corrected was gating external-project test routing at only one entry point; the common
route now enforces the same project/source guard for both consumers. The outdated root-only
overview assumption was removed rather than encoding the user's current directory arrangement.
Deliberate limits retain unknown target ambiguity, candidate relevance distinct from execution/
coverage, no claimed native-world/rustc equivalence, and no extra graph or scraper subsystem.
Enduring contracts are in blueprint §21.7 and the CLI skill reference. No authorized delivery
remains here; unselected conditional features are not a residual implementation backlog.
