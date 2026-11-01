---
id: ADR-0235
title: Declare code-intelligence wire contracts and explicit generation
status: proposed
date: 2026-10-31
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-15, DP-19]
blueprint: [§21.7]
review: docs/design_review/reviews/design_review_code-intelligence-contract-foundation_2026-10-31.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A consumed wire meaning needs competing declarations, direct reflection needs repeated dynamic transcoding, or generated validation cannot preserve required CEL error semantics.
verification: Plan61 A02 generation/preflight, descriptors and validation; B/C typed source/API/native consumers, authenticated local RPC and original work settlement; D and Q01 retain schema/payload and assembled acceptance.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/61a-contracts-generation-and-validation.md]
---

# ADR-0235: Declare code-intelligence wire contracts and explicit generation

## Context

Plan61's operator-confirmed RC01/RC02 select typed evidence and generated local helper
communication within blueprint §21.7. A01 tested representative composition; its test protocol
is not the production declaration or acceptance of migrated consumers.

## Scope

Amend code-intelligence declaration and metadata conventions inside the existing host and
explicit xtask generation boundary. Preserve ADR-0233/0234 custody and CPU settlement.
No new workspace crate, universal service or scientific policy moves into the host.
This record remains proposed until its governing decision PR changes status.

## Drivers

One declaration must support reusable typed facts, direct reflection and schema inspection
without interpreting category-sensitive JSON. Invalid schemas, ordinary violations and evaluation
defects remain distinct. Source-only work acquires no analyzer or generator prerequisite.

## Options

Strengthened native JSON retains handwritten framing and boundary interpretations. Prost/Tonic
is the coherent alternative if the selected composition fails, with simpler UDS hosting but
different view/reflection costs. Select Buffa/ConnectRPC and native validation for demonstrated
owned/eager-view projection; correct the existing generator instead of adding a competing CEL
parser or silently bridging every value. One compiled schema input feeds generation and descriptor
publication. Explicit local preparation adds maintenance; no quantitative performance claim follows.

## Outcome

New code-intelligence wire meanings are proto-authored. Existing registry meanings remain
derived; deliberate CLI presentation remains Serde/Schemars-owned. Generated types can be
consumed directly; checked construction adds contextual invariants, not a mirror wire DTO.

The B/C realization uses generated binary gRPC on the existing private Unix socket. Kernel peer
authentication repeats before each physical connection, including reconnects. Session-retained
workers own native completion independently of handlers and delivery; explicit cancellation
preserves the original outcome through Terminal. Plain observation DTOs remain available without
HIR execution dependencies. Native public presentation remains deliberate Serde/Schemars output;
original provider documents remain qualified inspection evidence. No legacy listener is retained.

Plan60a extends this same boundary with role-qualified relationship facts and one bounded
multi-seed native operation. Canonical declaration endpoints, occurrence witnesses and original
session/configuration/generation remain distinct. Traversal selection and answer selection are
independent; a shared request-local kernel preserves seed attribution, bounded support and
unknown coverage. Typed facts, rather than rendered context, drive composition. This extends
the selected host/schema boundary without a new crate, service, universal identity or persistent
graph. Plan60a owns its implementation and targeted acceptance.

### Consequences

Publish messages/views/validators/service bindings and full descriptors through GeneratedTree.
Distinguish the full source-aware artifact from its stripped runtime pool. Preserve native
HIR/rustdoc/source context. Preflight and staging precede aggregate writes. Corrected CEL
lowering preserves arithmetic, error and short-circuit semantics; unsupported required rules
refuse before publication.

### Compensating controls

Exact local tools, locked imports and revealing owned/view, descriptor and invalid-rule tests
protect this scope. No runtime fallback, compatibility listener or implicit generation follows.
A maintained local third-party generator correction is selected; upstream contact/publication
is separate authorization.

### Confirmation

The scoped review assesses architectural/domain and behavioral adequacy separately. Its
Accept-scoped judgment covers only the proposed A02 foundation under the operator's implementation
instruction. Plan61 owns implementation evidence. Review disposition is not formal ADR acceptance
or assembled qualification.

## Pros and cons

Shared representations remove duplicate interpretation, while explicit tool preparation and
local generator corrections add maintenance. Keep that correction bounded by semantic tests;
no general new CEL engine is selected.

## More information

- [Plan61a](../plans/61a-contracts-generation-and-validation.md) owns progress.
- [Plan61b](../plans/61b-typed-investigation-and-agent-answers.md) and
  [Plan61c](../plans/61c-local-rpc-and-native-completion.md) own consumer/transport acceptance.
- [Scoped review](../design_review/reviews/design_review_code-intelligence-contract-foundation_2026-10-31.md).
- [Architecture](../authoritative_design/sections/workflows-and-results.md#section-21-7).

## Status history

- 2026-10-31 — proposed for operator-authorized A02 implementation; formal status unchanged.
- 2026-11-01 — recorded operator-authorized B/C realization within the selected boundary; the
  scoped review remains A02-only and formal status remains proposed.
- 2026-11-01 — recorded operator-authorized Plan60a typed relationship and bounded composition
  extension before dependent implementation; formal status and the prior scoped review are unchanged.
