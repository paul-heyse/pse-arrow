# Observability and performance

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the
concurrent Plan66 working tree. Representative source and primary interfaces are
**Interface-checked**; proposed investigation and benefit are **Proposed**.
No live diagnostic, performance campaign or allocation measurement ran.

## Existing responsibilities

[The native knowledge reader](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/xtask/src/development/knowledge.rs)
already explains the same bounded selected-generation query with structured JSON.
It validates cursor/query identity and deliberately omits ANALYZE. This provides a
real current consumer for better execution-plan interpretation, not a need to
invent a generic plan dashboard.

[Query result observation](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_client/results.rs)
records statement duration only when supplied and publishes complete indexed
results after query-end/trailer settlement. [The native telemetry service](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-host/src/telemetry_service.rs)
already owns an optional Collector/OpenObserve installation with bounded batches,
queued delivery, native service identities and distinct metrics/logs/traces.
Its configuration report explicitly says delivery has not been observed.

[Plan68c C02–C05](../../../../plans/68c-surrealdb-v4-query-improvements.md)
owns matched access-path, ranking, spill and mixed-load evidence. The discovery
uses those existing questions without inheriting unperformed measurements or
creating a second qualification ledger. Blueprint §20/§24 owns operation and
evidence semantics.

## Capabilities and deployment availability

| Mechanism | Origin/status | Existing-purpose value |
| --- | --- | --- |
| Structured EXPLAIN and operator metrics | Pre-v4 interface, richer v4 executor | Locate actual access/expansion work in knowledge and canonical reads |
| Graph cache/block/delta and bitmap diagnostics | Earlier foundations plus v4 paths | Separate representation benefit from query syntax |
| Prometheus metrics and OTLP signals | Pre-v4 observability, retained | Correlate request, transaction, storage and process behavior |
| Query memory/spill and scan scheduling | V4 changes, feature/operation dependent | Assess long-query coexistence with small writes |
| SurrealKit JSON/reports and progress capture | Prerelease tooling | Make current schema/test workflows diagnosable |
| Enterprise audit/slow-query pipelines | Edition-dependent | Candidate only where existing diagnosis requires durable records |

The official [EXPLAIN reference](https://surrealdb.com/docs/reference/query-language/statements/explain)
documents structured plans and execution metrics, and warns that their shape can
change. Treat output as version-bound diagnostics. ANALYZE executes work; adding
it silently to an ordinary observer changes that observer's effects and cost.

The [observability overview](https://surrealdb.com/docs/manage/observability)
distinguishes shared metrics/traces from Enterprise record pipelines. Tagged
[v4 telemetry documentation](https://github.com/surrealdb/surrealdb/blob/v4.0.0/doc/TELEMETRY.md)
identifies shared instruments and process-gauge freshness. Available signals do
not establish that the selected launcher forwards every configuration or that
an exporter successfully delivered them.

## Highest-priority compositions

**Explain one complete existing read before tuning.** Join the current request's
selected corpus, predicates, order and result limit with its version-qualified
plan and complete native response timing. Compare candidate counts, fetched
records, expansions, bytes and result equality. This can identify work to remove
through native indexes, projection or traversal changes rather than adding a
parallel observer that repeatedly reconstructs the input.

The discriminating question is where the complete operation spends avoidable
work: database execution, transfer/preflight, hydration or application joining.
A cheaper operator does not establish a faster complete read. A fast top-K query
also needs the same authorized result and deterministic tie policy as its caller.
See [query planning](query-planning-indexes-and-search.md).

**Reuse existing telemetry for physical diagnosis.** Correlate the selected server
incarnation and observation interval with original native operation facts. Use
authenticated aggregate signals and existing Collector processing rather than
building a second bespoke telemetry receiver. Distinguish sampling or queue loss
from application success, and avoid high-cardinality payload copying merely to
join observations. Supplier tracing cannot replace original operation settlement.

**Evaluate spill and mixed load together.** Compare an actual large sort/group
with small canonical writes and background index work. The selected server
feature closure omits GQL: distinct/hash-join spill documented in GQL is not
automatically an available SurrealQL path. Current private scratch/explicit
byte-limit wiring is implemented; live benefit remains to be established.

Assess query, MX and host memory separately, preserve the maintainer's configured
host resource selection, and include spill allocation/cleanup and reader drain.
Cooperative query accounting can wait or overrun and is not a filesystem quota.
Do not choose restrictive scheduling or non-default morsel settings from a
microbenchmark alone. [Plan68c C05](../../../../plans/68c-surrealdb-v4-query-improvements.md)
already identifies relevant cancellation, crash and scratch-failure cases.

## Other useful directions and counterconditions

SurrealKit's [bounded progress contract](../v4-tooling-contracts.md#surrealkitbounded-progress)
could remove terminal-log parsing for an existing operator. Process-global logging,
task propagation and dropped-record reporting remain integration conditions.
Logs describe progress; structured reports and original native outcomes establish
their own facts. Prefer subprocess JSON when in-process embedding expands the
build/runtime boundary without removing meaningful machinery.

Retain ordinary nonexecuting plans when sufficient. Native diagnostic buffers and
the optional existing telemetry service may already provide the useful answer.
Enterprise functionality and other hosting choices remain technically eligible;
their extra operational footprint needs an actual existing diagnostic requirement.

## Evidence required for stronger claims

Use identical selected data, authorization, query meaning and expected results.
Separate cold preparation, warm execution, transfer and physical cleanup. Record
index readiness, server/patch/features, concurrency and snapshot/cursor identity.
Compare total allocation and retained bytes where memory or storage benefit is
claimed; a zero pending-work gauge is not proof of physical release.

No performance ranking was measured in this pass. Prioritize a representative
knowledge query and one canonical read first, then the mixed-load/spill case;
extend to other consumers only when their existing workflow supplies the input
and expected result. Use [coverage](../coverage.md) for evidence scope and the
owning plan for qualification status.
