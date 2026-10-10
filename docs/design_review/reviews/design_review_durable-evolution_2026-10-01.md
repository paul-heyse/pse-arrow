---
title: Durable contract evolution decision review
date: 2026-10-01
standard: core-3.3/process-simulator-1.3
tier: design
purpose: conformance
scope: Plan 25g durable interpretation, migration and retirement contracts
baseline: 6150c8c92
decision: Accept
evidence: Proposed
disposition_owner: git:4c24721e691187e1a5b28398b29722fbde671da8:docs/plans/25-design-remediation.md
---

# Durable contract evolution decision review

**Accept the proposed design. Metadata implementation may proceed.** Recorded interpretation,
directional consumption, exact writes and explicit migration have separate authority and
effects. The proposed cutover preserves historical meaning without retaining an old execution
engine. It composes transformation with the existing native publication boundary and makes
retirement inventory survive explicit reset. Restart-safe enumeration is sufficient for the
selected discovery contract; neither an exact listing cursor nor an atomic storage snapshot is
claimed.

This is acceptance at **Proposed** contract evidence, informed by **Interface-checked** source
inspection. It does not accept the expanded implementation, close G1–G4, extend the 25e/25f
receipts, or qualify PostgreSQL/storage recovery. A baseline refinery integration defect is
recorded below; the proposed typed preflight/refusal contract supplies its correction. Local
store confinement also needs explicit enforcement because the underlying listing follows
symlinks. These obligations belong to their affected owners and do not prevent G1/G2 work.

## Scope, baseline and authorities

The independent reviewer examined the completed-25f baseline `6150c8c92`, the proposed
expansion of [ADR-0146](../../adr/0146-preserve-versioned-operational-transitions.md), and
[Plan 25g](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25g-durable-contract-evolution.md), including its execution checkpoint.
The coordinator supplied the settled capability boundaries, finite-map transformation scope,
reset transaction contract, restart-safe scan choice and subsequent descriptor-proof condition.
The source assessment uses the named baseline; concurrent operations changes appearing during
the review are excluded from implementation judgments.

The standard is Core 3.3 with process-simulator 1.3 and the pse-arrow binding. This is a bounded
design-tier **CONFORMANCE** decision review of proposed contracts and their implementation
routes, not a whole-system or scientific qualification review. Relevant existing authority is
blueprint §4.3, §5.3 and §20.2–§20.6, ADR-0114 and the proposed supplement ADR-0146.
The [series coordinator](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25-design-remediation.md) owns finding disposition;
25g owns implementation state and [25k](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25k-integrated-qualification-and-closure.md)
owns aggregate qualification.

The boundary includes `pse-schema` compatibility, semantic witnesses and migration declarations;
`pse-relations` predicates; Delta contract opening and native artifact publication in
`pse-catalog`; native schema transformation in `pse-engine`; and schema admission, migration
history and catalog retention in `pse-operations`. Representation evolution can affect artifacts
from simulation, optimization, dynamics, estimation and studies, but this review changes no
formulation, provider, solver or numerical policy. Physical dimensions, bases, reference states,
envelopes and scientific evidence remain recorded semantic inputs. No numerical stage is
introduced, so a new physical-semantics table, well-posedness analysis and model conformance run
would not establish this decision.

## Why the boundaries fit

The historical declaration and a current consumer's requirements are different facts. At the
baseline, `pse-schema::compatibility::decode` checks format, canonical serialization, roots and
digest, while `require` performs symmetric semantic equality. Delta `DeclaredCheck::open`
checks observed execution fields and encoding but reconstructs adapter expressions from
DataFusion protobuf bytes. The proposed change strengthens recorded verification and replaces
the universal equality gate with distinct products:

| Operation/product | Inputs and owned decision | Output and effects |
|---|---|---|
| `VerifiedRecordedContract` | Exact selected snapshot; canonical witness, digest, complete support closure, observed fields and supported encoding | Private checked recorded meaning and layout; no current-registry substitution and no mutation |
| `ConsumerProjection` | Verified source plus the consumer's required meaning, including consumed references and constraints | Checked read projection or typed refusal/transformation-needed result; never write authority |
| Exact write admission | Verified selected table state plus the exact expected write declaration | Mutation capability bound to that declaration and state; a compatible reader cannot mint it |
| `MigrationAdmission` | Verified source and target declarations plus explicit declared transformation | Checked transformation plan; execution and publication remain explicit |

This is an adequate model of the distinctions that govern durable evolution, rather than a
compatibility boolean with flags. A consumer can understand a recorded enum `{a,b}` after the
current registry grows to `{a,b,c}` without admitting `c` into the old table. An exhaustive old
consumer must refuse an unknown member it needs to interpret. Changed member meaning,
constraints or references require explicit rules. Deprecation can be presentation-only only
when those semantics remain unchanged. Complete witness and field verification precede every
consumer comparison, so projecting away a field cannot hide a malformed support contract.

The proposed owned `DomainEnvironment` supplies recorded domains to the same predicate
compiler used by present declarations. This removes the concrete baseline dependency in
`pse-relations::validate::predicates::field_value` on the current `Registry` enum inventory.
It does not create a counterfeit historical Registry, retain an old DataFusion interpreter, or
promote executable codec labels to meaning. Unknown portable obligations or contradictory
descriptors refuse; sufficient older witnesses use the same interpreter. The integration
owner may derive SQL and adapters, but those are checked lowerings of recorded meaning.

An artifact profile has an additional proof obligation. Baseline `ArtifactDescriptor` v2 stores
`profile_contract`, profile and requested relations, but not the profile's required-root
inventory. `semantic_profile` frames that inventory separately from the selected semantic
closure; the closure or digest alone cannot reveal which roots were required. The supplied
target therefore persists canonical required roots in the new descriptor format and admits
v2 through the same interpreter only where its baseline inventory can be independently
established and its profile digest verified. Otherwise opening returns typed missing-proof or
migration-required. This restriction is sound: guessing roots, searching for a matching digest,
using current profile expansion as historical fact, or skipping the hash would not be.
The descriptor-format admission owner must preserve the v2 preimage and frame the new format
through the existing identity owner. ADR-0146 and the 25g checkpoint now record this condition.
Only the verified versioned v2 declaration rule can project the new inventory field as null;
generic consumer admission cannot invent missing fields. This keeps format adaptation separate
from semantic compatibility and does not fabricate historical root evidence.

## Composition, transition and recovery

The baseline `pse-engine::session::schema_transform` already lowers declared add/drop/rename
and nullability operations into native plans with checked defaults and declared output fields.
The proposed structural operations, domain finite maps and composite-reference key maps extend
that responsibility. They must reject missing mappings, invalid target values, duplicate target
keys and incomplete reference closure under the target declaration. A key map binds the
composite tuple as a key; independent per-column substitutions do not establish its meaning.
Unsupported transformations remain explicit refusals rather than arbitrary executable payloads.

`ArtifactPlan::prepare_publication`, Delta candidate admission and
`pse-operations::catalog::commit` supply the existing route: immutable member writes, complete
candidate validation, then one catalog visibility transaction. Migration must carry exact source
selections and source-to-transformation-to-target lineage through that route. A failed member
write can leave unpublished destination objects requiring settlement/reclamation; it does not
alter the source or make a partial publication visible. Lost acknowledgement uses the existing
settlement contract before retry. This is a composition of existing responsibilities, not a
second publication engine. Read projection is transient and cannot be reported as a persisted
migration.

The retained PostgreSQL executor is feasible with refinery 0.9.2 over tokio-postgres 0.7.18.
Source inspection of refinery's tokio-postgres driver and asynchronous migration loop confirms
that a migration's SQL and history insertion share one transaction by default. Qualified history
names are inserted into the generated queries; their correct isolation still needs the focused
executor control specified by G3. PostgreSQL session advisory ownership spans transactions,
while enum additions need to commit before later steps use their values. The dedicated session
and existing shared generation lease address both constraints. Refinery decides execution of
ordered SQL, not semantic compatibility.

Creation, validation-only opening, planning and executing the expected migration plan remain
separate effects. Planning cannot call an API that creates a history table as a side effect.
Execution rereads and validates its plan under namespace ownership; only exact committed history
and matching intermediate layout admit resumption. The readiness marker survives interruption,
and normal admission refuses an intermediate state after the crashed session releases its lock.
Existing V1–V5 histories and historical evidence remain immutable. Reopening prepares current
statements only after final verification. F01 below is the additional preflight boundary required
by the selected library.

Reset requires stronger preservation than an ordinary migration. Its exclusive schema session
lease must cover the inventory read and completed portable export, which lives outside both the
erased namespace and member roots. The manifest includes prior unresolved retirement obligations
and live reader/export protection. Drop, creation, inventory import, ResetID/digest and readiness
commit together. Export failure leaves the existing namespace intact; database failure rolls back
the reset; uncertain acknowledgement settles the exact ResetID and manifest digest before a new
reset. This explicitly requested maintenance operation cannot become mismatch recovery.

Discovery retains one root-native unordered listing stream during a live scan and checkpoints
deduplicated candidates and report state durably. After process restart it starts a new enumeration
generation from the established root. This gives bounded live batch work without pretending that
an offset is a durable ordering guarantee. A full successful pass means enumeration completion
for that pass, not an atomic view of concurrently changing storage. An incomplete/error pass
cannot be reported complete. Unattributable objects remain unresolved and contribute to reporting.

Deletion is a separate selected reclaim operation. Its owner freshly checks attribution,
publications, intents, retained versions, reader/export protections and the maintenance epoch;
discovery evidence alone is insufficient. Native local `object_store` 0.13.2 uses
`WalkDir::follow_links(true)`, and baseline `delta::collect::remove_prefix` lists before deletion.
Consequently root containment and symlink refusal must be enforced by the local integration
before accepting candidates or removing prefixes; a URL prefix alone is not confinement.

## Revealing scenarios and alternatives

| Scenario | Expected response and change owner | Evidence |
|---|---|---|
| <a id="s01"></a>S01: extend an enum used by a recorded artifact | Recorded verification/compiler retains old domain; new consumer may read it; exact writes remain separately admitted | Proposed; baseline witness and predicate interfaces inspected |
| <a id="s02"></a>S02: upgrade DataFusion without changing portable meaning | Expression lowering/adapters absorb API changes; no old protobuf engine or silently widened domain | Proposed; current Delta byte-decoder seam inspected |
| <a id="s03"></a>S03: change a constrained field or composite referenced key | Explicit checked mapping, target closure admission and new publication with lineage; source survives failure | Proposed; native transformation and candidate/commit interfaces inspected |
| <a id="s04"></a>S04: interrupt an ordered database transition or corrupt its history | Exact committed prefix plus matching live layout can resume; malformed/drifted history refuses before readiness changes | Proposed; locked executor and refinery integration inspected; F01 |
| <a id="s05"></a>S05: reset twice with an outstanding export or unresolved orphan | Completed external manifest precedes each reset; obligations/protection survive import; ResetID settles uncertainty | Proposed; existing lease/retention responsibilities inspected |
| <a id="s06"></a>S06: interrupt a root scan, then add/remove objects before restart | New enumeration generation, deduplication and honest pass report; selected reclaim rechecks fresh protection | Proposed; native streaming interface inspected |
| <a id="s07"></a>S07: read old profile descriptor lacking required-root inventory | Proven baseline inventory plus digest admits; missing proof refuses without hash bypass or historical profile expansion | Proposed; descriptor and profile framing inspected |

The simplest viable alternative coincides with the proposed design: checked ordinary functions
and capability values, one portable semantic compiler, the existing native plan/publication
machinery and library-owned SQL migration/streaming mechanisms. Universal equality is simpler
only by refusing supported directional evolution. Loosening it globally confuses readable with
writable. Transform-on-open creates hidden effects; retaining old protobuf execution creates
parallel authority and library-version coupling. A bespoke migration runner or second driver
does not solve semantic admission. A globally sorted listing or persisted offset adds machinery
for an exact-cursor guarantee the maintainer did not select. Extra layers are unjustified unless
supported operations outgrow these contracts. Remote storage recovery and performance claims
remain outside this review.

## Architectural assessment and gates

These verdicts concern the **proposed contracts**. They do not claim the baseline already
implements them.

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 | satisfied | Meaning/admission, predicate lowering, publication, schema transition and retention have coherent existing owners; vocabulary growth does not invalidate unrelated catalog identity |
| AP-02 | satisfied | Recorded read, consumer projection, exact write and explicit migration are distinct capabilities with explicit refusal and evolution boundaries |
| AP-03 | satisfied | G4 composes checked native transformations with existing ArtifactPlan/admission/catalog commit and settlement |
| AP-04 | satisfied | Recorded revision, consumed meaning, target transformation, retirement obligation and discovery evidence are distinct; each governs its operation rather than merely annotating output |
| AP-05 | satisfied | Complete closure/field verification, exact source/target identity, readiness and maintenance protection have identified enforcement owners |
| AP-06 | satisfied | Witness/predicate/admission logic can be exercised locally; effects belong to explicit executor/publication/maintenance journeys |

| Gate | Verdict | Evidence or scope reason |
|---|---|---|
| G1 Authority | pass | Proposed checked witnesses precede consumer requirements; historical domains/profile inventories cannot be supplied by registry growth |
| G2 Fidelity | pass | Proposed projection and explicit typed mappings preserve consumed distinctions; missing historical proof is an outcome |
| G3 Validity | pass | Proposed malformed/unsupported checks, target values/keys/reference closure and readiness reject before their dependent operation; F01 remains implementation work |
| G4 Hidden behavior | pass | Proposed opening and planning are read-only; creation, migration, publication, reset and reclaim are explicit |
| G5 Recovery | pass | Proposed ready/pending/history, atomic catalog visibility, ResetID settlement and incomplete scan outcomes distinguish partial from complete |
| G6 Transformation/reuse | pass | Proposed transformations bind exact snapshots and portable meaning; no inferred migration or current-domain replacement |
| G7 Claims | pass | Wider behavior remains Proposed; preserved 25e/25f receipts keep their scope; no full database/storage qualification claimed |
| G8 Library leverage | pass | Existing DataFusion/Delta/native publication, object_store streaming and refinery/PostgreSQL mechanisms are reused; semantic admission remains project responsibility |
| G9 Fitness | pass | All applicable foundations satisfied for S01–S07 at the proposed-design level |
| PS-G1 | not applicable | No physical formulation, convention conversion or balance change; recorded physical semantics remain preservation inputs |
| PS-G2 | not applicable | No variable-role, structural-analysis or solver-admission change |
| PS-G3 | not applicable | No solve, derivative, convergence or status mapping change; unsupported historical evidence remains unavailable under PS-12 |

## Finding and preservation obligations

<a id="f01"></a>**F01 — Retained migration history parsing can bypass typed refusal.**
**Interface-checked**, baseline `6150c8c92`: `pse-operations::schema::{history_prefix,
verify_history}` read version/name/checksum with infallible `Row::get`; they do not check
`applied_on`. Refinery 0.9.2's tokio-postgres `query_applied_migrations` uses `unwrap` for its
RFC3339 timestamp and `expect` for the numeric checksum. A history row can have matching
version/name/checksum but malformed `applied_on`, pass the project's preflight, and panic when
the runner loads it after readiness has been changed to pending. Nullable/wrongly typed history
fields can likewise bypass the intended classified error path. This violates the baseline's
typed preflight/refusal boundary under DP-03, DP-15 and DP-21/G3 in S04; it does not establish
data loss or a committed partial publication.

The G3 integration owner should validate every history field the selected driver consumes,
including non-null typed shape and timestamp encoding, before any readiness/schema mutation
and before invoking refinery. Return a typed history/drift refusal; do not repair history,
replace the library, or turn malformed metadata into a retry. A control with exact checksums
and invalid timestamp/null history distinguishes the correction from a checksum-only check.
This is a baseline source defect with a correction already required by the proposed
unknown/drifted-baseline refusal contract. It does not block the accepted metadata design;
implementation closure requires evidence through 25g/25k. Current disposition belongs to the
series coordinator and G3, not this review.

Other required preservation constraints are the proof-conditioned v2/new-format descriptor
boundary, local symlink/root confinement before candidate/reclaim effects, immutable historical
frame preimages and V1–V5 SQL, retained reader/export obligations across reset, and disposal of
the replaced protobuf authority/universal admission gate without a parallel production path.
They are explicit target obligations, not claims of existing implementation or separate findings
of defects in mechanisms that have not been built.

## Evidence limits, authority route and decision

No tests, benchmarks, live database commands or storage mutations ran for this review. Static
inspection establishes feasible interface and ownership routes; named focused controls in 25g
must establish actual behavior, and 25k must exercise database upgrade/restart, publication
failure/settlement, reset protection and storage discovery/reclaim journeys. No touched unit or
property model requires a new conformance receipt here. Existing 25e/25f evidence is referenced
as historical scoped evidence and was not rerun or promoted.

Library support was checked against the pinned refinery source and current primary documentation
retrieved through Context7: [refinery runner](https://github.com/rust-db/refinery/blob/main/_autodocs/1-runner.md),
[PostgreSQL 18 advisory locks](https://www.postgresql.org/docs/18/explicit-locking.html), and
[enum transition rules](https://www.postgresql.org/docs/18/sql-altertype.html). Exact local source
decides the refinery default transaction boundary and local object_store traversal behavior;
current documentation alone does not qualify this checkout or storage backend.

Conformance requires the declared decision route: ADR-0146 supplements ADR-0114 Outcome 23;
blueprint §4.3 and §20.5/§20.6 must describe the adopted recorded/profile interpretation,
separate creation/open/planning operations and preserving reset rather than retaining conflicting
current-only/no-migration wording. Blueprint §5.3 and §20.2/§20.4 own affected identity,
publication/retention meaning. Accepted ADR-0114 remains immutable; R-35's adopted trigger follows
its register owner. These are identified authority amendments through the existing decision/design
route, not permission to amend architecture in passing. No SHOULD exception or unsupported MUST
waiver is used.

**Behavioral/semantic adequacy:** the proposed bounded operations have adequate input, authority,
effect, refusal and recovery contracts. Runtime adequacy is unqualified. **Architectural fitness:**
the proposed ownership and composition satisfy AP-01–AP-06 for the examined scenarios.
**Overall decision: Accept at Proposed evidence.** G1/G2 metadata implementation can proceed;
G3 must close F01 and G4 must enforce the stated preservation boundaries. A later implementation
that infers historical profile/domain meaning, accepts malformed history, erases protection,
reports an incomplete scan as complete, or publishes migration through another visibility path
would falsify this acceptance and require correction/review at the affected boundary.
