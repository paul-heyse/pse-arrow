---
title: "Plan 52f: Evidence analysis and trace correlation"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md, docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md, docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
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

## Canonical interface integration

The unified-interface review's O07/O10 adds consumers, not another evidence authority. F01 owns
pairing/analysis semantics; ADP01/ADP04 expose native schemas/decoders/readers and ADP02 preserves
host outcomes/handles. CAP registers exact comparison/diagnostic operations and relevant guidance.
MCP05 connects interface/provider/request references to authentic native evidence. Do not duplicate
F01 in middleware or turn a FastMCP result into another stored outcome. Existing native/offline
consumers remain usable while these canonical consumers migrate.

Interface correlation can identify where an interaction stopped without proving a continuous
application→SDK→server span chain. F02 still owns that finite inquiry and I11. No tool label, shared
request ID or timestamp substitutes for an actual propagation edge; no extra telemetry database
or mandatory exporter is selected. Native invocation/source/result identities remain unchanged.

## Test-architecture integration

**Proposed:** [PI04](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI04)
refines F01's native section selection and reader consumers alongside 53e E05; completed 52k is
the execution baseline for canonical bindings. [PI09](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI09)
is separately selectable F03 below, per the operator's 2026-10-24 choice. It is not required to
close F01, original Plan53 or the originally selected Plan52 target. No new companion is needed:
comparison semantics and finite inquiry preparation already belong to this owner.

## Usability extension integration

The [usability review's O06](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O06)
adds demand-shaped reader consumers to F01, coordinated with 53e E05 rather than a parallel reader
project. [52l CON02](52l-typed-requests-results-and-diagnostics.md#CON02) owns typed host-family
presentation and migration of other relevant readers. Select answer sections and predicates at
the original owner before hydration/serialization where supported; mandatory population/ambiguity
work can legitimately remain global. Protected continuation and independent section completeness
must survive both native/offline and canonical consumers. Generic JSON queries over curtailed results
and schema-blind response trimming are not alternatives to that contract.

[O12](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O12) gives F01 a
recurring comparison consumer composed under G01/G02, without moving pairing semantics into a prompt
or MCP callback. One call does not establish one atomic source cut. F03 remains separately selectable;
CodeMode evaluation under 52o does not adopt its finite multi-question analysis. Interface progress
and correlation from 52n/G03 do not settle F02's distinct trace-continuity inquiry.

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

Select requested answer sections before query acquisition at the existing native owner. Keep
compact mandatory source/coverage/denominator meanings, with explicit optional contexts,
by-operation/association/capture detail and their own bounds/completeness. Current primary-row
truncation does not describe subsidiary `LIMIT 10001` sections. Preserve full semantic denominators
and ambiguity checks before display limits; do not infer completeness from a small serialized
result. Canonical middleware remains a final response safeguard. Full explicit export remains
legitimate, and ordinary selected reads avoid unrelated hydration through 53e E05's versioned reader.

For measurements, retain sample membership, pairing/order and relevant input/start/cache/build/
provider/capture conditions. Incompatible or unavailable associations remain so; a distribution
cannot repair confounding or prove causality/flakiness. Existing unsupported Criterion-to-invocation
membership remains unsupported unless an actual producer supplies it. Use current SciPy only for
a declared analysis when useful, not an automatic statistical verdict.

Acceptance: authentic sources include failure→pass, persistent failure, unexecuted subject,
renamed/ambiguous subject, changed selection and mismatched measurement context. Independently
specify expected pairs/denominators and show an inconclusive noisy cohort. Offline server-down
analysis and indexed same-cut results agree; duplicate import does not double outcomes. G05's operator view and the canonical FastMCP comparison consumer consume this contract.
Qualify one real canonical request/result read with the same independent expected pairs and
source cut as the native/offline reader, preserving per-section completeness, bounded details and unsupported association. Include large optional sections and a compact selected answer whose essential claims survive the host result bound. Delete replaced heuristic comparison parsers once their
callers move; historical source results remain unchanged.

Exercise an actual large protected source with a complete selected answer, independent full
denominator/ambiguity expectations, omitted optional sections and a stale/released continuation.
Retain original source membership and versioned custody/read semantics through 53e E05. Migrate
the comparison's actual canonical/resource callers with CON02's truthful serialized response;
do not advertise the complete native schema for a payload changed by a generic walker.

<a id="F02"></a>
### F02 — Finite application-to-server trace seam

**2026-10-24 pause context:** [52o's telemetry baseline](52o-conditional-composition-and-protocol-tasks.md#telemetry-baseline--separate-review-no-deployment)
records existing Rust tracing/capture, absent Rust OTel export and a bounded actual FastMCP SDK probe.
It does not close this package or establish Rust/SDK/database continuity. The maintainer is preparing
OTel documentation for a separate thorough, holistic deployment/design review before implementation.
Assess information-access/monitoring improvements alongside native service integration, without
turning spans into another queue, terminal or retry authority. No deployment is selected by that probe.

Resolve I11 against the exact custom server and pinned SDK/transport. Begin with the existing
operation/invocation identities and finite `surreal_server.py` trace capture. Inspect supported
context propagation, request metadata/header and reconnect behavior at the actual SDK bridge;
server OTel support alone does not establish application instrumentation. Associate diagnostic
operation IDs with spans without adding them to scientific hashes or using them as execution
permission. Prefer a narrow bridge integration to a new proxy/telemetry service.

A positive adoption supplies actual application→SDK/transport→server→Surrealism continuity for
one selected operation, with query planning/queue/network/module interpretation retaining its
observational limits. A lost/unsupported link is explicit, not synthesized from timestamps.
MCP05 may attach interface request/provider references to that authentic evidence; qualify the
edge actually supplied without promoting association to causal continuity. Use `storage_health` selected metrics with engine/service/cgroup identity and completeness.
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

<a id="F03"></a>
### F03 — Optional finite multi-question evidence inquiry

**Proposed, separately selectable:** consume one explicit baseline/candidate source cut and a
finite list of existing typed evidence questions under one `EvidenceAnalysis` owner. It already
creates one DataFusion context and bounded pool, parses/registers sources once, and exposes
`ask(&mut self, ...)`. Execute the questions sequentially because they mutate a temporary selected
view; no concurrent mutation of that context is supported. Validate finite question and cumulative
output limits before work, using explicit native admission and the existing per-question bounds.
Return each question's outcome/completeness, original source identities and the supplied cut.

This is an operation-shaped inquiry over the finite question vocabulary, not arbitrary SQL,
a generic batch executor, durable analysis handles or another cache. Expose selected behavior
through existing registry/document generators and completed canonical bindings. Keep the native/
offline single-question operation independently useful, particularly for tiny inquiries and
independent cancellation. Add no new analysis-session expiry/reconnect/retention protocol.

A cancelled or failed question retains its authentic outcome; completed answers cannot imply
whole-inquiry completion. Any remaining unexecuted questions are explicit. A failure does not
silently reuse a prior question's temporary view. The existing DataFusion query context supports
reuse; stream drop alone cannot bound aggregation/sorting intermediates or process RSS. Preserve
F01's independently specified pairs/denominators, PI04's selective sections and protected originals.
The [review's exact library sources](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#library-mechanisms-and-adoption-limits)
record enabling mechanisms and their limits; parse-once analysis is an existing foundation,
not new functionality claimed by this package.

**Selection and acceptance:** defer implementation until an actual F01/52g investigation needs
several related questions over the same cut and the operator selects F03. Then show one analysis
preparation serves several sequential questions; native/offline/canonical pairs and denominators
match independent expectations; duplicate import does not duplicate outcomes; per-question errors,
limits, unexecuted population and cancellation stay explicit. Qualify the actual comparison
consumer, not just a synthetic batch. No measured speed benefit is claimed. Keep F03 deferred if
single-question requests suffice; no original Plan53 obligation or required Plan52 journey waits
for this choice. Its own selected delivery, if authorized later, includes any replaced caller
migration/deletion and its targeted and assembled acceptance.

## Verification and dependencies

**Proposed:** F01 can begin with existing evidence sources independently of most series work.
F02 consumes delivered Plan40 identities/capture and FND00's relevant design slice, not all of
Plan40 qualification. Actual transport and database behavior need real bounded consumers in
addition to pure interpretation fixtures. Force-validation accompanies Arrow-consuming checks.
53e E05 owns the assessment reader/custody migration and F01 migrates its actual comparison consumers with it; F03 can wait independently. 52h and 53g can share an authentic same-cut journey while retaining distinct assertions. 52h qualifies composed investigations; neither a complete trace nor a passing comparison engine
establishes a scientifically valid result or a measured performance improvement.
