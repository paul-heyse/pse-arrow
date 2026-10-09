---
title: Canonical substrate and immutable revisions
status: in-progress
date: 2026-10-05
adrs: [ADR-0164]
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md, docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md]
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

The initial implemented profile uses a supervised local SurrealDB server with **RocksDB**,
authenticated loopback gRPC and a thin Rust SDK using explicit `protocol-grpc`. The 3.3 API/source profile is the reviewed
starting point; exact declarations and committed lockfiles record the implemented pair under
the [dependency policy](../dev/dependency-policy.md). Holds retain their specific revisit
rationale. Qualify the actually selected pair and record it in the eventual Outcome; this
parallel extension does not require a dependency or server upgrade.

The prospective replacement is owned by [30a](30a-native-websocket-rpc-and-operation-lifetimes.md)
and [30b](30b-persistent-services-and-receiver-generations.md), following the
[confirmed Plan 30 rules](30-websocket-and-persistent-agent-environment.md#confirmed-rule-changes).
Native WS must preserve exact codecs, complete statements, original operation clocks,
guarded publication and uncertain-acknowledgment settlement. Its bounded buffering and
completed-page contract replace the gRPC-specific target after working controls; the
gRPC observations below retain their original scope. A's canonical semantic owners
are not duplicated in the transport adapter.

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
and device flush guarantees and cannot be established by a process-kill test alone. The named native recovery control covers acknowledged process-kill/reopen and offline backup/restore; its conditions and exclusions are recorded in the Outcome.

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

## Completion-audit correction slices

The [coordinator](28-surrealdb-unified-substrate.md#finding-dispositions) owns F01/F03
status. The [capability investigation](../design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md)
supplies source-backed transaction and response semantics, not deployed-profile qualification.
The following work extends A1–A3 within their existing contracts.

| Slice and current source | Input, result and consumers | Required targeted completion |
|---|---|---|
| A3 lifecycle authority: `pse-operations/src/canonical_retention.rs` now restricts generic root mutation to History | Existing exact root/revision and owning run/attempt/analysis operation. Lifecycle minting/release remains atomic in its owner; generic APIs cannot redirect, forge or drop those roots. History may legitimately move. Run, staging, analysis and reclaim callers share this rule. | Execute `lifecycle_roots_cannot_be_redirected_or_released_by_generic_retention` through the native recipe. Verify direct native calls as well as Rust refusal, legitimate History movement, active attempt protection and eventual reclaim. Check all root-mutating callers; delete any remaining generic lifecycle bypass once covered. |
| A1 initialization: `pse-operations/src/canonical.rs` now consumes complete response then opens/verifies interpretation | Generated schema plus exact interpretation identity yield a verified open store. Statement errors, missing expected results or uncertain delivery do not report installation success; readback verifies the same identity without replaying DDL blindly. Ordinary create/open remains supported. | Execute new zero-statement, marker-verification, lost-ack and cancellation controls with the actual adapter. An SDK error and a valid End frame with zero statements are distinct cases. Do not attribute historical missing-table failures to this seam without decisive evidence. |
| A2 acquisition boundary, joined with B1 | Exact ID groups and `(scope, name)` pairs yield only demanded rows and complete selected extents. Preflight metadata for whole hydration groups, then bound payloads/decoded bytes and retain read protection through product-root admission. | Exercise sparse/cross-pair demands, absent IDs, conflicting namespace additions and grouped extent admission. Remove displaced singleton probes and duplicate decoding, preserving exact scientific codecs and physical admission. |

SDK 3.3.0 gRPC `query_inner` requires a terminal End frame and verifies its result count.
A truncated response is therefore a transport error, not an assumed empty success. Application
initialization still needs its own expected-statement and installed-interpretation checks.
Awaited `IndexedResults` accumulates received batches, so a per-frame cap alone does not bound
an entire grouped response; the owning acquisition must preflight total selected extents.

Named `FOR UPDATE` guards register commit-time conflicts; they do not block competitors or
lock a range predicate. Exercise an absent exact ID and a concurrent namespace/set addition
through every relevant mutation route. Retry the complete guarded decision on definite
conflict. Preserve operation identity across uncertain acknowledgments and settle readback
before deciding whether a retry is safe. READONLY fields or synchronous event refusal may
reinforce an invariant only through the generated schema owner; neither replaces lifecycle
ownership, and a blanket immutable history association would prevent valid history movement.

A's historical local profile/receipts below remain historical. Its current next step is these
targeted controls and the shared C/D protection handoff; assembled kill/reopen/restore and
combined read/retirement races belong to E3. Reopening an obsolete A/B integrated campaign is
not a prerequisite. No new server experiment was run for this documentation revision.

## Efficiency extension at the physical boundary

The [repository-wide extension](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
adds [28g](28g-bulk-data-operations.md) without changing this owner's revision, exact codec,
guard or protection authority. T0/T1 declare physical payload/index/resource bounds once and
derive server/replay ceilings through the registry/generator. T3 reuses grouped selected
acquisition for physical documents and composed result reads; T4 settles complete source-package
staging and activation before changing per-chunk publication. The current 512 KiB block and
selected transport profile remain initial ceilings, not a reason to frame six/eight-row blocks.

Preserve immutable request identity, complete positive/negative inventories, exact source
revision intervals and renewed read protection. Larger groups do not grant a new retention
authority or permit a transaction to contain native work. Any durable interpretation change
receives version-first admission and controlled regeneration through this owner. Actual access
paths remain T0 decisions; an optional predicate or a returned-page cap alone does not prove
examined-work efficiency. A's earlier Outcome retains its original evidence conditions.

## Parallel canonical protection and contention

The [parallel extension](28-surrealdb-unified-substrate.md#parallel-execution-integration)
adds A4 for Parallel F01/S01/S03/S06. RC01 was accepted conditionally on 2026-10-08;
its [decision route](28-surrealdb-unified-substrate.md#parallel-execution-rule-decisions-2026-10-08)
precedes any selected guard-contract change. The source assessment at `84a1caf17656f00f38b22e703c7cbc2b63a44d2d` establishes
a broad per-problem retention conflict domain, not the SQL attribution of each failed case.

`CanonicalStore::protect` creates a fresh protection and updates `retention:$problem`.
Protected queries validate the exact live pin/revision and register that same guard; release,
staging/product admission, root retirement and reclamation can update it. Equivalent publication
already has immutable acknowledgment/stage convergence. A4 must not assume that another client,
a wider retry cap or a second coalescing layer is the missing mechanism.

First classify the returning operation/retry owner in the failed complete-case control where
needed to choose a correction. Inspect consumed predicates and writers for the implicated
protect/read/release, stage, activation or product operation. Keep scientific work and bulk
staging outside the short atomic decision. Stop investigation with a selected safe correction
and its targeted controls; if attribution finds another cause, repair that actual owner and
retain the topology with an explicit reason. A continuing complete-route failure leaves F01
open irrespective of that local decision.

Compare existing-contract sharing/coalescing before introducing finer guards. Sharing requires
an exact revision/interpretation, adequate lifetime and independent complete dependency receipts;
B6 owns its runtime composition. If a finer topology is selected, every matching writer must
update the named guard in the same transaction. Preserve pin acquisition versus reclaim,
release/expiry, root/product admission versus reclaim, current-head and reclaimed-range checks,
absent-name/set premises, and cross-problem shared immutable-version protection. Ordinary snapshot
reads cannot replace the predicate fence merely because the record exists. Use registry/codegen
for changed declarations; do not hand-edit generated queries or add a second store path.

Retain the distinct recovery units. Protected decisions retry their whole transaction on definite
typed conflict. Product/source admission settles the exact immutable operation acknowledgment
before replay. Staging's connection/internal-error replay is justified by its own immutable
request and fresh generation fences, not a universal rollback assumption. Preserve statement-level
error inspection and complete gRPC statement termination.

| Package | Prerequisite and delivered behavior | Migration, deletion and targeted acceptance | Status |
|---|---|---|---|
| A4 — Parallel protection/publication decision | Confirmed RC01; existing A1–A3 guards and actual failed route. Attribute decisive operation/retry scope, select the smallest safe correction, route changed contract, implement the canonical slice consumed by B6/T7. | Migrate all writers/readers of changed guards together; delete replaced guard/query paths after callers and controls move. Exercise sixteen complete cases with original checks/order; race acquire/read/release/expiry, admission/root retirement and reclaim, absent/set edits, shared versions and exact lost-ack settlement. Reader-only stress or increased retries is insufficient. | Scheduled; attribution and selected mechanism remain unresolved. |

## Preparation description admission

The [preparation integration](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
adds A5 after confirmed RC03. [28j](28j-pure-preparation-and-publication.md) owns pure/effect
and reusable-description contracts; A owns their actual canonical admission, root, staging
and recovery realization. Keep short guarded decisions and exact acknowledged settlement.

An existing rooted exact product may be acknowledged without creating a new proposed row or
root for each revision. Current dependencies and consuming attribution must remain correct;
do not cache a publication permission. Expiry before new admission refuses, while a committed
exact acknowledgment can settle after the preparation pin expires. Released roots, changed
dependencies, corruption and another store remain consequential. J0/A5 must preserve this
distinction when moving descriptor/encoding work ahead of admission.

| Package | Prerequisite and target behavior | Completion and status |
|---|---|---|
| A5 — Reusable description admission | J0's complete description/dependency contract and accepted RC03. Select descriptor-first/grouped effect support only where current identity, atomicity and recovery allow; route changed metadata/commit contracts before implementation. | Implemented; Tested exact acknowledgment after revision change/pin expiry and protected concurrent reader/staging controls, with current guards/fences. J1 consumes owned descriptions; displaced repeated encoding is removed. 28e owns the affected scientific continuation. |

This package changes publication mechanics, not scientific authority or the database target.
No hand-edited schema or new generic store API is implied. If declaration/lowering changes
are needed, their owning generator and regeneration are part of that implementation.

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| A1 — Supervised remote substrate | R0 decision route. Repurpose `pse-operations`, add recipe-owned RocksDB/gRPC profile, typed operations, restart/ack recovery and bounded supervision. | Move runtime store construction/test fixtures to the target. Remove displaced PG connection/pool setup as the affected operations move; retain no fallback constructor. | Implemented; final verification in progress |
| A2 — Declarations, codecs and immutable selection | A1 and R0 schema/identity decision. Generate schema/codecs, immutable identity reads, named guards and minimal preparation protection/root admission. | Supply working selection/codec/guard/protection to B1/C1/D1. Retire matching PostgreSQL lowerings through the generator as consumers move, then regenerate. | Implemented; final verification in progress |
| A3 — Authoring, retention and restore | A2 plus B1 admission slice for runnable revisions. Implement head-CAS edits, membership/name changes, guarded retained roots, paged read protection and backup/restore. | Migrate source authoring and lifecycle maintenance. Delete bundle ancestry ownership and displaced Delta/PG retention mechanisms as their consumers move. | Implemented; final verification in progress |

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

A5 is implemented: owned descriptions preserve the exact encoding and original provenance,
and actual rooted acknowledgment is queried before new key, staging or receiving work.
Released roots require new admission; committed exact acknowledgment can recover after pin
expiry. Targeted qualification is in progress. Earlier A1–A4 controls keep their original
scope; the coordinator owns PA03 and 28e owns assembled acceptance.

A4's operation attribution located the sixteen-worker failure at `stage_small_edits`.
The integrated correction paces bounded same-owner staging RPCs per problem; it preserves
independent-client transaction guards, closed-stage replay, shared-version reclamation and
stale-generation fencing. Its concurrent staging controls and the two existing sixteen-worker
scientific controls pass. This supports the selected local correction, without claiming a
distributed staging mutex or changing source/product protection. B6/T7 caller reconciliation
and connected read/reclamation acceptance remain before E3. The selected store is preserved;
the reference campaign remains stopped.

A1–A3 functional mechanisms are implemented: supervised authenticated RocksDB/gRPC,
generated exact codecs/schema identity, closed payload manifests and leased staging,
head-CAS activation, selected read protection and retained roots, bounded reclamation,
and gated offline recovery. Protection and idempotent lease transactions retry complete
fresh guarded operations after native transaction conflicts. Product roots retain their
immutable origin and cannot be moved through generic retention. Already rooted exact
products are shared; fresh admission after explicit release uses a separate publication
identity without resurrecting the old root.

Shared unchanged document payloads carry local allocation leases; the complete parent-owner
chain is deleted. Managed native slots are independently fenced by their systemd unit and
kernel cgroup, including launcher loss. Runtime construction requires canonical deployment
configuration. Worker/study preparation passes immutable modeling revision identities;
remaining PostgreSQL execution/result lifecycle and Delta publication consumers belong to
C/D/E as their operations move.

Final generation and the focused native storage/recovery controls are complete. On
2026-10-06 the maintainer requested closeout and a committed checkpoint, then cessation
of work. A1–A3 remain implemented with final integrated acceptance pending. No measured
speed improvement or full Plan 28 series exit is claimed.

Continue with the correction slices above, then 28e's assembled target campaign.
Earlier local A/B state and receipts remain preserved historical evidence; there is no
requirement to resume their interrupted campaign or obsolete C/D/E migration boundary.

## Outcome (recorded after implementation)

### What was built

**Implemented:** A1–A3 mechanisms described above. The actual initial pair is released
SurrealDB server 3.3.0 and Rust SDK 3.3.0 (`protocol-grpc`, no embedded database core),
under the pinned repository toolchain. Server state records its actual binary digest;
future fresh setup follows the stable release resolver. Default supervision partitions
4 GiB into a 2 GiB server and two 1 GiB native slots; process caps supplement client
allocation and batching.

**Tested**, baseline zero: `just surreal-test` passed 19 supervisor controls;
`just surreal-fixture-test` exercised authenticated acknowledged write, forced restart,
offline backup, restore admission gating, finite slot capacity, launcher loss and actual
kernel memory/swap caps. `just canonical-recovery-test` passed generated-schema native
recovery of acknowledged revisions, retained historical selections, exact signed-zero
bits, a payload over 4 MiB transported in blocks, and a retained product. The typed
validator allowed protected reads and refused authoring before restore validation.
`NEXTEST_TEST_THREADS=4 just canonical-test <supervised-state>` passed all 22 exact
codec, selection, activation, retention and reclamation controls against the managed
3.3.0 server. These include concurrent protected reads and lease changes, fresh product
admission after explicit release, refusal of product-root reassignment, and settlement
of an exact deduplicated product after preparation protection ends. Settlement checks
the immutable root association and all recorded product meaning; it never resurrects
a released root. Atomic activation also covers inventories of 1,050 and 3,500 objects,
duplicate incoming names across physical pages, legal name swaps, and exact historical
selection. A cancelled query with no statement completion cannot establish success or
absence. Admission
separately bounds object count at 8,192 and the local closed metadata inventory at
32 MiB. Only its digest is submitted for activation. Wire submissions remain bounded
below the 4 MiB protocol limit; payloads use 512 KiB immutable blocks. The product
controls persist a 34 MiB body and dependency metadata over 4 MiB, share content
across retained roots, and reclaim the last released body's blocks in bounded pages.
An enlarged dependency scope is refused without hydrating the whole new collection
or contaminating the protected original selection. Four small
objects share one guarded staging transaction, with at most sixteen references and
64 KiB of payload each. Larger objects use the same declarations and guards with
separately submitted blocks and reference pages.

Activation selects changed logical memberships and namespace names through native
compound indexes, closes all displaced intervals before opening replacements, and
inserts native graph relations in bounded pages. Exact guard reads separate updates
from fresh inserts, avoiding released 3.3's create-first UPSERT duplicate recovery.
The entire revision still commits atomically under its head, stage and retention fences.

Compilation products use these same immutable blocks. Their small native headers name a
content-addressed body containing payload and dependency meaning. Closed staging admits
the product and revision root without advancing the authored head or creating source
memberships. The retained stage association protects shared body blocks and remains a
bounded reclamation frontier after the last product root is released. Discovery returns
an opaque extent receipt; the scientific consumer reserves memory before protected block
hydration and dependency qualification.

Complete-source authoring pages protected membership metadata and changes only altered
bindings. The targeted native `canonical_edits_preserve_unchanged_memberships` control
passed: a true no-op keeps its revision, an unrelated edit preserves the exact membership
interval, and deletion keeps a protected historical selection readable.

Final assembled acceptance is not complete. The earlier 106-control Rust selection
passed before the final blocked-product and native-owner integration. The reference-only
Python run passed `test_public_native_process_and_exact_results` on the earlier candidate;
its unfinished 1,000-point study was interrupted for the final extension refresh.
Neither is claimed as a completed final campaign. The final editable native extension
refresh passed through `just py-sync-native`; `just ready` and the repaired import
contracts passed. **Tested**, 2026-10-06, baseline zero: the final full `just hygiene`
bundle passed after the native-owner changes, including both Clippy modes, type checks,
generated drift checks and warning-free Rust documentation. One earlier import check
overlapped the extension's temporary uninstall; the recipe passed after reinstall, then
the actual full hygiene bundle was rerun successfully. Other already-running closeout
checks are recorded below when they finish.
The default 2 GiB server cap contained an out-of-memory termination during concurrent
full-reference authoring. Reopen preserved acknowledged state. Final full-reference
verification uses serial execution with the same resource caps; concurrent capacity
for that workload remains unqualified and belongs to E4.
The direct Python inspection fixture retains its original 64 GiB engine allowance,
16 GiB working allowance and recipe-owned native process cap. It does not qualify
full-reference execution in the default 1 GiB managed worker slots; those have their
own smaller worker journeys.
Process-kill recovery does not establish physical power-loss survival, remote TLS deployment,
universal spill or a disk quota. This delivery does not qualify C/D's complete terminal
result/query migration or E4 performance comparisons.

### A mistake made and corrected

**Tested (2026-10-06 completion-audit correction):** the native selection
`just unit-package pse-operations 'test(initialization_) | test(lifecycle_roots_cannot_) | test(explicit_history_roots_pins_) | test(root_and_pin_admission_race_)' --features pse-operations/canonical-tests`
passed 6/6 with explicit relation force-validation, the authenticated isolated RocksDB/gRPC
fixture and a zero-failure baseline. Generic mutation refuses existing run, analysis and
active-attempt roots, while deliberate history remains mutable; protected source survives
the composed reclamation control. Initialization refuses an error-free empty SDK response,
verifies normal/repeated creation and marker interpretation, and distinguishes cancelled
creation from explicit reopen after a committed lost acknowledgment. An initial rerun failed
three cleanup assertions because new fixture names lacked the authorized test prefix; those
names were corrected and all six controls reran. These controls do not establish the cause
of the earlier missing-table diagnostics or physical power-loss durability.

Native concurrent studies exposed missing transaction-conflict retries in protected read
and lease operations. Those operations now rebuild the complete bounded transaction and
recheck the current pin/reclamation guard. Independent review also exposed generic product
root reassignment and permanently blocked recompilation after root release; product roots
are admission-owned and publication identity is distinct from scientific request identity.
Final review found that a lost response from deduplication could not settle an older
publication key. Guarded acknowledgment resolution now returns the actual exact rooted
equivalent, including after preparation expiry.
Large reference authoring also exposed create-first UPSERT repeatedly rolling back
existing guards and rebuilding a growing RocksDB write index. Bounded native guard
reads, updates and inserts replace that operation. The protocol watchdog could return
an empty completion frame after cancellation; this now refuses success, and lost
activation acknowledgments still settle only against the exact immutable revision.
Scope-end generation changed the recorded codec fingerprint, so verification uses a
fresh controlled database without reinterpreting the previous database's records.
Concurrent tool builds exposed a shared xtask executable race in recovery testing;
the recovery control now has its own required-feature binary.
Full scientific descriptions also exceeded the original single-message product bound.
Product headers now point to the shared immutable block mechanism; the transport limit
does not set the logical scientific description limit. Review corrected cross-problem
candidate qualification and publication acknowledgment checks. A nested cleanup branch
returned no completion value; the cleanup decision now returns its bounded frontier.
The recovery reader's second product reservation initially overlapped a completed
first read. The reservation now follows the actual read lifetime; the original finite
budget and protected recovery checks remain unchanged.

### Deviations from the plan, deliberate

The maintainer authorized minimum adjacent C/D/E work required for A/B integration.
Immutable modeling revision references replace bundle payloads in durable worker/study
preparation, and explicit Arrow source export remains available. Full legacy execution,
result/query and crate retirement remains with its owning later companion. No importer,
compatibility constructor or normal in-memory source fallback was introduced. Scientific
admission continues outside short storage transactions.
