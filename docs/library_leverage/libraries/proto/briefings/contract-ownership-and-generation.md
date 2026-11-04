# Contract ownership and generation

Inspected 2026-11-03 at `f55951cfe995a689f9b75185b8e89cb362021616`, including
the supplied uncommitted Proto references. Current mechanisms below are **Interface-checked**
source observations; recommendations are **Proposed**. No generator, conformance suite, product
test or performance measurement was run for this account. SurrealDB v4 is excluded.

## Target: one declaration of meaning, multiple useful projections

The recommended target is an **owner-derived hybrid**, with a compiled descriptor corpus as
the shared contract between generation consumers. Domain Rust types and relation declarations
own scientific meaning and executable behavior. Stable tags, interpretation versions, local
constraints and discoverability metadata are co-declared at those owners. Protobuf descriptors
project that meaning to wire representations, generated validation, local services, independent
clients, reflection and documentation. Database and Arrow projections consume the appropriate
source meaning rather than reconstructing it from wire spelling.

This is a codebase-wide design, not a universal Proto domain model. An external protocol with
independent ownership can remain `.proto`-first. A local invariant-bearing quantity remains a
native domain type even when its portable representation is generated. The useful unification
is one interpretation and one generation input for a given contract, not one file format for
all computation and persistence.

The value is extension locality: a new provider request, diagnostic fact, model-edit variant,
or result product should add its meaning at the responsible owner and obtain mechanical
projections without separately editing client, validator, inspector and document schemas.
Conversely, the generator must not invent an admissible physical model from structural fields.

## Existing route and remaining opportunity

The current source already establishes much of this direction:

| Source owner | Observed responsibility | Target continuation |
| --- | --- | --- |
| `pse-schema::FieldContract`, `BoundaryRecord` | Registry fields carry explicit boundary tags; records carry names, interpretation versions, direction and reservations. | Keep public numbering deliberate and generated shape tied to actual relation definitions. Extend only for a consumer whose meaning is represented. |
| `pse-model::boundary_projection` | `ProjectionType`, `ProjectionGraph`, `Definition`, `TypeRef`, `Field`, `LocalRule`, `Service` and `Method` describe resolved native types and selected operations. | Share complete source-derived graphs across generation, validation and inspection, without making this a separately editable simulator IR. |
| `ProjectionRecord` and `ProjectionAlternative` | Access actual native field tuples at their defining owner, including private fields, with version/tag/layout checks. | Generate mechanical conversions while preserving fallible owner construction and attributed adoption refusal. |
| `pse-codegen::codegen::boundary_projection` | Produces descriptors, Buffa sources, warnings, documentation and native adapters. Supplied imports retain custom options; duplicate filenames refuse. | Consolidate reusable generation composition and improve exposed contract metadata before adding bespoke adapter families. |
| `xtask::codegen::engineering` | Builds scientific-worker, engineering and reference graphs; preflights validation; invokes Connect and validator plugins; publishes descriptor bytes and source relationships. | Reuse the complete generation route across additional selected contracts. Do not create another executable/tool-preparation owner. |
| Host Buf configuration | Selects Buffa vtable reflection, shared pool, JSON/text, packaging, Connect bindings and generated validators. | Align independently generated package mappings and naming. Share technical mechanisms without conflating host session lifetime with engineering-operation lifetime. |

Source pointers: [projection model](../../../../../crates/pse-model/src/boundary_projection.rs),
[typed owner access](../../../../../crates/pse-model/src/boundary_projection/access.rs),
[projection generation](../../../../../crates/pse-codegen/src/codegen/boundary_projection.rs),
[engineering orchestration](../../../../../xtask/src/codegen/engineering.rs),
and [host plugin configuration](../../../../../crates/pse-host/proto/buf.gen.yaml).
Blueprint §4.7 owns the current projection contract; these recommendations do not amend it.

The overlap with CBO-195 is concrete: manually translated protocol representations can disagree
with source meaning. CBO-042 covers document-envelope interpretation, CBO-046 canonical generated
calls, and CBO-040 baked registry products. These are related consumers, not a reason to merge
their distinct responsibilities into a universal schema owner. CBO-196's rustdoc intermediate
is a separate acquisition pipeline; protobuf generation is not evidence that its JSON artifact
should automatically be replaced.

## Concrete generation composition

### Compile and preserve the correct descriptor world

For authored `.proto`, use Buf, `protoc`, or a selected `protox` compiler route to resolve imports,
options, syntax and edition semantics. For source-emitted descriptors, validate the graph and
its full import closure before handing it to consumers. `buffa-codegen` is a descriptor-to-Rust
backend, not a source parser or a substitute for every compiler validity check.

Retain two deliberately different artifacts where their consumers need them:

- The complete generation descriptor corpus, including source annotations and imported custom
  options, for compiler diagnostics, documentation and provenance.
- The compact runtime descriptor corpus for linked reflection and schema discovery.

Buffa 0.9.2 strips `source_code_info` when embedding reflection descriptors. That runtime artifact
cannot reconstruct source comments or exact source ranges. Current host source relationships
and syntax witnesses therefore remain relevant rather than being replaced by descriptor columns.
Scientific identity remains owned by the semantic hashing contract; neither descriptor bytes nor
an arbitrary protobuf encoding becomes its replacement.

For Prost consumers, `prost_build::Config::compile_fds` accepts an existing descriptor set;
`skip_protoc_run()` consumes a selected descriptor artifact without launching `protoc` again.
`extern_path`, `bytes`, `btree_map`, and `ServiceGenerator` change generated realization without
transferring scientific authority. Use a single prepared descriptor input where multiple
generators would otherwise independently compile the same schema.

### Compose generation without reconstructing module scaffolding

Buffa 0.9.2 supplies `generate_with_diagnostics`, `GeneratedFile`, `GeneratedFileKind`,
`apply_companions`, shared descriptor roots and `FdsEmbedding`. Prefer the diagnostic-returning
entry point over `generate`, whose implementation discards nonfatal warnings. Preserve warnings
such as unmatched feature overrides and unresolved excluded-type mappings as inspectable build
diagnostics, with a deliberate policy for consequential omissions.

`apply_companions` is a useful extension point for extra traits, adapters and service-related
code emitted from the same descriptor world. A `GeneratedFileKind::Companion` names its package
and sibling file; the helper adds an `include!` at the corresponding `PackageMod`. It does not
generate Connect/gRPC dispatch itself. It is non-idempotent, and an unmatched package leaves an
orphaned file without automatic inclusion. A target integration must validate package membership,
filenames and collision freedom once and apply the composition once.

This is preferable to copying Buffa's package-tree rules into every generator. It is not a
requirement to force the existing separately generated Connect and validator trees into one file:
use it when the generator really emits compatible package-scope companions. Current plugin
orchestration already has a valid independent-output route.

The dedicated validator generator exposes `scan::gather`, `emit::render` and
`emit::render_with_options`. Its `Options` controls packaging, file-per-package output and module
placement. Programmatic composition could remove subprocess/temporary-file work in preparation,
but would couple the orchestrator to generator Rust APIs. Retain the plugin route unless that
complete preparation simplification justifies the coupling. The maintained local lowerer and
preflight must remain in either realization.

### Share representation mappings explicitly

Buffa `extern_path` maps protobuf types/packages to existing Rust owners; specific mappings
override broad package mappings. The message generator, packaging, service generator and
validator must agree on those mappings and naming conventions. Excluding a package preserves
its descriptors for option resolution but does not supply missing Rust field types.

Use custom storage only at measured or structurally consequential boundaries. Buffa supports
custom string, bytes, repeated, box and map representations, but map positions and generated
auxiliary traits impose additional constraints. An ordinary native domain type is not made a
valid protobuf field representation by naming it in `extern_path`. It must satisfy the required
codec, reflection, JSON and view contracts for its actual position. Invariant-bearing native
construction remains a distinct fallible adoption step.

## Complete alternatives and comparative conclusion

| Design | Complete useful route | Why it is or is not the preferred foundation |
| --- | --- | --- |
| Owner-derived hybrid | Domain declarations → descriptors and adapters → Buffa/Connect clients → native admission/execution → domain results plus generated inspection. | Preferred for first-party scientific and runtime contracts. It preserves executable domain owners while removing repeated mechanical boundary interpretation. Its generator complexity is justified only where additional projections share actual meaning. |
| `.proto`-first | Authored schema/options → generated Rust messages/validators → fallible domain conversion → storage/execution/results. | Preferred for independently governed wire standards and possible independently authored extension packages. Making all scientific authority Proto-first would still require dimensional, formulation, admission, hashing and execution behavior elsewhere; it risks an additional editable semantic mirror. |
| Rust-first `proto_rs` | `#[proto_message]`, explicit tags, `#[proto_rpc]`, schema export and shadow conversions → wire services → native operations. | Credible for a narrow native-owned contract. Published 0.11.26 provides Rust-first messages/services, `sun`/`sun_ir`, `ProtoShadowEncode`/`ProtoShadowDecode` and schema export. It does not automatically provide the registry's Arrow/database/documentation responsibilities or the existing Buffa/Connect validation composition. |
| Native-only | Existing typed calls, owner validation and Serde/document projections, without a service or protobuf record. | Retain for internal operations without an independent consumer or persistent contract. Adding protobuf there can introduce encoding, descriptor and migration machinery without removing work. |

The Rust-first comparison is not rejected because it differs from today's stack. Its strongest
case is avoiding separately generated shadow structs where native types already express the
right API. Its weaker complete case here is replacing several already-composed projection,
validation, reflection and Connect consumers at once. A release-specific future comparison can
reopen that choice if those complete capabilities improve.

Do not credit published `proto_rs` 0.11.26 with development 0.12's transport-neutral service layer,
`EncodedSnapshot`, patched Tonic owned-buffer handoff, or experimental Linux zero-copy route.
Those are distinct upgrade/research branches, not prerequisites for the recommended local target.

## Evolution, deletion and extension controls

Treat wire package major, semantic interpretation, validation policy, database mapping and Rust
source compatibility separately. Buf breaking checks can detect selected descriptor-level
incompatibilities; they cannot establish unchanged units, datum, mathematical interpretation or
constraint enforcement. Explicit tags and reservations remain necessary even when compatibility
is intentionally broken. A new major should have one active target path, with retained evidence
readers bound to their original interpretation where required.

When the generated replacement covers a complete consumer population, remove hand-written field
maps, duplicated local scalar checks and parallel document schemas whose meaning is now generated.
Retain scientific admission, transport framing, storage transaction policy and native lifecycle
owners. A schema-derived validator cannot replace these responsibilities.

Use three revealing extension cases before adopting the target:

1. Add a dimensioned provider request with an alternative and a local cardinality constraint.
   Its definition should generate matching Rust adapters, descriptors, validators and inspection
   metadata while physical datum/derivative admission remains executable at the domain owner.
2. Add a structured solver diagnostic with source and variable identities. The generated service
   and an independent reader must retain those identities without another string-based classifier.
3. Add a new result projection to an existing operation. Only useful metadata/control should
   acquire a protobuf contract; existing Arrow payload ownership must not be forced through
   dynamic protobuf merely to obtain discoverable metadata.

These are proposed discrimination cases, not executed extension or conformance tests. The
integrated comparative account owns cross-domain selection and subsequent decision routing.

## Sources and evidence limits

Context7 resolved `/anthropics/buffa` and `/websites/rs_prost`; its answers supplied capability
leads, not release authority. Decisive Buffa 0.9.2 `lib.rs`, `reflect.rs` and reflection APIs and
Prost 0.14.4 `message.rs` were inspected in the installed crate sources. The local maintained
validator README and `scan.rs`/`emit/mod.rs` bound its composition and coverage.
Context7's `proto_rs` resolution returned unrelated libraries; the published 0.11.26 crate page
supplied the direct follow-up for its messages, shadows, explicit tags and Tonic service route.

- [Buffa 0.9.2 generation, companion and diagnostics source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs).
- [Buffa 0.9.2 descriptor embedding source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/reflect.rs).
- [Prost-build 0.14.4 configuration](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html).
- [Maintained validator contract](../../../../../vendor/protoc-gen-protovalidate-buffa/README.md).
- [Buffa overview and advanced generation addendum](../../../../libraries/proto/Buffa.md).
- [Compilation/orchestration reference](../../../../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md).
- [Generator ecosystem reference](../../../../libraries/proto/protobuf_generator_ecosystem_rust.md).
- [Rust-first reference, including release/development distinctions](../../../../libraries/proto/Rust_first_proto_contracts_and_proto_rs.md) and [published proto_rs 0.11.26](https://docs.rs/crate/proto_rs/0.11.26).

No whole-workspace semantic reference census was prepared. Source observations cover the listed
owners and adjacent generator consumers; they do not prove every protocol mapping is duplicated
or that every native type should become a public contract.
