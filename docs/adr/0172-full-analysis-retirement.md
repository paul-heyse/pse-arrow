---
id: ADR-0172
title: Fully retire analysis occurrences through bounded admission and settlement
status: proposed
date: 2026-10-09
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-06, AP-07, DP-09, DP-19, DP-22]
blueprint: [§5.2, §20.3, §20.4, §20.6]
review: docs/design_review/reviews/design_review_full-analysis-retirement-independent-target_2026-10-09.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported backend cannot register absent-key conflicts, or source-retention authorities must be destroyed and recreated under identities still reachable by dispatched requests.
verification: Deterministic delayed-issue, missing-header, append, page-read and cleanup races; restartable bounded deletion; expiry renewal refusal; fresh same-content recreation; zero per-analysis residue after stale requests settle.
standard: {core: '3.4', process-simulator: '1.5'}
scenarios: [docs/design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#s07]
---

# ADR-0172: Fully retire analysis occurrences through bounded admission and settlement

## Context

An analysis is currently keyed by its source, method, configuration and inputs. Retirement
releases source roots and writes a permanent retirement marker; the header, graph and input
relations remain. That arrangement prevents missing-header recreation by preserving history.
It does not meet the selected target: a retired analysis must leave no analysis header, graph,
inputs, roots, protection, guard, admission, cleanup marker or retirement/audit payload.

Deleting the marker is insufficient. An already-dispatched creation can reach its transaction
after cleanup. Cancellation, a closed socket, a client timeout and an elapsed admission window
do not prove that transaction has settled. General `protect()` also creates an absent pin with
a newly calculated lifetime, so an analysis-specific retry cannot safely renew that operation
after retirement. The target needs explicit occurrence identity and a database creation fence.

## Scope

Change analysis identity, creation admission, staging, page acquisition and retirement, and
the analysis-input predicate consumed by run retention. Plan 33 EFF10 owns implementation and
acceptance. Run/study retirement contracts, authored inputs, result publication, numerical
analysis algorithms and general result-reader protection remain under their existing owners.
No historical analysis-reader service or general lease service is introduced.

## Drivers

Retirement must be complete, restartable and bounded per transaction. Original requests and
retries must lose effect authority without requiring client-process survival. Later explicit
creations, including the same scientific content, must remain possible. Source and result
retention must hold continuously through admission and partial cleanup.

## Options

Permanent per-analysis tombstones are simple but violate full deletion. Expiry alone has no
commit barrier and can reopen after a backward clock step. Local task draining cannot settle
remote work after process loss. A new global lifecycle coordinator adds unnecessary ownership.
Select an immutable finite creation intent, existing-record conflict barriers, and one monotone
creation-closure scalar on each existing primary source-retention authority.

## Outcome

### Identity and one-shot creation admission

Scientific content identity is separate from an analysis occurrence. Read-only preparation may
discover the existing primary authority and its server clock, without issuing creation or
protection. Before its first creation-effect await, an explicit creation constructs one owned intent containing a fresh nonce, primary problem
authority, absolute finite creation expiry `E`, and the exact immutable request. The occurrence
key binds the primary authority incarnation, nonce and `E` in a checked canonical
representation; the database verifies that binding. Neither the authority nor `E` can be
changed while retaining the same occurrence key. That bound key is the creation operation
identity; a separate transport request identifier cannot authorize another intent. The exact
request digest travels with that intent. Same-operation retries reuse the complete intent;
they cannot call a fresh-intent constructor or recompute `E`. Recreating the same content is
an explicit new operation with a fresh occurrence key. No content digest deduplicates lifetimes.

The database validates positive, bounded initial admission windows, exact primary authority
and key/expiry binding. A per-RPC transport deadline remains separate: recalculating that
deadline must not renew creation authority. The header retains the exact operation, request
digest and `E` while it exists. Reusing a live occurrence with different arguments refuses;
an idempotent retry cannot reactivate a retiring occurrence.

Creation is one transaction. It registers `FOR UPDATE` on the **existing** primary
`canonical_guards['retention:' + problem]` record, refuses absence without creating a guard,
and checks `now < E` and `E > analysis_creation_closed_through`. It registers existing source
retention guards, validates actual revisions/server sequences and reclaimed intervals, and
registers existing execution-run guards while validating input manifests, selections, terminal
attempts, actual result-reader protections and run-retirement eligibility. It then atomically
creates the staging header, source roots and analysis-input relations. Existing limits of
65 sources and 64 result inputs bound this admission. All refusal paths leave no analysis state.

Analysis persistence removes its preliminary generic `protect(source, lifetime)` issuance
loop. Source validation and root creation under the same retention transaction close that
race directly. Already-required source/result protections for work preceding admission keep
their own lifetimes; they are not recreated by an analysis retry. If a source was reclaimed
before admission, creation refuses rather than restoring authority from cached material.

### Existing-record mutation and reader fencing

The header owns `Staging -> Active -> Retiring -> absent`, with `Staging -> Retiring` also
permitted. Every append, activation, analysis-page acquisition, retirement and cleanup
transaction registers the exact header record `FOR UPDATE` before testing its phase. Missing
headers refuse or report an accurately scoped absent outcome without creating guards or rows.
Staging writes require the recorded operation authority; activation cannot reopen retirement.

Retirement writes `Retiring` under that header conflict barrier. This closes new mutations and
page acquisitions. An earlier registered transaction either commits before that write or
fails commit validation; a later transaction sees retirement or absence. Already-returned
owned Arrow pages remain usable. Current handles acquire individual copied pages and do not
hold a durable historical-read entitlement. Any later leased-reader feature would have to
fence acquisition and settle recorded owners before reclamation; it is outside this decision.

### Full settlement without a per-analysis tombstone

The existing primary retention guard gains `analysis_creation_closed_through`, one monotone
absolute-expiry scalar. It contains no analysis key, operation, lineage, retirement status,
cleanup history or list of retired occurrences. Its meaning is source-authority admission:
creation intents expiring at or below this value are permanently closed. It is constant-size
state per existing problem authority, independent of the number of retired analyses.

After `server_now >= E`, final settlement registers the exact header record `FOR UPDATE`,
**including when absent**, and the existing primary retention guard. It verifies that all
analysis-owned rows have been drained, advances the scalar to `max(old, E)` with a real guard
write, and deletes the drained header and any remaining temporary analysis-specific state
atomically. If an absent-header snapshot loses a race to creation, absent-key registration
forces conflict/retry so settlement observes and retires that header. An older creation
snapshot registered the same primary guard: it must have committed before this fence or fail
commit validation. An issuance beginning after the fence sees the closed expiry and refuses.
No late request may upsert an absent header, pin or analysis guard.

This settles **effect authority**. It does not claim the server task or network response has
finished. A committed earlier page response may arrive later; a late uncommitted transaction
may return a conflict. Neither may recreate persistent analysis state after final settlement.
Only definite transaction conflicts may retry automatically, using the original intent.
Uncertain delivery is resolved through that same intent and server state, never a fresh issue.

Normal valid fresh intents have `E_new > now >= closed_through` and are unaffected. A backward
clock step cannot revive an old intent because the scalar is durable. It may temporarily make
a newly constructed window fall below the scalar; that request truthfully refuses until an
explicit fresh valid window is possible. Safety must not be bypassed by clearing the scalar.

The primary guard must outlive every dispatched request capable of targeting that authority.
All guard mutators preserve the scalar and never decrease it. Missing guards refuse; generic
guard bootstrap is not reachable from analysis operations. Problem/database reset or restore
cannot reuse an old authority incarnation with a reset scalar while old requests remain
addressable. Such lifecycle changes require a fresh fenced incarnation. This preservation
obligation is part of the source-retention authority, not a retained per-analysis tombstone.

An absent header alone does not establish full settlement while creation may be in flight.
The caller retains its original intent for settlement; after restart an existing header
supplies its original expiry and primary authority. Settlement of an absent header with that
intent uses the same two conflict barriers and closure scalar. A lost acknowledgement can be
resolved by checking the closed expiry and empty owned rows. A bare unknown key without its
authority/expiry cannot yield a claim that an unknown creation grant has been settled.

### Bounded and restartable physical cleanup

The retiring header stores temporary cleanup progress. Cleanup transactions use indexed
analysis ownership and bounded pages, currently 64 rows. Delete edges before nodes: SurrealDB
node deletion cascades to connected edges, so node-first pages could hide unbounded edge work.
Then remove remaining nodes, input relations, source roots under their retention guards and
temporary analysis metadata. Native relation deletion owns graph-pointer removal. Source
roots remain until their actual release; a retiring header is discoverable for restart/resume.
An unclosed creation window yields pending settlement, not a completed retirement. Explicit
retirement drives or resumes cleanup until completion; temporary progress is not an audit.

Any surviving `canonical_analysis_inputs` row retains its referenced run's results. Run
retirement must test physical input-row existence through its run index, without a negative
retirement-marker predicate or an inference from an absent analysis header. Admission creates
those inputs under execution-run conflict guards; run retirement writes the same guards.
Physical input deletion releases the claim. Conservative refusal during partial cleanup is
safe and disappears when cleanup removes the relation.

Final bounded existence checks cover header, nodes, edges, input relations, source roots and
all analysis-specific protection/admission/guard/cleanup records. Completion leaves none of
them and no permanent retirement marker or audit record. The unrelated shared retention
authority remains, carrying only its aggregate creation-closure scalar.

### Transaction capability and evidence boundary

The selected server uses SurrealDB 3.3.0 with the RocksDB backend. This decision relies on
commit-time conflict registration, not globally serializable transactions or lock waiting.
In pinned source, `dbs::processor::process_record` routes `SELECT ... FOR UPDATE` to
`get_record_for_update`; `surrealdb-datastore::tx` reaches the underlying `getu` even when
the record is absent, before returning a default absent record. The RocksDB implementation
calls `get_for_update_opt(key, true, snapshot_read_options)` in a writable transaction, then
validates on commit. Existing upstream `select_for_update_test` cases distinguish locked
reads from plain snapshot reads and exercise conflict failure after a concurrent update.
These sources were inspected, not executed for this decision.

The application must use writable explicit transactions for these reads and preserve the
selected point-record path even for absent keys. Predicate scans alone do not supply phantom
protection. Missing-key creation, deletion and exact emitted-query races require deterministic
local controls before implementation acceptance. A substitute backend must supply equivalent
registration/commit behavior or refuse this capability; snapshot isolation alone is insufficient.

### Consequences

Generated analysis/retention functions, schema declarations, store bindings, runtime identity
and Python-facing occurrence handling move together. Remove the permanent analysis-retirement
table/path and old content-key recreation refusal after every caller has moved. Internal
analysis-product cutover must dispose of replaced internal rows through their retention owners;
authored/external inputs are not reset. No compatibility decoder or second production path is
retained. The source-retention guard's new scalar and lifetime contract are enduring obligations.

The explicit design-phase cutover preserves identified authored/external files before the
isolated lifecycle owner closes admission and drains holders/server. A current initializer
builds a fresh database/authority; old revisions are not relabelled. Only the identified
replaced internal database is disposable after preservation and acknowledged initialization.
Canonical admission remains closed throughout initialization. A separately selected initializer
requires the live lifecycle owner nonce/process/start/boot identity and exact fresh
namespace/database; owner death invalidates permission. Ordinary handles reject pending
maintenance independently of a stale admission boolean. Failure leaves the cutover closed.
The explicit recovery command resumes a durable rebuild/restore phase, verifies preserved
inventories and observes uncertain account/database effects before retrying. Restore writes
a fresh closed profile before any backup copy; unknown content or changed inputs refuse
readmission. The old normally startable profile is never copied.

Maintenance keeps ordinary canonical admission closed and drains registered holders and the
ordinary server. Its live lifecycle owner may run a temporary authenticated loopback server
using the declared binary, storage configuration and charged allocation. Native SQL/export/import
clients connect remotely and never open RocksDB directly: the pinned embedded CLI drops the
server's RocksDB configuration. This listener establishes no ordinary readiness, provisions no
accounts and starts no scientific workers. It has no automatic restart. Physical copying,
initializer handoff and ordinary readmission require acknowledged stop and actual descendant
drain. Owner loss or uncertain shutdown leaves admission closed and resource ownership retained;
recovery first settles the exact prior service invocation. Recorded ROOT credential candidates
are probed through fresh connections without selecting a missing catalog context or provisioning
credentials. Loopback binding alone does not fence holders of privileged credentials.

Same-interpretation restore removes copied analysis state
and rotates source incarnations while preserving closure floors before readmission, using a
fresh database identity and removing the old copied database. A new supervisor instance ID
alone does not revoke access by old authenticated namespace/database clients. Shared or
uncertain ownership refuses these actions; ordinary opening never invokes them.

### Compensating controls and confirmation

Deterministically hold creation before commit while retirement first observes no header;
exercise both commit orders. Hold an append, activation, page acquisition and cleanup against
retirement/final deletion. Dispatch the original issue before expiry but delay commit beyond
expiry and cleanup; repeat with server-side transaction suspension and client restart.
Exercise exact same-intent retries, forbidden renewed expiry on the same key, missing guards,
backward clock movement, lost acknowledgements, partial cleanup restart and high-degree nodes.
Verify source GC and run retirement at both admission commit orders. Recreate the same content
with a fresh valid occurrence after settlement. Query every owned table/root/guard/pin namespace
after delayed requests settle and require zero per-analysis residue. Verify all shared guard
writers preserve the scalar. Named executed controls are required for Tested claims; this ADR
does not provide behavioral or performance qualification.

## Pros and cons

The design uses existing store transactions, roots and conflict authorities and removes retained
analysis history. It adds one constant-size scalar and a key/expiry binding, plus explicit
pending settlement until the original finite creation window closes. Shared authority reset
now has a concrete fencing obligation. Those costs replace permanent per-analysis markers and
client-dependent drain assumptions; they do not create a general lifecycle framework.

## More information

Contract owners: blueprint §5.2 and §20.3–§20.6. Current implementations:
`pse-runtime/src/workflow/analyses.rs`, `pse-operations/src/canonical_analyses.rs`,
`canonical.rs`, `canonical_retention.rs`, `canonical_transport.rs`,
`pse-codegen/src/codegen/surreal_{analyses,execution,retention}.rs` and
`pse-schema/src/catalog/substrate.rs` under `crates/`.
Current transaction documentation was retrieved through Context7 from the
[SurrealDB transaction guide](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/learn/querying/concepts-and-guides/transactions.mdx);
the pinned 3.3.0 source paths and backend behavior above control the capability judgment.

## Status history

- 2026-10-09: Proposed for Plan 33 EFF10; author target review recorded separately. No
  implementation, executed transaction-race result or product acceptance is implied.
- 2026-10-09: Independent target review Accept at Proposed strength; C01–C06 remain
  implementation acceptance obligations. Architecture amended under revision 144.

- 2026-10-09: Clarified that read-only authority/clock discovery precedes intent construction; the immutable intent exists before every creation effect and is never renewed by retries.
