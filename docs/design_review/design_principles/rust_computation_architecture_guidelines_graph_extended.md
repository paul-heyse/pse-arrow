# Rust Model-Driven Computation: Architecture Guidelines

> **Superseded 2026-09-24.** General requirements moved to the [core design principles](core/design-principles.md); library-specific material belongs to the library skills. The [pse-arrow binding](binding/pse-arrow.md) maps each RCA section. Retained so reviews citing `RCA §n` stay readable.

## Purpose and interpretation

Preserve a typed, relational system of record while avoiding repeated dependency reconstruction, recursive expansion, and unnecessary recomputation. Optimize for semantic correctness, controlled intermediate growth, reuse, and low bespoke orchestration, not a single physical representation.

**MUST** denotes a correctness requirement of this design. **DEFAULT** denotes a preferred choice that may be overridden for a documented workload or semantic reason. **MAY** denotes an optional technique, not required scope. Verify exact APIs against the repository's locked dependency versions. Select a mutually compatible Arrow/DataFusion/delta-rs dependency set rather than independently selecting each latest release. This is an architectural contract, not version-specific implementation code.

**Revision scope:** Integrates the broader Rust graph-analysis backend catalog into the original computation architecture. Salsa remains the optional incremental boundary for semantic compilation and selected reusable analyses. Library capabilities below are externally verified additions; backend defaults and integration rules are architectural recommendations, not benchmark findings or a requirement to adopt every listed crate.

## 1. Separate algorithm selection from incremental reuse

Assign each operation both a **computation backend** and, independently, a **reuse/invalidation boundary**.

| Layer | Default responsibility | Selection criterion |
|---|---|---|
| Arrow | Columnar interchange and direct array/batch kernels | Arrays and the physical operation are already known: gather, filter, cast, construct batches. |
| DataFusion | Relational planning and batch execution | The operation is naturally specified as scans, joins, filtering, grouping, sorting, or composable expressions. |
| Graph analytics: Petgraph plus selected backends | Explicit topology, structural algorithms, and network analytics | Reachability, strongly connected components (SCCs), ordering, dominators, matching, centrality, community structure, or temporal relationships are the actual question; select the backend under Section 5. |
| Salsa | Demand-driven semantic computation, dependency tracking, memoization, invalidation | A stable keyed result has meaningful reuse within a build or across revisions. |
| Specialized Rust kernel/solver | Domain-specific computation | Neither relational nor graph execution expresses the required algorithm appropriately. |
| Delta Lake / delta-rs | Authoritative versioned model data and selected durable artifacts | State must survive process lifetime and support reproducible reads. |

Arrow kernels, DataFusion execution, graph algorithms, and Salsa tracked computations provide distinct mechanisms; using one does not exclude the others.[^arrow][^datafusion][^petgraph][^rustworkx][^salsa-overview]

**DEFAULT:** Salsa coordinates semantic compilation, not every runtime calculation. A tracked query may call a selected graph backend, Arrow, or ordinary Rust. Do not implement a second invalidation graph merely to duplicate dependencies Salsa already records. Graph representation, algorithm implementation, and incremental reuse are separate choices.

**Conditionality:** Omit a Salsa boundary when a computation is cheap, has little reuse, or tracking would destroy efficient batching. Use it when avoided work plausibly exceeds validation, equality comparison, metadata, retention, and integration costs. No universal row-count threshold applies.

## 2. Preserve model semantics across representations

**MUST:** Maintain one authoritative definition of entities, relationships, types, bindings, and operation semantics. Derived graphs, indices, intermediate representations (IRs), and cached artifacts must correspond to identified source versions; they are not independently edited sources of truth.

**DEFAULT:** Represent composition using stable IDs and direct references. Use typed Rust enums/newtypes, arenas, maps, and adjacency structures internally where appropriate. Arrow is a boundary/data-plane representation, not a requirement for every compiler lookup. Validate semantic invariants at representation boundaries instead of reconstructing them through joins repeatedly.

Distinguish these identities:

| Identity | Meaning |
|---|---|
| Definition | Reusable component or operation definition. |
| Specialization | Definition instantiated for structural configuration, types/shapes, and target semantics. |
| Instance | Scoped occurrence with its own bindings and potentially independent runtime state. |
| Artifact | A particular compiled or computed result with explicit semantic dependencies. |

**MUST:** Share only semantically equivalent computations. Equal definitions do not imply equal instance state. Expression identity must preserve opcode, result type, ordered operands, literals, and relevant execution semantics. Preserve meaningful edge multiplicity, ports, relationship kinds, and direction. Exclude incidental provenance from computational identity unless it affects meaning.

**DEFAULT:** Let model data select and connect a finite vocabulary of primitives, such as `RelationalPlan`, `KernelCall`, `ComponentCall`, `FixedPointBlock`, and `SolverBlock`. Adding instances should generally require model data, not new orchestration code. New computational semantics may require a new reusable primitive. A typed `GraphAnalysis` descriptor MAY specialize `KernelCall`; additional graph libraries do not require separate orchestration frameworks.

## 3. Make Salsa dependencies complete and identities stable

**MUST: dependency completeness.** Every result-affecting value must be represented by the query key or an observed Salsa dependency. A handle selects a memo; its mutable contents require tracked field reads. A query must not silently observe mutable files, “latest” tables, session registries, clocks, or unseeded randomness. Immutable external artifacts are acceptable when their identity pins their contents and relevant semantics.[^salsa-overview][^salsa-core]

**MUST: graph dependency completeness.** Observe the relevant projection definition, node/edge membership, weights or labels actually used, algorithm configuration, explicit randomness, and warm-start artifacts. An unweighted structural result need not depend on unused weight fields; a weighted or personalized result does. A shared graph/view handle is not a version guarantee: require immutable content or a proven pinned snapshot, not mutable backend state hidden behind the same identity.

**MUST: input ownership.** Apply external input changes outside tracked computation. Do not place externally required side effects inside memoized bodies: execution may be skipped or repeated. Publish writes through an explicit execution/publication boundary.[^salsa-overview]

**DEFAULT: input granularity.** Separate semantic definitions, topology, structural parameters, runtime values, and descriptive metadata. Keep large homogeneous datasets as immutable Arrow artifacts with suitably scoped version references rather than one Salsa input per cell. Reading an Arrow payload through one Salsa field does not establish dependencies on individual rows.[^salsa-input]

**DEFAULT: ingestion adapter.** Maintain `DomainId -> existing Salsa input handle`; compare values and update changed fields only. Track collection/index membership, including additions and removals, so successful lookups, failed lookups, and enumeration are invalidated correctly. Avoid making every lookup depend on one global index when scoped indices suffice. Current Salsa input constructors create distinct identities and setters do not suppress equal-value updates.[^salsa-core]

**MUST: derived-entity ownership.** Do not assume equal-looking tracked structs created by different queries are globally canonical. Use a canonical producer query for shared derived entities; use interning when immutable structural equality should define shared identity. Keep persistent domain IDs distinct from Salsa handles and Petgraph indices. Respect database lifetimes and reacquire derived handles after revisions rather than bypassing lifetime checks.[^salsa-tracked][^salsa-core][^petgraph-graph]

## 4. Choose semantic query boundaries, not arbitrary code boundaries

**DEFAULT:** Track independently reusable semantic results: resolved interfaces, validated components, structural specializations, compiled expressions/components, and graph/block analyses. Leave cheap arithmetic and batch inner loops untracked. A top-level tracked query is useful when it delegates to appropriately scoped tracked subqueries; it does not incrementalize ordinary helpers automatically.[^salsa-tracked]

Separate **interface from implementation**, **topology from annotations**, and **compiled structure from runtime values**. Consumers should read the narrowest result that captures their true dependency. Do not classify a parameter as nonstructural when it changes topology, types, shapes, code selection, or specialization.

**MUST: sound equality.** Results considered equal must be substitutable for downstream computations. Prefer stable semantic values, canonical summaries, or immutable artifact identities. Do not use fresh allocation identity as semantic equality or an ad hoc floating-point tolerance to hide meaningful changes. Define ordering, null, NaN, numerical, and reproducibility semantics where relevant. Salsa's unchanged-output reuse depends on result equality.[^salsa-algorithm]

**DEFAULT:** Canonicalize unordered structural outputs so incidental traversal order does not cause unnecessary changes. For large opaque outputs, expose a small stable semantic descriptor and separately owned payload. Do not hash or deeply compare whole datasets on every lookup; establish artifact identity at appropriate publication boundaries.

**Conditionality:** Coarsen queries when batching, metadata costs, or highly correlated changes dominate. Refine them when expensive unaffected work is repeatedly recomputed. Whole-region graph analysis is an acceptable initial boundary. Salsa can reuse or rerun that analysis; it does not automatically make the underlying graph algorithm dynamically incremental.[^salsa-algorithm]

**DEFAULT: graph query granularity.** Track a reusable projection and an analysis result at a semantically complete graph scope; separate them when multiple algorithms reuse the same adjacency. Keep PageRank iterations, community local moves, and random-walk steps inside the kernel, not individual Salsa queries. A global analysis can change after a local edge edit; restrict recomputation only when the algorithm and boundary conditions justify it.

**MUST: reproducibility and result identity.** For stochastic analyses, record the seed and all result-affecting settings. A seed alone is not evidence of determinism: validate ordering, tie-breaking, parallel reductions, and initialization. Execute uncontrolled nondeterministic work outside tracked computations and ingest its immutable result. Canonicalize partition labels by membership when label names are immaterial; do not treat community numbers as persistent identities. Cross-revision community matching or embedding-space alignment is a separate derived computation. Include a warm start as an explicit dependency when it can change the returned result.

## 5. Keep relational pipelines fused; use graphs for topology

**DEFAULT: DataFusion.** Keep substantial compatible relational transformations in composable subplans. Resolve explicit facts and bounded relationship patterns relationally when this is straightforward. Do not create one query execution per entity, property, or traversal step.

**DEFAULT: Arrow.** Use existing kernels for local array operations at established boundaries. Do not rebuild joins, grouping, or a query optimizer in handwritten Arrow loops. Do not break an otherwise useful DataFusion plan merely to invoke an available Arrow kernel.[^arrow]

**DEFAULT: graph analytics.** Build a narrow projection of direct relationships when topology or network structure is required. Store lightweight IDs and essential edge semantics; retain bulk attributes separately. Distinguish containment, computation dependencies, and expression operands. For computation edges, use `prerequisite -> dependent`; prerequisite discovery follows incoming edges.

**MUST:** A traversal projection must retain every intermediate node/edge needed by the requested semantics. An output filter is not automatically a valid input filter. Likewise, an SCC analysis region must include possible cross-boundary cycles or use a decomposition that proves them impossible. Include required global constraints in the root/scope definition, not only obvious output ancestors.

**DEFAULT:** Rebuild/cache a whole relevant graph region before implementing dynamic graph maintenance. Reuse graph projections where beneficial, but account for construction, conversion, and memory costs. For a small one-off bounded relationship query, an existing relational plan may be simpler than constructing a graph.

### 5.1. Specify the graph before choosing the algorithm

**MUST:** Define the projection and the analysis as typed configuration. The following is a semantic checklist, not a prescribed serialization format or giant Salsa key:

```text
GraphProjectionSpec:
  relevant source versions; domain scope; node/relationship kinds
  direction; context and boundary policy; isolated-node policy
  parallel-edge/self-loop policy; edge aggregation and weight semantics
  time model/window/event ordering, if any
  completeness; sampling/sparsification policy, if any

GraphAnalysisSpec:
  actual algorithm + backend/version/features
  objective/normalization/resolution; roots/terminals/personalization
  seed; initialization/warm-start reference; convergence/iteration rules
  exact/approximate/heuristic contract; resource/failure policy

GraphAnalysisResult:
  stable-domain-ID rows + projection/analysis identity
  values/membership/witnesses + status, scope, and provenance
```

**MUST:** Preserve the semantics required by the selected algorithm. Do not silently symmetrize directed relationships, collapse meaningful parallel edges, discard isolates, or convert a hyperedge into pairwise links. Use incidence/operation nodes where higher-arity meaning requires them. Define whether a weight is distance, affinity, capacity, probability, or confidence; transformations between them must be explicit. Validate the algorithm's actual restrictions on direction, weights, loops, and multiplicity.

**DEFAULT:** Use DataFusion to select and normalize projection facts; construct the required adjacency once; return graph outputs as typed Arrow relations for subsequent joins, filtering, and publication. Direct degree counts or bounded motifs may remain relational. Sparse numerical kernels may be preferable for an already matrix-shaped problem; do not force a graph crate solely because the input describes relationships.

### 5.2. Select a backend by required capability, not catalog breadth

The version families below are candidates, not a mutually validated dependency bundle. Inspected examples: `rustworkx-core` 0.18.1, `leiden-rs` 0.8.1, `graphops` 0.5.1, `graphina` 0.4.0-alpha.6, `rust-igraph` 0.7.0, and `raphtory` 0.17.0.

| Backend | Use when | Integration / qualification |
|---|---|---|
| **Petgraph** | General topology, traversal, SCCs, topological order, dominators, paths. | Default structural representation when its graph model fits. Do not build another representation merely to call a compatible generic algorithm.[^petgraph] |
| **`rustworkx-core` 0.18.x** | Additional centrality, connectivity/cut, DAG, coloring, matching, and path routines. | Default extension to Petgraph when the required routine exists. The Rust core exposes generic algorithms over supported graph traits; verify the Rust API rather than assuming Python-wrapper feature parity.[^rustworkx][^rustworkx-core-source] |
| **`leiden-rs` 0.8.x** | Leiden community analysis, quality objectives, resolution exploration, and partition evaluation. | Specialist candidate for community work. Select the objective explicitly. Uses its own CSR-based `GraphData`, with optional adapters; graph conversion remains an explicit cost and semantic boundary.[^leiden][^leiden-data] |
| **`graphops` 0.5.x** | Louvain, PageRank/PPR, walks, neighborhood similarity, graph kernels, or structural embedding utilities. | Use selected operators through compatible adapter traits. Check the specific module's output contract; see algorithm-name and Node2Vec qualifications below.[^graphops][^graphops-traits] |
| **`graphina` 0.4.x-alpha** | A needed gap in approximation, link prediction, metrics, or other available analytics. | Conditional gap-filling dependency. Pin an exact prerelease and validate the selected routine and enabled features; do not default to the whole catalog.[^graphina] |
| **`rust-igraph` 0.7.x** | A needed broad-coverage algorithm or an additional cross-check implementation. | Describes itself as a pure-Rust port, not C FFI. Validate the actual routine; upstream igraph's reputation is not proof of port equivalence. Declares GPL-2.0-or-later; require project license approval before adoption.[^rust-igraph] |
| **`raphtory` 0.17.x** | Time-indexed graph views and recurring temporal analytics materially simplify the workload. | Introduces a temporal graph data model, not just another Petgraph algorithm. Require explicit event/persistence semantics and project license approval; declares GPL-3.0.[^raphtory][^raphtory-persistent] |

**DEFAULT:** Prefer an already-adopted compatible implementation; add a specialist only for a concrete capability or workload benefit. Neither a broad API nor an alpha label establishes a particular routine's correctness or performance. Keep backend-specific types behind small typed adapters, without constructing a universal graph framework.

**MUST: dependency/type compatibility.** `rustworkx-core` 0.18.1 targets Petgraph 0.8 and re-exports it; `graphops` 0.5.1's optional Petgraph adapter targets 0.6. These types are not interchangeable. Prefer a coherent Petgraph dependency for the core and an application-owned adapter implementing `graphops` traits where suitable; explicit conversion or a maintained compatibility patch is conditional. Never bridge versions by layout assumptions or casts. Inspect `Cargo.lock`, enabled features, MSRV, and `cargo tree -d`; do not infer compatibility from crate names.[^rustworkx-manifest][^rustworkx-core-source][^graphops-manifest]

**MUST: index mapping.** Keep domain IDs separate from graph-local indices. Maintain explicit mappings, including any dense remapping required by a backend; preserve supported holes or compact them deliberately. Never zip a result vector with an unrelated node-iteration order. Check result indexing and missing-node conventions per function. An adapter must not allocate an entire graph or a neighbor vector repeatedly inside a hot loop merely to satisfy an interface.[^graphops-traits][^petgraph-graph]

**MUST: algorithm identity.** In `graphops` 0.5.1, the legacy `leiden*` entry points implement connectivity-refined Louvain, not Leiden's constrained-merge refinement. Record/use the actual algorithm, and choose a verified Leiden implementation when Leiden is required. Its `node2vec` module generates biased walks; that module alone is not a complete learned-embedding trainer.[^graphops-leiden][^graphops-node2vec]

### 5.3. Distinguish structural facts from analytical heuristics

**MUST:** Keep mathematical exactness, input completeness, and domain interpretation separate. SCCs or shortest paths are facts about a specified projection, not automatically about an incomplete underlying system. Community detection, ranking, similarity, embeddings, and predicted links are analytical results, not proof of computational equivalence, dependency absence, causal influence, or valid execution boundaries.

**DEFAULT:** Use community/centrality/similarity outputs for diagnostics, navigation, prioritization, or candidate generation. They MAY guide locality or execution grouping if all dependencies, constraints, and cross-group edges remain represented and correctness is independently validated. Community partitions are not SCCs and cannot replace cycle analysis. A matching, cut, or isomorphism result supports compilation only when the domain reduction and preserved semantics are explicit.

**MUST:** Do not promote inferred edges into authoritative dependency facts without a separate validated rule. Keep structural embeddings distinct from external semantic embeddings; record how any combined affinity graph is constructed. Bound candidate generation rather than materializing every pair by default. Approximation/sparsification changes the analysis contract and must not silently remove edges needed by exact execution semantics.

### 5.4. Separate storage versions from temporal graph semantics

**DEFAULT:** Use Delta snapshots plus DataFusion comparisons for versioned model state, change sets, and ordinary time-window aggregations. Add Raphtory when repeated temporal graph queries justify its additional model and representation; merely having timestamps is insufficient.

**MUST:** Distinguish domain/event time, storage transaction version, and Salsa revision. Specify insertion, deletion, re-addition, interval endpoints, and simultaneous-event ordering. Raphtory's persistent graph keeps an edge active until deletion, and a window can include edges active at different instants within it. Ordinary traversal over that window's combined edges does not prove simultaneous connectivity or a time-respecting path; select/implement the required temporal semantics explicitly.[^raphtory-persistent]

## 6. Give recursion explicit semantics

| Recursive behavior | Default implementation |
|---|---|
| Finite acyclic component composition | Demand-driven keyed compilation; share prerequisite results. |
| Reachability or dependency closure | Adjacency traversal with a visited set; return only required structure. |
| Structural cycles | SCC decomposition, condensation, then explicit block semantics; communities are not a substitute. |
| Monotone recursive inference | Worklist/delta propagation or justified fixed-point evaluation. Salsa cycle recovery is optional. |
| Iterative graph metrics or community optimization | Backend-controlled iterations under an explicit convergence/heuristic contract; not automatically a Salsa recursion cycle. |
| Coupled equations, optimization, temporal feedback | Solver/runtime block with explicit convergence or state-transition semantics. |
| Unbounded structural specialization | Symbolic/runtime representation, justified finite bound, or explicit diagnostic. |

**MUST:** Distinguish cycle detection from cycle resolution. SCC decomposition does not solve numerical equations. Salsa fixed-point recovery requires the documented monotonicity/convergence conditions; it is not a general nonlinear solver.[^petgraph][^salsa-cycles]

**DEFAULT:** Preserve shared nodes and direct edges. Avoid all-path expansion, all-pairs closure, or a full ancestor set cached for every node unless those outputs are actually required. Memoization cannot remove an intrinsically large output.

For explicit recursive graph construction, register semantic node identity before descending into its prerequisites, then analyze cycles. This is a graph-construction technique, not permission to return partially initialized Salsa query results. For recursive inference, revisit work when facts change; a once-only visited flag is insufficient.

**MUST:** Define termination and resource-failure behavior. Do not silently truncate required dependencies, facts, or specialization. Preserve sharing through backend lowering rather than expanding a compact IR into duplicated plans.

## 7. Separate compilation, execution, and publication

**DEFAULT integration:**

```text
Pinned model release
  -> DataFusion/Arrow preparation
  -> semantic diff applied to Salsa inputs
  -> tracked resolve/validate/analyze/compile
       -> selected graph backends, Arrow, or Rust kernels as appropriate
  -> owned executable specification + immutable input references
  -> DataFusion / batch kernel / solver execution
  -> immutable result artifacts and explicit publication
  -> optional artifact inputs for subsequent semantic analysis
```

The staged boundary is a default, not a prohibition on tighter integration. A pure in-process kernel may execute inside a tracked query when lifetime, cancellation, determinism, and cost are manageable. Prefer external orchestration for asynchronous I/O, long-running bulk execution, and side effects.

**MUST:** Distinguish compiled-plan reuse from executed-result reuse. DataFusion plans are lazy; retaining a plan is not materializing its output.[^datafusion]

```text
Compilation validity:
  relevant definitions + structural bindings + type/shape semantics
  + compiler/kernel versions + target/options

Execution-result validity:
  compiled artifact + relevant input artifact versions
  + runtime values + result-affecting execution/solver options
```

These describe dependency requirements, not mandatory giant query-key tuples. Let Salsa observe mutable semantic dependencies through tracked reads. Explicit persistent artifact keys must encode the corresponding validity contract. Graph results that affect executable structure are compilation dependencies; diagnostic-only results need not invalidate compilation. Include relevant projection/analysis semantics and result-affecting implementation changes, not every unrelated backend version.

**DEFAULT:** Introduce materialization where repeated execution is expensive enough to justify retained bytes, serialization, and lost optimization opportunities. Do not materialize every boundary or assume repeated subplans share executed results automatically. Start with existing Arrow/`MemTable` integration; custom DataFusion operators/providers require a concrete benefit and correct optimizer/execution contracts.[^datafusion][^datafusion-provider][^datafusion-operators]

**MUST: graph operator boundaries.** A graph result exposed as a table does not make its generating algorithm row-local or arbitrarily partitionable. Preserve required input columns, global scope, ordering/distribution requirements, and memory accounting. Only push output predicates/limits into a graph computation when that transformation preserves its meaning; computing global centrality after filtering the graph is generally a different query. Do not run a whole-graph computation independently per DataFusion batch or partition without a valid decomposition.

**Conditionality:** Salsa `specify` is optional, not a generic external cache-injection API. Its creating-query ownership and eligible-key restrictions must hold. Do not make it a prerequisite for batch integration.[^salsa-tracked]

## 8. Bound persistence, memory, and invalidation scope

**MUST:** A build consumes a consistent identified release. For multi-table Delta models, publish a manifest of exact table versions after constituent writes and validation succeed; do not equate independently read “latest” versions with one atomic model release. Delta's documented transactions are table-scoped.[^delta]

**DEFAULT:** Record the complete manifest for provenance, but use relevant dependency subsets for fine-grained reuse. Do not put a global release ID in every semantic key merely for traceability. Define separate handling for metadata-only updates when numerical or structural reuse should survive them.

**MUST:** Define ownership and retention for payloads, query keys, graph projections, and compiled artifacts. Honor cancellation and reject or segregate stale asynchronous results before publishing them as current. Keep revision changes behind an application-level update boundary so no consumer observes a partially applied model update.

**DEFAULT:** Keep bulk buffers outside fine-grained memos unless reuse justifies retention. Bound scenario/key proliferation as well as cached result bytes. Salsa LRU eviction does not by itself establish a bound on all memo metadata.[^salsa-tuning]

**DEFAULT: graph resource budget.** Account for source batches, ID maps, every retained adjacency/CSR conversion, algorithm workspaces, output size, and cached artifacts together. Streaming Arrow input does not make an in-memory graph algorithm streaming or out-of-core. Coordinate DataFusion, graph-backend, Rayon, and solver concurrency rather than independently maximizing every pool. Budget dense matrices, all-pairs paths/similarity, walk corpora, clique/path enumeration, and community parameter sweeps before launch. Prefer sparse, root-scoped, sampled, or approximate variants only where their semantics are acceptable; return explicit incompleteness/nonconvergence or fail rather than silently presenting a truncated exact result.

## 9. Implementation acceptance contract

For each nontrivial stage, record this compact contract in code/design documentation:

```text
Semantic output and equality:
Backend/version/features and physical representation:
Graph projection, ID mapping, and algorithm configuration, if applicable:
Semantic key / tracked query boundary, if any:
Observed dependencies, including membership and absence:
Structural versus runtime inputs:
Cycle/termination and temporal semantics:
Exactness, input completeness, heuristic interpretation, and reproducibility:
Expected intermediate/output cardinality, conversion cost, and algorithmic cost:
Ownership, retention, cancellation, and publication rules:
Reason for any departure from defaults:
```

Validate one selected architecture; do not build competing systems merely to choose a boundary. Compare incremental results against a clean recomputation for representative edits, including additions, deletions, failed-lookup recovery, and topology changes.

Required behavioral checks: shared dependencies are reused while their memos remain resident and valid; nonstructural edits preserve structural artifacts; unchanged interfaces preserve interface-only consumers; required global constraints remain included; cycles receive the intended semantics; resource limits fail explicitly.

Include chains, reconvergent diamonds, shared subcomponents, cross-region edges, cyclic blocks, and repeated scenarios. Instrument preparation/planning, graph construction, kernel execution, query re-execution, intermediate cardinality, retained memory, and publication separately. Optimize the measured boundary, not an assumed universal engine bottleneck.

**Graph acceptance additions:** Test directed asymmetry, parallel edges, self-loops, isolates, index holes/remapping, weight-only edits, projection/configuration changes, cold versus warm initialization, and repeated seeded runs. For temporal analyses, include deletion/re-addition and edges that never coexist. Compare alternative implementations only under matched graph and algorithm semantics; compare heuristic partitions using appropriate invariants/objectives rather than raw label equality. Validate a selected routine with small independent fixtures or differential checks, not a second full architecture.

**Governing principle:** Keep semantics model-defined, choose physical algorithms by computational structure, and place incremental boundaries where independently reusable meaning exists.

---

## Primary references

The original references below are retained from the supplied computation guidelines (original review: September 22, 2026). Graph-backend additions were checked against primary documentation and repository source on September 22, 2026. Context7 provided supporting architectural documentation; its available rustworkx-core entry was 0.17.1, so 0.18-specific claims were checked against the 0.18.1 source/catalog instead. Live pages can evolve; version labels identify what was inspected, not a guarantee of compatibility across a version family. No crate compilation, integration testing, or comparative benchmarks were performed for this revision. Implementation must follow the selected `Cargo.lock` and validate each adopted routine.

[^arrow]: Apache Arrow Rust computation kernels: https://arrow.apache.org/rust/arrow/compute/index.html
[^datafusion]: Apache DataFusion DataFrame API, lazy execution, streaming, and caching: https://datafusion.apache.org/library-user-guide/using-the-dataframe-api.html
[^petgraph]: Petgraph algorithm catalog: https://docs.rs/petgraph/latest/petgraph/algo/index.html
[^salsa-overview]: Salsa overview and deterministic-input model: https://salsa-rs.github.io/salsa/overview.html
[^salsa-input]: Salsa input macro and field-level dependencies: https://docs.rs/salsa/latest/salsa/attr.input.html
[^salsa-tracked]: Salsa tracked structs/functions and API restrictions: https://docs.rs/salsa/latest/salsa/attr.tracked.html
[^salsa-core]: Salsa identity, ownership, lifecycle, and memo semantics: https://docs.rs/salsa/latest/salsa/
[^salsa-algorithm]: Salsa incremental validation and backdating: https://salsa-rs.github.io/salsa/reference/algorithm.html
[^salsa-cycles]: Salsa cycle-handling requirements: https://salsa-rs.github.io/salsa/cycles.html
[^salsa-tuning]: Salsa cache tuning and retained dependency information: https://salsa-rs.github.io/salsa/tuning.html
[^petgraph-graph]: Petgraph Graph representation and index stability: https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
[^datafusion-provider]: DataFusion custom table providers and MemTable integration: https://datafusion.apache.org/library-user-guide/custom-table-providers.html
[^datafusion-operators]: DataFusion custom logical/physical operators: https://datafusion.apache.org/library-user-guide/extending-operators.html
[^delta]: Delta Lake FAQ, transaction scope: https://docs.delta.io/delta-faq/


### Added graph-backend references

[^rustworkx]: rustworkx-core Rust algorithm catalog (rendered 0.18.1 at review): https://docs.rs/rustworkx-core/latest/rustworkx_core/
[^rustworkx-core-source]: rustworkx-core 0.18.1 module declarations and Petgraph re-export: https://github.com/Qiskit/rustworkx/blob/0.18.1/rustworkx-core/src/lib.rs
[^rustworkx-manifest]: rustworkx 0.18.1 workspace manifest, including Petgraph dependency: https://github.com/Qiskit/rustworkx/blob/0.18.1/Cargo.toml
[^leiden]: leiden-rs Rust API, quality objectives, partition metrics, and resolution routines (rendered 0.8.1 at review): https://docs.rs/leiden-rs/latest/leiden_rs/
[^leiden-data]: leiden-rs GraphData CSR representation (rendered 0.8.1 at review): https://docs.rs/leiden-rs/latest/leiden_rs/graph/data/struct.GraphData.html
[^graphops]: graphops 0.5.1 Rust operator catalog: https://docs.rs/graphops/0.5.1/graphops/
[^graphops-traits]: graphops 0.5.1 adapter traits: https://docs.rs/graphops/0.5.1/graphops/graph/index.html
[^graphops-manifest]: graphops 0.5.1 published manifest, including optional Petgraph 0.6 dependency: https://docs.rs/crate/graphops/0.5.1/source/Cargo.toml
[^graphops-leiden]: graphops 0.5.1 connected-Louvain versus Leiden qualification: https://docs.rs/graphops/0.5.1/graphops/leiden/index.html
[^graphops-node2vec]: graphops 0.5.1 Node2Vec walk-generation scope: https://docs.rs/graphops/0.5.1/graphops/node2vec/index.html
[^graphina]: Graphina Rust module and feature catalog (rendered 0.4.0-alpha.6 at review): https://docs.rs/graphina/latest/graphina/
[^rust-igraph]: rust-igraph 0.7.0 API, implementation description, and declared license: https://docs.rs/rust-igraph/0.7.0/rust_igraph/
[^raphtory]: Raphtory 0.17.0 Rust API and declared license: https://docs.rs/raphtory/0.17.0/raphtory/
[^raphtory-persistent]: Raphtory 0.17.0 PersistentGraph time/window/deletion semantics: https://docs.rs/raphtory/0.17.0/raphtory/db/graph/views/deletion_graph/struct.PersistentGraph.html
