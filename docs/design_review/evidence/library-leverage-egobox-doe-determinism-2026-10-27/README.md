# egobox-doe seed determinism probe

Evidence for the library-leverage applicability assessment of
[CBO-033](../../../library_leverage/codebase/numerical-workflows.md#cbo-033), 2026-10-27,
source baseline `dbc4c9436`. It characterizes one library in isolation. It does not
exercise any pse-arrow design producer, because none exists.

## Question

Does `egobox-doe` 0.35.1 return bit-identical designs for a fixed seed when the build or
runtime settings change in ways that could plausibly alter floating-point results? Reading
the source left two things open:

- the default `LhsKind::Optimized` path calls `powf`, which is a platform-libm call, and
  computes distances in a rayon parallel iterator;
- opt-level and an FMA-capable `target-cpu` could change code generation.

## Inspected source and environment

- **Crates.** `egobox-doe` 0.35.1, `rand_xoshiro` 0.6.0, `rand` 0.8.8, `rand_core` 0.6.4,
  `ndarray` 0.16.1, `ndarray-rand` 0.15.0, `ndarray-stats` 0.6.0, `linfa` 0.8.1 and
  `rayon` 1.12.0. These came from a copy of the workspace `Cargo.lock`, which Cargo pruned
  to the probe's graph. The probe resolved the same versions as the workspace lock.
- **Static reading** of the installed sources:
  - `lhs.rs`, `random.rs`, `full_factorial.rs`, `utils.rs` and `traits.rs`;
  - rand 0.8.8 `seq::shuffle`/`gen_index` and `UniformFloat::sample`;
  - rand_xoshiro 0.6.0 `seed_from_u64`, which uses SplitMix64.
- **Toolchain.** `rustc 1.101.0-nightly (c1070d693 2026-09-28)`, the pinned toolchain,
  selected because the probe ran from the repository root.
- **Host.** AMD Ryzen 9 9950X3D (avx2, fma, avx512f), Ubuntu glibc 2.39, x86_64 Linux.

## Probe

A scratch crate outside the repository with `egobox-doe = "=0.35.1"`, `ndarray = "=0.16.1"`
and `rand_xoshiro = "=0.6.0"`. It generates:

- every `LhsKind` (Classic, Centered, Maximin, CenteredMaximin, Optimized) for nx=3/ns=10
  and nx=6/ns=40;
- `Random` for nx=3/ns=10;
- `FullFactorial` for nx=3/ns=10.

All seeded runs use `Xoshiro256Plus::seed_from_u64(seed)` with seeds 42 and 7. The probe
prints each design's FNV-1a-64 hash over the little-endian `f64::to_bits` bytes, plus full
bit dumps for seed 42. It also checks two properties: whether repeated `sample()` calls on
one instance repeat, and whether two entropy-seeded `Lhs::new` instances agree.

```rust
let d = Lhs::new(&xlimits).kind(kind)
    .with_rng(Xoshiro256Plus::seed_from_u64(seed))
    .sample(ns);
// hash: FNV-1a 64 over v.to_bits().to_le_bytes() for v in d.iter() (row-major logical order)
```

Commands, run from the repository root. `$P` is the scratch probe directory.

```bash
scripts/pse-env -- cargo build --offline --manifest-path $P/Cargo.toml             # dev, opt-level 0
scripts/pse-env -- cargo build --offline --release --manifest-path $P/Cargo.toml   # opt-level 3
scripts/pse-env -- env RUSTFLAGS="-C target-cpu=x86-64-v3 -C link-arg=-fuse-ld=mold" \
  cargo build --offline --release --target-dir $P/target-v3 --manifest-path $P/Cargo.toml
scripts/pse-env -- env RAYON_NUM_THREADS=32 $P/target/debug/egobox-doe-determinism-probe
scripts/pse-env -- env RAYON_NUM_THREADS=32 $P/target/release/egobox-doe-determinism-probe
scripts/pse-env -- env RAYON_NUM_THREADS=1  $P/target/release/egobox-doe-determinism-probe
scripts/pse-env -- env RAYON_NUM_THREADS=32 $P/target-v3/release/egobox-doe-determinism-probe
```

## Outcome

**Tested.** The builds and runs completed. The four configurations produced byte-identical
output once the entropy line was excluded: SHA-256 prefix `9d77f79af1f92b8a` in each case.
The configurations were dev at 32 threads, release at 32 threads, release at 1 thread, and
release with `x86-64-v3` at 32 threads. No configuration difference was observed.

Seed-42 hashes, identical in all four configurations:

```text
lhs-classic nx=3 ns=10          3d589b2dcb6e2d1e    lhs-classic nx=6 ns=40          da13a090e2f68dc1
lhs-centered nx=3 ns=10         c23b0969b21ead61    lhs-centered nx=6 ns=40         c17d5be4537a2247
lhs-maximin nx=3 ns=10          3d589b2dcb6e2d1e    lhs-maximin nx=6 ns=40          1104d26b992068d0
lhs-centered_maximin nx=3 ns=10 c23b0969b21ead61    lhs-centered_maximin nx=6 ns=40 c17d5be4537a2247
lhs-optimized nx=3 ns=10        018f369becf54f06    lhs-optimized nx=6 ns=40        8fb1bcc37bf98c65
random nx=3 ns=10               cf54e8714ef04b94    fullfactorial nx=3 ns=10        96616152761346e5
stateful-continuation equal=false    entropy-default equal=false
```

Further observations:

- **Stateful sampling.** A second `sample()` on the same instance continues the RNG stream
  and returns a different design. Reproducing a design requires a fresh instance seeded
  per design.
- **Entropy default.** `Lhs::new` and `Random::new` without `with_rng` are seeded from OS
  entropy and do not reproduce.
- **Bound mapping is not exact.** `sample()` maps by `u * (upper - lower) + lower`. The
  centered nx=3 design contains `-1.4999999999999998` (`bff7fffffffffffe`) where the
  decimal design value is -1.5. Endpoint values from `FullFactorial` and centered designs
  are therefore not guaranteed to equal the declared bounds bit-for-bit.
- **FullFactorial with a non-product `ns`.** With ns=10 and nx=3, egobox builds the 3×2×2
  grid and returns its first 10 rows, so the design is truncated.

## Limits

- **One host, one libm.** This probe cannot establish cross-CPU or cross-OS bit stability
  for the `Optimized` kind. That kind's `powf` calls reach the platform libm (glibc 2.39
  here; macOS and Windows libms differ). Classic, Centered, Maximin, Random and
  FullFactorial use only IEEE basic operations, integer RNG arithmetic and `sqrt`. Static
  reading supports platform independence for those kinds. That is an
  *Interface-checked* argument, not a cross-platform test.
- **Rayon.** `pdist`/`cdist` parallelism is order-preserving (`collect` on an indexed
  parallel iterator; per-row writes). Thread count did not matter in this probe.
- **Versions.** Value stability across `rand`/`rand_xoshiro`/`ndarray-rand` minor versions
  is not promised. The rust-random reproducibility policy permits value-breaking changes in
  minor releases. A dependency move is a different experiment.
- **Scope of the observation.** `egobox-doe` 0.35.2 (released 2026-10-12) has `src/`
  identical to 0.35.1 apart from line endings (diff with `--strip-trailing-cr`), so the
  observations should carry over. That was not run.
- **Raw outputs.** The probe crate's [Cargo.toml](probe/Cargo.toml),
  [src/main.rs](probe/src/main.rs) and raw `probe/out/` outputs are retained.
  The lockfile and build outputs are not retained.
