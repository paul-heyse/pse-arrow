---
title: "Plan 52f: Evidence analysis and trace correlation"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md, docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md, docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md, docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
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
application→SDK→server span chain. F02 owns that integration and I11. No tool label, shared
request ID or timestamp substitutes for an actual propagation edge. The operator-accepted
[OpenTelemetry direction](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-rule-decisions)
adds continuous observation alongside finite capture through 52p–52s. Native invocation/source/
result identities remain unchanged; the selected telemetry backend supplies observations rather
than another scientific outcome ledger.

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
### F02 — Real application-to-database propagation and finite capture

**Proposed:** implement the database-specific integration selected by the
[OpenTelemetry review O02/O07/O16](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md).
[52p TEL01/TEL03](52p-shared-telemetry-and-native-propagation.md) supplies identities, providers
and native propagation; [52q](52q-local-telemetry-pipeline-and-health.md) supplies the continuous
pipeline. F02 alone owns the SurrealDB SDK task/header seam and I11. The historical
[52o baseline](52o-conditional-composition-and-protocol-tasks.md#telemetry-baseline--separate-review-no-deployment)
establishes neither export nor this continuity. No proxy, scheduler or new execution authority is
implied by adding observation.

Inspect the exact pinned Rust SDK's gRPC dispatcher and spawned tasks alongside the custom server's
W3C extraction. Preserve the caller's tracing/OTel context across dispatch, then inject its
`traceparent`/`tracestate` into the actual request metadata. A context preserved in a caller future
but lost in an SDK worker is insufficient; headers without the authentic parent are insufficient.
Use the narrow patched dependency/integration surface when needed, with exact pins and affected
callers migrated together. Existing server→Surrealism context extraction remains the receiving
contract. Validate reconnect and concurrent requests so a shared worker cannot borrow another
operation's parent. Do not use user-supplied arbitrary headers as an alternate execution route.

The selected target is actual FastMCP/Python→native Rust→SDK/transport→server→Surrealism continuity
for a real selected database operation. Each edge retains its process/service instance and operation/
invocation identities. Show the parent IDs at the receiving spans, not a reconstructed timestamp
association. Keep network, queue, planning, module and commit observations distinct where the actual
producer supplies them; absence of a stage is not evidence of zero wait or skipped work. An edge
outside supported instrumentation is explicitly unsupported. If required continuity proves blocked,
record the actual incompatibility and reopen the target decision with the operator; merely documenting
the gap does not complete this selected package. Unsupported optional detail need not block it.

Route ordinary observations through TEL05–TEL08 and consume TEL10's database/storage metrics;
F02 does not duplicate their health, queue, reset or normalized query contracts. A selected finite
capture still has independent setup, target, restoration and cleanup outcomes. Migrate the existing
finite sink only under TQ05's equivalent boundedness, completeness and restoration contract; a
Collector file is not automatically sealed evidence. No mandatory Loki/Tempo/Grafana or Tokio
console deployment is added. Source bodies, credentials and scientific values follow TEL01's
explicit capture policy; trace IDs stay outside scientific hashes and admission.

Acceptance: one actual canonical request reaches the real custom database and Surrealism module
with independently inspected parentage across the complete chain; concurrent siblings retain their
own parents. Verify the transport/query span remains attributable through actual streamed response
and decode completion, with early stream cancellation/error preserving dispatch uncertainty and
native settlement rather than closing observation at headers or request return.
Exercise missing context and reconnect/task handoff with truthful explicit gaps,
plus a failed database operation whose native outcome remains unchanged. Show the same supported
edges through continuous retrieval and one finite capture, with independent bounded data,
restoration/drain and failed-restoration reporting. Shared fixture IDs alone do not establish these
claims. H02 owns assembled qualification with TEL03/TEL13/TEL14; reuse a valid same-run journey
instead of repeating it per companion. Publish changed blueprint §23.1 and substrate guide meanings
through the accepted decision route, preserving Plan40 snapshot freshness and separate Plan40e scope.

<a id="F03"></a>
### F03 — Optional finite multi-question evidence inquiry

The maintainer selected this bounded value/feasibility inquiry during remaining-Plan52 execution
planning on2026-10-24. Production multi-question delivery remains unselected; return its concrete
recommendation for operator selection. The coordinator owns inquiry progress.

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

## Outcome — 2026-10-25

### What was built

**Implemented:** F01 is a Rust-owned version1 comparison request/result over the unchanged
version2 source snapshot. Exact generations and correspondence preserve original identities,
ambiguity, unexecuted/unmatched populations, measurement conditions and full denominators.
Mandatory compact facts precede independently selected/paged detail; continuations bind the
same cut. Python, offline/indexed CLI, canonical FastMCP and headless HTML/SVG/PNG consumers
share this contract. ADR-0225 remains formally proposed.

**Tested, baseline zero:** the current operations selection passed 57 tests, zero failures:
`just unit-package pse-operations 'test(evidence_query::) | test(evidence::tests::) | test(canonical::transport::tests::) | test(canonical::lifetime::tests::)'`,
invocation `ee0bba9bf8b044fb83ecbcda3b4db8f0`. Independent native oracles cover pairs, hidden
large-cut ambiguity, mappings, repeated mixed outcomes, incompatible/noisy measurements and
strict codecs. Authentic source/native/offline equivalence is retained under
`build/plan52-f01-authentic-20261024`: the tiny selection has 3 baseline/4 candidate subjects,
3 pairs, 1 resolved, 1 persistent, 1 unexecuted and 1 unmatched candidate; the larger original
selection retains 14 resolved/17 persistent/218 unchanged pairs. This is typed equivalence,
not byte equality between omitted optional fields and materialized nulls. Indexed same-cut,
duplicate-import/cursor/custody and headless-renderer controls passed separately. G03/52o
Outcomes own actual Codex reads; no operator UI use or productivity benefit is claimed.

**Implemented and Tested:** F02 extends the pinned SDK task/header seam and narrow native-host
span filter. The real original-owner continuation
`scripts/pse-env --native -- .venv/bin/python /tmp/pse-f02-owned-journey.py retry7`
passed against the explicitly initialized F02 endpoint and existing telemetry pipeline.
The retained driver is `scripts/tests/plan52_f02_acceptance.py`; the temporary wrapper selects
the original fixture and settles prior subjects through their owners. Under
`~/.local/state/pse-arrow/telemetry-qualification/plan52-f02-20261024-first/retry7`,
`acceptance.json` verifies two independent 11-edge parent chains in each of continuous and
finite modes: FastMCP/Python→native helper/sibling→Rust operation→SDK→server→executor→module.
Span IDs establish sibling parents and process incarnations. Continuous retrieval supplied
2,968 rows; finite joined retrieval supplied 3,584 rows. Both explicitly retain bounded
retrieval rather than claiming complete backend populations.

Finite capture retained 2,389 spans/1,404,032 bytes with zero omissions/capture errors,
workload exit0 and restoration `restored`. Native cancellation/missing-branch refusal remain
independent in each producer document. `lifecycle.json` and `owned-study-cleanup.json` retain
successful original-owner settlement of the four exact subjects and stopped/drained endpoint.
Separate real HTTP/2 SDK controls cover absent context, stream completion, errors/cancellation
and dispatch uncertainty; operations controls cover connection/reconnect lifetime. Selected
functional continuity proves neither exact-once/durable telemetry nor whole-product behavior.

**Interface-checked and Tested, inquiry only:** F03’s native unit control uses a supplied fixture
to verify sequential existing questions under one analysis. Authentic F01 single-question
consumers retain their separate source oracles; no authentic production multi-question caller
is claimed. 52o retains actual CodeMode/small-read comparison evidence. Ordinary typed comparison/original readers remain delivered. Production
multi-question, CodeMode, task and Apps adoption remain unselected; recommended deferral is
not operator acceptance. A concrete repeated same-cut question is the revisit condition.

### A mistake made and corrected

The concurrent scientific producer exposed shared claim custody: rejected duplicate-dispatch
cleanup could close the winning publisher's ingestion. Each actual claim now retains its own
UUIDv7 through retries/acknowledgement settlement, preserving the writer guard. The two-study
native regression passed with distinct attempts/manifests, one attempt per point, objectives
near 0.5 and summary publication; the refreshed extension then passed the trace journey.
Earlier failures retain their original conditions.

An earlier intermittent decoder failure was not attributed conclusively. Bounded per-thread
decoder ownership and ordinary version/value/concurrency controls passed; this is neither
a proved crash cause nor sanitizer/fuzz qualification. C04 retains its accepted fallback.

### Deviations from the plan, deliberate

The original bounded capture/restoration owner was reused. No arbitrary SQL, generic batch
executor, durable analysis handle or replacement sink was added. Representative studies expose
the required distinctions without a production-width scientific campaign. Scope-end/manual and
assembled reporting remains with 52h; Plan40e/Plan53 retain their separate acceptance.
