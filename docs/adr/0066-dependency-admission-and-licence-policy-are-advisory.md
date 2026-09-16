---
id: ADR-0066
title: Make dependency admission and licence policy advisory during phases 0-1
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-56, DM-58, DM-59]
blueprint: [§3.1, §3.3, §3.3.2, §24.1]
review: docs/design_review/reviews/design_review_dependency-policy_2026-09-14.md#7-principle-findings
evidence: Implemented
supersedes: []
superseded-by: null
revisit: The first crate flips to `publish = true`, or the first PyPI release — register row R-31
verification: `tests/governance/tests/pins_match_blueprint.rs`; `rust / family-check`; `just deps-report` and `just policy`; `rust / deny` present and `continue-on-error`

---

# ADR-0066: Make dependency admission and licence policy advisory during phases 0-1

## Context

The repository treated its third-party dependency set as a closed list. Every
`[workspace.dependencies]` entry had to appear in blueprint §3.1 or in
`tests/governance/tooling_deps.toml` or `pins_match_blueprint` failed; `deny.toml` banned
ten named crates, allowed fourteen SPDX licences ("copyleft is absent by omission:
anything not listed fails"), forbade git sources, and denied duplicate versions behind
nineteen dated `skip` entries; six documents repeated verbatim that adding a dependency
needs an ADR **and** a design review; `rust / deny` was a required check. Adding a
library therefore meant amending an off-limits file (§3.1) through a `design:` PR.

ADR-0065 made this argument for the Arrow/DataFusion surface — *"Keep dependency
selection proportional to consumers, without permanent bans"* — and slated six of the
`deny.toml` bans for removal. The maintainer's direction generalizes it: the priority is a
functional codebase, and any library that serves that should be usable now. Substitution
is a decision to make once a library's role is characterized and the behaviour it supports
is testable, not a constraint to design around in advance.

## Scope

Binds **admission**: which third-party libraries may enter the graph, and under which
licences. It does **not** touch reproducibility — `=` and `==` pinning, committed
lockfiles and `--locked` gates are unchanged — and it does not touch the one type universe.

Following ADR-0065's precedent, the accepted records below are retained unchanged; this
record amends the *enforcement clause* each one relies on, and names them so no
verification claim is left dangling:

- **ADR-0018** — its Compensating controls cite `deny.toml`'s ban list and `rust / deny`
  as a gate. The pinning, lockfile and MSRV decisions stand; the gate becomes a report.
- **ADR-0021** — `serde-saphyr` remains the YAML loader. `serde_yaml` is no longer
  mechanically banned; the reason it is not used is recorded in prose.
- **ADR-0026** — the units decision stands (`pint` validates, never defines). The `uom`
  and `arrow-flight` bans are lifted; ADR-0065 already revised the Arrow Flight half.
- **ADR-0020, ADR-0037** — the `salsa`, `datafusion-ffi`, `egglog` and observability
  deferrals stay as register rows with triggers. Their `deny.toml` bans are lifted: a
  deferral is a review prompt, not an admission gate.

## Drivers

Adding a library required amending an off-limits design document. The licence allowlist
refused by omission while the project publishes nothing. `multiple-versions = "deny"`
blocked ordinary additions over hashers and proc-macro plumbing while the hazard it was
named for — a mixed arrow/datafusion family — is asserted directly and more precisely by
`family-check`. Machinery should be scaled to demonstrated need (DM-58).

## Options

1. Keep the closed list and add rows as needed — rejected: every library becomes a
   `design:` PR against the blueprint, which is the cost this decision exists to remove.
2. Delete the machinery — rejected: the licensing question is deferred, not answered, and
   deleting the tooling makes answering it a rebuild instead of a command.
3. Keep every mechanism installed and make it report instead of gate — selected.

## Outcome

No third-party library is refused and no licence is grounds to refuse one. Adding a
dependency needs no ADR, no design review and no blueprint row.

`deny.toml` stays, as the configuration of a **report**: `deny = []`, `multiple-versions =
"allow"` with the `skip` list removed, `unknown-git = "allow"`, advisories widened to
`unmaintained = "all"` and `yanked = "warn"`, and a licence list extended through the
copyleft families and reframed as a record of what a future review has already seen.
`rust / deny` runs on every pull request with `continue-on-error: true` and is off the
required-checks ruleset; `just ci-pr` no longer calls `just policy`. `just deps-report`
is the advisory run, `just policy` the strict opt-in audit.

`pins_match_blueprint` keeps failing on **drift** — a crate §3.1 names must declare the
version §3.1 says — and merely reports a dependency §3.1 does not mention. Blueprint
§3.1 is the pin authority, not an admission list; new §3.3.2 states the policy, and
§3.3's "Explicitly not added" paragraph becomes "Not currently used, and why".
import-linter's dependency-hygiene contract narrows to `idaes` alone, which is a
packaging constraint (uv workspaces enforce a single `requires-python`), not hygiene.

### Consequences

The cost is real and deliberately accepted: a copyleft dependency linked into a
distributed artifact can affect what `MIT OR Apache-2.0` means for what we ship, and
nothing now catches that automatically. Every crate is `publish = false` and there is no
PyPI release, so nothing is distributed today. Register row **R-31** carries the
obligation with the trigger that makes it answerable — the first `publish = true` crate or
the first PyPI release — and `just policy` as its check.

Three of the lifted bans encoded a semantic trap rather than hygiene:
`datafusion-spark` (silently different arithmetic), `parquet-variant` (a typed hole in
D1), `parquet_derive` (inverts the registry → Rust direction). Their reasoning survives in
prose, in `deny.toml` and in `docs/dev/dependency-policy.md`, where it informs rather than
refuses.

### Compensating controls

`cargo xtask family-check` remains a required check and is the load-bearing one: exactly
one resolved `arrow`, `parquet`, `object_store`, `datafusion` and `pyo3`, cross-checked
against the capability-map evidence lockfiles. Pin drift against §3.1 still fails.
`=`/`==` pins, committed `Cargo.lock` and `uv.lock`, and `--locked` on every gate are
unchanged, so a permissive admission policy costs nothing in reproducibility. The
numpy/scipy, pyomo/pint and `pse.contracts` import boundaries are untouched — they are
architecture. The clean-room rule is untouched: not copying from an
incompatibly-licensed source is a question about copying code, not about depending on a
library.

### Confirmation

`just deps-report` and `just policy` both run against the full graph; `cargo deny
--all-features --locked check` reports `advisories ok, bans ok, licenses ok, sources ok`
on the current lockfile. `tests/governance/tests/pins_match_blueprint.rs` passes with the
drift arm intact and the undescribed-dependency arm demoted to a note.

## Pros and cons

Development stops paying a design-document tax for ordinary library use, and the project
can reach for what it needs while its shape is still being established. The cost is that
the licensing and supply-chain question is deferred rather than answered, which is only
acceptable because nothing is published and because every mechanism that can answer it is
still installed and runnable.

## More information

- [Dependency and licence policy](../dev/dependency-policy.md) — the operational statement
- [ADR-0065](0065-full-arrow-datafusion-capability-access.md) — the same argument, scoped
  to the Arrow/DataFusion surface
- Blueprint §3.1, §3.3, §3.3.2, §24.1; register row R-31

## Status history

- 2026-09-14 — proposed after explicit maintainer direction to remove library and licence
  constraints while keeping the infrastructure for a manual check.
