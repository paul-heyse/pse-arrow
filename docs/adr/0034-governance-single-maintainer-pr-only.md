---
id: ADR-0034
title: Govern with a single maintainer, PR-only squash merges onto a linear signed main
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-14, DM-45, DM-60]
blueprint: [§0.1]
review: not-required: a repository-governance decision; neither review raised a finding against it
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A second maintainer joins, which changes the approval count, the bypass policy and the project board question
verification: `governance / pr-title`; the `main` ruleset declared in `.github/setup/*.json` and applied by `just gh-setup` (re-running it is a no-op)

---

# ADR-0034: Govern with a single maintainer, PR-only squash merges onto a linear signed main

## Context

The project has one maintainer, is public, and will accept outside contributions. The rules have to protect history and provenance without requiring a reviewer who does not exist.

## Scope

Binds branch protection, merge strategy, approvals, signing and where the rules are declared. Labels, milestones and issue forms follow from it.

## Drivers

A linear signed history is what makes a release tag mean something; zero approvals is honest for a sole maintainer where pretending otherwise would mean self-approval theatre; declared configuration is reviewable, clicked configuration is not.

## Options

Require one approval — rejected: the sole maintainer would approve their own PRs. Allow direct pushes to `main` — rejected: no check would ever run before a change lands. Merge commits — rejected: squash keeps the Conventional-Commit title as the changelog unit.

## Outcome

Single maintainer; every change arrives as a PR; squash merge only with the PR title as the commit subject; `main` requires linear history and signatures and forbids deletion and non-fast-forward; the ruleset is declared in `.github/setup/*.json` and applied idempotently.

### Consequences

Bypass is restricted to the admin role and every bypass push requires a `governance` follow-up issue. Required checks are staged: a check that never reports would block every PR (§6).

### Compensating controls

`governance / pr-title` enforces the Conventional-Commit types and scopes from `cliff.toml`, which is what `git-cliff` reads to build the changelog.

### Confirmation

`just gh-setup` re-applies the declared configuration; a drift shows up as a diff in the applied JSON rather than as a surprise.

## Pros and cons

Zero approvals is weak; the compensating control is that every check is required and history is protected.

## More information

`GOVERNANCE.md`; blueprint §0.1; `.github/setup/`; ADR-0001's decision-PR rule.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
