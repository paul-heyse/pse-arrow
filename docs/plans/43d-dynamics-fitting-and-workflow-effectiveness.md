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

This companion owns F08 and workflow contributors to F01/F06, plus dynamic/fitting/shooting detail in O02/O04 and Q03. [Plan43](43-solver-diagnostics-and-performance.md) owns shared decisions and dispositions. RC06 was accepted on 2026-10-17; RC03/RC05/RC07 retain their stated product/reuse/investigation bounds. At document authoring, product changes and acceptance were **Proposed**; the Outcome records delivered scope and evidence.

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

**Remaining production alignment — 2026-10-17:** `ShootingOracle::adjoint` in `crates/pse-runtime/src/workflow/shooting.rs` reuses the exact point but labels a newly required integral-objective `window_gradient` with `self.purpose`, which is `PrimalTrial` for the production oracle. The forward same-point upgrade path already selects `DerivativeUpgrade`. Give the missing adjoint product its actual derivative-upgrade purpose; retain distinct original-assessment/response purposes, legitimate additional integration and transactional caching. A value→gradient→repeated-gradient sequence must distinguish new derivative work from both primal work and reuse.

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

**Remaining production alignment — 2026-10-17:** `RunResult::invocation_evidence` in `crates/pse-runtime/src/workflow/run.rs` derives modeling sensitivity/root-response/reduced-Hessian availability from native `Result::is_ok` and shooting-trajectory availability from `trajectory.is_some()`. Native products survive later authored assessment in `modeling/results.rs::from_assessment`; shooting retains stitched data even when original checks refuse it. Complete the joined scientific-product availability contract using the actual governing original permission and requested product outcome. Modeling products use their individual step's completion; another step's refusal must not erase a valid step's product. Keep computed native evidence/partial trajectories available as explicitly diagnostic data, and keep response readiness separate. D04 owns this correction; A04 consumes it without a second availability authority.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D01 | Accepted RC06; pinned IDAS interpolation/event/effort contract; actual dependent consumer inventory | Combined boundary/interpolation/budget correction with all dynamic/workflow consumers and edge controls; remove sample-hard-stop path | done |
| D02 | Existing product validity; A04 record slice; B projection/product slice where changed | Actual integration purpose and complete same-point upgrades/reuse across fit/shooting/response/curvature; remove misleading aggregate-only interpretation | in progress |
| D03 | Q03 capability/accuracy/profile decision; C06 only for new native capability | Equivalent supported derivative/linear/corrector/checkpoint profiles or retained defaults, with correct work generations and eligibility | done |
| D04 | A04 endpoint meaning; actual workflow/result/publication owners | Complete qualification/withholding/readiness facts and consumers, without changing science or artifact authority | in progress |

<!-- Package state is owned in this table. -->

D01 is a combined correctness delivery, not independent mesh/budget/event edits with premature acceptance. D02/D04 can establish purpose/endpoint evidence under the existing driver; D03 alternatives need not wait for the entire diagnostic tool stack. Shared runtime/dynamic/fitting source requires coordinated editing.

## Verification and checkpoint

**Interface-checked — production completeness, 2026-10-17:** D01/D03 match the selected boundary/interpolation/cumulative-budget and retained-profile targets. D02 remains incomplete for same-point shooting adjoint purposes; D04 for joined scientific-product availability after authored refusal. **Tested, earlier 2026-10-17:** the Outcome below retains acceptance of the implemented scope and supported inquiry branches. [43g’s Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns assembled/scoped acceptance, repaired composite boundaries, scope-end checks and exclusions. The packet table is the native progress owner; formal proposed ADR status does not assert or replace implementation acceptance.

## Outcome (recorded after implementation)

### What was built

**Implemented — delivered scope; D02/D04 retain the production remainders above:** D01–D04 separate genuine event/reset/physical boundaries from covered output interpolation. Profile Version2 governs accepted advancement cumulatively for the complete forward trajectory, aggregate replay of a derivative operation and each complete backward functional/direction trajectory. Sampling, restores and reinitialization do not renew permission. Public Rust/Python/nested consumers migrate together and Version1 is explicitly rejected. Maintained IDAS guards operate at actual replay/per-memory advancement while NORMAL scheduling preserves covered output. Diffsol restores retain the actual producer and cumulative counters. Selected JVP/HVP products and admitted scratch serve actions; exact-point fit upgrades install complete products transactionally. Qualification, requested response/curvature, statistics withholding and readiness tails stay distinct.

**Tested, 2026-10-17:** the linked native 284-case selection includes independent trajectories, covered-output exhaustion, events/IC/sensitivity/quadrature, reset/sample order, replay/backward budgets, product actions and restored producer work. The integrated runtime selection passed 76/76, including nonlinear DAE sensitivities, terminal integrals, event routes, scaled actions, shooting/multiple shooting, exact/first covariance, failed original-fit withholding and horizon reuse. Native Python ordinary consumers and all three managed sixteen-owner/equal-occurrence/cancellation study journeys passed with cleanup; their composite boundary and exact commands are recorded in [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation).

**Interface-checked:** D03/Q03 retain IDAS Simultaneous/KLU, Diffsol BDF/faer and current checkpoint spacing250/max400. Actual alternative/corrector/linear/checkpoint controls establish supported behavior, not a representative latency winner. Exactly zero functionals/sample cotangents avoid reverse/jump work after original eligibility and scope checks. A valid rank-deficient profile result remains usable while unavailable statistical products are withheld.

### A mistake made and corrected

A restore previously replaced the numerical producer, losing continuity of cumulative work. It now preserves the actual producer and records restores separately. The runtime action fixture omitted the two required parameter scales; adding explicit scales preserved its numerical, derivative and atomic publication assertions. Python integration also exposed a displaced existing `accuracy_goals` property; restoring its decorator repaired the public regression.

### Deviations from the plan, deliberate

Retain IDAS NORMAL scheduling with per-memory guards; ONE_STEP advances all active backward directions and can destroy already covered interpolants while another direction catches up. No aggregate backward cap across independent directions, renewed segment budget, automatic solver winner or hidden finite-difference production path was introduced. Formal ADR-0194 remains proposed.
