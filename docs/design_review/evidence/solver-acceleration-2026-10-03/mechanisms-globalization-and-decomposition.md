# Mechanism contract cards: globalization and decomposition (M9–M17)

Evidence worker output for the solver-acceleration design review (2026-10-03). Bounded evidence,
no verdicts. Baseline: repository tree as read on 2026-10-03 (HEAD `f0b902589` plus the uncommitted
Plan 25k working tree); library corpus of the `native-solver-libraries` skill (SUNDIALS 7.1.1 via
`sundials-sys` 0.6.2, Diffsol 0.16.2, POUNCE 0.12.0) and the `pyomo-and-solvers` skill sources
(Pyomo 6.10.1, IDAES 2.13.0).

**Framing (from the maintainer).** All pertinent methods may be deployed. Derived or auxiliary
equation systems are acceptable when their provenance to the original equations is recorded and
their use is predicated on those same equations. Selection rests on mathematical facts known
before solving and on observations made during solving. No benchmarking is used for admission.
The final acceptance is always against the original problem in original coordinates.

**Evidence labels used in this file.** `[read]` = primary text or official documentation read
(full or relevant sections). `[restated]` = primary result read through a later paper by the same
or other authors that restates it verbatim. `[abstract]` = abstract, TLDR or bibliographic summary
only. `[known]` = standard result stated from domain knowledge, not re-read for this review.
`[interp]` = this worker's derivation or interpretation, not a claim made by the source.

---

## M9 — Natural-parameter and pseudo-arclength continuation

**1. Mathematical prerequisites.**
- `F: R^n × R → R^n`, C¹ along the path (C² for the step-length theory and curvature control).
- Start point `(x₀, λ₀)` is a regular solution.
- Near the path, the solution set is a smooth curve: `[F_x  F_λ]` has full row rank `n`.
- Natural-parameter continuation needs `F_x` nonsingular everywhere on the traversed segment, so no
  folds. Pseudo-arclength needs only the bordered (augmented) Jacobian to be nonsingular. That holds
  at regular points and at simple turning points (folds). It fails at branch (bifurcation) points,
  where `rank [F_x F_λ] < n` [read: LOCA manual §2.1.2; known: Keller].
- The target `λ₁` must lie on the connected component (branch) that contains the start. An isola
  holding the target cannot be reached.
- Every path point must lie inside the evaluator's domain of definition.

**2. Construction.**
- *Natural / first-order:* solve `F(x, λ_k) = 0` for a sequence of `λ_k`.
  - Zero-order predictor: `x_{k+1}^p = x_k`.
  - First-order (tangent) predictor: `F_x ẋ = −F_λ`, then `x^p = x_k + Δλ ẋ`. LOCA uses a
    forward-difference `F_λ ≈ [F(x, λ+δ) − F(x, λ)]/δ` when no analytic parameter derivative exists
    [read: LOCA eqs. 2.4–2.7].
- *Pseudo-arclength (Keller):* augment the system with
  `N(x, λ) = θ² ẋ_kᵀ(x − x_k) + λ̇_k(λ − λ_k) − Δs = 0`.
  - The Newton system is `[[F_x, F_λ], [θ² ẋ_kᵀ, λ̇_k]] [Δx; Δλ] = −[F; N]`.
  - The tangent `t = (ẋ, λ̇)` spans `ker [F_x F_λ]`, is normalized, and is oriented so that
    `t_kᵀ t_{k−1} > 0`.
  - The scale `θ` balances the solution and parameter contributions [read: LOCA eqs. 2.9–2.21].
- *Bordering (LOCA):* `J a = −F`, `J b = −∂F/∂λ`,
  `Δλ = −(N + ẋᵀ a)/(λ̇ + ẋᵀ b)`, `Δx = a + Δλ b`. These are two solves with one factor of `F_x`.
  The factor is singular exactly at a fold, which is why a sparse factorization of the full bordered
  matrix (sparse plus one dense row and column) or a deflated bordering is the robust choice at folds
  [read: LOCA eqs. 2.15–2.18; interp on the fold remedy].
- *PETSc `SNESNEWTONAL`* (load-parameter form `F(x, λ) = F_int(x) − F_ext(x, λ)`):
  - Split update `δx = δs·δx^F + δλ·δx^Q`, with `K δx^F = −F` and `K δx^Q = −∂F/∂λ`.
  - Constraint surface `‖Δx + δx‖² + ψ²Δλ² = L²`.
  - "Normal" correction: `δλ = −(Δx·δx^F)/(Δx·δx^Q + ψ²Δλ)`. "Exact" correction: root of a
    quadratic in `δλ`, with a partial correction when the roots are complex [read: PETSc SNES
    manual].
- *PITCON (Rheinboldt–Burkardt):* local parameterization. The continuation coordinate is chosen
  per step as the component with the largest tangent magnitude, and `e_iᵀx = x_i^pred` is appended.
  Curvature estimates control that choice and the step. The code also computes exact target points
  (a named variable reaching a specified value) and simple limit points [abstract].

**3. Retained state.**
- The last two or more converged points `(x_k, λ_k)` and the oriented tangent `t_k`.
- `Δs` and `θ`.
- The factor of `F_x` or of the bordered matrix, reused for the tangent solve and for chord or
  Broyden correctors.
- Corrector statistics: iteration count, contraction ratio, first-step length.
- Fold indicator: the sign of `λ̇`.
- Bifurcation test value: the sign of `det` of the augmented Jacobian.

**4. Step and globalization rule.**
- *LOCA adaptive rule.*
  - Failure halves the step.
  - Success sets `Δs_new = Δs_old (1 + a((N_max − N)/N_max)²)`, where `a ∈ [0, 1]` is the
    aggressiveness, `N` the corrector iterations used and `N_max` their limit. The result is
    clamped to `[Δ_min, Δ_max]`.
  - Tangent factor `τ = cos∠(t_k, t_{k−1})` scales the step as `Δs·τ^γ`. If `τ < τ_min`, the
    converged step is treated as a failure.
  - The final step is clamped so that it lands on the end value [read: LOCA §§2.1.1–2.1.3; Trilinos
    `LOCA::StepSize::Adaptive` documentation via search snippet].
- *Allgower–Georg asymptotic rule.*
  - Deceleration factor `f = max{√(κ/κ̃), √(δ/δ̃), α/α̃}`, clamped to `[½, 2]`; then `h ← h/f`.
  - `κ` is the contraction ratio of the first two corrector steps, `δ` the first corrector step
    length and `α` the angle between consecutive tangents. The tilde quantities are nominal values
    [restated via secondary sources; the book itself was not accessible].
- *Arc-length scaling:* rescale `θ` when the parameter contribution `θ²`-weighted drifts from its
  target. The sign of `dλ/ds` changes at a fold, so a fold is detectable from it [read: LOCA §2.1.3].

**5. Acceptance, intermediate meaning, relation to the original problem.**
- Each corrector must converge on the augmented system to a path tolerance. That tolerance is an
  intermediate one, distinct from the final requirement.
- When the path parameter is an *authored* physical parameter, the intermediate points are genuine
  solutions of neighbouring specifications `F(·, λ_k) = 0`. They are meaningful only when labelled
  with `λ_k` and are never results of the target specification.
- At the target, run a final correction with `λ` fixed: an ordinary solve of the original
  specification, then the original-space assessment.
- After a fold the target may be reached on a different branch (S-shaped curves, multiple steady
  states). Continuation yields *a* root. It does not yield "the root continuous in λ with the
  start" unless no fold was passed [interp from LOCA's turning-point discussion].

**6. Refusal and abandonment.**
- *Facts before solving.*
  - The parameter enters smoothly. No discrete or piecewise switch in `λ` lies on the range.
  - `F_λ` is available analytically or by finite difference.
  - Domain bounds are known.
  - Natural-parameter continuation is refused once a fold has been observed (see below).
- *In-solve observations.*
  - Corrector failure at `Δs_min`: abandon.
  - `λ̇` changes sign: a fold was passed. Natural-parameter continuation must switch to arclength or
    stop. Arclength continues, but it records that the path turned back.
  - `det(augmented)` changes sign without a `λ̇` sign change: a branch point was crossed, so branch
    switching is likely. Record it.
  - `λ` keeps moving away from the target after a fold: the target is not reachable on this branch.
    Abandon once the arclength budget is exhausted.
  - `‖x‖` grows without bound, or the tangent factor collapses.
  - Repeated evaluator domain errors: cut the step, and abandon if the errors persist at `Δs_min`.
- *Failure modes:* folds (natural-parameter only), bifurcations and branch jumping, isolas, paths
  leaving the domain, unbounded paths.

**7. Scaling, conditioning, sparsity.**
- The bordered Jacobian is the `F_x` pattern plus one dense column (`F_λ`) and one dense row (the
  tangent). Bordering keeps the original sparse factor.
- The arclength metric must use the same variable scaling as the original solve. Otherwise the
  parameter term dominates or vanishes, which is the reason for LOCA's `θ`.
- Near folds `F_x` is ill-conditioned, but the bordered matrix stays well-conditioned at simple
  folds.

**8. Cost versus a direct solve.**
- The cost is the number of steps times the corrector iterations, plus one extra back-solve per step
  for the tangent with the existing factor.
- Chord or Broyden correctors (Allgower–Georg ch. 7) reduce the number of factorizations.
- The gain is reachability and robustness, not fewer iterations on an easy problem.

**9. Interactions.**
- For optimization, the M9 tangent is the KKT parametric sensitivity: retained-factor prediction
  plus a corrector, i.e. parametric NLP path-following with active-set changes.
- M10 is M9 applied to a constructed `H(x, t)`.
- With BTF (M13), only the blocks downstream of the parameter's incidence change. The upstream
  blocks are solved once [interp].
- Inexact Newton or KINSOL setup-interval reuse in the corrector.
- Warm starts between steps.

**10. Library realization and minimum exposure.**
- None of Ipopt, POUNCE, KINSOL, IDAS, Diffsol, HiGHS, Clarabel or SCIP implements continuation
  (checked against the native skill's route matrix; a negative search, not a proof). The references
  are Trilinos LOCA, PETSc `SNESNEWTONAL`, PITCON and AUTO/MatCont/BifurcationKit; none of these is
  in the stack.
- Realization is a *composition of standard solves*:
  - natural-parameter: one KINSOL or Ipopt solve per step;
  - arclength: KINSOL on the `(n+1)` bordered system with a bordered residual and Jacobian;
  - predictor, orientation and step logic: bespoke and small.
- The pipeline must expose:
  - `F` and sparse `F_x`;
  - `∂F/∂λ` for any authored parameter, analytic from the symbolic model or by finite difference;
  - cheap re-evaluation at `(x, λ)` without recompiling;
  - the variable and row scaling vectors;
  - variable bounds and domain;
  - a reusable factor handle;
  - typed evaluation-error signals that distinguish a domain error from a numerical failure.

**11. References.**
- Allgower & Georg, *Numerical Continuation Methods* (Springer 1990; SIAM Classics 45, 2003),
  doi:10.1137/1.9780898719154 [known; the step-length formula restated via secondary sources].
- Allgower & Georg, "Continuation and path following", *Acta Numerica* 2 (1993),
  doi:10.1017/S0962492900002336 [abstract].
- Keller, "Global homotopies and Newton methods" (1978), in *Recent Advances in Numerical Analysis*
  [abstract/secondary].
- Rheinboldt & Burkardt, "A locally parameterized continuation process", ACM TOMS 9(2) 1983,
  doi:10.1145/357456.357460 [abstract].
- Salinger et al., *LOCA 1.0 Theory and Implementation Manual*, SAND2002-0396,
  https://www.osti.gov/biblio/800778 [read §2.1–2.2].
- PETSc SNES manual, `SNESNEWTONAL`: https://petsc.org/main/manual/snes/ [read].

---

## M10 — Constructed homotopies (Newton, fixed-point, affine, probability-one, bounded, model)

**1. Mathematical prerequisites.**
- `H: R^n × [0,1] → R^n` with an easy start root at `t = 0` and `H(·, 1) ≡ F`.
- *Probability-one theory* (Chow–Mallet-Paret–Yorke; Watson's sufficient conditions) [restated]:
  1. `ρ(a, t, x)` is C² and transversal to zero, i.e. its full Jacobian over `(a, t, x)` has full
     rank on its zero set.
  2. `ρ_a(0, x) = 0` has a unique solution.
  3. `ρ_a(1, x) = F(x)`.

  Then, for almost every `a`, a smooth zero curve leaves `(0, x₀)` with full-rank Jacobian along it.
  4. If, in addition, the zero set for `t ∈ [0, 1)` is *bounded*, the curve reaches `t = 1` at a
     root. If `F'` is nonsingular there, the arc length is finite.

  Condition 4 is typically the hard one to verify. For process models it is usually unverifiable a
  priori.
- The zero set can also contain closed loops, curves that return to `t = 0` and unbounded curves
  [restated: Watson 2002 classification].
- Smoothness matters: models with `min`, `max`, `abs` or piecewise property correlations void the C²
  premise, and the path may kink.
- *Global Newton homotopy (Smale/Keller):* on a bounded domain where Smale's boundary condition
  holds, the path `F(x) − λF(x₀) = 0` reaches a root. Singular `F'` on the path does not break the
  theory but forces small steps numerically [abstract/secondary].
- *Domain of definition:* Wayburn–Seader identify the path striking an interior boundary of the
  domain of definition of `F` (logs, square roots, mole fractions, EOS root selection) as a principal
  failure mode in process design [abstract/TLDR].

**2. Construction.** For each form: `H`, `H_x`, `H_t`.

| Form | `H(x, t)` | `H_x` | `H_t` | Notes |
|---|---|---|---|---|
| Newton (global) | `F(x) − (1−t)F(a)` | `F'(x)` | `F(a)` | Residuals shrink proportionally along `F(a)`. Turning points in `t` occur exactly where `F'` is singular [known]. |
| Fixed-point | `tF(x) + (1−t)A(x−a)` | `tF'(x) + (1−t)A` | `F(x) − A(x−a)` | Probability-one in `a` with `A = I` under boundedness [restated]. |
| Affine | `tF(x) + (1−t)F'(a)(x−a)` | `tF'(x) + (1−t)F'(a)` | `F(x) − F'(a)(x−a)` | Newton-like near `t = 0` [known]. |
| Model (Modelica `homotopy(actual, simplified)`) | `tF(x) + (1−t)F_s(x)` | `tF' + (1−t)F_s'` | `F − F_s` | `F_s` is a simplified model over the same unknowns. Dymola traces it with LOCA [read: Sielemann et al. 2011]. |
| Bounded (Paloschi) | Base homotopy plus a bounding term that confines the path to a prescribed domain | see notes | — | The 1995 form used a single penalty that made `H_x` *dense* once any variable entered the bounding zone. The 1997 sparse form keeps the original sparsity pattern; the 1998 paper used it in SPEEDUP to keep property evaluations valid. Malinen–Tanskanen (2008) track mapped variables to allow a narrow bounding zone [abstract/TLDR/secondary only; exact formulas not read]. |

*Sparsity-preserving fixed-point choice* [interp]. Take `A = D·P_M`, where `P_M` is the permutation
matrix of the structural perfect matching and `D` is a positive diagonal scaling in the solve's row
scaling. Then `pattern(A) ⊆ pattern(F')`, so `H_x` has the original pattern (cancellation aside). One
can also choose the signs in `D` to agree with the signs of the matched entries at `a`.

**3. Retained state.**
- The anchor `a`, plus the random seed if the probability-one property is claimed.
- The definition of `A` or `D` and `P_M`, or the identity of the simplified model, or the
  bounding-zone definition.
- The path points, tangents, orientation and arclength.
- The generation rule itself, which is the provenance.

**4. Step and globalization rule.**
- Arclength tracking as in M9 on `(x, t)`. `t` is not a valid parameter in general, because turning
  points in `t` are generic for the Newton homotopy and also occur in the other forms.
- HOMPACK provides three trackers:
  - ODE-based;
  - normal flow (Moore–Penrose quasi-Newton corrector);
  - augmented Jacobian (quasi-Newton).

  Each has dense and sparse variants and an "end game" near `t = 1` [abstract: Algorithm 652 and
  HOMPACK90].
- The Modelica example shows a sharp transition at `λ = 0.18`, which requires variable step sizes
  [read].

**5. Acceptance and intermediate meaning.**
- At `t = 1`, `H = F`. Run a final correction with an ordinary solver on `F`, then the
  original-space assessment.
- Intermediate points satisfy only `H(·, t) = 0`. They have no physical meaning:
  - For the Newton homotopy, `F(x(t)) = (1−t)F(a)` ("proportionally relaxed residuals").
  - For the model homotopy, they are solutions of a blended model.

  They are seed material only and never results.

**6. Refusal and abandonment.**
- *Facts before solving.*
  - Nonsmooth model parts: probability-one guarantees do not apply. The mechanism may still be used,
    labelled heuristic.
  - Unbounded variables without bounds: risk of paths to infinity.
  - Domain-restricted evaluators: require a bounded homotopy or domain-respecting step control.
  - Discrete or complementarity structure: not directly applicable.
  - A model homotopy requires a recorded derivation of `F_s` over the same variable identities.
- *In-solve observations.*
  - `t` turns back and heads to `0` (a curve returning to the start): abandon, or restart with a new
    anchor.
  - `‖x‖ → ∞` with `t` bounded (an unbounded curve): abandon.
  - An evaluator domain error at a path point: cut the step. If it persists, switch to a bounded
    homotopy or abandon.
  - A singular augmented Jacobian (nongeneric for random `a`): perturb `a`.
  - Arclength budget exceeded.
- *Process-specific failure modes.*
  - Wayburn–Seader: the path hits an interior domain boundary, goes to infinity, or returns
    [abstract plus the colleague's summary].
  - Modified bounded homotopies, per Malinen–Tanskanen's later study: start-point isolas (Newton),
    infeasible solutions (affine), and convergence only to certain roots (fixed-point)
    [abstract, 2011].
  - Modelica authors: the track diverges to ±∞, hits a bifurcation, or follows a regular path with
    turning points (the last is "not critical" with arclength) [read].

**7. Scaling, conditioning, sparsity.**
- The Newton and affine homotopies keep the `F'` pattern exactly.
- The fixed-point homotopy with `A = I` adds the diagonal. If `F'` has a structurally zero diagonal,
  that creates fill unless the system is first permuted by the matching, or `A = D·P_M` is used
  [interp].
- The 1995 bounded form densifies `H_x`; the sparse form does not [secondary].
- `A` (or `F_s`) must be scaled commensurately with `F` in the solve's row scaling. Otherwise one
  term dominates and the path has a sharp transition.

**8. Cost.** Long paths (arclength much greater than the distance between endpoints) take tens to
hundreds of corrector factorizations, against one Newton solve. The mechanism is justified when the
direct route has failed or the start is uninformed.

**9. Interactions.**
- Uses the M9 tracking machinery.
- The final correction is a native solve.
- *BTF:* the homotopy's incidence is the union of the incidences of `F` and of the auxiliary term.
  The Modelica paper notes that the iteration must cover everything from the first BLT block
  containing a homotopy operator through all dependent blocks. A homotopy can therefore be applied
  per irreducible block inside M13 [read plus interp].
- Bounded homotopy interacts with variable bounds and KINSOL sign constraints.
- Multistart (M17) varies the anchors to find further roots.
- M16: the model homotopy is the low-to-high-fidelity bridge.

**10. Library realization and minimum exposure.**
- HOMPACK/HOMPACK90 (Fortran, probability-one, sparse); Dymola uses LOCA for Modelica homotopy. None
  of these is in the stack.
- Realization is a bespoke tracker over KINSOL bordered corrector solves (composition).
- The pipeline must expose:
  - `F` and sparse `F'`;
  - the anchor `a` and its provenance (a seed proposal);
  - the matching permutation and scaling;
  - bounds;
  - typed domain-error signals;
  - for a model homotopy, a second evaluator `F_s` over identical variable identities.

**11. References.**
- Watson, Billups & Morgan, Algorithm 652 HOMPACK, ACM TOMS 13(3) 1987, doi:10.1145/29380.214343
  [abstract].
- Watson et al., Algorithm 777 HOMPACK90, ACM TOMS 23(4) 1997, doi:10.1145/279232.279235 [abstract].
- Chow, Mallet-Paret & Yorke, Math. Comp. 32 (1978), doi:10.1090/S0025-5718-1978-0492046-9
  [abstract].
- Watson, "Probability-one homotopies in computational science", JCAM 140 (2002),
  doi:10.1016/S0377-0427(01)00473-3 [restated via arXiv:1808.08052, Thms 1–2].
- Paloschi, CACE 21 (1997) 531–541, doi:10.1016/S0098-1354(96)00287-6 [TLDR].
- Paloschi, CACE 22 (1998), doi:10.1016/S0098-1354(98)00020-9 [TLDR].
- Paloschi, "Bounded homotopies to solve systems of algebraic nonlinear equations", CACE 19 (1995)
  [secondary].
- Malinen & Tanskanen, Chem. Eng. Sci. 63 (2008), doi:10.1016/j.ces.2008.04.006 [secondary].
- Wayburn & Seader, CACE 11(1) (1987) 7–25, doi:10.1016/0098-1354(87)80002-9 [TLDR].
- Jiménez-Islas et al., I&EC Res. 52 (2013), doi:10.1021/ie402418e [abstract].
- Sielemann, Casella, Otter et al., "Robust initialization of DAEs using homotopy", Modelica Conf.
  2011, doi:10.3384/ecp1106375, http://www.ep.liu.se/ecp/063/010/ecp11063010.pdf [read].
- Sielemann, Casella & Otter, "Robustness of declarative modeling languages: improvements via
  probability-one homotopy", SIMPAT 2013, doi:10.1016/j.simpat.2013.07.001 [title only].

---

## M11 — Pseudo-transient continuation (Ψtc)

**1. Mathematical prerequisites.**

*Kelley–Keyes 1998, Assumption 1.1* [restated verbatim in Kelley et al. 2008]:
1. `F` is Lipschitz continuously differentiable.
2. `F'(u*)` is nonsingular.
3. *Stability condition:* `u* = lim_{t→∞} u(t)` for `du/dt = −F(u)`, `u(0) = u₀`. This is a property
   of both `F` and `u₀`.
4. Near the trajectory, `‖(I + δF'(u))⁻¹‖ ≤ (1+βδ)⁻¹` for all `δ ≥ 0`, i.e. `F'` has no unstable
   eigenvalues near `u(t)` in this sign convention.

The result also requires `δ₀` and the inexact-Newton forcing terms to be sufficiently small.

*Extensions:*
- Kelley et al. 2008 weaken part 4. They allow semismooth `F` and a Lipschitz projection `P` onto a
  set `Ω` (convex box bounds give `M_P = 0`) [read].
- Coffey–Kelley–Keyes 2003 extend global convergence to *index-1 DAEs* [abstract].
- Deuflhard 2002 [read]:
  - The fixed point must be attractive. A hyperbolic fixed point gives "run away" except from a
    measure-zero set of starts.
  - Dynamical invariants, such as conservation `eᵀx = const`, make `F'` singular for every `x`.
    Newton fails, while Ψtc and fixed-point iterations implicitly keep `P⊥Δx = 0`.
  - The convergence analysis is affine-similar, with no nonsingularity assumption.

**What makes the artificial evolution well-posed for a general residual** [interp, grounded in the
sources above]:
- `F`, `−F`, `PF` (a row permutation) and `DF` have identical roots but *different* flows. A
  pseudo-time evolution is therefore not defined by the equations alone. It needs:
  1. a pairing of each equation with the variable it "evolves", which sets the pattern of the
     operator `V` (the shift `V/δ` must pair rows with columns);
  2. signs and scales for that pairing;
  3. evidence that the target is an *attracting* equilibrium of the resulting flow from `u₀`.
- Defensible constructions, in decreasing strength:
  - (a) *The authored dynamic form of the same model.* Physical accumulation (holdup) terms define
    `M`, and the flow is the physical dynamics. Pattison & Baldea build "statically equivalent"
    pseudo-transient DAE models of unit operations for exactly this purpose [abstract].
    - Even then, a physically *unstable* steady state (for example the middle steady state of an
      exothermic CSTR) is not reachable.
    - Closed inventory loops are dynamical invariants: the steady-state equations are singular, and
      the initial inventory, not the equations, selects the steady state.
  - (b) A *matching-paired* `V = D·P_M`, with signs taken from the matched Jacobian entries at `u₀`.
    This is only local evidence and carries no guarantee of attraction.
  - (c) The *gradient flow* of `½‖F‖²`, i.e. `u̇ = −F'ᵀF`. It always descends but can stop at
    stationary non-roots: Kelley et al. 2008 obtain only `∇f → 0` for gradient flows [read].

**2. Construction.**
- Iteration: `u₊ = u_c − (δ_c⁻¹V + F'(u_c))⁻¹F(u_c)`, with `V = I` in the classic form. This is one
  Newton step of implicit Euler for `V u̇ = −F(u)`, i.e. a Rosenbrock (linearly implicit Euler)
  step.
- Inexact form: `‖(δ⁻¹V + F')s + F‖ ≤ η‖F‖`.
- Projected form: `u₊ = P(u_c + s)` [read: Kelley et al. 2008 eqs. 1.4, 1.13, 2.3].
- PETSc `TSPSEUDO`: `G(Y) = F(Y, (Y−X)/dt)`, `[G']S = −F(X, 0)`, `X += S` [read].
- DAE form: `V` = mass matrix or `diag(I_d, 0)` (CKK 2003) [abstract].

**3. Retained state.**
- `δ` and the history of `‖F‖`.
- The definition of `V`: pairing, signs, scaling, mass operator.
- The projection.
- The previous step.
- The factorization. The shift changes every step, so the factor is reused only if `δ` is frozen or
  Krylov methods are used.

**4. Step and globalization rule.**
- *SER-A:* `δ₊ = min(δ_c‖F(u_c)‖/‖F(u₊)‖, δ_max)`, which equals `δ₀‖F(u₀)‖/‖F(u₊)‖` [read].
- PETSc variants: `dt_n = r·dt_{n−1}‖F_n‖/‖F_{n−1}‖` or `r·dt₀‖F_n‖/‖F₀‖`, plus `max_dt` [read].
- *SER-B:* step based on `δ_c/‖u₊ − u_c‖`.
- *TTE:* temporal truncation error target `3/4`.
- Common safeguard: reject steps that increase the residual and halve `δ`. The convergence theory
  covers SER-A [read: Kelley et al. 2008 §§1, 2.1.1, 4].
- *Deuflhard:*
  - optimal `τ = |[ν]|/…` from an estimate of the contractivity `ν` (affine-similar);
  - correction strategy if the residual is not monotone;
  - the method terminates when `[ν] ≥ 0`, meaning the steady state is not attractive in the
    residual sense [read].
- As `δ → ∞` the iteration becomes Newton.

**5. Acceptance and intermediate meaning.**
- `TSPSEUDO` stops at `‖F‖ < atol` or `‖F‖/‖F₀‖ < rtol` [read].
- In this pipeline the terminal iterate is a *seed* for the original correction, or the final
  Newton-limit steps *are* that correction. Acceptance comes only from the original-space
  assessment.
- Intermediate iterates are points of a deliberately inaccurate discretization of an artificial (or
  physical but holdup-modified) trajectory. They must never be presented as dynamic-simulation
  results.

**6. Refusal and abandonment.**
- *Facts before solving.*
  - Refuse automatic Ψtc when no flow justification exists from (a) or (b). The gradient-flow
    variant (c) may be offered, labelled as least-squares globalization.
  - Refuse when the algebraic part with singular `V` is not index-1, i.e. `∂F_a/∂u_a` is singular
    (CKK).
  - Refuse when the target is known to be dynamically unstable or a saddle.
- *In-solve observations.*
  - `δ` stagnates while `‖F‖` plateaus: the trajectory is near a slow manifold or a non-attracting
    equilibrium.
  - Repeated rejections collapse `δ`.
  - The residual increases once `δ` is large: loss of attraction near the target.
  - The Deuflhard estimate `[ν] ≥ 0`.
  - Gradient flow: `F'ᵀF ≈ 0` with `‖F‖` not small, i.e. a non-root stationary point. Report it as
    such.
  - Domain errors: project or cut the step.

**7. Scaling, conditioning, sparsity.**
- `pattern(δ⁻¹V + F') = pattern(F') ∪ pattern(V)`. With `V` diagonal on matched pairs (after the
  matching permutation) there is no fill beyond the diagonal.
- The shift regularizes early steps, in a Levenberg–Marquardt-like way.
- `V`, `δ₀` and the norms must use the solve's scaled coordinates.

**8. Cost.** Tens to hundreds of shifted linear solves, typically one factorization each, against a
handful for Newton. This is far cheaper than accurate time integration, which needs error control
and several Newton iterations per step.

**9. Interactions.**
- An alternative to M10 for basin enlargement.
- Uses the dynamic analysis mode of the same model (one model across modes).
- Needs M12's index analysis for the DAE form.
- Projection corresponds to bounds and KINSOL sign constraints.
- Anderson or NGMRES (M15) can accelerate it.
- The final Newton phase hands over to the native solver.
- Brune et al. classify Ψtc as a nonlinear *right* preconditioner: a precursor solve that puts Newton
  "within striking distance" [read].

**10. Library realization and minimum exposure.**
- PETSc `TSPSEUDO` is not in the stack.
- Compositions available in the stack:
  - (i) a bespoke loop over the library sparse factorization;
  - (ii) each step as a KINSOL solve of `G(u) = V(u − u_c)/δ + F(u)` limited to one iteration;
  - (iii) integration of the *authored* dynamic DAE to steady state with IDAS or Diffsol. This is a
    legitimate but costlier composition that relies on library error control.
- The pipeline must expose:
  - `F` and `F'`;
  - the mass or holdup operator `M` (or a constructed `V`) with its provenance;
  - the differential/algebraic identity vector;
  - bounds and the projection;
  - scaling;
  - typed domain errors.

**11. References.**
- Kelley & Keyes, SINUM 35 (1998) 508–523, doi:10.1137/S0036142996304796 [abstract; assumptions
  restated].
- Kelley, Liao, Qi, Chu, Reese & Winton, "Projected pseudo-transient continuation", SINUM 46(6)
  (2008), doi:10.1137/07069866X [read, open copy at mtchu.math.ncsu.edu].
- Coffey, Kelley & Keyes, SISC 25(2) (2003), doi:10.1137/S106482750241044X [abstract].
- Deuflhard, "Adaptive pseudo-transient continuation for nonlinear steady state problems", ZIB-Report
  02-14 (2002), https://opus4.kobv.de/opus4-zib/files/681/ZR-02-14.pdf [read §§1–3].
- PETSc `TSPSEUDO`: https://petsc.org/main/manualpages/TS/TSPSEUDO/ [read].
- Pattison & Baldea, AIChE J. 60 (2014) 4104–4123, doi:10.1002/aic.14567 [abstract].

---

## M12 — DAE consistent initialization and re-initialization

**1. Mathematical prerequisites.**
- *IDACalcIC, `IDA_YA_YDP_INIT`:*
  - The DAE is semi-explicit index-1 with a differential/algebraic identity vector.
  - Given `y_d`, `F(t₀, y, ẏ) = 0` determines `y_a` (and `ẏ_d`) uniquely. Locally this means
    `∂F/∂(ẏ_d, y_a)` is nonsingular.
- *`IDA_Y_INIT`:* given `ẏ` (typically `ẏ = 0` for quasi-steady states), `∂F/∂y` must be nonsingular
  [read: SUNDIALS IDAS docs].
- *Higher index:*
  - Pantelides' criterion and graph algorithm find which equation subsets must be differentiated,
    which yields hidden constraints on the initial values [abstract].
  - Dummy derivatives (Mattsson–Söderlind) then give a square, index ≤ 1 augmented system
    [abstract].
  - The structural index can understate the true index when numerical cancellation occurs. Pryce's
    Σ-method gives a numeric success test: structural analysis is valid where the system Jacobian
    built from the signature matrix is nonsingular [known].
- The number of free initial specifications must equal the dynamic degrees of freedom after index
  reduction. Specifying an algebraically determined state is an over-specification.

**2. Construction.**
- *IDAS (Brown–Hindmarsh–Petzold):* damped Newton with a line search on `F(t₀, y, ẏ) = 0` for the
  unknown components. It reuses the integrator's iteration matrix `∂F/∂y + c_j ∂F/∂ẏ` through
  "tricks involving the step size", and the result is retrieved with `IDAGetConsistentIC`
  [read/abstract].
- *Diffsol:* `StateRefMut::set_consistent` partitions the state by *zero diagonal of the mass
  matrix* and solves the algebraic equations for `(du, v)` [read: diffsol 0.16.2
  `src/ode_solver/state.rs`]. That partition assumes the algebraic variables show up as zero mass
  diagonal entries [interp].
- *Pantelides plus dummy derivatives:* build the differentiated augmented system and solve it as a
  square nonlinear system.
- *Kröner–Marquardt–Gilles:* a numerical method for higher-index DAEs with complexity reduced by
  structural information [TLDR]. Leimkuhler–Petzold–Gear approximate the derivative array with
  finite differences [abstract].
- *Gopal–Biegler SLP:* minimize the deviation from the specified values subject to linearized
  derivative-array equations, bounds and a trust region. It handles round-off and inconsistent user
  specifications, and it gives a criterion for which variables are continuous across a
  discontinuity at re-initialization [abstract/TLDR].

**3. Retained state.**
- The identity vector.
- The specification set and its values.
- The differentiation set, structural index and dummy-derivative selection, cached per mode and
  structure.
- The pre-event state.
- The integrator's Jacobian factor.

**4. Step rule.**
- IDACalcIC controls [read; vendored `idas.h` lines 179–188]:
  - `IDASetNonlinConvCoefIC` (convergence coefficient);
  - `IDASetMaxNumStepsIC`, `IDASetMaxNumJacsIC`, `IDASetMaxNumItersIC`, `IDASetMaxBacksIC` (limits);
  - `IDASetStepToleranceIC`;
  - `IDASetLineSearchOffIC`.
- `IDASetConstraints` is enforced: violations give `IDA_CONSTR_FAIL`.
- SLP uses an LP trust region (HiGHS).

**5. Acceptance and intermediate meaning.**
- The original residual must be satisfied to the integrator tolerance. For index > 1, the
  *differentiated hidden constraints* must also be satisfied: the original equations alone are not
  sufficient.
- An IC solve that alters the user's specified values (SLP minimal deviation) solves a *different*
  problem, a least-deviation projection. It must record which specified values moved and by how
  much. It is the original problem only when the deviation is zero [interp].

**6. Refusal and abandonment.**
- *Facts before solving.*
  - Structural index > 1 without index reduction: refuse IDACalcIC.
  - No identity vector: only `Y_INIT` is available.
  - Mass matrix that is not diagonal, or coupled: the Diffsol zero-diagonal partition is not
    trustworthy.
  - Specification count ≠ dynamic DOF: refuse as over- or under-specified (a structural check).
  - After an event, a mode change can change the index and DOF: redo the structural analysis.
- *In-solve observations:* `IDA_LINESEARCH_FAIL`, `IDA_CONV_FAIL`, `IDA_CONSTR_FAIL`,
  `IDA_FIRST_RES_FAIL` and `IDA_NO_RECOVERY` [read]. Escalation: with `y_d` fixed, the algebraic
  subsystem is a *square steady-state-like root problem*, so M13 block-sequential solving, M10
  homotopy or M9 continuation can produce a better `y_a` guess. SLP is the next step.

**7. Scaling, sparsity.**
- Uses the integrator's sparse structure.
- The algebraic subsystem is square and sparse.
- Near index changes `∂F/∂(ẏ_d, y_a)` is nearly singular.

**8. Cost.** Small relative to integration. Structural analysis is graph-linear in practice.

**9. Interactions.**
- `Y_INIT` with `ẏ = 0` is the steady-state root solve. The steady state of the dynamic model is the
  natural Ψtc target (M11).
- Sensitivity initial conditions via `IDAGetSensConsistentIC`.
- Events and re-initialization feed back into the structure caches.

**10. Library realization and minimum exposure.**
- IDAS: `IDACalcIC`, `IDACalcICB`, `IDASetId`, `IDASetConstraints`, `IDAGetConsistentIC`. Already
  bound in the repo: `crates/pse-backend-native/src/dynamics/idas.rs` maps
  `IdasInitialization::{AlgebraicAndRates, SteadyStates}` to `IDA_YA_YDP_INIT` / `IDA_Y_INIT`.
- Diffsol `set_consistent` and `set_consistent_augmented`, used in
  `crates/pse-backend-native/src/dynamics/integrator/adjoint.rs`.
- Pantelides and dummy derivatives: bespoke over the structural graph.
- SLP: a composition of HiGHS LP solves.
- The pipeline must expose:
  - `F(t, y, ẏ)`, `∂F/∂y` and `∂F/∂ẏ`;
  - the identity vector;
  - constraint signs;
  - the specification set;
  - mode identity at events.

**11. References.**
- Brown, Hindmarsh & Petzold, SISC 19(5) (1998), doi:10.1137/S1064827595289996 [abstract].
- Pantelides, SISSC 9(2) (1988), doi:10.1137/0909014 [abstract].
- Kröner, Marquardt & Gilles, CACE 16 (1992), doi:10.1016/S0098-1354(09)80015-X [TLDR].
- Unger, Kröner & Marquardt, CACE 19 (1995), doi:10.1016/0098-1354(94)00094-5 [TLDR].
- Gopal & Biegler, SISC 20(2) (1998), doi:10.1137/S1064827596307725 [abstract].
- Leimkuhler, Petzold & Gear, SINUM 28 (1991), doi:10.1137/0728011 [abstract].
- Mattsson & Söderlind, SISC 14 (1993), doi:10.1137/0914043 [abstract].
- SUNDIALS IDAS mathematics and usage docs, https://sundials.readthedocs.io/en/latest/idas/ [read].

---

## M13 — Structural decomposition as a solution method (matching, BTF, tearing, DM)

**1. Mathematical prerequisites.**
- A square system after the DOF specification, whose Jacobian *pattern* has a perfect matching, i.e.
  structural rank `n`.
- The block triangular form is the fine Dulmage–Mendelsohn decomposition of the well-determined part.
  It is unique up to the order of independent blocks [abstract: Pothen–Fan].
- Structural rank is *generic* rank, an upper bound on the numerical rank. Each diagonal block must be
  *numerically* nonsingular at every point visited [abstract: Parker et al. 2023, consistent with the
  repo's own caveat].
- The incidence must be exact and complete:
  - every input of an opaque or property call is declared;
  - piecewise expressions contribute the union of their branches' incidences.
- *Tearing:* the tear-variable iteration converges only if the tear map is a contraction (successive
  substitution) or the reduced tear Jacobian is nonsingular (Newton or Broyden on the tears) [known].

**2. Construction.**
- Maximum matching (MC21 / Hopcroft–Karp), then strongly connected components of the matched digraph
  (Tarjan / MC13), giving `P F' Q` block lower triangular [abstract: Duff–Reid Alg. 529;
  Pothen–Fan].
- DM coarse partition: under-, well- and over-determined parts.
- *Block-sequential exact solve:* for `k = 1..K`, solve `F_k(x_k; x_{<k}) = 0` with the predecessors
  fixed.
  - 1×1 blocks: explicit inversion or a scalar root (Pyomo uses `calculate_variable_from_constraint`
    for these).
  - Larger blocks: a square subsystem solve.
- *Tearing (Gundersen–Hertzberg):* choose tear variables in an irreducible block so the remaining
  equations become sequential; iterate on the torn equations' residual [abstract].

**3. Retained state.**
- The matching, block partition, topological order and tear sets. These are keyed by incidence
  identity and DOF specification, and are reusable across solves while those stay unchanged.
- Per-block factorizations, solver state and last solutions.

**4. Step rule.**
- Each block runs its own globalized Newton. Any of M9–M11 can be applied per block.
- Tear loops use successive substitution with Wegstein or Anderson acceleration, or Newton or Broyden
  on the tears.
- Block tolerances must be tighter than the full-system tolerance, because errors propagate through
  downstream blocks' sensitivities [interp].

**5. Acceptance and relation to the original.**
- No equations are derived; only an *ordering* is. When every block converges, the composed point is
  a solution of the original equations.
- It still requires the standard full-system residual check in original coordinates (composed
  tolerance) and the original-space assessment.
- A partially completed sweep satisfies a known prefix of row blocks. That is a legitimate
  intermediate fact ("rows R satisfied at x") [interp].

**6. Refusal and abandonment.**
- *Facts before solving.*
  - DM under- or over-determined parts are non-empty: refuse and report the DM sets as a diagnosis
    (Parker et al.).
  - Partial scope: refuse.
  - The incidence is uncertain (undeclared opaque inputs): refuse.
  - Optimization: inequality or objective coupling means the blocks are *not* independent problems.
    Only equality subsystems with the decisions fixed are square (repo caveat; Parker et al.).
- *In-solve observations.*
  - A block is numerically singular despite structural nonsingularity: a tiny pivot or a huge
    condition estimate. The structural witness was unqualified. Merge the block with neighbours, solve
    simultaneously, or use a per-block homotopy.
  - A block fails from the given predecessor values. The sequential scheme has no feedback, so
    upstream values may sit outside this block's feasible region (for example a negative upstream
    flow). Escalate to a simultaneous solve over the merged block set.
  - The tear iteration diverges (estimated spectral radius > 1): switch to Newton on the tears or
    to simultaneous solving.

**7. Scaling, sparsity.**
- BTF confines factorization to the diagonal blocks; off-diagonal blocks are used only in
  substitution.
- Per-block conditioning can differ from the global conditioning in either direction.
- Use per-block restrictions of the global scaling.

**8. Cost.**
- Matching is `O(τ√n)`; SCC is linear.
- Solve cost is the sum of the block costs, often far below a monolithic factorization.
- The sequential scheme loses global step coupling: there is no global line search.

**9. Interactions.**
- It is the substrate for M14 (implicit functions per block), M15 (GSN over BTF in topological order
  is *exact* in one sweep; NASM or GSN over irreducible partitions otherwise [interp]), per-block
  M9/M10, and the M12 algebraic subsystem.
- It underpins sensitivity block solves.
- KINSOL's KLU linear solver applies BTF internally at the linear level [known].

**10. Library realization and minimum exposure.**
- Graph libraries (petgraph, rustworkx-core) and the repository's library DM/BTF
  (`crates/pse-structural`). The repo's `initialization::Plan::from_analysis` refuses structurally
  deficient or partial scope [read].
- Reference implementations:
  - Pyomo `incidence_analysis.scc_solver.solve_strongly_connected_components` (1×1 blocks by
    `calculate_variable_from_constraint`);
  - IDAES `BlockTriangularizationInitializer`, whose `precheck` reports structural singularity via
    matching [read: pyomo-and-solvers corpus, Pyomo 6.10.1, IDAES 2.13.0];
  - HSL MC21/MC13.
- The pipeline must expose:
  - exact incidence, including the inputs of opaque calls;
  - per-block sub-evaluators (residual subset and Jacobian sub-block);
  - the DOF specification;
  - per-variable bounds;
  - a per-block numerical qualification: pivot growth, condition estimate, rank-revealing
    factorization of small blocks.

**11. References.**
- Pothen & Fan, ACM TOMS 16(4) (1990), doi:10.1145/98267.98287 [abstract].
- Duff & Reid, Algorithm 529, ACM TOMS 4(2) (1978), doi:10.1145/355780.355790 [abstract].
- Gundersen & Hertzberg, MIC 4(3) (1983), doi:10.4173/mic.1983.3.2 [abstract].
- Parker, Nicholson, Siirola & Biegler, CACE 178 (2023) 108383,
  doi:10.1016/j.compchemeng.2023.108383 [abstract].

---

## M14 — Reduced-space methods (implicit-function elimination; range/null-space; reduced Hessian)

**1. Mathematical prerequisites.**
- A partition `x = (y, z)` with equations `F₁` (`dim F₁ = dim y`) and `F₁,y` nonsingular *throughout
  the region the outer method visits*, including line-search trial points and restoration.
- By the implicit function theorem `y(z)` is then locally unique and `C^k` when `F` is `C^k`.
- Second-order outer methods need second derivatives of `y(z)` [abstract: Parker et al. 2022].
- *Range/null-space SQP:*
  - the constraint Jacobian `A` has full row rank;
  - the chosen basis `C` (the dependent columns) is nonsingular;
  - the reduced Hessian `ZᵀWZ` is positive definite at a strict local minimum (second-order
    sufficiency).

  The method is aimed at "few degrees of freedom" [abstract: Biegler–Nocedal–Schmid].
- Bounds on the eliminated `y` cannot be enforced directly. They become nonlinear constraints in `z`
  [known].

**2. Construction.**
- `G(z) = f(z, y(z))`, with `dy/dz = −F₁,y⁻¹F₁,z`.
- Reduced gradient by adjoint: `∇G = ∇_z f − F₁,zᵀ F₁,y⁻ᵀ ∇_y f`.
- Reduced Hessian via second-order adjoints.
- SQP step split: `p = Y p_Y + Z p_Z`, with `A Y p_Y = −c` and
  `(ZᵀWZ) p_Z = −Zᵀ(g + W Y p_Y)`. The coordinate basis `Z = [−C⁻¹N; I]` is exactly the
  implicit-function sensitivity [known/interp].
- BNS 1995 approximate the cross term `ZᵀWY p_Y` with a correction vector and use quasi-Newton
  updates on the reduced Hessian [abstract].
- *Nonlinear elimination* (Lanzkron–Rose–Wilkes 1996; Cai–Li 2011): eliminate only the equations and
  variables that cause slow Newton convergence. It reduces to an approximate Newton method near the
  root [abstract]. This is M14 applied locally, and it is identical to right nonlinear
  preconditioning (M15).

**3. Retained state.**
- The partition.
- The inner factor of `F₁,y`, reused for the derivative solves.
- The last inner solution, used as a warm start for the next outer trial.
- The quasi-Newton reduced Hessian.
- The recovered multipliers `λ₁ = −F₁,y⁻ᵀ ∇_y L`.

**4. Step rule.**
- The outer globalization (line search, trust region or filter) acts on the reduced problem.
- The inner Newton runs to a *tighter* tolerance than the outer one. An inexact implicit function
  makes the reduced function noisy and its derivatives inconsistent [interp].

**5. Acceptance.**
- Lift `(y(z*), z*)` and recover the full multipliers.
- Assess the full original system and its KKT conditions in original coordinates.
- Reduced KKT ⇔ full KKT only when the inner residual is zero.

**6. Refusal and abandonment.**
- *Facts before solving.*
  - No structurally nonsingular square selection for `y` (M13 matching).
  - Active or likely-active bounds on `y`.
  - Inequalities coupling into `F₁`.
  - A nonsmooth inner system.
  - Many degrees of freedom: dense `dy/dz` and a dense reduced Hessian.
- *In-solve observations.*
  - An inner failure at a trial `z` is reported to the outer solver as an *evaluation error*, so the
    outer step is cut (Ipopt supports evaluation errors in callbacks).
  - Repeated inner failures, or a growing inner condition estimate, in one region: abandon this
    elimination and go back to full space or another partition.
  - An indefinite reduced Hessian (inertia): use full space with inertia correction.

**7. Scaling, sparsity.**
- `dy/dz` is dense `n_y × n_z`, so the method fits small `n_z` only.
- Conditioning depends on the basis choice: a coordinate basis can be ill-conditioned even when `A`
  is well-conditioned [known].

**8. Cost.** Each outer evaluation costs an inner nonlinear solve. Derivatives cost `n_z` forward or
one adjoint back-solve with the inner factor. Parker et al. 2022 report better reliability than full
space on their case studies [abstract]. That is literature context, not an admission criterion here.

**9. Interactions.**
- M13 supplies candidate dependent sets.
- M15: nonlinear elimination is right preconditioning.
- KKT sensitivity and POUNCE's reduced Hessian are post-optimal.
- M17: the Schur complement is the linear-algebra analogue.
- M16: a surrogate can replace `y(z)` (black box).

**10. Library realization and minimum exposure.**
- Ipopt and POUNCE are full-space solvers.
- POUNCE `pounce-sens-core::reduced_hessian` computes the post-optimal `H_R = B K⁻¹ Bᵀ` from a
  converged KKT factor (a port of sIPOPT). It is not a reduced-space solver [read, corpus].
- Realization is a *composition*: an outer Ipopt, POUNCE or KINSOL whose callbacks perform the inner
  KINSOL or Newton block solves. The wiring is bespoke.
- Python reference: Pyomo `ExternalPyomoModel`
  (`dydx = −splu(jgy).solve(jgx)`), `ImplicitFunctionSolver` and `SccImplicitFunctionSolver`
  [read, corpus].
- The pipeline must expose:
  - partitioned sub-evaluators `F₁(y; z)`, `F₁,y`, `F₁,z`;
  - Hessian-of-Lagrangian blocks;
  - an evaluation-error channel into the outer solver;
  - inner warm-start state.

**11. References.**
- Biegler, Nocedal & Schmid, SIOPT 5(2) (1995), doi:10.1137/0805017 [abstract].
- Parker et al., "An implicit function formulation for optimization of discretized index-1 DAEs",
  CACE 168 (2022) 108042, doi:10.1016/j.compchemeng.2022.108042 [abstract].
- Lanzkron, Rose & Wilkes, SISC 17 (1996), doi:10.1137/S106482759325154X [abstract].
- Cai & Li, SISC 33(2) (2011), doi:10.1137/080736272 [abstract].

---

## M15 — Nonlinear preconditioning and solver composition

**1. Mathematical prerequisites.**
- *Left preconditioning* replaces `F(x) = 0` with `x − N(r, x) = 0`. The roots are preserved only if
  the fixed points of `N` are exactly the roots of `F`.
- For Schwarz methods, the subdomain problems `R_i F(P_i G_i(u) + (I − P_i R_i)u) = 0` must be
  solvable near the iterates, i.e. the local Jacobians `R_i J P_i` are nonsingular. The fixed point
  of the Schwarz iteration ⇔ `F(u) = 0` under existence and uniqueness of the subdomain solutions
  [read: Dolean et al. §3].
- Anderson or fixed-point acceleration needs a justified fixed-point map `G` with
  `fixed points ↔ roots` that is contractive near the solution.
- Picard (KINSOL `KIN_PICARD`) needs a splitting `F(x) = Lx − N(x)` with `L` nonsingular [known].
- Motivation: "unbalanced nonlinearities" make globalized inexact Newton stagnate at local minima of
  `‖F‖` [abstract: Cai–Keyes].

**2. Construction** (Brune et al., Table 3.1) [read]:

| Composition | Formula |
|---|---|
| Additive `M+N` | `x + α_M(M(r,x) − x) + α_N(N(r,x) − x)` |
| Multiplicative `M∗N` | `M(r, N(r, x))` |
| Left `M −L N` | outer `M` on the residual `x − N(r, x)` |
| Right `M −R N` | outer `M` on `r(N(r, x))` |

- For Newton, right preconditioning is approximately "N, then a Newton step". In Newton's case it is
  called *nonlinear elimination*.
- *ASPIN:* `F₁(u) = Σ P_i C_i(u)`, with the inexact Jacobian `−Σ P_i (R_i J(u) P_i)⁻¹ R_i J(u)`.
- *RASPEN:* `F̃(u) = Σ P̃_i G_i(u) − u`, with the *exact* Jacobian
  `−Σ P̃_i (R_i J(u⁽ⁱ⁾) P_i)⁻¹ R_i J(u⁽ⁱ⁾)`. It reuses the local factors already built by the
  subdomain Newton solves [read: Dolean et al. eqs. 3.16–3.18].
- *GSN (Gauss–Seidel–Newton):* block Newton applied multiplicatively over the blocks.
- *NASM:* additive block solves. RAS is the non-overlapping-injection variant.
- *NGMRES / Anderson:* combine the last `m` iterates into a residual-minimizing iterate.
- *FAS:* coarse-model correction with `b_H = R[b − F(x_s)] + F_H(R̂x_s)`, which guarantees a fixed
  point at the exact solution [read].

**3. Retained state.**
- Block partition, restrictions `R_i` and injections `P_i`.
- Local factors.
- The history window (`m` iterates and residuals) for Anderson or NGMRES.
- Damping and delay.

**4. Step rule.**
- For left preconditioning, the line search on `x − N(r, x)` is the "correct" one, but each evaluation
  costs inner solves. A line search can also miss stagnation of a weak inner solver, so that must be
  monitored [read: Brune et al. §3].
- Anderson: damping (`KINSetDampingAA`), window (`KINSetMAA`) and restarts.

**5. Acceptance.**
- The preconditioned system has the same roots, but small preconditioned residuals do not imply
  small original residuals when `N` is badly conditioned.
- Convergence tests and the final assessment use the *original* `F` [interp, consistent with Brune et
  al.'s line-search remark].

**6. Refusal and abandonment.**
- *Facts before solving.*
  - No numerically qualified partition. Structural blocks alone are insufficient, as in M13.
  - Subdomain solvability unknown.
  - Bounds or inequalities inside blocks.
  - Fixed-point map not justified (KINSOL `KIN_FP` must not reinterpret an arbitrary residual).
- *In-solve observations.*
  - Subdomain solve failures.
  - Outer stagnation: the preconditioned residual falls but the original residual does not.
  - The Anderson least-squares problem becomes ill-conditioned: restart.
  - NGMRES stagnation.

**7. Scaling, sparsity.**
- Preconditioned Jacobians are dense and are applied matrix-free (Krylov), except for RASPEN/ASPIN
  from local factors.
- Preconditioning balances nonlinearity rather than linear conditioning.

**8. Cost.** Each outer iteration costs subdomain nonlinear solves plus an outer Krylov solve. It pays
off when local nonlinearities dominate the convergence behaviour. More work per iteration, fewer
iterations.

**9. Interactions.**
- M13 blocks are the natural subdomains.
- M14 nonlinear elimination is right preconditioning.
- Ψtc and grid sequencing are right preconditioners [read].
- Anderson can accelerate tear loops.
- Inexact Newton (Krylov) is used for the matrix-free outer solve.
- FAS is in-solver multifidelity (M16).

**10. Library realization and minimum exposure.**
- PETSc SNES has all of these (`SNESNGMRES`, `SNESANDERSON`, `SNESNASM`, `SNESASPIN`,
  `SNESCOMPOSITE`, `SNESFAS`, `SNESNRICHARDSON`, `SNESNCG`, with the inner solver set through
  `SNESGetNPC`) [read]. PETSc is not in the stack.
- KINSOL provides `KIN_FP` with Anderson (`KINSetMAA`, `KINSetDampingAA`), `KIN_PICARD` and
  `KIN_LINESEARCH` [read: vendored `kinsol.h`].
- Everything else is a bespoke composition over KINSOL block solves.
- The pipeline must expose:
  - block residual and Jacobian sub-evaluators;
  - the restriction and injection maps;
  - fixed-point map declarations with their justification;
  - Jacobian-vector products.

**11. References.**
- Brune, Knepley, Smith & Tu, SIAM Rev. 57(4) (2015), doi:10.1137/130936725, arXiv:1607.04254
  [read §§3–5].
- Cai & Keyes, SISC 24(1) (2002), doi:10.1137/S106482750037620X [abstract].
- Dolean, Gander, Kheriji, Kwok & Masson, SISC 38(6) (2016), doi:10.1137/15M102887X,
  arXiv:1605.04419 [read §3].
- Walker & Ni, SINUM 49(4) (2011), doi:10.1137/10078356X [abstract].
- Washio & Oosterlee, SISC 21(5) (2000), doi:10.1137/S1064827598338093 [abstract].

---

## M16 — Multifidelity / surrogate model management and reduced-order models

**1. Mathematical prerequisites.**
- The high-fidelity model `f` stays the accuracy authority [abstract: Peherstorfer et al.].
- The low-fidelity model `a` maps the same inputs to the same outputs, or comes with a recorded map.
- *Trust-region model management* (Alexandrov et al.): provably convergent to a solution of the
  original high-fidelity problem [TLDR]. Its standard premise is *first-order consistency* at each
  trust-region centre: `a(x_c) = f(x_c)` and `∇a(x_c) = ∇f(x_c)`. This is achieved by additive or
  multiplicative correction and requires high-fidelity gradients [known].
- Eason–Biegler's trust-region filter for glass-box/black-box problems guarantees first-order
  criticality of the *original* hybrid problem when a reduced model replaces a black box [abstract].
  The κ-fully-linear model condition is [known].
- "Global convergence" in this literature means convergence to stationary points, not global optima.
- *ROM* (Benner et al.): a projection basis valid over the parameter range. Nonlinear models need
  hyper-reduction (for example DEIM) for a real cost reduction [abstract plus known].

**2. Construction.**
- Trust-region subproblem: `min a_k(x)` subject to `‖x − x_k‖ ≤ Δ_k` (plus constraints).
- Ratio test `ρ = (f(x_k) − f(x_k+s)) / (a_k(x_k) − a_k(x_k+s))`. Accept when `ρ > η₁`, then update
  `Δ`.
- Eason–Biegler: filter on `(θ = ‖y − d(w)‖, objective)`, plus compatibility and criticality steps.
- Peherstorfer taxonomy: adaptation, fusion, filtering.
- ROM: `x ≈ V x_r`, Galerkin `Vᵀ F(V x_r) = 0`.
- For *root problems* (simulation) there is no objective to manage. A low-fidelity solution is a
  *seed*, or the start of a model homotopy (M10) [interp].

**3. Retained state.**
- Surrogate parameters and the sample set.
- Trust-region centre and radius.
- Correction terms.
- The history of high-fidelity evaluations and gradients.
- The ROM basis.

**4. Step rule.** Ratio-driven radius updates. Filter acceptance. A sample refresh when the model is
inadequate.

**5. Acceptance.**
- Only high-fidelity evaluations are accepted.
- Surrogate optima are proposals.
- The final point is assessed on the original model in original coordinates.

**6. Refusal and abandonment.**
- *Facts before solving.*
  - No low-fidelity model with recorded derivation over the same coordinates.
  - No high-fidelity gradients and no fully-linear guarantee.
  - No outer loop of repeated evaluations: a single small solve does not repay construction. This is
    a fact about the study shape, not a measurement.
- *In-solve observations.*
  - `ρ ≤ 0` repeatedly, so the radius collapses: abandon in favour of direct high-fidelity solving.
  - The criticality measure stagnates.
  - Proposals land outside the sample hull (extrapolation).
  - A large callback-cost share of high-fidelity evaluations. This is an in-solve observation the
    callback metrics already record; it may *trigger* the mechanism, not benchmark it.

**7. Scaling.** Build surrogates in scaled variables. Watch the conditioning of the ROM basis.

**8. Cost.** Sampling and construction, plus at least one high-fidelity evaluation (with gradient) per
trust-region iteration.

**9. Interactions.**
- The model homotopy (M10) bridges fidelities.
- Seed producer for M9 and the original correction.
- FAS (M15) is the in-solver analogue.
- Surrogates of implicit blocks (M14).
- Cheap screening of multistart candidates (M17).

**10. Library realization and minimum exposure.**
- Pyomo `contrib.trustregion` implements Eason–Biegler (Python reference; not checked in the corpus
  for this card).
- For this stack: a composition of Ipopt solves with bespoke trust-region and filter management. The
  ROM would be bespoke, for example a faer SVD for POD.
- The pipeline must expose:
  - a high-fidelity evaluator with gradients;
  - a low-fidelity evaluator over identical variable identities;
  - the low-fidelity provenance;
  - a sample store.

**11. References.**
- Alexandrov, Dennis, Lewis & Torczon, Struct. Optim. 15 (1998), doi:10.1007/BF01197433 [TLDR].
- Peherstorfer, Willcox & Gunzburger, SIAM Rev. 60(3) (2018), doi:10.1137/16M1082469 [abstract].
- Benner, Gugercin & Willcox, SIAM Rev. 57(4) (2015), doi:10.1137/130932715 [abstract].
- Eason & Biegler, AIChE J. 62 (2016), doi:10.1002/aic.15325 [abstract].

---

## M17 — Multi-scenario / block-structured NLP decomposition, and multistart as study composition

**1. Mathematical prerequisites.**
- A bordered block-diagonal (arrowhead) KKT structure: scenario blocks coupled only through linking
  (first-stage) variables or coupling constraints.
- *Explicit Schur complement:*
  - each scenario block `K_i` is nonsingular after the IPM regularization;
  - `S = K₀ − Σ B_iᵀ K_i⁻¹ B_i` is nonsingular;
  - the inertia is additive, `In(K) = Σ In(K_i) + In(S)` (Haynsworth), which is required for
    IPM inertia correction [known];
  - the linking dimension is small.

  A large linking dimension calls for the *implicit* Schur method: PCG with a quasi-Newton
  preconditioner [TLDR: Kang et al. 2014].
- *Multistart:* independent local solves from distinct starts. Sampling needs finite bounds. There is
  no global guarantee: scatter search with a local NLP solver is a *heuristic* [abstract: Ugray et
  al.].

**2. Construction.**
- Per IPM iteration:
  - factor each `K_i` (in parallel);
  - form `S` from `K_i⁻¹ B_i` solves;
  - factor `S`;
  - back-substitute.
- Implementations: Zavala–Laird–Biegler (parallel Schur decomposition inside an Ipopt-based
  framework) [TLDR]; PIPS-NLP; Parapint (PyNumero + MPI) [abstract].
- *Multistart:* generate starts (scatter search, Latin hypercube, clustering such as MLSL), run local
  solves, cluster distinct solutions, select the best *accepted* one.

**3. Retained state.**
- Per-block factors and inertia, the Schur matrix, scenario and linking maps.
- Multistart: the solution set and cluster representatives.

**4. Step rule.**
- The IPM step rule is unchanged. An exact Schur decomposition is a *linear-algebra realization* that
  gives the monolithic step up to rounding, pivoting and regularization differences [interp].
- Multistart needs stopping rules on the number of starts.

**5. Acceptance.**
- As for the monolithic problem.
- In multistart, every local solution is assessed in original space. "Best found" is not "global"; a
  certificate needs a global solver (SCIP) or bounds.

**6. Refusal and abandonment.**
- *Facts before solving.*
  - No block structure, i.e. dense coupling.
  - Large linking dimension: use the implicit variant.
  - A structurally singular scenario block.
  - Unbounded variables: multistart sampling is undefined.
- *In-solve observations.*
  - Per-block inertia mismatch: the regularization must be applied consistently across blocks.
  - Ill-conditioned `S`.
  - PCG non-convergence (implicit variant).

**7. Scaling, sparsity.** Block sparsity is preserved. `S` is dense. Scaling must be consistent across
scenarios.

**8. Cost.** Parallel per-block factorizations, plus `n_d` solves per block to form `S`. It pays off
with many scenarios and few linking variables. Without parallel execution the main gain is memory and
locality.

**9. Interactions.**
- M14: Schur elimination is linear reduced space.
- M13: scenario blocks are the connected components of the incidence graph once the linking variables
  are removed [interp].
- M9: warm starts and predictions across neighbouring scenarios.
- M10 anchors and M16 screening feed multistart.

**10. Library realization and minimum exposure.**
- The Ipopt C API has no scenario decomposition. A custom linear solver is a C++-interface feature;
  whether the vendored build allows it was not checked.
- POUNCE `pounce-sens-core` has a `schur_driver.rs`. The file name was observed, but the content was
  not reviewed; it is likely sensitivity-oriented, not scenario decomposition.
- PIPS-NLP, Parapint and MadNLP are references.
- In the stack today: monolithic Ipopt or POUNCE with sparse direct solvers (MUMPS / FERAL).
  Decomposition would be bespoke.
- Multistart is a *composition of standard solves* (a study composition).
- The pipeline must expose:
  - scenario and linking identity in the problem representation;
  - per-block KKT assembly;
  - per-block inertia;
  - start-generation bounds.

**11. References.**
- Zavala, Laird & Biegler, Chem. Eng. Sci. 63(19) (2008), doi:10.1016/j.ces.2007.05.022 [TLDR].
- Kang, Cao, Word & Laird, CACE 71 (2014), doi:10.1016/j.compchemeng.2014.09.013 [TLDR].
- Chiang, Petra & Zavala, PIPS-NLP, PSCC 2014, doi:10.1109/PSCC.2014.7038374 [abstract].
- Rodriguez et al., Parapint, INFORMS JoC (2023), doi:10.1287/ijoc.2023.1272 [abstract].
- Shin, Zavala & Anitescu, overlapping Schwarz for graph-structured optimization, IEEE TCNS (2020),
  doi:10.1109/TCNS.2020.2967805 [abstract].
- Ugray et al., OQNLP, INFORMS JoC 19(3) (2007), doi:10.1287/ijoc.1060.0175 [abstract].

---

## Cross-cutting contracts implied by M9–M17

### X1. Derived-problem identity and provenance

Every derived or auxiliary system should carry one record [interp, synthesized from the cards]:

| Field | Content |
|---|---|
| Original identity | Canonical model, specification and frozen numerical policy (final tolerances, scaling). |
| Generation rule | One of an enumerated set, with its parameters: natural-parameter(param id); pseudo-arclength(param id, θ); Newton-homotopy(anchor); fixed-point-homotopy(anchor, `A = D·P_M`, seed); affine-homotopy(anchor); bounded-homotopy(zone); model-homotopy(low-fidelity model id); Ψtc(`V` source: authored dynamics / matching pairing / gradient flow); DAE-IC(mode, identity vector, specification set, deviation allowed?); block-subsystem(block id, predecessor snapshot); reduced-space(partition, inner tolerance); NPC(partition, inner solver); surrogate(model id, samples). |
| Anchor(s) | Points and their own provenance. An anchor is a seed proposal, never a result. |
| Path parameter | Value(s) of `λ`, `t` or `δ` at which a point was produced. |
| Affected rows and variables | The incidence delta. The auxiliary term's incidence unioned with the original's is the structure on which BTF and matching must be computed (Modelica BLT remark). |
| Coordinate map | Identity for the residual-level constructions. A lifting map for reduced-space or ROM coordinates (`y(z)`, `V x_r`). |
| Terminal identity | A mechanically checkable statement that the derived system *equals* the original at its terminal value (`H(·, 1) ≡ F`, `δ → ∞` gives the Newton step on `F`, `λ = λ_target`, block union = full system). Mechanisms without such an identity (surrogates, least-deviation IC) must say how their output re-enters the original problem: always as a seed. |
| Budget | Steps, factorizations, evaluations and arclength. |

### X2. Intermediate-solution semantics (four classes)

1. *Physical-neighbour solutions.* Natural-parameter continuation over an authored parameter gives
   valid solutions of a *different* specification. They may be retained or published only when
   labelled with that specification.
2. *Auxiliary-path points.* Constructed homotopy points, Ψtc iterates, preconditioned iterates and
   surrogate optima. These are seed material only.
3. *Exact partial solutions.* A BTF prefix: a known subset of the original rows is satisfied at `x`.
4. *Modified-specification solutions.* Least-deviation DAE-IC (Gopal–Biegler). These solve the
   original equations with *altered* specified values, and the deviation must be reported.

The terminal output of every mechanism enters the pipeline as a seed for the original correction.
Acceptance belongs only to the original-space assessment.

### X3. Escalation driven by facts and in-solve observations (no benchmarking)

*Admission facts, known before solving:*
- smoothness class;
- structural rank and the DM partition;
- the BTF block sizes;
- the presence of authored parameters with known `∂F/∂λ`;
- an authored dynamic form with holdups (a Ψtc flow);
- the differentiation index and identity vector;
- bounds and domain restrictions of the evaluators;
- block structure with linking variables;
- a declared low-fidelity model;
- the study shape (single solve versus outer loop).

*Trigger observations, from the solver and the evaluators:*

| Observation | Indicates | Next mechanism |
|---|---|---|
| Line-search or restoration failure far from the root; stagnation at a local min of `‖F‖` | Basin or globalization problem | Per-block solve (M13) → per-block or global homotopy (M10), Ψtc if a flow is justified (M11), continuation on an authored parameter (M9) |
| Evaluator domain errors at trial points | Path or iterates leave the domain | Bounded homotopy (M10) or projected Ψtc (M11); step-cut policy |
| Failures localized in one BTF block across attempts | Local stiff nonlinearity | Nonlinear elimination / right preconditioning (M14/M15) |
| Singular Jacobian at a structurally nonsingular point | Numerical singularity, a fold, or a dynamical invariant | Pseudo-arclength (M9) for folds; Ψtc handles invariants; report the unqualified witness |
| `λ̇` or `t` reversal; `‖x‖` growth on a path | Fold, returning path, path to infinity | Arclength (M9); new anchor or abandon (M10) |
| `δ` stagnation in Ψtc | Non-attracting target | Abandon Ψtc |
| IDACalcIC failure codes | Bad algebraic guess or index > 1 | Block solve of the algebraic subsystem, then SLP (M12) |
| Trust-region ratio `ρ ≤ 0` repeatedly | Surrogate inadequate | High-fidelity direct (M16) |

The ladder is ordered by the *assumptions* each rung adds, not by measured speed:
1. ordering only (M13);
2. local elimination (M14/M15);
3. parameter paths that keep physical meaning (M9);
4. constructed paths (M10);
5. constructed flows (M11).

Multistart (M17) varies anchors at any rung. Every rung ends in the same original correction and
assessment.

### X4. Qualifying structure witnesses before elimination or block solving

- Matching and BTF give *generic* rank. Each block that is to be solved or eliminated needs a numeric
  qualification at the point of use: pivot magnitudes relative to scaling, or a condition estimate, or
  a rank-revealing factorization for small blocks.
- The qualification must be re-checked whenever the point moves. For M14 that means every outer
  trial point.
- 1×1 explicit inversion needs uniqueness on the bounds (monotonicity in that variable). Otherwise it
  is a bracketed scalar root.
- DAE structural index: Pryce-style nonsingularity of the system Jacobian.
- Incidence must be exact, including opaque-call inputs and the union over piecewise branches.
- Record the witness together with its numerical qualification and validity region (a point or a box)
  in the provenance.
- Repository consistency: the repo already states that block structure does not prove independent
  eliminability and that matching does not prove numerical rank (colleague's reading of the
  architecture; `pse-structural` refuses deficient scope).

### X5. Pitfalls

- Treating `t` or `λ` as the step parameter through folds (natural continuation and the Newton
  homotopy fail exactly there).
- Accepting a branch reached after a fold as "the" solution. Record fold and bifurcation crossings.
- Wrapping an arbitrary residual in pseudo-time. The row order and sign of `F` change the dynamics.
  Unstable physical steady states are unreachable by Ψtc.
- Computing BTF on the original incidence while solving an auxiliary system whose incidence differs.
- Densifying the Jacobian through the homotopy construction (`A = I` with a zero-free diagonal
  missing; a single-penalty bounded homotopy).
- Inner tolerances looser than the outer ones in reduced space, which gives inconsistent derivatives
  and false stagnation.
- Testing convergence on a preconditioned residual instead of the original one.
- Surrogate success taken as model success. Least-deviation IC silently changing user
  specifications.
- Promoting an intermediate point (path, pseudo-time, surrogate) to a result, or into
  `PreviousAccepted` warm-start state.
- Claiming probability-one guarantees for nonsmooth process models or without boundedness.

### X6. Consolidated minimum information the pipeline must expose

- `F` and sparse `F_x`, with typed evaluation errors (domain versus numerical).
- `∂F/∂p` for any authored parameter.
- Variable and row scaling; bounds and domain.
- Reusable factor handles; Hessian-of-Lagrangian blocks and inertia.
- Exact incidence, including opaque-call inputs and piecewise unions, and the DOF specification.
- Matching, DM and BTF results with per-block sub-evaluators (residual subset, Jacobian sub-block).
- Mass or holdup operator and the differential/algebraic identity vector; mode identity at events.
- Scenario and linking identity.
- A low-fidelity evaluator over identical variable identities, with provenance.
- Callback cost and count metrics, which already exist per the colleague's reading.

---

## Evidence limits

- **Read in full or in relevant sections:**
  - LOCA theory manual §§2.1–2.2;
  - Kelley et al. 2008, "Projected Ψtc" (restates KK98 Assumption 1.1 and SER);
  - Deuflhard ZIB 02-14 §§1–3;
  - Brune et al. §§3–5;
  - Dolean et al. §3;
  - Sielemann et al. 2011 (Modelica homotopy);
  - PETSc `TSPSEUDO` and SNES manual pages;
  - SUNDIALS IDAS initialization docs;
  - vendored `idas.h` and `kinsol.h`;
  - Diffsol `state.rs`;
  - Pyomo and IDAES SCC, BT-initializer and implicit-function sources.
- **Abstract, TLDR or secondary only:**
  - Paloschi 1995/1997/1998 (the bounded-homotopy formulas were *not* read; the claims about dense
    versus sparse Jacobians come from secondary summaries);
  - Wayburn–Seader (TLDR, truncated);
  - Pattison–Baldea;
  - Kelley–Keyes 1998 and CKK 2003 (CKK assumptions beyond "index 1" not read);
  - Brown–Hindmarsh–Petzold, Pantelides, Kröner et al., Unger et al., Gopal–Biegler;
  - Pothen–Fan, Duff–Reid, Gundersen–Hertzberg, Parker et al. 2023 and 2022;
  - Biegler–Nocedal–Schmid, Cai–Keyes, Alexandrov et al., Peherstorfer et al., Benner et al.,
    Eason–Biegler;
  - Zavala et al., Kang et al., PIPS-NLP, Ugray et al.;
  - HOMPACK/HOMPACK90;
  - Watson 2002 (theorem restated via arXiv:1808.08052).
- **Not accessible:**
  - the Allgower–Georg book (the step-length rule comes from secondary sources);
  - Trilinos online docs (DNS failure; the LOCA SAND manual was used instead);
  - the Malinen thesis.
- **Library negative claims** (no continuation, Ψtc or NPC in the stack) rest on the native skill's
  route matrix and header greps. They are not exhaustive proof.
- The `[interp]` items are this worker's derivations and need review:
  - the matching-permutation sparsity-preserving homotopy;
  - the Ψtc pairing requirement;
  - the BTF-per-parameter scope;
  - the intermediate-solution classes.
