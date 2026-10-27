# petgraph and rustworkx-core

Realizations considered:
- installed `petgraph` 0.8.3 (released 2025-09-30; the newest release on 2026-10-27);
- installed `rustworkx-core` 0.18.1 (released 2026-07-29; the newest release).

Research date 2026-10-27. Sources:
- `~/.cargo/registry/src/*/petgraph-0.8.3/src/{algo/tred.rs, algo/matching.rs, algo/scc/*,
  graph_impl/mod.rs}`
- `~/.cargo/registry/src/*/rustworkx-core-0.18.1/src/{dag_algo.rs, traversal/mod.rs,
  max_weight_matching.rs}`

## Types

### petgraph/dag-transitive-reduction-closure
- **Need it serves:** all-pairs reachability or a minimal edge set of a DAG.
- **Recognizable code shapes:**
  - A loop over a topological order that unions predecessor `BTreeSet`s.
  - Repeated BFS/DFS from every node.
- **Mechanisms and APIs:**
  - `algo::tred::dag_to_toposorted_adjacency_list(g, &toposort) -> (UnweightedList<Ix>,
    revmap)`.
  - `algo::tred::dag_transitive_reduction_closure(&list) -> (reduction, closure)`. This is
    Habib–Morvan–Rampon, with `FixedBitSet` marks and neighbours in topological order.
  - Ordering input comes from `algo::toposort` or a caller-supplied order.
- **Obligations discharged:** the closure and the reduction as adjacency lists in topological
  rank space.
- **Residual caller obligations:**
  - Building the DAG. For block structures, condensation comes first.
  - Mapping ranks back to identities.
  - Restricting the closure to a node subset. For example, "which source parameters does each
    unknown depend on" needs closure rows filtered to parameter nodes.
- **Semantic conditions:**
  - The result is deterministic for a given toposort.
  - Worst-case time is O(|V|³) and the closure is O(|V|²) in size. A subset-restricted
    propagation (sets of sources only) can be much smaller than the full closure.
  - There are no cancellation or allocation hooks.
- **Version / build conditions:** petgraph 0.8.x.
- **Limits and negative results:**
  - The input must be acyclic. Cyclic graphs need `condensation` first.
  - There is no source-restricted closure, and no `petgraph` support for propagating labels or
    sets.
- **Evidence:** Interface-checked.

### rustworkx-core/deterministic-dag-order-and-reachability
- **Need it serves:** topological orders with caller-defined tie-breaking, plus per-node
  ancestor and descendant sets.
- **Recognizable code shapes:**
  - A topological sort followed by a manual tie-break.
  - Hand-written DFS for ancestors.
- **Mechanisms and APIs:**
  - `dag_algo::lexicographical_topological_sort(dag, key_fn, reverse, initial)`.
  - `dag_algo::layers`.
  - `traversal::{ancestors, descendants, bfs_successors, bfs_predecessors, dfs_edges,
    depth_first_search, breadth_first_search}`.
  - `connectivity::*`.
- **Obligations discharged:**
  - A key-ordered topological order. A key such as an identity gives a reproducible order.
  - Single-node reachability.
- **Residual caller obligations:** computing a closure for all nodes means running
  `ancestors`/`descendants` once per node.
- **Semantic conditions:** tie-breaking is deterministic and defined by the key. Traversals
  allocate internally.
- **Version / build conditions:** 0.18.1.
- **Limits and negative results:**
  - No transitive closure or reduction function exists. The search covered the `pub fn`
    listings in `dag_algo.rs`, `traversal/`, `connectivity/` and `transitivity.rs`.
    `transitivity.rs` computes the clustering-coefficient "transitivity", not a closure.
  - There is no bipartite Hopcroft–Karp. `max_weight_matching` is a general-graph weighted
    matcher.
- **Evidence:** Interface-checked.

### petgraph/matching (negative for stack-safe bipartite matching)
- **Need it serves:** maximum matching.
- **Mechanisms and APIs:** `algo::{greedy_matching, maximum_matching}` (Gabow, O(|V|³)) on
  undirected graphs.
- **Limits and negative results:** `maximum_matching` uses a recursive `augment_path`. It is a
  general-graph algorithm, and on bipartite incidences it is asymptotically slower than
  Hopcroft–Karp. It does not answer the stack-safety need.
- **Evidence:** Interface-checked.

### petgraph/storage-layout-sizing
- **Need it serves:** bytes retained by a `Graph`, computed before or after construction.
- **Recognizable code shapes:**
  - `size_of::<petgraph::graph::Node<N>>()` or `size_of::<Edge<E>>()` times counts.
  - `graph.capacity()`.
- **Mechanisms and APIs:**
  - `Graph::{with_capacity, capacity() -> (nodes, edges), node_count, edge_count}`.
  - The public structs `graph::Node<N, Ix>` and `graph::Edge<E, Ix>`, whose `size_of` tracks the
    real layout.
- **Obligations discharged:** exact element widths and the reserved capacity of the two backing
  `Vec`s.
- **Residual caller obligations:**
  - Payload heap memory.
  - Traversal workspaces (visit maps, stacks).
  - The multiplier for clones.
- **Semantic conditions:** `size_of` follows the compiled layout automatically, so it is not a
  hand mirror. `with_capacity` lets a caller reserve the exact admitted capacity.
- **Version / build conditions:** petgraph 0.8.x.
- **Limits and negative results:** algorithms do not report their workspace size.
- **Evidence:** Interface-checked.

## Current applicability

### [CBO-023](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-023) — Transitive closure loops duplicate graph-library functions
- **Types:** petgraph/dag-transitive-reduction-closure,
  rustworkx-core/deterministic-dag-order-and-reachability.
- **Realization:** installed pins.
- **Conclusion:** ruled out for the reconstruction loop as it stands. It fits with conditions only
  if the propagation is restated as a DAG closure.
- **What it would discharge / what remains local:**
  - The reconstruction loop is a *source-restricted* propagation. It finds, for each unknown, the
    set of parameters reachable through the BTF blocks in elimination order.
  - `dag_transitive_reduction_closure` would compute the full O(V²) closure and then need
    filtering, which costs more than the current walk.
  - The compiler's `dependencies.rs` and `implicit.rs` already use petgraph `Bfs`, reversed
    `Dfs` and `toposort`. Their duplication lies in graph construction and limits, not in the
    algorithms.
- **Prerequisites, configuration or extension:** a block-DAG builder, which no inspected library
  provides from BTF output.
- **Semantic gaps against the CBO's preservation constraints:**
  - `petgraph::algo::toposort` order is DFS- and insertion-dependent.
  - SemanticId tie-breaking needs `lexicographical_topological_sort` with a SemanticId key, which
    is already used in the structural layer.
- **Unsettled premises and what would settle them:** none material.
- **Evidence:** Interface-checked; repository call sites inspected at the published baseline.

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) (part) — Graph layout mirrors
- **Types:** petgraph/storage-layout-sizing.
- **Realization:** installed 0.8.3.
- **Conclusion:** the premise needs sharpening.
  - The inspected sites size petgraph storage with `size_of` over petgraph's public `Node`/`Edge`
    types and `capacity()`, which tracks the library layout.
  - The genuine hand mirrors are `std` BTree node geometry ("eleven key/value slots and twelve
    child links") and traversal-workspace multipliers (`size_of::<[usize; 8]>()` per node or
    edge).
  - petgraph supplies nothing more to replace them.
- **What it would discharge / what remains local:** the workspace multipliers and the BTree
  geometry remain local.
- **Prerequisites, configuration or extension:** none.
- **Semantic gaps against the CBO's preservation constraints:** none.
- **Unsettled premises and what would settle them:**
  - Whether BTree node geometry should move behind one shared helper. This is a local design
    question.
- **Evidence:** Interface-checked. Repository sites inspected:
  - `pse-structural/src/flowsheet.rs` (allocation bounds);
  - `pse-compiler/.../implicit.rs` (`implicit_node_bytes`);
  - `pse-modeling/src/check/phases.rs`.
