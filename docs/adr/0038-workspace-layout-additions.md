---
id: ADR-0038
title: Add pse-ipopt-sys, pse-buildinfo, xtask, benches and five test crates to the workspace layout
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-53, DM-60, DM-48]
blueprint: [§3.2, §24.1, §24.3, §18.3]
review: not-required: the layout additions follow from decisions the reviews already drove (R2-5 pins, F10 probes); no finding names §3.2's crate list
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A sixth test family is needed, or `xtask` grows logic that belongs in a shipped crate
verification: `tests/governance/tests/every_crate_registered.rs` (every workspace member appears in the blueprint §3.2 layout and carries the `pse-` prefix); `rust / test`

---

# ADR-0038: Add pse-ipopt-sys, pse-buildinfo, xtask, benches and five test crates to the workspace layout

## Context

Blueprint §3.2 lists the 23 `pse-*` crates, `python/`, `packages/reference/` and `tests/golden/`. The §24.1 test matrix names six test families and §18.3 requires a `-sys` crate with committed bindgen output, none of which appear in the layout block.

## Scope

Binds the workspace membership. **This record amends blueprint §3.2 rather than deviating from it** — the additions are written into the layout block with a `> Decision: ADR-0038` marker — which is why its level is `decision` and not `should-deviation`; the amendment is carried as revision 4 in the revision history.

## Drivers

A test family that is not a crate cannot be run by `cargo nextest`; governance tests need `cargo_metadata` and so need a crate; `xtask` is how the justfile stays a one-line surface over logic that needs Rust APIs; the Ipopt `links` key must live in its own `-sys` crate.

## Options

Put governance tests in each crate's `tests/` — rejected: they are workspace-wide assertions and would be duplicated 23 times. Make `xtask` a shell script — rejected: `cargo metadata` parsing, JSON and cross-platform behaviour are Rust's job.

## Outcome

The workspace adds `crates/pse-ipopt-sys/` (`links = "ipopt"`, bindgen output committed), `crates/pse-buildinfo/` (lockfile hashes, rustc version, profile, git sha), `xtask/`, `benches/` (`pse-benches`, criterion), and `tests/{governance,engine,conformance,lifecycle,structural}/` as member crates named `pse-tests-<name>`. `tests/golden/` stays data, not a crate. The `ipopt` feature is default-on in `pse-backend-native`, `pse-runtime` and `pse-py`, with `pse-ipopt-sys` optional so `--no-default-features` checks cleanly; phase-0 wheels build without it.

### Consequences

None of the added crates is ever published: `publish = false` stays set for `xtask`, `benches`, `tests/*` and `pse-py`.

### Compensating controls

`every_crate_registered` asserts the workspace membership matches the blueprint layout block, so the two cannot drift.

### Confirmation

`rust / clippy` runs once with default features and once with `--no-default-features`, which is what keeps the optional `-sys` crate honest.

## Pros and cons

Five test crates is more manifests than one; it is also what makes the §24.1 matrix runnable and separately profiled in `.config/nextest.toml`.

## More information

Blueprint §3.2 (workspace layout, revision 4), §24.1 (test layers), §24.3 (benchmarks), §18.3 (Ipopt `-sys` crate); ADR-0028.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
