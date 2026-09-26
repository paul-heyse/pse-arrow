---
id: ADR-0096
title: Keep current rationale and retire obsolete documentation history
status: accepted
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-06, DP-01, DP-22, DP-24]
blueprint: [§0.1, §0.3, §24.4]
review: docs/design_review/reviews/design_review_documentation-consolidation_2026-09-25.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A retained reader, tool or decision needs content that exists only in Git history, or a retirement removes a still-consumed input
verification: DecisionRecordTests in scripts/tests/test_setup.py (sparse IDs, retired peers, Git references, same-file relocation, append-only status history, history-based allocation, register high-water mark) via just setup-test; just adr-lint; just docs with offline fragment checks
---

# ADR-0096: Keep current rationale and retire obsolete documentation history

## Context

The working corpus held about 80,000 lines of plans, reviews, packets, superseded
decisions and a blueprint that mixed current native contracts with retired MathIR,
pass-pipeline, NL/Pyomo and phase text. Agents had to read precedence notes and old
decisions to find current meaning. The ADR tool required contiguous numbering and local
review files, so an obsolete record could not leave the working tree.

## Scope

Documentation lifecycle and decision-record mechanics: what the working tree keeps,
how a retired record is recoverable and how retained records stay consistent. It does
not change product contracts, rerun qualification or revise the selected design standard.
It extends ADR-0095's modular publication from incremental extraction to consolidation.

## Drivers

A reader must reach current contracts and their rationale without traversing history
(AP-01, AP-04). Each fact keeps one owner (DP-01). Retained decisions must stay truthful
and checkable (DP-22) while contracts evolve explicitly (DP-24). Checks must stay local
and cheap (AP-06).

## Options

Keep everything and relabel old material as History: rejected; search, links and agents
still pay for obsolete content, and precedence notes remain required reading. Publish a
separate archive book: rejected; it duplicates Git and needs maintenance. Supersede every
old record pairwise: rejected; it reconstructs chains nobody needs. Selected: retire by
applicability, keep Git as the archive, and keep a compact current set.

## Outcome

1. The working tree holds current contracts (`docs/authoritative_design/sections/`), their
   rationale (retained ADRs or the owning section), current reference, current work and
   enduring development instructions. Completed plans, packets, resolved reviews, old
   standard files and obsolete decisions are removed once their enduring meaning has an
   owner and no reader, tool or test consumes them. Git history is the archive; there is
   no archive tree, second book or per-file retirement ledger.
2. A decision is retained when its rationale explains the current system. Adoption status
   alone does not make a record current. An obsolete record is removed, not rewritten;
   when current rationale needs a record, a new current-basis record is written.
3. Numbers are never reused, and gaps are permitted. ADRs: allocation takes the highest
   number retained or ever added in Git history, plus one; the lint also requires the
   highest number ever issued on `main` to stay in the tree until a newer record exists,
   which covers clones without history. Plans: `just plan` allocates from retained and
   historical plan numbers alike. Register rows: a checked high-water mark in
   `register.md` names the highest issued row id; the lint rejects lowering it or reusing
   an id that `main` no longer lists, so an empty register reuses nothing.
4. Retained accepted records remain immutable. The permitted edits are `status`,
   `superseded-by`, appending to `## Status history`, and reference relocation: when a
   cited review or linked file is retired, the record's `review` becomes
   `git:<commit>:<path>[#anchor]` and a body link becomes this repository's permalink to the
   same path, at a commit in this repository's history (reachable from HEAD) whose file is the
   one the record cited.
   A retained supersession pair stays symmetric; a reference to a retired record is
   historical and needs no counterpart. The lint checks identity and existence, not
   historical semantics.
5. Surviving section identities keep one owner. A retired mechanism keeps a one-line
   retirement pointer where its identity is still cited; old blueprint anchors map to the
   new owners from a compatibility table outside Current search.
6. Completed work is not a backlog. Retired plans, reviews and register rows create no
   obligation; the register keeps only genuine deferrals and may be empty.

### Consequences

Readers see one current description. Old detail requires a Git lookup, which is deliberate.
Links into retired files must be repaired to a current owner or an immutable permalink.
Accepted records that cite retired reviews carry Git citations.

### Compensating controls

`just adr-lint` rejects duplicate IDs, removal of the highest issued record, asymmetric
retained pairs, unresolved blueprint identities, missing local or Git review sources and a
relocation that does not cite the same reachable file. The immutability check still rejects
any other accepted-record edit, and status history is append-only. A shallow clone checks
the shape of references whose history it lacks; CI clones full history. Offline fragment checks
on the built book catch dangling links. The consolidation baseline commit
`8950dd3d6ddb3aa7c78acc0db7d0601497b302a8` (on `main`) holds every retired file.

### Confirmation

The tooling is tested by `DecisionRecordTests` (`just setup-test`); the corpus retirement
is carried out by Plan 19. Architectural fitness: one owner per section identity and a
bounded Current scope. Document checks: `just adr-lint`, `just docs` and the offline
link/fragment check.

## Pros and cons

Git lookup is less convenient than a published archive; the maintainer prefers current
explanation over maintained history.

## More information

Blueprint §0.1, §0.3 and §24.4; ADR-0095; `docs/dev/documentation.md`;
`docs/plans/19-current-documentation-consolidation.md` (its finding dispositions record the
scoped review's C1–C8).

## Status history

- 2026-09-25 — proposed; implementation authorized by the maintainer through Plan 19.
- 2026-09-26 — accepted by the maintainer. Scoped review verdict Accept-scoped (review §13);
  its excluded items N1, N2 and the C3 register check were fixed before acceptance.
