# Native solver integration

This page covers `pse-backend-native` and the `pse-*-sys` crates, the vendored suppliers under
`vendor/`, and the Uno solver image. Contracts are owned by
[§15–§18](../../authoritative_design/sections/numerical-execution.md#section-15). The baseline is
on the [entry page](README.md). Following the
[supplier notes](../../../vendor/native-supplier-notes.md), vendored accounting, checkpoint
budget and fixed-pattern hooks are classified as integration, not as core-solver forks.

**System picture.** A static adapter table maps each backend to one adapter. Each adapter
carries capability, settings, admission and warm payloads. Representation runners (NLP, roots,
coefficients and cones) drive the adapters.

Libraries are used deeply:
- **Presolve, derivative checks and KKT backsolve** come from POUNCE crates.
- **Tear selection** is a HiGHS MILP, with petgraph as an independent witness.
- **Recycle** is a KINSOL fixed-point map.
- **HiGHS** use includes IIS, ranging, Hessians, bases, callbacks and lexicographic
  objectives.
- **SCIP** use includes exact solving, reoptimization, IIS, indicator and SOS constraints, and
  concurrency.
- **FERAL** supplies certified inertia and Hager–Higham condition estimates.

The following are legitimately custom:
- outcome truthfulness: original-coordinate quality, ray verification and trial-failure
  classification;
- per-solver status tables;
- exception-shielded C++ bridges.

<a id="cbo-026"></a>

## CBO-026 — Sparse factorization wrappers split across faer, FERAL and KLU

- **Operation and consumers.** Factor a sparse square or KKT matrix, solve with it, and estimate
  its conditioning. Consumers are:
  - Jacobian conditioning and KKT inertia;
  - the arclength tangent and the event rank probe;
  - square-response actions and engineering accuracy;
  - the KINSOL block preconditioner;
  - Diffsol and IDAS Newton solves.
- **Current mechanism.** The same wrapper — symbolic once, numeric refresh, scratch-bounded,
  observed, error-mapped — is written per factor type:
  - **faer LU, twice.** `square_response/actions.rs` and `kkt/path/arclength.rs` are
    near-isomorphic.
  - **FERAL Markowitz LU.** `conditioning.rs`, and `kinsol/block.rs` with its own fill bound.
  - **FERAL LDLᵀ.** `kkt/activity.rs` and `kkt/path.rs`.
  - **KLU.** `dynamics/linear.rs`, `kinsol.rs` and `dynamics/idas.rs`.

  Two consequences follow:
  - The same Jacobian gets different estimators and failure semantics: FERAL's estimate for
    conditioning, and local `matrix_norms` over faer for accuracy bounds.
  - Pattern reuse, scratch admission, singularity classification and failure mapping are
    decided independently in each wrapper.
- **Why it matters.** Fewer independent decisions on those points, and one estimator per
  question.
- **Patterns.** LOP-09 (primary), LOP-07, LOP-03.
- **Footprint.**
  - Eight files inspected.
  - Every `*.factor` observation site is suspected.
- **Preservation constraints.**
  - Deterministic pivoting with no FMA contraction.
  - The allowance is charged before allocation.
  - Singular and fatal outcomes stay typed separately.
  - The normalized-matrix scaling convention.
  - Observation names.
- **Counterevidence.**
  - faer hides `NumericLu` storage and has no LU condition estimate.
  - FERAL alone certifies inertia.
  - KLU is required by the SUNDIALS fixed-pattern integration.
  - The remedy may be a crate-local factor trait rather than fewer libraries.
- **Status.** Proposed consolidation.
- **Research questions.**
  - Can FERAL LU serve the tangent and square-response uses, including its condition estimate?
  - Does a newer faer expose a condition estimate or storage size?
  - Could one `SparseFactor` abstraction carry symbolic reuse and scratch admission?
- **Library assessment correction (2026-10-27).** The square-response `matrix_norms` computes backward-error norms, not a condition estimate, so "different estimators for one Jacobian" is partly a category error. FERAL publicly exports `HagerHighamOperator` / `hager_higham_inverse_norm_1`, so one estimator could run over faer and KLU solves. See [feral](../libraries/feral/README.md).
- **Comparative review correction (2026-10-27).** There are three faer LU owners, not two: Diffsol `FaerLu` in `dynamics/linear.rs` is the third. The KLU `rcond`/`condest` in Diffsol is a refactor-acceptance policy (the bundled SUNDIALS 7.1.1 KLU setup policy), not a diagnostic estimator, so each question already has one estimator. The allowance formula `n²·64 + 512n + 4096` is duplicated verbatim in `square_response/actions.rs` (`SparseFactor::allowance`) and `kkt/path/arclength.rs` (`assemble`). Diffsol `FaerLu` maps every faer failure to `LinearSolverError::Other(String)`, which `integrator::native` reports as numerical, so allocation failures are misclassified; the raw `Klu` does the same with `KLU_OUT_OF_MEMORY`. See the [comparative review](../reviews/native-solver-integration.md#cbo-026).

<a id="cbo-027"></a>

## CBO-027 — Diffsol Newton linear solvers and adjoint checkpoints re-implemented around upstream limits

- **Operation and consumers.** Newton linear solves inside Diffsol's BDF, TR-BDF2 and ESDIRK
  steps, plus checkpointed adjoints, for dynamics and shooting.
- **Current mechanism.**
  - **Linear solvers.** `dynamics/linear.rs` replaces Diffsol's faer and KLU solvers.
    - Upstream `LinearSolver::set_linearisation` is infallible, and the vendored solvers
      `unwrap()` a failed factorization (`vendor/diffsol-la/src/linear_solver/faer/lu.rs`).
    - The adapter re-wraps faer LU and raw KLU so that a failure is returned from
      `solve_in_place`.
  - **Adjoint checkpoints.** `dynamics/integrator/adjoint.rs` records its own checkpoints,
    because the reset metadata between Diffsol's checkpoint segments is crate-private.
  - **Version.** The pin is `diffsol` 0.16.2, and its numerical sources are byte-identical to
    upstream.
- **Why it matters.**
  - A fallible upstream trait or public checkpoint metadata would delete about 800 lines of FFI
    and private-API replication.
  - A supplier change or an upstream contribution is the only way to get there.
- **Patterns.** LOP-11 (primary), LOP-10, LOP-03.
- **Footprint.**
  - `linear.rs` and `adjoint.rs` read.
  - `integrator.rs` suspected.
- **Preservation constraints.**
  - Failures shrink the step through Diffsol's own recovery, then surface typed.
  - Nonfinite solutions are refused.
  - KLU refactor storage is reused.
  - Work counters.
- **Counterevidence.**
  - Whether later Diffsol releases change the trait is unknown.
  - The KLU accounting hooks may remain necessary anyway.
- **Status.** Current mechanism. Revisit on a supplier move.
- **Research questions.**
  - Do Diffsol releases after 0.16.2 make `LinearSolver` fallible or expose checkpoint
    metadata?
  - Is an upstream issue or PR viable?
- **Library assessment correction (2026-10-27).** The cited `faer/lu.rs` is the dense LU and does not panic. The `expect` panics are in `faer/sparse_lu.rs` and `suitesparse/klu.rs`. Only the reset-index setter is crate-private: the getter, `Checkpointing::new` and the Hermite interpolator are public, and the local loop exists for checkpoint limits, cancellation, the step budget and samples. The expected deletion is therefore smaller than "about 800 lines". `set_linearisation` remains infallible through Diffsol 0.18.0. Open upstream defect #419 is avoided by the local per-jump re-initialization. See [diffsol](../libraries/diffsol/README.md).
- **Comparative review correction (2026-10-27).** The `adjoint.rs` module doc still gives the crate-private reset setter as the reason for local checkpoints. The real reasons are the limit, cancellation, budgets and samples; roots on the forward pass are refused. See the [comparative review](../reviews/native-solver-integration.md#cbo-027).

<a id="cbo-028"></a>

## CBO-028 — Vendored supplier numerics whose premises may have aged

- **Operation and consumers.** The pinned supplier numerics under every dynamics, QP and NLP
  solve.
- **Current mechanism.**
  - **SUNDIALS.** The bundled SUNDIALS 7.1.1 (through `sundials-sys` 0.6.2) carries a workaround
    for an IDAS quadrature-history defect, alongside the integration hooks
    (`vendor/native-supplier-notes.md` records the revisit trigger).
  - **Uno.** The revision `805a27ec` patch (`docker/solvers/uno-pse.patch`, 374 lines) makes
    four numerical changes (`docker/solvers/uno-binding.md`):
    - forwarded HiGHS tolerances;
    - zero QP regularization;
    - a feasibility seed copied around a QP-initialization truncation;
    - dense materialization of the L-BFGS operator for HiGHS.
- **Why it matters.**
  - The defect workaround and the QP seed patch correct upstream behaviour. They are not
    diagnostics.
  - A newer supplier may fix the behaviour or offer the hook natively.
  - Dense L-BFGS materialization has a scaling cost.
- **Patterns.** LOP-11 (primary), LOP-10.
- **Footprint.**
  - The supplier notes and binding notes were read.
  - The patch diffs were not.
- **Preservation constraints.**
  - Registered controls and their tests.
  - Determinism with `threads=1`.
  - Receipt hash identity.
- **Counterevidence.**
  - L-BFGS materialization reflects Uno's operator interface, not a defect.
  - Moving suppliers breaks bit-reproducibility.
- **Status.** Current mechanism with a recorded revisit trigger.
- **Research questions.**
  - Do SUNDIALS releases after 7.1.1 fix IDAS quadrature after `IDACalcIC`?
  - Do Uno commits after `805a27ec` change QP warm start or L-BFGS handling with HiGHS?
  - Does another Uno QP route avoid materialization?
- **Library assessment correction (2026-10-27).** The SUNDIALS revisit trigger has fired: the IDAS quadrature-initialization fix landed in 7.8.0 (commit `a7393b501`), but `sundials-sys` 0.6.2 is still the newest crate. Uno v2.9.2 (2026-10-21) changed the HiGHS QP path that the feasibility seed works around, and seven commits since the pin touch the patched files. See [sundials](../libraries/sundials/README.md) and [uno](../libraries/uno/README.md).
- **Comparative review correction (2026-10-27).** `kinsol.rs` already calls `KINSetDamping` and `KINSetDampingAA` with the same value, which avoids the KINSOL damping defect fixed in 7.3. Instances are reused (the session cache, `solve_with_restarts`), so a move to 7.3 or 7.4 would hit the Anderson-depth reinitialization regression fixed in 7.5; a move must reach at least 7.5. Native build identity does not frame the vendored SUNDIALS patch content: `BackendExecution::build()` returns `None` for the dynamics adapters, and the KINSOL fingerprint frames version macros and declaration text, so a patch-only change moves no identity. For Uno, v2.9.2's nonbasic bound placement acts on QP readback, while the feasibility seed addresses HiGHS 1.15.0 QP initialization, so the seed premise probably survives v2.9.2 (unconfirmed). See the [comparative review](../reviews/native-solver-integration.md#cbo-028).

<a id="cbo-029"></a>

## CBO-029 — Two NLP-oracle callback translators for three native NLP solvers

- **Operation and consumers.** Present the shared `NlpOracle` to a native NLP solver. The oracle
  covers objective, gradient, constraints, CSC Jacobian, lower Hessian, multiplier signs and the
  failure policy. Consumers are Ipopt, Uno and POUNCE.
- **Current mechanism.**
  - **POUNCE.** `tnlp.rs` (624 lines) is the POUNCE TNLP adapter, with its own pattern,
    scratch, callback state and normalization.
  - **Ipopt and Uno.** `uno/callback.rs` (841 lines) is the shared dispatch for Uno and the
    Ipopt sequence bridge, with its own context, scratch, `catch_unwind` and callback state.
  - **Shared core.** Both use `nlp_pattern.rs` and `callback::classify`.
  - **Third lifecycle.** `ipopt/sequence.rs` (about 1.7k lines) wraps a third lifecycle around
    the retained C++ sequence.
- **Why it matters.**
  - A new derivative or normalization rule must be applied twice.
  - POUNCE's TNLP already has the Ipopt TNLP shape, so one oracle-to-TNLP adapter could
    plausibly feed the other ABIs.
- **Patterns.** LOP-09, LOP-02.
- **Footprint.** Both translators were read at the level of headers and structs. The two were
  not diffed.
- **Preservation constraints.**
  - Panic containment at the C boundary.
  - Original-coordinate recovery.
  - Stale values are never substituted.
  - Typed failure causes.
- **Counterevidence.**
  - Index base and sign conventions differ between the ABIs.
  - The residual duplication may be thin.
- **Status.** Proposed.
- **Research questions.** Can one `pounce_nlp` TNLP implementation be exposed through Ipopt's
  C++ TNLP and the Uno C ABI?
- **Comparative review correction (2026-10-27).** The premise is rejected: there is one translation chain. `tnlp::Adapter` (normalization and presolve facts, shared by all three solvers) feeds pounce-presolve and then either `pipeline::Transport` (`NlpOracle`) into POUNCE or the `foreign_nlp` C-ABI dispatch into the Ipopt C++ sequence and Uno. Ipopt has only the retained C++ sequence route (`ipopt.rs` re-exports `sequence::Sequence` as `Session`), and Uno's conventions are already configuration. See the [comparative review](../reviews/native-solver-integration.md#cbo-029).

<a id="cbo-030"></a>

## CBO-030 — Native session lifecycles re-expressed per adapter

- **Operation and consumers.** Create, reuse, invalidate and free a native session on its owning
  worker, under cancellation, deadline, memory admission and observation. Consumers are all
  native routes.
- **Current mechanism.** `execution.rs` defines `ComputeAdmission`, `Retained`, `WarmPayload` and
  `WarmCapability`. Each solver then has its own state machine:
  - `highs_lifecycle.rs` handles the global scheduler, reset and invalidation.
  - `petsc.rs` handles process lifetime.
  - `implicit.rs` holds a per-thread KINSOL session cache with a byte budget.
  - `pounce/retained.rs` holds FERAL leases.
  - `ipopt/sequence.rs` and `scip.rs` have their own lifecycles.
  - `root_isolation.rs` has an IBEX process mutex and wake signal.

  Each re-implements:
  - acquire;
  - quiesce;
  - byte counting;
  - drop-on-unwind.

  Thread-based test suites repeat per adapter.
- **Why it matters.** The same obligations are expressed with different primitives:
  exclusion, releasing capacity before waiting on a native guard, and teardown on panic.
- **Patterns.** LOP-08 (primary), LOP-09.
- **Footprint.** The seam was read. The individual lifecycles were skimmed.
- **Preservation constraints.**
  - Exactly-once free, including on unwind.
  - Cancellation on the original clock.
  - Real solver-global state: IBEX random state, the HiGHS scheduler and PETSc initialization.
- **Counterevidence.** The global-state differences are genuine, so a common abstraction may
  cover only the RAII-plus-budget handle shape.
- **Status.** Proposed.
- **Research questions.** This is a local "native owner" guard design question. No library lead
  is known.
- **Comparative review correction (2026-10-27).** `NativeWait` is already shared; what remains triplicated is the acquisition loops of the PETSc, SCIP and IBEX gates. `execution::Retained` already owns adapter sessions. The HiGHS scheduler gate diverges: it calls blocking `resume_compute` while holding the gate, unlike the ADR-0176 rule stated for the other gates. See the [comparative review](../reviews/native-solver-integration.md#cbo-030).

<a id="cbo-031"></a>

## CBO-031 — Convex and conic encodings hand-synchronized per solver

- **Operation and consumers.** Solve LP, convex QP and cone programs, certify the results, and map
  reports back to coefficient rows. Consumers are the convex and conic routes.
- **Current mechanism.** Each solver receives its own layout of the same coefficient program:
  - **HiGHS** gets a row-bounds form (`highs.rs`).
  - **Clarabel** gets a cone form (`conic/lowering.rs`) and a recognized epigraph form
    (`conic/recognized.rs`).
  - **POUNCE-convex** gets a third layout (`execution/pounce_convex.rs`): an
    equality/inequality split, a norm-first power cone and a lower-triangle PSD form.

  Around those layouts:
  - Ray and multiplier raising are re-derived per layout (`certificate.rs`).
  - Clarabel iterative refinement is a bespoke working-control heuristic (`conic/refinement.rs`).
  - The generalized power cone has no POUNCE route.
- **Why it matters.**
  - Sign and permutation conventions per solver are a classic source of hand-synchronized forms.
  - A new cone or solver touches lowering, recovery and certificates.
- **Patterns.** LOP-02 (primary), LOP-09, LOP-12.
- **Footprint.** Module headers and one function each were read. The permutation tables were not
  compared.
- **Preservation constraints.**
  - Row order and sign identity, which feed content hashes.
  - Independent verification of certificates.
  - Determinism across batch lanes.
- **Counterevidence.**
  - The solvers really need different layouts.
  - The layer is already one module.
  - Redundant routes may be deliberate second opinions.
- **Status.** Current promise.
- **Research questions.**
  - Can POUNCE-convex and Clarabel share the standard conic form?
  - Do Clarabel or POUNCE expose refinement or equilibration options that would make
    `refinement.rs` unnecessary?
- **Comparative review correction (2026-10-27).** The premise is rejected. The pse cone form is the hub and maps one-to-one onto Clarabel's cone types (`Power` → `PowerConeT`; upper PSD triangle). POUNCE uses one forward `Target` map for both directions, and HiGHS rays are converted once and verified once. The Clarabel refinement already reuses the solver through `update_settings` (`conic.rs` `Session::solve_once`). See the [comparative review](../reviews/native-solver-integration.md#cbo-031).

<a id="cbo-032"></a>

## CBO-032 — KKT-point sensitivity built beside fuller library sensitivity surfaces

- **Operation and consumers.** Parametric sensitivity, the reduced Hessian and advanced-step
  prediction at a qualified candidate. Consumers are fitting covariance, uncertainty and
  continuation.
- **Current mechanism.**
  - `kkt.rs`, `kkt/sensitivity.rs` (784 lines), `kkt/advance.rs` and `kkt/path/*` rebuild an
    active-set KKT system and factor it with FERAL.
  - `pounce-sens-core` is used only as the backsolver contract.
  - `kkt/sensitivity.rs` re-implements the sIPOPT pin formulation locally.
- **Why it matters.** The design is deliberate and library-first, and largely a strength. The
  remaining question is whether more of the library surface would discharge local work:
  - `pounce-sens-core` and `pounce-qp` application types;
  - sIPOPT on the retained Ipopt TNLP for the same-solver route.
- **Patterns.** LOP-03, LOP-10.
- **Footprint.** Module headers only.
- **Preservation constraints.**
  - Validity gating in original coordinates: LICQ, strict complementarity and second-order
    sufficiency.
  - Independence from the solving backend.
- **Counterevidence.** Decoupling sensitivity from the solver is the stated intent.
- **Status.** Current promise. Low priority.
- **Research questions.** Which public `pounce-sens-core` and `pounce-qp` APIs does `kkt/*`
  reconstruct?
- **Library assessment correction (2026-10-27).** The premise that `pounce-sens-core` is used only as the backsolver contract is false. The code already uses `SensApplication` (parametric step and reduced Hessian), `IndexSchurData`, `step_along_path`, `rowlimit` and `pounce_qp::ParametricActiveSetSolver`, and the remaining library surfaces were ruled out for further discharge. This opportunity is a retirement candidate for the next audit refresh. See [pounce](../libraries/pounce/README.md).
- **Comparative review correction (2026-10-27).** The review confirmed that the premise is false: `SensApplication`, `IndexSchurData`, `step_along_path`, `rowlimit` and `ParametricActiveSetSolver` are in use, and `refine_step_onto_bounds` is unused by the advanced step's own contract. It recommends retiring this opportunity at the next audit refresh. Fix-relax for the advanced step would be a functional change, for a functional review if wanted. See the [comparative review](../reviews/native-solver-integration.md#cbo-032).

