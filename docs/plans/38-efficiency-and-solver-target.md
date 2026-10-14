---
title: Efficiency and the common solver target
status: in-progress
date: 2026-10-14
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md
  - docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md
  - docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md
scenario_sources:
  - docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md
doc_owner: docs/plans/38-efficiency-and-solver-target.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Plan 38: Efficiency and the common solver target

## Scope, basis and current boundary

This standalone series adopts the reconciled conclusions of the [independent audit](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md), its [supporting inquiries](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/README.md), the corrected [consolidated review](../design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md) and [solver companion](../design_review/evidence/efficiency-and-solver-target-2026-10-12/solver-target.md). Audit corrections govern disagreements. Supporting opportunities receive explicit routes; illustrative mechanisms and unused libraries are not automatically implementation requirements.

The target is the best-in-class simulator of the profile and blueprint §0/§0.2: value edit → re-solve without structural rebuilding; sweeps, sensitivities, optimization and fitting; recycles/tears; dynamics from the same definitions; unit/property/solver extension; and compilation/testing with actual dependencies. Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 guide choices. Current ADRs, policies and pins describe the subject and amendment routes.

The original authoring request authorized plan documents only; its targets/checks were **Proposed**. Subsequent packet execution requests authorize the implementation recorded in the checkpoints and companion outcomes below. The [rule recommendations](#rule-decisions) have explicit operator acceptance; their conditional choices remain bounded by the recorded decisions. This series supersedes remaining Plan 36 execution; it does not resolve its findings or turn stopped/failed acceptance into a pass.

The original authoring boundary was eight new documents plus supersession notices/metadata in all six Plan 36 documents. Execution installs its governing amendments through their decision routes. Reconciliation of other active plans remains a separately requested step after this series is fully implemented; companion completion does not authorize their resumption or qualification.

Baseline: `6c368c88acda548d421b44e8fc4f457c95bcec7e`, inspected 2026-10-13, with the source review/audit/evidence and their existing index edit uncommitted. No product commit changed the audit baseline. Source/manifests are **Interface-checked**; no product builds, probes, tests, services or measurements ran during authoring. Historical implementation and evidence retain their original conditions.

## Reading route and ownership

| Document | Owned design and packet state |
|---|---|
| [38a: Authoring, preparation and validity](38a-authoring-preparation-and-validity.md) | Local admitted authority, edits/lineages, granular pure preparation, structure and immutable validity |
| [38b: Solver policy, observation and diagnostics](38b-solver-policy-observation-and-diagnostics.md) | Common dynamic selection/transition seam, contextual permissions, typed observations, diagnosis/replay/advice |
| [38c: Native methods, reuse and parallelism](38c-native-methods-reuse-and-parallelism.md) | Native compatibility/reset, consumed methods/counters, AU01/AU02 and transitive resources |
| [38d: Studies, recycles, dynamics and fitting](38d-studies-recycles-dynamics-and-fitting.md) | Scientific workflow integration, frontier/seeds, history, block schedules, dynamics/fitting demand |
| [38e: Durable records, results and analytics](38e-durable-records-results-and-analytics.md) | Topology decision, acknowledgement, occurrences/bulk snapshots, query/read/retire/backup |
| [38f: Boundaries, build, test and development](38f-boundaries-build-test-and-development.md) | Python/Arrow/RPC ownership, physical/file cache, installed binaries and actual development dependencies |
| [38g: Qualification and cutover](38g-qualification-and-cutover.md) | Decision installation, assembled/scientific acceptance, consumer retirement and completion |
| This coordinator | Shared choices, rule decisions, adopted finding dispositions and Plan 36 transfer |

Companions own packet progress; this document owns findings/decisions and links evidence rather than copying logs. Architecture remains the contract owner and ADRs the rationale owner. Front matter uses existing metadata vocabulary; new existing-policy bindings/current-work index edits are deferred by the user's boundary.

## Combined design and preserved foundations

An authored definition plus physical/provider/interpretation context governs meaning. A local admitted immutable product carries complete dependencies, structure/form programs, demanded derivatives/outputs and allocation ownership. A numerical binding supplies current values/roles/starts. Execution consumes these products without canonical selection/protection solely to reconfirm unchanged local meaning. Recording/publication are separate consumers with honest acknowledgement contracts.

One pure policy seam chooses the next permitted representation/profile and preparation demand from contextual capabilities, request permissions and typed observations. Class-specific builders/controllers retain scientific meaning; adapters own native option/capability facts; placement leases actual resources. Original-space assessment grants result permission. A common record serves policy, explanation, optional persistence and analytics without introducing a second walker or universal instruction language.

Preserve granular Salsa and owned detached products; cached value rebinding; LocalPlanner/PreparedLocalContract and checked Arrow ownership; existing structural plans/per-block initialization routes; pure strategy operations and aggregate work; independent original/assembled checks; actual POUNCE/FERAL retention; HiGHS native updates/explicit bases; admitted profile chains; quiescent resident/active capacity; and typed partial/failed/unknown outcomes with actual drain before release.

Physical typing, guards/envelopes, conservation, derivative/scaling meaning, branch/start permission and independent references survive every optimization. Pattern equality is not numeric-factor equality; numeric refactor is not policy-state reset. Carry immutable validation only through premise-preserving transforms; foreign data/native imports and new numerical states retain distinct checks. There is no universal zero-allocation, full-core utilization, bitwise replay or solver-superiority requirement.

The recommended default preserves automatic durable visibility, independent writers and strong admitted-reader lifetime while computation becomes local. Final persistence topology is a bounded explicit choice, not an assumed server deletion. Journals are not universal prerequisites. Existing data need not be migrated for this design-phase pivot; a selected cutover may regenerate disposable data/evidence, while meaningful current borrowers/operations require authentic disposition.

## Rule decisions

**The operator accepted all 27 recommendations as written on 2026-10-13.** Conditional investigation approval does not select its unestablished implementation premise. RC and S-RC refer to the [audit reconciliation](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#rule-impact-reconciliation). ARC01 is audit-local; R38-01/02 expose old R36-01/02. R36-03 is covered by RC01/03/05. Plan 36's twenty accepted decisions are historical constraints until a confirmed new change supersedes them. G0 installs concrete accepted amendments before dependent code; the table itself changes no rule.

| Item | Recommendation and consequence | Amendment route; consequence if rejected | Decision |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Permit local admitted authority; separate edit ack, attempt recording and visibility. Preserve automatic durable behavior by default; disposable work is explicit. Same-process background append cannot promise immediate-crash survival. | ADR-0164; blueprint §19/§21/§22.2/Python contract. Rejection retains store-dependent authority and only trims within that boundary. | Accepted 2026-10-13 |
| <a id="rc02"></a>RC02 | Complete local dependencies govern pure reuse; independently owned lineage DBs/detached products, durable content ids and byte/trust owners remain explicit. | Blueprint §14.3/§14.4; ADR-0178. Rejection retains persisted eligibility while narrowing redundant local work. | Accepted 2026-10-13 |
| <a id="rc03"></a>RC03 | Permit catalog/manifests plus immutable bulk data after complete publication/read/backup fit. Demanded batch reuse and exact analytics can proceed independently. | Blueprint §20.1/§20.2; ADR-0180; Plan36 RC08/09. Rejection keeps store IPC blocks with suitable provider/conversion improvements. | Accepted 2026-10-13 |
| <a id="rc04"></a>RC04 | Separate facts/coordination; remove only obsolete mechanisms. Preserve strong readers, occurrence ack, succession, shared borrowers and external-file GC; no grace-period weakening. | Blueprint §20.6; ADR-0177/0182; Plan36 RC06/12. Rejection consolidates current protocols without weakening outcomes. | Accepted 2026-10-13 |
| <a id="rc05"></a>RC05 | Compare complete current-publication, same-host transactional catalog and runner/log/remote alternatives. Preserve independent writers/current topology until a concrete choice. Approval selects investigation, not server removal. | ADR-0164/0181; Plan30; blueprint §20.6. Rejection keeps publication topology; local/native improvements remain independent. | Accepted 2026-10-13 |
| <a id="rc06"></a>RC06 | Permit compatible cross-attempt lane/session allocation/structure, with full bound/profile compatibility, terminal invalidation and explicit numeric/policy reset or declared continuation. | Blueprint §13.5/§17.6/native lifetime. Rejection keeps attempt sessions; pure reuse and correction of implicit history remain. | Accepted 2026-10-13 |
| <a id="rc07"></a>RC07 | Expose profile width and transitive resources under one deployment envelope, including children/analytics. Preserve resident/active/queued distinctions and current host partitions. | Blueprint §18.8; ADR-0166; MathPolicy. Rejection uses current placements with truthful limits; no host redesign follows. | Accepted 2026-10-13 |
| <a id="rc08"></a>RC08 | Extend existing compiled validation; retain actual SQL/UDF binding context and remove unchanged crossing work. | Blueprint §14.2/engine role. Rejection keeps engine binding with safe prepared-contract reuse. | Accepted 2026-10-13 |
| <a id="rc09"></a>RC09 | Scope receiving proof by local data, persisted memo and executable class; preserve ABI/environment/generation lifetime for native imports. | ADR-0164 receiving proof. Rejection keeps existing executable admission at the actual import boundary. | Accepted 2026-10-13 |
| <a id="rc10"></a>RC10 | Carry solver impacts only as corrected in individual S-RC rows; original prescriptions are not accepted wholesale. | Solver-review/blueprint §15–§18. Rejected subitems retain their rules; independent accepted items remain usable. | Accepted 2026-10-13 |
| <a id="rc11"></a>RC11 | Retire Plan36 engagement/receiving/generation mechanisms only where obligations disappear; keep complete held products and authentic import/borrower lifetime. | Plan36 RC03–05/governing records. Rejection retains their mechanisms while adapting consumers. | Accepted 2026-10-13 |
| <a id="s-rc01"></a>S-RC01 | Typed causes, physical best merit, screened transfer and explicit branch/start/global/reformulation permission; aggregate clocks/work and terminal vetoes remain. | Solver RC01; blueprint §17/§18. Rejection retains current rules and grants no new successor permission. | Accepted 2026-10-13 |
| <a id="s-rc02"></a>S-RC02 | One governing effective preference/transition seam over contextual assessment; adapters retain capabilities; resource-aware substitutions are visible permitted choices. | Solver RC02; blueprint §18.7. Rejection refines existing selectors without claiming unified policy. | Accepted 2026-10-13 |
| <a id="s-rc03"></a>S-RC03 | Typed consumed counters, absence/method meaning, raw recycle residual and numerical confidence; synchronous policy independent of lossy telemetry. | Solver RC03; blueprint §18.6/§19.2. Rejection limits observation-driven policies to actual evidence. | Accepted 2026-10-13 |
| <a id="s-rc04"></a>S-RC04 | Revisit R-48 for available structured Ipopt statistics/timing through the existing C++ bridge; no inference from text. | Solver RC04/register/bridge. Rejection retains trigger and other truthful counters. | Accepted 2026-10-13 |
| <a id="s-rc05"></a>S-RC05 | Retain supplied no-HSL/R-34 scope; compare nominal/gradient-based scaling and physical solution-level parity. | Solver RC05/parity. No HSL binding is selected; wider scope needs a later decision. | Accepted 2026-10-13 |
| <a id="s-rc06"></a>S-RC06 | Replace cache-dominance R-47 trigger with an actual consumed domain/recoverability/stop capability need. | Solver RC06/register. Rejection retains old trigger and qualified alternatives. | Accepted 2026-10-13 |
| <a id="s-rc07"></a>S-RC07 | Truthful recycle history and contextual per-block profiles; acceleration/restarts need repeatable-map qualification, with no universal Anderson default. | Solver RC07; blueprint §17/§18. Rejection retains method policy while PS-08 history remains explicit. | Accepted 2026-10-13 |
| <a id="s-rc08"></a>S-RC08 | Record effective advice/settings/start and branch/budget permission; distinguish replay/rerun/continuation; baseline policy works without history. | Solver RC08; blueprint §16.5/§19.3. Rejection keeps history out of result-affecting selection. | Accepted 2026-10-13 |
| <a id="s-rc09"></a>S-RC09 | Compact consumed attempt/phase product evidence with overlap/censoring/absence semantics; no full accounting/per-event journal requirement. | Solver RC09/result contract. Rejection restricts dependent comparisons/advice. | Accepted 2026-10-13 |
| <a id="s-rc10"></a>S-RC10 | Correct existing automatic boundary/catalog diagnostic scope; new numerical analysis remains bounded separately. | Solver RC10; blueprint §15. Existing diagnostic behavior remains. | Accepted 2026-10-13 |
| <a id="s-rc11"></a>S-RC11 | Correct capability claims: real native updates/bases/FERAL retention; actual analysis, stop, inertia and pool semantics. | Solver RC11; blueprint §18.9/strings. Rejection cannot authorize broader unsupported claims. | Accepted 2026-10-13 |
| <a id="s-rc12"></a>S-RC12 | Method-specific PETSc expansion under complete class/domain/derivative/stop/resource fit. MPIUNI child processes are independent solves. | Solver RC12/binding/admission. Rejection retains current methods and compares fitting alternatives. | Accepted 2026-10-13 |
| <a id="s-rc13"></a>S-RC13 | Optional bounded effort-aware StepControl with comparable counters, fallback, branch preservation and no budget reset. | Solver RC13; blueprint §17.5. Rejection retains current step rules. | Accepted 2026-10-13 |
| <a id="arc01"></a>ARC01 | Result-neutral reuse covers value-sensitive scaling/ordering/rescue latches: reset, declared history or fresh backend when necessary. | Audit AU02; blueprint §17.6/RC06. Rejection leaves neutrality unestablished on uncovered profiles. | Accepted 2026-10-13 |
| <a id="r38-01"></a>R38-01 | Investigate separate sparse-rank tier with its own convention; retain SVD semantics. Approval selects R36-01's decision route, not SPQR fit or weakened thresholds. | New binding/relation route; blueprint §15. Rejection retains explicit bounded dense-SVD support. | Accepted 2026-10-13 |
| <a id="r38-02"></a>R38-02 | Remove unused public hashing_may_use_pool/projections/tests after confirming no real consumer, preserving hash bytes. Separate from BLAKE3 rayon pruning. | Python-boundary decision/registry/codegen; R36-02. Rejection or a real consumer retains the setting with truthful meaning. | Accepted 2026-10-13 |

Final storage library/topology, remote/surviving-worker support changes, reader/visibility weakening or wider method/branch permission beyond these fences require a concrete subsequent decision. Routine compatible dependency changes follow pinned-dependency judgment; crate/family/contract changes follow the existing decision route.

## Contracts, prerequisites and priority

| Contract delivered | Producer → consumers | Dependency reason |
|---|---|---|
| Local admitted bundle, exact lineage/context | A1–A4 → B1/D1/F1 | Complete semantic ownership removes canonical selection; skipping connect is insufficient. |
| Effective request/profile/permission/observation | B1–B3 with C1 → C/D | Prevents a second Auto policy and ambiguous successor permission. |
| Compatibility/reset and actual resources | C1/C2/C4 → D1/D3–D5 | Numeric refactor does not reset policy; independence and capacity must be real. |
| Ack, exact snapshot and reader/retention | E1/E2 → E3–E5 | Required before topology/format replacement, not before native updates. |
| Compact attempts and branch/start dependence | B3/D1/D2 → B4/B5/E4 | Supports diagnosis/replay/advice without mandatory per-event persistence. |
| Actual selected dependencies/imported binary | F2/F3 → G2/G3 | Runtime independence does not establish compilation or extension identity. |

Correct truthful outcomes, hard-stop handling and recycle history first. Local bundles, policy refactoring, native bulk updates, demand-shaped preparation and boundary ownership proceed as their contracts become available. Native/workflow parallelism needs actual dependencies/resources; durable topology needs its decisions; advisory history needs trustworthy records/permissions. No whole companion or journal is a universal barrier. Priority follows consequence, not benchmark rankings; lower-priority contracts can enable urgent consumers.

Future shared edit surfaces include Runtime construction, strategy/routing, result schemas, native/sys bridges, manifests, recipes/runners, registry/generators and Python generated contracts. Execution coordination owns integration. Migrate producers and all affected consumers together, regenerate from declarations and delete replaced mechanisms/callers/tests as soon as targeted controls pass. No compatibility APIs or second production path are selected.

## Finding dispositions

This is the single current owner of adopted findings. **Scheduled** means a Proposed route; decisions/qualification still constrain closure. Scenarios refer to [audit S01–S08](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#scope-evidence-and-distinguishing-scenarios). G4 reconciles combined evidence, not document authoring.

| Finding | Scenarios | Disposition | Work/decision owner | Settling distinction |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f01) | S01/S07 | Scheduled | A1–A4/E2; RC01/02 | Complete local compute versus acknowledged durable effects. |
| [F02](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f02) | S05 | Open | E1–E5; RC04/05 | Actual occurrence/coordination, readers/GC/unknown/backup. |
| [F03](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f03) | S05/S08 | Open | I-E1/E1; RC05 | Writer/worker premise and complete alternatives. |
| [F04](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f04) | S02/S05 | Scheduled | A5/E3/E4/F1 | Demanded checked batches and exact bounded snapshots. |
| [F05](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f05) | S01/S02 | Scheduled | A2–A4; RC02/09/11 | Pure dependencies versus byte/coalescing/effect/trust owners. |
| [F06](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f06) | S01/S02/S08 | Scheduled | A3/C4/D3 | Existing structural routes and true execution independence. |
| [F07](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f07) | S01/S02/S04 | Scheduled | C1/C2/D1/D4/D5 | Actual compatibility, reset, start and terminal ownership. |
| [F08](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f08) | S08 | Scheduled | C4/D1/D3/D5/F3 | Transitive profiles and composed envelope, not case-only claim. |
| [F09](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f09) | S02/S03/S06/S07 | Scheduled | B1/B2/D1–D5 | One consumed seam with contextual assessment/permissions. |
| [F10](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f10) | S03 | Scheduled | D2/B3/C3 | Raw map history, repeatability and qualified acceleration. |
| [F11](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f11) | S03/S06 | Scheduled | B3/C1/C3/G1 | Real optional counters/capabilities and reason-specific stop. |
| [F12](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f12) | S03/S04/S05 | Scheduled | B4/E4 | Bounded diagnosis and actual replay level. |
| [F13](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f13) | S03/S06 | Open | I-C2/C3/D2–D4 | Every candidate gets consumed selection or explicit trigger. |
| [F14](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#f14) | S01/S07 | Scheduled | A4/A5/F1/F2 | Existing compiled validation and actual import/SQL trust. |
| [AU01](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#au01) | S01/S02 | Scheduled | C5 | Bulk/delta equivalence and partial-mutation invalidation. |
| [AU02](../design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md#au02) | S01/S02/S08 | Open | I-C1/C2; ARC01 | Reachable policy state and reset/fresh/declared-history. |

Fresh independent challenge A01–A09 are additional recommendations, not nine proven defects: A01 prerequisites→this sequence/G0; A02 ack/readers→E1/E2/E5; A03 hidden policy→C2; A04 common seam→B1/B2; A05 demand→A3/A5/D4/D5/E3; A06 capacity→C4/D1/F3; A07 diagnosis/replay→B3/B4; A08 actual dependencies→F2/F3; A09 native delta→C5. Companion investigations own their outcomes; no parallel findings ledger is created.

## Plan 36 transfer and supersession

The [Plan 36 stop checkpoint](36-efficiency-alignment.md#maintainer-stop-checkpoint-2026-10-12) and companion checkpoints remain historical baseline/evidence. New owners below receive remaining behavior and acceptance, not a mandate to reimplement delivered mechanisms. In particular lower admission, generator splits, compact layouts and several native/transport paths already exist. Supersession is not completion or a campaign restart.

| Former packet | New owner of remaining obligation |
|---|---|
| P-G0 | G0: concrete confirmed governing changes before dependent code. |
| P-G1 | G1–G4: current-target assembled/scientific acceptance and dispositions. |
| P-A1 | A1: typed parameter/fixed/start binding and provenance. |
| P-A2 | A2/A3: complete reference/namespace/absence/data closure and head arbitration. |
| P-A3 | A3: one complete demand-shaped preparation. |
| P-A4 | A4/E2: full memo/receipt associations, trust and ack. |
| P-A5 | A1/A4/D1: held bundle/engagement allocation/lifetime. |
| P-A6 | A4/C2/F1: authentic receiving/native/process associations. |
| P-A7 | A2/A3: granular lineage/backdating and detached histories. |
| P-B1 | E1/E2/E5: reader/creator/retire/unknown ordering in surviving tier. |
| P-B2 | D1/E2: eligible frontier/refill/recovery/bounded finalization. |
| P-B3 | C4/D1/D5/F1: resident/active/construction/Par/chunk-wake accounting. |
| P-B4 | E3/F1: exact original coordinates, blocks and completeness. |
| P-B5 | E2/E3: actual ack/close/seal and uncertainty. |
| P-B6 | B3/D4/E3: bounded telemetry and horizon role state. |
| P-B7 | E5: authorized collection and late replay/acquisition fences. |
| P-B8 | D1: eligible seeds before cutoff and owned admitted transport. |
| P-C1 | A3/C1: compact callbacks/layout/diagnostics. |
| P-C2 | A3: scalar optimization, actual derivative order/lazy workers. |
| P-C3 | C3/I-C3: cubic degree/branch/memo/IFT and certified fallback. |
| P-C4 | C2/D3: conditional sessions/profiles/rollback/original checks. |
| P-C5 | C6/I-C4: same-point factor/actions, fill and rank meaning. |
| P-C6 | D3: SCC-local rows/global decisions/costs and independent DAG witness. |
| P-C7 | A2/A3: indexed resolution with complete policy/physical/attribution. |
| P-C8 | C4: IBEX quiescent exclusion and charge handoff. |
| P-C9 | D4/D5: compact surrogate/snapshots and shooting achieved order. |
| P-D1 | A5/F2: actual bound validation owner/lower consumers. |
| P-D2 | A5: carried admission and exact lazy-diagnostic failure semantics. |
| P-D3 | F1/E3: bounded decode/integrity/response cancellation/drain. |
| P-D4 | F1/E4: lazy file capacity and truthful inactive inspection. |
| P-D5 | F2/G2: preserve delivered exact JSON round trips. |
| P-D6 | F2: installed interactive/correctness binary provenance. |
| P-D7 | F1/I-F1/I-F2: delayed pulls/SIGINT/drain/cache/gather/export. |
| P-E1 | I-F3/F3/G3: residual product-relevant owner/metadata acceptance; authenticated host/unknown obligations retained, no new broad ledger redesign. |
| P-E2 | F3: product dispatch/launch/cancel/setup; generic cleanup stays with its owner. |
| P-E3 | F2/F3: direct owner tests, exact inventory/effect admission and canonical journey. |
| P-E4 | F2: existing lower/generator/surrogate roots and exact remaining consumers/closures. |
| P-E5 | F3/I-F3: product classes/widths; current host policy retained. |
| P-E6 | F3/I-F2/I-F3: native inputs/invocation inventory/features/provenance/parity and retained alternatives. |

All 39 former packets are accounted for. F10's five capacity parts, F18's full cubic/IFT contract, F20's fill/rank limit, D-N1 entered-pull/drain and E-C01 same-unit rebuild remain explicit. R36-01→R38-01/I-C4; R36-02→R38-02/I-F2; R36-03→RC01/03/05/I-E1/E2 with original operation identity/clock/ownership until settlement.

### Earlier findings and complete opportunity coverage

Names below are qualified `36/Fnn` and `36/Ann`; their old disposition tables are historical under the notices. These are remaining contributions, not claims that all old code is missing.

| Earlier findings | Current remaining disposition/work route |
|---|---|
| 36/F01/F06 | Scheduled: A1/A4/D1 numerical binding, held preparation and current seed/start facts. |
| 36/F02 | Scheduled: A2/A3 reference, namespace/absence and data closure. |
| 36/F03/F04 | Scheduled: A3/A4/E2 complete preparation/receipt/effect. |
| 36/F05 | Scheduled: A4/C2/F1 artifact-class receiving/native lifetime. |
| 36/F07 | Scheduled: A2/A3 granular inputs/backdating and owned historical products. |
| 36/F08/F15 | Open: E1/E2/E5 surviving conflict/retirement/late effects. |
| 36/F09 | Scheduled: D1/E2 frontier/refill/completion. |
| 36/F10 | Scheduled: C4/D1/D5/F1 all five capacity obligations. |
| 36/F11/F12 | Scheduled: E2/E3/F1 locators/complete ack/close/uncertainty. |
| 36/F13/F14 | Scheduled: B3/D4/D5/E3 consumer demand/sensitivity/telemetry. |
| 36/F16/F17 | Scheduled: A3/C1 compact callbacks/demand programs. |
| 36/F18 | Open: C3/I-C3 full guarded-root/branch/IFT qualification. |
| 36/F19 | Scheduled: C2/D3 sessions/numeric refresh/rollback. |
| 36/F20 | Open: C6/I-C4 same-point factors/fill/SVD versus sparse tier. |
| 36/F21 | Scheduled: D3 globally coupled tear decisions and DAG witness. |
| 36/F22 | Scheduled: A2/A3 indexed policy/context lookup. |
| 36/F23 | Scheduled: A5/F1/F2 validation/IPC/build-mode distinctions. |
| 36/F24 | Scheduled: E4/F1 file capacity and actual SQL/physical inventory. |
| 36/F25 | Open: F3/I-F3/G3 product terminal/resource acceptance with scope-limited host remainder. |
| 36/F26/F27 | Scheduled: F2/F3/D5 direct tests/lower roots/surrogate and real dependency closure. |
| 36/F28 | Open: I-F3/F3 retained host policy and named constrained-product question, no unsupported harm claim. |
| 36/F29 | Scheduled acceptance only: F2/G2 preserve delivered float_roundtrip/exact bits. |
| 36/A01/A02 | Scheduled: D1 complete eligible seed discovery and owned transport. |
| 36/A03 | Scheduled: C4 IBEX exclusion/team/quiescent charge. |
| 36/A04/A05 | Scheduled: D4/D5 surrogate advancement/snapshot and shooting derivative demand. |

The [Plan 36 inquiry/probe mapping](36-efficiency-alignment.md#complete-evidence-and-opportunity-coverage) remains source evidence. All ten original inquiry families A/B/C/D/E/F/G/H/L1/L2 and five audit inquiries route through the packages above. The companions additionally account for parsed syntax/request interning, value-dependent class/presolve facts, reverse cursors/factor actions, seed representation/access plans, file/cache/UDF/reflection questions, exact build/provenance/reference setup and optional JIT/SIMD/FeOS. Investigations end with a choice, explicit retained limitation or observable deferral trigger; they do not require an optimization or rerun every historical probe. Other plans/campaigns remain unchanged.

| Consolidated-review supporting source | Reconciled route and limit |
|---|---|
| W-A/W-B obligation ledgers | A/D/E scientific/durability/worker consumers; actual necessary coordination is preserved rather than inferred absent from a local numerical call. |
| W-C provenance/failure archaeology | F3/G3 residual product setup/recovery questions. Selected failures/source counts remain leads, not measured defect rates or promised savings. |
| R1 durable-state survey | E1/E2/E5 compare complete constraints/events/references, transaction/claim, lifecycle, SDK/tooling and file/cutover fit for the selected topology; native primitives do not protect external files. |
| R2 preparation/evaluation survey | A2–A4/C3/C6 demand, pure dependency/lifetime, orderings and evaluator/provider alternatives. BTF equivalence, graph partitioning, persistent evaluators and optimization determinism require actual consumed contracts. |
| R3 observation/analytics survey | B3–B5/E3/E4/F1 typed tracing/metrics, actual DataFusion providers/pushdown, storage and descriptive statistics; no mandatory new telemetry or statistical platform. |
| R4 numerical survey and solver companion | C1–C6/D1–D5 explicit method-family outcomes, corrected exact-release numerical premises and real scientific consumers. |
| R5 capacity/Python survey | C4/D1/D5/F1/F3 actual pools, task cancellation/tracking, owner-thread/interpreter/native resources and bounded process lanes. Library task cancellation alone is not native drain. |
| Clean-sheet C1 and X1/X2 challenges | Complete alternative comparison and preserved counterexamples, reconciled through the audit rather than accepted as authorities. Fresh audit A01–A09 routes are recorded above. |

Source-review upgrade leads are optional bounded library-fit questions, not assertions of current latest versions or automatic upgrades: Salsa 0.28.5/access guards→I-A1/A2; FeOS 0.10.2 scientific/oracle correction→I-C3/D5; FERAL 0.19/reset/width→I-C1; SUNDIALS ≥7.3 acceleration→I-C2/D2; DataFusion 55.2/Arrow 60 and object_store 0.14→F2/E3/E4; statrs 0.19→B5. Reverify exact contracts/releases if selected, preserve one family/force-validation and independent physical/reference scope, and route a family major appropriately. Keeping current pins is an explicit outcome when no consumed improvement justifies a move. No plan completion requires upgrading every lead.

## Verification and completion

**Proposed:** while implementing, compile touched packages/capabilities and run revealing targeted controls, explicitly enabling Arrow force-validation for actual consuming closures. A packet also migrates affected consumers and deletes replaced mechanisms/tests/fixtures. An agreed interface or old focused pass is not an implemented/qualified prerequisite. G1–G3 select relevant assembled Rust/Python/native/scientific journeys plus applicable hygiene/manual checks once all selected functional scope is implemented.

Mandatory distinctions are value versus body/role edit; historical lineage; contextual pending/refused/ready; explicit versus Auto; root/LS/VI/global permission; physical branch/start acceptance; aggregate clocks/terminal vetoes; raw recycle history; bulk/delta/full model equivalence and partial mutation; native heuristic reset; parent/child charge/drain; live continue versus event/state restart; exact result prefix/ack/reader/backup; and actual installed Python/selected Cargo mode. [38g](38g-qualification-and-cutover.md) owns assembled scope and independent oracles.

Quantified gains need named representative baselines/conditions. No latency/RSS/throughput threshold, new accounting framework or zero-allocation criterion is invented. Product diagnostics/measurement capabilities and measurement of their benefit are separate. Successful bodies with failed/interrupted enclosing cleanup are not passes.

Completion requires every adopted contribution/investigation dispositioned, accepted changes installed, all selected consumers migrated, replaced production paths removed and relevant assembled/scientific checks passing against zero failures. Material remaining support limits need explicit decisions; a central rank/branch/recovery gap cannot vanish under a smaller completed package. Other-plan/status/index updates remain a later separately authorized reconciliation after fully executing this series.

## Authoring checkpoint

All eight documents are authored. All 27 rule recommendations were accepted as written on 2026-10-13; concrete conditional choices remain at their owning investigation. The six Plan 36 documents now carry explicit replacement links and the existing inactive lifecycle value; their bodies/checkpoints/evidence are preserved exactly. No other existing document was edited by this authoring change, including the already-dirty design-review index. Native lifecycle/current-work projection maintenance is deferred with other existing-document reconciliation by the user's explicit boundary.

**Interface-checked:** focused current-source/manifests and independent assembled-plan readback support the target boundaries. The independent readback found no consequential authoring blocker; its suggested clarification is incorporated as the early B3 map-sample contract followed by D2 production and later B3 aggregation. This is focused plan advice, not formal subsystem acceptance. **Proposed:** all product changes, investigations and future controls. **Tested/Measured product behavior during authoring:** none.

Documentation-only validation checked eight new files, 39 distinct packet transfers, 27 accepted decision rows and 16 adopted current findings; metadata validation, book/search publication and scoped new-page offline links passed against zero errors. A preservation comparison confirms the old six bodies differ only by inactive/supersession metadata and notices, and excluded existing owners remain unchanged. Root turn-end formatting/indexing is separate from product qualification.

The next executable scope is preparatory investigation/decision installation and the independently ready functional slices described above, subject to separate implementation authorization and plan-execution preparation. Conditional topology, native reset/method and sparse-tier choices have bounded investigations with actual dependents. Plan creation authorizes no product change, topology cutover, history disposal, existing campaign restart or ADR status change.


## Execution checkpoint — Plan 38a, 2026-10-13

The maintainer authorized execution of the authoring/preparation/validity companion. Proposed
ADR-0183 and architecture revision 160 install its confirmed local-authority and actual-scope
boundaries; the independent bounded target review accepted that amendment before dependent
edits. ADR status remains proposed. [38a](38a-authoring-preparation-and-validity.md#execution-checkpoint--2026-10-13)
owns its completed functional implementation, corrections, targeted acceptance and explicit
qualification limits. A1–A5 and I-A1–I-A4 are complete within that companion; assembled
qualification and shared findings remain open.

Local source authority, bounded lineage reuse, pure preparation/engagement, retained scalar
products, actual SQL admission scopes and demanded checked result retention are integrated.
Rust and Python now permit explicit store-free ephemeral composition; required durable effects
remain fail-closed with original-identity settlement. No storage topology, numerical method
policy or native session reuse choice is selected here. Other companions, shared finding
closure and assembled qualification remain open at their existing owners. Existing-plan,
status/index and campaign reconciliation remains deferred until the series is implemented.

## Execution checkpoint — Plan 38b, 2026-10-13

The maintainer authorized B1–B5 and I-B1–I-B3, including the minimum C1/C3 native
observation and D2 raw-map producer slices. Proposed ADR-0184 and architecture revision 161
install the independently reviewed target. [38b](38b-solver-policy-observation-and-diagnostics.md#execution-checkpoint--2026-10-13)
owns implementation acceptance and its exclusions. ADR status remains proposed.
B1–B5 and I-B1–I-B3 are complete within the companion's bounded acceptance; assembled
qualification and wider shared finding obligations remain open.

The shared pure policy now governs contextual routing and the existing workflow walker;
compact native/component evidence, retained-only summaries, immutable scientific replay and
optional complete matched-cohort advice are integrated. Actual native settings/admission,
original scientific assessment, aggregate budgets and nonblocking ordinary progress retain
their existing owners. New numerical methods, native-state reset/reuse, parallel capacity,
scientific controllers, storage/analytics and assembled qualification remain with their
companions. Minimum producer delivery does not close C1/C3/D2 or shared F09–F12; their
remaining obligations and 38g acceptance remain open. Existing-plan/status/index and campaign
reconciliation remains deferred until the series is implemented.

## Execution checkpoint — Plan 38c, 2026-10-13

The maintainer authorized the entirety of the remaining 38c scope after the prior closeout
and testing stop. [38c's Outcome](38c-native-methods-reuse-and-parallelism.md#outcome) now records
completed C1–C6 and I-C1–I-C4 within their targeted acceptance and explicit support decisions;
[its Verification](38c-native-methods-reuse-and-parallelism.md#verification) owns the exact
commands, results and exclusions. The final linked Python correctness extension/API refresh
and settings/local-runtime consumer controls passed. No selected 38c task remains open.
Proposed ADR-0185 and architecture revision 162 carry the enduring contracts; ADR status
remains proposed. The companion is retained for active dependents and qualification readers.

38a/38b retain their completed bounded outcomes. Shared review findings and assembled
qualification remain open at this coordinator and 38g. Remaining scientific/workflow,
storage/analytics and development scope stays with 38d–38f; none is started by this handoff.
The completed native/Rust/Python selections are scoped evidence, not whole-product or
comparative performance qualification.

All prior owned worker changes are integrated on main and their worktrees removed. The
current-work index points here instead of the superseded Plan36. Substantive reconciliation
of other plan contracts/statuses and historical campaigns remains deferred until series
completion. Do not infer acceptance of a historical stopped campaign from this handoff.

## Execution checkpoint — Plan 38d, 2026-10-13

[38d's Outcome](38d-studies-recycles-dynamics-and-fitting.md#outcome) records complete D1–D5
and I-D1–I-D3 within their selected support and targeted acceptance; its
[Verification](38d-studies-recycles-dynamics-and-fitting.md#verification--bounded-implementation-acceptance-2026-10-13)
owns actual commands, partial failed invocations, repaired controls and exclusions. Lazy
selected source/seed admission, effective block profiles and complete ready teams, frozen
recycle context/bounded restarts, live sample ACK/close/joined ownership, demanded retained
fitting workers/nested teams and optional effort adaptation are integrated. Final native,
canonical retained-seed and compiled Python consumer controls passed. ADR-0186 remains
proposed; architecture revision 164 owns enduring supported contracts.

Only the required current E2/E3 occurrence/prefix slices were delivered. Remaining 38e–38f
scope and assembled 38g qualification stay open, as do shared finding obligations at this
coordinator. Compact live output currently delivers physical values and refuses explicit
sensitivities before dispatch; Full retains admitted sensitivity products. Changed fitting
trials retain compiled operations and use fresh native IVPs. No measured method superiority,
new default acceleration, native-history serialization or storage topology was selected.

All worker changes are integrated on main and the three owned worktrees removed. The owned
private test store is stopped/drained; the quiesced primary profile, shared temporary storage
and other owners' work were preserved. No selected task remains in 38d. Substantive updates
to other active plans and historical campaigns remain deferred until series completion.


## Execution checkpoint — Plan 38e, 2026-10-14

[38e's Outcome](38e-durable-records-results-and-analytics.md#outcome-recorded-after-implementation)
records complete E1–E5 and I-E1/I-E2 within the selected same-host Linux process-crash target.
Its [Verification](38e-durable-records-results-and-analytics.md#verification--bounded-implementation-acceptance-2026-10-14)
owns actual commands, repaired failures and exclusions. SurrealDB retains shared arbitration;
sealed private IPC/opaque files, exact physical ACKs, strong future-file tickets, guarded native
query snapshots, independent scientific borrowers, irreversible collection and complete offline
backup/fresh-root restore are integrated. The actual native crash/reopen/backup/restore journey,
protected validation and stale-authority/fresh-creation controls passed. Proposed ADR-0187 and
architecture revisions 165–166 own the enduring selected contracts; formal ADR status is unchanged.

All worker changes are integrated on main and the three owned worktrees removed. Owned private
services are stopped/drained; failed-attempt evidence, the quiesced primary and other owners are
preserved. The explicitly approved stale allocation was archived/removed with its original
terminal outcome unproved. No selected 38e task remains.

[38f](38f-boundaries-build-test-and-development.md) functional boundary/development scope and
[38g](38g-qualification-and-cutover.md) assembled/scientific/Python/performance/hygiene qualification
remain open. General DataFusion file caches remain truthfully inactive; no throughput or power-loss
claim follows from 38e. Shared findings close only through G4's combined evidence. Substantive
updates to other active plans and historical campaigns remain deferred until series completion.

## Execution checkpoint — Plan 38f, 2026-10-14

[38f's investigation outcomes and focused verification](38f-boundaries-build-test-and-development.md#investigation-outcomes--2026-10-14)
own F1–F3 and I-F1–I-F3. The implementation retains exact original reader/response
ownership, consumes only real Runtime settings, reuses one validated invocation's
executable inventory, derives admitted default widths and observes native build
inputs at their actual consumers. Five exact local supplier sources preserve
upstream numerical trees/features and correct input watches/generated writes.
Complete source capture includes those trees under a correspondingly prepaid
bounded envelope. No source family, native capability or authenticated host guard
is removed to make acceptance pass.

Proposed ADR-0188, its independent target review and blueprint revision 167 precede
the public setting removal. The operator explicitly accepted removal of its
accidental equality refusal; actual remaining settings still control shared Runtime
compatibility. ADR status remains proposed pending its decision-PR route.

**Implemented / Tested:** the bounded 38f controls include actual ordinary/linked
Python modes, entered two-file SIGINT and late SDK drain, normal canonical
artifact/progress/result/export journeys, exact physical cache owners, selected
Cargo roots and relevant/irrelevant native-input controls. The companion records
initial failures, repairs and exact evidence limits. These outcomes complete its
focused scope, not whole-series finding dispositions or scientific qualification.

Next is separately scoped [38g](38g-qualification-and-cutover.md) planning/execution:
assembled scientific, parity, performance, hygiene and manual acceptance, final
combined finding closure and cutover. Prior stopped campaigns remain stopped;
other active-plan reconciliation/resumption is not authorized by this checkpoint.
Owned 38f changes are integrated on main, editing worktrees are removed and its
private service is drained/quiesced at handoff. Retained data/evidence and unrelated
allocations keep their original ownership and outcomes. The completed companion
remains available to the active qualification reader rather than being retired early.

## Execution checkpoint — Static remediation, 2026-10-14

The maintainer authorized repair of the current Clippy, Python typing and related static failures.
[38g's scoped static-check checkpoint](38g-qualification-and-cutover.md#static-check-remediation--2026-10-14)
owns the implementation, selected verification, corrected fixture assumptions and qualification
limits. Handwritten consumers now use current contracts; runtime ownership/control-flow repairs
and conservative lifecycle validation are integrated. Worker worktrees are removed.

Combined assembled/scientific/reference/performance qualification and G4 disposition remain
separately scoped in 38g. Existing Plan 28/32 work and stopped campaigns retain their prior status.
