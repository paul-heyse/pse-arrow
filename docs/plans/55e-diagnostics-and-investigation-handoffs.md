---
title: "Plan 55e: Diagnostics and investigation handoffs"
status: complete
date: 2026-10-25
adrs: []
review_sources: [docs/functional_reviews/mcp-development-effectiveness-2026-10-25.md]
doc_role: plan
doc_owner: docs/plans/55-mcp-development-effectiveness.md
doc_topics: [current-work, design, diagnostics, storage, performance]
---

# Plan 55e: Diagnostics and investigation handoffs

## Target and foundation

**Proposed:** let Codex express precise operational questions, explain an exact invocation and
carry an investigation into a new connection without replay or invented continuity. Owns O13–O16;
the [coordinator](55-mcp-development-effectiveness.md) owns their dispositions. Development
diagnosis is primary, with useful later production incident/query investigation behavior.

**Interface-checked:** [telemetry_metrics.py](../../scripts/telemetry_metrics.py) and generated
native declarations already publish finite instrument dimensions. Current
[`QueryRequest`/`backend_request`](../../scripts/telemetry_investigation.py) exposes metric/time/
service selection but not instrument-specific dimensions; its p95 expression aggregates other
dimensions with `sum by(le)`. Original product/assessment/workload readers, build observation,
profile/pressure capture and telemetry health/query already exist.

[`telemetry_selection.Investigation`](../../scripts/telemetry_selection.py) already retains
finite copied answers, schema descriptions and missing context through original native member-
pack custody. `knowledge_explain`, service/storage health and MX statistics are existing owners.
These are opportunities to extend useful questions and composition, not a new telemetry rollout,
second state ledger or assumption that all diagnostic data is missing.

Use the reviewed [OTel Python](../libraries/OpenTelemetry_Python.md) 1.45.1,
[OTel Rust](../libraries/OpenTelemetry_Rust.md) 0.33.1 / tracing-opentelemetry 0.34.0 and
[FastMCP observation](../libraries/fastmcp.md#114-opentelemetry-tracing) mechanisms. Source review
research distinguishes library feasibility from deployed/qualified behavior. No instrumentation,
backend migration or metric family is added merely to enable a query over already-produced facts.

## Query and explanation design

Extend the current metric query contract with permitted per-instrument predicates and grouping
derived from that instrument's original declarations. Inputs remain finite typed vocabulary,
not arbitrary PromQL/SQL or arbitrary tags. Apply predicates/grouping at the backend query owner
before result expansion. Retain selected labels/population, units, interval/lookback, reset/bucket
and estimator meaning in the answer. Missing streams, stale/unavailable backend, empty data and
overflow/partial coverage remain different observations.

For invocation explanation, start with one original issued work reference and the question:
“What is this exact invocation doing, and where can I inspect the next fact?” Compose available
selection/execution counts, preparation/build stage, terminal/drain and selected telemetry links.
Use a progressive workflow/resource if that answers the question economically; add a narrow
typed projection only where current separate reads leave a demonstrated composition gap.
Each observation keeps its owner/source cut; there is no atomic cross-owner snapshot promise.

For retained handoff, extend the existing investigation document with a bounded question, typed
original-owner references and unexecuted next-reader suggestions. Distinguish copied observations
then from original reads now. Schema version changes follow the existing document/version route;
unsupported older content is not filled with fabricated new fields. Reuse original evidence pack
custody/cursors and explicit export effects, not another agent memory store or automatic executor.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| E01 | complete | 55e | Instrument-specific metric filters/grouping with exact declared population and estimate limits |
| E02 | complete | 55e | One-invocation explanation composes original execution/preparation/observation facts |
| E03 | complete | 55e | Question-bearing retained handoff follows original references without replay |
| E04 | complete | 55e | Separate exact query/explain/engine readers retained; decision below |
| E05 | deferred | 55e | Revisit when an assigned investigation cannot express needed query/index/engine correlation through existing typed reads |

<a id="E01"></a>
### E01 — Query declared dimensions before expanding series

Extend original Python/native metric declarations only where the permitted dimension meaning
is missing; do not restate it in a second capability map. Expose the supported keys/value domains
and grouping through `telemetry_schema` and its existing resource. Extend QueryRequest validation,
backend construction and typed answers together. Reject unsupported instrument/key combinations
before backend work. Preserve earlier ungrouped semantics only as an explicitly understood
population, not an accidental default that claims stage-specific meaning.

Initial consumers distinguish preparation/execution durations by actual emitted stage/mode,
failures by declared class, and native acquisition producers/waiters where those instruments
provide the facts. Instrument vocabulary limits apply; do not promise every cross-product is
emitted or add source hashes/trace IDs as unbounded metric labels. OTel Views and cardinality/
overflow behavior constrain observation coverage, not scientific admission.

**Acceptance:** a controlled two-stage/two-mode population remains separated when filtered/
grouped; combined selection reports its actual population. Invalid dimensions refuse before
query. Exact count expectations challenge backend-expression errors; rate/p95/fraction tests
retain their estimator/window/bucket/reset semantics rather than assert them as exact totals.
Overflow, missing stream and unavailable backend do not masquerade as absence of failure.

<a id="E02"></a>
### E02 — Explain one original invocation

Reuse [workload_queries](../../scripts/capabilities/workload_queries.py), original assessment/
run inspection, build observation and selected telemetry/profile readers. Report requested,
selected, executed and failed counts when their owners supply them, along with current/frozen
state and remaining physical drain/cleanup obligations. Show a precise next original reader
for absent/partial facts, not an inferred replacement result. D02 consumes comparable history;
this package explains the selected invocation itself.

Differentiate cold compilation/preparation from a slow subject, observed progress from expected
cardinality, and terminal records from settlement. Native acquisition waiters suggest questions,
not a proven deadlock. Sampled profiles are not exact timelines, nested durations are inclusive,
and recorded idle is not OS off-CPU time. Export availability is independent of native success.
Use existing trace/span/native references and optional E01 dimensions; new instrumentation needs
a concrete missing fact, not a desire for a prettier summary.

**Acceptance:** one cold preparation, one bounded long subject and one failed/draining invocation
have distinct truthful explanations. Absent telemetry leaves native facts readable. Delayed or
partial owners do not produce a fabricated atomic snapshot. The actual agent can locate the
performing stage and original result without a new dashboard/task process or replay.

<a id="E03"></a>
### E03 — Retain the question and original next readers

Consume the existing Investigation/export/page schema and E02's original-reference vocabulary.
Add an explicit question, available source/policy/result identities, unresolved missing context
and finite next-reader suggestions. Record source interval/generation separately for each copied
selection; preserve copied completeness versus live population completeness. A suggested action
is not executed, admitted or newly authorized merely by reopening a handoff.

Original work references may outlive the MCP connection; endpoint-bound preview references do
not. If an original owner is unavailable/deleted or current source is incompatible, preserve that
fact rather than reconstruct state from telemetry. Use OTel `Link` / `add_link_with_attributes`
to associate a deliberately new execution where useful; it is not continuation of a closed
parent span or a native lineage authority. See the
[tracing extension trait](https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/trait.OpenTelemetrySpanExt.html).

**Acceptance:** a second connection reads a finite retained question/selection, understands its
limits and follows the exact native reader without duplicate work. Changed source, old schema,
unavailable owners and partial copies stay distinct. Export remains explicit bounded custody,
with no second telemetry query hidden inside publication.

<a id="E04"></a>
### E04 — Decide whether a query investigation packet adds value

Choose one real structured-query issue: population selection, traversal/index behavior or
retained-engine readers. First use separate existing query/generation, nonexecuting explain,
selected index/health/statistics and telemetry reads. Determine whether a narrowly composed
packet would remove actual manual ambiguity while preserving their separate source cuts.
It may consume B01 when MCP indexed access helps, but existing native query/explain suffices
for the inquiry; no artificial dependency on a new query service.

SurrealQL `EXPLAIN`, index information, version-specific graph/reference scan counters and
[MX 1.0 statistics](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs) are candidate
facts. `EXPLAIN ANALYZE` executes and requires explicit effect-safe subject/fixture admission;
nonexecuting explanation does not prove runtime work. Preserve raw versioned plan detail and
unsupported/absent counters, not an invented stable optimizer schema. Reader counts/retained
versions are hypotheses, not proof of a leak or slowness cause.

**Decision output:** retain separate reads with an observable revisit condition, or adopt a
named packet whose actual missing composition value is demonstrated. Do not prescribe inline
adjacency/indexes globally or use SurrealKV diagnostics against the MX store. Consult
[EXPLAIN](https://surrealdb.com/docs/reference/query-language/statements/explain) and
[the tooling overview](../libraries/surrealdb_tooling.md), verifying selected 3.3.4 capabilities.

<a id="E05"></a>
### E05 — Deliver the adopted query-specific composition

Requires positive E04. Compose exact selected query/generation, original explain/index facts
and explicitly selected engine/telemetry observations through existing owners. Describe effect
differences and unsupported detail; do not add another collector, monitor, dashboard or general
query interpreter. Analyze, if positively selected, is a distinct explicit action with bounded
subject scope, not a mode silently activated by passive diagnosis.

**Acceptance if adopted:** an actual agent distinguishes the chosen query-selection/index
problem, follows original raw facts and sees missing/unsupported observations. No live refresh,
query execution or service startup occurs during a passive read. A failed/partial component
does not erase independent observations or claim causality.

## Inquiry decision

### E04 inquiry decision — 2026-10-25

**Interface-checked:** the original knowledge explain response carries issuer/generation/query
identity, raw/population plans, coverage and freshness; the independent server metric snapshot
carries authenticated server identity and observation interval. For the representative generated-
artifact query-shape versus engine-reader question, these exact readers preserve necessary
attribution. A combined packet supplies neither an atomic cut nor causal proof, and no missing
composition value was demonstrated. Retain separate reads and defer E05. Revisit when an actual
assigned investigation repeatedly needs a correlation those typed routes cannot express.
This is static owner-contract evidence, not a reproduced slow query; no ANALYZE/backend was run.

## Verification and Outcome — 2026-10-25

**Implemented:** E01–E03 provide per-instrument finite filter/grouping domains, original invocation
and assessment explanations, and version-two question-bearing retained investigation handoffs.
Host views preserve actual selected/reported/failed populations, stage/drain unknowns, sampled
or inclusive timing and unexecuted next-reader suggestions. Native custody admits declared JSON
versions one and two with exact actual-header checks; unknown versions/header-body mismatches
refuse. No new telemetry pipeline, mutable execution ledger or automatic analysis is introduced.
E04 is complete; E05 remains deferred under its concrete correlation-gap trigger.

**Tested:** diagnostic/migration/retention controls pass in the full 2,277-case setup population.
Actual source-only MCP discovery exposed finite metric dimensions; invalid combinations refused
before querying and a valid request against an absent backend reported unavailable, not empty.
Actual original assessment/invocation reads retained real counts and unknown preparation/drain
facts. A native export then endpoint teardown/new connection reopened the same question,
continued original selections, retained unavailable-owner detail and followed the real assessment.
The derived CLI under the original native environment reopened the same retained page; a plain
CLI without that prerequisite correctly reported unavailable. Native admission controls include
all eight negative version/header cases. [55f](55f-qualification-and-publication.md#verification-and-outcome--2026-10-25)
owns the exact receipts and exclusions.

**Corrected mistake:** the host produced a version-two investigation, but the original native
custody whitelist still allowed version one only. Extending that exact declared protocol and
retaining version-first checks repaired the real consumer, without loosening arbitrary JSON
admission. No new backend throughput, query causality or live telemetry continuity is claimed.
