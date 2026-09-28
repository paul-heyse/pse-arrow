# Operational store (PostgreSQL 18)

> **Decision: [ADR-0114](../adr/0114-typed-operational-store.md)** (superseding ADR-0112)
> · [Plan 22 architecture §9](../plans/22-solver-capabilities-architecture.md#9-operational-store-and-publication-catalog-postgresql-18)
> · crate `pse-operations`
>
> Plan 22 B1, B2 and O8 have landed: the schema is generated from the registry and
> created or refused by fingerprint, every statement is SQL compiled by Cornucopia into
> the generated crate `pse-operations-queries`, run on tokio-postgres, and the catalog is
> the only publication authority (the Delta control table is gone). The Plan 22 docs
> step rewrites this page once more.

PostgreSQL owns what changes (attempts, jobs, leases, cancellation requests, live progress,
incumbents, reusable solutions, study status and the publication catalog); Delta owns what
is published. Durable work needs the store. Ephemeral library calls and unit tests do not.
This page covers setting it up and operating it on a development machine.

## One-time setup

```bash
just db-bootstrap        # asks for confirmation, then runs psql as `postgres` via sudo
just db-status           # PostgreSQL >= 18, reachable, schema fingerprint; exit 0
```

`db-bootstrap` creates the following. It is idempotent: running it again only restates
the same attributes and ownership.

- A login role named after your OS user (`$USER`) with `CREATEDB`. The test harness
  needs `CREATEDB` to create one database per test.
- Database `pse`, owned by that role.

Authentication is **peer** over the local Unix socket (`/var/run/postgresql`). The server
maps your OS user to the same-named role, so no password or other credential exists
anywhere. This relies on the Debian/Ubuntu default `local all all peer` line in
`pg_hba.conf`. Run the recipe interactively (sudo asks for your password), or
non-interactively with `just --yes db-bootstrap` in a shell where sudo does not prompt.

> **Deviation from ADR-0114 Outcome 21.** The decision names a login role `pse` using
> SCRAM. Local development uses a peer-authenticated role named after the OS user
> instead, so no secret has to be created, stored or rotated. A remote deployment uses
> SCRAM with the password in `~/.pgpass`, never in the repository or the URL.

## Connection URL

| Source | Value |
|---|---|
| Default | `postgres:///pse?host=/var/run/postgresql` (database `pse`, local socket, the OS user) |
| Override | `PSE_DATABASE_URL`, e.g. `postgres://pse@db.internal/pse?sslmode=verify-full` |

The default is declared once, as `pse_operations::DEFAULT_DATABASE_URL`. The `db-*`
recipes, query generation (`cargo xtask codegen`) and the doctor check read it from
there. The client is tokio-postgres, which reads neither `~/.pgpass` nor libpq service
files (`service=pse`), so the Rust side is configured by URL. The one libpq variable the
store honours is `PGUSER`, as the role when the URL names none: the solver container
runs as a user without a role of its own and is peer-authenticated as the host user
that way (`scripts/native-solver-runner.sh`). A remote password therefore comes from a
machine-local `PSE_DATABASE_URL` (in the gitignored `.envrc.local`), never from the
repository.

**Never commit a credential.** No URL containing a password belongs in the repository,
in `.envrc`, or in a recipe. Machine-local overrides go in the gitignored
`.envrc.local`.

## Schema

The registry owns the meaning and the shape of every operational relation: relation
`runtime.operational_<t>` (declared in `crates/pse-schema/src/catalog/operations.rs`) is
table `pse_ops.<t>`. `just codegen` renders the store from those declarations into
`crates/pse-operations/src/generated/` (generated; never edit it):

- `schema.sql`: the `pse_ops` schema, one PostgreSQL ENUM type per registry enumeration a
  store column uses, a `content_hash` domain (32-byte `bytea`), one domain per entity
  identity (`attempt_id`, `job_id`, ...; ADR-0115), and every table with NOT NULL and named
  primary-key, unique, foreign-key and CHECK constraints. The CHECKs are the registry's
  named row checks and field domains (every `double precision` column is finite).
- `cornucopia.toml`: the type mapping from those types to the registry's Rust types.
- `fingerprint.rs`: `SCHEMA_FINGERPRINT`, a digest of `schema.sql` and `physical.sql`.

`crates/pse-operations/physical.sql` is written by hand: indexes, partial indexes, column
defaults and the append-only revoke on `pse_ops.attempt_transitions`. Its enum literals are
checked when it is applied.

**Create or refuse, never migrate** (ADR-0114 Outcome 23). The store's contents are
regenerable, so there are no migrations:

- `Store::open`, which every durable runtime and worker calls, creates the schema on a
  database that has none. It applies `schema.sql` and `physical.sql` in one transaction
  and records the fingerprint as the schema's comment (`pse.ops.schema.v1 <hex>`).
- A store whose recorded fingerprint differs (or has none: a schema this build did not
  create) is refused with `SchemaMismatch`. It is never reset implicitly.
- `just db-reset` asks for confirmation, then drops `pse_ops` with everything in it and
  creates the schema again. Every attempt, job, stream, solution and catalog row is lost;
  published members are regenerated by rerunning. Long-lived processes need a restart
  after a reset.

Changing a store declaration, the generator or `physical.sql` changes the fingerprint:
run `just codegen`, then `just db-reset` for each existing store.

Transition legality lives in the Rust transition table (`pse_operations::lifecycle`),
not in SQL. Deletes are explicit: there is no `ON DELETE CASCADE`.

## Statements

Every statement is SQL in `crates/pse-operations/queries/*.sql`, one file per repository
(`store`, `attempts`, `jobs`, `cancellation`, `streams`, `solutions`, `sources`,
`catalog`), with Cornucopia's named parameters and hand-annotated nullability
(ADR-0114 Outcome 24). `cargo xtask codegen` (part of `just codegen`) renders the store
schema in memory, creates a temporary database on the local server from it and
`physical.sql`, prepares every statement there with Cornucopia 1.0.1, and writes the
generated crate `crates/pse-operations-queries/` (generated and committed; never edit it).
A misspelled column, a misspelled ENUM literal or a mistyped parameter fails generation.
`just codegen-queries-check` regenerates in scratch space and compares in both
directions; without a reachable server it fails and says so.

- **Rows.** A whole-row statement (`SELECT a FROM pse_ops.attempts AS a ...`,
  `RETURNING s`) returns the registry row (`RuntimeOperationalAttemptsRow`, ...): the
  Cornucopia mapping names each table's row type, and a generated `FromSql` in
  `pse-model` decodes it, checking the field names against the registry.
- **Typed parameters.** An identity parameter compared with a column is cast to its
  domain (`:attempt_id::pse_ops.attempt_id`) so the generated function takes the typed
  id; ENUM parameters take the registry enums. A domain-typed result column arrives as
  its base type, so a projected id is wrapped where it is read.
- **Enum literals** appear only where a partial index needs them (the job claim and the
  stale sweeps); PostgreSQL checks them when the statement is prepared.
- **Bulk inserts.** Progress events, progress values, incumbents, source documents and
  publication members go in by binary `COPY` (`generated/copy.rs`: the statement and a
  `WHERE false` probe that types the binary stream). The sequence numbers already stored
  are read first, so a re-sent batch is idempotent.

## Client stack

- **Pool.** deadpool-postgres over tokio-postgres (`application_name = pse-operations`,
  `idle_in_transaction_session_timeout` from `StoreOptions`). A pooled connection is
  verified before it is handed out again and replaced if it does not answer within a
  second: a cancelled operation may leave its statement running on the server.
- **TLS.** rustls with the ring provider named explicitly and the webpki roots; a local
  socket never negotiates TLS.
- **Notifications.** One listener task per store (`application_name =
  pse-operations-listener`) holds `LISTEN pse_ops_cancel` and `LISTEN pse_ops_progress`,
  reconnects with backoff and tells every watcher to re-read its authority after each
  reconnection (Plan 22 X8). Cancellation and progress watchers read the stored state on a
  notification, on that resynchronization and when they fall behind.
- **Errors.** Failures are classified by `SqlState` constant: 40001/40P01 retryable,
  55P03 lock unavailable, 57014 cancelled, 23505 duplicate, 23514/23503 a typed
  `InvariantViolation` naming the registry rule, class 08 and 57P01–57P03 unavailable
  (ADR-0114 Outcome 27). No driver type appears in the public API.

## Publication catalog

Publication visibility, heads, reader protection and retention are catalog rows
(Plan 22 X9–X12). Delta holds only immutable member tables and export manifests.
`pse-operations` owns the catalog statements, `pse-catalog` performs Delta member I/O and
never sees PostgreSQL, and `pse-runtime` composes the two (`Runtime::register_workspace`,
`prepare_publication`, `open`, `export_publication`, `retire_publications`, `collect`,
`reclaim_unpublished`).

- **Workspaces.** `workspaces` rows name one publication history with one head and the
  root its members are written under (`publication_heads`). A root that still holds a
  Delta control table (`{root}control/_delta_log/`) is refused as an unsupported
  historical format: regenerate its publications by rerunning them into a new root.
- **Intents.** Before the first member write, `publication_intents` records the
  publication, its workspace, its durable attempt (the publication attempt *is* the
  durable attempt) and the member prefix `{root}members/{attempt}/{publication}/`.
  Member writes carry receipts keyed by the attempt (`pse.member_attempt.v4`), so a
  re-preparation of the same intent recovers the members already written.
- **Commit.** Executing the candidate writes the members and returns the admitted
  `runtime.publication_manifests` record; one catalog transaction inserts the
  publication, its members, inputs and change windows and advances the head if it is
  still the expected parent. Locks are taken in one order (attempt, intent, selected
  publications, head) and marks are read in a new statement after the lock. A stale
  parent is `PublicationConflict` (re-prepare against the head, never rebase); an attempt
  already published as another publication is `PublicationIdentityReused`.
- **Settlement.** A lost commit acknowledgement is reported as unresolved and never
  retried implicitly. Settling the ticket queries the catalog: committed, proved not
  committed (no intent, an abandoned intent, or the head still at the parent with the
  intent locked, so no commit is in flight),
  conflict (the head moved), or unresolved when the store is unreachable. It writes
  only the `settlements` row.
- **Readers.** A reader takes a `reader_leases` row (`open`, `open_head`) in one short
  transaction that returns the complete record and the workspace's maintenance epoch,
  then reads Delta without a database session. The lease is renewed at a third of its
  lifetime and released on drop; a lapsed lease cancels the reader. The pair
  (workspace, epoch) is the session's `ReadScope`, the lookup input of every shared
  snapshot, resident and file-metadata cache; a session without one bypasses them.
- **Exports.** `export_publication` takes a lease held by `export:<destination>` for a
  stated time and writes a one-row manifest (Delta version 1) with the record, the
  lease, its expiry, the epoch and the store fingerprint. `open_export` opens it with no
  store, refusing an expired export or a former control table.
- **Retention.** Maintainers of a workspace serialize on a transaction advisory lock and
  advance `maintenance_epoch` before any effect. *Retire*: mark a publication expiring
  (never the head), wait for its leases, remove the tables only it selects (outputs, and
  inputs whose writer is already deleted), mark it deleted. *Collect*: fix every
  protected range (selected versions, change windows, live intents' prefixes), then
  fence, checkpoint and vacuum each selected table keeping them. *Reclaim*: abandon
  intents that can never commit, remove their prefixes, mark them reclaimed. Every step
  is idempotent; an interrupted run completes on rerun.

```bash
just pse-publication export --publication <hex> --destination file:///exports/x/ --valid-for-seconds 86400
just pse-publication release --receipt '<receipt json>'
just pse-publication retire --workspace <name> --publication <hex> [--wait-seconds 60]
just pse-publication collect --workspace <name>
just pse-publication reclaim --workspace <name>
```

## Tests

```bash
just db-test                                  # every pse-operations test
just db-test 'test(two_workers_never_claim_same_job)'
just unit-package pse-operations 'test(transition_unit)'   # pure; no database
just publication-test                         # publication catalog journeys (pse-runtime)
```

`pse_operations::testing::FaultProxy` (feature `test-support`) relays a store connection
and drops it just before or just after a marked transaction's `COMMIT`, reproducing a
lost acknowledgement; `pse_testkit::fault_store::FaultStore` injects one-shot object
store faults, including failed deletions and listings.

Every store test, in this crate and in others (feature `test-support`), uses
`pse_operations::testing::TestDatabase`. It creates a database named `pse_test_<uuid>`
(the name is minted, never caller text) on the server `PSE_DATABASE_URL` names (else the
default), opens it with `Store::open` from the generated schema, and hands out dedicated
sessions for test-authored SQL. `remove()` stops the store's listener, closes its pool
and drops the database; a failed test leaves its database in place for inspection. The
nextest `store` group bounds how many such tests run at once, since each holds a small
pool, a listener and sessions against the server's 100 connections.

## Backup and restore

```bash
just db-backup ~/backups/pse        # pg_dump -Fc; prints the dump file path
just db-restore ~/backups/pse/pse-20260927T120000Z.dump   # asks for confirmation
```

Backups are dump-based (`pg_dump --format=custom`). A dump contains operational state, so
keep dumps outside the working copy. `db-restore` runs `pg_restore --clean --if-exists
--single-transaction --no-owner` against the store. It replaces the objects in the dump
and preserves their privileges, including the append-only revoke. Point-in-time recovery
is out of scope until a remote deployment needs it.

## Doctor

`just doctor` (and the direnv status block) includes an `opstore` line. It is a
**warning, never a blocking failure**, because ephemeral work does not need the store.
It reports:

- the server version (18 or newer);
- whether the server is reachable;
- the recorded schema fingerprint, compared with `SCHEMA_FINGERPRINT_HEX` in
  `crates/pse-operations/src/generated/fingerprint.rs`. A missing schema is fine (the
  first durable open creates it); a different one points to `just db-reset`.

It probes only a Unix socket or a loopback host, with a two-second connect timeout. For a
remote store it says so and points to `just db-status`.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `role "…" does not exist` or `Peer authentication failed for user "…"` | No role for your OS user. Run `just db-bootstrap`. |
| `permission denied to create database` in tests | The role lacks `CREATEDB`. Rerun `just db-bootstrap`. |
| `SchemaMismatch`, or `just db-status` reports `MISMATCH` | The store was created by another build (or by the former migrations). Run `just db-reset`. |
| `just db-status` reports the schema absent | Nothing is wrong: the first durable open creates it. |
| `store queries not checked: PostgreSQL unreachable at …` | Query generation needs the local server. Run `just db-status`. |
| `Peer authentication failed` inside the solver container | `PGUSER` is not passed; the runner script sets it to your user. |
