---
id: ADR-0070
title: Unify native cache lifetimes and consumed-input identity
status: proposed
date: 2026-09-17
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-20, DM-26, DM-29, DM-31, DM-32, DM-33, DM-39, DM-43, DM-59]
blueprint: [§3.3.3, §5, §14.3, §18.8, §21]
review: docs/design_review/reviews/design_review_datafusion-delta-caching_2026-09-17.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: Native cache or snapshot interfaces change ownership, reset, validation or accounting guarantees at the pinned dependency revision
verification: just adr-lint; just family-check; docs/plans/09-native-caching-and-pivot-completion.md Verification
---

# ADR-0070: Unify native cache lifetimes and consumed-input identity

## Context

The caching review finds repeated planning, replay and decoding, runtime resource
settings in semantic identity and unaccounted native caches. Plan 09 integrates their
replacement with the remaining schema-first hard pivot.

## Scope

ADR-0074 and Plan 11 govern the integrated lifetime, evidence-preservation and
implementation-testing amendments. This proposed record retains its historical
evidence; its prior per-boundary or whole-stage mechanisms do not restrict that target.

Refine ADR-0068/0069 and the resource contract of ADR-0046 for existing functions.
Record the required blueprint amendment through the decision/design PR route; this
record does not claim acceptance of those proposed records or edit accepted ADRs.
The authorized implementation contains no predecessor-data compatibility period.

## Drivers

Preserve exact selected inputs and actual implementation ownership while avoiding
repeated native work. Account retained and in-flight cache resources without making
a cache another durable authority. Preserve cancellation, current admission and
independent process-model outcomes.

## Options

1. Keep full runtime settings in semantic identity: rejected; deployment resource
   changes spuriously alter native dependency and retry identity.
2. Persist opaque cache results or serialized snapshots outside publication: rejected;
   hides effects and competes with Delta log authority.
3. Native disposable caches with explicit lifetimes and consumed-input evidence:
   selected, with the common provider boundary admitting every use.

## Outcome

Enumeration identities are reflected once per declaration in
`reference.schema_enum_types`; `reference.schema_enums` remains the member relation.
References to an enumeration type target the declaration key, never the repeated
type column in the member relation. Both views derive from the registry's sole
enumeration declarations. This closes the target-key ambiguity found by complete
native publication admission during Plan 09 qualification.

Separate semantic settings and policies from resource-only configuration, retaining
complete diagnostic read-back and enforcing the current budget on every execution.
Distinguish invocation/attempt identity from an owned implementation generation;
unknown opaque implementations retain conservative fresh-execution requirements.

Use native cache contracts, reserved capacity envelopes for fixed native value types,
and reservation-owned snapshot/resident entries. Namespace file metadata by actual
store binding. Prepared strata use stable epoch sources and native plan reset with
explicit eligibility; no per-round fallback engine is retained.

Declare algorithm consumption and derive native dependency comparisons from actual
projected inputs, key/membership/multiplicity and absence obligations. Exact selections
remain provenance. Field projections cannot waive current validation or prove unknown
implementation identity. Delta remains the sole durable authority; checkpoints and
checksums are admitted native effects. Native maintenance fences protect cross-process
cache validity before destructive work. Snapshot/CDF accounting gaps are addressed at
the native dependency interface, not with copied replay logic.

### Consequences

Version changed settings/receipt/consumption contracts directly. Remove superseded
per-round planning, redundant fresh opens and duplicated cache policy. Propagate
resource ownership through providers, streams and Python handles. Remote byte-cache
routing is a declared seam; remote deployment and new simulator workflows stay out.

### Compensating controls

Keep unknown settings semantic, validate cache keys and load capabilities, refuse
unqualified reset/recovery, and retain read leases during actual consumption. Capacity
and retained bytes are separate from allocator/RSS measurements. Cache misses execute
the same admitted native path, never legacy code.

### Confirmation

Plan 09 K01–K11 and inherited Q01–Q14 cover identity, accounting, reset, selection,
recovery, lifetime and independent results. Exact-pin sources are recorded in its
planning evidence. No integration or performance acceptance is claimed; the full
campaign follows all implementation and deletions.

## Pros and cons

Native preparation and retained immutable inputs remove repeated work without a new
model authority. The cost is explicit cache lifetime, resource and invalidation proof,
plus narrow reproducible Delta interface changes where public seams are missing.

## More information

- [Plan 09](../plans/09-native-caching-and-pivot-completion.md)
- [Caching review](../design_review/reviews/design_review_datafusion-delta-caching_2026-09-17.md)
- [Pinned planning evidence](../design_review/evidence/native-caching-pivot-plan-2026-09-17.json)

## Status history

- 2026-09-17 — proposed before implementation under the maintainer's authorization of Plan 09; interface evidence only.
