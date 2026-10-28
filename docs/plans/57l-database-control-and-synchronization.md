---
title: "Plan57l: Database control and synchronization"
status: in-progress
date: 2026-10-28
adrs: [ADR-0232]
review_sources: [docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, design, storage, libraries, numerical-execution, validation]
---

# Plan57l: Database control and synchronization

## Purpose and foundation assessment

**Proposed:** consolidate complete durable control operations inside the canonical database
where this removes crossings and coordination, while retaining scientific and physical owners
needed outside transactions. Consume the [principal review](../design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md),
[scoped audit](../design_review/audit/study-execution-and-database-boundary.md) and
[exact research basis](../design_review/evidence/study-execution-and-database-boundary.md).
The [coordinator](57-library-leverage-and-correctness.md#database-boundary-review-integration)
owns disposition. Baseline is product `64a217de2` with retained inheritance corrections,
audit `5e8aea0cc` and review `277eee026`; Core 3.4, heuristics 1.0, test assessment 1.2
and process-simulator 1.5 apply. This is planned work, not renewed database qualification.

The existing database already owns leases, claims, generations, receipt reconciliation,
cancellation and file intents. Generated readiness acquires bounded facts, calls shared
Rust policy through Surrealism and applies hints in the same snapshot. Preserve these
strengths. The target is further useful integration, not a second scheduler or another
Rust-to-database export of readiness facts. Scientific source interpretation, preparation
and original-model acceptance remain shared owners used by local and worker execution.

The selected patched SurrealDB 3.3.4 build runs persistent MX 1.0 with SSI and synchronous
AOL/fsync despite its `mem://` URI. KV 1.0 is eligible but not integrated as the selected
backend. MX snapshot pins do not protect exported files, prove native join or establish
host process identity. These are different lifetime scopes, not reasons to reject integration.

## Target operations and concrete mechanisms

L01 maps each complete submit/cancel/reconcile/claim/renew/finish/read operation to its
actor, inputs, authoritative state, effect, transaction and recovery unit. A command
transaction validates immutable intent, relevant revisions and authority, applies its
bounded state transition and records the exact receipt. A worker executes outside it,
then publishes assessed output under the original fence. Unknown acknowledgement is
resolved against that receipt. Durable cancellation requests stop; terminal visibility
must not claim physical completion merely because cancellation committed.

Prefer existing SurrealQL functions and indexes. Move guard/body/postlude compositions
inward only when one transaction preserves the entire decision, original clock, exact
outcome and bounded response. Count actual round trips and retained representations,
not async method names. Do not combine distinct host effects into a claimed atomic unit.
Preserve bulk creation resume, compact outcomes, coalesced finalization and lazy cohorts;
the audit corrected older per-point/eager/unconditional-file premises at these sites.

### Bounded module composition

Surrealism 0.6.4 `sql`, `sql_with_vars` and `run` inherit the caller transaction under
narrowed host capabilities; they are not ordinary SDK sessions. Compare a guest call
with direct SurrealQL and the current pure shared policy. A proposed writeable export
must name its effect, allowed expressions/functions, snapshot, deadline, pool, memory
and failure behavior. Conditional RC04 permits this assessment, not blanket capabilities.
Volatile module KV is outside record transactions and can be shared by artifact aliases;
it cannot become durable claim/receipt authority. Network effects cannot roll back with
records. Broader eligible computations need their own WASI/library and server-contention
assessment; no complete native solver migration is selected by capability availability.

### Connectivity, synchronization and physical execution

Preserve one authoritative claim/fence path for workers and all public clients. L03
compares existing polling with bounded LIVE/Subscribe wakeups and changefeed catch-up.
The selected gRPC Subscribe rejects `resume_from`; lag can terminate a consumer. A reconnect
must re-establish authoritative state, not interpret silence as no work. If a subscription
is selected, define admission, finite notification allowance, teardown, registration/reread
ordering and polling backstop. A retained polling decision needs its reason and observable
reopening trigger; library availability alone is insufficient to add a perpetual listener.

Events may express transactional database consequences. Async queue insertion is transactional,
but later delivery/retry is separate and exhausted retries can remove work with a warning.
Events neither supply exactly-once external execution nor fire just because lease time passes.
Keep an explicit deadline/recovery authority. A replacement worker must fail stale publication
and retain old physical work until its performing owner proves drain. G owns authenticated
storage/receiver identities and supervision; F owns native joins and stop semantics.

### Payload/catalog and backend inquiry

L04 compares current inline/sealed-file placement with database-contained payload transitions
over representative small/large results, retention and concurrent readers. Identify the full
transaction, encoded/decoded live state and catalog visibility, not just a faster engine call.
Colocation is useful if it actually removes publication/attachment recovery. When files remain,
retain integrity, sync, intent, no-clobber and exported-reader protection. Bucket filesystem
writes are not established KV-atomic publication. A database reader pin cannot substitute
for an external reader lease.

Object_store 0.13.2 range/stream/conditional primitives are candidates for delivery; local
Update/durability semantics do not replace this protocol. Its newer multipart cleanup fix
is a family-upgrade lead, not an attributed current canonical bug. Compare an explicit coherent
DataFusion/object_store move only if selected use needs it. Bytes::from_owner already carries
backing ownership; slicing can retain larger storage and copies still need flush custody.

Assess KV 1.0 only against a concrete unmet backend need: SI/exact-key validation differs from
selected MX SSI. Any adoption needs adapter/build, isolation, durability, migration/restore and
actual application qualification. Retaining MX is a valid inquiry outcome. No backend switch,
data reset or quantitative performance claim follows from this document.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| L01 | open | Plan57l | Shared actor/transition contract and bounded composition choices settled with K01 |
| L02 | pending | Plan57l | Selected server command/ingress slices enforce original semantics and actor isolation |
| L03 | open | Plan57l | Worker discovery/catch-up decision and any selected integration preserve authoritative recovery |
| L04 | open | Plan57l | Payload/backend inquiry chooses placement, retained obligations and any conditional migration route |
| L05 | pending | Plan57l | Selected consumers and declarations migrate; replaced coordination is deleted and J receives evidence |

L01 supplies the early contract to K02; it does not wait for the entire Python client.
L02 consumes K02's ingress decision and applicable ADR/design amendments. It implements
each complete selected operation with its schema/codec generator, actor checks, original
receipt and targeted tests, allowing K03 to consume a working endpoint slice. Any module
capability change follows the conditional RC04 decision route before implementation.
Changed catalog digests require explicit activation/readmission; unchanged data is not reset.

L03/L04 may investigate alongside that work. Their outputs state the question, inspected
exact source, decision, retained semantics, dependent packages and reopening trigger. A
negative selection completes an inquiry, not an unimplemented alternative. If a result
changes governed contracts, obtain explicit disposition for any newly exposed rule impact
and complete the decision route before L05. L05 integrates only selected results, coordinates
E/G/K consumers, removes replaced functions/RPC loops/tests when proven and hands J actual
affected routes. Do not retain parallel production schedulers or dual-write migrations.

## Verification and checkpoint

**Interface-checked:** research/source capability contracts. **Proposed:** all packages.
Review S02–S06 guide future acceptance: concurrent actor/revision conflicts, exact repeated
command versus reused identity, lost acknowledgement, restart and stale fences, cancellation
before/after dispatch, dropped notifications, reconnect, lease expiry, database unavailability,
module refusal and payload interruption before/after attachment. Refused setup is not a
passing scientific negative case. Prove actual worker execution, assessed publication and
old-owner drain through J, separately from policy/unit controls.

An added study composition should extend shared policy; replacing a worker should preserve
receipts; growing result detail should change demand-shaped data access rather than require
the scientific root for headers. These scenarios motivate the boundaries without adding new
scientific methods. J owns joined qualification, independent reconciliation, decision/architecture
publication and benefit selection. Retain the current pause and stopped resources until a
separately authorized execution approach reconciles the changed baseline and consumers.

## Authorized first control slice — 2026-10-28

The maintainer authorized K01/K02/L01 and the browse/cancel/reconcile portions of L02,
K03/K04 with bounded J acceptance. The selected first API is async; import independence
within the existing distribution is required, independent installation is deferred.
ADR-0232 develops constrained DEFINE API ingress, HTTPX2 2.16.0/AnyIO 4.15.1 bounded
async delivery, authenticated record actors and durable exact cancellation receipts.
Existing cancellation state idempotence is not a request-specific receipt. Receipt absence
is unobserved, not proof a submitted command cannot commit. Source-only import/test
startup retains scientific validation at actual scientific consumers. Other K/L scope
and the wider D–F qualification campaign remain outside this selected execution.
