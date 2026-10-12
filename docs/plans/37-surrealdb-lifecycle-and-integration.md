---
title: SurrealDB lifecycle and integration target
status: draft
date: 2026-10-12
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#representative-changes-and-failure-journeys]
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, design, storage]
doc_retention: while-dependent
---

# Plan 37 — SurrealDB lifecycle and integration target

## Purpose and reading route

Deliver the target developed from the [2026-10-12 review](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md), including all seven findings, nine rule decisions and consequential investigations. This is a distinct undertaking. Existing plans remain unchanged until this work is implemented and accepted; only then reconcile affected contracts, prerequisites and status before returning to them.

The target supports value edit → re-solve without structural rebuilding; sweeps, sensitivities, optimization and fitting; recycles/tears; dynamics from the same definitions; unit/property/solver extension; and compilation/testing with a responsibility's actual dependencies. Persistence preserves scientific meaning and dependencies; numerical authority stays with scientific/library owners.

**Maintainer cutover decision, 2026-10-12:** pivot hard to the target. Existing data and evidence need not survive; they can be regenerated. Build fresh target stores, generated contracts and qualification evidence. Do not build legacy migration, backfill, historical-reader or compatibility paths. Future target data still receives the retention guarantees below. Reset is an explicit authenticated operation on a selected owned store, not authority to remove another owner's resources.

| Document | Responsibility |
|---|---|
| This coordinator | Combined target, rule decisions, finding dispositions, all package status, investigations, qualification and eventual existing-plan reconciliation |
| [37a — Scientific artifacts, history and policy](37a-scientific-artifacts-history-and-policy.md) | Retained inputs, useful content, history release, replay fences and production study policy; F01–F03/F07 |
| [37b — Connection and transport lifetimes](37b-connection-and-transport-lifetimes.md) | Caller release, physical shutdown, acknowledged restoration and exceptional completion; F04–F06 |
| [37c — Tooling and operational integration](37c-tooling-and-operational-integration.md) | Beta tooling, generated DDL, fixtures, forward rollouts, clean rebuild and capability investigations |

This coordinator is the sole disposition and package-status owner. Companion sections develop the designs without duplicating status. The source review keeps its historical observations; its local SCI/CON labels are not extra findings.

## Baseline and focused foundation assessment

**Implemented, source-inspected:** commit 677394474a543c884a8a21d1a2626a6820ff834e, with the uncommitted review/evidence and user-supplied second white paper preserved. Canonical revisions, preparation, runs, attempts, original assessment, closed outputs, readers and service generations already have distinct owners. Exact publication/creation/reclamation fences, protected analysis inputs, copied immutable bytes and independent operational metadata are useful foundations. Keep them; no universal lifecycle controller is needed.

The review's consequential gaps remain: late acquisition of selected external seeds, whole-run retention, permanent detailed history, incomplete shared-close ownership, nonterminal acknowledged restoration, exceptional admission release, and policy compilation coupling. None is established as a measured throughput or capacity deficit.

Focused authoring advice found existing seed admission already defines bit-preserving content, original inventory, assessment and layout/profile/data/backend compatibility; a small self-contained seed artifact can avoid generic shared-chunk machinery. The current pse-model study types are a fitting production home for pure study policy. Before new history deletion, retired study identity must dominate creation even when a different run is supplied; retaining today's marker alone is insufficient.

For beta.6, namespace-auth test mode keeps suite and parity databases in the configured namespace, permitting pre-creation enclosing ownership. Its schema preparer rewrites definitions, so generated-input acceptance does not prove equivalent behavior. Its SQL hashes do not replace the repository's DDL-plus-codec interpretation digest.

**Interface-checked:** release-specific library sources and documentation. **Proposed:** target design and future packages. No new product tests, runtime probes, installations, services, measurements or scientific qualification occurred during authoring. Read the [supporting evidence](../design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/README.md) for original scope and limits. Both white papers remain leads; the stale utilization catalog/capability maps are not evidence.

Architectural owners are blueprint §3, §4, §20 and §21. Criteria remain Core 3.4, its efficiency heuristics and Process Simulator 1.5. Existing rules describe the baseline; confirmed changes below govern the target.

## Combined target and preservation

Scientific owners specify sufficient immutable content for starts, states, trajectories and analyses. Canonical operations admit that content and its dependencies, expose bounded protected reads and release independent obligations. Producer retirement must not destroy a promoted artifact. Provenance does not automatically require retaining every producer occurrence or trajectory.

New reproducible durable studies bind exact external starts before activation. Preserve original bits, identities, assessment and compatibility; distinguish any deliberate acquisition-time request. Owned immutable bytes can outlive producer permission, while fresh DB reads require current artifact authority. History release removes unselected detail only after settlement and dominating identity fences. Unknown effects, live borrowers and deliberate audit obligations remain protective. Minimal replay attribution cannot promise reconstruction of deleted full responses.

Runtime close releases its logical caller. Admitted work owns the connection activities needed through renewal/publication. Whole-connection shutdown is an explicit deployment operation that refuses live dependents or atomically closes admission and physically drains when none remain. Restoration reaches ready or terminal failure under one acknowledged budget. Terminal admission release is independent of metadata garbage collection and distinct from payload drain or DB settlement.

Retain the patched WS production integration and SDK/server 3.3.0. Target SurrealKit 1.0.0-beta.6 through a standalone adapter consuming generated schema, namespace-owned fixtures and existing exclusive maintenance. Tool cleanup, parity and lock TTL are not disposal, scientific certification or effect authority. Bootstrap metadata remains available with the DB down. Native mutable scientific work stays process-local. One declaration owns every cross-boundary vocabulary/schema.

Preserve original numerical qualification, partiality, source/start identity and exact values; creator/reclaimer conflict; no resurrection; actual native/transport drain and charge ownership; independent scientific checks; and explicit runner outcomes. These govern future target behavior despite the authorized fresh-data cutover.

## Confirmed rule decisions

The maintainer explicitly accepted all nine per-item recommendations on **2026-10-12**. RC05–RC07 confirm their limits, not unconditional adoption of conditional alternatives. Confirmation is not implementation acceptance.

| Review item | Confirmed decision | Consequence and route |
|---|---|---|
| [RC01](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc01) | Accept retained exact inputs for reproducible studies; distinguish deliberate late binding | A1 changes admission/input ownership. D0 routes relationship/contract rationale and blueprint §20.3/§20.6 before code. |
| [RC02](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc02) | Accept independent scientifically sufficient artifacts | A0/A2 implement promotion/release beyond whole-run retention; new relation/operation ADR and blueprint §20.4/§20.6 as applicable. |
| [RC03](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc03) | Accept releasable detail, minimal fences and selected audit history | A3 changes history/replay semantics; D0 routes the contract decision and architecture amendment. |
| [RC04](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc04) | Accept caller release versus authoritative physical close | B1 changes the Rust/Python boundary; required ADR/design review and blueprint §20.5/§20.6/§21 precede code. |
| [RC05](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc05) | Retain patched WS while comparing alternatives | B2/B3 repair it; I3 cannot authorize replacement without a concrete confirmed mechanism decision. |
| [RC06](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc06) | Retain local-first binding | I7 records remote extension requirements only; preserve loopback safeguards. |
| [RC07](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc07) | Retain truthful failure/retry; assess checkpoint sufficiency | I7 distinguishes starts/history from restart; no durable numerical resume commitment. |
| [RC08](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc08) | Accept actual policy compilation/test isolation | A4 moves production policy to pse-model, migrates callers and updates blueprint §3/§20.6. No new crate selected. |
| [RC09](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc09) | Accept beta.6 under existing schema/fixture/operational owners | C0–C3 implement the seam; normal exact-pin rules apply. Changed activation/ownership contracts route through D0. No stable0.7 fallback. |

**Additional explicit instruction OP01:** the later hard-pivot instruction removes the earlier legacy-data-preservation premise. D0 records fresh-store cutover and removal of replaced historical activation/migration paths in the relevant decision and blueprint §20.5/§20.6. Do not add a fictional RC10 to the original review or ask again for this instruction.

D0 classifies ADR needs under the repository's decision rules: amend proposed records when suitable, supersede accepted immutable records if needed, and obtain the required review for changed boundary/crate decisions. Ordinary dependency/tool additions need no invented ADR. Fill front-matter ADR IDs when records exist; none are allocated here.

## Work packages

All status is prospective. Linked companions own the implementation visions and revealing tests.

| Packet | Required input | Delivery and owner | Acceptance and replacement | Status |
|---|---|---|---|---|
| D0 | Confirmed decisions/current owners | Coordinator: decision route, shared contracts, clean-cutover target | Required ADR/review/design changes before dependent implementation | Scheduled |
| A0 | Exact scientific and replay owners | [37a](37a-scientific-artifacts-history-and-policy.md#a0-scientific-closure-and-collection-decisions): settle artifact sufficiency/reduced replay | Valid/invalid selections and complete reachable mutation paths; gates A2/A3 | Scheduled |
| A1 | D0 input contract | [37a](37a-scientific-artifacts-history-and-policy.md#a1-retained-exact-study-inputs): exact external start admission | Both retirement orders, pending restart/retry and release; replace late-acquisition-only retained path | Scheduled |
| A2 | D0/A0, A1 input operation | [37a](37a-scientific-artifacts-history-and-policy.md#a2-independent-artifact-promotion-and-retirement): promotion/read/release | Producer retires, selected content survives, unrelated outputs drain; replace coarse-only consumer obligations | Scheduled |
| A3 | D0/A0, A2 closure | [37a](37a-scientific-artifacts-history-and-policy.md#a3-releasable-history-and-dominating-retirement): history release/collection | No retired identity resurrection; audit/unknown protection; delete unnecessary detail after settlement | Scheduled |
| A4 | Current pure study facts | [37a](37a-scientific-artifacts-history-and-policy.md#a4-production-policy-compile-and-test-boundary): relocate actual policy/tests | Same production policy compiles/tests without SDK/codegen/columnar; delete old implementation/imports | Scheduled |
| B1 | D0 close contract | [37b](37b-connection-and-transport-lifetimes.md#b1-logical-release-and-authoritative-shutdown): caller/activity ownership | Close/admission/renew/publication races and final drain; replace physical-close caller wiring | Scheduled |
| B2 | Current bounded WS contract | [37b](37b-connection-and-transport-lifetimes.md#b2-finite-acknowledged-restoration): finite ready/failed restoration | Missing ACK/send failure cannot park future callers or dispatch partial setup | Scheduled |
| B3 | Current admission/payload ownership | [37b](37b-connection-and-transport-lifetimes.md#b3-explicit-exceptional-admission-completion): explicit terminal release | Held metadata cannot retain non-payload slots; preserve payload physical drain | Scheduled |
| C0 | Beta.6/current generated output | [37c](37c-tooling-and-operational-integration.md#c0-beta-input-and-command-qualification): DDL/dispatch qualification | Semantic fidelity, exact selection/auth, executable provenance; gates tool consumers | Scheduled |
| C1 | C0 and namespace owner; C2 initial frozen chain for replay/parity | [37c](37c-tooling-and-operational-integration.md#c1-owned-declarative-database-tests): declarative tests/disposal | Register enclosing scope before creation; parity/setup/error/interrupt controls; replace duplicated assertions where fitting | Scheduled |
| C2 | D0/C0; B1 shutdown for persistent maintenance | [37c](37c-tooling-and-operational-integration.md#c2-frozen-forward-rollouts-and-exclusive-maintenance): forward rollouts | Frozen order/hash/repair plus behavior; owner loss fences effects; no old rollout import | Scheduled |
| C3 | D0/final target declarations | [37c](37c-tooling-and-operational-integration.md#c3-clean-rebuild-and-operational-cutover): reset/reseed/activation | Owned drain/reset/rebuild and fresh identities; delete replaced historical adapters | Scheduled |
| I1 | Concrete A0 relationships | [37c](37c-tooling-and-operational-integration.md#bounded-capability-investigations): native references | Exact structural meaning/races, equivalent simplification or explicit non-adoption | Scheduled |
| I2 | Notification/follow-up consumers | 37c: events/live/changefeeds | Useful replacement with retry/resync owner, or reason/trigger | Scheduled |
| I3 | Protected-read workloads/transport contracts | 37c: streaming and alternative transports | Complete-operation comparison; no unconfirmed WS cutover | Scheduled |
| I4 | Retention and backend behavior | 37c: history, files, reclaim/compaction | Explicit logical/physical outcomes and diagnostic need; no TTL/byte-return inference | Scheduled |
| I5 | Actual graph/query access | 37c: indexes/functions/EXPLAIN/edges/caches | Useful shape or no-consumer decision; no wholesale conversion | Scheduled |
| I6 | Fixture/inspection/maintenance consumers | 37c: testcontainers/MCP/Studio/metrics/wakeups | Minimal compatible integration or observable trigger | Scheduled |
| I7 | Confirmed local/retry limits | 37c with 37a: checkpoint and extension boundaries | Sufficiency/extension decisions; no remote/resume/search/WASM commitment | Scheduled |
| Q1 | Adopted functions implemented, I outcomes settled | Coordinator: assembled affected acceptance | Fresh actual Rust/native/Python/recovery artifacts and scope-end checks, zero failure baseline | Scheduled |
| H1 | Q1 accepted, adopted findings resolved | Coordinator: enduring contracts/existing-plan reconciliation | Update other plans/status only now; retain their actual incomplete qualification | Scheduled |

### Dependency slices and editing ownership

Begin A0/C0 and D0 contracts already decidable. A4/B2/B3 can progress independently in current contracts. B1 waits on its public boundary decision/review. A1 may first protect an input with a conservative producer obligation, but final A2 consumers must use independent artifact ownership; a coarse run pin cannot close F02. Remove any temporary coarse path when replaced and tested.

A2 needs scientific sufficiency, not generalized chunk GC. A3 waits on dominating fences, settled creation windows and actual borrowers. Design it alongside A1/A2 so new detail does not accidentally become permanent. C0 can inspect current DDL early, then must recheck changed generated definitions. C1 sync-only fixture work can proceed after C0 and namespace ownership; its replay/parity closure also needs C2's initial frozen chain. This chain generation does not wait for persistent maintenance qualification. C2's persistent maintenance slice needs B1 quiescence and exclusion of every participating writer. C3 requires working final target contracts; Q1 rebuilds actual consumers after integration.

The coordinator owns common registry declarations, manifests, generated-output coordination, digest, decision route and activation. Independent work may share logical prerequisites but cannot race on these files. Use disjoint ownership/worktrees for concurrent editing. Codegen is part of declaration changes; never edit generated output by hand.

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f01) | Review S02 | Scheduled | A1 with A0/A2 | Exact admission survives retirement/restart/retry; retirement-first refuses. Current late binding was not an established wrong-answer bug. |
| [F02](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f02) | Review S03 | Scheduled | A0/A2 | Useful artifact survives producer retirement while unrelated outputs drain; whole-run pin alone insufficient. |
| [F03](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f03) | Review S03/S08 | Scheduled | A0/A3 | Detail drains, audit/unknowns survive, retired identity replay cannot resurrect. |
| [F04](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f04) | Review S04 | Scheduled | B1 | Actual dependents govern close; caller release cannot tear down another's work. |
| [F05](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f05) | Review S05 | Scheduled | B2 | Finite acknowledged ready/failure; no partial-session dispatch. |
| [F06](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f06) | Review S05 | Scheduled | B3 | Non-payload slots release without epoch collection; payload drain remains correct. |
| [F07](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f07) | Review S06 | Scheduled | A4 | Actual policy/test closure excludes unrelated integration dependencies. |

## Verification and completion

**Proposed verification:** targeted package controls establish mechanisms; Q1 establishes composed consumers. Authoring, inspected tests and accepted decisions are not a finding's resolution. During implementation compile touched packages and run revealing unit/mechanism tests through scripts/pse-env, with explicit force-validation for Arrow-consuming closures. Delete replaced code/callers/tests/fixtures once replacements pass and callers move. Regeneration accompanies changed declarations.

At functional scope end, Q1 selects current canonical study, artifact/result/analysis, ordinary Rust and rebuilt native Python journeys, including cancellation and publication. Include real solve/start reuse and dynamic results where retention changes, preserving independent scientific checks and partiality. Exercise fresh setup, interruption, service restart, namespace disposal and maintenance owner loss in owned fixtures. Rebuild the real worker/extension before installed-boundary checks.

Run just hygiene and relevant governance/docs/native/powerset checks at scope end, fixing scoped failures and reporting command, mode, scope, baseline zero and actual result. Q1 owns affected acceptance, not the stopped Plan 36/28 campaigns. A relevant managed study can establish wiring without closing those separate campaigns. Quantitative claims need representative measurements; dependency closure or logical collection does not establish build speed or returned disk space.

I packages must record a settled adoption/non-adoption outcome, consumer/trigger and any dependent implementation. A material new rule impact needs confirmation. No unbounded investigation may masquerade as completed scope. Full numerical model breadth, power-loss guarantees and remote deployment remain outside qualification unless explicitly selected and evidenced.

Completion requires adopted packages implemented and replacements deleted, I outcomes recorded, seven findings resolved with evidence, fresh Q1 accepted, and enduring meaning transferred through the design route. H1 then reconciles only affected existing plans, preserving their remaining work and original evidence limits. Do not promote Plan 36's interrupted managed run or resume it automatically.

## Current checkpoint and authoring outcome

The four plan documents are authored. Product implementation is **not_run**. All nine recommendations and OP01 are confirmed. A0/C0 deliberately own remaining scientific-closure and beta-compatibility decisions; write their results into dependent designs before coding those branches. Documentation checks establish publication only.

Next executable work after product authorization: establish D0's ready contracts, settle A0/C0 and undertake independent A4/B2/B3. Existing execution plans/status remain unchanged. No service/store/data cleanup is performed by this authoring task.

## Outcome (recorded after implementation)

### What was built

Not yet implemented. Record evidence labels and named acceptance conditions when Q1 closes.

### A mistake made and corrected

Record an actual implementation correction at closeout.

### Deviations from the plan, deliberate

Record actual departures and their decision route at closeout. OP01 is already the target, not a future deviation.
