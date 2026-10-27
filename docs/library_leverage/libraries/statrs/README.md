# statrs

Realizations considered:
- **Installed.** `statrs` 0.18.0. The workspace declares `default-features = false`, but
  egobox-moe enables the defaults (`nalgebra` 0.33 and `rand` 0.8) through feature
  unification.
- **Upgrade route.** `statrs` 0.19.1 (released 2026-08-11) moves its optional
  dependencies to `nalgebra` ^0.35 and `rand` ^0.10.
- **Coupling.** egobox-moe, including 0.36.2, still requires `statrs` ^0.18 with
  defaults. A workspace upgrade to 0.19 would therefore add a second statrs version,
  not replace the first.

Research date 2026-10-27.

Sources:
- Installed `statrs-0.18.0/Cargo.toml` (features) and
  `src/distribution/{students_t,fisher_snedecor,chi_squared,gamma,normal,multivariate_normal}.rs`.
- crates.io dependency metadata for 0.19.1.
- Context7 `/statrs-dev/statrs` was resolved but not needed beyond source.

## Types

### statrs/inverse-cdf-quantiles
- **Need it serves:** Quantiles for interval construction: normal, χ², Student-t and
  Fisher–Snedecor critical values.
- **Recognizable code shapes:**
  - Hard-coded z or t tables.
  - Hand-written bisection on a CDF.
  - Intervals that always use normal or χ² thresholds.
- **Mechanisms and APIs:** `ContinuousCDF::inverse_cdf(p)` on:
  - `Normal::standard()`;
  - `ChiSquared::new(k)`, which delegates to `Gamma`'s bisection inverse;
  - `StudentsT::new(location, scale, freedom)`, through `beta::inv_beta_reg(ν/2, ½, 2·min(p, 1−p))`;
  - `FisherSnedecor::new(d1, d2)`, through `inv_beta_reg(d1/2, d2/2, p)`.
- **Obligations discharged:** The special-function inversions. All of these are
  available without the `nalgebra` and `rand` features.
- **Residual caller obligations:**
  - **Statistical model.** The statistical model is entirely local: the degrees of
    freedom (n − p, after withheld or excluded observations and active bounds), the
    residual-variance estimator, and which threshold applies. The usual choices are a
    t-quantile for marginal Wald intervals, `F(1, n−p)` or `t²` for profile-likelihood
    thresholds under estimated σ², and `p·F(p, n−p)` for joint regions.
  - **Argument checks.** Validate `p ∈ [0, 1]` and `ν > 0` before calling.
    `StudentsT::inverse_cdf` asserts its range and `FisherSnedecor::inverse_cdf` panics
    outside `[0, 1]`.
- **Semantic conditions:**
  - **Precision.** The results come from iterative or series special functions using
    std libm (`ln`, `exp`, `sqrt`). They are deterministic on one platform but not
    guaranteed bit-identical across libms.
  - **Panics.** Out-of-range arguments panic, which unwinds; they do not return errors.
- **Version / build conditions:** Present in 0.18.0. Not re-verified for 0.19.1.
- **Limits and negative results:**
  - No regression or fit-statistics layer: no residual-variance scaling, covariance
    scaling or reduced-χ² helpers.
  - No noncentral distributions were inspected.
- **Evidence:** Interface-checked: the `inverse_cdf` bodies in the installed 0.18.0
  sources.

### statrs/multivariate-normal-sampling
- **Need it serves:** Correlated normal draws and densities from a mean and covariance.
- **Recognizable code shapes:** Hand-written Cholesky followed by a normal-vector
  transform.
- **Mechanisms and APIs:**
  - `MultivariateNormal::new(mean: Vec<f64>, cov: Vec<f64>)` and
    `new_from_nalgebra(DVector, DMatrix)`. Both run a nalgebra `Cholesky` and return
    `MultivariateNormalError::CholeskyFailed` for a matrix that is not positive definite.
  - Sampling uses `rand::Distribution`, behind the `rand` feature.
- **Obligations discharged:** The factorization, the transform and the density.
- **Residual caller obligations:**
  - Handle positive-semidefinite or singular covariances (Cholesky refuses them).
  - Convert into nalgebra types.
  - Provide a seeded RNG.
- **Semantic conditions:**
  - **Type universe.** It is tied to statrs' nalgebra version: 0.33 at 0.18.0, 0.35 at
    0.19.1.
  - **Sampling path.** Normal draws use statrs' own ziggurat, which calls libm in its
    tails.
- **Version / build conditions:** It requires the `nalgebra` feature, which is on here
  only by unification from egobox-moe.
- **Limits and negative results:** Singular covariances are refused rather than handled
  through a square root.
- **Evidence:** Interface-checked: `multivariate_normal.rs` constructors and error enum
  at 0.18.0.

## Current applicability

### [CBO-036](../../../library_leverage/codebase/numerical-workflows.md#cbo-036) — Fit statistics assume absolute sigma; no estimated-variance path
- **Types:** `statrs/inverse-cdf-quantiles`.
- **Realization:** Installed 0.18.0. `StudentsT` and `FisherSnedecor` need no extra
  feature.
- **Conclusion:** fits with conditions. The library discharges only the quantile
  evaluation, which is about one call per interval. Everything that makes an
  estimated-variance fit statistically sound stays local:
  - the variance model;
  - the degrees-of-freedom accounting;
  - covariance scaling;
  - withheld-quantity semantics;
  - typed refusal.
- **What it would discharge / what remains local:**
  - **Discharged.** t and F critical values.
  - **Local.**
    - s² = SSR/(n−p), with the definition of n and p under exclusions and active bounds.
    - Scaling of the covariance and its origin record.
    - Interval and profile threshold choice.
    - The decision whether to accept a declaration that lacks standard deviations. Such
      declarations are currently refused at preparation; the
      `research_calibration_unknown_sigma_refuses_weighting_and_preserves_exclusions`
      test expects the error "positive difference-unit standard deviations".
- **Prerequisites, configuration or extension:**
  - Guard the argument domain before calling to avoid the panics.
  - Validate dof > 0, refusing when n ≤ p.
- **Semantic gaps against the CBO's preservation constraints:**
  - "Refusal rather than silent rescaling" is a local policy. The library is neutral
    toward it.
  - A t-quantile with large dof approaches but does not equal the normal quantile, so
    results differ from the absolute-σ route by design.
- **Unsettled premises and what would settle them:**
  - The consumer need, which the CBO itself flags as weak.
  - The statistical specification: which model owner defines estimated-σ fits.
  - A behavioral reference exists in scipy `curve_fit(absolute_sigma=False)` and similar
    tools.
- **Evidence:** Interface-checked: statrs sources, `fitting/covariance.rs` (`wald` uses
  `Normal::standard().inverse_cdf`), `fitting/profile.rs` (`ChiSquared::new(1.0)`) and the
  `lm_inquiry.rs` unknown-sigma test. The application is Proposed.

### [CBO-034](../../../library_leverage/codebase/numerical-workflows.md#cbo-034) — Only first-order uncertainty propagation
- **Types:** `statrs/multivariate-normal-sampling` and `statrs/inverse-cdf-quantiles`
  (normal inverse CDF for QMC to normal).
- **Realization:** Installed. The nalgebra 0.33 universe exists only through egobox.
- **Conclusion:** ruled out here for `MultivariateNormal` sampling, but `Normal`
  `inverse_cdf` fits with conditions.
  - `MultivariateNormal` would make pse-runtime depend on the nalgebra 0.33 universe that
    it currently gets only by accident through egobox.
  - Its sampling uses a libm-tailed ziggurat and an opaque RNG path.
  - It refuses singular PSD covariances, which the repository's PSD admission explicitly
    permits.
  - `Normal::standard().inverse_cdf(u)` is usable for mapping uniform or QMC points to
    normal deviates.
- **What it would discharge / what remains local:** The normal transform only. The
  covariance square root (see `faer.md`), the sampling design and the solves stay local.
- **Prerequisites, configuration or extension:** Keep the covariance factorization on the
  repository's own linear-algebra route.
- **Semantic gaps against the CBO's preservation constraints:** "Reproducible" holds per
  platform. The `inverse_cdf` uses std libm.
- **Unsettled premises and what would settle them:** Whether the reproducibility contract
  is cross-platform regeneration or recorded replay.
- **Evidence:** Interface-checked. The application is Proposed.

### [CBO-035](../../../library_leverage/codebase/numerical-workflows.md#cbo-035) — Separate dense-algebra universes
- **Types:** Feature and version conditions above.
- **Realization:** Installed 0.18.0, versus the 0.19.1 upgrade.
- **Conclusion:** unsettled. A correction to the CBO premise:
  - Restricting statrs features does not avoid a second nalgebra. `Cargo.lock` resolves
    `nalgebra` 0.33.3, solely through `statrs` 0.18.0, because egobox-moe enables statrs
    defaults. The workspace manifest comment already records this.
  - Upgrading pse-runtime to statrs 0.19.1 would not remove 0.33 while egobox-moe
    requires ^0.18.
- **What it would discharge / what remains local:** n/a.
- **Prerequisites, configuration or extension:** An egobox upstream change, either
  `statrs` with `default-features = false` or `statrs` 0.19.
- **Semantic gaps against the CBO's preservation constraints:** None.
- **Unsettled premises and what would settle them:** Upstream willingness.
- **Evidence:** Interface-checked: `Cargo.lock` edges, the workspace `Cargo.toml`
  comment, and crates.io metadata for egobox-moe 0.36.2 and statrs 0.19.1.
