---
id: ADR-0018
title: Pin both families with =, commit the lockfile, run cargo deny, and set MSRV to the pinned toolchain
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-48, DM-51, DM-59]
blueprint: [§3.1, §3.2, §3.3]
review: git:8950dd3d6ddb:docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A DataFusion or Arrow release requires a Rust version above the pinned toolchain, or a pre-1.0 crate raises its declared floor above ours
verification: `tests/governance/tests/pins_match_blueprint.rs`, `tests/governance/tests/toolchain_matches_msrv.rs`, `tests/governance/tests/dependency_floors.rs`; `rust / family-check`; `rust / deny`

---

# ADR-0018: Pin both families with =, commit the lockfile, run cargo deny, and set MSRV to the pinned toolchain

## Context

Blueprint §3.1 pins every crate in both families with `=`, requires a committed `Cargo.lock` and `cargo deny`, and states MSRV as 1.94.0 — DataFusion 55's own floor. The second review's R2-5 found 23 supporting crates unpinned; a floor the project never compiles against is a claim nothing checks.

## Scope

Binds the pinning rule, the lockfile, the supply-chain gates and the MSRV. **This record amends blueprint §3.1's MSRV row rather than deviating from it**, which is why its level is `decision` and not `should-deviation`; the amendment is carried in the blueprint's revision history as revision 4.

## Drivers

`=` pins bind direct dependencies only and `cargo tree -d` cannot see a mixed family; a deprecation encoded in semver build metadata is invisible to `cargo audit`; an MSRV nothing exercises is not a supported configuration.

## Options

Caret pins with a lockfile — rejected: a family can go mixed without a duplicate. MSRV 1.94.0 as DataFusion's floor — rejected as the project's MSRV: CI never builds it, so it would be an unverified claim; it stays in the blueprint as DataFusion's floor. A floating toolchain — rejected: nightly is this machine's default.

## Outcome

Both families are `=`-pinned family-wide with features fixed once in `[workspace.dependencies]`; `Cargo.lock` is committed; `cargo deny` and `cargo audit` and `cargo shear` run in `rust / deny`. `rust-version` equals the pinned stable toolchain — **1.98.1** at revision 4 — and `rust-toolchain.toml` is the single source of both.

### Consequences

Every dependency move is a deliberate `uv lock --upgrade-package` or a Dependabot PR in a family group (ADR-0035); consumers of a future published crate inherit a high MSRV, which is acceptable while the project is pre-1.0.

### Compensating controls

`cargo xtask family-check` asserts one resolved version per family and cross-checks the capability-map evidence lockfiles; `dependency_floors` asserts every resolved package's declared `rust_version` is at or below ours; `deny.toml` bans `serde_yaml`, `uom`, `arrow-flight` and the rest of the §3.1/§3.3 list with a reason each.

### Confirmation

`rust / family-check` and `rust / deny` are required checks; `toolchain_matches_msrv` fails if `rust-version` and `rust-toolchain.toml` diverge.

## Pros and cons

Pinning family-wide costs a wider Dependabot blast radius per upgrade; the alternative is the mixed-family failure mode that neither `cargo tree -d` nor `cargo deny` reports.

## More information

Blueprint §3.1 (version anchors, revision 4 MSRV row), §3.3 (library boundaries); review finding R2-5; `docs/capability-maps/evidence/rust/` lockfiles.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
