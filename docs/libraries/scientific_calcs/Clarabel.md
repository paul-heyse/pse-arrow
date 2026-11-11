# Clarabel.rs — Comprehensive Rust Library Reference

Research date: November 10, 2026 | Latest verified stable release: v0.11.1 | License: Apache 2.0 | Language: Rust

Official resources

- Repository: [https://github.com/oxfordcontrol/Clarabel.rs](https://github.com/oxfordcontrol/Clarabel.rs?utm_source=chatgpt.com)
- Rust API documentation: [https://docs.rs/clarabel/latest/clarabel/](https://docs.rs/clarabel/latest/clarabel/?utm_source=chatgpt.com)
- Official user guide: [https://clarabel.org/stable/](https://clarabel.org/stable/?utm_source=chatgpt.com)
- Crate: [https://crates.io/crates/clarabel](https://crates.io/crates/clarabel?utm_source=chatgpt.com)

## 1. Executive overview

Clarabel is a general-purpose numerical solver for continuous convex optimization, implemented natively in Rust. It uses a primal-dual interior-point method with a specialized homogeneous embedding to solve linear, quadratic, and conic optimization problems.

Its defining advantage is the combination of native convex quadratic objectives, expressive conic constraints, infeasibility detection, and configurable sparse linear algebra within a single solver.

Clarabel is particularly relevant when an optimization problem has continuous variables, convex objectives, and constraints that can be represented through supported convex cones.

Unlike many specialized quadratic-programming solvers, Clarabel supports second-order, exponential, power, generalized power, and positive-semidefinite cones. Unlike some homogeneous-embedding conic solvers, it can handle quadratic objectives directly rather than converting them into additional conic constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 1.1 Capability map

| Capability                    | What Clarabel provides                                                                    |
| ----------------------------- | ----------------------------------------------------------------------------------------- |
| Linear programming            | Continuous LPs with equalities, inequalities, and bounds                                  |
| Quadratic programming         | Convex QPs with native quadratic objectives                                               |
| Second-order cone programming | Norm constraints, conic quadratic optimization, robust convex models                      |
| Semidefinite programming      | Positive-semidefinite matrix constraints, with optional chordal decomposition             |
| Exponential-cone programming  | Exponential relationships, entropy, log-sum-exp and related convex formulations           |
| Power-cone programming        | Geometric-mean, weighted-product and power-law constraints                                |
| Generalized power cones       | Multivariate weighted products combined with Euclidean norm bounds                        |
| Infeasibility detection       | Homogeneous embedding and primal/dual infeasibility certificates                          |
| Sparse linear algebra         | QDLDL, optional Faer and Pardiso factorization backends                                   |
| Repeated optimization         | In-place numerical updates with fixed dimensions and sparsity                             |
| Custom termination            | Per-iteration callbacks with access to solver progress                                    |
| Numerical configuration       | Equilibration, regularization, iterative refinement, tolerances and precision selection   |
| Rust extensibility            | Solver traits, numerical abstractions, structured problem data and optional FFI           |
| Interoperability              | Native Rust, Python bindings, C/C++ and R interfaces, and a separate Julia implementation |

The library is an optimization solver, not a complete mathematical modeling language. It expects the application to formulate its problem as sparse matrices, vectors and cone specifications.

Its main responsibility is numerical solution, including convergence management, KKT system construction, linear algebra, and solver termination. It does not automatically derive arbitrary nonlinear equations, infer convexity from general expressions, or transform discrete decision problems into mixed-integer optimization models.

### 1.2 Where Clarabel sits in the solver landscape

Clarabel occupies an important position between several established categories:

- Compared with classical LP/QP solvers, it supports a substantially broader set of conic constraints.
- Compared with first-order conic solvers, it emphasizes interior-point accuracy and numerical convergence, at the cost of more expensive linear algebra per iteration.
- Compared with general nonlinear programming solvers, it sacrifices nonconvex modeling generality in exchange for specialized convex optimization algorithms and convexity-based guarantees.
- Compared with mixed-integer optimizers, it provides a continuous convex solver, not native branch-and-bound or integer-variable enforcement.

These distinctions make Clarabel a useful numerical engine for independently constructed conic models, convex subproblems within larger algorithms, and systems requiring higher-accuracy continuous solutions.

## 2. Mathematical model and problem representation

Clarabel solves problems of the form

\\[ \begin{aligned} \underset{x,s}{\operatorname{minimize}}\quad& \frac12 x^\top Px+q^\top x\\\ \text{subject to}\quad& Ax+s=b,\\\ &s\in\mathcal K, \end{aligned} \\]

where \\(P\succeq0\\), and the constraint cone is a Cartesian product:

\\[ \mathcal K=\mathcal K_1\times\cdots\times\mathcal K_p. \\]

The key architectural idea is that the nonlinear geometry of constraints is represented by cones, while the maps into those cones remain affine.

This creates a common numerical interface for substantially different optimization problems.

### 2.1 Primal and dual formulation

Clarabel's associated dual problem is

\\[ \begin{aligned} \underset{x,z}{\operatorname{maximize}}\quad& -\frac12x^\top Px-b^\top z\\\ \text{subject to}\quad& Px+A^\top z=-q,\\\ &z\in\mathcal K^\*. \end{aligned} \\]

Here \\(\mathcal K^\*\\) denotes the dual cone.

The corresponding KKT conditions include

\\[ \begin{aligned} Ax+s&=b,\\\ Px+A^\top z+q&=0,\\\ s^\top z&=0,\\\ (s,z)&\in\mathcal K\times\mathcal K^\*. \end{aligned} \\]

These conditions are important because Clarabel exposes more than a primal decision vector. Its solution object includes primal variables, dual variables and cone slacks, allowing callers to analyze feasibility, optimality and dual information.

For an adequately regular convex problem, successful convergence provides a numerical approximation to the primal-dual optimum. The distinction between such numerical convergence and exact mathematical certification remains important.

Source: [https://arxiv.org/html/2405.12762](https://arxiv.org/html/2405.12762?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org+1



### 2.2 Representing common constraints

Most basic linear constraints require no special modeling machinery beyond the `ZeroConeT` and `NonnegativeConeT` types.

| Mathematical constraint                           | Clarabel representation                                      |
| ------------------------------------------------- | ------------------------------------------------------------ |
| \\(Cx=d\\)                                        | `ZeroConeT` with \\(A=C,\ b=d\\)                             |
| \\(Cx\le d\\)                                     | `NonnegativeConeT` with \\(A=C,\ b=d\\)                      |
| \\(Cx\ge d\\)                                     | `NonnegativeConeT` with \\(A=-C,\ b=-d\\)                    |
| \\(l\le Cx\le u\\)                                | Stack \\(C\\) and \\(-C\\), with bounds \\(u\\) and \\(-l\\) |
| \\(\\\|Fx+g\\\|\_2\le h^\top x+k\\)               | Affine map into `SecondOrderConeT`                           |
| \\(M(x)\succeq0\\), with affine symmetric \\(M\\) | Scaled triangular affine map into `PSDTriangleConeT`         |

A significant practical detail is the sign convention:

\\[ s=b-Ax\in\mathcal K. \\]

For example, to impose \\(x\in\mathcal K\_{\mathrm{SOC}}\\), a direct representation is \\(A=-I\\), \\(b=0\\), so \\(s=x\\).

This uniform representation also means that user-facing variable bounds are not a separate native modeling layer. They are encoded as conic constraints.

### 2.3 Native quadratic objectives

Clarabel incorporates \\(P\\) directly into its interior-point equations.

This distinguishes it from approaches that first reformulate

\\[ \frac12x^\top Px \\]

as an epigraph constraint involving an additional cone.

An epigraph transformation can introduce extra variables and constraints and may require a factorization of \\(P\\). For sparse quadratic programs, these transformations can materially change the size and sparsity of the numerical system.

Clarabel avoids that reformulation, potentially reducing both problem-construction complexity and KKT factorization work.

However, this applies to convex quadratic objectives, where \\(P\succeq0\\). A general indefinite quadratic objective does not become admissible merely because Clarabel supports quadratic terms.

The same distinction applies to quadratic constraints: appropriately structured convex quadratic inequalities can often be represented using second-order or rotated-second-order-cone formulations, but Clarabel does not expose a general nonlinear quadratic-constraint parser.

Source: [https://arxiv.org/html/2405.12762](https://arxiv.org/html/2405.12762?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org+1



## 3. Supported cone families

Clarabel's breadth comes from its native support for both symmetric and nonsymmetric convex cones.

The supported Rust cone types are exposed through `SupportedConeT<T>` and re-exported by `clarabel::solver`.

### 3.1 Complete native cone inventory

| Rust type             | Mathematical meaning                                     | Important capabilities                                         |
| --------------------- | -------------------------------------------------------- | -------------------------------------------------------------- |
| `ZeroConeT(n)`        | \\(s=0\\)                                                | Affine equalities                                              |
| `NonnegativeConeT(n)` | \\(s_i\ge0\\)                                            | Inequalities, variable bounds, LPs and QPs                     |
| `SecondOrderConeT(n)` | \\(s_0\ge\\\|s\_{1:}\\\|\_2\\)                           | Norm bounds, robust optimization, convex quadratic constraints |
| `ExponentialConeT()`  | Closure of \\(y>0,\ ye^{x/y}\le z\\)                     | Entropy, exponential inequalities, log-sum-exp formulations    |
| `PowerConeT(a)`       | \\(x^ay^{1-a}\ge\|z\|\\), \\(x,y\ge0\\)                  | Weighted geometric means and power relationships               |
| `GenPowerConeT(a,n)`  | \\(\prod_i x_i^{a_i}\ge\\\|y\\\|\_2\\)                   | Multivariate geometric-mean and weighted-product constraints   |
| `PSDTriangleConeT(n)` | Scaled triangular representation of \\(\mathbb S\_+^n\\) | Semidefinite matrix inequalities, SDPs                         |

The three-dimensional power cone requires \\(0\<a<1\\); generalized-power weights must be positive and sum to one. The positive-semidefinite cone is available in Rust when the `sdp` feature is enabled.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs+1



### 3.2 Second-order cones

Second-order cones express constraints of the form

\\[ \\|(u_1,\ldots,u_k)\\|\_2\le t. \\]

They make Clarabel useful for much more than ordinary quadratic programming.

An affine second-order constraint can be used to represent Euclidean norm limits, conic quadratic inequalities, uncertainty sets, convex robust counterparts and several risk measures.

A particularly important distinction is that the existence of a quadratic term does not require an SOCP reformulation when that quadratic appears in the objective. Clarabel can handle a positive-semidefinite objective directly and use second-order cones only where the constraints require them.

Rotated second-order cones: Clarabel's public cone inventory does not contain a separate rotated-SOC variant. Such constraints can nevertheless be represented by applying a linear transformation into the standard SOC. The caller or modeling layer must perform that transformation.

This is useful for convex inequalities such as

\\[ 2uv\ge \\|w\\|\_2^2,\qquad u,v\ge0. \\]

### 3.3 Exponential cones

The exponential cone is defined by the closure of

\\[ \mathcal K\_{\exp} = \left\\{ (x,y,z): y>0,\quad ye^{x/y}\le z \right\\}. \\]

Its defining value is that many expressions involving exponentials and logarithms admit exact conic representations, rather than requiring iterative local nonlinear approximations.

Applications include entropy-related optimization, relative-entropy formulations, geometric-programming transformations and log-sum-exp constructions.

For example, an exponential inequality \\(e^u\le t\\) can be imposed through

\\[ (u,1,t)\in\mathcal K\_{\exp}. \\]

The representation is conic and convex, but the caller still needs to construct the appropriate affine mapping and any additional auxiliary variables.

### 3.4 Power and generalized power cones

The ordinary power cone represents

\\[ x^a y^{1-a}\ge |z|, \qquad x,y\ge0. \\]

Its generalized form extends weighted geometric products to multiple variables:

\\[ \prod\_{i=1}^{k}x_i^{a_i}\ge\\|y\\|\_2, \qquad x_i\ge0,\quad \sum_i a_i=1. \\]

These cones are useful because they express an entire class of geometric-mean inequalities natively.

For instance, a geometric-mean constraint involving several nonnegative quantities need not always be decomposed into numerous auxiliary second-order cones. The generalized power cone provides a direct representation of the relevant geometry.

Whether that direct formulation is numerically superior depends on the structure and the conditioning of the problem. A smaller conic representation does not guarantee cheaper interior-point iterations.

The generalized power cone is a particularly important capability because many conic solvers either omit it or require an alternative reformulation.

### 3.5 Semidefinite cones

Semidefinite programming introduces matrix-valued cone constraints:

\\[ X\succeq0. \\]

These arise in convex relaxations, covariance estimation, control-theoretic matrix inequalities, moment problems, spectral optimization and other applications involving symmetric matrices.

In Clarabel, `PSDTriangleConeT(n)` denotes an \\(n\times n\\) positive-semidefinite matrix, represented by \\(n(n+1)/2\\) scalar entries.

Its vectorization uses the upper triangle in column-major order with each off-diagonal entry multiplied by \\(\sqrt{2}\\).

For example, a symmetric \\(2\times2\\) matrix

\\[ X= \begin{bmatrix}a&b\\\b&c\end{bmatrix} \\]

is represented by

\\[ \operatorname{svec}(X)= \begin{bmatrix}a\\\\\sqrt{2}b\\\c\end{bmatrix}. \\]

This convention preserves the Frobenius inner product:

\\[ \operatorname{tr}(XY) = \operatorname{svec}(X)^\top\operatorname{svec}(Y). \\]

Incorrect triangular ordering or omission of the scaling factor changes the mathematical model.

Semidefinite capability depends on a BLAS/LAPACK-enabled build, and large dense PSD blocks can be considerably more demanding than similarly sized linear or second-order-cone blocks.

Sources:

[https://clarabel.org/stable/api_cone_types/](https://clarabel.org/stable/api_cone_types/?utm_source=chatgpt.com)

[https://clarabel.org/stable/examples/rs/example_sdp/](https://clarabel.org/stable/examples/rs/example_sdp/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs+1

### 3.6 Mixed cone compositions

Clarabel permits different cone families within the same optimization problem.

A single model can therefore combine a quadratic objective, equality constraints, ordinary inequalities, several SOC constraints, exponential constraints and positive-semidefinite conditions.

The composite cone architecture eliminates the need to coordinate separate solvers for each constraint type, provided all constraints admit a valid Clarabel representation.

The cone list's ordering is unrestricted, but its block dimensions and the corresponding row ordering in \\(A\\) and \\(b\\) must agree.

Architectural implication: Combining native cones can eliminate bespoke outer-loop coordination between independently solved constraint families. It does not eliminate the need to build the correct common convex formulation.

## 4. Interior-point architecture and homogeneous embedding

Clarabel's numerical method is more distinctive than the generic description “primal-dual interior-point solver” suggests.

It adapts a homogeneous embedding for monotone complementarity problems to convex quadratic conic optimization. This is what allows it to preserve quadratic objectives while retaining the ability to identify primal or dual infeasibility.

### 4.1 Homogeneous embedding

Classical homogeneous self-dual embedding (HSDE) is especially natural for linear-objective conic programs. For a quadratic objective, directly applying the conventional approach is less straightforward.

Clarabel instead constructs a homogeneous system involving the primal and dual variables and additional homogeneous variables commonly denoted by \\(\tau\\) and \\(\kappa\\).

Conceptually, the embedding separates three possible outcomes:

- A recoverable optimal primal-dual solution.
- A direction providing evidence of primal infeasibility.
- A direction providing evidence of dual infeasibility, which under appropriate conditions indicates an unbounded primal problem.

The solver can therefore report infeasibility through its algorithmic formulation rather than relying solely on failed optimization iterations.

However, the homogeneous formulation does not make every ill-conditioned problem numerically decidable. Clarabel also has explicit numerical failure, insufficient-progress, and approximate-solution statuses.

Source: [https://arxiv.org/html/2405.12762](https://arxiv.org/html/2405.12762?utm_source=chatgpt.com) — Sections 2.1 and 2.6.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org+1



### 4.2 Predictor-corrector iterations

The algorithm follows a primal-dual central path and uses predictor-corrector search directions.

Each major iteration involves:

1. Evaluating primal feasibility, dual feasibility, complementarity and barrier quantities.
2. Forming the cone-dependent scaling operator.
3. Solving a sparse KKT system for a predictor direction.
4. Constructing a centering/corrector direction.
5. Selecting a step length that respects cone interiors and numerical safeguards.
6. Updating the homogeneous variables and testing termination.

The algorithm is not a generic Newton method applied to arbitrary nonlinear equations. The cone-specific barrier machinery and homogeneous variables are central to its convergence behavior.

### 4.3 Symmetric versus nonsymmetric cone treatment

A consequential architectural distinction is Clarabel's separate treatment of symmetric and nonsymmetric cones.

Symmetric cones include the nonnegative orthant, second-order cones and positive-semidefinite cones. They support self-scaled barriers and Nesterov–Todd scaling.

Nonsymmetric cones include exponential and power cones. They require different scaling constructions based on barrier derivatives.

For nonsymmetric cones, Clarabel uses a third-order correction approach rather than applying the symmetric-cone Mehrotra correction unchanged.

This distinction matters because support for an exponential or generalized power cone is not simply a matter of adding a geometric membership test. It requires compatible barrier functions, scaling, central-path initialization and step-selection logic.

Source: [https://arxiv.org/html/2405.12762](https://arxiv.org/html/2405.12762?utm_source=chatgpt.com) — Sections 3.1–3.2.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org



## 5. Sparse KKT systems and linear algebra backends

The principal computational cost of a Clarabel iteration is generally the solution of a sparse, symmetric, indefinite linear system.

A regularized representation of its KKT matrix is

\\[ K= \begin{bmatrix} P+\epsilon_s I&A^\top\\\ A&-(H+\epsilon_s I) \end{bmatrix}, \\]

where \\(H\\) captures cone-dependent scaling.

Clarabel uses a sparse direct \\(LDL^\top\\) factorization:

\\[ K=LDL^\top. \\]

The symbolic sparsity structure can remain fixed while numerical values change across iterations. This creates an opportunity to reuse symbolic analysis and memory allocations, reducing repeated preparation work.

Static regularization helps retain quasidefinite structure, including when \\(P\\) is rank deficient. Dynamic regularization protects against numerically troublesome pivots, and iterative refinement improves the accuracy of computed Newton directions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org



### 5.1 Rust linear-solver backends

| Backend             | Availability                   | Distinctive characteristics                                                                |
| ------------------- | ------------------------------ | ------------------------------------------------------------------------------------------ |
| QDLDL               | Built in                       | Sparse quasidefinite \\(LDL^\top\\), pure Rust, single-threaded; lightweight baseline      |
| Faer sparse LDL     | `faer-sparse` feature          | Native Rust supernodal factorization, with support for multithreading                      |
| Intel MKL Pardiso   | `pardiso-mkl` feature          | External high-performance sparse direct solver, dynamically loaded; x86-64 MKL requirement |
| Panua Pardiso       | `pardiso-panua` feature        | External sparse direct solver requiring a separate commercial license                      |
| Automatic selection | `direct_solve_method = "auto"` | Selects from available backends according to the build and platform                        |

The `direct_solve_method` setting selects the KKT linear solver. `max_threads` controls applicable multithreaded backends; it does not turn QDLDL into a multithreaded implementation.

The available backend names also differ between Rust and Julia. For example, MA57 and CHOLMOD are documented as Julia-side options, not native Rust Clarabel backends.

The `faer-sparse` option is especially notable because it provides a higher-performance sparse direct linear algebra route without necessarily introducing a proprietary numerical runtime.

Sources:

[https://clarabel.org/stable/user_guide_linsolvers/](https://clarabel.org/stable/user_guide_linsolvers/?utm_source=chatgpt.com)

[https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultSettings.html](https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultSettings.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs+1

### 5.2 What backend selection changes

The mathematical optimization method remains Clarabel's interior-point algorithm. What changes is how the Newton/KKT system is factorized.

The relative merits depend on:

- KKT matrix size and sparsity structure.
- Fill-in under the symbolic factorization.
- Supernodal structure and available parallelism.
- Factorization setup cost versus repeated numerical factorizations.
- Memory bandwidth, memory capacity and runtime library overhead.
- Numerical behavior on ill-conditioned systems.

This means switching to a more sophisticated backend does not guarantee a faster complete solve. Small problems may favor QDLDL's minimal overhead; larger or structurally favorable problems may benefit more from Faer or Pardiso.

### 5.3 Sparse LDL versus BLAS/LAPACK

Two distinct linear-algebra needs should not be conflated:

Sparse KKT factorization: QDLDL, Faer or Pardiso solves the indefinite linear systems arising in interior-point iterations.

Dense semidefinite-cone algebra: SDP support uses BLAS/LAPACK-related machinery for matrix operations associated with positive-semidefinite cones.

Selecting a BLAS library for SDP support does not automatically replace the sparse KKT factorization backend.

In particular, a CUDA-enabled BLAS installation should not be interpreted as native CUDA execution of Clarabel's complete interior-point algorithm.

### 5.4 GPU execution boundary

The verified Rust release provides CPU-oriented sparse direct factorization backends. There is no documented native CUDA, ROCm, or GPU KKT solver backend in Clarabel.rs 0.11.1.

This matters because a GPU acceleration strategy would need to address more than matrix multiplication: sparse symbolic analysis, numerical factorization, cone-specific scaling, host/device transfers and the synchronization between sequential interior-point iterations all affect performance.

Clarabel's Rust numerical core may be composed with GPU-enabled components elsewhere in a system, but that is not equivalent to a supported GPU solver implementation.

## 6. Presolve, scaling, and structural reformulation

Clarabel performs several forms of numerical and structural preparation before iterations begin. These can materially change both speed and the ability to reuse a solver instance.

### 6.1 Presolve and cone simplification

The setup path includes problem-dimension validation, cone normalization, optional presolve, sparse-data preparation, scaling, and KKT-system construction.

Notable optimizations include:

- Consolidation of compatible consecutive cones.
- Simplification of degenerate or one-dimensional cone instances.
- Optional removal of explicitly stored zeros from sparse inputs.
- Presolve reductions that can eliminate or modify portions of the original problem.

These operations reduce unnecessary numerical work in favorable cases.

Their important tradeoff is that they may change the internal structure relative to the caller's original matrices. This affects subsequent in-place data updates, discussed below.

### 6.2 Equilibration

Clarabel can equilibrate problem data to reduce differences between row and column magnitudes.

Relevant settings include `equilibrate_enable`, `equilibrate_max_iter`, `equilibrate_min_scaling` and `equilibrate_max_scaling`.

Equilibration can improve KKT conditioning and convergence when input coefficients span very different scales. However, it is not a replacement for sound physical units or mathematically well-conditioned formulations.

A significant distinction for consumers of the Rust API: `solver.solution` contains the recovered solution, whereas internal iteration variables can remain in transformed or homogeneous coordinates. Directly interpreting internal `solver.variables` as the original problem's decision variables would be incorrect.

Source: [https://clarabel.org/stable/rust/getting_started_rs/](https://clarabel.org/stable/rust/getting_started_rs/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs



### 6.3 Chordal decomposition of semidefinite cones

Chordal decomposition is one of Clarabel's most consequential advanced capabilities.

For a large but sparsely structured semidefinite constraint, it can replace a single large PSD cone with several smaller PSD cones linked through their overlapping entries.

Conceptually:

Large sparse PSD constraint

\\(X(x)\succeq0\\)

Clique PSD block A

Clique PSD block B

Clique PSD block C

Overlapping matrix entries and consistency constraints connect the smaller blocks.

The advantage arises from exploiting sparsity in the semidefinite structure, not merely sparsity in the ordinary constraint matrix \\(A\\).

If the maximum clique size is much smaller than the full PSD matrix dimension, the savings in dense cone calculations and memory may be substantial. If the resulting cliques are large or overlap extensively, the decomposition can instead introduce overhead.

### 6.4 Chordal controls and reconstruction

Rust's `DefaultSettings` exposes:

| Setting                               | Purpose                                         |
| ------------------------------------- | ----------------------------------------------- |
| `chordal_decomposition_enable`        | Enable or disable SDP decomposition             |
| `chordal_decomposition_merge_method`  | Select clique-merging strategy                  |
| `chordal_decomposition_compact`       | Control the compact decomposition formulation   |
| `chordal_decomposition_complete_dual` | Control completion/recovery of dual information |

The documented merging strategies include `none`, `parent_child` and `clique_graph`.

These options determine the tradeoff between the number of cones, their sizes, the linking structure and the cost of recovering solution information.

Chordal decomposition is especially important when solving sparse SDPs involving many smaller, interconnected matrix blocks. It is less relevant to purely linear, quadratic or SOCP problems.

Like other structural transformations, it also has implications for incremental optimization: if decomposition changes the solver's internal structure, Clarabel cannot use its normal fixed-pattern data-update path.

Sources:

[https://clarabel.org/stable/user_guide_chordal/](https://clarabel.org/stable/user_guide_chordal/?utm_source=chatgpt.com)

[https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultSettings.html](https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultSettings.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs+1

## 7. Rust API and data representation

Clarabel's public Rust interface is relatively small at the application boundary, even though the underlying numerical implementation is extensively modularized.

The principal types are:

| Type                        | Responsibility                            |
| --------------------------- | ----------------------------------------- |
| `CscMatrix<T>`              | Compressed-sparse-column matrices         |
| `SupportedConeT<T>`         | Cone specifications                       |
| `DefaultSettings<T>`        | Solver and numerical configuration        |
| `DefaultSettingsBuilder<T>` | Builder-based settings construction       |
| `DefaultSolver<T>`          | Primary solver instance                   |
| `IPSolver`                  | Trait exposing the `solve()` operation    |
| `DefaultSolution<T>`        | Recovered primal/dual solution and status |
| `DefaultInfo<T>`            | Detailed iteration and solver diagnostics |
| `SolverStatus`              | Termination outcome                       |

### 7.1 Solver construction and execution

The primary constructor is conceptually:

```
DefaultSolver::new(&P, &q, &A, &b, &cones, settings)
```

It returns `Result<DefaultSolver<T>, SolverError>`, allowing invalid dimensions or settings to be handled at construction.

The solver then runs through `solve()` and exposes the result through `solver.solution`.

The following is an illustrative example aligned with the documented 0.11.1 Rust API; it has not been executed as part of this research.

```
use clarabel::algebra::*;
use clarabel::solver::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = CscMatrix::from(&[
        [6.0, 0.0],
        [0.0, 4.0],
    ]);
    let q = vec![-1.0, -4.0];

    let a = CscMatrix::from(&[
        [ 1.0, -2.0], // equality
        [ 1.0,  0.0], // inequalities
        [ 0.0,  1.0],
        [-1.0,  0.0],
        [ 0.0, -1.0],
    ]);
    let b = vec![0.0, 1.0, 1.0, 1.0, 1.0];

    let cones = [
        ZeroConeT(1),
        NonnegativeConeT(4),
    ];

    let settings = DefaultSettings {
        verbose: false,
        ..DefaultSettings::default()
    };

    let mut solver =
        DefaultSolver::new(&p, &q, &a, &b, &cones, settings)?;

    solver.solve();

    match solver.solution.status {
        SolverStatus::Solved => {
            println!("primal: {:?}", solver.solution.x);
            println!("dual:   {:?}", solver.solution.z);
            println!("slack:  {:?}", solver.solution.s);
        }
        status => {
            eprintln!("solver status: {status:?}");
        }
    }

    Ok(())
}
```

The first row of \\(A\\) is paired with the zero cone; the remaining four rows are paired with the nonnegative cone. Their ordering is part of the model's semantics.

Source: [https://clarabel.org/stable/rust/getting_started_rs/](https://clarabel.org/stable/rust/getting_started_rs/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs



### 7.2 Sparse CSC matrix handling

Clarabel uses its own `CscMatrix<T>` representation, with fields:

```
pub struct CscMatrix<T> {
    pub m: usize,
    pub n: usize,
    pub colptr: Vec<usize>,
    pub rowval: Vec<usize>,
    pub nzval: Vec<T>,
}
```

It supports direct CSC construction, triplet construction, sparse allocation, identity and zero matrices, transpose-related operations, row selection, sparse concatenation and canonicalization.

Several details are especially relevant to advanced integrations:

- `P` only needs the upper triangular portion of the symmetric quadratic matrix.
- The order of stored coefficients is consequential for indexed numerical updates.
- `canonicalize()` handles the canonical sparse representation, including ordering and duplicate entries.
- `dropzeros()` is separate; numerical zero and structurally absent are not necessarily interchangeable.
- `check_format()` can validate the CSC representation.

The official Rust guide explicitly states that the solver does not automatically verify the internal correctness of every supplied `CscMatrix`. This is important when matrices are assembled directly from external sparse-data pipelines.

Source: [https://docs.rs/clarabel/latest/clarabel/algebra/struct.CscMatrix.html](https://docs.rs/clarabel/latest/clarabel/algebra/struct.CscMatrix.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.3 Settings construction

The settings object has 43 documented fields in v0.11.1. Two supported construction patterns are struct-update syntax and `DefaultSettingsBuilder`.

For example:

```
let settings = DefaultSettingsBuilder::<f64>::default()
    .verbose(false)
    .max_iter(200)
    .tol_gap_abs(1e-9)
    .tol_feas(1e-9)
    .direct_solve_method("auto".to_string())
    .build()?;
```

The builder is preferable when explicit settings validation and configuration assembly are useful.

The most significant settings families concern tolerances, equilibration, regularization, linear-solver selection, iterative refinement, presolve and chordal decomposition.

Source: [https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultSettingsBuilder.html](https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultSettingsBuilder.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 8. Repeated optimization and parameter updates

In-place problem-data updates are among Clarabel's most valuable capabilities for applications solving a family of related convex problems.

A solver instance can be retained while coefficients are changed, avoiding reconstruction of the entire solver object on each run.

### 8.1 Supported update operations

Rust exposes the following methods:

| Method                     | Updated data                                   |
| -------------------------- | ---------------------------------------------- |
| `update_P(...)`            | Quadratic objective matrix                     |
| `update_q(...)`            | Linear objective vector                        |
| `update_A(...)`            | Constraint coefficient matrix                  |
| `update_b(...)`            | Constraint right-hand side                     |
| `update_data(...)`         | All four data components                       |
| `is_data_update_allowed()` | Query whether the solver permits data updating |

The update interfaces accept full replacements and more selective representations, including indexed value updates for matrix data.

This is materially more flexible than an interface supporting only right-hand-side changes.

For example, varying load forecasts, objective coefficients, penalty weights, matrix coefficients or operating bounds can be handled through the appropriate problem-data updates when the structure remains compatible.

### 8.2 Structural invariants

The main restrictions are:

- The number of variables and constraints must remain unchanged.
- Sparse matrix patterns must remain compatible.
- The ordered cone specifications cannot be replaced.
- `P` updates refer to its upper-triangular representation.
- Presolve, chordal decomposition or removal of stored zeros may prevent updates if they change internal structure.

The most important settings combination for guaranteed structural update eligibility is:

```
let settings = DefaultSettings {
    presolve_enable: false,
    chordal_decomposition_enable: false,
    input_sparse_dropzeros: false,
    ..DefaultSettings::default()
};
```

This is not an unconditional performance recommendation. Disabling these transformations can make an individual solve slower, particularly if presolve or chordal decomposition would otherwise deliver substantial reductions.

The value of data updates depends on whether saved setup work outweighs the optimizations forgone.

The Rust API is slightly more permissive than a simplified reading of the user guide: a transformation setting being enabled does not necessarily prohibit updating if the transformation did not actually modify the problem structure. The methods return explicit update errors when the operation is inadmissible.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 8.3 Reusing structure is not the same as warm starting

Three different optimizations should be distinguished:

| Mechanism                         | Meaning                                                   | Clarabel 0.11.1                            |
| --------------------------------- | --------------------------------------------------------- | ------------------------------------------ |
| Reuse the solver instance         | Avoid reconstructing an equivalent internal solver        | Supported                                  |
| Update numerical coefficients     | Modify \\(P,q,A,b\\) while preserving structure           | Supported                                  |
| Warm start primal-dual iterations | Initialize a new solve from a previous numerical solution | Not supported as a released public feature |

The fixed-pattern update interface is useful even without warm starting because it avoids portions of solver setup and memory allocation.

However, updating the data does not imply that Clarabel will initialize the new interior-point trajectory from the preceding solution.

As of November 10, 2026, upstream pull request #219, explicitly proposing warm-start support, remains open. Its reported benchmark improvements are results from an unmerged experimental implementation, not capabilities or performance guarantees of the released crate.

Source: [https://github.com/oxfordcontrol/Clarabel.rs/pull/219](https://github.com/oxfordcontrol/Clarabel.rs/pull/219?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



This distinction is especially consequential for repeated small QPs, model-predictive control, sequential convex programming and other workloads where neighboring problems may have closely related solutions.

### 8.4 Implications for orchestration

The documented update mechanisms can reduce the need for custom solver-object rebuilding and repeated sparse-structure allocation.

They do not replace an application's responsibilities for deciding:

- Whether two scenarios share the same cone topology.
- Whether the sparsity pattern remains valid.
- Whether the changed \\(P\\) remains positive semidefinite.
- Whether a different problem structure requires a new solver instance.
- Whether solutions from distinct scenarios can be reused analytically outside Clarabel.

Source: [https://clarabel.org/stable/user_guide_data_updating/](https://clarabel.org/stable/user_guide_data_updating/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs



## 9. Termination, diagnostics, and numerical reliability

Clarabel provides detailed termination information rather than reducing every outcome to a success/failure boolean.

### 9.1 Complete solver statuses

| Status                   | Interpretation                                      |
| ------------------------ | --------------------------------------------------- |
| `Solved`                 | Converged to the configured solution tolerances     |
| `AlmostSolved`           | Converged under reduced-accuracy tolerances         |
| `PrimalInfeasible`       | Primal infeasibility detected; certificate returned |
| `AlmostPrimalInfeasible` | Reduced-accuracy primal infeasibility determination |
| `DualInfeasible`         | Dual infeasibility detected; certificate returned   |
| `AlmostDualInfeasible`   | Reduced-accuracy dual infeasibility determination   |
| `MaxIterations`          | Iteration limit reached                             |
| `MaxTime`                | Time limit reached                                  |
| `NumericalError`         | Numerical failure                                   |
| `InsufficientProgress`   | Algorithm unable to make sufficient progress        |
| `CallbackTerminated`     | Stopped by user-defined termination callback        |
| `Unsolved`               | Initial state before solving                        |

The approximate statuses are not equivalent to the corresponding full-accuracy statuses. Likewise, `MaxIterations` and `MaxTime` do not establish that a returned iterate is feasible or optimal.

For infeasibility statuses, the documented solution contains the relevant certificate information rather than an ordinary optimal decision vector.

Source: [https://docs.rs/clarabel/latest/clarabel/solver/enum.SolverStatus.html](https://docs.rs/clarabel/latest/clarabel/solver/enum.SolverStatus.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.2 Solution and diagnostic fields

The `solver.solution` object exposes:

- `x`, `z`, `s`: primal decision variables, dual variables and primal cone slacks.
- `status`: termination condition.
- `obj_val`, `obj_val_dual`: primal and dual objective values.
- `r_prim`, `r_dual`: final residual measures.
- `iterations`, `solve_time`: execution statistics.

The separate `solver.info` object includes more algorithmic information:

| Field                            | Interpretation                                    |
| -------------------------------- | ------------------------------------------------- |
| `mu`                             | Interior-point central-path parameter             |
| `sigma`                          | Centering/reduction parameter                     |
| `step_length`                    | Current step size                                 |
| `cost_primal`, `cost_dual`       | Objective information                             |
| `res_primal`, `res_dual`         | Primal and dual residual measures                 |
| `res_primal_inf`, `res_dual_inf` | Infeasibility-related residuals                   |
| `gap_abs`, `gap_rel`             | Absolute and relative duality-gap measures        |
| `ktratio`                        | Homogeneous embedding diagnostic                  |
| `linsolver`                      | Selected KKT solver and factorization information |

The `linsolver` field is particularly useful: its `LinearSolverInfo` structure reports the backend name, threads used and nonzero counts `nnzA` and `nnzL`.

Consequently, Clarabel already exposes information for detecting sparse-factorization fill-in and backend selection without requiring a custom KKT instrumentation layer.

Sources:

[https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultInfo.html](https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/struct.DefaultInfo.html?utm_source=chatgpt.com)

[https://docs.rs/clarabel/latest/clarabel/solver/struct.LinearSolverInfo.html](https://docs.rs/clarabel/latest/clarabel/solver/struct.LinearSolverInfo.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 9.3 Numerical tolerance families

Clarabel distinguishes several forms of termination tolerance:

| Settings                           | Purpose                                    |
| ---------------------------------- | ------------------------------------------ |
| `tol_gap_abs`, `tol_gap_rel`       | Absolute and relative duality-gap accuracy |
| `tol_feas`                         | Primal and dual feasibility                |
| `tol_infeas_abs`, `tol_infeas_rel` | Infeasibility determination                |
| `tol_ktratio`                      | Homogeneous embedding termination          |
| `reduced_tol_*`                    | Thresholds used for approximate statuses   |

Other numerical controls include static/dynamic KKT regularization, iterative-refinement tolerances, line-search parameters, and minimum step-length thresholds.

Their interactions matter: tighter optimization tolerances may expose deficiencies in linear-system accuracy, while overly aggressive regularization can affect convergence accuracy. These are coupled numerical decisions, not independent precision switches.

### 9.4 Custom termination callbacks

Clarabel permits a Rust callback to inspect current iteration information and request termination.

An illustrative callback is:

```
solver.set_termination_callback(
    |info: &DefaultInfo<f64>| {
        info.iterations >= 50
    },
);
```

Returning `true` requests termination. The callback may capture application state and inspect available residual and timing information.

This enables custom stagnation detection, external cancellation checks, adaptive iteration budgets, and application-specific stopping conditions without modifying the core solver.

The callback is invoked during the solution process; it is not a mechanism for adding new mathematical constraints or changing the active optimization model at arbitrary iterations.

The callback can be removed with `unset_termination_callback()`.

Source: [https://clarabel.org/stable/examples/rs/example_callback/](https://clarabel.org/stable/examples/rs/example_callback/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs



## 10. Cargo features, numerical precision, and platform support

The `clarabel` crate uses compile-time features to control which numerical and interoperability capabilities are included.

### 10.1 Feature inventory

| Feature          | Effect                                                                 |
| ---------------- | ---------------------------------------------------------------------- |
| `serde`          | Serialization and JSON problem-data import/export; enabled by default  |
| `faer-sparse`    | Adds the Faer sparse LDL backend                                       |
| `pardiso-mkl`    | Enables Intel MKL Pardiso                                              |
| `pardiso-panua`  | Enables Panua Pardiso                                                  |
| `pardiso`        | Enables both Pardiso options                                           |
| `sdp`            | Adds semidefinite-cone support and associated BLAS/LAPACK dependencies |
| `sdp-openblas`   | SDP support using OpenBLAS                                             |
| `sdp-mkl`        | SDP support using Intel MKL                                            |
| `sdp-accelerate` | SDP support using Apple Accelerate                                     |
| `sdp-netlib`     | SDP support using reference Netlib BLAS/LAPACK                         |
| `sdp-r`          | SDP support through R-provided BLAS/LAPACK integration                 |
| `python`         | Builds the PyO3 Python interface                                       |
| `julia`          | Builds the interface used for Julia integration                        |
| `buildinfo`      | Enables build-configuration reporting                                  |
| `bench`          | Includes additional benchmarking functionality                         |

For example, a Rust build requiring both SDP support and the Faer backend can enable:

```
[dependencies]
clarabel = {
    version = "0.11.1",
    features = ["sdp-openblas", "faer-sparse"]
}
```

The practical build requirements depend on the chosen BLAS/LAPACK provider and external linear-solver libraries.

Importantly, `faer-sparse` and the SDP BLAS/LAPACK features control different parts of the numerical workload. They are not mutually exclusive substitutes.

Source: [https://docs.rs/crate/clarabel/0.11.1/features](https://docs.rs/crate/clarabel/0.11.1/features?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 10.2 Floating-point precision

Clarabel's numerical abstractions are generic over a `FloatT` type.

The ordinary native choices are `f32` and `f64`.

The Rust documentation makes a more nuanced statement about arbitrary-precision support:

- Without SDP support, other floating-point types can potentially be used if they satisfy the necessary `CoreFloatT` and related trait bounds.
- With BLAS/LAPACK-backed SDP support, the documented implementations are restricted to native `f32` and `f64`.
- Arbitrary-precision Julia `BigFloat` support belongs to the separate Julia implementation; it should not be assumed to be a turnkey Rust capability.

The existence of generic type bounds is not a guarantee that an arbitrary external precision type has a complete, tested solver implementation.

Source: [https://docs.rs/clarabel/latest/clarabel/algebra/trait.FloatT.html](https://docs.rs/clarabel/latest/clarabel/algebra/trait.FloatT.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 10.3 Compilation and platforms

The published crate specifies:

- Rust edition 2021.
- Minimum Rust version 1.70.0 for the base crate.
- Apache 2.0 licensing.
- Optional native BLAS/LAPACK and external sparse-factorization dependencies.

The crate includes WebAssembly compatibility code and test dependencies. The Rust ecosystem's `good_lp` integration also documents browser/WASM use with the Clarabel backend.

This makes non-SDP, pure-Rust solver configurations potentially useful in WASM deployments. That does not imply every feature combination is available on WASM: external Pardiso libraries, native BLAS dependencies and platform-specific threading require separate compatibility consideration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Sources:

[https://docs.rs/crate/clarabel/0.11.1/source/Cargo.toml.orig](https://docs.rs/crate/clarabel/0.11.1/source/Cargo.toml.orig?utm_source=chatgpt.com)

[https://github.com/oxfordcontrol/Clarabel.rs/blob/main/CHANGELOG.md](https://github.com/oxfordcontrol/Clarabel.rs/blob/main/CHANGELOG.md?utm_source=chatgpt.com)

## 11. Advanced integration and extension mechanisms

### 11.1 Trait-based solver architecture

Clarabel is not implemented as a completely opaque numerical object.

Its `DefaultSolver<T>` is a concrete type alias over a generic `Solver` assembled from several interacting components.

The public trait architecture includes:

| Trait         | Extensibility boundary                      |
| ------------- | ------------------------------------------- |
| `ProblemData` | Representation of optimization problem data |
| `Variables`   | Solver iteration variables                  |
| `Residuals`   | Problem-specific residual calculations      |
| `KKTSystem`   | Linear-system assembly and solution         |
| `Info`        | Monitoring and termination logic            |
| `Solution`    | Recovered result representation             |
| `Settings`    | Solver configuration                        |
| `InfoPrint`   | Diagnostic output behavior                  |

The documentation explicitly permits construction of custom solver implementations by supplying types that implement the required traits.

This is more consequential than simply exchanging QDLDL for Faer. It creates a route to specialized problem representations or numerical components that still participate in Clarabel's interior-point architecture.

However, these traits are interconnected. Replacing one component can require a compatible implementation of other components and their lifecycle behavior. This is an advanced extension mechanism, not evidence of an effortless runtime plugin interface.

For almost all conventional problems, the upstream documentation recommends the default implementation.

Source: [https://docs.rs/clarabel/latest/clarabel/solver/traits/index.html](https://docs.rs/clarabel/latest/clarabel/solver/traits/index.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 11.2 JSON problem serialization

The default-enabled `serde` feature exposes `SolverJSONReadWrite<T>` with:

```
solver.save_to_file(&mut file)?;

let restored =
    DefaultSolver::load_from_file(&mut file, None)?;
```

The API supports exporting and reloading solver problem data, with optional settings supplied on loading.

This is useful for reproducibility, creating portable solver test cases, isolating numerical failures and separating model generation from numerical execution.

The JSON interface is a problem-data serialization mechanism, not a documented checkpoint/restart system for interior-point iterations. It should not be treated as preserving the previous factorization or a warm-start state.

Source: [https://docs.rs/clarabel/latest/clarabel/solver/trait.SolverJSONReadWrite.html](https://docs.rs/clarabel/latest/clarabel/solver/trait.SolverJSONReadWrite.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 11.3 Configurable output and build introspection

Clarabel provides the `ConfigurablePrintTarget` interface for redirecting solver logs.

Supported operations include `print_to_stdout`, `print_to_file`, `print_to_stream`, `print_to_sink`, `print_to_buffer`, and `get_print_buffer`.

These can be useful when embedding the solver into a service, testing harness, notebook backend or application-specific logging infrastructure.

The optional `buildinfo` facility also exposes details about the compiled solver configuration.

A further, easily overlooked crate-level API consists of `INFINITY_DEFAULT`, `get_infinity()`, `set_infinity()` and `default_infinity()`, which control the threshold used for treating bounds as infinite. This is relevant when importing models that use large finite sentinels to represent unbounded constraints; changing the interpretation threshold can alter model semantics.

Sources:

[https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/type.DefaultSolver.html](https://docs.rs/clarabel/latest/clarabel/solver/implementations/default/type.DefaultSolver.html?utm_source=chatgpt.com)

[https://docs.rs/clarabel/latest/clarabel/](https://docs.rs/clarabel/latest/clarabel/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 11.4 Higher-level modeling through `good_lp`

Clarabel has a documented integration with the Rust modeling library `good_lp`.

The `good_lp` crate exposes Clarabel through its `clarabel` feature and solver adapter, with types including `ClarabelProblem` and `ClarabelSolution`.

This is useful when a project needs a higher-level Rust interface for linear objectives, affine constraints, variable definitions and access to dual values.

The integration exposes only a subset of Clarabel's native mathematical capabilities.

Inspection of the `good_lp` Clarabel adapter shows that it currently constructs equality and nonnegative cones and works with a linear objective. It does not expose the full range of native SOC, exponential, power, generalized-power or semidefinite modeling operations.

That means using Clarabel through `good_lp` is not equivalent to using Clarabel's full conic API.

This is a useful example of an important integration distinction: a modeling wrapper may improve ergonomics while narrowing the capabilities reachable from the underlying solver.

Sources:

[https://docs.rs/good_lp/latest/good_lp/solvers/clarabel/](https://docs.rs/good_lp/latest/good_lp/solvers/clarabel/?utm_source=chatgpt.com)

[https://docs.rs/good_lp/latest/src/good_lp/solvers/clarabel.rs.html](https://docs.rs/good_lp/latest/src/good_lp/solvers/clarabel.rs.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 11.5 Cross-language integrations

The core Rust implementation is also made available through other languages.

| Integration         | Relationship to Clarabel                                                          |
| ------------------- | --------------------------------------------------------------------------------- |
| Python `clarabel`   | Python interface to the Rust solver                                               |
| CVXPY               | Python modeling framework that can invoke Clarabel                                |
| C/C++               | Foreign-function interfaces around the Rust implementation                        |
| R                   | Separate R package integrating the Rust implementation                            |
| Julia `Clarabel.jl` | Separate native Julia solver implementation, with JuMP and Convex.jl integrations |

A version distinction is important: the version of a wrapper package does not necessarily match the version of the published Rust crate. For example, the R package can have its own release numbering and maintenance schedule.

Similarly, features and linear solver backends available to Julia's native implementation should not automatically be attributed to the Rust implementation.

Source: [https://clarabel.org/stable/](https://clarabel.org/stable/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org+1



## 12. Performance characteristics and comparison with adjacent solvers

### 12.1 What determines Clarabel's performance?

The main cost drivers are not simply the numbers of variables and constraints.

Important factors include:

- Sparse KKT structure and factorization fill-in.
- Rank deficiency and numerical conditioning.
- Number and dimensions of nonnegative, SOC and PSD blocks.
- Complexity of nonsymmetric cone scaling.
- Effectiveness of presolve and equilibration.
- Whether chordal decomposition reduces the size of PSD calculations.
- Sparse direct solver backend and available parallelism.
- Accuracy requirements and termination behavior.
- Solver construction cost versus repeated numerical solves.

A model with more scalar variables can sometimes be easier than a smaller model with one large dense semidefinite block. Likewise, a sparse QP with a favorable KKT structure may be considerably cheaper than an apparently simpler conic problem whose factorization fills in heavily.

### 12.2 Published benchmark evidence

The foundational Clarabel paper reports strong results across QP, constrained optimal control, LP, exponential-cone and SDP benchmarks, with particularly notable advantages for problems with quadratic objectives.

Those results are meaningful but should be interpreted in their original experimental context.

The 2024 evaluation used:

- Intel Xeon Platinum 8268 at 2.90 GHz.
- 64 GB RAM.
- Single-threaded execution.
- QDLDL for the reported Clarabel configurations unless otherwise noted.
- Rust compiler 1.72.0.
- A 300-second solve-time limit in most experiments.
- Presolve disabled where applicable to preserve comparability.

The paper compared Clarabel with several commercial and open-source solvers, including ECOS, HiGHS, MOSEK and Gurobi.

These measurements do not establish a universal performance ordering for the current 0.11.1 release, alternative KKT backends, different hardware or arbitrary workloads. In particular, they are not measurements of Clarabel running with Faer, Pardiso or a GPU.

Sources:

[https://arxiv.org/html/2405.12762](https://arxiv.org/html/2405.12762?utm_source=chatgpt.com)

[https://github.com/oxfordcontrol/ClarabelBenchmarks](https://github.com/oxfordcontrol/ClarabelBenchmarks?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://arxiv.org\&sz=32)

arxiv.org

### 12.3 Comparison with nearby optimization engines

| Solver     | Core approach                                               | Principal distinction relative to Clarabel                                                                        |
| ---------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Clarabel   | Primal-dual conic interior-point method                     | Native convex QP objective and broad cone support, with relatively high-accuracy interior-point solutions         |
| OSQP       | ADMM-based first-order QP method                            | Specialized convex QPs, low-overhead repeated solves and mature warm-start support                                |
| SCS        | Operator-splitting conic method                             | Broad conic coverage, quadratic objectives, first-order scaling options, and available GPU linear-system backends |
| HiGHS      | LP simplex/interior-point and QP methods, plus MIP          | Strong LP/QP/MIP specialization; not a general exponential/power/SDP conic engine                                 |
| Ipopt      | Nonlinear interior-point method                             | General smooth nonlinear constraints and potentially nonconvex objectives, generally targeting local solutions    |
| SCIP       | Mixed-integer optimization and constraint-solving framework | Discrete variables, branching, cuts and general mixed-integer models                                              |
| Hypatia.jl | Extensible conic interior-point method                      | More extensive support for specialized or exotic cones through barrier-oracle abstractions                        |

Two comparisons deserve particular attention.

Clarabel versus OSQP: Clarabel offers a broader convex modeling vocabulary. OSQP is more specifically optimized for bound-constrained convex QPs, has explicit warm starting and can benefit greatly from reused factorizations. Neither algorithm dominates every QP workload.

Clarabel versus SCS: Both support broad convex conic programming and native quadratic objectives in their current implementations. Clarabel uses a sparse-direct-factorization interior-point method, while SCS uses operator splitting. SCS also documents GPU-oriented linear-system options, whereas Clarabel 0.11.1 does not.

Source references:

[https://osqp.org/docs/](https://osqp.org/docs/?utm_source=chatgpt.com)

[https://www.cvxgrp.org/scs/index.html](https://www.cvxgrp.org/scs/index.html?utm_source=chatgpt.com)

[https://coin-or.github.io/Ipopt/index.html](https://coin-or.github.io/Ipopt/index.html?utm_source=chatgpt.com)

[https://www.scipopt.org/](https://www.scipopt.org/?utm_source=chatgpt.com)

[https://research.google/pubs/solving-natural-conic-formulations-with-hypatiajl/](https://research.google/pubs/solving-natural-conic-formulations-with-hypatiajl/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://osqp.org\&sz=32)

OSQP documentation+4

### 12.4 Clarabel inside larger optimization methods

Although Clarabel is not itself a mixed-integer or general nonlinear optimizer, continuous convex conic solvers are building blocks for more general algorithms.

Examples include:

- Convex subproblems generated by sequential convex optimization.
- Convex relaxations in mixed-integer branch-and-bound.
- Convex quadratic approximations in nonlinear optimization.
- Conic relaxations of nonconvex quadratic problems.
- Convex master problems in certain decomposition frameworks.
- Repeated optimization over parameterized scenarios.

These are architectural possibilities, not built-in Clarabel algorithms.

Using Clarabel in such frameworks requires another component to generate subproblems, manage convergence or branching, and enforce the correctness conditions of the outer algorithm.

## 13. Important limitations and engineering boundaries

| Boundary                          | Consequence                                                                                                   |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| Convexity                         | Requires a positive-semidefinite quadratic objective and compatible convex cone formulation                   |
| Integer variables                 | No native integer or binary decision variables                                                                |
| General nonlinear constraints     | Requires conic reformulation; no arbitrary nonlinear expression engine                                        |
| Warm starting                     | Not a released public capability in 0.11.1                                                                    |
| Parameter updates                 | Require compatible dimensions, cone structure and sparsity                                                    |
| GPU acceleration                  | No verified native GPU solver backend                                                                         |
| KKT systems                       | Released default solver requires a direct KKT linear solver                                                   |
| SDP support                       | Optional build feature with BLAS/LAPACK dependencies                                                          |
| Arbitrary cone types              | Not a general runtime barrier-oracle registry                                                                 |
| Numerical precision               | Finite-precision convergence and infeasibility results require appropriate status and residual interpretation |
| Automatic differentiation         | No verified native differentiable-solver or sensitivity-derivative API                                        |
| Large-scale first-order workloads | Sparse direct factorizations may become the limiting memory or time cost                                      |

Three boundaries are especially easy to misunderstand.

First, broad conic support is not equivalent to general nonlinear programming. Many important nonlinear expressions have convex conic formulations, but many others do not. Determining whether a formulation is valid remains the responsibility of the mathematical modeling layer.

Second, a solver's ability to report dual multipliers is not the same as providing a general automatic-differentiation interface. Dual solutions can support sensitivity analysis in suitable convex settings, but that does not amount to built-in implicit differentiation through the entire solver.

Third, the existence of modular Rust numerical traits does not mean arbitrary new cones or GPU backends are exposed as simple user-defined plugins. Both types of extension may involve substantial compatibility requirements with the solver's scaling, linear algebra and convergence machinery.

## 14. Key architectural takeaways

Clarabel has several characteristics that make it more capable than a simple Rust QP solver.

Its strongest feature is the unified convex conic model. Linear equalities, linear inequalities, norm constraints, exponentials, powers, generalized geometric means and semidefinite conditions can all participate in the same optimization problem, subject to the relevant build features.

Its most distinctive numerical feature is the direct treatment of convex quadratic objectives inside a homogeneous-embedding interior-point method. This avoids some reformulation overhead while preserving the framework for infeasibility detection.

Its most significant large-scale mechanisms are sparse KKT factorization, configurable LDL backends and chordal decomposition. These allow performance improvements that stem from exploiting mathematical and numerical structure rather than merely changing a solver tolerance.

Its most useful repeated-execution mechanism is fixed-pattern data updating. This can substantially simplify repeated convex optimization, but should not be confused with warm-starting, which remains outside the released v0.11.1 API.

Its most important Rust-specific design characteristic is the trait-based numerical architecture. Clarabel exposes meaningful internal extension boundaries without requiring most applications to use them. Higher-level modeling, including through `good_lp`, is available separately but does not necessarily expose the complete conic capability set.

## Primary-source reference map

| Topic                                  | Direct source                                                                                                                                                                                                                     |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Mathematical foundations and algorithm | [https://arxiv.org/html/2405.12762](https://arxiv.org/html/2405.12762?utm_source=chatgpt.com)                                                                                                                                     |
| Native Rust API                        | [https://docs.rs/clarabel/0.11.1/clarabel/](https://docs.rs/clarabel/0.11.1/clarabel/?utm_source=chatgpt.com)                                                                                                                     |
| Cone specifications                    | [https://clarabel.org/stable/api_cone_types/](https://clarabel.org/stable/api_cone_types/?utm_source=chatgpt.com)                                                                                                                 |
| Sparse matrix API                      | [https://docs.rs/clarabel/0.11.1/clarabel/algebra/struct.CscMatrix.html](https://docs.rs/clarabel/0.11.1/clarabel/algebra/struct.CscMatrix.html?utm_source=chatgpt.com)                                                           |
| Solver settings                        | [https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/struct.DefaultSettings.html](https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/struct.DefaultSettings.html?utm_source=chatgpt.com) |
| Linear-solver backends                 | [https://clarabel.org/stable/user_guide_linsolvers/](https://clarabel.org/stable/user_guide_linsolvers/?utm_source=chatgpt.com)                                                                                                   |
| Chordal decomposition                  | [https://clarabel.org/stable/user_guide_chordal/](https://clarabel.org/stable/user_guide_chordal/?utm_source=chatgpt.com)                                                                                                         |
| In-place data updates                  | [https://clarabel.org/stable/user_guide_data_updating/](https://clarabel.org/stable/user_guide_data_updating/?utm_source=chatgpt.com)                                                                                             |
| Solver diagnostics and statuses        | [https://docs.rs/clarabel/0.11.1/clarabel/solver/enum.SolverStatus.html](https://docs.rs/clarabel/0.11.1/clarabel/solver/enum.SolverStatus.html?utm_source=chatgpt.com)                                                           |
| Rust extension traits                  | [https://docs.rs/clarabel/0.11.1/clarabel/solver/traits/](https://docs.rs/clarabel/0.11.1/clarabel/solver/traits/?utm_source=chatgpt.com)                                                                                         |
| Cargo feature definitions              | [https://docs.rs/crate/clarabel/0.11.1/features](https://docs.rs/crate/clarabel/0.11.1/features?utm_source=chatgpt.com)                                                                                                           |
| Release history                        | [https://github.com/oxfordcontrol/Clarabel.rs/releases](https://github.com/oxfordcontrol/Clarabel.rs/releases?utm_source=chatgpt.com)                                                                                             |
| Rust modeling adapter                  | [https://docs.rs/good_lp/latest/good_lp/solvers/clarabel/](https://docs.rs/good_lp/latest/good_lp/solvers/clarabel/?utm_source=chatgpt.com)                                                                                       |
| Upstream benchmark suite               | [https://github.com/oxfordcontrol/ClarabelBenchmarks](https://github.com/oxfordcontrol/ClarabelBenchmarks?utm_source=chatgpt.com)                                                                                                 |

Overall assessment: Clarabel.rs is a technically sophisticated, broad-coverage continuous convex optimization engine whose strengths arise from the interaction of its conic expressiveness, native quadratic objectives, sparse direct numerical methods, infeasibility framework, and trait-based Rust design. Its principal boundaries are discrete and general nonconvex optimization, GPU solving, and the absence of released warm-start and general differentiable-solver interfaces.

# Clarabel.rs — Advanced Functionality Addendum

Research date: November 10, 2026 | Target: Clarabel.rs 0.11.1 | Scope: Additional Rust capabilities, implementation details, numerical behavior, and limitations omitted or underexplained in the original overview

This addendum supplements the original Clarabel.rs reference rather than replacing it. It concentrates on capabilities that affect solver extensibility, repeated execution, numerical reliability, performance instrumentation, and integration into larger Rust systems.

The analysis is based on the published Rust API, official documentation, and the version-tagged v0.11.1 upstream source. Where relevant, it distinguishes documented public behavior from implementation details that could change in later releases.

## 1. Standalone QDLDL sparse factorization

One of the most significant omissions from the original reference is that Clarabel exposes its QDLDL factorization machinery as a directly usable Rust API, not merely as an internal backend for conic optimization.

The public module `clarabel::qdldl` provides sparse symmetric \\(LDL^\top\\) factorization, linear-system solution, symbolic analysis, numerical updates, and factorization diagnostics.

This makes the crate potentially useful for numerical operations outside a complete Clarabel optimization solve.

### 1.1 Factorization model and public API

For a symmetric matrix \\(A\\), QDLDL computes a permutation-dependent factorization of the form

\\[ \Pi A\Pi^\top=LDL^\top, \\]

where \\(L\\) is unit lower triangular and \\(D\\) is diagonal.

The implementation targets symmetric quasidefinite systems and is especially appropriate for the regularized KKT matrices encountered in convex optimization. It should not be regarded as an unrestricted factorization algorithm for every arbitrary symmetric-indefinite matrix.

The principal public type is `QDLDLFactorisation<T>`, which exposes the permutation and factors alongside methods for updating them.

| Method or field                         | Capability                                                |
| --------------------------------------- | --------------------------------------------------------- |
| `new(A, opts)`                          | Construct a factorization from a sparse matrix            |
| `solve(&mut b)`                         | Solve a linear system in place using existing factors     |
| `update_values(indices, values)`        | Replace selected stored matrix coefficients               |
| `scale_values(indices, scale)`          | Multiply selected coefficients by a scalar                |
| `offset_values(indices, offset, signs)` | Apply signed offsets to selected coefficients             |
| `refactor()`                            | Recompute numerical factors after modifying matrix values |
| `nnzA()`                                | Inspect matrix nonzero count                              |
| `nnzL()`                                | Inspect sparse-factor nonzero count                       |
| `positive_inertia()`                    | Report positive inertia associated with the factorization |
| `regularize_count()`                    | Inspect how many regularization corrections were applied  |
| `L`, `D`, `Dinv`, `perm`                | Access factorization data directly                        |

The numerical update methods act on the stored matrix coefficients, not on an arbitrary newly sized matrix. The central capability is preserving a compatible sparsity structure while recomputing numerical factors.

Consequently, QDLDL can avoid repeating some symbolic work when solving successive systems with the same structure.

Primary source: [https://docs.rs/clarabel/0.11.1/clarabel/qdldl/struct.QDLDLFactorisation.html](https://docs.rs/clarabel/0.11.1/clarabel/qdldl/struct.QDLDLFactorisation.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 1.2 Symbolic-only factorization

`QDLDLSettings<T>` exposes a `logical` option that performs symbolic factorization without producing the numerical factors needed for a solve.

The associated settings include:

| Setting             | Functionality                                                           |
| ------------------- | ----------------------------------------------------------------------- |
| `logical`           | Request symbolic-only factorization                                     |
| `perm`              | Supply an explicit permutation instead of relying on automatic ordering |
| `amd_dense_scale`   | Control the approximate-minimum-degree ordering heuristic               |
| `Dsigns`            | Provide expected signs of diagonal factors                              |
| `regularize_enable` | Enable numerical regularization                                         |
| `regularize_eps`    | Configure the regularization threshold                                  |
| `regularize_delta`  | Configure the regularization adjustment                                 |

This exposes a useful distinction between analyzing the structure of a sparse factorization and performing a numerical factorization.

Symbolic analysis identifies the factor's sparsity and storage requirements, while subsequent numerical factorizations can reuse that structure if the pattern remains unchanged.

For example, `nnzL()` can be used to assess factorization fill-in. A symbolic analysis with a supplied permutation can therefore provide information about the cost of different sparse ordering choices without requiring a full optimization solve.

The public `solve()` method must not be used on a logical-only factorization until valid numerical factors are available.

Primary source: [https://docs.rs/clarabel/0.11.1/clarabel/qdldl/struct.QDLDLSettings.html](https://docs.rs/clarabel/0.11.1/clarabel/qdldl/struct.QDLDLSettings.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 1.3 Numerical updates and factor reuse

QDLDL separates changing matrix coefficients from computing new factors.

An illustrative sequence is:

```
use clarabel::qdldl::{
    QDLDLFactorisation,
    QDLDLSettings,
};

// A is a compatible symmetric sparse matrix.
let mut factor =
    QDLDLFactorisation::new(&a, Some(QDLDLSettings::default()))?;

// Modify a coefficient using its stored numerical index.
factor.update_values(&[index], &[new_value]);

// Recompute factors while retaining the structural analysis.
factor.refactor()?;

// Solve with the updated factorization.
factor.solve(&mut rhs);
```

This sketch assumes `a`, `index`, `new_value`, and `rhs` have already been constructed correctly. It is not an independently executable example.

The distinction between the operations is important:

- `update_values()` changes the stored numerical representation.
- `refactor()` updates the factorization.
- `solve()` applies the factorization to a right-hand side without modifying the matrix.

For a sequence of systems with fixed structure and changing coefficients, this can avoid custom sparse-factorization management.

`regularize_count()` and `positive_inertia()` provide additional diagnostics. Inertia should be interpreted with awareness that numerical regularization can alter the matrix actually factored; it is not necessarily an exact spectral characterization of the unregularized input.

Architectural boundary: This is an independently usable sparse numerical component. It does not expose Clarabel's full conic solver iteration machinery or provide a general-purpose GPU factorization interface.

## 2. Dynamically updating solver settings

Clarabel permits changes to selected solver configuration parameters without reconstructing the solver instance.

The relevant methods are implemented on the generic `Solver` type and are therefore also available through `DefaultSolver<T>`:

```
pub fn settings(&self) -> &SE;

pub fn update_settings(
    &mut self,
    settings: SE,
) -> Result<(), SettingsError>;
```

This extends the previously documented model-data update capability. A caller can change how a problem is solved independently of changing the problem's coefficients.

### 2.1 Configurations that can change

The v0.11.1 implementation distinguishes settings that affect the current iteration process from settings that determine structural preparation or solver construction.

| Configuration family                          | Runtime modification |
| --------------------------------------------- | -------------------- |
| Maximum iterations and time limit             | Supported            |
| Optimality tolerances                         | Supported            |
| Feasibility and infeasibility tolerances      | Supported            |
| Reduced-accuracy tolerances                   | Supported            |
| Verbose output                                | Supported            |
| Interior-point step controls                  | Supported            |
| Static and dynamic regularization             | Supported            |
| Iterative-refinement controls                 | Supported            |
| Equilibration configuration                   | Immutable            |
| KKT solver backend                            | Immutable            |
| KKT thread-count configuration                | Immutable            |
| Presolve and structural-zero-dropping options | Immutable            |
| Chordal decomposition options                 | Immutable            |
| Pardiso backend parameters                    | Immutable            |

A typical use is to retain an optimization problem while changing its termination criteria:

```
let mut settings = solver.settings().clone();

settings.max_iter = 500;
settings.tol_feas = 1e-7;
settings.tol_gap_abs = 1e-7;
settings.tol_gap_rel = 1e-7;
settings.verbose = false;

solver.update_settings(settings)?;
```

`update_settings()` validates the supplied configuration against the current settings. Attempting to change an immutable setting returns `SettingsError::ImmutableSetting`.

The implementation validates the new configuration before assigning it, unlike the potentially partial changes discussed later for problem-data updates.

### 2.2 Why some settings are immutable

The distinction is related to solver lifecycle.

Changing an iteration limit, tolerance, or regularization parameter generally affects subsequent iterations without requiring the problem structure to be reconstructed.

Changing the linear solver, equilibrated representation, chordal decomposition, or presolve configuration can invalidate previously constructed numerical components.

Rather than implicitly rebuilding all those components, the public API rejects such modifications.

This makes the runtime configuration interface especially useful for changing accuracy or numerical stabilization policies across solves while preserving a consistent structural solver configuration.

### 2.3 Validation does not establish numerical safety

`DefaultSettings::validate()` checks selected structural and enumerated settings, including supported linear-solver names. The upstream source explicitly states that it does not comprehensively check the numerical validity of every setting.

For example, successfully constructing a settings object is not proof that an arbitrary combination of tolerances, step fractions, and regularization coefficients is numerically sensible.

The Rust settings builder and `update_settings()` should therefore be understood as configuration-management mechanisms, not as guarantees of stable numerical execution.

Primary sources:[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/core/solver.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/core/solver.rs?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/settings.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/settings.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



## 3. Fine-grained problem-data updates and solver reuse

The earlier reference identified `update_P`, `update_q`, `update_A`, `update_b`, and `update_data`. Their implementations expose additional flexibility—and some important correctness constraints.

### 3.1 Supported input representations

Clarabel defines two relevant Rust traits:

- `MatrixProblemDataUpdate<T>` for \\(P\\) and \\(A\\).
- `VectorProblemDataUpdate<T>` for \\(q\\) and \\(b\\).

These support multiple forms of numerical replacement.

| Input representation                | Semantics                                                          |
| ----------------------------------- | ------------------------------------------------------------------ |
| `CscMatrix<T>`                      | Replace matrix values after verifying identical sparsity structure |
| `Vec<T>` or `[T]`                   | Replace the complete stored numerical values                       |
| `indices.iter().zip(values.iter())` | Selectively replace indexed coefficients                           |
| `(Vec<usize>, Vec<T>)`              | Owned representation of selective index/value updates              |
| Empty vector or slice               | No-op update                                                       |

For matrices, the indices address positions in the CSC `nzval` array, not `(row, column)` coordinate pairs. The corresponding `CscMatrix::index_to_coord()` operation can recover coordinates from stored-value positions.

For \\(P\\), these positions refer to the upper-triangular representation of the quadratic objective matrix.

A selective vector update can therefore be expressed as:

```
let indices = vec![1usize, 4usize];
let values = vec![2.5_f64, 3.0_f64];

solver.update_b(&(indices, values))?;
```

Only the specified components of \\(b\\) are replaced.

For large sparse models, this can avoid copying entire coefficient arrays when relatively few parameters change.

### 3.2 Sparsity identity versus numerical equality

Clarabel's matrix replacement rules compare the stored structural representation, including dimensions, `colptr`, and `rowval`.

This creates an important distinction:

\\[ \text{structural entry exists} \\;\not\equiv\\; \text{entry has a nonzero value}. \\]

An explicitly stored zero can later be replaced by a nonzero coefficient without changing the sparsity pattern. A structurally absent entry cannot be introduced through the fixed-pattern update interface.

This gives stored zeros a legitimate role as placeholders for parameterized models.

The `input_sparse_dropzeros` option can remove such placeholders during construction.

The exact v0.11.1 behavior is more nuanced than simply saying the option always disables updating:

- Data updating is rejected if structural zeros were actually removed.
- If the option was enabled but there were no stored zeros to remove, the zero-dropping condition alone does not make updating ineligible.
- Presolve reductions and chordal decomposition are independent reasons updating may still be disallowed.

`is_data_update_allowed()` tests the actual internal eligibility conditions. Despite a reversed description in one upstream code comment, its implementation returns `true` when the eligibility check succeeds.

### 3.3 Updates reuse the original equilibration

A particularly consequential implementation detail is that Clarabel does not automatically recompute its complete equilibration when coefficients change.

Let \\(D\\) and \\(E\\) represent the original variable and constraint scaling matrices, and let \\(c\\) be the objective scaling factor.

When external coefficients are supplied, the update implementation applies the existing transformations:

\\[ \begin{aligned} P\_{\mathrm{internal}}&=cDPD,\\\ q\_{\mathrm{internal}}&=cDq,\\\ A\_{\mathrm{internal}}&=EAD,\\\ b\_{\mathrm{internal}}&=Eb. \end{aligned} \\]

The implementation also invalidates cached unscaled norms when \\(q\\) or \\(b\\) changes.

For closely related scenarios, retaining the original scaling may avoid additional preprocessing and preserve numerical consistency.

However, a sequence of problems whose coefficient magnitudes change substantially may become poorly conditioned under the original equilibration. In those circumstances, constructing a fresh solver can be advantageous even when the old instance remains technically eligible for updates.

This is an important distinction between avoiding repeated setup and maintaining the best numerical conditioning for every new problem.

### 3.4 Update operations are not transactional

`update_data()` executes its four component updates sequentially:

```
self.update_P(P)?;
self.update_q(q)?;
self.update_A(A)?;
self.update_b(b)?;
```

There is no documented all-or-nothing transaction around these operations.

If an update to \\(A\\) fails, for example, earlier updates to \\(P\\) or \\(q\\) may already have succeeded.

There is a second, more subtle issue in selective updates: the implementation processes index/value pairs sequentially. If an invalid index is encountered after valid indices have already been processed, it can return an error after modifying some coefficients.

Furthermore, the tuple-based interface uses Rust's `zip` iterator. Unequal index and value vector lengths can therefore silently truncate the sequence to the shorter length, rather than being rejected by a separate length check.

These behaviors follow from the published v0.11.1 implementation; they are not documented transactional guarantees.

For systems requiring atomic scenario changes, validation and rollback management need to be provided outside the individual update methods.

### 3.5 Numerical factorization after updates

Changing numerical data does not automatically mean that a reusable previous numerical factorization remains valid.

Updates to \\(P\\) and \\(A\\) are propagated into the solver's KKT data structures. During subsequent interior-point iterations, the KKT system must still be numerically factorized as required by the algorithm.

Moreover, cone scaling changes during interior-point iterations, so factorization reuse is not equivalent to solving every new parameterized problem with one permanently fixed numerical factor.

The main savings arise from avoiding compatible structural setup and allocations—not eliminating all subsequent factorization work.

Primary source: [https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/data_updating.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/data_updating.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+3



## 4. Exact presolve behavior and infinity handling

The previous overview described presolve as a general preparation mechanism. The actual v0.11.1 default presolver is considerably narrower.

### 4.1 Presolve primarily removes infinite-bound inequalities

The implementation identifies nonnegative-cone constraint rows with right-hand-side bounds above an infinity threshold.

For a linear inequality represented as

\\[ a_i^\top x\le b_i, \\]

if \\(b_i\\) is considered infinite, the corresponding row can be eliminated.

This entails removing the row from the constraint matrix and right-hand-side vector, then adjusting the nonnegative-cone block dimensions accordingly.

The published implementation does not provide general-purpose LP presolve algorithms such as comprehensive bound propagation, algebraic redundancy elimination, or arbitrary infeasibility detection during preprocessing.

Its preparation is therefore more limited than the term presolve might suggest to readers familiar with highly developed LP/MIP presolvers.

### 4.2 Infinity is a configurable numerical convention

Clarabel exposes:

```
clarabel::INFINITY_DEFAULT
clarabel::get_infinity()
clarabel::set_infinity(value)
clarabel::default_infinity()
```

The default threshold is

\\[ \texttt{INFINITY\\\_DEFAULT}=10^{20}. \\]

This is not IEEE floating-point infinity. It is a large finite sentinel used when interpreting bounds.

The implementation uses a slightly contracted threshold, accounting for floating-point epsilon, when deciding which nonnegative-cone rows to eliminate.

It also caps the initial entries of \\(b\\) at the configured infinity threshold, even when presolve is disabled. This is done in problem-data construction outside the conditional presolver.

Consequently, a finite right-hand-side value larger than the threshold may not retain its exact original numerical meaning.

This is a material modeling distinction for applications whose constraints legitimately contain very large magnitudes.

### 4.3 Global configuration scope

The infinity threshold is stored in module-level atomic state.

`set_infinity()` therefore changes a shared library-level configuration value, not a field belonging to a particular `DefaultSolver`.

A presolver captures the threshold at construction, protecting its own reconstruction behavior from later changes to the module-level setting.

For software constructing multiple solver instances under different numerical conventions, this shared configuration scope is important. It is not equivalent to passing a per-instance infinity setting.

### 4.4 Reconstruction and update eligibility

When presolve eliminates a row, the solver retains enough metadata to reconstruct the original solution dimensions.

For eliminated nonnegative-cone rows, the output reconstruction sets:

- The slack to the captured infinity sentinel.
- The corresponding dual multiplier to zero.

Because removed rows are absent from the internal optimization problem, subsequent coefficient updates cannot simply overwrite them through the ordinary fixed-pattern update interface.

However, enabling presolve does not automatically imply that a reduction actually occurred. If the presolver leaves the structure unchanged, data updating may still be permitted.

Primary sources:[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/presolver.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/presolver.rs?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/problemdata.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/problemdata.rs?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/utils/infbounds.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/utils/infbounds.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+3



## 5. Chordal decomposition: advanced representation and reconstruction semantics

Chordal decomposition was covered in the original reference, but its effect on the internal representation and reported dual solutions deserves greater attention.

### 5.1 When decomposition is attempted

Clarabel does not attempt arbitrary SDP decomposition for every semidefinite cone.

The v0.11.1 implementation first checks whether an eligible PSD cone exists. In particular, the setup path does not attempt chordal decomposition when there are no PSD cones of matrix dimension greater than three.

It then tests whether the semidefinite structure is decomposable. If no decomposition is possible, the solver retains the undecomposed representation.

This distinction has an important implication for repeated solves: enabling `chordal_decomposition_enable` is not, by itself, proof that data updating has become inadmissible. The restriction depends on whether a structural decomposition actually occurred.

### 5.2 Compact and noncompact representations

`chordal_decomposition_compact` controls the assembly strategy for the decomposed problem.

A compact representation can reduce auxiliary variables and linking constraints associated with clique relationships, while alternative assemblies may produce different sparse KKT structures.

The `chordal_decomposition_merge_method` setting determines how candidate cliques are combined:

- `none`: avoid clique merging.
- `parent_child`: merge according to parent-child relationships.
- `clique_graph`: use the clique-graph merging strategy.

These decisions affect the number of PSD cones, the dimensions of their blocks, and the structure of the resulting optimization problem.

No strategy is uniformly optimal: merging reduces some overlap and coordination overhead while increasing the size of individual dense semidefinite blocks.

### 5.3 Reconstructing primal and dual results

The solver postprocessing implementation reverses transformations in the opposite order from construction:

1. Unscale the internal solution.
2. Reverse chordal decomposition, if applied.
3. Reverse presolve, if applied.

The purpose is to recover solution vectors in the dimensions and ordering of the original model.

However, chordal decomposition can change the representation of semidefinite dual variables.

The setting `chordal_decomposition_complete_dual` controls completion of PSD dual information after reversing the decomposition.

A positive-semidefinite completion is not necessarily unique. Consequently, two equivalent formulations—one solved with decomposition and one without—can produce different valid dual matrices.

This has consequences for:

- Comparing solver output across different decomposition settings.
- Interpreting matrix-valued dual multipliers.
- Calculating sensitivities from dual solutions.
- Verifying whether two numerically different results represent equivalent optima.

It is generally more appropriate to assess whether the recovered dual variables satisfy the relevant optimality and feasibility conditions than to require elementwise equality with the dual solution of an undecomposed formulation.

### 5.4 A numerical tradeoff, not an unconditional optimization

Chordal decomposition can greatly reduce the cost of solving SDPs with favorable aggregate sparsity.

However, it may not be advantageous when a PSD matrix has limited exploitable sparsity, decomposition produces large overlapping cliques, or solution recovery becomes a significant part of total execution time.

The most important performance question is whether the reduction in dense PSD algebra and factorization complexity exceeds the added linking and reconstruction overhead.

Primary sources:[https://clarabel.org/stable/user_guide_chordal/](https://clarabel.org/stable/user_guide_chordal/?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/problemdata.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/problemdata.rs?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/solution.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/solution.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://clarabel.org\&sz=32)

Clarabel jl/rs+2



## 6. Internal timing and performance instrumentation

Clarabel provides more detailed execution timing than the `solution.solve_time` field alone suggests.

### 6.1 Hierarchical timers

`DefaultSolver` exposes a public field:

```
pub timers: Option<Timers>
```

The `Timers` implementation provides a hierarchical collection of execution timers, with methods including:

| Method                   | Function                                         |
| ------------------------ | ------------------------------------------------ |
| `start_as_current(name)` | Start a named timer as the active timing context |
| `stop_current()`         | Stop the active timer                            |
| `reset_timer(name)`      | Reset a named timer                              |
| `suspend()` / `resume()` | Suspend and resume timing                        |
| `total_time()`           | Sum elapsed time across root-level timers        |
| `print()`                | Print the hierarchy of recorded timings          |

The solver records setup and solution-related activity using this mechanism.

During construction, identifiable timed phases include `setup`, `presolve`, `equilibration`, and `kktinit`.

This creates a means of distinguishing:

- Structural problem preparation.
- Numerical equilibration.
- KKT-system initialization.
- Interior-point execution.
- Overall solver-associated timing.

The native timer structure is useful for diagnostics, although it is not a complete structured profiling interface for every internal operation.

### 6.2 A v0.11.1 timing limitation

The v0.11.1 implementation contains an important distinction between the total timer collection and the time consumed by an individual `solve()` call.

`DefaultInfo::update()` and `DefaultInfo::finalize()` calculate:

```
self.solve_time = timers.total_time().as_secs_f64();
```

The total includes setup timing recorded during solver construction.

The same `solve_time` value is used to evaluate the `time_limit` termination condition.

Consequently, a solver with a costly construction phase can report a solve time that exceeds the duration of the actual numerical solve. Under a sufficiently small time limit, it may terminate with `MaxTime` even if the current solve itself has consumed less time than that limit.

This is especially consequential when attempting to measure the benefit of repeated execution after data updates.

Upstream pull request #242, opened October 3, 2026, proposes measuring each solve call using its own clock. As of the research date, that change remains unmerged and is not part of the published 0.11.1 release.

For rigorous per-call performance comparisons, an independent monotonic wall-clock measurement around `solve()` is more appropriate than relying solely on the reported `solution.solve_time` in this version.

### 6.3 Build information

The crate-level function `clarabel::buildinfo()` provides build-environment reporting.

When compiled with the `buildinfo` feature, it can report details such as:

- Compiler and Rust toolchain information.
- Build profile and optimization level.
- Target platform.
- Enabled Cargo features.
- CPU and operating-system information.

Without the feature, it prints that build information is unavailable.

This offers a native mechanism for recording relevant build characteristics alongside numerical benchmark results.

Primary sources:[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/timers/timers.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/timers/timers.rs?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/info.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/info.rs?utm_source=chatgpt.com)[https://github.com/oxfordcontrol/Clarabel.rs/pull/242](https://github.com/oxfordcontrol/Clarabel.rs/pull/242?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+3



## 7. JSON serialization and persistence semantics

The original reference correctly identified JSON import/export, but its relationship to solver transformations requires clarification.

### 7.1 What is serialized

With `serde` enabled, `SolverJSONReadWrite<T>` provides `save_to_file()` and `load_from_file()`.

The serialized representation includes:

\\[ (P,q,A,b,\text{cones},\text{settings}). \\]

Importantly, `save_to_file()` obtains these values from the solver's internal problem-data object, rather than preserving an untouched copy of the original caller-supplied formulation.

Before serialization, it reverses the solver's numerical equilibration.

However, it does not generally reverse all structural transformations. For example, if presolve removed constraints or chordal decomposition changed the cone representation, the serialized matrices and cones may reflect the transformed problem.

This makes JSON appropriate for exporting a numerical problem representation, but it is not necessarily a lossless archival format for the original mathematical model.

### 7.2 Settings restoration

The loader supports:

```
DefaultSolver::<f64>::load_from_file(
    &mut file,
    None,
)?;
```

The `None` argument selects settings stored in the JSON file.

Supplying `Some(settings)` instead overrides those stored settings.

The implementation additionally sanitizes infinite `time_limit` values for JSON serialization:

- Positive infinity is converted to `f64::MAX` when saving.
- `f64::MAX` is converted back to infinity when loading.

This convention introduces an edge case: an original finite time limit set exactly to `f64::MAX` is not semantically distinguishable from the serialized infinity sentinel.

### 7.3 Persistence boundary

JSON serialization does not preserve:

- A reusable sparse numerical factorization.
- The current interior-point iterate.
- A valid warm-start state.
- An exact copy of all original model-building inputs.
- All previous solve history and runtime diagnostics.

Loading reconstructs a new solver from the serialized problem data.

Thus, the feature is better understood as problem serialization and reproducibility infrastructure, rather than checkpoint/restart functionality for a numerical computation in progress.

Primary source: [https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/json.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/json.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 8. Sparse matrix utilities, validation, and error handling

### 8.1 Additional `CscMatrix` functionality

Clarabel's sparse matrix representation exposes more than basic construction.

| Capability                                   | Relevant API                                                 |
| -------------------------------------------- | ------------------------------------------------------------ |
| Coordinate-format construction               | `new_from_triplets()`                                        |
| Canonicalization and duplicate consolidation | `canonicalize()`                                             |
| Structural validation                        | `check_format()`                                             |
| Sparsity comparison                          | `is_equal_sparsity()`, `check_equal_sparsity()`              |
| Entry inspection and modification            | `get_entry()`, `set_entry()`                                 |
| Stored-index conversion                      | `index_to_coord()`                                           |
| Symmetric and triangular views               | `sym()`, `sym_up()`, `sym_lo()`, `to_triu()`                 |
| Matrix assembly                              | `hcat()`, `vcat()`, `blockdiag()` through `BlockConcatenate` |
| Sparse storage management                    | `spalloc()`, `dropzeros()`, `nnz()`                          |

A noteworthy distinction is between ordinary coordinate construction and direct CSC construction.

`new_from_triplets()` accepts unsorted coordinate data and combines repeated coordinates by addition.

`CscMatrix::new()`, by comparison, performs only basic dimensional checks. It assumes the caller has supplied valid column pointers and row-index ordering.

The solver does not automatically perform comprehensive sparse-format validation on every supplied matrix.

### 8.2 Canonicalization versus structural modification

`canonicalize()` sorts entries within columns and combines duplicates. `dropzeros()` removes explicitly stored numerical zeros.

These operations have different implications.

Canonicalization is useful for constructing a consistent sparse representation, but it can change the positions used by indexed numerical updates.

Removing stored zeros changes the sparsity pattern and can prevent later changes to coefficients that were initially zero.

Similarly, `set_entry()` can introduce a new structural entry if one is absent. That is an operation on a `CscMatrix`, not evidence that an existing solver permits its KKT sparsity structure to grow dynamically.

For parameterized optimization, stable coefficient indexing depends on preserving a consistent canonical CSC representation.

### 8.3 Structured error types

Clarabel distinguishes errors associated with sparse matrices, solver construction, data updates, and numerical execution.

`SparseFormatError` includes:

| Variant                 | Meaning                                              |
| ----------------------- | ---------------------------------------------------- |
| `IncompatibleDimension` | Dimensions or storage lengths are incompatible       |
| `BadRowOrdering`        | Row indices are not ordered correctly within columns |
| `BadRowval`             | Row index exceeds matrix dimensions                  |
| `BadColptr`             | Column pointer array is malformed                    |
| `SparsityMismatch`      | Two sparse structures do not match                   |

`DataUpdateError` includes:

| Variant                        | Meaning                                                           |
| ------------------------------ | ----------------------------------------------------------------- |
| `PresolveIsActive`             | Internal presolve reduction prevents updates                      |
| `DroppedStructuralZeros`       | Structural zeros were removed                                     |
| `ChordalDecompositionIsActive` | Chordal decomposition changed the internal representation         |
| `BadFormat(SparseFormatError)` | Numerical replacement violates format or dimensional requirements |

`SolverError` represents construction-level and I/O errors, including `BadInputData`, `SettingsError`, `IoError`, and `JsonError`.

These are separate from `SolverStatus`, which reports the outcome of numerical execution.

This distinction matters because a solver can be successfully constructed and still terminate with a numerical failure, while malformed matrix construction may fail or panic before an optimization solve begins.

### 8.4 Numerical assumptions are not automatically proved

Clarabel's objective requires a positive-semidefinite \\(P\\).

The documented dimension checks do not constitute a complete mathematical validation of positive semidefiniteness.

Similarly, accepting a sparse matrix representation does not prove that the associated conic formulation correctly encodes the intended optimization problem.

A modeling layer may provide mathematical validation and conversion logic, but those responsibilities should not be attributed to the low-level solver constructor.

Primary sources:[https://docs.rs/clarabel/0.11.1/clarabel/algebra/struct.CscMatrix.html](https://docs.rs/clarabel/0.11.1/clarabel/algebra/struct.CscMatrix.html?utm_source=chatgpt.com)[https://docs.rs/clarabel/0.11.1/clarabel/algebra/enum.SparseFormatError.html](https://docs.rs/clarabel/0.11.1/clarabel/algebra/enum.SparseFormatError.html?utm_source=chatgpt.com)[https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/enum.DataUpdateError.html](https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/enum.DataUpdateError.html?utm_source=chatgpt.com)[https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/enum.SolverError.html](https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/enum.SolverError.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



## 9. Further numerical controls and integration interfaces

### 9.1 Nonsymmetric-cone scaling transitions

Clarabel's interior-point implementation distinguishes `PrimalDual` and `Dual` scaling strategies.

The settings:

- `min_switch_step_length`
- `min_terminate_step_length`
- `linesearch_backtrack_step`

control aspects of scaling-strategy transitions and line search.

For nonsymmetric cones, Clarabel may switch away from primal-dual scaling when its permitted step becomes too small.

If insufficient progress persists under the alternative scaling strategy, the solver can terminate with `InsufficientProgress`.

This is relevant to exponential- and power-cone optimization, where barrier geometry and step selection can be challenging.

These settings are not user-facing choices between completely different optimization algorithms. They control behavior within the existing interior-point method.

The scaling strategy itself is exposed through the internal `ScalingStrategy` enum, while the relevant thresholds are configurable.

Changing these thresholds may affect convergence behavior and robustness, but their numerical interactions are not covered by comprehensive settings validation.

Primary sources:[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/core/solver.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/core/solver.rs?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/settings.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/settings.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 9.2 Low-level Pardiso configuration

When built with the optional Pardiso features, Clarabel exposes:

```
pardiso_iparm: [i32; 64]
pardiso_verbose: bool
```

The `pardiso_iparm` array allows selected Pardiso parameters to be overridden.

The default array uses `i32::MIN` as a sentinel meaning that a corresponding option has not been explicitly supplied.

When the caller provides custom values, Clarabel sets `iparm[0]` to indicate that user-supplied parameters are active.

Important details include:

- The array uses zero-based indexing.
- Parameter interpretation depends on the underlying Pardiso implementation.
- These settings are immutable after constructing the solver.
- The upstream validation function does not comprehensively check the validity of supplied Pardiso parameters.

Invalid combinations can produce numerical errors or failures within the external sparse-direct solver.

This is an advanced extension point for sparse factorization tuning, rather than an alternative mathematical optimization method.

Primary source: [https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/settings.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/implementations/default/settings.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 9.3 Precision-specific tolerance behavior

Clarabel's generic `FloatT` architecture supports native `f32` and `f64` execution, with broader numerical-type possibilities in configurations that do not require the BLAS/LAPACK-backed SDP machinery.

However, the documented default tolerances are not automatically adapted to every numerical type.

For instance, the default feasibility tolerance is \\(10^{-8}\\), while machine epsilon for `f32` is approximately \\(1.19\times10^{-7}\\).

This does not imply that every single-precision optimization problem fails at the default tolerance. Achievable residuals depend on scaling, conditioning and the particular solution.

It does mean that default accuracy expectations can be inappropriate for certain single-precision problems.

The same consideration extends to:

- Iterative-refinement tolerances.
- Dynamic regularization thresholds.
- Minimum step lengths.
- Accuracy-dependent termination behavior.

The `FloatT` abstraction makes precision configurable at the type level, but it does not eliminate finite-precision numerical limitations.

Primary sources:[https://docs.rs/clarabel/0.11.1/clarabel/algebra/trait.FloatT.html](https://docs.rs/clarabel/0.11.1/clarabel/algebra/trait.FloatT.html?utm_source=chatgpt.com)[https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/struct.DefaultSettings.html](https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/struct.DefaultSettings.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 9.4 C-compatible callback and FFI structures

The original reference covers Rust closure-based termination callbacks. The generic solver also supports:

```
set_termination_callback_c(callback, data_ptr)
```

This interface accepts a C-compatible callback function and an opaque pointer to user-managed data.

The callback receives an FFI-compatible representation of solver information and can request termination.

The released crate additionally exposes:

- `DefaultSettingsFFI<T>`
- `DefaultInfoFFI<T>`
- C-compatible representations of certain solver options and callback data.

These are separate from the ordinary Rust types.

This matters when embedding Clarabel in software using a C ABI, including libraries that wrap Rust from C, C++, Julia or other languages.

The FFI representation provides explicit cross-language data layouts, but responsibility for pointer validity, callback lifetimes, ownership and correct foreign-function declarations remains with the integration.

The presence of FFI types also does not imply that every internal Rust trait or numerical component is accessible through the C interface.

Primary sources:[https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/ffi/](https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/ffi/?utm_source=chatgpt.com)[https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/core/solver.rs](https://raw.githubusercontent.com/oxfordcontrol/Clarabel.rs/v0.11.1/src/solver/core/solver.rs?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 9.5 Output capture and embedding into services

`DefaultSolver` implements `ConfigurablePrintTarget`.

Beyond printing to stdout, this includes:

| Method                    | Functionality                                      |
| ------------------------- | -------------------------------------------------- |
| `print_to_file(file)`     | Redirect solver output to an owned file            |
| `print_to_stream(stream)` | Use a caller-supplied `Write + Send + Sync` stream |
| `print_to_sink()`         | Discard printed output                             |
| `print_to_buffer()`       | Capture output in an internal buffer               |
| `get_print_buffer()`      | Retrieve buffered output                           |

This is useful for separating numerical solver reporting from the rest of an application's logging system.

The custom stream interface is especially valuable because it uses Rust's standard `Write` abstraction rather than requiring an application to intercept process-level stdout.

The print-capture interface complements structured access to `DefaultInfo` and `DefaultSolution`; it does not replace those fields.

Primary source: [https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/type.DefaultSolver.html](https://docs.rs/clarabel/0.11.1/clarabel/solver/implementations/default/type.DefaultSolver.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 10. Additional version-specific limitations and unreleased developments

Several upstream developments are relevant to understanding where the Rust library may evolve. These are not released capabilities of v0.11.1.

### 10.1 A `no-default-features` compilation issue

The published `Cargo.toml` makes `serde` a default feature, while `serde_json` is an optional dependency.

However, the v0.11.1 `SolverError` enum unconditionally references `serde_json::Error`.

This creates a feature-gating problem when attempting to build the crate with:

```
clarabel = {
    version = "0.11.1",
    default-features = false
}
```

Upstream issue #244 and open pull request #245 identify that this configuration fails to compile, because the optional dependency is absent while its error type remains referenced.

The proposed correction places the error variant behind the `serde` feature.

This is particularly relevant to attempts to minimize dependencies or construct lightweight pure-Rust builds.

The regular default-feature configuration is not subject to this particular missing-dependency problem.

Source: [https://github.com/oxfordcontrol/Clarabel.rs/pull/245](https://github.com/oxfordcontrol/Clarabel.rs/pull/245?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 10.2 Relevant unreleased work

| Proposed functionality                                            | Upstream reference                                                                                                                    | Released in 0.11.1? |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| Warm starting from previous numerical state                       | [https://github.com/oxfordcontrol/Clarabel.rs/pull/219](https://github.com/oxfordcontrol/Clarabel.rs/pull/219?utm_source=chatgpt.com) | No                  |
| Independent timing of each solve call                             | [https://github.com/oxfordcontrol/Clarabel.rs/pull/242](https://github.com/oxfordcontrol/Clarabel.rs/pull/242?utm_source=chatgpt.com) | No                  |
| Native arbitrary-precision and symbolic SDP types                 | [https://github.com/oxfordcontrol/Clarabel.rs/pull/224](https://github.com/oxfordcontrol/Clarabel.rs/pull/224?utm_source=chatgpt.com) | No                  |
| Reusing a precomputed numerical-refactorization schedule in QDLDL | [https://github.com/oxfordcontrol/Clarabel.rs/pull/230](https://github.com/oxfordcontrol/Clarabel.rs/pull/230?utm_source=chatgpt.com) | No                  |
| Binary problem serialization                                      | [https://github.com/oxfordcontrol/Clarabel.rs/pull/207](https://github.com/oxfordcontrol/Clarabel.rs/pull/207?utm_source=chatgpt.com) | No                  |
| Fix for builds without default features                           | [https://github.com/oxfordcontrol/Clarabel.rs/pull/245](https://github.com/oxfordcontrol/Clarabel.rs/pull/245?utm_source=chatgpt.com) | No                  |

These proposals are all visible in the upstream open pull-request inventory as of November 10, 2026. Their presence does not establish that they will be merged, that their interfaces are stable, or that their current implementation is suitable for production use.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Two proposed features are especially consequential from an architectural standpoint.

Warm starting could introduce a genuine new numerical state-reuse mechanism, complementing the existing fixed-pattern coefficient updates.

Scheduled QDLDL refactorization could reduce overhead in repeated sparse numerical factorizations where the relevant factorization operations can be precomputed.

Neither capability should be attributed to a stable 0.11.1 dependency.

## 11. Consolidated architectural implications

The advanced mechanisms documented in this addendum reveal several additional roles for Clarabel's native Rust implementation.

| Mechanism                              | General capability enabled                                                 | Important boundary                                                                |
| -------------------------------------- | -------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Standalone QDLDL                       | Sparse quasidefinite linear-system solution and reusable symbolic analysis | Not a universal symmetric-indefinite factorizer                                   |
| Runtime settings updates               | Change numerical accuracy and stopping conditions without reconstruction   | Structural solver configuration remains fixed                                     |
| Indexed parameter updates              | Efficient replacement of selected optimization coefficients                | Fixed CSC structure and nontransactional update behavior                          |
| Original equilibration reuse           | Reduce repeated numerical preprocessing                                    | Can become poorly suited to substantially changed coefficients                    |
| Internal timers                        | Examine setup and solver phase costs                                       | v0.11.1's reported solve time includes setup                                      |
| Infinity-bound presolve                | Remove effectively unbounded inequalities                                  | Uses a configurable finite sentinel and shared module-level state                 |
| Chordal dual completion                | Recover SDP results after sparse decomposition                             | Recovered dual matrices need not be unique                                        |
| Structured errors and CSC utilities    | Validate sparse inputs and distinguish failure causes                      | Some constructors can panic; mathematical convexity is not automatically verified |
| FFI representations and callback hooks | Integrate solver monitoring across language boundaries                     | Foreign pointer and ABI correctness remain caller responsibilities                |

The most important distinction is that Clarabel provides reusable numerical infrastructure beyond its high-level conic solver interface, but reuse occurs at several different levels with different guarantees.

Sparse symbolic factorization can be reused without retaining previous optimization iterates. Numerical coefficient updates can reuse a solver's structural setup without reusing its previous solution as an interior-point starting point. Runtime settings can change termination and stabilization behavior without changing the mathematical model. SDP reconstruction can restore original dimensions without guaranteeing unique dual values.

Recognizing these differences makes the library considerably more useful as a technical building block—and avoids attributing stronger guarantees to its interfaces than the released implementation provides.

The additions above complete the significant gaps identified in the earlier assessment without changing the original overview's description of Clarabel's fundamental convex optimization capabilities.