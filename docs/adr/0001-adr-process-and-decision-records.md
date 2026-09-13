---
id: ADR-0001
title: Record decisions as ADRs with the charter §H fields
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-55, DM-59, DM-60]
blueprint: [§0.1]
review: not-required: this record establishes the review process; it governs no design surface
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A second maintainer joins, or two consecutive months pass in which ADRs are written after the change rather than before it
verification: `governance / adr-lint` runs `scripts/adr.py lint` and `scripts/check_register.py --lint` on every PR

---

# ADR-0001: Record decisions as ADRs with the charter §H fields

## Context

The blueprint states decisions but not why they beat the alternatives, and the two design reviews are evidence that expires. Charter §H asks a deviation to record principles, scope, reason, alternatives, consequences, compensating controls, evidence, owner and a revisit trigger; nothing in the repository held those fields.

## Scope

Binds where decisions live (`docs/adr/`), what a record must contain, and when one is required (the table in `docs/plans/01-repository-configuration.md` §7). Plans (`docs/plans/`) are a separate, living lifecycle and are not covered here.

## Drivers

Decisions must outlive the session that took them; the charter's §H fields must be mechanically checkable; the cost of the process must stay under the cost of the risk (charter §H: "a short, concrete decision record is sufficient").

## Options

`adr-tools` — rejected: unmaintained shell, no §H fields. A `DECISIONS.md` append-only log — rejected: no per-decision status, no supersession, no lint. Issue labels only — rejected: not in the repository, not reviewable in a PR.

## Outcome

MADR-4 body sections plus the charter §H fields in YAML front matter, one file per decision under `docs/adr/NNNN-kebab.md`, linted by `scripts/adr.py`. A record is immutable once accepted except `status`, `superseded-by` and an appended `## Status history` line; it changes by being superseded.

### Consequences

Every decision that alters D1–D14, a crate, a dependency family, a hashing or boundary contract, or governance now needs a PR labelled `adr` titled `adr: ADR-NNNN <title>`. The blueprint gains `> Decision: ADR-NNNN` one-liners at the governed sections.

### Compensating controls

`scripts/adr.py lint` checks the filename, contiguous numbering, required keys, the enums, `DM-NN` principle IDs, that every `§` citation resolves to a real blueprint heading, that the review path exists, that supersession is symmetric, and that accepted records are unchanged against `origin/main`.

### Confirmation

`governance / adr-lint` is a required check on `main`; `docs / build` runs `adr.py index --check` so the generated index cannot drift.

## Pros and cons

The cost is one short file per decision and one required check. The alternative — decisions living in commit messages and chat — is what produced the two design reviews' "unstated dependency" findings.

## More information

Charter §H (scoped exception record) and §D (evidence vocabulary) in `docs/design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md`; `docs/adr/template.md`; `.codex/skills/adr/SKILL.md`.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
