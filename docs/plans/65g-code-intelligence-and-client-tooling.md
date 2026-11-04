---
title: "Plan65g: Code intelligence and client tooling"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, libraries, preparation, validation]
---

# Code intelligence and client tooling

## Target and baseline

**Proposed:** make generated contracts, faithful native adapters and complete descriptor bundles
usable by code intelligence, the native CLI, offline inspectors, independent local clients and
scientific references. One reusable inspection/adoption infrastructure serves these consumers;
their domain meaning, permissions and lifetimes remain explicit. The independent host must
not import product mathematics, Arrow or runtime merely to understand a contract.

Joint execution uses the current native baseline alongside unfinished Plan64. This series may replace its interfaces,
schemas and code wherever the complete target improves. Inspected source at
`f61286a5fa5894b480efe8584c8b3d9b76d1b1fb`, including dirty projection changes and supplied
reference additions, is **Interface-checked** only. No compiler/client/reference journey or
tool availability check ran during authoring. The [coordinator](65-proto-integrated-target.md)
owns the combined target; G01–G06 below own this companion's work.

Current `pse-host::code_intelligence_contracts`, `semantic::wire` and `wire::{graph,investigation}`
provide generated messages plus handwritten guarded native mappings. `schema::payload` already
interprets selected descriptor worlds; its strict JSON checks reject duplicate token keys,
unknown generated fields, duplicate protobuf/JSON aliases and trailing data. API presentation
still has a supplier-to-local JSON intermediate. Runtime reference exchange distinguishes
checked native adoption from retained original messages; the isolated Python helper uses a
private descriptor pool and `message_factory.GetMessageClass` without computing answers.
These are actual consumers to migrate or retain, not absent capabilities to reinvent.

## Selected architecture

Use [65a](65a-contract-foundations-and-generation.md)'s pure `pse-contract` metadata/adoption
interfaces at native host owners and tooling generation. Keep product quantities/science
adapters at their owners. The host can interpret approved PSE descriptors as bytes and typed
options without linking scientific execution. Custom options carry stable meaning/effects;
source relationships carry actual provenance. Source comments and placeholder descriptor
spans do not become an executable metadata or byte-coordinate contract.

Generate mechanical native/wire correspondence for the current host request, configuration,
fact, dependency-use, graph and investigation products. Retain fallible hooks for context
guards, raw paths, feature/provider selection, checked numeric widths and opaque original
provider documents. Three-valued predicates, original versus adjusted types and compiler
observations remain distinct. Do not generate a universal HIR model or imply that structural
matching proves trait obligations, liveness or numerical behavior.

The selected representation is native checked host owners plus generated Buffa projections.
Handwriting every map repeats structural edits; interpreting all operations dynamically loses
useful static/domain checks. A second Prost reflection world is retained only for independently
governed external protocols, not introduced as a universal in-memory translation layer.
Published Rust-first alternatives remain technically eligible but must outperform the complete
generation/validation/client route rather than a shorter message declaration.

## Descriptor bundles and offline inspection

Deliver each coherent contract family as a versioned immutable bundle: complete dependency
closure, effective descriptor semantics, typed metadata options, original source associations,
validation-policy identity and supported interpretation. A compact runtime pool, full source
bundle and disclosed service-reflection bundle are distinct selected artifacts. Bind a payload
to its original approved world. Matching full names or arbitrary type URLs alone do not
establish equivalent meaning; descriptor-local indexes never cross pools.

Extend the existing payload/schema engine rather than write another field walker. Buffa
`DescriptorPool`, `file_containing_symbol`, descriptor field/oneof/enum APIs and
`DynamicMessage::from_options` drive generic discovery of local rules, effects, physical facets
and available products. Generated vtable reflection can inspect owned messages without a
dynamic round trip. Dynamic editing uses checked operations and then explicit native adoption;
it does not confer exact-parent mutation semantics.

The offline command accepts an explicit descriptor bundle, message identity, encoding and
bounded payload. It reports structural decode, local validation, known interpretation and
contextual admission separately; offline inspection normally cannot perform the last act.
Preserve unknown numeric enums/fields for forwarding where selected, and distinguish exact
original bytes from a normalized re-encoding or ProtoJSON view.

Unify binary/JSON/text policy with [65b](65b-representation-validation-and-interpretation.md).
Retain strict JSON protections while removing repeated parsing/tree construction only after
the replacement establishes equivalent behavior. Generated unknown-field tolerance differs
from dynamic JSON strictness. Explicitly support selected WKT semantics and `Any` URL policy;
do not blanket-map `.google.protobuf` to an implementation with different fidelity.
Descriptor-provided source comments are useful explanation, not exact Rust byte ranges.

## Independent local clients and reference exchange

### Early developer-evidence inspection

[65j](65j-workflow-lifecycle-and-developer-evidence.md) adds actual test reports,
inventory/failure references, fixture context and exact rerun to independent-reader consumers.
J01 defines their meanings before A01/A02 harden. G supplies inspection/client presentation
while native test owners perform execution, reconciliation and rerun.

G02 has a narrow early slice consuming working A/B artifacts and existing `schema::payload`
directly, without awaiting every G01 family. Use Buffa `DescriptorPool` and
`DynamicMessage::from_options` for selected worlds, and generated `ReflectMessage::for_each_set`
where an observational record already exists. The [0.9.2 reflection source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs)
and [runtime reflection source](https://github.com/anthropics/buffa/tree/v0.9.2/buffa/src)
characterize these separate dynamic/vtable routes. Successful decode or generated `Validate`
cannot construct a native verdict or settlement capability. Reuse A's shared semantic-option
checker for import/offline validation and old/new interpretation comparison; do not duplicate
its meaning in a tool-specific field walker.

Preserve original invocation/action, selection/inventory/report identity, executed population,
fixture requirements, assertion outcome, capture completeness and retained-reader status.
Exact rerun requests go to native `test_workflow::source` after original evidence and current
listing reconciliation, never command reconstruction from displayed failure text. Scientific
inspection consumes I's operation/product semantics and original decisions without minting
current admission. Bounded generated failure inputs retain bytes/seed/world/native test identity
for replay through J; they do not create a separate fuzz-result store.

Generate native Rust clients and publish complete offline descriptor bundles for independent
tools. One Rust client over generated bindings is the product-local route; a standard tool
or separately generated runtime must supply independent interoperability evidence. `grpcurl`
can use reflection or `-protoset`; `buf curl` can select Connect/gRPC/gRPC-Web and streaming
JSON. Select by actual Unix transport and stream shape. `granc-core` is useful Rust tooling
but shares Tonic infrastructure and is weaker evidence of independent runtime behavior.

Use the authenticated prepared endpoint and original native operations from
[65d](65d-local-operations-and-services.md). Help, descriptor inspection and client construction
must not prepare/build/start science. The client can discover/inspect, edit exact-parent
source, prepare, submit original work, observe/cancel and read protected products. It retains
terminal failures after partial results and never automatically replays an uncertain submit.
Health, reflection and connectivity are operational facts rather than scientific readiness.

Scientific references share only structural case/configuration/outcome contracts. Their actual
calculations, physical conventions and expected values are constructed independently. Generate
or dynamically load selected language bindings within the existing isolated reference
environments. Keep Arrow for useful tabular outputs and Proto envelopes for identity,
configuration, outcome and provenance. Unknown-field forwarding does not adopt unsupported
science; stale policy/mapping/physical interpretations refuse or migrate explicitly.

## Packages and acceptance

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| G01 | in-progress | Current Plan64 source baseline; 65a pure contract and bundle slice | Developer report/diagnostic/path/test-identity owners now supply native source declarations and one-way captured roots. Remaining configuration/request/fact/graph/investigation families must preserve fallible context/path/provider hooks, guards, raw paths, original/adjusted facts and unknown alternatives, with every replaced map removed alongside its migrated callers. |
| G02 | in-progress | Working narrow A/B family artifacts; G01 only for host families it migrates; I01/J01 design | First developer-evidence bundle and compiled reader are implemented; family/population qualification remains. Extend the same inspection to engineering/science/diagnostic worlds. Test option preservation/evolution, missing imports/interpretation, cross-pool identity, nesting and valid observation without native authority. Reuse the shared semantic checker; remove duplicate acquisition/walkers and JSON passes only with equivalent admission. |
| G03 | planned | G02; 65d working native operation/client route | Deliver generated client packages, explicit offline/reflection discovery and independent local invocation through full operation/readback. Client-side message limits, terminal errors, reconnect auth, unauthorized reflection and expired protected results must remain observable. Delete superseded client-specific contract copies/dispatch translations. |
| G04 | planned | G02; narrow E/F scientific/outcome and J03/J04 developer integrations according to family | Migrate independent reference helpers, diagnostic/developer readers and exact-rerun clients to approved bundles and generated/dynamic types. Test unknowns, exact bits, changed datum/formulation, original report/inventory binding, retained-context failure and complete coverage with independent expectations. Delete obsolete exchange/rerun maps while preserving authority and numerical independence. |
| G05 | complete | Exact pinned rustdoc supplier and curated consumer population; G01/G02 only if a new projection cutover is selected | Retained the bounded Serde bridge after source comparison against a complete recursive conversion closure. Four native tests passed for recursive types, attributes, unavailable shapes, HRTB/capture/stability and format refusal against zero failures. The decision and revisit conditions below preserve the full curated contract without widening accepted formats. |
| G06 | planned | G01–G05 selected consumers; 65a/b generation and fidelity controls | Complete developer-tool integration: compiler/plugin/edition/external mappings, explicit preparation, actionable generation diagnostics and Buf compatibility policy. Use one independently driven client/reference journey per actual supported contract family; reconcile every migrated consumer/deletion and supply evidence/limits to 65h. |

G01's full population includes J01-selected developer observational families with no inverse
native verdict/control constructors. G06 also reconciles J's completed developer reader
population without owning its verdict or lifecycle. G04's independently constructed cases
retain intended failures as well as successful outputs. J's developer readers can proceed
independently of E's scientific references, consuming their respective ready slices.

G05 is a bounded decision package: compare exact supplier/public type correspondence and
construction cost using one recursive signature population plus unknown/new-format controls.
Its decision changes only the API projection path. It does not block unrelated client/schema
delivery. A typed mirror that adds more upkeep is not an improvement merely because it avoids
`serde_json::Value`; retain the simpler bounded route when it wins the complete comparison.

### G05 supplier comparison and selected retention — 2026-11-04

**Selected:** retain `contracts::project` for the existing native schema61 presentation
contracts. This is the package's permitted retain outcome, rather than a missing Proto
migration. Input still passes `api::decode`'s `rustdoc_types::FORMAT_VERSION` preflight before
typed decoding; new format numbers refuse before version-specific fields are interpreted.
No supplier format, acquisition context or availability promise changes.

The exact supplier is `rustdoc-types =0.61.0`, its
[typed schema source](https://docs.rs/crate/rustdoc-types/0.61.0/source/src/lib.rs) and
[feature declaration](https://docs.rs/crate/rustdoc-types/0.61.0/source/Cargo.toml).
Its `Generics`, `Type`, `GenericArgs`, `GenericBound`, `Attribute`, `FunctionSignature` and
stability structures already supply the accepted Serde representation. Its selected release
does not supply the `JsonSchema` implementation needed by independent command-schema clients.
The local curated types add that public schema while preserving the supplier's fields and
enum spellings, including recursive constraints, exact IDs, HRTBs, precise captures and opaque
`Attribute::Other`. That last carrier remains uninterpreted text; it cannot supply a capture,
availability or stability fact. Rustdoc's [JSON documentation](https://doc.rust-lang.org/stable/rustdoc/unstable-features.html)
characterizes its experimental output; the repository's pin and format gate define the
actually accepted population.

The retained mechanism uses `serde_json =1.0.151`
[`to_value`](https://docs.rs/serde_json/1.0.151/serde_json/fn.to_value.html) followed by
[`from_value`](https://docs.rs/serde_json/1.0.151/serde_json/fn.from_value.html). It constructs
one in-memory `Value` and drives the target type's deserializer; it does not format and reparse
JSON text. The intermediate does allocate and traverse the selected tree. Conversion failures
remain `ApiError` or explicit unavailable shape in the existing pattern consumer. These are
ordinary Serde contracts, not ProtoJSON behavior or a generic native-admission mechanism.

The compared direct route would need an exhaustive recursive conversion closure for the
supplier's 14 `Type` alternatives and their path/argument/constraint/bound/generic/signature/
attribute/stability dependencies. Those fields are consumed by contract presentation, pattern
queries, implementation adjacency and precise-capture reporting. Handwriting that closure
adds another field/variant correspondence beside the supplier and curated schema. Generating
it requires a supported foreign-type declaration source that emits the complete curated schema;
the selected mechanism does not currently provide one. Adding a separate Proto vocabulary
would add yet another representation and generation/feature obligation without removing the
native supplier or public command schema. Source inspection supports retaining the bounded
bridge on total machinery; no allocation or latency improvement is claimed or measured.

Independent fixtures cover every current type alternative, nested function signatures,
borrowed/qualified/associated shapes, all generic argument forms, constants and constraints,
the selected attributes, opaque unknown attribute text, and missing or unrecognized shapes.
Existing controls retain generics/HRTB/capture/stability distinctions and new-format refusal.
These establish mechanical correspondence only, not API availability or compiler truth.
No fresh rustdoc acquisition was required for this source-backed bounded comparison.

**Tested:** native `scripts/pse test pse-host --lib` with the selector
`test(schema61_projection) | test(schema61_contract_keeps_association_hrtb_capture_and_stability_separate) | test(format_preflight_refuses_new_version_before_version_specific_shape)`
reconciled four selected/completed tests, zero failures, complete capture and settled original
ownership. This closes G05's bounded comparison and retain disposition; it establishes neither
new-format support, current compiler/API availability nor assembled G/H qualification.

Revisit if a pinned supplier supplies the consumed public schema directly, or the shared
generation foundation can derive every curated shape and failure distinction without a second
manual declaration/conversion forest. Select a replacement only when that complete route
removes machinery, preserves all consumers and has matching tests; measure whole-query work
before claiming a performance benefit.

G01/G02 cover the finite existing family population, not just a demonstration message. Additions
to native facts extend the owner declaration once. Tests of compiler fact truth remain with
the actual provider and independent assertions; generated round trips test correspondence only.
All fixture population and preparation changes migrate with their affected client/owner.

## Extension, evidence and handoff

Challenge a new scientific quantity/alternative, a diagnostic product, an unknown future
capability and a source rename. Existing generic inspection should reveal supported metadata
without another handwritten walker. Domain implementation and admission still change when
meaning changes. Exact UTF-8 byte/source witnesses remain source-owned, never reconstructed
from descriptor columns or fabricated spans.

Compiler/backend API compatibility, protobuf conformance, Connect/gRPC interoperability,
validation semantics, authenticated local custody and scientific equivalence are different
claims. Library conformance is necessary selected evidence, not assembled product acceptance.
Keep failed/unsupported configurations explicit and preserve the zero failure baseline.
Scope-end qualification is in [65h](65h-integration-and-qualification.md); no product test,
tool-installation or performance claim is made by these plan documents.

The pure dependency/crate and public metadata decisions follow the coordinator's proposed
ADR route before implementation. Client/document/tooling changes within those settled contracts
need no invented additional policy approval. Final cutovers remove replaced schemas, walkers,
mapping pairs, fixtures and compatibility dispatch paths together; retained historical readers
are explicit interpretation consumers rather than an indefinitely parallel product path.

## Sources

- [Client/intelligence/reference discovery](../library_leverage/libraries/proto/briefings/clients-code-intelligence-and-reference-tooling.md), [generation](../library_leverage/libraries/proto/briefings/contract-ownership-and-generation.md).
- [Buffa 0.9.2 guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md), [ConnectRPC 0.9.1 guide](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md), [grpcurl](https://github.com/fullstorydev/grpcurl), [Buf curl](https://buf.build/docs/curl/usage/).
- [Reflection/PBJSON](../libraries/proto/prost_reflection_and_pbjson.md), [compiler/orchestration](../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md), [Rust-first alternatives](../libraries/proto/Rust_first_proto_contracts_and_proto_rs.md).
- Source owners: `crates/pse-host/src/{code_intelligence_contracts,semantic/wire,schema/payload}.rs`, `api/query/contracts.rs`, `crates/pse-runtime/src/reference_exchange.rs` and `tests/reference/proto_exchange.py`. Inspect their post-64 delta before execution rather than treating current bodies as immutable target authority.
