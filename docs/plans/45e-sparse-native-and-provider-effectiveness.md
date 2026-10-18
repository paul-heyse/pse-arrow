---
title: "Plan45e: Sparse native and provider effectiveness"
status: in-progress
date: 2026-10-18
adrs: [ADR-0200]
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
| E01 | Exact sparse/native controls; RC08 before new binding | Meaningful repeated sparse comparison and selected qualified implementation/retained limitation; fresh-numeric and failure/drain tests | done |
| E02 | Actual repeated class consumer and exact library model/update contract | Supported class-specific integration/profile changes or precise no-change; original objective/feasibility/integrality/certificate tests | done |
| E03 | Existing valid property/implicit model and requested derivative eligibility | Provider work diagnosis and supported implementation; value/derivative/branch/stateful/stop controls | done |
| E04 | Same declared recycle map/tear/start and explicit numerical controls | Recycle mechanism comparison and supported implementation/profile; original closure/nonconvergence/drain tests | done |
| E05 | Accepted RC08; representative short stock screen; prerequisite native contract | Qualify simpler fixed-pattern private KLU lifetime with fresh numeric factors and real implicit-cache caller, or retain unresolved limitation with no unused candidate | in-progress |

## E05 — Reopened fixed-pattern sparse qualification

Screen stock KLU on 4,096/band-four, 64 units × eight variables with open/recycle coupling, and a 32×32 network. Use fixed support, changed coefficients, nonuniform manufactured roots and ordinary accuracy. Grow only an attribution-unresolved shape. A useful complete-operation case enables a dedicated private fixed-pattern KLU owner: freeze structure/settings, invalidate numeric state before every application including accepted starts, retain only valid symbolic state, full-factor current coefficients at first setup and preserve native within-call recovery. No copied CSC key or per-substitution scan. Qualify stock/eligible Krylov and the candidate with actual phase counts, original checks, failed setup/recovery/drain/accounting and the real implicit Session cache. No new public solver variant or universal default is selected. Remove any unadopted experiment; F13 remains unresolved if useful complete benefit is unestablished.

Current checkpoint: the independent prerequisite review accepted the target and ADR-0200/blueprint revision 204 install it before implementation. The short stock screen passed all 30 changed-coefficient observations (one test, 0.13 seconds execution, baseline zero). At 4,096 band-four variables, complete replacement/solve was 8.64–8.83 ms with 0.536–0.598 ms symbolic work; at 32×32 grid it was 2.63–2.78 ms with 0.270–0.284 ms symbolic work. Every stock KLU application full-factored once and the original residual checks passed at ordinary 0.001 accuracy. This supplies a useful candidate qualification case, not dominance or an adoption/speedup claim. Implementation of the narrower private owner is released. The first launch compiled but stopped before tests at the unrelated default registry migration fence; the successful run used a fresh explicitly selected private test-resource registry without changing host admission or a canonical database.

### E05 targeted outcome and selected adoption

**Implemented / Tested, baseline zero:** the dedicated fixed-pattern native owner is now the internal realization for existing KLU Sessions. Generic KLU remains unchanged for supported direct use and test comparison; no new public profile or global method default was added. The native setup still restores canonical CSC, fully factors current coefficients on every nonaccepted application, and leaves refactor/condition monitoring to KLU. Constant-time validity/object guards replace the previous experiment's copied key and per-substitution scans. Accepted starts and failed native/original checks release numerical factors; drained reset cancellations can retain the private owner, while uncertain native integrity cannot enter the implicit cache. Scalar accounting exposes no mutable native Common pointer.

The targeted ten-control selection passed with zero failures through `PSE_TEST_RESOURCE_STATE=<fresh private registry> just unit-native-capability-package pse-backend-native pse-backend-native/kinsol klu <explicit ten-test filter> --no-capture` (0.279 seconds execution; `/tmp/pse45-native-followup-targeted.log`). It covers the complete comparison, fixed-owner mutation/refusal/recovery, actual sixteen-variable implicit parameter cache, accepted starts, exact support replacement, counters/hooks, budget release, cancellation reuse, original-validation failure and recovered trial success. The independent implementation review at `17b2179a0` closed its sole original-validation cleanup finding; it found no other material issue in the inspected native C/Rust scope. Combined acceptance remains 45f/F05.

**Measured, one short local observation per application:** for three changed-coefficient 1,024-variable affine grid solves after initial setup, stock complete replacement/solve was 2.606/2.823/2.855 ms and fixed-pattern was 2.287/2.256/2.337 ms. Both retained 1,066,696 bytes. The nonlinear grid was 3.650/3.847/3.893 ms versus 3.366/3.312/3.639 ms, with identical original residuals and corresponding numeric/refactor/substitution work. Later fixed-pattern applications issued zero new symbolic analyses and one full numeric factorization each. Initial applications still analyze; Session construction precedes these clocks. The band and explicit SPGMR/Jacobi contrasts also passed. These scoped observations plus simpler ownership justify adoption under RC08; they are not statistical rankings, a universal speedup or a default Krylov decision.

The first candidate selection had eight passes and one setup failure: the sixteen-variable implicit fixture declared only 100 derivative entries. It now declares its actual first-derivative extent of 16×17=272, without weakening the production guard. A separate compile error in diagnostic string borrowing was repaired. Implementation review found native success could coexist with failed original validation; cleanup now consumes actual original quality/failure evidence and preserves recovered trials and original error causes.

## Verification and checkpoint

**Planned:** touched native/math/runtime compile checks and targeted force-validated units through existing recipes. Source feasibility is Interface-checked; production integration is Implemented; actual scoped checks are Tested; work/time/capacity are Measured only under their named conditions. A successful test body with failed enclosing drain is not assembled acceptance. 45f owns final selected native/public/resource integration and quality after functional scope.

E01–E04 have bounded targeted outcomes; assembled qualification remains with 45f. No new source/library/runtime qualification was performed merely to write this plan; existing exact evidence supplies the initial branches. E01's potentially retained F13 limitation stays separate from investigation completion. E02–E04 preserve established mechanisms where the experiment finds no better sound complete realization.

Current execution checkpoint: proposed ADR-0200 and blueprint revision 199 preceded the experimental symbolic-retaining KLU binding. Exact compatibility, fresh-numeric transitions, failure/recovery, accepted starts, admission and private-owner release were tested and independently inspected. The short 128/512-variable comparisons did not establish material complete-operation benefit sufficient to justify the extra state and bundled adapter maintenance, so RC08 selects stock KLU and the existing explicit Krylov/Jacobi route. The experimental production path is removed; experiment commits remain in Git history. I06 is settled, while F13 remains unresolved with the native sparse solver owner and a representative symbolic-dominant workload as its revisit trigger. Proposed ADR-0200 and blueprint revision 200 now describe this non-adoption, without changing formal ADR status. E02 retains the current HiGHS bulk/basis/QP and SCIP reoptimization mechanisms. The real SOC contrast preserves original analytic cone/bound/objective checks: default equilibration rebuilt four times, while explicit unscaled reuse used one setup and three updates, with four solves on each route. The latter is conditional on the existing conditioning/presolve/chordal/structural eligibility; no global unscaled default is selected. Fixed-assignment MINLP tests now declare their actual Ipopt/Pounce prerequisite rather than expecting a successful local continuous subsolve in an unavailable feature graph. E03's real local Peng–Robinson controls passed the scoped independent derivative and operational-effect checks below; this does not claim full selected-regime/reference-model qualification. E04 passed the actual three-coordinate recycle-map contrast with the optional orthogonalization field integrated and generated. Assembled/static acceptance remains open.

E03's real local Peng–Robinson controls passed independent Value/First/Second and weighted-product checks at nontrivial points, point/configuration/branch/order/stop/private-worker controls, and actual factor/RHS work counts. The inquiry exposed coordinator N01 and implemented provider-effect-aware selector reuse eligibility. Pure selectors retain fast reuse; current provider-backed anchors/restrictions cannot be skipped. The combined nine-control optional compiled-evaluator selection passed with zero failures. This does not claim qualification of the broader 120×120 selected-regime reference model. Accelerator acceptance and Nested original assessment remain separate: the inner solver returns a vector, not an issuer-owned verified-result permission that would allow either assessment to disappear.

E04 preserves the declared map, zero start and original physical closure across depth 0, depth 1/3 MGS, delayed/damped depth 3 and depth 3 ICWY. On the slow-contracting three-coordinate map (rates 0.93/0.94/0.95), map evaluations were respectively 78/7/5/6/5, with every coordinate meeting the same original feasibility checks. Explicit orthogonalization without nonzero history refuses before preparation. The optional public field defaults to the prior MGS behavior; no global depth, damping, delay or orthogonalization winner is selected. The initial scalar test could not admit depth 3 and was corrected to three nonredundant coordinates, preserving equal inputs across profiles rather than weakening native admission. The corrected control and actual shooting grid passed in the same linked runtime run; the unrelated physical-cache test failure remains owned by 45c/45f.

## Outcome (recorded after implementation)

### What was built

**Implemented / targeted Tested:** Provider-effect-aware reuse executes current operational anchors/restrictions even when a numerical jet is reusable. Optional public recycle orthogonalization preserves the previous MGS default. Conditional unscaled conic updates use existing exact eligibility; current HiGHS/SCIP mechanisms remain. The preceding package verification records targeted conditions; [45f](45f-short-run-qualification-and-publication.md#verification-and-checkpoint) records assembled acceptance and the shared scope-end checks.

### A mistake made and corrected

The first recycle contrast used one scalar coordinate while requesting depth three. It was corrected to three nonredundant coordinates with identical maps/starts across profiles. The provider inquiry also exposed and corrected N01 rather than treating same-point numerical reuse as permission to skip operational effects.

### Deviations from the plan, deliberate

The qualified experimental symbolic-retaining KLU binding did not establish material complete-operation benefit on the short 128/512-variable contrasts and was deleted. Stock KLU and explicit Krylov/Jacobi remain; F13 is unresolved under RC08. No global solver, equilibration, recycle or checkpoint default changed, and local Peng–Robinson controls do not qualify the full reference-model regime.
