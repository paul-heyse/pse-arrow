---
title: "Plan46d: Completion scheduling and fitting balance"
status: done
date: 2026-10-18
adrs: [ADR-0166]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, numerical-execution, performance]
---

# Plan46d: Completion scheduling and fitting balance

## Target and foundation

Own AF11/AF18, I07 and RC05 implementation. The [coordinator](46-execution-speed-and-numerical-efficiency.md) owns decisions/dispositions. D01–D05 adopted functional scope and the bounded I07 decision are complete on assembled main. Selected final qualification and whole-series acceptance are recorded by [G06](46g-investigations-qualification-and-design-publication.md#g06). The AF18 production cost-balancing gap remains explicit: compact inventories are adopted, while production retains stride.

**Interface-checked baseline:** block solves and parallel initialization both used waves, but their ownership differed: solve waves created scoped threads/fresh Retained state; parallel initialization opened/closed MathService sessions and joined a wave. Serial initialization already retained one session and per-block bindings. `prepare_blocks` sized maximum teams/foreign storage over simulated canonical waves. Fitting experiment lanes used static stride and full-length optional worker inventories; profile-chain dispatch was already dynamic. Studies already had bounded completion-driven scheduling, preparation and continuation ownership.

The target changes physical readiness and ownership while retaining one scientific stage/work policy. Native numerical iteration remains library-owned. No new scheduler platform or generic work-stealing runtime is needed. Dependency readiness uses the compiled execution graph, including provider/control inputs, not visible streams.

<a id="d01"></a>

## D01 — Prepare readiness and compact consumed bindings

**AF11.** Prepare predecessor counts, successor lists and exact block input mappings once with the actual conditional structure. Retain compatible structural maps across value rebinds; changed consumed structure invalidates them. Dispatch reads only required committed values rather than cloning the complete scalar map per block.

Use shared bookkeeping across existing parallel owners without discarding their different stage/session semantics. Retain serial initialization's working session/bindings. **Targeted acceptance:** chains, independent blocks, provider/control edges, cycle/refusal, duplicate input contributions, value-only versus structural change and same original assessment. Count readiness scans and bound input copies; no stream-topology approximation.

**Implemented:** both owners share prepared predecessor counts, successor lists and consumed-binding maps from compiler-issued block identities. Conditional maps retain every authored fixed variable and parameter, including unused declarations, plus selected unknowns and execution/local proof-slot inputs. Unread authored free coordinates are omitted; their effective conditional fixed flag does not turn them into source assumptions. Existing public full/effective-frozen validation remains unchanged, while explicit conditional bind/rebind admission checks the narrower current inventory. `Facts.matches` still checks exact consumed assumptions. Request-local shared maps retain their allocation leases; initialization reuses exclusive bases after child aliases drain and admits a fresh compact allocation when aliases remain.

<a id="d02"></a>

## D02 — Define and enforce live-set, budget and commit policy

**AF11; RC05; hard prerequisite for D03.** Replace simulated canonical-wave maxima with actual dispatch admission or an envelope that covers all permitted overlap. Qualify resident workers/stacks/factors, active native teams and process gates separately. Lend/quiesce parent compute before children need independent acquisition; no admission deadlock with parent-held permits.

Reserve iteration/work permission before dispatch, then reconcile actual issued work at completion. Merely subtracting completed counts lets concurrent children spend the same remainder. Preserve current complete iteration coverage requirements: unsupported POUNCE block routes must still decline before effects until their actual native coverage is implemented, with the ordinary original strategy unchanged.

Successful dependency commits may unlock descendants before an unrelated failure is known. Keep these provisional values distinct from accepted atomic stage completion. Stop new dispatch after known terminal failure, drain all issued descendants and preserve their work/reports; rollback cannot undo provider/native effects. Deterministic visible failure/report order is independent of completion order.

**Targeted acceptance:** unequal mixed backend team/storage widths forming live sets unavailable in old waves, scarce capacity/parent lending, simultaneous near-budget completion, missing native work coverage, sibling failure after descendant launch and cancellation/drain. One existing initialization/composition policy owns semantics, not a second scheduler acceptance layer.

**Implemented:** live dispatch checks active team demand; block-solve reservation covers permitted overlap rather than simulated waves. Parent compute is lent before parallel children acquire existing MathService resources. Finite iteration grants are reserved before dispatch and known actual work returns only unused permission; a remaining unit can serve a late chain despite vacant lanes. Scoped child ledgers retain registration through native work, independent assessment and recovery, then deregister at explicit whole-child drain. Unknown bounded work remains unknown in observations and retains its finite permission; later known hooks advance that retained upper bound. Unsupported POUNCE aggregate iteration coverage still refuses before effects.

<a id="d03"></a>

## D03 — Release committed successors on actual completion

**AF11; consumes working D01/D02 and A01 for provider validity.** Run dependency-ready work on existing bounded MathService lanes. A fast predecessor's successfully checked commit releases its successor without waiting for an unrelated slow sibling; unresolved/failed prerequisites release nothing. Migrate both block-solve and parallel initialization paths, preserving stage overlays, prior accepted starts and original full-model assessment.

Coalescing tiny compatible blocks is a conditional formulation choice with C01, not a scheduler assumption. Recycles/tears keep declared convergence and selection policy; a nonconvergent recycle remains an honest outcome. **Targeted acceptance:** controlled skewed sibling plus chain, exact prerequisite visibility, failure before/after unlock, deterministic report reduction, stage atomicity, original scope/deadline/work and complete drain. Delete replaced wave execution/barrier machinery once callers and tests move; retain canonical ordering where it still defines public reports, not as a hidden physical barrier.

**Implemented:** successful checked completions release successors immediately in both parallel owners. A known failure stops new dispatch and all issued work drains; provisional dependency commits remain distinct from atomic accepted stages. Canonical reduction includes native outcomes and pure dispatch refusals. The replaced wave helper and wave-specific test are deleted after the replacement completion/failure/cancellation controls passed. The serial route retains its existing session and does not acquire a Tokio runtime merely to execute synchronous work.

<a id="d04"></a>

## D04 — Retain only qualified neutral private owners

**AF11; RC05.** Reuse existing session/lane machinery and per-block or lane-private scratch across compatible tasks. Current `Retained::reset_independent` qualifies only certain neutral backend owners; longer thread lifetime does not qualify keeping arbitrary KINSOL/PETSc state or factors. Never transport solved roots, acceleration/history, scope or cancellation with neutral capacity. Accepted start/continuation transport stays at its scientific owner.

Account resident idle capacity and pressure retirement; owner identity/lease/drain follow actual work. Migrate both execution paths with D03 and preserve single-session serial behavior. **Targeted acceptance:** construction/checkout/reset counters, changed assembly/backend, prior failure/unknown integrity, pressure, cancellation and final release. Unsupported neutral reuse uses a fresh owner with explicit cost, not stale state or an unused compatibility layer.

**Implemented:** both parallel owners use existing independent-session checkout/park and backend-qualified `reset_independent`. Current bindings remain at request/preparation owners, never on a neutral parked session. The existing unsupported-backend fresh-state fallback, pressure retirement, original scope and final drain remain authoritative; longer lane lifetime does not qualify KINSOL/PETSc factors, numerical histories or accepted starts for neutral reuse.

<a id="d05"></a>

## D05 — Balance repeated fitting experiments with stable affinity

**AF18; I07.** Compare current stride with stable cost/affinity-aware assignment before considering dynamic unstarted dispatch. Use compact lane-owned worker inventories where static ownership makes full experiments × lanes optional slots unnecessary. Keep one private experiment owner, bounded residency, no replay after lane failure and authored-order atomic reduction.

Profile chains already claim tasks dynamically; their scientific pins remain sequential. Existing `max(native_threads, experiment_cores)` phase envelopes and compute lending are strengths, not evidence every pool multiplies independently. **Decision/targeted acceptance:** repeated uneven experiments, changed trial/order, lane failure/cancel, worker reconstruction/idle/resident cost and original gradient/curvature/fit acceptance. Adopt the simplest qualified assignment; no unbounded experiment replication to make work movable. Preserve study completion scheduling; only investigate remaining overlap if actual observations reveal idle work.

**Implemented / I07 decision:** production retains authored stride with compact assigned worker inventories and aligned per-trial requests. Each private worker is constructed once on its owner; failed work is not replayed and visible reduction remains authored-order atomic. A test-only comparison fixes distinct stride and balanced IVP/steady inventories before construction and observes actual completion, relative lane idle, construction, resident bytes and repeated original objective/gradient results. No production timing estimator or affinity policy is adopted. Generated rate-provider registrations make the existing transient population bound unknown, so residency weights cannot qualify general cost selection. Revisit balance when source-owned work/worker bounds or another explicit ownership contract supports an assignment that can be qualified at the real consumer. Dynamic profile-chain scheduling remains separate and unchanged.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D01 | Actual compiled dependency/conditional identities | Prepared readiness/consumed bindings for both parallel owners; graph/value controls | done |
| D02 | RC05 decision; effective profiles and complete native work coverage | Actual-overlap admission, reserved work and provisional/atomic commit policy; race/drain tests | done |
| D03 | Working D01/D02; effect-safe A01 for provider paths | Completion-driven dispatch in real block/initialization callers; skew/failure/original checks | done |
| D04 | D02/D03 session ownership; backend-qualified reset | Neutral retained lanes/scratch without numerical/start leakage; pressure/failure/release tests | done |
| D05 | Existing fit team/profile/experiment contracts; G01 observations | Compact inventories, retained production stride and explicit I07 comparison; ordered science controls | done |

## Historical executor verification and handoff

**Historical verification approach:** controlled scheduler units avoided stores/unrelated numerical startup; native integration controls separately exercised real lane/reset/gate/work contracts. Shared scheduler/admission owners used one integrator. Final selected studies/fitting/recycle and scope-end qualification belong to G06. Replaced wave dispatch paths are deleted, and partial stages remain distinct from accepted stages.

**Tested, historical scoped pre-followup checkpoint:** the coordinator's joined targeted `just unit-native-capability-package pse-runtime` selection, with KINSOL/Diffsol, native `solver,klu` and explicit correctness validation, ran 26 tests: 24 passed and 2 failed against a zero-failure baseline. Passing controls included prepared readiness/value rebinding, finite grants, child phase/recovery and concurrent reservation, both real completion-driven owners with failure/cancellation drain, neutral lane pressure/deadline release, and existing ordered fitting worker ownership. The automatic original-block journey failed on unnecessary serial Tokio-handle acquisition; the uneven fitting comparison failed because its helper expected success after deliberate cancellation. Both were subsequently corrected and exercised by the joined controls in the Outcome. The prior comparison selected identical stride inventories and supplies no evidence for an affinity policy.

**Historical not_run boundary after that checkpoint:** retained unknown-upper advancement with later known sibling hooks; serial reactor/cancellation fixes; legacy wave retirement; removal of unqualified production affinity selection; genuinely distinct test-only I07 assignments; and compiler compact conditional/source-frozen/backend shared-value controls had not yet received joined verification. The central native/runtime controls in the Outcome supersede those exclusions for their actual selections. No accuracy target or environment qualification claim changes.

## Execution checkpoint — 2026-10-18

All five packets and the bounded I07 inquiry are complete on assembled main. The D worktree
is integrated and removed. Independent review corrected premature child deregistration,
zero grants with vacant lanes, and lost known work after an unknown bounded completion;
joined controls now cover these repairs. G06 owns final scope-end qualification and
whole-series acceptance; no further D production policy is selected by the inquiry.


## Outcome

Recorded 2026-10-18.

**Implemented:** prepared readiness/compact consumed bindings, actual-overlap and reserved-work
admission, completion-driven block/initialization dispatch, qualified neutral session reuse and
compact fitting inventories. Replaced wave dispatch machinery and the unqualified residency
affinity selector are deleted. Successful provisional dependency commits remain distinct from
atomic accepted stage completion, and every issued child drains before release.

**Tested, zero baseline:** the central full-native/canonical/compiled runtime selection
recorded in 46g ran 97 tests, 96 passed and one stale plateau fixture failed because it
omitted the selected authored start. Production callers already supply it. The fixture now
explicitly checks missing-start refusal and supplies authored x=1 before its 24-edit owner
and residency plateau check. The follow-up native-local consumer selection ran 11 tests,
**11 passed / 0 failed**, including that corrected control. No validator was weakened.
The passing joined checks include finite aggregate initialization assessment, multi-phase and
unknown-upper child reconciliation, last-iteration dispatch, completion/drain/failure order,
neutral owner pressure/reset, compact binding reuse and original block acceptance.
Raw selections/results are retained in `/tmp/pse-plan46-integrated-runtime.log` and
`/tmp/pse-plan46-runtime-consumers.log`; G06 owns the exact invocations and final qualification.
The original 97-test run remains a failed run; the affected 11-test rerun closes its fixture
failure and does not imply another all-97 run.

**Measured, I07:** `fitting_uneven_experiment_affinity_compares_stride_without_replay` compares
three repeated objective/gradient trials of the same four mixed steady/transient experiments.
Stride assigns [[0,2],[1,3]]; the explicit test-only affinity assigns [[0,1],[2,3]]. Both retain
89,300,384 charged resident bytes and identical scientific products, one owner per experiment,
original reductions, changed-trial reset and cancellation. One observation reported 8.67 ms
versus 4.11 ms of callbacks, with construction 0.539 ms versus 0.675 ms. Stride's observed
per-trial relative idle sums were approximately 0.62–2.07 ms; split affinity 0.002–0.056 ms.
These local measurements reveal avoidable skew; they are not statistical throughput guarantees
or a global cost classifier. Original native iteration/accuracy controls are unchanged.

**I07 decision / retained AF18 limit:** adopt compact worker/request inventories; retain
production stride. Real transient workers contain required generated providers, so their
memory producer bounds are unknown and cannot stand in for execution cost. The unqualified
residency selector was removed rather than kept as a second production policy. A future
assignment change requires qualified source/work-cost information or a bounded fitting-local
calibration/ownership design, complete construction/repeated-trial benefit and unchanged
failure/drain/reduction semantics. Profile chains keep existing dynamic claiming and causal
pins. This bounded inquiry is complete; general cost balancing remains unimplemented.

Mistakes corrected: serial dispatch unnecessarily acquired a Tokio handle; a cancellation
fixture expected success; early child deregistration could hide issued work; finite grants
could strand the last permitted iteration; and an unknown upper bound could lose subsequent
known work. Targeted joined controls exercise the corrections. The final plateau fixture
omitted its required selected start; it now checks explicit missing-start refusal before
supplying authored x=1, preserving admission rather than weakening it.

Deliberate deviations: retain production stride and backend-qualified neutral reuse instead
of selecting a cost classifier from residency or extending parked numerical state. The
test-only manual affinity is quicker in the reported three-trial skewed control, but source
worker bounds and general construction/work-cost classification remain unknown. No universal
fit-speed claim or new scheduler platform follows from the completed inquiry.
