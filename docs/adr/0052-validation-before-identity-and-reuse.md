---
id: ADR-0052
title: Validate semantics before content identity and stage reuse
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-07, DM-14, DM-15, DM-31, DM-32, DM-33, DM-59]
blueprint: [§4.3, §5.4, §14.3, §20.1, §22.2]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A reuse route cannot explain validity from its explicit dependency and validation context, or an undeclared read is discovered
verification: `just governance`; semantic admission, invalid-candidate, invalid-rehashed-artifact and incremental-versus-clean fixtures in plan 03

---

# ADR-0052: Validate semantics before content identity and stage reuse

## Context

A checksum establishes encoded integrity, not validity. The implemented manifest checks only an envelope, and the original wave plan published candidates before P2. DataFusion can derive functional dependencies from unverified constraints and optimize away invalid-candidate evidence.

## Scope

ADR-0074 and Plan 11 govern the integrated lifetime, evidence-preservation and
implementation-testing amendments. This proposed record retains its historical
evidence; its prior per-boundary or whole-stage mechanisms do not restrict that target.

**Current construction target:** ADR-0067 and blueprint revision 37 replace this
record's former local replay, row-copy and phase-limited execution mechanisms.
Plan 05 owns implementation; the domain/identity/lifetime requirements retained
below are implemented through its single native preparation/completion route.

Amends the cited blueprint sections within the approved Wave 1 boundary. It supplements existing accepted decisions; their arguments remain immutable. Implementation is authorized by the maintainer's approved execution plan; formal ADR acceptance remains the decision-PR lifecycle.

## Drivers

Explicit semantics, enforceable validity, complete dependencies and honest capability claims before reuse or performance work.

## Options

Retain the inconsistent scaffold: rejected because its consumers cannot preserve the declared meaning. Build a general replacement platform: rejected without a demonstrated need. Complete the existing typed contracts: selected.

## Outcome

Run P2 on unpublished schema-admitted candidates without canonical snapshot identity, memo entries, planner constraints or functional dependencies. Only zero-error candidates gain validated PK/unique constraints and enter canonical publication/ref CAS. Hashes index content and reuse candidates; explicit contracts, invariant results and complete bound dependencies establish admissibility. Compare a cache candidate's complete dependency description and validation scope, not its key alone. Trusted immutable validation may be reused only while the validator, contracts, assumptions and dependency context remain applicable; untrusted restore repeats substantive admission. Implement exact target-runtime dependencies and independently specified cold/warm cases; the discarded engine is not an oracle. A changed declared dependency invalidates affected stages even if resulting values happen to be equal; preserve fresh lineage. No generic validation platform or second dependency engine is introduced.

Store reads and publication receive explicit, privately admitted parent snapshot handles bound by semantic role. They check the exact role set and actual schema/row contracts, retaining each handle's snapshot/manifest-checksum pair. A claimed parent digest does not locate or validate its content. Stage admission also binds one executable registered pass and requires its exact complete output ports; unsupported runtime producers do not gain admission from a run-kind label.

### Consequences

Findings about unpublished candidates have `subject_snapshot = null`; candidate validation must not invent a snapshot hash before acceptance. Publication consumes an immutable complete result; current-format imports/reopening establish unresolved registry properties over actual candidate and parent rows through the same immutable admission program. A new query wrapper does not rescan unchanged admitted owners. Missing validation implementation is a typed refusal, never an implicit pass. The catalog owns that validator interface and the rule engine implements it without a dependency cycle.

The same prepublication distinction applies to `provenance.derivations.snapshot_id`: it is null until a snapshot exists. Derivation identities and exact supporting row keys can be recorded without asserting a nonexistent snapshot identity.

Consumers must use checked contracts and explicit unsupported outcomes. Fixtures and generated output change with their authoritative declarations; this record does not claim runtime acceptance.

### Compensating controls

The scoped design review, negative fixtures and plan 03 terminal gates guard the correction. Existing canonical framing and dependency-family pins remain unchanged except the explicitly named direct promotion.

### Confirmation

`just governance`; semantic admission, invalid-candidate, invalid-rehashed-artifact and incremental-versus-clean fixtures in plan 03. Each result records mode and failure count against baseline zero. The execution ledger distinguishes source inspection, focused tests and complete-wave acceptance.

## Pros and cons

The correction removes an ambiguity or false guarantee with bounded implementation work. Conservative rejection/copying/recomputation may cost more until a separately measured refinement is justified.

## More information

[Wave 1 execution plan](../plans/03-wave-1-foundations.md); blueprint §4.3, §5.4, §14.3, §20.1, §22.2; charter DM-59 and G1-G7. The authorized blueprint amendment uses PSE_DESIGN_EDIT=1 and must carry a revision row in the decision PR.

## Status history

- 2026-09-14 — proposed before implementation; maintainer approved the correction plan and its validation-before-hashing clarification.

### Complete durable reuse context

Proposed before implementation: a durable stage hint stores an optional typed semantic context, separate from its lookup key. The context contains every actual registry declaration row through the declared current context encoding; in-process preparations share immutable owners instead of copying Cells, the complete original source inventory (package/document IDs, path and text), every selected policy ID and actual policy row, and the exact engine profile, semantic settings, built-in function inventory and pinned implementation versions when applicable. Durable reuse requires equality of these complete values plus exact required/absent input roles and manifest references, required current-format boundary admission and current preconditions. Retired formats are unsupported; a current-format hint missing required context forces recomputation. Reused attempts get new pass records; output lineage is never silently backdated. The context is bounded by the existing control-artifact limit and shared resource budget. Hashes select buckets and identify transport artifacts only.

- 2026-09-14 — reconciled with ADR-0067 and Plan 05; prior receipts describe their original code and do not certify the hard-pivot implementation.
