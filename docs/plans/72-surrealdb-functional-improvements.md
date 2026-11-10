---
title: "Plan72: SurrealDB functional improvements"
status: draft
date: 2026-11-09
adrs: [ADR-0261]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, design, deployment, storage, libraries, validation, diagnostics]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md]
scenario_sources: [docs/plans/72-surrealdb-functional-improvements.md]
---

# SurrealDB functional improvements

## Target and authority

**Proposed.** Authors define native contracts once, inspect their installed interpretation,
and evolve them through owned transitions. Canonical callers submit complete typed operations.
Engineers can search the full selected population, rank relevant evidence, explain bounded
dependency paths and follow current study state. Operators obtain coherent diagnostics,
protected result access and truthful maintenance/recovery through existing native owners.

Use native database planning, types, graph relationships, transaction procedures and SurrealKit
where they remove unnecessary interpretation, transfer and coordination. Scientific meaning,
independent assurance, original operation identity and effect custody remain explicit. Breaking
interfaces, representation, execution placement and physical implementation are acceptable.
Keep one selected production realization and remove what it replaces as consumers move.

This series derives from the complete [functional review](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md)
and its seven supporting assessments, published at `21f703887cc307f986975f9a2fba998f8bd2afa7`.
It supersedes Plan68 **without importing any scope, target, open package or acceptance obligation**.
Plan68 is inactive legacy. Historical citations inside the source review are provenance,
not an additional source of work for this series.

[Plan66](66-integrated-simulation-target.md) retains its scientific implementation and assembled
acceptance. [Plan71](71-workflow-and-simulation-test-feedback.md) is complete for its adopted
scope and supplies test infrastructure, original-attempt evidence and diagnostics.
Neither series is superseded or amended here.
Plan72 owns only the SurrealDB improvements and receiving obligations described below.

## Baseline and focused foundation assessment

**Interface-checked.** Authoring uses the review publication and current working tree, including
concurrent Plan71 changes. Source owners are the registry/Surreal lowering and canonical codecs;
`GuardedScript` and native transaction functions; study policy/module; metadata/graph/result
readers; native service and source-build preparation; SurrealKit fixtures and telemetry.
The relevant contract and source evidence is linked in each companion. Source inspection and
the review establish the rationale; they do not establish delivered or measured improvements.

The installed family is already v4: types/parser/expression 4.0.0, protocol 0.14.1, MX 1.1.0,
Surrealism 0.7.0 and source-pinned SurrealKit rc.1. Exact manifests, host source declarations,
features and patch identities remain their authorities. Reconfirm affected inputs when executing
a package; do not restart release cutover or inherit a v3 compatibility campaign.

The existing foundations support a direct evolution rather than a replacement framework:

- Registry lowering already emits native declarations/codecs. Improve its contract vocabulary,
  operation inputs and installed interpretation; do not introduce another editable model.
- Native operations already carry authority and original receipts. Explicit isolation and typed
  procedures can replace incidental framing; generation/lease meaning cannot be removed as if it
  were merely a database conflict key.
- Private knowledge and canonical graph queries already have selected cuts and bounded pages.
  Expose complete selection intent to the executor, and choose physical relationships with their
  access paths rather than assuming an adjacency cache accelerates every indexed relation scan.
- Keep compact canonical metadata in persistent MX and retain the bounded direct client as the
  selected starting realization. Full-range unsigned transport, subscription custody, migration
  effects and bucket capabilities are meaningful correctness-risk decisions, not reasons to defer
  the rest of the target.
- Existing native fixtures, structured reports and Collector supply reusable integration points.
  Consume Plan71 contracts and independent scientific readers; add neither a parallel runner nor
  a second diagnostic outcome authority.

Apply core 3.4, efficient architecture 1.0, process-simulator 1.5, test assessment 1.2 and the
[repository binding](../design_review/design_principles/standard.toml). This focused assessment
does not initiate a broad audit or certify an enclosing subsystem.

## Decisions and confirmed rule directions

The maintainer confirmed RC01 and RC02 explicitly on 2026-11-09 and selected richer current
workflows and breaking designs earlier in this engagement. Approval is of planning direction;
required decision records precede implementation of the affected enduring contract.

| Review impact | Planning decision and route |
| --- | --- |
| RC01 — canonical schema/value/interpretation/generation | Accepted direction, 2026-11-09. A01 names concrete changes to blueprint §4/§5/§20, identifies any superseding ADR/design revision, and schedules it before dependent representation changes. No hash or scientific meaning changes are implicit. |
| RC02 — transaction/policy/guards/defaults | Accepted direction, 2026-11-09. C01 identifies D10/D14, authority/commit and patched-default changes and their ADR/design route before dependent implementation. Keep semantic fences and original acknowledgements. |
| RC03 — richer discovery/observation/operating interfaces | Selected scope: implement richer existing workflows without requiring a pre-existing caller. D/E/G own explicit public semantics and changed feature/authorization contracts; use the normal route for enduring changes. |
| RC04 — persistence/transport/artifact alternatives | MX and direct bounded gRPC remain selected. F owns finite decisions for alternatives with actual isolation/history/capability consequences; a changed architectural selection uses its decision route before integration. |
| RC05 — tooling/fixtures/diagnostics | Integrate through existing owners and Plan71. Ordinary tooling/refactoring needs no invented ADR or approval gate; contract/governance changes still use their actual route. |

Pivot to the strongest design where source and qualitative reasoning provide a clear basis.
No benchmark, pilot or repeated benefit demonstration is required for clearly removed work.
Use a probe/comparison only for an unresolved premise posing serious correctness or performance
degradation risk. Targeted implementation tests still show new behavior. A quantitative claim,
if made, needs measured conditions; making no such claim is not an incomplete package.

## Series and contract ownership

| Companion | Owned output |
| --- | --- |
| [72a — Native contracts and schema interpretation](72a-native-contracts-and-schema-interpretation.md) | Shared declaration/native mapping, exact representation decision and compiler/live interpretation. |
| [72b — Schema evolution and deployment](72b-schema-evolution-and-deployment.md) | Typed effect-bearing transition, reconstruction/native/backfill selection, activation and recovery. |
| [72c — Canonical operations and policy](72c-canonical-operations-and-policy.md) | Named operations/isolation, typed readiness, precise conflicts/retries and shared authority. |
| [72d — Query, graph and ranked discovery](72d-query-graph-and-ranked-discovery.md) | Selected-population search, graph/path explanation, ranked lexical and justified retrieval extensions. |
| [72e — Observation and derived state](72e-observation-and-derived-state.md) | Owned following, summaries, precise invalidation and optional durable incremental projections. |
| [72f — Persistence and protected delivery](72f-persistence-and-protected-delivery.md) | Persistence/history and maintenance, artifact custody and complete transport/client delivery. |
| [72g — Developer and operator workflows](72g-developer-and-operator-workflows.md) | Database-contract composition and structured operating/client access using Plan71 foundations. |
| [72h — Diagnostics and execution fit](72h-diagnostics-and-execution-fit.md) | Selected-operation explanation, native execution/scratch and risk-directed mixed-workload evidence. |
| [72i — Integrated acceptance and closeout](72i-integrated-acceptance-and-closeout.md) | Series-wide receiving, applicable scope-end qualification, retirement and enduring documentation. |

A owns declared field/presence/value policy; F owns actual transport assembly, admission and
connection lifetime. C owns authoritative transaction/capture protection; E consumes that choice
for refresh and summary behavior. D owns query meaning; H observes the same builders and never
silently changes inspection into execution. B owns transitions over adopted A/C/F contracts.
G supplies database-specific contract composition while I selects assembled receiving.

## Dependency route and independent work

Start A01/A02, C01 and D's complete-query design from available code. G can consolidate existing
contract inputs while A's new representations are developed. F can settle precise value/wire and
bucket questions independently; no engine replacement is prerequisite for ordinary improvements.

A's chosen mapping and F's actual wire behavior must agree before native numeric cutover. B can
build typed transition intent before a particular representation migrates, then receive that
actual change. C's pure typed readiness can advance with A's carrier contract; precise witness
deletion depends on its real read/authority footprint, not on all of E. D's lexical and graph
improvements do not wait for subscriptions or buckets. E's snapshot-follow capability depends
on a qualified subscription lifetime; one-shot snapshots and query-first summaries remain useful
independently. H can join existing diagnostics while new query/stream producers arrive.

Each companion names its smaller prerequisite slices. An agreed schema or interface is not a
working implementation. Shared files need coordinated ownership even where packages are logically
independent. Use the existing checkout and preserve concurrent Plan66/71 edits.

## Cross-plan boundaries

| Existing owner | Plan72 integration rule |
| --- | --- |
| Plan66 contracts/results/runtime | Consume actual scientific source, result and protection semantics. Schedule necessary database-side consumer migration in Plan72 with its semantic owner; do not adopt or close Plan66's wider physics, numerical or assembled acceptance. |
| Plan71 CLI/original evidence/fixture efficiency | Reuse delivered references, reports, fixture ownership and selected diagnostic contracts. Record only SurrealDB-specific extensions here; inspect live work before treating a gap as missing. |
| Current architecture and accepted ADRs | Retain required meaning until the adopted decision route changes it. Do not use inactive Plan68 text as implementation authority or a hidden qualification obligation. |

## Opportunity dispositions

This table owns source-opportunity disposition; companion package tables own progress.
Scheduled means designed for implementation on adoption of this draft, not delivered.
Investigations have finite decisions and dependent branches; a justified retain/no-change
outcome can close that investigation without leaving an unbounded backlog.

| Opportunity | Disposition | Work owner | Evidence or revisit trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O01) | Scheduled | 72a A01–A03; 72f transport | Shared named vocabulary and policy-aware mappings; resolve full-range/native-kind risk before representation cutover. |
| [O02](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O02) | Resolved | [72a A02/A04 Outcome](72a-native-contracts-and-schema-interpretation.md#outcome) | Typed operation inputs, strict full-schema compiler/live represented agreement and drift/coverage controls delivered. Existing generator retained; no disjoint external emitter selected. |
| [O03](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O03) | Scheduled | 72b B01–B03 | Exact typed transition with data effects and recovery; select reconstruction/native migration/backfill per actual change. |
| [O04](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O04) | Scheduled | 72c C01/C03 | Explicit complete transactions and one retry owner; independent concurrency/refusal cases protect guard removal. |
| [O05](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O05) | Scheduled | 72c C02; 72a carriers | Typed pure shared readiness with visible acquisition; transaction access/init/bindings selected only for concrete benefit and supported contract. |
| [O06](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O06) | Scheduled | 72c C04 | Shared operation meaning and explicit invocation/execution authority; replace an ingress rather than add duplicate policy. |
| [O07](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O07) | Scheduled | 72d | Filter complete selected population before paging; choose useful access/index/count shape directly. |
| [O08](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O08) | Scheduled | 72d | Bounded evidence paths and relation-specific representation; finite materialization/GQL decisions by semantics and exposure. |
| [O09](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O09) | Scheduled | 72d | Ranked lexical mode; assess semantic extension against judged questions, not a requirement for an existing caller. Spatial has no selected geographical meaning. |
| [O10](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O10) | Scheduled | 72e; 72f subscription boundary | Snapshot/notification follow with explicit race/reconnect/custody; durable transitions are a separate contract. |
| [O11](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O11) | Scheduled | 72e with 72c conflict owner | Query-first summaries, precise invalidation; views/feed maintenance only with complete refresh/reconstruction semantics. |
| [O12](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O12) | Scheduled | 72f | Deliver useful maintenance/history under MX; record finite KV/standalone/topology decision and explicit triggers. |
| [O13](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O13) | Scheduled | 72f | Resolve complete bucket publication/range/no-clobber/transfer-capability custody; adopt replacement if supported, otherwise retain files with reason. |
| [O14](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O14) | Scheduled | 72a declared mapping; 72f wire/client | Exact generated mappings and demand-shaped delivery; retain direct client unless a complete public replacement removes its real gaps. |
| [O15](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O15) | Scheduled | 72g with Plan71 fixture/evidence owners | Shared SurrealKit contracts and independent consumers under native outer custody; no competing test infrastructure. |
| [O16](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O16) | Scheduled | 72g | Structured useful operating interactions; explicit optional MCP/standard-client decision without an automatic frontend/server estate. |
| [O17](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O17) | Scheduled | 72h with Plan71 diagnostics | Join original request, plan, response and incarnation; reuse existing telemetry and report owners. |
| [O18](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O18) | Scheduled | 72h; 72i receiving | Credible native execution with owned scratch; compare/tune only for serious remaining degradation risk or a quantitative claim. |

The source review's [complete discovery correspondence](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#complete-discovery-correspondence)
is included in scope at the level of its individual directions, not just these eighteen titles.
Companion correspondence tables map its detailed branches to packages and decision outcomes.
Earlier features, inline/reference/cache choices, sequences, native views/events, feeds, codec
alternatives, emitters, config/seeds and diagnostics remain eligible regardless of release age.
Do not silently turn all mechanism mentions into required adoption.

The review's deliberately unselected tails remain recorded decisions: geographical retrieval,
Agent Memory, a new Kafka/Node/web stack and unrelated hosted/browser deployment have no selected
functional proposition. D/F/G own observable within-scope triggers that would reopen them. Removed
SurrealML is unavailable. These dispositions capture the findings without manufacturing work.

## Verification and completion

**Proposed verification.** Packages name targeted behavior/refusal tests and their replacement
deletions; use scoped `pse check`/`pse test` and explicit force-validation where applicable.
Run codegen when declarations/generators change. I selects relevant integrated journeys and
scope-end hygiene/governance/docs/native checks once functional scope is implemented, following
repository policy. Do not repeat static campaigns per companion or require generic benefits proof.

Completion requires delivered selected behavior, closed finite investigations with their branch
outcomes integrated, moved consumers, deleted replacements, actual required receiving and settled
effects. Local implementation, assembled qualification and benefit measurements stay separate.
Plan66 scientific acceptance and Plan71 completion remain with their own owners. Move enduring
meaning to its architecture/developer owners through the required routes before retiring this
series and resolved source reviews. No production change is performed by authoring this draft.
