---
title: "25g: Durable contract evolution"
status: in-progress
date: 2026-09-30
adrs: [ADR-0146]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s05]
---

# 25g: Durable contract evolution

## Context and target

F13 combines a deliberate no-migration limit, unrelated schema-fingerprint coupling and
orphaned publication prefixes after reset. F14 makes executable predicate bytes a durable
compatibility authority. R8 requires four separate operations and preserves read-only opening.

The target preserves recorded meaning and publication/lineage identity through supported
evolution. New production execution has one path. Explicit old-content migration is a bounded
data transformation, not a retained old runtime. Ordinary opening never mutates an existing
artifact or store, and mismatch never silently resets it.

## Decisions

1. **Interpret recorded meaning:** verify format, canonicality, digest, field encoding and the
   recorded support closure, then interpret that declaration. Current registry expansion cannot
   enlarge a historical enum domain.
2. **Admit a consumer projection:** compatibility is directional and scoped to consumed meaning.
   A new consumer can read an old supported domain; an old exhaustive consumer refuses genuinely
   unknown members it must interpret. Deprecation is metadata only if meaning/constraints truly
   remain unchanged. Changed references, constraints and member semantics require explicit rules.
3. **Admit writes:** mutation requires the expected exact write declaration. Reader compatibility
   does not widen writer admission. A change of stored meaning uses explicit migration.
4. **Migrate explicitly:** a declared source/target transformation validates its output and
   publishes a new version with lineage. A requested read projection is read-only and does not
   masquerade as a stored migration.
5. **Persist portable predicates:** derive adapter checks from the verified recorded witness,
   fields and encoding contract through one predicate compiler. New artifacts contain portable
   meaning, not DataFusion protobuf expression bytes. Old artifacts with sufficient witnesses use
   this same interpreter; contradictory or inadequate witnesses refuse or require explicit
   migration. No old DataFusion execution engine or decoder is retained.
6. **Separate database identities:** catalog/control and operational schema histories cover their
   own declarations/support closures. They may remain in the same physical pse_ops schema.
   Changing queue/diagnostic vocabulary must not invalidate publication catalog identity.
   Every shared schema object has one migration owner; other histories depend on its supported
   version rather than independently managing the same object.
7. **Migration execution:** adopt refinery over existing tokio-postgres for immutable ordered SQL
   migrations. Generated target DDL remains the target declaration; migrations express transitions.
   A dedicated connection holds one deployment-wide session advisory lock for the shared
   namespace across per-migration transactions and both histories.
   Enum additions commit before later steps use the member. An explicit migrate operation
   verifies a known exact baseline, migration checksums and final target identity; unknown/drifted
   baselines refuse. Fresh creation is explicit; existing open validates read-only. Upgrades are explicitly
   quiescent, not online: drain workers/writers and close affected store generations before
   migrating. Ordinary runtime admission retains a shared schema-admission lease; migration
   acquires the exclusive lease for the whole transition sequence, not one SQL step. This
   excludes new runtime work and active writers as well as competing migrators.
8. **Retention:** preserve catalog inventory through migration. Reinitialization records retirement
   and cleanup obligations before discarding owned state. Existing orphan discovery is bounded
   to an established workspace root and reconciled against publications, intents and reader
   protection. Absence from the current catalog is not proof of safe deletion.

Refinery runs decided transitions; it does not decide semantic compatibility. A second database
driver, reset fallback, global loosened equality check and transform-on-open mutation were rejected.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="g1"></a>G1 Durable operation contracts | A1 facet purposes; I1 identity contract | Separate recorded read, consumer projection, write admission and migration | planned |
| <a id="g2"></a>G2 Portable predicate interpretation | G1; H4 | Compile recorded semantics and remove version-bound executable predicates | planned |
| <a id="g3"></a>G3 PostgreSQL identity and migrations | G1/I1; evolution ADR | Independent histories and explicit locked upgrades preserving catalog continuity | partial: required 25e/25f operational transitions |
| <a id="g4"></a>G4 Artifact transformation and orphan lifecycle | G2/G3; I1 | Explicit new-version migration, retirement inventory and bounded reconciliation | planned |

G3 precedes F4 and any changed persisted E/J contracts. It does not wait for their entire
implementations: once their declarations are settled, each owns its migration through this
mechanism. G2's comparator consumes A/H predicates rather than introducing another facet owner.

### G1 — Directional semantics

**Implementation vision.** The four operations produce different capabilities. Recorded opening consumes an exact snapshot
and stored witness, returning a verified relation identity/version, semantic support graph, field
contracts and execution layout. Consumer admission adds the requested consumed contract and
returns a projection from source fields/meanings to target slots. Writer admission produces a
mutation capability bound to the exact expected declaration and selected table state; a successful
read comparison cannot produce it. Migration admission binds source/target declarations and
returns the explicit transformation plan. Verification of stored meaning happens before any
comparison with the current registry. These are distinct products, not four flags around one
compatible boolean.

Define the four operations against verified witnesses. Include referenced contracts and encoding
layout, not only top-level enum membership. Report compatible projection, explicit transformation
needed, unsupported or malformed input with typed reasons. Keep write admission conservative.

Focused controls cover new-reader/old-artifact, old-reader/new-member, unrelated vocabulary
growth, changed constraints/references, malformed witness and independent writer refusal. Remove
the undirected shared equality gate as universal authority, preserving exact comparison where
the write contract requires it.

### G2 — Recorded-domain predicates

**Implementation vision.** The verified witness supplies domains, enum members, references and adapter requirements to the
shared predicate compiler. It returns executable checks/adapter registrations for this snapshot's
layout; session construction binds them without editing the declaration. With old enum {a,b} and
current enum {a,b,c}, the old table's reconstructed check still excludes c. A consumer may
understand the old domain without widening it. Contradictory fields, witness or encoding refuse
before rows are exposed. An obsolete executable-codec label is dispensable only when recorded
portable meaning suffices to reconstruct the same obligation.

Refactor predicate construction so both present declarations and verified recorded witnesses
supply the same semantic inputs. Reconstruct only when the witness and storage descriptor fully
establish meaning. An obsolete expression-codec label alone need not block reading, but a
contradictory descriptor is not ignored. If old content lacks required meaning, expose the explicit
migration/refusal path.

Focused controls prove the old enum domain stays restricted after registry expansion and that
opening changes neither properties nor table version. Delete predicate-byte serialization,
deserialization and executable codec identity as production semantic authority. SQL is a derived
representation when needed; no unproven unparser round-trip is required for this remedy.

### G3 — Versioned PostgreSQL transitions

**Implementation vision.** Generation supplies catalog/control and operational target contracts with support identities.
Each immutable migration declares source identity, target identity, checksum, ordered steps and
cross-history/shared-object dependencies. Planning returns the exact pending transitions without
executing them. The explicit executor locks the deployment namespace on a dedicated session,
verifies live baseline/history, executes committed steps in order, verifies the target and returns
an applied-transition report. Only committed matching history can resume after interruption.
Modified historical checksums or unexpected shared objects refuse. Creation, validation-only
opening and migration are distinct entry operations; no open-time inference chooses an upgrade.

Keep generated target schema/contract identities and immutable migration history coherent.
Use distinct qualified history table names for catalog/control and operations; verify their
configuration in a focused migration-executor test rather than assume search_path provides
isolation. Check exact source identity and reviewed checksums before applying transitions.
After interrupted work, resume only from committed history and a matching live schema,
including the shared support objects the baseline consumes. Persist migration-in-progress
before the first schema-changing step; normal open refuses intermediate states even after a
crash releases locks. Mark runtime-ready only after target verification. Reopen a fresh store
and rebuild affected prepared statements after success. Test attempted normal admission during
migration and restart from an interrupted committed intermediate state. Exercise competing migration
requests and overlapping support dependencies under the common lock.

Preserve publication IDs, lineage, attempts, leases and retention records. Additive enum changes
and destructive/transforming changes have explicit reviewed operations; no automatic schema-diff
guess changes scientific meaning. Do not move tables merely to get independent identities.

Focused controls use transition planning, checksum/baseline refusal and executor transaction
boundaries. Author real upgrade/restart fixtures with this packet and run them in K3. Delete the
single unrelated-vocabulary fingerprint and mismatch/reset normal path once consumers migrate.

Library selection is **Interface-checked**, not Tested: the
[refinery runner documentation](https://github.com/rust-db/refinery/blob/main/_autodocs/1-runner.md),
[PostgreSQL enum-change rules](https://www.postgresql.org/docs/18/sql-altertype.html) and
[session advisory-lock documentation](https://www.postgresql.org/docs/18/explicit-locking.html)
support the selected mechanism. Pin the selected dependency through the normal manifest route
when implementing; no new driver or license restriction is introduced.

### G4 — Publication migration and orphan reconciliation

**Implementation vision.** An artifact transformation binds exact source publication/member snapshots, target versions and
ordered column/domain/reference operations. Read projection exposes checked output transiently;
stored migration validates new members and commits a new publication/version through the existing
atomic boundary with source-to-target lineage. Source content remains unchanged on failure.
Orphan discovery takes an established workspace root, bounded cursor and protection snapshot,
then records candidate prefix, ownership evidence, discovery epoch, protections and disposition.
Discovery does not authorize deletion. Reclaim takes an explicitly selected attributable candidate
and rechecks protection under the maintenance epoch. Unattributable objects remain visible as
unresolved, not silently omitted from accounting.

Compile declared source-to-target table transformations through the existing catalog mechanisms.
Validate defaults, nullability, values, references and target meaning independently. Publish
through the existing atomic boundary with source/target lineage; failure leaves the source
publication intact. Retire the old schema-transform mechanism if the target replaces it.

For orphan repair, inventory workspace-owned prefixes in bounded batches, reconcile with live
publications/intents/readers under maintenance-epoch and retention rules, and record attributable
candidates before reclaiming explicitly selected ones. Preserve a cursor so scanning has bounded
work/memory and can resume. Unattributable ownership remains a reported limit, not guessed
deletion. Table vacuum cannot discover whole forgotten member prefixes.

Focused controls model live/dead/protected/unattributable objects and migration failures. K3 runs
actual storage lifecycles. Delete reset-induced forgetting and orphan-blind assumptions; keep
destructive reset separate and explicit.

## Authority and handoff

Supersede ADR-0114 Outcome 23 through ADR plus design review; update blueprint §20.5/§20.6 and
R-35 for the adopted evolution trigger. Preserve the read-only open guarantee. Hash changes are
I1's decision, facet conventions A1's and request/result meaning E/F's.

This plan supplies migration and interpretation to the series, not a second schema authority.
Stored content preservation does not retain old production code, APIs or tests for deleted
mechanisms. Independent historical fixtures remain only where they establish the supported
recorded-format contract.

## Consumed 25e prerequisite slice

**Implemented/Tested, 2026-10-01:** The required operational G3 slice splits catalog/control and operations support identities/histories, adds the registry-owned shared readiness declaration and explicit preserving transitions from the exact known 88b653d7 predecessor. Drift/checksum conflicts refuse without mutation; interrupted committed transitions resume; active and closed-but-borrowed generations prevent upgrades. No development database was reset or migrated. Wider G1/G2 directional interpretation, remaining G3 planning/report surfaces and G4 artifact/orphan lifecycle remain open.
[25e Verification](25e-declared-analyses-and-qualification.md#verification) owns commands,
conditions, composite results and limits; this does not close the companion plan.

## Consumed 25f prerequisite slice

**Implemented/Tested, 2026-10-01:** G3 adds an appended V5 transition from the frozen exact
25e operational source to occurrence-based study policy/outcome rows. It preserves historical
job/definition bytes, lifecycle and member inventory, makes unavailable historical policy explicit,
and retains independent catalog continuity without rewriting historical hashes. Strict legacy
marker codecs and frozen transition/source controls have focused evidence in
[25f Verification](25f-studies-diagnostics-and-continuation.md#verification). No live store
migration ran. G1/G2 directional interpretation, remaining G3 planning/report surfaces and G4
artifact/orphan lifecycle remain open; real PostgreSQL preservation/restart journeys remain 25k.

## Execution and evidence

The consumed 25e/25f operational slices above are **Implemented**; the remaining scope and expected benefits are **Proposed**. Packet status is planning state;
no implementation or new product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Outcome (recorded after implementation)

### What was built

Full-plan closure remains outstanding; the required 25e/25f operational transitions and their
remaining boundaries are recorded above, with focused evidence owned by the linked plans.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
