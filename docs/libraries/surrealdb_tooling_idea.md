# SurrealDB: Comprehensive tooling and AI-agent integration guide

Current ecosystem as of October 22, 2026 · SurrealDB 3.3.x

SurrealDB now has a considerably more mature operational and developer tooling ecosystem than it did in earlier releases. Particularly important are the introduction of first-party MCP servers, official AI-agent skills, SurrealKit schema management, SurrealDB Studio, and unified OpenTelemetry observability.

The latest stable database release is SurrealDB 3.3.4, released October 13, 2026. Several important tools are newer than the database features themselves, so relying on older documentation or an agent's pretrained knowledge would miss substantial capabilities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+3



My recommendation is to assemble an agent-native SurrealDB operations toolkit, rather than simply installing the database CLI and giving an agent shell access.

The ideal architecture has four complementary parts: first-party agent interfaces for database introspection and management; conventional CLIs for administration and repeatable operations; specialist language, migration, and diagnostic tools; and an observability stack that converts operational telemetry into evidence the agent can investigate.

The distinction matters because an intelligent programming agent needs more than the ability to execute SurrealQL. It needs to understand the database's actual schema, identify configuration and performance problems, test its hypotheses, deploy safe corrections, and verify their effects.

## 1. Complete tooling landscape

The following is the tooling inventory I would use as the basis for an agent environment. I separate tools that directly operate SurrealDB from tools that provide supporting infrastructure.

### First-party tools and products

| Tool / product                        | Primary capabilities                                                                                           | Value to an LLM agent                                      | Priority                 |
| ------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------ |
| SurrealDB CLI (`surreal db`)          | Server startup, SQL shell, import/export, readiness, validation, version inspection, model and module commands | Deterministic administration and scripting                 | Essential                |
| SurrealDB Cloud CLI (`surreal cloud`) | Provisioning, upgrades, scaling, backups, status, logs, metrics, users, usage                                  | Scriptable managed-instance operations                     | Essential for Cloud      |
| SurrealDB Studio                      | Schema explorer, query editor, record editing, graph visualization, instance monitoring                        | Visual investigation and human verification                | High                     |
| Built-in database MCP                 | Database discovery, schema inspection, queries, records, functions                                             | Direct, structured database access from an agent           | Essential                |
| Hosted SurrealDB MCP                  | Cloud infrastructure, data, logs, metrics, billing, Agent Memory                                               | Cloud operations without custom API wrappers               | Essential for Cloud      |
| Official agent skills                 | SurrealQL, performance, functions, SDKs, CLI, migrations, documentation                                        | Gives agent current domain knowledge and usage conventions | Essential                |
| SurrealQL Language Server             | Parsing, diagnostics, completion, symbol navigation, type information                                          | Programmatic code intelligence                             | Essential                |
| SurrealQL formatter (`surqlfmt`)      | Consistent formatting and style verification                                                                   | Reduces syntax churn and improves generated queries        | Essential                |
| SurrealKit                            | Declarative schema sync, phased rollouts, seeds, type generation, tests                                        | Safe and repeatable schema lifecycle                       | Essential                |
| Surreal Sync                          | Migration and incremental synchronization from external systems                                                | Data ingestion, migration, and reconciliation              | High when importing data |
| Official SDKs                         | Querying, transactions, authentication, subscriptions, embedded engines                                        | Integration and automated testing                          | Essential                |
| Surqlize                              | Type-safe TypeScript ORM/query builder                                                                         | Safer application-level query authoring                    | Conditional              |
| SurrealDB Agent Memory                | Persistent memory, provenance, recall, reflection                                                              | Operational knowledge retention across agent sessions      | Optional                 |
| SurrealDB ML and modules              | ML model import/export, WebAssembly extensibility                                                              | Inspecting or developing advanced database capabilities    | Workload-specific        |

The official CLI now uses a component-based architecture, with `surreal db` and `surreal cloud` versioned independently. SurrealKit and the formatter remain separate packages.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+3



### Supporting infrastructure

| Area                | Recommended tools                                                        | What they contribute                                  |
| ------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------- |
| Runtime management  | Docker, Docker Compose, systemd                                          | Local and single-node operations                      |
| Kubernetes          | `kubectl`, Helm, KIND, Enterprise operator where applicable              | Deployment, inspection, restarts, resource management |
| Metrics             | Prometheus, Grafana, Alertmanager                                        | Time-series collection, dashboards, alerts            |
| Logging             | OpenTelemetry Collector, Loki                                            | Structured log ingestion and searching                |
| Tracing             | OpenTelemetry Collector, Tempo or Jaeger                                 | Request and operation correlation                     |
| Profiling           | Tokio console, operating-system profilers                                | Advanced async and process diagnostics                |
| Load testing        | k6, Vegeta, SurrealDB's `crud-bench`                                     | Reproducible performance experiments                  |
| Storage diagnostics | `iostat`, `pidstat`, `vmstat`, `iotop`, `df`, filesystem tools           | I/O, CPU, RAM and disk pressure                       |
| Security            | Vault or equivalent secrets manager, TLS tooling, audit/SIEM integration | Least privilege, credential handling and auditability |
| CI/CD               | GitHub Actions, GitLab CI or equivalent                                  | Schema/test/deployment gates                          |

SurrealDB provides an official Helm repository at `https://helm.surrealdb.com`. Its Community single-node RocksDB model must use one database-owning pod and persistent volume; running multiple independent pods against the same RocksDB files is not a high-availability configuration. Self-hosted distributed deployments require the appropriate Enterprise architecture.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



## 2. The native SurrealDB capabilities most worth exposing to an agent

### A. Administration: the `surreal` CLI

The command surface is much richer than basic startup and query execution.

| Command family                                     | Agent use                                                       |
| -------------------------------------------------- | --------------------------------------------------------------- |
| `surreal version`, `install`, `upgrade`, `use`     | Detect, install and pin component versions                      |
| `surreal db start`                                 | Start a server with selected storage, auth and runtime settings |
| `surreal db sql`                                   | Execute SurrealQL and inspect results                           |
| `surreal db validate`                              | Syntax-check `.surql` files before deployment                   |
| `surreal db is-ready`                              | CI and service-readiness checks                                 |
| `surreal db export` / `import`                     | Logical backups, migration and restore rehearsals               |
| `surreal db mcp`                                   | Start a local, embedded MCP database over stdio                 |
| `surreal db module`                                | Build and run WebAssembly modules                               |
| `surreal db ml`                                    | Import/export supported ML models                               |
| `surreal cloud instance`                           | Get, list, create, resize, upgrade, pause, resume, delete       |
| `surreal cloud instance metrics/logs/status/usage` | Diagnose managed instances                                      |
| `surreal cloud instance backup ...`                | Manage Cloud backups and policies                               |
| `surreal cloud org/team/invite/token`              | Organization and access management                              |
| `surreal cloud catalog`                            | Discover supported versions, regions and instance types         |
| `surreal cloud --json`                             | Produce machine-readable results for automation                 |

Two cautions matter. First, the documented `surreal mod` standalone component has not yet been published; use `surreal db module` where supported. Second, `surreal db fix` is not a general-purpose SurrealDB 3.x repair utility. The current 3.x command is not implemented, so an agent must not treat it as a safe way to repair production storage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



For Cloud, there is also an authentication distinction: the CLI's personal access tokens cannot perform certain control-plane writes, even when scopes appear permissive. Those operations require the supported interactive authentication route. An agent should detect a `403`, report the credential limitation, and not repeatedly retry it as if the instance were broken.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### B. Native MCP: two different deployments

This is probably the most useful new part of the ecosystem for your stated purpose.

SurrealDB Cloud MCP

Hosted control plane + database

`https://mcp.surrealdb.com`

Manages Cloud instances, data, organization access, monitoring, usage, costs and Agent Memory.

Embedded database MCP

Self-hosted or local data plane

`http://127.0.0.1:8000/mcp`

Exposes queries, schema information, record operations and functions against an existing SurrealDB server.

The database MCP interface exposes these tool names in current releases:

`query`, `gql`, `graphql`, `select`, `create`, `insert`, `upsert`, `update`, `delete`, `relate`, `run`, `list`, `use`, `info`.

That is enough to give an LLM direct access to schema metadata and database operations without building a bespoke SurrealQL connector. Table and field `COMMENT` text is also surfaced through schema introspection, making descriptive schema metadata especially valuable for agent reasoning.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Critical distinction: `surreal db mcp` over stdio opens its own embedded database, with owner-level access. It does not connect to an already-running remote instance. For an agent working on the application database, connect to that server's `/mcp` HTTP endpoint instead.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



For a secure agent, provide read-only database credentials for routine investigation and separate elevated credentials for explicitly approved changes. MCP tool annotations are helpful, but they are not a substitute for database permissions.

### C. SurrealQL language tooling

The first-party language server deserves considerably more attention than a normal syntax-highlighting extension.

Its capabilities include syntax and semantic analysis, schema-aware completions, hover information, symbol navigation, and editor diagnostics. The project can now be installed as a component of the SurrealDB launcher:

```
surreal install lsp

# Programmatic static checks
surreal lsp check queries/

# Language-server stdio interface for an editor or agent
surreal lsp stdio
```

This is a useful division of responsibilities:

- Language server: static diagnostics and code intelligence.
- `surreal db validate`: database-provided syntax validation.
- SurrealKit tests: actual schema, behavior and permission correctness.
- Execution plans: runtime performance evaluation.

None of these replaces the others. The language server is particularly helpful because it lets an agent interrogate built-in function signatures and query structure without repeatedly executing exploratory statements.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



I would also install `@surrealdb/surql-fmt` and enforce a formatting check in CI.

### D. SurrealKit: schema management, migrations and validation

This would be a central component of the implementation.

The latest SurrealKit prerelease I verified is 1.0.0-beta.6, released October 2, 2026. It provides:

| Feature                                      | Why it matters                                                       |
| -------------------------------------------- | -------------------------------------------------------------------- |
| `surrealkit init`                            | Creates a conventional database project layout                       |
| `surrealkit sync`                            | Synchronizes declarative schema files to local/disposable databases  |
| `surrealkit sync --watch`                    | Applies development changes during iteration                         |
| `surrealkit typegen`                         | Generates machine-readable JSON schema and TypeScript types          |
| `surrealkit seed`                            | Controlled test and reference data                                   |
| `surrealkit test`                            | Tests SQL expectations, schema behavior, permissions and endpoints   |
| `surrealkit rollout baseline`                | Establishes a baseline for an existing database                      |
| `surrealkit rollout plan`                    | Produces a reviewed migration manifest                               |
| `surrealkit rollout lint`                    | Checks manifest integrity and dependencies without executing changes |
| `surrealkit rollout up --dry-run`            | Shows planned rollout execution                                      |
| `surrealkit rollout start/complete/rollback` | Phased deployment and recovery                                       |
| `surrealkit rollout status`                  | Diagnoses migration history and pending work                         |

The standout capability is the phased rollout mechanism. Changes can be planned, frozen to checksummed files, introduced in a non-destructive expansion phase, and completed after an application cutover.

This is significantly better for an agent than asking it to freely issue `DEFINE TABLE`, `REMOVE FIELD` or `REMOVE INDEX` statements against a live database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



I would make SurrealKit the authoritative schema deployment path, using raw SurrealQL only for discovery, testing and controlled exceptional repairs.

One specific trap: `surrealkit sync` can prune managed definitions that disappeared from source files. Therefore, unrestricted sync against a shared production database is inappropriate. Also, schema changes involving types and data may not be fully reversible merely by restoring the previous definition.

### E. Built-in SurrealQL diagnostic commands

An agent should learn and use these natively, because they provide more relevant evidence than generic database commands.

| Diagnostic                                | Purpose                                             |
| ----------------------------------------- | --------------------------------------------------- |
| `INFO FOR ROOT`                           | Global definitions and users                        |
| `INFO FOR NS`                             | Namespace configuration                             |
| `INFO FOR DB`                             | Tables, functions, indexes and database definitions |
| `INFO FOR TABLE ...`                      | Fields, indexes, events, permissions                |
| `INFO FOR INDEX ...`                      | Index definition and related state                  |
| `EXPLAIN <statement>`                     | Inspect query planning                              |
| `EXPLAIN FORMAT JSON <statement>`         | Machine-friendly query-plan output                  |
| `EXPLAIN ANALYZE FORMAT JSON <statement>` | Execution measurements                              |
| `SHOW CHANGES ...`                        | Investigate changefeed activity, where configured   |
| `REBUILD INDEX ...`                       | Administrative index maintenance                    |
| `DEFINE INDEX ... CONCURRENTLY`           | Online index-building workflow where supported      |

For query optimization, I would have the agent investigate index selection, record-ID locality, graph traversal shape, full-text indexes, vector indexes, filtering, sorting and intermediate row counts.

A useful pattern is:

```
EXPLAIN FORMAT JSON
SELECT * FROM person
WHERE email = "example@example.com";
```

Then, in a staging environment or with a safely bounded read:

```
EXPLAIN ANALYZE FORMAT JSON
SELECT * FROM person
WHERE email = "example@example.com";
```

`ANALYZE` runs the query to gather actual execution information. Importantly, SurrealDB warns that the execution-plan format can change, so an agent-side parser should be version-aware and tolerant of unfamiliar operators rather than treating the JSON as a permanent stable API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



## 3. Monitoring and troubleshooting: the observability stack

For self-hosting, my preferred stack is:

Recommended logical architecture. Prometheus and the OpenTelemetry collector can be configured independently or together.

Since SurrealDB 3.1, Community and Enterprise share unified metrics, traces and logs. Prometheus can scrape `/metrics`; OpenTelemetry can push signals to a collector using `SURREAL_TELEMETRY_PROVIDER=otlp`. SurrealDB's repository includes a development Compose stack using Grafana, Prometheus, Tempo and Loki, making this much easier to bootstrap.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### Metrics that should be accessible to the agent

| Diagnostic family                 | Questions an agent should answer                        |
| --------------------------------- | ------------------------------------------------------- |
| Query/statement duration          | Which query family has become slower?                   |
| Query errors                      | Which statements or API paths are failing?              |
| Transaction conflicts/retries     | Is concurrency causing avoidable contention?            |
| Process CPU/RAM                   | Is a bottleneck compute-related or memory-related?      |
| HTTP/RPC request activity         | Is load increasing, failing or backing up?              |
| Live-query activity               | Are subscriptions consuming excessive resources?        |
| Index and storage metrics         | Is the storage or index layer contributing to latency?  |
| Network activity                  | Are payloads or transfer rates unexpectedly high?       |
| MCP/GraphQL metrics               | Is an agent-facing or API-facing interface problematic? |
| Audit/slow-log queue health       | Is diagnostic information being lost?                   |
| Distributed consensus/replication | Is the cluster healthy? Enterprise/Scale only           |

The full metrics reference provides the actual metric names and labels. The agent should discover the version-specific catalog rather than hardcoding assumed metrics.

An important permissions detail: unauthenticated access to `/metrics` exposes only a restricted public allowlist. Full metrics require root-level access. I would therefore have a privileged collector scrape internally and expose a separately secured, sanitized Prometheus query interface to the coding agent, rather than handing the agent root credentials.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### Audit logs versus operational logs

These should not be conflated.

Community Edition provides ordinary server logging, metrics, tracing and a slow-query counter. Enterprise additionally supports dedicated audit and slow-query record pipelines, including structured NDJSON files, redaction, optional hash chaining, pipeline health metrics and optional OTLP log forwarding.

Dedicated Enterprise audit logging is off by default. Setting its environment variables on a Community build does not magically enable it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



For an agent, I would normalize both ordinary logs and Enterprise records into an investigation schema with timestamp, instance, namespace, database, error class, operation and correlation identifier.

That enables an investigation such as: “What changed shortly before p95 graph traversal latency increased, which queries are responsible, and did the most recent schema rollout create or remove an index?”

### Lower-level debugging

When metrics suggest an engine-level issue, useful auxiliary tooling includes:

- Tokio console: inspect async task activity, scheduling and poll behavior on a trusted diagnostic deployment.
- `perf` and system profilers: investigate CPU hotspots and native execution.
- `iostat`, `pidstat`, `vmstat`, `iotop`: identify disk contention, memory pressure and process saturation.
- Docker/Kubernetes diagnostics: inspect pod termination, restarts, OOM kills, resource limits, volumes and network configuration.
- k6 / Vegeta / `crud-bench`: reproduce and measure regressions under controlled load.

These are not all SurrealDB-specific, but they are essential to distinguish a query-planning issue from infrastructure bottlenecks.

## 4. Storage, deployment, security, recovery and data movement

### Storage-engine selection and deployment

| Storage / topology      | Operational implications                                   | Agent tooling                                       |
| ----------------------- | ---------------------------------------------------------- | --------------------------------------------------- |
| In-memory / SurrealMX   | Great for disposable tests; no durable restart persistence | Ephemeral test harness                              |
| RocksDB                 | Recommended single-node on-disk engine in SurrealDB 3.3    | Filesystem, compaction, I/O, snapshots, capacity    |
| SurrealKV               | Beta in SurrealDB 3.3; worth evaluating for embedded use   | Engine settings, memory/durability testing          |
| Distributed storage     | Available through Cloud Scale and self-hosted Enterprise   | Cluster metrics, node health, consensus/replication |
| Embedded SDK deployment | Application and database share a process                   | Application profiling and engine configuration      |
| SurrealDB Cloud         | Managed infrastructure and backups                         | Hosted MCP, Cloud CLI, Studio                       |

Although SurrealKV 1.0 has appeared as a storage-engine release, SurrealDB's documentation identifies SurrealDB 3.4 as the forthcoming integration that makes it the preferred on-disk choice. I would therefore not silently switch a SurrealDB 3.3 production instance from RocksDB to SurrealKV.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The operational agent should maintain a topology manifest defining the storage engine, instance type, version, data path, persistence, authentication configuration and monitoring endpoints. That prevents the model from proposing a distributed-cluster remedy for a single-node deployment, or a RocksDB maintenance action for a different backend.

### Backup, restore and upgrades

The minimum reliable operational setup should include logical exports, scheduled storage-level backups or managed snapshots, restoration testing and a defined recovery objective.

Important SurrealDB 3.3 details:

- Logical exports do not necessarily preserve historical revisions or all security material.
- In current 3.3 releases, exported access signing keys are redacted; restore procedures must reconstruct them securely from a secrets manager.
- Major/minor storage migrations can limit in-place downgrade options.
- A backup is not operationally verified until it has been successfully restored and checked.

I would make the agent produce a restore-verification report, not simply run an export command and declare the database backed up.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### Security and configuration

The agent should be able to inspect configuration and permissions but should not independently control the credentials that authorize its own escalation.

Include diagnostics for `DEFINE USER`, `DEFINE ACCESS`, table and field permissions, namespace/database roles, API capability restrictions, TLS, network bind settings and audit policies.

A particularly useful addition is to run permission-matrix tests in SurrealKit for the application’s principal types, rather than merely reading the declared rules.

### Data migration: Surreal Sync

Surreal Sync is more capable than its early migration examples suggest.

The official repository currently describes full or incremental ingestion options for MongoDB, MySQL, MariaDB, PostgreSQL (including trigger and logical-replication variants), Neo4j, Kafka and JSONL, plus snapshot or CDC paths for systems including Snowflake, SQL Server and BigQuery. It also supports an optional transform stage and checkpoints/watermarks.

However, the project still explicitly describes itself as under active development and not stable. I would use it as a specialized data-ingestion tool with reconciliation and restart testing, rather than an automatically trusted production replication layer.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

surrealdb/surreal-sync · GitHub



For an intelligent agent, give Surreal Sync a wrapper that reports source/target counts, last successful checkpoint, lag, rejected records, type conversions and idempotency-test results.

### Connectivity and integrations

SurrealDB publishes official SDKs for Rust, JavaScript, Python, Go, .NET, Java, Kotlin, PHP, Swift and Mojo. Its ecosystem also includes Airbyte, Dagster, Fivetran and n8n integrations, along with AI framework adapters such as LangChain and LlamaIndex.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



One newer feature deserves a warning: SurrealDB 3.3 has an optional PostgreSQL wire-protocol interface. This can permit connections from clients such as `psql`, but it does not make SurrealDB fully PostgreSQL-compatible. The query dialect and system catalogs differ, so generic database administration and introspection tools should not be assumed to work correctly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



For the agent’s primary database interface, prefer native SurrealQL over attempting to force PostgreSQL-oriented tooling onto the database.

## 5. Recommended agent-native architecture

I would not expose every binary as a separate unrestricted tool, nor build a large MCP server duplicating SurrealDB's own functionality.

Instead, use three logical interfaces: native database MCP, a small operations MCP adapter, and the ordinary coding-agent environment with skills and language tooling.

Recommended logical separation of agent knowledge, database access, operational diagnostics and controlled mutations.

### Interface 1: official tools, essentially unchanged

Use the native MCP server to perform normal schema and data introspection. Use SurrealDB's hosted MCP for Cloud control-plane work.

Install all 10 official agent skills, but preferentially surface the relevant ones for each task:

| Official skill                | When to load it                              |
| ----------------------------- | -------------------------------------------- |
| `surrealql`                   | Querying and schema definitions              |
| `surrealql-performance`       | Index and query tuning                       |
| `surrealql-functions`         | Function selection and signature lookup      |
| `surrealkit`                  | Schema lifecycle, type generation, testing   |
| `surrealdb-vector`            | Vector indexes, distance and KNN tuning      |
| `surrealdb-python`            | Python application integration               |
| `surrealdb-js`                | JavaScript/TypeScript integration            |
| `surrealdb-cli`               | Administrative CLI workflows                 |
| `surrealdb-docs`              | Current official documentation retrieval     |
| `surrealdb-agent-memory-docs` | Agent Memory integration and troubleshooting |

The official repository is [https://github.com/surrealdb/agent-skills](https://github.com/surrealdb/agent-skills?utm_source=chatgpt.com). Its contents are more complete than the shorter introductory SurrealDB agent-setup page, which still describes a three-skill starter set.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### Interface 2: a thin custom `surreal-ops` MCP server

This is where I would invest the limited amount of custom engineering.

Rather than simple command wrappers, expose diagnostic capabilities that combine evidence across multiple systems.

| Proposed custom tool       | Implementation                                   | Agent result                                       |
| -------------------------- | ------------------------------------------------ | -------------------------------------------------- |
| `inventory_environment`    | CLI, topology/config files, server metadata      | Actual versions, editions, engines and permissions |
| `snapshot_schema`          | Native introspection and SurrealKit JSON typegen | Canonical current schema                           |
| `compare_schema`           | Git definitions + live DB + migration snapshots  | Drift report                                       |
| `diagnose_query`           | EXPLAIN + bounded ANALYZE + index metadata       | Query bottlenecks and recommendations              |
| `inspect_index`            | `INFO`, index definitions, workload evidence     | Index status and potential changes                 |
| `query_metrics`            | Prometheus API or Cloud metrics                  | Time-windowed measurements                         |
| `search_logs`              | Loki API or Cloud logs                           | Filtered relevant errors and events                |
| `trace_operation`          | Tempo/Jaeger API                                 | Cross-service operation path                       |
| `inspect_storage`          | Filesystem, engine configuration, system metrics | Capacity, I/O or memory bottlenecks                |
| `inspect_migrations`       | SurrealKit rollout status and manifests          | Migration health                                   |
| `plan_schema_change`       | SurrealKit rollout planning/linting              | Reviewed proposed migration                        |
| `run_database_tests`       | SurrealKit + integration tests                   | Test outcomes and failures                         |
| `verify_backup`            | Backup inventory + isolated restore test         | Recovery readiness                                 |
| `benchmark_workload`       | Load test against disposable instance            | Comparative throughput/latency                     |
| `execute_approved_rollout` | Gated SurrealKit runner                          | Controlled deployment with evidence                |

This gives the agent tools that can actually answer operational questions, not merely print raw CLI output.

For example, `diagnose_query` should collect the database version, schema hash, parameterized query, execution plan, relevant indexes, timings, row counts and recent metric windows. It should return both structured results and a brief interpretation, with links or identifiers for the original evidence.

### Interface 3: declarative project knowledge

A programming agent should not have to rediscover the operating conventions on every session.

I would bundle everything into an ordinary version-controlled repository structure:

```
project/
├── AGENTS.md
├── .agents/
│   └── skills/
│       └── ...                # Official SurrealDB skills
├── database/
│   ├── surrealkit.toml
│   ├── setup.surql
│   ├── schema/
│   ├── rollouts/
│   ├── snapshots/
│   ├── seed/
│   └── tests/
│       ├── suites/
│       └── fixtures/
├── ops/
│   ├── topology.yaml
│   ├── policy.yaml
│   ├── versions.lock
│   ├── mcp/
│   │   └── surreal-ops/
│   ├── observability/
│   │   ├── compose.yaml
│   │   ├── dashboards/
│   │   └── alerts/
│   ├── benchmarks/
│   └── runbooks/
└── .github/
    └── workflows/
        ├── database-check.yml
        └── database-rollout.yml
```

This is a proposed layout, not a claim that SurrealDB requires these exact paths. Keep SurrealKit's generated `database/` conventions intact and place additional operational files alongside them.

In `AGENTS.md`, define the execution rules clearly. For example:

```
## SurrealDB operation policy

- Verify the actual database version before using a feature.
- Obtain live schema information before changing queries or definitions.
- Use SurrealQL, not PostgreSQL SQL assumptions.
- Run LSP checks, format validation, and SurrealKit tests.
- Use `surrealkit sync` only on disposable development databases.
- Use reviewed SurrealKit rollouts for shared environments.
- Never issue production DDL or DML without explicit authorization.
- Run performance experiments on staging unless production-safe.
- Record baseline and post-change query/telemetry evidence.
- Never expose secrets or production records in agent output.
- Treat backup success and restore verification as separate checks.
```

## 6. How to install and initialize the toolchain

On Ubuntu, I would start with the following sequence, assuming the `surreal` launcher, Node/npm, and Rust toolchain are available.

```

# Inspect and pin the database/launcher versions
surreal version

# Cloud control plane, only if using SurrealDB Cloud
surreal install cloud

# Native SurrealQL static analysis
surreal install lsp
surreal lsp check database/schema/

# All official LLM-agent skills
npx skills add surrealdb/agent-skills

# Formatter, pinned through the project package lock
npm install --save-dev @surrealdb/surql-fmt

# SurrealKit: install the tested 1.0 beta line explicitly.
# Verify the selected release artifact first; avoid silently
# installing older stable 0.x through a default Cargo install.
cargo binstall surrealkit --version 1.0.0-beta.6

# Run these after SurrealKit project initialization
surrealkit rollout lint
surrealkit test

```

For a new project, run `surrealkit init` first and review its generated configuration. For an existing project, baseline the current database carefully before generating migrations.

For remote MCP configuration, the minimal Cloud connection is:

```

{
  "mcpServers": {
    "surrealdb-cloud": {
      "url": "https://mcp.surrealdb.com"
    }
  }
}

```

For self-hosting, configure an additional MCP HTTP endpoint for the running database, and use the agent client's supported secure credential mechanism rather than committing Bearer tokens or passwords into JSON files.

The most useful startup routine is to run `surreal version`, collect `INFO FOR DB`, check the SurrealKit rollout state, test database health, and query recent metrics. Cache a compact environment report with a timestamp and schema fingerprint. Refresh it after a change or when investigating unexpected behavior.

## 7. Governance: what the agent may do autonomously

Even a highly capable coding agent benefits from explicit separation between diagnosis and mutation.

| Operation                                                  | Suggested control                                |
| ---------------------------------------------------------- | ------------------------------------------------ |
| Discover schemas and metadata                              | Autonomous, read-only                            |
| Inspect logs, metrics and traces                           | Autonomous, sanitized access                     |
| Produce query plans                                        | Autonomous within resource budgets               |
| Execute bounded read-only samples                          | Autonomous in staging; cautious in production    |
| Generate schema changes and rollout plans                  | Autonomous                                       |
| Run tests and apply migrations to disposable DB            | Autonomous                                       |
| Change development data                                    | Autonomous within designated sandbox             |
| Deploy staging changes                                     | Authorized workflow                              |
| Deploy production schema changes                           | Human approval and verified rollback/backup plan |
| Drop records/tables, alter permissions, rotate credentials | Explicit high-risk approval                      |
| Delete Cloud instances or backups                          | Separate privileged operator action              |

The main design principle is to separate the reasoning agent from the privileged executor. Give the agent precise inspection capabilities and approved workflows rather than unrestricted root credentials.

For ongoing troubleshooting, require it to produce an evidence packet: observed problem, baseline, relevant logs/metrics, query plan, hypothesis, proposed change, test results, approval status and post-change verification.

### Example: how the agent should diagnose a slowdown

1. Establish the affected database, query family, time window and observed latency regression.
2. Correlate Prometheus metrics, structured logs and recent deployment/migration history.
3. Capture current schema/index definitions and inspect the SurrealQL execution plan.
4. Reproduce the workload in staging, ideally with representative data and parameters.
5. Compare alternative record-ID patterns, index strategies or rewritten queries.
6. Generate a SurrealKit rollout plan if schema changes are needed, run tests, and verify before/after results.

That is qualitatively more powerful than an agent that simply runs `EXPLAIN` and guesses at improvements.

## 8. Gaps worth addressing, rather than adding more generic tooling

The ecosystem already handles basic database operations well. The largest opportunities for custom tooling are at the boundaries between systems.

Cross-system causal diagnosis. SurrealDB has the telemetry and query introspection, but a reusable higher-level tool that automatically correlates deployments, schema changes, query plans, resource saturation and error spikes would be particularly valuable.

Schema drift and structural understanding. SurrealKit provides state management and type generation; the agent should augment this with structured comparison of declared versus actual table/field/index/access definitions, along with impact analysis of graph relationships and dependent queries.

Performance regression testing. Introduce representative query corpora and capture execution-plan fingerprints, timing distributions, memory consumption, output row counts and index usage by database version. Avoid strict comparisons against unstable plan JSON internals.

Recovery drills. A scheduled, isolated restore test that validates schema, permissions, representative queries and data integrity is more valuable than merely checking whether backup files exist.

Version-aware feature detection. As shown by the fast-moving 3.3 ecosystem, command names, language-server behavior, migration options and storage recommendations can change. The agent needs an actual version/capability inventory instead of applying memorized assumptions.

Structured domain conventions. Graph edge semantics, record-ID design, index strategies, and table/field comments are particularly important for SurrealDB. Encoding these in the schema and agent skills reduces guesswork and improves the consistency of generated queries.

### My preferred bundle

For a sophisticated engineering environment, I would use SurrealDB 3.3.4, the native MCP server, all official agent skills, the SurrealQL language server, `surqlfmt`, SurrealKit 1.0 beta.6, and the full Prometheus/OpenTelemetry stack. Add the Cloud MCP and CLI when using managed instances. Add Surreal Sync only where migration or ongoing external ingestion is actually needed.

Build a small, security-conscious `surreal-ops` MCP adapter on top of that, concentrating on high-value operations that SurrealDB's native tools do not already combine.

This offers the best balance of capability, operational safety, minimal custom code, and agent effectiveness.

## Official references worth putting into the agent's knowledge base

| Area                        | Reference                                                                                                                                                               |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CLI reference               | [https://surrealdb.com/docs/reference/cli](https://surrealdb.com/docs/reference/cli?utm_source=chatgpt.com)                                                             |
| Cloud administration        | [https://surrealdb.com/docs/explore/cli/cloud](https://surrealdb.com/docs/explore/cli/cloud?utm_source=chatgpt.com)                                                     |
| Built-in MCP                | [https://surrealdb.com/docs/build/ai-agents/mcp/embedded](https://surrealdb.com/docs/build/ai-agents/mcp/embedded?utm_source=chatgpt.com)                               |
| Hosted Cloud MCP            | [https://surrealdb.com/docs/build/ai-agents/mcp](https://surrealdb.com/docs/build/ai-agents/mcp?utm_source=chatgpt.com)                                                 |
| Official skills             | [https://github.com/surrealdb/agent-skills](https://github.com/surrealdb/agent-skills?utm_source=chatgpt.com)                                                           |
| Language server             | [https://github.com/surrealdb/surrealql-language-server](https://github.com/surrealdb/surrealql-language-server?utm_source=chatgpt.com)                                 |
| SurrealKit                  | [https://github.com/surrealdb/surrealkit](https://github.com/surrealdb/surrealkit?utm_source=chatgpt.com)                                                               |
| Surreal Sync                | [https://github.com/surrealdb/surreal-sync](https://github.com/surrealdb/surreal-sync?utm_source=chatgpt.com)                                                           |
| Observability overview      | [https://surrealdb.com/docs/manage/observability](https://surrealdb.com/docs/manage/observability?utm_source=chatgpt.com)                                               |
| Metrics catalog             | [https://surrealdb.com/docs/manage/observability/metrics](https://surrealdb.com/docs/manage/observability/metrics?utm_source=chatgpt.com)                               |
| OpenTelemetry configuration | [https://surrealdb.com/docs/manage/observability/configuration](https://surrealdb.com/docs/manage/observability/configuration?utm_source=chatgpt.com)                   |
| Explain/query plans         | [https://surrealdb.com/docs/reference/query-language/statements/explain](https://surrealdb.com/docs/reference/query-language/statements/explain?utm_source=chatgpt.com) |
| Deployment/storage models   | [https://surrealdb.com/docs/manage/self-hosted/deployment-models](https://surrealdb.com/docs/manage/self-hosted/deployment-models?utm_source=chatgpt.com)               |
| Kubernetes                  | [https://surrealdb.com/docs/manage/self-hosted/kubernetes](https://surrealdb.com/docs/manage/self-hosted/kubernetes?utm_source=chatgpt.com)                             |
| Backups and recovery        | [https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery](https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery?utm_source=chatgpt.com)         |

The single most important implementation decision is to make diagnostics, schema lifecycle and operational evidence available as structured, version-aware agent tools. With that foundation, the LLM can move from answering SurrealDB questions to reliably diagnosing and improving an actual deployment.