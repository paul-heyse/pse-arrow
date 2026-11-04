---
title: "Plan64: Process-simulator target"
status: in-progress
date: 2026-11-02
adrs: []
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md, docs/plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md, docs/plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/README.md
doc_topics: [current-work, design, validation]
---

# Process-simulator target

## Purpose and authorization

**Proposed:** deliver reusable native process science and coherent engineering operations over the existing authored system, then close a finite expanded IDAES process-capability population. This series captures the integrated [target review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md), its [recommendation audit](../design_review/reviews/audits/design_review_process_simulator_target_2026-11-02_audit.md), and the [high-level package](../process-simulator-design/index.md). The maintainer authorized plan authoring on 2026-11-02 and selected full P0–P7 coverage with staged detail. Subsequent execution instructions authorize completion of all Proto-related scope first, followed by the entire remaining Plan64 scope. The companion package tables own implementation and acceptance state; authorization does not establish completion.

**Interface-checked:** authoring starts at `1acd8c83ca8f9e6f6e9b5e9c9cbf98e299d574b8`. The integrated review and current source retain scientific/runtime mechanisms inspected at `dcbb1c459`; the intervening commit publishes the recommendation audit. Concurrent skill and library-leverage drafts are excluded. Core3.4, efficiency heuristics1.0, ProcessSimulator1.5, test-assessment1.2 and the selected binding apply. No new broad audit, numerical execution or benefit measurement is claimed.

## Combined target and foundation assessment

**Execution authorization — 2026-11-03:** the maintainer authorized all remaining 64d–64g
packages, including the necessary enabling 64h inventory/reference and 64a contract slices.
The existing companion tables remain their status owners. This authorization does not close
H01's unenumerated population or qualify broader 64h/Plan58h work. Initial execution joins the
finite required-population inventory, state-coordinate and reaction foundations, and G08's
atomic capture replacement. Proposed ADR-0230 and blueprint revision344 record the capture
target before its production migration.

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

### Proto integration scope and authorization

The maintainer selected all six recommendations in the
[Proto improvement review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md)
for full planning integration, including breaking replacements of delivered B/C interfaces.
Both the local engineering service and local scientific worker are working-delivery targets,
not design-only investigations. The initial authoring instruction was followed by explicit
authorization to implement the complete Proto scope and then the entire remaining Plan64
scope. Earlier D–G closeout failures remain obligations of their existing owners.

The planning baseline was `2550fa665` after scientific/capture integration at `480628c1e`,
while capture/native-service tests and D–G checkpoint edits were still in progress. Their
closeout evidence remains owned by those companions; the current execution checkpoint is below.
Core 3.4, heuristics 1.0, ProcessSimulator 1.5 and test-assessment 1.2 apply. The
[briefing](../library_leverage/libraries/proto/briefing.md) and linked exact library contracts
inform implementation, rather than treating general Proto capability as local eligibility.

| Proto review rule impact | Operator decision (2026-11-03) | Required route |
| --- | --- | --- |
| RC01 | accepted: projections derive from actual registry and Rust operation/document owners | Proposed ADR-0239 and blueprint §4.7/§21.8; 64i settles source annotation/emitted-metadata realization before generation |
| RC02 | accepted: explicit versioned external/durable boundary breaks | 64i inventories actual readers/writers and retained interpretations; applicable decision and cutover before affected publication; semantic hashes/admission remain owned |
| RC03 | accepted: explicit local effectful service and optional foreign-client contract | Proposed ADR-0239; 64j uses shared typed handlers, preserving passive-reader boundaries and original operation custody |
| RC04 | accepted: explicitly prepared local scientific-worker binding | Proposed ADR-0240 and blueprint §9.11; 64k delivers the worker; remote deployment/distributed solving remain separate |

### Proto improvement dispositions

This is the sole scheduled-disposition owner for this review's I01–I06. These are
review-local references, distinct from the recycle review's I01–I12 and native packet IDs.
Companion tables own implementation progress. Scheduling is not resolution.

| Finding reference | Disposition | Decision / work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| I01 [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I01) | scheduled | 64i; 64a shared decision context | Owner-derived projection reaches complete selected consumer population; independent fidelity/extension controls and replaced mapping deletion |
| I02 [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I02) | scheduled | 64i; B/C source/edit consumers | Generated local rules and exact-parent edits preserve contextual admission and attributable failure |
| I03 [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I03) | scheduled | 64j; G scientific operation consumers | Independent generated client completes actual local service journey, reconnect, failure and native settlement |
| I04 [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I04) | scheduled | 64k; D/F provider consumers | Working authenticated local worker preserves exact science/demand/context and process settlement against in-process controls |
| I05 [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I05) | scheduled | 64h H02–H04; 64i projections | Independent case/output exchange, semantic mismatch controls and complete required output coverage |
| I06 [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I06) | scheduled | 64i/64j; C/G/H selected consumers | Selection reaches producer; removed repeated materialization retains extent/interruption guarantees; quantitative benefit requires matched whole-lifetime measurement |

64i/64j co-design one usable engineering slice; they do not wait for a universal catalogue.
64k consumes exact provider meaning and the required wire slice independently of the full
engineering service. H reference construction can proceed from settled exchange contracts.
D–F scientific increments depend only on the slices they consume. A schema, method listing,
accepted ADR or source projection does not establish prepared capability or scientific permission.

Expanded series completion requires working 64i–64k deliveries, affected B–H consumer
migrations/deletions and H's selected assembled evidence alongside the original finite scientific
population. Preserve B/C's original outcomes as historical evidence; explicitly version/migrate
new interpretations instead of retaining a second production path. Numeric equivalence,
structural compatibility and scientific usability retain separate checks. Required formal
decision reviews remain separate from this plan-authoring foundation assessment.

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
| [64i Engineering contracts and validation](64i-engineering-contracts-and-validation.md) | Proto I01/I02 and shared I06 | Owner-derived projections, typed edit/inspection and complete migration |
| [64j Engineering services and clients](64j-engineering-services-and-clients.md) | Proto I03 and service/client I06 | Working local engineering journey with original outcome/custody |
| [64k Prepared scientific workers](64k-prepared-scientific-workers.md) | Proto I04 | Working explicitly prepared local provider worker and selected science controls |

The coordinator owns review dispositions and shared decisions; companions own package state. These document groups are not serial execution lanes. Start with A01 and the finite reference/input inventory in 64h/64b. Settle A02's small shared contracts before affected APIs. Ordinary input A, native interface B/C and independent reference construction D can overlap once their scientific correspondence and consumed contracts agree. Integrate complete first delivery before broadening its completion claim.

P2/P3 grow by consumer closure, not by waiting for every source family. Steam can unlock a power track while electrolyte/transport work unlocks capture. Distributed variants consume the required P4 slice; steady units do not wait for every dynamic feature. P6 can consume an accepted steady model before all specialized families exist. P7 assembles actual completed populations; it cannot repair absent physical foundations with a campaign.

Logical dependencies, shared editing surfaces and physical work placement differ. Source/registry/Cargo declarations and shared permission APIs need explicit writers during execution even when scientific work is independent. Allocate no new scheduler, global readiness machine, permission service or catalog framework merely to coordinate this series.

## Review finding dispositions

| Finding | Disposition | Work owner | Completion or settling evidence |
|---|---|---|---|
| F01 / DA-039 manifestation | resolved | 64c C03 | Tested native success/intended failure and selected original-report readback in 64c's verification; specifications and no-commit assertions survive. Broader DA-039 remains outside this manifestation. |
| F02 / DA-049 | resolved | 64c C04 | Tested producer-derived selection permission and affected native readback; label changes cannot upgrade unknown/unusable science. See 64c's verification. |
| F03 / DA-048 | scheduled | 64c C05; 64g consumers | C05's native applicability/identity consumers are complete and tested; remaining analysis-family consumers belong to 64g. Structural analyses retain separate applicability. |
| F04 / DA-006 | unresolved | 64a A03; all consumers preserve its constraint | Identify an actual duplicate interpretation or missing consumed distinction before refactoring; otherwise retain existing lowering and close the investigation with that reason. No compiler-migration prerequisite. |
| F05 | resolved | 64a A02; 64c C02 | Tested C02 reuse/invalidation controls preserve consumed method distinctions and compatible value-only structure; see 64c's verification and A02's shared identity contract. |
| F06 | scheduled | 64a A02; 64c C05; 64g | C05's native structural/task-level exposure is complete and tested; remaining task-family consumers belong to 64g. Point-based products retain their real point. |

## Recycle improvement dispositions

This table is the single scheduled-disposition owner for the
[recycle performance/precision review](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md)
I01–I12. On 2026-11-02 the maintainer selected all findings for staged integration and
directed implementation of plausible optimizations without belaboring benefit selection.
This supersedes the review's conditional-adoption posture for plausible optimizations;
its source observations and evidence limits remain unchanged. Measurements refine changes
and expose regressions; correctness/compatibility questions remain inside the work package.
Keep first-party Rust at opt-level 2; a 2→3 increase is explicitly excluded.

| Finding | Disposition | Primary work owner and consumers | Completion or settling evidence |
|---|---|---|---|
| [I01](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I01) | resolved | 64c C02 | Exact field selection reaches tracked reuse before construction; semantic/refusal and construction controls, with time/RSS and retained-working-set refinement. |
| [I02](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I02) | resolved | 64c C02 | Indexed actual-instance discovery, shared membership/materialization and recipes preserve the same dependency closure without repeated owned projection. |
| [I03](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I03) | resolved | 64c C02; C03/C06 consumers | Valid preparation survives compatible lifecycle changes; original/tear/design/rating, independent attempts and default-start evidence retain their distinct premises. |
| [I04](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I04) | resolved | 64c C06; C02/C08/C09 inputs | Evaluator/numerical refinements use actual work and fixed meaningful requirements; original physical checks and failed alternatives retained; no Rust opt-level increase. |
| [I05](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I05) | resolved | 64c C08; 64b B01/B02/B04 inputs | Shared context covers governing targets using caloric/energy/flow and relevant goals; unit/datum invariance, missing-context refusal and coupled original acceptance. C07 stays distinct. |
| [I06](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I06) | resolved | 64c C09; C03–C06 and 64b consumers | Preconstruction activation and async propagation; successful/failed original reports retained and readable; diagnostic and scientific outcomes independent. |
| [I07](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I07) | resolved | 64c C09 | Selected interval completeness, recent history and bounded analysis stay distinct; loss/expiry/prefix limits and managed child coverage discriminate. |
| [I08](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I08) | resolved | 64c C01; 64b B03–B05 consumers | Selected/full ordered reads avoid edit products; exact order/custody/refusals and subsequent edit/CAS/publication survive, with real work counts. |
| [I09](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I09) | resolved | 64c C02; 64b B03/B05 consumers | Pre-reset census/cause, improved keys/history and owned extents; complete equality/negative premises, A/B/A, growing history and escaped products survive rotation. |
| [I10](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I10) | resolved | 64c C09 | Buffered/batched recording and retain-once analysis preserve nonblocking offers, truthful flushed prefixes and stop/drain; measured diagnostic perturbation. |
| [I11](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I11) | resolved | 64c C02 | Exact selector/bindings reach prelude reuse before premises/maps; shadowing, membership, inherited context, permission and retained-extent controls. |
| [I12](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md#I12) | resolved | 64c C02; 64b B03/B04 consumers | Qualified selection is carried while invocation/context premises hold; changed-context/provenance/cancellation/force-validation controls preserve receiving guarantees. |

**Implemented and Tested:** the companion package tables and
[64c's verification](64c-process-authoring-and-first-delivery.md#verification-to-date)
own evidence for these resolutions, including selection/custody controls, complete selected
native journeys and independent offline readback. **Measured:** that same account retains
actual preparation, callback/evaluator/factor work, retention observations and diagnostic
perturbation under their named conditions. No stable general speedup follows from the
shared-host samples. The current compiled alternative refuses its constructor allowance;
interpreted execution is qualified. The historical exchanger's earlier inconclusive cause
was not recoverable, while its current complete fixture report passes; neither is recast
as evidence of an inferred repair. The review's prototypes and incomplete captures retain
their historical scope. No new rule exception or Rust optimization-level increase is selected.

The integrated native work combines C09/retention observations, C02 selection corrections,
C01/64b reads and B/C08 context. C06 consumes their policy/report slices for numerical
selection and native acceptance. Existing Plans40/53/54/58 retain their responsibilities; the
collective dependency upgrade, H02/H03 references and wider qualification remain separate.

## Complete recommendation and investigation coverage

This table owns scheduling correspondence to the integrated target review; the preceding
I01–I12 table owns the improvement-review additions. The audit companion owns editorial
dispositions, not progress. Do not duplicate these states in a second review ledger.

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

### Current completion and remaining scope — 2026-11-04

Joint execution resumes from clean main `83485d6f6`, incorporating the Plan65 deliveries
and consolidated worker handoffs. Source drafts, compiled owners, generated publication and actual-process acceptance
are distinguished below; no percentage or package completion is inferred from file counts.

- **Completed:** A02–A04's design/disposition work and B01–B05/C01–C09's original native
  delivery, within their recorded evidence. The new Proto migrations of those surfaces
  remain owned by I/J; the retained B/C outcomes do not qualify the changed checkout.
- **Implemented and targeted Tested slices:** source-owned projection and local-rule
  machinery, complete authored registry row declarations, exact numeric/duration/physical
  value handling and passive scientific metadata in [64i](64i-engineering-contracts-and-validation.md#current-implementation-checkpoint--2026-11-04);
  a declared-case engineering client/service slice in [64j](64j-engineering-services-and-clients.md#current-implementation-checkpoint--2026-11-04);
  and prepared water-worker transport, native custody and actual-process controls in
  [64k](64k-prepared-scientific-workers.md#current-implementation-checkpoint--2026-11-04).
  Each owner distinguishes its passing snapshot from later unverified source changes.
- **Proto integration still required:** the earlier `HighsSettings.sparse_start` CEL
  lowering failure is repaired and complete generation now passes through Plan65's shared
  foundation. Complete full Study/model/provider RPCs and all selected operation families; prove
  original Study identity, frontend replacement, protected cohorts and native settlement;
  complete the worker equipment and independent binary reference comparisons. I01–I06
  remain scheduled, with their resolution criteria unchanged.
- **Remaining scientific scope:** A01/H01's complete finite population and input closure;
  D's remaining property/reaction/equipment and HDA/methanol integrations; E's native
  inventory, distributed/cyclic and derivative acceptance; F's configured power,
  gas-solid, capture/membrane, TSA/SOC and network families; G's remaining inference,
  uncertainty, surrogate/discrete, control/multiperiod and assessment consumers, plus
  G08's failing capture controls and legacy retirement. Their companion tables retain
  the exact package and configuration obligations. Proto metadata does not implement
  those scientific families.
- **Remaining assembled acceptance:** H02/H03's native/reference correspondence and
  negative controls, H04/H05's family/operational acceptance, applicable H06 measurements
  and H07's full scope-end checks, enduring publication and final evidence reconciliation.
  No comprehensive qualification of the current checkout has passed.

The worker, independent Rust client and reference-exchange journeys are acceptance for
the Proto additions using real Plan64 science. They also contribute to H's functional
qualification at their exact scope; they do not close all D–G families. Scientific output
assertions must consume the original quantity/unit and analytically meaningful engineering
error budgets. Scaled solver stopping tolerances and a successful termination are separate
facts, not certificates of physical output error.

One coordinator now executes both series with Proto delivery first. The shared publisher and
actual Recipe codec/qualification, protected capture/recovery, scientific input closure and
service/client integration are the first work packages. Scientific fixes needed to prove
the Proto paths and the finite input/configuration census proceed alongside them; broad
D–G feature expansion follows the Proto delivery milestone. Consume Plan65's current pure
foundation, metadata, completion observations, explicit worker Shutdown and batching instead
of recreating older mechanisms. Companions retain their native status/evidence ownership;
the Plan65 coordinator owns the shared delivery and remaining activation boundaries.

### Completed B/C evidence checkpoint — 2026-11-03

A02/A03/A04 design and disposition work is complete;
A01 remains in progress pending H01's exact finite population. B/C native implementation
and selected scientific acceptance are integrated under their companion package tables.
Ordinary recycle/rejection, exchanger design and same-case design-to-rating, compatible
closure and actual canonical source publication/reopen/RunHandle/readback have passed.
C08's ordinary strict inventories and complete historical tear/original inventories have
no canonical fallback; C09 retains actual rejected native feasibility separately from
passing declared checks. Source/preparation, evaluator/factor and native compilation
captures retain their actual bounds, failed alternatives and measurement conditions.
No stable general speedup is claimed. The current compiled candidate's constructor refusal
is retained; the passing interpreted realization is selected without an allocation-cap or
Rust optimization-level increase.

**Interface-checked:** final-source prefix/suffix qualification covers all 495 depth-two
feature configurations and 34 no-default workspace members. Nine warning-bearing xtask
configurations passed affected clean reruns after their generator gate was corrected;
the private-catalogue preservation test and generated-output comparison also passed.
Native-feature Clippy and the other scope-end checks passed for that completed B/C
baseline; they have not been run for the subsequent D–G changes. The companion
verification records the composite runs and corrected failures; the stopped original
aggregate remains stopped/failed. I01–I12 are resolved for their selected native scope.
C07 owns the
selected DA-062 correction and G08 the DA-063 atomic publication-witness replacement.
H02/H03 execution, broader H01/P7 qualification, G08 and the collective library upgrade
retain their separate owners. This checkpoint does not close A–D or the full series.

64a's [library-informed decisions](64a-scope-and-shared-contracts.md#executed-design-assessment--2026-11-02) govern the subsequent target: library-owned incremental/symbolic/structural operations, mixed CPU ownership, registry-generated native wire projections and an atomic witness capture rather than durable pending exclusion. Writable Surrealism is a preferred bounded prototype, not an approved capability widening: record its purity/attestation rule impact as a proposed decision before G08 adopts it. Ordinary dependency upgrades need no special approval; actual architecture/durable-contract changes retain their decision routes. The operator intends a separate coordinated upgrade of the latest compatible library set; current pins are not fixed target constraints.

**Completed native handoff:** B01–B05 and C01–C09 are complete under their companion
outcomes. Their records and B's source-selection account remain dependencies of this
coordinator, H02/H03 and proposed ADR-0196; owner release is not established. They are
removed from current-work selection and retained for that handoff. Enduring contracts
remain with the architectural owners and ADRs referenced by those plans. This closes no
other companion or broader scientific population.

## D through G implementation checkpoint

**2026-11-03 — execution resumed and extended.** The maintainer authorized completion
of all Proto-related I/J/K and cross-plan scope, followed by completion of the entire
remaining Plan64 scope. The earlier stopped wave below remains historical evidence;
its failures and unrun obligations remain open until corrected and verified. Current
Proto implementation checkpoints live in 64i/64j/64k. Complete that integrated scope
first, then continue the existing A–H package owners and the finite scientific population.

**Historical closeout, 2026-11-03 — stopped at the maintainer's request.** That
implementation wave was partial and its fixes, tests and diagnostics stopped. The independent agent's
proto/extensibility plan additions retain their own scope; they were not implemented
or qualified by this wave. The D/E/F/G package tables remain open, and their closeout
sections own the detailed passing evidence, failures and next obligations:
[scientific foundations](64d-scientific-assets-and-equipment.md#stopped-implementation-checkpoint),
[inventory](64e-inventory-distributed-and-cyclic-physics.md#stopped-implementation-checkpoint),
[steam/power](64f-specialized-process-families.md#stopped-implementation-checkpoint), and
[fitting/capture](64g-analyses-studies-and-applications.md#stopped-implementation-checkpoint).

**Implemented:** the selected scientific foundations, explicit CoolProp binding,
joint-fitting numerical-context correction and versioned atomic-capture implementation
are preserved. Later corrective edits also remain in the working tree. This does not
establish completion of D–G or the finite scientific population.

**Tested, historical partial checkpoint:** selected compiler and fitting controls passed, as did the frozen-v3
migration journey. Steam equipment admission still fails; ordinary capture controls
still have stack overflows, and the large-capture correction is not qualified. Every
failure is against the zero-failure baseline, not an accepted allowance.

Current joint execution has repaired the ordinary capture stack failures; the latest
[G08 evidence](64g-analyses-studies-and-applications.md#g08--atomic-capture-and-unrelated-liveness)
retains the unresolved 10,000-member timeout/drain result. Fresh captured-input steam cases
now expose the [K05 physical-context and authored-operation gaps](64k-prepared-scientific-workers.md#current-implementation-checkpoint--2026-11-04).
These current owners supersede the historical failure descriptions above.

**Not run / deferred by explicit instruction:** Clippy (including keep-going workspace,
native-feature and WASI coverage), hygiene, codegen comparison, additional regressions,
integrated/reference/performance campaigns and turn-end follow-ups. Full `just codegen`
passed ten targets before the later source-only corrections; that is not a final
generated-output comparison or a clean-tree qualification. No comprehensive static
or functional pass is claimed for this checkout.

At that closeout, all task-owned build/test/debug sessions were terminal. Worktrees were
reconciled and removed; the unused unintegrated Plan59 parser-provenance draft is preserved on its
original branch at `3d7096c15`, rather than reintroducing retired production code.
The shared sccache daemon was left running for other agents; it is not an active test
or build. Resume the recorded failures and pending targeted checks before starting
further implementation from this wave.
