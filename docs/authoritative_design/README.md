# Authoritative design

## What "authoritative" means

[`blueprint.md`](blueprint.md) is the design. When an implementation and the
blueprint disagree, one of them is a bug, and an ADR says which. Nothing else in
this repository has that status:

| Document | Status |
|---|---|
| [`blueprint.md`](blueprint.md) | **Authoritative.** One file, revised in git. Revision 5 is the current Proposed contract amendment; tags `design-rev2` and `design-rev3` mark the seeded history. |
| [`proposal.md`](proposal.md) | **Historical.** The thesis the blueprint makes concrete; superseded by it and kept for the record. Front matter says so. |
| [`../adr/`](../adr/) | **Why.** One record per decision. A record binds; it does not restate the design. |
| [`../design_review/reviews/`](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md) | **Evidence, not authority.** A review finds defects and cites measurements. It does not change the design; the ADR that responds to a finding does. |
| [`../capability-maps/`](../capability-maps/arrow-rust.md) | **Evidence.** What the pinned libraries actually do, with reproducible probes under `evidence/`. |

## Section numbers are citation targets

ADRs, reviews, plans, the agent instructions and the lint all cite the blueprint
by section: `blueprint §14.3`, `§D7`, `§3.1`. Those numbers are stable. New
material is inserted as a sub-section (`§14.3.1`), **never** by renumbering, and
`scripts/adr.py lint` resolves every `§` citation in every ADR against the
headings of `blueprint.md` — so a renumbering fails `governance / adr-lint`
rather than quietly rotting every record that cited it.

The decision behind that rule is
[ADR-0033](../adr/0033-one-blueprint-file-with-stable-section-numbers.md).

## The amendment rule

The blueprint changes only through a pull request that:

1. is labelled `adr` and titled `adr: ADR-NNNN <title>` (or is a named follow-up
   `design:` PR referenced by that ADR);
2. adds or changes exactly one decision record under [`../adr/`](../adr/);
3. adds a row to the blueprint's **Revision history** table naming the revision,
   the date, the change and the git tag or pull request;
4. adds an inline `> Decision: ADR-NNNN` line under the heading of every section
   the decision governs, so a reader of the section finds the record without
   searching;
5. keeps every existing section number exactly where it was.

An ADR may merge with `status: proposed` only if the PR is also labelled
`needs-review`. Changes that alter D1–D14, add or remove a crate, add, drop or
major-bump a dependency family, change the hashing contract, the Python boundary
contract, the metadata conventions or the commit contract, or that deviate from
a SHOULD, need an ADR **and** a design review with an Accept or Accept-scoped
verdict before the record becomes `accepted`. The full table is in
[`../plans/01-repository-configuration.md`](../plans/01-repository-configuration.md)
§7 and in `.codex/skills/adr/SKILL.md`.

## Revisions so far

| Revision | What it settled |
|---|---|
| 1 | The initial blueprint following the proposal. |
| 2 | The first review's priority-1 findings (F1–F7, F9, F15); the dependency anchors moved to DataFusion 55.1.0 / Arrow 59.3.0 with family-wide `=` pins. Tag `design-rev2`. |
| 3 | Every finding of both reviews, including the items revision 2 parked (F8, F10, F12–F14, F16, F17) and R2-1 – R2-8, using the four capability maps' measurements. Tag `design-rev3`. |
| 4 | Repository conventions: MSRV equals the pinned toolchain ([ADR-0018](../adr/0018-dependency-pins-lockfile-and-msrv.md)); workspace layout additions ([ADR-0038](../adr/0038-workspace-layout-additions.md)); documents under `docs/` ([ADR-0033](../adr/0033-one-blueprint-file-with-stable-section-numbers.md)). |
| 5 | **Proposed:** library-backed semantic/quantity admission, complete demand/stage/reuse contracts, guarded kernels/numerics, canonical v2 versus encoded integrity, shared accounted resources and bounded relational/import operators. ADR-0039–ADR-0048; [review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), [implementation handoff](../plans/02-blueprint-revision-5-contracts.md). |

## Evidence labels are mandatory

Every claim in the blueprint, in an ADR, in a plan's verification section and in
a register row carries a charter §D label — `Proposed`, `Interface-checked`,
`Implemented`, `Tested`, `Measured`, `Formally established`. `Tested` and
`Measured` must name the test or benchmark and the conditions. `Proposed` is not
a failure state; an unlabelled claim is.
