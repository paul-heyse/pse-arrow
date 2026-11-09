---
title: "Plan72h: Diagnostics and execution fit"
status: draft
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, diagnostics, performance, storage, deployment]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/diagnostics-and-execution-fit.md]
---

# Diagnostics and execution fit

**Proposed.** Deliver O17/O18 as selected-operation explanations and credible native execution
for larger reads alongside canonical writes. Improve query shape and remove unnecessary
materialization directly on a sound design basis. Do not require proof of a speedup to adopt
that design. Additional experiments settle only serious unresolved correctness or performance
degradation risk. The [coordinator](72-surrealdb-functional-improvements.md) owns dispositions.

## Baseline and foundation assessment

**Interface-checked.** At `21f703887cc307f986975f9a2fba998f8bd2afa7` plus concurrent changes,
the [canonical query builder](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_explanations.rs)
already describes the exact SQL/bindings used for bounded graph requests. The knowledge reader
has nonexecuting plan inspection. Native query results retain supplied statement duration only
when present and wait for complete terminal accounting. The [telemetry owner](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-host/src/telemetry_service.rs)
already configures a separately authenticated Collector/OpenObserve installation with bounded
queues. These supply useful foundations; no new receiver or generic evidence platform is needed.

Use blueprint §20/§24, core AP-04/AP-07 and the efficient-architecture/test companions to keep
selected facts, physical observations and original outcomes separate. Plan71's
[CLI evidence](71a-cli-inspection-and-evidence-lifecycle.md) and
[selected diagnosis](71d-fixture-efficiency-and-selected-diagnostics.md) are active consumers,
not missing infrastructure. Join SurrealDB facts there and preserve their live status owners.
No historical plan supplies scope or acceptance obligations for this companion.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [H01](#H01) | Planned | Existing query descriptions and Plan71 references; each selected 72d/72f operation as delivered | One bounded explanation joins query intent, versioned plan and complete native outcome without executing ordinary inspection. |
| [H02](#H02) | Planned | H01 identity; existing telemetry and 72g report owners | Real available signals/progress join the original operation with visible omission/loss and existing release semantics. |
| [H03](#H03) | Planned | 72d query contracts and 72f consumer lifetime; current native scratch owner | Eligible larger reads use native sort/group spill and owned cleanup without application-side duplicate materialization. |
| [H04](#H04) | Planned | H03 useful execution; only a named serious coexistence/regression risk | Resolve remaining scheduler/maintenance contention by the smallest relevant evidence; keep defaults if no risk warrants tuning. |

<a id="H01"></a>
## H01 — Explain the same operation that the caller performs

Define a bounded source-owned explanation joining selected request/query identity, issuer,
data cut, server/patch incarnation, static plan and original complete response facts. Reuse
the actual query builder and binding validation. Report useful index, candidate, graph
cache/block/delta and expansion observations without exposing one supposedly stable supplier
JSON shape as a permanent product contract. Missing counters remain unknown; output rows are
not scanned rows or complete semantic dependency evidence.

Ordinary inspect uses nonexecuting `EXPLAIN`. Profiling with `EXPLAIN ANALYZE FORMAT JSON`
is a separate explicitly admitted diagnostic operation with the statement's actual effects,
selection and deadline; never quietly analyze a write during a passive read. See the
[v4 EXPLAIN reference](https://surrealdb.com/docs/reference/query-language/statements/explain).
Retain original outcome and distinguish supplied database duration, client transfer/preflight,
hydration, waiting and cleanup only where those producers actually provide facts. Do not add
overlapping durations into a fabricated exclusive total.

Move graph/knowledge inspection and selected canonical readers to this shared projection,
deleting repeated plan parsing and caller reconstruction when replaced. Targeted acceptance:
the plan uses the same checked bindings; unknown operator versions preserve useful unknowns;
ordinary inspection does not execute; an incomplete response cannot become successful
completion. A static source/contract check settles obvious duplicated parsing without a benchmark.

<a id="H02"></a>
## H02 — Join existing telemetry and structured progress

Connect supported database OTLP signals to the already owned Collector rather than construct
another receiver. Preserve signal type, selected service incarnation, sampling and queue loss.
Use bounded trace/request records for correlation; avoid original operation IDs or payloads as
unbounded metric labels. Absence of delivered data is not success. The
[tagged telemetry contract](https://github.com/surrealdb/surrealdb/blob/v4.0.0/doc/TELEMETRY.md)
and current launcher/export configuration decide which facts exist.

Use SurrealKit structured reports for outcomes and `ProgressSink`/`capture`/`propagate` for
bounded progress, exposing overflow and task propagation gaps. Prefer subprocess JSON when
embedding would add global logger/runtime conflicts without removing meaningful work. Give
each diagnostic reference retrieval, stop and release through the existing Plan71 owner;
scientific payload lifetime and original verdict remain independent.

Replace log scraping and duplicate retention/progress stores. Targeted cases cover absent
export, sampled/lost records, bounded progress overflow, unavailable attachment and owner-backed
release. Verify a configured exporter actually delivers a named signal before reporting it as
available; no broad observability campaign or mandatory instrumentation of every primitive.
Enterprise audit/slow-query records stay nonselected unless a durable diagnosis cannot be served
by the selected facts and its retention has an owner. A richer diagnosis can justify that
branch without an existing caller, but does not justify a second canonical outcome ledger.

<a id="H03"></a>
## H03 — Use native physical execution and owned scratch

Select parameterized predicates/projections/order/limits under 72d before adding resources.
Use native sort/group spill for genuine necessary intermediate state, with the service's private
temporary directory and existing query-memory configuration. Keep bound values and accumulated
subquery results within the nonspillable model; a streaming transport does not make a
materialized producer incremental. Delete superseded application arrays/repeated hydration and
any second executor introduced solely to imitate a capability the selected engine supplies.

The selected source/feature closure, not the advertised full database, determines operators:
distinct/hash-join spill associated with unselected GQL must not be credited to SurrealQL.
Enable GQL only with 72d's selected explanation/query realization. Inspect the v4 source/CLI
controls when changing them; the [runtime configuration reference](https://surrealdb.com/docs/reference/cli/db/environment-variables)
is a navigation route, not proof that every backend or default matches the local build.

Handle cooperative memory waits/overruns and invalid/full temporary storage explicitly. Scratch
belongs to the original producing operation/service and is disposed after useful evidence transfer
and physical settlement. Preserve unresolved residuals for recovery instead of announcing drain
from a process exit or zero gauge. Keep the maintainer's ordinary unbounded host selection;
query accounting does not introduce host CPU partitions, compiler limits or RAM caps.

Targeted acceptance covers an eligible sort/group result, cancellation during spill, failed scratch
allocation and physical cleanup, chosen only for the paths changed. These controls expose serious
resource/correctness risks. Adoption of reduced work or native execution does not depend on a
matched before/after speed claim.

<a id="H04"></a>
## H04 — Resolve serious coexistence risks without a tuning programme

Keep the supplier's ordinary shared scheduling as the target. Tune morsel places, OLTP reserve,
work quantum or maintenance scheduling only when source inspection or receiving identifies a
credible serious risk that large reads/index work starve ordinary canonical operations or amplify
live state. Name the operation, shared resource and failure exposure before adding an experiment.

First resolve from source and observed lifecycle facts. If that leaves a consequential uncertainty,
use one matched coexistence case with a useful large read and small canonical transitions, adding
background index work only when it participates in the identified conflict. Match selected data,
authorization and expected outcomes; include physical drain where it is part of the operation.
Choose the smallest native setting change addressing that risk and exercise the impacted paths.
If no such risk remains, close H04 with the reasoned default decision; no benchmark is missing.
Do not rebuild old versions, run allocation/cache campaigns or invent performance accounting to
prove a clear design improvement. Quantitative claims, if deliberately made, still need measurement.

## Verification and coverage

**Proposed verification.** Targeted host/operations tests accompany new explanation, export and
cleanup behavior; actual result interpretation remains independently asserted. Scope-end integration
and relevant static checks belong to [72i](72i-integrated-acceptance-and-closeout.md). No live
diagnostic, spill experiment or measurement ran to author this plan.

| Review direction | Owned route |
| --- | --- |
| Query descriptions, versioned plans, counters and whole-read explanation | H01. |
| Collector/OTLP, authenticated signals, source incarnation and delivery | H02 with existing telemetry/Plan71 owners. |
| Structured report versus bounded progress; optional audit/slow-query logs | H02 selected report/progress separation, explicit durable-diagnosis branch. |
| Sort/group spill, memory, retained bytes and physical scratch | H03 actual selected operators and original cleanup owner. |
| Parallel scans, index maintenance, morsel/OLTP settings and cold/warm comparisons | H04 only for a named serious degradation risk; otherwise reasoned default selection. |
