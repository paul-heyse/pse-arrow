# SurrealDB CLI Ecosystem — Comprehensive Technical Reference

Research date: November 8, 2026 | Target: SurrealDB 4.0, with version-specific coverage of 3.x and 2.x | Emphasis: Rust-based command-line tools, automation, and advanced CLI capabilities

## 1. Scope and architecture

SurrealDB has a broader command-line tooling ecosystem than the `surreal` executable alone. It includes the official database and Cloud CLIs, Rust-based migration and synchronization tools, SurrealQL analysis tools, storage-engine diagnostics, extension packaging utilities, and testing and benchmarking executables.

This reference covers the functionality accessible through those command-line interfaces: their commands, significant options, automation capabilities, execution semantics, and interactions.

It does not independently document the Rust SDK or general database functionality except where necessary to explain a CLI command, build requirement, or integration boundary.

### 1.1 CLI ecosystem at a glance

| Tool or component         | Command                      | Primary capability                                                            |
| ------------------------- | ---------------------------- | ----------------------------------------------------------------------------- |
| Surreal launcher          | `surreal`                    | Install, select, update, and invoke versioned components                      |
| Database CLI              | `surreal db`                 | Database server, SurrealQL shell, imports/exports, validation, diagnostics    |
| Cloud CLI                 | `surreal cloud`              | Instance provisioning, operations, policies, identity, backups, observability |
| Extension CLI             | `surreal ext`                | Build, execute, inspect, sign, install, and upgrade Rust/WASM extensions      |
| SurrealQL language server | `surreal lsp`, `surreal-lsp` | Semantic checking, static analysis, LSP integration, structured diagnostics   |
| SurrealKit                | `surrealkit`                 | Schema reconciliation, versioned migrations, seeding, tests, type generation  |
| Surreal Sync              | `surreal-sync`               | External database ingestion, CDC, transformations, synchronization            |
| SurrealKV CLI             | `skv`                        | Physical key-value inspection, integrity checks, storage migration            |
| SurrealDB Migrations      | `surrealdb-migrations`       | Community migration generation, application, rollback, inspection             |
| SurrealQL formatter       | `surqlfmt`                   | SurrealQL formatting and CI checks                                            |
| Language-test harness     | Cargo-run executable         | SurrealQL regression and conformance testing                                  |
| CRUD Bench                | `crud-bench`                 | Configurable database workload benchmarking                                   |
| Agent Memory CLI          | `agent-memory`               | CLI for the separate SurrealDB Agent Memory product                           |

The formatter is implemented and distributed through the JavaScript/npm ecosystem, but is included because it forms part of the practical SurrealQL CLI toolchain. Agent Memory is included as a related but independently distributed product, not as a database-administration component.

### 1.2 Distribution and release status

The current stable database release is SurrealDB 4.0.0, released November 3, 2026. The command-line ecosystem does not have one shared release lifecycle.

| Component                         | Verified status                                                                                              |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Database CLI 4.0                  | Stable release                                                                                               |
| Surreal launcher                  | Separately versioned and distributed                                                                         |
| Cloud CLI                         | Separately versioned component                                                                               |
| SurrealQL language server         | Rust repository with native binary, headless CLI, and component-distribution workflow                        |
| SurrealKit                        | Stable 0.7.x line; newer 1.0 beta functionality documented upstream                                          |
| Surreal Sync                      | Under active development; upstream does not present it as stable                                             |
| SurrealKV `skv`                   | Separate Rust CLI distributed as `surrealkv-cli`                                                             |
| Surrealism extension CLI          | 4.0 command surface documented; standalone `ext` component's first publication is still described as pending |
| Language-test and benchmark tools | Upstream development utilities; interfaces are not necessarily stable public contracts                       |

The distinction between documented implementation, released executable, and stable supported behavior matters particularly for SurrealKit, Surreal Sync, and `surreal ext`. Examples involving these tools should be interpreted in that context.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+4



Primary sources:

[https://surrealdb.com/docs/reference/cli](https://surrealdb.com/docs/reference/cli?utm_source=chatgpt.com)

[https://surrealdb.com/releases](https://surrealdb.com/releases?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealdb](https://github.com/surrealdb/surrealdb?utm_source=chatgpt.com)

## 2. The official `surreal` launcher

The modern `surreal` executable is a component and version manager. It is not itself the database server.

Its primary purpose is to resolve, install, update, select, and execute independently versioned tools.

### 2.1 Component-management commands

| Command                | Function                                                                |
| ---------------------- | ----------------------------------------------------------------------- |
| `surreal install`      | Download and install one or more components                             |
| `surreal upgrade`      | Upgrade selected components, or the launcher itself                     |
| `surreal use`          | Set the default installed version of a component                        |
| `surreal uninstall`    | Remove installed component versions                                     |
| `surreal list`         | Inspect installed components, versions, selection, and executable paths |
| `surreal +VERSION ...` | Temporarily select a different component version                        |
| `surreal --version`    | Report launcher version                                                 |
| `surreal help`         | Display launcher command help                                           |

The launcher separates the executable used to manage installations from the executables that implement functionality.

For example:

```
# Install the database and Cloud components
surreal install db@4.0.0 cloud

# Install a second database release
surreal install db@3

# Select the default database version
surreal use db@4.0.0

# Execute a single command using 3.x
surreal +3 db version

# Inspect installed components
surreal list --json

# Upgrade the database, without upgrading Cloud
surreal upgrade db

# Upgrade the launcher itself
surreal upgrade cli
```

### 2.2 Version resolution

A component version selector can identify an exact version, a major version, or a release channel.

| Selector     | Meaning                                         |
| ------------ | ----------------------------------------------- |
| `db@4.0.0`   | Exact release                                   |
| `db@3`       | Latest available stable release on major line 3 |
| `db@latest`  | Latest stable release                           |
| `db@beta`    | Latest available beta                           |
| `db@alpha`   | Latest available alpha                          |
| `db@nightly` | Latest nightly build; database component only   |

Installed versions coexist under the launcher home, with each component's selected default stored separately.

This enables version-specific SQL parsing, migration checks, export operations, and CLI compatibility testing without repeatedly replacing one global database executable.

Important distinction: selecting an older executable does not reverse a datastore-format upgrade. Database binary version, storage format, and client/server compatibility remain separate concerns.

### 2.3 Installation, mirroring, and offline execution

The official installation script supports installing both the launcher and database, or the launcher alone:

```
# Launcher and database
curl -sSf https://install.surrealdb.com | sh

# Launcher only
curl -sSf https://install.surrealdb.com |
  SURREAL_CLI_INSTALL_DB=0 sh

# Install Cloud in the same operation
curl -sSf https://install.surrealdb.com |
  sh -s -- --with cloud
```

Installation automation supports several relevant environment variables.

| Variable                    | Purpose                                            |
| --------------------------- | -------------------------------------------------- |
| `SURREAL_CLI_HOME`          | Override component storage directory               |
| `SURREAL_CLI_OFFLINE=1`     | Forbid network requests or automatic downloads     |
| `SURREAL_CLI_DOWNLOAD_ROOT` | Substitute a component distribution mirror         |
| `SURREAL_CLI_JSON=1`        | Enable structured output where supported           |
| `SURREAL_CLI_PLAIN=1`       | Prefer plain output                                |
| `SURREAL_CLI_NO_BANNER=1`   | Suppress help banner                               |
| `SURREAL_CLI_INSTALL_DB`    | Control whether installation includes the database |

An unavailable component in offline mode produces an explicit failure, with launcher exit code `12`, rather than silently attempting a network fetch.

The launcher also supports `--no-input` for unattended operation, confirmation handling, and upgrade dry runs where documented.

### 2.4 Process dispatch and environment isolation

The launcher forwards arguments after a component name to that component without interpreting them.

The separation is:

- `SURREAL_CLI_*`: launcher configuration.
- `SURREAL_*`: database executable configuration.
- `SURREAL_CLOUD_*`: Cloud CLI configuration.

It additionally sets `SURREAL_INVOKED_AS` and `SURREAL_CLI_LAUNCHER` when launching a component. The latter allows a component to invoke another component through the same launcher, preserving version resolution.

The Cloud CLI uses this mechanism to delegate certain database operations to `surreal db` rather than duplicating query or export functionality.

### 2.5 Legacy command compatibility

Before the launcher architecture, database commands were invoked directly:

```
surreal start
surreal sql
surreal export
```

The modern canonical commands are:

```
surreal db start
surreal db sql
surreal db export
```

The launcher accepts a closed set of legacy database command forms with deprecation warnings. New database commands are not automatically supported through that older syntax.

Container images are an exception worth recognizing: they generally execute the database binary directly, so commands inside the container still use forms such as `start` rather than `surreal db start`.

Primary references:

[https://surrealdb.com/docs/reference/cli/launcher](https://surrealdb.com/docs/reference/cli/launcher?utm_source=chatgpt.com)

[https://surrealdb.com/docs/explore/cli/components](https://surrealdb.com/docs/explore/cli/components?utm_source=chatgpt.com)

[https://surrealdb.com/docs/explore/cli/install](https://surrealdb.com/docs/explore/cli/install?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2

## 3. `surreal db`: database command-line operations

The `db` component is the principal database administration executable. Its responsibilities include starting a database, connecting to an existing instance, executing scripts, moving database contents, validating queries, checking readiness, and running an embedded MCP server.

### 3.1 Complete command surface

| Command         | CLI responsibility                                                                         |
| --------------- | ------------------------------------------------------------------------------------------ |
| `start`         | Start a database server using an in-memory, persistent, or supported distributed datastore |
| `sql`           | Interactive SurrealQL REPL, piped query execution, or direct embedded-database access      |
| `import`        | Execute a SurrealQL import file against a database                                         |
| `export`        | Produce a logical SurrealQL export                                                         |
| `validate`      | Offline parser-based validation of query files                                             |
| `is-ready`      | Check server readiness                                                                     |
| `version`       | Inspect local database or remote server version                                            |
| `mcp`           | Start the built-in MCP server over stdio                                                   |
| `fix`           | Legacy storage-format conversion, operational only with relevant 2.x binaries              |
| `upgrade-check` | 3.3-specific compatibility check before upgrading to 4.0                                   |
| `help`          | Describe available commands and their options                                              |

Older releases also contain `ml` for SurrealML import/export and `module` for WebAssembly module management. SurrealDB 4.0 removes SurrealML and moves module development to the independently managed `ext` component.

The launcher, rather than this executable, handles database binary upgrades through `surreal upgrade db`.

Primary reference: [https://surrealdb.com/docs/reference/cli/db/commands](https://surrealdb.com/docs/reference/cli/db/commands?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 3.2 `surreal db start`: server process configuration

The command has five major control surfaces: storage selection, network listeners, authentication, capability permissions, and runtime maintenance/observability.

```
# In-memory development server
surreal db start \
  --bind 127.0.0.1:8000 \
  --user root \
  --pass secret \
  memory

# Persistent SurrealKV server
surreal db start \
  --bind 127.0.0.1:8000 \
  --user root \
  --pass secret \
  surrealkv://data
```

Storage selection is positional. In SurrealDB 4.0 the generally available choices are `memory` and `surrealkv`; TiKV requires the separate `surreal-tikv` server build. RocksDB and IndexedDB backend schemes are accepted as legacy spellings associated with migration to SurrealKV, not as independent 4.0 engines.

Relative datastore paths are resolved against the process's working directory. This is a significant CLI behavior for systemd, containers, and other service managers, because the same relative argument can identify a different database when the working directory changes.

#### Server option families

| Family              | Significant flags                                                         | CLI functionality                                     |
| ------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------- |
| Storage             | `[PATH]`, `--temporary-directory`                                         | Storage engine/path and temporary files               |
| Network             | `--bind`, `--web-crt`, `--web-key`                                        | Listener configuration and HTTPS                      |
| Alternate protocols | `--postgres-bind`, `--mysql-bind`                                         | Enable PostgreSQL/MySQL wire listeners                |
| Distributed KV      | `--kvs-ca`, `--kvs-crt`, `--kvs-key`                                      | Secure connections to supported remote KV backends    |
| Identity            | `--username`, `--password`, `--unauthenticated`                           | Initial root identity and unauthenticated-access mode |
| Origin and proxy    | `--allow-origin`, `--client-ip`, `--cookie-auth`                          | CORS, forwarded IP handling, cookies                  |
| Timeouts            | `--query-timeout`, `--transaction-timeout`, `--startup-operation-timeout` | Bound query, transaction, and startup operations      |
| Initialization      | `--default-namespace`, `--default-database`, `--no-defaults`              | Control initial namespace/database creation           |
| Startup scripts     | `--import-file`                                                           | Execute a SurrealQL file when starting                |
| Extensions          | `--lazy-surrealism`                                                       | Defer extension loading until first invocation        |
| Observability       | `--log`, `--log-format`, `--slow-log-threshold`                           | Standard and slow-query logs                          |

Root credentials and default namespace/database creation settings apply when initializing a new datastore. Repeating startup with different root credentials does not reset the credentials already persisted in storage.

The `--import-file` flag runs on each startup, not exactly once. It is therefore a script-execution mechanism rather than a migration-history manager.

#### Capability policy controls

One of the more advanced features of the server CLI is its granular permission and capability configuration.

| Capability group   | Main flags                                          | What is controlled                                  |
| ------------------ | --------------------------------------------------- | --------------------------------------------------- |
| Global defaults    | `--allow-all`, `--deny-all`                         | Baseline capability posture                         |
| Function execution | `--allow-funcs`, `--deny-funcs`                     | Built-in function families and individual functions |
| Outbound network   | `--allow-net`, `--deny-net`                         | Hosts, ports, wildcard domains, IP/CIDR targets     |
| Scripting          | `--allow-scripting`, `--deny-scripting`             | Embedded JavaScript execution                       |
| Guest access       | `--allow-guests`, `--deny-guests`                   | Guest query execution                               |
| Arbitrary queries  | `--allow-arbitrary-query`, `--deny-arbitrary-query` | Dynamic query access by user group                  |
| Query evaluation   | `--allow-eval-query`, `--deny-eval-query`           | `eval::surql` / `eval::gql`                         |
| RPC                | `--allow-rpc`, `--deny-rpc`                         | Individual RPC methods                              |
| Inbound HTTP       | `--allow-http`, `--deny-http`                       | Individual HTTP route families                      |
| MCP                | `--allow-mcp`, `--deny-mcp`                         | Individual MCP tools                                |
| Experimental       | `--allow-experimental`, `--deny-experimental`       | Experimental engine capabilities                    |

These controls are not interchangeable. For example, allowing outbound HTTP function calls does not imply allowing every inbound HTTP route; permitting MCP requests does not grant every MCP tool.

A restricted configuration can be expressed directly in the start command:

```
surreal db start \
  --deny-all \
  --allow-funcs "array,string,crypto::argon2,http::get" \
  --allow-net api.example.com \
  surrealkv://data
```

The database's own authentication and authorization remain in force independently of the server capability configuration. In particular, `--unauthenticated` is dangerous for exposed services because it allows privileged unauthenticated access.

An advanced distinction: the `eval-query` capability is denied by default even under `--allow-all` and must be explicitly enabled. It is also an additional gate on top of the relevant arbitrary-query permission.

#### Background execution and maintenance flags

The CLI also exposes operational controls that are easy to overlook because they are not central to everyday server startup.

| Flag                                                     | CLI-controlled operation                         |
| -------------------------------------------------------- | ------------------------------------------------ |
| `--index-compaction-interval`                            | Periodic consolidation of pending index updates  |
| `--graph-fold-interval`                                  | Background packing of graph adjacency entries    |
| `--index-build-resume-interval`                          | Resume index builds interrupted by failed nodes  |
| `--async-event-interval`                                 | Background async-event processing cadence        |
| `--reclaim-interval`, `--reclaim-grace`                  | Physical reclamation after object removal        |
| `--changefeed-gc-interval`                               | Changefeed garbage collection                    |
| `--node-membership-*`                                    | Node discovery, membership checking, and cleanup |
| `--tikv-gc-*`, `--tikv-lock-cleanup-interval`            | Distributed TiKV maintenance                     |
| `--durable-session-ttl`, `--durable-session-gc-interval` | HTTP RPC session retention and cleanup           |

These flags change the cadence or limits of database-maintenance tasks. Their existence does not imply that every backend uses every setting.

There is also an important distinction between explicit CLI options and engine-only environment variables. Settings such as `SURREAL_SCAN_BATCH_BYTES`, `SURREAL_OPERATOR_BUFFER_SIZE`, `SURREAL_KNN_PREFILTER_ENABLED`, and `SURREAL_TRANSACTION_MAX_WRITE_KEYS` are documented in the runtime configuration reference but should not be assumed to have identical command-line flags.

Primary references:

[https://surrealdb.com/docs/reference/cli/db/commands/start](https://surrealdb.com/docs/reference/cli/db/commands/start?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/cli/db/environment-variables](https://surrealdb.com/docs/reference/cli/db/environment-variables?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+3

### 3.3 `surreal db sql`: REPL and scripted execution

The SQL command supports both interactive terminal use and non-interactive execution through stdin.

```
# Interactive shell
surreal db sql \
  -e ws://localhost:8000 \
  --ns main --db main \
  --user root --pass secret

# Machine-oriented one-shot query
printf 'INFO FOR DB;\n' |
  surreal db sql \
    -e http://localhost:8000 \
    --ns main --db main \
    --json --hide-welcome

# Direct embedded storage access
surreal db sql \
  --endpoint surrealkv://local-data
```

Its primary options are:

| Flag                        | Function                                                      |
| --------------------------- | ------------------------------------------------------------- |
| `--endpoint`                | Remote HTTP/WebSocket/gRPC endpoint or local embedded storage |
| `--username`, `--password`  | Username/password authentication                              |
| `--token`                   | JWT authentication                                            |
| `--auth-level`              | Root, namespace, or database authentication context           |
| `--namespace`, `--database` | Query context                                                 |
| `--json`                    | JSON result presentation                                      |
| `--pretty`                  | Human-readable formatting                                     |
| `--multi`                   | Multiline statement entry                                     |
| `--hide-welcome`            | Remove REPL welcome output from pipelines                     |

Interactive operation includes command history and terminal history navigation.

The important behavioral distinctions are:

Piped execution. The command accepts SurrealQL from stdin without an interactive REPL. It is intended primarily for a modest number of statements; `import` is the dedicated mechanism for large database dumps.

Embedded execution. An endpoint such as `surrealkv://local-data` opens the database engine inside the CLI process. This is not a network connection and is subject to the datastore's exclusive-access rules.

Local versus remote capability flags. `sql` exposes database capability flags, but for a remote connection the server determines the effective capability policy. A local CLI flag cannot override a remote server's restrictions.

Exit status. SurrealDB 4.0 returns nonzero status for failures in piped query execution, improving shell error detection relative to certain earlier versions.

Output fidelity. `--json` produces JSON query results, but it should not be confused with a stable typed API, a schema-dump format, or database-server log output.

Primary reference: [https://surrealdb.com/docs/reference/cli/db/commands/sql](https://surrealdb.com/docs/reference/cli/db/commands/sql?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 3.4 `surreal db export`: selective logical exports

`export` produces a SurrealQL representation of database definitions and, optionally, records.

```
# Export a database
surreal db export \
  -e http://localhost:8000 \
  --ns main --db main \
  database.surql

# Export selected tables and records
surreal db export \
  -e http://localhost:8000 \
  --ns main --db main \
  --only \
  --tables person,order \
  --records \
  -- selected.surql

# Stream the export through stdout
surreal db export \
  --ns main --db main - > database.surql
```

The `--only` flag starts with an empty export selection; subsequent selector flags add the desired categories.

| Selector                   | Exportable information                                                                  |
| -------------------------- | --------------------------------------------------------------------------------------- |
| `--database-definition`    | Database definition, including applicable strictness, comments, and changefeed metadata |
| `--tables`                 | Table definitions, optionally restricted by name                                        |
| `--tables-exclude`         | Exclude specified tables                                                                |
| `--records`                | Record contents                                                                         |
| `--users`, `--accesses`    | User and access-method definitions                                                      |
| `--functions`, `--params`  | User-defined functions and parameters                                                   |
| `--analyzers`              | Search analyzer definitions                                                             |
| `--apis`, `--buckets`      | Custom API and bucket definitions                                                       |
| `--modules`                | Module-related definitions where applicable                                             |
| `--configs`, `--sequences` | Configuration and sequence definitions                                                  |

The selectors can be combined to produce schema-only, records-only, or selected-table exports.

Several limitations matter:

- Exports contain current record state, not SurrealKV historical versions. `--versions true` is not a working history-export option.
- Credential hashes are exported only when permitted; otherwise they are redacted.
- Bucket backend credentials can be redacted even for privileged exports.
- gRPC has a restriction on excluding the database definition.
- Version 4.0 sends export logs to stderr, keeping stdout suitable for redirection.
- A failed HTTP export now reports an error instead of silently treating a truncated stream as success in cases where older releases could do so.

Shell parsing subtlety: selector flags such as `--tables` accept optional values. When an output filename follows such a flag, use `--` to separate the filename so it is not interpreted as another flag value.

Primary reference: [https://surrealdb.com/docs/reference/cli/db/commands/export](https://surrealdb.com/docs/reference/cli/db/commands/export?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 3.5 `surreal db import`: logical import execution

```
surreal db import \
  -e http://localhost:8000 \
  --ns main --db main \
  database.surql
```

Unlike a filesystem restore, an import is executed as SurrealQL statements against the destination database.

Its execution semantics are particularly important:

Non-atomic execution. Statements commit individually. Successfully executed statements remain committed if a later statement fails.

Continuation after errors. A failed statement does not necessarily stop execution of subsequent statements. The CLI reports overall failure with exit code `1` after processing.

Request limits. The documented default HTTP import limit is 4 GiB per request. Streaming does not remove this cumulative limit.

Encoding. Imports require UTF-8, with UTF-16 explicitly rejected by the 4.0 CLI.

Side-effect suppression. Standard database exports include `OPTION IMPORT`, which suppresses selected normal query side effects such as events and table-view maintenance during restoration. From SurrealDB 3.0.4, the `/import` endpoint requires this import mode. Normal write-side behavior requires a different execution path through `/sql`.

This makes `import` suitable for logical restoration but not a drop-in replacement for normal application write execution, CDC replay, or migration tracking.

Primary reference: [https://surrealdb.com/docs/reference/cli/db/commands/import](https://surrealdb.com/docs/reference/cli/db/commands/import?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 3.6 Validation, readiness, and version commands

```
# Parse all .surql files recursively
surreal db validate

# Validate selected files and patterns
surreal db validate "schema/**/*.surql"

# Validate stdin
printf 'SELECT * FROM person;\n' |
  surreal db validate --stdin

# Readiness
surreal db is-ready \
  --endpoint http://localhost:8000 \
  --json

# Inspect remote version
surreal db version \
  --endpoint http://localhost:8000 \
  --json
```

`validate` is parser-based and does not execute queries. It cannot establish that referenced tables, fields, permissions, or functions exist.

`is-ready` and `version` both support JSON output in 4.0, but serve different purposes. One checks whether the server can accept connections; the other reports version information.

The local database component's version, the remote server version, and the launcher's selected component version may all differ.

Primary references:

[https://surrealdb.com/docs/reference/cli/db/commands/validate](https://surrealdb.com/docs/reference/cli/db/commands/validate?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/cli/db/commands/isready](https://surrealdb.com/docs/reference/cli/db/commands/isready?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/cli/db/commands/version](https://surrealdb.com/docs/reference/cli/db/commands/version?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2

### 3.7 `surreal db mcp`: embedded MCP server

The `mcp` subcommand starts an embedded Model Context Protocol server communicating over stdio.

```
surreal db mcp \
  --ns main --db main \
  memory
```

It accepts a storage path and initial namespace/database selection. Its database-tuning options overlap with those of `start`.

The essential distinction is that the process opens its own embedded datastore, and MCP tool calls execute with owner-level database access. Authentication flags initialize a root identity where necessary; they do not add per-call authorization to the stdio transport.

For a shared database and scoped authentication, the alternative is to start a normal server and use its `/mcp` HTTP endpoint. That endpoint is a server capability, not another invocation mode of the `mcp` subcommand.

Primary reference: [https://surrealdb.com/docs/reference/cli/db/commands/mcp](https://surrealdb.com/docs/reference/cli/db/commands/mcp?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 3.8 Legacy maintenance and compatibility commands

The remaining commands have narrower version-specific roles.

| Command                       | Valid context       | Behavior                                           |
| ----------------------------- | ------------------- | -------------------------------------------------- |
| `surreal db fix`              | 2.x binaries        | Convert supported 1.x storage to 2.x format        |
| `surreal db upgrade-check`    | Recent 3.3 binaries | Read-only preflight checks for 3.x-to-4.0 upgrades |
| `surreal db ml import/export` | Before 4.0          | Import/export SurrealML models                     |
| `surreal db module`           | Before 4.0          | Manage Surrealism modules through the database CLI |

The `fix` command is not a general-purpose database repair utility. From 3.0 onward it is not implemented.

The 3.3 `upgrade-check` command is more advanced: it can inspect a running server, perform client-side checks through `INFO` and `SELECT`, or inspect a stopped datastore directly.

```
surreal +3 db upgrade-check \
  -e http://localhost:8000 \
  -u root -p secret \
  --scan-records 500 \
  --json
```

Reports distinguish blocking compatibility failures from warnings and informational findings. Optional record scanning is bounded, and the output includes issues that cannot be detected from stored state alone.

It does not establish that arbitrary application queries will work under 4.0. Nor does it perform the upgrade itself.

Primary reference: [https://surrealdb.com/docs/reference/cli/db/commands/upgrade-check](https://surrealdb.com/docs/reference/cli/db/commands/upgrade-check?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



## 4. `surreal cloud`: managed infrastructure and operational CLI

The Cloud component is a separately distributed command-line control plane for SurrealDB Cloud. It replaces the earlier standalone `surrealctl` program and uses the Cloud API rather than directly manipulating a local database process.

```
surreal install cloud
surreal cloud auth login
surreal cloud org list
surreal cloud instance list --json
```

### 4.1 Command hierarchy

| Group                   | Significant commands                                                                                   | Purpose                                     |
| ----------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------- |
| `auth`                  | `login`, `logout`, `status`, `refresh`, `scopes`                                                       | Authentication lifecycle                    |
| `org`                   | `list`, `get`, `create`, `update`, `archive`, `use`, `roles`, `permissions`, `usage`, `spend`, `plans` | Organization administration and consumption |
| `instance`              | `list`, `get`, `deploy`, `update`, `delete`, `pause`, `resume`, `watch`, `status`                      | Instance lifecycle                          |
| `instance`              | `endpoint`, `token`, `jwks`, `sql`, `import`, `export`                                                 | Database access and credentials             |
| `instance`              | `metrics`, `logs`, `usage`, `estimate`                                                                 | Operational and financial inspection        |
| `instance capabilities` | `get`, `set`                                                                                           | Managed database capability policies        |
| `instance network`      | `get`, `set`                                                                                           | Earth instance network restrictions         |
| `instance backup`       | `list`, `create`, `policy get`, `policy set`                                                           | Managed snapshots and retention             |
| `team`, `invite`        | Membership and invitation operations                                                                   | Organization access management              |
| `token`                 | `list`, `create`, `delete`, `scopes`                                                                   | Cloud personal access tokens                |
| `catalog`               | `regions`, `instance-types`, `storage-types`, `instance-versions`, `billing-countries`                 | Discover available platform offerings       |
| `config`, `context`     | Configuration and context commands                                                                     | Profiles and execution context              |
| Other                   | `whoami`, `api`, `open`, `status`, `version`, `completion`                                             | Diagnostics, API access, shell integration  |

The CLI supports plural aliases for resource groups, `ls` for `list`, and `rm` for applicable deletion commands. The instance provisioning command accepts both `deploy` and `create`.

Primary reference: [https://surrealdb.com/docs/reference/cli/cloud/commands](https://surrealdb.com/docs/reference/cli/cloud/commands?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 4.2 Authentication and profile management

The Cloud CLI supports two credential classes.

Interactive login sessions use OAuth 2.0 authorization-code flows with PKCE, including browser, device-code, and paste-based authentication.

Cloud personal access tokens are suited to unattended read-only inspection. They are not database credentials.

```
# Interactive authentication
surreal cloud auth login

# Device-code flow
surreal cloud auth login --flow device

# Store a PAT supplied on stdin
surreal cloud auth login --with-token

# Inspect identity and scopes
surreal cloud auth status
surreal cloud auth scopes
surreal cloud whoami

# Switch organization
surreal cloud org use acme
```

The CLI supports named profiles and configuration contexts, with a separate credential store for each profile.

Important boundaries include:

- A Cloud PAT is refused for Cloud API mutations, regardless of nominal write scopes.
- PAT lifecycle commands require an interactive login session.
- An instance database token is distinct from a Cloud PAT.
- Credential material can be passed through environment variables or token files instead of command-line arguments.
- Interactive secret disclosure is guarded to avoid accidentally printing credentials into terminal scrollback.

The global configuration surface includes `--profile`, `--config`, `--org`, `--token`, `--token-file`, `--api`, `--json`, `--plain`, `--yes`, `--no-input`, `--quiet`, and timeout/debugging controls.

Primary references:

[https://surrealdb.com/docs/reference/cli/cloud/authentication](https://surrealdb.com/docs/reference/cli/cloud/authentication?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/cli/cloud/global-flags](https://surrealdb.com/docs/reference/cli/cloud/global-flags?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1

### 4.3 Instance provisioning, scaling, and lifecycle

The CLI handles both serverless Earth instances and dedicated instances.

```
# Earth instance
surreal cloud instance deploy edge-api \
  --residency eu \
  --location-hint weur

# Dedicated instance
surreal cloud instance deploy production \
  --region aws-euw1 \
  --nodes 3 \
  --type production-2 \
  --storage 100

# Scale an existing dedicated instance
surreal cloud instance update production \
  --compute-units 8

# Pause, resume, or observe
surreal cloud instance pause staging
surreal cloud instance resume staging
surreal cloud instance watch staging --until ready
```

A subtle default: deployment without a dedicated region creates an Earth instance. Earth and dedicated deployments expose different scaling, networking, backup, and observability commands.

Dedicated deployment options include node count, region, instance type, storage, version, and public/private access settings. Earth deployment offers residency, location hints, cell limits, and relevant network/capability policies.

#### Operation waiting

The CLI normally waits for selected asynchronous operations to finish, but supports `--no-wait` to return after acceptance.

`instance watch` can subsequently wait for `ready`, `paused`, or `deleted`.

A timeout is not necessarily an operation failure. It means the CLI did not observe completion within its wait interval.

#### Non-atomic updates

This is one of the most consequential properties of the Cloud CLI.

`instance update` can change several fields in one invocation, but sends separate API operations in a specified order. Each accepted change can restart the instance.

If a later change fails, earlier changes remain applied.

Similarly, instance creation has no generally available idempotency key, so blindly repeating a create request after an ambiguous network failure can create another billable instance.

Those semantics matter when using the CLI as infrastructure automation rather than an interactive administration tool.

### 4.4 Instance credentials and data-plane delegation

```
# Database connection endpoint
surreal cloud instance endpoint production

# Mint a database token
surreal cloud instance token production

# Inspect verification keys
surreal cloud instance jwks production

# Open an authenticated SQL shell
surreal cloud instance sql production -- \
  --ns main --db main
```

`instance sql`, `instance import`, and `instance export` resolve the Cloud instance, obtain the necessary database credentials, and delegate to the installed `surreal db` component.

The `--` separator passes subsequent arguments to the database command.

This is not a second SQL engine or import/export implementation. The same database CLI behavior, including import partial failures and export limitations, applies.

### 4.5 Capability and network-policy management

The Cloud CLI exposes managed capability configuration rather than requiring server startup flags.

```
# Inspect capability policy
surreal cloud instance capabilities get production

# Disable scripting
surreal cloud instance capabilities set production \
  --deny-scripting

# Restrict Earth network access
surreal cloud instance network set edge-api \
  --allow-ip 203.0.113.0/24
```

Capability changes can address scripting, guest access, arbitrary queries, evaluation, RPC, HTTP, function families, and network permissions.

The CLI reads the existing policy and applies the specified changes. List options replace the corresponding list, but unspecified policy fields are preserved.

Earth network policies can include IP/CIDR allowlists and denylists, with deny rules taking precedence.

### 4.6 Backups, observability, and billing

```
# Backup management
surreal cloud instance backup list production
surreal cloud instance backup create production

surreal cloud instance backup policy get production
surreal cloud instance backup policy set production \
  --daily 14 --weekly 8

# Restore into a new dedicated instance
surreal cloud instance deploy recovery \
  --region aws-euw1 \
  --type shared-1 \
  --restore-from production/SNAPSHOT_ID

# Operational inspection
surreal cloud instance metrics production --metric memory
surreal cloud instance logs production --level error,warn
surreal cloud instance usage production
surreal cloud instance estimate production
```

Backup creation returns acceptance rather than waiting for completion. The request is not automatically retried after ambiguous failures, because another request could create another snapshot.

Restoration is exposed through new-instance deployment using `--restore-from`, not through a generic in-place restore command.

The CLI also supports log following with newline-delimited JSON, metrics inspection, usage reporting, and pricing estimates. These operations are more extensive for dedicated instances than for Earth.

### 4.7 Automation contract: structured output and exit codes

The Cloud CLI has a particularly useful automation model:

- stdout contains command data; stderr contains prompts, progress, and diagnostics.
- `--json` returns structured payloads without decorative text.
- List commands support sorting, filtering by selection, limiting, custom columns, and plain output.
- `--no-input` forbids interactive prompts.
- `--yes` accepts supported confirmation prompts.
- Long-running operations expose explicit waiting controls.

The exit codes are designed to be interpreted programmatically.

| Exit code | Meaning                                          |
| --------- | ------------------------------------------------ |
| `0`       | Successful operation                             |
| `2`       | Invalid invocation or confirmation problem       |
| `3`       | Authentication failure                           |
| `4`       | Authorization or credential-type rejection       |
| `5`       | Resource not found                               |
| `6`       | Invalid state or failed precondition             |
| `8`       | Rate limiting after retry exhaustion             |
| `9`       | Unavailable or unsupported API capability        |
| `10`      | Wait timed out; operation may remain in progress |
| `12`      | Required CLI component unavailable               |
| `30`      | Interactive authentication could not complete    |

This enables scripts to distinguish a missing resource from an authorization error, or an operation timeout from a request that was rejected before execution.

The Cloud CLI's low-level `api` command is additionally useful when a supported Cloud API operation is not exposed by a specialized command, though it does not bypass authorization or provide undocumented API compatibility guarantees.

Primary references:

[https://surrealdb.com/docs/reference/cli/cloud/commands/instance](https://surrealdb.com/docs/reference/cli/cloud/commands/instance?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/cli/cloud/output-and-exit-codes](https://surrealdb.com/docs/reference/cli/cloud/output-and-exit-codes?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2

## 5. `surreal ext`: Rust/WASM extension development and deployment

Status: documented SurrealDB 4.0 CLI; standalone component publication remains pending in the official installation guide

`surreal ext` is the dedicated CLI for developing and managing Surrealism extensions: Rust code compiled to WebAssembly and packaged for execution inside SurrealDB.

Unlike the general Rust SDK, this toolchain has a complete command-line lifecycle for building, testing, inspecting, signing, and installing executable packages.

Availability qualification: although the 4.0 reference specifies these commands, the official workflow guide explicitly states that the first standalone `ext` component release has not yet been published, and `surreal install ext` does not work. The commands below document the implemented interface, not a verified downloadable binary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 5.1 Complete extension command surface

| Command               | Capability                                                                          |
| --------------------- | ----------------------------------------------------------------------------------- |
| `surreal ext init`    | Scaffold a Rust extension project and package manifest                              |
| `surreal ext build`   | Compile Rust to WASI, optimize, preinitialize, and package `.surli`                 |
| `surreal ext run`     | Invoke an exported function locally without a database                              |
| `surreal ext sig`     | Inspect a function's exported signature                                             |
| `surreal ext info`    | List package metadata and exported functions; supports JSON                         |
| `surreal ext config`  | Read and modify package metadata, capabilities, signing identity, and bundled files |
| `surreal ext sign`    | Sign a package with SSH or Sigstore                                                 |
| `surreal ext install` | Inspect provenance and capabilities, then define an extension in a database         |
| `surreal ext upgrade` | Compare and upgrade an installed extension with explicit approval controls          |

### 5.2 Project creation, build, and local execution

```
rustup target add wasm32-wasip2

surreal ext init \
  --headless --org example --name functions \
  ./functions

surreal ext build ./functions \
  --out functions.surli

surreal ext info --json functions.surli

surreal ext sig --fnc greet functions.surli

surreal ext run \
  --fnc greet \
  --arg '"Ada"' \
  functions.surli
```

`init` creates a Rust crate, `.cargo/config.toml`, `src/lib.rs`, and `package.toml`. The manifest declares the package name, version, ABI, and requested execution capabilities.

`build` produces a `.surli` archive. The build pipeline includes WASM compilation, optimization, preinitialization, export-signature extraction, manifest hashing, and package creation. `--debug` skips optimization for faster development builds.

The preinitialization mechanism is significant: initialization runs during packaging, and each function invocation receives fresh memory derived from the resulting state. It is not a persistent mutable cache shared across invocations.

`run` provides an especially useful development boundary because a function can be exercised without starting a database. Its arguments are SurrealQL-formatted values, supplied by repeating `--arg`.

`info` and `sig` expose metadata that can be inspected programmatically, including callable signatures and whether exports are read-only.

### 5.3 Package configuration and bundled resources

The `config` group provides commands such as:

```
surreal ext config show
surreal ext config get package.version
surreal ext config set package.version 1.1.0

surreal ext config add \
  capabilities.allow_net api.example.com:443

surreal ext config add \
  capabilities.allow_functions http::get

surreal ext config fs add ./resources --as /resources
```

Relevant subcommands include `show`, `get`, `set`, `add`, `remove`, `signer`, and `fs`.

The `fs add` operation is easily overlooked: it embeds a directory of static resources in the extension's read-only filesystem. This can avoid runtime network requests for suitable lookup tables, configuration assets, and other immutable resources.

The manifest describes capabilities such as database writes, arbitrary queries, script execution, permitted functions, outbound network destinations, timeouts, and memory constraints.

Requested capabilities are not automatically granted at installation. The package declares what it needs; the installer and database permission system determine what it receives.

### 5.4 Signing and supply-chain verification

The CLI supports several signing arrangements:

- SSH-key signing, implemented through `ssh-keygen -Y`.
- Sigstore keyless signing associated with GitHub Actions or GitLab CI identities.
- Importing an existing Sigstore bundle with a configured trust root.
- Private or offline Sigstore verification through explicit trusted-root options.

For example:

```
surreal ext config signer ssh ~/.ssh/release.pub
surreal ext build --out functions.surli
surreal ext sign --key ~/.ssh/release functions.surli
```

The archive's `MANIFEST` contains hashes of packaged files, and the extension definition pins its digest. The signature is stored separately, so signing does not modify the manifest-derived digest.

The distinction between content digest, publisher identity, and granted capabilities is central to the tooling. A validly signed package is not necessarily authorized to perform every operation it requests.

### 5.5 Installation and controlled upgrades

`install` accepts a package from a supported Silo source or from an archive already stored in a database bucket. It does not simply accept any local filesystem path as a deployable database source.

Illustrative installation, after placing the archive in a database bucket:

```
surreal ext install \
  'f"extensions:/functions.surli"' \
  --as functions \
  --endpoint ws://localhost:8000 \
  --ns app --db main \
  --trust-signer SHA256:TRUSTED_FINGERPRINT \
  --with-capability 'net:api.example.com:443'
```

For non-interactive operation, trust and capability decisions must be specified explicitly. Other significant switches include:

| Flag                | Purpose                                                             |
| ------------------- | ------------------------------------------------------------------- |
| `--dry-run`         | Perform review and print the proposed definition without installing |
| `--unsigned`        | Explicitly permit an unsigned package                               |
| `--offline`         | Avoid online Sigstore checks while retaining offline verification   |
| `--trust-root`      | Supply a custom trust root                                          |
| `--trust-signer`    | Specify acceptable signing identities                               |
| `--with-capability` | Grant individual requested capabilities                             |

There is deliberately no general option that silently grants every requested capability.

The `upgrade` command retains existing permissions and comments, starts from previously granted capabilities, and compares the replacement package with the installed one.

It distinguishes changes that may require separate authorization:

- A changed signer: `--allow-signer-change`.
- A version downgrade: `--allow-downgrade`.
- A changed organization or package: `--allow-package-change`.
- Additional requested capabilities: explicit `--with-capability` grants.

This gives the CLI a package-provenance and privilege-review role beyond merely invoking `DEFINE EXTENSION`.

Version boundary: SurrealDB 4.0 changes the extension ABI and adopts `ext::` naming. Earlier 3.x packages and `module` tooling cannot be treated as drop-in compatible.

Primary references:

[https://surrealdb.com/docs/reference/cli/ext](https://surrealdb.com/docs/reference/cli/ext?utm_source=chatgpt.com)

[https://surrealdb.com/docs/explore/cli/extensions](https://surrealdb.com/docs/explore/cli/extensions?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2

## 6. `surreal lsp`: standalone SurrealQL semantic analysis and language tooling

The SurrealQL language server is another Rust-based command-line program that deserves independent treatment. It is not merely a VS Code extension.

Its executable, `surreal-lsp`, supports both Language Server Protocol operation and headless project checking. A second executable name, `surrealql-language-server`, refers to the same implementation.

### 6.1 Commands and distribution

The upstream repository documents:

```
surreal install lsp

# Launch the LSP server over stdio
surreal lsp stdio

# Analyze SurrealQL files without an editor
surreal lsp check queries/

# Build from the Rust source tree
cargo build --release
```

For source builds, the Tree-sitter SurrealQL grammar is a separate repository checked out through the project's setup script or supplied through `TREE_SITTER_SURREALQL_DIR`.

The repository documents launcher-based installation from SurrealDB's component distribution system. I verified that documented integration but did not independently execute an installation or establish the publication status of every platform binary.

### 6.2 Headless semantic validation

The most significant CLI capability is `check`:

```
surreal lsp check queries/ \
  --json --fail-on warning

surreal lsp check \
  --stdin \
  --stdin-filename generated.surql \
  --workspace schema/ \
  --json
```

Unlike the database CLI's parser-only `validate`, the language server can use definitions from the surrounding workspace to produce semantic diagnostics.

The analysis includes:

| Capability          | What the CLI can detect or infer                                        |
| ------------------- | ----------------------------------------------------------------------- |
| Syntax              | Tree-sitter parse errors and source locations                           |
| Schema symbols      | Tables, fields, functions, indexes, parameters, and related definitions |
| Types               | Field constraints, inferred expression types, function parameter types  |
| References          | Undefined or incompatible references within analyzed scope              |
| Functions           | Argument count/type issues and known builtin signatures                 |
| Permissions         | Relevant permission posture and missing clauses                         |
| Embedded scripting  | Distinguishing JavaScript bodies from SurrealQL syntax                  |
| Cross-file analysis | Definitions and references distributed across workspace files           |

The headless runner emits structured diagnostics with locations, severity, diagnostic codes, and execution statistics.

Its exit statuses distinguish:

- `0`: no diagnostics meeting the configured failure threshold.
- `1`: diagnostics meeting that threshold.
- `2`: operational or input-processing failure.

This makes it suitable for static validation of generated queries without starting a database server.

### 6.3 LSP-specific capabilities

When used through `stdio`, the same engine supports contextual completion, hover type information, signature help, go-to-definition, references, document symbols, rename operations, code actions, and function call hierarchy.

These features are not separate shell commands, but they are functionality exposed through the CLI's stdio protocol.

Two advanced configuration facilities are important:

Schemaless diagnostics. `analysis.schemalessDiagnostics` supports `quiet`, `errors`, and `strict` modes, controlling how strongly the analyzer treats field assumptions in schemaless tables.

Externally bound parameters. `analysis.externalParams` describes variables supplied by the query caller, avoiding spurious undefined-variable findings.

The repository also distributes a generated `builtins.json` catalog containing builtin function signatures and type information. This is a useful machine-readable byproduct of the language-analysis toolchain.

### 6.4 Limits and compatibility

The analyzer has bounded workspace scanning: its documented limit is 5,000 `.surql` files, and files larger than 2 MB are skipped. The grammar is pinned to a specific upstream revision.

Accordingly, `surreal lsp check` is not a substitute for executing queries on the target database release. A static analysis pass cannot guarantee runtime permissions, index availability, transaction success, or exact server-version semantics.

It complements `surqlfmt`, `surreal db validate`, and database-backed tests, each of which answers a different question.

Primary reference:

[https://github.com/surrealdb/surrealql-language-server](https://github.com/surrealdb/surrealql-language-server?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1

## 7. SurrealKit: schema-management, migration, seeding, and testing CLI

SurrealKit is a separate Rust-based CLI for managing SurrealQL schema definitions and their lifecycle. Its principal distinction is that it supplies both declarative schema reconciliation and explicit staged migrations, rather than treating every schema change as a standalone SQL file.

Version qualification: the latest non-beta version exposed by docs.rs is 0.7.0, while the upstream main branch documents newer 1.0 beta functionality. In particular, immutable rollout SQL, multi-version catch-up, and some advanced testing behavior are documented for the beta line. The complete main-branch command surface should not be assumed to exist in 0.7.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 7.1 Installation and command groups

```
# Download a published binary through cargo-binstall
cargo binstall surrealkit

# Or build the published crate from source
cargo install surrealkit

# Initialize SurrealKit metadata and folders
surrealkit init
```

| Command   | Capability                                                                              |
| --------- | --------------------------------------------------------------------------------------- |
| `init`    | Scaffold schema, migration, seed, and test directories                                  |
| `sync`    | Reconcile `.surql` schema files with the database                                       |
| `rollout` | Plan, apply, complete, inspect, and recover staged migrations                           |
| `seed`    | Apply tracked seed files                                                                |
| `typegen` | Generate database-derived JSON/TypeScript type information                              |
| `test`    | Execute declarative SQL, permission, schema, and API tests                              |
| `apply`   | Additional statement/file application behavior, where supported by the selected release |

The CLI accepts connection arguments such as `--host`, `--ns`, `--db`, `--user`, `--pass`, and `--auth-level`.

It can connect to an existing database or use a compiled-in embedded storage backend. The published prebuilt executable intentionally includes only the in-memory embedded engine; opening SurrealKV files directly requires a source build with `kv-surrealkv`.

Embedded access still observes exclusive datastore locking. Running a migration CLI against a local SurrealKV directory already opened by another process can fail.

### 7.2 Declarative schema synchronization

```
# Reconcile the configured schema files
surrealkit sync

# Keep reconciling on file changes
surrealkit sync --watch

# Apply a selected schema module and its dependencies
surrealkit sync --schema billing

# Apply every configured module/target combination
surrealkit sync --all --keep-going
```

The `sync` mode compares desired definitions with the objects tracked as owned by SurrealKit. It creates or updates definitions and can remove managed objects whose source files were deleted.

This is considerably more powerful than importing a schema file, because it tracks ownership and identifies deletions as part of the desired state.

Its safeguards include:

- Refusing an empty schema source set before opening a connection, unless `--allow-empty-prune` is explicitly supplied.
- Limiting automatic removal to objects owned by the selected schema module.
- Requiring `--allow-shared-prune` before destructive reconciliation against configured shared databases.
- Restricting schema files to definition-oriented statements unless `--allow-all-statements` is supplied.
- Detecting malformed files before applying definitions or pruning objects.

For most definitions, sync rewrites `DEFINE` to be safely repeatable using `OVERWRITE`. Two exceptions illustrate why the tool is schema-aware rather than a simple script runner.

Sequences: overwriting an existing `DEFINE SEQUENCE` can reset its counter. SurrealKit therefore does not blindly overwrite these definitions.

Access methods: overwriting an access definition whose JWT key is implicitly generated can rotate the signing key and invalidate existing sessions. The tool warns about this behavior.

### 7.3 Schema modules, dependency ordering, and targets

SurrealKit can divide a large schema into named modules, with `depends_on` relationships controlling application order.

```
surrealkit sync --schema billing
surrealkit sync --schema billing --no-deps
surrealkit sync --target warehouse
surrealkit sync --all
```

Each module maintains separate object-ownership metadata, preventing one module from automatically removing another module's definitions.

Targets specify different namespace/database combinations and may limit which modules apply. Passwords can be supplied indirectly through named environment variables rather than stored in plaintext configuration.

The fan-out implementation is sequential, not a distributed transaction. A failure can leave some targets synchronized while others remain unchanged. `--keep-going` continues processing other targets, but the overall command returns a failure when any selected target fails.

This is relevant to CLI automation because it establishes a retryable, inspectable sequence of independent database changes rather than a globally atomic deployment.

### 7.4 Versioned rollouts: expand, cut over, contract

The newer rollout system separates planned migration contents from the mutable working schema.

```
# Establish an initial baseline
surrealkit rollout baseline

# Create a reviewed migration
surrealkit rollout plan --name add_customer_indexes

# Changes to existing definitions require explicit opt-in
surrealkit rollout plan \
  --name tighten_validation \
  --allow-modified

# Inspect migration consistency
surrealkit rollout lint

# Start pending migrations
surrealkit rollout up

# Finish the latest pending contract phase
surrealkit rollout up --complete

# Inspect database migration state
surrealkit rollout status
```

The underlying state model distinguishes:

1. Planning: compare desired and deployed definitions; generate a versioned migration manifest.
2. Expand: add new objects or compatible definitions while retaining legacy objects.
3. Application cutover: allow clients and dependent services to transition.
4. Contract: perform removals or incompatible cleanup after the cutover.

Additional rollout commands documented by the beta implementation include `start`, `complete`, `rollback`, `discard`, `repair`, and `freeze`.

These controls are more expressive than a single up/down migration because the system records intermediate lifecycle state.

#### Immutable migration inputs

The beta rollout planner copies changed SQL files into a directory associated with the migration manifest and records SHA-256 hashes.

This means a migration planned against an older revision can still execute its original statements when a later application revision is already present. Editing the frozen files invalidates their hashes.

Custom `run_sql` and `assert_sql` steps can be added to reviewed manifests for backfills and checks.

#### Recovery and rollback

The migration state is stored in the database. The CLI can detect a partially executed rollout and offer state-repair operations.

`repair` is not equivalent to rerunning all migration SQL. Its purpose is to reconcile recorded migration state when the execution and metadata have become inconsistent.

Rollback also has an important boundary: restoring an older field definition does not undo data already written under the newer definition. SurrealKit reports such changes as definition-only reversible where applicable.

#### Rollout linting

`rollout lint` detects more than SQL syntax errors. Its checks include missing or modified frozen files, conflicting histories, gaps in the rollout chain, and schema changes that have not been represented by a rollout.

That makes it a version-history validator, complementing parser and semantic-analysis tools.

### 7.5 Seed tracking

```
surrealkit seed
surrealkit seed --force
```

The seeding system records each file's content hash in database metadata. An unchanged file is skipped on subsequent runs; a changed file runs again.

`--force` bypasses that tracking and executes every seed.

This avoids the repeated-execution behavior of `surreal db start --import-file`, but is not equivalent to a transactional dependency manager for all seed operations. Seed SQL still needs to account for changes that cannot safely be reapplied.

### 7.6 Type generation

SurrealKit includes a `typegen` command that introspects database definitions and emits type information in machine-readable form.

Its documented output targets include JSON descriptions and TypeScript.

It does not provide a general native Rust struct generator from an existing SurrealDB schema. That distinction matters because the CLI's type-generation functionality should not be confused with Rust's `SurrealValue` derive system.

### 7.7 Declarative database-backed testing

```
# Run configured TOML suites
surrealkit test

# Restrict tests and emit structured results
surrealkit test \
  --suite 'permissions*' \
  --parallel 4 \
  --json-out results.json

# Compare schema behavior from sync and rollouts
surrealkit test --schema-from both
```

The test runner supports:

| Test type            | Purpose                                                        |
| -------------------- | -------------------------------------------------------------- |
| `sql_expect`         | Assert query results or expected failures                      |
| `permissions_matrix` | Exercise authorized and denied operations for specified actors |
| `schema_metadata`    | Assert definitions and metadata                                |
| `schema_behavior`    | Test constraints and schema behavior                           |
| `api_request`        | Exercise database-defined HTTP API endpoints                   |

Suites run against isolated ephemeral namespace/database contexts. Actors can represent root, namespace, database, record-access, bearer-token, and custom-header authentication configurations.

The CLI supports selection and execution controls such as `--suite`, `--case`, `--tag`, `--fail-fast`, `--parallel`, `--json-out`, `--no-sync`, `--no-seed`, `--no-setup`, `--timeout-ms`, and `--keep-db`.

A particularly advanced beta feature is `--schema-from sync|rollouts|both`. It allows test databases to be constructed from desired-state synchronization or by replaying the migration history, with parity checking between the two.

This can reveal a migration-history gap that would not be detected by testing only the final schema definition.

### 7.8 Templates and variable substitution

`init` supports bundled or custom templates, including local directories and Git repositories pinned to a revision. Template features can declare dependencies on other features, and non-interactive selection is supported with `--feature`, `--minimal`, and `--yes`.

SurrealKit also supports `${VAR_NAME}` substitution in schema, seed, and rollout SQL, resolved from `--var`, `SURREALKIT_VAR_*`, or `surrealkit.toml`.

This is textual substitution, not SQL parameter binding. It has two important consequences:

- A variable value change does not necessarily change the tracked file-content hash, so `sync` may not reapply that file.
- Variables inside SQL string literals are also substituted unless explicitly escaped.

The source describes these as known limitations, and they are material when using templates for environment-dependent schemas.

Primary references:

[https://github.com/surrealdb/surrealkit](https://github.com/surrealdb/surrealkit?utm_source=chatgpt.com)

[https://surrealdb.com/docs/manage/schema-migration](https://surrealdb.com/docs/manage/schema-migration?utm_source=chatgpt.com)

[https://surrealdb.com/docs/manage/schema-migration/rollouts](https://surrealdb.com/docs/manage/schema-migration/rollouts?utm_source=chatgpt.com)

[https://surrealdb.com/docs/manage/schema-migration/testing](https://surrealdb.com/docs/manage/schema-migration/testing?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+4

## 8. Surreal Sync: database migration, CDC, and transformation CLI

Status: first-party Rust project, explicitly under active development and not yet stable

`surreal-sync` is a standalone CLI for migrating data from external systems to SurrealDB. It differs fundamentally from `surreal db import`: it understands external source connectors, initial snapshots, change-data-capture streams, source checkpoints, and optional transformation workers.

It is not currently described as a component installed through `surreal install`.

### 8.1 Installation and command model

The upstream repository documents building the Rust executable from source or obtaining a release archive.

```
cargo build --release
# Result: target/release/surreal-sync
```

The general command shape is:

```
surreal-sync from <SOURCE> <COMMAND> \
  --connection-string "$SOURCE_CONNECTION" \
  --surreal-endpoint "$SURREAL_ENDPOINT" \
  --surreal-username "$SURREAL_USERNAME" \
  --surreal-password "$SURREAL_PASSWORD" \
  --to-namespace target_ns \
  --to-database target_db
```

The source selection determines the available subcommands. Supported workflows include `full`, `incremental`, `sync`, `snapshot`, and connector-specific import operations; not every connector exposes all of these.

### 8.2 Connector and command coverage

| Source family                 | CLI source                 | Supported operation                              |
| ----------------------------- | -------------------------- | ------------------------------------------------ |
| PostgreSQL triggers           | `from postgresql-trigger`  | Full, incremental, combined sync                 |
| PostgreSQL wal2json           | `from postgresql`          | Full, incremental, combined sync                 |
| PostgreSQL native replication | `from postgresql-pgoutput` | Snapshot plus streaming replication              |
| MySQL/MariaDB triggers        | `from mysql`               | Full, incremental, combined sync                 |
| MySQL/MariaDB binlog          | `from mysql-binlog`        | Snapshot plus streaming CDC                      |
| MongoDB                       | `from mongodb`             | Collection snapshot and change-stream processing |
| SQL Server                    | `from mssql`               | Snapshot and native CDC                          |
| Neo4j                         | `from neo4j`               | Nodes and relationships, full/incremental        |
| Kafka                         | `from kafka`               | Topic consumption, optional deduplication        |
| CSV                           | `from csv`                 | File ingestion                                   |
| JSON Lines                    | `from jsonl`               | File ingestion                                   |
| Snowflake                     | `from snowflake`           | One-shot snapshot ingestion                      |
| BigQuery                      | `from bigquery`            | One-shot snapshot ingestion                      |

The connector implementations differ in how they establish an initial snapshot, identify changed rows, preserve ordering, and record progress. Consequently, the presence of a connector is not a universal guarantee of equivalent CDC behavior.

For example, PostgreSQL trigger-based capture differs from native `pgoutput`: the former uses source-side change tables and sequence checkpoints, while the latter consumes logical replication output and tracks replication positions.

Source: [https://github.com/surrealdb/surreal-sync](https://github.com/surrealdb/surreal-sync?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

surrealdb/surreal-sync · GitHub



### 8.3 Full plus incremental synchronization

An illustrative PostgreSQL workflow is:

```
# Initial data load
surreal-sync from postgresql-trigger full \
  -c surreal-sync.toml

# Apply subsequent changes
surreal-sync from postgresql-trigger incremental \
  -c surreal-sync.toml

# Combined snapshot/catch-up workflow
surreal-sync from postgresql-trigger sync \
  -c surreal-sync.toml
```

The exact source-side prerequisites, credentials, publication/replication slots, trigger installation, and checkpoint options are connector-specific.

The key operational model is that `sync` can coordinate initial loading with later changes, whereas `full` alone only establishes a snapshot.

### 8.4 Shared pipeline and checkpoint semantics

Surreal Sync uses a common processing architecture across its supported connectors:

Source reader

Snapshot rows, CDC events, files, or Kafka messages

Optional transform stages

Stdio/NDJSON workers and ID conversion

Ordered SurrealDB writes

Apply transformed batches in source order

Checkpoint / watermark

Advance following successful sink application

The main execution distinction is between the read-ahead position and the last successfully applied source position.

`max_in_flight` can allow several batches to undergo transformation concurrently, but the final sink and watermark operations remain ordered.

For sources implementing durable post-sink checkpoints, progress is persisted only after writing the corresponding changes. Some other connectors maintain only an in-memory applied position during an incremental run, so a process restart can replay work since its last persisted phase checkpoint.

The pipeline therefore provides at-least-once, not exactly-once, semantics. It is important that any transformation process with external side effects can tolerate retries and replay.

### 8.5 External command workers

Surreal Sync's `--transforms-config` facility is a significant CLI extension mechanism. It can call arbitrary executable workers over stdin/stdout using newline-delimited JSON.

For example:

```
[pipeline]
failure_policy = "fail"
batch_size = 1000
batch_max_wait = "500ms"
timeout = "120s"
max_in_flight = 2

[[transforms]]
type = "command"
command = ["./normalize-records"]
mode = "persistent"
timeout = "60s"
stdio.framer = "ndjson"
retry.max_attempts = 3
retry.initial_backoff = "200ms"

[[transforms]]
type = "command"
command = ["./enrich-records"]
mode = "persistent"
timeout = "30s"
stdio.framer = "ndjson"
```

The CLI then receives the path:

```
surreal-sync from mysql-binlog sync \
  --connection-string "$SOURCE_CONNECTION" \
  --database source_db \
  --surreal-endpoint "$SURREAL_ENDPOINT" \
  --surreal-username root \
  --surreal-password "$SURREAL_PASSWORD" \
  --to-namespace main \
  --to-database main \
  --checkpoint-dir .sync-checkpoints \
  --transforms-config transforms.toml
```

Each command stage can have separate process arguments, timeout, retry/backoff policy, and lifecycle configuration. Stages run in their declared order.

The worker protocol associates requests with batch identifiers. A worker's successful response establishes only that the transform finished, not that the resulting records were durably written to SurrealDB.

The documented tool does not currently provide an arbitrary transformation DSL, WASM transform plugins, HTTP/TCP worker transports, or built-in dead-letter queues.

### 8.6 Failure policies

The `[pipeline]` configuration supports two failure behaviors:

| Policy | Effect                                                                            |
| ------ | --------------------------------------------------------------------------------- |
| `fail` | Stop on a failed transform or sink operation; do not advance past that batch      |
| `skip` | Log the failure and advance the source watermark without writing the failed batch |

The `skip` behavior is intentionally lossy. Once the checkpoint advances, the omitted source events will not be replayed through normal continuation.

Retries and transient failures are distinct from an explicit skip policy.

### 8.7 Record identity and composite keys

The CLI preserves source-record identity where the connector provides it.

For composite primary keys, it can generate SurrealDB array record IDs such as:

```
users:[42, "us-east"]
```

The optional `flatten_id` transformation converts array IDs to text IDs using a configurable separator.

This is not merely cosmetic: flattening can introduce collisions if the separator and key-part encoding are not unambiguous. Preserving structured array identity avoids that particular information-loss problem.

For SQL CDC connectors, key identity is typically discovered from the source schema. For file-based sources and applicable connectors, `--id-columns` or related flags can explicitly define the key columns.

### 8.8 When this CLI differs from the native importer

| Requirement                      | `surreal db import`             | `surreal-sync`                          |
| -------------------------------- | ------------------------------- | --------------------------------------- |
| SurrealQL dump import            | Native                          | Not its primary role                    |
| PostgreSQL/MySQL/MongoDB CDC     | No                              | Connector-specific support              |
| Restart checkpoints              | No built-in source CDC tracking | Yes, connector-dependent                |
| Streaming external changes       | No                              | Yes                                     |
| External transformation workers  | No                              | Yes                                     |
| Record identity mapping          | Input-script responsibility     | Source mapping and transform options    |
| Source-specific schema/CDC setup | No                              | Yes                                     |
| Stable production contract       | Database 4.0 functionality      | Tool explicitly still under development |

Primary references:

[https://github.com/surrealdb/surreal-sync](https://github.com/surrealdb/surreal-sync?utm_source=chatgpt.com)

[https://github.com/surrealdb/surreal-sync/blob/main/docs/sync-pipeline.md](https://github.com/surrealdb/surreal-sync/blob/main/docs/sync-pipeline.md?utm_source=chatgpt.com)

[https://github.com/surrealdb/surreal-sync/blob/main/docs/postgresql.md](https://github.com/surrealdb/surreal-sync/blob/main/docs/postgresql.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+3

## 9. `skv`: SurrealKV physical storage inspection and migration

`skv` is a separate Rust-based operator CLI packaged as `surrealkv-cli`.

It operates on the physical key-value storage layer, not on SurrealQL tables, record permissions, or the database's logical schema.

```
cargo install surrealkv-cli
```

### 9.1 Command surface

| Command        | Capability                                                 |
| -------------- | ---------------------------------------------------------- |
| `skv inspect`  | Summarize a datastore directory and its storage structures |
| `skv manifest` | Inspect the LSM level manifest and table hierarchy         |
| `skv sst`      | Inspect individual SSTable files, keys, and checksums      |
| `skv scrub`    | Verify SSTable data-block integrity                        |
| `skv get`      | Read a low-level key                                       |
| `skv scan`     | Scan physical key ranges or prefixes                       |
| `skv enc-info` | Inspect encryption metadata without decrypting data        |
| `skv migrate`  | Convert supported older storage formats                    |

Representative operations:

```
skv inspect path/to/db

skv manifest path/to/db

skv sst path/to/db/sstables/00000000000000000001.sst \
  --dump-keys \
  --verify-checksum

skv scrub path/to/db

skv get path/to/db "user:001"

skv scan path/to/db --prefix "user:"
```

### 9.2 Low-level diagnostics

`inspect` and `manifest` make it possible to investigate the on-disk LSM organization without executing SurrealQL.

`sst` provides more granular inspection of individual sorted-table files. Its checksum flag verifies relevant SSTable contents.

`skv scrub` performs a CRC32-based integrity sweep of SSTable data blocks.

These operations have a different evidentiary scope from application-level validation: a successful checksum sweep establishes certain physical integrity properties, not the correctness of graph relationships, table definitions, indexes, or application invariants.

`get` and `scan` work with physical storage keys. They should not be assumed to accept normal SurrealQL record identifiers as a human-facing record API.

### 9.3 Offline storage migration

```
skv migrate old-storage-dir new-storage-dir
```

The migration implementation supports specific older SurrealKV formats and SurrealDB RocksDB datastores, converting them into the newer SurrealKV representation.

The source database must not be concurrently opened or modified during offline migration.

For supported RocksDB imports, the compatibility implementation reads legacy storage files without linking the RocksDB engine. Its handling of historical versions depends on whether the source contains versioned records.

These physical migration capabilities should be distinguished from `surreal db fix`, which is a legacy 1.x-to-2.x operation, and from the 3.x-to-4.0 database upgrade-check procedure.

### 9.4 Encryption-aware inspection

```
# Inspect cipher and key identifiers without a decryption key
skv enc-info path/to/db

# Inspect an encrypted datastore with an authorized key file
skv inspect path/to/db --key-file mydb.key

# Equivalent environment-based approach
SKV_KEY_FILE=mydb.key \
  skv scan path/to/db --prefix "user:"
```

`SKV_KEY_FILE` contains a filesystem path, not the raw key material.

Encryption is a capability of the underlying SurrealKV storage library. The `skv` CLI supplies inspection and decryption-key handling; it is not a general-purpose encryption configuration system for every SurrealDB server deployment.

Primary reference:

[https://github.com/surrealdb/surrealkv](https://github.com/surrealdb/surrealkv?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

## 10. Additional schema and SurrealQL command-line tools

### 10.1 `surqlfmt`: SurrealQL formatter

Status: independently distributed CLI | Implementation: npm package, not Rust

`surqlfmt` is the official file-oriented SurrealQL formatter. It is relevant to the Rust CLI ecosystem because it provides a distinct static-analysis stage for `.surql` files, even though its own distribution is not Rust-based.

```
npm install -g @surrealdb/surql-fmt

# Check formatting without modifying files
surqlfmt --check 'schema/**/*.surql'

# Rewrite files
surqlfmt --write 'schema/**/*.surql'

# Format standard input
surqlfmt --stdin < query.surql

# Configure style
surqlfmt \
  --indent 4 \
  --indent-char tab \
  --max-line-length 120 \
  query.surql
```

Its functionality is limited to formatting whitespace, indentation, keyword casing, and related source presentation. It neither connects to a database nor verifies schema references or query behavior.

The useful distinction is between its three execution modes: printing formatted source, modifying files with `--write`, and checking existing files with `--check`.

Primary reference:

[https://surrealdb.com/docs/reference/cli/formatter/overview](https://surrealdb.com/docs/reference/cli/formatter/overview?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

### 10.2 `surrealdb-migrations`: archived community Rust CLI

Status: archived in favor of SurrealKit

`surrealdb-migrations` was a community Rust migration manager. Its upstream README now explicitly states that the project is archived in favor of SurrealKit, the official migration tool.

It remains relevant when interpreting legacy migration repositories and existing database histories, but should not be presented as an equally maintained alternative.

```
cargo install surrealdb-migrations

# Initialize a project
surrealdb-migrations scaffold template empty

# Generate a schema definition
surrealdb-migrations create schema post \
  --fields title,content,author

# Generate a timestamped migration and its reverse
surrealdb-migrations create AddProduct --down

# Apply migrations
surrealdb-migrations apply

# Revert to a migration
surrealdb-migrations apply \
  --down 20230604_164501_AddAdminUser

# Inspect state
surrealdb-migrations status
```

Its command groups include `scaffold`, `create`, `remove`, `apply`, `list`, and `status`, with optional/preview branching and schema-diff facilities.

The migration runner supports forward and backward files, reapplication, dry-run SQL output, migration selection, and tag-based filtering.

A representative dry run:

```
surrealdb-migrations apply \
  --dry-run --output \
  --tags v2 \
  --exclude-tags old
```

The tool historically separates schema files, event definitions, and data migration scripts. That differs from SurrealKit's module ownership and desired-state reconciliation model.

Existing migration records from this tool should not be assumed compatible with SurrealKit's metadata tables or rollout history. SurrealDB 4.0 compatibility likewise cannot be inferred from the fact that older versions of the CLI compiled successfully.

Primary reference:

[https://github.com/odonno/surrealdb-migrations](https://github.com/odonno/surrealdb-migrations?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

## 11. Rust test, benchmark, and profiling executables

SurrealDB's upstream repositories contain command-line utilities that are particularly relevant to engine developers and engineers evaluating database behavior. These are distinct from the publicly distributed `surreal db` administration CLI.

### 11.1 SurrealQL language-test runner

The main SurrealDB repository contains a Rust executable under `language-tests`.

Its purpose is to execute files written in SurrealQL, compare actual results with expected values, and test language behavior across storage backends.

From the relevant repository directory:

```
# Run all language tests
cargo run run

# Run tests matching a path substring
cargo run run graph

# Generate missing expected results
cargo run run --results accept

# Replace existing expected results
cargo run run --results overwrite

# Select the SurrealKV backend
cargo run run --backend surrealkv
```

The expectation model is more sophisticated than simple text comparison.

A test can declare expected results and errors through special structured comments inside a `.surql` file. It supports exact value comparisons, expected parser failures, approximate matching of nondeterministic identifiers, and SurrealQL expressions that evaluate whether a result is acceptable.

GraphQL test cases are also supported, including schema setup, variables, and operation selection.

The `--backend` option can run supported test cases against in-memory or persistent engines, subject to the Cargo features and database version being built.

Status boundary: this is SurrealDB's internal language-conformance tooling, not a stability-guaranteed third-party testing API. Its main-branch documentation still references RocksDB/TiKV feature combinations that require checking against the actual checkout before using them with a 4.0 build.

Primary reference:

[https://github.com/surrealdb/surrealdb/blob/main/language-tests/README.md](https://github.com/surrealdb/surrealdb/blob/main/language-tests/README.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 11.2 Language-test benchmark runner

The same language-test corpus can be executed as a benchmark suite.

```
# Benchmark a selected workload
cargo make bench -- scans/count

# Quick, lower-precision run
cargo make bench -- scans/count --quick

# Save a baseline
cargo make bench -- scans/count --save

# Export structured benchmark results
cargo make bench -- scans/count --json results.json

# Run against a persistent SurrealKV backend
cargo make bench -- scans/count --backend surrealkv

# Generate a measured-region profile
cargo make bench -- \
  scans/where_integer_in_many_full \
  --profile
```

The runner supports warmup, repeated samples, baseline comparison, JSON results, and statistical measures such as median, mean, variance-related metrics, confidence intervals, and significance testing.

It also supports dataset matrices. A single query definition can be benchmarked with different database configurations, such as indexed versus unindexed fields.

The profiling path uses `samply` to collect flamegraphs. It specifically excludes dataset setup from the measured region, preventing expensive initial imports from dominating profiles intended to analyze query execution.

Some benchmarks can compare against baselines stored remotely in SurrealDB through `--store-url`, subject to the required build feature.

These features make the harness useful for investigating engine regressions and query-execution changes, but benchmark results remain specific to the measured workload, backend, release, and hardware.

Primary reference:

[https://github.com/surrealdb/surrealdb/blob/main/language-tests/README.md](https://github.com/surrealdb/surrealdb/blob/main/language-tests/README.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 11.3 `crud-bench`: configurable multi-database benchmarking

`crud-bench` is another Rust CLI maintained under the SurrealDB GitHub organization.

It can run CRUD and query workloads against SurrealDB and several other database engines, including standalone SurrealKV.

```
# Inspect CLI options
cargo run -r -- -h

# Run a SurrealDB workload
cargo run -r -- \
  --database surrealdb \
  --samples 100000 \
  --clients 12 \
  --threads 24 \
  --random

# Connect to an existing SurrealDB server
cargo run -r -- \
  -d surrealdb \
  -e ws://127.0.0.1:8000 \
  -s 100000 -c 12 -t 24
```

Its main configuration groups are:

\| Option family | Examples |\
\|---|---|\
\| Database choice | `--database`, `--endpoint`, `--image` |\
\| Workload scale | `--samples`, `--clients`, `--threads` |\
\| Key distribution | `--key`, `--random` |\
\| Runtime resources | `--workers`, `--blocking` |\
\| Durability | `--sync`, `--persisted` |\
\| Test phases | `--mode all|load|query` |\
\| Measurement | `--warmup-time`, `--measurement-time`, iteration limits |\
\| Diagnostics | `--pid`, process resource measurements |\
\| Workload configuration | `--config` for applicable query and vector tests |

Several less obvious facilities are valuable.

Split load/query execution. `--mode load` can prepare and retain a dataset, while `--mode query` measures against it later. This avoids conflating database load time with query performance.

Process-aware resource sampling. `--pid` selects the database server process whose CPU, memory, and disk activity should be measured. Without that option, some resource metrics refer to the benchmark process.

Version-specific database adapters. The repository distinguishes SurrealDB 4.x and 3.x adapters, rather than assuming one client implementation can accurately exercise every historical server release.

Vector workloads. The repository documents nearest-neighbor query and index workload families, including exact-search ground truth, recall, filtered search, and parameter sweeps. Individual feature availability must be checked against the implemented code because the project also documents planned workloads.

`crud-bench` can automatically start selected databases through Docker or connect to existing deployments.

Primary reference:

[https://github.com/surrealdb/crud-bench](https://github.com/surrealdb/crud-bench?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1

### 11.4 `tokio-console`: asynchronous runtime diagnostics

The SurrealDB server can be custom-built with Tokio console instrumentation:

```
RUSTFLAGS="--cfg tokio_unstable" \
cargo build --release \
  --bin surreal \
  --features surrealdb-server/tokio-console
```

The separate Rust CLI is installed with:

```
cargo install --locked tokio-console
```

When the custom server is started with `SURREAL_TOKIO_CONSOLE_ENABLED=true`, the console client connects to the configured diagnostic socket, which defaults to `127.0.0.1:6669`.

It exposes async task lifetimes, polling behavior, waiting time, and scheduling stalls.

This is more specialized than ordinary tracing or database query metrics. It helps diagnose behavior inside the Tokio runtime rather than only observing completed database operations.

The feature is not included in standard released SurrealDB binaries or Docker images, and requires both the Cargo feature and Tokio's unstable instrumentation configuration.

Primary reference:

[https://surrealdb.com/docs/manage/observability/tokio-console](https://surrealdb.com/docs/manage/observability/tokio-console?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

## 12. Related deployment and product-specific CLIs

### 12.1 Docker, Helm, and Kubernetes

SurrealDB's container image runs the database executable directly.

Therefore, the container entrypoint normally uses:

```
docker run --rm \
  -p 127.0.0.1:8000:8000 \
  surrealdb/surrealdb:v4.0.0 \
  start memory
```

It does not require the host-style `surreal db start` command hierarchy.

For Kubernetes, SurrealDB publishes Helm chart instructions:

```
helm repo add surrealdb https://helm.surrealdb.com
helm repo update
```

The official single-node chart supports persistent storage, container configuration, and standard Kubernetes lifecycle operations.

The relevant distinction is that Helm and `kubectl` are orchestration CLIs around SurrealDB, not additional SurrealDB database commands.

A persistent single-node deployment should not place several independent SurrealDB pods against one SurrealKV filesystem directory. The official chart describes a single writable owner for that datastore.

Enterprise SurrealDS also has Kubernetes operator tooling for distributed deployment, with separately documented availability and licensing requirements.

Primary references:

[https://surrealdb.com/docs/running/docker](https://surrealdb.com/docs/running/docker?utm_source=chatgpt.com)

[https://surrealdb.com/docs/manage/self-hosted/kubernetes](https://surrealdb.com/docs/manage/self-hosted/kubernetes?utm_source=chatgpt.com)

[https://surrealdb.com/docs/manage/self-hosted/managed-kubernetes](https://surrealdb.com/docs/manage/self-hosted/managed-kubernetes?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

### 12.2 `agent-memory`: separate SurrealDB product CLI

`agent-memory` is an independently distributed CLI for the SurrealDB Agent Memory product, rather than the database itself.

Its command surface includes:

| Group             | Commands and capabilities                                    |
| ----------------- | ------------------------------------------------------------ |
| Memory operations | `remember`, `recall`, `context`, `chat`, `reflect`, `forget` |
| Documents         | `documents upload`, `documents list`, `ingest`               |
| Inspection        | `sessions`, `entities`, `traces`                             |
| Interactive tools | `repl`, `tui`                                                |
| Integration       | `mcp`                                                        |
| Lifecycle         | `login`, `config`, `upgrade`                                 |

The CLI exposes advanced options for memory inference mode, fact scope, document labels, retrieval mode (`hybrid`, `vector`, `bm25`, `graph`), and result limits.

Its `mcp` command prints client configuration for the separate Agent Memory MCP endpoint; it does not start the database CLI's embedded stdio MCP server.

The `tui` provides an interactive inspector with entity and trace views, while `repl` exposes memory operations as terminal commands.

The product and CLI have independent credentials and versioning. Its `SPECTRON_*` environment variable names reflect the product's earlier `spectron` CLI identity.

Primary reference:

[https://surrealdb.com/docs/agent-memory/reference/cli](https://surrealdb.com/docs/agent-memory/reference/cli?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Agent Memory

## 13. How the CLI capabilities fit together

The most important architectural distinction across these tools is which state and guarantees each command owns.

A command can be part of the same developer workflow without sharing its consistency, security, or execution semantics with the next command.

### 13.1 Static analysis, schema application, and runtime validation

| Stage                 | CLI                               | What it verifies or changes                       |
| --------------------- | --------------------------------- | ------------------------------------------------- |
| Formatting            | `surqlfmt --check`                | Textual source consistency                        |
| Parsing               | `surreal db validate`             | SurrealQL parser acceptance                       |
| Static semantics      | `surreal lsp check`               | Schema-aware references, types, and diagnostics   |
| Schema reconciliation | `surrealkit sync`                 | Database definitions and tracked desired state    |
| Versioned migrations  | `surrealkit rollout`              | Explicit schema transitions and migration history |
| Behavioral tests      | `surrealkit test`                 | Query results, constraints, and permissions       |
| Engine conformance    | Language-test CLI                 | Engine-level expected behavior                    |
| Runtime performance   | `crud-bench`, language benchmarks | Observed workload performance                     |

These facilities are complementary. In particular, formatter success does not imply parse validity, parse validity does not imply semantic correctness, and semantic analysis cannot prove the outcome of a transaction.

### 13.2 Data-movement tools have different consistency boundaries

| CLI or interface           | State moved                             | Important limitation                                             |
| -------------------------- | --------------------------------------- | ---------------------------------------------------------------- |
| `surreal db import/export` | Logical definitions and current records | Non-atomic import; historical versions not preserved             |
| `surreal-sync`             | External snapshots and change streams   | At-least-once delivery; connector-specific checkpoint guarantees |
| `surrealkit seed`          | Tracked reference/seed SQL              | Reexecution governed by file hashes                              |
| `skv migrate`              | Physical storage representation         | Offline engine-format operation                                  |
| Cloud backup commands      | Managed instance snapshot               | Asynchronous operation, dedicated-instance availability          |
| Experimental `/kvexport`   | Raw datastore keyspace                  | Server HTTP interface, not a `surreal db` subcommand             |

A useful but easily missed distinction is that physical and logical data movement may preserve different information. Logical exports can redact credentials and omit history; physical copies may contain sensitive keys and engine-specific metadata.

The command or endpoint being available does not itself establish a complete disaster-recovery procedure.

### 13.3 CLI automation and exit-status boundaries

The ecosystem does not provide one universal JSON or error-handling contract.

| Tool           | Automation interface                                                   |
| -------------- | ---------------------------------------------------------------------- |
| Launcher       | `list --json`, version selectors, offline and non-interactive controls |
| Database CLI   | JSON outputs for selected commands; stdin/stdout script execution      |
| Cloud CLI      | Structured JSON, explicit exit-code taxonomy, wait/no-wait controls    |
| Extension CLI  | JSON metadata, dry runs, explicit signer/capability grants             |
| SurrealQL LSP  | Structured diagnostics and severity-based exit statuses                |
| SurrealKit     | Migration state, linting, test JSON output, recoverable rollouts       |
| Surreal Sync   | Checkpoints, retries, ordered apply, configurable failure policy       |
| `skv`          | Low-level diagnostic output and filesystem operations                  |
| Benchmark CLIs | Structured measurements and comparison results                         |

For example, a Cloud CLI timeout may mean a provisioning operation is still running, whereas a Surreal Sync failure typically means its checkpoint has not advanced beyond the failed batch. Those are very different retry semantics.

### 13.4 Rust-specific tooling distinctions

For Rust-oriented systems, the most important CLI integration points are not simply that the executables themselves are written in Rust.

They include:

- `cargo install` and `cargo binstall` as distribution mechanisms for independently versioned tools.
- Cargo features controlling which embedded storage engines and diagnostics are compiled into an executable.
- `wasm32-wasip2` compilation and `.surli` packaging for database extensions.
- `tokio_unstable` and instrumentation features for async runtime inspection.
- Rust-native test and benchmark executables operating directly against database engine implementations.
- Version-specific Rust database adapters and compatibility boundaries in benchmark and migration tools.

Conversely, using the Rust SDK to execute arbitrary queries is not itself a CLI capability. It becomes CLI-relevant only when a command-line program exposes those operations, as SurrealKit, Surreal Sync, and the benchmark tools do.

## 14. Version and availability boundaries

A durable CLI reference needs to keep several distinctions visible.

| Component or feature       | Qualification                                                                              |
| -------------------------- | ------------------------------------------------------------------------------------------ |
| SurrealDB 4.0              | Stable release; CLI behavior differs from 3.x                                              |
| `surreal db upgrade-check` | Run with a suitable 3.3 binary, not the 4.0 database executable                            |
| `surreal db fix`           | Operational for legacy conversion using 2.x binaries only                                  |
| SurrealML CLI              | Removed in SurrealDB 4.0                                                                   |
| `surreal ext`              | 4.0 command surface documented, but standalone publication pending per official guide      |
| SurrealKit 1.0 features    | Beta/development documentation; not all belong to published 0.7.0                          |
| Surreal Sync               | Explicitly not yet stable                                                                  |
| SurrealQL LSP              | Upstream documents component installation; publication needs release-specific verification |
| `surrealdb-migrations`     | Archived in favor of SurrealKit                                                            |
| SurrealKV `skv`            | Separate crate and binary; physical storage operations                                     |
| Language-test harness      | Internal upstream tool rather than a guaranteed external interface                         |
| CRUD Bench                 | Development-focused; some listed workload families remain planned                          |
| Enterprise features        | Separate licensing, deployment, and component availability                                 |

The command examples here were researched from official documentation and upstream source; I did not execute them in a local SurrealDB environment. For version-sensitive automation, the exact installed executable's `--help` output remains the final authority for accepted flags.

## 15. Primary source index

The sources below are direct documentation or upstream repository URLs, suitable for independent research.

| Subject                              | Source                                                                      |
| ------------------------------------ | --------------------------------------------------------------------------- |
| CLI architecture and component index | `https://surrealdb.com/docs/reference/cli`                                  |
| Launcher                             | `https://surrealdb.com/docs/reference/cli/launcher`                         |
| Database command reference           | `https://surrealdb.com/docs/reference/cli/db/commands`                      |
| Database startup flags               | `https://surrealdb.com/docs/reference/cli/db/commands/start`                |
| Database environment variables       | `https://surrealdb.com/docs/reference/cli/db/environment-variables`         |
| Database import                      | `https://surrealdb.com/docs/reference/cli/db/commands/import`               |
| Database export                      | `https://surrealdb.com/docs/reference/cli/db/commands/export`               |
| Cloud CLI                            | `https://surrealdb.com/docs/reference/cli/cloud`                            |
| Cloud command index                  | `https://surrealdb.com/docs/reference/cli/cloud/commands`                   |
| Extension CLI                        | `https://surrealdb.com/docs/reference/cli/ext`                              |
| Extension installation status        | `https://surrealdb.com/docs/explore/cli/extensions`                         |
| SurrealKit                           | `https://github.com/surrealdb/surrealkit`                                   |
| SurrealQL language server            | `https://github.com/surrealdb/surrealql-language-server`                    |
| Surreal Sync                         | `https://github.com/surrealdb/surreal-sync`                                 |
| Sync pipeline and transformations    | `https://github.com/surrealdb/surreal-sync/blob/main/docs/sync-pipeline.md` |
| SurrealKV and `skv`                  | `https://github.com/surrealdb/surrealkv`                                    |
| SurrealQL formatter                  | `https://surrealdb.com/docs/reference/cli/formatter/overview`               |
| Archived migration CLI               | `https://github.com/odonno/surrealdb-migrations`                            |
| Language-test/benchmark harness      | `https://github.com/surrealdb/surrealdb/blob/main/language-tests/README.md` |
| CRUD Bench                           | `https://github.com/surrealdb/crud-bench`                                   |
| Tokio console integration            | `https://surrealdb.com/docs/manage/observability/tokio-console`             |
| Agent Memory CLI                     | `https://surrealdb.com/docs/agent-memory/reference/cli`                     |

## Overall technical picture

The SurrealDB command-line ecosystem offers several distinct classes of functionality.

The core executable and launcher cover versioned database execution, administration, SurrealQL, import/export, security policy, and operational inspection. The Cloud CLI adds independently authenticated infrastructure control, provisioning, scaling, backup, policy management, and machine-readable automation. The development tools add static semantic analysis, schema lifecycle management, phased migrations, seeding, and database-backed tests. The data tools cover external CDC and transformations, physical storage diagnostics, and format migration. Finally, the Rust-native extension and benchmark tools expose executable packaging, provenance checks, language conformance, and performance profiling.

The most distinctive capabilities are not individual command names, but mechanisms such as version-isolated execution, schema ownership and reconciliation, immutable migration manifests, ordered CDC checkpoints, structured semantic diagnostics, capability-granted WASM deployment, physical SSTable inspection, and workload-specific Rust benchmarking.

Those mechanisms are where the CLI ecosystem can replace substantial custom tooling—provided their release status, trust boundaries, data fidelity, and failure semantics match the problem being addressed.