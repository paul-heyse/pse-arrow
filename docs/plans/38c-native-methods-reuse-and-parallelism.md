---
title: Native methods, reuse and parallelism
status: draft
date: 2026-10-13
adrs: []
review_sources: [docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md]
doc_owner: docs/plans/38-efficiency-and-solver-target.md
doc_topics: [architecture, libraries, preparation, validation]
---

# Plan 38c: Native methods, reuse and parallelism

## Scope and baseline

This companion owns consumed native capability, actual retained-state/reset contracts and transitive resource placement for [Plan 38](38-efficiency-and-solver-target.md). Scientific choice belongs 38b/38d; immutable model programs belong 38a; storage does not govern native lifetimes. It addresses F07/F08/F11/F13, AU01/AU02, sparse-rank R38-01 and remaining P-C1–C8/native investigations. [Native evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/native-contracts-and-methods.md) and [parallelism evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/parallelism-and-coverage.md) identify exact sources and limits.

The baseline already retains actual role-specific FERAL backends through the project's POUNCE factory, forces a new-application numeric factor and discards fatal/quality-altered leases. HiGHS genuinely updates coefficients/bounds/Hessian and submits explicit bases. KINSOL has conditional retained setup; profile chains already use admitted parallel teams. Preserve these mechanisms. Ipopt 3.14.20 and SUNDIALS 7.1.1 can repeat native symbolic work despite retained wrappers. A universal allocation-free second solve is unsupported. FERAL Auto/rescue policy can survive a fresh factorization, conditionally affecting later retries; no outcome regression was reproduced.

Current contracts are blueprint §15–§18 and named native ADRs. Accepted RC06/07, S-RC03–07/11–13, ARC01/R38-01 define target fences; G0 routes concrete amendments/new bindings before code. **Interface-checked** baseline; all implementations/probes below **Proposed**.

## Packets

| Packet | Prerequisites | Delivery and retirement | State |
|---|---|---|---|
| <a id="c1"></a>C1 | Existing native profiles/quality seam; B1 request meaning | Complete per-profile compatibility/start/stop/counter contract, corrected claims and compact callback ownership | Planned |
| <a id="c2"></a>C2 | C1; I-C1 for affected Auto policies; RC06/ARC01 | Lane-owned compatible allocation/structure with explicit numeric/heuristic policy; remove accidental worker-history reuse | Planned |
| <a id="c3"></a>C3 | B1/B2 permission; I-C2/I-C3; C1 actual contracts | Selected complete native methods/getters/stop snapshots integrated with real scientific consumers | Planned |
| <a id="c4"></a>C4 | C1 actual transitive profiles; RC07 | Scoped pools/process lanes/exclusion/charged capacity, preserving quiescent handoff and actual drain | Planned |
| <a id="c5"></a>C5 | Existing retained HiGHS coefficient path | Bulk vector updates, justified optional value deltas and transactional invalidation; delete redundant scalar loops | Planned |
| <a id="c6"></a>C6 | Same-point/scale contract; I-C4/R38-01 | Shared factor/actions, honest fill admission and separately named sparse diagnostic tier if fitting | Planned |

C5 can proceed independently of routing/topology; C1 truthful observations can precede extended reuse. C2 neutrality on affected profiles requires I-C1, not every unrelated package. C4 is an actual prerequisite for D's new parallel execution; thread-count arithmetic is not placement. No store/journal replacement is prerequisite here.

## C1/C2 — Native compatibility, retained state and terminal use

Compatibility includes backend/profile/options, actual normalized pattern/index space, selected programs/derivative scheme, physical/provider/runtime owner, scaling and bound classifications. Same dimensions/nnz are insufficient. Ipopt may refresh numeric bounds while fixed/free, finite/infinite, equality and treatment maps remain compatible; rebuild on changed classifications. Retain immutable conversion/structure/workspaces where actual library contracts allow, not a promise of zero internal allocations or zero reanalysis.

Classify retained state: pure symbolic/pattern; neutral workspace; numeric factors/bases/L-BFGS/predictors/integrator history; and value-sensitive scaling/ordering/rescue policy. Independent attempts use a declared reset/fresh profile; continuation edges/session policy explicitly permit and record numerical history. Causal scheduling/lane assignment cannot invent those edges. A tolerance label alone cannot authorize a different physical branch/local stationary point. Preserve exact start/dual/branch provenance and fresh original assessment.

I-C1 inspects actual reachable FERAL Auto/rescue settings and library reset hooks. The pinned `mc64_retry_not_adopted` latch is pattern-keyed although rescue depends on values; successful outer recovery may return that backend to the pool. Prefer a library reset preserving neutral work; compare fixed profiles, fresh backend fallback or explicitly owned policy history when no fitting seam exists. Do not implement MC64/factorization locally. Existing numeric-refactor and quality-discard behavior remain necessary but do not alone settle this question.

Terminal reuse is method/profile-specific: callback invalidity/panic/fatal or uncertain native mutation cannot return a healthy reusable object merely because a common result exists. Record actual attempted/accepted native settings and unavailable statistics. Owner-thread destruction, exclusive mutable leases and charged resident allocation continue until native work/borrowers physically drain. New-attempt counters are actual deltas/resettable facts; lifetime extrema cannot become attempt extrema.

**Targeted controls:** compatible value versus pattern/bound-class/profile changes; Fresh/RequireReuse and refusal; different predecessor/lane histories on the same target; singular then potentially rescuable values; branch/seed semantics; fatal/quality escalation/callback failure; main/restoration/simultaneous exclusive leases; stale same-size pattern collision; outstanding borrowed allocation and owner-thread teardown. Inspect and record latch applicability before asserting a reproduced harm. Every candidate still passes independent original assessment.

## C3 — Complete methods and truthful native observations

I-C2 gives every source-companion method family an explicit consumed-capability outcome. Selected methods need actual scientific consumers, original-coordinate mapping, settings, stop/proposal/report contract and targeted tests; no dormant adapter-only completion. Numerical iteration/globalization/factors remain library-owned. The following decisions guide that investigation without binding every available library:

| Capability family | Selected route or explicit investigation/trigger |
|---|---|
| Affine and scalar blocks | Use one direct factorization plus bounded solve/refinement and original residual check; a bracketed root needs continuity/evaluability/sign-change and authored branch. Select a fitting native root implementation through I-C2. |
| Small dense/nonlinear root trust region | Compare actual KINSOL Dense and PETSc SNESNEWTONTR contracts for a revealing difficult root workload. A generic optimizer named TrustRegion or scaled block_solve alone does not establish fit. |
| Exact/certified polynomial roots | I-C3 preserves exact f64-to-rational defining polynomial, degree/multiplicity/branch/IFT and guarded real-interval route with certified fallback. No blanket endpoint/refinement replacement. |
| Opaque-map DQ/JFNK | Investigate KINSOL native DQ Jv and PETSc matrix-free for repeatable residual, scaling/perturbation and suitable preconditioner. Select only a consumed root/map request; no unpreconditioned universal default. |
| Two-sided bounds/VI/TAO | VI requires authored complementarity, not merely bounds. Compare TAO bounded optimization/BRGN where the actual goal is optimization/LS; stationarity is not root acceptance. |
| PETSc damping/QN/NGMRES/Anderson/arclength | Separate method-by-method contracts. NLEQ-ERR is generally Newton damping; SSLS limitation is VI-specific. Select on an unmet consumed request or defer with that request as trigger. |
| Recycle acceleration | D2 owns repeatable-map/default/restart decision; bind supported native damping/Anderson under actual current/selected SUNDIALS contract, not custom generic iteration. |
| Ipopt options/statistics/timing | Promote actual structured available facts through existing C++ bridge. Compare nominal and available gradient scaling under supplied no-HSL scope. Preserve concrete Pardiso curvature/inertia caution; a convex objective alone is insufficient. |
| KLU across-solve reuse | Compare fitting existing/custom SUNLinearSolver seam only for actual setup gap; reanalysis occurs at setup, not every early-exit call. Retain current factor/rcond/full-factor fallback where valid. |
| POUNCE batches | Compare cold independent batches with retained related sessions. A batch includes its complete restoration/backend recipe, per-instance budget and aligned warm seeds; no blanket mutual exclusion of batching and retention. |
| Dynamics diagnostics | Expose consumed IDAS getters/Diffsol order/step facts and actual continue/restart semantics; D4 owns accuracy/event acceptance. |
| CVODES/ARKODE | Retain current integrator alternatives; reopen when a specified ODE/stiff/partitioned workload lacks the required complete capability. No feature enablement merely to fill inventory. |
| HiGHS sequence/runtime | Preserve real model updates, explicit basis input/output and actual runtime evidence; C5 improves update grain. |
| SCIP statistics/export | Expose actually consumed typed statistics/original-problem export with profile/build resource contract; otherwise defer until a concrete diagnostic/global-work consumer. |
| Sparse QR | C6/I-C4 defines a distinct tier; no silent SVD substitution. |
| SHOT/Bonmin/qpOASES/DAQP/diffsol-nl | Explicit candidate decisions under unmet convex-MINLP/parametric-QP/root capability and complete domain/stop contract. R-47 is capability-based; no cache-dominance or library-census trigger. |
| JIT/SIMD/evaluator persistence/FeOS | Compare actual public APIs, guards, exact programs and machine/FMA/determinism/physical contracts before production use. Keep current evaluator and FeOS reference-only status absent a selected fitting request. |

Native observations extend B3's optional typed families; text logs do not establish counters. FERAL n_tiny/perturbation mode is numerical confidence, not a theorem of exact inertia. MUMPS/Pardiso/SPRAL analysis claims describe the selected actual path and early exits. Ipopt restoration-aware callback access supplies original-space primal/dual snapshots, not all slacks; hard stop never waits for restoration diagnosis. KINSOL normally returns the last evaluated fixed-point point unless ReturnNewest is explicitly selected; failed map evaluation still needs last-successful state. Positive fixed-point callback handling must preserve the project's necessary failure latch.

**Targeted controls:** exact class/bounds/domain/derivative refusals, native status and callback failures, available/absent counters, restoration stop, last map snapshot, limits/cancellation and original-coordinate acceptance. Root/LS/VI/global assurance remains distinct. New methods may legitimately remain unselected after investigation, but every actual supported capability gap needs a concrete solution or explicit support decision.

## C4 — Actual resources, pools and process lanes

Declare the selected profile's transitive resources, including nested SCIP→Ipopt/MUMPS/BLAS, Clarabel backend variants, actual loaded interface instances, HiGHS scheduler, PETSc/IBEX and OpenMP/MKL controls. Ipopt's MUMPS mutex is conditional/local to that library interface, not a universal direct-MUMPS lock. PETSc MPIUNI process lanes isolate independent solves, not one MPI decomposition. SUNDIALS supports per-thread contexts; sequential migration remains an investigation, so pin affected contexts until settled.

Install explicit admitted ambient Rayon scopes where needed; faer uses ambient, global fallback otherwise. POUNCE already installs a local pool; FERAL owns a persistent pool with an unresolved width seam. Source bit-neutral driver claims are not an executed determinism guarantee, and do not justify universally serializing every path. Compare actual width/configuration/reset seams per profile. Pool creation, foreign stacks, workspace and live queues remain charged.

Preserve CPU/population/active distinctions, resident parked sessions, tentative-permit release and no hold-and-wait. IBEX waiting becomes quiescent without dropping parent resident charges; acquire exclusion plus actual compute without a guard/permit cycle. A callback cannot release a whole team's allowance while siblings execute. Process children and analytics consume one deployment envelope with bounded IPC/backpressure, authentic attempt/owner identity and late-result rejection. Capacity is released after actual drain/join, not caller timeout or empty superficial observer alone.

**Targeted controls:** multiple independent teams and nested work under finite allowance; saturated resident/active queues; IBEX/fit phase handoff; zero-byte spillable departure wakes waiters; cancellation during construction/exclusion/native work; width/profile refusal versus capacity waiting; process death/late completion; parent accounting until children drain. Actual observed width and team behavior must match the selected profile before parallel capability is claimed.

## C5 — HiGHS bulk update and failure atomicity

The retained staged coefficient path already reaches Session::update and sends all costs/bounds/nonzeros/rows/objective/Hessian. Use pinned native Range/Set/Mask vector operations first. A trusted normalized predecessor delta or retained uploaded snapshot may then skip unchanged matrix/Hessian content when its scan/memory/invalidation cost is justified. Pattern identity alone does not establish unchanged numbers; no general matrix-value bulk API is assumed. Keep a checked full-update/rebuild fallback, not a second coefficient authority or persisted per-entry hash ledger.

Preserve binary clipping, sparse zero transitions, original/native coordinates, sense/offset/Hessian and explicit warm/basis policy. Commit uploaded-content premise only after every mutation succeeds. Partial failure invalidates/restores the session even if RequireReuse forbids rebuild; preserve the typed cause instead of reducing all errors to structure mismatch. Diagnostic/lexicographic edits also restore or invalidate content. Allocation/model update, implicit native state and explicit basis are separate facts.

**Targeted controls:** full rebuild/update/bulk/delta effective-model and fresh original equivalence for single cost/bound/coefficient, zero, broad/Hessian/structure changes; injected partial failure including forbidden rebuild; restoration of temporary edits; actual submitted native-operation counts. Whole-model transport/upload verification/original assessment remain, so do not claim end-to-end O(changes) or speed without complete-route measurement.

## C6 — Same-point factors and honest sparse diagnostics

Retain exact same-point/scale/provider/row identity and shared response/multi-RHS actions where real factor lifetime permits. KKT independent assessment is not redundant because the optimizer owns a factor. Numeric rank/singularity is value-dependent; distinguish factor/action reuse from structural ordering reuse. Preserve actual fill/scratch admission and truthful unavailable/refusal rather than an invented worst-case sparse bound.

I-C4 investigates SPQR exact API/build/index-width/memory/threshold fit as a separately named rank/evidence tier under accepted R38-01. Existing SVD uses singular-value cutoffs and near-null singular vectors, including rectangular null space; QR thresholds/bases do not reproduce that meaning. Retain bounded SVD and explicit large-size support limits. Selecting a new tier requires its binding/relation route, original-coordinate mapping and independent residual/rank evidence; no current sparse support is claimed from a prototype.

## Investigation outcomes and scope-end evidence

I-C1 settles profile reachability/reset/history for AU02 and native width/context hooks; I-C2 records a select/investigate/defer outcome for every family above, with an actual request trigger and fitting alternative; I-C3 settles remaining cubic/implicit/presolve/evaluator validity, preserving current implemented KLU/IDAS reverse-cursor and guarded-root decisions rather than redoing them; I-C4 settles sparse tier/fill support. Retain start/dual/accuracy-dependent presolve facts as fresh, FERAL SparseLu Markowitz semantics and request-local IBEX Function/System even if immutable Rust transport declarations are retained.

Use library skills, Context7 and exact pinned/fork sources; probes are discretionary and cannot touch another owner's active work or clear shared caches. Each outcome states evidence, selected/retained contract, affected dependents and support limit. Compile/targeted unit controls accompany changes, with explicit Arrow validation if consumed. G1–G3 own assembled native/scientific/performance and scope-end checks after all selected functional work. Historical failed/stopped native assessments remain failed/stopped until that current scope is qualified.

## Dependency handoff — Plan 38b, 2026-10-13

[38b](38b-solver-policy-observation-and-diagnostics.md#execution-checkpoint--2026-10-13)
delivers the authorized minimum C1/C3 producer slice: optional method-specific observations,
actual effective settings/defaults and stop/checkpoint fidelity, bounded submitted-oracle
snapshots, early-failure counter truth and portable seed restoration with explicit refusal
of unavailable native continuation. Native/component records retain their admitted owners.
Existing numerical algorithms, pins and defaults are unchanged. Its Verification owns the
scoped native controls; these are not full per-profile, reset/history or scientific acceptance.

C1/C3 remain Planned for their wider compatibility, callback, method-family, dynamics and
scientific-consumer obligations. I-C1–I-C4 and C2/C4–C6 are not completed by this handoff.
Consume the existing observation product rather than adding a second counter/attempt authority.
