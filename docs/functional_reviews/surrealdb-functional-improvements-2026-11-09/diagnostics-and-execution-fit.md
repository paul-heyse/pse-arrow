# Diagnostics and execution fit

Supporting assessment for the [principal review](../surrealdb-functional-improvements-2026-11-09.md).
Inspected 2026-11-09 at `4f07766942ff3b21274fdfb686ad483031439462` plus concurrent
dirty source. Observed source and selected supplier contracts are **Interface-checked**;
capabilities and benefits proposed here are **Proposed**. No live collection, load test or
performance measurement ran. Core 3.4, efficient architecture 1.0, process-simulator 1.5,
test assessment 1.2 and the binding apply; AP-07 and DP-09/DP-15/DP-20 are central.

<a id="O17"></a>
## O17 — Explain complete operations through one coherent diagnostic view

**Proposed direction: combine version-bound query plans, native outcome facts and the existing
telemetry installation into a selected-operation explanation.** Reuse the current Collector
and native service identity rather than adding a second custom telemetry receiver.

Today [knowledge inspection](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/xtask/src/development/knowledge.rs)
can explain the same scoped query without ANALYZE. [Native query result observation](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_client/results.rs)
retains a supplied statement duration and complete frame accounting. The
[telemetry owner](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-host/src/telemetry_service.rs)
already configures Collector/OpenObserve, signal-specific streams and queued delivery.
The operator currently has several truthful pieces; availability does not prove end-to-end
delivery or explain where the complete operation spent work.

After this improvement, the operator selects an original request/query and can distinguish
database access/expansion, transfer/preflight, application hydration, waiting and cleanup.
Plans, query/transaction spans and service signals share a known incarnation/interval.
Metrics remain aggregated; request identities belong in bounded traces or selected records,
not unbounded metric labels. Missing, sampled or dropped observations remain visible.

The concurrent [Plan71 diagnostics work](../../plans/71d-fixture-efficiency-and-selected-diagnostics.md)
and [CLI evidence work](../../plans/71a-cli-inspection-and-evidence-lifecycle.md) supply existing
owners for original progress and readable evidence. Integrate SurrealDB facts there rather
than assume missing infrastructure from a stale draft marker or build a second result ledger.

Native `EXPLAIN`/`EXPLAIN ANALYZE FORMAT JSON`, graph cache/block/delta facts, bitmap tiers
and supplied statement durations provide the concrete mechanisms. Prefer the nonexecuting
plan for ordinary inspection; make execution/profiling an explicit owned diagnostic operation.
Return version-qualified useful observations rather than exposing one supposedly stable
operator JSON schema. Output rows alone do not describe input scans or semantic completeness.

Tagged [v4 telemetry documentation](https://github.com/surrealdb/surrealdb/blob/v4.0.0/doc/TELEMETRY.md)
describes shared metrics/log/tracing exporters and explicit OTLP selection. Connect that
supplier export to the existing installation, verify actual delivery, and retain original
native outcome separately. Enterprise audit/slow-query records can add a useful durable view
if that diagnosis warrants their lifecycle; they are not required to use common telemetry.

For schema/test work, consume SurrealKit structured reports as results and its bounded
`ProgressSink`/`capture`/`propagate` records as progress. This removes log scraping while
preserving overflow and task-propagation gaps. Process-wide logger conflicts remain relevant
to embedded use. An empty progress buffer does not prove no activity.

Compared with a generic always-on dashboard, selected explanations reduce reconstruction
and payload retention. Compared with bare Prometheus gauges, they explain the actual request
and its authorization/data cut. Retain simple native buffers when those already answer the
question. No global per-primitive event stream or copied scientific payload corpus is needed.

Delivery examples: a slow protected read distinguishes database filtering from transfer;
a misleading zero gauge is not displayed as physical drain; a missing export is reported as
missing, not successful; a static observer never executes its selected statement. Diagnostic
schemas may break with the new source-owned contract. Observability cannot grant repair,
release or initialization authority; rule changes need the normal decision route.

<a id="O18"></a>
## O18 — Let native execution support useful workloads before tuning concurrency

**Proposed direction: improve query shape first, then use native sort/group spill and shared
scheduling to make selected larger reads coexist with canonical writes.** Preserve the
maintainer's ordinary unbounded host selection. Supplier query accounting and scratch ownership
are separate from imposing host CPU or RAM partitions.

The current working-tree launcher already supplies a private temporary directory, `TMPDIR`
and selected query-memory variables. [Plan68 C05](../../plans/68c-surrealdb-v4-query-improvements.md)
records sort/group spill eligibility and the unselected GQL distinct/hash-join closure.
This is real wiring, not evidence of useful live spill or successful crash cleanup.

The richer workflow is a user exploring a larger selected corpus, grouped result history or
graph explanation while ordinary project/study operations remain usable. The operation exposes
necessary selection, projection and ordering to v4 rather than assembling every candidate in
application arrays. Spill is a physical route for genuinely needed intermediate state; it
does not excuse a full-corpus scan for a narrow question.

Use `SURREAL_MEMORY_LIMIT`, `SURREAL_QUERY_MEMORY_MINIMUM`, `SURREAL_QUERY_MEMORY_MAX` and
owned temporary storage in the execution profile. Treat cooperative waits/overruns, nonspillable
bound values and scratch exhaustion honestly. Reclaim files under the original operation/service
owner; process stop alone does not establish complete cleanup. Native morsel places, OLTP reserve
and scan work quantum are eligible tuning seams only after the composed workload is understood.

| Alternative | Judgment |
| --- | --- |
| Better selective native plan with current defaults | First choice: can remove unnecessary decoding, expansion or sorting before any extra machinery. |
| Native spill plus default shared scan scheduling | Recommended route for genuinely large eligible work, with explicit scratch and truthful refusal/recovery. |
| Tune morsel/OLTP settings for mixed workloads | Conditional on representative coexistence evidence; do not adopt a global preset from one fast query. |
| Application-side materialization and another executor | Retain only when native capability cannot deliver the consumed operation; otherwise adds transfer/live-state and another scheduler. |
| Enable GQL to obtain its operators | Decide with O08's useful path/query capability, not merely to claim all advertised spill operators are enabled. |

A discriminating delivery scenario combines one selective/one broad knowledge query with
small canonical transitions and background index work. Compare equal authorized results,
continuations and useful outcomes; distinguish cold preparation, warm execution and physical
drain. Exercise cancellation while spilling, invalid/full scratch, interrupted service and
reopen, and held-reader memory. Native memory gauges, process RSS and reclaimed file bytes
measure different things. The public [EXPLAIN interface](https://surrealdb.com/docs/reference/query-language/statements/explain)
provides relevant observations; it is not a universal allocation account.

This recommendation removes avoidable materialization and preserves credible physical execution
before protective refusal. It does not claim a throughput/latency improvement, require a broad
benchmark campaign, or suggest compiler-job limits/serial independent work. Changing supported
workload bounds, scheduling or effectful diagnostic behavior needs explicit owned contracts.

## Discovery correspondence, interaction and evidence

| Captured direction | Disposition in this assessment |
| --- | --- |
| Complete read explanation, structured plans and versioned counters | O17 recommended with O07/O08's selected query semantics. |
| Existing telemetry, authenticated signals, incarnation/interval correlation | O17 recommended; verify delivery instead of adding a second collector. |
| Enterprise audit/slow-query and bounded SurrealKit progress | O17 conditional durable diagnostic records; progress/report separation recommended. |
| Native query memory, spill, mixed load and physical observation | O18 recommended execution route after query-shape improvements. |
| Parallel scan/index maintenance and morsel/OLTP tuning | O18 conditional on composed-workload evidence; preserve current host selection. |
| Cold/warm allocation, cache and retained-byte comparisons | Delivery evidence for a claimed benefit, not an automatic measurement programme. |

The [discovery account](../../library_leverage/libraries/surrealdb/briefings/observability-and-performance.md)
and [reusable tooling contracts](../../library_leverage/libraries/surrealdb/v4-tooling-contracts.md)
retain broader sources and limitations. Tagged telemetry and current launcher/query/Collector
sources were inspected. Actual exported signal coverage, selected operator behavior and spill
cleanup remain implementation evidence obligations. Existing Plan68 measurements/status stay
with their owners. Architectural fitness and scientific adequacy are separate: useful diagnostics
or lower query work do not establish a scientifically valid result or completed native settlement.
