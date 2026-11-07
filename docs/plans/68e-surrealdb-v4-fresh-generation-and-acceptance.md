---
title: "Plan68e: Fresh SurrealDB v4 generation and assembled acceptance"
status: in-progress
date: 2026-11-07
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, deployment, storage, validation]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# Fresh SurrealDB v4 generation and assembled acceptance

## Target and ownership

Deliver one selected v4 generation across the existing catalog, private
evidence and knowledge, maintenance, tools and fixture routes. Plan66 keeps
the authority for scientific meaning, protected result interpretation and the
rest of its qualification population. Completed Plan67 supplies working
artifact custody, tool selection, allocation evidence and retirement. Use
the exact target release selected in [68a](68a-surrealdb-v4-supplier-runtime-and-extension.md)
and the generated schema and activation witness from
[68b](68b-surrealdb-v4-schema-transport-and-authority.md).
[Coordinator](68-surrealdb-v4-cutover.md) ·
[O10](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O10) ·
[Canonical deployment owner](../dev/surreal-substrate.md)

### E01 — Prepare the candidate and classify source state

Inventory each actual deployed generation and named current evidence consumer
with the native state/selection owners. Separate three classes.

| Class | Receiving action |
| --- | --- |
| Authored inputs and reproducible outputs | Reinitialize in a fresh owned level-4 catalog from the generated definitions and exact extension. Regenerate ordinary data from source owners. New runs have new identities. Recreate disposable private evidence/knowledge by their existing preparation and selection routes. |
| Named original still required by its protected reader | Verify custody, identity, admitted bytes, descriptor interpretation and required external payload closure. Preserve a portable original where its existing reader can consume it. Convert into fresh v4 once only when the consumer requires a catalog representation. Revalidate the selected item's exact membership, original commitments, interpretation and permissions before release. |
| Released incident evidence, retired derivatives and obsolete fixtures | Follow Plan67 owner decisions and regenerate a named case when needed. Reuse the frozen v3 generation only as the pre-write rollback selection. |

Upstream v4's opt-in in-place 3.x conversion is experimental and one-way.
New stores start at the v4 record format and compatibility level 4.
Select new owned directories and fresh indexes. Keep the old datastore
closed or under its original admission while rehearsing. If a named
one-time import uses a supplier bulk path, `OPTION IMPORT` can bypass
ordinary derived effects. Rebuild the affected derivatives and validate
definition, index readiness and protected membership before activation.
Use existing owner-issued export, inventory, checkpoints, copy and restore
controls to bind files and catalog data together. A database export alone
does not attest external scientific payloads, protected holds or unsettled
operations.
[Upstream 3.x upgrade contract](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x) ·
[Maintained view import and refill](https://surrealdb.com/docs/reference/query-language/statements/define/table) ·
[Supplier backup boundaries](https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery) ·
[Native offline custody](../../crates/pse-operations/src/canonical_files/offline.rs)

#### E01 checkpoint — read-only owner census, 2026-11-07

The candidate checkout's user-manager inventory showed 20
loaded `pse-surreal-*.service` units, all `inactive/dead`; 37 matching unit
files were present (20 enabled, 17 disabled). A targeted native
`pse-host activity` observation confirmed one named unit as inactive with no
main process. The unqualified activity read could not associate this agent
process with a loaded unit. These observations do not identify a catalog's
level, exact bytes, consumer, version, payload closure, or whether other
service managers or manually launched processes own data.

Main's v3 reader still selects the frozen 3.3.4 release. The rollback ref
`pse/plan68-v3-rollback` preserves the corresponding main source, and the
native owner selects the byte-identical release from the separate
`surreal-v3-rollback` tool root. The local default catalog directory is
absent. Keep configured historical catalogs under their existing owners.

The [68a source build](68a-surrealdb-v4-supplier-runtime-and-extension.md)
now supplies the private 4.0.0/MX 1.1.0 release. The normal `server prepare
--from-release` owner imported it into the rehearsal root and preserved
both executable digests, the lock and the original build receipt. The
same selection built the ABI 4 Study extension for the live fixtures.

The existing
`pse-canonical-recovery` inventory/backup/restore route owns canonical
payload-file custody. Identifying a historical catalog's current consumer
and recovery holds requires its original owner.

Classify named evidence consumers through their original owners before
selecting a historical import or release. E02 now runs against the private
server, helper and module receipt. Filesystem-only
`canonical_files::offline` controls exercise copy and inventory integrity.
Live native controls cover catalog initialization, SQL and service recovery.

**Tested, v3 baseline only.** Main's native `scripts/pse test pse-operations
--lib -E 'test(/offline_(complete_inventory_backup_and_lockless_archive_restore|unnamed_copy_interruption_and_exact_published_restore_resume|checkpoint_resumes_exact_request_and_rechecks_every_byte)/)'`
ran three filesystem custody controls with zero failures against a zero target
and complete settlement. The initial filter matched zero tests. This supplies
baseline payload copy and inventory integrity evidence for the named restore
case on the v4 fixture.

### E02 — Rehearse the complete receiving boundary

Use an explicitly prepared private source-built v4 service. Exercise
the target through actual registered product, maintenance and developer
entry points with the matched v3 baseline, preserving each command's
mode and native completion. Minimum receiving population includes:

- **Scientific authority.** Receive Plan66l L02's aqueous Study journey
  through execution, exact original member permission, protected canonical
  query and independent IPC/Proto readback after producer release. Include
  the existing selected fit, trajectory and native result/export routes
  wherever they consume changed storage or interpretation. Assert missing
  versus partial evidence and permission denial independently.
- **Canonical effects and recovery.** Run guarded writes, write skew and
  absent-range phantoms, error rollback, lost acknowledgements, session
  saturation and socket loss, reader protection through drain, and
  acknowledged-write crash/reopen. Inspect exact original results and
  receipts through a separate owner where required. Test fresh initialization,
  invalid activation and owned restore of a named original, including
  changed bytes, incomplete inventory, interruption and wrong authority.
- **Developer receiving and retirement.** Run D02/D03's owned fixture,
  existing finite evidence/knowledge reads, working-report failure and
  current rerun, and physical disposal. Verify foreign endpoint refusal and
  allocation/retirement observations. Do not derive verdicts from newly
  recreated evidence as if they were earlier original outcomes.

Reuse existing controls, including
`aqueous_integrated_original_native_outcome_survives_protected_export_and_portable_read`
and the canonical study lost-acknowledgement fixtures. Inventory all affected
entry points before selecting the assembled commands. These representatives
establish the intended boundaries. They do not reduce the inventory or
complete unrelated Plan66 scientific packages.
[Plan66l receiving scope](66l-required-population-and-assembled-acceptance.md#packages) ·
[Canonical native-service tests](../../crates/pse-operations/tests/native_service.rs) ·
[Developer receiving](68d-surrealdb-v4-developer-and-test-receiving.md)

The first live current-schema runs pass fresh startup, maintenance and
ingress reconstruction, connection-bound session ownership, and HTTP actor
grants. The [68b checkpoint](68b-surrealdb-v4-schema-transport-and-authority.md#private-candidate-checkpoint)
records the catalog and index changes that cleared startup. The targeted
native recovery rerun passed the original checkpoint, process restart,
activation and exact receipt readback. The serializable-isolation and Study
lost-acknowledgement cases passed. Query controls passed after the graph
fixture separated its installation from the permission and read assertions.
The live SurrealKit and scoped evidence controls passed in [68d](68d-surrealdb-v4-developer-and-test-receiving.md#private-candidate-checkpoint--november-7).
The selected native fit journey passed. The aqueous protected-export run
completed the scientific, permission and independent Arrow/Protobuf checks.
Its fixture owner then timed out while draining one canonical connection.
Explicit test-local joins and final-owner changes reproduced the teardown
failure. The original test remains intact. Plan66l owns the assembled
scientific fixture and its remaining lifecycle work. The selected v4
process, recovery and developer disposal tests completed their own drains.

### E03 — Select, open writes and release

The isolated pre-write rehearsal selected the exact v4 server, returned to
the frozen v3 source and receipt, then reselected the same v4 generation.
The workstation has no default catalog and no active SurrealDB service.
The historical catalogs and v3 rollback release remain in isolated custody.

Freeze the admitted v3 selection, close old write admission and verify
physical settlement, including outstanding readers, native operations and
independent release failures. Verify the closed-state selection and named
originals. Before admitting v4 writes, exercise returning to the frozen v3
selection and switching again to the exact verified v4 server, extension,
catalog and client/tool family. This is the rollback boundary. No v3 process
opens v4 state and neither generation receives simultaneous authoritative
writes. Use v4 forward recovery after admitting writes and retain actual
original-operation evidence when a reply or service selection is uncertain.

Release the frozen generation and superseded preparation through their
settled owners after the recovery and protected-read obligations pass. Delete
the replaced runtime, pins, adapters, patches, old production selectors
and obsolete fixture paths. Release one-time conversion inputs when their
named owner no longer requires them. Verify actual allocation separately
from logical deletion. Transfer enduring contracts to the architectural
owners through the applicable decision route and report the deployed
supplier tuple, verified scope and any out-of-scope deferrals.
[Quiesce and restore mechanics](../dev/surreal-substrate.md#quiesce-stop-and-backup) ·
[Plan67e generation release](67e-resource-retirement-and-storage-verification.md#e04--prepared-tool-and-deployment-generations)

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| E01 | In progress | A/B private generation, completed Plan67 state owners | Fresh declared v4 candidate, verified inventory and named original receiving paths. |
| E02 | Platform receiving verified | E01, D02/D03, required A/B native controls and C receiving, existing Plan66 vertical slices | Current v4 storage, recovery, developer and query behavior qualified for fresh source selection. The aqueous fixture drain remains with Plan66. |
| E03 | In progress | Scoped platform receiving and governing decision route | Verify the authoritative v4 source and local default selection. Keep historical v3 custody until its own release conditions hold. |

## Verification and outcome boundary

Use native compile and targeted test routes per package. Select explicit
force-validation for every Arrow-consuming correctness closure. Run selected
assembled native, component, solver/reference and integration evidence and
`just hygiene` after functional integration. Report exact commands, scope,
observed failures against zero, and original settlement. Keep supplier
interface evidence, private tests, durable receiving and measured benefit
separate. Review probes do not establish any of E02 or E03. Optional C
investigations can finish after cutover. A correctness regression on an
existing route blocks E03.
[Qualification guide](../dev/validation-assessment.md)
