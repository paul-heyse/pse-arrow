---
title: "Plan 57: Library leverage and correctness"
status: in-progress
date: 2026-10-27
adrs: [ADR-0055, ADR-0082, ADR-0108, ADR-0118, ADR-0210]
review_sources: [docs/library_leverage/reviews/README.md, docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md]
scenario_sources: [docs/library_leverage/reviews/README.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, libraries, design, numerical-execution, storage, diagnostics, validation]
---

# Plan 57: Library leverage and correctness

## Database boundary review integration

The maintainer authorized creation of this amendment on 2026-10-28 from the
[study execution and database boundary review](../design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md)
and its scoped production/test audits and research. **Proposed:** adopt its coherent hybrid
as the combined planning target: database-facing durable commands/reads, optional local
scientific preparation and independently supervised native execution. This supersedes
mandatory scientific Runtime construction for durable callers in the planned target;
it does not declare the new boundary implemented. The maintainer subsequently authorized
the first K/L browse/cancel/reconcile slice; wider D–F qualification remains paused.
Existing source corrections and original evidence are preserved.

The review/audits at `277eee026`/`5e8aea0cc` inspect `64a217de2` plus the retained operations
inheritance corrections and 57j checkpoint. The historical whole-audit baselines are unchanged.
[57k](57k-durable-control-and-python-boundary.md) owns public contracts/imports/ingress and
consumer migration; [57l](57l-database-control-and-synchronization.md) owns transactional
composition, synchronization and payload/backend decisions. E retains Rust transport,
F lifecycle/attribution, G host/file mechanisms, I scientific public-family/developer adapters,
and J joined qualification/publication. A–C's completed selected work is not reopened.

### Confirmed review rule directions

All references below mean this review's RC identities, not the older RC01–RC09 in this plan.
Operator confirmation occurred on 2026-10-28 during plan preparation. It permits developing
the target; formal ADR acceptance and production execution are separate.

| Source impact | Decision | Route before dependent implementation |
|---|---|---|
| review RC01 | accepted — native-independent durable Python access | K01/K02 prepare Python-boundary ADR/formal design review and blueprint §21 amendment before K03 |
| review RC02 | accepted — public ingress may replace mandatory gRPC | K02 selects exact ingress; reconcile proposed ADR-0214 and blueprint §3.3/§20.6 before K03/L02; worker transport may retain useful gRPC |
| review RC03 | accepted — producer requirements follow scientific provenance consumption | K01 inventories operation requirements; supersede accepted ADR-0164-related decisions if changed and amend applicable architecture before migration |
| review RC04 | accepted conditionally — justified bounded module compositions only | L01 compares ordinary functions first; any selected capability change gets its specific decision/design route before L02 |

K/L authors assess exact affected decision text; acceptance of direction is not permission
for a blanket relaxation. Any newly discovered rule impact requires explicit operator disposition.
Keep accepted ADRs immutable. J schedules enduring amendments/revision rows through their owner.

### Coverage and execution dependency changes

R01's primary diagnosis remains DA-046; R02 remains DA-047. The following source-qualified
rows in the sole Finding dispositions table own scheduled correction status. Existing DTA-050,
DTA-055 and DTA-058 retain primary test-account homes; J consumes their corrected route,
fixture and evidence distinctions rather than allocating duplicate defects.

| Review material | Work route and completion boundary |
|---|---|
| R01/DA-046; S01 | K01–K05 and L01/L02: public durable use without native science construction, migrated callers and deletion |
| R02/DA-047 | F07: explicit poll/destruction contract, restoration controls and affected consumer adoption |
| S02; shared scientific meaning | L01/L02 and D/F: one scientific/policy owner for local and worker operations |
| S03; original receipts/replacement | L02/L03, E and G: unknown-effect reconciliation, stale fences and actual old-owner drain |
| S04; growth/data access | K05/L04 with A/G preserved owners: compact headers, bounded hydration and explicit payload placement |
| S05; connectivity/synchronization | K02/L03: unavailable versus failed, notification catch-up and lease/recovery authority |
| S06; library/transport replacement | E05/K02: full consumed contracts, not adjacent API similarity |
| Additional scientific research | D02: recorded points, DOE/ESE bounds, array materialization, GP reuse and restart distinctions |
| Corrected DA-033/034/035/042 premises | Preserve exact settlement, bulk resume/compact outcomes/inline payloads/lazy cohorts and exact telemetry selection; no work recreating them |

Useful next order is K01/L01's shared operation/actor contract, K02 ingress/package selection
and its decision route, then working L02 endpoint slices and K03/K04 consumers. F07 and E05
can settle independently where inputs are available; E05 waits on K02 only when public ingress
changes its actual obligation. L03/L04 and D02 are bounded inquiries, not automatic feature
adoption. K05 consumes selected L04 placement and existing data custody. J reconciles the
new working baseline before selected joined acceptance. Document letters do not impose serial
host execution or whole-document barriers.

Previous D–F implementation authorization is historical; it does not silently authorize this
new target or a restart of stopped services. The next execution engagement must reconcile
its selected packages and the three dirty operations corrections before compiling/running.
Full powerset/no-default validation remains deferred to series end, with all ordinary host
capacity available. Optional upstream contact, scientific method changes and backend migration
remain unselected unless their stated decision routes establish otherwise.


## Purpose, authorization and evidence boundary

**Proposed:** correct the exposed scientific/input-boundary defects and adopt the worthwhile
library/residual-owner improvements from the [reassessed leverage review](../library_leverage/reviews/README.md).
Capture the whole comparative judgment, not just its recommended replacements: all 53 allocated
CBO identities, split mechanisms, additional arithmetic findings, investigations, retained
contracts and deliberately unselected alternatives have a route below.

The maintainer authorized this document series on 2026-10-27 and selected resolving technical
choices during authoring where practical. That authorizes planning, its focused foundation
assessment and navigation/metadata integration. It does not authorize production implementation,
dependency installation, an upstream submission, supplier rebase, data reset, ADR acceptance or
a qualification campaign. All execution packages start planned/open. A published plan or accepted
ADR is not evidence of migrated consumers or corrected product behavior.

The coordinator owns the combined target, shared decisions, CBO/finding dispositions and overall
completion. Companions own their packages, local state and evidence; this document links those
owners rather than maintaining a second package-status table. Architecture sections retain
contract authority, ADRs rationale, and the leverage collections capability/comparative evidence.
Plans 28/32/40/53/54 keep their own outstanding work; completed Plan56 is an inspected foundation,
not authorization or an execution owner for these changes.

## Baseline and integral foundation assessment

**Interface-checked:** authoring starts at clean `93946f572`; product source is unchanged from
`4f52fd239`. The source review and integrated input corrections are `93946f572`, original review
`b29fcf802`, first codebase/library publications `c2f8b485a`/`34f9f07ee`, and new counterexample
evidence `1540fd164`. Use base patterns 0.2, additional patterns revision 4, Core 3.4, efficiency
heuristics 1.0, Process Simulator 1.5 and the selected repository binding. Applicable test evidence
is interpreted with the test-assessment companion. This is a focused assessment integral to
authoring, not another broad diagnostic audit or formal enclosing-system acceptance.

Read each companion's actual source/library baseline and supporting accounts. Existing
*Tested*/*Measured* observations keep their recorded source, mode, fixture and limits. In
particular, hostile IPC behavior is a library test plus a static product trace; the cubic probe
reconstructs the old ladder; direct-client 37/37 observations omit bounded transport; registry
timings are a small shared-host sample; rounding-cell and msgspec probes disprove proposed
details rather than test replacements. None establishes a current product fix.

| Foundation | Reuse and warranted change |
|---|---|
| Native columnar ownership | Reuse allocation scopes, attached input owners, force-validation and representation admission; cover decoder-owned residuals and expose safe framing at import boundaries before deleting whole-file grants |
| Scientific mathematics | Keep authored branch/residual meaning and library algebra/isolation; replace the unsafe cubic refinement/cache path without silently adding a correct-rounding requirement |
| Class-specific native execution | Keep independent original-space qualification, existing sparse choices and native destruction witnesses; fix supplier identity, terminal causes and shared helper boundaries; investigate the complete no-FMA closure |
| Canonical requests | Keep original clocks, acknowledged-only restoration, owned physical joins, independent clients and serialized admission; move SDK-specific result/session work only with those policies preserved |
| Lifetimes and observation | Keep completion ownership and authority separate from attribution; use neutral leaf mechanisms and explicit relationship-aware runtime context without reversing crate dependencies |
| Host publication and maintenance | Keep no-clobber/crash recovery, private ledgers, exact process/cgroup identity and supervisor ownership; consolidate mechanics without equating supervisor exit with group drain |
| Registry and language boundaries | Keep typed Rust declarations and independent source assembly authoritative; assess complete embedded-product compatibility and Python public-family realization before replacing their existing representations |
| Developer evidence and capture | Keep canonical SurrealDB and Rust domain enforcement; reuse generated Python declarations through real validation/publication adapters and repair incomplete-capture reporting at actual consumers |

These conclusions use current source and the complete review pages, including contrary cases.
They do not require universal new registries, proof manifests, alignment lints, cost models or a
benchmark per consolidation. Necessary pre-allocation accounting remains a product requirement;
that is distinct from inventing accounting machinery to justify architectural judgment.

## Combined target and document ownership

The target removes repeated decisions where a library or an existing owner supplies the complete
capability, and retains local policy where it expresses a distinct scientific, effect, ownership
or recovery contract. Stronger part extents do not become a claimed complete allocation bound;
serialized values do not become scientific permission; a task tracker does not seal admission;
process exit does not prove group drain; and replaying inputs does not satisfy an arithmetic rule.

| Companion | Delivered responsibility |
|---|---|
| [57a](57a-result-data-and-prepaid-accounting.md) | Typed sinks, safe IPC streams/files, column-wise selection and shared accounting classification |
| [57b](57b-mathematics-and-structural-preparation.md) | Cubic containment/refinement, checked decomposition and mathematics residual-bound ownership |
| [57c](57c-native-solvers-and-arithmetic.md) | Supplier identity, complete arithmetic inquiry, SUNDIALS move, factor/callback/gate integration |
| [57d](57d-sampling-and-statistical-workflows.md) | Recorded-point design production and explicit retained statistical/solver choices |
| [57e](57e-canonical-client-and-requests.md) | One canonical client/request seam, guarded scripts and the complete conflict-retry owner |
| [57f](57f-execution-lifecycle-and-observation.md) | Attribution, typed lifecycle facts, stop/drain/lease composition and Python/native waits |
| [57g](57g-host-files-and-maintenance.md) | Publication, child supervision, process observation, ledgers and maintenance-output inquiry |
| [57h](57h-registry-identity-and-diagnostics.md) | Nested identity framing, diagnostic authority, registry feasibility and generator hardening |
| [57i](57i-python-and-developer-workflows.md) | Trusted codec preparation, public-type feasibility, generated evidence adapters and capture |
| [57j](57j-qualification-and-publication.md) | Joined acceptance, benefit-selection boundaries, independent reconciliation and enduring publication |
| [57k](57k-durable-control-and-python-boundary.md) | Native-independent public durable client, ingress and consumer migration |
| [57l](57l-database-control-and-synchronization.md) | Transactional commands, synchronization and payload/backend investigations |

Package letters follow the companion; their order is navigation, not an agent allocation or
mandatory serial execution. Shared CBO-001 has several contributing owners but only one
disposition here. CBO-010 has one combined owner in 57e, including retry policy; 57f provides
attribution/stop primitives without redefining request policy. CBO-046 splits templating and
SurrealQL decisions explicitly. CBO-048 retains supervisor/store policy on both sides.

## Operator decisions and rule-change routes

| Item | Decision or current condition | Consequence and route |
|---|---|---|
| U1 — upstream options | Confirmed 2026-10-27: include conditional upstream options | Plan source-backed upstream seams where useful, but released capability is required before adoption; no contact/submission is authorized. Maintainer-owned fork changes remain explicit local integration, not an excuse to assume an unavailable feature. |
| U2 — design reproducibility | Confirmed 2026-10-27: preserve recorded-point replay | D's producer records realized values; provenance is attribution. No seed-portability promise, new StudyRequest version or algorithm holds solely for regeneration. Arithmetic rules remain independent. |
| U3 — developer authority | Confirmed 2026-10-27: keep SurrealDB canonical | I's declaration/adapter improvement proceeds without a new evidence/knowledge store. SQLite/Parquet authority migrations are not selected. Offline/source-only routes retain their present contracts. |
| RC01 — public Python types | Confirmed 2026-10-27: include conditional native-public-type target | I inventories/designs native public families. Dependent migration still requires the ADR/formal design review and full preservation assessment. Preserve constructors, indexed views, handle equality, mutation behavior and stubs unless a specific change is selected. Direction confirmation is not ADR acceptance. |
| RC02 — embedded registry product | Confirmed 2026-10-27: include conditional embedded-product target; exact guard remains unestablished | H first settles the full serialization/compatibility/integrity route. No cutover until complete evidence; amend proposed ADR-0210 before acceptance and retain independent source equivalence. A stale-together digest is not proof. |
| RC03 — supplier identity | Proposed correction under existing numerical identity intent; classification must be recorded before implementation | C determines whether framing effective numerical patches is a bug fix within the current contract or changes that contract. If the latter, obtain explicit operator disposition and ADR/formal review before fixing the dependent target. Identity attribution alone does not remove FMA. |
| RC04 — arithmetic and cubic output | Current FMA-free and original-residual/branch rules retained | No waiver through U2/ISA attribution. Correct rounding is an optional stronger target, not selected by this series. A proposed arithmetic weakening would require a separate explicit decision; none is assumed. |
| RC05 — metadata and persistence | Current bytes/identity/lineage semantics retained | A's shared predicate and G's publication helper preserve existing meaning. A newly proposed encoding/identity change needs explicit disposition and the applicable ADR route before dependent work. |
| RC06 — functional/statistical scope | Retain first-order uncertainty, declared sigma, LM route and advanced-step validity | CBO-034/036 and fix-relax are not silently added. If their concrete consumer triggers fire, use functional/decision routes, including supersession of accepted ADR-0118 item 2 where required. |
| RC07 — optional derive/crate | No new proc-macro crate selected | A06 first classifies the real hand-written residue. A proposed crate addition requires explicit selection, ADR and formal review; ordinary dependency additions/patch moves do not. |
| RC08 — enduring documentation | Owner amendments scheduled, not performed by authoring | Reconcile stale blueprint §19.2 and changed enduring contracts through their decision/design route and revision row at publication. Accepted ADRs remain immutable; source reviews are not authority to edit them in passing. |
| RC09 — canonical-client realization | E01 must classify exact contract impact before the dependent cutover | Preserve ADR-0214's intent and the contracts at blueprint §3.3 and §20/§20.6. A refactor within those contracts and ordinary dependency removal need no ADR. Any actual governed change requires explicit operator disposition and the applicable ADR/formal review; architecture wording follows its amendment route. No blanket rule change is accepted here. |

Unanswered choices remain conditional, not accepted. The create-plan operator-confirmation rule
applies before fixing a rule-changing target; it does not block independent planning or turn
ordinary dependency moves, refactors and bug fixes into approval gates. Proposed ADR statuses
are not changed by this document.

## Dependency structure and useful execution order

The initial high-consequence work is exposed IPC input, cubic root completeness, supplier patch
identity and diagnostic-code truthfulness. It need not wait for the complete client migration,
public-class inventory or optional derive question. Actual execution needs separate authorization.

| Prerequisite supplied | Consumer and readiness boundary |
|---|---|
| A02 admitted message safety and ownership | A03 can migrate cached file readers once the shared mechanism exists; raw-span knowledge alone does not permit deleting complete grants |
| A03/A04/B03/C04 migrated bound owners | CBO-001 closes only after each selected caller footprint and its residual allocations are covered; a shared signature alone is insufficient |
| C01 effective supplier identity | C03 changes SUNDIALS numerics only after that identity is constructed/tested, with original policy classification settled |
| G's shared supervised-child behavior | B's compiler child and capture launcher consume the working park/kill/drain contract; they do not each invent another supervisor |
| E's script/result/request design | Query sites migrate once to the complete client/envelope seam; retry classification, original clocks and acknowledgement semantics move together |
| F's wait/lifecycle behavior | I's public native classes preserve actual wait/drain behavior without a second rewrite; neutral columnar flights receive caller instrumentation rather than runtime imports |
| H's complete embedded-product evidence and RC02 route | A conditional registry cutover starts only after current-type compatibility, integrity, canonical encoding and source equivalence are established |
| I's full public-family/stub assessment and RC01 route | Migrate each supported family with its indexed views/helpers rather than removing wrappers by inventory count |
| C02 complete arithmetic remedy or explicit unresolved limitation | No series-wide FMA-free qualification or full conformance claim while this remains unsettled; unaffected repairs and their narrower evidence may still proceed |
| Selected functional packages and owner migrations | J performs one joined scope-end qualification and publication, preserving original failures and partial evidence |

Independent packages may share files; coordinate those edits without treating shared files as
semantic dependency or waiting for an entire companion when a working prerequisite slice suffices.
Use all ordinary host capacity under the selected environment; preserve scientific pool/timing
and exclusion contracts. No lower machine partitions or serial host policy are introduced.

<a id="finding-dispositions"></a>

## Finding dispositions

This is the sole current scheduled-disposition owner for this series. `planned` means a proposed
correction/integration package, `open` an inquiry or target decision, `deferred` a deliberately
unselected replacement with an observable reopening trigger, and `disproved` a rejected premise.
The latter two do not create implementation work merely to populate a plan. They do not imply
that the corresponding product was newly tested. A source opportunity can survive this plan's
bounded completion. The selected A–C resolutions below consume their actual implementation and
functional evidence; the maintainer deferred full series-wide feature validation to J03.

| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
|---|---|---|---|---|
| CBO-001 | [Complete prepayment](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-001) | open — selected prepayment implemented; broader inquiries remain | [A](57a-result-data-and-prepaid-accounting.md#a05), [B](57b-mathematics-and-structural-preparation.md), [C](57c-native-solvers-and-arithmetic.md) | Selected envelopes/derive inquiry delivered; generic alias-header accounting and tighter complete whole-Jacobian/opaque replacements remain unresolved |
| CBO-002 | [Bounded sinks](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-002) | resolved — selected typed sinks | [A01](57a-result-data-and-prepaid-accounting.md#a01) | Typed overflow/checkpoint/I/O, unchanged splitting and deleted trial path; A/J Outcomes |
| CBO-003 | [Publication](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-003) | planned | [G](57g-host-files-and-maintenance.md) | No replacement, directory durability, resumable staging and legacy alias crash states |
| CBO-004 | [Supervised child](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-004) | open — compiler/capture core tested; remaining G scope | [G](57g-host-files-and-maintenance.md) | Working shared child lifecycle; exact-unit delegation, controller/observer implications before optional containment |
| CBO-005 | [IPC trust boundaries](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-005) | resolved — selected IPC boundaries | [A02/A03](57a-result-data-and-prepaid-accounting.md#a02) | Admitted stream/file representation, retained residual custody and actual import/readback; A/J Outcomes |
| CBO-006 | [Dense indexing](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-006) | resolved — selected dense indexing | [A04](57a-result-data-and-prepaid-accounting.md#a04) | Original-column indexing, window-local selection and canonical readback; A/J Outcomes |
| CBO-007 | [Predicate meaning](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-007) | resolved — predicate; planner premise disproved | [A04](57a-result-data-and-prepaid-accounting.md#a04) | Shared predicate/metadata codec and indexed canonical pruning preserved; A/J Outcomes |
| CBO-008 | [Progress](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-008) | resolved — sink; representation retained | [A01](57a-result-data-and-prepaid-accounting.md#a01) | Typed bounded progress sink adopted; heterogeneous JSON policy retained; A Outcome |
| CBO-009 | [Canonical client](../library_leverage/reviews/canonical-store-transport.md#cbo-009) | planned — decision route conditional | [E](57e-canonical-client-and-requests.md), [K](57k-durable-control-and-python-boundary.md) | Rust canonical/xtask/fixture surface and K public ingress; original receipts, bounded delivery and actual drain |
| CBO-010 | [Retry and scripts](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-010) | planned | [E](57e-canonical-client-and-requests.md) | One migration, typed guarded result slots, settled-effect retry and original pacing/clock |
| CBO-011 | [Shared flights](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-011) | planned — unused wrapper only; flights retained | [F](57f-execution-lifecycle-and-observation.md) | Confirm wrapper consumers absent, preserve producer completion and explicit prepaid authority |
| CBO-012 | [Field-local checks](../library_leverage/reviews/result-data-and-prepaid-accounting.md#cbo-012) | deferred — retain current | [A](57a-result-data-and-prepaid-accounting.md#dependencies-retained-scope-and-qualification) | Reopen CHECK adapter on concrete grammar/version change or failure; no full-plan replacement selected |
| CBO-013 | [Lifecycle facts](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-013) | planned | [F](57f-execution-lifecycle-and-observation.md) | Owner-typed event, independent sinks and population-specific counters without subscriber dependence |
| CBO-014 | [Attribution](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-014) | planned | [F](57f-execution-lifecycle-and-observation.md) | Ordered capture/enter, shared producer links, window-team propagation; no authority/cyclic dependency |
| CBO-015 | [Stop/drain/lease](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-015) | planned | [F](57f-execution-lifecycle-and-observation.md) | Explicit first renewal and deadline/cancellation/drain behavior; no silent cadence normalization |
| CBO-016 | [Notification](../library_leverage/reviews/canonical-store-transport.md#cbo-016) | open — L03 synchronization inquiry; polling retained pending selection | [L03](57l-database-control-and-synchronization.md#package-state), [E](57e-canonical-client-and-requests.md) | Authoritative catch-up, finite subscription ownership and a reasoned retain/adopt decision; notifications grant no claim authority |
| CBO-017 | [Python waits](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-017) | planned | [F](57f-execution-lifecycle-and-observation.md) | Signals, final checks, per-caller stop action, recovery key and owned drain |
| CBO-018 | [Native joins](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-018) | planned — owner consolidation | [F](57f-execution-lifecycle-and-observation.md) | Per-owner join observes TLS/native destruction; pidfd replacement only after settling its exact exit semantics |
| CBO-019 | [Process facts](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-019) | planned — remaining G migrations; reader foundation tested | [G](57g-host-files-and-maintenance.md) | Shared Rust reader preserves boot/namespace/start identity, unknown/malformed state and owner tests |
| CBO-020 | [Primary observation](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-020) | open — revised realization | [G](57g-host-files-and-maintenance.md) | Worker/supervisor distinction, authenticated handoff, group lifetime and conservative fallback |
| CBO-021 | [Symbolica mirrors](../library_leverage/reviews/mathematics-preparation.md#cbo-021) | resolved — selected hold/bound work; mirrors retained | [B](57b-mathematics-and-structural-preparation.md) | Canonical source-coupling/non-RNE hold, shared bounds and exact proof constants; B Outcome |
| CBO-022 | [Reuse lifetimes](../library_leverage/reviews/mathematics-preparation.md#cbo-022) | disproved — consolidation premise | [B](57b-mathematics-and-structural-preparation.md) | Four distinct lifetimes retained; revisit only if actual duplicate owner/invalid reuse is demonstrated |
| CBO-023 | [Decomposition](../library_leverage/reviews/mathematics-preparation.md#cbo-023) | resolved — shared checked decomposition | [B](57b-mathematics-and-structural-preparation.md) | Both consumers migrated, caller stack preserved, admitted root/reconstruction cases pass; B/J Outcomes |
| CBO-024 | [Expression consumers](../library_leverage/reviews/mathematics-preparation.md#cbo-024) | resolved — FBBT soundness; IR premise disproved | [B03](57b-mathematics-and-structural-preparation.md#b03) | Interval constants/exact exponents enclose authored and current numerical values; compiler big-M controls pass; B Outcome |
| CBO-025 | [Exact/cubic operations](../library_leverage/reviews/mathematics-preparation.md#cbo-025) | resolved — selected cubic; LDLᵀ retained | [B01](57b-mathematics-and-structural-preparation.md#b01) | Bounded complete associations and verified finite candidates; debug/release and runtime controls; B/J Outcomes |
| CBO-026 | [Sparse factors](../library_leverage/reviews/native-solver-integration.md#cbo-026) | resolved — selected LU helper/bound | [C](57c-native-solvers-and-arithmetic.md) | Three faer consumers and supplier block bound integrated; opaque diagnostic alternatives remain under CBO-001; C Outcome |
| CBO-027 | [Diffsol](../library_leverage/reviews/native-solver-integration.md#cbo-027) | resolved — local terminal fixes; integration retained | [C](57c-native-solvers-and-arithmetic.md) | Typed causes and owned-stop precedence tested through actual boundaries; C/J Outcomes |
| CBO-028 | [Native supplier moves](../library_leverage/reviews/native-solver-integration.md#cbo-028) | resolved — selected SUNDIALS; Uno trigger retained | [C](57c-native-solvers-and-arithmetic.md) | Consumed-source identity, 7.9.0 rebase, corrected history/restart/storage controls; C/J Outcomes |
| CBO-029 | [NLP translation](../library_leverage/reviews/native-solver-integration.md#cbo-029) | disproved — duplicate translator premise | [C](57c-native-solvers-and-arithmetic.md) | Semantic normalization/presolve and ABI containment remain successive distinct layers |
| CBO-030 | [Native exclusion](../library_leverage/reviews/native-solver-integration.md#cbo-030) | resolved — selected helper; IBEX exception retained | [C](57c-native-solvers-and-arithmetic.md) | PETSc/SCIP/HiGHS acquisition and writer preference tested; distinct IBEX policy retained; C/J Outcomes |
| CBO-031 | [Conic layouts](../library_leverage/reviews/native-solver-integration.md#cbo-031) | disproved — missing hub premise | [C](57c-native-solvers-and-arithmetic.md) | Current hub/spoke mappings and independent ray/refinement checks retained |
| CBO-032 | [Retired occurrence](../library_leverage/reviews/native-solver-integration.md#cbo-032) | disproved — retired; ID not reused | [C](57c-native-solvers-and-arithmetic.md) | Named POUNCE surfaces already used; fix-relax only on a separately selected functional target |
| CBO-033 | [Design producer](../library_leverage/reviews/numerical-workflows.md#cbo-033) | planned — recorded-point route | [D](57d-sampling-and-statistical-workflows.md) | Seeded normalized producer plus local mapping/Grid, actual overlays/identity and admitted workspace |
| CBO-034 | [Uncertainty methods](../library_leverage/reviews/numerical-workflows.md#cbo-034) | deferred — first-order contract retained | [D](57d-sampling-and-statistical-workflows.md) | Reopen nonlinear methods on a selected consumer and functional review; no FMA waiver through replay |
| CBO-035 | [Least-squares algebra](../library_leverage/reviews/numerical-workflows.md#cbo-035) | deferred — LM retained | [D](57d-sampling-and-statistical-workflows.md) | Reopen at release, upstream feature fix or selected bounded-fit capability; no false family-removal benefit |
| CBO-036 | [Estimated variance](../library_leverage/reviews/numerical-workflows.md#cbo-036) | deferred — declared sigma retained | [D](57d-sampling-and-statistical-workflows.md) | Consumer/statistical specification and ADR-0118 route before an estimated-scale method; statrs supplies quantiles only |
| CBO-037 | [Authoring parser](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-037) | planned — local hardening; parser retained | [H](57h-registry-identity-and-diagnostics.md) | Exhaustive walkers/generator witnesses; no grammar rewrite or new error-recovery capability |
| CBO-038 | [Source diagnostics](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-038) | planned — presentation; retained-source inquiry | [H](57h-registry-identity-and-diagnostics.md) | Bounded miette rendering where source is owned; preserve native/MCP redaction and structured causes |
| CBO-039 | [Identity frames](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-039) | planned | [H](57h-registry-identity-and-diagnostics.md) | Nested Claim/Permission decisions exhaustive, curated byte layout unchanged |
| CBO-040 | [Registry products](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-040) | open — feasibility before conditional cutover | [H](57h-registry-identity-and-diagnostics.md) | Independent current-type compatibility, full-input decode, canonical encoding, integrity and source equivalence |
| CBO-041 | [Trusted codec](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-041) | planned | [I](57i-python-and-developer-workflows.md) | Immediate native output only, cached handles plus per-thread decoders, public untrusted admission unchanged |
| CBO-042 | [Directional schemas](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-042) | deferred — custom strict envelopes retained | [H](57h-registry-identity-and-diagnostics.md) | Reopen on a demonstrated strict flatten/naming supplier capability, not an empty search |
| CBO-043 | [Python model families](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-043) | deferred — distinct contracts retained | [I](57i-python-and-developer-workflows.md) | A replacement must preserve construction validation, equality keys and strict documents; native attrs support alone is insufficient |
| CBO-044 | [Public native classes](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-044) | open — confirmed conditional direction; inventory pending | [I](57i-python-and-developer-workflows.md) | Full family/indexed-view/stub contract and ADR/formal review under confirmed RC01 before migration; no blanket wrapper deletion |
| CBO-045 | [Diagnostic authority](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-045) | planned | [H](57h-registry-identity-and-diagnostics.md) | One macro authority; exception/report codes and meaning agree at real consumers |
| CBO-046 | [Generated text](../library_leverage/reviews/declarations-registry-and-python-boundary.md#cbo-046) | planned — qualifier; SurrealQL retained | [H](57h-registry-identity-and-diagnostics.md), [E](57e-canonical-client-and-requests.md) | Parameterized import/qualifier context, no template engine; native SurrealQL parser/catalog stays |
| CBO-047 | [Host ledgers](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-047) | planned | [G](57g-host-files-and-maintenance.md) | Stdlib cold start, shared flock/clock/private files and per-owner policies preserved |
| CBO-048 | [Manager/store supervision](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-048) | deferred — current composition retained | [G](57g-host-files-and-maintenance.md), [E](57e-canonical-client-and-requests.md) | Reopen only on demonstrated missing manager/store capability or recovery defect; physical backup and readiness not replaced by logical export |
| CBO-049 | [Test tool composition](../library_leverage/reviews/developer-tooling.md#cbo-049) | deferred — selected catalogue retained | [I](57i-python-and-developer-workflows.md) | Reopen only when exact tool overlay semantics can express repository-owned groups/coverage policy |
| CBO-050 | [Evidence declarations](../library_leverage/reviews/developer-tooling.md#cbo-050) | planned — canonical store retained | [I](57i-python-and-developer-workflows.md) | Raw nested validation, finite/domain rules, Struct-to-publication shape and generated choices at actual consumers |
| CBO-051 | [Capture peer](../library_leverage/reviews/developer-tooling.md#cbo-051) | planned — sink remedy; raw peer retained | [I](57i-python-and-developer-workflows.md) | HTTP/2 ownership and explicit incomplete reporting satisfy current acceptance consumers; front-matter owners unchanged |
| CBO-052 | [Command surfaces](../library_leverage/reviews/developer-tooling.md#cbo-052) | disproved — blanket consolidation premise | [I](57i-python-and-developer-workflows.md) | Keep coherent MCP/CLI/recipe layers; only actual request declaration duplication joins I's work |
| CBO-053 | [Maintenance output](../library_leverage/reviews/host-processes-and-durable-files.md#cbo-053) | open — demand/lifetime inquiry | [G](57g-host-files-and-maintenance.md) | Full catalogs, nonce ack and copy logs differ; select bounded ownership without arbitrary restore timeout |
| FMA | [Arithmetic conflict](../library_leverage/reviews/native-solver-integration.md#faer-kernels-and-the-fma-free-premise) | resolved — consumed faer realization | [C02](57c-native-solvers-and-arithmetic.md#c02) | Maintained nonfused source/dispatch closure, optimized-IR probe and actual LU/SVD/EVD consumers; C/J Outcomes; no cross-CPU measurement |
| Boundary R01 / DA-046 | [Review S01–S06](../design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md#r01) | scheduled | [K](57k-durable-control-and-python-boundary.md), [L](57l-database-control-and-synchronization.md) | Native-independent durable client, actual migrated consumers, authorization and joined acceptance |
| Boundary R02 / DA-047 | [Review attribution](../design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md#r02) | scheduled | [F07](57f-execution-lifecycle-and-observation.md#f07--settle-lifecycle-attribution-and-migrate-promised-boundaries) | Explicit lifecycle promise and destructor/restoration controls, then actual affected trace consumers |

## Investigation outcomes and non-selected alternatives

Open inquiries are work with a decision product, not invitations to an unlimited campaign.
Their owner records inspected conditions, evidence, competing outcomes and consequences for
dependent packages. Required feasibility includes complete IPC residual envelopes, real
hand-accounting shapes, cubic stopping/storage/conversion, no-FMA closure, native factor bounds,
registry serialization/guard closure, public Python family/stub behavior and maintenance output.
Do not substitute approximate symbol counts, one-host probes or absence of a library search hit
for those conclusions.

The comparison's measured questions do not all become performance obligations. Exact IPC sizing
is revisited only if hard-bound fallback frequency makes an extra encode traversal worthwhile.
Notification/pidfd/manager replacements require a concrete consumer burden and preservation
evidence. Uno's initialization seed remains until a real rebased QP control removes its premise.
Optional nonlinear uncertainty, estimated variance, fix-relax and multi-error parsing keep their
functional/decision triggers; they are captured, not selected implementation. A deferred decision
that enters the repository decision process uses the existing ADR register and observable
trigger, not a new parallel register in this series.

## Verification, completion and next work

**Proposed verification:** each package compiles what it changes and runs targeted controls with
explicit Arrow force-validation where applicable. Deletion accompanies the replacement and its
migrated consumers; do not port tests of a removed mechanism while retaining obsolete fixtures
as evidence. Preserve tests of surviving domain/ownership contracts independently of their old
implementation. `just turn-end` belongs to the root at changed-turn end; static hygiene and
assembled integration/solver/Python/manual checks belong once at scope end under 57j.

The main decision and inquiry routes must be settled before their dependent implementation, but
independent local corrections can acquire their own truthful targeted evidence. A bounded inquiry
can conclude that a proposed replacement is unsuitable; reconcile scope and consequences here
instead of fabricating a winning option. An unresolved required MUST/conformance gap cannot be
hidden by completing its investigation: narrow the supported claim or obtain the required
decision and finish the remedy before declaring the corresponding full target complete.

Series completion requires the selected producer/consumer migrations, needed inquiries and
decisions, replacement deletions, joined acceptance, independent reconciliation and enduring
publication described by J. Documentation creation alone completes none of those packages.
Benefits remain Proposed until their selected measurement supports them; optional measurement
does not silently block delivery. Retire resolved plan/review material only after its surviving
meaning and readers move to the established owners, with Git as history.

## Selected A–C execution — 2026-10-27

The maintainer authorized execution of 57a–57c, the minimum working G03 prerequisite for B03,
and selected-scope joined qualification through 57j. The G03 slice includes process facts and
shared supervision with compiler and capture/review consumers; the other G03 migrations remain
with 57g. This does not authorize optional upstream contact or unrelated supplier rebases.

The selected cubic policy tries the qualified midpoint image, then lower and upper endpoint
images, with at most one representative per exact-root association. Equal float bits never
merge distinct root associations. Original residual/branch acceptance remains authoritative;
correct rounding of algebraic roots is not selected. MPFR directed conversion and deterministic
work/storage admission are implementation prerequisites.

RC03 is classified as a missing-input correction under the existing build-identity contract:
`BackendExecution::build` already requires facts beyond the package pin when that pin does not
fully determine the realization. Keep `NativeBuildV1`; any actual change of hashing meaning
still follows the decision route. The first C02 candidate uses maintained faer/traits nonfused
scalar arithmetic and generic matmul while preserving caller-selected parallelism; it is not
accepted as complete until the consumed dispatch closure and scientific controls establish it.

**Current selected outcome:** A01–A06, B01–B03 and C01–C07 are complete for their selected
implementation, inquiry/retention and functional acceptance scope at product source `2bd80093b`.
The minimum G03 compiler/capture prerequisite is delivered; other G03 consumers and D–I remain
with their owners. Current composed acceptance is **153 Rust case executions** across canonical,
native, IDAS-only-backend and compiled-evaluator modes, plus **five Python cases** with composed
passes. [57j's Outcome](57j-qualification-and-publication.md#selected-a-c-outcome--2026-10-27)
owns exact commands, actual modes, original failures, immutable artifacts, independent review,
repaired checks and original-owner settlement. The maintainer-fork source is published and pinned
at the remotely verified immutable revision after explicit approval.

On 2026-10-27 the maintainer canceled the remaining feature campaign and deferred full matrix/
no-default validation to the end of this plan series. Its retained prefix is **179 passed,
1 interrupted, 321 not_run** of **501** configurations; the no-default sweep did not start.
The original interruption/wrapper outcomes remain unchanged, and only the exact emptied campaign
allocation was released through its owner. No background campaign or owned test service remains.
This deferral is not a full feature qualification claim or whole-Plan57 completion.

Enduring contracts are published through blueprint revision 298, with the selected qualification
and timing amendment recorded at revision 299. Broader CBO-001 alias-header/tighter-bound inquiries,
retained source/API couplings and conditional upstream/supplier choices remain explicit.
Performance and cross-CPU benefits are unmeasured. A–C leave active current-work selection but
remain for active readers and shared inquiry context; the coordinator and J own remaining work.

## Selected D–F execution — 2026-10-28

The maintainer authorized D01, E01–E04 and F01–F06, their required shared prerequisites,
targeted acceptance and selected scope-end integration/publication. Execution starts from
clean `6d13fc986`; A–C retains its recorded evidence. The full feature matrix/no-default
sweep remains deferred to series end. Other G–I work, optional statistical methods,
upstream contact and formal ADR status changes remain outside this selection.

RC09 is classified as a contract-preserving client realization: generated protocol/types
retain the existing gRPC, response, clock, restoration, receipt and drain commitments.
ADR-0214’s module SDK/ABI clause retains its Surrealism meaning. A discovered governed
contract change must return to the coordinator before dependent implementation.

D01 returns raw PointOverlay assignments, not already-admitted StudyPointDefinition
records. Existing overlay validation does not establish numeric domain/bounds membership;
the producer checks those through compiled domain owners before generation.

## First control slice execution — 2026-10-28

K01/K02/L01 and browse/cancel/reconcile portions of L02/K03/K04 are authorized, with
selected J acceptance. Async-first and import independence within the existing distribution
are operator-selected; separate installation and a synchronous facade remain unselected.
ADR-0232 and its independent scoped target review accept the Proposed architecture, not
implemented behavior. The original refreshed audit remains applicable to this engagement.

The selected adapter uses HTTPX2 2.16.0 and AnyIO 4.15.1, promoted from quality-only to runtime
dependencies with unchanged versions/resolution. Server actors and immutable cancellation
receipts must cover direct-function authorization, store/actor/operation tuple identity,
retirement/non-reuse and absence uncertainty. Canonical state idempotence alone is insufficient.
The decision commit is `1535f9381`; generated declarations, implementation, consumer deletion
and actual native-loading-refused/server/worker acceptance are still in progress. No broader
qualification or measured speed/capacity claim follows.
