# Clarabel

Realizations considered: installed `clarabel` 0.11.1 (default features off, `serde`). 0.11.1
(2025-06-11) is the latest release; upstream `main` has had only maintenance commits since then
(latest b6aa0421, 2026-04-13). There is no upgrade route.
Research date 2026-10-27. Sources: Context7 `/oxfordcontrol/clarabeldocs` (data-updating and
settings pages); installed `clarabel-0.11.1/src/solver/implementations/default/{settings.rs,data_updating.rs}`,
`src/solver/core/{solver.rs,cones/supportedcone.rs}`; GitHub releases, branches and commits of
oxfordcontrol/Clarabel.rs.

## Types

### clarabel/conic-standard-form
- **Need it serves:** solving LP/QP/SOCP/exp/power/generalized-power/PSD programs in one form.
- **Mechanisms and APIs:** `min ½xᵀPx + qᵀx  s.t.  Ax + s = b, s ∈ K`, with `P` upper triangle
  CSC and `K` an ordered list of `SupportedConeT::{ZeroConeT, NonnegativeConeT, SecondOrderConeT,
  ExponentialConeT, PowerConeT(α), GenPowerConeT(α, dim2), PSDTriangleConeT(n)}`. `PSDTriangleConeT`
  uses the upper triangle, column-wise, with off-diagonals scaled by √2. The power cone is
  "norm-last": `x₁^α x₂^{1−α} ≥ |x₃|`. Equalities are `ZeroConeT` rows inside the same `A`.
  Optional chordal decomposition (`chordal_decomposition_*`) and presolve (`presolve_enable`).
- **Obligations discharged:** IPM, cone barriers, infeasibility certificates in Clarabel's own
  coordinates.
- **Residual caller obligations:** lowering to this layout, and mapping rows, signs and cone
  orientations back to the caller's rows and certificates.
- **Semantic conditions:** cone orientation and triangle conventions differ from other conic
  solvers (see applicability).
- **Evidence:** Interface-checked: `supportedcone.rs`; Context7 docs.

### clarabel/accuracy-controls-and-resolve
- **Need it serves:** reaching a requested accuracy, and re-solving with tighter controls when an
  independent check fails.
- **Mechanisms and APIs:** settings `tol_gap_abs/rel`, `tol_feas`, `tol_infeas_*`, `tol_ktratio`,
  `reduced_tol_*` (AlmostSolved), `equilibrate_enable/max_iter/min_scaling/max_scaling` (Ruiz),
  `static_regularization_*`, `dynamic_regularization_*`, `iterative_refinement_enable/reltol/abstol/max_iter/stop_ratio`
  (refinement of each KKT linear solve, not of the final solution), `max_iter`, `time_limit`,
  `max_threads`. `DefaultSolver::update_settings(settings)`; data updates
  (`update_P/A/q/b`, `update_data`) that keep shape and sparsity, are refused when presolve or
  chordal decomposition is enabled (`is_data_update_allowed`), and cannot change the cone list;
  `set_termination_callback(f(&info) -> bool)` sees per-iteration info (residuals, gap), not the iterate.
- **Obligations discharged:** inner-loop numerical accuracy controls; reuse of the solver object
  across a re-solve with new settings (subject to the update rules).
- **Residual caller obligations:** any outer re-solve policy driven by a quality measure that
  Clarabel does not compute (for example original-coordinate residuals after the caller's own
  normalization).
- **Limits and negative results:** no warm start; no solution polishing or outer refinement; the
  termination callback cannot inspect the iterate, so it cannot apply an original-space test in
  the loop.
- **Evidence:** Interface-checked: `settings.rs` fields, `solver.rs` (`set_termination_callback`,
  `update_settings`), `data_updating.rs`; Context7 data-updating page.

## Current applicability

### [CBO-031](../../../library_leverage/codebase/native-solver-integration.md#cbo-031) — Convex and conic encodings hand-synchronized per solver (Clarabel part)
- **Types:** clarabel/conic-standard-form, clarabel/accuracy-controls-and-resolve
- **Realization:** installed 0.11.1; no upgrade route.
- **Conclusion:** ruled out here for removing `conic/refinement.rs`; fits with conditions only for
  narrowing what the heuristic owns. Clarabel's `iterative_refinement_*` and equilibration act on
  each KKT solve and on scaling, not on the original-coordinate quality ratio that triggers the
  local re-solve, so no option makes the heuristic unnecessary.
- **What it would discharge / what remains local:** the re-solve could use `update_settings` on a
  retained solver (when presolve and chordal decomposition are off) rather than rebuilding, if
  it does not already. The working-tolerance choice remains local policy.
- **Semantic gaps against the CBO's preservation constraints:** Clarabel's PSD triangle (upper,
  column-wise) and power-cone orientation (norm-last) differ from POUNCE-convex's lower-triangle
  `svec` and norm-first power cone (`pounce.md`), and Clarabel keeps equalities as `ZeroConeT`
  rows of one `A` where POUNCE-convex splits `A`/`G` and keeps bounds first-class. A shared
  "standard conic form" therefore still needs a per-solver permutation and sign table. That table
  is the irreducible local part, and it feeds row-order and sign identity in content hashes.
- **Unsettled premises and what would settle them:** whether the local re-solve already reuses the
  solver via `update_settings` — read `conic.rs` `Session` (not inspected in depth).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The unsettled premise is resolved: `conic.rs`
`Session::solve_once` already calls `update_settings` on the retained solver. The pse cone form is
the hub and maps one-to-one onto Clarabel's cone types (`Power` → `PowerConeT`; upper PSD
triangle); POUNCE uses one forward map for both directions. The comparative review rejects the
CBO's premise and retains the current encodings. See the
[comparative review](../../reviews/native-solver-integration.md#cbo-031).
