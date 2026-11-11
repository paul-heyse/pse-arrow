# POUNCE — Comprehensive Rust Optimization Solver Reference

Research date: November 10, 2026 | Target release: 0.16.0 (November 7, 2026) | Language: Rust | License: EPL-2.0

POUNCE is a native Rust numerical optimization suite developed by John R. Kitchin at Carnegie Mellon University. Originally a pure-Rust port of IPOPT, it has expanded into a broader optimization system encompassing nonlinear programming, convex and conic optimization, active-set quadratic programming, polynomial global optimization, and post-optimal sensitivity analysis.

Its significance extends beyond reproducing IPOPT in Rust. POUNCE provides several complementary optimization algorithms built around shared sparse linear algebra, with native APIs for problem construction, solver control, warm starts, differentiation, and analysis.

The current Rust integration entry point is `pounce-rs`, a facade crate exposing the solver family's supported public APIs. The default build focuses on nonlinear programming; additional solver families are exposed through feature flags.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Primary resources:

- Official repository: `https://github.com/jkitchin/pounce`
- Project documentation: `https://kitchingroup.cheme.cmu.edu/pounce/`
- Rust API documentation: `https://docs.rs/pounce-rs/0.16.0/pounce_rs/`
- Crate distribution: `https://crates.io/crates/pounce-rs`
- Release history: `https://github.com/jkitchin/pounce/releases`

## 1. Capability overview

POUNCE is best understood as a family of continuous optimization solvers rather than a single mathematical-programming algorithm.

| Capability                             | Native functionality                                                       | Important distinction                                                                  |
| -------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| General nonlinear programming (NLP)    | Primal-dual interior-point method with filter line search                  | Supports smooth nonconvex problems, generally with local convergence                   |
| Sequential quadratic programming (SQP) | Active-set SQP with an interior-point handover path                        | Particularly relevant to constrained local refinement and warm-started problems        |
| Linear programming (LP)                | Dedicated convex optimization path                                         | Global convex solution rather than general-purpose NLP treatment                       |
| Quadratic programming (QP)             | Convex interior-point and parametric active-set QP solvers                 | Different algorithms for cold solves versus repeatedly modified problems               |
| Conic optimization                     | Second-order, positive-semidefinite, exponential, and power cones          | Extends beyond ordinary LP/QP into structured convex models                            |
| Polynomial global optimization         | Sum-of-squares and Lasserre semidefinite relaxations                       | Can establish global optimality under appropriate polynomial and relaxation conditions |
| Sensitivity analysis                   | Parametric solution sensitivities, reduced Hessians, warm-start predictors | Uses optimization-system derivatives rather than requiring repeated independent solves |
| Sparse linear algebra                  | Pure-Rust FERAL symmetric factorization; optional HSL MA57                 | Avoids mandatory native Fortran/BLAS dependencies                                      |
| NLP preprocessing                      | Bound propagation, variable elimination, redundancy handling               | Simplifies certain models before numerical solution                                    |
| Programmatic integration               | Native Rust, C ABI, Python, AMPL, Pyomo, CLI, WebAssembly                  | Multiple modeling and deployment paths reach the same solver family                    |

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+2

Three distinctions are especially important:

First, POUNCE is not limited to locally solving NLPs. Its dedicated convex solvers and polynomial SOS machinery provide capabilities that a conventional IPOPT interface does not directly supply.

Second, its two QP approaches have different intended roles. The convex interior-point path is oriented toward general convex QP solving, while the active-set engine maintains working-set information useful in parametric QP sequences and SQP.

Third, POUNCE does not provide a general-purpose global nonlinear branch-and-bound engine. Its polynomial global optimization capabilities must not be confused with global optimization of arbitrary nonconvex functions, integer programming, or mixed-integer nonlinear programming.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



## 2. Architecture and crate organization

### 2.1 Solver stack

The architecture separates problem specification, optimization algorithms, numerical linear algebra, and user-facing interfaces.

Application and modeling interfaces

Native Rust · Python · C ABI · AMPL/Pyomo · CLI · WebAssembly

Problem representation and orchestration

TNLP · Builder · Expression/derivative tapes · Solver selection · Presolve

NLP / SQP

Filter IPM\
Active-set SQP

Convex / Conic

LP / QP\
SOCP / SDP / SOS

Parametric QP

Working sets\
Homotopy

Shared numerical infrastructure

KKT assembly · Sparse symmetric factorization · Refinement · Sensitivity

FERAL

Default pure-Rust LDLᵀ

HSL MA57

Optional native backend

Conceptual architecture synthesized from POUNCE's published workspace organization and solver documentation.

The numerical center of the system is the Karush–Kuhn–Tucker (KKT) linear system. Different algorithms construct different KKT systems, but the sparse symmetric factorization infrastructure is shared.

This provides a useful separation: changes in algorithmic strategy do not necessarily require rebuilding the entire linear algebra integration layer.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 2.2 Major Rust crates

| Crate                  | Responsibility                                                  |
| ---------------------- | --------------------------------------------------------------- |
| `pounce-rs`            | Curated Rust-facing API and feature-gated facade                |
| `pounce-nlp`           | `TNLP` interface, nonlinear problem adaptation                  |
| `pounce-algorithm`     | Filter-IPM and SQP algorithms, application driver               |
| `pounce-common`        | Solver types, options, numerical utilities                      |
| `pounce-linalg`        | Dense/sparse vectors, matrices, triplet and CSC representations |
| `pounce-linsol`        | Backend-independent sparse symmetric linear-solver interfaces   |
| `pounce-feral`         | Pure-Rust sparse LDLᵀ factorization implementation              |
| `pounce-hsl`           | Optional MA57 integration                                       |
| `pounce-convex`        | LP, convex QP, conic, SDP, SOS optimization                     |
| `pounce-qp`            | Parametric active-set QP subproblem solver                      |
| `pounce-sensitivity`   | NLP sensitivities, correctors, reduced Hessians                 |
| `pounce-presolve`      | NLP preprocessing and bound propagation                         |
| `pounce-restoration`   | Recovery and second-opinion solve strategies                    |
| `pounce-l1penalty`     | Exact-penalty treatment for difficult or degenerate constraints |
| `pounce-nl`            | AMPL `.nl` parsing and derivative evaluation                    |
| `pounce-cinterface`    | IPOPT-compatible C interfaces                                   |
| `pounce-observability` | Logging and iteration capture                                   |
| `pounce-solve-report`  | Structured solver-result reporting                              |
| `pounce-cli`           | Command-line solver                                             |
| `pounce-studio-core`   | Diagnostic and solve-report processing                          |

The workspace also includes Python bindings, WebAssembly integration, studio tooling, and additional specialized components. The workspace crate layout is an internal implementation organization; `pounce-rs` deliberately provides a more stable public integration boundary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 3. Native Rust integration

### 3.1 Build features

POUNCE uses Cargo feature flags to select additional solver families.

| Feature       | Exposed modules and capabilities                                                |
| ------------- | ------------------------------------------------------------------------------- |
| Default       | NLP builder, `TNLP`, interior-point solver, presolve/restoration infrastructure |
| `convex`      | `pounce_rs::convex`; LP, convex QP, conic programs, SDP, SOS                    |
| `qp`          | `pounce_rs::qp`, `pounce_rs::sqp`; active-set QP and SQP warm-start support     |
| `sensitivity` | `pounce_rs::sensitivity`; parametric NLP sensitivity                            |
| `minima`      | `pounce_rs::minima`; multistart/local-minimum discovery                         |
| `full`        | Enables the four additional solver families                                     |
| `ma57`        | Optional integration with HSL MA57                                              |

The `minima` feature is important to distinguish from SOS global optimization: multistart searches for good or distinct local optima but does not establish a global-optimality certificate.

The published 0.16.0 facade includes the `minima` and `session` modules in addition to the core solver interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



A complete Rust dependency declaration can be written as:

```
[dependencies]
pounce-rs = { version = "0.16.0", features = ["full"] }
```

Or with a narrower solver surface:

```
[dependencies]
pounce-rs = {
    version = "0.16.0",
    features = ["convex", "qp", "sensitivity"]
}
```

The optional MA57 backend requires separately installed native HSL libraries and appropriate licensing. It is not necessary for the ordinary pure-Rust build.

### 3.2 High-level `Problem` / `Nlp` API

The simplest Rust programming model consists of the `Problem` trait and `Nlp` builder.

The `Problem` trait makes the objective mandatory while keeping other callbacks optional. The builder handles variable bounds, constraint bounds, starting points, options, and solver execution.

```
use pounce_rs::prelude::*;

struct Rosenbrock;

impl Problem for Rosenbrock {
    fn objective(&self, x: &[f64]) -> f64 {
        let a = 1.0 - x[0];
        let b = x[1] - x[0] * x[0];
        a * a + 100.0 * b * b
    }
}

let result = Nlp::new(Rosenbrock)
    .x0(&[-1.2, 1.0])
    .option_num("tol", 1e-8)
    .solve();
```

The builder supports default derivative strategies:

- Missing gradients and Jacobians are approximated by finite differences.
- The default Hessian treatment is a limited-memory L-BFGS approximation.
- Explicit gradients and Jacobians can be supplied through the trait callbacks.
- Options use IPOPT-style names and typed setter methods.

This lowers integration complexity for function-based models, but it has an important cost: finite differences can increase objective/constraint evaluations, introduce truncation error, and obscure known sparsity.

The higher-level interface is therefore primarily an ergonomic abstraction, not automatically the most efficient representation for large structured models.

The `Nlp` builder also supports validated solver configuration:

- `option_num`, `option_int`, `option_str` for options.
- `solve()` for direct execution.
- `try_solve()` for setup-error handling without panicking.
- `.capture_iterations()` to retain iteration history.

A nonconverged optimization run is not necessarily an API error: `try_solve()` can return `Ok(Solution)` with `success == false`. Callers must inspect the actual solver status.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/rust.html`

### 3.3 Low-level `TNLP` interface

The `TNLP` trait is POUNCE's higher-control nonlinear problem interface, conceptually analogous to IPOPT's TNLP interface.

It allows explicit specification of:

| Method               | Role                                                         |
| -------------------- | ------------------------------------------------------------ |
| `get_nlp_info`       | Variable/constraint dimensions and derivative nonzero counts |
| `get_bounds_info`    | Variable and constraint bounds                               |
| `get_starting_point` | Initial primal and supported dual values                     |
| `eval_f`             | Objective                                                    |
| `eval_grad_f`        | Objective gradient                                           |
| `eval_g`             | Constraint function                                          |
| `eval_jac_g`         | Constraint Jacobian, structure and values                    |
| `eval_h`             | Lagrangian Hessian, structure and values                     |
| `finalize_solution`  | Result collection                                            |

An application uses `IpoptApplication` to initialize and solve an object implementing `TNLP`.

A consequential feature is that Jacobian and Hessian callbacks distinguish sparsity structure from numerical values through `SparsityRequest::Structure` and `SparsityRequest::Values`.

This permits numerical derivative values to change without requiring the problem's sparsity structure to be rediscovered every iteration.

The low-level path becomes important when a model can provide exact, sparse derivatives. Such derivatives may originate from analytic formulas, symbolic differentiation, automatic differentiation, or an external modeling system; `TNLP` itself is the evaluation interface, not a general symbolic modeling language.

Other reasons to use `TNLP` include custom scaling and advanced solver configuration.

Documentation: `https://docs.rs/pounce-rs/latest/pounce_rs/`

### 3.4 Solution and numerical result semantics

The high-level NLP solution exposes more than a primal point.

Its result includes the final objective, constraint values, constraint multipliers, lower and upper bound multipliers, solve status, and numerical statistics.

This makes it possible to inspect whether a constraint is active, estimate local marginal values, evaluate feasibility, and distinguish convergence behavior.

One practical distinction: the facade populates solution vectors during finalization. If execution stops before that stage, those vectors can be empty. A caller should check the status and vector availability before indexing into them.

The distinction between API-level success, solver termination, feasibility, stationarity, and global optimality is essential when integrating POUNCE into an automated optimization system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 4. General nonlinear optimization

### 4.1 The IPOPT-derived interior-point method

POUNCE's primary nonlinear solver addresses

\\[ \begin{aligned} \min_x\quad & f(x)\\\ \text{s.t.}\quad & g_L\leq g(x)\leq g_U\\\ &x_L\leq x\leq x_U. \end{aligned} \\]

The problem can contain nonlinear equalities, nonlinear inequalities, variable bounds, and nonconvex objectives.

The core algorithm is a primal-dual interior-point method with filter line search, following the Wächter–Biegler IPOPT formulation.

Rather than imposing a conventional weighted penalty on infeasibility and objective deterioration, the filter framework assesses trial steps against objective and constraint violation.

The solver combines barrier subproblems, sparse Newton/KKT solves, inertia control, backtracking, restoration, and multiple convergence checks.

This is a substantial algorithmic port rather than merely using IPOPT's C interface from Rust.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 4.2 Barrier parameter strategies

POUNCE supports the principal IPOPT-style barrier update strategies:

| Strategy                   | Mechanism                                                          |
| -------------------------- | ------------------------------------------------------------------ |
| `monotone`                 | Progressive reduction of the barrier parameter                     |
| `adaptive`                 | Chooses barrier parameters using the current iterate and an oracle |
| `quality-function` oracle  | Evaluates candidate barrier parameters using a quality function    |
| `loqo` / `probing` oracles | Alternative adaptive barrier-selection methods                     |

Important controls include `mu_strategy`, `mu_init`, `mu_min`, `mu_max`, and the oracle configuration.

The barrier parameter \\(\mu\\) controls the relaxation of complementarity. Aggressive reductions may improve local convergence but can worsen conditioning; conservative reductions can improve robustness while increasing iteration counts.

These strategies can produce meaningfully different trajectories on degenerate, highly nonlinear, or poorly scaled models.

The ability to select and tune the barrier strategy is particularly relevant when a solver appears stalled despite improving primal feasibility.

Source: `https://kitchingroup.cheme.cmu.edu/pounce/options.html`

### 4.3 Hessian strategies

Hessian evaluation can dominate the runtime and memory consumption of large nonlinear optimization problems.

POUNCE supports the following mechanisms across its NLP and SQP paths:

| Mechanism                               | Characteristics                                                                   |
| --------------------------------------- | --------------------------------------------------------------------------------- |
| Exact Lagrangian Hessian                | Uses second derivatives provided by the problem interface                         |
| Limited-memory BFGS                     | Approximates curvature without constructing a dense full Hessian                  |
| Damped BFGS                             | Alternative positive-curvature quasi-Newton approximation on the SQP path         |
| Finite-difference Hessian approximation | Numerical approximation when explicit second derivatives are unavailable          |
| Sparse Hessian structure                | Enables sparse storage and repeated numerical factorization                       |
| Inertia correction                      | Adjusts problematic KKT/Hessian systems to obtain an appropriate search direction |

Exact second derivatives can improve convergence substantially for strongly nonlinear and ill-conditioned problems. However, their generation and storage are not free.

L-BFGS is particularly useful when Hessian construction is disproportionately expensive. In POUNCE's SQP path, it also avoids the dense \\(O(n^2)\\) storage of full BFGS.

The limited-memory implementation additionally supports sparse/low-rank handling of quasi-Newton information rather than requiring the full approximation to be explicitly materialized in every KKT system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 4.4 Restoration and degeneracy handling

A significant part of a production NLP solver is devoted to situations where the main algorithm cannot accept a useful step.

POUNCE implements:

- Feasibility restoration when filter-line-search progress fails.
- Soft and hard restoration mechanisms.
- Inertia adjustment and regularization.
- Recovery from difficult or inconsistent initial conditions.
- An \\(\ell_1\\) exact-penalty formulation for certain degenerate and complementarity-constrained problems.

The `pounce-l1penalty` crate implements a Thierry–Biegler-style penalty-barrier approach intended to help with degenerate constraints and mathematical programs with complementarity constraints (MPCCs).

MPCC support is not the same as a general discrete or complementarity branch-and-bound solver. The penalty treatment aims to obtain useful local nonlinear solutions in situations where conventional constraint qualifications fail.

POUNCE also supplies a second-opinion recovery ladder through `pounce-restoration`. When an ordinary run fails, this mechanism can attempt alternative scaling, barrier schedules, or starting points before returning a final verdict.

The builder and supported higher-level frontends invoke this recovery mechanism automatically, while a caller driving `IpoptApplication` directly can invoke `run_second_opinion_ladder`.

The recovery ladder carries additional computational cost on failures; its extra attempts should not be mistaken for free robustness.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



## 5. Sequential quadratic programming and active-set optimization

POUNCE includes an SQP implementation distinct from the filter-IPM algorithm.

At a current iterate \\(x_k\\), SQP constructs a local quadratic approximation to the nonlinear program, solves it, and updates the nonlinear iterate.

A simplified subproblem has the form

\\[ \begin{aligned} \min_d\quad& \nabla f(x_k)^Td+\frac12d^TB_kd\\\ \text{s.t.}\quad& g_L\leq g(x_k)+J_g(x_k)d\leq g_U. \end{aligned} \\]

Here \\(B_k\\) approximates the Hessian of the Lagrangian.

### 5.1 Active-set working sets

The `pounce-qp` engine explicitly maintains the active constraints and bounds involved in solving its QP subproblems.

This is a consequential distinction from a conventional interior-point solver.

An interior-point solver approaches active constraints through the central path, while an active-set solver can operate directly on a hypothesized active constraint set.

When repeatedly solving related optimization problems, POUNCE can retain the previously identified working set.

Its warm-start state includes:

\\[ (x,\lambda_g,\lambda_x,\mathcal W) \\]

where \\(\mathcal W\\) identifies active bounds and constraints.

The solver can exploit this information so that a slightly perturbed problem does not have to rediscover its active constraints.

This creates a useful architectural opportunity for receding-horizon control, continuation, parametric optimization, and repeated local refinements.

### 5.2 SQP algorithmic features

The implemented SQP functionality includes:

| Capability              | Mechanism                                                              |
| ----------------------- | ---------------------------------------------------------------------- |
| Sparse QP subproblems   | Triplet-based KKT assembly and sparse factorization                    |
| Indefinite Hessians     | Inertia-controlled factorization and shifting                          |
| Working-set reuse       | Carries active constraints across related solves                       |
| Parametric homotopy     | Tracks solutions as QP parameters change                               |
| Elastic feasibility     | Introduces penalized slacks when linearized constraints are infeasible |
| Anti-cycling            | Bland's rule, Harris ratio tests, GMSW EXPAND                          |
| Sparse Schur updates    | Updates factorizations as the working set changes                      |
| Second-order correction | Addresses the Maratos effect on nonlinear constraints                  |
| SQP globalization       | Filter or \\(\ell_1\\)-elastic merit-function approaches               |
| Hessian approximations  | Exact, damped BFGS, or L-BFGS                                          |

These features matter in combination. For instance, an active-set method with warm starts but without reliable handling of changing working sets can become brittle precisely where continuation crosses a constraint boundary. POUNCE addresses that through add/drop steps, homotopy, elastic modes, and fallback behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 5.3 Interior-point to SQP handover

A notable capability is IPM-to-SQP handover.

A difficult optimization problem can be approached initially through the interior-point method. Once sufficiently close to a solution, the active-set SQP path can take over, identify the binding constraints, and complete the solve.

The option family includes `ipm_sqp_handover` and its tolerance controls.

This is useful when the interior-point method provides robust progress from a cold start, but an active-set method can finish more efficiently near a well-defined solution.

The October 2026 release notes identify this as an important enhancement in POUNCE 0.14.0, alongside sparse low-rank L-BFGS handling for large SQP models.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 5.4 Rust solver selection

The normal NLP interface can activate SQP through an option:

```
let result = Nlp::new(problem)
    .x0(&initial_values)
    .option_str("algorithm", "active-set-sqp")
    .solve();
```

This direct algorithm selection does not require the `qp` facade feature. That feature is required when exposing the additional QP types and explicit SQP working-set API through `pounce-rs`.

For cross-solve working-set reuse, the lower-level API exposes `SqpIterates`, `set_sqp_warm_start`, `last_sqp_working_set`, and related interfaces.

The distinction between selecting SQP for one solve and explicitly managing working sets across solves is important.

Source: `https://kitchingroup.cheme.cmu.edu/pounce/active-set-sqp.html`

## 6. Linear and quadratic programming

The `pounce-convex` crate supplies dedicated algorithms for structured convex problems.

A conventional convex QP is represented as

\\[ \begin{aligned} \min_x\quad &\frac12x^TPx+c^Tx\\\ \text{s.t.}\quad &Ax=b\\\ &Gx\leq h\\\ &l\leq x\leq u \end{aligned} \\]

with \\(P\succeq0\\).

Setting \\(P=0\\) gives a linear program.

### 6.1 Dedicated convex interior-point solver

Unlike the general NLP path, the convex solver exploits the known mathematical structure of the problem.

It uses Mehrotra predictor–corrector optimization and a homogeneous self-dual embedding (HSDE) formulation.

Important consequences include:

- Convex global-optimality guarantees subject to numerical convergence.
- Primal and dual solution information.
- Infeasibility and unboundedness detection through checked certificates.
- Specialized KKT systems.
- Presolve and dual reconstruction.
- Warm-start and batch-solving capabilities.

A solver that fails to produce a valid optimality or infeasibility certificate should not be assumed to have established one merely because it stopped iterating.

### 6.2 Direct Rust QP interface

The `convex` feature exposes `QpProblem`, `QpOptions`, `Triplet`, and `solve_qp_ipm`.

Its model separates the quadratic Hessian (`p_lower`), linear objective (`c`), equalities (`a`, `b`), inequalities (`g`, `h`), and variable bounds.

The lower triangle of the quadratic Hessian is stored in triplet form.

This avoids having to express an ordinary convex quadratic program as an arbitrary nonlinear callback model.

Source: `https://kitchingroup.cheme.cmu.edu/pounce/convex-solver.html`

### 6.3 Cold solves versus active-set sequences

The two QP engines have different tradeoffs.

| Property                     | Convex IPM                            | Active-set QP                                |
| ---------------------------- | ------------------------------------- | -------------------------------------------- |
| Convex LP/QP                 | Yes                                   | Yes                                          |
| Indefinite objective Hessian | Not supported as a valid convex solve | Supported for local solutions                |
| Cold-start robustness        | Generally stronger                    | More dependent on working-set identification |
| Parametric sequences         | Warm starts and factorization reuse   | Explicit working-set reuse and homotopy      |
| Exact active-bound solution  | Converges toward the active boundary  | Can identify it explicitly                   |
| Global nonconvex guarantee   | No                                    | No                                           |

POUNCE's documentation reports that its convex interior-point solver solved 137 of 138 problems in its Maros–Mészáros QP benchmark tier, substantially more than the cold active-set path.

That comparison is specific to the reported benchmark suite and should not be generalized to all QP workloads. It supports the architectural distinction: cold solve robustness favors the interior-point approach, whereas active-set reuse can be especially valuable across related models.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 6.4 Factorization reuse and batching

POUNCE exposes more than warm-starting a previous solution.

`QpFactorization` can retain symbolic analysis and ordering information when the KKT sparsity structure is unchanged.

This is valuable because the numerical matrix entries may change even when its graph does not.

The package also exposes batch-solving routines, including `solve_qp_batch_parallel`, using Rayon to distribute independent optimization problems across workers.

A persistent `ActiveSetSession` supplies another kind of reuse: it retains the prior QP state and attempts parametric continuation rather than solving each instance cold.

Its reuse status distinguishes successful homotopy, working-set reuse, and cold fallback. A warm attempt is not assumed to be valid; its result is verified.

A meaningful limitation is that parametric homotopy has structural requirements. For example, changing the Hessian or changing whether a constraint is an equality can disqualify the homotopy route, although other warm-start methods may remain possible.

The combination of numerical warm starts, symbolic-factor reuse, working-set reuse, and independent batch parallelism offers several distinct ways to reduce repeated-solve overhead.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



## 7. Conic and semidefinite optimization

POUNCE's convex engine extends beyond LP and QP.

Its conic formulation uses constraints of the form

\\[ h-Gx\in K \\]

where \\(K\\) is a product of supported convex cones.

### 7.1 Supported cone families

| Cone                        | Representative use                                       |
| --------------------------- | -------------------------------------------------------- |
| Nonnegative orthant         | Linear inequalities                                      |
| Second-order / Lorentz cone | Norm constraints, SOCP, convex quadratic constraints     |
| Exponential cone            | Entropy, exponential and logarithmic convex formulations |
| Power cone                  | Power-law convex inequalities, norms and geometric means |
| Positive-semidefinite cone  | Semidefinite optimization and matrix inequalities        |

Second-order cones permit formulations such as

\\[ \\|Ax+b\\|\_2\leq c^Tx+d. \\]

This can express norm minimization, trust-region-like constraints, convex quadratic inequalities, and some robust optimization problems.

Exponential and power cones extend the expressiveness to models that cannot naturally be represented as standard LP or QP problems without approximation.

### 7.2 Distinct numerical treatment of cone types

POUNCE uses different numerical machinery for its cone families.

For symmetric cones, the implementation uses Nesterov–Todd scaling. Exponential and power cones use a non-symmetric HSDE approach.

This distinction leads to an important compatibility constraint:

Positive-semidefinite cone blocks cannot currently be mixed with exponential or power cone blocks in a single POUNCE problem.

PSD functionality is also principally intended for smaller dense semidefinite problems; it should not be treated as equivalent to a specialized large-scale SDP system.

### 7.3 Convex QCQP reformulation

POUNCE can recognize certain convex quadratically constrained quadratic programs (QCQPs) and reformulate their quadratic constraints into second-order cones.

For a quadratic inequality

\\[ \frac12 x^TQx+a^Tx+b\leq0,\qquad Q\succeq0, \\]

factorization of \\(Q\\) allows the convex quadratic form to be represented through a second-order-cone constraint.

This means some models that appear to require a general NLP solver can instead be solved through a specialized convex algorithm with global-optimality guarantees.

The automatic routing is conservative: when the solver cannot establish convexity, it can fall back to the general NLP path rather than silently applying an invalid reformulation.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/choosing-a-solver.html`

`https://kitchingroup.cheme.cmu.edu/pounce/lp-qp-routing.html`

## 8. Polynomial global optimization and multiple-minimum discovery

POUNCE provides two fundamentally different approaches to nonconvex optimization: certified polynomial optimization and heuristic searches over local optima.

They should not be conflated.

### 8.1 Sum-of-squares and the Lasserre hierarchy

The `pounce-convex` crate implements polynomial global optimization through sum-of-squares (SOS) and moment semidefinite relaxations.

For a polynomial optimization problem

\\[ \min_x p(x) \quad\text{subject to}\quad g_i(x)\geq0,\quad h_j(x)=0, \\]

the method constructs an SDP relaxation that produces a lower bound on the objective. The algebraic certificates are based on representing nonnegative polynomials through sums of squares and constraint multipliers.

Increasing the relaxation order can tighten the lower bound, but the associated moment and localizing matrices become larger.

Important implemented functionality includes:

- Polynomial representation through `Polynomial` and `PolyProblem`.
- A Rust `sos_minimize` entry point, available through the `convex` feature.
- Polynomial equality and inequality constraints.
- Configurable Lasserre relaxation order.
- Lower-bound and moment-certificate results.
- Flat-truncation checks to identify exact relaxations.
- Facial-reduction-based recovery of multiple global minimizers.

The last capability is especially valuable: the solver may identify distinct globally optimal solutions rather than returning only one point. For example, the polynomial \\(x^4-2x^2+3\\) has global minimum 2 at both \\(x=-1\\) and \\(x=1\\).

A concise Rust example, adapted from the project documentation, is:

```
use pounce_rs::convex::{sos_minimize, PolyProblem, Polynomial};
use pounce_rs::linsol::backend;

let p = Polynomial::new(
    1,
    vec![
        (vec![4], 1.0),
        (vec![2], -2.0),
        (vec![0], 3.0),
    ],
);

let result = sos_minimize(&PolyProblem::new(p), None, backend);
```

The key qualification is that an SOS relaxation is not automatically exact at every chosen order. An exactness check and sufficiently accurate numerical solution are essential to interpreting its lower bound as the true global optimum.

Furthermore, finite-precision numerical certificates should not be confused with independently verified formal proofs.

The approach is most attractive for modest-dimensional polynomial problems. Higher polynomial degree, higher relaxation order, and increasing dimension can rapidly make the semidefinite program impractical.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/global-optimization.html`

### 8.2 Heuristic exploration of multiple local minima

The `minima` feature exposes `pounce_rs::minima`, backed by the `pounce-minima` crate.

Rather than form an SDP, this component repeatedly invokes a local nonlinear solver and maintains an archive of distinct minima.

Six search strategies are supported:

| Strategy       | Search mechanism                                        | Distinctive role                                         |
| -------------- | ------------------------------------------------------- | -------------------------------------------------------- |
| `multistart`   | Independent random or scrambled Sobol starts            | Parallelizable baseline                                  |
| `mlsl`         | Multi-Level Single-Linkage                              | Avoid redundant descents into previously explored basins |
| `basinhopping` | Perturb and locally minimize with Metropolis acceptance | Traverse a rugged landscape of minima                    |
| `flooding`     | Add adaptive Gaussian repulsion near found minima       | Destabilize previously discovered basins                 |
| `deflation`    | Add softened inverse-distance repulsive terms           | Exclude previously discovered solutions                  |
| `tunneling`    | Modified objective designed to escape known minima      | Continue toward alternative, potentially lower minima    |

The Rust library exposes `LocalSolve`, `IpoptLocalSolve`, `find_minima`, and related configurable search machinery. Its local solver can be supplied through an abstraction, rather than requiring every search strategy to own a separate optimizer.

The search engine includes useful safeguards and accounting:

- Deduplication by variable-range-scaled distance.
- Solver-call budgets, patience limits, and target minimum counts.
- Reproducible sampling seeds.
- Hessian-based saddle rejection when curvature is available.
- Final polishing on the unmodified objective after repulsion-based search.
- Distinct termination reasons such as target reached and budget exhausted.

For the repulsion strategies, POUNCE can use limited-memory curvature on the modified objective and polish candidates with the original problem. This prevents the repulsive terms from necessarily imposing dense modified-Hessian storage throughout the search.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



Neither multistart nor any of these five other search strategies certifies global optimality. They can find a very good solution—or multiple apparently distinct local solutions—without establishing that unexplored regions contain no better point.

The package's separate Python API also provides stationary-point enumeration, saddle search, and reaction-network construction. Those higher-level Python routines should not be assumed to exist as identical public Rust facade APIs merely because the underlying solver is written in Rust.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/find-minima.html`

`https://kitchingroup.cheme.cmu.edu/pounce/find-minima-choosing.html`

`https://kitchingroup.cheme.cmu.edu/pounce/find-minima-cli.html`

## 9. Presolve, bound propagation, and structural simplification

POUNCE has preprocessing capabilities at both the general NLP level and within its specialized convex solver.

### 9.1 Nonlinear bound propagation

The `pounce-presolve` infrastructure can perform feasibility-based bound tightening (FBBT).

Its principle is to propagate known variable intervals forward through expression graphs and use constraint bounds to propagate restrictions backward.

For example, if

\\[ x+y\leq10,\qquad x\geq7,\qquad y\geq0, \\]

reverse propagation can infer \\(y\leq3\\) without invoking the nonlinear solver.

The same principle extends to suitable nonlinear expressions using interval arithmetic.

The Rust-side machinery includes `FbbtTape`, `FbbtOp`, `TightenReport`, and `FbbtReport`. Supported elementary operations cover arithmetic and selected nonlinear operators, including integer powers, square roots, exponentials, logarithms, and absolute values, subject to domain and implementation limitations.

Two qualifications materially affect the feature's applicability.

First, it requires access to the constraint expression structure, not just opaque numerical callbacks. AMPL `.nl` models provide such structure, while native Rust models can expose it through the `Problem::constraint_expression` facility. A general `TNLP` implementation providing only numerical evaluations does not automatically supply an expression tape.

Second, nonlinear FBBT is opt-in: the relevant configuration requires both `presolve=yes` and `presolve_fbbt=yes`.

Because bound propagation can change the feasible domain, the expression tape must faithfully represent the actual constraint callbacks. An incorrect expression graph is not merely a performance problem; it can yield incorrect bound reductions.

POUNCE includes consistency checks at sample points, but those are not a mathematical proof that the graph and callback are identical everywhere.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/fbbt.html`

### 9.2 Structural NLP preprocessing

Additional preprocessing can identify and simplify:

- Fixed variables and corresponding substitutions.
- Linear equalities eligible for elimination.
- Redundant or dependent constraints.
- Certain linear-independence constraint qualification (LICQ) problems.
- Bound inconsistencies detectable before numerical iteration.
- Auxiliary equalities and associated degeneracy issues.

These transformations can reduce the size of a KKT system, improve numerical conditioning, or prevent wasted work on a provably infeasible formulation.

The Rust API exposes `PresolveTnlp`, `PresolveOptions`, `wrap_with_presolve`, and related diagnostics.

There is an important distinction between wrapping a problem in `PresolveTnlp` and invoking the complete high-level preprocessing orchestration: some outer transformations, such as linear-equality elimination, may be handled at a different layer.

Presolve should therefore be understood as a sequence of transformations with recorded mappings, not as a single monolithic algebraic rewrite.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 9.3 Dedicated convex presolve and postsolve

The LP/QP path performs additional linear-algebraic and polyhedral simplifications inspired by established LP presolvers.

These include empty-row and empty-column detection, singleton constraints, bound tightening, redundant rows, substitutions, and related structural reductions.

Its postsolve logic reconstructs original-space primal and dual quantities. That reconstruction is necessary because a solution of the reduced model is not directly usable as a solution of the original model when variables and constraints have been eliminated.

The convex path also verifies relevant certificates against the original model before reporting them.

An important integration detail is that presolve defaults differ by entry point. The CLI's convex routing can enable preprocessing automatically, whereas some direct programmatic QP entry points leave it disabled unless requested.

Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/lp-qp-routing.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



## 10. Post-optimal sensitivity analysis and differentiation

Sensitivity analysis is one of POUNCE's most consequential capabilities because it can extract information from a converged optimization problem without solving a fully independent optimization problem for every parameter perturbation.

### 10.1 Differentiating the optimal solution

For a parametric optimization problem

\\[ x^\*(p)=\arg\min_x f(x,p) \\]

subject to parameter-dependent constraints, the goal is to estimate

\\[ \frac{\partial x^\*}{\partial p}. \\]

Under suitable regularity assumptions and a stable active set, the sensitivity can be derived by differentiating the KKT conditions.

Writing those conditions as \\(F(z,p)=0\\), where \\(z\\) contains primal and dual variables, gives

\\[ \frac{\partial F}{\partial z} \frac{\partial z}{\partial p} = -\frac{\partial F}{\partial p}. \\]

Thus, sensitivity becomes a linear solve involving the derivative of the KKT system.

Because POUNCE already factorizes a related KKT matrix during optimization, it can reuse this numerical work for post-optimal queries.

The `pounce-sensitivity` crate implements an sIPOPT-derived sensitivity system for general NLPs. It provides:

- First-order changes in the optimal primal variables.
- Perturbed multiplier estimates.
- Parameterized warm-start predictors.
- Bound-aware sensitivity refinement.
- Reduced-Hessian calculations and eigenanalysis.
- KKT factor reuse across multiple derivative queries.
- Diagnostics about the reliability of a predicted change.

These capabilities are exposed through the `sensitivity` feature in the Rust facade.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 10.2 Rust `SensSolve` interface

The published Rust interface includes a builder that ties sensitivity calculations to the converged nonlinear solver.

An excerpt illustrating the API shape:

```
use pounce_rs::sensitivity::SensSolve;

// Assumes `app` is a configured IpoptApplication
// and `tnlp` is the associated problem.
let result = SensSolve::new(vec![2, 3])
    .with_deltas(vec![0.05, 0.0])
    .with_reduced_hessian()
    .with_reduced_hessian_eigen()
    .run(&mut app, tnlp);
```

The specified indices identify the parameter-pinning constraint rows, while `with_deltas` supplies the perturbations.

`SensSolve` also offers bound refinement through `with_boundcheck(eps)`.

At the lower level, POUNCE exposes a retained-factor sensitivity `Solver`, including operations such as `kkt_solve`, parametric steps, reduced-Hessian computations, and activity classification.

The distinction matters: a one-shot `SensSolve` is convenient, but a retained solver/factorization is particularly relevant when many sensitivity queries follow one converged solve.

The Rust package currently implements the first sIPOPT perturbation tier; arbitrary `n_sens_steps` values greater than one are not supported through the sIPOPT-compatible option interface.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 10.3 Active-set changes: linear prediction, fix-relax, and path following

A linear sensitivity predictor assumes that the local active set remains valid.

That assumption fails when a perturbed optimum reaches a bound, leaves a bound, or crosses a degenerate point where complementarity changes.

POUNCE has three related approaches:

| Mechanism         | Behavior                                                                           | Limitation                                                    |
| ----------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Linear prediction | Applies the KKT-derived first-order step                                           | May violate bounds or constraints after clamping              |
| Fix-relax         | Pins variables hitting bounds and releases bounds whose multipliers lose validity  | More KKT work; still an approximation for a nonlinear problem |
| Path following    | Traverses the parameter change in segments, updating the active set at breakpoints | NLP curvature is still linearized around the base solution    |

Path following identifies events such as a free variable reaching a bound or an active bound being released.

For suitably structured parametric QPs, a fixed-Hessian active-set path can be piecewise affine and therefore exact within the numerical solve tolerance. For general NLPs, the path remains an approximation because the nonlinear model is not fully relinearized at every breakpoint.

The project exposes this machinery through the Python sensitivity API and through lower-level Rust sensitivity components. The exact convenience entry points differ between languages; Python's `sens_solution(mode="path")` is not a Rust facade method of the same name.

There is also an important numerical subtlety: adding an active bound can often be handled through a Schur-complement update; releasing a tightly active bound may require refactorization because its contribution to the original barrier KKT matrix can be too large to remove accurately from the retained factor.

This makes active-set transitions an important predictor of sensitivity-query cost.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 10.4 Sensitivity at kinks and degenerate solutions

A bound can be weakly active: the solution lies on the bound, but the associated multiplier is approximately zero.

At such a point, a two-sided derivative of the optimizer with respect to a parameter may not exist.

POUNCE distinguishes activity classifications including strongly active, weakly active, inactive, and ambiguous.

For weakly active bounds, the sensitivity engine can use directional decision machinery to determine which constraints should engage for the requested perturbation direction.

The NLP and convex sensitivity implementations share parts of this decision logic through `pounce-sens-core`.

This is more sophisticated than simply differentiating a converged primal solution and assuming every near-active constraint stays fixed.

Nevertheless, degeneracy, nonunique solutions, singular KKT systems, or changes in the local optimum can invalidate a sensitivity prediction. A directional derivative is not a substitute for a global regularity guarantee.

There is a performance tradeoff in activity classification: refining an ambiguous classification can require another KKT backsolve per entry. On large models, querying every ambiguous constraint can cost considerably more than querying selected quantities of interest.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 10.5 Reduced Hessians: an important sign convention

The NLP sensitivity machinery can produce a reduced Hessian associated with selected parameters or pinned rows.

POUNCE's sIPOPT-derived pin-row reduced Hessian is returned with a negative sign.

If \\(H_R\\) denotes the conventional positive-curvature reduced Hessian, the reported NLP quantity is

\\[ H\_{\mathrm{reported}}=-H_R. \\]

This affects both direct matrix interpretation and eigenanalysis.

For a positive-definite minimum, the reported NLP pin-row matrix can have all-negative eigenvalues without indicating a numerical failure.

The sign convention does not transfer directly to the convex QP sensitivity implementation:

| API family   | Reduced-Hessian method                             | Convention |
| ------------ | -------------------------------------------------- | ---------- |
| NLP / sIPOPT | KKT inverse / Schur-complement route over pin rows | \\(-H_R\\) |
| Convex QP    | Null-space projection                              | \\(+H_R\\) |

Consequently, two outputs named “reduced Hessian” are not semantically interchangeable. Calling code must preserve their mathematical definitions rather than unifying them solely by type or field name.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 10.6 Covariance, information matrices, and identifiability

POUNCE's retained KKT machinery supports a further application: statistical analysis of nonlinear parameter-estimation problems.

For a suitable least-squares objective, KKT inverse blocks or the reduced Hessian can be used to construct asymptotic parameter covariance estimates.

That enables:

- Standard errors for fitted parameters.
- Covariance and correlation matrices.
- Identifiability analysis through eigenvalues and eigenvectors.
- Information-matrix analysis.
- Conditional treatment of active parameter bounds.
- Heteroscedastic noise-group calculations in the higher-level Pyomo integration.
- Observed-Lagrangian versus Gauss–Newton information approximations.

These are post-solve calculations, not repeated finite-difference optimization experiments.

The richer declarations and convenience functions—such as `declare_sens_fitted`, `declare_sens_residual`, `sens_covariance`, and `sens_information`—belong to the `pyomo-pounce` integration.

They should not be described as native `pounce-rs` functions. The numerical sensitivity core is reusable from Rust, but the Pyomo-specific model introspection, residual declarations, and statistical result presentation live in the companion integration.

The statistical assumptions matter: an inverse Hessian does not automatically constitute a valid covariance matrix for arbitrary objectives. Noise scaling, residual weighting, parameter identifiability, and local curvature must be interpreted consistently.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/sensitivity.html`

## 11. Persistent NLP sessions and repeated-solve execution

POUNCE offers an additional abstraction for repeated nonlinear solves: `TnlpPresolveSession`, exposed through `pounce_rs::session`.

This solves a different problem from one-shot warm-starting.

### 11.1 What a session retains

A session maintains a solver application and tracks mappings between the original problem variables and the reduced representation produced by presolve.

Its logical execution path is:

User `TNLP` — original model space

Warm-start injector

Stage previous primal/dual solution

Presolve and linear-equality elimination

Project starting point into reduced space

NLP solver

Original-space solution and next warm seed

The session supports cold solves and warm solves using the previous solution, and it records information about how the warm point was transformed.

Presolve transformations are cached when their relevant inputs remain unchanged. If a fingerprint detects that the transformation is stale, POUNCE rebuilds it and maps the current warm seed through the new transformation.

An explicit `invalidate()` method is necessary when the fingerprint cannot observe a material change—for example, changes in FBBT expression tapes or certain nonlinear data involved in auxiliary preprocessing.

Two other distinctions are operationally significant:

- Sessions do not automatically invoke the high-level second-opinion ladder on failure.
- Per-iteration history must be explicitly enabled for session solves.

A session therefore provides stateful reuse and a controlled warm-start lifecycle, not an implicit guarantee of the same recovery behavior as every other POUNCE entry point.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



API: `https://docs.rs/pounce-rs/latest/pounce_rs/session/index.html`

### 11.2 Combining the reuse mechanisms

POUNCE's reuse facilities address different costs:

| Mechanism                       | Reused information                               | Main benefit                                             |
| ------------------------------- | ------------------------------------------------ | -------------------------------------------------------- |
| NLP warm start                  | Primal/dual point                                | Better initial iterate                                   |
| `TnlpPresolveSession`           | Warm seed and presolve mappings                  | Avoid repeated transformation work                       |
| `QpFactorization`               | Sparse KKT structure and symbolic analysis       | Avoid repeated symbolic factorization                    |
| `ActiveSetSession`              | Active working set and parametric state          | Avoid rediscovering active constraints                   |
| NLP sensitivity retained factor | Converged KKT factorization                      | Compute derivatives with backsolves                      |
| IPM-to-SQP handover             | A near-optimal solution and candidate active set | Combine cold-start robustness with active-set refinement |

These mechanisms are complementary but not interchangeable.

For example, retaining a KKT factor from a converged NLP is useful for local sensitivity queries. It does not mean that factor remains valid after arbitrary structural changes to the nonlinear problem.

Similarly, reusing symbolic sparse analysis is not equivalent to reusing numerical factorization: if matrix coefficients change, the factor usually must be recomputed even if its symbolic elimination pattern remains valid.

This separation makes POUNCE's stateful interfaces valuable without obscuring the conditions under which their cached information is valid.

## 12. Sparse linear algebra and numerical performance

All major POUNCE algorithms ultimately depend on solving sparse linear systems. For many large NLPs, sparse factorization rather than objective evaluation is the dominant runtime cost.

### 12.1 FERAL: the default sparse symmetric solver

POUNCE uses FERAL as its default KKT factorization engine.

FERAL performs sparse symmetric-indefinite LDLᵀ factorization:

\\[ P^TAP=LDL^T, \\]

where \\(P\\) is a fill-reducing permutation, \\(L\\) is a lower-triangular factor, and \\(D\\) contains the pivot blocks.

Its capabilities include:

- Symmetric-indefinite factorization with numerical pivoting.
- Positive, negative, and zero inertia counts.
- Sparse supernodal factorization machinery.
- Multiple fill-reducing ordering algorithms.
- Reuse of symbolic analysis when the sparsity pattern remains constant.
- Multiple-right-hand-side solves.
- Numerical scaling and iterative refinement.
- Parallel factorization and solve operations on supported CPU configurations.

KKT inertia is not simply a diagnostic statistic. Interior-point algorithms use it to determine whether a Newton direction has the appropriate curvature properties. A factorization can finish numerically yet still have inertia unsuitable for the optimization step.

Because FERAL is written in Rust, POUNCE does not require BLAS, LAPACK, Fortran, or an externally installed sparse solver for its default numerical path.

FERAL is also an independently usable library; its additional capabilities, such as its separate sparse LU basis engine, should not automatically be interpreted as POUNCE solver algorithms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

feral+1



FERAL documentation: `https://kitchingroup.cheme.cmu.edu/feral/`

### 12.2 Fill-reducing ordering

Sparse factorization performance is strongly influenced by the elimination ordering.

POUNCE exposes FERAL ordering controls such as:

- Approximate minimum degree (AMD).
- Approximate minimum fill (AMF).
- METIS-style nested dissection.
- SCOTCH-style nested dissection.
- KaHIP-style graph partitioning.
- Automatic ordering selection.
- `auto_race`, which pays additional symbolic-analysis cost to compare candidates.

Different orderings can change fill, memory usage, and factorization time by large amounts even when the mathematical problem is identical.

There are two opposing costs: a more sophisticated ordering may produce a smaller numerical factor, but discovering it consumes additional setup time.

Expensive ordering analysis is more attractive when a problem preserves its KKT pattern over many Newton iterations or repeated solves. If the model structure changes continually, the amortization is weaker.

The exact algorithms included in `auto_race` and the automatic dispatch heuristics are FERAL-version-dependent; they should not be hardcoded from an older POUNCE troubleshooting example.

Source: `https://kitchingroup.cheme.cmu.edu/feral/ordering.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

feral



### 12.3 Numerical scaling and refinement

POUNCE distinguishes scaling at two levels:

1. NLP-level scaling, which changes the relative magnitudes of objective and constraint derivatives presented to the optimizer.
2. Linear-system scaling, which equilibrates the assembled KKT matrix to improve numerical factorization and backsolve accuracy.

Relevant options include `nlp_scaling_method`, `linear_system_scaling=ruiz`, `feral_scaling`, `feral_pivtol`, and iterative-refinement controls.

The difference is substantial. A model can have appropriately scaled physical variables but still produce a badly conditioned augmented KKT matrix, especially close to convergence when barrier terms become extreme.

POUNCE's troubleshooting documentation includes cases where Ruiz scaling improves strict-convergence status or materially reduces runtime. Such results are model-dependent, not an assurance that scaling always improves a solve.

The underlying factorization can also affect the NLP trajectory: different numerical pivots may perturb the Newton step enough to produce different barrier updates or local outcomes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 12.4 Optional HSL MA57 backend

POUNCE also supports HSL MA57 through `pounce-hsl`.

The installation requires the appropriate CoinHSL library and license, a build with the `ma57` feature, and explicit selection:

```
pounce problem.nl linear_solver=ma57
```

Enabling the Cargo feature makes the backend available; it does not automatically make MA57 the default.

A subtle compatibility limitation is that POUNCE recognizes several inherited IPOPT linear-solver option names without implementing every associated backend.

For example, names corresponding to MUMPS, MA27, MA97, PARDISO, and SPRAL may be recognized by the option machinery but are not implemented selectable POUNCE backends. Selecting such a name is rejected rather than silently using FERAL.

This is particularly important when reusing IPOPT configuration files: parsing an option name does not establish the availability of that solver.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



Installation and build reference: `https://kitchingroup.cheme.cmu.edu/pounce/installation.html`

### 12.5 CPU and GPU execution boundary

POUNCE's documented numerical backends are CPU sparse direct solvers: pure-Rust FERAL and optional MA57.

I did not find a documented native CUDA, ROCm, or GPU sparse-factorization backend for POUNCE 0.16.0.

The JAX and PyTorch companion integrations can use accelerated tensor operations for parts of model evaluation or derivative generation, but that is distinct from running POUNCE's interior-point or SQP linear-system factorization on a GPU.

Consequently, GPU-enabled autodifferentiation should not be mistaken for GPU-enabled optimization execution. The solver's numerical KKT path remains CPU-based in the documented configurations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 13. Modeling, differentiation, and ecosystem interoperability

One of POUNCE's architectural strengths is that its numerical solver is available through several different model representations and integration layers.

### 13.1 AMPL `.nl` parsing and automatic differentiation

The `pounce-nl` crate reads AMPL nonlinear model files.

This is a richer integration than simply loading objective and constraint coefficients.

The `.nl` representation can carry:

- Expression trees for nonlinear objectives and constraints.
- Structural derivative information.
- Variable and constraint bounds.
- Linear and nonlinear terms.
- Sparsity patterns.
- Suffixes and metadata used by compatible optimization interfaces.

POUNCE's `.nl` evaluation machinery uses automatic-differentiation tapes to calculate derivatives.

It also implements structural sparsity exploitation, including coloring-based Hessian recovery and matrix-free Hessian-vector products through the corresponding Python `NlProblem` API.

A Hessian-vector product computes

\\[ H(x)v \\]

without assembling the complete Hessian. This is distinct from a sparse explicit Hessian, and it can be attractive for iterative linear algebra or sensitivity algorithms that consume directional curvature information.

POUNCE's principal NLP algorithm still uses its supported KKT factorization machinery; availability of matrix-free HVP evaluation does not imply a general matrix-free Newton–Krylov solver mode.

The `.nl` route can also classify linear and quadratic model structure and dispatch suitable problems to the dedicated convex engines instead of evaluating them as general nonlinear programs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



Python derivative interface: `https://kitchingroup.cheme.cmu.edu/pounce/python.html`

### 13.2 Automatic solver selection

POUNCE's CLI and modeling-language paths recognize several optimization problem classes and route accordingly.

| Identified model         | Default or eligible path           |
| ------------------------ | ---------------------------------- |
| LP                       | Convex LP interior-point solver    |
| Convex QP                | Convex QP interior-point solver    |
| Convex QCQP              | SOCP reformulation where supported |
| General smooth NLP       | Filter-IPM                         |
| Nonconvex QP             | General NLP local solver           |
| Parametric active-set QP | Explicit opt-in                    |
| Polynomial SOS           | Explicit `sos_minimize` workflow   |

Automatic routing is not arbitrary mathematical inference from an opaque user function.

It depends on access to a suitable structural expression representation and on conservative tests, including convexity and Hessian definiteness.

The available `solver_selection` values can explicitly choose `nlp`, `lp-ipm`, `qp-ipm`, `socp`, or `qp-active-set`.

The explicit active-set QP route is particularly important because the automatic classifier prefers the convex interior-point solver for one-shot convex QPs.

Some IPOPT options have different semantics or availability on the specialized convex paths. For unsupported requests, POUNCE may reject an explicit selection or fall back to the NLP route under automatic selection.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 13.3 Language and modeling-system interfaces

| Integration        | Mechanism                                            | Boundary                                                    |
| ------------------ | ---------------------------------------------------- | ----------------------------------------------------------- |
| Rust               | `pounce-rs` facade, `Problem`, `TNLP`, QP/conic APIs | Native library integration                                  |
| Python             | `pounce-solver` / `pounce` through PyO3              | Companion binding over Rust numerical core                  |
| SciPy-style Python | `pounce.minimize`                                    | Python API convenience layer                                |
| Pyomo              | `pyomo-pounce`, `SolverFactory("pounce")`            | Modeling plugin and `.nl`/in-process integration            |
| AMPL               | `.nl` input and `.sol` output                        | File-based solver protocol                                  |
| C                  | `pounce-cinterface`                                  | IPOPT-compatible C ABI                                      |
| Julia / JuMP       | IPOPT-compatible interface integration               | Companion interoperability; not native Rust modeling        |
| CasADi             | `nlpsol` plugin                                      | Requires separately configured integration                  |
| GAMS               | Solver-link integration                              | External modeling-system bridge                             |
| JAX                | `pounce.jax`                                         | Differentiable Python integration                           |
| PyTorch            | `pounce.torch`                                       | Autograd integration                                        |
| WebAssembly        | `pounce-wasm`                                        | Browser/WASI deployment, not a standard Rust facade feature |

This table describes supported integration routes, not a guarantee that every feature is identically available across every frontend.

For instance, an AMPL-compatible `.nl` file can carry structural derivatives and suffixes that are absent from a simple numerical Rust callback. Conversely, native Rust can retain state and directly orchestrate solver objects without a file round trip.

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+3

### 13.4 Differentiable optimization layers

The JAX and PyTorch interfaces expose optimization as an operation embedded in a differentiable computation.

This is typically achieved through implicit differentiation of the optimality conditions rather than by tracing every interior-point iteration.

The distinction is architecturally important:

- The forward pass solves an optimization problem.
- The backward pass uses local KKT sensitivity to propagate gradients through the optimal solution.
- A caller can include the resulting optimization layer inside a larger differentiable model.

The JAX integration offers JIT-compatible interfaces and sparse colored differentiation for suitably structured callback models. The PyTorch integration uses `torch.autograd.Function`.

These are companion Python APIs, not a guarantee that `pounce-rs` provides a native GPU tensor abstraction or native `torch`/JAX primitives.

As with any implicit optimization layer, differentiability becomes problematic at nonunique solutions, active-set transitions, constraint degeneracy, or singular KKT systems.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 13.5 Browser and WebAssembly deployment

POUNCE can run its Rust solver stack in WebAssembly, including the `.nl` reader, derivative tape, sparse factorization, and nonlinear optimizer.

The project provides browser demonstrations for both uploaded `.nl` models and Pyomo models running through Pyodide.

Its documented target is `wasm32-wasip1`, with a small WASI compatibility shim and a browser worker to avoid blocking the UI.

This offers an alternative deployment model: optimization can execute locally in the browser without a backend solving service.

The WebAssembly deliverable is a separate workspace component. It should not be assumed that every Cargo feature, native shared-library backend, or Python extension is portable unchanged into a browser.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/wasm.html`

## 14. Interactive optimization debugging and agent-facing tooling

POUNCE has an unusually extensive diagnostic environment built around the optimization process itself.

This functionality is distinct from ordinary solver logging and deserves attention because it exposes internal numerical state that otherwise requires custom instrumentation.

### 14.1 Live interior-point debugger

POUNCE's CLI includes an interactive debugger for the interior-point algorithms.

It can be started in several modes:

```
pounce problem.nl --debug
pounce problem.nl --debug-json
pounce problem.nl --debug-on-error
pounce problem.nl --debug-on-interrupt
```

The debugger can pause the optimizer at internal checkpoints, including:

| Checkpoint              | Available information                         |
| ----------------------- | --------------------------------------------- |
| `iter_start`            | Current accepted primal-dual iterate          |
| `after_mu`              | Updated barrier parameter                     |
| `after_search_dir`      | Newton direction, KKT inertia, regularization |
| `after_step`            | Accepted step and step lengths                |
| `step_rejected`         | Rejected step before restoration              |
| `pre_restoration_entry` | State that triggered feasibility restoration  |
| `post_restoration_exit` | State following restoration                   |
| `terminated`            | Final iterate and termination reason          |

These hooks allow an engineer to investigate why an iteration stalled, rather than merely observing that it stalled.

The debugger can inspect primal and dual variables, bound activity, constraint residuals, search directions, KKT matrices, and the LDLᵀ factorization.

It also supports conditional breakpoints, solver-event breakpoints, watchpoints, saved iterates, and re-solving from modified initial conditions.

Particularly useful events include KKT regularization, restoration entry, tiny steps, rejected line searches, stagnating barrier updates, and nonfinite values.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 14.2 Rewinding and changing a solve

The debugger provides `goto`, `restart`, `load`, `save`, and `resolve` functionality.

For example, an engineer can capture an iterate, change a barrier strategy, and launch another solve using that point as a starting state.

This is not bit-exact time travel.

The rewind operation restores the primal-dual iterate, but not necessarily the entire internal history of the filter, barrier-update strategy, or quasi-Newton approximation.

The distinction matters when evaluating numerical reproducibility: a re-solve from a recorded state is a new optimization trajectory, not necessarily a replay of the old trajectory.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 14.3 Programmatic debugger protocol

The `--debug-json` mode exposes a structured newline-delimited JSON protocol.

A client receives an initial `hello` message identifying supported commands, metrics, checkpoints, and capabilities.

It can then send commands and receive `pause`, `progress`, `result`, and `terminated` events, with request IDs for matching responses.

This is a meaningful integration facility for automated numerical diagnosis. A separate program can inspect solver behavior and modify its execution without parsing free-form console output.

The protocol is capability-discoverable because not every command is available for every optimization algorithm.

Limitation: the interactive debugger supports the NLP and convex/conic interior-point engines, but does not currently attach to the active-set SQP or active-set QP iteration loops. Some advanced KKT inspection and re-solve commands are NLP-only even though simpler debugging is shared with the conic solvers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/debugger.html`

### 14.4 POUNCE Studio and MCP integration

POUNCE also has a companion diagnostics ecosystem centered on `pounce-studio-core` and its Python/MCP integration.

This provides model inspection, pre-solve analysis, convergence diagnostics, comparison of solver runs, identification of stalls and restoration episodes, and access to live debugger sessions.

The MCP functionality is implemented in companion tooling, not as a native model-context-protocol server embedded in every `pounce-rs` solve.

Its main architectural benefit is that analysis tools can consume structured numerical data instead of requiring another application to reverse-engineer solver logs.

The project also provides model-preflight capabilities through its Pyomo integration, including identifying unset variables, invalid initial evaluations, and constraint violations before a full optimization run.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

Kitchin Research Group+1



## 15. Structured results, independent verification, and reproducibility

### 15.1 Versioned JSON solve reports

The `pounce-solve-report` crate defines the machine-readable report format:

`pounce.solve-report/v1`

The format can include:

| Component       | Contents                                                                   |
| --------------- | -------------------------------------------------------------------------- |
| `fair_metadata` | Result identity, creation time, solver version, platform, input provenance |
| `problem`       | Problem dimensions and model metadata                                      |
| `solution`      | Solver status, objective, primal/dual variables                            |
| `statistics`    | Iterations, numerical residuals, evaluation and timing information         |
| `iterations`    | Optional per-iteration optimization trajectory                             |
| `linear_solver` | Optional factorization diagnostics                                         |

A versioned schema is important because downstream systems can preserve a stable contract even while the Rust solver APIs evolve.

The documented compatibility policy permits adding fields within the schema major version, while breaking changes require a new major version.

Consumers should tolerate unknown fields and distinguish solver status from numerical quality indicators. The optional `linear_solver` diagnostics are backend-dependent rather than guaranteed to appear identically for FERAL and MA57.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Schema reference: `https://kitchingroup.cheme.cmu.edu/pounce/schema/solve-report-v1.html`

### 15.2 Independent solution verification

A less obvious capability is the separate verifier:

```
pounce verify problem.nl solution.sol \
    --feas-tol 1e-6 \
    --opt-tol 1e-6
```

It independently evaluates a claimed solution against the supplied model rather than trusting the optimizer's reported status.

The verifier checks primal variable bounds and constraint feasibility. If sufficient dual information exists, it can also evaluate first-order stationarity and complementarity residuals.

There is a critical default behavior: feasibility determines the verification verdict unless `--require-optimal` is supplied.

Even with the additional KKT check, it does not establish global optimality for a nonconvex problem.

The verifier can emit a JSON receipt containing hashes of the model and solution, enabling a consumer to establish which exact inputs were checked.

Optional HMAC signing supports a separate trust-boundary design, but the project's documentation explicitly treats the remote signing/service architecture as a proof of concept rather than a hardened production security system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/verify.html`

### 15.3 Observability and numerical traces

The Rust facade supports per-iteration capture through `.capture_iterations()`, `with_iter_capture`, and the lower-level collector infrastructure.

It also integrates with Rust's `tracing` ecosystem.

This makes it possible to capture objective progress, primal and dual infeasibility, barrier values, step lengths, evaluation counts, and related statistics without implementing a separate logging system around the optimizer.

The project's dedicated debugging facilities can go further, examining individual KKT matrices, inertia corrections, and factorization behavior.

These different interfaces serve distinct purposes: lightweight per-iteration monitoring for ordinary use, structured reports for auditing, and invasive numerical-state inspection when diagnosing a difficult solve.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 16. Important limitations, maturity, and solver selection boundaries

### 16.1 Mathematical and algorithmic limits

| Boundary                  | Practical implication                                                                 |
| ------------------------- | ------------------------------------------------------------------------------------- |
| General smooth NLP        | Finds local KKT solutions, not necessarily global minima                              |
| Nonconvex QP              | Active-set methods may find local solutions; global optimality is not established     |
| Mixed-integer programming | No native general MILP branch-and-bound solver                                        |
| General MINLP             | No native comprehensive spatial branch-and-bound engine                               |
| Polynomial SOS            | Relaxation size and order can become prohibitive                                      |
| Large sparse SDP          | Supported SDP functionality is not equivalent to a specialized large-scale SDP engine |
| Differentiation           | May be ill-defined around degenerate or nonunique optima                              |
| GPU optimization          | No documented native GPU sparse KKT backend                                           |
| External linear solvers   | FERAL and optional MA57, not the full IPOPT backend catalog                           |

The project documents some PSD chordal-clique processing in its conic debugger, but that does not by itself establish broad large-scale sparse-SDP performance. PSD problem suitability remains dependent on matrix dimensions, clique structure, memory use, and numerical behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+2



### 16.2 Relationship to neighboring solvers

POUNCE overlaps with several established optimization libraries, but the overlap differs substantially by problem class.

| Alternative | Main distinction                                                                                                                              |
| ----------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| IPOPT       | Mature general NLP solver; POUNCE ports its core algorithm into Rust and adds other solver families                                           |
| Clarabel    | Dedicated convex conic solver with a mature Rust integration surface; POUNCE includes conic functionality within a broader NLP-oriented suite |
| OSQP        | ADMM-based convex QP solver, often attractive for repeated QPs with different accuracy/performance tradeoffs                                  |
| HiGHS       | Dedicated LP, MILP, and QP algorithms; supplies discrete optimization that POUNCE does not                                                    |
| SCIP        | Extensive mixed-integer, branch-and-bound, and global search capabilities beyond POUNCE's local NLP and polynomial SOS scope                  |
| `discopt`   | Separate modeling/global-optimization project that can use POUNCE as its local nonlinear solver                                               |

POUNCE is therefore not a generic abstraction layer over every optimization backend. Its distinctive proposition is native Rust implementation of multiple continuous-optimization algorithms sharing numerical infrastructure.

That is different from a solver-agnostic modeling library or a multiplexer that delegates to third-party solvers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 16.3 Reported benchmark evidence

The upstream project maintains optimization benchmark suites that include standard NLP problems and comparisons with IPOPT.

For example, the 0.12.0 release reports strict optimal convergence on 1,284 of 1,326 instances for POUNCE versus 1,241 for an IPOPT-MA57 reference.

Those figures are upstream-reported, historical test-suite results. They are not an independently reproduced performance comparison for POUNCE 0.16.0, and they do not imply that POUNCE is faster or more reliable on every problem class.

The release documentation also notes reference-data and measurement qualifications. Meaningful comparisons need identical stopping tolerances, timing environments, linear solver settings, problem scaling, and success definitions.

Separately, improvements to FERAL's factorization kernels have produced measured end-to-end runtime improvements on POUNCE's internal benchmark corpora. Those benefits are credible for the measured KKT sequences, but they should not be extrapolated into universal speedup factors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 16.4 Version maturity and API stability

As of November 10, 2026, `pounce-rs` 0.16.0 is published and documented on docs.rs; the corresponding `pounce-solver` Python release was published November 7, 2026.

The official documentation has some examples still referring to `pounce-rs = "0.9"` and some pages describing older APIs. The current published facade documentation should take precedence when checking the exposed Rust feature flags and entry points.

The project is also pre-1.0 and evolving quickly. Its Rust facade reduces dependence on internal crate organization, but does not guarantee the kind of long-term API stability associated with a mature 1.x library.

This overview documents the published functionality rather than claiming the APIs have been compiled or benchmarked independently in this research session.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://pypi.org\&sz=32)

PyPI+2



## 17. Primary source reference map

The following are direct, standalone sources for deeper investigation:

| Subject                           | Primary source                                                          |
| --------------------------------- | ----------------------------------------------------------------------- |
| Repository and workspace          | `https://github.com/jkitchin/pounce`                                    |
| Current Rust facade               | `https://docs.rs/pounce-rs/latest/pounce_rs/`                           |
| Rust integration                  | `https://kitchingroup.cheme.cmu.edu/pounce/rust.html`                   |
| Solver architecture and selection | `https://kitchingroup.cheme.cmu.edu/pounce/choosing-a-solver.html`      |
| Options and backends              | `https://kitchingroup.cheme.cmu.edu/pounce/options.html`                |
| SQP and active-set warm starts    | `https://kitchingroup.cheme.cmu.edu/pounce/active-set-sqp.html`         |
| Convex optimization               | `https://kitchingroup.cheme.cmu.edu/pounce/convex-solver.html`          |
| Polynomial global optimization    | `https://kitchingroup.cheme.cmu.edu/pounce/global-optimization.html`    |
| Multiple-minimum search           | `https://kitchingroup.cheme.cmu.edu/pounce/find-minima.html`            |
| Nonlinear FBBT                    | `https://kitchingroup.cheme.cmu.edu/pounce/fbbt.html`                   |
| Sensitivity and retained KKT      | `https://kitchingroup.cheme.cmu.edu/pounce/sensitivity.html`            |
| Live debugger                     | `https://kitchingroup.cheme.cmu.edu/pounce/debugger.html`               |
| Independent verification          | `https://kitchingroup.cheme.cmu.edu/pounce/verify.html`                 |
| Structured results                | `https://kitchingroup.cheme.cmu.edu/pounce/schema/solve-report-v1.html` |
| FERAL numerical engine            | `https://kitchingroup.cheme.cmu.edu/feral/`                             |
| Installation and MA57             | `https://kitchingroup.cheme.cmu.edu/pounce/installation.html`           |
| Releases                          | `https://github.com/jkitchin/pounce/releases`                           |

## Overall architectural significance

POUNCE is especially interesting because it combines capabilities that are often implemented as separate systems: general nonlinear optimization, specialized convex programming, active-set methods, polynomial SOS relaxations, sparse numerical factorization, retained-state sensitivity, and detailed numerical diagnostics.

The strongest combinations are not simply the availability of multiple solver algorithms. They are the opportunities to use structural classification to select a more appropriate algorithm, reuse prior numerical work across related solves, differentiate converged optimization results through retained KKT factors, and independently verify outcomes.

Its defining constraints remain clear: CPU-based sparse direct linear algebra, local convergence for ordinary nonconvex NLPs, restricted polynomial global optimization, and no general-purpose mixed-integer or global spatial branch-and-bound engine.

For a Rust ecosystem that needs sophisticated continuous numerical optimization without depending on a large collection of native C/Fortran solver installations, POUNCE provides an unusually broad—and still rapidly evolving—set of capabilities.

# POUNCE (Rust) — Advanced Capabilities Addendum

Research date: November 10, 2026 | Reference release: POUNCE 0.16.0 (November 7, 2026)

This addendum extends the original POUNCE library overview with capabilities identified during the subsequent gap analysis. It concentrates on previously omitted functionality, significant implementation mechanisms, Rust extension points, numerical semantics, and integrations whose capabilities go beyond the core solver API.

Scope and version boundary: POUNCE is a multi-crate Rust workspace with several accompanying Python, Julia, C, and modeling-system interfaces. The distinction between functionality implemented in its Rust numerical engines and functionality publicly exposed by `pounce-rs` is important. Where a capability is documented only through a companion interface, that boundary is stated explicitly.

Primary version references:

- Published Rust facade: `https://docs.rs/pounce-rs/0.16.0/pounce_rs/`
- POUNCE 0.16.0 release: `https://github.com/jkitchin/pounce/releases/tag/v0.16.0`
- Upstream repository: `https://github.com/jkitchin/pounce`

## 1. Mathematical programming with complementarity constraints

Rust solver core · Declared-complementarity frontends · Significant enhancements in 0.16.0

POUNCE's complementarity solver is substantially more capable than the general \\(\ell_1\\) exact-penalty fallback described in the original reference.

### 1.1 Complementarity as a first-class mathematical structure

A complementarity relation is written

\\[ 0\leq a(x)\perp b(x)\geq0, \\]

equivalent to

\\[ a(x)\geq0,\quad b(x)\geq0,\quad a(x)b(x)=0. \\]

It expresses a disjunctive condition: either the first quantity is zero or the second is zero, possibly both.

Mathematical programs with complementarity constraints (MPCCs) appear in equilibrium calculations, contact mechanics, capacity markets, bilevel optimization, phase-change models, and optimization through piecewise-smooth functions.

The central difficulty is that directly imposing \\(a(x)b(x)=0\\) creates a degenerate constraint system. At a feasible complementary point, standard nonlinear-program constraint qualifications frequently fail, even when the underlying physical or economic model is well defined.

POUNCE's declared-complementarity route preserves knowledge of the paired relationship, enabling specialized continuation, lifting, stationarity assessment, and branch-polishing mechanisms that an ordinary product-equality NLP cannot automatically exploit.

### 1.2 Relaxation and continuation

A common MPCC regularization replaces exact complementarity with a relaxed product condition:

\\[ a(x)b(x)\leq\tau,\qquad \tau>0. \\]

The optimization procedure reduces \\(\tau\\) toward zero while solving progressively more tightly complementary problems.

POUNCE supports two conceptually distinct approaches:

- Staged continuation: Solve a succession of NLPs with successively smaller complementarity relaxation parameters, warm-starting each stage.
- Barrier-coupled continuation: Tighten the complementarity relaxation alongside the interior-point barrier parameter within the optimization process.

The option `complementarity_continuation=coupled` selects the latter, which became the default in 0.16.0.

The broader MPCC solver also exposes `complementarity_strategy`, permitting specialized handling rather than blindly treating the model as an arbitrary smooth NLP.

Coupled continuation can reduce the overhead of separately converging many relaxation stages. However, it does not eliminate the nonconvexity or singularity risks inherent in complementarity-constrained optimization.

According to the 0.16.0 release measurements, the coupled approach improved completion rates on the project's MacMPEC benchmark collection and reduced iterations on its optimal-power-flow and capacity-model tests. These are upstream-reported workload-specific measurements.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 1.3 Complementarity lifting and KKT sparsity

A less obvious enhancement is structural lifting of wide complementarity expressions.

Consider

\\[ 0\leq g(x)\perp h(x)\geq0, \\]

where \\(g(x)\\) or \\(h(x)\\) depends on many model variables.

Directly incorporating the paired expressions can produce wide Jacobian rows and unfavorable coupling in the KKT system.

POUNCE can introduce an auxiliary variable \\(w\\), enforce the relationship through an equality, and express complementarity using the narrower variable representation.

For a suitable one-sided pair:

\\[ g(x)-w=c,\qquad 0\leq w\perp h(x)\geq0. \\]

This adds a variable and an equality but can substantially reduce the width of nonlinear coupling.

The tradeoff is important: more variables can lead to less sparse-factorization work when the original nonlinear rows create expensive elimination fill.

The `complementarity_lift` option supports `auto`, `yes`, and `no`. The 0.16.0 release documents automatic lifting for qualifying rows with at least eight Jacobian entries when exact Hessians are available.

On an upstream 64–128–128–128–10 ReLU network example, lifting reduced the reported per-iteration time from approximately 115 ms to 21–30 ms. This improvement arises from the altered factorization structure, not from a change to the underlying mathematical optimum.

The release notes also clarify that Pyomo's complementarity transformation already introduces separate variables, so this automatic lifting optimization does not apply identically to those models.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 1.4 Branch polishing and complementarity fidelity

The usual barrier termination tolerance does not guarantee that a complementary pair's product is as small as the requested tolerance.

In degenerate cases, the two sides can approach zero at approximately \\(O(\sqrt{\mu})\\), leaving a materially nonzero complementarity product or an ambiguous active branch.

POUNCE 0.16.0 adds a polishing stage that identifies nearly biactive pairs and attempts a local re-solve with the smaller side fixed to zero.

The polished solution is accepted only when the resulting point is sufficiently nearby and improves the relevant solution-quality criteria.

This matters because an NLP solver can report successful first-order convergence while leaving enough complementarity error to change the interpretation of the modeled regime.

The published reports expose:

- `natural_residual`: the complementarity residual in the model's natural representation.
- `polish`: whether polishing was performed and its outcome.
- `polish_threshold`: the threshold governing relevant polishing decisions.

These fields are available in the structured solve report and Python complementarity diagnostics. POUNCE Studio also interprets them through MPCC-specific diagnostic findings.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 1.5 Strong stationarity and solution interpretation

MPCCs require more nuanced optimality terminology than an ordinary smooth NLP.

A useful distinction is between a point that merely satisfies the relaxed NLP's KKT conditions and one that meets the stronger first-order conditions associated with the MPCC's complementarity structure.

POUNCE's benchmark reporting includes S-stationarity assessment. This matters particularly at biactive pairs, where both members of the complementarity relationship are zero and more than one local branch may be consistent with the point.

S-stationarity is not a certificate of global optimality. It is a first-order necessary-condition concept under appropriate assumptions.

For example, a trained ReLU network can be represented exactly by

\\[ y=\max(0,z) \iff 0\leq y\perp y-z\geq0. \\]

POUNCE can then locally optimize through the network using declared complementarity.

The 0.16.0 release compares these local MPCC solutions against MILP-certified optima on selected networks. The existence of a separate MILP certificate is essential: it is what establishes global optimality for the benchmark, not the MPCC solver itself.

The broader structural implication is that some piecewise-linear and equilibrium relationships can be represented without a large collection of manually constructed binary variables. Whether this is preferable depends on the requirement for global guarantees, the geometry of the solution, and the difficulty of the resulting MPCC.

Rust integration boundary: This capability is implemented in POUNCE's solver infrastructure, but the release notes do not establish a direct complementarity-pair declaration method on the `pounce-rs::builder::Problem` trait. Native Rust callers should distinguish solver-core functionality from the exact public declaration route supported by their chosen frontend. The ordinary NLP builder must not be assumed to infer declared pairs from arbitrary product constraints.

Source: `https://github.com/jkitchin/pounce/releases/tag/v0.16.0`

## 2. Structured KKT systems and block-aware numerical execution

Native Rust API · Interior-point/FERAL execution path

The original overview explained sparse LDLᵀ factorization and ordering algorithms but omitted POUNCE's ability to accept model-specific decomposition information.

This is important because the mathematical structure of an optimization model can contain decomposition opportunities that are not evident from generic sparse-matrix ordering heuristics.

### 2.1 Externally supplied KKT ordering

`IpoptApplication::set_external_ordering` allows a caller to install its own sparse elimination permutation.

The supplied permutation uses new-to-old indexing: `perm[k]` specifies the original matrix index moved into position \\(k\\).

Critically, this is a permutation of the augmented KKT system, not merely the primal variables.

A valid permutation must account for the solver's internal variable, slack, and constraint-dual blocks.

POUNCE checks its dimensions and bijectivity during the factorization setup. An invalid permutation produces a solver error rather than silently modifying the model.

The configuration persists across solves until cleared with `clear_external_ordering`.

Such external orderings can represent block-triangular or tearing structures identified by a modeling system, graph analysis, or domain-specific decomposition method.

They affect factorization fill and execution cost; they do not alter the optimization problem itself.

### 2.2 Schur-complement elimination

`set_kkt_schur_block` permits explicit designation of part of the KKT matrix for Schur-complement treatment.

For a partitioned symmetric system,

\\[ K= \begin{bmatrix} A&B\\\ B^T&C \end{bmatrix}, \\]

eliminating the \\(A\\) block formally gives

\\[ S=C-B^TA^{-1}B. \\]

Instead of treating all variables as one undifferentiated sparse matrix, the algorithm can factor the independent block and solve through its smaller coupling system.

This can be attractive when a model consists of many large subsystems coupled through relatively few variables.

The benefit depends on the size and density of the Schur complement. A dense or disproportionately large coupling block can consume more memory and work than ordinary sparse factorization.

### 2.3 Explicit block-parallel KKT partitions

POUNCE provides two additional entry points:

| API                                           | Representation                                       |
| --------------------------------------------- | ---------------------------------------------------- |
| `set_block_structure(var_blocks, con_blocks)` | One block label per original variable and constraint |
| `set_kkt_block_structure(labels)`             | One block label per augmented KKT row/column         |

The first is especially significant because it uses model-space dimensions and permits the solver to map the labels into its internal KKT ordering.

The second provides lower-level control. Negative labels designate shared or border variables, and an empty label vector requests structural detection.

Conceptually, a block-bordered system may have the form

\\[ K= \begin{bmatrix} K_1&0&0&B_1\\\ 0&K_2&0&B_2\\\ 0&0&K_3&B_3\\\ B_1^T&B_2^T&B_3^T&C \end{bmatrix}. \\]

Each independent \\(K_i\\) can be processed separately, with the coupling retained in a smaller border system.

This structure arises naturally in separable estimation, scenario-based optimization, multistage models, coupled physical subsystems, and optimization with a small set of global coordination variables.

### 2.4 Validation and fallback behavior

The native APIs do not blindly trust user-supplied partitions.

POUNCE validates the specified structure against the assembled KKT matrix and can fall back to ordinary full-space factorization when the decomposition is inconsistent, malformed, or unsuitable.

These facilities are documented for the interior-point solver with FERAL and exact Hessians. They should not be generalized to the MA57 backend, active-set SQP, or other solver families without separate verification.

The block and ordering configurations are persistent, so changes to the model's dimensions or sparsity can invalidate previously supplied information.

For repeated structured solves, this creates a useful division of labor: the modeling layer can determine and retain the mathematical decomposition, while POUNCE handles KKT assembly, factorization, validation, and fallback.

Primary Rust API:

`https://docs.rs/pounce-rs/0.16.0/pounce_rs/prelude/struct.IpoptApplication.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 3. Post-convergence crossover and active-set certification

Native Rust · Opt-in NLP capability

POUNCE supports a distinct optimization phase after the interior-point algorithm has converged: active-set crossover.

This must be distinguished from the IPM-to-SQP handover described in the original reference.

### 3.1 Why convergence does not identify exact activity

An interior-point method keeps slacks strictly positive, approaching active bounds without ordinarily placing the iterate exactly on them.

For a strictly complementary solution, this is often sufficient to infer the active set.

But at a weakly active constraint, the multiplier also approaches zero. The primal distance to the constraint can then decay much more slowly than the overall KKT residual.

Consequently, an interior-point solution can pass its stopping tests without revealing definitively which constraints should be considered active.

This can matter when the result feeds into sensitivity analysis, statistical covariance estimation, working-set warm starts, or interpretation of marginal values.

### 3.2 Crossover algorithm

With `crossover=yes`, POUNCE performs the following additional work:

1. Start from the converged interior-point primal-dual iterate.
2. Estimate active constraints using primal-distance and multiplier tolerances.
3. Perform an equality-constrained-QP-equivalent step on that estimated set.
4. If necessary, continue through the full active-set SQP algorithm.
5. Validate the refined solution before accepting it.

The main controls are:

| Option                 | Default       |
| ---------------------- | ------------- |
| `crossover`            | `no`          |
| `crossover_max_iter`   | 30            |
| `crossover_mult_tol`   | \\(10^{-8}\\) |
| `crossover_primal_tol` | \\(10^{-6}\\) |

An important correctness property is that crossover works against the original declared variable and constraint bounds rather than the slightly relaxed bounds used inside the interior-point algorithm.

Otherwise, a point exactly on the user's declared bound could still appear interior to the solver's numerical relaxation.

The method can return a linearly independent active working set and multipliers consistent with the refined point.

### 3.3 Rust result inspection and working-set transfer

The public API exposes:

- `crossover_report()` — retrieves the most recent crossover result.
- `last_sqp_working_set()` — retrieves the identified active working set where available.
- `handover_report()` — independently reports IPM-to-SQP handover.

For `crossover_report()`, a missing report and a rejected crossover have different meanings.

`None` indicates crossover did not run. A present report with `accepted == false` means the phase ran but its candidate was not accepted.

A successfully identified working set can then become useful input for subsequent active-set SQP solves.

The distinction between handover and crossover: handover changes which algorithm is responsible for completing a solve; crossover polishes an already converged interior-point result to identify an exact active set.

There is also a separate `qp_crossover` mechanism for LP vertex purification in the convex solver. Its implementation has documented severe runtime regressions on some degenerate LP benchmarks, so it remains opt-in.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/crossover.html`

`https://docs.rs/pounce-rs/0.16.0/pounce_rs/prelude/struct.IpoptApplication.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+2



## 4. Native Rust extension points and solver lifecycle control

Public `IpoptApplication` APIs · Advanced integration

POUNCE exposes more internal control than merely implementing `Problem` or `TNLP` and adjusting solver options.

### 4.1 Replaceable linear-solver factories

`IpoptApplication::set_linear_backend_factory` accepts a factory producing implementations of `SparseSymLinearSolverInterface`.

Conceptually, this separates the optimization algorithm from the code that handles sparse symmetric factorization.

A custom backend must honor the numerical solver's contract, including the capabilities required by the selected algorithm:

- Correct sparse symmetric matrix interpretation.
- Symbolic analysis and numerical factorization.
- Appropriate treatment of indefinite pivots.
- Correct inertia information when requested.
- Linear solves with consistent ordering.
- Valid numerical failure and singularity reporting.

This interface can support experiments with alternative sparse factorization strategies or specialized numerical implementations.

However, the existence of this extension point does not establish automatic compatibility with GPU solvers or arbitrary sparse linear algebra libraries. An implementation must match POUNCE's expected KKT and numerical-factorization semantics.

The standard FERAL/MA57 configurations remain the established built-in paths.

### 4.2 Pluggable restoration phases

POUNCE also exposes:

- `set_restoration_factory`
- `set_restoration_factory_provider`

These inject implementations of the `RestorationPhase` interface.

A restoration factory creates an alternative feasibility-recovery component used when the main filter line search cannot continue productively.

The factory-provider distinction is consequential.

A normal `set_restoration_factory` supplies a factory for an optimization invocation. Some outer optimization procedures—including the exact-penalty-barrier machinery—may execute multiple internal NLP solves within one outer operation.

Those repeated inner solves need fresh restoration components. `set_restoration_factory_provider` supports that lifecycle, avoiding accidental reuse of an already-consumed factory.

The provider takes precedence when both mechanisms are configured.

This could support alternative feasibility-restoration methods without replacing the interior-point algorithm itself, but such changes need to preserve the outer algorithm's expectations about termination and infeasibility.

### 4.3 Convergence hooks and retained numerical state

`set_on_converged` registers a callback invoked after the interior-point method has converged but before the final solution is returned through `TNLP::finalize_solution`.

The callback can access the internal iterate, calculated quantities, NLP representation, and full-space KKT solver.

This is how POUNCE can construct post-optimal sensitivity machinery from the numerical state of a converged solve.

Two further methods are important:

- `take_on_converged`: temporarily removes an installed callback, allowing another component to install its own and subsequently restore the original.
- `set_on_converged_refreshes_hessian`: controls whether the Hessian is refreshed at the converged point before the callback executes.

The Hessian refresh is enabled by default.

Why this matters: convergence can be detected before the final iteration updates its Hessian. Without refreshing it, a sensitivity calculation could combine the final gradient and Jacobian with a Hessian evaluated at the preceding iterate.

The refresh corrects that inconsistency at the cost of an additional Hessian evaluation.

### 4.4 Retry semantics and authoritative results

POUNCE's second-opinion recovery mechanisms create a subtle lifecycle issue.

A convergence callback can fire during an optimization attempt that is later discarded in favor of an earlier, better result.

Thus, capturing the most recently observed converged state does not necessarily capture the result ultimately reported to the caller.

The application exposes `answer_restored_from_floor()` and `reported_on_converged_id()` to distinguish those cases.

The distinction matters particularly when retaining a KKT factorization for later differentiation. Unlike a final solution vector, a factorization cannot simply be reconstructed from a previously saved primal-dual point without further numerical work.

A robust retained-state consumer must associate its cached numerical state with the optimization attempt whose result was actually selected.

### 4.5 Presolve and diagnostic lifecycle hooks

Further native controls include:

| Method                           | Function                                                                                                                 |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `set_presolve_already_applied`   | Prevent automatic wrapping when an external caller has already applied a specialized presolve transformation             |
| `optimize_tnlp_without_presolve` | Run without introducing the generic presolve wrapper, preserving original coordinates for operations such as sensitivity |
| `set_diagnostics`                | Install shared diagnostic state for KKT instrumentation                                                                  |
| `set_debug_hook`                 | Attach a one-solve interactive debugging hook                                                                            |
| `enable_iter_history`            | Capture accepted iterations for numerical reporting                                                                      |
| `warm_start_diagnostics`         | Inspect acceptance and reconstruction of supplied warm-start data                                                        |
| `linear_solver_summary`          | Retrieve factorization statistics and failure summaries                                                                  |
| `timing_stats`                   | Access subsystem-level timing measurements                                                                               |

The distinction between temporary and persistent configuration is important. Some hooks are consumed by a solve, while others remain installed until explicitly cleared.

These APIs collectively make `IpoptApplication` an extensible numerical execution driver, rather than only a wrapper around an optimization function.

Source: `https://docs.rs/pounce-rs/0.16.0/pounce_rs/prelude/struct.IpoptApplication.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 5. Nonconvex QP certification and convex QP sensitivity

Native Rust · `pounce-qp`, `pounce-convex`, and `pounce-sens-core`

### 5.1 Second-order checks in indefinite active-set QP

The original overview explained that POUNCE's active-set QP method supports indefinite quadratic objectives.

But indefinite QPs introduce a distinction between a stationary point and a local minimum.

For a QP

\\[ \min_x\quad\frac12x^THx+c^Tx \\]

subject to linear constraints, a first-order KKT point is not necessarily minimizing when \\(H\\) is indefinite.

For a fixed active working set \\(\mathcal W\\), a second-order necessary condition for a local minimum involves the feasible tangent directions.

A simplified equality-constrained condition is

\\[ d^THd\geq0 \quad\text{for all }d\neq0 \text{ satisfying }A\_{\mathcal W}d=0. \\]

If POUNCE finds a feasible direction with negative curvature, it can attempt to move along that direction rather than certifying the stationary point as a minimum.

The option `sqp_qp_certify_second_order` controls this additional check.

The default differs by context:

- Standalone active-set QP solves enable second-order certification by default.
- QPs embedded within the nonlinear SQP driver disable it by default.

This difference is deliberate. An SQP subproblem is a local approximation used to generate progress on the nonlinear objective. Requiring every intermediate quadratic approximation to satisfy second-order optimality can be counterproductive.

A further optimization is that the expensive indefinite-Hessian check can be skipped when the Hessian is already known to be positive semidefinite.

Second-order checking improves the interpretation of an indefinite QP solution, but it does not produce a global nonconvex QP certificate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Source: `https://kitchingroup.cheme.cmu.edu/pounce/active-set-sqp.html`

### 5.2 Sparse Schur updates inside active-set iterations

The active-set QP solver can avoid complete KKT refactorization after each working-set change.

With `sqp_qp_use_schur_updates`, POUNCE maintains a fixed-dimension factorization and absorbs active-constraint changes through low-rank Sherman–Morrison–Woodbury updates.

The option `sqp_qp_max_schur_updates_before_refactor` limits the number of accumulated updates before rebuilding the factorization.

This is separate from the block-decomposition APIs discussed in Section 2.

The two mechanisms address different structure:

- Block decomposition exploits separability present in the mathematical model.
- Active-set Schur updates exploit small changes in which constraints are active.

The latter is particularly valuable in a sequence of QP subproblems when only a few constraints enter or leave the working set.

The current documentation describes automatic selection based on QP size, with safeguards for large, indefinite, or poorly conditioned cases. This mechanism is a performance optimization rather than a different mathematical objective.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 5.3 Convex `QpSensitivity`

The `pounce-convex` solver exposes a dedicated `QpSensitivity` API.

This is distinct from the sIPOPT-derived general NLP sensitivity implementation.

For a parametric convex QP, it uses the converged primal-dual state and linear-system information to evaluate how the solution changes under parameter perturbations.

Among its capabilities are:

- Classification of active variable bounds and constraints.
- Identification of weakly active or ambiguous constraints.
- Sensitivity calculations informed by the achieved complementarity.
- Treatment of active-set changes using shared bound-check machinery.
- Reduced-Hessian calculations using the convex QP's null-space geometry.

The activity classifier's effective barrier scale is derived from the achieved primal-dual complementarity, not necessarily the precise barrier parameter used in the final interior-point iteration.

Those two quantities may be close at convergence but are not identical, and cross-solver diagnostic comparisons should preserve the distinction.

The implementation shares important degeneracy-handling concepts with `pounce-sens-core`, providing a common foundation for reasoning about weakly active constraints.

### 5.4 Multiple-right-hand-side solving

The convex QP interface also supports `solve_qp_multi_rhs`, alongside independent batch solving and reusable `QpFactorization` handles.

There are two different computational opportunities here.

In a multiple-right-hand-side linear solve, a retained factorization \\(K=LDL^T\\) can be applied to many right-hand sides:

\\[ KX=B. \\]

This avoids repeated numerical factorization and is useful when several sensitivity directions or related linear systems share the same KKT matrix.

In a batch of separate QPs, each problem may have its own matrix and nonlinear iteration history, so full numerical-factor reuse is generally not guaranteed.

The two mechanisms should be distinguished when analyzing repeated-solve performance.

Sources:

`https://docs.rs/pounce-convex/0.16.0/pounce_convex/sensitivity/struct.QpSensitivity.html`

`https://kitchingroup.cheme.cmu.edu/pounce/choosing-a-solver.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 6. Initialization, warm-state construction, and batched nonlinear solves

Rust solver initializer · Python orchestration conveniences

### 6.1 Cold-start preprocessing changes the supplied iterate

The default interior-point initializer does not simply use an input `x0` verbatim.

For bounded variables, POUNCE pushes the initial iterate away from the bounds, initializes inequality slacks, estimates or initializes multipliers, and sets the initial barrier parameter.

The principal controls include:

| Option                                 | Purpose                                                                    |
| -------------------------------------- | -------------------------------------------------------------------------- |
| `bound_push`, `bound_frac`             | Move initial variables into the admissible interior                        |
| `slack_bound_push`, `slack_bound_frac` | Analogous push for inequality slacks                                       |
| `bound_mult_init_val`                  | Initialize bound multipliers                                               |
| `bound_mult_init_method`               | Select constant, barrier-based, or least-squares multiplier initialization |
| `constr_mult_init_max`                 | Cap initialized constraint multipliers                                     |
| `least_square_init_primal`             | Compute a least-squares point from the linearized constraints              |
| `mu_init`                              | Set the initial barrier parameter                                          |
| `start_with_resto`                     | Enter restoration at the beginning of optimization                         |

The default interior push is significant: a variable passed exactly on a lower bound can be moved inward before the first objective evaluation.

A starting point that is feasible and close to optimal therefore does not automatically provide a high-quality interior-point starting state.

For largely linear constraints, `least_square_init_primal=yes` can reduce initial infeasibility before the principal optimization iterations begin.

These are native solver features rather than Python-specific initialization routines.

### 6.2 A complete interior-point warm start

A genuinely warm interior-point solve requires more than supplying the previous primal solution.

POUNCE's documented warm-start contract includes:

1. Enabling warm-start initialization.
2. Supplying compatible primal and dual multipliers.
3. Seeding an appropriately small barrier parameter.
4. Configuring warm-start bound pushes to avoid excessively displacing the prior solution.

Relevant options include:

`warm_start_init_point`, `warm_start_bound_push`, `warm_start_bound_frac`, `warm_start_slack_bound_push`, `warm_start_mult_bound_push`, and `warm_start_mult_init_max`.

The Rust `IpoptApplication` API adds `warm_start_diagnostics()`, which reports whether multiplier blocks were accepted, reconstructed, or discarded and what barrier initialization was ultimately selected.

The Python `WarmStart` object packages the corresponding state and provides serialization through `.save()` and subsequent loading.

The Python interface can also transfer warm-state components across changed model layouts using a user-defined mapping. The transfer contract includes primal values, general and bound multipliers, the working set, and barrier state.

The significance is that repeated-solve reuse is a state-mapping problem, not merely copying an array.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 6.3 Batched NLP solving and expression-tape reuse

The Python `solve_nlp_batch` facility solves multiple nonlinear problems in one orchestrated batch.

Its most important distinction is the ability to reuse structural information for related models.

A parsed `NlProblem` can create variants with different initial points or bounds while sharing the expression representation, automatic-differentiation tapes, sparsity structure, and coloring information.

This avoids reconstructing much of the modeling and derivative infrastructure separately for every batch member.

The batch executor can release the Python GIL and distribute independent solves across a Rayon worker pool.

There is a resource tradeoff:

- Running several optimization problems concurrently can exploit parallelism across independent solves.
- Solving a small number of large problems sequentially can leave more CPU resources available to each problem's internal sparse factorization.

The Python API exposes `parallel` controls to distinguish these approaches.

It also supports a `warms` argument for chaining a batch of related instances to a later batch, transferring the previous primal-dual solution and barrier information.

A dimensionally incompatible warm seed falls back to cold initialization for the affected instance rather than assuming the state can safely be reused.

### 6.4 Initial-point generation and preflight analysis

The companion Python tooling provides `generate_starts`, `project_to_feasible`, and `race_starts` for preparing and comparing candidate initial points.

The CLI also exposes:

```
pounce check-x0 model.nl
pounce check-x0 model.nl --json
```

This evaluates the proposed starting state without running the full optimizer.

Its checks include invalid objective or constraint evaluations, variable-bound violations, initial constraint infeasibility, expected bound-push changes, and numerical derivative-scale problems.

A related `pounce.preflight` API performs analogous checks for Python callback models.

This is particularly useful when a model contains logarithms, divisions, square roots, or other domain-restricted nonlinear expressions. Moving a point within its declared variable bounds does not necessarily make every nonlinear expression evaluable.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/initialization.html`

`https://kitchingroup.cheme.cmu.edu/pounce/python.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



## 7. Differentiable optimization: the `DiffHandoff` contract and advanced JAX execution

Rust sensitivity infrastructure · Python JAX/PyTorch frontends

### 7.1 A common contract for differentiating solver outputs

POUNCE's `pounce-sensitivity` infrastructure defines `DiffHandoff`, a Rust-side representation of the information needed to differentiate an optimization result.

This addresses a subtle integration problem: the optimization solution can be correct while the derivative propagated through it is incorrect because the caller interprets active constraints or multipliers differently.

The contract includes:

| Field                          | Meaning                                                                         |
| ------------------------------ | ------------------------------------------------------------------------------- |
| `lambda`                       | Canonically represented general-constraint multipliers                          |
| `mult_x_lower`, `mult_x_upper` | Bound multipliers                                                               |
| `pinned_vars`                  | Boolean mask identifying variables treated as fixed under local differentiation |
| `active_constraints`           | Boolean mask identifying active constraint rows                                 |
| `active_tol`                   | Threshold used to classify activity                                             |

The Python `Problem.solve` result exposes the active-set information directly.

For example, the `pinned_vars` mask identifies variables that cannot move under a local parameter perturbation when their bounds are treated as strongly active.

This is particularly useful because the nonlinear and convex solver families use different names and signs for multiplier blocks.

The canonical handoff prevents downstream consumers from assuming that a field named `lam` has identical meaning in the NLP and QP implementations.

It also provides a mechanism for deliberately pinning variables in an optimization layer—for example, continuous variables fixed by a separate discrete optimization decision—without requiring every differentiation consumer to rediscover the active set.

The masks are locally valid numerical classifications. They do not eliminate ambiguity at weakly active constraints or guarantee differentiability when the optimal solution is nonunique.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 7.2 Reusing the converged KKT factor for differentiation

For a general parameterized KKT system

\\[ F(z,p)=0, \\]

implicit differentiation gives

\\[ F_z\\,dz=-F_p\\,dp. \\]

POUNCE's JAX integration can apply the retained factorization of the converged KKT matrix rather than reconstructing and factoring a separate dense derivative system.

`JaxProblem` exposes this through `factor_reuse=True`, the default.

The approach uses the solver's retained numerical state together with matrix-free applications of the KKT operator.

This is particularly useful when many derivative queries involve the same optimization result or when constructing a dense Jacobian/Hessian would be expensive.

An important numerical distinction is that the retained factorization corresponds to the converged barrier system. Its derivatives approximate the limiting active-set derivative, with error tied to the achieved barrier parameter and conditioning.

It should not automatically be interpreted as exact differentiation of a nonsmooth active-set transition.

### 7.3 Higher-order JAX differentiation

POUNCE's JAX frontend implements a more sophisticated differentiation rule for `JaxProblem.solve` than a reverse-mode-only callback.

The retained-factor path uses `jax.lax.custom_linear_solve` with a matrix-free KKT operator.

This permits differentiation through the implicit linear solve, including second-order and higher-order differentiation on supported problems.

However, the API surfaces are not equivalent:

| API                        | Differentiation behavior                                                  |
| -------------------------- | ------------------------------------------------------------------------- |
| `JaxProblem.solve`         | Supports forward/reverse differentiation and higher-order transformations |
| `solve_with_warm`          | Reverse-mode differentiation; forward-mode JVP not generally supported    |
| `vmap_solve`               | Batched differentiable solves                                             |
| `vmap_solve_parallel`      | Independent parallel host solves                                          |
| `batched_solve`            | One stacked NLP, reverse-mode differentiable                              |
| PyTorch optimization layer | Custom autograd backward through KKT sensitivity                          |

The actual numerical optimization and sparse factorization are still executed by the POUNCE host solver. JAX tracing or JIT compatibility does not turn the KKT factorization into a CUDA kernel.

### 7.4 Two distinct batching architectures

POUNCE supports two notably different ways to batch differentiable optimization.

Independent parallel solves: `vmap_solve_parallel` dispatches individual problems across CPU workers, with each worker retaining its own problem state.

Stacked block-diagonal solve: `batched_solve` constructs a single larger NLP whose objective is the sum of independent objectives and whose constraint Jacobian and Hessian are block diagonal.

For \\(B\\) independent problems,

\\[ \min\_{x^{(1)},\ldots,x^{(B)}} \sum\_{k=1}^B f_k(x^{(k)}) \\]

is solved as one structured system.

The latter can reduce repeated solver setup and may exploit block sparsity, but it couples the problems operationally through a shared optimizer invocation and termination process. Independent parallel solves permit different convergence histories and resource allocation.

This is a meaningful architectural choice, not simply two spellings for `vmap`.

### 7.5 Factor ownership and thread affinity

The Python documentation also identifies an important runtime boundary.

An `NlProblem` expression evaluator can be shared across threads, although its Python callbacks do not release the GIL and therefore do not execute concurrently.

By contrast, held numerical objects such as `Solver`, `QpFactorization`, and `QpSensitivity` are thread-affine in the documented Python interface.

Those objects must be used and released on the thread that created them.

This matters because expression-tape reuse, numerical-factor reuse, and parallel solve execution have different concurrency semantics. Combining them without respecting ownership can defeat the intended reuse or violate the binding's lifecycle requirements.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/differentiable-solves.html`

`https://kitchingroup.cheme.cmu.edu/pounce/python.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



## 8. Parametric continuation, solution-branch tracing, and inverse mapping

Native KKT sensitivity foundation · Python/JAX/PyTorch orchestration

The original overview covered local parameter sensitivities and active-set path following. POUNCE also provides higher-level machinery for following entire solution branches.

### 8.1 `Continuation`: orchestration of related NLPs

The Python `pounce.Continuation` interface coordinates a sequence of optimization problems as parameters change.

It can retain and transfer:

- Primal and dual states.
- Barrier initialization.
- Active working sets.
- Parametric sensitivity predictors.
- State mappings between problems with different shapes.
- Correction and rejection statistics.

The distinction between the two principal modes is fundamental.

| Mode                         | Behavior                                                                                   | Accuracy implication                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| `run(thetas)`                | Performs a corrected solve at each prescribed parameter point                              | Each requested point is solved to the specified numerical criteria      |
| `follow(theta_of_s, s_span)` | Predicts along the path, checks KKT residuals, and skips corrective solves when acceptable | Some accepted points are predictions rather than fully converged solves |

In `run`, failed corrections can cause automatic subdivision between prescribed parameter values. Inserted intermediate points are tracked separately from the points originally requested.

In `follow`, a user-supplied KKT monitor determines whether a predicted point is adequate.

If no suitable monitor is supplied, the implementation falls back to correction rather than accepting unverified predictions.

### 8.2 What the continuation predictor actually saves

The tangent predictor uses derivatives such as

\\[ \frac{dx^\*}{d\theta} \\]

computed from the converged KKT system.

A prediction for a small parameter step is

\\[ x^\*(\theta+\Delta\theta) \approx x^\*(\theta) + \frac{dx^\*}{d\theta}\Delta\theta. \\]

This is inexpensive relative to a full nonlinear solve when the factorization is already available.

However, the upstream warm-start benchmark reveals an important limitation.

For a prescribed sequence requiring full correction at every point, adding tangent prediction to an already warm-started interior-point solve changed total iteration counts by only approximately ±3% in the reported MPC experiments.

The explanation is that a well-warm-started interior-point method may already converge in one iteration. Improving its starting point does not remove the algorithm's initialization and barrier requirements.

The meaningful saving occurs when the monitor can accept a predicted point without a new optimization solve.

That is an explicit tradeoff between solution accuracy and computational effort, controlled through `monitor_tol`.

The Python and differentiable frontends share a step-size controller, but obtain their derivatives and residual monitors through different interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 8.3 Pseudo-arclength continuation through folds

Ordinary parameter continuation treats \\(\theta\\) as the independent coordinate.

This fails at a turning point where the local solution branch folds and \\(dx^\*/d\theta\\) becomes singular.

POUNCE addresses this with `trace_arclength`, exposed through its Python continuation and differentiable path-following interfaces.

Rather than fixing \\(\theta\\), the method solves the stationarity/constraint system

\\[ R(z,\theta)=0 \\]

together with an additional arclength constraint.

The augmented problem can continue along a branch even when \\(\theta\\) reverses direction.

The distinction matters for nonlinear equilibrium problems: a fold is not necessarily the end of the solution branch, but the end of its representation as a single-valued function of the chosen parameter.

The two implementations use different numerical approaches.

`Continuation.trace_arclength` assembles the augmented system from the problem's callbacks and uses sparse linear algebra. The JAX/PyTorch `PathFollower` formulation is organized around differentiable problem representations and associated dense linear algebra for the arclength correction.

A fold cannot generally be traversed using the same nonsingular KKT factorization employed in ordinary tangent prediction. The singularity is precisely why the augmented system is needed.

The arclength interfaces are limited to suitable equality-constrained or unconstrained problems, with scalar-parameter and fixed-active-set restrictions for the differentiable implementation. They do not constitute general bifurcation detection, branch switching, or Hopf analysis.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+2



### 8.4 Inverse mapping of optimization outputs

POUNCE's differentiable frontend also supplies `inverse_map_rhs`.

Suppose an optimization problem determines \\(x^\*(\theta)\\), and an observable is

\\[ y=h(x^\*(\theta),\theta). \\]

If the desired output trajectory \\(y(s)\\) is prescribed, one can solve for the required parameter trajectory:

\\[ \frac{d\theta}{ds} = \left(\frac{\partial y}{\partial\theta}\right)^{-1} \frac{dy}{ds}, \\]

where

\\[ \frac{\partial y}{\partial\theta} = \frac{\partial h}{\partial x} \frac{\partial x^\*}{\partial\theta} + \frac{\partial h}{\partial\theta}. \\]

POUNCE supplies the derivative of the optimized solution through its sensitivity infrastructure and combines it with derivatives of the observable.

The resulting right-hand side can be passed to an external ODE integrator, such as Diffrax under JAX or an ordinary Python integrator under PyTorch.

This replaces a repeated outer inverse optimization with a differential path calculation under suitable assumptions.

The documented interface currently requires a square output-to-parameter Jacobian. The relevant matrix must also remain sufficiently nonsingular along the path.

Distinction: this is not a general nonlinear inverse solver. It is a local differential inverse mapping around a regular solution branch.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/continuation.html`

`https://kitchingroup.cheme.cmu.edu/pounce/path-following.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



## 9. Differential equations, DAEs, and boundary-value optimization

Python numerical interfaces · Rust FERAL and optimization components

POUNCE's broader distribution contains a differential-equation toolkit that was entirely absent from the original reference.

These components use numerical infrastructure from the Rust workspace, but the documented high-level interfaces are Python APIs, not `pounce-rs` modules.

### 9.1 Stiff initial-value problems: `pounce.ode.solve_ivp`

The `solve_ivp` implementation provides three-stage Radau IIA collocation:

- Fifth-order implicit integration.
- L-stability.
- Adaptive time stepping.
- Sparse linear solves through FERAL's unsymmetric LU engine.
- Analytic or finite-difference Jacobians.
- Dense output and event handling for the supported ODE/mass-matrix route.
- Reuse of Jacobian/factorization information when possible.

It handles systems of the form

\\[ M\dot y=f(t,y). \\]

For nonsingular \\(M\\), this is an implicit ODE representation.

For an appropriately structured singular mass matrix, it can represent an index-1 DAE.

The solver can project inconsistent initial algebraic variables onto the constraint manifold before integration.

A subtle issue concerns dense output: even when algebraic constraints hold at accepted integration steps, interpolation between steps can introduce constraint residuals.

`project_output=True` can correct interpolated algebraic variables without changing the accepted integration trajectory.

The interface implements `method="Radau"` rather than the full range of SciPy integration methods.

### 9.2 Fully implicit DAEs: `pounce.ode.solve_dae`

The `solve_dae` interface accepts the more general residual form

\\[ F(t,y,\dot y)=0. \\]

This permits models whose equations are not conveniently expressible as a fixed mass matrix times the state derivative.

The solver uses Radau collocation and supports:

- Separate Jacobians with respect to \\(y\\) and \\(\dot y\\).
- Automatic identification of algebraic variables using the structure of \\(\partial F/\partial\dot y\\).
- Consistent-initial-condition projection.
- Optional projection of dense-output values onto nonlinear algebraic constraints.
- Adaptive integration with sparse LU factorization.

Its supported DAE scope is index 1. Higher-index systems require prior index reduction.

Event handling is not supported by this fully implicit interface.

The corresponding JAX and PyTorch `daeint` interfaces use fixed-mesh backward Euler or BDF2 collocation. Differentiation uses the implicit-function theorem applied to the discretized residual equations.

Thus, the adaptive forward simulator and the differentiable fixed-mesh integrator are distinct computational products even though they solve related mathematical problems.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



### 9.3 Ordinary boundary-value problems: `solve_bvp`

POUNCE also implements two-point boundary-value problems:

\\[ \begin{aligned} y'(t)&=f(t,y,p),\\\ b(y(t_0),y(t_f),p)&=0. \end{aligned} \\]

It discretizes the differential equations using fourth-order Lobatto IIIA/Hermite–Simpson collocation.

The resulting system can be solved through two paths:

| Method            | Numerical approach                                                               |
| ----------------- | -------------------------------------------------------------------------------- |
| `method="newton"` | Modified Newton method on the square collocation residual system                 |
| `method="ipm"`    | Equality-constrained feasibility NLP solved by POUNCE's interior-point algorithm |

The Newton path uses FERAL sparse LU and can reuse factorizations across Newton iterations.

The IPM path works through a larger saddle-point KKT formulation and is the foundation for constrained BVP optimization.

The ordinary NumPy BVP interface supports adaptive mesh refinement, boundary conditions containing unknown parameters, and continuous interpolation of the solved trajectory.

The JAX/PyTorch interfaces use a fixed mesh to preserve differentiability with respect to model parameters.

The documented `S` singular-term argument from SciPy's BVP interface is not implemented.

### 9.4 Constrained BVPs and discretized optimal control

`pounce.solve_bvp_constrained` extends the collocation formulation with state and parameter bounds, path inequalities, and an optional objective.

Its structure is

\\[ \begin{aligned} \min\_{Y,p}\quad &J(Y,p)\\\ \text{s.t.}\quad &R\_{\mathrm{collocation}}(Y,p)=0,\\\ &R\_{\mathrm{boundary}}(Y,p)=0,\\\ &y_L\leq Y\leq y_U,\\\ &c_L\leq c(t_i,Y_i,p)\leq c_U. \end{aligned} \\]

This becomes a genuine nonlinear optimization problem rather than just a root-finding problem.

An important modeling detail is that the documented path constraints are imposed at mesh nodes. They do not automatically establish continuous-time satisfaction between nodes.

The implementation assembles sparse block-diagonal Jacobians for node-local path constraints. The documented objective gradient uses finite differences, while limited-memory quasi-Newton curvature is used for the NLP.

For a fully determined BVP, adding arbitrary path constraints does not create additional degrees of freedom. To solve a meaningful constrained optimization problem, the formulation must contain free parameters, controls, boundary conditions, or other decision variables.

This makes the component useful for certain discretized optimal-control problems, although it is not a general-purpose optimal-control transcription framework with arbitrary dynamics, control discretizations, or multi-phase trajectory interfaces.

### 9.5 Differentiable BVP solves

The differentiable BVP APIs support sensitivities of the solved trajectory and unknown parameters with respect to model coefficients and boundary conditions.

For collocation residual \\(R(z,\theta)=0\\),

\\[ \frac{dz^\*}{d\theta} = -R_z^{-1}R\_\theta. \\]

The Newton-based differentiable path uses a sparse transposed LU solve for the backward pass.

The IPM path can provide higher-order differentiation when explicitly requested through `second_order=True`.

The latter entails additional numerical work because higher-order derivatives require further evaluation of the implicit solution rule.

This division between Newton and IPM backends is important: they expose related solutions but different derivative capabilities and factorization costs.

Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/ode.html`

`https://kitchingroup.cheme.cmu.edu/pounce/dae.html`

`https://kitchingroup.cheme.cmu.edu/pounce/bvp.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+3



## 10. Glass-box/black-box trust-region filter optimization

Python `pounce.trf_minimize` · General NLP solver underneath

POUNCE supplies a specialized optimization method for systems containing a mixture of explicit mathematical relationships and expensive externally evaluated subsystems.

The interface is `pounce.trf_minimize`, based on trust-region filter (TRF) methods for hybrid glass-box/black-box optimization.

### 10.1 Mathematical formulation

Consider

\\[ \begin{aligned} \min_x\quad &f(x)\\\ \text{s.t.}\quad &h(x)=0,\\\ &g(x)\leq0,\\\ &y=d(w), \end{aligned} \\]

where \\(f\\), \\(g\\), and \\(h\\) are known algebraic functions, while \\(d\\) is an externally evaluated program.

The black box may be a converged physical simulation, an external prediction service, or another computational subsystem whose internal equations are not available to the optimizer.

Directly substituting a fitted surrogate \\(\bar r(w)\\) for \\(d(w)\\) is unsafe because an approximate model can predict both incorrect feasibility and incorrect optimality.

POUNCE instead applies zero-order and first-order corrections at the current iterate:

\\[ r_k(w)=\bar r(w) +\bigl[d(w_k)-\bar r(w_k)\bigr] +\bigl[J_d(w_k)-J\_{\bar r}(w_k)\bigr]\(w-w_k). \\]

The resulting surrogate satisfies

\\[ r_k(w_k)=d(w_k),\qquad J\_{r_k}(w_k)=J_d(w_k). \\]

Thus, local value and derivative consistency hold even when the original surrogate is systematically biased.

A trust region limits the step to where this corrected approximation is locally credible. The algorithm evaluates the real black box at the trial point and uses a filter balancing objective value against the consistency residual

\\[ \theta=\\|y-d(w)\\|. \\]

This separates objective improvement from fidelity to the external subsystem instead of collapsing them into a single arbitrarily weighted penalty.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 10.2 Surrogate basis options

The documented `basis` mechanism supports several representations:

| Basis           | Mechanism                                                          | Evaluation tradeoff                      |
| --------------- | ------------------------------------------------------------------ | ---------------------------------------- |
| `"zero"`        | Local linearization using the truth-model Jacobian                 | No additional surrogate-fitting samples  |
| `"quadratic"`   | Locally fitted quadratic surrogate with ZOC/FOC correction         | Requires \\((n_w+1)(n_w+2)/2\\) samples  |
| Custom `Basis`  | Caller-defined `fit`, `predict`, `jacobian` implementation         | Depends on the surrogate                 |
| Prefitted basis | Retain existing surrogate coefficients and update only corrections | Avoids repeatedly fitting the same basis |

The quadratic basis becomes expensive as the number of black-box inputs grows. At ten inputs, a complete quadratic design requires 66 sampling points.

A particularly interesting mathematical simplification is that an affine surrogate provides no improvement over the zero basis when first-order correction is applied.

For

\\[ \bar r(w)=a+Bw, \\]

the basis-dependent part of the correction cancels:

\\[ \bar r(w)-\bar r(w_k)-B(w-w_k)=0. \\]

Hence a fitted affine surrogate adds sampling work but produces the same corrected local model as `"zero"`.

This is a useful example of an optimization capability replacing otherwise custom surrogate-management logic: the algorithm handles local correction and trust-region acceptance, while the supplied basis only needs to contribute curvature or other useful nonlinear structure.

### 10.3 Black-box derivatives and sampling cost

Both glass-box and truth-model Jacobians are optional.

If `truth_jac` is unavailable, POUNCE estimates it through finite differences, potentially adding one black-box call per input dimension per iteration.

The trust-region radius and finite-difference sampling radius are separate controls. A large trust region does not justify taking an equally large finite-difference step.

A supplied analytic truth-model Jacobian can therefore reduce the number of expensive external evaluations substantially.

The documented method still assumes a sufficiently smooth and reasonably deterministic black box. Finite differences do not solve the problem of a genuinely discontinuous, noisy, or nonreproducible external computation.

Convergence is local, and the trust-region/filter mechanism does not certify the global optimum of the full hybrid problem.

Rust boundary: `trf_minimize` is a Python-facing orchestration algorithm using POUNCE's NLP solver. The published `pounce-rs` facade does not document an equivalent high-level TRF interface.

Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/trf.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



## 11. Constrained statistical estimation, data influence, and streaming curve fitting

Python companion APIs · Retained Rust numerical factorization

POUNCE's `curve_fit` family is not simply a convenience wrapper around a least-squares objective. It combines constrained optimization, numerical sensitivity, robust statistics, and uncertainty propagation.

### 11.1 Constrained nonlinear regression

`pounce.curve_fit` accepts a SciPy-like model \\(f(t,p)\\), observed data, and an initial parameter vector.

The fitted parameter problem can be written

\\[ \min_p\\;\sum\_{i=1}^{m}\rho\left(\frac{y_i-f(t_i,p)}{\sigma_i}\right) \\]

subject to parameter bounds and general equality or inequality constraints.

It supports ordinary weighted least squares, along with smooth pseudo-Huber and Cauchy robust losses.

The `"huber"` and `"soft_l1"` options use the same twice-differentiable pseudo-Huber formulation, not the nonsmooth second derivative of the classical piecewise Huber loss.

For robust losses, covariance estimation incorporates a sandwich correction rather than assuming the ordinary unweighted least-squares information matrix remains valid.

The result exposes parameter estimates, standard errors, covariance, correlation, confidence intervals, error statistics, activity masks, and the underlying optimizer result.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 11.2 Active-constraint-aware covariance

An important extension beyond ordinary least-squares fitting is accounting for active parameter constraints.

For a locally active linearized constraint matrix \\(A\\), permissible parameter perturbations satisfy

\\[ A\\,dp=0. \\]

If \\(Z\\) spans the nullspace of \\(A\\), the relevant reduced curvature is

\\[ H_R=Z^THZ. \\]

A covariance approximation in the original parameter coordinates can then be expressed through the reduced space:

\\[ \Sigma_p\approx ZH_R^{-1}Z^T\\,s^2, \\]

with the appropriate Hessian normalization and statistical noise scaling.

Consequently, a parameter pinned by an active bound can have zero local sensitivity under a fixed-active-set interpretation.

The result's `active_mask` and `cov_source` fields help distinguish unconstrained from projected covariance calculations.

These are local asymptotic calculations. They should not be interpreted as conventional two-sided Gaussian uncertainty estimates when the true parameter distribution is truncated, strongly non-Gaussian, weakly identified, or influenced by changing active constraints.

### 11.3 Sensitivity of fitted parameters to observations

With `sensitivity=True`, `curve_fit` computes

\\[ \frac{\partial p^\*}{\partial y}, \\]

an \\(n\_{\mathrm{parameters}}\times n\_{\mathrm{observations}}\\) matrix.

This is a useful form of influence analysis.

Rather than refitting the complete model after perturbing each observation, POUNCE calculates sensitivity right-hand sides and applies the converged KKT factorization through batched linear backsolves.

For constrained fits, the derivatives are projected onto the nullspace of the active constraints.

The default calculation uses a Gauss–Newton approximation. It neglects the residual-curvature contribution to the exact objective Hessian,

\\[ \sum_i r_i\nabla^2 f_i, \\]

which can be significant for strongly nonlinear models or solutions forced away from an unconstrained optimum.

POUNCE therefore provides `sensitivity="exact"`.

The exact mode estimates the complete objective Hessian using central differences of the objective gradient. It requires approximately \\(2n\\) additional gradient evaluations and constructs a separate dense inverse rather than reusing the Gauss–Newton factorization.

This creates a meaningful distinction:

| Sensitivity mode | Advantage                                                         | Limitation                                                        |
| ---------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------- |
| Gauss–Newton     | Retained sparse-factor backsolves; efficient for many data points | Neglects residual-curvature effects                               |
| Exact Hessian    | More faithful local derivative of the fitted solution             | Additional derivative evaluations and dense \\(n\times n\\) solve |

The upstream documentation reports that the Gauss–Newton approximation's discrepancy relative to finite re-solves varies substantially with the problem and whether constraints are active. It should not be assigned a universal error percentage.

For robust losses, the sensitivity right-hand side must also incorporate derivatives of the robust weighting function. Correcting only the Hessian without correcting this right-hand side does not yield the proper influence.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 11.4 Confidence bands versus prediction bands

`CurveFitResult.confidence_band` distinguishes uncertainty in the estimated mean response from uncertainty in an individual future observation.

For an observable with parameter gradient \\(g\\),

\\[ \operatorname{Var}[\hat f(t)]\approx g^T\Sigma_p g. \\]

This is the confidence-band variance.

A prediction band adds the future observation's noise variance:

\\[ \operatorname{Var}[y\_{\mathrm{new}}]\approx g^T\Sigma_p g+\sigma\_{\mathrm{new}}^2. \\]

POUNCE uses Student-\\(t\\)-based intervals with the fitted degrees of freedom and permits explicitly supplied observation-noise values.

Thus, the same fitted covariance can answer two genuinely different statistical questions without requiring separate estimation systems.

### 11.5 Out-of-core estimation: `curve_fit_streaming`

`pounce.curve_fit_streaming` supports datasets too large to fit into memory.

The input is a factory returning a fresh iterator of data batches:

```
def data_source():
    for x_batch, y_batch in batches():
        yield x_batch, y_batch

result = pounce.curve_fit_streaming(
    model,
    data_source,
    p0=initial_parameters,
)
```

This is an illustrative use of the documented Python API.

The objective, gradient, and Gauss–Newton information are accumulated over batches, requiring memory proportional to the parameter dimension and batch size rather than the total observation count.

A key constraint is that the data source must be replayable. Every solver pass needs to revisit the full dataset.

The streaming implementation retains scalar fit diagnostics, covariance, confidence intervals, robust losses, weights, bounds, and parameter constraints.

It intentionally omits outputs whose size grows with the observation count:

- Full residual vectors.
- The complete \\(\partial p^\*/\partial y\\) influence matrix.

These fields are returned as `None`.

When computing prediction bands, the streaming path also cannot reconstruct the original pointwise observation-noise vector, so it uses a homoscedastic noise level unless explicitly supplied.

The companion `curve_fit_minima` combines fitting with the multiple-local-minima machinery to discover alternative parameter fits. Its results remain local candidates rather than globally certified solutions.

Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/curve-fitting.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



## 12. In-memory expression graphs, sparse automatic differentiation, and solver routing

Rust-backed expression engine · Python construction and evaluation APIs

### 12.1 In-memory nonlinear expression construction

POUNCE's `.nl` integration is more than a file parser.

Its Python frontend provides two additional routes into the same compiled expression and differentiation engine:

- `parse_nl_text(text, ...)` parses `.nl` content already held in memory.
- `build_nl_problem(...)` constructs an optimization problem directly from `NlExpr` expression nodes.

Both produce an `NlProblem`, with support for objective, constraints, gradients, sparse Jacobians, Hessians, Hessian-vector products, and model variants.

The latter path can avoid serializing a model to AMPL `.nl` simply to reconstruct it immediately into an internal expression graph.

`NlExpr` supports arithmetic and transcendental functions, as well as operations such as `atan2`, `erf`, `min`, `max`, comparison nodes, conditional selection, and logical composition.

Some of these expressions cannot be represented faithfully by common `.nl` writers. In-memory construction can therefore preserve functionality lost through a modeling-language serialization step.

### 12.2 Expression DAG sharing and nesting limits

An important performance feature is structural subexpression sharing.

`NlExpr` operations retain references to their operands instead of deep-copying the expression underneath them.

For example:

```
x = pounce.NlExpr.vars(2)
shared = x[0] * x[1]

objective = pounce.NlExpr.sum([
    shared,
    shared * shared,
    3 * shared,
])
```

The common subexpression is represented once and reused.

When the evaluation tape is constructed, this permits shared computation and adjoint accumulation rather than repeated traversal of equivalent expression trees.

The representation is therefore closer to a directed acyclic graph than a pure recursive syntax tree with independent copies of every term.

There is also a consequential implementation limit: expression nesting depth is capped at 10,000 levels.

A long chain of binary additions increases depth linearly with the number of additions, whereas `NlExpr.sum(terms)` creates a flat n-ary sum.

The documentation notes that graph traversal uses a worker thread with an enlarged stack, alongside explicit nesting-depth checks.

The relevant distinction is depth versus width: a model with a large number of terms need not have a deep evaluation graph.

Current limitations include the absence of pickling support and the inability to represent AMPL imported external functions through `build_nl_problem`. Models requiring those external functions must use the `.nl` parser route.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 12.3 Matrix-free Hessian-vector products

`NlProblem.hessian_vector_product` computes

\\[ \left( \sigma\nabla^2 f(x) +\sum_i\lambda_i\nabla^2g_i(x) \right)v \\]

without constructing the full Hessian.

The implementation uses forward-over-reverse automatic differentiation, seeding the direction vector into the compiled expression tape.

An explicit sparse Hessian is constructed differently: POUNCE uses structural sparsity information and coloring to recover Hessian entries from directional derivative evaluations.

A matrix-free Hessian-vector product can therefore be less expensive than assembling the complete Hessian when only a small number of directions is needed.

This is useful for iterative curvature analysis and matrix-free algorithms consuming \\(Hv\\), but does not imply that POUNCE's ordinary sparse-direct KKT solver has been replaced by a general Krylov optimizer.

### 12.4 Structural sparse differentiation in JAX and PyTorch

POUNCE's Python differentiation adapters also implement sparse derivative generation directly from JAX and PyTorch models.

The `sparse=True` option enables coloring-based differentiation.

For JAX, sparsity discovery first attempts to analyze the `jaxpr` program structure rather than only probing numerical derivative values.

This distinction is especially important for scientific models containing terms that underflow or have zero derivatives at selected probe points.

A mathematically nonzero structural derivative may evaluate to zero at a particular point. If numerical probing classifies it as structurally absent, subsequent compressed derivative recovery can become incorrect.

The implementation provides three ways to obtain the sparsity pattern:

| Route                                   | Behavior                                                                      | Important limitation                                               |
| --------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| Explicit `jac_pattern` / `hess_pattern` | Uses caller-provided structure directly                                       | Missing entries can corrupt compressed derivatives                 |
| JAX program analysis                    | Propagates structural index dependencies through supported `jaxpr` operations | Some control-flow and dynamic-indexing constructs require fallback |
| Numerical probing                       | Unions observed derivative nonzeros across sampled points                     | Value-dependent zeros can hide genuine structural entries          |

An explicitly supplied pattern must contain every structurally possible nonzero. Extra entries reduce efficiency but generally preserve correctness; missing entries can cause aliasing between colored derivative entries.

The documentation also distinguishes `custom_jvp` and `custom_vjp` functions. A custom JVP can expose derivative structure to the analyzer, whereas an opaque custom VJP may force dense dependency assumptions.

On a documented banded model with 2,000 variables, POUNCE reports an 18.4× Jacobian-evaluation speedup, a 5.4× Hessian-evaluation speedup, and a 7.6× whole-solve speedup from sparse coloring. These figures are specific to the benchmark's bounded-color sparsity and should not be generalized to dense problems.

For dense derivative structures, coloring may offer little benefit and introduces additional setup work.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/python.html`

### 12.5 Important correction: automatic solver routing differs by frontend

The original overview described structural LP/QP/QCQP routing too broadly.

The precise behavior depends on the entry point.

| Entry point                                     | Solver-selection behavior                                         |
| ----------------------------------------------- | ----------------------------------------------------------------- |
| CLI reading `.nl`                               | Structural classification and automatic selection where supported |
| CLI reading `.cbf`                              | Explicit conic model representation                               |
| Python `minimize`                               | Defaults to `solver_selection="nlp"`                              |
| Python `minimize(..., solver_selection="auto")` | Opts into probe-and-validate routing                              |
| Native Rust NLP facade                          | Does not implement all Python-level solver-selection routes       |

The Python `minimize` auto-classifier probes the objective and constraints, determines whether the problem appears linear or quadratic, tests convexity where applicable, and validates the reconstructed model against additional points.

If the tests fail, it uses the general NLP path.

There is an additional numerical nuance: a model inferred from opaque callbacks is not mathematically guaranteed to be identical to those callbacks.

The inferred coefficients can differ slightly because of finite-difference error.

POUNCE therefore evaluates the original objective at the returned solution, exposing that value as `res.fun`, while retaining the reconstructed solver-model objective separately as `res.info["obj_model"]`.

The solver-routing tolerance controls acceptable differences in the detection process; it does not prove exact algebraic equivalence.

The documentation explicitly notes that even a successfully routed convex QP can have a slightly displaced solution relative to optimizing the original callback representation directly.

This is a meaningful distinction between structural classification of an exact symbolic model and approximate classification through numerical evaluation.

Further, Python's `lp-ipm`, `qp-ipm`, and `socp` selectors are intercepted by its frontend. The Rust NLP application does not accept those same selectors as a general internal dispatch interface.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE+1



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/choosing-a-solver.html`

## 13. Derivative verification, solver-result fidelity, and numerical-quality auditing

Native Rust diagnostics · Python and JSON reporting

### 13.1 Structured derivative checking

POUNCE provides a derivative-checking facility for comparing analytic or automatic derivatives against numerical approximations.

The Rust application exposes derivative-test methods, while the Python result includes a structured `derivative_check` field when derivative testing is enabled.

`DerivativeCheckSummary` includes:

| Diagnostic               | Meaning                                                                 |
| ------------------------ | ----------------------------------------------------------------------- |
| `checked`, `suspicious`  | Number of derivative entries inspected and flagged                      |
| `missing_structure`      | Entries found numerically but absent from a supplied sparsity structure |
| `max_rel_error_gradient` | Largest relative gradient disagreement                                  |
| `max_rel_error_jacobian` | Largest relative Jacobian disagreement                                  |
| `max_rel_error_hessian`  | Largest relative Hessian disagreement                                   |
| `flagged`                | Indexed analytic/numerical comparisons, capped at 200 entries           |
| `clean`                  | Whether the checker found discrepancies above its configured thresholds |

This is important because a derivative mismatch can produce plausible-looking optimizer progress before causing eventual failure or convergence to the wrong stationary conditions.

An omitted sparse Jacobian or Hessian entry is particularly dangerous: the solver may never request that derivative value, so ordinary numerical logging will not reveal that the supplied model structure is incomplete.

The checker is primarily diagnostic. Its findings do not automatically imply that every solve should terminate or that the numerical approximation is authoritative near discontinuities and ill-conditioned expressions.

The same report appears in the versioned JSON solve report, enabling automated checks without parsing console output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 13.2 Scaled and unscaled KKT residuals

POUNCE distinguishes internal residuals from their equivalents in the user's original model units.

Examples include:

| Internal metric     | Original-unit counterpart    |
| ------------------- | ---------------------------- |
| `final_kkt_error`   | `final_unscaled_kkt_error`   |
| `final_dual_inf`    | `final_unscaled_dual_inf`    |
| `final_constr_viol` | `final_unscaled_constr_viol` |
| `final_compl`       | `final_unscaled_compl`       |

This matters because scaling can make an internally acceptable stationarity residual appear much smaller than its physical-unit equivalent.

The intermediate iteration callback's `inf_pr` has yet another relevant meaning: it reports the solver's internal scaled slack-form primal residual, which can differ from the original constraint violation reported in the solve summary.

The distinction is not cosmetic. A solution intended for sensitivity analysis, marginal-value interpretation, or certificate verification should be assessed in the correct coordinate system.

### 13.3 Objective scaling and result-quality warnings

POUNCE records the objective scaling applied during a solve.

The diagnostic object includes the gradient-based scaling factor, total objective scaling, initial gradient magnitude, and information about convergence certificates rejected because scaling obscured a problematic residual.

The reported `warnings` field can identify conditions such as:

- Extremely small objective scaling.
- Excessive unscaled stationarity residuals.
- Large dual scaling.
- Unscaled dual infeasibility exceeding the acceptable threshold.

The Python interface also documents an automatic solve-quality audit that can re-solve from the current answer when extreme objective-gradient scaling is detected.

These diagnostics should not be collapsed into a single success flag.

A successfully terminated optimization run can still warrant scrutiny if the unscaled residuals are insufficient for its intended downstream use.

Similarly, POUNCE distinguishes an ordinary callback requesting termination from a callback that raises an exception.

The Python interface maps raised callback exceptions to `Callback_Error` (`-198`), preserving error text, while an intentional `return False` produces `User_Requested_Stop`.

Both can produce an iterate, but their meaning is different.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 13.4 Bound relaxation and reported feasibility

Interior-point solvers may temporarily relax variable bounds to improve numerical behavior.

POUNCE has controls including `bound_relax_factor` and `honor_original_bounds`.

With original-bound honoring enabled, the final point can be projected back onto the caller's original variable bounds.

However, that projection is not a new optimization solve. It can alter the objective, stationarity, or nonlinear constraint values slightly.

A correct result consumer should distinguish:

- Internal feasibility relative to relaxed solver bounds.
- Feasibility relative to the original declared bounds.
- Residuals measured before final projection.
- Values of the actual reported solution after projection.

The distinction became particularly relevant in 0.16.0, which includes a fix for incorrect infeasibility verdicts associated with the treatment of internal slack variables under limited-memory Hessian approximation.

In that case, a feasible original constraint could be misclassified because an internal slack residual was used in the wrong comparison.

The broader lesson is that the original model's constraint semantics and the optimizer's internal slack representation are not always interchangeable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Sources:

`https://kitchingroup.cheme.cmu.edu/pounce/options.html`

`https://kitchingroup.cheme.cmu.edu/pounce/python.html`

## 14. Advanced conic sensitivity and face-aware differentiation

Native `pounce-convex` 0.16.0 · Shared `pounce-sens-core` infrastructure

The convex sensitivity implementation also supports conic solution geometry, an important distinction missing from the original overview and the earlier gap analysis.

### 14.1 Conic sensitivity requires faces, not independent active rows

For an ordinary linear inequality, activity can be classified by inspecting the inequality slack and multiplier.

A second-order, exponential, power, or semidefinite cone does not generally have this row-by-row interpretation.

Conic complementarity is expressed through the pairing of entire slack and dual blocks:

\\[ \langle s,z\rangle=0,\qquad s\in K,\quad z\in K^\*. \\]

Individual components of a conic dual vector can be negative even when the complete dual vector belongs to the correct dual cone.

Consequently, independently classifying each component as if it were a nonnegative-orthant constraint can produce plausible but incorrect activity labels.

POUNCE's `QpSensitivity::build_conic` accepts the same `ConeSpec` partition used for the original conic solve and constructs the appropriate face geometry for each cone block.

This covers the cone families available through `ConeSpec`, including nonnegative, second-order, exponential, power, and semidefinite cones.

### 14.2 Face geometry and curvature

For a second-order-cone block, the solution can lie in the cone's interior, on its curved boundary, or at its apex.

These regimes have different derivative behavior.

- An interior block does not impose a local active constraint on the sensitivity system.
- A boundary block contributes a tangent constraint and curvature.
- An apex block can produce singular or nonunique local behavior and must be treated separately.

For curved cone faces, the reduced Lagrangian Hessian must include their curvature contribution:

\\[ H_R=Z^T(P+C)Z, \\]

where \\(C\\) captures the curvature associated with the active cone faces.

Using \\(Z^TPZ\\) alone would incorrectly treat curved constraints as linear.

POUNCE constructs the active-face rows and curvature terms as part of the sensitivity system.

The `cone_block_kinds()` API exposes information about the classified blocks.

For mixed conic models, `activity()` remains meaningful for ordinary orthant rows and variable bounds, but its rowwise classifications must not be interpreted as the activity of curved cone components.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 14.3 Robustness and provenance of sensitivity steps

POUNCE exposes several checks relevant to sensitivity reliability:

| API                          | Meaning                                                         |
| ---------------------------- | --------------------------------------------------------------- |
| `kkt_cond_estimate()`        | Estimated condition number of the active-set sensitivity system |
| `last_step_residual()`       | Achieved relative residual of the sensitivity backsolve         |
| `ill_conditioned()`          | Consolidated warning about numerical unreliability              |
| `weakly_active_ineq()`       | Weak activity in applicable orthant constraints                 |
| `weakly_active_bound_vars()` | Weakly active variable bounds                                   |
| `cone_block_kinds()`         | Conic face classifications                                      |
| `path_segment_target()`      | Decodes events from active-set path following                   |
| `refined_row_target()`       | Decodes bound/constraint indices in refinement results          |

One subtle failure mode is that the regularized sensitivity KKT matrix can be numerically solvable while the returned step poorly satisfies the unregularized sensitivity equations.

A condition-number estimate alone may miss this; POUNCE therefore also monitors the actual residual of the refined sensitivity solve.

Another issue is index-space provenance. Active-face rows for conic blocks may be linear combinations of original constraint rows rather than a single original row.

`active_ineq()` can identify the originating block without providing an original constraint row that can safely be indexed and evaluated directly.

The decoding APIs exist precisely to prevent confusing primal indices, multiplier indices, observer coordinates, and cone-block provenance.

The `reduced_hessian()` operation constructs a dense null-space basis and is consequently much more expensive than the sparse parametric backsolve. It is appropriate for moderate-sized reduced-Hessian analysis rather than as a replacement for sparse sensitivities on large systems.

### 14.4 Scope of conic sensitivities

These derivatives are locally meaningful under the mathematical regularity conditions of the active conic face.

A first-order derivative through a curved cone boundary is not an exact finite-parameter continuation path.

At a degenerate cone apex, a sensitivity may be nonunique or unsupported by the available local linearization. The API can reject a sensitivity construction or mark its numerical results unreliable.

The distinction is particularly important for semidefinite constraints near rank transitions, where the active face changes as eigenvalues approach zero.

Primary API: `https://docs.rs/pounce-convex/0.16.0/pounce_convex/sensitivity/struct.QpSensitivity.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 15. Additional modeling-system, C ABI, and deployment capabilities

### 15.1 Native Julia and JuMP integration

Companion package: `Pounce.jl` 0.1 · C ABI integration

POUNCE has a dedicated Julia package rather than relying exclusively on IPOPT's ABI compatibility.

`Pounce.jl` exposes a `MathOptInterface` optimizer usable through JuMP:

```
using JuMP, Pounce

model = Model(Pounce.Optimizer)

@variable(model, x >= 0, start = 1.0)
@variable(model, y >= 0, start = 1.0)
@constraint(model, x + y >= 2)
@objective(model, Min, x^2 + y^2)

optimize!(model)
```

It supports continuous affine, quadratic, and nonlinear models, sparse derivative callbacks, primal and dual initial points, and native POUNCE solver options.

The wrapper also handles the sign conventions required to translate multipliers between POUNCE and JuMP/MathOptInterface, including maximization problems.

This is an important interoperability feature because merely exposing an IPOPT-compatible C library does not automatically provide a complete, correctly typed Julia optimizer implementation.

The package includes diagnostic attributes such as:

| Julia attribute                              | Purpose                                            |
| -------------------------------------------- | -------------------------------------------------- |
| `Pounce.NativeResiduals()`                   | Native convergence residuals                       |
| `Pounce.RestorationStatistics()`             | Restoration-phase statistics                       |
| `Pounce.LinearSolverStatistics()`            | Sparse-factorization diagnostics                   |
| `Pounce.FiniteDifferenceHessianStatistics()` | Numerical Hessian information                      |
| `Pounce.RecordIterationHistory()`            | Enable iteration capture                           |
| `Pounce.SolveReport("full")`                 | Retrieve the structured POUNCE solve report        |
| `Pounce.NativeSolveTimeSec()`                | Solver execution time excluding some wrapper setup |

The interface distinguishes total optimizer-call timing from native solver timing. This is relevant when benchmarking repeated solves because Julia compilation, model construction, callback overhead, and numerical factorization contribute different costs.

Packaging boundary: `Pounce.jl` is installed from the source checkout and requires a locally built `pounce-cinterface` shared library. Version 0.1 accepts the native 0.15.x and 0.16.x libraries after checking their required C symbols. The documentation does not describe a registered Julia package with bundled JLL binaries.

The JuMP route currently exposes continuous NLP functionality through the C API. It does not automatically expose the entire CLI's conic, polynomial SOS, or discrete optimization capabilities.

Source: `https://kitchingroup.cheme.cmu.edu/pounce/julia.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 15.2 Extended C ABI and persistent SQP working sets

POUNCE's C interface extends the conventional IPOPT-compatible solve interface with explicit active-set state transfer.

The published additions include:

- `IpoptGetWorkingSet`
- `IpoptSetWarmStartWorkingSet`
- `IpoptClearWarmStartWorkingSet`
- `IpoptSolveWarmStart`

These allow a C caller to retain the identities of active variable bounds and constraints between optimization invocations.

The working-set status representation distinguishes inactive, lower-active, upper-active, and fixed/equality conditions.

The last distinction is important for constraints that are inherently equalities: their treatment is not interchangeable with an inequality that happens to be active.

The warm-start state is algorithm-specific.

An interior-point solve does not consume the active-set SQP warm-start payload. Conversely, the SQP path requires its own active-set state to obtain the full benefit of working-set reuse.

A solver application may switch between algorithms across solves, but the caller must transfer the state that is meaningful to the newly selected method.

The C ABI additions are additive: conventional IPOPT-compatible callers do not need to adopt these functions to continue using the basic NLP interface.

Source: `https://kitchingroup.cheme.cmu.edu/pounce/active-set-sqp.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 15.3 GAMS working-set persistence

POUNCE's native GAMS integration can reconstruct an SQP working set from variable and equation marginals across successive solve statements.

This is a different reuse mechanism from an in-process Rust session because a GAMS solver link can be invoked afresh for each solve.

Two approaches are documented:

Marginal reconstruction: POUNCE inspects the GAMS variable and equation marginals and infers which constraints are active.

Explicit state file: The `sqp_state_file` option preserves the working set in a separate binary representation.

The state file is keyed to the model's dimensions and bound structure, allowing the link to detect incompatible changes.

Marginal reconstruction is convenient but lossy near degenerate solutions, where a constraint can be active with a zero multiplier. The explicit state file preserves the working-set classification more faithfully.

The relevant option is:

```
algorithm       active-set-sqp
sqp_state_file  .model.pou-ws
```

This capability is specific to the native GAMS solver link and is not automatically available in every pip-installed or file-based integration.

Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/active-set-sqp.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 15.4 Conic Benchmark Format input

In addition to AMPL `.nl`, POUNCE's CLI reads Conic Benchmark Format (`.cbf`) files.

```
pounce model.cbf
```

This offers direct access to the conic solver through an established optimization interchange format.

A `.cbf` file expresses conic structure explicitly, avoiding the need to infer a conic representation from opaque nonlinear callbacks.

This is particularly relevant for second-order-cone, exponential-cone, power-cone, and supported semidefinite models.

The project reports testing its reader and solver against CBLIB instances.

The direct `.cbf` route is a CLI/model-protocol capability; it is not an API for automatically translating arbitrary Rust functions into conic form.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 15.5 AMPL imported functions

POUNCE's `.nl` parser supports AMPL external-function declarations, enabling compatible models to call external numerical functionality.

This is especially significant for modeling systems whose nonlinear expressions contain property functions or other native callbacks.

It also creates a deployment distinction:

- The native executable can use compatible external-function integrations.
- The in-memory Python `NlExpr` builder does not provide the same imported-function declaration facility.
- The browser WebAssembly build cannot dynamically load the native external-function libraries.

A model may therefore be valid for the native executable yet unavailable to the browser version without reformulating or replacing its external functions.

The separation between an expression tape, a compiled callback, and an externally loaded library is material to portability and numerical differentiation.

Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/python.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 15.6 WebAssembly execution details

The original overview mentioned browser execution, but the actual deployment boundary is worth specifying.

The `pounce-wasm` component provides a compact JSON-based interface for:

- Allocating input memory.
- Loading an optimization model.
- Solving it.
- Retrieving a solution.
- Releasing resources.

The browser demonstration can generate AMPL `.sol` output or a CSV representation.

The documentation identifies three significant runtime restrictions:

| Restriction                        | Consequence                                                                  |
| ---------------------------------- | ---------------------------------------------------------------------------- |
| Single-threaded execution          | Rayon parallel paths run serially                                            |
| No dynamic AMPL external functions | Models requiring native external-function libraries cannot execute unchanged |
| No HSL MA57                        | Uses the pure-Rust FERAL backend                                             |

Thus, the browser version preserves the core optimization machinery but changes the execution environment and available integration mechanisms.

Source: `https://kitchingroup.cheme.cmu.edu/pounce/wasm.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



## 16. Further numerical execution distinctions

A few additional mechanisms are worth recording because they change solver behavior or performance in specific circumstances.

### 16.1 Active-set SQP time budgets

POUNCE's SQP driver propagates wall-clock and CPU-time limits into its nested QP subproblems.

The budget is checked across outer SQP iterations and the inner QP work rather than restarting a separate unrestricted timer for each subproblem.

This matters because an algorithmic time limit is ineffective if one inner optimization problem can consume substantially more time than the outer solver's budget.

The documented time-limit options are `max_wall_time` and `max_cpu_time`.

The checks occur at defined solver checkpoints, so an already-running numerical factorization may still complete before the deadline is observed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 16.2 Cold-QP homotopy versus warm parametric continuation

The active-set QP engine also has an optional cold-start homotopy mechanism, `sqp_qp_use_homotopy`.

This differs from following a previously solved parametric QP.

The cold-start method begins with a simpler box-only relaxation and progressively introduces the general linear constraints.

The working set changes as constraints become binding or active multipliers reach zero.

The approach can avoid a conventional phase-I feasibility process on some difficult QPs. It is not enabled by default, and its benefit depends on the structure of the constraint system.

This should be distinguished from `ActiveSetSession` continuation, where the preceding solution already provides a meaningful working set.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 16.3 FERAL's symbolic reuse under changing active sets

POUNCE's active-set implementation has an additional factorization-reuse path when low-rank Schur updates are unsuitable.

For sufficiently large QPs, the solver can retain dropped constraint rows as structural placeholders in the KKT pattern.

The numerical values change, but the sparse matrix structure remains recognizable to the symbolic analyzer.

This allows repeated numerical refactorizations without repeatedly performing full symbolic analysis.

It illustrates three distinct reuse levels:

| Level     | What remains reusable                                                                      |
| --------- | ------------------------------------------------------------------------------------------ |
| Symbolic  | Sparsity graph, ordering, and factorization structure                                      |
| Numerical | Factorization of unchanged coefficients                                                    |
| Low-rank  | Updates to a retained factorization when mathematical changes have suitable rank structure |

These mechanisms have different validity conditions and costs.

The active-set implementation automatically selects among some of these paths based on problem size, warm-start conditions, and whether the Hessian is known to be positive semidefinite.

Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/active-set-sqp.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



### 16.4 Retained differentiation factors have resource costs

The JAX `factor_reuse=True` path is not universally faster than assembling small dense differentiation systems.

For a single large sparse NLP with many sensitivity queries, retained sparse-factor backsolves can provide substantial advantages.

However, for a large batch of very small independent optimization layers, POUNCE documents cases where batched dense JAX backsolves outperform applying a single large retained factor repeatedly.

The difference arises because the full stacked sparse factor can be much larger than each individual per-sample KKT block, and reverse-mode differentiation may require many right-hand sides.

The JAX frontend therefore allows `factor_reuse=False` even when the forward optimization has already been solved.

There is also a lifecycle cost: retained FERAL solver objects can hold Rayon thread pools as well as numerical factors. Accumulating many distinct retained solves can consume operating-system threads, not just memory.

The `JaxProblem` interface exposes `clear_solver_cache()` and manages its factor registry through a bounded cache.

This is a consequential distinction between caching a solution vector, caching derivative tapes, and retaining a live sparse factorization with associated runtime resources.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://kitchingroup.cheme.cmu.edu\&sz=32)

POUNCE



Documentation: `https://kitchingroup.cheme.cmu.edu/pounce/differentiable-solves.html`

## 17. Capability boundaries and addendum synthesis

The expanded set of capabilities falls into three distinct layers.

| Layer                                           | Representative capabilities                                                                                                                             | Public integration boundary                                                 |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Native Rust numerical functionality             | MPCC algorithms, structured KKT decomposition, crossover, pluggable solver factories, QP second-order checks, conic sensitivity, derivative diagnostics | Rust workspace crates and selected `pounce-rs` exports                      |
| Rust-backed higher-level optimization workflows | Trust-region black-box optimization, constrained curve fitting, out-of-core estimation, continuation, differentiable optimization, BVP/ODE/DAE solution | Primarily Python and differentiable-framework interfaces                    |
| Modeling and deployment integrations            | JuMP/MOI, GAMS SQP working sets, AMPL external functions, `.cbf`, C ABI, browser execution                                                              | Companion packages, native solver links, and specialized executable targets |

This layering is important to preserve. A capability implemented somewhere in the Rust workspace does not imply a stable, documented equivalent on every Rust facade or language binding.

The most consequential additions to the original reference are the mechanisms that expose more of the mathematical structure surrounding an optimization problem:

- Complementarity pairs enable specialized degeneracy handling rather than arbitrary product equalities.
- Explicit block decompositions let numerical factorization exploit separability rather than relying only on generic sparse ordering.
- Crossover and second-order checks improve the interpretation of converged points.
- Retained KKT factors support differentiation, continuation, and constrained statistical analysis.
- Expression graphs and sparsity-aware automatic differentiation reduce derivative-generation overhead.
- Trust-region filtering, constrained collocation, and model-sensitive estimation extend optimization into broader numerical workflows.

These mechanisms also have important conditions: local versus certified-global convergence, valid active-set assumptions, sparsity correctness, numerical regularity, state compatibility, and sometimes substantial factorization or memory overhead.

Taken together with the original reference, this addendum establishes a broader picture of POUNCE as an extensible, structure-aware numerical optimization platform built around native Rust solver infrastructure, with higher-level companion interfaces for hybrid simulation, differentiation, continuation, and statistical estimation.