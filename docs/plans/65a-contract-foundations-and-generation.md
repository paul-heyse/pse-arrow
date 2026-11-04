---
title: "Plan65a: Contract foundations and generation"
status: in-progress
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [current-work, design, libraries, preparation]
---

# Contract foundations and generation

## Target and baseline

**Proposed:** make the structural contract of a selected scientific, operational or tooling
boundary a reusable projection of its actual semantic owner. One resolved contract world
drives messages, adapters, local-rule validation, selected method metadata, inspection,
documentation and applicable database/Arrow projections. Domain behavior remains executable
at its owner; a descriptor neither supplies scientific admission nor authorizes effects.

The [coordinator](65-proto-integrated-target.md) owns the combined target and source dispositions.
This companion owns A01–A07. Execution assumes completed Plan64. Its implementation and every
code surface it touches remain eligible for replacement; no concurrent editing partition or
earlier plan ownership constrains this design. The inspected source is
`f61286a5fa5894b480efe8584c8b3d9b76d1b1fb`, including the observed dirty projection work.
Those observations are **Interface-checked**, not post-Plan64 qualification.

Criteria are core 3.4, efficient-architecture heuristics 1.0 and process-simulator 1.5 through
the selected standard. CBO-195, CBO-042, CBO-040 and CBO-046 motivate the shared generation;
CBO-196 retains its separate acquired-rustdoc interpretation question. Supporting rationale
is the [generation briefing](../library_leverage/libraries/proto/briefings/contract-ownership-and-generation.md).

## Focused foundation assessment and decisions

The [improvement review](../plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md)
refines the target through its I05/I07–I09/I11 recommendations. Before A01 fixes the shared
interfaces, consume the design-level operation/product semantics from
[65i I01](65i-scientific-operations-and-product-semantics.md) and developer evidence/construction
direction from [65j J01](65j-workflow-lifecycle-and-developer-evidence.md). These are agreed
meaning and consumer inputs, not whole-companion implementation barriers. A narrow working
A/B slice then enables those companions' actual delivery; no universal graph must be finished first.

Current `pse-model::boundary_projection` contains useful resolved shape and stable tags,
`ProjectionType`, owner-local `ProjectionRecord`/`ProjectionAlternative` adoption, local
rules, service declarations and macros. `pse-codegen::boundary_projection` lowers these to
descriptors and Buffa. `xtask::codegen::engineering` composes worker, engineering and reference
generation. The native host has independently authored Proto contracts and hand mappings.

The generic graph is currently housed with product meaning. Importing that product graph into
the independent host would couple code intelligence to scientific/model dependencies. Keeping
another graph in the host instead duplicates structural interpretation. The selected target is
**one new pure `pse-contract` crate**, containing generic shape/declaration/adoption interfaces
and source-level annotation macros. It has no Arrow, database, solver, Buffa, ConnectRPC or
runtime dependency. Product identity/quantity implementations and domain construction remain
at their existing owners. Code generation and compiler/plugin invocation remain tooling work.
The new crate and dependency-placement decision require the coordinator's proposed ADR route
before implementation; this plan does not declare that route accepted.

Execution refinement, 2026-11-04: [proposed ADR-0244](../adr/0244-shared-proto-contract-adapter.md)
places the selected Buffa descriptor adapter in `pse-contract-proto`. Generation and independent
host inspection need the same lossless option acquisition, schema correspondence and normalized
input to the pure checker. The host cannot import product-heavy codegen, and duplicating that
adapter would repeat consequential interpretation. Move its implementation and focused tests;
retain graph lowering/compiler invocation in tooling and domain facet callbacks at their owners.
The pure crate remains wire-free. This is part of A02/G02/J02 delivery, not another vocabulary,
descriptor service or new native authority.

Alternatives were considered as complete routes. Retaining the graph in `pse-model` is adequate
only while product consumers are its sole population; the selected host migration defeats that
premise. Expanding `pse-vocabulary` mixes scalar vocabulary and construction/generation behavior.
Moving everything into `pse-schema` drags relation/Arrow concerns into unrelated tools. A new
generic foundation buys a concrete second consumer and local reasoning, not a universal IR.

Retain the owner-derived hybrid for first-party meaning: source owner → structural declaration
→ coherent descriptors/adapters → local representation checks → native admission/execution →
result projections. Independently owned database/telemetry protocols remain `.proto`-first.
`proto_rs` 0.11.26 native annotations/shadow conversions remain a credible narrow alternative,
but replacing the complete existing validation/reflection/Connect/storage projection route
would introduce another runtime/interpretation composition without demonstrated simplification.
Native-only/Serde remains selected where no independently interpreted boundary benefits.

## Contract worlds and metadata

A contract world is a closed published root population, its complete imported descriptor
closure, effective feature settings, typed interpretation metadata and selected validation
policy. Descriptor artifact bytes identify that artifact; they do not replace model semantic
identity, scientific dependency keys, database mapping revision or validation-policy identity.
Store these identities as distinct associations rather than hashing display comments into a
new universal semantic key. Use the existing artifact/provenance and identity owners.

Publish one world per coherent consumer boundary—initially host intelligence, engineering,
scientific worker and scientific reference exchange—with shared imports where appropriate.
Do not aggregate all contracts into one globally invalidated corpus. Share `SharedCorpusContext`
only for exactly matching descriptor and representation inputs; shared imports alone are
insufficient. Link all retained messages/readers to their immutable world generation.

**Select typed versioned custom options for machine-readable contract metadata.** Define the
interoperability option schema under `pse.contract.v1`; message, field and method annotations
carry owner correspondence, interpretation/direction, declared semantic links/facets and
operation effects. Emit them from actual source owners. Effect annotations describe intent;
native authorization and execution still check their own context. Unknown or unrepresentable
selected facets must fail generation with an attributable source diagnostic, not become a
`Debug` string or a JSON bag. Domain-specific facets use their own typed imported definitions
under one semantic owner rather than growing a universal scientific ontology in `pse-contract`.

The option schema is an independently versioned interoperability contract, bootstrapped as
ordinary `.proto`; it does not become a second declaration of scientific quantities. Reserve
extension numbers/names deliberately, validate collisions against the imported world, and
retain unknown options where forwarding is the declared operation. The exact option schema
and allocation are published in A02 before dependent generator/consumer changes.

**Core metadata mechanism.** The proto2 source
`crates/pse-contract/proto/pse/contract/v1/metadata.proto` allocates
`pse.contract.v1.message`, `.field` and `.method` at 51065 in the distinct
`MessageOptions`, `FieldOptions` and `MethodOptions` extension namespaces.
`MessageMetadata` carries metadata version, Rust type/declaration-file provenance,
owner interpretation version, direction, construction role and optional original
action effects plus the required native owner shape. `FieldMetadata` carries version,
native field correspondence, direction and the required native type/local-rule contract;
`MethodMetadata` carries version and explicit method effects. The action's
intent and the RPC's composed intent remain separate declarations. Unspecified enum
zero is unavailable meaning. The major namespace does not imply that every
native owner's interpretation is version 1.

Compile these ordinary sources and their complete imports with Buf, then link the
lossless Buffa `FileDescriptorSet` into a `DescriptorPool`. Use
[`DynamicMessage`](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/struct.DynamicMessage.html)
and fallible
[`ReflectMessageMut::try_set`](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/reflect/trait.ReflectMessageMut.html#method.try_set)
to create concrete typed option messages against that acquired schema. Checked
extendee/type matching and Buffa's unknown-field extension storage preserve unrelated
imported options; ordinary typed Prost re-encoding is excluded. Protobuf's
[custom-option resolution rules](https://buf.build/docs/reference/protobuf-language-spec/#options)
define the separate extendees. The current adapter checks known enum numbers, field tags,
types and explicit presence before interpreting an imported lookalike as current metadata.

One `pse-contract::metadata` checker consumes borrowed normalized facts, with no wire
or product dependency. Generation and descriptor inspection use that checker for
required metadata, versions, import/reference closure, extension allocation and effect
coherence. Old/new comparison attributes role, direction, action/method effect and
interpretation changes under a named policy independently of wire compatibility.
Known owner/version associations come from the actual generation graph; offline readers
must select the immutable interpretations they have available. A missing owner interpretation
or external domain policy remains Unknown. Rust path/file moves remain provenance
changes. Selected registry facets use their native typed validator and purpose-specific
normalizer; the named comparison policy rejects changes to normalized semantic facts.
Introducing a selected facet on an existing field needs an explicit owner-policy
decision; individual validity does not establish compatibility with the previously
unannotated field. Borrowed native coverage selects expected owners, fields (including
alternative payloads) and methods independently of option presence. Stripped annotations
or removed selected descriptors cannot disappear into a vacuously compatible world;
inspection without approved coverage remains Unknown for semantic comparison.

The descriptor comparison takes separate old/new source policies. Each domain callback
validates its decoded value against that value's original acquired pool before
normalization; the current pool never re-decodes old payloads as a substitute for an
unavailable original interpretation. The pure checker preserves that availability
boundary even when bytes are identical and a current policy exists.

Within one strictly checked pool, IPC-to-typed-fact consistency uses fallible encoding
for known nested message facts: Buffa's in-memory equality distinguishes a constructed
empty repeated list from its decoded absent spelling, while protobuf gives those lists
the same meaning. Explicit optional presence remains observable. This local check is
separate from cross-world, purpose-specific semantic comparison. Reference convenience
and correlated forms normalize through the existing `ReferenceContract` owner and actual
native nullability, recursively through the real native field tree; original IPC bytes
remain retained, and changed null policy/component targets refuse agreement.

Execution investigation, 2026-11-04: complete A02 with typed native field-shape meaning from
the existing `TypeRef`/`Scalar` owner, rather than only native field names and descriptor
references. A wire-compatible change can alter unsigned/signed meaning, IEEE bits, nominal
identity versus opaque bytes, ordered versus set membership, nested absence, or map key/value
meaning. Required native distinctions must survive typed option publication and shared
normalization/comparison. Reuse the existing structural vocabulary; do not add another IR or
infer this meaning from Rust display names. The precise typed schema/version extension and
matching descriptor/source policy controls remain bounded A02 work before policy freeze.

The current writer selects interoperability metadata version **2** in the existing
major namespace. Its required native field contract transports the actual `TypeRef`,
closed `Scalar` and `LocalRule` declarations (including exact binary64 interval bits).
Required native owner shape transports the existing record/alternative layout and
canonical string vocabulary, including source tags and per-field direction. Decoding
normalizes back to those existing native types; it does not add a second semantic IR.
Old schema/version interpretation remains Unknown, and current missing facts refuse.
Raw historical descriptor inputs remain inspectable without a legacy production writer.

The selected registry policy must also separate retained full option facts from the semantic
projection it compares. Documentation and structure presentation alone must not become physical
incompatibility merely because their encoded option bytes differ. Use the native owner's
metadata-purpose decisions for that normalization, preserving meaningful physical, collection,
reference and storage-policy changes. Independently test both wire-compatible native meaning
changes and presentation-only changes; unavailable interpretation still yields Unknown.

The composed comparison purpose is explicitly **native-crossing + registry-execution +
canonical-publication**. It is not a scientific identity or universal storage identity.
Source-derived coverage includes required domain extensions and typed option member paths,
so stripping a quantity, canonical policy or physical carrier from both acquired worlds
cannot erase its native obligation. The same source-key lowering owns emission and coverage.
Domain callbacks validate and normalize in each world's own `DescriptorPool`; full original
FDS/options remain retained. Prose-only facts are retained but have no semantic comparison
value for this purpose. Other identity purposes retain their existing metadata rules.

Actual registry fields carry a standard Arrow IPC Schema Message containing exactly the
source-owned native `Field`, emitted by Arrow 59.3
[`IpcDataGenerator::schema_to_bytes_with_dictionary_tracker`](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/writer/struct.IpcDataGenerator.html#method.schema_to_bytes_with_dictionary_tracker).
The source-only canonical field carrier is not a published JSON type schema. Before
[`fb_to_schema`](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/convert/fn.fb_to_schema.html),
the domain adapter verifies the acquired FlatBuffer, exactly-one-field cardinality,
metadata key/value presence and uniqueness at every nested field, and supported logical
union, dictionary, precision/unit and child-layout premises. This is necessary because
the pinned [conversion source](https://docs.rs/arrow-ipc/59.3.0/src/arrow_ipc/convert.rs.html)
otherwise folds duplicate metadata into a map and has infallible panic branches for
unsupported type declarations. Invalid/unsupported input refuses before normalization.
Typed registry facets must agree with the decoded native field metadata. Existing
`pse_columnar::native_field::project(ExecutionIdentity)` plus the existing canonical native
field renderer provides this purpose's physical projection: timestamps retain unit/timezone,
as do decimal, dictionary, nullability and nested field meaning, even when Rust/wire scalar
representations overlap. Explicit typed canonical policy carries publication meaning that
the existing execution projection deliberately excludes. Complete native `TypeRef` versus
Arrow source-rendering correspondence remains an explicit A02 obligation; neither these
local guards nor exact compiled-world binding alone establish that broader equivalence.

The lossless wire adapter lives in `pse-contract-proto` (ADR-0244), shared by generation
and the independent host. It depends on pure `pse-contract`, the selected Buffa family
and native error support; registry, Arrow, runtime and scientific libraries remain
outside its dependency closure. `MetadataError` retains linker, codec, reflection and
external domain diagnostics with actual file/subject context. The generator's
`ProjectionCodegenError` composes registry and metadata errors without converting a
newly crossing native failure into a string. Graph lowering and selected domain
policies stay with codegen/their source owners; the old adapter implementation and
namespace are removed.

Native crossing mechanics use the same lightweight adapter owner through its opt-in
`crossings` capability. Buffa's owned messages and `MessageField` preserve presence;
the maintained validator's `Validate::validate` supplies original field/rule violations
and compilation/evaluation failures. `BoundaryProjectionError` retains those typed
causes, distinguishes uncontrolled ingress from outgoing producer failures, and keeps
consumer diagnostic presentation external. Runtime supplies its local
`ProjectionDiagnosticError`; native diagnostic owners acquire no wire dependency.
The source scalar owners implement pure `ProjectionBytes`, so identity widths and
nominal reconstruction stay with IDs/model rather than foreign transport impls.
Standard scalar/container mechanics have one owner; generated adapters use that owner
for both engineering and independent scientific interchange. See the pinned
[Buffa message API](https://docs.rs/buffa/0.9.2/buffa/trait.Message.html) and
[maintained validation API](https://docs.rs/protovalidate-buffa/0.7.2/protovalidate_buffa/trait.Validate.html).
The metadata-only crate does not select the validator capability; actual workspace
feature unification can still select the validator's Connect support through runtime
and host consumers.

The adapter emitter resolves nested Rust module names through Buffa's
[`CodeGenContext::nested_module_name`](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/context/struct.CodeGenContext.html#method.nested_module_name),
including acronym and digit boundaries, rather than approximating library casing.
Synthetic protobuf map entries contain their key/value fields; collection-value
wrappers are sibling messages in the containing scope, so Buffa emits their actual
owned and borrowed types. Native adapters and descriptor qualification follow that
same source lowering. Explicit source public paths make native field access independent
of the generator consumer module's location.

Maintained validation emission discharges only comparisons established by the
actual Rust integer domain; declared descriptor rules, strict endpoints, combined
failure categories and violation paths remain intact. Preparing a patched plugin
requires an explicit locked source reinstall before generation. A04 still needs the
existing tool/provenance owner to bind that prepared binary to the actual vendor
source and locked build: a matching version and filesystem path alone do not
establish that correspondence. This extends existing preparation rather than adding
a separate preparation framework.

An actual `projection_struct!` declaration may select `construction: ObservationOnly`.
The same fields generate native local validation, graph metadata and borrowed
`ProjectionObservationRecord` snapshots; selected passive fields are cloned individually,
without requiring the live owner to implement Clone. `@local` custody fields need no
initializer and are never serialized or reconstructed; explicit borrowed checks remain
available. The emitter borrows nested sealed owners through `Borrow<ActualOwner>`,
including owned boxes and collection snapshots, and emits no inverse for them. A Data
parent containing any sealed descendant is refused. Public/crate/parent wrapper visibility
is selected separately from the strict or retained-wire unknown-field policy.

**Registry facet lowering.** The registry owner publishes
`pse.schema.v1.registry_field`, a message-valued `google.protobuf.FieldOptions`
extension allocated at 51066 in
`crates/pse-schema/proto/pse/schema/v1/field_facets.proto`. Its versioned
`RegistryFieldFacets` contains typed extension uses, column roles, documentation,
structure names, unbound quantity references, transfer-context versions, correlated
references, graph endpoints, collection order/cardinality/uniqueness, integer ranges
and discriminator-to-payload selections. Extension parameters are a closed oneof:
enumeration, ordinal target relation, expression grammar, or parameter-free extension.
Every enum spelling/number must match its native owner. A quantity name remains an
unresolved reference; this annotation does not manufacture physical quantity semantics.

Identity annotations carry the native source identity name and explicit ownership
declaration separately; ordinary carriers do not acquire catalog ownership. Canonical
field annotations carry the closed native opaque-key kind, optional explicit mutability
and record-key declaration, without minting a database record or execution authority.
Expression source annotations retain the owner-authored path template and closed
list/zero/flatten ordinal and domain-before-name binder policies. The native source
builders and option readers share these owner parsers and vocabularies. The schema's
enum allocations are checked against the native correspondence; a selected key that
acquires a new native meaning requires an explicit typed interpretation update.

Canonical publication annotations additionally retain source ASSERT/DEFAULT, initial
lifecycle member, exceptional transition predicate, sealing predicate/refusal and reference
kind. `pse-schema::model::CanonicalFieldPolicy` owns this interpretation, composed with the
existing opaque key/readonly/record-key contract. SurrealDB 3.3.4
[`surrealdb_syn::expr_for_definition`](https://docs.rs/surrealdb-syn/3.3.4/surrealdb_syn/fn.expr_for_definition.html)
uses stored-definition grammar and requires the entire input (`Parser::assert_finished`);
the inert `surrealdb_sql::Expr` is rendered with native
[`ToSql`](https://docs.rs/surrealdb-types/3.3.4/surrealdb_types/trait.ToSql.html).
Formatting-equivalent expressions therefore compare by canonical native syntax, while
meaningful policy changes remain incompatible for canonical publication. The original
concrete `ParseError` remains in the native diagnostic source chain. An initial member is
native enum vocabulary, never an expression. Refusal identities use the existing closed
native vocabulary; enum membership and physical field associations require their actual
resolved owners. Syntax success establishes neither database/variable/type validity nor
execution permission. Native declaration validation and option interpretation call this
same owner; no generated policy table or generic metadata fallback replaces it.

The existing source-derived metadata map is an internal carrier. The domain lowerer
uses shared `pse-schema` canonical parsers, also used by native field admission, for
structured annotations; it neither invents a second JSON interpretation nor rebuilds
Arrow storage from a projection type. At source rendering, `ReferenceContract::for_contract`
normalizes scalar foreign keys using the actual native field's nullability. Malformed
present role, extension, extension parameter or scalar-reference declarations refuse
instead of disappearing through convenience defaults. Native storage and catalog
admission remain separate from reading these typed annotations.

Boundary tags agree with the descriptor field number. Nested `BoundaryShape` annotations
agree with the actual resolved native graph owner, interpretation, direction,
reservations and alternative name/number population; the descriptor and core owner
options carry that structure rather than an additional JSON shape annotation. Other
selected facet keys require a typed owner interpretation before publication and refuse
with field/key attribution when unsupported. This applies to known but not yet lowered
facets as well as unknown keys; no annotation is silently omitted.

Buffa 0.9.2 `DescriptorPool` resolves the compiled authored schema closure;
`DynamicMessage::try_set` checks each named typed field before the common extension
writer attaches its encoded message payload. The independent domain reader uses
`DynamicMessage::field_by_number` and rejects unknown selected fields, unknown
versions/enums, incomplete oneofs and invalid native facet constraints. It supplies
the shared checker with an explicit domain policy; an unavailable policy remains
Unknown. These use [Buffa's reflection API](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/struct.DynamicMessage.html)
and [protobuf message-valued custom options](https://protobuf.dev/programming-guides/proto2/#customoptions),
not display comments or an untyped metadata bag.

Keep actual source relationships separate. An owner path/type association is not an exact Rust
source span. A02 replaces synthetic `SourceCodeInfo` owner/facet comments and the selected
action-effects sidecar with typed options; it retains genuine available source information in
imported schemas. Remove any replaced comment parsing and duplicated machine-policy sidecars
with their consuming migration. Human documentation may render the same metadata without
becoming another authority.

### Observation is not executable authority

The pure declaration vocabulary must distinguish inspectable data from native owner-issued
capabilities. Generated messages may describe a prepared handle, admission decision, completion,
test verdict or protection, but decoding that description cannot construct the authority-bearing
native value. Generate fallible correspondence only for the observable product; owner factories
resolve issued identities and establish current context, custody and permission. Keep sealed
constructors and handles outside generic `ProjectionRecord` adoption where reconstruction would
manufacture a grant. An authentic historical record is evidence of its original scope, not a
current execution capability. Test a structurally valid serialized grant-shaped record and require
that it remains an observation until the actual owner independently authorizes the operation.
This gives review I05 a concrete construction-direction obligation without treating every ordinary
domain constructor as security-sensitive.

### One semantic metadata checker and evolution comparison

A02 selects one native checking operation over an approved contract world and an explicit
interpretation policy. It validates required/known metadata versions, resolvable semantic links,
effect declarations and declared interpretation relationships. Generation, bundle import and
offline inspection reuse it; they do not each restate those rules. The old/new-world comparison
returns compatible, incompatible or unknown under a named policy and attributes meaningful
option changes even when ordinary wire checks pass. Scientific admissibility remains at the
scientific owner; a metadata check cannot establish physics or mint authority.

The existing native generation/import entry points are the initial consumers. Buf check plugins
are an optional adapter for a real independent authoring/publication consumer, not another rule
authority. They use Bufplugin/PluginRPC rather than `CodeGeneratorRequest`/`CodeGeneratorResponse`.
The documented Go SDK offers `check.RuleSpec`, descriptor handlers and source-attached
annotations; a native implementation must honor the underlying protocol. A04 compares adapter
maintenance/dependencies with invoking the native checker directly, and implements the adapter
only when it removes useful integration work. Never introduce a Go subsystem solely to repeat
the same checks. See [Buf check-plugin API](https://buf.build/docs/bsr/checks/plugins/)
and [descriptor-rule tutorial](https://buf.build/docs/bsr/checks/plugins/tutorial/).

## Generation and artifact flow

Review I07 extends preservation to the complete real acquisition route: compiler output →
descriptor/image conversion → import normalization → generator request → effective runtime
and independent inspection artifacts. Identify each actual initial producer and its intended
loss/normalization contract. Linking successfully after a typed intermediary has discarded
custom options is insufficient. Use a lossless selected route and independently specified
message/field/method options, including imported extensions and unknown forwardable options.

`protox::Compiler::encode_file_descriptor_set()` is a concrete comparison capability: it includes
extension options unlike re-encoding the ordinary typed `prost_types::FileDescriptorSet`.
`include_imports(true)` and `include_source_info(...)` govern separate material. This versioned
[Protox 0.9.1 API](https://docs.rs/protox/0.9.1/protox/struct.Compiler.html#method.encode_file_descriptor_set)
demonstrates the loss trap and a supplied preservation route; it does not mandate adopting Protox
or assert that the current Buffa producer loses options. Keep intentional source-info omission,
effective feature transformation, logical metadata preservation and archival byte identity
separate. An independent client must observe required option meaning after the complete route.

The current custom generator encodes the full `FileDescriptorSet` and passes those bytes to
`buffa_codegen::shared_descriptor_root_module`. Buffa's normal `encode_descriptor_set` applies
feature overrides and removes `source_code_info`; the root helper merely embeds caller bytes.
Thus the present custom path is not evidence that its runtime pool is compact, and a nondefault
override could otherwise disagree with reflected semantics. Default overrides in the inspected
route do not establish a current runtime mismatch.

A03 produces two deliberate artifacts from one effective world:

- A complete source-aware generation artifact with imports, options and genuine available
  source correspondence for provenance, documentation and independent tooling.
- An effective runtime artifact with the same relevant feature and custom-option semantics,
  omitting source-only material after typed metadata no longer depends on comments.

The first integrated A03 slice uses Buffa 0.9.2
[`FdsEmbedding::Sidecar`](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/enum.FdsEmbedding.html)
with `IncludeMode::Relative("")` in `shared_descriptor_root_module`. Each actual publisher
already writes the exact `descriptor.bin`; the generated shared root now references that
file with `include_bytes!`, and Git marks `.bin` artifacts binary. All five source-derived
worlds retain byte-identical descriptor files. Their combined shared-root Rust source falls
from 5,660,088 to 12,245 bytes; this is a generated-source size observation, not a runtime,
compile-time or resident-memory benefit measurement. Combined publication still covers all
13 targets. Effective feature application, source/runtime artifact separation and runtime
compaction remain open; changing embedding alone does not discharge those obligations.

Normalize effective feature handling once. Generated code, validation, reflection and client
schemas must interpret the same feature settings. Use `encode_descriptor_set` for its supported
runtime transformation, with independent checks that relevant custom options survive. Do not
alter full artifact bytes in place or derive real source coordinates from placeholder spans.

Use `generate_with_diagnostics`, preserve consequential warnings, and publish outputs atomically
through the existing generated-tree/provenance machinery and `just codegen`. Compatible
`GeneratedFileKind::Companion` files use `apply_companions` once after package/collision checks;
retain independent plugin outputs where that is simpler. Do not reproduce Buffa module-tree
assembly in each consumer. Keep `extern_path`, naming, excluded packages, validators and service
generation consistent. A mapped Rust type must satisfy the contracts of its actual field position.

Prost consumers can use `prost_build::Config::compile_fds` or a deliberately prepared descriptor
input with `skip_protoc_run`; they do not need another authored domain schema. Retain the external
Prost/Tonic protocol route without converting every external message to Buffa in memory.

## Packages and readiness

All statuses are **planned**. Dependencies describe available products, not whole-document barriers.

| Package | Status | Consumes → delivers | Migration, deletion and targeted acceptance |
| --- | --- | --- | --- |
| A01 | complete | Actual native baseline, 65i I01/65j J01 design inputs, proposed ADR-0241 and scoped independent assessment | Pure source-owned graph/macros/adoption extracted; owner implementations and all callers/generators moved; old foundation and temporary aliases deleted. Tests: contract 8/8, model 27/27, codegen 25/25, diagnostics 2/2; workspace all-target check zero errors/warnings. Integrated snapshot is 7e35abae2; H qualification remains separate. |
| A02 | in-progress | Working A01 field/method semantics → versioned typed metadata option schema and owner emission | All three selected worlds publish typed owner/field/action/method options and all 20 native facet keys. Shared lossless acquisition/checking and native crossing helpers moved to independent pse-contract-proto; replaced implementation/comments/sidecar removed. Source-selected coverage, unknown policy/schema and original-pool controls pass. Native field-shape meaning, domain semantic normalization and independent host policy/readback remain in progress with A05/G02/J02. |
| A03 | in-progress | A02 metadata plus declared effective feature settings → complete and runtime contract-world artifacts | Five source-derived worlds now reference their byte-identical published binary descriptors instead of escaped Rust literals. Complete effective feature application and separate full/runtime artifact publication. Test nondefault features against generated/reflected semantics, runtime compaction preserving typed options, full-source retention and immutable old-world readers. |
| A04 | planned | A03 closed worlds → shared generation composition for all four initial boundary populations | Integrate Buffa, packaging, validation, Connect and selected external Prost generation. Remove repeated module/descriptor assembly and hand-kept plugin naming maps. Test warning/refusal propagation, orphan companions, duplicate filenames, excluded packages and exact import identity. |
| A05 | planned | A01–A04 narrow working slice plus B01/B02 adoption policy → complete native/protocol correspondence for selected populations | Migrate host native request/fact families, engineering/worker/reference structural adapters, domain-owned document boundaries, and c/f/g consumer projections. Inventory actual remaining maps at implementation baseline; classify semantic checks explicitly and remove all replaced mechanical maps. Test native→wire→native against independently constructed expected meaning, raw paths and missing/unknown alternatives. |
| A06 | planned | Current registry/function owners plus A02/A03 → source-declared selected storage/Arrow projection metadata consumed by C packages | Reuse source facts without decoding ProtoJSON or inferring database behavior from field spelling. Delete duplicate selected field/schema declarations with producer/consumer migration. Preserve transactional/domain-specific logic; test precise numeric, relationship and mapping-version changes. |
| A07 | planned | Complete selected consumer migration → regeneration equivalence, extension controls and handoff | Add provider request, method effect, diagnostic and result facet through their actual owners and verify every selected projection follows. Run package compile/targeted checks during work and generation when declarations change; scope-end assembled/static checks belong to 65h. Delete remaining obsolete fixtures and parallel publication paths. |

Review-driven package obligations are part of these existing packages:

- **A01:** consume 65i I01/65j J01 design products before fixing generic interfaces. Classify
  observable records versus non-reconstructible issued capabilities and preserve sealed native
  construction through extraction and every generated adoption surface (review I05/I11).
- **A02:** implement the shared metadata checker and old/new semantic comparison together with
  typed options. Independent wire-compatible descriptor pairs must expose changed semantic
  references, validation interpretation and method effects with attributable outcomes (I08).
- **A03/A04:** prove required options at each real acquisition/publication consumer; deliberately
  insert a lossy typed conversion in a negative control. Configure complete import/source/effective
  semantics and use the same checker at generation/import/inspection. A04 selects a Buf adapter
  only for the identified useful consumer, otherwise delivers native invocation without an adapter
  or duplicate rule set (I07/I08).
- **A05/A07:** use bounded schema-driven inputs and controlled perturbations for mechanical
  correspondence. Buffa 0.9.2 `CodeGenConfig::generate_arbitrary` emits `arbitrary::Arbitrary`
  support gated on the consuming crate's `arbitrary` feature; qualify actual custom/extern
  storage and representation closure rather than force it everywhere. Retain input bytes/seed,
  contract world and the original native test inventory/report/failure identity through 65j.
  Independently specified semantic/scientific cases remain separate evidence (I09).

An early A01/A02/A03 slice may serve diagnostics and tooling without waiting for every registry
projection. A05 is complete only for its actual declared migration population, not one demo
message. A01/A02 changes cannot create a second generic graph or authorize arbitrary peer schemas.

## Verification and completion

**Proposed verification:** contract-owner and generator tests cover declaration correspondence,
effective feature semantics, typed options, recursion, imports and precise failure attribution.
Independent generated consumers must read the resulting descriptors and verify expected fields,
presence/effects and interpretation identity; comparing two artifacts from the same incorrect
lowering is insufficient evidence. Run `scripts/pse check` for touched packages and targeted
`scripts/pse test` with explicit force-validation for Arrow-consuming closures. Run `just codegen`
when source/generator declarations change. The root follows the repository rhythm for formatting,
scope-end hygiene and integrated checks; no broad campaign is due for this plan's authorship.

Completion requires all four initial world populations and every selected downstream consumer to
use the new foundation, machine-readable metadata to survive runtime preparation, generation
warnings/refusals to remain attributable, and replaced implementations to be deleted. Retained
historical worlds remain explicit interpretation products rather than active compatibility
production routes. Record **Implemented**, named **Tested** and any **Measured** conditions
separately; passing generation establishes neither scientific fidelity nor a performance benefit.

## Technical sources

Current source: [graph](../../crates/pse-contract/src/boundary_projection.rs),
[owner adoption](../../crates/pse-contract/src/boundary_projection/access.rs),
[lowering](../../crates/pse-codegen/src/codegen/boundary_projection.rs),
[orchestration](../../xtask/src/codegen/engineering.rs), and
[host wire](../../crates/pse-host/src/semantic/wire.rs). These are implementation leads to
reconcile against completed Plan64, not immutable target boundaries.

Selected mechanisms: [Buffa 0.9.2 generation source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs),
[descriptor embedding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/reflect.rs),
[Prost-build 0.14.4](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html),
[maintained validator](../../vendor/protoc-gen-protovalidate-buffa/README.md),
[Buffa reference](../libraries/proto/Buffa.md), and
[Rust-first comparison](../libraries/proto/Rust_first_proto_contracts_and_proto_rs.md).
The complete eight-reference reread and exact installed-source checks inform this plan;
documentation-only authoring runs no generator or product test.
