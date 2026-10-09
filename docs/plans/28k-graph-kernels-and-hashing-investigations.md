---
title: Graph kernels and hashing investigations
status: draft
date: 2026-10-09
adrs: []
review_sources: [docs/design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md]
scenario_sources: [docs/design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#scenarios-and-scientific-preservation]
---

# 28k: Graph kernels and hashing investigations

## Responsibility, target and baseline

Develop bounded decisions about additional graph preparation, fast hashing and their combined
reuse/invalidation role. The maintainer selected **bounded investigations** on 2026-10-09,
alongside planning the review's concrete corrections. This document owns GH investigation
progress and decision evidence; [Plan 28](28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration)
owns combined readiness and finding dispositions. [28e](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance)
owns affected assembled acceptance and authorized benefit measurement.

The [review](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md)
has a **Revise** verdict, three source-supported findings and a bounded framing premise U01.
[28j/J3](28j-pure-preparation-and-publication.md#j3-indexed-description-settlement),
[28b/B8](28b-selected-compilation-and-reuse.md#b8-retained-supplier-topology) and
[28f/N12–N13](28f-shared-numerical-preparation.md#graph-hash-flow-and-suppliers)
develop the corrections. None depends on fast-hash adoption or a persisted graph runtime.

Authoring baseline: `main` at `d0f2c41818a34539a910654dfea4760771603f45`, preserving the
dirty tooling/documentation tree and untracked references. The selected standard is Core 3.4,
Heuristics 1.0, ProcessSimulator 1.5 and the pse-arrow binding. This is a **Proposed** plan;
investigations and correction implementations have not been executed by its creation.
The review's interface checks and historical product receipts keep their original scope.

Existing exact basis keys, Salsa equality cutoff, factorable hash-consing, dense node IDs,
portable descriptions, allocation owners and current effect settlement are reusable foundations.
Study admission already retains prerequisites; a whole graph rebuild per dispatch was not
established. Native mutable state and scientific assessment remain private/current. New graph
or hash machinery must remove work at an identified operation or serve a concrete capability.

## Investigation contracts and decisions

Start with current source, then exact-release capability evidence and current primary
documentation where an API decision needs research. Inspect consumers needed to settle the
question; do not create an exhaustive inventory or new proof machinery. Use a selective probe
only when a material premise or alternative cannot be settled by source reasoning. Compare
construction, necessary traversal/checks, current effects, retained state and recovery together.
Numeric performance claims require measurements under named conditions.

<a id="gh1"></a>

### GH1 — Graph preparation and placement

Identify the repeated operation and its semantic edge meaning before choosing a graph kernel.
Compare existing functions/prepared views with compact adjacency/CSR, existing petgraph
algorithms, relational bulk operations and selective persisted SurrealDB access. Examine
cross-call flow preparation only against actual repeat consumers and key/validation costs;
one build per public operation is not itself a defect. B8's admitted-product retention is
the first supplier correction; cross-revision sharing is a separate proposal.

Preserve isolates, stable diagnostics and parallel physical occurrences. CSR's unique-endpoint
representation requires an explicit consumer projection and a correspondence back to
occurrences; do not silently flatten a flowsheet. SCC organization must preserve directed
internal edges, labels and merge/split semantics. Keep topology, incidence, supplier dependency
and study prerequisites distinct. A new analysis or supplier kind should extend its meaning
owner and consume shared mechanics without copying numerical policy.

For database placement, identify selective persisted reads/joins actually removed from the
operation. Require complete bounded closure, coherent membership/revisions and inspected inner
query/write errors. Include transport, short transactions and whole-decision retry; graph syntax
does not supply numerical execution or scientific validity. Retain useful relational optimizer
visibility rather than forcing every operation through a neutral graph interface.

**Decision output:** named operation, consumed graph meaning, simplest adequate realization,
integration/retention owner, migration scope and evidence that distinguishes alternatives.
Retain the existing realization where no useful repeat/placement premise is established.
A universal task/operation compiler is not selected; reconsider only for concrete consumers
whose execution/composition contracts cannot be supplied by the bounded kernels.

<a id="gh2"></a>

### GH2 — Fast hashing independently of graph adoption

Separate map bucket selection, interning, change fingerprints, durable identity, integrity
and attestation. Inspect actual key sizes and key construction/equality/lookup work at selected
hot candidates, starting with factorable interning and preparation lookup. Dense IDs, ordered
access and current equality may remove more machinery than changing a digest.

Compare std table hashing with suitable rustc-hash/foldhash/XXH map capabilities and explicit
XXH3-128 fingerprints against current BLAKE3 only for their intended roles. The review reports
twox-hash 2.1.5 locked transitively for XXH32/64 and cached xxhash-rust 0.8.19 outside the lock;
neither fact enables XXH3 in the product. Verify actual selected features and exact APIs if
adoption is proposed. Preserve the no-`target-cpu=native` numerical constraint; distinguish
compile-time SIMD from runtime dispatch and one-shot from streaming/storage costs.

Table hashes remain bucket selectors with full key equality and explicit observable order.
Fingerprint proposals specify canonical bytes, domain/version, collection boundaries, roles,
order and numeric encoding, including exact rationals, signed zero and the current NaN policy.
Request a 128-bit result explicitly rather than interpreting `Hasher::finish()` as 128 bits.
An intentionally colliding candidate must not merge unequal meaning. Test exact equality or
other explicit collision-confirmation paths independently of the proposed digest.

**Decision output:** selected role/site, candidate versus current/ordered/dense alternatives,
complete integration cost and supported benefit premise. Keep table hashing separate from
sealed semantic identity equality. A local hasher proposal can preserve durable bytes;
durable replacement remains eligible but requires its own trust, collision, reference and
reconstruction/migration design. No blanket replacement or speed multiplier is inferred.

<a id="gh3"></a>

### GH3 — Complete dependencies and finer invalidation

Consume [28f/N0's catalog-invalidation investigation](28f-shared-numerical-preparation.md#library-and-lifecycle-decisions)
rather than create another decision owner for the same question. GH3 supplies the graph/hash
comparison and links its conclusion back to N0. Inspect actual upstream admission work for
unrelated/selected edits and edit/revert; selected equality cutoff does not prove upstream
checking was avoided. Prefer extending current Salsa inputs/dependency domains over a manually
maintained parallel graph when both represent the same meaning.

Specify positive and negative lookups, membership, deletion, provider/policy/context changes,
structural literals, demand and interpretation actually consumed. Preserve current spans and
attribution even when equal mathematical output stops propagation. Compare conservative complete
domains with sound narrower ones; no positive-edge list or global content multiset establishes
completeness. For cycles, distinguish supplier rejection from legitimate physical recycles;
include internal-edge edits and SCC merge/split behavior.

**Decision output:** the current completeness authority, useful narrowing/equality boundary,
upstream work removed, failure/invalidation scenarios and dependent consumers. A narrowing
proposal without a complete negative/membership argument remains blocked at that boundary;
existing exact reuse and F01–F03 corrections continue independently.

<a id="gh4"></a>

### GH4 — Flow framing and persistent pure reuse

First settle Graph/hash U01's admitted identity universe and actual fingerprint consumers.
`FlowGraph` full equality currently compares declaration/bindings; the abstract segmentation
example in the review is not a demonstrated authored/compiler-supported failure. Inspect
supported construction and identity roles before a bounded counterexample. Do not invent
unreachable raw IDs as evidence of scientific misreuse. If a supported counterexample exists,
record the revealed defect and route its changed interpretation explicitly. Otherwise state
the established premise or exact remaining gap; no sole-key equivalence claim follows by default.

Before combined/persistent adoption, compare existing canonical portable products with Salsa's
feature-gated persistence and content-addressed graph products. Identify the expensive pure
product actually reconstructed, serialization/compatibility and complete transitive dependencies.
Typed handles, accumulators, eviction/GC, process generations and restoration behavior require
exact-release evidence. Mutable evaluators/factors, native pointers, attempts and publication
permission are not persistent pure products.

Topology-sensitive fingerprints must include the relevant directed edges, identities/roles,
labels and scope. Sorted reachable-content multisets or WL/SCC member labels are not exact graph
equality. Preserve current receiving qualification, corruption refusal, producer interpretation,
fresh attribution and exact settlement after restart. Include store/query truncation, interrupted
publication, stale completion, released roots and reconstruction failures.

**Decision output:** U01 premise/counterexample, candidate pure product and completeness contract,
selected persistent realization or reason to retain current portable reconstruction, compatibility
and receiving/recovery obligations, and any contingent rule changes. Without a concrete useful
product and complete compatibility/dependency argument, do not schedule a persisted runtime.

## Rule-change boundary

**Selected rule changes: none (2026-10-09).** J3/B8/N12/N13 preserve current contracts;
this document plans investigations rather than adopting their alternatives. The review's
conditional notes retain their original IDs and are not accepted or rejected by plan creation.

| Review note | Current planning treatment | Adoption route if selected; consequence if current rule stays |
|---|---|---|
| [RC01](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc01) finer domains | GH3/N0 tests completeness within blueprint §14.4. | Narrower complete domains already fit the contract. Version any changed durable interpretation; conservative domains still permit the corrections. |
| [RC02](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc02) durable hashes | GH2/GH4 assess benefit, equality/trust and references. | Explicit operator decision, hashing ADR/design route and new interpretation/migration before dependent implementation. Retaining current identity permits local map/fingerprint options. |
| [RC03](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc03) persisted runtime | GH4 compares a concrete consumer with portable reconstruction. | Explicit operator decision and blueprint §14.4/§20 ownership/compatibility route before dependent work. Current portable products remain the durable reuse mechanism. |
| [RC04](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc04) flow framing | GH4 settles U01 before sole-fingerprint use. | Explicit operator decision, hashing ADR/design route and new frame/version; never reinterpret FlowProjectionV1 bytes. Keeping it does not block N12. |

If an investigation reveals another actual rule change, present its consequence for explicit
accept/reject before fixing the dependent target. Record confirmed decisions and their dates/routes
at the coordinator, link them here, and adapt rejected alternatives. A new pass/kernel contract
within an existing decision uses the repository's short-ADR route; local refactors do not.
Exact dependency additions/feature choices are ordinary implementation decisions, separately
qualified at their consumers. Licenses are not technical rejection reasons.

## Packages, evidence and completion

| Package | Input and readiness | Completion product | Progress |
|---|---|---|---|
| GH1 | Review and current graph consumers; B8 design available, working B8 needed only to compare its actual retained realization. | Graph/layout/placement decision at named operations, consumer and lifetime constraints. | Proposed; investigation not run. |
| GH2 | Current identity/equality owners, representative keys and exact library contracts. | Role-specific hash or retained-design decision; encoding/collision obligations and measurement limits. | Proposed; investigation not run. |
| GH3 | N0 catalog evidence plus exact selected dependency ownership. | Linked N0 conclusion and complete graph/hash invalidation comparison. | Proposed; investigation not run. |
| GH4 | U01 supported identity/consumer analysis first; GH2/GH3 completeness/encoding decisions before a combined adoption proposal. | Framing premise and persistent-product decision, including required rule routes. | Proposed; investigation not run. |

GH1/GH2 and GH3's source inquiry may proceed independently. U01 source analysis is an early
GH4 slice; persistent adoption depends on the complete meaning supplied by GH2/GH3. J3/N12
do not wait for investigations, and N13 waits only for working B8. Shared declarations/manifests
and compiler/runtime edits have one integration owner even where logical work is independent.

**Proposed verification:** choose small independently specified cases from the review's S01–S08:
collision/reorder/numeric encoding, negative and membership changes, internal-edge/SCC edits,
selected unrequested cycles, eviction/clear/escaped aliases, concurrent independent occurrences,
restart and stale completion. Use existing controls where they expose the premise; add a
bounded probe only for a missing material argument. Compare cold preparation, warm selection,
retained state and recovery as appropriate, not bulk digest throughput alone. Any quantitative
claim names actual source/artifact/profile, mode, key/workload shape, command and zero-failure
baseline; no performance threshold or benchmark campaign is mandatory merely to complete a decision.

Each question ends in a supported retained-design decision, a concrete adoption proposal,
or an unresolved premise with an owner, settling evidence and blocked dependent work. A proposal
names required contracts, migration/deletion and acceptance at the existing companion. It is
not implemented by its publication. If a decision is deferred, specify an observable trigger,
check and owner and use the existing ADR register rather than a second backlog. Investigation
completion does not resolve the coordinator's findings or restart E4/E5.

## Checkpoint

The documents and investigation boundaries are prepared. No GH inquiry, correction package,
hash dependency change or persistence adoption is claimed complete. Source-supported review
conclusions are carried forward; source inspection during authoring preserved selection-bound
failure semantics and existing allocation/effect owners. Next, when execution scope is selected,
start J3/N12 or B8 and the independent GH source inquiries under the coordinator's chosen route.
The original narrowed qualification continuation and its historical Outcomes remain intact.
