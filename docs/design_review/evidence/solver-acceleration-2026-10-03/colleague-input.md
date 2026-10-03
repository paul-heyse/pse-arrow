
# Contextual grounding for solver pipeline refinements

Yes. I would frame the work as **solver-pipeline acceleration and globalization**, not as “adding another solver.” The purpose is to make the existing unified solver kernel do less unnecessary numerical work and reach its high-fidelity solve from a materially better numerical state, while leaving the original mathematical problem and final qualification unchanged.

### 1. Functional targets

| Functional target | What we are actually trying to improve |
|---|---|
| **A. Initial-point quality** | Enter the high-fidelity nonlinear solve closer to a relevant root/KKT point, with better-scaled and more physically/numerically plausible state values. The goal is fewer globalization steps, restoration iterations, rejected trials, and failures caused by a poor cold start. |
| **B. Cross-solve information reuse** | Treat a solved nearby case as numerical information, not merely as a vector of previous values. Reuse sensitivities, KKT/Jacobian factors, multipliers, working sets, bases, continuation tangents, etc. so related cases can be **predicted** before correction. |
| **C. Expensive evaluation reduction** | Reduce calls to nonlinear residuals, thermodynamic/property evaluations, Jacobians, Hessians and derivative programs. In many PSE problems these can dominate runtime more than the nominal solver iteration itself. |
| **D. Factorization / linear-algebra reduction** | Avoid rebuilding and accurately solving essentially the same linearized system unnecessarily. Reuse factorizations where valid, lag Jacobians/preconditioners when justified, and use inexact Newton forcing so early linear solves are no more accurate than useful. |
| **E. Globalization / basin enlargement** | Make difficult models reliably reachable from mediocre starts. Continuation, bounded homotopy, pseudo-transient methods, trust regions and related mechanisms are principally about avoiding divergence, invalid-property regions, singularities and failed restoration—not merely shaving iterations from an already easy solve. |
| **F. Mathematical reduction of the solve** | Exploit matching/BTF, eliminable blocks, reduced-space structure and eventually nonlinear preconditioning so the expensive outer solve acts on the genuinely coupled problem rather than repeatedly solving everything monolithically. |
| **G. Approximation proportional to usefulness** | Permit cheap linear/QP/local models, lower-accuracy inner work or other approximations **only as intermediate numerical mechanisms**, with their cost and validity explicitly controlled. High fidelity is spent where it changes the final answer. |
| **H. Seamless automatic selection** | Ultimately make these mechanisms consequences of mathematical facts—structure, derivative availability, parameter proximity, conditioning, retained state, domain limits—not user-authored “try technique X” scripts. The solver pipeline should be able to decide that the best preparation is also **none**. |

The important invariant is that these mechanisms are intended to change **the route to the answer**, not the definition of the answer. Your existing architecture is already strong here because candidate acceptance is re-evaluated against the original problem in original coordinates rather than trusting the approximation or native solver status. 

So a concise objective statement for an agent would be:

> **Minimize total high-cost nonlinear work and enlarge the reliable convergence region by exploiting mathematical structure, prior-solve information, controlled approximations and adaptive numerical accuracy, while preserving the original problem, final numerical requirements and independent qualification.**

### 2. What appeared absent or not yet generalized in `pse-arrow`

The notable point from inspecting the repository was **not that the underlying ingredients were missing**. You already have an unusually large fraction of them: structural matching/DM/BTF and block initialization; typed warm starts and native-state reuse; adaptive authored homotopy; KINSOL inexact-Newton/Anderson controls; and KKT sensitivity with retained-factor advanced-step prediction.   

What looked missing was the **general acceleration layer tying those mechanisms together**.

Your retained KKT predictor, for example, is already capable of using one backsolve to predict a perturbed solution and testing active-set changes. But the architecture documents its consumer specifically as the rolling-horizon advanced-step controller; it did not appear to be a general mechanism automatically available to arbitrary compatible parameter studies, continuation steps or sequences.  

Likewise, your continuation framework is strong, but it currently advances **authored parameter/fixed-value overlays**. I did not see a generic mechanism for constructing mathematically derived residual homotopies, bounded homotopies, pseudo-transient paths, or other automatically generated easier problems from the original problem when cold-start globalization is needed. 

Your structural machinery currently provides decomposition and predecessor-first initialization. I did not see that structure generalized into **reduced-space execution or nonlinear preconditioning**, where qualified blocks become reusable numerical operators during the main solve rather than only an initialization schedule. The architecture itself correctly warns that block structure alone does not prove independent eliminability, so this would need additional mathematical qualification rather than simply turning every BTF block into an elimination. 

KINSOL exposes sophisticated controls—Eisenstat–Walker forcing terms, Krylov methods, preconditioners, Jacobian setup intervals, Anderson acceleration and damping—but these appear primarily as **selected method settings**, rather than an overarching evidence-driven policy deciding when a problem would benefit from direct KLU versus inexact Krylov, more aggressive reuse, Anderson, etc. 

And I did not see a generic **approximate-start / seed-proposal abstraction** broad enough to say: “this point came from sensitivity prediction / a local QP / homotopy / pseudo-transient evolution; it is legitimate as a starting point under these validity conditions, but it is not itself a qualified result.” Your current warm-start and `CandidateUse` semantics are deliberately rigorous, so I would add such a concept rather than overload existing result permission.  

That leads to what I think is the key framing for the agents:

**The codebase does not primarily need more solver integrations. It already has much of the machinery. The opportunity is to turn the existing solver, structural, sensitivity, continuation and reuse capabilities into a unified evidence-driven predictor / preparation / correction pipeline, and then fill the specific mathematical mechanisms that pipeline currently lacks.**

That should give them a useful starting hypothesis while still leaving them free to inspect the implementation and conclude that particular pieces are already more generalized than appeared from my review.




# Start of detailed technical content

There are several generalizable approaches that fit your requirements. The strongest opportunities are to **predict solutions from previously solved nearby problems, avoid oversolving early numerical steps, exploit structural reductions, and use safeguarded approximations when a good starting point is unavailable**.

Importantly, you have already implemented substantial parts of this. Your retained-KKT prediction mechanism is particularly close to the “cheaply estimate the answer, then converge precisely” capability you describe. The next architectural opportunity is to make these mechanisms consistently available through the shared execution policy—not to create another independent solver subsystem. 

I inspected the numerical-execution architecture and relevant implementation paths at commit `f0b9025`. This is a source-level assessment; I have not benchmarked these strategies against your fixtures.

## 1. What your existing architecture already provides

The repository is further along in this direction than the question’s premise might suggest.

| Existing capability | What I found | Implication for this proposal |
|---|---|---|
| **Structural initialization** | Matching, Dulmage–Mendelsohn decomposition, block-triangular structure, and predecessor-first conditional block solves. | You already have a general mathematical basis for reducing initialization to smaller problems, rather than writing unit-specific initialization scripts.  |
| **Adaptive continuation** | Authored stages and parameter homotopy run through one staged-sequence primitive, with bounded attempts, step reduction, reuse, and a final solve of the unchanged specification. | Extend the existing continuation machinery rather than introduce a separate workflow. The current mechanism changes declared parameter/fixed values; it is not a general automatically constructed residual homotopy.  |
| **Sensitivity-based prediction** | `pse-backend-native/src/kkt/advance.rs` retains a qualified parametric factor and predicts a new primal/dual point through a backsolve. | This is already a sophisticated cheap estimator. Its documented consumer is the rolling-horizon controller; broader use in compatible studies and continuation is a natural extension.  |
| **Inexact Newton and derivative reuse controls** | KINSOL settings expose Eisenstat–Walker forcing terms, Krylov methods, preconditioning, and the interval between linear setups. | Much of “do inexpensive work early, precise work late” belongs inside the native solver and already has an integration surface.  |
| **Typed warm starts and independent allocation reuse** | Primal, dual, basis, barrier, and working-set state have compatibility and transformation contracts. | You have the right foundation for reuse without conflating “same allocation,” “compatible initial point,” and “valid native warm start.”  |
| **Independent original-problem assessment** | Final candidate use is governed by original-coordinate feasibility, optimality evidence, physical checks, and typed permissions. | Approximation can remain an execution technique without becoming a second definition of correctness.  |

One constraint should remain central: your architecture assigns iterative numerical algorithms to libraries, while the project owns admission, representations, policies, transport, and qualification. I would preserve that separation throughout this work. 

## 2. The important correction to “treat everything as linear first”

For a nonlinear equation system

\[
F(x)=0,
\]

linearization at an initial point \(x_0\) produces

\[
F(x_0)+J(x_0)d=0,
\qquad x_{\text{trial}}=x_0+d.
\]

**That is a Newton step.** A Newton-based nonlinear solver already performs this operation, usually with safeguards. A separate preliminary linear solve may simply repeat its first expensive Jacobian evaluation and factorization. KINSOL’s mathematical formulation explicitly follows this pattern. :chatgpt-content-reference{index="8"}

Nor is the resulting point necessarily closer. For \(F(x)=x^2-2\), starting at \(x_0=0.1\), the linearized solution is \(10.05\)—much farther from the positive root than the starting point.

The useful distinction is therefore not just *linear versus nonlinear*. It is:

| Lever | What becomes cheaper |
|---|---|
| **Better prediction** | Fewer nonlinear iterations are needed because the initial point is better. |
| **Inexact numerical work** | Each early iteration uses less linear-algebra or derivative work. |
| **Lower-cost model evaluation** | Some expensive evaluations are replaced by controlled approximations. |

These are complementary, but they need different contracts.

Also, a process simulation solving \(F(x)=0\) is a **root problem**, not necessarily an optimization problem. A nonlinear program adds an objective and constraints. Your implementation already distinguishes regular-square root sensitivity from optimizing KKT analysis; that distinction should govern acceleration as well. 

## 3. The methods most applicable to your approach

### A. Sensitivity-based prediction: the strongest first investment for related solves

Suppose a solved system depends on parameters:

\[
F(x,p)=0.
\]

At a regular solution, differentiating gives

\[
F_x\,\Delta x=-F_p\,\Delta p.
\]

Thus, for a nearby parameter value, the predicted solution is

\[
x_{\text{pred}}=x^\star+\Delta x.
\]

For constrained optimization, the analogous calculation differentiates the appropriate optimality system and predicts both primal variables and multipliers. This is the basis of sensitivity-based warmstarting and advanced-step methods. :chatgpt-content-reference{index="10"}

**Why it can be particularly inexpensive:** when the relevant matrix factorization is retained, prediction requires backsolves rather than a new nonlinear solve. Your `Advance` implementation explicitly retains the factor, parameter identities, candidate, multipliers, Jacobian information, and bounds for this purpose. 

My recommendation is to generalize that existing capability to **compatible parameter studies and continuation steps**, with root problems using their separate implicit-response machinery. Eligibility should depend on supported parameter changes and mathematical compatibility—not merely two cases having similar filenames or dimensions.

A useful extension would combine the existing active-set screening with a normalized step-size limit, evaluation of the predicted point against the new original problem, and full correction when required.

Two qualifications matter:

**First, linearized active-set checks are not a nonlinear accuracy certificate.** Your predictor checks possible entering/leaving constraints using available local information. Passing those checks does not prove that the actual nonlinear solution remains on that active set.

**Second, active-set changes need not permanently exclude predictive methods.** Research on semiderivative and generalized-equation predictors handles some changes that ordinary differentiable sensitivity cannot, but under additional structural and regularity assumptions. I would treat that as a later capability, not weaken your current sensitivity qualification. :chatgpt-content-reference{index="12"}

### B. Inexact Newton: spend precision only where it is useful

An inexact Newton method does not solve each linearized system to high precision. Instead, it seeks a step satisfying a condition such as

\[
\|J_kd_k+F_k\|\leq \eta_k\|F_k\|.
\]

The forcing term \(\eta_k\) controls how accurately the linear system is solved. Eisenstat–Walker methods adapt it using nonlinear progress, avoiding excessive linear work when the current linear model does not justify it. Appropriate forcing-term behavior retains strong local convergence properties. The foundational reference is Eisenstat and Walker’s **“Choosing the Forcing Terms in an Inexact Newton Method” (1996)**. :chatgpt-content-reference{index="13"}

This is an especially good match for your preference for **mathematical rules instead of domain-specific tuning**: the controller responds to numerical progress, not whether the equations represent a heater, reactor, or separation unit.

You already expose `Eta::Choice1`, `Choice2`, and constant forcing terms. However, your default KINSOL linear method is **direct KLU**. Adaptive forcing terms principally matter when an iterative Krylov route is used; changing `eta` alone does not make a direct factorization cheaper. 

The broader family includes reusing Jacobians, refreshing preconditioners only when needed, and using approximate Hessians where appropriate. Your settings already expose several of these choices. My recommendation is to benchmark coherent native profiles rather than implement a second project-owned Newton controller.

**This mechanism changes intermediate work, not the original model or final acceptance tolerances.** It therefore has a particularly clean architectural fit.

### C. Structural reduction: solve less before approximating more

Your block structure is valuable not merely for producing a starting point, but for reducing the numerical problem itself.

The first opportunity is to distinguish **exactly solvable or eliminable structure** from structure that merely looks weakly coupled. Proven affine blocks can sometimes be solved exactly; genuinely triangular dependencies can be processed in order. Your current structural initialization already derives conditional blocks from the admitted equation structure. 

A more advanced extension is **reduced-space solving**. Partition variables into \(y,z\), with a subsystem

\[
F_1(y,z)=0.
\]

When \(F_{1,y}\) is nonsingular, it locally defines \(y=y(z)\), with

\[
\frac{dy}{dz}=-F_{1,y}^{-1}F_{1,z}.
\]

The outer solver can then work on a reduced system while the inner subsystem supplies values and derivatives. This is a mathematical construction; it need not rely on process-unit labels.

My recommendation is to use the existing structural witnesses to establish candidates for such treatment, then require the additional numerical and coupling qualifications.

Your documentation already makes the essential caveat: **block structure does not prove independent eliminability**, and structural matching does not prove numerical rank. Objective and inequality coupling also prevent treating independent-looking equation blocks as independent optimization problems. 

This is a higher-value foundation than indiscriminately linearizing every equation.

### D. Safeguarded local linear/quadratic initialization: the closest version of your proposed cold-start method

When there is no useful prior solution, a bounded local approximation can still be valuable.

For a normalized residual \(r(z)\), one possible auxiliary problem is

\[
\begin{aligned}
\min_{d,e}\quad&
\frac12\|e\|^2+\frac{\lambda}{2}\|d\|^2\\
\text{subject to}\quad&
e=r(z_0)+J(z_0)d,\\
&\ell\leq z_0+d\leq u,\\
&\|d\|_\infty\leq\Delta.
\end{aligned}
\]

This is a **regularized, bounded local least-squares model**: linearized residuals, a penalty on large corrections, original variable bounds, and a trust-region limit. The Levenberg–Marquardt family provides the classical foundation for stabilizing linearized least-squares steps. :chatgpt-content-reference{index="17"}

The lifted residual variable \(e\) also gives you a sparse QP representation without explicitly constructing \(J^\top J\).

For your architecture, I would treat this as a **derived auxiliary QP**, potentially executed through an existing compatible coefficient/conic route. It must not cause the original nonlinear problem to be reclassified as affine. Your current coefficient extraction deliberately requires exact affine/quadratic qualification; that distinction should remain intact. 

The trial point should be checked against the original evaluator. Trust-region model management compares actual improvement with predicted improvement and restricts steps when the approximation is unreliable; it does not assume that an accurately solved surrogate produces an accurate original solution. :chatgpt-content-reference{index="19"}

I would make this a **small, bounded preparation strategy**, not a compulsory phase. Important limits are:

- It needs an evaluable starting point. A Jacobian cannot generally be obtained at an invalid logarithm or failed opaque property evaluation.
- A stationary point of residual least squares need not be a root.
- For optimization, a feasibility-oriented start does not necessarily help the objective.
- Your POUNCE active-set SQP route already performs local quadratic optimization, so a separate QP initializer must demonstrate value beyond native SQP’s own first iterations. 

### E. Continuation and homotopy: make the difficult problem reachable

Continuation replaces one difficult solve with a path of related solves. Your current implementation already supports a parameter-based version: declared initialization endpoints, adaptive progress, transactional overlays, and a final solve of the original specification. 

The more general mathematical extension is to construct an auxiliary family. For a square normalized residual, one possible construction is

\[
H(z,t)=(1-t)A(z-z_0)+t\,r(z)=0,
\]

where \(A\) is nonsingular.

At \(t=0\), \(z_0\) is a known solution. At \(t=1\), the equations are the original problem. A predictor–corrector continuation method follows the solution path while adapting the step in \(t\).

This construction requires no assumption such as “ideal thermodynamics is sufficiently accurate.” Its assumptions are instead about the existence, regularity, and admissibility of the path. Continuation libraries such as LOCA explicitly address parameterized nonlinear solution families and their bifurcations. :chatgpt-content-reference{index="22"}

For `pse-arrow`, I would first strengthen **predictor-assisted continuation over existing declared parameters**. Automatically generated residual homotopies would be a separate, explicitly admitted realization because they change the equations along the path, unlike your current value-only overlays.

They are not universally cheap or successful: the path may encounter singularities, leave the admissible domain, or lead to a different solution branch. Intermediate path solutions must therefore remain auxiliary evidence, never substitutes for the final original-problem solve.

### F. Pseudo-transient continuation: useful, but less assumption-free than it appears

Pseudo-transient continuation introduces an artificial evolution toward a steady root. A typical correction has the form

\[
\left(\frac{M}{\Delta t}+J_k\right)d_k=-F_k.
\]

With small pseudo-time steps, the added term changes the correction substantially; with large steps, the method approaches Newton behavior. Kelley and Keyes’ **“Convergence Analysis of Pseudo-Transient Continuation” (1998)** establishes convergence for a general formulation under its assumptions. PETSc also provides a native pseudo-transient method. :chatgpt-content-reference{index="23"}

The caveat is important for your “domain-independent” requirement: **an arbitrary residual does not automatically define a stable artificial dynamical system**. Residual signs, the operator \(M\), and the attraction properties of the target root matter.

I would admit this when there is a defensible mathematical evolution or operator construction, rather than automatically wrapping every equation system in pseudo-time. It is a worthwhile robustness capability, but below sensitivity reuse and native inexact solving in my initial priority order.

### G. Nonlinear preconditioning and Anderson acceleration: use cheap solves throughout the solve

A cheap solver need not be used only once at the beginning. It can repeatedly improve the state before an outer method computes its next correction.

This is the central idea behind **nonlinear preconditioning and solver composition**: use local nonlinear solves, sweeps, or simpler iterations as components of a stronger outer algorithm. Brune, Knepley, Smith, and Tu’s **“Composing Scalable Nonlinear Algebraic Solvers”** develops a general framework for such combinations. :chatgpt-content-reference{index="24"}

For your codebase, this suggests extending conditional block solves from “initialization schedule” to an explicitly qualified **nonlinear preconditioning operator**. That would require a clear contract relating the operator to the original residual, along with derivative and failure semantics.

Anderson acceleration is a related opportunity for admitted fixed-point or Picard iterations. You already expose its history, damping, delay, and orthogonalization controls. Importantly, your implementation does not arbitrarily reinterpret any residual as a fixed-point map. I would preserve that requirement: the map or splitting must be justified, and success remains defined by the original equations.  

This becomes especially attractive when large models contain numerically difficult local blocks inside otherwise manageable coupling.

### H. Multifidelity and reduced-order models: valuable once evaluation cost justifies them

True multifidelity methods replace expensive evaluations with cheaper models: reduced-order representations, interpolation, response surfaces, coarser discretizations, or other surrogates.

The critical principle is **high-fidelity model management**, not simply solving the cheap model and trusting its result. Trust-region approaches can correct the low-fidelity model to match high-fidelity values and derivatives locally, check proposed progress against the expensive model, and update the approximation. Peherstorfer, Willcox, and Gunzburger’s **“Survey of Multifidelity Methods in Uncertainty Propagation, Inference, and Optimization” (2018)** is a useful map of these approaches. :chatgpt-content-reference{index="27"}

For your current stage, I would build the interface for this but defer broad automatic surrogate construction. A small residual system may cost less to solve directly than to sample and approximate.

The important limitation is that **a general model-management framework does not automatically supply a cheap, accurate surrogate for every model**. Repeated expensive workloads are where the construction cost becomes more plausible.

Similarly, I would treat learned initial guesses as another producer of qualified seed proposals—not as a different authority over model semantics or result acceptance.

## 4. How I would integrate this into your pipeline

### A. Make acceleration a declared strategy, not a hidden fallback

Your current execution design deliberately disallows undeclared fallback engines and disables several hidden native retries. That is compatible with automation, but it means automation should be **authorized and resolved as a strategy**, not introduced as opportunistic switching after a solver fails.  

Conceptually:

```text
Immutable original problem + resolved numerical policy
                         │
              Strategy admission and planning
                         │
       ┌─────────────────┼──────────────────┐
       │                 │                  │
 Compatible          Structural       Bounded local
 prediction          preparation       approximation
       └─────────────────┼──────────────────┘
                         │
              Initial-point qualification
                         │
             Original-problem native solve
                         │
       Existing original-space completion assessment
```

Continuation or nonlinear preconditioning would be explicitly selected strategy forms—not extra boxes every solve must traverse.

The planner should select from mathematical facts: intent, smoothness, derivative availability, structure, compatible retained state, bounds, discrete domains, and supported transformations. It should also be able to select **no preparation**.

### B. Introduce an initial-point contract distinct from result acceptance

This is probably the most important new abstraction.

Your current `CandidateUse`, staged starts, and `PreviousAccepted` rules distinguish usable results from more restricted candidates. They should not silently be reinterpreted to mean that any approximate, infeasible point is an accepted result or ordinary accepted predecessor.  

I would introduce the conceptual equivalent of a **`SeedProposal`**—a proposed type, not an assertion that this API already exists—with:

| Contract facet | Required meaning |
|---|---|
| **Identity and coordinates** | Original problem identity, supported parameter change, semantic variable mapping, normalization, and any auxiliary-to-original transport. |
| **Numerical evidence** | Finiteness, bound/domain admissibility, original residual observations, prediction provenance, and applicable validity limits. |
| **Permission** | Allowed as an initial iterate; not asserted to satisfy the original problem or to be publishable as its result. |

An initial point can legitimately be infeasible with respect to equations. That is different from being outside a function’s domain, and different again from being a qualified solution.

This separation lets you use rough estimates without weakening final correctness.

### C. Preserve one model authority and label every auxiliary problem

A local QP, homotopy stage, reduced problem, and original model must remain distinguishable.

My proposed derived-problem contract would record the original identity, generation rule, anchor point, parameter values, coordinate maps, affected rows, and termination budget. It would be a disposable mathematical realization derived from the canonical model—not a second authored model.

In particular, **a QP’s success means the auxiliary QP succeeded**. It does not mean the original nonlinear system is feasible, stationary, or globally optimal.

### D. Keep cross-solver transport explicit

A predicted primal point can often be mapped by semantic variable identity. Native multipliers, basis information, working sets, and internal factors require stronger compatibility.

Your existing warm-start stamps include backend and layout, and your receipts record normalization and presolve transformations. A HiGHS-produced local-QP point therefore should not be blindly submitted as an Ipopt or KINSOL native warm start. Use an explicit primal initial-point transport unless the additional state has a separately justified mapping. 

### E. Separate intermediate accuracy from final accuracy

The original numerical policy should stay frozen.

A predictor may have an approximation-error budget; a Krylov solve has a forcing term; a continuation stage has a progress criterion. None should mutate the original problem’s final feasibility, stationarity, complementarity, or physical-check requirements.

Your current policy-derived native controls and independent completion assessment are already organized around this distinction. 

I would be especially cautious about making nested thermodynamic evaluations less accurate early. That needs an **inexact-evaluation contract**, including derivative consistency and error control. It should not be implemented by quietly loosening inner tolerances while advertising unchanged derivative accuracy.

### F. Charge everything to one work budget

The quantity to minimize is

\[
T_{\text{total}}
=
T_{\text{preparation}}
+
T_{\text{prediction}}
+
T_{\text{screening}}
+
T_{\text{correction}}.
\]

Reduced nonlinear iteration count alone is insufficient.

The strategy budget should include surrogate construction, compilation, matrix setup, rejected predictions, failed stages, and all qualification work. I would record those costs alongside the existing callback and attempt metrics, so policy decisions can be assessed without introducing another observability system. Your callback boundary already accounts for evaluation calls and time, including rejected trials. 

## 5. What “preserving accuracy” can and cannot mean

The right target is:

> **Preserve the original mathematical problem and its completion requirements while changing the computational route used to reach an acceptable candidate.**

That can avoid accepting a low-fidelity answer merely because it was inexpensive.

It does **not** guarantee that every accelerated solve reaches the same root or local optimum as an unaccelerated solve. Different initial points can select different branches. Nor does “global convergence” in much of the nonlinear-solver literature mean global optimization: the trust-region model-management paper explicitly uses it to describe convergence toward stationary points or local optimizers under its assumptions. :chatgpt-content-reference{index="35"}

For your platform, branch selection, local optimality, and global certification should therefore remain distinct requirements. Better initialization must not implicitly upgrade one into another.

## 6. My recommended implementation order

### First: operationalize what you already have

I would start with **broader compatible sensitivity prediction, explicit seed-proposal semantics, and measured native inexact/reuse profiles**.

This is the smallest architectural expansion and directly exploits existing retained factors, structural analysis, numerical policy, and native solver controls. It also establishes the common contracts later approximations will need.

### Second: add one carefully bounded cold-start strategy

I would then benchmark **regularized local residual/QP preparation** against direct native execution, followed by predictor-assisted continuation.

Do not make the preliminary QP compulsory. It should earn admission through measured benefit on the relevant problem families.

Automatically constructed residual homotopies would follow only after auxiliary-problem identity and intermediate-result semantics are established.

### Third: add recurring cheap operators and multifidelity evaluation

Nonlinear preconditioning, reduced-space execution, and surrogate model management are more substantial extensions. I would prioritize them when profiling shows that difficult blocks or expensive evaluator calls—not simply poor starting points—dominate runtime.

For qualification, your simpler fixtures are an advantage. I would build tests around exact affine systems, near-singular Jacobians, badly scaled coordinates, invalid-domain starts, active-set changes, multiple roots, and failed/cancelled auxiliary stages. Include explicit tests that an auxiliary success cannot become an original-problem success.

Performance comparisons should hold final requirements constant and report **end-to-end time, success rate, expensive evaluation counts, factorization counts, and tail behavior across starting points**. Nearby-parameter studies should compare prediction against both a cold solve and an ordinary previous-solution warm start; otherwise it is easy to attribute gains to sensitivity prediction that simple reuse already provides.

## Bottom line

**The best first extension is not “automatically linearize the process model and solve it before every nonlinear solve.” It is a shared acceleration strategy that exploits available evidence and does only the preparatory work justified by that evidence.**

For `pse-arrow`, I would prioritize:

1. **Generalize your existing retained-factor prediction and compatible warm-start machinery.**
2. **Use native inexact solving and structural reduction to avoid unnecessary work.**
3. **Add bounded, safeguarded auxiliary models and continuation behind an explicit initial-point contract.**

That gives you a path toward the seamless behavior you want: domain-independent mathematical mechanisms, adaptive numerical decisions, reusable library implementations, and **one unchanged authority for whether the original problem has actually been solved**.



Yes. I would give an agent a fairly large bibliography rather than only a few “canonical” papers, because several distinct mathematical literatures converge on the architecture you want. The sources below are selected for **algorithmic transferability**: formulas, eligibility assumptions, predictor/corrector structure, globalization, safeguards, reuse rules, failure conditions, and implementation details.

I have marked **★★★** the sources I would expect an agent to read particularly carefully. The corpus is deliberately somewhat broader than the immediate implementation so the agent can discover refinements and terminology.

A useful way for the agent to mine every source is to extract the same eight things: **(1) mathematical prerequisites, (2) approximation/predictor constructed, (3) retained state/factorizations, (4) step/globalization rule, (5) acceptance test, (6) fallback/refusal conditions, (7) scaling/conditioning requirements, and (8) cost model versus a full solve.**

### 1. Parametric sensitivity, warm prediction, and advanced-step methods

This is the literature most directly connected to your existing retained-KKT `Advance` mechanism. The central idea is to turn a previously solved point and its factorization into an inexpensive approximation to a nearby solution.

1. **★★★ Pirnay, López-Negrete & Biegler — “Optimal sensitivity based on IPOPT” (2012), i.e. sIPOPT.** Probably the single most directly relevant paper for what you already built. It covers KKT sensitivity, reuse of IPOPT matrix factorizations, inexpensive approximate solutions to perturbed NLPs, Schur-complement strategies, and reduced Hessians. [Optimal sensitivity based on IPOPT](https://doi.org/10.1007/S12532-012-0043-2?utm_source=chatgpt.com) :chatgpt-content-reference{index="1"}

2. **★★★ Stechlinski, Khan & Barton — “Generalized Sensitivity Analysis of Nonlinear Programs” (2018).** Important next-generation extension because it addresses parametric perturbations that **change the active set**, using lexicographic directional derivatives rather than ordinary differentiability assumptions. [Generalized Sensitivity Analysis of Nonlinear Programs](https://epubs.siam.org/doi/10.1137/17M1120385?utm_source=chatgpt.com) :chatgpt-content-reference{index="3"}

3. **Büskens & Maurer — “Sensitivity Analysis and Real-Time Optimization of Parametric Nonlinear Programming Problems” (2001).** Very clean treatment of differentiability conditions, primal/dual sensitivities, numerical obstacles, and Taylor approximations to nearby NLP solutions. [Sensitivity Analysis and Real-Time Optimization of Parametric NLPs](https://eref.uni-bayreuth.de/3836/?utm_source=chatgpt.com) :chatgpt-content-reference{index="5"}

4. **Fiacco — “Sensitivity analysis for nonlinear programming using penalty methods” (1976).** Classical foundation for differentiating local solutions and multipliers under parameter changes. [Fiacco sensitivity analysis paper](https://explore.metascienceobservatory.org/papers/W2065846434?utm_source=chatgpt.com) :chatgpt-content-reference{index="7"}

5. **Fiacco — *Introduction to Sensitivity and Stability Analysis in Nonlinear Programming* (1983).** A broader classical reference for the conditions behind sensitivity formulas rather than merely their computational application. [Fiacco sensitivity and stability book](https://books.google.com/books?id=J19RAAAAMAAJ&utm_source=chatgpt.com) :chatgpt-content-reference{index="9"}

6. **Robinson — “Strongly Regular Generalized Equations” (1980).** Underlying theory for when parametrized solution mappings are locally unique and Lipschitz-continuous; extremely relevant when deciding when a predictor can be mathematically trusted. [Strongly Regular Generalized Equations](https://pubsonline.informs.org/doi/10.1287/moor.5.1.43?utm_source=chatgpt.com) :chatgpt-content-reference{index="11"}

7. **Dontchev & Rockafellar — *Implicit Functions and Solution Mappings: A View from Variational Analysis*, 2nd ed.** Much deeper treatment of metric/strong regularity, generalized derivatives, nonsmooth solution mappings, and numerical variational analysis. Useful if you eventually want a principled generalized-prediction layer beyond smooth KKT sensitivity. [Implicit Functions and Solution Mappings](https://link.springer.com/book/10.1007/978-1-4939-1037-3?utm_source=chatgpt.com) :chatgpt-content-reference{index="13"}

8. **★★★ Zavala & Biegler — “The advanced-step NMPC controller: Optimality, stability and robustness” (2009).** The advanced-step architecture: solve/predict ahead, then cheaply correct after parameter/state information changes. The control application is incidental; the predictor/corrector logic is highly relevant. [The advanced-step NMPC controller](https://www.sciencedirect.com/science/article/pii/S0005109808004196?utm_source=chatgpt.com) :chatgpt-content-reference{index="15"}

9. **Zavala, Laird & Biegler — “A fast moving horizon estimation algorithm based on nonlinear programming sensitivity” (2008).** Concrete implementation of nominal solve + cheap online sensitivity correction; especially useful as an example of deciding what work can be moved out of the expensive correction phase. [Fast MHE using NLP sensitivity](https://www.sciencedirect.com/science/article/pii/S0959152408001091?utm_source=chatgpt.com) :chatgpt-content-reference{index="17"}

10. **Biegler, Yang & Fischer — “Advances in sensitivity-based nonlinear model predictive control and dynamic real-time optimization” (2015).** Useful survey tying interior-point NLP sensitivity, advanced-step algorithms and large first-principles process models together. [Advances in sensitivity-based NMPC and D-RTO](https://www.sciencedirect.com/science/article/abs/pii/S0959152415000281?utm_source=chatgpt.com) :chatgpt-content-reference{index="19"}

11. **Würth, Hannemann & Marquardt — “Neighboring-extremal updates for nonlinear model-predictive control and dynamic real-time optimization” (2009).** Worth reading for neighboring-extremal/second-order update ideas rather than only first-order sensitivity prediction. [Neighboring-extremal updates](https://www.sciencedirect.com/science/article/pii/S0959152409000341?utm_source=chatgpt.com) :chatgpt-content-reference{index="21"}

12. **Diehl, Bock & Schlöder — “A Real-Time Iteration Scheme for Nonlinear Optimization in Optimal Feedback Control” (2005).** Shows the more aggressive idea of intentionally performing only limited Newton-type correction per update and relying on repeated nearby problems to converge collectively. [Real-Time Iteration Scheme](https://epubs.siam.org/doi/pdf/10.1137/S0363012902400713?utm_source=chatgpt.com) :chatgpt-content-reference{index="23"}

**Search vocabulary:** `parametric NLP sensitivity`, `KKT sensitivity`, `strong regularity`, `solution mapping`, `neighboring extremal`, `advanced-step optimization`, `real-time iteration`, `predictor corrector NLP`, `active-set sensitivity`, `semiderivative sensitivity`, `lexicographic directional derivative`.

---

### 2. Inexact Newton, Newton–Krylov, Jacobian reuse and cheap early iterations

This is the literature behind “do not solve an approximate Newton system more accurately than the nonlinear state warrants.”

13. **★★★ Dembo, Eisenstat & Steihaug — “Inexact Newton Methods” (1982).** The foundational inexact-Newton paper. Start here for the residual condition that turns approximate linear solves into a mathematically controlled nonlinear method. [Inexact Newton Methods](https://epubs.siam.org/doi/10.1137/0719025?utm_source=chatgpt.com) :chatgpt-content-reference{index="25"}

14. **★★★ Eisenstat & Walker — “Choosing the Forcing Terms in an Inexact Newton Method” (1996).** Directly relevant to your KINSOL `Eta` controls. It explains why adaptive forcing terms prevent oversolving the Newton linear system while recovering fast local convergence. [Choosing the Forcing Terms in an Inexact Newton Method](https://epubs.siam.org/doi/10.1137/0917003?utm_source=chatgpt.com) :chatgpt-content-reference{index="27"}

15. **Eisenstat & Walker — “Globally Convergent Inexact Newton Methods” (1994).** Complements the local inexactness theory with globalization from poor starting points. [Globally Convergent Inexact Newton Methods](https://epubs.siam.org/doi/10.1137/0804022?utm_source=chatgpt.com) :chatgpt-content-reference{index="29"}

16. **Brown & Saad — “Hybrid Krylov Methods for Nonlinear Systems of Equations” (1990).** Particularly good because it combines Newton–Krylov with Powell dogleg and line-search safeguards rather than treating Krylov efficiency in isolation. [Hybrid Krylov Methods for Nonlinear Systems](https://epubs.siam.org/doi/10.1137/0911026?utm_source=chatgpt.com) :chatgpt-content-reference{index="31"}

17. **Brown & Saad — “Convergence Theory of Nonlinear Newton–Krylov Algorithms” (1994).** Deeper convergence theory for combining nonlinear Newton iterations with Krylov solution of the linearized system. [Convergence Theory of Nonlinear Newton–Krylov Algorithms](https://epubs.siam.org/doi/10.1137/0804017?utm_source=chatgpt.com) :chatgpt-content-reference{index="33"}

18. **Knoll & Keyes — “Jacobian-free Newton–Krylov methods: a survey of approaches and applications” (2004).** Excellent architectural survey of JFNK, Jacobian-vector products, preconditioning requirements, and the tradeoffs between assembled and matrix-free treatment. [Jacobian-free Newton–Krylov survey](https://www.sciencedirect.com/science/article/pii/S0021999103004340?utm_source=chatgpt.com) :chatgpt-content-reference{index="35"}

19. **C. T. Kelley — *Iterative Methods for Linear and Nonlinear Equations*.** Compact but unusually implementation-oriented reference covering Newton, inexact Newton, GMRES, Broyden methods and globalization. [Iterative Methods for Linear and Nonlinear Equations](https://epubs.siam.org/doi/10.1137/1.9781611970944?utm_source=chatgpt.com) :chatgpt-content-reference{index="37"}

20. **Walker & Ni — “Anderson Acceleration for Fixed-Point Iterations” (2011).** The standard modern Anderson-acceleration reference; also connects Anderson mixing to multisecant/quasi-Newton ideas. [Anderson Acceleration for Fixed-Point Iterations](https://epubs.siam.org/doi/pdf/10.1137/10078356X?utm_source=chatgpt.com) :chatgpt-content-reference{index="39"}

21. **Toth & Kelley — “Convergence Analysis for Anderson Acceleration” (2015).** Adds convergence theory needed to reason about when Anderson should be admitted rather than merely exposing history depth as an option. [Convergence Analysis for Anderson Acceleration](https://explore.metascienceobservatory.org/papers/W2071486596?utm_source=chatgpt.com) :chatgpt-content-reference{index="41"}

22. **★★★ SUNDIALS KINSOL — Mathematical Considerations.** This is almost as important as the papers because it documents the exact production algorithms you are wrapping: modified/inexact Newton, line search, forcing terms, Picard/fixed-point methods and Anderson QR implementation. [KINSOL Mathematical Considerations](https://sundials.readthedocs.io/en/v7.7.0/kinsol/Mathematics_link.html?utm_source=chatgpt.com) :chatgpt-content-reference{index="43"}

23. **KINSOL usage/API documentation.** Useful for implementation nuances around Anderson depth/delay/damping, scaling, constraints, Jv products and preconditioners. [KINSOL usage documentation](https://sundials.readthedocs.io/en/latest/kinsol/Usage/index.html?utm_source=chatgpt.com) :chatgpt-content-reference{index="45"}

24. **★★★ PETSc SNES manual.** Even if you do not use PETSc, its nonlinear solver architecture is an excellent reference implementation for composable Newton, trust region, quasi-Newton, nonlinear GMRES/Anderson and nonlinear preconditioners under a common interface. [PETSc SNES nonlinear solver manual](https://petsc.org/main/manual/snes/?utm_source=chatgpt.com) :chatgpt-content-reference{index="47"}

**Search vocabulary:** `inexact Newton forcing term`, `Eisenstat Walker`, `Newton Krylov`, `modified Newton Jacobian reuse`, `lagged Jacobian`, `lagged preconditioner`, `Jacobian-free Newton Krylov`, `nonlinear GMRES`, `Anderson mixing`, `multisecant acceleration`.

---

### 3. Safeguarded linear/QP approximations, trust regions, SQP and SLP

This is the literature closest to your original intuition of “solve a cheap linearized version first,” but it explains the safeguards that make that idea robust.

25. **★★★ Conn, Gould & Toint — *Trust Region Methods*.** The comprehensive reference. Of particular relevance are model adequacy, actual-versus-predicted reduction, radius adaptation, conditional models and constrained trust-region methods. [Trust Region Methods](https://epubs.siam.org/doi/10.1137/1.9780898719857.ch1?utm_source=chatgpt.com) :chatgpt-content-reference{index="49"}

26. **Moré — “The Levenberg–Marquardt algorithm: Implementation and theory” (1978).** A classic implementation reference for stabilizing linearized least-squares models with implicit scaling and adaptive regularization. Very relevant to a generic residual-minimization initializer. [Levenberg–Marquardt: Implementation and Theory](https://cir.nii.ac.jp/crid/1360574092887766016?utm_source=chatgpt.com) :chatgpt-content-reference{index="51"}

27. **Sorensen — “Newton’s Method with a Model Trust Region Modification” (1982).** Foundational analysis of restricting Newton/quadratic model steps to a region where the local model is trustworthy. [Newton's Method with a Model Trust-Region Modification](https://digital.library.unt.edu/ark%3A/67531/metadc283479/?utm_source=chatgpt.com) :chatgpt-content-reference{index="53"}

28. **Nocedal & Wright — *Numerical Optimization*, 2nd ed.** Broad reference for Newton, quasi-Newton, trust regions, SQP, interior-point methods, constrained optimization and globalization. [Numerical Optimization](https://link.springer.com/book/10.1007/978-0-387-40065-5?utm_source=chatgpt.com) :chatgpt-content-reference{index="55"}

29. **★★★ Fletcher, Gould, Leyffer, Toint & Wächter — “Global Convergence of a Trust-Region SQP-Filter Algorithm for General Nonlinear Programming.”** Particularly valuable if an auxiliary QP is introduced: normal/tangential decomposition, approximate QP solves, filter acceptance and trust-region safeguards. [Trust-Region SQP-Filter Algorithm](https://epubs.siam.org/doi/10.1137/S1052623499357258?utm_source=chatgpt.com) :chatgpt-content-reference{index="57"}

30. **★★★ Wächter & Biegler — “On the implementation of an interior-point filter line-search algorithm for large-scale nonlinear programming.”** This is the Ipopt algorithm paper. Read it for restoration, filter globalization, second-order correction, inertia correction, initialization and practical performance heuristics. [Ipopt algorithm implementation paper](https://doi.org/10.1007%2FS10107-004-0559-Y?utm_source=chatgpt.com) :chatgpt-content-reference{index="59"}

31. **Ipopt current options documentation.** Important when distinguishing algorithmic concepts from exposed native controls—especially warm starts, barrier initialization, restoration, scaling and linear-solver behavior. [Ipopt Options Reference](https://coin-or.github.io/Ipopt/OPTIONS.html?utm_source=chatgpt.com) :chatgpt-content-reference{index="61"}

32. **Fletcher, Leyffer & Toint — “On the global convergence of an SLP-filter algorithm.”** Directly relevant to the idea of repeatedly solving LP approximations while controlling model validity without an awkward penalty parameter. [SLP-filter algorithm paper](https://optimization-online.org/2000/08/209/?utm_source=chatgpt.com) :chatgpt-content-reference{index="63"}

33. **Chin & Fletcher — SLP-filter with EQP steps.** Interesting hybrid: inexpensive LP subproblems for globalization with equality-constrained QP steps when second-order information becomes valuable. [SLP-filter with EQP steps](https://discovery.dundee.ac.uk/en/publications/on-the-global-convergence-of-an-slp-filter-algorithm-that-takes-e/?utm_source=chatgpt.com) :chatgpt-content-reference{index="65"}

34. **Mao, Szmuk & Açıkmeşe — “Successive Convexification of Non-Convex Optimal Control Problems and Its Convergence Properties.”** The application is optimal control, but the generic lessons are useful: linearization alone is dangerous; combine it with trust regions and explicit mechanisms for temporarily recovering feasibility. [Successive Convexification paper](https://arxiv.org/abs/1608.05133?utm_source=chatgpt.com) :chatgpt-content-reference{index="67"}

35. **MINPACK modern project + reference list.** An excellent source of mature nonlinear-equation implementation ideas: Powell hybrid methods, dogleg/trust-region strategies and Levenberg–Marquardt. [Modern MINPACK project](https://github.com/fortran-lang/minpack?utm_source=chatgpt.com) :chatgpt-content-reference{index="69"}

**Search vocabulary:** `sequential linear programming`, `SLP filter`, `sequential quadratic programming`, `normal tangential step`, `elastic mode`, `feasibility restoration`, `trust region ratio`, `Levenberg Marquardt`, `dogleg nonlinear equations`, `successive convexification`, `model adequacy`.

---

### 4. Continuation, homotopy and predictor–corrector path following

I would give this section substantial weight. It is the most principled general answer when a difficult target problem can be embedded in a smoother family of easier problems.

36. **★★★ Allgower & Georg — *Numerical Continuation Methods: An Introduction*.** Probably the best single broad continuation reference: predictor–corrector methods, tangent computation, Newton correction, adaptive step length, Jacobian updating, large-scale problems, bifurcation and homotopy. [Numerical Continuation Methods](https://link.springer.com/book/10.1007/978-3-642-61257-2?utm_source=chatgpt.com) :chatgpt-content-reference{index="71"}

37. **Allgower & Georg — “Continuation and path following” (Acta Numerica).** More compact survey of predictor–corrector and piecewise-linear continuation, homotopy and software considerations. [Continuation and path following review](https://www.cambridge.org/core/journals/acta-numerica/article/abs/continuation-and-path-following/4368C662C0FA6F729FA4B2A5C1B60085?utm_source=chatgpt.com) :chatgpt-content-reference{index="73"}

38. **Allgower & Georg — Newton corrector chapter.** Very concrete generic predictor–corrector algorithm formulation. [Newton's Method as Corrector](https://epubs.siam.org/doi/10.1137/1.9780898719154.ch3?utm_source=chatgpt.com) :chatgpt-content-reference{index="75"}

39. **Allgower & Georg — predictor/corrector updating chapter.** Particularly relevant to reducing repeated Jacobian calculation/factorization while tracing a path. [Predictor-Corrector Methods Using Updating](https://epubs.siam.org/doi/10.1137/1.9780898719154.ch7?utm_source=chatgpt.com) :chatgpt-content-reference{index="77"}

40. **★★★ Watson, Billups & Morgan — “Algorithm 652: HOMPACK.”** A software-oriented treatment of three distinct homotopy path-following algorithms, including sparse variants. This is useful precisely because it turns theory into reusable solver machinery. [HOMPACK Algorithm 652](https://www.researchgate.net/publication/220493190_ALGORITHM_652_HOMPACK_a_suite_of_codes_for_globally_convergent_homotopy_algorithms?utm_source=chatgpt.com) :chatgpt-content-reference{index="79"}

41. **Rheinboldt & Burkardt — “A Locally Parameterized Continuation Process” / PITCON.** Curvature-controlled local parameterization, target points and limit points; very relevant to adaptive continuation that must survive folds. [A Locally Parameterized Continuation Process](https://www.researchgate.net/publication/220492719_A_Locally_Parameterized_Continuation_Process?utm_source=chatgpt.com) :chatgpt-content-reference{index="81"}

42. **Keller — “Global Homotopies and Newton Methods.”** Useful introduction to pseudo-arclength continuation and why naïvely using the physical continuation parameter becomes problematic near singularities/turning points. [Global Homotopies and Newton Methods](https://authors.library.caltech.edu/records/24xxv-vpz22?utm_source=chatgpt.com) :chatgpt-content-reference{index="83"}

43. **Wayburn & Seader — “Homotopy continuation methods for computer-aided process design” (1987).** Highly relevant because it explicitly catalogues process-simulation failure modes: paths returning to the easy problem, leaving the domain of definition, and going to infinity. [Homotopy continuation for process design](https://www.sciencedirect.com/science/article/pii/0098135487800029?utm_source=chatgpt.com) :chatgpt-content-reference{index="85"}

44. **★★★ Paloschi — “Bounded homotopies to solve systems of sparse algebraic nonlinear equations” (1997).** One of the sources I would prioritize most for `pse-arrow`: it constructs homotopies intended to stay inside bounded evaluation domains **while retaining the original Jacobian sparsity pattern**. [Bounded sparse homotopies](https://www.sciencedirect.com/science/article/pii/S0098135496002876?utm_source=chatgpt.com) :chatgpt-content-reference{index="87"}

45. **Paloschi — “Using sparse bounded homotopies in the SPEEDUP simulation package” (1998).** Practical follow-on showing exactly why bounded paths matter when property packages fail outside their valid region. [Sparse bounded homotopies in SPEEDUP](https://www.sciencedirect.com/science/article/pii/S0098135498000209?utm_source=chatgpt.com) :chatgpt-content-reference{index="89"}

46. **Jiménez-Islas et al. — “Nonlinear Homotopic Continuation Methods: A Chemical Engineering Perspective Review” (2013).** Useful map of the chemical/process literature and vocabulary for finding more specialized homotopies later. [Chemical-engineering homotopy review](https://pubs.acs.org/doi/10.1021/ie402418e?utm_source=chatgpt.com) :chatgpt-content-reference{index="91"}

47. **★★★ Trilinos LOCA documentation.** A strong production architecture reference: continuation and bifurcation machinery layered minimally over an ordinary nonlinear-solver interface, including parameter derivatives and augmented continuation equations. [Trilinos NOX and LOCA](https://trilinos.github.io/nox_and_loca.html?utm_source=chatgpt.com) :chatgpt-content-reference{index="93"}

48. **PETSc Newton with arc-length continuation.** Another concrete production implementation worth comparing with LOCA, especially its augmented system and step handling. [PETSc SNES arc-length continuation](https://petsc.org/main/manual/snes/?utm_source=chatgpt.com) :chatgpt-content-reference{index="95"}

**Search vocabulary:** `predictor corrector continuation`, `pseudo arclength continuation`, `natural parameter continuation`, `tangent predictor`, `secant predictor`, `homotopy path tracking`, `step-length adaptation`, `bounded homotopy`, `sparse homotopy`, `fold tracking`, `continuation Jacobian updating`.

---

### 5. Pseudo-transient continuation and dynamic/DAE initialization

This literature is relevant both to difficult steady states and to your dynamic solver path.

49. **★★★ Kelley & Keyes — “Convergence Analysis of Pseudo-Transient Continuation” (1998).** Foundational convergence treatment. Important for understanding why PTC is not merely “small timesteps before Newton,” and under what assumptions it actually globalizes the root solve. [Convergence Analysis of Pseudo-Transient Continuation](https://doi.org/10.1137/S0036142996304796?utm_source=chatgpt.com) :chatgpt-content-reference{index="97"}

50. **★★★ Coffey, Kelley & Keyes — “Pseudotransient Continuation and Differential-Algebraic Equations” (2003).** Particularly useful algorithmically: adaptive pseudo-time stepping followed by transition toward Newton behavior as the steady root is approached. [Pseudotransient Continuation and DAEs](https://epubs.siam.org/doi/10.1137/S106482750241044X?utm_source=chatgpt.com) :chatgpt-content-reference{index="99"}

51. **PETSc `TSPSEUDO`.** Excellent concrete implementation reference; documents the pseudo-time residual, one-step formulation and switched-evolution-relaxation-style timestep adaptation. [PETSc TSPSEUDO implementation documentation](https://petsc.org/main/manualpages/TS/TSPSEUDO.html?utm_source=chatgpt.com) :chatgpt-content-reference{index="101"}

52. **★★★ Pattison & Baldea — “Equation-oriented flowsheet simulation and optimization using pseudo-transient models” (2014).** Process-specific but very pertinent: constructs statically equivalent pseudo-transient models to enlarge the convergence basin of equation-oriented flowsheets. [Equation-oriented flowsheet simulation using pseudo-transient models](https://aiche.onlinelibrary.wiley.com/doi/10.1002/aic.14567?utm_source=chatgpt.com) :chatgpt-content-reference{index="103"}

53. **★★★ Brown, Hindmarsh & Petzold — “Consistent Initial Condition Calculation for Differential-Algebraic Systems” (1998).** Core reference for systematic DAE initialization, including cases where differential variables versus derivative vectors are specified. [Consistent Initial Condition Calculation for DAEs](https://epubs.siam.org/doi/10.1137/S1064827595289996?utm_source=chatgpt.com) :chatgpt-content-reference{index="105"}

54. **Pantelides — “The Consistent Initialization of Differential-Algebraic Systems.”** Structural reasoning about which differentiated equations impose additional initial constraints. [The Consistent Initialization of DAE Systems](https://epubs.siam.org/doi/10.1137/0909014?utm_source=chatgpt.com) :chatgpt-content-reference{index="107"}

55. **Kröner, Marquardt & Gilles — “Computing consistent initial conditions for differential-algebraic equations” (1992).** Combines implicit-Euler/extended-system initialization ideas with structural information to reduce computational complexity. [Computing consistent initial conditions for DAEs](https://www.sciencedirect.com/science/article/pii/S009813540980015X?utm_source=chatgpt.com) :chatgpt-content-reference{index="109"}

56. **Unger, Kröner & Marquardt — “Structural analysis of differential-algebraic equation systems—theory and applications” (1995).** Strong connection between structural analysis, index, degrees of freedom, initialization obligations and numerical method selection. [Structural analysis of DAE systems](https://www.sciencedirect.com/science/article/abs/pii/0098135494000945?utm_source=chatgpt.com) :chatgpt-content-reference{index="111"}

57. **Gopal & Biegler — “A Successive Linear Programming Approach for Initialization and Reinitialization after Discontinuities of Differential-Algebraic Equations.”** Particularly germane to your initial question because it applies **successive linear approximations explicitly to initialization**. [SLP for DAE initialization and reinitialization](https://www.researchgate.net/publication/242913489_A_Successive_Linear_Programming_Approach_for_Initialization_and_Reinitialization_after_Discontinuities_of_Differential-Algebraic_Equations?utm_source=chatgpt.com) :chatgpt-content-reference{index="113"}

**Search vocabulary:** `pseudo transient continuation`, `switched evolution relaxation`, `SER timestep`, `DAE consistent initialization`, `derivative array initialization`, `structural DAE initialization`, `implicit Euler initialization`, `steady-state DAE globalization`.

---

### 6. Structural decomposition, reduced-space solving and nonlinear preconditioning

This is where your existing DM/BTF infrastructure can eventually become more than an initialization ordering mechanism.

58. **★★★ Pothen & Fan — “Computing the block triangular form of a sparse matrix” (1990).** Foundational implementation reference connecting matching and sparse block triangularization. [Computing the block triangular form of a sparse matrix](https://explore.metascienceobservatory.org/papers/W2093992309?utm_source=chatgpt.com) :chatgpt-content-reference{index="115"}

59. **Duff & Reid — “Algorithm 529: Permutations To Block Triangular Form.”** Earlier concrete algorithm/software reference for BTF construction. [Algorithm 529 reference](https://www.cs.kent.ac.uk/projects/toms/Volumes/V4.html?utm_source=chatgpt.com) :chatgpt-content-reference{index="117"}

60. **Gundersen & Hertzberg — “Partitioning and Tearing of Networks Applied to Process Flowsheeting” (1983).** Classic process-flowsheet decomposition/tearing reference and useful bridge between graph structure and iterative numerical execution. [Partitioning and Tearing of Networks](https://www.mic-journal.no/ABS/MIC-1983-3-2.asp/?utm_source=chatgpt.com) :chatgpt-content-reference{index="119"}

61. **Parker, Nicholson, Siirola & Biegler — “Applications of the Dulmage–Mendelsohn decomposition for debugging nonlinear optimization problems” (2023).** Modern explanation of what DM does and, importantly, what conclusions can legitimately be drawn from structural versus numerical information. [Modern DM decomposition paper](https://www.sciencedirect.com/science/article/pii/S0098135423002533?utm_source=chatgpt.com) :chatgpt-content-reference{index="121"}

62. **Biegler, Nocedal & Schmid — “A Reduced Hessian Method for Large-Scale Constrained Optimization” (1995).** Useful foundation for reduced-space methods, sparse factorization and range/null-space decomposition when the effective degrees of freedom are much smaller than the full state. [Reduced Hessian Method for Large-Scale Constrained Optimization](https://epubs.siam.org/doi/10.1137/0805017?utm_source=chatgpt.com) :chatgpt-content-reference{index="123"}

63. **★★★ Brune, Knepley, Smith & Tu — “Composing Scalable Nonlinear Algebraic Solvers.”** The key reference if you want block initialization/local solves eventually to become **nonlinear solver components/preconditioners**, rather than one-time preprocessing. [Composing Scalable Nonlinear Algebraic Solvers](https://epubs.siam.org/doi/10.1137/130936725?utm_source=chatgpt.com) :chatgpt-content-reference{index="125"}

64. **Cai & Keyes — “Nonlinearly Preconditioned Inexact Newton Algorithms” (2002).** Foundational treatment of replacing \(F(x)=0\) with a mathematically equivalent but numerically better-conditioned nonlinear system before applying Newton. [Nonlinearly Preconditioned Inexact Newton Algorithms](https://epubs.siam.org/doi/10.1137/S106482750037620X?utm_source=chatgpt.com) :chatgpt-content-reference{index="127"}

65. **Dolean et al. — “Nonlinear Preconditioning: How to Use a Nonlinear Schwarz Method to Precondition Newton’s Method” (2016).** More modern construction of nonlinear preconditioners from convergent local nonlinear iterations. [Nonlinear Schwarz Preconditioning of Newton](https://epubs.siam.org/doi/10.1137/15M102887X?utm_source=chatgpt.com) :chatgpt-content-reference{index="129"}

66. **PETSc nonlinear-preconditioning architecture.** Worth inspecting alongside the papers because PETSc explicitly lets nonlinear GMRES/Anderson/Newton methods consume another nonlinear solver as a preconditioner. [PETSc nonlinear solver composition documentation](https://petsc.org/release//manual/snes/?utm_source=chatgpt.com) :chatgpt-content-reference{index="131"}

**Search vocabulary:** `nonlinear preconditioning`, `ASPIN`, `RASPEN`, `nonlinear Schwarz`, `multiplicative nonlinear preconditioner`, `solver composition`, `block nonlinear Gauss Seidel`, `reduced-space Newton`, `range-space null-space`, `block triangular nonlinear equations`, `tearing`.

---

### 7. Multifidelity, surrogate/model management and reduced-order models

I would treat these as a later layer, but the architectural contracts are worth understanding now so that the initial-point abstraction does not preclude them.

67. **★★★ Alexandrov, Dennis, Lewis & Torczon — “A trust-region framework for managing the use of approximation models in optimization” (1998).** This is the crucial conceptual paper: do not simply trust a cheap surrogate; manage it with a trust region and high-fidelity evaluations so convergence remains tied to the original model. [Trust-region management of approximation models](https://www.tandfonline.com/servlet/linkout?type=Scholix&url=https%3A%2F%2Fdx.doi.org%2F10.1007%252Fbf01197433&utm_source=chatgpt.com) :chatgpt-content-reference{index="133"}

68. **★★★ Peherstorfer, Willcox & Gunzburger — “Survey of Multifidelity Methods in Uncertainty Propagation, Inference, and Optimization” (2018).** Broad open-access taxonomy of low/high-fidelity model combinations, including adaptation, fusion and filtering, with the high-fidelity model retained as accuracy authority. [Survey of Multifidelity Methods](https://epubs.siam.org/doi/10.1137/16M1082469?utm_source=chatgpt.com) :chatgpt-content-reference{index="135"}

69. **Benner, Gugercin & Willcox — “A Survey of Projection-Based Model Reduction Methods for Parametric Dynamical Systems” (2015).** Best broader entry point if you eventually want reusable reduced-order physics models rather than merely local Taylor approximations. [Projection-Based Model Reduction Survey](https://dspace.mit.edu/entities/publication/b7c394a6-4591-4e5b-a7df-7e0fef385dfb?utm_source=chatgpt.com) :chatgpt-content-reference{index="137"}

**Search vocabulary:** `multifidelity trust region`, `model management`, `surrogate correction`, `first-order consistency surrogate`, `space mapping`, `response correction`, `reduced-order model`, `projection ROM`, `adaptive fidelity`, `high-fidelity acceptance`.

---

## What I would have the agent read first

If the goal is **implementing useful capability now**, rather than academically surveying all 69 references, there is a fairly coherent path through the corpus:

**sIPOPT → generalized NLP sensitivity → Dembo/Eisenstat/Steihaug → Eisenstat–Walker forcing terms → KINSOL mathematical docs → Trust Region Methods → Wächter/Biegler Ipopt → Allgower/Georg continuation → Paloschi bounded sparse homotopy → Kelley/Keyes pseudo-transient continuation → Brown/Hindmarsh/Petzold DAE initialization → Brune et al. nonlinear solver composition → Alexandrov et al. model management.**

Those papers together essentially specify the intellectual architecture we discussed:

\[
\boxed{
\text{reuse/predict}
\;\rightarrow\;
\text{cheap safeguarded progress}
\;\rightarrow\;
\text{continuation/globalization if needed}
\;\rightarrow\;
\text{full original-problem correction}
\;\rightarrow\;
\text{original-space qualification}
}
\]

One nuance I would make explicit to the programming agent is that **these should not all become sequential mandatory phases**. The deeper common design is an *admission-driven family of acceleration mechanisms*. Sensitivity prediction is exceptional when an appropriate previous solution exists; adaptive inexact Newton is useful during the correction itself; a local QP/LP model is useful only when its cost is justified; bounded homotopy is primarily a robustness mechanism; pseudo-transient continuation needs an appropriate evolution structure; nonlinear preconditioning becomes attractive when there is useful decomposable nonlinear structure; and multifidelity methods become worthwhile when model evaluation is expensive enough to repay construction of the surrogate.

For your particular solver architecture, I think **references 1, 2, 14, 22, 25, 30, 36, 44, 49, 53, 63 and 67** collectively give an agent enough material to design an unusually sophisticated unified implementation without resorting to domain-specific initialization rules.