---
id: ADR-0174
title: Exact resource metadata and invocation terminal freeze
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-06, AP-07, DP-08, DP-18]
blueprint: [§24.1]
review: docs/design_review/reviews/design_review_plan36-resource-metadata_2026-10-11.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required indivisible cross-resource disposition cannot be expressed by immutable invocation freeze and resumable exact projection.
verification: Actual-file crash prefixes; both registration/freeze orders; exact identity collision refusal; interrupted projection; original deadlines; authentic terminal association and late effect/drain controls.
---

# ADR-0174: Exact resource metadata and invocation terminal freeze

## Context

Plan 36 P-E1 adopts RC19. Whole-ledger exclusion and rewriting makes unrelated resources interfere. More seriously, registration after terminal reconciliation currently creates an incomplete pinned resource outside that terminal result. Removing unrelated exit sweeping does not correct registration/finalization ordering.

## Scope

Change the test-resource metadata convention and terminal registration protocol in blueprint §24.1. Resource policy, test selection, report authentication, retention, disposal and service/drain authority remain with their existing owners. This decision creates no production-content deletion authority.

## Drivers

An identified resource should consume only its actual metadata dependencies. An invocation must have a closed membership before it can claim terminal projection complete. Lost acknowledgments, interrupted registration and issued late effects must retain exact uncertainty and ownership.

## Options

Retaining the whole mutable ledger preserves unrelated contention and cannot itself fence late registration. A transactional database could make all-resource updates atomic, but no consumed invariant requires their simultaneous visibility. Select per-resource files with one immutable invocation terminal and resumable projection. Reopen the database choice only if the stated indivisible invariant arises.

## Outcome

Use one stable `resources/<id>.json` authority, exact full-identity indexes, active/unresolved access paths and invocation membership. Index hashes select paths; stored normalized full identities must agree. Hot/cold phase changes do not create a second writable resource copy. Bounded cleanup enumerates active/unresolved paths, never historical resources.

Lock order is invocation, logical identity, resource. A resource-first operation never acquires invocation exclusion. Stable lock files survive phase movement and ordinary cleanup. All acquisition consumes the original absolute deadline; no metadata lock covers external creation, service work, removal or drain.

Under invocation exclusion registration verifies Open, reserves identity, durably records Pending membership before resource authority, publishes authority and identity index, marks Registered, then returns its capability. External effects start only after accepted registration. Private publication flushes contents before replacement and the containing directory before promising crash recovery. Freeze resolves every Pending member: a proven never-published metadata intent can roll back; contradictory or uncertain authority remains unresolved and pinned. Post-freeze registration refuses before effects.

The selected terminal composer freezes an immutable authenticated terminal identity binding catalog/selection, workload status/signal, complete or partial reports, dispositions, reconciliation and drain. It then releases invocation exclusion and projects exact members individually. Projection is idempotent against that terminal identity and can resume after interruption. Pass needs complete authentic passing evidence. Freeze does not prove effect drain: accepted creation/effect borrowers remain owned and charged through settlement. Bare unassociated resources remain incomplete/pinned. Reports and membership remain until projection, diagnostic retention and references release them.

### Explicit abandonment without a terminal (Plan50 C05)

The maintainer selected this extension on 2026-10-22. An explicit exact-invocation
abandonment may close membership separately from terminal reconciliation, under the same
invocation authority, exclusion order and original deadline. Resolve proven unpublished
Pending intents and reconstruct published members using the existing conservative rules;
uncertain or contradictory publication remains protected. Durably publish membership closure
before projecting any release and refuse later registration before effects. Repeated or
interrupted requests resume the same disposition; they never reopen membership.

An absent terminal remains absent. Abandonment supplies no terminal identity, outcome,
selection acknowledgement or automatic-cleanup eligibility. An existing frozen terminal stays
immutable. The selected terminal composer remains the sole outcome authority; disposition
must not race a live composer or resurrect closed execution. Physical owner/descendant drain,
creation borrowers, reference protections and unknown external effects retain their existing
release conditions. Resource removal remains exact, separately resumable owner work.

The alternative of synthesizing an incomplete terminal would claim reconciliation that did
not occur; releasing individual resources without closing membership leaves a late-registration
race. A separate membership fence expresses the actual operator decision without either error.
Plan50 C05 owns registration/disposition races, crash-prefix recovery, truthful missing-terminal
reads and protected-effect controls. **Tested:** the explicit disposition controls in
`scripts.tests.test_invocation_disposition`, including the actual dead-composer/no-terminal
case, pass within Plan50's full 1,343-test setup suite; its Outcome records conditions and
limits. This implementation evidence does not change this ADR's proposed decision status.

Cutover excludes legacy writers using the existing legacy lock; imports and validates identified live/unresolved and known cold authorities; then durably publishes the unsupported-version refusal sentinel at the legacy ledger path **before** v2 activation. A crash in this fenced-but-unactivated state makes old writers refuse; the new owner resumes/validates the same staged imports under legacy exclusion and publishes activation only when they agree. It never serves writable v1 while recovering. All consumers migrate together. Preserve ambiguous records and migration evidence. Never restore a writable v1 ledger after activation. No automatic live production registry cutover is part of the implementation tests.

### Consequences

Replace mutable whole-ledger borrowing with exact storage operations. Migrate fixture, report, runner, recovery and explicit cleanup consumers together. One terminal composer owns reconciliation; inherited observer children consume that ownership instead of finalizing independently. Preserve the child's original exit while separately reporting reconciliation or drain errors.

### Compensating controls

Exercise real fsync/replace failures and both racing orders, not only an abstract model. Unknown ownership, incomplete reports, uncertain external effects and missing drain evidence remain pinned. Stable lock retirement needs exclusion of all openers and is outside ordinary per-record compaction.

### Confirmation

Proposed: Plan 36e and its investigation establish the selected protocol and alternatives. The bounded change review assesses the design separately; actual-file implementation controls remain required. ADR status acceptance remains the decision-PR route.

## Pros and cons

Exact access removes unrelated ledger work while keeping policy local. Pending membership and terminal projection add explicit recovery states and directory durability obligations. They avoid a new database dependency without pretending multi-file publication is atomic.

## More information

[Plan 36e](../plans/36e-build-test-and-development.md#exact-metadata-and-terminal-composition) owns execution and acceptance. The original review F25 and audit correction remain historical evidence; Plan 36 owns disposition.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of the accepted Plan 36 target.

- 2026-10-22 — maintainer selected explicit abandonment membership closure for Plan50 C05; terminal and release safeguards retained; status unchanged.
