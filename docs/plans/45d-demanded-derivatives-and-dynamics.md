---
title: "Plan45d: Demanded derivatives and dynamics"
status: in-progress
date: 2026-10-18
adrs: [ADR-0199]
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, numerical-execution]
---

# Plan45d: Demanded derivatives and dynamics

## Resulting behavior and baseline

This companion owns F12 and I05/I07/I08. The [coordinator](45-pipeline-execution-optimization.md) owns their outcomes and accepted RC07/RC10. **Interface-checked:** source at `22ae87c16` already separates selected forward directions from complete values, fuses a fresh final fitting product, preflights optional rank products and conditionally uses exact IDAS forward quadratures. Preserve those completed mechanisms. Non-Forward requests still obtain complete parameter gradients before consumer projection; checkpoint and native-method choices retain different work/storage lifetimes. The [native inquiry](../design_review/evidence/post-plan44-pipeline-optimization-2026-10-18/native-inquiry.md) records the exact-release contracts and limits. Everything below is **Proposed**, not a new runtime result.

The target carries consumed derivative/functional demand to native work without truncating physical values or the required state adjoint. Libraries own integration, nonlinear iterations and factors; workflows own declared objectives, schedules/anchors, route eligibility and independent original assessment. Conditional product selection happens before effects. Changing a physical trial always establishes its own native IVP validity; retained arithmetic workers are not retained multistep history.

## D01 — Consumed adjoint parameter-gradient coordinates

**F12/I05; accepted RC07.** Define one internal ordered adjoint parameter-gradient selection, distinct from forward directions, state adjoints and second-order directions. Ordinary public/full adjoints explicitly request all coordinates. An explicit empty selection requests no parameter-gradient coordinates; it is not full demand, unavailable evidence or a vector of fabricated zero derivatives.

Carry the selection through profile/preparation identity, admission, `workflow/integrated.rs` gradient/window operations, fitting/shooting demand, native parameter products/quadratures, initial corrections, direct dependence, scheduled interval mapping, anchor translation and returned layout. Diffsol's non-Forward parameter products and IDAS's backward quadrature sizes must follow this demand while model evaluation retains the full physical parameter vector and state adjoint. Select unique physical axes, preserve order/interval identity and accumulate tied/scaled contributions according to the consumer's declared mapping.

Inspect second-order callers before changing common gradient shapes. Migrate their selected first-order handling without relabeling Hessian directions/columns, or keep a distinct caller explicitly full where its actually consumed contract requires it. Hidden backend-only narrowing or unmarked full-then-project consumers are not F12 completion. Public derivative meaning is not broadened silently.

**Targeted acceptance:** both supported native backends; full/selected/empty demand; one free parameter plus zero/four nonzero coupled fixed inputs; direct and initial-state effects; tied/scaled mapping; one schedule and anchored window. Assert independent analytic/physical gradients, complete predictions and returned coordinate identity. Include invalid/duplicate axes, derivative failure, cancelled upgrade, retained weaker products and existing Hessian callers. Observe actual parameter-product/quadrature widths and allocations, not just returned columns. Keep opaque/factor allowances unless a separate sound bound qualifies.

## D02 — Exact functional products and route crossover

**I05; D01 supplies the selected-adjoint comparison; accepted RC10 for new composition.** Start from the implemented combined IDAS branch: already-forward consumers with integrals can use exact forward quadratures after every window preflights; integral-only or rejected preflight retains adjoints. Preserve lazy trial products, original assessment, independent final fitting checks and no fallback after effects begin.

Compare integral-only versus combined terminal/path-plus-integral consumers, one/two windows and independently varied consumed controls/integral channels. Keep physical objective, parameters and accuracy identical. Record actual forward/backward/replay/quadrature requests, demanded widths, callback work, rejected trials and simultaneous storage. Do not compare different objectives or turn parameter count alone into a global policy.

Investigate an exact weighted-functional binding when contracting the demanded objective before derivative work can remove complete channels/passes. Stock quadrature sensitivity shape follows primal quadrature shape, so buffer truncation alone cannot implement it. Include direct/initial/tied/scaled/scheduled/anchor terms and all ordinary integral values still required by other consumers.

Select and implement the supported exact route within the accepted bound: keep the existing conditional route, qualify a weighted functional, or make a consumer-specific explicit profile/product choice. A missing exact API retains the current supported route with its precise gap/trigger. Do not replace exact derivatives with differences, erase hybrid/reset refusals, retry another method after effects or adopt a universal crossover threshold.

**Targeted acceptance:** analytic objective/gradient for combined and integral-only consumers; all-window preflight refusal before integration; cancelled/failed stronger product leaving weaker evidence intact; schedule/anchor and direct dependence; actual avoided passes versus shifted storage. A supported no-change decision is legitimate, but an unrun cheap control is not evidence that long-duration work is needed.

## D03 — Dynamics method, stiffness and linear shape

**I07; class-specific native integration under current science.** Use the smallest model that separates smooth nonstiff behavior, sparse stiffness/coupling/fill, index-1 DAE consistency and event/reset/scheduled boundaries. Hold output density independently from internal solver work. Keep the existing matched 5/25-time case as a control rather than comparing it to a differently scheduled case.

Inspect actual rejection/setup/nonlinear/error-test reasons and derivative/JVP/preconditioner work. Qualify existing eligible explicit Diffsol methods for nonstiff models and direct/Krylov/preconditioned implicit routes where needed. A currently explicit IDAS block-factor preconditioner refusal is an integration gap to investigate only if the concrete workload makes it useful and exact native APIs can supply the complete callback/lifecycle contract. Adding a fitting library binding uses the short-ADR route before implementation; broader public/scientific support changes require a concrete decision.

Choose and integrate qualified explicit profiles or supported method/preconditioner integration where the observations and library fit warrant it. Preserve class eligibility and original outcome interpretation. A profile that wins a tiny case is not a new global default; a step or setup count from one backend is not interchangeable with another's. An unavailable route must refuse honestly before effects rather than silently selecting a weaker problem.

**Targeted acceptance:** analytic/reference or conservation/original checks for smooth and stiff ODEs, DAE consistent initialization, event/reset and scheduled side; actual native rejected/accepted steps and setup reasons, RHS/Jacobian/JVP work and resource/stop outcomes. Include unsupported derivative combinations and nonconvergent/domain-limited cases. Larger sparse shape is allowed to expose a mechanism; extending a trivial horizon solely for timing is not.

## D04 — Checkpoints, replay and scheduled adjoint lifetimes

**I08; accepted RC10 for changed binding/API.** Compare one/several scheduled segments, terminal-only versus nonzero interior observation weights and two checkpoint intervals at the same physical policy. Record separately forward recording, replay, backward integration, consistency initialization and simultaneous retained history. Both adapters already skip zero-weight jumps; this is a preservation control, not missing work. Nonzero jumps and algebraic consistency are legitimate work.

Compare Diffsol's per-segment retained solvers against IDAS replay of earlier segments as complete memory/work alternatives. Investigate the pinned checkpoint/reset-metadata gap and a narrow upstream API change only where it can remove host recorder/retained state machinery. Use exact Diffsol 0.16.2 and SUNDIALS 7.1.1 contracts; current or older documentation is discovery, not proof of an installed API.

Implement the qualified bounded outcome: an exact library API/adapter correction, a justified explicit checkpoint profile, a compact retained representation or supported retention/replay composition. Keep conservative native opaque bounds, library-owned replay and original initial/direct corrections. This package does not introduce general durable native-history continuation, cross-trial checkpoint reuse or new hybrid derivatives.

**Targeted acceptance:** sample exactly at a schedule boundary, interior observation jumps, consistent algebraic adjoint, initial correction, cancelled replay/backward work, memory refusal and final release. Verify the same physical objective/gradient independently; a smaller history alone is not a benefit if replay amplification dominates the complete operation. If the alternative does not qualify, retain the current route with a precise deciding premise and next trigger.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D01 | RC07 decision; current derivative/value/initial mapping | Distinct selected adjoint contract through both native adapters and actual fitting/shooting consumers; full/selected/empty and second-order controls | in-progress |
| D02 | Current combined quadrature controls; working D01 for selected comparison; RC10 before new functional binding | Crossover investigation, selected exact implementation and objective/gradient/preflight/failure tests | in-progress |
| D03 | Actual class/method eligibility and short case; decision route before a new binding | Stiffness/shape/method investigation and qualified integration/profile; original-state and native-work/refusal tests | in-progress |
| D04 | Existing checkpoint/adjoint contracts; RC10 before changed binding/API | Recording/replay/history investigation and qualified implementation or supported retained route; schedule/initial/stop/memory controls | in-progress |

## Verification and checkpoint

**Planned:** touched runtime/native compile checks and targeted force-validated integration/fitting/shooting units. Existing exact initial/quadrature/anchor/derivative tests retain their meaningful thresholds; ordinary simulation accuracy is not tightened to make a performance comparison. Final public projection/replay, durable framing if actually changed, integrated native/resource journeys and static checks belong to 45f.

All four packages are open, with no new numerical observations at authoring. Current-route characterizations can proceed independently of D01; selected-adjoint comparisons require its real migrated implementation. Shared `idas.rs`, integrator and runtime integrated/shooting files require one writer despite logically independent questions. The coordinator records each inquiry outcome and F12 closure separately.

Current execution checkpoint: D01 is integrated through both native backends and fitting/shooting consumers. Its selected native suite passed 28/28 and the runtime gradient/schedule/fitting/shooting selection passed 10/10 against zero failures. Known selected buffers are admitted in addition to unchanged state/factor/opaque allowances. The new IDAS test uses the existing adjoint oracle threshold; no scientific profile accuracy changed. D02–D04 short contrasts and selected outcomes remain open; final assembled acceptance remains 45f.
