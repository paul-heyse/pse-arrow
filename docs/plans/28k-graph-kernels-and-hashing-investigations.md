---
title: Graph kernels and hashing investigations
status: in-progress
date: 2026-10-09
adrs: [ADR-0167]
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
Heuristics 1.0, ProcessSimulator 1.5 and the pse-arrow binding. At creation this was a
**Proposed** plan; neither investigations nor correction implementations had executed.
The review's interface checks and historical product receipts keep their original scope.

Execution refreshed a clean `main` at `46545b2ad3e2999b4335692ac49af6091438c3fc` on
2026-10-09. The maintainer authorized GH1–GH4 investigations, using source-led inquiry and
selective probes. The decisions below completed that original scope; the authorized followup section
records the subsequently selected J3/B8/N12/N13 changes and RC04. E4/E5 retain their owners. The authoring paragraph above records the original
baseline rather than current progress.

Existing exact basis keys, Salsa equality cutoff, factorable hash-consing, dense node IDs,
portable descriptions, allocation owners and current effect settlement are reusable foundations.
Study admission already retains prerequisites; a whole graph rebuild per dispatch was not
established. Native mutable state and scientific assessment remain private/current. New graph
or hash machinery must remove work at an identified operation or serve a concrete capability.

## Investigation contracts and decisions

**Later ownership handoff, 2026-10-09:** the inquiry conclusions below retain their original
scope and conditions. [Plan 33](33-efficiency-principles-remediation.md) independently owns
the later efficiency review and its prospective work: EFF06 numerical facts, EI04 typed-checker
decision/integration and EI05 further graph/hash/pure-persistence decisions. Earlier N0/GH
owner references describe that inquiry's handoff, not a second current work owner. No old
timing or internal numerical output is an acceptance baseline for the new work.

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

**Decision, 2026-10-09 — retain bounded graph owners; use B8's retained supplier topology.**
`AdmittedModeling::implicit_order_for` reconstructs a dependency graph and derives value-level
provider demands from the admitted implicit inventory on each call. Runtime case resolution
registers at Value before binding and can register again after binding at a higher derivative
order; observations also request supplier ordering. This establishes repeated discovery of
immutable topology, independently of a timing claim. B8 should retain compiler-owned adjacency
and the identity-to-node mapping using the existing petgraph mechanics first. N13 consumes it
with fresh demand, hints, derivative capabilities and private registration state. Selected
root lookup stays selection-local, closure precedes selected cycle rejection, and existing
inventory-wide body-demand failures remain observable. Isolates and full-scope case obligations
remain. Charge retained adjacency/index payload and escaped aliases to the admitted owner.

The other inspected operations do not justify a new graph realization. `prepare_modeling_flow`
builds once per invocation and `PreparedFlow` retains the graph; its modeling and conditional
recycle callers do not establish repeated preparation within one operation. Flow connections
are distinct physical occurrences, including parallel edges. A CSR projection would add
occurrence correspondence and isolate obligations without an established useful consumer.
Ephemeral study admission retains prerequisites for scheduling. Durable `study_scope` obtains
current immediate predecessor records, bounded to 64, and claim-time revision/generation
fencing remains necessary even with an in-memory graph. One-shot analysis graphs likewise do
not establish a reusable execution kernel. Retain these owners and selective persisted reads;
neither database graph syntax nor a universal graph interface removes their current duties.

Source: compiler [supplier ordering](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-compiler/src/workspace/modeling/executable/implicit.rs)
(`implicit_order_for`); runtime [case registration](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-runtime/src/workflow/modeling/cases.rs)
(`resolve_case`, `finish_case`), [flow ownership](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-runtime/src/math/flows.rs)
(`prepare_modeling_flow`), and operations [durable studies](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-operations/src/canonical_studies.rs)
(`study_scope`). Cross-call flow sharing or a different layout can be reconsidered by the
respective preparation owner when an actual repeated consumer identifies work removed after
keying, admission and retention; no such implementation is scheduled here.

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

**Decision, 2026-10-09 — retain current hashing; identify a local table-policy candidate.**
The factorable `Builder.interned` is a viable site for a future
`HashMap<Node, NodeId, rustc_hash::FxBuildHasher>` comparison. Full `Node` equality would remain
the collision confirmation; IDs derive from `nodes.len()` and no interner iteration determines
observable order. Keys include variable-length operand vectors and arbitrary-precision rational
constants, as well as small ordinal nodes. Folding, operand construction, hashing/equality,
miss cloning and allocation would remain. Existing dense node storage serves ID lookup, not
the reverse structural lookup. An ordered map adds a total-order contract and comparisons
without a sorting consumer. `foldhash::fast::RandomState` is another technically viable table
policy; Fx's fixed policy changes collision-amplification exposure. No measured interner cost
or end-to-end benefit supports adoption now. Reopen at the factorable owner only when an
attributable preparation profile makes table work material, then compare actual structured
keys and retain an independently forced-collision equality control.
J3's indexed description settlement and N12's sorted policy lookup remove identified discovery
work independently; neither needs a hash-policy change.

`BasisKey` hashes and compares request, complete selected dependencies, root, instance,
bindings and limits by value; `Arc` does not make those traversals pointer-only. A retained
prefilter could avoid later hash traversal but still needs first construction and exact
equality. `BodyInput` instead defines equality by its sealed 256-bit `SemanticBodyHash`:
changing that digest changes the identity contract, including portable lookup. It is not a
local bucket-policy optimization. Retain framed BLAKE3 for current semantic identities,
references and integrity roles. No useful explicit XXH3-128 consumer was established beyond
an additional prefilter requiring its own canonical encoding and collision confirmation.

Factorable float equality preserves bits and signed zero; supported constants are finite.
Its durable rational encoding uses normalized numerator/positive-denominator strings. General
`FramedHasher::f64` separately canonicalizes NaN payloads while preserving signed zero. A future
fingerprint must follow its actual consumer's convention rather than invent one shared policy.
Topology fingerprints must also retain directed edges, roles, labels and collection boundaries;
the reference's sorted reachable-content multiset cannot establish topology equality.

Exact-release inspection and the read-only feature inventory established Salsa 0.28.4 with
`inventory,macros,salsa_unstable`, rustc-hash 2.1.3 with `default,std`, foldhash 0.2.0 with
`default,std`, and twox-hash 2.1.5 with only `xxhash32,xxhash64`. Cached xxhash-rust 0.8.19 is
absent from the lockfile. Neither existing twox feature enables XXH3. xxhash-rust's
`xxh3_128`/`Xxh3::digest128` return 128 bits, while `Hasher::finish` returns 64; twox's
128-bit type supplies `oneshot`/`finish_128` and does not implement `std::hash::Hasher`.
xxhash-rust SIMD selection is compile-time; corresponding twox features can enable runtime
dispatch. Neither requires changing the no-`target-cpu=native` rule.

Source: [factorable interning](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-math/src/factorable.rs)
(`Node`, `Constant`, `Builder::push`), [basis keys](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-runtime/src/math/preparation.rs)
(`BasisKey`, `basis_key`), [sealed body inputs](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-compiler/src/workspace/modeling/executable/grouped.rs)
(`BodyInput`, `plan`), and [framing](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-ids/src/derive.rs).
Library API details were checked against the named cached releases after Context7 research;
no dependency or feature was changed.

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

**Decision, 2026-10-09 — retain complete canonical premises and Salsa equality cutoff.**
[N0](28f-shared-numerical-preparation.md#library-and-lifecycle-decisions) owns the adopted
catalog conclusion. Canonical `SelectedDependencies` records positive and absent name/logical
lookups, complete scope/kind/reference membership and consumed interpretation identities.
`recheck_dependencies` qualifies those premises under a new protected selection and merges
the acquired delta only on success. Selected preparation additionally binds structural inputs,
physical/provider context and demand at the products that consume them (blueprint §14.4).
An edge-only supplier graph or global content multiset cannot replace that authority.

The useful boundary is already admitted selection versus changed authored publication.
`publish_modeling_with` returns immediately for exactly unchanged rows/scope/documents;
otherwise it invokes `check_modeling`/`pse_modeling::check_with` before publishing to Salsa.
Thus unrelated authored edits and edit/revert can still repeat whole-package admission even
when downstream equal bodies are reused. `publish_modeling_revision` consumes an already
admitted revision without that check. These are different paths: downstream execution events
alone would miss the upstream checker, and the authored result does not prove every warm
canonical study recompiles its package.

Narrowing only `Catalog.checked` or using a faster digest does not eliminate that upstream
work. A future incremental authored checker would need typed dependency domains covering
negative resolution, membership additions/deletions, visibility, provider/policy/physical
interpretation, structural literals and actual demand, with current diagnostics and atomic
failed-publication behavior. Extend current semantic owners/Salsa inputs if that consumer is
established; do not add a parallel manually maintained dependency graph. Current complete
checking is retained. N0's reopen condition is a repeated authored-edit workload with a material
admission share, followed by a complete checker-domain design; no narrower checker is scheduled.

Source: [publication and checking](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-compiler/src/workspace/modeling.rs)
(`publish_modeling_with`, `check_modeling`, `publish_modeling_revision`) and
[canonical premises](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-operations/src/canonical_selection.rs)
(`SelectedDependencies`, `recheck_dependencies`). Existing compiler controls
`kernel_queries_reuse_math_across_values_unrelated_edits_and_clean_rebuild`,
`kernel_publication_rejects_invalid_batch_atomically` and
`complete_body_queries_reuse_unrelated_math_and_refresh_revision_attribution` were read,
not rerun. Their scenarios explain existing coverage; this inquiry claims no new test result.

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

**U01 result, 2026-10-09 — bounded unresolved encoding premise, with dependent work identified.**
The compiler's flow nodes are actual instance IDs, ports are admitted member IDs, and quantities
and units come from the admitted physical context. Root instances reuse declaration IDs;
nested instances and members use the modeling-member derivation. Typed wrappers therefore do
not by themselves prove disjoint byte roles. FlowProjectionV1 emits a node ID followed by
port/quantity/unit IDs without per-node collection framing. Source inspection did not establish
an authored/compiler-supported pair of distinct admitted declarations with identical streams.
No raw-ID construction is promoted to such evidence.

Inspected consumers are `PreparedFlow::document`, native tear problem identity/assumptions and
tear-session compatibility. `FlowGraph` equality also compares declaration and physical bindings.
Runtime tear selection requires Fresh/NoPriorStart, creates a fresh native session and verifies
the resulting tear witness against the actual graph. Retention associates the graph's allocation,
not a semantic-key cache entry. These facts do not establish scientific misreuse, nor do they
prove injective framing for every supported flow. **The identity/structural owners, with the
compiler flow owner, must settle the remaining admitted-role/segmentation argument before any
sole-fingerprint graph equivalence or cross-call semantic cache is implemented.** Settling
evidence is either a compiler-supported equal-stream counterexample with affected consumers,
or an encoding/identity-universe argument covering node/port grouping and all collections.
If changed framing is selected, RC04 requires a new version and decision route; N12 proceeds
independently. The coordinator retains U01 as an open premise, not a fourth demonstrated defect.

Source: [flow admission/equality](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-structural/src/flowsheet.rs),
[compiler flow construction](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-compiler/src/workspace/modeling/flow.rs),
[instance/member derivation](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-modeling/src/specialize.rs)
(`root_instance`, `member_id`), and [native tears](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-backend-native/src/tears.rs)
(`compile`, `solve`, `recover`).

**Persistence decision, 2026-10-09 — retain qualified portable-body reconstruction.**
The concrete pure candidate is admitted mathematical body reconstruction, already served by
`reuse_body`: protected canonical eligibility, receiving/producer qualification, requested
identity and physical context, envelope/specification and sealed reconstructed identity are
checked before allocation-owned use. Corrupt/incomplete hits refuse. Current effects and fresh
mutable evaluators remain separate. No material uncovered reconstruction cost was established
that warrants another persisted runtime.

Salsa persistence is a technically eligible alternative accelerator, currently disabled.
Exact source supports persistence of keys/results and transitive edges, but all consumed input
leaves must persist. `semantic_body(Inventory, BodyKey)` includes expression/provider/context
inputs and returns library-owned `AdmittedBody` or errors; those representations do not become
persistable by adding an attribute. The existing portable payload is an explicit conversion
boundary. A preparation frontier also contains specialized models and products; its runtime
wrapper adds solved state, generations and allocation owners. Select a pure representation
before considering restoration, and reissue process owners and current attribution.

Persisted ingredient indices do not establish cross-build schema compatibility. Interned IDs
have generations and GC/liveness obligations; they are not durable semantic references.
Memo serialization does not automatically restore diagnostic accumulators. The inspected
workspace has no such accumulator consumer, so that is a future integration obligation rather
than a current defect. Eviction can recompute, and workspace rebuild discards memos and advances
generation. Restoration must occur into disposable fresh state with versioned interpretation,
corruption refusal, complete receiving qualification, roots/allocation admission and stale-fill
fencing before publication. Mutable/native state and publication permission remain excluded.

Runtime/procedural macros resolve to Salsa 0.28.4, but locked `salsa-macro-rules` is 0.28.5.
Locked macro/source inspection confirmed persistence machinery; shared skill roundtrips at
all-0.28.4 remain historical interface evidence, not consumer tests of this resolution.
Reopen at the compiler/runtime portable-product owners only for a useful measured pure-product
reconstruction cost and a complete dependency/compatibility representation. Qualify restart,
GC/eviction, stale completion and receiving refusal at that consumer before any adoption.
RC03 and any changed identity interpretation retain their explicit decision routes.

Source: [portable receiving/reconstruction](https://github.com/paul-heyse/pse-arrow/blob/46545b2ad3e2999b4335692ac49af6091438c3fc/crates/pse-runtime/src/math/portable.rs)
(`reuse_body`, `reuse_body_current`), compiler body queries above, and exact cached Salsa
0.28.4 `input/input_field.rs`, `function/memo.rs`, `database.rs`, `zalsa.rs` plus locked
salsa-macro-rules 0.28.5 setup macros. These local release paths identify inspected library
source; they are not repository files or new persistence qualification.

## Rule-change boundary

**Selected rule changes: none (2026-10-09).** J3/B8/N12/N13 preserve current contracts;
the completed investigations retain those contracts. The review's
conditional notes retain their original IDs and are not accepted or rejected by plan creation.

| Review note | Current planning treatment | Adoption route if selected; consequence if current rule stays |
|---|---|---|
| [RC01](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc01) finer domains | GH3/N0 tests completeness within blueprint §14.4. | Narrower complete domains already fit the contract. Version any changed durable interpretation; conservative domains still permit the corrections. |
| [RC02](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc02) durable hashes | GH2/GH4 assess benefit, equality/trust and references. | Explicit operator decision, hashing ADR/design route and new interpretation/migration before dependent implementation. Retaining current identity permits local map/fingerprint options. |
| [RC03](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc03) persisted runtime | GH4 compares a concrete consumer with portable reconstruction. | Explicit operator decision and blueprint §14.4/§20 ownership/compatibility route before dependent work. Current portable products remain the durable reuse mechanism. |
| [RC04](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#rc04) flow framing | GH4 settles U01 before sole-fingerprint use. | Accepted by maintainer 2026-10-09; ADR-0167 and formal scoped review precede new V2 implementation. Never reinterpret FlowProjectionV1 bytes; N12 remains independent. |

If an investigation reveals another actual rule change, present its consequence for explicit
accept/reject before fixing the dependent target. Record confirmed decisions and their dates/routes
at the coordinator, link them here, and adapt rejected alternatives. A new pass/kernel contract
within an existing decision uses the repository's short-ADR route; local refactors do not.
Exact dependency additions/feature choices are ordinary implementation decisions, separately
qualified at their consumers. Licenses are not technical rejection reasons.

## Packages, evidence and completion

| Package | Input and readiness | Completion product | Progress |
|---|---|---|---|
| GH1 | Current named graph consumers and B8 design. | Retain current flow/study/analysis owners; B8 petgraph supplier retention has a concrete repeated-work premise. | Completed; B8/N13 followups implemented with scoped evidence below. |
| GH2 | Actual interner/basis/body keys and exact library sources/features. | Retain hashing; local Fx/foldhash comparison has a bounded reopen condition, separate from durable identity. | Completed source/interface inquiry; no adoption or measurement. |
| GH3 | N0 catalog owner plus canonical dependency and checker paths. | Retain complete domains/backdating; identify upstream authored checking and prerequisites for narrowing. | Completed; conclusion integrated at N0. |
| GH4 | Supported flow construction/consumers and GH2/GH3 boundaries. | U01 gap has owner/settling evidence/blocked consumers; retain qualified portable reconstruction. | Completed bounded inquiry; U01 remains an open premise. |

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

## Investigation verification — 2026-10-09

**Interface-checked:** exact cached library source and resolved features were inspected after
Context7 resolve/query research for Salsa, rustc-hash and xxhash-rust. The read-only command
`scripts/pse-env -- cargo tree -e normal -p salsa -p twox-hash -p rustc-hash@2.1.3 -p foldhash@0.2.0 --depth 0 -f '{p} features={f}'`
completed successfully with zero command failures against a zero-failure baseline. Its output
supports resolution/features only. Product tests, new probes and benchmarks: **not_run**;
source settled the retained-design choices, and U01's exact gap remains explicit. No performance
or enclosing product qualification is claimed. Documentation checks are recorded in the Outcome.

Research used Context7 `/websites/rs_salsa` for database serialization/deserialization,
`/rust-lang/rustc-hash` for small structured-key table hashing, and `/doumanash/xxhash-rust`
for explicit 128-bit APIs and SIMD selection. Current documentation was a discovery aid;
the exact releases governed applicability. Primary references include
[Salsa 0.28.4 database source](https://docs.rs/salsa/0.28.4/src/salsa/database.rs.html),
[rustc-hash 2.1.3 API](https://docs.rs/rustc-hash/2.1.3/rustc_hash/), and
[xxhash-rust 0.8.19 source](https://docs.rs/crate/xxhash-rust/0.8.19/source/src/xxh3.rs).

## Authorized followup execution — 2026-10-09

The maintainer authorized graph/hash corrections and their direct Plan 28 architectural
intersections, bounded measured inquiries, and Graph/hash RC04 (FlowProjectionV2). This
extends the completed GH1–GH4 inquiry below; it does not select RC02 durable hash replacement,
RC03 persisted incremental runtime, or full unrelated Plan 28 closure.

Execution proceeds through retained sorted portable inventories and exact description access
(J3), success-only shared supplier topology at the existing fallible registration boundary
(B8/N13), and sorted flow-policy lookup (N12). The topology is derived, excluded from semantic
equality, conservatively charged before admission and shared across owner attachments;
selection, cycle checks, derivative demands and numerical state remain current.

[ADR-0167](../adr/0167-frame-flow-projection-v2.md) and its independent formal review precede
V2 implementation. V2 uses a new domain with collection/item tags and all parent collection
counts, retaining canonical values, BLAKE3 and full graph/context equality. V1 keeps its
historical meaning without a current fallback. Direct acceptance covers document fingerprints,
native tears and recycle causal identity. V2 resolves byte-role segmentation only; it does not
turn the graph key into complete physical-registry equivalence.

The bounded inquiries exercise actual factorable projection/interner work and preparation key
cost, source admission versus downstream invalidation, matched portable reconstruction costs,
and an opt-in real Recipe persistence probe under the locked Salsa family. A local hash change
preserves exact equality and is adopted unless it fails correctness or clearly and significantly
impairs performance. The maintainer selected this criterion on 2026-10-09, recognizing noise
from concurrent repository workloads and the value of later architectural integration. Small
gains, inconclusive timing, or slight impairments do not exclude a technically supported
optimization. This supersedes the earlier requirement for benefit beyond observed variation.
Persisted query decode reuse is distinct from mathematical reconstruction.
A consequential contract adoption remains a concrete decision with its own completeness,
compatibility, receiving qualification, ownership and deletion obligations.

The selected local hashing changes are Fx bucket selection for factorable node interning,
one structure-key computation per factorable projection, and a retained Fx table prehash for
the immutable preparation basis key. Full structural/key equality remains authoritative;
none changes durable BLAKE3 identity or substitutes hashing for protected reads and settlement.
The earlier GH2 retained-hashing decision above records the original inquiry outcome and is
superseded for these three bounded sites. Reuse across additional calls, upstream incremental
checking and persistent query execution remain integration questions with their existing owners,
rather than benefits claimed from these local changes alone.

Targeted functional checks run with each correction; affected native/Python journeys and
scope-end checks establish only this followup scope. Plan 28e retains assembled acceptance and
the paused campaign. Current disposition stays at the coordinator until correction evidence
supports closure. The historical inquiry Verification and Outcome below retain their original
scope and date.

## Checkpoint

Current execution handoff, 2026-10-10: GH1–GH4 and selected followups mechanisms are implemented;
scoped evidence retains its original conditions and does not close Plan 28.

Retained supplier topology, fresh registrations, sorted flow-policy lookup, FlowProjectionV2 and local interner/prehash/key reuse are implemented with scoped acceptance in the followup Outcome. Plan 33/EFF06 completed the actual numerical-fact correction and fresh PC-SAFT preparation smoke; EI04 retains complete authored checking. Those are completed decisions, not new investigation backlogs. RC02 durable hash replacement and RC03 persisted runtime remain unselected.

The maintainer authorized all remaining Plan 28 scope including the full campaign.
The [coordinator](28-surrealdb-unified-substrate.md#current-checkpoint) owns finding
dispositions and [28e](28e-rebuild-retirement-and-qualification.md#checkpoint-and-next-step)
owns current integration, E3/E4/E5 and retirement. Earlier pause/failure-only instructions
are superseded. Next: complete bounded campaign preparation and required consumer
reconciliation, then assembled acceptance; do not restart these implemented mechanisms
from historical status prose.

## Investigation Outcome — 2026-10-09

**Implemented — what was built:** completed investigation decisions and source/interface
evidence, integrated into the existing coordinator, N0 and correction owners. Production APIs,
schemas, hashes, dependencies and numerical behavior were not changed. No product-test or
measurement claim follows from these documents.

**Tested — documentation only:** `just docs` passed, publishing 290 chapters with scoped
search. A focused `scripts/pse-env -- python` HTML link/fragment check covered the six changed
pages: 381 local targets checked, zero failures against a zero baseline. Its first run found
one existing plans-index link to unpublished `.codex/skills/README.md`; changing that source
reference to a repository link and rebuilding removed the failure. External URLs were not
availability-tested. No integrated product, native or scientific campaign ran.

**A mistake made and corrected:** the initial flow-key consumer search found document export
but missed native tear identity/assumption and compatibility fields. Following `graph.key()`
into the native backend corrected the consumer account. Fresh-session construction and the
runtime's Fresh/NoPriorStart requirement still support the bounded conclusion; the key was
not dismissed as merely display metadata.

**Deliberate deviations:** no benchmark matrix or runtime probe was needed to retain the
current designs. U01 completed with a precise unresolved premise rather than a fabricated
raw-ID counterexample. No correction package or paused campaign was implemented as part of
investigation completion.

<a id="followup-verification-and-outcome-2026-10-09"></a>

## Followup verification and outcome — 2026-10-09

**Implemented:** J3 retains canonical primary/original portable-body inventories and uses
sorted exact description lookup while preserving current protected settlement. B8/N13 retain
success-only, shared supplier discovery at the existing fallible boundary; selected closure,
cycle checks, demanded outputs/derivatives, starts and private evaluators remain current.
N12 uses binary search over sorted unique tear decisions. FlowProjectionV2 explicitly frames
collections and parent boundaries under proposed ADR-0167; full graph/context equality remains
necessary. The replaced per-use discovery, nested lookup and policy scan are removed.

**Implemented:** local factorable interning uses rustc-hash 2.1.3 with complete `Node`
equality, each projection computes its structure key once, and immutable preparation keys
retain a table prehash with complete key equality. Dependencies remain exactly pinned; the
lockfile adds only the compiler's once_cell edge and math/runtime rustc-hash edges. No durable
digest or semantic-equality contract changes. Temporary inquiry features, capture hooks,
included probe modules and benchmark branches are removed from production. Replay sources and
diagnostic results live in the [bounded evidence directory](../design_review/evidence/graph-hash-followups-2026-10-09/README.md).

**Tested:** targeted compiler controls passed 6/6, structural flow controls 6/6, identity
controls 61/61 and backend tear controls 3/3. All used locked dependencies, pinned tools,
explicit force-validation and a zero-failure baseline:

```bash
just unit-package pse-compiler 'test(owned_frontier_inventory) | test(selected_shared_observation_body_demands_only_its_implicit_provider_output) | test(supplied_promoted_implicit_descriptors_precede_nested_consumer_planning) | test(supplier_topology_)'
just unit-package pse-structural flowsheet --profile local --status-level pass --final-status-level pass
just test-package pse-ids -p pse-relations -E 'package(pse-ids)' --profile local --status-level pass --final-status-level pass
just unit-package pse-backend-native tears --profile local --status-level pass --final-status-level pass
```

**Tested:** after the maintainer cleared target artifacts, the native worker and editable
Python extension were rebuilt from the permanent implementation. Final factorable controls
passed 47/47 and runtime controls 24/24 in the native/canonical environment with force-validation
and Nextest's local profile. The factorable selection includes the runtime demanded-callback
consumer. Runtime controls cover exact-basis attribution/released roots, many-body indexed
settlement, deliberately colliding preparation prehashes, portable reconstruction, nested
suppliers, flow handling and authored causal recycle. Exact commands and conditions are in
the [hashing acceptance report](../design_review/evidence/graph-hash-followups-2026-10-09/hashing-results.md#permanent-implementation-acceptance).
These checks had zero failures against the zero baseline.

**Tested:** the four selected native Python journeys passed 4/4 with Python 3.14.7,
pytest 9.1.1, four xdist workers and canonical-owner grouping, against zero failures:

```bash
just native-python build/graph-hash-followups-20261009-python-final python/pse/tests/test_modeling_strategies.py::test_authored_recycle_uses_declared_ports_and_owned_results python/pse/tests/test_modeling_run.py::test_authored_solve_join_warm_start_checks_and_retention python/pse/tests/test_modeling_kernel.py::test_complete_preparation_policy_controls_public_preparation_and_execution python/pse/tests/test_modeling_kernel.py::test_conformance_preparation_policy_reaches_native_and_pure_compilers
```

**Tested:** the necessary observer-cap restoration and generation-publication race repairs
passed all 539 tooling tests through `scripts/pse-env --resource-class light -- just setup-test`.
The supervisor module accounted for 85 passing controls. Actual native observer completion
restored the enclosing allocation to its original 144 GiB cap. No foreign service was stopped;
the task-owned timing service was quiesced and drained through the supported lifecycle.

**Tested:** `just bench-case-smoke build/graph-hash-followups-20261009-restart-final --case k4-restart-local-1`
passed with the permanent implementation, native-process/force-validation, one native thread
and `measured: false`. Actual portable reconstruction, independent fresh admission, original
history/float bits and weak-owner/zero-pool release checks passed. The same executable uses a
quiet warmed loader; this is not managed-worker/process/Python startup, server restart, crash
or strict deployment qualification. Temporary context snapshots confirmed equal independent
seed/receiving observations in the diagnostic run and are removed from production. They do
not recover the changed field in the earlier refused execution.

The final PC-SAFT smoke harness completed all four stages with typed invariant-fact refusals;
no prepared product or larger-workload hash comparison was obtained. [28f](28f-shared-numerical-preparation.md#pc-saft-numerical-fact-investigation-boundary)
owns the concrete numerical-fact investigation, preserving the existing scientific types,
invariant checks and precision policy.

**Tested — scope-end checks, zero baseline:** `just ready` passed. The initial
`just hygiene` returned exit 1 with three unsuccessful recipes: `lint-typos`, `lint-py`
and `typecheck`; generation, schema, family, both Clippy modes, Rust documentation and
the other hygiene recipes passed. The three named failed recipes subsequently passed
after valid reference terminology, loop captures/context formatting and optional-value/
fixture typing were corrected. Final `just typecheck` emitted zero diagnostics, and
`scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.test_surreal_server`
passed 85/85. This is repaired composite hygiene acceptance, not a fresh successful full
bundle execution. No quality baseline or suppression was added.

**Tested:** `just governance` passed all 102 governance tests across 18 binaries,
code-generation comparisons and family checks. `just lint-native-contracts` and
`just lint-native-data` both passed with force-validation and warnings denied in the
native checkout environment. Their earlier separate light-scope invocations were
interrupted by signal 15; the final checks ran within the held native allocation.
`just docs` published 296 chapters; final handoff publication follows the last document
updates. The maintainer cancelled the subsequent feature-powerset run. Both
`features-combinations` and the keep-going `features-no-default` stage were interrupted;
the combinations log reached the started `207/313` entry. These are two unsuccessful
stages against the zero-failure baseline, not feature-powerset acceptance. The retained
report has `input_coverage: false` and cannot support unchanged-input reuse. No continuation
is selected by this handoff.

**Measured:** the [hashing report](../design_review/evidence/graph-hash-followups-2026-10-09/hashing-results.md)
retains actual scalar interner/projection/framing samples and the original preparation-key
traversal inquiry. The 0.535 µs key traversal and 150.857 ms complete warm preparation are
different operation boundaries, not a before/after speedup or an exact internal latency share.
Eight scalar fresh-process controls preserve full Std/Fx and forced-collision equality; noisy
whole-projection ranges do not establish an end-to-end improvement. No larger PC-SAFT hash
comparison was obtained. The real Recipe and authored-checker probes passed two selected
compiler tests with the temporary persistence feature, force-validation and local profile;
their [results](../design_review/evidence/graph-hash-followups-2026-10-09/README.md) distinguish
snapshot decode reuse from mathematical reconstruction and upstream checking from downstream
equality cutoff. Production retains qualified portable reconstruction and the complete checker.

**A mistake made and corrected:** the larger inquiry initially used obsolete benchmark
document/import boundaries and two stale physical quantity declarations. The fixture now uses
the accounted owned-document loader, explicit species dependency and canonical Helmholtz output
types without changing numerical inputs or tolerances. Observer control admission also exposed
an actual cap-restoration defect and concurrent generation rename race; both were repaired and
reviewed rather than bypassing workload admission. Initial feature/store selection mistakes
were corrected before the successful current-source checks. The restart fixture's single
scheduler yield was also an invalid release assumption: completion publication can precede
the supervisor's final Arc destruction. Its replacement waits for both actual owner release
and zero pool reservations before retaining the original assertions. Hygiene exposed three
Python typing errors; narrowing/fixture annotations were corrected, `just typecheck` reported
zero diagnostics and the supervisor's 85 controls passed again.

**Deliberate deviations:** local optimizations were adopted under the maintainer's
correctness/significant-regression criterion despite marginal or inconclusive timing. The
larger diagnostic hash campaign was not repeated after fixture repair; functional acceptance
does not create a PC-SAFT performance claim. RC02 durable fast hashes and RC03 production
persisted incremental execution remain unselected. The typed-checker investigation proposal
now routes to [Plan 33/EI04](33-efficiency-principles-remediation.md#ei04); broader graph layouts,
cross-call flow sharing and persistence route to that plan's EI05 and require their own
complete consumer contracts. Full remaining Plan 28 E3/E4/E5 execution is now authorized at its existing owner.

**Handoff and retention:** active Plan 28 companions and proposed ADR-0167 still consume
this scoped implementation and inquiry evidence. Retain this record while those references
are needed; it is not a backlog or authorization for the numerical-fact, typed-checker,
durable-hash or persisted-runtime investigations. Their existing owners define any next
authorized work, with the later independent Plan 33 handoff above applying prospectively.
Enduring V2 framing and supplier-preparation meaning is recorded at the
identity/publication and numerical-execution architecture owners.
