# POUNCE (sens-core, qp, convex, nlp TNLP, cinterface)

Realizations considered: installed POUNCE fork (github.com/paul-heyse/pounce) at revision
7f5e2e9d78555bd53f780c1af186c8fd22c922f5, crate versions 0.12.0 (`pounce-sens-core`, `pounce-qp`,
`pounce-rs` with `qp`/`convex`, `pounce-nlp`, `pounce-feral`, ...). `pounce-sensitivity` and
`pounce-cinterface` exist in that tree but are not workspace dependencies. Later fork revisions
were not assessed.
Research date 2026-10-27. Sources: installed checkout `~/.cargo/git/checkouts/pounce-*/7f5e2e9/crates/`
— `pounce-sens-core/src/{lib.rs,sens_app.rs,boundcheck.rs,activity_kernel.rs,rowlimit.rs}`,
`pounce-convex/src/{lib.rs,qp.rs,cones/{composite.rs,psd.rs}}`, `pounce-nlp/src/tnlp.rs`,
`pounce-cinterface/src/lib.rs`, `pounce-sensitivity/src/lib.rs` (headers).

## Types

### pounce/engine-agnostic-parametric-sensitivity
- **Need it serves:** sIPOPT-style parametric sensitivity, reduced Hessians and active-set path
  following from any converged KKT factor, independent of the solver that produced it.
- **Recognizable code shapes:** a KKT factor wrapped as `SensBacksolver`; `IndexSchurData` row
  selections with ±1 signs; `parametric_step`, `compute_reduced_hessian`.
- **Mechanisms and APIs:** trait `SensBacksolver { dim, solve(rhs, lhs) }` (plus `BoundRow`);
  `SensApplication::new(IndexSchurData, backsolver, SensOptions)` with `parametric_rhs`,
  `parametric_step`, `run_sens_step`, `compute_reduced_hessian`, `compute_reduced_hessian_eigen`;
  `IndexPCalculator`/`DenseGenSchurDriver`/`StdStepCalc` (the `P = K⁻¹A` / Schur stack);
  `boundcheck::{step_along_path, path_direction, refine_step_onto_bounds, bound_violations}`
  (fix-relax, breakpoint path following, directional derivative at a kink);
  `rowlimit::{RowLimitView, WatchedRow}` (limits written as constraint rows);
  `activity_kernel::{classify, classify_entry, off_path, contaminated}` (interior-point barrier
  activity rules over `mu`, slack and multipliers).
- **Obligations discharged:** Schur-complement machinery, reduced Hessian and its eigensystem,
  path following across active-set changes.
- **Residual caller obligations:** assembling the KKT system at the point, the pin formulation for
  parameters, validity gating (LICQ, strict complementarity, second-order sufficiency) and unit
  mapping.
- **Semantic conditions:** `activity_kernel` is defined on an IPM iterate (barrier parameter
  `mu`, slack/multiplier products). It does not apply directly to an active-set KKT at a
  qualified candidate. The NLP-specific corrector and activity plumbing live in
  `pounce-sensitivity` and are coupled to POUNCE's own filter-IPM `IpoptData`/`PdSensBacksolver`.
- **Evidence:** Interface-checked: `pounce-sens-core/src/lib.rs` module docs and exports;
  `boundcheck.rs` signatures; `pounce-sensitivity` crate doc.

### pounce/convex-qp-and-cone-form
- **Need it serves:** LP/QP/SOCP/exp/power/PSD solves with presolve, active-set or IPM engines.
- **Mechanisms and APIs:** `QpProblem { n, p_lower, c, a (eq), b, g (ineq, Gx ≤ h), h, lb, ub }`
  with first-class bounds; `ConeSpec::{Nonneg, SecondOrder, Exponential, Power(α), Psd(n)}`.
  `Power(α)` is "norm-first": `|x₁| ≤ x₂^α x₃^{1−α}`; `Psd(n)` uses `svec` over the lower
  triangle, column by column, with the √2 isometry. Entry points `solve_qp_ipm`,
  `solve_qp_active_set`, `ActiveSetSession`, `certify_psd_lower_triangle`; Ruiz equilibration,
  presolve/postsolve, HSDE, chordal handling, crossover.
- **Limits and negative results:** no generalized power cone (`ConeSpec` has no such variant).
- **Evidence:** Interface-checked: `qp.rs`, `cones/composite.rs`, `cones/psd.rs`.

### pounce/tnlp-and-c-abi
- **Need it serves:** presenting an NLP to POUNCE or to an Ipopt-compatible C ABI.
- **Mechanisms and APIs:** Rust `pounce_nlp::tnlp::TNLP` (Ipopt TNLP shape: `get_nlp_info`,
  `get_bounds_info`, `get_starting_point`, `eval_f`, `eval_grad_f`, `eval_g`, `eval_jac_g` with
  `SparsityRequest`, `eval_h`, `finalize_solution` with `IpoptData`/`IpoptCq`, optional metadata,
  scaling and linearity), plus `ExpressionProvider` for presolve tapes. `pounce-cinterface`
  implements Ipopt 3.14's C API (`CreateIpoptProblem`, `IpoptSolve`, intermediate callbacks) **on
  top of** POUNCE, i.e. C callbacks → POUNCE TNLP — the reverse of a TNLP → foreign-ABI bridge.
- **Limits and negative results:** no POUNCE crate adapts a Rust `TNLP` to Ipopt's C++ TNLP or to
  Uno's C API (searched the crate list and `pounce-cinterface` exports at the fork revision).
- **Evidence:** Interface-checked: `tnlp.rs` trait; `pounce-cinterface/src/lib.rs` module doc and
  exported functions.

## Current applicability

### [CBO-032](../../../library_leverage/codebase/native-solver-integration.md#cbo-032) — KKT-point sensitivity built beside fuller library sensitivity surfaces
- **Types:** pounce/engine-agnostic-parametric-sensitivity
- **Realization:** installed fork revision.
- **Conclusion:** ruled out here for further discharge. The local code already uses most of the
  engine-agnostic surface; the remaining library surfaces are engine-coupled or IPM-specific.
- **What it would discharge / what remains local:** already used: `SensApplication`
  (`parametric_step`, `compute_reduced_hessian`, `compute_reduced_hessian_eigen`),
  `IndexSchurData`, `SensOptions`, `SensBacksolver`/`BoundRow`, `boundcheck::step_along_path`
  (which calls `path_direction`), `rowlimit`, and `pounce_qp::ParametricActiveSetSolver`.
  The local pin formulation, KKT assembly and validity gating are domain policy.
  `activity_kernel` targets IPM iterates, not a qualified active-set candidate.
  `pounce-sensitivity` would couple sensitivity to POUNCE as the solving engine, contrary to the
  backend-independence constraint. sIPOPT on the retained Ipopt is not built into the native
  image (no `sipopt` in the solver build scripts), and it is same-solver-coupled too.
- **Semantic gaps against the CBO's preservation constraints:** both remaining routes violate
  "independence from the solving backend".
- **Unsettled premises and what would settle them:** whether `boundcheck::refine_step_onto_bounds`
  (fix-relax) is already reached through `step_along_path`, or would add value at the advance
  step — read `kkt/advance.rs` against `boundcheck.rs`.
- **Evidence:** Interface-checked; local imports read in `kkt/{sensitivity,advance,path,activity}.rs`
  and `kkt/path/qp.rs`; application Proposed.

### [CBO-031](../../../library_leverage/codebase/native-solver-integration.md#cbo-031) — Convex and conic encodings (POUNCE part)
- **Types:** pounce/convex-qp-and-cone-form
- **Conclusion:** ruled out here for a shared layout. The conventions genuinely differ from
  Clarabel (equality/inequality split, first-class bounds, norm-first power cone, lower-triangle
  `svec`, no generalized power cone), so a shared canonical form still needs per-solver tables.
  That is a local LOP-02 derivation question with no library mechanism.
- **Evidence:** Interface-checked; application Proposed.

### [CBO-029](../../../library_leverage/codebase/native-solver-integration.md#cbo-029) — Two NLP-oracle callback translators (POUNCE part)
- **Types:** pounce/tnlp-and-c-abi
- **Conclusion:** no library route. A theoretical inversion — feeding POUNCE through
  `pounce-cinterface`'s Ipopt C ABI so one C-callback translator serves all three solvers — would
  lose `ExpressionProvider` presolve tapes, typed Rust failure propagation and the rich
  `finalize_solution`/intermediate data. It would also add a C boundary inside a pure-Rust solver.
  Unifying "one TNLP implementation feeding the C ABIs" is local design: implement the oracle
  semantics once and generate both the Rust `TNLP` and the C callback set from it.
- **Evidence:** Interface-checked; application Proposed.
