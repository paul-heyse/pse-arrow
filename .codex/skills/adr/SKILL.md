---
name: adr
description: Write, lint, index or supersede an architecture decision record for pse-arrow. Use when a change alters a D1-D14 decision, adds or removes a crate, moves a dependency family or pin, changes the hashing, Python-boundary, metadata or commit contract, deviates from a SHOULD, moves the IDAES parity pin, fires a deferred trigger from the register, or touches governance - and whenever someone says "ADR", "decision record", "needs-adr", "why was this decided", "supersede ADR-NNNN", "just adr-new", or asks whether a change needs a decision record or a design review.
allowed-tools: Read, Glob, Grep, Bash, Write, Edit
user-invocable: true
model-baseline: claude-5 (2026-08)
---

# Decision records

`docs/authoritative_design/README.md` routes to the design collection. **ADRs record
why its contracts read the way they do.** Design reviews under `docs/design_review/reviews/` are
evidence, not authority. Plans under `docs/plans/` record how work is sequenced
and are living documents; an ADR is immutable once accepted.

The tree keeps only decisions whose rationale explains the current system (ADR-0096).
Obsolete records were retired to Git history; their numbers are never reused, so the
index has gaps. To understand the current system, read the owning section first.

## 1. Does this change need one?

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; majors one of the four pinned families (arrow, datafusion, object_store, pyo3); changes the hashing contract, the Python boundary contract, the metadata conventions or the commit contract; any SHOULD deviation; governance changes | **ADR + design review** — label the PR `needs-review`; the review's verdict must be Accept or Accept-scoped before the record's status becomes `accepted` |
| New relation family, pass, kernel contract or backend binding **within** an accepted decision; a small local SHOULD deviation; moving the IDAES parity pin; a deferred trigger in `docs/adr/register.md` fires | **ADR (short)**; review at maintainer discretion |
| Bug fixes; refactors inside existing contracts; tests; documentation wording; a patch bump inside a pinned family; tooling | **Neither** — an ordinary PR with the Evidence field of the PR template filled |

If you are unsure, look at what the change touches: a change that makes a
previously-recorded decision untrue needs an ADR that supersedes it, not an edit.

## 2. Write one

```bash
just adr-new my-slug --title "Imperative one-liner"     # or:
python3 scripts/adr.py new my-slug --title "Imperative one-liner"
```

That allocates the highest retained number plus one, copies `docs/adr/template.md` and stamps today's
date. Then fill the front matter — these are the **design principles §H** fields, and the
lint checks the required fields (optional traceability fields are described below):

| Field | Rule |
|---|---|
| `id` | `ADR-NNNN`, equal to the filename's number. Allocated by the tool; never edited. |
| `title` | Imperative, one line. It becomes the PR title: `adr: ADR-NNNN <title>`. |
| `status` | `proposed` \| `accepted` \| `rejected` \| `deprecated` \| `superseded`. |
| `date` | `YYYY-MM-DD`, the day the decision was taken. |
| `deciders` | GitHub handles. |
| `level` | `decision` \| `should-deviation` \| `must-gap`. A `must-gap` **narrows the supported scope**; it never claims compliance. A record that *amends* the architecture sections is a `decision`, not a deviation — say so in Scope. |
| `principles` | The principle IDs this bears on: architectural foundations `AP-01` … `AP-06`, core refinements `DP-01` … `DP-24`, process-simulator profile `PS-01` … `PS-13` (see `docs/design_review/design_principles/standard.toml`). Accepted records keep their legacy `DM-nn` IDs from the retired charter. Cite the ones the decision actually turns on, not a wall. |
| `blueprint` | The sections governed: `[§14.3, §D7]`. Every citation must resolve to one owning heading under `docs/authoritative_design/sections/`, including `§D1` … `§D14`. |
| `review` | A path into `docs/design_review/reviews/…` (optionally `#anchor`) where a finding motivated this, an immutable historical source `git:<commit>:<path>[#anchor]` when that review was retired, **or** `not-required: <reason>`. |
| `evidence` | A design principles §D label — `Proposed`, `Interface-checked`, `Implemented`, `Tested`, `Measured`, `Formally established`. |
| `supersedes` / `superseded-by` | Symmetric for a retained pair: if A supersedes B and both remain, B's `superseded-by` is A and B's status is `superseded`. Use `just adr-supersede`. A retained record may name a retired predecessor; a record superseded by a retired one is itself retired. |
| `revisit` | An **observable trigger**, not a date. "A measurement shows X", "crate Y releases Z", "the first non-FFI `unsafe`". If the trigger is not imminent, add a row to `docs/adr/register.md` as well. |
| `verification` | Name the scenario/property and the analysis, contract test, lint or measurement that can settle it. An architectural judgment names the review argument and scope; a syntax lint alone cannot establish architecture. Avoid an unspecified "code review". |
| `standard` (optional) | Snapshot the reviewed core/profile versions when relevant; current version selection remains in `standard.toml`. |
| `scenarios` (optional) | Inline list of repository-relative scenario references; link their definitions rather than copying them. |

Body sections: **Context · Scope · Drivers · Options · Outcome** (with
*Consequences*, *Compensating controls*, *Confirmation*) **· Pros and cons ·
More information · Status history**. A short, concrete decision record is
sufficient — one to three sentences per section. Cite
`blueprint §14.3` instead of restating it.

### Architectural drivers and tracking

For architecture decisions, connect the selected option to concrete change scenarios and
responsibility boundaries. Compare composition, consumed contracts, local test setup and
integration costs as well as correctness. A new trait, crate or registry is not itself an
improvement. Use the current review template; keep its architectural and behavioral judgments
distinct and preserve the scope of each claim.

The owning plan holds current finding dispositions and packet status. Link that owner from
More information; accepting an ADR does not resolve implementation work. Accepted `evidence:`
records support at decision time and is not rewritten with every later test run. Historical
reviews retain their standard versions and observations. Deferred decisions still use the
register and an observable trigger; ordinary rollout work belongs in its plan.

### The evidence-label rule

`evidence:` describes the state of *this decision's* support, and the labels are
different claims, not one ladder:

- `Proposed` — the decision is a position; nothing is built. Most backfill
  records are honestly `Proposed`, and that is fine.
- `Interface-checked` — a required interface or mechanism was inspected (a
  capability-map probe, a rustdoc extraction, a `cargo info` check). Say which.
- `Implemented` / `Tested` / `Measured` — `Tested` and `Measured` must name the
  test or benchmark **and the conditions**. A measured implementation can still
  be incorrect; an interface-checked design can still need real engineering.
- `Formally established` — a specific property follows from an identified formal
  argument with explicit assumptions. Ordinary testing is not this.

Never raise a label to make a record look finished. A wrong label is the defect
the design principles' §G "attractive claim" table is about.

### Citation forms

`blueprint §14.3` · `§D7` · `ADR-0082` · `AP-06` · `DP-09` · `PS-10` · `G4` · register row `R-05`.
In front matter, blueprint sections carry the `§`: `blueprint: [§3.1, §D1]`.

## 3. Lint, index, supersede

```bash
just adr-lint            # python3 scripts/adr.py lint  +  check_register.py --lint
just adr-index           # regenerates docs/adr/README.md
just adr-supersede <old> <new>
just register-check      # rows that are due, and runs the automatable checks
```

`scripts/adr.py lint` checks the filename pattern, unique numbers, that the highest
number issued on `origin/main` is still present (so allocation never reuses a number),
the required keys, the enums, `AP-NN`/`DP-NN`/`PS-NN` (or legacy `DM-NN`) shapes, that
every `§` citation resolves to one owning heading, that the `review` path or Git source
exists, symmetric supersession of retained pairs, and **immutability**. `scripts/adr.py index` regenerates
`docs/adr/README.md`; never hand-edit that generated index. Book navigation is derived
during publication from `docs/site.toml`, not edited by the ADR tool.

### Immutability

A record whose status on `origin/main` is anything other than `proposed` may
change only in `status`, `superseded-by`, and the `## Status history` section.
Everything else — every other front-matter key and every other body section — is
frozen. `just adr-lint` and the manually dispatched `governance / adr-lint` check
can detect violations when the maintainer chooses to run them.

**To change a decision, supersede it.** Write a new record that states what
changed and why, run `just adr-supersede <old> <new>`, and append the reason to
the new record's Status history. Do not edit the old one's argument.

**Reference relocation** is the one other permitted edit (ADR-0096): when a cited review
or linked plan is retired, move the accepted record's `review` to
`git:<commit>:<same path>` and a body link to the repository permalink of the same path
at that commit. The lint verifies the path exists at the commit.

### Retirement

When a record's rationale no longer explains the current system — its mechanism is gone
or a later decision fully replaced it — delete the file rather than rewrite it (ADR-0096).
Move any surviving meaning to the owning section first, repair inbound links, and keep the
highest-numbered record until a newer one exists. Retirement needs no per-record ADR;
write a new current-basis record only when current rationale needs one.

## 4. The decision-PR rule

An ADR enters the repository or changes status only in a pull request that:

1. is **labelled `adr`** and **titled** `adr: ADR-NNNN <title>`;
2. amends the authoritative collection in the same PR (or in a named follow-up `design:`
   PR the ADR references) with a **revision-history row** in `blueprint.md` and an inline
   `> Decision: ADR-NNNN` line under each governed section's heading;
3. preserves section identifiers — insert `§14.3.1`, never renumber. A move leaves an
   anchor/link stub and one normative owner (ADR-0095);
4. may merge with `status: proposed` **only** if it also carries `needs-review`.

Maintainer merge is the approval (ADR-0034). If the decision defers something,
add the register row in the same PR so the trigger has an owner and a date.

## 5. Before you finish

Run these when the maintainer requests an ADR check; they are not a commit, push or
plan-close prerequisite (AGENTS.md *Execution rhythm*):

- `just adr-lint` and `just adr-index` are clean.
- The `verification:` field names something that exists or is created by this PR.
- If the record defers anything, `docs/adr/register.md` has a row for it.
- If the record supersedes another, both files changed and the lint agrees.
- Each governed section's owner carries the `> Decision:` marker; the blueprint carries
  the collection revision row.
