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

That reuse depends on `CARGO_TARGET_DIR` staying unset. sccache keys every rustc call on
its `CARGO_*` environment, so an absolute per-checkout target path would make every Rust
compilation miss the entries another checkout made. The build environment
(`scripts/build_environment.py`, applied by `scripts/pse-env` for direnv, every recipe and
agent commands) therefore removes `CARGO_TARGET_DIR` when it
names the checkout's own `target/`, which is Cargo's default, and when it is empty or
points outside the checkout, as a value inherited from another checkout does: a
long-running agent or editor started under one checkout's direnv carries its absolute path
into every worktree. A directory inside the checkout, such as `target/measure-production`,
stays explicit, and `PSE_CARGO_TARGET_DIR` chooses another directory deliberately.
`scripts/pse-env -- sh -c 'echo ${CARGO_TARGET_DIR-unset}'` prints `unset` in an
ordinary checkout. A bare `cargo` outside a recipe or an activated direnv keeps whatever
its process inherited, possibly another checkout's target directory, so build through the
recipes.

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

The shared environment requires sccache client-side compilation, keeping compiler
children inside the invoking operation's supervision. It disables the C/C++ direct
include shortcut so preprocessing observes newly present headers. The repository's
`.config/sccache.toml` supplies the default configuration; explicit configurations
remain supported. Logging or distributed scheduler settings that disable client-side
compilation cause ordinary builds to use the compiler directly. An explicit cache
experiment refuses those settings rather than measuring a different execution mode.
The local cache remains a trusted build input, not an artifact provenance authority.

## Native preparations

The native operation owner uses one persistent root,
`${PSE_NATIVE_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/pse-arrow/native}`. Recipes request
only their required capabilities. Solver extraction is keyed by immutable image identity;
compiled providers include actual source, compiler, target, flags and build-tool inputs.
Private staging publishes immutable generations after complete byte/interface verification.
Construction and publication use separate coordination, so an admitted reader need not wait
for another generation's build. An operation retains its selected generation until actual
descendants drain; a new operation verifies it again. Missing or changed files cause readmission
or rebuilding. Unverifiable surviving owners retain conservative guards. Explicit `IPOPT_DIR`
remains supported with full boundary verification. Manual sourced helpers grant no operation
reuse; the math helper selects KLU/root isolation and the pipeline helper selects UNO/PETSc.

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
When the selected native units consume the canonical store, select its serving profile and
the existing scientific workload allocation in the launch environment. `PSE_REQUIRE_STORE=1`
keeps that selected service available during admission, equivalent to `pse-env --store`.
For example, the Plan 28 wide workflow uses `PSE_SURREAL_STATE`, `PSE_WORKER_BINARY`,
`PSE_RESOURCE_CLASS=exclusive` and `PSE_MEMORY_MAX=144G` outside a workload owner.
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

`bench-builds` starts a pinned Python observer outside the workload allocation. Its
workload, including source snapshots and compiler-only builds, enters the existing
`pse-env` supervisor. With `--native` or `--workflow`, `complete-operation.json`
separately records launch/admission, selected native setup, the measurement child
and authenticated final scope drain. It includes initial setup and drain in the
complete-operation wall time; Cargo and target-operation durations keep their
original narrower boundaries. A missing or unverified drain fails the measurement.
An invocation already inside a native owner remains explicitly target-only because
its earlier setup and later drain cannot be observed from that child.

Keep each checkout's `target/` and at most one temporary campaign. Run
`just build-storage` before large campaigns. Below the configured
50 GiB free-space floor, the measurement runner refuses to start. Reclaim identified
inactive campaign targets explicitly; preserve reports and the active incremental,
compiler, downloaded-source and native caches. Avoid broad `cargo clean` recovery.
No automatic deletion or second compiler-artifact cache is implemented.

Build measurements are not product qualification. Cache reuse cannot substitute for
current source, native-byte and executed-test evidence.

`just build-storage` reports a versioned JSON metadata snapshot of configured stores and the
bounded legacy report cohorts. It reports allocated/apparent bytes, ownership and unavailable
or partial observations. `just build-storage --metadata-only` avoids recursive measurement.
Persistent state and shared skill targets are identity-only observations; nested symlinks are
not traversed and alias roots are reported once. Hardlinks are deduplicated within each row;
parent/cohort rows overlap and must not be summed. Concurrent writers make this a non-atomic
snapshot, and active use/resource identity may remain unknown. There is no reclaimable-byte
total or generic prune command: assessment, native, compiler-cache, campaign and persistent
state owners retain their separate disposal contracts. `.git` and `.venv*` are excluded from
measurement. Ordinary document queries never invoke this scan.
