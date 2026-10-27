# Uno

Realizations considered: installed Uno at Git revision 805a27ec (2026-10-03; between releases
v2.9.0 and v2.9.1) with the 374-line local patch, built against the HiGHS archive from
`highs-sys` 1.15.0; upgrade routes Uno v2.9.1 (released 2026-10-13) and v2.9.2 (2026-10-21,
latest); unreleased `main` at d637515c (2026-10-26). HiGHS v1.15.1 (2026-07-02) is the latest
HiGHS release.
Research date 2026-10-27. Sources: github.com/cvanaret/Uno cloned to scratch:
`uno/ingredients/subproblem_solvers/HiGHS/{HiGHSSolver.cpp,HiGHSQuadraticProgram.cpp}` at
805a27ec, v2.9.2 and `main`; `interfaces/C/Uno_C_API.h` at 805a27ec; commit fabb3a8e8 (#1003);
the repository's binding notes and patch (read; they record an earlier v2.9.1 inspection dated
2026-10-21).

## Types

### uno/highs-qp-subproblem-controls
- **Need it serves:** solving SQP/SLP subproblems with HiGHS under caller-chosen accuracy,
  regularization, determinism, starting point and resource limits.
- **Recognizable code shapes:** a patch to `HiGHSSolver.cpp` that calls `setOptionValue` for
  tolerances, `qp_regularization_value`, `threads`, `parallel`, `qp_allow_hot_start`, or that
  passes a basis/solution into HiGHS before `run()`.
- **Mechanisms and APIs:** upstream `HiGHSSolver` sets only `output_flag=false` and (from v2.9.2)
  `qp_iteration_limit = 1000 + 10·(n+m)`. It reads `getSolution()`/`getBasis()` after the solve.
  From v2.9.2, nonbasic variables are placed on their active bound and projected into bounds
  ("HiGHS' QP solver may report a nonbasic variable at a bound with a value off that bound"), and
  Uno's own Lagrangian-Hessian regularization is applied in COO order before scattering to CSC.
  Uno does not forward its primal/dual tolerances to HiGHS, set HiGHS threads or parallel mode,
  disable HiGHS QP regularization, or seed a feasible basis — at 805a27ec, v2.9.2 or `main`.
  General options reach Uno through `uno_set_solver_*_option`; no inspected route forwards
  arbitrary HiGHS options.
- **Obligations discharged:** QP formulation, HiGHS model assembly, solution/multiplier readback,
  (v2.9.2) a QP iteration safeguard and nonbasic bound placement.
- **Residual caller obligations:** the four numerical policy changes (tolerance forwarding, zero
  QP regularization, feasibility seed, dense operator materialization) and every integration
  hook (scope callback, deadlines, memory latch, serial controls).
- **Semantic conditions:** v2.9.2's iteration limit makes a stalled QP return
  `SubproblemStatus::ERROR` — a new termination route that interacts with caller deadlines. Its
  nonbasic placement changes the primal step whenever HiGHS reports an off-bound nonbasic value.
- **Version / build conditions:** between 805a27ec and v2.9.2, commits fabb3a8e, 0f476092, 8aa9c731,
  779ef064, 25e7f29a, 875dfd7d and f8bff013 touch the HiGHS subproblem files or `NoRelaxation.cpp`,
  which are the patch's files, so a rebase is not mechanical.
- **Limits and negative results:** inspection covers the named HiGHS files and the C API header,
  not every globalization change.
- **Evidence:** Interface-checked: file contents at the three revisions; commit message and diff
  of fabb3a8e8.

### uno/hessian-representation-for-qp
- **Need it serves:** giving the QP engine a quasi-Newton (L-BFGS) Hessian.
- **Mechanisms and APIs:** `HiGHSQuadraticProgram::compute_hessian_sparsity` throws when the
  subproblem has no explicit Hessian matrix (`subproblem.has_hessian_matrix()`), at 805a27ec,
  v2.9.2 and `main`. BQPD accepts a Hessian operator. The C API offers
  `uno_set_lagrangian_hessian_operator` (Hessian-vector product) and
  `uno_set_jacobian_operator`/`..._transposed_operator` for matrix-free models.
- **Limits and negative results:** no HiGHS operator route exists upstream; any HiGHS use of
  L-BFGS needs materialization. This confirms the repository's 2026-10-21 v2.9.1 inspection
  through `main` d637515c.
- **Evidence:** Interface-checked: `HiGHSQuadraticProgram.cpp` at `main` (lines ~118–127).

### uno/c-api-conventions
- **Need it serves:** presenting one NLP oracle to Uno through C callbacks.
- **Mechanisms and APIs:** `UNO_ZERO_BASED_INDEXING`/`UNO_ONE_BASED_INDEXING`,
  `uno_set_lagrangian_sign_convention(UNO_MULTIPLIER_POSITIVE|NEGATIVE)` (default negative),
  `hessian_triangular_part` on `uno_set_lagrangian_hessian`, COO Jacobian/Hessian sparsity,
  `uno_set_initial_primal_iterate`/`dual_iterate`, notify/termination callbacks.
- **Obligations discharged:** index-base and sign-convention adaptation are configuration rather
  than translation code; Ipopt's conventions (positive sign, lower triangle) can be selected.
- **Residual caller obligations:** callback containment, finite checks, COO pattern export.
- **Evidence:** Interface-checked: `Uno_C_API.h` at 805a27ec.

## Current applicability

### [CBO-028](../../../library_leverage/codebase/native-solver-integration.md#cbo-028) — Vendored supplier numerics whose premises may have aged (Uno part)
- **Types:** uno/highs-qp-subproblem-controls, uno/hessian-representation-for-qp
- **Realization:** installed 805a27ec + patch; upgrade route v2.9.2 or `main`.
- **Conclusion:** unsettled — tolerance forwarding, zero QP regularization and dense
  materialization are still not upstream. v2.9.2 changes the very HiGHS path the feasibility seed
  works around, and it is unknown whether nonbasic bound placement removes the
  "small-activity truncation" that motivated the seed.
- **What it would discharge / what remains local:** a move to v2.9.2 would bring the
  `qp_iteration_limit` safeguard and the nonbasic placement for free. The tolerance, regularization
  and materialization changes and every integration hook stay local.
- **Prerequisites, configuration or extension:** rebase the patch across seven upstream commits
  touching its files; re-derive the feasibility-seed premise against v2.9.2's behaviour.
- **Semantic gaps against the CBO's preservation constraints:** registered controls would
  observe new statuses (QP iteration-limit `ERROR`) and changed steps. Determinism with
  `threads=1` is unaffected by the upstream change but remains local configuration. Receipt hash
  identity moves.
- **Unsettled premises and what would settle them:** whether the seed is still needed under
  v2.9.2 — probe request P1. Whether the truncation is a HiGHS QP-solver behaviour fixed in HiGHS
  1.15.1 was not investigated.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** On reading the patch, v2.9.2's nonbasic bound placement acts
on QP *readback*, while the feasibility seed addresses HiGHS 1.15.0 QP *initialization*, so the
seed premise probably survives v2.9.2 (unconfirmed). The comparative review retains the current
pin: the original-space acceptance already guards outcomes, and the rebase is non-mechanical and
needs an image rebuild and requalification. Revisit at the next image rebuild, on a probe result
or on an observed defect. Under upstream contribution, generic HiGHS subproblem-option forwarding
would turn three of the four numerical hunks into configuration. See the
[comparative review](../../reviews/native-solver-integration.md#cbo-028).

### [CBO-029](../../../library_leverage/codebase/native-solver-integration.md#cbo-029) — Two NLP-oracle callback translators (Uno part)
- **Types:** uno/c-api-conventions
- **Realization:** installed 805a27ec.
- **Conclusion:** fits with conditions as configuration only — Uno's C API can be configured to
  Ipopt's index base, sign convention and triangle, which is why one C-callback dispatch can
  already serve Uno and the Ipopt sequence bridge. It offers no adapter from a Rust TNLP.
- **What it would discharge / what remains local:** convention translation can be configuration;
  the oracle-to-callback translator itself remains local.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The CBO's "two translators" premise is rejected: there is one
chain (`tnlp::Adapter` → pounce-presolve → either the `NlpOracle` transport into POUNCE or the
`foreign_nlp` C-ABI dispatch into the Ipopt C++ sequence and Uno), and Uno's conventions are
already configuration. See the [comparative review](../../reviews/native-solver-integration.md#cbo-029).
