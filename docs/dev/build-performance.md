# Rust build reuse

Ordinary local `just` commands and direnv activate an installed `sccache`. Explicit
`RUSTC_WRAPPER` settings take precedence, including the empty string. CI retains its
existing cache policy. Workspace incremental compilation stays enabled. The Linux
linker is mold, as declared in `.cargo/config.toml`.

```bash
just build-stable unit-package pse-compiler 'test(workspace_tests::)'
just build-dev unit-package pse-compiler 'test(workspace_tests::)'
just build-uncached check-package pse-compiler
just build-cache-probe
just build-storage
```

`build-dev` is an experimental, dated nightly route. `.config/build.toml` owns its
date, frontend threads, jobs and cache/storage budgets. It has one persistent target
tree shared by compatible recipes. `build-stable` selects `rust-toolchain.toml` and
the ordinary stable target. Promoting nightly is a governance change backed by fresh
`bench-builds` measurements; the retired
[build-performance plan](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/15-rust-build-performance.md)
records the earlier screening. Stable remains the canonical compiler.
Invoke the stable route from a fresh shell if caller flags contain nightly options;
the command refuses those flags rather than silently changing their meaning.

For noninteractive commands, use the same environment entry point:

```bash
python3 -m scripts.build_environment --mode nightly --frontend 2 --jobs 16 --cache on -- just check-package pse-compiler
```

The environment preserves explicit encoded flags, or `RUSTFLAGS`, according to
Cargo's precedence. Without those overrides it retains the repository target flags,
including mold. No `target-cpu=native`, fast-math, panic-abort or global incremental
disable is introduced. Correctness recipes still request Arrow force validation.

The automatic local cache has a 32 GiB budget under
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
just bench-builds build/build-measurements/nightly-screen --mode nightly --frontend 4 --jobs 16 --screen
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

Reports retain source/command identity, Cargo artifact JSON and timing HTML, cache
statistics, wall/CPU time, disk availability, and Linux process-tree RSS samples.
RSS is sampled every 100 ms and counts shared resident pages in each process.
Campaign-owned foreground cache servers are included in CPU/RSS accounting; shared
servers are explicitly excluded in reports. Wall time excludes cache-server startup
and shutdown. Inspect Cargo's timing HTML for the critical path;
summed compilation durations are not wall time. The cold run is a screening sample;
replicate leading candidates before claiming a cold speedup. Other active builds and
source changes make a run diagnostic. No linker comparison is included.

Keep the stable and selected nightly working sets plus at most one temporary
candidate. Run `just build-storage` before large campaigns. Below the configured
50 GiB free-space floor, the measurement runner refuses to start. Reclaim identified
inactive campaign targets explicitly; preserve reports and the active incremental,
compiler, downloaded-source and native caches. Avoid broad `cargo clean` recovery.
No automatic deletion or second compiler-artifact cache is implemented.

Build measurements are not product qualification. Cache reuse cannot substitute for
current source, native-byte and executed-test evidence.
