# Structural correspondence and shared mathematical operations

Inspected 2026-11-09 at `f6e91eb37`; **Interface-checked** observations use locked Graphica 3.0.1, maintained Symbolica 3.0.1 and the named repository owners. Integrated workflows below are **Proposed** discovery hypotheses, not implementation or adoption decisions. No execution probe, test or performance measurement was run. The supplied Graphica addendum was read in full; original discovery findings retain their original dates outside the corrections explicitly stated here.

## The opportunity is mapped mathematics, not graph equality alone

A canonical graph can identify descriptions of the same topology despite different numbering, but an executable mathematical template needs more: ordered inputs and outputs, coordinate transformations, immutable mathematical definitions, derivative demand, source/guard support and effect boundaries. The promising composition is Graphica correspondence → owner-qualified mathematical template → ordered Symbolica parameters and named definitions → demanded derivative product → shared evaluator. It could remove repeated symbolic preparation across genuinely equivalent instances, or enable a useful new topology-design operation. It does not follow that all isomorphic process networks have interchangeable calculations.

The current source already separates useful identities. `pse-structural::flowsheet::Declaration` retains units, complete port inventories, explicit connection occurrences and declared tear groups. `FlowGraph` validates port ownership, quantity contracts and conversion plans before producing a semantic key; parallel occurrences are never merged by endpoints. In compiler `workspace::modeling::executable`, definitions obtain numeric and guard read sets from resolved expressions; free inputs become ordered physical `Formal` entries, and output regions are prepared from resolved equations. The dependency graph separates numerical and guard edges. Only numerical edges enter arithmetic topological sorting, while guard edges still participate in consumer closure.

Those are concrete seams for correspondence. They are also counterevidence to canonicalizing a bare adjacency graph and treating its key as permission to reuse all downstream products.

## A candidate correspondence representation that retains edge and port roles

A direct Graphica graph whose vertices are units and edges are connections cannot by itself transport every current flow record. `CanonicalForm` has a vertex map but no edge map. Interchangeable parallel connections may carry distinct binding and tear records; hiding occurrence IDs does not manufacture a unique edge correspondence.

A promising *research representation* is a colored incidence graph:

- Represent a unit, each named port role, each connection occurrence and each declared decision group as distinct graph vertices, colored by kind.
- Represent ownership, source/target attachment, binding membership and tear-group membership as differently labeled directed edges. If a connection contains multiple scalar bindings, give each binding its own vertex or equivalent lossless incidence structure.
- Place result-affecting physical contracts, direction, mathematical/equipment role and applicable decision policy in visible labels. Keep original occurrence IDs and diagnostic provenance in an external mapping or `HiddenData` only where their renaming is explicitly allowed.

This makes connection/port correspondence part of Graphica's **vertex** permutation, rather than requiring a nonexistent canonical edge map. It also preserves parallel occurrences as separate structural objects. It is not a selected schema: conversion amplification and canonicalization cost may outweigh the saved preparation, especially when existing definition-level sharing already captures repeated equipment mathematics. A unit kind alone cannot stand in for full equation/provider/parameter semantics. If equality-relevant data includes actual occurrence IDs, cross-model equivalence disappears; if too much is omitted, the projection becomes only a candidate index requiring owner validation.

For two equal canonical structural graphs, `c_B^{-1} ∘ c_A` provides a concrete original-A to original-B vertex correspondence. Multiple correspondences can be valid under automorphisms. A science consumer must either tolerate those choices through explicit coordinate permutation, or label the distinctions that make a particular correspondence necessary. Template reuse can bind changing parameter values; deduplicating completed scientific trials requires agreement of every relevant value, context and outcome dependency.

**Next discriminator:** take two independently authored copies of one repeated equipment/network motif with renamed units, ports and connections. Include two parallel occurrences, a grouped tear decision and one physical-port distinction that must prevent equivalence. Determine whether the incidence encoding yields a useful correspondence and whether the compiler owner can validate mathematical template equality without repeating nearly all preparation. Contrast against existing definition-level reuse; a new graph index earns its machinery only if it reaches additional useful reuse.

## Carry correspondence into Symbolica without losing lexical scope

The current `pse-math::library::exact_evaluator` already gives Symbolica the complete output set and alias definitions, with an explicit ordered parameter list and direct translation. Direct translation resolves aliases as a shared instruction DAG instead of recursively expanding every output. A structural discovery should aim to reuse that prepared mathematical operation, not introduce a second expression optimizer.

Source-checked `FunctionMap` makes an important distinction:

- Ordinary registered function bodies see global evaluator inputs and explicit arguments, with arguments shadowing matching globals.
- `add_aliases` registers argument-free definitions that always inline in the **caller's local scope**; function-shaped aliases match their full argument list as fixed tags.

Therefore a graph-derived correspondence cannot be implemented safely by renaming printed symbols or turning every alias into an ordinary reusable function. The alias/function distinction changes capture. A useful pure mathematical template can instead have an owner-validated canonical formal order; each instance supplies a mapping from physical inputs into that order, and maps outputs back. Existing formal-pool atoms are derived representation identities, not process entities. If mathematical remapping is required, use Symbolica-native mapping on qualified expressions with explicit lexical/capture rules rather than formatted names.

`execution::LocalPartials::new` already differentiates each named definition with respect to its supported local formal inputs and stores demanded first/second partials as aliases. A reused template must transport its input axes, selected outputs, directions and weights consistently. Selected columns may become different ordinal columns after permutation; Hessian row and column axes both move; provider/context tokens and derivative eligibility do not become invariant merely because arithmetic is symmetric. Source obligations still identify the original instance's occurrences when a calculation fails.

**Potential removed work:** repeated local derivative construction, repeated alias/function preparation, duplicate exact coefficient conversion and duplicate Horner/CPE preparation for equivalent qualified templates. **Not established:** that these costs currently repeat after existing caches and instance sharing. The first investigation should trace one pair of real consumers through the current preparation owner and name the exact work that survives existing reuse.

## Bounded graph synthesis followed by typed mathematics

Graphica generation can start from external legs or from an internal seed when the external list is empty. Signatures are repeatable allowed incident multisets, not a fixed equipment inventory. This enables a proposed workflow for bounded open/closed network design:

1. Specify a finite equipment/vertex inventory bound and permitted typed attachment roles. Structural generation works on this deliberately constrained design space.
2. Use built-in maxima and safe partial filters to prune permanently invalid completions, and completed-candidate predicates for exact counts or properties. Named-port and equipment-kind constraints need an explicit encoding because internal generated nodes use `Default` and signature order is not port order.
3. Convert a candidate into ordinary authored declarations with explicit identities, ports, bindings, equations and policies. Run existing model/physical admission. A generated graph supplies no quantity proof, supported provider, causality or feasible process.
4. For admitted candidates, use Symbolica's appropriate exact representation for a selected screen: supported linear extraction, small polynomial consistency/elimination or exact identity checks, retaining parameter/domain conditions. Inconclusive or generic coverage is not proof of infeasibility.
5. Reuse compatible admitted mathematical templates and prepare joint evaluators for repeated numerical investigation. Preserve original-coordinate residual/derivative checks and all failed/refused candidates; graph symmetry alone cannot collapse scientifically distinct trials.

This is a coherent new-functionality hypothesis rather than a missing current caller. A restricted circulation network with an independently known small candidate population is a useful first discriminator. A general equipment-synthesis framework with arbitrary equations is not justified by the graph API alone.

Several library details matter to feasibility:

- `max_loops` does not bound degree-two subdivisions or arbitrarily long cycles. The API's presence-of-termination-control check is weaker than a finite-family proof; use an explicit size/inventory bound for the candidate family.
- Safe partial filters operate using the completed-vertex boundary. Missing required motifs cannot be rejected while they can still arise. Exact cycle count should be queried only on completed connected graphs, because `num_loops` computes `E-V+1`.
- Progress sees one newly inserted canonical graph, not the map or group size. Returning false retains the triggering graph in `Err(partial_map)`. Abortion is polled only after a connected completed candidate; neither hook proves bounded latency through recursive search or canonicalization. Results and canonical search leaves remain retained memory, not streamed ownership.
- The zero-flow bridge predicate is topology-specific, not physical conservation. If any valence-one vertex exists it asserts index zero is external; closed generation with internally produced leaves needs particular care. Bridge counts omit terminal edges, and default zero-flow restrictions can exclude otherwise connected closed candidates with bridges.

## Corrections and residual addendum territory

**Corrected exact-version fact:** locked `get_edge_automorphism_group_size` doubles only **undirected** self-loops. Identical directed self-loops contribute `m!`; identical undirected self-loops contribute `2^m m!`. The prior published account's statement including directed loops was wrong. Total group size includes edge permutations, whereas exposed orbit generators are vertex permutations. A consumer needing explicit transformations cannot derive all edge actions from the total cardinality.

`HiddenData` excludes auxiliary metadata from equality/order/hash, but displays it and does not canonically order or merge it across symmetric objects. Structural template keys must not be mistaken for canonical serialization of complete provenance. The current flowsheet declaration's semantic identity must remain separately addressable even where a derived template ignores renameable occurrence IDs.

Direction mutation changes only a flag. Converting a directed edge to undirected can leave reversed endpoint storage; `canonize_edges` sorts but does not normalize endpoints. Full canonicalization reconstructs through normalizing `add_edge`, so standalone edge symmetry after mutation needs care. Prefer constructing the intended labeled incidence representation rather than toggling directions in place during an equivalence proof.

Canonicalization accepts disconnected and empty graphs, useful for a whole model containing isolated units; generation emits connected graphs. Do not silently omit isolated equipment when constructing a template key. Result-map iteration is not a stable enumeration order. `Empty`, edge helpers, `Display`, cloneable settings and DOT/Mermaid export are useful incidental integration aids, but none establishes durable encoding, sanitized output or a scientific interchange format.

## Source basis and next assessment

Repository seams inspected: `crates/pse-structural/src/flowsheet.rs` (declarations, physical bindings, semantic projection), compiler `workspace/modeling/executable.rs` and its `dependencies.rs` (resolved mathematical and guard reads, formal order, region construction), `crates/pse-math/src/library.rs` (formal pool, joint exact evaluator), `execution.rs::LocalPartials`, and maintained `vendor/symbolica/src/evaluate/function_map.rs`. Exact Graphica source is the Cargo.lock registry package `graphica-3.0.1/src/lib.rs`, especially `CanonicalForm`, canonicalization, `generate`/`generate_impl`, `has_zero_flow_bridges`, `set_directed` and edge-symmetry counting. Source inspection establishes these contracts; no generated template, preservation experiment or quantitative benefit was demonstrated.

The next deeper assessment should choose either one repeated motif with a real preparation consumer or one finite synthesis population, then inspect the complete structural-to-mathematical operation. It should distinguish what becomes shared, what remains per-instance, what changes invalidate the shared product, and whether graph conversion/search removes more machinery than it adds. Required guard/source/provider semantics remain with their actual owners; incumbent representations may be redesigned when a better target preserves their meaning.

## Prefer the existing indexed mathematical canonicalizer where it fits

A separate alternative is narrower than the flowsheet projection above: a mathematical motif already expressed as indexed Symbolica functions may not need an application-owned graph projection at all. Maintained `tensors.rs::canonize_tensor_product` strips factors unrelated to the specified indices, builds an internal `TensorGraphNode` graph, handles open indices, rejects indices contracted more than once, invokes Graphica `canonize`, and reconstructs the mathematical form through the returned vertex mapping. `AtomCore::canonize_tensors` returns canonical mathematics with external and dummy index information. The source tests `open_index` and `nested_sum` exercise these distinctions; they were read, not executed in this inquiry.

For an actual repeated indexed constitutive contraction, this could let the supplier own both structural canonicalization and the algebraic representation, instead of building an independent motif graph followed by reconstituting equivalent scalar mathematics. The next discriminator is a named sum/contraction whose dummy indices truly may be renamed within declared groups and whose open indices preserve physical component identities. Mathematical tensor symmetry and dimension/group constraints must be declared lawfully; arbitrary physical ports, species labels, provider effects and source guards do not become interchangeable dummy indices. The contracted-more-than-once restriction also excludes unrestricted graph incidence encodings masquerading as tensor expressions.

Compare three complete routes for that operation: current scalar/formal preparation, native indexed canonicalization followed by joint evaluator preparation, and an explicit structural candidate index plus owner validation. Avoid representing a whole flowsheet as a tensor merely to obtain a graph canonicalizer; the native tensor route earns its advantage only when the scientific operation is already an indexed mathematical object. This preserves the inquiry's shared mathematical objective while considering a supplier-native composition beyond the first general graph API.
