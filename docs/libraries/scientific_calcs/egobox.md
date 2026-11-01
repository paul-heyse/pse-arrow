# EGObox — Comprehensive Rust Library Capability Reference

Research date: October 28, 2026 | Scope: EGObox 1.0.0 and its four Rust crates

## 1. Executive orientation

EGObox is a Rust-based numerical optimization and surrogate-modeling toolkit specializing in gradient-free Bayesian optimization of expensive black-box functions. Its principal algorithm, Efficient Global Optimization (EGO), uses probabilistic surrogate models to identify promising evaluation points while balancing exploitation of known good solutions against exploration of uncertain regions.

Unlike a standalone optimizer, EGObox provides a composable stack spanning experimental design, Gaussian process regression, mixtures of local surrogate models, and sequential optimization.

### 1.1 Crate architecture and versions

The latest upstream EGObox release is 1.0.0 (October 12, 2026). Its Rust crates have independently versioned releases:

| Crate        | Current version | Primary responsibility                                                                                        |
| ------------ | --------------- | ------------------------------------------------------------------------------------------------------------- |
| `egobox-doe` | 0.35.2          | Design of experiments, space-filling sampling                                                                 |
| `egobox-gp`  | 0.37.1          | Gaussian process/Kriging regression, sparse GP, dimension reduction                                           |
| `egobox-moe` | 0.36.2          | Mixture-of-experts surrogate modeling, adaptive local models                                                  |
| `egobox-ego` | 0.42.0          | Bayesian optimization, constraints, mixed variables, trust regions, parallel and multi-objective optimization |

The project also publishes the Python package `egobox` 1.0.0 and the separate Rust CLI crate `egobox-gpx` 0.6.0. The 1.0.0 release introduces experimental multi-objective optimization while stabilizing the Python API; the Rust changes for that release are additive.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Primary repository: `https://github.com/relf/egobox`

Release history: `https://github.com/relf/egobox/releases`

### 1.2 Conceptual architecture

`egobox-ego` — Optimization orchestration

Acquisition functions · Constraints · Batches · Trust regions · Mixed variables · Multi-objective strategies

`egobox-moe` — Composite probabilistic surrogates

Gaussian-mixture clustering · Expert selection · Hard/smooth recombination · Cross-validation

`egobox-gp` — Gaussian process regression

Posterior predictions · Uncertainty · Kernel families · KPLS · Sparse FITC/VFE

`egobox-doe` — Experimental design

Latin hypercubes · Factorial sampling · Random sampling · Reproducibility

Conceptual dependency and capability layering; not an exact crate dependency graph.

The primary capability families are:

1. Sampling and surrogate construction: Structured sampling, probabilistic regression, uncertainty quantification, model selection, local experts, and sparse approximations.
2. Optimization algorithms: Expected-improvement acquisition, constrained optimization, trust-region refinement, cooperative high-dimensional search, and parallel evaluations.
3. Operational integration: Mixed-variable domains, external evaluation control, persisted optimization state, restart mechanisms, and interfaces to Rust numerical libraries.

Architectural significance: EGObox makes the surrogate model, experimental design, and search algorithm independently useful. An application can employ GP regression for uncertainty-aware prediction without running EGO, use DOE for simulation coverage without surrogate fitting, or employ the complete stack for expensive numerical optimization.

## 2. `egobox-doe` — Design of experiments

Crate 0.35.2 | Core APIs: `SamplingMethod`, `Lhs`, `LhsKind`, `FullFactorial`, `Random`

This crate generates experimental designs over bounded numerical domains. It is independently useful for black-box sampling, surrogate training, sensitivity investigations, numerical probes, and initialization of optimization algorithms.

### 2.1 Sampling strategies

| Strategy                   | Mechanism                                           | Consequential distinction                                              |
| -------------------------- | --------------------------------------------------- | ---------------------------------------------------------------------- |
| `Random`                   | Independent random samples within bounds            | Simple baseline; no stratification guarantee                           |
| `FullFactorial`            | Combinations of coordinate levels                   | Structured coverage; rapidly becomes expensive as dimensionality grows |
| `LhsKind::Classic`         | One randomized location per marginal stratum        | Stratifies each dimension without requiring a full Cartesian grid      |
| `LhsKind::Centered`        | Centers points within their strata                  | Removes within-stratum positional randomness                           |
| `LhsKind::Maximin`         | Maximizes separation between design points          | Discourages clustering and improves geometric coverage                 |
| `LhsKind::CenteredMaximin` | Combines centered strata with maximin separation    | Deterministic stratum positions with optimized permutations            |
| `LhsKind::Optimized`       | Enhanced Stochastic Evolutionary (ESE) optimization | Improves spatial design quality at additional computational cost       |

The optimized LHS method implements the ESE approach described by Jin, Chen, and Sudjianto, rather than simply generating repeated random hypercubes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 2.2 Interface and implementation capabilities

Design-space representation. The canonical input is an `ndarray` matrix of shape `(n_dimensions, 2)`, with lower and upper bounds per variable. Generated samples have shape `(n_samples, n_dimensions)`.

Normalized sampling. `SamplingMethod::normalized_sample()` generates points in the unit hypercube, while `sample()` maps them to the declared physical bounds. This makes the sampling logic reusable independently of variable scaling.

Random-generator injection. `Lhs::new_with_rng()` and `with_rng()` permit caller-supplied RNG implementations. The default uses Xoshiro; seeded RNGs support reproducible experimental designs.

Generic numeric and sampling interfaces. `Lhs<F, R>` is generic over floating-point values and RNG implementations. The `SamplingMethod` trait provides a common sampling abstraction, allowing callers to substitute strategies without changing the broader data-processing interface.

Parallel design optimization. The implementation uses Rayon; optimized sampling may therefore benefit from parallel CPU work. This does not imply that evaluation of the external objective function is automatically parallelized.

Mixed-domain boundary. The fundamental DOE crate describes continuous bounded coordinates. Mixed integer/categorical sampling is handled through the decorators and domain specifications in `egobox-moe`, rather than by treating discrete variables as native DOE dimensions.

### 2.3 Architectural implications

DOE supplies an independent, reusable sample-location generator, not a surrogate or evaluator.

Its most useful interactions are with GP/MoE training and EGO initialization. Sampling can also be separated from evaluation: the generated matrix can be dispatched to arbitrary numerical software and the results subsequently returned to another component.

An important limitation is that Latin hypercube stratification is marginal. It does not guarantee good coverage of every multidimensional projection, capture complicated feasible regions, or adapt to observed objective values. Its geometric properties can also be weakened when points are rounded or projected into discrete domains.

Sources:

`https://docs.rs/egobox-doe/latest/egobox_doe/`

`https://docs.rs/egobox-doe/latest/egobox_doe/enum.LhsKind.html`

`https://docs.rs/egobox-doe/latest/egobox_doe/struct.Lhs.html`

## 3. `egobox-gp` — Gaussian process and Kriging regression

Crate 0.37.1 | Core APIs: `GaussianProcess`, `GpParams`, `SparseGaussianProcess`, `SgpParams`

`egobox-gp` provides both full Gaussian process regression and sparse approximations. It is a substantial standalone probabilistic modeling library, not merely an internal dependency of the optimizer.

Its principal strengths include configurable covariance models, regression trends, hyperparameter fitting, uncertainty estimation, prediction derivatives, conditional sampling, partial-least-squares dimensionality reduction, and sparse inference.

### 3.1 Mathematical model and kernels

The full GP uses the universal-Kriging form:

\\[ Y(x)=m(x)+Z(x) \\]

where \\(m(x)\\) is a regression trend and \\(Z(x)\\) is a correlated Gaussian random process.

The covariance is parameterized by a kernel and learned parameters, commonly represented as

\\[ \operatorname{Cov}(Z(x),Z(x'))=\sigma^2 R\_\theta(x,x'). \\]

The library exposes three polynomial trend families and four kernel families.

| Component | Supported families          | Modeling significance                                                        |
| --------- | --------------------------- | ---------------------------------------------------------------------------- |
| Trend     | Constant, linear, quadratic | Captures systematic low-order structure separately from correlated residuals |
| Kernel    | Squared exponential         | Very smooth fitted functions                                                 |
| Kernel    | Absolute exponential        | Rougher, less smooth response structure                                      |
| Kernel    | Matérn 3/2                  | Intermediate regularity; less restrictive than squared exponential           |
| Kernel    | Matérn 5/2                  | Smoother than Matérn 3/2, but less restrictive than squared exponential      |

`correlation_models::CorrelationModel` and `mean_models::RegressionModel` provide the underlying modeling abstractions. `GpParams` is generic over the mean and correlation model types; `Kriging` denotes the constant-mean, squared-exponential specialization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Why this matters: separating trend from covariance enables the model to represent a broad deterministic response with a simpler stochastic residual. A quadratic trend, for example, may reduce the burden on the kernel when the response contains a dominant low-order polynomial component.

### 3.2 Prediction and uncertainty capabilities

The full GP implements more than point prediction.

| Capability                   | API handles                                         | Uses                                                                 |
| ---------------------------- | --------------------------------------------------- | -------------------------------------------------------------------- |
| Posterior mean               | `predict`                                           | Function approximation and surrogate evaluation                      |
| Marginal predictive variance | `predict_var`, `predict_valvar`                     | Uncertainty-aware decisions and acquisition functions                |
| Mean gradients               | `predict_gradients`                                 | Local sensitivity and gradient-based optimization of surrogates      |
| Higher derivatives           | `predict_kth_derivatives`                           | Curvature and higher-order local analysis                            |
| Variance gradients           | `predict_var_gradients`, `predict_valvar_gradients` | Differentiating uncertainty-aware objectives                         |
| Conditional sampling         | `sample`, `sample_chol`, `sample_eig`               | Sampling possible response functions under the fitted model          |
| Joint posterior covariance   | `predict_covariance`                                | Correlations between candidate points, relevant to batch acquisition |

The model also exposes fitted hyperparameters, process variance, likelihood, and input/output dimensionality.

Two especially consequential features deserve distinction.

Derivative evaluation: EGObox can differentiate the fitted surrogate even when the original objective function has no accessible derivatives. These derivatives describe the approximation, not necessarily the true black-box function.

Joint posterior covariance: Individual predictive variances do not describe how uncertain predictions co-vary. Covariance between proposed points is necessary for more faithful joint batch reasoning and underlies the multi-objective qEHVI functionality introduced in the 1.0.0 release.

The upstream 1.0.0 release explicitly identifies `GaussianProcess::predict_covariance()` as a newly exposed capability used by the multi-objective machinery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 3.3 Hyperparameter training and numerical controls

`GpParams` provides a relatively detailed optimization interface:

| Control                  | API            | Function                                                                |
| ------------------------ | -------------- | ----------------------------------------------------------------------- |
| Starting hyperparameters | `theta_init`   | Initial length-scale/correlation parameters                             |
| Search bounds            | `theta_bounds` | Constrains the hyperparameter search                                    |
| Tuning mode              | `theta_tuning` | Fixed, fully optimized, or partially optimized components               |
| Multistart               | `n_start`      | Repeated hyperparameter optimization from different starting conditions |
| Evaluation budget        | `max_eval`     | Caps likelihood evaluations per optimization                            |
| Numerical nugget         | `nugget`       | Adds regularization to stabilize covariance calculations                |
| Dimensionality           | `kpls_dim`     | Uses PLS-reduced coordinates                                            |

The `ThetaTuning` variants are especially useful:

`Fixed` uses supplied parameters without optimizing them; `Full` optimizes the parameters within specified bounds; `Partial` optimizes only selected components while retaining the others. This supports controlled model fitting, including freezing known correlation parameters while adapting others.

`GpParams` implements Linfa's parameter-validation pattern through `ParamGuard`. The API exposes model-quality metrics, including k-fold and leave-one-out predictive \\(Q^2\\).

The GP implementation includes an optimization backend for hyperparameter estimation, so this is not simply a covariance kernel evaluation library.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

`https://docs.rs/egobox-gp/latest/egobox_gp/struct.GpParams.html`

`https://docs.rs/egobox-gp/latest/egobox_gp/enum.ThetaTuning.html`

### 3.4 KPLS: Partial Least Squares dimension reduction

EGObox integrates Kriging with Partial Least Squares (KPLS) to reduce the effective dimension of the correlation model.

Mechanism: PLS identifies directions associated with the variation in the response. GP correlation modeling is then performed in a reduced parameter space rather than independently parameterizing every original dimension.

Advantages: fewer correlation parameters, potentially better-conditioned fitting, and lower hyperparameter optimization burden on problems with substantial effective low-dimensional structure.

Limitations: PLS is supervised dimensionality reduction, not a universal guarantee of accurate approximation in high dimensions. It can lose low-variance but important directions, especially when the response contains localized behavior that the training data fail to reveal.

Crucially, KPLS primarily addresses input-dimensional complexity. It is not the same as sparse GP regression, which targets training-sample complexity.

### 3.5 Sparse Gaussian processes: FITC and VFE

The second major modeling family is `SparseGaussianProcess`, configured using `SgpParams`.

A full GP typically requires a dense \\(N \times N\\) covariance matrix for \\(N\\) training observations, with approximate training complexity \\(O(N^3)\\) and memory complexity \\(O(N^2)\\).

The upstream documentation characterizes its sparse implementation as reducing these to approximately:

\\[ \text{Training: }O(NM^2) \qquad \text{Memory: }O(NM), \\]

where \\(M\\) is the number of inducing points and \\(M \ll N\\). These are algorithmic complexity characterizations, not measured end-to-end speedups.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



| Sparse method        | Mechanism                                            | Distinction                                                                                 |
| -------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `SparseMethod::Fitc` | Fully Independent Training Conditional approximation | Low-rank GP approximation with a diagonal correction to the training conditional covariance |
| `SparseMethod::Vfe`  | Variational Free Energy approximation                | Fits a variational sparse GP approximation using an evidence-lower-bound formulation        |

Inducing point configuration uses `Inducings`:

- `Randomized(M)` selects a specified number of training observations as inducing locations.
- `Located(Array2<F>)` supplies the inducing locations explicitly.

This makes inducing-point selection a meaningful model-design decision rather than a fixed internal implementation detail.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Important distinctions:

A sparse GP is generally an approximation rather than an exact interpolator. FITC and VFE also have different likelihood approximations and uncertainty behavior; they should not be treated as interchangeable merely because both use inducing points.

Sparse modeling is particularly relevant when expensive function evaluations have nevertheless accumulated into substantial historical datasets. It is less automatically beneficial when only a few dozen evaluations exist and the modeling overhead is already small.

### 3.6 Incremental training updates

The September 22, 2026 release introduced `GaussianProcess::update()`, explicitly described upstream as supporting incremental training data without a full refit.

This permits reuse of existing model state when incorporating new observations, avoiding some of the work associated with rebuilding a GP entirely.

It is especially interesting for sequential simulation, active learning, and Bayesian optimization loops where new data arrive incrementally. However, incremental model updating is not equivalent to complete reoptimization of the kernel hyperparameters; whether and when to retune those parameters is a separate modeling consideration.

Source: `https://github.com/relf/egobox/releases/tag/0.37.9`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 3.7 Important numerical and modeling boundaries

GP quality depends on the interaction of training design, output scale, covariance assumptions, numerical conditioning, and hyperparameter fitting. A predictive variance is an uncertainty estimate conditional on the model assumptions, not a guarantee of true prediction error.

A nugget can stabilize numerically difficult covariance matrices, but it should not automatically be interpreted as a complete, independently estimated observation-noise model.

For large \\(N\\), sparse GP methods are the major native approximation mechanism. For large input dimension, KPLS is the principal reduction mechanism. These address different bottlenecks and can be combined where supported by the chosen GP/MoE configuration.

Primary documentation:

`https://docs.rs/egobox-gp/latest/egobox_gp/`

`https://docs.rs/egobox-gp/latest/egobox_gp/struct.GaussianProcess.html`

`https://docs.rs/egobox-gp/latest/egobox_gp/struct.SparseGaussianProcess.html`

## 4. `egobox-moe` — Mixture-of-experts surrogate modeling

Crate 0.36.2 | Core APIs: `GpMixture`, `GpMixtureParams`, `MixintGpMixture`, `NbClusters`, `Recombination`, `GpType`

`egobox-moe` addresses a fundamental limitation of global Gaussian process models: a single stationary covariance structure may approximate functions poorly when different regions exhibit different smoothness, curvature, discontinuities, or other local response characteristics.

It partitions the data using a Gaussian mixture model, trains GP experts for the resulting clusters, and recombines their predictions.

### 4.1 Clustering and expert construction

The model comprises three elements:

1. Clustering: `linfa-clustering` fits a Gaussian mixture model (GMM), providing cluster membership probabilities and the basis for regional partitioning.
2. Local regression: GP experts fit the observations associated with their respective clusters, using the underlying `egobox-gp` functionality.
3. Gating/recombination: Cluster responsibilities determine which experts contribute to a prediction.

For a smooth mixture, the predicted mean can be understood schematically as

\\[ \hat f(x)=\sum\_{k=1}^{K}w_k(x)\mu_k(x), \\]

where \\(w_k(x)\\) is the responsibility of expert \\(k\\), and \\(\mu_k(x)\\) is that expert's prediction.

This is a collection of locally trained GP models, not a single GP with a specialized globally stationary kernel. Consequently, local model fitting and the gating mechanism both matter to prediction quality and uncertainty calibration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.2 Hard versus smooth recombination

| Mode                         | Mechanism                                                          | Advantages and limitations                                                                       |
| ---------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `Recombination::Hard`        | Selects the expert with the largest cluster membership probability | Preserves distinct regional behaviors, but predictions can be discontinuous at expert boundaries |
| `Recombination::Smooth(...)` | Combines predictions using cluster responsibility weights          | Produces a continuous combined prediction, but can blur abrupt changes between regions           |

The smooth mode has a configurable Heaviside factor that modifies how sharply responsibilities transition between experts. The implementation can select this factor automatically or accept a supplied value.

These choices are consequential for downstream optimization. A discontinuous hard-gated surrogate may be valuable when representing genuinely different operating regimes, but its gradients can behave poorly at region boundaries. Smooth gating can improve numerical behavior when optimizing acquisition functions, at the expense of potentially smoothing real discontinuities.

Crucially, weighting expert means and uncertainties does not automatically reproduce the joint posterior of a single Gaussian process. That distinction matters for acquisition criteria requiring a coherent joint covariance structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.3 Adaptive model selection

`GpMixtureParams` exposes multiple levels of model configuration:

| Capability             | API handles                                                    | Consequence                                                        |
| ---------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------ |
| Expert count           | `NbClusters::fixed(...)`, automatic selection                  | Fixed regional structure versus data-driven model complexity       |
| Regression families    | `RegressionSpec`                                               | Candidate constant, linear and quadratic trends                    |
| Kernel families        | `CorrelationSpec`                                              | Candidate squared-exponential, exponential and Matérn correlations |
| Dimension reduction    | `kpls_dim` / PLS configuration                                 | Reduced effective correlation dimension                            |
| Hyperparameter fitting | Initial parameters, tuning specifications, multistart controls | More control over fitting cost and model flexibility               |
| Sparse modeling        | `GpType`, `SparseMethod`, `Inducings`                          | Sparse-GP experts in place of full GP experts                      |
| Model recombination    | `Recombination`                                                | Hard expert selection versus continuous weighting                  |

Regression and correlation specifications support multiple permitted models. EGObox can evaluate alternative GP configurations and select suitable combinations, rather than requiring every expert to use one fixed model family.

Automatic cluster count selection is supported through `find_best_number_of_clusters()`, using cross-validation. This reduces the need for a separate bespoke model-selection routine, although it adds fitting cost and remains sensitive to the available data.

The capability is particularly valuable when a response is piecewise smooth but globally complex. It is less compelling if a single well-calibrated GP fits adequately or if there are too few observations to train reliable local experts.

### 4.4 Surrogate quality assessment

The MoE crate exposes dedicated validation interfaces, including `GpMetrics`, `GpQualityAssurance`, and `GpMetric`.

Supported assessment families include:

- Predictive \\(Q^2\\), including k-fold and leave-one-out variants, to quantify out-of-sample predictive fit.
- Predictive variance adequacy measures, evaluating whether model uncertainty behaves consistently with observed errors.
- Integrated absolute-error and alpha-related diagnostics, providing additional information about surrogate reliability.

These checks are especially important when a surrogate is used to make decisions rather than merely to interpolate existing data. A high-quality mean prediction and a well-calibrated uncertainty estimate are distinct requirements: acquisition functions depend on both.

The validation machinery is part of the modeling stack and need not be independently reimplemented merely to compare its supported surrogate alternatives.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.5 Mixed-variable domain modeling

`egobox-moe` contains much of the infrastructure through which the optimizer handles discrete variables.

Important types include `MixintContext`, `MixintSampling`, `MixintGpMixture`, and `XType`.

The domain model supports:

- `Float`: bounded real-valued coordinates.
- `Int`: bounded integer-valued coordinates.
- `Ord`: ordered, discrete levels.
- `Enum`: categorical choices.

`to_continuous_space()`, `to_discrete_space()`, and `as_continuous_limits()` perform conversions between user-facing mixed domains and the continuous representation used by the surrogate and acquisition optimization.

Categorical variables may be expanded into additional continuous coordinates; after optimization, candidates are folded or projected back into valid discrete choices.

This is continuous relaxation and projection, not exact mixed-integer or combinatorial optimization. Multiple relaxed points can represent the same discrete candidate, and continuous surrogate geometry may not faithfully represent categorical similarity.

The October 7, 2026 upstream fixes specifically addressed categorical folding/unfolding and sampling fairness. Integer, ordered and enumerated levels are now sampled with equal probability by the relevant mixed-domain sampling mechanisms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.6 Sparse experts and incremental surrogate maintenance

MoE can combine two different approximations:

- Local partitioning reduces the complexity of the response fitted by each expert.
- Sparse GP inference reduces the cost of fitting an expert with substantial training data.

These mechanisms are complementary, though their interaction increases configuration complexity and can make cross-validation more expensive.

A major improvement in the October 2026 releases is adaptive surrogate updating within EGO. Rather than retraining every surrogate at every iteration, EGObox retains fitted models and incrementally updates them when the cluster structure is unchanged and new observations pass a model-consistency check. Otherwise it triggers a full refit.

This is an important architectural distinction from a simplistic Bayesian optimization loop that rebuilds every surrogate from scratch. It reduces repeated training work where a new observation does not justify revisiting clustering or hyperparameters. The benefit depends on how often the model can safely retain its previous structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 4.7 Extension interfaces and persistence

The crate defines a hierarchy of surrogate abstractions:

| Trait                | Role                                                                   |
| -------------------- | ---------------------------------------------------------------------- |
| `GpSurrogate`        | Common surrogate prediction interface                                  |
| `GpSurrogateExt`     | Additional prediction capabilities, including derivatives and sampling |
| `GpSurrogateParams`  | Model configuration and construction                                   |
| `FullGpSurrogate`    | Full GP capabilities, including update support                         |
| `SgpSurrogate`       | Sparse GP modeling                                                     |
| `MixtureGpSurrogate` | Expert-mixture-specific capabilities                                   |
| `SurrogateBuilder`   | Surrogate training and construction interface                          |
| `Clustered`          | Cluster-structured model behavior                                      |

These abstractions support model-level composition and make it possible to develop alternative compatible surrogate builders without having to replace the entire optimization framework.

`AffinedSurrogate` is an additional wrapper that applies an affine transformation to another surrogate's predictions. It provides a reusable route for representing transformed model outputs.

Persistence: Current MoE models support serialization and `save()`/`load()` operations, with JSON and binary formats documented. The October 2026 changes made persistence part of the default MoE/EGO configuration rather than requiring the older `persistent` feature switch.

Sources:

`https://docs.rs/egobox-moe/latest/egobox_moe/`

`https://docs.rs/egobox-moe/latest/egobox_moe/struct.GpMixtureParams.html`

`https://docs.rs/egobox-moe/latest/egobox_moe/trait.SurrogateBuilder.html`

`https://github.com/relf/egobox/releases`

## 5. `egobox-ego` — Efficient Global Optimization

Crate 0.42.0 | Core APIs: `EgorBuilder`, `EgorConfig`, `Egor`, `EgorSolver`, `EgorServiceFactory`, `MooConfig`

This is EGObox's principal optimization engine. It combines DOE, probabilistic surrogate modeling, acquisition optimization, constraints, and sequential evaluation into a configurable Bayesian optimization algorithm.

Its distinctive capabilities extend substantially beyond conventional single-objective expected improvement: constrained and hidden-feasibility optimization, adaptive surrogate maintenance, batched evaluations, trust-region search, cooperative high-dimensional optimization, external ask-and-tell operation, and experimental multi-objective optimization.

### 5.1 Optimization mechanism and evaluation contract

A conventional EGO iteration consists of four principal operations:

1. Update GP/MoE surrogate models from evaluated observations.
2. Construct an acquisition function using predicted objective values, uncertainties, and applicable constraints.
3. Optimize that comparatively inexpensive acquisition function to choose new candidate points.
4. Evaluate the actual expensive function at the candidates and incorporate the observations.

For minimization, the standard expected improvement criterion can be represented as

\\[ \mathrm{EI}(x)=\mathbb E[\max(f\_{\min}-Y(x),0)]. \\]

The acquisition mechanism can favor both regions expected to improve the current best value and regions whose uncertainty makes improvement plausible.

Important distinction: the original function is treated as gradient-free, but the acquisition optimization can still exploit derivatives of the surrogate. The absence of objective derivatives does not require every internal numerical operation to be derivative-free.

The Rust objective interface accepts a matrix of \\(p\\) input points and returns a matrix of corresponding outputs.

For a single objective with \\(m\\) expensive constraints, each returned row has the logical layout

`[objective, constraint_1, ..., constraint_m]`.

For multi-objective optimization with \\(k\\) objectives:

`[objective_1, ..., objective_k, constraint_1, ..., constraint_m]`.

`EgorBuilder::optimize()`, `EgorFactory::configure()`, `min_within()`, and `min_within_mixint_space()` are the primary entry points.

A supplied initial DOE may contain input coordinates alone or both coordinates and already evaluated outputs. The latter enables reuse of existing simulation data without repeating those expensive evaluations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.2 Acquisition functions and internal optimization

| Criterion                   | API                                  | Significance                                                      |
| --------------------------- | ------------------------------------ | ----------------------------------------------------------------- |
| Expected Improvement        | `InfillStrategy::EI`                 | Standard balance of predicted improvement and uncertainty         |
| Log Expected Improvement    | `InfillStrategy::LogEI`              | More numerically stable treatment when EI becomes extremely small |
| Watson–Barnes 2             | `InfillStrategy::WB2`                | Alternative infill criterion oriented toward locating extrema     |
| Scaled WB2                  | `InfillStrategy::WB2S`               | Scaled variant of WB2                                             |
| Constrained acquisition     | Constraint-aware EI/LogEI mechanisms | Incorporates feasibility into candidate selection                 |
| Feasibility-enhanced infill | `FeasibleInfillStrategy`             | Responds to uncertain or difficult feasibility regions            |

The default single-objective criterion is WB2. EI, LogEI and WB2S are configurable alternatives.

LogEI is particularly notable because evaluating EI directly can become numerically difficult as improvement probabilities and expected improvements approach zero. A logarithmic formulation can improve acquisition optimization without changing the intended optimization problem.

The `criteria` module also defines an `InfillCriterion` trait, allowing the criterion layer to be investigated or extended independently of the outer EGO iteration machinery.

Internal optimization controls include `InfillOptimizer`, multistart acquisition optimization (`n_start`), and GP hyperparameter search configuration. The supported acquisition optimizers include SLSQP and COBYLA.

The October 2026 changes moved the default internal optimization implementation to the pure-Rust Basin ecosystem. Earlier documentation referring to `argmin` or optional `nlopt` is therefore not an accurate description of the current backend. C-backed optimizer alternatives are now opt-in through `c-cobyla` and `c-slsqp` features.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 5.3 Explicit constraints: three distinct mechanisms

EGObox distinguishes several kinds of feasibility information.

| Mechanism                     | Where constraints are evaluated            | Main distinction                                                        |
| ----------------------------- | ------------------------------------------ | ----------------------------------------------------------------------- |
| Expensive modeled constraints | Returned alongside the objective           | Learn surrogate approximations to constraint responses                  |
| Function constraints          | Supplied separately through `subject_to()` | Evaluate explicit constraints directly during candidate search          |
| Hidden constraints            | Discovered when function evaluations fail  | Learn or infer regions where the objective cannot be evaluated reliably |

Expensive constraints. The objective evaluation returns constraint values as additional columns. EGObox models these responses with surrogates and uses their predictions to guide the search.

The canonical feasibility convention is \\(g(x)\leq0\\), but `CstrSpec` supports alternative specifications:

| `CstrSpec` | User-visible meaning   | Internal conversion         |
| ---------- | ---------------------- | --------------------------- |
| `Leq(b)`   | \\(c(x)\leq b\\)       | \\(c(x)-b\leq0\\)           |
| `Geq(b)`   | \\(c(x)\geq b\\)       | \\(b-c(x)\leq0\\)           |
| `Eq(b)`    | \\(c(x)=b\\)           | Two unilateral inequalities |
| `Btw(a,b)` | \\(a\leq c(x)\leq b\\) | Two unilateral inequalities |

Constraint tolerances determine acceptable violation. An equality constraint therefore requires appropriate tolerance handling rather than assuming exact floating-point equality.

The October 2026 Rust results preserve the original user-facing constraint values in `OptimResult.y_opt` and `y_doe` when constraint specifications are used, while internally maintained DOE files retain the transformed convention. This distinction matters when observations are reused, inspected, or supplied to an external optimization controller.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Surrogate-based feasibility controls. The optimizer exposes `cstr_infill()` for incorporating probability of feasibility into acquisition and `ConstraintStrategy::{MeanConstraint, UpperTrustBound}` for selecting how modeled constraint information is used.

The former accounts for predicted uncertainty in feasibility; the latter changes the criterion used to evaluate the constraint surrogate. They address related but different decisions.

Function constraints. `EgorFactory::subject_to()` accepts separately defined functions through the `CstrFn` interface. These can represent inexpensive geometric, algebraic, or domain constraints, allowing the acquisition optimizer to check feasibility without invoking the full expensive objective.

In mixed-variable problems, the October 2026 fixes ensure that these functions receive folded, discrete-domain inputs where appropriate, with gradients mapped back to the relaxed representation.

### 5.4 Hidden constraints and failed evaluations

This is one of the more easily overlooked capabilities.

A black-box function may fail for certain inputs because a simulator does not converge, a numerical model becomes invalid, or an underlying implementation rejects the chosen parameters. Such failure regions are not necessarily described by an explicit algebraic constraint.

`FailsafeStrategy` provides three approaches:

| Strategy     | Mechanism                                                      | Main tradeoff                                                               |
| ------------ | -------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `Rejection`  | Excludes failed observations                                   | Simple, but may repeatedly explore regions with unmodeled failure risk      |
| `Imputation` | Substitutes a surrogate-derived objective estimate for failure | Preserves a numerical training response, but creates synthetic observations |
| `Viability`  | Uses a surrogate for probability of evaluation success         | Explicitly models whether evaluation is likely to succeed                   |

The imputation documentation describes a prediction-based substitute with an uncertainty penalty; such imputed values must not be confused with actual objective observations.

The additional `FeasibleInfillStrategy::EfiFe(...)` implements a feasibility-enhanced acquisition mechanism intended to improve search behavior in difficult feasible regions.

These features make EGObox relevant to failure-prone engineering simulations, where the feasibility landscape may be only partially known before evaluation.

Limitations remain important: learning a failure boundary requires informative successes and failures, while imputation can bias optimization if the substitute responses are treated as equally reliable as measured data.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.5 Parallel and batched optimization: qEI

`EgorConfig::configure_qei()` configures batch candidate selection, allowing multiple candidate points to be identified before their actual objective values become available.

The batch-selection methods include:

| `QEiStrategy`               | Virtual observation mechanism                                  |
| --------------------------- | -------------------------------------------------------------- |
| `KrigingBeliever`           | Uses the surrogate mean at previously proposed batch points    |
| `KrigingBelieverLowerBound` | Uses an optimistic lower surrogate estimate                    |
| `KrigingBelieverUpperBound` | Uses a pessimistic upper surrogate estimate                    |
| `ConstantLiarMinimum`       | Uses the best observed objective value as the virtual response |

These strategies manage pending information. Once the first candidate is selected, its outcome is temporarily approximated so the selection of subsequent candidates can account for the pending point.

This helps reduce redundant proposals when external evaluations are expensive and several workers can evaluate candidates concurrently.

There is a consequential boundary: qEI produces candidate batches, but the surrounding application remains responsible for executing arbitrary external evaluators concurrently. The batch-capable Rust objective signature does not itself establish a universal remote or asynchronous execution service.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.6 TREGO: Global and local trust-region optimization

`TregoStrategy` implements Trust Region EGO, an algorithmic variant combining global exploration with localized refinement.

Its mechanism alternates between:

- Global EGO iterations, which search across the full design space.
- Local iterations, which search within a trust region centered around a promising candidate.

The trust-region radius changes according to observed progress, allowing the method to adapt the extent of local refinement.

The configurable parameters include the balance of global/local steps, radius bounds, and contraction/expansion behavior. `EgorConfig::trego()` provides activation, while `configure_trego()` and `iteration_strategy(Box<dyn IterationStrategy>)` expose more detailed customization.

Why it matters: conventional EGO may continue spending effort on global exploration even when a promising basin has been identified. TREGO provides a native mechanism for directing a portion of the evaluation budget toward local convergence without abandoning global search entirely.

The distinction is especially important when the objective evaluations are costly enough that improving the allocation of a limited evaluation budget matters more than the computational cost of the optimizer itself.

Current implementation refinement: local TREGO steps can retain and incrementally update the fitted surrogates, reducing repeated training work.

TREGO is not currently supported with the experimental multi-objective mode.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 5.7 CoEGO: Cooperative high-dimensional optimization

`CooperativeActivity` implements the Cooperative EGO variant using a Cooperative Components Bayesian Optimization (CCBO) approach.

Its central idea is to avoid optimizing every design variable simultaneously at every step.

Instead:

1. Variables are grouped into components.
2. Only selected components are active during a particular subproblem.
3. The remaining variables are represented by a context vector.
4. Components are optimized cooperatively, with the expensive objective evaluated after context updates.

The default `FullActivity` strategy keeps all variables active, while `CooperativeActivity` provides the alternative component-based mechanism.

This targets a different form of high-dimensional difficulty from KPLS:

- KPLS reduces the effective dimension used by the surrogate.
- CoEGO reduces the number of actively optimized coordinates in a subproblem.

The strategies can be conceptually complementary, although suitability depends on the actual objective structure and on the combinations supported by the configured optimizer.

Cooperative grouping is most compelling when variables admit useful partial optimization. Strong, widely distributed cross-variable interactions can make a component-wise search less effective.

`ActivityStrategy` separates the variable-activity policy from the main solver. Combined with `IterationStrategy`, this provides two meaningful extension dimensions: which variables participate in an iteration and whether that iteration searches globally or locally.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/egobox-ego/latest/egobox_ego/activity_strategy/`

`https://docs.rs/egobox-ego/latest/egobox_ego/iteration_strategy/`

### 5.8 Experimental multi-objective optimization — new in 1.0.0

The October 12, 2026 release added multi-objective optimization to Rust `Egor` and exposed it through Python `Belfegor`.

This is released but explicitly experimental functionality. The broader Python EGObox API is designated stable; that does not imply the multi-objective implementation has the same maturity.

The Rust interface uses `EgorConfig::n_obj()` and `configure_moo()`. `Egor::run_pareto()` returns a `ParetoResult` containing the approximated Pareto set and front, a compromise point, and experimental data.

`Egor::run()` remains available and selects a compromise point from the Pareto front using a normalized, uniformly weighted augmented Tchebycheff criterion. All objectives are minimized.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



#### Multi-objective strategy comparison

| Strategy              | Algorithm                              | Model structure                                  | Main advantage                                                                      |
| --------------------- | -------------------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `MooStrategy::Ehvi`   | Expected Hypervolume Improvement       | One surrogate per objective                      | Directly targets expected improvement of dominated hypervolume                      |
| `MooStrategy::Eim`    | Expected Improvement Matrix            | One surrogate per objective                      | Flexible improvement aggregation through Euclidean, maximin or hypervolume criteria |
| `MooStrategy::ParEgo` | Random weighted scalarization          | One scalarized-objective surrogate per iteration | Avoids the full multi-output hypervolume acquisition problem                        |
| `MooStrategy::QEhvi`  | Batch expected hypervolume improvement | Joint posterior of objective surrogates          | Chooses related candidates while accounting for their joint contribution            |

EHVI is the default for two or three objectives. The upstream implementation uses closed-form EHVI on a box decomposition of the nondominated region and reports support for up to eight objectives. The computational burden of hypervolume calculations can increase substantially with objective count and front complexity; the supported limit does not establish practical performance for arbitrary datasets.

EIM supports alternative aggregation policies. This provides a different way to balance expected improvements across objectives when hypervolume is not the preferred selection geometry.

ParEGO is the default beyond three objectives. It changes scalarization weights across iterations, using a single-objective surrogate machinery rather than fitting an independently modeled acquisition response for each objective. This reduces one aspect of model/acquisition complexity, but scalarization can represent the Pareto front unevenly depending on the objective geometry.

qEHVI uses EHVI for the first batch candidate, followed by a greedy sequence of candidates evaluated through Monte Carlo joint-posterior hypervolume improvement. This is meaningfully different from independently choosing the highest-EHVI candidates: correlations among pending candidates become relevant.

The present qEHVI implementation is restricted to batches of at most four points and requires single-cluster surrogates. It therefore has a narrower modeling envelope than unrestricted MoE-based EGO.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



#### Multi-objective operational features and restrictions

| Capability                                | Multi-objective support                     |
| ----------------------------------------- | ------------------------------------------- |
| Explicit expensive constraints            | Supported                                   |
| Separately evaluated function constraints | Supported                                   |
| Mixed integer/categorical design spaces   | Supported                                   |
| Batched candidate selection               | Supported                                   |
| Warm/hot restart                          | Supported                                   |
| Ask-and-tell                              | Supported                                   |
| Hypervolume-based stopping                | `MooConfig::hv_stop(tolerance, iterations)` |
| Failure imputation                        | EHVI, EIM and qEHVI; not ParEGO             |
| TREGO                                     | Not supported                               |
| CoEGO                                     | Not supported                               |
| Single-objective target threshold         | Not supported                               |

The library additionally provides `find_pareto_front_indices()` and `find_compromise_index()` helpers. These can operate on existing evaluation data independently of a full optimization run.

Architectural consequence: the multi-objective addition makes EGObox a viable candidate for expensive Pareto optimization without requiring a separate optimization package for basic front identification and hypervolume-driven infill. Nevertheless, the experimental designation, qEHVI restrictions, and limited combination with the high-dimensional strategies are material boundaries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Primary sources:

`https://docs.rs/egobox-ego/latest/egobox_ego/struct.MooConfig.html`

`https://docs.rs/egobox-ego/latest/egobox_ego/enum.MooStrategy.html`

`https://docs.rs/egobox-ego/latest/egobox_ego/struct.ParetoResult.html`

`https://github.com/relf/egobox/releases`

### 5.9 External control: ask-and-tell optimization

`EgorServiceFactory` and `EgorServiceApi` expose the optimizer as a candidate suggestion service, separating optimization decisions from actual objective evaluations.

`EgorServiceApi::suggest(x_data, y_data)` accepts an already evaluated dataset and returns promising new candidate inputs.

This inverted-control interface is one of EGObox's most valuable integration mechanisms. The external system can own evaluation scheduling, storage, execution, retries, and data collection while using EGObox to decide which points deserve evaluation.

It is particularly applicable to distributed simulations and evaluation pipelines in which an arbitrary Rust closure is not the appropriate execution abstraction.

The interface should not be mistaken for a built-in HTTP or RPC server. The documented API is an in-process Rust service abstraction; a network endpoint or scheduler would require a separate integration layer.

A further practical detail: the current Rust documentation does not declare `EgorServiceApi` as `Send`. Consequently, shared cross-thread service usage should not be assumed without examining the precise ownership and synchronization requirements. External evaluations can still be parallelized independently of the service object.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source:

`https://docs.rs/egobox-ego/latest/egobox_ego/struct.EgorServiceApi.html`

### 5.10 Persistent state, restart and execution management

`Egor` now uses a Basin executor and a solver implementing `basin::Solver`, exposing executor functionality including checkpointing, observers and interruption handling.

The optimizer maintains explicit state across iterations, including DOE, surrogate, iteration-strategy, and cooperative optimization state.

| Mechanism    | Behavior                                                           | Main use                                        |
| ------------ | ------------------------------------------------------------------ | ----------------------------------------------- |
| `outdir`     | Persists configuration and optimization history                    | Inspection and reproducibility                  |
| `warm_start` | Loads previously evaluated DOE and starts a new optimization phase | Continuing an experiment with additional budget |
| `hot_start`  | Resumes from a saved solver checkpoint                             | Recovering interrupted execution                |
| `RunInfo`    | Associates metadata with a run                                     | Identifying and qualifying results              |
| `max_iters`  | Sets evaluation-iteration budget                                   | Bounded optimization                            |
| `target`     | Stops at a specified single-objective target                       | Known acceptable objective threshold            |
| `timeout`    | Limits elapsed optimization time                                   | Runtime control                                 |
| `seed`       | Controls randomized components                                     | Reproducible candidate selection                |

Typical persisted artifacts include `egor_config.json`, initial/current DOE `.npy` files, and a per-iteration optimization history. Additional files can contain checkpointed state, fitted GP models and failed evaluation points.

Warm start and hot start are not equivalent. A warm start reconstructs optimization from previously evaluated observations; a hot start resumes a saved execution state, including relevant iteration progress. Their budget and convergence semantics can consequently differ.

A significant 2026 improvement is retaining and incrementally updating surrogates rather than retraining every model on every iteration. This optimization is conditional: EGObox performs a more complete refit when its consistency checks indicate that the previous structure is no longer suitable.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/egobox-ego/latest/egobox_ego/struct.EgorConfig.html`

`https://docs.rs/egobox-ego/latest/egobox_ego/struct.EgorSolver.html`

## 6. Ecosystem integration and build boundaries

### 6.1 Rust numerical interfaces

EGObox integrates tightly with the Rust scientific computing ecosystem.

| Component                        | Role                                                                             |
| -------------------------------- | -------------------------------------------------------------------------------- |
| `ndarray`                        | Dense matrix representations for design inputs, observed outputs and predictions |
| `linfa`                          | Fitting, prediction and validated-parameter interface conventions                |
| `linfa-clustering`               | Gaussian mixture clustering underlying MoE                                       |
| `linfa-pls`                      | Partial Least Squares dimensionality reduction                                   |
| `linfa-linalg`                   | Default pure-Rust linear algebra                                                 |
| `ndarray-linalg`                 | Optional external BLAS/LAPACK integration                                        |
| Basin                            | Current optimization backend for solver execution and internal optimization      |
| `serde`, `serde_json`, `bincode` | Model and execution-state serialization                                          |
| `ndarray-npy`                    | Persistence and interchange of numerical DOE/history matrices                    |
| Rayon                            | Parallel CPU execution where supported by individual components                  |

EGObox does not require an external BLAS installation by default. Its optional `blas` feature allows use of an external linear algebra backend with the appropriate Linfa/`ndarray-linalg` configuration.

This may change performance and deployment requirements. A BLAS backend should not be assumed to improve all workloads, particularly when surrogate matrices are small or overhead dominates.

The current pure-Rust optimizer backend also reduces the need for foreign numerical optimization libraries in default configurations. Older references to the `nlopt` feature should be interpreted in their historical version context; the October 2026 release explicitly removed that feature in favor of the newer defaults and optional `c-cobyla`/`c-slsqp` alternatives.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 6.2 Python bindings and CLI

The `egobox` Python package exposes the native Rust functionality through Python-facing abstractions.

| Python interface    | Related native capabilities                             |
| ------------------- | ------------------------------------------------------- |
| `Egor`              | Single-objective EGO and its variants                   |
| `Belfegor`          | Experimental multi-objective optimization               |
| `Gpx`               | GP/MoE surrogate modeling                               |
| `SparseGpx`         | Sparse GP surrogate modeling                            |
| Sampling interfaces | DOE generation, including mixed-variable specifications |
| `gpx` CLI           | Command-line surrogate-model workflows                  |

The Python 1.0.0 release makes several previously deprecated names invalid and changes GP hyperparameter and likelihood accessors to read-only properties. The Rust API changes for that release are described upstream as additive, but this should not be interpreted as a guarantee that the independently versioned Rust crates all have stable 1.x APIs.

The project additionally maintains `egobox-gpx`, a separate Rust CLI crate, and bundles the `gpx` command with the Python distribution. These are companion interfaces rather than replacements for the four core Rust capability families.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

`https://pypi.org/project/egobox/1.0.0/`

`https://docs.rs/egobox-gpx/latest/egobox_gpx/`

## 7. Capability interactions and consequential tradeoffs

The most valuable architectural opportunities often arise from combinations of features rather than one feature in isolation.

| Problem characteristic                                  | Relevant native combination                        | Important consideration                                                |
| ------------------------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------- |
| Expensive, smooth, low-dimensional objective            | LHS + full GP + EI/LogEI                           | Simple high-quality surrogate modeling may be sufficient               |
| Response with several different local regimes           | MoE clustering + expert-specific GP models         | Expert boundaries and uncertainty combination affect acquisition       |
| Large historical training dataset                       | Sparse FITC/VFE + inducing-point selection         | Approximation trades model fidelity for training cost                  |
| Many design variables with low effective dimension      | KPLS + EGO                                         | PLS can overlook localized or weakly represented directions            |
| High-dimensional optimization with separable structure  | CoEGO + component activity                         | Strong cross-component interactions can limit effectiveness            |
| Promising local basin with remaining global uncertainty | TREGO + incremental surrogate update               | Local/global allocation must match the evaluation budget               |
| Costly or uncertain feasibility                         | Constraint surrogates + probability of feasibility | Feasibility uncertainty can dominate candidate selection               |
| Function evaluation failures                            | Viability modeling or EFI-FE                       | Failure data must represent meaningful feasibility information         |
| Multiple available simulation workers                   | qEI + external batch execution                     | Candidate generation and scheduling remain distinct responsibilities   |
| Competing expensive objectives                          | EHVI/EIM/ParEGO/qEHVI + Pareto tracking            | Multi-objective mode is experimental with important restrictions       |
| Externally orchestrated evaluation                      | Ask-and-tell + persisted DOE                       | The caller owns data consistency, evaluation lifecycle and concurrency |

### 7.1 Three distinct scalability dimensions

A useful distinction is that EGObox addresses three different sources of complexity:

- Training-data size: sparse GP approximations and incremental updates reduce fitting cost.
- Input-dimensional complexity: KPLS and CoEGO reduce different aspects of high-dimensional modeling and search.
- Evaluation throughput: qEI and the ask-and-tell interface permit candidate generation to be separated from concurrent external evaluations.

These mechanisms are not interchangeable. For example, sparse GP inference does not solve a fundamentally difficult high-dimensional acquisition problem; CoEGO does not automatically reduce the cubic cost of fitting a dense GP with many observations.

### 7.2 Optimization guarantees and limitations

Global optimality is not guaranteed. EGO is a sequential, model-based optimization heuristic. Its behavior depends on the objective landscape, initial design, surrogate assumptions, acquisition criterion and evaluation budget.

Surrogate uncertainty is model-dependent. A GP predictive variance is not an independently validated confidence guarantee. Kernel misspecification, inaccurate clustering, sparse approximations, and discrete relaxation can all affect its quality.

Mixed variables are approximate. Relaxation and projection enable use of continuous surrogate machinery but are not equivalent to exhaustive combinatorial search or an exact mixed-integer solver.

No native multi-fidelity hierarchy is established here. EGObox provides surrogate modeling and multiple optimization variants, but the reviewed primary documentation does not establish a general-purpose, explicitly coupled multi-fidelity/co-Kriging optimization engine, automatic fidelity allocation, or built-in physics-informed solver. Such systems would require additional modeling or orchestration unless supported by an independently verified extension.

Multi-objective support has narrower combinations. Most importantly, TREGO and CoEGO are not currently available in the experimental multi-objective mode, and qEHVI has batch-size and expert-count restrictions.

### 7.3 What EGObox can replace—and what remains external

EGObox can replace substantial bespoke implementations of space-filling experimental design, GP model fitting and validation, mixture-of-experts construction, acquisition criteria, constrained candidate selection, trust-region EGO, and restartable optimization state management.

It does not replace the underlying objective evaluator, a general algebraic modeling language, a deterministic constrained global solver, an external job scheduler, or a durable application database.

This boundary is particularly relevant when objective evaluations are performed by simulations, separate processes, distributed workers, or heterogeneous numerical software. EGObox can provide the optimization intelligence while those components retain responsibility for executing and recording the actual evaluations.

## 8. Primary source map

The following plain-text URLs provide the most useful direct entry points for deeper API investigation.

| Area                                        | Primary source                                                            |
| ------------------------------------------- | ------------------------------------------------------------------------- |
| Project and implementation                  | `https://github.com/relf/egobox`                                          |
| Release history and current-version changes | `https://github.com/relf/egobox/releases`                                 |
| DOE API                                     | `https://docs.rs/egobox-doe/latest/egobox_doe/`                           |
| GP API                                      | `https://docs.rs/egobox-gp/latest/egobox_gp/`                             |
| Sparse GP configuration                     | `https://docs.rs/egobox-gp/latest/egobox_gp/struct.SgpParams.html`        |
| MoE API and mixed-variable helpers          | `https://docs.rs/egobox-moe/latest/egobox_moe/`                           |
| EGO API and algorithm variants              | `https://docs.rs/egobox-ego/latest/egobox_ego/`                           |
| EGO configuration                           | `https://docs.rs/egobox-ego/latest/egobox_ego/struct.EgorConfig.html`     |
| EGO external control                        | `https://docs.rs/egobox-ego/latest/egobox_ego/struct.EgorServiceApi.html` |
| EGO acquisition extension points            | `https://docs.rs/egobox-ego/latest/egobox_ego/criteria/`                  |
| EGO variable grouping                       | `https://docs.rs/egobox-ego/latest/egobox_ego/activity_strategy/`         |
| EGO iteration policies                      | `https://docs.rs/egobox-ego/latest/egobox_ego/iteration_strategy/`        |
| Python bindings                             | `https://pypi.org/project/egobox/1.0.0/`                                  |
| CLI companion crate                         | `https://docs.rs/egobox-gpx/latest/egobox_gpx/`                           |

Version note: some search-indexed docs.rs method pages still surface older EGObox crate versions despite the root documentation and upstream releases having advanced. The current release identification and newer behavior described here are based primarily on the October 2026 upstream release notes and the current crate-level documentation. The multi-objective implementation is explicitly experimental; other capabilities are described as released unless otherwise qualified.

Overall architectural assessment: EGObox's strongest distinguishing feature is the depth of its native composition: sampling, validated probabilistic surrogates, local expert models, constraint-aware Bayesian acquisition, alternative optimization strategies, and persisted incremental execution are parts of one toolkit. Its advanced value lies less in providing a single optimizer than in making these mechanisms available as independently useful and interoperable Rust capabilities.