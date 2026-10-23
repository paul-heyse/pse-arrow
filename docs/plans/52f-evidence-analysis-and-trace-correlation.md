---
title: "Plan 52f: Evidence analysis and trace correlation"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, validation, diagnostics, performance, storage]
---

# Plan 52f: Evidence analysis and trace correlation

## Target and foundation

**Proposed:** answer explicit cross-run questions and correlate selected application/database work
without inventing outcomes or causal certainty. Owns O23/O25;
[the coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns I11/RC06/RC09 and state.
Plan51's evidence and Plan40's diagnostic owners remain authoritative.

`pse-operations::evidence_query` has six finite operations over explicitly supplied sources,
parsed once per analysis into registry-owned Arrow/DataFusion relations. `evidence_sources`,
`evidence_projection` and `evidence_store` provide authentic codecs, source cuts and disposable
persistent generations. Neither public arbitrary SQL nor another result ledger is needed.

Plan40 already supplies `pse-diagnostics::observation::OperationId`, a bounded process hub and
one atomic per-process checkpoint image, with diagnostic-only native-child parent locators outside
scientific hashes. Extend correlation at its application/SDK seam; do not rebuild current/recent
history. The [substrate guide](../dev/surreal-substrate.md) establishes server executor→Surrealism
parentage, not application-originating Rust trace continuity. Plan40e's broader assembled
profile/locality campaign stays there.

## Work packages

<a id="F01"></a>
### F01 — Typed baseline/candidate evidence comparison

Extend existing finite evidence questions with explicit baseline/candidate source selection,
subject correspondence and paired outcome/measurement views. Expose new/resolved/persistent
failures, unmatched/ambiguous/renamed subjects, missing execution, repeated mixed outcomes and
selection denominators. Preserve original binary IDs; do not use stable-looking spelling or
nearest timestamps to invent cross-build correspondence. Exact compatible owner identity or an
explicit supplied mapping supports a pair; ambiguity stays unpaired.

Use existing registry/document generators for changed public documents and FND00's applicable
contract slice. Keep host source readers independently usable; no automatic tests, builds,
service startup or raw artifact opens just to compare supplied data. Parse each selected source
once and perform relational work in one selected engine context. Persistent queries use the same
facts/cut through existing discovery, with explicit stale/missing populations and rebuildability.
Do not widen native EvidenceSnapshot codecs into arbitrary knowledge notes.

For measurements, retain sample membership, pairing/order and relevant input/start/cache/build/
provider/capture conditions. Incompatible or unavailable associations remain so; a distribution
cannot repair confounding or prove causality/flakiness. Existing unsupported Criterion-to-invocation
membership remains unsupported unless an actual producer supplies it. Use current SciPy only for
a declared analysis when useful, not an automatic statistical verdict.

Acceptance: authentic sources include failure→pass, persistent failure, unexecuted subject,
renamed/ambiguous subject, changed selection and mismatched measurement context. Independently
specify expected pairs/denominators and show an inconclusive noisy cohort. Offline server-down
analysis and indexed same-cut results agree; duplicate import does not double outcomes. G05's
operator view consumes this contract. Delete replaced heuristic comparison parsers once their
callers move; historical source results remain unchanged.

<a id="F02"></a>
### F02 — Finite application-to-server trace seam

Resolve I11 against the exact custom server and pinned SDK/transport. Begin with the existing
operation/invocation identities and finite `surreal_server.py` trace capture. Inspect supported
context propagation, request metadata/header and reconnect behavior at the actual SDK bridge;
server OTel support alone does not establish application instrumentation. Associate diagnostic
operation IDs with spans without adding them to scientific hashes or using them as execution
permission. Prefer a narrow bridge integration to a new proxy/telemetry service.

A positive adoption supplies actual application→SDK/transport→server→Surrealism continuity for
one selected operation, with query planning/queue/network/module interpretation retaining its
observational limits. A lost/unsupported link is explicit, not synthesized from timestamps.
Use `storage_health` selected metrics with engine/service/cgroup identity and completeness.
Capture, target workload, restoration and cleanup have independent outcomes. Span/payload limits,
credential exclusion and optional scientific detail remain under existing capture policy.

If the exact SDK cannot propagate safely within the selected design, retain explicit operation
association and established server→Surrealism tracing, record the unsupported seam and a concrete
revisit trigger. Do not call that end-to-end continuity. The bounded inquiry can close with this
supported narrower outcome; any expanded protocol guarantee needs its own selection before
implementation. No permanent Loki/Tempo/Grafana deployment or Tokio console build is required.
Use such free local tools only for a specific unresolved consumer question.

Acceptance: real finite capture with continuity on the supported path, one unsupported link,
bounded data and successful restoration/drain—or explicit failed restoration/cleanup without
falsifying target outcome. Keep app tracing separate from Plan40 snapshot freshness, physical
state and scientific usability. Publish changed blueprint §23.1 and guide meanings through the
design route, without changing Plan40e status or reviving withdrawn provenance-proof work.

## Verification and dependencies

**Proposed:** F01 can begin with existing evidence sources independently of most series work.
F02 consumes delivered Plan40 identities/capture and FND00's relevant design slice, not all of
Plan40 qualification. Actual transport and database behavior need real bounded consumers in
addition to pure interpretation fixtures. Force-validation accompanies Arrow-consuming checks.
52h qualifies composed investigations; neither a complete trace nor a passing comparison engine
establishes a scientifically valid result or a measured performance improvement.
