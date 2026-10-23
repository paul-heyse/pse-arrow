---
title: "Plan 51a: Storage foundations and project ownership"
status: draft
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, storage, libraries]
---

# Plan 51a: Storage foundations and project ownership

## Target and boundaries

**Proposed:** extend the existing canonical SurrealDB service with owned project/research
metadata operations and a separately disposable tooling evidence database. Retain sealed bulk
payload files and their protected exact-membership readers. This is the selected foundation,
not an execution-stage backend competition. The [coordinator](51-engineering-workspaces-and-evidence.md)
owns package state, O10/O13/O16/O17 dispositions, rule impacts and the shared contracts.

Project state belongs with the existing canonical source/study/result catalogue because its
operations consume those identities and invariant checks. Imported test and diagnostic facts
belong in a separate tooling database on that service: an importer must never acquire the power
to settle invocations, overwrite scientific results or release their resources. The tooling
database has its own declaration, access role, import generation and rebuild/drop lifecycle.
It does not require another daemon, generic storage interface or new scientific data authority.

The current custom SurrealDB 3.3.4/MX1.0 integration, serializable snapshot isolation, gRPC,
capability configuration and installed Surrealism module remain the deployment basis. Integrate
new queries with its existing attestation and health paths. A catalogue transaction owns a
metadata decision; native science, file staging and interactive sessions stay outside it.

## Current seams and foundation assessment

**Interface-checked:** the source baseline is recorded in the coordinator. Inspect
`pse-operations` canonical source, study, result and retention operations; `pse-schema` registry
declarations; runtime `workflow/worker.rs`, `workflow/modeling/canonical.rs` and
`workflow/modeling/canonical/local.rs`; generated query declarations through their generator;
and `canonical_results`, `result_snapshot`, `result_files` at their current owners.

The consumed database behavior includes unique run sequence/attempt generation/study occurrence
indexes, enforced edges and phase constraints, immutable terminal seeds and same-transaction
readiness/policy decisions. Replacing that substrate would transfer real behavior, not just CRUD.
`canonical_results::open_result_file` checks protected membership. The snapshot reader holds
`FileReader<ProtectedFile>`; `access_block` accounts for integrity/footer/body/alignment work and
uses indexed file reading. This is seekable protected access, not established mmap or zero-copy.

The supplied [KV overview](../libraries/SurrealKV.md), local source
`docs/libraries/SurrealMX.md`, [SurrealDB overview](../libraries/surrealdb.md) and
[tooling overview](../libraries/surrealdb_tooling.md) orient the investigation. Consequential
mechanisms are checked against the exact versions:

| Mechanism | Role and condition |
| --- | --- |
| MX `with_serializable_snapshot_isolation()` | Keep the patched adapter's selected isolation. MX1.0's default alone does not change an adapter explicitly selecting snapshot isolation. Check the repository adapter, not stock release assumptions. |
| SurrealDB indexed queries, relations and transactional statements | Express selected project membership, head/generation checks and compact evidence filters in the owned query declarations; reuse native enforcement instead of duplicating a Rust index. |
| Existing Surrealism query/module surface | Use installed, attested operations where the existing boundary fits. Add module code only for an actual owned computation; project organization is not a reason to move numerical execution into Wasm. |
| gRPC `query_stream` / `subscribe_live` | Available mechanisms for selected query delivery/refresh. A successful terminal statement establishes query completion; subscription `resume_from: None` does not provide durable history. |
| MX statistics and configured memory/age/backpressure controls | Observe the actual selected engine and existing health route. Do not equate approximate engine accounting with host RSS or change ordinary environment limits as part of this plan. |

Primary sources: [MX transactions](https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/tx.rs),
[database/options](https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/db.rs),
[SurrealDB 3.3.4 memory adapter](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs).
The repository's custom integration remains the authority for its actual selected build.

### Alternatives settled during authoring

| Alternative | Benefit fairly considered | Why not selected / observable revisit |
| --- | --- | --- |
| Direct persistent KV1.0, metadata plus payloads | Atomic bounded values/catalogue and one recoverable engine root can simplify publication. | SI-aware application invariants and owned query/index code replace consumed database behavior. `Value` is owned bytes; large products need seekable chunk access and closed manifests. Revisit a required atomic-byte operation the chosen system cannot supply, or measured workload failure. |
| Embedded SurrealDB with patched MX | Keeps higher-level native query behavior and can support an independently editable desktop. | Needs the same adapter patches, core feature forwarding for Surrealism, runtime capabilities and module installation. Independent workers/observers still need brokerage or changed ownership. Revisit a concrete standalone-editable delivery requirement. |
| Additional direct MX working collection | Strong native transactions and ordered collections for single-process state. | Existing immutable source/RPDS state plus catalogue transactions cover the identified operations. Revisit an actual operation that cannot fit those owners. |
| Split MX/KV hot/cold store | Different residency and persistence characteristics. | No selected cross-engine publication/replication requirement justifies a second lifecycle. Revisit a concrete workload and declared consistency model, not library availability. |

KV1.0 value-log reclamation is prefix-based; a long-lived early live value can retain later
obsolete log files. Inline values trade that issue for compaction rewriting. Native checkpoints
are useful engine closures but do not cover externally sealed payloads automatically. Inspect
[KV transactions](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/transaction.rs),
[Tree](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/lsm.rs)
and [checkpoints](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/checkpoint.rs).
The stock [3.3.4 KV adapter](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-surrealkv/src/lib.rs)
uses 0.21.4 APIs including `with_versioned_index`; selecting KV1.0 would be a real port.
These alternatives are not mandatory execution prototypes or interchangeable backend switches.

## Repository operations and physical work

Use existing registry declarations for new product relations and stored tooling projections.
Keep original host-only Python documents with their native codecs/readers. Project metadata
references exact admitted source/physical closure, branch head and ownership; evidence metadata
references imported source identities and complete import generations. Neither stores a duplicate
scientific model or blanket copy of raw artifacts.

An operation receives its selected authority/context, typed arguments, exact expected generation,
operation identity and original allowance. Read the decision-relevant rows in one short snapshot,
check native constraints and publish the decision atomically. A conflict retries that entire
decision within its original allowance. An uncertain response settles the exact operation before
another effect. Never silently convert a failed durable write to ephemeral success.

Query project/cohort/import membership using indexed native predicates, stable ordering and
bounded headers. Freeze exact membership and completeness before opening selected payloads.
Materialize selected detail outside the metadata transaction through current protection owners.
Avoid N+1 scalar/result fetches, one transaction per projected cell and scans of every raw file.
Use selected Arrow batches/DataFusion for the analytical portion rather than store-side numerical
evaluation. Pagination bounds delivery, not automatically all server work; inspect query plans
and add an index only for an actual operation and its update/maintenance cost.

Tooling evidence can be queried offline from supplied authentic sources when the service is
unavailable. Persistent import never becomes a prerequisite for `activity`, owner-record inspection,
terminal composition or emergency recovery. Tooling database credentials confer no lifecycle
effects on the canonical database; apply existing capability/permission configuration to real
public commands, not only internal repository methods.

## Work packages

<a id="ST01"></a>
### ST01 — Declarations and decisions

Declare the project/branch/draft, research lineage and imported evidence meanings at their
existing schema owners. Prepare applicable short ADRs for new relation families before their
implementation, following RC06 and the existing decision route. Map each field to its scientific,
presentation or derived meaning; require original-authority references and explicit unavailable
values rather than inferred schemas. Generate bindings/queries/contracts with `just codegen`.

Readiness: current source/time and result identity contracts are available; PJ01/SC01/EV01 specify
the consumers of each declaration slice. Do not block all consumers on one monolithic schema.
Acceptance: targeted declaration/codec round trips reject wrong identities, duplicate membership
and contradictory time/coverage meaning; generated Rust/Python consumers agree. Delete any
prototype parallel declarations in the same change. No actual rule waiver is selected here.

<a id="ST02"></a>
### ST02 — Project and research metadata operations

Implement operation-shaped create/fork/list/select, draft-generation, source-head publication and
lineage-reference repository operations. PJ01/PJ02 supply the first real Rust/Python/CLI consumers;
SC01–SC03 extend the same owner for research records. Use current service selection, source admission,
canonical CAS and strong result references. A reusable internal helper may share mechanics, but
there is no public backend-neutral key/value facade or second source authority.

Acceptance: two concurrent writes against one expected head cannot both publish; independent
branches have independent source heads; replaying an uncertain successful command settles once;
missing/foreign source references fail before publication. Test through the real owned operation
and one public consumer. Move/delete replaced ad hoc catalogue lookups alongside migration.

<a id="ST03"></a>
### ST03 — Derived evidence database

Provide explicit create/open/import-generation/activate/query/rebuild/drop operations for the
separate tooling database. An import stages compact typed metadata, reconciles expected source
coverage and activates a generation only when complete. Interrupted imports remain distinguishable
and repeatable; a query receives an explicit source cut/freshness description. Configure a narrow
reader/importer role and use existing service diagnostics without spawning another server.

Give this database an explicit initialization, interpretation/schema attestation and narrow
reader/importer roles of its own. Reuse service/transport lifetime, not the ordinary scientific
`CanonicalStore`/`Runtime` opening sequence that attests scientific interpretation, installed
modules and native receivers. An unavailable tooling index cannot prevent opening scientific
authority, and tooling readers need no scientific mutation or native execution privilege.

Readiness: ST01's projection declaration and EV01's source identity/coverage shape. EV02 offline
queries do not depend on this package. Acceptance: interrupted/repeated imports do not expose
partial generations or inflate counts; dropping/rebuilding the index cannot mutate source terminals,
resources or result descriptors; unavailable service leaves supplied-only inspection usable.

<a id="ST04"></a>
### ST04 — Shared selection, effect and lifetime integration

Integrate selected membership/cursors, typed errors and exact-operation settlement into every
new public consumer as it lands. Keep metadata result lifetime, protected live payload borrowing,
copied offline closure and observer lifetime distinct. Route protocol errors and failed statement
completion truthfully; do not certify provisional streaming rows as a complete selection.

Acceptance: wrong issuing cursor context, released payload, changed authority, stale generation,
connection loss after commit and error after provisional rows expose the appropriate distinct
failure. Observe no duplicate mutation or granted file lifetime from a metadata-only result.
This cross-cutting package participates in early consumers; it is not a late safety retrofit.

## Verification and handoff

**Proposed:** targeted tests accompany each implemented slice; compile the affected operation,
runtime/tool and Python closure with explicit Arrow force-validation when applicable. Relevant
assembled and static qualification belongs to [51g](51g-qualification-and-publication.md).
No backend benchmark, full upstream DST or new platform claim is required for these authoring
choices. Performance improvements remain unmeasured until a named workload establishes them.

Current checkpoint: design and source assessment only. All package progress is at the coordinator.
First executable work is ST01's project-head/source and evidence-generation declaration slices,
then ST02 with PJ01/PJ02 and ST03 with EV01/EV03. No storage production code has changed.
