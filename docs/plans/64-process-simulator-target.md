---
title: "Plan64: Process-simulator target"
status: draft
date: 2026-11-02
adrs: []
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
doc_role: plan
doc_owner: docs/plans/README.md
doc_topics: [current-work, design, validation]
---

# Process-simulator target

## Purpose and authorization

**Proposed:** deliver reusable native process science and coherent engineering operations over the existing authored system, then close a finite expanded IDAES process-capability population. This series captures the integrated [target review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md), its [recommendation audit](../design_review/reviews/audits/design_review_process_simulator_target_2026-11-02_audit.md), and the [high-level package](../process-simulator-design/index.md). The maintainer authorized plan authoring on 2026-11-02 and selected full P0–P7 coverage with staged detail: A–D executable, later phases concrete outcomes and bounded investigations where decisions remain open. This does not authorize production execution.

**Interface-checked:** authoring starts at `1acd8c83ca8f9e6f6e9b5e9c9cbf98e299d574b8`. The integrated review and current source retain scientific/runtime mechanisms inspected at `dcbb1c459`; the intervening commit publishes the recommendation audit. Concurrent skill and library-leverage drafts are excluded. Core3.4, efficiency heuristics1.0, ProcessSimulator1.5, test-assessment1.2 and the selected binding apply. No new broad audit, numerical execution or benefit measurement is claimed.

## Combined target and foundation assessment

An engineer authors a physical definition once, selects a case and task, inspects its meaning, makes attributable edits, prepares a supported numerical realization and obtains physical outputs or useful failures. Scientific definitions, parameter selections and evidence retain exact identities from source to stored revision and consuming case. A value edit can reuse compatible structure; topology, selected science, roles and required products retain their actual invalidation effects.

| Existing foundation | Assessment and target consequence |
|---|---|
| `pse-runtime` modeling/source-delta and canonical services; `pse-admission` | Reuse parented immutable edits, contextual checking, staging and guarded activation. Do not add another accepted-knowledge application. |
| `ModelingPackage::with_declarations` and `prepare_flow` | Typed declaration editing already removes old text authority; topology preparation is independent of numerical incidence. Use these operations for the first native interface. |
| `prepare_declared`, `initialize_declared`, declared diagnostics | Solve preparation is not universal task admission. The full initialization-report route exists; migrate process-facing initialization consumers rather than implement another initializer. |
| `completion.rs`, connected-result selection | Reuse completion permission and exact step evidence; correct exported caller-supplied usability and extend product applicability/scope only for exposed products. |
| Specialized closure obligations and compiler annotations | Runtime consumes prepared endpoint/context/lineage and binds late numerical policy. F04 does not justify a mandatory compiler migration. |
| Scientific packages and independent reference drivers | Reuse native definitions and analytic controls; fixture/oracle records do not automatically satisfy ordinary-input eligibility. |
| Existing native numerical owners and canonical substrate | Keep repeated evaluation in prepared native objects and state in its actual attempt owner; store/discovery calls do not enter scalar residual loops. |

Read source at `crates/pse-runtime/src/workflow/modeling.rs`, `modeling/declared.rs`, `modeling/cases.rs`, `completion.rs`, `connected_results.rs`, scientific research/publication owners and `crates/pse-modeling/src/specialize/process.rs`. Blueprint §8–§14, §19–§22 and §24.2 own current contracts. These pointers bound investigation, not the affected consumer migration inventory.

## Confirmed rule changes

All four directions were explicitly **accepted by the maintainer on 2026-11-02**. Acceptance is a design decision, not evidence the rule documents changed.

| Review item | Decision and consequence | Route and implementation prerequisite |
|---|---|---|
| RC01 | Accept expanded released-process target; finite configurations/cases and separately pinned examples/compatibility references | 64a A01 reconciles relationship/scope owners through the design/ADR route where required, before dependent expanded claims. Preserve clean-room separation and IDAES2.13.0 pin. |
| RC02 | Accept correction of stale Python authoring/result wording | A01 edits the binding description through its owner, consistent with current native product and isolated scientific references; no restoration of a Python product. |
| RC03 | Accept semantic-request association versus prepared dependency identity | A02 reconciles catalog and execution/hierarchy descriptions before affected preparation APIs freeze. Each artifact consumes only its actual dependencies. |
| RC04 | Accept task-owned analysis products with optional point/member association | A02 reconciles owner, both owning lists and hierarchy; C05/G01 consume this without requiring a successful physical state for structural diagnosis. |

Bug fixes restoring declared report and producer-permission guarantees need no policy exception. A change to an enduring D decision, durable contract or other listed decision trigger still follows the existing ADR/design route before implementation. This plan does not amend accepted ADRs or authoritative architecture in passing.

## Documents, shared contracts and execution route

| Companion | Owned outcome | Handoff |
|---|---|---|
| [64a Scope and shared contracts](64a-scope-and-shared-contracts.md) | P0 rules, admission, identities and focused foundation investigations | Agreed meanings and exact prerequisite slices; not a universal dispatcher |
| [64b Native scientific intake](64b-native-scientific-intake.md) | A and growing scientific-source path | Ordinary eligible exact scientific closure and retained readback |
| [64c Process authoring and first delivery](64c-process-authoring-and-first-delivery.md) | B/C and native contribution to D | One usable edit/prepare/run/result path and F01–F03 consumers |
| [64d Scientific assets and equipment](64d-scientific-assets-and-equipment.md) | P2/P3 | Executable property/reaction/unit configurations with consumers |
| [64e Inventory, distributed and cyclic physics](64e-inventory-distributed-and-cyclic-physics.md) | P4 | Qualified storage, transfer, temporal/spatial and event contracts |
| [64f Specialized process families](64f-specialized-process-families.md) | P5 | Configured power, gas-solid, capture and electrochemical/adsorption tracks |
| [64g Analyses, studies and applications](64g-analyses-studies-and-applications.md) | P6 | Task-specific derived products, control, inference and assessment |
| [64h Reference coverage and qualification](64h-reference-coverage-and-qualification.md) | P0 finite denominator, D and P7 | Matched independent references, assembled evidence and truthful scope |

The coordinator owns review dispositions and shared decisions; companions own package state. These document groups are not serial execution lanes. Start with A01 and the finite reference/input inventory in 64h/64b. Settle A02's small shared contracts before affected APIs. Ordinary input A, native interface B/C and independent reference construction D can overlap once their scientific correspondence and consumed contracts agree. Integrate complete first delivery before broadening its completion claim.

P2/P3 grow by consumer closure, not by waiting for every source family. Steam can unlock a power track while electrolyte/transport work unlocks capture. Distributed variants consume the required P4 slice; steady units do not wait for every dynamic feature. P6 can consume an accepted steady model before all specialized families exist. P7 assembles actual completed populations; it cannot repair absent physical foundations with a campaign.

Logical dependencies, shared editing surfaces and physical work placement differ. Source/registry/Cargo declarations and shared permission APIs need explicit writers during execution even when scientific work is independent. Allocate no new scheduler, global readiness machine, permission service or catalog framework merely to coordinate this series.

## Review finding dispositions

| Finding | Disposition | Work owner | Completion or settling evidence |
|---|---|---|---|
| F01 / DA-039 manifestation | planned | 64c C03 | Full initialization evidence on success/failure and local/selected recorded readback; intended failure reaches its actual stage and restores specifications. Broader DA-039 remains outside this manifestation. |
| F02 / DA-049 | planned | 64c C04 | Exact producer-derived selection permission; label changes cannot upgrade unknown/unusable science; coordinates/coverage reconcile. |
| F03 / DA-048 | planned | 64c C05; 64g consumers | Product identity includes actual step/member/reference point and assumptions; structural analyses use their own applicability. Exposed consumers migrate together. |
| F04 / DA-006 | unresolved | 64a A03; all consumers preserve its constraint | Identify an actual duplicate interpretation or missing consumed distinction before refactoring; otherwise retain existing lowering and close the investigation with that reason. No compiler-migration prerequisite. |
| F05 | planned | 64a A02; 64c C02 | Same semantic question with incompatible consumed method settings does not collide; value-only edits can preserve compatible structure. |
| F06 | planned | 64a A02; 64c C05; 64g | Task-level analysis with zero numerical members; consistent owner/owning lists; point-based products retain their real point. |

## Complete recommendation and investigation coverage

This table owns scheduling correspondence to the integrated review. The audit companion owns editorial dispositions, not progress. Do not duplicate these states in a second review ledger.

| Review obligation or question | Route and expected decision/evidence |
|---|---|
| §2 retained responsibilities, process operations, editing representation and topology | A02/A03 and C01–C02: native typed declaration editing first; source/declaration/instantiated/numerical scopes and effects explicit. |
| §2 task profiles, late binding, required/optional products and reconstruction | A02, C05 and 64g: supported route mapping and attributable refusal; no universal enum or invented inverse. |
| §3 physical conventions, states, inventories, reactions, empirical/population/observation/assessment meaning | 64d–64g: exact units/bases/datums, scoped conservation, validity and evidence per selected consumer. Unknown chemistry stays unknown. |
| §3 targeted sourcing, prepared evaluation and scientific discovery | 64b: exact source roles/variants/joint fits/missing data, typed deltas, guarded publication, declared query coverage and no hot-path database lookup. |
| §4 S01/S02 composition and edits | C01/C02/C06 and 64d: exchanger rating/design and compatible closure substitution without rewritten side balances. |
| §4 S03–S05 failures, export and diagnosis-only analysis | C03–C05: independent failure attribution, producer permissions, full reports and no phantom state. |
| §4 S06 reuse/concurrency/interruption and §5 execution fit | All companions, assembled 64h: immutable preparation reuse, attempt-local mutation, scoped custody/drain and proportional materialization; measurements only for quantitative claims. |
| §4 S07–S09 dynamics, joint inference and broader semantic stress | 64e–64g: physical continuity, shared parameters, population measures and weaker empirical evidence; illustrative industries do not automatically enlarge P1. |
| §4 S10 and §8 external/native alternatives | 64d–64g compare exact values/derivatives/datum/branches/effects, residual versus IO versus time advance; preserve candidates and their reopening conditions. |
| §6 inherited DA-062 / DA-063 | A04 establishes current applicability and one repair owner before dependent broad acceptance. Shared interval meaning must preserve hard guards; capture isolation must preserve complete membership and recovery fencing. |
| §7 gates and §10 verification | 64h: architectural and scientific judgments remain separate; source reading is not a numerical pass or PS-G qualification. |
| §8 authored-native default, optional external realization, rejected mirrored hierarchy/universal planner, fixed collection versus streaming | All target designs retain comparative rationale. A consumer can reopen a choice with concrete evidence; no automatic candidate adoption or compulsory streaming. |
| §9 P0–P7 and first A–D delivery | All companion outcomes, explicit package prerequisites and 64h finite population closure. No phase is omitted because only P1 receives immediate execution detail. |
| §11 ordinary scientific-input eligibility | B01/B02 identify exact records, provenance and missing sources before ordinary execution. |
| §11 reference compatibility | 64h freezes matched configuration and mapping, independently pinned repositories and prerequisites; retain blocked cases in denominator. |
| §11 derived-product scope | C05/G01 choose exposed products and complete consumer migration; broader families remain named work rather than assumed support. |
| §11 dynamic breadth | 64e settles exact index/layout/event/derivative fit before choosing integration. |
| §11 empirical/population sufficiency | 64d/64e choose a contrasting authored case; a counterexample to existing axes/relations/quantities, not an industry name, triggers generic change. |
| §11 external substitution and assembled execution | 64d–64g capability decisions, 64h measurements/qualification; library availability is not feasibility or achieved benefit. |
| §12 target, rule impacts and limits | Accepted RC decisions above, A01/A02 routing, shared completion below. |

## Existing plan interfaces

Plans28/32/40/53/54/58/58h retain their status and acceptance. Consume actual publication/revision/control contracts from 28, lifecycle metadata from 32, relevant observations from 40, scientific assurance/fixture/reference contracts from 53, supported build preparation from 54 and native operations from 58. Plan58h remains a separate assembled native/deployment qualification owner; this series does not declare it complete. Reuse an applicable receipt only with its original configuration and scope.

No active-plan reference by the scoped DA-062/DA-063 IDs was found during preparation. A04 must inspect semantic overlap with existing plan packets before allocating repair. If already owned, name the exact packet and consume its evidence; if unowned and applicable to selected scope, establish one explicit owner here or in that existing plan through its authorized route. Do not silently absorb unrelated numerical or canonical work.

## Verification and completion

**Proposed verification:** targeted compile/owner tests during implementation with actual Arrow force-validation. Independent scientific assertions remain separate from shared production calculation. Local tests, complete native/reference journeys and measurements establish different claims. Scope-end hygiene, governance/docs, applicable native/powerset checks and integrated campaigns run after the selected functional scope is assembled, not after every packet edit.

A–D complete only when ordinary source closure, edits, full success/failure journeys, producer permission, exact readback and complete matched reference outputs pass their own discriminators. P7 closes every mandatory finite population item with accepted evidence, zero failures and truthful public support limits. Missing required prerequisites fail selected runs; unattempted work never counts as a pass. Twelve domain witnesses and 55 notebook families are planning inputs, not the final denominator.

**Current checkpoint — 2026-11-02:** 64a A02/A03/A04 design and disposition work is complete; A01 remains in-progress pending H01's exact finite population. RC02–RC04 descriptive/catalog corrections and RC01's target wording are published; no expanded science or production adoption is established. C07 owns the DA-062 correction and G08 the DA-063 atomic publication-witness replacement. Other production packages remain planned. Next ready design work is H01/B01 input/reference closure; no runtime repair or library upgrade was executed here. All quantitative performance/capacity benefits remain unmeasured.

64a's [library-informed decisions](64a-scope-and-shared-contracts.md#executed-design-assessment--2026-11-02) govern the subsequent target: library-owned incremental/symbolic/structural operations, mixed CPU ownership, registry-generated native wire projections and an atomic witness capture rather than durable pending exclusion. Writable Surrealism is a preferred bounded prototype, not an approved capability widening: record its purity/attestation rule impact as a proposed decision before G08 adopts it. Ordinary dependency upgrades need no special approval; actual architecture/durable-contract changes retain their decision routes. The operator intends a separate coordinated upgrade of the latest compatible library set; current pins are not fixed target constraints.
