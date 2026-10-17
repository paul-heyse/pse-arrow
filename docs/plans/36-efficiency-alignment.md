---
title: Efficiency alignment across the simulator and development loop
status: abandoned
superseded-by: https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md
date: 2026-10-11
adrs: [ADR-0173, ADR-0174, ADR-0175, ADR-0176, ADR-0177, ADR-0178, ADR-0179, ADR-0180]
review_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
scenario_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
doc_owner: docs/plans/36-efficiency-alignment.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Plan 36: Efficiency alignment

> **Superseded 2026-10-13 by [Plan 38](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md).** Remaining execution and acceptance obligations now belong to the [new series transfer](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md#plan-36-transfer-and-supersession). The inactive `abandoned` lifecycle value records replacement of this execution plan, not completion or loss of its delivered work. All packet statuses, running notes, decisions and checkpoints below are historical; failed/interrupted evidence is unchanged. Do not resume this plan.


This standalone series reconciles the [original review](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md) and [independent audit](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md) into one target and actionable work. It covers all 29 original findings, five new audit findings, twenty rule recommendations, and the inquiries' additional opportunities and unexamined areas. The audit's corrections replace the original diagnoses where they conflict; neither report is edited.

The user accepted all twenty recommendations in this document as written on **2026-10-11**. That selects the stated target boundaries for authoring. **Accepted-target product implementation was separately authorized on 2026-10-11 and is in progress; enclosing acceptance remains open.** Existing plan documents and their index are excluded from review and updates; later reconciliation is reserved until this series is fully implemented. This document owns dispositions for these two reviews within the new series, not the status of any earlier work.

Source baseline: `b01a45d71a5f5ab29991ad193679f5588ac3662f`, inspected 2026-10-11. Concurrent minor validation edits were preserved. The judgments below are **Interface-checked** from static source and exact library contracts; planned remedies and controls are **Proposed**. No new product test or performance campaign ran during authoring. Historical measurements and the audit's single cancellation control remain attributed to their original conditions.

Execution is stopped at the maintainer’s request on **2026-10-12**, after immediate repair, worktree reconciliation and handoff. The [stop checkpoint](#maintainer-stop-checkpoint-2026-10-12) owns the current state and remaining route; earlier running/pending notes below are historical. Plan 36 remains in progress.

## Purpose and reading route

The target is the process simulator in the profile's Functional target and blueprint §0/§0.2: interactive value edit → re-solve without rebuilding structure; sweeps, sensitivities, optimization and fitting; recycles/tears; dynamics from the same definitions; unit/property/solver extension; and compiling/testing one responsibility with its actual dependencies. Core 3.4, Efficient Architecture 1.0 H1–H28 under AP-07/G9, Process Simulator 1.5 and the selected pse-arrow binding judge the design. Current ADRs, policies, plans and pins describe the subject and its amendment routes.

| Document | Contract and current-state owner |
|---|---|
| [36a: Authoring, preparation and validity](36a-authoring-preparation-and-validity.md) | Typed edit intent; source lineage; complete preparation and held engagement; content/receipt/eligibility separation; observation validity |
| [36b: Studies, coordination and results](36b-studies-coordination-and-results.md) | Exact conflict domains; stage admission; ready work; demanded output, close/seal, seed discovery/transport and settled retirement |
| [36c: Mathematical and native execution](36c-mathematical-and-native-execution.md) | Compact trial layouts, derivative demand, cubic validity, conditional sessions, sparse response, tears, IBEX, surrogate/shooting demand |
| [36d: Relational and Python boundaries](36d-relational-and-python-boundaries.md) | Actual validation-owner retention, admission carriage, IPC safety, lazy file capacity, explicit JSON feature and extension provenance |
| [36e: Build, test and development](36e-build-test-and-development.md) | Exact resource metadata, truthful command outcomes, direct owner tests, lower admission/generator dependencies and workload launch policy |
| This coordinator | Combined outcome, rule decisions, cross-document dependencies, finding dispositions and assembled acceptance |

Each companion's Implementation packets table owns its package progress. Its bounded-investigation discussion owns the question, decision criterion and next step. This coordinator links those owners rather than repeating their status. The review/evidence reports remain immutable source observations; architecture sections and ADRs remain the enduring contract/rationale owners.

## Combined design and foundations

An authored revision owns scientific definitions, physical meaning and attribution. A numerical binding changes admitted independent coordinates under explicit parent lineage. A held engagement owns currently protected source selection, exact physical/provider/registry/validation context, immutable preparation and synchronous settled effects; it is not a timeless cache of permission. Each new numerical point still admits changing hints, bounds, providers, starts and seed permissions, then receives independent original-space assessment.

Prepared layouts and mathematical programs describe stable work. Attempt-local evaluators and native sessions own trial state. Retained symbolic structure/allocation does not authorize stale numeric factors or branch selection. Population, resident memory, active jobs, CPU and demanded output have distinct lifetimes. Queues remain finite and cancellable under the original clock; a parent cannot hold permits needed by its independently admitted child. Quiescent library waits can yield compute while memory and drain obligations stay owned.

Canonical transactions own effect permission, exact attempts/generations, cancellation serial order and retention. Conflict registration narrows to actual mutable records, including absent-key creation and both creation/reclaim commit orders. Immutable receipt/content reuse does not replace fresh admission. Normal completion uses actual acknowledged coverage and authoritative close/seal; uncertain effects settle by original identity. Copied immutable bytes and live storage handles have deliberately different retirement behavior.

Publication follows declared compact/full demand. Exact typed original-row locators replace per-field cells; demanded emptiness is acknowledged at close. One horizon remains one attempt with bounded controller/estimator/background role state and coherent final coverage. Ordinary telemetry can report loss without interrupting scientific assessment; terminal scientific authority is retained.

Development ownership follows those same responsibilities. Direct owner tests avoid canonical fixtures only where the assertion does not consume the store. A lower document/physical admission crate lets its genuine consumers compile without MathService/native workflows. Surrogate integration moves into its actual Runtime consumer; full product-schema generation retains real higher dependencies. Exact per-resource metadata and command-local outcome ownership replace unrelated ledger scans and exit sweeps. Existing native/scientific integration remains real.

The source assessment settled the key architectural choices before scheduling implementation:

| Foundation / alternative | Selected design and reason | Condition that reopens it |
|---|---|---|
| Value API versus incremental compiler first | Expose existing typed overlay semantics over an ordinary admitted parent; structural cutoff is separate | A requested edit changes role/shape/physical meaning and belongs to structural admission |
| Declared fixture demand | Extend the existing physical specialization, then complete executable preparation once | A procedure has a genuinely different execution product; do not force all procedures through algebraic solve |
| Full receipt versus body-local premise minimization | Share one complete receipt and separate immutable content/current roots first | Narrowing requires complete positive/absence dependency evidence and demonstrated remaining benefit |
| Source revision frame | Preserve V4, including complete merged logical framing where required | I-A4 demonstrates a useful new frame and obtains a separate approved version/migration |
| Exact cancellation/retention transactions | Read-register exact cancellation; two-way creator/reclaim fences and exact pins | Pinned backend semantics change or emitted-query race controls expose a missing premise |
| Result indexing | One typed exact row locator per scientific scalar row, decoded-row verification | Populated access evidence justifies typed block bounds without changing ordering/selection |
| Cubic and numerical rank | Attempt-local scratch and complete memo validity; guarded real-only certified isolation with exact fallback; retain singular-value rank | Qualification of the guarded root route; a sparse rank/threshold contract needs the separate decision below |
| Validation retention | Explicit bound actual factory/registry/assembly owner; per-query semantic scopes stay distinct | A different snapshot/config/function implementation requires a new owner, not a name hash |
| Metadata backend | Per-record files with exact logical identity and one policy owner | A real indivisible multi-record invariant cannot fail closed/reconcile; revisit SQLite before migration |
| Admission/build topology | One lower `pse-admission` crate, coherent Runtime surrogate integration, actual selected-unit closure | Actual consumed dependencies contradict the proposed boundary; do not copy DTOs or add facades to force it |

These choices add no universal scheduler, cost model, certificate database or cache hierarchy. Existing library mechanisms remain eligible on technical merit. Exact pinned sources and library skills/Context7 were used where contract details mattered; stale capability maps and utilization JSONL were not used.

Credible extensions help judge locality. An ordinary new unit/property adds authored declarations and genuinely new provider behavior, using the same physical admission and outcome checks. A new solver implements its actual class/order/status/resource/drain contract without rewriting persistence. A new physical concept may legitimately require a shared contract change. A new result consumer declares demand and consumes original rows/complete outcome rather than forcing every run to retain its history. A document parser or lease-policy change should compile/test at its actual lower owner.

## Rule decisions

The user accepted **all 20 rows as written on 2026-10-11**, including retained rules and rejected original prescriptions. Each row qualifies the original RC with the audit correction; RC20 belongs to the audit. Acceptance of a conditional option does not establish its technical premise. P-G0 routes the concrete governing amendments before dependent code changes. Outcomes outside the stated scope return for a new decision.

| Rule reference | Recommendation and consequence | Amendment route | Operator decision |
|---|---|---|---|
| RC01 | Add explicit numerical binding, delta ingress and lineage-preserving reload while retaining complete replacement as a bulk operation. A value-only edit preserves structure and records exact parent/overlay lineage; direct bound or role changes remain distinct. | Blueprint §19/§22.2; Python-boundary ADR/review for changed public Python contract | Accepted 2026-10-11 |
| RC02 | Correct §14.4 to distinguish delivered reuse from unsupported across-edit backdating. Permit retained process-local compiler storage where its real dependencies and lifetime support it; no persistent Salsa database is required. | Blueprint §14.4; design decision follows measured/scoped structural-edit need | Accepted 2026-10-11 |
| RC03 | Permit a bounded held consumer/engagement to share preparation, complete selection receipts and settlement. Preserve independent admission at new trust transitions and exact protection, uncertain effects and original clocks; no automatic background or skipped settlement. | ADR-0164 amendment by superseding ADR and blueprint §14.3 | Accepted 2026-10-11 |
| RC04 | Permit retained receiving qualification only under an enforceable unchanged-artifact/configuration context; changed or unknown premises requalify or refuse. Full observation remains the fallback; dirty run context is not cached as timeless provenance, and stat tuples alone are insufficient. | ADR-0164/blueprint §14.3 receiving and outer observation contract | Accepted 2026-10-11 |
| RC05 | Permit retention of verified immutable native-generation content under a real lifetime/mutation owner, while preserving receiver, executable, process, nonce and placement association. First remove duplicate same-operation work; current once-per-operation verification remains until stronger lifetime is established. | Native identity architecture/guide; hashing or boundary change takes ADR/review route | Accepted 2026-10-11 |
| RC06 | Replace broad per-problem mutation exclusion/shared guard writes with exact conflict domains. Preserve both commit orders, absent-row creation/reclaim, cancellation fences, pin renewal, closed-through recovery and uncertain mutation settlement. | Blueprint §14.3/§20.4/§20.6; ADR/review if the governing decision changes | Accepted 2026-10-11 |
| RC07 | Replace immediate active-job refusal with bounded stage admission under original cancellation/deadline. Separate resident session capacity from active compute so nested work cannot deadlock while holding needed permits. | Math policy/conformance and capacity contract | Accepted 2026-10-11 |
| RC08 | Replace per-scalar-cell indexing with exact typed key bounds or row locators; declare empty membership at close. Preserve original decoded-row verification, exact values, ordering and complete coverage. | Blueprint §19.2; relation-family ADR where required | Accepted 2026-10-11 |
| RC09 | Accept an actual accumulated acknowledgment set plus authoritative close/seal as completion evidence, with readback for uncertainty/recovery. Keep acknowledged durability before successful return; distinguish copied snapshots from live handles subject to retirement. | Blueprint §20.6 clarification; no early-return durability change selected | Accepted 2026-10-11 |
| RC10 | Add explicit compact/full study output demand and bounded horizon live history. Retain one run/attempt and coherent close/seal per horizon; preserve controller, estimator, predecessor and scientific consumers. | Study output contract and ADR-0110 supersession if its contract changes | Accepted 2026-10-11 |
| RC11 | Make ordinary sampled/coalesced telemetry noninterfering with scientific execution, recording drops/partiality. Keep terminal scientific outcome authoritative; any strict telemetry policy is explicit, with no automatic second guaranteed stream. | Resolve blueprint §19.2/§20.6 progress-failure contract | Accepted 2026-10-11 |
| RC12 | Permit lifecycle residue collection only when replay, late acquisition and ownership obligations are settled. A deadline, record age or size is never sufficient deletion authority. | Blueprint §20.4 retention contract and relevant decision route | Accepted 2026-10-11 |
| RC13 | Retain DP-19 attempt ownership. Permit exclusive transfer of drained/reset empty scratch with correct charge and handle ownership; no simultaneous sharing or standard waiver. | Clarify local ownership contract if needed; no standard change | Accepted 2026-10-11 |
| RC14 | Retain current public numerical-rank/cutoff evidence for committed implementation. Investigate sparse numerical evidence separately; return with a concrete method/threshold proposal before changing that contract. | No initial rule change; optional blueprint §15.5.1 method decision remains unaccepted | Accepted 2026-10-11 |
| RC15 | Allow an interactive extension without force-validation while correctness closures explicitly use a validated build. Make installed feature provenance and selection truthful; preserve IPC/FFI structural safety. | Build defaults/provenance and proposed ADR-0170 alignment; no correctness waiver | Accepted 2026-10-11 |
| RC16 | Reject the original mandatory commit-or-retire premise: Rust SQL/invariant routes exist. Preserve those roles, correct unused eager allocations selectively, and treat a new Python query facade as an optional separate capability. | No engine-role retirement; only precisely unsupported surface wording may change | Accepted 2026-10-11 |
| RC17 | Permit dependency-driven extraction through the existing crate ADR route, and narrowing/removing workspace-hack's fallback for Cargo that ignores unstable unification if this is necessary to restore actual dependency closures. Preserve the supported pinned toolchain, exact type families and independent validation. | ADR-0122 fallback change or crate ADR/review; crate governance itself remains | Accepted 2026-10-11 |
| RC18 | Use explicit workload classes and widths derived from actual allocation. Keep memory/headroom, timing isolation and finite populations; investigate affinity/quota alternatives before changing fixed host partition policy. | Recipe/admission contract first; any wider host-policy change returns with evidence | Accepted 2026-10-11 |
| RC19 | Complete the existing live/pending/unresolved coordination intent using exact hot-owner access and bounded cleanup/locks. Preserve retained evidence and drain obligations; unrelated cleanup must not overwrite the command's result. | Implementation and agent-environment contract; no retention relaxation | Accepted 2026-10-11 |
| Audit RC20 | Keep the existing source revision identity frame. Capture compositional identity as an optional investigation; any selected new frame requires its own explicit approval and migration design. | No initial change; hashing ADR/review before a future frame change | Accepted 2026-10-11 |


The public unused `hashing_may_use_pool` option is a later investigation in 36d: wiring a concrete consumer or retiring the public setting needs a concrete boundary decision. Optional background/ephemeral publication, sparse rank changes, compositional source identity, JIT/SIMD, FeOS production authority, a Python relational product and host-partition redesign are likewise unselected. Their investigations do not imply approval. Ordinary exact dependency updates remain governed by the current dependency policy; crate topology and public/hash contracts use their existing decision routes.

## Implementation packets

| Packet | Required input | Delivery and acceptance | Status |
|---|---|---|---|
| P-G0 | Accepted per-item choices; concrete contract written in the owning companion | Route superseding/proposed ADRs, blueprint revisions and boundary/schema decisions before dependent implementation. Verify that each adopted amendment says the selected contract and records its preservation constraints. | In progress |
| P-G1 | Relevant local packages implemented, consumers migrated, investigations dispositioned | Assemble the target journeys below, scoped scientific/integration evidence, applicable hygiene/manual checks and final disposition; publish enduring contracts through their owners. | In progress |

P-G0 is a continuing coordinator responsibility, not one all-series approval barrier. A docs-truth correction, explicit JSON feature or direct command route can proceed after its own applicable contract is ready. ADR acceptance does not establish implementation acceptance. During a later authorized implementation, use one integration owner for shared contracts/manifests/registry generation and short bounded executor scopes.

### Dependency slices and useful execution route

Begin with independently ready work: P-D5 explicit float parsing; P-A1 typed value binding under an ordinary parent; P-A3 single declared preparation; P-D1 retained actual validation owner; P-B8 seed completeness attached to the existing authoritative seal; P-E2 direct Python dispatch/unrelated sweep removal; P-E3 direct policy tests; and numerical P-C1/P-C2/P-C3 ownership/memo changes after their local contracts. No performance campaign is needed to justify demonstrated repeated work. Lower priority P-C6 tears/P-C7 policy indexing can proceed independently when useful.

Build P-B1 exact effect protocols and P-B3 stage/resource ownership independently. P-B1's delivered source creation/reclaim slice, plus P-A3/P-A4 products, enables P-A5 held engagement. A1 does not wait for A5, and F10 admission is not gated wholly on F08. P-B2 readiness can use its actual claim protocol before affinity adopts A5. P-C8 needs the B3 quiescent handoff slice, not the entire study scheduler.

P-B4 defines demand/empty coverage and row-locator publication with all current readers. P-B5 can first improve actual acknowledgments for existing output, then integrate B4's changed demand. P-B8's terminal seed witness first uses existing close/seal; B5 must preserve it later. Seed transport needs only the slim completion-read slice. P-B6 horizon streaming consumes the B4/B5 sink and A5 preparation where appropriate; ordinary telemetry can improve independently. P-B7 collection waits for installed replay/creation settlement, not elapsed time.

P-D1 supplies stable native admission ownership to P-D2 and ingress/reflection consumers. P-D3 owned IPC transport joins B4/B5 at immutable payload/digest and fresh-read integrity; it does not redefine receipt framing. P-E4 extraction can design against these settled interfaces but integrates their actual owners and regenerated consumers. P-D5 precedes surrogate/feature pruning so Clarabel removal cannot silently drop a precision feature. P-C9 and E4 coordinate the surrogate module move with compact advancement; neither creates a temporary second production path.

P-C1 compact layouts simplify P-C4 conditional transport. P-C2 actual-order preparation and P-C3 implicit validity can proceed independently. P-C5 factor sharing needs same-point identity and honest fill admission, while rank substitution remains unselected. P-E1 metadata and E2 interruption compose through authenticated terminal outcome; a per-resource finalization needs that frozen terminal authority, not all-or-none visibility of independent dispositions.

Document boundaries do not impose execution barriers. Shared edits still require coordination: root/member Cargo manifests, `justfile`, registry/generator declarations, Runtime constructors/imports, result schema/writer/readers, Python generated contracts, `scripts/test_resources.py`, runners and environment wrapper. Producers and affected consumers migrate in the same coherent package. Regenerate from declarations; do not hand-edit generated outputs. Remove replaced mechanisms/callers/tests once their replacements' targeted controls pass, without compatibility shims or two production paths.

### Priority is consequence, not a measured ranking

High priority covers interactive intent and stable preparation, exact concurrent-study permission, useful result blocks/completion, ordinary telemetry interference and callback work. A01 seed search completeness is a correctness priority independent of size measurements. Demand/retained history matter for long studies/horizons; resource admission matters under saturation. Numerical-factor/rank and cubic algorithm work is route/scale dependent. Tear formulation, policy indexing, cold sort/export and small script invalidations are lower unless evidence changes their consequence. Host quota arithmetic and source repetition do not prove end-to-end dominance.

## Finding dispositions

Every row points to the original or audit source and the audit scenario. **Scheduled** means the corrected obligation has an implementation route; **Open** means a material design/qualification branch remains. None is resolved by authoring. F10 retains five separate obligations; F18 and F20 cannot close from only their easy slices. The owner records any deliberately retained limitation and its justified review outcome before an enclosing finding can close.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f01) | [S01](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A1/P-A5](36a-authoring-preparation-and-validity.md) | Typed parameter/fixed-coordinate/free-start binding; parent/context lineage; role/bound edits distinct. |
| [F02](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f02) | [S02](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A2; I-A2/I-A4](36a-authoring-preparation-and-validity.md) | Changed reference fields plus namespace/absence invalidation; retain V4 and data/fit ownership. |
| [F03](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f03) | [S01](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A3/P-A5](36a-authoring-preparation-and-validity.md) | One physical demand/completion; bounded candidate windows; no fixed two-compilation claim. |
| [F04](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f04) | [S02](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A4/P-A5](36a-authoring-preparation-and-validity.md) | Share full premises and payload; fresh eligible roots and synchronous settlement remain. |
| [F05](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f05) | [S01](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A6; I-A3](36a-authoring-preparation-and-validity.md) | Separate receiving/outer/managed admission; correct scoped cancellation and actual native retirement; no stat-only proof. |
| [F06](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f06) | [S05](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A5; P-B2](36a-authoring-preparation-and-validity.md) | One stable input engagement; changing point/seed facts still admitted independently. |
| [F07](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f07) | [S02](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-A7; I-A1](36a-authoring-preparation-and-validity.md) | Retained granular compiler lane selected by exact-pin mechanism controls; detached historical products and complete namespace/absence dependencies. Product cutoff qualification remains Proposed. |
| [F08](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f08) | [S05](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B1](36b-studies-coordination-and-results.md) | Both commit orders, exact absence/cancel/retirement/renewal; definite-conflict retry distinct from uncertain effect. |
| [F09](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f09) | [S05](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B2](36b-studies-coordination-and-results.md) | Ready hint plus current claim; refill and projected finalization. Original quadratic count remains hypothesis. |
| [F10](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f10) | [S05](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B3; P-C8](36b-studies-coordination-and-results.md) | (a) population/resident/jobs; (b) fit phase handoff; (c) construction/live accounting; (d) admitted Par; (e) chunk admission and zero-byte allowance wake. All five required. |
| [F11](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f11) | [S06](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B4; P-D3](36b-studies-coordination-and-results.md) | Typed original-row locator, useful bounded IPC blocks and close-declared emptiness; no composite key in time bounds. |
| [F12](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f12) | [S06](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B5](36b-studies-coordination-and-results.md) | Build actual accumulator; full authoritative coverage and uncertainty recovery; slim live reads; acknowledged durability retained. |
| [F13](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f13) | [S06](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B4/P-B6; 36c IDAS investigation](36b-studies-coordination-and-results.md) | Explicit full/compact consumer migration and bounded role state including Staged history; required sensitivities retained. |
| [F14](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f14) | [S05](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B6 telemetry](36b-studies-coordination-and-results.md) | Bounded noninterfering ordinary tap, partiality/drop evidence, existing terminal scientific authority; strict mode explicit. |
| [F15](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f15) | [S06](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B7](36b-studies-coordination-and-results.md) | Settled closed grant/replay authority before collection; preserve late-acquisition and lineage obligations. |
| [F16](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f16) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C1](36c-mathematical-and-native-execution.md) | Compact trial maps/order buffers; exact diagnostics and before-consumer checks. Dense Hessian only Second. |
| [F17](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f17) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C2](36c-mathematical-and-native-execution.md) | Scalar optimization once, demand-specific products/lazy workers; public Symbolica API limits preserved. |
| [F18](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f18) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C3](36c-mathematical-and-native-execution.md) | Guarded exact real-root interval route with existing certified fallback selected; endpoint-root counterexample forbids blanket refinement replacement. Complete product branch/IFT qualification still required. |
| [F19](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f19) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C4](36c-mathematical-and-native-execution.md) | Retain compatible local allocation/structure and operation grain; authored starts, numeric refresh, rollback and fresh original checks. |
| [F20](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f20) | [S04](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Open | [P-C5](36c-mathematical-and-native-execution.md) | Share identical normalized matrix/factor; retain rank meaning. Honest fill bound and large-n dense spectrum limitation remain explicit. |
| [F21](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f21) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C6](36c-mathematical-and-native-execution.md) | SCC-local rows with globally coupled decisions/costs, acyclic mandatory edges and usable incumbent witness. |
| [F22](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f22) | [S02](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C7](36c-mathematical-and-native-execution.md) | Index targets/declarations; retain only complete policy/physical/attribution premises. |
| [F23](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f23) | [S09](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-D1/P-D2/P-D3/P-D6](36d-relational-and-python-boundaries.md) | Distinct validation lifetime, admission/diagnostics, IPC safety and build mode obligations; F-P1 was not executed. |
| [F24](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f24) | [S09](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-D4](36d-relational-and-python-boundaries.md) | Keep Rust SQL/invariants/physical inventory; activate file capacity only on actual qualified source. No mandatory new Python product. |
| [F25](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f25) | [S08](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-E1/P-E2](36e-build-test-and-development.md) | Exact hot/cold access and bounded cleanup; preserve authentic terminal/drain and already-retired native records. |
| [F26](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f26) | [S08](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-E3](36e-build-test-and-development.md) | Move only subject/setup mismatches to real direct owners; constructor is pure binding; retain scientific canonical fixtures. |
| [F27](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f27) | [S03](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-E4/P-E2; P-C9; P-D5](36e-build-test-and-development.md) | Lower admission/generator roots, surrogate relocation and actual supported feature closure; no store-free production Runtime. |
| [F28](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f28) | [S08](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Open | [P-E5/P-E6 host investigation](36e-build-test-and-development.md) | Explicit classes/widths scheduled; current partitions retained. Polling and disjoint slots corrected; tuning needs real interference evidence. |
| [F29](../design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f29) | [S08](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Resolved | [P-D5](36d-relational-and-python-boundaries.md) | P-D5 scoped outcome: explicit float_roundtrip ownership, two passing exact-bit controls and solver-independent actual unit graph; no observed unified-binary corruption claimed. |
| [A01](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a01) | [S06](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B8 completeness](36b-studies-coordination-and-results.md) | Filter eligible sealed inventory before limit; older usable seed behind 64 unsealed headers must be found. |
| [A02](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a02) | [S06](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-B8 transport](36b-studies-coordination-and-results.md) | One owned admitted seed across start/prediction; preserve allowed numeric-data/options variation and fresh later permission. |
| [A03](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a03) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C8; P-B3](36c-mathematical-and-native-execution.md) | Quiescent IBEX waits yield active compute; active teams retain charge absent request-local quiescence. |
| [A04](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a04) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C9; P-E4](36c-mathematical-and-native-execution.md) | Compact Advance; explicit charged full snapshots; actual state/RNG/proposals preserved. |
| [A05](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a05) | [S07](../design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#representative-operation-and-change-scenarios) | Scheduled | [P-C9](36c-mathematical-and-native-execution.md) | Requested/achieved shooting order; primal-only demanded trial and valid upgrade; no report-as-checkpoint assumption. |

## Complete evidence and opportunity coverage

The [original evidence index](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/README.md), [operation map](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/operation-map.md), [inquiry briefs](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/inquiry-briefs.md), [coordinator handoff](../design_review/reviews/efficiency-alignment-audit-handoff-2026-10-11.md) and [audit evidence index](../design_review/evidence/efficiency-alignment-audit-2026-10-11/README.md) remain the source record. Handoff doubts are leads; accepted corrections below rest on review/source contracts. Inquiry-local labels are qualified here to avoid confusing original A-01 with audit A01.

| Inquiry / local candidates | Reconciled owner and additional obligations |
|---|---|
| [A: authoring/admission](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/authoring-admission-and-incremental-preparation.md): A-01–A-07 | 36a: A-01→F01/F06; A-02→F02; A-03→F04; A-04/A-06→F03; A-05→F07; A-07→F06/F08. Include record-data closure, text reload, complete-reference changes, fixed membership pages, request interning, parsed syntax/prepare_flow and immutable private Python hooks. |
| [B: mathematics](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/mathematical-compilation-and-evaluation.md): B-01–B-10 | 36c: B-01–04→F16; B-05/06→F18; B-07/08→F17; B-09→F22; B-10→36b F10. Include value-dependent factorable/class analysis, immutable request/reachability indexing, callback accounting, Taylor/guard preservation and optional evaluator persistence/JIT/SIMD/FeOS. |
| [C: single solve](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/single-solve-native-route.md): C-01–C-08 | 36c: C-01→F16; C-02/06→F19; C-03→F20; C-05→F21; C-08 factor reuse investigation. C-04 sensitivity retention→36b F13, C-07 telemetry→F14. Presolve, KLU/FERAL/KKT and IDAS checkpoint limits stay explicit. |
| [D: multi-case](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/multi-case-workflows-and-capacity.md): D-01–D-09 | 36b: D-01/09→F09; D-02→F08; D-03/06/08→F10; D-04→36a F06; D-05/07→F13. Include cheap ephemeral scans, default jobs decision, fitting experiment order/parallelism, sparse response clones, transient objective/gradient and scalar-order covariance limits. |
| [E: publication](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/publication-persistence-and-results.md): E-01–E-07 | 36b: E-01/04→F11; E-02/03 fences→F08; E-03/05 acknowledgments→F12; E-06→36a F05; E-07→F15. Include live completion-only reads, binary-seed version option, read coalescing and deadline-bounded recovery. |
| [F: relational](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/relational-and-columnar-layer.md): F-01–F-06 | 36d: F-01/02/06→F23; F-04→F24; F-05 single-slot physical cache investigation; F-03→36b F10 chunk admission. Include take_reserved kernel selection, actual sorted-source declarations, lazy diagnostics, buffered export, reflection and metadata-preserving wrappers. |
| [G: development](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/build-test-and-development-loop.md): G-01–G-08 | 36e: G-01/08→F25; G-02→F26; G-03/06→F27; G-04→F28; G-05→36a native assurance; G-07→E-C01 bounded invalidation. Include invocation lifetime/interruption, nextest inventory, parity sync, provenance scope and retained artifact ownership. |
| [H: assurance](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/assurance-identity-and-observation.md): H-01–H-05 | 36a: H-01/02/03→F05; H-05 premises→F04 and digest transport→36d; H-04 callbacks→36c F16/events→36b F14. Faster hashes follow repetition removal and explicit framing/mutation support, not mmap/stat assumptions. |
| [L1: numerical libraries](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/numerical-library-utilization.md): L1-01–L1-07 | 36c: L1-01→F20; L1-02→36b F10; L1-03→F17; L1-04→F18; L1-05 factor reuse; L1-06→36a compiler identity/cutoff; L1-07→36e surrogate graph. Preserve pinned public evaluator limits, real native session seams, independent KKT and reference-only FeOS. |
| [L2: data/infrastructure](../design_review/evidence/efficiency-alignment-codebase-2026-10-11/data-and-infrastructure-library-utilization.md): L2-01–L2-08 | L2-01/05→36d F24/features→36e; L2-02→36b F08; L2-03→36d F23; L2-04 SDK lifecycle/upstream thin hook→36b; L2-06 opportunities stay in boundary/library branches; L2-07→36d F29; L2-08→36a F02/I-A2. Preserve actual request-context deadline/cancel/drain, bulk/server operations and Flights semantics. |
| [Audit A](../design_review/evidence/efficiency-alignment-audit-2026-10-11/authoring-preparation.md), [B](../design_review/evidence/efficiency-alignment-audit-2026-10-11/studies-coordination-results.md), [C](../design_review/evidence/efficiency-alignment-audit-2026-10-11/mathematical-native-execution.md), [D](../design_review/evidence/efficiency-alignment-audit-2026-10-11/python-relational-assurance.md), [E](../design_review/evidence/efficiency-alignment-audit-2026-10-11/build-test-development.md) | All corrections are incorporated above. A01/A02→P-B8; A03→P-C8; A04/A05→P-C9. D-N1 remains P-D7's delayed-stream experiment; E-C01 remains P-E6's same-unit build-input investigation. Reverse IDAS/proof transport, current feature provenance and exact resource outcomes are explicit companion obligations. |

A bounded investigation ends with evidence answering its named question, an implementation choice within existing authority or an explicit retained tradeoff/limitation, and the affected finding consequence. It does not require an optimization. Unselected future capabilities remain deferred until their actual demand/contract trigger; no open-ended benchmark program is created. I-A1–I-A5 are defined in 36a; named numerical/boundary/development branches are owned in 36c–36e.

### Probe reconciliation and purpose

Original local probe labels are qualified by inquiry letter. These are routes to evidence, not a mandate to rerun all original probes. Unsafe/inconclusive setups are replaced by controls below. Quantified benefit claims require controlled observations; static structural corrections do not.

| Source probe | Selected use / settling condition and owner |
|---|---|
| A-P1, H-P1 | 36a/P-G1 focused edit journey: startup, cold/warm prepare, consumed value edit, direct binding and structural edit. Count actual preparation/canonical/qualification work under exact mode and cache conditions; do not assume fixed two-basis or 25+2B crossings. |
| A-P2, L2-P3 | I-A2: populated exact acknowledgment/membership/candidate access plans, examined versus returned rows and historical intervals. Keep byte/message limits and original deadline. |
| A-P3 | P-A4 fixture payload/receipt/root counts and one encoding per shared receipt; retained complete premises versus optional body-local proof. |
| H-P2/H-P3, E-P2, G-P3 | P-A6/I-A3 managed startup, multiple Runtime and exact native-generation owners. Old per-pass hash timings remain historical, not assembled startup time. |
| H-P4 | Receiving-context late-load/configuration negative controls under known image state; distinguish costly qualification from replay already outside support. |
| H-P5, B-P1, C-P1 | P-C1 callback sequence/allocation profile only where needed for priority; correctness controls and compact layout remain required. No inherited percentage threshold. |
| B-P2, L1-P4 | P-C2 actual scalar optimization/order construction, cold/rebind and lazy worker counts. Distinguish derivative expansion from optimizer reuse. |
| B-P3, L1-P3 | P-C3 cubic call frequency plus complete near-critical/degree-reduced/branch controls before algorithm selection. Residual-only candidates cannot settle it. |
| B-P4, L1-P5 | Deferred FeOS/PC-SAFT component-growth comparison under matched physical/oracle contract; no production-authority change selected. |
| C-P2 | P-C4 recycle/conditional fixture distinguishes transport/session reuse from branch/start/numeric refresh; preserves independent commits. |
| C-P3 | 36c presolve investigation splits structure/fixed-bound work from start/dual/LICQ facts; retain current path if remaining repeat is immaterial. |
| C-P4 | P-C6 bounded multi-SCC/grouped tear comparison and whole-graph witness; time-limit incumbent semantics retained. |
| L1-P2 | P-C5 sparse-response scale, actual fill/scratch admission and retained dense-spectrum limitation; independent numerical rank/cutoff and perturbed-root controls remain. |
| D-P1 | Pinned optimistic read-registration semantics already settled by audit source. Emitted-query race controls remain mandatory; recheck library premise on backend/release change. |
| D-P2/D-P3, L2-P1 | P-B1/P-B2 owned-store conflicts, examined candidates, protected crossings, retries/wait and continuous refill under skew. No source arithmetic reported as measured count. |
| D-P4, L1-P1 | P-B3/P-C8 explicit admitted faer Par and actual scratch/dispatch mode. Supernodal-use observation can change priority, not CPU ownership requirements. |
| E-P1 | P-B4 exact isolated result-set metadata, block/index/empty counts. No global production-history sweep or numerical campaign needed. |
| E-P3 | I-A2 / 36b access-plan investigation: populated latest_results ORDER BY sequence DESC LIMIT 1 EXPLAIN, examined rows and actual index/limit pushdown. P-B5 normal acknowledgment/recovery controls are a separate acceptance obligation. |
| E-P4 | P-B7 fixture-specific before/after settled residue counts and late-grant race controls; no age-only deletion. |
| F-P1 | P-D1 exact validation-owner and preparation counter controls; original report did not execute this probe. |
| F-P2 | P-B3 register/grow/drop versus chunk-reservation contention and wake counts, including zero-byte spillable departure. Measure only if deciding residual notifier changes; waiter correctness remains mandatory. |
| L2-P2 | P-D4 pool reservation before/after qualified activation. Accounted capacity is not RSS; optional memory observation only if claimed. |
| F-P3 | P-D2 successful-validation structural/diagnostic work versus predicate kernels, paired with exact failure-path equivalence. |
| F-P4 | P-D7 physical-package diversity census and interleaved-source demand before selecting a weighted multi-entry cache. Original distinct-document count is fixture-specific, not a sorted-source proof. |
| F-P5 | P-D6 explicit interactive/validated extension modes and actual feature provenance. Compare same operation/build conditions only if claiming latency. |
| G-P1/G-P2 | P-E1/P-E2 temporary metadata and import/dispatch controls; existing ledger/import figures are historical. No need to reopen production registry to establish new exact access. |
| G-P4 / audit E-C01 | P-E6 isolated same target/host/feature/profile unit, plain→native-env→plain fingerprint sequence. Different unit keys, script execution or unchanged output alone do not establish downstream rebuild. No cache clearing. |
| G-P5 | P-E3 direct-owner versus necessary canonical setup; historical refusal median cannot be subtracted to infer compatible fixture lifecycle cost. Pooling needs a reset/isolation contract first. |
| G-P6 | P-E2 direct Python selection/exit equivalence; no synthetic touch/link measurement required to remove Rust dispatch. Any latency comparison uses equivalent selected work. |
| G-P7 | P-E5/P-E6 host isolation/width investigation under admitted quiet conditions and unchanged artifacts. Quota arithmetic or changed code/profile does not prove benefit. |
| Audit D-N1 | P-D7 delayed-read/SIGINT, confirmed entered-pull, signal-before and ordinary-bridge controls; observe cancellation/settlement/protection and late arrays, not only wall time. |
| Audit A03–A05 | P-C8 quiescent/team admission distinction; P-C9 compact snapshot and shooting requested/achieved-order controls. Priority/default measurements only when their answer changes policy. |

### Limits and further examination

The combined source review is broad but not exhaustive. Preserve these boundaries as planned investigation triggers, not undiscovered defects or silent qualification claims.

| Area | What is known / what remains | Route when relevant |
|---|---|---|
| Parser, quantity inference and authored references | Bounded parser/borrowed tokens and physical inference are strengths; not every construct/ill-posed diagnostic was requalified | P-A2/P-A3 changed-reference/fixture controls; follow actual new syntax/physical contracts through affected consumers |
| Native/proof adapter breadth | Selected lifecycle/IBEX/source contracts inspected; not every ABI/status/error mapping, provider or solver class tested | P-C8 and each touched adapter's exact native modes/status/cancel/drain controls; new solver variation uses its actual class contract |
| Qualified prior leads recorded in original §9.3 | Current Runtime checks actual consumed services; five recovery/readiness controls pass. Current scientific phase/deadline/cache-lifetime source still requires phase-specific acceptance. Native coexistence matrix and IBEX handoff are specified in 36c. | [36e current source/controls](36-investigation-evidence/development.md) and 36c; P-G1 retains actual native/provider/recovery acceptance. No inherited disposition or older plan status is imported |
| Workspace admission/interruption leads in original §9.3 | Current source confirms the nextest selector mismatch: preserve_external_and_deployment_databases versus preserve_deployment_database. Actual control observes deployment existence and needs exclusive store-effect classification. | P-E3 corrects exact selected name and derives effect capacity independently of CPU width; direct owner tests retain real canonical integration. No broader campaign failure is attributed to this mismatch |
| Dynamics, continuation, collocation | Existing pattern/numeric separation, exact preparation stencils and IDAS checkpoint replay can be necessary | P-C9 plus bounded reverse-cursor/presolve/factor investigations; retain second-order jump, window and independent original checks |
| Surrogates, factorable/derived/physical models | New demand defects inspected; no complete scientific/model adequacy campaign | P-C9 consumer migration and relevant independent fixtures; PS-13 scope explicitly named in P-G1 |
| DataFusion and Python breadth | Actual SQL/invariant/physical routes retained; not all optimizer behavior or Python APIs audited | P-D1–D7 actual consumers; support for larger analysis, cross-output query or inbound Arrow needs its own product contract |
| Supervisor, runner and resource lifecycle | Selected admission/metadata paths inspected; enclosing signal ownership and every interruption combination not fully tested | P-A6 and P-E1/P-E2 exact owner/interruption/drain controls before changing lifetime; no numeric PGID as drain proof |
| Code generation and build | Real production admission and feature coupling inspected; broad schema consumers remain legitimate | P-E4 exact selected-unit graphs and full aggregate regeneration; P-E6 distinguishes script execution from actual recompilation |
| Scientific parity and scale | No new physical closure, derivative/rank campaign, teqp/IDAES parity or complete native suite during authoring | P-G1 selected touched science with independent oracle/tolerances; no claim of whole-model or IDAES equivalence |
| Host capacity and end-to-end performance | Isolated historical observations; no new throughput/RSS/latency ranking | Bounded P-E6 or target-journey measurements only when a decision/benefit claim needs them; preserve concurrent work and admission |

## Verification, migration and completion

**Proposed local verification:** compile touched packages and run targeted functional controls as each mechanism changes, with the checkout environment, exact dependency/toolchain pins and explicit force-validation for every actual Arrow-consuming closure. Native tests use their linked environment and authentic operation owner. Generated changes regenerate from their authoritative declarations in the same package. Do not run broad integration/static hygiene mid-implementation or alter unrelated concurrent files to make a check pass.

**Proposed P-G1 assembled acceptance:** after all selected functional scope is implemented, select the relevant Rust/Python/canonical/native integration and scientific journeys. Exercise the audit scenarios S01–S09 with these decisive distinctions:

- Warm typed edit keeps structural preparation and source parent; structural expression change invalidates its actual references and preserves unrelated products.
- Concurrent sweeps/continuation/fitting complete under exact cancellation, stage saturation and lost acknowledgments; wrong-generation effects refuse and issued uncertainty settles.
- Compact/full study and long horizon consumers see coherent original rows, honest empty/partial outcomes, bounded live role state, permitted seeds and correct retired-handle behavior.
- Recycle/conditional, implicit cubic, response and dynamics/shooting preserve original-space/physical/branch/rank/derivative contracts; new numerical states never inherit unchecked validity.
- Unit/property/solver extension consumes the intended owner boundaries; lower document/policy tests and focused generation have actual dependency graphs without forbidden Runtime/native reach.
- Interactive and validated correctness extension modes report the actual imported binary; uncontrolled Arrow input refuses safely, and confirmed stream interruption uses the right cancel/drain owner.
- Resource registration/finalization/reclaim survive both race orders and interruption; unrelated cleanup never erases a workload outcome, and unknown drain never becomes deletion authority.

Use independently specified values/physical expectations or independent reference methods for numerical changes where a round trip could preserve the same defect. Bitwise equivalence applies only when the algorithm/order contract preserves it; derivative-mode integration can change steps and must meet the stated tolerances and original qualification. PS-01–PS-13 and PS-G1–PS-G3 remain applicable to changed scientific scope. Explicitly record excluded models/modes, retained support limits and whether each claim is Proposed, Interface-checked, Implemented, Tested or Measured.

At functional scope end, the root selects applicable `just hygiene`, `just governance`, `just docs`, native/powerset lints and integration/performance campaigns under repository timing. Each executed check reports command, mode, scope, zero-failure baseline and actual result. A failure remains a failure; unrelated failures are reported without expanding authorized remediation. No quantified speedup or universal SLA is required by this plan; a claimed benefit needs a named baseline, unchanged comparable conditions and measured result. One operation's acceptance does not qualify every simulator model.

Migration includes actual consumers, durable representation activation/rebuild and interruption recovery, not just new producers. Plan37 OP01 / ADR-0182 now govern the canonical-store transition: fresh v5 regeneration replaces historical substrate backfill/import and compatibility readers; exact original identities and provenance within current content, uncertain effects and owner-authorized retirement remain protected. Current-version backup/restore is separate. Host resource/invocation metadata cutover still retains known cold evidence and refuses ambiguity. Old writers cannot recreate the superseded format after activation. OP01 authorizes only explicitly selected owned-store regeneration after authentic drain; it grants no unrelated production deletion. Code paths, mechanism-specific tests and generated declarations retire when replaced and all consumers move; retained evidence/documents follow their own owner/retention policy.

Final closure requires all selected packages and mandatory revealing controls, explicit disposition of every investigation and every finding contribution, accepted rule amendments actually installed, no remaining replaced production path, and assembled acceptance with material limitations stated. Optional unselected capabilities may remain deferred; a central unresolved F18/F20 obligation must remain open or receive an explicit supported-scope decision, never disappear under a completed scratch/factor package. Move enduring contracts and rationale through the architecture/ADR route. Updating older plans and their index remains a separately requested later step after implementation; authoring this series does not reconcile them now.

## Authoring outcome

The six standalone documents capture the combined target, selected responsibilities, package prerequisites, consumer migration, preservation constraints, rule decisions and verification/investigation routes. All twenty operator decisions are recorded. Foundation inspection resolved design blockers sufficiently to define the committed routes; remaining conditional alternatives are bounded and have explicit dependents.

**Interface-checked:** review/source reconciliation and focused library-contract research support these designs. An independent assembled-plan review found and corrected a potential IBEX guard/compute-permit cycle, missing registration-versus-terminal-freeze fencing, several probe mappings and minor opportunity coverage gaps; final readback identified no remaining consequential contract blocker or material coverage gap. **Proposed:** every product change and future control. **Tested / Measured during authoring:** none for product behavior or performance. Scoped document checks validated six files, 135 local links, 34 finding rows, 38 package rows and 20 accepted rule rows with zero errors against a zero-error target. The original review hash remained unchanged. These checks and the independent plan review do not qualify implementation.

The next executable step, after separate implementation authorization, is to refresh the touched source baseline and choose an independently ready package above, routing its concrete P-G0 amendment first where required. Do not reinterpret approval of these plan documents or the twenty rule recommendations as authorization to begin product changes.

## Investigation outcome and current checkpoint, 2026-10-11

The user authorized the investigation execution approach, including all listed optional branches and bounded isolated prototypes. Investigation baseline: `2d6bf6ae7`, initially clean. This authorization covered research, probes and refinement of this new series; it did not begin product implementation or enact architecture/ADR amendments. [Investigation evidence](36-investigation-evidence/README.md) links the source-backed decisions and reproducible discriminators. Earlier authoring observations above retain their original scope/date.

The target is now specified at the cross-owner seams: a retained granular compiler lane with detached historical outputs; typed numerical/delta ingress; one complete preparation receipt and protected engagement; exact creator/reclaim/cancel protocols; separate resident/population/active resource leases; explicit demand and acknowledged seal; actual bound validation owner; admitted owned IPC with pre-constructor safety; Python-controlled pull cancellation; exact per-resource metadata under invocation freeze; and lower admission/build dependencies. Each companion owns its selected interfaces, consumers, activation/migration, preservation constraints, acceptance and retained alternatives. Implementation packages remain Scheduled and original findings remain unresolved.

The principal design corrections from investigation are consequential:

- Structural incremental reuse is selected now, following exact Salsa controls, with P-A7 owning the actual migration. Value binding remains independent of compiler cutoffs.
- Real-only certified root isolation is available, but a nearcritical endpoint counterexample requires a guarded route and the existing exact fallback. No residual-only or approximate completeness claim is accepted.
- Exact-key transaction conflicts and actual populated access plans support narrow conflict domains while disproving the assumption that 64 returned membership rows bounds examined history. Seed eligibility must precede LIMIT.
- Root factors and RHS actions can be shared at the same exact numeric point. Faer's private numeric fill growth and the public dense singular-value evidence remain separate constraints; F20 is not closed by factor sharing.
- Arrow force_validate alone does not prevent all malformed-IPC constructor panics, and upstream accepts streams outside the application's EOS/span contract. Preserve pre-decode bounds and remove only demonstrated duplicated checks after ownership migration.
- Terminal projection needs pending invocation membership before resource publication. A current negative control accepts late registration after finalization, safely pinned but not covered by the prior terminal outcome.
- The exact sampled build graphs narrow without changing normal Arrow features; correctness roots retain explicit force_validate. The same-unit build-input spike confirms unnecessary environment invalidation reaches a miniature downstream, without establishing full backend latency.

### Shared contracts and implementation order

| Consumed contract | Producer / integrating consumers | Required before |
|---|---|---|
| BoundModel retains actual factory/registry/assembly, with acyclic native snapshots | P-D1; Runtime/Python ingress, lower admission, reflection and result validation | Admission carriage P-D2; lower generator/source migration P-E4 |
| Complete prepared product + current protected engagement + synchronous settlement | P-A3/A4/A5 and P-B1 exact source/reclaim protocol | Study affinity and repeated point preparation; never a timeless permission cache |
| Retained compiler staged input revision and detached charged products | P-A7 consumes P-A2 source changes and P-A5 lifetime | Structural-edit reuse; no canonical effect inside Salsa queries |
| Resident/population lease distinct from active job/CPU; cancellable try-resume | P-B3 with P-C8; fitting/nested/native routes | Conditional/session retention and IBEX paired admission; no guard held over blocking reacquisition |
| Typed demand, exact original-row locator, empty membership and acknowledged close/seal | P-B4/B5 with P-D3 transport | Horizon role retention P-B6; seed transport reads the existing authoritative seal independently |
| Immutable admitted payload/digest with backing charge, fresh-store verification | P-D3 and canonical source/result writers/readers | Writer/digest/check elimination; sharing cannot weaken reopened storage trust |
| Frozen authenticated terminal and pending invocation membership | P-E1/E2 and existing observer/runner owners | Per-resource projection/cutover and invocation artifact retirement |
| Explicit float_roundtrip, surrogate advance/snapshot contract, registry generated roots | P-D5, P-C9 and P-E4 | Surrogate dependency relocation/feature pruning; no generated hand edits |

Ready implementation slices remain independent as described above. P-G0 routes **concrete amendments before their dependent product changes**, not a new all-series approval barrier. Investigations requiring a target shape used owned prototypes or source contracts; they did not wait for production changes or import older plan status. The remaining whole-product/native/scientific controls belong to implementation acceptance, not another exploration cycle.

### Concrete governing amendments

The original twenty accepted rows remain the authority for selection. This map makes their next drafting work concrete without changing current rules:

| Accepted rows | Amendment content now specified / owner |
|---|---|
| RC01/02 | Public typed binding/delta/reload lineage and retained compiler input/product lifetime; blueprint §19/§22.2/§14.4 and Python-boundary design route. P-A1/A2/A7. |
| RC03/04/05 | Complete shared receipt/content/current roots, synchronous held engagement, scoped receiving/outer/native observation and full fallback at uncontrolled boundaries; ADR-0164 supersession and blueprint §14.3. P-A3–A6. |
| RC06/12 | Exact absence/creation/reclaim/cancellation/renewal/closed-through protocols, original uncertain settlement and settled residue collection; blueprint §14.3/§20.4/§20.6. P-B1/B7. |
| RC07/13 | Separate resource lifetimes, bounded original-clock admission, parent/child handoff, actual nonblocking paired IBEX/compute acquisition and exclusive drained/reset scratch transfer. P-B3/P-C8/P-C2. |
| RC08/09/10/11 | Typed row locators, demand/empty/partial membership, actual acknowledgment set and authoritative seal, coherent one-attempt horizon and noninterfering ordinary telemetry; schema/activation and ADR-0110 route where affected. P-B4–B6. |
| RC15 | Actual binary cfg identity and explicit interactive/correctness selection, preserving Arrow/FFI structural safety and all correctness closures; build/provenance route and ADR-0170 alignment. P-D6. |
| RC17 | Lower admission crate, actual generated/import roots, Runtime surrogate integration, workspace-hack fallback retirement on supported pinned nightly; crate and ADR-0122 routes. P-E4. |
| RC18/19 | Actual allocation-derived launch widths and separate effect capacity, exact metadata/terminal/cleanup outcomes with no retention relaxation; environment/tooling contract route. P-E1/E2/E5. |
| RC14/16/20 | Retain singular-value evidence, real Rust SQL/invariant roles and V4 source identity. No initial rank/hash/product substitution is installed. |

### Choices outside the twenty accepted boundaries

The evidence fully describes alternatives without silently selecting them. **R36-01** is the numerical owner's separately proposed sparse rank/evidence family: it must not masquerade as the retained singular-value contract or silently relax its thresholds. **R36-02** is the boundary owner's concrete proposal to remove the public unused `hashing_may_use_pool` setting and its projections/mechanism tests while preserving hashing bytes. **R36-03** is the preparation owner’s concrete optional publication proposal: `ReusablePublication::{Required,Explicit}`, a prepared-unpublished outcome distinct from persisted reuse, and an explicitly awaited publication operation with no automatic queue. Dispatched effects retain the original operation identity/deadline, complete receipt, protection and allocation through settlement/drain. It changes the public effects contract beyond RC03 and depends on P-A4/P-A5/P-B1. None of these three proposals is approved by the earlier twenty decisions. The companions state the recommendation, effects, preserved behavior and acceptance; product implementation of any of them needs a subsequent explicit selection. Retaining them unselected leaves the committed target truthful; F20 retains its named scale/support constraint.

Optional background/ephemeral publication, a new source identity frame, Python relational/inbound-Arrow product, evaluator persistence/JIT/SIMD, FeOS production authority, used-fixture pooling and host-lane borrowing remain unselected or explicitly retained/deferred with observable reopening triggers. They are not silently added to the implementation backlog. Missing actual product/rank/cancellation qualification stays visible; no prototype is claimed to supply it.

### Verification and limits

**Interface-checked:** current source/consumer tracing, exact library source and reconciled cross-owner contracts. **Tested / Measured:** only the named isolated controls and graph populations in the evidence. Scoped document validation checked six plans, eight evidence documents, 179 local links, 34 finding rows, 39 implementation packets and twenty accepted rule rows with zero errors against a zero-error target. Both original reviews were byte-identical to HEAD. The [independent assembled readback](36-investigation-evidence/readback.md) found no additional consequential blocker after correcting the current-head access-plan consistency; it retained F20’s support blocker. `scripts/pse-env -- just turn-end` passed ADR-index and formatting with exit 0 against the zero-failure baseline; no product qualification is implied. **Proposed:** every product correction, migration and scientific/native acceptance control. The investigation did not run full simulator integration, physical/rank/derivative campaigns, parity, native powersets, production cutover or an end-to-end latency/RSS campaign. Quantified prototype observations retain their actual fixture conditions.

Investigation completion means every listed branch has a selected, retained, separately proposed or explicitly deferred answer and a concrete owning route. It does not complete this implementation series, resolve the original findings, accept governing amendments, or qualify unexercised models. The next step is selection of any desired new public/rank proposals and separately authorized product execution against these contracts. Older plans/index and both original reviews remain unchanged.


## Product execution checkpoint, 2026-10-11

The maintainer selected **Accepted target only** and authorized implementation of all
39 packets. R36-01, R36-02 and R36-03 remain unselected. F20 retains its public sparse
rank/support limitation. This is production execution authorization; it does not change
ADR status, authorize unrelated cleanup or reconcile older plans.

Initial production baseline is `2d6bf6ae7`, with the six plan refinements and investigation
evidence preserved. P-D5 explicitly declares JSON exact float round-trip support; its
finite-bit and known-counterexample controls pass. P-D1 migrates actual retained model
ownership through Runtime, Python, standalone pure admission and generator consumers.
P-B8 adds complete-original seed eligibility bound to the frozen attempt inventory,
with generated source and explicit seal-caller migration. P-E2 removes the Rust Python
dispatch and unrelated scoped-exit sweep; targeted runner controls pass. P-E3 separates
direct deployment/interval checks from canonical integration setup and corrects the exact
exclusive test selector. These are delivered slices, not enclosing packet acceptance.

P-G0 supplies Proposed ADR-0173–0178 with governed owner/revision amendments. Independent
bounded target reviews accepted resource metadata, responsibility-local build roots and
the corrected canonical/preparation contracts. The latter exposed omitted study-association
creators; P-B1 owns their actual both-order retirement controls. No ADR status has changed.
Root alone publishes generated outputs and integrates shared surfaces.

Delivered source includes exact per-resource metadata and authenticated terminal freeze,
preserving migration/refusal fencing, terminal composition and own-release refusal;
lower admission with migrated callers/tests and actual generator capabilities; workspace-hack
retirement with explicit validation/family preservation; checked-batch carriage; lazy
file-cache activation; dense binding and compact applicability evidence; lazy scalar-order
construction and charged dynamic snapshots; Runtime-owned compact surrogate advancement
and explicit snapshots; indexed numerical requirements; physical-core default widths and
demand-specific native build inputs. Scoped controls have passed for these slices; native
and all-consumer acceptance is still open.

P-B1/P-B7 exact canonical conflict and settlement, P-B3 population/resident/active and
quiescent fitting ownership, and P-D6 actual compiled Python flags/correctness builds are
being integrated. The shared private test registry needed a one-file preserving repair
after an in-flight draft activation schema sharpened; production/default metadata was
untouched. New schema publication and native compile/targeted controls precede dependent
engagement/source/compiler adoption. Production cutover/history deletion remain excluded.
Remaining A/B/C/D/E scope and P-G1 are not complete. Targeted checks continue; assembled
native/scientific/Python qualification and hygiene follow all functional scope. Original
findings remain unresolved until their full obligations are demonstrated; only F29/P-D5
currently has scoped resolution evidence.


The current integrated source also includes private cubic workers, guarded real-root
isolation and exact achieved-order memo validity; typed `solve_bound` with current
binding provenance; and ordered V4 ingress metadata plus an explicit parent-qualified
source delta boundary. The latter conservatively re-resolves outgoing references and
hydrates data only when the semantic supplier closure consumes it. Parent-preserving
text reload, shared parsed fields and retained granular compiler adoption remain open.
Relational owner-carriage/traversal controls pass, including existing tagged-arm masking
controls. These source slices are undergoing the combined native Runtime/Python compile;
actual canonical races, bound solves, shooting upgrades and installed-extension tests
have not yet established acceptance. The canonical/native state remains private and
production cutover or content retirement is not authorized by these tests.


Checkpoint continuation: P-C6 now has SCC-local ordering with global group decisions and
five passing pure projection controls, recorded by Proposed ADR-0179/revision153. Native
optimum/incumbent qualification remains open. P-B5 has a narrow protected completion source
slice; actual acknowledged normal close/seal and P-B4 demand migration remain open.
Proposed ADR-0180/revision154 records those already selected result contracts before dependent
registry/consumer migration. P-B2's completion-driven lane loop now refills freed lanes after
progress even if another lane previously reported Idle, and schedules recovery during
continuous refill; its focused controls are written and not yet run. Readiness hints,
projected finalization and affinity adoption remain open. No enclosing finding is resolved
by these source checkpoints.

Current continuation: the private general-history activation exercises paged
original IPC, derived locators, partial resume/replay and refusal fencing. The
enclosing first journey failed at final service startup; a preserved-fixture
continuation now passes the actual public historical reader and authenticated
shutdown, preserving incomplete and retired histories as ineligible. A fresh
enclosing journey remains required. The failed-start repair releases only an
authenticated unbound launch; bound, live, foreign-affiliation and uncertain
cases remain charged. Independent recheck found no unresolved issue in that
narrow repair.

SDK response ownership now retains actual inbound grants through late response
handling and abandoned delivery. Focused SDK and Operations/columnar controls
pass; Runtime shared-grant and installed Python cancellation acceptance remain
open. Both interactive extension variants have built and reported their actual
compiled features; correctness capture refuses their lack of force-validation.
The two validated artifacts and the remaining Runtime/native/Python controls
still precede assembled qualification. Disk capacity has been restored and cold
builds have resumed. The fresh whole-history backup/restore and preserved-history
continuation now pass their public-reader and authority-rotation controls; the
earlier enclosing journey retains its failed cleanup outcome.

Completion continuation, **2026-10-12**: the current Runtime campaign exposed double charging of prepaid constructor
envelopes and a readiness-recovery loop that could starve study discovery. Both
repairs are implemented; the constructor repair has independent source review,
and affected functional controls require rerunning. That campaign also retained
fixture creation borrowers after test-process exit, so its enclosing allocation
release failed. Authenticated recovery drained its exact registered receivers
without changing outcomes, retention pins or databases; normal admission
reconciliation then released the owning allocation. A test-fixture lifetime repair
now makes final lexical teardown join the tracked automatic protection release;
its gated acknowledgment control and neighboring lifetime/timeout controls pass.
Production asynchronous cleanup remains unchanged. Subsequent Runtime retries
now complete their owned drain. They also exposed active-admission self-wakeup
starvation and continuation screening outside the retained lane's population
scope; both production repairs have independent source review and their affected
functional controls pass. The two stronger assembled controls now pass:
the source-edit journey distinguishes retained semantic compilation from new
numerical check workspaces, and the one-slot continuation oracle requests
explicit row/variable accuracy rather than relying on unscaled fallback budgets.
The instrumented edit journey also passes with receiving-process syscall and
before/after-first-native map observations; tracing excludes separately placed
workers/services and establishes no speedup or universal latency claim.
Ordinary and native interactive extensions both report their actual installed
features, reject correctness dispatch, and pass direct integer/millisecond
boundary controls. Native-only horizon state is scoped to the actual
DiffSol driver while preserving ordinary inspection and canonical serialization.
Process exit and a passing test body do not establish fixture drain or enclosing
acceptance. All four installed Python modes now pass their direct boundary controls;
both validated builds regenerate their actual stubs without compiler warnings.
The real entered-pull SIGINT control passes against the exact preserved private
history fixture, retaining the response grant until transport drain and escaped
arrays after close. The ordinary bridge control passes too. Selected native
Python initially completed with 45 passing and three failing controls. The binding
identity getter now preserves its exact digest while returning the algorithm-tagged
form required by the typed Python consumer. The reload control compares complete
declarations by identity and uses V4 to witness source order; membership enumeration
does not promise ingress order. The sweep control now checks one structural view per
execution lane and exact preparation accounting across all 1,000 scientific points.
These repairs retain parent/history and original scientific assertions. The rebuilt
native validated extension passes its direct boundary controls; affected Python
reruns remain pending. The first full native Rust assessment completed with failures; managed-primary
journeys and scope-end qualification remain open. R36-01–R36-03 remain unselected;
no production migration or history deletion has been performed.

The assembled native run exposed additional boundary defects: redundant full-output
support projection during derivative upgrades; a valid cached compact program rejected
after its weak original source expired; default backend settings counted through an
unsupported serialization; acknowledged completion cancellation bypassing exact-fence
recovery; and internally tagged unit variants accepting unknown document fields. These
have bounded repairs and independent source review, with revealing regressions retained
or strengthened. Their reruns and enclosing qualification remain pending. The functional
assessment route now composes its actual producer-fixture prerequisite; its full tooling
test module passes. Fixture corrections preserve original scientific, memory, ordering,
empty-result and lifetime obligations rather than admitting invalid rows or widening
production limits. A queued-memory control now waits for actual release notification,
not elapsed time. None of these source repairs closes P-G1 by itself.

The first assembled ordinary-native attempt also exposed incomplete terminal-resource
projection: its frozen outcomes are preserved, but one five-second clock covered the
entire finite membership and expired after a prefix. Projection completion and test
success remain distinct. The bounded repair retains caller/cancellation clocks, uses
ordinary per-operation metadata bounds when no caller clock exists, and settles an
already projected prefix without rewriting unchanged authority. Its regression controls,
independent readback and authentic resume precede the next assembled attempt. The current
worker and validated native extension rebuilds pass; scope-end Rust repairs still require
their affected artifacts to be rebuilt. The fresh enclosing historical-activation journey
now passes with all 78 frozen script/configuration inputs unchanged, including actual
historical readers, backup/restore and authenticated shutdown. Tooling setup and documentation publication
have passing scoped results; later source changes still require their affected checks.

Scope-end reconciliation corrected retired Python scalar-cell consumers, invalid tagged
declaration fixtures and explicit full-result consumers, while retaining every-arm transport,
concurrency and scientific assertions. Python lint and type checks pass, as do the affected
codec controls. The full setup rerun passes after typed metadata and signal-owner repairs.
An independent readback caught integer-to-float deadline rounding; the helper now retains
the admitted numeric value exactly, with an exact-integer regression. Rust hygiene repairs
preserve admitted inline lane ownership and replace unavailable-lane panics with typed
refusal after all issued work drains. Their compile, affected functional and enclosing
qualification remain in progress; neither source presence nor a partial result closes P-G1.

The authentic terminal projection resume completed all 788 frozen members while retaining
the original 208-member prefix files and all frozen outcomes. It did not turn the failed
assessment into a pass or invent missing drain evidence. Governance, generated-output
comparisons, doctests and both ordinary Clippy feature modes now pass. The native lint and full feature powerset checks now pass, as do the rebuilt worker
and validated native extension with its actual installed boundary controls. Ordinary-native,
Python and managed-primary acceptance remain open. All 101 clock-consumer
controls now pass with the completed activation's light-slot allocation released.

The cancellation publication repair preserves the original refusal: a failed append group
discards its completion intents, so a second cleanup flush cannot claim coverage without
ACKs and replace exact cancellation with an incomplete-response error. The original
cross-process SCIP control, both durable completion controls and the new real-study
cancelled-flush regression pass together with enclosing cleanup. Arbitrary incomplete
responses remain infrastructure failures.
A separate host-release defect is repaired: a storage-only drain projection now selects
only its own ACK guards while the parent retains and checks every execution guard.
Affected host/supervisor controls and independent readback pass; actual failed-run drain
now completes without changing the failed test outcome or releasing unproved resources.

The next ordinary-native assessment completed its test bodies with 3,455 passes and five
failures. Three fixture corrections preserve full candidate admission, additive historical
frame identities and actual native-seed ownership; the two direct compiler/identity
controls pass. A real response-handler lifetime race also remained: ACK delivery could
wake a caller before the native snapshot released its payload grant. The SDK now joins
only native holds under the original request clock while the receiver retains decoded
output; all engine API and native WebSocket controls pass, with independent source
readback. The immediate queue-drain and subsequent cold-producer assertions remain.
The fifth failure occurred during private two-worker setup's release-metadata fetch,
before worker execution; the actual two-worker retry now passes. The repaired
queue-drain, cancellation and durable-completion controls also pass with enclosing
cleanup. The continuation fixture now uses the exported-seed POUNCE NLP route
without an unbound declared-composition request; its original scientific and
ownership assertions pass. No numerical tolerance or production permission changed.

The assessment's resource reconciliation additionally exposed a five-second metadata
clock retained across a separately allowed ninety-second database removal. Reclaim now
uses one existing finite removal allowance for claim, effect and settlement, preserving
any supplied original clock. Failed reconciliation stays recorded and unproved resources
stay pinned. Concurrent review prose changed only the producer prerequisite's overly
broad tooling scope; its declared Rust/build input closure now excludes that unrelated
prose while retaining product, toolchain, execution and cleanup inputs. Affected tooling
controls, rebuilt worker/extension, native repairs, Python and managed journeys precede
the next assembled attempt. P-G1 remains open.

The subsequent full setup run exposed two genuine cancellation-cleanup race orders:
a repeated owner signal could escape after the signal handler was restored but before
physical observation, and launcher reaping could precede descendant drain. Isolated
controls passing did not settle those races. Their repair keeps the non-raising
signal owner through authenticated physical join, preserves the original clock and
refuses unavailable or mismatched ownership. The pending-signal fixture uses exact
thread-directed injection to test its claimed masked-thread boundary. Independent
readback also preserved child-wait timing separately from physical drain. A subsequent
enclosing attempt exposed temporary recipe packages resolving production modules under
an inherited checkout path; those fixtures now isolate their actual recorders and the
exact accidental disposable ADR was removed. All 940 setup controls now pass, without
widening allowances. The actual installed native validated extension retains its verified
digest and passes its direct boundary controls. All 49 selected actual native Python tests
now pass, including constructor thread progress, typed binding, parent-preserving reload
and exact 1,000-point preparation accounting. Managed Rust passed three of four journeys;
its publication/reader journey exposed an eight-page fixture bound for a nine-page
retirement protocol. The inventory-derived correction preserves all lifetime and payload
oracles and has independent readback; all four managed Rust journeys now pass.
Managed Python completed six tests before the durable 1,000-point sweep stopped its
worker on a stale recovery hint. Recovery's fresh point was already observed, but it
attempted another observation. The fresh assigned/settled guard and settled/retryable
regressions pass with enclosing cleanup. The interrupted client and exact receiver have drained;
unfinished outcomes remain unproved and pinned. Rebuilt managed Python, refreshed
activation, assembled native and final hygiene acceptance remain open.

Rebuilt managed cancellation also exposed background recovery fencing a still-live
cancelled owner before its own drain and settlement. The correction requires terminal
readback or lease expiry and preserves the existing local candidate turn until that
owner joins. All four latest-source recovery/rotation/completion controls pass with
enclosing cleanup, and independent readback found no material flaw. All four rebuilt
managed Rust journeys pass with enclosing cleanup, including actual sixteen-owner
cancellation/drain. Rebuilt managed Python acceptance was interrupted after its worker failed during sealing, as recorded below. P-G1 remains open; earlier passing journeys retain their
original source and artifact scope.

Managed Python then exposed repeated definite seal conflicts during the durable sweep.
The existing bounded retry owner was already rebuilding the exact transaction; terminal
active-root removal unnecessarily notified the shared problem-wide source-creator
guard. A held-transaction regression reproduces that false conflict. Proposed ADR-0177
and blueprint §20.4 now clarify removal's exact-root/reclaim registration and retained
run-source dependency before the generator correction. Actual root creators retain
their both-order reclamation protocol. Regeneration and the revealing preservation controls are recorded at the stop checkpoint below; rebuilt acceptance remains pending. Failed/interrupted outcomes remain
unproved and pinned after authenticated drain.

## Maintainer stop checkpoint, 2026-10-12

Plan37 has completed its separately owned lifecycle/artifact/connection/tooling target; its [Outcome](37-surrealdb-lifecycle-and-integration.md#outcome-recorded-after-implementation) records the bounded acceptance and exclusions. The prospective contracts in these companions now consume that target. The historical checkpoints below remain unchanged evidence. This handoff does not resume Plan36 or close P-G1 or its finding dispositions.

The maintainer requested finishing immediate tasks, reconciling and removing worktrees,
updating the owning documents and stopping. The prepared next acceptance queue was
**not_run**. No further product or scientific campaign is running under this handoff.
The six plans retain `in-progress`; P-G1 and the original finding dispositions are not
promoted by this checkpoint.

**Implemented:** the terminal-seal generator now removes the owned active-attempt root
under the existing reclaim/exact-root registrations without writing the shared
source-creator fence. The retained run root still protects the source; actual creators
keep their paired reclamation protocol. Proposed ADR-0177 and blueprint §20.4/revision156
record that clarification before the generated change. `just codegen` regenerated all
outputs successfully. Independent bounded readback found no material flaw. The revealing
tests needed an SDK optional-result type correction and an actual row read rather than
a record-ID comparison; neither correction weakened the transaction or lifetime oracles.

**Tested:** the immediate four-control native canonical selection passes all four bodies
against zero permitted failures (226 excluded, local profile, explicit validated closure).
The four bodies completed in 29.553s; enclosing execution and authenticated cleanup exited 0. The log is `/tmp/pse36-completion-r22rcl9o/seal-independent26.log`.
The command was:

```sh
env PSE_TEST_RESOURCE_STATE=/tmp/pse36-completion-r22rcl9o/seal-independent26-registry \
  PSE_SURREAL_STATE=/tmp/pse36-completion-r22rcl9o/ordinary-canonical-state \
  scripts/pse-env --resource-class exclusive --store \
  --native=solver,klu,isolation,uno,petsc -- \
  just native-test --profile local -p pse-operations --lib \
  -E 'test(exact_terminal_seal_and_unrelated_source_creator_commute_both_orders) | test(exact_terminal_root_release_and_source_collection_preserve_run_both_orders) | test(exact_source_root_and_pin_creation_reclaim_both_orders) | test(exact_result_pin_creation_retirement_both_orders)'
```

The controls establish both commit orders for independent source creation/seal, active-root
release/source collection with retained run protection, source root/pin creation/reclaim
and result pin creation/retirement, including same-operation seal replay or rebuild.
They do not establish that every possible seal conflict is removed or that the actual
1,000-point managed Python sweep now succeeds. The previous managed Python attempt
completed six tests before worker failure, then was interrupted and authentically drained;
it is not a nine-test pass. Its incomplete outcomes and retention pins remain unchanged.
The rebuilt worker, extension and four passing managed Rust journeys precede this latest
seal change and retain that earlier artifact scope.

All five auxiliary worktrees were reconciled and removed: `p28a-description-20261008`,
`p28j-compiler-20261008`, `plan35-composition`, `plan35-lifecycle` and `plan35-lookup`.
Their committed heads were already ancestors of main. Independent bounded readbacks
found their dirty contributions integrated or superseded; obsolete alternative mechanisms
were not transplanted into newer source. Full semantic equivalence of every historical
Plan 28 hunk was not asserted. Before removal, all 12,464 regular-file/symlink entries,
staged/unstaged binary diffs, untracked/ignored contents and branch/head/status metadata
were preserved and verified in the local restorable snapshots at
`/home/paul/.local/share/pse-arrow/worktree-closeout/20261012T164932Z/`.
The manifest and closeout record live there; branches were retained. No observable
process use of these worktrees was found; inaccessible process surfaces do not establish
a universal host drain claim. Only the main checkout remains registered.

The remaining acceptance route, when work resumes, is:

1. Rebuild the actual managed worker and validated native extension from the latest source;
   repeat their direct installed-boundary controls and producer/activation bridge.
2. Run all four managed Rust and all nine managed Python journeys, including the revealing
   durable 1,000-point sweep and actual cancellation/drain. Investigate failures without
   changing clocks, ownership or scientific oracles to make them pass.
3. Refresh/freeze the activation's actual inputs and run its enclosing preservation journey,
   then run the assembled ordinary-native assessment. The earlier 3,455-pass/five-failure
   assessment stays failed despite later focused repairs.
4. Complete affected scope-end hygiene and manual native/governance/docs checks, then
   reconcile original findings and packet acceptance from the actual combined evidence.
   Earlier 940-control setup, 49-test native Python and 398-configuration feature results
   retain their recorded source and artifact scopes.

F20's sparse numerical-rank/fill limit and F28's host-policy investigation remain open.
R36-01, R36-02 and R36-03 remain unselected. No production-history deletion, earlier-plan
reconciliation, ADR status change, whole-product qualification or Plan 36 closure occurred.
**Tested:** root `just turn-end` passes ADR-index regeneration and formatting against the
zero-failure baseline, using the explicit private registry
`PSE_TEST_RESOURCE_STATE=/tmp/pse36-completion-r22rcl9o/stop-turn-end-registry`.
Its enclosing report is
`build/assessment/20261012T165401.965686Z-turn-end-2233435-28161b/summary.md`.
The first invocation refused report registration before either step because the default
resource registry requires explicit fenced migration recovery; that registry was left
untouched. The isolated rerun exercised the same checkout and steps without bypassing
that safeguard. Broader hygiene/manual checks were not restarted for this stop handoff.
All subagents have finished; no owned worker, native-test controller or acceptance queue
remains running. Shared parked services and unresolved retained evidence remain owned
by their existing lifecycles. Main contains the preserved uncommitted changes; no commit,
push or new acceptance queue was issued during this wrap-up.
