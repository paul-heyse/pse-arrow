---
id: ADR-0211
title: Separate bounded worker compute from joined publication completion
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, DP-10, DP-19, PS-10]
blueprint: [§19.3, §20.6]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: Publication ownership cannot be bounded through actual ACK/error and drain, or supported same-worker refill introduces an admission, fairness or teardown regression.
verification: Hold actual canonical export on one width-one managed worker and require its next independent occurrence to reach native completion before release; verify finite claim/product bounds, publication backpressure, uncertain/lost ACK, late export failure, cancellation and complete owner drain without premature terminal state.
---

# ADR-0211: Separate bounded worker compute from joined publication completion

## Context

The existing worker returns a scientific lane only after its durable run handle has finished
export and ACK. Other independently submitted native work can proceed, but the same worker
cannot refill when all its bounded lanes await publication. The earlier sibling-progress
probe therefore did not settle same-worker throughput. Separating those two capacities removes
that scheduling dependency without treating pending publication as completed work.

## Scope

Narrow blueprint §19.3 and §20.6 within Plan48 F06/I30 and accepted RC12. Extend the existing
managed worker scheduler with bounded Running and Publishing states. No detached commit
service, generic queue platform, changed durable fence or public early-result contract.

## Drivers

Let available scientific capacity consume another ready independent occurrence while prior
required publication waits. Preserve explicit source, attempt, heartbeat and uncertainty
ownership and all issued work's actual cancellation/drain. A first-principles removal of
serialization supports adoption without a marginal benchmark winner requirement.

## Options

Joined lane completion is simple but retains the serialization. Raising all worker widths
also permits more simultaneous science and does not represent publication capacity separately.
A new commit service duplicates recovery/ownership machinery. The selected route uses the
existing scheduler with distinct finite capacities and unchanged terminal ACK semantics.

## Outcome

For scientific width W, precharge exact coordination/lane/future/channel storage before claims
and retain at most W Running plus W Publishing owners, with at most 2W total lane owners.
A worker-only handoff becomes eligible only after actual native owning join and original
scientific completion. When a publication slot is available, acknowledge that state and allow
a distinct preallocated lane to start new work. A full publication stage applies backpressure;
it cannot create an unbounded backlog or claim unstarted work beyond the finite inventory.

The existing public RunHandle::wait remains terminal through publication/ACK or typed error.
The old lane, its private preparation permissions, source/result allocation leases, heartbeat,
fence, observation/detail owner and required native closure remain retained by its completion
future. They do not transfer to the replacement lane. Parked native state retains its actual
charges but no active compute permission that prevents the new lane's lawful progress.
Only actual terminal settlement increments completed actions and advances readiness; a
publication handoff does neither. Existing per-member uncertainty and lost-ACK recovery remain.
An idle replacement lane does not suspend ordinary polling/recovery while an older publication
remains live: with free scientific capacity and an available finite lane, the existing poll
interval still discovers later arrivals.

On cancellation or fatal error, stop new claims and cancel every issued Running/Publishing
owner through its existing control. Join all native closure and pending publication/error
settlement before reporting drained. Awaiting publication-slot acknowledgement remains
cancellation-aware and cannot strand a heartbeat or result owner. Failed publication preserves
its original typed effect/ACK uncertainty rather than inventing success or reissuing work.
If cancellation reaches an owner before publication-slot acknowledgement, settle its existing
attempt through the cancellation/inline-completion route, without result/seed/completion payload
publication outside the capacity bound. Retain its original source/heartbeat/fence until that
cleanup returns. Preserve an earlier typed scientific or original-completion failure; a cleanup
failure remains separately observable in the durable record and may remain uncertain.
Loss of the scheduler or capacity-ACK channel without actual cancellation is a distinct
failure. Drain the already-owned progress stream and use the existing fenced export-rejection
route; do not infer cancellation, invent an acknowledged completion or emit a new payload
without a publication slot.

### Consequences

One bounded scheduler can overlap science and required publication without merging attempt
permissions. More results can remain live, so all coordination is prepaid and each result's
existing actual allocation lease remains charged through completion. Memory pressure may
refuse lawful new work; this does not weaken existing admission or terminal truth.

### Compensating controls

Exercise a real same-worker width-one export gate, finite two-owner claim bound and saturated
publication backpressure. Include late export error, cancellation at handoff and publication,
actual native join, failed seal/lost ACK and uncertain effects. Verify source/fence/heartbeat
retention and no terminal manifest or processed-action count before actual settlement.

### Confirmation

**Tested:** both actual canonical/native worker controls passed in the explicitly
force-validated default Nextest profile:
`same_worker_publication_handoff_refills_and_bounds_actual_unacknowledged_attempts` and
`same_worker_publication_refills_a_study_created_after_actual_idle_discovery`.
With scientific width one and the unchanged eligible Ipopt bounded-root fixture, the first
holds Table export while a second scientific/native owner completes, retains exactly two
unacknowledged claims, preserves late Completion failure, and settles a cancelled waiting
owner without payload admission before releasing the first publisher. The second creates
a new occurrence only after actual idle discovery and completes its scientific owner while
the earlier publication remains held. Native service shutdown precedes assertions, and the
owned canonical fixture was quiesced and stopped after the selection.

Pure backpressure/late-arrival/lost-channel controls and
`lost_publication_ack_keeps_original_failure_and_separate_durable_uncertainty` also passed.
The [F06 package Outcome](../plans/48f-scheduling-resources-and-result-completion.md#outcome-recorded-after-implementation)
retains commands, conditions and the failed combined-selection boundary: individual F06
passes do not turn adjacent fitting/dynamics failures into a suite pass. This functional
fixture does not establish a managed-primary deployment journey, universal throughput gain
or completed H02/H03 qualification. Formal status remains proposed.

## Pros and cons

The route removes known serialization using existing owners and recovery contracts. It adds
one finite scheduler state/handoff and bounded retained result pressure, requiring precise
backpressure/cancellation accounting rather than a second publication authority.

## More information

- [Plan48 F06/I30](../plans/48f-scheduling-resources-and-result-completion.md#i30--does-joined-publication-actually-block-useful-compute)
- [Plan48f Outcome](../plans/48f-scheduling-resources-and-result-completion.md#outcome-recorded-after-implementation)
- [Plan48 coordinator](../plans/48-execution-performance-and-numerical-efficiency.md)

## Status history

- 2026-10-20 — Proposed before dependent production changes under authorized Plan48 execution and the operator's deployment-first direction.
