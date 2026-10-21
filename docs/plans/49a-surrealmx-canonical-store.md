---
title: SurrealMX canonical store and generated lifecycle
status: complete
date: 2026-10-21
adrs: [ADR-0214]
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [storage, libraries, design, validation]
---

# SurrealMX canonical store and generated lifecycle

This companion develops the canonical-store target of
[Plan49](49-library-native-remediation.md). The coordinator owns all mutable package
status, rule decisions, review dispositions and source-coverage accounting. Package
identities below identify responsibilities, prerequisites and acceptance; they are not
a second progress ledger.

**Implemented:** one managed, authenticated loopback SurrealDB server, built from pinned
3.3.4 source with a narrow SurrealMX `=1.0.0` adapter patch, persistent memory-resident
catalog state and gRPC request transport. The maintainer explicitly selected MX 1.0
on 2026-10-21: “Whatever integration it takes for 1.0, we should pursue that”. The
coordinator records this bounded choice and all other rule decisions at GOV01.
Implementation tests qualify
the selected engine realization; the 0.27 dependency declaration is no incompatibility
finding and does not reopen permission to design for 1.0.

Retain all 16 durable lanes, independent caller processes, short guarded visibility
transactions, caller-independent continuation and existing supervised-server topology.
No new execution daemon or parallel production authority is introduced. Bulk scientific
payloads continue through their protected immutable file owners.

## Basis and affected foundations

The [integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md)
§4.1 supplies R1–R3, F03 and F07; its
[recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md)
corrects lifecycle, sequence, effect and maintenance premises. The complete
[SurrealDB overview](../libraries/surrealdb.md) supplies capability leads; tagged sources
below refine its general statements for the selected versions. Core 3.4, its efficiency
heuristics and Process Simulator 1.5 remain the assessment basis.

Architecture owners are blueprint §3.3, §20.1–§20.6 and §23.2. The authoring baseline described
RocksDB and patched WS; blueprint239 now records the selected MX/gRPC target. GOV01 records rule decisions and GOV02 routes backend, transport,
lifecycle, metadata and commit-contract changes through the applicable decision/design
route before dependent production edits. Existing server topology remains. Authoring
this proposed target does not amend those owners or accept all review recommendations.

The source responsibilities retained by this implementation are:

- `pse-schema/src/catalog/substrate.rs` declares canonical relations/access paths;
  `catalog/operations.rs` declares observational states.
- `pse-codegen/src/codegen/surreal.rs` emits DDL/codecs; the `surreal_*` generators emit
  guarded lifecycle functions. Generated outputs change only by regeneration.
- `pse-operations/src/canonical*.rs` owns bounded typed operations, transport activities,
  exact receipts, result/file protection, conflict retries and connection lifetimes.
- Runtime durable/study/worker consumers own scientific orchestration; supervision and
  maintenance tooling own selected state, server activation and exclusive maintenance.

At authoring, the generator mapped `UInt64` to bounded integral decimal and emitted a key UNIQUE index
per canonical table, without lowering registry enumerations into native literal unions.
Attempt fields encoded ingestion/closed/terminal separately. The implementation below
replaces those representations while retaining bounded decimal integers. These are physical
persistence phases, unlike observational `AttemptState`. `canonical_transport.rs` carries
the original clock and response owner through nested operations. Those are migration
inputs, not WS-specific behavior to discard.

### Tagged integration evidence

| Source | Consequence for the target |
|---|---|
| [3.3.4 server manifest](https://github.com/surrealdb/surrealdb/blob/v3.3.4/Cargo.toml) and [memory manifest](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/Cargo.toml) | Official source resolves MX 0.27.0 and KV 0.21.4. An SDK patch alone cannot upgrade an independently installed server engine; the selected server build changes that integration explicitly. |
| [3.3.4 memory adapter](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs) and [configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/cnf.rs) | Existing persistent memory-backend wiring covers persistence path, AOL, snapshot and sync modes; it refuses versioned reads. |
| [MX v1.0.0](https://github.com/surrealdb/surrealmx/tree/v1.0.0) and [0.27.0…1.0.0 comparison](https://github.com/surrealdb/surrealmx/compare/v0.27.0...v1.0.0) | The bounded adapter delta is read ownership: `get`, `getm`, `get_for_update` and cursor values move from `Bytes` to `ByteSlice`. Inspected constructor, option, savepoint and commit shapes stay recognizable. |
| [MX options](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs) | Explicit finite `DatabaseOptions.memory_limit` and `max_transaction_age` replace unbounded engine defaults. |
| [3.3.4 gRPC engine](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/src/engine/remote/grpc.rs) and [options](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/src/opt/grpc.rs) | Native multiplexed requests and incremental delivery need the repository's admission, deadline, response and drain hooks ported into their actual lifecycle. |
| [SurrealKit beta.6 README](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/README.md) and [migration notes](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/MIGRATING.md) | Select frozen rollout SQL, embedded schema artifacts and sync/replay parity under existing schema and maintenance owners. |

The official version constraint records a dependency baseline, not incompatibility evidence.
The observed read-value delta is ordinary integration work. No static feature omission
identified here requires a final RocksDB/WS target or an intermediate MX 0.27 default.

### Minimal version integration before deliberate capability changes

The [official 1.0 upgrade notes](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0)
and tagged code support a two-stage implementation. STO01 first changes the server-side
requirement to `=1.0.0` and compiles the existing adapter. Its `DatabaseOptions` construction
already uses `..Default::default()` and its error match has a wildcard arm. Do not rewrite
either solely because new fields or variants exist. Change only conversions that the actual
`Bytes`/`ByteSlice` integration requires. The first stage preserves the explicit
`with_snapshot_isolation()` call and current guards; it isolates version compatibility
from intentional isolation, policy and observability changes. It is not a second production
path or a proposal to defer MX 1.0 selection.

STO02 then deliberately integrates useful 1.0 controls/statistics and selects
`with_serializable_snapshot_isolation()` for canonical transactions, subject to RC21.
Standalone SSI is the default in both inspected versions; SurrealDB's explicit SI call
overrides it. The dependency bump alone therefore supplies no stronger-isolation claim.
Tagged 1.0 tracks writing transactions' point reads, including absent keys, and scanned
ranges. Test that the actual SurrealQL/index/adjacency paths carry those premises, plus
write-skew rejection and 16-lane retry/fairness. Keep existing lifecycle/fence guards:
KV serializability is not external-effect atomicity, lifetime authority or commit-order
publication. Read-only snapshots retain their actual stated scope.

The maintainer selected fresh regeneration of all existing data. No old AOL is opened,
converted or retained as a rollback input. Newly generated MX1.0 logs are reopened only
with their matching selected build; the fresh-store contract does not weaken recovery. An encoded transaction exceeding 4 GiB returns `AppendFailed`; retain narrower
application/encoded-transaction bounds rather than splitting an atomic lifecycle operation.
The release also specifies poison-on-unrecoverable-failure. New resource/age/poison errors
initially reach the wildcard fallback; STO02 maps them deliberately into truthful typed
capacity, stale-transaction and failed-store outcomes, without calling a dispatched effect
absent. Keep the original receipt reconciliation on uncertain commit failure.

## Target responsibilities and contracts

### Engine integration and durable configuration

The supervised server owns one MX database and its persistence directory. Independent
Rust/Python callers connect to that server; they do not each embed an engine or open a
shared database directory. The server build and SDK pin move coherently to 3.3.4 with
exact resolution and a reproducible narrow server patch selecting MX 1.0.0. Executable
provenance includes the patched source revision and resolved engine version. A reported
SDK version is insufficient evidence of the running engine.

Inspect returned `ByteSlice` values through the memory KVS adapter's existing `Val` conversion,
conditional comparisons and cursor ownership boundaries; adjust only what the new types
actually require after the dependency change. Views cannot outlive the
transaction/backing that makes them valid. Copy where the generic KVS contract requires
independent ownership, rather than at every internal read. Review `getm` ordering/missing
entries, conditional updates and reverse/ranged cursor progression together. Keep snapshot
isolation explicit: SI for the minimal STO01 upgrade, selected SSI in STO02. Compilation
alone does not establish transaction/conflict or persistence semantics.

Persistent MX uses synchronous AOL commit, `FsyncMode::EveryAppend` and periodic snapshots.
Selected state records persistence path and a finite snapshot interval. Exact intervals
and memory amounts are deployment settings within the existing resource envelope, not
new semantic choices. Pure volatile mode is unsuitable for ordinary canonical execution.
SQL `VERSION` history stays disabled; canonical receipts and retained source/result
relations retain their independent history meaning.

Resident catalog data, engine overhead, pinned versions, RPC buffers and snapshot/replay
workspace share a finite server envelope. Expose explicit finite `memory_limit` and
`max_transaction_age` through the narrow server configuration patch. The engine limit
covers its own domain; an outer supervised cap still covers other server allocations.
Transaction age bounds pinned versions, while stream cancellation must promptly release
its actual transaction. Durable protected-read tickets do not require a permanently open
engine snapshot. Keep both memory admission and physical ownership observable in failures.

Expose `db.stats()`, `active_readers()` and `oldest_pinned_version()` through the existing
bounded metrics/diagnostic owner, so limits and lingering reader pins can be explained.
Do not create a second metrics service. Configure finite asynchronous queue count/byte
limits if that persistence mode is exercised in tests or explicitly admitted later; the
selected production mode remains synchronous. Queue controls and gRPC admission have
different owners. `TxTooOld`, `MemoryLimit`, `AsyncQueueFull`, `Poisoned` and corrupt
persistence retain distinct operator meanings and never become conflict-retry aliases.

Crash recovery, acknowledged commit survival, torn AOL tails, snapshot replacement,
replay and exhaustion are implementing acceptance before cutover. They are scheduled in
STO07/08, not prerequisites to authoring. Lower latency or greater capacity remains a
hypothesis until complete operations are measured at the selected synchronous settings.

### One gRPC request path with equivalent lifecycle ownership

The final production request transport is gRPC. Carry legitimate integration into the
new engine instead of treating `protocol-grpc` as sufficient. Stock 3.3.4 ignores connect
capacity, session readiness does not consume `EngineContext.request_context`, timeout
comes from connection query settings, and default engine disconnect is unsupported.
These are identified patch seams rather than reasons to retain WS as the target.

Keep application admission and reserved control admission separately bounded. One original
absolute deadline covers local queues, session readiness, authentication/selection
restoration, dispatch, conflict retries, response decoding and cleanup. Each RPC consumes
the remaining original budget. A ready wait or reconnect cannot restart the clock.
Before-dispatch cancellation establishes absence only when known; cancellation after
dispatch retains uncertain effect until the original receipt/completion settles it.

`GrpcConfig::max_message_size` limits one encoded frame, not aggregate response size,
buffered `IndexedResults`, decoded allocations or concurrent streams. Carry actual response
owners into spawned/session work and enforce finite operation and aggregate limits.
Renewal, cancellation and physical drain retain reserved progress under saturated
application admission. Exact statement and session-selection acknowledgements precede
any readiness/success conclusion.

Use `stream_items()` for incremental catalog/export reads where useful. Awaited queries
still materialize `IndexedResults`. Rows are provisional until `StatementEnd` succeeds
and the enclosing transaction commits. Late failure discards provisional products; it
never permits irreversible publication based on earlier rows. Drop/drain must release
the actual RPC/transaction while retaining charges until physical completion.

Runtime close releases the logical caller; admitted work retains connection activities.
Whole-connection shutdown closes admission and physically drains RPCs, sessions and retained
transport work. Migrate renewal, settlement, protected reads, maintenance and Python
interruption together. Delete WS production code and transport-only fixtures after the
replacement controls pass. Preserve tests of logical behavior at the new boundary.

### Generated synchronous lifecycle and refusal meaning

The registry declares native literal/tagged enum meaning and legal physical transitions;
the generator emits constraints and matching codecs/projections. Attempts represent
ingestion, closed-awaiting-manifest, closed-with-manifest, terminal and terminal-compacted meaning. The
compacted phase records irreversible removal of completion payloads while preserving terminal
outcome metadata; privileged direct mutation cannot restore that payload or reverse the phase. Close,
cancel and recover can stop ingestion before `manifest` arrives. Scientific terminal
observation does not prove ingestion closure, payload drain or manifest availability.
Explicit projections to `AttemptState` and `StudyPointState` remain exhaustive, including
stored `succeeded` to observed `completed`.

Use `READONLY` only for immutable identity columns after checking all writers. Guard
touches, study cancellation, recovery owner changes, generation increments and operation
compaction remain legal controlled mutations. Generations preserve the full u64 domain
through `18446744073709551615dec`. Assertions cover same-phase changes, payloads and
fences as well as phase pairs. Guarded functions own cross-record decisions; field
assertions backstop them. A registry-declared synchronous event compares full
before/after rows to keep the seed-manifest pointer immutable after terminal
publication; non-NONE pointers match the closed manifest. This adds no scientific
eligibility rule. Refusal rolls back the statement or explicit transaction, including
sibling mutations. No asynchronous event replaces synchronous readiness.

Declare refusal codes/classes once and generate code-prefixed `THROW` values. One decoder
first verifies native `is_thrown()`, then recognizes known codes. Unknown codes and ordinary
schema `Internal` errors fail closed. Schema refusal is an invariant breach, not prose to
reinterpret as a domain decision. The selected prefix form needs no maintenance-only
value parser in default builds.

`CanonicalError::Refused` retains class, code, operation identity/stage and truthful effect.
Conflict retry exhaustion becomes typed. Runtime worker/durable/study policy consumes
these meanings instead of wrapper shapes/prose; worker loss becomes an infrastructure
termination. Acknowledged study cancellation, uncertain dispatched effects and replay
permission stay distinct. Retryability grants no replay; preserve fresh-premise rereads
and original receipt settlement. Generate every attach/release owner guard from one holder
declaration, closing the missing analysis-owner release branch.

### Record identities, native graphs and necessary serialization

Move `WHERE key =` reads to direct record fetches and whole-table `key >` paging to bounded
record-id ranges before index removal. Then emit immutable `key = record::id($this.id)`
assertions and remove redundant UNIQUE indexes only after replacement statements/plans
pass on MX. Keep legitimate compound selection indexes.

Select `LIGHTWEIGHT` only for endpoint associations with no edge fields, events,
changefeeds or live-edge consumers. Keep attributed relations classic and `ENFORCED`,
and enforce study-dependency endpoints. Use bounded `INLINE EDGES 64` and `INLINE
REFERENCES 32` caches for reached low-degree vertex access with ordinary high-degree
fallback. Source fit or implementing evidence may justify other finite thresholds.
Page readiness traversal within the relation idiom instead of limiting after full expansion.
Preserve any independent edge identity used by receipts/portable consumers before choosing
lightweight representation. Fewer records alone do not prove semantic equivalence.

Endpoint/reference checks do not replace guard/sentinel writes. `FOR UPDATE` tracks
observed records, not arbitrary range phantoms. Creator/reclaimer, dependency installation/
retirement and absent-row decisions retain common conflict keys.

F03 contains two different questions. Visible-run paging requires commit-order sequence
serialization and the full u64 extent. A sequence draw does not order its caller's
publication commit. The broader `ProblemTurns` mutation turn may queue unrelated work;
STO06 investigates actual 16-lane pacing and narrows only demonstrably excess exclusion,
preserving cancellation fairness, guard races and no-lost-page paging.

### Schema artifacts and privileged maintenance

`pse-schema` remains SQL/domain declaration authority. Generated artifacts feed SurrealKit
`=1.0.0-beta.6` embedding and frozen forward rollouts. Toolkit content hashes do not replace
the DDL-plus-codec interpretation identity; toolkit type generation is no new Rust/domain
authority. This companion owns storage definitions and their toolkit integration.
[49g](49g-contracts-maintenance-and-tooling.md) owns shared privileged commands and caller
migration, consuming the schema/frozen-function products defined here.

Installation, restore activation and reconciliation need exclusive maintenance before
ordinary writers exist. They cannot require an ordinary function whose own admission
presupposes active canonical state. Keep domain maintenance functions separate from toolkit
definition application. Direct DDL and embedded/toolkit sync/replay must produce equivalent
schema/behavior despite beta.6 preprocessing. Cold catch-up and modified frozen SQL need
controls. Definition rollback is not data rollback. Register namespace ownership before
suite/parity database creation; disposal remains with the existing fixture owner.

SurrealKit beta.6 is already selected in `.config/surrealkit.toml`.
`scripts/surreal_rollout_producer.py`, `scripts/surrealkit_maintenance.py` and
`canonical_maintenance.rs` are existing foundations to adapt, not infrastructure to replace
by name. Preserve exact pending nonce/input/phase settlement, authentic child drain and
activation controls. Inventory the toolkit's actual client alongside product clients in
STO04; migrate its endpoint/transport integration without assuming stock toolkit support
or inventing an incompatibility. Frozen artifacts and exact maintenance settlement remain
required throughout the transport and engine change.

## Execution packages

| Package | Required input | Delivered behavior and replacement |
|---|---|---|
| STO01 | GOV01/02 storage decision route; no gRPC or SSI prerequisite | Smallest 3.3.4 server requirement upgrade to MX 1.0.0 plus required conversions, preserving current SI; replace current engine build/version detection. |
| STO02 | STO01 qualified version integration, RC21 and supervised state/resource owners | Deliberate SSI, persistent finite engine/stream ownership, typed new errors and bounded stats; replace RocksDB startup assumptions. |
| STO03 | Accepted lifecycle/refusal rule slice and registry | Generated physical lifecycle/refusals and migrated consumers; delete flag/prose reinterpretation. |
| STO04 | STO01 SDK plus transport/lifetime contracts | gRPC admission/deadlines/uncertainty/response/drain; migrate all WS production consumers. |
| STO05 | STO03 identity/graph declarations and selected engine | Direct records/paging and native graph fit; delete replaced indexes after caller migration. |
| STO06 | STO03 typed conflicts and working 16-lane store | Commit-order publication plus justified local pacing; delete replaced broad exclusion if safe. |
| STO07 | STO01–STO06 target integrated; privileged installation/activation interfaces | Fresh fenced initialization, regenerated data and one activation; retire replaced production paths. RES migration is not a prerequisite. |
| STO08 | Functional packages and affected consumers integrated | Assembled crash/concurrency/public-boundary acceptance, joined to QUAL01–QUAL05. |

### STO01 — Integrate MX 1.0 at the server boundary

Build exact 3.3.4 source with MX 1.0.0 first and adapt read ownership only where compilation
and the actual conversion contracts require it. Preserve existing options/defaults and
explicit SI in this stage. Scope the patch to actual adapter gaps; do not copy the engine
into product code. Executable
identity distinguishes stock 3.3.4 from the selected build. SDK/protocol changes follow
exact-pin/targeted-lock rules without wholesale resolution.

Targeted controls cover missing/present single/multi reads, conditional put/delete,
forward/reverse cursor continuation, savepoint rollback, commit/conflict mapping and
values retained across transaction release. Include fresh framed-log recovery, poisoned/corrupt persistence and encoded append bounds.
Historical AOL conversion is removed under the fresh-data decision.
Compile the patched server and SDK, not just
an application talking to a stock binary. Delete superseded detection/build paths after
the target is exercised. No fallback to 0.27 is scheduled.

### STO02 — Own persistent resident state and transaction lifetime

After the minimal version upgrade, select explicit SSI and wire durable/finite MX options,
truthful new error classes and bounded engine statistics into supervised startup, stop and
restart. Preserve selected
state, endpoint authorization and shared-process admission; refuse ordinary startup without
synchronous persistence. Surface memory/age errors without inventing absent effects.

Targeted controls distinguish acknowledgement from unacknowledged dispatch, reopen snapshot
plus log, release stream-pinned versions after cancellation/drain and exhaust a finite
catalog allowance. A snapshot must fit the enclosing envelope with 16 admitted lanes.
Add point-read/write-skew, absent-key and scanned-range interference controls through the
actual server, along with retry-budget/fairness and retained guard semantics. Verify the
statistics explain a pinned reader and clear after physical drain. Keep asynchronous
queue-full tests scoped to that mode; they do not justify changing production durability.
Compare complete durable operations, not volatile-engine microbenchmarks.

### STO03 — Generate lifecycle meaning and consume typed failures

Derive complete transitions from close/cancel/recover/manifest/seal/compaction/retirement
before replacing fields. Migrate generated functions, codecs, projections and every
writer/reader together. One integration owner controls registry/codegen surfaces. RES01 coordinates any shared
declarations through that owner but is independently scheduled; existing result representation
is sufficient for this store scope. Generated output is never edited by hand.

Controls reject unknown/illegal state on CREATE/INSERT/UPSERT/UPDATE and roll back a
multi-row illegal update. Accept close-before-manifest, recovery, cancellation, legal
compaction and boundary u64 generations. Cover all attach/release holder kinds. Verify
nested/transactional refusals and unknown-code failure, distinct worker loss, conflict
exhaustion, acknowledged cancellation and response loss. Delete prose matches, unreachable
retry tails and obsolete flag-state fixtures when the replacement passes.

### STO04 — Migrate transport hooks and all consumers

Port operation context through session readiness and each RPC, retaining reserved control
progress and aggregate accounting. Shutdown drains actual transport/session work, including
interruption while restoring authentication. Migrate endpoint constructors/callers together;
there is one final request transport rather than a production backend switch.

Controls saturate application slots then renew/cancel/drain; expire the original clock
during readiness and retry; lose acknowledgements before/after dispatch; exceed aggregate
allowance with individually legal frames; and drop provisional streams before statement/
transaction completion. Reuse canonical transport/lifetime behavior controls with fault
injection at the actual gRPC adapter. Delete WS once consumers and these controls pass.

### STO05 — Replace redundant access machinery with native records/graphs

Migrate supervision paging and key predicates first. Qualify CREATE, UPSERT and
`INSERT ... ON DUPLICATE KEY UPDATE` key assertions, then remove each redundant index.
Inspect MX plans for bounded record ranges and real readiness traversal. Historical
RocksDB evidence remains historical; it is not target-engine acceptance.

Classify relations by fields/consumers, then generate the selected classic/lightweight
and bounded inline forms. Controls cover duplicate association, attributed edge values,
high-degree fallback, absent endpoints and both creator/retirement commit orders.
Retain exact receipt membership and semantic edge occurrence identity.

### STO06 — Preserve sequence order and settle excess pacing

The oracle is visible-run paging: every committed run after a cursor remains discoverable
once, with no later lower sequence appearing behind the cursor. Preserve the guarded
sequence commit. Inspect local wait/conflicts for starts, selection, cancellation and
independent mutations across 16 lanes; narrow `ProblemTurns` only when a working
replacement preserves every relevant race. If broad pacing is necessary, record the
precise dependency at the coordinator rather than claiming a win. Delete replaced locks
and implementation-specific fixtures in the implementing change.

### STO07 — Initialize and activate one fresh target

The maintainer explicitly waived existing-data preservation: all data is generated and
can be regenerated. Close old admission, fence stale callers, physically drain owned
operations and stop the old server. Use the existing owned rebuild/state lifecycle to
discard the old catalog/payload state and create fresh database, credential, authority
and payload incarnations. No export/import, retained-batch reconciliation, legacy-log
conversion, backfill or old-data rollback path is required.

Install final generated definitions/frozen functions under exclusive maintenance. Check
actual server provenance, synchronous persistence, finite resource configuration,
interpretation identity and protected file admission before one fenced activation.
Regenerate fixtures and representative histories from current source declarations.
Interrupted initialization stays closed and resumes through exact pending nonce/input/
phase settlement; it never reopens the old authority. Old callers must refuse.

RES01–RES03 and broader TOOL04 remain independently owned work. Include only shared
installation/activation interfaces and existing maintenance callers actually changed by
this store's lifecycle/transport. Remove replaced RocksDB activation/configuration and
obsolete transition callers after replacement acceptance. New data still requires
closed backup/restore, protected access and durable recovery.

### STO08 — Qualify the assembled store once

Join affected integration/component/Python journeys and manual static checks to
[49h](49h-qualification-and-design-publication.md), after functional integration. Exercise
two independent caller processes, all 16 durable lanes, caller exit, server crash/restart,
fsync acknowledgement, torn-tail recovery, snapshot interruption, replay and interrupted
fresh initialization. A process kill alone does not simulate power loss; name each fault model.

No acknowledged receipt disappears; uncertainty is never reported as absence; stale
authority cannot reopen admission; protected readers survive permitted retirement;
cancellation releases physical work/charges. Native/Python journeys preserve scientific
outcomes, partiality and exact bits. Measurements compare whole durable operations at
equal sync settings; storage microbenchmarks do not qualify throughput or latency.

## Acceptance, handoff and rule boundaries

**Proposed verification:** use relevant `just check-package` and `just unit-package`
routes during implementation, with explicit force-validation for Arrow consumers. Select
patched-server adapter commands against its actual source tree and run within
`scripts/pse-env --`. No invented recipes or unrun passes. Scope-end hygiene, integration
and manual checks run once through QUAL01–QUAL05 after functional scope.

**Interface-checked/source-derived authoring basis:** repository owners, integrated review/audit and
tagged integration sources above. **Not run during authoring:** builds, tests, probes,
installation, migrations, dependency changes or product qualification. No new recovery,
capacity or transport-equivalence claim was made by authoring. The coordinator's execution
checkpoint and Outcome own subsequent acceptance and its limits.

The coordinator owns RC19/RC20/RC21 and the lifecycle, metadata and commit-contract decisions.
[49b](49b-protected-results-and-selection.md) owns result-selection obligations; 49g owns
shared maintenance/tooling migration. An agreed interface permits design/preparation;
working migrated consumers establish implementing acceptance.

## Outcome

**Implemented and Tested, 2026-10-21:** all STO01–STO08 are complete at the
[coordinator's store Outcome](49-library-native-remediation.md#outcome--plan49a-canonical-store),
which owns commands, composite repaired acceptance, mistakes, deliberate choices and limits.
Enduring contracts live at blueprint §3.3/§20/§23.2 and selected qualification at §24.2
through blueprint239/240; ADR-0214 remains proposed. The active coordinator and downstream
boundary/maintenance companions still consume this design, so it remains retained under
ADR-0096. It neither closes combined findings independently nor closes the Plan49 series.
