---
title: Rust build performance and persistent compilation reuse
status: in-progress
date: 2026-09-24
adrs: []
phase: 1
---

# Rust build performance and persistent compilation reuse

## Context and recommendation

This is a tooling plan alongside [Plan 14](14-library-owned-process-simulator.md).
It does not replace the simulator design or claim M22 qualification. The scope is
faster builds and executable test feedback, without prescribing more `cargo check`
use or restructuring crates.

**Recommendation:** enable sccache for reusable dependencies while retaining local
incremental compilation; preserve artifacts with bounded retention; qualify a dated
nightly LLVM toolchain with a parallel frontend as the normal local development
choice. Keep pinned stable LLVM for canonical qualification and releases. Then reduce
avoidable build variants and measure dependency optimization settings. Retain existing
debug settings. Mold is the user-selected Linux linker; linker comparison is excluded. Cranelift is a later
compatibility experiment, not the recommended simulator development default.

The initial assessment changed no build configuration, dependency pin, installed
toolchain or cache. Implementation is now in progress; the checkpoint below records
the execution boundary. CLI/graph probes establish configuration, not speedups.

## Current evidence

Inspected on 2026-09-24: manifests/configuration, just recipes, native preparation,
measurement scripts, CI setup, retained M21 logs, installed tools and Cargo unit graphs.
The local `rust-code-model` skill's `content/topics/project-context.md` and
`content/layers/cargo-metadata.md` guided package/feature inspection. These local-only
references are not required to build the documentation. Declared manifest features
and metadata package lists do not establish the invocations or enabled features of
an individual build.

| Finding | Evidence and implication |
|---|---|
| Local sccache activation is missing | Refreshed shell: `RUSTC_WRAPPER` and `RUSTC_WORKSPACE_WRAPPER` unset; no user Cargo config found. `.cargo/config.toml` says justfile/direnv sets the wrapper, but neither currently does. Installed version: 0.17.0. |
| Existing cache data does not establish this workspace's reuse | `sccache --show-stats`: zero requests in current server statistics, 10 GiB limit. Default disk cache occupies about 11 GiB. This does **not** mean it is empty or was never used. Measure per-command counter deltas. |
| Incremental compilation is already active | Probed workspace units use incremental compilation; registry units do not. `target/debug/incremental` occupies about 12 GiB. |
| Debug-information reductions already exist | Dev uses `line-tables-only`; dependency debug info is disabled; `debugging` enables full workspace debug info. Reapplying this advice saves nothing. |
| Dependency optimization is broad | `[profile.dev.package."*"].opt-level=2` includes build tooling: probed `cc` and three `syn` versions use level 2. This wildcard takes precedence over `build-override`. |
| Delta needs separate cache consideration | `deltalake-core` comes from captured source in `vendor/delta-rs/crates/core`, not a published crate. Its path-dependency unit uses level 2 **and incremental compilation**, so enabling sccache alone will not cache it. Preserve its source pin. |
| A fast linker is already the default | No repository override. rustc 1.98.1 on `x86_64-unknown-linux-gnu` defaults to rust-lld. Installed mold 2.42.0 is available for comparison. Confirm actual link commands during measurement. |
| Host LLVM differs from compiler LLVM | rustc 1.98.1 reports bundled LLVM 22.1.8; the refreshed shell finds native Clang/LLD 23.1.2. Host LLVM/Z3 installation changes do not upgrade rustc's backend. |
| Artifact pressure is material | Approximately 30 GiB in `target/debug/deps`, 12 GiB incremental, 339 MiB build-script output; filesystem 88% used, 193 GiB available. Point-in-time observations, not a growth forecast. |
| Native preparations are lost to broad cleaning | KLU installs into `target/native-math/klu-profile-v1`; solver image extraction uses `target/native-solver/<digest>`. They occupy only about 6.2/8.5 MiB here: improving their persistence is useful but not the largest disk saving. |
| CI already caches artifacts | The shared setup action uses pinned `Swatinem/rust-cache`, including workspace crates; main writes caches. Several workflows disable incremental compilation, and wheels enable sccache. Local and CI policies differ. |
| Measurement machinery already exists | `scripts/build_measurements.py` snapshots dirty source and retains Cargo artifact JSON/timing HTML. It currently forces wrappers off, uses an older workload and one repetition. Extend it instead of replacing it. |

Host capacity: 16 physical cores / 32 logical CPUs, approximately 188 GiB RAM;
the inspected shell has affinity to all 32 CPUs. Record effective quotas and other
active builds during measurement; hardware capacity alone does not select concurrency.

### Observed rebuilds and distinct build graphs

Retained logs report 4m39s for the native Python extension and 2m24s for subsequent
xtask compilation during post-clean recovery. Native test enumeration took 3m45s.
These are **Measured observations**, from `/tmp/pse-m21-rebuild-clang.log` and
`build/plan14/m21-development/plan14-native-enumeration.log`, with different command
graphs and evolving cache state. They are not controlled cold-build comparisons or
a repeatable aggregate baseline. Temporary logs may disappear.

**Interface-checked:** installed `nightly-2026-09-13` reports rustc
`1.100.0-nightly (809936eac 2026-09-12)`, LLVM 23.1.1. No-build probes produced:

| Selection | Build units | Important differences |
|---|---:|---|
| `xtask --no-default-features` | 429 | Arrow lacks `force_validate`; narrower DataFusion/faer features. |
| `pse-py`, `force-validate,native-solvers` | 758 | Arrow validation, broader DataFusion features, faer `rayon`/`npy`/`rand`, PyO3 extension-module linking. |
| Native development packages, library/test targets | 772 | `pse-relations` additionally has `test-support`; test artifacts differ from the extension. |
| xtask with nightly workspace feature unification | 518 | More units than 429; Arrow still lacks `force_validate`. This option does not enable all optional features. |

Probe prefix: `cargo +nightly-2026-09-13 build --offline --locked --unit-graph
-Zunstable-options`. Native selectors: `pse-backend-native`, `pse-compiler`,
`pse-kernels`, `pse-math`, `pse-runtime`, `pse-structural`, `pse-tests-conformance`,
with `--lib --tests` and
`pse-relations/force-validate,pse-runtime/native-solvers,pse-tests-conformance/native-acceptance`.
These probes compile nothing; they are not exact maturin invocation traces.

The retained metadata snapshot contains 651 packages and 30 workspace members,
including multiple `syn`, `hashbrown`, `itertools` and numeric/compression versions.
That is an audit input, not a count of crates compiled by every command. Nightly also
reports two unused-workspace-dependency warnings: `arrow-data` and
`datafusion-physical-plan`. Check their pin/governance role before removal; do not
silently accept or suppress warnings when qualifying nightly.

## Cache ownership

| Mechanism | Reuses | Does not solve |
|---|---|---|
| Cargo registry/git cache | Downloaded sources | Compilation/linking |
| Cargo target/build directories | Fresh libraries, build-script outputs and final artifacts | Different compiler/features/flags/target/ABI inputs |
| rustc incremental state | Prior work within changing workspace/path crates | Erased build state or cross-toolchain reuse |
| sccache | Eligible whole compiler invocations, especially registry libraries and supported native compilations | Incremental Rust invocations; executable/cdylib/proc-macro linking; execution of build scripts |
| Native caches/install prefixes | Built libraries and extracted images | Changed compiler/ABI/library configuration |

Cargo's [build-cache documentation](https://doc.rust-lang.org/cargo/reference/build-cache.html)
distinguishes final and intermediate artifacts. Cargo freshness should win first:
an unchanged build need not invoke sccache. Low hit counts can coexist with excellent
reuse; many hits can conceal avoidable Cargo rebuilds.

Z3 has no role in Cargo/rustc caching. rustc's own
[incremental query system](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html)
tracks compiler dependencies and reuse. Product Salsa does not replace it. Do not
introduce a Z3/Salsa build scheduler or bespoke compiler artifact cache.

## Recommended changes

### 1. Enable compiler caching without disabling workspace incremental compilation

Use Cargo's existing `RUSTC_WRAPPER` integration consistently across just, direnv,
maturin subprocesses and noninteractive shells. Explicit cache-enabled execution must
report that it is active. Respect user wrapper overrides and retain an uncached
diagnostic mode. Installing sccache without activating it is insufficient.

Keep incremental compilation for changing workspace crates. Registry dependencies are
already nonincremental in the inspected graph. Evaluate a **package-specific**
`incremental=false` override for effectively immutable vendored dependencies such as
the captured Delta crate, with an editing opt-out. Do not set `CARGO_INCREMENTAL=0`
globally on this workstation. See the exact
[sccache 0.17.0 Rust exclusions](https://github.com/mozilla/sccache/blob/v0.17.0/docs/Rust.md)
and [Cargo's path/workspace incremental behavior](https://doc.rust-lang.org/cargo/reference/profiles.html#incremental).

Start with a proposed 32 GiB sccache budget outside `target`; adjust from working-set
size, eviction and free space. Use built-in eviction/statistics. Choose one server
owner per cache directory, respecting the
[0.17.0 local-cache contract](https://github.com/mozilla/sccache/blob/v0.17.0/docs/Local.md).
Do not stop another workspace's server. Verify effective server configuration.

Cross-worktree/repository reuse depends on compiler inputs, paths and dependency
identities. Do not assume current README path-normalization examples work for Rust
in installed 0.17.0. Prove representative dependency cache hits; a small transport
canary alone does not establish workspace benefit.

### 2. Qualify dated nightly frontend parallelism with LLVM

Use installed `nightly-2026-09-13` as the initial candidate. Pin one qualified date;
avoid floating upgrades and nightly-only language features in product source.

The [Cargo performance guide](https://doc.rust-lang.org/cargo/guide/build-performance.html#enable-the-experimental-parallel-frontend)
suggests testing up to eight frontend threads. Start at four, compare 1/2/4/8 with
Cargo jobs held at 16, then compare 16/24/32 jobs for the winning frontend setting.
These are host-specific experiment candidates, not prescribed universal limits.
Rust's [jobserver integration](https://blog.rust-lang.org/2023/11/09/parallel-rustc/)
coordinates compiler concurrency: Cargo jobs times frontend threads is not an
independent multiplicative thread budget. Separate Cargo invocations, linkers and
native tools can still compete. Use one build owner per active target tree.

**Interface-checked:** this nightly's `-Zhelp` now directs `-Zthreads` users toward
`--jobs-frontend`. CLI-only `--print cfg` probes gave these results:

| Options | Stable 1.98.1 | nightly-2026-09-13 |
|---|---|---|
| `--jobs-frontend=4` | Expected rejection: unknown option | Expected rejection: requires unstable-options |
| `-Zunstable-options --jobs-frontend=4` | Not applicable | Accepted |
| `-Zthreads=4` | Not applicable | Accepted |

Use the accepted `-Zthreads=4` spelling initially and verify it through sccache.
Adopt the newer spelling only after the installed cache parser accepts and caches it.
The [new compiler CLI documentation](https://doc.rust-lang.org/beta/rustc/command-line-arguments.html)
is ahead of the stable pin; published syntax is not proof of local support.

Use command-scoped settings and one persistent nightly target directory shared by
compatible recipes. Keep stable artifacts separate to avoid continual toolchain
switching in the active cache. Do not create a target directory per test or recipe.
Encode flags once, preserve required caller flags and account for
[rustflags precedence](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags).

After qualification, make this the documented local development route through the
existing command surface. Retain `rust-toolchain.toml` and its stable MSRV relationship
for canonical gates. Before changing default-toolchain governance, prepare the
ADR/design review required by ADR-0018 and AGENTS.md; do not edit the accepted record.

### 3. Preserve artifacts and native prerequisites within a storage budget

Replace broad-clean recovery advice with inventory and explicit selection of obsolete
experiment/profile/toolchain outputs. Preserve active incremental state, dependency
downloads, sccache and native installations. Retain stable plus the selected nightly
working sets and at most one temporary candidate. A proposed local free-space floor
is 50 GiB; below it, reclaim identified inactive outputs before starting large runs.

Move the existing native preparations to one XDG-aware, overridable persistent prefix.
Keep solver extraction keyed by image digest. Key KLU by source identity, target,
compiler identity, ABI-affecting flags and exact CMake options. Reuse flock, publish
completion atomically and verify required installed files. Update loaders and evidence
discovery to use the same resolved prefix; delete replaced target-only path logic.
No duplicate cache authority or committed home paths.

Pinned `gmp-mpfr-sys` 1.7.1 already provides persistent caching through
`GMP_MPFR_SYS_CACHE` and platform defaults; preserve this library-owned mechanism.
For HiGHS/SUNDIALS/KLU, use supported native compiler launchers where appropriate,
following [sccache's CMake integration](https://github.com/mozilla/sccache/blob/v0.17.0/README.md#usage).
Verify actual C/C++ compilation traverses the launcher: the Rust wrapper alone does
not cache it. Inspect each pinned build script before assuming environment settings
reach CMake. Do not invent Fortran caching or substitute unpinned system solvers.

Make native compiler/resource-directory discovery reliable in noninteractive recipes.
The recent terminal-refresh fix should not become a dependency on personal interactive
shell initialization. Record native identities without logging license values.

### 4. Reduce accidental build variants and measure optimization levels

Keep crate boundaries intact. Inventory recipe package/features, compiler, profile,
explicit/implicit target, flags, Python ABI and native inputs. Measure
`py-sync-native` → stub generation → native units, including returning to an earlier
selection. Different features legitimately require different compiler artifacts;
sccache cannot merge those invocations.

Batch compatible selectors using Cargo; align accidental flag/environment differences.
Retain intentional force-validation, test-support, PyO3 linking, bootstrap and production
measurement differences. Tests keep explicit force-validation. Stub generation keeps
compiled-boundary introspection. Do not add dummy dependencies or run a stale xtask
binary to bypass Cargo freshness.

The nightly [feature-unification option](https://doc.rust-lang.org/cargo/reference/unstable.html#feature-unification)
requires its `-Zfeature-unification` switch as well as configuration. The observed
429→518 xtask graph expansion argues against global adoption. Keep selected-package
unification unless total workflow measurements and isolated-package correctness prove
a better choice. Bootstrap and extension/test boundaries require particular care.

Compare wildcard level 2 against level 1 with named level 2 overrides only where
execution measurements justify them. Address build tooling separately: adding
`build-override.opt-level=0` beneath the current wildcard does not override it.
Cargo documents [precedence and generic sharing](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides-and-generics).
Generic code can be instantiated in our crates, so optimizing a dependency does not
necessarily optimize the consumer's numerical loop. Select for build **plus test
execution** latency; scientific/performance qualification retains canonical optimization.

Audit duplicate versions/features by cost and consumer requirements. DataFusion's
umbrella defaults are enabled; inspect current consumers and the captured Delta source
before trimming them. Multiple support-library versions are distinct from violations
of the single Arrow/DataFusion/object_store family invariant. Preserve pins and required
capabilities. No blanket dependency upgrade or zero-duplicate mandate.

### 5. Other options

| Option | Disposition |
|---|---|
| mold | Already selected by the user before implementation. Retain it; no linker comparison or separate mold qualification campaign. |
| Wild linker | Defer until LLD/mold leave a measured linking bottleneck, then check native/PyO3 compatibility. |
| Cranelift | Defer default adoption: [upstream](https://github.com/rust-lang/rustc_codegen_cranelift#not-yet-supported) lists partial `std::arch` SIMD and experimental panic unwinding. Our numeric stack, `catch_unwind` and PyO3 require explicit proof. Never switch to abort to make it build. Slower tests can erase compilation savings. |
| `hint-mostly-unused` | Test only on measured large, sparsely used dependencies. [Its contract](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/hint-mostly-unused.html) defers codegen to consumers; cost may move onto frequently rebuilt crates. |
| `-Zno-embed-metadata` | Optional pinned-nightly disk experiment after checking cache compatibility and artifact consumers; [removes duplicated embedded metadata](https://doc.rust-lang.org/cargo/reference/unstable.html#no-embed-metadata). |
| `-Zchecksum-freshness` | Consider for proven checkout/mtime-driven CI rebuilds. It currently [does not replace build-script input mtime checks](https://doc.rust-lang.org/cargo/reference/unstable.html#checksum-freshness). |
| Codegen units / LTO | Leave dev defaults initially. Tune only for measured codegen cost; additional parallelism can cost memory/runtime quality. Keep release fat-LTO/one-codegen-unit settings out of dev. |
| Full debug / split DWARF | Retain explicit debugging support. Existing dependency overrides still disable dependency debug info; a dependency-debugging mode needs its own deliberate override. |
| Remote sccache / distributed compilation | Defer until local reuse works and measured multi-machine or cold-build demand justifies transfer/service overhead. |
| Build-dir relocation / shared targets | Storage placement is not automatic worktree/toolchain reuse. Keep current Rust layout first; any later relocation must update path consumers and account for clean behavior. |
| Nightly locking/layout experiments | Defer; one build owner per active tree is preferable to new shared mutable-cache machinery here. |
| More `cargo check` / crate splitting | Explicitly outside scope; no new check-first workflow. |

## Measurement design

Extend `scripts/build_measurements.py` and `just bench-builds`, preserving exact dirty
source snapshots and supported compiler-artifact JSON. Replace obsolete workload/edit
anchors with current selections; parameterize compiler/wrapper/settings. Preserve
[Cargo timing HTML](https://doc.rust-lang.org/cargo/reference/timings.html), without
parsing its private JavaScript or equating summed crate time with the critical path.

Use supported nightly `-Zsection-timings` and `-Zbuild-analysis` report commands for
section costs and rebuild explanations. These [built-in diagnostics](https://doc.rust-lang.org/cargo/reference/unstable.html#build-analysis)
are preferable to a custom analyzer. Separate diagnostic runs from timed comparisons
if instrumentation changes artifacts or adds material overhead.

| Workload | Controlled question |
|---|---|
| Unchanged repeat | Warm target/incremental/sccache: do expected units remain fresh? |
| Private body edit | Controlled semantic edit to an existing leaf in an isolated snapshot: executable feedback time. |
| Central crate edit | Edit a frequently rebuilt compiler/relations crate; observe downstream work without changing crate boundaries. |
| Python + stubs + native units | Actual consecutive graphs, then return/repeat: avoidable churn versus required variants. |
| Artifact recovery | Empty disposable target, warm compiler/native caches: benefit after artifact loss. |
| Cold compiler cache | Empty task-owned target/cache; identify separately cached downloads/native prerequisites. Never erase the user's cache or drop filesystem caches. |
| Native recovery | Disposable native prefix; separate extraction/configure/compile/link costs and upstream cache reuse. |
| Second worktree | Same source/lock/compiler/features, different path and separate target: prove actual shared-cache reuse. |

Record commands, source/lock identity, rustc commit, linker/native compiler, target,
allowlisted environment, features, cache state, sccache deltas, fresh/rebuilt units,
wall/section/test time, artifact size, process-tree peak memory and disk availability.
The existing Python `ru_maxrss` is a cumulative child high-water mark, not an independent
per-workload process-tree peak. Use isolated sample processes and an appropriate sampler.

Use at least three paired repetitions for short/warm workloads and report dispersion.
Screen with one cold run, replicating only leading configurations before cold-speedup
claims. Alternate order where practical; avoid or record competing builds. Promote
improvements larger than observed noise without material regressions in frequent
workflows, resource use or correctness. No promised percentage before a baseline.
Preserve rejected candidates' reasons rather than their target trees.

## Execution packets

Use the existing just/scripts surface. These are tooling work items, not new simulator
packages or a new build system. Implementation is in progress; no packet is fully
qualified yet.

| Packet | Work and dependencies | Targeted acceptance and deletion |
|---|---|---|
| B00 — Baseline | First: update existing runner for current selections, mode identity, cache deltas, isolation and repeatability; collect unchanged/edit baseline. | Fixture tests for capture/parser/isolation; original dirty source unchanged. Replace obsolete edit anchors and wrapper-disabled-only behavior. |
| B01 — Compiler caching and retention | After B00: common environment activation, qualify installed sccache, explicit disk budget, preserve incremental workspace builds, correct stale config comments. | Subprocess tests for enabled/disabled/missing-wrapper and nested propagation. Disposable-cache repeated eligible compilation hits; incremental compilation passes through. No global incremental disable. |
| B02 — Nightly LLVM route | After B01: one date pin, command-scoped flags/toolchain, persistent target identity, cache-parser qualification, thread/job matrix. Prepare required governance amendment before changing documented defaults. | CLI controls and force-validated library units; propagation through Cargo/maturin/xtask. Stable route has no unstable flags. No floating nightly or `RUSTC_BOOTSTRAP`; remove conflicting flag selection. |
| B03 — Native persistence | After B01, using B02 when available: portable prefixes, complete identities, atomic reuse, upstream GMP cache, supported native launchers, reliable Clang discovery. | Disposable-prefix tests for identity misses, interrupted installs, concurrent preparation and lost files; explicit IPOPT_DIR/CI image still works. Delete target-only native path assumptions. |
| B04 — Variants and profiles | After B00–B03: align accidental invocation drift, batch compatible selectors, compare level 1/selective level 2, assess immutable path-dependency incremental overrides. | Feature controls retain validation/bootstrap/PyO3 contracts; native boundary units pass. Remove only proven-unused dependencies or superseded recipe branches; retain old settings when experiments lose. |
| B05 — Remaining bottlenecks | Conditional on measurements: mold, mostly-unused hints, metadata size reduction or CI mtime handling; Cranelift remains separate and optional. | Relevant executable/FFI/unwind units and improved paired timings. Delete rejected settings and task-owned artifacts; no blanket unstable-option bundle. |
| B06 — Final qualification | Last: selected development configuration and canonical stable qualification; publish measured results and update docs/recipes/CI where evidenced. Coordinate shared journeys with Plan 14 M22. | Full relevant format/lint/governance/codegen/docs and Rust/Python/native qualification, zero failures against baseline zero. Measured warm/edit/recovery comparisons; budgets and stable fallback verified. |

Avoid a full Cartesian experiment matrix. Screen baseline → cache → frontend/jobs →
native/variant/profile improvements → conditional linker/backend options. Qualify the
combined winner because independent improvements may interact.

### Integration requirements

- Propagate the chosen environment through existing validation/Plan 14 runners and
  maturin. Inspect explicit `rustup run <stable>` calls: environment variables do not
  override an explicitly selected toolchain. Canonical paths remain stable; development
  paths deliberately select the dated nightly.
- Evidence must record the compiler/configuration and native bytes actually used.
  Recollect affected receipts after changes; cache hits do not replace provenance.
  The maintainer removed the M21 source seal during this implementation. B06
  coordinates M22 and records fresh execution evidence for the new artifacts.
- `pse-buildinfo/build.rs` intentionally tracks crate/vendor sources. Diagnose rebuilds;
  do not drop provenance inputs for speed. Avoid rewriting identical generated output
  where supported, preserving generator/content contracts. Use Cargo's
  [build-script change detection](https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection).
- Preserve unwinding, force-validation and numerical semantics. No `target-cpu=native`,
  fast-math, disabled physical checks or weaker tests.
- CI cache changes retain trust/write policies and canonical compilers. Measure archive
  restore/upload cost before adding caching layers; CI's nonincremental policy does
  not become the workstation default.
- Capture only allowlisted environment fields. Licenses and personal shell files are
  not build-report content.

## Verification

**Interface-checked:** configuration inspection; installed stable/nightly identities;
CLI acceptance/rejection controls; four no-build unit-graph resolutions; sccache
version/statistics; pinned native build-script/GMP cache contracts.

**Measured:** the verified Linux stable campaigns below establish artifact-recovery
cache benefit. Earlier campaigns with changed source or recovery paths remain
diagnostic. The corrected runner preserves recovery paths and separates memory
sampling from wall-clock timing.

**Proposed:** B00–B06. Completion must name commands, compiler/profile, validation mode
and failures against baseline zero. B06 owns broad qualification and final
**Tested**/**Measured** claims. Packet checks establish bounded contracts only;
development-cache success does not establish scientific simulator qualification.

## Open items

- The maintainer accepted one frontend thread and 16 Cargo jobs and stopped further
  concurrency screening. The 32 GiB persistent compiler cache is configured and its
  recovery benefit accepted; no further cache comparisons are required. Dependency
  profile decisions remain open. Mold is settled and excluded from comparisons.
- Nightly and sccache are not yet qualified together on the product. The two unused
  manifest declarations were removed without changing resolved dependency pins.
- Doctor reports an outdated Python environment and an extension import failure in
  the unlinked shell. Resolve through the native recipe environment before Python
  measurements; separate repair time from compilation. `just py-sync-native` rebuilt
  and installed the extension during implementation; native recipes supply its loader
  environment.
- Cranelift becomes actionable only with compatible SIMD/unwind/FFI and a measured
  improvement in total feedback latency.

## Implementation checkpoint

The common environment, persistent native preparations and expanded build runner are
implemented. See [build reuse](../dev/build-performance.md) for commands, ownership
and retention. Stable remains canonical; the dated nightly route is experimental.
The two unused workspace declarations (`arrow-data`, `datafusion-physical-plan`) were
removed; resolved family pins and force-validation remain enforced by existing checks.
Dependency optimization defaults remain unchanged pending paired execution evidence.

Tooling controls exercise explicit/missing/disabled wrappers, subprocess propagation,
flag precedence, native identity invalidation, corrupted/missing files, interruption
and concurrent publication. Stable and dated-nightly disposable cache probes observe
an eligible Rust cache hit and incremental pass-through. Product timing and full
qualification are tracked below as results become available.

## Outcome (recorded after implementation)

### What was built

**Implemented, qualification incomplete:** common compiler/cache environment,
experimental dated-nightly route, persistent native preparations, storage inventory,
isolated measurement controls and unchanged-output writes in `pse-buildinfo`.
The M21 source seal and its CLI/recipe gates were removed at maintainer request;
actual functional-evidence checks for performance remain.

**Tested:** `just setup-test` passed 80 isolated tooling tests, zero failures against
baseline zero. `just unit-consolidation-tools` passed its Python controls and the Rust
historical-plan rejection unit with explicit force-validation. Disposable stable and
nightly cache probes observed eligible Rust hits and incremental pass-through.

**Tested:** the maintainer applied the staged LLVM migration. Subsequent
`python3 scripts/llvm_system.py --verify` passed installed prefix/resource/library,
C/C++ execution, login, noninteractive, standard-PATH and user-service checks.
See [system LLVM selection](../dev/llvm-system.md) for rollback and environment scope.
The existing tracked Cargo mold settings are preserved.

**Measured:** `just bench-builds build/plan15/stable-uncached-verified --cache off
--recovery` and `just bench-builds build/plan15/stable-cached-system --cache on
--cold-cache --recovery --second-worktree --execute` used stable 1.98.1, dev profile,
16 jobs and explicit Arrow force-validation. Both preserved the original source.

| Workload | Uncached | Cached |
|---|---:|---:|
| Cold target/compiler cache | 163.48 s | 194.21 s |
| Unchanged build, median of three | 0.177 s | 0.164 s |
| Private edit, median of three | 0.608 s | 0.659 s |
| Public edit, median of three | 0.615 s | 0.669 s |
| Artifact loss, same target path | 164.21 s | 28.48 s |

Recovery recorded 346 Rust and 108 C/assembler hits. A second worktree took 196.34 s
and recorded zero Rust hits, with 108 C/assembler hits. Cold/recovery values are
individual samples; warm/edit reports retain dispersion. The maintainer accepted
this evidence as sufficient and ended caching comparisons.

**Tested:** the cached campaign and nightly one-/two-thread screens each executed
18 compiler and 33 relations tests three times, zero failures against baseline zero.
Nightly screens (`--mode nightly --frontend 1` or `2`, `--jobs 16 --cache off
--screen --execute`) took 154.70 s and 157.08 s cold respectively. The maintainer
accepted one thread/16 jobs; the four-thread run was stopped and eight-thread/job-count
screens were not run. The nightly temporal-conversion warning was subsequently fixed;
its targeted qualification remains pending.

B04 profile/variant decisions and B06 final integration qualification remain open.
No simulator M22 qualification is claimed. This checkpoint precedes that final Plan 14
phase; committing it does not claim the remaining gates passed.

### A mistake made and corrected

The initial recovery experiment changed the target directory, defeating cache reuse
through changed compiler input paths. Recovery now removes only its owned artifacts
and rebuilds at the same path. A polling sleep also quantized short build timings;
memory sampling now runs independently of process completion.

### Deviations from the plan, deliberate

Mold comparisons are excluded at maintainer direction. Stable remains canonical while
the nightly route is experimental. System LLVM discovery was repaired before further
comparisons, and the obsolete M21 source seal was deleted at maintainer direction.
Linux performance is the priority; Windows CI work is deferred at maintainer direction.
Ordinary work stays in the existing `main` checkout, with separate trees reserved for
genuinely concurrent agent editing that requires isolation.
