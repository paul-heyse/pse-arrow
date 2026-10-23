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

> Decision: [ADR-0172](../../adr/0172-full-analysis-retirement.md) (proposed): an analysis occurrence has a finite immutable creation grant distinct from scientific content.

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

> Decision: [ADR-0215](../../adr/0215-resolved-library-execution.md) — Proposed, maintainer-authorized Plan49 target; implementation/qualification remains at the owning plan.

Resolved body/derived identities use explicit version/domain frames and canonical local roles, with instance bindings separate. Plan49 retires the unconsumed relation canonicalizer and its dedicated API, while retaining shared allocation/admission helpers, canonical float bits, framing and surviving content/encoding identities.

> Supplement: [ADR-0171](../../adr/0171-body-relative-physical-occurrences.md)
> (proposed; scoped target review Accept at Proposed strength).

Compact physical occurrence wires resolve body/node coordinates against complete owner
syntax inventories. Function records use version 2, admitted recipes `pse.admitted-body.v2`,
and runtime admitted-body envelopes/request identity v4. Structural indexes use a dedicated
frame and full equality behind prehashes; historical frames retain their meaning. Rebuild
incompatible disposable products directly, with no legacy decoder or historical-result
obligation. Authored/external inputs and unrelated live ownership remain protected.

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

Proposed ADR-0189 separates scientific caller-code identity from macro external-I/O
assurance. An explicitly reviewed tokens-only contract may omit whole-caller I/O grants
only for the exact guarded owner and complete selected helper/feature/compiler closure.
Paste rejects environment selectors at the actual shared parser; Pest accepts inline
grammar only and disables generator file inclusion. py_literal retains one inline grammar
authority. Generated invocations meet the same owner checks. Stock effectful and unknown
owners retain conservative caller/grammar requirements; a package name or metadata flag
alone grants nothing. Relevant caller code still enters scientific identity.

The proposed ADR-0189 finite preparation operation acquires target-local ownership before dependency setup, bootstraps the capture tool, then refreshes the selected executor-family union once before captures. Its actual writer descendants retain that exclusion; it is not host-wide admission or protection from arbitrary Cargo tools. A first exact executor compilation requires successful non-Fresh compiler evidence. Later Fresh reuse requires preceding compilation within that same operation and unchanged dependency-complete source/configuration/reviewed input/environment/native/absence and original-output premises. Actual Maturin child context remains authoritative; unknown families and terminal/incomplete operations refuse. Build-script output messages are associations, not independent proof of execution in that child. Sealed history and final original-path current verification retain their separate meanings.

The maintainer withdrew the additional environment/provenance proof targets from Plan39 on 2026-10-14: strict all-role capture, source-completeness investigations, executor projections and compiler/launcher/installer phase associations. These targets are eliminated, not deferred; no revisit or future campaign is retained. Ordinary design-phase validation exercises current worker/Python builds, installed workflows and scientific behavior, with explicit Arrow force-validation and existing runtime safeguards. Environment fingerprints retain their provenance meaning; minor changes alone do not invalidate useful functional results. Existing strict producer mechanisms retain their current conservative semantics without becoming an implementation or acceptance obligation for this work.

Deployment preparation owns startup observation before store effects, native join before
scratch release, and only escaped observation data afterward. Strict captures preserve
raw unit associations, consumed volatile inputs and selected artifacts in a sealed historical
view before another role refreshes mutable Cargo paths. Historical reconstruction never
grants current deployment freshness. Python strict preparation captures Maturin's actual
Cargo child and protects that raw realization before installation; independent import and
full-byte admitted transformation checks still establish the installed association.

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
historical meaning and cannot admit new products. The runtime body envelope is version 4 under ADR-0171;
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


> Decision: [ADR-0209](../../adr/0209-qualified-fitting-residual-methods.md) (proposed; Plan48 E07 target).

Current numerical strategy documents use version4 and `NumericalStrategyV4`: a profile names
its real `NumericalMethod` (native backend or residual LM), rather than assigning every method
an NLP backend. `FitProfileV5` includes the requested fitting optimizer. Replay constructors
retain that choice under their current versioned artifact contract. Producers, decoding,
strategy projection and published/Python documents migrate together; no old-field shim
reinterprets historical bytes. Frozen old frame spellings remain historical and do not
admit execution. Physical scalar-cache encoding integrity has its separate nonsemantic role.

IDAS native-step default selection has its own current receiving migration (ADR-0207):
IdasSettings and enclosing dynamic Profile documents use version3, with `DynamicProfileV12`
identities. This includes omission of nested IDAS settings: an older enclosing profile cannot
silently acquire the new automatic policy. Explicit configured requests keep their current
meaning; unsupported historical concrete profile versions refuse without a compatibility shim.
Unresolved study/fit profile requests mean current authored-default resolution on fresh admission;
actual resolved profile identities bind preparation/result reuse, while replay retains its captured
concrete profile. Generic outer requests do not assert a frozen historical native-step default.

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
contracts. Accepted Plan45 RC03 keeps this complete identity and source-revision framing
unchanged while introducing a separately versioned resolver-issued consumed-physical
projection for reusable mathematical products (§14.4). Consumed mathematical premises use `pse.math.physical-premises.v1`; grouped consumer
bodies use `pse.modeling.consumer-body.v5`, and bound views use
`pse.compiler.modeling-view.v6`. The resulting body identity also governs artifact
requests. No historical digest is reinterpreted. Exact misses, complete
candidate populations, visibility and checker/provider interpretation are dependencies,
not only selected positive IDs. Unknown behavior without complete identity disables reuse.
Checked function occurrences retain selected admissions; composed unit scales,
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

**Relation canonicalization retired (Plan49 I24, ADR-0215).** The unconsumed
`pse.canon.v2` engine and dedicated API/tests were removed. No production publication
or reuse path consumed its logical hashes. Runtime identities retain the named
projections in §5.2; canonical admission names exact immutable versions and result
manifests (§20.2). Encoding checksums still identify exact stored bytes without
claiming logical relation equivalence. Source documents retain their text checksums.

Shared allocation/admission envelopes, canonical float bits and framing remain at their
actual consumers. The reserved `pse.snapshot.v2` frame still has no aggregate snapshot
implementation. A future relation equivalence capability requires an explicit owner and
versioned contract; stored-byte hashing is not an equivalent replacement.

A hash identifies input; it does not establish admission, validity, applicability or
scientific equivalence. Package invariants and receiving permission retain their own owners.
Removal is Implemented; survivor qualification remains with Plan49 TOOL07/QUAL.

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

Protection acquisition and cleanup have a completion owner established before submission.
Cloned followers retain the same exact issuing-store protection; explicit final finish waits
for release acknowledgement, and final-owner drop schedules cleanup through its retained
executor. Abandoning an outer future cannot release another follower's live protection or
resurrect an already released acquisition. Process loss and failed cleanup still rely on
server expiry. Selected modeling acquisition resolves only its demanded logical closure;
complete inventory uses grouped bounded reads with stable ordering and complete premises.

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

> Decision: [ADR-0214](../../adr/0214-surrealmx-canonical-store.md) (proposed; maintainer-authorized Plan49a target). Select pinned SurrealDB3.3.4 with persistent MX1.0, explicit SSI, synchronous AOL/EveryAppend, finite capacity/transaction age and bounded gRPC. gRPC sessions are scoped to their actual physical connection and reclaimed on disconnect; new connections restore acknowledged setup rather than reuse dead-session IDs. Fresh initialization regenerates existing data; historical import/backfill and old-data rollback are not required. Newly generated data retains durable receipts, protected payloads and backup/restore/recovery obligations. STO01–STO08 own implementation and qualification; prior RocksDB/WS evidence does not qualify this target.

One authenticated loopback SurrealDB database holds canonical source revisions, compilation
products, execution receipts, scientific result descriptors and analysis lineage. Bulk
result/artifact/seed payloads reside in sealed private files; persistent MX is the selected
replacement catalog backend. The registry declares their contracts; `pse-operations` supplies
typed operations and generated native structural functions. Scientific computation and bulk
payload staging stay outside short guarded admission transactions. Ordinary execution
retains the actual scientific outcome automatically. Explicit ephemeral execution keeps its
observations in process; a durable storage failure never falls back to ephemeral execution.

### 20.1 Canonical durable relations

> Decision: [ADR-0187](../../adr/0187-sealed-payloads-and-strong-result-snapshots.md) (proposed; maintainer-authorized Plan38e target).

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

**Sealed payload results.** Registry-admitted Arrow rows use completed uncompressed IPC files in the selected private state's `payloads/<payload_incarnation>/<database>/` root. Compatible batches share one file only within an attempt, relation, schema and interpretation. Packing targets 16 MiB and has a 32 MiB hard encoded-file bound; demanded acknowledgements and live-prefix boundaries flush immediately rather than waiting for unrelated work. Logical descriptors retain original ordering/coordinates and bind unique file instance, encoding digest, batch index, schema and interpretation. JSON seed/completion owners retain their exact declared codecs through the same opaque file tier. File framing/layout/integrity admission precedes decoder allocation; neither file completion nor a submitted write is a catalog acknowledgement. Selected qualified coverage belongs to the Plan39d Outcome (§24.2).

Sparse scalar indexes and dense output descriptors reference the original IPC batch and
row/field coordinates; they do not replace it with a second scientific payload. Finite
numeric projections serve predicates and are verified against authoritative values.
Missingness is not zero, and signed zero and exceptional diagnostic bits retain their
declared meaning. Dense trajectory groups preserve symbol/sample identity and original
coordinates; half-open relation row ranges select the recorded stored order. Output indexing currently
covers solve variables/constraints, fit parameters/observations and simulation sample
values. Unsupported scientific output fields refuse explicitly.

### 20.2 Exact coherent publication

> Decision: [ADR-0187](../../adr/0187-sealed-payloads-and-strong-result-snapshots.md) (proposed; Plan38e target).

**File publication.** Reserve the original intent and unique physical instance before filesystem work. Finish, validate, synchronize and publish without replacement before transactionally attaching the exact descriptor, references and receipt under current permission. Unresolved intents retain their physical objects. A collecting instance never revives and its path is never reused; delayed unlink cannot target a replacement. Deduplication checks bytes as well as digest/encoding. Semantic content, occurrence, interpretation and physical placement remain distinct. Logical descriptors omit absolute roots and storage incarnation; only closed restore rehomes physical placement, while owner capabilities bind the selected incarnation.

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

**Retained exact inputs (ADR-0182, proposed; Plan37 target).** Durable studies resolve
and admit explicit external starts before activation, including scientific compatibility.
Their independent artifact obligation survives producer retirement and supplies dispatch,
restart and retry. Within-study exact-input deduplication preserves occurrence role and
provenance. Admission-first survives retirement; retirement-first refuses without fresh
fallback. Preparation remains outside the visibility transaction and value-only edits
retain the existing structural reuse contract.

An immutable analysis records method, configuration, interpretation and all selected source
revisions. Result-input edges name the exact run, attempt and manifest. Dependency analysis
separates original numerical incidence from conservative execution support; retained-result
analysis preserves existing sensitivity evidence. Different method or input identity yields
a different scientific analysis meaning. Each explicit creation has a fresh occurrence, whose
checked key binds the primary authority incarnation, nonce and original absolute admission
expiry. Same-operation retries preserve that intent; same-content recreation obtains a new
occurrence. Surviving headers retain exact source attribution; retired occurrences retain no
header or lineage payload (ADR-0172).

Names, metadata, hashes and graph reachability alone do not certify validity, execution or
equivalence. Scientific completion and result meaning remain owned by
[§19](workflows-and-results.md#section-19).

### 20.4 Reproduction, reuse and retention

> Decision: [ADR-0177](../../adr/0177-exact-conflicts-and-settled-retirement.md) (proposed amendment; Plan 36 P-B1/P-B7).

Source creators register/check exact reclaim authority and reclaimed intervals, install
their exact root/pin and write a creator fence atomically. Collection registers the creator
fence before scanning roots/pins, registers affected exact authorities, and publishes
reclamation atomically with its reclaim fence. Both commit orders invalidate the stale
opposing decision; rebuilt creation refuses reclaimed history. Pure active-attempt root
removal registers reclaim authority and its exact root without writing the creator fence;
it adds no dependency, and the durable run root retains the same source until run
retirement. A collector can conservatively retain a removed root; collector-first
invalidates stale release through reclaim authority. Result retirement uses the
analogous result-creator/retirement pair. Result pins, analysis input edges, summary-study
headers and point/run associations all participate, including association installation on
an existing run. Absence at registration is not permanent absence unless the complete
association is immutable there and every later creator verifies it. Continued reads and
renewal check exact live pin plus reclaim/retirement authority, and result manifest where
applicable; expiry/release cannot revive. Grant settlement closes the original authority
incarnation/nonce/deadline through a monotone floor before bounded residue collection.
Unresolved effects and run lineage retain their borrowers. Production deletion remains a
separately authorized operation; implementation race controls remain Plan 36-owned.

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

Source admission/renewal and reclamation retain their finite exact permission guards. The Plan38e result-read target instead owns a nonexpiring exact manifest ticket protecting metadata and every future unread file until actual drain. Reader admission and retirement/collection share transactional arbitration. File owners register against store/storage incarnation and Linux boot, PID namespace, PID and process start; supervised workers also bind their invocation. Children register before using payload capabilities. Reaping requires authenticated exact death/drain, never silence or lease expiry; unknown remains protective. Plans, partitions, retries and read-ahead retain the ticket. Close seals admission and joins file/transport/release work; failed release remains observable and protective. Decoded copied arrays retain their real allocation owners independently of durable protection.

**Independent scientific artifacts (ADR-0182, proposed; Plan37 target).** Scientific
owners admit small copied seed capsules and selected artifact-owned bounded IPC copies.
Occurrence identity, original scientific identity and encoding digest have distinct roles.
Original units/basis, coordinates, case/model/source interpretation, assessments and
partiality travel with sufficient content. Actual final endpoints carry mode/integral/input
interpretation; trajectories carry event/transition/coverage context; local analyses carry
validity, conventions and required upstream context. Missing sufficiency refuses promotion.
Persisted graph analyses retain their existing occurrence owner and reference typed
artifact block/row/field evidence; producer IPC strings alone cannot grant active access.

A sealed creation intent preserves original source authority, nonce, finite expiry and
exact request. Each file is sealed independently with its own codec/schema identity. Already sealed artifact metadata attaches in groups of at most 64 rows and the existing index/message envelope; acknowledgments must contain the exact ordered original keys. Unknown/lost acknowledgment leaves staging unsettled and cannot authorize activation or a renewed replay. Only short metadata transactions are grouped; no transaction spans scientific execution. Staging maintains original producer protection; short atomic activation
registers retirement/source authority and installs verified content, independent roots and
initial obligations. Existing source-retention creation floors settle delayed creators at
final collection. Study, explicit-user/project, analysis, protected-read and audit obligations
release independently. One release cannot invalidate another retained owner's future reads.
Collection is bounded/resumable and unknown creation remains protective. Neither copying
nor a digest invents numerical permission or numerical continuation checkpoints.

**Explicit retirement and detailed history (ADR-0182, proposed; Plan37 target).** Source
collection rechecks reachability and removes unreachable intervals/versions/edges/blocks
in bounded pages. Retired authority fences reopening. Producer payload retirement closes
producer reads, while independently retained artifacts remain usable. Payload release and
run/study history release are separate explicit owner operations; terminality/age does not
authorize deletion. Audit and unsettled effect/grant/creation obligations retain their detail.

Permanent run/study fences dominate all mutation, early receipt return, absence fallback,
readiness and recovery. A retired study ID refuses even with a different run. Existing opaque
operation IDs retain compact server-issued authority/run/attempt/study/point/kind/disposition
locators, installed atomically with the original effect. Transactional owner/fence checks
precede receipt readback. Released settled history returns retired/history-unavailable;
compact locators cannot reconstruct a successful closure or authorize native redispatch.
Live/unsettled requests retain exact comparisons. Bounded collection releases occurrence,
request/result and header detail after actual obligations settle. Compact identity state can
still grow with issued IDs; logical collection does not establish returned disk space.

**Full analysis retirement (ADR-0172/ADR-0182, proposed).** An analysis header owns staging,
active and retiring phases. Creation atomically validates exact source revisions and finite
`ArtifactRead` grants, installs source roots, and attaches independent analysis obligations
to each selected artifact under existing source-retention and artifact conflict guards.
It never recreates generic source protections. Surviving analysis inputs retain their
artifacts independently of producer run payloads. Mutation, activation and individual
copied-page acquisition register the existing header in a writable transaction and refuse
absent or retiring authority. Returned arrays own their bytes and do not promise future
page access; original run/attempt locators preserve provenance without granting reads.

Retirement closes the header and deletes bounded pages of edges before nodes, then releases
its exact artifact obligations while removing input rows, source roots and temporary
metadata; other artifact owners remain independent. Progress lives only in the retiring
header. Original creation can still be dispatched without a header. Final settlement therefore registers the
exact header even when absent and the existing primary source-retention guard, after the
immutable creation window closes. It advances that guard's monotone
`analysis_creation_closed_through` scalar and removes the drained header atomically. Earlier
registered creators either committed before settlement or fail commit validation; later
creators refuse the closed expiry. The occurrence key cannot be reused with a renewed expiry.
This settles effect authority, not socket emptiness or completion of server tasks.

The scalar is constant-size admission authority per existing problem, with no analysis key,
operation, lineage or retirement list. All guard writers preserve it. Missing guards refuse;
a reset/restore cannot reuse the same addressable authority incarnation with a cleared floor.
A fresh valid occurrence remains creatable, including identical scientific content. An absent
header alone cannot establish settlement of an unknown original grant. After full settlement
under the same source-authority incarnation there is no analysis-owned header, graph,
input, root, protection, guard, admission, cleanup, retirement or audit payload. Partial
cleanup is restartable from its retiring header.

**Restored historical settlement (ADR-0187, proposed).** Restore fences the old creation
authority while preserving admitted analyses and artifacts. Cleanup validates the exact
existing immutable header; it never grants fresh creation permission or advances the new
authority's unrelated expiry floor. A drained historical header remains a terminal retiring
receipt for exact lost-response settlement. It admits neither reads, activation nor new
borrowers, and retains no graph, input, root, block or file obligation. An absent unknown
old header is not proof of settlement. Historical protection cleanup likewise requires
its exact persisted grant or admitted read/header association and no live strong ticket.
It retains a released terminal grant receipt for retry without advancing the replacement
authority floor; new acquisition and renewal still require current authority.

**Explicit selected results (ADR-0187 proposed amendment; accepted Plan45 RC04).** Full snapshots preserve complete frozen relation/range inventory and synchronous access. A distinct selected reader receives immutable output/range predicates before discovery and uses bounded selected pages under exact read protection. Explicit asynchronous selected snapshots freeze the complete original selected universe, never an unread tail; their native providers restrict that universe before residual filters, limits and projection. Full and selected paths share one protected codec/admission owner. Python selected streams retain their existing meaning. A full reader consumes its frozen inventory without a second storage walk. Selected readers discover only their selected universe; independent full readers and actual IPC imports retain complete receiving checks.

**Protected evidence composition within ADR-0182.** Analysis and standalone promotion share one protected selection and bounded byte/context-bound scientific facts produced during the copy. Provisional member coordinates become usable graph evidence only after exact artifact/member acknowledgments, final locator binding and independently retained artifact input/read authority. Unknown acknowledgments keep original intent and unsettled protection; independent reopen still performs receiving admission. Activation acknowledgments are checked against the generated whole-row request with only the permitted active lifecycle transition normalized. Temporary equality copies are prepaid. Client evidence qualification uses bounded indexes, and each server graph page resolves distinct endpoint/artifact/block facts once while preserving every edge predicate and retirement lock. Standalone artifact copies remain independently owned; no shared-file ownership transfer or arbitrary projected-schema admission is implied.

**Plan38e file lifetime target (ADR-0187, proposed).** Artifact occurrences, independent analysis obligations, historical runs, unresolved publication intents and exact read tickets retain their sufficient file closure independently. Occurrence retirement, history-detail release and physical collection are separate transitions. Original creation/replay floors fence late acquisition; collection never infers authority from age, size or content equality. Native DataFusion providers consume exact frozen file lists under these tickets and the existing SQL/function/resource context; a directory listing is not a snapshot.

**Physical row publication.** Each physical relation is encoded once into a bounded
memory spool or DataFusion-managed spill. The identity hash and canonical writes consume
those same exact bytes. Warm exact-owner cache hits precede encoding; changed exact rows
produce their own receipt. Accepted Plan45 RC06 (proposed ADR-0187 amendment) selects
bounded deterministic groups within each relation: at most four original blocks and
twice the canonical per-block payload bound. Immutable block order/length determines
groups; memory pressure refuses rather than changing them. Prepaid payload/header
ownership is distinct from the existing encoder/SDK scratch and spool reader. The
grouped-v2 producer namespace binds this policy, content identity and first/end original
ordinals without reinterpreting earlier per-block records or changing payload/scientific
identity. Existing source planning/staging/CAS and immutable request receipts settle
original-intent replay and unknown ACK. Only the exact returned group receipt advances
the parent. The final per-relation revision and separate context receipt preserve
selective replay and package/context association. Fitting relations activate coherently
in one group; larger relations retain deterministic group closures and existing payload
staging crossings. Atomic whole-relation publication of arbitrarily large spilled input
is not claimed. Group and context edits admit their concrete future before construction
and use separate non-inline construction/poll boundaries on the original task, preserving
activity, cancellation and transport ownership without increasing the runtime stack.

The optional physical cache charges immutable exact receipt/source payloads and concrete
worker/pool identity. It excludes caller-specific protection/release owners and store clones.
Every returned holder obtains fresh independently charged protection and validates current
worker/pool/model/source identity. Warm immutable payload reuse does not reduce active grants;
escaped protection survives cache eviction and uncertain release remains charged. The cache
budget is unchanged; capacity is not grounds to undercharge. Empty input still prepays context
transport. Earlier protection-heavy cache observations do not describe this payload-only owner.

**Local export.** Exact checked reads finish, the IPC stream finishes and the file is
synchronized before final publication. Interrupted staging remains `.incomplete`; the final
destination refuses an existing file. Metadata records exact source/run/attempt, terminal
class, manifest, interpretation, row range and output predicate. The file owns its copied
bytes and is not an exported database lease or another publication catalog.

### 20.5 Current contracts and schema evolution

> Decision: [ADR-0216](../../adr/0216-project-and-evidence-foundations.md) (proposed; maintainer-authorized Plan51a target).

Project/research relations extend the canonical installation. A separately disposable tooling
catalogue has its own generated schema, codecs, interpretation and issuing identity on the same
selected service. Explicit privileged initialization is separate from validation-only opening;
database-scoped reader/importer credentials cannot mutate the canonical database. Tooling opening
requires no scientific module, native receiver or payload-placement attestation. Evidence-only
schema changes do not alter canonical installation compatibility.

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

**Explicit design-phase cutover (ADR-0182, proposed; Plan37 OP01 target).** The existing
lifecycle owner regenerates selected disposable state from current authoritative definitions
and fixtures without a preserved-input export or old-data survival gate. It closes admission,
drains authentic activities/workers/transports/holders, refuses unrelated or unknown borrowers,
and stops the owned server before selected physical replacement. New database identity,
issuing authority and credentials are fresh; source documents and unrelated workspaces are
outside reset authority. Initialization and validation run under closed admission; only the
exact live initializer may write. Failed/interrupted phases stay closed for explicit recovery.
Dispose only exact replaced owned state and explicitly activate/readmit the validated target.
Ordinary opening never resets, repairs or activates an unsupported store.

**Frozen forward rollout (ADR-0182, proposed; Plan37 C2 target).** Generated definitions and
codecs remain registry authority. Producer-owned frozen steps are immutable; changes append
forward steps with exact hashes/predecessors. Beta SHA is distinct from the repository schema
digest. The exclusive maintenance owner fences participating application/tool writers. Its
fixed schema settler binds original pending nonce/inputs/step identities and actual generated
postconditions, after authentic prior SQL/service drain. Unknown data needs its domain owner's
exact atomic witness/postconditions; unsupported/ambiguous steps remain unknown. CLI exit,
repair, lock expiry and local drain alone do not establish effect settlement.

Canonical maintenance validation is separate from activation. Activation verifies installed
definitions, codec/scientific obligations and exact settled operation before publishing the
marker. Compatible forward activation preserves issuing identity and checks its predecessor;
fresh activation mints it. Atomic direct fresh creation remains supported. Ordinary open is
validation-only. Neither the beta tool nor a supplied settlement file grants activation authority.

Maintenance keeps ordinary canonical admission closed and drains registered holders and the
ordinary server. Its live lifecycle owner may run a temporary authenticated loopback server
using the declared binary, storage configuration and charged allocation. Native SQL/export/import
clients connect remotely and never independently open the server's persistence directory.
This listener establishes no ordinary readiness and starts no scientific workers.
Only fresh initialization bootstraps ROOT/VIEWER under its exact pending lifecycle owner;
credential recovery never provisions missing accounts. It has no automatic restart. Physical copying,
initializer handoff and ordinary readmission require acknowledged stop and actual descendant
drain. Owner loss or uncertain shutdown leaves admission closed and resource ownership retained;
recovery first settles the exact prior service invocation. Recorded ROOT credential candidates
are probed through fresh connections without selecting a missing catalog context or provisioning
credentials. Loopback binding alone does not fence holders of privileged credentials.


Startup configuration stays bounded. Frozen payload inventories, failed-target owner/placement
records and retry placements belong to one immutable private maintenance record, referenced by
its exact path, extent and digest. Its stable operation identity and original source
namespace/database/incarnation survive initializer-owner and target changes. Publish the record
before its configuration pointer; retire only an exact predecessor no longer referenced by the
current configuration. Missing or changed evidence refuses recovery and deletion. Retain the
current record through payload cleanup, database disposal and listener drain; durably clear
pending maintenance before retiring it. Unknown crash-window files remain preserved.

`recover-maintenance` resumes only the durable phase recorded by that owner. Current-version
restore verifies its backup inventory; clean rebuild verifies current initializer authority.
Both observe account/database effects before retrying uncertain operations. Restore publishes its fresh closed identity before copying any backup
bytes; the old startable profile is never copied. Unknown files, changed inputs, unknown
catalog owners or an unvalidated initializer leave admission closed. Disposal follows
acknowledged current initialization and targets the exact replaced database.

A same-interpretation restore also cannot expose copied creation authority at an old
address. Before readmission, its isolated maintenance owner preserves admitted analysis
headers, graphs, roots and independent artifact/file obligations while rotating source
incarnations and retaining closure floors. Unissued old creation requests remain fenced;
historical reads and exact cleanup retain their admitted meaning. The owner removes the
old copied database after establishing the fresh database identity. Existing authenticated
clients must not retain access to a copied old namespace/database. A changed local supervisor
instance identifier alone does not fence database requests.

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

> Decision: [ADR-0220](../../adr/0220-selected-closure-and-fresh-authority-exchange.md) (proposed; maintainer-authorized Plan51e–51g target).

Selected logical project closure has one resolver over original source, research, study,
result and artifact owners. Manifest v1 records source revisions, physical/provider premises,
selected dataset/calibration/publication ancestry, study definitions, selected result ranges,
artifact block coordinates, codecs, checksums, copied extents and explicit omissions. Original
protection spans copying and verification; the manifest publishes last. Shared contained-copy
primitives also serve diagnostic packs, while each format keeps its own interpretation.

Independent readers validate contained paths, actual member bytes, exact artifact coverage and
original scientific IPC lineage/finalization. They do not follow ambient paths, connect to the
origin, install providers or execute science. Array/registry admission remains at the selected
scientific consumer. Explicit descriptive export uses narrow metadata projections without
hydrating completion, point outcomes or payloads; it requires no physical admission. Its missing
source/provider/data premises remain execution-required omissions. Offline completeness and
executable completeness are separate facts; descriptive packs cannot be imported as executable.

Fresh-authority import binds the original operation, receiving issuer, exact bundle digest and
fresh destination coordinates before effects. Ordinary source/physical admission and streamed
artifact staging precede original source/project guarded activation. A source checkpoint can
exist while project availability remains false: project/research/context reads and fresh mutation
guards stay fenced until copied evidence is active and the original import finishes. Original
source, artifact and acknowledgement primitives remain available for settlement. Completed
imports reopen from receiving artifact custody after the supplied pack is removed, carrying
owned physical context for ordinary typed draft admission and supported new studies.

Exact artifact retries finish an already active original even after its creation allowance has
expired. Inactive expired originals must settle and be collected by their original owner before
a deliberate later copy occurrence; bounded retired-original receipts prevent an old recovery
request from aborting a newer occurrence. No retry renews the original clock. Imported historical
executions, results, studies and publications remain origin-qualified evidence; they never mint
receiving producer receipts, terminal authority, live protection grants or seed eligibility.

Lifecycle selection/explanation observes actual owners and bounded blockers; it grants no release
right and does not infer physical reclaimed bytes. Explicit release rechecks the issuing store,
exact revision/generations, current roots, borrowers and in-flight effects. Noncurrent raw-draft
detail can retire while retaining immutable provenance and a discoverable incomplete summary.
Original history-retirement receipts retain exact revision/generation for lost-reply settlement
after collection. Evidence-index deletion does not release original artifacts; evidence keys
alone are observational and cannot substitute original artifact intents.

Application backup manifest v4 extends the existing quiesce/drain/stop and selected external-file
closure with bounded frozen application catalogue/reference inventories. Read-only inspection
and restore share completeness verification. Restore compares the original application inventory
before fresh authority/credentials and original semantic validation; an engine snapshot alone is
not application recovery. An incomplete pack refuses before a destination is created. No live
engine-directory reader or automatic repair from newer source/results is supported.

Native/offline logical tools and explicit export/import with exact-parent merge are selected for
the sole operator. Browser UI/persistence, live synchronization and application encryption are not
adopted. Local filesystem access control and explicit custody are the supported protection
boundary; host volume encryption is not established. Revisit a named browser operation, concrete
multiwriter/automatic-exchange workflow or confidential exchange/at-rest requirement respectively.
A confidentiality selection must cover catalogue/credentials, payloads, sources, measurements,
packs, diagnostics/traces/crash dumps, temporary files, keys and backups, including key recovery.
Operator commands and recovery limits live in the [substrate guide](../../dev/surreal-substrate.md).


> Decision: [ADR-0216](../../adr/0216-project-and-evidence-foundations.md) (proposed; Plan51a owns implementation and targeted acceptance).

> Decision: [ADR-0217](../../adr/0217-project-editing-and-historical-context.md) (proposed; maintainer-authorized Plan51b implementation, targeted evidence at its Outcome).

Stable project/branch identities are distinct from mutable presentation names. Draft heads select
immutable raw versions and savepoints; admission is explicit. Object-based merge uses exact base,
target and proposal selections and the existing source CAS. Undo creates a compensating revision.
Historical engineering context has UTC half-open effective intervals, explicit correction ancestry
and a project-scoped recorded cut independent of source or membership generations. Ambiguous
selection refuses; tombstones do not silently resurrect a superseded value. Recorded studies/fits
retain their exact selected context and actual scientific source/physical closure. Current hidden
or corrected context does not rewrite that original consumption. Payload and scientific closure
release remain at their existing owners.

Exact project selection freezes issuing authority, project, source problem/revision, admitted
source identity and physical closure identity. Reopening it validates those original premises;
later branch movement or presentation rename cannot substitute a new source. Metadata pages
are bounded live navigation, not an historical snapshot. Draft visibility and savepoint removal
do not release immutable versions or their protected artifact custody. Explicit local admission
records version-linked diagnostics without changing a branch. Typed draft recovery authenticates
the original store and canonical base before rebinding a local source token.

Merge decisions bind complete source inputs, explicit resolutions and ancestry to the original
publication intent. Foreign issuers refuse even if caller-selected revision keys coincide.
Nonoverlapping logical edits compose; overlapping document edits still need a choice. Derived
whole-package inventory checksums are distinct from authored manifest fields. The existing loader
recomputes that checksum only from a complete original byte inventory; missing IR preimages
retain the existing absent-text convention. Authored manifest differences still refuse. A private
admission proof retains original text only when its complete declaration and fit inventory is
the selected composition's actual preimage. Otherwise the preview reports unavailable composed
text and original inputs remain in retained ancestry; the public IR/text admission guard is
unchanged. Undo publishes a new revision and preserves old recorded consumption.

Historical records root both original authored and physical source closures. Effective instants
are signed UTC microseconds and intervals are half-open. A transitive correction dominates its
ancestors only within its own interval; unrelated overlaps refuse. A recorded cut freezes the
project journal independently of source and membership generations. Exact selected context flows
through study preparation, retained workers, point definitions, standalone fit/horizon descriptors,
lazy occurrences, replay and review. A changed scientific delta clears inherited context unless
the original source and physical identities still match. Reopening history loads protected original
closures rather than receiving current documents; released or unavailable closure refuses.

A project organizes exact admitted source and research references. Each writable branch names its
own canonical source problem; the existing problem head remains the sole mutable source head.
Project publication checks branch/draft generations and source head and publishes its exact
operation acknowledgement in the same source-activation transaction. Raw draft custody can retain
malformed or empty text but grants no scientific admission. Research references retain their
actual source/result closure through its current owner; no fitted value or result authority is
minted by a catalogue link. Bulk staging stays outside short metadata transactions.

> Decision: [ADR-0215](../../adr/0215-resolved-library-execution.md) — Proposed, maintainer-authorized Plan49 target; implementation/qualification remains at the owning plan.

Generated guarded functions own operator domain mutations and bounded resumable reclamation. Privileged install/restore/activation remains a separate phase; existing generated data can be regenerated, while new data retains protected references, durable receipts and recovery.

> Decision: [ADR-0211](../../adr/0211-bounded-worker-publication.md) (proposed; bounded publication ownership target).

Compute/publication scheduler separation does not weaken durable completion. The bounded
old lane retains its source scope, heartbeat, fence, result/allocation and native-close
owner through actual ACK/error and drain. At most W Running and W Publishing owners use
precharged metadata and actual retained-result leases. Lost/uncertain ACK remains uncertain;
cancellation joins every issued owner, and only terminal outcomes advance processed state.

> Decision: [ADR-0181](../../adr/0181-separate-caller-activity-and-physical-connection-lifetimes.md) (proposed; Plan 37 RC04 target).

**Caller, activity and physical lifetime.** Logical runtime release is idempotent and
closes that caller's admission; aliases share the caller identity, independently created
runtimes do not. Admitted activities retain continuation authority through heartbeat,
progress drain, publication, cancellation and exact settlement or explicit recovery
transfer. An idle alias is not an activity. Direct operations and activation cannot bypass
the shared physical gate. A protected reader transfers activity ownership to its tracked
release before appearing drained. Ordinary release neither clears shared caches nor
disconnects drivers.

The deployment/fixture owner atomically refuses physical shutdown while actual activities
or readers remain, leaving their RPCs usable. Otherwise Open becomes Closing before new
admission is excluded and both ordinary/activation drivers drain without holding the gate.
Interrupted or partial teardown remains Closing with authentic continuation/recovery
ownership; completed close is terminal. A new validated owner is required to reopen.
Local client drain, native join and remote effect settlement remain separate obligations.


> Decision: [ADR-0180](../../adr/0180-demanded-result-coverage-and-acknowledgments.md) (proposed; maintainer-authorized Plan 36 target).

> Decision: [ADR-0177](../../adr/0177-exact-conflicts-and-settled-retirement.md), [ADR-0178](../../adr/0178-retained-preparation-and-source-intents.md) (proposed amendments; Plan 36).

Cancellation alone writes its exact authority; claim, start, append, heartbeat, observe,
close, seal and finalize register it before their decision. Checked immutable problem and
study association travel with capabilities, while server effects verify current identity
and generation. Local turns follow actual written domains in stable order and release
before hydration, native work and retry backoff. Definite conflicts rebuild the complete
decision under the original clock. Uncertain dispatch settles its original receipt/fence
without blind application replay or renewed identity/expiry. Valid earlier effects remain
in the retained prefix. A complete cancelled export preserves original assessment under
its cancelled outcome; incomplete export grants no scientific or seed permission.

Immutable body content, complete interpretation/dependency receipts and store-issued
product associations are distinct from eligible roots. Shared material retains one charged
owner, full equality and collision checks; exact acknowledgment verifies the original
association/root. A live preparation engagement owns synchronous once-per-engagement
settlement, exact protection and already-issued uncertainty through drain. The fresh Plan37
target (ADR-0182) replaces historical substrate activation/backfill and
compatibility readers. Install all current consumers together against the generated v5
contract, regenerate scientific evidence, and retain current scientific source-version
semantics. This is not permission to reinterpret old substrate payloads or automatically
resume other plans' qualification campaigns.

> Decision: [ADR-0173](../../adr/0173-seal-seed-eligibility-before-discovery.md) (proposed). Seed discovery separates terminal lifecycle from complete-original scientific inventory permission.

A controlled seal explicitly declares seed eligibility. Only complete acknowledged original
scientific inventory can issue the bounded attempt-level witness, bound to the frozen closed
manifest in that same seal. Recovery, summaries and unavailable cancellation prefixes issue
no eligibility. Candidate discovery filters this witness, exact compatibility and retirement
before its deterministic bounded cutoff. Protected acquisition independently verifies the
chosen seed's inventory membership and per-step scientific permission; discovery does not
confer future storage permission. Old terminal records are never upgraded by inference.


> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed). `pse-operations` owns the bounded authenticated gRPC client and typed guarded operations; ADR-0214 selects its persistent MX integration. Native science stays outside database transactions.

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

Canonical connection construction consumes storage identity, credentials, interpretation
and admission only. Native allocation and receiver association are interpreted together by
runtime deployment composition for the selected database, from the same authoritative
declaration. Storage-only access does not validate unused native policy. Managed execution
still requires finite native allocation, the actual receiver association and fresh admission;
configuration selection alone is not execution authority.

Host readiness distinguishes compatible ready, owned startup, actionable absence,
unavailable observation and authenticated mismatch. Compatible borrowers join the startup
owner under their original clock; uncertainty or mismatch cannot authorize restart.
Actual mutations retain lifecycle exclusion and authentic descendant-drain checks.
Allocation admission reconciles ended owners without starting unrelated parked services.
An explicit store demand resumes only its selected service through the current allocation
owner; independent storage and reference storage retain their respective admission rules.
Recovery compares immutable receiver content and exact permitted installation mapping,
including intermediate-directory associations, permissions and the external interpreter.
Service invocation, readiness and admission are re-established for the restored incarnation.

Context selection uses a private read-only system account and an exact-record probe,
then restores the write account. Selection and acknowledged reconnect replay cannot
implicitly define missing namespaces/databases or enumerate unbounded catalogs.
Service setup/readmission owns account provisioning; schema installation remains explicit.

**Authority and client.** Short native transactions establish source roots, immutable
operation identity and current cancellation/generation premises under named conflict guards,
including absent-name and set premises. Explicit MX serializable snapshot isolation tracks
point and range reads in writing transactions. Named guard writes remain required for
lifecycle decisions and external-effect authority. Analysis settlement additionally consumes
point-record `FOR UPDATE` registration, including absent keys. This is optimistic commit
validation rather than blocking lock acquisition; actual generated-query races qualify the
selected adapter. Prior RocksDB controls retain their historical scope (ADR-0172). Full-domain decimal sequence/generation and actual expiry checks prevent
wall-clock ordering or an earlier transaction timestamp from renewing stale authority.
Immutable operation acknowledgments settle uncertain effects without rerunning science.

**Generated physical lifecycle (ADR-0214).** The registry declares ingestion,
closed-awaiting-manifest, closed-with-manifest, terminal and terminal-compacted phases.
The compacted phase records irreversible completion-payload removal while retaining
terminal metadata. Exhaustive projections preserve separate public attempt/study
observations; terminal observation alone does not prove payload drain or manifest
availability. Native constraints backstop legal phases, immutable identities, exact
record keys, full-u64 generations and completion shape. A synchronous row event freezes
the terminal seed pointer without deciding scientific eligibility; guarded functions
retain cross-record authority and synchronous readiness.

Record identities serve exact lookup and bounded whole-table cursor ranges. Compound
selection indexes remain. Attributed, receipt-owned edges retain classic identity and
endpoint enforcement, with finite inline caches and high-degree fallback. Study ingestion
installs all nodes before bounded dependency edges, then activates only a complete graph.
Problem mutation pacing retains the common creator/reclaimer conflict dependency across
staging and close; run-specific renewal/cancellation and distinct problems remain separate.
Visible-run sequence publication stays serialized through its commit.

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

Acknowledged study cancellation may interrupt export after native work has joined. Exact
typed writer refusal permits the existing recovery/reconciliation owner to freeze that
attempt's prefix and settle Cancelled with scientific completion unavailable. A complete
export preserves its original assessment under existing cancellation-race semantics.
Unrelated transport/export failures remain errors. Durable records and study facts consume
the acknowledged stored completion, never provisional in-memory success. Renewal continues
through completion export and normal reconciliation, then its joined outcome is inspected
before terminal admission. Immutable operation receipts settle uncertainty without science
rerun; incomplete observations confer no seed, activation or scientific usability.

**Progress and seeds.** Retained progress is appended in bounded typed chunks under the
same ingestion gate; exact counters and event order do not depend on floating projections.
An isolated synchronous native producer may wait for bounded queue admission while the
writer runs, limited by the existing heartbeat timeout or earlier receiver closure. A
callback entered into an async runtime refuses overflow rather than blocking that executor.
Failure cancels the scientific run and remains observable; queue liveness alone cannot
cause indefinite waiting. Original accepted seeds retain layout/profile/data compatibility,
permission and lineage, with bounded chunk coverage and accounted decoding. Reuse selects
the exact compatible scientific occurrence rather than a stage prefix or newest timestamp.

> Decision: [ADR-0214](../../adr/0214-surrealmx-canonical-store.md) (proposed;
> maintainer-selected RC22). A pinned SDK/ABI and content-addressed immutable Surrealism
> artifact join the schema activation identity. Install the read-only module from a
> restricted local bucket before activation, and refuse a missing or mismatched artifact.
> Finite scope, controller, linear-memory and strict execution-time limits apply. The
> module has no arbitrary SQL, network, scripting or KV capability. Trap or timeout aborts
> the enclosing transaction; an uncertain reply retains the exact native operation intent
> and receipt reconciliation under its original deadline. Replace external scientific-scope
> transfer and stale-decision application after the in-transaction controls qualify.

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

Worker loss retains an unknown scientific effect. Recovery can establish absence only
after fencing the exact writer: an immutable summary-only capability has no native
dispatch authority, or a refreshed same-attempt point snapshot proves native work never
started. The native-start marker and recovery share the execution-run fence, so a start
that committed first must be visible and a stale start cannot commit afterward. A started
but lost occurrence retains uncertainty; cancellation alone supplies no absence proof.

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

**Operational limits.** The target deployment is authenticated loopback gRPC with persistent
MX1.0, synchronous AOL/EveryAppend, periodic snapshots and finite server/native-worker allocations. Offline backup closes admission and excludes GC before freezing its catalog, drains authenticated file owners and managed work, and copies the stopped database together with its exact required file closure. Acknowledged unpublished/incomplete data and unresolved intents are included; legitimately absent/unsealed staging remains unresolved. Missing required sealed files fail closure. Completion is published last. Restore validates inventory, integrity and interpretation under a fresh root/incarnation/credentials before activation; old-root cleanup still requires authentic drain. This is an offline operator lifecycle.
These contracts alone do not establish qualification or claim power-loss durability, remote
deployment support, capacity measurements or comprehensive scientific coverage. Completed selected Plan39/38 acceptance belongs to §24.2 and the Plan39d Outcome.
Plan28's remaining independent campaign belongs to the
[current packet](../../plans/28-surrealdb-unified-substrate.md), not this architecture page.

### Protected result decoding

> Decision: ADR-0180 and ADR-0186 (proposed amendments; maintainer-approved Plan46 execution).

Production result reads retain one mutex-owned decoder/partition with frozen ticket selection
and complete integrity admission. Cancellation-aware lane acquisition precedes the decode
allocation grant, so waiters do not reserve decoder storage they cannot use. Independent
file-affine readers were compared as bounded test controls; no parallel decoder or prefetch
backend is selected. Adoption requires an admitted off-poll CPU owner, private charged reader
state and benefit across complete protected reads, including stop/failure/release. Optional
immutable payload retention is separate from active escaped protection lifetimes.
