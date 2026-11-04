---
id: ADR-0245
title: Bind native recipe membership to authenticated bytes and admitted reconstruction
status: proposed
date: 2026-11-04
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-05, AP-07, DP-01, DP-24, PS-01, PS-03]
blueprint: [§5.3, §14.3]
review: docs/design_review/reviews/design_review_plan65_native_replay_membership_2026-11-04.md#follow-up-design-assessment
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A recipe requires unsupported normalization, or an ordinary supported shape cannot be admitted through borrowed-view adoption and complete validation.
verification: F01 exact-byte/same-root canonical and private reconstruction; F02 complete surviving-node differential membership against the pinned DOM oracle; F03 source-profile extent, expansion, overflow, exhaustion, cancellation and original-drain controls before protobuf activation.
standard: {core: "3.4", heuristics: "1.0", process-simulator: "1.5"}
---

# ADR-0245: Bind native recipe membership to authenticated bytes and admitted reconstruction

## Context

Current exact JSON qualification binds original payload digest, qualification and request;
nested physical/math replay also requires record membership and complete receipt use.
Full Proto conversion must preserve these contracts without a complete JSON value tree or
associating independently supplied native data with the payload's authority. Review F01–F03
identify the association, normalization and whole-lifetime admission obligations.

## Scope

Amend blueprint §5.3/§14.3 for controlled native-root qualification and DOM-free structural
membership V1. Future protobuf recipes use `pse.admitted-body.v4`; JSON v3 remains authoritative
until complete replacement and targeted acceptance. Encoding changes exact-byte digest and
consequent scope binding; structural V1 preservation does not promise identical artifact
identities across encodings. This proposal supports prerequisite implementation within
authorized Plan65, not full activation or formal ADR acceptance.

## Drivers

Scientific meaning remains at native owners; Buffa supplies transport and local validation.
Canonical recovery and the private child must reconstruct the same authenticated recipe they
indexed, under original receiving context and ownership. Changed records, selected providers,
request, receipt order and physical premises must refuse independently of wire decoding.
Pure membership controls need no solver/database; real receiving controls exercise original
owners. New scientific records must not require another handwritten semantic schema.

## Options

Retaining the qualified JSON leaf is safe today but retains its DOM peak and excludes the
full recipe from source-derived transport. An owner/tag/bit visitor introduces a new membership
interpretation and another scientific declaration/consumer migration. A whole-payload permit
removes per-record protection. Native-to-DOM conversion could preserve V1 but retains the peak.
Select native Serde traversal with pinned JSON normalization and monomorphic compiler receivers:
reuse actual serialization without another scientific IR, while explicitly implementing
normalization, source-profile admission and borrowed adoption.

## Outcome

Only audited monomorphic canonical/private compiler receivers establish the new native-root
premise. They accept immutable exact bytes and original digest/qualification/request/context,
verify interpretation, decode once into a private owner, borrow that root to build membership,
then consume the same root for strict reconstruction. No caller-supplied root/index is accepted.
Their `pub unsafe` boundary asserts audited origin; Rust does not establish caller identity.
The wire-independent identity crate may expose the corresponding unsafe native-root primitive
for those reviewed paths. Safe decoding/hashing never issues permission, readiness or
publication rights.

Serde `Serializer::Ok` carries digest and contiguous postorder event range. Object children
retain rendered key, digest, range and emission order; lexical processing keeps the last equal
rendered key. Remove overwritten descendant ranges before publishing a sorted, deduplicated
owned digest vector; identical digests surviving elsewhere remain admitted. Queries share the
normalizer without constructing a global index. Pinned serde_json primitive conversion and a
one-entry map-to-null helper supply numeric/key semantics; helper dummy nodes never enter the
index. Preserve human-readable mode, null/unit/None, newtypes, external variants and actual
emitted lengths. The selected profile excludes arbitrary precision and explicitly refuses
integers outside combined i64/u64 range and unsupported private raw/number sentinels, including
earlier receiving refusal. Actual recipe owners and resolved features delimit support;
universal equivalence for arbitrary Serde implementations is not claimed.

### Consequences

One existing operation workspace owns receiving through reconstruction and original drain.
Canonical receiving uses its lease; the private child's ceiling is bound to the parent-owned
original request. Decoded content grants no lease. Replace the JSON byte multiplier with
the concrete phased route assessed in the review:

1. Pre-admit the source-generated lazy probe, retained fragment/vector/stack capacities and
   eager oneof/map/group/extern carve-outs. Share one decode context across recursive traversal
   and merge singular fragments into one logical message.
2. Drop the probe; pre-admit eager borrowed view, complete validation and native-adoption peaks.
   Decode the same bytes and use generated borrowed adoption into the actual native recipe.
   No owned protobuf tree is selected.
3. Before `Validate`, admit the source-derived worst-case complete violation population,
   paths, keys/messages and rule temporaries. The existing API collects all violations and
   has no fail-fast/sink argument; a sink requires a separately qualified change.
4. Count actual native Serde emissions before membership allocation; admit events, keys,
   child buffers, dead ranges, final index, normalization helper and maximum concrete query
   scratch. Include simultaneous recipe/receipt ownership; consume streams into replay or
   admit retained copies.
5. Validate scalar restoration expansion against selected signatures and checked-sum actual
   capacities before allocation. Resolved syntax's `formal_count` expands placeholder strings
   beyond wire slots, potentially again for each selected function; count those occurrences.

Use actual source declarations for extent/adoption support, without a shadow DTO, bespoke
protobuf parser, generic cost framework or new workspace owner. All phases preserve typed
refusal, cancellation and original operation/process settlement.

### Compensating controls

Keep the current production leaf until full activation evidence exists. Differential controls
compare every surviving node with the pinned DOM oracle, including overwritten nested maps,
duplicate rendered keys, order, variants, numeric boundaries and sentinel refusals. Receiving
controls attack changed bytes/root/digest/request/interpretation/physical context and preserve
record/complete-receipt refusals. Exercise compact default expansion, eager carve-outs, skewed
maps, formal-count multiplication, overflow, exhaustion, cancellation and drain. Historical
interpretation support is an explicit reader decision, never fabricated old authority.

### Confirmation

The review is **Accept scoped at Proposed strength** for this prerequisite architecture;
full protobuf activation remains **Not Accept** until F01–F03 implementation and original
targeted evidence exist. Plan65e owns these obligations/dispositions; 65h owns assembled
scientific/extension and benefit qualification. No new Tested, Measured, complete memory or
numerical adequacy claim follows.

## Pros and cons

This avoids a complete membership DOM and second scientific schema but requires exact
normalization, last-wins survival and source-profile extent/adoption work. Canonical map
validation is borrowed and allocation-free but may perform quadratic unique iteration;
ordinary map growth/skew remain execution-fit concerns. No speedup or capacity is inferred.

## More information

[Plan65e](../plans/65e-scientific-portability-and-execution.md#remaining-recipe-qualification-decision)
owns delivery. The [independent follow-up](../design_review/reviews/design_review_plan65_native_replay_membership_2026-11-04.md#follow-up-design-assessment)
retains detailed phases, source findings and activation obligations. Library sources:

- [Serde 1.0.229 Serializer](https://docs.rs/serde/1.0.229/serde/ser/trait.Serializer.html):
  custom `Ok`, compound serializers, actual emitted entries/elements and human-readable mode.
- [serde_json 1.0.151 value serializer](https://github.com/serde-rs/json/blob/v1.0.151/src/value/ser.rs)
  and [number representation](https://github.com/serde-rs/json/blob/v1.0.151/src/number.rs):
  integer/float normalization, key rendering and map replacement.
- [Buffa 0.9.2 lazy views](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/view.rs)
  and [decode context](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs):
  `decode_lazy_with_ctx`/`merge_lazy`, `fragments`/`raw_elements`, shared unknown/element/depth
  state and eager `DecodeOptions::decode_view`. Default getters recreate subtree allowances;
  fragment charging alone omits retained spare capacity.
- [Maintained validator](../../vendor/protoc-gen-protovalidate-buffa/src/emit/mod.rs):
  eager owned/view `Validate` collects complete violations; lazy views lack that validator
  and reflection. Generated borrowed native inverse adapters remain prerequisite work.

## Status history

- 2026-11-04 — Proposed after independent Accept-scoped assessment, before native-root
  authority/hash changes; full recipe activation remains withheld.
