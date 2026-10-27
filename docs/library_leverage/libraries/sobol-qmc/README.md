# sobol-qmc (and sobol 1.0.2)

Realizations considered:
- `sobol-qmc` 2.5.1 (released 2025-10-28; it calls itself a fork of `sobol`).
- `sobol` 1.0.2 (released 2020-12-27, unmaintained since).
- Neither is in the workspace lock.

Research date 2026-10-27.

Sources:
- Downloaded crate source `sobol-qmc-2.5.1/{Cargo.toml,README.md,src/lib.rs,src/type_support.rs,src/params/}`.
- crates.io metadata for both crates.

## Types

### sobol-qmc/unscrambled-sobol-sequence
- **Need it serves:** A deterministic, unscrambled Sobol low-discrepancy sequence with
  Joe–Kuo direction numbers.
- **Recognizable code shapes:** Hand-written Gray-code Sobol generators.
- **Mechanisms and APIs:**
  - `Sobol::<f64>::new(dims, &JoeKuoD6::STANDARD | EXTENDED)` returns
    `Result<_, SobolError>`.
  - `new_with_resolution(dims, params, Some(bits), render)`.
  - It is an `Iterator<Item = Vec<T>>` with `nth` skipping.
  - Renderers: `UnitRender`, plus `GaussianRender(statrs::Normal)` and
    `MultiDimGaussianRender`, both of which apply `statrs` `inverse_cdf`.
- **Obligations discharged:** The Antonov–Saleev Gray-code recursion and the direction
  numbers.
- **Residual caller obligations:**
  - Randomization: there is no scrambling and no seed, so independent replicates require
    a caller-side digital shift.
  - Skipping the leading all-zero point if unwanted.
  - Bounds and integer levels.
- **Semantic conditions:**
  - **Unit render.** It is exact: a `u64` with at most 53 significant bits, cast to
    `f64` and divided by 2^64, so it is platform-independent.
  - **Gaussian render.** It goes through `statrs` `inverse_cdf`, which uses std libm, so
    it is platform-libm dependent.
- **Version / build conditions:**
  - It depends on `statrs` ^0.18 with default features, bringing nalgebra 0.33 and
    rand 0.8. Both already exist in this lock through egobox.
  - It has a build script that decompresses the direction-number data.
- **Limits and negative results:**
  - No scrambling, no seed and no Halton.
  - `sobol` 1.0.2 predates the fork and was not inspected further.
- **Evidence:** Interface-checked: `src/lib.rs` (`update`, `render_next`, `Render`) and
  `type_support.rs` (f64 render, `MAX_RESOLUTION`) at 2.5.1.

## Current applicability

### [CBO-033](../../../library_leverage/codebase/numerical-workflows.md#cbo-033) — No producer of sampling or experimental designs
- **Types:** `sobol-qmc/unscrambled-sobol-sequence`.
- **Realization:** A new dependency at 2.5.1.
- **Conclusion:** fits with conditions. It is deterministic and exact for unit-cube
  points. With no seed, every caller gets the same sequence, so "seed-determined"
  degenerates to "index-determined" unless local scrambling is added. `sobol_burley`
  already supplies seeded scrambling.
- **What it would discharge / what remains local:**
  - **Discharged.** The sequence.
  - **Local.** Scrambling or shift, bounds and integers.
- **Prerequisites, configuration or extension:** As above.
- **Semantic gaps against the CBO's preservation constraints:** No seed or replicate
  independence.
- **Unsettled premises and what would settle them:** None beyond the QMC-versus-LHS
  choice.
- **Evidence:** Interface-checked. The application is Proposed.
