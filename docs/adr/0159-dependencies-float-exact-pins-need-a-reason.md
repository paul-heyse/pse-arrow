---
id: ADR-0159
title: Dependencies float; exact pins need a recorded reason
status: superseded
date: 2026-10-04
deciders: [paul-heyse]
level: decision
principles: [DP-14, DP-15]
blueprint: [§3.1, §3.3.2]
review: docs/design_review/reviews/design_review_dependency-float-policy_2026-10-04.md#9-decision
evidence: Implemented
supersedes: []
superseded-by: ADR-0165
revisit: A version that floated under a caret or `>=` floor breaks a build, a test or an identity contract (the dependency then gets a recorded pin); a `==` pin or Python cap is added without a reason, which would end the instruction-only route for Python reasons; or the first crate flips to `publish = true`, or the first PyPI release (register row R-31).
verification: Governance test `dependency_pins` (a requirement that bounds a version from above — `=`, `<`, `<=`, `~`, a wildcard — or a git revision is a declared family member or has a `[workspace.metadata.pse.pins]` reason; family members are exact at the family version; stale reasons fail; its unit cases cover a caret, a floor, unlisted exact, capped, tilde, wildcard and git pins, listed pins, exact and caret family members); `just family-check`; `just upgrade` regenerates the workspace-hack and runs `family-check`; `cargo metadata --locked` and `uv lock --check` on the converted manifests with no resolved version moved.
standard: core-3.3 / process-simulator-1.4
scenarios: []

---

# ADR-0159: Dependencies float; exact pins need a recorded reason

## Context

ADR-0122, restating ADR-0018, kept every external dependency `=`-pinned once in
`[workspace.dependencies]` (its Outcome item 6), and blueprint §3.1 and §3.3.2 required `==`
pins for runtime Python libraries. `dependency_pins` failed on any requirement that was not
exact. The committed lockfiles and `--locked` gates already make every run reproducible, so
the exact specifiers bought no reproducibility; they made each upgrade a manifest edit and hid
which pins exist for a reason. The maintainer decided on 2026-10-04 that dependencies float by
default and that a pin exists only where a reason specific to that dependency is recorded
beside it.

## Scope

This record **amends** ADR-0122's exact-pin clause (Outcome item 6) and retains ADR-0122,
which still owns the dated nightly, the `rust-version` floor, workspace feature unification,
the `pse-workspace-hack` (with its traversal exclusions and
`workspace_hack_keeps_force_validate_opt_in` control), per-checkout `target/`, and its four
revisit triggers. It likewise amends the clause of ADR-0066 that names `=`/`==` pinning as
unchanged, following ADR-0066's own precedent for ADR-0018; ADR-0066 still owns admission. Both
records carry a dated Status-history pointer here. It amends blueprint §3.1 (version
authority) and §3.3.2 (the rules that admission does not relax).

Unchanged and owned elsewhere: relaxed admission (ADR-0066); the toolchain and feature
unification (ADR-0122); committed `Cargo.lock` and `uv.lock` with every recipe `--locked` or
`--frozen`; `cargo deny`, `cargo audit` and `cargo shear`; the families in
`[workspace.metadata.pse.families]` with `family-check`, and an ADR for a major move of one of
the four families; `no_patch_tables`, `delta_revisions` and `dependency_floors`; Dependabot's
family groups (ADR-0035), now `versioning-strategy: lockfile-only`.

It also revises the wording of the core design standard's DP-14 and DP-15 from "pin" to
"qualify at the resolved version", with a dated note in the standard; the obligations are
unchanged.

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
| Carets everywhere, families included | Simplest manifest; `just upgrade` would move families and fail `family-check` on every upstream patch | Rejected |
| **Carets and `>=` floors by default; family members exact; other pins only with a recorded reason** | Upgrades move the lock, not the manifest; each remaining pin carries its reason in `Cargo.toml` or `pyproject.toml` | **Selected** |

## Outcome

Third-party Rust dependencies in `[workspace.dependencies]` use caret requirements (the
`cargo add` default) and runtime Python libraries use `>=` floors (the `uv add` default). A
requirement that bounds a version from above — an exact `=`/`==`, `<`, `<=`, `~` or a
wildcard — or a git revision is allowed only for:

- a member of a family in `[workspace.metadata.pse.families]`, which must be exact at the
  family's declared version; or
- a dependency listed in `[workspace.metadata.pse.pins]` (crate name → one-line reason), or,
  for Python, a dependency whose `pyproject.toml` comment records its reason.

The acceptable reasons are the one list in
[the dependency policy](../dev/dependency-policy.md#pin-reasons). `just upgrade` moves the
lockfiles to the latest compatible versions (`just upgrade <package>` moves one), regenerates
the workspace-hack and runs `just family-check`; there is no cadence. A caret never crosses a
major: crossing one is an explicit requirement bump in the manifest.

### Consequences

- At adoption every converted requirement uses its locked version as the caret base or floor,
  so no resolved version moved (`Cargo.lock` unchanged; `uv.lock` changes only in requirement
  metadata).
- Kept pins at adoption: the family members; the POUNCE and FERAL git revisions; the vendored
  `deltalake` and `validator` (copied into the vendored Delta manifest); `syn`, `prettyplease`,
  `bindgen`, `cornucopia` and `schemars` (committed generated output); the FeOS reference
  oracle closure `feos`, `feos-core`, `quantity` and `num-dual` (it generates the committed
  `feos-0.10.1` oracle package and is compared live by conformance); `salsa`
  (`<dyn salsa::Database>::memory_usage`, gated by `salsa_unstable`). `blake3` floats: the
  golden vectors in `pse-ids` freeze the identity digests. Python keeps the parity group, the
  numpy/scipy test oracles and the teqp reference exact; `maturin`'s build-system cap is
  dropped.
- A dependency bump can now arrive through `just upgrade` without a manifest diff; the tests
  the move affects are the check, and `BUILD_IDENTITY` already digests both lockfiles.

### Compensating controls

`family-check` holds the one type universe whatever the specifiers say, and `just upgrade`
runs it after every move. `dependency_pins` refuses a Rust requirement that bounds a version
from above, or a git revision, unless it is an exact family member or listed with a reason, so
a Rust pin cannot be added silently. Python pin reasons are instruction-only: a scoped
deviation from §H's checkable-claim norm, with the revisit trigger above. `dependency_floors`
holds every resolved package to the `rust-version` floor. The `--locked` gates keep every run
on the committed lock.

### Confirmation

Implemented: `tests/governance/tests/dependency_pins.rs` and its unit cases; `cargo metadata
--locked` and `uv lock --check` pass on the converted manifests; `just governance` passes.
Architectural acceptance: the governance-tier design review, verdict Accept-scoped, findings
F01–F07 resolved in commit `b8ddd5412`.

## Pros and cons

The lockfile, not the manifest, is now where a version is read. The capability maps describe
the versions of their own evidence lockfiles, which `family-check` compares with the graph for
family packages only; they remain evidence about their extraction versions, and DP-15
qualifies behaviour at the resolved version.

## More information

- Blueprint §3.1 and §3.3.2; [dependency policy](../dev/dependency-policy.md).
- [ADR-0066](0066-dependency-admission-and-licence-policy-are-advisory.md) (admission) and
  [ADR-0122](0122-nightly-toolchain-and-feature-unification.md) (toolchain and feature
  unification), each amended here for one clause.
- `Cargo.toml` `[workspace.metadata.pse.pins]` and `[workspace.metadata.pse.families]`.
- [Governance-tier review](../design_review/reviews/design_review_dependency-float-policy_2026-10-04.md).

## Status history

- 2026-10-04 — proposed on the maintainer's dependency-policy decision of 2026-10-04 ("full
  shift"); implemented in the same change. Acceptance waits for the governance-tier design
  review (`needs-review`). Initially recorded as superseding ADR-0122; changed before
  acceptance to amend ADR-0122 and ADR-0066 and retain both (review F01).
- 2026-10-04 — accepted on the governance-tier design review
  (`design_review_dependency-float-policy_2026-10-04.md`, verdict Accept-scoped); findings
  F01–F07 resolved in commit `b8ddd5412`.
- 2026-10-06 — superseded by ADR-0165.
