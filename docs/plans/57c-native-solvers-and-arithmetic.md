---
title: "Plan 57c: Native solvers and arithmetic"
status: in-progress
date: 2026-10-27
adrs: []
review_sources: [docs/library_leverage/reviews/native-solver-integration.md, docs/library_leverage/reviews/result-data-and-prepaid-accounting.md, docs/library_leverage/reviews/numerical-workflows.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, design, libraries, numerical-execution, environment, validation]
---

# Plan 57c: Native solvers and arithmetic

## Target, baseline and decision boundaries

**Proposed:** make native supplier identity truthful, resolve the complete FMA-free kernel route,
move bundled SUNDIALS to the reviewed release after its decision prerequisite, consolidate the
three faer-LU mechanics and native gate acquisition, and preserve terminal resource causes
through Diffsol. This companion covers CBO-026–031, records CBO-032's retirement and supplies
native bounds to CBO-001. The [coordinator](57-library-leverage-and-correctness.md) is the sole
full CBO disposition owner; package progress lives here. This is implementation planning only.
No native image, provider, solver, dependency, fork, ADR or architecture change is authorized by
creation of these documents.

The consumed G6 review is
[native-solver integration](../library_leverage/reviews/native-solver-integration.md), including
its full-closure FMA correction, published at `93946f572`, against product source `4f52fd239`.
The authoring checkout was clean at `93946f572`; input accounts and patterns revision 4 are at
that revision, while `1540fd164` contains the new counterexamples. Core 3.4, its
efficient-architecture heuristics and process-simulator 1.5 apply.
Blueprint §15.4, §16.5, §18.2, §18.3, §18.6, §18.8 and §18.10 own the consumed contracts.
The active native-solver-libraries reference supplies the source-backed KINSOL/IDA/IDAS,
Diffsol, sparse-factor and class-specific solver routes; its probe receipts are not evidence
that this checkout's changed configuration works.

The operator selected U1 conditional upstream options without contact, U2 recorded-point replay
and U3 SurrealDB canonical storage. U2 does not weaken the standing FMA-free arithmetic
requirement. An identity or provenance label can identify arithmetic but cannot make FMA-free
execution true. Formal rule changes and deferred-trigger decisions remain conditional until
confirmed and routed by the coordinator.

## Focused foundation assessment

There are three separable foundations, each with a different remedy.

**Supplier identity.** `pse-backend-native/build.rs::kinsol_profile` copies dependency declaration
text into `kinsol-source.txt`; it does not include the local SUNDIALS delta. KINSOL adds native
version/compiler/provider facts, while the dynamics adapters in `execution/dynamics.rs` inherit
`BackendExecution::build() = None`. Local native C changes at an unchanged package version can
therefore preserve identity; an IDAS-only configuration can miss a native-version move as well.
A shared supplier fact must reach every consuming profile before numerical supplier changes.
Hash the maintained complete delta or an equivalent authenticated source receipt, not arbitrary
build-directory contents or a serving-time source scan. Construction-once loaded artifacts stay
valid for their original premises.

**Arithmetic.** Installed faer 0.24.4 enables pulp `x86-v3` itself. Pulp 0.22.3 has explicit fused
kernels. Faer's `linalg/matmul/mod.rs` also chooses `private_gemm_x86::{Avx512,Avx256}` directly,
uses `nano_gemm::planless` for small products and `gemm` on another path. These faer LU/SVD/EVD
consumers are confirmed. Nalgebra-to-`matrixmultiply` is a conditional closure-audit lead, not a
proven production call merely because those dependencies resolve: the current residual-LM QR
uses scalar/custom dot operations. Include that route only when a production call is traced.
A pulp-only override and removing `target-cpu=native` do not settle the
whole closure. This affects sparse LU through dense/supernodal operations, covariance SVD and
uncertainty PSD admission through EVD. Exact source establishes exposure, not a measured
cross-CPU numerical difference. Keep the arithmetic remedy open until its complete consumed
closure is demonstrated; retain the scientific controls rather than changing acceptance to
allow the current violation.

**Lifecycle and ownership.** The three faer owners share mechanics, but not factor lifetimes:
square response shares immutable symbolic analysis and fresh numeric factors; arclength retains
an attempt workspace; Diffsol has an infallible linearization/deferred-error interface. A narrow
helper can share scratch admission, errors, finite solves and allowance formulas. A universal
faer/FERAL/KLU trait would transfer unrelated lifecycle and inertia distinctions into consumers.
FERAL's condition estimate, KLU's refactor acceptance and backward-error checks answer different
questions and should remain separate. Similarly, `Retained` already owns native session storage;
the repeated PETSc/SCIP/IBEX wait loop needs one acquisition helper, not a universal session
framework.

Resource bounds also have different strengths. Faer predicts scratch with `StackReq` but not
complete numeric factor storage. FERAL LDLᵀ has supplier-side conservative storage support in
the maintainer fork; FERAL Markowitz LU does not. SUNDIALS workspace reporters are post-hoc and
deprecated, while local predictive formulas are version-coupled. Dense opaque allowances are
intentionally structure-blind complete bounds, not accidental mirrors to delete. Native
contributions must compose with [57a](57a-result-data-and-prepaid-accounting.md)'s CBO-001 owner;
no local improvement may silently tighten accepted dynamics or diagnostic sizes.

## Resulting boundaries

The build owner publishes one immutable supplier/configuration premise for each consumed native
realization. Runtime adapters frame it into their existing profile build identity. Scientific
execution continues through class-correct libraries, with original-coordinate acceptance
independent of solver status. Explicit arithmetic configuration belongs to the supplier/build
owner and is tested along all relevant kernel routes; profile identity records its actual policy.

The faer helper receives caller-selected scratch budget, parallelism and observation prefix. It
queries `SymbolicLu::{factorize_numeric_lu_scratch,solve_in_place_scratch}`, admits before
`MemBuffer::try_new`, uses `MemStack` for factorization/solve and checks finite outputs.
Callers retain symbolic keys, leases and factor ownership. A typed terminal-cause owner bridges
Diffsol's limited error interface without conflating recoverable singular trials with allocation
or contract failure. A gate helper coordinates existing `NativeWait`, cancellation and compute
admission while each adapter retains global state and destruction policy.

Replacing SUNDIALS is a release move, not replacement of its integration. Budget, fixed-pattern,
checkpoint and observation hooks remain local. Only the supplier-owned numerical quadrature
correction retires when the upstream release supplies its behavior. All changes remain in the
existing ownership chain; no compatibility execution route is added.

## Package state and sequencing

| Package | Status | Owner | Completion |
|---|---|---|---|
| C01 | implemented; provider followthrough and joined qualification pending | [Supplier identity](#c01) | Same-version delta and IDAS-only moves change the consumed build premise; controls pass |
| C02 | implemented; joined qualification pending | [FMA-free closure inquiry and realization](#c02) | Complete FMA-free realization demonstrated; an unresolved inquiry outcome leaves arithmetic conformance open |
| C03 | in progress | [SUNDIALS release move](#c03) | Decision route, identity, integration-hunk rebase, formula/status migration and targeted controls complete |
| C04 | in progress | [LU mechanics and native bounds](#c04) | Three faer consumers migrated; supplier LU bound integrated without unsupported tighter limits |
| C05 | implemented; joined qualification pending | [Diffsol terminal causes](#c05) | Resource/contract causes survive solver retries and all affected integration paths |
| C06 | implemented; joined qualification pending | [Quiescent gates](#c06) | Gate consumers migrated with their health/poison/panic behavior; HiGHS inquiry disposed |
| C07 | retained dispositions; joined reconciliation pending | [Retained suppliers and conditional upstream work](#c07) | Retention reasons and observable triggers reconciled; no unsolicited rebase/contact |

C01 precedes every SUNDIALS source/hunk change. C03 preserves the existing contracts and follows
ordinary dependency/bug-fix handling; an actual contract change needs its exact decision route
and operator disposition before dependent work. C04 and C05 share a terminal-cause
classification contract and should land as integrated usable slices. C02 can proceed as source
research alongside them, but new arithmetic-sensitive acceptance cannot claim compliance until
its realization passes. C06 depends on the shared notification/cancellation contract, not on the
whole lifecycle companion. C07 supplies bounded trigger outcomes; it is not a mandatory Uno or
Diffsol upgrade. Global assembled qualification belongs to 57j after functional implementation.

<a id="c01"></a>

### C01 — Authenticate the consumed supplier build in every native profile

Source: [CBO-028 identity prerequisite](../library_leverage/reviews/native-solver-integration.md#cbo-028).

Use existing build-receipt and framed-identity owners to include exact supplier version, local
delta and relevant build/provider configuration. For bundled SUNDIALS, hash the maintained
`vendor/deltas/sundials-sys.patch` together with its authenticated upstream baseline, or the
same facts from a checked equivalent receipt. Ensure that receipt describes the actual selected
patched tree; a stale patch digest is not the identity of edited native source. Generate/verify
the delta through its maintenance owner and give Cargo the necessary input invalidation. Do
not introduce startup scanning of a previously admitted artifact. Selected external providers
consume their actual provider receipt, not an unused bundled patch as a substitute.

Both IDAS and KINSOL expose this premise through `BackendExecution::build`/`Table::build_identity`
even when KINSOL is unlinked. Review Diffsol and other locally patched supplier identities with
the same principle: the currently unchanged Diffsol numerical source is a fact, not permission
for future same-version numerical edits to remain invisible. Preserve `pse.native.build.v1`
framing if this is a missing-input fix within ADR-0108; if completing it changes the hashing
contract, stop that dependent implementation for the coordinator's ADR/design-review route.
The draft records the alternative and does not choose the operator's rule disposition.

**Acceptance and deletion.** Add pure identity controls showing that a same-version changed
SUNDIALS hunk changes IDAS and KINSOL profiles, identical selected inputs remain stable, unrelated
unconsumed build facts do not change a profile, and missing required facts refuse. Include an
IDAS-only feature configuration. Replace declaration-only completeness assumptions in comments
and tests, rather than keeping two competing build identities. Compile and run targeted
`pse-backend-native` identity/profile units under their supported feature closure; provider
construction tests follow the environment's owning routes when execution is authorized.
This establishes identity coverage, not native numerical equivalence or no-FMA behavior.

<a id="c02"></a>

### C02 — Settle and realize a complete FMA-free supplier route

Source: [FMA-free premise correction](../library_leverage/reviews/native-solver-integration.md#faer-kernels-and-the-fma-free-premise).

**Bounded inquiry.** Trace the actual consumed sparse LU, dense SVD/EVD, matmul and relevant
nalgebra routes through exact release sources and enabled features. Inspect pulp, private GEMM,
nano-gemm, gemm and matrixmultiply wherever reached, including small/large shapes and
supernodal/simplicial alternatives. Separate explicit fused intrinsics/`mul_add` from compiler
contraction. The settling evidence is a concrete supplier configuration or maintained patch
that disables fused operations throughout this consumed closure, together with a credible way
to force/exercise each relevant path. A list of CPU names or one pulp ISA choice is insufficient.
Use Context7 first, then exact source; its current faer result describes explicit SIMD but does
not offer a documented complete disable switch.

Compare, in this order, an existing complete upstream override, a maintained supplier patch
covering all dispatch owners, and a source-backed scalar/nonfused configuration preserving the
consumed library algorithms and problem classes. A released U1 upstream override is a candidate,
not an available dependency. Do not substitute a home-written solver or drop covariance/PSD
checks to avoid the issue. Preserve admitted parallelism and local machine resource policy;
forcing serial host execution is not an arithmetic remedy. `Par::Seq` is retained where the
scientific contract already selects it, not imposed as a new workstation cap.

**Outcomes and dependent work.** If a complete route exists, record exact features/patches and
owned update obligations, implement it through the supplier/build owner, include its arithmetic
premise in C01's identity and remove replaced dispatch assumptions. If it cannot be established
within the bounded inquiry, record the exact remaining routes and a proposed next decision in
the coordinator. Do not mark the arithmetic requirement satisfied, hide the blocker behind U2,
or adopt a throughput regression limit invented by this plan. Any proposed policy weakening
requires explicit operator confirmation and the existing decision route before dependent work.

**Acceptance.** Static/source review and focused generated-code or dispatch controls must cover
all selected kernel paths; an ordinary numeric pass alone cannot prove that FMA was absent.
Use independent adversarial nonfused-versus-fused arithmetic cases at actual entry points and
force relevant dispatch variants where supported. Run targeted sparse response/arclength/
Diffsol, fitting covariance SVD and uncertainty PSD-EVD controls for the resulting configuration.
Keep original residual, derivative, rank/PSD and scientific tolerance checks. At scope end 57j
assembles the selected journeys. No performance benchmark is mandatory; measure only if a
concrete throughput decision needs it, and do not claim an unmeasured speed or cost result.

<a id="c03"></a>

### C03 — Move bundled SUNDIALS to 7.9.0 after identity and decision prerequisites

Source: [CBO-028 SUNDIALS](../library_leverage/reviews/native-solver-integration.md#cbo-028).

R-41's upstream quadrature-fix trigger has fired in the source review. It has no owning ADR
and requires upstream source comparison plus the three corrected-history controls before
retiring the workaround. Treat the release move/hunk retirement as ordinary dependency/bug-fix
work when the existing numerical/binding contract is preserved; a generic trigger label alone
does not require a new ADR. If investigation reveals a changed kernel, binding, numerical or
hashing contract, name that exact impact and obtain its decision route before dependent work.
Prefer the reviewed 7.9.0 native tree within the maintained `sundials-sys`
0.6.2 integration, with exact provenance, rather than jumping to an uninspected 8.x release.
A later upstream `sundials-sys` release is a conditional U1 alternative once it meets the same
contract. A local package-version change is optional attribution, not a substitute for C01.

Rebase the complete integration delta, including `idaa.c`, `idaa_io.c`, `idas_impl.h`,
`idas_io.c`, `idas.c`, `sunlinsol_klu.{h,c}`, public IDAS headers and `wrapper.h`; retain bounded
checkpoint population, original-clock accepted-step/replay budgets, fixed-pattern KLU and work
notifications. Remove the local `IDAInitialQuadDerivatives` post-`IDACalcIC` correction and
helper-only factoring when the upstream predictor path supplies it. Preserve complete
byte-provenance/delta maintenance and the native provider/freshness wiring.

Re-derive all version-coupled Rust consumers before accepting the move: SPGMR/SPFGMR layout,
KINSOL header/Krylov/Anderson storage, BBD bytes, local-error payloads, error-handler stack,
IDAS statistics/history bounds, KINSOL counter/restart boundaries and KLU refactor policy.
Map `IDA_TOO_CLOSE` and other changed returns exhaustively; verify reinitialized counter
semantics, the 7.5 Anderson-depth fix for retained sessions and captured Anderson failures.
Review 7.7 namespaced CMake options and deprecated workspace APIs. Keep predictive formulas
local and source-derived; moving them into vendored C adds rebase burden without an upstream
prediction capability. Workspace reporters reconcile retained usage after construction; they
do not authorize first allocation. SUNDIALS 8.x workspace removal remains a later trigger.

**Acceptance.** Target the existing controls
`quadrature_histories_use_corrected_algebraic_initial_state_and_sensitivities`,
`quadrature_histories_use_corrected_algebraic_scheduled_state_and_sensitivities`, and
`corrected_quadrature_history_callback_failures_stop_before_physical_steps`, plus retained
scheduled-DAE products, KINSOL session reuse/restart total caps and fixed-pattern KLU refusal/
fresh-numeric recovery. Add changed-status, counter-reset and resource-refusal controls where
existing tests do not expose the release changes. Preserve callback panic containment and
truthful partial trajectories. Compile and run the selected native package tests through
`scripts/pse-env --native --`/native-aware recipes with explicit force-validation. An image
rebuild and broader Plan49-derived/native journeys are selected by 57j, not started during
plan authoring. The move can legitimately change numerics/counters under a new build identity;
recorded earlier points/results retain their original identities.

<a id="c04"></a>

### C04 — Share faer LU mechanics and move native bounds to their allocating owners

Sources: [CBO-026](../library_leverage/reviews/native-solver-integration.md#cbo-026) and
[native CBO-001 input](../library_leverage/reviews/native-solver-integration.md#cbo-001-native).

Create one crate-private helper around faer 0.24.4
`factorize_symbolic_lu`, `SymbolicLu::factorize_numeric_lu`, `LuRef::solve_in_place_with_conj`
and the scratch APIs named above. Migrate square-response `SparseFactor`, arclength
`TangentWorkspace` and dynamics `FaerLu` together with their existing controls. The helper takes
a scratch allowance, admitted `Par`, structural qualification context and observation prefix.
It preserves `square_response.faer.*`, `arclength.faer.*`, `diffsol.faer.*`, multi-RHS use and
caller-owned scratch lifetimes. It does not impose arclength's half-budget split on other
callers. Diffsol charges scratch against the factor allowance already reserved by its session,
not an arbitrary smaller new limit.

Classify numeric singularity as numerical, allocation as memory, index/extent overflow as
contract and unexpected failures as internal. `LuError::SymbolicSingular` after an authenticated
complete structural witness is a structural/contract breach; without that premise do not invent
it. Keep finite-solution and original backward-error checks. Delete duplicated mappers and the
repeated `n²·64 + 512n + 4096` allowance formula once consumers use the shared owner. Keep FERAL
inertia/condition estimation and KLU rcond/condest where their distinct questions belong.

For shared CBO-001, move KINSOL block preconditioner's FERAL Markowitz LU conservative bound
from `kinsol/block.rs::checked_extent` into the maintainer fork beside `SparseLu` allocating
code, using the existing fork LDLᵀ `conservative_storage_bytes` practice as a pattern. Carry
projections, CSR copies, lazy heaps, historical fill, temporary overlap and checked overflow in
the owner bound. This is local maintained-fork integration, independent of U1's external
contribution option, and is not upstream contact.
Keep local policy/admission at the caller; consume the bound before construction and delete
the replaced fork-internal mirror only after exact-source and targeted extent controls pass.

**Bounded investigations.** Before applying a cubic LU charge to whole-Jacobian
`jacobian_condition`, inspect actual diagnostic sizes and compare a tighter supplier-owned
Gilbert–Peierls or other complete bound; a blanket cubic charge could refuse supported work.
Before replacing opaque dense allowances, check structure-derived candidates on real relevant
Jacobians. KLU BTF bounds must include symbolic storage, off-diagonal `nzoff`, growth/reallocation
overlap and work arrays, not only sum(block_size²). George–Ng supplies a faer simplicial fill
bound, not complete supernodal padding or allocation-growth storage. Retain existing allowances
and strict-memory refusal unless the complete replacement is established. Report supported
cases, residual gaps and resulting choice to the single CBO-001 disposition owner; an unrun
real-Jacobian check remains unresolved, not a reason to shrink limits.

**Acceptance.** Target helper scratch-under/over-allowance cases before allocation, allocation/
index/singularity mapping, finite solve, same-pattern numeric refresh and changed-pattern refusal,
multi-RHS action lifetimes and stable observation names. Retain square-response shared-factor
and arclength action quality controls. C05 tests the final Diffsol outcome. Bounds controls
attack overflow and fill/growth shapes using source-derived independent extents; measurement
alone never proves an upper bound. Compile and targeted-test `pse-backend-native`; fork source
changes use that fork's assigned maintenance/verification route before pin integration.

<a id="c05"></a>

### C05 — Preserve terminal typed causes across Diffsol recovery

Source: [CBO-027](../library_leverage/reviews/native-solver-integration.md#cbo-027), composed with CBO-026.

Share a typed terminal-cause owner between the dynamics linear solvers and integration state,
following the existing callback terminal-failure latch. `set_linearisation` remains infallible
in Diffsol 0.16.2 / `diffsol-la` 0.1.1; preserve its deferred-result contract and library-owned
step shrinking. Faer/KLU allocation and contract failures latch their exact terminal class;
recoverable singular/nonfinite-current-matrix failures remain recoverable numerical trials.
At final mapping in `dynamics/integrator.rs::native`, prefer a latched terminal cause over
`TooManyNonlinearSolverFailures` without overwriting cancellation or another already owned
terminal failure. Scope/reset the latch with the attempt/session reuse boundary so a failed
previous attempt cannot contaminate a later successful one. Use one classification contract
with C04, not strings parsed out of `LinearSolverError::Other`.

Migrate forward, replay/backward and sensitivity paths that actually construct `FaerLu` or
`Klu`; tracing those constructors is necessary for completion. Correct the adjoint module
explanation: local checkpoint driving preserves population limits, cancellation, original-clock
step budgets and sample jumps, not merely access to a crate-private reset setter. Keep the
existing consistent restart after sample jumps that avoids upstream issue #419.

**Acceptance.** Inject deterministic allocator/extent failures at the solver seam and confirm
that the final integration outcome is memory/contract, while a singular trial can still recover
through ordinary step reduction. Exercise repeated factor refresh, clean reuse after failure,
forward and relevant adjoint paths, cancellation/deadline precedence and preserved partial
sample outcomes. Do not provoke host OOM as a test method. Delete replaced untyped terminal
mapping after the new cause reaches every consumer. An upstream fallible-linearization/abort
API can later retire the deferred idiom only after it preserves these controls.

<a id="c06"></a>

### C06 — One quiescent acquisition loop with adapter-owned policy

Source: [CBO-030](../library_leverage/reviews/native-solver-integration.md#cbo-030).

Build a crate-private acquisition helper beside `NativeWait` in `execution.rs`. Compose
`WakeSignal`/`WakeWatch`, registration-before-inspection, `pause_compute`, `try_resume_compute`
and `resume_compute`. Tentatively acquire the native gate, attempt immediate compute
reacquisition, then release/notify before any blocking capacity wait. Retain the original
deadline, cancellation/abandonment and retry ownership. Parameterize lock/health check, poison
mapping, checkpoint and nesting; names do not mandate a new general framework.

Migrate PETSc and SCIP, preserving PETSc initialization and SCIP terminal quarantine/TPI state.
Include IBEX only if its "poison means unavailable", algorithm checkpoint and
`catch_unwind`/compute restoration remain clear and testable; otherwise retain its separate
loop with an explicit coordinator disposition. Remove migrated loop copies in the same change.
Native construction/destruction remain exactly once on their owning thread.

**HiGHS inquiry.** Determine whether the existing read/exclusive gate's blocking compute
resume intentionally enforces Uno writer preference through `WAITERS`. Settling evidence is
owner intent plus a concrete wait/transition schedule showing safety and writer progress under
the chosen discipline. Either migrate with writer preference preserved or retain a documented
specific exception through the proper §18.8 clarification/decision route. No observed deadlock
is not proof that semantics are interchangeable. Do not silently expand the current §18.8 rule
to HiGHS as an accepted policy change.

**Acceptance.** Target notification races, tentative-gate rollback, compute shortage, cancellation,
abandonment, timeout, poison, SCIP quarantine and IBEX panic restoration. Verify that no migrated
path blocks for compute while owning the gate. Use controlled synchronization rather than timing
sleeps. The shared helper's tests do not qualify unrelated adapter initialization or shutdown.

<a id="c07"></a>

### C07 — Retain supplier choices and carry conditional trigger work explicitly

- **CBO-027:** retain Diffsol 0.16.2, local linear solvers and driven adjoint loop. Diffsol 0.18
  adds API churn/transpose requirements without removing panic/admission/refactor or checkpoint
  policy gaps. Reopen on fallible `set_linearisation`, a checkpoint observer/limit hook, #419
  correction or a needed numerical fix. Under U1, record those upstream options without contact.
- **CBO-028 Uno:** retain `805a27ec` plus `uno-pse.patch`, including tolerance forwarding,
  zero regularization, feasibility seed, dense operator materialization and integration hooks.
  At the next solver-image rebuild, observed bound-violating-trial defect, credible seed-obsolete
  evidence or released generic forwarding feature, perform the bounded QP-initialization check
  against the candidate release. v2.9.2's nonbasic placement fix is QP readback, so it does not
  establish that the initialization seed can disappear. Check `qp_iteration_limit`'s new ERROR
  route if rebasing. Delete the seed only on decisive initialization evidence plus original-space
  controls; no mandatory rebase is scheduled here. BQPD is a separate major composition decision.
  Generic HiGHS option forwarding (tolerances, `qp_regularization_value`, `threads`, `parallel`)
  is a conditional U1 option, with adoption only after release and preserved binding controls.
- **CBO-029:** retain `Adapter → Transport → foreign_nlp` as one successive chain. Preserve
  presolve tapes, structural retention, typed Rust errors and rich finalize data. No generated
  duplicate translator or direct POUNCE bypass is scheduled.
- **CBO-030 KINSOL cache:** retain strict LRU and freeing outside the thread-local `RefCell`
  borrow. `quick_cache` insertion-time eviction/CLOCK-PRO would change those semantics. No cache
  replacement is needed to complete C06.
- **CBO-031:** retain the pse cone hub, one solver spoke table each, original-coordinate
  certificate verification and retained Clarabel refinement. Generalized power cones in POUNCE
  remain a new supplier capability, not an inferred missing implementation obligation.
- **CBO-032:** the corrected premise is retired; sensitivity, backsolver, bound/path and
  parametric-QP library surfaces are already consumed. Do not recreate a package to "use more"
  of them. `refine_step_onto_bounds` fix-relax would admit active-set-changing predictions that
  the present validity contract refuses; reopening requires a functional decision, not leverage
  cleanup. The historical ID remains traceable through the review and coordinator.

## Execution checkpoint — 2026-10-27

C01 source authentication is integrated in `148969150`; C02's complete maintained faer/traits
source realization and native arithmetic premise follow in `74cc0636c`. The repeatable source
and optimized-IR probe traces the consumed primitive, matmul, dense decomposition and sparse
factor entry points, including dispatch callbacks and vtables. Targeted native arithmetic,
fitting-rank/PSD and sparse-action controls passed. This is no performance or cross-CPU result.
Diffsol-only KLU provider framing is being reconciled with the changed-provider campaign.

C03 has the official 7.9.0 native source projected into the existing 0.6.2 wrapper, with the
integration delta and version-derived storage/status/counter consumers rebased. The source
receipt distinguishes the published wrapper baseline from the selected native release. Initial
builds exposed stale CMake-cache contamination and changed public enum names; these were fixed,
and the changed-provider controls are running. Workaround retirement is conditional on their
actual corrected-history results.

C04's narrow faer helper now serves response, arclength and Diffsol while their factors, leases,
parallelism and scratch policies retain their existing owners. C05 retains first terminal typed
allocation/contract causes across Diffsol recovery with an attempt-scoped transport. Focused
forward/adjoint scratch-refusal, refresh and clean-reuse controls passed. The FERAL bound is a
tested local fork commit `a9b950aa`; publishing a reachable immutable supplier revision and
integrating its caller remain pending. Whole-Jacobian diagnostics retain their opaque allowance;
a smaller complete structure-derived replacement has not been established.

C06 shares acquisition for PETSc, SCIP and HiGHS/Uno. The controlled writer schedule confirms
that compute shortage releases the native gate while WAITERS retains writer preference.
IBEX keeps its existing loop because algorithm checkpoints, poison-as-unavailable and unwind
restoration have a different error contract; it already drops tentative exclusion before
capacity waits. Reopen that exception only when a shared interface preserves those semantics
without obscuring them. No native construction, destruction or quarantine policy was moved.
Targeted gate/HiGHS controls passed; actual provider and quarantine journeys remain for J.

C07 retains the reviewed Uno, Diffsol, cone hub, strict-LRU cache and successive adapter chain.
This execution does not trigger a solver-image rebuild, external contact or an unrelated rebase.
The selected SUNDIALS clause alone can retire on the new numerical evidence; the other supplier
triggers and distinct original-space acceptance obligations remain intact.

## Verification and evidence limits

**Interface-checked:** decisive repository build/dynamics/factor owners, exact faer/pulp and
Numerica sources, and the integrated review's native contracts were read. Context7 was consulted
first for faer and described explicit SIMD dispatch; it did not settle a complete FMA-free
configuration. Context7 also confirmed the 7.8 IDAS quadrature fix and 7.3 workspace deprecation from
[upstream release notes](https://github.com/llnl/sundials/blob/v7.9.0/CHANGELOG.md).
The G6 review/source accounts supply release facts and precise pinned-source
references; implementation must inspect the selected release/patch itself before adopting it.

**Proposed:** every package and remedy. **Not run:** native compilation, provider/build/image
workloads, probes, solver tests, cross-CPU tests and performance measurements during plan authoring.
57j owns the selected assembled solver/dynamics/fitting/uncertainty journeys and scope-end hygiene
and manual checks. Supplier identity, truthful error mapping, arithmetic compliance, scientific
acceptance and performance are separate claims; no one local pass establishes all five.
