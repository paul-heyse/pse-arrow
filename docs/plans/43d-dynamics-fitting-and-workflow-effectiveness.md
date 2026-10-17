---
title: Dynamics, fitting and workflow effectiveness
status: in-progress
date: 2026-10-17
adrs: [ADR-0194]
review_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, libraries, validation]
---

# Plan 43d: Dynamics, fitting and workflow effectiveness

## Purpose and consumed foundations

This companion owns F08 and workflow contributors to F01/F06, plus dynamic/fitting/shooting detail in O02/O04 and Q03. [Plan43](43-solver-diagnostics-and-performance.md) owns shared decisions and dispositions. RC06 was accepted on 2026-10-17; RC03/RC05/RC07 retain their stated product/reuse/investigation bounds. Product changes and acceptance are **Proposed**.

**Interface-checked:** IDAS/Diffsol drivers and adjoint paths, runtime dynamic windows/experiments, fitting oracle/final identifiability, shooting integral/terminal products, public Rust/Python requests/reports and related publication consumers. The same authored model definitions, typed event/reset and derivative contracts, independent closure/qualification, retained workers and explicit failed products are suitable foundations.

IDAS currently hard-stops at each requested output. The planned correction combines driver, event, effort and all consumer changes; removing one stop call is not an independently complete package. Existing fitting/shooting same-point products can legitimately require additional integrations, and retained workers do not imply retained IVP/checkpoint state across changed trials.

## Target packages

<a id="d01"></a>

### D01 — True boundaries, interpolation and cumulative effort together

The integrator owner distinguishes physical segment/discontinuity boundaries, requested samples, current native internal time and returned model time. Hard stops mark genuine boundaries. Eligible output within native coverage uses library interpolation without forcing a new stop or charging an invented step. Do not interpolate across a reset, changed input law, branch or reinitialized IVP.

Advance and output service have different effort semantics. A covered sample may be returned after the cumulative step allowance is exhausted; taking another native step may not exceed it. IDASetMaxNumSteps is per-call and native zero selects a default, not zero remaining work. Establish baselines/generations across reinitialization, segment transitions, checkpoint replay and backward sessions. Preserve the enclosing clock/cancellation/native drain and distinguish failed attempts from accepted step counts.

Handle native roots before a requested sample, scheduled changes and coincident root/reset/sample using the current post-transition sample contract. Consistent ICs, state and derivative output, sensitivities and quadratures remain in the appropriate native segment. Off-grid endpoints and partial/compact prefixes preserve requested scope and produced coverage. “Internal time passed tout” is not by itself permission to serve a sample if intervening event semantics are unsettled.

Migrate the full actual consumer closure: IDAS Session::segment and its effort/statistics; Diffsol counterpart conventions; runtime dynamic profile/identity/window/experiment and sample checks; fitting trial/response/adjoint/curvature/final assessment; single/multiple shooting windows and stitched outputs; Rust/Python dynamic requests, reports, coverage and publication. Preserve capability restrictions for supported DAE/events/adjoints/curvature; this package does not promise arbitrary new integrator combinations.

Required controls:

| Case | Required behavior |
| --- | --- |
| Several outputs inside the last native step | Serve each requested sample in order without a new hard stop or fabricated step |
| Budget exhausted with internal coverage beyond next sample | Covered samples remain available; further advancement ends honestly |
| Root before requested output | Root handling/reset precedes any output that would cross the transition |
| Scheduled change/root/reset coincides with sample | Preserve selected post-transition sample and consistent initialization |
| Immediately adjacent pre/post-event samples | Interpolation stays in the correct compatible segment |
| Off-grid final horizon, sparse/empty/intermediate output selection | Requested completion, native coverage and produced output remain distinct |
| Failure during reset/sensitivity/quadrature service | No partial product/cache completion; retain governing failure and honest prefix |
| Restored/restarted forward/replay/backward counters | Charge actual deltas under their native generation |

A controlled grid-density observation keeps equations, ICs, tolerances, events and first output fixed. Independently check trajectories, closure/conservation and derivatives, then observe steps/evaluations/setups/failures/output work. A complete once-only case can establish absence of forced stepping; no repeated timing campaign or speed ratio is required. Actual benefit remains unmeasured until observed.

<a id="d02"></a>

### D02 — Integration purpose, exact-point products and derivative upgrades

At the performing workflow owner, record primal trial, derivative upgrade, forward response, adjoint/replay, curvature, original assessment and rank/covariance/identifiability reasons. Retain both distinct parameter-vector and callback populations. Final identifiability can legitimately request forward responses after a gradient fit; that work is not an ordinary optimizer gradient.

Integrate fitting experiments, dynamic windows and shooting terminal/integral objectives without flattening different products into one optimizer-evaluation count. Mixed objectives may require several products at one vector. Link actual model/input/profile/output/derivative/accuracy/branch premises to completed result reuse. A completed curvature product can serve changed solver weights where its derivative contract permits; changed directions or weighted products use their own applicable identity.

Reuse retained mathematical workers/layouts separately from fresh IVP initialization, native factors, trajectories and checkpoint histories. A failed/cancelled upgrade publishes no partial completion and does not reinterpret an existing lower-order result. Preserve lower-order reuse only under its complete original premises. Follow B04/B05 product/projection contracts rather than add another workflow cache over them.

A04 owns invocation/event record meaning; D02 supplies its actual purpose/start/attempt facts. Fitting/shooting/native reports, durable records, benchmark observations and inspection/export consume those same facts. No extra integration is initiated merely to produce diagnostics.

Acceptance covers new trial, exact cache reuse, same-point response/curvature upgrades, changed accuracy/configuration/output/direction, failed upgrades and repeated final assessment. Verify complete product validity independently and assert actual integration roles/counts. Observe legitimate additional work without labeling it inefficiency automatically.

<a id="d03"></a>

### D03 — Dynamic derivative and native-profile alternatives

Settle the dynamic slice of Q03 with existing typed choices: simultaneous/staggered IDAS sensitivity correctors, eligible forward versus checkpointed adjoint, checkpoint spacing, exact versus admitted approximate curvature, sparse/JVP linear routes and Diffsol KLU/faer alternatives. Fix requested outputs, error-control policy, initial/event semantics and derivative accuracy when comparing.

Method counters have different meanings and denominators. Accepted/rejected steps, residuals, sensitivities, IC setup, forward recording, replay and backward solve work remain separate. Restoring a solver with inherited statistics does not create fresh work. Record checkpoint count/retained bytes/spacing and measured replay under actual generation; allowances remain allowances.

Prefer fitting library capabilities and explicit profile eligibility. A requested derivative may be unavailable for an event/reset/output combination; that is not zero response or permission for an unreported fallback. Exact transient Hessian and other restricted routes retain their actual provider/backend conditions. Do not change a default simply because an alternative is already representable.

Completion is a concrete profile decision supported by equivalent scientific products/work, or retained defaults with an evidence-supported no-adoption conclusion for the considered alternative. Interpolation correctness in D01 does not depend on selecting a new corrector or linear backend. Any broader newly exposed native capability follows C06's exact-release/ABI route.

<a id="d04"></a>

### D04 — Workflow qualification, requested products and readiness tails

Make workflow original qualification and postsolve analysis/publication/access work identifiable under the original operation. A backend native quality report may precede governing workflow closure or output-accuracy checks. Retain original status, candidate permission, withheld response/covariance and durable publication state separately.

A typed response can be ready with an unavailable scientific product. Failed qualification has no qualification-completion timestamp, although terminal elapsed exists. Distinguish requested analysis computation, materialization, canonical publication and actual reopening/querying; do not invent every tail for a consumer that did not request it. Timing uses actual owner endpoints and overlaps, not summed child durations.

Migrate dynamic/fitting/shooting and applicable study/result Rust/Python consumers, benchmark/report readers and E04 views together. Preserve existing immutable artifact/protection/ACK owners; diagnostics supplies no activation or cleanup permission. A failed optional writer never revokes scientific success, while a required scientific publication failure retains its actual result-availability consequence.

Acceptance covers usable point with withheld local analysis, original failure after native success, analysis failure after qualification, publication/materialization/reopen failure, partial trajectory, cancelled fitting/study and related-owner expiry. Required outputs and governing outcomes remain interpretable after complete or prefix-only diagnostic retention.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D01 | Accepted RC06; pinned IDAS interpolation/event/effort contract; actual dependent consumer inventory | Combined boundary/interpolation/budget correction with all dynamic/workflow consumers and edge controls; remove sample-hard-stop path | open |
| D02 | Existing product validity; A04 record slice; B projection/product slice where changed | Actual integration purpose and complete same-point upgrades/reuse across fit/shooting/response/curvature; remove misleading aggregate-only interpretation | open |
| D03 | Q03 capability/accuracy/profile decision; C06 only for new native capability | Equivalent supported derivative/linear/corrector/checkpoint profiles or retained defaults, with correct work generations and eligibility | open |
| D04 | A04 endpoint meaning; actual workflow/result/publication owners | Complete qualification/withholding/readiness facts and consumers, without changing science or artifact authority | open |

<!-- Package state is owned in this table. -->

D01 is a combined correctness delivery, not independent mesh/budget/event edits with premature acceptance. D02/D04 can establish purpose/endpoint evidence under the existing driver; D03 alternatives need not wait for the entire diagnostic tool stack. Shared runtime/dynamic/fitting source requires coordinated editing.

## Verification and checkpoint

**Proposed:** native driver fixtures and independent analytic trajectories/derivatives, actual fitting/shooting product controls and targeted workflow/Python checks; force-validation for Arrow correctness closures. G02 assembles dense-output/event and derivative journeys after integration. G03 selects grid/work observations only where they settle a decision or support a claimed benefit.

The supporting checkpoint below records observation-only integration purposes and endpoints. The existing driver and numerical products remain in force; no driver optimization or speedup is claimed. The remaining D scope stays open. Full F08 and workflow F01/F06 closure belongs to the coordinator with their other contributors.


### Plan43a supporting-scope checkpoint — 2026-10-17

Observation-only D02/D04 support records actual fitting/shooting integration purpose and requested response/adjoint/curvature products, direct workflow endpoints and per-invocation references. No new integration, derivative request, IDAS driver or method policy is introduced; D01–D04 numerical and broader product scope remains open. [43a](43a-operation-evidence-and-coherent-live-diagnostics.md#a-execution-checkpoint) owns the selected execution checkpoint; [Plan43](43-solver-diagnostics-and-performance.md#current-checkpoint) owns finding disposition. [43a’s Outcome](43a-operation-evidence-and-coherent-live-diagnostics.md#outcome) records passing selected acceptance and its exclusions; package table states continue to describe each package’s complete scope, now authorized for execution.
