---
title: Rust build reuse (sccache, cargo-shear, cargo-hakari)
status: review
date: 2026-10-24
tier: design
purpose: target
focus: library-targeted
doc_role: review
doc_topics: [design, environment, preparation]
standard: core-3.4
profile: not-applicable (build tooling; no physical or numerical semantics)
baseline: 68e44a37dcaeec1b0a0b1db8d9241ec7bfc9b305
evidence: Measured
decision: Revise
---

# Rust build reuse: sccache, cargo-shear and cargo-hakari

**Revise the build-reuse configuration.** Its architecture is sound: nightly workspace
feature unification, client-side sccache on one local disk cache, incremental workspace
crates and cacheable non-incremental dependencies. Six concrete defects waste work or state
untrue facts:

- Every new target directory recompiles about a third of a cold build despite a warm cache
  ([F01](#f01)).
- Agent and human builds keep two copies of the same cache entries ([F02](#f02)).
- The compiler cache is an undeclared, unverified build input ([F03](#f03)).
- Manifests declare 43 dependencies their source does not use, and a macro forces 13 of them
  ([F04](#f04)).
- Three detectors report one fact, two of them only falsely, and one mutates tracked files
  ([F05](#f05)).
- mtime freshness rebuilds unchanged crates after routine Git and formatter operations
  ([F06](#f06)).

The build guide also overstates cross-checkout reuse ([F07](#f07)).

The corrections are configuration, manifest and documentation changes. None alters a D1–D14
decision or adds a crate, so no ADR is needed:

- sccache 0.18.0 as a pinned, checksum-verified binary;
- one environment for agents and humans;
- cargo-shear as the single, pinned dependency detector in `just hygiene`, reached through
  root-cause manifest and macro fixes;
- Cargo content fingerprints.

cargo-hakari stays retired: the duplication it could remove costs 0.8% of a cold build. The
largest single loss (F01) is owned upstream. The review names the release that would let the
repository remove it and the measurement that would confirm it.

This report consolidates and supersedes, for planning purposes, the
[functional-extension review](../../functional_reviews/rust-build-reuse-tooling-2026-10-24.md)
of the same date. That review keeps its opportunity references (O01–O11), and this report
maps to them. Its probes remain the reproducible evidence:
[unit-graph census](../../functional_reviews/probes/rust-build-reuse-unit-census-2026-10-24.py)
and [sccache key shim](../../functional_reviews/probes/rust-build-reuse-sccache-key-shim-2026-10-24.py).

## 1. Contract, scope and evidence

**Tier and purpose.** A design-tier, target-purpose, library-targeted review of the Rust
build-reuse subsystem. Suppliers: Cargo on the pinned nightly, sccache, cargo-shear,
cargo-hakari, cargo-machete and cargo-udeps. Consumers: every recipe, direnv, rust-analyzer,
maturin, and agent sessions through `scripts/pse-env`.

Out of scope: remote, multi-level and distributed caching; CI portability; native C/C++
preparation beyond its compiler-cache keys. The process-simulator profile does not apply;
the subject carries no physical or numerical semantics. Standard: core 3.4 and the pse-arrow
binding.

**Maintainer decisions taken as premises:**

- upgrade sccache;
- adopt cargo-shear;
- assess cargo-hakari as a challenger to native workspace unification;
- leave the [library overview](../../libraries/rust_caching.md) unchanged (reviewed separately).

**Baseline.** First pass at `95ee5dfdd`; probes at `68e44a37d`. Concurrent agents'
uncommitted `scripts/capabilities/*` work was neither inspected nor touched. Machine: 32
cores, 16 Cargo jobs, toolchain `nightly-2026-09-29` (rustc 1.101.0-nightly). Tool versions:

| Tool | Version used |
|---|---|
| sccache | 0.17.0 installed; 0.18.0 official musl release, sha256 verified, run from a scratch prefix |
| cargo-shear | 1.14.0 |
| cargo-machete, cargo-udeps | as installed |

**Method.**

1. **Cargo `--unit-graph` resolution** (no compiling) for the daily selections. Selections:
   - `check --workspace --all-targets`, with and without force-validate;
   - `check -p pse-model`;
   - `check -p pse-runtime --features native-solvers`;
   - `test --workspace --no-run` (force-validate);
   - `build -p pse-py --lib`;
   - `test -p pse-tests-conformance` / `pse-tests-engine` (force-validate).
2. **Miss attribution against the real 83 GiB cache, read-only.** A probe-owned foreground
   server read the managed cache directory with `SCCACHE_LOCAL_RW_MODE=READ_ONLY`, so nothing
   was written. It logged per-crate hits and misses. A recording `RUSTC_WRAPPER` captured each
   call's hashed arguments and `CARGO_*` environment. Attribution ran in server mode, because
   `SCCACHE_LOG` silently disables client-side mode (`src/config.rs`); both modes compute
   keys with the same code. Target directories were passed as `--target-dir`, never as
   `CARGO_TARGET_DIR`, which would itself enter the key.
3. **A disposable detached worktree** for the shear prototype, the freshness probe and a cold
   uncached timed build. The worktree, its target directories, the probe target directories
   in the main checkout and the probe servers were removed afterwards. The managed server was
   never touched.
4. **Source reading:** sccache v0.17.0 and v0.18.0, cargo-shear v1.14.0, the upstream issues
   and PRs cited, Cargo's unstable-features reference, and the repository owners cited below.

**Evidence limits.** Timings are single runs on a machine other agents were using at the
same time. Hit/miss counts, unit counts and feature sets are exact. Native-feature crates
were resolved but not compiled, so removals in native-only code are covered by shear's
cfg-agnostic parse, not by a compile. Lifetime cache statistics cover an unknown window.

## 2. Current system

| Concern | Current mechanism | Owner |
|---|---|---|
| Feature unification | `-Z feature-unification` with `[resolver] feature-unification = "workspace"`, `fine-grain-locking`, resolver 3. The workspace-hack fallback was retired | `.cargo/config.toml`, ADR-0122, ADR-0175 |
| Compiler cache | `scripts/sccache` → `scripts/compiler_cache.py`, a supervised daemon with a dedicated socket. `scripts/build_environment.py` sets `SCCACHE_CLIENT_SIDE=1` and `SCCACHE_DIRECT=false` and strips inherited `CARGO_TARGET_DIR`. 100 GiB disk budget | `docs/dev/build-performance.md`, `.config/build.toml`, `.config/sccache.toml` |
| Profiles | Workspace crates `opt-level 2`, incremental. Dependencies (`package."*"`) `opt-level 3`, non-incremental, no debuginfo. `build-override` `opt-level 2`, shadowed for dependencies by `package."*"` | `Cargo.toml` |
| Freshness | Cargo mtime fingerprints | Cargo default |
| Dependency hygiene | `just deps-report` (shear, machete, deny, audit; advisory); `just udeps` separately | `scripts/audit_tools.py`, `scripts/validation_scope.py` |
| Tool declaration | `.config/diagnostic-tools.toml`: `[cargo-tools]` exact versions, and URL+sha256 binaries (trace-processor, mcpls, rr). sccache and cargo-shear are undeclared | bootstrap, doctor |
| Measurement | `just bench-builds`, `just build-cache-probe`, `just build-uncached` | build guide |

Lifetime statistics of the managed cache:

| Measure | Value |
|---|---|
| Rust hits / misses | 23,406 / 10,003 (70% hit rate) |
| C/C++ hit rate | 97.6% |
| Non-cacheable calls | 20,891 (crate type 7,160, incremental 6,303, multiple inputs 4,389) |
| Rust cache errors | 12 |
| Size | 83 GiB of 100 GiB |

**Strengths retained by the target.**

- **Workspace unification holds across selections.** `check -p pse-model` against
  `check --workspace`: 0 of 69 shared third-party packages differ in features. The opt-in
  features (force-validate, native-solvers) are the only intended exceptions.
- **The profile split is right:** incremental for edited workspace crates, cacheable
  dependencies.
- **Client-side mode keeps compiler children under the caller's supervision.**
- **Measurement and probe recipes already exist:** `bench-builds` simulates edits with real
  content changes and restorations.

## 3. Findings

| ID | Finding | Principles / gates | Evidence |
|---|---|---|---|
| [F01](#f01) | Fresh target directories miss ~⅓ of a cold build through build-script `OUT_DIR` keys | AP-07, DP-09, G6 | *Measured* |
| [F02](#f02) | Agent and human environments split the cache keyspace (`CARGO_TERM_COLOR`) | DP-01, DP-09, G6 | *Measured* |
| [F03](#f03) | The compiler cache is an undeclared, unverified build input | DP-01, DP-15, G1 | *Interface-checked* |
| [F04](#f04) | Manifests declare dependencies the source does not use; a macro leaks one into 13 consumers | AP-03, DP-01, DP-14, G1 | *Tested* (prototype) |
| [F05](#f05) | Three dependency detectors for one fact; two add only false positives; one mutates tracked files | DP-01, DP-16, G4 | *Measured* |
| [F06](#f06) | mtime freshness rebuilds unchanged crates after routine file operations | AP-07, DP-09 | *Measured* |
| [F07](#f07) | The build guide overstates cross-checkout cache reuse | G7 | *Interface-checked* |

<a id="f01"></a>
### F01: fresh target directories miss about a third of a cold build

**Evidence.** Read-only against the real cache, `check --workspace --all-targets`:

| Build | Rust hits | Rust misses | Hit rate |
|---|---|---|---|
| Fresh `--target-dir` inside the main checkout | 489 | 189 | 72% |
| Fresh detached worktree of the same commit | 483 | 195 | 71% |

**Which crates miss.** The misses include the complete `arrow*`, `datafusion*`, `parquet`,
`surrealdb*`, `symbolica`, `faer`, `tokio`, `serde*`, `prost` and `pyo3*` stacks. A
different checkout adds only six misses: `py_literal`, plus crates downstream of the vendored
`paste` proc-macro.

**Mechanism** (sccache v0.18.0 `src/compiler/rust.rs`, hash-key step 8). The values of
dep-info env-deps are hashed verbatim. 17 third-party crates record
`env-dep:OUT_DIR=<absolute build path>` because they `include!` build-script output:

> `serde_core`, `serde`, `thiserror`, `ref_cast`, `typetag`, `snap`, `bigdecimal`,
> `rust_decimal`, `target_lexicon`, `pulp`, `private_gemm_x86`, `nano_gemm_{c32,c64,f32,f64}`,
> `gmp_mpfr_sys`, `tracing_perfetto_sdk_schema`

Their rmeta embeds the included file's absolute path, so every dependent's `--extern` content
hash changes (step 5). `basedirs` is applied only in `src/compiler/c.rs` and the
preprocessor cache. In the cold uncached test build below, those crates account for 2,075 of
5,822 CPU-seconds.

**Consequence.** Every new target directory recompiles the heaviest third of the dependency
graph. That covers new worktrees (`just worktree`, agent worktrees under `~/pse-arrow-wt/`),
full-tree copies (`/tmp/pse-52d-actual-feature-campaign-*`), `bench-builds` snapshots and
in-checkout `--target-dir` experiments. The main `target/` keeps stable keys. The measured 72%
fresh-directory hit rate is almost exactly the 70% lifetime rate, which suggests (but does not
prove) that fresh directories dominate historical misses.

**Correction.** This is owned upstream. Issue #2870 (open since 2026-09-22) states this cause;
PRs #2678 and #2794 normalize cwd, `CARGO_MANIFEST_DIR` and dep-info env values under
`basedirs`. #2794 requires an effective `--remap-path-prefix` covering the working directory,
which is what makes rmeta bytes, and so the cascade, location-independent.

When a release contains the fix:

- set `basedirs` to the checkout and worktree roots;
- add `--remap-path-prefix=<checkout root>=.` to the repository rustflags.

Local alternatives were rejected (§6). Before the release, a repeated full-tree campaign can
reuse a stable per-purpose path so that its later runs hit their own entries.

**Verification.** Repeat the read-only probe: a fresh worktree's
`check --workspace --all-targets` should reach a hit rate near the main checkout's warm
rate, and the miss list should no longer contain the 17 roots or their dependents.

<a id="f02"></a>
### F02: agent and human builds keep separate cache entries

**Evidence.** `.claude/settings.json` (`env`) and `.codex/config.toml`
(`shell_environment_policy.set`) set `CARGO_TERM_COLOR=never`; direnv and the interactive
terminal do not. Cargo passes its environment to rustc. sccache excludes only
`CARGO_MAKEFLAGS`, `CARGO_REGISTRIES_*`, `CARGO_BUILD_JOBS` and `CARGO_ENCODED_RUSTFLAGS`. For
the same crates, the recording wrapper showed identical hashed arguments and different
environment keys, with `CARGO_TERM_COLOR` as the only differing variable. Both keyspaces
exist in the cache: both variants hit equally on a fresh `pse-model` check.

**Consequence.** Every compile that both environments perform is stored twice and missed
twice after each toolchain or lockfile change. That doubles first-build cost for whichever
environment builds second and consumes budget in the 100 GiB cache.

**Correction.** Remove `CARGO_TERM_COLOR` from both agent configurations. Agent shells are
not terminals, so Cargo's default `auto` already disables color. This keeps the single
environment boundary (`scripts/pse_env.py`) free of a special case. Optionally, upstream a
one-line addition of `CARGO_TERM_COLOR` to sccache's exclusions; the variable does not affect
rustc output.

**Verification.** Run the recording wrapper once from an agent session and once from a
terminal: the environment keys should match.

<a id="f03"></a>
### F03: the compiler cache is an undeclared build input

**Evidence.**

- sccache is absent from bootstrap's `CARGO_TOOLS` and from `.config/diagnostic-tools.toml`.
  The installed 0.17.0 came from a host-local `cargo install`.
- `PSE_SCCACHE_BINARY` and the managed wrapper already insist on an exact executable, but
  nothing declares which one.
- cargo-shear is likewise undeclared. Its findings change with its release once it gates
  hygiene (F05).

**Consequence.** The tool that serves every compile changes only when someone happens to
reinstall it. Its version and provenance are not reproducible or reviewable, unlike every
other pinned build input (ADR-0165 discipline).

**Correction.** Declare sccache 0.18.0 as a URL+sha256 binary entry, the shape already used
for `trace-processor` and `rr`:

- asset `sccache-v0.18.0-x86_64-unknown-linux-musl.tar.gz`;
- sha256 `45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89`.

Bootstrap installs it, and doctor reports it. Pin cargo-shear in `[cargo-tools]`.

**Upgrade facts, *Measured*:**

- **Rust entries survive:** Rust `CACHE_VERSION` stays `6` and the key function is
  unchanged. 0.18.0 reading the 0.17 cache gave the identical 52 hits / 11 misses.
- **C/C++ entries reset once:** `CACHE_VERSION` goes 12 → 13 because of the assembler key
  (#2843), so native C/C++ preparations rebuild once.
- **The repository probe passes:** `just build-cache-probe` with
  `PSE_SCCACHE_BINARY=<0.18.0>` gives one miss then one hit, with incremental pass-through.
- **Other gains in 0.18.0:** file-descriptor allow-listing on daemonization (#2841), no
  jobserver for children that can't use it (#2827), gcc `TooHard` handling (#2833) and a
  preprocessor-check fix (#2824).

<a id="f04"></a>
### F04: manifests disagree with source

**Evidence.** cargo-shear 1.14.0 at baseline: 43 errors and 18 warnings. Prototype in the
worktree:

1. `cargo shear --fix`;
2. `#[doc(hidden)] pub use miette as __miette;` in `pse-diagnostics`, with the
   `impl_diagnostic!` macro's 17 `miette::` paths rewritten to `$crate::__miette::`;
3. `pse-buildinfo/identity.rs` moved into `src/`;
4. `tempfile` restored in `benches`.

Results:

- Shear errors 43 → 0.
- `cargo check --workspace --all-targets` exits 0, and so does
  `cargo check -p pse-tests-conformance --all-targets --features force-validate`.
- Unit counts and all third-party feature sets are unchanged for the workspace check and
  test, `pse-py`, the native `pse-runtime` check and `pse-tests-engine`.
- `pse-tests-conformance` drops from 847 to 839 units (the `proptest` family). `Cargo.lock`
  loses only edges.

**Structural causes.**

1. **A macro leaks an implementation dependency into consumers.** `impl_diagnostic!`
   expands to bare `miette::` paths, so 13 crates must declare `miette` without naming it.
   That is a consumer obligation the owning crate should hold (AP-03, DP-14).
2. **Stale declarations accumulate** because the existing detector is advisory and noisy
   (F05). There are 25 truly unused declarations, e.g. `pse-runtime` `object_store` and a
   duplicate dev `url`, `pse-py` `pse-kernels`/`url`, and six in `tests/conformance`.
3. **Redundant feature-forwarding edges.** Removing six forwarders from
   `tests/conformance` left force-validation unchanged on every crate in its closure.
   `pse-runtime/force-validate` already propagates through existing dependencies, as
   ADR-0170 states, and `pse-testkit` left the closure (839 → 838). It compiled with
   `--features force-validate`.

**Tool limitation to accommodate.** The workspace has 21 parent-relative `#[path = "../…"]`
modules, a deliberate convention for shared fixtures (`tests/support`, `tests/fixtures`,
`scripts/workspace.rs`). cargo-shear joins these onto an absolute parent and canonicalizes
only paths beginning with `.` (`src/context.rs`). They are therefore reported unlinked, and
their dependencies unused. Upstream: issues #587 and #589, both open.

The prototype confirmed four wrong removals in `pse-buildinfo` and one in `benches`. It also
confirmed that `tests/engine/src/oracle.rs` (included from
`tests/engine/tests/pushdown_vs_unpruned.rs`) and `xtask/src/canonical_recovery.rs` are
linked, although shear reports them unlinked. The first-pass suggestion to delete
`oracle.rs` was wrong.

`crates/pse-modeling/src/kernel_envelopes.rs` is a genuinely empty orphan. The optional
`arrow-cast` in `pse-columnar` exists only to apply `force_validate`; it is a legitimate
activation dependency.

**Correction.**

- the `$crate` re-export;
- remove the 25 unused declarations and delete the empty file;
- move single-use path modules into the conventional layout;
- package-level `ignored` / `ignored-paths` entries for shared-fixture modules, each citing
  #589 (shear's redundant-ignore check flags them once fixed), and for `arrow-cast`, citing
  ADR-0170;
- judge the remaining feature-only forwarders with the same unit-graph comparison;
- set `doctest = false` / `test = false` where `--check-test-targets` reports empty targets
  (22 and 5).

An empty doctest pass costs 1.3–2.1 s per crate, so that last item saves seconds per
`just doctest`.

**Verification.**

- `cargo shear --check-test-targets` exits 0.
- `just check` passes, and so does `just check-package pse-tests-conformance --features
  force-validate`.
- The census probe shows unchanged feature sets for the daily selections.
- Explicit force-validation remains on the Arrow leaves.

<a id="f05"></a>
### F05: three detectors for one fact

**Evidence.**

- **cargo-machete:** findings are a subset of shear's, plus build-script false positives
  (`cc`, `pkg-config` and `sha2` in the `-sys` crates, `toml` in `pse-backend-native`'s
  `build.rs`, `pyo3-build-config`). Its `--with-metadata` mode, which `just deps-report`
  uses, rewrote six tracked `vendor/*/Cargo.lock` files and created three more during this
  review; they were restored by hand (G4).
- **cargo-udeps:** on the shear-clean worktree it ran 416 s (a full compile). Its unique
  findings were uses behind feature gates it does not compile:
  `#[cfg(all(test, feature = "native-acceptance"))]` in conformance and
  `#[cfg(all(test, feature = "canonical-tests"))]` in `pse-operations`. Otherwise it repeated
  shear's feature-forwarding warnings.
- **Shear** analyzes every `cfg` branch statically and runs in 0.4 s.

**Consequence.** Duplicate authority for one fact (DP-01). Findings get ignored because two
of the three sources are mostly false. One detector has a hidden write side effect on vendored
sources.

**Correction.** Retire `audit-machete` and `just udeps` and their bootstrap entries. Keep shear
as the only detector, pinned (F03) and run in `just hygiene`. It fits the scope-end bundle and
is the adopted detector, not a new lint. `scripts/audit_tools.py` already classifies its exit
codes. Keep it out of `turn-end` and the edit hook. `cargo shear --expand` is not needed
routinely, because the workspace compile served as the compiler-backed check of every
removal.

<a id="f06"></a>
### F06: mtime freshness rebuilds unchanged work

**Evidence** (worktree, `check --workspace --all-targets`, 16 jobs):

| Scenario | mtime (current) | content fingerprints |
|---|---|---|
| No-op build | 0.62 s, 0 rebuilt | 0.64–0.87 s, 0 rebuilt |
| `touch crates/pse-ids/src/lib.rs` (content unchanged) | 16.0 s, 29 crates rechecked | 0.65 s, 0 rebuilt |
| `git stash` + `git stash pop` (21 files rewritten) | not run | 1.7 s, 0 rebuilt |
| One-time switch | n/a | 420 s, all 518 crates rechecked once |

**Consequence.** Agents stash, switch branches, rebase and run formatters that rewrite
identical content. Each operation re-checks or rebuilds the touched crates and their
downstream: in test builds that means relinking test binaries, and for incremental crates,
re-running rustc.

**Correction.** Add `[unstable] checksum-freshness = true` and `[build] fingerprint =
"content"` to `.cargo/config.toml`. Add both keys to `CONFIG_KEYS` in
`scripts/installation_context.py`, which otherwise refuses unknown Cargo configuration
("Unsupported Cargo configuration").

Limits, from Cargo's unstable reference:

- build-script `rerun-if-changed` inputs keep mtime semantics;
- the checksum algorithm may change between Cargo versions, which a toolchain bump already
  rebuilds anyway.

`bench-builds` edits and restorations are real content changes, so its edit-latency phases
measure the same thing as before. The scheme stays per checkout; it does not make a shared
build directory sound (ADR-0122).

<a id="f07"></a>
### F07: the build guide overstates cross-checkout reuse

`docs/dev/build-performance.md` says "Across checkouts, the shared sccache serves identical
compilations." Per F01 that holds only for dependencies without build-script path inputs and
outside the vendored paths. Narrow the sentence; when F01's upstream fix is adopted, restate
it. The library overview's §3.3 statement about basedirs and worktrees describes C/C++
behavior. The maintainer is having that document reviewed separately, and this review leaves
it unchanged.

## 4. Recommendations

| ID | Change | Findings | Functional-review reference |
|---|---|---|---|
| <a id="r01"></a>R01 | Pin sccache 0.18.0 as a verified binary; pin cargo-shear | F03 | O01, O08 |
| <a id="r02"></a>R02 | Remove `CARGO_TERM_COLOR` from the Claude and Codex configurations | F02 | O11 |
| <a id="r03"></a>R03 | Shear to zero: `$crate` re-export, removals, layout moves, justified ignores (#589, ADR-0170), forwarder pruning, test-target keys | F04 | O05, O06 |
| <a id="r04"></a>R04 | Shear as the sole detector in `just hygiene`; retire machete and udeps | F05 | O07, O08 |
| <a id="r05"></a>R05 | Content fingerprints plus the `CONFIG_KEYS` allowlist | F06 | O10 |
| <a id="r06"></a>R06 | Narrow the build-guide reuse sentence | F07 | O02 |
| <a id="r07"></a>R07 | Adopt sccache Rust basedirs with `--remap-path-prefix` once released (#2678/#2794) | F01 | O02 |
| <a id="r08"></a>R08 | Keep hakari retired and the incremental split as is | strengths | O03, O09 |

**Prerequisites and order.** R01–R06 are independent, and together they form one small
tooling plan. R04 should land after R03, so that hygiene starts at zero. R07 waits on an
upstream release. Optional upstream contributions would remove local workarounds: the
cargo-shear `..` normalization (#589) and the sccache `CARGO_TERM_COLOR` exclusion.

## 5. Library fit and ownership cost

| Capability | Current mechanism | Target mechanism | Fit and limits | Recommendation |
|---|---|---|---|---|
| Cross-selection feature unification | Cargo `-Z feature-unification` (workspace) | Unchanged | Holds across all daily selections; opt-in features are separate by design | Keep (ADR-0122/0175) |
| Host/target unification | None | hakari `unify-target-host` (rejected) | 32 duplicates ≈ 45 CPU-seconds of a 5,822 CPU-second cold build (`syn` 16.6 s, `regex-automata` 9.8 s). A workspace-hack widens every small closure (ADR-0175: 423 → 382 units on removal) | Do not adopt; revisit if Cargo gains native host/target unification |
| Compiler cache | sccache 0.17.0, client-side, local disk | sccache 0.18.0, pinned and verified | Rust entries survive; C/C++ resets once. `SCCACHE_LOG` disables client-side mode, so diagnostics need a probe server | Adopt |
| Cross-directory Rust reuse | None effective | sccache basedirs for Rust plus rustc `--remap-path-prefix` (unreleased) | Only remapping makes rmeta location-independent. The remap changes panic and debug paths | Adopt when released |
| Freshness | mtime | Cargo content fingerprints | Build-script inputs keep mtime; algorithm may change between Cargo versions | Adopt |
| Dependency hygiene | shear + machete + udeps | shear only | Static and cfg-agnostic; #589 false positives on parent-relative `#[path]` | Adopt shear alone |

## 6. Alternatives and tradeoffs

| Alternative | Why considered | Assessment | Selection / revisit |
|---|---|---|---|
| Re-adopt hakari | Removes the host/target splits | 0.8% saving against closure widening, a crate addition and an ADR | Rejected; revisit if a cold-build critical path shows host duplicates |
| Run an unmerged sccache PR build now | Removes F01 immediately | Changes a build input that serves every compile, based on unreviewed code | Rejected; adopt the release |
| Share one build directory across checkouts | Shares `OUT_DIR` paths | Unsound: Cargo keys path packages by workspace-relative path (ADR-0122). Content fingerprints remove the mtime half only | Rejected |
| Shared build dir for in-checkout `--target-dir` experiments | Avoids F01 inside one checkout | Interacts with measurement isolation (`bench-builds --cold-cache`) and with whether rustflags enter unit hashes; not examined | Not pursued; open question Q6 |
| Disable incremental for workspace crates | Makes them cacheable | Edit-loop latency gets worse; across checkouts they would still miss (F01) | Rejected; revisit only after R07 |
| Normalize `CARGO_TERM_COLOR` in `pse_env.py` | Fixes F02 centrally | Adds a special case to the one environment boundary when deleting two configuration lines suffices | Rejected in favour of R02 |
| Keep udeps for compiler-backed coverage | Sees expansion | Single feature configuration; 416 s; its unique findings were false. The workspace compile already validated removals | Retire |
| Remote, multi-level or distributed sccache | Overview capabilities | No consumer on one machine; local reads take 3 ms | Out of scope |

## 7. Architectural assessment

| Foundation | Verdict | Evidence / required action |
|---|---|---|
| AP-01 Separation of concerns | Satisfied | Tool roles are distinct (Cargo resolution, sccache reuse, shear hygiene) |
| AP-02 Stable contracts | Satisfied after R01 | The managed wrapper contract is stable; its executable is undeclared (F03) |
| AP-03 Composition | Not satisfied | `impl_diagnostic!` makes consumers declare its implementation dependency (F04, R03) |
| AP-04 Domain model / authority | Not satisfied | Three detectors for one fact (F05) and duplicate keyspaces (F02); single owners after R02/R04 |
| AP-05 Explicit structure | Satisfied after R03 | Activation-only and fixture-path exceptions become explicit, cited ignores |
| AP-06 Local reasoning / testability | Satisfied | Read-only probes and unit-graph checks settle each claim locally |
| AP-07 Execution fits workload | Not satisfied | Fresh-directory recompilation (F01, upstream) and spurious mtime rebuilds (F06) |

| Gate | Result | Evidence / action |
|---|---|---|
| G1 Authority | Fail → R01, R03 | Undeclared cache binary; manifests stating false dependencies |
| G2 Semantic fidelity | Pass | Feature sets and force-validation are unchanged by the prototype |
| G3 Validity | Pass | No refused or silently dropped validation; `arrow-cast` activation retained |
| G4 Hidden behavior | Fail → R04 | machete rewrites vendored lockfiles; `SCCACHE_LOG` silently changes the execution mode (document it in the probe guidance) |
| G5 Consistency and recovery | Pass | Cache misses degrade to compilation; read-only probing proven non-mutating |
| G6 Transformation and reuse | Fail → R02, R07 | Reuse keyed on accidental inputs (`CARGO_TERM_COLOR`, absolute `OUT_DIR`) |
| G7 Truthful capability claims | Fail → R06 | Build guide's cross-checkout claim |
| G8 Library leverage | Pass with R05/R07 | Native Cargo and upstream sccache mechanisms are preferred over local machinery |
| G9 Architectural fitness | Not met (follows AP-03, AP-04, AP-07) | Met by R01–R06 except F01, which R07 resolves upstream |

## 8. Verification

| Claim | Label | Method | Conditions | Result |
|---|---|---|---|---|
| Workspace unification holds across selections | *Interface-checked* | `--unit-graph` census | Pinned nightly; seven selections | 0 differing feature sets except opt-in features |
| Fresh target directory misses 189/678 | *Measured* | Read-only probe server + debug log | `check --workspace --all-targets`, real cache, server mode | 489 hits / 189 misses |
| Worktree adds 6 misses | *Measured* | Same, detached worktree | Same | 483 / 195 |
| `OUT_DIR` roots | *Interface-checked* | dep-info `env-dep` lines in the probe target | 17 third-party crates | Listed in F01 |
| Cascade cost | *Measured* | Cold uncached `test --workspace --no-run --timings` (force-validate) | `RUSTC_WRAPPER=` empty, 16 jobs | 2,075 / 5,822 CPU-s; 541 s wall |
| `CARGO_TERM_COLOR` splits keys | *Measured* | Recording wrapper, two environments | `check -p pse-model`, fresh targets | Same arguments, different environment key |
| 0.18.0 reads the 0.17 cache | *Measured* | Read-only probe with the 0.18.0 binary | Fresh `check -p pse-model --all-targets` | 52 / 11, identical to 0.17 |
| 0.18.0 passes the repository probe | *Tested* | `just build-cache-probe`, `PSE_SCCACHE_BINARY=0.18.0` | Disposable cache | One miss then one hit; incremental pass-through |
| Shear prototype is safe | *Tested* | `cargo check --workspace --all-targets`; conformance with force-validate | Worktree, offline lock update | Exit 0; feature sets unchanged |
| Forwarders are redundant (conformance) | *Interface-checked* | Unit-graph features before and after | `--features force-validate` | No validation change; −1 unit |
| udeps adds only false positives | *Measured* | `cargo udeps --workspace --all-targets` | Shear-clean worktree | 416 s; feature-gated false positives |
| Content fingerprints remove spurious rebuilds | *Measured* | Touch and stash/pop scenarios | Worktree, 16 jobs | 29 crates / 16 s → 0 / 0.65 s |
| Hakari's target is small | *Measured* | Cold timing of the duplicated units | Same cold build | ≈ 45 CPU-s (0.8%) |
| Empty doctest pass cost | *Measured* | `scripts.arrow_validation test --doc -p …`, warm | Four large crates | 1.3–2.1 s each; 3.0 s together |

## 9. Further investigation and open questions

These do not block R01–R06 unless noted.

1. **When #2678 or #2794 ships (R07).** Rerun the F01 probe with `basedirs` set and
   `--remap-path-prefix=<root>=.`. Before adopting, check these effects of the remap:
   - panic messages and backtraces showing relative paths;
   - insta snapshots or tests that assert `file!()`/`Location` strings;
   - rust-analyzer and debugger source mapping;
   - the one full rebuild the rustflags change causes.
2. **Remaining feature-only forwarders.** `benches` (`pse-rules`), `pse-runtime`
   (`pse-rules`, `pse-testkit`), `tests/engine` (`pse-runtime`, `pse-testkit`),
   `tests/governance` (`pse-codegen`) and `tests/lifecycle` (`pse-testkit`). Apply the
   unit-graph comparison per edge. `pse-runtime`'s may be deliberate test-support activation.
3. **Native-feature compile of the R03 removals.** Run
   `just check-native-package pse-runtime native-solvers` (and the backend crate) once R03
   lands. The prototype resolved but did not compile native features.
4. **Cache composition and budget.** Each new target-directory path adds about 190 entries
   per mode and profile until R07. Check whether the 100 GiB budget evicts warm main-checkout
   entries under heavy worktree or campaign use (`just build-storage`, cache-size trend).
5. **Who creates the `/tmp/pse-52d-actual-feature-campaign-*` copies.** No script under
   `scripts/` names them. Identify the owner before deciding whether stable per-purpose paths
   are worth it before R07.
6. **Shared build directory for in-checkout experiments.** Whether `build.build-dir` could
   let in-checkout `--target-dir` experiments share intermediates depends on whether rustflags
   enter unit metadata hashes on this nightly, and on `bench-builds` isolation. Not examined.
7. **Other accidental `CARGO_*` keys.** The census covered agent and terminal environments
   only. Tools that set their own variables (e.g. `cargo llvm-cov`'s `CARGO_LLVM_COV*`) form
   separate keyspaces by design. Confirm nothing else in `scripts/pse_env.py` or the recipes
   varies a `CARGO_*` value between equivalent builds.
8. **The 12 lifetime Rust cache errors.** These can't be attributed without a server log. If
   they recur during an investigation, use a probe server with
   `SCCACHE_LOG=sccache::compiler::compiler=debug`; remember that this is server mode.
9. **Checksum freshness with rust-analyzer and maturin.** Both read the same
   `.cargo/config.toml`; confirm a normal rust-analyzer session and `just py-sync` after
   R05 (expect one full recheck each).
10. **Upstream contributions (optional):**
    - cargo-shear lexical `..` normalization in `src/context.rs` (#589);
    - sccache `CARGO_TERM_COLOR` exclusion.

## 10. Rule impacts and disposition

- **No ADR or decision record is required.** These are tooling, dependency and documentation
  changes (`.claude/rules/decisions.md`).
- **Owners touched:**
  - `.config/diagnostic-tools.toml` and bootstrap (R01);
  - `.claude/settings.json` and `.codex/config.toml` (R02, under the tooling rule; the
    environment guide stays accurate);
  - manifests and `pse-diagnostics` (R03);
  - `scripts/validation_scope.py`, `scripts/audit_tools.py`, the `justfile` and the hygiene
    bundle (R04);
  - `.cargo/config.toml` and `scripts/installation_context.py` (R05);
  - `docs/dev/build-performance.md` (R06).
- **The ADR-0175 retirement of the workspace-hack stands.**
- **Disposition owner.** The plan created from this review owns the findings' disposition.
  Until it exists, F01–F07 are open. F01's correction is externally owned; its trigger is
  an sccache release containing #2678 or #2794.

## 11. Decision

- **Behavioral adequacy holds.** Builds are correct. Feature and validation semantics are
  unaffected by every proposed change: tested for the prototype, reasoned for R01, R02, R05
  and R06.
- **Architectural fitness is not met.** AP-03, AP-04 and AP-07 fail through F01–F06.
- **Decision: Revise.** R01–R06 in one tooling plan; R07 on upstream release; R08 retained
  as is.
- **Strongest evidence:** the read-only miss attribution, the shear prototype compile with
  unchanged unit graphs, and the cold timing.
- **Main uncertainties:** the exact share of historical misses due to fresh directories (the
  72% vs 70% match is suggestive, not proof), and native-feature compilation of the removals
  (Q3).

| Priority | Change | Findings | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| 1 | R03 → R04 | F04, F05 | `cargo shear --check-test-targets` exits 0 in hygiene; `just check` passes; unchanged census | Future tooling plan |
| 2 | R05 | F06 | The touch scenario rebuilds 0 crates; installation context accepts the keys | Future tooling plan |
| 3 | R01, R02 | F03, F02 | Doctor reports 0.18.0; `just build-cache-probe` passes; matching environment keys | Future tooling plan |
| 4 | R06 | F07 | Guide sentence narrowed | Future tooling plan |
| 5 | R07 | F01 | Fresh-worktree probe hit rate ≈ warm rate | Upstream release trigger |
