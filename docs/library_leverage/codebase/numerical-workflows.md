# Numerical workflows

This page covers fitting, uncertainty, shooting, horizon, dynamics workflows, multistart and
studies in `pse-runtime/src/workflow/` and `pse-runtime/src/math/`. The contracts are owned by
[§13](../../authoritative_design/sections/workflows-and-results.md#section-13),
[§15](../../authoritative_design/sections/numerical-execution.md#section-15) and
[§19](../../authoritative_design/sections/workflows-and-results.md#section-19). The baseline
is on the [entry page](README.md).

**System picture.** The runtime orchestrates numerical work; it is not where the numerical
kernels live. A census for local linear-algebra, statistics, root-finding, quadrature,
interpolation and finite-difference code found almost nothing generic. The heavy lifting is
done by libraries:
- **Covariance.** It comes from the KKT reduced Hessian. The fallback is Gauss–Newton over a
  faer SVD.
- **Intervals.** Wald and profile-likelihood intervals use `statrs`.
- **Fit oracle.** It uses faer sparse Gram and matmul.
- **Residual fits.** These use `levenberg-marquardt`.
- **Surrogate search.** It runs an egobox/argmin loop.
- **Shooting.** It uses IDAS forward and adjoint sensitivities.

The following are legitimately custom:
- withheld-quantity policy;
- original-coordinate qualification;
- bit-reproducible, FMA-free sums with `-0.0` accumulators;
- typed failure causes;
- memory leases.

<a id="cbo-033"></a>

## CBO-033 — No producer of sampling or experimental designs

- **Operation and consumers.** Several workflows consume point sets, but nothing produces them:
  - multistart seeds;
  - study points and sweeps;
  - profile and global checks;
  - surrogate warm starts.
- **Current mechanism / missing support.**
  - `PreparedMultistart` freezes a caller-supplied seed map (`math/solves/multistart.rs`).
  - The only generator is `StartPerturbation { seed, scale }`, a single local displacement
    delegated to the native adapter.
  - Studies take explicit points (`workflow/study.rs`).
  - A repository-wide search found no Latin hypercube, Sobol, Halton or factorial generator in
    Rust, Python or scripts.
  - `egobox-doe` is pinned and compiled, but only a test uses it
    (`math/surrogate/library.rs`).
  - [§0.2](../../authoritative_design/sections/architecture-overview.md#section-0-2) lists IDAES
    parameter sweeps as in scope.
- **Why it matters.**
  - Reproducible space-filling designs are a common need across multistart, sweeps, global
    checks and surrogates.
  - Without a shared producer, each caller will invent its own.
- **Patterns.** LOP-12 (primary), LOP-03.
- **Footprint.** The multistart and study entry points were inspected. The Python authoring
  surface for study points was searched but not read.
- **Preservation constraints.**
  - Point sets must be seed-determined and platform-independent, because points carry identity
    hashes and replay records.
  - Bounds come from the original declaration.
  - Points must be recorded as inputs in evidence.
  - Integer variables must be handled.
- **Counterevidence.**
  - Callers may deliberately supply their own designs.
  - Sweep scope is listed, but no current contract promises design generation.
- **Status.** Proposed addition, within the listed sweep scope.
- **Research questions.**
  - Is `egobox-doe` output bit-stable for a given seed across CPUs?
  - Does it handle bounds and integer variables?
  - Is there a Sobol or scrambled-sequence crate with a stability guarantee?
  - Where should designs be authored: in Rust settings or in a Python helper?
- **Library assessment correction (2026-10-27).** egobox-doe is not test-only in effect: production egobox-ego already runs its LHS for infill multistarts and TREGO starts, seeded from `SurrogateOptions.seed`. Multistart point identity is `semantic_point_key` over realized values, not the seed, so a platform difference changes identity honestly instead of corrupting replay. A probe found egobox-doe 0.35.1 designs bit-identical across opt-level, CPU flags and thread count on one host ([evidence](../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md)). See [egobox-doe](../libraries/egobox-doe/README.md) and [LOP-X006](../assessment-patterns.md#lop-x006).

<a id="cbo-034"></a>

## CBO-034 — Only first-order uncertainty propagation

- **Operation and consumers.** Propagating a fit covariance to outputs. Callers are the sensitivity
  request propagation and fit predictions.
- **Current mechanism / missing support.**
  - Only the first-order `Σ_y = J Σ_θ Jᵀ` exists (`workflow/uncertainty.rs`).
  - Profile-likelihood intervals exist for parameters (`fitting/profile.rs`) but not for
    predictions.
  - There is no sigma-point, sampling or prediction-profile route.
- **Why it matters.** First-order results are valid only near the estimate. A nonlinear check
  would give users an honest validity test. It could also reuse the study fan-out and
  [CBO-033](#cbo-033).
- **Patterns.** LOP-12.
- **Footprint.** One module plus the prediction path.
- **Preservation constraints.**
  - The withheld semantics must hold: negative variance is refused, not clamped.
  - The covariance origin must be verified.
  - Sampling must be reproducible.
  - Every sample must be a fully qualified solve.
- **Counterevidence.**
  - No consumer has asked for this yet.
  - Every sample costs a full solve.
  - A prediction profile may be the better route.
- **Status.** Proposed addition.
- **Research questions.**
  - Which consumer needs a nonlinear propagation, and at what cost budget?
  - Can a faer Cholesky plus `statrs` produce correlated draws reproducibly?
- **Library assessment correction (2026-10-27).** faer Cholesky plus statrs is reproducible on one CPU class only: faer dispatches SIMD at runtime with FMA, and statrs and `rand_distr` use the platform libm because `num-traits/std` is unified on ([LOP-X006](../assessment-patterns.md#lop-x006)).

<a id="cbo-035"></a>

## CBO-035 — Separate dense-algebra universes for least-squares and surrogate routes

- **Operation and consumers.**
  - The residual Levenberg–Marquardt route runs on nalgebra 0.35.
  - The egobox surrogate runs on ndarray.
  - Everything else runs on faer.
- **Current mechanism.**
  - `levenberg-marquardt` is pinned to a git revision for nalgebra 0.35 compatibility.
  - `statrs` features are constrained so that a second nalgebra is avoided.
  - nalgebra appears only in `fitting/oracle/residual.rs` and `lm_inquiry.rs`.
  - ndarray appears only in `surrogate/library.rs`.
  - The LM route is narrow: unbounded, no general rows, and response derivatives only.
- **Why it matters.**
  - Each extra family adds pin moves and copy boundaries.
  - It carries a dependency family for a small, gated route.
- **Patterns.** LOP-09, LOP-11, LOP-10.
- **Footprint.** About three production files for LM and one for egobox.
- **Preservation constraints.**
  - The LM criteria policy is recorded in evidence and is part of fit identity.
  - The non-MINPACK defaults are a contract.
- **Counterevidence.**
  - LM is library-owned by design.
  - The boundaries are copy points, not mixed algebra.
  - Removing LM would remove a distinct solver choice.
- **Status.** Current promise with a maintenance question.
- **Research questions.**
  - Is there a released LM crate on nalgebra 0.35, or a faer-based LM or trust-region method?
  - Does the LM route outperform the NLP Gauss–Newton route for the fits it is eligible for?
  - Could a bounded method lift the restriction to unbounded problems?
- **Library assessment correction (2026-10-27).** The premise "statrs features avoid a second nalgebra" is false: `Cargo.lock` resolves nalgebra 0.33.3 through statrs because egobox-moe enables statrs' default features. nalgebra 0.35 is also a non-optional dependency of `diffsol-la`, `num-dual` and `feos`/`quantity`, so removing the LM route would remove only pse-runtime's direct use.

<a id="cbo-036"></a>

## CBO-036 — Fit statistics assume absolute sigma; no estimated-variance path

- **Operation and consumers.** Fit reports, namely intervals and prediction bands.
- **Current mechanism / missing support.**
  - Wald intervals use normal quantiles, and profiles use χ² thresholds (`fitting/covariance.rs`,
    `profile.rs`).
  - Unknown-sigma fits are refused (see the `lm_inquiry.rs` tests).
  - There is no Student-t, reduced-χ² or residual-variance scaling path, although `statrs`
    already provides the needed distributions.
- **Why it matters.** Fits with estimated measurement variance currently have no statistical
  route at all.
- **Patterns.** LOP-12, LOP-03.
- **Footprint.** Two modules.
- **Preservation constraints.**
  - The documented statistical model.
  - Refusal rather than silent rescaling.
- **Counterevidence.** This appears to be deliberate current scope. The pattern match is weak.
- **Status.** Proposed addition.
- **Research questions.** Does any consumer need estimated-variance fits?
- **Library assessment correction (2026-10-27).** Estimated-σ fits are refused at fit preparation, so they cannot be prepared today.

## Examined and not retained

- **Small dense products.** These are the covariance sandwich, the Gauss–Newton covariance and
  response assembly in `uncertainty.rs`, `fitting/covariance.rs` and `fitting/oracle.rs`. They
  are written as scalar loops on purpose: bit-reproducible, FMA-free summation with
  cancellation checkpoints. faer's blocked kernels reorder sums. The matrices are
  parameter-count sized. The opportunity would need a deterministic sequential reduction mode,
  and none is known.
- **Source reduction.** `workflow/reduction.rs` implements a ddmin-style loop. Its anchors,
  spans, protected bytes and evidence leasing dominate, and no fitting library is known.
