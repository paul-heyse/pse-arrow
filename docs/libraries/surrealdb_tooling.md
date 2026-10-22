# SurrealDB Tooling Ecosystem — Comprehensive Rust Technical Reference

Research date: October 22, 2026 | SurrealDB 3.3.4 | Rust-focused | Official, companion, and third-party tooling

## 1. Executive orientation

SurrealDB has developed a much broader tooling ecosystem than its core database server and Rust SDK. It now includes dedicated storage engines, schema-management and migration tooling, native MCP interfaces, Rust/WASM extension infrastructure, graph and vector-query diagnostics, distributed tracing, and independently usable database components.

The most important architectural distinction is between:

1. The SurrealDB query and transaction layer, which implements SurrealQL, graph operations, indexing, transactions, permissions, APIs, and query execution.
2. Underlying storage engines, including RocksDB, SurrealKV, SurrealMX, and distributed storage backends. These have different diagnostic, persistence, concurrency, and operational characteristics.
3. Development and operational tooling, including the `surreal` CLI, SurrealKit, Surreal Sync, database introspection, benchmarking, observability, and backup/repair utilities.
4. Integration and extension infrastructure, including the Rust SDK, Surrealism/WASM plugins, MCP, PostgreSQL wire compatibility, gRPC, GraphQL, ISO GQL, and other interfaces.

These boundaries matter because some tools operate through the SurrealDB query layer, while others directly inspect or manipulate a storage engine. They are not interchangeable.

### Version and availability baseline

The current stable SurrealDB release is 3.3.4, released October 13, 2026. Its Rust SDK is also documented at 3.3.4.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+1



Two major storage developments occurred immediately before this research:

- SurrealMX 1.0 was announced October 12, 2026. It is an independently usable, Rust-native, in-memory MVCC engine with optional persistence. The version of SurrealMX incorporated into a given SurrealDB binary must be distinguished from the standalone crate release.
- SurrealKV 1.0 was announced October 13, 2026. It is a Rust-native persistent storage engine intended to replace RocksDB as SurrealDB's persistent storage engine in version 4. That does not mean RocksDB is already removed or that SurrealDB 3.3.4 automatically uses SurrealKV 1.0.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

  SurrealDB

  +2



Both developments materially change the longer-term tooling landscape.

The reference below prioritizes capabilities usable from Rust or through Rust-based binaries. It also includes important language-neutral operational interfaces and selected non-Rust utilities where they provide functionality that the Rust tooling does not directly replace.

## 2. Tooling inventory and capability map

| Tool or component                   | Principal capability                                                 | Implementation / status                       |
| ----------------------------------- | -------------------------------------------------------------------- | --------------------------------------------- |
| `surreal` / `surreal db`            | Server management, SQL shell, import/export, validation, repair      | Official Rust CLI; stable                     |
| `surreal cloud`                     | Managed instance, backup, credentials, organisation administration   | Official CLI component                        |
| `surrealkit`                        | Schema reconciliation, migrations, seeding, testing, type generation | Official Rust CLI/crate; stable 0.7, 1.0 beta |
| `surreal-sync`                      | Bulk migration, CDC, replication into SurrealDB                      | Official Rust tool                            |
| `surrealdb` crate                   | Remote and embedded database interaction                             | Official Rust SDK                             |
| `surrealdb-types` and derive macros | Rust/SurrealDB value mapping                                         | Official Rust crates                          |
| `surrealdb-parser` / AST crates     | SurrealQL parsing and language infrastructure                        | Engine-level Rust components                  |
| SurrealMCP / built-in MCP           | Agent access to data, queries, schema, graph operations              | Official Rust implementation                  |
| Surrealism                          | Custom Rust functions executing inside SurrealDB                     | Rust-to-WASM extension system                 |
| SurrealML                           | Database-integrated model serialization and inference support        | Rust-related subsystem                        |
| `EXPLAIN ANALYZE`                   | Query plans and operator-level performance diagnostics               | Native query engine                           |
| `INFO` / `REBUILD INDEX`            | Schema introspection and index lifecycle diagnostics                 | Native SurrealQL                              |
| OpenTelemetry / Prometheus          | Metrics, tracing, logs, query and storage diagnostics                | Native instrumentation                        |
| `surrealmx`                         | Lock-free transactional in-memory KV engine                          | Independent Rust crate, 1.0                   |
| `surrealkv`                         | Durable, versioned LSM-based KV engine                               | Independent Rust crate, 1.0                   |
| `skv` / `surrealkv-cli`             | SST inspection, integrity checking, migration, storage analysis      | Official Rust CLI                             |
| `rocksdb` / `surrealdb-rocksdb`     | Persistent RocksDB backend and embedded access                       | C++ engine with Rust bindings                 |
| RocksDB utilities                   | SST, WAL, compaction, cache, integrity and storage performance tools | Native C++ tools                              |
| TiKV toolchain                      | Distributed KV deployment and diagnostics                            | External distributed backend                  |
| `crud-bench`                        | Comparative embedded/networked KV and database benchmarks            | Official Rust benchmark suite                 |
| Language tests / Criterion          | SurrealQL correctness and performance regression testing             | Upstream Rust tooling                         |
| Surrealist / SurrealDB Studio       | Visual query, graph, schema and data exploration                     | Official UI; not Rust-native                  |
| PostgreSQL wire / `psql`            | SQL-shell access and standard client connectivity                    | SurrealDB 3.3 protocol support                |
| GraphQL / ISO GQL                   | Schema-based APIs and graph query interfaces                         | Native server interfaces                      |
| `surqlfmt` and editor integrations  | SurrealQL formatting, highlighting and authoring                     | Auxiliary tooling                             |

The official Rust CLI now uses a component launcher, so `surreal db`, `surreal cloud`, and future `surreal mod` releases have separate versioning. Older top-level commands remain as deprecated aliases; importantly, `surreal mod` itself had not yet shipped as a separate component at the research date, while `surreal db module` was available.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+3



## 3. Storage engines and their diagnostic ecosystems

### 3.1 Exact storage-engine version boundaries

The SurrealDB 3.3.4 source manifest establishes an especially important distinction:

| Component       | Dependency in SurrealDB 3.3.4            | Independently released version |
| --------------- | ---------------------------------------- | ------------------------------ |
| SurrealMX       | `surrealmx = "0.27.0"`                   | 1.0.0                          |
| SurrealKV       | `surrealkv = "0.21.4"`                   | 1.0                            |
| RocksDB binding | `surrealdb-rocksdb = "0.24.0-surreal.4"` | Separately versioned           |

These are version requirements from the tagged workspace manifest, not proof of the exact resolved dependency versions in every compiled binary. They establish that a standalone SurrealMX 1.0 or SurrealKV 1.0 release is not equivalent to having that version integrated into SurrealDB 3.3.4.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Tagged manifest: `https://github.com/surrealdb/surrealdb/blob/v3.3.4/Cargo.toml`

### 3.2 SurrealMX 1.0 — in-memory engine and direct Rust instrumentation

SurrealMX implements an in-memory transactional KV store using MVCC over a concurrent adaptive radix tree, with a commit ring and concurrency mechanisms designed to minimize shared contention.

It offers capabilities below the SurrealQL abstraction:

- Transactional access: serializable write transactions, snapshot-isolated read transactions, rollback, and nested savepoints.
- Direct KV operations: `get`, `set`, `put`, `del`, `exists`, and zero-copy value inspection via `with_value`.
- Efficient scans: `scan_with`, `scan_into`, `keys_for_each`, `keys_into`, forward/reverse iteration, and reusable buffers.
- Memory lifecycle: configurable cleanup and garbage collection, including targeted version cleanup and full-database sweeps.
- Optional persistence: append-only logging, group commit, asynchronous persistence, snapshots, configurable fsync, and LZ4 snapshot compression.
- Concurrency verification: deterministic simulation testing against a reference model and Criterion-based benchmarks.

The direct point-lookup and scan APIs can be useful when SurrealMX is used as an independent KV engine. They are not automatically APIs for reading SurrealDB records: SurrealDB stores its own encoded keyspace and database semantics above the engine.

SurrealMX also exposes manual maintenance functions such as `run_cleanup`, `run_gc_tracked`, and `run_gc`. These are valuable for diagnosing version retention and garbage collection when using the engine directly. They should not be assumed to be externally callable through a running SurrealDB server.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



Primary sources:

- `https://github.com/surrealdb/surrealmx`
- `https://docs.rs/surrealmx/`
- `https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine`

### 3.3 SurrealKV 1.0 — persistent storage and the `skv` diagnostic toolchain

SurrealKV is a persistent Rust-native LSM engine with MVCC, write-ahead logging, compaction, and versioned storage options. Its 1.0 release adds particularly consequential capabilities for database administration and diagnostics:

- Opt-in historical value reads using `get_at` and `history`, with configurable retention.
- Optional value-log separation to reduce repeated compaction of large values.
- Encryption at rest with configurable key management.
- Native checkpointing and recovery facilities.
- Storage-file inspection and checksum-based corruption detection.
- Migration readers for previous SurrealKV versions and SurrealDB's RocksDB format.
- Deterministic differential testing and a standalone simulation harness.

Its `skv` utility deserves special attention because it provides lower-level capabilities not supplied by ordinary SurrealQL inspection.

| `skv` command | Purpose                                              |
| ------------- | ---------------------------------------------------- |
| `inspect`     | Inspect database layout and storage breakdown        |
| `manifest`    | Inspect LSM levels and the file manifest             |
| `sst`         | Examine SSTables and verify checksums                |
| `scrub`       | Check stored SSTable blocks for corruption           |
| `get`         | Read an individual KV entry                          |
| `scan`        | Inspect key ranges or prefixes                       |
| `enc-info`    | Inspect encryption metadata                          |
| `migrate`     | Migrate supported old RocksDB/SurrealKV data formats |

Example commands from upstream documentation:

```
cargo install surrealkv-cli

skv inspect ./data
skv manifest ./data
skv scrub ./data
skv scan ./data --prefix "user:"
skv enc-info ./data

skv migrate ./old-rocksdb ./new-surrealkv
```

The associated Rust components are also important:

| Crate / subsystem          | Functionality                                     |
| -------------------------- | ------------------------------------------------- |
| `surrealkv`                | Transactional storage engine                      |
| `surrealkv-cli`            | The `skv` operator tool                           |
| `surrealkv-sim`            | Deterministic simulation and differential testing |
| `surrealkv-compat-rocksdb` | Pure-Rust reader for supported RocksDB databases  |
| `surrealkv-compat-v1`      | Reader for historical SurrealKV formats           |
| `surrealkv-compat-indxdb`  | Conversion support for legacy IndexedDB stores    |

SurrealKV 1.0's RocksDB compatibility layer is especially notable: it can read supported RocksDB database files without linking the RocksDB C++ library. It is designed for migration rather than as a general substitute for a live RocksDB administration interface.

Opening historical storage with the new engine can trigger one-way format conversion. The upstream documentation explicitly cautions that the retained original files are not necessarily a complete rollback point; a separate copy is needed for rollback safety.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Primary sources:

- `https://github.com/surrealdb/surrealkv`
- `https://github.com/surrealdb/surrealkv/tree/main/crates`
- `https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0`

### 3.4 RocksDB — mature persistent engine and native diagnostic utilities

For SurrealDB 3.3, RocksDB remains the officially recommended persistent engine for conventional single-node server workloads. SurrealDB's Rust integration uses its separately versioned `surrealdb-rocksdb` bindings to the C++ RocksDB engine.

RocksDB provides an extensive operational and performance-analysis ecosystem, much of which operates independently of SurrealDB.

| Tool / interface                          | Diagnostic or management capability                                                                    |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `ldb`                                     | Scan raw keys and values, inspect manifests, read properties, and perform administrative KV operations |
| `sst_dump`                                | Inspect SSTable records, properties, compression, structure, and checksums                             |
| `db_bench`                                | Benchmark RocksDB storage primitives and workload configurations                                       |
| `db_stress`                               | Exercise concurrency, recovery, and correctness under stress                                           |
| `trace_analyzer`                          | Analyze RocksDB request traces and access patterns                                                     |
| Block-cache analysis tools                | Investigate cache hits, misses, and key/block access distributions                                     |
| I/O tracing and analysis                  | Identify read/write and storage-device access behavior                                                 |
| RocksDB statistics / property APIs        | Observe compaction, flush, WAL, stalls, block cache, and memory utilization                            |
| `rocksdb` / `surrealdb-rocksdb` Rust APIs | Programmatic access to engine options, statistics, and native storage mechanisms                       |

The important diagnostics are at the LSM-tree level: how many bytes are written during compaction, whether write stalls occur, how often SSTables are read, what percentage of block-cache accesses miss, and how WAL/fsync activity contributes to latency.

These can explain performance behavior that a SurrealQL query plan cannot.

However, RocksDB utilities inspect physical KV records encoded by SurrealDB, not ordinary `person:alice` records or graph relationships. Editing those KV records directly bypasses SurrealDB's schema, indexes, transactional invariants, and metadata management. Standalone database-opening tools may also conflict with the live server's database lock. Read-only offline inspection or isolated snapshots are materially safer than modifying live files.

For Rust-only application code, the practical integration distinction is between using the RocksDB crate for a separate KV application and observing RocksDB as a storage engine managed by SurrealDB. The former gives direct control over its API; the latter does not automatically expose every RocksDB option or property through SurrealQL.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Primary sources:

- `https://github.com/facebook/rocksdb`
- `https://github.com/facebook/rocksdb/wiki/Administration-and-Data-Access-Tool`
- `https://github.com/facebook/rocksdb/wiki/RocksDB-Overview`
- `https://github.com/surrealdb/rocksdb`
- `https://surrealdb.com/docs/manage/self-hosted/deployment-models`

### 3.5 TiKV and distributed-storage tooling

SurrealDB also supports a TiKV backend for local distributed experimentation. TiKV is a distributed transactional KV engine built in Rust, backed by a Placement Driver (PD) cluster and Raft replication.

Its independent tooling ecosystem includes TiUP for deployment and component management, PD and TiKV control utilities, and monitoring and debugging facilities for replication, region placement, hot regions, storage latency, and distributed transaction conflicts.

The main diagnostic concerns differ fundamentally from RocksDB:

| Diagnostic problem                          | Relevant tool or mechanism                   |
| ------------------------------------------- | -------------------------------------------- |
| Placement, leaders, regions, and scheduling | PD API / `pd-ctl`                            |
| Node state and low-level store inspection   | `tikv-ctl`                                   |
| Local cluster provisioning                  | TiUP                                         |
| Consensus and region imbalance              | TiKV / PD metrics                            |
| Distributed storage latency                 | TiKV metrics, tracing, and client statistics |
| Network partitions and failed nodes         | Cluster logs, Raft and PD diagnostics        |

TiKV support should not be equated with SurrealDB's commercial multi-node high-availability offering. Official 3.3 documentation describes Community TiKV deployments as suitable for local multi-node experimentation; production multi-node HA is supplied through SurrealDB Cloud Scale or self-hosted Enterprise distributed storage. Those platforms have different diagnostic and operational boundaries.

Likewise, multiple SurrealDB processes must not concurrently mount one RocksDB directory in an attempt to obtain clustering. RocksDB is a local embedded storage engine, not a shared distributed database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Primary sources:

- `https://github.com/tikv/tikv`
- `https://github.com/tikv/pd`
- `https://github.com/pingcap/tiup`
- `https://docs.pingcap.com/tidb/stable/`
- `https://surrealdb.com/docs/manage/self-hosted/deployment-models`

### 3.6 Storage-engine selection as a tooling boundary

These are not simply replaceable implementations with identical operating characteristics.

| Capability                             | SurrealMX                                   | SurrealKV                                          | RocksDB                                         | Distributed storage                               |
| -------------------------------------- | ------------------------------------------- | -------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------- |
| Primary role                           | In-memory transactional KV                  | Persistent Rust-native KV                          | Persistent mature LSM KV                        | Shared transactional storage                      |
| Direct Rust engine access              | Yes                                         | Yes                                                | Rust bindings                                   | Backend/client-specific                           |
| Native diagnostics                     | GC, MVCC, snapshots, simulations            | `skv`, WAL, SST, scrub, migration                  | `ldb`, `sst_dump`, compaction and cache tooling | Consensus, replica, region and cluster monitoring |
| SurrealDB 3.3 temporal `VERSION` reads | No                                          | Supported when configured                          | Supported when configured                       | Depends on deployment                             |
| Integrated SurrealDB 3.3 status        | Stable backend                              | Beta backend                                       | Recommended persistent backend                  | Deployment-dependent                              |
| Standalone 1.0 implication             | New SurrealMX crate independently available | New stable SurrealKV crate independently available | Independently versioned C++ engine              | Separate platform lifecycle                       |

Notably, SurrealDB 3.3 rejects temporal versioning configuration on the in-memory backend, even though standalone SurrealMX internally maintains MVCC versions. MVCC for transactional concurrency is not the same thing as user-addressable historical queries.

The SurrealDB documentation identifies 3.4 as the planned integration point for SurrealKV 1.0 as its recommended disk engine, with a broader RocksDB replacement direction for 4.0. Neither prospective integration should be attributed to the 3.3.4 release.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



## 4. SurrealDB CLI and administration toolchain

### 4.1 Versioned component launcher and database CLI

The official `surreal` launcher now manages independently versioned command-line components. The database component is invoked through `surreal db`.

Its most useful administrative commands are:

| Command               | Function and important distinction                                                        |
| --------------------- | ----------------------------------------------------------------------------------------- |
| `surreal db start`    | Start local server; select engine, auth, capability policy, logging and network endpoints |
| `surreal db sql`      | Interactive REPL or script execution; connects remotely or opens an embedded engine       |
| `surreal db validate` | Syntax parsing only for SurrealQL files, not schema or execution correctness              |
| `surreal db export`   | Logical SurrealQL export                                                                  |
| `surreal db import`   | Restore or apply a SurrealQL export                                                       |
| `surreal db is-ready` | Check remote server readiness                                                             |
| `surreal db version`  | Inspect CLI/server version information                                                    |
| `surreal db mcp`      | Run embedded MCP over stdio                                                               |
| `surreal db module`   | Scaffold, build, inspect and invoke WASM extensions                                       |
| `surreal db ml`       | Manage SurrealML model import/export                                                      |
| `surreal db fix`      | Historical migration command; not implemented in SurrealDB 3.x                            |

The last entry merits emphasis: in 3.x, `fix` is still advertised in command references but exits with `Fix is not implemented`. The working 1.x-to-2.x conversion belongs to a 2.x binary. It is not a general SurrealDB 3.x database repair tool.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



The component launcher also enables side-by-side version installation and targeted invocations, such as:

```
surreal install db
surreal upgrade db
surreal use db@3

surreal db version
surreal db validate "schema/**/*.surql"
surreal db is-ready --endpoint http://127.0.0.1:8000
```

For command-specific capabilities, actual help output from the installed binary is authoritative. The container image runs the database executable directly, so its command form remains `surreal start`, not `surreal db start`.

Direct source: `https://surrealdb.com/docs/reference/cli`

### 4.2 Embedded versus remote SQL shell

One less obvious capability is that the CLI REPL can operate as an embedded database host. For example:

```
# Connect to an existing SurrealDB server
surreal db sql --endpoint ws://127.0.0.1:8000

# Run the SurrealDB engine directly inside the CLI process
surreal db sql --endpoint surrealkv://./data
```

In remote mode, security capabilities and experimental features are enforced by the server. In embedded mode, they are configured on the CLI process because it is itself executing the database.

This makes the CLI valuable for reproducing an issue without the network transport, but opening a persistent datastore independently should be done only when another process is not using that datastore. It is not a general-purpose concurrent attach mechanism.

SurrealDB 3.3 additionally supports `grpc://` and `grpcs://` endpoints from the SQL shell.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/reference/cli/db/commands/sql`

### 4.3 Datastore lifecycle and configuration diagnostics

The `start` command and the associated configuration reference expose materially different controls for each backend:

```
# Durable, temporal SurrealKV server
surreal db start --user root --pass secret \
  "surrealkv://data?versioned=true&sync=every&retention=30d"

# Persistent RocksDB server
surreal db start --user root --pass secret \
  "rocksdb://data?versioned=true&retention=30d"

# In-memory SurrealMX with an append-only log and snapshots
surreal db start --user root --pass secret \
  "mem://data?aol=sync&snapshot=60s&sync=5s"
```

These are illustrative configurations, not interchangeable defaults. SurrealMX uses AOL/snapshot controls; persistent engines use separate flush and temporal-history settings.

The official configuration reference also includes storage-engine-specific options such as RocksDB cache and compaction settings, SurrealKV value-log options, memory allocation, query timeouts, transaction controls, live-query behavior, and experimental feature gates.

Two easy-to-miss operational details are important:

- Storage URLs are path-sensitive. An unintended working directory can create what appears to be an empty database; mixing engine types in one directory risks corrupting files.
- `--user` and `--pass` create the root user on a new datastore. They do not reset an existing root password when the server is restarted.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

  Database



Direct sources:

- `https://surrealdb.com/docs/reference/cli/db/commands/start`
- `https://surrealdb.com/docs/reference/cli/db/environment-variables`

### 4.4 Backup, restore and recovery tooling

SurrealDB offers two distinct backup layers.

Logical exports use `surreal db export` or the Rust SDK's `.export()` method. They produce portable SurrealQL that can be imported into another compatible datastore, independently of whether the original used RocksDB or SurrealKV.

Physical recovery uses storage-engine checkpoints, snapshots or consistent copies of the datastore directory. These preserve engine-specific state but introduce storage-format and engine-version dependencies.

Logical exports have two particularly significant limitations in 3.3.4:

- They export current record state, not historical versions, even when temporal storage is enabled. The `--versions` export option is rejected from version 3.3.1.
- Secret access-signing keys are redacted. Imports cannot recreate those accesses without separately supplied keys. Versions before 3.3.1 could recreate an access with the literal redacted placeholder as its signing key, creating a security hazard.

This makes logical export, physical backup, and temporal history three different mechanisms rather than interchangeable forms of recovery.

The Rust SDK exposes the same logical export/import capability, including when SurrealDB is embedded. Cloud instances additionally have independently managed backup policies and instance-level backup commands.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Direct sources:

- `https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery`
- `https://surrealdb.com/docs/reference/cli/db/commands/export`
- `https://surrealdb.com/docs/reference/cli/cloud/commands`

### 4.5 SurrealDB Cloud CLI and `surrealctl`

The current Cloud control-plane tooling is `surreal cloud`, the successor to the independently installed `surrealctl` binary. Its command families cover:

- Organisation administration, roles, invitations, access tokens and cloud contexts.
- Instance create, update, pause, resume, resize, status, endpoint and deletion.
- Instance logs, metrics, usage and cost estimates.
- Backup creation, listing and policies.
- Supported regions, storage types, instance sizes and database versions.
- Scriptable Cloud API calls and machine-readable output.

Especially useful diagnostic commands include `surreal cloud instance metrics`, `logs`, `status`, `watch`, `usage`, and `surreal cloud status` (also exposed through the `doctor` alias).

Cloud commands support structured JSON output and documented exit codes, allowing them to be integrated into automated diagnostics. This is a control-plane interface; it is not a replacement for local `surreal db` commands or direct storage-engine diagnostics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Direct source: `https://surrealdb.com/docs/reference/cli/cloud`

## 5. SurrealKit — schema management, migration, testing and generated metadata

SurrealKit is one of the most significant independent tooling components in the ecosystem because it adds database lifecycle capabilities beyond the native `DEFINE` and `REMOVE` statements.

It is an official Rust crate and CLI. At the research date, the documentation distinguishes the stable 0.7 line from the 1.0 beta line; the 1.0 documentation examples specifically describe `1.0.0-beta.1`, while the upstream repository contains subsequent beta changes. Consequently, particular 1.0 behaviors must be checked against the installed beta rather than attributed to stable 0.7.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### 5.1 Declarative sync and schema reconciliation

`surrealkit sync` treats `.surql` files as a desired schema state.

The engine tracks definitions and their source hashes in metadata tables, then applies changes and prunes objects whose definitions have disappeared.

Relevant mechanisms include:

- Change detection and skipping unchanged schema files.
- `--watch` mode with debounced automatic reconciliation.
- `--dry-run`, `--fail-fast`, and selection of specific schema modules or targets.
- Detection of empty schema directories to reduce accidental destructive pruning.
- Guardrails around pruning shared databases.
- Tracking of tables, fields, indexes, analyzers, users, access methods, functions, modules, models, buckets and other schema objects.

The distinction between schema-object tracking and mere file execution matters. Allowing arbitrary non-`DEFINE` statements weakens SurrealKit's ability to track individual entities and their removal.

SurrealKit's own metadata tables, such as `__entity`, `__rollout` and `__seed`, form its internal schema lifecycle model. Those tables should be treated as tool-owned state, not ordinary application schema.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.2 Phased rollouts and migration recovery

The `surrealkit rollout` family offers a more controlled alternative to immediate sync.

| Command            | Capability                                                       |
| ------------------ | ---------------------------------------------------------------- |
| `rollout baseline` | Capture an existing database schema as an initial baseline       |
| `rollout plan`     | Diff desired schema and generate a reviewable migration manifest |
| `rollout lint`     | Validate a manifest without applying it                          |
| `rollout start`    | Apply the expansion/non-destructive phase                        |
| `rollout status`   | Inspect persistent rollout state                                 |
| `rollout complete` | Apply the contraction/destructive phase                          |
| `rollout rollback` | Undo the expansion phase before completion                       |
| `rollout repair`   | Reconcile interrupted rollout metadata without rerunning SQL     |

The tool stores a persistent state machine and tracks rollout steps so interrupted operations can be recovered.

A particularly important nuance: `rollout repair` is not equivalent to database storage repair. It reconciles rollout bookkeeping and does not rerun the underlying SQL. Likewise, rollback before the contraction phase should not be confused with a guarantee that arbitrary historical data can be reconstructed after a destructive migration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/manage/schema-migration/rollouts`

### 5.3 Declarative database and permission testing

SurrealKit includes a database testing framework using declarative TOML suites. Its test model includes:

- Isolated ephemeral databases per suite.
- Schema setup, seeding, and parallel test execution.
- Authenticated actors, including root, database users, record-level access, tokens, and custom headers.
- SurrealQL success/failure assertions and returned-value assertions.
- Tests of custom HTTP API endpoints.
- `--suite`, `--case`, `--tag` and `--parallel` filtering.
- `--keep-db` for retaining a failed test database for inspection.
- `--json-out` for machine-readable diagnostics.

This is consequential because schema constraints, authorization rules and custom APIs can be tested at the database boundary, rather than reimplemented solely as application-side assertions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.4 Type generation and Rust integration

`surrealkit typegen` introspects the live schema and produces a structured JSON representation. It can additionally emit TypeScript types.

It does not currently provide native Rust struct generation. The JSON output, however, is independently accessible and can support downstream tooling without requiring a second introspection implementation.

As a Rust library, SurrealKit exposes schema introspection, sync, rollout and other functionality from within an application process. This is especially relevant for embedded SurrealDB, where independently starting a second process against the same persistent datastore may be impossible due to file locking.

The 1.0 API documentation also covers an `embed_schema!` macro for packaging schema definitions into a Rust binary, reducing dependence on separate schema files at runtime.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Primary sources:

- `https://github.com/surrealdb/surrealkit`
- `https://surrealdb.com/docs/manage/schema-migration`
- `https://surrealdb.com/docs/manage/schema-migration/library`
- `https://surrealdb.com/docs/manage/schema-migration/typegen`

### 5.5 Alternative Rust migration tooling

The community-maintained `surrealdb-migrations` crate provides a different model: versioned schema, event and data migrations with a Rust `MigrationRunner` and CLI.

It is useful to distinguish these approaches:

- SurrealKit: desired-state schema reconciliation, staged rollouts, schema metadata, seeding, type export, and declarative tests.
- `surrealdb-migrations`: migration-file execution and versioned migration history, with a model closer to conventional migration runners.

The latter project's own published material includes maturity cautions, so its production suitability should be assessed separately from SurrealKit's official status.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://github.com/Odonno/surrealdb-migrations`

## 6. Rust SDK, data types and protocol-level tooling

### 6.1 `surrealdb` — Rust SDK and embedded database host

The official Rust SDK 3.3.4 requires Rust 1.95 or newer and supports server and embedded database access.

Its architecture includes several independently useful feature sets:

| Functionality                      | Rust SDK mechanism                                      |
| ---------------------------------- | ------------------------------------------------------- |
| Remote connection                  | HTTP, WebSocket, and 3.3 gRPC transport                 |
| Embedded in-memory execution       | `kv-mem` / SurrealMX                                    |
| Embedded RocksDB                   | `kv-rocksdb`                                            |
| Embedded SurrealKV                 | `kv-surrealkv`                                          |
| Queries and variable binding       | `.query()` and `.bind()`                                |
| Typed records                      | `SurrealValue`, `RecordId`, and typed result extraction |
| Transactions                       | Client-side `.begin()`, `.commit()`, `.cancel()`        |
| Continuous updates                 | `LIVE SELECT`, `.live()`, and notification streams      |
| Large query results                | 3.3 `.stream_items()`                                   |
| Backups                            | `.export()` and `.import()`                             |
| Introspection and server functions | SurrealQL and SDK function calls                        |

RocksDB requires C++ compilation and `libclang` for its embedded feature. SurrealKV is Rust-native and does not carry that RocksDB build requirement.

The storage features should be selected deliberately: embedding the database engine is materially different from building an ordinary remote client.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Direct sources:

- `https://surrealdb.com/docs/reference/rust`
- `https://surrealdb.com/docs/reference/rust/embedding`
- `https://docs.rs/surrealdb/`

### 6.2 `surrealdb-types` and the `SurrealValue` type system

SurrealDB 3.x exposes a dedicated type crate independently of the full engine.

Important facilities include:

- The `SurrealValue` trait and derive macro.
- Typed `RecordId` and record-ID key representations.
- `kind_of()` for mapping supported Rust types to SurrealQL kinds.
- Surreal-specific derive attributes, including renaming and flattening.
- Value-construction macros such as `object!`, `array!`, `set!`, `vars!`, and `kind!`.
- Explicit conversions between Rust values and SurrealDB's own value model.

A `RecordId` is not merely a string. Its table and key have distinct semantics, and keys may be strings, numbers, UUIDs, or other supported forms. Binding `"person:1"` as a string is not equivalent to binding a typed `RecordId` with a numeric key.

Likewise, `SurrealValue` derive attributes are separate from Serde's derive attributes. Treating them as interchangeable can produce unexpected serialization and deserialization behavior.

This type system provides a better boundary for Rust tooling than untyped JSON conversion when database-specific values or structured errors must be preserved.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/reference/rust/concepts/working-with-types`

### 6.3 Structured errors and execution diagnostics

The 3.x SDK exposes structured errors with stable kinds, optional details and a cause chain.

Errors can represent validation, configuration, query, serialization, permission, not-found, already-exists, connection and internal failures.

There are also two levels of query error:

1. The overall Rust `Result` may fail because of a connection, authentication, parsing, or request-level problem.
2. A successful request may contain statement-level errors in its returned response.

`take_errors()` and `.check()` are therefore important in diagnostics for multi-statement scripts; testing only whether `.query(...).await` returned `Ok` can miss statement failures. Error kinds are more stable for programmatic classification than matching error-message text.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/reference/rust/concepts/error-handling`

### 6.4 Client-side transactions, live queries and streaming

Client-side transactions let Rust code hold a transaction across multiple SDK operations rather than expressing the entire unit of work as one query string.

This changes resource lifetime and concurrency behavior: callers need to manage commit/cancel paths, connection failures, contention and transaction duration.

In 3.3, `.stream_items()` is particularly important for memory-sensitive diagnostics and large scans. It streams rows before the whole query result is materialized and produces statement-end events with result status and statistics.

Rows may arrive before the final statement succeeds. A consumer that writes streamed rows to an irreversible external sink before receiving the successful terminal event must account for that distinction.

This is different from a `LIVE SELECT` subscription, which streams future data changes rather than progressively emitting the results of an ordinary query.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Direct sources:

- `https://surrealdb.com/docs/reference/rust/concepts/transaction`
- `https://surrealdb.com/docs/reference/rust/methods/query`

### 6.5 Engine-level Rust crates and SurrealQL internals

SurrealDB's workspace contains more reusable-looking Rust crates than the public SDK alone suggests. They are valuable reference points for understanding implementation boundaries, but most are internal engine components, not stable extension APIs.

| Component                | Responsibility                                             |
| ------------------------ | ---------------------------------------------------------- |
| `surrealdb-core`         | Core query, execution, transaction and datastore machinery |
| `surrealdb-syn`          | SurrealQL parsing and syntax infrastructure                |
| `surrealdb-types`        | Public database value representations and type conversions |
| `surrealdb-idx`          | Indexing subsystem and search-related implementation       |
| `surrealdb-kvs`          | Datastore abstractions and storage integration             |
| `surrealdb-engine-local` | Embedded engine integration                                |
| `surrealdb-server`       | Server-side protocols and services                         |
| `surrealdb-gql`          | ISO GQL parsing and execution integration                  |
| `surrealdb-mcp`          | Embedded database MCP implementation                       |

These packages can help an advanced Rust developer investigate parser behavior, optimizer decisions, index machinery, storage mappings or MCP internals.

The boundary is significant: using an internal crate does not convey the same compatibility expectations as using the published SDK. In particular, `surrealdb-gql` is explicitly an internal implementation crate; it is not a stable public Rust graph-query-builder API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+1



Source: `https://github.com/surrealdb/surrealdb/tree/v3.3.4`

## 7. SurrealQL authoring, language intelligence and static diagnostics

### 7.1 Official Rust SurrealQL Language Server

The official SurrealQL Language Server is particularly valuable because it goes well beyond editor syntax highlighting.

Its Rust implementation uses a dedicated tree-sitter grammar and performs semantic analysis across SurrealQL definitions and query expressions.

Its capabilities include:

- Tree-sitter syntax diagnostics and schema-aware semantic diagnostics.
- Type inference from `DEFINE` statements, field types and query flow.
- Contextual completion of fields, typed record references, functions and statements.
- Hover information for inferred types, function signatures and permission posture.
- Go-to-definition, references and rename support.
- Function call hierarchy, document symbols and signature help.
- Code actions relating to missing permissions.
- Recognition of embedded JavaScript function bodies, avoiding inappropriate SurrealQL parsing errors.

These features create a static-analysis layer that does not require executing every query against a running database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



The language server is available as its own CLI component:

```
surreal install lsp

surreal lsp stdio

surreal lsp check queries/
surreal lsp check queries/ --json --fail-on warning
surreal lsp check --stdin \
  --stdin-filename src/feed.surql \
  --workspace schema/
```

`surreal-lsp` is also the name of the native binary built from source.

The headless `check` command is noteworthy: it exposes the same analyzer used by editor integrations, returns structured diagnostics in JSON, and has nonzero exit codes suitable for pre-commit checks, CI and agent-driven repair loops. Its JSON diagnostics use LSP ranges, stable diagnostic codes and structured hints.

Compared with other tooling:

| Tool                        | Validation boundary                                                      |
| --------------------------- | ------------------------------------------------------------------------ |
| `surreal db validate`       | SurrealQL syntax parsing                                                 |
| `surreal lsp check`         | Static syntax, schema and semantic analysis                              |
| `surrealkit rollout lint`   | Migration-manifest validation                                            |
| SurrealKit tests            | Actual database execution, returned values, permissions and API behavior |
| Rust SDK `.query().check()` | Runtime statement execution and errors                                   |

The tools are complementary rather than duplicates. Static inference can detect many mistakes early, but it cannot reproduce runtime indexes, record contents, permissions, transaction conflicts or query latency.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Primary source: `https://github.com/surrealdb/surrealql-language-server`

### 7.2 Tree-sitter grammar and machine-readable function metadata

The language server has two additional components of interest for constructing Rust-based development tooling.

Dedicated SurrealQL tree-sitter grammar. This is maintained in a separate upstream repository. The language-server project pins a grammar revision because its semantic analyzer depends on the grammar's node kinds. This makes the grammar useful as a parsing foundation, but introduces a version-alignment obligation for tools implementing their own analysis.

Source: `https://github.com/surrealdb/surrealql-tree-sitter`

Generated built-in function catalogue. The language-server repository contains `builtins.json`, generated from the SurrealDB engine's parser, function dispatch machinery and type metadata. At the checked revision, the catalogue describes 434 functions, including argument types, return types, arity and method receivers.

Unlike handwritten duplicated function inventories, this catalogue is derived from a pinned engine checkout and verified for freshness in CI. It can support function-aware analysis or generated documentation without maintaining another independently authored source of SurrealQL function signatures.

The generator also includes a `verify-returns` facility that exercises declared function-return metadata against an in-memory engine. That is a useful tool for detecting inconsistency between the engine's type descriptions and its actual results.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Direct source: `https://github.com/surrealdb/surrealql-language-server/blob/master/builtins.json`

### 7.3 Formatting, editors and alternative parsers

The official formatter is `surqlfmt`, distributed as `@surrealdb/surql-fmt`. It is not currently a Rust-native CLI component. It supports formatting and check-only operation for SurrealQL source files.

```
npm install -g @surrealdb/surql-fmt
surqlfmt --check './schema/**/*.surql'
```

For an otherwise Rust-only application stack, this can remain an isolated development tool; it does not introduce a JavaScript dependency into the Rust server runtime.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/reference/cli/formatter/overview`

A separate community project, `surql-parser`, offers Rust parsing and development tooling built around SurrealQL 3.x. Its components include a `surql` CLI, compile-time checking macros, schema analysis and supporting packages for migrations and MCP.

This is worth investigating where a Rust process needs programmatic language manipulation rather than simply connecting to SurrealDB. It is not an official SurrealDB compatibility guarantee, and its API and maturity should be assessed independently of the official language server.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Direct source: `https://github.com/overrealdb/surql-parser`

## 8. Model Context Protocol: database, Cloud and memory

SurrealDB now has several distinct MCP surfaces. They should not be conflated merely because all speak the Model Context Protocol.

### 8.1 Embedded database MCP

Built into SurrealDB since 3.1, this server exposes database operations through MCP without requiring a separately maintained database adapter.

| Tool                                                       | Functionality                                                             |
| ---------------------------------------------------------- | ------------------------------------------------------------------------- |
| `query`                                                    | Execute SurrealQL                                                         |
| `gql`                                                      | Execute ISO GQL                                                           |
| `graphql`                                                  | Execute configured GraphQL queries                                        |
| `select`, `create`, `insert`, `upsert`, `update`, `delete` | Record CRUD                                                               |
| `relate`                                                   | Create graph relationships                                                |
| `run`                                                      | Call database functions                                                   |
| `list`                                                     | Discover namespaces, databases, tables, indexes, users and other metadata |
| `info`                                                     | Inspect schema or engine information                                      |
| `use`                                                      | Select database context, subject to MCP protocol behavior                 |

The MCP server also exposes schema information as resources. Importantly, table and field `COMMENT` definitions appear in metadata provided to agents, making native schema documentation useful for automated query construction.

The `gql` tool is not read-only: ISO GQL supports mutations, and SurrealDB 3.3 corrects its tool annotations accordingly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



There are two deployment modes.

A. Stdio MCP — database owned by the agent process

```
surreal db mcp memory
```

The process embeds its own datastore, running locally and communicating over standard input/output. It operates with owner-level authority over that embedded database.

This is appropriate for isolated experimentation, not an authenticated connection into an already-running production database. A persistent path also raises the same storage-locking concerns as other embedded clients.

B. HTTP MCP — existing SurrealDB instance

```
http://127.0.0.1:8000/mcp
```

Here the MCP endpoint shares the server's data, permissions, authentication and capability policy. It is the natural choice for inspecting an existing remote or locally hosted database.

The HTTP endpoint has its own protections and limits, including:

- Allowed-host restrictions, loopback by default.
- An execution timeout, defaulting to 60 seconds.
- A serialized result cap of 256 KiB by default.
- Limits on parameter count, argument count and embedded SurrealQL parameter length.
- HTTP body-size limits.

These are important when an agent reports truncated results, unexplained failures, or inability to retrieve a large diagnostic dataset. They also mean MCP should not be treated as a substitute for bulk export or streaming SDK interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/build/ai-agents/mcp/embedded`

### 8.2 Hosted SurrealDB Cloud MCP

The hosted MCP endpoint is:

`https://mcp.surrealdb.com`

It exposes the database data tools plus the SurrealDB Cloud control plane:

- Organisation membership and access administration.
- Instance creation, deletion, configuration, pausing, upgrading and resizing.
- Instance status, metrics, logs and usage.
- Billing, supported versions, regions and instance catalogues.
- Cloud database queries and record operations.
- Agent Memory context administration.

It uses account-level authentication and Cloud role/scope controls rather than granting unrestricted local ownership.

The main distinction is management scope. The built-in `/mcp` endpoint operates on a database. Cloud MCP can manage the hosted environment around that database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/build/ai-agents/mcp`

The official Rust repository `surrealmcp` also exists as an independently maintained MCP-server codebase. It is a separate packaging and deployment concern from the built-in database MCP and the hosted Cloud service; those interfaces should not be presumed version-identical.

Source: `https://github.com/surrealdb/surrealmcp`

### 8.3 SurrealDB Agent Memory MCP

SurrealDB Agent Memory, previously developed under the Spectron name, operates as a separate application layer backed by SurrealDB.

Its context-level MCP server exposes seven purpose-specific tools:

`remember`, `recall`, `context`, `reflect`, `forget`, `upload`, `inspect`.

Unlike database MCP, those tools represent higher-order memory operations: fact ingestion, hybrid retrieval, contextual assembly, reflection, erasure and document handling.

Agent Memory maintains provenance, temporal belief history, graph-linked entities and retrieval/decision traces in the backing database. This is materially different from simply issuing SurrealQL statements from an agent.

The MCP endpoint is served at `/mcp` on an Agent Memory context host and uses the context's authorization model. The public documentation does not currently list a dedicated Rust Agent Memory SDK or an embeddable Agent Memory library; Rust applications can use its REST/OpenAPI or MCP interface instead.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Agent Memory

+1



Its separate `agent-memory` CLI provides:

- `remember`, `recall`, `context`, `reflect`, `forget` and `chat`.
- Document upload, ingestion and retrieval.
- Session, entity and trace inspection.
- An interactive REPL and a four-pane terminal workbench (`tui`).
- Replay of recorded JSONL interactions without live HTTP access.
- Retrieval modes including hybrid, vector, BM25 and graph.

The trace inspector and replay facilities are particularly relevant to diagnosing why a retrieval pipeline selected particular evidence or produced a particular remembered conclusion.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Agent Memory



Primary sources:

- `https://surrealdb.com/docs/agent-memory`
- `https://surrealdb.com/docs/agent-memory/reference/cli`
- `https://surrealdb.com/docs/agent-memory/integrations`

### 8.4 Official agent-facing documentation packages

SurrealDB also publishes agent skills as instruction packages for coding agents. These are not MCP servers; they supply domain-specific guidance on SurrealQL, vector search, performance and related workflows.

They complement MCP by improving how agents formulate database operations before invoking tools. No server-side Rust dependency is implied.

Source: `https://github.com/surrealdb/agent-skills`

## 9. Graph-query tooling, graph storage and traversal diagnostics

SurrealDB offers three distinct graph-query interfaces, plus the visual exploration provided by Studio:

1. Native SurrealQL graph traversal expressions over `RELATE` edges and record references.
2. ISO GQL graph-pattern queries, including `MATCH` and path-search constructs.
3. GraphQL, which generates a typed API from database schema definitions.

They operate on overlapping data but serve different purposes. ISO GQL is not GraphQL, and neither should be assumed to accept arbitrary statements from the other language.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 9.1 Native graph models and query planning

SurrealDB represents relationships in several ways.

| Representation           | Semantics and tooling implications                                              |
| ------------------------ | ------------------------------------------------------------------------------- |
| Record links             | Typed or untyped record IDs stored in document fields                           |
| `REFERENCE` fields       | Inverse reference tracking for records referenced elsewhere                     |
| Ordinary relation tables | First-class edge records with `in`, `out`, fields, events and indexes           |
| `ENFORCED` relations     | Typed endpoint restrictions and referential-integrity checks                    |
| `LIGHTWEIGHT` relations  | Record-less edges with identity derived from endpoint pairs                     |
| ISO GQL patterns         | Declarative graph matching over the same underlying tables and relation records |

Native graph paths can use `->`, `<-`, and `<->` traversal operators. They compose with field projections, predicates, record dereferencing and recursive path expressions.

This integration can eliminate separately maintained graph projections for workloads where documents and relationships already live in SurrealDB. The tradeoff is that traversal fanout, edge-record decoding, intermediate result shapes and permissions still determine actual cost.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### 9.2 SurrealDB 3.3: lightweight relations

`LIGHTWEIGHT` is one of the most consequential graph-storage additions in 3.3.

```
DEFINE TABLE follows
    TYPE RELATION IN person OUT person
    LIGHTWEIGHT;
```

A lightweight relation has no separately stored edge record. Its identity is the ordered endpoint pair, such as `follows:[person:a, person:b]`.

The implications are unusually strong:

- Repeating `RELATE` for the same endpoints is idempotent.
- The relation cannot have arbitrary fields, indexes or events.
- It cannot be updated to attach properties.
- Endpoint integrity is enforced.
- Reads synthesize the expected edge shape with `id`, `in` and `out`.
- Existing populated ordinary relation tables cannot simply be redefined as lightweight.

This is more than a storage optimization. It changes the graph's identity and uniqueness model: duplicate-prevention machinery becomes unnecessary when an edge is semantically just one directed connection.

For relationships that need weights, timestamps, provenance, metadata, live subscriptions or edges that themselves serve as graph vertices, ordinary relation tables remain different and necessary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 9.3 Inline adjacency caches and graph-folding diagnostics

SurrealDB 3.3 also introduced `INLINE EDGES` and `INLINE REFERENCES` on vertex tables.

```
DEFINE TABLE person
    INLINE EDGES 64
    INLINE REFERENCES 32;
```

These maintain small adjacency caches directly alongside vertex data so a traversal can use batched lookups rather than independent storage-range scans for every source record.

The limits are significant:

- Each cache has a maximum configured capacity of 256 entries.
- High-degree records above the cap fall back to ordinary scans.
- Cache maintenance happens transactionally on edge/reference changes.
- Caches are built as relevant writes occur; enabling the option does not immediately populate every record's cache.
- Cache-relevant queries can avoid some scans, but maintaining larger caches increases write amplification.

`EXPLAIN ANALYZE` reports `GraphEdgeScan` and `ReferenceScan` counters including `cache_hits` and `cache_misses`.

A related background graph-folding mechanism groups recently written adjacency keys into blocks. Its interval can be configured through `SURREAL_GRAPH_FOLD_INTERVAL`, defaulting to five seconds. Query plans report `block_hits` and `delta_hits`, distinguishing reads satisfied through packed blocks from reads through individual adjacency keys.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



This is a valuable diagnostic distinction: a graph traversal can be slow because of the number of logical edges, because its adjacency representation generates too many KV reads, or because a graph cache has poor coverage. Query-plan counters make these causes more distinguishable.

### 9.4 Inline edge properties and predicate pushdown

The `INLINE` attribute on a relation field duplicates its value into adjacency entries.

```
DEFINE TABLE likes TYPE RELATION IN person OUT post;
DEFINE FIELD score ON likes TYPE number INLINE;

SELECT ->(likes WHERE score > 8)->post
FROM person:one;
```

A traversal can then test `score` while scanning adjacency entries, avoiding the retrieval and decoding of edge records that fail the predicate.

This works when the predicate is placed inside the traversal path. `EXPLAIN` identifies this mechanism with `predicate_scope: payload`.

The architectural distinction is between denormalization manually maintained by application code and denormalization maintained transactionally by SurrealDB itself. The latter preserves the database's consistency guarantees while imposing additional write-side storage and maintenance cost.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct sources for these graph mechanisms:

- `https://surrealdb.com/docs/reference/query-language/statements/define/table`
- `https://surrealdb.com/docs/reference/query-language/statements/define/field`
- `https://surrealdb.com/docs/learn/data-models/graph/graph-traversal`

### 9.5 ISO GQL: dedicated graph-query engine

ISO GQL became enabled by default in SurrealDB 3.3, after its experimental introduction in 3.2.

It exposes graph patterns such as:

```
MATCH (a:person)-[:knows]->(b:person)
RETURN a.name, b.name
```

Advanced capabilities include multi-pattern matching, optional match blocks, path quantifiers, grouping, aggregation, shortest-path forms and mutations.

Unlike an adapter that translates Cypher into SurrealQL strings, the GQL implementation parses and lowers graph queries into SurrealDB's internal execution machinery, including graph-match planning.

Access methods include:

- `POST /gql`
- `gql` over the WebSocket RPC protocol
- The `gql` MCP tool
- PostgreSQL wire connections using `SET dialect = 'gql'`
- The capability-gated `eval::gql()` function inside SurrealQL

The dialect follows ISO GQL syntax rather than claiming complete compatibility with Neo4j's openCypher. Differences in comments, labels, inequality expressions, path quantifiers and available operators can affect migration tooling.

`eval::gql()` is separately permission-gated, even though the external GQL endpoint is on by default in 3.3. The `eval` capability is not enabled merely by `--allow-all`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/learn/querying/gql/overview`

### 9.6 Graph visualization and schema inspection

SurrealDB Studio provides graph-oriented query-result visualization, interactive graph exploration and schema management. It is not a Rust-native desktop binary, but it operates against the same Rust database and can inspect graph structures without an additional application-specific visualizer.

The CLI and MCP provide corresponding machine-readable functionality through `INFO`, `list`, query results and `EXPLAIN ANALYZE`. Studio is best understood as a visual consumer of the database's metadata and query interfaces, not a separate graph storage or execution engine.

## 10. Vector, full-text and hybrid-search diagnostics

### 10.1 HNSW, DISKANN and exact search

The principal vector-search choices are:

| Search mechanism     | Key properties                                                | Tooling implications                                           |
| -------------------- | ------------------------------------------------------------- | -------------------------------------------------------------- |
| Brute-force distance | Exact comparison with matching vectors                        | Useful accuracy reference; may require scanning many records   |
| HNSW                 | Approximate graph search with an in-memory hot graph          | Measure graph construction, memory, cache and recall           |
| DISKANN              | Approximate graph stored in the KV backend with bounded cache | Measure KV I/O, cache behavior and recall; available since 3.1 |

SurrealDB supports index configurations for dimensions, distance metrics, element types and construction/search parameters.

The optimizer does not route every query with a distance function to an ANN index. In particular, specifying an explicit distance form in the KNN operator can select brute-force execution instead of using the configured index.

Therefore, an index's existence is not evidence that the query uses it. Query plans and comparative accuracy tests are essential.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes`

### 10.2 SurrealDB 3.3 prefiltered vector search

A substantial new diagnostic surface is the combination of vector nearest-neighbor search with an indexed filter.

When the ordinary predicate can be resolved into an exact record allow-list, the KNN executor can incorporate it before or during vector search.

Eligible predicates include typed scalar equality/range conditions backed by an appropriate standard or unique index, and supported full-text index matches. `NOT` and other ineligible predicates still require later evaluation. Queries using `WITH` or `VERSION` do not use this prefilter path.

`EXPLAIN ANALYZE` exposes the selected `KnnScan` `prefilter_tier`:

| Tier              | Default selection condition              | Mechanism                                               |
| ----------------- | ---------------------------------------- | ------------------------------------------------------- |
| `exact`           | Up to 2,000 matching records             | Exact distance computation on the allow-list            |
| `graph`           | Up to 100,000                            | Filtered ANN with increased search effort               |
| `graph_unboosted` | More than 100,000                        | Filtered ANN at ordinary search effort                  |
| `fallback`        | Allow-list cannot be built within budget | Candidate search followed by normal predicate filtering |

Configuration includes `SURREAL_KNN_PREFILTER_EXACT_THRESHOLD`, `SURREAL_KNN_PREFILTER_EF_BOOST_THRESHOLD`, `SURREAL_KNN_PREFILTER_EF_BOOST`, `SURREAL_KNN_PREFILTER_EF_MAX`, `SURREAL_KNN_PREFILTER_ENABLED`, and `SURREAL_BITMAP_BRANCH_BUDGET`.

The default thresholds are tunable implementation settings, not universal optimal values.

Version-specific caveat: an index created under SurrealDB 3.2 can continue functioning under 3.3 while failing to participate in the new prefilter mechanism. Both the vector index and the filter index need to be created or rebuilt under 3.3 for this optimization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/learn/data-models/vector-search/similarity-search`

### 10.3 Full-text indexes, analyzers and rank fusion

SurrealDB's full-text subsystem combines:

- `DEFINE ANALYZER` tokenization and filtering.
- Full-text indexes using BM25.
- Full-text matching and score/highlight extraction.
- Vector proximity queries.
- Ranking fusion, including `search::rrf`.

This can keep full-text and vector retrieval inside one database and make it possible to combine those results with graph traversals and transactional records.

However, the retrieval approaches differ:

- Full-text scoring measures lexical relevance.
- ANN measures proximity under a vector metric and may be approximate.
- Reciprocal-rank fusion combines ranks; it does not magically calibrate the underlying scores or guarantee vector recall.
- Graph expansion adds relational context and can increase fanout or alter relevance.

For diagnosis, examine each retrieval stage independently before judging the combined ranking. A missing result may be due to tokenization, indexing, approximate recall, prefilter conditions or the fusion policy, rather than a generic query failure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search`

### 10.4 Query plans and index maintenance as diagnostic interfaces

The central query-diagnostic statement is:

```
EXPLAIN ANALYZE FORMAT JSON
SELECT * FROM person WHERE age >= 30;
```

The plan exposes operators, row and batch counts, execution timing and operator-specific attributes.

Other particularly useful commands are:

```
INFO FOR DB;
INFO FOR TABLE person;
INFO FOR INDEX idx_age ON person;

REBUILD INDEX idx_age ON person CONCURRENTLY;
```

`INFO FOR INDEX` reports index-build states such as `started`, `initial indexing`, `updated indexing`, `built` and `error`.

A concurrent rebuild returns before completion; subsequent inspection is needed to establish that the resulting index is ready.

SurrealDB's own `EXPLAIN` reference explicitly warns that its output is informational and may change. JSON output is machine-readable, but should not be treated as an indefinitely stable public schema without version-aware adaptation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Primary sources:

- `https://surrealdb.com/docs/reference/query-language/statements/explain`
- `https://surrealdb.com/docs/reference/query-language/statements/info`
- `https://surrealdb.com/docs/reference/query-language/statements/rebuild`

## 11. Observability, performance diagnostics and runtime inspection

SurrealDB 3.1 introduced a unified observability surface covering metrics, logs and distributed traces. In 3.3, this has become one of the most useful ways to distinguish query, transaction, network, authentication, MCP and storage-engine problems.

### 11.1 Prometheus and OpenTelemetry

There are two independent signal-export paths:

- Prometheus pull: `GET /metrics`, available by default.
- OpenTelemetry push: OTLP export when `SURREAL_TELEMETRY_PROVIDER=otlp`.

They can operate simultaneously. Community Edition includes the principal query, storage and request metrics. Enterprise adds separate durable audit and slow-query pipelines and distributed-storage cluster metrics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The most operationally useful metric families include:

| Metric family                   | What it helps diagnose                                   |
| ------------------------------- | -------------------------------------------------------- |
| `surrealdb.statement.*`         | Statement latency, result volume and errors              |
| `surrealdb.query.*`             | Overall query execution volume and latency               |
| `surrealdb.transaction.*`       | Transaction latency, errors, KV operations and conflicts |
| `surrealdb.rpc.*`               | RPC latency and method-level failures                    |
| `surrealdb.http.*`              | HTTP requests, throughput and concurrency                |
| `surrealdb.session.*`           | Active protocol sessions                                 |
| `surrealdb.network.*`           | Network data movement                                    |
| `surrealdb.auth.*`              | Authentication activity and failure patterns             |
| `surrealdb.live_query.*`        | Active subscriptions and their behavior                  |
| `surrealdb.graphql.*`           | GraphQL operations and latency                           |
| `surrealdb.mcp.*`               | MCP tool invocation rates, durations and errors          |
| `surrealdb.storage.<backend>.*` | Backend-specific cache, storage and compaction signals   |
| `surrealdb.slow_query.*`        | Statements exceeding configured thresholds               |

The storage metric set is backend-specific, so the precise gauge names and availability must be determined from the active binary's `/metrics` output.

The transaction family is particularly valuable. It exposes KV read/write operation counts and bytes, and `surrealdb_transaction_conflicts_total` provides a signal of contention and retry pressure.

Prometheus metric names differ from the dotted OpenTelemetry names. For example, `surrealdb.statement.duration` is exposed as `surrealdb_statement_duration_seconds`. Instrument names were reorganized in 3.1, so dashboards written for 3.0 require a metric-name migration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/manage/observability/metrics`

### 11.2 Metrics security and multi-tenant boundaries

The `/metrics` endpoint is not an unrestricted public diagnostic API.

Anonymous scrapers receive only a small public allowlist. Root authentication provides access to the full operator surface, including more sensitive per-user, per-namespace, storage, authentication and RPC metrics.

Consequently, exposing operator-grade metrics to unrelated consumers can disclose tenancy and workload information. In multi-tenant environments, isolating and filtering the metrics endpoint is part of the security boundary, not just an infrastructure convenience.

Useful settings include:

```
SURREAL_METRICS_ENABLED=true
SURREAL_TELEMETRY_PROVIDER=otlp
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
SURREAL_SLOW_QUERY_METRIC_THRESHOLD_MS=1000
```

These controls are independent: turning off OTLP does not remove the Prometheus endpoint, and disabling metrics push does not necessarily disable tracing or logging.

Direct source: `https://surrealdb.com/docs/manage/observability/configuration`

### 11.3 Logs, slow queries and audit trails

SurrealDB's ordinary logging supports configurable levels, text/JSON formatting, file output, rotation and socket output.

```
surreal db start \
  --log debug \
  --log-format json \
  --slow-log-threshold 500ms \
  rocksdb://./data
```

The Community server's line-based slow-query logging must be distinguished from the Enterprise durable slow-query pipeline.

The Enterprise pipeline supports NDJSON storage, rotation, redaction, optional hash chaining and optional OTLP log export. Enterprise audit logs provide a separate administrative/compliance signal.

For latency analysis, the three complementary views are:

| View                       | Granularity                                                          |
| -------------------------- | -------------------------------------------------------------------- |
| Statement-duration metrics | Aggregated statistical behavior                                      |
| Slow-query logs            | Individual expensive statements and context                          |
| `EXPLAIN ANALYZE`          | Execution structure and operator behavior for a particular statement |

OTel traces add context spanning request processing and async execution. None alone substitutes for the others.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Primary sources:

- `https://surrealdb.com/docs/manage/observability/logging`
- `https://surrealdb.com/docs/manage/observability/slow-query-logging`

### 11.4 Tokio console — Rust async-runtime diagnosis

SurrealDB provides opt-in support for `tokio-console`, a Rust-native terminal UI for inspecting runtime scheduling.

```
cargo install --locked tokio-console

SURREAL_TOKIO_CONSOLE_ENABLED=true \
  surreal db start --user root --pass secret memory

# In another terminal
tokio-console
```

By default, the console connects on `127.0.0.1:6669`.

Its diagnostics include task counts, poll durations, task lifetime and time spent waiting versus actively executing. It is useful for investigating symptoms such as slow async scheduling, tasks that are rarely polled, long-running work, starvation and suspected deadlocks.

This is a runtime-level view, not a SurrealQL query planner. It may explain why work is not being scheduled effectively even when an individual query's logical plan is reasonable.

Because this feature exposes internal task information and adds instrumentation overhead, upstream documentation recommends enabling it on trusted development or staging instances rather than assuming it should run continuously in production.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/manage/observability/tokio-console`

### 11.5 Readiness, health and startup diagnostics

SurrealDB distinguishes server process availability from readiness to serve the database.

A process can be accepting network connections while its datastore is initializing, schema operations are running, or startup imports have not completed.

Relevant interfaces are the HTTP health/readiness routes, notably `/health` and `/ready`, and `surreal db is-ready`.

The dedicated readiness behavior introduced in 3.2 is particularly useful for automation and Kubernetes probes: treating a bound listener as proof of operational readiness can incorrectly send traffic to an incompletely initialized database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



## 12. Rust extensibility, database functions and WebAssembly

### 12.1 Surrealism: first-party Rust/WASM extensions

Surrealism allows custom database functions to be implemented in Rust, compiled into a WASM component, then invoked from SurrealQL.

Its artifacts are `.surli` archives rather than native shared libraries.

The toolchain includes:

```
surreal db module init my-module
surreal db module build my-module
surreal db module info my-module.surli
surreal db module sig my-module.surli --fnc my_function
```

The supported development and runtime capabilities include:

- Exporting functions with `#[surrealism]`.
- Metadata attributes such as `writeable` and `comment`.
- Namespaced function exports.
- One-time module initialization.
- Async functions.
- Optional read-only assets included in the archive.
- Execution-time, memory and module-KV resource limits.
- Module signature and metadata inspection without invoking the code.

The build target is WASI Preview 2 (`wasm32-wasip2`), using the WebAssembly component model. The engine executes these through its Surrealism ABI rather than loading arbitrary native Rust code into the SurrealDB process.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



The runtime uses pooled module controllers, with `SURREAL_SURREALISM_MAX_POOL_SIZE` controlling the ceiling. Module loading is eager by default, while `--lazy-surrealism` or `SURREAL_LAZY_SURREALISM=true` can defer compilation/loading to the first use.

This creates a clear tradeoff between startup latency and first-call latency. It also means that extension performance depends on the WASM execution boundary, module initialization, ABI serialization and host calls—not simply the speed of equivalent native Rust code.

Surrealism remains experimental in SurrealDB 3.3; its production compatibility and security assumptions should be evaluated independently of stable SurrealQL built-ins.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### 12.2 Module installation and lifecycle

A compiled `.surli` archive is made available through SurrealDB bucket storage, and `DEFINE MODULE` registers the exported functions with the database.

This separates artifact distribution from schema-level module registration.

A precompiled module can be inspected with `module info` and `module sig` before it is uploaded. After an archive changes, deployment tooling must update the corresponding module definition or artifact registration so the running database uses the intended code.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The future standalone `surreal mod` component has not yet been published. `surreal db module` is the usable CLI interface as of this research date.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Primary sources:

- `https://surrealdb.com/docs/learn/extensions`
- `https://surrealdb.com/docs/learn/extensions/guides/module-architecture`
- `https://surrealdb.com/docs/learn/extensions/guides/using-wasm-plugins`
- `https://surrealdb.com/docs/reference/cli/db/commands/module`

### 12.3 Other extension mechanisms: functions, events, materialized views and APIs

Surrealism is not the only extensibility layer. SurrealDB also supports server-resident functionality expressed in SurrealQL.

| Mechanism                   | Role                                                        |
| --------------------------- | ----------------------------------------------------------- |
| `DEFINE FUNCTION`           | Reusable SurrealQL business/domain operations               |
| `DEFINE EVENT`              | Database-triggered operations on changes                    |
| `AS SELECT` views           | Derived, maintained data representations                    |
| `DEFINE API`                | Custom HTTP endpoints backed by SurrealQL                   |
| API middleware              | Request processing, policies and reusable endpoint behavior |
| `DEFINE MODULE`             | Register WASM extension functions                           |
| `eval::surql` / `eval::gql` | Capability-gated dynamic query evaluation                   |

This distinction is valuable for limiting extension complexity. A database-native function, event or view can be sufficient without a separate WASM build and deployment process.

`eval::*` is a more powerful but more sensitive mechanism because it evaluates query strings dynamically. SurrealDB provides an explicit deny-by-default capability for this rather than granting it implicitly under broad capability flags.

`DEFINE API` can also move selected application-facing HTTP operations into SurrealDB itself; it should not be interpreted as providing unrestricted native web-server middleware or a general Rust HTTP framework.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### 12.4 Buckets, external object storage and SurrealML

`DEFINE BUCKET` supports database-managed file objects. Depending on configuration and version, backends include memory, local filesystem and external object storage such as S3, Google Cloud Storage and Azure Blob Storage.

This is particularly relevant for extension archives, model artifacts and larger binary data that should not be treated as ordinary embedded document fields.

Bucket storage is a separate file/object layer, not an interchangeable transactional KV backend. The exact external-bucket feature gates and server capabilities should be checked against the deployed 3.3 binary.

SurrealML adds model import/export through `surreal db ml`, using the `.surml` artifact format. This is distinct from Surrealism's general Rust/WASM function-extension system. It provides a model lifecycle interface rather than a general-purpose runtime for arbitrary Rust crates.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Primary sources:

- `https://surrealdb.com/docs/reference/cli/db/commands/ml`
- `https://surrealdb.com/docs/reference/query-language/statements/define/bucket`

## 13. Protocol adapters and the external tooling they enable

The core Rust SDK is normally the most complete native application interface, but SurrealDB 3.3 adds several protocol choices that can extend operational tooling.

| Interface                    | Supported purpose                                                           | Material limitations                                     |
| ---------------------------- | --------------------------------------------------------------------------- | -------------------------------------------------------- |
| Rust SDK over WebSocket/HTTP | Typed application access, queries, authentication, live subscriptions       | Network transport and connection lifetime                |
| Native embedded engine       | In-process queries and transactions                                         | Engine feature flags, local file ownership               |
| gRPC over HTTP/2             | RPC and streaming execution, including progressive result delivery          | Requires compatible SDK/protocol support                 |
| PostgreSQL wire protocol     | `psql`, `tokio-postgres`, prepared requests and familiar connection tooling | SurrealQL/GQL dialects, not PostgreSQL SQL compatibility |
| REST `/sql`                  | Straightforward HTTP execution and scripting                                | Request/response rather than a full SDK abstraction      |
| `/gql`                       | ISO graph-pattern queries                                                   | ISO GQL subset; different from GraphQL                   |
| `/graphql`                   | Generated typed GraphQL API                                                 | Configured schema and GraphQL-specific semantics         |
| `/mcp`                       | Agent-controlled database access                                            | Tool policies, authorization and result limits           |

### 13.1 PostgreSQL wire compatibility

This is particularly valuable for Rust tooling because a `tokio-postgres` client can now communicate with a SurrealDB server.

The 3.3 listener supports the PostgreSQL v3 protocol, including simple and extended queries, positional parameters, prepared statements, cancellation, interactive transactions, and optional TLS.

For example:

```
surreal db start \
  --user root --pass secret \
  --postgres-bind 127.0.0.1:5432 \
  memory
```

But PostgreSQL wire compatibility is not PostgreSQL dialect compatibility.

At the research date, important unsupported features include ANSI SQL translation, `pg_catalog` emulation, `COPY`, live queries over the Postgres connection and GQL statements inside an open interactive transaction.

These gaps are consequential for SQLx compile-time introspection, ordinary PostgreSQL ORMs, automatic schema browsers, and clients that assume `pg_catalog` is available. They can connect at the protocol level without necessarily functioning as database-management tools against SurrealDB.

Results also cross a type-conversion boundary: record IDs may arrive as text and nested values as `jsonb`, while prepared result schemas may use a generic JSON representation when exact static typing is unavailable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct source: `https://surrealdb.com/docs/reference/rest-api/postgres-protocol`

### 13.2 gRPC streaming

SurrealDB 3.3 introduces gRPC transport over HTTP/2 and streaming query results. This is useful for Rust clients processing large result sets incrementally and for applications where transport behavior itself affects throughput or memory usage.

The distinction between gRPC streaming and live subscriptions remains important: query-result streaming delivers the result of a particular query progressively, whereas live queries observe subsequent changes.

Streaming also creates partial-result obligations. A diagnostic consumer should not assume that receipt of some rows means the statement ultimately completed successfully.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Direct source: `https://surrealdb.com/docs/reference/rest-api`

## 14. Bulk data movement, migration and change-data capture

### 14.1 Surreal Sync — Rust-based migration and ingestion

`surreal-sync` is the official Rust implementation for moving data from external sources into SurrealDB. Its role is different from both logical database backup and schema migration.

It supports multiple ingestion mechanisms:

| Source          | Data movement mechanism                                                                            |
| --------------- | -------------------------------------------------------------------------------------------------- |
| PostgreSQL      | Initial snapshot plus logical replication via `pgoutput` or `wal2json`; trigger-based alternatives |
| MySQL / MariaDB | Initial copy plus binlog or trigger-based incremental capture                                      |
| MongoDB         | Initial import plus change streams                                                                 |
| Neo4j           | Initial import with timestamp-based incremental tracking                                           |
| SQL Server      | Snapshot and native CDC                                                                            |
| Kafka           | Topic consumption with optional deduplication                                                      |
| Snowflake       | Snapshot ingestion                                                                                 |
| JSONL           | File-based bulk ingestion                                                                          |

Its architecture includes source adapters, transformations, sink application, checkpoints and watermarks.

This distinction is important: replication does not automatically imply that source and target schemas, relationship representations, update ordering and deletion semantics have identical meanings. For example, Neo4j's timestamp-based incremental tracking is not the same mechanism as PostgreSQL's WAL-based CDC.

The tool has source-specific documentation and a transformation configuration layer that can reduce the need to construct independent ingestion pipelines for common databases.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

surrealdb/surreal-sync · GitHub



Direct source: `https://github.com/surrealdb/surreal-sync`

### 14.2 Three independent migration problems

It is useful to distinguish the major migration toolchains by what they actually change.

| Migration target                                          | Appropriate mechanism                                                                      |
| --------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Evolve tables, fields, indexes, permissions and functions | SurrealKit sync / rollout                                                                  |
| Move data from an external database into SurrealDB        | Surreal Sync                                                                               |
| Copy a SurrealDB namespace/database logically             | `surreal db export/import`                                                                 |
| Move between storage-engine file formats                  | Engine-specific migration tools, including SurrealKV compatibility utilities               |
| Migrate older SurrealDB internal database formats         | Version-specific SurrealDB upgrade/migration instructions                                  |
| Recover from a damaged LSM file                           | Offline storage-engine checks, backup restoration and engine-specific repair investigation |

These operations preserve different levels of information. A successful logical export does not demonstrate that the source's storage-level history has been preserved; successful storage conversion does not demonstrate that every application-level schema migration has been applied.

### 14.3 SurrealKit's migration-parity testing

A notable capability in the SurrealKit 1.0 development line is testing schema creation from both desired-state sync and replayed migrations.

The runner can execute suites under:

- `schema_from = "sync"`
- `schema_from = "rollouts"`
- `schema_from = "both"`

When rollouts are replayed, SurrealKit can compare the resulting schema against one built using sync and fail when definitions diverge.

This is a powerful correctness mechanism: it tests not just whether the current schema works, but whether the migration history actually recreates that schema. It also makes it possible to test access rules and APIs against the database state produced by the same rollout sequence as a deployed environment.

This functionality belongs to the documented 1.0 development line, rather than being safely attributed to every stable 0.7 installation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Direct source: `https://github.com/surrealdb/surrealkit`

## 15. Benchmarking, correctness testing and engine development tools

### 15.1 `crud-bench` — the broader Rust benchmarking harness

`crud-bench` is especially relevant because it supports SurrealDB, SurrealMX and SurrealKV as separate benchmark targets, alongside RocksDB and other database engines.

It is not restricted to measuring generic CRUD operations.

The current upstream harness supports or documents workload families covering:

- Single-record and batch CRUD.
- Full-table and range scans.
- Predicate filtering and index lookups.
- Structured document sizes and record-key distributions.
- Configurable concurrency, clients, threads and dataset sizes.
- Full-text search.
- Vector index construction and querying.
- Graph relationship traversal and multi-level graph workloads.

Its more advanced vector-search measurement facilities include:

- Brute-force exact ground-truth comparisons.
- Recall\@k.
- HNSW and DISKANN parameter sweeps.
- Recall/latency curves.
- Filtered KNN across different predicate selectivities.
- Vector index memory and disk footprint.
- Reproducible generated corpora.

Those capabilities make it more informative than a single throughput test when analyzing the tradeoff between approximate search quality, indexing cost and query latency.

The CLI also has useful operational options:

```
cargo run -r -- --database surrealdb --samples 100000
cargo run -r -- --database surrealmx --samples 100000
cargo run -r -- --database surrealkv --samples 100000

cargo run -r -- \
  --database rocksdb \
  --samples 100000 \
  --sync \
  --store-results
```

The `--sync` option is important when comparing persistent backends because durability settings can dominate write throughput. It would be misleading to compare an engine flushing synchronously with another configured to acknowledge writes before durable persistence.

Benchmark parameters, generated data, results and database configuration must all be considered together. The harness also supports storing results in SurrealDB and comparing result JSON files through its browser visualization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Direct source: `https://github.com/surrealdb/crud-bench`

### 15.2 Upstream SurrealQL language tests and backend-specific benchmarks

The main SurrealDB repository includes its own language-testing infrastructure.

It runs `.surql` language tests against different engine backends, including memory, SurrealKV, RocksDB and TiKV when the corresponding compile-time features or external services are available.

The upstream test infrastructure also includes dedicated benchmarking support for execution against different backends, and Rust Criterion benchmarks for engine and SDK operations.

Its value is semantic regression coverage across the database implementations—not just performance measurement.

One limitation is explicit in SurrealDB's documentation: the language test suite is an internal developer facility, and its output can differ from that produced through a running server. It is useful source material for advanced engine investigation but is not presented as a stable community-facing testing API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Direct sources:

- `https://github.com/surrealdb/surrealdb/tree/main/language-tests`
- `https://github.com/surrealdb/surrealdb/tree/main/surrealdb/benches`
- `https://surrealdb.com/docs/learn/querying/concepts-and-guides/testing`

### 15.3 Correctness testing within SurrealQL

SurrealQL itself provides several useful primitives for database-side assertions:

- Strong field kinds and schema constraints.
- `ASSERT` and custom validation expressions.
- Explicit error generation with `THROW`.
- `value::expect` and `.expect()` for asserting intermediate values.
- Manual transactions that can be deliberately rolled back.
- Unique and relational-integrity constraints.
- Permission definitions that execute inside the database.

This can eliminate the need to duplicate some correctness rules in Rust.

The distinction is between enforcing invariants and testing that those invariants are correct. Schema and permission constraints provide enforcement; SurrealKit or Rust integration tests can verify the policy under relevant callers and transaction conditions.

A particularly useful pattern is testing database-side permission behavior as the restricted actor, rather than testing only as a privileged developer connection.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### 15.4 Supporting Rust packages

Two adjacent crates deserve attention.

`revision`, maintained by SurrealDB, implements revision-tolerant Rust serialization and deserialization with support for evolving structured formats. It is principally relevant to low-level persistence, compatibility and internal serialization design. It is not a SurrealQL database migration tool.

Source: `https://github.com/surrealdb/revision`

`somnia`, a community Rust ORM/query-builder project, provides a `SurrealRecord` derive macro, typed query expressions, graph traversal construction, schema generation and migration support. It offers a model-driven alternative to manually assembled SurrealQL strings.

Its author documents literal inlining with escaping rather than exclusive reliance on bound query parameters, which is a consequential implementation distinction. Its compatibility with the precise SurrealDB 3.3 feature set and long-term API stability should be verified independently. It is not an official SurrealDB SDK component.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: `https://github.com/vbasky/somnia`

## 16. Visual tooling, deployment and adjacent integrations

### 16.1 SurrealDB Studio

SurrealDB Studio is the current first-party visual workbench, covering much of the territory historically associated with Surrealist.

Its capabilities include:

- A multi-pane query editor with query history and language-aware completion.
- Results rendered as tables, lists, graphs or live feeds.
- Interactive graph exploration, including expanding connected nodes.
- Schema graph exploration and editing of fields, indexes, permissions and events.
- Browsing of tables, records, functions, analyzers and API endpoints.
- User and access-method inspection.
- Cloud instance management, diagnostics and Agent Memory context views.
- A browser-based WASM database sandbox, with optional IndexedDB persistence depending on the mode.

Studio is therefore broader than a graphical SQL shell. It combines database authoring, schema exploration, graph visualization and operational tooling in one workspace.

It is not Rust-native as a development application, but it integrates with a Rust-based deployment and can make many diagnostic relationships easier to inspect visually.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Your database and memory, visualised



Direct sources:

- `https://surrealdb.com/studio`
- `https://surrealdb.com/docs/explore/studio`

### 16.2 Container, Kubernetes and deployment tooling

The supporting operations stack includes the official container image, Helm charts, Kubernetes manifests, readiness probes and the Cloud control plane.

For single-node persistent RocksDB deployments, upstream Kubernetes instructions use one pod with a persistent volume and explicitly warn against running multiple pods against the same database directory.

The Enterprise distributed deployment has a different architecture and supporting tooling, including distributed-storage monitoring and replication-aware operations. The presence of Kubernetes replicas does not convert a local RocksDB directory into distributed storage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Direct sources:

- `https://surrealdb.com/docs/manage/self-hosted/kubernetes`
- `https://surrealdb.com/docs/manage/self-hosted/managed-kubernetes`
- `https://helm.surrealdb.com`

### 16.3 Official machine-consumable documentation for programming agents

SurrealDB's reference documentation is directly usable by programming agents without requiring them to scrape the visual documentation site.

Official support includes:

- Markdown content by appending `.md` to a documentation page.
- Markdown content negotiation using `Accept: text/markdown`.
- A documentation index in `llms.txt`.
- An aggregated reference corpus in `llms-full.txt`.
- The generated SurrealQL function metadata from the language-server project.
- Official agent skills for specific SurrealDB domains.

Together, these can provide query-language definitions, typed function catalogues, integration guidance and version-specific operator documentation without maintaining another handwritten copy.

A limitation remains: dynamically served documentation may advance ahead of a pinned database release. For exact compatibility, tagged upstream source and version-specific release notes remain the stronger evidence.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Direct sources:

- `https://surrealdb.com/docs/llms.txt`
- `https://surrealdb.com/docs/llms-full.txt`
- `https://github.com/surrealdb/surrealql-language-server/blob/master/builtins.json`
- `https://github.com/surrealdb/agent-skills`

### 16.4 Adjacent ecosystem products that are not Rust-native tooling

A few neighboring projects are still useful to recognize, while keeping their implementation boundaries clear.

SurrealFS provides a database-backed virtual filesystem for agents. It represents files and folders in SurrealDB, exposing filesystem operations plus full-text and semantic search. Its published implementation and framework adapters are Python-oriented rather than a Rust filesystem library. It is therefore an optional integration concept, not a Rust-native SurrealDB component.

Source: `https://github.com/surrealdb/surrealfs`

SurrealDB Agent Memory provides a separate knowledge/memory layer with an MCP server, REST API, CLI and managed contexts. Its APIs can be used from Rust through standard protocols, but it should not be mistaken for an embeddable Rust module inside SurrealDB.

Airbyte, Fivetran, Dagster and n8n represent further data-pipeline and automation integration paths. They are useful when interacting with systems outside the Rust implementation, but they do not alter SurrealDB's native transaction or storage guarantees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Agent Memory

+1



## 17. Diagnostic routing: which tool answers which question?

A practical way to understand the ecosystem is by identifying the lowest layer that can explain a symptom without losing necessary context.

| Problem being investigated                                  | Primary diagnostic interface                      | Secondary or lower-level tooling                           |
| ----------------------------------------------------------- | ------------------------------------------------- | ---------------------------------------------------------- |
| SurrealQL fails to parse                                    | `surreal lsp check`, `surreal db validate`        | Rust parser/AST internals                                  |
| A query returns unexpected data                             | SurrealDB Studio / Rust SDK / CLI                 | Schema `INFO`, actor-specific tests                        |
| A query is unexpectedly slow                                | `EXPLAIN ANALYZE FORMAT JSON`                     | Statement metrics, traces, storage metrics                 |
| Graph traversal has excessive reads                         | `GraphEdgeScan`, `ReferenceScan`                  | Inline cache hits, graph-folding counters, backend metrics |
| Filtered vector search underperforms                        | `KnnScan` `prefilter_tier`                        | `INFO FOR INDEX`, ANN benchmark recall/latency             |
| Full-text search has missing matches                        | Analyzer/index definitions                        | Query plans, tokenization and data inspection              |
| Queries conflict under concurrency                          | Transaction conflict counters                     | Tokio console, backend transaction instrumentation         |
| Database latency is high with low query CPU                 | Storage backend gauges                            | `skv`, RocksDB tools, host I/O profiling                   |
| Server appears hung                                         | `/ready`, logs, process metrics                   | Tokio console and Rust CPU profiling                       |
| Embedded database cannot open                               | Server/SDK errors, process ownership              | Engine-specific file-lock and recovery diagnostics         |
| An index was rebuilt but is not used                        | `INFO FOR INDEX`, `EXPLAIN`                       | Index definition and version checks                        |
| MCP agent cannot execute a query                            | MCP tool error, permissions and capability policy | MCP timeout/output limits, auth metrics                    |
| Migration or rollout was interrupted                        | SurrealKit `rollout status`                       | `rollout repair`, migration metadata                       |
| Logical restore is incomplete                               | SurrealQL export/import verification              | Storage backup, historical-version limitations             |
| RocksDB corruption is suspected                             | Controlled offline engine inspection              | `ldb`, `sst_dump`, engine recovery tools                   |
| SurrealKV corruption is suspected                           | `skv scrub`, `manifest`, `sst`                    | Physical backup and engine compatibility investigation     |
| WASM module fails or is slow                                | Module CLI metadata and logs                      | WASM runtime limits, pooling, tracing                      |
| Rust SDK returns an apparent success with failed statements | `.check()` / `.take_errors()`                     | Statement-level error details                              |
| CDC migration is incomplete                                 | Surreal Sync checkpoints and source-specific logs | Source WAL, change streams, binlogs, target verification   |

The important diagnostic separation is:

- Query correctness belongs primarily at the SurrealQL/schema layer.
- Execution efficiency belongs at the query planner and operator layer.
- Runtime scheduling and contention belong at the transaction and Tokio layers.
- Physical I/O, durability and file integrity belong to the storage engine.
- Agent failures may also be caused by MCP permission, result-size or context-management boundaries.
- Lifecycle failures may belong to SurrealKit's migration state rather than the database itself.

Escalating immediately to physical KV inspection for a logical-query problem risks losing context. Conversely, analyzing SurrealQL plans cannot fully explain RocksDB compaction stalls or SurrealKV file corruption.

## 18. Release maturity and compatibility considerations

The major version boundaries from the research are worth preserving together.

| Capability or tooling                             | October 2026 status                                                          |
| ------------------------------------------------- | ---------------------------------------------------------------------------- |
| SurrealDB server and Rust SDK                     | Stable 3.3.4                                                                 |
| RocksDB backend                                   | Supported and recommended persistent backend for 3.3 single-node deployments |
| SurrealKV backend in SurrealDB 3.3                | Supported with its documented beta/deployment qualifications                 |
| Standalone SurrealKV crate                        | 1.0 released; not equivalent to its 3.3 integrated dependency                |
| Standalone SurrealMX crate                        | 1.0 released; not equivalent to its 3.3 integrated dependency                |
| Embedded database MCP                             | Released since 3.1                                                           |
| ISO GQL                                           | Enabled by default in 3.3, previously experimental in 3.2                    |
| PostgreSQL wire protocol                          | Available in 3.3; ANSI SQL translation not yet available                     |
| gRPC streaming                                    | Available in 3.3                                                             |
| Lightweight graph relations and inline adjacency  | Available in 3.3                                                             |
| Vector-search prefiltering                        | Available in 3.3; index rebuild conditions matter                            |
| Surrealism Rust/WASM extensions                   | Available but experimental                                                   |
| `surreal db module`                               | Available                                                                    |
| Standalone `surreal mod` component                | Not yet published                                                            |
| SurrealKit 0.7                                    | Stable tooling line                                                          |
| SurrealKit 1.0                                    | Beta/development line during research                                        |
| SurrealDB Studio                                  | Available                                                                    |
| SurrealQL Language Server                         | Available as Rust binary / CLI component                                     |
| `surreal db fix` under 3.x                        | Command exists, but repair functionality is not implemented                  |
| Enterprise audit and durable slow-query pipelines | Edition-specific, available from 3.1                                         |

The most consequential unresolved integration question is not whether SurrealMX 1.0 or SurrealKV 1.0 exists. It is which versions the exact SurrealDB build resolves and which APIs or on-disk formats that integration exposes. Treat the standalone engines and the database-integrated backends as independently versioned products until upstream integration is explicitly confirmed.

## 19. Consolidated primary-source resource directory

The following references cover most of the Rust-oriented tooling surface without depending on a particular repository or deployment.

| Topic                             | Direct URL                                                                     |
| --------------------------------- | ------------------------------------------------------------------------------ |
| Database source and crates        | `https://github.com/surrealdb/surrealdb`                                       |
| Version-specific 3.3.4 source     | `https://github.com/surrealdb/surrealdb/tree/v3.3.4`                           |
| Rust SDK API                      | `https://docs.rs/surrealdb/`                                                   |
| Rust SDK concepts                 | `https://surrealdb.com/docs/reference/rust`                                    |
| CLI launcher and commands         | `https://surrealdb.com/docs/reference/cli`                                     |
| SurrealKit source                 | `https://github.com/surrealdb/surrealkit`                                      |
| Schema migration documentation    | `https://surrealdb.com/docs/manage/schema-migration`                           |
| Surreal Sync                      | `https://github.com/surrealdb/surreal-sync`                                    |
| SurrealMX                         | `https://github.com/surrealdb/surrealmx`                                       |
| SurrealKV                         | `https://github.com/surrealdb/surrealkv`                                       |
| RocksDB                           | `https://github.com/facebook/rocksdb`                                          |
| SurrealDB RocksDB fork            | `https://github.com/surrealdb/rocksdb`                                         |
| Rust language server              | `https://github.com/surrealdb/surrealql-language-server`                       |
| Tree-sitter grammar               | `https://github.com/surrealdb/surrealql-tree-sitter`                           |
| Database MCP                      | `https://surrealdb.com/docs/build/ai-agents/mcp/embedded`                      |
| Cloud MCP                         | `https://surrealdb.com/docs/build/ai-agents/mcp`                               |
| Agent Memory APIs and CLI         | `https://surrealdb.com/docs/agent-memory/reference`                            |
| Query-plan diagnostics            | `https://surrealdb.com/docs/reference/query-language/statements/explain`       |
| Vector index mechanisms           | `https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes`    |
| Prefiltered vector search         | `https://surrealdb.com/docs/learn/data-models/vector-search/similarity-search` |
| Graph storage and relation types  | `https://surrealdb.com/docs/reference/query-language/statements/define/table`  |
| ISO GQL                           | `https://surrealdb.com/docs/learn/querying/gql/overview`                       |
| Observability and metrics         | `https://surrealdb.com/docs/manage/observability`                              |
| Complete metric catalogue         | `https://surrealdb.com/docs/manage/observability/metrics`                      |
| Tokio console integration         | `https://surrealdb.com/docs/manage/observability/tokio-console`                |
| Rust/WASM extensions              | `https://surrealdb.com/docs/learn/extensions`                                  |
| PostgreSQL protocol compatibility | `https://surrealdb.com/docs/reference/rest-api/postgres-protocol`              |
| Benchmark harness                 | `https://github.com/surrealdb/crud-bench`                                      |
| Studio                            | `https://surrealdb.com/studio`                                                 |
| Official agent skills             | `https://github.com/surrealdb/agent-skills`                                    |
| Revision-aware Rust serialization | `https://github.com/surrealdb/revision`                                        |
| Community Rust ORM/query builder  | `https://github.com/vbasky/somnia`                                             |

### Architectural synthesis

The most important development in SurrealDB's tooling landscape is that advanced functionality increasingly exists inside the database and its own toolchain, rather than requiring separate application-maintained mechanisms.

SurrealKit provides schema lifecycle, migration parity and permissions testing. The language server provides static semantic analysis and a generated function catalogue. SurrealQL supports first-class graph traversal, typed schema constraints, server-side functions and query plans. SurrealDB 3.3 adds physical graph-layout optimization and prefiltered vector execution. OpenTelemetry, Tokio console and storage-engine-specific diagnostic tools expose progressively deeper performance information. MCP makes much of the same database functionality accessible to programming agents.

These capabilities compose most effectively when their distinct responsibilities remain clear. A unified Rust implementation can use the same database semantics for application logic, introspection, tests, migrations and diagnostics, while preserving the independent storage-engine, runtime and protocol mechanisms needed to explain behavior that SurrealQL alone does not expose.

The main cautions are version alignment, experimental extension APIs, differences between standalone and integrated storage engines, and the fact that protocol compatibility is not automatically query-language or data-model compatibility.