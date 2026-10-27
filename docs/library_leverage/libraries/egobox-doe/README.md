# egobox-doe (with egobox-moe mixed-integer sampling)

Realizations considered:
- **Installed.** `egobox-doe` 0.35.1, a direct dev-dependency that is also compiled in
  production through `egobox-ego` 0.38.1, `egobox-gp` 0.36.4 and `egobox-moe` 0.35.5.
  `egobox-moe` 0.35.5 supplies `MixintSampling`.
- **Upgrade routes.** `egobox-doe` 0.35.2 (released 2026-10-12); its `src/` is identical
  to 0.35.1 except for line endings. `egobox-moe` 0.36.2 (released 2026-10-12) adds
  equal-probability level mapping.
- **Family coupling.** `egobox-moe` 0.36.2 requires `egobox-gp` ^0.37.1 and `egobox-doe`
  ^0.35.2, and `egobox-ego` 0.42.0 requires `egobox-moe` ^0.36.2. Taking the mixed-integer
  fix therefore moves the whole egobox family, adding `basin` ^1.13 through `egobox-gp`
  0.37.

Research date 2026-10-27.

Sources:
- Installed sources `egobox-doe-0.35.1/src/{lib,lhs,random,full_factorial,utils,traits}.rs`
  and `egobox-moe-0.35.5/src/mixint.rs`.
- Downloaded crate sources `egobox-doe-0.35.2` and `egobox-moe-0.36.2/src/mixint.rs`.
- The egobox `CHANGELOG.md` on GitHub `relf/egobox` main: the 0.38.1 entry "Fix
  mixed-integer sampling fairness", and the 1.0.0 entry listing crate versions.
- crates.io dependency metadata.
- Probe: [egobox-doe determinism](../../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md)
  (repository path `docs/design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md`).

## Types

### egobox-doe/seeded-space-filling-designs
- **Need it serves:** Produce a reproducible box-bounded design: Latin hypercube, random
  or full-factorial points for multistart seeds, sweeps, surrogate initial designs or
  global checks.
- **Recognizable code shapes:**
  - Callers that require a caller-supplied point list.
  - Single-point perturbation generators.
  - Ad hoc `rand` loops filling a box.
  - Hand-written stratified sampling.
- **Mechanisms and APIs:**
  - `SamplingMethod<F>` with `sampling_space()`, `normalized_sample(ns)` returning
    `[0,1]^nx`, and `sample(ns)`, which maps the unit sample by `u * (ub - lb) + lb`.
  - `Lhs::new(&xlimits)` takes an `(nx, 2)` ndarray. Add `.kind(LhsKind::{Classic,
    Centered, Maximin, CenteredMaximin, Optimized})`, where `Optimized` (ESE maximin) is
    the default. Add `.with_rng(R: Rng)`.
  - `Random::new(&xlimits).with_rng(..)`.
  - `FullFactorial::new(&xlimits)`.
  - The RNG generic is any `rand` 0.8 `Rng`. The documented reproducible choice is
    `rand_xoshiro::Xoshiro256Plus::seed_from_u64(seed)`, which `rand_xoshiro` seeds through
    SplitMix64.
  - `normalized_sample` exposes the unit-cube design, so a caller can do its own exact
    bound mapping or integer-level mapping.
- **Obligations discharged:**
  - Stratification and permutation (LHS).
  - Maximin selection among 5 candidate designs, or ESE optimization of the φp criterion.
  - Factorial level allocation "as evenly as possible".
  - Seeded stream consumption.
- **Residual caller obligations:**
  - **Seeding.** Without `with_rng`, `Lhs::new` and `Random::new` seed from OS entropy.
    Seeding must be explicit.
  - **One instance per design.** The RNG lives in the instance behind
    `Arc<RwLock<R>>`, so a second `sample()` continues the stream. Cloning shares the
    `Arc`, so clones also continue one stream.
  - **Bounds.** `xlimits` must be finite. The library does not validate `lb < ub` or
    finiteness: infinite bounds produce NaN or inf, and `ncols != 2` panics.
  - **Bound mapping.** The mapped value is computed in floating point, so endpoints are
    not guaranteed to equal the declared bounds exactly. Exact bounds membership or
    endpoint identity must be checked or constructed by the caller.
  - **Discrete coordinates.** Integer and categorical coordinates are not handled by
    `egobox-doe` (see the mixed-integer type below).
  - **FullFactorial.** It returns the first `ns` rows of the smallest balanced grid with
    at least `ns` points. When `ns` is not a product of the level counts, the design is
    truncated and unbalanced. There is no per-coordinate level control.
- **Semantic conditions:**
  - **Determinism.** Determinism for a fixed RNG state is by construction. Classic,
    Centered, Maximin, Random and FullFactorial use only:
    - integer RNG arithmetic;
    - the rand 0.8 `UniformFloat` mapping, a separate multiply and add with no `mul_add`;
    - `shuffle` via `gen_index`, which samples `u32` when the length fits, so it is
      portable between 32- and 64-bit targets;
    - ndarray elementwise operations;
    - `sqrt`, an IEEE correctly-rounded operation, in `pdist`.
  - **Optimized kind.** `LhsKind::Optimized` also uses `powf` in the φp criterion and
    swap updates. `powf` reaches the platform libm, and ESE acceptance compares those
    values, so cross-libm bit stability is not established for that kind.
  - **Parallelism.** `pdist` and `cdist` run under rayon but preserve order.
  - **Mutation.** A sampler instance mutates under a write lock. Do not share one sampler
    across threads when deterministic designs are required.
- **Version / build conditions:**
  - **Ecosystem.** The crate sits on `ndarray` 0.16, `ndarray-rand` 0.15 (rand 0.8),
    `rand_xoshiro` 0.6 and `linfa` 0.8 (`Float`). These form an ndarray/rand-0.8
    universe.
  - **Value stability.** It is not promised across `rand`, `rand_xoshiro` or
    `ndarray-rand` minor versions. Under the rust-random policy, minor versions may make
    value-breaking changes.
  - **Features.** Only `serializable` (serde) is optional. The `rayon` dependency is
    unconditional.
- **Limits and negative results:**
  - Inspected `src/` of 0.35.1 and 0.35.2 and the upstream main `crates/doe/src` listing.
  - There is no Sobol, Halton or other low-discrepancy sequence, no scrambling, no
    orthogonal-array or strength-2 LHS, and no discrepancy metric.
  - There is no constraint-aware design beyond the box.
  - Panics, rather than `Result`, on a malformed `xlimits`.
  - **Maximin cost and a small-design panic (added 2026-10-27).** `LhsKind::Maximin` and
    `CenteredMaximin` call `_maximin_lhs(ns, centered, 5)`, which computes `pdist` over all
    `ns` points for each candidate: O(ns²) memory and time per candidate. `Optimized` repeats
    `pdist` in each ESE iteration. `pdist(&lhs).min().unwrap()` panics when `ns < 2`. A caller
    therefore needs a size bound before admitting the maximin kinds, a guard for `ns ≥ 2`, and
    Classic as the default for large designs (Interface-checked against
    `egobox-doe-0.35.1/src/lhs.rs`).
- **Evidence:**
  - Interface-checked: the installed source files listed above, and the rand 0.8.8
    `seq::gen_index` and `UniformFloat::sample` source.
  - Tested: the [determinism probe](../../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md).
    Every kind was bit-identical across opt-level 0 and 3, `target-cpu=x86-64-v3`, and
    rayon 1 versus 32 threads, on one x86_64 glibc 2.39 host. Seed continuation and the
    entropy default behaved as described above. Cross-OS and cross-libm behavior was not
    tested.

### egobox-moe/mixed-integer-design-mapping
- **Need it serves:** Map a continuous design onto integer, ordered or categorical
  coordinates.
- **Recognizable code shapes:**
  - Rounding continuous samples to integers after the fact.
  - Hand-built level grids for discrete variables.
- **Mechanisms and APIs:**
  - `XType::{Float(lb, ub), Int(i32, i32), Ord(Vec<f64>), Enum(usize)}` and
    `as_continuous_limits(&xtypes)`.
  - `MixintSampling::new(method, xtypes)`, with `work_in_folded_space(bool)` choosing
    between enum indices and one-hot masks.
  - `to_discrete_space`.
  - **0.35.5 (installed).** `sample()` maps continuously over `[lb, ub]` and then applies
    `round()` (integers) or nearest-level snapping (ordered).
  - **0.36.2 (upgrade).** Adds `LevelMapping::EqualSlices` as the default. It cuts the
    unit sample `u` into equal slices per level (`level_index(u, n)`) for random and LHS
    designs, so each level is equally likely. `LevelMapping::Nearest` is kept for factorial
    grids.
- **Obligations discharged:**
  - Folding and unfolding of enum masks.
  - Level projection.
  - Equal-probability levels and per-level LHS stratification, from 0.36.2 only.
- **Residual caller obligations:**
  - Integer bounds must fit `i32`.
  - Ordered levels are `f64`.
  - The domain model must still state which coordinates are discrete and their
    admissible sets.
- **Semantic conditions:**
  - **Under-sampled bounds (0.35.5).** Bound levels of `Int`/`Ord` variables get half
    weight: `[lb, ub]` is sampled continuously and then rounded. The changelog calls this
    "bounds … under-sampled".
  - **Rounding mode.** `round()` rounds half away from zero.
  - **Determinism.** It inherits the underlying sampler's determinism. Level mapping is
    integer and compare arithmetic.
- **Version / build conditions:**
  - `MixintSampling` lives in `egobox-moe`, not `egobox-doe`.
  - The fair mapping requires the 0.36 family move described above.
- **Limits and negative results:** No support for integer bounds outside `i32` or for
  mixed constraints. There is no factorial level choice per discrete coordinate beyond
  `FullFactorial`'s even allocation.
- **Evidence:**
  - Interface-checked: `egobox-moe-0.35.5/src/mixint.rs`, covering `as_continuous_limits`,
    `cast_to_discrete_values_mut` and `MixintSampling::sample`, and
    `egobox-moe-0.36.2/src/mixint.rs`, covering `LevelMapping` and
    `normalized_to_discrete`.
  - Interface-checked: the CHANGELOG entry.

## Current applicability

### [CBO-033](../../../library_leverage/codebase/numerical-workflows.md#cbo-033) — No producer of sampling or experimental designs
- **Types:** `egobox-doe/seeded-space-filling-designs`,
  `egobox-moe/mixed-integer-design-mapping`.
- **Realization:**
  - Continuous LHS, Random and FullFactorial at the installed pin, with no dependency
    change. `egobox-doe` only needs to move from `[dev-dependencies]` to
    `[dependencies]`; it is already in the production graph.
  - Fair integer mapping only on the upgrade route: egobox-moe 0.36.2 together with
    egobox-gp 0.37.1 and egobox-ego 0.42.0 (Rust API changes described upstream as
    additive), which also brings `basin`.
- **Conclusion:** fits with conditions. Continuous box designs with explicit seeding meet
  the reproducibility premise on a given platform, and static reading supports platform
  independence except for `LhsKind::Optimized`.
- **What it would discharge / what remains local:**
  - **Discharged.** LHS stratification and permutation, maximin and ESE selection, random
    and factorial generation, and seeded stream handling.
  - **Local.** Design authoring (kind, `ns`, seed) and resolving design bounds from the
    original declaration. Local code must also:
    - refuse non-finite or unbounded coordinates;
    - check or construct exact bound membership, because the multistart entry refuses
      out-of-bound values and must not clamp;
    - map integer coordinates;
    - create a fresh seeded sampler per design;
    - record the realized points as inputs.
  - The repository's point identity (`semantic_point_key` over realized values) already
    keys on the realized point. Regeneration-from-seed portability therefore affects
    reproducibility of the design, not the integrity of a recorded identity.
- **Prerequisites, configuration or extension:**
  - Choose a non-`Optimized` kind, or accept libm-dependent selection for `Optimized`.
  - `with_rng(Xoshiro256Plus::seed_from_u64(seed))`.
  - Map `normalized_sample` to bounds locally, using the repository's FMA-free arithmetic,
    when exact endpoint or bound behavior matters.
- **Semantic gaps against the CBO's preservation constraints:**
  - **Platform independence.** It is unproven for `Optimized` (powf/libm). For the other
    kinds it is supported by reading and a single-host probe.
  - **Integer variables.** They are unsupported in `egobox-doe`. The installed
    `MixintSampling` under-weights bound levels, and the equal-slice fix requires an
    egobox family upgrade. An integer level map over `normalized_sample` is a small local
    alternative.
  - **Bounds.** No validation, and the mapped endpoints are not exact.
  - **Version stability.** Bumping `rand`, `rand_xoshiro` or `ndarray-rand` may silently
    change designs for the same seed. A golden-hash test would detect it.
- **Unsettled premises and what would settle them:**
  - Cross-OS behavior of `Optimized`: a run on macOS, Windows or musl.
  - Whether designs must be regenerable from seed across versions, or whether recording
    the realized points suffices. This is a design decision for comparative review.
  - Whether integer coordinates occur in multistart or study specifications at all. The
    multistart seed map is `SemanticId -> f64`, checked against `contract.variables`
    bounds; the domain owner would settle whether discrete variables appear there.
- **Evidence:**
  - Interface-checked: the egobox sources and the multistart validation in
    `math/solves/multistart.rs`.
  - Tested: the [probe](../../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md).
  - The application to pse-arrow is Proposed.

**Reviewer correction (2026-10-27).** The consumer set is narrower than the CBO assumed. Profile
chains (secant predictors seeded from their predecessor) and global certification (SCIP) do not
consume designs; multistart and surrogate preparation are Rust APIs with only test callers; Python
`with_numerical_strategy` binds only declared-case rungs. Studies are the only live public
consumer. A parity reference exists even though no local contract promises designs: IDAES 2.13
parameter sweeps sample with Latin hypercube, uniform, Halton, Hammersley, CVT and custom
methods. The comparative review recommends one seeded design producer on `normalized_sample`
(Classic LHS by default; Centered, Maximin and Random; a local per-coordinate grid instead of
`FullFactorial`), with a fresh `Xoshiro256Plus::seed_from_u64` per design, exact endpoint mapping
that verifies membership and never clamps, equal-slice level mapping for discrete coordinates,
and maximin kinds admitted only below a prepaid size bound that also charges the O(n²) `pdist`
workspace. `egobox-doe` moves from dev to normal dependencies and a direct `rand_xoshiro =0.6.0`
is added (already in the lock). Whether `Optimized` and seed-regenerable designs are admitted
depends on the replay-versus-regenerate decision. See the
[comparative review](../../reviews/numerical-workflows.md#cbo-033).

### [CBO-035](../../../library_leverage/codebase/numerical-workflows.md#cbo-035) — Separate dense-algebra universes
- **Types:** The ndarray/rand-0.8 ecosystem condition recorded above.
- **Realization:** Installed.
- **Conclusion:** unsettled, and context only.
  - Using `egobox-doe` for CBO-033 adds no new universe.
  - The ndarray 0.16 / rand 0.8 / nalgebra 0.33 (via `statrs` default features in
    `egobox-moe`) cluster is held in place by the egobox family. It stays held even at
    egobox-moe 0.36.2, which still requires `statrs` ^0.18 with default features.
- **What it would discharge / what remains local:** Nothing for CBO-035 directly.
- **Prerequisites, configuration or extension:** Removing ndarray from pse-runtime would
  require removing the egobox surrogate route.
- **Semantic gaps against the CBO's preservation constraints:** None specific.
- **Unsettled premises and what would settle them:** Whether egobox upstream would accept
  `statrs` with `default-features = false`, which would drop nalgebra 0.33 and rand 0.8
  from that path.
- **Evidence:** Interface-checked: `Cargo.lock` dependency edges and crates.io metadata
  for egobox-moe 0.36.2.

**Reviewer correction (2026-10-27).** The lock has two ndarray lines: 0.16.1 through egobox and
0.17.2 through `numpy` 0.29 and `rustworkx-core` 0.18.1, so removing the egobox route would not
remove ndarray from the graph either. The comparative review retains the current routes; an
upstream `statrs default-features = false` change in egobox-moe would drop nalgebra 0.33 only
after a release and a family move. See the [comparative review](../../reviews/numerical-workflows.md#cbo-035).
