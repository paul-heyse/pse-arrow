> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Principal reviewer's mathematical analysis.
>
> This version was revised after an adversarial numerical-methods challenge ([numerical-methods-challenge.md](numerical-methods-challenge.md)). Repository facts cite the wave-1 evidence in this folder: A1–A3, B1–B5b. Baseline: `ff5a4aa2f` plus the Plan 36 working tree, diff fingerprint `45ceea51b0698679`.
>
> Method behaviour comes from the cited literature and library documentation. As a recommendation for this repository, every method claim is *Proposed*. None is a measurement.

# Method landscape and selection: what the pipeline should be able to choose, and on what basis

## 1. What a process simulator actually solves

The simulator's workloads produce a small number of mathematical shapes: edit → re-solve, studies, recycles, dynamics and extension. The numerical character of a shape decides which method works better than its nominal class does.

| Shape | Typical size and structure | Dominant difficulty | Where it arises |
|---|---|---|---|
| Square sparse nonlinear system F(x) = 0 (equation-oriented flowsheet) | 10² to 10⁶ unknowns, very sparse. BTF gives many small blocks plus a few large strongly connected components (SCCs) from recycles and heat integration. | Global convergence from poor starts. Bounds the physics requires. Restricted domains: log, sqrt, fractional powers. Near-singularity at phase boundaries and turning points. Magnitudes spanning 10⁻¹⁰ to 10⁸. Nonsmooth correlations. | Simulation, initialization, every block of a block sequence |
| Low-dimensional fixed point x = G(x) (sequential-modular tear) | 10 to 10³ tear variables. Each G evaluation is expensive because it solves the units. | Slow convergence when ∂G/∂x has a dominant eigenvalue near 1. Divergence or oscillation. A G evaluation can fail. G is "noisy" when unit solves are inexact. | Recycles, design specifications |
| Scalar or small dense implicit equations | 1 to 10 unknowns, evaluated very many times | Bracketing, multiplicity, branch selection (equation-of-state roots), poles | Property providers, nested implicit functions, 1×1 BTF blocks |
| Nonconvex NLP with few degrees of freedom | n large, n − m small, bounds active | Nonconvexity; degeneracy (LICQ failure); poor scaling; active-set change along studies | Optimization, design, NMPC |
| Nonlinear least squares | Many residuals, often block-angular (experiments sharing parameters) | Rank deficiency and identifiability; small versus large residual; outliers | Estimation, data reconciliation |
| DAE initial-value problem | 10² to 10⁵ states, stiff, with events. Mostly index 1. Higher index arises from fixed-volume gas holdups, pressure–flow networks and output specifications. | Consistent initialization, stiffness, discontinuities, index | Dynamics, shooting |
| Large structured NLP from discretization | 10⁴ to 10⁷ unknowns; banded (collocation) or block-angular (multi-period) structure | KKT fill; trajectory initialization | Dynamic optimization, multi-period design |
| LP, MILP, convex QP | Tear MILP, data-reconciliation QP, MPC QP, planning | Degeneracy; integrality gap; warm starts along sequences | Tears, reconciliation, MPC |
| MINLP / global | Synthesis, certification | Exponential search; relaxation quality | Synthesis, certification |

**Consequence for selection.** The best method depends more on structure than on class. Structure here means block decomposition, size and sparsity, degrees of freedom, how bounds arise, the contraction and noise of a fixed-point map, nonsmoothness, stiffness and index. One "square root" problem decomposes into small blocks and a few large SCCs, and each wants a different method.

## 2. Square nonlinear systems

Primary references: Deuflhard 2004, *Newton Methods for Nonlinear Problems*; Kelley 2003; Dennis & Schnabel 1996; Nocedal & Wright 2006, chapter 11; Brune, Knepley, Smith & Tu 2015.

### 2.1 Choosing a method for each BTF block

BTF yields ordered square blocks solved by forward substitution.

- **What forward substitution guarantees.** It is exact in exact arithmetic. In floating point, upstream tolerances propagate. An early block with several roots can commit to a root that makes a later block infeasible, a commitment that a simultaneous Newton solve would not make. A structurally nonsingular block can still be numerically singular at the current point.
- **Why per-block independent assessment matters.** The repository already keeps per-block independent original assessment, commit and rollback (§17.1). That is the right protection to retain.

| Block | Warranted method | Library owner | Conditions and cautions |
|---|---|---|---|
| 1×1, affine in its unknown | Closed form x = −b/a | none needed | Guard a ≠ 0 *and* \|a\| relative to the scales of b and x (PS-06; Higham 2002). Check the residual afterwards. |
| 1×1, nonlinear, with a bracket from bounds or guards | Bracketed method (Brent) | argmin `BrentRoot`, pinned (B5a) | Needs continuity on the bracket: at a pole such as 1/x it converges to the singularity. Returns *a* root, not necessarily the physical branch. |
| 1×1 needing branch certainty or proof of absence | Interval isolation; certified polynomial roots | IBEX (bound); Symbolica certified cubic (bound) | IBEX also proves *no root* in a box. The certified cubic is the right tool for equation-of-state branches. |
| 1×1 without a bracket | Bound-projected damped Newton, plus a bracket search over the finite box, or IBEX | KINSOL Dense/line search; IBEX | Bisection needs a sign change, so "Newton with bisection fallback" is not available without a bracket. |
| Affine block (affine in the block's unknowns, given upstream values) | One sparse LU solve, with a residual check or iterative refinement | faer or KLU (bound) | Coefficients are evaluated at upstream values. |
| Small dense nonlinear block | Dense Newton with line search, or a dogleg trust region | KINSOL Dense (bound); PETSc `NEWTONTR`/`NEWTONTRDC` (bound) | Line search, dogleg and LM all minimize ‖F‖² and share its non-root stationary points. None is affine invariant. The dense/sparse crossover is uncalibrated: OpenModelica uses a density threshold of 0.1 and a size threshold of 1000 as reference practice (B5b). |
| Large sparse block | Sparse Newton with direct LU; Newton–Krylov with BTF-block preconditioning when direct LU is too costly | KINSOL KLU; KINSOL SPGMR/SPFGMR with the bound FERAL BTF-block preconditioner | Krylov pays only with a good preconditioner. |
| Large SCC with decomposable substructure | Nonlinear additive Schwarz or ASPIN over a partition *inside* the SCC | PETSc NASM (bound, explicit-only) | NASM adds nothing over triangular BTF blocks, where forward substitution is already exact. ASPIN needs subdomain Jacobians. |

The share of 1×1 and affine blocks in this repository's flowsheets is *unmeasured*. Literature and experience suggest it is large, and presolve alias and linear elimination reduce it.

**Equation-level tearing** (Elmqvist & Otter 1994; Cellier & Kofman 2006) chooses iteration variables t inside an SCC so the rest of the block solves sequentially as r(t). It reduces the Newton dimension, and exact derivatives follow from the implicit function theorem. Its risks are material (Baharev, Schichl & Neumaier):

- it is structural pivoting without numerical pivoting, so a small pivot amplifies error;
- intermediate r(t) can leave their domains even when t is in bounds;
- g(t) is often more nonlinear and worse conditioned than the original block;
- with a sparse direct LU, a smaller Newton dimension does not imply a cheaper factorization.

Compare NASM and the existing BTF-block preconditioner first. Treat tearing as a later, bounded structural transformation that carries PS-06 guards.

### 2.2 How bounds arise determines the method

Two-sided bounds on the unknowns of a square system arise in three mathematically different ways:

1. **Domain-guard bounds with an interior root.** These are the common case: positivity, fractions in [0, 1]. Fraction-to-boundary step truncation or a log/logit variable transform suffices, and both are cheaper than any bound-constrained solver. KINSOL sign constraints (bound) cover the one-sided case.
2. **The root lies outside the box.** The model and data are inconsistent with the bounds. The honest output is bounded least squares, min ½‖F‖² subject to bounds, as a diagnostic candidate together with an infeasibility report (PS-10).
   - Library owners: TAO BNTR, TRON, or BRGN over BNLS (pinned PETSc, unbound; B2/B5a).
   - Ipopt with a zero objective solves a *different* problem: it ends at a local ℓ1 minimizer through restoration rather than ½‖F‖².
3. **Genuine complementarity**, for example phase disappearance written as an MCP. A VI or MCP solver is then the right tool (PETSc `SNESVINEWTONRSLS`/`SSLS`, pinned, unbound).
   - PETSc pairs Fᵢ with xᵢ: xᵢ = Lᵢ requires Fᵢ ≥ 0, and xᵢ = Uᵢ requires Fᵢ ≤ 0.
   - For a general flowsheet, row order and sign are arbitrary, so an active-bound VI point is an artefact of that pairing. Admit VI only when the formulation declares the pairing.

The selector must know which case applies. That is a formulation fact (PS-06), not a numerical guess.

### 2.3 Globalization

Line search, error-oriented damping and trust region share one failure mode: a stationary point of the merit function with F ≠ 0. They are alternatives with different invariance and conditioning behaviour, not rungs of increasing robustness.

- **Damped Newton with backtracking on ‖F‖²** (KINSOL LineSearch, PETSc `bt`).
  - `KIN_STEP_LT_STPTOL` can mean convergence or stall, so check ‖F‖ before escalating.
- **Error-oriented, affine-covariant damping (NLEQ-ERR)** (Deuflhard; PETSc `SNESLINESEARCHNLEQERR`, pinned, unbound).
  - Invariant under F → AF for any nonsingular A, which is broader than row scaling. It depends on *variable* scaling through its norm on Δx, so it needs the PS-07 nominals.
  - It combines with VINEWTONSSLS but not RSLS.
  - It governs damping with the *simplified* Newton correction Δx̄ₖ₊₁ = J(xₖ)⁻¹F(xₖ₊₁), at the cost of one extra back-solve. The monitor is Θₖ = ‖Δx̄ₖ₊₁‖/‖Δxₖ‖, accepted when the restricted monotonicity test Θ(λ) < 1 − λ/4 holds. The ratio of two ordinary Newton steps is not this quantity.
  - KINSOL does not compute Θ. PETSc computes it internally and has no getter, so the observation needs `SNESMonitorSet` or a selected line search. Reconstructing Θ outside PETSc would duplicate the library's monitor.
- **Trust region** (PETSc `NEWTONTR`, bound; TRDC).
  - Better behaved near singular J.
  - Its radius is observable only through the TR pre- and post-check hooks; no getter exists (B2).
- **Pseudo-transient continuation, Ψtc** (Kelley & Keyes 1998; PETSc `TSPSEUDO`, bound, explicit-only).
  - It converges when the trajectory of the flow M ẋ = −F(x) from x₀ converges to a *stable* x*. With M = I that requires −J(x*) to be stable, which depends on how equations are paired and signed.
  - Index-1 DAEs need the Coffey–Kelley–Keyes variant.
  - It cannot reach unstable steady states, such as the middle CSTR branch.
  - When the authored model has dynamics, its mass matrix is the physically meaningful M.
- **Homotopy and continuation** with a predictor–corrector and pseudo-arclength.
  - *Probability-one* convergence (Chow, Mallet-Paret & Yorke 1978; Watson) holds for the **fixed-point homotopy** H = λF(x) + (1 − λ)(x − a), for almost every a, under a boundary condition, and needs C² on the domain. Domain restrictions break that assumption.
  - The **Newton homotopy** F(x) − (1 − λ)F(x₀) is continuous Newton. It can turn back and pass through singular J.
  - **Physical homotopies** (ideal → rigorous thermodynamics; relaxed specifications) carry no guarantee but often work best. IDAES and the Modelica `homotopy()` operator use them (B5b).
  - Library fit:
    - The Newton homotopy fits PETSc `SNESNEWTONAL` (pinned, unbound) directly, as F̃(x) = F(x) − F(x₀) with a constant load.
    - The fixed-point homotopy needs an x-dependent load, which the documentation does not confirm. The current `kkt/path/arclength.rs` and the existing staged homotopy are the alternatives.
    - Adapt the homotopy step to observed solver effort (iteration targets) rather than geometric growth. This is IDAES `iter_target` and OpenModelica adaptive-τ practice (B5b). The current `StepControl` grows by a fixed factor.
- **Bounded least squares as the honest end of the ladder.** Produce a least-infeasible *diagnostic candidate*, never a solution (PS-10).
  - The diagnostic value is the dependent combinations, read from the left null space of J at that point (Jᵀy = 0), and sparse ℓ1 attribution.
  - Reading large |Fᵢ| depends on row scaling.
  - A local minimum of ‖F‖ does not prove inconsistency.
  - Chinneck's deletion filter gives a minimal inconsistent subset. The repository's `explain_nonlinear` implements one.

### 2.4 Acceleration and cost

- **Modified Newton and Jacobian reuse** (KINSOL `msbset`, residual monitoring; partly bound). Pays when Jacobian evaluation is expensive relative to residuals; `CallbackState.seconds` already holds the cost ratio.
- **Inexact Newton with Eisenstat–Walker forcing** (bound for Krylov). The same theory governs inexact *function* evaluation, such as nested unit solves, so inner tolerances should track outer forcing.
- **Broyden** (PETSc `SNESQN`, unbound). Useful when J is expensive or unavailable.
- **Deflation** (Farrell, Birkisson & Funke 2015). After one root, finds others, for wrong-branch or nonphysical roots in phase and equation-of-state blocks. It is a small composition over any Newton.
- **Rank-reduction Newton** (NLEQ2). Handles near-singular J at phase boundaries; it is a candidate library (B5a).

## 3. Recycle (tear) convergence

Primary references: Westerberg, Hutchison, Motard & Winter 1979; Biegler, Grossmann & Westerberg 1997, chapters 3–4; Walker & Ni 2011; Fang & Saad 2009; Kelley 2018; Upadhye & Grens 1975. Practice: Aspen Plus (B5b).

- **Damped successive substitution.**
  - Converges for every real eigenvalue μ < 1 of ∂G/∂x, *including μ ≤ −1*, with suitable damping β (|1 − β(1 − μ)| < 1).
  - The current default is undamped (damping 1, Anderson depth 0). KINSOL damping is bound.
  - Oscillation is the case damping fixes.
- **Anderson(m).**
  - A multisecant type-II Broyden method (Fang & Saad).
  - With untruncated memory and no stagnation it is GMRES-equivalent on linear problems (Walker & Ni), and it *can converge when ρ(G′) > 1*.
  - Proved rates are r-linear with an improved rate.
  - It adds no G evaluations. Its practical limiter is the **noise floor of G**: inexact unit solves cap the attainable accuracy.
  - Library-owned: KINSOL FP + MAA, with delay, damping and orthogonalization bound; or `SNESANDERSON`.
- **Broyden on F(x) = x − G(x)** (`SNESQN`, unbound).
  - Locally superlinear when I − G′ is nonsingular.
  - Largely overlaps Anderson, so a separate rung needs a reason, such as a better B₀ from a dense finite-difference tear Jacobian.
- **Dense finite-difference tear Jacobian + Broyden.** For n_tear ≲ 50 this is the classic and often cheapest exact-ish route. Its columns are independent evaluations of G (a future parallel seam). It is project composition, or `SNESQN` with a supplied initial Jacobian.
- **JFNK on tears** (KINSOL line search + SPGMR with the internal difference-quotient Jv, unbound because the adapter requires an analytic Jv; or PETSc `-snes_mf`). It needs four conditions:
  - G is deterministic, free of hysteresis and independent of warm starts;
  - G is smooth along the probes;
  - inner unit tolerances lie far below the difference increment (Jv accuracy is about √ε_inner, so typical inner tolerances degrade it to linear convergence);
  - perturbations do not push units into infeasibility, which may need one-sided differences.

  These are *conditions to check, not defaults*.
- **Exact Newton on tears.** Needs J = I − ∂G/∂x via implicit-function-theorem sensitivities through the unit chain. In reverse mode that costs one adjoint per *tear output* propagated through the chain; per unit, about min(inputs, outputs) back-solves after factorization. Units are opaque causal maps today, and sensitivities are refused in the sweep (A3), so this is a later capability.
- **Wegstein (bounded q) and dominant-eigenvalue / Aitken extrapolation.**
  - Componentwise, history-dependent q makes the map stateful. It cannot ride on KINSOL FP, and β > 1 is not expressible through `KINDampingFn`, so it must be an owned loop.
  - q ∈ [−5, 0] only accelerates; it *cannot damp* oscillation.
  - Mathematically it is largely subsumed by Anderson. Extrapolation methods (MPE/RRE) generalize the dominant-eigenvalue method.
  - It is the *default tear method in Aspen Plus, Pyomo SequentialDecomposition, DWSIM and IDAES practice* (B5b). That parity with the reference tools is its main justification. Adopting it is a bounded composition PS-09 permits, but it is not mathematically necessary.
- **The tear set changes the conditioning.** It changes ρ itself, not only the count (Upadhye & Grens). Tear selection should weigh convergence properties, for example stable tear variables and torn-loop weights (Aspen) or minimax tears per loop (Pyomo). Unweighted greedy is the current default (A1/A3).
- **Design specifications.**
  - *Nested* convergence is robust only when inner tears converge at every outer trial value; inexact inner convergence corrupts the outer secant.
  - *Simultaneous* convergence (one Broyden or Newton system) is faster.
  - Aspen exposes this as `Design Spec Nesting` = Inside (default), Outside or Simultaneous, with staging such as "converge tears to a loose tolerance first" and "wait N direct iterations before acceleration".
  - In equation-oriented form a specification is simply a fixed/free variable swap. This is a declared policy (PS-08), chosen by rule or observation.

**Observable selectors.** All are cheap once recorded, and none is recorded today:

- the geometric-mean contraction ρ̄ over a window (diffsol-nl's estimator), not a single ratio;
- the sign of Δxₖ₊₁·Δxₖ, which detects oscillation;
- per-tear normalized error |x − G(x)|/tol, which Aspen's iteration views key on;
- the unit-failure rate on tear trials;
- the inner-tolerance/noise level of G.

KINSOL `KIN_FP` calls the map exactly once per iteration, so the history is available from the callback at no extra cost (B2).

**Rule sketch** (*Proposed*):

1. Start with damped successive substitution or Anderson(m ≈ 3–5) with a delay.
2. On oscillation, increase damping.
3. On slow contraction, increase Anderson depth.
4. With noisy G, tighten inner tolerances before acceleration.
5. On persistent failure, move to the simultaneous equation-oriented route.
6. A small tear set may use a finite-difference Jacobian with Broyden.
7. JFNK only when its conditions hold.

## 4. NLP

Primary references: Nocedal & Wright 2006; Biegler 2010; Wächter & Biegler 2006; Nocedal, Wächter & Waltz 2009; Zavala & Biegler 2009; Vanaret & Leyffer 2024 (Uno). Library facts are from B1.

- **Interior point (Ipopt; POUNCE IPM).**
  - The default for large sparse nonconvex NLPs.
  - Warm starts help only when well centered: bound and multiplier pushes and μ reduced together, monotone μ, small `mu_init`. Adaptive μ can *raise* μ and destroy a warm start.
  - Same-structure re-solve (`ReOptimizeTNLP`, FERAL pattern reuse) saves symbolic analysis regardless.
  - The strongest study warm start for an interior point method is a **first-order sensitivity predictor** (sIPOPT style; POUNCE `step_along_path`, used in the repository's prediction layer).
- **Active-set SQP** (POUNCE SQP with working-set transfer, bound; Uno filterSQP, bound with L-BFGS and HiGHS QP).
  - Strong along sequences with few active-set changes.
  - Uno's own `auto` rule switches to an interior point at n + m ≥ 2000 or nnz ≥ 5·10⁴; these thresholds are uncalibrated upstream.
  - Uno is built with HiGHS only, so its interior-point preset cannot run here.
- **The strongest warm start in the stack** is HiGHS simplex with a retained basis, for LP and QP sequences. The adapter's `Highs_clearSolver` before each solve discards HiGHS's internal hot-start state (B4).
- **Hessian.**
  - Exact when available.
  - **Finite-difference Hessian with CPR coloring** (POUNCE) when the sparsity is known and second derivatives are missing or expensive. That is often better than L-BFGS.
  - L-BFGS otherwise.
  - Partitioned quasi-Newton for element-separable Lagrangians.
  - Gauss–Newton only for least squares.
- **KKT linear solver.**
  - Inertia-revealing factorization is required for nonconvex inertia correction.
  - MKL Pardiso's inertia is unreliable under perturbed pivots, which disqualifies it for nonconvex problems (B1).
  - Reference practice: MA27 and MA57 are fast serially on small-to-medium KKT systems; Tasseff et al. 2019 often found MA27 the best serial choice. MA86, MA97, SSIDS and Pardiso suit large problems with many threads. MUMPS is robust but slower.
  - Ordering matters as much as the solver: POUNCE 0.12 reports METIS 2–3.3× faster on collocation and optimal-control structure and slower on grid QPs. That is upstream's measurement, not this repository's.
  - Build facts:
    - No HSL in the Ipopt image. For a non-academic personal user only MA27, MA28 and MC19 are free, from the Coin-HSL Archive; MA57 and MA97 are free only to academics (B5a). MC19 also gates Ipopt's equilibration scaling.
    - Ipopt's inexact algorithm is not built.
- **Barrier strategy.**
  - Monotone is predictable and suits warm starts.
  - Adaptive with the quality-function oracle usually needs fewer iterations from cold starts. Its `obj-constr-filter` globalization falls back to monotone.
  - Mehrotra probing is for LP and convex QP.
- **Reduced space and feasible path.** Eliminating the states at every iterate through a converged square solve (an implicit function) is the **feasible-path** approach. Bounds on the eliminated variables can remain as outer inequalities with implicit-function gradients. This differs from **reduced SQP** (Biegler, Nocedal & Schmid 1995), an *infeasible-path* null-space method. Both suit n − m ≪ n. The current `ReducedSpace` mechanism is the feasible-path kind.
- **Schur decomposition for block-angular structure** (multi-period, multi-scenario, multi-experiment).
  - Each block's KKT must be nonsingular. Inertia is additive (Sylvester's law of inertia), so the dense Schur complement over the linking variables must stay small.
  - Only POUNCE offers it here; Ipopt does not.
  - It is also the natural future parallel seam.
  - References: Zavala, Laird & Biegler 2008; Kang et al. 2014.
- **MPCC** (phase appearance, flow reversal, vanishing stages) violates MFCQ.
  - Regularization schemes (Scholtes) can converge to C- or M-stationary, i.e. spurious, points.
  - The ℓ1 elastic formulation (POUNCE `L1ExactPenalty`) reaches strongly stationary points under MPCC-LICQ (Anitescu 2005; Ralph & Wright 2004).
  - IDAES uses a smooth-max square flash (Burgard et al. 2019), which matters for parity.
  - PS-06 requires the choice to be declared and its approximation reported.
- **Global.**
  - Multistart improves local optima and proves nothing (PS-12).
  - Spatial branch-and-bound (SCIP) is worst-case exponential, needs finite boxes and a factorable export, and its certificate is floating-point and tolerance-based.
  - For *small* factorable problems, a budgeted SCIP run is the one method that can **certify infeasibility**. That is the warranted response to a "locally infeasible" verdict when the request permits it. It is not a warranted automatic response to a generic local failure.
  - Current code places SCIP third in the automatic SmoothNlp chain, reached when SCIP's own eligibility holds (A1).
- **Ipopt restoration slacks.** Restoration is an ℓ1-elastic problem with a proximal term (Wächter & Biegler 2006 §3.3). Its slacks p and n attribute infeasibility to constraints at no extra cost, but they are not exposed today. Rerunning a separate ℓ1 solve from the same point often lands in the same place.

## 5. Least squares and estimation

- **Gauss–Newton** converges quadratically only at zero residual; otherwise linearly with rate about ‖(JᵀJ)⁻¹S‖. LM or a trust region guards ill-conditioning.
  - Library owner: PETSc TAO BRGN with LM regularization, pinned and unbound (B2, B5a). This is a binding task, not a method to write.
- **Large residual:** a full Hessian or structured quasi-Newton (NL2SOL; TAO BQNLS or BNTR).
- **Data reconciliation:** keep the constraint curvature λ∇²c unless multipliers are small. Robust M-estimators (Huber, Cauchy) handle gross errors. Ceres provides them; it is a candidate library (B5a).
- **Multi-experiment estimation** is block-angular.
  - Exploit it through Schur decomposition in an interior point method (Zavala, Laird & Biegler 2008; Kang et al. 2014) or through Bock's condensing in multi-experiment Gauss–Newton.
  - parmest solves the extensive form, and DAKOTA treats the model as a black box; neither uses Schur.
  - The current fitting NLP sets no separator (A3).
- **Choosing Gauss–Newton + forward sensitivities versus L-BFGS + adjoint gradient** is a real method choice. It depends on the parameter count, the residual count and the residual size. Today it comes from the `FitDerivatives` profile (`fitting/modeling.rs`).
- **Identifiability diagnostics:** SVD of the scaled sensitivity matrix, collinearity indices (Brun et al. 2001) and FIM eigenvalues. All are local, and at a failed fit they describe the failed point. Profile likelihood (Raue et al. 2009) assesses practical identifiability.

## 6. Dynamics

Primary references: Brenan, Campbell & Petzold 1996; Hairer & Wanner 1996; Hindmarsh et al. 2005; Kennedy & Carpenter 2016. Library facts are from B3.

- **Index.** High index arises from fixed-volume gas holdups, pressure–flow networks and output specifications.
  - Index detection and reduction (Pantelides; dummy derivatives, Mattsson & Söderlind 1993) is the missing upstream step; the current scope refuses higher-index problems (§25).
  - A singular ∂g/∂z at t₀ (index > 1 there) is the dominant failure of consistent initialization, so it calls for a rank or Dulmage–Mendelsohn check. Algebraic residuals alone do not diagnose it.
- **BDF (IDAS; Diffsol BDF).**
  - Efficient on long smooth segments.
  - A(α) stability is about 86°, 73° and 52° at orders 3, 4 and 5, so oscillatory modes force order reduction. IDA has no stability-limit detection.
  - `IDAReInit`, needed for state resets and right-hand-side switches but not for plain root returns, restarts at order 1 with a forced Jacobian setup. With fewer than about 10–20 steps per segment, BDF never amortizes.
- **One-step L-stable methods.**
  - TR-BDF2 (γ = 2 − √2) is L-stable. Check the Diffsol `esdirk34` tableau.
  - Stage order 2 gives order reduction on stiff and algebraic components, so "full order" means nominal order 2–3.
  - These methods suit event-dense horizons with *located* events. Restarts still need a new h₀ and a refactorization.
- **Rosenbrock** (PETSc TSROSW; reachable through the existing TS bridge with a type switch). One-step and linearly implicit, so it suits event-dense index-1 systems.
- **Radau IIA** is stiffly accurate with high stage order and is Hairer & Wanner's reference for DAEs. It is in no pinned library.
- **Multirate and IMEX (ARKODE).** Mass-free only, because ARKODE needs a nonsingular mass matrix. `sundials-sys` 0.6.2 generates no ARKODE bindings, which makes this a later option.
- **CVODES** is bindable by enabling a feature and handles ODEs only. Its additions are Adams for non-stiff problems, stability-limit detection, projection and a mature adjoint.
- **Repeated error-test failures near one time** signal an *unlocated discontinuity*. Locate it with a root function and reinitialize. Tightening tolerances makes it worse.
- **Sensitivities.**
  - Compare p·c_fwd with q·c_adj. Locally, c_adj includes replaying each earlier segment forward.
  - Gauss–Newton needs the full residual Jacobian, which favours forward sensitivities.
  - Local adjoints refuse events.
  - Second order needs forward-over-adjoint.
- **Dynamic optimization.**
  - Collocation handles unstable dynamics and path constraints, but its mesh is fixed and it takes no authored events.
  - Multiple shooting keeps adaptive error control and events, needs sensitivities at every node, and segments the horizon. The segments are independent, a future parallel seam.

## 7. Cross-cutting

### 7.1 Scaling

- PS-07 places nominal magnitudes in the model contract.
- Use automatic Jacobian-based scaling (Curtis–Reid, geometric mean, Ruiz) as a **diagnostic and suggestion**: report rows and columns whose declared nominals disagree with the Jacobian's natural scale.
- As a remedy, the libraries own scaling: MC64 inside FERAL and POUNCE, Ipopt `gradient-based` (which only scales down), Ruiz inside Clarabel.
- IDAES's `ScalerProfiler` compares scaling methods by condition number and by re-solve success. There is no equivalent here (B5b).

### 7.2 Linear algebra

| Problem | Factorization |
|---|---|
| Small n | Dense LU |
| Circuit-like unsymmetric matrices | KLU (BTF + AMD/COLAMD) |
| Symmetric-indefinite KKT systems with inertia | MUMPS, Pardiso, SSIDS, HSL |
| Rank-revealing sparse QR | SPQR only. faer's sparse QR does not pivot (B5a); see Plan 36 R36-01. |

Free condition estimates are computed and then discarded today (B2):

- KLU `rcond`: the crude min/max |U_kk| ratio, free after a refactor;
- KLU `condest`: a few extra solves;
- FERAL κ₁, min |pivot| and `n_tiny`;
- KLU `numerical_rank` and `singular_col`.

All are meaningful only on the *scaled* matrix.

### 7.3 Derivatives

- Exact first and second orders come from Symbolica. The KINSOL Krylov route uses directional actions without assembling J; the earlier F16 is closed for that route (A3).
- Hessian-vector products need forward-over-reverse or second-order directional derivatives. None exists (A3).
- Newton converging only *linearly with constant contraction* signals a wrong or inexact Jacobian, or a singular root. The remedy is a derivative check (Ipopt `derivative_test`, PETSc `SNESTestJacobian`), not more globalization.

### 7.4 Presolve

- Use FBBT and OBBT bound tightening as *information*: initialization projection, brackets, scaling, SCIP relaxations, domain-trial prevention. Tightened bounds may not be usable as NLP barriers, because their duals are untracked (the code's stated reason).
- Repeated domain or NaN trials call for tightening and step truncation, not another globalization rung.

## 8. Feature model

Each feature has a **lifetime**, meaning what invalidates it:

| Lifetime | Invalidated by |
|---|---|
| `Structure` | A structural edit |
| `BoundSnapshot` | A bound or domain change |
| `Values` | A change of fixed or parameter values |
| `History` | The previous outcome for the same structure, or the distance from the last converged point (sits between `Values` and `Start`) |
| `Start` | A new initial point |
| `Iterate` | Progress within the solve |

Only `Structure` and `BoundSnapshot` features may enter compiler facts or prepared-structure identity.

| Feature | Lifetime | Cost | Predicts | Caution |
|---|---|---|---|---|
| Structural rank; DM over- and under-determined parts | Structure | Already computed | Every method fails (refuse before solving; PS-04) | Strongest signal; already an admission refusal |
| n, m, nnz, density | Structure | Free | Dense or sparse; direct or Krylov; KKT solver | Crossovers are uncalibrated |
| BTF block count and size distribution; largest SCC | Structure | Already computed | Per-block method; block sequence versus simultaneous | — |
| Affine-in-block-unknowns per block | Structure + Values (coefficient signs and magnitudes) | Affine rows exist; needs projection | Direct solve | A coefficient can vanish at the current values |
| Nonsmooth operators (abs, min, max, piecewise, regime switches) | Structure | Compiler | Newton or JFNK failure; smoothing or semismooth methods | — |
| Domain-restricted operators | Structure, plus distance at x₀ | Compiler, plus one evaluation | Domain trials; step truncation | — |
| Polynomial degree; transcendental or implicit providers | Structure | Compiler facts | Hessian strategy; SCIP factorability | — |
| Degrees of freedom n − m | Structure | Free | Feasible path versus full space | — |
| Separator size against block sizes | Structure | The compiler separator exists | Schur decomposition | — |
| Bound origin: domain guard, data bound, complementarity | Structure (formulation declaration) | Formulation | §2.2 case | Must be declared, not inferred |
| Complementarity, MPCC and GDP structure | Structure | Registry | ℓ1/elastic; reformulation choice | — |
| Convexity certificate | **Values** (the sign of c in c·x²) + BoundSnapshot | Already computed | Local = global; convex-solver eligibility | Does not choose interior point versus active set |
| Integrality | Structure | Free | MILP or MINLP routes | — |
| Parameter count p, output count q, residual count | Request | Free | Forward versus adjoint; Gauss–Newton versus L-BFGS | — |
| Steps per segment between discontinuities | Structure + Request (horizon / expected h) | Estimate | BDF versus one-step methods | Better than a raw event count |
| Index indicators | Structure | DM on the DAE | Index reduction; initialization strategy | — |
| Scaled first Newton correction ‖D⁻¹Δx₀‖; Θ₀ from the simplified correction; α_max to the box | Start | One factorization, plus one back-solve | Need for globalization or continuation; bound handling | Prefer these over ‖F(x₀)‖, which is not affine invariant |
| Scaled condition estimate; numerical rank and `singular_col` | Start | Free or a few solves after the first factorization | Rescale, or trust region; near-singular diagnosis | Must be computed on the scaled J |
| Jacobian-to-residual cost ratio | Start/Iterate | Already in `CallbackState.seconds` | Modified Newton or quasi-Newton | — |
| Inner tolerance and noise of G; unit failure rate | Iterate | Recorded from tear trials | JFNK and Anderson viability | — |
| Θₖ (simplified-correction monitor), backtracks, step lengths | Iterate | Only through PETSc NLEQ-ERR monitoring | Damping, or a switch to continuation | Not available from KINSOL |
| Geometric-mean contraction ρ̄; oscillation sign | Iterate | Free from the FP history (once recorded) | Damping, depth, then simultaneous | A single ratio is unreliable for non-normal maps |
| Ipopt δw / δc, restoration share, inf_pr/inf_du trends | Iterate | Intermediate callback | §9 | δw signals nonconvexity; δc signals LICQ failure |
| IDAS error-test failure clustering in time; error weights × local errors | Iterate | Library getters (unbound) | Unlocated discontinuity; step-limiting states | — |
| Previous winner and outcome for this structure; ‖Δp‖·‖dx/dp‖ | History | History store | Advisory reordering; predictor use | Layer 3 only; reorder within the admitted set |

## 9. Escalation semantics: cause → warranted next step

Escalation should be typed by *diagnosed cause*. The current pipeline keys only on the termination category (A1).

| Observed cause (layer-2 evidence) | Warranted next step |
|---|---|
| Stall at F ≠ 0 with small ‖JᵀF‖, a local minimum of ‖F‖ | Fixed-point homotopy or Ψtc (if a stable flow exists), *not* another merit-function method. Trust region and LM converge to the same point. Read the left null vectors for diagnosis. |
| Line-search failure with a high *scaled* condition estimate | Separate a scaling problem (rescale) from genuine near-singularity (trust region; rank-reduction Newton; deflation if the root is wrong) |
| Repeated truncation at bounds | Root outside the box: bounded least squares and an infeasibility report. Domain guard with an interior root: fraction-to-boundary steps or a variable transform. Declared complementarity: VI. |
| Newton converging linearly with constant contraction | Derivative check (`derivative_test`, `SNESTestJacobian`); exactness claim under suspicion (PS-07) |
| Repeated domain or NaN trials | FBBT tightening and step truncation |
| Converged to a nonphysical root | Deflation, homotopy or a different start |
| Fixed point oscillating | Increase damping (bound) |
| Fixed point slow (ρ̄ near 1) | Deeper Anderson; check the noise of G; then the simultaneous route; for small tear sets, a finite-difference Jacobian with Broyden |
| Ipopt restoration failure, inf_pr stalled | Read the restoration slacks for attribution; the ℓ1 explanation (deletion filter) to localize; budgeted SCIP certification only for small factorable problems when permitted |
| Ipopt with large δw every iteration | Nonconvexity: review scaling, other starts (multistart if permitted) |
| Ipopt with large δc | LICQ failure or redundant equalities: ℓ1/elastic, or `dependency_detector=mumps` |
| IDAS repeated error-test failures near one time | Locate the discontinuity (a root function) and reinitialize; do not tighten tolerances |
| IDAS initial-condition failure | Rank or DM check of ∂g/∂z (index); solve the initialization system with the square-system ladder |

Every transition keeps three things:

- the explicit-selection rule;
- request-owned permissions (reformulation, global, multistart);
- one work budget.

Derived families (ADR-0155) carry provenance of the original equations and of the modifications.

## 10. Decomposition opportunities ("segmentation")

1. A method for each BTF block (§2.1). This is the largest structural lever for equation-oriented simulation.
2. NASM or ASPIN inside large SCCs; equation-level tearing as a later, guarded option.
3. A sequential-modular to equation-oriented hybrid: converge tears or a block sequence to initialize, then a simultaneous polish.
4. Schur decomposition of block-angular NLP and estimation. The compiler produces the structure; routing does not use it.
5. Multiple-shooting segmentation of dynamics.
6. A dense finite-difference tear Jacobian, with independent columns.

All of these are also the natural seams for later parallel execution (H22). They are noted here, not designed.
