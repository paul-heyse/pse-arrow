---
title: Identity and publication
status: current
---

# Identity and publication

This area decides what makes two things the same, what an immutable model, case, run
or publication names, and how results become durable without a reader ever observing a
mixed state. `pse-ids` is the only hasher and owns identity framing; `pse-columnar`
owns relation canonicalization; `pse-schema` owns semantic contract identity and
compatibility; `pse-runtime::workflow` owns scientific admission, execution and completion;
`pse-engine` owns the native provider hierarchy; `pse-operations` owns canonical source
revisions, execution records, immutable products, result and analysis admission, protected
selection and reclamation in one database. Registry declarations generate its typed wire
contracts and native structural functions.

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
> blocks are instances, so there is no separate root, definition, member or block identity.
> Canonical lookup keys remain separate from these scientific identity domains under
> [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed).

| Form | Representation | Assigned by | Scope |
|---|---|---|---|
| Semantic ID | `pse.semantic_id`, 128-bit `pse_ids::SemanticId` | authoring (authored entities), keyed derivation (derived entities), runtime minting for scientific runs and direct attempts; keyed derivation for claimed worker attempts | survives revisions, reordering, re-batching, projection and publication |
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
payload and storage lookup keys do not enter it. A token names a row within an exact selection;
it is not a membership proof, and actual key columns remain authoritative.

**Typed identities.** A registry key column may declare an entity identity
(`declare_identity`, `FieldContract::with_identity`). A foreign-key column inherits the
identity of the column it references, and assembly refuses a conflicting declaration.
Ownership is declared separately (`FieldContract::with_owned_identity`): an owner is its
relation's single-column primary key, each identity has at most one, and a column that only
carries an identity never owns it. Storage membership and scientific lineage remain separate
from the declaration of an identity domain. `reference.schema_identities` describes the declarations. The
generator emits one typed id per identity into `pse-model` through the single
`semantic_id_newtype!` macro of `pse-ids`, which the `pse-quantity` physical-registry ids
also use. A typed id is a transparent wrapper with `From` in both directions and
`Display`; its serde form and framing are those of the wrapped value, so no identity byte
moved. Generated rows and the Python `NewType`s of `pse.contracts.identities` follow.
Modeling identities include package, declaration, instance and fit. There is no separate
model or case identity: a model is named by the root declaration it specializes and a case
by its case declaration, both as `declaration` ([§20.3](#section-20-3)). Consumers take typed
ids, so passing one identity where another belongs fails to compile; `compile_fail`
doctests pin representative swaps.

**Canonical lookup keys.** Native record keys identify exact revisions, versions, runs,
attempts, manifests and analysis receipts. They are distinct from scientific semantic IDs
and content hashes. A canonical run key carries its scientific run identity; its attempt
key is an opaque framed derivation of the run and claim-operation identity. The complete
canonical key is used for lookup, never a truncated scientific ID, display name or request
hash. Per-problem sequence and attempt generation are recorded ordering domains, not
wall-clock estimates. The codec preserves the complete integer domain as checked decimal
values. Finite scientific values retain authoritative bits and a verified numeric projection;
signed zero remains distinct, and exceptional diagnostic bits have their separate domain.

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
| Prepared artifact | consumed specialization constants, profile, actual library environment and qualified relevant producer inputs | mutable native state; unrelated executable inputs |
| Native layout | coordinate order, sparsity, compatibility | coefficient values |
| Used start | the submitted seed by content, predecessor attempt, partial start and reuse of retained native state, framed into the completed step's request identity ([§16.5](numerical-execution.md#section-16-5)) | the run and attempt that produced the seed |
| Result, publication | separate scopes of their own | the request identity |

**Run and attempt.** Starting a prepared case mints a fresh scientific run ID (UUIDv7).
Ordinary execution registers its canonical run and obtains a fenced attempt before native
effects. Explicit ephemeral execution has no durable attempt. An allowed retry retains the
run and creates a new attempt generation; repeated compatible requests otherwise create
separate run histories. Result rows retain scientific lineage while exact native handles
select the canonical run and attempt. Neither identity is part of the mathematical request
merely because it is a storage address. A changed submitted start changes the completed
request identity ([§20.3](#section-20-3)). A closed result manifest identifies exact admitted
observations; it does not constitute a separate publication or certify numerical usability.

### 5.3 Canonical framing and hashing

> Decision: [ADR-0167](../../adr/0167-frame-flow-projection-v2.md) (proposed; maintainer-authorized implementation).

Current physical flow projections use `pse.flow.projection.v2`, with explicit collection
and item tags, top-level inventory counts, and separate port/binding counts under each
node/connection. Canonical fields, sorted order and canonical floating-point bits retain
their existing interpretations. Historical `pse.flow.projection.v1` identities retain
their original meaning; current admission does not emit or fall back to V1. A flow key is
a versioned projection fingerprint, not complete physical-registry equivalence: retained
binding quantities and the current checked context remain part of full graph equality.
Tear assumptions/compatibility and recycle causal identities consume the current key;
this evolution introduces no sole-fingerprint cache or durable fast-hash replacement.

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed target). Versioned relevant producer identities replace whole-tree build hashes in scientific product eligibility. Actual consumed source/generated/native inputs and Cargo units govern those identities; unknown inputs block persistent reuse. Complete executable attestation remains at the run boundary.

Relevant producer capture uses the actual selected Cargo production root and its units, resolved source/version,
features/profile/target and emitted compiler dep-info, including consumed generated/native inputs.
Uncompiled tests/examples are excluded; ambiguous associations and unreviewed executable I/O
remain refusals. A source-bound completeness declaration can qualify reviewed build-script and
procedural-macro owners. Consumed raw source is a conservative fallback when normalized source
cannot preserve macro/location meaning; that fallback may invalidate on otherwise irrelevant
inline text changes. No blanket closure claim follows from Cargo rerun hints or file discovery.
Operator qualification binds the expected package, target, kind, profile, features and ABI
of the actual selected production root to its relevant build-input identity and independently
observed deployed artifact bytes. Relevant inputs are identified before linking; artifact
association follows build/install, including admitted Python installation transformations.
Complete dirty outer source/build observation, including authored Rust, Python and tooling
composition roots, root build declarations/locks and Git context, is recorded at
deployment/run admission and is not a compiled input of every associated role. Role capture
retains the tool and flag inputs actually consumed by that artifact. An installed artifact
without its owning authored checkout records source observation as absent and identifies its
actual artifact context; another Rust project is not treated as that checkout. Missing outer
source evidence does not prevent ordinary admission or persistence and never supplies a
scientific producer qualification.
Role artifact identities need not match: their required scientific/ABI compatibility and each
actual artifact association must hold. Unrelated outer edits change that observation without
rebuilding unaffected artifacts; dirty consumed inputs invalidate affected associations.
Unknown consumed inputs, same-path artifact replacement and manufactured matching receipts
refuse eligibility. Identity/receipt versions are admitted before their current shape is decoded;
historical keys retain their original meaning. Incomplete deployment evidence permits normal
admission and persistence but refuses cross-build scientific replay.

Ordinary preparation can instead use a distinct versioned deployment-local admission:
exact independently observed receiving role/artifact, interpreter, supported loaded ELF
closure and effective reconstruction configuration. It requires no source/compiler receipt
and confers no cross-build relevant-source qualification. On the controlled Linux/glibc
2.39 profile, actual persisted receiving and new eligible publication qualify that
receiving context. Immutable reconstruction uses a short loader callback; the controlled
mathematical symbol universe admits compatible append-only registration and excludes
callbacks or replacement of live definitions. Independently admitted immutable mathematics
then retains its complete semantic meaning without another host capture on each memory
lookup. Unsupported loaders, unknown/JIT/deleted mappings or changed consumed context
refuse eligible replay and new eligible publication. Historical unqualified publications
remain unqualified. The local namespace is `pse.local-runtime.v2:`; version 1 retains its
historical meaning and cannot admit new products. The runtime body envelope is version 3;
scientific recipe and canonical product/blob frames retain their existing interpretations.
The proposed ADR-0164 records the supported roots and mutation assumptions, separating
strict producer qualification from this narrower guarantee.

Product publication and scientific request have separate identities. A store-issued protected
admission supplies a stable publication identity through transaction/acknowledgment retries.
Already rooted exact descriptions are checked at the actual store using their original
provenance before a new publication identity, producer observation or staging copy is
constructed. Current dependencies are independently rechecked; old provenance grants no
new publication permission. Acknowledgment errors remain errors. An exact rooted
acknowledgment can settle after selection expiry, while new admission requires a live pin.
Already rooted descriptions are shared without changing their immutable origin. Explicit
release fences that publication; fresh scientific admission can publish another. Generic retained
root operations mint or move deliberate history only; product, run, analysis and active-attempt
roots are issued by their admission owners. Generic release cannot redirect or release
execution or analysis lifecycle roots.


> Decision: [ADR-0155](../../adr/0155-numerical-derived-families.md) and
> [ADR-0154](../../adr/0154-declared-numerical-strategy.md) (proposed target).

Current declared numerical strategy, derived-family structure/binding and retained products
use separately versioned frames over their consumed dependencies. Semantic proposal transport
and native payload compatibility are distinct. New current frames do not change historical
bytes or grant current reuse from readable historical provenance. Current decoding and admission check their recorded interpretation; unsupported historical
execution refuses without inventing policy or scientific facts.


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

Source revision, admitted closure, semantic body, prepared view, binding, profile and lineage
request hashes are distinct types. `.as_id()` is an explicit lowering at a stored/raw transport
boundary, not a grant of admission or identity equivalence. Canonical immutable requests,
payload digests and descriptor keys use versioned framing through `pse-ids`; operation
acknowledgments compare exact supplied requests. Historical frame spellings and bytes retain
their original meaning without retaining the retired operational job or storage APIs.

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
named projections in [§5.2](#section-5-2), and canonical admissions name exact immutable versions and result manifests
([§20.2](#section-20-2)). The unkeyed frame reserves `pse.snapshot.v2` for an aggregate
snapshot identity; no implementation computes it. Source documents are identified by
the encoding checksum of their text.

### 5.4 Exact relation selection and the provider catalog

`pse-engine::provider` binds relations into a native DataFusion catalog/schema/table
hierarchy for its actual relational and optimizer consumers. Names resolve once per
operation to exact relation versions, roles and provider owners; aliases do not float.
Backend failure is an error, not absence. Admission checks run at platform logical-plan
boundaries, including subqueries. Pushdown claims preserve complete values and
multiplicities; unknown statistics remain unknown. Plan encodings and `EXPLAIN` output
remain diagnostic evidence, never relation identity
([ADR-0044](../../adr/0044-noncanonical-plan-evidence.md)). This capability does not provide
a second publication authority or a public SQL convenience over canonical results.

**Selected source reads.** A `SelectedRead` owns a store-issued protection on one immutable
revision. It records the actual consumed premises: present or absent scoped names and logical
objects, complete scope/kind/reference membership and compiler/provider interpretations.
A store-issued paged membership cursor is bound to that same protection and selector.
Only a completed inventory records its complete premise; interruption or cursor loss leaves
publication and reuse refused. Grouped reads share a bounded protected acquisition while
preserving each exact selector. Absence and complete inventory are meanings, separate from
the conflict-guard generations used to recheck them atomically. An edit outside consumed
meaning need not invalidate a product; changed consumed membership or interpretation does.

**Selected results.** A result read selects one admitted terminal attempt and its exact
closed manifest, then resolves only the sets and contiguous batch ordinals admitted by that
manifest. Batch identity, payload digest, row count, schema and range coverage are checked
before returning scientific rows. Late, abandoned or unrelated staging never joins the
selection. An explicitly selected historical terminal attempt remains readable while its
history is retained, even after another attempt becomes current. Failed, partial and
cancelled selections retain their own terminal class and observations. A latest-problem
selector explicitly declares acceptable classes and orders by recorded run sequence; it
never substitutes success for a requested failed result or orders by wall clock.

## 20. Persistence, publication and reproducibility

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed). The implemented canonical substrate replaces the PostgreSQL/Delta publication composition. The decision remains proposed; implementation and focused controls do not establish comprehensive qualification or decision acceptance.

One authenticated loopback SurrealDB database holds canonical source revisions, compilation
products, execution receipts, original scientific results and analysis lineage. RocksDB is the
supported native backend. The registry declares their contracts; `pse-operations` supplies
typed operations and generated native structural functions. Scientific computation and bulk
payload staging stay outside short guarded admission transactions. Ordinary execution
retains the actual scientific outcome automatically. Explicit ephemeral execution keeps its
observations in process; a durable storage failure never falls back to ephemeral execution.

### 20.1 Canonical durable relations

The former Delta durable-relation mechanism is retired. This section identity remains the
persistence pointer: canonical source/result admission is owned by
[§20.2](#section-20-2), lifetime and reclamation by [§20.4](#section-20-4), and execution by
[§20.6](#section-20-6). There is no parallel publication catalog, member table journal,
COPY client or compatibility backend.

**Sources.** A problem head selects an immutable revision with an exact recorded sequence.
Revision membership uses logical object identities and immutable versions over sequence
intervals; names and structural reference edges are selected at that revision. Documents,
typed physical input rows and other declared source kinds retain their exact payload
receipts. Canonical physical-source objects have a separate 3 MiB input-object bound. Their declared
source kind and interpretation checksum are checked independently of the canonical revision
lookup key and compiler physical-context identity.
Immutable revision receipts remain attributable after explicit source reclamation; range
markers prevent those receipts from reopening removed content.

**Results.** Registry-admitted Arrow rows are stored once in independent uncompressed IPC
blocks of at most 512 KiB, including schema, one record batch and explicit end marker.
Framing, schema and layout preflight precede decoder allocation; compression, dictionaries
and unsupported layouts refuse. Row splitting preserves exact scientific values and schema
but cannot make an indivisible oversized schema fit the transport purpose. These are
per-operation bounds, not a claim that total trajectory size, process RSS or provider
buffering fits one blob. Seed and completion owners use their own exact bounded chunk
codecs under the same admitted result-set membership.

Sparse scalar indexes and dense output descriptors reference the original IPC batch and
row/field coordinates; they do not replace it with a second scientific payload. Finite
numeric projections serve predicates and are verified against authoritative values.
Missingness is not zero, and signed zero and exceptional diagnostic bits retain their
declared meaning. Dense trajectory groups preserve symbol/sample identity and original
coordinates; half-open relation row ranges select the recorded stored order. Output indexing currently
covers solve variables/constraints, fit parameters/observations and simulation sample
values. Unsupported scientific output fields refuse explicitly.

### 20.2 Exact coherent publication

The visibility boundary is canonical admission of an exact closed descriptor. Neither
staging bytes nor observing a prefix establishes coherent membership or scientific success.
A source edit, compilation product, result or analysis uses its own owning admission while
sharing immutable payload, protection and conflict-guard mechanisms.

**Staging and activation.** Bounded batches create immutable objects/blocks and exact
structural references with digest-and-byte comparison; an identical retry can share them,
and conflicting content refuses rather than overwriting. A finite staging lease and its
generation fence every write. Closing freezes the exact manifest. Structural validation
and complete coverage reconciliation precede activation; the short activation transaction
checks the expected parent, frozen descriptor and actual guards before moving a source head
or exposing the product. Large payloads do not enter that visibility transaction.

Immutable operation requests and acknowledgments settle uncertain outcomes by exact
identity and request comparison. A lost response is not success, absence or permission to
rerun an effect. Typed transaction conflicts retry the whole guarded decision. Other errors
remain errors or invoke the operation's exact acknowledgment settlement. Two callers
publishing the same in-flight product do not take over its live stage: one owns the writer
fence and the other waits within the request deadline for the immutable acknowledgment.
An exact completed replay preserves the original generation, expiry and origin.

**Scientific products.** Protected selected reads record consumed premises before product
admission. Product lookup and publication recheck those premises, current interpretation,
protection and relevant producer eligibility. Only the controlled scientific producer can
issue a description eligible for mathematical replay; generic bytes and matching hashes
cannot acquire that authority. A product's stable publication identity is separate from
its scientific request and is retained through acknowledgment settlement.

**Results and analyses.** Result close freezes the append frontier. The scientific owner
reconciles set descriptors bounded to 128 KiB outside the transaction, then seal atomically compares
the manifest and current recovery authority. Failed, partial and cancelled completions may
retain their truthful admitted observations. Analysis admission stages exact declared graph
membership and input roots, then exposes only the activated complete graph. Original
numeric evidence remains in admitted IPC row/field coordinates; an edge does not invent a
derivative, rank, quantitative sensitivity or scientific usability.

### 20.3 What a run references

A run records the exact identities it consumed, not names. Its canonical header retains the
primary revision, every additional selected source revision including physical inputs,
scientific demand/resolved configuration, source selection and complete deployment
attestation. Scientific lineage remains in the original admitted result relations:

- `runtime.run_lineage` names model, case, instance and fit, revision, request, preparation,
  profile, numerical, physical and environment identities. The model is its root declaration;
  the case is its case declaration; instances and fits retain their original identity roles.
  Producers derive these through `pse_model::lineage`. An algebraic completed request also
  frames the start actually used and retained native-state reuse
  ([§16.5](numerical-execution.md#section-16-5)).
- `runtime.solve_metrics` retains effective options, submitted starts and actual native
  observations; unavailable metrics carry typed reasons rather than synthesized zero.
- `runtime.computation_runs` retains the joined operation's kind, source/profile identities,
  state, termination, candidate and qualification facts.
- The exact attempt and closed manifest identify admitted result membership separately
  from numerical permission or scientific usability.

The start receipt retains predecessor attempt and seed origin, layout/profile/data stamps,
backend, typed payload, explicit partial seed, applied transformations and whether the
native API received it ([§17.6](numerical-execution.md#section-17-6)). Selected solution
identity and exact original bits remain attributable. A missing or incompatible explicitly
requested seed is a typed refusal, not an invented empty start.

An immutable analysis records method, configuration, interpretation and all selected source
revisions. Result-input edges name the exact run, attempt and manifest. Dependency analysis
separates original numerical incidence from conservative execution support; retained-result
analysis preserves existing sensitivity evidence. Different method or input identity yields
a different analysis while older retained headers keep their recorded attribution.

Names, metadata, hashes and graph reachability alone do not certify validity, execution or
equivalence. Scientific completion and result meaning remain owned by
[§19](workflows-and-results.md#section-19).

### 20.4 Reproduction, reuse and retention

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed). Semantic reuse, allocation ownership and durable retention are independent lifetimes.

| Lifetime | Owner | Governs |
|---|---|---|
| Semantic reuse | compiler and scientific preparation owners | complete consumed meaning and qualified relevant producer compatibility |
| In-process retention | shared native cache service, pool and immutable value owners | bounded retained references and buffers; attempt-private mutable native state |
| Durable history | canonical roots, protections and owning retirement operations | exact source/result/analysis selections that remain reopenable |

Cache eviction never deletes scientific history or establishes invalidity. Explicit clear
fences late insertion; existing active owners remain valid through completion. The Runtime
privately retains at most one exact physical document admission and one physical IPC receipt
set within the shared allocation budget. Keys include exact canonical revision/checksum,
store/registry/session owners, package metadata, quantity/precondition owners and original
checked batch owners as applicable. The compiler physical identity alone cannot authorize a
hit when support rows changed. Hits freshly protect the exact source. Protections survive
through new run/analysis root admission, and a generation captured before the first await
prevents an old in-flight load from republishing after clear. Mutable solver workspaces and
attempt authority are never cached in these immutable products.

**Roots and reads.** Reachability includes problem heads, deliberate history, admitted
products/runs/analyses, active attempts and protected selections. Generic retention can mint
or move deliberate history only. Product and lifecycle roots are issued atomically by their
owners; generic release cannot release run, analysis or active-attempt roots. Selected root
identity, problem, revision and owner are checked against server receipts rather than
caller-supplied sequence claims. Explicit forgetting of default history does not release
independent roots.

Read acquisition, renewal, admission and reclamation share the retention guard. Every
continued database read requires a live exact protection; expiration or reclamation refuses
rather than recovering bytes from an old cache. Result reads additionally protect the exact
run/attempt/manifest. Final-drop release uses the captured executor, with finite server
expiry bounding shutdown bookkeeping. Decoded copied Arrow arrays retain their allocation
owner and can remain usable after reader drop and database reclamation; their final drop
releases the memory charge. Buffer ownership is not a database lease.

**Explicit retirement.** Source collection rechecks reachability and removes unreachable
intervals, versions, reference edges and blocks in bounded pages. Retired ranges retain
receipts and fence reopening. Result retirement requires settled attempts, no live result
reads and no retained study/analysis obligation. Its tombstone fences claims, reads,
operation-replay fallback and new admissions before payload removal. Bounded resumable
cleanup removes scientific blobs and derived indexes while preserving lifecycle and lineage
receipts. Study and analysis result retention is withdrawn through their own operations;
there is no implicit elapsed-age deletion of deliberate scientific history.

**Local export.** Exact checked reads finish, the IPC stream finishes and the file is
synchronized before final publication. Interrupted staging remains `.incomplete`; the final
destination refuses an existing file. Metadata records exact source/run/attempt, terminal
class, manifest, interpretation, row range and output predicate. The file owns its copied
bytes and is not an exported database lease or another publication catalog.

### 20.5 Current contracts and schema evolution

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed). Creation and validation-only opening are separate operations; unsupported stores refuse rather than silently migrating or resetting.

The registry generates canonical tables, indexes, row codecs, interpretation and schema
digest. A fresh store creates the complete declarations and current marker in one atomic
transaction. Initialization requires a completed acknowledged response and subsequent
verification of the installed interpretation and schema digest. A zero/missing response,
statement error or marker mismatch cannot establish readiness. Full schema installation
uses the existing bounded structural-activation client role; ordinary inventory/read/open
deadlines remain separate. Uncertainty may be followed by explicit opening; initialization
does not retry DDL or claim success merely because a marker later becomes visible.

`open` performs no DDL. An unmarked partial canonical database or unsupported interpretation
requires explicit operator action. The retired PostgreSQL/Delta descriptor, COPY, migration
and publication APIs have no production compatibility path. There is no automatic importer,
reset or inferred migration from unsupported historical storage.

**Recorded meaning.** The portable semantic-contract witness records consumed fields,
keys, enum domains, extension contracts, checks, invariants and storage policy. Verification
checks its canonical form, digest, support closure and observed encoding rather than
inferring historical meaning from today's registry. `VerifiedRecordedContract`, directional
`ConsumerProjection`, `ExactWriteAdmission` and `MigrationAdmission` are distinct products.
A readable old subset does not admit new writes; an exhaustive consumer refuses unknown
consumed members. Changes to physical meaning, constraints or references require an explicit
transformation. These schema-level distinctions do not supply a retired storage importer or
an automatic native-store migration.

Versioned scientific documents and recorded contracts retain their meaning independently
of database lookup keys. Version checks precede decoding current nested policy or numerical
configuration. Changed preimages receive new frames; readable provenance does not grant
current execution, write admission or cross-build reuse. Unsupported historical execution
must receive explicit current readmission or a typed refusal without invented fields,
qualification or policy. Generated contracts remain declarations, never inferred layouts.

### 20.6 Operational store and durable execution

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed). `pse-operations` owns the thin authenticated native WebSocket client and typed guarded operations under Plan 30 implementation. Native science stays outside database transactions.

One original monotonic operation clock covers admission, queueing, reconnect, dispatch
and complete response handling. Definite conflicts may retry the guarded decision within
that clock; unknown writes settle by original immutable identity. Expired undispatched
work refuses locally. Submitted work retains correlation/admission through bounded
transport drain; socket closure does not prove server abort or native drain. Transaction
expiry and live-authority fences precede the final return. UTC and statement timeouts
do not promise durable commit before the caller's deadline. Complete bounded pages retain
statement completion, exact coordinates and accounted decoded/escaped ownership.

Explicit pre-database catalog creation uses a fresh unselected administrative connection.
SurrealDB 3.3 plans even built-in clock calls at database context level, so initial
namespace/database DDL cannot carry the in-database UTC fence. Its original transport
clock and finite server backstops remain enforced; loss of a submitted DDL response
leaves an unknown catalog outcome. Only acknowledged creation admits context selection.
Atomic schema installation and scientific mutations retain their final transaction fences.

Disk/service identity, immutable receiver artifacts and invocation/database context have
separate lifetimes. Compatible receiver changes do not stop storage; schema creation is
explicit and ordinary open refuses an incompatible interpretation. Python forwards explicit
database selection to the same Rust context validation. Persistent owned user-manager
services consume host admission on startup/restart and cannot relaunch during intentional
parking. Fixed host lanes account independent commands, services and receivers; nested
work borrows its authentic owner and capacity is released only after actual descendant drain.
Storage readiness validates its own charged allocation, actual unit invocation/cgroup,
PID start, finite memory/CPU caps and affinity independently of science placement.
Qualified restart retains that storage charge through the restart delay and only rebinds
after the owned predecessor drains. Missing or stale ownership refuses replacement.
Dedicated timing storage and its separately admitted timing caller retain their selected lane.

Context selection uses a private read-only system account and an exact-record probe,
then restores the write account. Selection and acknowledged reconnect replay cannot
implicitly define missing namespaces/databases or enumerate unbounded catalogs.
Service setup/readmission owns account provisioning; schema installation remains explicit.

**Authority and client.** Short native transactions establish source roots, immutable
operation identity and current cancellation/generation premises under named conflict guards,
including absent-name and set premises. No predicate lock or `FOR UPDATE` assumption grants
that authority. Full-domain decimal sequence/generation and actual expiry checks prevent
wall-clock ordering or an earlier transaction timestamp from renewing stale authority.
Immutable operation acknowledgments settle uncertain effects without rerunning science.

**Run lifecycle.** Registration and fenced claim precede native effects. Renewal and
append require the current unexpired generation and open ingestion gate. Closing freezes
membership; controlled scientific terminal admission compares the frozen manifest and
current authority atomically. Success, partial, failure and cancellation reflect actual
completion, assessment and storage admission. Arbitrary staged rows cannot be promoted
through a general safe success API, and failed storage remains observable when a native
solve returned usable values. A succeeded run cannot acquire another native attempt;
scientific retry after failure/partial requires explicit policy and effect knowledge. A
cancelled run stays cancelled; new semantic execution has a new run.

Recovery obtains current authority, preserves closed observations and attempt history, and
records truthful interruption/worker-loss completion or settles an already committed
operation. A revoked or expired worker cannot append late science. Recovery never infers
success or repeats numerical work merely to repair storage. Study cancellation participates
through indexed point/header association in renewal and append fences.

**Progress and seeds.** Retained progress is appended in bounded typed chunks under the
same ingestion gate; exact counters and event order do not depend on floating projections.
An isolated synchronous native producer may wait for bounded queue admission while the
writer runs, limited by the existing heartbeat timeout or earlier receiver closure. A
callback entered into an async runtime refuses overflow rather than blocking that executor.
Failure cancels the scientific run and remains observable; queue liveness alone cannot
cause indefinite waiting. Original accepted seeds retain layout/profile/data compatibility,
permission and lineage, with bounded chunk coverage and accounted decoding. Reuse selects
the exact compatible scientific occurrence rather than a stage prefix or newest timestamp.

**Studies and workers.** Immutable occurrence descriptors retain authored point policy,
source bindings and starts. Discovery reads bounded truthful summaries. The shared Rust
study policy consumes the candidate and actual immediate predecessor facts; only ready
occurrences prepare their numerical workspace. A guarded claim compares all consumed point
revisions and study cancellation. Selected starts preserve exact predecessor attempt lineage.
Portable qualified evidence supports the shared secant proposal and original corrector;
native factors are reconstructed only for an actual prediction consumer and stay process
local. Finite worker allocations and bounded action discovery are separate from scientific
retry permission; there is no queued-priority or independent global retry-policy owner.

Study summary finalization is effect-free. Its controlled owner atomically admits the parent
result and concludes the study under all-settled and cancellation premises. Competing live
writers are observed; an expired summary writer may be replaced without rerunning settled
points. Generic run recovery leaves an unfinished study header recoverable and does not
invent the summary conclusion.

> Decision: [ADR-0166](../../adr/0166-bounded-parallel-scientific-execution.md)
> (proposed, under Plan 28/30 implementation). The reference primary durable group has
> sixteen bounded lanes sharing one runtime, pool and CPU owner in one supervised
> process; observers add no unbudgeted native assistance. Discovery and local in-flight
> suppression do not grant canonical claim authority. Server, group, observer and
> selected test placements consume one declared finite envelope. The reference pool
> remains 128 GiB, with compatible process/ancestor caps required before launch;
> insufficient placement refuses rather than shrinking the workload. Backend-specific
> isolation uses whole operations and explicit partitions where required. Numeric
> placement and actual readback are L7 prerequisites, not RSS guarantees.

**Operational limits.** The target deployment is authenticated local native WebSocket with native
RocksDB and finite server/native-worker allocations. Backup drains managed work and copies
a stopped coherent database; it is an operator lifecycle, not an online export lease.
These implemented contracts do not claim power-loss durability, remote deployment support,
assembled worker/reopen/backup qualification, capacity measurements or comprehensive scientific
qualification. Current acceptance and remaining work belong to the
[current packet](../../plans/28-surrealdb-unified-substrate.md), not this architecture page.
