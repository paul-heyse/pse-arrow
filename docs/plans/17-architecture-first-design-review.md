---
title: Architecture-first design review
status: in-progress
date: 2026-09-25
adrs: [ADR-0094]
phase: 1
---

# Architecture-first design review

## Context

Implement the maintainer-approved six-foundation review proposal. This plan owns the
process rollout; Plan 16 retains product implementation and qualification. No runtime
refactor, dependency upgrade, test campaign or historical requalification is implied.

## Decisions

[ADR-0094](../adr/0094-architecture-first-design-review.md) records the governance change.
The [standard declaration](../design_review/design_principles/standard.toml) selects the
current core, profile and binding. Existing accepted records and historical reviews stand.

## Plan

This table owns rollout status. Each packet depends on its predecessor.

| Packet | Responsibility | Acceptance | Status |
|---|---|---|---|
| P01 | Core principles and review template | Six foundations, consequential architecture verdict, retained correctness obligations and version mapping | in-progress |
| P02 | Skills, profiles, roles and binding | Architecture questions lead; detailed mechanisms remain conditional; generated roles agree | pending |
| P03 | Decision and plan tracking | One disposition owner, concrete scenario links, current-status index links, governance amendment | pending |
| P04 | Process pilots and governance review | One extension and one subsystem reviewed from live source; calibrate actual findings and justified abstractions | pending |
| P05 | Applicable final checks | Skill validation, setup controls, ADR/agent/document checks; review all changes | pending |

Delete superseded active wording that excludes architectural consequences, makes library
integration costs unassessable, or treats specifications as inherently competing authority.
Do not delete historical reviews or obsolete principles referenced by accepted records.

## Verification

**Proposed:** pilot reviews establish whether the process can expose change amplification,
composition and test-isolation problems. Static checks establish links and configuration only.
This documentation/governance scope needs no product compilation, solver or performance run.
The small ADR identifier change uses existing setup controls plus a representative ADR check.
Final results and their zero baseline will be recorded here once, at closure.

## Finding dispositions

This table owns findings raised by the process review and pilot; source reviews remain
observations. Each row will link the scenario, decision or work owner, and closure evidence.
Product findings may be deferred with a concrete revisit trigger without claiming product
acceptance. This plan completes when the process is implemented, piloted and checked.

| Finding | Scenario | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|

## Open items

ADR-0094 remains proposed until its decision PR is accepted. Local implementation and review
are authorized; this plan does not manufacture an approval or publish changes remotely.

## Outcome (recorded after implementation)

### What was built

Pending final checks.

### A mistake made and corrected

Pending pilot and integration review.

### Deviations from the plan, deliberate

None recorded yet.
