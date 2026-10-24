# Rust build reuse: sccache, cargo-shear and cargo-hakari

Functional-extension review · 2026-10-24 · developer-workflow scope, single local machine.

## Recommendation and scope

The question: what is the best-in-class configuration of the three build-reuse tools in
[the capability overview](../libraries/rust_caching.md) for this workspace? Remote and
distributed caching and CI portability are out of scope. The answer differs per tool.

- **sccache:** keep the current architecture. Client-side mode, one managed local disk
  cache, incremental compilation for workspace crates and cacheable non-incremental
  dependencies are already the right shape for this workload. Upgrade to 0.18.0 and give it a
  declared version ([O01](#O01)). Correct one premise in the overview and the build guide:
  `SCCACHE_BASEDIRS` does not normalize Rust cache keys, so a second checkout reuses
  registry crates but not vendored path crates or workspace crates ([O02](#O02)). The
  incremental split stays as it is ([O03](#O03)).
- **cargo-shear:** make it the single dependency-hygiene detector and drive its findings to
  zero by fixing their causes rather than adding ignores ([O05](#O05)). Its opt-in test-target
  analysis is worth adopting ([O06](#O06)). Retire cargo-machete ([O07](#O07)). Move shear into
  `just hygiene` once it reports zero ([O08](#O08)); that routing is the maintainer's call.
- **cargo-hakari:** do not bring it back ([O09](#O09)). Cargo's workspace feature unification
  already keeps dependency features identical across recipe selections. The one remaining
  class of duplicate, host and target builds of about 30 small dependencies, is real, but a
  workspace-hack would trade it for wider small-member closures, the cost ADR-0175 measured
  when it removed the hack.

The opportunities are proposals. They do not authorize installation, configuration changes
or edits to any plan. [O04](#O04) and [O10](#O10) are conditional diagnostics, not selected
scope.

## Baseline, method and evidence limits

Source baseline: `main` at `95ee5dfdd`. Unrelated dirty files (`history.txt`, test-audit and
validation documents, plan 52l, the generated capability reference) were neither inspected nor
edited. Toolchain: `nightly-2026-09-29` (rustc 1.101.0-nightly). Installed tools:
sccache 0.17.0, cargo-shear 1.14.0, cargo-machete, cargo-udeps; cargo-hakari is installed but
nothing calls it.

Method:

- **Cargo unit graphs**, resolved without compiling (`--unit-graph -Z unstable-options
  --locked` under `scripts/pse-env`). Selections: `check --workspace --all-targets`, the same
  with `pse-relations/force-validate`, `check -p pse-model`, `check -p pse-runtime
  --features native-solvers`, `test --workspace --no-run` with force-validate, and `build -p
  pse-py --lib` (the maturin shape). Analyzed with
  [the census probe](probes/rust-build-reuse-unit-census-2026-10-24.py). *Interface-checked*:
  this is Cargo's own resolution, but no compile times were measured.
- **cargo-shear 1.14.0** default analysis and `--check-test-targets` (JSON), compared with
  `cargo machete --with-metadata`. Representative findings were checked against the source.
- **Managed sccache statistics** (`scripts/sccache --show-stats` through `scripts/pse-env`),
  accumulated since the shared server last started. The window is unknown, so these numbers
  are an observation, not a benchmark.
- **sccache v0.18.0 source and release notes** (tag commit `01c35e6`): `README.md`,
  `docs/Configuration.md`, `src/compiler/rust.rs`.

No `bench-builds` campaign ran. Wall-time and cold-build claims below are reasoned
hypotheses unless labelled *Measured*.

## Current capabilities

| Tool | What exists | Owner |
|---|---|---|
| Feature unification | Nightly `-Z feature-unification`, `[resolver] feature-unification = "workspace"`, `fine-grain-locking`, resolver 3 | `.cargo/config.toml`, ADR-0122, ADR-0175 |
| sccache | Managed wrapper `scripts/sccache` → `scripts/compiler_cache.py`. `scripts/build_environment.py` forces client-side mode, sets `SCCACHE_DIRECT=false`, a 100 GiB cache at `~/.cache/pse-arrow/sccache` and a dedicated socket, and strips inherited `CARGO_TARGET_DIR` | `docs/dev/build-performance.md`, `.config/build.toml`, `.config/sccache.toml` |
| C/C++ launchers | `.config/native-cache.cmake`, explicit KLU launchers | build guide, native preparations |
| Measurement | `just bench-builds` (cold cache, second worktree, frontend screens), `just build-cache-probe`, `just build-uncached` | build guide |
| Profiles | Workspace crates: `opt-level 2`, incremental. All dependencies (`package."*"`): `opt-level 3`, non-incremental, `debug = false`. `build-override`: `opt-level 2`, which the `package."*"` override takes precedence over | `Cargo.toml` |
| Dependency hygiene | `just deps-report` runs `audit-shear`, `audit-machete`, deny and audit. Advisory only; `scripts/audit_tools.py` classifies exit codes. `just udeps` is separate | `scripts/validation_scope.py` |

The live statistics show the cache is doing real work. Rust: 23 406 hits and 10 003 misses
(70 % hit rate); C/C++: 97.6 % hit rate. The cache holds 83 GiB of its 100 GiB budget. Average
cache read is 0.003 s, against 3.78 s for an average compile. Non-cacheable calls: crate type
7 160 (build scripts, proc-macros, test harnesses, the `pse-py` cdylib), incremental 6 303
(workspace crates), multiple input files 4 389, missing input 1 291. There were 12 Rust cache
errors and no write errors. *Measured* for that statistics window only.

<a id="O01"></a>
## O01: sccache 0.18.0 with a declared version

**Proposed.** Upgrade from 0.17.0 to 0.18.0 and declare the version in
`.config/diagnostic-tools.toml` `[cargo-tools]`. That section already pins the
protocol-sensitive tools (nextest, cargo-hack, llvm-cov, mutants, fuzz) for bootstrap and
reuse; bootstrap then installs sccache and doctor reports its version. Today sccache is
neither in the bootstrap tool list nor declared anywhere. Unlike the advisory tools, it
changes every build's execution, so it is the declaration that matters most. Whether
cargo-shear should be declared too follows from O08: once it runs in `just hygiene`, its
findings depend on its version. ADR-0165's exact-pin rule covers declared dependencies; this
extends the same discipline to a build input.

The release notes give 0.18.0 three improvements that matter here, none of them headline features:

- The assembler gcc/clang would invoke is now part of the cache key (#2843). This workspace
  compiles assembler through the native C/C++ launchers (5 803 assembler hits observed), so
  this closes a wrong-object risk for toolchain changes. It is a correctness gain, not a speed
  gain.
- Daemonization uses an allow-list of inherited file descriptors (#2841), fixing `make`
  deadlocks caused by leaked jobserver descriptors, and children that cannot use the jobserver
  no longer receive it (#2827). This repository has already been hurt by inherited descriptors
  (see the agent-capacity supervision); the fix belongs to the upstream daemon, not to new
  local code.
- gcc flags that need extra caching are marked `TooHard` (#2833), and a preprocessor-check
  miss is fixed (#2824). These matter for the native C/C++ builds.

Keep `client_side_mode = true`, `SCCACHE_DIRECT=false`, the local disk backend and the
100 GiB budget. Multi-level, remote and distributed modes have no consumer on one machine. The
cache is already in the fastest tier, and a read takes three milliseconds. Leave zstd
compression at its default: with local reads that fast, compression cost is not where the
time goes.

To show it delivered: `just build-cache-probe` passes on 0.18.0, doctor reports the declared
version, and a warm `just check` keeps its hit rate. Whether the existing 83 GiB cache stays
readable after the version change was not checked; the same warm check shows it.

<a id="O02"></a>
## O02: correct the cross-checkout premise

**Interface-checked against the sccache v0.18.0 source.** In `src/compiler/rust.rs`, the Rust
cache key hashes:

- the compiler arguments, except `--extern`, `-L`, `--out-dir`, `--check-cfg` and
  `--diagnostic-width`;
- the content of sources and externs;
- every `CARGO_*` environment variable except `CARGO_MAKEFLAGS`, `CARGO_REGISTRIES_*`,
  `CARGO_BUILD_JOBS` and `CARGO_ENCODED_RUSTFLAGS`;
- the compile's **cwd**.

`basedirs` is applied only in `src/compiler/c.rs` and the preprocessor cache. The release note
"strips base directories from the compiler arguments" therefore describes C/C++ arguments
only.

So, for a second checkout such as `just worktree`:

- **Registry crates** (most of the roughly 650 third-party libraries) have cwd and
  `CARGO_MANIFEST_DIR` under `~/.cargo/registry`, which is identical across checkouts, so they
  hit.
- **The eight vendored path packages** in the test closure (`diffsol`, `diffsol-la`,
  `diffsol-nl`, `paste`, `pest_derive`, `py_literal`, `surrealdb`, `surrealdb-engine-api`)
  have checkout-specific `CARGO_MANIFEST_DIR` values, so they miss in every new checkout.
- **Workspace crates** are already non-cacheable because they are incremental. Even without
  incremental, they would miss for the same path reason.

The build guide's sentence "Across checkouts, the shared sccache serves identical compilations"
is true for registry crates only. The overview's claim (§3.3) that `SCCACHE_BASEDIRS` helps
"Git worktrees" holds for C and C++, not for Rust.

Recommended: correct both texts, and write the boundary down so nobody tunes basedirs for
Rust. Candidates, not selected:

- Set `basedirs` for native C/C++ builds only if a probe shows their staging paths vary per
  generation **and** cause misses. The 97.6 % C/C++ hit rate suggests they mostly do not.
- An upstream sccache change that normalizes cwd and `CARGO_MANIFEST_DIR` under basedirs for
  Rust. Dependencies here build with `debug = false`, so their rlibs are probably relocatable.
  Still, the change needs upstream review of `file!()` and panic-location paths, so it is
  reasoned, not established. The value is limited to eight vendored packages per new
  worktree.

<a id="O03"></a>
## O03: keep the mixed incremental strategy

**No change; this records the reasoning.** The overview presents disabling incremental
compilation as a lever (§3.4). Here it does not pay. The 6 303 incremental non-cacheable calls
are workspace crates. Within one checkout, incremental compilation already beats a
whole-crate cache for the edit loop. Across checkouts, O02 means a non-incremental workspace
crate would still miss. Turning incremental off would make edit latency worse and buy no
reuse. Other target directories inside the same checkout, which share cwd, are the only place
it could help: `target/frontend-N`, coverage, `target/measure-*`. Those change rustflags or
profile anyway (`-Zthreads`, instrumentation, the wrapper-disabled `producer` profile), so the
key differs regardless.

Revisit if agents move to many short-lived worktrees that each compile the full workspace once,
and only after a measured sccache change for Rust paths (O02). Even then, the selection would
be scoped (`CARGO_INCREMENTAL=0` for that worktree's non-interactive builds), never a global
profile change.

<a id="O04"></a>
## O04: attribute Rust misses before tuning further (conditional)

10 003 Rust misses at about 3.8 s each is roughly ten CPU-hours of compiles in the observed
window. That is the largest remaining sccache lever, but the reasons behind it are unknown. Likely
sources: the force-validate feature variant (arrow, arrow-array, arrow-data, arrow-cast and
everything downstream), profile variants (release, profiling, dist), frontend-screen
rustflags, toolchain date bumps, and lockfile moves.
Each of these is a legitimate distinct key, not a cache defect.

**Candidate.** No new machinery. Use the existing `bench-builds` campaigns, which already
capture cache statistics. Take one zero-stats window over a typical day of recipes, plus a
debug-logged run in a disposable foreground server (`SCCACHE_LOG`, not `SCCACHE_ERROR_LOG`,
which disables client-side mode). Inspect the 12 Rust cache errors in the same pass. Pursue it
only if the misses turn out to come from an accidental key difference, such as an environment
variable that varies per recipe.

<a id="O05"></a>
## O05: cargo-shear findings to zero, fixed at their causes

**Proposed. The finding counts are *Measured*: `cargo shear --format json --locked`, 1.14.0,
baseline `95ee5dfdd`.** Shear reports 43 errors and 18 warnings against a zero target. The run
takes 0.4 s wall time and compiles nothing. Classified:

| Class | Examples | Resolution |
|---|---|---|
| Macro-only use (13 `miette` entries) | `pse-ids`, `pse-schema`, `pse-compiler`, `pse-quantity`, `pse-rules`, … | `pse-diagnostics`' `macro_rules!` emits `impl miette::Diagnostic for $ty`, so every caller needs `miette` in scope. Shear is right that the source never names it. Fix the macro, not the manifests: a `#[doc(hidden)] pub use miette;` in `pse-diagnostics` with `$crate::miette::…` paths. The 13 declarations then become genuinely removable, with no ignore list |
| `#[path = "../x.rs"]` modules shear does not follow | `pse-buildinfo/identity.rs` (reported unlinked, which also flags `pse-ids`, `serde_json`, `sha2` and `tempfile` as unused); `xtask/src/canonical_recovery.rs` | Move the module into the conventional layout (`src/identity.rs`; a `src/` module of the xtask bin), or report upstream. Re-run shear before removing any dependency it named for these crates |
| Empty or orphan files | `pse-modeling/src/kernel_envelopes.rs` (0 lines); `tests/engine/src/oracle.rs` (53 lines, unlinked, last changed 2026-09-14) | Delete, or link if `oracle.rs` still has an owner |
| Truly unused declarations | `pse-columnar` dev `proptest` and `allocation-counter`; `pse-runtime` `object_store` and a duplicate dev `url`; `pse-py` `pse-kernels` and `url`; conformance `object_store`, `proptest`, `serde`, `serde-saphyr`, `tempfile`, `url`; `xtask` `prettyplease`; `pse-rules` `async-trait` and `pse-authoring` | Remove (`cargo shear --fix`, then review the diff), then run `just check` and the affected targeted units |
| Feature-activation-only edges | optional `arrow-cast` in `pse-columnar` (exists to apply `force_validate`); the `force-validate` forwarders in `tests/*` and `benches` | Judge each against ADR-0170. Where the edge activates validation on a crate in the actual closure, keep it with a package-level `ignored` entry and a reason. Where another forwarder already reaches the same crate, delete it |

Use `[package.metadata.cargo-shear] ignored` only for what remains after these fixes. Shear's
redundant-ignore diagnostic keeps that list honest, so it does not need its own review
ritual. `keep-path-dependencies` is unnecessary: there are no pre-declared path dependencies
in `[workspace.dependencies]`.

Removing a declaration only shrinks the build where the package leaves the resolved closure
(overview §4.10). Most of these packages stay reachable transitively. The value is manifest
correctness and a zero baseline that later regressions show up against, not compile time.
`pse-py` dropping `pse-kernels` and the test crates dropping `object_store` are the likeliest
closure reductions; the unit-graph census confirms which ones actually shrink.

To show it delivered: `cargo shear` exits 0 with no ignores beyond the justified ones, and
`just check` passes.

<a id="O06"></a>
## O06: adopt shear's test-target analysis

**Measured finding count:** `cargo shear --check-test-targets` adds 22
`doctest_enabled_without_doctests` and 5 `test_enabled_without_tests`. `just doctest`
and `cargo test` run rustdoc's doctest pass on every library. Setting `doctest = false` on
crates without doctests removes those invocations, and `test = false` removes empty
harness links. These are ordinary Cargo target keys, so the change is native and local. Include
the flag in the routine shear invocation so a crate that gains its first doctest is reported
instead of silently skipped. The saving per run is *Proposed*, since doctest timing was not measured.

<a id="O07"></a>
## O07: one dependency-hygiene detector

**Proposed: retire cargo-machete.** On this workspace, machete's findings are a subset of
shear's, plus false positives shear avoids: dependencies that are used only by build
scripts, such as `cc`, `pkg-config` and `sha2` in the `-sys` crates, `toml` in
`pse-backend-native`'s `build.rs`, and `pyo3-build-config`. It also walks `vendor/`,
`tooling/` and plan probe crates, which are not members. During this review, its
`--with-metadata` run rewrote six tracked `vendor/*/Cargo.lock` files and created three new
ones, which then had to be restored by hand. That side effect is reason enough to retire it. Two detectors for one fact break the
one-authority-per-fact principle (DP-01). Drop `audit-machete` from `deps-report`, its recipe
and its bootstrap entry.

**cargo-udeps: decide with one comparison.** udeps compiles, so it sees macro expansion and
`cfg` the way the compiler does, which is shear's documented blind spot (§4.6). Shear's
`--expand` covers the same ground with one tool. In the plan, run `cargo shear --expand` and
`just udeps` once. If udeps finds nothing that `--expand` misses, retire it as well and keep
`--expand` as the occasional compiler-backed check. The default analysis remains the routine
one.

<a id="O08"></a>
## O08: route shear into `just hygiene` once at zero (maintainer decision)

At 0.4 s with no compile, shear costs less than any other hygiene step. Once O05 reaches zero,
running it in `just hygiene` instead of only in the advisory `deps-report` keeps the baseline at
zero (prime directive 1) without a new lint: it is the existing detector in the existing scope-end
bundle. It must never run in the edit hook or `turn-end`, in line with the execution rhythm. This
changes which bundle owns an existing check, so the maintainer decides. The alternative is to
leave it advisory and accept that findings will return between scope ends.

<a id="O09"></a>
## O09: cargo-hakari as challenger, not re-adopted

**Interface-checked: Cargo unit graphs, baseline `95ee5dfdd`.** The question was whether
hakari can remove duplication that native unification leaves.

1. **Across selections, native unification already holds.** `check -p pse-model` against
   `check --workspace`: 0 of 69 shared third-party packages differ in feature set. The only
   differences come from opt-in features, as intended. Force-validate changes `arrow`,
   `arrow-array`, `arrow-data` and `arrow-cast`. `native-solvers` changes 11 packages:
   `clarabel`, `pounce-nlp`, `tracing-subscriber`, `tracing-log`, and the host side of
   `regex`, `toml_*`, `winnow`, `either`, `indexmap`, `prettyplease`. The `pse-py` lib
   build differs from the workspace test build in seven packages because of dev dependencies
   and force-validate. Hakari's ADR-0122 configuration excluded these features, so it would
   not have changed any of them.
2. **Within one selection, the host/target split remains.** `test --workspace` builds 32
   third-party packages twice. 25 have different features on the host side (proc-macros and
   build scripts) and the target side: `proc-macro2` (`span-locations`), `serde_json`,
   `regex`, `regex-automata`, `prost`, `indexmap`, `hashbrown`, `libc`, … The other 7
   inherit the split from those, for example `syn`, `quote` and the `toml` family. The
   `pse-py` build has 27. Profiles do not cause these, because `package."*"` overrides
   `build-override`, so both roles get `opt-level 3`. In `cargo check`, a further 58 packages
   split into check and build, which no feature setting can remove. Cargo's unstable flags
   (`-Z feature-unification`, `host-config`, `target-applies-to-host`) have no host/target
   feature unification; hakari's `unify-target-host` is the only mechanism.
3. **The cost of the mechanism.** A workspace-hack makes every member depend on every unified
   package. ADR-0175 measured that removing it took `pse-engine` checks from 423 to 382
   units and packages from 285 to 273. That is roughly the same number of units as the host
   duplicates it would remove, and it is paid by every small selection rather than only by
   full workspace builds. The host duplicates are small leaf crates. The warm `target/`
   keeps both variants, and sccache caches both, so they cost time only on a cold build of
   a fresh checkout.

The judgment: hakari's benefit here is a cold-build-only saving on about 30 small crates. Its
price is a crate addition, an ADR, a regeneration route, and wider closures for the
`check-package`-style selections agents run most. The comparison is reasoned from unit counts.
Neither side's wall time was measured, so it stays a judgment, not a measurement.

Revisit if Cargo gains native host/target feature unification. Or if a cold-build timing from
`bench-builds` shows host-duplicate crates on the critical path. Or if fresh-worktree builds
become the dominant workflow. A lighter alternative exists for the `native-solvers` split:
declare the extra `tracing-subscriber` features in `[workspace.dependencies]`. That uses
Cargo itself, but the dependents gaining from it are few, so it is not recommended without a
consumer.

ADR-0175 keeps its decision. The installed `cargo-hakari` binary is harmless and not
in bootstrap. The census probe answers the "does duplication remain?" question from Cargo's own
resolution, so no hakari diagnostic route is needed.

<a id="O10"></a>
## O10: Cargo checksum freshness (adjacent, candidate)

Outside the three tools, but the same workflow: the pinned nightly offers
`-Z checksum-freshness`, which decides freshness from file content instead of mtime. The
workspace already relies on `[unstable]` keys in `.cargo/config.toml`. Git checkouts,
stashes, rebases and rewrites that leave content unchanged update mtimes, and today they
trigger rebuilds of that crate and everything downstream. With checksum freshness they would
not. The cost is hashing sources on each freshness check, and adopting it invalidates
fingerprints once.

**Candidate with a probe:** touch, without changing, a lower-layer source file such as
`pse-ids`, then compare the `just check` rebuild count and wall time with the flag on and off.
Adopt only if the probe shows the avoided rebuilds. The flag addresses the mtime half of the
shared-build-directory problem (`.cargo/config.toml`, ADR-0122). Cargo still keys units by
workspace-relative path, so per-checkout `target/` directories remain correct.

## Interactions and priorities

- **Do first, low burden, no ADR:** O01 (version and pin), O02 (documentation correction),
  O05, O06 and O07 (manifest edits, macro re-export, file layout, recipe removal). O05's macro
  change touches `pse-diagnostics` and every crate that uses its derive macro in one
  change. It needs `just check` and the diagnostics unit tests.
- **After O05:** O08's routing decision.
- **Conditional:** O04 (only if the miss attribution is wanted), O10 (only after its probe),
  the O02 candidates.
- **Not recommended:** re-adopting hakari (O09), disabling incremental globally (O03),
  remote, multi-level or distributed sccache.

Planning mostly consumes this review through `create-plan`, as a tooling plan. None of the
selected opportunities changes a D1–D14 decision or adds a crate, so no ADR is required. O02
edits a non-authoritative guide and the library overview. If the overview correction is
wanted, its scraped favicon and link debris (the `[image](https://www.google.com/s2/…)` lines)
can go in the same pass.

## Sources

- [Rust caching overview](../libraries/rust_caching.md) §2.4, §3.3, §3.4, §4.3, §4.6, §4.9,
  §4.10, §5.
- sccache v0.18.0: `README.md` ("Normalizing Paths with `SCCACHE_BASEDIRS`", "Known
  Caveats"), `docs/Configuration.md` (`basedirs`), `src/compiler/rust.rs` (hash-key
  construction, steps 3–10), release notes (#2824, #2827, #2833, #2841, #2843).
- Cargo unstable flags as listed by `cargo -Z help` on `nightly-2026-09-29`.
- Repository owners: ADR-0122, ADR-0165, ADR-0170, ADR-0175, `docs/dev/build-performance.md`,
  `docs/dev/agent-environment.md`. Design standard 3.4: DP-01, DP-09, DP-13, DP-14,
  DP-16, AP-07.
