---
title: "Plan60a: Evidence contracts and query composition"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Proposed ADR-0235 and the retained Plan60 Outcome depend on this contract and its original acceptance evidence."
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, diagnostics, libraries]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md, docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: [ADR-0235]
---

# Evidence contracts and query composition

## Purpose and foundation assessment

**Proposed:** consumed extensions to the delivered evidence contract and bounded question composition, contributing to
[O01/O02/O07/O08](60-engineering-code-intelligence.md#opportunity-dispositions). The
[coordinator](60-engineering-code-intelligence.md) owns dispositions, cross-package sequencing
and conditional options. This document owns A01–A03 implementation state.

`crates/pse-host/src/investigation/{model,composition,rust}.rs` already own typed requests,
related traversal, scopes, demand, identity and refinement. `SearchResult`, `EntityIdentity`,
`Truth` and `RefinementFact` are the existing seam. Native `TypedFacts` distinguish original
and adjusted observations. CLI inspection/render/schema adapters consume those owners. Build
on them; extra result kinds must not invent another entity universe or force every fact into
an unqualified JSON bag. The source review's O01/O02/O07/O08 explains the functional value and
limits; blueprint §21.7 remains the contract owner.

Plan61 already supplies authored proto, Buffa/ConnectRPC generation, validators, typed facts,
direct VTable projection and request-local client/proof reuse. The starting codebase is
`42bede27a`, including Plan62's completed native/source observations and selected assembled
acceptance recorded in its [Outcome](62-code-intelligence-capability-expansion.md#outcome).
Reuse protocol4 typed predicates, original use-site guards and existing compact views rather
than schedule their producer/cutover work again. Current destination rows and rendered relationship context are insufficient for
role-qualified edges. Improvement review I01/I02 and functional review O01 explain this residual.
The coordinator owns dispositions; blueprint §21.7 remains authoritative.

## Target design

A relationship identifies its kind, direction/endpoints, witness, selected world or artifact,
provider and evidence limits. Written syntax, resolved target, reference use, explicit call and
implicit language operation remain different meanings. Type facts distinguish written,
inferred, adjusted and substituted forms. Original owner identities remain intact beneath a
shared projection. An unknown target or unsupported predicate is representable without a
fabricated false result. The representation need not create one new type per conceptual fact.

Multi-seed questions compose existing validated predicates, scopes and references. They can
show shared consumers, boundary crossings and bounded paths with cycles/frontiers. Separate
the traversal domain from output selection: an intermediate outside an output package may
be essential. Share nodes/edges and witnesses within a request instead of repeating whole
paths or reloading an analysis per seed. Reuse current native owner admission/cancellation
and actually join owned work. Native handles stay in their owner; return suitable owned facts.

Presentation is separate from evidence demand. A focused summary can group operations or align
alternatives while preserving identity, coverage, unknowns and guarded expansion. A small
render allowance cannot imply complete examination, omit a final record, or secretly turn
first-N into a global scan. Existing fields/grouping/NDJSON and reference freshness are the
foundation. Materialize only the compact views the selected question needs; projection can
reduce upstream work only where provider semantics permit it.

Use Serde typed views/borrowed parsing and, where necessary, `RawValue` for opaque original
payloads. Keep Schemars serialization/deserialization-direction schemas and typed constructors
aligned with the one validator. Clap parser metadata/value sources and Figment provenance
explain effective query/defaults/overrides before provider effects. Preserve existing
ErrorRecord/TypedDiagnostic cause/label structure; Thiserror/Miette do not define another wire
taxonomy. Indicatif remains observed stderr activity, never proof of completion. Mechanism
references are the review O07/O08/O11 and the local [Serde](../libraries/serde_serde-json.md),
[Schemars](../libraries/schemars.md), [Clap](../libraries/cli/clap.md) and
[Figment](../libraries/cli/figment.md) overviews; verify exact pins for new API use.

## Work packages

<a id="a01"></a>
### A01 — Extend the generated contract with the first consumed edges

Consumes delivered Plan61 generation/validation and current query/result/native facts. First settle relation semantics, contextual
identity, witness/unknown/omission representation and expansion with B01/C01/D02 consumers.
Assess the concrete change against the existing ADR/schema/publication route before coding;
do not infer that a new relation name requires a new service or crate. Publish contract meaning
through the normal owner when required. Then extend the existing constructors/validator and
shared schema/output adapter for the first consumed native and artifact facts. Consumer design
can use agreed semantics; integration requires working constructors and validation.

Acceptance uses public typed JSON and flag/saved-query routes as applicable: equivalent queries
have the same meaning; malformed combinations fail before provider effects. Distinct providers
cannot join same-spelled or stale entities by display name. Unknown/refused/cancelled and false
remain distinguishable. Input and output schemas exercise their actual directions. A01 is not
complete on a schema sketch alone. Implement the first B01/A02 consumer together: qualified
source/destination endpoints, structural role, original occurrence witness, provider/world or
artifact, and unknown/omission meaning. Reuse existing identities and Buffa constructors/
ReflectMessage::get/ValueRef; do not decode rendered context or parse type displays for edges.
Two parameter positions sharing one type remain two witnessed relations. B01/C01 extend this
slice as needed, without requiring all relation families before any consumer can integrate.
Delete replaced duplicate field/identity conversions with migrated consumers, without maintaining
a compatibility API or removing supported original evidence.

<a id="a02"></a>
### A02 — Deliver bounded multi-seed responsibility investigations

Consumes implemented A01 and B01's declaration/containment relation slice, not the whole native
document. Extend existing Related/All/Any/Not and request-local native reuse. Current traversal
uses one seed and prunes destinations outside request.sources before enqueueing; independent
lists do not implement the selected multi-seed question. Define traversal selection separately
from returned-source selection, preserving compatible source/world qualification and validated
flag/JSON/saved-query meaning. Preserve which seeds and edges establish shared-neighbor answers.
Share demand-selected adjacency within the request instead of re-extracting per seed or
materializing every path. Keep node identity separate from edge/witness deduplication.
Deliver “which operations consume these request/policy declarations and cross their boundaries?”
without a process DSL, persistent graph or mandatory index.

Return reusable edges/witnesses, shared neighbors and an explicit bounded frontier. Test a
cycle, shared neighbor, overlapping seeds, mixed/stale contexts, a limit reached before full
traversal and a necessary intermediate outside output scope. A lexical narrowing must not
silently restrict a global reference question. Cancellation must settle the actual provider
work and leave sibling requests' state independent. Compare against native facts in a small
authored fixture; merely comparing two renderings of the same traversal is insufficient.
Remove scalar request assembly duplicated by this public consumer once callers migrate.

<a id="a03"></a>
### A03 — Focused answers, progressive expansion and query assistance

Consumes A01; compose each new producer as it lands. Use existing `search --explain`, schemas,
fields/grouping and `inspect` references before adding a view. Deliver a compact relationship
or alternative comparison with a useful next expansion preserving original identities and
coverage. Selection demand, source witness context and rendered allowance remain independent.
Test actual human/JSON/NDJSON consumers, complete stream finality, stale expansion refusal,
unknown facts and truncated demand without fabricated omission totals. No numeric token SLA
or speed claim is part of acceptance.

For structural authoring, expose pinned grammar `NODE_TYPES` and native query-error locations
where useful, with explicit ast-grep `stopBy`/strictness examples. Do not maintain a second
Rust node taxonomy. The source review O11 identifies public ast-grep0.51 insertion helpers
with `todo!()`; do not qualify them by documentation alone. This package adds inquiry assistance,
not an editing subsystem or wholesale pretty-printing of maintained source.

Question-specific native query-error locations stay here. The demonstrated invalid-field and
reference-handoff corrections, parser discovery and validator details belong to F01–F03 in
[60f](60f-agent-interaction-and-contract-diagnostics.md), using the same envelope and addresses.
A03 consumes those working controls for its public journeys; it does not duplicate them.
Only a demonstrated additional missing local correction warrants further enrichment.
Only demonstrated discovery burden warrants a passive shared-query catalog, coordinated with
E01. Those triggers remain optional and do not block the base focused views. Migrate consumers
and remove replaced oversized adapters/duplicated extraction where superseded; do not remove
full original inspection used for other questions.

## Execution boundary (2026-11-01)

The authorized implementation combines A01–A03 with B01's minimal containment,
parameter/return/field nominal type-dependency and resolved-use producers. Remaining B01
families and B02–B05 retain their existing owner. The target supports the audit, design,
functional-extension, library-leverage and planning skills; adoption frequency is not an
acceptance gate.

Related requests replace scalar `seed`/`relation` with nonempty `seeds`/`relations`, an
independent working-tree `traversal` selector, `seed_match` (`any` by default, or `all`),
and positive node/edge/depth bounds. Ordinary `sources` select answer nodes; hidden
intermediates remain available as guarded support. All seeds share the original prepared
session, configuration and generation. One native operation and snapshot run a shared pure
traversal kernel, with request-local adjacency and minimum distance per seed. The declared
depth is an intentional boundary; exhausted provider/node/edge work makes within-depth
absence unknown. Canonical edge direction never changes when traversing incoming edges.

Nominal dependencies are sets within each outer parameter/field role: repeated nested type
positions are not claimed where the pinned HIR walk deduplicates them. Distinct outer roles
retain parallel witnesses; enum variant fields and declared versus async return forms remain
distinct. Incoming structural coverage comes from the selected prepared declaration population,
not only textual reference nominations. No persistent index or HIR handle crosses the boundary.

The first focused graph view is a settled bounded answer. Its NDJSON records are final
records emitted after native work settles, without mutable graph updates. Existing source
streams remain streaming. Grammar assistance reads pinned `NODE_TYPES`, and input/output
schemas use their respective deserialization/serialization directions. F01–F03 universal
controls and conditional capability triggers retain their separate ownership.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| A01 | complete | Shared investigation contract owner | Proto-authored native graph, checked projections, contextual identities, role/witness evidence and public schema consumers |
| A02 | complete | Query composition owner | One-worker multi-seed traversal with independent output selection, attribution/frontier, count/unknown semantics and scalar cutover |
| A03 | complete | Query/inspection presentation owner | Settled expandable graph views, native grammar data and structured query-error locations in public formats |

## Verification and handoff

**Tested (2026-11-01, local Linux):** targeted host/native unit tests and real public CLI
journeys exercise the implemented scope; exact commands and boundaries are in the Outcome.
E03 retains engineering-process integration and Q01 retains series-wide static/integrated
qualification. Source-only grammar and syntax questions require no analyzer preparation.

## Outcome

**Implemented:** A01–A03 and the minimal B01 producer slice. The authored proto and generated
Buffa contracts carry one normalized graph operation; protocol 5 fences earlier helpers while
administrative stop remains available. Native workers retain their immutable snapshot,
cancellation hook, actual CPU settlement and original session/configuration/generation.
Checked role alternatives, source guards and response indexes refuse malformed/unknown facts.
Declaration identity is distinct from a use occurrence. Definition edges preserve original
use-to-declaration meaning; incoming references terminate at occurrences rather than inventing
occurrence-to-occurrence targets.

Containment preserves module/impl/trait/variant/field ownership. Nominal parameter, return and
enum/struct/union field dependencies preserve parallel outer-role witnesses; async output is
separate from declared return. Explicit source paths and receiver methods supply resolved uses.
Unresolved uses, unsupported field-use classification, block-local population and unmapped
sources remain incomplete. Implicit language operations are a separate meaning and are not
claimed. The actual traversal domain intersects source selection with the active prepared
world; excluded inactive files are identified as outside that world, and inactive/plural seed
contexts still refuse. This does not establish absence across other configurations.

The pure traversal kernel caches adjacency and propagates per-seed minimum distances, including
later attribution, shared intermediates and positive cycles. Node/edge/provider limits retain
unknown absence; positive Exists can stop after its seed rule is met. Count units concern
selected answer nodes, independently of edge examination and row projection. Separate graph
scopes survive Boolean composition. Scalar Related assembly and frontier-per-RPC traversal
were removed with migrated callers. Existing full native inspection remains available.

**Tested:** the following commands passed against the zero failure baseline. Tests use the
native correctness route, including its actual force-validation closure selection; these are
targeted component/public-consumer checks, not whole-Plan60 qualification.

```bash
just codegen
scripts/pse check pse-host --all-targets --features native-hir
scripts/pse test pse-host --lib --features native-hir -E 'test(bounded_graph_) | test(normalized_graph_wire_) | test(investigation::graph::tests::) | test(investigation::model::tests::) | test(investigation::config::tests::) | test(source_query::tests::) | test(code_intelligence_contracts::tests::)'
scripts/pse test pse-host:source_queries -E 'test(passive_grammar_assistance) | test(actual_tree_sitter_query_failure)'
scripts/pse test pse-host:composed_investigations -E 'test(bounded_multi_seed_graph_) | test(actual_native_namesakes_)'
scripts/pse test pse-host --lib --features native-hir -E 'test(bounded_graph_) | test(normalized_graph_wire_)'
```

Generation published all 10 configured targets. The unit selection passed 36 tests; the source
CLI selection passed 2 tests and the native CLI selection passed 2 tests.
The final affected native/wire rerun passed 4 tests after active-world admission was corrected.
Public native checks
independently expect the two policy types to reach `consume` through a hidden `Shared` type at
distance 2, with parameter positions 0/1 retained. They validate JSON and each NDJSON record
against advertised schemas, human actions, enum fields, async output, finite limits and stale
expansion refusal. Native unit checks include deterministic admitted-worker cancellation,
actual settlement and sibling independence. Grammar checks compare native `NODE_TYPES` and
query-error coordinates with the pinned producers, without preparing an analyzer.
`just ready` passed after the environment-shaped fixture refusal; final `just turn-end`
passed both ADR indexing and formatting. Neither establishes Q01 hygiene or integration.

### Mistakes corrected

The first public native run used detached-session fixtures inside an enclosing managed test
workload: both selected tests failed setup. The corrected harness establishes an actual
independent caller scope before opening its gate; production nested-detachment refusal remains
intact. Subsequent public execution exposed a default filesystem/native-domain mismatch and
truncated fixture declaration anchors. Active-world admission and original name-token seeds
corrected those failures. Earlier failed/interrupted checks receive no acceptance credit.
Independent implementation review also found domain leakage, lost positive-cycle selection,
unsupported use completeness and declaration/occurrence normalization gaps; focused regressions
exercise the corrections.

### Deliberate boundaries and handoff

**Implemented:** graph answers are final bounded records after native settlement. Ordinary
source streams still stream; no mutable graph continuation, persistent index, new service,
universal identity or dependency was added. Nominal dependencies are a set per outer role;
nested occurrence paths are not invented. Exact return/parameter/field source witnesses and
original semantic-node expansion remain available regardless of optional text projection.

**Proposed / not_run:** full B01 associated/visibility/supertrait work, B02–B05, artifact
extensions, C–F consumers, E03 process journeys and Q01 series-wide hygiene/integration remain
with their existing owners. B01 is partially delivered, not complete. No performance, adoption,
scientific or universal compiler-equivalence claim follows. Feature-use frequency is not an
acceptance gate. The user-owned library-document changes remain untouched. ADR-0235 remains
proposed; this acceptance does not extend the earlier formal review's scope.
