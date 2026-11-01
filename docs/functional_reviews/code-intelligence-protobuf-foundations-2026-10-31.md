---
title: Protobuf foundations for existing code intelligence — candidate and documentation review
status: proposed
date: 2026-10-31
doc_role: review
doc_owner: docs/functional_reviews/README.md
doc_topics: [design, libraries, diagnostics, environment]
---

# Protobuf foundations for existing code intelligence

## Judgment and milestone boundary

**Proposed:** improve the existing evidence contract and examine replacement of the local
helper's custom communication machinery. Treat these as separable opportunities. Protobuf
can support the first without gRPC; generated local RPC can support the second without a
remote deployment. No runtime stack is selected at this milestone.

The strongest immediate investment is detailed documentation of competing foundations and
their compositions. Do not adopt the supplied ecosystem overview's entire preferred bundle.
Prost/Tonic, Buffa/ConnectRPC and Rust-first options make materially different choices about
declaration authority, representation, validation and transport. Installed generators and
diagnostic tools lower investigation effort but do not settle those choices.

This is the authorized first milestone: opportunities, candidate dispositions, preliminary
target alternatives and a documentation handoff. Stop before final architecture selection.
After the commissioned documentation is available, refine this same principal review; selected
implementation then follows through normal plan/decision routes. Reconsider Plan60 after the
foundational improvement is implemented and exercised. Its future features are neither an
implementation prerequisite nor the sole justification for this work. No Plan60 status changes
or production changes are made by this review.

## Scope, baseline and evidence

Inspected repository baseline: `f78b3a5bb` (which commits the supplied library/tool documents).
The delivered CLI and native helper are the subject. Source-only discovery, semantic/native
facts, API refinement, original diagnostics and source proposals are consumers to preserve;
this is not a rewrite of LSP, rustdoc, the compiler or every CLI command.

Criteria are Core3.4, Efficient Architecture1.0 and the pse-arrow binding. Test Assessment1.2
informs independent evidence and Process Simulator1.5 limits interpretation: a transport or
schema cannot establish scientific/model correctness. No broad diagnostic audit or conformance
certification is claimed. AP-04/AP-05 and DP-01–DP-03 favor explicit meaning and owned
construction; AP-07 and heuristics H4/H6/H16/H27 favor removing unnecessary conversions and
coordination while assessing the complete generated and runtime machinery.

**Interface-checked:** current source contracts and the library/tool mechanisms cited below.
The full [ecosystem overview](../libraries/rust_protobuf_ecosystem_overview.md) and
[installed-tool inventory](../libraries/protobuf-grpc-installed-tools.md) informed the inquiry.
The [Prost](../libraries/prost.md) and [Tonic](../libraries/tonic.md) references were read during
preparation and their consequential limits revisited against source and current pins. Context7
supplied discovery; exact release documentation, installed sources and upstream repositories
settled the qualifications identified below. Main-branch documentation is labelled as such.

Executable discovery and selected version/help/build-metadata observations from preparation
establish local availability only. Selected `protoc` reports36.2, `/usr/bin/protoc`3.21.12,
Buf1.75.0, Granc0.7.7 and the three Rust plugins0.5.0/0.5.0/0.4.0. grpcurl's version display
says development build, while embedded Go metadata identifies modulev1.9.4. The JSON Schema
plugin's metadata identifies `github.com/pubg/protoc-gen-jsonschema`v0.8.0. Executable name
and displayed version alone are therefore insufficient identity evidence.

No generator, candidate server, new native preparation, load test, packet capture, collector,
benchmark or product qualification was run. No runtime or context-saving improvement is
Tested or Measured. Source reads through `pse inspect` are inspection evidence only.

## Decision-shaping research results

These qualifications refine the supplied overviews; they are not adoption decisions.

- **Buffa0.9.2:** published archives contain text support and `buffa-descriptor`
  feature-gated `DescriptorPool`, `DynamicMessage` and reflection interfaces. Its package
  README nevertheless says text is unsupported, and default-feature rustdoc hides reflection.
  Reconcile release source, enabled features and generated-code prerequisites; do not infer
  absence or main-only support from those pages. Newer main-guide lazy-view claims still
  need release qualification. Sources: [runtime archive](https://static.crates.io/crates/buffa/buffa-0.9.2.crate),
  [descriptor archive](https://static.crates.io/crates/buffa-descriptor/buffa-descriptor-0.9.2.crate).
- **ConnectRPC0.9.1:** its [released API](https://docs.rs/connectrpc/0.9.1/connectrpc/)
  exposes four RPC shapes and Connect/gRPC/gRPC-Web. Its Buffa representation is a real
  stack choice; a suitable Unix-socket realization remains a question to qualify.
- **prost-protovalidate0.6.0:** released documentation covers both backends, direct
  standard-rule validators, runtime-only rule policy and Buffa `runtime_bridge`. Edition2023
  normalization compensates for prost-reflect0.16's Editions limitation. Buffa's schema
  support therefore does not establish unrestricted composed validation. Investigate bridge
  conversions and `fail_on_runtime_only`: [build API](https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/),
  [runtime API](https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/).
- **Rust-first alternatives:** [Tarpc0.38.0](https://docs.rs/crate/tarpc/0.38.0) merits
  a local-RPC account. [proto_rs main](https://github.com/4r33x/proto-rs), manifest0.12.0,
  documents a bundled Tonic patch for its optimized owned route and an upstream buffered
  alternative. Exact release retrieval failed during this inquiry: the patch condition is
  main-source evidence, not established released behavior.
- **Installed generators:** prost/tonic plugins0.5.0 declare0.14-family dependencies;
  generated Tonic code uses `tonic_prost::ProstCodec`. Serde plugin0.4.0 delegates to
  pbjson-build0.8.0, distinct from the overview's0.9.0 family. Insertion-point output needs
  coordinated Prost generation/include order and consistent WKT/extern mappings; separate
  invocations need the documented alternate include arrangement. Runtime compilation remains
  untested. Sources: [Prost plugin](https://docs.rs/protoc-gen-prost/0.5.0/protoc_gen_prost/),
  [Tonic plugin](https://docs.rs/protoc-gen-tonic/0.5.0/protoc_gen_tonic/),
  [Serde plugin](https://docs.rs/protoc-gen-prost-serde/0.4.0/protoc_gen_prost_serde/).
- **JSON Schema:** installed PUBG0.8.0 is not the similarly named chrusty project. Defaults
  differ from ProtoJSON int64 strings. Its presence option also needs source inspection before
  use as an ingress validator: making presence-bearing fields required must not reject valid
  omitted optional fields. An option name is not a conformance guarantee. Source:
  [exact README](https://raw.githubusercontent.com/pubg/protoc-gen-jsonschema/v0.8.0/README.md).

Context7 coverage was uneven: some Rust ConnectRPC/proto_rs/plugin searches returned unrelated
libraries. Primary sources were used rather than inferring capability absence. Two bounded
independent research contributions covered runtime combinations and the installed toolchain;
the coordinator reconciled their qualifications with the code seams below.

## Current responsibilities and useful seams

| Current owner | Observed behavior | Consequence for a foundation change |
| --- | --- | --- |
| `investigation/model.rs` | Typed Request, Truth, guarded references, source contexts and Begin/Source/Result/Diagnostic/Terminal events; serialization-direction Schemars schema | Reuse these distinctions; shared context and stream finality are delivered foundations, not missing features |
| `semantic.rs` | Typed Answer/Failure/context surround category-labelled `Section.evidence: Value`; Reply::Answer carries Value | Normalize consumed meanings where useful while preserving deliberately opaque original backend data |
| `semantic/service.rs` | Authenticated Unix socket, one JSON-line request/reply, cancellation control byte and native settlement acknowledgement | A real local RPC comparison exists today; replacing framing must preserve peer identity and actual CPU completion |
| `semantic/native.rs` and engine/transport | Native snapshot facts and live LSP operations have different contexts and capabilities | Wire uniformity must not invent provider parity or remove native context qualification |
| `api/bridge.rs` | Original artifact/source/Cargo-feature-target qualification before linking API evidence to native context | A descriptor or type URL does not replace this evidence or confer semantic identity |

Paths above are relative to `crates/pse-host/src/`. The enduring owner is
[blueprint §21.7](../authoritative_design/sections/workflows-and-results.md#section-21-7).
Native helper isolation keeps its compatible HIR family separate from ordinary source syntax.
Existing prepared-tool, source, managed-process and completion owners remain relevant whatever
codec wins. This review establishes concrete improvement surfaces, not a finding that every
Value or custom protocol operation is defective.

<a id="O01"></a>
## O01 — Typed evidence with preserved original observations

**Recommend as a design objective; encoding remains open.** An agent should inspect a reference,
type observation, failure or candidate without interpreting a category string and arbitrary
payload anew. Promote fields that consumers actually interpret into declared evidence kinds;
keep original LSP/rustdoc/compiler data available with its own provider/context meaning.

Compare extending current Rust enums/constructors with Protobuf messages and `oneof` payloads.
Prost maps `oneof` to an optional enum, message presence to Option and enum fields to raw integer
representations. Successful decoding does not establish all required fields, valid contextual
joins or application invariants. Strong Rust types can remain the construction boundary even
if generated transport types are selected. Conversely, intentionally shared generated types
can be appropriate when their semantics fit. Do not add a duplicate domain model by default.

Acceptance in a later implementation would include actual existing semantic and API consumers,
a positive fact, unknown/unavailable evidence, a stale reference and mismatched context. A
rendered type spelling or identical field shape cannot establish identity. This benefits
existing inspection and refinement independently of Plan60's new relationships.

<a id="O02"></a>
## O02 — Compact answers and progressive inspection

**Recommend independently of binary encoding.** Preserve current shared source/provider context,
bounded demand, payload projection, guarded expansion and terminal coverage. Extend typed
selection into places that still expose whole sections when a consumer needs a few facts.
An agent should obtain a useful compact answer and expand one exact observation with its
premises intact. Do not concatenate complete alternative artifacts to obtain a comparison.

Descriptor-aware projection, Protobuf FieldMask, generated typed views and ordinary Serde views
are candidates. FieldMask supplies paths, not an implementation of semantic query planning;
presence and omitted-versus-default distinctions still need deliberate meaning. Buffa borrowed
views may reduce copying, but a small view can retain a large backing allocation, and lazy
decoding can change validation/error timing. Investigate exact available APIs before selecting.

Binary byte savings and agent-context savings are different claims. To measure the latter,
hold the question, required evidence and rendered format comparable; include coverage and
expansion needed to reach the useful answer. No token SLA or speed claim is established here.

<a id="O03"></a>
## O03 — Generated communication at the existing local helper boundary

**Recommend a substantive comparison now.** Prost/Tonic can supply generated method dispatch,
typed requests/responses and streaming over a local connector. Buffa/ConnectRPC is a competing
stack. Tarpc is a bounded Rust-first comparator; Protobuf messages with retained local transport
separate representation benefits from RPC benefits. Remote deployment is not a prerequisite.

Map any replacement against the existing one-request connection, peer authentication, preparation
identity, cancellation request and settled reply. A gRPC stream reset or dropped Rust future
does not establish that native/Rayon work stopped. Current native_response cancels and retains
the actual CpuTask owner through settlement. Preserve that property with a native integration,
not a second universal task registry. HTTP/2 flow control does not bound producer allocations.

The comparison must show which framing/dispatch/control code disappears and what generated
services, adapters, buffers and lifecycle work replace it. Retain original errors and terminal
facts through the selected protocol. Tarpc's documented cascading cancellation is a relevant
mechanism, not proof of cancellation of separately running CPU work. See
[Tarpc](https://github.com/google/tarpc) and the supplied Tonic §§4–7.

<a id="O04"></a>
## O04 — One declaration and coherent derived representations

**Recommend; authority choice unresolved.** A field change should have one authored meaning,
with message types, codecs, relevant schemas and documentation following a declared derivation.
Compare schema-first `.proto` ownership, Rust-first generation and selective Protobuf boundary
types against strengthened current Rust/Serde/Schemars ownership.

`prost-build::Config::{compile_fds,extern_path,file_descriptor_set_path}` and generation
attributes can reuse descriptors and compatible external types. Installed protoc plugins offer
another invocation route to these families, not automatically another authority. Buf can
coordinate local plugins; its core local functions do not require hosted registry adoption.
Compare direct build-library generation before adding plugin orchestration. Buf's v2 local
plugin configuration exposes whole-schema versus directory strategies, import/WKT selection
and managed options; these affect meaning and output composition. The Prost0.5 plugin's
`file_descriptor_set` output preserves raw encoded per-module descriptors, including extensions,
but is not automatically a complete transitive descriptor bundle. Compare this with a retained
standard FileDescriptorSet produced by Buf, keeping encoded original metadata where needed.
See [Buf descriptor formats](https://buf.build/docs/reference/descriptors/) and
[generation configuration](https://buf.build/docs/configuration/v2/buf-gen-yaml/).
Choose one owner
for schema dependencies and output publication; Buf, protofetch and protoweld should not each
introduce another resolution or generation layer without a distinct responsibility.

Current Rust output schemas distinguish serialization direction. ProtoJSON has its own enum,
bytes, int64, naming, presence and well-known-type rules. A schema generated from `.proto` need
not describe the existing JSON contract. This question includes public consumers, saved requests,
portable references and native output; a coordinated intentional cutover is legitimate, while
retained artifacts still require their original interpretation. Compatibility tools inform this
decision rather than imposing indefinite internal compatibility.

<a id="O05"></a>
## O05 — Descriptor inspection and useful discovery without a new registry

**Investigate selectively.** An agent could inspect the exact supported evidence/message shape,
its fields and documented conditions without bespoke catalog code. Static descriptors,
`prost-reflect::DescriptorPool`/`DynamicMessage`, typed ReflectMessage integration and optional
service reflection can serve different parts of this need. Existing parser/Schemars discovery
is the baseline to improve, not something to replace by name alone.

Retain custom options and source information where downstream interpretation needs them;
descriptor loading can discard extensions if the route is wrong. Distinguish message type
identity, descriptor artifact identity and code-analysis world identity. `Any` does not infer
the meaning of an unknown payload, and `Struct` is not a replacement for the typed evidence
model. Runtime reflection earns its cost for a named discovery/projection consumer; statically
known CLI operations may need only generated types and exported descriptor files.

grpcurl can list/describe from a protoset without a running endpoint. Prefer that over creating
a service solely for schema exploration. A hosted schema registry and persistent semantic
index remain separate, unselected decisions.

<a id="O06"></a>
## O06 — Reuse declarative validation where it removes duplicate rules

**Candidate, not a mandatory validation engine.** Compare generated direct-field validation,
descriptor-based validation and native Rust constructors for local and cross-field constraints.
The current prost-protovalidate project documents both Prost and Buffa backends; this is a
cross-stack comparison axis. Complex CEL rules may require a runtime bridge and additional
evaluation machinery. Qualify exact rule coverage and unsupported behavior before selection.

Schema rules cannot establish source freshness, provider applicability, completed CPU work or
the validity of a cross-artifact API/native association. Keep contextual checks under those
owners. Prefer structurally impossible invalid states where practical, not a second runtime
rule system duplicating a Rust invariant. A later consumer test must challenge absent/default
presence, invalid combinations and unsupported rules; passing a wire decoder is insufficient.
See [implementation and build documentation](https://github.com/zs-dima/prost-protovalidate).

<a id="O07"></a>
## O07 — External inspection and diagnosis instead of custom tooling

**Recommend reuse during research and future operation.** The installed tools can eliminate
throwaway inspectors and offer independent views of generated contracts. This does not require
wrapping every command in pse or embedding every client's library. Prefer one scriptable client
for routine interrogation, retaining other tools when their distinctive capability helps.

| Tools | Useful role | Limit or selection condition |
| --- | --- | --- |
| Buf build/lint/breaking/generate; protoc | Descriptor/generation experiments and classified schema changes | Formatting/fix modes only in explicitly owned scratch; no blanket no-breaking policy |
| Rust/Serde/JSON Schema/doc plugins | Inspect actual generated representations and publication routes | Output compatibility with chosen runtime/serializer must be checked |
| grpcurl; buf curl; granc | Scriptable schema discovery/invocation, descriptor-file or reflection routes | Current JSON-line semantic socket is not a gRPC endpoint; Granc embedding is a separate choice |
| grpcui; Evans; protolens | Interactive exploration of a question that benefits from it | No interactive-tool requirement for ordinary agent use |
| protoscope; prototext | Inspect field presence, unknown wire data and conversion behavior | Byte preservation does not establish semantic canonicalization |
| protoscan; reproto; sample instantiation | Recover metadata or form examples when originals are unavailable | Reconstructed schemas/samples are not authored authority or independent semantic truth |
| grpc_health_probe | Query a standard service's reported status | Does not independently establish native preparation or drain |
| otelcol-contrib; existing tracing | Inspect selected request/stream outcomes and traces | No collector deployment or new telemetry stack is required merely to assess a codec |
| strace/socat/nghttp; packet/kernel diagnostics | Resolve a specific socket/framing/stall question | Use the relevant owned workload; do not start broad capture/profiling by default |
| hyperfine; ghz; h2load; flamegraph | Named CLI, RPC, HTTP/2 or CPU measurements | Their workloads and claims differ; no mandatory benchmark campaign |
| Go/C++ message and RPC generators | Optional independent interoperability challenge | No new production language; compiler/runtime pairing matters |
| gateway/OpenAPI generators | Conditional external/browser interface study | No current requirement for a gateway, REST service or second public API |

Use the repository environment/ownership routes for actual workloads. Preserve its pinned Rust
toolchain even though stable Rust is installed. Tool availability reduces setup work; it does
not authorize broad captures, service startup, publication or tool upgrades. No new diagnostic
framework or receipts ledger is proposed.

<a id="O08"></a>
## O08 — Explicit generation and evolution without development blockers

**Recommend as a target property.** One selected derivation should reproduce outputs from
authored inputs, generator settings and exact tool/runtime families. Buf lint/breaking and
native compilation can expose different failures; neither establishes the meaning of every
generated view. Reuse current codegen/publication owners and delete replaced code/callers/tests
when the selected replacement is exercised. Do not retain a second production path as evidence.

Distinguish source compatibility, wire compatibility, JSON compatibility and retained-evidence
interpretation. Ordinary development can use coordinated rebuild/cutover where no independent
reader requires coexistence. A new wire format does not create a requirement for durable
recovery, hosted distribution or rolling upgrades. Protocol bytes are not automatically
canonical; continue to define what content identities actually claim.

## Preliminary complete alternatives

| Alternative | Potential value for today's tooling | Residual questions before selection |
| --- | --- | --- |
| Strengthen current Rust/Serde/Schemars | Typed consumed payloads and compact views while retaining current owners | Can it also simplify communication sufficiently? Which generic transport code remains justified? |
| Protobuf data contract, current local transport | Generated message/descriptor ecosystem without HTTP/2 lifecycle | Does the codec/generation benefit exceed conversion work? What framing remains? |
| Prost/Tonic with selected companions | Established generated messages/RPC and standard external inspection | Actual deletion of custom machinery, exact JSON contract, cancellation acknowledgement and peer authentication |
| Buffa/ConnectRPC | Competing owned/borrowed representations and service/protocol composition | Exact released reflection/text/view support, local transport and full integration lifecycle |
| Rust-first proto_rs or Tarpc | Keep Rust declarations primary while generating schemas or local RPC | Published generation semantics, interoperability and patch requirements; preserve typed meaning rather than derive a second editable contract |

These are alternatives, not a request for five implementations or exhaustive cross-product
benchmarks. Narrow using documented semantics first. A small probe is justified only where an
unsettled fact changes the decision. Additional protocols, Editions, zero-copy and unknown-field
preservation are useful capabilities to assess, not automatic requirements for this system.

The provisional interaction is: agent request → validated question/demand → original provider
owner → typed consumed observations plus qualified original evidence → compact answer and
guarded expansion. The helper crossing may use generated RPC; source-only operations remain
local and usable without preparing it. Facts/context, transfer and presentation have different
responsibilities. This sketch selects no new service, persistent store or universal graph.

## Documentation handoff

Commission the following focused documents, grouped by decisions they can change. Document
capabilities rather than prescribe the pse-arrow architecture. Existing Prost/Tonic overviews
need only targeted additions where companion or exact-release coverage is missing.

| Priority and brief | Coverage requested | Decision unlocked |
| --- | --- | --- |
| First — Buffa family | buffa, buffa-build, buffa-codegen/protoc-gen-buffa, buffa-types and buffa-descriptor: exact owned/borrowed APIs, reflection/text/JSON, presence, unknown fields, Editions, decode limits, view lifetimes and released versus main-only features | Whether representation/preservation benefits change the preferred foundation |
| First — ConnectRPC Rust | connectrpc/build, connectrpc-health and connectrpc-reflection, structured errors, Buffa coupling, supported transports, Unix/custom connection route, streaming, cancellation, shutdown, middleware and conformance limits | Whether this is a complete local-helper alternative to Tonic |
| First — Rust-first contracts | proto_rs message/service/schema generation and supported upstream/patched Tonic routes; separate short Tarpc local-RPC/transport account | Choice of declaration authority and whether HTTP/2/Protobuf RPC is necessary |
| First — Reflection and JSON | prost-reflect/build plus pbjson/build/types, their exact family compatibility, dynamic versus static APIs, extensions/source info, WKT mapping and JSON presence/int64/bytes/enums | What can be generated/shared without changing or duplicating current JSON meaning |
| First — Validation | prost-protovalidate runtime/build/types, Prost/Buffa support, direct rules versus CEL/runtime bridge, unsupported behavior, constraints and structured violations | Whether declarative validation removes real duplicated policy |
| First — Compilation/orchestration | protox, protoc and Buf local commands/plugins/images, compiler parity, imported/custom options and source locations; contrast protofetch/protoweld only where distinct | One coherent schema dependency/compiler/generation route |
| First — Installed generator family | Exact protoc-gen-prost/tonic/serde/crate versions/options/output layout/runtime bridge; PUBG JSON Schema plugin and protoc-gen-doc semantics | Compatibility and publication choices that can invalidate a superficially simple stack |
| Supporting — Local RPC and diagnostics | Focused Tower/Tokio/Bytes integration, peer identity, stream/lifecycle behavior, grpcurl/buf curl/granc capabilities, relevant upstream conformance facilities | What generic machinery is delegated and how independent inspection works |

Every brief should provide project/release identity, concrete APIs/commands, composition and
type-family requirements, ownership/allocation behavior, unsupported cases, primary references
and meaningful alternatives. Do not call a library absent merely because Context7 lacks it.
Do not convert current main documentation into a released capability claim. Runtime library
and command-line wrapper compatibility need separate treatment; a plugin's version is not the
version of every library resolved into its executable.

### Screened and conditional coverage

Keep Google protobuf4 and official grpc on a bounded current-status screen; do not confuse
protobuf4 with rust-protobuf3 under the same crate name. Deepen if their distinct runtime or
interoperability properties change the shortlist. Keep quick-protobuf, pbrs, Bilrost and other
codecs as landscape alternatives; Bilrost's lack of Protobuf wire compatibility is a material
choice, not a licensing rejection. Investigate wire-forensics libraries such as protofish and
protobin only if a diagnostic consumer needs them. Tree-sitter proto/textproto belongs in the
deep queue if schema-source search/editing becomes a selected capability; resolved descriptors
and source syntax are complementary, not interchangeable.

Defer hosted schema registries/Kafka adapters, xDS/mesh, REST/browser gateways, TLS-specific
deployment and dedicated telemetry wrappers until their own consumer or failure boundary earns
them. Local Buf use does not require BSR adoption. A local deployment still needs its actual
concurrency and identity semantics, but not every distributed-system mechanism.

## Later discriminating evidence and rule impacts

After documentation arrives, select revealing existing journeys: source discovery→inspection,
semantic facts→guarded expansion, API→contextual refinement, original compiler error→source,
source-bound proposal, and incremental results→terminal completion. Include missing capability,
unknown versus negative evidence, same-spelled unrelated identity, stale source/context,
unusual path bytes, invalid field combinations and consumer closure during native work.

For generation, qualify outputs against the selected runtime and actual serializer on known
presence, int64, enum, WKT and unknown-field cases. Independent Go/C++ or upstream conformance
can challenge a specific interoperability claim; generator round trips sharing one model do
not establish independent semantic correctness. Keep binary/ProtoJSON/validation/RPC conformance
claims separate. No future acceptance is supplied by this first-milestone source inspection.

Potential rule impacts are **Proposed**, not operator-approved: RC01 would choose the governing
declaration/derivation and possibly change current schema-publication meaning; RC02 would choose
a helper protocol/control realization within blueprint §21.7; RC03 would classify consumer and
retained-artifact compatibility and any intentional cutover. Their exact scope depends on the
chosen design. Later plan creation obtains operator disposition and follows the applicable
ADR/design route. This functional review does not substitute for a required formal design review.

## Handoff and evidence limits

The first milestone ends with the candidate set and documentation briefs above. The next
input is the commissioned detailed documentation, followed by refinement of this review into
a supported target recommendation. Production implementation, dependency/tool upgrades,
architecture publication and Plan60 changes remain outside this milestone.

No conclusion yet establishes that all existing code-intelligence output should become
Protobuf, that gRPC will reduce total complexity, or that any option reduces latency, memory or
agent context. The evidence supports a meaningful comparison now and identifies concrete
opportunities without presupposing its winner.
