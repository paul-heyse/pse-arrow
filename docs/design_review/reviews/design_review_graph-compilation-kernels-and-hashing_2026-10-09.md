---
title: Graph compilation, reusable kernels and hashing
date: 2026-10-09
tier: design
purpose: target
status: review
---

# Graph compilation, reusable kernels and hashing

**Architectural fitness: Revise. Behavioral and scientific adequacy: unresolved for the assembled boundary. Overall decision: Revise.**

Further graph preparation can improve this simulator, particularly by retaining immutable implicit-provider dependencies across case registration. Faster hashing has a separate, narrower role in lookup and candidate selection. Combining dependency graphs with fingerprints could support finer invalidation and additional persistent reuse, but the reviewed source does not justify a universal graph compiler, another task framework, or replacement of all durable hashes.

The system already has important parts of the target architecture: library-owned structural algorithms, Salsa queries, exact preparation keys, immutable shared mathematics, value rebinding, bounded retention and flights, current attribution, and explicit publication effects. Complete preparation reuse is implemented; it is inaccurate to describe every study point as rebuilding its compiler and mathematical basis.

Three residual mechanisms warrant revision. Warm description settlement repeatedly searches an immutable description vector. Flow admission repeatedly scans all decisions for each connection. Implicit registration reconstructs immutable supplier topology before resolving demand-specific numerical state. Two corrections need ordinary indexed access; one earns a reusable graph view. None requires faster durable hashing.

Recommendations are **Proposed**. Existing paths are **Implemented**, established by source inspection. Selected library contracts are **Interface-checked**. No new product tests, numerical experiments, benchmarks or qualification campaign were executed.

## Scope, target and evidence

This independent **DESIGN / TARGET** review applies Core/template 3.4, Heuristics for Efficient Architecture 1.0, ProcessSimulator 1.5 and the selected pse-arrow binding. The functional target is an extensible simulator supporting interactive edit/re-solve, studies, recycles, dynamics and fitting with physically meaningful, independently assessed outcomes.

The boundary covers preparation, structural projections, orchestration dependencies, expression sharing, hashing and reuse through publication. Numerical iteration, provider thermodynamics, full derivative chains, store durability and every analysis mode were not requalified. Build/tooling are considered where adoption changes product realization or reconstruction. No capacity SLA or mandatory new machinery is inferred.

**Baseline:** `main` at `d0f2c41818a34539a910654dfea4760771603f45`, with concurrent dirty tooling/documentation changes and preserved untracked reference material and Plan 31. The reviewer made no repository edits. HEAD alone does not identify installed artifacts.

The [Plan 28 checkpoint](../../plans/28-surrealdb-unified-substrate.md#current-checkpoint) and companions 28b/28f/28i/28j provide current context. B7/N11 complete preparation reuse is implemented; scoped qualification remains distinct from the paused E4/E5 campaign. This review neither reopens corrected findings without current evidence nor closes those acceptance obligations.

The supplied local fast-hashing reference `docs/graph_hashing_reference/Fast hashing for graph systems.md`, Rust implementation comparison `docs/graph_hashing_reference/xxhash-rust-and-twox-hash_2.md`, and library-context reviews `design_review_graph-compilation-kernels-and-hashing_2026-10-09.md` and `design_review_graph-compilation-and-reuse-plan_2026-10-09.md` supplied mechanisms and counterexamples. These local reference inputs are not published chapters. Their diagnoses and performance numbers are not evidence of pse-arrow behavior. Current architecture owners are blueprint §5, §14, §15, §17, §18 and §20; their mechanisms describe the subject rather than constrain target judgment.

## Structures, operations and responsibility boundaries

“Graph compilation” comprises different operations whose meanings must remain separate.

| Structure | Meaning and owner | Preservation obligation |
|---|---|---|
| Flowsheet topology | Units, ports, directed connection occurrences and tear groups; compiler process projection and structural flow owner | Isolates, parallel occurrences, physical bindings and policies |
| Equation–variable incidence | Original equalities, free coordinates, conservative support and coupling; structural/math owners | Complete class-specific scope; topology cannot substitute for incidence or numerical rank |
| Expression graph | Factorable nodes, provider calls, guards and outputs; mathematical owner | Exact constants, selection meaning, domains and derivative obligations |
| Compiler dependencies | Source, physical context, bindings, provider descriptions and demand; compiler/Salsa | Presence, absence, membership and consumed interpretation inputs |
| Study prerequisites | Occurrences, ordering, usability and continuation permissions; study policy | Equal computations remain separate occurrences with independent effects |
| Implicit supplier dependencies | Which admitted provider supplies another provider or observation; compiler | Selected reachability and cycle rejection |
| Physical execution plan | Layouts, evaluator products, native workspaces and scheduling | Private mutable state, capability-specific reuse and original assessment |

These structures can share graph algorithms and compact identity mappings. They cannot share an undifferentiated edge meaning or invalidation policy.

[Compiler modeling](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-compiler/src/workspace/modeling.rs) owns immutable admission and selected specialization. [Grouped preparation](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-compiler/src/workspace/modeling/executable/grouped.rs) trims consumer expressions and transitive function/provider dependencies. [Math preparation](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-runtime/src/math/preparation.rs) retains pure products. [Workflow preparation](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-runtime/src/workflow/modeling/preparation.rs) binds current lineage and settles acquisition/publication. Study policy owns scientific prerequisites; canonical adapters own I/O and fencing. Numerical libraries retain fitting iteration and factors.

A new unit using existing physics should extend declarations and scientific behavior. A new study should compose preparation/execution while adding its actual policy. Solver substitution belongs to capability selection and the integration owner: equal topology does not establish equal derivative, accuracy, start or cancellation contracts.

| Owned operation | Inputs and decisions → outputs/effects | Equality and lifetime |
|---|---|---|
| Prepare selected model | Exact request/dependencies, root, instance, bindings and limits → pure basis | Full BasisKey equality; bounded shared retention and completion-owned flights |
| Bind current use | Current revision/sources → current Solved lineage and consumed-source metadata | Fresh attribution; shared mathematics remains immutable |
| Prepare supplier topology | Admitted bodies/descriptors and output relationships → adjacency and identity maps | Equal dependency structure; admitted-product lifetime |
| Register demanded suppliers | Selected rows/coordinates, derivative demand, case/policy/controls → private registrations | Fresh demands and numerical state; topology equality is insufficient |
| Settle descriptions | Portable bodies, retained descriptions and protected read → acknowledgment or qualified publication | Exact identity/read correspondence; retained bytes grant no authority |
| Decide study candidate | Candidate and immediate predecessor facts/policies plus cancellation → typed action | Adapter fences every consumed revision before effects |

Ordinary typed functions can implement these contracts. A serializable universal operation language is unnecessary.

## Strengths that constrain the remedy

`selected` and `specialized` already use Salsa; equal selected results can stop downstream propagation. `modeling_request` retains immutable root/instance/binding/limit requests, enabling A/B/A reuse. Its linear bounded search is an optimization opportunity, not evidence that semantic incrementality is absent.

`prepare_selected` checks the exact retained basis before constructing a pure workspace. `BasisKey` compares the full request, dependencies, root, instance, bindings and limits; hashing supplies lookup, not equality. Generation fencing and allocation owners protect retention. Attribution and effects remain current. Source controls for many physical points support this separation, but were not rerun and do not establish end-to-end thousand-point qualification.

`Builder::push` in [factorable mathematics](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-math/src/factorable.rs) already hash-conses exact Node keys into dense IDs. Another expression interner would duplicate that mechanism unless it serves a distinct lifetime and workload.

[Study policy](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-operations/src/study_policy.rs) separates topology admission from candidate decisions. `AdmittedStudy` retains immediate predecessors; `candidate_action` consumes scoped facts. [Canonical studies](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-operations/src/canonical_studies.rs) fence consumed revisions. No whole-study reconstruction per dispatch was established. [Canonical analyses](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-operations/src/canonical_analyses.rs) distinguish staged membership from active complete graphs; a new hash cannot replace activation completeness.

Flow admission retains connection occurrences rather than collapsing endpoint pairs and checks connected physical contracts before projection. These are constraints on compact lowerings, not reasons to retain every existing scan or allocation.

## Findings

<a id="f01"></a>
### F01 — Warm description settlement repeats linear identity searches

**Implemented/source-inspected diagnosis.** AP-07, DP-10, DP-21; S01/S02.

`PreparedBasis` stores descriptions in a vector. `settle_preparation` loops over each portable body and calls `basis.descriptions.iter().find(|old| old.semantic_identity == identity)`. `PreparedModeling::portable_bodies` additionally constructs a fresh BTreeMap over primary and original bodies on each call to deduplicate the inventory.

Many distinct matching bodies therefore repeat triangular identity comparisons before acknowledgment work, even on a complete basis hit. Required current acknowledgments have a separate purpose; they do not justify rediscovering immutable descriptions.

**Proposed correction:** retain the canonical portable inventory and use an exact index or merge over canonical order. An ordered index suffices; faster hashing is optional. Keep read correspondence and acknowledgment/publication in the workflow owner. A successful acknowledgment cannot be remembered as permanent authority.

**Verification:** many-body warm preparation avoids repeated full searches while preserving changed reads/dependencies, expiry, missing acknowledgment, fresh lineage and original provenance. Include index construction and retained bytes. No latency benefit is measured.

<a id="f02"></a>
### F02 — Flow admission scans all decisions for each edge

**Implemented/source-inspected diagnosis.** AP-07, DP-07; S03.

[FlowGraph::admit](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-structural/src/flowsheet.rs) sorts decisions, then constructs its forbidden-edge witness graph with a retained-edge predicate calling `d.decisions.iter().any(...)` for every connection. Independently declared decision groups cause edge count multiplied by decision count examinations of policy already identified by each edge. This is incidental work before legitimate global cycle computation. Finite limits do not justify the amplification.

**Proposed correction:** binary-search the already sorted decisions or use an exact shared decision lookup. Preserve connection occurrences and the independent forbidden-cycle witness. No new compiler, cache or hashing family is necessary.

**Verification:** reordered declarations, parallel occurrences, unused/missing decisions, policy variants and forbidden cycles retain outcomes and identities. Source reasoning can establish removal of the nested scan; timing requires measurement.

<a id="f03"></a>
### F03 — Case registration repeatedly reconstructs immutable implicit topology

**Implemented/source-inspected diagnosis.** AP-07, DP-10, PS-11; S02/S04.

[AdmittedModeling::implicit_order_for](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-compiler/src/workspace/modeling/executable/implicit.rs) constructs a new graph, visits every implicit system's bodies, computes value-level provider demand across their outputs, and derives supplier edges. It then optionally restricts to requested rows and topologically sorts.

[ModelingPackage::inner_registrations](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-runtime/src/workflow/modeling/implicit.rs) enters this operation for fresh observations/cases. [Case preparation](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-runtime/src/workflow/modeling/cases.rs), including `resolve_case` and demand upgrades in `finish_case`, can register more than once as actual derivative requirements become known.

Fresh registrations and demands are necessary; reconstructing unchanged supplier adjacency from immutable bodies is not. Many nested cases multiply this discovery despite shared basis reuse.

**Proposed correction:** retain immutable supplier adjacency and observation-to-root mappings in the compiler-owned admitted product. Each use computes selected closure/order; runtime still resolves current derivative demand, policy, hints and private provider state.

Preserve the current order of obligations: row filtering precedes topological cycle rejection. A valid requested closure must not be rejected because an unrequested component cycles. A globally validated DAG or universal precomputed order would change behavior. Retained adjacency, optionally SCC metadata, must support selection-bound rejection/order. Full-scope requests retain full-scope obligations.

**Verification:** compare retained selection against current clean construction for selected/full scope, an unrequested cycle, sibling suppliers, demand upgrades and changed edges. New case state and actual output/coordinate derivative demands remain fresh. Compare construction and retained-state costs.

The findings have separate closure obligations: identity lookup during effects, policy lookup during topology admission, and graph preparation lifetime. A shared optimization project should not obscure those owners.

## Opportunities and complete alternatives

Finer compiler admission is eligible. `publish_modeling_with` avoids unchanged input, but changed packages enter checking and update a whole checked catalog. Selected-result equality can stop downstream work without eliminating upstream checking. Granular tracked declarations/domains could help small frequent edits; first establish what work remains necessary for complete references, physical context and negative lookups. Another manually maintained dependency graph would worsen authority.

Repeated flow preparation is also eligible. `prepare_modeling_flow` builds anew and associates actual allocation ownership. Cross-call reuse could help identical requests, but inspected callers show one build per operation, not demonstrated in-operation reconstruction. Another cache must earn key construction, validation and retention.

Selection closure, supplier adjacency, identity mapping and prerequisite evaluation are credible reusable kernels. Generic mechanics can share libraries while seed usability, branch selection, physical binding and original assessment remain domain operations. A kernel earns its boundary by removing repeated mechanics or supporting real variation.

| Alternative | Capability and full-lifecycle burden | Judgment |
|---|---|---|
| Current | Existing exact reuse and ownership; residual scans/discovery | Revise F01–F03 |
| Shared functions/prepared views | Removes repeated discovery with little lifecycle expansion | Preferred starting point |
| Compact graph | Dense traversal and identity-based diagnostics; conversion and retained adjacency | Fits F03; choose container by consumed algorithm/edge semantics |
| Native database | Indexed selection and coherent durable membership; queries, transactions and transport | Appropriate where persisted selective access benefits; not default numerical placement |
| Relational lowering | Bulk joins, projection/reduction and engine optimization | Preserve useful engine visibility |
| Library incrementality | Read tracking, equality cutoff and dependency maintenance | Extend existing Salsa before another engine |
| Universal operation/task compiler | Broad executable contract, persistence and composition machinery | Not earned by demonstrated defects |
| Graph plus content-addressed persistence | Cross-restart pure-product reuse | Conditional on complete dependencies, compatibility and recovery |

| Opportunity | Graph only | Hash only | Combined | Selection condition |
|---|---|---|---|---|
| Warm description lookup | No new graph needed | Exact index may use map hashing | No added benefit | Sorted merge/index first |
| Flow policy lookup | Existing graph stays | Optional exact decision map | No added benefit | Binary search can use existing order |
| Implicit supplier discovery | Retained adjacency removes repeated discovery | Cannot replace edges or reachability | Could identify compatible adjacency across revisions | Admitted-product sharing first |
| Small edit/revert | Tracked dependency domains | Canonical fingerprints find candidates | Complete domains plus equality/cutoff | Compare upstream admission against existing Salsa |
| Many cases | Shared structure and demand views | Lookup selects compatible products | Existing basis mechanism already combines these roles | Remove residual work before another cache |
| Persistent pure reuse | Declared dependency/completion structure | Durable content identity and integrity | Reconstruct compatible products and validate current premises | Reuse expensive products; do not persist effects |

**Container and database fit.** Petgraph 0.8.3 already supplies the relevant structural algorithms. CSR may improve compact adjacency traversal, but conversion requires sorted unique endpoints and cannot preserve parallel connections directly. A projection that deduplicates endpoints needs an explicit consumer meaning and a mapping back to occurrences; isolates must also survive. The checked interface supports Tarjan SCC on CSR, while Kosaraju requires a visit interface CSR does not implement. Conversion, sorting and retained memory belong in the comparison against the existing graph, rather than assuming traversal gains dominate.

SurrealDB can own persisted relationships, selective source retrieval and coherent graph membership without becoming the numerical kernel. The product uses a patched vendored 3.3.0 SDK with WebSocket/rustls features; reference material discussing 3.3.2 does not qualify its transport, correlation or replay behavior. Any database-derived reusable closure must distinguish a complete bounded answer from truncation, validate consumed revisions and inspect query/write failures. Canonical claim/publication transactions remain effects with separate recovery obligations. The benefit of database placement depends on selective persisted access removed from the full operation, not the presence of graph syntax alone.

Cold work includes admission, graph/index construction, canonicalization and encoding. Warm work includes key discovery, current checks and effects. Retention includes indexes, revisions and escaped aliases. Recovery includes receiving qualification and exact settlement. A concise declaration does not eliminate these obligations.

## Hashing as an independent decision

| Job | Relevant realization | Contract |
|---|---|---|
| Maps | Exact lookup, possibly specialized hasher | Full equality and explicit observable order |
| Interning | Existing factorable/Salsa mechanisms | Exact structure and local handle lifetime |
| Change fingerprint | Optional fast canonical digest | Complete scope, versioned encoding and collision-safe decision |
| Durable identity | Existing BLAKE3 or explicitly redesigned contract | Versioning, references and reconstruction/migration |
| Integrity | Cryptographic digest at an identified boundary | Digest does not establish scientific validity or origin |
| Attestation | Actual producer/receiving/publication authority | Public domain-separated hashing is not authentication |

The locked twox-hash 2.1.5 currently exposes XXH32/XXH64 through compression/Parquet consumers. XXH3/std/runtime-dispatch features are not already enabled. xxhash-rust 0.8.19 is cached but not locked. Transitive presence is not whole-capability adoption.

The library-evidence work executed `scripts/pse-env -- cargo tree --locked -i twox-hash -e features --depth 2`, `scripts/pse-env -- cargo tree --locked -i salsa -e features --depth 1`, and `scripts/pse-env -- cargo tree --locked -i petgraph -e features --depth 1`: all **passed**, exit 0, against a zero command-failure target. These inspect the workspace default-selection feature union, with no package selection, all-features, native or force-validation mode; they are dependency inventory, not product tests. Salsa's selected union includes inventory/macros/salsa_unstable, not persistence.

**Interface-checked exact alternatives:** twox-hash 2.1.5 `XxHash3_128::oneshot(&[u8]) -> u128` requires `xxhash3_128`; its ordinary `new/write/finish_128` streaming state additionally requires `alloc`, constructing an allocated secret. `RawHasher<S>` offers caller-owned secret storage. The crate's defaults include alloc, std, random and both XXH3 widths, unlike the actual consumer feature set. xxhash-rust 0.8.19 `xxh3::xxh3_128(&[u8]) -> u128`, `Xxh3Default::new/update/digest128` and `Xxh3DefaultBuilder` require `xxh3`; no feature is enabled by default, and the default streaming state uses inline storage. Its SIMD selection is compile-time target-feature based; twox has std-enabled runtime selection on applicable targets.

The 128-bit digest must be requested explicitly: xxhash-rust's `Hasher::finish()` returns 64 bits, while twox's XXH3-128 state uses `finish_128()` rather than implementing `std::hash::Hasher`.

These contracts were read in cached published source, respectively `twox-hash-2.1.5/src/{lib,xxhash3_128}.rs` and Cargo.toml, and `xxhash-rust-0.8.19/src/xxh3.rs` and Cargo.toml. They establish interfaces/feature obligations, not this project's speed or cross-language qualification. The supplied local implementation comparison `docs/graph_hashing_reference/xxhash-rust-and-twox-hash_2.md` records broader external probes; its results were not reproduced.

Map alternatives include ordered maps/binary search, std RandomState, rustc-hash 2.1.3 `FxBuildHasher`, foldhash 0.2.0 `fast::RandomState`, twox XXH64 state and xxhash-rust's default builder. Cached `rustc-hash-2.1.3/src/lib.rs` and `foldhash-0.2.0/src/{lib,fast}.rs` establish their builder interfaces and security limits. Their presence in Cargo.lock is not a recommendation to expose them throughout the product. Dense IDs/arrays can avoid map hashing; structured semantic IDs are not suitable for unchecked no-hash assumptions. Select against key sizes, trusted-key exposure and construction/lookup/retention. Bulk one-shot throughput cannot establish small-key map benefit.

Fast in-memory hashes select candidate buckets followed by exact equality. `BodyInput` currently equates a sealed semantic identity; replacing that identity with XXH is not equivalent to changing a map hasher. [Derivation](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-ids/src/derive.rs), [framing](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-ids/src/frame.rs) and [float handling](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-ids/src/float.rs) distinguish domain-separated identity from canonical preimages. NaNs collapse only for hashing; signed zero remains distinct. Collection boundaries, roles, order and exact rational constants must survive new encoding.

**BLAKE3 is not an immutable architectural requirement of this review.** Durable noncryptographic identity remains eligible under an appropriate equality/trust contract. Its benefit must earn collision handling, identity interpretation, schemas/references, historical reads and reconstruction. Retaining cryptographic identities while adding fast candidate fingerprints currently introduces less machinery. [Producer identity](https://github.com/paul-heyse/pse-arrow/blob/d0f2c41818a34539a910654dfea4760771603f45/crates/pse-buildinfo/identity.rs) separately records scientific consumed inputs and external file checksums; faster lookup cannot remove completeness or observation-to-use obligations.

### U01 — Flow fingerprint structural framing remains a bounded open premise

`FlowGraph::admit` writes each node ID followed by three IDs per port without a per-node port count or role tag; other inventories are variable length. Every part is length-framed, but ID parts have the same width. Abstractly, node A with port B/quantity C/unit D can produce the same part stream as isolated nodes A,B,C,D when their IDs sort accordingly. This is encoding ambiguity, not an accidental hash collision.

The public Declaration boundary checks uniqueness within node/connection/decision inventories and among ports; it does not enforce global disjointness between those roles and quantity/unit identities. Compiler `SemanticModeling::flow_graph` supplies actual instance/member/registered physical identities, a stronger source than arbitrary raw values. The inspected identity and registry contracts did not establish a global role-exclusion premise sufficient to settle whether the abstract example is a legitimate authored/compiler-supported graph. No such product case was executed or established. `FlowGraph::PartialEq` also compares full declarations/bindings, limiting inference from equal keys.

Accordingly this is an unresolved fingerprint-contract premise, not F04 or a claim of scientific misreuse. Before relying on a new flow fingerprint as sole equality, establish the admitted identity universe or use explicit inventory/port counts and role tags. Changing existing FlowProjectionV1 bytes would require a new interpretation; neither F02 nor the selected corrections require that change. A supported counterexample would warrant a separate fidelity/reuse finding.

## Combined invalidation, cycles and persistent reuse

A dependency graph identifies potential influence; a fingerprint identifies encoded content. Neither proves complete dependencies. Selective reuse includes absent lookups, matching membership, deletion and provider/policy changes. Previously returned objects cannot represent an absent-to-present transition or new member. Conservative domains are preferable to unsound precision; narrowing requires a current completeness authority.

Name the graph before handling cycles. Physical recycles and equation coupling are legitimate; supplier execution cycles may be invalid only in the requested closure. SCC condensation can organize propagation, but member sets alone do not identify topology. Fingerprints must preserve directed internal edges, occurrence identities, roles and labels. Merge/split edits require affected partition recomputation; recursive child hashes or WL labels cannot establish exact graph equality.

Output equality stops propagation only for consumers observing substitutable meaning. Equal mathematics can coexist with changed spans/provenance needing refreshed attribution. Edit/revert can recover a pure product, never old execution authority.

Persistent reuse should extend useful existing portable products. Salsa 0.28.4 supports feature-gated persistence, currently off in the reviewed consumer; it does not replace scientific receiving qualification. Interned/tracked handles remain governed by runtime/generation contracts. Mutable evaluators, factors and attempt state remain separately capability-qualified. No second persisted incremental runtime is selected here.

Local-only capability evidence is labelled rather than treated as a published repository artifact: `salsa/content/capabilities/salsa.persistence.md` requires persisted query inputs and serde-compatible keys/results, and notes that accumulated values do not round-trip; interner collection can cause restored memos to rerun. The salsa skill's B001 establishes equal-result backdating, with `no_eq` changing the behavior. `rust-graphs/content/index/capability-matrix.tsv` distinguishes CSR's Tarjan support from its unavailable Kosaraju route. Both live under the selected shared library-skill roots. These contracts do not qualify this consumer's persistence or graph conversion. In particular, the supplied fast-hashing reference §7.4's sorted reachable-content multiset recipe is insufficient for topology-sensitive reuse: different edges can yield the same reachable content. The stronger §6.1 discussion includes internal edges and explicitly warns about symmetric/WL-equivalent cases. Neither recipe transfers automatically.

## Scenarios and scientific preservation

| ID | Stimulus | Required response |
|---|---|---|
| S01 | Unchanged/value-only/structural edit/revert | Reuse compatible products; refresh consumed values and attribution; structural literals stay structural |
| S02 | Many cases/concurrent equivalent preparation | Share immutable work; preserve independent stop/deadlines, private state and charges through drain |
| S03 | Recycle/internal edge change/SCC merge or split | Preserve multiplicity/bindings; recompute affected topology and tear validity |
| S04 | Nested suppliers/selected rows/unrequested cycle | Reuse adjacency; selected cycle rejection and actual derivative demands remain current |
| S05 | New unit/property/study/solver | Extend meaning/capability owner; reuse preparation without copied policy |
| S06 | Absent-to-present/member insert-delete/provider-policy edit | Invalidate complete affected domains, including negative answers |
| S07 | Collision/reorder/signed zero/NaN/exact constants | Unequal meanings never merge; encoding and ordering remain explicit |
| S08 | Cancellation/restart/stale completion/publication failure | Pure products grant no effects; generation/fencing/settlement stay current |

| Scientific element | Physical meaning and preservation |
|---|---|
| Ports/connections | Declared quantity/unit, basis/reference/convention; connected physical correspondence and conversion |
| Expressions/providers | Formal/output types, guards, envelope, branch and derivative capability |
| Case values/roles | Fixed/free/parameter/guess distinctions; value presence cannot imply structure |
| Dynamics/fitting | Same model plus time/mode/sample/observation correspondence; topology is not trajectory/statistical validity |
| Published candidate | Original identities/units, start provenance and typed outcome; required fresh numerical checks |

Blueprint §15 describes class-aware matching over original rows/free variables and model identities in refusals. Matching does not prove numerical rank. Proposed graph/hash mechanisms cannot bypass it. No new formulation, derivative method or solver is introduced: guards, derivative source/order, scaling, class/capability, tolerances and typed outcome remain with existing owners. Blueprint §18's original-space assessment and physical closure remain mandatory for new candidates. No unit/property conformance or reference parity was rerun.

## Independent judgments

| Foundation | Verdict | Scoped basis |
|---|---|---|
| AP-01 | Satisfied | Meaning, pure preparation, numerical state and effects have identifiable owners |
| AP-02 | Satisfied | Exact requests/dependencies and capability-specific boundaries |
| AP-03 | Satisfied | Preparation and policy compose without universal graph machinery |
| AP-04 | Satisfied | Inspected distinctions govern admission/binding/selection |
| AP-05 | Satisfied | Explicit requests, physical contracts and ownership |
| AP-06 | Satisfied | Pure compiler/policy responsibilities and bounded correction surfaces |
| AP-07 | Violated | F01–F03 repeat work unrelated to changed meaning |

These assess inspected responsibilities, not every model/backend. U01 concerns unsettled fingerprint sufficiency; it does not establish loss of the domain model.

| Gate | Judgment | Evidence or limit |
|---|---|---|
| G1 Authority | Pass, scoped | Derived products consume identified owners |
| G2 Fidelity | Pass, inspected scientific graph mappings; unresolved fingerprint premise | Occurrences/roles preserved; U01 prevents broader sole-key fidelity claim |
| G3 Validity | Unresolved, assembled | Checks exist; complete scientific/provider execution not audited |
| G4 Hidden behavior | Pass, scoped | Pure products separated from current effects |
| G5 Recovery | Unresolved, assembled | Fences/ownership/settlement inspected; no new restart campaign |
| G6 Reuse | Pass, inspected exact-basis contracts; unresolved broader fingerprint-only reuse | Full basis equality/current attribution; U01 and prospective fine reuse not settled |
| G7 Claims | Pass at stated evidence | No transferred performance or qualification claim |
| G8 Libraries | Pass, scoped | Existing graph/incrementality/numerical leverage; no new generic engine needed |
| G9 Fitness | Fail | AP-07 violations F01–F03 |
| PS-G1 Physical consistency | Unresolved, assembled | Admission inspected; provider/conservation qualification not renewed |
| PS-G2 Well-posedness | Unresolved, assembled | Distinct topology/incidence and class-aware contract; paths not requalified |
| PS-G3 Numerical integrity | Unresolved, assembled | Required new-state assessment preserved; full derivative/status chains not audited |

## Rule impacts, follow-up and acceptance

**Actual rule impacts of selected corrections: none.** F01 exact indexed description access, F02 indexed policy access and F03 retained immutable adjacency with selection-bound ordering preserve current identity, dependency, physical, numerical and publication contracts. No ADR, durable hash change, persisted query engine or authority amendment is recommended by these corrections. If implementation changes an enduring internal API, its owner should explain it; that is not itself a changed rule.

| Recommendation | Current location/text meaning | Proposed correction | If current rule remains |
|---|---|---|---|
| F01 | Blueprint §14.3: portable inventory and retained encoding; every consumer settles current effects | Exact retained lookup/inventory | Correction remains fully available; acknowledgment rules stay |
| F02 | Blueprint §15.3 and FlowGraph::admit: complete physical occurrence graph and independent cycle witness | Indexed existing decision policy | Correction remains available; topology and witnesses stay |
| F03 | Blueprint §14.3–§14.4: shared immutable products, fresh current demands/state | Retain adjacency; compute selected order without global cycle rejection | Correction remains available; no semantic/identity change |

The following are **optional alternative impact notes**, not recommended or accepted rule changes:

<a id="rc01"></a>**RC01 — Finer complete dependency domains.** Blueprint §14.4 and Plan 28 already permit complete narrower dependencies; no rule change follows merely from pursuing them. Changing actual durable projection meanings would need explicit versions. If conservative domains remain, existing reuse and F01–F03 corrections remain available.

<a id="rc02"></a>**RC02 — Durable hash replacement.** A separately selected replacement would affect blueprint §5.1/§5.3, frozen frames, identity rationale, codecs/references and portable reconstruction. It must specify new interpretation and retirement/rebuild or migration; old bytes cannot be silently reinterpreted. Keeping current hashes permits all selected corrections and candidate-map alternatives.

<a id="rc03"></a>**RC03 — Persisted incremental runtime.** A separately selected Salsa store/general persisted operation graph would change blueprint §14.4's current local runtime and §20 reconstruction/receiving ownership. If those rules remain, existing portable products continue serving durable reuse. This alternative is not selected.

<a id="rc04"></a>**RC04 — Flow fingerprint framing.** Changing FlowProjectionV1 encoding would affect blueprint §5.3's frozen interpretation and require a new frame/version. This is contingent on settling U01 or adopting a separately specified stronger fingerprint. Keeping current framing does not block F02; sole-key equality remains unestablished by this review.

No recommendation is scheduled or implemented. If adopted, the [Plan 28 coordinator](../../plans/28-surrealdb-unified-substrate.md#finding-dispositions) should own disposition and link the packet. Suggested owners: runtime preparation/28j for F01; structural flow preparation/28f for F02; compiler/runtime supplier preparation/28b/28f for F03. U01 belongs to identity/structural owners for a bounded contract decision. This review is not another status ledger.

Source evidence suffices to recommend removing demonstrated amplification. No probe is necessary to establish the nested scans or repeated topology discovery. Selecting a particular map hasher, broader layout, finer incremental boundary or persistent cache needs representative complete cold/warm, retention and recovery evidence. Licenses and current pins do not exclude technically suitable alternatives.

**Decision:** revise F01–F03, preserve complete basis reuse, and consider additional graph/hash machinery only at named operation boundaries. Acceptance requires preserved semantics and lifetimes; implementation completion and wider E4/E5 qualification remain separate.
