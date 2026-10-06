---
title: Canonical substrate and immutable revisions
status: draft
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28a: Canonical substrate and immutable revisions

## Responsibility and foundation assessment

This companion of [Plan 28](28-surrealdb-unified-substrate.md) owns canonical storage,
deployment, generated schema/codecs, immutable revision selection, conflict guards and
retention/read protection. It supplies the storage contract consumed by B/C/D. Shared
decisions and finding dispositions belong to the coordinator; this document owns A progress
and local evidence.

Repurpose `pse-operations` as the concrete canonical substrate boundary. It owns the thin
remote client, typed database operations and transaction/retry mechanics; runtime composition
owns application/server configuration and scientific orchestration. Keep database-client and
embedded-engine dependencies out of `pse-schema`, `pse-modeling`, `pse-compiler` and native
mathematics. Existing semantic types are consumed through typed interfaces. Do not add a
generic storage abstraction for a second production backend. Remove `pse-operations-queries`
and, once its readers have moved, `pse-catalog`; the coordinator's R0 covers crate retirement.

Current PostgreSQL operations and Delta catalog machinery have useful lifecycle obligations,
but their physical representation and recovery protocols are replaced. The registry in
`pse-schema/src/catalog`, scientific document declarations and `pse-codegen` remain the
single semantic declaration route. The current `BundleOwner` ancestry and Arrow-based full
source export are not the canonical revision representation. There is no importer or legacy
reader deliverable: the maintainer selected a clean rebuild, with controlled inputs and
scientific artifacts regenerated against the target.

## Server profile and acknowledgment contract

Select a supervised local SurrealDB server with **RocksDB**, authenticated loopback gRPC,
and a thin Rust SDK using explicit `protocol-grpc`. The 3.3 API/source profile is the reviewed
starting point; committed resolution records the implemented SDK version. Dependencies float
normally. A held server/SDK version needs the repository's specific pin rationale rather
than an indefinite pin merely because the investigation used 3.3.0. Qualify the actually
selected pair and record it in the eventual Outcome.

Use `rocksdb://<application-state>/database?sync=every&versioned=false`. Domain revisions
provide history, so database storage-version history is unnecessary. Current official docs
recommend RocksDB for persistent server deployment and label SurrealKV beta; that favors a
conservative first profile without a performance claim. `sync=every` documents synchronizing
before transaction confirmation. See [deployment guidance](https://surrealdb.com/docs/manage/self-hosted/deployment-models)
and [server configuration](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/start).

The supervisor owns the state directory, configuration, process lifecycle and reconnect
behavior. Use existing recipe/xtask surfaces for setup and test fixtures. Do not compile the
embedded database core into all application crates. A configured shared remote endpoint can
use the same typed operation boundary; local supervision is the required initial supported
profile. Network deployment requires its own TLS/authorization configuration and qualification,
not an assumption that the loopback profile proves it.

Keep explicit initial 4 MiB server/client gRPC message limits and size write batches/result
blocks below the encoded limit, including metadata. A sequence of rows can stream while an
individual block still exceeds the limit. SDK `stream_items` forwards true server-produced
rows on the gRPC route; statement-end failure makes a preceding prefix provisional. D2 owns
the Arrow/query consequence. See [Rust connection documentation](https://surrealdb.com/docs/reference/rust/methods/connect)
and the [reviewed source limits](../design_review/evidence/surrealdb-unified-substrate-2026-10-05/integrated-capabilities.md).

The local application supervisor owns a finite managed worker group and partitions its
configured resource envelope between the server and native worker processes; C3 consumes
that allocation through existing process-local admission. A remote shared deployment does
not inherit this local capacity qualification.

Bound database and native resources together. Set the RocksDB cache from the allocated
database-process memory budget rather than its host-RAM default. A database allocation
threshold is not a total-RSS cap; process supervision supplies the cap. Bounded client queues,
query/result batches, native sessions and temporary disk have separate lifetimes. Temporary
sort files do not establish universal aggregation spill. Configuration must make failures
observable and permit cancellation/drain and retry without retaining complete source graphs.

Acknowledged durable operations must survive the supported abrupt-process-restart scenario.
An acknowledgment lost after commit is resolved through its operation identity; it is not
permission to blindly repeat a mutation. A1 establishes this with targeted server checks
before consumers rely on durability. Physical power-loss survival also depends on filesystem
and device flush guarantees and cannot be established by a process-kill test alone. The
current evidence includes clean reopen, not a completed crash campaign.

## Schema and scientific values

Keep one authoritative declaration per meaning. Extend the existing declaration route with
canonical records, relation identities, field domains and index/constraint lowerings. Generate
Surreal schema and codecs, required Rust/Python types and Arrow boundaries from that route.
Scientific declaration shapes must not be reauthored as hand-maintained SurrealQL DDL. Native
query functions may own structural operations, with versioned behavior; shared scientific
kernel predicates retain their existing semantic owner. A database assertion is not a second
implementation of evolving physical admissibility.

Canonical families include authored object versions/revisions/membership, semantic dependencies
and portable products, run/attempt/study state, scientific result sets/observations and method
lineage. Use identity-bearing relations when their occurrence/properties have meaning; not
every scalar cell needs its own graph node. Result physical layout belongs to D1 and terminal
admission to C1. Their declarations participate in this same schema/codegen route.

Define an exact scientific cell codec: authoritative f64 bits in bytes with a mechanically
derived finite numeric projection; checked integer/identifier domains; scientific absence,
state and quality distinct from database `NONE`/`NULL`. Preserve the registry's finite
`Float64` domain (blueprint §4.5): nonfinite values are refused in those fields. Explicit
tagged raw diagnostic domains can retain nonfinite bit payloads. Signed zero remains exact.
Reading a convenient numeric projection cannot replace authoritative bits or bypass admission.

Represent full-range u64 fields as native decimal integers, bound through the SDK's decimal
value path rather than ordinary u64/JSON conversion. Declare decimal type, integral value
and range 0 through 18446744073709551615; writers use canonical scale zero. Decoders reject
fractional, negative and out-of-range values before checked conversion: conversion alone can
truncate fractions. Decimal comparisons and selector bounds stay decimal, never f64. Use
UUID/string record IDs and an indexed unique decimal field when an unsigned domain identity
is required; gRPC does not support a decimal record-ID key. This is source-supported, with
endpoint round-trip/index qualification required in A2/D1. See [native number types](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/numbers)
and [decimal conversion](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html).

Native record IDs and strings remain different types. Use the registry's units, basis, state
and extension metadata at Arrow/Python boundaries. Untested generic JSON/HTTP/WebSocket
conversion is not the codec.

Database schema, generated codec and function interpretations have explicit versions. Direct
privileged imports can bypass assertions/events/views, so normal application writes use
controlled operations. Controlled fixture regeneration likewise passes admission; no bulk
import is announced ready merely because records exist. Backups restore an already qualified
database with its recorded interpretation. Unknown schema/reader compatibility fails clearly.

## Revision and conflict contract

Each problem has a linear revision sequence and a compare-and-set head. Authored objects
have stable logical identities and immutable versions. Membership records associate a problem,
logical object/name and version with a validity interval. The current-head inventory has
indexed active membership; historical selected identities resolve directly. A full historical
inventory may require an interval selection: make that operation explicit rather than copying
each full graph or inventing a custom snapshot trie.

One short authoring transaction fences the expected head, validates the supplied mutation's
structural premises, writes immutable new versions, closes/opens changed membership intervals,
updates relevant name/scope guards and advances the head. Value-only edits replace the actual
value version; unrelated objects are not rewritten. Structural scientific admission happens
through B1's shared kernel before an operation is eligible for execution. A coherent draft
revision can retain diagnostic failures without being mislabeled an admitted runnable model.

Use named records for head, name and membership-scope premises. Exact record `FOR UPDATE`
registers a conflict even for a missing named record in the inspected implementation. Every
mutator that changes a premise updates its guard in the same transaction. Native predicates
over a set are not predicate locks; a scan of an absent name without its named guard does
not protect a decision. On conflict, rerun resolution and the entire decision on fresh state.
Generation/operation identities make retries deterministic and acknowledgment recovery explicit.

Immutable selections carry problem/revision/version identities and interpretation. B consumes
them without complete ancestor replay. Namespace and name resolution retain their actual
scope; deletion and newly introduced names invalidate dependencies through the guards B2
records. Built-in materialized table views are not an incremental scientific compiler and
must not substitute for those dependencies.

## Retention and protected reads

Persistent history is intentional. Default normal runs retain their selected revision,
configuration, observations and descriptions. History deletion requires explicit retention
policy; there is no inherited in-memory eviction meaning. Root reachability includes current
heads, retained revisions/runs/analyses/compilation products, active attempts and protected
queries, exports and scientific preparations. Garbage collection
reclaims unreachable versions and fenced abandoned staging in bounded batches, without
replaying every problem or racing a newly admitted root.

A multi-page query/export or compilation preparation acquires a protected immutable selection
before deletion can start and releases it on completion/cancellation. Compilation registers
its durable product and consumed source/interpretation roots atomically under the same
retention guard before releasing protection. Expiry fences further reads and product admission
before reclamation.
The acquisition, root admission and deletion decision share named retention guards. A native
database transaction alone cannot protect a selection after its transaction has ended. This
replaces Delta-specific leases/windows while preserving the real long-read protection need.
Pin mechanics stay local to this boundary and D's reader adapter; do not reproduce the old
cross-store manifest protocol.

Support backup/restore and reopen of the target database. Restored state includes schema,
interpretation and operation identities, with verification before accepting writes. E1's
clean regeneration is the adoption route; it does not remove the target's ongoing recovery
obligations.

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| A1 — Supervised remote substrate | R0 decision route. Repurpose `pse-operations`, add recipe-owned RocksDB/gRPC profile, typed operations, restart/ack recovery and bounded supervision. | Move runtime store construction/test fixtures to the target. Remove displaced PG connection/pool setup as the affected operations move; retain no fallback constructor. | Scheduled |
| A2 — Declarations, codecs and immutable selection | A1 and R0 schema/identity decision. Generate schema/codecs, immutable identity reads, named guards and minimal preparation protection/root admission. | Supply working selection/codec/guard/protection to B1/C1/D1. Retire matching PostgreSQL lowerings through the generator as consumers move, then regenerate. | Scheduled |
| A3 — Authoring, retention and restore | A2 plus B1 admission slice for runnable revisions. Implement head-CAS edits, membership/name changes, guarded retained roots, paged read protection and backup/restore. | Migrate source authoring and lifecycle maintenance. Delete bundle ancestry ownership and displaced Delta/PG retention mechanisms as their consumers move. | Scheduled |

A2's early slice includes real exact codecs, immutable identity reads, conflict guards and
preparation protection/root admission;
it is sufficient for selected B/C contracts without waiting for all A3 authoring operations.
A3's full retention surface integrates with C4/D2. Root coordinates schema and generated
outputs; no worker hand-edits generated paths or independently changes the shared declarations.

## Verification

**Proposed acceptance:** targeted units and touched-package compile checks through repository
recipes, with force-validation for scientific controls. A1 adds recipe-owned targeted server
tests of the new mechanism; they are not a repeated assembled campaign. [28e](28e-rebuild-retirement-and-qualification.md)
owns final integrated testing and qualification.

Verify independent exact value fixtures at signed-zero/integer boundaries, finite-field
nonfinite refusals, declared raw diagnostic payloads, domain
missing states and typed IDs through the selected protocol. Test absent-name and membership
write skew, two edits from the same head, idempotent edit recovery and whole-decision retries.
S01/S02 should retain unchanged identities and reject stale or incompatible selection.
Decimal u64 round-trip/equality/range/order controls include 0, 2^63-1, 2^63, u64::MAX and
adjacent high values; reject fractional/negative/above-max values and inspect indexed selectors.

Kill/reopen the server around acknowledged mutation/staging/claim/terminal operations, with
fixed operation identities. Already acknowledged data must survive; uncertain operations
must settle consistently. Do not call clean reopen crash qualification. Test reader protection
against concurrent retention, interrupted export and bounded orphan reclamation. Restore a
target backup and compare its selected exact values and operation identities before writes.

## Checkpoint and next step

No A package is implemented by plan authoring. R0 precedes A1 and the A2 shared declaration
slice. The backend/transport choice is settled; exact configuration and crash behavior remain
implementation qualification, not a reopening of the overall storage decision.

## Outcome (recorded after implementation)

### What was built

Pending implementation and named evidence.

### A mistake made and corrected

Pending execution.

### Deviations from the plan, deliberate

Pending execution; consequential decision changes follow the decision route.
