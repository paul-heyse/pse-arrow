---
id: ADR-0180
title: Bind result demand to exact rows and authoritative acknowledgments
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-07, DP-09, DP-18, PS-10]
blueprint: [§19.2, §19.3, §20.6, §21]
review: docs/design_review/reviews/design_review_plan36-result-demand_2026-10-11.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A populated selected-output workload requires a different registry-owned ordering or mandatory terminal consumer that the explicit demand and original-row contracts cannot represent.
verification: Exact typed scalar-row selection and original field predicates, demanded empty versus interrupted export, full/compact studies, actual ordered acknowledgment coverage and lost-ack settlement, authoritative close/seal, protected slim completion and retired/expired refusal, bounded horizon role history and saturated telemetry.
---

# ADR-0180: Bind result demand to exact rows and authoritative acknowledgments

## Context

Plan 36 P-B4–P-B6 selects explicit result demand, row-shaped indexing, actual acknowledgment reuse and bounded live horizon state. Current per-field scalar cells make transport depend on field count; full inventories and reports are assembled even for narrow completion consumers.

## Scope

Amend blueprint §19.2/§19.3/§20.6 and the generated Rust/Python study demand surface under accepted RC08–RC11. Original completion, scientific assessment, candidate permission, exact source/start provenance and required synchronous durable return retain their contracts. This selects no optional-publication policy or general cross-output query product.

## Drivers

Studies need compact summaries as well as explicitly requested full detail. Narrow reads and live executors should carry their actual required state. Exact original rows remain scientific authority through selection, publication, uncertainty and retirement.

## Options

Select one typed row locator per indexed scalar row and explicit demanded coverage. Keeping per-field copies repeats transport and verification work. Globally sorting every relation merely to simplify an index changes unrelated ordering. Dropping detail silently, treating absent appends as empty or constructing acknowledged totals from producer inputs would lose required meaning.

## Outcome

Registry-owned scalar row locators bind exact attempt/set, relation-owned typed primary-key tuple, original block and row offset. Owner/partition selection resolves that original row independently of requested field. Supported-field, finite, missing and bounds predicates execute on decoded original columns with Arrow kernels; readers verify locator/key/row/field association. No Float64 surrogate represents arbitrary composite keys. Signed zero, exceptional bits, units, basis, quality, missingness and stored relation order remain exact. Dense trajectory descriptors retain per-symbol original ranges/time bounds and may share a payload block.

Payload-shaped packing respects each self-contained IPC bound, bounded transport metadata, message envelope and accounted scratch. Bounded append batching retains each block's individual identity/receipt. The close contract advertises requested materialized relations, declared requested empty relations and unrequested detail distinctly. Missing export cannot become empty. Registry authority supplies empty schema, while close supplies exact demanded membership. Mandatory original completion, assessment/usability, diagnostics, start/source/run/attempt and demanded summary remain present. Study demand explicitly selects compact summaries/KPIs, named relation/member detail or full occurrence reports; existing full consumers migrate explicitly. Failed/flagged detail also has bounded selected demand.

The attempt publication owner accumulates actual authoritative append acknowledgments with bounded per-set ordinal/row/digest state, exact demand/empty membership, completion coverage and eligible seed/progress coverage. Duplicate settlement advances only once for the same immutable request. Server admission checks every advertised set/count and demanded coverage against frozen canonical state, not a guessed client total. Close acknowledgment carries actual recovered closure authority; original attempt generation cannot substitute for it. Normal return uses the actual seal acknowledgment and digest-acknowledged original completion. Uncertain append/close/manifest/seal or restart retains exact receipt settlement and reconstruction/comparison. Heartbeat stays owned through reconciliation, then stops/joins with its outcome inspected before terminal admission.

Key-only completion properties use fresh exact terminal/revision/result-retirement permission, fetching only original completion coverage and bounded digest-verified chunks. They need neither seed discovery nor manifest descriptor transport. Expired or retired permission cannot revive; copied bytes from an already completed read do not authorize later storage access. General result readers retain their full exact-manifest contract.

The hash and coverage authorities are explicit. Operations computes the unchanged derive-key framed batch-selection digest only from opaque actual append acknowledgments or exact immutable settlement. The accumulator has no public constructor from caller totals or serialized descriptors. The server validates complete set membership, contiguous extents, interpretation and exact final receipts under close authority. SurrealDB's plain-string BLAKE3 builtin is not the existing derive-key framed hash and cannot replace it. This trusted-writer contract does not claim integrity against arbitrary raw-database mutation; uncertainty and recovery independently reconstruct and compare the existing digest from immutable batch records.

Horizon execution retains current estimator and last accepted result/seed per role, advanced background factor/result, current candidate permission, priors/anchors and the rolling measurement/input window. General staged historical consumers retain their distinct history contract. Completed rows/seed chunks stream privately under one attempt fence and freeze one coherent final demanded outcome. Explicit full history materializes under admission. Ordinary iteration telemetry samples/coalesces without blocking or cancelling science on overflow, retaining exact dropped/partial counters and bounded accounted state; strict-audit interruption is an explicit separate policy. Mandatory science and seed permission still have their existing completion authority.

**Plan46 RC09 target amendment, 2026-10-18:** native callbacks capture bounded selected
physical samples, with checked encoding at existing supervision/publication. Per-sample flush
remains the initial/default boundary. Any qualified bounded batching has explicit finite
visibility/flush limits and required event/schedule/attempt/selection/terminal/ACK boundaries.
Original ordinals and actual contiguous acknowledged prefixes remain authoritative; unknown
append cannot advance Recorded. The supervisor retains both channel-envelope ownership and
its selected-lineage/projection scratch through queue drain, independently of subscriber
drop and native completion. Production keeps its one charged decoder lane; waiting
acquisition precedes decode storage admission. Independent reader controls do not qualify
a new parallel backend. Frozen selection, integrity and escaped Arrow ownership remain intact.

### Consequences

Migrate registry declarations, generated Rust/Python contracts, study admission/executors, cursors, durable publication, output readers and occurrence-report consumers together. Remove scalar-cell production/schema/indexes and schema-only empty appends only after all consumers move. Replace redundant normal reconciliation/reopen only after actual acknowledgments and server coverage admission exist. Preserve uncertainty readback. Do not retain unselected full RunResults merely to service compact demand.

### Compensating controls

New schema/artifact activation is an explicit separately reviewable operation with writer fencing, original uncertain-effect settlement, preserved reachable history/borrowers and old-oracle comparison. No production cutover, history deletion or raw-material retirement is authorized here. Target controls distinguish empty from incomplete, original result from rerun, lifecycle terminality from scientific permission, and returned copied ownership from live store permission. Partial implementations retain their actual acceptance limits.

The offline adapter preserves exact recognized V3 run, attempt, result, study and analysis records and their source/root associations. It derives typed row locators from the original IPC through the existing registry-bound decoder and projection owner. An immutable `canonical_historical_results` admission binds each supported sealed terminal attempt to its original run/revision/sequence/generation, manifest/digest/outcome/interpretation and exact issuing store, migration and frozen snapshot. Historical read permission requires that admission plus the existing current protection and retirement checks; it grants neither new execution nor seed eligibility. Known terminal incomplete prefixes remain preserved as incomplete, without fabricated manifests or completion. Unknown, mutable or undecodable history refuses activation. Server write operations independently require the current interpretation even when an old caller supplies an old interpretation. Derived pages and original snapshot verification precede the final marker switch; interruption resumes the same migration identity.

### Confirmation

**Implemented / scoped Tested, 2026-10-18 (Plan46):** prepaid selected capture with supervisor encoding, original native ownership, authentic per-sample flush/ACK, dropped/unread cursor lifetime and preparation cancellation; cross-sample batching remains unselected.
[Plan46g Outcome](../plans/46g-investigations-qualification-and-design-publication.md#outcome)
owns exact selected native/canonical/compiled commands, initial failures and affected repairs.
This is affected-scope acceptance, not a new whole-product or universal-performance claim;
formal ADR status remains proposed.

Proposed: the approved reconciled plan and its investigation evidence select these contracts. The narrow completion source slice is written and Operations cfg-test compilation checked; Runtime controls and integrated consumer acceptance remain open. No performance measurement or full packet acceptance is claimed.

## Pros and cons

Explicit demand, original-row indexing and actual acknowledgments reduce unnecessary movement and repeated discovery while preserving one scientific authority. They require coordinated schema/consumer migration and complete close coverage; omission cannot serve as a shortcut.

## More information

[Plan 36b](../plans/36b-studies-coordination-and-results.md) owns packets, removals and current evidence. [Plan 36d](../plans/36d-relational-and-python-boundaries.md) owns owned IPC and Python boundary integration. ADR-0173 retains complete-original seed eligibility and ADR-0177 supplies exact creator/retirement conflict safety.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of the accepted Plan 36 target; dependent declaration/consumer migration remains pending.
