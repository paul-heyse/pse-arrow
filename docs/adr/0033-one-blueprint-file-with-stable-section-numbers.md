---
id: ADR-0033
title: Keep one blueprint file, revised in git, with section numbers as stable citation targets
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-49, DM-55]
blueprint: [§0.1]
review: not-required: a documentation-authority decision; neither review raised a finding against it
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A revision needs to remove a section entirely, or the file grows past the point where one document is readable
verification: `governance / adr-lint` (every ADR `§` citation must resolve to a heading in `blueprint.md`); `docs / build`

---

# ADR-0033: Keep one blueprint file, revised in git, with section numbers as stable citation targets

## Context

The design existed as `…-blueprint.md` and `…-blueprint-rev3.md` side by side, with reviews citing line numbers into whichever file they were written against. A citation that does not survive the next revision is not a citation.

## Scope

Binds the blueprint's file layout, its revision mechanism and its citation form. The amendment procedure for a decision is ADR-0001's decision-PR rule.

## Drivers

ADRs, reviews, plans and agent instructions all cite the blueprint; git already versions files; section numbers are the only stable handle a reader and a linter can share.

## Options

One file per revision — rejected: every citation rots at the next revision. A docs site with anchors only — rejected: anchors change with titles; numbers do not.

## Outcome

One `docs/authoritative_design/blueprint.md`, revised in git with tags `design-rev2` and `design-rev3` marking the seeded history. Section numbers are stable citation targets: new material is inserted as a sub-section, never by renumbering. §0.1 states that the file is authoritative, that ADRs record why, and that reviews are evidence, not authority.

### Consequences

A section that becomes wrong is rewritten in place with a revision-history row and a `> Decision: ADR-NNNN` line, not deleted and renumbered.

### Compensating controls

`scripts/adr.py lint` resolves every `§` citation against the file's headings, so a renumbering breaks the build rather than silently rotting the records.

### Confirmation

The revision-history table gained a `git` column so each revision names its tag; `docs / build` publishes the file with the ADR index beside it.

## Pros and cons

Stable numbers constrain how the document can be reorganised; rotating citations constrain everything that cites it.

## More information

Blueprint §0.1 (what this document is), revision history; `docs/authoritative_design/README.md`.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
