---
title: Native methods, reuse and matrix boundaries
status: done
date: 2026-10-17
adrs: []
review_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, libraries, validation]
---

# Plan 43c: Native methods, reuse and matrix boundaries

## Purpose and foundations

This companion owns F04, native F02/F07 contributors and O01/O02/O04 native work in [Plan43](43-solver-diagnostics-and-performance.md). It develops Q02/Q03/Q04 with explicit decision branches. Rule recommendations RC01–RC11 were accepted as written on 2026-10-17; acceptance of a bounded investigation does not select its unknown answer. At document authoring, product implementation was **Proposed**; the Outcome records delivered scope and evidence.

**Interface-checked:** current callback/TNLP/presolve/sequence, structural/routing, native settings, conditioning and backend extraction owners plus the review's exact-version source. Numerical iteration, globalization, factorization and fitting model management remain library-owned. Typed policy, original assessment, charge ownership, panic containment and native drain are suitable foundations.

Preserve existing progress/factor evidence while correcting the gaps: POUNCE detailed timer values need activation-aware availability, optional histories need bounded integration, some native internals are genuinely opaque, and retained native objects do not establish compatible numeric state. Callback copying is distinct from bridge construction work. `coo()`/`std::set` runs at Ipopt sequence creation and is already amortized by reuse; PETSc already preallocates row capacity.

## Target packages

<a id="c01"></a>

### C01 — Truthful native availability and ordinary progress

Use A01's producer/availability contract to project actual backend facts. First correct POUNCE disabled timing: unavailable/not requested/running/interrupted/absent are distinct from measured zero. Keep existing intermediate iteration callbacks independent from optional native statistics history. Ipopt's structured terminal availability is a useful preservation model, not a universal field mapping.

Wire supported coherent checkpoints into A02 across the actual numerical families: NLP restoration/ordinary iterations, roots/fixed-point maps, dynamics, HiGHS LP/QP/MIP, Clarabel cones and SCIP search. Preserve method-specific iteration, bound/gap, certificate, internal/output time and actual backend meanings. A foreign getter may run only at an owner-safe callback/return. No concurrent private native reads or statistics inferred from Jacobian count.

Migrate terminal reports, live/durable progress, inspection/timeline, benchmark and generated Rust/Python readers. Ordinary native facts require no detailed collector or arbitrary option passthrough. Cover disabled/enabled/running/interrupted/absent facts, failed factors and initialization failures; no optional observer failure changes scientific permission.

<a id="c02"></a>

### C02 — Prepared transport, bulk publication and valid witnesses

Give foreign callbacks/TNLP/presolve contexts reusable point/multiplier/derivative scratch and immutable coordinate/addend maps. The evaluation owner fills complete scratch, normalization/accumulation uses its checked map, then native output is published once. A final copy may be necessary to guarantee untouched output on failure. Preserve signed/objective conventions, physical/native scales, duplicated addends and finite representability; fused scales must not introduce overflow by changing a checked sequence.

Cover AlgebraicOracle fill interfaces, Ipopt/Uno/POUNCE callbacks, presolve Transport and PETSc vector/matrix publication. Refill becomes permutation only when bijective; otherwise accumulation remains. Prefer supported bulk insertion over one PETSc value call per nonzero, using its existing row-capacity preallocation. Do not hide backend bulk opportunities behind an unnecessarily scalar neutral adapter.

Replace bridge construction set checks only with a complete checked canonicality/range/triangle contract that protects arbitrary foreign callers. Retained sequence reuse already amortizes the construction check; do not report it as repeated callback work. The transformed row/column/support inventory needs its own complete structural witness or fresh analysis. Blind original witness forwarding is not reuse.

Targeted controls include transformed permutations/aliases, duplicated contributions, nontrivial scaling/objective sign, changed multipliers, malformed bridge indices, recoverable domain refusal, panic/nonfinite output, and no partial publication. Keep transport versus oracle versus construction observations separate under A01.

<a id="c03"></a>

### C03 — Compatible native and conditioning reuse

Settle Q02 with a validity matrix for retained allocation, symbolic pattern/order, numeric factor/preconditioner, fixed/equality/bound classifications, scaling, starts and policy. Include original problem/configuration/value identities, failed predecessor and reset state. Numerical setup history is not implied by a pool checkout.

Ipopt same-structure reuses classification and scaling-related state beyond nnz; KINSOL no-initial-setup may retain a numeric Jacobian/preconditioner. If the native owner supports symbolic reuse with fresh numerical initialization, prefer that narrower fitting contract. Stronger state retention needs a concrete decision establishing changed-value/start/classification/scaling behavior. Until then retain fresh numeric setup and current allocation reuse.

Direct FERAL conditioning may retain compatible symbolic preparation while each current candidate receives a fresh numeric factor and residual/inertia assessment. Reuse its actual owner lifetime, not a new global universal cache. Never hand KINSOL/KLU factors into faer IFT merely because a root converged; B05's optional handoff requires current point/Jacobian/layout/scaling and derivative residual guarantees.

Migrate sequence/KINSOL/session cache, conditioning, KKT/regular-square response and explicit failure-diagnostic consumers. Record actual reuse, incompatibility/rebuild, eviction and retained-byte facts at the performing owner. Cache observation does not initiate work.

Acceptance includes unchanged pattern with changed values, fixed/equality/bound classification changes, start/scaling/policy changes, singular/nonsingular transitions, failed predecessor, cold-after-warm, RequireReuse refusal, cancelled/failed reset and resource release only after native drain. A supported retained-fresh branch settles Q02 but does not pretend factor reuse occurred.

<a id="c04"></a>

### C04 — Declared numerical profiles and formulation choices

Settle Q03 through existing typed settings, route eligibility and strategy composition rather than another tuning controller. Cover actual root, initialization, study, fitting and nested implicit consumers. Compare fitting existing libraries under equivalent original obligations:

- Global KINSOL/direct or JVP/Krylov with its existing BTF block preconditioner versus eligible conditional block solves. Coalescing preserves dependency order and independently assessed original equations. A scalar1×1 block is bracket-eligible only with the actual polynomial/domain/interval/continuity/sign contract; bracketing is not uniqueness proof.
- Native damping/Anderson depth/delay/orthogonalization/restart and Newton/Picard/JVP tear routes. Raw-map residual observations are separate from accelerated points and nested work. No unconditional depth3–5 or bespoke Broyden/Newton engine.
- Existing native/library scaling before a custom rectangular equilibration layer. Keep physical nominals/budgets frozen; transport bounds, duals and derivative/JVP actions. Symmetric KKT scaling is not a general rectangular Jacobian contract.
- Eligible root/init fixed/connection elimination and FBBT through existing transformations. Multiplier irrelevance removes one requirement, not primal recovery/domain/branch/threshold obligations. Keep coefficient/equality preservation thresholds explicit.
- Finite ε-continuation using the existing staged strategy with target-model assessment and restored specification on failure. RR dimensionless regularization/clipping constants retain their mathematical role; do not multiply them mechanically by physical nominals.
- Exact/limited-memory curvature, MUMPS/SPRAL/Pardiso/FERAL order/scaling and actual monolithic/Schur paths. Typed useful controls may be promoted, but n/nnz alone does not select a winner. FERAL Auto/AutoRace already exists; any persisted ordering result is compatible prepared state, not universally optimal timing advice.

Each comparison names demanded outputs, start permissions, total failed/inner/recovery/assessment work and eligible capabilities. A requested route may fall back; record the actual route/reason. Preserve current defaults unless a concrete family-specific decision selects a replacement. Do not make a solver tournament or Cartesian option sweep mandatory.

Explicit Q03 subdecisions include KINSOL setup-interval profiles 1–3 versus monitored default 10; selection or supported rejection of an adaptive-μ retry rung; and consumer-needed typed `mumps_pivtol`, `mumps_mem_percent` and permuting-scaling controls. A selected retry inherits only permitted starts, consumes remaining cumulative budget, retains failed-attempt costs and reports actual termination. Existing typed representability is not itself a decision to add/change that rung or default.

<a id="c05"></a>

### C05 — Refinement, convergence and candidate policy composition

Keep one explicit effective numerical configuration. Ipopt max-budget versus POUNCE min-budget tolerance aggregation reflects different native convergence floors; raw option equality is not equivalent acceptance. KINSOL interval 10 is native monitored maximum, not ten guaranteed stale iterations; inner implicit already interval 1. Adaptive μ and acceptable stopping are already typed possibilities, not missing generic mechanisms.

Compose FERAL refinement with outer method needs: actual default steps 10, limited-memory interior-point carve-out versus SQP, and refinement precheck versus inner stopping. Do not import environment/default magic or disable the only SQP refinement. Explicit configuration and actual work/failure remain visible.

Relaxed-bound solving is an intentional policy option, not free performance. Any projected candidate is independently checked against original residuals/bounds/domains, and changed stationarity/active-set/derivative validity can withhold KKT/response products. Acceptable/limited termination remains truthful and selected, never relabeled ordinary full convergence. Current defaults remain until Q03's concrete choice.

Test failed refinement, perturbed/singular factors, exact/limited-memory/interior-point/SQP distinctions, projection-induced constraint failure and withholding, and equivalent original acceptance under backend-specific raw tolerances.

<a id="c06"></a>

### C06 — Selected owner-safe native detail and library fixes

Settle Q04 by required event family and pinned implementation. Add detailed hooks only where they answer a consumed question: POUNCE/FERAL internal retry/refinement and actual order/fill; SUNDIALS/KLU setup/refactor/solve; PETSc stages/events/KSP/PC; Uno structured iterations/subproblems; native coefficient/conic/discrete detail. Unsupported internals stay unknown. A newer SDK is not an activated/qualified hook.

POUNCE history needs a finite collector integrated with A03 and caller tracing composition; end-only truncation/metric flattening is replaced. SUNDIALS 7.3 damping/depth hooks govern fixed-point/Picard, not general Newton monitoring. Pinned7.1.1 logger/profiling and PETSc 3.24 headers are checked at their actual release/build route. An upgrade or narrow maintained source extension is considered only for a concrete missing contract, with corresponding ABI/build tests; licence is not a rejection criterion.

Keep eligible native-library overhead fixes at the library owner: snapshot debug controls at the appropriate lifetime; make dump cloning conditional; reduce MC64 matching/scaling clones and KKT value copies only where exact pinned ownership shows redundancy; prefer existing into/workspace routes; skip hashing/comparison only through a valid immutable checked pattern identity; investigate factor arenas with pivot/growth/failure/retained-byte obligations. Preserve immutable retained inputs, scaling validity and failure behavior. Some refined-solve scratch is already pooled, and four KKT copies are not universal. These overhead fixes do not depend on a new Q04 hook.

Resolve the apparent A03/C06 cycle through real slices: C06 can deliver coherent owner-safe extraction against A02 before detailed-stream integration; A03 supplies finite sink/coverage semantics; selected C06 native-history integration then consumes that A03 contract. Neither whole package must complete before the other's first useful delivery.

R2's six Ipopt deep options have a source-supported current route. Boolean/text timing/info flags, integer file print level and text output/derivative-test options are unreserved; first C++ initialization reads pre-applied output settings despite file-only help wording. Printing timing activates collection; print verbosity affects visibility. Info strings are native output tags, not structured checkpoint fields. Derivative checking adds selected callback work. Output files are explicit effects; changing an initialized retained session follows its rebuild/new-session contract. Qualify selected logging/hooks, not every possible option automatically.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C01 | A01 availability/producer slice; A02 for coherent projections | Truthful native activation/current progress across supported classes, terminal/live consumer migration and no zero-for-disabled timing | done |
| C02 | Existing callback/output semantics; narrow B fill slice where needed | Prepared maps/owned scratch/bulk output plus valid bridge/transformed witnesses; no partial publication or unchecked foreign structure | done |
| C03 | Q02 validity/owner inspection; accepted RC05 route; extra concrete decision for stronger retention | Qualified compatible native/conditioning reuse or fresh-state branch, actual reuse facts and adversarial invalidation/drain controls | done |
| C04 | Q03 capability/formulation decision; accepted RC07 route | Declared family profiles with native-owned algorithms, original recovery and concrete default decision or retained defaults | done |
| C05 | ResolvedAccuracy/effective config authority; specific Q03 choice | Sound method-specific refinement/stopping/bound policy and original-candidate qualification; remove conflicting implicit defaults | done |
| C06 | Exact library/build evidence; Q04 only for hook work; A02 for extraction and A03 only for selected history integration | Needed bounded native hooks and independently ready library-owner fixes, supported unknown branches and consumer/ABI qualification | done |

## Verification and checkpoint

**Tested, 2026-10-17:** complete packet acceptance and selected supported inquiry branches are recorded in the Outcome below. [43g’s Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns assembled/scoped acceptance, repaired composite boundaries, scope-end checks and exclusions. The packet table is the native progress owner; formal proposed ADR status does not assert or replace implementation acceptance.

## Outcome (recorded after implementation)

### What was built

**Implemented:** C01–C06 retain truthful native activation and coherent progress, prepared callback/TNLP/KINSOL/PETSc transport and bulk publication, checked original/transformed witnesses, typed MUMPS pivot/max-pivot/memory/permuting controls and one effective FERAL refinement authority across IPM/SQP exact/limited-memory methods. Actual KLU issued/completed counters, optional Uno accepted/trial detail and PETSc owned nested hook extraction retain unknown/refusal and generation meaning. The coherent maintained POUNCE/FERAL family is pinned at `cfa07f0d4089bbab756def9df25dbca740547f03`.

**Tested, 2026-10-17:** `just unit-native-capability-package pse-backend-native native-solvers solver,klu,isolation,uno,petsc` with callback/presolve/sequence/POUNCE/implicit/dynamics/KINSOL/PETSc/transport/lifetime selectors passed 284/284. Actual Uno ABI (8), backend (9), selected POUNCE factor (1), maintained-family fatal factor detail (1) and final PETSc getter/refusal ABI (1) controls passed in their linked routes. The integrated runtime selection passed 76/76 with original equations, branch restoration, profiles, recycling and consumed-accuracy assertions. [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns integrated/static evidence and exact boundaries.

**Interface-checked:** Q02 retains compatible allocation and symbolic/pattern reuse with fresh point-dependent numeric state: no Ipopt same-structure warm-start shortcut, KINSOL setup suppression, or stale KLU/FERAL numerical factor. Q03 retains global setup10/inner1 after actual 1/2/3/10 controls; existing bounded declared conditioning/composition routes keep original acceptance and do not become automatic retries. Typed explicit FERAL refinement defaults remain true/10 and explicit false is honored by every method. Q04 selects bounded actual KLU/PETSc/Uno and maintained-family hooks; unsupported getters remain unavailable/unknown, never disabled-as-zero. These are completed decisions, not pending performance winners.

### A mistake made and corrected

Actual native tests exposed an empty-Hessian SQP declared-product boundary and stale factor-detail survival after fatal factorization. The boundary now supplies the declared empty product without a full-oracle call or zero-curvature certificate; fatal factor detail is cleared at its library owner. PETSc getter failure now reports unknown instead of falsely inactive, with an actual ABI refusal assertion.

### Deviations from the plan, deliberate

Stronger numerical-state retention and native factor handoff were not adopted without valid compatibility premises. Existing default setup/strategy choices remain supported conclusions rather than tournament deferrals. Native accepted/trial or inclusive hook observations are not relabelled iterations/exclusive times. No license restriction excluded a technically fitting library. Formal ADR statuses remain unchanged.
