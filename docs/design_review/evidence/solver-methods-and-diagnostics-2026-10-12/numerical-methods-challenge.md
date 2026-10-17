> Supporting evidence for [the solver methods, selection and diagnostics review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). An adversarial numerical-methods challenge to the principal reviewer's draft method landscape, by a fresh `library-research` agent, published unchanged. The corrections it raised are folded into [method-landscape-and-selection.md](method-landscape-and-selection.md); section numbers below refer to the draft. Its judgments are the challenger's.

# Adversarial review of the method-landscape draft: square systems, tears, NLP, estimation, dynamics, features and escalation

I worked read-only. I wrote no files. I checked the draft against evidence files A1–A3 and B1–B5a, the PETSc manual pages for `SNESVINEWTONRSLS`, `SNESLINESEARCHNLEQERR`, `SNESNEWTONAL` and `SNESNewtonALSetFunction`, and the standard references. Most method knowledge here comes from the literature and has no measurement behind it. I mark the places where I relied on recall.

## 1. The most consequential corrections

1. **The Deuflhard monitor is defined wrongly (§2.1.4, §8).**
   - Θₖ = ‖Δx̄ₖ₊₁‖/‖Δxₖ‖, where Δx̄ₖ₊₁ = J(xₖ)⁻¹F(xₖ₊₁) is the *simplified* Newton correction: the old Jacobian at the new point, costing one extra back-solve.
   - The ratio of two ordinary Newton steps is a different, Jacobian-dependent quantity, and Deuflhard's theory does not apply to it.
   - Θ is a property of the current iterate. It is not a certificate that "the start lies outside the convergence region".
   - KINSOL never computes Θ. PETSc `nleqerr` computes it internally and has no getter. So "free from library statistics" (§8) is wrong. B2 also shows KINSOL gives final totals only, and its line-search callback mixes trial and accepted points.
2. **The VI route needs an equation-to-variable pairing and a sign orientation (§2.1.3).**
   - PETSc's complementarity pairs Fᵢ with xᵢ: xᵢ = Lᵢ requires Fᵢ ≥ 0, and xᵢ = Uᵢ requires Fᵢ ≤ 0 (`SNESVINEWTONRSLS` manual page).
   - For a general flowsheet, row order and row sign are arbitrary. A VI point with active bounds is then an artefact of that pairing, not a model solution.
   - VI is warranted only when the model really is a complementarity system, such as phase disappearance written as an MCP. Otherwise an active-bound VI point is a non-solution, just like the least-squares point.
3. **"Ψtc is globally convergent" needs conditions the draft omits (§2.2.4).**
   - With M = I it needs −J(x*) to be stable, which again depends on the arbitrary pairing and sign of the equations.
   - Index-1 DAE structure needs the Coffey–Kelley–Keyes DAE variant.
   - It can only reach *stable* steady states. It cannot find an unstable middle CSTR branch.
4. **The probability-one guarantee covers only the fixed-point homotopy, under a boundary condition (§2.2.5).** The Newton homotopy is continuous Newton (Branin) and can turn back. Physical homotopies carry no guarantee. Domain restrictions (log, sqrt) break the C² on ℝⁿ assumption.
5. **The fixed-point selection rule is mathematically off (§3).**
   - Anderson with enough depth is GMRES-like and can converge when ρ(G′) > 1 (Walker & Ni 2011; Pollock & Rebholz 2021). So ρ̂ ≥ 1 does not rule Anderson out.
   - Oscillatory divergence (real eigenvalues < −1) is fixed by plain damping, which KINSOL already binds (`KINSetDamping`).
   - Anderson(m) *is* a multisecant type-II Broyden method (Fang & Saad 2009). Treating Anderson and Broyden as separate rungs keyed on ρ̂ has little basis.
   - The ratio estimate ρ̂ is unreliable early on: recycle Jacobians are non-normal, and a complex dominant pair makes the ratios oscillate.
6. **"Jacobian-free Newton on tears works with opaque causal maps" holds only conditionally (§3).**
   - G must be deterministic and free of hysteresis, independent of warm starts, and smooth along the probed directions.
   - Inner unit tolerances must be far below the difference increment. Jv accuracy is roughly √ε_inner, so with typical inner tolerances Newton degrades to linear convergence.
   - `KINSetRelErrFunc` is unbound (B2).
   - For n_tear ≲ 50, a dense finite-difference tear Jacobian plus Broyden updates is the classic, often cheaper choice, and its columns can be evaluated in parallel.
7. **The draft's "reduced space" is not reduced SQP (§4).**
   - Eliminating states through a converged square solve at every iterate is the *feasible-path* (nested, implicit-function) approach.
   - Biegler–Nocedal–Schmid 1995 rSQP is an *infeasible-path* null-space method.
   - Bounds on the eliminated variables can stay as outer inequalities with implicit-function gradients. "Bounds must be inactive" is too strong.
8. **The regularization row in §9 has the cause reversed.**
   - Ipopt's δw (primal regularization for wrong inertia) signals *nonconvexity*. LICQ failure shows up as δc, the constraint-Jacobian regularization (Wächter & Biegler 2006 §3.1).
   - The remedies differ. For δc: ℓ1/elastic, or `dependency_detector=mumps` (recalled; the linked MUMPS supports it, B1/B5a). For δw: scaling review or multistart, not SQP.
9. **The IDAS error-test row in §9 recommends the wrong things.**
   - Repeated error-test failures near one time point mean an *unlocated* discontinuity. The fix is to locate it with a root function and reinitialize.
   - Tighter tolerances make this worse.
   - ESDIRK helps only when discontinuities are frequent *and located*.
10. **Multi-experiment estimation is misattributed (§5).** parmest solves the extensive form (by default via mpi-sppy's ExtensiveForm), and DAKOTA is black-box; neither uses a Schur decomposition. The references are Zavala, Laird & Biegler 2008, Kang et al. 2014 (implicit Schur IPM) and Bock's condensing for multi-experiment Gauss–Newton. The draft's "to confirm in B2" for TAO BRGN is settled: BRGN exists with LM regularization and is unbound.
11. **Equation-level tearing is oversold (§2.4).**
   - Tearing is structural pivoting without numerical pivoting. The a ≠ 0 guard is necessary but not sufficient, because a small |a| amplifies error.
   - Intermediate r(t) leave their domains even when t is in bounds.
   - g(t) is usually more nonlinear and worse conditioned than the original block (Baharev, Schichl & Neumaier).
   - With a sparse direct LU, a smaller Newton dimension does not mean a cheaper factorization.
12. **The draft is stale against the evidence in three places.**
   - F01/C12 is *resolved* (A1). The draft says "check status".
   - F16 is closed for the KINSOL Krylov route (A2). The draft says "to be checked".
   - B5a shows argmin `BrentRoot` (already pinned), IBEX interval isolation (bound) and `SNESNEWTONAL` cover gaps the draft treats as open or as needing composition.

## 2. Claim-by-claim

Verdicts: C = correct; Cond = correct only under omitted conditions; O = overstated; W = wrong; M = missing an alternative.

| § | Claim (short) | Verdict | Correction or condition | Reference |
|---|---|---|---|---|
| 1 | The method depends more on structure than on the nominal class | C | — | Biegler 2010; BGW 1997 |
| 2.1 | Solving BTF blocks in sequence is exact | Cond | Exact in exact arithmetic. But tolerances propagate downstream, and an early 1×1 block with several roots can commit to a root that leaves a later block infeasible; the full simultaneous Newton would not commit. A structurally nonsingular block can be numerically singular at the point. | Duff & Reid; Westerberg 1979 |
| 2.1 | 1×1 blocks are more than 80 % of blocks | O | Uncited and unmeasured. Alias and linear elimination in presolve (pounce-presolve) shrinks this fraction. Mark it *Proposed*. | — |
| 2.1 | Affine 1×1: x = −b/a with an a ≠ 0 guard | Cond | Also needs a guard on \|a\| relative to the scales of b and x, not only a ≠ 0. | Higham 2002 |
| 2.1 | "Newton with bisection fallback" when no bracket exists | W | Bisection needs a sign change. Without a bracket, use a bound-projected, damped Newton plus a bracket search over the finite box, or IBEX interval isolation (bound, B5a), which also proves *absence* of a root. | B5a §5; Moore/Neumaier |
| 2.1 | Brent's convergence is guaranteed inside a bracket | Cond | Needs continuity. At a pole or discontinuity such as 1/x, it converges to the singularity. With several roots in the box it returns *a* root, not the physical branch. Symbolica's certified cubic roots (bound) are the right tool for EOS branches. | Brent 1973; B5a |
| 2.1 | Small blocks: dense dogleg (hybrj) or LM is "robust" when J is ill-conditioned | O | Both minimize ‖F‖², so both converge to non-root stationary points. Neither is affine invariant: both depend on row scaling. hybrj also uses Broyden updates. | Moré, Garbow & Hillstrom 1980; Dennis & Schnabel |
| 2.1 | Dense below about 50 | Cond | The crossover depends on block density and sparse symbolic overhead, and is uncalibrated. KINSOL dense is already bound. | — |
| 2.1 | An affine block needs one factorization | C | Add a residual check or iterative refinement, since the coefficient matrix is evaluated at upstream values. | — |
| 2.1 | Bound choices (i) VI, (ii) feasibility NLP, (iii) Coleman–Li are "different problems" | Cond/W | (ii)-least-squares and (iii) are the *same* problem with different algorithms. Ipopt with a zero objective is a third problem, whose failure mode is a local ℓ1 infeasibility minimizer from restoration, while ½‖F‖² gives an ℓ2 one. (i) depends on the pairing and sign (correction 2). | PETSc VINEWTONRSLS page; Benson & Munson 2006; Bellavia, Macconi & Morini 2003 |
| 2.1 | Two-sided boxes need one of (i)–(iii) | M | Usually the bounds are domain guards and the root is interior. Then a fraction-to-boundary step truncation, or a log/logit variable transform, is enough and cheaper. Library-owned alternatives exist for bounded least squares: TAO BNTR/TRON and BRGN over BNLS. Coleman–Li is in no pinned library. | Kanzow, Yamashita & Fukushima 2004; B5a |
| 2.1 | Θₖ = ‖Δxₖ₊₁‖/‖Δxₖ‖; Θ ≥ ½ means outside the convergence region | W | Use the simplified correction (correction 1). The theory relates Θ ≈ hₖ/2 to the Kantorovich quantity. Θ ≥ 1 means full-step contraction fails *at this iterate*. ½ is a heuristic. For damping, the governing test is restricted monotonicity, Θ(λ) < 1 − λ/4. | Deuflhard 2004 ch. 2–3 |
| 2.2.1 | The line search stalls at non-root minima of ‖F‖ | C | `KIN_STEP_LT_STPTOL` can also mean convergence (KINSOL guide), so check ‖F‖ before escalating. | KINSOL guide |
| 2.2.2 | NLEQ-ERR is invariant under row scaling | Cond | It is affine covariant under any F → AF, which is broader than row scaling. But it depends on *variable* scaling through its norm on Δx, so it needs the PS-07 nominals. It cannot be combined with VINEWTONRSLS, only with SSLS. | PETSc NLEQERR page; Deuflhard 2004 |
| 2.2 | The ladder runs from cheapest to most robust | O | Line search, NLEQ-ERR and trust region share the same failure mode: a stationary point of the merit function with F ≠ 0. They are not nested in robustness. The trust-region radius trigger needs the TR pre/post-check hooks, because no getter exists (B2). | Nocedal & Wright ch. 11 |
| 2.2.4 | Ψtc is globally convergent under dissipative flows | Cond | Correction 3. Kelley & Keyes 1998 assume the flow trajectory from x₀ converges to a stable x*. DAEs need the variant in Coffey, Kelley & Keyes (SISC 2003). Steady models without authored dynamics have no natural M. | Kelley & Keyes 1998 |
| 2.2.5 | Probability-one theory guarantees a path | Cond | Correction 4. Bifurcations on physical-parameter paths are not excluded. | Watson 1986/1997; Chow, Mallet-Paret & Yorke 1978 |
| 2.2.5 | Constructed homotopies need derived systems; arclength is in the code | M | The Newton homotopy fits PETSc `SNESNEWTONAL` (pinned) directly, as F̃(x) = F(x) − F(x₀) with constant load −F(x₀). The fixed-point homotopy needs an x-dependent load. The `SNESNewtonALSetFunction` callback receives U, but the documentation does not confirm an x-dependent load. | PETSc manual; B5a |
| 2.2.6 | The JᵀF = 0 stationarity conditions identify the inconsistent equations | O | It is the left null space of J at that point (Jᵀy = 0) that identifies *dependent combinations*. Large \|Fᵢ\| depends on row scaling. ℓ1 gives a sparse attribution, and Dulmage–Mendelsohn gives the structural attribution. A local minimum of ‖F‖ does not prove inconsistency. | Nocedal & Wright ch. 11; Duff & Reid (DM) |
| 2.3 | Eisenstat–Walker matters only with Krylov | O | Inexact-Newton theory also governs inexact *function* evaluations, such as nested unit solves in tear maps. The inner tolerance should track the outer forcing term. | Eisenstat & Walker 1996; Kelley 2003 |
| 2.3 | Anderson is equivalent to GMRES for linear contractions | Cond | Only with untruncated memory (m ≥ k), no stagnation, and up to one map application. It does not need contraction. Finite depth m has no equivalence. | Walker & Ni 2011 |
| 2.3 | Anderson needs a safeguard | C | Add the noise floor: inaccurate G caps the attainable accuracy. | Toth et al. 2017; Kelley 2018 |
| 2.3 | NPC over "BTF blocks or unit groupings" | O | Over triangular BTF blocks, forward substitution is already exact, so NASM adds nothing there. NASM and ASPIN matter *inside* an SCC, over a partition. ASPIN needs subdomain Jacobians, which opaque units lack. | Brune et al. 2015; Cai & Keyes 2002 |
| 2.4 | Tearing reduces the Newton dimension 5–20× and its exact derivatives come from the implicit-function theorem | O | Correction 11. The derivative statement is correct. The benefit is uncited. | Cellier & Kofman 2006; Baharev et al. |
| 3 | Successive substitution diverges when ρ ≥ 1 | Cond | Only undamped. Damped substitution converges when \|1 − β(1 − μ)\| < 1 for every eigenvalue μ, which covers any real μ < 1, including μ ≤ −1. | Westerberg 1979 |
| 3 | Bounded Wegstein with q ∈ [−5, 0] | Cond | q ≤ 0 only accelerates. It *cannot damp* oscillatory components (s < 0 needs q ∈ (0, 1)), which contradicts B2's "oscillation: bounded q". [−5, 0] is Pyomo SequentialDecomposition's default, so its only justification is IDAES parity (recalled). | Wegstein 1958; Pyomo docs (recall) |
| 3 | Wegstein as "≈ 50 lines over the KINSOL FP map" | W | Componentwise, history-dependent q makes the map stateful, which breaks KINSOL FP and Anderson assumptions, and β > 1 cannot be expressed (B2). It has to be an owned loop. | B2 §5 |
| 3 | Dominant-eigenvalue method removes the slow mode | Cond | Only for a single real dominant mode, well separated. It is a special case of vector extrapolation (MPE/RRE) and is largely subsumed by Anderson. | Orbach & Crowe 1971; Sidi |
| 3 | Anderson is "superlinear-like" | O | The proved results are r-linear with improved rate (Evans et al. 2020). It is a multisecant relative of Broyden. | Toth & Kelley 2015; Fang & Saad 2009 |
| 3 | Broyden is locally superlinear | C | Needs nonsingular I − G′ and a good B₀. With B₀ = I on x − G(x), it terminates in at most 2n steps on linear problems (Gay 1979). | Dennis & Moré 1977 |
| 3 | NGMRES costs 1 G evaluation | Cond | Only with `select none/difference`. The line-search variants cost more. | Oosterlee & Washio 2000 |
| 3 | JFNK works with opaque maps | Cond | Correction 6. Trial perturbations can also push units infeasible (negative flows), which needs one-sided differencing. | Kelley 2003 ch. 3 |
| 3 | Exact tear Newton: "one adjoint per unit" | W | Reverse mode needs one adjoint per *tear output* propagated through the chain, not one per unit. Per unit, the cost is min(inputs, outputs) back-solves after factorization. | Griewank & Walther |
| 3 | The ρ̂ thresholds | O | Correction 5. Anderson adds no G evaluations, so the real selectors are noise and unit-failure rate, not ρ̂ < 0.5. Use a geometric mean, as diffsol-nl does, plus the sign of Δxₖ₊₁·Δxₖ to detect oscillation. | Kelley 2018 |
| 3 | ρ̂ is "free" | Cond | It costs no extra evaluations, but nothing records it today. B2 says it needs a callback history. | B2 §1.4 |
| 3 | Nested is robust, simultaneous fast but fragile | O | Nested loops are robust only if the inner tears converge at every outer trial value. Inexact inner convergence corrupts the outer secant. In equation-oriented form, a specification is just a fixed/free variable swap. | BGW 1997 ch. 3–4 |
| 4 | IPM warm start needs centering | C | Missing the strongest IPM study warm start: a first-order sensitivity predictor (sIPOPT style; POUNCE `step_along_path`, which B1 shows is bound in the library). | Zavala & Biegler 2009 |
| 4 | SQP warm starts; Uno's switch at about 2000 | C | Matches B1. | B1 |
| 4 | HiGHS basis is the strongest warm start | C | `Highs_clearSolver` before each solve discards it (B4). | B4 |
| 4 | Exact Hessian versus L-BFGS | M | Finite-difference Hessian with CPR coloring (POUNCE, B1) for implicit providers is better than L-BFGS when the sparsity is known. | B1 §2.1 |
| 4 | Linear-solver ranking | Cond | MKL Pardiso inertia is unreliable under perturbed pivots, which disqualifies it for nonconvex inertia correction (B1). SSIDS GPU support is irrelevant without a CUDA build. Tasseff et al. 2019 found MA27 often the best serial choice. HSL is moot in this build. | B1; Tasseff, Coffrin, Wächter & Laird 2019 |
| 4 | Adaptive barrier is erratic on nonconvex problems | O | `obj-constr-filter` globalization falls back to monotone (Nocedal, Wächter & Waltz 2009). The real issue is that adaptive can *raise* μ and destroy a warm start, so warm-started sequences should use monotone with a small `mu_init`. | Nocedal, Wächter & Waltz 2009 |
| 4 | Reduced space is reduced SQP | W | Correction 7. | Biegler, Nocedal & Schmid 1995; Biegler 2010 ch. 7 |
| 4 | Schur decomposition for block-angular problems | Cond | Each block's KKT must be nonsingular, with inertia additive by Sylvester's law. The dense Schur complement must stay small. Only POUNCE offers it; Ipopt does not. | Zavala, Laird & Biegler 2008 |
| 4 | MPCC violates MFCQ everywhere; ℓ1 / Scholtes / FB | Cond | Regularization schemes can converge to C- or M-stationary (spurious) points. ℓ1/elastic reaches strongly stationary points under MPCC-LICQ. Missing: IDAES's smooth square flash (smooth max), which matters for parity. | Scheel & Scholtes 2000; Anitescu 2005; Ralph & Wright 2004; Burgard et al. 2019 |
| 4 | Branch-and-bound is not an automatic fallback | C | Nuance: for small factorable problems, budgeted SCIP is the only method that can *certify infeasibility*, which is the warranted response to "locally infeasible". SCIP's certificate is floating-point and tolerance-based, not rigorous. | B4 |
| 5 | Gauss–Newton is "quadratic-like" for small residuals | O | Quadratic only at zero residual. Otherwise linear with rate about ‖(JᵀJ)⁻¹S‖. | Nocedal & Wright §10.3 |
| 5 | Large residual needs full Hessian or NL2SOL | C | — | Dennis, Gay & Welsch 1981 |
| 5 | Data reconciliation uses NLP with a Gauss–Newton objective Hessian | Cond | Keep the constraint curvature λ∇²c unless the multipliers are small. Missing robust M-estimators for gross errors. | Bock; Narasimhan & Jordache |
| 5 | Schur "as in parmest and DAKOTA" | W | Correction 10. | Kang et al. 2014 |
| 5 | Identifiability diagnostics: SVD, collinearity index, FIM | Cond | All are local and, at a failed fit, describe the failed point. Missing: profile likelihood (practical identifiability) and generic-point rank (structural identifiability). | Brun et al. 2001; Raue et al. 2009 |
| 6 | Process dynamics are index-1 | Cond | High index arises from fixed-volume gas holdups, pressure–flow networks and output specifications. Missing: index reduction (Pantelides, dummy derivatives). | Brenan, Campbell & Petzold 1996; Mattsson & Söderlind 1993 |
| 6 | BDF stability region shrinks at high order | C | A(α): about 86°, 73° and 52° for orders 3, 4 and 5. | Hairer & Wanner IV.3 |
| 6 | ESDIRK is L-stable and better than BDF with many events | Cond | TR-BDF2 (γ = 2 − √2) is L-stable; check the Diffsol esdirk34 tableau. Stage order 2 causes order reduction on stiff and algebraic components. "Full order" is nominal order 2–3. Restarts still need a new h₀ and refactorization. | Hairer & Wanner IV.15; Kennedy & Carpenter 2016 |
| 6 | BDF must restart at order 1 after each event | Cond | Only for events that need `IDAReInit`: state resets or right-hand-side switches. A plain root return continues at the current order. The crossover is about ≲ 10–20 steps per segment (B3), so normalize the event count by horizon/h. | B3 §1.1, §5.1 |
| 6 | Forward if p is small relative to q, adjoint if q ≪ p | Cond | Compare p·c_fwd with q·c_adj. Locally c_adj includes replay (B3). Gauss–Newton needs the full residual Jacobian, which favours forward. Local adjoints refuse events (B3). Second order needs forward-over-adjoint. | B3 §5.3; Cao, Li, Petzold & Serban 2003 |
| 6 | Initialization failures are diagnosable by algebraic residuals at t₀ | O | The dominant failure is a singular ∂g/∂z, meaning index > 1 at t₀. Use a rank or Dulmage–Mendelsohn check. Missing: solving the initialization system with the square-system ladder, as IDAES practice does, and reinitializing after discontinuities. | B3; BCP 1996 |
| 6 | Collocation, multiple shooting and single shooting trade-offs | Cond | Collocation cannot take authored events and its mesh is fixed (B3), so event presence favours shooting. Multiple shooting needs sensitivities at every node. | Biegler 2010 ch. 10; Bock & Plitt 1984 |
| 7.1 | Curtis–Reid and the other scaling formulas | C | Ipopt `gradient-based` only scales down. | Curtis & Reid 1972 |
| 7.3 | JVP/HVP from forward mode | Cond | HVP needs forward-over-reverse or a second-order directional derivative. A2 finds no HVP exists. | A2 |
| 8 | Convexity certificate has Structure + bound-snapshot lifetime | W | Convexity of c·x² depends on the sign of c, which is a parameter value, so the lifetime is Values. | — |
| 8 | Convexity predicts interior point versus active set | W | It predicts that local equals global and that convex solvers are eligible. Interior point versus active set depends on size and warm-start needs. | — |
| 8 | ‖F(x₀)‖ predicts the need for globalization | O | It is not affine invariant; this is the very reason Deuflhard rejects residual monitors. Better: the scaled first Newton correction ‖D⁻¹Δx₀‖ and α_max to the box. | Deuflhard 2004 |
| 8 | Condition estimate is "free" | O | KLU `rcond` is the crude min/max\|U_kk\| ratio. `condest` costs a few extra solves. Both must be computed on the *scaled* J. Neither is exported today (B2). | B2 §4 |
| 9 | All rows | see §3 of this review | — | — |

## 3. Missing methods, alternatives and predictors

**Methods and alternatives**
- **Deflation** (Farrell, Birkisson & Funke 2015). It finds additional roots after the first and suits wrong-branch or nonphysical roots in phase and EOS blocks. It is a small composition over any Newton method.
- **NLEQ2 rank-reduction Newton** (Deuflhard), for near-singular J at phase boundaries (B5a).
- **IBEX interval root isolation and Symbolica certified cubic roots** for 1×1 blocks. Both are already bound. They certify the existence, absence and branch of roots, which Brent cannot.
- **Fraction-to-boundary step truncation and log/logit transforms** for domain-guard bounds, before reaching for VI or NLP.
- **Ipopt `dependency_detector=mumps`** (recalled) for LICQ failure from redundant equalities.
- **Irreducible infeasible subset by a deletion filter** (Chinneck) over feasibility solves. It gives a minimal set of inconsistent equations, which is sharper than reading ℓ1 slacks.
- **Extracting Ipopt's restoration slacks p and n.** Restoration *is* an ℓ1-elastic problem with a proximal term (Wächter & Biegler 2006 §3.3). Re-running ℓ1 elastic from the same point often lands at the same place. The attribution is the value, and today it is not exposed.
- **Sensitivity-based predictor** (sIPOPT / POUNCE `step_along_path`) for studies, together with natural-parameter continuation of a converged point.
- **Tear-set choice affects ρ** (Upadhye & Grens 1975). A2 records that the default tears are unweighted, so tear selection should weigh convergence, not only the tear count.
- **Inside-out and two-tier methods** (Boston & Britt) for property-dominated units in sequential-modular mode.
- **Rosenbrock and Radau IIA integrators.** Rosenbrock (PETSc TSROSW, already reachable) is one-step and linearly implicit, which suits event-dense index-1 systems. Radau IIA is stiffly accurate with high stage order, Hairer & Wanner's reference for DAEs, and in no pinned library.
- **Index detection and reduction** (Pantelides, Σ-method, dummy derivatives) before IDA, with `IDASetSuppressAlg` for poorly scaled algebraic variables.
- **Profile likelihood**, and **robust-loss** least squares (Huber, Cauchy) for gross errors.

**Predictors absent from §8**
- **Structural rank and the Dulmage–Mendelsohn over/under-determined parts.** Structure lifetime, already computed by presolve. They predict that *every* method fails, which is a stronger signal than any rung choice.
- **Numerical rank at x₀:** KLU `numerical_rank` and `singular_col`, FERAL `n_tiny`. Cheap, and currently reduced to a message (B2).
- **Nonsmooth operators** (abs, min, max, piecewise correlations, regime switches). They predict failure of Newton-type methods and JFNK, and call for smoothing or a semismooth method.
- **Domain-restricted operators and x₀'s distance to their domain boundary.** They predict NaN or domain trials.
- **α_max**, the largest step fraction before the first Newton step leaves the box. It predicts whether bounded methods are needed better than "bound density" does.
- **Jacobian-to-residual evaluation cost ratio.** Already in `CallbackState.seconds` (B2). It predicts modified Newton or quasi-Newton.
- **Inner tolerance and noise level of G** for tears. It predicts whether JFNK and Anderson are viable.
- **Unit failure rate on tear trials.**
- **A History lifetime:** the previous outcome and winning method for the same structure, and the parameter step ‖Δp‖·‖dx/dp‖ from the last converged point. It is the strongest empirical predictor along studies and needs its own lifetime between Values and Start.
- **Steps per segment between discontinuities**, rather than a raw event count.
- **Kantorovich-type first-step quantities**, such as Θ₀ from the simplified correction, in place of ‖F(x₀)‖.

**§9: corrections and missing rows**

Corrections to existing rows:
- **Row 1 (local minimum of ‖F‖):** trust region and LM converge to the same stationary point. "Least squares to diagnose" from that point adds only confirmation; extract left null vectors instead. Prefer the fixed-point homotopy or Ψtc over the Newton homotopy, whose path passes through singular J.
- **Row 2 (ill-conditioned J):** first separate scaling from genuine near-singularity by computing the scaled condition estimate. Rescaling is the remedy for the first.
- **Row 3 (repeated truncation at bounds):** split into three causes:
  - a root outside the box: bounded least squares and an infeasibility report;
  - a domain guard with an interior root: interior or fraction-to-boundary steps;
  - a genuine complementarity: VI.
- **Row 4 (ρ̂ ≥ 1 or oscillation):** add damping for oscillation (`KINSetDamping`, bound) and deeper Anderson before Broyden or Newton.

Missing rows:
- **Newton converging only linearly with constant Θ.** This points to a wrong or inexact Jacobian or a singular root. The remedy is a derivative check (`derivative_test`, `SNESTestJacobian`), not globalization.
- **Repeated domain or NaN trials.** The remedy is FBBT tightening and step truncation, not another globalization rung.
- **Converged to a nonphysical root.** The remedy is deflation, homotopy or a different start.

## 4. Places that re-implement library machinery

- **Wegstein and dominant-eigenvalue extrapolation (§3)** would be an owned iteration loop. They cannot ride on KINSOL FP (correction in §2 of this review). Anderson (bound, default depth 0) subsumes both mathematically. Justify Wegstein only by IDAES parity, if that is a requirement, and say so.
- **Computing Θₖ (§2.1, §8)** outside PETSc would re-implement the NLEQ-ERR monitor. Select `nleqerr` and observe it through `SNESMonitorSet` or `SNESSetConvergenceHistory`; neither is bound yet (B2). Do not reconstruct Θ from KINSOL.
- **A dense dogleg or LM for small blocks (§2.1)** is not needed. KINSOL dense with line search is bound, and PETSc `NEWTONTR`/`NEWTONTRDC` covers dogleg. MINPACK and the `levenberg-marquardt` crate were excluded as duplicates (B5a), with a revisit trigger only if PETSc setup overhead dominates tiny blocks.
- **A scalar safeguarded Newton–bisection (§2.1)** would be bespoke. argmin `BrentRoot` (pinned) handles brackets; IBEX handles isolation and absence (B5a).
- **Coleman–Li (§2.1, iii)** is in no pinned library. Use TAO BNTR, TRON or BRGN instead.
- **Constructed homotopies over local arclength code (§2.2.5).** Fit the Newton homotopy to `SNESNEWTONAL` before extending `kkt/path/arclength.rs`. The prediction already has a library owner: POUNCE `step_along_path`.
- **"A dedicated LM route is absent" (§5).** It is TAO BRGN-lm in the pinned PETSc: a binding task, not a method to write.
- **Curtis–Reid and Ruiz scaling (§7.1)** are acceptable only as diagnostics. As a remedy, the libraries own scaling: MC64 inside FERAL and POUNCE, Ipopt `gradient-based`, Ruiz inside Clarabel.
- **Equation-level tearing (§2.4)** is a bespoke structural transform that also takes on numerical-stability risk. Inside SCCs, NASM (bound) or the existing FERAL BTF-block preconditioner should be compared with it first.
- **Ψtc (§2.2.4)** is correctly library-owned through `TSPSEUDO`. That binding is explicit-only today (A1), and the draft should not imply that a constructed mass matrix is a library concern.