---
id: ADR-0252
title: Query original developer evidence through ephemeral typed relations
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, DP-01, DP-02]
blueprint: [§6.13, §21.7, §23.1]
review: "not-required: short relation-family decision within existing Runtime diagnostic and Arrow publication contracts; authorized Plan66 E07/F04"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A selected consumer requires persistent analytical state, inferred scientific membership, or action authority from a query answer.
verification: Existing evidence-reader queries join and group explicit original report members while preserving exact counts, null/empty/unknown coverage and source attribution; changed commitments and conflicting identities refuse without changing native verdicts.
---

# ADR-0252: Query original developer evidence through ephemeral typed relations

## Context

The independent host verifies generated passive workflow observations against exact original
report bytes and terminal commitments. Its public acquisition summary omits action, outcome and
diagnostic causes needed to explain patterns across original invocations. Existing scientific
invocation relations identify runs, attempts and source clocks that developer reports need not
contain.

## Scope

Extend blueprint §6.13/§21.7/§23.1 with an ephemeral derived developer-evidence relation family
within existing Runtime diagnostic and Arrow publication contracts. Developer members are
explicit supplied inputs, separate from scientific `FrozenEvidenceSet` membership. Its ten
native analysis questions retain their meaning. Root integration owns architecture publication
and the formal decision route; proposed status claims neither acceptance nor qualification.

## Drivers

Explain repeated prerequisite refusals and failure/capture/recorded-settlement patterns across
a finite original population with exact source references. Reuse source-owned observations
and the existing evidence reader. Keep the independent host Arrow/DataFusion-free, native
verdicts authoritative, and unknown observations distinct from zero.

## Options

Retain single-member readback for acquisition inspection. Overloading scientific
`diagnostic_invocations` invents run/attempt/clock meaning; another JSON field map repeats
native interpretation. A durable verdict database adds lifecycle and invalidation obligations
without helping this finite query. Select native verified facts, declared typed relations and
DataFusion joins/grouping through the existing observer client.

## Outcome

Declare distinct derived `runtime.developer_invocation_facts`,
`runtime.developer_diagnostic_causes` and `runtime.developer_prerequisite_facts` relations.
Join on exact invocation identity plus original
report SHA-256. Invocation rows retain original action/outcome, interpretation, nullable selected
count, required capture coverage and recorded-at-read settlement/pending state. Cause rows
retain a projection row ordinal, typed diagnostic code/class and the exact original field/ordinal
pointer; their projection order is not a source clock or causal order. Every answer retains
supplied-member attribution, including invocations with empty cause populations.

The optional prerequisite projection reads only the existing typed `pending` field from the
same authenticated original JSON bytes. Missing is unknown; a present empty list is known empty.
Present null, duplicate fields, unsupported native states and populations exceeding the finite
projection allowance refuse. Reuse passive `Pending` and fixture input observations without
deserializing `Report`, private group verdicts, current readiness or native permissions. Native
capability and fixture-input rows retain exact original pointers, input state and diagnostic code;
native capability arrays retain the source serializer's unique sorted order rather than
normalizing a different array and changing its original index attribution.
Empty capability/input collections retain their original pending entry. No embedded path or
recorded preparation argv is followed.

The existing host report-adoption owner supplies passive facts after descriptor/interpretation,
original report/terminal and selected inventory checks. Reuse its generated
`WorkflowReportObservation`; do not create a second native verdict model. Explicit reader inputs
select verified developer members alongside optional supplied scientific context. No embedded
path is followed and no scientific membership is inferred. Missing producer generation, source
clock, selected count, omission account or settlement remains nullable/unavailable; do not
substitute blank identifiers, zero, reader time or current generation.

Use Arrow 59.3.0 and DataFusion 55.1.0 `MemTable::try_new`,
`SessionContext::new_with_config_rt`, `register_table` and `sql`. The maintained initial query
left-joins original causes and groups/orders action/outcome/cause/capture/settlement patterns,
with explicit original member references. A separate maintained prerequisite query preserves
known-empty and unknown coverage and groups original pending inputs/capabilities. It does not
cross-product causes with prerequisites or change the first question. Query selection is finite
and maintained; callers
cannot submit arbitrary SQL or discover external tables. Small semantic folds and numerical
history/comparability policy remain native. No answer grants preparation, rerun, cancellation,
current physical control, producer authenticity or scientific permission.

### Consequences

The registry declares columns/nullability once and normal generation publishes its artifacts.
Reuse the existing Arrow projection and observer allocation ownership, retaining leases through
escaped buffers. A query-local session uses that same pool through
`RuntimeEnvBuilder::with_memory_pool`, with temporary disk disabled. Bound new parsed entries
at their source projection and reserve known live Arrow/string and answer populations before
construction through those existing owners. These admission accounts are not allocator/RSS
bounds or a canonical reconstruction guarantee. Input/row/response limits
bound this observer operation without imposing host workload caps. Release session/tables with
acquired evidence; add no persistent analytics or verdict cache.

### Compensating controls

Challenge changed commitments, conflicting duplicate original identities, unsupported
interpretations, multiple/empty causes, partial capture and recorded-versus-current settlement.
Test exact UInt64 above binary64 precision, missing versus zero and unavailable independent
clocks. Historical unavailable facts remain unavailable. Retain original observation limitations
beside query results and refuse before publishing invalid tables.

### Confirmation

Interface-checked: current host original-report adoption, passive generated observation,
scientific invocation relation and Arrow/MemTable seam were inspected. Context7 supplied
orientation; installed DataFusion 55.1.0 source establishes session, registration, shared-pool and
disabled-disk APIs. Plan66 E07/F04/F06 own implementation and actual original-invocation query
acceptance. This decision precedes that implementation and claims no measured benefit.

## Pros and cons

Typed tables reuse the query engine without duplicating native meaning. Three explicit schemas
and an observer-only reader route add integration work. Separate developer/scientific inputs
require explicit caller selection when a review needs both, preserving their actual identities.

## More information

- [Plan66 data/results](../plans/66e-protected-data-and-result-delivery.md),
  [developer clients/evidence](../plans/66f-developer-operations-clients-and-evidence.md)
- [Original adoption](../../crates/pse-host/src/developer_evidence.rs),
  [native report observation](../../crates/pse-host/src/test_workflow/execute/report.rs)
- [Diagnostic registry](../../crates/pse-schema/src/catalog/diagnostic_evidence.rs),
  [Arrow projections](../../crates/pse-runtime/src/tooling/capture/evidence.rs),
  [evidence-reader investigation](../../crates/pse-runtime/src/tooling/reader/investigation.rs)
- [DataFusion 55.1.0 SessionContext](https://docs.rs/datafusion/55.1.0/datafusion/execution/context/struct.SessionContext.html),
  [RuntimeEnvBuilder](https://docs.rs/datafusion-execution/55.1.0/datafusion_execution/runtime_env/struct.RuntimeEnvBuilder.html),
  [MemTable](https://docs.rs/datafusion/55.1.0/datafusion/datasource/struct.MemTable.html)

## Status history

- 2026-11-06 — proposed before the authorized Plan66 E07/F04 derived relation implementation.
