# Rust build reuse: sccache, cargo-shear and cargo-hakari

Functional-extension review · 2026-10-24 · developer-workflow scope, single local machine.
Revised the same day after maintainer direction and follow-up investigations (see
[revision](#revision)).

## Recommendation and scope

The question: what is the best-in-class configuration of the three build-reuse tools in
[the capability overview](../libraries/rust_caching.md) for this workspace? Remote and
distributed caching and CI portability are out of scope. The maintainer has already decided
two things: upgrade sccache, and adopt cargo-shear.

- **sccache:** upgrade to 0.18.0 as a declared, checksum-verified tool ([O01](#O01)). The
  Rust cache survives the upgrade. The investigation found that the largest source of misses
  is not configuration: every **new target directory** recompiles about a third of a cold
  build despite a warm cache. Build-script `OUT_DIR` paths enter the key of 17 widely used
  crates, and their rmeta carries the path on to every dependent ([O02](#O02)). Upstream has
  a fix in review; adopt it when it is released. Two cheap local fixes stand on their own:
  - stop agent sessions from splitting the cache keyspace with `CARGO_TERM_COLOR`
    ([O11](#O11));
  - keep the incremental split ([O03](#O03)).
- **cargo-shear:** the prototype works.
  - Its 43 errors reach zero with source fixes plus one documented upstream false positive,
    the whole workspace still compiles, and no build selection's features change
    ([O05](#O05)).
  - Make it the only dependency detector: machete and udeps added only false positives
    ([O07](#O07)).
  - Pin it and run it in `just hygiene` ([O08](#O08)). Its test-target analysis is a small,
    free tidy-up ([O06](#O06)).
- **cargo-hakari:** do not bring it back ([O09](#O09)). Measured: the host/target duplicates
  it could remove cost about 45 CPU-seconds (0.8%) of a 5,822 CPU-second cold build.
- **Adjacent, recommended:** Cargo's content fingerprints (`-Z checksum-freshness`) remove
  spurious rebuilds after `touch`, `git stash` or a branch round-trip, at no measurable no-op
  cost ([O10](#O10)).

The opportunities are proposals for `create-plan`. Apart from the measurement worktree
described below, the review changed no production configuration. None needs an ADR.

## Baseline, method and evidence limits

Source baselines: `main` at `95ee5dfdd` for the first pass, and `68e44a37d` for the follow-up
probes. Concurrent agents' uncommitted `scripts/capabilities/*` work was not touched.
Toolchain: `nightly-2026-09-29` (rustc 1.101.0-nightly), 32 cores, 16 Cargo jobs. Tools:
sccache 0.17.0 (installed) and 0.18.0 (the official musl release, sha256 verified, run from a
scratch prefix), cargo-shear 1.14.0, cargo-machete, cargo-udeps.

Method:

- **Cargo unit graphs**, resolved without compiling, analyzed with
  [the census probe](probes/rust-build-reuse-unit-census-2026-10-24.py).
- **Miss attribution against the real cache, read-only.** A probe-owned foreground server
  reads the managed cache directory (83 GiB) with `SCCACHE_LOCAL_RW_MODE=READ_ONLY`, so
  nothing is written. Per-crate hits and misses come from that server's debug log, and
  [a recording wrapper](probes/rust-build-reuse-sccache-key-shim-2026-10-24.py) captures each
  call's hashed arguments and `CARGO_*` environment. Attribution runs use server mode, because
  setting `SCCACHE_LOG` silently disables client-side mode; both modes compute keys with the
  same code.
- **A disposable detached worktree** of the baseline holds the cargo-shear fixes, the
  freshness probe and a cold build. It was removed afterwards, along with its target
  directories and the probe target directories in the main checkout.
- **Sources read:** sccache v0.17.0 and v0.18.0 source (`src/compiler/rust.rs`,
  `src/config.rs`, `src/compiler/c.rs`) and release notes; cargo-shear v1.14.0 source
  (`src/context.rs`, `src/source_parser.rs`); upstream issues and PRs cited below; Cargo's
  unstable-features reference.

Every timing is a single run on a machine that other agents were using at the same time.
Treat timings as indicative; the hit/miss and unit counts are exact. Native-feature crates
were resolved but not compiled.

## Current capabilities

| Tool | What exists | Owner |
|---|---|---|
| Feature unification | Nightly `-Z feature-unification`, `[resolver] feature-unification = "workspace"`, `fine-grain-locking`, resolver 3 | `.cargo/config.toml`, ADR-0122, ADR-0175 |
| sccache | Managed wrapper `scripts/sccache` → `scripts/compiler_cache.py`. `scripts/build_environment.py` sets client-side mode, `SCCACHE_DIRECT=false`, a 100 GiB cache and a dedicated socket, and strips inherited `CARGO_TARGET_DIR` | `docs/dev/build-performance.md`, `.config/build.toml`, `.config/sccache.toml` |
| Measurement | `just bench-builds`, `just build-cache-probe`, `just build-uncached` | build guide |
| Profiles | Workspace crates: `opt-level 2`, incremental. Dependencies (`package."*"`): `opt-level 3`, non-incremental, no debuginfo | `Cargo.toml` |
| Dependency hygiene | `just deps-report` runs shear, machete, deny and audit, advisory only; `just udeps` is separate | `scripts/audit_tools.py`, `scripts/validation_scope.py` |
| Tool pins | `.config/diagnostic-tools.toml` pins protocol-sensitive cargo tools in `[cargo-tools]` and URL+sha256 binaries (trace-processor, mcpls, rr). sccache and cargo-shear have no declared version | bootstrap, doctor |

Lifetime statistics of the managed cache: Rust hit rate 70% (23,406 hits, 10,003 misses),
C/C++ 97.6%, 83 of 100 GiB used, 12 Rust cache errors.

<a id="O01"></a>
## O01: sccache 0.18.0 as a declared, verified binary

**Selected by the maintainer. Compatibility *Measured*.**

- **The Rust cache survives.** Rust `CACHE_VERSION` is `6` in both releases and the hash-key
  function is unchanged. 0.18.0 reading the 0.17-written cache gave exactly the same result
  as 0.17: 52 hits and 11 misses on a fresh-target `check -p pse-model --all-targets`.
- **The C/C++ cache resets once.** C/C++ `CACHE_VERSION` goes from 12 to 13, because the
  assembler is now part of the key (#2843). Native C/C++ preparations recompile once after the
  upgrade.
- **The repository probe passes.** `just build-cache-probe` with
  `PSE_SCCACHE_BINARY=<0.18.0>` gives one Rust miss then one hit, with incremental
  pass-through intact.
- **What 0.18.0 brings here:** the assembler key (a correctness fix); file-descriptor
  allow-listing on daemonization and no jobserver for children that can't use it (#2841,
  #2827), relevant to this repository's inherited-descriptor history; gcc `TooHard`
  handling (#2833) and a preprocessor-check fix (#2824).

**Install route.** Declare sccache in `.config/diagnostic-tools.toml` as a URL+sha256 binary
entry, the same shape as `trace-processor` and `rr`. The asset is
`sccache-v0.18.0-x86_64-unknown-linux-musl.tar.gz`, sha256
`45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89`. Bootstrap installs it and
doctor reports it. A prebuilt verified binary avoids compiling sccache with whatever
toolchain happens to be the default. `PSE_SCCACHE_BINARY` and the managed wrapper already
select an exact executable, so nothing else changes. Keep `client_side_mode = true`,
`SCCACHE_DIRECT=false`, the local disk backend and the 100 GiB budget. A read takes 3 ms, so
multi-level, remote, distributed and compression tuning have nothing to gain on one machine.

<a id="O02"></a>
## O02: fresh target directories are the dominant miss source

***Measured*, read-only against the real cache.**

| Build (`check --workspace --all-targets`) | Rust hits | Rust misses | Hit rate |
|---|---|---|---|
| Fresh `--target-dir` inside the main checkout | 489 | 189 | 72% |
| Fresh detached worktree of the same commit | 483 | 195 | 71% |

A fresh target directory in the *same* checkout misses 189 crates, which is why its hit rate
almost exactly matches the cache's 70% lifetime rate. The misses include the whole `arrow*`,
`datafusion*`, `parquet`, `surrealdb*`, `symbolica`, `faer`, `tokio`, `serde*`, `prost` and
`pyo3*` stacks. A different checkout adds only 6 more misses: `py_literal`, plus crates
downstream of the vendored `paste` proc-macro.

**Cause.** Source-checked in sccache v0.18.0 `src/compiler/rust.rs`: the Rust key hashes the
values of the env-deps recorded in dep-info verbatim. 17 third-party crates record
`env-dep:OUT_DIR=<absolute build path>` because they `include!` build-script output:
`serde_core`, `serde`, `thiserror`, `ref_cast`, `typetag`, `snap`, `bigdecimal`,
`rust_decimal`, `target_lexicon`, `pulp`, `private_gemm_x86`, the four `nano_gemm_*`,
`gmp_mpfr_sys` and `tracing_perfetto_sdk_schema`. Their rmeta embeds that path in turn, so
every dependent's `--extern` content hash changes. `basedirs` applies only to C/C++ in 0.18.0
(`src/compiler/c.rs`, the preprocessor cache). The overview's §3.3 sentence about worktrees
describes C/C++ behavior; the maintainer is having that document reviewed separately, and
this review leaves it untouched.

**Cost.** In a cold uncached `test --workspace --no-run` build (force-validate), those crates
take 2,075 of 5,822 CPU-seconds. So every new target directory pays about a third of a cold
build. That covers new worktrees (`just worktree`, agent worktrees under `~/pse-arrow-wt/`),
full-tree copies such as the `/tmp/pse-52d-actual-feature-campaign-*` directories,
`bench-builds` snapshots and in-checkout `--target-dir` experiments. The main `target/` keeps
stable keys and is not affected.

**Direction.** Upstream fix: issue #2870 (open since 2026-09-22) states this exact cause, and
two PRs are in review:

- #2678: basedirs normalization for cwd, `CARGO_MANIFEST_DIR` and dep-info env values.
- #2794: the same, but enabled only when an effective `--remap-path-prefix` covers the
  working directory. That remap is what makes rmeta bytes, and therefore the cascade,
  location-independent.

Neither is released. *Proposed:* when a release contains the fix, set `basedirs` to the
checkout and worktree roots and add `--remap-path-prefix=<checkout root>=.` to the
repository rustflags. Judge the remap's effect on panic and debug paths at that time; it
changes every compile's flags, so expect one full rebuild. Running an unmerged PR build of a
build-affecting tool is not recommended.

Locally, nothing short of that fix removes the cascade. Sharing a build directory across
checkouts stays unsound (ADR-0122). If repeated full-tree campaigns matter before the release,
place each purpose's copy at a stable, reused path, so that its second and later runs hit
their own entries. The build guide's sentence "Across checkouts, the shared sccache serves
identical compilations" should be narrowed to "dependencies without build-script path
inputs".

<a id="O03"></a>
## O03: keep the mixed incremental strategy

**No change.** The 6,303 incremental pass-throughs are workspace crates. Within a checkout,
incremental beats a whole-crate cache. Across checkouts, O02 means they would miss anyway,
because `CARGO_MANIFEST_DIR` and the cwd are hashed. Revisit only together with the upstream
fix in O02.

<a id="O04"></a>
## O04: where the Rust misses come from

**Investigated.** Ranked:

1. **New target directories** (O02): about 190 crates and a third of a cold build each time.
   This dominates.
2. **Split keyspace between agents and humans** (O11): every compile run under both
   environments is stored and missed twice.
3. **Legitimate distinct keys**, which need no change: the force-validate variants of `arrow`,
   `arrow-array`, `arrow-data`, `arrow-cast` and their dependents; release, profiling and dist
   profiles; frontend-screen rustflags; toolchain date bumps; lockfile moves.

The 12 lifetime Rust cache errors cannot be attributed after the fact; the managed server
keeps no per-call log. They are not worth new machinery. If they recur during an
investigation, a probe server with the debug log shows them.

<a id="O11"></a>
## O11: one cache keyspace for agent and human builds

***Measured* (key difference); *Proposed* (fix).** `.claude/settings.json` and
`.codex/config.toml` set `CARGO_TERM_COLOR=never` for agent sessions. Direnv and the
interactive terminal do not. Cargo passes its environment to rustc, and sccache hashes every
`CARGO_*` variable outside its short exclusion list (`CARGO_MAKEFLAGS`, `CARGO_REGISTRIES_*`,
`CARGO_BUILD_JOBS`, `CARGO_ENCODED_RUSTFLAGS`). The recording wrapper showed identical
arguments and a different environment key for the same crates, with `CARGO_TERM_COLOR` as
the only differing variable. Both keyspaces already exist in the cache. The cost is
duplicate storage and duplicate first-time misses after every toolchain or lockfile change.

Remove `CARGO_TERM_COLOR` from both agent configurations. Agent shells are not terminals,
so Cargo's default `auto` already disables color. That keeps a single environment boundary
(`scripts/pse_env.py`) rather than adding a normalization there. A small upstream PR adding
`CARGO_TERM_COLOR` to sccache's exclusion list, next to `CARGO_BUILD_JOBS`, would fix it for
everyone; it is a candidate, not a dependency.

<a id="O05"></a>
## O05: cargo-shear to zero

**Selected by the maintainer. Prototype *Tested* in the measurement worktree:**
`cargo check --workspace --all-targets` and `cargo check -p pse-tests-conformance
--all-targets --features force-validate`, both exit 0.

Applied in the worktree: `cargo shear --fix`; a `#[doc(hidden)] pub use miette as __miette;`
in `pse-diagnostics` with the `impl_diagnostic!` macro's 17 `miette::` paths rewritten to
`$crate::__miette::`; and `pse-buildinfo/identity.rs` moved to `src/identity.rs`.

Result:

- Shear errors 43 → 0. The 17 remaining warnings are classified below.
- `Cargo.lock` loses only dependency edges; no package leaves it.
- Unit counts and every third-party feature set are unchanged for `check --workspace`,
  `test --workspace` (force-validate), `build -p pse-py`, `check -p pse-runtime
  --features native-solvers` and `test -p pse-tests-engine`.
- `test -p pse-tests-conformance` goes from 847 to 839 units: the `proptest` family leaves.

The value is manifest truth and a zero baseline, not build time.

| Finding class | Resolution, as tested |
|---|---|
| 13 `miette` declarations used only through `pse_diagnostics::impl_diagnostic!` | `$crate` re-export; the declarations become genuinely removable |
| Truly unused (25 declarations), e.g. `pse-columnar` `proptest`/`allocation-counter`, `pse-runtime` `object_store` and duplicate dev `url`, `pse-py` `pse-kernels`/`url`, conformance `object_store`/`proptest`/`serde`/`serde-saphyr`/`tempfile`/`url`, `xtask` `prettyplease`, `pse-rules` `async-trait`/`pse-authoring` | Removed; everything compiles |
| **Parent-relative `#[path]` false positives** | See below |
| Empty file `pse-modeling/src/kernel_envelopes.rs` | Delete |
| Optional `arrow-cast` in `pse-columnar`, which exists only to apply `force_validate` | Keep, with a package-level `ignored` entry citing ADR-0170 |
| Feature-forwarding-only edges (13 warnings) | Judge each with a unit-graph check (below) |

**Parent-relative `#[path]` false positives.** The workspace has 21 parent-relative
`#[path = "../…"]` modules, a deliberate convention for shared fixtures (`tests/support`,
`tests/fixtures`, `scripts/workspace.rs`). cargo-shear 1.14.0 joins those paths onto an
absolute parent directory and canonicalizes only paths that begin with `.`
(`src/context.rs`). So `/…/src/../identity.rs` never matches the walked file, and the
module's dependencies look unused. Upstream: #587 (unlinked file) and #589 (false
`unused_dependency`), both open.

In the worktree this caused three things:

- `pse-buildinfo`'s `pse-ids`, `serde_json`, `sha2` and `tempfile` were removed wrongly.
  Moving its single-use module into `src/` fixed it.
- `benches` lost `tempfile`, which `tests/support/workflow_runtime.rs` uses; it had to be
  restored.
- Two files are reported unlinked although they are linked:
  - `tests/engine/src/oracle.rs`, included from `tests/engine/tests/pushdown_vs_unpruned.rs`.
    *This corrects the first pass, which suggested deleting it.*
  - `xtask/src/canonical_recovery.rs`.

Recommended: move single-use modules into the conventional layout, as with `identity.rs`.
For the shared-fixture modules, add a package-level `ignored` or `ignored-paths` entry whose
comment cites #589. Shear's redundant-ignore check reports each one once a fixed release
lands. A lexical `..` normalization in `context.rs` is the upstream fix, and a small PR there
is worthwhile.

**Feature-forwarding edges.** In `tests/conformance`, removing the six edges `pse-admission`,
`pse-engine`, `pse-operations`, `pse-rules`, `pse-schema` and `pse-testkit` (each only
forwarding `force-validate`) leaves force-validation on every crate in the closure unchanged.
`pse-runtime/force-validate` already reaches them, and `pse-testkit` leaves the closure
(839 → 838 units). The crate still compiles with `--features force-validate`. This matches
ADR-0170's "propagation through existing dependencies". Apply the same unit-graph comparison
to the remaining forwarders in `benches`, `pse-runtime`, `tests/engine`, `tests/governance`
and `tests/lifecycle`. Remove an edge when validation is unchanged, and keep it with an
ignore entry where it is the only activation path.

<a id="O06"></a>
## O06: shear's test-target analysis

***Measured* finding count and cost.** `--check-test-targets` reports 22 libraries with
doctests enabled but none written, and 5 with an empty test harness. An empty doctest pass
costs 1.3–2.1 s per crate when warm (`pse-compiler`, `pse-operations`, `pse-relations` and
`pse-engine` through `scripts.arrow_validation test --doc`), and 3.0 s for all four in
parallel. Setting `doctest = false` / `test = false` therefore saves seconds per
`just doctest`. Include the flag in the routine shear call, so that a crate gaining its first
doctest is reported instead of silently skipped.

<a id="O07"></a>
## O07: one detector: retire cargo-machete and cargo-udeps

***Measured.***

- **cargo-machete:** its findings are a subset of shear's, plus build-script false positives
  (`cc`, `pkg-config` and `sha2` in the `-sys` crates' `build.rs`, `toml` in
  `pse-backend-native/build.rs`, `pyo3-build-config`). Its `--with-metadata` mode, which
  `just deps-report` uses, rewrote six tracked `vendor/*/Cargo.lock` files and created three
  more during this review; they were restored by hand.
- **cargo-udeps:** on the shear-clean worktree it ran for 416 s (a full compile). Its findings
  were shear's feature-forwarding warnings again, plus false positives for uses behind
  feature gates it never compiles: `#[cfg(all(test, feature = "native-acceptance"))]` in
  conformance, and `#[cfg(all(test, feature = "canonical-tests"))]` in `pse-operations`.

Shear analyzes every `cfg` branch statically, which is the right model for this
feature-heavy workspace. `cargo shear --expand` was not run: the workspace compile served as
the compiler-backed check of every removal. Remove `audit-machete`, `just udeps` and their
bootstrap entries (DP-01).

<a id="O08"></a>
## O08: pin cargo-shear and run it in `just hygiene`

**Recommended, following the maintainer's adoption decision.** At 0.4 s with no compile,
shear fits the scope-end hygiene bundle (prime directive 1). It is the adopted detector in an
existing bundle, not a new lint. Shear findings change with its release, so pin it in
`[cargo-tools]`. `scripts/audit_tools.py` already classifies its exit codes. Keep it out of
`turn-end` and the edit hook.

<a id="O09"></a>
## O09: cargo-hakari stays retired

***Measured.***

- Workspace unification already keeps feature sets identical across selections (unit
  graphs). The opt-in features are the intended exceptions.
- Hakari's `unify-target-host` could merge 32 host/target duplicates in `test --workspace`,
  25 of them direct feature splits.
- In a cold uncached build (541 s wall, 5,822 CPU-seconds), the extra copies cost about
  45 CPU-seconds in total: `syn` 16.6 s, `regex-automata` 9.8 s, everything else under 4 s.
  That is 0.8%, a few seconds of wall time at 16 jobs.
- A workspace-hack's wider small-member closures (ADR-0175: 423 → 382 units on removal)
  outweigh that.

Revisit only if Cargo gains native host/target unification.

<a id="O10"></a>
## O10: content fingerprints (`-Z checksum-freshness`)

***Measured*, worktree, `check --workspace --all-targets`, 16 jobs:**

| Scenario | mtime fingerprints (current) | content fingerprints |
|---|---|---|
| No-op build | 0.62 s, 0 rebuilt | 0.64–0.87 s, 0 rebuilt |
| `touch crates/pse-ids/src/lib.rs` (content unchanged) | 16.0 s, 29 crates rechecked | 0.65 s, 0 rebuilt |
| `git stash` + `git stash pop` (21 files rewritten) | not run | 1.7 s, 0 rebuilt |
| One-time switch | n/a | 420 s, all 518 crates rechecked once |

Agents stash, switch, rebase and run formatters that rewrite files with identical content
all the time. Each of these currently re-checks or rebuilds the touched crates and everything
downstream: in a test build that means re-linking test binaries, and in incremental work it
means re-running rustc. *Proposed:* add `[unstable] checksum-freshness = true` and
`[build] fingerprint = "content"` to `.cargo/config.toml`, and add both keys to
`CONFIG_KEYS` in `scripts/installation_context.py`, which otherwise refuses unknown Cargo
configuration keys. Limits from Cargo's reference: build-script `rerun-if-changed` inputs
still use mtimes, and the checksum algorithm may change between Cargo versions, which a
toolchain bump already rebuilds anyway. The scheme stays per checkout; it does not make a
shared build directory sound.

## Interactions and priorities

1. **One small tooling plan, with no ADR:**
   - O01: sccache entry plus bootstrap.
   - O11: two configuration lines.
   - O05 and O06: manifests, the `pse-diagnostics` re-export, the `identity.rs` move, deleting
     `kernel_envelopes.rs`, ignores citing #589 and ADR-0170, test-target keys.
   - O07: retire machete and udeps.
   - O08: pin shear and route it into hygiene.
   - O10: two config keys plus `CONFIG_KEYS`.
   - The build-guide narrowing from O02.

   Acceptance:
   - `cargo shear --check-test-targets` exits 0.
   - `just check` and `just check-package pse-tests-conformance --features force-validate`
     pass.
   - `just build-cache-probe` passes on the pinned 0.18.0.
   - The unit-graph census shows unchanged feature sets.
2. **Upstream (optional, small, high leverage):**
   - cargo-shear `..` normalization (#589).
   - sccache `CARGO_TERM_COLOR` exclusion.
3. **Revisit trigger:** a sccache release containing #2678 or #2794. Then evaluate `basedirs`
   plus `--remap-path-prefix` against the O02 probe: a fresh target directory should hit
   nearly everything.
4. **Not recommended:** re-adopting hakari; disabling incremental; remote, multi-level or
   distributed sccache; running an unmerged sccache build.

<a id="revision"></a>
## Revision

The first pass (commit `68e44a37d`) left the sccache misses unattributed. It recommended
editing the library overview, proposed deleting `tests/engine/src/oracle.rs`, and left udeps,
checksum freshness and hakari's cost as open probes. After the maintainer's direction
(upgrade sccache; adopt cargo-shear; leave the overview unchanged), this revision:

- adds O11;
- rewrites O02 and O04 from the read-only miss attribution;
- corrects the `oracle.rs` and `pse-buildinfo` classifications and adds the `#[path]`
  false-positive class;
- turns O05–O10 from reasoning into measured results.

Opportunity references are unchanged; O11 is new.

## Sources

- sccache v0.17.0/v0.18.0: `src/compiler/rust.rs` (`CACHE_VERSION`, hash-key steps 3–10),
  `src/compiler/c.rs` (`CACHE_VERSION`, basedirs), `src/config.rs` (client-side mode is
  disabled by `SCCACHE_LOG`), `docs/Configuration.md` (`SCCACHE_LOCAL_RW_MODE`), release notes
  (#2824, #2827, #2833, #2841, #2843); issue #2870; PRs #2678, #2794.
- cargo-shear v1.14.0: `src/context.rs` (linked-path canonicalization),
  `src/source_parser.rs`; issues #587, #589.
- Cargo unstable reference: `checksum-freshness`, `build.fingerprint`.
- [Rust caching overview](../libraries/rust_caching.md) (cited, unchanged).
- Repository owners: ADR-0122, ADR-0165, ADR-0170, ADR-0175, `docs/dev/build-performance.md`,
  `scripts/installation_context.py`. Design standard 3.4: DP-01, DP-09, DP-13, DP-16, AP-07.
