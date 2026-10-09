---
title: Flow projection V2 canonical framing
date: 2026-10-09
tier: change
purpose: target
status: review
---

# Flow projection V2 canonical framing

**Architectural fitness: satisfied. Behavioral and semantic adequacy: satisfied for the proposed contract. Overall decision: Accept the bounded target design.**

ADR-0167 provides a proportionate correction to an unresolved framing premise. It makes collection membership and parent boundaries explicit while keeping physical admission, graph algorithms, numerical policy and hashing under their existing owners. It introduces neither another graph representation nor a reuse mechanism.

This is acceptance of a **Proposed** design, supported by **Interface-checked** contracts and source inspection. V2 implementation, consumer integration and regression behavior remain to be established by execution evidence. This review neither demonstrates an admitted V1 collision nor qualifies the enclosing Plan 28 architecture.

## Scope, target and coverage

This independent **CHANGE / TARGET** review applies Core/template 3.4, Heuristics for Efficient Architecture 1.0, ProcessSimulator 1.5 and the selected pse-arrow binding.

The functional target is dependable flow preparation for authored topology, tear selection and causal recycle execution. The distinguishing change is evolution of a canonical identity projection whose nodes, ports, connections, bindings and decisions have variable cardinalities. Structural edits must change the projection as specified; declaration reordering must not.

The inspected baseline is `main` at `46545b2ad3e2999b4335692ac49af6091438c3fc`, with proposed ADR-0167 and concurrent implementation/documentation work. The inspected flow source already contains the independent N12 binary decision lookup. It still emits FlowProjectionV1. Unrelated compiler/preparation changes are outside this review.

Inspected owners and adjacent consumers:

- `crates/pse-ids/src/derive.rs`: frame catalog and framed scalar/digest encoding.
- `crates/pse-structural/src/flowsheet.rs`: physical admission, canonicalization, equality, fingerprint construction and tear witnesses.
- `crates/pse-compiler/src/workspace/modeling/flow.rs`: selected authored topology and physical projection.
- `crates/pse-runtime/src/math/flows.rs`: allocation ownership, public graph document and fresh tear-attempt controls.
- `crates/pse-backend-native/src/tears.rs`: native problem identity, assumptions, compatibility and independent recovery.
- `crates/pse-runtime/src/workflow/strategies/conditional.rs`: recycle causal identity consuming the graph key.
- Blueprint §5.3 and §15.3, with the neighboring physical-flow and recycle contract.
- ADR-0167 and the original graph/hash review's framing inquiry.

No numerical iteration, provider thermodynamics, full simulator adequacy or persisted-product compatibility was requalified. The bounded source search found no directly declared stored FlowGraph family; that observation does not establish absence of every external document consumer.

Current followup and acceptance status belongs to [Plan 28k](../../plans/28k-graph-kernels-and-hashing-investigations.md), with its linked Plan 28 consumer work. This review does not maintain a second disposition ledger.

## Ownership and contract

The proposed responsibility split is coherent:

| Responsibility | Owner and consumed contract |
|---|---|
| Hash family, domain separation and framing primitives | `pse-ids`; unchanged BLAKE3 derive-key construction, length-prefixed parts and typed identifiers |
| Physical topology and canonical projection fields | Structural flow admission; sorted declarations, checked bindings, original occurrences and declared policies |
| Authored selection and closure | Compiler flow projection; known instances, complete selected connections and explicit tear declarations |
| Documents and product ownership | Runtime flow owner; exposes the admitted projection without reconstructing its meaning |
| Tear optimization and witness recovery | Native integration and structural graph owner; fresh attempt and independent residual-DAG check |
| Recycle composition | Workflow owner; combines the graph key with revision, case structure, values and request |

V2 changes the projection grammar, not these responsibilities. Each top-level collection receives its named tag and u64 count. Each item receives its named tag. Ports are counted independently under each node; bindings are counted independently under each connection. Existing fields retain their order and encoding.

Using `FramedHasher::str` for tags, `u64` for counts and existing identifier methods makes the grammar explicit through the existing identity mechanism. No serializer schema, second hasher or generic operation language is required.

### Physical meaning and equality

| Element | Meaning and authority | Preservation obligation |
|---|---|---|
| Port | Quantity and unit identifiers from the checked registry/context | Preserve identifiers and canonical order |
| Binding | Checked compatible physical contracts and resolved affine conversion | Preserve conversion scale/offset, full retained quantity contract and rejection behavior |
| Connection | Directed original occurrence between declared unit owners | Preserve direction, multiplicity, binding membership and witness identities |
| Decision | Explicit group, finite nonnegative cost and tear policy | Preserve cost bits, policy values and grouped selection |

V2 does **not** encode every field of the retained `Binding.quantity`. Its digest is a versioned projection fingerprint, not a complete substitute for graph equality across physical registries. `FlowGraph::PartialEq` compares declarations, physical bindings and the key; this distinction must remain.

The public `FlowGraphDocument.identity` description currently calls it the identity of the “exact admitted flow graph.” During implementation, clarify that wording to express the versioned projection fingerprint and its physical-context limit. This is an implementation documentation obligation already implied by ADR-0167's compensating controls, rather than a required redesign.

The flow projection is not equation–variable incidence and does not establish equation well-posedness. V2 must leave those authorities separate. Tear witnesses establish the declared residual topology's acyclicity and policy compliance; they are not numerical convergence evidence.

No formulation, derivative, scaling or numerical solver stage is changed by this proposal. Consequently, the profile's detailed numerical-stage columns are not applicable here.

## Distinguishing scenarios

| Scenario | Expected behavior and boundary | Evidence |
|---|---|---|
| S01 — Reorder equivalent declarations | Canonical sorting yields the same V2 fingerprint; consumers need no ordering logic | Proposed; current sorting path inspected |
| S02 — Change ports or bindings under a parent | Explicit parent-specific counts and item tags preserve the changed segmentation in the preimage | Proposed; ADR grammar inspected |
| S03 — Admit physically invalid or forbidden topology | Existing physical, membership, policy and cycle checks retain their outcomes and original identities | Implemented current checks; preservation required |
| S04 — Run tear selection or causal recycle | New graph key propagates into problem/composite identity; fresh native state and actual graph witnesses remain | Implemented consumer paths inspected; V2 integration untested |
| S05 — Read historical identity-bearing output | Historical V1 bytes retain their original meaning; current admission emits V2 without a V1 fallback | Proposed explicit evolution contract |

The review does not use S02 to claim that two physically admitted V1 graphs were shown to share an encoding. No such example was established. Removing that unresolved premise is a legitimate contract improvement without claiming a demonstrated collision.

For structural growth and skew, tags and counts add bounded incremental hashing alongside the existing traversal of necessary items. The proposal does not add graph copies, another retained index, persistence, locks, transactions or per-item tasks. No latency benefit is claimed.

## Architectural assessment

| Foundation | Judgment and reason |
|---|---|
| AP-01 | **Satisfied.** Identity primitives, physical projection and numerical consumers retain separate reasons for change. |
| AP-02 | **Satisfied.** The changed derivation has a new domain/version; historical meaning is preserved explicitly. |
| AP-03 | **Satisfied.** Documents, tears and recycle composition consume the admitted key through existing contracts. |
| AP-04 | **Satisfied.** Nodes, ports, original connections, bindings and tear groups remain meaningful, governed distinctions. The fingerprint's semantic limit is explicit. |
| AP-05 | **Satisfied.** Collection and parent structure becomes explicit; admission remains the enforcement boundary. |
| AP-06 | **Satisfied.** Encoding and admission can be exercised with their actual local physical inputs without starting storage or native execution. Consumer integration has separate focused checks. |
| AP-07 | **Satisfied.** Incremental framing follows the necessary input traversal and adds no materialization or lifecycle mechanism. |

The relevant heuristics are H1, H4, H15, H20 and H26: preserve physical behavior, implement only the framing gap, retain operation-shaped representations, distinguish fingerprints from independent assurance, and keep auxiliary identity work proportionate.

| Gate | Verdict and scope |
|---|---|
| G1 Authority | **Pass for the proposed design.** One frame declaration and one structural encoding owner; consumers do not duplicate encoding. |
| G2 Semantic fidelity | **Pass for the proposed design.** Explicit new version, parent boundaries and unchanged physical fields; no historical reinterpretation. |
| G3 Validity | **Pass for the proposed design.** Existing physical and topology admission remains required. |
| G4 Hidden behavior | **Pass.** Incremental hashing introduces no external effect or numerical policy. |
| G5 Consistency and recovery | **Pass within this change.** No new persistence or recovery protocol; successful admission still exposes the completed immutable graph. |
| G6 Transformation and reuse | **Pass for the proposed design.** Projection scope is explicit, full equality remains, and no sole-fingerprint reuse is introduced. |
| G7 Truthful capability claims | **Pass for the proposed design.** ADR-0167 disclaims demonstrated V1 collision, performance benefit and complete cross-registry equivalence. Public wording must follow that scope. |
| G8 Library leverage | **Pass.** Existing BLAKE3 and framed hashing supply generic capability; domain grammar remains owned code. |
| G9 Architectural fitness | **Pass.** All affected foundations are satisfied independently. |
| PS-G1 Physical consistency | **Pass for the proposed preservation contract.** Quantity admission and affine conversion remain unchanged. No physical behavior was requalified. |
| PS-G2 Well-posedness | **Pass for the affected topology distinction.** Flow topology, tear witness and equation structure remain distinct; no new equation well-posedness claim. |
| PS-G3 Numerical integrity | **Not applicable to encoding itself.** Inspected consumers retain fresh state and independent witnesses; solver convergence and scientific outcome adequacy are outside this review. |

## Findings and alternatives

No material defect requiring revision of ADR-0167's proposed design was found. No SHOULD exception is required.

Retaining V1 avoids identity churn but leaves the framing premise unresolved. Changing bytes under V1 would silently change historical interpretation and is unsuitable. Counts alone under a new version could provide a smaller valid grammar, but the proposed item tags make roles explicit with modest bounded work and no additional subsystem. Generic serialization or a new hash family adds unrelated compatibility or trust decisions.

The selected design is the simplest alternative that directly supplies the requested explicit grammar while preserving current ownership and identity policy.

## Verification and evidence limits

| Claim | Evidence | Acceptance still required |
|---|---|---|
| Existing hasher supplies length-prefixed tags, integers and digests | **Interface-checked** in `pse-ids` source | Frame catalog/spelling controls include V2 and retain V1 |
| Proposed grammar expresses collection and parent boundaries | **Proposed**, assessed from ADR-0167 | Independent golden preimage/digest and parent-segmentation controls |
| Canonical ordering and physical admission are preserved | Current paths **Implemented**, source inspected | Reordering, affine conversion, occurrence and policy regression checks |
| Direct consumers receive the graph key without new reuse | Current paths **Implemented**, source inspected | Document, native tear and recycle composite-identity checks |
| V1 has an admitted encoding collision | **Not established** | No collision claim is needed for this decision |
| V2 improves performance | **Not established** | No performance claim is made |

No product tests, compilation, benchmark or qualification campaign was run by this reviewer. Raw identifier encoding fixtures must be distinguished from physically admitted examples. Tests should challenge the grammar independently rather than compute every expected value by invoking the production encoder.

## Rule impacts and disposition

<a id="rc01"></a>

| ID | Current rule and location | Required evolution | If retained unchanged |
|---|---|---|---|
| RC01 | Current flow projection derivation uses FlowProjectionV1 in structural admission; frame catalog and blueprint §5.3/§15.3 describe identity/structural ownership | Introduce FlowProjectionV2, amend the current projection contract and retain historical V1 meaning | Current admission cannot realize the selected explicit V2 grammar |

RC01 is the bounded realization of the original graph/hash review's RC04, already selected by the maintainer on 2026-10-09. It introduces no additional operator decision. ADR-0167 and the owning plan carry the decision and implementation route; this review does not change ADR status or architecture text.

No durable fast-hash replacement, persistent compiler state, flow cache or numerical-policy rule change follows from this acceptance.

## Decision

**Accept the bounded FlowProjectionV2 target design at Proposed/Interface-checked evidence level.** Its framing, ownership and evolution contract satisfy the applicable architectural and semantic obligations.

Dependent implementation may proceed. Implementation acceptance remains with Plan 28k and its affected consumers: exact V2 grammar, preserved V1 meaning, truthful fingerprint scope, unchanged physical admission and witnesses, propagated document/native/recycle identities, and fresh runtime/native state. The original U01 inquiry is resolved only to the extent that these explicit framing obligations are implemented and verified; it does not become a claim of complete physical-registry equivalence.
