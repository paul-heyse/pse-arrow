# Rust build reuse

The toolchain is the dated nightly in `rust-toolchain.toml`, its only declaration
(ADR-0122). The source uses no nightly language feature. The nightly carries Cargo's
**workspace feature unification** (`-Z feature-unification`,
`[resolver] feature-unification = "workspace"`), which `.cargo/config.toml` turns on for
every Cargo run in a checkout, including rust-analyzer and maturin. Dependency features
resolve over all workspace members whatever `-p` selects, so `check-package`,
`unit-package`, `unit-native-package` and `check-solver-contracts` share one build of
symbolica, DataFusion and Arrow per mode. Only opt-in features —
`pse-relations/force-validate` and `native-solvers` — select a different build.

`pse-workspace-hack` (cargo-hakari, `.config/hakari.toml`) states the same unification in
manifests, for Cargo that does not read `[unstable]`. Hakari simulates every member with
all features, so the opt-in paths are excluded from its traversal; `just codegen`
regenerates it and `just codegen-check` compares it.

Each checkout keeps its intermediates in its own `target/`. Cargo's build directory is
not shared between checkouts: Cargo keys a workspace crate's unit by its
workspace-relative path and checks freshness by mtime, so a shared directory let one
worktree reuse another's artifact for different sources (ADR-0122). Across checkouts,
the shared sccache serves identical compilations.

Ordinary local `just` commands and direnv activate an installed `sccache`. Explicit
`RUSTC_WRAPPER` settings take precedence, including the empty string. CI retains its
existing cache policy. Workspace
crates use O2, with incremental compilation in dev/test; release retains non-incremental
ThinLTO. Imported dependencies use O3 without incremental compilation, including vendored
path dependencies. Keep Cargo's artifacts for direct reuse; sccache provides a fallback
for eligible compilations. The Linux linker is mold, as declared in `.cargo/config.toml`.
The existing profiles serve the daily workflows; no separate optimized test profile is
needed.

```bash
just build-frontend 2 unit-package pse-compiler 'test(workspace_tests::)'
just build-uncached check-package pse-compiler
just build-cache-probe
just build-storage
```

`build-frontend N` is the parallel rustc frontend experiment (`-Zthreads=N`, N in 1, 2,
4, 8). It builds into `target/frontend-N`, so its flags never replace the checkout's
ordinary artifacts. `.config/build.toml` owns the cache and
storage budgets; `.cargo/config.toml` owns the default 16 Cargo jobs, and explicit job
overrides take precedence. For noninteractive commands, use the same environment entry
point:

```bash
python3 -m scripts.build_environment --frontend 2 --jobs 16 --cache on -- just check-package pse-compiler
```

The environment preserves explicit encoded flags, or `RUSTFLAGS`, according to
Cargo's precedence. Without those overrides it retains the repository target flags,
including mold. No `target-cpu=native`, fast-math, panic-abort or global incremental
disable is introduced. Correctness recipes still request Arrow force validation.

The automatic local cache has a 100 GiB budget under
`${XDG_CACHE_HOME:-$HOME/.cache}/pse-arrow/sccache`, with a dedicated endpoint. It
does not stop the user's default cache server. An explicit `SCCACHE_DIR` keeps the
user's server configuration. `build-cache-probe` uses a disposable server and checks
an eligible Rust hit plus incremental pass-through. Product benefit must be measured
with `bench-builds`; a canary is insufficient.

## Native preparations

`scripts/native-solver-env.sh` and `scripts/native-math-env.sh` use one persistent
root, `${PSE_NATIVE_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/pse-arrow/native}`. Solver
extraction is keyed by immutable image identity. KLU is keyed by vendored source,
compiler bytes/version, target, flags, CMake identity and options. Preparations are
locked per identity, installed into private staging directories and published after
required files and content hashes are recorded. Missing or changed files trigger
repreparation. Explicit `IPOPT_DIR` and the CI image route remain supported.

Bindgen selects resource headers from its selected Clang without changing the C/C++
compiler. When no user CMake toolchain file exists, `.config/native-cache.cmake`
supplies the supported C/C++ compiler launchers through `CMAKE_TOOLCHAIN_FILE`, which
the pinned `cmake` build dependency forwards. KLU also supplies launchers explicitly.
Fortran is unchanged. GMP/MPFR retains its upstream persistent cache mechanism.

## Measurements and retention

```bash
just bench-builds build/build-measurements/baseline --cache off
just bench-builds build/build-measurements/cached --cache on --cold-cache --recovery --second-worktree
just bench-builds build/build-measurements/frontend-screen --frontend 4 --jobs 16 --screen
just bench-builds build/build-measurements/profile-one --cache on --dependency-opt 1 --execute
just bench-builds build/build-measurements/native-workflow --cache on --native --workflow
```

Every output directory must be new. The runner snapshots dirty and untracked source
without changing the original index or files. It compiles current compiler/relations
test targets, or the explicit native selection, then repeats unchanged/private/API
edits three times by default. `--screen` omits edits and cannot establish total edit
feedback latency. `--workflow` creates an isolated Python environment and measures
extension → compiled stubs → native units repeatedly; those units execute.
`--execute` repeats compiler/relations library tests with explicit force-validation.
`--cold-cache` uses an empty compiler cache owned by that campaign. Source downloads
and native installations remain separately cached. Candidate profile overrides live
only in the snapshot's Cargo configuration, so nested recipes use the same policy.
By default, measurements preserve the manifest's dependency optimization level;
`--dependency-opt 1`, `2` or `3` explicitly selects a candidate override.

Reports retain source/command identity, Cargo artifact JSON and timing HTML, cache
statistics, wall/CPU time, disk availability, and Linux process-tree RSS samples.
RSS is sampled every 100 ms and counts shared resident pages in each process.
Campaign-owned foreground cache servers are included in CPU/RSS accounting; shared
servers are explicitly excluded in reports. Wall time excludes cache-server startup
and shutdown. Inspect Cargo's timing HTML for the critical path;
summed compilation durations are not wall time. The cold run is a screening sample;
replicate leading candidates before claiming a cold speedup. Other active builds and
source changes make a run diagnostic. No linker comparison is included.

Keep each checkout's `target/` and at most one temporary campaign. Run
`just build-storage` before large campaigns. Below the configured
50 GiB free-space floor, the measurement runner refuses to start. Reclaim identified
inactive campaign targets explicitly; preserve reports and the active incremental,
compiler, downloaded-source and native caches. Avoid broad `cargo clean` recovery.
No automatic deletion or second compiler-artifact cache is implemented.

Build measurements are not product qualification. Cache reuse cannot substitute for
current source, native-byte and executed-test evidence.
