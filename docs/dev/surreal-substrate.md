# Local SurrealDB substrate

[Plan 30](../plans/30-websocket-and-persistent-agent-environment.md) owns
the transport and agent-environment delivery scope. ADR-0164 owns the substrate decision. This page describes the local server
operator surface implemented by `scripts/surreal_server.py`; it does not qualify
scientific operations, remote deployment or physical power-loss survival.

## Setup and state ownership

Use `just surreal <command>` and the focused test recipes in `just --list`.
The underlying tool commands are:

```bash
.venv/bin/python scripts/surreal_server.py setup --interpretation pse.substrate.v2
.venv/bin/python scripts/surreal_server.py start
.venv/bin/python scripts/surreal_server.py status
```

`--state PATH` selects an isolated application or fixture directory. The default is
`$PSE_SURREAL_STATE`, otherwise `$XDG_STATE_HOME/pse-arrow/surreal-functional-v2`, falling back to
`~/.local/state/pse-arrow/surreal-functional-v2`. Setup refuses nonempty unowned state and paths
through symlinks. Repeated setup of an owned compatible state preserves its chosen
release, credentials and allocation; it is not a reconfiguration or upgrade command.
No legacy import, cleanup or deletion of other application state occurs.

Fresh setup selects the pinned released SurrealDB 3.3.0 binary; an explicit release
selection remains an administrative choice. It verifies the release asset SHA256,
extracts only the executable, checks the reported version and records archive and
binary digests. Existing state retains its exact binary. The private tool root is
`$XDG_DATA_HOME/pse-arrow/tools/surreal`, with the usual home fallback. No application
engine is embedded in Cargo and setup does not modify `PATH`.

The owned state contains:

| Path | Meaning |
|---|---|
| `config.json` | Endpoint, namespace/database, interpretation, admission state, server receipt and finite resource allocation |
| `credentials.json` | Owner and read-only selection credentials; mode 0600 |
| `database/` | RocksDB files, including recovery logs and committed state |
| `tmp/` | Server temporary-file location; not included in an offline backup |
| `server.log`, `.log.1`, `.log.2` | Private supervisor-captured server logs, at most 8 MiB each |
| `server-process.json` | Current child PID/start identity, instance and host allocation; readiness verifies its listener and cgroup ownership |
| `service-launch.json` | Exact admitted service selection, original startup clock, allocation and bound unit invocation/cgroup identity |
| `.supervisor.lock` | Short metadata decisions; never spans workload or drain IPC |
| `.generations/` | Immutable supervisor/source closures and optional worker/producer bytes |
| `.contexts/`, `.receivers/` | Isolated database association and independently admitted receiver configuration |
| `.recovery-controls/` | Private recovery-context identity and outcome evidence, retained when qualification is incomplete |

State directories have mode 0700. Credentials are passed to the child through its
environment, never command arguments, status output or the systemd unit definition.
The supervisor redacts the password from captured server output. Privileged same-user
processes can still read state/process environments; this is a local application
profile, not an isolation boundary against the account that owns it.

## Endpoint and allocation

Fresh functional setup binds `127.0.0.1:18240` by default (`setup --port PORT`).
Application RPC uses authenticated native binary WebSocket at `ws://127.0.0.1:PORT`,
with a 4 MiB complete encoded-message bound. There is one SDK application session
plus the server's implicit connection session. Guest queries, outbound networking
and scripting are denied. Query and transaction backstops are finite; canonical
operations carry their original clocks through admission, dispatch and settlement.
A local timeout does not establish server abort or absence of committed effects.

Opening a Rust/Python context does not install schema. A private root VIEWER account
selects the exact namespace/database without implicit DDL, probes one interpretation
record, then restores the owner account for explicit writes. Ordinary selection does
not enumerate root or namespace catalogs. `canonical-init` is the explicit schema
initialization route. Administrative startup defines the read-only account and checks
authenticated WebSocket readiness; process/listener readiness and interpretation/
receiver admission remain separate.

Pre-database catalog creation cannot use the ordinary in-database UTC fence in
SurrealDB 3.3. It uses a fresh unselected administrative connection under the original
transport clock and finite server backstops. A lost submitted DDL response remains
an unknown outcome; only acknowledged creation admits subsequent selection.

The persistent store is `rocksdb://<state>/database?sync=every&versioned=false`.
Functional services use owned user-manager units and admitted immutable supervisor
closures. They can survive agent exit; continuous logout/reboot availability depends
on the account's verified user-manager/linger setup. Scientific work never starts at
boot. Recovery initially requires explicit action. Restart-on-failure is enabled only
by a positive process-recovery qualification of that service generation, with three
starts per five minutes and a five-second delay. Intentional parking suppresses automatic
restart; an explicit `start` or `ensure` is an admitted unpark operation.

Storage has its own admitted placement. Outside exclusive mode, functional and timing
stores each use their declared store allocation, independently of the caller's heavy
slot. The persistent unit sits under `pse.slice` with its own finite memory cap and CPU
quota. An exclusive owner instead charges borrowed storage inside its aggregate
allocation. Storage affinity follows that storage owner; starting a server does not
materialize a receiver's scientific roles. Primary, observer and worker admission
establish their own placement separately.

Listener readiness checks the storage allocation's actual unit invocation, cgroup
identity, PID start identity, direct/effective memory and CPU limits, and thread
affinity. A receiver's placement receipt or a responding unrelated listener cannot
establish storage ownership. Capacity remains charged across a pending qualified
restart. The new supervisor can rebind that owner only after the exact predecessor
and its descendants have drained; absent, stale or conflicting ownership is refused.
Explicit stopped service readmission/start is the repair route, preserving disk and
earlier generation evidence.

Run `qualify-recovery --state PATH` only on the explicitly selected owned service
when all its contexts can be quiesced and drained. Under one original 90-second clock,
the control explicitly provisions an administrative `pse_recovery` namespace and a
unique `recovery_<operation>` database, checking every provisioning acknowledgment
and the exact selected context. It does not require the base `pse/canonical` database
or install canonical schema. It writes and reads an acknowledged operation in that
context, abandons another fully submitted response, kills the process, reopens the
same disk generation, and reconciles the original identities in the same context.
Failure retains any created probe database and private context evidence with restart
qualification disabled. Local cleanup has its own bounded drain clock; an operation
timeout does not prove that submitted effects are absent. The control finishes stopped
and quiesced; use explicit `start` to reopen serving after success.
A positive receipt enables the bounded automatic restart policy for that generation.
Service readmission or restore disables automatic restart until the new generation
qualifies. The receipt does not qualify native scientific interruption or
filesystem/device power loss.

The host profile declaration owns resource partitions. Compatible worker rebuilds
publish new immutable receiver generations per isolated context while the storage
service continues. Existing receivers keep their exact admitted worker, producer
receipt, source closure and configuration. Runtime readiness checks actual bytes,
process generation, database and role placement, rather than mutable build paths.
Reference mode preserves the original primary/observer allocation; functional and
timing profiles are separate declarations.

Timing uses a dedicated state and endpoint. With an already built linked worker:

```bash
timing_state="${XDG_STATE_HOME:-$HOME/.local/state}/pse-arrow/surreal-timing-v2"
scripts/pse-env --resource-class timing -- just surreal setup --state "$timing_state" \
  --port 18242 --interpretation pse.substrate.v2 --execution-profile timing \
  --worker-executable "$PWD/target/debug/pse-worker"
scripts/pse-env --resource-class timing -- just surreal start --state "$timing_state"
```

`setup --execution-profile timing` records the timing service class and allocation;
it does not select the ordinary functional store. Timing storage starts on admitted
demand rather than being enabled at user-manager startup. Use this state and the
timing execution profile for registered timing contexts. Timing contexts on a
functional service, and functional/reference contexts on a timing service, are refused.
Changing the command's resource class alone does not convert an existing service.
Changing an existing selected service profile requires stopped, drained `reconfigure`.

For an explicitly selected older owned profile, quiesce and drain all contexts, stop,
then use `upgrade` for the supported profile revision or `readmit` for the stopped
service closure/selection credentials. Those commands preserve earlier profile and
credential evidence. Unknown or incompatible profiles are refused. `ensure`, `drain`
and `recover` operate only on the selected ownership; status is read-only and redacts
secrets. A wider allocation must be explicitly admitted before launching its roles.

RocksDB's block cache is one quarter of the allocated server memory. Each write buffer
is `min(32 MiB, allocation / 64)`, with two buffers per column family. The tracked-memory
threshold is three quarters of the server allocation. Compaction read-ahead is 4 MiB,
background thread count is two, background job count is four, and subcompactions are two.
`SURREAL_RUNTIME_WORKER_THREADS=4` bounds the server's custom Tokio runtime; setting only
`TOKIO_WORKER_THREADS` would leave its host-CPU default. These overrides prevent the
initial host-RAM/CPU defaults from defining this application allocation. The threshold
is not a total-RSS cap; systemd supplies that cap.

Supervisor logs have fixed rotation and private ownership. RocksDB additionally owns
its internal `database/LOG`: this release exposes retained-file count (set to two)
and log level (set to `error`), but no current-file byte limit. Temporary files and
database growth likewise need operator disk provisioning/monitoring; the memory cap
does not impose a disk quota. No universal disk or spill qualification is claimed.

## Managed native workers

Launch native work through the configured finite slot allocation:

```bash
.venv/bin/python scripts/surreal_server.py worker --state /path/to/state \
  --worker-command /path/to/pse-worker --until-idle
```

Arguments after `--worker-command` belong to the child. Each slot uses a fixed systemd
user scope name, the profile's `native_worker_memory_bytes` hard cap, zero swap and
128 tasks. Exhausted slots and unavailable systemd supervision fail before execution.
The helper exports `PSE_SURREAL_STATE`, `PSE_NATIVE_WORKER_SLOT` and
`PSE_NATIVE_WORKER_MEMORY_BYTES`; the worker's admitted engine budget must fit this cap.
The lifecycle lock covers admission and scope registration, then releases while the
worker runs. Quiesce can therefore close new claims while existing work drains.
A launcher dying does not free its slot while the named scope or its cgroup remains
populated. A leftover active scope is collected only after its kernel cgroup is
proven empty, then the unit is checked again before reuse or offline backup. Worker
terminal output remains attached to its caller.

## Quiesce, stop and backup

Close application write admission before draining its existing native work:

```bash
.venv/bin/python scripts/surreal_server.py quiesce
# Runtime composition drains managed native workers and in-flight mutations.
.venv/bin/python scripts/surreal_server.py stop --drained
```

`quiesce` records `accepting_writes=false` without interrupting the server. `--drained`
is the caller's acknowledgement that runtime work has drained. The supervisor also
requires every registered receiver, observer, worker scope and pending launch to be
drained before
stopping the server or copying the database. The acknowledgment does not itself
perform or prove that drain. Server stop sends SIGTERM through the service and waits
for the service to become inactive and its cgroup to be empty. The configured deadline can end shutdown forcibly;
RocksDB recovery then occurs at next open. Restart explicitly reopens admission once
the server is ready, except for an unvalidated restore.

For the initial supported backup route, quiesce and drain, then:

```bash
.venv/bin/python scripts/surreal_server.py backup --drained --destination /path/to/new-backup
```

Backup leaves the server stopped and admission closed. It copies the entire offline
database together with configuration, credentials, interpretation, admitted immutable
generation closures and isolated context descriptors, then writes a
SHA256 inventory in `backup.json`. Keep this private backup as carefully as live
credentials. A destination must be new or empty, outside the live state tree. No online
file copy, export/import reconstruction or independent WAL/data-file copy is supported.
An interrupted incomplete copy lacks a valid complete manifest and is not a ready backup.

## Restore and validation

Restore into a new or empty owned target, never over an existing application:

```bash
.venv/bin/python scripts/surreal_server.py restore --state /path/to/restored-state \
  --source /path/to/new-backup --interpretation pse.substrate.v2
.venv/bin/python scripts/surreal_server.py validate --state /path/to/restored-state \
  --interpretation pse.substrate.v2 --check-command <typed-semantic-validator> <arguments>
.venv/bin/python scripts/surreal_server.py start --state /path/to/restored-state
```

Restore checks the full file inventory, digests, known profile and matching interpretation
before creating the target. Managed generation/context paths are rerooted into the
new state; restored serving does not depend on the original state directory. Live
process and admission receipts are not restored. It preserves database bytes and operation identities, assigns
a new supervisor instance identity, and records `validation_required` with
`accepting_writes=false`. Ordinary start refuses that state.

`validate` starts the gated server, authenticates against `pse/canonical`, and requires
`canonical_interpretations:current.interpretation` to match `pse.substrate.v2`. It then
runs the supplied semantic validator with `PSE_SURREAL_STATE` selecting the restore.
The validator owns exact selected values, operation identities and schema/codec checks;
it must use the read path permitted during validation, not normal write admission.
Its output is suppressed to avoid accidental credential disclosure. Failure/timeout
keeps admission closed. Success stops the validation server and leaves the state
quiesced, ready for explicit normal start. Direct privileged access can bypass this
application admission contract and is not a supported normal writer.

## Targeted fixture controls

```bash
.venv/bin/python -m unittest scripts.tests.test_surreal_server -v
.venv/bin/python -m scripts.tests.surreal_fixture_check
```

The unit controls cover private credentials, preservation of unrelated state, finite
joint allocation, inherited authentication-bypass refusal, bounded log rotation, corrupted
backup refusal, interpretation mismatch and the restore admission gate.
Resource-reconfiguration controls additionally cover unchanged rejection, saved
allocation and identity preservation, lifecycle locking and budget use on explicit
restart with mocked process supervision. The disposable
released-server journey uses its own new state directory and a finite 1 GiB server cap.
It authenticates an acknowledged fixed operation record, sends SIGKILL to the service
group, reopens and compares the record, then performs offline backup/restore and gated
validation of that exact record. `kill --state PATH` also supports a root-owned crash
fixture; it deliberately bypasses graceful drain.

This helper journey's data control uses authenticated HTTP on the same server port.
It does not replace Rust native WebSocket exact-codec, operation-identity or transaction controls.
An acknowledged process-restart survival result also does not establish survival of
physical device/filesystem power loss.

The profile follows the official [start command](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/start),
[environment variables](https://surrealdb.com/docs/reference/cli/surrealdb-cli/environment-variables)
and [WebSocket connection contract](https://surrealdb.com/docs/reference/rust/methods/connect).
Source and actual released-binary startup take precedence over unsupported documentation
values: 3.3.0 rejected RocksDB storage log level `none`; this profile uses `error`.

## Disposable tests and retained evidence

Runners register opaque invocation/test resources before side effects. Rust fixture
Drop and Python fixture exit publish borrower/native drain; the existing selected
runner terminal result determines disposition. Only a reconciled pass with completed
drain and no references or manual pin is automatically disposable. A failing or
incomplete result remains pinned. A bare command without a runner association cannot
infer success and keeps its resources. Unknown historical materials are preserved.

Fixture registration and database/control cleanup claims enter the service's short
context-admission gate and recheck current ownership before publication. Removal runs
outside metadata locks. Storage lifecycle reservations refuse both live undrained
fixtures and live cleaners, including cleaners of already drained fixtures. Evidence
file cleanup remains independent of storage lifecycle.

Use `python -m scripts.test_resources list` or `status RESOURCE` to inspect exact
ownership; `pin RESOURCE`, `release RESOURCE` and `reclaim RESOURCE` are explicit
operator routes. Retiring a retained evidence dependency uses `release-reference
RESOURCE --receipt PATH --digest SHA256`; the exact original digest must match, and
pins/drain still govern reclamation. Successful new report compaction preserves
checks, selections, provenance and JSON receipts. Old reports and frozen worktrees
are not cleanup candidates merely because they occupy space.

## Typed scientific inspection

Use an already admitted `pse.Runtime` and `ModelingPackage` for authored and scientific
identities. This is distinct from raw native MCP inspection: constructing a runtime
requires its existing deployment settings and qualified producer, and does not grant
permission to initialize a missing database. The small authored Root fixture in
[the modeling journey](../../python/pse/tests/test_modeling_run.py) and
[the exact result/reopen journey](../../python/pse/tests/test_canonical_results.py)
show supported package and run construction. With their `package`, `case`, `settings`,
`runtime` and completed `result` already available:

```python
import pyarrow as pa

inspection = package.inspect(case, settings)
sources = package.source_tables()  # Explicit complete immutable source export.
try:
    source_rows = {identity: pa.table(stream).to_pylist()
                   for identity, stream in sources.items()}
finally:
    for stream in sources.values():
        stream.close()

run = result.canonical_run_key
attempt = result.canonical_attempt_key
assert run is not None and attempt is not None
records = []
for read, key in ((runtime.run_record, run),
                  (runtime.attempt_record, attempt),
                  (runtime.result_manifest, attempt)):
    stream = read(key)
    try:
        records.append(pa.table(stream).to_pylist())
    finally:
        stream.close()
usage = runtime.resource_usage()  # Typed deployment resource report.
```

Keep run and attempt distinct and retain the exact keys with the observed records.
Close streams on success or error; close the runtime when its owning workflow finishes.
Source export intentionally materializes one package's inventory, not the whole database.
These reads do not solve a model or establish scientific correctness. The
[agent environment guide](agent-environment.md#runtime-capabilities) owns native MCP
session opt-in and raw-context selection.
