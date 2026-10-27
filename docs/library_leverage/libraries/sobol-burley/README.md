# sobol_burley

Realizations considered:
- **Not in the workspace lock.** `sobol_burley` 0.5.0 (released 2023-07-05, the latest on
  crates.io) is present in the local Cargo registry cache only.
- **Upgrade route.** Adopting it would be a new dependency at 0.5.0. Its only optional
  feature is `simd`, which is on by default.

Research date 2026-10-27.

Sources:
- Installed registry source `sobol_burley-0.5.0/src/{lib.rs,parts.rs}`, `CHANGELOG.md`
  and `Cargo.toml`.
- crates.io metadata.
- Brent Burley, "Practical Hash-based Owen Scrambling" (JCGT 2020), as cited by the crate.

## Types

### sobol-burley/seeded-owen-scrambled-sobol
- **Need it serves:** A seeded, randomized low-discrepancy (QMC) point set computed in
  pure integer arithmetic. Different seeds give statistically independent scrambles.
- **Recognizable code shapes:**
  - Sobol or Halton generators written by hand.
  - Pseudo-random Monte Carlo used where QMC would converge faster.
  - Needs for "independent replicate designs".
- **Mechanisms and APIs:**
  - `sample(sample_index: u32, dimension: u32, seed: u32) -> f32` and `sample_4d(...)`.
  - `parts::{sobol_rev, owen_scramble_rev, hash, u32_to_f32_norm, …}` exposes the
    integer pipeline. A caller can take the scrambled `u32` and convert it to `f64`
    exactly (`(n as f64) / 2^32`), avoiding the f32 rounding.
  - Dimensions use Joe–Kuo direction numbers, up to 256 dimensions (`NUM_DIMENSIONS`).
  - The seed shuffles the index (Owen scramble of the reversed index) and, separately,
    sets each dimension's scramble.
- **Obligations discharged:**
  - The Sobol construction.
  - Hash-based nested-uniform (Owen) scrambling.
  - Seeded independent replicates.
  - Random access by index.
- **Residual caller obligations:**
  - **Length limit.** The maximum length is 2^16 points. Debug builds panic beyond it;
    release builds return unspecified values.
  - **Output precision.** Output is `f32`. Use `parts` for a full-precision `u32`.
  - Bound mapping, integer-level mapping and dimension count are the caller's.
- **Semantic conditions:**
  - **Determinism.** It is fully integer-defined. The SIMD path (`Int4`) computes the
    same integer functions, so results are platform-independent by construction.
  - **No global state.** Results are a pure function of `(index, dim, seed)`.
- **Version / build conditions:** No dependencies. The `simd` feature uses `unsafe` SSE
  on x86-64.
- **Limits and negative results:**
  - It is graphics-oriented: f32, 2^16 points and 256 dimensions. No discrepancy metric
    is provided.
  - The last release was 2023, and the CHANGELOG lists no 0.5.0 entry.
- **Evidence:** Interface-checked: `lib.rs` module docs, the `sample` implementation,
  `parts.rs` exports and `Cargo.toml` at 0.5.0. Not executed.

## Current applicability

### [CBO-033](../../../library_leverage/codebase/numerical-workflows.md#cbo-033) — No producer of sampling or experimental designs
- **Types:** `sobol-burley/seeded-owen-scrambled-sobol`.
- **Realization:** A new dependency at 0.5.0.
- **Conclusion:** fits with conditions. It is the only inspected Rust QMC route that is
  both scrambled and seeded and that rests on integer arithmetic alone. Its limits
  (2^16 points, 256 dimensions, f32 unless built from `parts`) are acceptable for
  multistart, sweep and global-check scales.
- **What it would discharge / what remains local:**
  - **Discharged.** The low-discrepancy construction and scrambling.
  - **Local.**
    - Converting `u32` to `f64` exactly.
    - Bound mapping and refusal.
    - Integer levels.
    - Seed recording.
- **Prerequisites, configuration or extension:** Compose `parts::sobol_rev`,
  `owen_scramble_rev` and `hash` exactly as `sample()` does, but keep the `u32`.
- **Semantic gaps against the CBO's preservation constraints:** None for determinism.
  Integer variables and bounds remain local.
- **Unsettled premises and what would settle them:**
  - Whether QMC is wanted at all versus LHS. That is a comparative-review choice.
  - Maintenance posture of a 2023 crate.
- **Evidence:** Interface-checked. The application is Proposed.

### [CBO-034](../../../library_leverage/codebase/numerical-workflows.md#cbo-034) — Only first-order uncertainty propagation
- **Types:** `sobol-burley/seeded-owen-scrambled-sobol`.
- **Realization:** A new dependency.
- **Conclusion:** fits with conditions. It is a candidate base for randomized-QMC
  sampling propagation (uniform points mapped through an inverse normal CDF and a
  covariance factor). Normal mapping and factorization are separate types (`statrs`,
  `faer`), each with its own determinism conditions.
- **What it would discharge / what remains local:**
  - **Discharged.** Uniform QMC points.
  - **Local.** Normal transform, covariance square root, the per-sample solve fan-out and
    the statistics.
- **Prerequisites, configuration or extension:** As in CBO-033.
- **Semantic gaps against the CBO's preservation constraints:** Inverse-CDF mapping
  reintroduces libm dependence (see `statrs.md`).
- **Unsettled premises and what would settle them:** The required sample counts.
  Sigma-point routes need 2p+1 solves, not QMC.
- **Evidence:** Interface-checked. The application is Proposed.
