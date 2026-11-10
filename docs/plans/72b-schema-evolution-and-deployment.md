---
title: "Plan72b: Schema evolution and deployment"
status: active
date: 2026-11-09
adrs: [ADR-0262]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, deployment, design, storage, validation]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md]
---

# Schema evolution and deployment

## Target and baseline

An operator previews exact transition effects and reconstructs explicitly disposable state
into a clean v12 target under the original operation. The maintainer's execution clarification
selects fresh empty reconstruction: existing scientific results need not survive this design-phase
cutover. Fresh issuer/credentials, validated installation, original-operation recovery and explicit
old-state disposal are the selected implementation. Authored inputs outside owned state remain
untouched; reconstructing an empty database does not claim to import authored models.

The [coordinator](72-surrealdb-functional-improvements.md) owns O03 and shared rule decisions.
This companion owns B01–B03. Plan66 and Plan71 remain active independent owners; their files
and acceptance scope do not change. No scope is inherited from the superseded plan.

**Interface-checked baseline:** `21f703887cc307f986975f9a2fba998f8bd2afa7` with the
concurrent working tree. Current [FrozenRollout](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_maintenance/frozen_rollout.rs)
checks bounded hashed schema files and explicit removals, including a data-witness flag.
The [owned preview](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/native_service/rollout_preview.rs)
binds the manifest to catalog/listener/module/tool identity. These are suitable authority
foundations; the narrow action enum is the deliberate extension boundary. Do not bypass it
with a generic runner to gain native data migration.

Use selected core, efficient architecture, process-simulator and binding criteria. RC01's
planning direction is accepted on 2026-11-09. Changes to blueprint §4/§20/§24 or metadata/
publication contracts take their specific ADR/design route before dependent effects.

## Complete transition design

The native transition owner consumes exact source/target identity, target schema/module inputs,
explicit empty-state policy and the original operation. Synchronous preparation returns a
caller-held controller before any asynchronous launch. Borrowed launch/reconciliation cannot
discard the stopped source or target on cancellation. Validated readiness precedes the checked
handoff; passive provenance binds the same target and prevents duplicate target creation.
Preview remains observational and grants no later execution authority.

SurrealKit rc.1 provides frozen SQL, chain lint, ordered catch-up, tracked seeds and repair;
v4 supplies `MIGRATE EXISTING ROWS`, compatibility levels and read-time conversion with
background rewriting. Use supplier history rather than build a parallel interpretation.
Definition submission, migration completion, schema parity and consumer activation remain
different facts. [Migration adapter](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/migrate.rs),
[release notes](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md),
[field migration](https://surrealdb.com/docs/reference/query-language/statements/define/field).

For a separately selected retained-state transition, native conversion is useful where it can
express the transformation faithfully. Its
removal/restoration of blocking indexes creates a real uniqueness interval; coordinate actual
writers across that interval. A fallback value is declared domain policy. Cross-field or
restricted transformations use a bounded explicit backfill rather than coercing unsupported
logic into the native clause. Module/target partitions are useful where meanings evolve
independently, without introducing cross-database atomicity. These retained-state alternatives
are unselected for the current cutover. No frozen v11 reader, compatibility profile, backfill
executor or second production transition path is retained merely to implement an unused option.

### Execution amendment: selected acceptance

The B02/B03 retained-value conversion, duplicate-index restoration and rewrite-interruption
controls below describe alternatives from the original plan. They are superseded for this
execution by the maintainer's explicit fresh-state selection. Current acceptance is the complete
source-to-fresh-v12 physical lifecycle: nested-target/module mismatch refusal before creation,
fresh identity and credentials, absence of a source-only row in the target, cancellation after
real target launch, unknown acknowledgement, reconciliation without a second factory, checked
ready handoff, original retention until explicit disposal, and provenance after target reopen.
Native migration/cleanup/removal effects still receive bounded frozen parsing and barrier
requirements in passive preview. There is no claim that retained-data migration executes.

## Packages

### Execution checkpoint — 2026-11-09

Execution is authorized under proposed ADR-0262. The shared target is one combined
72b/72c schema/module cutover. The maintainer clarified during execution that existing
scientific results need not survive: select fresh reconstruction into v12 rather than
a retained-v11 backfill estate. The original stopped state remains owned until the
validated fresh target and actual authored-input obligations are accepted; disposal
is explicit. No arbitrary configured target has been selected for reset. Authored
source files/workspaces remain outside replacement authority. B01 exact frozen effect
preview remains observational; unselected native conversions grant no execution capability.
Plan71 is complete and its fixture/result custody is consumed. B01–B03 remain open until
the selected reconstruction, physical handoff and original-operation recovery controls pass.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| B01 | Planned | Current frozen input and preview owners; selected 72a contract as needed | Typed effect-bearing transition preview, identity and admission model ready for intended issuer; reconstruction/native/backfill choice explicit. |
| B02 | Planned | B01; selected target/source mapping from A02/A03; required decision route | Chosen transition executes under original custody with native/supplier orchestration and a coherent writer/index policy; old rewrite/history machinery removed. |
| B03 | Planned | B02 issued operation; actual affected consumers and protected readers | Completion, activation and recovery distinguish data/index/catalog facts; destructive cleanup and obsolete readers removed only at their safe cut. |

<a id="B01"></a>
### B01 — Admit exact transition intent and show its effects

Extend the frozen manifest with explicit native data-conversion/cleanup action meaning where
selected. Keep bounded files, ordered steps, original SQL/checksums and independent data-witness
requirements. Distinguish metadata-only definitions, value conversion and irreversible removal.
Bind substituted effective bytes if variables are useful; otherwise retain explicit frozen
inputs. Source hashing alone does not identify changed substituted SQL. Track seeds/modules/
targets through stable owner identity and idempotent input rather than duplicate configuration.

A preview must explain affected fields, dependent indexes/reference/view structures, expected
loss and reader interpretation, without executing or acquiring an unrelated deployment. It
chooses reconstruction where source regeneration preserves required originals with less machinery.
For retained state it names the native conversion or explicit backfill and the data witness.
Acceptance includes changed input bytes/issuer/predecessor refusal and visible destructive effects.

<a id="B02"></a>
### B02 — Execute the selected transition with one history owner

Use named migration identity for eligible stored-value conversions, supplier frozen history/
catch-up for definition progression, and native ownership for effects. Repeated identical named
application is idempotent; changed transformation under the same name must not silently rerun.
Close or appropriately fence relevant writers while uniqueness indexes are absent and retain
the source needed to reconstruct derived structures. Do not add broad persistent leases or
multi-target coordination beyond actual participants.

Expose authentic operation status if interrupted. A logical definition may be visible before
background rewrite is finished; keep `await_migrations` or equivalent native conformance
completion in the selected flow. Native field restrictions involving indexes, references,
views, inline fields and identity endpoints are meaningful; prepare/remove/recreate dependencies
through the transition owner. Use explicit backfill only for the residual unsupported conversion.

Migrate current schema-only callers to typed transition requests and delete any duplicated
local history/scan-rewrite bookkeeping actually replaced. Targeted controls cover repeated named
application, wrong predecessor, rejected fallback and duplicate data at index restoration.
These are serious corruption exposures; no separate speed-benefit pilot is required.

<a id="B03"></a>
### B03 — Complete, activate and recover without guessing

Recover the original transition on its issuer after unknown acknowledgement; metadata repair
does not authorize replay of uncertain data-changing SQL. Before target activation establish
the selected data effect, completed conformance, required ready indexes and readable interpretation.
Use independent old/new data expectations for destructive or lossy steps. Reverse definitions
do not restore discarded information; use the preserved original/reconstruction policy instead
of claiming universal rollback.

When consumers move, remove superseded interpretation adapters, obsolete fields, old fixtures
and migration code that served only a replaced mechanism. Retain historical interpretation only
for a real protected original requiring it. Logical removal and returned physical space remain
separate effects owned with 72f. Existing required protections cannot be inferred from a preview
digest or schema parity alone.

Acceptance selects one real representation transition, interruption during conversion/index
rebuild, lost acknowledgement, mismatched reader and destructive undeclared-field cleanup.
Observe exact remaining effects and recovery outcomes rather than introduce a universal migration
test campaign. Coordinate fixture/report reuse with Plan71; this companion supplies the database
effect cases, not a competing harness.

## Coverage and verification

| Review direction | Planned disposition |
| --- | --- |
| O03 native migration versus reconstruction/backfill | B01 selects by data meaning; B02 executes the complete selected route |
| Frozen history, repair, modules/targets and seeds | B01/B02 supplier orchestration with native authority, no second history owner |
| Effective variables, destructive cleanup, compatibility and completion | B01 exact effects/identity; B03 conformance and consumer activation |
| O01/O02/O14 representation prerequisites | Consume 72a's selected mappings; 72f supplies actual protected readers and storage effects |

Use scoped compile checks and targeted native fixture tests during implementation. Additional
proof is restricted to unresolved migration/corruption/performance risks; clear deletion of rewrite
or duplicated history machinery needs no benchmark. Scope-end receiving belongs to 72i.
**Proposed:** no deployment, migration, runtime check or measured benefit occurred in authoring.
