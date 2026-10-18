---
title: "Plan45e: Sparse native and provider effectiveness"
status: draft
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, numerical-execution]
---

# Plan45e: Sparse native and provider effectiveness

## Target and baseline

This companion owns F13/I06 and I10; the [coordinator](45-pipeline-execution-optimization.md) owns their decisions and accepted RC08. [45d](45d-demanded-derivatives-and-dynamics.md) owns dynamics/adjoint composition, while this document owns repeated sparse preparation, other numerical classes, property/implicit callbacks and recycle effectiveness. Outcomes include qualified implementation, not merely an experimental report. No universal backend ranking, new default or smaller opaque allowance is presumed.

**Interface-checked:** source at `22ae87c16` and exact SUNDIALS 7.1.1 establish that nontrivial KINSOL applications initialize KLU and repeat symbolic analysis. Allocation/layout reuse and accepted starts mean different things; accepted exact starts can bypass setup. Plan44 observed symbolic intervals of roughly 1.8–10.7 microseconds on tiny cases, including hooks. This does not warrant another maintained binding by itself. Compatible HiGHS numeric updates/basis retention, same-point provider reuse with stateful barriers and explicit recycle controls already exist. All changes/experiments below are **Proposed**.

## E01 — Repeated sparse preparation and qualified alternatives

**F13/I06; accepted RC08 for a new ownership contract.** Keep stock KLU in production while comparing one repeated sparse system under changed numeric coefficients, unchanged CSC structure and complete symbolic-analysis settings. Increase dimension/coupling/fill only until the preparation question is visible within short runs. Compare stock direct KLU, a narrowly owned symbolic-retaining `SUNLinearSolver` candidate and an existing eligible Krylov/preconditioned route. KINSOL continues to own nonlinear iteration/globalization and within-call setup policy.

A symbolic-retaining candidate owns an exact structure/settings identity, library symbolic state and explicit **fresh numerical validity** per application. Initialization may retain symbolic state only under complete eligible premises and must invalidate numerical state. Changed pattern/settings, numeric pivot/refactor failure, singular recovery, release, cancellation and actual drain all have explicit outcomes. `KLUReInit` frees both symbolic and numeric state and is not the remedy. Do not suppress initialization, trust old factors, reimplement generic nonlinear iteration or transplant factors into the separate faer IFT problem.

Count issued/completed symbolic, full numeric, refactor and substitution work separately, with actual native status and original physical residual checks. Compare complete preparation/execution, simultaneous storage and integration/upgrade burden, not a faster isolated symbolic interval. An alternative requiring new generic machinery without material benefit does not qualify merely because it compiles.

**Implementation branches:** adopt a qualified symbolic owner with explicit selection/eligibility, accounting and recovery after its decision/short ADR; integrate a supported existing Krylov/profile choice where it is the better complete alternative; or retain stock with its measured limitation and observable representative-work trigger. Conditional approval is not a global default decision. If stock is retained, I06 can settle, but F13 remains explicitly unresolved/deferred rather than fixed.

**Targeted acceptance:** accepted versus ordinary start; fixed structure/changed coefficients; changed ordering/all analysis settings/pattern; singular/refactor failure then recovery; pressure, cancellation, repeated release and current physical residuals. Actual symbolic reuse and fresh numeric setup are separate assertions. Cold/new-layout controls cannot masquerade as compatible repeated calls.

## E02 — Other numerical classes and native model-management capabilities

**I10 class branch.** Start with real repeated coefficient/bound/objective updates in LP/QP, one cone consumer and a small factorable mixed-integer problem. Inspect the actual existing owner and its consumed API with the native-solver-libraries skill; use additional library research only for a concrete capability gap. Exact dependency releases and patched/vendor behavior govern adoption.

HiGHS compatible updates and basis/allocation retention already exist; test whether actual repeated consumers use them correctly rather than proposing a new warm-start mechanism. Distinguish LP/convex QP support from MIQP/nonconvex QP. For Clarabel, compare useful update reuse against presolve/chordal/structural-zero restrictions; an incompatible structure rebuilds or refuses explicitly, never reuses stale state. For SCIP, retain factorable expression/bound visibility, original integrality, feasible candidate, global bound/certificate and termination meanings. Optional PETSc/Uno/JVP/preconditioner routes are eligible where a named class/workload demonstrates the need.

Choose and implement supported library/model-management or explicit-profile improvements in real consumers. Preserve optimizer visibility rather than flattening a rich native model through a neutral wrapper that loses its capabilities. New native bindings follow the required decision route; an unknown API or a small timing win does not authorize broader scientific support. Retained-current results name the deciding restriction and useful next trigger.

**Targeted acceptance:** original objective/feasibility/integrality/certificate checks, compatible update versus changed structure, stale state invalidation, infeasible/limited outcomes and native release. Record basis/presolve/setup/update work at the actual owner. Tiny cross-class times cannot choose a universal solver.

## E03 — Actual property/implicit provider work

**I10 property branch; consumes 45b products where actually affected.** Select an existing real property-heavy scalar/implicit model with a qualified requested derivative, not solely an artificial scalar callback. The existing reference/property fixtures and provider capability skills supply leads; do not copy external reference implementation code. Separate fresh source admission from prepared property evaluation, nested solve, factorization, IFT derivative and original verification work.

Inspect actual same-point/weight/point/state invalidation and library-owned bulk/weighted/derivative capabilities. Preserve Plan44's existing immutable reuse and stateful barriers. Qualify an optimization only under the complete provider/branch/derivative-order contract: new points or weights and operational anchors cannot inherit stale products. The guarded selected implicit function remains distinct from an unselected residual root relation.

Implement a supported provider/product/composition correction where the observations establish unnecessary work and the library can supply its consumed contract. A missing exact derivative must be identified by its actual capability gap; do not silently substitute finite differences, fabricate zeros or count a refusal as a successful derivative solve. If a fixture lacks required science, construct the smallest already-supported valid control or record the exact blocking capability without expanding into an unapproved property-family reimplementation.

**Targeted acceptance:** independent physical values and derivative/backward-error checks at nontrivial points, changed weights/point/provider configuration, stateful provider isolation, selected-branch domain failure, unsupported order, cancelled nested work and final release. Retain sparse IFT support/finiteness/verification work already corrected by Plan44. A full reference reload and a retained prepared callback are different measurement scopes.

## E04 — Recycle numerical effectiveness

**I10 recycle branch.** Use a small slow-contracting declared recycle/tear model with the same original fixed-point solution, physical checks and starting conditions. Compare existing acceleration depth, damping and orthogonalization controls one relevant dimension at a time. KINSOL's Newton setup interval is not a recycle tuning axis, and topology alone is not dependency/tear order.

Observe actual map evaluations, iterations, convergence history, acceleration/restart decisions, provider/native cost and original closure. Distinguish a useful accelerated route from extra history/orthogonalization that costs more for cheap maps. Retain explicit initialization/tear policies, branch and start permissions; no hidden automatic search or universal new default is introduced.

Integrate a supported explicit profile or fitting library-owned improvement in the actual recycle consumer if the short mechanism contrast warrants it. Otherwise retain the current mechanism with its specific effect/limit recorded. A near-stagnating case may justify a later different workload; a long run without new causal information is not required.

**Targeted acceptance:** independently known fixed point and original residual/closure, slow and ordinary contraction controls, nonconvergent/domain-limited map, cancellation/restart and resource release. Compare maps only when their definitions are identical; unrelated map timings or iteration counts are not a valid acceleration comparison.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| E01 | Exact sparse/native controls; RC08 before new binding | Meaningful repeated sparse comparison and selected qualified implementation/retained limitation; fresh-numeric and failure/drain tests | open |
| E02 | Actual repeated class consumer and exact library model/update contract | Supported class-specific integration/profile changes or precise no-change; original objective/feasibility/integrality/certificate tests | open |
| E03 | Existing valid property/implicit model and requested derivative eligibility | Provider work diagnosis and supported implementation; value/derivative/branch/stateful/stop controls | open |
| E04 | Same declared recycle map/tear/start and explicit numerical controls | Recycle mechanism comparison and supported implementation/profile; original closure/nonconvergence/drain tests | open |

## Verification and checkpoint

**Planned:** touched native/math/runtime compile checks and targeted force-validated units through existing recipes. Source feasibility is Interface-checked; production integration is Implemented; actual scoped checks are Tested; work/time/capacity are Measured only under their named conditions. A successful test body with failed enclosing drain is not assembled acceptance. 45f owns final selected native/public/resource integration and quality after functional scope.

All four packages remain open. No new source/library/runtime qualification was performed merely to write this plan; existing exact evidence supplies the initial branches. E01's potentially retained F13 limitation stays separate from investigation completion. E02–E04 preserve established mechanisms where the experiment finds no better sound complete realization.
