---
title: "Plan65a: Contract foundations and generation"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md]
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

Keep actual source relationships separate. An owner path/type association is not an exact Rust
source span. Current generated `SourceCodeInfo` entries use placeholder spans and comments for
owner/version/direction/facets; method effects appear in documentation and an action sidecar.
Dynamic option inspection cannot recover data carried only there. After the typed options
serve all selected machine consumers, remove comment parsing and duplicated machine-policy
sidecars. Human documentation may render the same metadata without becoming another authority.

## Generation and artifact flow

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
| A01 | planned | Post-Plan64 baseline and required ADR decision → pure structural foundation plus owner-local domain adapters | Move graph/macros/adoption interfaces, update all current product/codegen consumers and selected host declarations, remove the old product-owned generic implementation. Compile dependency boundaries and test recursive types, private fields, fixed IDs, unsupported types and fallible construction; no forwarding compatibility module. |
| A02 | planned | A01 field/method semantics → versioned typed metadata option schema and owner emission | Generate owner/version/direction/facets/effects once; migrate machine consumers from comments/sidecars. Test missing/unknown options, source association without invented spans, collisions, option import closure and a new physical facet reaching inspection. |
| A03 | planned | A02 metadata plus declared effective feature settings → complete and runtime contract-world artifacts | Replace direct full-byte runtime embedding and inconsistent override handling. Test nondefault feature changes against generated/reflected semantics, runtime compaction preserving typed options, full-source retention and immutable old-world readers. |
| A04 | planned | A03 closed worlds → shared generation composition for all four initial boundary populations | Integrate Buffa, packaging, validation, Connect and selected external Prost generation. Remove repeated module/descriptor assembly and hand-kept plugin naming maps. Test warning/refusal propagation, orphan companions, duplicate filenames, excluded packages and exact import identity. |
| A05 | planned | A01–A04 narrow working slice plus B01/B02 adoption policy → complete native/protocol correspondence for selected populations | Migrate host native request/fact families, engineering/worker/reference structural adapters, domain-owned document boundaries, and c/f/g consumer projections. Inventory actual remaining maps at implementation baseline; classify semantic checks explicitly and remove all replaced mechanical maps. Test native→wire→native against independently constructed expected meaning, raw paths and missing/unknown alternatives. |
| A06 | planned | Current registry/function owners plus A02/A03 → source-declared selected storage/Arrow projection metadata consumed by C packages | Reuse source facts without decoding ProtoJSON or inferring database behavior from field spelling. Delete duplicate selected field/schema declarations with producer/consumer migration. Preserve transactional/domain-specific logic; test precise numeric, relationship and mapping-version changes. |
| A07 | planned | Complete selected consumer migration → regeneration equivalence, extension controls and handoff | Add provider request, method effect, diagnostic and result facet through their actual owners and verify every selected projection follows. Run package compile/targeted checks during work and generation when declarations change; scope-end assembled/static checks belong to 65h. Delete remaining obsolete fixtures and parallel publication paths. |

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

Current source: [graph](../../crates/pse-model/src/boundary_projection.rs),
[owner adoption](../../crates/pse-model/src/boundary_projection/access.rs),
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
