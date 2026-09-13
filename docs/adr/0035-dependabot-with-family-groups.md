---
id: ADR-0035
title: Update dependencies with Dependabot, grouped by the pinned families
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-51, DM-48]
blueprint: [§3.1]
review: not-required: a supply-chain process decision; neither review raised a finding against it
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Dependabot cannot express a grouping the family rule needs, in which case Renovate is the declared fallback (register row R-13)
verification: `rust / family-check` on every Dependabot PR; `rust / deny`; `governance / pr-title`

---

# ADR-0035: Update dependencies with Dependabot, grouped by the pinned families

## Context

ADR-0018 pins both families `=` family-wide, so an upgrade is never a single-crate bump: every `arrow*` and `parquet*` crate must move together, and so must every `datafusion*` crate plus `object_store`.

## Scope

Binds the update mechanism and its grouping. Which upgrades are accepted is a per-PR decision with `family-check` as the gate.

## Drivers

Ungrouped updates would open a dozen PRs that each fail `family-check`; a grouped PR is reviewable as one decision; `github-actions` pins by commit SHA and needs its own cadence.

## Options

Renovate — kept as the declared fallback rather than the default: Dependabot needs no extra app installation on a single-maintainer public repository. Manual upgrades — rejected: nothing would notice a yanked crate.

## Outcome

Cargo monthly with `versioning-strategy: increase` and groups `arrow-family` (`arrow*`, `parquet*`), `datafusion-family` (`datafusion*`, `object_store`), `pyo3-family` (`pyo3*`, `numpy`), `codegen` (`syn`, `quote`, `proc-macro2`, `prettyplease`) and `numerics`. uv monthly with `quality-tools` and `runtime` groups and `idaes-pse` ignored. GitHub Actions weekly, grouped. Docker for the solver base image.

### Consequences

A `codegen` group PR must carry a regeneration commit (ADR-0031); the `parity` pin never moves by bot, because moving it is an ADR (ADR-0003).

### Compensating controls

`rust / family-check` fails a PR that moves one crate of a family; `pinact` and Dependabot together keep every action pinned by SHA with a version comment.

### Confirmation

Every bot PR runs the full required-check set before it can merge; `cargo deny` catches a newly yanked or newly advisory-carrying version.

## Pros and cons

Monthly cadence trades freshness for review load; the weekly actions cadence exists because action SHAs move for security reasons.

## More information

Blueprint §3.1 (pins and families); `.github/dependabot.yml`; ADR-0018; register row R-13.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
