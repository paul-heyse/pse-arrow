---
title: "Plan46e: Dynamic actions and checkpoint lifetimes"
status: done
date: 2026-10-18
adrs: [ADR-0186, ADR-0193]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, numerical-execution, performance]
---

# Plan46e: Dynamic actions and checkpoint lifetimes

## Target and foundation

Own AF06/AF15 and I08–I10. The [coordinator](46-execution-speed-and-numerical-efficiency.md) owns rule/inquiry/finding decisions. E01–E04 adopted functional scope and bounded decisions are complete on assembled main, including native dynamics and joined runtime action controls. Selected final qualification and whole-series acceptance are recorded by [G06](46g-investigations-qualification-and-design-publication.md#g06). The AF06 stronger coupled DAE preconditioner remains a retained production limitation.

**Interface-checked starting point:** Diffsol 0.16.2 has in-place operator actions, while the pre-change adapter assembled raw partials before products/transposes and repeated some stable scatter searches. IDAS already called direct JVP and native adjoint checkpoint routines; it was not missing an adjoint lifecycle. Its Krylov route has None/Jacobi and refuses BlockFactor. Before E01, `Session::solve` checked checkpoint population only after `IDASolveF(..., IDA_NORMAL)` returned, although native IDAS allocated checkpoints within that call. Selected sensitivity layouts, fixed-layout linear owners, schedules/events and original assessment are existing foundations.

Retain the fixed `diag(I,0)` ODE/index-1 scope. The actual DAE operator uses its current integration shift and algebraic mask, not a root preconditioner transplanted unchanged. Native libraries own integration/nonlinear iteration; parameter/IVP/mode changes invalidate history under existing contracts.

<a id="e01"></a>

## E01 — Enforce checkpoint population at the native-call boundary

**AF15; RC08 retains the existing finite population rule.** Integrate the existing IDAS single-step/checkpoint lifecycle or an equally sound native control that checks remaining checkpoint permission before issuing work that could exceed it. Establish exactly when the initial checkpoint, later stored checkpoints and segment reset allocate/release; an after-call count is observation, not pre-allocation enforcement.

Preserve output interpolation, root/event returns, stop times, accepted-step/replay accounting and native history semantics. Smaller target chunks alone cannot be called a bound unless every possible native checkpoint creation is covered. Cancellation/limits retain partial/terminal meanings and drain/release actual owned native memory. Do not invent a new integrator or weaken the configured checkpoint maximum to match the implementation.

**Targeted acceptance:** a long output interval versus several shorter intervals, very small checkpoint maximum, exactly-full population, next-checkpoint refusal, schedule/segment reset, event at a boundary, forward-over-adjoint sensitivity state, cancellation/failure and final release. Observe population during work, not only terminal refusal. Keep physical/integration/derivative tolerances unchanged; no long wall-time run is required to make many native steps observable.

<a id="e02"></a>

## E02 — Select direct versus completed-matrix actions coherently

**AF06; I08; consumes relevant A03/A04 and B02/B05 capabilities.** Prepare stable scatter offsets and integrate existing Diffsol in-place JVP/transpose/parameter action interfaces. An action-only request should execute its actual supported product without constructing all unconsumed partials. When a direct solver or repeated consumers already need a completed compatible Jacobian, consume that matrix instead of adding a second preparation/evaluation.

Keep the reuse relationship at existing assembly/product owners; product identity and current provider effects remain explicit. Do not add a global best-mode policy or infer that matrix-free always wins. IDAS direct JVP remains the control and retains its completed owned-result publication boundary.

**Decision/targeted acceptance:** action-heavy and factor-heavy trajectories with nontrivial selected parameter axes and physical scaling, transpose/sign controls and same-point repeated products. Count preparation/derivative components/matrix assembly/action work across complete trajectories; compare independently known solutions and directional derivatives. Include modes/schedules, capability refusal and late provider failure. Adopt the qualified demanded/matrix reuse route in actual workflows and remove superseded scatter/search paths.

<a id="e03"></a>

## E03 — Qualify a coupled index-1 DAE preconditioner

**AF06; I09.** Compare current None/Jacobi with a suitable native/library-owned block, triangular, overlap or Schur realization for the actual operator `cj M − J` under the IDAS residual convention (or equivalent residual partial form). Preserve differential/algebraic coupling, actual mass structure, current shift/mode and consistent initialization. Structural splits need scientific/operator provenance, not stream connectivity or arbitrary SCC subdivision.

Define setup refresh and scratch/factor lifetime at the existing adapter before adoption. A root BlockFactor cannot be reused merely because dimensions match. Native failures/recoverable trials and actual admission remain explicit; integration and original physical checks do not inherit preconditioner success.

**Decision/targeted acceptance:** small stiff index-1 fixture with a genuinely coupled algebraic block, changing cj, consistent starts, scheduled changes and nonnormal/stability control. Count rejected steps, linear/nonlinear work, factor/apply/setup, memory/refusal and complete accepted trajectory. Select an in-bound qualified binding after the required decision route, or retain a precise limitation; no new DAE class is silently admitted.

<a id="e04"></a>

## E04 — Resolve forward/adjoint and checkpoint/replay tradeoffs

**I10; U10; E01 correctness precedes any IDAS population-sensitive adoption.** Compare selected forward axes versus selected adjoint functionals using the same scientific demand, including gradient preparation, replay and derivative products. Few parameters or few objectives inform the comparison but are not a universal crossover formula. Retain ordinary full public demands and complete physical/state dependencies.

Start with current fixed checkpoint spacings and segment retention on short controlled horizons; include interpolation storage, native setup and replay work. More checkpoints can increase setup as well as memory; fewer can increase replay. A new memory/recomputation scheduler is conditional on a demonstrated unmet demand and fitting library contract, not a required bespoke algorithm.

**Decision/targeted acceptance:** scheduled/interior sample functionals, few/many axes versus objectives, exact gradient/action controls, resource refusal and replay failure/stop. Existing selected-adjoint/quadrature/spacing tests are leads, not new executed evidence. Changed parameters never reuse old multistep/checkpoint history; retained mathematical workers remain separate. A longer follow-up needs a named remaining uncertainty, intended scale and bounded observation/stop plan.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| E01 | Existing configured checkpoint/step contract; exact IDAS allocation lifecycle | Pre-excess checkpoint enforcement in production; exact-boundary/event/stop/release controls | done |
| E02 | Working actual product capability/effect validity; prepared sparse maps | I08 decision and direct/completed-matrix action consumers; trajectory/transpose/axis tests | done |
| E03 | E02 operator semantics; actual native preconditioner/binding eligibility | I09 coupled shift/algebraic/start comparison; KLU/None/Jacobi retained; stronger binding unqualified | done |
| E04 | E01; selected demands; G01 observations | I10 sampled-functional comparison; explicit forward/adjoint demand and spacing 250 retained | done |

## Verification and handoff

**Tested:** targeted index-1/sensitivity/adjoint controls and actual native lifecycle tests
are recorded below, with exact pinned native/Diffsol source inquiry supporting the bounded
decisions. Joined runtime action controls are also complete for their selection. G06 owns
final dynamics/fitting/Python/static qualification and G07 design publication. No
higher-index/general implicit extension, new accuracy or long campaign is claimed.

## Execution checkpoint — 2026-10-18

All E01–E04 adopted functional scope and bounded I08–I10 decisions are complete. The
coordinator retains accepted rule decisions, shared dispositions and final qualification.
The native and joined runtime selections below exercise the integrated implementation.

The E01 implementation uses the plan’s equally sound native-control option: actual maintained
IDAS allocation-point guards, current/peak usage and latched refusal. Exact source inspection
showed this preserves NORMAL/root/interpolation semantics without host single-step orchestration.
It does not change native integration, checkpoint spacing, accepted-step budgets or accuracy.

## Outcome

Recorded 2026-10-18.

**Implemented:** IDAS guards both initial and subsequent native checkpoint allocations and
retains current/peak population, latched refusal and caller limits across segment reset.
NORMAL interpolation, event returns and native step/replay ownership remain unchanged.
Diffsol now guards before checkpoint state construction. Rejected/failed attempts publish no
gradient. Different requested output schedules may change the native initial-step estimate;
controls compare bounded/unbounded behavior under the same schedule and independently check
cross-schedule physical results at the unchanged 0.001 accuracy.

**Implemented, I08 decision:** actual dynamic assembly and anchored shooting/window
consumers request supported JVP/VJP products with selected physical chain maps and current
provider/guard effects. A compatible completed matrix remains reusable; action-only consumers
do not force raw full partials. Diffsol retains private action candidates and prepared refill
maps; generic valid sparse oracles may omit zeros or use uncompressed storage without being
mistaken for the packed-offset fast path. Failed/late actions publish no partial prefix.

**Tested, zero baseline:** `BASH_ENV='' PSE_TEST_RESOURCE_STATE=/tmp/pse-plan46-root-resources-20261018 just unit-native-capability-package pse-backend-native pse-backend-native/idas,pse-backend-native/diffsol solver,klu 'test(dynamics::)' --success-output immediate`
ran 118 tests, **118 passed / 0 failed**, 276 unselected, in the native-local correctness mode.
This includes checkpoint boundaries/reset, event/schedule trajectories, adjoints and
forward-over-adjoint, selected axes/signs, anchored transpose actions, sparse fallback and
atomic refusal. Earlier three failures led to a corrected native-density expectation, the
Diffsol pre-construction guard, and a separate compact conditional binding repair owned by D.
That 118-test historical dynamics boundary alone did not validate the D repair; the later
central native and joined runtime selections below exercise the integrated consumers.

**Measured, I09:** `coupled_index1_shift_preconditioner_inquiry_preserves_scheduled_and_nonnormal_trajectories`
ran twelve short complete scheduled index-1 cases: two coupling scales, two strengths and
KLU/SPGMR-None/SPGMR-Jacobi, at rtol/atol 0.001. Actual current `cj` is observed at Jacobian,
JVP and preconditioner setup. KLU used 33 steps/39 nonlinear iterations; None used 39/41 and
65 linear/JVP iterations; Jacobi used 38 steps, 48–51 nonlinear and 30–36 linear iterations,
29 setups and 82–91 applies. Physical trajectories passed. Each route took one observation,
approximately 0.12–0.27 ms; those tiny timings select no winner.

**I09 decision / retained AF06 limitation:** retain existing KLU, None and Jacobi. KLU is the
qualified coupled direct control. No qualified coupled matrix-free block/Schur binding exists
for current `cj M - J`; root BlockFactor is not transplanted. Reopen when a representative
larger index-1 trajectory shows linear/fill dominance and supplies scientifically justified
splits, algebraic regularity, current-shift refresh and a library-owned finite-storage binding.
Bounded E03 comparison is complete; the absent stronger Krylov preconditioner is not implemented.

**Measured, I10:** `short_forward_adjoint_checkpoint_inquiry_preserves_the_same_sampled_functional`
used a short scheduled nonlinear DAE, interior weighted samples, four integration parameters,
ordinary 0.001 accuracy and one observation per demand. Forward required 240 oracle calls/58
steps. Adjoint for one/all four selected axes used 692/660 calls at spacing 3 (peak seven
checkpoints), versus 513/488 at spacing 25 or default 250 (peak one). Reserved bounds were
approximately 258–289 kB; reservation is not measured native residency. Complete callback
counts include forward/replay/backward work. Gradient-induced practical functional changes
agree with the forward control; derivative checks do not tighten simulation accuracy.

**I10 decision:** retain explicitly selected forward responses or adjoint scalar gradients and
current default spacing 250. This tiny single-functional case identifies no universal
parameter/objective crossover and no unmet demand for another checkpoint scheduler. More
checkpoints increased work here; fewer selected axes did not guarantee fewer calls because
native error control differs. Additional objective populations and long horizons require an
actual consumer and a named replay/memory bottleneck. Changed parameters/modes always obtain
fresh native history; retained mathematical workers are a separate lifetime.

**Tested, zero baseline, central integration:** the native selection recorded in 46c and
`/tmp/pse-plan46-native-numerical.log` passed **163/163**, including dynamics and shared
conditional binding controls. G06's full-native/canonical/compiled runtime selection in
`/tmp/pse-plan46-integrated-runtime.log` ran **97 tests: 96 passed, one initialization fixture
failed**. Selected Diffsol cotangent and scaled-axis/complete-publication controls passed.
The initialization fixture was corrected without weakening admission; its affected
native-local consumer rerun in `/tmp/pse-plan46-runtime-consumers.log` passed **11/11**.
These selections complete the adopted E functional scope, not whole-product qualification;
G06 owns exact invocations and selected final qualification.

Mistakes corrected: checkpoint population was initially observed too late to prevent native
allocation, and Diffsol checked after checkpoint construction. Guards now act at the actual
allocation boundary. A density control confused schedule-dependent native step estimates
with the checkpoint rule; corrected controls compare identical schedules and independently
check physical results. Sparse action fallback now accepts valid omitted-zero/uncompressed
oracles while preserving atomic late-failure publication.

Deliberate deviations: use maintained native allocation-point guards instead of host
single-step orchestration; retain KLU/None/Jacobi because no stronger coupled Krylov binding
is qualified; and retain explicit forward/adjoint demand and checkpoint spacing 250 because
the short 0.001-accuracy sampled-functional inquiry establishes no universal crossover or
need for another scheduler. No GPU/general implicit DAE class, accuracy change or
long-duration campaign is claimed. Python and final static/manual results remain G06-owned.
