> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Challenge X2 (design-reviewer): numerical-methods challenge of the first draft. Finding numbers it cites follow the first draft; see the README mapping. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# X2: numerical-methods challenge to the solver companion and principal F05–F09, §6.2–§6.3

- **Role.** An adversarial challenge, not a review. It carries no verdict, finding IDs or decision. The coordinator owns the disposition. Lettered facts (A–I) and item numbers (1–8) are local references only.
- **Subject.** `solver-companion-draft.md` in full. From `principal-draft.md`: F05–F09, §6.2 and §6.3.
- **Baseline.** `main` at `6c368c88a`. Static reading only: no build, test, probe or measurement. No network requests.
- **Inputs read.**
  - The scratch inputs R2, R4 and R5, each with its Part B file.
  - The prior solver evidence, mainly `method-landscape-and-selection.md` and `numerical-methods-challenge.md`, plus B1–B3 where they are cited below.
  - Profile PS-08–PS-11 and core DP-11.
  - Blueprint §17.1, §17.6 and §19.3 (the study policy).
  - The pinned library sources: Ipopt 3.14.20 in the skill corpus; SUNDIALS 7.1.1 vendored in `sundials-sys-0.6.2`; POUNCE at `ff63869`; FERAL 0.18.0.
- **Labels.**
  - *Interface-checked*: the source was read and is named.
  - *Proposed*: every corrected statement.
  - *Recall*: literature knowledge not re-checked here.

## 0. Verdicts at a glance

| # | Draft statement (short) | Verdict | Corrected statement (one line; detail below) |
|---|---|---|---|
| 1 | Lanes keep sessions warm across re-solves and same-lane study points; recording provenance satisfies PS-11 | **Wrong as worded** | Allocation reuse can be invisible to results. Numerical warm state may flow only along *declared* seed edges or a declared session start policy, never along whatever order a dynamic lane queue produces. Several "retained" reuses are not retained at the pins (facts A, B) |
| 2 | Per-block rules (affine, Brent, certified root with branch chosen by bounds, `block_solve` ≤ 8, KLU/Krylov+BTF, VI, TR/NLEQ-ERR) with a final simultaneous polish | **Needs conditions; three rules are wrong** | Branch choice belongs to the authored rule, not the bounds. Krylov with BTF preconditioning is vacuous inside one SCC. NLEQ-ERR is not a near-singular remedy. `block_solve` is unscaled presolve tooling (fact E). Run the polish only when global verification fails |
| 3 | Wavefront blocks, shooting windows and fit experiments in parallel, deterministic by fixed level, commit and reduction order | **Needs conditions** | Fixed order is not enough. Results are width-invariant only with level-synchronous semantics, block-owned (not thread-cached) numeric state, fixed inner width and per-block work budgets |
| 4 | Under multiple lanes, prefer SPRAL/Pardiso/FERAL over MUMPS; the curvature test when inertia is unreliable, with the δw change recorded | **Wrong in framing, conditional in content** | Concurrency class decides *placement*, not method. MUMPS is serialized but safe. Pardiso fits only convex problems or runs with the curvature test. FERAL inertia holds only when `n_tiny` = 0 (fact F). The curvature test changes what δw *and* inertia mean |
| 5 | KINSOL fixed point with damping and Anderson by default; a contraction diagnosis leads to a simultaneous SCC Newton step | **Needs conditions** | Anderson needs safeguards KINSOL 7.1.1 lacks. Record history from the map callback, not KINSOL's norm (fact G). The successor depends on the cause and on how units are represented |
| 6 | One integrator memory per transient experiment, reinitialized; forward or adjoint by the parameter-to-output ratio | **Needs conditions; the rule is mis-stated** | Reinitialization keeps allocations, not KLU symbolic analysis (fact B). It is IDAS-only, because event-crossing fits run on Diffsol. The forward/adjoint choice is coupled to the algorithm: Gauss–Newton/LM needs forward sensitivities |
| 7 | History: lexicographic score, then PAR-k; failures right-censored; one frozen snapshot per study | **Needs conditions; censoring is mis-stated** | Only budget exhaustion is censored. Records must be conditioned on plan position and start kind, or history is biased toward the incumbent. Reproducibility rests on the recorded effective plan, not on recomputing the aggregate |
| 8 | Stop-fidelity table; a policy-stopped iterate is a start proposal | **Mostly correct; three rows need correction** | KINSOL Newton keeps the last accepted iterate (fact G). Ipopt stopped during restoration returns the iterate from before restoration (fact H). Anderson-type iterates can leave the domain. Clarabel and HiGHS-IPM iterates are not general starts |

## 1. Source facts that change the drafts (all *Interface-checked*)

**A. Ipopt 3.14.20 repeats the linear solver's symbolic analysis on every `OptimizeTNLP`/`ReOptimizeTNLP`, for every HSL-free solver.**
- `IpoptApplication::call_optimize` re-initializes the algorithm (`Interfaces/IpIpoptApplication.cpp` ~637). `TSymLinearSolver::InitializeImpl` calls `solver_interface_->Initialize` (`IpTSymLinearSolver.cpp:80`) and sets `initialized_ = false` "to make sure that InitializeStructure is called" (`:129-131`). Each solver then discards its analysis:
  - MUMPS: `InitializeImpl` sets `have_symbolic_factorization_ = false` (`IpMumpsSolverInterface.cpp:188`, ~217–220).
  - SPRAL: `InitializeStructure` always calls `spral_ssids_analyse` (`IpSpralSolverInterface.cpp:444-481`).
  - MKL Pardiso: `InitializeImpl` releases all memory with phase −1 and resets (`IpPardisoMKLSolverInterface.cpp:190`, ~216–233).
- `warm_start_same_structure` keeps only the TNLP adapter's index maps and allocations.
- This contradicts B1 ("ReOptimizeTNLP … saves the symbolic analysis regardless"), R4 Part B §1.1 ("KKT structure … retained") and the principal F06 verification ("a second solve … performs no symbolic analysis").
- At the pins, symbolic reuse across solves comes from POUNCE/FERAL (`pattern_reused`), Clarabel `update_*`, HiGHS, and the project's own factorizations. Ipopt does not provide it.

**B. SUNDIALS 7.1.1 repeats KLU symbolic analysis at every `KINSol` call and after every `IDAReInit`.**
- **KINSOL:** `KINSol` → `KINSolInit` → `kin_linit` (`kinsol.c:565, 1601-1603`) → `SUNLinSolInitialize_KLU`, which sets `first_factorize = 1` (`sunlinsol_klu.c:231-238`). The next setup frees `SYMBOLIC`, runs `klu_analyze` again and factors with full pivoting (`:255-281`).
- **IDAS:** `IDAReInit` sets `ida_SetupDone = SUNFALSE` (`idas.c:856`). The next `IDASolve` or `IDACalcIC` runs `IDAInitialSetup` (`idas.c:2608-2610`; `idas_ic.c:147`), which calls `ida_linit` (`idas.c:5129-5131`) and leads to the same reset.
- Symbolic analysis is kept only *within* one solve or integration segment. This corrects B2 (`nonlinear-and-linear-solver-capabilities.md`, "reuses the symbolic analysis"), B3 ("IDAReInit keeps … KLU's symbolic analysis"; "reused across segments") and R4 Part B §1.1.
- Retaining it across solves would need a custom `SUNLinearSolver`. The precedent is `BN/dynamics/linear.rs` for Diffsol.

**C. Retained KINSOL sessions carry *numeric* state across points and threads.**
- `replace()` keeps `setup_usable`, so the next solve runs with `KINSetNoInitSetup(TRUE)` and the previous point's factorization whenever `ReusePolicy ≠ Fresh` (`BN/kinsol.rs:1286-1290, 1334`).
- The run records this as `input_stale`.
- The implicit inner-session cache is `thread_local!` and keyed by content (`BN/implicit.rs:124-160`).
- Consequence: the iteration path depends on what that *thread* solved before.

**D. A POUNCE batch is outer-parallel with *fresh* applications.**
- Each instance builds its own `IpoptApplication` on the worker. "No cross-instance KKT-structure sharing is attempted" (`pounce-algorithm/src/batch.rs` module docs).
- `*_warm` seeds come from an index-aligned `warms` vector, so they are deterministic.
- Batches and lane-retained `TnlpPresolveSession`s are therefore **alternatives**, not complements.
- The upstream doc line "a warm start only affects an instance's iteration count, not its solution" is false for nonconvex NLPs (*Recall*). Do not adopt it.

**E. pounce `block_solve` is presolve tooling, not a qualified block solver** (`pounce-presolve/src/block_solve.rs:17-46, 185-300`). It:
- stops on an absolute, unscaled ‖F‖∞ < 1e-8;
- treats any pivot below 1e-14 (absolute) as singular;
- aborts on *any* failed trial evaluation (`EvalFailed`), so it cannot back off from a domain error;
- checks bounds only after convergence;
- reports only an iteration count.

**F. FERAL's inertia can be the inertia of a perturbed matrix.**
- POUNCE inherits FERAL's default, which arms cascade-break (ratio 0.5, ε 1e-10) (`pounce-feral/src/lib.rs:206-217, 813-825`). Heavy-delay supernodes are then force-accepted and perturbed.
- `n_tiny` counts those perturbations and "does not affect inertia" (`feral numeric/solver.rs:129-136`).
- FERAL's own C API advises thresholding min/max pivot "even when `feral_num_neg` reports the correct inertia" (`capi.rs:502-505`).

**G. What KINSOL leaves behind when it stops early.**
- **Newton and line search:** on an unrecoverable callback return, the loop breaks *before* `uu = unew`, so `uu` holds the last accepted iterate (`kinsol.c:655-705`).
- **Fixed point:** G is evaluated at `uu` at the top of the loop. A failure leaves `uu` at an iterate whose G is unknown (`:2856-2872`).
- **Fixed-point convergence test:** KINSOL tests ‖`unew` − `uu`‖, which under Anderson is the *accelerated step*, not ‖G(u) − u‖ (`:2919-2939`).
- **Positive (recoverable) returns are ignored in fixed-point mode.** The project already latches failures negative (`BN/kinsol.rs` residual callback comment). Keep that.

**H. An Ipopt user stop during restoration returns the iterate from before restoration.** `RestoMinC_1Nrm` rethrows `RESTORATION_USER_STOP` before copying the restoration iterate back (`IpRestoMinC_1Nrm.cpp:321-325`). The outer loop maps it to `USER_REQUESTED_STOP` (`IpIpoptAlg.cpp:532-537`). The restoration progress and the p/n slacks are lost.

**I. `warm_start_same_structure` does not recompute Ipopt's fixed-variable map.**
- The whole structure block is skipped (`IpTNLPAdapter.cpp:400-1358`).
- `GetBoundsInformation` then reuses `x_fixed_map_`, checked only by a `DBG_ASSERT` (`:1411-1418`).
- If a bound edit changes which variables have lb = ub, the retained session silently solves a different problem.
- Today's bridge copies bounds when a sequence is created (`BN/ipopt/sequence.rs` ~436), which makes this safe. Lane retention across bound edits must not lose that safety.

## 2. Items 1–8

### 1. Warm reuse on lane-retained sessions (principal F06, §6.2, §6.3; companion §3.7, §9)

**Draft.**
- "Re-solves and same-lane study points reuse the session warm."
- "Lanes pulling points from one queue … a lane keeps sessions warm."
- PS-11 is "met, with the forward condition" that warm starts are recorded.

**Verdict: wrong as worded.**

**Why.**
- **The draft merges two reuse kinds that the current contract keeps apart.** §17.6 says "Numerical start policy … is independent of native allocation reuse … Reusing a retained native model never implies a warm start." §19.3 lets a seed flow only along a declared dependency ("Continuation defaults to a usable compatible predecessor seed"). Independent occurrences start as declared.
- **Recording is not declaring.** If warm state follows a dynamic queue, the chain of predecessors (which point warm-starts which) is decided by completion order and lane count. For a nonconvex NLP, a system with several roots, or a cubic/implicit property near a phase boundary, the *local solution reached* then depends on scheduling (*Recall*: continuation tracks the current branch; a cold start from the declared start can converge to another one). PS-11's MUST ("a declared dependency") and DP-11 ("record when … thread count can change results") are then breached for the study as a whole, even if each record is truthful.
- **The draft contradicts itself.** Companion §3.5 freezes the history snapshot "so point results do not depend on completion order", yet §3.7 makes them depend on it through warm starts.
- **Facts A–C cut the claimed benefit.**
  - Ipopt and SUNDIALS-KLU redo symbolic analysis on every solve.
  - What KINSOL actually carries across is a *stale numeric factor* (C), which is history-dependent by construction.

**When warm state or a retained factor is mathematically invalid** (all *Proposed*):
- **Pattern change, including a change in the solver's internal pattern.**
  - Fixed/free status, row set, or the Ipopt fixed-variable set when lb = ub is edited (fact I).
  - A POUNCE presolve transform: FBBT and redundancy reductions depend on values and bounds. Reuse only when `PresolveFingerprint` matches.
  - A KLU refactor needs the identical pattern, explicit zeros included. `kinsol/block.rs` keeps explicit zeros; keep that.
- **Pivot-sequence staleness.**
  - A refactor along an old pivot order is a value-dependent heuristic. SUNDIALS falls back to full pivoting only when rcond < ε^(2/3) *and* condest > ε^(−2/3) (`sunlinsol_klu.c:302-327`), a permissive guard.
  - Across study points, reset to fresh pivoting or guard on growth.
- **Bound or active-set change.**
  - An interior-point seed outside the new box must be pushed back in. Use small `mu_init`, monotone μ and paired pushes (prior landscape §4), or the warm start does little.
  - An SQP working set inherited across a bound change must be reconciled by the library or dropped.
  - A HiGHS basis stays a valid basis after bound or cost edits, but row or column additions need it extended.
- **Branch change in an implicit or cubic property.**
  - A warm start can follow a *metastable* root into a region where another phase is stable. The point is numerically converged but physically wrong.
  - Branch selection must re-run the authored rule (lowest Gibbs energy or the declared phase) at every point. Independent assessment must check the branch, not only residuals (PS-10).
  - The same applies across regime boundaries of piecewise correlations.

**Corrected statement** (*Proposed*).
- **Lanes own allocation reuse** (memory, contexts, compiled programs, and symbolic analysis where the library keeps it: POUNCE/FERAL, Clarabel, HiGHS, project factorizations). This reuse must leave results unchanged: numeric retained state (`NoInitSetup` factors, old pivot orders, quasi-Newton memory, HiGHS basis) is reset unless the start policy allows it.
- **Numerical warm state flows only along declared edges.**
  - In studies, along the study policy's seed edges. To gain warm benefit in sweeps, the *study definition* declares continuation chains, for example along an ordered parameter path. The chain partition is derived from the definition alone, never from the available lanes. Lanes execute chains; a chain is the unit of lane affinity and of durable grouped claims.
  - In interactive use, along a declared session `StartPolicy` (for example `PreviousAccepted`), recorded per result.
- PS-11 is then met by construction. As drafted, the forward condition is insufficient.

### 2. Per-block plans over BTF (companion §2 square row, §3.2 per-block rules; principal F08)

**Verdict: needs conditions; three rules are wrong.** Each rule:

| Rule | Corrected statement (*Proposed*) |
|---|---|
| Affine → sparse LU, "zero Newton iterations" | Exactly one linear solve, which is one Newton step. It needs value-lifetime nonsingularity and relative pivot magnitude at the current upstream values. The result gets a residual check or one refinement step. Affinity is a body-lifetime fact; nonsingularity is a value-lifetime fact |
| 1×1 bracketed → Brent | Admissible only after a *verified* sign change on an interval where f is continuous and evaluable. Bounds propose the interval and may themselves be outside the domain. Brent stops on bracket width, not residual: acceptance must re-evaluate the scaled residual, or the x-tolerance must be derived from it via \|f′\|. It returns *a* root |
| 1×1 polynomial → certified isolation, "branch chosen by bounds" | **Wrong.** Several roots usually lie inside physical bounds, for example the compressibility factor Z. Bounds only exclude roots. The branch comes from the authored phase/branch rule. Isolation certifies the polynomial with rationalized floating coefficients; near a double root (critical point, spinodal) the *count* of real roots is ill-conditioned in those coefficients |
| Small dense (≤ 8) → pounce `block_solve` | **Condition (fact E).** As is, it is unscaled and domain-intolerant. Use KINSOL Dense with scaling, recoverable evaluation errors and a session retained per block (allocation reuse removes the setup-cost argument), or argmin `Newton`/`TrustRegion`. If `block_solve` is kept as a cheap first try, the callbacks must be nominal-scaled and fall back on any error |
| Large smooth SCC → Newton+KLU or "Krylov with BTF preconditioning" | **Second half wrong.** An SCC is irreducible, so its BTF is one block, and the BTF-block preconditioner (`kinsol/block.rs`: "library BTF diagonal blocks … right preconditioner") becomes the full LU. That preconditioner pays on the *whole* system. Krylov on one SCC needs a cheaper preconditioner over a partition *inside* it (unit or subsystem block Jacobi, ILU) or NASM (prior challenge §2.3) |
| Two-sided bounds with declared complementarity → VI | Correct as amended by the prior challenge: pairing and sign are declared; NLEQ-ERR works only with SSLS. Most two-sided bounds are domain guards; truncation or a variable transform comes first |
| Near-singular → TR or NLEQ-ERR | **NLEQ-ERR half wrong.** NLEQ-ERR is affine-covariant damping. At near-singular J its Θ monitor *detects* failure but the corrections stay huge. The near-singular remedies are TR/LM regularization, rank-reduction Newton (NLEQ2, not pinned) and rescaling first. "Near-singular" is a start or iterate fact (scaled condition estimate after the first factorization), not a pattern or definition feature |

**Escalation and polish.**
- **Can per-block solving return a point the simultaneous check rejects? Yes.** The assembled residual equals the block residuals, but acceptance can still differ when:
  - global acceptance uses an aggregate norm or closure (a per-block ∞-norm tolerance does not bound a 2-norm over N blocks, nor a balance closure that sums many rows);
  - block acceptance used block-local solver scaling rather than the global nominals (§17.1 derives KINSOL scales per block);
  - a block returned a bounded-least-squares or VI point, which is not a root and must never commit;
  - an upstream block committed a root branch that the global branch or domain checks reject.
- **The reverse also occurs.** A simultaneous solve can succeed where the sequence fails, because an early multi-root block committed a branch that leaves a later block without a root (prior landscape §2.1).
- **Corrected rule.**
  - Accept per block on the *same* original-space, nominal-scaled, row-wise criterion that global verification applies to those rows. Solver-internal scaling may differ; acceptance may not.
  - Run global verification on the assembled point. Run the polish *only if it fails*, as the same square Newton (not an NLP with `bound_push`, which moves a converged point off its bounds) started from the assembled point.
  - If the polish runs, the polished state is the result and the per-block commits are its provenance.
  - Escalation from a block failure is cause-typed per prior landscape §9, never "next rung".

### 3. Parallel wavefronts, shooting windows and fit experiments (companion §3.7; principal F07)

**Draft.** "Deterministic level order and commit order"; "deterministic reduction".

**Verdict: needs conditions.** Fixed commit order is necessary but not sufficient. Parallel results equal the sequential reference (bitwise, at fixed library width) only if (*Proposed*):

1. **Semantics are level-synchronous.** A level runs to completion before any stop or escalation, in sequential runs too. Otherwise a failure at block *k* commits a different sibling set in parallel than in sequence, and the escalation starts from a different point.
2. **Per-block numeric state is block-owned, not thread-cached.** The thread-local KINSOL cache with stale-setup reuse (fact C) makes iteration paths depend on work-stealing assignment. Key retained sessions to block identity, or use fresh numerics.
3. **Inner library width is a recorded profile input, fixed per profile.** It must not be "derived from the lane width at entry" (companion §3.7; principal §6.3). faer flags `Par` results as non-portable across thread counts (R2 F6). Pardiso and SPRAL invariance across widths is not established. DP-11 requires the width to be part of what the result depends on; contention must not choose it.
4. **Work budgets are allocated per block or per level deterministically.** A shared iteration or time budget exhausts at a scheduling-dependent block. Time budgets are inherently non-reproducible and must be recorded as such.
5. **Reductions run in fixed index order after the join:** the objective, the gradient and JᵀJ. Stacked residual-Jacobian blocks and continuity constraints need no reduction.
6. **Thread affinity of native state is respected.**
   - POUNCE sessions are `!Send`.
   - SUNDIALS guarantees safety only for "a context per thread"; sequential migration is an open premise (R4 Part B §4).
   - Until it is settled, experiments and windows are assigned statically to team threads, not work-stolen.
7. **Adjoint windows each own their checkpoints.** A gradient from a partially run horizon is a different quantity and must not stand in for the full one.

### 4. KKT linear solver without HSL (companion §3.2, §3.8, §5; principal F07, §6.3)

**Verdict: wrong in framing, conditional in content.**
- **Concurrency class is placement, not method.**
  - Ipopt+MUMPS is *serialized-safe*: the mutex covers MUMPS init, analysis, factorization and solve (`IpMumpsSolverInterface.cpp:106, 240, 414, 479, 574, 671`), while evaluations and the rest of each Ipopt iteration run concurrently. It is not process-exclusive like PETSc or IBEX.
  - Choosing SPRAL under multiple lanes and MUMPS on one lane makes the same case's iterates, and for nonconvex problems possibly its local solution, depend on execution context. That is a G6/DP-11 defect even when recorded.
  - Corrected: choose the KKT solver from problem features and one context-independent default. Concurrency class then limits placement only: serial lanes, process lanes, or fewer concurrent MUMPS cases. A context-dependent choice is allowed only as an explicit, declared study profile.
- **Inertia reliability per solver.**
  - MUMPS (INFOG(12)) and SPRAL (threshold pivoting, no static perturbation) give usable inertia.
  - MKL Pardiso's perturbed pivots do not (prior B1). Admit it for convex problems, or for nonconvex ones only together with `neg_curv_test_tol` > 0.
  - FERAL/POUNCE inertia is trustworthy only when `n_tiny` = 0 (fact F). The typed `n_tiny` is exactly the signal the plan needs.
- **What δw means.**
  - With `neg_curv_test_tol` > 0, Ipopt *ignores* wrong inertia and accepts directions that pass the Chiang–Zavala curvature test (`IpPDFullSpaceSolver.cpp:81-115`). δw > 0 then records curvature-test failures along the step, not inertia (*Recall* for the paper). Inertia is no longer a signal at all, and δc still signals singularity.
  - The observation family must carry `inertia_mode ∈ {inertia, curvature_test}`.
  - δw magnitudes are not comparable across linear solvers, whose own scaling and pivot thresholds decide when correction triggers. History and feature bins must be partitioned by (linear solver, inertia mode). The profile key partly does this already.
- **Benefit claims.**
  - Fact A: a lane-retained Ipopt session still re-analyses (SPRAL also repeats its METIS/hwloc analyse cost per solve), so premise 9 (SPRAL overhead on small KKT systems) also covers *every* re-solve.
  - `dependency_detector=mumps` re-enters the MUMPS mutex even when the KKT solver is SPRAL.
  - SCIP's nested Ipopt inherits whatever linear solver is configured.
- **"Compensating levers" conflate two uses of MC19.**
  - SPRAL `ruiz`/`mc64` and MUMPS scaling replace MC19 only as *linear-system* scaling, which affects factorization accuracy.
  - The loss of Ipopt's equilibration *NLP* scaling, which changes the barrier path and the termination tests, is compensated only by `nlp_scaling_method=user-scaling` fed from PS-07 nominals (R4 §1.2). That makes the nominal contract load-bearing for NLP robustness without HSL.

### 5. Recycle defaults (companion §4; principal F09)

**Verdict: needs conditions.** The direction is right: PS-08 history first, then damping and Anderson rather than plain substitution.

**Corrections** (*Proposed*):
- **Record history from the map callback** as ‖G(x) − x‖ per tear and normalized by Err/Tol, plus the oscillation sign. Under Anderson, KINSOL's `fnorm` is the accelerated step (fact G), so it is neither a contraction estimate nor a convergence certificate. The project's original residual validator stays the acceptance.
- **The contraction diagnosis (ρ̄) is meaningful only over plain or damped iterations,** that is, the Anderson delay window or a probe window. After Anderson starts, residual ratios measure Anderson's own rate. Diagnose Anderson by stagnation over m iterations instead.
- **Safeguard the default.**
  - Anderson coefficients can be negative, so accelerated iterates can leave unit domains (negative flows). KINSOL 7.1.1 has no restart or fallback on residual increase or G failure (fact G: a fixed-point callback failure is terminal).
  - A safe default is "damped fixed point with Anderson(m) after a delay, plus a project-owned restart rule". On a G failure or a residual increase, re-enter KINSOL from the last good iterate with Anderson memory cleared and stronger damping. That is bounded composition under PS-09.
  - Damped plain fixed point stays in the convex hull of u and G(u) and is the domain-safer rung.
- **Precondition for Anderson or Broyden: G must be a function of x.** Unit solves inside G must not warm-start from the previous sweep beyond the noise floor. Otherwise the secant information is corrupted. The noise floor caps attainable accuracy and calls for tighter inner tolerances, not a method change.
- **The successor is cause-typed and depends on the representation.**
  - Oscillation: more damping.
  - Slow convergence: deeper Anderson, then the simultaneous route.
  - Noise floor reached: tighten inner tolerances.
  - Divergence: Anderson may still converge (prior challenge, correction 5); otherwise the simultaneous route.
  - "Simultaneous SCC Newton" exists only for equation-oriented units. For opaque causal units the successor is Broyden on x − G(x), or a finite-difference tear Jacobian with Broyden for small tear sets, or JFNK under the four prior conditions.

### 6. Transient fits (companion §2 fitting row, §3.2; principal F06)

**Verdict: needs conditions; the rule is mis-stated.**
- **Reinitialization per experiment.** Valid for sensitivities and adjoints when each oracle call:
  - restores t₀ and y₀(θ) and reruns `IDACalcIC` under the new θ;
  - calls `IDASensReInit` with *fresh* s₀ = ∂y₀/∂θ, never the previous call's terminal sensitivities;
  - calls `IDAAdjReInit` and records a full forward pass with checkpoints (`BN/dynamics/idas.rs` already restarts this way at segment boundaries);
  - re-applies the profile's tolerances and options.
- **What it saves.** Allocation, contexts and evaluator workers, which is the real gain over today's per-call construction (W-B 3). It does not save KLU symbolic analysis at 7.1.1 (fact B).
- **Scope.** IDAS forward sensitivities refuse to cross events (`BN/dynamics.rs:1686`), and adjoints cross neither events nor resets (`:1486`). Event-crossing transient fits therefore run on Diffsol, which rebuilds its solver per θ because the solver borrows the problem. For Diffsol, "one integrator per experiment" means retaining the problem and the project `LinearSolver` (R4 Part B §1.1). The principal's F06 verification ("allocates one integrator per experiment") holds for IDAS only.
- **Oracle purity.** The oracle must stay a function of θ: nothing carried between calls may change rounding, such as a retained pivot order or stale factors. The NLP line search and derivative checks rely on f(θ) being reproducible at the same θ.
- **Forward versus adjoint.**
  - Gauss–Newton and LM need the residual Jacobian (n_obs × p), which forces forward sensitivities whatever the output count.
  - An adjoint returns the gradient of one scalar objective. It pairs with L-BFGS or a gradient-only method and pays when p is large.
  - Corrected rule: the choice is (algorithm class, derivative mode) jointly. Compare p·c_fwd with q·c_adj only within gradient-only algorithms, where replay is part of c_adj. Events force forward. An exact Hessian needs the second-order forward-over-adjoint route.
  - Companion §3.2 states the joint form correctly; the stage-column wording "by the parameter-to-output ratio" does not.

### 7. History layer (companion §3.5; principal F08, §6.4)

**Verdict: needs conditions; censoring is mis-stated.**
- **Censoring.** Only budget-exhausted attempts are right-censored. A terminal failure (locally infeasible, restoration failed) has a known runtime and is an unsuccessful outcome, not a censored one. PAR-k assumes a common cutoff. With per-attempt budgets that differ (deadlines, cooperative stops), compare only records with equal budgets, or use a censoring-aware estimator.
- **Work units.** "Machine-independent work" is comparable only within a method family (iterations and evaluations mean different things for IPM, Newton and fixed point). Across families, use wall time inside one environment partition.
- **Selection bias.**
  - Successors are observed only on instances where their predecessor failed, which are harder instances. Their scores are therefore biased low, and history can never promote them.
  - Records must be conditioned on plan position and start kind (cold, warm, predictor).
  - Below a minimum sample per bin, differences count as ties and the layer-1 order stands.
  - Otherwise the advisory layer is a self-reinforcing incumbent bias. It stays bounded only because it may permute within a tier.
- **Reproducibility.**
  - Replay rests on the *recorded* permutation and tunings, which companion §3.1 already records per attempt. Recomputing the aggregate is not the mechanism: DataFusion's parallel aggregation does not fix floating-point summation order, so near-ties can flip on recomputation.
  - Break ties by a tolerance-based rule that falls back to layer-1 order.
  - Pin the snapshot for the study's whole life, resumption included. Retention must keep either the referenced log prefix or a materialized advice table.
  - Interactive re-solves need a session-frozen snapshot too. Otherwise unchanged inputs can pick a different method as history grows.

### 8. Cooperative-stop fidelity (companion §3.4; principal F10)

**Verdict: mostly correct; three rows need correction.** "A policy-stopped iterate is a start proposal, never a result (PS-10)" is right. The translation must say what transfers. Primal x transfers when it passes the in-bounds and domain screen. Multipliers transfer only within the same family. IPM barrier state transfers only to an IPM with a matching μ. A basis transfers only to simplex. A trust-region radius transfers only within the same SNES.

| Library | Correction (*Proposed*, from *Interface-checked* facts) |
|---|---|
| Ipopt | The iterate is the last accepted filter iterate. **If stopped during restoration, it is the point where restoration began** (fact H). The ℓ1 slacks are lost, which matters to F11's attribution. A diagnostic stop should wait for restoration to exit, or the observer must snapshot slacks per restoration iteration |
| KINSOL | **Better than "iterate uncertain".** In Newton and line-search mode, `uu` is the last accepted iterate (fact G). In fixed-point mode, `uu` is the current, possibly accelerated, iterate with G unevaluated. Counters remain readable |
| Anderson, NGMRES, QN (KINSOL fixed point, PETSc) | Accelerated iterates can lie outside bounds or domains. Screen and project them before proposing; fall back to the last iterate whose G succeeded |
| Clarabel | The HSDE iterate scaled by 1/τ is meaningless when τ → 0 (the infeasibility-certificate path). Clarabel has no warm start, so its iterate is a primal hint for other families only |
| HiGHS | Simplex: the *basis* is the transferable state, not the point. IPM without crossover: an interior point only. MIP: the incumbent plus the bound |
| SCIP | Resumable state (`SCIPsolve` again), not a start proposal. The incumbent is feasible within tolerances |
| IDAS, Diffsol | Accepted-step state at `tret` is consistent. Resuming needs `IDAReInit` (order 1), so a resumed trajectory differs within tolerance from an uninterrupted run, and is recorded as such |

For divergence causes, the successor's start should be the best-merit iterate, which the observer must track because no library does, or the original start. It should not be the last iterate.

## 3. Further corrections to F05–F09, §6.2 and §6.3

- **F05, form facts.** Form facts (affine, polynomial, closed-form solvable) cannot be "keyed by pattern identity". An expression edit that keeps incidence backdates the pattern but changes the form. Key form facts by (block, body programs), as companion §3.1 correctly calls them definition features. §6.2's "pattern products backdate when incidence is unchanged" must exclude form facts.
- **F05, symbolic factorization.** A Salsa-owned *symbolic factorization* is consumable only by project-owned factorizations: faer, direct KLU in `dynamics/linear.rs`, and FERAL in `kinsol/block.rs` and `conditioning.rs`.
  - Ipopt's solvers, SUNDIALS-KLU and FERAL inside POUNCE analyse internally. At most they accept an *ordering* (KLU `klu_analyze_given`; MUMPS ICNTL(7) = 1; SPRAL and Pardiso user permutations; *Recall* for the latter three).
  - So "one producer per fact" holds for structure facts and orderings, not for library-internal symbolic objects. Reuse of those comes only from library session retention, which facts A and B show is absent for Ipopt and for SUNDIALS-KLU across calls.
- **F07 and §6.3, "memory admitted per lane from pattern-fact estimates".** A project fill estimate bounds library memory only when the library uses the same ordering. Otherwise it is a heuristic (R2 F6: "an upper estimate, not an allocator bound").
- **§6.3 and companion §3.7 "Process lanes … PETSc NASM, ASPIN and field split".** In this MPIUNI build, separate processes parallelize *independent PETSc problems*. They do not parallelize the subdomains of one NASM or ASPIN solve, which needs MPI ranks inside one PETSc communicator (R4 §1.5). Structure-level PETSc parallelism is absent at this build, process lanes included.
- **Companion §3.2 "Uno filterSQP for small NLPs".** The bridge allows only L-BFGS with convex HiGHS QP subproblems (B1). A size-based preference has no rationale. Keep Uno as a second-opinion profile with different globalization.
- **Principal F07 "Sixteen concurrent NLP cases … queue on one mutex".** Overstated. Only MUMPS phases queue; how much that matters scales with the factorization share, which is unmeasured.

## 4. What the drafts get right (preserve)

- A cooperative stop as a start proposal, never a result.
- Per-block original-space commit and rollback (§17.1).
- VI only with declared complementarity.
- Typed `n_tiny` and κ₁ as observation.
- Batch time limits per instance.
- Explicit TLS propagation into structure tasks.
- History limited to permuting within a tier, with the effective plan recorded.
- The PS-08 recycle history as the first change, independent of architecture.
- The stop-fidelity concept.
- The concurrency-class inventory, as a placement input.

## 5. Uncertainties and search coverage

- **Not executed.** Every corrected statement is *Proposed*. Library behaviour beyond the cited lines is *Recall*: Chiang–Zavala semantics, metastable-root continuation, and Pardiso/SPRAL width invariance.
- **Premises that could change a correction.**
  1. Whether SUNDIALS permits sequential migration of a context between threads (item 3.6).
  2. Whether the project's Ipopt compatibility stamp includes the lb = ub set. The bridge freezes bounds at creation today; lane retention across bound edits was not traced.
  3. Whether units inside causal recycle maps warm-start internally (item 5). `BN/recycle.rs::sweep` calls `unit.evaluate`; unit internals were not read.
  4. FERAL 0.19 changes are unreviewed.
- **Coverage.**
  - Ipopt: `IpTSymLinearSolver.cpp`, the MUMPS, SPRAL and Pardiso-MKL interfaces, `IpTNLPAdapter.cpp`, `IpIpoptApplication.cpp`, `IpRestoMinC_1Nrm.cpp`, `IpIpoptAlg.cpp`.
  - SUNDIALS: `kinsol.c`, `idas.c`, `idas_ic.c`, `idas_ls.c`, `sunlinsol_klu.c`.
  - POUNCE: `pounce-presolve/block_solve.rs`, `pounce-algorithm/batch.rs`, `pounce-feral/lib.rs`.
  - FERAL 0.18.0: `numeric/solver.rs`, `capi.rs`.
  - Project: `BN/{kinsol.rs, implicit.rs, recycle.rs, kinsol/block.rs, dynamics.rs, dynamics/idas.rs, ipopt/sequence.rs}` and `pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`.
  - libgomp, MKL and SSIDS internals were not read.

**File written:** this file only.
