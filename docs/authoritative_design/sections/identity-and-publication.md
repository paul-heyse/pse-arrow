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
`pse-engine` owns the native provider hierarchy; `pse-catalog` owns Delta durability,
publication, settlement and retention.

## 5. Identity, revisions and exact selection

Identity is a contract, not a convenience. Every identity is produced from declared
meaning through a versioned projection, never from row position, display names,
floating-point accidents or whole-object serialization. A different meaning gets a new
projection or context string; an old one is never reinterpreted. The rationale for
versioned projections is [ADR-0089](../../adr/0089-semantic-identity-projections.md).

### 5.1 Forms of identity

| Form | Representation | Assigned by | Scope |
|---|---|---|---|
| Semantic ID | `pse.semantic_id`, 128-bit `pse_ids::SemanticId` | authoring (authored entities), keyed derivation (derived entities), UUIDv7 (runs, publications, attempts) | survives revisions, reordering, re-batching, projection and publication |
| Artifact-local position | integer ordinal or coordinate index | the preparation that built the artifact | only within that artifact |
| Content hash | `pse.content_hash`, 256-bit BLAKE3 `pse_ids::ContentHash` | a named projection or canonical preimage ([§5.3](#section-5-3)) | identifies one immutable meaning under its projection version |

**Authored entities.** Identity is assigned at creation and stored with the entity; it is
never computed from the entity's name, so a rename changes a label and nothing else. The
package declares its policy (`IdPolicy`, the `id_policy` of `authored.packages`):

- `explicit` (default): the entity carries a UUIDv7. A missing ID is
  `authoring.parse.missing_id`; the document editor's `assign_ids`
  (`pse-runtime::authoring_driver::document`) inserts missing IDs as source edits.
- `named`: for reference packages whose qualified names are the public contract (units,
  elements, constants, standard templates). `pse_ids::named_id` derives the ID from
  package ID and qualified name under the frozen `pse:named:v1` context. A rename is a new
  entity by construction, so renaming such an entity is refused
  (`authoring.reference.rename_named`).

References between entities (template submodels, connections, case targets) are stored
by identity, so no authored fact depends on a name. Resolution lives in
`pse-authoring::ids`.

**Derived entities.** A derived ID is a keyed digest of what the entity was created from:
`FramedHasher::new(context)` over the parent identities and index members, finished as the
first 16 bytes of BLAKE3's extendable output. Examples are port identity
(`named_id(instance, "port:<name>")`) and scalar occurrence identity (instance,
declaration and index members, in `pse-runtime::workflow::composition`). Unchanged inputs
reproduce identical IDs; changing an index set changes only the IDs that depend on it.

**Positions.** Native layouts assign coordinate order, sparsity positions and block
indices. These are artifact-local. Coordinate order and sparsity belong to the native
layout's identity; cross-artifact references always use semantic IDs.

**Row tokens.** `pse-relations::identity` derives a versioned row token from relation
identity, key-contract version and the ordered declared primary-key values. Mutable
payload and Delta version do not enter it. A token names a row within an exact selection;
it is not a membership proof, and actual key columns remain authoritative.

`pse-ids` is the sole direct `blake3` dependent (`tests/governance/tests/blake3_owner.rs`);
other crates hash only through its framing APIs. Golden vectors in
`crates/pse-ids/tests/golden_vectors.rs` freeze the context strings and digests.

### 5.2 Model revisions, cases, runs and attempts

Definitions, instance bindings, case values, analysis requests, resolved numerical
policy, prepared artifacts, used starts, results and publications have distinct
identities and lifecycles. Cases and results never mutate the model
([ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md)).

**Model revision.** `ModelBuilder::freeze` admits the selected model, its dependencies and
its cases through the single selected-admission boundary
([ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)) and returns an
immutable `ModelRevision`. Editing starts a new builder from a revision; a failed
admission leaves the original untouched. The revision identity
(`pse.native.model-revision.v2`) frames the model ID, physical context identity, each
selected case identity, flowsheet graph identities, resolved balance/numerical/scaling
declarations, material contract, dynamic/fit/observation/dataset declarations, the
semantic contract of the consumed source relations and provider registrations.

**Case.** A case identity (`pse.native.selected-case.v1`) frames the case ID, the
admitted case structure and its values. Because a revision includes its selected case
identities, editing a case yields a new revision; compiler reuse is decided by
dependency, so unchanged definitions and structure are reused
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
| Used start, result, publication | separate scopes of their own | the request identity |

**Run and attempt.** Starting a prepared case mints a fresh run ID (UUIDv7). The run ID
names the attempt; it is not part of request identity, so repeating the same request is a
new run with the same request and preparation identities (`runtime.run_lineage`). A
publication has its own publication ID and attempt ID, distinct from the run it
publishes ([§20.2](#section-20-2)).

### 5.3 Canonical framing and hashing

`pse-ids` defines two frozen framings, and the difference between them is contract:

- **Keyed derivation** (`derive_id`, `derive_hash`, `FramedHasher`): BLAKE3
  `derive_key(context)`; every part, integers included, is a u64 little-endian length
  followed by its bytes. Used for entity IDs and every named projection digest.
- **Canonical preimage** (`FrameSink`): unkeyed; variable-length components carry a
  u64 length, fixed-width components (IDs, hashes, counts, schema versions) carry their
  declared width and no length. Used for relation content hashing.

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

A published catalog is opened from its control row, not assembled from independent
latest-table reads: `pse-catalog::delta::publication::Publication::open` binds the exact
control version and member versions; payloads stay lazy. Delta and Arrow streams are the
hot path; IPC remains an encoding and spill boundary.

## 20. Persistence, publication and reproducibility

Durability is an explicit effect, and visibility is decided by one conditional commit.
Execution stays in memory until a caller asks to publish; publication never re-runs
science, and a scientifically failed attempt can still be published faithfully. The
protocol is decided in [ADR-0091](../../adr/0091-immutable-publication-contract.md).

### 20.1 Delta durable relations

Delta tables persist authored sources and declarations, published results, provenance,
publication control and retention records. Native Delta and DataFusion operations own
reads, writes, DML (`pse-catalog::delta::dml`), schema transformation, change data and
maintenance. Arrow buffers and streams serve transient execution; no intermediate is
forced to commit. There is no separate manifest store, content-addressed store or
publication journal: the control relation is the only publication record.

Each relation's registry declaration is lowered to native Delta `CHECK` constraints and
identity table properties (`pse-catalog::delta::contract`). Declared Arrow types that
Delta cannot store directly use inspectable lossless layout projections
(`pse-catalog::delta::layout`). Every Delta command receives the actual invocation
`SessionState`; the validating write route preserves local constraints and commit
properties, and raw provider mutation cannot bypass it. Candidate-wide key, reference,
completeness and domain checks run against exact candidate versions
(`pse-catalog::delta::admission`).

### 20.2 Exact coherent publication

A caller publishes a completed `RunResult` in two steps
(`pse-runtime::workflow::publication`):

1. `prepare_publication_request` takes a `PublicationRequest`: a base directory URL,
   workspace ID, the exact expected parent publication (never rebased), a
   caller-stable publication ID and attempt ID. It plans every member and mints a
   serializable `PublicationTicket` before any effect. It performs no writes.
2. `PublicationAttempt::commit` writes each member to an isolated immutable location
   `members/<publication>/<attempt>/<schema>/<table>/`, then commits the control row in
   `runtime.publications` last, conditional on the expected parent still being the head.
   It returns a `PublicationRoot`: the control location and its exact Delta version.

The control row names every member's relation, contract fingerprint, table URI and exact
Delta version, plus the exact inputs consumed. Readers pin a root; there is no
latest alias. An artifact descriptor (`pse-model::artifact`) records the publication
profile's semantic contract, requested relations, release members, source/build and
algorithm/target identities and value assumptions; required relations appear even when
empty (`reference.artifact_profiles`).

Outcomes are distinct (`pse-catalog::delta::publish::PublicationError`):

| Outcome | Meaning |
|---|---|
| committed | the control row names exactly this attempt's members |
| `Conflict` | a concurrent publication won or the expected parent is no longer head |
| `IdentityReused` | the same publication or attempt identity was used with a different request |
| `Unresolved` | storage cannot establish whether the commit happened; never treated as rollback |

`PublicationTicket::settle` is read-only: it inspects the control history and the
recorded member attempts and returns `Committed`, `ProvedNoncommit`, `Conflict` or
`Unresolved`. It never writes, executes producers or calls a solver, and repeated
settlement returns the same established outcome. Missing history is unresolved. An
uncertain outcome is never permission to rematerialize an attempt. Delta
`SetTransaction` is recorded as a durable witness; at the pinned version it does not
deduplicate replay, and an attempt label alone is never authority
(`pse-catalog::delta::attempt`).

Member writes can remain after a failed or conflicting attempt without becoming visible;
they are reclaimable ([§20.4](#section-20-4)). A failed attempt never poisons the logical
base. Publication races, lost acknowledgements and repeated settlement are exercised by
the catalog and `tests/lifecycle` suites.

### 20.3 What a run references

A run records the exact identities it consumed, not names:

- `runtime.run_lineage`: model ID, revision identity, case ID and the request,
  preparation, profile, numerical, physical and environment identities of every step.
- `runtime.solve_metrics`: effective backend options, the submitted start and native
  observations; unavailable metrics carry a typed reason, never a synthesized zero.
- `runtime.computation_runs`: the joined job's kind, source and profile identities,
  state, native termination, qualification and candidate facts.
- The publication control row: exact input and member selections.

Provider parameter data are retained with the immutable revision. Provenance and
diagnostics are typed relations queryable through the same provider hierarchy. Names,
metadata and hashes alone never certify validity, execution or equivalence. Result
meaning is owned by [§19](workflows-and-results.md#section-19).

### 20.4 Reproduction, reuse and retention

Three lifetimes are independent. Invalidating one never silently changes another.

| Lifetime | Owner | Governs | Never |
|---|---|---|---|
| Semantic reuse | `pse-compiler::workspace` (Salsa) | whether prepared mathematics can be reused; keys include complete inputs, absence states, math environment, source and build identity | persisted; evicted by storage maintenance |
| Retained runtime artifacts | `pse-runtime` prepared products and native sessions | memory held by immutable programs and native workspaces; shared immutable products, attempt-private mutable state | treated as program validity; retained past the last owner |
| Storage snapshots | `pse-catalog::delta` retention and maintenance | which Delta versions remain reopenable | collected while referenced or while a local reader holds a lease |

Semantic reuse compares complete inputs; conservative recomputation is always valid. An
explicit program clear fences late insertion; a retiring in-flight preparation remains
owned until completion and is distinct from a new caller's cancellation. Salsa rotation
follows retained entries and bytes. Salsa databases and handles are never persisted.
Fixed symbol registration is a determinism control, not a claim of bitwise
reproducibility across environments.

**Reproduction.** Reopen exact versions through `Publication::open`; derived layouts are
regenerated as needed. Bounded change data over an exact selection
(`pse-catalog::delta::changes`) supports change impact; a missing log entry or changed
declaration refuses the window rather than returning an empty change set. Change data is
neither permanent audit history nor automatic compiler incrementality.

**Retention.** Protected versions are computed by a native query over publication,
output, attempt and change-window records (`runtime.retained_versions`,
`runtime.release_checkpoints`). Maintenance (`MaintenanceAction::Collect`, `Optimize`,
`ReclaimUnpublished`; `pse-catalog::delta::maintenance`) is an explicit writer operation: it takes exclusive coordination,
requires the expected publication head, materializes and verifies every protected
version before deleting anything, and refuses when history is missing. Unreferenced
member attempts can be reclaimed; referenced ones cannot. There is no blanket archival
requirement.

**Local coordination.** Local file tables share one OS lock file per table
(`pse-catalog::delta::lease`). Writers initialize coordination before emitting files;
readers only acquire existing shared ownership and never initialize or repair it.
Remote destructive maintenance is refused as unqualified; remote reads carry no
reader-protection protocol.

### 20.5 Current contracts and schema evolution

**Semantic contract.** `pse-schema::fingerprint::SemanticContract`
(format `pse.semantic-contract.v2`) is the complete support closure of the requested
relations: fields, keys, enums, extension types, checks, invariants, snapshot class and
storage policies. Prose and incidental library encodings are excluded; the native
execution encoding has a separate identity. Each durable table records both as Delta
table properties. `pse-schema::compatibility` decodes the recorded witness and compares
it with the independently compiled consumer expectation:

| Result | Meaning |
|---|---|
| `MigrationRequired` | an older or unversioned contract; the reader does not interpret it |
| `Incompatible` | a valid recorded meaning differs from the expected meaning |
| `UnsupportedEncoding` | the meaning agrees, but this reader cannot execute the recorded layout |
| `Malformed` | the witness is missing, contradictory or noncanonical |

A documentation-only change therefore remains readable, and an equal hash is never
treated as a migration. Artifact descriptors apply the same rule to their own format
version.

**Schema evolution.** Opening never discovers or applies a conversion. An explicit
migration is declared (`reference.schema_migrations`, `pse-schema::model::migration`)
and compiled by `pse-engine::session::schema_transform` into native projections, checked
defaults and nullability obligations; the caller selects both versions.

No store written before the current contracts is supported, and there is no legacy
runtime. Fixtures start from source. Entity IDs, publication IDs, Delta versions and
table locations are distinct and never substitute for one another. Relation-level
detail is in the [generated runtime reference](../../generated/relations/runtime.md).
