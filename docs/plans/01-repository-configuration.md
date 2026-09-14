---
title: Repository configuration and working environment for pse-arrow
status: in-progress
date: 2026-09-13
adrs: [ADR-0001 … ADR-0038]
phase: 0
---

# Plan: repository configuration and working environment for `pse-arrow`

## Completion ledger (2026-09-13)

The initial design below is historical planning context. The repository is now public
at `paul-heyse/pse-arrow`; the starting commit for completion is `3c9c95f`, already
pushed to `main`, with a clean working tree. Completion uses signed commits and a
pull request, followed by verification of the merged commit. No release is published.

| Work | Status / acceptance |
|---|---|
| Environment | Tested: bootstrap in a fresh clone with relative and absolute venv paths (including spaces); doctor clean; directory entry made zero network connections |
| Solver access | Tested: immutable CI/dev pins agree with consumers; container preflight passes; no-cache rebuild matches published library checksums |
| Claude/Codex | Implemented: nine native roles, shared skills/rules/hooks; Tested: ten setup fixtures and Codex role/skill discovery; interactive hook trust and Claude usage-limit recovery remain |
| CI / packaging | Implemented: parity install, coverage artifact upload, unique check names, five-platform wheels with ten clean installs, sdist build/install; final CI evidence is on PR #1 |
| GitHub | Tested: main/tag rulesets active; settings re-applied without duplicate objects; two consecutive comparisons report zero differences |
| Qualification | Tested: local Rust/Python gates, depth-two feature combinations and release tests, fresh bootstrap, six negative controls; final PR/main checks remain the merge acceptance record |

**Tested (starting state):** `just doctor`, local mode, reports 1 blocking failure
and 3 warnings, baseline zero. `just lint-agents` and `just adr-lint`, read-only local
mode, report 0 failures, baseline zero. These are structural checks, not runtime
certification. The latest Python CI fails selecting the parity venv; Rust CI is pending.

Keep the recorded phase boundaries: real model generators, API-reference doc lint,
release tooling and numerical parity beyond the existing preflight are deferred.
Neither a clean generated tree nor a successful preflight proves those capabilities.
Registry publishing and new release tags are outside this completion pass.

## Original context (historical)

The revision-3 blueprint (`docs/design_review/Arrow-native-idaes-core-architecture-blueprint-rev3.md`) is the authoritative architecture: a Cargo workspace of 23 `pse-*` crates, a pyo3/pyo3-arrow extension, a Python package `pse`, shipped reference packages, and the §24.1 test/governance matrix that the repository has to be able to run. Nothing in the current working tree is under version control, no toolchain file exists, and the only pinned artifacts are the capability-map evidence lockfiles under `docs/design_review/evidence/`. The user asked for a best-in-class repo configuration and working environment (Rust and Python toolchains, testing, public GitHub setup, design-decision/ADR process) aligned to rev 3.

**Decisions the user has made in this session**

| Decision | Choice |
|---|---|
| Repository home | New public repo under `paul-heyse`; `idaes-arrow` (the fork) stays as-is |
| Naming | Crate prefix `pse-*`, import name `pse`; PyPI distribution `pse-arrow` (`pse` is taken) |
| License | `MIT OR Apache-2.0` |
| Solvers in CI | Ipopt 3.14.x + MUMPS + ASL built from pinned sources in a cached container image; HSL is a local opt-in recorded by `probe_host` |
| MSRV | `rust-version` == pinned stable toolchain (1.98.1 today); blueprint §3.1's "1.94.0" becomes "DataFusion's floor", amended by ADR |
| Docs tool | mdBook on GitHub Pages; Python API docs deferred (register row) |

**Constraints inherited from the blueprint (verified against the text):** `=`-pinned arrow 59.3.0 / datafusion 55.1.0 families, committed `Cargo.lock`, CI one-version-per-family assertion (`cargo tree -d` cannot detect a mixed family); edition 2024; `cargo deny` + `cargo audit`; generated sources committed and diffed in CI (relations, Python contracts, `docs/generated/`, Ipopt bindgen output) via an explicit step, never `build.rs`; `force_validate` in test/CI builds only; every Python library `==`-pinned with two interpreter ranges (platform ≥ 3.11; parity 3.11–3.13 with `idaes-pse==2.12.0`); test families `tests/{governance,engine,conformance,lifecycle,structural,golden}`, `python/pse/tests`, `python/pse/parity`; residual risks CI must re-check (pre-1.0 crate floors, Ipopt/HSL/MUMPS recipe per platform, FeOs tracking num-dual 0.15, `datafusion-proto` byte stability).

**Environment facts:** stable 1.98.1 current, nightly 1.100 is the machine default (a `rust-toolchain.toml` is mandatory); `tooling/dfarrow-apiex` keeps its own `nightly-2026-08-18` pin (rustdoc JSON format 61); uv 0.12.13, maturin 1.15.0, ruff, pyrefly (author's checker), cargo-{nextest,deny,audit,hack,llvm-cov,semver-checks,insta,msrv,machete,shear,mutants,geiger,binstall}, just, taplo, typos, ast-grep, docker/podman installed; system Ipopt is 3.11.9 (too old); `gh` authenticated as `paul-heyse`; uv workspaces enforce one `requires-python` (intersection), so the parity set cannot be a narrower workspace member.

**Conventions reused from `paul-heyse/idaes-arrow`:** `AGENTS.md` canonical + short real `CLAUDE.md` starting `@AGENTS.md`; justfile recipes grouped by cost tier (`env/discovery/local/pr/scheduled/mutating`) with `[group]/[doc]/[confirm]`, mutating recipes never a gate dependency; `.envrc` "fast, no network" + stdlib `scripts/doctor.py`; `scripts/bootstrap.sh` with exact cargo-binstall pins; `deny.toml`/`clippy.toml`/`.config/nextest.toml`/`.taplo.toml`/`.editorconfig`; `.cargo/config.toml` that never sets `target-cpu=native`; `panic = "unwind"`; tools from `.venv/bin` never `$PATH`; `.python-version` as the single interpreter source of truth; Dependabot family groups; `.claude/settings.json` allow/ask/deny + hooks + path-scoped rules; `.claude/skills → .codex/skills` symlink; `plans/NN-*.md` with a README index. Dropped because the repo is greenfield: the `.quality/*-baseline.json` ratchet (zero tolerance instead).

---

## 1. Repository identity

- `paul-heyse/pse-arrow`. Description: "Arrow-native process systems engineering core in Rust: typed relations as the only model authority, a relational math IR, DataFusion-based inference, native NLP solving, and a generated Pyomo backend. Clean-room re-implementation of core IDAES-PSE capabilities, parity-tested against idaes-pse 2.12.0. Not affiliated with IDAES." Topics: process-systems-engineering, apache-arrow, datafusion, rust, pyo3, chemical-engineering, flowsheet-modeling, equation-oriented-modeling, nonlinear-optimization, ipopt, pyomo, idaes.
- `docs/relationship-to-idaes.md` (the analogue of idaes-arrow's `FORK.md`): clean-room (read `external/idaes-pse` for behaviour, never copy; the parity harness is the only coupling; enumerations preserved by name are listed in blueprint §6.14), the 2.12.0 parity pin and how it moves (ADR), trademark/non-affiliation statement, citation of the IDAES paper. Summarised in `README.md`.

## 2. Layout and migration

```
pse-arrow/
├── Cargo.toml  Cargo.lock  rust-toolchain.toml  deny.toml  clippy.toml  rustfmt.toml  .cargo/config.toml  .config/nextest.toml
├── pyproject.toml  uv.lock  .python-version  conftest.py
├── justfile  .envrc  .editorconfig  .gitattributes  .gitignore  .pre-commit-config.yaml  .taplo.toml  typos.toml  sgconfig.yml  sgrules/
├── AGENTS.md  CLAUDE.md  README.md  CONTRIBUTING.md  GOVERNANCE.md  SECURITY.md  CODE_OF_CONDUCT.md  CHANGELOG.md  CITATION.cff
├── LICENSE-MIT  LICENSE-APACHE  LICENSES/{MIT.txt,Apache-2.0.txt}  REUSE.toml  cliff.toml
├── crates/pse-*/ (23 per blueprint §3.2)  crates/pse-ipopt-sys/  xtask/  benches/
├── tests/{governance,engine,conformance,lifecycle,structural}/   # workspace member crates pse-tests-<name>
├── tests/golden/                                                 # data: golden stores + fingerprints (not a crate)
├── python/pse/ (contracts/ GENERATED, codec/, authoring/, adapters/pyomo/, parity/, tests/)  python/stubs/
├── packages/reference/
├── docker/solvers/{Dockerfile, build.sh, checksums.sha256, conda/env.yml, test/hs071_c.c, LICENSES.md, README.md}  .devcontainer/
├── docs/{book.toml, SUMMARY.md, README.md, relationship-to-idaes.md, dev/ci.md, doc-lint-allow.toml}
│   ├── authoritative_design/{README.md, blueprint.md, proposal.md}
│   ├── adr/{README.md (generated), template.md, register.md, NNNN-*.md}
│   ├── plans/{README.md, NN-*.md}
│   ├── design_review/{design_principles/, reviews/}          # unchanged: the design-review skill's output contract
│   ├── capability-maps/{arrow-rust,datafusion-rust,supporting-rust-libraries,python-libraries}.md
│   │   ├── evidence/{README.md, rust/ (lockfiles, probes, outputs, gen scripts), python/ (probes, apidump.py, apiq.py, exported requirements)}
│   │   └── facts/{arrow593,df551,support}.paths.txt.zst        # compact path indexes for doc-lint (full facts are 364 MB: not committed)
│   └── generated/                                              # pse-schema output; committed; linguist-generated
├── external/                     # gitignored: scripts/fetch-external.sh clones idaes-pse@2.12.0, arrow-rs@59.3.0, datafusion@55.1.0
├── scripts/{doctor.py, bootstrap.sh, adr.py, check_register.py, check_agent_config.py, check_generated.py, fetch-external.sh, labels-sync.sh}
├── tooling/dfarrow-apiex/        # kept; workspace `exclude`; own nightly rust-toolchain.toml; hard-coded /home/paul paths replaced
├── .github/{CODEOWNERS, PULL_REQUEST_TEMPLATE.md, ISSUE_TEMPLATE/, dependabot.yml, labels.yml, setup/*.json, actions/setup-rust/, workflows/}
├── .claude/{settings.json, hooks/, rules/, agents/ (canonical), skills -> ../.codex/skills}
└── .codex/{skills/ (canonical: design-review, adr), agents -> ../.claude/agents}
```

Migration from `/home/paul/idaes-pse-rust-datafusion` (new repo initialised in a fresh directory; the old tree is left untouched):

| Current | New | Action |
|---|---|---|
| proposal | `docs/authoritative_design/proposal.md` | front matter `status: historical` |
| blueprint rev 2, then rev 3 | `docs/authoritative_design/blueprint.md` | two seeding commits, tags `design-rev2`, `design-rev3`; revision table gains a `git` column; no `-revN` files from now on |
| `design_principles/`, `reviews/` | unchanged paths | skill contract |
| four capability maps + `evidence/` | `docs/capability-maps/` (+ `evidence/`) | fix relative links in the two evidence READMEs; drop the self-declared superseded top-level `apisurface-Cargo.toml`/`gen_rustdoc.sh`; keep `arrow_probe.rs` |
| `tooling/dfarrow-apiex/` | same | `ROOT` from `git rev-parse --show-toplevel` / `Path(__file__).parents[2]`; `verify/Cargo.toml` path deps → `external/`; README anchors 55.1.0/59.3.0 |
| `build/`, `target/` | not committed | regenerable: `just evidence-regen` |
| `idaes-pse/` (2.13.0rc0), `.worktrees/` (59.2.0/55.0.0) | `external/` at the pinned tags | not submodules; tags read from `Cargo.lock` (arrow/datafusion) and the `parity` group (IDAES); a lint checks the script's IDAES tag equals the pin |
| `.claude/` == `.codex/` | canonical + symlinks | `check_agent_config.py` |

## 3. Rust workspace, toolchain, and supply chain

**`Cargo.toml` (root).** `[workspace] resolver = "3"`, `members = ["crates/*", "tests/governance", "tests/engine", "tests/conformance", "tests/lifecycle", "tests/structural", "xtask", "benches"]`, `exclude = ["tooling/dfarrow-apiex", "tooling/dfarrow-apiex/verify", "docker"]`. `[workspace.package]`: `version = "0.0.1"`, `edition = "2024"`, `rust-version = "1.98.1"` (== toolchain by governance test), `license = "MIT OR Apache-2.0"`, `repository`, `authors`, `publish = false` (flipped per library crate at phase-1 exit; never for `pse-py`, `xtask`, `benches`, `tests/*`). `[workspace.dependencies]` carries every §3.1 pin exactly once with features fixed there: arrow family `=59.3.0` (`arrow` with `ipc`, `ffi`, `canonical_extension_types`; `arrow-ipc`/`arrow-cast` `default-features = false`; `parquet` with `arrow`, `async`, `object_store`), datafusion family `=55.1.0` (defaults only), `object_store =0.13.2`, `pyo3 =0.29.x` + `pyo3-arrow =0.19.0`, `tokio =1.53.1` (rt-multi-thread, sync, macros, time), `serde =1.0.229` (derive, rc), `serde_arrow =0.15.0` (`arrow-59`), `serde-saphyr =1.2.0`, `toml =1.1.6`, `winnow =1.0.4`, `syn =3.0.5`/`quote =1.0.47`/`proc-macro2 =1.0.107` (`span-locations`)/`prettyplease =0.3.0`, `petgraph =0.8.3`, `num-dual =0.15.0`, `faer =0.24.4`, `blake3 =1.8.7` (`rayon`), `rayon =1.12.0`, `tracing =0.1.44`, `thiserror =2.0.20`, `miette =7.6.0`; dev/tooling pins (proptest, insta, criterion, tempfile, tracing-subscriber, cargo_metadata, clap, regex, ignore, bindgen, pkg-config, anyhow for xtask only) listed in `tests/governance/tooling_deps.toml` with reasons so the "every crate appears in §3.1" test can exempt them; `feos`, `diffsol`, `egglog` enter only with their consumer crate and phase. Patch versions not yet verified are pinned to whatever the seeded lockfile resolves. `[workspace.metadata.pse]`: `blueprint` path, `validate-features = ["pse-relations/force-validate"]`, `unsafe-allowlist = ["pse-ipopt-sys", "pse-backend-native", "pse-kernels-ext", "pse-py"]`, `[workspace.metadata.pse.families]` (arrow 59.3.0 over `arrow*`/`parquet*`; datafusion 55.1.0 over `datafusion*`; object_store 0.13.2; pyo3 0.29 minor-match over `pyo3*`) consumed by `cargo xtask family-check`. No `[patch]`/`[replace]` ever (governance test).

**Lints (`[workspace.lints]`, every crate `lints.workspace = true`).** rust: `unsafe_code = "deny"` (the four allowlisted crates carry `#![allow(unsafe_code, reason = "...")]`), `missing_docs`, `unreachable_pub`, `unused_qualifications`, `trivial_casts`, `rust_2018_idioms`, `missing_debug_implementations` warn (`-D warnings` in CI), `non_ascii_idents = "forbid"`; rustdoc `broken_intra_doc_links = "deny"`; clippy `all` + `pedantic` + `cargo` warn (priority −1), `multiple_crate_versions`/`module_name_repetitions`/`must_use_candidate`/`cast_precision_loss` allowed, panic policy `unwrap_used`/`expect_used`/`panic`/`unimplemented`/`todo`/`exit`/`mem_forget`/`print_stdout`/`print_stderr`/`dbg_macro` deny, `disallowed_methods`/`disallowed_types`/`disallowed_macros` deny, `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`, `allow_attributes_without_reason` deny, `float_cmp` warn. `clippy.toml`: `allow-{unwrap,expect,panic,dbg,print}-in-tests = true`, `avoid-breaking-exported-api = false`, `doc-valid-idents` (DataFusion, PyO3, IDAES, Ipopt, MUMPS, …), `disallowed-methods` = the type-aware layer of the §24.1 bans (`Field::extension_type`, `SessionConfig::set_str`, `SchemaLike::from_type`/`from_samples`, `IpcWriteOptions::try_with_compression`; definition paths confirmed against the extracted facts at implementation), `disallowed-types = [anyhow::Error]` (xtask opts out at crate level).

**Profiles.** `dev`: `debug = "line-tables-only"`, `[profile.dev.package."*"] opt-level = 2` (arrow/datafusion/faer at −O0 make the suite crawl; float semantics unchanged); `debugging` (inherits dev, `debug = "full"`); `release`: `opt-level = 3`, `lto = "thin"`, `codegen-units = 16`, `panic = "unwind"` (required by PyO3 and the Ipopt `catch_unwind` callbacks), `debug = "line-tables-only"`, `strip = "none"`; `profiling` (inherits release, full debug, for samply); `dist` (inherits release, `lto = "fat"`, `codegen-units = 1`; used by `maturin --profile dist` and release binaries). **`force_validate` mechanism:** it is an arrow feature, not a profile; `pse-relations` declares `force-validate = ["arrow/force_validate"]`, never in defaults; every test invocation (justfile `test`/`coverage`, the CI composite action, `cargo llvm-cov`) passes `--features pse-relations/force-validate` explicitly (no alias, so workflows stay readable); a governance test asserts the feature exists and is named in `validate-features`.

**Toolchain and cargo config.** `rust-toolchain.toml`: `channel = "1.98.1"`, `profile = "minimal"`, components rustfmt, clippy, rust-src, rust-analyzer, llvm-tools; comment states nightly is never the default and lives only in `tooling/dfarrow-apiex/`. `.cargo/config.toml`: deliberately minimal — `[alias] xtask = "run --quiet --package xtask --"`, `[future-incompat-report] frequency = "always"`; no linker override (rustc ≥ 1.90 already links x86_64 Linux with rust-lld; mold stays a per-user opt-in in `~/.cargo/config.toml` rather than a bootstrap requirement), never `target-cpu=native` (FMA contraction would change float results between dev and CI), sccache only via `RUSTC_WRAPPER` in `.envrc`/justfile, no `--locked` key exists so every gate passes it explicitly; a governance test forbids `linker =`, `target-cpu`, `[build] rustflags`, `[patch]`, `paths =` in this file. `rustfmt.toml`: `style_edition = "2024"`, `max_width = 100`, field-init and try shorthand, Unix newlines; generated files carry `#![rustfmt::skip]`.

**Per-crate template.** `version/edition/rust-version/license/repository/authors/publish.workspace = true`, `[lints] workspace = true`, deps via `.workspace = true`. Features from phase 0: `pse-relations/force-validate`; `ipopt` (default on) in `pse-backend-native`, `pse-runtime`, `pse-py` with `pse-ipopt-sys` optional so `--no-default-features` checks cleanly without Ipopt; `pse-kernels-ext/{coolprop,feos}` off; `pse-mathir/egglog` off (phase 4); `pse-backend-native/diffsol` off (phase 4).

**`deny.toml`.** `[graph] targets` = the five wheel targets, `all-features = true`; `[advisories] yanked = "deny"`, `unmaintained = "workspace"`, `ignore = []` (every entry needs id, reason, owner, review date); `[licenses] allow` = MIT, MIT-0, Apache-2.0 (+LLVM exception), BSD-2/3, ISC, Zlib, 0BSD, BSL-1.0, Unicode-3.0/DFS-2016, CC0-1.0, CDLA-Permissive-2.0, MPL-2.0; `[bans] multiple-versions = "deny"`, `wildcards = "deny"`, `allow-wildcard-paths = true`, `deny` list = serde_yaml, uom, arrow-flight, arrow-avro, datafusion-spark, arrow-pyarrow, parquet-variant, parquet_derive, openssl-sys (each with the §3.1/§3.3 reason), `skip` seeded from the first `cargo deny check bans` run (the evidence lockfiles predict syn 1/2 beside syn 3, thiserror 1 under pyo3-arrow, hashbrown/rand/getrandom generations, zstd 0.13/0.14, …; every entry reason + owner + review date), `skip-tree = [windows-sys]`; `[[bans.features]] crate = "datafusion" deny = ["serde", "avro", "parquet_encryption", "backtrace"]`; `[sources]` unknown registry/git denied. Plus `cargo audit` and `cargo shear` in the same job.

**`.config/nextest.toml`.** Test groups `solver` (max-threads 2; `pse-backend-native`, lifecycle, `solve_*` tests; 300 s slow timeout) and `serial` (object-store/golden writers). `default`: fail-fast, 60 s slow timeout. `ci`: `fail-fast = false`, `retries = 0` (a flaky test is a red test), 120 s slow timeout × 3, `global-timeout = "2h"`, `failure-output = "immediate-final"`, JUnit to `target/nextest/ci/junit.xml`. `flaky-hunt`: opt-in exponential retries with `flaky-result = "fail"` for diagnosis only. Every gate pairs `cargo nextest run` with `cargo test --doc --workspace --locked` (nextest does not run doctests).

**`xtask` (logic) vs `justfile` (surface).** The justfile is the one-line command surface in cost tiers; xtask owns anything needing Rust APIs, JSON, or cross-platform behaviour. `xtask` subcommands: `codegen [--only relations|python|docs|bindgen]` (calls `pse_schema::codegen::generate`; emits `// @generated by pse-schema <fingerprint>`, `#![rustfmt::skip]`, `#![allow(clippy::all, clippy::pedantic, missing_docs, reason = "generated")]`; Python emitted final-form; bindgen from `$IPOPT_DIR/include/coin-or/IpStdCInterface.h` with an allowlist of the §18.3 functions, runs inside the solver container), `codegen --check` (`git diff --exit-code` plus untracked-file check on the four generated paths), `family-check [--evidence docs/capability-maps/evidence/rust/*.lock]` (one resolved version per family from `cargo metadata --locked`; packages shared with an evidence lock must match), `governance`, `doc-lint` (backticked `a::b::c` and `[rustdoc:…]` identifiers in `docs/**` resolve in the committed path indexes or in workspace rustdoc JSON generated with the apiex nightly pin; allowlist file), `probe-host [--json]` (thin wrapper over `pse_runtime::probe_host()`), `release <version>` (bump workspace version, `git cliff`, commit, tag). `tests/governance/` (crate `pse-tests-governance`): `no_shadow_structs`, `banned_patterns` (regex layer of the §24.1 greps incl. `config_options(` under kernels, `SERDE_ARROW:`, `{:?}` as hash input in `pse-ids`), `pins_match_blueprint`, `every_crate_registered`, `toolchain_matches_msrv`, `dependency_floors` (every resolved package's declared `rust_version` ≤ ours — the blueprint's "pre-1.0 floors re-checked at every upgrade"), `no_patch_tables`, `unsafe_allowlist`, `ffi_callbacks_catch_unwind`, `error_taxonomy` (every `pub enum *Error` derives thiserror + miette::Diagnostic with §23.2 codes; no `anyhow`/`miette::Result` in `crates/*`), `blake3_owner`. `benches/` = `pse-benches` (criterion, `harness = false`, one bench per §24.3 group, saved baselines per snapshot; CI smoke = `cargo test --benches` with no timing gate).

**Justfile recipes** (all gates `--locked`): env `bootstrap`, `bootstrap-venv|quality|solvers|rust-tools|linters`, `doctor`, `doctor-json`, `fetch-external`; discovery `versions`, `lib-outline`, `metadata`; local `check`, `clippy`, `fmt-check`, `test`, `doctest`, `docs`, `codegen-check`, `family-check`, `governance`, `ci-fast` (= fmt-check check clippy test doctest); pr `policy` (deny+audit+shear), `coverage`, `bench-smoke`, `doc-lint`, `adr-lint`, `quality` (python), `parity`, `ci-pr` (= ci-fast governance policy docs bench-smoke quality adr-lint); scheduled `features-powerset`, `test-release`, `udeps`, `mutants-file`, `unsafe-surface`, `floors-latest`; decisions `adr-new`, `adr-index`, `adr-supersede`, `plan`, `register-check`; mutating (`[confirm]`) `fmt`, `codegen`, `snapshots-accept`, `release`, `solver-image`, `evidence-regen`, `labels-sync`, `gh-setup`. `scripts/bootstrap.sh --rust-only` binstalls the exact CI tool set (nextest 0.9.143, deny 0.20.2, audit 0.22.2, shear, machete, llvm-cov 0.9.0, insta 1.48.0, hack 0.6.45, msrv, mutants, geiger, udeps, semver-checks 0.50.0, git-cliff, pinact, zizmor, taplo, typos, maturin 1.15.0).

**Release/versioning.** Single workspace version (`0.0.x` in phase 0, `v0.1.0` at phase-1 exit), one `v*` tag per release on `main`; `cargo-semver-checks` from the first tag (informational on PRs, gating on PRs labeled `release`); changelog by `git-cliff` from Conventional-Commit squash titles (`cliff.toml`; types feat/fix/perf/refactor/docs/test/build/ci/chore/deps/adr/design, scopes = crate short names), title convention enforced by `governance / pr-title`; crates.io publishing tooling (release-plz vs cargo-workspaces) decided by ADR at phase-0 exit.

## 4. Solver container and native linkage

`docker/solvers/` builds Ipopt `releases/3.14.20` (tag commit `1e71ba4…`, tarball sha256 `43bddd6f…`, corroborated by Homebrew and conda-forge), ThirdParty-Mumps `releases/3.0.14` (MUMPS 5.9.1 + its patches), ThirdParty-ASL `releases/2.1.0` (`solvers-20241108`), from tarballs verified against `checksums.sha256`; base `ubuntu:24.04@sha256:<digest>`. Choices inside the recipe: netlib reference BLAS/LAPACK (deterministic; no threaded BLAS), `--without-metis` like IDAES's `compile_solvers.sh` so MUMPS ordering matches IDAES binaries and iteration counts stay comparable, no HSL ever (Ipopt 3.14's `hsllib` option dlopens a local `libhsl.so`; `probe_host` records `ma27/ma57/ma86/ma97` availability; verify `hsllib` behaviour at implementation), `-O2 -fPIC`, no `-march=native`; post-build test compiles `hs071_c.c` via `pkg-config` and runs `bin/ipopt` on a tiny `.nl`. Stages: `solvers` (runtime libs, headers, `.pc`), `ci` (+ build-essential, git, pkg-config, libclang for bindgen, uv with managed CPython 3.11/3.12/3.13 for the parity job; `IPOPT_DIR=/opt/pse-solvers`, `PKG_CONFIG_PATH`, `LD_LIBRARY_PATH`, `PATH`), `dev` (+ rustup pinned toolchain, cargo tool set, just, uv) used by `.devcontainer/devcontainer.json`. `solvers-image.yml`: build on PRs touching `docker/solvers/**`; on `main` build + push `ghcr.io/paul-heyse/pse-solvers:{ipopt3.14.20-mumps5.9.1-asl20241108-r1, ci-<tree-hash>, dev-<tree-hash>}` and open a PR updating the `SOLVER_IMAGE@sha256` env in the workflows; weekly `solvers-image-rebuild-check` rebuilds without cache and compares library checksums (recipe reproducibility). `LICENSES.md` records Ipopt EPL-2.0, MUMPS CeCILL-C, ASL, netlib.

`crates/pse-ipopt-sys`: `links = "ipopt"`; `build.rs` emits link directives only (`IPOPT_DIR` or `pkg_config ... atleast_version("3.14.0")`; `cargo:version=` → `DEP_IPOPT_VERSION` recorded by `probe_host`; actionable error naming the container, `brew install ipopt`, or the conda env); bindings committed by `xtask codegen --only bindgen`. `index_style = 0` assertion and `catch_unwind` per callback live in `pse-backend-native`.

Phased OS coverage: phase 1a Linux container gating; phase 1b macOS (Homebrew `ipopt` 3.14.20) and Windows (conda-forge `ipopt` 3.14.20 via micromamba, `CARGO_TARGET_DIR=C:\t`, `shell: bash`) non-gating until a green week, trajectory-parity tests filtered off-Linux; phase 2 run `build.sh` natively on macOS and under MSYS2 on Windows, cached by recipe hash, and build the NL external-function libraries in the same jobs (blueprint §26). Wheel linkage (bundle libipopt/MUMPS via auditwheel/delocate/delvewheel vs runtime `libloading`) is a phase-1 ADR + register row; phase-0 wheels build `pse-py` without the `ipopt` feature.

## 5. Python packaging, environments, and tests

**Decisions.** Distribution `pse-arrow`, import `pse`. One root `pyproject.toml`, no uv workspace, committed `uv.lock`. maturin backend, `dynamic = ["version"]` from `crates/pse-py/Cargo.toml` (workspace-inherited: one version authority). pyomo/pint/numpy are the `pyomo` extra, not core (`pse.open/compile/solve` are native; the adapter serves parity and ecosystem tools, §21.3); numpy/scipy otherwise only in the `test` group. pyrefly type-checks (`[tool.pyrefly]` in pyproject), ruff lints/formats, import-linter enforces module boundaries, REUSE enforces headers. Zero tolerance, no baselines.

**`pyproject.toml` (key content).** `[build-system] requires = ["maturin>=1.15,<2"]`; `[project] name = "pse-arrow"`, `requires-python = ">=3.11"`, `license = "MIT OR Apache-2.0"`, `license-files = ["LICENSES/*.txt"]`, `dependencies = ["pyarrow==25.0.1", "attrs==26.1.0", "cattrs==26.2.0", "msgspec==0.21.1"]` (§3.1 `==` rule; O1 below), `[project.optional-dependencies] pyomo = ["pyomo==6.10.1", "pint==0.26.1", "numpy==2.5.3"]`. `[dependency-groups]`: `quality` (ruff, pyrefly, import-linter, reuse, taplo, typos, pre-commit, maturin; all `==`), `test` (pytest, pytest-cov, pytest-xdist, numpy==2.5.3, scipy==1.18.1), `parity` (includes `test` + `"idaes-pse==2.12.0; python_full_version < '3.14'"`), `evidence` (includes `parity`), `docs` (empty for mdBook; reserved), `dev` (quality + test). The marker keeps `uv sync --group parity` on 3.14 honest; if a future IDAES pin conflicts with the platform pins `uv lock` fails, which is the right signal. `[tool.uv]`: `package = true`, `required-version = "==0.12.13"`, `python-preference = "managed"`, `default-groups = ["dev"]`, `cache-keys` on `pyproject.toml`, `Cargo.toml`, `Cargo.lock`, `uv.lock`, `rust-toolchain.toml`, `crates/**/Cargo.toml`, `crates/**/*.rs`; `[[tool.uv.index]] name = "testpypi"` with `publish-url`, `explicit = true`. `[tool.maturin]`: `bindings = "pyo3"`, `manifest-path = "crates/pse-py/Cargo.toml"`, `module-name = "pse._native"`, `python-source = "python"`, `python-packages = ["pse"]`, `locked = true`, `profile = "release"`, `editable-profile = "dev"`, `strip = true`, `auditwheel = "repair"`, sdist `include` of `uv.lock`, `rust-toolchain.toml`, `REUSE.toml`; wheel `exclude` of `python/pse/tests/**`, `python/pse/parity/tests/**`; `abi3-py311` is a pyo3 feature in the crate, not a maturin key. `[tool.ruff]`: `target-version = "py311"`, `line-length = 88`, explicit `select` = idaes-arrow's set (E,W,F,I,B,C4,UP,SIM,RUF,PT,PERF,N,ARG,PTH,TRY,PLC,PLE,PLW,ISC) + ANN, TC, TID, D (google), S, T20, ERA, PGH, PIE, RET, SLF, FA, LOG, G, DTZ, PYI, NPY, FURB, FLY, INP, EXE, ICN, TD; `ignore = ["ISC001", "TD002"]`; `format.exclude = ["python/pse/contracts/**"]`; `runtime-evaluated-base-classes = ["msgspec.Struct"]` + attrs decorators; `banned-api` (TID251): `typing.Any`, `cattrs.Converter/GenConverter/structure/unstructure` (use `pse.codec.converter()`), `pyarrow.register_extension_type` (use `register_all()`), `assert_units_consistent` (use `identify_inconsistent_units`), `json`, `tomllib` (msgspec), `pickle`; per-file ignores for tests, generated contracts, codec, tools. `[tool.pyrefly]`: includes, `search-path = ["python", "python/stubs"]`, `python-version = "3.11"`, `check-unannotated-defs = true`, `ignore-errors-in-generated-code = false`, sub-configs `replace-imports-with-any` for `idaes.*`/`pyomo.*`/`pint.*` under `parity/**` and `adapters/pyomo/**` until typed stubs land in phase 1 (confirm key names with `pyrefly dump-config`; rely on venv auto-discovery rather than an OS-specific interpreter path). `[tool.importlinter]`: numpy/scipy only in `pse._array`, `pse.parity.**`, `pse.tests.**`; `idaes`, `pandas`, `pydantic`, `sympy`, `networkx`, `matplotlib`, `click` never outside `pse.parity`; pyomo/pint only in `pse.adapters.pyomo.**`, `pse.parity.**`, tests; `pse.contracts` depends on nothing in `pse`; layers `pse.parity` > `pse.adapters | pse.authoring` > `pse.codec` > `pse.contracts`. `[tool.pytest.ini_options]`: `--strict-markers --strict-config --import-mode=importlib -ra --durations=100 --durations-min=2`, `xfail_strict = true`, `filterwarnings = ["error"]` + commented allow-list policy (scoped entries with upstream issue links), `required_plugins`, markers `unit|component|integration|performance|parity|golden(name)`; `[tool.coverage]` branch, `source_pkgs = ["pse"]`, `fail_under = 90`, site-packages path mapping.

**Package layout.** `python/pse/{__init__.py, py.typed, _native.pyi (hand-written; a test diffs it against dir(_native)), _build.py (BuildInfo msgspec struct), _array.py (only non-parity numpy importer; zero_copy_only=True; copy_reason mandatory), contracts/ (GENERATED: attrs classes, msgspec manifest structs, ten ExtensionTypes, enums, GENERATED.sha256), codec/ (converter factory with forbid_extra_keys; msgspec helpers; transform_error → findings), governance.py (import-time attrs.fields walk failing on Any/bare dict/bare list), authoring/, adapters/pyomo/, parity/ (+tests), tests/}`. No `from __future__ import annotations` under `python/pse` (PEP 563 breaks the import-time `Any` lint); ast-grep enforces.

**Build info.** A `pse-buildinfo` crate (used by `pse-catalog` for the manifest's `toolchain.lockfile_hash` and by `pse-py`) whose `build.rs` hashes `Cargo.lock` and `uv.lock` (blake3 for the manifest; sha256 twins for Python verification), plus rustc version, profile, git sha; `pse.build_info()` returns a frozen msgspec `BuildInfo`; `BUILD_INFO.json` is attached to releases; a test fails when the built extension is stale relative to the checkout's lockfiles.

**uv workflow.** `uv sync --locked --extra pyomo` (platform dev on 3.14.7 from `.python-version`); `uv run maturin develop --uv --release` for a release-profile extension; `MATURIN_PEP517_ARGS="--profile release" uv sync --reinstall-package pse-arrow` in CI test jobs; parity env `UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --extra pyomo --group parity --python 3.13`; `uv lock --check` in pre-commit and CI; `uv lock --upgrade-package <name>` is the only sanctioned way to move a pin; `uv sync --check` feeds `doctor.py`. `.envrc` = idaes-arrow's minus IDAES vars, plus `PSE_GOLDEN_DIR` and a guarded `PATH_add /opt/pse-solvers/bin`. Evidence probes run from the `evidence` group; the hand-maintained `requirements-py31x.txt` become `uv export --frozen …` outputs written by `just evidence-regen`.

**Test plugins (root `conftest.py`, stdlib + pytest).** (1) exactly one of `unit|component|integration|performance` per item, enforced at collection with one `UsageError` listing all offenders; (2) `--performance` with IDAES semantics; (3) `--parity`: without it parity items are deselected with a one-line summary; with it a session autouse fixture fails (never skips) unless Python < 3.14, `idaes.__version__ == "2.12.0"`, and `ipopt` is on PATH; `pytest.skip`/`importorskip` under `parity/**` banned by ast-grep; (4) `VerifyCleanup` ported from IDAES and extended to fail when a test modifies a tracked file. Fixtures (`python/pse/tests/conftest.py`): `golden(name)` opens `tests/golden/<name>` read-only (stores are produced by `cargo xtask golden`; Python never writes them), `registered_extension_types`, `no_numpy_on_import` (subprocess), `build_info_matches_checkout`. Boundary tests per §24.1: `test_forbidden_extra_keys`, `test_any_lint`, `test_extension_round_trip`, `test_nullable_ndarray_refusal`, `test_preflight_capability_backend` (raises before `ConcreteModel` construction), `test_manifest_fingerprint`, `test_native_stub_surface`, `test_versions_agree`; parity `test_00_preflight` asserts `probe_host` reports Ipopt 3.14.x, `mumps`, ASL, IDAES 2.12.0 and `SolverFactory("ipopt").available(False)`, and writes `artifacts/probe_host.json`.

**Wheels and release.** `wheels.yml` via `PyO3/maturin-action@v1` (`command: build`, `--release --out dist --locked`, `sccache: true`): ubuntu-24.04 x86_64 and ubuntu-24.04-arm aarch64 at `manylinux: 2_28` (pyarrow 25.0.1 ships only 2_28), macos-15 arm64 + macos-15-intel x86_64 (thin wheels), windows-2022 x64; assert every wheel tag is `cp311-abi3`; sdist via `uv build --sdist` then a PEP 517 install from the sdist in a fresh venv; `verify-clean-install` on 3.11 and 3.14 from a checkout. `release.yml` on `v*`: version gate (`cargo metadata` pse-py version == tag == every wheel filename; `CHANGELOG.md` heading exists) → wheels/sdist → PEP 740 attestations (`astral-sh/attest-action`) → `uv publish --index testpypi --trusted-publishing always` (environment `test-release`) → TestPyPI install smoke with retry → `uv publish` (environment `release`) → GitHub Release with `dist/*`, `uv.lock`, `Cargo.lock`, `uv export` snapshots (platform, parity), `BUILD_INFO.json`, `probe_host.json`, solver-image digest.

**Pre-commit (`default_install_hook_types: [pre-commit, pre-push]`).** ruff-format, ruff-check (no `--fix`); local `language: system` hooks from `.venv/bin`: taplo fmt --check, typos, reuse lint, `uv lock --check`, `scripts/check_generated.py` (sha256 manifest of `python/pse/contracts/`), ast-grep scan, actionlint, shellcheck; pre-push: pyrefly, import-linter, cargo fmt --check, cargo clippy. `scripts/bootstrap.sh --linters-only` fetches ast-grep/actionlint/shellcheck.

**ast-grep rules (`sgrules/`).** Python: `no-future-annotations`, `to-numpy-only-in-array-boundary`, `no-direct-native-import`, `identify-not-assert-units`, `no-skip-in-parity`, `no-zero-copy-false-without-reason`. Rust: `arrow-through-datafusion`, `pyarray-new-panics`, plus structural forms of the §24.1 bans.

## 6. CI job graph and required checks

Workflow files: `rust.yml`, `rust-scheduled.yml`, `python.yml`, `wheels.yml` (`workflow_call` + PR paths), `parity.yml`, `solvers-image.yml`, `release.yml`, `docs.yml`, `governance.yml`, `register-review.yml`, `repo-hygiene.yml`; composite `.github/actions/setup-rust` (rustup with `--default-toolchain none` → `rustup show` installs the pinned toolchain; `Swatinem/rust-cache` with `shared-key` per job kind and `save-if` on main; `taiki-e/install-action` with exact tool versions from env). Common: `permissions: contents: read` at top, widened per job; `concurrency` per PR/ref with `cancel-in-progress` on PRs only; `CARGO_INCREMENTAL=0`, `CARGO_TERM_COLOR=always`, `RUST_BACKTRACE=1`, tool versions as env vars, `SOLVER_IMAGE=ghcr.io/paul-heyse/pse-solvers:ci-<hash>@sha256:<digest>`; no `RUSTFLAGS` in CI; sccache off in CI; every action pinned by commit SHA with a version comment and kept current by Dependabot `github-actions` + `pinact`; `zizmor` + `actionlint` in `repo-hygiene`.

| Check context (required on `main`) | Job | Runs |
|---|---|---|
| `rust / fmt` | fmt | `cargo fmt --all --check`; `taplo fmt --check` |
| `rust / clippy` | clippy (solver container) | `cargo clippy --workspace --all-targets --locked -- -D warnings`; again with `--no-default-features` |
| `rust / test` | test (solver container) | `cargo nextest run --workspace --locked --profile ci --features pse-relations/force-validate`; `cargo test --doc --workspace --locked`; `cargo test --benches -p pse-benches --locked`; JUnit artifact |
| `rust / codegen-diff` | codegen-diff (container: Ipopt headers + libclang) | `cargo xtask codegen --check` |
| `rust / family-check` | family-check | `cargo xtask family-check --evidence docs/capability-maps/evidence/rust/*.lock`; `cargo metadata --locked` |
| `rust / deny` | deny | `cargo deny check`; `cargo audit`; `cargo shear` |
| `python / lint` | lint | `uv sync --locked --group quality --no-install-project`; ruff format --check, ruff check, pyrefly, import-linter, reuse lint, typos, taplo, `uv lock --check`, `check_generated.py` |
| `python / test` | test (3.11 and 3.14) | `MATURIN_PEP517_ARGS="--profile release" uv sync --locked --extra pyomo --group test --no-default-groups`; `pytest -m "unit or component" -n auto --cov` |
| `python / parity` | parity (container `ci` stage, 3.13 on PRs; 3.11–3.13 nightly) | installs the Linux x86_64 wheel artifact, `uv run --no-sync pytest --parity` |
| `docs / build` | build | `adr.py index --check`; `mdbook build docs`; `lychee --offline` on the book; pages artifact |
| `governance / adr-lint` | adr-lint | `adr.py lint`; `check_register.py --lint`; `needs-adr` label rule; IDAES tag in `fetch-external.sh` == parity pin |
| `governance / pr-title` | pr-title | `amannn/action-semantic-pull-request` (types/scopes from `cliff.toml`) |

Non-required but run on PRs: `rust / docs` (`RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps`), `rust / doc-lint` (paths-filtered), `rust / coverage` (`cargo llvm-cov nextest … --lcov`, Codecov OIDC, informational in phase 0), `rust / semver` (from the first tag; gating only with the `release` label), `rust / msrv-floors` (`cargo hack check --rust-version` only when `Cargo.lock` changed), `rust / test-macos` and `rust / test-windows` (phase 1b, `continue-on-error` until a green week), `wheels / *` (paths-filtered), `governance / reuse`, `governance / agent-config`, `repo-hygiene / *`. `docs / deploy` on main via `actions/deploy-pages`. `rust-scheduled.yml` weekly (cron `0 6 * * 1`, each job with an owner and a documented response in `docs/dev/ci.md`): feature powerset (`cargo hack --feature-powerset --depth 2`), `test-release` (`--cargo-profile release`), `udeps` (pinned nightly), `mutants` on `pse-ids`/`pse-mathir` (`continue-on-error`, artifact), `unsafe-surface` (geiger), full `doc-lint`, `floors-latest` (`cargo update --dry-run` + floors against a temp lock), `solvers-image-rebuild-check`. Miri is dropped (no `unsafe` outside FFI crates); re-adoption trigger = first non-FFI `unsafe` (register row). `codecov.yml`: informational, `after_n_builds` = number of uploads.

Required-check staging: phase-0 set = the six `rust / *`, `docs / build`, `governance / adr-lint`, `governance / pr-title`; `python / *` added by a `just gh-setup` re-run once the Python package first reports (a check that never reports blocks every PR).

## 7. Decision records and the design process

**Two directories, two lifecycles.** `docs/adr/NNNN-kebab.md` records decisions (immutable after acceptance except status fields; changed by superseding). `docs/plans/NN-kebab.md` records how work is sequenced and verified (living until done, then an appended `## Outcome` with "a mistake made and corrected" and "deviations, deliberate"; "when the code and a plan disagree, the code is what runs"). Plans list the ADRs they implement in front matter.

**ADR template** (`docs/adr/template.md`): MADR 4 sections plus charter §H fields in YAML front matter: `id`, `title`, `status` (proposed | accepted | rejected | deprecated | superseded), `date`, `deciders`, `level` (decision | should-deviation | must-gap — a must-gap narrows scope, never claims compliance), `principles` (DM-xx), `blueprint` (sections governed), `review` (path#finding or `not-required: <reason>`), `evidence` (charter §D label), `supersedes`/`superseded-by`, `revisit` (observable trigger), `verification` (test/lint/benchmark that shows the decision holds). Body: Context · Scope · Drivers · Options · Outcome (Consequences, Compensating controls, Confirmation) · Pros/cons · More information · Status history (append-only). A small deviation fills each section in one line.

**Tooling:** `scripts/adr.py` (stdlib, ~200 lines): `new`, `lint` (file-name pattern, contiguous numbering, required keys, enums, `DM-\d{2}`, blueprint `§` anchors resolve to headings in `blueprint.md`, review path exists, symmetric supersession, immutability of accepted ADRs vs `origin/main`, register rows valid), `index [--check]` (regenerates `docs/adr/README.md` and the ADR block in `docs/SUMMARY.md`), `supersede`. `just adr-new|adr-lint|adr-index|adr-supersede|plan|register-check`. A small `.codex/skills/adr/SKILL.md` navigator explains when an ADR is required and how to fill §H fields. `adr-tools` rejected (unmaintained shell, no §H fields).

**Decision-PR rule** (`GOVERNANCE.md`): an ADR enters or changes status only in a PR labeled `adr`, titled `adr: ADR-NNNN <title>`; the same PR (or a named follow-up `design:` PR) amends `blueprint.md` with a revision row and an inline `> Decision: ADR-NNNN` at the governed section; maintainer merge is the approval; a PR may merge with `status: proposed` only if also labeled `needs-review`.

**When an ADR / review is required:**

| Change | Needs |
|---|---|
| Alters D1–D14; adds/removes a crate; adds/drops/majors a dependency family; changes hashing contract, Python boundary contract, metadata conventions, or commit contract; any SHOULD deviation; governance changes | ADR + design review (`needs-review`; verdict Accept/Accept-scoped before `accepted`) |
| New relation family, pass, kernel contract, or backend binding within an accepted decision; small local SHOULD deviation; moving the parity pin; a deferred trigger fires | ADR (short); review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, docs wording, patch bumps inside a pinned family, tooling | neither; ordinary PR with the evidence field filled |

**Blueprint convention:** one file, revised in git; section numbers are stable citation targets (insert `§14.3.1`, never renumber); §0.1 states "this file is authoritative; ADRs record why; reviews are evidence, not authority".

**Register** (`docs/adr/register.md`): rows `R-NN | item | ADR | trigger | check | owner | last-checked | next-check | status`. Initial rows: salsa (§14.3); `LogicalPlan::Extension` (§14.2); `datafusion-tracing`/`instrumented-object-store` release matching the pinned engine (crates.io API check); `datafusion-ffi`; `egglog` determinism (phase 4); `feos-core` tracking num-dual 0.15 (`cargo info` check); pre-1.0 crate floors per upgrade (`dependency_floors`); `object_store` multi-writer commit; Ipopt/HSL/MUMPS recipe per platform (phase 1 exit); wheel solver linkage (phase 1 ADR); `datafusion-proto` byte stability per bump; Miri re-adoption; crates.io release tooling (phase 0 exit); Renovate fallback; Python API docs tool; project board at second contributor; Windows symlink policy. `register-review.yml` (monthly cron, `issues: write`) runs `check_register.py --due`, executes automatable checks, and opens/updates one `Register review YYYY-MM` issue.

**Backfill ADRs in the seeding PR** (short §H records citing the blueprint; evidence mostly Proposed/Interface-checked): 0001 ADR process; 0002 name/license/prefix/PyPI distribution; 0003 clean-room relationship + IDAES 2.12.0 parity pin; 0004–0017 D1–D14; 0018 family-wide `=` pins, lockfile, cargo deny, **MSRV = pinned toolchain (amends §3.1)**; 0019 plan fingerprint is evidence, not a memo key; 0020 salsa deferred; 0021 serde-saphyr; 0022 num-dual 0.15 + FeOs conditional; 0023 blake3 derive_key in pse-ids + `pse.canon.v1`; 0024 pyo3-arrow over arrow-pyarrow; 0025 `LogicalPlan::Extension` deferred; 0026 uom/arrow-flight dropped, pint validates never defines; 0027 commit contract = P2 validity; 0028 solver acquisition (source-built Ipopt+MUMPS, HSL probed with declared fallback); 0029 FairSpillPool with explicit limit; 0030 canonical float hashing, no Float64 keys; 0031 generated sources committed and diff-checked; 0032 external checkouts not vendored; 0033 one blueprint file, stable section numbers; 0034 governance (single maintainer, PR-only, squash-only, linear signed main); 0035 Dependabot with family groups; 0036 mdBook on Pages; 0037 remaining deferred-with-trigger items; 0038 workspace layout additions (`crates/pse-ipopt-sys`, `pse-buildinfo`, `xtask/`, `benches/`, `tests/*` crates; amends §3.2).

**Doc conventions:** new files lowercase kebab-case (exceptions: the three UPPER_SNAKE principle files, root governance files, the skill's `design_review_{slug}_{date}.md`); YAML front matter on ADRs, plans, capability maps (`pins:`, `regenerated:`), authoritative design (`status:`, `revision:`); citations `blueprint §14.3`, `ADR-0020`, `DM-33`, `G4`; charter §D evidence labels mandatory in ADR `evidence:`, the PR template, plan Verification sections, register rows, with `Tested`/`Measured` naming the test/benchmark and conditions; generated docs carry `<!-- @generated by pse-schema; do not edit -->`.

## 8. GitHub setup

Declared config in `.github/setup/*.json`, applied idempotently by `just gh-setup` (`gh api` PUT/PATCH; rulesets updated by name; `gh ruleset` in gh 2.45 is read-only).

1. `gh repo create paul-heyse/pse-arrow --public --disable-wiki --description … --homepage https://paul-heyse.github.io/pse-arrow`; push seeded history + tags.
2. `gh repo edit`: default `main`, squash merge only (title = PR title, body = PR body), delete branch on merge, allow update branch, Discussions on, wiki/projects off; topics via PUT.
3. Security: vulnerability alerts, automated security fixes, private vulnerability reporting, secret-scanning push protection.
4. Ruleset `main` (`~DEFAULT_BRANCH`): deletion, non_fast_forward, required_linear_history, required_signatures (squash commits are GitHub-signed; SSH signing documented for bypass pushes), pull_request (0 approvals — sole maintainer; dismiss stale; require thread resolution; squash only), required_status_checks (strict; the §6 list, staged). Bypass: RepositoryRole admin `always` (fallback `User` id if the role id is rejected); every bypass requires a `governance` follow-up issue. Ruleset `tags` on `refs/tags/v*`: creation/update/deletion/required_signatures. Branch naming documented, not enforced: `<kind>/<kebab>`.
5. Labels (`.github/labels.yml`, `just labels-sync`): `kind/{bug,feature,design-decision,parity-gap,library-upgrade,docs,chore,governance}`, `area/{schema-registry,identity-catalog,physical-typing,materials-properties,math-ir,compiler-passes,rules-inference,numerics-solvers,backends,python-boundary,diagnostics,docs,governance,ci,tooling}`, `phase/{0-foundations,1-slice-a,2-slice-b,3-slice-c,4-breadth}`, `adr`, `needs-review`, `needs-adr`, `register`, `release`, `blocked`, `dependencies`. Milestones per delivery phase 0–4.
6. Issue forms: `bug-report.yml`, `design-decision.yml`, `parity-gap.yml` (IDAES reference, tolerance, which side is believed correct and why), `library-upgrade.yml` (family, capability-map regeneration, register rows); `config.yml` with blank issues off and links to Discussions/SECURITY.
7. PR template: Summary; `Implements: ADR-NNNN` / `Plan:` / `Closes:`; Evidence (§D label + named tests/benchmarks); checklist (ADR referenced or "no ADR needed because…", generated sources regenerated, lockfiles updated deliberately, capability map regenerated if a family moved, register rows touched, no edits under generated/external/build); legal acknowledgement (MIT OR Apache-2.0) + clean-room attestation.
8. Root files: `SECURITY.md` (private reporting, 7-day acknowledgement, FFI/untrusted-document scope), `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1, contact paul@heyse.io), `CONTRIBUTING.md` (environment, repo map, change workflow, when an ADR/review is required, the "proves / does not prove" table, generated code and pins, docs conventions, licensing, release pointer), `GOVERNANCE.md`, `LICENSE-MIT`/`LICENSE-APACHE` + `LICENSES/` for REUSE, SPDX two-line header on every authored source file (`SPDX-License-Identifier: MIT OR Apache-2.0`, `Copyright (c) 2026 Paul Heyse`; the generator emits it) enforced by `reuse lint` (`REUSE.toml` covers headerless types by glob), `CITATION.cff` (references the IDAES paper as the parity reference), `CHANGELOG.md` via git-cliff.
9. `dependabot.yml`: cargo monthly, `versioning-strategy: increase`, groups `arrow-family` (`arrow*`, `parquet*`), `datafusion-family` (`datafusion*`, `object_store`), `pyo3-family` (`pyo3*`, `numpy`), `codegen` (`syn`, `quote`, `proc-macro2`, `prettyplease`; PR must carry a regeneration commit), `numerics` (`num-dual`, `faer*`, `petgraph`, `diffsol`, `feos*`, `egglog`); uv monthly with groups `quality-tools`, `runtime`, `ignore: idaes-pse`; github-actions weekly grouped; docker for the solver base image. Renovate is the fallback (register row).
10. Environments `test-release` (branch policy `main`) and `release` (tag policy `v*`, reviewer = maintainer, `prevent_self_review: false`); PyPI trusted publisher for `release.yml`; Pages source = workflow.

## 9. Agent working environment

- `AGENTS.md` canonical (~150 lines): identity + clean-room rule; "start here" (`just doctor` → `just bootstrap` → `just --list`; "`just --list` is the contract"); prime directives (baseline is zero; never edit generated dirs, fix the generator; one authoritative declaration per meaning; label every claim with §D vocabulary; ADR before changing a D-decision or a family; tools from `.venv`/pinned toolchain, never `$PATH`; report failure counts with baseline); repository map; where authority lives (blueprint, ADR index, register, `Cargo.toml` header, `[dependency-groups]`, capability maps via `just lib-outline`); invariants; gotchas seeded from real incidents (umbrella pin does not pin the family; deleted extraction made rustdoc markers unreproducible; PyPI `pse` taken; `/home/paul` paths; YAML/Actions/PowerShell items from idaes-arrow); "what each command proves / does not prove" table; documentation and decision rules; off-limits paths; runtimes.
- `CLAUDE.md`: real file starting `@AGENTS.md`, then settings/hooks/rules summary, "use plan mode when the change touches D1–D14, adds a crate or dependency, touches codegen or the Python boundary", subagents, skills, "plans go in `docs/plans/`, never `~/.claude/plans`".
- `.claude/settings.json`: allow list of read-only `just`/`cargo`/`.venv/bin`/`git`/`gh` read commands; ask for push/commit/merge/rebase/reset/tag, `gh api`, `gh pr create/merge`, `just adr-new|plan|labels-sync|gh-setup|evidence-regen|solver-image`, publishes, `uv pip install`, `uv add`, `cargo add`; deny force-push and Edit/Write under `.git`, `target`, `build`, `external`, `docs/generated`, `crates/**/generated`; env `UV_PROJECT_ENVIRONMENT=.venv`; hooks `session-start.sh` (emits `just doctor`), `guard-protected-paths.sh` (PreToolUse, exit 2; also blocks accepted ADRs and `docs/authoritative_design/**` unless `PSE_DESIGN_EDIT=1`), `format-after-edit.sh` (PostToolUse; cargo fmt / ruff format / taplo fmt; never fails). No per-edit `check-fast` hook.
- `.claude/rules/{rust,python,docs,decisions,generated,ci}.md` with `paths:` globs. Keep the nine agents and the design-review skill; add the `adr` skill; `just lint-agents` runs `check_agent_config.py`.
- `.editorconfig` (idaes-arrow's; 88 for `.py`, 100 for `.rs`), `.gitattributes` (`* text=auto eol=lf`; `*.arrow`/`*.parquet`/`*.feather`/`*.ipc` binary; `*.nl`/`*.sol` LF; `linguist-generated` for lockfiles, `docs/generated/**`, `crates/*/src/generated/**`, `*.snap`, evidence lockfiles/logs/outputs), `.gitignore` (`/target/`, `/build/`, `/external/`, `/docs/book/`, `/dist/`, `.venv*/`, caches, `python/pse/_native*.so`, `*.pyd`, `.envrc.local`, `.claude/settings.local.json`, tooling targets, `/.worktrees/`, `/idaes-pse/`, `artifacts/`).

## 10. Docs site

mdBook (latest 0.5.x pinned exact at implementation; installed by `cargo-binstall`). `docs/book.toml` (`src = "."`, `site-url = "/pse-arrow/"`, repo/edit URLs) + `docs/SUMMARY.md` (overview, relationship-to-idaes, authoritative design, decisions index + register + generated ADR block, plans, design reviews, capability maps + evidence READMEs, generated reference, contributing/governance/security includes). `docs / build` never needs the Rust extension (`docs/generated/` is committed); rustdoc output is copied into the book when the `rust / docs` artifact exists. Python API docs (mkdocstrings) revisited via register row when `python/pse` grows a public surface.

## 11. Execution order and verification

Each step is one PR after the seeding commits (a fresh repo cannot require checks that do not yet exist, hence the staged rulesets in §6/§8).

1. **Init and seed history.** New directory (e.g. `~/pse-arrow`), `git init -b main`; commit 1: design principles + proposal + blueprint rev 2 + first review (tag `design-rev2`); commit 2: blueprint rev 3 + second review (tag `design-rev3`); commit 3: capability maps + evidence (links fixed) + `tooling/dfarrow-apiex` (paths fixed).
2. **Root scaffolding.** Licenses, `LICENSES/`, `REUSE.toml`, README, relationship doc, CONTRIBUTING/GOVERNANCE/SECURITY/CoC/CITATION/CHANGELOG, `.editorconfig`, `.gitattributes`, `.gitignore`, `.taplo.toml`, `typos.toml`, justfile, `.envrc`, `scripts/doctor.py`, `scripts/bootstrap.sh`, `scripts/fetch-external.sh`.
3. **Rust workspace skeleton.** Root `Cargo.toml` with every pin, toolchain/cargo/rustfmt/clippy/deny/nextest files; 23 crates + `pse-ipopt-sys` + `pse-buildinfo` + `xtask` + `benches` + five `tests/*` crates as minimal libs with full manifests; `cargo generate-lockfile`, seed `deny.toml` skips from the first `cargo deny check bans`; `cargo xtask family-check --evidence`; the governance tests that run on an empty workspace (pins match blueprint, every crate registered, toolchain matches msrv, dependency floors, no patch tables, unsafe allowlist, blake3 owner, error taxonomy).
4. **Python skeleton.** `pyproject.toml`, `uv.lock`, `.python-version`, root `conftest.py`, minimal `python/pse` (init, `_native.pyi`, `_build.py`, governance lint), minimal `pse-py` pymodule with `build_info`; `uv sync --locked`; `test_versions_agree`, `test_native_stub_surface`, `build_info_matches_checkout`; pre-commit config; sgrules.
5. **Solver image.** `docker/solvers/`, local build, push to GHCR, `.devcontainer`; `pse-ipopt-sys` build.rs against the image; `cargo xtask codegen --only bindgen` inside the container; `probe_host` smoke via `cargo xtask probe-host`.
6. **Decision records and docs.** `scripts/adr.py`, template, 38 backfill ADRs, register, `docs/plans/README.md` + plan 01 (this plan, transcribed), `book.toml`/`SUMMARY.md`, `adr` skill, blueprint §0.1 authority note + `git` column + ADR citations for the amendments (§3.1 MSRV, §3.2 layout additions).
7. **Agent environment.** `AGENTS.md`, `CLAUDE.md`, `.claude/settings.json`, hooks, rules, symlinks, `check_agent_config.py`.
8. **CI.** Workflows, composite action, `codecov.yml`, `dependabot.yml`, labels, issue forms, PR template, `.github/setup/*.json`, `cliff.toml`; `pinact run` + `zizmor` clean.
9. **GitHub.** `gh repo create`, push, `just gh-setup` (phase-0 ruleset), environments, Pages, milestones, labels; PyPI pending publisher.
10. **Verification pass** (recorded in plan 01's Outcome):
    - fresh clone → `just doctor` reports what is missing → `just bootstrap` → `just doctor` clean → `just ci-fast` and `just ci-pr` green locally; `direnv` entry touches no network.
    - `cargo xtask family-check` fails on a deliberately mixed family in a scratch branch; `rust / codegen-diff` fails on a hand edit under `crates/pse-relations/src/generated/`; `governance / adr-lint` fails on an ADR with a bad `§` anchor and on an edit to an accepted ADR; `python / lint` fails on a `typing.Any` field; a parity run on 3.14 fails (not skips) the pre-flight.
    - a direct push to `main` is rejected by the ruleset; a PR without a conventional title fails `governance / pr-title`; `just gh-setup` re-run is a no-op.
    - `python / parity` in the container finds `ipopt` 3.14.20 and passes the pre-flight; `solvers-image-rebuild-check` reproduces library checksums.
    - `docs / build` + `docs / deploy` publish the book; `lychee` finds no broken internal links.
    - `wheels.yml` produces `cp311-abi3` wheels on all five targets and `verify-clean-install` passes on 3.11 and 3.14.

## Open questions for the maintainer (recommended default in bold)

- O1 `==` pins in `[project.dependencies]` of the published wheel (blueprint §3.1 rule; hostile to consumers): **keep through phases 0–3; revisit by ADR at the first PyPI release**.
- O2 Wheel solver linkage (bundle Ipopt/MUMPS vs runtime `libloading`): **phase-1 ADR; phase-0 wheels ship without the native backend**.
- O3 Reserve `pse-*` names on crates.io now: **no; publish at phase-0 exit**.
- O4 Two arrow/datafusion Dependabot groups vs one: **two** (`family-check` guards).
- O5 Backfill all 38 ADRs in the seeding PR vs D1–D14 only: **all** (short records).
- O6 Free-threaded (`cp314t`) wheels: **defer; nightly non-blocking job later**.
- O7 `pyo3-stub-gen` for `_native.pyi` vs hand-written + surface test: **hand-written now**.
- O8 CoC contact `paul@heyse.io` vs a Discussions category: **email**.

---

## Outcome and remaining runtime activation

### What was built

**Implemented:** setup completion is delivered through [PR #1](https://github.com/paul-heyse/pse-arrow/pull/1).
The shared framework has nine repository-specific Claude roles and generated native
Codex definitions, canonical skills with runtime aliases, and shared session/edit/
format hooks. AGENTS.md routes both runtimes to the same scoped rules. Pyrefly is the
only configured Python type checker; the unused alternative was removed from the
quality group, lockfile, environment and Dependabot configuration.

**Tested:** all counts below use a zero-failure baseline. Local Rust qualification
uses toolchain 1.98.1; Rust test commands explicitly enable
`pse-relations/force-validate`.

| Command / condition | Result and limit |
|---|---|
| `just ci-fast`, dev | 23 Rust tests passed, zero skipped; doctests passed (phase-zero crates contain no doctest cases) |
| `just test-release`, release | 23 Rust tests passed, zero skipped |
| `IPOPT_DIR=<prefix> just features-powerset`, libraries extracted from the pinned CI image | 47 depth-two feature configurations and 31 no-default-feature checks passed |
| `just ci-pr`, local | Composite Rust, governance, policy, docs, benchmark smoke, Python quality and 36 Python tests passed |
| `just quality` | Ruff, Pyrefly, import contracts and repository checks passed; ten setup fixtures passed |
| `just parity-container`, pinned dev image, Python 3.13 | 41 existing package/preflight tests passed; numerical modeling parity is not established |
| Fresh-clone `just bootstrap`, `just doctor`, `just quality` | Relative and absolute venv selection passed; an absolute path containing a space was exercised |
| `strace -f -e connect direnv exec . true` | Zero IPv4/IPv6 connections during directory entry |
| `just solver-rebuild-check` | No-cache source build reproduced published shared-library checksums byte for byte |
| `just gh-setup-check` twice after re-applying setup | Zero configuration differences; main and tag rulesets remain unique and active |

**Tested:** disposable fixtures rejected a staged generated-file edit (`just codegen-check`),
a mixed family (`just family-check`), an invalid blueprint citation (`just adr-lint`),
missing/double pytest cost markers, and parity on Python 3.14. The setup fixtures also
reject accepted-ADR edits against a base ref; existing Python tests reject `typing.Any`.
A temporary non-conventional title on PR #1 failed `governance / pr-title`; restoring
the conventional title restored the check. No negative fixture remains in the source tree.

**Interface-checked:** a fresh Codex runtime listed all nine custom roles; the app server
listed both skills and all three hooks without configuration errors. The hooks are
reported as **untrusted**, so automatic execution is not certified. Shared hook behavior
is covered directly by the setup fixtures.

### Mistakes found and corrected

**Implemented:** the previous Python parity job selected no environment when installing
the wheel; it now names `.venv-parity`. Coverage uploads now run on a host runner with
its verification tools. Windows fixture paths are normalized, directory skill aliases
are excluded from sdists, and benchmark smoke selects the relations crate when enabling
its validation feature. Doctor reads package metadata because invoking a tool wrapper's
`--version` attempted network access. Dependabot's Cargo strategy is `auto`; GitHub
rejected the earlier `increase` value. The wheel action's explicit `stable` override
was removed so it reads the pinned toolchain; clean installs assert the embedded compiler
version matches that pin. Final cross-platform and packaging results are
recorded by PR #1's checks, rather than inferred from local compilation.

### Deliberate boundaries and handoff

**Implemented:** source/configuration parity is complete; this plan stays in progress
until the remaining runtime activation is verified. In Codex, review and trust the three
project hooks with `/hooks`, then start a fresh session and exercise an ordinary edit
and a protected-path rejection. In Claude Code, accept the repository trust dialog and
repeat the `plan-scout` smoke check when the account limit permits: the attempted live
check returned HTTP 429 with the existing monthly spend-limit message.

**Interface-checked:** local commits carry SSH signatures, but GitHub reports the local
key as unknown. Registering that public signing key needs the account's
`admin:ssh_signing_key` scope, which the current credential does not have. The squash
merge uses GitHub's verified signature; no admin bypass is used to waive required checks.
The declared admin-role bypass remains, so an admin credential cannot prove ordinary
contributors' direct pushes are rejected. No test push to main is attempted.

**Implemented:** real model generators, API doc lint, numerical parity, automated solver
pin PR credentials, and package publication retain their existing phase boundaries.
The API-doc job reports its R-20 prerequisite explicitly while indexes are absent.
Generic secret scanning is unavailable for this personal repository; supported secret
scanning and push protection are enabled (see `.github/setup/README.md`). No registry
publisher is activated, package uploaded, or release tag created by this completion pass.
