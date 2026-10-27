# Numerica

Realizations considered: installed `numerica` 3.0.1 (crates.io, released 2026-09-29, the newest
release), consumed through Symbolica's `domains`/`tensors` re-exports. The unreleased Symbolica
`main` at `bfb481cdc` carries Numerica in `lib/numerica`.
Research date 2026-10-27.
Sources:
- installed `~/.cargo/registry/src/*/numerica-3.0.1/src/{tensors/matrix.rs, tensors/sparse.rs,
  domains/float/interval.rs}`;
- `git log v3.0.1..HEAD -- lib/numerica/src/tensors` and `git show 9922a41` in the Symbolica clone.

## Types

### numerica/exact-matrix-algebra
- **Need it serves:** linear algebra over exact fields such as ℚ, finite fields and
  rational-function fields.
- **Recognizable code shapes:**
  - hand-written Gaussian elimination over a rational type;
  - sparse row maps of rationals with local pivot rules.
- **Mechanisms and APIs:**
  - Dense `Matrix<F>`: `solve`, `solve_any`, `solve_fraction_free`, `det`, `det_in_place`, `inv`,
    `row_reduce`, `partial_row_reduce(_fraction_free)`, `rank`, `gram_schmidt`,
    `basis_reduction`, `back_substitution(_fraction_free)`.
  - Sparse `SparseMatrix<F>` (CSR): `solve`, `solve_parallel`, `det`, `inv`, `from_triplets`,
    `row_weight`, `sort_rows_by_pivot`.
- **Obligations discharged:** exact solve, determinant, rank and inverse over any `Field`.
- **Residual caller obligations:** symmetric factorisation, inertia, semidefiniteness decisions and
  factor witnesses.
- **Semantic conditions:** exact arithmetic with no rounding. Elimination order is internal, so the
  caller cannot choose a symmetric minimum-degree pivot.
- **Version / build conditions:** 3.0.1. On `main` the only `tensors` changes since 3.0.1 are
  `6c99925` (reject inconsistent sparse systems) and `03b2705` (determinant sign fix).
- **Limits and negative results:**
  - No LDLᵀ, Cholesky, Bunch–Kaufman, inertia or definiteness API at 3.0.1 or `main`.
  - Search scope: function names in `tensors/matrix.rs` and `tensors/sparse.rs`, plus a grep for
    `ldl|cholesky|chol|inertia|signature|definite` across `lib/numerica/src` and `src/` at `main`.
  - `det`/`rank` on leading minors could decide positive definiteness (Sylvester) at greatly higher
    cost. They cannot decide semidefiniteness or give a witness.
- **Evidence:** Interface-checked (source listing and grep).

### numerica/ball-enclosures
- **Need it serves:** rigorous enclosures of real/complex values.
- **Recognizable code shapes:**
  - local interval arithmetic types;
  - forward interval evaluation of expression DAGs.
- **Mechanisms and APIs:**
  - `RealBall { center: Float, radius: Float }` and `ComplexBall = Complex<RealBall>` in
    `domains/float/interval.rs`.
  - Unreleased `main` commit `9922a41` adds `EvaluationDomain` impls so an `ExpressionEvaluator`
    can run over balls.
- **Obligations discharged:** directed-rounding enclosures for `+ − × ÷` and inversion; for complex
  balls also nonnegative integer powers.
- **Residual caller obligations:**
  - transcendental functions;
  - reverse (backward) propagation;
  - midpoint-radius versus endpoint semantics.
- **Semantic conditions:** the source states that operations other than the listed arithmetic "are
  not currently certifying". Ball (midpoint-radius) enclosures differ from endpoint intervals near
  zero and for one-sided bounds.
- **Version / build conditions:** the type is at 3.0.1; the evaluator domain is unreleased.
- **Limits and negative results:** with uncertified transcendental functions, ball evaluation of
  `exp`/`log`/`sin` is not a sound substitute for certified forward interval evaluation.
- **Evidence:** Interface-checked (doc comments in source).

## Current applicability

### [CBO-025](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-025) — Exact rational LDLᵀ
- **Types:** numerica/exact-matrix-algebra.
- **Realization:** installed 3.0.1; nothing newer is released.
- **Conclusion:** ruled out here. Numerica has no symmetric exact factorisation, and leading-minor
  determinants cannot decide semidefiniteness or give the Gram factors or the
  negative-direction witness the CBO must preserve.
- **What it would discharge / what remains local:** nothing beyond exact field arithmetic, which
  the local kernel already uses through `symbolica::domains::rational::Rational`.
- **Prerequisites, configuration or extension:** an upstream exact LDLᵀ with symmetric pivoting
  and zero-pivot handling.
- **Semantic gaps against the CBO's preservation constraints:**
  - no `Indefinite`/`GramFactors` diagnostics;
  - no budget or cancellation hook.
- **Unsettled premises and what would settle them:** none material.
- **Evidence:** Interface-checked.

### [CBO-024](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-024) — Interval views (secondary)
- **Types:** numerica/ball-enclosures.
- **Realization:** the evaluator domain is upgrade-route only (unreleased).
- **Conclusion:** ruled out at present. Certification stops at arithmetic, and the CBO's
  interval/FBBT consumers need certified elementary functions and reverse propagation.
- **What it would discharge / what remains local:** nothing at present.
- **Prerequisites, configuration or extension:** a release with certified transcendental balls.
- **Semantic gaps against the CBO's preservation constraints:** the evaluated shape is the
  optimised one (see symbolica/evaluator-instruction-interchange).
- **Unsettled premises and what would settle them:** whether upstream intends certified
  transcendentals. The release notes of the next Symbolica release would settle this.
- **Evidence:** Interface-checked.
