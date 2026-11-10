# Graphica: discovery briefing

The integrated investigation develops [structural correspondence and mathematical reuse](mathematical-template-reuse.md), including the existing Symbolica indexed-canonicalization alternative and bounded topology-to-mathematics workflows.

The [advanced-addendum follow-up](addendum-followup.md), inspected at `924b0c3b2`, adds targeted research priorities and exact-source clarifications after the original discovery baseline.

Inspected 2026-11-09 at `d9e82627feec6cea5470dbcf1974fd4e8dfe19df`, including the user-supplied untracked overview and unrelated concurrent Plan72 work. **Interface-checked:** representative repository reads and locked Graphica 3.0.1 source/tests. Directions below are **Proposed** research priorities, not adoption conclusions. No test, benchmark or generated companion reference census was run by this lane.

## Current role and opportunity

Graphica 3.0.1 is already in the lockfile and Symbolica re-exports it as `graph`; this does not establish a direct product use of graph canonicalization. Its distinctive contribution is symmetry-aware equivalence and constrained enumeration. The existing graph consumers instead use petgraph/rustworkx for directed ordering, SCCs, cycles and initialization, and POUNCE for structural incidence. These already delegated algorithms are material counterevidence to a general graph replacement.

[Flowsheet projection](../../../../crates/pse-structural/src/flowsheet.rs) deliberately includes semantic node/port/connection identities, physical quantity/unit contracts, decision groups and bindings. Sorting semantic IDs is not failed isomorphism canonicalization: it implements a different key. [Incidence](../../../../crates/pse-structural/src/incidence.rs) preserves scoped row/column membership and deterministic structural ordering. Any topology-equivalence product must specify what may be renamed and preserve an explicit mapping back to those semantic owners.

## Research priorities

1. **Topology templates shared across independently authored instances.** `canonize` returns a canonical graph, `vertex_map`, orbits and exact group size. Investigate role-labeled graph projection → canonical template → mapped preparation reuse. This could recognize useful repeated structure that semantic IDs intentionally distinguish. A physical port or binding cannot be discarded as incidental metadata merely because the graph stays isomorphic. The discriminating question is whether a concrete repeated-equipment consumer can reuse preparation after all mathematical dependencies and port roles are included, without reconstructing most of that preparation to establish eligibility. A port-as-vertex/edge encoding may preserve roles, but its extra conversion and search burden must be counted. Template reuse may parameterize changing values; deduplicating scientific trials instead requires equal relevant values. Qualify the parameter, axis and port permutations and transported values/derivatives, not just the canonical vertex map.
2. **Symmetry-reduced topology exploration.** `Graph::generate` combines typed incoming/outgoing half-edges, allowed vertex signatures, partial pruning and canonical deduplication. This could enable a new bounded network/flowsheet design exploration workflow rather than replacing an existing caller. Its signature multiset has no named-port ordering, and internally generated node payloads use `Default`; distinct equipment kinds with identical attachment signatures therefore need an explicit representation. Investigate one restricted network family and its complete physical admission after generation. Safe pruning must reject only partial candidates whose every completion is invalid. Numerical feasibility, economics and scientific multiplicity remain outside graph generation.
3. **Orbits for equivalent candidate work.** Canonical orbits identify actual global labeled-graph symmetries, unlike equal degrees or local shape. Investigate whether alternative placements or network decisions are interchangeable after boundaries, parameters, starts, costs and providers are labeled. A symmetry of topology alone does not justify skipping a scientific trial with different data. Group size includes edge permutations and a self-loop convention; it is not automatically a probability weight or count of distinct engineering alternatives.
4. **Compact graph inspection/export.** `to_dot` and `to_mermaid` can provide structure inspection, but do not render or sanitize arbitrary payloads. `HiddenData` ignores metadata in equality/hash/order yet displays it, so it is unsuitable as a secrecy boundary. Investigate only an actual report consumer needing those native structures; converting an already suitable petgraph solely for export is weak value.

## Complete capability disposition

| Overview territory | Discovery disposition and countercondition |
| --- | --- |
| §§1–2: representation, labels, directions, indexing, hidden metadata | Useful foundation for topology/template hypotheses. Mixed directions, parallel edges and self-loops are native. `HiddenData` requires caller-defined metadata aggregation; ordinary `Graph` equality/hash remain index-sensitive. |
| §3: canonicalization/isomorphism | Promising for cross-instance structural equivalence. `canonize_edges` only orders edges under fixed vertices; use that cheaper operation when it suffices. `is_isomorphic` is Boolean; canonical maps supply correspondences. Difficult symmetry search retains extra state. |
| §4: automorphisms/orbits/exact sizes | Promising for symmetry-aware exploration, conditional on full scientific labels. Vertex and edge symmetry factors must not be conflated. |
| §5: generation, half-edge constraints, pruning, progress/abort | Useful new-functionality direction. Generates connected graphs and accumulates a result map, not a streaming result iterator. Abort is checked only at selected completed candidates; partial results must retain incompleteness. |
| §6: trees, chains, bridges/connectivity | Useful within generated topology analysis. Traversal ignores direction and bridges exclude topological leaves; cannot replace directed SCC/ordering or physical-flow analysis. `num_loops` assumes connected input. |
| §7: mutation/lifecycle | Append/backtrack model suits enumeration; less suitable for arbitrary live graph editing. Canonical products require invalidation after identity-relevant edits. Direction toggling does not recover an earlier unsorted orientation. |
| §8: visualization | Conditional developer/report capability; labeling/escaping and rendering remain outside native export. |
| §9: features, traits, concurrency, serialization | Direct use can avoid a Symbolica facade for graph-only consumers, but defaults request Numerica's native arithmetic backend. New direct use alone removes no workspace dependency already needed elsewhere. Serialization flags do not establish graph codec implementations. No native parallel enumerator was observed in inspected source. |
| §§10–14: ecosystem, alternatives, limitations, example and interactions | Keep petgraph/rustworkx/POUNCE at their established roles unless a distinct complete alternative emerges. Combine canonicalization with generation and semantic mapping; do not promote the overview's unexecuted example to test evidence. |

## Sources and handoff

The [full overview](../../../libraries/scientific_calcs/graphica.md) was read in full. Exact locked source is `graphica-3.0.1/src/lib.rs` from the registry identified by Cargo.lock; relevant functions are `Graph::{canonize,canonize_impl,is_isomorphic,generate,generate_impl,get_spanning_tree,get_edge_automorphism_group_size,to_dot,to_mermaid}` and `GenerationSettings`. The original manifest has Numerica defaults forwarding and optional serialization dependencies. Inline tests `iso_check_permutations` and `generate_with_filter` were read, not executed. [Versioned source](https://docs.rs/crate/graphica/3.0.1/source/src/lib.rs) supplies a reproducible external route.

Next work should identify a named template-reuse or topology-exploration operation, then assess its semantic labels, mapping and total conversion/search/preparation work. Existing Plan66 work retains its own dispositions. No production or implementation authority follows from this briefing.

See [canonicalization and generation boundaries](canonical-generation.md) and the [integrated discovery](../symbolica/briefing.md).

Fresh defining-package API exports and their limits are recorded in the [integrated evidence account](../symbolica/coverage.md#generated-evidence-and-limitations).
