---
title: Native methods, reuse and parallelism
status: in-progress
date: 2026-10-13
adrs: [ADR-0185]
review_sources: [docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md]
doc_owner: docs/plans/38-efficiency-and-solver-target.md
doc_topics: [architecture, libraries, preparation, validation]
---

# Plan 38c: Native methods, reuse and parallelism

## Scope and baseline

This companion owns consumed native capability, actual retained-state/reset contracts and transitive resource placement for [Plan 38](38-efficiency-and-solver-target.md). Scientific choice belongs 38b/38d; immutable model programs belong 38a; storage does not govern native lifetimes. It addresses F07/F08/F11/F13, AU01/AU02, sparse-rank R38-01 and remaining P-C1–C8/native investigations. [Native evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/native-contracts-and-methods.md) and [parallelism evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/parallelism-and-coverage.md) identify exact sources and limits.

The baseline already retains actual role-specific FERAL backends through the project's POUNCE factory, forces a new-application numeric factor and discards fatal/quality-altered leases. HiGHS genuinely updates coefficients/bounds/Hessian and submits explicit bases. KINSOL has conditional retained setup; profile chains already use admitted parallel teams. Preserve these mechanisms. Ipopt 3.14.20 and SUNDIALS 7.1.1 can repeat native symbolic work despite retained wrappers. A universal allocation-free second solve is unsupported. FERAL Auto/rescue policy can survive a fresh factorization, conditionally affecting later retries; no outcome regression was reproduced.

Current contracts are blueprint §15–§18 and named native ADRs. Accepted RC06/07, S-RC03–07/11–13, ARC01/R38-01 define target fences; proposed ADR-0185 and architecture revision 162 installed the concrete amendments before dependent code. The scope-end outcomes below distinguish implemented bindings, targeted tests and explicit support limits; assembled acceptance remains 38g.

## Packets

| Packet | Prerequisites | Delivery and retirement | State |
|---|---|---|---|
| <a id="c1"></a>C1 | Existing native profiles/quality seam; B1 request meaning | Per-profile compatibility/start/stop/counter contracts, corrected claims and compact callback ownership | Implemented; Rust controls passed; final Python consumer acceptance open |
| <a id="c2"></a>C2 | C1; I-C1 for affected Auto policies; RC06/ARC01 | Lane-owned compatible allocation/structure with explicit numeric/heuristic policy; accidental worker-history reuse removed | Implemented; Rust controls passed; final Python consumer acceptance open |
| <a id="c3"></a>C3 | B1/B2 permission; I-C2/I-C3; C1 actual contracts | Selected complete methods/getters/stop snapshots integrated with scientific consumers; remaining families have explicit support/trigger decisions | Implemented; targeted controls passed |
| <a id="c4"></a>C4 | C1 actual transitive profiles; RC07 | Scoped pools, bounded private child, exclusions and charged capacity through actual drain | Implemented within stated profile/reconstruction limits; targeted controls passed |
| <a id="c5"></a>C5 | Existing retained HiGHS coefficient path | Bulk vectors, optional value deltas and transactional invalidation; redundant scalar loops removed | Implemented; targeted controls passed |
| <a id="c6"></a>C6 | Same-point/scale contract; I-C4/R38-01 | Shared factor/actions and honest fill admission; sparse QR not selected | Implemented; targeted controls passed |

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

### I-C1 — Independent state and actual native reset

**Interface-checked / Implemented:** the maintained POUNCE family is pinned coherently to
`d02beafd370fd3815d6c0541fab1630ff839a739`. FERAL's new application reset restores the
construction-time numerical/quality policy and clears Auto/MC64 non-adoption, ordering,
cascade and race latches. Changed symbolic ordering is invalidated; ambiguous inertia,
fatal, interrupted or otherwise unhealthy state is discarded. Its explicit owned pool width
replaces ambient global-width discovery. Main/restoration and pooled batch instances remain
exclusive. Numeric factors and prior application objects do not cross an independent boundary.
This addresses reachable carried policy; it does not assert that the original latch caused a
reproduced wrong solution or that every native analysis is eliminated.

The common retained-state owner distinguishes pre-mutation incompatibility from failed or
uncertain mutation. Failure destroys the object and its charge before returning the original
typed cause, including under RequireReuse. Independent terminal reset parks only accepted
HiGHS and POUNCE neutral state; every other direct backend uses fresh-object fallback.
Request-owned advanced factors and implicit KINSOL TLS state are dropped before parking.
Subsequent RequireReuse observes an actual fallback rather than silently creating a replacement.

Within a finite compatible sequence, KINSOL retains native allocations but requests fresh
initial numerical setup on every call and clears project Jacobi/block setup state. Ipopt
retains its TNLP/application and actual ReOptimize operation while setting
`warm_start_same_structure=no`: Ipopt 3.14.20 then recomputes spaces, maps and value-dependent
gradient scaling from the current problem/start. Numerical dependency detection conservatively
refuses changed bound data. Clarabel 0.11.1 applies original equilibration to updates and does
not recompute it; equilibrated sessions therefore refuse reuse before mutation and rebuild
when permitted. Unscaled reusable-data sessions retain genuine native updates and native
fresh-iterate initialization. Explicit starts alone authorize numerical start data.

HiGHS clears solver state, linear objectives and options at independent boundaries, retaining
only the model and separately charged successful-upload witness. The witness is optional:
allocation pressure selects the checked full-update fallback. It holds no completed task
ledger, evaluator or request control. A borrowed POUNCE factor keeps its own allocation lease
after the application/session drops; charge release follows actual borrower drain.

POUNCE-convex previously kept only prior QP numerical starts in its apparent retained session,
despite advertising no start support. That session/cache is deleted. Independent convex
members use the library's cold APIs, accept Fresh/AllowRebuild and refuse RequireReuse or
explicit numerical seeds. Repeated changed QPs are compared with fresh original-qualified
solves. Convex batching remains supported; it does not imply retained neutral allocations.

### I-C2 — Consumed method decisions

**Interface-checked:** exact pinned source and the library skills resolve the following
decisions. **Implemented** marks new bindings with scientific consumers; retained methods
keep their existing qualification scope. No row claims a measured speedup.

| Family | Outcome, consumer and support/revisit condition |
|---|---|
| Affine / scalar blocks | **Implemented:** registered `accelerated(affine)` reaches existing faer factor/solve and original guarded verification; `accelerated(toms748)` reaches root1d 0.4.1 for closed scalar polynomial residuals on an authored finite branch interval. Endpoint, sign, evaluation, stop and final residual checks remain outside library width termination. The compiler's scalar selection/IFT contract is preserved; general coupled authored branch/neighborhood selection is not newly claimed. |
| Small dense / trust-region roots | Retain KINSOL Dense and the existing actual PETSc SNESNEWTONTR consumer. A difficult root workload needing a different globalization, with original residual and complete domain/stop evidence, reopens the choice; an optimizer's similarly named trust region is not a replacement. |
| Certified polynomial roots | Retain exact Symbolica defining-polynomial and guarded/certified real-root fallback; see I-C3. TOMS748 adds an explicit alternative, not a replacement for multiplicity or certified branch meaning. |
| Opaque-map DQ / JFNK | Defer additional routes until a consumed repeatable residual/map has a required directional-derivative capability and a qualified preconditioner, physical perturbation/scaling and stop contract. Compare KINSOL native DQ Jv and PETSc matrix-free then; existing analytic JVP remains. D2 owns the map request. |
| Two-sided bounds / VI / TAO | Retain constrained root routing and explicit unsupported combinations. Reopen VI for authored complementarity; TAO/BRGN for a specified optimization/least-squares request. Bound satisfaction or optimizer stationarity never grants root acceptance. |
| PETSc damping / QN / NGMRES / Anderson / arclength | Retain the currently consumed method profiles. New methods require a concrete root/map or continuation request unmet by them and complete native callback/globalization/stop semantics. NLEQ-ERR is damping; SSLS restrictions concern VI. No general method-family enablement is selected. |
| Recycle acceleration | Preserve current SUNDIALS damping/Anderson, raw-map residual and last-successful state contracts. D2 owns repeatability, default/restart and additional method selection; this packet does not introduce custom iteration or reinterpret last evaluated point as ReturnNewest. |
| Ipopt observations / scaling | Retain consumed structured statistics, available timing and restoration-aware snapshots through the existing C++ bridge; remove obsolete register R-48. Refresh current gradient scaling as I-C1 specifies. Keep supplied no-HSL scope and Pardiso curvature/inertia caution; a convex objective alone is insufficient. |
| KLU setup | Retain native factor/rcond/full-factor fallback and current owning contexts; require fresh new-application numerical setup. Reopen a custom SUNLinearSolver binding only for a demonstrated unmet setup capability, with actual early exits distinguished from calls that perform setup. |
| POUNCE batches | Retain actual cold convex batches and related finite NLP sessions. Delete the convex adapter's implicit seed-only cache. Single convex members use the admitted FERAL width; parallel batch members factor at width one inside the admitted outer pool. Apply neutral reset/discard to retained NLP library recipes. The NLP adapter remains `batch:false`; native NLP batching is deferred until a cold independent population requires a complete restoration/backend, per-instance budget and aligned-seed comparison. Library availability alone is not a consumed route. |
| Dynamics diagnostics | **Implemented:** typed IDAS 7.1.1 last/current step and order, and Diffsol 0.16.2 current order/next-step size, published through existing segment statistics. Capture native mesh time separately from returned/root time, before rewind. Last-step data requires actual steps; no missing Diffsol last-step getter is invented. D4 retains event/accuracy acceptance. |
| CVODES / ARKODE | Defer until a specified ODE, stiff or partitioned workload lacks a required complete capability in IDAS/Diffsol; compare exact native APIs and original dynamics meaning then. No feature census is selected. |
| HiGHS sequences | **Implemented:** native Range vector updates, optional exact value-bit delta witness, transactional invalidation and neutral reset. Preserve explicit basis, clipping, objective/Hessian and native diagnostic semantics; full upload verification and original assessment remain. |
| SCIP statistics / export | Retain consumed Certification/global statistics and checked native factorable-model lowering/readback. User-visible original-problem file export and additional counters are deferred until an actual diagnosis/replay/global-work consumer requires them and the linked resource contract qualifies. Native lowering is not a diagnostic file-export producer. |
| Sparse QR | Retain bounded SVD and explicit large-size refusal. I-C4 does not qualify a sparse QR production tier. |
| SHOT / Bonmin / qpOASES / DAQP / diffsol-nl | Defer until a specified convex-MINLP, parametric-QP or root request lacks a complete supported capability. Compare fitting native alternatives, callback/domain/recoverability/stop and original acceptance then. R-47 is corrected to this capability trigger; KINSOL cache cost alone does not select diffsol-nl. Licensing is not a rejection criterion. |
| JIT / SIMD / evaluator persistence / FeOS | Retain current exact-program/evaluator guards and FeOS reference-only role. Reopen for a consumed evaluator or physical-property capability that the current owner lacks, qualifying public API, machine/FMA/determinism, validity and physical conventions first. No machine-dependent build or persisted evaluator becomes authority. |

The difficult-root KINSOL/PETSc comparative workload and nominal-versus-gradient difficult-NLP
performance campaign were not run. Existing consumers and complete-method support are retained;
the new Ipopt control establishes current-application scaling refresh, not performance superiority.
G3 owns representative comparative measurements when qualifying a new preference or performance
claim; no such preference is introduced here.

### I-C3 — Preserved scientific validity

**Interface-checked / Tested:** cubic controls preserve exact f64-to-rational coefficients,
actual degree reductions, multiplicity, interval endpoints and guarded certified fallback.
A repeated root cannot acquire nonsingular IFT derivatives. Scalar acceleration preserves
original guards, authored selection and physical residual acceptance; same-sign tiny endpoints
are refused before root1d's width shortcut. Parameter changes refactor affine values and
recompute implicit derivatives; singular, out-of-bounds and cancelled trials refuse.

Presolve remains fresh with respect to starts, dual/accuracy controls and current values.
IBEX Function/System objects are request-local; immutable transport is not native object
authority. Preserve FERAL SparseLu Markowitz semantics and existing KLU/IDAS reverse-cursor
and exact evaluator guards. No alternative polynomial representation, approximate branch
cache, unconstrained root substitution or new scientific continuation meaning is selected.

### I-C4 — Sparse fill and rank support

**Interface-checked / Implemented:** pinned faer 0.24.4 sparse LU owns opaque symbolic/fill
vectors not bounded by a quadratic input-shape estimate or its MemStack scratch requirement.
Strict complete-memory requests therefore refuse before oracle/derivative/factor work,
including retained response actions/backsolves. Non-strict execution keeps separately charged
input/factor allowances and exact known scratch admission without claiming a complete fill bound.
Same-point/provider/scale/row identity and shared multi-RHS factors remain authoritative.

SPQR is not selected: the pinned suitesparse_sys 0.1.4 wrapper refers to `spqr.h`, while the
actual C API is `SuiteSparseQR_C.h`; the selected managed KLU/solver image supplies neither
qualified SPQR/CHOLMOD headers nor libraries. The source corpus's SPQR 4.3.3, CHOLMOD 5.2.1
and SuiteSparse_config 7.7 also require explicit 64/32-bit index and BLAS integer qualification.
QR pivot-column thresholds and bases do not reproduce SVD singular-value cutoffs or modes;
CHOLMOD memory statistics are not an enforceable complete allocation bound. Both consumers
retain bounded SVD and explicit large-size refusal. Reopen only for an approved separately
named QR evidence convention with corrected binding/native image/ABI, original-coordinate
checks and owned allocation/drain support; R38-01 is settled by this support decision, not by
claiming a sparse implementation or changing SVD meaning.

### C4 — Profile placement and lifetime outcomes

**Interface-checked / Implemented:** placement follows the actual selected dependency chain.
No host partition, canonical store lifecycle or distributed native execution is changed.

| Profile | Actual placement and independent-boundary contract |
|---|---|
| POUNCE / FERAL | Explicit positive owned factor-pool width; main/restoration instances remain exclusive. Convex batches use the admitted outer Rayon pool and serial per-member factors. Library reset clears value-sensitive policy before neutral NLP parking. |
| HiGHS / Uno | Preserve the actual shared scheduler lifetime and width agreement; Uno exclusion covers the loaded interface. Incompatible width refuses rather than pretending to resize a live shared scheduler. |
| Ipopt / BLAS | Existing selected-interface exclusions and scoped MKL controls remain; no universal direct-MUMPS lock is inferred from Ipopt's local mutex. Fresh per-application spaces/scaling coexist with retained TNLP allocation. |
| SCIP concurrent | The selected SCIP 10.0.2 `tny` TPI pool is process-static. Concurrent instances hold a shared-interface gate through native free/join, using paired compute pause/try-resume without waiting under the gate. A joined, separately admitted helper queries actual OpenMP and MKL global/domain defaults; unknown or nonserial nested defaults refuse. Parent MKL is scoped to one. Selected PaPILO is built with TBB off. |
| SCIP cleanup uncertainty | Failed destruction retains raw status as terminal Infrastructure. Quarantine holds the native owner, watcher, storage and gate, and takes actual compute ownership when drain is unproved. Replacement is refused; no numerical recovery can spend the quarantined allowance. Injection controls free an idle native fixture before simulating failure and do not establish recovery from real live-worker shutdown failure. |
| Clarabel | Existing Pardiso resource scope remains; SDP/BLAS scopes MKL even with QDLDL. PSD quality uses an explicitly serial low-level eigensolver. Equilibrated data uses fresh-object fallback. |
| KINSOL / IDAS | Preserve thread-affine SUNDIALS contexts; do not claim migration or across-independent-request setup reuse. KINSOL allocation reuse inside a compatible sequence forces initial numerical setup. |
| PETSc / IBEX | Preserve existing process-interface exclusion, quiescent waits and request-local native objects. MPIUNI independent process isolation is not distributed decomposition; no new general PETSc child recipe is selected. |
| faer / diagnostics | Implicit factors use admitted width only inside an actually installed Rayon pool, serial otherwise and under nested execution. Rank/full-nullspace SVD, convexity products and original-space quality explicitly use low-level serial kernels; no global pool initialization or settings mutation selects accidental work. |

Service lanes retain only accepted neutral native state. Parked stacks, residency and population
stay charged; requests reacquire active/CPU/construction under their original cancellation and
deadline, replace scope inventory, and drop source/provider/request-owned state before parking.
Pressure eviction and shutdown join actual owners before releasing resident charges. Failed
scope entry publishes empty inventory before replying, avoiding stale stack accounting.
Bounded per-backend discard metadata survives scope/width changes, repeated clear/reset and
failed entry; native objects never cross scope teardown. RequireReuse cannot bootstrap a
discarded backend, while the established first construction for a previously unseen backend
remains allowed when no live foreign state is held.

**Implemented:** the store-free private process lane consumes a compiler-issued closed,
provider-free portable root recipe and its exact physical declarations through the ordinary
MathService scheduling/admission path. Bounded inherited-socket IPC authenticates attempt,
parent, execution, exact selected executable bytes and a concrete matching KINSOL profile.
The profile binds source pins, configuration/ABI and the managed KLU qualifier's explicitly
selected receipt. A mutable external prefix or absent receipt yields no child profile; a stale
file merely present in that prefix cannot authorize it. Executable observation separately
binds deployed bytes: the profile is not an independent archive census or proof of complete
parent/child Rust-source equivalence.

Original cancellation/deadline checks, kill/reap and late/foreign/malformed rejection retain
parent capacity through drain. Known buffers, transport/index and owner stack are charged
separately from a finite opaque child allowance. Strict complete-memory selection refuses;
the allowance is not a complete RSS bound. Full PreparedSolve, opaque provider closures,
native continuation and distributed PETSc reconstruction remain unsupported. No public
untrusted stdin DTO or canonical durable action is added.

### C5/C6 — Preserved mutation and mathematical meaning

**Implemented / Tested:** HiGHS uses native Range vector updates and an optional, separately
charged exact successful-upload witness for matrix/Hessian changes. Old/new witness overlap
is admitted before replacement; publication follows complete native mutation/readback.
Unavailable witness capacity selects full update. Request refusals before mutation preserve
the healthy model; native mutation/solve/diagnostic failure invalidates it and preserves its
typed cause, including under RequireReuse. Temporary lexicographic/diagnostic edits restore
or invalidate. Explicit basis permission remains separate from allocation retention.

Same-point/provider/scale/row identity gates multi-RHS and predictor actions on a shared factor.
The strict-fill refusal in I-C4 also applies to retained backsolves. Original residual/KKT
assessment, rectangular nullspaces, singular-value cutoffs, sparse zero transitions and
original/native maps are preserved. No end-to-end O(changes), complete opaque allocation
bound, numerical equivalence between QR and SVD, or measured speedup is claimed.

## Dependency handoff — Plan 38b, 2026-10-13

[38b](38b-solver-policy-observation-and-diagnostics.md#execution-checkpoint--2026-10-13)
delivers the authorized minimum C1/C3 producer slice: optional method-specific observations,
actual effective settings/defaults and stop/checkpoint fidelity, bounded submitted-oracle
snapshots, early-failure counter truth and portable seed restoration with explicit refusal
of unavailable native continuation. Native/component records retain their admitted owners.
Existing numerical algorithms, pins and defaults are unchanged. Its Verification owns the
scoped native controls; these are not full per-profile, reset/history or scientific acceptance.

That handoff did not complete the wider C1/C3 or C2/C4–C6 obligations. The implementation
and investigation outcomes above now supply this companion's scoped delivery. They consume
the existing observation product rather than adding a second counter/attempt authority.

## Execution checkpoint — 2026-10-13

The maintainer authorized all C1–C6 and I-C1–I-C4. The approved approach selects narrow
maintained POUNCE/FERAL reset and explicit-width enhancements and, after per-profile
acceptance, AllowRebuild with neutral independent-request reset as the ordinary default.
Explicit Fresh/RequireReuse and numerical start permission remain separate.

Proposed ADR-0185 and architecture revision 162 install concrete governing contracts before
dependent production changes. C1 and resource/reset/mutation contracts precede persistent
lanes; C5 is independently ready after the mutation outcome seam. Fork controls and bounded
lane ownership precede default activation. C3/C6 consumer work follows its actual selected
prerequisites; all method families receive explicit outcomes. Broader 38d–38f and assembled
38g acceptance remain open. Existing-plan/status/index reconciliation remains deferred.

Current source corrects several historical leads: structured Ipopt statistics and gradient
scaling, public PETSc trust-region execution and optional pooled POUNCE batch backends
already exist. The batch pool's carried policy is part of I-C1. Existing cubic certified
fallback and same-point factor reuse are preserved. Native algorithms are library-owned.

## Verification

**Tested, 2026-10-13:** the failure baseline is zero. Native-local selections use the linked
`solver,klu,isolation,uno,petsc` capabilities, the pinned toolchain and scope-aware Arrow
force-validation, with a fresh task-owned `PSE_TEST_RESOURCE_STATE` for each invocation.
Runs are separate, overlapping selections; their counts are not a unique-test total.
The deliberately quiesced canonical profile and shared temporary directories were untouched.

| Command / selected conditions | Result and established boundary |
|---|---|
| `just unit-package pse-math 'test(implicit::affine::tests) \| test(implicit::scalar::tests) \| test(implicit::cubic::tests)'` | 12 passed, 299 skipped, zero failures: affine value changes/IFT, bracket/guard/endpoint and certified cubic controls. |
| `just unit-native-capability-package pse-runtime native-solvers solver,klu,isolation,uno,petsc '<compiler/math/sys selection>' -p pse-compiler -p pse-math -p pse-ipopt-sys` | 42 passed, 1,369 skipped, zero failures. Selection: authored_affine_accelerator_, kernel_nested_implicit_provider_, implicit affine/scalar/cubic tests, nested_selected_products_, diagnostics tests, numerical_convexity_distinguishes_, and all pse-ipopt-sys tests. Exercises actual compiler consumers, serial full-nullspace diagnostics and handwritten TNLP/sequence bridge. |
| Maintained fork: `scripts/pse-env -- cargo test -p feral -p pounce-feral -p pounce-algorithm --lib application_reset --locked` at `d02beafd370fd3815d6c0541fab1630ff839a739` | 13 passed, zero failures: reset reaches library-owned quality/Auto policy and explicit factor-pool controls. No production speed claim follows. |
| `just unit-native-capability-package pse-backend-native native-solvers solver,klu,isolation,uno,petsc '<final native selection>'` | 66 passed, 531 skipped, zero failures. Selection: pounce_convex_, kinsol::build_tests, empty_independent_reset_, empty_pounce_reset_, independent_application_reset_, scip::resource_tests, reoptimization_session_, reoptimized_sequence_, worker_threads_are_scoped_, tagged_native_, highs_, controls_, strict_, same_point_, dynamics_, clarabel_. Includes changed convex QPs versus fresh, reuse/seed refusal, transactional HiGHS updates and actual native/resource provenance controls. |
| KINSOL targeted native setup/map/JVP controls under the same native-local recipe | 14 passed, 564 skipped, zero failures: current-application numerical setup, callback/raw-map and configured method meaning. |
| Explicit linked `scripts.arrow_validation build -p xtask --bin pse-native-lane --no-default-features --features native-solvers --locked` | Actual private child compiled with Arrow validation; exact deployed-binary execution controls are accounted separately below. |
| `.venv/bin/python -m unittest scripts.tests.test_plan30_native_prefix.PreparedKluTests scripts.tests.test_native_operation.NativeOperationTests.test_existing_writable_generation_is_sealed_without_rebuild scripts.tests.test_native_operation.NativeOperationTests.test_external_lookalike_receipt_never_grants_owned_reuse -v` through `scripts/pse-env --` | 10 passed, zero failures: managed KLU receipt selection and mutable-external-prefix nonauthority. |
| `just codegen` | Passed: registry/operational contracts, schemas, Python documents and Ipopt bindings regenerated by their owners. The default is emitted as AllowRebuild with NoPriorStart. A feature-disabled helper warning was subsequently repaired with the matching source cfg. |

Final native/lifecycle verification before the maintainer's stop instruction:

| Command / selected conditions | Result and established boundary |
|---|---|
| `just unit-native-capability-package pse-backend-native native-solvers solver,klu,isolation,uno,petsc '<boundary selection>'` | 39 passed, 559 skipped, zero failures. Selection: execution::tests, pounce::retained::tests, feral_owned_pool_width_, native_layout_and_effective_, native_step_observations_, sample_elimination_factors_, strict_products_, same_point_products_, serial_psd_quality_, equilibrated_conic_, unscaled_conic_, scip::resource_tests. Includes final discard-history and hard-terminal quarantine corrections. |
| `just unit-native-capability-package pse-runtime native-solvers solver,klu,isolation,uno,petsc '<lifecycle/consumer selection>'` | 17 passed, 824 skipped, zero failures. Selection: independent_lane_, parked_, pressure_retires_, optional_parking_, compatible_native_scope_, failed_native_scope_entry_, native_scope_affinity_, native_scope_changes_, supplied_task_scope_, queued_native_step_, cancelled_session_waiter_, abandoned_session_waiter_, dropping_service_, resident_population_, generic_preparation_, structural_tear_deadline_, consistent_controls_, tagged_native_report_failures_. Covers scope width 1→2→1, source/capacity lifetime, fresh fallback consumers and terminal recovery veto. A misplaced rustdoc comment caused one compile warning during this run; its source association was corrected before the final warning-free build. |
| `PSE_MEMORY_MAX=24G scripts/pse-env --native=solver,klu,isolation,uno,petsc -- .venv/bin/python -m scripts.arrow_validation build -p xtask --bin pse-native-lane --no-default-features --features native-solvers --locked` | Final child executable build passed, without warnings, after the scope-history and native-profile corrections. |
| `PSE_NATIVE_LANE_EXECUTABLE=<checkout>/target/debug/pse-native-lane just unit-native-capability-package pse-runtime native-solvers solver,klu,isolation,uno,petsc 'test(math::native_child::tests)'` | 10 passed, 831 skipped, zero failures in 46.398 s. Actual native child and owned fake-process controls include foreign native fingerprint, attempt/response mismatch, death, malformed/oversize/truncated output, cancellation/original deadline, original residual screening and kill/reap accounting. The run had finished before the stop handler searched for its authenticated nextest owner; no further test was launched. |

**Not run:** final `just py-sync-native-correctness` and store-free Python settings/local-runtime
consumer tests after the default activation. Generated Python/schema defaults are regenerated,
but the installed extension has not been refreshed/qualified against this final source. This
is an open 38c closeout prerequisite, not evidence of a failing or passing Python consumer.

No representative difficult-root/globalization or nominal-versus-gradient NLP performance
campaign, large-population throughput/RSS campaign, full dynamics/fitting journey, canonical
durable journey, distributed PETSc run or whole-product qualification was run. Neither
SPQR nor a complete opaque native-memory bound is qualified. These limits preserve G1–G3
and the method-specific support decisions rather than changing acceptance thresholds.

## Outcome

**Implemented:** C1–C6 and I-C1–I-C4 deliver the selected contracts described above. Ordinary
controls now use AllowRebuild with NoPriorStart, with per-profile neutral reset or explicit
fresh fallback. Finite sequence continuation, explicit starts, live integrator history and
original-coordinate scientific assessment retain separate authority. The generated Python
default follows the Rust owner; one-shot tear/fitting/IC paths accept the allowed fresh
fallback and continue to refuse required retention they cannot provide.

**Implemented:** the native owner preserves mutation causes, uses real bulk HiGHS APIs,
installs explicit FERAL widths, refreshes value-sensitive setup/scaling, and parks only
accepted neutral objects under charged residency. New affine/scalar and dynamics observations
reach real scientific consumers. Strict unknown fill and strict child-memory requests refuse.
The private child remains a bounded closed-root capability, not general model serialization.

**Interface-checked:** independent implementation readback accepted the child/native-provider
binding and SCIP terminal quarantine after identified corrections. The target review preceded
dependent code; ADR-0185 remains proposed. Architecture revision 162 and the register hold
enduring contracts and corrected capability triggers. Coordinator finding dispositions remain
with Plan38; this companion does not close all F07/F08/F11/F13 or AU01/AU02 obligations.

### Mistakes found and corrected

Initial integration left old boolean update consumers, invalid native fixtures, an undeclared
direct Rayon dependency and invalid test coercions/identifiers; targeted builds/tests exposed
and repaired them. No scientific or admission barrier was weakened. Codegen exposed a
feature-disabled private helper warning, now guarded by the actual consuming profile.

Review of the new default exposed an implicit convex QP seed cache advertised as no start
support; deleting it preserves cold batch meaning. Clarabel's old equilibration survives
updates, so equilibrated reuse now refuses before mutation. Ipopt's same-structure setting
could preserve old gradient scaling, so every application recomputes it. These are source-led
corrections; no historical wrong physical solution is claimed.

The first final native selection had 65 passes and one failure: request-only HiGHS method
refusal invalidated a still-healthy model. Moving request admission ahead of native invalidation
repaired that failure; the final 66-test selection passed. Earlier failing runs remain failed
invocations. Resource admission also refused an invalid `24GiB` spelling, a second simultaneous
exclusive test request, and the fenced default registry; valid finite `24G`, sequential tests
and a task-private registry resolved those invocation prerequisites without overriding safeguards.

Independent readback identified a stale receipt risk for unmanaged KLU prefixes and numerical
classification of SCIP shutdown uncertainty. Explicit qualifier-selected provenance and hard
terminal Infrastructure cleanup now prevent those paths. No simulated cleanup test establishes
successful drain after a real live-native shutdown error.

The final lifetime readback also found that scope re-entry lost discard provenance and could
silently rebuild under RequireReuse. Fixed-size per-backend metadata now survives teardown,
idempotent clears/resets and failed construction. Existing unseen-backend bootstrap behavior
is preserved; width changes cannot turn discarded native state into a pristine sequence.
The initial runtime consumer check also caught an incorrect local type alias; the import
was repaired before executing the selection.

### Maintainer stop and remaining acceptance

On 2026-10-13 the maintainer requested immediate task/worktree/document closeout, then explicitly
instructed: "please cease any further testing". No additional testing is authorized by this
checkpoint. The final child run above had already completed when cancellation was attempted;
there was no matching live owned nextest process to signal. All owned execution sessions have
completed. Integration and worktree cleanup proceed without additional tests, builds or
qualification checks. A later explicit instruction is required before resuming testing.
The root's `just turn-end` completed its ADR-index regeneration and formatting only; no
functional test, build, hygiene or manual qualification suite ran after that instruction.
All product worker deltas are integrated on main. The clean `p38c-highs`, `p38c-methods`
and `p38c-native-reset` worktrees were removed; local alignment-only commits were not
reapplied as product changes. Their branch history remains in Git. No owned workload
process remained at cleanup; no canonical profile or shared test directory was changed.

This companion remains **in-progress** for final compiled Python consumer acceptance
and any defects it exposes. The scoped native/Rust deliveries and investigation support
decisions are integrated; no additional product package is started during closeout.

All selected additional methods have either a consumed implementation or an explicit current
support decision with a concrete unmet-request trigger. New native NLP batches, general coupled
affine branch selection, SPQR and general child reconstruction are not silently promised.
No comparative speedup or whole-model O(changes) claim is made.

Broader workflow/controllers and topology scope remains 38d–38f. Assembled native/scientific,
integration, Python/durable and performance campaigns, hygiene and manual scope-end checks
remain 38g after the selected functional series. Existing-plan/status/index reconciliation
and retirement remain deferred until series completion. This in-progress companion is retained
for those active dependents and its qualification conditions.
