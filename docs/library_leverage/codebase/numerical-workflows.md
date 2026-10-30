# Numerical workflows

This page covers fitting, uncertainty, sampling, surrogates, shooting and the statistics layered
over studies in `pse-runtime/src/workflow/` and `pse-runtime/src/math/`. The contracts are owned by
[§13](../../authoritative_design/sections/workflows-and-results.md#section-13),
[§15](../../authoritative_design/sections/numerical-execution.md#section-15),
[§19](../../authoritative_design/sections/workflows-and-results.md#section-19) (fitting in §19.4,
uncertainty in §19.8) and the supported-scope table in
[§25](../../authoritative_design/sections/scope-and-open-design.md#section-25). The baseline is on
the [entry page](README.md). Activities that are absent from the product altogether, rather than
extensions of these workflows, are on [Capability gaps](capability-gaps.md).

**System picture.** The runtime orchestrates numerical work; the kernels are libraries. Fitting is
weighted least squares with one fixed loss, `0.5 Σ importance ((pred − obs)/σ)²`, over two routes:
a native NLP (Ipopt or POUNCE) and a `levenberg-marquardt` residual route used only for
unconstrained, unbounded shapes. Fit products are a faer-SVD rank diagnostic, one covariance (exact
from the KKT reduced Hessian, or Gauss-Newton), Wald intervals and profile-likelihood chains on
`statrs` quantiles, and first-order propagation `Σ_y = J Σ_θ Jᵀ`. Designs come from `egobox-doe`
(Latin hypercube and uniform) plus a local Cartesian grid, mapped to bounds or authored levels and
fed to studies as point overlays. Surrogates run `egobox-ego` step by step inside one actor and
return only proposals. Cohort statistics are a DataFusion aggregate over frozen study cohorts.

What is strong and should be preserved: typed withholding with a stated reason, original-space
qualification of every estimate, realized values as the identity authority for designs, scalar
FMA-free sums with `-0.0` accumulators in the small covariance kernels, and a surrogate that never
presents a model output as an original result.

The opportunities here share one shape ([LOP-X022](../assessment-patterns.md#lop-x022)): a careful
local analysis owner exists beside repeat-evaluation machinery, and the non-asymptotic or nonlinear
counterpart users reach for when the local answer is withheld has no owner. `statrs` is called for
two quantiles. Designs, studies, frozen cohorts, qualified re-solves and a certified covariance all
exist; the workflows that compose them do not. Demand for any of them is unrecorded in the
architecture owners and unmeasured. The surrogate and multistart entry points are qualified Rust
APIs whose callers under `crates/` are tests; studies are the only live public consumer of designs.

**Coverage on this page.** Read closely: `fitting/covariance.rs`, `fitting/profile.rs`
(`Chains::solve`, `ChainPolicy::run`), `fitting/{preparation,foreign,experiments,regression}.rs`,
`workflow/uncertainty.rs` with `uncertainty/psd.rs`, and `workflow/sampling.rs` with
`math/sampling.rs`. Read to a stated depth: `fitting/oracle.rs` through its non-test code except the
`NlpOracle` gradient and Hessian bodies and the transient half of `evaluate_experiment`;
`fitting/oracle/residual.rs` (eligibility, `solve_using`, `termination`); `math/surrogate.rs` to the
actor and handle; `math/surrogate/library.rs` options and `Worker`. Only skimmed or read by outline:
`fitting/modeling.rs` (experiment construction), `fitting/results.rs`
(`encode_fit` and exports), `fitting/sparse.rs` (Gram refill), `workflow/analyses.rs` (about 1,500
lines), `study_variants.rs` and `shooting.rs::ShootingProblem::new`. Tests were read by name. The
parmest and `parameter_sweep` comparisons come from reading IDAES behaviour at class and docstring
level; no parity test was run.

<a id="cbo-036"></a>

## CBO-036 — Fit quality, identifiability and the unknown-sigma model

- **Operation and consumers.** Anyone reading a fit report: a kinetics or thermodynamic fitter
  deciding whether the declared model and sigmas are adequate, a modeller with a rank-deficient fit
  deciding which parameters to fix, and a fitter whose data lack trustworthy sigmas.
- **Current mechanism.** `fit_observations` publishes per-observation `residual`,
  `standardized_residual` and `objective_contribution`, and the covariance is published. No run-level
  degrees of freedom, reduced χ², χ² p-value, information criterion, parameter correlation matrix or
  leverage was found. Sigma is mandatory: `prepare_fit_problem_bound` refuses included observations
  without finite positive deviations, Wald intervals use a normal quantile (`covariance.rs::wald`) and
  profiles use χ²₁ (`profile.rs::profiles`). The rank verdict is published with condition and raw
  right singular vectors; interpretation falls to the user. `oracle.rs::decompose` computes no left
  singular vectors, and the dense response and singular values already sit on `FitReport`.
- **Why it matters.** A qualified fit with a large χ² produces a covariance and intervals with no
  signal that the declared sigma or model is inconsistent with the data. The same retained facts
  would support a correlation view and a pivoted-QR subset suggestion for rank-deficient fits.
- **Patterns.** LOP-12, LOP-X022, LOP-03.
- **Evidence.** Read `fitting/covariance.rs`, `oracle.rs::{decompose, rank_scratch,
  response_rank_admitted, observe}`, `results.rs::uncertainty`, generated `fit_observations` column
  names, `modeling.rs::prepare_fit_problem_bound`, §19.4, §25 Fitting row. Interface-checked.
- **Footprint.** Inspected as above. `results.rs::encode_fit` unread, so a test helper or relation may
  already compute a statistic the search did not see.
- **Preservation constraints.** Three obligations stay apart. (1) Additive diagnostics against declared
  sigma need no contract change. (2) Identifiability guidance advises and never alters the fit;
  parameter-scale dependence is stated and global identifiability is not claimed. (3) An estimated-variance
  path changes the statistical model. ADR-0118 item 2 decides the declared-sigma model and lists a fit
  without declared deviations as a negative control, and §25 lists "estimated residual variance" as
  refused, so (3) supersedes both. A scaled covariance would be a separately labelled approximation,
  never a silent rescale, and non-unit importance stays withheld.
- **Counterevidence and uncertainty.** Withholding is a deliberate honesty contract, and
  declared-sigma consistency is arguably more relevant than estimated sigma. Pyomo parmest 6.10.1
  estimates σ² = SSE/(n − l) over unique data points (read on 2026-10-27, not re-read). No consumer is
  recorded for (3). The pattern match for (3) is weak, and stronger for (1) and (2).
- **Status.** Current mechanism (published residual rows, rank verdict); Current promise (declared-sigma
  model); Proposed (diagnostics, identifiability guidance, estimated-variance path).
- **Research questions.** Which statistics need no contract change? Are tail quantiles of `statrs`
  Student-t, F and χ² accurate at large degrees of freedom? Does any consumer need estimated variance?

<a id="cbo-034"></a>

## CBO-034 — Non-local uncertainty: sampled propagation, global sensitivity and cohort statistics

- **Operation and consumers.** A fitter or flowsheet engineer wanting the distribution of a KPI through
  the nonlinear model, with percentile bands or a limit-exceedance probability. A sweep user ranking
  inputs across a range (Morris screening, Sobol indices). A study user summarizing a cohort by
  dispersion, percentiles and input-output correlation.
- **Current mechanism.** Only first-order `Σ_y = J Σ_θ Jᵀ` exists (`workflow/uncertainty.rs`), and it is
  withheld at a parameter bound, under rank deficiency or non-unit importance, leaving no continuation
  in exactly the cases where a nonlinear answer is wanted. Profiles cover parameters, not predictions.
  Designs map uniform unit values only; `DesignCoordinate` has no distribution or correlation, and
  `GeneratedDesign::attribution` disclaims a scientific distribution. Sensitivity is local throughout;
  `workflow/analyses.rs` is provenance reachability. `study_workbench.rs::aggregate` computes count,
  minimum, maximum and mean through DataFusion.
- **Why it matters.** First-order results are valid near the estimate only. The execution chain of design,
  study, frozen cohort and `TableProvider` is coherent; only a distribution mapping at the front and
  estimators at the back are absent. A prediction profile is a third route beside sampling and sigma points.
- **Patterns.** LOP-12, LOP-X022, LOP-05, LOP-X006.
- **Evidence.** Read `uncertainty.rs::{propagate, propagate_checked}`, `workflow/sampling.rs`,
  `math/sampling.rs::construct`, `study_workbench.rs::aggregate`, §19.8, §25; vocabulary searches for
  Monte Carlo, Sobol, Morris, Saltelli, jackknife found nothing. Interface-checked, by search for absence.
- **Footprint.** `study_variants.rs` overlay limits and `freeze_study_cohort` unread.
- **Preservation constraints.** Distinct obligations: (a) sampled propagation must carry the covariance
  origin, count failed and infeasible samples rather than drop them, and never call a Gaussian
  approximation a posterior; (b) global sensitivity needs an explicit failed-pair policy because failures
  break Saltelli pairing; (c) cohort statistics must keep failed, missing and unattempted members visible,
  give variance squared units, and decide exact versus approximate percentiles. Realized points govern replay.
- **Counterevidence and uncertainty.** IDAES propagation is itself first-order, so parity does not require
  this, and §19.8 scopes propagation as first-order. A frozen cohort already exports to IPC, so external
  engines can compute quantiles now; the design and its distributions cannot be done externally. Each
  sample is a full solve. Sampled draws from a faer factor plus `statrs` reproduce on one CPU class only
  (runtime SIMD and FMA dispatch, platform libm); realized values as identity make that tolerable.
- **Status.** Current mechanism (first-order propagation, fixed aggregates); Proposed (non-local routes).
- **Research questions.** Which consumer needs nonlinear propagation, at what solve budget? How are
  correlated non-Gaussian inputs expressed? Do maintained low-discrepancy sequences meet the recorded-point
  contract? Which DataFusion aggregates suit the cohort's physically typed columns?

<a id="cbo-135"></a>

## CBO-135 — Train, validate and embed a surrogate as a product

- **Operation and consumers.** A modeller replacing an expensive implicit block or property calculation
  with a trained approximation and optimizing through it; a sweep user training a cheap model of a KPI
  on cohort rows, validating it and querying it with uncertainty.
- **Current mechanism.** `egobox-ego`, `egobox-moe` and `egobox-gp` are linked, but the only entry is
  `EgorSolver` (`math/surrogate/library.rs`). The GPs trained at each iteration are discarded; the result
  is a `SurrogateProposal` screened by the original solve, and `SurrogateSnapshot` exposes `EgorState`
  only. No source calls a GP predict method. The embedding seam exists: `pse_kernels::ProviderFactory`,
  `Registration`, the optional output `envelope`, and a linked Rust author can reach it through
  `Runtime::modeling_package_registered`. Nothing connects a design, a cohort and a fit to it.
  `egobox-ego` carries the `persistent` feature (Cargo.toml) although no surrogate code saves or loads
  state; the supplier pin is 0.38.1.
- **Why it matters.** The library, design producer, study fan-out and fidelity vocabulary are already
  paid for. A trained model would also supply cheap evaluation to [CBO-034](#cbo-034). Whether the
  right form is an opaque worker or a symbolic or polynomial surrogate lowered to authored mathematics
  (compiled derivatives, factorable certification, no provider) is open; IDAES embeds its ONNX
  surrogate as algebraic constraints.
- **Patterns.** LOP-X022, LOP-12, LOP-03, LOP-X006.
- **Evidence.** Read `surrogate/library.rs`, `surrogate.rs` actor and handle,
  `compiler/typed_math/providers.rs::prepare_selected_providers`, `pse-authoring` parser for `external`,
  installed egobox surfaces by name. Interface-checked. `MathService::prepare_surrogate` has only test
  callers.
- **Footprint.** Whether an external worker process obtains a provider registration is unresolved: no
  `registered` or `providers` hit in `worker.rs`, `native_worker.rs`, `durable.rs`, `native_deployment.rs`.
  So durable studies through a registered provider are not established.
- **Preservation constraints.** Fidelity may change approximation quality, never the target; a surrogate
  proposal grants no result permission. Validity region and training-data identity retained; extrapolation
  refused or labelled. Non-`Send` egobox objects confined to a worker. A persisted form must be
  identity-hashable (library serde is version-coupled). Typed trial versus contract failure.
- **Counterevidence and uncertainty.** No named consumer; the model-seed cases are small. Embedding in an
  NLP needs derivatives egobox provides to first order only. ALAMO-style algebraic surrogates may be the
  higher-value form and are a different mechanism. Removing `persistent` may fail to compile.
- **Status.** Declared target, undelivered: surrogates are a target family, "Not implemented" in the coverage table, with no active plan. The proposal-only actor is a current mechanism. A retained model product and embedding route are Proposed.
- **Research questions.** Does `egobox-moe` expose serializable trained models with analytic derivatives?
  Is a symbolic-regression or ONNX-runtime output admissible through the Symbolica route? Is anyone
  waiting for a reloadable surrogate, or should `persistent` go?

<a id="cbo-133"></a>

## CBO-133 — Repeated-fit statistics over one prepared fit

- **Operation and consumers.** Parameter-estimation users who want bootstrap intervals, leave-experiments-out
  influence, objective maps over a parameter grid and joint confidence regions. Parmest-class behaviour is
  known for these from general knowledge; not checked against the pinned reference.
- **Current mechanism.** One qualified re-solve exists: `profile.rs::Chains::solve` takes an arbitrary seed
  and runs the full original acceptance (`observe_pin`, `assess_candidate`, `pin_acceptance`). It pins one
  parameter. The native path calls `Pinned::new(oracle, &[(column, value)])`, which takes a slice; the
  residual route's `solve_using` takes `Option<(usize, f64)>`. Importance and inclusion are authored in the
  source revision, so varying them means a new revision, and each study member of a fit pays full preparation.
- **Why it matters.** Prepared metadata, sparse layouts, seeding, chain teams and admission are built around
  one narrow variation. Generalizing the variation (pin set, inclusion or multiplicity vector) would serve
  several expected workflows through one owner.
- **Patterns.** LOP-12, LOP-07, LOP-09.
- **Evidence.** Read `profile.rs::{Chains::solve, ChainPolicy::run, PreparedFit::profiles}`, `residual.rs::
  solve_using`, `oracle.rs::observe_pin`, `preparation.rs::finish`, `FitOperation` comment. Interface-checked.
- **Footprint.** `transform::Pinned` in the backend and `study_variants.rs` fit handling unread.
- **Preservation constraints.** Each replicate independently assessed against original checks; failed
  replicates reported, never dropped; resampling seeds recorded; prepared-context identity includes anything
  that changes the problem. Non-unit importance withholds covariance, which is correct per replicate and
  means aggregation needs its own owner.
- **Counterevidence and uncertainty.** Sparse Gram and Hessian patterns depend on which observations are
  included (`sparse.rs::Layout` filters on `included`), and the prepared key hashes the declaration, so
  inclusion variation needs weight-only variation over a retained pattern or re-preparation. With
  experiments as the resampling unit, full re-preparation may be cheap next to the solves. Block resampling
  of grouped data is a domain decision.
- **Status.** Current mechanism (single pin); Proposed (generalization and workflows).
- **Research questions.** Can inclusion vary as weights over a retained pattern? Does a bootstrap interval
  method need order statistics beyond `statrs::statistics`?

<a id="cbo-134"></a>

## CBO-134 — Fit multistart and local-minimum detection

- **Operation and consumers.** A kinetics or thermodynamic regression with a nonconvex objective: the fitter
  wants more than one start and wants to know whether the reported optimum was beaten.
- **Current mechanism.** §19.4 refuses fit seeds and makes declared values the start. A fit can already be a
  study member, and `prepare_fit_problem_bound` overwrites a parameter value from a validated binding, so a
  study of fits over designed starts looks possible, each with full preparation (a design-to-start run was not
  traced end to end). Missing are the design-to-parameter-point bridge, reuse of one preparation, and
  comparison of optima. `workflow/fitting` has no multistart match, though `math/solves/multistart.rs` is a
  declared-seed specification elsewhere. `ChainPolicy::run` detects a pinned fit that reaches a lower objective
  than the estimate and only stops that chain with `IntervalOutcome::Stopped`; the better point is not surfaced.
- **Why it matters.** Local minima are plausible for nonconvex regressions, though not established on models here. Today the outcome comparison is manual, and a discovered
  better point is discarded.
- **Patterns.** LOP-12, LOP-09.
- **Evidence.** Read `profile.rs::ChainPolicy::run` (the `below` branch), the head of `multistart.rs`,
  `modeling.rs::prepare_fit_problem_bound`, §19.4. Interface-checked.
- **Footprint.** `multistart_tests.rs`, `math/prediction.rs` screening and `study_variants.rs` unread.
- **Preservation constraints.** Start-origin honesty (screened proposals retain their producer); every start
  independently qualified; replay of fit documents; no silent replacement of a declared start; global
  identifiability not claimed.
- **Counterevidence and uncertainty.** Most of this is reachable through a study, lowering urgency. Clustering
  distinct minima needs a local rule. A global route for factorable problems through SCIP is a different
  mechanism and was not examined. The refusal of seeds is a solver-reuse rule and no obstacle to several starts.
- **Status.** Proposed. The current limit is deliberate: seeds for fitting are refused and no global claim is made.
- **Research questions.** How would a design map to parameter points with units and bounds from the original
  declaration? What is the smallest product that reports the best of several optima honestly?

<a id="cbo-136"></a>

## CBO-136 — Measurement-error models beyond independent absolute sigma

- **Operation and consumers.** Analysts with correlated measurements (one instrument across a transient, a
  shared calibration offset), outliers, or heteroscedastic errors; plant-data users who also want
  reconciliation and gross-error tests ([CBO-137](#cbo-137)).
- **Current mechanism.** The loss is fixed and per-row: `FitOracle::residual` returns `(pred − obs)/σ` scaled
  by √importance. There is no measurement covariance, robust loss, estimated error model or errors-in-variables
  form, and §19.4 says reconciliation has no separate template. Importance cannot stand in for reweighting
  because non-unit importance withholds covariance.
- **Why it matters.** Ignoring correlation in a dense transient series gives overconfident intervals, and one
  outlier moves a quadratic fit. A whitening factor and a per-residual loss are small numerically, but they
  touch residual, Gram sparsity, Hessian, covariance and statistics owners.
- **Patterns.** LOP-12, LOP-05.
- **Evidence.** Read `oracle.rs::FitOracle::residual`, parts of `residual.rs`, the importance admission in
  `modeling.rs`, §19.4. Interface-checked.
- **Footprint.** `fitting/sparse.rs` refill for dense whitened blocks and the `pse-model` measured-attribute
  schema unread.
- **Preservation constraints.** Unknown uncertainty cannot enter a weighted fit as zero; physical conversion
  of sigma has no offsets; sparse support stays declared. The profile threshold χ²₁ holds only for a true
  likelihood, so a robust loss must withhold or relabel covariance and profile intervals.
- **Counterevidence and uncertainty.** A robust loss needs a sandwich covariance to be honest, and the LM
  route has no loss concept, so it would need IRLS or the NLP route only. Demand is plausible, not recorded.
- **Status.** Proposed. The fixed loss is the current limit.
- **Research questions.** Does a whitened-residual form keep Gram sparsity where the correlation is block
  local? Which robust loss admits an honest covariance?

<a id="cbo-138"></a>

## CBO-138 — Propagation demand: variances, bands and dense kernels

- **Operation and consumers.** `FitUncertainty.predictions` and `SensitivityRequest.propagation` callers; a
  transient fit predicting a dense time grid is the natural large case.
- **Current mechanism.** `propagate_checked` computes and stores the full `m × m` `Σ_y` with closure-indexed
  scalar loops, `O(m²p)` and sequential, though a confidence band needs the diagonal. A fit's own-prediction
  request is refused at preparation when `included² > max_cells`, and covers only the included observation
  points (`results.rs::uncertainty`), so a band on another time grid is not a fit product. Nothing adds
  observation variance for a prediction interval. `response_rank_admitted` and `covariance_values` carry hand
  triple loops, including a sparse lookup per element.
- **Why it matters.** A diagonal-only demand removes most of the cost and matches what a plot consumes; the
  quadratic refusal removes bands from long series entirely.
- **Patterns.** LOP-05, LOP-01, LOP-X006.
- **Evidence.** Read `uncertainty.rs::{propagate, propagate_checked}`, `results.rs::uncertainty`, the test
  `propagation_preserves_scalar_sum_bits_and_upstream_partiality`, the faer note in Cargo.toml. Interface-checked.
- **Footprint.** Fully inspected for the propagation route; the sparse-lookup loop in `oracle.rs` is a lead only.
- **Preservation constraints.** Scalar-sum bits are tested, with a `-0.0` start and a deliberate fold order;
  negative variance withholds rather than clamping; cancellation checkpoints; prepaid working memory; the
  `propagated_covariances` relation keeps its meaning for callers who ask for the full matrix.
- **Counterevidence and uncertainty.** The max-cells refusal is honest and bounded, so this is a capability
  limit, not a memory risk. The bit contract may rule out BLAS-style reassociation, and the inputs already come
  from runtime-dispatched faer kernels. Replacing the kernel alone is low value; the demand shape is the point.
- **Status.** Current mechanism; Proposed.
- **Research questions.** Can a diagonal-only request keep the sequential-sum contract? Where would a band on
  a caller-chosen grid get its sensitivities?

<a id="cbo-139"></a>

## CBO-139 — Multiple-shooting parameter estimation

- **Operation and consumers.** A fitter of an unstable or stiff transient system, where single-shooting fits
  commonly diverge and multiple shooting is the standard remedy.
- **Current mechanism.** Two capabilities sit side by side and do not meet. `FitOracle` integrates each
  transient experiment over its whole horizon (`oracle.rs::evaluate_experiment`). `ShootingProblem` has node
  anchors, continuity rows, parallel window teams and stitched assessment, but its objective is terminal and
  integral weighted outputs with controls as unknowns, with no measurement residuals, no Gauss-Newton form and
  no covariance path; second-derivative demand is refused.
- **Why it matters.** A dynamic parameter-estimation user with a sensitive trajectory has no route short of
  hand-built continuity. The numerical parts exist, and reuse would bring window parallelism to long fits.
- **Patterns.** LOP-12, LOP-09.
- **Evidence.** Read `shooting.rs` module doc, `ShootingOracle::evaluate`, `solve_inner`,
  `oracle.rs::evaluate_experiment` head, §19.4 experiment table. Interface-checked.
- **Footprint.** `ShootingProblem::new` and `admit_windows` unread, so a measurement reference there would be missed.
- **Preservation constraints.** The covariance rule must account for added node states (a reduced form over
  parameters, not the raw response SVD); original continuous-trajectory assessment; conservation stitching;
  a fresh IVP state per trial.
- **Counterevidence and uncertainty.** The covariance route is the hard part, since Gauss-Newton from a raw
  response does not apply unchanged once node states are variables. No demand is recorded. Adjoint limits at
  events are on [CBO-148](capability-gaps.md#cbo-148).
- **Status.** Proposed.
- **Research questions.** What is the reduced covariance when node states are eliminated? Can the fitting
  oracle accept window residuals without a second assembly path?

<a id="cbo-137"></a>

## CBO-137 — Experiment design and data reconciliation

- **Operation and consumers.** (a) An experimentalist choosing the next run that most reduces parameter
  uncertainty, from a rank-deficient or qualified fit. (b) A plant analyst reconciling measurements to balance
  constraints and detecting gross errors. They are distinct obligations sharing a primitive set.
- **Current mechanism.** `runtime.response_directions` publishes unidentified combinations, and
  `response_sensitivities` and the Gram exist; the information implied by the current data sits in
  `FitReport.responses` and singular values. Nothing evaluates candidate experiments against them: no hits for
  Fisher information, D-optimal or data reconciliation (some design-of-experiments names were not opened). The
  rolling-horizon least squares in `horizon.rs` is structurally reconciliation over a window, and reconciliation
  may already be authorable as an NLP.
- **Why it matters.** Both close loops between existing products (rank diagnosis, sweep, fit) with few additions.
  For (b) the gap is gross-error statistics and a measurement-adjustment product, not a solver.
- **Patterns.** LOP-12.
- **Evidence.** Read §19.4 and §15.5 text, `oracle.rs::response_rank_admitted`, `horizon.rs` module documentation.
  Interface-checked; footprint skimmed.
- **Footprint.** Whether a linked Rust author can fetch per-experiment response Jacobians publicly is unverified;
  candidate-sensitivity plumbing unread.
- **Preservation constraints.** Local design validity at nominal parameters is labelled (locally optimal);
  candidate sensitivities qualify under existing response rules with withholding intact; reconciled values are a
  derived product and never retroactively edit measured values.
- **Counterevidence and uncertainty.** The most speculative entry here. No scope document promises either
  capability and no consumer is named; (a) depends on the statistics in [CBO-036](#cbo-036).
- **Status.** Proposed.
- **Research questions.** Which criterion (D, A, E) and what parameter dependence? What does a flagged measurement
  do to retained provenance?

<a id="cbo-132"></a>

## CBO-132 — Design generator family: sequences and classical forms

- **Operation and consumers.** `generate_design` callers planning sweeps, initial surrogate data and multistart
  starts. The Rust API is the public path; studies consume the points.
- **Current mechanism.** `DesignKind` is Classic, Centered, Maximin, CenteredMaximin, Random or Grid. Latin
  hypercube and random forms call `egobox-doe` 0.35.1; the grid is local, preserving authored levels, order and
  endpoint bits. Maximin is best of five candidates under an admitted quadratic work bound. There is no Sobol,
  Halton or Hammersley sequence, no centroidal Voronoi design, no Plackett-Burman, fractional factorial, central
  composite or Box-Behnken design, no constrained-region design and no augmentation of existing points. The
  installed `egobox-doe` source supplies full factorial, Latin hypercube and random only. The IDAES pysmo
  sampling module also has Halton, Hammersley and CVT classes (class list read).
- **Why it matters.** Sequence designs extend without regeneration, which Latin hypercube cannot, and suit
  progressive sweeps; classical response-surface designs are what experimentalists expect. The boundary is good:
  everything reduces to a unit matrix passed to the same mapping and `PointOverlay` path.
- **Patterns.** LOP-12, LOP-03.
- **Evidence.** Read `workflow/sampling.rs`, `math/sampling.rs`, a directory listing of `egobox-doe-0.35.1/src`.
  Interface-checked; tests by name only.
- **Footprint.** Fully inspected; production callers beyond tests not found by search.
- **Preservation constraints.** Seed and attribution recorded; admitted work and byte extents before
  noninterruptible supplier calls; exact endpoint bits without clamp; exact authored levels; coordinate order;
  integer-level coordinates. Realized assignments are the identity authority, so a seed is not a portability promise.
- **Counterevidence and uncertainty.** Availability is not need: no consumer is shown. Response-surface designs have
  fixed sizes, in mild tension with a positive complete point count. A 2026-10-27 probe found `egobox-doe` designs
  bit-identical across optimization level, CPU flags and thread count on one host only.
- **Status.** Current mechanism (Latin hypercube, random and grid designs, shipped in `bee1a2271`); Proposed (more
  generators). The IDAES "Parameter sweeps" row is Implemented in the §25 coverage table.
- **Research questions.** Is there a low-discrepancy crate with scrambling, a version-stability promise and a
  dimension limit that fit? How do integer levels combine with quasi-random sequences?

<a id="cbo-035"></a>

## CBO-035 — Least-squares route: value, eligibility and dense-algebra split (weak)

- **Operation and consumers.** `FitOptimizer::Automatic`, which selects the `levenberg-marquardt` residual route
  whenever a fit is eligible, so this route is the default for its shape. Fitters with unbounded parameters.
- **Current mechanism.** `oracle/residual.rs::eligible_inputs` requires no finite parameter bounds and no general
  rows, and a dense-QR workspace under `max_cells`. Any bound on any parameter, common for positive kinetic constants,
  sends the fit to the native NLP route, including profile re-solves. The crate is a git-revision pin for nalgebra
  0.35 compatibility, with the algorithm unchanged from release 0.15 (pin reason in `[workspace.metadata.pse.pins]`).
  Production `nalgebra::` use is `residual.rs` alone (`lm_inquiry.rs` is test-only); production `ndarray::` use is
  `surrogate/library.rs` and `math/sampling.rs`. The lockfile holds nalgebra 0.33.3 and 0.35.0 and ndarray 0.16.1
  and 0.17.2; nalgebra 0.35 also arrives through other dependencies, so removing the route removes no family.
- **Why it matters.** The question that matters is whether this route beats the NLP Gauss-Newton route on the
  fits it is eligible for; if not, extending it to bounds has little value, and the dependency-family framing is
  secondary. Nothing measured answers it.
- **Patterns.** LOP-09, LOP-11, LOP-10.
- **Evidence.** Read `residual.rs::{eligible_inputs, matrix_bytes, solve_using}`, Cargo.toml pin and its comment,
  `Cargo.lock`, `use` sites by `rg`. Interface-checked.
- **Footprint.** Inspected; the profile-chain use of the route is in `profile.rs` read for [CBO-133](#cbo-133).
- **Preservation constraints.** The LM criteria policy is recorded in evidence and is part of fit identity; the
  non-MINPACK defaults are a contract; covariance bound evidence differs by route (`BoundEvidence::Residual` versus
  `Native`); log-transforming would change coordinates and statistical meaning.
- **Counterevidence and uncertainty.** The copies at these boundaries are small and deliberate, and the crates do
  not exchange matrix types. The route is library-owned by design. No strong bounded Rust method is known.
- **Status.** Current mechanism. Weak.
- **Research questions.** Is there a released least-squares crate on nalgebra 0.35 or on faer? Does a trust-region
  reflective method exist that keeps the termination mapping honest?

## Examined and not retained

- **Small dense products.** The covariance sandwich, Gauss-Newton covariance and response assembly are scalar loops
  on purpose: bit-reproducible, FMA-free summation with cancellation checkpoints, on parameter-count-sized
  matrices. The loops fix summation order but not portability, since their inputs come from faer kernels with
  runtime SIMD and FMA dispatch. The demand shape is [CBO-138](#cbo-138).
- **Source reduction.** `workflow/reduction.rs` implements a ddmin-style loop whose anchors, spans, protected bytes
  and evidence leasing dominate. Not re-read in this pass.
- **Profile-chain end-root iteration.** Once bracketed, `ChainPolicy::run` uses plain false position with a clamped
  fraction and a loose 10⁻³ tolerance. The residual is near linear for well-behaved fits, and a few-line local
  update would need no library.
- **Shooting Jacobian through an ordered map.** `ShootingOracle::evaluate` rebuilds a `BTreeMap` per trial where
  the fitting oracle uses prepared assembly slots. Integration dominates a trial, so this is a consolidation note.
- **Replacing the Gauss-Newton SVD** with `JᵀWJ` or a library covariance: the SVD is better conditioned and reuses
  the rank diagnostic. **Local grid versus `egobox-doe` full factorial:** the local form keeps authored levels, order
  and endpoint bits and uses no random stream.
- **Stepwise surrogate advance versus an argmin `Executor`:** the retained, phase-bounded, snapshotting advance is
  the reason. **Two quantile routes** (Wald normal, profile χ²₁): the same number at one degree of freedom, within
  tolerance. **Hard-coded design attribution string** naming `egobox-doe` and the generator version: a second spelling
  of a pin, harmless because realized values are the identity.
- **Prepaid byte models and owner-affine worker teams** in this workflow family are the same needs as
  [CBO-001](cross-cutting-mechanisms.md#cbo-001) and [CBO-083](execution-lifecycle-and-observation.md#cbo-083).

## Retired opportunities

<a id="cbo-033"></a>

- **CBO-033 — No producer of sampling or experimental designs.** Retired 2026-10-30: resolved by `bee1a2271`;
  `MathService::generate_design_job` and `math/sampling.rs::construct` call `egobox-doe` and a local grid in
  production code, with levels and integer coordinates handled. Successors:
  [CBO-132](#cbo-132), [CBO-034](#cbo-034). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/numerical-workflows.md#cbo-033).
