---
id: ADR-0122
title: Pin a dated nightly for workspace feature unification, with a cargo-hakari workspace-hack
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [DP-01, DP-09, DP-13, DP-03, DP-20]
blueprint: [§3.1, §3.2]
review: docs/design_review/reviews/design_review_build-infrastructure_2026-09-29.md
evidence: Implemented
supersedes: [ADR-0018]
superseded-by: null
revisit: Cargo stabilizes `feature-unification` (the nightly pin can then return to a stable channel); a nightly regression or a missing component blocks moving the date; `cargo hakari generate` emits a `force_validate` or native-solver line despite the traversal excludes; or Cargo keys path packages by absolute source location, so a build directory shared across checkouts becomes sound.
verification: Unit-graph comparison of the seven recipe selections without compiling (`cargo … --unit-graph -Z unstable-options`), before and after, counting distinct units of symbolica, DataFusion, arrow-array and faer; `just check`, `just check-solver-contracts` and `just codegen-hakari-check` on nightly-2026-09-29; governance tests `toolchain_matches_msrv`, `every_crate_registered::workspace_hack_keeps_force_validate_opt_in` and `dependency_floors`; `scripts/tests/test_build_environment.py` and `test_audit_tools.py`.
standard: core-3.1
scenarios: []

---

# ADR-0122: Pin a dated nightly for workspace feature unification, with a cargo-hakari workspace-hack

## Context

Cargo resolves dependency features against the packages a command selects. The recipes
select different packages (`check-package`, `unit-package`, `unit-native-package`,
`check-solver-contracts`), so every worktree held two to seven builds of the heavy
dependencies. A no-compile unit-graph count over seven recipe selections found seven
distinct builds each of symbolica, DataFusion and arrow-array, and six of faer. Each checkout
also compiled them again in its own `target/`. ADR-0018 pinned a stable toolchain equal to
`rust-version`, and Cargo's workspace feature unification exists only on nightly
(blueprint §3.1).

## Scope

This record amends blueprint §3.1's toolchain rule and adds a crate under §3.2. It
supersedes ADR-0018 and restates what stands from it. It rejects a Cargo build directory
shared between checkouts (Options). Build performance was not measured, and nothing here
changes a dependency version.

## Drivers

- DP-09: one reuse mechanism keyed on complete dependencies. A dependency with the same
  features and profile should be one unit whatever the selection; reuse across checkouts
  must be keyed on their actual sources.
- DP-20: concurrent builds in several worktrees must not interfere.
- DP-01: one declaration of the toolchain date.
- DP-03: `force_validate` stays opt-in with an enforcement point.
- DP-13: use Cargo's own mechanisms before bespoke scripts.

## Options

| Option | Effect | Result |
|---|---|---|
| Stable, as ADR-0018 decided, with a recipe discipline of one selection per crate | Still two to seven builds per worktree, and one per checkout | Rejected |
| cargo-hakari alone on stable | Unifies selections, but its generator simulates all features, so opt-in features leak unless excluded | Kept as the stable-Cargo carrier only |
| **Dated nightly with workspace unification and the hakari crate** | One unit per crate and role in each checkout; a fallback for Cargo that ignores `[unstable]` | **Selected** (maintainer decision, 2026-09-28) |
| One `build.build-dir` (`{cargo-cache-home}/build/pse-arrow`) shared by every checkout, with `-Z fine-grain-locking` | One build of each dependency per repository, but unsound for workspace crates. Cargo keys a path package by its workspace-relative path: every `pse-ids` fingerprint in the shared directory carried the same `path` hash, whichever checkout built it. Freshness is mtime over package-relative dep-info. A checkout whose sources were older than another checkout's artifact reused that artifact, and a merge of this branch with `main` failed to compile with ``no variant `ConeLoweredRowV1` found for enum `pse_ids::Frame` ``, a variant present in its source. `-Z checksum-freshness` would make the checkouts rebuild over each other's workspace crates instead | **Rejected** (implemented, then removed on the coordinator's evidence, 2026-09-28) |

## Outcome

1. **Toolchain.** `rust-toolchain.toml` pins one dated nightly (`nightly-2026-09-29`, rustc
   1.101.0-nightly), with rustfmt, clippy, rust-src, rust-analyzer and llvm-tools. It is the
   only place the date is written. `.config/build.toml` names no toolchain, and the
   solver `dev` image receives the channel as a build argument. The source uses no nightly
   language or library feature.
2. **`rust-version`** is the stable language floor the source is written against (1.98.1).
   `toolchain_matches_msrv` requires a dated nightly channel whose release is at or above
   the floor. Cargo's MSRV-aware resolver, clippy's `incompatible_msrv` and
   `dependency_floors` hold the code and the graph to the floor.
3. **Feature unification.** `.cargo/config.toml` sets `[unstable] feature-unification` and
   `[resolver] feature-unification = "workspace"`. Only requested features —
   `pse-relations/force-validate` and `native-solvers` — change a dependency's build.
4. **Build directories stay per checkout.** Each checkout keeps its intermediates in its
   own `target/`, Cargo's default; no configuration sets `build.build-dir`. Reuse across
   checkouts comes from the shared sccache, whose keys hold because the recipe
   environment no longer exports the default `CARGO_TARGET_DIR` (commit `b3fa3141`).
5. **`pse-workspace-hack`** (cargo-hakari, `.config/hakari.toml`) carries the same
   unification for Cargo that ignores `[unstable]`. The opt-in feature paths — `arrow`
   (for `force_validate`) and the native-solver crates — are excluded from hakari's
   traversal. `pse-ids` and `pse-diagnostics` (the build-dependency closure of
   `pse-buildinfo`) and the generated `pse-operations-queries` are excluded from it. The
   dependency ceilings and the N06 ownership check do not follow its feature-only edge.
   `just codegen` regenerates the crate and `just codegen-check` compares it.
6. **What stands from ADR-0018.** Both families and every external dependency stay
   `=`-pinned once in `[workspace.dependencies]`. `Cargo.lock` stays committed, and every
   recipe runs `--locked`. `cargo deny`, `cargo audit` and `cargo shear` stay in the
   supply-chain checks. `family-check` and `dependency_floors` are unchanged.

### Consequences

- Each checkout still compiles its own copy of every dependency; only sccache hits, not
  Cargo freshness, save work across checkouts.
- The hakari crate widens small members' build closures. Building `pse-model` alone now
  compiles the unified DataFusion and Arrow crates, though it cannot name them.
- Moving the nightly date requires a rebuilt, re-pinned solver `dev` image before
  `just parity-container` or the devcontainer builds.
- Stable Cargo still builds the workspace: it ignores `[unstable]` and warns that it ignores
  `resolver.feature-unification`.
- Nothing compiles the source on the stable floor. Only std-API drift is caught, by
  clippy's `incompatible_msrv`.
- The nightly reports lints that 1.98.1 did not:
  - deprecated `fetch_update`, now `try_update` (stable since 1.95);
  - the `recursion_depth_exceeding_limit` future-incompatibility lint, now
    `#![recursion_limit = "256"]` in five crates;
  - six `unused_qualifications` in `pse-runtime`'s native-solver code, now removed;
  - Cargo's `unused_dependencies`, allowed workspace-wide, because every edge to the
    workspace-hack is a feature-only false positive (cargo-shear stays the detector);
  - a future-incompatibility report for `proc-macro-error2` 2.0.1, reached through the
    vendored delta-rs `validator_derive`, which is not ours to fix.

### Compensating controls

- `workspace_hack_keeps_force_validate_opt_in` fails if hakari switches `force_validate` on.
- A native-solver leak makes `just check` build native C libraries outside the native
  environment.
- `just codegen-hakari-check` and the CI codegen job compare the generated crate.
- AGENTS.md's checkout workflow and `.cargo/config.toml` state why the build directory is
  never shared between checkouts.

### Confirmation

- **Unit-graph counts** of distinct units of symbolica, DataFusion and arrow-array,
  across the seven selections, with no compiling: 7 with stable 1.98.1 before; 4 with
  workspace unification; 4 with unification and the hakari crate; 4 with the hakari crate
  alone under selected unification. faer counts 6, 4, 4 and 5. Four means check and build,
  each with and without `native-solvers`.
- **Feature comparison.** A unit-level comparison against workspace unification alone
  found no `force_validate` or native-solver feature added by the hakari crate, including
  in a release `pse-py` build.
- **Tested, compile only.** On nightly-2026-09-29 with `CARGO_BUILD_JOBS=8`:
  - `just check` and `just check-solver-contracts` exit 0, with no compiler or Cargo
    warning in workspace code (the `proc-macro-error2` report remains);
  - `just codegen-hakari-check` (`cargo hakari generate --diff`,
    `cargo hakari manage-deps --dry-run`) exits 0;
  - `just setup-test` passes 91 tests, 0 failed.

  No Rust test suite was run, and no build was timed or measured.

## Pros and cons

The nightly pin trades a stable channel for a Cargo feature the maintainer wants now. The
date is explicit, and the floor keeps a route back to stable.

A shared build directory would have saved a compile of every dependency per extra
checkout, but it traded that for silent staleness in workspace crates across checkouts on
different branches. Correctness decides it; sccache recovers part of the saving safely.

## More information

- Blueprint §3.1 and §3.2.
- [Build reuse](../dev/build-performance.md).
- `.config/hakari.toml` and `.cargo/config.toml`.
- The [change-tier review](../design_review/reviews/design_review_build-infrastructure_2026-09-29.md).

## Status history

- 2026-09-28 — accepted on the maintainer decisions of 2026-09-28 and the change-tier review (Accept, author review); supersedes ADR-0018, whose pins, lockfile and supply-chain checks stand. Before merge, the shared build directory the maintainer had also decided was withdrawn as unsound (Options; review F09).
