# Persistence, transport and relational engines

This page covers `pse-operations` (the canonical SurrealDB store and its owned protocol client),
`pse-engine` (DataFusion), `pse-relations` and `pse-columnar`. Contracts are owned by
[§20](../../authoritative_design/sections/identity-and-publication.md#section-20),
[§4](../../authoritative_design/sections/schema-and-relations.md#section-4) and
[§3.3](../../authoritative_design/sections/workspace-and-dependencies.md#section-3-3). The baseline is
on the [entry page](README.md). These opportunities share one premise: SurrealDB stays canonical,
and durability spans multiple processes without a daemon. Every opportunity here is judged within
that premise.

**System picture.** The store has four layers.

1. A database program emitted by `crates/pse-codegen/src/codegen/surreal*.rs` (about 4.5k lines, mostly
   SurrealQL in Rust raw strings) into `generated/surreal.surql`. By grep line count it holds about
   90 tables, 708 fields, 122 indexes, 205 stored functions, 541 `THROW` lines and 199 `FOR UPDATE`
   lines.
2. About 76k lines of Rust operation modules (tests included) that compose inline SurrealQL, binds,
   settlement and hand decoding.
3. An owned protocol client, `canonical_client/` with `canonical_transport`, `transport`,
   `transport_retry` and `transport_lifetime`. It drives `surrealdb-protocol` stubs over an owned
   hyper HTTP/2 service, so the SDK's engine is not in the production graph. This is about 3.7k lines
   excluding its test files.
4. Server integration: 14 ordered patches in `scripts/surrealdb-source/` and a Surrealism WASI policy
   module.

SurrealQL is used in depth: SSI, `SELECT … FOR UPDATE` conflict registration, `DEFINE API` ingress,
enforced relations with inline edge caches, in-transaction policy, `READONLY` and `ASSERT` backstops,
sealed events and compound indexes with plan observation. The original request clock is enforced
inside the transaction. DataFusion is also used deeply: cache-plan extension nodes, `SpillManager`,
custom `TableProvider`s with pushdown and statistics, catalogs, an `ObjectStoreRegistry`, UDFs and the
shared `MemoryPool`. Arrow kernels are called, not re-implemented. `transport_retry::Attempt` already
has the shape of a retry owner whose step supplies classification after settlement.

The following are legitimately custom: the exact `u64` and `Decimal` wire codec, identity hashing, the
retention and protection protocol, and deadline semantics that must survive cancellation.

The opportunities cluster in three places: a large embedded program without store-language tooling
(see [CBO-046](registry-generation-and-python-boundary.md#cbo-046) for the program itself and
[CBO-010](#cbo-010) for its call envelope), read paths realized as durable writes
([CBO-071](#cbo-071), [CBO-072](#cbo-072)), and supplier burden at the client wire and the server
patch series ([CBO-066](#cbo-066), [CBO-070](#cbo-070)).

**Coverage on this page.** Closely read: the client path (`canonical_client*`, `owned.rs` in full,
`outbound.rs`, `errors.rs`), the study capture path and its generated functions, the bodies of
`canonical_projects`, `canonical_results`, `canonical_artifacts`, `canonical_analyses` and
`canonical_project_imports` (long lines truncated), and the Rust of the retention and lifecycle
modules. Read at representative depth: `canonical_script`, `canonical_transport`, `transport_retry`,
`transport_lifetime`, `canonical.rs`, `canonical_selection`, `canonical_studies`, migration and
maintenance heads, and six of the 14 server patches in full. Skimmed or unread: `canonical_staging`
and `canonical_files` (heads only), `progression*.rs`, test modules, the generated retention and
lifecycle function bodies (which hold the actual retention logic), and the tails of the partially read
patches. Nothing was run and no cost is measured. [CBO-011](#cbo-011) and [CBO-012](#cbo-012) were
carried forward and rechecked at heads and by search only.

<a id="cbo-071"></a>

## CBO-071 — A live status poll is realized as a durable, globally gated capture

- **Operation and consumers.** `StudyObserver::poll_checked` (`pse-runtime/src/workflow/study_observation.rs`),
  reached by `follow`, `bin/project/runtime.rs` and `tooling/reader/canonical.rs`. The page documents
  it as one bounded read-only poll. The consumer is the sole operator watching a study.
- **Current mechanism.**
  - `poll_checked` calls `freeze_study_cohort`, then `CanonicalStore::capture_study_workbench`. Each poll
    mints a fresh capture key, so `CAPTURE_OWNERS` deduplication and the `open_capture` replay branch
    never apply to polls.
  - Durable effects of one poll, with P selected points (at most 64): `open_capture` upserts the single
    global row `canonical_study_capture_gate:current` and creates a `canonical_study_captures` row;
    `commit_capture` inserts a header, P member rows and up to P+1 protections, then updates the gate and
    the capture; two `capture_page` reads take `FOR UPDATE`; dropping the custody spawns
    `release_capture`. That is at least three write transactions and about 2P+7 records written, each
    through synchronous AOL under the selected profile.
  - `fn::pse_workbench_v1::publication` throws `study_capture_publication_busy` while the gate holds a
    capture, and appears at 64 sites in the generated schema, including run begin, execution
    publication and project mutations. Between `open_capture` and `commit_capture`, every such writer
    in the database refuses and paces.
  - I found no `DELETE` of `canonical_study_captures`, and `released` is a retained tombstone state, so
    each poll appears to leave one row with its request bytes (search only).
- **Why it matters.** Watching a study adds durable write load, apparently unbounded residue and a global contention point against the lanes being watched (*Proposed*; no cost *Measured*). Whether a lighter read can serve the observer is the question to put, because ADR-0230 chose the gate after an ungated 10,000-member capture failed with a read conflict under active writes. The observer's single-study population of at most 64 members may need less than that case, and that has not been established.
- **Patterns.** LOP-X016, LOP-05, LOP-07, LOP-04.
- **Evidence.** Read `canonical_study_workbench.rs` (`capture_owner`, `capture_study_workbench`,
  `release_capture_as`, `CaptureOwner::drop`, `workbench_script`), the generated `open_capture`,
  `commit_capture`, `capture_page`, `recover_capture`, `release_capture` and `publication`, and
  `study_observation.rs` (`status_page`, `poll_checked`, `follow`). Record counts are from reading.
- **Footprint.** Inspected. `tooling/reader/canonical.rs` and `bin/project/runtime.rs` were read at the
  call line only.
- **Preservation constraints.** The context check and attempt discovery in `poll_checked`; bounded
  response; exact `metadata_revision` per point; refusal on an incomplete population; capture semantics
  for KPI export and cohort pages, which need completeness and result protection; capture-versus-publication
  exclusion wherever a frozen cut is truly needed.
- **Counterevidence and uncertainty.** `StudyObservationStatus` retains `_custody`, so a consumer may rely
  on the frozen cut while reading sources, and not every consumer of the page was traced. The observer may
  have been built on capture to reuse one bounded response owner rather than for its semantics. Whether one
  read script gives header-plus-member consistency under MX needs confirming.
- **Operational gap, suspected from reading and not run.** The gate clears only in `commit_capture` and `recover_capture`, and ADR-0230 states that an expired timestamp never permits writers to bypass an unchanged gate. A process death between open and commit therefore appears to leave the gate set. `open_capture` itself calls `publication`, so the next poll or capture refuses as well as all 64 publication sites. No complete recovery route was found. `recover_study_capture` has no non-test caller, and no route reads the gate row's stored `capture` key, although one `SELECT` would supply it. The key would not be enough: `release_capture` by a different custodian throws `study_capture_original_owner_not_drained` unless the owner row is drained, and `fn::pse_files_v1::drain_owner` is reached only from a live process's own `disconnect`. A refusal text names `recover_file_owners`, which exists only in that string, and `pse-canonical-recovery --operation owner-observations` reports death without writing. §20.4 says interrupted capture state "retains its declared recovery meaning". The search covered non-generated `crates/` and `xtask`, so dynamically composed SurrealQL could escape it.
- **Status.** Current promise in §20.4 without a complete recovery route found; polling through capture is a current mechanism.
- **Research questions.** Does a single read script satisfy the observer's consistency need? Can exact
  tombstones expire after the original grant window? Timer-driven waiting around this call is
  [CBO-016](execution-lifecycle-and-observation.md#cbo-016).

<a id="cbo-066"></a>

## CBO-066 — Query responses pass through five representations behind a hand-kept preflight

- **Operation and consumers.** Every metadata and inline-payload read, through `ResponseLimits`,
  `MetadataResponse` profiles and `retained_bytes` charging by selection and result owners.
- **Current mechanism.**
  - A response frame is walked by a borrowed varint counter (`canonical_client/wire.rs::Counts`) driven by
    a generated, string-matched table of about 390 arms (`generated/grpc_wire_schema.rs`). It is then
    decoded by prost, converted by `Value::try_from`, collected into `IndexedResults` and `Object`, and
    decoded again by generated or hand decoders (about 90 `decode_` functions) into typed rows. Frames are
    held until `QueryEnd` and trailers.
  - `WireBody` in `owned.rs` is a second gRPC deframer: it reassembles HTTP/2 data into a `BytesMut`,
    parses the 5-byte header and runs `Counts::check`, then tonic frames and decodes again. tonic's public
    `Codec` and `Decoder` seam sees each complete message before decode, but the generated stubs hard-code
    `ProstCodec`.
  - Charging is conservative prepaid arithmetic with fixed allowances (for example the 64 MiB
    `ProductBlob::retained_bytes` allowance).
  - The schema generator (`codegen/grpc_wire.rs`) derives wire shapes by parsing the supplier's prost
    attributes with `syn` and bails unless the supplier is exactly 0.13.1.
  - The protocol defines Arrow IPC batches, but the pinned server does not serve them and the client refuses
    them.
- **Why it matters.** The prepay bound exists because prost and value conversion take no budget, so the
  code mirrors both wire and representation costs. Each protocol bump needs schema regeneration and
  revalidated constants, and the charges cover copies a less translated path would not make (*Proposed*;
  not *Measured*).
- **Patterns.** LOP-06, LOP-10, LOP-X001, LOP-X009, LOP-X003, LOP-07.
- **Evidence.** Read `wire.rs` (to about 120 lines), the head of `grpc_wire_schema.rs`, `results.rs` to
  `collect`, `owned.rs` in full, `errors.rs`, `outbound.rs`, the head of `codegen/grpc_wire.rs`, and the
  protocol's `QueryRequest` comments and the server's `query` handler.
- **Footprint.** The client path is inspected. The cost constants across `MetadataResponse` are suspected
  but unread. The tonic fit is a framing check against the generated client, not research.
- **Preservation constraints.** Limits enforced before owned allocation; cumulative byte, node, depth and
  container limits; complete-response publication; substantive-error precedence; fail-closed `LIVE` and
  malformed framing; exact decimal `u64`; `IndexedResults` semantics (§20.6); the prepaid grants held by
  `WireBody`; refusal of compressed responses; bail-on-unknown-wire-form in the generator.
- **Counterevidence and uncertainty.** The prepay contract is real, and post-hoc measurement or prost's
  recursion limit would not discharge it. The walker is small and generated from authority, so the
  string-keyed dispatch may be insignificant. The generator is 317 lines, and the Rust source is what the
  client decodes, so reading it is arguably the stronger binding than a descriptor. Columnar results need a
  server move or patch, and Arrow would suit only row-shaped results, not the receipt-shaped returns of
  guarded functions.
- **Status.** Current mechanism.
- **Research questions.** Can a stateful codec hold cumulative `Counts` and remove the second framing path?
  Do later server versions serve Arrow or a bounded-verification encoding (see [CBO-070](#cbo-070))? Is there
  a prost-adjacent decoder with allocation budgets, or a descriptor source that keeps bail-on-new-form? The
  prepay mirrors in general are [CBO-001](cross-cutting-mechanisms.md#cbo-001).

<a id="cbo-072"></a>

## CBO-072 — Read custody is durable lease rows with their own write and release cycle

- **Operation and consumers.** `read_results`, `read_completion`, `read_artifact` and selected reads, and
  through them every protected result or source read in `pse-runtime`.
- **Current mechanism.** `terminal_read_owner` calls `protect`, which obtains a grant key and writes a
  lease. `read_results` is issued as `QueryEffect::Mutation` and runs `RESULT_ADMISSION`, which upserts
  `canonical_result_protections` and calls `result_creator` (a guard `touch`). `admit_manifest_ticket` adds
  a ticket. `ReadOwner` and `ResultReleaseTask` spawn release on drop, tracked by `ResultReadDrain`;
  `RENEW_PROTECTION` extends leases and `settle_protection_grant` collects residue. Opening a result read
  costs several fsynced transactions plus a release path, and creator guards make readers conflict with
  reclaimers by design.
- **Why it matters.** This is reader registration against garbage collection, rebuilt above an MVCC store
  because snapshots cannot outlive a transaction and transactions have a finite age (`max_transaction_age`,
  90 s by default in `mx-1.0-controls.patch`). Plausible gains (*Proposed*): fewer writes on read paths
  through a coarser scheme, such as one process-scoped lease covering many reads, deferred reclamation by
  epoch, or a read-only proof of no reclaim using `reclaimed_ranges`.
- **Patterns.** LOP-X016, LOP-08, LOP-03.
- **Evidence.** Read `canonical_results.rs` (`terminal_read_owner`, `read_results`, `read_completion`,
  `RESULT_ADMISSION`, `RENEW_PROTECTION`), `canonical_artifacts.rs::read_artifact`,
  `canonical.rs::protection_grant_key` and retention functions by call site.
- **Footprint.** Read-path owners inspected; per-path counts not established. The heads of `protect` and
  `admit_manifest_ticket` were read by call site only, and the generated retention function bodies were not
  read.
- **Preservation constraints.** No reclamation under a live reader, across processes; lease expiry
  independent of client health; restore fencing; exact ticket and manifest binding; no scientific bytes
  reachable after release; caller-outliving semantics.
- **Counterevidence and uncertainty.** Results are opened rarely relative to bytes read, and file reads after
  opening need no store round trips, so absolute cost may be irrelevant. Multi-process coordination without a
  daemon needs durable state of some kind. No library is implied; this may be a pure design question.
- **Status.** Current mechanism.
- **Research questions.** Which reclamation guarantees need a per-read lease and which could be proved by
  epoch? The write-for-read pattern is shared with [CBO-071](#cbo-071).

<a id="cbo-070"></a>

## CBO-070 — Server patch series carries integration later releases may absorb

- **Operation and consumers.** Server preparation by `pse_host::server` from `scripts/surrealdb-source/*`.
- **Current mechanism.** Fourteen ordered patches, about 2,863 lines and about 1,050 changed lines by `^[+-]`
  count (lockfile and tests included). By hunks read:
  - New server integration: `grpc-connection-ownership` adds `ntw/owned_connection.rs` (`OwnedAcceptor`,
    `ConnectionOwner`, cleanup on stream drop) and session binding. It is the largest substantive patch.
  - Upstream bug class: `grpc-session-admission` (a racy `len()` cap becomes a semaphore reservation) and
    `api-failure-rollback`. Both look generally upstreamable.
  - Observability: `grpc-status-details`, whose production weight looks smaller than its size.
  - Engine move: `mx-1.0-minimal` and `mx-1.0-controls`, thin adapter edits.
  - Policy: `canonical-profile`, kept in the server because `ConfigMap` silently ignores invalid values.
  - Runtime limits: three `surrealism-*` patches for admission, deadlines and one linear memory.
  - Build only: `pinned-compiler` (50 files), `diskann-0.56.0`, `surrealism-locked-build`,
    `surrealism-build-features`.
  None rewrites the query engine. About half the patch lines are mechanical.
- **Why it matters.** A supplier move requalifies this series together with the client's `xtask grpc-wire`
  schema and the `Error::from_wire` hold. Upgrade burden concentrates in `owned_connection`, the module
  runtime patches and the adapter. Columnar results may be reachable only through such an upgrade
  ([CBO-066](#cbo-066)), and upstream shows active gRPC work (*Proposed*).
- **Patterns.** LOP-11, LOP-X004 (inverted: the patch is server-side with no public seam), LOP-10.
- **Evidence.** Hunks read in full for `grpc-session-admission`, `api-failure-rollback`, `canonical-profile`,
  `diskann-0.56.0`, `grpc-status-details` and `surrealism-memory-count`; the first 40 to 110 changed lines of
  the other eight.
- **Footprint.** The `rpc/grpc.rs` half of connection ownership and the tails of controller-deadline and
  global-controllers are unread. A whitespace-insensitive diff was not computed. No upstream release research
  was done.
- **Preservation constraints.** Each patch's exact semantics; the explicit SI-to-SSI stage; provenance;
  unknown-commit truthfulness.
- **Counterevidence and uncertainty.** Upstream state is unknown. The pin is deliberate, and a newer server
  also needs a protocol bump, wire-schema regeneration and full qualification.
- **Status.** Current mechanism.
- **Research questions.** For each patch, the first upstream release, if any, with the same guarantee. Which
  are suitable to contribute?

<a id="cbo-069"></a>

## CBO-069 — Schema rollout has overlapping machineries

- **Operation and consumers.** Schema installation, activation and evolution, used by operators and fixtures.
- **Current mechanism.**
  - `database/canonical-schema` is a SurrealKit frozen rollout. Its `producer-register.json` names
    `scripts/surreal_rollout_producer.py`, which does not exist. Its `schema/canonical.surql` (237 KB) differs
    by hash from the generated `surreal.surql` (423 KB) and was last touched earlier. `database/canonical-rollout`
    holds empty directory skeletons.
  - `.config/surrealkit.toml` pins a beta tool. A search of `crates`, `xtask`, `scripts`, `justfile` and `.config`
    finds no invocation, only a fixture, `canonical_maintenance.rs` and `canonical.rs`. The
    `surrealkit_rollout_pending` admission field and the `.surrealkit/<nonce>/settled.json` path remain in
    product code.
  - `canonical_migration.rs` (1,761 lines) carries two frozen predecessor declarations
    (`substrate-v10-pre-epoch.surql`, `substrate-v10-schema-v2.surql`) and fixture constructors, while ADR-0214
    says fresh installation regenerates data with no old-data import or backfill.
  - `canonical_maintenance.rs` performs its own AST definition comparison and activation.
- **Why it matters.** An operator or agent cannot tell which is the rollout route, and dead declarations look
  authoritative. Either an established schema-diff tool owns rollout, which would shrink local maintenance, or
  it does not, and its residue should go (*Proposed*).
- **Patterns.** LOP-11, LOP-09, LOP-X003.
- **Evidence.** Read the rollout toml head, `producer-register.json`, `.config/surrealkit.toml`, the heads and
  declaration constants of `canonical_migration.rs` and `canonical_maintenance.rs`, the
  `surrealkit_rollout_pending` sites and the §20.5 row in `workspace-and-dependencies.md`.
- **Footprint.** The absence search covered the listed paths only. `docs/dev/surreal-substrate.md` was not read
  closely for an operator route, and the SurrealKit fixture's consumer was not read. Dead premises elsewhere
  are collected in [CBO-065](developer-tooling.md#cbo-065).
- **Preservation constraints.** Exact definition witnesses, initializer nonce, fresh-incarnation fencing,
  module identity joined to activation, and the §20.5 schema contract. REUSE metadata lists `database/`.
- **Counterevidence and uncertainty.** The predecessors are exercised by `crates/pse-operations/tests/native_service.rs`
  (`native-service-predecessor` fixtures) and a retained-v2 upgrade test, so migration is tested behavior and
  "orphaned" applies only to `database/` and SurrealKit. "Fresh only" may be out of date rather than the
  migration, and SurrealKit may be an intended, not yet wired, tool.
- **Status.** Current mechanism.
- **Research questions.** Does an established rollout tool meet the fencing needs above, or is local AST
  comparison the right owner?

<a id="cbo-010"></a>

## CBO-010 — Guarded scripts: logic split between stored functions and inline text, reparsed per call

- **Operation and consumers.** Every guarded read, mutation and staging call that runs a framed script
  against the store, always within the original request clock.
- **Current mechanism.**
  - Retry is largely consolidated. `transport_retry.rs` owns `Attempt`, `RetryPolicy`, `retry_conflicts`,
    `retry_with_state` and `retry_pending`, and all backoff shifts sit in `pace`, with four pacing forms
    (`Guarded`, `Initialization`, `BeginStage`, `Yield`) in one enum. Capture-busy retries do not count toward the
    attempt cap (`Attempt::Busy`), which couples publishers to the gate in [CBO-071](#cbo-071). A fixed 10 ms
    poll remains in `canonical_selection::publish_description_inner`.
  - The envelope is consolidated: `canonical_script.rs` has `GuardedScript`, `Envelope` and `ResultSlot`/`ScriptResponse`.
    Non-test `BEGIN;` is now about 17 sites, mostly in that file.
  - The residue is that multi-statement business bodies remain inline Rust text. `canonical.rs` and
    `canonical_staging.rs` hold substantial `IF … THROW` logic (for example the product publication block),
    while comparable fences live in the stored `fn::pse_*` functions. Line-pattern counts of inline statements are
    about 158 and 85 respectively, heuristic and including tests. Every call is shipped as text and parsed by the
    server. The server's `Run` RPC, which calls a function by name, is unused here.
- **Why it matters.** One language and one set of invariants have two authoring homes. Moving a stable script into
  a versioned function would also place it under schema digest and activation, which inline text escapes. Parse
  cost on hot paths (append, heartbeat, claim) is *Proposed* and not *Measured*, and may be negligible next to the
  AOL fsync, leaving only the ownership benefit.
- **Patterns.** LOP-07, LOP-09, LOP-05.
- **Evidence.** Read `canonical_transport.rs::query_class`, `canonical_script.rs` to the envelope match,
  `canonical_migration.rs::TRANSACTION_ADMISSION`, the product publication script in `canonical.rs`, the protocol's
  method list and the server `query` and `run` heads; `transport_retry.rs`.
- **Footprint.** The envelope and a sample of inline bodies are inspected. Whether `Run` supports sealed bindings and
  transaction semantics is unread.
- **Preservation constraints.** Sealed envelope bindings; schema admission and server clock in the same transaction;
  business result evaluated before the postlude; live-authority recheck before commit; complete response before
  publication; ordinary single native functions keep their own explicit authority (§20.6).
- **Counterevidence and uncertainty.** Dynamic heads are deliberately ordinary SQL body text. `Run` may not admit a
  client-framed transaction, in which case a defined outer function carries the envelope. The envelope is careful
  and not a defect. A generic retry crate would add little, since classification must be step-supplied after
  settlement.
- **Status.** Current mechanism.
- **Research questions.** Does 3.3.4's executor cache parses or plans for repeated query text? What does `Run` do
  about transactions and timeouts? Which inline bodies should join the stored functions? Unchecked correspondence of
  signatures, binds and decoders across the language boundary is [CBO-046](registry-generation-and-python-boundary.md#cbo-046).

<a id="cbo-073"></a>

## CBO-073 — A study occurrence takes many round trips and reassembles known facts

- **Operation and consumers.** One study point through claim, execution and observation, and the recovery path:
  `work_study_candidate` in `workflow/worker.rs`, `record_study_attempt` and `recover_study_point` in `workflow/study.rs`,
  and `observe_study_point` in `canonical_studies.rs`.
- **Current mechanism.**
  - The observed sequence is `progress_studies`, `canonical_study_point`, `canonical_study`, an optional seed read,
    `prepared_claim`, `mark_study_started`, `renew_attempt`, appends, close and seal, a second `canonical_study`
    (only for `cancelled`) and `observe_study_point`: about ten round trips. The scope already carried `cancelled`
    and `generation` at discovery.
  - `record_study_attempt` and `recover_study_point` build `PointFacts` and `PointOutcome` with near-identical hand
    merges. Native code calls `study_policy::may_retry` and passes a bare `settled` boolean into
    `fn::pse_study_v1::observe`, while the same Rust policy runs inside the database as a WASM module.
  - The module takes `Row.policy`, `Row.facts` and `outcome` as JSON `Bytes`, decodes them in WASM and re-encodes its
    decisions. `Decision.disposition` is a free string, `uint` converts `Decimal` through
    `normalize().to_string().parse()`, and aggregate byte preflights are hand-written.
- **Why it matters.** The duplicated merge sits on the crash path, which is the path least exercised. Short occurrences
  are dominated by coordination. Opaque columns block native queries such as failed points by reason. Both are
  *Proposed* gains with nothing *Measured*.
- **Patterns.** LOP-09, LOP-04, LOP-05, LOP-06, LOP-03.
- **Evidence.** Read `worker.rs`, `study.rs`, `canonical_studies.rs`, `canonical_studies/progression.rs`, the head of
  `fn::pse_study_v1::observe` in the pre-epoch migration, and `pse-study-module/src/lib.rs` to its tests.
- **Footprint.** The remaining `pse_study_v1` functions, `study_tables::projected_outcome_row`, the SurrealQL callers
  of the module and the SurrealQL `recover` body are unread.
- **Preservation constraints.** Short guarded transactions with no numerical work inside; one policy owner; revision
  and attempt fences; exact settlement of lost acknowledgments; cancellation freshness at the decision; a truthful
  worker-loss receipt; a stateless, read-only module with one export; bounded input before decoding.
- **Counterevidence and uncertainty.**
  - The second `canonical_study` read may intentionally fetch fresh cancellation after a long run, and each read has
    its own response-ownership and memory contract.
  - Opaque bytes keep `pse-model` free of a Surreal dependency and avoid `Decimal` and object-size surprises.
  - Typed columns may complicate schema migration, and page volume is low (128 rows), so the module's benefit may be
    mostly clarity. Small points may be rare in the real workload.
- **Status.** Current mechanism.
- **Research questions.** Can an `observe` evaluated by the module decide settlement inside the transaction, and can
  `RETURN` blocks fold claim, start and renew into one guarded call? Is `Decimal` a type with a direct integer
  conversion, and can `SurrealValue` be implemented on shared types?

<a id="cbo-074"></a>

## CBO-074 — No composed application backup and restore

- **Operation and consumers.** The maintainer protecting retained scientific history against loss and restoring it to
  a fresh location, plus operator backup and selected-closure exchange (`canonical_files/offline.rs`,
  `pse-canonical-recovery`, `canonical_project_imports.rs`, §20.6 operational limits).
- **Current mechanism.**
  - Backup is stop-and-copy: offline freeze, drain and stop, then exact-closure copy. `docs/dev/surreal-substrate.md`
    states that the retired Python quiesce, backup and inspect-backup commands have no equivalent high-level command
    and that the physical helper does not recreate the retired restore supervisor.
  - The primitives exist: `NativeCanonicalService::shutdown`, `SchemaMaintenance`, the `canonical_files` offline
    primitives, the recovery helper and per-project `ProjectExport` and `ProjectImport` packs.
  - The composition does not exist: database export, configuration, credentials, module identity, application
    inventory and sealed payload closure, published last.
  - The server offers `ExportSurql`, `ImportSurql` and `ExportDirectory` RPCs, and none is called. The client exposes
    only query, setup and session methods.
  - The project archive is a bespoke container: an 8-byte manifest length, manifest bytes, then members copied by
    `portable_payload::copy_exact`. It is also the retained `project-import-evidence` content. `tar` and `zip` are pinned
    workspace dependencies, but the search found them used only by `pse-host` diagnostic tools.
- **Why it matters.** A long campaign cannot take a consistent store capture without stopping, and a store-wide
  recoverable copy needs hand composition from interfaces documented as carrying no adoption authority. Urgency follows
  the first non-regenerable data. Demand beyond the sole operator is unproven (*Proposed*).
- **Patterns.** LOP-12, LOP-09, LOP-03.
- **Evidence.** Read `surreal-substrate.md` ("Quiesce, stop and backup", "Restore and validation"), §20.6 operational
  limits, the head of `offline.rs` with a grep of its syscalls, `canonical_project_imports.rs` in full, the tonic stubs
  and server method list, and `project_portability.rs::archive` with its signature list. Export semantics were not read.
- **Footprint.** The body of `project_portability.rs` was skimmed, the `canonical_files` bodies are unread, and the
  `tar` and `zip` consumers were not traced.
- **Preservation constraints.** Closed admission; no live engine-directory copy; exact external-file closure with
  unresolved intents included; fresh owned destination and fresh authority on restore; completion published last. An
  engine snapshot alone is not application recovery.
- **Counterevidence and uncertainty.** A database export does not cover the external IPC file tier or frozen
  inventories, so it would be at most one component. Selected-closure exchange has domain semantics no generic export
  provides. Development stores are declared regenerable, and offline is an accepted contract. A generic backup tool
  cannot own quiesce or identity rebinding.
- **Status.** Current promise before non-regenerable data, with no implementation owner found. Plan 58h lists backup, restore and disaster recovery as a qualification obligation and 58d places related owners, but no package that builds the composition was found.
- **Research questions.** Could SurrealDB export and import through the admin connection, filesystem reflink or
  snapshots for immutable sealed payloads, or a deduplicating backup engine as transport for a quiesced tree serve as
  components? Would a standard container serve project packs (random access to members, inspection)?

<a id="cbo-068"></a>

## CBO-068 — Named guards and FOR UPDATE registrations may overlap what SSI tracks

- **Operation and consumers.** `fn::pse_execution_v1::{touch,register,source_creator,…}`, the `canonical_guards` table
  and the retention, creator and cancellation interplay (ADR-0172, ADR-0177).
- **Current mechanism.** Hand-assigned conflict domains keyed by strings (`'retention:'+problem`,
  `'source-creators:'+problem`, `'execution-run:'+run`). `touch` increments a decimal generation and `register` does
  `SELECT … FOR UPDATE`. The design predates MX SSI. §20.6 now says SSI "tracks point and range reads in writing
  transactions" while named guard writes remain required for lifecycle decisions and external-effect authority.
- **Why it matters.** Every mutation transaction increments shared rows, and those rows make otherwise independent
  work conflict, which in turn needs `ProblemTurns` local pacing and a 32-attempt retry budget. If some guards only
  stand in for a read set, removing them narrows contention (*Proposed*). The candidate is a reassessment of each
  guard's reason, not a removal.
- **Patterns.** LOP-03, LOP-11, LOP-08.
- **Evidence.** Read `touch`, `register`, `source_creator`, the `canonical_guards` definition,
  `canonical.rs::ProblemTurns` and `protected_query`, `transport_retry.rs`, and §20.6 "Authority and client".
- **Footprint.** A sample of guards; no per-guard census.
- **Preservation constraints.** Conflict dependencies named in §20.6 (problem creator and reclaimer pacing, serialized
  visible-run sequence); generations as externally visible fences; exact conflict retry classification and race tests.
- **Counterevidence and uncertainty.** The architecture explicitly retains named guards, and generations carry meaning
  outside any transaction (fence tokens held by writers), which SSI does not provide. Absence-premise protection is
  called out as needing `FOR UPDATE`. Many guards are probably real generation fences read by later transactions.
- **Status.** Current mechanism.
- **Research questions.** Does MX 1.0 range-read tracking cover the absent-key and range-predicate premises these
  functions use? Is there a test that distinguishes a fence from a read-set substitute?

<a id="cbo-067"></a>

## CBO-067 — Referential and lifecycle rules rebuilt on string keys beside native references

- **Operation and consumers.** Retention, release, roots, borrowers, artifact holders and reclamation
  (`canonical_retention`, `canonical_result_retention`, `canonical_lifecycle`, `surreal_retention.rs`,
  `ARTIFACT_HOLDERS`), and identity checks throughout.
- **Current mechanism.** The generated schema declares about 404 `TYPE string` field lines and no record-typed fields
  except `in` and `out` of 6 enforced relations (`record<` appears 12 times). `type::record('table',$key)` appears on
  about 666 lines. Referential presence is checked by procedural `SELECT … IF $x=NONE THROW`, and reverse lookups use
  explicit compound indexes (`root_revision`, `file_borrower_instance`). Tables declare `INLINE REFERENCES 32`, but no
  field-level `REFERENCE` clause was found. Upstream has reference strategies (`ON DELETE`).
- **Why it matters.** ADR-0214 already moved toward native record and graph access. Delete rejection and reverse
  references are the store-native form of "cannot release while referenced", which retention now expresses through hand
  scripts and scanned pages. Possible gains (*Proposed*): fewer procedural guards and indexes, a schema-level backstop
  for dangling references, cheaper reverse lookups.
- **Patterns.** LOP-03, LOP-10, LOP-12.
- **Evidence.** Greps over `generated/surreal.surql`; read `ARTIFACT_HOLDERS` and `INDEXES` in `substrate.rs`,
  `begin_run`, `canonical_codec::{record_link,check_record_identity}` and §20.6 "Record identities serve exact lookup".
- **Footprint.** The schema was inspected by pattern. The retention function bodies, the heads of the retention modules
  and upstream reference semantics under SSI and MX are suspected but unread.
- **Preservation constraints.** Immutable identities are opaque text with no coercion; lifecycle receipts outlive
  payload removal, including retirement receipts retained after collection; imported history is origin-qualified;
  bounded 64-item reclamation pages with replayable receipts; a reference must not silently pin or cascade.
- **Counterevidence and uncertainty.** Many "references" point to records deliberately deleted while a receipt survives,
  and may be cross-domain, so a constraint could be wrong for them. Paged reclamation conflicts with an unbounded reject
  scan or cascade. Reference maintenance under SSI, and its write amplification, is unknown. The string discipline was a
  reaction to coercion bugs. Applicability may be limited to immutable-parent relations.
- **Status.** Proposed.
- **Research questions.** What are the semantics and cost of `REFERENCE ON DELETE REJECT` under explicit SSI in MX 1.0:
  does the check register a conflict key, and is it bounded?

<a id="cbo-011"></a>

## CBO-011 — Single-flight, I/O gates and load admission beside caching libraries

- **Operation and consumers.** Deduplicate concurrent loads and bound native I/O and load admission. By search the
  consumers are `pse-runtime` math and physical caches (`math/artifacts.rs`, `math.rs`, `math/preparation.rs`,
  `workflow/physical_cache.rs`, `workflow/modeling/canonical.rs`) and `pse-math/src/execution.rs`. The former
  `pse-engine` flight wrapper no longer exists.
- **Current mechanism.**
  - `pse-columnar/src/flight.rs` (454 lines) implements completion-owned single flights with a shared future and an
    atomic waiter count, cancelling on the last waiter. Its header comment says it owns its entry until actual exit.
  - `pse-engine/src/stores/admission.rs` wraps the object store as `SharedIo` with a task-local `ACTIVE_IO` lease chain.
  - `cache_service/load.rs` has its own `LoadCounters`, semaphore and reservation guard.
  - `quick_cache` is declared only by `pse-runtime`, and no `LimitStore` use appears.
- **Why it matters.** Several bounded-concurrency and deduplication mechanisms are coordinated by convention. The
  remaining opportunity is probably narrow, most likely the I/O gate (*Proposed*).
- **Patterns.** LOP-08, LOP-03, LOP-01, LOP-X005.
- **Evidence.** Heads of the four modules and a search for consumers. The cache-service flight file was found deleted.
- **Footprint.** Callers were not traced and the narrow-gate conclusion was not verified.
- **Preservation constraints.** A native load keeps its leases until it actually exits, even after every waiter leaves;
  cancellation on the last waiter; typed shared errors; the DataFusion pool as the single budget.
- **Counterevidence and uncertainty.** Exit-joined semantics differ from `moka` `try_get_with` and `quick_cache` placeholder
  guards, so "beside caching libraries" is not a replacement claim. `LimitStore` may not compose with the nested-scope
  re-entrancy `SharedIo` implements.
- **Status.** Current mechanism.
- **Research questions.** Can a library single flight retain its producer through waiter cancellation? Does
  `object_store::limit::LimitStore` compose with `SharedIo`'s nested scopes? Eviction policy across math lifetimes is
  [CBO-022](mathematics-and-compilation.md#cbo-022).

<a id="cbo-012"></a>

## CBO-012 — Imperative nested-occurrence traversal beside the plan route (weak)

- **Operation and consumers.** Relation admission: find the rows and nested occurrences that violate declared
  obligations, with exact occurrence paths for diagnostics.
- **Current mechanism.**
  - `pse-relations/src/validate/occurrences.rs` (612 lines; `CHUNK = 4096`, `Node::prepare`, `visit`, `take`, `ranges`)
    walks nested paths and is called from `validate/prepared.rs`.
  - `validate/obligations/nested_values.rs` expresses nested-value checks as DataFusion plans (`Occurrence`, `range`,
    `UnnestOptions`).
  - `validate/predicates.rs` carries an adapter for Delta's pinned SQL parser, used for CHECK text.
- **Why it matters.** Two routes may answer a similar "which rows violate" question, and the adapter ties one of them to
  a parser version.
- **Patterns.** LOP-09, LOP-05, LOP-11.
- **Evidence.** Heads and signatures of the three files. The module comments differ: `occurrences.rs` says kernels see
  visible values before evaluation, while `nested_values.rs` retains source columns, multiplicity and actual nested
  indices. That suggests different stages and at most partial overlap, a reading of comments and not a trace.
- **Footprint.** Heads and signatures of the three files only; bodies of the 5.4k-line `validate/` tree are mostly unread.
- **Preservation constraints.** Exact diagnostics per occurrence path; memory bounded by the pool; visible-value semantics.
- **Counterevidence and uncertainty.** Whether the routes overlap was not traced. The imperative path may supply location
  detail the plan route cannot. The plan route already builds occurrence ordinals by unnesting a `range` beside the list,
  and DataFusion has no native ordinality (library assessment, 2026-10-27; see [datafusion](../libraries/datafusion/README.md)).
- **Status.** Current mechanism. Weak.
- **Research questions.** Does the plan route yield occurrence coordinates for every case the imperative walk serves?

## Examined and not retained

- **Generated codecs.** The registry-generated Surreal codec and the Arrow row codecs are imperative per-field code, and
  one declaration yields several forms. This is the intended single-authority generation (prime directive 2). The cost
  question is recorded under [LOP-X003](../assessment-patterns.md#lop-x003) and
  [CBO-040](registry-generation-and-python-boundary.md#cbo-040).
- **Drain waits.** Poll-and-sleep waits for owner drain and file locks are recorded under
  [CBO-016](execution-lifecycle-and-observation.md#cbo-016) and [CBO-003](cross-cutting-mechanisms.md#cbo-003).
- **Replacing `OwnedHttp2` with tonic `Channel` or tower layers.** The obligation is physical drain and completion
  ownership (LOP-X005).
- **A generic retry library for `transport_retry`.** Classification is step-supplied after settlement (LOP-X012) and the
  module is 324 lines.
- **Client-driven RPC transactions (`BeginTransaction`).** They add round trips and server state, and only the test-only
  `HeldTransaction` uses them.
- **Scientific IPC files in `DEFINE BUCKET`.** That would route bytes through the server and away from worker-owned sealed
  identity and the offline closure.
- **`DEFINE SEQUENCE` for generations.** Gaps or non-transactional allocation would conflict with commit-order publication.
  Reasoned, not verified upstream.
- **Replacing `FlushFence` and `OutboundBody`.** They track when bytes were flushed by the socket, a guarantee hyper does not
  surface.

## Retired opportunities

<a id="cbo-009"></a>

- **CBO-009 — Large local SurrealDB SDK delta with a mirrored lifetime layer.** Retired 2026-10-30: `bee1a2271` removed
  `vendor/deltas/surrealdb*.patch` and `vendor/surrealdb`, `Cargo.lock` has no `surrealdb` package, and the owned
  `canonical_client` over `surrealdb-protocol` replaced the SDK path. The residual burden is split into
  [CBO-066](#cbo-066) (client wire), [CBO-070](#cbo-070) (server patches) and
  [CBO-079](execution-lifecycle-and-observation.md#cbo-079) (custody carriers). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/persistence-and-transport.md#cbo-009).
