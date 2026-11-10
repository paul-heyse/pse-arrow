# Graphica — Comprehensive Rust Library Overview

Research date: November 9, 2026 | Target: Graphica 3.0.1 | Language: Rust | License: MIT

Graphica is a specialized Rust graph library developed within the Symbolica ecosystem. Its primary strengths are graph canonicalization, graph isomorphism, automorphism-group computation, and exhaustive generation of structurally distinct graphs.

Unlike general-purpose graph libraries such as `petgraph`, Graphica is designed around structural identity and symmetry. It supports labeled multigraphs containing directed and undirected edges simultaneously, including parallel edges and self-loops. Its modified implementation of McKay's graph-canonicalization algorithm makes graph equivalence a first-class computational operation.

This is especially valuable for combinatorial structure enumeration, mathematical graph generation, diagrammatic calculations, symbolic systems, and applications where graphs differing only by permissible vertex relabeling should be treated as equivalent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



## 1. Executive orientation

| Dimension              | Graphica capability                                                                             |
| ---------------------- | ----------------------------------------------------------------------------------------------- |
| Core representation    | Generic, labeled multigraphs with mixed edge directions                                         |
| Canonicalization       | Deterministic representative of a graph's isomorphism class                                     |
| Isomorphism testing    | Structural equivalence respecting vertex labels, edge labels, direction, and multiplicity       |
| Symmetry analysis      | Automorphism generators, vertex orbits, and exact symmetry-group sizes                          |
| Graph generation       | Enumerate connected graphs satisfying specified vertex and edge attachment rules                |
| Generation constraints | Vertex count, cycle count, bridges, self-loops, zero-flow bridges, custom pruning               |
| Structural analysis    | DFS spanning trees, chain decomposition, connectivity, bridge detection                         |
| Customization          | Generic payload types, equality/identity customization, filtering and progress callbacks        |
| Visualization          | Graphviz DOT and Mermaid text export                                                            |
| Primary boundary       | Not a comprehensive shortest-path, centrality, community-detection, or graph-database framework |

Graphica is deliberately narrower than a general graph-analysis platform, but unusually capable within its specialization. Its major advantage is the combination of constrained enumeration and symmetry reduction: a system can describe permissible graph structures, generate candidate topologies, identify equivalent configurations, and retain their canonical representations and symmetry factors without implementing those mathematical mechanisms independently.

### Project and version information

| Property              | Verified status                            |
| --------------------- | ------------------------------------------ |
| Rust crate            | `graphica`                                 |
| Target version        | `3.0.1`                                    |
| Rust edition          | 2024                                       |
| Minimum Rust version  | 1.89                                       |
| License               | MIT                                        |
| Maintainer            | Ben Ruijl / Symbolica                      |
| Core dependencies     | `ahash`, `dyn-clone`, `numerica`           |
| Optional dependencies | `serde`, `bincode`, `bincode-trait-derive` |

The upstream manifest declares version 3.0.1 and depends on Numerica 3.0. Some search-indexed rustdoc pages still display versions 2.1.0 or 3.0.0. Accordingly, this overview uses the 3.0.1 repository source as the primary authority for implementation-level behavior, supplemented by the published API documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Primary sources:

- Repository: `https://github.com/symbolica-dev/graphica`
- API reference: `https://docs.rs/graphica/latest/graphica/`
- Crate registry: `https://crates.io/crates/graphica`
- Upstream implementation: `https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs`
- Cargo manifest: `https://github.com/symbolica-dev/graphica/blob/main/Cargo.toml`

## 2. Conceptual and data model

Graphica represents graphs as two indexed collections: vertices and edges. The graph does not require a homogeneous direction convention, nor does it require a unique edge between two vertices.

### 2.1 Generic graph representation

The principal type is:

```
pub struct Graph<NodeData = Empty, EdgeData = Empty> {
    nodes: Vec<Node<NodeData>>,
    edges: Vec<Edge<EdgeData>>,
}
```

Its node and edge structures are conceptually:

```
pub struct Node<N = Empty> {
    pub data: N,
    pub edges: Vec<usize>,
    pub valence: usize,
}

pub struct Edge<E = Empty> {
    pub vertices: (usize, usize),
    pub directed: bool,
    pub data: E,
}
```

The source confirms these layouts. Vertices and edges are referenced through `usize` indices rather than owned graph objects, references, or opaque node handles. Nodes maintain adjacency through edge-index vectors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



This design provides several useful properties:

- Arbitrary vertex and edge payloads are directly supported through Rust generics.
- Parallel edges are represented independently and retain their own data.
- Self-loops are valid; a self-loop contributes two to its vertex's valence.
- The same graph may contain directed and undirected edges.
- Construction and traversal can use compact integer identifiers.

A node's `edges` vector stores each incident edge index once, including a self-loop, while `valence` counts a self-loop twice. These quantities must not be treated as interchangeable.

### 2.2 Mixed directed and undirected edges

A directed edge uses its stored vertex pair as source and target. An undirected edge is normalized so that the lower-indexed endpoint comes first when added.

For example, a graph can contain all of the following simultaneously:

Illustrative multigraph topology: arrows indicate directed edges; plain lines indicate undirected edges. Edge labels are omitted.

This expressiveness matters when modeling structures where different connection types have different orientation semantics: chemical or physical diagrams, typed relationships, computational operators, and heterogeneous network topologies.

Direction and label information are also respected during canonicalization, so two graphs with identical untyped connectivity but different edge directions or data can belong to different isomorphism classes.

### 2.3 Graph identity versus graph indexing

A crucial distinction is that Graphica supports both indexed structural equality and isomorphism-aware equality, but they are not the same operation.

`Graph` derives `PartialEq`, `Eq`, `Ord`, and `Hash` from its stored representation. Consequently, ordinary comparison and hashing are sensitive to how vertices and edges are numbered and ordered.

By contrast, `is_isomorphic()` and `canonize()` reason about permissible relabelings.

For example, two separately constructed graphs might have:

- Identical topology and payloads.
- Different vertex insertion orders.
- Different edge insertion orders.

They may be isomorphic without initially comparing equal as Rust `Graph` values.

Canonicalization converts them to comparable representatives, making conventional equality and hashing useful for deduplication by isomorphism class.

This distinction is foundational to using Graphica correctly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 2.4 HiddenData: separating graph identity from auxiliary metadata

Graphica provides an especially useful generic wrapper:

```
pub struct HiddenData<T, U> {
    pub data: T,
    pub hidden: U,
}
```

`HiddenData<T, U>` treats `data` as semantically relevant and `hidden` as auxiliary metadata.

Its implementations of equality, hashing, and ordering operate on the public `data` field only. Its `Display` implementation, however, includes both fields.

For example:

```
use graphica::HiddenData;

let payload = HiddenData::new(
    "reaction",
    "origin: imported-model-42",
);
```

Two instances with the same public value but different hidden metadata compare identically when used as graph labels.

That permits attaching provenance, annotations, external identifiers, or diagnostic information without causing logically equivalent graphs to become distinct merely because their metadata differs.

Important semantic qualification: ignoring hidden data in identity does not mean it is removed from the graph or concealed from output. It remains accessible and may appear in formatted graph exports.

Furthermore, when two graphs are deduplicated according to their visible data, the library does not provide an automatic mechanism to merge their differing auxiliary metadata. Applications needing such aggregation must define the policy separately.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Source: `https://docs.rs/graphica/latest/graphica/struct.HiddenData.html`

## 3. Graph canonicalization and isomorphism

Canonicalization is the most important computational capability in Graphica.

It converts an arbitrary labeled graph into a canonical representative that is independent of permissible vertex renumbering.

Formally, for a graph \\(G\\), canonicalization computes a representative \\(C(G)\\) such that:

\\[ G_1 \cong G_2 \iff C(G_1)=C(G_2) \\]

Here, isomorphism preserves the vertex payloads, edge payloads, directionality, and edge multiplicity defined by Graphica's graph model.

### 3.1 McKay-style canonical labeling

Graphica implements a modified version of Brendan McKay's graph canonical-labeling approach, the family of algorithms associated with nauty.

Its implementation combines several mechanisms:

| Mechanism                    | Purpose                                                                   |
| ---------------------------- | ------------------------------------------------------------------------- |
| Initial vertex partitioning  | Group vertices by payload or color                                        |
| Partition refinement         | Distinguish vertices using their labeled and directed neighborhoods       |
| Individualization            | Select a vertex from a non-singleton partition to break symmetry          |
| Search-tree exploration      | Investigate alternative vertex labelings                                  |
| Canonical candidate ordering | Select the representative according to the library's graph-ordering rules |
| Automorphism discovery       | Recognize different labelings yielding the same graph                     |
| Orbit-based pruning          | Avoid exploration of branches known to be symmetry-equivalent             |

The refinement mechanism is particularly important. It examines the neighbors of vertices relative to the existing partitions, accounting for edge data and whether connections are incoming, outgoing, or undirected.

This means the algorithm is not simply comparing adjacency matrices. It progressively identifies structural distinctions, reducing the number of vertex permutations requiring further exploration.

When refinement cannot distinguish every vertex, the algorithm individualizes vertices and explores the corresponding search tree. Symmetry information discovered during the search is used to eliminate redundant alternatives.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



Implementation reference:

`https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs`

### 3.2 CanonicalForm: more than a canonical graph

Calling:

```
let canonical = graph.canonize();
```

returns:

```
pub struct CanonicalForm<N, E> {
    pub vertex_map: Vec<usize>,
    pub orbit_generators: Vec<Vec<Vec<usize>>>,
    pub orbit: Vec<usize>,
    pub automorphism_group_size: Integer,
    pub graph: Graph<N, E>,
}
```

Each field supports a different operation.

| Field                     | Meaning and architectural use                                                   |
| ------------------------- | ------------------------------------------------------------------------------- |
| `graph`                   | The canonical graph suitable for structural comparison and deduplication        |
| `vertex_map`              | Original vertex index to canonical vertex index mapping                         |
| `orbit_generators`        | Automorphism generators represented as collections of vertex permutation cycles |
| `orbit`                   | Vertex orbit-equivalence information, using representative indices              |
| `automorphism_group_size` | Exact automorphism-group cardinality, including edge-permutation factors        |

The `vertex_map` is particularly valuable for retaining relationships between original objects and their canonical counterparts.

For example, suppose an external data structure associates information with vertex 7 of the original graph. After canonicalization, that association can be transferred to:

```
let canonical_index = canonical.vertex_map[7];
```

Without an explicit vertex mapping, canonicalization would discard the correspondence between original identifiers and canonical identifiers, requiring a separate graph-matching procedure to recover it.

The canonical graph and its mapping are distinct outputs. The former establishes structural identity; the latter makes that identity practically usable for external data alignment.

The `orbit_generators` and `orbit` fields provide access to symmetry information that would otherwise require separate group-theoretic analysis.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



`https://docs.rs/graphica/latest/graphica/struct.CanonicalForm.html`

### 3.3 Canonicalization versus edge canonicalization

Graphica exposes two related but fundamentally different operations:

| Operation          | What it normalizes                                         | What it does not do                           |
| ------------------ | ---------------------------------------------------------- | --------------------------------------------- |
| `canonize_edges()` | Edge ordering and indices for a fixed vertex numbering     | Does not search vertex permutations           |
| `canonize()`       | Vertex numbering and edge ordering under graph isomorphism | Does not discard semantically relevant labels |

`canonize_edges()` sorts the edges and rebuilds the adjacency indices. It is useful when the vertices already have fixed identities and the difference between graphs is merely their edge insertion order.

`canonize()` performs the substantially more expensive isomorphism-class canonicalization.

These should not be used interchangeably.

An efficient architecture can sometimes avoid full canonicalization when vertex identities are already fixed and only deterministic edge ordering is required.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.4 Direct isomorphism testing

The method:

```
graph_a.is_isomorphic(&graph_b)
```

tests whether a label- and direction-preserving isomorphism exists.

The implementation uses preliminary rejection checks before full canonicalization:

1. Compare vertex and edge counts.
2. Check direct graph equality.
3. Compare initial vertex partitions and their payload information.
4. Refine the partitions and compare structural invariants.
5. Canonicalize and compare if the earlier checks do not settle the question.

These preliminary checks matter because non-isomorphic graphs can often be rejected without performing a complete canonical-label search.

A graph may be non-isomorphic to another even if its vertices and edges have identical counts, because different directionality, labels, connectivity, or multiplicities may prevent equivalence.

The method returns a Boolean, not an explicit isomorphism witness. When a mapping is needed, canonicalization of both graphs provides the basis for reconstructing a correspondence.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 3.5 Performance architecture

Graphica's canonicalizer contains several implementation-level optimizations:

- It selects smaller index representations according to vertex count: `u16`, `u32`, or `usize`.
- Partition refinement incorporates edge colors and directions before branching.
- It uses discovered automorphisms to prune equivalent search branches.
- It uses partition invariants to reject branches that cannot improve the selected canonical representative.
- It reuses some search-tree resources and tracks previously discovered canonical candidates.

These mechanisms reduce unnecessary search and memory overhead, particularly for graphs where vertex labels, directions, or neighborhood patterns distinguish many vertices.

However, canonical labeling remains a potentially expensive combinatorial problem. Highly symmetric graphs and adversarial graph families may still require substantial search.

The upstream material does not establish a universal runtime bound or benchmark demonstrating superiority over nauty, Traces, bliss, or other specialized canonical-labeling implementations. Consequently, no broad quantitative performance ranking is warranted.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



## 4. Automorphisms, orbits, and exact symmetry factors

Graphica's symmetry analysis is not merely an ancillary feature of canonicalization. It exposes information useful for reducing redundant computations and understanding how a graph can be transformed without changing its structure.

### 4.1 Vertex automorphisms

An automorphism is an isomorphism from a graph to itself.

For a graph with interchangeable vertices, multiple vertex permutations can preserve the entire graph.

Consider a cycle with four identically labeled vertices and four identically labeled undirected edges. Rotations and reflections preserve its topology. These transformations form its automorphism group.

Graphica discovers automorphisms during canonical-label search and represents generators as collections of vertex permutation cycles.

That can expose symmetry without having to enumerate every possible permutation separately.

### 4.2 Vertex orbits

A vertex orbit is the set of vertices that can be mapped into one another by some automorphism.

Vertices in the same orbit are structurally interchangeable under the graph's full label- and direction-preserving symmetry group.

This provides a useful distinction:

- Two vertices having the same degree does not establish that they are symmetry-equivalent.
- Two vertices having identical local neighborhoods may still be globally distinguishable.
- Vertices belonging to the same automorphism orbit are equivalent under an actual global graph symmetry.

Consequently, orbit information can support selection of canonical representatives, elimination of symmetric alternatives, and avoidance of equivalent evaluations.

The distinction between local structural similarity and global automorphic equivalence is particularly consequential for combinatorial algorithms.

### 4.3 Vertex symmetry and edge symmetry are both counted

A less obvious Graphica capability is its treatment of identical parallel edges.

The library distinguishes automorphisms induced by vertex permutations from permutations of indistinguishable edges while keeping the vertices fixed.

`get_edge_automorphism_group_size()` computes the latter contribution.

For a group of \\(m\\) indistinguishable parallel edges, the fixed-vertex edge-permutation contribution is:

\\[ m! \\]

For multiple groups of identical edges, the contributions multiply. Graphica also applies a factor of two for each self-loop, according to its documented convention.

The total reported group size is calculated as:

\\[ |\operatorname{Aut}(G)| = |\operatorname{Aut}\_{V}(G)| \cdot |\operatorname{Aut}\_{E}(G\mid V)| \\]

where the second factor represents permitted edge permutations with vertices fixed.

This is a consequential distinction from graph algorithms that count only vertex automorphisms.

Important: Graphica's self-loop factor applies to every stored self-loop, including directed self-loops. This is its implementation's symmetry-counting convention and should not automatically be equated with the usual vertex-automorphism count of a simple directed graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 4.4 Arbitrary-precision symmetry-group cardinalities

Automorphism-group sizes can grow factorially. A fixed-width machine integer is therefore inadequate for general exact counting.

Graphica uses Numerica's `Integer` type, which automatically moves between machine-sized and arbitrary-precision representations.

This enables exact representation of large symmetry-group cardinalities without forcing callers to implement big-integer management.

The distinction is valuable for applications using graph counts as mathematical coefficients, combinatorial multiplicities, or diagram symmetry factors.

Graphica does not itself perform every downstream calculation involving these factors. For example, constructing a coefficient \\(1/|\operatorname{Aut}(G)|\\) may involve rational arithmetic supplied by Numerica or another mathematical system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



Numerica reference:

`https://docs.rs/numerica/latest/numerica/domains/integer/`

### 4.5 Symmetry-aware structural processing

Combining canonical forms, vertex mappings, and symmetry-group sizes enables a useful general pattern:

1. Construct or generate a graph.
2. Obtain its canonical representative.
3. Use the representative to recognize equivalent structures.
4. Retain the automorphism-group size as an exact multiplicity or symmetry factor.
5. Use the canonical vertex mapping to align any associated data.

A key benefit is separating the number of possible descriptions or labelings of a structure from the number of genuinely different structures.

This can eliminate entire classes of duplicated work in structural enumeration problems, although any subsequent domain-specific calculations remain the caller's responsibility.

## 5. Exhaustive graph generation under structural constraints

Graph generation is Graphica's other major distinguishing capability.

Rather than merely generating random graphs or standard topologies, it can enumerate connected graphs whose vertices obey specified local attachment rules, while eliminating isomorphic duplicates.

The principal interface is:

```
Graph::<N, E>::generate(
    external_edges,
    vertex_signatures,
    settings,
)
```

Its return type is:

```
Result<
    HashMap<Graph<N, E>, Integer>,
    HashMap<Graph<N, E>, Integer>
>
```

The map contains canonical graphs as keys and their automorphism-group sizes as values.

This design means canonicalization and symmetry counting are integrated into enumeration, rather than being separate optional postprocessing tasks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 5.1 HalfEdge: the foundation of attachment constraints

Graph generation uses the `HalfEdge<E>` abstraction:

```
pub struct HalfEdge<E> {
    pub direction: Option<bool>,
    pub data: E,
}
```

The direction is interpreted relative to the vertex to which the half-edge attaches.

| Constructor                  | Direction            | Meaning                                         |
| ---------------------------- | -------------------- | ----------------------------------------------- |
| `HalfEdge::undirected(data)` | `None`               | No orientation                                  |
| `HalfEdge::incoming(data)`   | `Some(false)`        | Directed into the vertex                        |
| `HalfEdge::outgoing(data)`   | `Some(true)`         | Directed out of the vertex                      |
| `.flip()`                    | Reverses `Some(...)` | Produces the complementary attachment direction |

A directed edge connects complementary incoming and outgoing half-edges of the same edge-data type. An undirected edge connects compatible undirected half-edges.

This is a stronger generative abstraction than simply specifying a graph's degree distribution. It permits precise constraints on both the number and types of incident edges.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



`https://docs.rs/graphica/latest/graphica/struct.HalfEdge.html`

### 5.2 Vertex signatures

A vertex signature specifies an allowed multiset of incident half-edges.

For example:

```
let g = HalfEdge::undirected("g");
let q = HalfEdge::incoming("q");

let signatures = [
    vec![g, g, g],
    vec![q.flip(), q, g],
    vec![g, g, g, g],
];
```

These rules permit three kinds of vertices:

| Signature          | Meaning                                                |
| ------------------ | ------------------------------------------------------ |
| `[g, g, g]`        | Three undirected connections of type `g`               |
| `[q.flip(), q, g]` | One outgoing `q`, one incoming `q`, one undirected `g` |
| `[g, g, g, g]`     | Four undirected connections of type `g`                |

Graphica explores graphs satisfying these local connection patterns and the global generation settings.

Signatures are treated as connection multisets rather than ordered port arrangements. Their entries are sorted internally.

This is an important semantic distinction: Graphica models which connection types a vertex admits, not a fixed sequence of named ports with independent identities.

A separate application layer would be needed when port position, physical orientation, or a particular attachment ordering is semantically significant.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 5.3 External half-edges and internal vertices

The `external_edges` parameter is a list of pairs containing:

- The payload of an external vertex.
- The half-edge that must connect it to the generated graph.

Each external vertex is created with degree one. External vertices are initially inserted into the graph and are not directly connected to one another by the generator.

If two external vertices have the same semantically relevant payload, their interchange may be recognized as a graph symmetry. Different payloads can make those external vertices distinguishable.

Internally generated vertices are currently created using `N::default()`.

This exposes a relevant limitation: vertex signatures select attachment patterns, but the generator does not directly assign distinct internal node payloads for each signature.

Different internal vertex roles can be represented through distinguishable edge signatures, but explicit per-vertex-type payload assignment is not part of the documented `generate()` interface.

The generation API is therefore particularly natural when vertex type can be inferred from incident connections, and somewhat less direct when multiple domain-specific vertex identities share exactly the same attachment pattern.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 5.4 Generation constraints

The `GenerationSettings<N, E>` builder supports the following controls.

| Setting                       | Capability                                                     | Important distinction                    |
| ----------------------------- | -------------------------------------------------------------- | ---------------------------------------- |
| `max_vertices(n)`             | Maximum total vertex count                                     | Includes external vertices               |
| `max_loops(n)`                | Maximum independent cycle count                                | Not merely the number of self-loop edges |
| `max_bridges(n)`              | Maximum non-external bridge count                              | External leaf connections are excluded   |
| `allow_self_loops(bool)`      | Permit edges whose endpoints coincide                          | Default `false`                          |
| `allow_zero_flow_edges(bool)` | Permit bridges not required to connect external vertices       | Default `false`                          |
| `filter_fn(...)`              | Custom pruning of incomplete candidates                        | Called during recursive generation       |
| `abort_check(...)`            | Request early termination                                      | Returns partial results on cancellation  |
| `progress_fn(...)`            | Observe each newly discovered unique graph and optionally stop | Receives a graph, not an iterator        |

The structural constraints interact. For example, allowing self-loops while setting a small cycle limit permits some cyclic structures but excludes others. Bridge restrictions can substantially reduce generated graphs before their canonicalization and insertion into the results collection.

These are not merely post-generation filters: the implementation uses bounds and partial-structure checks to prune parts of the recursive search.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



### 5.5 Cycles versus self-loops

The term loop requires particular care.

In graph theory, a self-loop is an edge connecting a vertex to itself. In Graphica's cycle-counting methods, however, the principal notion is the number of independent cycles in a connected graph.

For a connected graph:

\\[ L = E - V + 1 \\]

where \\(E\\) is the edge count and \\(V\\) is the vertex count.

A graph may therefore have several independent cycles without containing a single self-loop edge.

Conversely, a self-loop contributes one independent cycle.

This is why `max_loops()` and `allow_self_loops()` serve different purposes.

The formula also explains an implementation caveat: the public `num_loops()` method evaluates `E + 1 - V` directly. It is meaningful as a cycle-rank calculation for connected graphs, but it does not provide the general disconnected-graph formula \\(E-V+C\\), where \\(C\\) is the number of connected components.

On some disconnected graphs, its unsigned arithmetic can underflow. For an empty graph, the expression evaluates to 1, which should not be interpreted as one actual cycle.

Use `num_loops()` with its connected-graph assumptions in mind.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 5.6 Custom pruning through filter_fn

`filter_fn` is one of the most valuable extension points.

The callback receives a reference to the current incomplete graph and a vertex-count boundary indicating how many initial vertices have been completed.

This allows graph generation to incorporate domain-specific structural constraints without modifying the enumeration algorithm.

Examples include restricting the number of vertices with particular attachment patterns, rejecting forbidden local motifs, or enforcing constraints that can be determined from completed vertices.

An especially important property is that the callback operates during generation, before the entire graph is complete.

A sound pruning rule must therefore be safe for partial graphs. Rejecting a partial graph discards all of its possible extensions.

For example, rejecting a candidate because it already contains too many permanently completed vertices of a forbidden type is safe. Rejecting it because it currently lacks a feature that could still be added later is generally not safe.

This is the difference between correct constraint propagation and merely applying a predicate to an incomplete result.

The upstream tests include an example limiting the number of completed vertices matching a particular edge-type signature.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 5.7 How enumeration avoids redundant structures

The generation implementation is a recursive backtracking search.

At a high level, it:

1. Creates the external vertices and their required attachments.
2. Chooses a permitted signature for the current internal vertex.
3. Distributes remaining half-edges among compatible vertices.
4. Creates new internal vertices when necessary.
5. Checks partial compatibility, degree bounds, and structural restrictions.
6. Validates completed graph connectivity and bridge constraints.
7. Canonicalizes accepted graphs and inserts them into the result map.

There are several layers of redundancy control.

Local symmetry pruning: When existing candidate vertices have equivalent partial attachment patterns, the generator can avoid exploring assignments to structurally equivalent destinations.

Partial attachment validation: The implementation precomputes permitted partial signature patterns. A developing vertex that can no longer satisfy any allowed final signature can be rejected before completing the graph.

Cycle and degree bounds: These eliminate configurations that cannot lead to acceptable complete graphs.

Global canonical deduplication: Completed graphs are canonicalized before insertion into the result map, eliminating isomorphic duplicates that survive earlier pruning.

This combination is significantly more capable than naive generate-all-then-compare enumeration. The degree of improvement nevertheless depends on how restrictive the signatures and generation settings are.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



### 5.8 Termination, cancellation, and result accumulation

Several operational details deserve attention.

A termination control is mandatory. If none of `max_vertices`, `max_loops`, `progress_fn`, or `abort_check` is specified, the generator panics to prevent unbounded enumeration. Supplying only `max_bridges` or a filter function does not satisfy this requirement.

Cancellation preserves generated results. `generate()` returns `Ok(map)` after normal completion and `Err(map)` when generation is aborted. Thus `Err` is not a conventional error message; it contains the partial set of discovered graphs.

Results accumulate in memory. The API returns a `HashMap` rather than a lazy iterator. Progress callbacks can observe individual discoveries, but the library still retains the generated graph map.

Progress is based on unique discoveries. `progress_fn` runs when a previously unseen canonical graph is inserted, not necessarily on every recursive branch or every completed candidate.

Abort polling is not continuous. In the inspected implementation, `abort_check` is evaluated upon reaching a connected completed candidate. It should not be assumed to provide immediate cancellation in every part of an expensive recursive search.

These distinctions affect the design of large enumerations, bounded searches, and user-facing cancellation mechanisms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



### 5.9 Relevance to Feynman-diagram generation

The Symbolica documentation explicitly demonstrates Graphica-style generation for Feynman diagrams.

In that application, half-edge labels represent particle or field types, vertex signatures represent permitted interactions, and cycle constraints correspond to diagram loop order.

For each unique diagram topology, the automorphism-group size supplies the mathematical symmetry factor used in diagrammatic calculations.

Symbolica demonstrates calculations using the reciprocal of the returned group size:

\\[ \text{Symmetry factor} = \frac{1}{|\operatorname{Aut}(G)|} \\]

This provides a concrete example of Graphica's two most significant features working together: graph generation and exact symmetry analysis.

The ability to generate typed multigraphs and identify their equivalences is more general than particle physics, however. Comparable structural problems arise in combinatorial chemistry, graph rewriting, operator networks, and topology enumeration. These are architectural possibilities, not claims of built-in domain-specific solvers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra



Symbolica graph documentation:

`https://symbolica.io/docs/graphs/`

## 6. Structural graph analysis: spanning trees, bridges, and connectivity

Although Graphica is not a general-purpose graph-algorithms library, it includes several structural analysis mechanisms that support graph generation and can also be used independently.

These facilities are particularly relevant to identifying cycles, bridges, and the topology of connected graphs.

### 6.1 Depth-first spanning trees

`Graph::get_spanning_tree(start_vertex)` constructs a depth-first spanning tree rooted at a specified vertex.

The result is a `SpanningTree` containing node-level DFS information and the order in which vertices were visited.

Its principal data structures are:

| Type           | Exposed information                                                                     |
| -------------- | --------------------------------------------------------------------------------------- |
| `SpanningTree` | Node information and DFS visitation order                                               |
| `NodeInfo`     | Parent, DFS position, parent edge, chain assignment, external-vertex status, back-edges |
| `BackEdge`     | Target vertex and associated chain identifier                                           |

The parent relationship is represented by vertex indices. The root's parent is the root itself.

`NodeInfo.external` indicates whether the vertex has valence one in the original graph. This is a topological classification, not a persistent marker identifying vertices originally supplied as external arguments to `generate()`.

That distinction matters when the two concepts are not equivalent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 6.2 Chain decomposition

`SpanningTree::chain_decomposition()` assigns chain identifiers to vertices and back-edges.

The decomposition is used to determine which edges participate in cycles and which are bridges.

A bridge is an edge whose removal would disconnect the underlying undirected graph.

The exposed methods include:

| Method                    | Result                                                                 |
| ------------------------- | ---------------------------------------------------------------------- |
| `chain_decomposition()`   | Annotates tree nodes and back-edges with chain information             |
| `bridges()`               | Iterates over non-external bridge edge indices                         |
| `count_bridges()`         | Counts those bridges                                                   |
| `has_zero_flow_bridges()` | Detects bridges that do not contribute to connecting external vertices |
| `is_connected()`          | Checks spanning-tree connectivity                                      |

`chain_decomposition()` must be called before relying on bridge information derived from chain assignments.

The ability to classify zero-flow bridges is more specialized than conventional graph connectivity testing.

For a graph with designated external legs, one may want to exclude a bridge that leads to an internal substructure containing no external vertex. Such a branch is structurally connected but unnecessary for connecting external terminals.

Graphica incorporates this distinction directly into its generation constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 6.3 Important connectivity semantics

Graphica's traversal operates on the underlying adjacency graph, without restricting traversal to the direction of a directed edge.

Consequently, `is_connected()` checks weak connectivity, not strong connectivity.

For example:

```
A → B → C
```

The graph is connected under Graphica's traversal semantics, even though C cannot reach A by following directed edges.

This also applies to spanning-tree construction and bridge analysis: directionality is not used to define directed reachability.

For algorithms requiring strongly connected components, directed cycles, dominator trees, or directed reachability, a different algorithmic implementation is necessary.

The source also treats an empty graph as connected. This is an explicit implementation convention that may differ from an application's preferred definition.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 6.4 Relationship to graph generation

These structural methods serve multiple roles inside the generator.

Chain decomposition supports bridge restrictions. Connectivity ensures only connected completed candidates are accepted. The cycle-rank calculation supports generation bounds. External-vertex information supports zero-flow bridge elimination.

This is an important architectural characteristic: Graphica implements enough specialized graph analysis to make its constrained generator self-contained.

It does not depend on an external graph-processing engine to perform these fundamental structural checks.

## 7. Mutation, graph lifecycle, and data access

Graphica provides a relatively small set of graph mutation primitives.

| API                                        | Behavior                                                  |
| ------------------------------------------ | --------------------------------------------------------- |
| `Graph::new()`                             | Creates an empty graph                                    |
| `add_node(data)`                           | Adds a vertex and returns its index                       |
| `add_edge(source, target, directed, data)` | Adds an edge and returns its index                        |
| `set_node_data(index, data)`               | Replaces vertex payload and returns the previous value    |
| `set_edge_data(index, data)`               | Replaces edge payload and returns the previous value      |
| `set_directed(index, directed)`            | Changes an edge's direction flag and returns the old flag |
| `delete_last_edge()`                       | Removes the most recently added edge                      |
| `delete_last_empty_node()`                 | Removes the final vertex if it has no edges               |
| `node(index)` / `edge(index)`              | Reads indexed vertex or edge information                  |
| `nodes()` / `edges()`                      | Returns immutable slices of stored vertices and edges     |

The two deletion operations are documented as \\(O(1)\\), which is consistent with the vector-backed representation and its append/pop behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.1 Optimized for construction and backtracking

The mutation model reveals an important design choice.

Graphica supports fast append operations and last-in-first-out deletion rather than unrestricted deletion of arbitrary vertices and edges.

This is particularly suitable for its recursive graph generator:

1. Add an edge or temporary vertex.
2. Recursively explore candidate completions.
3. Remove the last edge or empty vertex.
4. Explore the next possibility.

The implementation therefore avoids requiring complex index-remapping machinery during recursive enumeration.

For long-lived graphs that require frequent deletion of arbitrary vertices or edges, this is a less convenient mutation model than libraries offering stable handles, tombstones, generational arenas, or explicit removal and compaction policies.

### 7.2 Index and direction invariants

Graphica validates endpoint indices when adding an edge. Invalid indices produce an error.

Other indexed accessors and setters use direct vector indexing, so invalid indices can panic.

The following detail also deserves attention: `set_directed()` changes the direction flag without changing the stored endpoint ordering.

Because an undirected edge is normalized by endpoint index when created, converting it to a directed edge makes its stored first endpoint the source.

This can differ from the endpoint order originally passed to `add_edge()` when the edge was undirected.

A caller that needs to preserve an independently defined source/target orientation should not assume toggling the direction flag recovers that original orientation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 7.3 Mutation and canonical identity

The graph's payloads, directions, and edges all participate in structural identity.

Changing any of these after canonicalization can invalidate the previously computed canonical representation or associated equivalence assumptions.

Similarly, inserting a mutable graph into a hash-based collection and then changing its identity-relevant data would be unsafe for collection invariants.

Graphica's Rust API requires mutable access for mutation, which naturally prevents ordinary mutation through an immutable reference to a graph stored as a `HashMap` key. However, any system maintaining separate canonical keys, cached hashes, or mappings must manage their invalidation.

A canonical representation should therefore be understood as a snapshot of a graph's structure, not a persistent identity automatically maintained through subsequent edits.

## 8. Visualization and graph export

Graphica provides textual export to two widely used graph-description formats.

### Graphviz DOT

```
let dot = graph.to_dot();
```

The implementation emits a `digraph` and represents undirected edges using `dir=none`.

Consequently, the generated DOT can represent mixed edge directions visually even though the outer graph declaration uses the directed-graph form.

### Mermaid

```
let mermaid = graph.to_mermaid();
```

The generated Mermaid syntax uses directed or undirected connectors according to each edge's direction flag.

Both exporters require `Display` implementations for the vertex and edge data types.

These capabilities are useful for debugging, documentation, structural inspection, and generated engineering reports.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



There are two important boundaries.

First, Graphica generates graph-description strings. It does not provide a complete visualization engine, interactive graph viewer, layout optimizer, or image renderer.

Second, its exporters construct strings using displayed payload values. The inspected implementation does not provide a general-purpose escaping or sanitization layer for arbitrary user-provided labels.

Payloads containing quotation marks or other syntax-significant characters may therefore need escaping before being used in generated DOT or Mermaid.

The same consideration applies to `HiddenData`: hidden metadata can appear in exported labels through its `Display` implementation.

## 9. Rust integration, dependencies, and build features

### 9.1 Basic dependency

For the researched version:

```
[dependencies]
graphica = "3.0.1"
```

The core graph types are accessible directly through the crate.

### 9.2 Generic payload constraints

Graphica's generic design allows relatively unconstrained payload types for basic graph construction.

More advanced operations impose additional trait requirements.

| Operation                    | Important payload requirements                                         |
| ---------------------------- | ---------------------------------------------------------------------- |
| Basic construction           | No general equality or ordering requirement                            |
| Graph comparison and hashing | Corresponding `Eq`, `Hash`, or ordering traits                         |
| DOT and Mermaid export       | `Display`                                                              |
| Edge canonicalization        | Edge data implementing `Eq`, `Ord`, and `Hash`                         |
| Full canonicalization        | Cloneable, ordered, hashable, equality-comparable vertex and edge data |
| Graph generation             | The above properties, plus `Default` for vertex data                   |

The generation requirement for `Default` follows from the creation of internal vertices using the default vertex payload.

A practical consequence is that arbitrary floating-point payloads cannot directly satisfy all canonicalization constraints because standard floating-point types do not implement a total `Ord` ordering.

Applications needing floating-point labels must choose an appropriate ordered representation or domain-specific wrapper, taking care that equality, ordering, and hashing encode the same intended semantics.

For example, a measured physical quantity may require tolerance-based equivalence or quantization rather than exact floating-point equality. Graphica does not impose that policy automatically.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.3 Numerica integration

Graphica depends on Numerica 3.x.

Numerica provides the exact integer type used in symmetry-group calculations, along with combinatorial routines that Graphica uses to construct allowed partial attachment patterns.

This is an internal dependency, not a requirement to use Symbolica's symbolic-expression engine.

The Cargo manifest enables `numerica/default` through Graphica's default feature.

Numerica's own 3.0.1 default features select GMP-backed integers and MPFR-backed floating-point arithmetic. Alternative backends are available in Numerica, including Malachite for integers, but Graphica does not expose a comprehensive dedicated backend-selection interface of its own.

Build configurations that disable defaults or change the arithmetic backend should therefore be considered in terms of the combined Cargo feature graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



Sources:

`https://github.com/symbolica-dev/numerica`

`https://docs.rs/numerica/latest/numerica/`

### 9.4 Optional serialization features: a qualification

The Graphica 3.0.1 manifest declares:

```
[features]
serde = ["dep:serde"]
bincode = [
    "dep:bincode",
    "dep:bincode-trait-derive",
]
```

However, the inspected main `lib.rs` does not contain corresponding `serde` or `bincode` implementations or feature-gated serialization code for its public graph types.

Therefore, the presence of these optional features is not sufficient evidence that `Graph<N, E>` directly implements `Serialize`, `Deserialize`, `Encode`, or `Decode`.

This is an area where the declared manifest features and demonstrable public functionality need to be distinguished.

For applications requiring durable persistence, using an explicit intermediate representation of vertex data, edge data, directionality, and endpoints is a viable general strategy. Reconstruction can then use the normal graph-construction API.

That would be application-defined serialization rather than a verified native Graphica serialization capability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 9.5 Concurrency

Graphica's core graph operations are ordinary synchronous Rust methods.

The generation callback traits require cloneable, `Send + Sync` callback implementations, and graph values can participate in Rust's usual thread-safety model when their payload types permit it.

The inspected implementation does not expose a dedicated parallel enumeration API, configurable thread pool, GPU execution backend, or distributed graph-generation facility.

It would be possible to parallelize independent externally partitioned workloads around Graphica, but such partitioning and result merging would belong to the surrounding application.

The sequential search implementation itself should not be described as a native parallel graph enumerator.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 10. Relationship to Symbolica and other graph libraries

### 10.1 Graphica and Symbolica

Graphica originated within the Symbolica ecosystem, but it is distributed as a separate MIT-licensed Rust crate.

The Symbolica Rust library also re-exports Graphica:

```
pub use graphica as graph;
```

This makes Graphica's functionality available through Symbolica's Rust graph module.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Symbolica's Python package exposes a related `Graph` API for constructing, canonicalizing, comparing, and generating graphs, using Symbolica expressions as payloads.

That Python API is not identical to the generic Rust crate interface. The Rust library's ability to use arbitrary payload types is an important distinction.

Graphica itself does not require using Symbolica's commercial computer-algebra capabilities.

Source: `https://symbolica.io/docs/python_api/Graph/`

### 10.2 Relationship to petgraph and other graph engines

Graphica's specialization becomes clearer when compared with adjacent libraries.

| Library          | Principal emphasis                                                                  | Relationship to Graphica                                                                     |
| ---------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| `graphica`       | Canonical graph labeling, exact symmetry analysis, constrained graph enumeration    | The specialization examined here                                                             |
| `petgraph`       | General graph representations, traversal, paths, graph algorithms                   | A broader foundation for general graph-processing workflows                                  |
| `rustworkx-core` | High-performance graph-algorithm implementations based around Rust graph structures | Useful for analysis operations beyond Graphica's scope                                       |
| `graphina`       | High-level graph analysis, centrality, communities, metrics, graph mining           | Oriented toward network data science                                                         |
| nauty / Traces   | Specialized canonical labeling, isomorphism, automorphism analysis                  | Algorithmically adjacent alternatives warranting comparison for canonical-labeling workloads |
| bliss            | Graph automorphism and canonical-labeling algorithms                                | Another specialized alternative for symmetry-analysis workloads                              |

Graphica is particularly distinctive in combining mixed-direction labeled multigraphs with direct constrained generation and automorphism-aware deduplication.

A general-purpose graph library may be preferable for shortest paths, large-scale connectivity analysis, graph transformations, and conventional network algorithms.

A specialist canonical-labeling engine may be appropriate when large or difficult graph-isomorphism instances dominate performance.

These distinctions are about capabilities, not a demonstrated performance ranking.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 10.3 Complementary use rather than replacement

Graphica can occupy a dedicated role alongside broader graph infrastructure.

For example, a generic graph-processing workflow might involve constructing and analyzing graphs in a general-purpose graph library, converting selected graph instances to Graphica for canonicalization or symmetry calculations, and transferring the results back to the original representation.

Graphica's `vertex_map` helps preserve identity correspondences during such operations.

However, the inspected crate does not supply a native `petgraph` adapter or general interchange trait implementation. Conversion requires an explicit mapping between vertex indices, edge directions, multiplicities, and payloads.

Whether that conversion cost is justified depends on the frequency and significance of canonicalization relative to the other graph operations.

## 11. Important limitations and engineering considerations

The following distinctions define the practical boundaries of Graphica 3.0.1.

| Area                     | Limitation or qualification                                                                          | Consequence                                                         |
| ------------------------ | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Algorithms               | Focused on canonicalization, generation, and a small set of structural methods                       | Not a replacement for a full graph-algorithm library                |
| Graph generation         | Enumerates connected graphs only                                                                     | Disconnected topology generation requires a separate strategy       |
| Internal vertex identity | Generated internal vertices use `N::default()`                                                       | Signature-specific internal payloads are not directly supported     |
| Scale                    | Exhaustive generation can grow combinatorially                                                       | Bounds and safe pruning rules are important                         |
| Result storage           | Generation accumulates canonical graphs in memory                                                    | Large output sets may be memory-intensive                           |
| Cancellation             | Abort checks occur at selected search stages                                                         | Immediate cancellation is not guaranteed                            |
| Mutability               | Direct deletion is restricted to the last edge or last empty vertex                                  | Less convenient for highly dynamic graph editing                    |
| Connectivity             | Directionality is ignored for spanning-tree connectivity                                             | Strongly connected components are not provided                      |
| Identity                 | Hashing and ordinary equality are representation-sensitive                                           | Canonicalization is needed for isomorphism-based deduplication      |
| Serialization            | Cargo flags exist, but public graph serialization is not established by the inspected implementation | Persistence requires additional verification or explicit conversion |
| Visualization            | Exports DOT and Mermaid strings                                                                      | Rendering and interactive visualization are external                |
| Performance              | No verified cross-library benchmark establishing a universal advantage                               | Workload-specific comparison remains necessary                      |

Two additional implementation characteristics are worth highlighting.

Canonicalization search memory. The upstream implementation retains discovered search leaves in a hash map, with a source comment identifying the potential need to limit its growth. This indicates that memory use is not determined solely by the size of the input graph; the canonical-label search itself may consume substantial intermediate memory on difficult graphs.

Label semantics. Canonicalization relies on the consistency of Rust equality, ordering, and hashing for payload types. Domain-level notions of equivalence—such as approximate numeric equality, aliases, semantically equivalent identifiers, or unordered compound payloads—must be represented through appropriate payload types before canonicalization. Graphica does not automatically infer these equivalences.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



## 12. Compact verified Rust example

The following reproduces the central generation pattern from the upstream documentation, with explicit Rust types.

```

use graphica::{GenerationSettings, Graph, HalfEdge};

fn main() {
    let g = HalfEdge::undirected("g");
    let q = HalfEdge::incoming("q");

    let graphs = Graph::<i32, &str>::generate(
        &[(1, g), (2, g)],
        &[
            vec![g, g, g],
            vec![q.flip(), q, g],
            vec![g, g, g, g],
        ],
        GenerationSettings::new()
            .max_loops(2)
            .max_bridges(0)
            .allow_self_loops(true),
    )
    .unwrap();

    assert_eq!(graphs.len(), 19);

    for (canonical_graph, group_size) in &graphs {
        println!("Automorphism group size: {group_size}");
        println!("{}", canonical_graph.to_dot());
    }
}

```

The example demonstrates several capabilities in combination: typed half-edge signatures, constrained exhaustive generation, elimination of isomorphic duplicates, arbitrary-precision symmetry-group cardinalities, and graph export.

The upstream documentation states that these settings yield 19 unique graphs. This example is source-derived; it was not independently compiled or benchmarked for this overview.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 13. Architectural significance

Graphica's greatest value is not any individual graph API. It is the ability to treat graph structure, structural equivalence, and symmetry as explicit computational objects.

Three broader architectural possibilities follow.

### Symmetry-reduced enumeration

When a problem requires enumerating many possible structures, canonicalization allows the search to operate on isomorphism classes instead of treating every vertex labeling as a distinct result.

Graphica goes beyond a postprocessing deduplication utility because this capability is integrated with constrained generation and local symmetry pruning.

That can substantially simplify custom combinatorial generators.

### Canonical structural identity

A canonical graph can serve as an identity key independent of arbitrary vertex numbering.

This is valuable for deduplication, memoization, caching, comparison of independently generated topologies, and reconciliation of structurally equivalent objects.

`vertex_map` provides the mapping needed to associate that canonical identity with the original graph's elements.

The qualification is that canonical identity depends on payload-equivalence rules and on the library's canonicalization semantics. It should not automatically be regarded as a version-independent binary or string serialization format.

### Exact combinatorial accounting

Automorphism-group analysis permits a workflow to distinguish one structural configuration from its many equivalent labelings and edge permutations.

This supports exact multiplicity and symmetry-factor calculations without implementing a separate automorphism-counting engine.

The combination is particularly powerful when the output of enumeration is a set of mathematically distinct objects rather than merely a collection of graph data structures.

## 14. Primary source reference map

| Subject                                  | Direct source URL                                                          |
| ---------------------------------------- | -------------------------------------------------------------------------- |
| Repository and development               | `https://github.com/symbolica-dev/graphica`                                |
| Rust API                                 | `https://docs.rs/graphica/latest/graphica/`                                |
| Graph implementation and algorithms      | `https://raw.githubusercontent.com/symbolica-dev/graphica/main/src/lib.rs` |
| Graph type and methods                   | `https://docs.rs/graphica/latest/graphica/struct.Graph.html`               |
| Generation settings                      | `https://docs.rs/graphica/latest/graphica/struct.GenerationSettings.html`  |
| Canonical-form data structure            | `https://docs.rs/graphica/latest/graphica/struct.CanonicalForm.html`       |
| Half-edge representation                 | `https://docs.rs/graphica/latest/graphica/struct.HalfEdge.html`            |
| Cargo version and features               | `https://raw.githubusercontent.com/symbolica-dev/graphica/main/Cargo.toml` |
| Numerica integer and arithmetic support  | `https://docs.rs/numerica/latest/numerica/`                                |
| Symbolica graph-generation documentation | `https://symbolica.io/docs/graphs/`                                        |
| Symbolica Python graph API               | `https://symbolica.io/docs/python_api/Graph/`                              |

## Overall assessment

Graphica 3.0.1 is a compact but mathematically sophisticated Rust library. Its defining capabilities are modified McKay-style canonical labeling, exact automorphism analysis, generation of connected multigraphs from typed half-edge constraints, and symmetry-aware structural deduplication.

Its less obvious strengths include explicit canonical vertex mappings, edge-permutation contributions to automorphism groups, custom partial-graph pruning, chain-based bridge analysis, and the ability to separate structural identity from auxiliary metadata using `HiddenData`.

Its principal limitation is scope: it is not designed as a comprehensive graph-analysis, graph-storage, or graph-transformation platform.

The most consequential reason to investigate Graphica is when graph equivalence and symmetry are central to the problem itself. In that setting, it can replace a substantial amount of bespoke canonicalization, automorphism counting, constrained enumeration, and deduplication machinery with a coherent native Rust implementation.

# Graphica (Rust) — Capability Addendum

Version: 3.0.1 | Research date: November 9, 2026

This addendum supplements the original Graphica overview with previously omitted capabilities, more precise API semantics, and implementation-level constraints. It focuses on functionality that changes how Graphica can be used for graph enumeration, canonical structural identity, automorphism analysis, and graph manipulation.

The source review covers Graphica 3.0.1, including its Rust implementation, public API, generation tests, and the release commit correcting self-loop symmetry counting. Some indexed API pages still reflect older versions; version-sensitive conclusions below follow the current upstream implementation.

Primary implementation: [https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs](https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs?utm_source=chatgpt.com)

## 1. Corrected automorphism semantics: directed and undirected self-loops

Graphica 3.0.1 corrects the symmetry contribution of directed self-loops. This supersedes the corresponding explanation in the original overview.

The important distinction is between permutations of indistinguishable parallel edges and reversals of individual undirected self-loops.

In Graphica 3.0.1, the extra factor of two applies only to undirected self-loops:

```
if !e.directed && e.vertices.0 == e.vertices.1 {
    count *= 2;
}
```

For \\(m\\) identically labeled self-loops on the same vertex, with that vertex fixed:

\\[ |\operatorname{Aut}\_E|= \begin{cases} m!, & \text{directed self-loops}\\\ 2^m m!, & \text{undirected self-loops} \end{cases} \\]

The distinction arises because undirected self-loops permit reversal of their two incidences under Graphica's symmetry-counting convention, whereas directed self-loops do not.

| Edge configuration                    | Fixed-vertex edge automorphism factor |
| ------------------------------------- | ------------------------------------- |
| One directed self-loop                | 1                                     |
| One undirected self-loop              | 2                                     |
| Two identical directed self-loops     | 2                                     |
| Two identical undirected self-loops   | 8                                     |
| Three identical directed self-loops   | 6                                     |
| Three identical undirected self-loops | 48                                    |

Parallel edges with different data, direction flags, or ordered endpoint pairs are not grouped as interchangeable by `get_edge_automorphism_group_size()`.

The corrected treatment was introduced in the release commit titled Fix symmetry contribution of directed self-edge, which also advanced the crate version from 3.0.0 to 3.0.1.

Source: [https://github.com/symbolica-dev/graphica/commit/6156fc1b35f7c9a46e11c62b54508715a2559265](https://github.com/symbolica-dev/graphica/commit/6156fc1b35f7c9a46e11c62b54508715a2559265?utm_source=chatgpt.com)

### 1.1 Vertex generators versus total automorphism-group cardinality

`CanonicalForm` exposes both:

```
pub orbit_generators: Vec<Vec<Vec<usize>>>,
pub automorphism_group_size: Integer,
```

These do not describe the same mathematical object at the same level of detail.

`orbit_generators` represents discovered automorphisms as vertex permutations, encoded through cycles. It does not enumerate individual permutations of indistinguishable edges.

The reported automorphism-group size additionally includes the fixed-vertex edge-permutation factor:

\\[ |\operatorname{Aut}(G)| = |\operatorname{Aut}\_V(G)| \cdot |\operatorname{Aut}\_E(G\mid V)| \\]

Graphica obtains the vertex contribution during canonical-label search and combines it with `get_edge_automorphism_group_size()`.

Consequently, the total reported group size may be larger than the order of the permutation group generated by the exposed vertex permutations alone.

This matters when an application needs actual symmetry transformations rather than only their total number. The exposed generators are sufficient to investigate vertex automorphisms, but Graphica does not expose corresponding generators for all edge permutations and self-loop reversals.

The standalone `get_edge_automorphism_group_size()` method is also independently useful: it calculates fixed-vertex edge symmetries without requiring full canonicalization.

## 2. Generation of connected graphs with no external edges

Graphica's `Graph::generate()` supports an empty `external_edges` collection.

This enables enumeration of closed connected graph topologies whose vertices and edges are all internal to the generated structure.

The implementation explicitly initializes a default-valued internal seed vertex when no external edges are supplied:

```
if external_edges.is_empty() {
    edge_signatures.push(vec![]);
    g.add_node(N::default());
}
```

This means external legs are optional, not structurally required by the generator.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 2.1 Closed-graph generation

A representative invocation is:

```
use graphica::{GenerationSettings, Graph, HalfEdge};

let e = HalfEdge::undirected("e");

let signatures = [
    vec![e, e],
];

let graphs = Graph::<(), &str>::generate(
    &[],
    &signatures,
    GenerationSettings::new()
        .max_vertices(8)
        .max_loops(1)
        .allow_zero_flow_edges(true),
);
```

This is an illustrative source-derived example, not an independently executed test.

The signature describes vertices with two undirected incidences. With no external vertices, permitted connected completions can include cycles formed entirely from internal vertices.

Such structures appear in vacuum-diagram enumeration, closed interaction networks, combinatorial topology generation, and other settings where there are no designated terminal vertices.

The key architectural consequence is that Graphica can generate both externally anchored structures and closed structures using the same signature-driven enumeration engine.

### 2.2 Interaction with zero-flow bridge restrictions

The default setting is:

```
allow_zero_flow_edges(false)
```

When a graph contains no degree-one vertices, Graphica's zero-flow bridge check treats any non-external bridge as disallowed.

Consequently, the default can remove closed connected graphs containing bridges even when they otherwise satisfy the vertex signatures and cycle-count constraints.

The interpretation is specialized to Graphica's generation model. It should not be understood as a universal definition of a bridge carrying zero flow in an arbitrary physical or computational network.

For closed-graph enumeration where bridge-containing structures are valid, `allow_zero_flow_edges(true)` changes the admissible topology set.

The interaction with degree-one vertices requires further care, as detailed in Section 6.

### 2.3 Vertex signatures specify allowed kinds, not required counts

A useful additional distinction is that `vertex_signatures` is a collection of permitted vertex attachment patterns, not an inventory of vertices that must each appear once.

For example:

```
let signatures = [
    vec![e, e],
    vec![e, e, e],
];
```

does not require exactly one degree-two vertex and one degree-three vertex.

Instead, the generator can use these permitted patterns repeatedly, subject to its global constraints and connectivity requirements.

This makes the signature model flexible for open-ended structural enumeration, but it does not natively express exact multiplicity requirements for each internal vertex kind.

Those requirements can be enforced through suitable filtering of partial or completed graphs.

There is a related limitation: all generated internal vertices receive `N::default()`. A vertex's chosen signature does not automatically become its node payload. When distinct signatures must correspond to distinct node-data values, an additional representation or transformation is necessary.

### 2.4 A cycle bound is not always a finite size bound

A further consequence of repeated signatures is that `max_loops()` does not mathematically bound the number of vertices in every possible signature system.

Consider connected undirected cycles in which every vertex has degree two:

\\[ C_3,\ C_4,\ C_5,\ldots \\]

All have:

\\[ E=V,\qquad L=E-V+1=1 \\]

Their vertex counts grow indefinitely while their independent cycle count remains one.

The generator requires at least one termination-related setting, such as `max_vertices`, `max_loops`, `progress_fn`, or `abort_check`. But satisfying this API check using `max_loops` alone does not guarantee that the admissible graph family is finite.

The explicit `max_vertices()` setting supplies a structural size bound independent of cycle count.

This distinction is important for any enumeration using vertex signatures that permit arbitrarily long chains, cycles, or subdivisions. It follows from the graph mathematics and the generator's documented bound semantics; it is not a measured nontermination claim for a particular execution.

## 3. Reconstructing graph isomorphisms from canonical vertex maps

`Graph::is_isomorphic()` provides a Boolean result. However, the `vertex_map` field returned by `canonize()` makes it possible to reconstruct an explicit correspondence between the vertices of two isomorphic graphs.

This is useful when structural equivalence must be accompanied by object-level alignment.

### 3.1 Deriving an isomorphism witness

Suppose two graphs \\(A\\) and \\(B\\) canonicalize to the same graph \\(C\\).

Their canonicalization results supply mappings:

\\[ c_A:V(A)\rightarrow V(C) \\]

\\[ c_B:V(B)\rightarrow V(C) \\]

A vertex mapping from \\(A\\) to \\(B\\) is then:

\\[ f\_{A\rightarrow B}=c_B^{-1}\circ c_A \\]

Because the canonical mappings are permutations of vertex indices, the inverse can be constructed without additional graph matching.

For example:

```
let ca = a.canonize();
let cb = b.canonize();

assert!(ca.graph == cb.graph);

// Invert B's original-to-canonical mapping.
let mut inverse_b = vec![0; cb.vertex_map.len()];

for (original, &canonical) in cb.vertex_map.iter().enumerate() {
    inverse_b[canonical] = original;
}

// Map each original vertex of A to a vertex of B.
let a_to_b: Vec<usize> = ca.vertex_map
    .iter()
    .map(|&canonical| inverse_b[canonical])
    .collect();
```

This is an illustrative composition of documented API outputs, not a separately supplied Graphica method.

The resulting mapping preserves the vertex labels, edge labels, multiplicities, and directionality defining the graph isomorphism.

When a graph has nontrivial automorphisms, the correspondence need not be unique. Several equally valid vertex mappings may exist.

### 3.2 No equivalent canonical edge mapping

`CanonicalForm` exposes an original-to-canonical vertex map, but not an original-to-canonical edge map.

During canonicalization, Graphica reconstructs and sorts the canonical edge collection. Edge indices therefore need not correspond to their original indices.

The difference matters because vertex and edge identity have different ambiguity properties.

For vertices, the canonicalization algorithm produces an explicit permutation even when several permutations are equally valid.

For indistinguishable parallel edges, there may be no uniquely meaningful correspondence between their original identities and their canonical positions.

Consequently, a system attaching independent records to individual edges must preserve or reconstruct that correspondence separately.

One option is to include distinguishing identifiers in edge payloads, but this changes the equivalence relation: edges with different identity-bearing payloads are no longer interchangeable.

Another is to preserve an external association and reconcile equivalent edges according to application-specific rules.

The correct choice depends on whether edge identity is part of structural equivalence or merely auxiliary bookkeeping.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



[https://docs.rs/graphica/latest/graphica/struct.CanonicalForm.html](https://docs.rs/graphica/latest/graphica/struct.CanonicalForm.html?utm_source=chatgpt.com)

## 4. HiddenData and the separation of canonical structure from provenance

`HiddenData<T, U>` supports a useful distinction between data participating in graph identity and auxiliary data that should not affect equivalence.

Its `Eq`, `Ord`, and `Hash` implementations depend only on `data`, while `hidden` is ignored.

An important consequence, however, is that a canonical graph need not provide a canonical ordering of information stored in `hidden`.

### 4.1 Equivalent structures can retain different metadata

Consider two interchangeable vertices:

```
HiddenData::new("component", "source-A")
HiddenData::new("component", "source-B")
```

The canonicalization algorithm compares these vertices using their visible `"component"` values.

Their hidden source identifiers do not influence the graph-isomorphism equivalence relation.

If their connectivity is also symmetric, canonicalization may choose either vertex as the representative at a particular canonical index, without considering the hidden metadata.

Thus, two graphs can have equal canonical forms under Graphica's equality semantics while retaining different auxiliary metadata associated with their canonical vertices.

This distinction becomes relevant when using canonical graphs as persistent, reproducible artifacts.

Canonical structural identity is not the same as canonical serialization of all attached data.

### 4.2 HiddenData is not a redaction mechanism

There is an additional distinction in `Display`.

Graphica formats `HiddenData` using both fields:

```
write!(f, "{} ({})", self.data, self.hidden)
```

Therefore, the hidden metadata can appear in Graphviz DOT, Mermaid, and ordinary graph text output.

This makes `HiddenData` useful for provenance and diagnostics without changing structural identity. It does not make the metadata confidential or exclude it from generated graph descriptions.

When canonical graphs are exported, their auxiliary metadata should be treated as ordinary visible data unless an application explicitly removes or transforms it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Source: [https://docs.rs/graphica/latest/graphica/struct.HiddenData.html](https://docs.rs/graphica/latest/graphica/struct.HiddenData.html?utm_source=chatgpt.com)

## 5. Generation callbacks: exact contracts and advanced composition

Graphica's three generation callbacks provide distinct forms of control over the recursive enumerator.

The original overview described their broad purposes. Their precise contracts reveal additional opportunities and constraints.

| Callback      | Signature or trait               | Receives                                    | Evaluation stage                                      |
| ------------- | -------------------------------- | ------------------------------------------- | ----------------------------------------------------- |
| `filter_fn`   | `Fn(&Graph<N,E>, usize) -> bool` | Working graph and completed-vertex boundary | During recursive enumeration                          |
| `progress_fn` | `FnMut(&Graph<N,E>) -> bool`     | Newly discovered canonical graph            | After completed-graph validation and canonicalization |
| `abort_check` | Numerica `AbortCheck` trait      | No graph argument                           | At connected completed candidates                     |

The `FilterFn` and `ProgressFn` trait objects also require `DynClone + Send + Sync`.

These trait requirements do not imply that Graphica automatically runs generation in parallel.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 5.1 Filtering works on partial graphs

The second argument to `filter_fn` identifies how many initial vertices are completed:

```
Fn(&Graph<N, E>, usize) -> bool
```

If that argument is `completed`, vertices with indices below `completed` will receive no additional edges during that branch of generation.

This distinction is useful when writing monotonic structural constraints.

Suppose a graph may contain at most two vertices of a certain connection type.

Counting such vertices only among completed vertices provides a safe basis for pruning: once a completed vertex has that structure, future extensions cannot alter it.

By contrast, rejecting an incomplete candidate because it does not yet contain a required motif may incorrectly exclude valid completions.

The distinction makes `filter_fn` an extension point for domain-specific constraint propagation rather than merely post-generation filtering.

### 5.2 Composing upper bounds with exact constraints

Graphica's built-in bounds describe maxima, but callbacks can also enforce exact structural requirements on completed connected candidates.

An illustrative filter for exactly one independent cycle is:

```
let settings = GenerationSettings::new()
    .max_vertices(8)
    .max_loops(1)
    .filter_fn(Box::new(
        |g: &Graph<(), &str>, completed: usize| {
            if completed == g.nodes().len() && g.is_connected() {
                g.num_loops() == 1
            } else {
                true
            }
        },
    ));
```

The condition defers the exact cycle-rank test until the current graph is complete and connected.

This matters because Graphica's `num_loops()` computes \\(E-V+1\\), which is the connected-graph cycle-rank formula. It is not a general partial-graph cycle counter.

The same pattern can enforce exact counts of completed vertex kinds, required connectivity properties, or other predicates that become meaningful only once construction has finished.

These constraints do not need to be built into Graphica's enumerator to be applied.

### 5.3 The progress callback receives a canonical graph, not the result map

The `GenerationSettings::progress_fn` documentation contains a misleading description stating that the map of all generated graphs is passed to the callback.

The implementation and public trait establish the actual contract:

```
pub trait ProgressFn<N, E>:
    FnMut(&Graph<N, E>) -> bool
    + DynClone
    + Send
    + Sync
{}
```

The callback receives a reference to an individual newly discovered canonical graph.

It does not receive the complete accumulated result map or the associated automorphism-group size.

This distinction matters for consumers that want to inspect canonical objects as they are discovered.

The callback can perform observations, maintain counters through its mutable closure state, or request cancellation. It is not a native streaming interface that transfers ownership of results out of the generator.

### 5.4 Cancellation retains the triggering result

The implementation invokes `progress_fn` while inserting a previously unseen canonical graph into the result map.

If the callback returns `false`, the insertion still completes, and generation subsequently returns:

```
Err(partial_results)
```

The graph that caused cancellation is therefore included in the partial result map.

`abort_check` has different timing: it is evaluated after a completed candidate passes connectivity testing, before bridge validation and canonicalization.

This means the callback mechanisms cannot be assumed to have equivalent responsiveness or identical effects on the returned results.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs](https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs?utm_source=chatgpt.com)

[https://docs.rs/graphica/latest/graphica/struct.GenerationSettings.html](https://docs.rs/graphica/latest/graphica/struct.GenerationSettings.html?utm_source=chatgpt.com)

## 6. Bridge detection and external-vertex semantics

Graphica's spanning-tree and bridge-analysis methods contain specialized conventions designed around graph generation with external legs.

These conventions are important when reusing the methods for arbitrary graph-analysis workloads.

### 6.1 External status is inferred from valence

`get_spanning_tree()` initializes each `NodeInfo` using:

```
external: n.valence == 1
```

An external vertex therefore means a vertex with valence one, not necessarily a vertex originally supplied through the `external_edges` argument to `Graph::generate()`.

This creates a meaningful distinction between a structurally external vertex and a designated external vertex.

For example, an internally generated degree-one vertex is classified as external by the spanning-tree code.

A graph may also contain specially designated vertices whose degree is not one; those vertices will not be recognized as external by this mechanism.

### 6.2 Bridge counting excludes terminal edges

The `SpanningTree::bridges()` implementation deliberately excludes bridges adjacent to degree-one vertices.

Consequently, `count_bridges()` does not necessarily equal the conventional graph-theoretic number of bridges.

Consider a three-vertex path:

```
A — B — C
```

Both edges are conventional bridges because deleting either edge disconnects the graph.

Under Graphica's non-external bridge convention, neither edge is counted: each is incident to a degree-one endpoint.

The difference is intentional within the library's specialized bridge classification, but consequential for general graph analysis.

A consumer needing all graph-theoretic bridges must not treat `count_bridges()` as an unrestricted bridge counter.

### 6.3 Zero-flow bridge detection has a root precondition

`SpanningTree::has_zero_flow_bridges()` has a stricter precondition than its name suggests.

If any degree-one vertex exists, the implementation asserts:

```
assert!(self.nodes[0].external);
```

Thus the vertex at index zero must itself have valence one.

This is more specific than simply having a spanning tree rooted at some degree-one vertex elsewhere in the graph.

For example, constructing a spanning tree of a star graph with its central vertex at index zero and calling `has_zero_flow_bridges()` can trigger this assertion.

When no vertices have valence one, the method instead treats any qualifying non-external bridge as a zero-flow bridge.

The consequence is that `has_zero_flow_bridges()` should be understood as a specialized structural predicate with rooting assumptions, not as a universally applicable graph-analysis routine.

Graphica's generator normally inserts supplied external vertices first, which explains why its standard externally anchored enumeration is compatible with the assumption. Closed-graph generation and manually constructed graphs require additional attention.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 6.4 Connectivity and spanning-tree edge cases

The standalone `Graph::is_connected()` method returns `true` for an empty graph.

By contrast, `get_spanning_tree(start_vertex)` requires a valid starting vertex and is not a way to obtain an empty spanning tree from an empty graph.

On disconnected graphs, the spanning tree covers only the component reachable from the chosen root.

For ordinary bridge and chain analysis, connectivity should therefore be established before treating the spanning-tree result as a decomposition of the entire graph.

These qualifications do not restrict the separate canonicalization and isomorphism methods, which can operate on disconnected graphs and graphs with isolated vertices.

## 7. Direction changes and endpoint normalization

A further implementation-level distinction concerns `Graph::set_directed()`.

When an undirected edge is initially added, Graphica normalizes its endpoint order so that the lower vertex index comes first.

However, `set_directed()` only changes the edge's Boolean direction flag:

```
pub fn set_directed(
    &mut self,
    index: usize,
    directed: bool,
) -> bool {
    std::mem::replace(
        &mut self.edges[index].directed,
        directed,
    )
}
```

It does not reorder the stored endpoints.

### 7.1 Directed-to-undirected conversion

Consider an edge originally stored as:

```
vertices = (5, 2)
directed = true
```

After:

```
graph.set_directed(edge_index, false);
```

the edge remains stored with endpoints `(5, 2)`.

An equivalent undirected edge constructed directly through `add_edge(5, 2, false, ...)` would instead be stored as `(2, 5)`.

Thus, changing the direction flag can create two undirected edges that are semantically equivalent but have different stored endpoint orderings.

### 7.2 Consequence for edge symmetry

`get_edge_automorphism_group_size()` groups edges using their stored endpoint pairs, direction flags, and payloads.

If two otherwise identical undirected edges have stored endpoints `(2, 5)` and `(5, 2)`, they will fall into different endpoint groups.

This can prevent the method from recognizing their expected interchange symmetry.

The implication follows directly from the implementation; it is not a published benchmark or a separately executed regression test.

`canonize_edges()` sorts existing edge records but does not renormalize their endpoints.

Full `canonize()` reconstructs the canonical graph through `add_edge()`, which does normalize undirected endpoints.

This is therefore an important distinction between edge sorting and full canonicalization when graph directionality is modified after construction.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



[https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs](https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs?utm_source=chatgpt.com)

## 8. Additional public functionality and smaller distinctions

A few public capabilities were omitted from the original reference. They are relatively small individually but useful to know when working directly with Graphica's graph representation.

| API or type                                           | Additional capability                                                                                                                       |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `Edge::is_self_loop()`                                | Tests whether an edge's stored endpoints are identical                                                                                      |
| `Edge::is_neighbor(vertex)`                           | Tests whether a particular edge is incident to a given vertex                                                                               |
| `Empty`                                               | Provides a zero-information payload implementing `Default`, equality, ordering, hashing, cloning, debugging, and empty `Display` formatting |
| `Graph` implementing `Display`                        | Produces a line-oriented text representation of vertex payloads and labeled directed or undirected edges                                    |
| `Graph::canonize()` on disconnected graphs            | Supports canonicalization of disconnected graphs, including isolated vertices                                                               |
| `Graph::canonize()` on an empty graph                 | Returns an empty canonical graph, empty mappings, and automorphism-group size one                                                           |
| `GenerationSettings` implementing `Clone` and `Debug` | Allows duplicating suitable settings and inspecting their configuration state                                                               |

### 8.1 Directed and undirected self-loop construction

The half-edge generator handles directed and undirected self-loops differently.

A directed self-loop requires complementary incoming and outgoing half-edges carrying equal edge data:

```
vec![
    HalfEdge::incoming("x"),
    HalfEdge::outgoing("x"),
]
```

An undirected self-loop consumes two undirected incidences of the same type:

```
vec![
    HalfEdge::undirected("x"),
    HalfEdge::undirected("x"),
]
```

These signatures permit the corresponding attachment combinations but do not themselves require that the two incidences be connected into a self-loop.

Actual self-loop formation additionally depends on `allow_self_loops(true)` and the rest of the generation constraints.

### 8.2 Display formatting as a lightweight inspection mechanism

In addition to DOT and Mermaid export, `Graph` implements the ordinary Rust `Display` trait.

A graph with printable vertex and edge payloads can therefore be formatted directly:

```
println!("{graph}");
```

The output lists vertices by index and then edges with their data and direction notation.

This is a lightweight inspection representation rather than a fully specified interchange or serialization format.

The `Empty` type is particularly convenient when a graph's topology matters but a particular vertex or edge category does not need a semantic label.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 8.3 Canonical graphs and deterministic iteration

Graph generation returns an `ahash::HashMap` keyed by canonical graphs.

Although the canonicalization result supports structural comparison, the result map does not establish a deterministic iteration order.

Therefore, the order in which generated graphs are encountered through `.iter()` or `.keys()` should not be treated as a stable enumeration sequence.

A stable ordering can instead be established through an explicit sorting procedure based on the canonical graph values or another application-defined representation.

This is particularly relevant to reproducible output sets, persisted catalogs, and comparisons across independent enumeration runs.

## 9. Interpretation of the additional capabilities

The additions clarify three aspects of Graphica's architecture.

First, its generator is more expressive than an externally anchored graph generator. It supports closed topologies, repeated vertex signatures, typed self-loop construction, and custom constraints on incomplete candidates. The important tradeoff is that some restrictions—especially vertex-type multiplicities and exact structural requirements—must be expressed through custom pruning rather than dedicated settings.

Second, structural equivalence and object identity are deliberately separate. Canonical vertex maps support reconstruction of graph correspondences, but edge identity and hidden metadata require their own treatment. This distinction is essential when canonical graphs are used as durable keys or when results must be reconciled with external object records.

Third, several graph-analysis methods are specialized for Graphica's enumeration model. Non-external bridge counting, zero-flow detection, and fixed-vertex edge symmetry have semantics that differ from superficially similar general graph algorithms. Their usefulness is substantial within the intended model, but their preconditions must be preserved when applied independently.

## Source references

The principal sources supporting this addendum are:

Graphica 3.0.1 API

[https://docs.rs/graphica/latest/graphica/](https://docs.rs/graphica/latest/graphica/?utm_source=chatgpt.com)

Upstream implementation: graph generation, callbacks, canonicalization, and bridge analysis

[https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs](https://github.com/symbolica-dev/graphica/blob/main/src/lib.rs?utm_source=chatgpt.com)

Graphica 3.0.1 directed self-loop symmetry correction

[https://github.com/symbolica-dev/graphica/commit/6156fc1b35f7c9a46e11c62b54508715a2559265](https://github.com/symbolica-dev/graphica/commit/6156fc1b35f7c9a46e11c62b54508715a2559265?utm_source=chatgpt.com)

CanonicalForm API

[https://docs.rs/graphica/latest/graphica/struct.CanonicalForm.html](https://docs.rs/graphica/latest/graphica/struct.CanonicalForm.html?utm_source=chatgpt.com)

GenerationSettings API

[https://docs.rs/graphica/latest/graphica/struct.GenerationSettings.html](https://docs.rs/graphica/latest/graphica/struct.GenerationSettings.html?utm_source=chatgpt.com)

SpanningTree API

[https://docs.rs/graphica/latest/graphica/struct.SpanningTree.html](https://docs.rs/graphica/latest/graphica/struct.SpanningTree.html?utm_source=chatgpt.com)

HalfEdge API

[https://docs.rs/graphica/latest/graphica/struct.HalfEdge.html](https://docs.rs/graphica/latest/graphica/struct.HalfEdge.html?utm_source=chatgpt.com)

The addendum is based on upstream source inspection rather than independent compilation or benchmarking. No additional major released Graphica functional subsystem was identified beyond the families covered in the original reference; the improvements here are the additional capabilities, precise semantics, and consequential constraints within those families.