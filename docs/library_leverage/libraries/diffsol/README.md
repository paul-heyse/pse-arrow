# Diffsol

Realizations considered: installed `diffsol` 0.16.2 with `diffsol-la` 0.1.1 and `diffsol-nl` 0.1.1
(vendored copies, numerical sources byte-identical to the crates.io archives; feature `faer`);
upgrade route `diffsol` 0.17.0 (released 2026-09-26), 0.17.1 (2026-09-28) and 0.18.0 with
`diffsol-la` 0.3.0 / `diffsol-nl` 0.3.0 (all released 2026-10-20); unreleased `main` at 991b4cf
(2026-10-24).
Research date 2026-10-27. Sources: Context7 `/martinjrobins/diffsol` (custom linear solver
query; it returned `main`'s faer `SparseLu` reuse design); installed `vendor/diffsol-la/src/linear_solver/{mod.rs,faer/lu.rs,faer/sparse_lu.rs,suitesparse/klu.rs}`
and `vendor/diffsol/src/ode_solver/{method.rs,checkpointing.rs}` at 0.16.2/0.1.1; upstream Git
tags `diffsol-v0.16.2`, `diffsol-la-v0.2.0`, `diffsol-la-v0.3.0`, `diffsol-v0.18.0` and `main`
(github.com/martinjrobins/diffsol, cloned to scratch); upstream issues #404, #419, #389, #223.

## Types

### diffsol/fallible-newton-linear-solver
- **Need it serves:** a Newton linear factorization inside an implicit ODE/DAE step whose failure
  (structural or numerical singularity, allocation failure, nonfinite solution) must shrink the
  step through the integrator's own recovery and finally surface as a typed error, not a panic.
- **Recognizable code shapes:** a caller-owned type implementing `diffsol_la::LinearSolver<M>`
  that stores a `Result` from `set_linearisation` and returns it from `solve_in_place`; wrappers
  around faer `NumericLu`/KLU objects created only to avoid `expect("Failed to factorise matrix")`.
- **Mechanisms and APIs:** `LinearSolver<M>` with `set_sparsity(&mut self, op)`,
  `set_linearisation(&mut self, op)` (returns `()`), `solve_in_place(&self, &mut V) -> Result<(), LaError>`;
  from `diffsol-la` 0.3.0 also a **required** `solve_transpose_in_place` (added by upstream PR #406).
  `LinearSolverError` variants (`LinearSolverNotSetup`, `LuSolveFailed`, `Other(String)`, ...) give
  the typed channel. The solver type is a generic parameter of the BDF/SDIRK/TR-BDF2/ESDIRK
  method constructors, so a caller-supplied solver is a supported extension point. A failed
  `solve_in_place` is counted by the Newton solver as a nonlinear failure and the step is
  reduced (behaviour of the step-size controller; see residual conditions).
- **Obligations discharged:** step-size reduction and retry policy, nonlinear-failure limit,
  final error propagation; the integrator owns when to refactor.
- **Residual caller obligations:** the factorization itself and its failure capture (deferred
  from `set_linearisation` to `solve_in_place`), finite-solution checks, storage reuse across
  refreshes, condition checks, scratch accounting and observation.
- **Semantic conditions:** the trait is infallible at `set_linearisation` at every inspected
  release, so a failure is observable only at the next `solve_in_place`; the deferred-error
  idiom depends on the Newton loop always calling `solve_in_place` after `set_linearisation`.
  The library's own implementations differ: the faer dense `LU` uses `full_piv_lu` (infallible,
  singularity shows as nonfinite values), while the faer sparse LU (`sparse_lu.rs`) and the KLU
  solver `expect(...)` on factorization failure (panic). At `diffsol-la` 0.1.1 neither library
  sparse solver reuses numeric storage across `set_linearisation` calls (fresh
  `Lu::try_new_with_symbolic` / `KluNumeric::try_from_raw` each time). **At 0.3.0 the faer sparse
  LU was redesigned:** the symbolic LU, one `NumericLu` per batch and `MemBuffer` factor/solve
  scratch are allocated once in `set_sparsity` and reused, with `factorize_numeric_lu` refreshing
  in place. It still `expect`s on failure, and its scratch is allocated with `MemBuffer::new`,
  with no external admission. The KLU solver at 0.3.0 still builds a fresh `KluNumeric` per
  linearisation (no `klu_refactor`). No library solver checks rcond or finite solutions.
- **Version / build conditions:** `set_linearisation` remains infallible and the sparse faer and
  KLU solvers still `expect` at `diffsol-la` 0.1.1, 0.2.0, 0.3.0 and `main` 991b4cf. Moving to
  0.3.0 adds the required `solve_transpose_in_place` method to every caller-supplied solver
  (faer: `solve_transpose_in_place_with_conj`; KLU: `klu_tsolve`). Unreleased `main` commit
  a3f7312 ("count nonlinear solver failures per step and lower the default limit to 10") changes
  how many deferred linear failures a step tolerates.
- **Limits and negative results:** no released or `main` version makes `set_linearisation`
  fallible; no upstream issue or PR proposing it was found (searched issue titles/bodies for
  "set_linearisation", "panic factor"; GitHub search only). A fallible trait would remove only the
  deferral idiom, not the need for a storage-reusing, condition-checking solver.
- **Evidence:** Interface-checked: `linear_solver/mod.rs` and implementations at the tags above;
  `git log diffsol-v0.16.2..diffsol-v0.18.0` for #406/#417. Error-to-step-reduction behaviour is
  taken from the library's Newton/step code comments and the consumer's documentation, not
  re-traced line by line.

### diffsol/checkpointed-adjoint
- **Need it serves:** reverse-mode (continuous adjoint) gradients over a forward ODE/DAE solve
  using stored checkpoints and Hermite interpolation of the forward trajectory.
- **Recognizable code shapes:** driven `step()` loops that clone solver state every k steps,
  collect Hermite points, then call `Checkpointing::new(...)` and `problem.adjoint_equations(...)`.
- **Mechanisms and APIs:** `OdeSolverMethod::solve_with_checkpointing(final_time, max_steps_between_checkpoints)`
  and `solve_dense_with_checkpointing(t_eval, ...)` return a `CheckpointingPath` split at reset
  events; public `Checkpointing::new(start_solver, start_idx, checkpoints, segment)`,
  `HermiteInterpolator::{new, reset, interpolate}`, `first_checkpoint`, `last_checkpoint`,
  `terminal_reset_root_idx()` (public getter); `AdjointEquations`, `AdjointOdeSolverMethod`,
  `solve_adjoint_backwards_pass`. From 0.18.0 a discretise-then-optimise route `step_vjp` exists on
  BDF/SDIRK/explicit RK (upstream #389); `interpolate_vjp` and accuracy work (#422, #425) are on
  `main` after 0.18.0.
- **Obligations discharged:** checkpoint replay, Hermite interpolation of the forward state,
  adjoint equation assembly, adjoint integration and the initial-value correction.
- **Residual caller obligations:** with the library's own `solve_with_checkpointing`: none of
  checkpoint-count limits, per-step cancellation, step budgets, progress events or sample
  emission are available (the loop runs to `final_time` inside the library). A caller needing
  any of these drives `step()` itself and passes its own checkpoints to `Checkpointing::new`,
  which is fully public.
- **Semantic conditions:** the reset-segment metadata setter (`set_terminal_reset_root_idx`) is
  `pub(crate)` at 0.16.2 and 0.18.0, but its getter is public; it matters only for problems with
  a reset operator (root-triggered state resets). Open upstream issue #419 (2026-10-22): after a
  cost jump, `IntegrateDeltaG` updates the differential adjoint but not the algebraic one, so the
  library's backward pass can restart a DAE adjoint from inconsistent values when the
  differential equations depend on algebraic states. A caller that re-initializes consistently
  after each jump avoids it. Open #404 proposes `max_number_timesteps` (a step cap); unreleased.
- **Version / build conditions:** API breaks between 0.16.2 and 0.18.0 include batched
  sensitivities/adjoints (#373, in 0.17.0) and `SmallVec`/`SmallMat` tableau refactors (#369).
- **Limits and negative results:** no released hook for checkpoint-population limits,
  cancellation or per-step observation in the library-driven checkpointing solve.
- **Evidence:** Interface-checked: `method.rs` `solve_with_checkpointing` at 0.16.2;
  `checkpointing.rs` visibility at 0.16.2 and `diffsol-v0.18.0`; issues #419, #404, #389 (bodies read).

## Current applicability

### [CBO-027](../../../library_leverage/codebase/native-solver-integration.md#cbo-027) — Diffsol Newton linear solvers and adjoint checkpoints re-implemented around upstream limits
- **Types:** diffsol/fallible-newton-linear-solver, diffsol/checkpointed-adjoint
- **Realization:** installed 0.16.2 / `diffsol-la` 0.1.1; upgrade route 0.18.0 / `diffsol-la` 0.3.0
  (moves `diffsol`, `diffsol-la`, `diffsol-nl` together and requires re-vendoring the three
  patched manifests; adds `solve_transpose_in_place` to the local solvers).
- **Conclusion:** ruled out here as a deletion route — no inspected release makes the trait
  fallible or exposes what the local adjoint loop needs. The current code already uses the
  library's supported extension points (caller-supplied `LinearSolver`, public `Checkpointing::new`).
- **What it would discharge / what remains local:** an upstream fallible `set_linearisation`
  would remove only the deferred-`Result` idiom at 0.16.2. At 0.3.0 the library's faer sparse LU
  already has the storage-reusing design (symbolic + `NumericLu` + `MemBuffer` reuse). With a
  fallible trait on top of that, the local faer solver would shrink to scratch admission,
  nonfinite-solution refusal and observation, which could be a thin wrapper around the library
  solver. The KLU solver would still need local `klu_l_refactor` + `klu_l_rcond`/`klu_l_condest`
  handling, because upstream KLU refactors nothing. The adjoint recording loop exists chiefly for the checkpoint-count limit
  checked before each clone, cancellation, the step budget, sample emission and progress events.
  It does not exist only because of the private reset setter: the consumer treats a root on the
  adjoint forward pass as an error, so reset segments do not arise.
- **Prerequisites, configuration or extension:** an upstream contribution would need either a
  fallible `set_linearisation` (breaking trait change) or a step-observer/limit hook in
  `solve_with_checkpointing`. #404 (step cap) would cover only the step budget.
- **Semantic gaps against the CBO's preservation constraints:** the library-driven backward
  pass (`solve_adjoint_backwards_pass`) carries open defect #419 for DAEs with `A_ad ≠ 0`. The
  local backward pass re-initializes through consistent initialization after each sample jump,
  which is the remedy #419 itself proposes. Moving onto the library driver would import that
  defect until it is fixed. Unreleased a3f7312 changes the nonlinear-failure limit that the
  "failures shrink the step" constraint relies on.
- **Unsettled premises and what would settle them:** whether upstream accepts a fallible trait
  (ask upstream; no issue exists); whether 0.18's `step_vjp` (discretise-then-optimise) is an
  acceptable alternative to the continuous adjoint for the gradient contract (a scientific
  choice, not an API fit).
- **Evidence:** Interface-checked as listed above; application Proposed.
