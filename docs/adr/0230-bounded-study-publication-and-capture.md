---
id: ADR-0230
title: Own resumable study installation and immutable cohort capture
status: proposed
date: 2026-10-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-04, AP-07, DP-01, DP-04, DP-09, PS-10]
blueprint: [§19.3, §20.1, §20.4, §20.5, §20.6]
review: docs/design_review/reviews/design_review_product-and-workload-ownership_2026-10-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Pinned engine capture cannot preserve active cohort facts or bounded installation cannot retain original authority.
verification: Plan56d exact append settlement, interrupted population pages, concurrent immutable capture, protected lazy cohort reads and 10000-point creation/summary/cohort qualification.
standard: core-3.4/process-simulator-1.5
scenarios: [docs/plans/56d-publication-and-workload-fit.md#acceptance-and-benefit-limits]
---

# ADR-0230: Own resumable study installation and immutable cohort capture

## Context

DA findings scheduled in 56d identify repeated append guards/frontier work, per-point run creation,
descriptor-heavy summary reads and small full-materialization cohort bounds. Existing canonical
settlement, inline/sealed placement and scientific permission are preservation constraints.

## Scope

Own the new durable installation/capture relation families within existing canonical authority.
D01–D06 remain the execution scope. Publish pse.substrate.v10 once with all current
writers/readers, retention and backup classifications before activation on new owned disposable
state. Preserve prior stores and evidence. No compatibility path or numerical algorithm changes.

## Drivers

Bounded pages must resume exact installed work after interruption without renewing issued authority.
Summary work should consume compact outcome facts. A live cohort must retain one coherent captured
fact set, including unavailable members, without holding a remote transaction or full snapshots
across page reads. Publication and export lifetimes must follow their actual physical effects.

## Options

A transport array calling the single append function repeatedly does not remove shared guard and
frontier work. Use native bulk INSERT/INSERT RELATION and grouped set/frontier realization under the
existing exact owner. Live keyset paging can mix times; terminal-only capture weakens supported live
semantics. Long remote transactions pin global engine versions and broaden cleanup obligations.
Select a short engine-native immutable compact capture under pinned serializable snapshot isolation,
preceded by a committed publication gate and followed by bounded reads. The ungated 10,000-member
surrogate capture actually failed with a read conflict under active writes; FOR UPDATE is optimistic
and cannot supply blocking exclusion. A scoped gated surrogate captured all 10,000 facts coherently,
including unavailable members and protection references. Repeated optimistic retries can starve;
a caller-held remote snapshot pins global versions and broadens cleanup. Read-only staging still
needs protected publication and larger movement. A scoped gate is the simplest established complete
route. The pinned mx controls patch and installed adapter source, rather than generic online
isolation documentation, supply these premises.

## Outcome

Installation owns immutable study/population identity and digest, generation, phase and exact node/
edge frontiers. Bounded run-and-point pages atomically acknowledge their exact immutable request;
replay conflicts refuse. Separate edge closure and activation prove complete population. Expired
issued work and uncertain effects retain original clocks, fences, cancellation and readback causes.
A resumed invocation does not recreate run identities or imply a renewed old page grant.

A scoped publication gate commits before capture reads. Every answer-affecting mutation checks the
same guard inside its native writable transaction before effects: selector universe (including new
matching studies), membership, compact facts, attempts/manifests, availability and retirement.
A per-study guard alone does not protect an open multi-study predicate. Pre-gate writers either
commit first or lose SSI validation. The gate binds exact incarnation, immutable request and original
bounded grant. Capture authenticates these, atomically publishes facts/protections/exact receipt and
fences/closes the gate before paging; no global exclusion spans user reads.

Busy publication is definitely pre-effect only after native refusal/rollback establishes that fact.
Waiting or exact resubmission consumes the original clock/cancellation and retains already computed
numerical output; it never reruns science or renews a grant. An expired timestamp never permits
writers simply to bypass an unchanged gate. Exact recovery changes/closes the checked guard
incarnation before reopening, forcing stale in-flight capture to conflict (or an equally strong
fenced publication). Lost ACK, cancellation, expired recovery and cleanup keep attribution and
unknown-effect knowledge. There is no new generic lease manager or automatic thaw.

Admission publishes compact outcome projections consumed by summary/finalization observers.
Cohort capture creates immutable headers, member facts, unavailable classifications and selected
attempt/manifest references in one coherent transaction. A compact capture receipt and bounded
keyset pages replace complete documents. One protection owns captured manifests through lazy shared
relation scans and release; later completion cannot reclassify captured unavailable members.
No per-member full snapshots or DataFrame union chain remains in that provider.
Capture identity binds the complete immutable selection request. Capture facts and all selected
manifest protections are installed atomically in the same native transaction (or consumed from an
already-held protection); no commit-to-protect retirement gap is permitted. Exact replay returns
the same receipt; changed content under that identity refuses. Lost acknowledgement settles by
exact readback under the original grant. Idempotent release authenticates that capture owner;
expiration/abandonment cleanup drops only its references, after active scan custody drains.
Installation/capture declarations explicitly classify backup and retention, including interrupted
installs and abandoned captures. They are recovered canonical state where required for exact
resumption; cleanup cannot infer absent authority merely from a disconnected caller.

Native offline copy progress is bounded, typed and bound to the original frozen inventory and
checkpoint premises. Progress is not final acknowledgement. Explicit resume revalidates premises;
stalled work is diagnosed without thaw or renewed scientific authority. Preserve distinct source,
copy, destination and semantic restore integrity purposes while reusing identical immutable facts.
Selected exchange holds compact exact selection/protections and admits, encodes and releases one
useful revision/document at a time. Export and checksum/count explanation consume the same traversal;
allowance precedes avoidable copy and the manifest publishes last.

### Consequences

All release/retention/backup consumers must understand capture ownership and new installation
facts. Existing study ceilings stay until exact-engine evidence and bounded real-workload controls
justify change. Qualify 10000 points end to end; 100000 remains an unqualified admission ceiling.
Streaming queries are not selected: the pinned SDK rejects payload-owned streaming queries.

### Compensating controls

Exercise exact replay/conflict/lost ACK, stale fences, empty/partial demanded appends, interrupted
node/edge pages, refused variants and incomplete activation. Mutate live facts between captured page
reads, hold readers through retirement, cancel scans and check release/version retention. Qualify
private recovery separately from shared test contexts. Measure affected physical work at scope end.

### Confirmation

Proposed before dependent implementation at c5ae90b84. The focused review assesses the contract;
56d owns exact-engine feasibility, implementation, functional acceptance and benefit measurements.
The original ungated mechanism failed and remains unselected. Independent review accepted the
revised gated target at Proposed design evidence after inspecting both probes. Surrogate controls
are not canonical production qualification; the complete 10,000-point journey, expiry races,
retirement, genuine unknown ACK and scan drain remain with D04 before any cap increase.

## Pros and cons

Short capture bounds transaction lifetime and permits coherent lazy pages. It introduces durable
capture lifetime and explicit release obligations. Compact metadata does not establish bounded
resident execution unless all provider and export consumers release their physical units.

## More information

[Plan56d](../plans/56d-publication-and-workload-fit.md) owns delivery;
[Plan56](../plans/56-design-principle-alignment.md) owns finding dispositions.
ADR-0227 retains scientific permission; ADR-0228 retains exact settlement and placement.
The supplied SurrealDB, SurrealKV, SurrealMX and tooling references inform capability discovery;
exact pinned source and the active-concurrency probe settle consequential engine claims.

## Status history

- 2026-10-26: Proposed for authorized joint 56d/56e execution; formal acceptance remains separate.

- 2026-10-26: Retained failed ungated SSI evidence and selected bounded gated capture after independent reassessment; production qualification remains open.
