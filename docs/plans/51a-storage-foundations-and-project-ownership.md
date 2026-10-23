---
title: "Plan 51a: Storage foundations and project ownership"
status: complete
date: 2026-10-22
adrs: [ADR-0216]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, storage, libraries]
---

# Plan 51a: Storage foundations and project ownership

## Target and boundaries

**Implemented:** extend the existing canonical SurrealDB service with owned project/research
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

**Tested, 2026-10-22:** ST01–ST04 foundations and their first real consumers passed the targeted
checks below against a zero-failure baseline on Linux, using the selected custom SurrealDB
3.3.4/MX1.0 service. Arrow-consuming commands used the scope-verified force-validation route;
the Python component used the linked force-validated correctness extension. The coordinator
owns package status; these checks do not qualify the enclosing series.

| Command / selected scope | Final result |
| --- | --- |
| `scripts/pse-env -- just codegen` | Passed all five generation targets; generated Rust/Python documents and compiled API stubs published through their owners |
| `scripts/pse-env -- just check-package pse-operations -p pse-runtime -p pse-py -p xtask --features xtask/canonical-tools,xtask/evidence-tools` | Passed affected all-target compile, zero errors |
| `scripts/pse-env --store -- just unit-package pse-operations 'test(canonical_projects) \| test(project_research_reference_retains_exact_closed_results)' --features pse-operations/canonical-tests` | 9 passed, zero failed; independent heads, CAS, exact replay, forged intents, source closure, authority changes and retained closed results |
| `scripts/pse-env -- .venv/bin/python -m scripts.arrow_validation test -p pse-operations --features canonical-tests --lib evidence::tests` | 4 passed, zero failed; strict codecs/membership, scoped roles, partial source coverage, staged/history cuts, lost replies, replay, changed issuer and atomic disposal |
| `scripts/pse-env --store -- just unit-package pse-runtime project_large_draft_ack_rejects_coherently_forged_original_artifact_header --features pse-runtime/canonical-tests` | 1 passed, zero failed; coherently forged large intent refused, selected descriptor retains original body after current draft advances |
| `scripts/pse-env --store -- just unit-package pse-operations 'test(canonical_artifact_owner_release_fences_both_acquisition_orders) \| test(canonical_artifact_live_read_prevents_collection_after_owner_release) \| test(canonical_files_unknown_collection_and_forged_intent_leave_no_physical_residue)' --features pse-operations/canonical-tests` | 3 passed, zero failed; existing protected-owner regression controls |
| `scripts/pse-env --store -- just unit-package pse-operations 'test(grpc_exact_cells_and_guarded_revisions) \| test(concurrent_identical_source_api_edits_return_one_exact_revision) \| test(native_kind_authenticates_every_known_refusal_and_unknowns_fail_closed)' --features pse-operations/canonical-tests` | 3 passed, zero failed; existing source/codec/refusal regression controls |
| `scripts/pse-env -- just unit-package pse-codegen evidence_family_does_not_change_canonical_installation` | 1 passed, zero failed; a tooling-only declaration cannot change canonical installation identity |
| `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_evidence_projection scripts.tests.test_evidence_store` | 23 passed, zero failed; authentic host adapter, no implicit scientific opening, literal argument dispatch and inherited activity/drain |
| `scripts/pse-env -- just py-sync-correctness`, then `scripts/pse-env --store -- .venv/bin/python -m scripts.python_tests python/pse/tests/test_projects.py -m component -q --basetemp FRESH_OWNED_DIRECTORY` | Correctness build passed; 1 component passed, zero failed; raw custody, admission/publication/reopen, replay after draft edits, large sealed custody and altered-intent refusal |
| `just evidence-build`, `just project-build`, `just project --help` through `scripts/pse-env --` | Both dedicated operators built; project command surface exercised |
| Public `just evidence prepare/init/import/query/status/generations/snapshot` through `scripts/pse-env --` | Passed with authentic invocation sources: repeat A after importing B preserved B, actual command failure retained partial coverage, final successful project-component cut selected |
| `scripts/pse-env -- just turn-end` and `scripts/pse-env -- just docs` | Passed ADR index/format bundle and documentation publication; these are mechanical checks, not product qualification |

The initial Python attempt failed before collection because the shared `/tmp/pytest-of-paul`
directory was root-owned. A fresh owned `--basetemp` resolved that environment issue;
`just ready` passed. The shared directory was not changed. Earlier compiler and native control
failures drove the corrections below; their final affected checks passed. No baseline was waived.

**Not run:** full hygiene, whole-series integration/scientific journeys, performance campaigns
and platform qualification; these remain at 51g. **Measured:** no performance benefit is claimed.

## Outcome

**Implemented:** registry-owned project/branch/draft/research declarations, native indexed
operations and independent evidence installation/codecs. Branch publication extends the existing
source activation transaction; it adds no second head. Saved raw drafts admit science only through
the ordinary package loader and guarded source activation. Large drafts retain an original sealed
artifact intent and hydrate selected descriptors through existing protected readers. Exact research
references retain original source and closed-result owners.

**Implemented:** explicit evidence initialization/open/import/activation/query/history/rebuild/drop
and a first authentic FileStore invocation adapter. Reader and importer are database-scoped VIEWER
and EDITOR; EDITOR has DDL power within the disposable database, not user management or canonical
authority. Opening is independent of scientific schema/modules/native receivers. Host activity
holds the actual native child's kernel lifetime through connection drain. Lifecycle errors retain
the exact installation/disposal issuer and selected database. The ordinary source activation path
was extended directly; no prototype store or compatibility activation path remains.

**Tested:** independent review exposed stale-generation reactivation, an attestation/removal race,
mutable intent coordinates, false large-body settlement and a second mutable draft read. Corrections
settle immutable receipts, fence removal in one native SSI transaction, validate full retained
intent identity and hydrate the selected descriptor. Disposal controls prove native REMOVE/CANCEL
rollback and refusal after a concurrent replacement. The final focused checks above passed, and
the review confirmed closure of its reported findings. This is bounded implementation review,
not a whole-plan completion audit.

Deliberate choices: dedicated `pse-project`/`pse-evidence` artifacts avoid codegen replacing an
operator-enabled `xtask`; large custody uses retained intents rather than minting a replacement
artifact on retry. Fork closure uses ordered exact native selections and the existing shared
source bound. No dependency upgrade, alternate backend, new memory policy or scientific workflow
replacement was needed. These are implementation choices within the selected target, not waivers.

Enduring boundaries are at blueprint §20.5–§20.6 and §24.1 with ADR-0216; operational usage is in
the [substrate guide](../dev/surreal-substrate.md#project-and-derived-evidence-operations) and
[agent environment guide](../dev/agent-environment.md#project-and-evidence-commands).
PJ01/PJ02/PJ04 and EV01/EV03 have initial foundation consumers, not full package closure.
Research datasets/calibration, offline columnar queries, full workspace UX, lifecycle release and
exchange remain with 51b–51f. Keep this completed foundation packet while those consumers and
51g qualification depend on it; retire it through the coordinator's Q04 handoff.
