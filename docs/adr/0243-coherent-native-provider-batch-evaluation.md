---
id: ADR-0243
title: Expose coherent native provider batches with attributable point outcomes
status: proposed
date: 2026-11-04
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-10, DP-24, PS-01, PS-13]
blueprint: [§9.11]
review: docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required provider cannot preserve independent point identity, exact demand, physical qualification or native lifetime through the selected grouped operation.
verification: Independent mixed-validity batches, exact scalar/group physical correspondence, malformed extent/identity refusal, cancellation with native drain and real grouped native API invocation.
---

# ADR-0243: Coherent native provider batch evaluation

## Context

Plan65 E05 requires an actual grouped operation, rather than a scalar RPC loop. The existing
pinned CoolProp API supplies PropsSImulti and state-array functions. Their global success
does not establish every point/output succeeded; their partial-result and error ownership
contracts must shape the kernel and worker interface.

## Scope

Extend the existing provider contract with explicitly declared batch support for its exact
prepared demand. The default is unsupported, with no automatic scalar fallback. Initial
water support may be value-only while preserving all original phase, mechanical-response,
datum and h-u-p/rho qualification. Derivative batches require their own actual support/evidence.
The source-owned worker projects the same operation with original handle and batch identity.

## Drivers

Preserve scientific meaning, amortize useful native work, and make every point attributable.
One malformed frame differs from one scientifically refused point. Native work and output
buffers remain owned through completion even when an observer disappears.

## Options

Scalar RPC iteration hides overhead and cannot establish native grouping. Requiring every
provider to implement batching immediately invents unsupported capability. Select an explicit
optional kernel capability and a genuine existing library group operation for eligible demands.

## Outcome

### Consequences

Envelope admission checks framing, identity uniqueness and checked aggregate extents. Exact raw
point values remain representable until per-point scientific admission. Gather eligible inputs
with their original point mapping; an all-refused population need not call the native library.
Per-point results retain success/refusal independently of batch-wide infrastructure failure.
Never automatically replay completed or uncertain points.

Use owned checked output storage. Initialize native slots to nonfinite sentinels and validate
all required slots for each point; native global success is insufficient. Preserve the shared
original water qualification policy rather than copy it into a second batch-specific policy.
CoolProp global error access remains exclusive through call and error retrieval, avoiding a
failure retry that would replay the group. PropsSImulti constructs one state per group rather
than reusing a live prepared state; that tradeoff needs whole-operation measurement.

### Compensating controls

Exercise mixed good/bad points, unsupported demand, duplicate IDs, malformed shapes, nonfinite
slots, datum/phase/closure, original ordering, cancellation and retained admission. Use independent
physical expectations alongside direct/group correspondence. A grouped wire route does not
establish new physics or a speedup.

### Confirmation

The operator authorized the Plan65 target and breaking changes on 2026-11-04. This proposed
kernel/binding record precedes activation; formal acceptance and scientific qualification remain
separate. Plan65e retains precise immutable library-source characterization and execution status.

## Pros and cons

The interface exposes genuine native grouping and honest partial outcomes. It introduces checked
batch storage and demand-specific capability work; global native locking may reduce concurrency.

## More information

[Plan65e](../plans/65e-scientific-portability-and-execution.md) owns E05 delivery and library
sources; [Plan65i](../plans/65i-scientific-operations-and-product-semantics.md) owns scientific use.

## Status history

- 2026-11-04: Proposed before grouped kernel/provider implementation.
