---
id: ADR-0016
title: Adopt D13: initialization, homotopy and fix-then-release are immutable case overlays
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-13, DM-29, DM-20]
blueprint: [§D13, §19.1, §17.1]
review: not-required: neither review raised a finding against D13
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An operation genuinely needs to change structure mid-run, for example a native GDP lowering that adds equations during a solve
verification: `tests/lifecycle` overlay-immutability tests; `python / test` golden fixtures open stores read-only

---

# ADR-0016: Adopt D13: initialization, homotopy and fix-then-release are immutable case overlays

## Context

Blueprint D13 separates a model revision (structure), a case revision (values, bounds, fixed status, objectives, overlays) and a run (what happened), and makes initialization stages, homotopy steps and fix-then-release operations immutable case overlays.

## Scope

Binds the write paths during execution. The change-set model for authored data is ADR-0027.

## Drivers

IDAES's initialization mutates the model and restores it, which makes a failed initialization leave a model in an undefined state; reproducing a run requires knowing what was mutated.

## Options

Mutate and restore — rejected: an interrupted restore is unrecoverable and unreproducible. Copy the model per stage — rejected: overlays are the cheap, inspectable form of the same thing.

## Outcome

Cases and results never mutate the model. Every stage is an overlay with its own identity; a run records the overlay chain it applied.

### Consequences

Every operation that IDAES expresses as mutation becomes an overlay with a name, which is more objects but a legible history.

### Compensating controls

`tests/lifecycle` asserts that an interrupted run leaves the model revision byte-identical; the golden-store fixture opens read-only so a Python test cannot write one.

### Confirmation

The last iterate of a cancelled or failed run is written under the run's terminal status, so a cancelled run is distinguished by `runs.status`, never by absent rows (§18.3).

## Pros and cons

Overlays cost storage and a resolution step; mutation costs reproducibility.

## More information

Blueprint §D13, §17.1 (plan model), §19.1 (cases and overlays), §23 (failure semantics).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
