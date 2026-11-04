---
title: "Plan65 native scientific replay membership"
date: 2026-11-04
status: complete
standard: core-3.4
heuristics: "1.0"
profiles: [process-simulator-1.5]
tier: design
purpose: target
scope: Plan65 E02 recipe qualification and concrete record-membership consumers
verdict: not-accept
doc_role: review
doc_owner: docs/plans/65e-scientific-portability-and-execution.md
doc_topics: [design, scientific-portability, proto, validation]
---

# Native scientific replay membership

**Prefer a DOM-free Serde traversal preserving membership V1, but do not accept the
current cutover proposal yet.** It can remove the JSON value tree without changing
the concrete consumers' structural membership meaning. The remaining decisions are
how the compiler binds its exact decoded root to authenticated protobuf bytes, how
the traversal handles normalization and overwritten subtrees, and how admission bounds
the complete decode/index/reconstruction working set. These are design premises, not
merely requests for tests of an otherwise complete design. Whole-payload permission
without nested record membership is unsuitable.

This is an independent, bounded assessment within the authorized Plan65 execution,
with the combined library-targeted and design-principle lens. It is not a refreshed
broad diagnostic audit or a review of the whole simulator. Criteria are core 3.4,
efficient-architecture heuristics 1.0, process-simulator 1.5 and the repository binding.
The source baseline is `main` at `f55bb133c369af8637237dcd06350d9fe9ff5cd0`.
Concurrent C/native-record and generator/publication edits were observed and excluded
from this judgment. No authority/hash implementation for this decision was inspected
as completed. The reviewer did not author the candidate implementations or their plan.

All source/API conclusions below are **Interface-checked**. The replacement and its
benefits are **Proposed**; no runtime test, scientific qualification, measured speedup
or complete memory/RSS bound is established by this review. [Plan65e](../../plans/65e-scientific-portability-and-execution.md)
owns E02 disposition; the [coordinator](../../plans/65-proto-integrated-target.md)
owns integration and decision routing. The review is evidence, not acceptance of an ADR.

## Existing contract and responsibility boundary

The enduring contract is blueprint §14.3: complete compiler-issued payload,
qualification and exact request bind opaque replay authority; missing or incompatible
records and receipts refuse without fresh inference or proof. Native decoding exposes
data; current scientific permission comes from the controlled qualification route and
the original scientific constructors. This separation is useful and must survive.

`pse-ids::scientific_replay` currently authenticates exact JSON payload bytes using
`admitted_recipe_payload_hash`, then parses a `serde_json::Value`. Its scope hash binds
source kind, exact digest, qualification and request using `MathReplayAuthorityV1`.
Postorder `structural_hash` indexes every surviving JSON node. Objects have lexical
key order and final unique-key count; arrays have actual length and ordered child
hashes; numbers use `Number::to_string`. `RecordAuthority::admits<T: Serialize>` first
uses `serde_json::to_value`, then the same structural framing.

Membership V1 is consequently **normalized structural content membership**, not a
typed-owner, field-location, multiplicity or original number-token proof. Equal
structures can share a digest; repeated identical nodes share set membership. A new
owner/tag framing or raw floating-point-bit framing intentionally changes this
meaning even if the artifact byte hash remains separately exact. Neither strengthening
nor weakening it should occur accidentally during an encoding change.

| Responsibility | Current owner and consumed contract | Preservation requirement |
| --- | --- | --- |
| Compiler-issued recipe and reconstruction | `pse-compiler/src/typed_math_portable.rs::{Recipe,capture,reconstruct_portable}` | One compiler-native root governs interpretation, bounds, complete records and final body/spec/source-role comparison. |
| Persisted current qualification | `pse-runtime/src/math/portable.rs::reuse_body_current` | Canonical dependency/eligible-producer/interpretation selection and exact identity/physical request precede minting. Keep its leases through reconstruction. |
| Private receiving qualification | `pse-runtime/src/math/native_child.rs::reconstruct_and_solve` and its authenticated private channel | Preserve original image/build/attempt/execution/request premises, provider-free closure, registry/prerequisite reconstruction and full physical-identity comparison before authority. |
| Opaque authority and structural membership | `pse-ids/src/scientific_replay.rs` | No safe decoder, arbitrary caller root/hash/index, descriptor or DTO may mint current authority. Preserve source-kind scope, qualification/request binding and non-deserializable permit. |
| Scientific restoration | Quantity, math, modeling, kernel and compiler owners below | Complete owning-record membership remains necessary alongside owner-specific checks; data adoption alone grants none of these permissions. |
| Pure projection vocabulary | `pse-contract::boundary_projection::{ConstructionRole,ProjectionRecord}` and owner declarations | `Data` supports structural construction; owner/version/tags describe projection. Neither role nor successful local validation is a scientific grant. |

The compiler currently decodes the typed recipe after the permit indexes its JSON.
It nests physical receipt replay around syntax/support/provider restoration and
mathematical receipt replay around `Request::admit_inner`. Both streams require
membership and supported versions. Physical replay matches ordered operation and
boundary requests and drains both populations; mathematical replay matches ordered
proof requests and drains its population. Original inference/provers refuse accidental
entry during strict replay. Reconstructed support retains no live preparation context;
the completed body retains immutable exact bytes, not the permit or session.

### Concrete consumer closure

The bounded search covered `require_record(`, `RecordAuthority`, and authority
`admits` calls across `crates/`, then read the relevant restoration bodies. This is
the concrete closure that a cutover must retain; it is not an assertion that arbitrary
future `Serialize` implementations have identical behavior.

| Owning source | Complete checked records / direct membership consumers |
| --- | --- |
| `pse-quantity/src/resolved_receipts.rs` | `ContractRecord`, `ContractBoundaryRecord`, `OutputBoundaryRecord`, `InferenceRecord`, `SchemeRecord`; complete `ReceiptStream` in `replay`. |
| `pse-quantity/src/unit.rs` | `CanonicalConversionRecord`, `RepresentationConversionRecord`, `MagnitudeRecord`. Magnitude restoration independently refuses nonfinite values. |
| `pse-kernels/src/portable.rs` | `ProviderRecord`, followed by interpretation and native provider-key equality. |
| `pse-math/src/typed_provider.rs`, `typed_receipts.rs` | `PreparedProviderCallRecord`, including restored registration/contracts/conversions; complete `ProofStream` in `replay`. |
| `pse-compiler/src/typed_math/provider_records.rs` | `SelectedProvidersRecord`, including exact selected-call joins and derivative/formal/output checks. |
| `pse-modeling/src/resolved/{records,support_record,support}.rs` | `ResolvedRecord`, its selected `FunctionRecord`, `SupportRecord`, `PhysicalEvidenceRecord`; syntax identities, pool/coordinate checks and absence of live evidence remain owned there. |
| `pse-modeling/src/portable.rs`, `expression/admission.rs` | Public `FunctionRecord`, `ValueRecord`, `TypeRecord`, `PhysicalAdmissionRecord`; nested physical and nominal meaning remains owner checked. |

Some paths are broader than the current provider-free native child. Canonical portable
replay and admitted support/function restoration still consume them. Qualifying only
the child, provider descriptor, or two receipt streams would leave real consumers
uncovered. Preserve all concrete `require_record<T: Serialize>` consumers without
making IDs, quantity or math depend on Buffa, generated wire messages or runtime services.

### Physical meaning and numerical scope

| Model element | Dimension/unit and basis | Reference/convention and validity | Authority |
| --- | --- | --- | --- |
| Physical registry, prerequisites and contracts | Complete native quantity/unit declarations and contract shapes; no new basis | Existing registry admission, prerequisite meaning and physical identity; original conversion bits and finite-magnitude checks | Quantity/compiler owners; receiving reconstruction compares complete original physical identity. |
| Providers and prepared calls | Exact port contracts, shapes, conversions and selected roles | Original validity/derivative capability and provider semantic-key checks; no descriptor-created factory | Kernel/math/compiler restoration; executable providers remain separately admitted. |
| Selected graph, support and proof outcomes | Original types, occurrences, guards and support | Original receipt requests, source-role inventories and strict no-inference/no-proof replay | Modeling/compiler owners plus physical and mathematical replay. |

No variable role, degree-of-freedom policy or numerical formulation changes are
proposed. Structural checks and `compile_problem` remain receiving/solver preparation
responsibilities; membership cannot substitute for them. This assessment follows the
private route as far as its receiving compile/solve call and the preserved parent
original-coordinate screening contract, without qualifying structural rank, derivatives,
solver convergence or balance closure. No unit/property model is changed; PS-13
conformance suites were not run and no new conformance result is claimed.

## Complete alternatives and library fit

| Alternative | Complete route and benefit | Burden, protection and judgment |
| --- | --- | --- |
| Preserve current exact JSON artifact | Existing capture → exact qualified bytes → JSON DOM index → typed decode → strict reconstruction. Simplest operational baseline while this decision is open. | Preserves known authority behavior, but retains JSON artifact/parsing and simultaneous representations. Does not deliver the full E02 protobuf target. Keep until its replacement is complete. |
| Decode protobuf to exact native `Recipe`, then use existing JSON DOM index | Compiler owns bounded decode/adoption and its binding to authenticated bytes; `to_value(native_root)` supplies membership; concrete query consumers remain unchanged. | Viable staged realization and useful comparison oracle. Retains native root plus value tree/index, and changes where root normalization occurs. It still needs exact root binding, source-normalization compatibility and protobuf-appropriate admission. It is not a free reuse of the existing constructor or a necessary intermediate production path. |
| DOM-free Serde traversal preserving membership V1 | Existing owner `Serialize` implementations drive a `Serializer` whose `Ok` carries structural summaries/digests. Composite traits finalize actual children; native root builds the index and concrete queries use the same normalization. | Preferred candidate: removes full JSON values/text round trips for indexing and avoids a second scientific field visitor. Retains necessary hashing/index/sorting work and bounded temporary child summaries. Exact normalization, duplicate handling and complete admission remain conditions below. No latency or RSS benefit was measured. |
| Versioned source-projection typed owner/tag traversal | Extend native projection declarations to derive an explicitly new membership interpretation; root and every concrete consumer use matching owner/version/tag/scalar/collection framing. | Technically credible if typed membership is independently wanted. Distinguishes owners and scalar bits but creates a new authority meaning and a full consumer migration. `ProjectionRecord::Fields`/`FIELD_LAYOUT` expose structural data, not an existing replay traversal/permit. No demonstrated E02 requirement needs that extra change now. Revisit for an explicit owner-sensitive permission requirement; do not select merely because Proto tags exist. |
| Whole-payload-only permit | Authenticate artifact and restore any locally valid decoded records under it. | Reject: substituted locally valid nested records can bypass their current membership checks even while receipt streams and outer spec appear valid. Local validity and exact artifact authentication serve different obligations. |

Serde 1.0.229 explicitly supports a serializer-defined `Ok` type and matching
`SerializeSeq`, `SerializeTuple`, `SerializeTupleStruct`, `SerializeTupleVariant`,
`SerializeMap`, `SerializeStruct` and `SerializeStructVariant` results. This is the
library-owned traversal capability, not a need to reimplement Rust record discovery.
The project still owns membership framing, authority and bounded summaries because
Serde does not supply that contract. Preserve `is_human_readable() == true`, which
matches the current value serializer and can affect IDs and custom owners. These
interfaces were checked in [the exact-release trait](https://docs.rs/serde/1.0.229/serde/ser/trait.Serializer.html)
and [Serde's serializer guide](https://serde.rs/impl-serializer.html).

Context7 resolution/query came first. Its versioned Serde query returned shallow
examples; its serde_json query described a newer source, so neither settled exact
release details. The installed Cargo-registry Serde/serde_core 1.0.229 and
serde_json 1.0.151 sources were then read, including `ser/mod.rs`, `value/ser.rs`,
`number.rs` and the map path. The public exact-release references are
[value serialization](https://github.com/serde-rs/json/blob/v1.0.151/src/value/ser.rs)
and [number normalization](https://github.com/serde-rs/json/blob/v1.0.151/src/number.rs).
The following conditions derive from those sources, rather than a generic claim that
all Serde serializers produce equivalent values.

## Findings and necessary decisions

<a id="f01"></a>
### F01 — Authenticated bytes must determine the indexed native root

**Interface-checked gap; AP-01/AP-04/AP-05, G1/G3/G6; S01/S02.** Existing IDs minting
parses the very bytes it authenticates, while the compiler independently decodes
those bytes for reconstruction. Replacing it with a generic `mint(payload_hash, root,
index)` would let the two meanings diverge. A correct digest over payload A says
nothing about an unrelated caller-supplied root B, even if B passes local validators.

**Proposed correction:** the compiler-owned receiving path must check the selected
interpretation, decode and structurally adopt the exact authenticated bounded bytes,
then build membership from that same immutable native root and use it for strict
reconstruction. The controlled canonical/private qualification assertions retain
their original complete premises and source-kind/qualification/request scope. Keep
authority construction sealed behind those audited routes; a public data constructor
or generic index builder must not confer permission. No round trip of arbitrary
caller data, claimed digest or validated descriptor is a substitute. IDs supplies the
wire-independent authority mechanism; the compiler owns the recipe/codec association.
If roots are decoded more than once, the binding must still establish the identical
selected interpretation, not rely on caller assurances or coincidentally equal headers.

**Verification:** a legitimate authenticated payload must not authorize a changed
decoded record, root, request, qualification or interpretation; ordinary raw decode
must not make a usable permit. Exercise both canonical selection and the actual
private child. These are distinct routes, not one fixture relabeled twice.

<a id="f02"></a>
### F02 — Native traversal needs an explicit normalization and survivor contract

**Interface-checked gap; AP-02/AP-04/AP-05, DP-07/DP-08, G2/G6; S01–S03.** The plan's
typed owner/tag preference changes structural membership, whereas a direct serializer
can preserve it only by reproducing consequential value-serializer behavior. Hashing
Serde calls or raw native bits is not the current contract.

For the preferred alternative, settle these cases before implementation:

- `None`, unit and unit structs normalize to null; `Some` and newtype structs unwrap.
  Unit variants are strings; external newtype/tuple/struct variants add their object
  wrapper, with nested array/object nodes included in membership. Bytes become arrays.
- Count actual emitted sequence elements and surviving object entries, not length
  hints. Unknown sequence lengths and custom serializers with inaccurate hints must
  not change the resulting digest. Since V1 frames length before children, retaining
  ordered child digests until finalization is legitimate bounded work.
- Render map keys as the pinned value serializer does, then order the resulting strings
  lexically. Native key ordering alone is insufficient. Preserve supported key forms
  and rejection of unsupported/nonfinite keys.
- Use pinned JSON number normalization, including signed/unsigned integer ranges,
  floating representation and nonfinite-value-to-null behavior. Number display is
  the membership input, not `Number` equality or raw float bits. `f32` value conversion
  and parsed JSON text must not be assumed identical without checking the actual
  source closure. Scientific records already storing numerical bits as integers retain
  those bits through their own representation; this is no license to normalize them.
- Settle feature-sensitive private number/raw-value sentinel handling and custom
  `collect_str`. The workspace declares `float_roundtrip`; `schemars` also declares
  `raw_value`, which can enable serde_json's corresponding feature through unification.
  This review did not acquire a complete compiler-unit feature report. Either preserve
  applicable sentinel semantics or establish and enforce their absence from the
  accepted source/query profile; do not silently reinterpret them as ordinary structs.

Most critically, `serde_json::Value` maps use **last rendered key wins**. In
`{"k":{"secret":1},"k":0}`, only the final `k` and its ancestors/siblings survive.
An eager global digest insertion while reading the first value would grant membership
to `{"secret":1}` even though the old index never contains it. Deferring the parent
hash alone does not retract those descendant grants.

**Proposed correction:** choose either exact last-key-wins survivor semantics with
bounded deferred summaries/reachability, or a justified accepted-source constraint
that every rendered object key is unique. The latter must be explicit in the new
interpretation and established for every concrete native root and query consumer;
Rust map-key uniqueness alone does not prove rendered-key injectivity. Reject ambiguous
input before granting any authority. Do not retain per-node copies of all descendant
sets: nested records would amplify state unnecessarily. This review prefers preserving
the existing semantics; it does not accept a silent uniqueness assumption.

The typed alternative can be chosen deliberately, but then every concrete row in the
consumer table needs a matching typed-membership contract, with intentional distinctions
and version/refusal behavior. Source projection declarations do not establish that
migration automatically. The existing current JSON route has not been shown defective
by this candidate-design finding.

**Verification:** compare the new digest and complete authorized node set against the
pinned DOM implementation for admitted concrete roots and normalization cases. Include
overwritten nested subtrees, duplicate rendered keys, order/length changes, wrappers,
numeric boundaries, nonfinite values and the selected feature profile. Verify both
positive nested membership and changed-record refusal; top-level hash equality alone
would miss an overbroad index.

<a id="f03"></a>
### F03 — The JSON scratch formula does not establish protobuf admission

**Interface-checked gap; AP-07, DP-20, G3/G5/G9; S04.**
`QualifiedScientificRecipe::scratch_bytes` currently uses `payload_bytes * 256 + 32 KiB`
for simultaneous JSON, index and DTO scratch. Canonical replay reserves it before
parsing; native submission includes it in its known-allocation lease through child
settlement and parent screening. Protobuf default elision, compact repeated values,
generated field expansion and native allocation capacities change the relation between
bytes and that working set. Reusing the constant is neither a demonstrated bound nor
a demonstrated failure; its premise is unresolved for the replacement.

**Proposed correction:** use actual bounded codec/decode and native adoption allowances
for frame bytes, repeated elements/allocated memory, recursion and unknown meaning,
then cover native records, survivor/child summaries, membership set, receipt clones
and reconstruction temporaries that coexist. Reserve before the allocation being
admitted; checked overflow/exhaustion must refuse without partial authority. Retain
the accounting lease to the last consumer and original process drain. A bounded
protobuf frame or recursion limit alone does not cover the complete operation.
The compiler/receiving owners should own this concrete admission; no generic cost
model, proof framework, new scheduler or full RSS claim is needed.

Full-input hashing is necessary to index the complete qualified root. Lexical map
ordering, child summaries and the digest set remain necessary state unless a narrower
contract is deliberately selected. The proposed serializer removes the full value
tree; it is not allocation-free. Avoid rebuilding the complete root index per
`require_record` query, repeatedly encoding its text, or expanding descendant summaries
quadratically. Existing repeated nested query hashing remains part of the complete
route and is not eliminated merely by changing its serializer. Ordinary admitted recipe
shapes, including small encodings with many defaulted native fields, need a credible
route under the chosen allowances; an always-refusing safe limit is insufficient.

**Verification:** use bounded ordinary and expansion-heavy roots, nested/map-skewed
records, exhaustion and checked-overflow controls; inspect simultaneous ownership
through success/refusal/cancellation. This establishes named admission behavior, not
native-library private allocations or complete process RSS. Measurements would be
needed for quantitative capacity or performance claims.

F01 and F02 establish what may be authorized; F03 determines whether the complete
operation can prepare that authority within its admitted resources. None can replace
another. All belong to E02 and its decision, without moving physical or mathematical
policy into transport.

## Change scenarios and execution fit

| Scenario | Expected ownership and revealing case | Evidence/acceptance boundary |
| --- | --- | --- |
| S01 — Mechanism substitution: full recipe protobuf | Compiler decodes the exact qualified bytes and indexes the resulting root; IDs and concrete restore owners retain pure structural contracts. A valid but different root must refuse. | Proposed route; F01/F02 must be settled. No requirement for a retained compatibility reader is inferred. Explicit old-interpretation refusal is valid. |
| S02 — Domain extension: add a physically admitted provider capability | New domain data and constructors extend their owning records; Serde traversal follows the concrete representation, generated wire adapters remain compiler/transport integration. A locally valid but absent provider/conversion record still refuses. | Source-traced current consumers; no unrelated transport policy switch or decoder-created factory. |
| S03 — Local policy/membership testing | Exercise structural membership and receipt absence/order/drain without starting a solver, store or provider factory. Separately exercise receiving qualification at the real boundary. | Proposed equivalence/refusal tests. No source-derived candidate oracle for independent scientific expectations. |
| S04 — Grow or interrupt replay | More nested/defaulted records or concurrent selected recipes grow native allocations, summaries, index and receipt state together; failure releases original leases and does not publish partial authority or skip drain. | Qualitative AP-07 assessment; bounded admission and original lifecycle required by F03, no measured scale claim. |

The only numerical stage directly adjacent to this decision is the existing private
receiving compile/solve route. Its formulation remains the reconstructed admitted graph
and original physical/source bindings; derivative source/order, scaling, KINSOL problem
class and tolerances remain existing contracts, not serializer policy. Solver status
and parent original-coordinate screening continue to govern numerical outcomes. No
new numerical algorithm, solve acceptance, workflow, dynamic support or continuation
capability is supplied by a record index. Study/edit reuse can retain immutable
qualified mathematics; each new execution still owns its outcome checks.

## Architectural and behavioral judgments

Verdicts assess the proposed cutover as currently specified, not the completed JSON
implementation. Unimplemented tests alone are not a design failure; missing meanings
and allocation premises are unresolved design decisions.

| Foundation | Verdict and basis |
| --- | --- |
| AP-01 | **Satisfied** for the proposed direction: compiler owns recipe/codec binding, IDs owns wire-independent membership, scientific owners retain restoration policy. F01 must preserve that assignment. |
| AP-02 | **Unresolved**: DOM-free traversal can preserve the existing concrete consumed contract, but normalization/survivor semantics and any accepted-source restriction are unsettled (F02). |
| AP-03 | **Satisfied**: existing Serde traversal and strict physical/math sessions compose without a second scientific workflow or wire dependency in pure owners. |
| AP-04 | **Unresolved**: the model distinguishes data, qualified root, record membership and receipt consumption adequately, but the new root-to-authenticated-payload operation is not specified tightly enough (F01/F02). |
| AP-05 | **Unresolved**: exact mint inputs, interpretation, duplicate handling and allocation/refusal boundaries need enforceable contracts (F01–F03). |
| AP-06 | **Satisfied** at proposed architecture level: pure membership and receipt controls can run locally, with receiving qualification separately integrated. |
| AP-07 | **Unresolved**: removing the full value tree has a clear structural benefit, but protobuf/native/index simultaneous admission remains unspecified; child summaries and query work still count (F03). |

| Gate | Judgment within this boundary |
| --- | --- |
| G1 Authority | **Unresolved**, F01; original qualification/request scope and controlled mint must bind the exact decoded root. |
| G2 Semantic fidelity | **Unresolved**, F02; preserve normalization/survivors or declare a deliberate versioned change with the complete consumer closure. |
| G3 Validity | **Unresolved**, F01–F03; data validation cannot replace qualification, exact membership or preallocation refusal. |
| G4 Hidden behavior | **Pass at Proposed strength**: pure traversal/indexing introduces no provider, inference, prover, filesystem or solver effect. |
| G5 Consistency/recovery | **Unresolved**, F03 and survivor publication in F02; authority must be complete or absent, retaining original settlement ownership. |
| G6 Transformation/reuse | **Unresolved**, F01/F02; normalized native-root correspondence and strict complete receipt replay must survive replacement. |
| G7 Truthful claims | **Pass at Proposed strength**: bounded decision, no decoded grant, no scientific/performance/RSS qualification claimed. |
| G8 Library leverage | **Pass at Interface-checked/Proposed strength**: Serde provides the native traversal; its serializer does not already provide the required authority/digest contract. No bespoke protobuf codec is recommended. |
| G9 Architectural fitness | **Unresolved**, following AP-02/AP-04/AP-05/AP-07 independently. |
| PS-G1 Physical semantics | **Unresolved for cutover fidelity**, F01/F02; existing physical identity and owner checks are preservation constraints. No new physical-model defect is established. |
| PS-G2 Well-posedness | **Not applicable to membership design**: this mechanism changes no variable roles, incidence analysis or solve admission. Adjacent numerical capability is unqualified here. |
| PS-G3 Numerical integrity | **Unresolved for replay preservation**, F01/F02 and full proof drain; new solver-state accuracy/convergence is outside this review. |

## Verification boundary, rule impacts and decision

Source reading establishes the current concrete consumer closure, exact framing,
request/qualification bindings and receipt-drain mechanisms. Exact-release source/API
reading establishes the proposed Serde extension point and its normalization traps.
No probes or tests were run (`not_run`); no build, broad audit, static campaign,
generated publication or scientific journey is represented as passed. The failure
target for eventual targeted controls remains zero. Behavioral implementation evidence
and assembled H qualification stay with their owning packages.

| Rule impact | Current rule/location | Proposed treatment and effect if retained |
| --- | --- | --- |
| <a id="rc01"></a>RC01 | Plan65e, “Remaining recipe qualification decision,” prefers typed owner/version/tag/scalar-bit membership. | Replace that preference with the selected explicit contract after F01–F03 are settled. If retained, typed membership requires an intentional new interpretation and migration of every concrete consumer; it cannot be called preserved V1. |
| <a id="rc02"></a>RC02 | Exact JSON artifact parsing and `MathAdmittedRecipeV1`/interpretation association at `pse-ids::scientific_replay` and `typed_math_portable::Recipe`; blueprint §14.3 describes opaque exact-payload/record authority. | Route the new artifact interpretation, native-root qualification premise and any hash change through the coordinator's proposed ADR/architecture amendment before implementation. Preserve the exact-payload/qualification/request and nested-record guarantees. Keeping the current implementation association keeps JSON and postpones the full protobuf recipe target. |

No licence restriction, SHOULD deviation, generic proof machinery or change to scientific
meaning/solver ownership is recommended. Duplicate-key refusal, if selected instead of
exact survivor semantics, must be recorded explicitly with RC02; it is not an inferred
pre-existing constraint. This review does not change those rules or their acceptance.

**Behavioral adequacy: unresolved. Architectural fitness: unresolved. Overall: Not
Accept the current cutover proposal.** The preferred remedy is the DOM-free Serde
alternative preserving normalized membership V1, all concrete consumers, original
qualified scope and complete receipt drain. Settle exact root binding, normalization/
survivor semantics and concrete bounded admission in the decision before implementing
authority or hash changes. Existing JSON remains the operational baseline meanwhile;
the DOM-backed native-root alternative is a viable deliberate stage if its temporary
machinery and deletion boundary are justified. Typed membership remains a separate
credible choice when its changed meaning has an actual requirement and complete design.
