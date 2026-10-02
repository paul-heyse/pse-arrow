---
title: "25g: Durable contract evolution"
status: done
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
| <a id="g1"></a>G1 Durable operation contracts | A1 facet purposes; I1 identity contract | Separate recorded read, consumer projection, write admission and migration | done |
| <a id="g2"></a>G2 Portable predicate interpretation | G1; H4 | Compile recorded semantics and remove version-bound executable predicates | done |
| <a id="g3"></a>G3 PostgreSQL identity and migrations | G1/I1; evolution ADR | Independent histories and explicit locked upgrades preserving catalog continuity | done |
| <a id="g4"></a>G4 Artifact transformation and orphan lifecycle | G2/G3; I1 | Explicit new-version migration, retirement inventory and bounded reconciliation | done |

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

**Implemented/Tested, 2026-10-01:** The required operational G3 slice splits catalog/control and operations support identities/histories, adds the registry-owned shared readiness declaration and explicit preserving transitions from the exact known 88b653d7 predecessor. Drift/checksum conflicts refuse without mutation; interrupted committed transitions resume; active and closed-but-borrowed generations prevent upgrades. No development database was reset or migrated. At that prerequisite handoff, wider G1/G2 interpretation, remaining G3 planning/report surfaces and G4 artifact/orphan lifecycle remained open; the completed scope is recorded in the Outcome below.
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

### Execution checkpoint — 2026-10-01

Descriptor version 3 records canonical profile required roots. Version 2 is interpreted only
after establishing its baseline root inventory and recomputing the recorded digest; missing
proof refuses. Its historical identity preimage is preserved. Only that verified versioned
descriptor rule may project the new optional inventory column as null.

Execution starts from the clean completed-25f tree at `6150c8c92`. The remaining G1/G2
reader cutover, G3 creation/planning/reporting and G4 transformation/retirement/discovery
are authorized. ADR-0146 records the cross-owner target before production changes; the
bounded decision review precedes the metadata cutover.

Recorded verification and directional consumption remain separate from exact writes.
Current and recorded predicates share one compiler. Artifact migration extends the
existing checked native transformation/publication owners. The operations executor
preserves V1–V5 and appends transitions for new inventory declarations.

The maintainer selected restart-safe streaming: candidate/report checkpoints persist,
but listing starts a new generation from the established root after process restart.
No ordered-offset or atomic-snapshot guarantee is inferred. Reset exports completed
inventory first, then recreates/imports/readies the schema in one transaction; unresolved
and reader/export protection survives. Work remains read-only toward the development
database; real storage/restart journeys and aggregate qualification remain 25k.

The bounded ADR-0146 design review accepted the proposed metadata cutover before implementation.
G1–G4 now share verified recorded interpretation, explicit lifecycle products and existing native
transformation/publication owners. The review's F01 history-parser defect is corrected by exact
shape/non-null/time/checksum preflight before refinery or readiness mutation. Root integration
and an executor's read-only foreign-source check resolved current-field provenance and global
cross-workspace protection findings. The runtime could not allocate a fresh implementation-reviewer
thread; this executor check is not represented as a formal implementation review.

## Verification

**Tested, 2026-10-01:** The following isolated controls ran on local Linux from the completed-25f
baseline `6150c8c92`, in the licensed `direnv exec .` environment, using the pinned toolchain and
recipe-owned explicit `pse-relations/force-validate`. Failure target is zero. These are composite
focused receipts after repairs, not an initially clean run or integrated product qualification.

| Command (each prefixed with `direnv exec .`) | Scope and final result, baseline zero |
|---|---|
| `just unit-package pse-schema 'test(compatibility::tests) \| test(model::migration::mapping_unit::)'` | **10 passed, 0 failed**: directional enum admission, independent exact writes, consumed support/extension scope, malformed closure, historical descriptor projection and portable finite/key policies. |
| `just unit-package pse-engine 'test(provider::recorded::durability_unit)'` | **1 passed, 0 failed**: only the actual verified provider grants recorded field admission. |
| `just unit-package pse-engine 'test(session::schema_transform::mapping_unit::)'` | **6 passed, 0 failed**: old source absent from current registry, cardinality, unmatched/null policies, exact endpoints, same-version changed support, current nullability and added-column target-domain provenance. |
| `just unit-package pse-model 'test(artifact::durability_unit)'` | **2 passed, 0 failed**: descriptor root proof and unchanged historical preimage. |
| `just unit-package pse-catalog 'test(delta::contract::tests)'` | **4 passed, 0 failed**: recorded domains, sufficient historical codec-independent witness, contradictory checks and scoped current projection. |
| `just unit-package pse-catalog 'test(artifact::migration::mapping_unit::) \| test(artifact::descriptor::durability_unit)'` | **4 passed, 0 failed**: composite reference-map agreement, retained-dependent refusal, ordinary artifact refusal of synthetic Migration capability, and exact descriptor output/member inventory. |
| `just unit-package pse-catalog 'test(delta::discovery::discovery_unit)'` | **6 passed, 0 failed**: unordered retained slices/accounting, cancellation, root escape, ancestor/descendant symlink refusal before candidate or deletion effects. |
| `just unit-package pse-operations 'test(schema_unit) \| test(inventory_unit) \| test(retirement_unit) \| (test(migration_) & not test(/supported_source\|unknown_source_and_drift\|both_history_conflicts\|interrupted_committed\|committed_partial\|open_generation\|read_only_repeated\|fresh_and_upgraded\|inspection_is_read_only\|inventory_interruption/))'` | **14 passed, 0 failed**: exact frozen predecessor/immutable histories, target signatures, pending-plan identity, malformed timestamp typed refusal, manifest completion/digest/bounds/confinement and attributable/protected/unresolved policy. DB journeys excluded. |

Final compile-repair controls used the same local pinned/force-validation conditions:

| Command (prefixed with `direnv exec .`) | Final result, baseline zero |
|---|---|
| `just unit-package pse-modeling 'test(fixture_diagnostics_resolve_members_once)'` | **1 passed, 0 failed**: typed diagnostic rule plus retained member resolution and invalid fixture refusal. |
| `just unit-package pse-compiler 'test(binary_bounds_outside_unit_box_refused) \| test(integer_bounds_tightened_inward_and_recorded)'` | **2 passed, 0 failed**: typed domain refusal/tightening diagnostics retain physical bounds and observations. |

The durable-evolution focused total is **47 passed, 0 failed**. Three additional diagnostic assertion controls pass after final compilation repairs (total **50 passed, 0 failed** across the selected final controls). The expanded engine mapping command initially
had **1 failure of 6 against zero**: rebinding an already-resolved exact enum default lost its
domain declaration. Exact-target admission fixes it; the same six controls and affected catalog
controls were rerun successfully. Earlier compatibility/native compile failures were repaired
at their owners, including versioned relation IDs, recorded enum resolution, predicate SQL
normalization, canonical error conversion and query-generation bootstrap. Own-source private
visibility and cast warnings were corrected. The first final `direnv exec . just check` collected three compile errors against zero: two compiler domain controls and one modeling fixture control still compared the 25f typed diagnostic rule with a string. Those assertions now use the declared rule variants. `direnv exec . just unit-package pse-modeling 'test(fixture_diagnostics_resolve_members_once)'` also exposed one stale negative-case message (1/1 failed against zero); it now expects the existing procedure/temporal-route refusal and passes 1/1. Its scientific/member assertions remain intact. Cargo still reports the third-party
`proc-macro-error2 2.0.1` future-incompatibility warning; static zero-warning qualification remains
25k work, and no clean full lint claim is made.

**Implemented/Interface-checked:** Changed declarations regenerate Rust, Python, docs, store
DDL and operational statements. Query generation and target-signature capture use disposable
PostgreSQL databases only. Package all-target checks compile authored database and native storage
journeys without executing them. Final `direnv exec . just codegen` passed (six schema targets, isolated query generation, Ipopt bindings and hakari). Hakari reported no changes. Final `direnv exec . just check` passed: `cargo check --keep-going --workspace --all-targets --locked`, zero compile errors and zero own-source warnings. The reported third-party future-incompatibility warning remains; this is compile coverage, not test execution or lint qualification.

**not_run:** PostgreSQL preservation/restart/concurrency/rollback/lost-acknowledgement/reset-expiry
journeys, native publication migration/recovery and end-to-end discovery/reclaim journeys,
Python/native refresh, aggregate integration, formatting/static/manual gates and performance
campaigns. They remain 25k's qualification scope. No development schema was reset or migrated,
no commit/push ran, and no performance or complete storage recovery claim is made.

## Outcome (recorded after implementation)

### What was built

**Implemented/Tested, 2026-10-01:** G1–G4 deliver distinct verified recorded meaning,
directional scoped consumer projection, exact writer admission and explicit migration admission.
Recorded witnesses include complete support and observed encoding; unrelated growth cannot
invalidate scoped readers or enlarge historical domains. One predicate compiler accepts current
or verified recorded environments. The protobuf serializer/decoder and codec authority are
removed, while sufficient historical witnesses use the same production interpreter.

Descriptor 3 records canonical profile roots; descriptor 2 requires independently established
roots and its original digest. Historical identity frames remain unchanged. Portable structural,
finite domain and flat composite-key transformations use the existing checked native compiler,
validate target obligations and publish immutable members plus registry-generated lineage through
the ordinary atomic boundary. Source lease ownership spans execution/commit, and exact source
publication/transformation dependency facts prevent stale changed-map recovery. A generic product
cannot assert Migration publication capability. Read projection stays transient and read-only.

Store creation, validation-only opening, read-only planning and expected-plan execution are
separate. Frozen V1–V5 remain unchanged; catalog V2 and operations V6 append preserving inventory
support to the exact 25f predecessor. Refinery executes under the existing namespace session
lease/readiness protocol. Exact support histories, checksums, shape and timestamps refuse before
mutation; no fingerprint mismatch silently resets a store.

Reset requires a completed external versioned manifest, preserves prior unresolved obligations,
and atomically recreates/imports/records reset identity and readiness. Fresh proven retired prefixes
can become reclaimable after captured protections expire; unknown/prior unresolved inventory
remains protected. Discovery persists bounded candidates/report cursors and retains unordered
native streams between slices. A restarted process starts a new enumeration generation from the
root, retaining deduplicated inventory. Explicit selected reclaim freshly reconciles global
publication/input/window/intent/reader/export protection, holds the exclusive protection fence
through physical removal/disposition commit, and refuses local symlinks or escaped prefixes.
Lease acquisition/renewal uses the actual clock after the fence, preventing expired queued renewal
from resurrecting deleted content.

Enduring meaning is amended through ADR-0146's decision/design route in blueprint §4.3,
§20.2/§20.4/§20.5/§20.6 and the operator guide, with a blueprint revision row. ADR-0146 remains
proposed pending its decision PR; design acceptance at Proposed evidence is not production
qualification. The former R-35 trigger has been adopted. The coordinator owns resolved F13/F14
functional dispositions and the review F01 correction; integrated qualification remains open.
Required H4/I1/J slices have their owning handoff notes without closing those broader plans.
The next functional plan is 25h.

### A mistake made and corrected

Mapping initially substituted the original input field for the current ordered expression.
That rejected valid nullability changes and target-domain added columns. Foreign-source review
identified the provenance error; native expressions now supply their current fields and explicit
Recorded/Target origin selects the existing domain environment. The resulting new default control
caught incorrect rebinding of an already-resolved enum field; exact target admission fixes it.
Reference closure also now refuses a retained dependent when its owner is finitely recoded, even
if coincidental target key existence could pass ordinary referential validation.

### Deviations from the plan, deliberate

The maintainer selected restart-safe streaming over an invented durable provider cursor. A full
enumeration generation proves traversal completion, not an atomic storage snapshot; provider
internal buffering, I/O, latency, total RSS and external filesystem actors are explicit limits.
Application observations/proof pages and retained stream count are bounded/accounted. Unknown
ownership remains visible and protected. Supported key maps are flat correlated composite tuples;
nested correlated reference paths and nonprimitive Identity lowering refuse explicitly. Each column
has one composed mapping policy; overlapping rewrites refuse rather than claim unproven reference
closure. Same relation-version migrations are allowed only for changed recorded meaning with an
explicit policy; registered structural declarations still require forward versions.

The runtime thread limit prevented a fresh implementation-reviewer allocation. Root integration
used the executor's read-only foreign-owner correctness check and targeted controls; it makes no
fresh formal implementation-review claim. Broad storage/DB and static qualification is intentionally
reserved for the series-wide 25k campaign.
