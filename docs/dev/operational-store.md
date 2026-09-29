# Operational store (PostgreSQL 18)

> **Decision: [ADR-0114](../adr/0114-typed-operational-store.md)** (superseding ADR-0112),
> with [ADR-0115](../adr/0115-registry-typed-identities-and-vocabularies.md) for identity
> domains · contract: blueprint
> [§20.6](../authoritative_design/sections/identity-and-publication.md#section-20-6)
> (the store and durable execution) and
> [§20–§20.5](../authoritative_design/sections/identity-and-publication.md#section-20)
> (publication) · crates `pse-operations`, `pse-operations-queries` (generated),
> `pse-runtime` (`workflow::{durable, worker, study, publication, reading, retention,
> operational_tables, progress}`, binaries `pse-worker` and `pse-publication`)

PostgreSQL owns what changes: attempts, jobs, leases, cancellation requests, live progress
and incumbents, reusable solutions, studies and the publication catalog. Delta owns what
is published. The store holds only regenerable state, so it is never migrated: a schema
change resets it. Durable work needs the store; ephemeral library calls and unit tests do
not. This page is the operator's and developer's guide to the store as built: setting it
up, changing it, running workers, publishing and maintaining publications, querying it,
and testing it. The blueprint sections above own the contract; this page does not restate
their rationale.

## One-time setup

```bash
just db-bootstrap        # asks for confirmation, then runs psql as `postgres` via sudo
just db-status           # PostgreSQL >= 18, reachable, schema fingerprint; exit 0
```

`db-bootstrap` creates the following. It is idempotent: running it again only restates
the same attributes and ownership.

- A login role named after your OS user (`$USER`) with `CREATEDB`. The test harness and
  query generation need `CREATEDB` to create their own databases.
- Database `pse`, owned by that role.

Authentication is **peer** over the local Unix socket (`/var/run/postgresql`). The server
maps your OS user to the same-named role, so no password or other credential exists
anywhere. This relies on the Debian/Ubuntu default `local all all peer` line in
`pg_hba.conf`. Run the recipe interactively (sudo asks for your password), or
non-interactively with `just --yes db-bootstrap` in a shell where sudo does not prompt.

The first durable open creates the schema, so a fresh database needs nothing else.

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
recipes, `pse-worker`, `pse-publication`, query generation (`cargo xtask codegen`) and the
doctor check read it from there. The client is tokio-postgres, which reads neither
`~/.pgpass` nor libpq service files (`service=pse`), so the Rust side is configured by
URL. The one libpq variable the store honours is `PGUSER`, as the role when the URL names
none: the solver container runs as a user without a role of its own and is
peer-authenticated as the host user that way (`scripts/native-solver-runner.sh`). A
remote password therefore comes from a machine-local `PSE_DATABASE_URL` (in the gitignored
`.envrc.local`), never from the repository.

**Never commit a credential.** No URL containing a password belongs in the repository,
in `.envrc`, or in a recipe. Machine-local overrides go in the gitignored
`.envrc.local`.

## Schema: generated, created or refused, never migrated

The registry owns the meaning and the shape of every operational relation: relation
`runtime.operational_<t>` (declared in `crates/pse-schema/src/catalog/operations.rs`) is
table `pse_ops.<t>`, 21 tables in all. `just codegen` renders the store from those
declarations into `crates/pse-operations/src/generated/` (generated; never edit it):

- `schema.sql`: the `pse_ops` schema; one PostgreSQL ENUM type per registry enumeration a
  store column uses; a `content_hash` domain (32-byte `bytea`); one domain per entity
  identity (`attempt_id`, `job_id`, ... over `uuid`, and `source_bundle_id` over
  `content_hash`); and every table with NOT NULL and named primary-key, unique,
  foreign-key and CHECK constraints. The CHECKs are the registry's named row checks and
  field domains. Every `double precision` column has a finiteness check, and every array
  column one over its elements, so "no bound" is NULL, never an infinity.
- `copy.rs`: the binary `COPY` statement of every table and a `WHERE false` probe that
  types the stream.
- `cornucopia.toml`: the type mapping from those types to the registry's Rust types.
- `fingerprint.rs`: `SCHEMA_FINGERPRINT`, a digest (frame `pse.ops.schema.v1`) of
  `schema.sql` and `physical.sql`.

`crates/pse-operations/physical.sql` is written by hand: indexes, partial indexes, column
defaults and the append-only revoke on `pse_ops.attempt_transitions`. Its enum literals are
checked when it is applied, and the fingerprint covers it.

**Create or refuse, never migrate** (ADR-0114 Outcome 23):

- `Store::open`, which every durable runtime, worker and maintainer calls, creates the
  schema on a database that has none. It applies `schema.sql` and `physical.sql` in one
  transaction and records the fingerprint as the schema's comment
  (`pse.ops.schema.v1 <hex>`).
- A store whose recorded fingerprint differs (or has none: a schema this build did not
  create) is refused with `SchemaMismatch`. It is never reset implicitly.
- `just db-reset` asks for confirmation, then drops `pse_ops` with everything in it and
  creates the schema again (`pse-ops reset`). Without a terminal, run
  `just --yes db-reset`. Every attempt, job, stream, solution, study and catalog row is
  lost; published members are regenerated by rerunning. Long-lived processes (workers, a
  Python session holding an `OperationalStore`) need a restart after a reset.

Transition legality lives in the Rust tables of `pse_operations::lifecycle`, not in SQL.
Deletes are explicit: there is no `ON DELETE CASCADE`, and no default mints an identity.

## Changing the store

| You change | Then |
|---|---|
| A store declaration in the registry, the `postgres` generator or `physical.sql` | Regenerate in bootstrap order (below), then `just --yes db-reset` for each existing store |
| A statement in `crates/pse-operations/queries/*.sql` | `just codegen` (the query crate); the schema and fingerprint do not move |
| Only Rust in `pse-operations` | Nothing generated; `just check-package pse-operations` and `just db-test` |

**Bootstrap order.** The full generator's build links `pse-runtime`, which links the query
crate, so a stale query crate stops it from building. When the store schema changes,
regenerate the query crate first with the xtask build that does not link the runtime,
then the rest:

```bash
cargo run -p xtask --no-default-features -- codegen --only queries   # needs the local server
just codegen-bootstrap        # or `just codegen` when no Rust contract changed
just --yes db-reset
```

`--only queries` renders the store schema in memory, so it works before the committed
`schema.sql` is regenerated. The checks are `just codegen-postgres-check` (no database)
and `just codegen-queries-check` (needs the server, and fails rather than passes without
one); both are part of `just codegen-check`.

## Statements

Every statement is SQL in `crates/pse-operations/queries/*.sql`, one file per repository
(`store`, `attempts`, `jobs`, `cancellation`, `streams`, `solutions`, `sources`,
`catalog`, `studies`, and `tables` for the query surface), with Cornucopia's named
parameters and hand-annotated nullability (ADR-0114 Outcome 24). `cargo xtask codegen`
runs Cornucopia 1.0.1 as a library under the workspace lockfile: it renders the store
schema in memory, creates a temporary database on the local server from it and
`physical.sql`, prepares every statement there, and writes the generated crate
`crates/pse-operations-queries/` (generated and committed; never edit it). xtask writes the
crate's manifest itself (workspace inheritance, `[lints] workspace = true`) and a `lib.rs`
with a reasoned crate-level `allow`, and re-prints every other file with prettyplease, so
the committed bytes never depend on a `$PATH` tool. A misspelled column, a misspelled
ENUM literal or a mistyped parameter fails generation.

- **Rows.** A whole-row statement (`SELECT a FROM pse_ops.attempts AS a ...`,
  `RETURNING s`) returns the registry row (`RuntimeOperationalAttemptsRow`, ...): the
  Cornucopia mapping names each table's row type, and a generated composite `FromSql` in
  `pse-model` decodes it, checking the field names against the registry.
- **Typed parameters.** An identity parameter compared with a column is cast to its
  domain (`:attempt_id::pse_ops.attempt_id`) so the generated function takes the typed
  id; ENUM parameters take the registry enums. A domain-typed result column arrives as
  its base type, so a projected id is wrapped where it is read.
- **Enum literals** appear only where a partial index needs them (the job claim and the
  stale sweeps); PostgreSQL checks them when the statement is prepared.
- **Cornucopia limits, worked around.** A statement returning `void` is refused, so
  advisory-lock statements select `true` from `pg_advisory_xact_lock(...)`. A projection
  that mixes a whole mapped row with another column generates uncompilable code, so the
  catalog statements project columns.

## Client stack

- **Pool.** deadpool-postgres over tokio-postgres (`application_name = pse-operations`,
  `idle_in_transaction_session_timeout` from `StoreOptions`). A pooled connection is
  verified before it is handed out again and replaced if it does not answer within a
  second: a cancelled operation may leave its statement running on the server, and the
  next caller would queue behind it (`dropped_statement_does_not_block_the_pool`).
- **TLS.** rustls with the ring provider named explicitly and the webpki roots; a local
  socket never negotiates TLS.
- **Listener.** One task per store (`application_name = pse-operations-listener`) owns a
  dedicated connection, holds `LISTEN pse_ops_cancel` and `LISTEN pse_ops_progress`,
  reconnects with capped exponential backoff (50 ms to 5 s) and, after every successful
  `LISTEN`, broadcasts a resynchronization (Plan 22 X8). Cancellation and progress
  watchers re-read the stored state on a matching notification, on a resynchronization and
  when they fall behind the broadcast: a notification only shortens latency, and whatever
  was sent while the connection was down is gone. NOTIFY payloads are 32-character
  hexadecimal ids. Dropping the store stops the task.
- **Errors.** Failures are classified by `SqlState` constant: 40001/40P01 retryable,
  55P03 lock unavailable, 57014 cancelled, 23505 duplicate, 23514/23503 a typed
  `InvariantViolation` naming the table and the registry rule, class 08, 57P01–57P03,
  53300 or a closed connection unavailable (naming the target without credentials),
  anything else internal (ADR-0114 Outcome 27; blueprint §23.2). Retry decisions read
  `OperationsError::is_retryable`. No driver type appears in the public API.

## Binary COPY

Progress events, progress values, incumbents, source documents, publication members and
change windows are inserted with tokio-postgres' `BinaryCopyInWriter`, using the generated
statement and column list of `generated/copy.rs`; the `WHERE false` probe supplies the
column types the binary stream is written in. Producers number their rows, and the
sequence numbers already stored are read first in the same transaction, so a re-sent batch
is idempotent. Numeric progress values are typed columns in the `runtime.solve_metrics`
value vocabulary, so every number round-trips exactly; a nonfinite real is stored as the
unavailable reason `nonfinite`, never as a number.

## Durable runtimes and attempts

A runtime is `Ephemeral` (no store; in-memory progress; it cannot publish) or `Durable`:
every run is then an attempt registered in the store before any effect. The class is an
explicit policy, never a fallback. In Rust, `Operations::connect(url, worker, policy)`
connects, opens the schema and runs recovery; in Python, pass the store:

```python
import pse

runtime = pse.Runtime(settings, store=pse.OperationalStore())
runtime.runs()          # durable attempts, newest first (runtime.operational_attempts rows)
```

- **Lifecycle.** planned → queued → running → {completed, partial, failed, cancelled};
  running → stale on lease expiry; stale → superseded by a new attempt; planned and queued
  attempts cancel at once. Every change is written to the append-only
  `attempt_transitions`. A study's own attempt follows the second table, `COORDINATING`: it
  holds no lease, stays queued while its points run and ends from queued, so it never goes
  stale (blueprint §20.6).
- **Terminations.** `termination_class` selects exactly one typed column (native
  termination, run state, trajectory termination, a runtime outcome such as `cancelled`,
  `infrastructure` or `unattempted`, or the `DiagnosticCode` of a violated rule), with a
  versioned `termination_detail` document that names the violated rule.
- **Leases and recovery.** A running attempt holds one lease, renewed by a heartbeat that
  also returns `cancel_requested`. `LeasePolicy` defaults: 30 s lease, 10 s heartbeat,
  progress batches of 512 events or 200 ms. Recovery at every durable connect requeues the
  jobs of expired leases as new attempts, marks other expired running attempts stale, and
  removes the streams of attempts finished more than seven days ago.
- **Cancellation.** `cancel_requested` is the authority; the listener only shortens
  latency. A cancel reaches a try running in another process.

## Jobs and `pse-worker`

A job names its current try's attempt and a typed payload (`JobPayload` version 3, JSON
Schema `docs/generated/schema/job-payload.schema.json`): a `ModelingJob` (the physical and
modeling source bundles by package content hash, the case, route, typed `SolveSettings`, a
`JobStart` of `fresh`, `resume_from_parent` or `stored_solution`, and for a study point its
`StudyPointBinding`) or a study finalization. A worker refuses a version it does not know
(`UnknownPayloadVersion`). `Operations::put_sources` stores a package's documents as a
content-addressed source bundle; `Operations::enqueue(job, idempotency_key, retry,
priority)` frames the request identity from the typed job and creates the job with its
first attempt, and the same idempotency key returns the existing job. The retry policy is a
maximum number of tries and an exponential backoff with a cap.

Workers claim queued jobs by priority, then availability, with `FOR UPDATE SKIP LOCKED`.
Each try is a new attempt under the worker's lease; its end goes through the job's retry
policy.

```bash
just pse-worker --until-idle                 # serve the queue until it is empty
just pse-worker --jobs 10 --memory-mib 8192  # stop after ten jobs; SCIP needs the larger budget
```

`pse-worker` options: `--url`, `--name` (default `pse-worker:<host>:<pid>`),
`--lease-seconds` (30), `--heartbeat-ms` (10000), `--poll-ms` (500), `--jobs`,
`--until-idle`, `--memory-mib` (4096) and `--threads`. The recipe sources the native
execution environment and builds with `native-solvers`. The worker sets the process-level
OpenMP environment SPRAL needs (`OMP_CANCELLATION`, `OMP_PROC_BIND`, `OMP_PLACES`,
`HWLOC_COMPONENTS`) before any thread starts, re-executing itself when they are missing;
an operator's explicit values stand. It sweeps expired leases at start-up and
periodically. In-process, `Runtime::work` (Python `Runtime.work(jobs=None)`) serves the
queue the same way.

## Progress, incumbents and stored solutions

- **Progress** streams to `progress_events` and `progress_values` without an event cap;
  publication snapshots it into `runtime.solve_metrics` under namespace
  `event.<seq>.<phase>`.
- **Incumbents.** A branch-and-bound search (SCIP; HiGHS on MIP callback kind 4) reports
  typed incumbents: objective with the export offset applied, dual bound, gap, nodes,
  native seconds and a throttled primal (the first at once, then at most one a second, the
  last always kept). Each captured primal is stored as a seed of its step in
  `solutions` with origin `incumbent`, in the same transaction as its `incumbents` row,
  which keeps the step, phase and elapsed time of the reporting event. A durable run
  publishes its incumbent stream as `runtime.incumbents`, read back when the attempt ends.
- **Stored solutions** are keyed by the coordinate-compatibility stamp and the preparation
  identity; `origin` is `output` for a step's accepted seed and `incumbent` for a capture.
  `with_stored_start` starts a step from the newest compatible output seed
  (`StoredStart::Latest` skips captures) or a named one (`StartSource::Stored`). A job with
  `JobStart::ResumeFromParent` starts from the latest incumbent in its parent attempt
  chain, so a killed worker's successor resumes the search; the try's `job.start` event
  records the start it used.
- **Retention.** Streams of finished attempts, their incumbents included, are removed
  after seven days, except the incumbents of the attempt chain above an unfinished retry.
  Captured solutions go with them unless an unfinished job's stored start or a waiting
  study point's predecessor names them. Output seeds are never pruned; register R-36 holds
  the trigger for an automatic policy for them and for publications.

## Publication catalog

Publication visibility, heads, reader protection and retention are catalog rows (Plan 22
X9–X12). Delta holds only immutable member tables and export manifests. `pse-operations`
owns the catalog statements, `pse-catalog` performs Delta member I/O and never sees
PostgreSQL, and `pse-runtime` composes the two (`Runtime::register_workspace`,
`RunResult::prepare_publication`, `open`, `open_head`, `export_publication`,
`retire_publications`, `collect`, `reclaim_unpublished`). Only a durable runtime
publishes.

- **Workspaces.** `workspaces` rows name one publication history with one head and the
  root its members are written under (`publication_heads`). A root that still holds a
  Delta control table (`{root}control/_delta_log/`) is refused as an unsupported
  historical format (`LegacyWorkspace`): regenerate its publications by rerunning them
  into a new root.
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
  already published as another publication is `PublicationIdentityReused`; a retained
  input no live publication protects is `InputRetired`.
- **Settlement.** A lost commit acknowledgement is reported as `PublicationUnresolved` and
  never retried implicitly. Settling the ticket (`settle_publication`) queries the catalog:
  committed, proved not committed (no intent, an abandoned intent, or the head still at
  the parent with the intent locked, so no commit is in flight), conflict (the head moved),
  or unresolved when the store is unreachable. It writes only the `settlements` row.
- **Readers.** A reader takes a `reader_leases` row (`open`, `open_head`) in one short
  transaction that returns the complete record and the workspace's maintenance epoch,
  then reads Delta without a database session. The lease (60 s) is renewed at a third of
  its lifetime and released on drop; a lapsed lease cancels the reader. The pair
  (workspace, epoch) is the session's `ReadScope`, the lookup input of every shared
  snapshot, resident and file-metadata cache; a session without one bypasses them.
- **Exports.** `export_publication` takes a lease held by `export:<destination>` for a
  stated time and writes a one-row manifest (Delta version 1) with the record, the
  lease, its expiry, the epoch and the store fingerprint. `open_export` opens it with no
  store, refusing an expired export or a former control table.

Python: `Runtime.register_workspace(name, root)`, `workspace(name)`, `head(workspace_id)`,
`RunResult.prepare_publication(...)` then `PublicationAttempt.commit()`,
`settle_publication(ticket)`, `open(publication_id)`, `open_head(workspace_id)`,
`export_publication(publication_id, destination, valid_for=)` and
`pse.open_export(location, settings=)`.

## Retention and maintenance

Maintainers of a workspace serialize on a transaction advisory lock and advance
`maintenance_epoch` before any effect. The catalog computes what stays reachable for three
reasons: the versions a live publication selects, the prefixes of live intents, and the
change windows a live publication read. Every step is idempotent; an interrupted run
completes on rerun (`interrupted_deletion_resumes`).

- **Retire.** Mark a publication expiring (never the head), wait for its leases, remove
  the tables only it selects (outputs, and inputs whose writer is already deleted), mark
  it deleted.
- **Collect.** Fix every protected range, then verify, fence, checkpoint and vacuum each
  selected table keeping them. A protected version whose history is gone refuses that
  table.
- **Reclaim.** Abandon intents that can never commit (abandoned, attempt stale or
  superseded, or published as another publication), remove their prefixes, mark them
  reclaimed.

```bash
just pse-publication export --publication <hex> --destination file:///exports/x/ --valid-for-seconds 86400
just pse-publication release --receipt '<receipt json>'
just pse-publication retire --workspace <name> --publication <hex> [--wait-seconds 60]
just pse-publication collect --workspace <name>
just pse-publication reclaim --workspace <name>
```

Nothing retires publications automatically: no policy decides which publications to retire
(register R-36). Captured incumbent solutions expire with their attempt's streams unless a
live row, job or waiting study point still names them; output seeds are never pruned. The protocol is exercised on local file tables
and the in-memory object store; remote object stores (S3-compatible) are not qualified
(register R-37).

## Studies across workers

A durable study (Plan 22 O7, architecture S15) is many authored-case solves run by any
number of workers and published once. `Runtime::start_study` (Python
`ModelingPackage.study(..., runtime=, workspace=)`) stores the package's sources and a
typed `StudyDefinition`, then creates in one transaction (`pse_operations::studies`):

- the study's own attempt (kind `study`). It coordinates and never runs: it is queued
  while the points run and ends from queued (`lifecycle::COORDINATING`) in the
  transaction that makes the last point terminal — completed, partial, failed, or
  cancelled when the study was cancelled. It never holds a lease, so it never goes
  stale, and its publication can always commit once it has ended;
- the study's one publication intent for that attempt, registered before any point
  writes a member under its prefix (X9);
- one job per point with payload v3 (`JobTask::Modeling` with a `StudyPointBinding`:
  study, point index, binding hash, typed overlay, predecessor). A point without a
  predecessor is queued; a point with one waits (job `waiting`, attempt `planned`);
- a waiting finalization job (attempt kind `study_finalization`).

A point follows its job in the transaction that moves the job — claimed (`assigned`),
requeued after a lost lease (`pending`), finished or cancelled. A completed point
releases the points that name it as their predecessor, which then start from its stored
solution (`StartSource::Stored`; the `job.start` event records `predecessor` and the
solution, or why the point started fresh). A point that fails or is cancelled cancels
its dependents transitively as `unattempted`, each job recording which predecessor did
not complete. A completed try writes its result tables under
`{prefix}points/{index}/{attempt}/` (catalog `point_{index}`, receipts naming the
point's attempt) and records them in `study_point_members` with the completion. When the
last point is terminal the finalization is released: it writes `runtime.study_outcomes`
(one row per point) under `{prefix}summary/` and commits one publication of the study's
attempt with the summary and every completed point's members; failed points contribute
none. Point transitions of one study serialize on the study row. Cancelling a study
(`StudyHandle::cancel`, or `request_cancel` of its attempt) cancels the points that have
not started, asks running tries to stop, and still publishes what completed. A package
changed in memory (`with_declarations`, `with_fit_data`, `with_limits`) cannot be run as a
durable study, because workers load its authored documents.

Known gaps: a crashed point try's partial member tables are never collected; a
finalization that exhausts its retries leaves the study `concluded` with no automatic
recovery; the 10 000-point scale is unmeasured.

```bash
just worker-test study_parallel_workers_publish_once   # two pse-worker processes
just db-test 'test(study_tests)'                       # the repository
```

## Query surface

A durable runtime's query sessions see the operational relations as read-only DataFusion
tables under the schema `pse_ops` (Plan 22 O9): `attempts`, `attempt_transitions`,
`jobs`, `progress_events`, `progress_values`, `incumbents`, `solutions`, `studies`,
`study_points`, `workspaces`, `publications`, `publication_members` and `settlements`.
`Runtime::query_session` builds one, optionally over a publication's session and a run's
retained result relations (`workspace.<namespace>.<name>`), so SQL joins the store with
results; Python's `Runtime.query(sql, result=, publication=)` streams its answer. An
ephemeral runtime binds no `pse_ops` tables, and publication sessions do not bind them by
default. Python also lists `Runtime.jobs(states=, limit=)` and `Runtime.studies()`.

```python
stream = runtime.query(
    "SELECT a.attempt_id, a.state, count(e.seq) AS events "
    "FROM pse_ops.attempts a LEFT JOIN pse_ops.progress_events e USING (attempt_id) "
    "GROUP BY a.attempt_id, a.state"
)
```

- **Scans.** Each table's provider runs its generated statement
  (`queries/tables.sql`, `pse_operations::tables`) and builds each page with the
  relation's generated `pse-relations` builder, so values are checked against their
  registry field contracts on the way in. A scan reads pages of at most the session's
  batch size (64 rows for solutions, whose vectors can be long) after a primary-key
  position; it holds no connection between pages and honours a pushed limit. Pages are
  read at READ COMMITTED: a row that existed throughout the scan is read exactly once,
  but the scan is not one snapshot.
- **Pushdown.** Equality and `IN` on identity columns and state vocabularies, and
  comparisons and `BETWEEN` on the time column, become the statement's typed filter.
  They are `Inexact`: the statement returns a superset and DataFusion applies every filter
  again. Anything else is evaluated by DataFusion over the unfiltered pages, and a filter
  that cannot match runs no statement.
- **Freshness.** The providers declare no constraints, so a session binds them as
  observed native sources and a query that reads one is never served from a cache.
- **Streams.** `Runtime::progress` (Python `Runtime.progress(attempt_id, follow=True,
  page=256)`, a closable `ProgressStream`) reads an attempt's progress events and
  incumbents in observation order, a bounded page at a time; following waits on the
  store's listener until the attempt stops working (and up to five seconds for the attempt
  to appear), and closing the stream ends a waiting read. Each incumbent keeps the progress
  context of the event that reported it (step, phase, elapsed time) and the search's node
  count and running time; Python's `ProgressEvent.incumbent` is a `pse.Incumbent`.

**ADBC is not adopted.** The ADBC PostgreSQL driver (`adbc_core`/`adbc_driver_manager`
0.24, which fit arrow 59) would read these tables into Arrow directly, but it adds a C
driver manager and a second PostgreSQL client for tables the generated builders already
serve with registry-checked values, typed parameters and the store's own pool.
`datafusion-table-providers` is not usable either: its release requires datafusion 54
and arrow 58, which would split the one type universe. Revisit when a release matches
`datafusion =55.1.0`, or if a relation must be read at a volume the per-row builders
cannot sustain. Statement performance at scale is unmeasured.

```bash
just unit-native-package pse-runtime pse-runtime/native-solvers 'test(operational_tables)'
just db-test 'test(tables_tests)'
```

## Tests

```bash
just db-test                                  # every pse-operations test
just db-test 'test(two_workers_never_claim_same_job)'
just unit-package pse-operations 'test(transition_unit)'   # pure; no database
just worker-test                              # pse-worker processes: resume, cancel, studies
just publication-test                         # publication catalog journeys (pse-runtime)
```

`pse_operations::testing::FaultProxy` (feature `test-support`) relays a store connection
and drops it just before or just after a marked transaction's `COMMIT`, reproducing a
lost acknowledgement; `pse_testkit::fault_store::FaultStore` injects one-shot object
store faults, including failed deletions and listings. The worker tests spawn `pse-worker`
processes, kill one once an incumbent is stored, and need the solver image
(`PSE_SOLVER_IMAGE`) for SCIP.

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
and preserves their privileges, including the append-only revoke. A dump taken under
another schema fingerprint restores a store this build refuses; reset instead. Point-in-time
recovery is out of scope until a remote deployment needs it.

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
| `permission denied to create database` in tests or generation | The role lacks `CREATEDB`. Rerun `just db-bootstrap`. |
| `SchemaMismatch`, or `just db-status` reports `MISMATCH` | The store was created by another build. Run `just --yes db-reset` and restart long-lived processes. |
| `just db-status` reports the schema absent | Nothing is wrong: the first durable open creates it. |
| `just db-reset` fails with no terminal | The recipe asks for confirmation; run `just --yes db-reset`. |
| The full `cargo xtask codegen` does not build after a store change | The query crate is stale; regenerate it first in bootstrap order (`--only queries`). |
| `store queries not checked: PostgreSQL unreachable at …` | Query generation needs the local server. Run `just db-status`. |
| Python `OperationalStore` tests fail after a registry change | The dev store predates the new schema; reset it. |
| `Peer authentication failed` inside the solver container | `PGUSER` is not passed; the runner script sets it to your user. |
| `LegacyWorkspace` when registering a workspace | The root holds a former Delta control table; publish into a new root and rerun. |
| `ReaderLeaseLapsed` during a long read | The lease could not be renewed (store unreachable, or the process stalled); reopen the publication. |
