---
title: "Plan61 contract and generation foundation"
date: 2026-10-31
doc_role: review
doc_owner: docs/plans/61a-contracts-generation-and-validation.md
doc_topics: [design, libraries, validation, diagnostics]
---

# Plan61 contract and generation foundation

## Scope, evidence and disposition

**Accept-scoped**, independent prospective assessment by the A02 design reviewer on
2026-10-31, edited for publication by the coordinator. This change-tier target review combines
library and principle focus. It consumes the [current scoped refresh](../audit/plan61-contract-foundation.md),
source `580a5bd4f`, RC01–RC03 and the approved execution approach. Git identifies publication
separately from that baseline. Core 3.4, heuristics 1.0, test assessment 1.2 and the binding
apply. No new runtime test or benchmark ran in this review.

The acceptance is architectural/domain and behavioral target adequacy, conditional on the
realizations below. It does not accept implementation, mark an ADR accepted or qualify later
callers. The boundary covers wire declarations, generated representations, descriptors,
preflight and publication. B/C/D migration, managed custody/reconnect and Q01 remain excluded.
No equation, physical convention, solver choice or scientific result changes.

## Model, alternatives and execution

Selection, predicates, requested answer, displayed detail, observations, qualified context,
failure and actual completion are distinct phenomena. Unknown is not false; partial counts
are not exact; cancellation and delivery do not establish CPU settlement.
Proto may own the new wire boundary without displacing provider authority, cloning registry
meanings or forcing CLI presentation into ProtoJSON. Host-local schemas and a narrow module
suffice; compiler/generator dependencies remain tooling-owned. A new crate adds no demonstrated value.

Reuse one compiled Buf input for preflight, generators and descriptor export, preserving
owned/imported selection and external-type mappings. Match message/packaging corpora and
shared-pool settings. Retain full descriptors separately from stripped runtime projections.
Correct the selected validator compiler's parser/schema/lowering pipeline instead of adding
a second checker or default dynamic conversion bridge. Arithmetic correction must include
error propagation and masking. A02 generation succeeds before the writer acts; no new durable
transaction framework or filesystem-wide crash-atomicity claim is needed.

| Change/failure scenario | Required behavior |
| --- | --- |
| Add fact/control alternative | Change its declaration; migrate actual callers in their package |
| Replace generator/runtime | Qualify consumed contracts; no silent fallback/listener |
| Invalid/unsupported required CEL | Rule/schema diagnostic before publication |
| Valid CEL arithmetic error | Typed evaluation error and correct masking, no panic |
| Preflight/generator failure | Previously published outputs unchanged |
| Passive source/schema read | No implicit generation/service preparation |
| Schema-to-native association | Original source/Cargo/provider qualification remains required |

## Foundation and behavioral judgments

AP-01/AP-03/AP-06/AP-07 accept scoped ownership/composition: separate provider/wire/presentation
owners, explicit capabilities, analyzer-free tests and one compiled input/direct projection.
AP-02/AP-04/AP-05 accept-scoped: generated contracts expose consumed meaning, but native
contextual enforcement, substitution and explicit publication/rule diagnostics require implementation.
G1/G4/G8/G9 accept the target on these grounds; G2/G3/G5/G6/G7 depend on the revealing checks
below, not current runtime passes. PS-G1–PS-G3 are outside this non-scientific boundary.

## Findings and readiness

### F01 — Scoped declaration authority

Proto owns new wire meaning; registry meanings remain derived; native presentation remains
directional. A02 supplies a usable B/C/D contract, not a renamed toy service. Challenge unknown/
false, exact/lower-bound count, raw paths, context guards and typed failure/control distinctions.

### F02 — Preflight before aggregate writes

The inspected writer ordering permits partial aggregate effects before Plan61 generation.
Construct and validate the A02 tree before any aggregate publication. Inject invalid CEL,
descriptor conflict and plugin failure with existing outputs present and assert no modification.

### F03 — Supported validation semantics

Caught division panic and misclassified malformed CEL are defects. Reuse descriptor-bound
preflight with invalid/unsupported diagnostics, correct arithmetic/conversions and error
propagation/masking, and preserve owned/view parity. Unsupported required lowering refuses
publication. Assert exact error categories for malformed syntax, unknown fields/functions,
wrong types, constant/dynamic zero division, overflow and masked errors.

## Rule impacts and completion boundary

ADR-0235/blueprint §21.7 realize operator-confirmed RC01/RC02; RC03 remains hard cutover
without data-deletion authority. No further rule impact or workspace crate is selected.
Plan61a owns scheduled dispositions. Publish this review and the scoped decision route before
production enabling; fulfill F01–F03 before A02 completion. Formal ADR acceptance remains with
its governing PR. This review supplies no corrected-generator or assembled qualification.
