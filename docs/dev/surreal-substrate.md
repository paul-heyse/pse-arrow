# Local SurrealDB substrate

[Plan 28a](../plans/28a-canonical-substrate-and-revisions.md) owns the current delivery
scope. ADR-0164 owns the substrate decision. This page describes the local server
operator surface implemented by `scripts/surreal_server.py`; it does not qualify
scientific operations, remote deployment or physical power-loss survival.

## Setup and state ownership

Use `just surreal <command>` and the focused test recipes in `just --list`.
The underlying tool commands are:

```bash
.venv/bin/python scripts/surreal_server.py setup --interpretation pse.substrate.v1
.venv/bin/python scripts/surreal_server.py start
.venv/bin/python scripts/surreal_server.py status
```

`--state PATH` selects an isolated application or fixture directory. The default is
`$PSE_SURREAL_STATE`, otherwise `$XDG_STATE_HOME/pse-arrow/surreal`, falling back to
`~/.local/state/pse-arrow/surreal`. Setup refuses nonempty unowned state and paths
through symlinks. Repeated setup of an owned compatible state preserves its chosen
release, credentials and allocation; it is not a reconfiguration or upgrade command.
No legacy import, cleanup or deletion of other application state occurs.

Setup selects the current stable official GitHub release for Linux amd64/arm64.
`--version vX.Y.Z` can reproduce a recorded released binary. It verifies the archive
against the official release asset's SHA256 digest, extracts only the server executable,
checks its reported version and records archive/binary digests in a private receipt.
The tool root is `$XDG_DATA_HOME/pse-arrow/tools/surreal`, falling back to
`~/.local/share/pse-arrow/tools/surreal`; `--tool-root PATH` overrides it. Installation
adds no database engine to the application Cargo dependency graph and does not modify
`PATH`. The initial observed release was 3.3.0; future fresh setup follows the release
resolver. Existing state records the actual selected version, rather than silently
switching its database executable on restart.

The owned state contains:

| Path | Meaning |
|---|---|
| `config.json` | Endpoint, namespace/database, interpretation, admission state, server receipt and finite resource allocation |
| `credentials.json` | Root `username` and generated `password`; mode 0600 |
| `database/` | RocksDB files, including recovery logs and committed state |
| `tmp/` | Server temporary-file location; not included in an offline backup |
| `server.log`, `.log.1`, `.log.2` | Private supervisor-captured server logs, at most 8 MiB each |
| `server-process.json` | Current child PID and instance identity; readiness verifies its listener and cgroup ownership |
| `.supervisor.lock` | Serializes operator lifecycle commands |

State directories have mode 0700. Credentials are passed to the child through its
environment, never command arguments, status output or the systemd unit definition.
The supervisor redacts the password from captured server output. Privileged same-user
processes can still read state/process environments; this is a local application
profile, not an isolation boundary against the account that owns it.

## Endpoint and allocation

The server binds `127.0.0.1:18080` by default (`setup --port PORT`). Authentication is
enabled. gRPC uses that same port, with `endpoint` in `config.json` set to
`grpc://127.0.0.1:PORT`. The database is `pse/canonical` and the interpretation is
`pse.substrate.v1`. An initial server does not create application schema; the typed
substrate boundary owns initial declaration installation and the interpretation record.

`config.json` supplies `endpoint`, `namespace`, `database`, `schema_interpretation`,
`max_message_bytes`, `credentials_file` and `accepting_writes` to the Rust client.
The client must honor admission closure for existing connections as well as new ones.
Status prints paths and public configuration, never credential contents. Readiness checks
that the listener belongs to the recorded child in the owned systemd cgroup, then checks
health. It does not establish authentication, schema or operation correctness.

The persistent store is:

```text
rocksdb://<state>/database?sync=every&versioned=false
```

The server explicitly sets `SURREAL_GRPC_MAX_MESSAGE_SIZE=4194304`; the SDK must set
the same initial 4 MiB client limit. Batch/result sizing includes encoded metadata and
must stay below that limit. The server denies guest queries, outbound networking and
embedded scripting, and applies 90-second query and transaction maxima. Ordinary
client queries have a 20-second server deadline and a 30-second transport deadline.
Atomic revision activation uses a separate client with a 60-second query deadline
and a 90-second transport deadline. A missing statement completion is an error;
uncertain activation settles only through its immutable operation receipt.

The default configured application memory envelope is 4096 MiB: server 2048 MiB and
two managed native-worker slots of 1024 MiB each. Setup supports
`--memory-mib`, `--server-memory-mib`, `--native-workers` and
`--native-worker-memory-mib`; it rejects over-allocation, fewer than one/more than 32
workers and a server allocation below 512 MiB. Runtime composition consumes the finite
worker count and per-worker allocation. The helper launches native processes within those
fixed capped slots and refuses excess admission.

Change an existing owned allocation while admission is quiesced and the server is
stopped:

```bash
just surreal quiesce
# Drain runtime work before acknowledging it.
just surreal stop --drained
just surreal reconfigure --memory-mib 32768 --server-memory-mib 16384 \
  --native-worker-memory-mib 8192
just surreal start
```

`reconfigure` accepts the three memory options above. Omitted values preserve the
saved allocation exactly; setup's defaults apply only to new state. The supervisor
verifies the stopped server's cgroup and every existing worker group is empty under
the lifecycle lock, validates the complete joint budget, then atomically replaces
only the allocation and its derived cache, write-buffer and threshold fields.
Invalid or partially specified budgets that exceed the saved joint envelope,
active or unverifiable process groups, and admission that is not fully quiesced
leave the configuration unchanged. Explicit `--native-workers`, `--port`,
`--version`, `--interpretation` or `--tool-root` options are rejected; this command
preserves worker count and unit names, endpoint, interpretation, release,
credentials and database contents. It leaves the server stopped and admission
closed. The next explicit `start` consumes the new server budget and derived
settings; subsequent managed workers consume the new per-worker budget.

The supervisor creates a transient systemd **user service** with `MemoryMax` equal to
the server allocation, `MemorySwapMax=0`, `TasksMax=128`, `KillMode=control-group` and
a 45-second stop deadline. The cap includes the wrapper and server child. A user manager
is required; unavailable supervision fails rather than starting uncapped. It discovers
the account's standard runtime bus when shell bus variables are absent. This uses the
same systemd resource-control surface as `scripts/memory-cap.sh`, with an explicit
service lifecycle. No automatic restart is configured: failed/abruptly killed units
must be restarted explicitly, and application clients own reconnect/uncertain-operation
resolution.

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
requires every configured worker scope to be inactive and its cgroup unpopulated before
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
database together with configuration, credentials and interpretation, then writes a
SHA256 inventory in `backup.json`. Keep this private backup as carefully as live
credentials. A destination must be new or empty, outside the live state tree. No online
file copy, export/import reconstruction or independent WAL/data-file copy is supported.
An interrupted incomplete copy lacks a valid complete manifest and is not a ready backup.

## Restore and validation

Restore into a new or empty owned target, never over an existing application:

```bash
.venv/bin/python scripts/surreal_server.py restore --state /path/to/restored-state \
  --source /path/to/new-backup --interpretation pse.substrate.v1
.venv/bin/python scripts/surreal_server.py validate --state /path/to/restored-state \
  --interpretation pse.substrate.v1 --check-command <typed-semantic-validator> <arguments>
.venv/bin/python scripts/surreal_server.py start --state /path/to/restored-state
```

Restore checks the full file inventory, digests, known profile and matching interpretation
before creating the target. It preserves database bytes and operation identities, assigns
a new supervisor instance identity, and records `validation_required` with
`accepting_writes=false`. Ordinary start refuses that state.

`validate` starts the gated server, authenticates against `pse/canonical`, and requires
`canonical_interpretations:current.interpretation` to match `pse.substrate.v1`. It then
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
It does not replace Rust gRPC exact-codec, operation-identity or transaction controls.
An acknowledged process-restart survival result also does not establish survival of
physical device/filesystem power loss.

The profile follows the official [start command](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/start),
[environment variables](https://surrealdb.com/docs/reference/cli/surrealdb-cli/environment-variables)
and [gRPC connection contract](https://surrealdb.com/docs/reference/rust/methods/connect).
Source and actual released-binary startup take precedence over unsupported documentation
values: 3.3.0 rejected RocksDB storage log level `none`; this profile uses `error`.
