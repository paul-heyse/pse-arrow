# Mechanism contract cards: local, predictive and globalization mechanisms (M1–M8)

Evidence worker output for the solver acceleration and globalization review (2026-10-03).
Scope: M1 parametric NLP sensitivity; M1b generalized / active-set-changing sensitivity and
path following; M2 advanced-step, real-time iteration, neighboring extremal; M3 tangent and
secant predictors for square roots; M4 inexact Newton, Newton–Krylov and JFNK; M5 modified
Newton, Jacobian and preconditioner lagging, Broyden; M6 Anderson acceleration and fixed-point
admission; M7 trust-region and Levenberg–Marquardt local models, SLP/SQP-filter auxiliary
models; M8 Ipopt's globalization as a production algorithm.

**Framing.** All pertinent mechanisms are deployed. Admission is decided from mathematical
facts known before solving and from observations made during the solve, not from
benchmarks. A mechanism's success means only that its intermediate result is usable; final
acceptance is always against the original problem in its original coordinates.

**Evidence labels.**
- `[read]` means I read the paper text (open copy), the official documentation or the pinned
  library source; the location is named.
- `[abstract]` means I relied on the abstract, the publisher page or a restatement in another
  paper I read.
- `[known]` means standard results from memory, not re-read for this note.
- `[interp]` means my own interpretation or design inference.

Library source citations refer to the native-solver-libraries skill corpus
(`content/corpus/<crate>@<version>/…`). Repository observations come from read-only `grep` of
`crates/pse-backend-native/src` at the review baseline.

Card facets: 1 prerequisites · 2 construction · 3 retained state · 4 step/globalization rule ·
5 acceptance/validity · 6 refusal and abandonment (facts / in-solve observations) · 7 scaling ·
8 cost · 9 interactions · 10 library realizations and minimum exposed information ·
11 references and evidence note.

---

## M1 Parametric NLP sensitivity prediction (Fiacco / Büskens–Maurer / sIPOPT)

**1. Prerequisites.**
- f and c must be k+1 times differentiable in x and k times in p, with C² in x as the minimum.
  At the base KKT point s* = (x*, λ*, ν*) at p0, three conditions must hold: LICQ, the strong
  second-order sufficient condition (SSOSC) and strict complementarity (SC). Under them, s* is
  an isolated minimizer with unique multipliers, and for p near p0 there is a C^k solution map
  s(p) whose binding set is unchanged (Fiacco, Thm 3.2.2; sIPOPT Property 2)
  `[read: sIPOPT §2.1]`.
- An interior-point factor is taken at μ > 0. The barrier solution s(μ; p) is differentiable
  in p, and the predictor error is o(‖Δp‖) + o(μ) (sIPOPT eqs. 9–10) `[read]`.
- For the advanced-step use, Biegler states ‖s̃(p) − s*(p)‖ ≤ L_s‖Δp‖² (DYCOPS 2013 survey,
  eq. 10) `[read]`.
- The KKT matrix at the accepted point must have the correct inertia with no Hessian
  regularization. sIPOPT notes that these assumptions "can be checked by the inertia of M"
  `[read]`. POUNCE's port exposes `sens_max_pdpert` (default 1e-3), the maximum primal-dual
  perturbation it accepts in a sensitivity step
  `[read: pounce-sens-core@0.12.0/src/sens_app.rs]`.
- The parameter must be *localized*: it becomes a variable w fixed by the row w − p0 = 0, so
  that Δp enters only the right-hand side (sIPOPT eq. 11–13) `[read]`.
- When the KKT system is linear in p (a QP with linear constraints and parameter-affine data),
  an admitted prediction is exact while the active set holds `[known]`. pse-arrow's
  `kkt/advance.rs` states the same.

**2. Construction.**
- M(s*)·ds/dp = −N_p, so Δs = −M⁻¹(N_p Δp), or with the μ→0 correction
  Δs = −M⁻¹(N_p Δp + N_μ) (sIPOPT eqs. 7–10) `[read]`.
- With localized parameters the RHS is the pinned row e_pin·Δp. One backsolve serves any Δp,
  and n_pert perturbations cost n_pert backsolves (sIPOPT §2.7) `[read]`.
- Fix-relax handles predicted active-set changes (sIPOPT §2.4, eqs. 16–21) `[read]`:
  - pin a variable that the step carries past a bound (E_xᵀΔx + x_i* = 0, relaxing its
    complementarity row);
  - zero a bound multiplier that the step drives negative (E_νᵀΔν + ν_i* = 0).
  - Both cases are solved with the Schur complement C = −EᵀK⁻¹E,
    C Δν̄ = EᵀK⁻¹r_s − r₁, K Δs = −(r_s + E Δν̄).
- sIPOPT itself does not implement the QP form of active-set-changing sensitivity (§2.4); see
  M1b `[read]`.
- Büskens–Maurer add Taylor-based "real-time approximations" of perturbed solutions
  `[abstract]`.

**3. Retained state.**
- The factored KKT matrix exactly as used at the accepted point: the same μ, scaling, δ_w = 0
  and δ_c.
- The full primal-dual iterate, including slacks and bound multipliers.
- Bounds and their relaxation (`bound_relax_factor`).
- The parameter identity map: which variables or rows are parameters, and their units.
- Solver-to-natural unit factors. POUNCE's `SensBacksolver::natural_units_factor` exists
  because the factor works in scaled coordinates `[read]`.
- Activity classification (strongly active, weakly active, inactive) with its margins.
- A structure stamp: variable and row identity, sparsity and ordering.

**4. Step / globalization rule.**
- The full first-order step; there is no line search. Options:
  - clip the step length along Δs to stay within bounds (Biegler 2013 survey; Yang–Biegler)
    `[read]`;
  - apply fix-relax;
  - split Δp into segments, which turns M1 into M1b path following.

**5. Acceptance / validity.**
- (a) *Screen.* Check that the predicted active set is unchanged: no multiplier sign changes
  and no bound or row crossing beyond a margin. Inactive rows are tested on their
  linearization. This screen is not a certificate `[read: advance.rs doc; sIPOPT §2.3]`.
- (b) *Original evaluation at the new p.* Measure primal infeasibility, dual infeasibility and
  complementarity in original coordinates.
- (c) The prediction is a final result only if (b) meets the original final tolerances, which
  is exact only in the QP case. Otherwise it is a seed for a corrector: one Newton/SQP step or
  a warm-started full solve `[interp]`.
- Success of the prediction means "a valid initial iterate", never "solved".

**6. Refusal / abandonment.**
- *Facts that refuse the mechanism:*
  - no retained factor, or a factor whose structure, scaling or parameter-set stamp differs;
  - a model that is not C² between p0 and p (selectors, min/max, discrete decisions);
  - inertia correction active at the solution (δ_w > threshold) or δ_c > 0, which means SSOSC
    or LICQ is doubtful;
  - weakly active constraints at the base (s_i and z_i both small, of order √μ), which means SC
    fails; route to M1b;
  - a parameter change that alters which variables are fixed or free.
- *Prediction-time observations that trigger fix-relax, M1b, or abandonment to a full solve:*
  - the number of predicted active-set changes;
  - Schur complement conditioning or failure;
  - the POUNCE refinement stop reason (`RefineStop::IterationLimit`, `WorseThanPlain`)
    `[read: boundcheck.rs]`;
  - the scaled norm of the predicted step relative to variable scales;
  - the KKT residual at the predicted point versus the KKT residual of the *unpredicted* base
    point evaluated at the new p. Comparing them takes two evaluations and shows whether the
    prediction beats a plain warm start `[interp]`.
- *Corrector-time observations that shrink the radius used for future predictions:*
  corrector iterations, restoration entry, and a change of status `[interp]`.

**7. Scaling / conditioning.**
- Predict in the solver's scaled space and map back with the unit factors.
- Keep the primal activity margin and the dual release threshold as two separate numbers.
  POUNCE documents that a single `bound_eps` of 1e-2 suppressed every release on a model with
  multipliers of order 1e-3 `[read: boundcheck.rs]`.
- Active bounds carry σ = z/s ≈ 1/μ in the held factor. Computing a release from the held
  factor therefore gets worse the tighter the solve converged: 2e-4 off at tol = 1e-10 against
  7e-9 at 1e-6. POUNCE refactors with that σ dropped `[read: boundcheck.rs]`.

**8. Cost (qualitative).**
- One sparse backsolve per perturbation (n_p backsolves for the full sensitivity matrix).
- One evaluation of residuals and derivatives at the new p, for screening and acceptance.
- Fix-relax: per pass one dense k×k solve plus k+1 backsolves, with no refactor for pins and
  a refactor for releases (POUNCE) `[read]`.
- A full solve needs tens of factorizations and evaluations `[known]`.

**9. Interactions.**
- Depends on M8 or the POUNCE IPM for the base solve and the retained factor.
- Feeds a seed of primal values and duals, with a working set only for active-set consumers,
  into a warm start and then a corrector (M8 with `warm_start_init_point`, or POUNCE SQP).
- Composes with:
  - M1b, when SC fails or Δp is large;
  - M2, since advanced-step is M1 plus a solve-ahead;
  - M3, its root analogue;
  - continuation, as the stage predictor.
- Generalizes to parameter studies ordered by proximity, to continuation stages and to
  sequential estimation `[interp]`.

**10. Library realizations; minimum information.**
- sIPOPT (Ipopt `contrib/sIPOPT`, C++; needs the exact Hessian) `[read: ipopt@3.14.20 corpus]`.
- POUNCE `pounce-sens-core` 0.12.0 provides `SensApplication`, the `SensBacksolver` trait
  (`dim`, `solve`), `IndexSchurData`, `StdStepCalc`, and the options `n_sens_steps`,
  `sens_boundcheck`, `sens_bound_eps` and `sens_max_pdpert` `[read]`.
- pse-arrow already uses `SensApplication`, `IndexSchurData` and `SensOptions` in
  `kkt/advance.rs` and `kkt/sensitivity.rs` `[read]`.
- *Minimum information the pipeline must expose:*
  - parameter identity and Δp in solver coordinates;
  - the retained factor handle with its stamp (μ, δ_w, δ_c, scaling, structure);
  - the primal-dual iterate, bounds and activity margins;
  - an original-problem evaluator at the new p.

**11. References / evidence.**
- Pirnay, López-Negrete, Biegler (2012), *Math. Prog. Comp.* 4:307–331,
  doi:10.1007/s12532-012-0043-2 — `[read]`, optimization-online preprint 3008 (§1–3).
- Fiacco (1976) *Math. Prog.* 10:287–311 and Fiacco (1983, book) — `[abstract]`, via the
  restatements in sIPOPT and Suwartadi.
- Büskens & Maurer (2001), in *Online Optimization of Large Scale Systems*, Springer,
  doi:10.1007/978-3-662-04331-8_1 — `[abstract]`.
- Robinson (1980) *Math. Oper. Res.* 5:43–62, doi:10.1287/moor.5.1.43 — `[abstract]`.
- Biegler (2013), "A survey on sensitivity-based NMPC", IFAC DYCOPS — `[read]`.
- POUNCE source — `[read]`.

---

## M1b Generalized (active-set-changing) sensitivity and parametric path following

**1. Prerequisites.**
- C² data with LICQ and SSOSC at the base. Strict complementarity is **not** required.
  - Under these conditions the primal-dual solution map is PC¹, locally Lipschitz and
    directionally differentiable.
  - Its directional derivative uniquely solves a QP over the critical cone (Scholtes'
    specialization of Ralph–Dempe; Stechlinski–Jäschke–Barton Thm 1.1; Suwartadi Thm 2)
    `[read]`.
  - Ralph–Dempe assume MFCQ, CRCQ and general SSOSC `[read: SJB intro]`.
- Kyparsis-type conditions give the weakest setting for a unique directional derivative:
  MFCQ, SSOSC for all multipliers, and constant rank (sIPOPT Property 4) `[read]`.
- Robinson strong regularity of the KKT generalized equation gives a locally unique Lipschitz
  solution map. Under LICQ it is equivalent to SSOSC (Kojima) `[known]`.
- For path following, strong regularity must hold at every point of the path. Under that
  assumption the solution set decomposes into finitely many non-intersecting Lipschitz
  branches, and an Euler–Newton tracker reaches O(h⁴) accuracy (Dontchev–Krastanov–
  Rockafellar–Veliov 2013) `[abstract]`.
- Stechlinski–Khan–Barton (2018): LICQ + SSOSC give lexicographic directional derivatives and
  B-subdifferential elements of the solution map `[abstract]`. These are needed when a
  *generalized Jacobian* is consumed (nonsmooth Newton, outer optimization through an NLP).
  A single prediction along a known Δp needs only the directional derivative `[interp]`.

**2. Construction.**
- (a) *Pure predictor QP:*
  min ½ΔxᵀW Δx + Δxᵀ∇²_{xp}L Δp, subject to the linearized equalities. Strongly active
  inequalities (K₊) are equalities; weakly active ones (K₀) are ≤ (Suwartadi eq. 10; SJB
  QP⁽¹⁾(d)) `[read]`.
- (b) *Predictor–corrector QP:* add the constraint residuals and ∇F at the new parameter.
  Strongly active rows stay equalities; all other rows are linearized inequalities. The QP
  multipliers replace the duals (Suwartadi eqs. 11–15; Kungurtsev–Diehl 2014) `[read / abstract]`.
- (c) *Piecewise-linear path on a fixed factor* (POUNCE `step_along_path`) `[read]`:
  - apply the perturbation up to the first breakpoint;
  - the three ratio tests are an inactive variable reaching a bound (hold it with a Schur
    row), a base-active bound multiplier reaching zero (release), and a hold's multiplier
    crossing zero (drop);
  - continue with the remainder under the new set.
  - This is exact for QPs (piecewise affine) and remains a predictor for NLPs, because
    nothing is re-linearized.
- (d) *Directional derivative at a kink* where a bound is weakly active: POUNCE
  `path_direction(_with)` `[read]`.
- (e) *Homotopy in t:* p(t) = (1−t)p0 + t·p_f, with one QP per stage (Suwartadi Alg. 2) `[read]`.
- (f) *Stepwise sIPOPT:* sum the QP solutions over segments split at the active-set change
  (sIPOPT §2.3) `[read]`.

**3. Retained state.**
- Everything in M1, plus W and the Jacobians at the base (or the KKT factor).
- Strong, weak and inactive classification with margins, including POUNCE's "weak rows".
- The QP working set, which can warm-start an active-set QP.
- For (b): fresh constraint residuals at each stage, and optionally fresh Jacobians.

**4. Step / globalization rule.**
- Suwartadi uses a fixed Δt; if the QP is infeasible, Δt ← α₁Δt and the step is retried
  `[read]`.
- Breakpoint ratio tests give exact segment lengths. A maximum segment count applies; when it
  is reached, POUNCE takes the remainder in one step and the returned count shows that it
  happened `[read]`.
- An adaptive Δt driven by corrector contraction (M3) is `[interp]`.

**5. Acceptance / validity.**
- The QP must be solved: it is feasible, and convex on the critical cone because of SSOSC.
- Evaluate the predicted point against the original problem at p_f. It is final only if the
  original tolerances hold, otherwise it seeds the corrector.
- POUNCE refuses refinements that end further outside the bounds than the plain step, and
  returns the plain step instead `[read]`.

**6. Refusal / abandonment.**
- *Facts:*
  - LICQ fails (degenerate multipliers). Jäschke–Yang–Biegler handle non-unique multipliers
    with an extra LP `[abstract, via Suwartadi]`; otherwise refuse.
  - SSOSC is doubtful (inertia correction at the base).
  - The functions are nonsmooth beyond the inequality-induced kinks, or integers are present.
- *Observations:*
  - the stage QP is infeasible repeatedly until Δt < Δt_min;
  - the reduced Hessian loses positive definiteness along the path;
  - the breakpoint count exceeds the budget;
  - the Schur pin residual is above 1e-11 on both operators (POUNCE chooses between plain and
    regularized operators by pin residual, because a singular plain operator can return "Ok")
    `[read]`;
  - the corrector residual does not decrease between stages;
  - the working-set Jacobian becomes rank-deficient.

**7. Scaling / conditioning.** As in M1 (two margins, σ magnitudes). The critical-cone QP
Hessian may be **indefinite in full space** while being positive definite on the cone
`[read: Suwartadi §3.1]`.

**8. Cost.**
- (a): one QP solve on retained data, warm-started from the base working set.
- (c): per segment, a k×k Schur solve plus backsolves, and a refactor per release `[read]`.
- (b)/(e): N QPs plus N residual (and possibly derivative) evaluations. This is still well
  below N full NLP solves; with fresh derivatives every stage it approaches SQP `[interp]`.

**9. Interactions.**
- M1b extends M1, and its predictor–corrector QP is close to the RTI QP (M2), with the
  difference that strongly active rows are equalities `[read: Suwartadi §3.3]`.
- POUNCE active-set SQP can act as the corrector, warm-started with a `WorkingSet` built from an
  IPM iterate using a multiplier-sign and primal-distance heuristic `[read: sqp/warm_start.rs]`.
- It feeds a seed with an optional working set.

**10. Library realizations; minimum information.**
- POUNCE `pounce-sens-core` 0.12.0, pinned in pse-arrow `[read]`:
  - `refine_step_onto_bounds` — fix-relax, both halves;
  - `step_along_path` — piecewise-linear path through breakpoints;
  - `path_direction` / `path_direction_with` — directional derivative at a kink;
  - `RefineStop`, `PathSegment`, `release_floor`.
- **pse-arrow does not call any of these.** A grep over `crates/` for `refine_step_onto_bounds`,
  `step_along_path` and `path_direction` finds no matches.
- POUNCE's engine-coupled corrector lives in `pounce-sensitivity`, which is not pinned.
- `pounce-qp` (parametric active-set QP with Schur updates) is reachable through
  `pounce-rs` feature `qp`.
- HiGHS QP and Clarabel require a positive semidefinite Hessian, so they are **not general
  solvers for the critical-cone QP**. They work only when W is PSD on the full space
  `[known/interp]`.
- *Minimum information:* everything in M1, plus strong and weak margins, W and the Jacobians
  (or the factor), the parameter path, and a QP solver that accepts a reduced-PD Hessian.

**11. References / evidence.**
- Stechlinski, Khan, Barton (2018) *SIAM J. Optim.* 28:272–301, doi:10.1137/17M1120385 —
  `[abstract]`; the MIT open-copy download returned HTML.
- Stechlinski, Jäschke, Barton, "Generalized sensitivity analysis of NLPs using a sequence of
  QPs" (NTNU preprint) — `[read]`.
- Suwartadi, Kungurtsev, Jäschke (2017) *Processes* 5:8, doi:10.3390/pr5010008 — `[read]`,
  LAPSE copy, §3.
- Kungurtsev & Diehl (2014) *Comput. Optim. Appl.* 59:475–509, doi:10.1007/s10589-014-9696-2
  — `[abstract]`. The abstract notes that standard SQP globalization can block warm-start
  benefits.
- Dontchev & Rockafellar, *Implicit Functions and Solution Mappings* (2nd ed. 2014),
  doi:10.1007/978-1-4939-1037-3 — `[known]`.
- Dontchev, Krastanov, Rockafellar, Veliov (2013) *SIAM J. Control Optim.* 51:1823–1840 —
  `[abstract]`.
- POUNCE `boundcheck.rs` — `[read]`.

---

## M2 Advanced-step NMPC, real-time iteration and neighboring-extremal updates

**1. Prerequisites.**
- *Advanced step (asNMPC/asMHE).*
  - The NLP at an anticipated parameter p̂ can be fully solved before the actual p is known.
  - M1's conditions hold at that solution: LICQ, SSOSC, and SC (or M1b handles its absence).
  - The error is ‖s̃ − s*‖ ≤ L_s‖p − p̂‖² `[read: Biegler 2013 survey §3–4]`.
  - Nominal stability equals that of ideal NMPC, and robustness holds in the ISS sense
    (Zavala–Biegler 2009) `[abstract]`. These are control-loop properties, not per-solve
    accuracy guarantees `[interp]`.
- *Real-time iteration (RTI)* (Diehl–Bock–Schlöder, Thm 4.1) `[read]`:
  - L_k is C², and the second-derivative approximations J^k are continuous with bounded
    inverse;
  - a κ < 1 exists with ‖J(y′)⁻¹(J(y+tΔy) − ∇²L(y+tΔy))Δy‖ ≤ κ‖Δy‖ (4.1a), measuring how
    good the Jacobian approximation is;
  - an ω < ∞ bounds the Lipschitz-type variation (4.1b–c);
  - the first step is small: δ₀ = κ + (ω/2)‖Δy⁰‖ < 1;
  - the ball of radius ‖Δy⁰‖/(1−δ₀) lies in the domain.
  - Under these conditions the iterates contract with ‖Δy^{k+1}‖ ≤ (κ + ω/2‖Δy^k‖)‖Δy^k‖ and
    approach the moving optima, with a bound on the loss of optimality.
  - The parameter enters through an *initial-value embedding*: an extra variable s_k with
    the constraint s_k = x_k, so the linearization is independent of x_k.
- *Neighboring-extremal updates (NEU).* Second-order sensitivity information around a nominal
  solution, plus an a-priori error estimate of the deviation from the optimal trajectory. The
  estimate is used to choose how many QP iterations to run per horizon (Würth–Hannemann–
  Marquardt 2009) `[abstract]`.

**2. Construction.**
- *asNMPC.* In the background, solve NLP(p̂) and hold K*. Online, compute
  s̃ = s*(p̂) − K*⁻¹φ(s*(p̂), p), where φ is the KKT residual; this is one Newton step from the
  old solution toward the new problem (survey eq. 9) `[read]`.
  - Active-set changes are handled by step clipping or by a QP extension `[read]`.
  - amsNMPC predicts N_samp steps ahead with an augmented sensitivity system `[read: survey §6]`.
- *RTI.* Each sample runs three phases: preparation (evaluate ∇L and J at y_k and prepare the
  factorization without x_k), feedback (complete Δy = −J⁻¹∇L once x_k is known) and
  transition (shift onto the next problem with Π_{k+1}). Exactly one Newton-type iteration is
  done per problem `[read]`.
- *Path-following asNMPC* (Jäschke–Yang–Biegler 2014; Suwartadi 2017) is M1b applied online
  `[read]`.

**3. Retained state.**
- asNMPC: the KKT factor and primal-dual solution at p̂, and the parameter map.
- RTI: the pre-assembled linearization and factor at y_k, plus the shift operator.
- NEU: second-order sensitivity quantities along the nominal solution, plus the error-model
  constants.

**4. Step / globalization rule.**
- asNMPC: a full sensitivity step, optionally clipped (a fraction of Δs that keeps bounds) or
  QP-based.
- RTI: a full step with **no globalization**; it relies on the contraction in (4.1).
- NEU: a limited number of QP iterations, chosen from the error estimate.

**5. Acceptance / validity.**
- In control, the approximate answer is used as the control move and is not checked against
  the exact problem. **That acceptance rule does not transfer to a simulator's final results**
  `[interp]`.
- Transferable meaning:
  - the advanced-step output is a seed;
  - the RTI idea (one corrector iteration per nearby problem) is legitimate only for
    *intermediate* problems, such as continuation stages, homotopy steps or ordered sweeps,
    whose results are not published. The final target still gets a fully converged solve
    and the original-problem assessment `[interp]`.

**6. Refusal / abandonment.**
- *Facts:*
  - no background solution at the same structure;
  - the M1 prerequisites fail;
  - for RTI, the problems are not ordered so that they change slowly, or nothing bounds the
    Jacobian-approximation quality.
- *Observations:*
  - the KKT residual at s̃ for the actual p;
  - a predicted active-set change, which routes to clipping or a QP;
  - for RTI, the contraction ratio ‖Δy^{k+1}‖/‖Δy^k‖. If it reaches or exceeds 1, (4.1d) has
    failed and the scheme must fall back to a full solve to get back "on track"
    `[read: Suwartadi §3.3; interp]`;
  - for NEU, the error estimate exceeding a threshold, which adds QP iterations or triggers
    re-optimization `[abstract]`.

**7. Scaling.**
- κ and ω are norm-dependent. Work in the scaled coordinates in which the factor was built.

**8. Cost.**
- asNMPC: online cost is one backsolve. The background solve is still a full NLP.
  **Advanced step reduces latency, not total work**, unless one background solve serves
  several later cases or runs in parallel `[interp]`.
- RTI: one QP (one factorization) per problem. It does reduce total work, because
  intermediate problems are never converged.

**9. Interactions.**
- M1 and M1b are the online part; M8 or POUNCE is the background solver.
- RTI corresponds to Euler–Newton continuation with one corrector step per stage (see M3 and
  the continuation cards).
- amsNMPC is multi-step anticipation, analogous to predicting several study points from one
  factor.

**10. Library realizations; minimum information.**
- asNMPC online step: sIPOPT, POUNCE `SensApplication`, and pse-arrow `Advance` (rolling-horizon
  controller) `[read]`.
- RTI: acados / ACADO (with HPIPM or qpOASES) are the reference toolkits, but none is in the
  stack `[known]`. Composition is possible from derivative programs plus POUNCE active-set QP;
  HiGHS QP works only for a convex QP.
- NEU: bespoke.
- *Minimum information:*
  - a retained factor or linearization stamped by parameter and structure;
  - a cheap KKT-residual evaluation at the new parameter;
  - a declared ordering or proximity between successive problems;
  - a per-problem flag saying whether the result is intermediate or final.

**11. References / evidence.**
- Zavala & Biegler (2009) *Automatica* 45:86–93, doi:10.1016/j.automatica.2008.06.011 —
  `[abstract]`; the preprint PDF's font encoding was unreadable.
- Zavala, Laird, Biegler (2008) *J. Process Control* 18:876–884,
  doi:10.1016/j.jprocont.2008.06.003 — `[abstract]`.
- Diehl, Bock, Schlöder (2005) *SIAM J. Control Optim.* 43:1714–1736,
  doi:10.1137/S0363012902400713 — `[read]`, KU Leuven copy, §3–5.
- Würth, Hannemann, Marquardt (2009) *J. Process Control* 19:1277–1288,
  doi:10.1016/j.jprocont.2009.02.001 — `[abstract]`; the full text was not accessible.
- Biegler, Yang, Fischer (2015) *J. Process Control* 30:104–116 — not read. Its predecessor,
  Biegler's 2013 DYCOPS survey, was `[read]`.

---

## M3 Implicit-function (tangent) and secant predictors for square roots F(x, p) = 0

**1. Prerequisites.**
- F is C¹ near (x*, p0), and C² for the O(Δp²) error bound.
- F(x*, p0) = 0 with F_x(x*, p0) nonsingular, i.e. a regular root. The IFT then gives a unique
  C¹ branch x(p) `[known]`.
- The secant predictor needs two converged points on the same branch, with no fold between
  them and the same parameter direction.
- Natural-parameter prediction fails at folds, where F_x is singular. Beyond a fold you need
  pseudo-arclength (Keller), which only requires rank n for [F_x F_p] `[known]`.
- The corrector's convergence region is governed by Newton–Kantorovich, h = βηγ ≤ ½. Here β
  bounds ‖F_x(x_pred)⁻¹‖, η is the length of the first Newton step and γ is the Lipschitz
  constant of F_x. These constants are unknown a priori and are estimated during the solve
  from the corrector's contraction (Deuflhard) `[known]`.
- The predicted point must lie in the evaluation domain: bounds, log or sqrt arguments, and
  the validity envelopes of property methods.

**2. Construction.**
- *Tangent (Euler/IFT):* solve F_x(x*, p0)·dx = −F_p·Δp, then x_pred = x* + dx. This is one
  backsolve per parameter direction, or one with a combined RHS.
  - Error is O(‖Δp‖²), with a constant that scales with ‖F_x⁻¹‖, ‖F_xx‖ and ‖dx/dp‖²
    `[known]`.
- *Secant (extrapolation):*
  x_pred = x_k + ((p_{k+1} − p_k)/(p_k − p_{k−1}))·(x_k − x_{k−1}).
  - It is defined only for a collinear parameter path.
  - Error is O(Δp_k(Δp_k + Δp_{k−1})).
  - Its higher-order relatives use polynomial extrapolation over three or more points
    `[known]`.
- *Pseudo-arclength:* the unit tangent t spans ker[F_x F_p], oriented by the sign of the
  determinant of the augmented Jacobian. The predictor is (x, p) + h·t `[known]`.
- pse-arrow's `square_response.rs` already computes dx/dp at a regular root. It uses a dense
  faer LU with an SVD rank check and withholds the result with `Withheld::Rank`, `Matching`
  and similar typed causes `[read]`.

**3. Retained state.**
- A factor of F_x **at x\***.
  - KINSOL's modified Newton evaluates the Jacobian at most every `msbset` (10) iterations,
    so the factor it holds at return may be from an earlier iterate. A tangent built from it
    is inexact.
  - An exact tangent needs one fresh Jacobian and numeric refactor at x* `[read: KINSOL
    source; interp]`.
- F_p columns, i.e. derivative-program directional derivatives with respect to the
  parameters.
- The parameter identity and units.
- For secant prediction, a branch history of (p, x) pairs with an identical variable mapping
  and structure stamp.
- For arclength, the tangent orientation.

**4. Step / globalization rule (step-length control).**
- Allgower–Georg (ch. 6) adapt the step from the corrector's behaviour `[known]`:
  - compare the contraction κ, the first corrector step δ and the angle α between successive
    tangents with nominal targets;
  - compute f = max(√(κ/κ̃), √(δ/δ̃), α/α̃) and set h_new = h/f, with f clipped to [½, 2].
- Deuflhard-style control `[known]`:
  - target a first-corrector contraction Θ₀ = ‖Δx̄¹‖/‖Δx⁰‖ ≤ ¼;
  - halve Δp when Θ₀ ≥ ½ or the corrector fails.
- Clip the prediction to bounds (fraction-to-boundary), or shrink Δp if the prediction leaves
  the domain `[interp]`.
- With strong regularity along the path, an Euler predictor plus Newton corrector reaches
  O(h⁴) (DKRV 2013) `[abstract]`.

**5. Acceptance / validity.**
- Before the corrector:
  - the prediction is finite and inside the domain;
  - its scaled residual ‖D_F F(x_pred, p_new)‖ is compared with that of the plain warm start,
    ‖D_F F(x*, p_new)‖. That costs two evaluations and keeps the prediction honest against
    plain reuse `[interp]`.
- Success means the corrector (KINSOL or diffsol-nl) converges to F(·, p_new) = 0 at the
  original tolerance, in original coordinates.
- Branch checks: the gap between the converged root and the prediction is consistent with
  the error model, and the sign of det F_x is unchanged (no fold or bifurcation crossed)
  `[interp]`.

**6. Refusal / abandonment.**
- *Facts:*
  - F_x is rank-deficient or ill-conditioned at the base (the SVD rank check);
  - the residual is nonsmooth between p0 and p (selectors, min/max, regime switches);
  - the parameter change alters structure;
  - for the secant, there are fewer than two branch points or the direction reverses.
- *Observations:*
  - Θ₀ > ½, which shrinks Δp;
  - corrector divergence or a line-search failure (`KIN_LINESEARCH_NONCONV`/`BCFAIL`), which
    retreats;
  - a sign change of det F_x, meaning a fold was passed: switch to arclength or stop;
  - a prediction outside the domain;
  - a converged root far from the prediction relative to the error model, which suggests a
    branch jump `[interp]`.

**7. Scaling.** Compute the tangent in scaled coordinates (KINSOL D_u, D_F). Normalize Δp by
parameter scales. Make every step-size test in scaled norms.

**8. Cost.**
- Tangent: n_p backsolves (or one), plus an F_p evaluation (one JVP per parameter direction).
  If the factor is stale, add one Jacobian and one numeric refactor, the same as one Newton
  iteration.
- Secant: vector operations only.
- Both are tiny next to the corrector iterations they save `[known]`.

**9. Interactions.**
- M3 feeds the M4/M5 corrector.
- The factor at x* can serve as the corrector's first Jacobian (modified Newton), and Broyden
  updates along the path (Allgower–Georg ch. 7/8) `[known]`.
- KINSOL can skip the initial linear-solver setup when `KINSetNoInitSetup(TRUE)` is set. The
  next `KINSol` call then reuses the matrix and factor held in the same KINSOL instance (the
  source sets `sthrsh = 1`).
  - pse-arrow currently sets it to 0, which forces a refresh
    `[read: kinsol.c; pse-backend-native/src/kinsol.rs]`.
- M3 is the root analogue of M1. It is the stage predictor for continuation (another card),
  and it suits ordered parameter studies.

**10. Library realizations; minimum information.**
- No Rust continuation library is known to me `[known; not searched exhaustively]`.
- Composition in the stack:
  - faer LU or KLU for the tangent;
  - KINSOL or diffsol-nl as the corrector;
  - pse-arrow `square_response` for regular-root dx/dp.
- External reference implementations: LOCA (Trilinos), PETSc arc-length SNES, PITCON, AUTO,
  HOMPACK `[known]`.
- *Minimum information:*
  - parameter identity and the F_p operator;
  - the Jacobian or factor at the converged root, stamped with the point at which it was formed;
  - the branch history with its structure stamp;
  - scaling vectors and domain bounds;
  - per-iteration corrector step norms (for Θ₀).

**11. References / evidence.**
- Allgower & Georg, *Introduction to Numerical Continuation Methods* (SIAM Classics,
  doi:10.1137/1.9780898719154), ch. 2–3 and 6–8 — `[known]`; not re-read, and no open copy was
  available.
- Deuflhard, *Newton Methods for Nonlinear Problems* (Springer) — `[known]`. The ZIB NLEQ1
  source was `[read]`.
- Dontchev, Krastanov, Rockafellar, Veliov (2013) — `[abstract]`.
- Keller (1977), "Global homotopies and Newton methods" — `[known]`.
- KINSOL 7.1.1 source — `[read]`.
- pse-arrow `square_response.rs` — `[read]`, docs and signatures only.

---

## M4 Inexact Newton, forcing terms, Newton–Krylov and JFNK

**1. Prerequisites.**
- *Local theory* (Dembo–Eisenstat–Steihaug 1982) `[read via EW94]`:
  - F is C¹ with F′ Lipschitz near x*, and F′(x*) is nonsingular;
  - η_k ≤ η_max < 1 gives q-linear convergence in a weighted norm; η_k → 0 gives
    superlinear convergence; η_k = O(‖F_k‖) gives quadratic convergence.
- *Global theory* (Eisenstat–Walker 1994, Thm 6.1) `[read]`: if INB does not break down and
  {x_k} has a limit point x* at which F′ is invertible, then F(x*) = 0, x_k → x*, and
  eventually full inexact steps are accepted.
- *Krylov solves* need:
  - J·v products, either analytic JVPs from the derivative program or finite differences;
  - a preconditioner P ≈ J. Knoll–Keyes stress that algorithmic scalability is achieved in
    the preconditioner `[read]`.
- *JFNK with finite differences* also needs F to be smooth and deterministic at the
  differencing scale. Nested iterative property calculations with loose inner tolerances
  inject noise that spoils FD JVPs `[interp]`.

**2. Construction.**
- Find s_k with ‖F_k + J_k s_k‖ ≤ η_k‖F_k‖ (KINSOL uses D_F-scaled norms) `[read]`.
- *EW Choice 1:* η_k = |‖F_k‖ − ‖F_{k−1} + J_{k−1}s_{k−1}‖| / ‖F_{k−1}‖, with the safeguard
  η_k ← max(η_k, η_{k−1}^{(1+√5)/2}) whenever η_{k−1}^{(1+√5)/2} > 0.1. Convergence is
  q-superlinear, two-step q-quadratic, and of r-order (1+√5)/2 `[read: EW96 §2]`.
- *EW Choice 2:* η_k = γ(‖F_k‖/‖F_{k−1}‖)^α, with γ ∈ [0,1] and α ∈ (1,2], and the safeguard
  max(η_k, γη_{k−1}^α) whenever γη_{k−1}^α > 0.1. Convergence is of q-order α when γ < 1
  `[read]`.
- *KINSOL 7.1.1* `[read: kinsol.c KINForcingTerm]`:
  - default Choice 1;
  - η clipped to [1e-4, 0.9], with η₀ = 0.5;
  - Choice 2 defaults γ = 0.9, α = 2; the constant forcing term defaults to 0.1;
  - the stop is ‖Jδ + F‖ < (η + U)‖F‖.
- *Krylov solvers:* GMRES, FGMRES, BiCGStab and TFQMR with right preconditioning. The
  finite-difference product is J v ≈ [F(u+σv) − F(u)]/σ, with σ from KINSOL's √U-based
  formula or Brown–Saad's typical-u formula `[read: KINSOL docs; Knoll–Keyes §2.3]`.

**3. Retained state.** The previous ‖F‖ and linear-model norm (KINSOL tracks sFdotJp and
sJpnorm for Choice 1), the previous η, the preconditioner and its setup age, and the Krylov
workspace.

**4. Step / globalization rule.**
- *INB* `[read: EW94 §6]`: while ‖F(x+s)‖ > [1 − t(1−η)]‖F(x)‖, set s ← θs and
  η ← 1 − θ(1−η), with θ ∈ [θ_min, θ_max].
- *KINSOL line search* on f = ½‖D_F F‖² `[read]`:
  - α-condition with 1e-4 and β-condition with 0.9;
  - λ_min = steptol / ‖δ̄‖∞;
  - mxnewtstep defaults to 1000‖D_u u₀‖₂, and never below 1;
  - at most 10 β-condition failures (MXNBCF_DEFAULT).
- KINSOL **disables residual monitoring with inexact (Krylov) linear solvers** `[read: kinsol.c]`.
- Brown–Saad (1990) combine Krylov steps with a backtracking line search or a dogleg trust
  region restricted to the Krylov subspace `[abstract]`.

**5. Acceptance / validity.**
- The inner test is the linear-residual condition.
- The outer test is ‖D_F F‖∞ < fnormtol (KINSOL default U^{1/3}).
- A small scaled step (scsteptol, U^{2/3}) is **not** success, because it may be a stall
  `[read]`.
- Inexactness changes only the route. Final acceptance re-evaluates the original residual in
  original coordinates.

**6. Refusal / abandonment.**
- *Facts:*
  - A direct sparse factorization (KLU) that is affordable and robust makes forcing terms
    irrelevant: η only controls Krylov work (colleague's point, consistent with KINSOL)
    `[read]`.
  - Large fill-in or a matrix-free setting favours Krylov.
  - Krylov requires a preconditioner; unpreconditioned Krylov on badly scaled PSE Jacobians is
    not a credible default `[interp]`.
  - JVP availability: an analytic JVP makes "Jacobian-free" exact.
  - Evaluation noise (nested iterative property calls) refuses FD JFNK or forces a larger σ.
- *In-solve observations* (KINSOL getters wired in pse-arrow) `[read: kinsol.rs]`:
  - `KINGetNumLinIters` per nonlinear iteration and `KINGetNumLinConvFails` growing, or
    `KINGetLastLinFlag` reporting failure, trigger a preconditioner refresh (M5) or a switch
    to direct;
  - `KINGetNumBacktrackOps`, `KINGetNumBetaCondFails` and a tiny `KINGetStepLength` signal
    weakening globalization;
  - `KIN_LINESEARCH_NONCONV`/`BCFAIL` escalate to M7, continuation or pseudo-transient;
  - a residual ratio ‖F_{k+1}‖/‖F_k‖ ≪ 1 marks the local regime, where η falls automatically.
- These counters are **cumulative after return**. Per-iteration ratios need either
  `KINSetInfoHandlerFn`/print-level parsing or iteration-bounded calls `[interp]`.

**7. Scaling.** D_u and D_F enter the norms, η and the FD σ. Preconditioned conditioning
governs the Krylov iteration count, and a poor σ ruins the FD JVP at both extremes
(truncation versus round-off) `[read: Knoll–Keyes §2.3.1]`.

**8. Cost.**
- Per Newton iteration: one F evaluation, plus Krylov iterations × (one JVP + one
  preconditioner solve), plus amortized preconditioner setup.
- Direct Newton: one Jacobian plus a sparse LU per setup.
- Adaptive η cuts Krylov work at far iterates and preserves the local rate `[read]`.

**9. Interactions.**
- M5: lagging only the preconditioner keeps the outer iteration Newton. Knoll–Keyes Table 2:
  JFNK with the preconditioner refreshed every 10 iterations is fastest; lagging the Jacobian
  in the outer iteration (modified Newton–Krylov, p = 5) took 681 versus 158 Newton
  iterations `[read]`.
- M3/M1 supply initial guesses.
- M7 supplies the trust-region variant.
- Pseudo-transient and nonlinear preconditioning (other cards) run inexact Newton per stage.

**10. Library realizations; minimum information.**
- KINSOL linear solvers: SPGMR, SPFGMR, SPBCGS, SPTFQMR, plus KLU and dense.
- KINSOL controls: `KINSetEtaForm`, `KINSetEtaConstValue`, `KINSetEtaParams`,
  `KINSetJacTimesVecFn`, `KINSetPreconditioner`, `KINSetMaxSetupCalls`.
- All of these are wired in pse-arrow `settings/kinsol.rs` and `kinsol.rs` `[read]`.
- PETSc SNES/KSP with Eisenstat–Walker is a reference implementation outside the stack
  `[known]`.
- *Minimum information:*
  - JVP operator (analytic preferred);
  - preconditioner setup/solve with an age stamp;
  - D_u and D_F scaling;
  - sparsity and fill estimate from symbolic analysis;
  - an evaluation-noise class;
  - per-iteration linear statistics.

**11. References / evidence.**
- Dembo, Eisenstat, Steihaug (1982) *SIAM J. Numer. Anal.* 19:400–408, doi:10.1137/0719025 —
  `[abstract / restated in EW94, EW96]`.
- Eisenstat & Walker (1994) *SIAM J. Optim.* 4:393–422, doi:10.1137/0804022 — `[read]`, the
  author's copy, §1 and §6; OCR was partial.
- Eisenstat & Walker (1996) *SIAM J. Sci. Comput.* 17:16–32, doi:10.1137/0917003 — `[read]`,
  §2 and 2.1.
- Brown & Saad (1990) *SIAM J. Sci. Stat. Comput.* 11:450–481, doi:10.1137/0911026, and (1994)
  *SIAM J. Optim.* 4:297–330, doi:10.1137/0804017 — `[abstract]`.
- Knoll & Keyes (2004) *J. Comput. Phys.* 193:357–397, doi:10.1016/j.jcp.2003.08.010 —
  `[read]`, open copy §2–3 and §5.1.
- KINSOL 7.1.1 "Mathematical Considerations" and vendored source — `[read]`.

---

## M5 Modified Newton, Jacobian and preconditioner lagging, Broyden quasi-Newton

**1. Prerequisites.**
- *Newton's local regularity:* F′(x*) nonsingular and F′ Lipschitz.
- *General approximate-Jacobian contraction condition* (Bock; DBS 2005 (4.1a–d)) `[read]`:
  - ‖J̃⁻¹(J̃ − F′(y))‖ ≤ κ < 1 on the region, plus an ω bound;
  - this gives a linear contraction with rate about κ + (ω/2)‖Δx‖;
  - it licenses any lagged or approximate Jacobian, but only while κ < 1.
- *Chord* (J frozen at x₀): q-linear convergence, with a rate that grows with ‖x₀ − x*‖.
- *Shamanskii* (refresh every m iterations): q-order m+1 per Jacobian `[known; confirmed by
  arXiv:1609.03328 summary]`.
- *Broyden:* x₀ near x* and B₀ near F′(x*) give local q-superlinear convergence through
  bounded deterioration (Broyden–Dennis–Moré). It has no global property on its own
  `[known]`.
- *Cross-solve reuse:* an identical structure (sparsity, ordering, scaling) is required.
  Numerical staleness is tolerated, and the monitors below catch it.

**2. Construction.**
- *Chord / modified Newton:* s = −J̃⁻¹F(x_k) with a reused factor.
- *KINSOL refresh rules* `[read: KINSOL docs; kinsol_impl.h defaults]`:
  - the setup runs every `msbset` = 10 nonlinear iterations;
  - it is also forced on the first iteration (unless NoInitSetup), on a linear-solver failure
    or a line-search failure with an outdated Jacobian, and on a small step with outdated
    information;
  - with Krylov solvers, it is also forced when the last step satisfies
    ‖λδ_{n−1}‖_{D_u,∞} > 1.5;
  - *residual monitoring (direct solvers only):* after `msbsetsub` = 5 iterations, refresh if
    ‖F(u_n)‖ > ω‖F(u_m)‖, where ω = min(ω_min·e^{max(0, ρ−1)}, ω_max),
    ρ = ‖F_n‖/fnormtol, ω_min = 1e-5 and ω_max = 0.9.
- *Lagged preconditioner only (JFNK):* the outer iteration stays Newton and only the Krylov
  count grows (Knoll–Keyes §5.1) `[read]`.
- *Broyden:* B₊ = B + (y − Bs)sᵀ/(sᵀs).
  - *MINPACK `hybrj`* rank-1 updates the QR factor every iteration (`r1updt`) and recomputes
    the Jacobian after two consecutive failed trust-region steps (`ncfail == 2`)
    `[read: minpack.f90]`.
  - *NLEQ1* uses rank-1 updates only when the last two damping factors were 1 (full steps)
    and the damping predictor exceeds σ·λ (σ default 3). Otherwise it builds a new Jacobian.
    It refreshes the Jacobian if λ < λ_min while the current Jacobian came from rank-1
    updates; otherwise it fails `[read: nleq1.f]`.
- *diffsol-nl convergence monitor* `[read: convergence.rs]`:
  - rate = (‖Δ_k‖/‖Δ₁‖)^{1/(k−1)};
  - it declares "diverged" if rate > 0.9, or if rate^{max−k}/(1−rate)·‖Δ_k‖ > tol;
  - the caller then refreshes the Jacobian or reduces its step.
- *NLP analogue:* quasi-Newton Hessians (Ipopt `hessian_approximation=limited-memory`,
  history 6; POUNCE SQP BFGS/L-BFGS). They save Hessian evaluations at the cost of the local
  rate, and **M1 sensitivity requires the exact Hessian** `[read: Ipopt source; sIPOPT §2.5]`.

**3. Retained state.** The Jacobian or preconditioner factor with a formation stamp (point,
parameter, structure, scaling), the iterations since setup, ‖F(u_m)‖ at setup, the rank-1
update history or the updated QR factor, and the contraction history.

**4. Step / globalization rule.** Unchanged from the host method: a line search (KINSOL), a
trust region (MINPACK) or affine-covariant damping (NLEQ1). A failure with a stale Jacobian
means "refresh and retry", not "abandon".

**5. Acceptance / validity.** Unchanged final test on the original residual. Lagging alters
only the route.

**6. Refusal / abandonment.**
- *Facts:*
  - Compare the cost of Jacobian evaluation plus numeric factorization with the cost of a
    residual evaluation plus backsolve.
  - Lagging pays when factorizations are expensive (large sparse, high fill) relative to
    residual evaluations. It gains little for tiny dense systems `[known]`.
  - Strongly nonlinear far-field iterations are a reason to lag little.
  - Dense Broyden destroys sparsity, so refuse it for large sparse systems unless a
    limited-memory form is used `[known]`.
- *Observations:*
  - the contraction ratio ‖Δx_{k+1}‖/‖Δx_k‖ or the residual ratio above a threshold
    (diffsol-nl 0.9, KINSOL residual monitor ω), which triggers a refresh;
  - a line-search failure, which refreshes in KINSOL;
  - two consecutive trust-region failures, which refresh in MINPACK;
  - a damping factor below 1, which disables Broyden (NLEQ1);
  - a rising Krylov count with a stale preconditioner, which refreshes the preconditioner.

**7. Scaling.** Monitors use scaled norms. Finite-difference Jacobian increments are
σ_j = √U·max(|u_j|, 1/D_u,j) `[read: KINSOL docs]`. KLU numeric refactor reuses the symbolic
analysis and pivot order only while the pattern is unchanged `[known]`.

**8. Cost.** Lagging saves Jacobian evaluations and numeric factorizations, and pays for them
with extra nonlinear iterations (F evaluations plus backsolves). Dense Broyden costs O(n²)
per update, and an O(n²) QR update.

**9. Interactions.**
- M4: the preconditioner is lagged.
- M3: the tangent's factor becomes the corrector's first Jacobian; Allgower–Georg update
  along the path.
- M1: the same factor-reuse idea, applied across parameters.
- M6: Anderson acceleration is a multisecant (Broyden-type II) method.
- M7: MINPACK's hybrid method is Broyden inside a dogleg trust region.
- M8: quasi-Newton Hessians.

**10. Library realizations; minimum information.**
- KINSOL: `KINSetMaxSetupCalls`, `KINSetMaxSubSetupCalls`, `KINSetResMonParams`,
  `KINSetResMonConstValue`, `KINSetNoResMon` and `KINSetNoInitSetup`.
  - pse-arrow wires `MaxSetupCalls` and `NoInitSetup(0)`.
  - It does **not** wire `MaxSubSetupCalls`, residual-monitor parameters, `NoInitSetup(TRUE)`
    for cross-solve reuse, `MaxBetaFails` or `ReturnNewest` (grep of
    `crates/pse-backend-native/src`).
- diffsol-nl: `NewtonNonlinearSolver` with `reset_jacobian` and its convergence monitor.
- MINPACK `hybrj`/`hybrd` (fortran-lang/minpack, with a C API) and NLEQ1/NLEQ2 (ZIB, licence)
  are outside the stack.
- PETSc `SNESSetLagJacobian`/`SNESSetLagPreconditioner` (−2/−1/1/n, with persistence across
  solves) is a reference implementation `[read: PETSc manual page]`.
- *Minimum information:*
  - per-iteration correction and residual norms in scaled units;
  - setup counts and formation stamps;
  - a cost model for Jacobian and factorization versus residual (from callback metrics);
  - a structure stamp for cross-solve reuse.

**11. References / evidence.**
- Kelley, *Iterative Methods for Linear and Nonlinear Equations* (SIAM FR16, 1995,
  doi:10.1137/1.9781611970944) ch. 5 and 7 — `[known]`; not re-read.
- Diehl, Bock, Schlöder (2005), Thm 4.1 — `[read]`.
- Knoll & Keyes (2004) §5.1 — `[read]`.
- Broyden, Dennis, Moré (1973) — `[known]`.
- KINSOL docs and source, MINPACK source, NLEQ1 source, diffsol-nl source — `[read]`.
- PETSc `SNESSetLagJacobian` page — `[read]`.

---

## M6 Anderson acceleration and fixed-point admission

**1. Prerequisites.**
- A **fixed-point map G with x* = G(x\*) that is justified independently of AA.** In KINSOL
  there are two routes `[read: KINSOL docs]`:
  - fixed-point iteration, where the user supplies G;
  - Picard iteration, F(u) = Lu − N(u) with L constant and nonsingular, and
    G(u) = u − L⁻¹F(u).
- Convergence theory assumes G is Lipschitz and contractive near x* (‖G′‖ ≤ c < 1):
  - *Toth–Kelley (2015):* local r-linear convergence with r-factor ≤ c, provided the
    coefficient sums Σ|α_j| stay bounded `[abstract; restated in EPRX]`;
  - *Evans–Pollock–Rebholz–Xiao (2020):* for C² G the residual rate is θ_k((1−β) + βc), plus
    O(‖w‖²) terms. θ_k ∈ [0,1] is the optimization gain. AA *improves linearly convergent*
    iterations, does **not** improve quadratically convergent ones, and with damping can
    extend the convergence radius beyond the contractive region, with no guarantee `[read]`.
- A generic residual is **not** a fixed-point map. G(x) = x − F(x) contracts only if
  ‖I − F′‖ < 1, which depends on scaling and rarely holds `[known]`.

**2. Construction (Walker–Ni; KINSOL).**
- f_k = G(x_k) − x_k and m_k = min(m, k).
- Solve min_γ‖f_k − ΔF_k γ‖₂ via a QR factorization of ΔF_k.
- Update x_{k+1} = x_k + βf_k − (ΔX_k + βΔF_k)γ, where β is the damping (1 means undamped).
- Options: a delay of d plain iterations before AA starts; MGS, ICWY, CGS2 or DCGS2
  orthogonalization `[read: KINSOL docs]`.
- The gain is θ_k = √(1 − (‖Q₁ᵀf_k‖/‖f_k‖)²), the direction sine, which can be computed from
  the QR. EPRX use it for adaptive damping `[read]`.
- Untruncated AA on linear problems is essentially GMRES (Walker–Ni), and AA is a multisecant,
  type-II Broyden method `[abstract / known]`.

**3. Retained state.** The last m iterate and residual differences (ΔX, ΔF), their QR factors
and the iteration count. No Jacobian is needed. Picard needs the factor of L, which is
constant across iterations.

**4. Step / globalization rule.** KINSOL's fixed-point and Picard modes have **no line
search**. Controls are:
- fixed damping β (`KINSetDampingAA`);
- delay (`KINSetDelayAA`) and depth m (`KINSetMAA`);
- when the history is full, the oldest column is dropped by QR deletion `[read]`.

Further controls exist only in the literature: adaptive damping from θ_k (EPRX §5.3), and
restarts or column dropping on ill-conditioned R (Walker–Ni) `[read / abstract]`.

**5. Acceptance / validity.**
- Picard tests ‖F(u_{n+1})‖ directly `[read]`.
- For a plain fixed point, a small ‖G(u) − u‖ implies a small F only through the declared
  relation between G and F. For G = u − L⁻¹F, ‖F‖ ≤ ‖L‖‖G(u) − u‖ `[interp]`.
- Final acceptance re-evaluates the original F in original coordinates.

**6. Refusal / abandonment.**
- *Facts:*
  - There is no declared, justified G or splitting. Acceptable sources are a constant-L
    Picard splitting, a tear or recycle substitution map, a block Gauss–Seidel sweep, or
    another documented relaxation.
  - There is no evidence or argument for contraction. Examples of such evidence: an M-matrix
    or diagonally dominant splitting, or an estimate of ‖L⁻¹N′‖ < 1.
  - Sign or bound protection is required. KINSOL fixed-point and Picard forbid constraints,
    and pse-arrow refuses that combination `[read: kinsol.rs]`.
  - A cheap, reliable Newton Jacobian exists, and AA does not accelerate quadratic
    iterations `[read: EPRX]`.
  - AA is most valuable when Jacobians are unavailable or expensive and G contracts slowly,
    with c close to 1 `[interp]`.
- *Observations:*
  - the fixed-point residual ratio ‖f_{k+1}‖/‖f_k‖ ≥ 1 repeatedly, meaning no contraction:
    damp, then abandon;
  - θ_k ≈ 1, meaning the extrapolation gains nothing: reduce depth or stop;
  - a small diagonal of R, meaning ΔF is near-dependent: drop columns or restart;
  - stagnation at maximum iterations: escalate to Newton;
  - G evaluation failures from domain errors, which cannot be recovered because there is no
    line search.
- KINSOL does not expose θ_k or cond(R) `[interp: not found in the API list reviewed]`.

**7. Scaling.** The least-squares problem is in ℓ₂ over f's components, so the residual
scaling shapes γ. Whether KINSOL applies `fscale` inside the AA least squares was not verified
`[open question]`. Conditioning of ΔF limits the useful depth.

**8. Cost.** One G evaluation per iteration plus an O(n·m) QR update. Picard adds one
backsolve with the constant L factor. There are no Jacobians or refactorizations. Convergence
is linear.

**9. Interactions.**
- Natural partner of structural decomposition (tear streams) and of nonlinear preconditioning,
  where G is block solves (other cards).
- An outer accelerator for Gauss–Seidel sweeps.
- A relative of M5 (multisecant).
- Not an accelerator of M4 Newton iterations.

**10. Library realizations; minimum information.**
- KINSOL `KIN_FP` and `KIN_PICARD` with AA, via `KINSetMAA`, `KINSetDampingAA`,
  `KINSetDelayAA` and `KINSetOrthAA`. pse-arrow wires all four and refuses constraints with
  FP/Picard `[read]`.
- PETSc `SNESANDERSON`/`SNESNGMRES` is a reference implementation `[known]`.
- *Minimum information:*
  - the declared G or (L, N) with its provenance and contraction rationale;
  - an original-F evaluator for acceptance;
  - consistent scaling;
  - per-iteration fixed-point residual norms.

**11. References / evidence.**
- Walker & Ni (2011) *SIAM J. Numer. Anal.* 49:1715–1735, doi:10.1137/10078356X —
  `[abstract]`.
- Toth & Kelley (2015) *SIAM J. Numer. Anal.* 53:805–819, doi:10.1137/130919398 —
  `[abstract]`; the NCSU copy returned HTML.
- Evans, Pollock, Rebholz, Xiao (2020) *SIAM J. Numer. Anal.* 58:788–810, arXiv:1810.08455 —
  `[read]`. Its DOI, 10.1137/19M1245384, is from memory.
- KINSOL docs and source — `[read]`.

---

## M7 Trust-region and Levenberg–Marquardt local models; SLP/SQP-filter auxiliary models

**1. Prerequisites.**
- F (or f and c) is C¹ with a Lipschitz Jacobian on the region.
- F and J must be **evaluable at the centre point**. A Jacobian cannot be formed at an
  invalid logarithm or a failed property call (colleague's point) `[read]`.
- *Least-squares models:*
  - LM and Gauss–Newton trust regions converge globally to *stationary points of ½‖F‖²*,
    which may be non-roots (JᵀF = 0 with F ≠ 0) `[known]`;
  - local quadratic convergence holds under a **local error bound**, which is weaker than
    nonsingularity, when λ_k = ‖F_k‖² (Yamashita–Fukushima 2001; Fan–Yuan generalize to
    λ = ‖F‖^δ with δ ∈ [1,2]) `[abstract]`.
- *Dogleg (Powell hybrid):* needs a nonsingular J for the Newton point `[known]`.
- *SLP/SQP-filter:* C² data plus the constraint-qualification-type assumptions of the
  filter-convergence proofs `[abstract]`.

**2. Construction.**
- *LM (Moré 1978):* (JᵀJ + λDᵀD)d = −JᵀF with ‖Dd‖ ≤ Δ `[known + read: minpack.f90]`.
  - λ comes from the secular-equation iteration in `lmpar`.
  - D holds column norms, updated monotonically as D_j ← max(D_j, ‖J_j‖).
- *Powell dogleg with Broyden updates:* MINPACK `hybrj` `[read]`.
- *Lifted bounded LM QP* (colleague): min ½‖e‖² + (λ/2)‖d‖² subject to e = r + Jd,
  ℓ ≤ z₀ + d ≤ u, ‖d‖∞ ≤ Δ.
  - It is a convex QP with a PSD Hessian, so HiGHS QP or Clarabel can solve it. It is a
    projected or bounded LM `[interp]`.
- *SLP-filter* (Fletcher–Leyffer–Toint; Chin–Fletcher take EQP steps) `[abstract]`:
  - LP subproblems inside an ∞-norm trust region;
  - filter acceptance instead of a penalty;
  - restoration when the LP is infeasible.
- *Trust-region SQP-filter* (Fletcher–Gould–Leyffer–Toint–Wächter 2002) `[abstract/known]`:
  - a composite step: a normal step toward feasibility, then a tangential QP step;
  - restoration when the trust-region QP is incompatible.
- *Successive convexification* (Mao–Szmuk–Açıkmeşe) `[read]`:
  - linearize the nonconvex constraints;
  - add *virtual control*, an ℓ₁-penalized slack against artificial infeasibility from
    linearization;
  - add a *trust region* against unboundedness of the linearized problem.

**3. Retained state.** The centre z_k with F_k and J_k (or its factor or QR), the radius Δ_k,
λ_k, the scaling D_k, and the counters ncsuc, ncfail, nslow1 and nslow2. Filter methods add
the (θ, φ) filter entries.

**4. Step / globalization rule.** Trust-region radius updates are driven by ρ = ared/pred.
- *`hybrj`* `[read]`:
  - Δ₀ = factor·‖Dx₀‖ (factor default 100), capped by the first step length;
  - accept the step if ρ ≥ 1e-4;
  - if ρ < 0.1, set Δ ← Δ/2;
  - if ρ ≥ 0.5 or there were two successes in a row, Δ ← max(Δ, 2‖Dp‖);
  - if |ρ − 1| ≤ 0.1, Δ ← 2‖Dp‖.
- *`lmder`* `[read]`:
  - if ρ ≤ 0.25, shrink by a factor in [0.1, 0.5];
  - if ρ ≥ 0.75 or λ = 0, Δ ← 2‖Dp‖;
  - accept the step if ρ ≥ 1e-4.
- *Successive convexification* (with 0 < ρ₀ < ρ₁ < ρ₂ < 1 and α > 1) `[read]`:
  - reject if r < ρ₀;
  - shrink by α if r < ρ₁;
  - keep the radius if ρ₁ ≤ r < ρ₂;
  - grow by α otherwise.

**5. Acceptance / validity.**
- **An auxiliary LP/QP succeeding means only that the auxiliary problem was solved.**
- The trial point is judged by the *original* evaluator through ared versus pred.
- Termination is tested on the original residual (roots) or the original KKT conditions
  (NLP), in original coordinates.
- An LM/hybrid stop with info 4/5 (no progress), or with ‖JᵀF‖ ≈ 0 while ‖F‖ is large, must be
  reported as "least-squares stationary, not a root" `[read: minpack.f90; interp]`.

**6. Refusal / abandonment.**
- *Facts:*
  - no evaluable centre or Jacobian;
  - nonsmooth functions inside the region;
  - for NLPs, a feasibility-only model ignores the objective, so it is only a seed;
  - **for NLPs solved by POUNCE active-set SQP, the SQP already solves local QP models,** so a
    separate QP initializer duplicates its first iterations `[read: colleague; POUNCE sqp/]`.
- *Observations:*
  - ρ persistently small, causing Δ to collapse;
  - Δ ≤ xtol·‖Dx‖ (MINPACK info 2/3);
  - `nslow2 == 5`, i.e. no progress over 5 Jacobian evaluations (`hybrj` info 4);
  - `nslow1 == 10`, i.e. no progress over 10 iterations (`hybrj` info 5);
  - ‖JᵀF‖ ≈ 0 with ‖F‖ ≫ tol, a local least-squares minimum: escalate to continuation or
    homotopy (other cards);
  - an evaluation failure at the trial point, treated as ρ = −∞ so Δ shrinks
    (a domain-aware trust region) `[interp]`;
  - an infeasible LP/QP subproblem or repeated restoration, which ends the model method.

**7. Scaling.** The scaling D (monotone column-norm maximum) shapes the trust region. Residual
scaling and λ are scale-dependent. An ∞-norm trust region maps to bounds, which suits LP/QP
solvers.

**8. Cost.**
- Per iteration: one Jacobian (or a Broyden update), plus a small factorization (QR or the LM
  augmented system; `lmpar` may factor more than once), plus one F evaluation per trial.
- A lifted QP solve through HiGHS or Clarabel costs more than one Newton step. It is therefore
  reasonable only as a short preparatory sequence, not as a phase run every time `[interp]`.

**9. Interactions.**
- M5: Broyden inside the hybrid method.
- M4: the Brown–Saad Krylov trust region.
- M8: Ipopt's restoration phase is itself an ℓ₁ feasibility problem with a proximity term, the
  same family as the feasibility-oriented LM initializer; Ipopt also offers
  `start_with_resto`.
- Continuation or homotopy takes over when LM stalls at a non-root.
- The output is a seed.

**10. Library realizations; minimum information.**
- MINPACK `hybrj`/`hybrd`/`lmder`/`lmdif` (fortran-lang/minpack has a C API), not in the
  stack `[read]`.
- Rust crates `levenberg-marquardt` (a MINPACK port) and `argmin` (trust-region solvers)
  `[known; not verified here]`.
- HiGHS (convex QP via Hessian upload) and Clarabel solve the lifted QP and are in the stack
  `[read: skill matrix]`.
- POUNCE active-set SQP uses a filter **line search** (the Fletcher–Leyffer dominance test
  without f/h-mode switching) or an ℓ₁-elastic merit, **not a trust region**
  `[read: sqp/filter.rs]`.
- KINSOL has no trust region; it uses a line search and the `mxnewtstep` cap.
- Ipopt has no trust region.
- *Minimum information:*
  - residual and sparse Jacobian at a point;
  - bounds and scales;
  - a **typed domain-failure signal** from evaluators;
  - an original evaluator for ared;
  - per-iteration ρ, Δ and stop reason.

**11. References / evidence.**
- Conn, Gould, Toint, *Trust Region Methods* (SIAM 2000, doi:10.1137/1.9780898719857) —
  `[known]`.
- Moré (1978), LNM 630 — `[known]`. The MINPACK `lmder` and `hybrj` source was `[read]`.
- Sorensen (1982) *SIAM J. Numer. Anal.* 19:409–426 — `[known]`.
- Fletcher et al. (2002) *SIAM J. Optim.* 13:635–659, doi:10.1137/S1052623499357258 —
  `[abstract]`.
- Fletcher–Leyffer–Toint, SLP-filter report (optimization-online 2000/08/209) — `[abstract]`;
  the PDF text was unreadable.
- Chin & Fletcher (2003) *Math. Prog.* 96:161–177 — `[abstract]`.
- Mao, Szmuk, Açıkmeşe, arXiv:1608.05133 — `[read]`.
- Yamashita & Fukushima (2001) — `[abstract via search summary]`.

---

## M8 Ipopt's globalization as a production algorithm (Wächter–Biegler)

**1. Prerequisites.**
- f and c are C², with an exact Hessian or a quasi-Newton approximation.
- Bounds are handled by a barrier with strictly interior iterates.
- The filter line-search global convergence results (Wächter–Biegler, SIAM J. Optim. 2005)
  assume bounded iterates, a uniformly bounded inverse reduced Hessian, and well-defined
  restoration, among other conditions `[known]`.
- Fast local convergence, with second-order corrections against the Maratos effect, needs
  LICQ, SOSC and SC `[known]`.
- Degenerate structures thrash the standard filter. Examples are a rank-deficient Jacobian and
  complementarity constraints (MPCC-like). POUNCE carries Thierry–Biegler ℓ₁ penalty-barrier
  wrapping for these cases `[read: pounce-l1penalty README]`.

**2. Construction.** Defaults are taken from the Ipopt 3.14.20 option registrations, which cite
the implementation paper's equation numbers `[read]`.
- *Primal-dual barrier Newton step.* The augmented system is
  [W + Σ + δ_w I, A; Aᵀ, −δ_c I].
- *Inertia correction.* It applies when the inertia is not (n, m, 0):
  - δ_w⁰ = 1e-4;
  - δ_w grows ×100 on the first correction and ×8 afterwards;
  - it restarts from (last δ_w)/3;
  - δ_c = 1e-8·μ^{0.25} when the constraint Jacobian is rank-deficient.
- *Fraction to the boundary* uses τ = max(0.99, 1 − μ).
- *Monotone barrier update:* μ ← max(tol/10, min(0.2μ, μ^{1.5})), or an adaptive
  quality-function oracle.
- *Filter* over (θ = constraint violation, φ = barrier objective).
  - A trial is acceptable if θ ≤ (1 − γ_θ)θ_k or φ ≤ φ_k − γ_φθ_k, with γ_θ = 1e-5 and
    γ_φ = 1e-8.
  - The *switching condition* is m_k(α) < 0 and [−m_k(α)]^{s_φ}·α^{1−s_φ} > δ·θ_k^{s_θ},
    with s_φ = 2.3, s_θ = 1.1 and δ = 1. When it holds and θ ≤ θ_min = 1e-4·max(1, θ₀),
    the Armijo condition φ ≤ φ_k + η_φ m_k(α) applies, with η_φ = 1e-8.
  - θ_max = 1e4·max(1, θ₀).
- *Line search and its safeguards:*
  - backtracking halves α;
  - up to 4 second-order corrections, with κ_soc = 0.99;
  - an α_min safety factor of 0.05;
  - a watchdog after 10 shortened iterations, allowing up to 3 trial iterations;
  - up to 5 filter resets;
  - soft restoration for at most 10 iterations.
- *Restoration phase.* It runs an inner IPM on
  min ρ‖p+n‖₁ + (ζ/2)‖D_R(x − x_R)‖² subject to c(x) − p + n = 0.
  - It returns once the trial is filter-acceptable and the infeasibility has been cut to a
    fraction 0.9 of its value (`required_infeasibility_reduction`).
  - Convergence to a stationary point with nonzero θ reports `Infeasible_Problem_Detected`,
    which means *local* infeasibility only.
- *Initialization:*
  - x₀ is pushed into the interior with `bound_push` = `bound_frac` = 1e-2;
  - λ₀ is a least-squares estimate (capped by `constr_mult_init_max` = 1e3);
  - z₀ = 1.
- *Warm start* `[read: Ipopt OPTIONS]`:
  - `warm_start_init_point=yes` uses the supplied x, λ and z;
  - push values are `warm_start_bound_push` = `warm_start_mult_bound_push` = 1e-3, and
    `warm_start_mult_init_max` = 1e6;
  - `mu_init` defaults to 0.1;
  - `warm_start_same_structure` assumes an identical structure.

**3. Retained state across solves.** The primal-dual iterate (x, λ, z_L, z_U, slacks), μ at
termination, NLP scaling factors and a structure stamp. Ipopt keeps **no** factor or filter
across solves; sensitivity layers keep the factor.

**4. Step / globalization rule.** As above: the filter line search with switching condition,
SOC, watchdog and resets, and the restoration phase when α < α_min.

**5. Acceptance / validity.**
- Ipopt's own test is a scaled error ≤ `tol` (1e-8) together with unscaled
  `dual_inf_tol` = 1, `constr_viol_tol` = 1e-4 and `compl_inf_tol` = 1e-4.
- "Acceptable" needs `acceptable_tol` = 1e-6 for 15 consecutive iterations.
- `bound_relax_factor` (1e-8) relaxes the bounds.
- **Native status is not original-problem acceptance.** pse-arrow re-assesses in original
  coordinates. `Solved_To_Acceptable_Level` must not be read as solved `[interp, consistent
  with the pse-arrow design]`.

**6. Refusal / abandonment.**
- *Facts known before solving:*
  - nonsmooth functions or integer variables mean Ipopt is not admissible;
  - degenerate constraint structure calls for an explicit ℓ₁ penalty-barrier mode (POUNCE),
    not a silent fallback;
  - a known good primal-dual seed calls for `warm_start_init_point` with `mu_init` matched to
    the seed's complementarity `[known]`;
  - a nearly infeasible start with a small feasible region makes `start_with_resto` a
    declared option.
- *In-solve observations* from the `Intermediate_CB` fields (`alg_mod`, `inf_pr`, `inf_du`,
  `mu`, `d_norm`, `regularization_size`, `alpha_du`, `alpha_pr`, `ls_trials`) and from
  `GetIpoptCurrentIterate`/`GetIpoptCurrentViolations`, both used by pse-arrow
  `[read: IpStdCInterface.h; ipopt.rs]`:
  - `alg_mod = 1` means restoration was entered;
  - persistent `regularization_size > 0` means negative curvature, so SSOSC is doubtful and
    a later M1 is refused;
  - high `ls_trials` or tiny `alpha_pr` mean blocked steps;
  - stagnant `inf_pr` means infeasibility is not decreasing;
  - `mu` stalling.
- *Terminal statuses:* `Restoration_Failed`, `Infeasible_Problem_Detected` (local),
  `Search_Direction_Becomes_Too_Small`, `Maximum_Iterations_Exceeded`, and diverging iterates
  (above 1e20).

**7. Scaling.**
- Gradient-based NLP scaling caps gradients at 100, with a minimum factor of 1e-8.
- `tol` is scaled while the component tolerances are unscaled.
- κ_d = 1e-5 damps one-sided bounds, and `slack_move` handles tiny slacks.
- The final point may violate bounds by up to the relaxation factor, which acceptance must
  handle `[read]`.

**8. Cost.**
- Each iteration costs one evaluation of f, c, ∇f, J and the Hessian, plus at least one
  factorization (more under inertia correction) and backsolves.
- Line-search trials and SOC cost function evaluations only.
- Restoration is an inner IPM.
- Interior-point warm starts are usually much weaker than active-set warm starts. Bound push
  moves active variables off their bounds, and a mismatched `mu_init` re-centres the iterate
  `[known]`.
- Kungurtsev–Diehl make the general point that standard globalization can prevent a solver
  from exploiting good warm starts `[abstract]`.

**9. Interactions.**
- M8 is the base solve for M1 and M2; the factor is retained by POUNCE or sIPOPT sensitivity
  and has to be free of regularization (`sens_max_pdpert`).
- It is the corrector for M1, M1b and M3-style predictions through warm starts.
- M5 enters through its quasi-Newton Hessians.
- M7 enters through its restoration and feasibility models.
- POUNCE active-set SQP is the alternative corrector when the working set is known.

**10. Library realizations; minimum information.**
- Ipopt 3.14.20 C API (`CreateIpoptProblem`, `IpoptSolve`, `SetIntermediateCallback`,
  `GetIpoptCurrentIterate`/`Violations`), already used by pse-arrow.
- POUNCE:
  - `pounce-algorithm`, a port of the Ipopt algorithm (filter LS, SOC, watchdog,
    μ-updates, inertia correction);
  - `pounce-restoration`;
  - `pounce-l1penalty`, whose verdicts are judged on the **original** model's feasibility;
  - active-set SQP with filter or ℓ₁ merit, and IPM→SQP `WorkingSet` warm start
    `[read: READMEs, sqp/*.rs]`.
- *Minimum information:*
  - exact derivatives and scaling;
  - initial primal-dual values with provenance;
  - a warm-start policy (`mu_init`, push values);
  - per-iteration callback records;
  - typed status mapping;
  - original-coordinate re-assessment.

**11. References / evidence.**
- Wächter & Biegler (2006) *Math. Prog.* 106:25–57, doi:10.1007/s10107-004-0559-y — the paper
  was **not re-read** (the IBM preprint download timed out). Algorithm semantics and defaults
  come from the Ipopt 3.14.20 source option registrations, which cite its equations `[read]`.
- Ipopt OPTIONS documentation — `[read]`.
- Wächter & Biegler (2005) *SIAM J. Optim.* 16:1–31 and 32–48 — `[known]`.
- POUNCE READMEs and source — `[read]`.

---

## Cross-cutting contracts implied by M1–M8 `[interp unless marked]`

**C1. A predicted start is a seed, with validity limits.** Every predictor (M1, M1b, M2, M3,
M7 output, and the AA or Picard iterate when used only as a start) produces a seed. A seed
carries:
- *Provenance:* the mechanism, the base-point stamp, Δp, and the approximation order with its
  error indicator.
- *Validity assumptions:* the active set held fixed (M1), SC assumed or not, the parameter
  radius used, and the branch identity.
- *Permission:* it may be used as an initial iterate. It is never a result.
- *Transport class:* primal values transfer by semantic identity. Duals, working sets, bases
  and factors transfer only to a compatible solver family, with matching layout, sign
  conventions and scaling (the POUNCE `natural_units_factor` lesson).

**C2. Retained state needs a compatibility stamp.** The stamp covers:
- structure: variable and row identity, sparsity and ordering;
- scaling;
- the parameter set and the values at formation;
- backend;
- the point at which it was formed (KINSOL's Jacobian may be from an earlier iterate);
- the accuracy regime: μ, δ_w, δ_c, tolerance and Jacobian age;
- validity indicators: inertia, numerical rank, activity margins.

Reuse is permitted only when the stamp matches. Numerical staleness is acceptable, because
in-solve monitors handle it; structural mismatch is not.

**C3. Intermediate accuracy and final accuracy are separate.** Forcing terms η, predictor error
budgets, one-iteration stages (RTI or Euler–Newton), AA tolerances and trust-region stopping
apply only to intermediate work. The final numerical policy and the original-problem
assessment never change.

**C4. In-solve trigger vocabulary (typed and recorded).**
- step contraction Θ = ‖Δx_{k+1}‖/‖Δx_k‖ and residual ratio ‖F_{k+1}‖/‖F_k‖;
- line-search backtracks, β-condition failures and step length;
- trust-region ratio ρ and radius Δ, plus no-progress counters;
- Krylov iterations and failures per nonlinear iteration;
- Jacobian and preconditioner age and refreshes;
- Ipopt `alg_mod`, `regularization_size`, `ls_trials` and α;
- predicted active-set changes and Schur pin residuals;
- the AA gain θ_k and the conditioning of R;
- restoration entry.

Each trigger maps to one of four actions: refresh, shrink (Δp or Δ), switch to a declared
mechanism, or abandon to a full solve. *Gap:* KINSOL exposes cumulative counters after it
returns. Per-iteration ratios need an info handler, print-level capture or bounded calls
`[read: kinsol.rs getters]`.

**C5. Facts used for admission before solving.**
- smoothness class, including where selectors and kinks are;
- derivative availability: exact Hessian, analytic JVP, F_p operator;
- structural regularity (matching) and numerical regularity at the base (rank, inertia,
  SC margins);
- parameter identity and normalized proximity |Δp|;
- whether retained state is available and its stamp;
- whether a declared fixed-point map or splitting exists;
- bounds and domain;
- cost ratios (Jacobian or factorization versus residual) taken from structure and callback
  metrics, not from benchmarks.

**C6. All work is charged.** This includes predictor evaluations, screening, rejected
predictions, refreshes, fix-relax refactors and auxiliary QP solves. A predictor is checked
against a plain warm start at the same evaluation cost: compare the residual at the
prediction with the residual of the unpredicted base point.

**C7. Ordered sequences are a first-class structure.** M2 (RTI or amsNMPC), M3 (secant) and
M1b (path following) all assume that problems are visited in an order that keeps successive
problems close. A parameter study or continuation should therefore expose its ordering, and
mark each problem as intermediate or final.

## Pitfalls

1. **Linearized active-set checks are screens, not nonlinear certificates.** sIPOPT and POUNCE
   both say the result "stays a predictor" for NLPs `[read]`.
2. **A prediction can select a different branch or local minimizer.** This happens at folds,
   with non-unique solutions and in nonconvex problems. The corrector may then converge
   elsewhere, so branch-identity checks are needed (the det F_x sign and the distance against
   the error model).
3. **A barrier factor is not the NLP KKT matrix.** The error is O(μ). Active bounds carry
   σ ≈ 1/μ, which corrupts releases at tight tolerances. A factor with δ_w > 0 invalidates
   sensitivity (`sens_max_pdpert`) `[read]`.
4. **Stale factors give inexact tangents.** KINSOL's held factor is not necessarily
   F_x(x\*) `[read]`.
5. **Anderson or Picard on an arbitrary residual is not admissible.** AA does not speed up
   quadratic iterations. KINSOL fixed-point and Picard modes have no domain or sign
   protection and no line search `[read]`.
6. **Forcing terms are irrelevant to direct solves.** KINSOL also turns residual monitoring
   off for Krylov solvers `[read]`.
7. **Model and auxiliary success is not original success.** LM can stop at least-squares
   stationary non-roots. Critical-cone QPs can be indefinite in full space, so HiGHS and
   Clarabel (PSD-only) are not general substitutes.
8. **Advanced step reduces latency, not work, in batch settings.** RTI's "use the approximate
   answer" is a control-loop property. It transfers only to intermediate stages.
9. **Globalized interior-point warm starts can waste a good prediction.** Bound push,
   `mu_init` mismatch and filter history all work against it. Warm-start parameters must
   match the seed's accuracy (Kungurtsev–Diehl's general warning) `[abstract]`.
10. **A small step is not convergence.** KINSOL's steptol exit may be a stall `[read]`.
11. **Primal and dual activity margins are different quantities.** Folding them into one
    `bound_eps` silently returns the wrong active set `[read: POUNCE]`.
12. **Every sensitivity mechanism here needs exact second derivatives** (M1, M1b and M2).
    Quasi-Newton Hessians (M5 for NLP) exclude them.

## Evidence limits

- **Read in full or in the relevant sections:**
  - sIPOPT (optimization-online preprint), §1–3;
  - Stechlinski–Jäschke–Barton sequential-QP preprint, introduction and Thm 1.1;
  - Suwartadi–Kungurtsev–Jäschke 2017, §1–3;
  - Diehl–Bock–Schlöder 2005 SICON, §3–5;
  - Biegler 2013 DYCOPS survey (the predecessor of Biegler–Yang–Fischer 2015);
  - Eisenstat–Walker 1994 (OCR partial) and 1996;
  - Knoll–Keyes 2004, §2–3 and §5.1;
  - Evans–Pollock–Rebholz–Xiao 2020;
  - Mao et al. 2016;
  - KINSOL 7.1.1 documentation plus vendored source;
  - Ipopt 3.14.20 source option registrations and the OPTIONS page;
  - MINPACK (fortran-lang) `hybrj` and `lmder` source; ZIB NLEQ1 source; diffsol-nl source;
  - POUNCE 0.12.0 `pounce-sens-core`, `sqp`, restoration and l1penalty READMEs and source.
- **Abstract or secondary only:**
  - Stechlinski–Khan–Barton 2018 (the MIT copy came back as HTML);
  - Zavala–Biegler 2009 (preprint font-garbled);
  - Zavala–Laird–Biegler 2008; Würth–Hannemann–Marquardt 2009;
  - Kungurtsev–Diehl 2014; Dontchev–Krastanov–Rockafellar–Veliov 2013;
  - Robinson 1980; Büskens–Maurer 2001; Walker–Ni 2011; Toth–Kelley 2015;
  - Brown–Saad 1990/1994; Dembo–Eisenstat–Steihaug 1982 (via EW restatements);
  - Fletcher et al. TR-SQP-filter, the Fletcher–Leyffer–Toint SLP-filter (PDF unreadable),
    and Chin–Fletcher; Yamashita–Fukushima.
- **Not re-read (`[known]`):**
  - Wächter–Biegler 2006 (the download timed out; source defaults were used instead);
  - Allgower–Georg (no open copy) and Deuflhard's book;
  - Kelley's FR16, Conn–Gould–Toint, Moré 1978, Sorensen 1982, Broyden–Dennis–Moré;
  - Dontchev–Rockafellar's book; Keller.
- **DOIs from memory rather than from a page I saw:** Zavala–Biegler 2009,
  Zavala–Laird–Biegler 2008, Evans–Pollock–Rebholz–Xiao 2020, Toth–Kelley 2015.
- **Repository claims** (pse-arrow does not call the POUNCE boundcheck path functions, and the
  listed KINSOL setters are unwired) come from `grep` over `crates/` and
  `crates/pse-backend-native/src` at the review baseline. An empty grep does not rule out
  indirect use through other names.
- **No numerical experiments were run.** Cost statements are qualitative operation counts.
- **Open questions:**
  - whether KINSOL applies `fscale` inside the AA least squares;
  - whether `pounce-qp` accepts reduced-PD, full-space-indefinite Hessians for M1b QPs.
