---
id: ADR-0196
title: Declare immutable diagnostic evidence views
status: proposed
date: 2026-10-17
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, PS-10]
blueprint: [§23.1]
review: docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A selected evidence query cannot preserve event membership, explicit unknowns or finite ownership under this declaration.
verification: Plan43e immutable evidence-view controls for exact source event keys, repeated and missing records, nonfinite/availability, admitted buffers and independent export failure.
---

# ADR-0196: Declare immutable diagnostic evidence views

## Context

Plan43e requires queryable retained evidence without mixing values from different native events. Existing source JSONL and diagnostic capture own raw evidence and retention; a derived Arrow view supplies analysis, not scientific publication.

## Scope

Add derived diagnostic field, invocation and capture relations within blueprint §23.1. The maintainer delegated complete Plan43 choices; formal ADR acceptance remains separate.

## Drivers

Queries need exact operation/producer/generation/event membership, field ordinals, units, availability and coverage. Live observation must remain independent from analytical export and scientific storage.

## Options

Inferring schemas or flattening only numeric values loses meaning. A second persistent diagnostic store duplicates ownership. Select one registry-declared Arrow projection over existing immutable source pages, using existing allocation leases and DataFusion table capabilities.

## Outcome

`runtime.diagnostic_evidence_fields` declares the field projection. `runtime.diagnostic_invocations` and `runtime.diagnostic_captures` preserve the actual joined invocation and optional capture documents, their endpoint/coverage meaning and exact operation links. Event and field ordinals preserve membership; zero field ordinal preserves an empty event. Typed values and explicit unavailable/nonfinite categories remain distinct. Source pages remain the raw authority, and repeated event identities refuse this projection rather than silently deduplicate them.

### Production-completeness correction — 2026-10-17

**Implemented / Tested, 2026-10-17:** Launch captures extend the existing capture projection with an explicit process subject and optional operation link; the derived key identifies the actual operation or launched process without inventing a diagnostic operation before execution publishes one. Numerical diagnostic exports retain optional originating invocation/attempt/failure context. These references preserve interpretation only and do not grant scientific use, replay, cancellation or publication permission. Changed view/document versions migrate all actual consumers through the registry and existing generators. Selected functional checks and public consumers are recorded in [43g’s remediation Outcome](../plans/43g-qualification-and-design-publication.md#production-completeness-remediation-2026-10-17); formal proposed status remains separate.

### Consequences

Rust/Python readers select finite pages and finite analytical memory. Escaped Arrow buffers retain their admitted lease; DataFusion consumes the same schema and buffers. Views preserve partial coverage and cannot upgrade scientific permission or imply a complete history.

### Compensating controls

No new scheduler, store writes, SQL parser or charge ledger. Exact times remain seconds/nanoseconds. Export failure leaves raw evidence and target science intact; queries choose compatible families, products and outcomes explicitly.

### Confirmation

**Tested, 2026-10-17:** actual registry-derived Arrow/DataFusion views preserve source keys, nullable unavailable fields, exact unsigned sequences, leased lifetime, repeated-key refusal and memory/independent projection failure. Actual Python and Trace Processor consumers passed. Plan43e/43g Outcomes own selected/composite conditions; derived views remain observational and do not establish causal certainty or scientific validity.

## Pros and cons

The projection enables ordinary Arrow/SQL analysis with one authority. Field rows require explicit event-key joins, which preserve the original coherent boundary instead of guessing simultaneity.

## More information

[Plan43e](../plans/43e-diagnostic-analysis-artifacts-and-profiling.md), [Plan43](../plans/43-solver-diagnostics-and-performance.md).

## Status history

- 2026-10-17 — proposed before dependent relation implementation under delegated execution authority.

## Plan47 accepted target extension

The maintainer accepted RC01/RC02 and authorized 47a/47b execution on 2026-10-18.
Frozen evidence uses recorded source time and explicitly supplied immutable membership;
live inspection retains current-clock discovery. Descriptive context remains available
without Replay reconstruction admission. Bounded versioned data-only readback recreates
neither native permissions nor allocation ownership. Original scientific qualification,
unknown clock relationships, optional capture loss and per-product availability remain
independent. Targeted implementation and acceptance are owned by Plan47a/47b; the later
companions retain assembled publication/CLI/qualification scope. Formal ADR status is unchanged.

**Implemented / Tested — 2026-10-18:** the [47a Outcome](../plans/47a-frozen-evidence-and-review-context.md#outcome)
and [47b Outcome](../plans/47b-native-diagnostics-and-causal-attribution.md#outcome) record
selected frozen/context/membership/native delivery, bounded controls and corrections.
Formal status remains proposed; later analysis, pack, CLI and assembled acceptance remain
with the active coordinator. No broader scientific or performance qualification follows.

## Accepted Plan47c–e execution target

**Proposed — 2026-10-18:** maintainer-authorized C–E implementation extends §23.1 with
member-namespaced ten-question analysis over immutable owner facts, copied engine reports,
explicit preconstruction source activation/acknowledgement, contextual bounded queries and
selected sealed offline review packs. Detail output and frozen-set v2 replace their v1 codecs;
DetailRequest remains v1. Complete zero-issued families require actual applicable activation.
Pack publication, membership/context completeness, capture and scientific outcome remain
independent. CLI reads never run science or discover live neighbours; explicit diagnostic work
establishes its finite deadline before source admission/preparation. Existing tools with stated
limits are selected; no Samply source/version correction or extra profiler platform is adopted.
The C–E companions own targeted acceptance; 47f owns assembled qualification. Formal ADR status
and numerical stopping policies are unchanged.

### Plan47 completeness repair

**Implemented — 2026-10-18:** CA01–CA12 retain the accepted §23.1 boundary while replacing the
analysis and pack envelopes with v2, and source-recording request/ACK/launch-result envelopes
with v2. DetailRequest/launch-request remain v1. Actual performed physical sessions supply bounded
copied policy/plan/metric facts; source maps use factual admission and original coordinate
correspondences. Analysis owns admitted preparation/output and independent clock selections.
Optional selected producer handoff retains original joined/pre-handle facts under its independent
selection/receipt; absence adds no retention. Shared publication composes multiple protected
scientific selections and registry diagnostic IPC, resolves actual supported source pointers,
and assesses question/product-dependent explanatory closure before sealing. Human HTML uses
local references and bounded retained orientation. These repairs add no scientific evaluation,
accuracy change, environment proof or new profiler platform. The coordinator and companions own
focused acceptance; 47f retains broad assembled qualification. Formal ADR status is unchanged.

### Plan47f supplier and export correction

**Implemented — 2026-10-18:** explicit standalone initialization/recycle context is copied
from the original prepared owner and joined report, with its actual run/operation and no
synthetic RunResult invocation. Default execution retains no context copy; optional copying
failure preserves the scientific result. Escaped copies retain their finite allocation.
Table publication compares the selected typed original source with its finalized frozen
member before projection, avoiding JSON trees for unrelated attachments. Frozen document
ingress and Arrow projection allowances remain independent and do not establish total private
peak memory. Plan47f owns assembled acceptance and actual measurements; formal status stays
proposed and scientific defaults are unchanged.

**Implemented — 2026-10-19:** selected dynamic context also retains the original joined
native segment statistics with run identity and native reset scope. This preserves original
getter populations when the compact live metric catalogue reports curtailment, without new
native queries. Native-excluding handoffs reject these actual payloads regardless of a supplied
selection header. Unsupported standalone context is an independent unavailable copy after
mandatory scientific result construction. Plan47f owns the actual acceptance and limits.


**Tested — 2026-10-19:** [Plan47f assembled acceptance](../plans/47f-qualification-and-design-publication.md#assembled-acceptance-boundaries--2026-10-19)
records original source/context/counter comparisons, actual cooperating native CLI publication
and relocated offline readback, separately placed managed inventory and deterministic active
native-workflow reader cancellation. Nine short observer observations retain original scientific
qualification and actual counters; finite growth/skew retains independent failure/coverage
semantics. Descriptive copy bounds count selected fields/physical IPC rather than broader
retained owners; Replay admission and general numerical policies are unchanged. The Outcome
owns final static/manual results and limits. Formal ADR status remains proposed.
