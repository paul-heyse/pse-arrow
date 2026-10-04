---
id: ADR-0159
title: Dependencies float; exact pins need a recorded reason
status: proposed
date: 2026-10-04
deciders: [paul-heyse]
level: decision
principles: [DP-14, DP-15]
blueprint: [§3.1, §3.3.2]
review: not-required: proposed record; the governance-tier design review that acceptance requires has not run (maintainer policy decision of 2026-10-04, Status history)
evidence: Implemented
supersedes: [ADR-0122]
superseded-by: null
revisit: A version that floated under a caret or `>=` floor breaks a build, a test or an identity contract (the dependency then gets a recorded pin); or the first crate flips to `publish = true`, or the first PyPI release (register row R-31); or Cargo stabilizes `feature-unification` (carried from ADR-0122).
verification: Governance test `dependency_pins` (an exact `=` requirement or git revision is a declared family member or has a `[workspace.metadata.pse.pins]` reason; its unit cases cover a caret, an unlisted exact pin, a listed exact pin and a family member); `just family-check`; `cargo metadata --locked` and `uv lock --check` on the converted manifests with no resolved version moved; and from ADR-0122, `toolchain_matches_msrv`, `dependency_floors` and `just codegen-hakari-check`.
standard: core-3.3 / process-simulator-1.4
scenarios: []

---

# ADR-0159: Dependencies float; exact pins need a recorded reason

## Context

ADR-0122, restating ADR-0018, kept every external dependency `=`-pinned once in
`[workspace.dependencies]`, and blueprint §3.1 and §3.3.2 required `==` pins for runtime
Python libraries. `dependency_pins` failed on any requirement that was not exact. The
committed lockfiles and `--locked` gates already make every run reproducible, so the exact
specifiers bought no reproducibility; they made each upgrade a manifest edit and hid which
pins exist for a reason. The maintainer decided on 2026-10-04 that dependencies float by
default and that a pin exists only where a reason specific to that dependency is recorded
beside it.

## Scope

This record supersedes ADR-0122 for its exact-pin clause (its Outcome item 6, "every external
dependency stays `=`-pinned"). It amends blueprint §3.1 (version authority) and §3.3.2
(the rules that admission does not relax). It also amends the compensating-control clause of
ADR-0066 that names `=`/`==` pinning as unchanged; ADR-0066 itself is retained, following its
own precedent for ADR-0018.

Carried forward unchanged, so that no decision lapses:

- **Admission stays relaxed** (ADR-0066): no library and no licence is refused through
  phases 0-1, and adding a dependency needs no ADR, review or documentation row.
- **The dated nightly and feature unification stay** (ADR-0122 Outcome items 1-5):
  `rust-toolchain.toml` pins one dated nightly as its only declaration, `rust-version` is the
  stable floor at or below it, `.cargo/config.toml` enables workspace feature unification,
  `pse-workspace-hack` carries it for Cargo that ignores `[unstable]`, and each checkout keeps
  its own `target/`. ADR-0122's revisit triggers carry over.
- **The lockfiles and `--locked` stay.** `Cargo.lock` and `uv.lock` are committed and every
  recipe runs `--locked` or `--frozen`; `cargo deny`, `cargo audit` and `cargo shear` stay in
  the supply-chain checks.
- **Families stay.** Arrow/Parquet, DataFusion, `object_store` and PyO3 (minor match) resolve to
  one version each, declared in `[workspace.metadata.pse.families]` and checked by
  `family-check`; their members stay exact. A major move of one of the four families still
  needs an ADR and a design review.
- **No `[patch]` or `[replace]` tables** (`no_patch_tables`), the vendored Delta revisions
  (`delta_revisions`) and `dependency_floors` are unchanged.

Out of scope: interpreter and toolchain versions, CI action SHA pins, content-addressed
acquisitions (solver image digests, archive sha256s) and the IDAES parity pin (ADR-0097),
which moves by its own short ADR.

## Drivers

- Reproducibility belongs to the lockfiles (DP-15's "one resolved version of each
  type-sharing family" is a resolution property, enforced by `family-check`).
- A pin should say why it exists, so that the next upgrade knows what to re-check (DP-14:
  use what the resolved version provides).
- Upgrades are at the agent's discretion through `just upgrade`, with the affected tests.

## Options

| Option | Effect | Verdict |
|---|---|---|
| Keep every dependency exact | Reproducible, but the lock already gives that; every upgrade is a manifest edit and reasoned pins are indistinguishable from default ones | Rejected |
| Carets everywhere, families included | Simplest manifest; a family member could move within its caret range ahead of its family and needs `family-check` to notice | Rejected |
| **Carets and `>=` floors by default; exact `=`/`==` or a git revision only for a family member or a recorded reason** | Upgrades move the lock, not the manifest; each remaining pin carries its reason in `Cargo.toml` or `pyproject.toml` | **Selected** |

## Outcome

Third-party Rust dependencies in `[workspace.dependencies]` use caret requirements (the
`cargo add` default) and runtime Python libraries use `>=` floors (the `uv add` default). An
exact `=` requirement, an `==` pin, an upper cap or a git revision is allowed only for:

- a member of a family in `[workspace.metadata.pse.families]`; or
- a dependency listed in `[workspace.metadata.pse.pins]` (crate name → one-line reason), or,
  for Python, a dependency whose `pyproject.toml` comment records its reason.

Reasons that count: a named breakage or incompatibility; a type-sharing family; a fork, git
revision or vendored source; committed or byte-stable generated output, or an unstable-API
use; a parity or reference oracle. Reproducibility, "already in the lock" and a version
entering a key or digest are not reasons. `just upgrade` moves the lockfiles to the latest
versions the manifests allow (`just upgrade <package>` moves one); there is no cadence.

### Consequences

- At adoption every converted requirement uses its locked version as the caret base or floor,
  so no resolved version moved (`Cargo.lock` unchanged; `uv.lock` changes only in requirement
  metadata).
- Kept pins at adoption: the family members; the POUNCE and FERAL git revisions; the vendored
  `deltalake`; `syn`, `prettyplease`, `bindgen`, `cornucopia` and `schemars` (committed
  generated bytes); `validator` (copied into the vendored Delta manifest); `num-dual` (the
  FeOS trait universe); `blake3` (the semantic identity hash); `salsa` (`salsa_unstable` API).
  Python keeps the parity group and the test oracles exact.
- A dependency bump can now arrive through `just upgrade` without a manifest diff; the tests
  the move affects are the check.

### Compensating controls

`family-check` holds the one type universe whatever the specifiers say. `dependency_pins`
refuses an exact requirement or git revision that is neither a family member nor listed with
a reason, so a pin cannot be added silently. `dependency_floors` holds every resolved package
to the `rust-version` floor. The `--locked` gates keep every run on the committed lock.

### Confirmation

Implemented: `tests/governance/tests/dependency_pins.rs` inverts the check and covers the four
cases in its unit test; `cargo metadata --locked` and `uv lock --check` pass on the converted
manifests. Architectural acceptance needs the governance-tier design review.

## Pros and cons

The lockfile, not the manifest, is now where a version is read; the capability maps and
`library_utilization.py` already read resolved versions from `Cargo.lock`.

## More information

- Blueprint §3.1 and §3.3.2; [dependency policy](../dev/dependency-policy.md).
- [ADR-0066](0066-dependency-admission-and-licence-policy-are-advisory.md) (admission,
  retained) and [ADR-0122](0122-nightly-toolchain-and-feature-unification.md) (toolchain and
  feature unification, superseded here for the exact-pin clause and restated above).
- `Cargo.toml` `[workspace.metadata.pse.pins]` and `[workspace.metadata.pse.families]`.

## Status history

- 2026-10-04 — proposed on the maintainer's dependency-policy decision of 2026-10-04 ("full
  shift"); implemented in the same change. Acceptance waits for the governance-tier design
  review (`needs-review`). Supersedes ADR-0122 for its exact-pin clause (`just adr-supersede`).
