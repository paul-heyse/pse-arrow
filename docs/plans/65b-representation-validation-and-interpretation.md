---
title: "Plan65b: Representation, validation and interpretation"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [current-work, design, libraries, validation]
---

# Representation, validation and interpretation

## Target and assessed foundations

**Proposed:** give each selected boundary an explicit interpretation and assurance policy,
implemented by library capability plus only the remaining domain gap. Binary forwarding,
exact-byte archival, native adoption, transformation and human inspection are different
operations; they must not inherit one undifferentiated unknown-field or strictness policy.

Execution starts after Plan64 is implemented and may replace its representation/validation
decisions. The [coordinator](65-proto-integrated-target.md) owns combined scope; B01–B07 below
own this companion's progress. Observed source at `f61286a5fa5894b480efe8584c8b3d9b76d1b1fb`
plus dirty projection changes is **Interface-checked**, not tested post-Plan64 behavior.
Core 3.4, efficient-architecture heuristics 1.0 and process-simulator 1.5 govern the design.

Current local rules expose Required, UnsignedRange, Length and Utf8Bytes. The maintained
Buffa validator is a corrected bounded CEL lowerer. Host payload inspection already distinguishes
decode/validation and performs strict JSON duplicate-key/alias/trailing-input controls; its
intermediate JSON tree is an optimization question, not proof of missing validation. Native
projection adapters already enforce owner construction and distinguish typed adoption errors.
These are useful foundations to improve rather than a requirement to retain every check/pass.

## Interpretation policy and validity lifetime

Represent the policy selected by an actual operation as part of its closed contract world:
accepted grammar, message/type identity, presence/unknown behavior, local validation policy,
supported native interpretation and caller-selected admission limits. Separate these facts
from database mapping revision and scientific model identity. A reader identifies the policy
it used; a historical read does not silently apply newly strengthened constraints and claim
the original record was invalid or unchanged.

The selected historical contract is **read under the original retained policy and expose its
evidence scope; current adoption requires current explicit admission**. Migration to a new
policy is a separate versioned operation that can refuse, reconstruct or regenerate. Retain
the exact original bytes when byte identity is promised. Bind original descriptors/policy
availability to retained-record lifetime in C packages and expose absence/incompatibility
truthfully in F/G readers. Do not keep an old production writer merely to preserve history.

Use four separately attributable stages:

1. Frame/admit original bytes before operations that can allocate from uncontrolled lengths.
2. Decode and enforce declared local structural/value constraints in the selected grammar.
3. Fallibly adopt a known native interpretation through invariant-bearing owner construction.
4. Admit scientific/revision/transaction context at the actual operation owner.

Carry established local properties through immutable values and representations that preserve
their premises. Revalidate when uncontrolled mutation, policy/world change, native adoption or
new context invalidates them. Do not turn every internal layer crossing into another full
encode/decode/validate pass. Conversely a type named Validated does not establish bounded
allocation, current context or scientific readiness.

## Concrete representation decisions

Use Buffa owned messages for ordinary first-party exchange and eager generated views for
read-heavy borrowed payloads. `OwnedView` may retain backing `Bytes` across awaits. Read
generated values through vtable `ReflectMessage::for_each_set` without dynamic materialization
where only inspection is needed. Generic mutation uses `DynamicMessage` and checked
`ReflectMessageMut::try_set/try_clear`, followed by policy validation and native adoption before
publication. Direct `field_mut` replacement is not an accepted untrusted mutation boundary.
Generated/dynamic bridges encode/decode and are not a numerical hot-path optimization.

Lazy views remain a selected-boundary option, not a universal default: they lack the same owned,
reflection and text surfaces and may repeatedly decode nested fields. Custom storage/layout is
selected only where an actual operation benefits and all singular/repeated/map/oneof codec,
reflection, validation and JSON traits are supported. Standard generated storage is the default.

Keep externally specified Prost/Tonic contracts in that stack and cross through an explicit
wire/domain boundary. Use `prost-reflect` for its actual descriptor-aware consumer. PBJSON is
selected only when that externally owned Prost population needs compiled ProtoJSON; no duplicate
PBJSON route is introduced for first-party Buffa messages. Native-only and Arrow remain complete
alternatives where an independent Proto interpretation adds no useful capability.

### Format and value policy

| Distinction | Selected handling and implementer consequence |
| --- | --- |
| Presence/default/null | Preserve explicit optional/alternative semantics; missing is not clearing. Report declared defaults separately from set presence. Exact-parent edit semantics remain at the domain owner. |
| Unknown binary fields | Forwarding may retain them; native adoption refuses unknown meaning it cannot represent. Unknown alternatives/enums never default to success. Preserve original bytes for exact archive promises. |
| JSON unknowns and duplicates | Declare ProtoJSON versus native Serde. Uncontrolled adoption rejects unknown keys and duplicate logical fields including proto/JSON aliases before generated JSON defaults can ignore them. Passive inspection may disclose unknowns under its separate policy. |
| Numeric and temporal values | Preserve full integers, exact decimals, IEEE bits and quantity precision explicitly; use a supported signed/unsigned temporal intersection. WKT Duration is not native nonnegative duration; JSON `double` is not exact arbitrary precision. |
| `Any` | Resolve the declared fully qualified type and admitted URL convention in its selected world; whole-URL typed matching and dynamic resolution are not interchangeable identity checks. No arbitrary executable plugin admission follows. |
| Enum aliases/Proto2 | Test the actual imported population's required/default/closed-enum behavior and canonical alias needs. A generated integer or successful decode alone does not establish those rules. Full Proto2/MessageSet support is not a universal new requirement. |
| Well-known types | Use explicit selected mappings; test Timestamp range/offset, FieldMask name/selection meaning, `Value` nonfinite behavior and descriptor-aware `Any`. Do not remap `.google.protobuf` wholesale to a partially conformant JSON companion. |
| Redaction | Source-owned sensitivity can emit standard `debug_redact` where supported. Debug redaction does not protect ProtoJSON/text/binary exports; routine summaries and intentional payload inspection have separate disclosure policies. |

Prost `Message::merge` is nontransactional: decode into a fresh value, validate/adopt and only
then replace published state. Its protobuf duplicate/merge rules differ from application edit
semantics. The reference's 0.14.4 implicit-packed-option issue is a concrete imported-schema
discriminator, settled by explicit packing or a verified corrected release for that population,
not a reason to rewrite a working external protocol.

## Validation composition

Stable first-party contracts select owner-declared rules → A02 typed/options emission →
maintained generator preflight → generated `Validate` for owned/view messages. Cover uncontrolled
ingress and relevant producer/outgoing boundaries. Extend LocalRule only for selected consumed
rules, with matching native and emitted semantics. First additions are composable code-point and
UTF-8 byte constraints where both are demanded, local format checks with a concrete client,
and meaningful collection/cross-field relations. Reject unsupported rule combinations before
publishing generated code rather than silently dropping one projection.

Do not expose arbitrary upstream CEL merely because it is documented. Distinguish schema
validity, backend generation coverage and actual invocation coverage. Compiler/evaluator errors
are infrastructure/implementation failures, not ordinary user violations. Keep `violations`,
`compile_error` and `runtime_error` separate, preserve exact arithmetic/error masking guarantees,
and produce typed field/rule paths for F's attributed diagnostics. Public truncation must not
manufacture a complete trusted violation account.

For an approved runtime-selected schema consumer, B04 has a bounded dynamic-validation branch:
`prost-protovalidate::ValidatorOption::MessageDescriptors`, `AdditionalDescriptorSetBytes` and
`DisableLazy` preload one immutable closed world. Per-call `ValidationOption::NowFn` supplies
the operation's declared logical clock for replayable relative-time rules. `Filter`/`FailFast`
results carry their exact selected scope; unknown-constraint tolerance cannot establish complete
enforcement. This branch is not a fallback for unsupported stable generated rules.

`prost-protovalidate-build` native-only `fail_on_runtime_only(true)` can provide a comparison for
selected schemas; default warnings/skipped implementations are not runtime fallback. Its Buffa
bridge adds reflection/CEL/layout and output constraints. Use the maintained dedicated Buffa
generator for the stable first-party population; select a runtime validator only for an actual
approved dynamic consumer with its complete lifecycle/cost justified in B04.

## Allocation, encoding and lifetime

Match useful detail at the producer before hydration or encoding. Then choose physical library
mechanisms at the actual ownership boundary:

- `Message`/`ViewEncode::try_encode_bounded` and cache variants preflight encoded size before
  writing. Size refusal leaves the sink unchanged; it does not reserve every later allocation.
- `SizeCachePool` reuses spill slots and amortizes allocation, not traversal; choose retention
  from actual repeated work rather than treating cache slots as payload bytes.
- `Rope::with_backing`, `with_min_segment`, `into_segments` and `RopeBuf` may preserve useful
  borrowed segments. Compression, JSON, interceptors and contiguous transport can flatten them.
  Copy a compact subset when that releases a much larger backing allocation sooner.
- Binary, JSON and text parsing have different allocation/element/depth behavior. Keep admission
  valid for the chosen grammar and cumulative operation, not just each framed message.

Pools and buffers belong to the operation/session that can establish their validity and release,
not a new global cache. D/F/G packages integrate the selected physical routes; B06 supplies
their common discrimination controls without pretending a codec benchmark qualifies a solve.

## Packages

| Package | Status | Inputs → deliverable | Migration/deletion and targeted acceptance |
| --- | --- | --- | --- |
| B01 | planned | Post-Plan64 producer/consumer population plus A01/A02 semantics → explicit operation policies and historical interpretation | Classify every selected boundary as forwarding/archive/adoption/transformation/inspection; publish policy and identity associations once. Remove inherited implicit format assumptions. Test original-policy read versus current-policy adoption and unavailable old policy. |
| B02 | planned | B01 policies and A03 world artifacts → faithful binary/JSON/text decode and fallible native adoption | Migrate host, engineering, worker, reference and selected durable diagnostic/document consumers. Remove replaced per-consumer presence/default/key/type checks only after equivalent shared policy exists. Test large integers, exact bits, alias keys, unknowns, fresh-decode merge and typed refusal. |
| B03 | planned | Actual selected local-rule needs plus A02 options → complete native/generated rule parity | Extend the narrow rule vocabulary where useful, update maintained preflight and owned/view generated validators, and delete duplicate local scalar checks. Test unsupported combinations, mixed arithmetic, error masking, codepoint/byte differences, outgoing violations and category-preserving failures. |
| B04 | planned | Approved dynamic schema inspection/import consumer, B01 policy and closed descriptors → choose whether dynamic execution is warranted | Compare eager closed-world runtime validation with generated stable support and explicit unsupported refusal. A runtime choice must pass logical-clock replay, unknown constraints, filtered/FailFast scope, initialization and retained-world controls. If no consumer benefits, record non-selection and remove prototype-only code; do not install a fallback. |
| B05 | planned | B02/B03 and selected external protocol population → exact WKT/Any/Proto2/alias conversion contracts | Configure only actual external mappings; migrate affected ProtoJSON readers/writers together. Delete redundant JSON conversion routes. Independently test Timestamp limits/offsets, signed Duration, FieldMask intent, nonfinite Value, required/default fields and unknown enum/extension behavior. |
| B06 | planned | Actual D/F/G selected producer paths → demand-shaped representations and bounded lifetimes | Integrate eager views/owned values/direct reflection, bounded encoding and conditional segmented output; remove superseded temporary buffers and repeated conversions. Test async retention, oversized backing slices, compression/flattening, cumulative limits and cancellation release; compare matched complete boundaries for measured claims. |
| B07 | planned | All selected population migrated → conformance/fidelity controls and documented extension route | Add a field/alternative/rule to one boundary and show generated/adopted/readback behavior; test stale-world reuse, mutable invalidation and public omission. Remove old adapters/fixtures only with migrated consumers. Hand assembled controls to65h; do not infer science or throughput from wire conformance. |

## Verification and completion

**Proposed:** B tests independently construct expected native meaning and malformed inputs;
generated round trips test correspondence but cannot alone establish semantic fidelity. Named
upstream conformance populations, application policy, scientific admission and measured benefits
remain separate evidence scopes. Use package-local `scripts/pse check` and targeted `scripts/pse
test` during implementation, with explicit force-validation for Arrow closures. Regenerate when
rules/declarations change. Scope-end hygiene and assembled evidence belong to65h.

Completion requires every selected producer/consumer to obey its declared policy, historical
interpretation to be explicit, validation coverage/error categories to be honest, and obsolete
parallel conversion/checking routes to be deleted. It does not require every library feature
to be adopted. No correctness or numerical/performance evidence was executed while authoring.

## Sources and conditions

Supporting accounts: [representation briefing](../library_leverage/libraries/proto/briefings/representation-reflection-and-validation.md),
[Buffa](../libraries/proto/Buffa.md), [Prost](../libraries/proto/prost.md),
[reflection/PBJSON](../libraries/proto/prost_reflection_and_pbjson.md), and
[validation](../libraries/proto/protobuf_validation_rust.md). The end-to-end reread includes
their original and advanced sections; exact releases remain distinct from main/development APIs.

Precise mechanisms: [Buffa 0.9.2 reflection](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/message.rs),
[dynamic messages](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs),
[encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs),
[size cache](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/size_cache.rs),
[sinks](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs),
[Prost merge](https://docs.rs/prost/0.14.4/src/prost/message.rs.html),
[PBJSON builder 0.9.0](https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html),
[runtime validator 0.6.0](https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidatorOption.html),
[native builder](https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/struct.Builder.html),
and [maintained generator](../../vendor/protoc-gen-protovalidate-buffa/README.md).
The plan selects exact local mechanisms where researched; implementation may move an exact pin
or family with its affected controls, rather than freezing an obsolete defect into the target.
