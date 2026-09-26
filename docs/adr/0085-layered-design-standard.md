---
id: ADR-0085
title: Adopt a layered design standard with a process-simulator profile
status: accepted
date: 2026-09-24
deciders: [paul-heyse]
level: decision
principles: [DP-13, DP-16, DP-22, DP-23]
blueprint: [§2.1]
review: git:8950dd3d6ddb:docs/design_review/reviews/design_review_m22-local-qualification_2026-09-24.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A second domain profile or a second repository adopts the core layer, or a review finds a principle it cannot apply without a repository-specific reading.
verification: The next design reviews under docs/design_review/reviews/ cite DP/PS IDs, and settle G1–G8 and PS-G1–PS-G3; `just adr-lint` accepts DP/PS principle IDs.
---

# ADR-0085: Adopt a layered design standard with a process-simulator profile

## Context

The design standard lived in four overlapping documents: the Data Model–Based Design
Charter (60 principles), the agent directive, the review template, and the Rust computation
architecture guidelines (RCA). The `design-review` skill's reference restated their indexes
and conflicts. Reviews used about 20 of the 60 principles. The standard had no principle on
library-first implementation or on the numerical integrity of a simulator. RCA mixed
principles with version-specific library facts that predate Plan 14.

## Scope

This record replaces the standard used by design reviews and ADR `principles:` fields. It
does not change the blueprint or any accepted decision. It adds no executable validation of
design alignment and mandates no probes, tests or records beyond what an agent's judgment
calls for: alignment is established by judgment and review (principles §0, DP-23).

## Drivers

- A standard that is itself one authority per fact and that reviewers can audit quickly.
- Reuse beyond this repository: the core and the profile carry no repository references.
- Library-first implementation, built-ins over bespoke code, and removal of replaced code.
- Simulator-specific correctness: physical consistency, well-posedness, numerical integrity.
- No new checking burden while the design is still changing.

## Options

- Keep the charter and amend it in place — rejected: its IDs would change meaning under
  accepted records, and it would stay repository-neutral without any domain profile.
- One merged repository-specific document — rejected: the maintainer wants the core and
  review method to be reusable, with repository variants layered on top.
- A machine-readable review record with a review lint — rejected by the maintainer as
  checking burden that does not justify its cost at this stage.

## Outcome

Design reviews use a three-layer standard declared in
`docs/design_review/design_principles/standard.toml`:

- **Core** (`core/`): principles DP-01–DP-24 in five pillars, gates G1–G8 (G8 library
  leverage is new), evidence vocabulary §D, library consideration §F, exception record §H, and a
  review template with change and design tiers, conformance and target purposes, and slots
  1–12. It is repository- and domain-agnostic.
- **Process-simulator profile** (`profiles/process-simulator/`): PS-01–PS-13 and gates
  PS-G1–PS-G3, plus review additions to the core slots. It names no library.
- **pse-arrow binding** (`binding/pse-arrow.md`): no principles; it maps roles to this
  repository's authorities, commands, routes and local policies, records known conflicts,
  and maps RCA sections.

Profiles and bindings add or tighten, never relax. The `design-review` skill applies the
core; the new `design-review-process-simulator` skill layers the profile onto it.

### Consequences

New ADRs cite `DP-nn` or `PS-nn`. Accepted records keep `DM-nn`, which principles §I maps.
The old charter, directive, template and RCA remain in place with a superseded banner so
their links resolve. The charter's weighted assessment dimensions are retired.

### Compensating controls

Every retired ID has a mapping (principles §I; the binding's RCA lineage). The binding's
known-conflicts table records where the blueprint predates the standard.

### Confirmation

The next reviews apply the standard as described in `verification:`.

## More information

- `docs/design_review/design_principles/core/design-principles.md`
- `docs/design_review/design_principles/profiles/process-simulator/principles.md`
- `docs/design_review/design_principles/binding/pse-arrow.md`

## M22 local qualification

**Tested and Measured:** the [M22 packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m22-execution.md#verification)
records local Linux functional Q01–Q17, the 23 cached-development case-cost workloads,
zero required failures and retained-origin conditions. It distinguishes admitted memory
allowances from measured pool/RSS observations and excludes Rust build time.

The [independent final review](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/design_review/reviews/design_review_m22-local-qualification_2026-09-24.md)
accepts the relevant scoped contracts with no open MUST finding. Companion runtime,
scientific and claims reviews cover G1–G8 and PS-G1–PS-G3. Blueprint revision 51 and
ADR-0087 govern local acceptance. Strict Clippy cleanup and release/remote/platform
qualification remain separate; no broader clean or empirical claim follows.

## Status history

- 2026-09-24 — proposed.

- 2026-09-24 — accepted for local Linux M22 scope under ADR-0087 after independent final review; blueprint revision 51 reconciles the contracts. No remote or release qualification is claimed.
