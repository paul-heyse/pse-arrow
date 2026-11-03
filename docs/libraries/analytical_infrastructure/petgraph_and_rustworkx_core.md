# Petgraph and rustworkx-core — Comprehensive Rust Library Reference

Research date: October 26, 2026

Versions covered

- petgraph 0.8.3 — released September 30, 2025.
- rustworkx-core 0.18.1 — released July 29, 2026.
- Compatibility: rustworkx-core 0.18.1 depends on `petgraph ^0.8`, making these versions compatible under Cargo's version resolution rules.

These are the latest published stable releases verified against their Rust documentation and crate records. The overview focuses on released Rust capabilities rather than unreleased development-branch functionality.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



## 1. Architectural orientation

petgraph and rustworkx-core are complementary components of a general-purpose Rust graph computation stack.

Petgraph provides the foundational graph representations, graph operations, traversal abstractions, and a substantial collection of algorithms. Rustworkx-core builds on those foundations, adding specialized graph algorithms, structural transformations, network analysis, graph generation, and algorithms useful for optimization and scheduling.

Conceptual layering, not a requirement to adopt all three layers. Both crates can be used independently by Rust applications, with rustworkx-core consuming petgraph-compatible graph types.

### Division of responsibilities

| Dimension               | petgraph                                                   | rustworkx-core                                                             |
| ----------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------- |
| Primary role            | Graph data structures and foundational operations          | Extended graph algorithm library                                           |
| Graph storage           | `Graph`, `StableGraph`, `GraphMap`, `MatrixGraph`, `Csr`   | Primarily consumes petgraph graphs                                         |
| Graph traversal         | BFS, DFS, topological traversal, visitor traits, adaptors  | Additional traversal algorithms and utilities                              |
| Shortest paths          | Dijkstra, A\*, Bellman–Ford, SPFA, Floyd–Warshall, Johnson | Additional shortest-path enumeration, distance-matrix and related routines |
| Structural analysis     | Components, SCCs, cycles, isomorphism, dominators          | Centrality, cycle analysis, k-core, connectivity and cuts                  |
| Graph transformations   | Filtering, mapping, composition, DAG condensation          | Contraction, graph extension traits, line graphs and related operations    |
| Optimization algorithms | MST, max flow, matching, clique and Steiner algorithms     | Weighted matching, coloring, token swapping, Steiner and other algorithms  |
| Graph generation        | Limited, partly feature-gated generators                   | Extensive deterministic and random graph generators                        |
| Ecosystem role          | Reusable graph abstraction underlying numerous Rust crates | Algorithmic core shared with the rustworkx project                         |

This is not a strict separation: the crates overlap substantially in algorithms, and certain functions in rustworkx-core originated as functionality required by the Python rustworkx project.

The most consequential architectural capability is composition through petgraph's trait interfaces. Many algorithms consume graph capabilities rather than a specific concrete graph type. A single graph representation can therefore support multiple algorithm families, and filtered or adapted views can sometimes be analyzed without constructing a second graph.

There are two important qualifications:

1. Trait support varies by graph representation and algorithm; not every algorithm accepts every graph type or adaptor.
2. Rustworkx-core is a native Rust library, not the Python rustworkx API exposed unchanged to Rust. Its Rust module coverage and function signatures must be assessed independently.

Primary documentation:

- petgraph: [https://docs.rs/petgraph/0.8.3/petgraph/](https://docs.rs/petgraph/0.8.3/petgraph/?utm_source=chatgpt.com)
- rustworkx-core: [https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/?utm_source=chatgpt.com)
- petgraph algorithms: [https://docs.rs/petgraph/0.8.3/petgraph/algo/](https://docs.rs/petgraph/0.8.3/petgraph/algo/?utm_source=chatgpt.com)
- rustworkx-core source and project: [https://github.com/Qiskit/rustworkx](https://github.com/Qiskit/rustworkx?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

## 2. Petgraph: graph representation and structural model

### 2.1 Selecting a graph representation

Petgraph 0.8.3 provides five principal graph representations. The choice determines memory layout, graph identity, edge multiplicity, mutation characteristics, and which generic algorithms are directly supported.

| Representation           | Internal model                         | Distinctive capabilities                                                            | Important tradeoffs                                                            |
| ------------------------ | -------------------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `Graph<N,E,Ty,Ix>`       | Indexed adjacency lists                | General-purpose dynamic graphs, arbitrary node/edge data, multigraph support        | Deletions can renumber other nodes and edges                                   |
| `StableGraph<N,E,Ty,Ix>` | Indexed adjacency lists with vacancies | Surviving indices remain stable across deletions                                    | Sparse index space, additional bookkeeping and memory                          |
| `GraphMap<N,E,Ty>`       | Hash-indexed adjacency structure       | Node values are identifiers, fast node/edge membership queries                      | Node IDs must be `Copy + Eq + Hash + Ord`; no parallel edges                   |
| `MatrixGraph`            | Dense adjacency matrix                 | Direct edge lookup, dense connectivity, configurable representation of absent edges | Quadratic space; no parallel edges                                             |
| `Csr<N,E,Ty,Ix>`         | Compressed sparse row                  | Compact adjacency storage and efficient neighbor scans                              | Restricted mutation, specialized construction and less complete trait coverage |

All support directed or undirected graph types. However, directedness, edge multiplicity and graph-index semantics are separate design choices.

Documentation:

[https://docs.rs/petgraph/0.8.3/petgraph/graph/](https://docs.rs/petgraph/0.8.3/petgraph/graph/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/stable_graph/](https://docs.rs/petgraph/0.8.3/petgraph/stable_graph/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/graphmap/](https://docs.rs/petgraph/0.8.3/petgraph/graphmap/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/matrix_graph/](https://docs.rs/petgraph/0.8.3/petgraph/matrix_graph/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/csr/](https://docs.rs/petgraph/0.8.3/petgraph/csr/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+4

### 2.2 Graph: the general-purpose dynamic multigraph

`Graph<N, E, Ty, Ix>` stores nodes and edges in indexed arrays with adjacency links. `NodeIndex` and `EdgeIndex` provide compact handles, independent of the associated node or edge values.

Principal capabilities include:

- Directed and undirected graphs, including self-loops and parallel edges.
- Arbitrary Rust types as node and edge payloads, including structs, enums, shared references and domain-specific types.
- Insertion, removal, edge updates, adjacency iteration, directed incoming/outgoing traversal, and graph-wide iteration.
- Capacity management, graph construction from edge lists, and node/edge payload mapping.
- `retain_nodes`, `retain_edges`, `filter_map`, `map`, and their consuming equivalents.
- Fallible insertion and update operations using `try_*` methods, alongside methods that can panic on invalid indices or capacity limits.

The principal limitation is its swap-removal identity model. Removing an element can move the last stored element into the vacant slot. Consequently, an external reference to a graph index may no longer denote the same element after mutations.

This makes `Graph` especially suitable for algorithmic working graphs whose identifiers can remain local to the computation. Long-lived external identity requires additional consideration.

One easily overlooked distinction: `add_edge` permits parallel edges; `update_edge` searches for an existing edge and updates it or adds a new one. The latter does not retroactively normalize an existing multigraph.

### 2.3 StableGraph: stable surviving identifiers

`StableGraph` addresses the index invalidation problem by retaining vacant slots after removals.

This provides a useful separation between structural modification and identity of surviving objects. For example, an analysis can remove a subgraph without renumbering every unrelated node.

Its capabilities include:

- Stable indices for surviving nodes and edges.
- Vacancy-aware node and edge iteration.
- Indexed access, filtering, mapped transformations and selective retention.
- `node_bound` and related indexing concepts that distinguish allocated index space from live element count.
- Conversion from `Graph` without changing indices, and conversion back to `Graph` with compaction.
- Graph transformations such as `filter_map` that can preserve surviving indices.

Stable indices are not permanent, generation-checked object identities. A deleted identifier is invalidated, and the implementation can reuse vacant slots. An application requiring globally unique or historical identities still needs an appropriate identity scheme.

Another consequential distinction is the difference between `node_count()` and `node_bound()`. Algorithms allocating work arrays indexed by nodes may need space proportional to the bound rather than the number of live nodes.

This matters particularly for graph algorithms using dense vectors, bitsets, or direct index addressing.

Source:

[https://docs.rs/petgraph/0.8.3/petgraph/stable_graph/struct.StableGraph.html](https://docs.rs/petgraph/0.8.3/petgraph/stable_graph/struct.StableGraph.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 2.4 GraphMap: graph relationships over existing identifiers

`GraphMap` uses the node's value as its identifier instead of creating separate `NodeIndex` handles.

An edge can be represented directly as a relation between existing keys, such as integer IDs or compact enums.

This eliminates the need for an external ID-to-index mapping in many cases.

Important properties:

- Hash-backed edge existence testing with expected constant-time behavior.
- `add_edge` automatically inserts missing endpoint nodes.
- Edge insertion updates the existing edge payload when the same endpoint pair already exists.
- Edge identifiers are effectively endpoint pairs; parallel edges are not supported.
- Node values must be cheap to copy and satisfy key ordering and hashing requirements.
- Optional parallel iteration over nodes and edges is available with Rayon support.

The principal tradeoff is that `GraphMap` cannot independently store a large, non-`Copy` node object as the identity-bearing node payload.

A natural alternative is to use small IDs for the graph and maintain richer node records elsewhere.

Conversion can lose information: `GraphMap::from_graph` merges nodes having equal weights and retains only the last parallel edge between a pair. Conversion back to `Graph` assigns new indices. These are semantic conversions, not identity-preserving changes of storage layout.

Source:

[https://docs.rs/petgraph/0.8.3/petgraph/graphmap/struct.GraphMap.html](https://docs.rs/petgraph/0.8.3/petgraph/graphmap/struct.GraphMap.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 2.5 MatrixGraph and Csr: contrasting computational representations

`MatrixGraph` uses an adjacency matrix, requiring \\(O(V^2)\\) storage. Its distinctive functionality includes a configurable `Nullable` representation for edge absence, with `Option<E>` as the default and `NotZero<E>` available for suitable sentinel-based representations.

It may be advantageous where edge-existence queries or dense adjacency operations dominate. Undirected graphs store only the triangular portion of the matrix. The representation does not support parallel edges.

`Csr` instead stores sparse adjacency information compactly by source node. It provides efficient contiguous neighbor and edge slices, enabling locality-friendly graph scans.

Its `from_sorted_edges` constructor accepts sorted, unique edge sequences and runs in \\(O(V+E)\\). For undirected CSR construction, both directions must be explicitly represented in the supplied sorted edge sequence.

CSR is not entirely immutable—it permits adding nodes and edges—but dynamic edge insertion is substantially less attractive than bulk construction. The documentation gives \\(O(VE)\\) for adding all edges through its incremental insertion path, even when supplied in row-major order.

The practical distinction is:

- `MatrixGraph` favors dense relationships and direct adjacency access.
- `Csr` favors relatively static sparse relationships and repeated adjacency scans.
- `Graph` and `StableGraph` favor more general structural mutation.

Neither representation should be assumed to expose exactly the same trait interfaces as `Graph`. In particular, CSR does not implement all direction-aware visitation traits.

Sources:

[https://docs.rs/petgraph/0.8.3/petgraph/matrix_graph/struct.MatrixGraph.html](https://docs.rs/petgraph/0.8.3/petgraph/matrix_graph/struct.MatrixGraph.html?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/csr/struct.Csr.html](https://docs.rs/petgraph/0.8.3/petgraph/csr/struct.Csr.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

## 3. Petgraph's generic graph abstraction

This is one of petgraph's most architecturally significant capabilities.

Instead of requiring every algorithm to operate on `Graph`, petgraph defines a collection of relatively small traits describing operations on graphs.

### 3.1 Graph traits as a capability interface

The `visit` and `data` modules provide the core abstractions.

| Trait family                  | Important traits                                                           | Capabilities exposed                                                 |
| ----------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Identity and graph properties | `GraphBase`, `GraphProp`, `GraphRef`                                       | Node/edge identifier types, directedness, shared graph access        |
| Iteration                     | `IntoNodeIdentifiers`, `IntoNodeReferences`, `IntoEdgeReferences`          | Enumerating graph elements                                           |
| Adjacency                     | `IntoNeighbors`, `IntoNeighborsDirected`, `IntoEdges`, `IntoEdgesDirected` | Accessing relationships and their direction                          |
| Indexing                      | `NodeIndexable`, `NodeCompactIndexable`, `EdgeIndexable`                   | Mapping graph IDs into integer index spaces                          |
| Traversal state               | `Visitable`, `VisitMap`                                                    | Creating and updating visited-node sets                              |
| Data                          | `Data`, `DataMap`, `DataMapMut`                                            | Associated node and edge types, immutable and mutable payload access |
| Construction                  | `Build`, `Create`, `FromElements`                                          | Abstract graph creation and construction from graph elements         |

Generic algorithms can declare precisely which capabilities they need. For instance, a traversal may require only adjacency iteration and a visited-node map, whereas an algorithm building an indexed work array may additionally require `NodeIndexable`.

These distinctions matter because an algorithm's generic signature is a capability contract, not a promise that every petgraph graph implements it.

The crate's documentation explicitly acknowledges that trait-based generalization remains incomplete for some algorithms.

Documentation:

[https://docs.rs/petgraph/0.8.3/petgraph/visit/](https://docs.rs/petgraph/0.8.3/petgraph/visit/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/data/](https://docs.rs/petgraph/0.8.3/petgraph/data/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 3.2 Graph adaptors: compute over views rather than copies

Petgraph includes adaptors that alter the graph presented to an algorithm without necessarily materializing another graph.

| Adaptor             | Behavior                                                                    |
| ------------------- | --------------------------------------------------------------------------- |
| `NodeFiltered`      | Exposes only nodes accepted by a predicate and their eligible relationships |
| `EdgeFiltered`      | Exposes only edges accepted by a predicate                                  |
| `Reversed`          | Reverses the direction of edges exposed by the graph                        |
| `UndirectedAdaptor` | Exposes directed edges through an undirected traversal model                |

These are valuable because a graph can remain the authoritative representation while an algorithm operates on a particular subset or interpretation.

Consider a graph containing dependency edges, optional relationships and different relationship classifications. An algorithm that only requires one relationship category may operate through an `EdgeFiltered` view rather than receiving a newly constructed graph.

This can eliminate redundant storage, conversion costs, and synchronization of duplicate graph structures.

However, filters and adaptors do not automatically preserve all traits, algorithm assumptions or structural guarantees. Some algorithms may still require a concrete copy, an indexing remap, or a representation implementing a particular interface.

Documentation:

[https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.NodeFiltered.html](https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.NodeFiltered.html?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.EdgeFiltered.html](https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.EdgeFiltered.html?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.Reversed.html](https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.Reversed.html?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.UndirectedAdaptor.html](https://docs.rs/petgraph/0.8.3/petgraph/visit/struct.UndirectedAdaptor.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 3.3 Stateful traversal as an extensibility mechanism

Petgraph provides `Bfs`, `Dfs`, `DfsPostOrder` and `Topo` traversal types, along with a callback-based `depth_first_search` interface that exposes DFS events and traversal control.

A useful characteristic of the walker-based visitors is that they do not retain a borrow of the graph throughout the entire traversal; the graph is supplied to their `next` operation.

This permits traversal logic to interact with graph-associated data between steps, subject to Rust's borrowing rules and the traversal's correctness assumptions.

`DfsPostOrder` is useful for dependency ordering and tree reductions; `Topo` exposes topological traversal of directed graphs; and event-driven DFS supports custom discovery and completion behavior.

These components can replace bespoke traversal engines when custom computation is more important than implementing a new search algorithm.

### 3.4 Acyclic: enforce DAG invariants during mutation

`petgraph::acyclic::Acyclic<G>` is an especially important advanced abstraction.

It wraps an underlying directed graph and maintains the DAG invariant as edges are introduced, rather than relying exclusively on a separate cycle-detection pass afterward.

Its implementation uses a dynamically maintained topological ordering based on the Pierce–Kelly algorithm.

Available operations include:

- `try_from_graph` to validate and wrap an existing graph.
- `try_add_edge` and `try_update_edge` to preserve acyclicity.
- `is_valid_edge` to check whether adding an edge would create a cycle.
- `get_position`, `at_position`, `nodes_iter` and `range` for access to the maintained topological ordering.

This changes the architecture of a DAG-based computation: cycle prevention and topological ordering can be responsibilities of the graph abstraction itself.

The invariant is maintained through the wrapper's supported mutation interface. Obtaining the underlying graph does not authorize arbitrary mutation without considering the invariant.

Failed cycle-producing additions are represented by dedicated errors through the fallible methods, although some generic construction interfaces can panic.

Source:

[https://docs.rs/petgraph/0.8.3/petgraph/acyclic/struct.Acyclic.html](https://docs.rs/petgraph/0.8.3/petgraph/acyclic/struct.Acyclic.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 4. Petgraph's algorithmic capabilities

Petgraph 0.8.3 contains a broader algorithm collection than just BFS, DFS, Dijkstra and minimum spanning trees.

### 4.1 Pathfinding and weighted graph computations

| Algorithm      | Implementation                                            | Distinctive consideration                                                         |
| -------------- | --------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Dijkstra       | `dijkstra`, `bidirectional_dijkstra`, `with_dynamic_goal` | Nonnegative edge costs; goal-directed variants                                    |
| A\*            | `astar`                                                   | Heuristic-guided search; optimality depends on heuristic conditions               |
| Bellman–Ford   | `bellman_ford`, `find_negative_cycle`                     | Supports negative edges and detects reachable negative cycles                     |
| SPFA           | `spfa`                                                    | Queue-based Bellman–Ford variant; worst-case behavior can be poor                 |
| Floyd–Warshall | `floyd_warshall`, `floyd_warshall_path`                   | All-pairs distances and path reconstruction                                       |
| Johnson        | `johnson`, `parallel_johnson`                             | Sparse all-pairs shortest paths, including negative edges without negative cycles |
| Kth shortest   | `k_shortest_path`                                         | Computes the cost of the kth shortest route, not a list of routes                 |
| Simple paths   | `all_simple_paths`, `all_simple_paths_multi`              | Enumerates loop-free paths, potentially exponential output                        |

An important recent addition is `bidirectional_dijkstra`, introduced in petgraph 0.8.3, together with `with_dynamic_goal`, which permits more flexible termination logic.

Johnson's algorithm, added in 0.8.2, is another notable capability: it provides an alternative to Floyd–Warshall for all-pairs shortest paths on suitable sparse graphs. The Rayon-enabled parallel variant can distribute shortest-path computations across sources.

For a shortest-path algorithm with potentially negative edge costs, Dijkstra and A\* should not simply be substituted for Bellman–Ford or Johnson.

Also, the `k_shortest_path` interface returns a mapping from nodes to kth-shortest path costs. It is not equivalent to Yen's algorithm producing the first k explicit simple paths.

Sources:

[https://docs.rs/petgraph/0.8.3/petgraph/algo/dijkstra/](https://docs.rs/petgraph/0.8.3/petgraph/algo/dijkstra/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/johnson/](https://docs.rs/petgraph/0.8.3/petgraph/algo/johnson/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/k_shortest_path/fn.k_shortest_path.html](https://docs.rs/petgraph/0.8.3/petgraph/algo/k_shortest_path/fn.k_shortest_path.html?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/simple_paths/](https://docs.rs/petgraph/0.8.3/petgraph/algo/simple_paths/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 4.2 Connectivity and structural analysis

Petgraph includes connected component counting, strongly connected components using Kosaraju and Tarjan algorithms, graph condensation, cycle detection, bipartiteness testing, bridge detection and articulation points.

Several capabilities are particularly consequential.

Strongly connected component condensation replaces every SCC in a directed graph with a single node, creating a directed acyclic condensation graph. This enables algorithms designed for DAGs to operate over the dependency structure between cyclic groups.

Dominators, provided through `algo::dominators::simple_fast`, identify nodes that must be traversed on every path from a designated root to another node. Unlike ordinary reachability, domination expresses mandatory control-flow structure.

Articulation points and bridges identify nodes and edges whose removal disconnects an undirected graph. These are structural criticality measures, distinct from centrality scores.

Transitive reduction and closure, provided by `algo::tred`, expose the difference between direct edges and reachability. For DAGs, transitive reduction can remove redundant edges while preserving the same reachability relation.

Together, these algorithms can support dependency simplification, control-flow analysis and decomposition without custom foundational graph logic.

Sources:

[https://docs.rs/petgraph/0.8.3/petgraph/algo/scc/](https://docs.rs/petgraph/0.8.3/petgraph/algo/scc/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/dominators/](https://docs.rs/petgraph/0.8.3/petgraph/algo/dominators/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/tred/](https://docs.rs/petgraph/0.8.3/petgraph/algo/tred/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/articulation_points/](https://docs.rs/petgraph/0.8.3/petgraph/algo/articulation_points/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 4.3 Network optimization, matching and graph structure

Petgraph also implements:

| Problem family               | Notable capabilities                                         |
| ---------------------------- | ------------------------------------------------------------ |
| Spanning structures          | Kruskal-style `min_spanning_tree`, Prim's MST                |
| Flows                        | Ford–Fulkerson and Dinic maximum-flow algorithms             |
| Matchings                    | Greedy matching and maximum-cardinality matching             |
| Clique analysis              | Bron–Kerbosch maximal clique enumeration                     |
| Steiner structures           | Kou-based Steiner tree computation                           |
| Graph isomorphism            | Full and subgraph isomorphism; node/edge matching predicates |
| Graph coloring               | DSATUR coloring                                              |
| Ranking                      | PageRank, including a parallel variant                       |
| Cycle-breaking approximation | Greedy feedback arc set                                      |

These are not interchangeable.

Maximum-cardinality matching optimizes the number of matched edges, while maximum-weight matching optimizes their aggregate weight. Petgraph's matching algorithms therefore complement, rather than replace, rustworkx-core's weighted matching algorithm.

Likewise, maximal cliques and maximum cliques are different: a maximal clique cannot be extended by adding another vertex, while a maximum clique has the greatest possible size. Enumeration of maximal cliques can produce exponentially many results.

Petgraph's Steiner implementation uses Kou's algorithm and accepts an undirected connected graph and terminal set. It is an approximation-oriented method rather than a general exact Steiner optimizer.

Isomorphism support is also significant: in addition to structural equivalence testing, petgraph provides matching-predicate variants and subgraph isomorphism enumeration. Unlike a simple graph hash, these can establish structural correspondence, but the search can be computationally expensive.

Documentation:

[https://docs.rs/petgraph/0.8.3/petgraph/algo/matching/](https://docs.rs/petgraph/0.8.3/petgraph/algo/matching/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/maximum_flow/](https://docs.rs/petgraph/0.8.3/petgraph/algo/maximum_flow/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/isomorphism/](https://docs.rs/petgraph/0.8.3/petgraph/algo/isomorphism/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/maximal_cliques/](https://docs.rs/petgraph/0.8.3/petgraph/algo/maximal_cliques/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/algo/steiner_tree/](https://docs.rs/petgraph/0.8.3/petgraph/algo/steiner_tree/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

## 5. Rustworkx-core: extended graph algorithms

Rustworkx-core does not introduce a competing general-purpose graph storage engine. Its principal value is an extensive algorithm collection that operates on petgraph-compatible graph types, supplemented by graph mutation traits and generator utilities.

The public Rust API is organized into modules for centrality, connectivity, shortest paths, DAGs, coloring, matching, graph transformations, traversal, geometry and generators.

The discussion below distinguishes genuinely additional functionality from algorithms that substantially overlap with petgraph.

### 5.1 Shortest paths, route enumeration and distance matrices

Rustworkx-core provides `dijkstra`, `astar`, `bellman_ford`, `negative_cycle_finder`, `k_shortest_path`, `all_shortest_paths`, `single_source_all_shortest_paths` and `distance_matrix`.

The important differences from petgraph are not simply algorithm names.

Fallible weight callbacks. Several rustworkx-core path algorithms accept edge-cost callbacks returning `Result<T, E>`. This permits weight evaluation failures to propagate through the algorithm instead of requiring every cost to be available as an infallible value. It can be particularly useful when edge costs are calculated from structured attributes.

Configurable result storage. The `DistanceMap` trait abstracts the destination for some shortest-path results. `dictmap::DictMap` is an accompanying hash-based map alias combining `IndexMap` and `Foldhash`. Certain algorithms also accept optional path-collection state rather than always constructing every predecessor path.

Distinct forms of path enumeration:

| Function                           | Meaning                                                          | Important limit                                            |
| ---------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------- |
| `dijkstra`                         | Least-cost distances from a source                               | Requires nonnegative edge costs                            |
| `all_shortest_paths`               | Enumerates all equally optimal paths between two specified nodes | Output can grow exponentially                              |
| `single_source_all_shortest_paths` | All shortest paths from one source, grouped by destination       | More costly than distances alone                           |
| `k_shortest_path`                  | Kth-shortest path costs                                          | Does not return the first k explicit paths                 |
| `negative_cycle_finder`            | Finds a negative-weight cycle                                    | Cycle existence is distinct from shortest-path computation |
| `distance_matrix`                  | Dense all-pairs unit-edge-cost distances                         | Allocates quadratic-size output                            |

The `distance_matrix` function is particularly easy to misunderstand. It treats every edge as cost 1, supports an `as_undirected` option, and returns an `ndarray::Array2<f64>` with a caller-chosen sentinel for unreachable pairs. It is not an arbitrary weighted all-pairs shortest-path routine.

It can parallelize source-node computations above a caller-specified `parallel_threshold`; the documented example threshold is 300 nodes, not a universal optimum.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/shortest_path/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/shortest_path/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/shortest_path/fn.all_shortest_paths.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/shortest_path/fn.all_shortest_paths.html?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/shortest_path/fn.distance_matrix.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/shortest_path/fn.distance_matrix.html?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/distancemap/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/distancemap/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

### 5.2 Centrality and network importance

The `centrality` module is a major extension beyond basic connectivity and reachability analysis.

| Family                  | APIs                                                                                    | Measures                                                     |
| ----------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| Local connectivity      | `degree_centrality`                                                                     | Importance through incident connections                      |
| Shortest-path brokerage | `betweenness_centrality`, `edge_betweenness_centrality`                                 | Participation in shortest paths between other nodes          |
| Distance accessibility  | `closeness_centrality`, `newman_weighted_closeness_centrality`                          | Accessibility through shortest-path distances                |
| Recursive influence     | `eigenvector_centrality`, `katz_centrality`                                             | Importance derived from connections to other important nodes |
| Group importance        | `group_degree_centrality`, `group_closeness_centrality`, `group_betweenness_centrality` | Importance of a selected collection of nodes                 |

These metrics answer different structural questions and cannot be treated as interchangeable rankings.

Betweenness centrality identifies vertices occupying shortest-path corridors. The implementation supports normalized and unnormalized results, optional inclusion of endpoints, and multithreaded execution above a configured node threshold.

Closeness centrality summarizes path-distance accessibility. The `wf_improved` parameter applies a reachability correction for disconnected graphs. Directed graph results depend on the orientation used for path distances.

Eigenvector and Katz centrality use iterative numerical methods. They are not guaranteed to converge under arbitrary parameters and graphs; their APIs provide iteration limits and convergence tolerances. An `Option` result can communicate nonconvergence. Weighted forms accept callbacks for edge strengths.

Group centralities are especially significant: they measure a set of nodes as a collective entity rather than obtaining a group ranking merely by summing individual centrality scores. The metrics can distinguish the coverage or brokerage contributed by a coordinated set of vertices.

Several computations can use Rayon internally, controlled by `parallel_threshold`. Parallel speedups depend on node count, graph structure, per-source computational work and thread overhead.

A further important detail is that some centrality APIs return `Vec<Option<f64>>` indexed by petgraph node positions. An absent node in a sparse `StableGraph` can therefore be represented differently from an existing node whose measured centrality equals zero.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/centrality/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/centrality/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/centrality/fn.betweenness_centrality.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/centrality/fn.betweenness_centrality.html?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/centrality/fn.katz_centrality.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/centrality/fn.katz_centrality.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

### 5.3 Connectivity, graph decomposition and cycle analysis

The `connectivity` module includes algorithms that expose structural information beyond whether nodes are mutually reachable.

| Operation                              | Purpose                                                            |
| -------------------------------------- | ------------------------------------------------------------------ |
| `connected_components`                 | Enumerate undirected connected components as node sets             |
| `articulation_points`, `bridges`       | Identify critical vertices and edges                               |
| `chain_decomposition`                  | Construct graph chain decomposition useful for structural analysis |
| `core_number`                          | Calculate k-core membership strength for vertices                  |
| `cycle_basis`                          | Obtain a basis of undirected cycles                                |
| `johnson_simple_cycles`                | Enumerate elementary directed cycles                               |
| `find_cycle`                           | Find a cycle from a particular traversal context                   |
| `stoer_wagner_min_cut`                 | Compute global minimum cut of an undirected weighted graph         |
| `all_simple_paths_multiple_targets`    | Enumerate simple paths to several possible destinations            |
| `longest_simple_path_multiple_targets` | Search for long simple paths across multiple candidate targets     |

K-core decomposition is particularly useful for understanding nested connectivity. A node's core number is the largest k for which it belongs to a subgraph where each node has at least k neighbors within that subgraph. This is different from its ordinary degree in the original graph.

However, the `core_number` documentation explicitly assumes the absence of self-loops and parallel edges. Using it on a multigraph without ensuring the intended simple-graph semantics can produce unexpected or incorrect values.

Stoer–Wagner minimum cut computes a global minimum-weight separation of an undirected graph. This differs from computing maximum flow between one designated source and sink, and it does not require choosing those terminals in advance. Its edge weights must satisfy the algorithm's nonnegative-weight requirements.

Simple-cycle enumeration is another substantial capability. The complete set of elementary cycles can be exponentially large, so the distinction between finding one cycle, computing an undirected cycle basis, and enumerating every directed elementary cycle is important.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/connectivity/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/connectivity/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/connectivity/fn.core_number.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/connectivity/fn.core_number.html?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/connectivity/fn.stoer_wagner_min_cut.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/connectivity/fn.stoer_wagner_min_cut.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.4 DAG algorithms and dependency ordering

Rustworkx-core has a dedicated `dag_algo` module covering DAG-specific analysis:

- `lexicographical_topological_sort`
- `layers`
- `longest_path` and `longest_path_length`
- `collect_runs` and `collect_bicolor_runs`
- `traversal_directions`

Lexicographical topological sorting resolves the ambiguity among multiple valid topological orders using a caller-supplied key. This is useful where deterministic ordering matters beyond satisfying dependencies. The API also supports reverse traversal and an optional initial frontier.

DAG layering assigns nodes to execution or dependency layers. Unlike a total topological order, layers preserve useful information about nodes that may be processed at the same stage, subject to any additional resource constraints outside the graph.

Longest paths in DAGs can be computed efficiently because the acyclic structure permits dynamic programming over a topological ordering. `longest_path` can return the path together with its accumulated weight; `longest_path_length` avoids constructing the explicit path where only the scalar length is required.

This is not a general solution to the longest simple path problem on cyclic graphs, which is computationally much harder.

An architectural composition is particularly valuable here: a graph can have its acyclicity enforced by petgraph's `Acyclic` wrapper, then be analyzed using the DAG-specific algorithms of rustworkx-core, subject to the wrapper implementing the required visitor traits.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/dag_algo/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/dag_algo/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/dag_algo/fn.lexicographical_topological_sort.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/dag_algo/fn.lexicographical_topological_sort.html?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/dag_algo/fn.longest_path.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/dag_algo/fn.longest_path.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.5 Graph coloring and matching

The `coloring` module includes greedy vertex and edge coloring, multiple coloring strategies, preassigned colors, two-color testing, and specialized edge-coloring algorithms.

Important functions include `greedy_node_color`, `greedy_node_color_with_coloring_strategy`, `greedy_node_color_with_preset_colors`, `greedy_edge_color`, `misra_gries_edge_color` and `two_color`.

The documented greedy strategies include `Degree`, `Saturation` and `IndependentSet`. They differ in which vertices are selected for coloring next. This can materially affect color counts, computation cost and reproducibility, even though all target a valid coloring.

Preset-color support is particularly useful when a graph is only partially unassigned. It allows a coloring process to respect an existing allocation rather than always coloring from scratch.

The distinct `bipartite_coloring::bipartite_edge_color` algorithm addresses edge coloring of bipartite graphs, rather than simply assigning two colors to bipartite vertices.

Coloring should not be confused with proving optimality. Greedy methods generally construct valid colorings but need not establish the minimum possible number of colors. The Misra–Gries routine is a different, constructive edge-coloring algorithm with its own applicability and guarantees.

Rustworkx-core's `max_weight_matching` addresses a separate optimization problem. It computes a maximum-weight matching in a general undirected graph using a weighted matching algorithm based on Galil's work.

Its distinctive parameters include:

- `max_cardinality = false`: optimize matching weight.
- `max_cardinality = true`: first require maximum cardinality, then maximize weight among those matchings.
- A fallible callback returning an `i128` edge weight.
- An optional optimality-verification flag intended primarily for tests; a verification failure can panic.

The documented worst-case running time is \\(O(V^3)\\). This functionality is materially different from greedy matching or maximum-cardinality matching alone.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/coloring/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/coloring/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/bipartite_coloring/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/bipartite_coloring/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/max_weight_matching/fn.max_weight_matching.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/max_weight_matching/fn.max_weight_matching.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.6 Graph contraction and mutation extensions

Node contraction is one of rustworkx-core's most valuable structural capabilities.

Contraction merges a selected collection of vertices into a replacement vertex, redirecting external relationships from the removed vertices to that replacement.

The extension traits are:

| Trait                           | Graph orientation | Edge behavior                                       |
| ------------------------------- | ----------------- | --------------------------------------------------- |
| `ContractNodesDirected`         | Directed          | Can introduce parallel edges                        |
| `ContractNodesUndirected`       | Undirected        | Can introduce parallel edges                        |
| `ContractNodesSimpleDirected`   | Directed          | Merges edges to preserve simple-graph relationships |
| `ContractNodesSimpleUndirected` | Undirected        | Merges edges to preserve simple-graph relationships |

Simple variants accept a merge function to combine edge payloads when multiple original relationships collapse into the same new relationship.

This is a significant capability because contracting vertices is not equivalent to merely removing them. Contraction must preserve external connectivity and define what happens to edge attributes, self-loops, parallel edges and cycles.

For directed graphs, the extension interfaces can apply cycle checking and return structured contraction errors. The separate `can_contract` function helps evaluate whether a proposed contraction preserves the necessary constraints.

Implementation coverage is narrower than the abstract trait design might suggest. The documented contraction traits are implemented for `StableGraph` and `GraphMap`, not ordinary `Graph`.

The broader `graph_ext` module also provides:

- `HasParallelEdgesDirected` and `HasParallelEdgesUndirected`.
- `NodeRemovable`, `EdgeRemovable` and `EdgeFindable`.

These traits help generic graph transformations express specific mutation capabilities.

In a system that repeatedly coarsens, groups or simplifies graph structures, contraction can eliminate substantial bespoke edge-redirection and attribute-merging code.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/graph_ext/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/graph_ext/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/graph_ext/contraction/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/graph_ext/contraction/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 5.7 Additional structural transformations and optimization algorithms

| Module          | Capability                                                                | Key distinction                                                           |
| --------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `line_graph`    | Construct a graph whose vertices correspond to edges of an original graph | Converts edge-adjacency relationships into vertex-adjacency relationships |
| `planar`        | Test whether an undirected graph is planar                                | `is_planar` returns a boolean, not a planar embedding or visualization    |
| `transitivity`  | Global transitivity / clustering analysis                                 | Quantifies triangle-based connectivity, not reachability closure          |
| `steiner_tree`  | Metric closure and approximate Steiner tree                               | Connects selected terminals through intermediate vertices                 |
| `token_swapper` | Approximate token-swapping sequence                                       | Solves a constrained graph-rearrangement problem                          |

Line graphs are useful when the entities under analysis are relationships rather than the original vertices. Adjacent edges in the original graph become adjacent vertices in the line graph. This is a new materialized graph, so its size and attributes must be considered.

Planarity testing uses a Left–Right planarity test. It establishes whether an undirected graph can be embedded in the plane without edge intersections. It does not construct coordinates, produce a drawing, or serve as a complete layout engine.

Transitivity measures the extent to which connections form triangles. Directed and undirected forms are available. It is distinct from the transitive closure of a reachability relation, despite the similar terminology.

Steiner tree algorithms build an approximate tree connecting designated terminal vertices, potentially through additional intermediate nodes. Both petgraph and rustworkx-core have Steiner-related functionality, so the distinction is in the particular APIs, outputs and implementation assumptions—not exclusive ownership by one library.

Token swapping is a more specialized but potentially valuable capability. Given a graph and a mapping of token locations to destinations, `token_swapper` constructs a sequence of adjacent swaps. The algorithm is approximately optimal, supports partial mappings representing missing tokens, accepts a random seed and number of trials, and can run parallel trials above a configured threshold.

Its `MapNotPossible` error indicates that a requested mapping could not be found. It should not be interpreted as a proof that a returned solution minimizes the number of swaps.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/line_graph/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/line_graph/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/planar/fn.is_planar.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/planar/fn.is_planar.html?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/transitivity/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/transitivity/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/steiner_tree/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/steiner_tree/?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/token_swapper/fn.token_swapper.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/token_swapper/fn.token_swapper.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+4

### 5.8 Graph generation and synthetic topology models

Rustworkx-core provides a substantial collection of graph generators. This is a meaningful complement to petgraph's more limited, partly experimental generator facilities.

| Generator family      | Notable functions                                                                                         | Typical graph characteristics                               |
| --------------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| Elementary structures | `path_graph`, `cycle_graph`, `star_graph`, `complete_graph`                                               | Controlled deterministic topology                           |
| Regular and composite | `grid_graph`, `barbell_graph`, `lollipop_graph`, `binomial_tree_graph`, `full_rary_tree_graph`            | Structured topology with known connectivity                 |
| Random edge models    | `gnp_random_graph`, `gnm_random_graph`, `random_regular_graph`                                            | Random connectivity with varying density/degree constraints |
| Growth models         | `barabasi_albert_graph`, `dorogovtsev_goltsev_mendes_graph`                                               | Preferential attachment or iterative growth                 |
| Communities           | `sbm_random_graph`, `random_bipartite_graph`                                                              | Block/community relationships or bipartite structure        |
| Spatial / geometric   | `random_geometric_graph`, `hyperbolic_random_graph`                                                       | Connectivity influenced by node positions or geometry       |
| Specialized           | `heavy_hex_graph`, `heavy_square_graph`, `hexagonal_lattice_graph`, `petersen_graph`, `karate_club_graph` | Known structural patterns and reference topologies          |

These generators can eliminate bespoke graph construction code for benchmarks, algorithm tests, topology experiments and synthetic workloads.

The important distinction between random graph models is their underlying generative assumption. For example, \\(G(n,p)\\) models an independent probability for eligible edges, while \\(G(n,m)\\) specifies an edge count. A stochastic block model additionally encodes community-dependent probabilities.

A preferential-attachment graph, a random regular graph and a geometric graph can have markedly different clustering, degree distributions, shortest-path characteristics and algorithmic difficulty even at the same node and edge counts.

Many generators are generic over petgraph construction traits and accept functions for node and edge weights, allowing generated topology to be combined with application-defined payload types.

Care is required when selecting directedness and bidirectional edge construction. For a multigraph representation, adding both directed orientations to an undirected graph can create duplicate relationships.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/generators/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/generators/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.9 Graphs embedded in metric spaces

The `geometry` module is a significant addition in rustworkx-core 0.18.

It provides distance functions including:

- Euclidean, angular, \\(L_p\\) and maximum distance.
- Hyperboloid and polar hyperbolic distances.
- `greedy_routing` and `greedy_routing_success_rate`.

The interesting architectural capability is routing based on positions in a metric space, rather than requiring all route selection to rely on a globally computed shortest-path tree.

Greedy routing chooses the next step based on a local geometric criterion. The success-rate routine evaluates what fraction of relevant source–destination pairs can be successfully routed by that mechanism.

Unlike Dijkstra or suitably configured A\*, greedy geographic routing need not discover a globally shortest path and can fail to reach a destination even if the graph contains a valid route.

This makes the module most naturally applicable to networks with meaningful geometric or metric embeddings, where local routing behavior is itself worth analyzing.

The geometry API is separate from general graph drawing. Its existence does not imply that rustworkx-core provides a complete layout or visualization subsystem.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/geometry/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/geometry/?utm_source=chatgpt.com)

Release notes describing the 0.18 geometry additions:

[https://www.rustworkx.org/release_notes.html](https://www.rustworkx.org/release_notes.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

docs.rs+1

### 5.10 Traversal facilities beyond petgraph

Rustworkx-core also exposes event-driven traversal APIs and higher-level traversal queries:

| Family                | Examples                                                        |
| --------------------- | --------------------------------------------------------------- |
| Interactive traversal | `breadth_first_search`, `depth_first_search`, `dijkstra_search` |
| Events                | `BfsEvent`, `DfsEvent`, `DijkstraEvent`                         |
| Reachability          | `ancestors`, `descendants`                                      |
| BFS-derived structure | `bfs_layers`, `bfs_predecessors`, `bfs_successors`              |
| Path discovery        | `dfs_edges`, `generate_random_path`                             |

These routines reduce the need to rewrite traversal scaffolding for common graph questions.

The event-based interfaces can support computations that need discovery, examination and completion notifications rather than merely a final visited-node set.

`generate_random_path`, introduced in the 0.18 release family, additionally provides random-walk-like path generation, useful for graph sampling and randomized analysis.

Documentation:

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/traversal/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/traversal/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

## 6. Combining the two crates into richer graph operations

The most useful architectural opportunities often come from composing native capabilities, rather than using each algorithm as an isolated operation.

| Problem shape                         | Native capability combination                              | What becomes possible                                                                             |
| ------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Dynamically changing dependency graph | `StableGraph` + `Acyclic` + rustworkx DAG algorithms       | Stable references, prevention of new cycles, deterministic ordering and dependency-layer analysis |
| Cyclic dependency decomposition       | Petgraph SCCs + condensation + DAG analysis                | Reduce a cyclic graph into components whose inter-component dependencies form a DAG               |
| Graph-based resource allocation       | Rustworkx coloring + maximum-weight matching               | Assign mutually incompatible tasks to groups and optimize pairwise assignments                    |
| Network criticality analysis          | Bridges/articulation points + minimum cuts + centrality    | Distinguish structural disconnection, global cuts and shortest-path brokerage                     |
| Graph simplification                  | Filtered views + contraction + transitive reduction        | Simplify relationships while explicitly controlling reachability, multiplicity and identity       |
| Structural comparison                 | Petgraph isomorphism + rustworkx connectivity / centrality | Separate exact structural equivalence from approximate structural characterization                |
| Synthetic workload analysis           | Rustworkx generators + petgraph algorithms + Rayon         | Produce controlled graph families and compare algorithm behavior under different structures       |

These combinations have limits worth emphasizing.

Graph condensation and contraction are not the same operation. SCC condensation preserves the connectivity relationships among strongly connected groups and produces a DAG; general node contraction can alter reachability and create new edges or cycles unless the implementation imposes the appropriate restrictions.

Transitive reduction preserves reachability, not arbitrary graph semantics. Removing a redundant reachability edge may change route counts, weighted shortest paths or attributes associated with that edge. It is not automatically valid for a weighted multigraph merely because two vertices remain connected.

Views and transforms are different. `NodeFiltered` and `EdgeFiltered` can expose a subgraph without constructing a separate graph, whereas contractions, line graphs and SCC condensation produce transformed structures with different identity and relationship semantics.

### 6.1 A minimal cross-library Rust example

The following illustrative example uses the petgraph re-export from rustworkx-core and applies two different algorithm families to the same in-memory graph.

```

use std::convert::Infallible;

use rustworkx_core::petgraph::{
    graph::UnGraph,
    visit::EdgeRef,
};
use rustworkx_core::{
    centrality::betweenness_centrality,
    max_weight_matching::max_weight_matching,
};

let graph = UnGraph::<(), i128>::from_edges(&[
    (0, 1, 5),
    (1, 2, 11),
    (2, 3, 5),
]);

// Structural importance of nodes.
let centrality =
    betweenness_centrality(&graph, false, true, 200);

// Maximum-cardinality matching; weight breaks ties.
let matching = max_weight_matching(
    &graph,
    true,
    |edge| Ok::<i128, Infallible>(*edge.weight()),
    false,
).unwrap();

assert_eq!(centrality.len(), graph.node_count());
assert_eq!(matching.len(), 2);

```

This illustrates three significant design properties. The graph's Rust payload types are independent of the algorithms; algorithms borrow the graph rather than requiring a conversion to a rustworkx-specific graph class; and weight interpretation is supplied at the algorithm boundary.

The example is documentation-derived rather than independently executed here.

## 7. Integration, feature flags and execution considerations

### 7.1 Cargo versions and compiler requirements

A minimal dependency on the verified rustworkx-core release is:

```
[dependencies]
rustworkx-core = "0.18.1"
```

Because rustworkx-core re-exports its petgraph dependency, downstream Rust code can use `rustworkx_core::petgraph` without declaring a separate direct petgraph dependency merely to access graph types.

Where direct petgraph features are needed, Cargo can include an explicit compatible dependency:

```
[dependencies]
petgraph = { version = "0.8.3", features = ["serde-1", "rayon"] }
rustworkx-core = "0.18.1"
```

Rustworkx-core 0.18.1 depends on petgraph `^0.8`. Keeping petgraph versions compatible avoids separate incompatible versions of graph types being compiled into the dependency graph.

Rustworkx-core 0.18 requires Rust 1.85 or newer. The 0.18 release raised the minimum supported compiler version from 1.79 to 1.85, alongside its transition to Rust 2024 edition.

A documentation caveat: the rustworkx-core crate homepage still contains an old Cargo example using `rustworkx-core = "0.11"`. That snippet is not its current recommended release version.

Sources:

[https://docs.rs/crate/rustworkx-core/0.18.1](https://docs.rs/crate/rustworkx-core/0.18.1?utm_source=chatgpt.com)

[https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/?utm_source=chatgpt.com)

[https://www.rustworkx.org/release_notes.html](https://www.rustworkx.org/release_notes.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2

### 7.2 Petgraph feature flags

Petgraph supports both standard and optional functionality.

| Feature        | Status       | Capability                                                 |
| -------------- | ------------ | ---------------------------------------------------------- |
| `graphmap`     | Default      | Enables `GraphMap`                                         |
| `stable_graph` | Default      | Enables `StableGraph`                                      |
| `matrix_graph` | Default      | Enables `MatrixGraph`                                      |
| `std`          | Default      | Standard-library integration                               |
| `serde-1`      | Optional     | Serde support for `Graph`, `StableGraph`, `GraphMap`       |
| `rayon`        | Optional     | Parallel iterators and selected algorithms                 |
| `dot_parser`   | Optional     | Graph construction from Graphviz DOT descriptions          |
| `generate`     | Experimental | Additional graph generators with unstable API expectations |
| `unstable`     | Experimental | Enables unstable crate functionality                       |

Petgraph can be used in some `no_std` environments with default features disabled and appropriate allocation support.

That property should not be transferred automatically to rustworkx-core, which has a different dependency footprint including Rayon and ndarray. A Rust application requiring `no_std` should distinguish petgraph's supported configurations from rustworkx-core's actual build requirements.

Rustworkx-core has substantial graph-generation functionality without depending on petgraph's experimental `generate` feature.

Source:

[https://docs.rs/petgraph/0.8.3/petgraph/](https://docs.rs/petgraph/0.8.3/petgraph/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 7.3 Serialization, interoperability and graph visualization

Petgraph provides several useful interfaces beyond its algorithms.

Serde integration. The `serde-1` feature enables serialization for its principal dynamic graph types. This offers an established mechanism for encoding and decoding graph structures, although application-specific identifiers, schema versions and payload compatibility remain separate concerns.

Graphviz DOT. `petgraph::dot::Dot` exports graphs to DOT, with configuration options and callback-based attribute generation. The optional `dot_parser` feature supports building `Graph` and `StableGraph` from DOT input.

DOT is a graph-description format; actual layout and image rendering are responsibilities of external tools such as Graphviz.

graph6. The `graph6` module provides interoperability with the compact graph6 representation for undirected graphs. This is particularly relevant to structural graph datasets and graph-algorithm research; it is not a universal interchange format for arbitrary typed, weighted, directed multigraphs.

Graph operators and supporting structures. Petgraph also includes graph construction operators and a reusable `UnionFind` disjoint-set implementation. Such utilities are useful when graph construction, connectivity maintenance or disjoint-set computation is part of a larger algorithm.

Documentation:

[https://docs.rs/petgraph/0.8.3/petgraph/dot/](https://docs.rs/petgraph/0.8.3/petgraph/dot/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/graph6/](https://docs.rs/petgraph/0.8.3/petgraph/graph6/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/operator/](https://docs.rs/petgraph/0.8.3/petgraph/operator/?utm_source=chatgpt.com)

[https://docs.rs/petgraph/0.8.3/petgraph/unionfind/](https://docs.rs/petgraph/0.8.3/petgraph/unionfind/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 7.4 Parallel execution and memory behavior

Neither crate should be characterized simply as providing “parallel graph algorithms” across the board.

Petgraph's Rayon feature enables selected parallel operations, including algorithms such as `parallel_johnson` and parallel PageRank.

Rustworkx-core includes Rayon-based parallelism in several centrality, distance-matrix and token-swapping computations. Some APIs expose a threshold at which parallel processing begins, and thread count can be configured through `RAYON_NUM_THREADS`.

However, algorithmic complexity and data representation usually dominate the underlying performance characteristics:

| Consideration                | Consequence                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------ |
| Sparse adjacency traversal   | `Graph` or `Csr` may avoid scanning nonexistent edges                          |
| Dense connectivity           | `MatrixGraph` trades quadratic memory for direct matrix access                 |
| Sparse `StableGraph` indices | Index-addressed arrays may scale with `node_bound`, not live `node_count`      |
| Dense all-pairs distances    | Memory grows as \\(O(V^2)\\), independently of input graph sparsity            |
| Path/cycle enumeration       | Output can grow exponentially even with efficient per-result traversal         |
| Repeated shortest paths      | Parallel sources can reduce wall time, but may replicate working state         |
| Small graphs                 | Thread dispatch and synchronization can outweigh parallel benefit              |
| Cost callbacks               | Expensive or fallible payload interpretation can dominate edge relaxation work |

Parallelism also imposes `Send`, `Sync` and closure-related constraints on certain APIs. Not every visitor or graph adaptor satisfies those bounds automatically.

These are architectural performance mechanisms rather than quantitative benchmark claims. Actual speedups require measurement against a particular graph family and workload.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



### 7.5 Important correctness and semantic boundaries

Several graph-library mistakes originate not in the algorithm implementation but in mismatched assumptions.

| Concern                 | Material distinction                                                                                              |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Node identity           | Stable indices are not globally unique generation-checked IDs                                                     |
| Multigraphs             | Some graph representations merge duplicate edges; some algorithms assume simple graphs                            |
| Directedness            | Directed and undirected transformations change graph semantics                                                    |
| Edge weights            | Cost, strength, capacity, distance and matching benefit are not interchangeable                                   |
| Disconnected graphs     | Zero, infinity, missing and `None` can mean different things                                                      |
| Return ordering         | Graph iteration, hash maps, parallel execution and equivalent-optimum tie-breaking may not provide a unique order |
| Approximation           | Steiner, greedy coloring, token swapping and heuristic routing have different optimality guarantees               |
| Algorithm preconditions | Acyclicity, nonnegative costs, connectedness and valid metrics are not universally enforced by types              |
| Data conversion         | Converting between graph representations may change indices, multiplicity or traversal ordering                   |

In particular, `GraphMap` cannot preserve parallel-edge multiplicity; k-core decomposition assumes simple-graph behavior; Dijkstra assumes nonnegative costs; and path enumeration must be distinguished from kth-path cost computation.

There is also a version-sensitive caveat in the upstream rustworkx documentation: its unreleased release-note preview describes a correction to `rustworkx_core::dag_algo::longest_path_length` for a DAG whose node at index zero was deleted. Because that correction appears in unreleased notes, it should not be assumed to be present in the published 0.18.1 crate without verifying the tagged implementation.

Source:

[https://www.rustworkx.org/release_notes.html](https://www.rustworkx.org/release_notes.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

## 8. Architectural boundaries and adjacent libraries

The petgraph ecosystem is a strong foundation for in-memory pairwise graph modeling and algorithms, but it is not a complete graph-data platform.

### Graph structure versus domain modeling

Both crates support strongly typed Rust node and edge payloads. However, their core graph abstractions model vertices connected by pairwise edges. Domain-specific schemas, relationship categories, cross-graph identity, provenance, transaction semantics and higher-order hyperedges still require an appropriate domain representation.

The benefit is that these semantics can often be expressed directly in Rust types and supplied to generic algorithms without modifying those algorithms.

### Graph computation versus graph persistence and queries

Petgraph and rustworkx-core do not provide a transactional graph database, a general graph query language, or distributed graph execution.

Their strength is direct graph manipulation and algorithmic computation after graph data is in memory.

A surrounding data system may store canonical node and relationship records, construct or expose graph views for computation, and consume results as ordinary structured Rust data. The conversion and identity mapping remain meaningful responsibilities.

### Native Rust versus Python rustworkx

The Python `rustworkx` project exposes related algorithms and graph structures through a Python-facing interface. Rustworkx-core is the reusable native Rust algorithm layer, not that entire Python interface.

Consequently, Python-only capabilities—such as particular graph-layout or visualization functions—should not be attributed to the Rust crate without confirming that an equivalent Rust API exists.

For Rust/Python interoperation, PyO3 or another appropriate binding mechanism is a separate integration layer. Rustworkx-core itself does not make a Rust graph automatically available as a Python rustworkx object.

## 9. Capability-selection synthesis

These libraries are most effective when their distinct roles remain clear.

Petgraph is the graph foundation. Its most valuable architectural capabilities are multiple representations with different identity and performance tradeoffs, generic visitor traits, filtered graph views, dynamically enforced acyclicity, graph transformations, and a substantial core algorithm suite.

Rustworkx-core is an extended algorithm and structural-transformation toolkit. Its most valuable differentiators include centrality and group centrality, extensive graph generators, graph contraction, advanced matching and coloring, graph coarsening and structural analysis, DAG-specific operations, token swapping, geometric routing, and configurable parallel computation.

The strongest interaction is not one crate replacing the other. It is that one suitable graph representation can serve a broad collection of computations without bespoke conversions or duplicate graph structures—provided the chosen graph type satisfies the algorithms' trait contracts and the transformations preserve the required semantics.

The most important design decisions therefore tend to concern four underlying questions: what constitutes node identity, what edge multiplicity and direction mean, how frequently topology changes, and which operations must preserve or transform the original graph.

For deeper investigation, the highest-value primary sources are:

| Area                                 | Primary reference                                                                                                                                           |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Petgraph full API                    | [https://docs.rs/petgraph/0.8.3/petgraph/](https://docs.rs/petgraph/0.8.3/petgraph/?utm_source=chatgpt.com)                                                 |
| Petgraph generic graph traits        | [https://docs.rs/petgraph/0.8.3/petgraph/visit/](https://docs.rs/petgraph/0.8.3/petgraph/visit/?utm_source=chatgpt.com)                                     |
| Petgraph algorithm catalog           | [https://docs.rs/petgraph/0.8.3/petgraph/algo/](https://docs.rs/petgraph/0.8.3/petgraph/algo/?utm_source=chatgpt.com)                                       |
| Petgraph acyclic graph wrapper       | [https://docs.rs/petgraph/0.8.3/petgraph/acyclic/](https://docs.rs/petgraph/0.8.3/petgraph/acyclic/?utm_source=chatgpt.com)                                 |
| Rustworkx-core full API              | [https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/?utm_source=chatgpt.com)                       |
| Rustworkx-core complete symbol index | [https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/all.html](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/all.html?utm_source=chatgpt.com)       |
| Rustworkx-core structural extensions | [https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/graph_ext/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/graph_ext/?utm_source=chatgpt.com)   |
| Rustworkx-core graph generators      | [https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/generators/](https://docs.rs/rustworkx-core/0.18.1/rustworkx_core/generators/?utm_source=chatgpt.com) |
| Rustworkx upstream source            | [https://github.com/Qiskit/rustworkx](https://github.com/Qiskit/rustworkx?utm_source=chatgpt.com)                                                           |
| Petgraph upstream source             | [https://github.com/petgraph/petgraph](https://github.com/petgraph/petgraph?utm_source=chatgpt.com)                                                         |
| Rustworkx release notes              | [https://www.rustworkx.org/release_notes.html](https://www.rustworkx.org/release_notes.html?utm_source=chatgpt.com)                                         |

The references above are version-scoped where practical; the upstream repositories and release notes are useful for subsequent changes but should not be treated as evidence that development-branch additions are already published in the covered crate versions.