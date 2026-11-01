---
title: "Plan60 agent engineering opportunities"
status: proposed
date: 2026-11-01
doc_role: review
doc_topics: [design, libraries, diagnostics, validation]
---

# Plan60 agent engineering opportunities

## Judgment, audience and current capability

**Proposed:** make qualified engineering questions the unit of agent interaction. The strongest
additions combine existing producers into a useful answer, explain the answer's evidence and
limits, and let the agent expand one consequential uncertainty. Fuller use of analyzer, Cargo,
protocol and CLI libraries can support this without adding a universal graph or another service.

This is the functional half of the [combined Plan60 assessment](../plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md).
That companion owns remaining-plan reconciliation, integration amendments I01–I08, public probe
results and the inspected baseline. This document owns new opportunity references; its O01–O08
are local to this review and do not rename Plan60's original O01–O11. Recommendations remain
proposals. Plan revision, production changes and new governing decisions are separately instructed.

Audience: agents conducting audits, solution and functional reviews, library-leverage inquiry,
plan creation/improvement, and implementation investigations. The current CLI already supplies
typed selection/composition, guarded source/native/artifact references, explicit preparation,
original diagnostics, schema/payload navigation and current test selection. Plan61 records
assembled foundation acceptance; Plan62 implements richer source/native/API/Cargo/compiler
evidence while its assembled acceptance remains open at the inspected checkpoint.

Source baseline is `7cc70e2013fe3811188a842b5c300dc0af3a628d` plus relevant concurrent dirty
sources and capability documentation. Core3.4, Efficient Architecture1.0, Test Assessment1.2,
the pse-arrow binding and applicable Process Simulator1.5 limits informed assessment. No code
relationship establishes scientific adequacy, execution, coverage or resource release by itself.
Public probe results are bounded **Tested** observations; opportunities below are **Proposed**.

<a id="O01"></a>
## O01 — Responsibility paths with explanations for unusual origins

An agent could start with two request/policy declarations, see shared construction/validation/
dispatch consumers and follow a bounded path across package boundaries. A result would retain
the structural role and occurrence witness instead of just a list of destination names. This
extends the original Plan60 responsibility target; the additional functional value is explaining
references and implementations that ordinary token/path navigation misses.

HIR `Module::{declarations,impl_defs}`, `Function::{params_without_self,ret_type,async_ret_type}`,
`Type::{fields,tuple_fields,walk}` and `Trait::items_with_supertraits` supply selected structural
relationships. For a format-string reference or derive-generated impl, pinned .358
`Semantics::{as_format_args_parts,check_for_format_args_template_with_file,derive_helper,
impl_generated_from_derive,descend_node_into_attributes,original_range_opt}` supplies further
resolved observations and source nominations. See HIR original §§3,5–7 and addendum §§1–2;
the [exact .358 semantic source](https://docs.rs/ra_ap_hir/0.0.358/src/ra_ap_hir/semantics.rs.html)
supports these leads.

Result interaction: choose declaration → inspect role-qualified neighbors → expand the relevant
format occurrence or implementation origin → inspect original source. Return optional or plural
origins honestly. `impl_generated_from_derive` yields an originating ADT, not a complete inverse
macro history. Formatting parts may include assembly operands rather than global definitions.
Custom macros and unprepared expansions remain unresolved.

Text/AST search is a useful cheap nomination alternative, but cannot resolve these identities.
A complete macro provenance database would add disproportionate retention and mapping work.
Recommend the structural responsibility slice; add specialized provenance for concrete format/
derive investigations through the same relation owner. Cost is consumed native/proto/adapter
extension, not another analyzer. Closure should include escaped braces, an unresolved macro,
multiple expansion origins, namesakes and two parameter positions sharing one type. I01/I02
describe the required traversal and representation correction under DP-04/DP-07.

<a id="O02"></a>
## O02 — Library-fit inquiry through richer shapes and effective APIs

An agent could ask for APIs accepting a callback or bounded associated shape, inspect relevant
default/associated methods and documented conditions, then follow a local integration and request
contextual availability. This advances Plan60 C01/C02 beyond a rich declaration display.

Current `api::query::TypePattern::{parse,from_syn,matches}` supports wildcard, path/type arguments,
references, qualified types, tuples, slices and pointers. It rejects lifetime/const/binding
arguments, callable-trait forms and other shapes. Plan62's richer output contracts do not remove
this input/matcher gap. Extend that same matcher using syn's typed function-pointer/array/
generic-argument/binder structures and schema61 `GenericArgs`, `AssocItemConstraint`,
`WherePredicate`, `GenericBound` and `Term`. Preserve each binder scope; `_` remains an
independent wildcard, not a named unification variable.

Use rustdoc `Item.links`, `Trait::{items,bounds,implementations}` and
`Impl::{items,provided_trait_methods,trait_,for_,generics,is_negative,is_synthetic,blanket_impl}`
for bounded adjacency. Provided defaults can be absent from `Impl.items`; external path summaries
can lack a full definition. Native context owns actual receiver adjustments, imports and
obligations. Documentation exposure and `Deref`-mediated discoverability do not establish a
callable method at a selected use site. HIR effective environments can enrich compatible-world
explanations without joining unrelated rustdoc/native identities by spelling.

Recommend richer patterns and bounded adjacency through existing owners. Keep original
declaration inspection for rare unsupported questions, and native completion for ordinary
call-site discovery. A full trait solver or eager merged API graph has much greater burden.
Closure: callback versus nominal type, array const3 versus4, associated equality versus bound,
different HRTB scopes, a default method, negative/blanket impl, missing import and matching shape
with an unproved obligation. I05 supplies the integration boundary; DP-07/DP-15 preserve meaning.

<a id="O03"></a>
## O03 — Selected package, compiler-unit and semantic-context deltas

An agent planning a feature/target/root change could compare two explicitly selected contexts,
see why package inclusion and unit features differ, then inspect changed native cfg/resolution
where both worlds exist. This is useful even when the local source spelling is unchanged.

Plan62's `cli::dependency::{UnitContext,join_units}`, `correctness::{Resolution,CompilerUnit}`
and `semantic::NativePackageContext` already expose important premises. Compose comparison over
their original retained records. Match package identity separately from unit correspondence
and semantic declaration correspondence. Host/target/test role, target name/kinds/source and
mode affect pairing; a feature difference should appear as a difference rather than make every
unit unmatched. Missing/ambiguous worlds must remain visible.

The new `cargo_metadata` guide §§2–8 and addenda supplies alias/dependency-kind, target, raw-field
fidelity and build-event distinctions. Use actual admitted units for execution-selection claims.
Guppy `PackageGraph`/`PackageSet` and `FeatureGraph` provide useful native set/conditional-edge
operations for repeated questions; `CargoSet::to_summary` and `Summary::diff` provide a modeled
host/target comparison alternative. Adopt them only for an explicitly modeled counterfactual
consumer that existing retained-observation comparison cannot serve. Workspace feature
unification compatibility remains to be qualified. Determinator can nominate impact but cannot
establish adequate validation or guarantee complete affected selection.

Recommend observed-context delta first; keep modeled counterfactuals independently selectable.
Compare against today's separate inspection calls: a single aligned view can remove manual
joins while preserving original evidence. Cost is correspondence and comparison projection;
simulation adds a separate semantic commitment. Challenge with aliases, build/test/host units,
cfg-only resolution change, absent before-world and metadata packages absent from selected units.
I03 develops AP-04/DP-04/DP-09 consequences.

<a id="O04"></a>
## O04 — Evidence paths from a change or failure to contracts and revealing checks

An agent could follow an original failure into the exact production/test/setup definition,
inspect the assertion and its expected-value derivation, then nominate current checks. A change
investigation could follow a declared schema origin or explicit contract citation into the
canonical section and original qualification record. Each hop would explain its basis.

Use existing inventory `TestRow`/`FailureRow`, compiler `DiagnosticEvidence`, fixture/selection/
report owners and Plan62 native runnables. Source syntax can expose selected assertion arguments;
syn known-macro-body parsing or RA CST can interpret a supported form. A custom assertion macro
stays opaque until its own grammar/definition is inspected. Native relationships earn their cost
for aliases, shared helper fan-out and ambiguity; simple source inspection is often enough.

Document associations consume `xtask::documentation::ownership` and native lifecycle/reference
interpreters. Source `ownership::PlanOwnership` currently supplies `NotDeclared`; package
membership cannot invent a plan binding. RA syntax `HasAttrs::{doc_comments,attrs_with_doc}`
can nominate source citations; HIR `resolve_doc_path_on` can resolve applicable intra-doc symbols.
Those have different meaning from architecture citations and declared document ownership.

Recommend query-shaped evidence paths, retaining original addresses/status and source freshness.
Avoid a new manual source-to-plan registry or automatic assurance score. Test setup failure before
an assertion, a shared faulty comparator, a self-derived expected value, a misleading rejected-
alternative mention, moved section and historical citation. Correct links do not establish
implementation, current qualification or oracle independence. I07/I08 and Test Assessment1.2
explain why this belongs beside E02 rather than another runner or automatic rerun controller.

<a id="O05"></a>
## O05 — Progressive discovery and reliable reference handoff for agents

An agent could discover a small relevant command description, request its arguments/schema,
explain the effective query, receive compact evidence, and execute an exact returned next action.
Unsupported input would identify the relevant argument/field and correction without requiring
the agent to parse a terminal report or read a large catalog.

Clap `CommandFactory`, public metadata getters, `TypedValueParser` and `ValueSource` provide the
syntax/discovery foundation. `catalog` already attaches lazy argument candidate providers.
Use those for concise and full descriptions, rather than duplicate command declarations.
Public getters do not expose all conditional validation; unavailable metadata must stay honest.
Figment `merge`/`join`, `find_metadata`, optional `Tagged<T>` and source-relative path support
can explain composed input. The current `investigation::config` already derives effective
field provenance from its actual Figment; do not rebuild it as a separate configuration engine.

Use existing `ResultReference::encode` and ready next actions to publish an opaque address
consistently. The review's Python round trip succeeded; JavaScript's ordinary number parsing
rounded a nanosecond guard and caused refusal. This is a concrete interoperability problem
with a simple existing native mechanism. Keep structured reference detail for exact inspection.

For diagnostics, preserve Thiserror typed context and Miette diagnostic-aware causes, related
findings, source bytes and help in the existing envelope. `into_diagnostic` can erase rich
metadata, and ordinary transparent errors do not automatically forward diagnostic labels.
For progress, use Indicatif draw targets/trackers and optional `MultiProgress` only as a human
presentation of observed work. `InMemoryTerm` supports terminal rendering tests if that behavior
changes; non-TTY streams still need original outcomes/finality independent of animation.

Recommend this alongside A03/E01, using demonstrated discovery, invalid-field and reference
journeys. A new projection DSL, interactive shell, automatic provider preparation or alternate
diagnostic protocol is unnecessary. Distinguish preparation unavailable, request invalid,
cancel requested, work settled and delivery incomplete. I06 contains actual probes and costs;
AP-05/AP-06 and DP-21/DP-24 support the proposal.

<a id="O06"></a>
## O06 — Schema-addressed validation corrections using the selected validator

An agent correcting a nested request or payload could inspect the violated field, rule and
schema identity directly. Current `schema::payload::static_preview` reduces validation failure
to string diagnostics; RPC retains canonical details opaquely in `original_rpc_details`.
Useful structure is present upstream but not fully projected into the correction workflow.

Pinned `protovalidate-buffa0.7.2` supplies `Violation`, `FieldPath`, `FieldPathElement` and
`decode_violations`, including numbered/indexed paths and rule identifiers. Project supported
details through the existing diagnostic and schema-inspection owners, retaining original bytes,
sanitization and validator defects separately. Public RPC details intentionally remove some map
selectors/messages and cap whole violations at4KiB; omitted/redacted locations must not be
reconstructed or presented as exhaustive. See validation overview §§3.5,6.3,7.2 and exact installed
`src/error.rs`/`src/connect.rs`.

Recommend this focused addition when invalid-query/payload examples use the same correction
surface. It adds useful interpretation without another CEL engine or dynamic validator.
Compare with original error inspection, which remains sufficient for occasional unsupported
details. Challenge it with repeated indices, redacted map entries, truncated details, a required
constraint violation and a validator compile/evaluation defect. Compiled schema validation still
does not establish a native fact's truth. This extends I06 under DP-14/DP-21.

<a id="O07"></a>
## O07 — Conditional external source-and-descriptor bundle acquisition

A developer or agent investigating an external contract may have original `.proto` sources
whose imports are supplied as compiled descriptors rather than an on-disk source tree. The
existing immutable bundle/payload/comparison tools are useful consumers; accepting this input
shape could avoid manually reconstructing all imports as source files.

The unused `protox` route supplies `Compiler`, `FileResolver`, `DescriptorSetFileResolver::decode`
and `ChainFileResolver`. Combine explicitly acquired inputs, retain logical import identities,
compiler identity and source provenance, and publish through the existing immutable `schema::Bundle`.
Use complete import/source information and `encode_file_descriptor_set()` where custom options
matter; a plain `prost_types::FileDescriptorSet` conversion may lose extension options. See
compilation overview §§2.1–2.2,5.3,9. The overview's protox0.10.0 and relevant Editions support
need exact-release qualification before adoption.

Keep this conditional on selecting that external-input capability. Existing Buf/protoc full
descriptor acquisition remains the simpler choice for current repository contracts. Cost is an
additional compiler interpretation and representation boundary, not a replacement production
generator. Test conflicting logical imports, missing imports, custom `buf.validate` options and
unsupported language editions before recommending implementation. No remote registry/service
or second decoder is needed. The option is useful independently of I01/I02 and must not block them.

<a id="O08"></a>
## O08 — Conditional implicit dependency and phase explanations

Beyond current capture/cleanup facts, an agent could ask why a generated shim, implicit drop or
vtable dependency appears, or what changed between qualified compiler phases. Reuse acquired
artifacts first. The compiler addenda identify preoptimization `MentionedItems`, generated
shims, coroutine storage conflicts and drop-validity distinctions as deeper leads.

Current compiler analysis already stages early/drop-elaborated/optimized observations across
selected bodies to respect query consumption. Adding a new question must preserve that order,
exact compiler/body/phase and source correspondence. `mir_promoted`,
`mir_drops_elaborated_and_const_checked`, `optimized_mir`, `mir_coroutine_witnesses` and native
dataflow are existing foundations, not an invitation to register a general pass framework.

Defer new extraction until an implicit-dependency question remains unanswered by current
diagnostics/artifacts. Exact pinned accessibility and interpretation must be established then.
Compare with source-level implicit-operation explanation, which may already suffice. Test a
generated body without unique authored origin, normal versus unwind cleanup, C versus C-unwind,
and incompatible phase/compiler observations. Preserve the separate loan/Polonius trigger:
storage, initialization, loans and observed release are different facts. I04 explains the
recommended initial lifetime consumer; this optional depth is not its prerequisite.

## Combined selection and alternatives

Recommend O01's structural/multi-seed slice, O02, observed O03, O04 and O05 as coherent candidates
for the later Plan60 update. O06 is a focused extension using the selected stack. Specialized
origins in O01 can start with a useful format/derive case. O07/O08 and modeled counterfactuals
within O03 retain separate consumer/feasibility conditions. This is recommendation priority;
working dependencies and integration order are described by I01–I08, not an automatic adoption.

| Additional library route | Comparative disposition and revisit condition |
| --- | --- |
| Buffa eager/owning/lazy views, custom storage and segmented output | Keep owned small facts/direct VTable reflection. Owning views need an actual retained binary-input consumer; lazy views lack the pinned reflected/OwnedView surface. Segmentation helps only if downstream transport retains segments. Plan62 already records these boundaries. |
| Dynamic validation via `prost-protovalidate::Validator` | Potentially useful for an actual external runtime-schema validation consumer. Current external schemas expose metadata; do not silently promise evaluation or introduce another validator to solve O06. |
| Prost/Tonic, prost-reflect/PBJSON, Rust-first `proto_rs` | No selected advantage for the responsibility/composition gap. Reconsider for a concrete consumer requiring their particular capabilities and compare the full authority/reflection/validation/cancellation migration. A second stack would not implement O01 itself. |
| `protoc-gen-doc` | Useful for a selected browsable generated contract-reference consumer unmet by present inspection. Comments/descriptors can supply generated documentation; native document status remains authoritative. |
| PUBG JSON Schema / PBJSON | Require a distinct external JSON-contract consumer. Check int64, presence, WKT and Editions behavior; do not replace deliberate Serde/Schemars CLI presentation by default. |
| `grpcurl`, `buf curl`, `granc`/`granc-core`, protoscope, health/reflection/conformance tooling | Useful targeted diagnostic/interoperability tools if a concrete investigation needs them. Offline descriptors already provide discovery without a running service; no health/reflection deployment is selected merely for feature count. |
| StaticIndex/SCIP/LSIF, typed Tree-sitter wrappers, retained graph | Preserve Plan62's repeated/global/export and runtime-compatibility triggers. Scoped multi-seed intent does not require persistent indexing; standalone syntax .360 and native .358 remain distinct type worlds. |

## Sources and library boundaries

The preparation covered all thirteen analyzer documents in full, including unique addenda;
two bounded research contexts then inspected decisive current source independently. The protocol
inquiry deeply inspected selected current-stack sections and screened alternatives, honoring
the operator's narrower reading instruction. No complete read of every proto document is claimed.
All four named CLI guides were read; current root source/probes grounded the agent perspective.

| Source family | Concrete research route and consequence |
| --- | --- |
| [HIR](../libraries/rust_analyzers/ra_ap_hir.md), [IDE](../libraries/rust_analyzers/ra_ap_ide.md), [rust-analyzer](../libraries/rust_analyzers/rust-analyzer.md) | HIR §§3,5–7/addenda1–4; IDE deferred resolution, SSR, runnables and index sections; context/scope guidance. O01/O02/O04/O08. Exact .358 installed sources govern native APIs. |
| [RA syntax](../libraries/rust_analyzers/ra_ap_syntax.md), [ast-grep](../libraries/rust_analyzers/ast-grep-core.md), [Tree-sitter](../libraries/rust_analyzers/tree-sitter-rust.md), [syn/quote/prettyplease](../libraries/rust_analyzers/syn_quote_prettyplease.md) | Typed docs/attributes, specialized parsing, capture/replacement extent, query demand/cancellation and source fidelity. Reuse Plan62 interpretation/preview owners. ast-grep insertion helpers remain unimplemented; structural captures do not establish binding identity. |
| [Cargo metadata](../libraries/rust_analyzers/cargo_metadata.md), [dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md) | Metadata/event fidelity; dependency addenda A–E/G for set algebra, summaries and impact. O03. Package graphs, modeled feature activation, actual selected units and build events retain different authority. |
| [rustdoc](../libraries/rust_analyzers/rustdoc.md), [rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) | rustdoc §§4–6,8/addenda2–5; schema61 §§5–10/addendaB/E/F/G. O02/O04. Documentation omissions, stripped fields, external summaries and native transformed examples need explicit limits. |
| [compiler](../libraries/rust_analyzers/rust-compiler-internals.md), [continued compiler](../libraries/rust_analyzers/rust-compiler-internals-continued.md) | MIR/dataflow/initialization/drop/shim/coroutine addenda. O08/I04. Shared sections are one source family; fixed-point analyses are composed, not one universal solve. Exact c1070d693 sources govern. |
| [Buffa](../libraries/proto/Buffa.md), [ConnectRPC](../libraries/proto/ConnectRPC_for_Rust.md), [validation](../libraries/proto/protobuf_validation_rust.md) | Buffa §§3,7; ConnectRPC request ownership/lifecycle/limits; validation §§3.5,6.3,7.2. I01/O06. Direct reflection, native work settlement and sanitization are separate obligations. |
| [compilation](../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md), [generators](../libraries/proto/protobuf_generator_ecosystem_rust.md), [local RPC](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) | Mixed source/descriptor resolution, generated references and diagnostic tools. O07 and conditional alternatives above. |
| [Clap](../libraries/cli/clap.md), [Figment](../libraries/cli/figment.md), [Indicatif](../libraries/cli/indicateif.md), [Thiserror/Miette](../libraries/cli/thiserror_and_miette.md) | Clap §§3,6–7,9–11; Figment §§2–8,10–11; Indicatif §§2–7; diagnostic protocol/wrapping/rendering sections. O05/I06. Library rendering does not define a new wire protocol or work outcome. |

Context7 resolved and queried the relevant upstream projects, including rust-analyzer, Cargo/
rustdoc, Buffa, protovalidate and the CLI libraries. Exact installed sources resolved material
release questions. In particular, Buffa main claims strict generated text parsing while pinned
0.9.2 codegen emits `dec.skip_value()` for unknown fields; keep qualified protoc text parsing.
ConnectRPC's lookup returned unrelated projects, so versioned installed sources governed instead.
Syn documentation describes3.0.9 while the checkout pins3.0.5; Tree-sitter .27.1 documentation
does not establish every .27.0 API. Tonic overview .14.8/.14.9 does not establish pinned .14.6
features. Generator/runtime versions are separate. No dependency upgrade is selected here.

Exact reference entry points include [HIR .358](https://docs.rs/ra_ap_hir/0.0.358/ra_ap_hir/),
[rustdoc-types schema61](https://github.com/rust-lang/rustdoc-types/blob/v0.61.0/src/lib.rs),
[Cargo unit graphs](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph),
and [pinned compiler orchestration](https://github.com/rust-lang/rust/blob/c1070d693/compiler/rustc_mir_transform/src/lib.rs).
These support named mechanism leads; they do not constitute executed acceptance of this review's
proposals. Public command failures, successful bounded source probes and unrun native journeys
are preserved in the companion's evidence section. No speed, token, memory or assurance gain was
measured. Subsequent planning should select opportunities and carry their mechanisms and limits
forward without turning these illustrative journeys into mandatory process steps for every agent.
