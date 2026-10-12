# Local SurrealDB substrate

[Plan 30](../plans/30-websocket-and-persistent-agent-environment.md) owns
the transport and agent-environment delivery scope. ADR-0164 owns the substrate decision. This page describes the local server
operator surface implemented by `scripts/surreal_server.py`; it does not qualify
scientific operations, remote deployment or physical power-loss survival.

## Setup and state ownership

Use `just surreal <command>` and the focused test recipes in `just --list`.
The underlying tool commands are:

```bash
.venv/bin/python scripts/surreal_server.py setup --interpretation pse.substrate.v4
.venv/bin/python scripts/surreal_server.py start
.venv/bin/python scripts/surreal_server.py status
```

`--state PATH` selects an isolated application or fixture directory. The default is
`$PSE_SURREAL_STATE`, otherwise `$XDG_STATE_HOME/pse-arrow/surreal-functional-v2`, falling back to
`~/.local/state/pse-arrow/surreal-functional-v2`. Setup refuses nonempty unowned state and paths
through symlinks. Repeated setup of an owned compatible state preserves its chosen
release, credentials and allocation; it is not a reconfiguration or upgrade command.
No legacy import, cleanup or deletion of other application state occurs.

The current generated interpretation is `pse.substrate.v4`, including typed scalar
result-row locators. Plan 36 exercises it on newly owned fixtures. These commands
do not authorize activation, migration or deletion of an existing deployment;
an existing interpretation mismatch still refuses ordinary startup/admission.

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

Storage construction reads only the authenticated storage projection. Runtime/worker
composition selects native allocation and receiver policy together for the connected
database, checks registered receiver generation and actual executable bytes, and refuses
changed configuration before reporting readiness. Invalid unused native policy does not
prevent storage inspection. An explicit execution request still requires valid policy.

Immutable protected reads share a local per-problem turn. Protection creation/renewal,
publication, staging and other mutations take an exclusive, fair writer turn. The turn
covers a complete short RPC/retry decision, never native work or hydration; its queue
consumes the original clock. Server guards and generations remain distributed authority.

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

Startup distinguishes a ready owned service, a compatible startup already in progress,
observed absence, unavailable observation and mismatch. `start`/`ensure` join an existing
compatible startup under the original caller clock, and a healthy fast path takes no
mutation reservation. Unavailable observation is a refusal or bounded re-observation;
it is not permission to restart a borrowed service. Actual lifecycle mutations still
require the exact reservation and all-context drain. Borrowed-service cleanup checks the
current launch allocation and bound incarnation inside that reservation before mutating
the service; an old cleanup snapshot cannot stop a replacement owner.
After proving the selected service and its recorded process incarnation drained,
exclusive cleanup returns only that borrowed service affiliation under the original
stop clock. It preserves the enclosing caller and its other registered work. A concurrent
registration prevents allocation retirement until that work also drains.

Host allocation admission reclaims ended owners without starting unrelated queued parked
services. An explicit `--store` demand resumes only the selected service under its admitted
owner. A functional store can reacquire its declared independent storage allocation; the
original reference store still requires a matching exclusive/reference allocation. Explicit
reconciliation retains failed resume intent for a later compatible owner, and an operator
stop withdraws that service's queued resume.

Run `qualify-recovery --state PATH` only on the explicitly selected owned service
when all its contexts can be quiesced and drained. Under one original 90-second clock,
the control explicitly provisions an administrative `pse_recovery` namespace and a
unique `recovery_<operation>` database, checking every provisioning acknowledgment
and the exact selected context. It does not require the base `pse/canonical` database
or install canonical schema. It writes and reads an acknowledged operation in that
context, abandons another fully submitted response, kills the process, reopens the
same disk generation, and reconciles the original identities in the same context.
Successful reconciliation removes only this operation's probe database and checks its
absence in the namespace catalog before publishing positive qualification. The saved
context records both settlement and retirement. Historical or uncertain probes are not
removed by a later qualification.
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
Context publication creates each receiver-directory level privately and refuses an
existing parent with unsafe permissions, ownership or a symlink. Native selection
checks those same private premises before execution. Permission validation is not
an implicit repair or adoption route for an older profile.
Reference mode preserves the original primary/observer allocation; functional and
timing profiles are separate declarations.

Timing uses a dedicated state and endpoint. With an already built linked worker:

```bash
timing_state="${XDG_STATE_HOME:-$HOME/.local/state}/pse-arrow/surreal-timing-v2"
scripts/pse-env --resource-class timing -- just surreal setup --state "$timing_state" \
  --port 18242 --interpretation pse.substrate.v4 --execution-profile timing \
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

Readiness reports a phase and cause with the selected and observed owner identities;
startup, stopping, maintenance, mismatch and unavailable manager observations are distinct.
Metadata exclusion and control waits consume an existing enclosing clock, including an
expiry check after acquisition. Cancellation retains separate cleanup ownership. Native
drain reports terminal unit ownership and empty descendants; a missing leader or an
unavailable manager alone does not establish drain.

Worker and primary launch reservations retain their exact child handoff until the matching
pending nonce is settled. The child records its actual scope inode before running the payload;
retiring the host allocation ledger does not erase that proof. Failed registration joins and
cleans up the owned launcher under its cleanup clock. If ownership or drain remains unknown,
the reservation stays closed. A later `drain`, `stop` or stopped-profile `readmit` retries
reconciliation against that same launch; missing allocation metadata alone cannot release it.

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
for the service to become inactive, its cgroup to be empty, and the recorded PID/start
lifetime to disappear. An unreaped zombie still retains its allocation; host reconciliation
uses the same exact affiliated process premise before reclaiming capacity.
The configured deadline can end shutdown forcibly;
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
file copy or independent WAL/data-file copy is supported. The separately owned restore
maintenance phase uses native export/import to establish its fresh database identity;
that phase is not an alternate backup format.
An interrupted incomplete copy lacks a valid complete manifest and is not a ready backup.

## Restore and validation

Restore into a new or empty owned target, never over an existing application:

```bash
.venv/bin/python scripts/surreal_server.py restore --state /path/to/restored-state \
  --source /path/to/new-backup --interpretation pse.substrate.v4
.venv/bin/python scripts/surreal_server.py validate --state /path/to/restored-state \
  --interpretation pse.substrate.v4 --check-command <typed-semantic-validator> <arguments>
.venv/bin/python scripts/surreal_server.py start --state /path/to/restored-state
```

Restore checks the full file inventory, digests, known profile and matching interpretation
before creating the target. Receiver equivalence checks immutable bytes, permissions,
the exact external interpreter and the permitted source-to-restored installation mapping;
directory permissions may narrow while retaining owner access, but may not widen; exact file
modes remain checked. Intermediate-directory and manifest symlinks refuse. Managed generation/context paths are rerooted into the
new state; restored serving does not depend on the original state directory. Live
process and admission receipts are not restored. Restore reserves the destination and publishes
a fresh closed namespace/database before copying backup bytes. It removes copied analysis
products, rotates source-authority incarnations while retaining their closure floors, copies
the current database into the fresh identity, rotates root credentials and removes the exact
old copied database. Unrelated catalog owners refuse maintenance. Non-analysis operation
identities remain current; old authenticated clients cannot write the copied old database.
The target records `validation_required` with `accepting_writes=false`; ordinary start refuses it.
Credential rotation may materialize this fresh installation's systemd unit under maintenance
ownership. That unit file establishes neither readiness nor restart qualification. Scientific
recovery compares complete schema metadata and exact float/binary bits while disregarding
metadata map order and transport chunk boundaries.

`validate` starts the gated server, authenticates against `pse/canonical`, and requires
`canonical_interpretations:current.interpretation` to match `pse.substrate.v4`. It then
runs the supplied semantic validator with `PSE_SURREAL_STATE` selecting the restore.
The validator owns exact selected values, operation identities and schema/codec checks;
it must use the read path permitted during validation, not normal write admission.
Its output is suppressed to avoid accidental credential disclosure. Failure/timeout
keeps admission closed. Success stops the validation server and leaves the state
quiesced, ready for explicit normal start. Direct privileged access can bypass this
application admission contract and is not a supported normal writer.

Maintenance uses a temporary loopback server with ordinary admission closed, declared
RocksDB resources and no autonomous restart or scientific workers. Native CLI SQL/export/import
connect remotely because embedded CLI operations do not carry the server's RocksDB configuration.
Its live lifecycle owner must stop and drain the exact invocation before filesystem copies,
initializer handoff or normal start. Pending ownership or uncertain shutdown stays closed.
Credential recovery probes recorded ROOT candidates with fresh unselected connections; it never
creates an account or missing database to make recovery succeed.

## Explicit design-phase reconstruction and recovery

For an isolated, stopped and authentically drained profile, `rebuild --state PATH
--initializer-command COMMAND ...` regenerates the selected design-phase state from
current authoritative sources and fixtures in a fresh namespace/database. No old data
or exported authored-input inventory is required. External source files are untouched.
The initializer receives the live `PSE_CANONICAL_INITIALIZER` lifecycle nonce. Ordinary
canonical handles remain closed; initializer authority expires with its exact owner.
Only after current initialization and interpretation validation is the exact replaced
database removed. Fresh database/store identities and credentials fence old capabilities.

Generated endpoint enforcement changes the schema digest even when the interpretation
version stays unchanged. Ordinary opening refuses an old digest. Adopt the current schema
through explicit regeneration or supported current-version restore and semantic validation;
neither supervisor readmission nor successful server startup upgrades canonical schema.
Memberships, analysis edges/inputs, problem runs and run sources enforce existing endpoints.
Study dependency ingestion permits valid references to predecessors in later pages. Native
import bypasses endpoint admission, so post-restore integrity checks remain required.

Interrupted rebuild or restore remains closed. `recover-maintenance --state PATH` resumes
its durable phase after verifying exact identities; an interrupted rebuild initializer also
requires `--initializer-command COMMAND ...`. Recovery drains the preceding owner and
regenerates an unacknowledged initializer target with fresh authority. Unknown phases,
borrowers or changed current-version backup inventories refuse recovery. Current-version
restore retains its backup and exact export/copy ownership contract; it is independent of
fresh design-phase regeneration.

## Targeted fixture controls

```bash
.venv/bin/python -m unittest scripts.tests.test_surreal_server -v
.venv/bin/python -m scripts.tests.surreal_fixture_check
```

The design-phase rebuild/current-restore control uses the actual canonical initializer
and native SQL/export/import on fresh profiles. Build `xtask` with `canonical-tools`,
then run it inside an admitted exclusive allocation, after other backend journeys drain:

```bash
cargo build -p xtask --no-default-features --features canonical-tools --locked
.venv/bin/python scripts/tests/efficiency_maintenance_actual_check.py \
  --directory /absolute/new/private-maintenance-control --port 18243
```

The lifecycle fixture also requires that `canonical-tools` initializer. It initializes
the selected namespace/database through the production schema path before toy writes;
the native MCP-only control independently exercises its disposable namespace. Successful
owned fixture retirement requires service, process and scope drain plus a read-only host
ledger observation showing no live/restored service references. A remaining reference or
unavailable observation retains the fixture and prevents positive evidence publication.

The directory must be new. This control checks untouched external authored file hashes, current
schema initialization, full analysis removal, fresh database/account authority, closed
validation and interrupted-initializer recovery. Its final report includes actual drain
errors; it does not compile the preserved authored model or establish scientific parity.
Catalog observations use top-level `INFO` statements with exact catalog/nonce acknowledgments;
the pinned server requires namespace selection for catalog expressions assigned to variables.
Failure output occupies one bounded private slot, with secrets redacted before truncation.

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

## Standalone beta tooling fixtures

`just surrealkit` acquires the standalone version declared in
`.config/surrealkit.toml` under the owned tools directory. Its release receipt records
archive/executable checksums, exact source commit, committed lockfile and official
locked-build workflow. This is provenance, not a reproducible-build attestation.
The production SDK/server and its transport patch remain separately selected.

Use a fresh disposable service created with `just surreal setup --state PATH
--tooling-fixture ...` for the beta parity journey. This explicit profile permits the
beta's retained SDK clone; ordinary simulator service settings stay unchanged.
Provision a registered namespace with `just surrealkit provision --state PATH`, then
stage production generated definitions with `just surrealkit stage --resource ID`.
`plan`, `sync`, `replay`, `test-sync` and `test-both` require that resource and its
owned `--folder PATH`. Initial support admits the exact generated schema, built-in
beta metadata setup and the repository's qualified SQL expectations. It refuses
ambient targets, explicit actors, external/file inputs, seeds, changed schema files,
coverage overrides and unowned rollout changes. Expansion requires qualification.

The complete disposable generated-contract control is:

```bash
scripts/pse-env --native= --resource-class light -- python -m scripts.tests.surrealkit_fixture_check \
  --state /absolute/private/tooling-state --tool-root /absolute/private/tools \
  --evidence /absolute/new/private/evidence
```

This compares direct generated installation with fresh sync/frozen replay/parity,
checks namespace credentials and indexed readiness selection, and removes only
terminally passed/drained namespaces. Beta keeps child databases for the external
resource owner. Checked namespace removal uses an unselected root administration
session and verifies absence: the pinned SQL shell otherwise reselects and recreates
a just-removed namespace. Failed/unknown activities remain protected for exact
reconciliation; a child exit does not prove server work settled. Permission evidence
from this control covers root/namespace-owner execution, not record-user policy.

Exclusive rollout execution lives in `scripts/surrealkit_maintenance.py`. It requires
an already closed, drained, privately owned service; it does not stop another caller's
work or activate an interpretation. Its durable pending selection keeps ordinary
start/open/backup/restore/rebuild paths closed until the actual operation owner
settles the exact submitted phases and the native child/listener drain. The immutable
predecessor marker is captured before the first SQL submission. A phased start and
completion share one pending nonce; completion requires an independently settled start.
Settlement retains the selected pending record and phase evidence until explicit canonical
activation. Metadata repair refuses unsettled SQL, including beta's `running_complete`
shortcut. Beta TTL, CLI success and catalog parity alone cannot supply settlement.
The fixed `settle` route loads the immutable publication from `database/canonical-rollout`,
checks beta's exact manifest/step identities and acknowledgements, and invokes
`canonical-validate-definitions` under this reservation's live initializer nonce.
Only producer-owned schema actions are admitted. Unknown schema SQL may reconcile after
authentic child/listener drain and exact AST postconditions; arbitrary data SQL requires
its own domain owner. A `sql_settled` phase permits metadata repair but cannot activate;
settle again after beta reaches the required terminal phase. Final `activate` independently
checks the current generated target, original predecessor and exact operation settlement,
then publishes an atomic activation receipt/marker. Lost activation acknowledgement retries
the same nonce; cleanup failure retains the fence. Ordinary admission remains closed until
explicit normal start. Phase-only definition witnesses cannot activate the product.

The `up`, `start`, `complete`, `status` and `repair` commands use `--binary` for the pinned
beta executable. `settle` and `activate` use it for the compiled `canonical-tools` xtask;
all commands require the exact `--state` and `--folder`, and continuation/recovery requires
`--pending-nonce`. Production publication is append-only through its producer; normal
code generation does not rewrite frozen steps. Actual persistent cutover qualification
remains in Plan37 until the integrated journeys pass.

The bounded actual persistent journey is
`scripts/tests/surrealkit_persistent_fixture_check.py`. Under the coordinator's
exclusive allocation, invoke it through `scripts/pse-env --resource-class exclusive
--native= --` (no solver capabilities) and pass a new `--directory` below this checkout's `build/`, the
verified beta executable as `--binary`, and the current compiled canonical-tools
xtask as `--canonical-binary`. It registers pinned evidence before creating private
profiles. It exercises independent start/completion settlement, exact activation
receipt replay after caller acknowledgement loss, a synthetic compatible forward
index change, fresh regeneration and actual lifecycle caller SIGKILL. The killed
caller's arbitrary SQL remains explicitly unknown; its separate profile and all
fixture evidence remain pinned. This control never edits beta TTLs, fabricates an
SQL acknowledgement, or treats physical drain as SQL settlement. Its synthetic
predecessor does not establish historical migration compatibility.

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
