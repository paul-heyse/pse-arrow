---
title: "Plan46c: Root transformations, preconditioning and responses"
status: done
date: 2026-10-18
adrs: [ADR-0145, ADR-0185]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, numerical-execution, libraries]
---

# Plan46c: Root transformations, preconditioning and responses

## Target and foundation

Own AF04/AF05/AF07/AF16 and I03–I06. The [coordinator](46-execution-speed-and-numerical-efficiency.md) owns RC04/RC06/RC07/RC10 and current dispositions. C01–C06 adopted functional scope and bounded decisions are complete on assembled main, including PETSc and joined runtime consumer controls. Final qualification and whole-series acceptance remain owned by [G06](46g-investigations-qualification-and-design-publication.md#g06); completed decisions do not remove the retained AF16 strict opaque-fill limitation.

**Interface-checked foundation:** root execution and NLP presolve have distinct eligibility and reconstruction obligations. KINSOL BlockFactor uses BTF diagonal blocks and fresh Markowitz setup; its cubic reservation covers conservative peak factorization scratch, not measured retained factors. Markowitz's value-dependent pivots replace both symbolic and numeric phases. PETSc GMRES/ILU and nonlinear Schwarz already exist. Square response faer LU separates symbolic and numeric preparation, with escaping immutable factor ownership and truthful strict opaque-fill refusal. Full dense scaled SVD remains a separately requested product. C02 and C04 replace the former per-application scratch allocation described in the review.

The target uses one transformation authority and native algorithms, with exact consumer obligations. It does not choose a universal root solver, eliminate necessary coupled work or reinterpret a successful factor solve as rank evidence.

<a id="c01"></a>

## C01 — Consumer-qualified root transformations and formulations

**AF04; I03; RC04.** Specify primal reconstruction eligibility for root/initialization separately from optimizer multiplier/KKT recovery. Integrate qualified alias/affine connection elimination into actual root consumers with complete original row/bound/physical checks. Retain the native NLP's stricter dual obligations. Caller expression/provider revisions remain in reuse identity; a library warm fingerprint alone is insufficient.

Compare further interval/bound contraction, causal/BTF execution, coalesced tiny blocks and regular implicit condensation independently. Coefficient magnitude relative to a presolve threshold is not scientific insignificance: rounding, physical scales, guards, eliminated-variable bounds and reconstruction error must qualify each transform. A scalar structural block is not proof of unique root/bracket. Condensation can increase fill, conditioning cost or property solves.

**Decision/targeted acceptance:** unequal scales, near-threshold coefficients, eliminated bounds, changed provider revisions and reconstruction failure; same supported recycle/tear formulation/start/branch. Compare complete original accepted operation including preparation, property/derivative work and reconstruction. Choose a conforming in-bound transform or retain a named limit; do not silently reenable all NLP passes.

**Implemented; I03 decision:** adopt the pinned POUNCE `linear_eq_plan` primal map for qualified Root and conditional Initialize consumers. Eligibility is independent of optimizer dual/KKT recovery. Equality, coefficient and feasibility transformation filters are zero: a represented small coefficient is retained, and no solver stopping accuracy is changed. Infeasible, capped, nonfinite, identity or nonsquare plans are declined automatically; a required unavailable affine pass refuses explicitly. Both the original contract and the actual reduced contract retain selected-adapter admission. If elimination turns supported half-lines into an unsupported box, automatic policy keeps the original native route.

The wrapper projects starts and primal warm payloads, transports physical scales/budgets, and reconstructs the complete original candidate. Original residual/Jacobian callbacks still execute over the full original point; a final independently counted original evaluation checks every row, bound and provider/guard obligation. A zero-dimensional library result is labeled separately from native convergence. Layout identity contains the actual support/order/map/scales and profile, while current facts and original data/provider context remain in transformation and numeric reuse identity. Conditional initialization rebuilds proof facts after each refill from authored source-frozen values and local proof slots; stale presolve facts are not reused. The central native and runtime selections in the Outcome exercise these integrated consumers.

**Tested:** `actual_root_primal_affine_recovers_original_scales_and_detects_reconstruction_failure` gives the same `[1,3]` branch for reduced and original KINSOL runs at unequal physical scales, then detects an inconsistent affine reconstruction through original feasibility and preserves an original domain failure. The latest tiny control used 16 original callbacks after reduction versus 15 without it; reduced dimension is not evidence of reduced total work. Separate controls preserve a `1e-15` coefficient and eliminated bounds, refuse stale structure, and preserve native bound eligibility.

Further I03 choices remain bounded. The executed FBBT control for `x²=4` over `[-3,3]` retains both roots and zero; an opaque tape retains an entire interval. Contraction therefore grants neither a unique scalar root nor a branch. Retain the existing causal/BTF execution and its structural meaning. No new coalescing, scalar substitution or implicit condensation is adopted. Revisit coalescing when G01 identifies per-block preparation/scheduling as material in the same accepted physical operation; revisit condensation for a concrete regular eliminated subproblem with qualified derivatives, provider refresh, conditioning, fill and storage. The executed full-operator/zero-A11 control in C03 demonstrates why an arbitrary SCC split is insufficient.

<a id="c02"></a>

## C02 — Retain BlockFactor application scratch

**AF05; concrete correction.** Put global/per-block RHS, result and gather/scatter buffers with the actual KINSOL Blocks owner, using stable prepared maps and admitted storage. Eliminate per-application temporary vectors and repeated map interpretation. Every setup refreshes current numeric factors under its current contract; partial setup failure invalidates all affected factors.

Preserve the true coupled JVP and right-preconditioner semantics. Keep the existing conservative reservation until a different producer/factor path supports a sound replacement; reduced scratch or low RSS does not justify removing the cubic allowance. **Targeted acceptance:** repeated applies, zero crossings, multiple blocks, partial failure then recovery, cancel/reset and unchanged complete root checks; count allocations independently of factor/setup cost.

**Implemented and Tested:** the KINSOL Blocks owner retains per-block RHS and complete result storage with prepared gather/scatter maps. Repeated applies publish only a complete result. Numeric setup remains fresh; failed partial setup invalidates the factors before recovery. The coupled JVP and right-preconditioner semantics are unchanged. Replaced per-apply vector construction was deleted; the conservative cubic factor reservation remains, with explicit retained scratch accounted separately.

`repeated_block_applies_allocate_zero_after_native_scratch_warmup` counts **zero Rust allocations/bytes for 32 warmed applies**, excluding factor/setup. Companion controls exercise storage reuse, zero crossings, multiple blocks, partial failure/recovery, cancellation/reset, changed source data and actual coupled KINSOL solves. These are bounded allocation controls, not a complete native heap or RSS bound.

<a id="c03"></a>

## C03 — Choose scalable native factors/preconditioners using existing profiles first

**AF05; I04; conditional RC07 via G02.** Compare current KLU, BlockFactor and existing PETSc GMRES/ILU and nonlinear Schwarz on weak/strong coupling, dominant SCCs, nonnormal/fill growth and difficult starts. Nonlinear Schwarz is not linear field splitting. Existing KINSOL line search, setup intervals, forcing, Anderson and Krylov settings are calibration dimensions, not new algorithms/defaults.

For reuse compare explicit Gilbert–Peierls/KLU/faer symbolic-numeric alternatives against Markowitz, preserving pivot/fill/stability and complete admission. Do not extract a symbolic phase from value-dependent Markowitz. Stronger triangular/overlap/Schur candidates require actual operator and preconditioning matrix, split provenance, refresh triggers and failure behavior; a zero A11 can defeat an A11 Schur preconditioner. No arbitrary SCC split or explicit inverse.

**Decision/targeted acceptance:** factor setup/apply/backsolves, Krylov/nonlinear work, memory/admission refusal, original residuals and recovery. Adopt qualified adapter/profile changes and delete unused candidates; inner threading/process changes are selected only through G02. A lower iteration count with more total work is not a win, and a tiny timing does not choose a global default.

**Tested; I04 decision:** the committed comparison control executes 32 KINSOL routes: four eight-variable matrix families, two starts, setup intervals 1/10, and KLU versus BlockFactor/GMRES. Families cover weak tridiagonal coupling, a strong dense nonlinear SCC, asymmetric nonnormal coupling and a fill-producing ring. All retain original physical feasibility. For the weak case at start zero/setup 10, KLU used 6 evaluations/3 nonlinear iterations versus BlockFactor's 18/3. For the strong case at the same start/setup, KLU used 11/8 versus BlockFactor's 23/4. Fewer nonlinear iterations did not imply less total work. No universal solver, profile or setup default is changed.

The explicit FERAL Gilbert–Peierls/Markowitz control performs **16 factorizations** across four families and two numeric refreshes. Gilbert–Peierls reuses a genuine symbolic analysis; Markowitz remains value-dependent. All physical linear solves pass. Fill agrees at 22, 64 and 34 entries in three families; the nonnormal family uses 27 with Gilbert–Peierls versus 22 with Markowitz. Observed growth is 1 in these small controls. This does not select a general pivoting winner or justify removing Markowitz's conservative reservation. C04 separately qualifies faer symbolic reuse for its actual response consumer.

The full matrix `[[0,1],[1,0]]` solves while its zero A11 block cannot factor. No arbitrary Schur, triangular, overlap or explicit-inverse candidate is installed. A stronger preconditioner requires a concrete original operator, qualified split and refresh/failure/storage behavior, plus representative G01 evidence of dominant factor/apply work.

**Historical executor boundary:** the first eight-route PETSc GMRES/ILU comparison and nonlinear Schwarz/domain/gate attempt entered cold PETSc setup and was cooperatively stopped and drained before tests. The central native selection in the Outcome subsequently passed these controls, including the distinct mathematical composition. NASM requires a declared `BlockComposition`/domain witness; the generic Root adapter correctly refuses to infer it from an arbitrary matrix. G02 owns any thread/process adoption; the bounded inquiry selects no change to serial root profiles and does not qualify the restricted NativeChild protocol as generic PETSc isolation.

<a id="c04"></a>

## C04 — Retain response symbolic plans and action-executor scratch

**AF07.** Share exactly compatible immutable symbolic analysis by original support/order/scaling/settings as actually consumed; construct fresh matrix-specific numeric factors. Keep factor point identity explicit. Retain batched RHS and residual/MemStack buffers in each actual action executor, not unsynchronized mutable fields on an Arc-shared escaping factor.

Migrate response/predictor/steady-fitting consumers that construct repeated same-layout factors. Separate action availability from requested rank/spectrum and strict eligibility. **Targeted acceptance:** multiple points same layout, changed support/order, singular→regular recovery, concurrent independent RHS owners, batch sizes, late failure and escaped factor release. Count symbolic/numeric/solve work separately. Existing fixed-layout native KLU reuse is preserved and is a different owner.

**Implemented:** exactly compatible SquareScopes share the immutable faer symbolic LU; each point constructs a fresh numeric matrix/factor. Compatibility checks cover original identity, row/variable order and actual support, with normalization checked at symbolic consumption and fixed serial/default factor settings. Root response/predictor and steady-fitting consumers retain their layout owners. A shared passive allocation anchor survives either scope or escaping-factor drop order. Root caches conservatively retain the original request allowance under Local policy; this is not a complete fill bound. Cache publication follows producer admission, so a first strict refusal retains no empty cache reservation.

Each `SparseActionExecutor` owns its RHS, solution, residual and MemStack storage. Predictor clones obtain separate mutable executors. Immutable scaled matrix norms are prepared once for backward-error checks. Replaced action scratch construction was deleted from migrated repeated-action consumers; standalone convenience calls still construct a private executor for their single action.

**Tested:** same-layout multiple points share symbolic analysis while retaining different numeric point identities; independent executors retain distinct buffers; changed order refuses reuse; singular action failure recovers at a regular point; strict action refusal leaves a Local executor usable; both allocation-anchor drop orders release correctly. The 32 warmed two-RHS batch control allocates only its 32 escaping result matrices, with setup excluded. Central native and runtime consumer controls below also cover constructor/cache integration and strict empty-cache/zero-charge refusal. These tests do not establish concurrent workload acceleration.

<a id="c05"></a>

## C05 — Qualify separately named sparse regularity/rank only for consumed meaning

**AF07; I05; RC06.** Identify a real consumer for a non-spectrum sparse assessment. Compare suitable SPQR or singular-value estimators against its required meaning; preserve full scaled SVD and its relative singular cutoff for existing public spectrum/covariance semantics. Successful LU, backward error, QR pivot rank and SVD rank remain distinct.

Qualify exact feature/linkage, Rust/native ownership, storage, interruption and result method/scaling/cutoff before binding. Vendored SPQR source alone is insufficient. Migrate only consumers accepting the new product; no silent substitution. **Decision/targeted acceptance:** exact and near rank loss, scaled dependencies, nonnormal systems, permutation/support changes, method labels and independent actions remaining available. If no consumer accepts different meaning or the route is unqualified, retain current SVD and record the precise limitation instead of adding an unused API.

**Interface-checked and Tested; I05 decision:** retain the full scaled SVD and its relative singular cutoff. Current public square response spectra and fitting weighted-response identifiability/covariance consume singular values and, for fitting, the right singular basis. Engineering accuracy and predictors consume actions; they do not require a new sparse-rank meaning. No current consumer is migrated to a differently named sparse assessment, and no unused rank API is added.

The near-rank-loss control with a singular value `2^-30` and relative cutoff `1e-8` withholds the SVD response at rank 1/2 while an independently checked predictor action remains available. Exact-zero numeric LU construction may succeed, but its action then refuses. Successful construction/backward error is therefore not relabeled as SVD rank or a forward-error certificate.

Exact-source inspection found that the selected `suitesparse_sys` image enables static KLU/AMD/COLAMD/BTF, not SPQR. Vendored SPQR source does not establish an enabled ABI, Rust owner, complete storage or interruption contract. Revisit only when a real consumer accepts a specified distinct method/scaling/cutoff/order product and its exact feature/linkage/ownership/storage/interruption route is qualified. Existing SVD spectrum and covariance semantics remain unchanged.

<a id="c06"></a>

## C06 — Resolve strict sparse-response consumer/producer eligibility

**AF16; I06; RC10.** Identify where actual consumers request strict linear-storage eligibility versus ordinary deployment-local actions. Preserve ordinary response optimization and conservative unknown-fill refusal. Compare source-owned faer allocation knowledge or an alternative factor owner only to the extent needed for those consumers; no deployment-wide memory/provenance proof campaign is created.

Reuse does not make opaque heap fill known. Any producer improvement must cover actual construction/action/fallback lifetimes and refuse before effects when unsupported. RSS and deployment reservation are not complete bounds. **Decision/targeted acceptance:** strict refusal before derivative evaluation, Local success, escaping factor/action storage and failed setup/recovery. A policy relaxation is outside the recommended RC10 boundary and requires a new concrete decision. If strict support remains unavailable, state its exact consumer consequence and observable revisit trigger; do not mark it implemented by C04.

**Implemented; I06 decision:** preserve strict unknown-fill refusal and Local response optimization. Actual strict selections arise through task admission for declared foreign storage and the restricted NativeChild contract; ordinary deployment-local response use does not acquire that policy. Runtime fresh partials and coordinate accuracy now call sparse producer admission before worker/derivative evaluation. Optional strict accuracy failure retains its typed cause and leaves the base scientific solve available.

Source inspection of faer 0.24.4 shows private numeric fill and nested heap allocations beyond declared MemStack scratch. Reuse and deployment reservations do not bound complete construction/action/fallback/escape lifetimes. The existing native KLU owner is not a qualified escaping response-factor replacement. No alternative producer or strict-support claim is adopted. A strict consumer continues to receive `Unsupported` for this opaque-fill response producer.

**Tested:** backend strict requests refuse before evaluator entry; Local factors/actions succeed; strict executor construction/action refuses without invalidating later Local use; failed setup/action and escaping owners recover/release as covered by C04. Central runtime controls assert zero evaluations/factorizations and, for fresh root partials, no cache publication or retained allowance on refusal. Revisit for a concrete strict consumer and a complete producer lifetime bound or qualified alternative owner. RSS, scratch counts and a policy relaxation do not satisfy this trigger.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C01 | RC04 decision; original reconstruction/transformation facts | I03 affine adoption; original bound/scale/branch controls; actual Root/Initialize consumers | done |
| C02 | Existing Blocks owner/admission | Retained apply storage/maps and failure recovery; replaced temporaries deleted | done |
| C03 | C02; current profiles; G01 observation; RC07 only for expanded routes | I04 KINSOL/FERAL comparison and bounded no-default-change decision | done |
| C04 | Exact response layout/factor/action ownership | Shared symbolic plans, fresh numeric factors, isolated action scratch and allocation anchors | done |
| C05 | RC06 decision; named consumer semantics; exact binding eligibility | I05 retain-SVD decision; no unconsumed sparse-rank binding | done |
| C06 | RC10 decision; actual strict consumer inventory | I06 retain strict limitation; admission before derivative effects | done |

## Historical executor verification and handoff

**Tested:** on the isolated C worktree with the pinned toolchain, private resource state and native-local solver/KLU capability route, the following scoped command compiled without warnings and ran **34 tests: 34 passed, 0 failed, 324 unselected**, against a zero-failure baseline:

```bash
BASH_ENV='' PSE_TEST_RESOURCE_STATE=/tmp/pse-plan46-c-resources-20261018 \
  just unit-native-capability-package pse-backend-native \
  pse-backend-native/kinsol solver,klu \
  'test(kinsol::block) | test(root_primal_affine) | test(interval_contraction) | test(square_response) | test(engineering_accuracy_square)' \
  --execution-effects native-local --success-output immediate
```

The 32 KINSOL profile routes and 16 FERAL factorization cases are cases inside these tests, not additional test counts. Allocation controls exclude setup and count Rust allocations; they do not qualify all native heap use. These bounded synthetic controls establish the stated behavior, not a universal speed winner or assembled scientific qualification.

At that executor endpoint PETSc comparison/Schwarz/gate tests and joined runtime consumers were **not_run** after owned cold setup was canceled and drained. The 34-test backend-only run did not validate the later cache-publication test or assembled conditional `source_frozen_ids` proof binding. These are historical exclusions: central native and runtime selections below subsequently exercised them. G06 owns final qualification. No formatter, lint, hygiene or integration suite was run by the C executor mid-plan.

Library inquiry used the native-solver and Symbolica/faer skills, current Context7 discovery and exact pinned source. No stale capability map or utilization record substitutes for source/consumer evidence. Root primal recovery and optimizer KKT acceptance remain distinct, and scientific acceptance accuracy 0.001 is unchanged.

## Outcome

Recorded 2026-10-18.

**Implemented:** C01 primal affine integration, C02 retained BlockFactor scratch, C04 retained response symbols/actions and allocation anchors, and C06 admission before derivative effects are integrated on main. I03–I06 decisions and all adopted C01–C06 functional work are complete, with the targeted native and joined runtime evidence below. The C worktree is integrated and removed. G06 records selected final qualification and whole-series acceptance.

Mistakes corrected during targeted controls: a new KINSOL comparison fixture initially used unsupported two-sided bounds; the corrected control uses supported original bounds and separately tests automatic fallback when affine bound transfer creates an unsupported reduced box. Batch backward-error checks initially retained an extra per-call norm allocation; preparing immutable matrix norms removed it. The later cache correction prevents strict refusal from retaining an unconstructed producer allowance.

Deliberate deviations and retained limits: full original callbacks still execute after primal mapping; optimizer presolve is not broadly reenabled; Markowitz is not given a fictitious symbolic phase; sparse actions are not rank certificates; strict opaque-fill support remains unavailable under AF16; and no solver/profile/thread/process default is selected from tiny controls. The retain-SVD and retain-strict-refusal decisions complete their bounded inquiries without implementing a new rank or strict factor producer.


### Central native qualification — 2026-10-18

**Tested, zero baseline:** `BASH_ENV='' PSE_TEST_RESOURCE_STATE=/tmp/pse-plan46-root-resources-20261018 just unit-native-capability-package pse-backend-native pse-backend-native/kinsol,pse-backend-native/petsc,pse-backend-native/idas,pse-backend-native/diffsol solver,klu,petsc 'test(petsc) | test(square_response) | test(affine) | test(block) | test(shared_conditional_bindings) | test(dynamics::)' --success-output immediate`
passed **163/163**, 308 unselected, native-local correctness mode. This includes the eight
PETSc Newton/GMRES/ILU physical controls, derived NASM/domain/gate/lifetime checks, backend strict
producer refusal and compact conditional bindings with unused authored fixed/parameter
declarations. Expected contained-panic output is part of a passing PETSc failure control.
No native test failed and no warning remained at this numerical endpoint.

**Measured, I04:** PETSc controls used two starts across four coupling/fill families, with
9–15 evaluations and 2–4 nonlinear iterations; factorization/proof coverage remained unknown,
not zero. These are eight internal cases in one unit test, not eight extra test identities.
Combined with the earlier KINSOL/FERAL comparisons, they select no new universal solver,
preconditioner, refresh or threading default.

### Central joined runtime consumers — 2026-10-18

**Tested, zero baseline:** G06's full-native/canonical/compiled runtime selection ran 97 tests:
96 passed and one initialization plateau fixture failed because it omitted the required
selected authored start. The correction adds explicit missing-start refusal and supplies
authored x=1. The subsequent native-local consumer selection ran **11 tests, 11 passed /
0 failed**, including corrected initialization, engineering accuracy, strict composed-work
refusal, root predictors and fitting response/rank consumers. Exact selections and raw results
are retained in `/tmp/pse-plan46-integrated-runtime.log` and
`/tmp/pse-plan46-runtime-consumers.log`; G06 owns the invocation and final qualification.
The original 97-test run remains a failed run; the affected consumer rerun closes its fixture
failure without implying a second all-97 run or whole-product qualification.
