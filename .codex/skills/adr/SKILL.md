---
name: adr
description: Write, lint, index or supersede an architecture decision record for pse-arrow. Use when a change alters a D1-D14 decision, adds or removes a crate, moves a dependency family or pin, changes the hashing, Python-boundary, metadata or commit contract, deviates from a SHOULD, moves the IDAES parity pin, fires a deferred trigger from the register, or touches governance - and whenever someone says "ADR", "decision record", "needs-adr", "why was this decided", "supersede ADR-NNNN", "just adr-new", or asks whether a change needs a decision record or a design review.
allowed-tools: Read, Glob, Grep, Bash, Write, Edit
user-invocable: true
model-baseline: claude-5 (2026-08)
---

# Decision records

`docs/authoritative_design/blueprint.md` is the design. **ADRs record why it
reads the way it does.** Design reviews under `docs/design_review/reviews/` are
evidence, not authority. Plans under `docs/plans/` record how work is sequenced
and are living documents; an ADR is immutable once accepted.

## 1. Does this change need one?

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; adds, drops or major-bumps a dependency family; changes the hashing contract, the Python boundary contract, the metadata conventions or the commit contract; any SHOULD deviation; governance changes | **ADR + design review** — label the PR `needs-review`; the review's verdict must be Accept or Accept-scoped before the record's status becomes `accepted` |
| New relation family, pass, kernel contract or backend binding **within** an accepted decision; a small local SHOULD deviation; moving the IDAES parity pin; a deferred trigger in `docs/adr/register.md` fires | **ADR (short)**; review at maintainer discretion |
| Bug fixes; refactors inside existing contracts; tests; documentation wording; a patch bump inside a pinned family; tooling | **Neither** — an ordinary PR with the Evidence field of the PR template filled |

If you are unsure, look at what the change touches: a change that makes a
previously-recorded decision untrue needs an ADR that supersedes it, not an edit.

## 2. Write one

```bash
just adr-new my-slug --title "Imperative one-liner"     # or:
python3 scripts/adr.py new my-slug --title "Imperative one-liner"
```

That allocates the next number, copies `docs/adr/template.md` and stamps today's
date. Then fill the front matter — these are the **charter §H** fields, and the
lint checks every one of them:

| Field | Rule |
|---|---|
| `id` | `ADR-NNNN`, equal to the filename's number. Allocated by the tool; never edited. |
| `title` | Imperative, one line. It becomes the PR title: `adr: ADR-NNNN <title>`. |
| `status` | `proposed` \| `accepted` \| `rejected` \| `deprecated` \| `superseded`. |
| `date` | `YYYY-MM-DD`, the day the decision was taken. |
| `deciders` | GitHub handles. |
| `level` | `decision` \| `should-deviation` \| `must-gap`. A `must-gap` **narrows the supported scope**; it never claims compliance. A record that *amends* the blueprint is a `decision`, not a deviation — say so in Scope. |
| `principles` | The charter IDs this bears on, `DM-01` … `DM-60`. Cite the ones the decision actually turns on, not a wall. |
| `blueprint` | The sections governed: `[§14.3, §D7]`. Every citation must resolve to a heading in `blueprint.md`, including `§D1` … `§D14`. |
| `review` | A path into `docs/design_review/reviews/…` (optionally `#anchor`) where a finding motivated this, **or** `not-required: <reason>`. |
| `evidence` | A charter §D label — `Proposed`, `Interface-checked`, `Implemented`, `Tested`, `Measured`, `Formally established`. |
| `supersedes` / `superseded-by` | Symmetric: if A supersedes B, B's `superseded-by` is A and B's status is `superseded`. Use `just adr-supersede`. |
| `revisit` | An **observable trigger**, not a date. "A measurement shows X", "crate Y releases Z", "the first non-FFI `unsafe`". If the trigger is not imminent, add a row to `docs/adr/register.md` as well. |
| `verification` | The named test, lint, benchmark or CI job that shows the decision holds — `tests/governance/tests/pins_match_blueprint.rs`, `rust / family-check`, `governance / adr-lint`. Not "code review". |

Body sections: **Context · Scope · Drivers · Options · Outcome** (with
*Consequences*, *Compensating controls*, *Confirmation*) **· Pros and cons ·
More information · Status history**. Charter §H: *"a short, concrete decision
record is sufficient"* — one to three sentences per section. Cite
`blueprint §14.3` instead of restating it.

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
the charter's §G "attractive claim" table is about.

### Citation forms

`blueprint §14.3` · `§D7` · `ADR-0020` · `DM-33` · `G4` · register row `R-05`.
In front matter, blueprint sections carry the `§`: `blueprint: [§3.1, §D1]`.

## 3. Lint, index, supersede

```bash
just adr-lint            # python3 scripts/adr.py lint  +  check_register.py --lint
just adr-index           # regenerates docs/adr/README.md and the SUMMARY block
just adr-supersede 0020 0041
just register-check      # rows that are due, and runs the automatable checks
```

`scripts/adr.py lint` checks the filename pattern, contiguous numbering from
0001, the required keys, the enums, `DM-NN` shapes, that every `§` citation
resolves to a real blueprint heading, that the `review` path exists, symmetric
supersession, and **immutability**. `scripts/adr.py index` regenerates
`docs/adr/README.md` and the block between `<!-- adr:begin -->` and
`<!-- adr:end -->` in `docs/SUMMARY.md`; never hand-edit either.

### Immutability

A record whose status on `origin/main` is anything other than `proposed` may
change only in `status`, `superseded-by`, and the `## Status history` section.
Everything else — every other front-matter key and every other body section — is
frozen. `governance / adr-lint` fails the PR otherwise.

**To change a decision, supersede it.** Write a new record that states what
changed and why, run `just adr-supersede <old> <new>`, and append the reason to
the new record's Status history. Do not edit the old one's argument.

## 4. The decision-PR rule

An ADR enters the repository or changes status only in a pull request that:

1. is **labelled `adr`** and **titled** `adr: ADR-NNNN <title>`;
2. amends `blueprint.md` in the same PR (or in a named follow-up `design:` PR the
   ADR references) with a **revision-history row** and an inline
   `> Decision: ADR-NNNN` line under each governed section's heading;
3. keeps every blueprint section number where it was — insert `§14.3.1`, never
   renumber (ADR-0033);
4. may merge with `status: proposed` **only** if it also carries `needs-review`.

Maintainer merge is the approval (ADR-0034). If the decision defers something,
add the register row in the same PR so the trigger has an owner and a date.

## 5. Before you finish

- `just adr-lint` and `just adr-index` are clean.
- The `verification:` field names something that exists or is created by this PR.
- If the record defers anything, `docs/adr/register.md` has a row for it.
- If the record supersedes another, both files changed and the lint agrees.
- The blueprint carries the `> Decision:` marker and a revision row.
