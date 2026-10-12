---
id: ADR-0176
title: Quiescent stage admission and exclusive scratch transfer
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, DP-19, PS-10]
blueprint: [§14.3, §14.3.2, §18.8]
review: 'not-required: bounded ownership refinement within ADR-0166 and accepted Plan 36 RC07/RC13; no standard or scientific-contract change'
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported nested request cannot progress without retaining active admission needed by its child, or transferred storage cannot retain exact compatibility and allocation ownership.
verification: Saturated resident/active admission, fitting phase handoff, queued cancellation and original deadline, paired IBEX/compute contention, sequential scratch reset and incompatible-transfer refusal controls.
---

# ADR-0176: Quiescent stage admission and exclusive scratch transfer

## Context

Plan 36 RC07 and RC13 are accepted. Current job/session coupling can make idle resident owners retain permits needed by their next stage or children. Literal fresh allocation also prevents exclusive reuse of drained evaluator storage. ADR-0166's bounded population, native teardown and original-clock requirements remain applicable.

## Scope

Refine resource and attempt ownership for P-B3, P-C8 and P-C2. Preserve host partitions, MathPolicy jobs=4 and existing conservative compiler/worker allowances until actual construction bounds justify a separate revision. No public hashing, scientific result, rank, solver-selection or deadline contract changes.

## Drivers

Resident bytes and native lifetime differ from active computation. Queueing cannot hold permits needed to release admission. Reusing storage cannot reuse prior scientific state or controls.

## Options

Increasing width retains circular waits. Releasing resident charges while retaining allocations misstates ownership. Blocking compute reacquisition while holding IBEX can deadlock an active team. Select explicit lifetimes and paired nonblocking reacquisition.

## Outcome

Own finite population, resident storage and active job/CPU admission separately. Reserve complete next-stage construction/transient demand before active job and CPU acquisition. Failed tentative acquisition releases temporary owners before waiting under original cancellation/deadline. Oversized, closed, cancelled and expired admission have typed operational outcomes. Idle native sessions retain population, memory and joined teardown, but no active job/CPU solely to preserve storage. Optional idle retention yields when needed for progress.

Nested synchronous work borrows its enclosing allowance. Independently admitted children start only after a genuinely quiescent parent releases active admission, including fitting estimation-to-profile chains. Results retain declared order and scientific meaning. Active siblings prevent whole-team release.

At a quiescent IBEX wait, release active computation, retain resident ownership, and acquire the library guard together with immediate `ComputeAdmission::try_resume` against the original execution. If unavailable, release the guard before cancellable readiness/retry. Never hold IBEX across blocking compute reacquisition. All supported proof entrypoints use this ownership distinction; optional-execution paths do not gain a new clock. Opaque native waits without a quiescent seam retain their team allowance.

An attempt may relinquish empty scratch to one lane owner only after native/provider/callback drain. Compatibility includes exact assembly/program/profile identity. Transfer only reset/poisoned frames, instruction-bearing evaluator storage and empty reusable buffers. Discard providers, numerical/memo products, applicability witnesses, failure latches and old execution/cancellation handles. Receiving attempts obtain exclusive ownership and fresh controls; reset failure destroys the candidate. Idle storage remains charged to its lane owner until destruction. No simultaneous mutable sharing or global worker cache is introduced. Independent original-space verification stays independent.

### Consequences

Runtime supplies detachable resident accounting and active-stage lifetimes; mathematics consumes them. Actual faer parallelism is admitted per invocation, with nested work sequential unless separately admitted. Scalar optimization and lazy order construction can proceed independently of scratch transfer.

### Compensating controls

Unknown foreign/provider allocation retains a conservative preallocation guard. Cancellation does not release native ownership before join. Escaped immutable results retain their storage charge through their actual output owner. Reservations do not establish RSS bounds.

### Confirmation

Proposed target; implementation and targeted controls remain Plan 36b/36c-owned. Assembled qualification remains P-G1. ADR acceptance follows the decision-PR route.

## Pros and cons

Explicit lifetimes permit progress and allocation reuse while retaining bounded ownership. They require real quiescent seams and expose unsupported opaque waits.

## More information

Plan 36b P-B3 and Plan 36c P-C2/P-C8 own migration, deletion and acceptance. ADR-0166 retains its remaining deployment, native coexistence, population and teardown meaning.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of the accepted Plan 36 target.
