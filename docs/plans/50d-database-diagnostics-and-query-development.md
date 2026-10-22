---
title: "Plan 50d: Database diagnostics and query development"
status: complete
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/50-codex-effectiveness-and-diagnostics.md
doc_topics: [diagnostics, libraries, storage, performance, validation]
---

# Plan 50d: Database diagnostics and query development

The foundation paragraphs describe the named authoring baseline. Selected implementation,
inquiry outcomes, corrected mistakes and qualification conditions are recorded in the
[coordinator Outcome](50-codex-effectiveness-and-diagnostics.md#outcome).

## Purpose, sources and current foundation

Deliver O10–O13: inspect the actual query and storage behavior, develop queries/schema with
early useful feedback, rehearse changes in disposable fixtures and retrieve scoped log/trace
evidence. The [coordinator](50-codex-effectiveness-and-diagnostics.md) owns state and optional
inquiry dispositions. **Interface-checked:** `bb61ae3f5`; **Acceptance criteria:** this design/acceptance.

The new working reference `docs/libraries/surrealdb_tooling.md` was read comprehensively.
Its §§5–12 and 14–15 contributed concrete source-backed mechanisms below; orientation does
not establish what this selected binary exposes. The custom build remains SurrealDB 3.3.4 at
`1ac2b1dba7cd177621a4cf5d507caebeb2414395`, MX1.0 and Surrealism with the local patches.
Do not switch engines, replace it with the component launcher or adopt another schema authority.

Current source already has native MCP, generated schema/functions/indexes, inline adjacency,
SurrealKit beta.6 built against the patched gRPC SDK and root VIEWER selection authentication.
`canonical_transport::query` attaches `CanonicalStatistics` through `Query::observe_statistics`
under a current observation. It publishes statement received/success/failure/timed/untimed facts
and `canonical.query.latest_sample`; the buffered/streamed terminal paths are covered by existing
controls. This is not missing instrumentation. A latest sample is not a full statement timeline.

## Target responsibilities and interfaces

The production query builder owns SQL, binding construction and source/generator identity.
Diagnostics exposes that owned representation, native schema/index inspection and plan output.
Static explanation and actual execution experiments are distinct commands/actions. No handwritten
parallel diagnostics-SQL catalogue is introduced. Native output is preserved by reference;
selected stable meanings are interpreted tolerantly rather than promising a universal plan schema.

The supervisor owns selected listener/service/credential validation, metrics retrieval and log
configuration/lifecycle. A02 provides the common selected-service validation slice. Endpoint-specific
consumers still construct their own exact loopback `/mcp` or `/metrics` route; never turn header
validation into arbitrary authenticated URL access. No inspection starts/reconciles a service.

Existing registry/codegen declarations remain schema authority. SurrealKit plans, sync/replay,
parity and permission tests are mechanisms beneath that authority. Readonly/module metadata is
explained through the existing attestation owner, without another module authority or invocation.

## Packages

<a id="D01"></a>

### D01 — Inspect the actual query, schema, index and graph path

Begin with `CanonicalStore::study_candidates` and its actual readiness query/index. Factor only
the owned construction needed for dispatch and inspection to consume the same SQL/bindings.
Expose statement identity, parameter names/types/source and safe selected values, namespace/db,
declaration/generator origin and whether surrounding function/transaction work is included.
A wrapper function's plan does not establish visibility into every nested operation.

Compose `INFO FOR TABLE ... STRUCTURE`, `INFO FOR INDEX ... ON ...`, `EXPLAIN FORMAT JSON` and,
under explicit experimental selection, `EXPLAIN ANALYZE FORMAT JSON`. Compare with current
trailing `EXPLAIN FULL` behavior for the exact query shape. `AnalyzePlan` inherits its inner
access mode and executes/drains it: do not classify arbitrary ANALYZE as read-only. Keep bound
parameters bound and retain raw output/actual fixture conditions.

For actual canonical graph consumers, interpret direction, table, pushed limit/order and
`predicate_scope` (`key/payload/record`). Source 3.3.4 exposes positive `cache_hits/cache_misses`
and folded `block_hits/delta_hits`; their absence need not mean unavailable instrumentation or
a demonstrated miss. Existing declarations already generate `INLINE EDGES 64 INLINE REFERENCES 32`.
Diagnose whether a traversal uses them or falls back; do not "adopt" them again. TOOL08 supplies
the semantic bounded-neighborhood response when delivered. Keep result completeness separate
from physical access observations; `LIGHTWEIGHT` conversion is not a tooling optimization.

Acceptance compares the inspected and dispatched query, actual binding safety and expected
readiness index on a populated fixture. A graph case retains revision/continuation, distinguishes
predicate placement and omits unavailable counters honestly. D01's readiness consumer does not
wait for all of TOOL08.

<a id="D02"></a>

### D02 — Identity-bound native metric snapshots

Extend the supervisor with a selected-field/family metrics observer: one authenticated request
to the frozen selected listener, identity recheck around observation, and a result carrying
interval, service/process identity, values and availability. Reuse `pse-selection ON ROOT ROLES
VIEWER`; source `OperatorAuth` checks root scope, not OWNER role. Retain the credential internally.
It is less privileged than OWNER, not an established metrics-only role.

Start with the eleven existing `surrealdb_storage_surrealmx_*` gauges and relevant native
process/query/transaction families. Names/types/descriptions belong to the exporter; do not
create a general metric catalogue. Preserve zero, normal absence of `oldest_pinned_version`
with no reader, unregistered/unavailable metric, request/auth/parse failure and changed identity.
Retain selected raw exposition only where useful and protected.

Compare two snapshots around an explicit operation without claiming exclusive attribution on
a shared server. Gauge differences are not counter deltas. Estimated key/value memory is not
RSS/disk and async queue zero is not synchronous-AOL durability proof. Surface poison/reader/
version retention facts as observations, not unproved leak causes.

Reuse `surreal_mx_check` fixture scenarios: root VIEWER access, anonymous filtering, held reader,
release, normal absent field and changed server identity. A02 validation is the prerequisite;
no new fixture/server is needed for every metric. Never print authorization output.

<a id="D03"></a>

### D03 — Exact-parser and advisory language feedback

Use D01's first owned query and generated schema context for source-mapped early checking.
Handwritten SQL retains literal/edit-owner coordinates and binding names; generated SQL retains
generator declaration/output origin. Do not start by externalizing every embedded query.

The exact selected server's `validate --stdin` is the syntax oracle. I02 evaluates language
server 0.6.3 revision `5aa6a12a29b7def0e7df662c09ac9203e66f5033` with
`check --stdin --stdin-filename ... --workspace ... --param ... --json`. Headless check does not
connect to the database; its builtins point to `9d9a5b069`, and grammar gaps are known.
Preserve diagnostic codes, source ranges, hints, scan losses and 0/1/2 exit meaning. Context
skips can lose schema even with exit zero. Engine-valid syntax stays valid.

Select usable advisory assistance, assistance with explicit unsupported constructs or no-adopt
for that version. In the last case deliver exact-parser/source feedback and leave semantic
assistance explicitly unavailable rather than pretend it was qualified. A positive decision
pins the tool/revision deliberately and adds the existing command/LSP interaction, not another
parser authority. Formatter beta.3 remains optional pending provenance and round-trip behavior;
generated formatting belongs in its generator, never frozen rollout artifacts.

<a id="D04"></a>

### D04 — Disposable query/index/schema rehearsal

Compose C04/resource ownership with the existing SurrealKit wrapper and production declarations.
Show the registry/generated-schema delta, actual query/static feedback, native/toolkit plan,
fixture execution, behavioral/permission tests and authentic drain. Build a representative
fan-out from the diagnosed operation; this is neither a generic CRUD score nor a new performance
campaign. An actual index/query alternative produces correctness and plan/work observations
plus a proposed source change, not automatic canonical deployment.

Use current wrapper routes: `sync --no-prune`, `rollout plan`, `rollout up --complete` and
`test --schema-from sync|both --no-seed --keep-db --json-out ...` where selected. Add missing
`rollout lint` forwarding narrowly if useful; do not assume generic rollback exists. Existing
`test-both` already reaches sync/replay schema parity; expose that report rather than implement
another schema diff. It does not prove arbitrary data rollback or canonical activation.

Use beta.6 `permissions_matrix` for an actual restricted actor such as the inspection account.
The current wrapper binds exact fixture bytes, rejects `actors` and admits only `sql_expect`
cases. First extend and qualify that admission narrowly for the selected actor/case; upstream
support alone does not make a new TOML fixture runnable here. Preserve target-override, HTTP
and seed restrictions, the authenticated gRPC session and its drain owner.
Assert resulting state independently: VIEWER CREATE may return `[]` rather than a permission
exception. Reuse the independent privileged MCP readback controls; lack of an error is not
write authorization.

Expose module explanation from `inspect_archive` and `validate_study_module`: selected artifact
versus installed registration/bucket, sole readonly export/signature and local manifest limits.
Direct-binary `module info`/`module sig` may supply offline detail; no execution/reinstall occurs
merely to inspect metadata. Preserve local patched deadline/memory/pool semantics.

First consumer is a readiness index/declaration change with sync/replay parity and relevant
actor behavior. Persistent immutable publication/maintenance/activation remains a separate
existing owner operation; this rehearsal never bypasses its pending nonce/settlement guards.

<a id="D05"></a>

### D05 — Scoped logs and existing statement observations

Provide selected log/statistic retrieval through C01/C03 and runtime inspection using the
existing statement observer. A statement sample's server wall duration is not local active CPU.
Only extend current retention if a real diagnostic needs more than the latest sample; preserve
loss/selection and do not introduce a duplicate event stream or LIVE subscription lifetime.

The current server fixes `--log=warn`; initialization creates filters. Add explicit diagnostic
log configuration with inspection separate from application. A scoped candidate is
`warn,surrealdb::mcp::audit=info` plus supported JSON output. Verify the selected binary's filter
parser. Preserve `BoundedLog`, private paths and no query/row payload in the audit target.

Applying/restoring filters needs explicit owned restart; no live reload was established.
Existing `reconfigure` is allocation-only and must not silently gain unrelated semantics. Use
quiesce/drain/stop, selected launch configuration/readmission and a new service identity, reporting
interrupted application honestly. A logging-only change is not a schema/scientific interpretation
change, but actual owner generation/qualification promises still apply.

Acceptance observes one scoped MCP audit record with tool/scope/outcome/error kind/duration,
correct query-statistic association and restoration of baseline warn behavior. Verify unrelated
services untouched, no secret output and stopped/pending state after interrupted activation.

<a id="D06"></a>

### D06 — Close optional ecosystem investigations

Resolve I08/I11/I12 with concrete consuming questions and adopt/no-adopt consequences:

- One owned OTLP capture tests actual propagation Rust→gRPC→database→Surrealism. Useful partial
  spans retain gaps; exporter activation alone is not correlation. An on-demand collector can
  be enough. Permanent Prometheus/Grafana/Loki/Tempo is not selected without longitudinal demand.
- A new operations MCP is initially unnecessary. Select it only if repeated structured invocation
  of delivered owner commands materially improves Codex use, with no duplicated state/credentials.
- GQL must distinguish server endpoint, compiled core and enabled MCP tool. Server `gql` is not
  selected, while MCP pulls core GQL. Investigate only a useful TOOL08 expression, with actual
  read/write permissions; do not widen deployment or change canonical edge semantics for tooling.
- Server Console is a separate diagnostic build/target from E04's application executor. E04 owns
  its feasibility/effects; environment variables alone do not enable an omitted server feature.
- The exact upstream language-test harness supports remote HTTP/WS, planner/permission/root-state
  differences and remote-exclusive cases. Use a minimized engine reproducer on an owned disposable
  server only after a concrete engine bug; never run the corpus or accept/overwrite outputs against
  the canonical store. This does not qualify gRPC ownership. `crud-bench` is only a workload lead
  for a diagnosed missing measurement, with exact revision/narrow features; no default benchmark.

Document no-adopt decisions and the condition that would reopen them. Other backend tools,
Cloud, Sync/CDC, Agent Memory, PostgreSQL wire and TS clients have no selected consumer here.
Do not install their ecosystems merely to close a tooling inventory.

## Dependencies, verification and sources

**Tested:** selected package and composed acceptance are recorded with commands, modes,
zero-failure baseline and exclusions in the [series Outcome](50-codex-effectiveness-and-diagnostics.md#outcome).

A02 supplies exact service validation; C04 supplies disposable lifetime; Plan49 declarations,
SurrealKit and module owners are existing foundations. D01/D02 can proceed independently;
D03/D04 consume the actual query/schema slices. D05 uses C01/C03 and explicit lifecycle actions.
No whole-companion barrier is needed. 50g owns composed acceptance and scope-end checks.

**Acceptance criteria:** query/binding equivalence, real index/graph plan controls, root VIEWER/anonymous
metrics, LSP diagnostic/scan controls, sync/replay parity, actor readback, module mismatch and
interrupted log activation. Actual measurements retain fixture/concurrency scope. Ordinary
source/metadata checking needs no full scientific solve.

Primary mechanisms: [3.3.4 EXPLAIN](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/exec/operators/explain.rs),
[graph scan](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/exec/operators/scan/graph.rs),
[MX statistics](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs),
[metric authentication](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/server/src/observe/session.rs),
[LSP checker](https://github.com/surrealdb/surrealql-language-server/blob/5aa6a12a29b7def0e7df662c09ac9203e66f5033/src/native/check.rs),
[SurrealKit tester](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/crates/surrealkit/src/tester/runner.rs),
[module architecture](https://surrealdb.com/docs/learn/extensions/guides/module-architecture),
[language harness](https://github.com/surrealdb/surrealdb/blob/v3.3.4/language-tests/README.md).
Local routes: `canonical_transport.rs`, `canonical_studies.rs`, `canonical_study_module.rs`,
schema/codegen declarations, `surreal_server.py`, `surrealkit.py`, `surrealkit_maintenance.py`,
`surreal_rollout_producer.py`, `surrealism_module.py` and existing MCP/MX fixtures.

## Outcome

**Implemented:** the selected packages in this companion, with the optional mechanism decisions
recorded at the coordinator. **Tested:** package-local controls and selected composed consumers
under the commands, modes, zero-failure baseline and exclusions in the
[series Outcome](50-codex-effectiveness-and-diagnostics.md#outcome). That owner also records
corrected mistakes and deliberate deviations; this companion does not duplicate live status.
