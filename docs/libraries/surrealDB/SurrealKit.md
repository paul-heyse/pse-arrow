# SurrealKit — Comprehensive Rust Library and Tooling Overview

Research date: November 8, 2026 | Ecosystem: Rust / SurrealDB | Maintainer: SurrealDB

## 1. Executive overview

SurrealKit is SurrealDB's official schema management, migration, seeding, type generation, and database testing toolkit. It is both a standalone CLI and an embeddable Rust library.

Its distinguishing characteristic is the combination of two complementary schema-management approaches:

- Declarative synchronization: Maintain a desired schema in SurrealQL (`.surql`) files or embedded Rust data, automatically reconciling the database with those definitions.
- Controlled schema rollouts: Convert changes into ordered, reviewable migrations with separate expansion and contraction phases, explicit steps, rollback handling, and execution-state tracking.

SurrealKit extends beyond conventional migration tooling. It also provides schema introspection, generated types, declarative security tests, data seeding, project templates, and multi-database schema organization.

For Rust applications, it is particularly relevant when schema lifecycle management should run inside the application process, including applications using embedded SurrealDB rather than a separately managed database server.

The central architectural distinction is that SurrealKit manages the database's structure and lifecycle, while the official `surrealdb` Rust SDK performs ordinary runtime database operations. SurrealKit is not an ORM, general-purpose query builder, or replacement for the SDK.

Primary sources:

- Repository: [https://github.com/surrealdb/surrealkit](https://github.com/surrealdb/surrealkit?utm_source=chatgpt.com)
- Official documentation: [https://surrealdb.com/docs/manage/schema-migration](https://surrealdb.com/docs/manage/schema-migration?utm_source=chatgpt.com)
- Rust integration: [https://surrealdb.com/docs/manage/schema-migration/library](https://surrealdb.com/docs/manage/schema-migration/library?utm_source=chatgpt.com)
- Crate: [https://crates.io/crates/surrealkit](https://crates.io/crates/surrealkit?utm_source=chatgpt.com)

### Version status

Latest stable

# 0.7.0

June 11, 2026

Latest identified prerelease

# 1.0.0-beta.6

October 2, 2026

The 1.0 prerelease introduces significant additional capabilities, including modular schema management, multiple deployment targets, tracked seeding, expanded testing, and improvements to rollout execution.

Because the 1.0 functionality materially changes SurrealKit's capabilities, this overview covers both the stable foundation and the advanced 1.0 prerelease functionality, explicitly distinguishing them where consequential.

The official documentation primarily describes `1.0.0-beta.1`, while upstream releases and migration notes include changes through `beta.6`. Accordingly, version-specific claims need to be interpreted against the identified release rather than assuming every documented behavior exists in 0.7.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



## 2. Capability map

| Capability              | What SurrealKit provides                                                       | Principal value                          |
| ----------------------- | ------------------------------------------------------------------------------ | ---------------------------------------- |
| Schema synchronization  | Content-hash tracking, `DEFINE` execution, reconciliation, pruning, watch mode | Declarative schema management            |
| Embedded schemas        | `embed_schema!`, `EmbeddedSchemaFile`, `Sync` builder                          | Self-contained Rust deployments          |
| Production migrations   | Phased rollouts, manifests, snapshots, rollback, recovery                      | Controlled schema evolution              |
| Data migrations         | Custom SQL steps, assertions, ordered execution                                | Schema-and-data transitions              |
| Schema organization     | Independent modules, dependencies, named targets (1.0 beta)                    | Modular, multi-database management       |
| Seeding                 | Ordered SurrealQL files, change tracking (1.0 beta), forced execution          | Reference and fixture data               |
| Type generation         | Database introspection, schema JSON, TypeScript generation                     | Schema-aware code generation             |
| Database testing        | SQL, permissions, metadata, behavior, and HTTP assertions                      | Executable database contracts            |
| Development integration | File watching, project templates, Vite plugin                                  | Short development feedback loops         |
| Automation              | CLI, Docker, GitHub Actions, Rust library                                      | Reproducible deployment and verification |

These capabilities form three complementary layers:

Schema source of truth

SurrealQL files · Embedded Rust schemas · Modules · Template variables

Sync

Immediate desired-state reconciliation

Rollouts

Controlled, phased migration

SurrealDB

Actual schema · Managed-object catalog · Migration history

Seeding

Data initialization

Typegen

Schema extraction

Testing

Contract verification

The database holds the actual state and SurrealKit's tracking metadata. Files, manifests, and embedded definitions express the intended state and changes. The distinction between those representations is central to understanding its behavior.

## 3. Declarative schema synchronization

### 3.1 Content-based reconciliation

SurrealKit's Sync mechanism reconciles schema definitions rather than blindly replaying every SQL file on each invocation.

Its normal operation consists of discovering schema definitions, identifying changes using stored hashes, applying new or modified definitions, and removing definitions no longer owned by the active schema source.

SurrealKit maintains an internal `__entity` metadata table for tracking. Consequently, changes can be detected without rebuilding the entire database schema on every invocation.

The tracked definition families include:

| Family                            | SurrealQL definitions                   |
| --------------------------------- | --------------------------------------- |
| Core data model                   | `TABLE`, `FIELD`, `INDEX`               |
| Programmability and configuration | `FUNCTION`, `PARAM`, `CONFIG`           |
| Search                            | `ANALYZER`                              |
| Security                          | `ACCESS`, `USER`                        |
| Other database objects            | `BUCKET`, `SEQUENCE`, `MODEL`, `MODULE` |

This scope is important for advanced SurrealDB applications. Schema management need not be limited to tables and fields; it can encompass database-defined functions, search components, authorization definitions, and other objects represented by supported `DEFINE` statements.

SurrealKit does not implement the runtime semantics of those objects. It manages their definitions; SurrealDB implements their behavior.

Source: [https://surrealdb.com/docs/manage/schema-migration/sync](https://surrealdb.com/docs/manage/schema-migration/sync?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 3.2 Pruning, ownership, and drift

Pruning removes previously managed definitions that are absent from the desired schema.

This makes Sync a genuine reconciliation mechanism, rather than a tool that only adds or changes definitions. It also introduces an important destructive operation.

SurrealKit offers several controls:

| Control                      | Behavior                                                          |
| ---------------------------- | ----------------------------------------------------------------- |
| `prune(false)`               | Preserve definitions absent from the supplied schema              |
| `dry_run(true)`              | Inspect intended changes without applying them                    |
| `fail_fast(true)`            | Stop processing after an error                                    |
| `allow_shared_prune(true)`   | Explicitly permit destructive reconciliation on a shared database |
| `allow_all_statements(true)` | Permit SQL other than schema definitions                          |

For filesystem-based synchronization, the 1.0 prerelease also refuses an empty schema selection by default. This protects against treating an incorrect working directory or missing schema files as an instruction to delete managed objects. An intentional empty selection requires `--allow-empty-prune`.

There is a consequential distinction between object-level tracking and file-level tracking. When non-`DEFINE` statements are enabled, SurrealKit cannot reliably determine the managed schema objects represented by arbitrary SQL. It therefore falls back to file-level hashes and disables individual-object catalog tracking for that run.

That behavior makes `allow_all_statements` substantially more than a permissive parsing option. It changes what the synchronization engine can reason about.

Source: [https://surrealdb.com/docs/manage/schema-migration/sync](https://surrealdb.com/docs/manage/schema-migration/sync?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 3.3 Compile-time schema embedding in Rust

One of SurrealKit's most useful Rust-specific capabilities is `embed_schema!`.

The procedural macro reads `.surql` files at compile time and generates a Rust module containing their contents.

```
surrealkit::embed_schema!();

async fn initialize(
    db: &surrealkit::Surreal<surrealkit::engine::any::Any>,
) -> surrealkit::anyhow::Result<()> {
    embedded_schema::sync(db).await?;
    Ok(())
}
```

The generated `embedded_schema` module provides a `SCHEMA` slice and an asynchronous `sync` function.

This changes the deployment boundary: the application can carry its schema without requiring a `database/schema` directory at runtime.

It is valuable for single-binary applications, embedded databases, hermetic deployment artifacts, and integration tests.

A custom source directory can be supplied to the macro, relative to the invoking Cargo package's manifest.

Source: [https://surrealdb.com/docs/manage/schema-migration/embed-schema-macro](https://surrealdb.com/docs/manage/schema-migration/embed-schema-macro?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 3.4 The `Sync` builder and programmatically supplied schemas

The lower-level `Sync` builder accepts a slice of `EmbeddedSchemaFile` definitions. Those definitions can come from the embedding macro, explicit Rust statics, or another producer.

An illustrative 1.0 API usage:

```
use surrealkit::{EmbeddedSchemaFile, Sync};

static SCHEMA: &[EmbeddedSchemaFile] = &[
    EmbeddedSchemaFile {
        path: "schema/customer.surql",
        sql: include_str!("../schema/customer.surql"),
    },
];

async fn sync_schema(
    db: &surrealkit::Surreal<surrealkit::engine::any::Any>,
) -> surrealkit::anyhow::Result<()> {
    Sync::embedded(SCHEMA)
        .prune(false)
        .fail_fast(true)
        .run(db)
        .await
}
```

Here, `path` and `sql` have distinct meanings. The path is a stable identity key; it does not have to identify a file that exists at runtime. The SQL field contains the definition text to apply.

Keeping the tracking key stable is important. Renaming a key can cause SurrealKit to interpret the old source as deleted and the new one as added, potentially activating pruning.

A further architectural implication follows from the interface: SurrealKit can consume schema definitions that originate in a programmatic schema model or generator, provided they are materialized into the required SurrealQL representation. That makes SurrealKit a possible execution and reconciliation layer beneath a higher-level schema system, without requiring SurrealKit itself to provide that higher-level model.

The `Sync` builder is filesystem-independent during execution, and its setup work is handled internally.

Source: [https://surrealdb.com/docs/manage/schema-migration/library](https://surrealdb.com/docs/manage/schema-migration/library?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 4. Production rollouts and controlled schema evolution

### 4.1 Expand–contract migration model

Rollouts provide a different execution contract from declarative Sync. Rather than immediately reconciling the database to a target schema, a rollout records an ordered sequence of changes and executes them through explicit lifecycle phases.

Start — expansion

Introduce new schema elements while preserving the old application's required definitions. Execute compatible preparation steps.

Application cutover

New and old consumers coexist temporarily. Any necessary data transformation and compatibility handling must be planned explicitly.

Complete — contraction

Remove obsolete objects and finalize the target catalog after dependent consumers no longer require them.

Rollback is available before completion, through explicitly defined rollback behavior.

Phased migrations can enable rolling application deployments without breaking either application version. They do not inherently guarantee zero downtime: schema compatibility, query behavior, data transformations, index-building costs, and application release sequencing remain consequential.

For an existing database, `surrealkit rollout baseline` captures the current schema into snapshots. Subsequent planning compares those snapshots with the changed source definitions and generates a reviewable TOML manifest.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: [https://surrealdb.com/docs/manage/schema-migration/rollouts](https://surrealdb.com/docs/manage/schema-migration/rollouts?utm_source=chatgpt.com)

### 4.2 Declarative manifests and programmatic Rust rollouts

In 1.0.0-beta.6, Rust callers can construct rollouts directly using `RolloutSpec`, `RolloutStep`, `RolloutPhase`, and `Rollout`.

A representative example, adapted from the upstream library documentation:

```
use surrealkit::{
    EmbeddedSchemaFile, Rollout, RolloutCompatibility,
    RolloutPhase, RolloutSpec, RolloutStep,
    Surreal,
};
use surrealkit::engine::any::Any;

static TARGET: &[EmbeddedSchemaFile] = &[
    EmbeddedSchemaFile {
        path: "schema/account.surql",
        sql: "DEFINE TABLE account SCHEMAFULL;",
    },
];

async fn deploy(db: &Surreal<Any>) -> anyhow::Result<()> {
    let spec = RolloutSpec::builder("20261108__account")
        .name("Introduce account")
        .compatibility(RolloutCompatibility::Phased)
        .step(RolloutStep::apply_schema(
            "create_account",
            RolloutPhase::Start,
            "DEFINE TABLE account SCHEMAFULL;",
        ))
        .step(RolloutStep::run_sql(
            "backfill_accounts",
            RolloutPhase::Complete,
            "UPDATE account SET active = true WHERE active = NONE;",
        ))
        .build();

    let rollout = Rollout::new(spec, TARGET);

    rollout.start(db).await?;
    // Application cutover happens separately.
    rollout.complete(db).await?;

    Ok(())
}
```

This example demonstrates the API, not a universally safe migration plan. In particular, a real rollout involving updated consumers normally needs a considered rollback definition and may need to backfill before cutover rather than during completion.

A `RolloutStep` has exactly one action:

| Step constructor  | Purpose                                               |
| ----------------- | ----------------------------------------------------- |
| `apply_schema`    | Apply inline SurrealQL schema definitions             |
| `apply_files`     | Apply specified `.surql` files or frozen file content |
| `run_sql`         | Execute SQL, including data transformations           |
| `assert_sql`      | Enforce an expected query result                      |
| `remove_entities` | Explicitly remove designated schema objects           |

This allows schema changes, data transformations, and verification barriers to form one recorded operation. It is substantially more expressive than automatic schema-diff application alone.

The target `EmbeddedSchemaFile` slice tells SurrealKit the intended managed schema catalog once the rollout completes. For fully self-contained step definitions, an empty target slice is permitted.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 4.3 Ordered rollout histories from Rust

The 1.0.0-beta.6 library also exposes `Rollouts`, a higher-level interface for loading and replaying migrations from a project directory.

```
use surrealkit::{Rollouts, Surreal};
use surrealkit::engine::any::Any;

async fn apply_pending(db: &Surreal<Any>) -> anyhow::Result<()> {
    let rollouts = Rollouts::load("./database")?;

    let pending = rollouts.pending(db).await?;
    println!("{:?}", pending.pending);

    let report = rollouts.up(db).await?;

    if report.waiting.is_some() {
        // Complete only after the required application cutover.
        rollouts
            .clone()
            .complete_newest(true)
            .up(db)
            .await?;
    }

    Ok(())
}
```

`Rollouts::up` is not equivalent to indiscriminately executing every file. It processes previously unapplied migrations in order, completes the older migrations, and normally leaves the newest one at `ready_to_complete`. That distinction preserves the opportunity for application cutover.

This provides a library-native equivalent to the CLI's `surrealkit rollout up` workflow and reduces the need to implement a separate migration-history engine in a Rust service.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 4.4 Execution state, locking, and recovery

SurrealKit records rollout state in `__rollout`. The execution lifecycle distinguishes states such as `running_start`, `ready_to_complete`, `running_complete`, `completed`, `running_rollback`, and `rolled_back`.

An active rollout blocks starting another rollout against the same tracked database scope. Execution state is recorded so interrupted operations can be inspected and, where supported, resumed or repaired.

The CLI includes `rollout status` and `rollout repair`. These become relevant when an interruption occurs after SQL has executed but before the rollout metadata is finalized.

A particularly important beta.4 change replaced wholesale replacement of the managed-object catalog with incremental, bounded catalog updates. This reduces the amount of metadata work as schema catalogs grow and improves recovery after partial catalog-write failures.

There are limits to these guarantees:

- `run_sql` data changes must be designed to tolerate re-execution; rollout tracking does not automatically make arbitrary SQL idempotent.
- Rollback is not available after completion, and a reversal of schema definitions is not necessarily a reversal of data mutations.
- `Rollout::abandon` is an administrative escape hatch that updates tracking state; it does not undo already executed SQL.
- A successful rollback requires appropriate rollback steps. Merely labeling a migration as reversible is insufficient.

These distinctions are essential when assessing recovery semantics. SurrealKit is an execution-state and migration-coordination system, not a general transactional undo engine for arbitrary transformations.

Sources:

[https://surrealdb.com/docs/manage/schema-migration/rollouts](https://surrealdb.com/docs/manage/schema-migration/rollouts?utm_source=chatgpt.com)

[https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



### 4.5 Frozen SQL, planning constraints, and drift

The 1.0 prerelease planner produces migration manifests tied to schema snapshots and hashes. In beta.6, the associated SQL is also frozen alongside each rollout and checked for subsequent modification.

This matters because a migration executed in production needs a stable definition independent of later edits to the current schema source. It separates a historical operation from the latest desired-state files.

Additional safeguards include:

- `rollout lint` for offline manifest and history validation;
- source/target schema hashes to detect unexpected divergence;
- requirements for explicit handling of modified or renamed definitions;
- `--allow-modified` for planner cases involving non-additive definition changes;
- a `definition_only` reversibility designation when reversing definitions would not restore transformed data.

A renamed field or changed `VALUE`, `DEFAULT`, `ASSERT`, or permission expression is not necessarily semantically reversible. The planner's caution around these changes is therefore valuable, but it cannot determine every application-level compatibility implication.

One subtle beta.6 behavior protects database sequences: SurrealKit no longer implicitly overwrites an existing `DEFINE SEQUENCE`, because doing so could reset its counter. A changed sequence definition may therefore require an explicit data or administrative migration rather than ordinary Sync.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



## 5. Schema modules, dependencies, and deployment targets

Introduced in the 1.0 prerelease series

### 5.1 Independently managed schema modules

A major architectural addition in SurrealKit 1.0 is the ability to split a schema into independently tracked modules.

Configuration uses named `[schema.<name>]` sections, with optional dependencies:

```
[schema.core]

[schema.billing]
depends_on = ["core"]

[schema.analytics]
depends_on = ["core"]
```

Each module can have separate schema, snapshot, rollout, and seed directories.

Dependencies determine application order. Selecting a module normally includes its dependencies, cycles are rejected before application, and independent modules use deterministic ordering.

More importantly, SurrealKit partitions its `__entity` tracking records by module. A sync of `billing` therefore does not automatically prune objects managed by `analytics`.

That is the key capability distinction: modules are not merely a directory-layout convenience. They represent independent ownership and reconciliation boundaries within a database.

Module identifiers must remain stable. Renaming a module changes its tracking partition and can cause existing managed definitions to appear stale. SurrealKit explicitly warns against renaming an already deployed module without a deliberate migration.

Source: [https://surrealdb.com/docs/manage/schema-migration/modules-and-targets](https://surrealdb.com/docs/manage/schema-migration/modules-and-targets?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.2 Named database targets

A target represents a named connection destination, potentially specifying a host, namespace, database, authentication configuration, and an allowed selection of schema modules.

For example:

```
[target.production]
host = "wss://database.example"
ns = "application"
db = "production"
pass_env = "DB_PRODUCTION_PASSWORD"
schemas = ["core", "billing"]
primary = true
```

Target definitions inherit unspecified connection settings from the ambient configuration.

Authentication secrets are referenced through environment variables. The configuration rejects literal password entries and fails before connecting if a required target password variable is missing.

The CLI can select individual modules and targets or process a matrix:

```
surrealkit sync --schema billing
surrealkit sync --target production
surrealkit sync --all
surrealkit sync --all --keep-going
```

A matrix execution processes module-target pairs sequentially. It is not a distributed transaction. If the fourth database fails, the first three may already have changed.

The 1.0-beta.1 documentation says rollout target selection is not supported, but this became outdated during the prerelease series. By beta.6, several rollout execution commands support one explicit target; rollout status can inspect multiple targets, whereas planning and linting remain local, offline operations.

This is an example where the tagged migration notes are more authoritative than the older feature documentation.

Sources:

[https://surrealdb.com/docs/manage/schema-migration/modules-and-targets](https://surrealdb.com/docs/manage/schema-migration/modules-and-targets?utm_source=chatgpt.com)

[https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 6. Data seeding and compile-time embedded seeds

### 6.1 Seed tracking and execution semantics

SurrealKit's seeding mechanism is intentionally separate from schema synchronization. Sync manages definitions; seeding executes data initialization statements.

In the 1.0 prerelease, seeds are tracked by stable path and source-content hash in `__seed`. Unchanged seed files are skipped, while new or modified files execute again.

This is a substantial change from 0.7, where every invocation re-executed seed files.

A normal filesystem project uses `database/seed/*.surql` in lexicographic filename order. A forced run bypasses the tracking check:

```
surrealkit seed
surrealkit seed --force
```

Two implications are worth distinguishing.

First, a seed is execution-deduplicated, not necessarily mathematically idempotent. Editing a file or forcing a rerun can still duplicate records or conflict with existing records unless its SurrealQL is written appropriately.

Second, the hash is computed from source content before template substitution. Changing a variable's value does not by itself trigger a seed rerun.

Source: [https://surrealdb.com/docs/manage/schema-migration/seeding](https://surrealdb.com/docs/manage/schema-migration/seeding?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 6.2 `embed_seed!` and the `Seed` builder

SurrealKit 1.0.0-beta.6 provides a compile-time seed embedding macro analogous to `embed_schema!`.

```
surrealkit::embed_seed!();

async fn initialize_data(
    db: &surrealkit::Surreal<surrealkit::engine::any::Any>,
) -> anyhow::Result<()> {
    embedded_seed::seed(db).await?;
    Ok(())
}
```

The generated module embeds seed SQL in the executable, allowing data initialization without runtime seed files.

For greater control, the Rust API exposes `EmbeddedSeedFile` and `Seed`:

```
use surrealkit::{EmbeddedSeedFile, Seed, Surreal};
use surrealkit::engine::any::Any;

static SEEDS: &[EmbeddedSeedFile] = &[
    EmbeddedSeedFile {
        path: "seed/regions.surql",
        sql: "UPSERT region:us SET name = 'United States';",
    },
];

async fn seed_data(db: &Surreal<Any>) -> anyhow::Result<()> {
    Seed::embedded(SEEDS).run(db).await?;

    // Explicitly reapply all supplied seed definitions:
    // Seed::embedded(SEEDS).force(true).run(db).await?;

    Ok(())
}
```

As with `EmbeddedSchemaFile`, the embedded seed's `path` identifies the source for tracking; it does not need to exist in the deployed filesystem.

`Seed::from_dir("database")` and the `seed(db, folder, vars)` function provide filesystem-oriented alternatives.

An important integration distinction is that the CLI's ordinary seed discovery reads top-level `.surql` files, while `embed_seed!` documents recursive discovery beneath `database/seed`. Those two source-selection rules should not be assumed identical.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 7. Schema introspection and type generation

SurrealKit's type generation is a separate capability family built on live database introspection.

Rather than inferring types solely from checked-in `.surql` files, it queries the database's actual definitions and constructs a structured schema representation.

### 7.1 Structured `SchemaTypes` output

The Rust API exposes:

```
use surrealkit::typegen::{
    generate,
    render_typescript,
    write_typescript,
};
```

`generate(&db).await?` returns a `SchemaTypes` document describing the namespace, database, tables, fields, functions, and relevant parameter definitions.

SurrealKit can serialize this representation to JSON, with a default CLI destination of `database/types/schema.json`. Internal tracking tables are excluded.

```
let schema = generate(&db).await?;
let typescript = render_typescript(&schema)?;
```

This creates a useful architectural boundary:

SurrealDB

Actual definitions

SchemaTypes

Structured model

JSON

Machine-readable schema representation

TypeScript

Generated client types

For Rust tooling, the structured representation can be more valuable than its bundled TypeScript emitter. It gives custom generators or validators a typed introspection input without requiring them to scrape human-readable `INFO` output themselves.

That is an architectural possibility, not evidence that SurrealKit ships a general-purpose Rust code generator.

### 7.2 TypeScript emission

The bundled TypeScript generator targets the SurrealDB JavaScript SDK v2. It generates table interfaces, typed record identifiers, and mapped field types.

Generation can be configured using the `[typegen]` section of `surrealkit.toml`, including an output path, filename, and optional formatter. In beta.6, the `typescript` setting can identify the output `.ts` file directly.

With configured output, synchronization can also trigger regeneration after schema changes.

There are two important limitations:

- Native Rust model generation is not a documented bundled emitter. Rust consumers can use `SchemaTypes` as input to their own code generation, but should not assume SurrealKit generates Rust `struct`s, `serde` implementations, or Rust SDK query models.
- Target-aware generation is more limited than target-aware synchronization. The documented 1.0 behavior introspects the explicitly selected connection, and a multi-target sync may overwrite a single configured TypeScript output with whichever target runs last.

Also, the generated `schema.json` is a SurrealKit schema representation; it should not be mistaken for a standards-conformant JSON Schema document without checking its structure.

Source: [https://surrealdb.com/docs/manage/schema-migration/typegen](https://surrealdb.com/docs/manage/schema-migration/typegen?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 8. Declarative database and security testing

SurrealKit includes a testing framework for database-level contracts. Rather than embedding all assertions in application tests, it reads declarative TOML suites and provisions an isolated database context for each suite.

The five primary case families are:

| Test kind            | Scope                                                       |
| -------------------- | ----------------------------------------------------------- |
| `sql_expect`         | Query outcomes, expected errors, returned values            |
| `permissions_matrix` | Create, select, update, delete authorization outcomes       |
| `schema_metadata`    | Structural definitions, fields, indexes, metadata           |
| `schema_behavior`    | Computed fields, functions, relations, operational behavior |
| `api_request`        | HTTP response codes, bodies, and headers                    |

Tests can run against named actors representing root credentials, database users, record access authentication, bearer tokens, or supplied headers.

The runner supports suite/case filters, tags, bounded parallelism, fail-fast execution, JSON reporting, test timeouts, and preserving ephemeral databases for debugging.

This is particularly consequential for SurrealDB because authorization and data behavior can be encoded directly in database definitions. Testing only application-level code may not adequately exercise those contracts.

Source: [https://surrealdb.com/docs/manage/schema-migration/testing](https://surrealdb.com/docs/manage/schema-migration/testing?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 8.1 Permission semantics and assertion correctness

`permissions_matrix` is more sophisticated than checking whether a query returned a permission error.

In a database with row-level authorization, a denied operation may yield an empty result instead of a thrown error. SurrealKit 1.0 revised probing behavior to distinguish these cases, including record-specific testing and cleanup of generated test records.

Beta.6 further improves tests involving `UNIQUE` indexes and authenticated records. For example, `record_id = "$auth"` can target the record used to authenticate a test actor, while `equals_auth = "$auth.id"` can compare returned data against the actor's identity.

Another significant correction in 1.0 is that missing assertion paths now fail by default. In 0.7, a mistyped or nonexistent JSON path could silently pass, effectively skipping the intended comparison.

Explicit absence assertions remain possible with `exists = false`.

These changes make the testing framework materially better at detecting false-positive security and data-contract tests.

Sources:

[https://surrealdb.com/docs/manage/schema-migration/testing](https://surrealdb.com/docs/manage/schema-migration/testing?utm_source=chatgpt.com)

[https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 8.2 Testing schema equivalence across Sync and rollouts

This is one of the most important advanced additions in beta.6.

The test runner supports three schema construction modes:

```
[defaults]
schema_from = "both"
```

| Mode       | How the suite database is created                 |
| ---------- | ------------------------------------------------- |
| `sync`     | Apply the current declarative schema              |
| `rollouts` | Replay the migration chain from an empty database |
| `both`     | Run each suite against both representations       |

When rollouts are replayed, SurrealKit can first compare the resulting schema against one built using Sync. The parity check detects differences between current desired state and historical migration-derived state.

That catches a class of failure ordinary migration tests can miss: a definition exists in the current schema files but was never incorporated into a rollout.

It also detects cases where migration replay produces different definitions from direct synchronization, even when each process completes without an obvious SQL error.

The parity check can be disabled through:

```
[rollouts]
parity = false
```

However, parity covers schema definitions, not proof that arbitrary data migrations have identical effects or that both pathways preserve every historical dataset.

Rollout replay also requires a valid migration chain from an empty database. A system whose first migration assumes a preexisting baseline cannot necessarily use the replay mode without additional preparation.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 9. Rust integration and embedded database operation

### 9.1 Cargo features and dependency footprint

SurrealKit is useful as an executable CLI and as a library dependency, but those two roles have different build requirements.

In the beta.6 source, the default feature set is:

```
default = ["kv-mem", "cli"]
```

The relevant features are:

| Feature        | Function                                                |
| -------------- | ------------------------------------------------------- |
| `cli`          | Builds the CLI and includes CLI-specific dependencies   |
| `kv-mem`       | Enables in-memory embedded storage                      |
| `kv-surrealkv` | Enables embedded SurrealKV                              |
| `kv-rocksdb`   | Enables embedded RocksDB                                |
| `embedded`     | Convenience feature enabling all three storage backends |

For a library consumer, disabling default features avoids CLI-only dependencies such as `clap`, `inquire`, and related terminal/build support.

The tagged beta.6 documentation recommends:

```
[dependencies]
surrealkit = {
    version = "1.0.0-beta.6",
    default-features = false,
    features = ["kv-mem"]
}
```

Publication qualification: I verified the `v1.0.0-beta.6` GitHub release and tagged source, but the accessible docs.rs version index still shows 0.7.0 and 1.0.0-beta.1. I could not independently confirm beta.6's availability on crates.io. For an exact, reproducible dependency on the verified beta.6 source, Cargo also supports a pinned Git tag:

```
[dependencies]
surrealkit = {
    git = "https://github.com/surrealdb/surrealkit",
    tag = "v1.0.0-beta.6",
    default-features = false,
    features = ["kv-mem"]
}
```

The default published stable dependency is `surrealkit = "0.7.0"`, with its corresponding older API and feature set.

Sources:

[https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/Cargo.toml](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/Cargo.toml?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-beta.6](https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-beta.6?utm_source=chatgpt.com)

[https://docs.rs/crate/surrealkit/latest](https://docs.rs/crate/surrealkit/latest?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



### 9.2 Embedded versus remote SurrealDB

SurrealKit operates on a SurrealDB client of type `Surreal<Any>`, using the official Rust SDK.

The beta.6 library supports managing databases reached through remote connections as well as in-process engines such as `mem://`, `surrealkv://`, and `rocksdb://`, subject to the corresponding compiled engine capabilities.

Cargo feature unification matters here: if a consuming Rust crate enables an embedded engine through its `surrealdb` dependency, SurrealKit can use that backend without independently enabling the same feature on its own package declaration.

For a standalone CLI, the embedded engine must be included in the CLI build. The published binary's default configuration includes the in-memory backend rather than every persistent storage backend.

A material operational distinction is that embedded persistent datastores generally require exclusive process ownership. Running an external SurrealKit CLI process against an embedded store while an application process has the store open may fail because of datastore locking. Running SurrealKit within the owning Rust process avoids that particular cross-process conflict.

SurrealKit's `connect` helper recognizes embedded endpoints and skips authentication for a fresh embedded datastore, while ordinary remote connections use the configured authentication workflow.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 9.3 Multi-module compile-time embedding

The 1.0 Rust integration includes a less obvious form of `embed_schema!` that can embed several named schema modules.

```
surrealkit::embed_schema!(
    core = "database/modules/core/schema",
    billing = "database/modules/billing/schema",
);
```

This generates a combined entry point as well as module-specific sync entry points:

```
embedded_schema::sync(&db).await?;
embedded_schema::billing::sync(&db).await?;
```

Alternatively, a supplied schema can select its tracking module through the builder:

```
Sync::embedded(BILLING)
    .module("billing")?
    .run(&db)
    .await?;
```

This is different from the configuration-file dependency system. For multi-module embedding, the macro's declarations control execution order; developers must arrange dependencies appropriately rather than assuming a TOML dependency graph is consulted at runtime.

One important Cargo integration issue is explicitly documented upstream. Changes to existing embedded files can trigger recompilation, but adding a new `.surql` file does not necessarily invalidate the procedural macro expansion because the directory listing is not automatically tracked.

The documented workaround is a `build.rs` containing:

```
fn main() {
    println!("cargo:rerun-if-changed=database");
}
```

This matters for correctness, not merely build convenience: without suitable rebuild tracking, an application binary can unintentionally omit a newly introduced schema file.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 9.4 Rust SDK compatibility

The beta.6 release specifies:

| Component                                                | Verified target              |
| -------------------------------------------------------- | ---------------------------- |
| Rust minimum                                             | 1.95                         |
| SurrealDB Rust SDK                                       | 3.3                          |
| Embedded SurrealDB engine                                | 3.3                          |
| Server versions explicitly reported compatible           | 3.2.0 onward in the 3.x line |
| Server versions explicitly tested in the migration notes | 3.2.0, 3.2.4, 3.3.0          |

The distinctions between SDK and server versions matter. A SurrealKit library build is tied to a particular Rust SDK version even when that SDK can communicate with multiple server versions.

SurrealDB 4.0.0 became stable on November 3, 2026, after SurrealKit beta.6 was released. I found no corresponding beta.6 compatibility certification for the SurrealDB 4 server or Rust SDK.

It would therefore be incorrect to assume SurrealKit beta.6 is fully compatible with SurrealDB 4. Likewise, using a direct `surrealdb = "4"` dependency alongside a SurrealKit build using SDK 3.x introduces a potential Rust type and dependency-version boundary that needs explicit resolution.

Sources:

[https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)

[https://surrealdb.com/releases](https://surrealdb.com/releases?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



## 10. Supporting tooling and extensibility

### 10.1 Project templates as reusable capability bundles

`surrealkit init` supports more than generating empty directories. It can assemble a project from a template manifest whose individual features contribute schema files, seed files, test suites, and fixtures.

Templates are described by `template.toml`, with named features and dependency relationships.

The initializer supports local template paths, Git repositories, and pinned Git revisions. It also supports noninteractive selections such as `--feature`, `--yes`, and `--minimal`.

This allows database capability bundles to carry their own definitions and validation artifacts. For example, a reusable access-control feature could provide tables, permissions, initial reference records, and tests as a coordinated unit.

There is an important boundary: template features describe what gets scaffolded into a project. They are not the same mechanism as independently tracked runtime schema modules.

Source: [https://surrealdb.com/docs/manage/schema-migration/templates](https://surrealdb.com/docs/manage/schema-migration/templates?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 10.2 Development watch mode and Vite integration

For filesystem workflows, `surrealkit sync --watch` observes schema changes and runs reconciliation. It supports a configurable debounce interval and can drive generated TypeScript updates.

The companion `vite-plugin-surrealkit` integrates schema synchronization into a Vite development or build workflow. The beta.6 repository documents options including module selection and `reloadOnSync`, with a Vite 8 requirement.

This plugin is not part of the Rust runtime API. It invokes SurrealKit from JavaScript build tooling.

That boundary is useful: a Rust application can use the SurrealKit library directly while a web frontend independently benefits from schema synchronization and generated client types during development.

Source: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 10.3 Automation, deployment, and observability

SurrealKit also provides integration surfaces for delivery systems:

| Integration       | Capability                                                                   |
| ----------------- | ---------------------------------------------------------------------------- |
| CLI               | Direct execution of sync, rollout, seed, typegen, test, and setup operations |
| GitHub Action     | Installs and executes SurrealKit commands                                    |
| Container images  | Packaged CLI execution, including Docker Compose workflows                   |
| JSON test reports | Machine-readable test results for CI                                         |
| `log` facade      | Progress reporting from embedded Rust library execution                      |
| `rollout lint`    | Offline verification of migration manifests and history                      |

For Rust applications, progress events are emitted through the `log` facade. Unless the host application installs a logger, the library remains silent.

For release operations, beta.6 adds a default 30-second connection/sign-in deadline, while long-running rollout SQL steps have no default timeout. Query timeouts can be configured separately.

This is a sensible distinction for potentially expensive schema operations such as index construction, though an unbounded operation still requires external monitoring when used in a deployment pipeline.

Sources:

[https://surrealdb.com/docs/manage/schema-migration/testing/ci-cd](https://surrealdb.com/docs/manage/schema-migration/testing/ci-cd?utm_source=chatgpt.com)

[https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



## 11. Cross-cutting limitations and correctness considerations

Several details materially affect how SurrealKit's capabilities compose.

| Area                                   | Consequential distinction                                                                                                                                                           |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Desired state versus migration history | Sync reconciles current definitions; rollouts preserve ordered historical operations. Neither alone substitutes for the other in every environment.                                 |
| Hash tracking                          | Unchanged source content is skipped even when template variable values change. A changed deployment environment does not automatically mean changed tracking state.                 |
| Text substitution                      | `${VAR}` is textual interpolation, not SurrealDB query parameter binding. Values inserted into identifiers, SQL expressions, or quoted strings need appropriate trust and escaping. |
| Source identity                        | Embedded file paths and module names are persistent ownership identifiers. Renaming them is not necessarily a harmless refactor.                                                    |
| Data migration idempotence             | Tracking prevents ordinary duplicate execution, but a retry after interruption may execute a partially completed custom step again.                                                 |
| Rollback                               | Reversing DDL does not guarantee restoration of transformed records, deleted data, counter positions, access secrets, or external effects.                                          |
| Authentication definitions             | Overwriting a record-access definition without a stable JWT signing key can invalidate existing sessions. Beta.6 emits a warning.                                                   |
| Sequences                              | Beta.6 prevents implicit sequence overwrites to avoid counter resets, but consequently a definition change may not be applied automatically.                                        |
| Multi-target deployment                | The matrix runs sequentially; no atomicity guarantee spans multiple target databases.                                                                                               |
| Type generation                        | The bundled client-type emitter is TypeScript-oriented. Native Rust model generation is not established by the primary documentation.                                               |
| Schema/parser compatibility            | The executing SurrealDB server determines the syntax and semantics accepted for deployed definitions.                                                                               |
| Metadata ownership                     | `__entity`, `__rollout`, and `__seed` are internal management state; arbitrary edits can disrupt reconciliation and historical tracking.                                            |

Two implementation details are particularly worth noting.

First, beta.6 introduced a more robust SurrealQL tokenizer for statement splitting and object identification. It addresses cases such as semicolons inside regex literals, quoted identifiers containing spaces, and `DEFINE` statements separated from their object kind by comments or newlines. Parse errors can now stop processing before a problematic file is applied or pruned, with source locations reported.

Second, the 1.0 migration changed filesystem tracking keys to paths relative to the project folder rather than the process working directory. This reduces differences between local runs, containers, and CI environments, but handwritten embedded keys should follow a consistent convention when they are intended to correspond to filesystem-managed files.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 12. Stable release versus 1.0 prerelease

The major differences are worth consolidating because the newer documentation and tagged source describe functionality not available in the current stable 0.7 line.

| Capability                                | 0.7.0 stable                        | 1.0.0-beta.6 tagged source                                         |
| ----------------------------------------- | ----------------------------------- | ------------------------------------------------------------------ |
| Declarative sync                          | Yes                                 | Yes, expanded safety and tracking                                  |
| Rust embedded schema                      | Yes                                 | Yes, including named multi-module embedding                        |
| Programmatic rollouts                     | Yes                                 | Expanded with ordered `Rollouts` facade and frozen SQL             |
| Phased migration execution                | Yes                                 | Improved history handling, recovery, and sequencing                |
| Named schema modules and targets          | No documented 1.0 implementation    | Yes                                                                |
| Tracked seeds that skip unchanged content | No; seeds rerun                     | Yes                                                                |
| `embed_seed!` and `Seed` builder          | Not in stable 0.7 API               | Yes                                                                |
| Database type introspection               | Yes                                 | Expanded output configuration                                      |
| Declarative test suites                   | Yes                                 | Improved assertions and permissions                                |
| Sync-versus-rollout test parity           | Not available                       | Yes                                                                |
| Empty filesystem-source protection        | Less restrictive                    | Explicit preflight refusal by default                              |
| Sequence-overwrite protection             | Earlier behavior can reset counters | Protected unless explicitly overwritten                            |
| SurrealDB SDK                             | 3.1.x                               | 3.3                                                                |
| Minimum Rust version                      | Older version requirement           | 1.95                                                               |
| Status                                    | Stable published crate              | GitHub prerelease; registry publication not independently verified |

The 1.0 migration documentation also identifies several behavioral changes that can affect existing consumers: removed legacy `DATABASE_*` environment variable aliases, removal of the single-file `seed.surql` fallback, corrected `--folder` handling, stricter test assertion semantics, and changes to lower-level Rust function signatures.

Primary migration reference: [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



## 13. Architectural significance

SurrealKit's capabilities are most interesting when considered together rather than in isolation.

Embedded application lifecycle management. Compile-time schema embedding, compile-time seed embedding, hash-based tracking, and in-process SDK execution can allow a Rust binary to own database initialization without shipping separate SQL deployment artifacts or introducing an external migration process. The value is reduced packaging and operational coordination, conditional on a suitable schema ownership model.

Declarative authoring with controlled deployment. A single desired-state schema can serve rapid development synchronization, while frozen rollout histories provide an auditable route into shared databases. The two mechanisms avoid requiring every development edit to be manually encoded as a historical migration while still permitting staged production changes.

Executable database contracts. Declarative testing, authenticated actors, rollout replay, and schema parity checking jointly make it possible to validate both the intended schema and the process used to produce it. This can eliminate substantial bespoke test harness logic around permissions, database fixtures, migration replay, and schema-drift detection.

Higher-level schema tooling. `EmbeddedSchemaFile`, independent schema modules, `SchemaTypes`, and programmatic rollouts expose interfaces that a domain-specific schema compiler or model-driven system could build upon. SurrealKit can potentially provide the execution, tracking, and verification layers beneath such a system without becoming responsible for the higher-level domain model itself.

The boundaries remain important. SurrealKit does not replace the SurrealDB Rust SDK, does not establish a Rust ORM abstraction, does not guarantee reversible arbitrary data transformations, and does not automatically make sequential cross-database deployment atomic.

### Primary source map

| Subject                                         | Source                                                                                                                                                                                                                        |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository and releases                         | [https://github.com/surrealdb/surrealkit](https://github.com/surrealdb/surrealkit?utm_source=chatgpt.com)                                                                                                                     |
| Tagged beta.6 Rust library reference            | [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/README.md?utm_source=chatgpt.com)   |
| Tagged beta.6 migration and compatibility notes | [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md?utm_source=chatgpt.com)                                 |
| Tagged beta.6 Cargo features                    | [https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/Cargo.toml](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/crates/surrealkit/Cargo.toml?utm_source=chatgpt.com) |
| Stable Rust API documentation                   | [https://docs.rs/surrealkit/latest/surrealkit/](https://docs.rs/surrealkit/latest/surrealkit/?utm_source=chatgpt.com)                                                                                                         |
| Rust integration documentation                  | [https://surrealdb.com/docs/manage/schema-migration/library](https://surrealdb.com/docs/manage/schema-migration/library?utm_source=chatgpt.com)                                                                               |
| Schema synchronization                          | [https://surrealdb.com/docs/manage/schema-migration/sync](https://surrealdb.com/docs/manage/schema-migration/sync?utm_source=chatgpt.com)                                                                                     |
| Phased rollouts                                 | [https://surrealdb.com/docs/manage/schema-migration/rollouts](https://surrealdb.com/docs/manage/schema-migration/rollouts?utm_source=chatgpt.com)                                                                             |
| Modules and targets                             | [https://surrealdb.com/docs/manage/schema-migration/modules-and-targets](https://surrealdb.com/docs/manage/schema-migration/modules-and-targets?utm_source=chatgpt.com)                                                       |
| Seeding                                         | [https://surrealdb.com/docs/manage/schema-migration/seeding](https://surrealdb.com/docs/manage/schema-migration/seeding?utm_source=chatgpt.com)                                                                               |
| Type generation                                 | [https://surrealdb.com/docs/manage/schema-migration/typegen](https://surrealdb.com/docs/manage/schema-migration/typegen?utm_source=chatgpt.com)                                                                               |
| Declarative testing                             | [https://surrealdb.com/docs/manage/schema-migration/testing](https://surrealdb.com/docs/manage/schema-migration/testing?utm_source=chatgpt.com)                                                                               |
| Templates                                       | [https://surrealdb.com/docs/manage/schema-migration/templates](https://surrealdb.com/docs/manage/schema-migration/templates?utm_source=chatgpt.com)                                                                           |

Overall assessment: SurrealKit is substantially more capable than a simple SurrealQL migration runner. Its most distinctive Rust-facing mechanisms are compile-time schema and seed embedding, programmatic phased rollouts, module-isolated schema ownership, structured database introspection, and migration-replay testing. The beta.6 release meaningfully extends this capability set, but its prerelease status and SurrealDB 3.x SDK dependency are material constraints—particularly with SurrealDB 4 now stable.

# SurrealKit — Technical Capability Addendum

Research date: November 8, 2026 | Rust / SurrealDB | Supplement to the original SurrealKit overview

## Scope and version clarification

This addendum documents additional SurrealKit functionality not adequately covered in the original overview. It concentrates on the newer schema compilation architecture, semantic type generation, external emitters, MCP integration, SurrealDB 4 data migrations, rollout-history management, advanced testing, and lower-level Rust integration capabilities.

The primary technical reference is the upstream `v1.0.0-rc.1` tag, dated November 4, 2026. This is newer than the `v1.0.0-beta.6` release covered in the original overview.

An important distinction applies:

| Version              | Verified status                                               | Relevant capabilities                                                                                 |
| -------------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| 0.7.0                | Latest stable registry version verified                       | Established Sync, rollouts, embedded schemas, typegen, and testing                                    |
| 1.0.0-beta.6         | Published GitHub prerelease                                   | Frozen rollouts, enhanced recovery, modules, targets, tracked seeds                                   |
| 1.0.0-beta.8 changes | Present in upstream history                                   | Schema compilation, document v2, external emitters, expanded testing, SDK 4                           |
| 1.0.0-rc.1           | Verified upstream Git tag; registry publication not confirmed | SurrealDB 4-only operation, native field data migrations, compatibility-level management, MCP tooling |

The latest GitHub release listing I could independently verify still identifies beta.6, while docs.rs lists 0.7.0 as stable. Therefore, capabilities introduced after beta.6 are described here as tagged prerelease functionality, not assumed available from an unqualified `cargo install surrealkit`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



For the remainder of this addendum, API descriptions refer to rc.1 unless otherwise indicated. The official documentation site still primarily describes beta.1, so the tagged upstream source and migration guide are the principal references for newer behavior.

Primary sources:

[https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1](https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md?utm_source=chatgpt.com)

## 1. Offline SurrealQL compilation and semantic schema analysis

Introduced in the beta.8 development series

### 1.1 `surrealkit compile`: compilation without a running database server

SurrealKit's newer architecture includes a schema compiler that operates on checked-in SurrealQL definitions without connecting to an existing database.

This is substantially different from ordinary `surrealkit typegen`, which introspects a live database.

`surrealkit compile` performs the following operation:

1. Resolves selected schema modules and their dependencies.
2. Collects their SurrealQL definitions in application order.
3. Applies the definitions to an ephemeral, embedded SurrealDB engine using SurrealKit's synchronization semantics.
4. Introspects the resulting schema through the engine's `INFO` interfaces.
5. Constructs a normalized semantic schema document.
6. Produces configured output artifacts, including external generator outputs.

The important architectural distinction is that SurrealKit uses SurrealDB itself as the semantic interpreter, rather than implementing an independent approximation of SurrealQL's type and definition semantics.

That enables the compiler to observe the database's normalized definitions, defaults, permissions, indexes, and other structural properties, rather than simply extracting declarations from source text.

Typical interfaces include:

```
surrealkit compile
surrealkit compile --check
surrealkit --schema billing compile
surrealkit compile --allow-experimental surrealism
```

A configured schema document version controls the shared JSON output:

```
[typegen]
schema_version = 2
json = "generated/schema.json"
strict = true
engine = "4.0"
```

The compiler can run without database credentials or network access. It does not read normal database connection settings, and its embedded engine prohibits network access and scripting.

Template variables are taken from explicit configuration and `--var` arguments. Environment-sourced `SURREALKIT_VAR_*` values are deliberately excluded to improve reproducibility.

This creates a useful foundation for compile-time schema validation, deterministic generated models, and builds that should not depend on a running database service.

#### Compilation equivalence and limitations

SurrealKit applies definitions as its synchronization engine would, including overwrite preparation and the special treatment of sequences. Consequently, a compiled document is intended to describe the schema that Sync would deploy, rather than what an arbitrary raw SQL execution would produce.

The upstream test suite checks compilation against live introspection using equivalent engine versions.

However, engine-version equivalence matters. In rc.1, the embedded compiler uses SurrealDB 4.0 semantics. A schema accepted by SurrealDB 3.x is not necessarily accepted by 4.0, and differences in normalized definitions may affect the generated document.

The `engine` configuration detects an expected engine-minor mismatch using the `compile-engine-mismatch` diagnostic. It does not emulate an older engine.

Other important boundaries include duplicate definitions, nondeterministic `DEFINE PARAM` expressions, unsupported root- or namespace-level objects, and experimentally gated definitions. These produce diagnostics rather than being silently assumed equivalent.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs?utm_source=chatgpt.com)

### 1.2 Programmatic compilation from Rust

The functionality is not limited to the CLI.

The public `compile` module exposes three important structures:

| Rust API        | Responsibility                                                                      |
| --------------- | ----------------------------------------------------------------------------------- |
| `CompileModule` | Named schema module containing ordered schema files                                 |
| `CompileOpts`   | Template variables, experimental capabilities, and expected engine version          |
| `Compiled`      | Generated semantic v2 document, corresponding v1 representation, and v1 diagnostics |

The public entry point is:

```
surrealkit::compile::compile(
    modules: &[CompileModule],
    opts: &CompileOpts,
)
```

It is asynchronous and returns a `Compiled` result.

The result contains both a `SchemaDocument` (v2) and `SchemaTypes` (v1). This is significant because the original TypeScript interface generator still consumes the v1 representation, while the newer semantic tooling and external generators use v2.

A Rust application can therefore use the compiler directly to obtain a structured model without launching an external `surrealkit` process.

For systems that generate schema definitions programmatically, this provides a potential boundary between an upstream domain-specific schema representation and downstream SurrealDB semantics. The caller still needs to provide valid SurrealQL and explicitly define the desired module composition.

The API does not eliminate the embedded SurrealDB engine dependency or make compilation independent of SurrealDB version semantics.

Source: Tagged Rust implementation, `compile.rs`.

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs?utm_source=chatgpt.com)

## 2. Schema document v2: a semantic interchange format

Opt-in, draft contract in the rc.1 source

Schema document v2 is one of SurrealKit's most consequential newer capabilities.

It is designed as an interchange format between SurrealKit, which understands SurrealQL and database introspection, and downstream generators that need structured database semantics without implementing a SurrealQL parser.

This is not simply a JSON serialization of table names and field types.

### 2.1 Structural and semantic coverage

The document represents a much broader collection of information:

| Domain          | Semantic information represented                                                            |
| --------------- | ------------------------------------------------------------------------------------------- |
| Tables          | Schema mode, relation characteristics, record identifiers, view definitions                 |
| Fields          | Structured kinds, optionality, defaults, computed values, read-only behavior                |
| Validation      | `ASSERT` expressions and recognized constraints                                             |
| Transformations | `VALUE` expressions and whether they read the supplied input                                |
| Relationships   | Record references, relation endpoints, endpoint type restrictions                           |
| Indexes         | Index definitions and associated attributes                                                 |
| Events          | Database event definitions                                                                  |
| Functions       | Arguments, return types, and relevant function definitions                                  |
| Authorization   | Permissions, access definitions, authentication expressions                                 |
| Other objects   | Parameters, analyzers, sequences, buckets, configurations, and supported module definitions |
| Provenance      | Engine version, adapter, generation source, capabilities                                    |
| Diagnostics     | Structured errors, warnings, and informational findings                                     |

SurrealQL kinds are represented as structured trees rather than opaque strings. This allows consumers to distinguish such forms as `option<T>`, `record<a | b>`, typed arrays, sets, tuples, object literals, and typed table or file values.

The schema-document specification provides a JSON Schema using draft 2020-12, exposed in Rust through:

```
surrealkit::typegen::SCHEMA_DOCUMENT_V2_JSON_SCHEMA
```

The format is explicitly versioned so that generators can reject unsupported contract versions rather than guessing at schema meaning.

Source:

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md?utm_source=chatgpt.com)

### 2.2 Field-presence semantics and operation-specific models

An especially valuable capability is the field-presence model. SurrealDB does not treat omission, `NONE`, `NULL`, defaults, computed values, and read-only fields identically across database operations.

Schema document v2 attempts to preserve those distinctions so that generated types can reflect the actual operation being performed.

A representative subset:

| Field declaration         | Read (`SELECT`)    | `CREATE` input     | `MERGE` input                   | `CONTENT` input             |
| ------------------------- | ------------------ | ------------------ | ------------------------------- | --------------------------- |
| `TYPE T`                  | Required           | Required           | Optional                        | Required                    |
| `TYPE option<T>`          | Optional key       | Optional           | Removable                       | Optional; omission removes  |
| `TYPE T \\| null`         | Required, nullable | Required, nullable | Optional, nullable              | Required, nullable          |
| `TYPE T DEFAULT e`        | Required           | Optional           | Optional; default not reapplied | Required                    |
| `TYPE T DEFAULT ALWAYS e` | Required           | Optional           | Resettable                      | Optional; default reapplied |
| `COMPUTED e`              | As declared        | Excluded           | Excluded                        | Excluded                    |
| `READONLY`                | As declared        | May be supplied    | Excluded                        | Excluded                    |

In this model, removable means an explicit `NONE` can remove a stored value, while omission generally leaves it unchanged. Resettable means an explicit `NONE` can cause a `DEFAULT ALWAYS` expression to be reapplied.

These distinctions are not interchangeable with Rust's `Option<T>` or a single nullable field representation.

In particular, a client generator may need to distinguish:

- A key absent from an object.
- A key explicitly holding the database's `NONE` value.
- A key holding `NULL`.
- A key supplied to creation but not accepted during updates.
- A key omitted from `CONTENT`, which has different replacement semantics from `MERGE`.

This creates opportunities for operation-specific Rust request types, validation schemas, and serializers that reflect database semantics more accurately than a single generated record struct.

The schema-document contract also defines behavior for nested fields, `FLEXIBLE` objects, computed values, relation endpoints, record IDs, and schemafull versus schemaless records.

A necessary qualification: these capabilities are semantic information supplied to generators. SurrealKit does not itself generate a complete family of Rust operation-specific structs or serializers. That would require a compatible downstream generator.

### 2.3 Semantic digests and reproducible schema identity

Every v2 document contains a `semanticDigest` derived from canonicalized semantic content using SHA-256.

The digest excludes provenance, diagnostics, source-rendered `DEFINE` text, and selected analysis-only fields, while including semantically relevant properties such as field kinds, expressions, permissions, and authentication-policy expressions.

Its purpose is to represent schema meaning independently of incidental generation details.

This provides several possible capabilities for downstream tooling:

| Application                | How the digest helps                                                 |
| -------------------------- | -------------------------------------------------------------------- |
| Generated-code consistency | Associate generated types with the schema that produced them         |
| Schema comparison          | Detect changes in represented semantics rather than file timestamps  |
| Reproducible builds        | Avoid changing artifacts when the same schema is compiled again      |
| Test verification          | Compare the prepared test database against expected schema identity  |
| Artifact provenance        | Trace generated files to a particular semantic schema representation |

The digest is not a complete database-state hash. It excludes stored records, runtime sequence allocation state, physical database identity, secrets, and some definitions not represented by the contract.

It also is not guaranteed comparable across engine minor versions. Differences in how SurrealDB normalizes and represents definitions can affect the semantic projection. Cross-version comparison requires an explicit compatibility assessment.

### 2.4 Introspection adapters and unsupported definitions

The v2 implementation uses version-qualified adapters over `INFO ... STRUCTURE`, with support for SurrealDB 3.2, 3.3, and 4.0 representations.

The contract recognizes that this internal database interface can change. Unsupported engine versions use a plain-`INFO` fallback with a `structure-unsupported-engine` diagnostic.

Diagnostics have structured severity levels, codes, affected schema locations, source text, and explanatory information.

For rc.1, important limitations include incomplete v2 representation of SurrealDB 4 `DEFINE TYPE` and certain extension metadata. Those cases are reported rather than silently discarded.

The normative specification describes the format as a draft contract. It should not be treated as a permanently frozen interchange standard merely because a JSON Schema and reference implementation exist.

Primary source:

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md?utm_source=chatgpt.com)

## 3. External code generators and managed output publication

External emitter protocol 1, introduced in the beta.8 development series

### 3.1 Generators as isolated subprocesses

SurrealKit introduces an external emitter protocol that separates schema analysis from language-specific generation.

An external emitter is an executable that receives schema document v2 on standard input and returns a structured description of files to generate.

For example:

```
[typegen.emit.rust]
command = ["my-surreal-emitter", "--language", "rust"]
out = "src/generated/surreal"
```

This illustrates a hypothetical custom Rust generator, not a bundled SurrealKit executable.

An emitter receives the normalized schema document rather than raw SurrealQL and therefore does not need to implement the database's parser or introspection protocol.

The process contract includes:

| Interface                     | Contract                                                            |
| ----------------------------- | ------------------------------------------------------------------- |
| Standard input                | UTF-8 JSON containing schema document v2                            |
| Standard output               | Exactly one JSON response with protocol version and generated files |
| Standard error                | Diagnostics and process logging                                     |
| `SURREALKIT_EMIT_PROTOCOL`    | Protocol version                                                    |
| `SURREALKIT_EMIT_NAME`        | Configured emitter identity                                         |
| `SURREALKIT_EMIT_OUT`         | Destination directory, for information only                         |
| `SURREALKIT_EMIT_ACKNOWLEDGE` | Explicitly accepted diagnostic codes                                |

A successful emitter returns a response of this general form:

```
{
  "protocol": 1,
  "files": [
    {
      "path": "models.rs",
      "contents": "// Generated Rust definitions\n"
    }
  ],
  "diagnostics": []
}
```

The paths are relative to the configured output directory. They must be unique, well formed, and free of traversal components. The emitter must not write the output files itself.

This makes the protocol language-neutral. A generator could be written in Rust, Python, TypeScript, or another language, provided it implements the subprocess protocol.

### 3.2 Managed output ownership

SurrealKit, rather than the emitter, owns publication of generated artifacts.

It maintains a manifest named `.surrealkit-emit.json` within each emitter's output directory. This records the files published by that emitter.

That ownership model enables SurrealKit to distinguish generated files from unrelated application files.

The publication mechanism provides several guarantees:

| Mechanism                          | Consequence                                                               |
| ---------------------------------- | ------------------------------------------------------------------------- |
| Temporary files followed by rename | Avoids exposing partially written generated files                         |
| Unchanged-byte detection           | Avoids unnecessary file rewrites and watcher invalidation                 |
| Previous-output manifest           | Identifies files no longer generated                                      |
| Stale-file removal                 | Removes obsolete emitter-owned files                                      |
| Ownership validation               | Refuses to overwrite files it did not generate                            |
| `--check` support                  | Detects missing, modified, and unexpected generated files without writing |

Two emitters cannot own overlapping output directories, and built-in JSON or TypeScript output cannot be placed inside an emitter-owned directory.

This can eliminate substantial bespoke machinery around generated-file ownership, stale-code cleanup, and build-time freshness checks.

It is not a general atomic transaction across an entire directory tree: individual files are replaced atomically, while publication of a collection of files remains a multi-step operation.

### 3.3 Error handling and strict generation

The emitter protocol distinguishes an unsuccessful generator process from a successful generator that reports semantic diagnostics.

In strict mode, an emitter failure prevents publication of the generated outputs. The `compile` command is strict by default, and `--lenient` relaxes diagnostic handling rather than accepting broken emitter execution.

In non-strict `sync` or `typegen` workflows, a failed emitter can leave its previous files untouched while other valid outputs are published.

A `--check` run fails if a required emitter cannot produce its output, regardless of permissive generation settings.

The configuration also supports diagnostic acknowledgements:

```
[typegen]
acknowledge = {
    "param-value-nondeterministic" = "This parameter is informational"
}
```

Acknowledgement does not remove the diagnostic. It records an explicit exception that SurrealKit and compatible emitters can recognize.

### 3.4 Relationship to `effect-surrealkit`

The upstream `effect-surrealkit` package is a reference implementation of this architecture.

It generates Effect v4 schemas and codecs for the SurrealDB JavaScript SDK, including operation-specific record types, relation endpoints, record identifiers, function arguments, and runtime handling of SurrealDB values.

This demonstrates the expressive potential of schema document v2, but it is a TypeScript companion package, not a Rust ORM or Rust model generator.

For Rust, the generalizable capability is the externally specified schema representation and emitter protocol, which can support a custom generator without reproducing SurrealKit's engine-level schema analysis.

Primary sources:

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/external-emitters.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/external-emitters.md?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1/packages/effect-surrealkit](https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1/packages/effect-surrealkit?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com

## 4. First-party MCP server for programming agents

Available in the verified rc.1 source

SurrealKit now implements a Model Context Protocol server for database development and schema management.

This is a separate integration surface from both the standalone CLI and the Rust library API.

It is also distinct from SurrealDB's hosted Cloud MCP service, which manages cloud instances and cloud-account resources. SurrealKit's MCP server is oriented toward a local SurrealKit project, its schema files, and the associated database lifecycle.

### 4.1 Runtime and transport

The MCP server is launched with:

```
surrealkit mcp
```

It uses stdio transport. In a typical editor integration, the editor launches SurrealKit as a child process with its working directory set to the database project.

A minimal MCP client configuration is:

```
{
  "mcpServers": {
    "surrealkit": {
      "command": "surrealkit",
      "args": ["mcp"],
      "cwd": "/path/to/project"
    }
  }
}
```

When SurrealKit is installed as a component of the `surreal` CLI, the equivalent command is `surreal kit mcp`.

The upstream Cargo configuration gates this capability behind the `mcp` feature, which depends on `rmcp` and the noninteractive template engine. It deliberately does not require the full `cli` feature.

This means a Rust program can incorporate the MCP server functionality without necessarily importing all the CLI's interactive dependencies.

### 4.2 MCP tools, resources, and prompts

The rc.1 documentation identifies 21 tools.

| Tool family               | Exposed operations                                                      |
| ------------------------- | ----------------------------------------------------------------------- |
| Project management        | `project_info`, `init`, `setup`                                         |
| Schema lifecycle          | `sync`, `seed`, `apply`                                                 |
| Generation and validation | `typegen`, `test`                                                       |
| Compatibility             | `compat_status`, `compat_raise`                                         |
| Rollout preparation       | `rollout_baseline`, `rollout_plan`, `rollout_lint`                      |
| Rollout execution         | `rollout_start`, `rollout_complete`, `rollout_rollback`, `rollout_up`   |
| Rollout maintenance       | `rollout_repair`, `rollout_status`, `rollout_freeze`, `rollout_discard` |

Resources under the `surrealkit://` namespace expose relevant project materials, including configuration, schema definitions, seed files, rollout manifests, and test suites.

The server also provides four workflow-oriented prompts:

| Prompt             | Intended workflow                                       |
| ------------------ | ------------------------------------------------------- |
| `author_rollout`   | Develop and review migration definitions                |
| `diagnose_rollout` | Investigate migration failures and inconsistent history |
| `review_sync`      | Inspect proposed schema synchronization                 |
| `write_test_suite` | Create declarative schema and permission tests          |

These prompts are workflow guidance supplied to the MCP client. They are not additional database execution engines.

The tool interface reduces the need for agents to construct shell commands, parse CLI formatting, or independently discover the project's file layout.

### 4.3 Security and containment

The MCP implementation has several explicitly documented safety properties.

Destination restrictions. Database credentials are not accepted as arbitrary tool arguments. Targets are resolved from the project's configuration and the MCP process's environment. Tools cannot independently substitute an arbitrary host, namespace, or password.

Path restrictions. Paths are resolved against the project root and canonicalized. Paths escaping that root, including through symlinks, are rejected.

Secret redaction. Resources redact inline test-actor credentials and tokens. `.env` files are not exposed as resources.

Destructive-operation acknowledgement. Operations such as destructive Sync, forced seeding, compatibility changes, rollout completion, rollback, and discard require a `confirm: true` argument.

The last mechanism is not an authorization boundary. An MCP client can set `confirm: true`; the actual authorization depends on the client approval mechanism and the database credentials granted to the server.

The server exposes one explicitly selected database per operation rather than allowing uncontrolled multi-target execution.

These features make the integration more constrained than a generic shell-execution tool, but do not eliminate the risks of granting an agent write access to a privileged database.

### 4.4 Deliberate limitations

The upstream implementation does not expose every CLI behavior through MCP.

In particular, `sync --watch` is excluded because a watch loop would never complete as a normal tool invocation. Initializing projects from arbitrary remote Git template URLs is also restricted to the CLI.

The MCP server uses stdio rather than a remotely accessible Streamable HTTP endpoint. Upstream documentation explicitly defers stateless HTTP transport until an appropriate authentication model exists.

This keeps the MCP process under the local client's process and permission boundaries, although the process can still connect to remote databases.

Primary sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/mcp/mod.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/mcp/mod.rs?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml?utm_source=chatgpt.com)

## 5. SurrealDB 4 native data migrations

SurrealDB engine functionality integrated into SurrealKit 1.0.0-rc.1

### 5.1 Migration of existing field values

SurrealDB 4 introduces a native field migration mechanism that can transform records already stored in a table when its field definition changes.

For example:

```
DEFINE FIELD stock ON product TYPE int
    MIGRATE EXISTING ROWS AS stock_to_int
    USING <int> $value
    FALLBACK 0;
```

This is a SurrealDB engine feature. SurrealKit adds schema-source management, migration identity tracking, rollout planning, recovery, and dependency coordination around it.

The distinction is important because the conversion itself is performed under database semantics, while SurrealKit determines when a migration should be submitted and how it participates in the surrounding schema lifecycle.

### 5.2 Named migration identity and tracking

A named field migration executes once per table under its designated identity.

SurrealKit queries the database to determine which migration names have already executed and records corresponding information in a dedicated `migration` partition of its `__entity` metadata.

It also prevents a migration name from being reused for a different transformation.

One subtle behavior concerns subsequent edits to a field definition. SurrealDB does not reapply a `DEFINE FIELD` statement carrying a migration name that has already executed. If the statement were simply resent unchanged, later changes to permissions or comments could be ignored.

SurrealKit therefore removes the already-executed migration clause from the statement it submits, allowing the remaining definition to be updated.

The migration clause is excluded from the ordinary definition hash, which prevents already-executed migration metadata from causing unnecessary schema reapplication.

This separates the persistent desired-state definition from the historical act of transforming records.

### 5.3 Migration dependencies and index handling

A field cannot be migrated while an existing index depending on that field blocks the transformation.

SurrealKit handles this by identifying affected indexes, removing them, applying the migration, and recreating their definitions.

This eliminates some of the custom index-dependency orchestration otherwise needed around schema changes.

However, the interval between removing and recreating an index has meaningful consequences.

For a `UNIQUE` index, uniqueness enforcement is temporarily absent while that index is removed. SurrealKit warns about this condition, but the database does not guarantee uninterrupted uniqueness enforcement across the operation.

The migration also cannot freely read arbitrary other fields. A `USING` expression operates on `$value`, representing the field being migrated.

Consequently, a transformation requiring data from another field—for example copying `old_name` into `new_name`—still requires an operation such as a rollout `run_sql` step.

SurrealKit also permits the specialized SurrealDB 4 statement:

```
ALTER TABLE customer
    MIGRATE EXISTING ROWS AS cleanup_customer
    REMOVE UNDECLARED FIELDS;
```

This is an exception to the usual restriction on non-`DEFINE` statements in schema files. General `ALTER` operations are not thereby permitted through the default declarative Sync path.

The rc.1 catalog additionally recognizes SurrealDB 4's `DEFINE TYPE` and `DEFINE EXTENSION` definitions, tracking them as `type` and `module` entity kinds respectively.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md?utm_source=chatgpt.com)

### 5.4 Migration-aware rollback

A traditional schema rollback can restore the previous `DEFINE FIELD` statement, but that does not necessarily restore the data previously stored under it.

SurrealKit rc.1 makes this distinction explicit for native field migrations.

When a rollout includes `MIGRATE EXISTING ROWS`, the planner can introduce a `revert_migration` step requiring the author to provide an appropriate reverse conversion.

For example:

```
[[steps]]
id = "revert_product_stock"
phase = "rollback"
kind = "revert_migration"
table = "product"
field = "stock"
name = "stock_to_int_revert"
using = "<string> $value"
fallback = ""
```

This illustrates the manifest structure. Whether the reverse conversion is correct or lossless depends on the original transformation and data domain.

SurrealKit validates the reverse transformation before starting the rollout. An empty `using` expression or an expression containing unsupported operations causes planning validation or execution to fail.

If a database rejects the reverse migration, SurrealKit permits correction of an unexecuted rollback step without invalidating already-executed steps. The operation can then be retried.

If no reverse conversion can legitimately restore the original data semantics, the migration can be marked irreversible by omitting the reverse step. SurrealKit reports that status and refuses rollback once the forward transformation has executed.

This is a valuable distinction between:

- Definition reversibility: the previous database schema can be restored.
- Data reversibility: the stored values can be transformed back into their former representation.
- Operational reversibility: the application and database can safely return to their previous combined behavior.

SurrealKit can express and coordinate the first two to an extent. It cannot prove the third for arbitrary application workloads.

### 5.5 Completion barriers with `await_migrations`

The migration system introduces an `await_migrations` rollout step.

It waits for SurrealDB to complete the relevant migration work and verify that records conform to the resulting definitions.

The documented default wait limit is one hour, configurable through `timeout_secs`.

This is useful when a schema migration executes asynchronously or requires substantial database work after its initial submission.

Without such a barrier, a rollout could advance to destructive cleanup while the data transformation or validation it depends on remains incomplete.

There is still an important compatibility limitation: a native migration can change existing field values immediately when it commits. Existing application code expecting the old representation may encounter changed data before the final rollout phase.

For genuinely overlapping application deployments, introducing a new field, performing a controlled backfill, switching consumers, and removing the old field later may still be preferable to changing existing values in place.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 6. Database compatibility levels and SurrealDB 4 transitions

Introduced in SurrealKit 1.0.0-rc.1

SurrealDB 4 separates the software version from the compatibility level associated with an individual database.

A database created under SurrealDB 4 normally begins at compatibility level 4. A database created under SurrealDB 3 may retain compatibility level 3 after the datastore software has been upgraded.

SurrealKit exposes explicit inspection and transition operations:

```
surrealkit compat status
surrealkit compat raise --yes --wait
```

The Rust-facing implementation is exposed through the public `compat` module.

### 6.1 Compatibility-level effects on schema mutation

At compatibility level 4, schema changes can have different consequences for existing records.

For a `SCHEMAFULL` table containing records, SurrealDB can refuse a field-definition change when the existing data cannot be assumed compatible with the new definition. An explicit migration may be necessary.

Removing a field can also remove the stored values associated with that field.

This changes the meaning of apparently ordinary schema-management operations.

For example, pruning a field definition from a schemafull table can be a data-destructive operation, not simply a metadata change.

SurrealKit reports this distinction in dry-run output and execution warnings, identifying field removals that delete stored values under compatibility level 4.

### 6.2 Inspecting and raising compatibility levels

`compat status` provides information about the database's compatibility level, migration history, field conformance, and outstanding validation activity.

`compat raise` executes the relevant compatibility transition and, with `--wait`, observes validation across schemafull tables.

Where existing records fail the new constraints, SurrealKit can identify the affected schema file and suggest the `MIGRATE EXISTING ROWS` clause needed for resolution.

The transition should be distinguished from installing a newer SurrealDB binary. The rc.1 documentation specifies that compatibility should be raised only once all database nodes are running SurrealDB 4.

This makes compatibility management an explicit part of schema lifecycle control rather than an assumed side effect of server upgrades.

### 6.3 Rust SDK and server version boundary

The upstream prerelease sequence has an important compatibility split:

| SurrealKit revision   | Rust SDK      | Supported database server               |
| --------------------- | ------------- | --------------------------------------- |
| 1.0.0-beta.6          | SurrealDB 3.3 | SurrealDB 3.2+ as documented for beta.6 |
| beta.8 implementation | SurrealDB 4.0 | Supports 3.2+ servers, including 4.0    |
| 1.0.0-rc.1            | SurrealDB 4.0 | Requires 4.0 or later                   |

In rc.1, ordinary operations such as `Sync`, `Rollout`, `Rollouts`, and `Seed` check server compatibility.

A Rust application using the rc.1 library also needs compatible SurrealDB 4 Rust SDK types. The `Surreal<Any>` handle from an SDK 3 dependency is not interchangeable with the SDK 4 handle expected by the newer library.

The Rust minimum version remains 1.95.

SurrealDB 4 removes the RocksDB engine feature, so `kv-rocksdb` is not available through the rc.1 Cargo feature set. The supported embedded backend features are `kv-mem` and `kv-surrealkv`, with `embedded` enabling both.

These are version-specific distinctions, not general limitations of SurrealKit across all releases.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml?utm_source=chatgpt.com)

## 7. Historical rollout management and migration-chain integrity

Major improvements introduced in beta.6 and retained in newer source

The original overview explained phased rollout execution but did not fully describe the machinery for preserving, validating, modifying, and replaying a historical migration chain.

These capabilities are particularly important when database instances do not advance through every application release at the same time.

### 7.1 Historical SQL freezing

Newly planned rollouts preserve the SurrealQL they need to execute in an associated directory:

```
database/
  rollouts/
    20261108__add_customer_status.toml
    20261108__add_customer_status/
      schema/
        customer.surql
```

The migration manifest records content hashes for its frozen SQL files.

This makes historical migration execution independent of later edits to the application's current schema files.

A database that missed several releases can execute the migration definitions belonging to those releases, rather than accidentally applying the latest source files as though they represented each historical migration.

For older rollout manifests that did not contain frozen SQL, SurrealKit provides:

```
surrealkit rollout freeze <rollout-id>
```

This converts historical manifests to the newer self-contained representation using the schema state associated with their original planning revision.

Freezing changes the manifest's checksum. It must not be performed while an instance of the affected rollout is in progress, because the stored migration identity may no longer match the resulting artifact.

Frozen files should be treated as immutable historical records. Line-ending transformations can change their hashes, so the documentation recommends disabling Git text normalization for the frozen rollout directory where relevant.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 7.2 Ordered catch-up across multiple releases

`surrealkit rollout up` does more than execute the most recent migration.

It determines database progress from recorded rollout history or baseline schema hashes and identifies the pending migration chain.

```
surrealkit rollout up --dry-run
surrealkit rollout up
surrealkit rollout up --complete
```

The normal catch-up process completes older pending migrations and leaves the newest migration ready for application cutover.

This distinction matters when a database is several releases behind.

For example, suppose a database is at release A, while the available migration chain is:

A → B

Complete

B → C

Complete

C → D

Start only

The newest rollout remains ready for completion after application cutover.

The older migrations' contraction phases execute during catch-up. They do not wait for a fresh application cutover at each historical release.

This is suitable when the catch-up migration chain has been designed for that behavior, but it does not automatically prove compatibility with an application still depending on definitions removed in intermediate releases.

`rollout up --from <id>` provides an explicit starting point when database history cannot determine which migration should execute next.

### 7.3 Migration-chain validation

`rollout lint` can validate the entire local migration history without connecting to a database.

```
surrealkit rollout lint
```

It checks relationships between manifests, not merely the syntax of one migration.

This includes detection of migration-chain gaps, competing migrations planned from the same snapshot, mismatched frozen SQL, and current schema changes that are not represented by a planned rollout.

Two development branches can independently produce migrations from the same starting snapshot. Those migrations may both be individually valid, yet form incompatible branches of the historical migration chain.

SurrealKit detects that condition so the history can be reconciled before execution.

This provides a useful CI verification boundary for migration artifacts.

### 7.4 Discarding proposed migration history

The `rollout discard` operation removes a planned migration and restores the local planning snapshots to their state before that migration was created.

```
surrealkit rollout discard <rollout-id>
```

The distinction from manually deleting a manifest is important.

SurrealKit's snapshots record the schema state from which subsequent migrations are planned. Deleting only the manifest could leave the snapshots reflecting changes that no longer have a corresponding migration.

`discard` maintains consistency between the manifest chain and its planning baseline.

The operation is restricted to the newest rollout, and it must not remove a migration already completed by a database.

It is distinct from `rollback`, which operates on a database's execution state, and from `repair`, which reconciles migration metadata after interruption.

### 7.5 Rolled-back and skipped migration history

A rolled-back migration remains a meaningful part of historical state.

SurrealKit does not automatically restart it when executing `rollout up`. The caller must deliberately resolve the rolled-back migration, either by restarting it or by discarding and replanning it under the applicable restrictions.

The status system also identifies migrations that a database has passed without executing.

This is particularly consequential for data migrations: a database may have a current-looking schema even though historical data transformations never ran.

SurrealKit reports skipped history but does not automatically apply potentially unsafe data transformations retroactively.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/README.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/README.md?utm_source=chatgpt.com)

## 8. Rollout recovery, concurrency, and metadata consistency

### 8.1 Incremental catalog reconciliation

SurrealKit maintains a catalog of managed schema objects in `__entity`.

Earlier versions could rewrite the entire catalog while completing or rolling back a migration. This created a failure mode in which the existing catalog was removed but replacement writes failed, leaving the database schema and SurrealKit's tracking metadata inconsistent.

The newer implementation instead performs incremental reconciliation.

It writes changed records in bounded chunks, preserves existing records until their replacements are established, removes stale records last, and reads the resulting catalog back before marking the rollout complete.

The benefit is not merely reduced database work. It also changes the recovery properties of partial metadata writes.

When a catalog update fails, SurrealKit records the failure and can use the saved rollout state to reconstruct the expected catalog.

### 8.2 Transaction-conflict retries in distributed deployments

The beta.8 implementation improves concurrency handling for the metadata operations surrounding a rollout.

These include lock acquisition and release, step-log writes, rollout-status transitions, and other bookkeeping operations that may encounter transaction conflicts.

The implementation retries qualifying conflicts up to six attempts using backoff and logs the retries.

Lock owners also include a random component in addition to the process identifier. This prevents collisions between unrelated processes that happen to reuse the same PID, a realistic possibility in container-based execution environments.

These mechanisms reduce failures caused by metadata contention. They do not automatically retry arbitrary user-defined SQL transformations or guarantee that every interrupted database operation is safe to execute again.

### 8.3 State-aware recovery

`rollout repair` reconciles migration state without replaying the rollout's data-changing SQL.

In the newer implementation, the appropriate recovery behavior depends on the recorded state and completed steps.

| Recorded condition                                 | Recovery behavior                                    |
| -------------------------------------------------- | ---------------------------------------------------- |
| Start phase interrupted, all start steps completed | Can advance to `ready_to_complete`                   |
| Start phase interrupted, steps remain              | Resume or mark failure according to the step history |
| Completion interrupted after steps finished        | Reconcile target metadata and mark completed         |
| Rollback interrupted after steps finished          | Reconcile source metadata and mark rolled back       |
| Start attempted while completion is in progress    | Refused; caller is directed to completion recovery   |
| Completion attempted before start finished         | Refused until the start phase is resolved            |

The `Rollout::abandon` operation remains a separate administrative escape hatch. It can clear a stuck migration's tracking state, but it does not reverse schema or data changes already executed.

This distinction makes the step log and rollout metadata important recovery artifacts. Manually editing these records can defeat the assumptions on which safe resumption depends.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/rollout.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/rollout.rs?utm_source=chatgpt.com)

## 9. Disposable database orchestration for external tests

`test --exec` introduced in the beta.8 development series

The newer testing framework can serve as a database provisioning and isolation layer for arbitrary application-level tests, rather than only executing its own declarative TOML cases.

### 9.1 Executing application tests against prepared suites

The interface is:

```
surrealkit test --exec "cargo nextest run"
```

The runner prepares an isolated suite database, applies its selected schema and seeds, executes its declarative cases, then runs the supplied external command before cleaning up the database.

This allows Rust SDK integration tests, generated-model verification, or other test programs to run against the same database definitions being validated by SurrealKit.

The external command does not have to be written in Rust.

If there are no declarative suite files, the runner can create an implicit `exec` suite for the external command.

The command is invoked directly, not through a shell. Its arguments are split on whitespace, so commands requiring complex quoting or shell expansion need an appropriate wrapper executable or script.

### 9.2 Database connection and identity injection

SurrealKit provides the external test process with information about its prepared database.

| Environment variable            | Meaning                                               |
| ------------------------------- | ----------------------------------------------------- |
| `SURREALKIT_EXEC_ENDPOINT`      | Database server endpoint                              |
| `SURREALKIT_EXEC_NAMESPACE`     | Isolated namespace                                    |
| `SURREALKIT_EXEC_DATABASE`      | Isolated database name                                |
| `SURREALKIT_EXEC_TOKEN`         | Temporary database-scoped authentication token        |
| `SURREALKIT_EXEC_TOKEN_<ACTOR>` | Authentication token for a configured suite actor     |
| `SURREALKIT_EXEC_ACTORS`        | Available actor names                                 |
| `SURREALKIT_EXEC_SCHEMA_DIGEST` | Semantic digest of the prepared database schema       |
| `SURREALKIT_EXEC_SUITE`         | Current test suite                                    |
| `SURREALKIT_EXEC_SCHEMA_FROM`   | Whether the schema was built through Sync or rollouts |
| `SURREALKIT_EXEC_API_URL`       | Configured HTTP API base URL, when applicable         |

This is a useful interface because the test program does not need to construct its own ephemeral database, bootstrap credentials, apply fixtures, and determine which migration history was used.

The schema digest also connects the test database to the semantic schema-generation system described earlier.

### 9.3 Test credentials and lifecycle guarantees

The default `SURREALKIT_EXEC_TOKEN` is associated with a temporary database-level user rather than server-wide root access.

The token is valid for one hour and is removed with the disposable database.

SurrealKit deliberately omits the root actor's token unless `--exec-root-token` is supplied.

The runner also removes inherited database username and password variables from the external command's environment, preventing those credentials from being passed through automatically.

The external command's exit status is recorded as a test case, so failure contributes to the ordinary test report.

Cleanup occurs after successful execution, failed preparation, failed assertions, external process failure, and interruption. `--keep-db` is the explicit exception.

### 9.4 Important limitation: the tester requires a database server

The test runner has a material restriction that was not established in the original overview:

The rc.1 test harness does not support an embedded SurrealDB endpoint or authentication level `none`.

It needs a server endpoint with root- or namespace-level authentication, because it must create isolated database environments and sign in as test actors.

Database-scoped credentials alone are insufficient to establish the isolated databases used by the harness.

This is distinct from SurrealKit's ordinary schema-management library APIs, which can operate against embedded engines.

An application can therefore use embedded SurrealDB for its normal runtime while using a separately running SurrealDB server for SurrealKit's managed integration tests.

### 9.5 Additional declarative assertion semantics

The newer testing functionality also deserves several qualifications beyond the original overview.

`sql_expect` evaluates the first SQL statement's result. A multi-statement script should not be assumed to expose its final statement as the assertion target.

Missing JSON paths and missing HTTP headers fail assertions by default; absence must be explicitly requested using `exists = false`.

The framework supports comparisons against authenticated actor identity through `equals_auth`, including `$auth.id`, which is valuable for record-level authorization tests.

A present `NONE` value and an absent key are not equivalent. Because TOML cannot directly express every relevant database value, some comparisons are most accurately performed inside the SurrealQL query itself.

Finally, the CLI's `--schema-from` setting overrides both a suite-level selection and the global testing default. This allows the same suite to be tested against direct Sync, historical rollout replay, or both without changing the suite source.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/mod.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/mod.rs?utm_source=chatgpt.com)

## 10. Expanded Rust integration APIs

The newer Rust crate exposes several useful lower-level components that extend beyond the Sync, Seed, and Rollout examples in the original overview.

### 10.1 Programmatic testing and structured reports

The `tester` module exposes:

```
surrealkit::tester::run_test_report
surrealkit::tester::run_test_to
surrealkit::tester::ReportFormat
surrealkit::tester::TestOpts
```

`run_test_report` returns a structured `RunReport` containing suite, case, assertion, and failure information.

A particularly useful semantic distinction is that `run_test_report` can return a successful Rust `Result` even when individual test cases failed. The caller examines `RunReport::cases_failed` to determine the test outcome.

By contrast, `run_test_to` renders the report and returns an error if cases failed.

This supports two different integration styles:

| API               | Use                                                                                     |
| ----------------- | --------------------------------------------------------------------------------------- |
| `run_test_report` | Obtain structured test results for custom evaluation, reporting, or downstream analysis |
| `run_test_to`     | Write a human-readable or JSON report to a supplied writer                              |
| `run_test`        | Execute suites with the default human-oriented reporting behavior                       |

The separation can eliminate the need to parse CLI output simply to obtain per-test results.

### 10.2 Structured progress capture

The `progress` module provides a mechanism for capturing SurrealKit execution logs as structured data.

Important public interfaces include:

| API             | Role                                                    |
| --------------- | ------------------------------------------------------- |
| `ProgressSink`  | Bounded collection of progress records                  |
| `ProgressLine`  | Structured log level and message                        |
| `CaptureLogger` | Routes logging to active capture contexts               |
| `capture`       | Executes a future with a task-local progress sink       |
| `propagate`     | Carries an existing capture context into a spawned task |
| `StderrLogger`  | Conventional fallback logging                           |

The default progress buffer is bounded to 4,096 lines or 512 KiB, whichever is reached first. Excess records are counted rather than being allowed to grow memory consumption indefinitely.

The `propagate` mechanism matters in asynchronous Rust because task-local state is not automatically inherited by futures spawned through `tokio::spawn`.

This design was introduced partly to support MCP: the server needs to return execution progress as structured tool data without writing arbitrary logs to stdout and corrupting its JSON-RPC transport.

The same interfaces can be useful for Rust applications that expose schema-operation progress through their own monitoring or orchestration layer.

The public module and task-local interfaces are verified in the rc.1 source; their suitability for a particular concurrent logging setup depends on how the host initializes the process-global logging facade.

Source:

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/progress.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/progress.rs?utm_source=chatgpt.com)

### 10.3 Programmatic schema preparation and diagnostics

The `schema_state` module includes `prepare_schema_sql`, which supersedes the older `ensure_overwrite` helper.

Instead of returning only transformed SQL text, the newer API returns a result containing prepared SQL and associated warnings or errors.

This is relevant to code that wants to inspect how SurrealKit will prepare definitions before submitting them to the database.

In particular, SurrealKit's statement preparation has specialized behavior for sequences, migration clauses, and supported schema-definition syntax.

A programmatic caller may therefore use the preparation interface to avoid reproducing those rules independently.

The API does not replace full engine-level parsing or semantic validation; the actual database remains authoritative for whether a definition can be executed.

### 10.4 Schema and target selection as a shared abstraction

`selection::Selection` represents the schema-module and target matrix resolved from project configuration.

It allows different frontends to share the same selection model rather than independently implementing configuration-based module ordering and target resolution.

This is especially relevant when integrating the library into custom operational interfaces where the same selection needs to be applied consistently across Sync, seeding, and other commands.

The public API also exposes `ProjectConfig`, `Target`, `Module`, and `Partition`, supporting more explicit interaction with configuration and tracked schema ownership.

### 10.5 Rust API version migration details

Several public types have changed between the 0.7, beta, and rc.1 implementations.

Among the consequential differences:

| Area            | Version-sensitive change                                                  |
| --------------- | ------------------------------------------------------------------------- |
| Rollout files   | `FileRef` and `FrozenFile` represent path-based and frozen SQL references |
| Rollout actions | rc.1 adds `RevertMigration` and `AwaitMigrations`                         |
| Entity kinds    | rc.1 adds `EntityKind::Type`                                              |
| Schema catalog  | Migration metadata and migrated-entity reporting are added                |
| Type generation | New `FieldType` and `PrimitiveType` variants and diagnostic-aware APIs    |
| Testing         | Additional execution fields in `TestOpts`; structured result APIs         |
| Connections     | Additional timeout fields in `DbCfg` and `DbOverrides`                    |
| Compilation     | New public `compile` module and semantic document representations         |

Rust code using exhaustive matches on enums or struct literals is particularly sensitive to these changes.

Another noteworthy change is that the programmatic `Rollout` facade no longer implicitly creates a filesystem project when executed. Filesystem-backed behavior is selected explicitly with `.folder(...)`.

This makes embedded programmatic rollouts more suitable for applications that want migration execution without runtime filesystem side effects.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/lib.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/lib.rs?utm_source=chatgpt.com)

## 11. Configuration resolution and template-variable semantics

### 11.1 Configuration discovery and validation

SurrealKit 1.0 discovers `surrealkit.toml` by searching upward from the working directory until it finds the nearest configuration file.

This allows a nested Rust crate or application directory to use configuration located at a parent project root.

However, configuration discovery and database schema-folder selection are separate concerns. The schema folder is controlled by `--folder` or `SURREALDB_FOLDER`, which defaults to `./database`.

The 1.0 release also corrected an earlier issue in which `--folder` was parsed but did not take effect. This is consequential when upgrading existing command invocations that may have relied unintentionally on the old fallback.

Configuration is validated before connecting to the database.

Validation includes module dependency cycles, unknown module references, duplicate primary targets, malformed target definitions, unknown configuration keys, and explicitly forbidden inline passwords.

The distinction is operationally valuable: configuration errors can be detected before any database in a multi-target operation is modified.

### 11.2 Type generation and explicit target selection

The newer `typegen` implementation resolves the database it introspects according to the normal single-target selection rules.

Resolution follows the explicit target, then the configured primary target, then the only declared target, and finally the ambient database connection.

This corrects earlier behavior in which type generation could unintentionally introspect the ambient database even when a different target was selected.

The distinction matters because a generated schema representation can be syntactically valid while describing the wrong database.

For applications using multiple database targets, a generator's source database is therefore part of its effective build configuration.

### 11.3 Template variables and change detection

SurrealKit supports text substitution in schema definitions, seeds, one-off SQL, and rollout execution.

Normal variable precedence is:

| Priority | Source                                   |
| -------- | ---------------------------------------- |
| 1        | Explicit `--var KEY=VALUE`               |
| 2        | `SURREALKIT_VAR_*` environment variables |
| 3        | `[variables]` in `surrealkit.toml`       |

Variable identifiers are case-insensitive, undefined variables cause errors, and `$${...}` escapes substitution.

The important semantic caveat is that source-content hashes and substituted SQL are not the same identity.

A schema file may remain byte-identical even when the value of a substituted variable changes.

SurrealKit's normal source-based change detection therefore may not reapply that file solely because the variable changed.

Likewise, watch-mode variables are resolved when watching begins; changing configuration variables during a watch session is not necessarily reflected until the process restarts.

Offline compilation intentionally uses a narrower variable source set: explicitly configured and supplied variables, excluding ambient `SURREALKIT_VAR_*` values.

That choice improves reproducibility, but it also means compilation and runtime synchronization can differ when deployment behavior depends on environment-sourced substitutions.

Template interpolation is textual rather than a typed SQL-parameter-binding mechanism. Callers remain responsible for ensuring substituted values are appropriate for the SurrealQL context.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



[https://surrealdb.com/docs/manage/schema-migration/configuration](https://surrealdb.com/docs/manage/schema-migration/configuration?utm_source=chatgpt.com)

## 12. Metadata setup and direct SQL execution

### 12.1 `surrealkit setup`

SurrealKit exposes an explicit setup operation for establishing or redefining the metadata structures used by its tracking system:

```
surrealkit setup
```

The internal setup implementation has an optimization beyond merely checking whether metadata tables exist.

It computes a hash of the setup definitions and records that hash in metadata. Ordinary schema-management operations can then avoid reapplying setup definitions when they have not changed and the required indexes remain present.

This matters because some SurrealDB versions rebuild indexes when `DEFINE INDEX OVERWRITE` is executed, even if the intended definition is unchanged.

Repeated metadata setup could therefore produce unnecessary database work and interfere with the indexes on which schema tracking and rollout locking depend.

SurrealKit also verifies the presence of required metadata indexes. A missing index causes setup to be reapplied.

For filesystem-backed operations, the setup implementation can use its built-in definitions when the project directory cannot be written, avoiding a hard dependency on runtime scaffolding in read-only container deployments.

Embedded library synchronization initializes the required metadata without creating schema files in the process working directory.

The `setup` CLI command remains the explicit way to force redefinition of these structures.

Source:

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/setup.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/setup.rs?utm_source=chatgpt.com)

### 12.2 `surrealkit apply`

The CLI also provides a direct SQL execution operation:

```
surrealkit apply path/to/script.surql
```

The command reads a specified SurrealQL file, performs configured template substitution, and executes it against the selected database.

Unlike tracked seeding or migration execution, `apply` does not inherently record the script as a historical operation that should run only once.

It is consequently useful for explicitly controlled administrative SQL, but it is not a substitute for the execution-history guarantees of the rollout system.

It also introduces a broad execution capability into the MCP interface, which explains why SurrealKit treats `apply` as an operation requiring explicit confirmation in that interface.

Source:

[https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/main.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/main.rs?utm_source=chatgpt.com)

## 13. Additional tooling and build integration

### 13.1 Machine-readable CLI output

The newer CLI introduces `--json`, also available through `-j` or `SURREAL_KIT_JSON=1`.

```
surrealkit --json typegen > schema.json
surrealkit --json test > report.json
```

In JSON mode, stdout is reserved for the structured output appropriate to the command. Progress information is emitted separately.

Errors can also be represented as JSON on stderr, including structured fields such as the error kind, message, hint, command, and exit code.

This is useful for orchestration systems that need predictable process output rather than parsing human-readable command messages.

The CLI now also integrates as the `kit` component of SurrealDB's broader `surreal` CLI:

```
surreal install kit
surreal kit sync
```

The `surreal kit` entry point exposes the same underlying SurrealKit command set as the standalone executable.

These integration mechanisms complement the Rust API and MCP server; they do not change the underlying database-management semantics.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 13.2 Vite generation mode

The companion `vite-plugin-surrealkit` supports a generation-oriented execution mode:

```
surrealkitPlugin({
  mode: "generate"
})
```

In this mode, the plugin runs the offline schema compiler rather than synchronizing a live database.

This allows a frontend development environment to regenerate schema-derived types and codecs as SurrealQL files change without provisioning a database or performing database writes.

The capability belongs to the Vite companion package, not the core Rust library.

Its architectural value is that schema-derived artifacts can be maintained as part of a normal development build graph, while database synchronization and rollout execution remain separate lifecycle operations.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



[https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1/packages/vite-plugin-surrealkit](https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1/packages/vite-plugin-surrealkit?utm_source=chatgpt.com)

### 13.3 Preflight parsing, source identity, and destructive-operation safeguards

The newer implementation uses a SurrealQL-aware scanner to identify statements and managed objects.

This addresses previously problematic cases involving semicolons or quotes inside regular expressions, comments between `DEFINE` and its object kind, and quoted identifiers containing whitespace.

The scanner can report malformed source with file, line, and column information before applying or pruning schema objects.

This is particularly important for declarative synchronization because a failed parsing pass must not incorrectly interpret unrecognized definitions as objects intentionally removed from the desired schema.

Filesystem-based synchronization also rejects an empty schema selection by default, even with options that would otherwise make the run appear nondestructive.

The safeguard protects against incorrect working directories and configuration errors that could otherwise look like an intentional request to remove every managed definition.

Source identity remains consequential. Tracked file keys are relative to the project folder in the 1.0 series, rather than depending on the process working directory. This improves consistency across local execution, containers, and CI.

A final lifecycle restriction concerns privileged objects. SurrealKit does not assume that saved `INFO` output can restore database users, access methods, or sequences completely. Secrets may be redacted, and sequence counters are runtime state rather than merely definition text.

Those objects therefore require more careful treatment than ordinary table or field definitions during rollback.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



## 14. How the additional capabilities fit together

The additions described above reveal several capability combinations that were not apparent from the original overview.

| Capability combination                                        | Resulting architectural possibility                                                                      | Important condition                                                                         |
| ------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Offline compilation + schema document v2                      | Derive structured database semantics without connecting to a live server                                 | The embedded engine version must represent the intended database semantics                  |
| Schema document v2 + external emitters                        | Generate language-specific models, validators, and codecs without reimplementing SurrealQL introspection | Emitters must implement the draft protocol and the desired language mappings                |
| Semantic digest + generated-file ownership                    | Detect stale generated artifacts and avoid unnecessary rebuilds                                          | Digest coverage is limited to represented schema semantics                                  |
| Frozen rollouts + chain validation + ordered catch-up         | Maintain consistent historical migrations across independently advancing database instances              | Historical data steps must actually execute; skipped transformations require reconciliation |
| Native field migrations + reverse steps + completion barriers | Coordinate schema definitions, data transformations, and validation                                      | Transformations may be irreversible or incompatible with old application versions           |
| Schema compilation + disposable database testing              | Compare generated schema expectations against runtime behavior                                           | The test runner requires an authenticated database server                                   |
| MCP + structured progress + constrained target selection      | Expose schema-management workflows directly to programming agents                                        | Database privileges and client-side approval remain security boundaries                     |
| Programmatic Rust APIs + embedded SurrealDB                   | Integrate schema lifecycle management into an application without an external migration process          | Cargo feature compatibility and engine lifecycle ownership must be respected                |

The most consequential distinction is between schema meaning, schema deployment history, and runtime data state.

Schema document v2 represents normalized schema meaning. The rollout system records how database definitions and data transformations are applied over time. SurrealDB itself holds the records, indexes, counters, and operational state.

SurrealKit increasingly connects those layers, but it does not collapse them into a single automatically reversible model.

A semantic digest does not prove that historical data migrations executed. A successful migration chain does not prove that transformed data is losslessly reversible. A generated model does not guarantee that application versions remain compatible throughout a rollout.

The newer functionality is valuable precisely because it makes several of these boundaries explicit and provides mechanisms to manage them without requiring an entirely bespoke orchestration system.

## 15. Primary-source references for the added functionality

The following version-specific sources are especially useful for investigating the implementation rather than relying on introductory documentation.

| Subject                             | Direct source                                                                                                                                                                                                                                               |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust library and public API         | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/README.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/README.md?utm_source=chatgpt.com)                                                         |
| Complete 1.0 migration history      | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md?utm_source=chatgpt.com)                                                                                       |
| Offline compiler implementation     | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs?utm_source=chatgpt.com)                                               |
| Schema document v2 specification    | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md?utm_source=chatgpt.com)                                           |
| Schema document JSON Schema         | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/schema/schema-document-v2.schema.json](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/schema/schema-document-v2.schema.json?utm_source=chatgpt.com) |
| External emitter protocol           | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/external-emitters.md](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/external-emitters.md?utm_source=chatgpt.com)                                             |
| Type-generation Rust implementation | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/typegen/mod.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/typegen/mod.rs?utm_source=chatgpt.com)                                       |
| Native data migration orchestration | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/migrate.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/migrate.rs?utm_source=chatgpt.com)                                               |
| Compatibility-level implementation  | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compat.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compat.rs?utm_source=chatgpt.com)                                                 |
| Rollout execution and recovery      | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/rollout.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/rollout.rs?utm_source=chatgpt.com)                                               |
| MCP implementation                  | [https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1/crates/surrealkit/src/mcp](https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1/crates/surrealkit/src/mcp?utm_source=chatgpt.com)                                                             |
| Structured testing API              | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/mod.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/mod.rs?utm_source=chatgpt.com)                                         |
| Progress capture                    | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/progress.rs](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/progress.rs?utm_source=chatgpt.com)                                             |
| Cargo features and SDK dependency   | [https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml?utm_source=chatgpt.com)                                                       |

This addendum is intended to be read alongside the original overview. It adds the newer compiler, semantic-document, migration, testing, MCP, and Rust integration capabilities without restating the established schema synchronization and rollout fundamentals.