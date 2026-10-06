---
id: ADR-0165
title: Declared dependencies are pinned exactly
status: accepted
date: 2026-10-06
deciders: [paul-heyse]
level: decision
principles: [DP-14, DP-15]
blueprint: [§3.1, §3.3.2]
review: not-required: operator decision of 2026-10-06 to restore exact pins without ceremony; reverses ADR-0159's policy and keeps its controls
evidence: Implemented
supersedes: [ADR-0159]
superseded-by: null
revisit: Exact pins themselves cause the disruption they were restored to prevent (for example, repeated failed bumps from conflicting exact requirements), or the first crate flips to `publish = true` or the first PyPI release (register row R-31), where library requirements must be ranges.
verification: Governance test `dependency_pins` (every registry requirement in `[workspace.dependencies]` is a single exact `=x.y.z`; a git source names a full commit; path dependencies are exempt; family members are exact at the family version; each `[workspace.metadata.pse.pins]` hold has a reason and names a declared, pinned dependency; its unit cases cover caret, floor, range, cap, tilde and wildcard failures and an exact pin with no hold passing); `cargo metadata --locked` with `Cargo.lock` unchanged and `uv lock --check` with no `version =` change on the converted manifests.
standard: core-3.3 / process-simulator-1.4
scenarios: []

---

# ADR-0165: Declared dependencies are pinned exactly

## Context

ADR-0159 let dependencies float under carets and `>=` floors, with `just upgrade` moving the
lockfiles at an agent's discretion. In practice versions shifted under agents, environments
were disrupted, and agents were surprised by versions they had not chosen. The operator
decided on 2026-10-06 to pin every declared dependency again, without ceremony.

## Scope

Supersedes ADR-0159 and amends blueprint §3.1 (version authority) and §3.3.2 (admission
specifier, locked resolution). Unchanged: relaxed admission (ADR-0066), the toolchain and
feature unification (ADR-0122), the families and `family-check`, an ADR for a family major,
`--locked` gates, `dependency_floors`, and Dependabot's family groups with
`versioning-strategy: lockfile-only`. Interpreter and toolchain pins are out of scope.

## Drivers

- An environment should change only when someone changes it on purpose.
- Changing a version stays ordinary agent judgment; no ADR, approval or vetting.

## Options

| Option | Verdict |
|---|---|
| Keep floating (ADR-0159) | Rejected: the disruption and surprise above |
| **Pin every declared dependency exactly; the lockfiles hold the rest; bumps are judgment** | **Selected** |
| Also pin transitive dependencies in manifests | Rejected: the committed lockfiles already hold them |

## Outcome

Every registry dependency in `[workspace.dependencies]` is `=x.y.z` (`cargo add
name@=x.y.z`); a git dependency is pinned by its full `rev`; every declared Python dependency
is `==x.y.z`, and `[tool.uv] add-bounds = "exact"` makes `uv add` write it. An agent adds or
bumps a dependency when the work calls for it: edit that pin (or its family), run
`just upgrade <package>` (moves it, regenerates the workspace-hack, runs `family-check`),
check the lock diff moved only what was meant, run the affected tests and name the move in
the commit. No wholesale re-resolve (`uv lock --upgrade`, bare `cargo update`) unless the
operator asks; `just upgrade` has no whole-lock form. `[workspace.metadata.pse.pins]` (and,
in Python, a comment beside the pin) records holds: versions that must not be bumped
casually, each with its reason.

### Consequences

- At adoption every caret and floor was frozen at its locked version; `Cargo.lock` is
  unchanged and `uv.lock` changed only in requirement metadata.
- The existing pins entries remain as holds.

### Compensating controls

`dependency_pins` refuses any non-exact registry requirement and any stale hold;
`family-check` keeps the one type universe.

### Confirmation

Implemented in the same change: `tests/governance/tests/dependency_pins.rs` and its unit
cases pass; `cargo metadata --locked` and `uv lock --check` pass on the converted manifests.

## Pros and cons

The manifest again states the version in use. The cost is that a bump is a manifest edit,
which is the point.

## More information

- Blueprint §3.1 and §3.3.2; [dependency policy](../dev/dependency-policy.md).
- [ADR-0159](0159-dependencies-float-exact-pins-need-a-reason.md) (superseded).

## Status history

- 2026-10-06 — accepted on the operator's decision to pin declared dependencies again;
  supersedes ADR-0159; implemented in the same change.
