---
id: ADR-0114
title: Generate the operational store's schema from the registry and compile its statements with Cornucopia on tokio-postgres
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, AP-06, DP-01, DP-02, DP-03, DP-04, DP-13, DP-15, DP-19, DP-20, DP-21, DP-24]
blueprint: [§3.2, §4.2, §17.1, §19.1, §20.1, §20.2, §20.4, §23.2, §25, §26, §D10, §D13]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md#decision
evidence: Interface-checked
supersedes: [ADR-0112]
superseded-by: null
revisit: Any of these fires. (a) The first operational store or catalog whose contents must survive a schema change (register R-35: adopt versioned migrations). (b) Cornucopia generation cannot run reproducibly under the workspace lockfile. (c) A statement defect caused by hand-annotated nullability escapes the store tests. (d) A deployment needs more than one PostgreSQL primary or a cross-region catalog, or cannot run PostgreSQL 18. (e) The Q1 event-volume measurement exceeds what binary COPY sustains.
verification: Architecture scenarios S14–S17, S19, S20, S24 and S25. They are settled by the Plan 22 tests below, together with `just db-status` reporting 18.x and a matching schema fingerprint, and governance every_crate_registered. B1 tests: generated_schema_creates_empty_store, store_schema_mismatch_refused, registry_enums_are_postgres_enums, misspelled_enum_literal_fails_prepare, catalog_relations_registered, row_invariants_generated_and_enforced, operational_timestamps_are_microseconds, postgres_value_mapping_round_trips. B2 tests: store_queries_regenerate_identically, generated_query_rejects_misspelled_column_and_literal, sqlstate_classified_by_constant, listener_resyncs_after_connection_loss. Retained O3–O9 and G8 tests on the new stack: illegal_transition_rejected, lease_expiry_marks_stale, ephemeral_cannot_publish, durable_run_listed_after_restart, two_workers_never_claim_same_job, expired_lease_requeues_as_new_attempt, cancel_notify_stops_running_job, cancel_survives_listener_reconnect, unknown_payload_version_refused, progress_stream_complete_under_volume, stored_seed_reused_across_processes, study_parallel_workers_publish_once, concurrent_publishers_one_winner_no_lost_update, lost_ack_settles_via_catalog, maintenance_waits_for_reader_leases, catalog_protects_published_versions, exported_publication_opens_offline and killed_worker_attempt_goes_stale_and_resumes_from_incumbent.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s14, docs/plans/22-solver-capabilities-architecture.md#s15, docs/plans/22-solver-capabilities-architecture.md#s16, docs/plans/22-solver-capabilities-architecture.md#s17, docs/plans/22-solver-capabilities-architecture.md#s19, docs/plans/22-solver-capabilities-architecture.md#s20, docs/plans/22-solver-capabilities-architecture.md#s24, docs/plans/22-solver-capabilities-architecture.md#s25]
---

# ADR-0114: Generate the operational store's schema from the registry and compile its statements with Cornucopia on tokio-postgres

## Context

ADR-0112 put operational state and the publication catalog in PostgreSQL 18 through
`pse-operations`, using sqlx with runtime-typed queries. It rejected Cornucopia and sqlx's
`query!` macros because they generate code. Core 3.1 dropped that preference, and the
[typed data contracts review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md)
assessed the store as Plan 22 O2–O6 built it.

The review found:
- **TD01:** each table is described four times by hand — migrations, records, `FromRow` impls and column lists — beside the registry. Only part of that is compared, and the seven catalog tables are outside the registry.
- **TD02:** 68 statements with 165 positional binds are checked only when they execute. A misspelled enum literal against a text CHECK column prepares and silently matches nothing; this was measured on PostgreSQL 18.6.
- **TD03:** stored codes and outcomes are strings, and SQLSTATE codes are matched as strings.
- **TD09:** timestamps claim nanosecond precision the store cannot hold.

The maintainer directed on 2026-09-28:
- to switch to rust-postgres/tokio-postgres where advantageous, counting sqlx's listener, migrations and test harness as minor;
- that the store holds only regenerable data;
- that programmatic derivation is preferred to static declarations.

## Scope

**Supersedes ADR-0112.** It restates, unchanged, every part of ADR-0112 that stands: publication
through the catalog, reader protection and retention, the D10 division, lifecycle, identity,
queue, cancellation, durability classes, workers, distribution scope, deployment and failure.
It replaces:
- how the store's schema is declared and created;
- how its statements are written and checked;
- how enumerations and identities are stored;
- the client stack, notifications, bulk inserts, error classification and test databases.

It adds the crate `pse-operations-queries`, which is generated.

The typed entity identities that the store maps are decided in
[ADR-0115](0115-registry-typed-identities-and-vocabularies.md). Implementation is Plan 22
packets B1 and B2, which precede O7–O9.

## Drivers

- **DP-01 and AP-04.** The registry is the one declaration of each operational relation's meaning and shape (D1). Every other representation is derived.
- **DP-03, DP-15 and AP-05.** A statement is checked against the schema before anything runs, and a misspelled literal or column fails generation (S20).
- **DP-02 and DP-21.** Stored vocabularies and identities are types, and store failures are classified by typed codes.
- **DP-19, DP-20 and DP-24.** ADR-0112's lifecycle, queue, cancellation and catalog guarantees are kept. A schema change is detected and never reinterpreted.
- **AP-06.** Value mapping and lifecycle legality are pure. The store is testable against isolated real databases (S25).
- **Maintainer direction (2026-09-28).** Programmatic over static; regenerable contents; the driver switch.

## Options

| Candidate | Assessment | Selection and revisit trigger |
|---|---|---|
| ADR-0112 as built: sqlx runtime `query`/`query_as`, hand-written `FromRow` and migrations, partial conformance test | Four hand-kept descriptions per table; statements checked only by executing them (TD01–TD03) | Replaced |
| sqlx 0.9 `query!` macros with `.sqlx` offline metadata, registry-generated DDL and `sqlx::Type` impls | Checks statements at build time and keeps sqlx's listener, migrations and test harness. But: positional binds remain; SQLSTATE stays a string; COPY is raw bytes only; `.sqlx` metadata is a second regeneration mechanism (`prepare --check` needs a live database and prints no diff; sqlx-cli cannot be installed `--locked`); and `sqlx::Type` impls would put sqlx-core into semantic crates, or require wrapper types | Not selected. Revisit if (b) fires |
| **Registry-generated schema; `.sql` statements compiled by Cornucopia 1.0.1 into a generated crate over tokio-postgres 0.7.18, deadpool-postgres 0.14.2 and postgres-types 0.2.14** | Named parameters; generated row and parameter structs; `types.mapping` from ENUM types and domains to registry types; no Cornucopia runtime dependency; workspace-inherited pins; a library entry point (`gen_fresh`) for our own regeneration check. tokio-postgres adds typed `SqlState`, typed binary COPY and pipelining. The costs: hand-annotated nullability (Cornucopia issue #226), a small listener task and test harness, and regeneration against a PostgreSQL server | **Selected** |
| Versioned migrations (refinery, or sqlx migrate) | Contents are regenerable, so create-or-refuse by schema fingerprint is simpler and cannot drift | Deferred to register R-35 |
| PostgreSQL ENUM types versus text with CHECK | ADR-0112 rejected database enum types as drift-prone, but that held only while DDL was hand-written. ENUM literals fail at `PREPARE`, while text literals silently match nothing (*Measured*) | ENUM types, generated |
| SeaQuery; Diesel or SeaORM; pgvector; pgrx; testcontainers | Dynamic querying belongs to DataFusion. ORMs and database-first schemas would make the database or a DSL a second schema authority. There is no vector data and no in-server computation. Generation needs the local server anyway | Rejected. Revisit: a store statement must be composed at run time; vector data enters scope; an operation must run inside PostgreSQL; an environment has no local server |
| `datafusion-table-providers` 0.13.1 | Requires datafusion ^54 and arrow ^58, breaking the one type universe | Blocked. Revisit when a release matches `datafusion =55.1.0` |
| ADBC PostgreSQL driver (0.24) versus the generated Arrow builders in `pse-relations` | Both fit arrow =59.3 | Chosen at Plan 22 O9, with the generated builders as the incumbent |

## Outcome

### Retained from ADR-0112, unchanged

1. **Publication** is an explicit effect after completion and never re-runs science. A scientifically failed attempt can still be published faithfully.
2. **Members** are written to immutable, attempt-scoped Delta locations. Publication names an explicit expected parent and is never rebased. A serializable ticket carrying the attempt identity exists before any effect.
3. **Settlement.** Conflict, uncertain completion and confirmed completion stay distinct. Settlement is read-only and inspects the attempt and its exact members. An uncertain outcome never permits rematerialization.
4. **Reopening** is read-only and names exact versions and compatibility contracts. An unsupported historical format is a typed migration-required refusal; opening never migrates.
5. **Retention** preserves the closure needed to reopen every retained publication.
6. **Visibility boundary.** One PostgreSQL catalog transaction:
   - checks that `publication_heads` names the expected parent;
   - inserts the publication and its members (URI, exact Delta version, contract fingerprint);
   - advances the head.

   A lost race is a typed `Conflict`, and the loser re-prepares against the new parent. The attempt identity is unique in `publications`, so settlement queries the catalog and returns `Committed`, `ProvedNoncommit` or `Conflict`. `Unresolved` remains only while the catalog is unreachable.
7. **Reader protection and retention.**
   - Readers hold a lease row with an expiry, taken in a short transaction; no reader keeps a database session open while reading Delta files.
   - Retention computes the protected versions in the catalog.
   - Deletion is two-phase: mark the publication `expiring`, wait until leases are released or expired, let `pse-catalog` delete the member files, then mark it `deleted`.
   - Transaction-scoped advisory locks serialize maintainers.
8. **D13.** Cases and results never mutate the model. Overlays and derived preparation products carry their own identity. A cancelled or failed attempt is distinguished by its typed lifecycle state, never by absent rows.
9. **No import** of Delta publications written under the former control table: they are regenerated by rerun, and such a store is refused with a typed migration-required diagnostic. The Delta control table, its code and the file leases are deleted once the catalog path is proven (Plan 22 O8). An export command writes a read-only manifest from the catalog for offline readers.
10. **Division of authority (D10).**
    - PostgreSQL owns what changes: attempts, jobs, leases, cancellation requests, live progress, incumbents and bounds, reusable solutions, study status and the publication catalog.
    - Delta owns what is published. Published operational relations are derived snapshots with identity.
    - The registry (D1) owns the meaning **and the shape** of every operational relation (Outcome 22).
11. **Crate `pse-operations`** owns the store contract, the repositories, the catalog and reader leases, with errors deriving `thiserror` and `miette` with §23.2 codes. `pse-runtime` depends on it; no semantic or native crate does. `pse-catalog` keeps Delta member I/O, and executes deletion and optimization when the catalog instructs.
12. **Lifecycle.**
    - planned → queued → running → {completed, partial, failed, cancelled}; running → stale on lease expiry; a stale attempt is superseded by a new attempt. Planned and queued attempts may be cancelled.
    - One pure Rust transition table in `pse-operations` is the only authority for legality. Repository functions apply it inside a transaction under row locks.
    - The database enforces the value domain (now the generated ENUM type) and append-only history.
13. **Identity.** The runtime mints attempt, run and publication identities on its UUIDv7 `SemanticId` path before any effect. No database default mints a domain identity. Event rows are keyed by (attempt, sequence).
14. **Queue.**
    - Jobs are claimed with `FOR UPDATE SKIP LOCKED`. A heartbeat extends the attempt's lease; expiry marks it stale and requeues the job under an explicit retry policy.
    - Each try is a new attempt, and publication is idempotent per attempt. Idempotency keys are unique.
    - Job payloads are versioned, and an unknown version is refused. Source bundles are content-addressed by the §6.1 content hash.
    - `MathService` queues durable work instead of refusing it.
15. **Cancellation.** `cancel_requested` is the authority, and the heartbeat returns it. `NOTIFY` only shortens latency. A worker issues `LISTEN`, commits, then re-reads state, and re-reads it again after any lost listener connection (Outcome 26).
16. **Durability classes.**
    - `Ephemeral`: no store; in-memory queue and progress; cannot publish.
    - `Durable`: a registered attempt that may publish, queue, run studies or run long.

    The class is an explicit policy, never a fallback.
17. **Streams and solutions.** Progress, incumbents and bounds are retention-bounded and snapshotted into Delta at publication. Reusable solutions are keyed by the coordinate-compatibility stamp and the preparation identity, and the seed identity enters lineage.
18. **Workers.** `pse-worker` is a binary target of `pse-runtime`. It sets the process-level OpenMP environment SPRAL requires (ADR-0108).
19. **Distribution scope.** Workers on one or more hosts claiming jobs is in the target. Distributing one solve across processes is not (ADR-0102).
20. **Failure.**
    - With the database unavailable, durable operations fail with the infrastructure class naming the connection target, and ephemeral solves are unaffected. There is no fallback to Delta control.
    - After a crash, leases expire, attempts become stale, and unpublished members stay reclaimable through the catalog.

### Replaced and added

21. **Deployment.**
    - PostgreSQL ≥ 18 (locally 18.6, cluster `18/main`, port 5432).
    - A login role owning database `pse`, with `CREATEDB` for test databases and generation.
    - Connection by `PSE_DATABASE_URL`, over the local socket with peer authentication by default. No credential is ever committed.
    - Recipes: `db-bootstrap` (idempotent, behind the outward-recipe confirmation), `db-reset` (drops and recreates `pse_ops` from the generated schema; confirmed and destructive; replaces `db-migrate`), `db-status` (server version, reachability, schema fingerprint), `db-backup` and `db-restore`.
    - A doctor check for version, reachability and fingerprint.
22. **The registry generates the schema.**
    - The registry declares every `pse_ops` table, including the seven catalog tables, with:
      - the row invariants (the solution-vector and one-value rules among them);
      - microsecond timestamps;
      - JSON-document columns;
      - the enums `RetentionPhase`, `SettlementOutcome` and `TerminationCode`;
      - entity identities (ADR-0115).
    - A `pse-codegen` target `postgres` emits into `crates/pse-operations/src/generated/`:
      - the DDL: one ENUM type per registry enum; one domain per entity identity over `uuid`; a `content_hash` domain over 32-byte `bytea`; tables, NOT NULL, keys, foreign keys, and CHECKs from registry invariants;
      - the schema fingerprint;
      - the Cornucopia type-mapping table.
    - A hand-written `physical.sql` in `pse-operations` adds indexes, partial indexes, defaults, grants and the append-only revoke. It is SQL because SQL is the declaration language for access paths, and its enum literals are checked when it is applied.
23. **Create or refuse; never migrate.**
    - `Store::open` on an empty database applies the generated DDL and `physical.sql`, and records the fingerprint.
    - On a different fingerprint it returns a typed `SchemaMismatch` naming `just db-reset`. It never resets implicitly.
    - Stored contents are not migrated while they are regenerable (maintainer, 2026-09-28). A reset forgets the catalog; published members are regenerated by rerun.
    - Register R-35 holds the trigger for versioned migrations. At that trigger, the generated DDL becomes the target every migration must reach.
24. **Statements are SQL files compiled into a generated crate.**
    - Statements live in `crates/pse-operations/queries/*.sql`, with named parameters and declared nullability. Whole-row statements take the row declarations generated from the registry.
    - `xtask codegen` calls Cornucopia 1.0.1's `gen_fresh` under the workspace lockfile against the local PostgreSQL 18 server. It loads the generated DDL and `physical.sql` into a temporary database and writes `crates/pse-operations-queries/`: generated, committed, with workspace-inherited dependencies, and its manifest normalized to the workspace MSRV and edition.
    - `--check` regenerates into a temporary directory and diffs in both directions (ADR-0051). Without a reachable server, that arm exits with a notice rather than passing.
    - Repository modules call the generated functions. Their public API is typed records (the generated rows), never driver types.
    - No SQL is assembled at run time, and no enum literal is written into a statement where a typed parameter can carry it.
25. **Value mapping.**
    - An optional `postgres` feature on `pse-ids` and `pse-model` carries generated postgres-types `ToSql`/`FromSql` impls:
      - registry enums map to their ENUM types by name;
      - typed ids map to their domains (accepting the domain for parameters and `uuid` for results);
      - `ContentHash` maps to the `content_hash` domain.
    - The dependency is the value protocol, not a driver; only `pse-operations-queries` enables it.
    - There are no parse helpers and no hand-written `FromRow`.
26. **Client stack.**
    - tokio-postgres 0.7.18, with pipelining.
    - deadpool-postgres 0.14.2, with `prepare_cached`.
    - tokio-postgres-rustls 0.14 (ring) for remote stores.
    - **Notifications:** a listener task on a dedicated connection. It re-issues `LISTEN` after reconnecting with backoff, and signals a resynchronization so workers re-read `cancel_requested` (Outcome 15).
    - **Bulk inserts:** progress events, progress values and incumbents are inserted with `BinaryCopyInWriter`.
27. **Errors.** Store failures are classified by `SqlState` constants, never by string:

    | SQLSTATE | Typed error |
    |---|---|
    | 40001, 40P01 | Retryable |
    | 55P03 | Lock not available |
    | 57014 | Cancelled |
    | 23505 | `Duplicate` |
    | 23514, 23503 | A typed `InvariantViolation`, not `Internal` |
    | class 08, 57P01–57P03 | `Unavailable` |

    Retry decisions read the type.
28. **Tests.**
    - `pse-operations` test support creates one database per test from the generated schema, and drops it afterwards.
    - Lifecycle legality and value mapping are pure tests.
    - Every statement is exercised by a store test, which bounds the hand-annotated nullability.
29. **Removed.**
    - sqlx and its `whoami` workaround.
    - `sqlx::migrate!` and the migrations.
    - `#[sqlx::test]` and `PgListener`.
    - `codec.rs`.
    - Hand-written records that restate a row, and their `FromRow` impls.
    - Column macros, positional binds and `UNNEST` batch statements.
    - String SQLSTATE matching.
    - The migration-conformance comparison, which generation makes redundant.

### Consequences

- **Server needed for regeneration.** Regenerating the query crate needs a reachable PostgreSQL 18 server, which is already a development prerequisite of the store tests.
- **Resets lose history.** A schema change resets local operational state and the catalog; only regenerable results are lost.
- **Hand-annotated nullability.** Nullability is annotated by hand until Cornucopia infers it.
- **Bespoke parts.** The listener task and the test-database harness are small bespoke parts of `pse-operations`.
- **Generated paths.** The generated paths gain `crates/pse-operations/src/generated/` and `crates/pse-operations-queries/`. AGENTS.md prime directive 2, the edit hook and the deny rules are updated with them.
- **Relocated retention computation.** The retention computation leaves the Delta control relations and moves into the catalog SQL.

### Compensating controls

- The schema fingerprint check at `Store::open` and in `db-status`.
- The regeneration check for both generated trees.
- The generator's negative control: a misspelled column or ENUM literal fails generation.
- A store test for every statement.
- The pure lifecycle table.
- The `Ephemeral` class.
- The confirmation on `db-reset`.
- Register R-35.

### Confirmation

**Architectural reasoning.** The typed data contracts review, slots 4–9.

**Measured.** A psql probe inside rolled-back transactions on PostgreSQL 18.6, `18/main`, 2026-09-28. A misspelled literal compared with an ENUM column failed at `PREPARE`. Compared with a text domain carrying a CHECK, it prepared and returned zero rows, and failed only when inserted.

**Interface-checked.**
- Cornucopia at `b078d22`: `src/lib.rs` (`gen_fresh`), `src/codegen/cargo.rs` (no runtime dependency; workspace dependencies), `src/type_registrar.rs` (mapping by PostgreSQL type, domains included).
- rust-postgres at `1084ca8`: `SqlState`, `binary_copy`, pipelining, and no reconnection (issue #802).
- PostgreSQL 18: `printtup.c` sends a domain column's base type.

**Tested.** The tests in `verification:` settle the implementation in Plan 22 B1, B2 and O7–O9. Acceptance does not certify that work.

## Pros and cons

| Option | For | Against |
|---|---|---|
| Cornucopia on tokio-postgres | Generated, reviewable source under one regeneration discipline; named parameters; typed SQLSTATE and COPY; light value-protocol coupling | Manual nullability; small bespoke listener and harness; generation needs a server |
| sqlx compile-time macros | One crate with its conveniences; inferred nullability | Opaque metadata with a second check mechanism; positional binds; string SQLSTATE; sqlx-core in semantic crates |

## More information

- [Typed data contracts review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md): TD01–TD03, TD09, slot 8.
- Architecture companion [§9.2, §9.3, §9.7, §9.8](../plans/22-solver-capabilities-architecture.md#9-operational-store-and-publication-catalog-postgresql-18) and [§12](../plans/22-solver-capabilities-architecture.md#12-typed-data-contracts).
- The maintainer's external reviews: [PostgreSQL options](../external-review-postgresl-options.md); [typing and other enhancements](../external-review-postgresl-typing-and-other-typing-enhancements.md).
- The Plan 22 target review's T02, T03, T13 and T16 remain in force through the restated outcomes.
- Register R-35.
- Plan 22 packets B1, B2 and O7–O9.
- [ADR-0115](0115-registry-typed-identities-and-vocabularies.md) (entity identities) and [ADR-0051](0051-generated-trees-and-regeneration-check.md) (regeneration check).

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's approval of the typed data contracts review (Accept-scoped, author review, Proposed evidence level). Supersedes ADR-0112, restating Outcomes 1–20 of it and replacing its schema, statement, enum-storage, client-stack, test and deployment parts.
