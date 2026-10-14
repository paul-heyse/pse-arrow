---
id: ADR-0188
title: Use only consumed runtime settings for shared runtime identity
status: proposed
date: 2026-10-14
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-07, DP-16]
blueprint: [§14.3, §18.8, §21]
review: docs/design_review/reviews/design_review_consumed-runtime-settings_2026-10-14.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A concrete hashing operation needs an independently selected scheduling policy.
verification: Bounded target review of consumed settings and process-runtime identity; installed validated EngineSettings and Runtime controls, retained hash golden vectors and selected dependency closures, owned by Plan38f.
standard: {core: '3.4', process-simulator: '1.5'}
---

# ADR-0188: Use only consumed runtime settings for shared runtime identity

## Context

`ResourceBudget::hashing_may_use_pool` grants a hashing permission that no hashing operation
consumes. Its actual observable behavior is through derived budget equality: Python rejects
otherwise identical settings when this boolean differs. It never created an isolated pool.
Plan38's accepted R38-02 and the maintainer's explicit execution choice remove that distinction.

## Scope

Amend blueprint §14.3, §18.8 and §21 to remove the Rust field and Python constructor/getter,
including their equality distinction. Hash algorithms, framing and bytes do not change. This
does not select parallel hashing, a new Runtime isolation option or different native teams.

## Drivers

Resource settings should describe behavior their owner consumes. Adding a real scheduling
policy later should change its actual execution owner and tests, rather than preserve an
unread permission across every Runtime constructor. Identical consumed settings continue to
select the one process-owned pool, spill directory and executor; conflicting real limits refuse.

## Options

Retaining the flag preserves an unsupported promise and an accidental rejection condition.
Wiring it into a hashing implementation would invent a workload requirement and new scheduling
behavior. Remove it instead, using the existing budget and Runtime owners, with no new service,
dependency or compatibility API. The simplest alternative is the selected option; no library
capability is rejected by removing unused local policy.

## Outcome

Only consumed resource settings participate in process-runtime compatibility. Remove
`hashing_may_use_pool` from ResourceBudget, settings projections and the public Python surface.
Formerly distinct values of that permission no longer distinguish otherwise equal budgets.

### Consequences

Callers must stop passing the removed keyword. Existing resource bounds and hash identity are
preserved. This is an intentional public API change in the design phase, not a deprecation path.

### Compensating controls

Generate native stubs from the rebuilt extension. Installed validated tests check the surviving
constructor/getters, identical-settings reuse and refusal of genuinely conflicting settings.
Retain exact hash vectors and explicit Arrow validation in actual correctness closures.

### Confirmation

The linked bounded review evaluates the proposed boundary before implementation. Plan38f owns
current implementation and targeted acceptance evidence; Plan38g owns assembled qualification.
ADR status remains proposed until the repository's decision-PR route is completed.

**Implemented / Tested (2026-10-14):** the field and public projections are removed,
and the rebuilt extension generates their actual absence. The installed validated
`test_process_runtime_reuses_equal_consumed_settings_and_refuses_conflicts` passes
within the 23-test `scripts.python_tests` boundary selection. The selected `pse-ids`
JSON float/framed-hash golden controls pass (3 tests, zero failures). Plan38f records
the actual commands, mode hashes and limits; these do not establish assembled
Plan38g qualification or change this record's proposed status.

## Pros and cons

The public contract becomes smaller and truthful. The cost is removing a keyword and its
accidental incompatibility behavior; the maintainer expressly selected both changes.

## More information

[Plan38f](../plans/38f-boundaries-build-test-and-development.md) owns disposition and execution.
[The Python boundary](../authoritative_design/sections/workflows-and-results.md#section-21)
owns process-runtime sharing; the focused review records the source-backed correction to the
historical claim that BLAKE3 `update_rayon` was gated by this flag.

## Status history

- 2026-10-14 — proposed for maintainer-authorized Plan38f execution.
