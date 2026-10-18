---
title: Sparse verification and native effectiveness
status: done
date: 2026-10-17
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md]
doc_owner: docs/plans/44-pipeline-performance-optimization.md
doc_topics: [current-work, numerical-execution, libraries, design, validation]
---

# Plan 44d: Sparse verification and native effectiveness

## Purpose and foundations

This companion owns F05/F06 and O04/O05. [Plan44](44-pipeline-performance-optimization.md) owns dispositions; [44e](44e-short-run-investigation-and-optimization.md) supplies short mechanism comparisons. **At authoring — Proposed:** all work and performance benefits below; the specified-change checkpoint records subsequent implementation, without a measured speedup claim.

The existing math owner extracts sparse implicit Jacobians, shares factors across RHS products and independently verifies derivatives. Native KINSOL owns nonlinear iteration/setup; its adapter owns compatibility/transport. Symbolica/providers own derivative products with intentional stateful projection barriers. Preserve those boundaries rather than moving nonlinear iteration into the host, treating all reuse as one cache or importing a factor across unrelated problems.

Use the native-solver and Symbolica/faer library skills, current Context7 documentation when resolving library questions and exact pinned implementation contracts. The review source chain establishes existing repeated work, not a qualified remedy. No stale utilization/catalog record is an adoption premise.

## D01 — Sparse implicit backward-error verification

Replace `pse-math/src/implicit.rs::check_solve`'s dense traversal of structurally absent entries with sparse action from the already extracted support. Compare deterministic support traversal with pinned faer sparse/dense multiplication for the consumed first and packed-second RHS shapes; choose the simplest conforming implementation. Preserve independent backward-error assessment and native factor sharing rather than reusing the factor's success as proof of a valid derivative.

The verification scale remains the sum of **absolute contributions**, `Σ |a_ij x_j|`, plus the existing RHS contribution—not `|Σ a_ij x_j|`. Preserve explicit Jacobian/solution/RHS finiteness so skipping zero structure cannot hide nonfinite solved coordinates. Keep physical scaling, singularity/domain refusal and current tolerances. This optimization does not remove the separately dense residual jet or invent a sparse derivative engine.

Obtain an eligible diagonal/banded multi-unknown implicit fixture with explicit branch/first-derivative admission. The review's four/eight-unknown refusals are retained negative controls, not scaling measurements. Use 4/8 eligible unknowns first and increase only if the distinction remains too small to inspect; count support visits/RHS and actual factors/solves rather than requiring long timing runs.

Acceptance includes analytic first/second derivatives, packed RHS, cancellation-prone sums, singularity, nonfinite coordinates and unchanged negative capability controls. Assert the new traversal follows nonzeros and RHS where claimed, while original backward-error refusal still detects a bad solve. Remove the dense verifier loop and its obsolete implementation-specific tests after replacement controls pass.

## D02 — Retained KINSOL symbolic versus numeric state

Inspect the pinned `KINSol` → linear initialization → `SUNLinSolInitialize_KLU`/setup chain together with the adapter's complete pattern/ordering identity. Measure actual symbolic analysis, numerical factor/refactor and solve populations across retained applications with changed coefficients, equal pattern, changed pattern/ordering, accepted primal start and failed factorization. A zero-setup accepted start is valid; exclude it as proof of amortized sparse setup.

Investigate a supported native capability first. If none supplies the required distinction, assess a narrow maintained linear-solver integration against its lifecycle/upgrade burden and the short observed symbolic cost. Exact pattern/ordering compatibility may permit symbolic reuse; each new application still requires fresh numerical setup. Do not suppress KINSOL initialization, reuse stale numeric factors, transfer KLU state to the separate faer IFT problem or implement a host nonlinear loop.

Select an improvement only with a concrete safe native contract and meaningful avoided structural work. If no sound route is justified, retain current initialization, document the limitation and give F06 an honest unresolved/deferred disposition with its owner and observable trigger; investigation completion is not resolution of the repeated-work finding. A proposed native binding/kernel contract or accepted-decision change follows its actual prior decision route.

Acceptance for a selected route covers changed coefficients/pattern/ordering, new application versus within-application setup, singular/refactor failure, cancellation/drain and repeated release. Compare original roots, native status and independent qualification at unchanged accuracy. No factorization speedup is inferred merely from allocation reuse.

## D03 — Provider weighted-product reuse under explicit premises

Investigate O04 using the current same-point projection/weighted assembly paths and intentional provider barriers. Separate complete immutable-provider jets from stateful providers whose execution order, context, guard or failure can matter. A changed multiplier may recombine a complete same-point immutable jet; that premise cannot be inferred from a pure-looking function name or from flat Symbolica lowering.

Compare changed weights, changed point, derivative order, guard/domain/context changes and stateful provider calls. Reuse must preserve source/failure attribution, aliases, physical coordinates and all demanded products. Prefer an existing qualified immutable premise or narrow explicit capability over a general cache/classifier. Preserve stateful barriers and failed-upgrade behavior; if the premise is unavailable, retaining current execution is a supported outcome.

Acceptance checks independent weighted derivatives/HVPs, call counts under the declared premise and a stateful-provider counterexample that still re-executes. Do not require a new immutable provider API without an actual supported consumer. Any new semantic/public contract is a concrete decision item before adoption, not implied by the review's question.

## D04 — Conditional numerical profiles at practical accuracy

Use existing native controls/profiles to compare the observed KINSOL setup tradeoff: default scalar work of 16 function/2 Jacobian evaluations versus interval-1 work of 5/4. Those numbers are case observations, not universal acceptance counts or permission to change every default.

Run short roots and recycles with cheap/expensive residuals and derivatives, diagonal/coupled sparse factors, cold/repeated preparation and ordinary/accepted starts. Record trial recoveries, residual trajectory, domain rejections, setup/symbolic/numeric/solve counts and attributed cost where available. A reduction in callbacks can still cost more factorization work. Preserve one native iteration/globalization owner and actual original-space convergence.

Select only a concrete conditional profile whose inputs and behavior are understandable through existing settings. Keep conservative defaults when evidence does not distinguish a route. Do not tighten solve accuracy to manufacture solver differences, introduce a bespoke predictive routing model or rank unrelated classes by one toy elapsed value. If short controls cannot settle a stiffness/scale-dependent profile, hand that exact question to the later review without adopting a guessed winner.

Acceptance tests selected profile resolution and original answers/refusals, including a revealing adverse case for the proposed profile. Remove superseded duplicate policy only when all its consumers move. A justified no-default-change result still documents the work tradeoff and follow-up trigger.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D01 | Existing sparse extraction/backward-error contract; eligible implicit fixture | Sparse verification with exact contribution scale and finiteness; analytic/singular/packed tests; delete dense traversal | done |
| D02 | Exact pinned native source; E03 retained sparse application controls | Supported symbolic/numeric reuse decision and selected implementation, or explicit unresolved F06 limitation; fresh numeric setup retained | done |
| D03 | Existing provider projection premises; E03 weighted-product controls | Safe narrowly qualified reuse or retained barrier conclusion; stateful/guard/failure counterexamples | done |
| D04 | Existing numerical profiles; E03 work/cost controls | Conditional profile decision and selected implementation or supported default retention; unchanged scientific accuracy | done |

## Verification and checkpoint

Source-established work growth warrants D01 without a large benchmark. D02–D04 need their respective compatibility/consumer premises before changes; a library API name alone is insufficient. Targeted mechanism tests precede deletion. [44f](44f-integrated-acceptance-and-follow-up.md) owns assembled native/scientific and final quality checks. At authoring all D packages were open; no new factor, provider or solver policy is implemented at authoring.

## Historical specified-change checkpoint

**Implemented and targeted Tested:** implicit derivative verification traverses the actual CSC support for each first/packed-second RHS, using O(n) row accumulators, deterministic column order, RHS-inclusive absolute contribution scale and the existing tolerance/floor. Matrix, solved/RHS coordinates, residual and absolute-scale finiteness are checked; a cancelling action with an overflowing absolute scale is refused. The affine shared caller uses the same sparse verifier. Performing-owner `implicit.derivatives.verification.support_visits` reports actual contributions. Diagonal/banded, packed RHS, cancellation-scale, singular/bad solve, nonfinite/overflow and end-to-end first/second derivative controls pass. D02–D04 remain unrun; no KLU symbolic-reuse, weighted-provider cache or solver-profile winner is selected.

The coordinator owns command/result totals and the enclosing acceptance boundary. These package results do not close the 44e campaign or 44f qualification.

## Investigation outcomes

**Interface-checked and Tested — D02:** pinned SUNDIALS 7.1.1 returns an accepted initial guess before linear initialization, but each nontrivial application initializes KLU and marks a fresh symbolic analysis/numeric factorization. `KLUReInit` frees both; changing ordering and the mutable Common analysis settings prevents a generic silent retention patch from having a complete premise. The retained-layout/coefficient/ordering controls, existing singular/refactor/cancellation controls and accepted-start contrasts passed in the coordinator's native selection. Fresh numerical setup remains mandatory. No native integration change was selected.

**Measured, once-only owner contrasts:** 24 four/eight-dimensional diagonal/coupled cubic, actual exponential-sum and logarithmic root profiles at 0.001, plus 24 accepted-start controls, recorded actual callback work and synchronous symbolic/numeric/refactor/substitution wall intervals. Symbolic intervals in this small selection were about 1.8–10.7 microseconds, including hook overhead; they are neither exclusive CPU nor a large-process estimate. This does not justify a maintained replacement linear owner for the short campaign. F06 remains **deferred and unresolved**, with D02's owner/CSC/analysis-settings experiment in 44f's follow-up. Never report it resolved or treat allocation reuse as valid factors.

**Tested — D03:** changed weights/points, order upgrades, late provider refusal, context and cancellation controls preserve the generic stateful provider barrier. Existing nested providers already reuse complete same-point jets under their explicit fixed-configuration, coordinate/order/accuracy premise; dynamic hint resolvers deliberately reexecute in the original refusal order. No current generic ProviderFactory contract establishes evaluation transparency. Retain both the performing provider's qualified reuse and the outer call barrier; no new cache or speculative purity capability was added.

**Tested and Measured — D04:** the root contrasts and six ordinary/slow/oscillatory recycle profiles retained original qualification at 0.001. Interval 1 can save many residual calls while adding Jacobian/refactor work: the four-dimensional uncoupled exponential fixture made 5 residual/3 Jacobian oracle calls versus 13/2 at interval 10, with 3 versus 2 setups. Oracle counts include actual adapter/original calls; they are not interchangeable with native getter populations. The slow recycle used 130 versus 40 iterations with Anderson depth 0 versus 2; other maps had different work. Recycles do not consume the Newton setup interval. Retain existing explicit controls and the interval-10 default: neither these small equations nor their timings establish a general derivative/factor-cost discriminator or universal acceleration winner. A later representative workload may select an explicit existing profile; no host iteration/routing heuristic was introduced.

## Outcome

**Implemented and Tested:** sparse first/packed-second IFT verification visits actual support with the unchanged backward-error and finiteness safeguards. Native/provider/profile mechanism controls passed at baseline zero; the root/recycle contrasts above are once-only observations at ordinary 0.001 engineering accuracy. [44f](44f-integrated-acceptance-and-follow-up.md#assembled-acceptance-checkpoint) owns assembled commands and quality.

**Mistake corrected:** allocation reuse and an accepted initial start could be mistaken for symbolic-factor reuse. Exact source and actual symbolic/numeric counters distinguish them: accepted starts can bypass setup; nontrivial calls still repeat symbolic work. **Deliberate decisions:** retain fresh numeric initialization, stateful provider barriers, explicit profiles and conservative defaults. F06 remains deferred and unresolved with its material-cost trigger and concrete next experiment; finishing D02 does not resolve it. No unsupported symbolic-retention patch or universal profile winner was adopted.
