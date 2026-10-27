# rand family (rand, rand_core, rand_xoshiro, rand_pcg, rand_chacha, rand_distr) and num-traits float math

Realizations considered: installed versions per `Cargo.lock`:
- **rand.** 0.8.8 arrives through egobox, ndarray-rand, linfa and statrs; 0.9.5 through
  argmin, faer, symbolica and proptest; 0.10.3 through rustworkx-core and quinn.
- **rand_core.** 0.6.4, 0.9.5 and 0.10.1.
- **rand_xoshiro.** 0.6.0 (egobox) and 0.7.0 (argmin).
- **rand_chacha.** 0.3.1 and 0.9.0.
- **rand_pcg.** 0.10.2.
- **rand_distr.** 0.4.3, 0.5.1 and 0.6.0.
- **num-traits.** 0.2.19, with `libm` 0.2.16 in the lock.

pse-runtime declares no direct `rand` dependency. Research date 2026-10-27.

Sources:
- The rust-random book chapter "Reproducibility" (`src/crate-reprod.md`, master), read
  through Context7 `/rust-random/book` and the raw file.
- Installed sources: `rand-0.8.8/src/seq/mod.rs` (`gen_index`, `shuffle`),
  `rand-0.8.8/src/distributions/uniform.rs` (`UniformFloat`),
  `rand_core-0.6.4/src/lib.rs` (`seed_from_u64` PCG32 expansion),
  `rand_xoshiro-0.6.0/src/xoshiro256plus.rs` (SplitMix64 `seed_from_u64`),
  `rand_distr-{0.4.3,0.5.1,0.6.0}/Cargo.toml` (`std_math` feature) and
  `num-traits-0.2.19/src/float.rs` (std/libm cfg precedence).

## Types

### rand/portable-seeded-streams
- **Need it serves:** A seed-determined random stream whose values do not depend on
  platform, so that a seed recorded in evidence regenerates the same draws.
- **Recognizable code shapes:**
  - `seed_from_u64`.
  - `StdRng`/`SmallRng` used for "reproducible" runs.
  - `thread_rng`/`rng()` in code that claims replayability.
- **Mechanisms and APIs:**
  - Named algorithm RNGs:
    - `rand_xoshiro::{Xoshiro256Plus, Xoshiro256PlusPlus, …}`, seeded via SplitMix64 in
      `seed_from_u64`;
    - `rand_pcg::{Pcg64, Pcg64Mcg}`;
    - `rand_chacha::{ChaCha8Rng, ChaCha12Rng, ChaCha20Rng}`, which also supports
      `set_stream` and `set_word_pos` for independent substreams.
  - `rand_core::SeedableRng::{seed_from_u64, from_seed}`. The default `seed_from_u64`
    expands the seed with PCG32 in little-endian order.
  - `Rng::gen_range` (0.8) / `random_range` (0.9+) on `usize` samples `u32` where it can
    (`gen_index` in 0.8 `seq`), which keeps shuffles portable between 32- and 64-bit
    targets.
- **Obligations discharged:**
  - A stream algorithm with a stated portability policy.
  - Seed expansion.
  - Uniform float generation by mantissa bits followed by a separate multiply and add.
  - Fisher–Yates shuffle.
- **Residual caller obligations:**
  - Choose a named PRNG. `StdRng` and `SmallRng` are declared non-portable, and from
    rand 0.10/1.0 they may change values in any release.
  - Pin exact versions. Minor releases may make value-breaking changes to portable items,
    which must be noted in the CHANGELOG.
  - Never sample `usize` directly.
  - Keep one stream per independent consumer, by seed derivation or ChaCha streams, when
    work is parallel.
- **Semantic conditions:** The policy states that portable items should give the same
  results across platforms and across patch releases. Floats are the stated exception (see
  the next type).
- **Version / build conditions:**
  - The policy text names rand 0.10/1.0 as the point from which non-portable items may
    change values in any release. Up to 0.9, non-portable items could not change values in
    patch releases.
  - Several major lines coexist in one build here (0.8/0.9/0.10). Each line is its own
    value universe.
- **Limits and negative results:**
  - There is no cross-minor-version value guarantee.
  - The policy is a stated expectation, tested with reference vectors upstream. It is not
    a formal compatibility contract.
- **Evidence:** Interface-checked: the book chapter and the installed sources listed
  above.

### rand-distr/float-transcendental-portability
- **Need it serves:** Normal, Student-t and other continuous draws that must be
  bit-reproducible across platforms.
- **Recognizable code shapes:**
  - `StandardNormal`, `Normal::new` and `rand_distr::*` sampling.
  - `statrs` distributions implementing `rand::Distribution`.
  - Box–Muller written by hand.
- **Mechanisms and APIs:**
  - `rand_distr` uses `num_traits::Float` for `exp`, `ln` and `cos`. `num-traits` selects
    std (platform libm) when its `std` feature is on, and the pure-Rust `libm` crate only
    when `std` is off and `libm` is on (`#[cfg(all(not(feature = "std"), feature =
    "libm"))]`).
  - `rand_distr`'s `std_math` feature merely enables `num-traits/std`.
  - Ziggurat normal sampling uses only multiply and compare on its fast path. Its tail and
    wedge branches call `exp` and `ln`.
- **Obligations discharged:** The sampling algorithms themselves.
- **Residual caller obligations:**
  - For cross-platform bit stability, either avoid transcendental calls or call
    `libm::{exp, log, …}` explicitly.
  - Alternatively, record realized draws as inputs rather than relying on regeneration.
- **Semantic conditions:**
  - **Feature unification decides the math source.** Any crate in the build enabling
    `num-traits/std`, which is a default feature, makes every `num_traits::Float`
    transcendental call the platform libm.
  - **Policy text.** The rust-random book states that `rand_distr` float distributions
    "may not be portable" and that it "prefers libm".
- **Version / build conditions:** Applies to `rand_distr` 0.4.3, 0.5.1 and 0.6.0 as
  installed. All three expose `std_math = ["num-traits/std"]` and depend on `num-traits`
  with `libm`.
- **Limits and negative results:**
  - Only the cfg precedence was inspected.
  - The libm results were not compared across platforms here.
- **Evidence:** Interface-checked: `num-traits-0.2.19/src/float.rs` cfg blocks,
  `rand_distr` Cargo.toml features, and the book "Portability of floats" section.

## Current applicability

### [CBO-033](../../../library_leverage/codebase/numerical-workflows.md#cbo-033) — No producer of sampling or experimental designs
- **Types:** `rand/portable-seeded-streams`.
- **Realization:** Installed. Use `rand_xoshiro` 0.6.0 / `rand` 0.8.8 inside egobox, or a
  direct `rand_chacha` or `rand_pcg` dependency for a local design generator.
- **Conclusion:** fits with conditions. Named, seedable, portable PRNGs exist at the pins.
  Value stability holds only while `rand`, `rand_core` and the PRNG crate stay at exact
  versions.
- **What it would discharge / what remains local:**
  - **Discharged.** The stream.
  - **Local.**
    - Seed derivation per design or point.
    - Recording `(algorithm, crate version, seed)`, or the realized points, in evidence.
    - Treating a family bump as a design-changing move.
- **Prerequisites, configuration or extension:** Name the PRNG explicitly. Never use
  `StdRng`, `SmallRng` or the entropy-default egobox constructors.
- **Semantic gaps against the CBO's preservation constraints:** "Platform-independent"
  holds within one version set. It does not hold across rand minor versions.
- **Unsettled premises and what would settle them:** Whether evidence keys on the seed or
  on the realized points (see CBO-033 in egobox-doe.md).
- **Evidence:** Interface-checked. The application is Proposed.

### [CBO-034](../../../library_leverage/codebase/numerical-workflows.md#cbo-034) — Only first-order uncertainty propagation
- **Types:** `rand/portable-seeded-streams` and
  `rand-distr/float-transcendental-portability`.
- **Realization:** Installed.
- **Conclusion:** fits with conditions. Reproducible correlated sampling is achievable on
  one platform. Cross-platform bit identity of normal draws is not guaranteed by any
  installed route, because `num-traits/std` is unified on.
- **What it would discharge / what remains local:**
  - **Discharged.** Uniform and normal draws.
  - **Local.**
    - The normal-draw source: ziggurat through std libm, an inverse CDF of uniform or QMC
      points via `statrs` (std libm), or explicit `libm` calls.
    - Recording the realized parameter samples as inputs.
- **Prerequisites, configuration or extension:** None for a same-platform route. For
  portable draws, call the `libm` crate explicitly in local code.
- **Semantic gaps against the CBO's preservation constraints:** "Sampling must be
  reproducible" is satisfied same-platform. Cross-platform reproducibility needs either
  explicit `libm` use or recorded samples.
- **Unsettled premises and what would settle them:** Whether the reproducibility contract
  is "regenerate from seed anywhere" or "replay from recorded points". The owner of
  evidence and replay semantics settles this.
- **Evidence:** Interface-checked. The application is Proposed.
