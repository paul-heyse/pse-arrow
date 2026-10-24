---
title: "Plan 52s: Telemetry queries and agent investigations"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, diagnostics, environment, validation, libraries]
---

# Plan 52s: Telemetry queries and agent investigations

## Purpose and ownership

**Proposed:** make selected telemetry history useful through exact typed reads, progressive
investigation, existing profiling/custody and selected backend-native watches. This develops
the [OTel review's O12–O17](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md#O12)
with O10/O18/O20 as cross-cutting requirements. The
[coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns package state,
[rule decisions](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-rule-decisions),
[opportunity dispositions](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-opportunity-dispositions)
and TQ01–TQ05. RC01/RC02 were accepted for planning on 2026-10-24. This is authoring scope;
backend queries, capture, deployment and production changes require execution authorization.

The selected production query backend is native OpenObserve OSS behind one integration owner.
Agent reads enter the existing FastMCP endpoint with exact domain/workflow selection, typed
requests, selected results, resource links and invoke-read fallback. No separate developer
MCP, semantic search, universal query DSL or Python telemetry query engine is selected.

[52p](52p-shared-telemetry-and-native-propagation.md) owns identity/bootstrap/native
instrumentation, [52q](52q-local-telemetry-pipeline-and-health.md) deployment and independent
health, and [52r](52r-operational-metrics-and-diagnostic-integration.md) populations/views/sampling/exemplars.
Existing [52f F02](52f-evidence-analysis-and-trace-correlation.md#F02) alone owns the actual
application/database continuity seam and I11. [52g](52g-agent-and-operator-workflows.md) owns
assembled workflows/operator-view selection; [52h](52h-qualification-and-publication.md) owns
assembled qualification. These contracts are consumed here rather than reimplemented.
FND00 supplies the decision/declaration route for any actual new backend binding, relation
family or protected boundary; an ordinary third-party dependency does not require an ADR.

## Baseline and foundation judgment

Authoring uses main `6e93c07c1` plus concurrent dirty Plan54/build-tooling changes. Preserve their
actual producers and consumers; querying existing build records does not depend on future Plan54
completion. Core3.4, Efficient Architecture Heuristics1.0, Process Simulator1.5 and the binding
shape this focused assessment. No broad diagnostic audit or new formal review is initiated.

The existing [capability interface](../dev/capability-interface.md) already separates discovery,
effectful execution, passive readiness, native references and loaded source revision. Its
[binding composition](../../scripts/capabilities/server.py),
[typed host reads](../../scripts/capabilities/read_contracts.py),
[presentation](../../scripts/capabilities/presentation.py) and
[reference generator](../../scripts/capabilities/reference.py) are the right integration surface.
Extend those mechanisms rather than create an exposure catalogue or generated backend API mirror.
Native schemas/readers remain native; Pydantic owns host query and presentation contracts,
without restating scientific records or gaining scientific authority.

The [diagnostic guide](../dev/simulation-diagnostics.md#explicit-bounded-capture) and
[runtime capture](../../crates/pse-runtime/src/tooling/capture.rs) already provide finite Timeline/CPU
capture, separately owned commands, exact target checks and explicit partiality. A long span is
elapsed lifetime; native CPU capture and retained source detail answer different questions.
Reuse these before introducing another profiling stack. Existing
[evidence detail](../../scripts/evidence_detail.py),
[evidence sources](../../scripts/evidence_sources.py) and
[offline publication](../../crates/pse-runtime/src/tooling/review_pack.rs) preserve custody and
frozen interpretation independently of a live telemetry service. A backend result is another
selected source, not permission for a second evidence ledger.

AP-01/03 place backend variation in one adapter and expose it through existing composition.
AP-02/04/05 require populations, source cuts, units, partiality and identity to govern answers.
AP-06 favors independently specified fixture expectations; AP-07 favors backend predicates,
projection/aggregation and selected detail before hydration. PS-09/10 preserve native scientific
control/results. Trace status, watch activation and protocol completion never replace them.

## Query realization and retained semantic limits

The [retained driver](../../scripts/tests/opentelemetry_query_inquiry.py) and
[captured OpenObserve1.1.2 result](../functional_reviews/probes/opentelemetry-query-2026-10-24.json)
provide a concrete starting point. The driver posts SQL search to
`/api/{organization}/_search?type={signal}` with `query.sql`, `start_time`, `end_time`,
`from` and `size`; SQL time intervals are microseconds. It also exercises
`/api/{organization}/prometheus/api/v1/query_range` and `query_exemplars`, whose query times
use seconds/RFC3339. These are tested fixture paths, not a qualified production adapter.

Use the [search API](https://openobserve.ai/docs/reference/api/search/search/),
[trace API](https://openobserve.ai/docs/reference/api/traces/trace-search-api/),
[API surface](https://api.openobserve.ai/swagger/) and
[v1.1.2 metric ingestion source](https://github.com/openobserve/openobserve/blob/v1.1.2/src/core/src/metrics/otlp.rs)
to resolve actual selected-release behavior. Current Context7 search/query documentation points
to main; optional flags such as `track_total_hits`, sampling and partial-response fields require
release-specific validation under TQ01. An API field's presence does not establish its semantics.

| Observed fixture limit | Required consumer behavior |
|---|---|
| Trace `service_service_instance_id`; logs/metrics `service_instance_id` | One versioned per-signal field map; preserve original names in schema/detail. Never assume punctuation replacement is universal. |
| `reference_parent_span_id` / `reference_parent_trace_id` | Normalize actual parent references; absence stays missing, with links distinct from parentage. |
| Five spans returned in2/2/1 pages; response `total` was2/2/1 | Report page population separately from selected population/completeness. A count query is a distinct observation, not a snapshot guarantee. |
| SQL preserved counter start timestamps; PromQL exposed the latest `start_time` label over older points | Preserve SQL point/reset provenance; label PromQL interpretation and avoid attributing that label to each historical point. |
| `increase` extrapolates; histogram quantiles interpolate | Present estimates with units/population/window; neither becomes exact work counts or raw-observation percentiles. |
| Exemplar pointer appeared even below its value bucket; fixture times preceded the trace | Treat IDs as related examples. Strong membership/causality needs qualified population, clock, timestamp and bucket evidence. |

The fixture tested direct synthetic OTLP JSON, SQL identity/parent fields, correlated log,
counter/reset representation, nonempty PromQL results and exemplar pointer retention. It did
not test concurrent-ingestion pagination/equal keys, a general PromQL oracle, Collector or
SDK network ingress, actual Rust/database continuity or assembled FastMCP consumers. TQ01 owns
those query/metric/exemplar uncertainties with TEL12; TQ02 owns retention/recovery with TEL08.
Do not let narrower interface evidence close either delivered capability.

## Typed requests, selected answers and continuation

Start with meaningful reads: trace/operation lookup, selected span/log detail, an operational
distribution, pipeline health and schema/capability description. Requests carry explicit time
and identity filters, requested sections/fields and output/page bounds. Push filtering,
projection and aggregation into the backend. Internal query generation validates identifiers
and encodes values through the selected backend contract; caller text is never interpolated
as executable SQL. No shared arbitrary-string query protocol is needed for ordinary reads.

Answers retain query/adapter/schema/backend identity, requested/source intervals and time units,
retrieval time, selected service/process/source identities, population/aggregation meaning,
returned rows and known partial/scanned/retention limits. Optional sections have their own
selection/bounds/completeness. Empty, unsupported, unavailable, expired and truncated remain
different. A missing span cannot establish that an operation did not execute; telemetry health
may explain a missing source without granting authority to reconstruct its history.

Continuation binds the exact filters, sort, projection, time interval, backend/configuration
and supported source cut. Prefer a stable supported backend cut/order; otherwise explicitly
report concurrent-ingestion limitations. TQ01 challenges equal keys and inserts between pages.
If selected semantics cannot establish stable continuation, refuse that claim or offer an
explicit finite copied selection under TEL16; never silently skip/deduplicate a moving population.
Protect original reader references using the existing continuation/result-reference machinery.
An opaque token does not turn an unstable backend read into a coherent snapshot.

Schema resources expose actual supported fields, units, signal streams, normalization,
query/aggregation capabilities and limitations with their selected source/version identity.
Changing backend field layout updates this one adapter/schema owner. Exact native operation,
result and capture references lead to their original readers rather than embedding another
interpretation of science in each span result. Trace detail can be absent/expired while the
original native result remains readable.

A backend-native read-only query capability is conditional on a concrete unmet advanced workflow
under TQ01. If selected, describe its exact SQL/PromQL surface and limitations separately from
ordinary typed reads, apply selected backend/stream/deadline/output boundaries, and ensure no
mutation/export side effect escapes the read contract. Do not expose the entire backend API by
default or build a custom parser/engine merely to simulate a backend access policy.

## Work packages and consumer acceptance

Package state and inquiry dispositions live only in the coordinator. Local packages use
targeted tests/compile checks where relevant; 52h owns integrated testing and scope-end static
checks. The mechanisms below remain proposed until actual consumers and failure cases qualify.

<a id="TEL13"></a>
### TEL13 — Typed backend queries, normalization and selected results

Implement one backend adapter and explicit typed requests/answers for the selected reads above.
Bind signal normalization, SQL/PromQL units, population/temporality semantics, independent section
bounds and source-aware continuation there. TQ01 settles full query/metric/exemplar semantics
with TEL12; richer structural matching is unsupported unless actual selected APIs establish it.
Do not reconstruct parent/descendant edges from timestamps or shared operation labels.

The early contract/fixture slice consumes TEL01 identity meaning and the retained backend
responses. Actual backend acceptance needs TEL05/06 service/routing; metric reads need TEL09/12's
implemented populations/exemplar semantics. A schema agreement alone does not qualify an emitted
metric. Source/configuration changes coordinate with the endpoint freshness owner.

Acceptance independently specifies identities, parent/link distinctions, units, counts and reset
boundaries, then exercises the actual adapter. Include2/2/1 pages, equal sort keys, concurrent
insertion, expired trace, backend error/partial response, unknown field and out-of-bound interval.
Check the metric oracle separately from the backend's own aggregate: estimated results remain
estimated. Preserve the known exemplar defect as pointer-only unless new evidence establishes
stronger semantics. Retire test-only query helpers only where their production consumers have
moved; retained manual inquiry drivers remain historical evidence while depended upon.

<a id="TEL14"></a>
### TEL14 — Existing-endpoint bindings, discovery, guidance and availability

The authorized 52p/52q refinement includes bounded configuration/health/known-trace bindings and
direct original native process/checkpoint readers. Their acceptance belongs to the existing
52p/52q checkpoint. This supplies useful source-attributed investigation primitives; the full
TEL13 query/normalization/continuation contracts and remaining TEL14 consumers below stay open.

Bind actual TEL13 reads and TEL07 health through existing FastMCP capability composition,
Pydantic host contracts, exact domain/workflow discovery and selected tools/resources. Derive
reference/schema publication from those definitions. Keep fixed-inventory invoke-read usable;
do not require a host refresh, semantic search or another MCP connection to investigate.

Compose progressive guidance: compact stage/coverage summary → suspect interval → selected
spans/logs → original native operation/result/detail. Availability distinguishes passive
configuration from explicitly exercised service/query support. Backend-down guidance links
TEL07's local observations and existing native readers. Discovery and schema description do not
install/start the backend, initialize capture or import native execution as an accidental effect.

TEL14's declarations can precede complete ingress, with truthful unavailable states; usable live
reads require tested TEL13/TEL07 slices. Coordinate shared catalogue/binding/guidance edits with
52l/52m/52n and 52g rather than overwrite their presentation/source-cut contracts. Existing
52n progress remains coalesced presentation; no token/notification does not mean native stop.

Acceptance goes through real STDIO discovery, selection, description, call and invoke-read:
an agent reaches a retained native reference from a compact query without bulk hydration.
Exercise backend down, stale loaded serving source, host fixed inventory, unknown selection
and independently failed notification delivery. Guidance explains source/units/limits and
does not imply retry, cancellation, task support or continuous trace propagation.

<a id="TEL15"></a>
### TEL15 — Native profiling escalation and optional mechanisms

Connect an observed suspect stage to existing `CaptureRequest`/`CaptureSession` Timeline/CPU
and launch-capture owners. Preserve actual checkpoint/operation/process incarnation, thread
identity, sampling interval, clock uncertainty, output bounds, useful symbols and loss. A
completed operation can supply retained history but cannot supply preceding CPU samples through
late attachment. Capture stop/drop drains only capture; science remains with its native owner.

TQ03 is a finite usefulness/fit assessment of optional Pyroscope, supported eBPF/off-CPU tooling
and runtime-specific diagnostics. Use the review's
[Pyroscope deployment](https://grafana.com/docs/pyroscope/latest/reference-pyroscope-architecture/deployment-modes/)
and [platform support](https://grafana.com/docs/pyroscope/latest/configure-client/supported-platforms/)
leads; verify actual native/Linux, Rust/Python, symbolization and query support before selecting
an integration. Experimental Rust `ContextObserver`/`GlobalContextObserver` report context-entry
transitions; they are not profilers or a qualified bridge. Prefer an existing supported bridge
over bespoke trace-to-profile machinery. No blanket host permission changes are selected.

TEL15 consumes native identities from TEL01/04 and TEL13's suspect selection, but an optional
profiling stack is not required for ordinary telemetry query delivery. Acceptance first shows
the existing actual native capture route, including unavailable tool/permission, wrong target
identity, partial symbols and overlap; elapsed span duration must not be reported as CPU time.
TQ03 records adopt/defer/reject per optional mechanism, exact unmet question and observable
revisit trigger. If adopted, include its source-backed collection/query/lifecycle acceptance;
an API exploration alone cannot close the selected capability.

<a id="TEL16"></a>
### TEL16 — Finite offline investigation export through existing custody

Export explicitly selected query answers/signal detail plus original parameters, backend/schema/
adapter identity, source/request/retrieval intervals, native references and known sampling,
loss/retention/continuation limits. Use existing immutable evidence publication and protected
member readers; extend their owned codecs where needed. Do not write a second mutable analysis
record into SurrealDB or copy the entire telemetry database. Original native IDs remain original.

Copy and finalize under finite input/output/working allowances. Distinct sources can have
different cuts; label them individually rather than promise an atomic multi-source snapshot.
Truncation/incomplete export is readable only under the evidence owner's actual incomplete
publication contract. Offline interpretation consumes supplied members and recorded intervals,
without current reader time, neighbouring files, live services or rerunning science.

TEL16 consumes tested TEL13 answer/continuation meaning and the existing custody contract;
coordinate F01/53e's selected reader evolution. TQ05 with TEL08 decides whether standard OTLP
JSON/file output can replace the finite sink while preserving identities, bounds, restoration
and reader/custody guarantees. Interchange output alone cannot establish sealed publication.

Acceptance exports one bounded multi-signal investigation with a missing/expired member and
explicit separate source cuts. Stop the backend and read the same selected explanation through
the original offline consumer. Wrong member digest, partial publication and output exhaustion
remain explicit; no unlisted path is followed. If TQ05 selects substitution, migrate actual
finite-capture callers/readers and delete the replaced sink/tests/fixtures in the same coherent
change. A non-equivalent candidate leaves the distinct existing capture contract intact.

<a id="TEL17"></a>
### TEL17 — Selected backend watches/alerts and operator-view input

TQ04 assesses useful recurring conditions and exact OSS support: queue saturation/export loss,
service failure, stale publication or selected duration distributions. Prefer backend-native
scheduled query/alert evaluation and existing service supervision. Verify chosen release APIs,
evaluation population/window, missing data, silence/deduplication, persistence and available local
delivery/history. Current documentation/main API inventories do not prove OSS1.1.2 eligibility.
No paid/cloud dependency or new MCP polling scheduler is selected.

Return the condition, its actual evaluation/source interval, supporting observations, current
availability and related process/operation references through TEL14. Service failure can remain
visible via TEL07 even when the backend cannot evaluate its own alert. No heartbeat, quiet
callback or long span proves deadlock. Alert notification does not authorize restart, tuning,
retry or scientific cancellation; those remain explicit native actions.

TEL17 needs TEL07 and relevant tested TEL09/TEL13 populations/reads. 52g G05/I13 owns optional
operator-view selection and consumes these same queries, not a second dashboard semantic model.
Acceptance exercises a selected actual recurring condition and a no-data/disabled/backlogged
evaluation, showing the matching native references and lack of implicit intervention. TQ04
dispositions identify which native alerts/watches are selected and which are deferred with a
concrete trigger. If a useful required behavior lacks OSS fit, return that decision to the
coordinator before adding infrastructure; do not silently close it with a manual query.

## Verification, benefit and durable ownership

**Proposed verification:** local tests challenge adapter semantics and actual canonical
consumers; 52g supplies assembled investigation journeys and 52h H01–H04 owns integrated
qualification/O20 and selected measurements. F02 owns real database continuity separately.
Failure baseline is zero. No product tests, backend queries, capture, services or probes were
run for this plan authoring; existing inquiry evidence retains its original scope and limits.

**Proposed benefit:** backend selection/projection removes repeated full-history hydration;
progressive reads reduce response/context burden; retained/offline answers let a later session
continue an investigation. Quantitative latency, overhead, capacity or productivity claims need
named representative measurements under 52h, rather than a small fixture or a schema check.

At closure, move enduring query/normalization/partiality and exposure meaning to blueprint §23.1
through the accepted route and the capability/diagnostic/environment guides. Existing custody,
native schemas and source owners keep their meaning. The coordinator records actual outcome,
mistakes corrected, deliberate deviations and retirement after consumers/references move;
this supporting document maintains no competing status or evidence ledger.
