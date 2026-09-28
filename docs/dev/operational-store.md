# Operational store (PostgreSQL 18)

> **Decision: [ADR-0114](../adr/0114-typed-operational-store.md)** (superseding ADR-0112)
> · [Plan 22 architecture §9](../plans/22-solver-capabilities-architecture.md#9-operational-store-and-publication-catalog-postgresql-18)
> · crate `pse-operations`
>
> This page describes the store as built with sqlx and versioned migrations. Plan 22 B1 and
> B2 replace that setup:
> - a schema generated from the registry, created or refused by fingerprint;
> - `just db-reset` instead of `db-migrate`;
> - statements compiled by Cornucopia on tokio-postgres;
> - a test harness over the generated schema.
>
> The page is rewritten when those packets land.

PostgreSQL owns what changes (attempts, jobs, leases, cancellation requests, live progress,
incumbents, reusable solutions, study status and the publication catalog); Delta owns what
is published. Durable work needs the store. Ephemeral library calls and unit tests do not.
This page covers setting it up and operating it on a development machine.

## One-time setup

```bash
just db-bootstrap        # asks for confirmation, then runs psql as `postgres` via sudo
just db-migrate          # applies the embedded pse_ops migrations
just db-status           # PostgreSQL >= 18, reachable, no pending migrations; exit 0
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

> **Deviation from ADR-0112 Outcome 20.** The decision names a login role `pse` using
> SCRAM. Local development uses a peer-authenticated role named after the OS user
> instead, so no secret has to be created, stored or rotated. A remote deployment uses
> SCRAM with the password in `~/.pgpass`, never in the repository or the URL.

## Connection URL

| Source | Value |
|---|---|
| Default | `postgres:///pse?host=/var/run/postgresql` (database `pse`, local socket, the OS user) |
| Override | `PSE_DATABASE_URL`, e.g. `postgres://pse@db.internal/pse?sslmode=verify-full` |

The default is declared once, as `pse_operations::DEFAULT_DATABASE_URL`. The `db-*`
recipes and the doctor check read it from there. sqlx reads `~/.pgpass` for a password.
It does not read libpq service files (`service=pse`), so the Rust side is always configured
by URL.

**Never commit a credential.** No URL containing a password belongs in the repository,
in `.envrc`, or in a recipe. Machine-local overrides go in the gitignored
`.envrc.local`.

## Schema and migrations

The embedded migrations live in `crates/pse-operations/migrations/` and create schema
`pse_ops`. `sqlx::migrate!` embeds them into the crate, and `just db-migrate` (the `pse-ops`
binary) applies them. sqlx records applied versions and checksums in
`public._sqlx_migrations`.

- **Never edit an applied migration.** Its checksum is part of the schema identity, and
  `just db-status` reports a changed file as `EDITED`. Add a new timestamped file instead.
- Migration files use LF line endings; the repository's `.gitattributes` forces LF.
- Enumerations are `text` columns with a `CHECK` on their value domain. Transition legality
  lives in the Rust transition table (`pse_operations::lifecycle`), not in SQL.
- `pse_ops.attempt_transitions` is append-only: the migration revokes `UPDATE`, `DELETE`
  and `TRUNCATE` from the role that owns it.

## Tests

```bash
just db-test                                  # every pse-operations test
just db-test 'test(two_workers_never_claim_same_job)'
just unit-package pse-operations 'test(transition_unit)'   # pure; no database
```

`db-test` maps `PSE_DATABASE_URL` (or the default) to `DATABASE_URL`, which `#[sqlx::test]`
requires, and then runs `just unit-package pse-operations`. Each store test gets a fresh
database named `_sqlx_test_…`, created from the store's database and migrated before the
test runs. The database is dropped when the test passes; a failed test leaves its
database in place for inspection, and the next run of that test replaces it. The
harness keeps its bookkeeping in a `_sqlx_test` schema inside `pse`. That schema is not
operational state, and backups exclude it.

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
- pending or unknown migrations, compared by version with the files in
  `crates/pse-operations/migrations/`.

It probes only a Unix socket or a loopback host, with a two-second connect timeout. For a
remote store it says so and points to `just db-status`, which also compares checksums.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `role "…" does not exist` or `Peer authentication failed for user "…"` | No role for your OS user. Run `just db-bootstrap`. |
| `permission denied to create database` in tests | The role lacks `CREATEDB`. Rerun `just db-bootstrap`. |
| `DATABASE_URL must be set` | You ran a store test outside `just db-test`. Use `just db-test`, or export `DATABASE_URL`. |
| `just db-status` exits 1 with pending migrations | Run `just db-migrate`. |
| `EDITED` or `DIRTY` migrations | An applied migration changed, or one failed part way. Restore from a dump, or recreate the development database and migrate again. |

sqlx-postgres 0.9.0 resolves the default user through `whoami` without its `std` feature,
which makes the default user `anonymous`. The workspace enables `whoami/std` (see the root
`Cargo.toml`) so that peer authentication sees your real user name.
