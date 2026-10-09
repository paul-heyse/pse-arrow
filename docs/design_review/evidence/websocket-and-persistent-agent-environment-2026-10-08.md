---
title: WebSocket and persistent agent environment evidence
date: 2026-10-08
status: review
---

# WebSocket and persistent agent environment evidence

This supporting document records bounded source and host observations for the
[principal review](../reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md).
It supplies evidence rather than a second architectural verdict or finding ledger.
The review was requested independently of the remaining failing-test repair.

## Baseline and effects

**Interface-checked:** the inspected checkout was dirty `main` at
`2410880efc5ab20e623006fde78e624044d3358d` on 2026-10-08. The uncommitted Plan 28
preparation, publication, interruption and retention changes are part of the examined
source. HEAD does not identify that complete tree or qualify installed binaries.
Repository mapping, library research and the principal assessment were read-only.
The coordinator publishes documentation; this review changes no application code,
dependencies, service configuration, database contents or resource allocation.

No product tests, builds, server queries, timing experiments, migrations, cleanup or
process signals were run for this review. Existing records keep their original
conditions and are not new tests or measurements of the proposed design.

The operator confirmed automatic cleanup of successful disposable test data, with
failed, interrupted and incomplete evidence retained until explicitly released; and
concurrent functional work with reserved capacity for timing-sensitive tests.
These are target requirements. Existing preserved materials are not thereby disposable.

## Existing persistence and machine conditions

**Interface-checked:** `surreal_server.py::serve` already starts the released server
with `rocksdb://<state>/database?sync=every&versioned=false`. The temporary-directory
argument selects spill/scratch space; it does not select an ephemeral database.
`start` creates a transient user service with `Restart=no`. `unit_name` derives the
service name from the owned state directory. Thus persistent storage, continuously
resident service and coordinated cross-session ownership are distinct properties.

**Measured, bounded host inventory:** read-only `ps` inspection found seven running
SurrealDB 3.3.0 processes with disk-backed RocksDB endpoints. A separate configuration
inventory at approximately 22:44 America/New_York found twelve saved state
configurations beneath `~/.local/state/pse-arrow`, including both `pse.substrate.v1`
and `pse.substrate.v2`. Saved configurations are not evidence of live processes;
the seven processes do not exhaust all possible application/fixture state.
No process or state was selected for removal.

Read-only `/usr/bin/systemd --version` and `lscpu` identified systemd
255.4-1ubuntu8.17, an AMD Ryzen 9 9950X3D, sixteen physical cores and thirty-two logical
CPUs on one NUMA node. `free -h` reported approximately 188 GiB usable physical memory
and 114 GiB available at one snapshot; available memory is not reserved capacity.
`df -h` reported approximately 26 GiB available on the repository filesystem.
These are host observations, not workload capacity or performance qualification.

`surreal_server.py::reference_resources` materializes the reference declaration as
a 160 GiB envelope: 16 GiB server, 140 GiB worker process and 4 GiB observer, with a
128 GiB runtime pool and sixteen physical CPU lanes. `ensure_execution_placement`
explicitly records `physically_reserved: false`. Its selected width consumes this
host's sixteen physical cores; SMT siblings do not provide another independent
sixteen-core timing allocation. The inspected parent `pse.slice/memory.max` was
`max`, while commands and reference deployments have their own limits.

Context7 supplied current systemd documentation; applicability was checked against
the installed 255 family and the primary
[v255 resource-control source](https://github.com/systemd/systemd/blob/v255/man/systemd.resource-control.xml).
CPU quota and memory maximum are ceilings; CPU weights distribute contested CPU
time and allowed CPU masks restrict placement. They do not establish exclusive
host ownership. The existing source itself preserves this distinction. No stronger
reservation or isolation capability was exercised on this host.

## Test identity, ownership and disposal

**Interface-checked:** `testing.rs::canonical_fixture_store` creates a unique
`canonical_test_<uuid>` database on the selected existing server. Schema installation
is separate from server startup. `CanonicalStore::initialize_with` currently retries
only acknowledged transaction conflicts, rereads schema inventory, and does not
replay uncertain submissions. The sixteen-initializer source control exists; it was
not rerun for this review.

`FixtureLifetime::drop` waits for database removal even during test unwinding.
`std::thread::panicking()` changes cleanup-error reporting, not whether removal is
attempted. Ordinary Rust test failure therefore does not pin its fixture database.
Abrupt process death can bypass Drop and leave the same class of database behind.
`remove_isolated_fixture` drains result-reader releases before removal; that useful
drain obligation is separate from the missing test-outcome disposition.

Python `conftest.py::canonical_substrate` returns the configured state path for the
session. `workflow.rs::NativeRuntime::new` opens its configured database; it does not
allocate a per-test database there. Unique study/run identifiers do not independently
isolate problem histories or namespace/database control. The statement in
`validation-assessment.md` that operational databases are owned per test does not
match this inspected Python construction path.

`ManagedStudyFixture::__enter__` obtains a state-wide empty `O_EXCL` lock file,
waiting at most sixty seconds. That file contains neither owner PID/start identity
nor a recoverable reservation. The supervisor's `state_lock`, worker-slot and observer
ownership mechanisms are stronger: they use `flock`, process start identity and/or
actual cgroup liveness. Those existing mechanisms are candidate suppliers, not proof
that the qualification lock already uses them.

`ManagedStudyFixture::__exit__` ignores the exception arguments and calls `close`.
After successful caller/worker/service drain, `close` removes its control directory,
including `caller.log`, and its lock. This behavior applies after an ordinary failed
assertion as well as a pass. Evidence elsewhere in assessment logs may survive, but
it is not equivalent to preserving these fixture controls and stored scientific data.

## Retained results and assessment evidence

**Interface-checked:** `canonical_result_retention` and runtime `workflow::retention`
provide explicit study/analysis withdrawal, settled-run retirement and bounded
reclamation. Active readers and retained scientific dependencies can prevent disposal;
lifecycle receipts remain. Source reclamation has its own head/root/protection checks.
Neither owner derives scientific-history expiry from test status or elapsed age.

`validation.py::fresh_output` refuses overwrite and creates a unique assessment path.
`mark_latest` supplies a convenience pointer rather than evidence authority.
`validation_receipts.py::reuse_checks` binds retained observations to explicit parent
and original-origin paths/digests and verifies referenced artifacts. A successful
report can therefore remain required by another report. Age or a moved latest link
does not make it disposable. No automatic evidence purge or explicit evidence-release
operation was found in these examined owners.

Terminal test reconciliation distinguishes missing results from passing ones.
A lost or absent terminal report cannot establish safe automatic success cleanup.
Referenced successful evidence, operator-retained history, failed fixtures and crash
leftovers require deliberate classification rather than a directory-prefix rule.

## Transport capability and contrary evidence

**Interface-checked at SurrealDB 3.3.0:** the library researcher examined the matching
skill corpus and upstream tag at commit `238bfeb11f5725bebed370167656748df8067595`,
with current official documentation and Context7 as additional sources.

| Consumed boundary | Source-supported fact and limit |
|---|---|
| Native values and errors | The native WS engine uses library FlatBuffers codecs, retaining bytes and structured query errors. This need not introduce a JSON boundary. |
| Query deadline | gRPC propagates `Config.query_timeout`; WS does not consume it. The WS RPC request envelope and engine context have no equivalent absolute deadline field. |
| Queue and reconnect | `handle_route` can defer requests during replay; `dispatch_route` does not check receiver closure or operation expiry. A finite route-channel capacity alone does not bound pending requests and deferred replay routes. |
| Query cancellation | The server offers connection-scoped `query_stream`/`query_cancel`; cancellation acknowledgment is not completed rollback/drain. The current Rust WS engine uses the buffered stream fallback instead. |
| Effect settlement | Caller timeout or disconnect does not prove that a write failed to commit. Explicit RPC transaction controls and SQL multi-statement transactions have different timeout ownership. |
| Result memory | Ordinary WS queries encode a complete reply. Client receive limits do not bound the server's complete response materialization; finite output and in-flight budgets remain necessary. |
| Fixture provisioning | WS SDK backup/import/export support is absent from the checked engine. Supported provisioning control-plane operations must be distinguished from the selected application RPC transport. No checked database-cloning guarantee was found. |

Decisive primary sources include
[WS routing](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/engine/remote/ws/mod.rs),
[request envelope](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/rpc/src/request.rs),
[engine stream fallback](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/engine-api/src/lib.rs),
[transaction-control methods](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/rpc/src/method.rs),
[socket lifecycle](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/rpc/websocket.rs),
[response encoding](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/rpc/response.rs),
and the [official RPC contract](https://surrealdb.com/docs/reference/rest-api/rpc-protocol).
These establish interfaces and source behavior, not a qualified new transport composition.

[Plan 29's AE-25/AE-26](../../plans/29-agent-workspace-effectiveness.md#latency-and-concurrency-s1-s5-s7)
records earlier shared Python state, concurrent schema conflicts and shared-server
slowdown under a different source/resource profile, with a counterexample of study
tests running together at serial speed. The current initializer's definite-conflict
retry differs from that older source. These observations motivate comparing service
topologies and initialization lifetimes; they do not prove a current universal server
bottleneck or a WebSocket speedup.

[28e's Outcome](../../plans/28e-rebuild-retirement-and-qualification.md#outcome-recorded-after-implementation)
retains the managed sweep's HTTP/2 failure after twenty-seven settled points.
The earlier actual-SDK read-only control completed 4,096 queries without reproducing
that failure; it exercises neither the same scientific workload nor uncertain writes.
The failure cause, a WS remedy's runtime behavior and the managed scientific pass remain
unestablished by this review. Preparation, numerical execution, service startup and
transport costs are not interchangeable explanations.
