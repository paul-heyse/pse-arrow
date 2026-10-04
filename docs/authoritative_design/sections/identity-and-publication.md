---
title: Identity and publication
status: current
---

# Identity and publication

This area decides what makes two things the same, what an immutable model, case, run
or publication names, and how results become durable without a reader ever observing a
mixed state. `pse-ids` is the only hasher and owns identity framing; `pse-columnar`
owns relation canonicalization; `pse-schema` owns semantic contract identity and
compatibility; `pse-runtime::workflow` owns revisions, runs and publication requests;
`pse-engine` owns the native provider hierarchy; `pse-catalog` owns Delta member I/O and
maintenance; `pse-operations` owns the operational store and the publication catalog, which
decides visibility, settlement and retention.

## 5. Identity, revisions and exact selection

Identity is a contract, not a convenience. Every identity is produced from declared
meaning through a versioned projection, never from row position, display names,
floating-point accidents or whole-object serialization. A different meaning gets a new
projection or context string; an old one is never reinterpreted. The rationale for
versioned projections is [ADR-0089](../../adr/0089-semantic-identity-projections.md).

### 5.1 Forms of identity

> Decision: [ADR-0115](../../adr/0115-registry-typed-identities-and-vocabularies.md) — entity identities are declared on registry key columns, inherited by
> foreign keys, and generated as typed ids; bytes and minting are unchanged (Plan 22 B1, B3,
> B7, implemented). As built, modeling roots, definitions and members are declarations and
> blocks are instances, so there is no separate root, definition, member or block identity,
> and the source-bundle identity wraps a `ContentHash` rather than a `SemanticId` (refining
> ADR-0115 Outcomes 1 and 2).

| Form | Representation | Assigned by | Scope |
|---|---|---|---|
| Semantic ID | `pse.semantic_id`, 128-bit `pse_ids::SemanticId` | authoring (authored entities), keyed derivation (derived entities), UUIDv7 minted by the runtime (runs, attempts, jobs, studies, publications, settlements, reader leases) | survives revisions, reordering, re-batching, projection and publication |
| Artifact-local position | integer ordinal or coordinate index | the preparation that built the artifact | only within that artifact |
| Content hash | `pse.content_hash`, 256-bit BLAKE3 `pse_ids::ContentHash` | a named projection or canonical preimage ([§5.3](#section-5-3)) | identifies one immutable meaning under its projection version |

**Authored entities.** Identity is assigned at creation and stored with the entity; it is
never computed from the entity's name, so a rename changes a label and nothing else. The
package declares its policy (`IdPolicy`, the `id_policy` of `authored.packages`):

- `explicit` (default): the entity carries a UUIDv7. A missing ID is
  `authoring.parse.missing_id`; the document editor's `assign_ids`
  (`pse-runtime::authoring_driver::document`) inserts missing IDs as source edits.
- `named`: for reference packages whose qualified names are the public contract (units,
  constants and other named physical reference declarations). `pse_ids::named_id` derives the ID from
  package ID and qualified name under the frozen `pse:named:v1` context. A rename is a new
  entity by construction, so renaming such an entity is refused
  (`authoring.reference.rename_named`).

References between entities (template submodels, connections, case targets) are stored
by identity, so no authored fact depends on a name. Resolution lives in
`pse-authoring::ids`.

**Derived entities.** A derived ID is a keyed digest of what the entity was created from:
`FramedHasher::new(frame)`, under a frame of the catalog ([§5.3](#section-5-3)), over the
parent identities and index members, finished as the
first 16 bytes of BLAKE3's extendable output. Examples are port identity
(`named_id(instance, "port:<name>")`) and scalar occurrence identity (instance,
declaration and index members, in `pse-modeling::specialize::member_id`). Unchanged inputs
reproduce identical IDs; changing an index set changes only the IDs that depend on it.

**Positions.** Native layouts assign coordinate order, sparsity positions and block
indices. These are artifact-local. Coordinate order and sparsity belong to the native
layout's identity; cross-artifact references always use semantic IDs.

**Row tokens.** `pse-relations::identity` derives a versioned row token from relation
identity, key-contract version and the ordered declared primary-key values. Mutable
payload and Delta version do not enter it. A token names a row within an exact selection;
it is not a membership proof, and actual key columns remain authoritative.

**Typed identities.** A registry key column may declare an entity identity
(`declare_identity`, `FieldContract::with_identity`). A foreign-key column inherits the
identity of the column it references, and assembly refuses a conflicting declaration.
Ownership is declared separately (`FieldContract::with_owned_identity`): an owner is its
relation's single-column primary key, each identity has at most one, and a column that only
carries an identity never owns it (the export manifest's key carries `publication`, which the
catalog's intents own). `reference.schema_identities` describes the declarations. The
generator emits one typed id per identity into `pse-model` through the single
`semantic_id_newtype!` macro of `pse-ids`, which the `pse-quantity` physical-registry ids
also use. A typed id is a transparent wrapper with `From` in both directions and
`Display`; its serde form and framing are those of the wrapped value, so no identity byte
moved. Generated rows, the operational store's identity domains
([§20.6](#section-20-6)) and the Python `NewType`s of `pse.contracts.identities` follow. The
operational and publication identities are run, attempt, job, study, solution, settlement,
reader lease, workspace, publication and source bundle; the modeling identities include
package, declaration, instance and fit. There is no separate model or case identity: a
model is named by the root declaration it specializes and a case by its case declaration,
both as `declaration` ([§20.3](#section-20-3)). Consumers take the typed ids, so passing one
identity where another belongs fails to compile; `compile_fail` doctests pin
representative swaps.

`pse-ids` is the sole direct `blake3` dependent (`tests/governance/tests/blake3_owner.rs`);
other crates hash only through its framing APIs. Golden vectors in
`crates/pse-ids/tests/golden_vectors.rs` freeze the frame spellings and digests.

### 5.2 Model revisions, cases, runs and attempts

Definitions, instance bindings, case values, analysis requests, resolved numerical
policy, prepared artifacts, used starts, results and publications have distinct
identities and lifecycles. Cases and results never mutate the model
([ADR-0114](../../adr/0114-typed-operational-store.md)).

**Package revision.** Runtime modeling admission seals the exact package closure,
visibility, declarations, physical context and the physical name bindings it resolved into
an immutable compiler `ModelingRevision`, identified by its source revision
([§5.3](#section-5-3)). A checked revision is not interchangeable with arbitrary raw rows or
a caller-supplied physical context. Immutable edits re-admit the changed revision; failure
leaves prior packages intact. Selected root/instance, bindings, analysis route and limits
are explicit tracked requests. Proposed
[ADR-0099](../../adr/0099-modeling-language-and-identities.md) records this refinement.

**Case.** Specialization derives semantic member identities from original declarations,
instance paths and coordinates. Case bindings supply values, fixed/free status, bounds and
requests. Native coordinate ordinals remain local to the prepared layout. Body, case,
request, physical inventory and resolved numerical identities retain their separate named
projections; compiler reuse depends on admitted meaning, not on an old builder revision
hash. Diagnostic spans can refresh without changing mathematics
([§14](mathematics-and-compilation.md#section-14)).

**Identity scopes.** Each scope has its own versioned projection:

| Scope | Includes | Excludes |
|---|---|---|
| Definition body | typed expressions, formals, guards, physical and provider contracts | diagnostic definition IDs, source spans, documentation |
| Instance / revision | bindings and topology | display names, source prose |
| Case | roles, values, bounds, parameters, guesses | unselected stored cases |
| Analysis / numerical policy | outputs, mode, derivatives, effective coordinates, budgets, source provenance | adapter defaults not selected by the resolved policy |
| Prepared artifact | consumed specialization constants, profile, actual library environment, source and build identity | mutable native state |
| Native layout | coordinate order, sparsity, compatibility | coefficient values |
| Used start | the submitted seed by content, predecessor attempt, partial start and reuse of retained native state, framed into the completed step's request identity ([§16.5](numerical-execution.md#section-16-5)) | the run and attempt that produced the seed |
| Result, publication | separate scopes of their own | the request identity |

**Run and attempt.** Starting a prepared case mints a fresh run ID (UUIDv7). The run is
its own identity: an ephemeral run has no attempt, and a durable run's tries are its
attempts, so a retried job's attempts share its run ID. Result rows name the run; the store
and the publication name the attempt. The run ID is not part of request identity, so repeating the same request with
the same start is a new run with the same request and preparation identities
(`runtime.run_lineage`), while a different start is a different request identity
([§20.3](#section-20-3)). A publication has its own publication ID; the attempt it names is
the durable attempt of the run it publishes ([§20.2](#section-20-2)).

### 5.3 Canonical framing and hashing

> Decision: [ADR-0155](../../adr/0155-numerical-derived-families.md) and
> [ADR-0154](../../adr/0154-declared-numerical-strategy.md) (proposed target).

Current declared numerical strategy, derived-family structure/binding and retained products
use separately versioned frames over their consumed dependencies. Semantic proposal transport
and native payload compatibility are distinct. New current frames do not change historical
bytes or grant current reuse from readable historical provenance. Generated strategy/result
changes use the existing directional compatibility and explicit preserving migration owners.


> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

Demand-indexed support/assessment products frame their consumed semantic, physical, provider, ordered selection, policy and relevant build/runtime contracts. Existing role separation remains: changing current preimages versions those frames and never reinterprets historical bytes. Runtime observation does not invalidate mathematics that did not consume it.

> Decision: [ADR-0115](../../adr/0115-registry-typed-identities-and-vocabularies.md) — every frame context is declared once in the `Frame` catalog of
> `pse-ids`, with spellings and golden vectors unchanged (Plan 22 B3, implemented);
> [ADR-0116](../../adr/0116-typed-boundary-documents.md) — a document's request identity is
> framed from its typed value, never from order-preserving JSON text (Plan 22 B5,
> implemented). As built, settings identity no longer frames Rust type names, so six frames
> moved to new versions; this refines ADR-0116's expectation that settings identity stays
> unchanged (below).
>
> Decision: [ADR-0123](../../adr/0123-typed-package-schema.md) — a modeling source revision
> is framed over its structured rows, its physical-inventory identity and its physical name
> bindings, and every frame whose preimage changes takes a new variant (Outcome 8; Plan 23
> KR3 and KR8, implemented).

`pse-ids` defines two frozen framings, and the difference between them is contract:

- **Keyed derivation** (`derive_id`, `derive_hash`, `FramedHasher`): BLAKE3
  `derive_key(context)`; every part, integers included, is a u64 little-endian length
  followed by its bytes. Used for entity IDs and every named projection digest.
- **Canonical preimage** (`FrameSink`): unkeyed; variable-length components carry a
  u64 length, fixed-width components (IDs, hashes, counts, schema versions) carry their
  declared width and no length. Used for relation content hashing.

**Frame catalog.** Every keyed context is a variant of `pse_ids::Frame`, declared once with
its exact spelling, area and meaning. `FramedHasher::new`, `derive_id`, `derive_hash`, the
keyed preimage entry points and the document serializer `pse_ids::document::of` take a `Frame`, so no
context is a literal at a call site. A new meaning or derivation version is a new variant,
never a new spelling for an existing one. Spellings are unique (`frame_spellings_unique`)
and unchanged against the list captured when the catalog replaced the literals, 104
production contexts (`frame_spellings_unchanged`); the catalog grows only by deliberate new
variants. The [generated frame reference](../../generated/frames.md) lists it by area.

**Document identity.** `pse_ids::document` frames Rust-owned typed values, field names and
variant spellings without Rust type names. NaN payloads canonicalize; signed zero and
infinities stay distinct. Key-order-independent serialization belongs to this one owner.
Changed unrestricted-float/settings/warm-start/profile preimages take new named frame
versions; finite admitted domains whose preimages remain unchanged keep their frames.
Historical frame spellings and recorded digests are immutable facts.

Source revision, admitted closure, semantic body, prepared view, binding, profile, lineage
request and operational job hashes are distinct types. `.as_id()` is the explicit lowering
at a stored/raw transport boundary. `RecordedOperationalJobIdentity` retains a digest and
its optional recorded frame; unknown/older proof cannot satisfy a current operational key.
The preserving operations V7 migration renames `request_identity` to
`operational_job_identity`, retains every historical byte and adds nullable
`operational_job_frame`. New attempts record `DurableJobRequestV3`; history lacking frame
proof stays unknown rather than receiving an invented version. Queued job identity includes its
explicit idempotency scope; direct attempts include run and attempt identities; study jobs include
their declared occurrence. V7 also normalizes only the six independently verified PostgreSQL 18
NOT NULL constraint names suffixed by the immutable V5 table recreation. Historical SQL
checksums remain unchanged, and current layouts require the canonical names.

> Supplement: [ADR-0150](../../adr/0150-checked-admission-and-owned-reuse.md)
> (proposed; authorized implementation).

**Admitted study binding identity.** ADR-0148 (proposed; authorized Plan 25f implementation)
adds `pse.study.binding.v1`. It frames physical context and ordered member identities, expected
quantity identity, parameter role and exact finite canonical bits. Attribution paths and supplied
units do not participate; the selected source revision is retained separately for admission and
replay validation. Compatible explicit-ID renames can retain content identity; changed meaning
or named-policy identity cannot. Occurrence, run and attempt identity remain distinct from binding
content. Conflicting duplicate assignments, including different signed-zero bits, refuse before
identity is derived.

**Modeling source identity.** A modeling source revision's identity
(`pse-runtime::math::modeling::source_revision`, frame `pse.modeling.source-revision.v4`)
frames the structured declaration rows in order, each binary document identity and
content hash, and the identity of the physical inventory they are admitted against (`pse.math.physical-inventory.v6`,
[§8](physical-semantics.md#section-8)) and the physical name bindings admission resolved
them with, each name with the identity it denotes. Binding a name to another type, admitting
the same rows against another inventory, or changing a row is another revision; unchanged
inputs reproduce it. The frames whose preimages render canonical DSL spellings, whose unit
literals became canonical unit products ([§8.2](physical-semantics.md#section-8-2)), moved
to new variants, each replacing its predecessor: `pse.modeling.dispatch-body.v2`,
`pse.modeling.finite-function.v2`, `pse.modeling.continuity.v2`,
`pse.modeling.definite-integral.v2`, `pse.modeling.consumer-body.v2`,
`pse.modeling.implicit-residual.v2` and `pse.math.typed-definition.v3` (DP-24). A unit
product's identity is the new frame `pse.quantity.unit-product.v1`. *Tested* by
`source_revision_changes_with_a_physical_name_binding` and
`unchanged_inputs_reproduce_the_source_revision` (runtime units), and by the frozen vectors
`the_structured_ir_frame_variants_are_frozen` and `the_unit_product_identity_is_frozen`
(`pse-ids`).

**Physical and field-purpose identities.** The current physical inventory preimage is
`pse.math.physical-inventory.v6`, including declared kind equivalences and complete operation
contracts. Checked function occurrences retain selected admissions; composed unit scales,
actual binders, map/slot identity, reconstruction normalization, scientific references and
transfer/reference operations enter the typed-body product. Its frame is
`pse.math.typed-definition.v7`; concrete resolved admissions use
`pse.math.resolved-admissions.v2`. A changed preimage takes a new declared variant; old
recorded digests are never reinterpreted.

A field identity first selects its schema-declared purpose (§4.4), then frames that
projection. Execution, value and logical type identity remain distinct; root normalization
and recursive metadata omission occur only where the selected purpose permits them.
Unknown metadata is retained. Transfer owner/coordinate/direction context is semantic
and survives identity projection. These field projections establish equivalence for their
purpose, not physical compatibility or directional target admission. Source revision,
relation content and typed execution products keep their separate owners.

> Decision: [ADR-0136](../../adr/0136-declared-field-facets.md) (proposed) — field comparison
> purposes are declared once and consumed at their actual identity boundaries;
> [ADR-0135](../../adr/0135-physical-expression-contracts.md) (proposed) — admitted physical meaning
> and scientific reconstruction dependencies enter preparation identity.

**Floating-point rule** ([ADR-0030](../../adr/0030-canonical-float-hashing-and-no-float-keys.md)).
`canonical_f64_bits`/`canonical_f32_bits` map every NaN to the positive quiet NaN with
zero payload and preserve `-0.0`. They apply to hashing copies only; values sent to a
solver keep their payload. All floating framing and reuse equality use the same
function, so signed zero distinguishes identities and incremental reuse agrees with
clean preparation. Authored numerical inputs must be finite
(`authoring.parse.nonfinite_number`). DataFusion grouping, `DISTINCT` and hash joins
merge `-0.0` with `+0.0` and treat NaN as self-equal; therefore registry admission
refuses floating-point and quantity primary-key columns (`pse-schema::checks`), and
analytics that group on floats accept that collapse
([§19](workflows-and-results.md#section-19)).

**Relation content (`pse.canon.v2`).** `pse-columnar::canon` computes the logical
identity of a complete relation under its declared contract:

1. Admit the batch schema exactly, including metadata; undeclared contextual keys are
   refused. Sort unique primary keys with explicit nulls-first order; a null or
   duplicate key is invalid. Batch boundaries and dictionary codes carry no identity.
2. Build a normalized hashing copy: decode dictionaries after domain checks; zero hidden
   null payload recursively through nested and masked children; omit an all-valid
   bitmap; drop unreferenced capacity. Unknown layouts are refused.
3. Carry declared semantic metadata as a separate sorted `(field_path, key, value)`
   relation.
4. Write metadata and data as two separately framed, finished Arrow IPC streams
   (metadata V5, 64-byte alignment, uncompressed), preceded by the version string,
   relation ID, schema version and registry fingerprint. Apply the floating-point rule.
5. `logical_hash` is BLAKE3 of that preimage. `encoding_checksum` is BLAKE3 of one
   stored object's finished bytes. The `LogicalHash` and `EncodingChecksum` role types
   prevent either standing in for the other.

Construction limits are explicit configurable reservations checked before allocation;
exhaustion is `runtime.resource_limit`, never a partial result. Raising a limit never
changes identity bytes. Changing framing or the one-batch representation requires a new
version string. Equal declared content hashes identically across batch layout, hidden
null payload, dictionary encoding and supported file round trips; property and
metamorphic tests live in `crates/pse-columnar/tests` and `tests/conformance`.

A hash identifies input; it does not establish admission, validity, applicability or
equivalence. Keys, references and domain invariants are established by the relation and
catalog boundary before publication or reuse.

**Limits.** `pse.canon.v2` is implemented and tested, but no current production
publication or reuse path consumes relation logical hashes: runtime identities use the
named projections in [§5.2](#section-5-2), and publications name exact Delta versions
([§20.2](#section-20-2)). The unkeyed frame reserves `pse.snapshot.v2` for an aggregate
snapshot identity; no implementation computes it. Source documents are identified by
the encoding checksum of their text.

### 5.4 Exact relation selection and the provider catalog

`pse-engine::provider` binds relations into a native DataFusion catalog/schema/table
hierarchy. Names resolve once per operation to exact relation versions, roles and
provider owners; aliases do not float. Remote metadata is resolved explicitly into
retained provider generations before cheap synchronous lookup; backend failure is an
error, not absence. Each bound source carries a typed witness
(`pse-engine::provider::witness`) recording the exact selection the provider owner
supplied, so a completed plan can report the exact inputs it consumed, including empty
ones (`pse-catalog::selection::selected_dependencies`).

Invariants:

- Admission checks run at every platform logical-plan boundary, including subqueries
  (`pse-engine::session::admission`). Metadata lookup never executes a producer.
- Pushdown claims (Exact, Inexact, Unsupported) preserve complete values and
  multiplicities; `tests/engine/tests/pushdown_vs_unpruned.rs` compares each provider
  with its unpruned equivalent. Statistics are published only when established for the
  selected scan; unknown remains unknown.
- Unpublished candidates are bound through checked candidate sessions
  (`pse-runtime::session_factory`). They carry no publication identity and expose no
  unproved key or uniqueness constraints. Only complete admitted results cross
  publication.
- Every registered relation declares one snapshot class (`model`, `case`, `derived` or
  `sidecar`); registry admission refuses a missing class
  (`schema.missing_snapshot_class`).
- Plan encodings and `EXPLAIN` output are noncanonical diagnostic evidence, never relation
  identity ([ADR-0044](../../adr/0044-noncanonical-plan-evidence.md)).

A publication is opened from its catalog record, not assembled from independent
latest-table reads: the catalog grants the complete record under a reader lease
([§20.4](#section-20-4)), and `pse-catalog::delta::publication::Publication::open` binds
exactly its member versions; payloads stay lazy. Delta and Arrow streams are the hot path;
IPC remains an encoding and spill boundary. In a durable runtime's query sessions the
operational store's relations join the same hierarchy as read-only providers under
`pse_ops` ([§20.6](#section-20-6)).

## 20. Persistence, publication and reproducibility

Durability is an explicit effect, and visibility is decided by one conditional commit.
Execution stays in memory until a caller asks to publish; publication never re-runs
science, and a scientifically failed attempt can still be published faithfully. PostgreSQL
owns what changes and Delta owns what is published ([D10](architecture-overview.md#section-d10)):
a durable attempt's immutable result tables are Delta member tables, and one transaction of
the operational store's publication catalog makes a publication of them visible. The
protocol was decided in ADR-0091 and then ADR-0112; [ADR-0114](../../adr/0114-typed-operational-store.md)
restates it with the visibility boundary in the catalog. `pse-operations` owns the catalog
statements, `pse-catalog` performs Delta member I/O and never sees PostgreSQL, and
`pse-runtime::workflow::{publication, reading, retention}` composes the two. The
operational store itself is [§20.6](#section-20-6).

### 20.1 Delta durable relations

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) — the publication record
> lives in the PostgreSQL catalog ([§20.2](#section-20-2)); Delta keeps immutable member data
> and export manifests (Plan 22 O8, implemented).

Delta tables persist authored sources and declarations, published results, provenance and
export manifests. Native Delta and DataFusion operations own reads, writes, DML
(`pse-catalog::delta::dml`), schema transformation, change data and maintenance. Arrow
buffers and streams serve transient execution; no intermediate is forced to commit. There
is no separate content-addressed store or publication journal: the catalog's publication
record is the only publication record. A root that still holds the former Delta control
relation (`runtime.publications`) is an unsupported historical format: registering it as a
workspace root is refused as migration-required (`LegacyWorkspace`), naming regeneration by
rerun into a new root, and an export reader refuses a former control table the same way.

Each relation's registry declaration is lowered to native Delta `CHECK` constraints and
identity table properties (`pse-catalog::delta::contract`). Declared Arrow types that
Delta cannot store directly use inspectable lossless layout projections
(`pse-catalog::delta::layout`). Every Delta command receives the actual invocation
`SessionState`; the validating write route preserves local constraints and commit
properties, and raw provider mutation cannot bypass it. Candidate-wide key, reference,
completeness and domain checks run against exact candidate versions
(`pse-catalog::delta::admission`).

### 20.2 Exact coherent publication

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) — one catalog transaction
> with a compare-and-set on the workspace head is the visibility boundary; settlement
> queries the catalog (Plan 22 O8, implemented). As built, the publication attempt is the
> run's durable attempt, and a publication intent is registered before the first member
> write, refining the pre-effect ticket of ADR-0114 Outcome 2.

Publication happens in a registered **workspace**: a named history with one head and the
root URI its members are written under (`Runtime::register_workspace`;
`pse_ops.workspaces`, `publication_heads`). Only a durable runtime publishes
([§20.6](#section-20-6)); an ephemeral run is refused with `EphemeralPublication`. A caller
publishes a completed `RunResult` in two steps (`pse-runtime::workflow::publication`):

1. `RunResult::prepare_publication` names the workspace, the exact expected parent (never
   rebased) and a publication ID, minted when none is given. The attempt it publishes is
   the run's durable attempt. It plans every member under the intent's prefix
   `{root}members/{attempt}/{publication}/<schema>/<table>/` and mints a serializable
   `PublicationTicket`. It performs no write.
2. `PublicationAttempt::commit` registers the **publication intent** (`publication_intents`:
   the publication, workspace, durable attempt and member prefix) before the first member
   write, then executes the candidate: members are written to isolated immutable Delta
   tables, and `pse-catalog` admits the complete record exactly as a reader will open it and
   returns it with every member's actual version (`runtime.publication_manifests`). One
   catalog transaction then inserts the publication, its members, inputs and change
   windows and advances the head, conditional on the head still being the expected parent.

The commit takes the shared catalog protection fence, then the workspace maintenance fence, before its row locks (the attempt, the intent, every live publication a
retained member or input selects, then the head), and it refuses a retained member or input
that no live publication still protects. The record names every member's relation,
contract fingerprint, table URI and exact Delta version, and the exact inputs consumed;
each is one registry `MemberDescriptor`, a named structure emitted once for every relation
that carries it. Readers resolve a publication ID or a workspace head through the catalog;
there is no latest alias. An artifact descriptor (`pse-model::artifact`) records the
publication profile's semantic contract, requested relations, release members,
source/build and algorithm/target identities and value assumptions; required relations
appear even when empty (`reference.artifact_profiles`).

Outcomes are typed (`pse_operations::OperationsError`, `WorkflowError`):

| Outcome | Meaning |
|---|---|
| committed (`Published`) | the publication is the workspace head and names exactly this attempt's members |
| `PublicationConflict` | the head moved; re-prepare against the new head with the same publication ID, which recovers the members already written without rewriting them |
| `PublicationIdentityReused` | the publication or attempt identity is already committed with a different request, such as an attempt published as another publication |
| `PublicationUnresolved` | the catalog could not confirm the commit (a lost acknowledgement, or an unreachable store); never treated as rollback |

Settlement (`Runtime::settle_publication`) queries the catalog for the ticket's attempt and
writes only its own `settlements` row. It returns `Committed`; `ProvedNoncommit` when no
intent is registered, the intent is abandoned, or the head is still the parent while the
intent is locked, so no commit is in flight; `Conflict`, with its reason and the head it
met; or `Unresolved` while the catalog is unreachable. It never writes Delta, executes
producers or calls a solver, and an uncertain outcome is never permission to
rematerialize an attempt. Member receipts are keyed by the attempt
(`pse.member_attempt.v4`; a v3 receipt is refused as migration-required), so a
re-preparation recovers the members already written. Delta `SetTransaction` is recorded as
a durable witness; at the pinned version it does not deduplicate replay, and an attempt
label alone is never authority (`pse-catalog::delta::attempt`).

Member writes can remain after a failed or conflicting attempt without becoming visible;
an intent that can never commit is reclaimable ([§20.4](#section-20-4)). A failed attempt
never poisons the logical base. Concurrent publishers in two processes, lost
acknowledgements and repeated settlement are exercised by the `pse-runtime`
`publication_catalog` journeys and the `pse-operations` catalog tests.

### 20.3 What a run references

A run records the exact identities it consumed, not names:

- `runtime.run_lineage`: the model, case, instance and fit a step solved, its revision
  identity, and the request, preparation, profile, numerical, physical and environment
  identities of every step. The model ID is the root declaration the model specializes; the
  case ID is that root when it is a case or a test (a case with an oracle); the instance ID
  is what the root became (the experiment's instance for a fit experiment). A fit names its
  fit ID, plus the model and case only when all its experiments share them. Every producer
  derives these through `pse_model::lineage`, so one model has one ID everywhere, and
  `runtime.solve_runs` and `runtime.numerical_requirements` carry the same columns.
  An algebraic step's request identity (`pse.completed.request.v2`) also frames the start
  it actually used and whether it reused retained native state
  ([§16.5](numerical-execution.md#section-16-5)).
- `runtime.solve_metrics`: effective backend options, the submitted start and native
  observations; unavailable metrics carry a typed reason, never a synthesized zero.
- `runtime.computation_runs`: the joined job's kind, source and profile identities,
  state, native termination, qualification and candidate facts.
- The catalog's publication record, and an export manifest's copy of it: exact input and
  member selections.

The start a step received is the JSON text metric `start`/`request` in
`runtime.solve_metrics`; the seed a step offers for later use is `start`/`available`.
Python's `start_json` returns the same document (`StartReceipt::snapshot`,
[§17.6](numerical-execution.md#section-17-6)):

| Field | Content |
|---|---|
| `previous_attempt` | The predecessor attempt whose accepted result seeded the step under `PreviousAccepted`, or null |
| `seed` | The submitted seed, or null: `origin` (run and attempt, or null), the `layout`, `profile` and `data` stamps as hex, `backend` by its registry spelling, and a `payload` tagged by `kind` (`root`, `nlp`, `highs` or `pounce_sqp`) |
| `sparse_seed` | An explicit partial MIP seed keyed by semantic ID, or null |
| `transformations` | The typed path the seed took, externally tagged: `{"normalization": <hash>}`, then `{"presolve": {"transformation": <hash>, "passes": [...]}}` when the library applied a pass, with hashes in their `blake3:` form and passes by their snake-case names; empty when neither a seed nor a partial seed was supplied |
| `submitted` | Whether the native API received the seed |

Provider parameter data are retained with the immutable revision. Provenance and
diagnostics are typed relations queryable through the same provider hierarchy. Names,
metadata and hashes alone never certify validity, execution or equivalence. Result
meaning is owned by [§19](workflows-and-results.md#section-19).

### 20.4 Reproduction, reuse and retention

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) — catalog reader leases,
> change windows and two-phase deletion replace the local lock files (Plan 22 O8,
> implemented; findings T02 and T16 of the Plan 22 target review). The protocol is
> exercised locally; remote object stores are deferred to register R-37.

Three lifetimes are independent. Invalidating one never silently changes another.

| Lifetime | Owner | Governs | Never |
|---|---|---|---|
| Semantic reuse | `pse-compiler::workspace` (Salsa) | whether prepared mathematics can be reused; keys include complete inputs, absence states, math environment, source and build identity | persisted; evicted by storage maintenance |
| Retained runtime artifacts | `pse-runtime` prepared products and native sessions | memory held by immutable programs and native workspaces; shared immutable products, attempt-private mutable state | treated as program validity; retained past the last owner |
| Storage snapshots | the publication catalog (`pse-operations::catalog`), executed by `pse-catalog::delta::collect` | which Delta versions remain reopenable | removed while a live publication selects them, a live intent may still commit them, or a live reader lease protects them |

Semantic reuse compares complete inputs; conservative recomputation is always valid. An
explicit program clear fences late insertion; a retiring in-flight preparation remains
owned until completion and is distinct from a new caller's cancellation. Salsa rotation
follows retained entries and bytes. Salsa databases and handles are never persisted.
Fixed symbol registration is a determinism control, not a claim of bitwise
reproducibility across environments.

**Reproduction.** Reopen exact versions through the catalog (`Runtime::open`, `open_head`);
derived layouts are regenerated as needed. Bounded change data over an exact selection
(`pse-catalog::delta::changes`) supports change impact; a missing log entry or changed
declaration refuses the window rather than returning an empty change set. Change data is
neither permanent audit history nor automatic compiler incrementality.

**Readers.** A reader takes a `reader_leases` row in one short transaction, which returns
the complete publication record and the workspace's maintenance epoch; it then reads Delta
files holding no database session. A `ReaderLeaseGuard` renews the lease in short
transactions at a third of its lifetime while any owner holds it (the publication's
session, or a stream of it), releases it when the last owner drops, and cancels the reader
when a renewal finds the lease lapsed. Acquisition and renewal compute expiry with the actual clock after obtaining the protection fence; a queued transaction cannot renew an already lapsed lease using its earlier transaction start time. The pair (workspace, maintenance epoch) is the
session's `ReadScope` (`pse-catalog::delta::scope`), the lookup input of every shared
snapshot, resident and file-metadata cache; a session without one bypasses those caches.
An **export** is a lease held by `export:<destination>` for a stated time:
`export_publication` writes the record, the lease, its expiry, the epoch and the store
fingerprint as a one-row `runtime.publication_manifests` Delta table, which `open_export`
reads without the store, refusing an expired export or a former control table.

**Reader caches.** A Delta scan's Parquet predicate-cache allowance (an active and a
prefetched file per native reader, sized by `max_predicate_cache_size`) is admitted against
the native cache service's aggregate allowance and the memory pool once per execution, for
the actual reader partitions: the output partitions of the scan's leaf sources, however
many output partitions the operators above them fan out to. The first executed partition
admits it, the others share it until the last stream drops, and another execution admits
its own. The resident cache reuses a decoded exact selection while its input is the one it
was planned with, or an equivalent rebuild of it: the same native leaves and, above them,
the same operators by name, one-line rendering and schema, as when the physical optimizer
replaces a round-robin repartition with an identical one. A rebound source or an altered
operator forfeits reuse. *Tested* by
`reader_lease_admits_actual_reader_partitions_once_per_execution` (engine units) and
`resident_reuse_survives_an_equivalent_rebuild_only` (catalog units).

**Retention.** The catalog computes what stays reachable, in SQL, for three reasons
(`RetentionReason`): the exact versions a live publication selects (`publication`), the
member prefixes of live intents (`attempt`), and the change windows a live publication read
(`changes`, recorded in `publication_windows`). A publication is live until it is marked
(`retention_marks`), and an expiring one stays protected while a live lease reads it.
Maintainers of a workspace serialize on a transaction-scoped advisory lock and advance the
workspace's `maintenance_epoch` before any effect. Maintenance is an explicit request
(`Runtime::retire_publications`, `collect`, `reclaim_unpublished`; the `pse-publication`
binary), and every step is idempotent, so an interrupted run completes on rerun:

- **Retire**: mark a publication `expiring` (never the head), wait until its leases are
  released or expired, remove the tables only it selects (its outputs, and inputs whose
  writer is already deleted), then mark it `deleted`.
- **Collect**: fix every protected range, then verify that each protected version opens,
  commit a fence, checkpoint and vacuum each selected table keeping those versions. A
  protected version whose history is gone refuses collection of that table.
- **Reclaim**: abandon every unpublished intent that can never commit (abandoned, its
  attempt stale or superseded, or published as another publication), remove its member
  prefix, then mark it reclaimed.

**Retirement and orphan inventory.** Explicit schema reset first exports a completed
versioned manifest outside member roots under the exclusive namespace lease, retaining
prior unresolved records, original publications/members/inputs/windows/intents/retention,
and reader/export expiry. Drop, recreate, inventory import, reset identity/digest and
readiness commit in one PostgreSQL transaction. A lost acknowledgement settles that
identity before another reset; rollback retains the source namespace.

`Runtime::discover_orphans` records bounded candidate/protection/disposition pages under
an established workspace root. A retained native unordered stream spans runtime slices;
a new process starts a new enumeration generation from the root and keeps deduplicated
candidates. The report has a durable keyset cursor; the provider listing has no fabricated
restart cursor. Only a successful complete generation establishes enumeration completion,
not an atomic storage snapshot. Local confinement rejects symlinks and escaped prefixes.
Application observations and work per slice are bounded and accounted; provider buffering,
I/O, latency and total RSS are not claimed bounded by that slice budget.

Discovery never grants deletion. `Runtime::reclaim_orphan` takes one explicitly selected
recorded attributable prefix and freshly checks ownership, live publications/intents,
reader/export and retention protection, including overlapping prefixes in other workspaces. An exclusive catalog protection fence precedes the workspace maintenance fence and spans physical removal and disposition commit; protection writers take the shared fence in the same order. Unattributable or unresolved retirement obligations remain
visible and protected. `pse-publication discover`, `orphans` and `reclaim-orphan` expose these
separate operations. External actors modifying local filesystem paths outside this protocol
are not serialized by PostgreSQL locks.

The catalog's marks, epochs and leases are the only coordination; there are no lock files.
There is no automatic retention policy: publications are retired on request only (register
R-36). There is no blanket archival requirement. The protocol is exercised on local file
tables and the in-memory object store; remote object stores are not qualified (register
R-37).

### 20.5 Current contracts and schema evolution

> Proposed amendment: [ADR-0155](../../adr/0155-numerical-derived-families.md), Plan 25n N10.

Changed automatic interpretation receives new settings/strategy/study/job versions and request,
preparation and durable frames. Version checks precede current-body decoding. Historical bytes and
result readability stay intact; execution requires explicit current readmission or typed refusal.
Native source/build identity names the immutable revision without invalidating unrelated evaluators.

> Supplement: [ADR-0146](../../adr/0146-preserve-versioned-operational-transitions.md) (proposed; authorized implementation) replaces normal reset recovery from ADR-0114. Fresh stores are created from generated declarations. Existing stores open read-only after exact support/readiness validation; `just db-migrate` explicitly transitions the supported predecessor while retaining records. Unknown source, conflicting immutable history, incompatible layout or active generations refuse before mutation. Namespace ownership spans separately recorded catalog/control and operations histories. Readiness precedes changed schema, each transition commits with its checksum history, and matching committed progress can resume. Runtime generations hold schema admission until their connections close; current statements are unavailable without verified readiness. Destructive reset is separate maintenance. Historical records retain their recorded contract and never infer new qualification facts from absent fields. Plan 25g implements the directional recorded-contract and retirement protocols below; their focused evidence is owned by that plan and integrated recovery qualification remains Plan 25k.

**Recorded meaning and capabilities.** The portable `pse.semantic-contract.v2` witness
records complete consumed support: fields, keys, enum domains, extension contracts, checks,
invariants, snapshot class and storage policy. Recorded interpretation first verifies its
canonical form, digest, support closure and observed field encoding independently of today's
registry. `VerifiedRecordedContract`, directional `ConsumerProjection`, `ExactWriteAdmission`
and `MigrationAdmission` are distinct products. An understood historical enum domain remains
closed after registry growth. A new consumer may understand an old subset; an exhaustive
consumer refuses unknown consumed members. Changes to constraints, references or physical
meaning require explicit transformation. Unrelated declarations and deprecation alone do not
change consumed meaning. Projection never grants write authority or mutates stored content.

**Portable checks.** Current declarations and verified recorded domains feed one predicate
compiler. Delta checks are reconstructed from that meaning and checked against recorded SQL
and layout. Executable protobuf bytes and codec labels have no compatibility authority and
new writes omit them. Sufficient historical witnesses use the same interpreter; missing,
contradictory or unsupported proof returns a typed refusal without an older execution engine.

**Descriptors.** Version 3 records canonical profile required roots as well as requested
relations and release selections. Version 2 is readable only when an established root
inventory reproduces its recorded profile digest. Only this verified versioned descriptor
rule projects the newly declared inventory column as null. Historical descriptor identity
preimages remain unchanged; new identity roles use the frame catalog (§5.1).

**Explicit artifact transformation.** Portable ordered structural edits, finite domain
recoding and composite reference-key mappings bind exact recorded sources and current target
declarations. Missing mappings refuse unless an explicit identity policy applies. Checked
native lowering validates values, nullability, map uniqueness, keys and reference closure.
Transient migration reads remain read-only; stored migrations use `ArtifactPlan` and the
existing atomic publication boundary, publish new immutable members and include
`runtime.artifact_migration_lineage`. Failure leaves source selections unchanged. Ordinary
output admission retains its semantic guard. Each column requires one composed mapping policy; overlapping rewrites, nonprimitive Identity lowering and nested correlated reference paths return explicit unsupported/refusal results.

**Store lifecycle.** Creation, validation-only opening, read-only migration planning and
expected-plan execution are separate operations. Refinery runs immutable declared transitions
on the existing PostgreSQL driver; it does not decide compatibility. A dedicated namespace
session lease spans both owned histories. Execution rechecks its expected plan, complete
history parsing, exact source and intermediate layout before mutation and reports applied
steps and final readiness. Frozen historical transitions remain unchanged; new persisted
obligations append a preserving transition. No unknown source is reset implicitly.

Entity IDs, publication IDs, Delta versions and table locations are distinct and never
substitute for one another. Supported historical data does not retain historical production
APIs. Generated relation contracts remain the declarations, not inferred storage layouts.

### 20.6 Operational store and durable execution

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) — the registry generates
> the store's schema, and statements are SQL files compiled by Cornucopia into
> `pse-operations-queries` on tokio-postgres (Plan 22 B1, B2, implemented); the lifecycle,
> queue, cancellation, streams, solutions and durability classes it restates from ADR-0112
> (Plan 22 O3–O6, G8, implemented); studies across workers (O7) and read-only query
> providers (O9), implemented; solution origins, pruned captures and published incumbents
> (G8f), implemented. [ADR-0115](../../adr/0115-registry-typed-identities-and-vocabularies.md)
> — identity domains and typed ids. As built, attempt legality has two pure tables and a
> termination is a class with typed per-class columns, refining ADR-0114 Outcomes 12 and 22
> (below).

**Authority.** PostgreSQL 18 holds what changes: attempts, jobs, leases, cancellation
requests, live progress and incumbents, reusable solutions, studies and the publication
catalog. Its records include scientific evidence and publication protections that explicit upgrades and retirement must preserve. The registry owns the meaning and the shape of
every table: relation `runtime.operational_<t>` (`pse-schema::catalog::operations`) is
table `pse_ops.<t>`. Published `runtime.computation_runs`, `runtime.run_lineage` and
`runtime.solve_metrics` are derived snapshots of a published attempt. `pse-operations` owns
the store contract, the repositories, the catalog and reader leases; `pse-runtime` depends
on it, and no semantic, native or columnar crate does.

**Generated schema.** The `pse-codegen` target `postgres` renders
`crates/pse-operations/src/generated/`:

- `schema.sql`: one ENUM type per registry enumeration a store column uses; one domain per
  entity identity, over `uuid`, or over the 32-byte `content_hash` domain for the source
  bundle; and every table with NOT NULL, named primary, unique and composite foreign keys
  and the registry's named row checks;
- `copy.rs`: the binary `COPY` statement of every table, with a `WHERE false` probe that
  types the stream;
- the Cornucopia type mapping and the schema fingerprint (frame `pse.ops.schema.v1` over
  `schema.sql` and `physical.sql`).

A registry `Float64` is finite, so every `double precision` column carries a named
finiteness check and every array column one over its elements; an absent bound is NULL.
There is no separate `finite` facet, because finiteness already belongs to the registry's
float contract. Store CHECK constraints come from named row checks, never from relational
invariants ([§4.1](schema-and-relations.md#section-4-1)). Deletes are explicit: there
is no `ON DELETE CASCADE` and no identity-minting default. The hand-written `physical.sql`
adds indexes, partial indexes, defaults and the append-only revoke on
`attempt_transitions`; its enum literals are checked when it is applied. `Store::create` explicitly creates an absent schema in one transaction and records its
fingerprint. `Store::open` validates only and refuses absent, unsupported or unready
schemas; explicit planning and migration follow [§20.5](#section-20-5).

**Statements.** Every statement is SQL in `crates/pse-operations/queries/*.sql`, one file
per repository, with named parameters and hand-annotated nullability. `cargo xtask codegen`
runs Cornucopia 1.0.1 as a library under the workspace lockfile: it loads the freshly
rendered DDL and `physical.sql` into a temporary database on the local server, prepares
every statement there, and writes the generated crate `crates/pse-operations-queries/`
with a manifest normalized to the workspace. A misspelled column, a misspelled ENUM literal
or a mistyped parameter fails generation. Whole-row statements return registry rows
through a generated composite `FromSql`; identity parameters are cast to their domains, so
the generated functions take typed ids. No SQL is assembled at run time. Progress events
and values, incumbents, source documents, publication members and change windows are
inserted by binary `COPY`, and a re-sent batch is idempotent.

**Value mapping.** The optional `postgres` features of `pse-ids`, `pse-model`,
`pse-diagnostics` and `pse-vocabulary` carry the postgres-types mappings: registry
enumerations to their ENUM types, typed ids to their domains, `ContentHash` to
`content_hash`. They depend on the value protocol, never on a driver, and only the store's
query crate enables them.

**Client.** `pse_operations::Store` is a deadpool pool of tokio-postgres connections. A
pooled connection is verified before reuse, because a cancelled call may leave its
statement running; rustls with the ring provider serves remote servers, and a local socket
uses peer authentication. One listener task per store holds `LISTEN` on the cancellation
and progress channels on a dedicated connection, reconnects with backoff and broadcasts a
resynchronization after every `LISTEN`; a watcher re-reads its authority on a
notification, on a resynchronization and when it falls behind, because a notification only
shortens latency. Store failures are classified by `SqlState` constant
([§23.2](operations-and-validation.md#section-23-2)).

**Durability classes.** A runtime is `Ephemeral` (no store; in-memory progress; it cannot
publish) or `Durable`: every run is then an attempt registered in the store before any
effect, and publication, queued jobs and studies require it. The class is an explicit
policy of the runtime, never a fallback: a durable run whose store is unreachable fails with
the infrastructure class, and ephemeral work never needs the store.

**Attempt lifecycle.** planned → queued → running → {completed, partial, failed,
cancelled}; running → stale on lease expiry; stale → superseded when a new attempt replaces
it; planned or queued work may be cancelled. Pure Rust tables in
`pse-operations::lifecycle` are the only authority for legality; repository functions apply
them in a transaction under a row lock and write every change to the append-only
`attempt_transitions`. There are two tables, selected by the attempt's kind: `TRANSITIONS`
for attempts that execute, and `COORDINATING` for a study's own attempt, which holds no
lease, stays queued while its points run and ends from queued, so it never goes stale and
its publication can always commit. This refines ADR-0114 Outcome 12's single table: one
owner and a pure, testable authority remain, while a coordinating attempt run under a lease
would go stale if its finalization crashed, leaving its intent reclaimable and its points'
members deletable. The runtime mints every identity (run, attempt, job, study, publication,
settlement, reader lease) on the UUIDv7 path before any effect.

**Terminations.** An attempt records one typed termination. `termination_class`
(`TerminationClass`: native, run_state, trajectory, runtime, rule) selects exactly one typed
column, enforced by the named `one_termination` check: `NativeTermination`,
`NativeRunState`, `TrajectoryTermination`, `RuntimeTermination` (cancelled, infrastructure,
unattempted, constant evaluation, unassessed) or the `DiagnosticCode` of a violated rule,
whose named rule is recorded in the versioned `TerminationDetail` document. This refines
ADR-0114 Outcome 22's single `TerminationCode` enumeration; `TerminationCode` survives as
the Rust type over those columns (`pse-operations::attempts`).

**Queue and workers.** A job names its current try's attempt and a typed, versioned
payload (`JobPayload` version 6): a `ModelingJob` (content-addressed source bundles keyed
by the §6.1 package content hash, the case, route, typed `SolveSettings` and a `JobStart`
policy), a `StudyOperationJob` (source bundles and the immutable `StudyPointBinding`,
including its admitted operation and occurrence policy), or a study's finalization. An
unknown version is refused (`UnknownPayloadVersion`). Current study operation descriptors and public study requests use version 2;
stored study definitions use version 4. Decode the version boundary before nested current
preparation and library profiles. Unsupported historical readmission refuses explicitly
without changing stored bytes or inventing missing policy. Operational readiness fingerprints
cover SQL declarations; opaque JSON values require their own version boundaries. `Operations::enqueue` takes the typed
job and frames its request identity from it; an idempotency key is unique. Workers claim
queued jobs with `FOR UPDATE SKIP LOCKED` by priority and availability. Each try is a new
attempt run under the worker's lease, renewed by a heartbeat that also returns the
cancellation flag. An expired lease makes the attempt stale and requeues its job as a new
attempt under the job's retry policy (maximum tries, capped exponential backoff); every
durable connection runs this sweep at start-up, and a worker repeats it. `pse-worker`, a
binary of `pse-runtime`, is the claim loop; it sets the process-level OpenMP environment
SPRAL needs before any thread starts ([ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md)).
`Runtime::work` serves the queue in-process. The CLI's default deployment memory is
8 GiB. Its existing one-sixteenth numeric-worker allocation admits the compiler's
default scratch allowance; explicitly smaller deployments retain their declared capacity
and may refuse compilation before solver execution. Compilation reservations remain
subject to the deployment pool and the same typed resource-admission checks.

**Cancellation.** `cancel_requested` is the authority. Planned and queued attempts cancel
at once; a running try sees the flag through its heartbeat or a listener notification, and
the owning worker maps it onto cooperative native cancellation, across processes.

**Streams.** A durable attempt's progress events are written in batches without an event
cap, their values as typed columns in the `runtime.solve_metrics` value vocabulary, and
publication snapshots them into `runtime.solve_metrics` (namespace `event.<seq>.<phase>`).
A branch-and-bound search's incumbents stream beside them. SCIP reports each new best
solution, and HiGHS each improving MIP solution (callback kind 4), as a typed
`IncumbentEvent`: the objective in post-solve convention with the export offset applied,
the dual bound, gap, node count, native seconds and a throttled primal in original
coordinates (the first at once, then at most one per second, the last always kept;
nonfinite values are absent). Each captured primal is stored as a seed of its step in the
transaction that stores its incumbent row, which keeps the step, phase and elapsed time of
the event that reported it. Streams of attempts that finished longer ago than the retention
policy (seven days by default) are removed by the start-up recovery, except the attempt
chain above an unfinished retry, and the captured solutions go with them: a capture is
deleted unless an incumbent row, the start of an unfinished job or a waiting study point's
predecessor still names it. Output seeds are never pruned; automatic retention of anything
else remains register R-36. A durable run publishes its incumbent stream as the store holds
it when the attempt ends, in `runtime.incumbents` (one row per incumbent with its step,
phase, elapsed time, objective, bound, gap, nodes and the captured solution's identity, which
the published row outlives); an ephemeral run keeps its incumbents in
`runtime.solve_metrics`. *Tested* by `captured_solutions_pruned_with_streams` (store units)
and `published_incumbents_equal_stream_snapshot` (runtime units).

**Solutions and resumption.** Reusable seeds (`solutions`) are stored in original
coordinates, keyed by the coordinate-compatibility stamp and the preparation identity, with
the vectors their kind allows. Each records its origin (`StoredSolutionOrigin`): `output`
for a step's accepted output seed, and `incumbent` for a point captured from an attempt's
incumbent stream, which must name that attempt (row check `capture_has_attempt`).
`with_stored_start` starts a step from the newest compatible output seed, never from an
incumbent capture, or from a named seed (`StartSource::Stored`,
[§17.4](numerical-execution.md#section-17-4)); the seed's content identity enters lineage
(`latest_start_skips_incumbent_captures`). A job's `JobStart::ResumeFromParent` starts
from the latest incumbent in its parent attempt chain, injected into SCIP or given to HiGHS
as a start, so a killed worker's successor resumes the search; `StoredSolution` names one
seed. Studies across workers are [§19.3](workflows-and-results.md#section-19-3).

**Study member recovery.** ADR-0148 records the exact pre-effect native publication ticket
under the fenced occurrence before member writes. A stale native-started attempt has an unknown
effect. Under the study/job locks and expected revision, an absent ticket proves no write was
authorized only after excluding a running worker and checking the current receipt state. A
persisted ticket instead requires inspection by the existing member-attempt owner. Absent,
provisioned or partially written native members remain unresolved; actual partial members stay
inspectable. Only every exact ticket member committed establishes idempotent effect knowledge.

Written members retain actual versions. Repeated attachment accepts an identical descriptor;
a conflicting descriptor refuses. A newer attempt can replace the point member inventory only
when the previous effect is known absent or idempotent, retaining its attempt history. Receipt
knowledge establishes effects and available members, never scientific usability or an independent
catalog publication commit. Recovery preserves attempt history and the live-lease/revision fences;
retry consumes the known effect under the shared occurrence policy.

**Query surface.** A durable runtime's query sessions see thirteen operational relations
as read-only DataFusion tables under `pse_ops` (attempts, attempt transitions, jobs,
progress events and values, incumbents, solutions, studies, study points, workspaces,
publications, publication members and settlements), provided by
`pse-runtime::workflow::operational_tables`. A scan runs a generated statement, pages in
primary-key order at READ COMMITTED without holding a connection between pages, and builds
batches with the registry-generated `pse-relations` builders, so values are checked against
their field contracts. Equality and `IN` on identities and states, and comparisons on the
time column, push down as `Inexact`; everything else is evaluated by DataFusion. The
providers declare no constraints, so a query that reads one is never served from a cache.
`Runtime::query_session` joins them with a run's results or an open publication, and
`Runtime::progress` merges an attempt's progress events and incumbents in time order,
following the listener until the attempt ends. The ADBC PostgreSQL driver is not adopted:
it would add a C driver manager and a second client for tables the generated builders
already serve ([operational-store guide](../../dev/operational-store.md)).

**Limits.** A scan's pages are not one snapshot. Study point transitions serialize on the
study row, and the 10 000-point scale of scenario S15 and statement performance at volume
are unmeasured. Native receipt reconciliation is required before a crashed study point can replay;
real restart/publication qualification remains Plan 25k scope. A finalization that exhausts its retries leaves its study concluded without automatic
recovery. A direct cancel of a waiting point can deadlock with its predecessor's
completion; the transaction retries (register R-42). Remote servers and remote object stores are not qualified (register R-37).
Operating the store (bootstrap, reset, generation order, backup, doctor) is covered by the
[operational-store guide](../../dev/operational-store.md).
