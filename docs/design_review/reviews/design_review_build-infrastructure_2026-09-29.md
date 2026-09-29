# Design review: build infrastructure (ADR-0122)

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | [ADR-0122](../../adr/0122-nightly-toolchain-and-feature-unification.md). It pins a dated nightly, turns on workspace feature unification, and adds the cargo-hakari crate `pse-workspace-hack`. It supersedes ADR-0018. A build directory shared across this repository's checkouts, with fine-grain locking, was implemented and then withdrawn as unsound ([F09](#f09)). **Neighbours read:** ADR-0018, ADR-0066 and ADR-0117; blueprint §3.1 and §3.2. |
| Standard | Core **3.1** (AP-01–AP-06, DP-01–DP-24, G1–G9); binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). The process-simulator profile does not apply: no model, solver or result contract changes. |
| Tier / purpose | **Change tier, conformance purpose.** The record is judged against the core standard and the decisions it supersedes or lies beside. |
| Reviewer / date | Agent T-B (Plan 22 build infrastructure), 2026-09-28. **Author review, not an independent review.** |
| Decisions | Architectural fitness **passes**; behavioural adequacy **passes** after F01–F05. Overall: **Accept** at the *Implemented* level. See [slot 12](#decision). |
| Disposition owner | ADR-0122; F06 and F07 go to the maintainer as follow-ups. |

**Functional target.** Every recipe selection in a checkout reuses one build of each
heavy dependency per mode and feature role, without weakening the opt-in features, and
no checkout ever builds from another checkout's artifacts.

**Inspected.**
- **Configuration:** `.cargo/config.toml`, `rust-toolchain.toml`, `.config/build.toml`, `.config/hakari.toml` and the generated crate.
- **Scripts:** `scripts/build_environment.py`, `build_measurements.py`, `native-solver-runner.sh`, `audit_tools.py` and `doctor.py`.
- **Checks:** `xtask` `dependency_ceilings.rs`, `scripts/native-engine-boundaries.py` and the governance tests.
- **Consumers:** the CI workflows and `setup-rust` action, `docker/solvers/Dockerfile`, `.devcontainer` and `scripts/parity-container.sh`.
- **Cargo and hakari:** Cargo's reference for `build.build-dir`, `feature-unification` and the `unused_dependencies` lint; the hakari configuration and algorithm documentation (docs.rs).

**Not examined.** Build time, disk use and concurrent builds were not measured. No test
suite was run. The only executed evidence is the unit-graph analysis and the compile
checks named in the record.

## 4. Change scenarios

| ID | Scenario | Expected response |
|---|---|---|
| <a id="s01"></a>S01 | One worktree runs `check-package pse-math`, then `unit-package pse-compiler`, then `unit-native-package pse-runtime pse-runtime/native-solvers` | One unit of symbolica, DataFusion and arrow-array per mode, plus one per mode for the native-solver feature role |
| <a id="s02"></a>S02 | Two worktrees on different branches build the same recipe concurrently | Each builds from its own sources in its own `target/`; identical compilations hit the shared sccache |
| <a id="s03"></a>S03 | A release or production build (`maturin --release`, `bench-production`) | No `force_validate`, and no native-solver crate, unless requested |
| <a id="s04"></a>S04 | Stable Cargo, or Cargo in a CI image, builds the workspace | The build succeeds; hakari supplies the unification that `[unstable]` would |
| <a id="s05"></a>S05 | The nightly date moves | One edit in `rust-toolchain.toml`, then a rebuilt `dev` image; no second date to find |

## Assessment

**Foundations.**
- **AP-04 and DP-01 (satisfied).** The toolchain date has one declaration. Unification has one owner, Cargo, with the hakari crate derived from `.config/hakari.toml`.
- **AP-05 and DP-03 (satisfied after F01).** `force_validate` stays opt-in, and a governance test is its enforcement point.
- **AP-06 (satisfied).** A developer can confirm the effect without compiling, from the unit graph.
- **DP-09 (satisfied after F09).** One unit per dependency and role within a checkout (S01); reuse across checkouts is keyed on the compilation itself (sccache), not on a workspace-relative path (S02).
- **DP-20 (satisfied).** Checkouts do not share Cargo state, so concurrent builds cannot interfere (S02).
- **AP-01, AP-02 and AP-03: not affected.**

**Gates.**
- **G1 (one authority):** pass after F05.
- **G3 (invariants enforced):** pass after F01, F02 and F09.
- **G8 (library first):** pass. Cargo's own feature unification, and hakari rather than scripts.
- **G9:** pass.

**Library fit (interface-checked).**
- Cargo 1.101.0-nightly lists `feature-unification`.
- Stable 1.98.1 ignores `[unstable]` silently and warns once about `resolver.feature-unification`.
- cargo-shear 1.14 recognises the workspace-hack; cargo-machete 0.9.2 needs the workspace ignore.

## 7. Findings

| ID | Finding | Principles / gate / scenario | Consequence | Correction |
|---|---|---|---|---|
| <a id="f01"></a>F01 | Hakari simulates every member with all features. The first generation switched on `arrow` `force_validate` and the native-solver C libraries (sundials-sys, suitesparse_sys, diffsol, clarabel BLAS/LAPACK) for every build | DP-03; G3; S03 | Release wheels would validate every Arrow array. Plain builds would need the native environment | `arrow` and the native-solver crates are excluded from traversal in `.config/hakari.toml`. The governance test `workspace_hack_keeps_force_validate_opt_in` fails if `force_validate` returns. A unit comparison found no opt-in feature in any selection, including release `pse-py`. After rebasing onto `main` (4e0d2107), its new optional `dep:diffsol-la` in `pse-backend-native`'s `diffsol` feature switched on faer's `rayon`, `npy` and `rand` for every build; `diffsol-la` joined the excludes and the comparison is clean again |
| <a id="f02"></a>F02 | `pse-ids` is a build dependency of `pse-buildinfo`, and `pse-diagnostics` is in its closure. Either one depending on the hack built DataFusion and Arrow a second time for the host | DP-09; G3; S01 | A second host build of the heavy graph in every build | Both are in `final-excludes`, and the configuration names the rule for new members |
| <a id="f03"></a>F03 | Test executables are intermediates, so they lived in the shared build directory, which the native runner did not mount | G3; S01 | Native unit tests would fail in the solver container | Withdrawn with the shared build directory (F09): test executables are under `target/` again, which the runner mounts |
| <a id="f04"></a>F04 | `cargo clean` removes the build directory, which was shared by every checkout, and cleaning tools (cargo-geiger, cargo-llvm-cov) run it | DP-20; S02 | A clean in one worktree cold-starts, or breaks, the others | Withdrawn with the shared build directory (F09); `cargo clean` again touches one checkout |
| <a id="f05"></a>F05 | Second nightly declarations: `.config/build.toml` `nightly`, the `udeps` recipe and scheduled job, and the solver `dev` image's `ARG RUST_TOOLCHAIN` | DP-01; G1; S05 | A date move silently misses a consumer | All three removed. The image takes the channel as a build argument from `rust-toolchain.toml` |
| <a id="f06"></a>F06 | The hack widens small members' build closures: `pse-model` alone now builds the unified Arrow and DataFusion crates | DP-17 (build cost only); S01 | The first build of a small crate is slower, though the semantic ceilings still hold for code | Accepted under the maintainer's direction to adjust the checks rather than fight hakari. Alternative: add the semantic roots to `final-excludes` |
| <a id="f07"></a>F07 | Nothing compiles the source on the `rust-version` floor any more | DP-22; S04 | Language-feature drift above 1.98.1 goes unnoticed; std-API drift is caught by clippy `incompatible_msrv` | Follow-up for the maintainer: a scheduled stable build, or raise the floor with each date move |
| <a id="f09"></a>F09 | The shared build directory is unsound for workspace crates. Cargo keys a path package by its workspace-relative path: every `pse-ids` fingerprint under `~/.cargo/build/pse-arrow/debug/build/pse-ids/*/fingerprint/*.json` carried the same `"path"` hash, whichever checkout built it, and freshness is `"mtime"` over package-relative dep-info. The author had assumed each checkout kept its own units | DP-09, DP-03; G3; S02 | A checkout whose sources are older than another checkout's artifact reuses that artifact for different sources. The coordinator's merge of this branch with `main` failed with ``no variant `ConeLoweredRowV1` found for enum `pse_ids::Frame` ``, a variant present in the merged `crates/pse-ids/src/derive.rs`. `-Z checksum-freshness` would instead make checkouts on different branches rebuild over each other's workspace crates | `build.build-dir` and `-Z fine-grain-locking` removed, with everything that existed only for them (the runner mount, CI and devcontainer overrides, isolated cleaning tools, the `cargo clean` gotcha, register R-38). Intermediates are per checkout again; sccache serves cross-checkout reuse because the recipe environment no longer exports the default `CARGO_TARGET_DIR` (`b3fa3141`) |
| <a id="f08"></a>F08 | The nightly surfaced warnings the stable toolchain did not: deprecated `fetch_update`, the `recursion_depth_exceeding_limit` future-incompatibility lint, six `unused_qualifications` in `pse-runtime`'s native-solver code, and Cargo's `unused_dependencies` lint on the hack's feature-only edges | G3 | A non-zero warning baseline | `try_update` (stable since 1.95); `#![recursion_limit = "256"]` in five crates; the qualifications removed; `[workspace.lints.cargo] unused_dependencies = "allow"`, with cargo-shear as the unused-dependency detector. The `proc-macro-error2` 2.0.1 future-incompatibility report comes from the vendored delta-rs `validator_derive` and stays open |

## 11. Disposition

| Finding | Disposition | Owner | Evidence or trigger |
|---|---|---|---|
| F01, F02, F05, F08 | Resolved in ADR-0122 and its implementing commit | T-B | `just check`, `just check-solver-contracts`, `just codegen-hakari-check`, the unit-graph comparison |
| F03, F04 | Withdrawn with the shared build directory | T-B | — |
| F09 | Resolved: the shared build directory is rejected in ADR-0122's Options | T-B | `.cargo/config.toml` sets no `build.build-dir` |
| F06 | Accepted: the widened build closures are hakari's build cost (DP-17), and the semantic ceilings still hold for code | Maintainer | ADR-0122 Consequences |
| F07 | Decided by the maintainer on 2026-09-29: no scheduled stable build; the project tracks a recent nightly and fixes breakage as it appears | Maintainer | §3.1 |

## <a id="decision"></a>12. Decision

**Architectural fitness (G9): pass. Behavioural adequacy: pass** with F01, F02, F05, F08
and F09 corrected. **Overall: Accept** ADR-0122 at the *Implemented* level.

**Strongest evidence.** The no-compile unit-graph counts in the record. Distinct units of
symbolica, DataFusion and arrow-array across seven recipe selections fall from 7 to 4
(check and build, each with and without `native-solvers`). The unit-level feature
comparison shows the hack adds no opt-in feature.

**Main uncertainty.**
- The traversal excludes are a hand-kept list: the first rebase already needed `diffsol-la` added. Only the `force_validate` test and a failing plain build catch a leak.
- The build-time effect of workspace unification and the hakari crate (not measured).
- How much cross-checkout reuse sccache recovers now that each checkout keeps its own `target/`.
