---
title: "Plan60a: Evidence contracts and query composition"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, diagnostics, libraries]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md, docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: []
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
direct VTable projection and request-local client/proof reuse. Plan62 supplies richer native/
source observations, with assembled acceptance still pending at the inspected checkpoint.
Reuse them. Current destination rows and rendered relationship context are insufficient for
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

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| A01 | planned | Shared investigation contract owner | Consumed typed semantics, constructors, validation and schema/consumer integration |
| A02 | planned | Query composition owner | Public multi-seed question with preserved scope/frontier and migrated scalar assembly |
| A03 | planned | Query/inspection presentation owner | Focused expandable answers and useful existing-owner query assistance |

## Verification and handoff

**Proposed:** use targeted host/CLI unit checks for changed contracts/composition and their
consumer journeys. Run code generation when declarations/generators change. E03 owns actual
process-use integration and Q01 owns scope-end static/integrated qualification. A source-only
question must still work without analyzer preparation. No product changes or new qualification
are claimed by this plan's publication.
