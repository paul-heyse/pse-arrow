# Design review — aligning pse-arrow with the Rust computation architecture guidelines

**Implementation-status addendum (2026-09-23).** This review retains its original
baseline, findings and scoped verdict. Plan 13 is now active; see the
[W19 repair checkpoint](../../plans/13-w19-repair-checkpoint.md) for implemented
changes, current isolated/static evidence and the incomplete functional campaign.
W20 and final G1–G7 acceptance remain open. This pointer does not convert the
original review or its historical measurements into current product qualification.


Standard: `DATA_MODEL_DESIGN_CHARTER.md` (DM-01–DM-60, G1–G7) and
`rust_computation_architecture_guidelines_graph_extended.md` (cited **RCA §n**).
Mode: **both** — the blueprint (revision 43, working tree) and the code that claims to implement it.
Depth: **deep**. Date: 2026-09-23.

## 1. Decision and scope

**Proposal.** Re-allocate every computation in the model compiler and engine to the mechanism
RCA §1 assigns it — Arrow, DataFusion, a graph backend, Salsa, a specialised kernel or Delta —
instead of the present rule that DataFusion implements "every product data operation"
(blueprint D10, §3.3.3). Concretely:

- Salsa owns semantic-compilation reuse (P0–P10) and replaces the bespoke in-process memo machinery.
- petgraph, with rustworkx-core where it adds a needed routine, owns topology, closure, ordering
  and structural analysis, through typed projections.
- DataFusion keeps set-oriented relational work: rules, validation joins, case and scenario data,
  batch evaluation and results.
- Delta keeps pinned releases and selected durable artifacts.
- The crate graph is layered so that domain semantics compile without DataFusion or Delta.

**Status.** Target design: *Proposed*. Current-code observations: *Implemented* where a
`file:line` is cited. Compile and runtime figures: *Measured* under the conditions in §1.2.
Library capability statements: *Interface-checked* from the local `rust-graphs`, `salsa`,
`datafusion` and `deltalake` skills (their compile and behaviour probes) and context7, except
where noted.

**Reviewer.** Claude (design-review skill), for paul-heyse.
**Affected revisions.** Commit `a46f358` plus the uncommitted working tree; blueprint revision 43;
ADR-0065, ADR-0066, ADR-0068 (proposed), ADR-0074 (proposed); register R-01, R-22.

**Observable outcome sought.**
1. Shorter build and test cycles for the engineering test problems.
2. A model compiler whose graph, recursion and reuse work uses maintained libraries rather than
   hand-rolled loops and per-entity queries.
3. One reuse mechanism with sound semantic equality in place of six pointer-identity predicates.
4. Each operation's backend and reuse boundary stated explicitly (RCA §1, §9).

**Baseline.** A process model is compiled by composing P3–P10 as DataFusion `LogicalPlan`
extension nodes (`crates/pse-compiler/src/native/model.rs:66-77`, a fixed `vec![P3 … P10]`).
Inside each pass, plans are executed and decoded into Rust `BTreeMap`s, and the real work is done
there (§4). Graph work is split three ways:

- recursive SQL rules and recursive CTEs;
- DataFusion join and anti-join frontier loops;
- about a dozen hand-rolled DFS, BFS and topological-sort routines.

petgraph has exactly one code use (`crates/pse-compiler/src/passes/p5/tears.rs:18`), which
ADR-0075 withdraws. Salsa is not a dependency (register R-01). Reuse is keyed on `Arc::ptr_eq` and
a random per-assembly UUIDv7 (`crates/pse-engine/src/session/factory.rs:319-321`). The structural
analysis crate is an empty stub (`crates/pse-structural/src/lib.rs`).

**Supported scope.**
- **In scope:** the allocation of mechanisms across authoring/P0–P2, P3–P10, the engine/catalog
  reuse and publication machinery, the planned structural analysis (blueprint §15.3), the crate
  and dependency structure as it drives build time, and the selection of graph libraries and
  library features.
- **Non-goals:** Ipopt and numerical-solver internals; the Python/Pyomo boundary (ADR-0075, R-32);
  IDAES parity; results post-processing beyond placement.

**Constraints and uncertainty.**
- Salsa facts are for **0.28.4** (skill pin). petgraph **0.8.3** is the only petgraph in
  `Cargo.lock`, shared with DataFusion's `datafusion-physical-expr`. rustworkx-core **0.18.1**
  targets petgraph 0.8.
- No Salsa prototype was built. Every Salsa-cost statement is a hypothesis (DM-39) until the §9
  measurements exist.

### 1.1 Method and coverage

**Read directly** (citations in this review are to lines I read myself):
- The RCA document in full.
- Blueprint §1–§2 (D1–D14), §3.3.1, §3.3.3, §4.2, §14.1–§14.4 and §15.3.
- ADR-0020, ADR-0042 and ADR-0074 (Outcome sections); register R-01.
- Code at:
  - `native/model.rs:22-78`
  - `p9.rs:230-258`, `p9/kernels.rs:55-68`
  - `p3/config/lookup.rs:25-31`
  - `p4/predicates.rs:280-292`
  - `p5/ports/walk.rs:40-60`
  - `pse-authoring/src/p0.rs:134-172`, `targets/native.rs:340-375`
  - `pse-schema/src/checks.rs:172-192`
  - `pse-engine/src/session/factory.rs:314-321`, `session/reuse.rs:215-235`
  - `pse-catalog/src/artifact.rs:222-234`, `artifact/consumption.rs:186-196`, `delta/publish.rs`
    (function list and control-row comment)
  - `pse-mathir/src/graph.rs:56-66`, `pse-numerics/src/arena.rs:18-28`
  - `p5-containment-transitive.sql`
  - every crate's `[dependencies]`
- The rust-graphs, salsa, datafusion and deltalake skills: coverage and algorithm indexes,
  behaviour probes, features, library pages.
- context7: `pathfinding` and `faer`.

**Mapped by four read-only agents and treated as leads:**
- pass-by-pass mechanism tables;
- memo-table inventories and line counts;
- the generated-code anatomy;
- the per-crate surveys.

Where a lead carries a finding, I re-read the cited line. Leads not re-read are marked *(lead)*.
Line totals for the reuse machinery (about 13k) and the generated tree (443 relation files, about
820 lines each) are the agents' `wc`/`grep` counts, *(lead)*.

**Not inspected:** Python code; `pse-py` beyond its dependencies; Delta maintenance internals;
DataFusion optimizer-rule correctness; parity tests. Guarantees not attacked are recorded as
*asserted* in §6.

### 1.2 Measurements

All builds used an isolated `CARGO_TARGET_DIR` in scratch space, the `dev` profile (which sets
`opt-level = 2` for dependencies), `--locked`, and 32 cores / 188 GB. The toolchain was the pinned
rustc 1.98.1.

| # | What | Result | Conditions |
|---|---|---|---|
| M1 | Cold `cargo build --tests -p pse-tests-engine --features pse-relations/force-validate --timings` | **4 min 21.7 s wall, 44 min 42 s CPU**, 507 units | empty target dir |
| M2 | Critical path, from the M1 timing data (start → end, seconds) | sqlparser 11.2→34.7 · datafusion-common 35.8→41.8 · … · datafusion 101.5→142.4 · **pse-relations 130.5→174.2 · pse-engine 170.2→222.0 · pse-catalog 219.7→244.6 · pse-compiler 243.8→255.7** · test binaries 255.7→261.7 | pse crates are serial for 131 s of 262 s |
| M3 | Largest units in M1 | pse-engine 51.8 s · pse-relations 43.8 · deltalake-core 42.3 · datafusion 40.9 · datafusion-catalog 28.7 · pse-catalog 24.9 · aws-lc-sys build script 16.5 · zstd-sys 12.2 · pse-compiler 11.9 | unit wall time |
| M4 | Incremental rebuild after appending a comment | tests-only 1.3 s · pse-compiler 5.2 · pse-engine 6.1 · pse-relations 12.0 · pse-diagnostics 12.8 | warm M1 target |
| M5 | Incremental rebuild after adding a `pub fn` (the public API changes) | pse-compiler 15.5 s · pse-engine 7.6 · pse-relations 16.6 · pse-schema 17.0 · pse-diagnostics 15.7 | warm M1 target |
| M6 | Rebuild of the engine tests **without** `force-validate` after M1 | **239 s** — effectively a full rebuild (507 units; pse-engine 51.5 s, pse-relations 43.7 s): `arrow/force_validate` changes the features of `arrow-data`/`arrow-array`, invalidating every dependent | same target dir; this is the switch between `check`/`clippy` and `just test` |
| M7 | `cargo build --workspace --tests --features pse-relations/force-validate` after M1 | **257 s more** (718 units, including each crate's unit-test build: pse-engine lib-test 60.9 s, pse-relations lib-test 32.2 s); scratch target dir 67 GB after M1+M6+M7 | same target dir |
| M8 | Unique normal-dependency crates per package (`cargo tree -e normal`) | pse-diagnostics **193** · pse-ids 245 · pse-quantity 247 · pse-mathir 253 · pse-schema 252 · pse-relations 351 · pse-engine 384 · pse-catalog 504 · pse-compiler **528** | lockfile as committed |
| M9 | Model-compilation runtime: `native_engineering_workflows` (5 tests, P3 or P3–P10) | **All 5 tests terminated by nextest at 900 s each** (`.config/nextest.toml`: period 300 s × terminate-after 3); 0 passed, total 4,500 s — including the P3-only `material_configuration_…` test. The repository's own note records the heater reaching P10 in 293.374 s on 2026-09-15 (`.config/nextest.toml`, the `native_engineering_workflows` override). Stack samples: §1.3 | built test binary, `--test-threads=1` |

**Reading M1–M9.** "Compile time for the test engineering problems" has two components, and they
differ by more than an order of magnitude.

1. **The model compiler's runtime is the dominant cost (M9).**
   - A heater or mixer flowsheet does not get from source to canonical graph within 15 minutes.
   - Even the P3-only test does not finish.
   - Eight days earlier the heater reached P10 in 293 s, so the cost is growing as passes are added.
   - §1.3 locates it: DataFusion *logical-plan construction* over a composed cross-pass plan tree.
     §7 F0 carries the analysis.
2. **Rust build time is secondary but real.**
   - Warm incremental rebuilds are cheap (M4, M5: 1–17 s).
   - A whole-graph rebuild costs about 4 minutes and 45 CPU-minutes (M1). It happens on every
     cold build or fresh worktree, on every `force-validate` flip (M6: 239 s), and on every
     lockfile or toolchain change.
   - Two structural causes make those rebuilds long: the serial pse chain in M2, and a
     DataFusion/Delta dependency cone reaching crates that do no data processing (M8).

   §7 F5 carries the analysis.

### 1.3 Where the model-compile time goes (M9, stack samples)

**Method.** `perf` is unavailable (`perf_event_paranoid = 4`), so I sampled the running test
process with gdb instead: 8 innermost-40-frame samples of all threads, then 3 full backtraces of
the worker thread. The target was `heater_fctp_source_to_canonical_graph`, 10 to 13 minutes into
its run.

**Result.**
- **Innermost frames, 8 of 8:** inside
  `datafusion_expr::logical_plan::plan::LogicalPlan::using_columns`, recursing through
  `apply_with_subqueries` → `apply_children` → `Expr::apply`, including
  `<pse_engine::session::contract::ExecutionContract as UserDefinedLogicalNode>::inputs`.
- **Full backtraces, 3 of 3 (216–243 frames):** the pse frames are identical:
  ```text
  <P9 as Algorithm>::compose_queries        crates/pse-compiler/src/passes/p9.rs:54
   → p9::selections::plans                  crates/pse-compiler/src/passes/p9/selections.rs:177
   → passes::native_rows::join              crates/pse-compiler/src/passes/native_rows.rs:235
   → LogicalPlanBuilder::join_detailed      (datafusion-expr 55.1.0)
   → … LogicalPlan::using_columns (whole input tree)
  ```

**Interpretation** (*Measured* as to location, *hypothesis* as to mechanism).
- **The time goes into building a plan, not into executing one.** The CPU is spent on the *plan
  construction* of P9's selection query, not on executing any data operation.
- **P9's inputs are not materialised tables.** They are the lazily composed plans of P3–P8
  (`native/model.rs:27`: "This builds a graph; it neither runs algorithms nor creates intermediate
  publications").
- **A shared upstream plan is walked once per reference.** A `LogicalPlan` is a tree of `Arc`
  children, so a sub-plan referenced from several downstream places is walked once per reference.
  Each builder operation that normalises columns (`join_detailed`, and projections over
  `USING`-sensitive inputs) re-walks the whole upstream tree. The walk therefore grows with the
  *expanded* size of the composed compiler, not with its data.
- **The P3-only test was not sampled.** That it also exceeds 900 s says the pattern is not unique
  to P9, but where its time goes is **unverified**.

## 2. Authority and lifecycle map

Reconstructed for the **target** design. Cells marked † are decisions the blueprint has not yet
made.

| Concept or fact | Semantic type and identity | Authority / owner | Revision or snapshot boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Authored model (packages, cases) | `authored.*` relations; `SemanticId` (D4) | Delta release: the publication control row naming exact member versions | One publication (manifest of table versions, RCA §8) | Authoring → P1/P2 validation → publish | Salsa inputs (per document/package †); normalized/inferred/compiled results |
| Registry declarations | `RelationDecl`, `RuleDecl` in `pse-schema/src/catalog` | Source code (one declaration per meaning) | Registry fingerprint | Code change → `just codegen` | Generated `pse-relations` tree, `contracts.rs` mirror (checked, not independent), docs |
| Semantic-compilation results (P3–P10) | Typed Rust values and `ExprGraph` arenas keyed by semantic IDs | **Derived.** Salsa tracked results † (today: DataFusion plans re-executed per process) | Salsa revision; for durable reuse, the release manifest plus a stable implementation version † | Never edited; recomputed from inputs | Arrow relations at publication; `CanonicalMathProblem` bundle |
| Graph projections | `GraphProjectionSpec` per relationship kind † (RCA §5.1) | Derived from the semantic results | Same revision as the source result | Rebuilt, not maintained dynamically (RCA §5) | petgraph containers with a `SemanticId ↔ NodeIndex` map |
| Implementation identity | Today: random UUIDv7 per assembly (`factory.rs:319-321`). Target: crate version + registry fingerprint + per-algorithm version constant † | Build | Build | Release | Reuse keys; provenance |
| Reuse state | Today: six memo tables (§7 F2). Target: the Salsa database | Salsa | Salsa revision | Salsa input setters only, outside tracked code (RCA §3) | — |
| Case values, runtime results | `runtime.*` relations | Delta tables per run | Run | Run commit | DataFusion reports, sweeps |

**Deliberately opaque behaviour.** The Ipopt solve, Pyomo tear selection (ADR-0075) and the
Hopcroft–Karp kernel (§7 F6). Each is a `KernelCall`/`SolverBlock` under a contract (RCA §2
DEFAULT vocabulary).

**Identity behaviour.**
- **Salsa `Id`s and petgraph `NodeIndex`es are never persisted or published** (RCA §3 MUST,
  §5.2 MUST). Salsa interned slots are reused after garbage collection (salsa skill probes
  B019/B021), and petgraph `Graph` indices shift on removal.
- `SemanticId` remains the only published identity.
- A rename changes `qualified_name` only (D4). In Salsa terms it must be a tracked field of the
  entity, **not** part of its identity. A rename would otherwise discard every query keyed on the
  entity (probe B018).

## 3. Semantic contracts and invariants

| Contract or invariant | Representation | Enforcement boundary | Failure behaviour | Verification evidence |
|---|---|---|---|---|
| Expression identity keeps opcode, result type, ordered operands and literals (RCA §2 MUST) | `ExprGraph::typed_key` byte key over (opcode, payload, children, quantity type) (`graph.rs:59-64`; key body *(lead)* `graph.rs:~555`) | Insert-time hash-consing | Equal structure returns the first `NodeId` | Existing mathir tests *(lead)*. Must survive a Salsa move: intern on the same key, never on allocation. |
| Reuse substitutability (RCA §4 MUST sound equality) | Today: `Arc::ptr_eq` plus settings equality (`reuse.rs:215-235`) | Per memo table | Conservative: equal semantics compare unequal, so work is recomputed | No cross-process reuse is possible (`artifact.rs:228-230`) |
| Dependency completeness (RCA §3 MUST) | Today: static plan analysis (`session/dependencies.rs`, *(lead)*). Target: Salsa's observed reads | Salsa records every tracked read; configuration must be an `#[salsa::input]` (probe B003 shows a plain field read returns a **stale** value) | Stale result, **silent** | Clean-versus-incremental comparison (§9 V3) |
| Absence tracking (RCA §3 DEFAULT) | Today: `Source::Absent` (`assembly/derived.rs:13-19`, *(lead)*). Target: tracked membership indices per scope † | Salsa tracked index functions | An addition invalidates the lookup that missed | V3 with a failed-lookup-then-add edit |
| Graph projection semantics (RCA §5.1 MUST) | None today † | — | — | §7 F4, F6 |
| Release consistency (RCA §8 MUST) | Publication control row committed after its members (`delta/publish.rs:52`: "Publish a declared control row after all native dependencies complete") | Delta application transaction plus parent admission (`publish.rs:387` `admit_parent`) | Old reference unchanged | *Asserted.* Not attacked in this review. |

**Absence and uncertainty.** Unchanged by this proposal, with one addition. Salsa distinguishes a
*cancelled* query (unwinds with `Cancelled`) from a failed one (returns `Err`). The blueprint's
`runtime.cancelled` failure class must map from the former. Probe B024 shows cancellation is
**ignored** inside a query that has a `cycle_initial` fixpoint strategy, so fixpoint queries need
their own bounded round limit, as the rule strata already have (`strata/mod.rs:143`, *(lead)*).

**Equivalence requirements.** Salsa backdating (probe B001) uses `PartialEq` on results. Results
carrying `f64` must use the blueprint's canonical float semantics (§5.3: `-0.0`, NaN) rather than
IEEE `==`. Otherwise `NaN != NaN` forces needless re-execution and `-0.0 == 0.0` hides a change
(RCA §4 MUST).

## 4. Derivation and execution design

### 4.1 Backend and reuse-boundary allocation per stage (RCA §1)

"Today" is what executes; "Target" is the recommendation. Every row keeps publication to typed
Arrow relations at the boundary.

| Stage | Today (evidence) | Target backend | Target reuse boundary |
|---|---|---|---|
| Authoring parse | YAML/TOML → serde DTOs → Arrow batches; the parser also runs as a DataFusion `ScalarUDF` inside an aggregate/unnest plan (`pse-authoring/src/native.rs:25-90`, *(lead)*) | Plain Rust parser → typed documents | Salsa input per document; text in, AST out |
| Name resolution | DataFusion cross and inner joins (`binding/native.rs:102,128`, *(lead)*) | Rust maps over typed documents | Tracked function per package scope |
| P0 package order | O(n²) fixpoint; the cycle error carries no witness (`p0.rs:136-170`: `return Err(contract("cyclic package dependency"))`) | petgraph `toposort`, with `tarjan_scc` for the cycle witness | Tracked function over package headers |
| P1/P2 staging and validation | DataFusion validation programs (`pse-rules/src/invariants.rs`, *(lead)*) | **DataFusion (keep).** Key, foreign-key and uniqueness checks over actual candidate data are relational (RCA §5). | Per candidate release |
| P3 template/instance expansion | Worklist with retry-on-missing-parent (`p3/config/instances.rs:63`, *(lead)*); a private in-memory query engine over decoded batches (`p3/config/lookup.rs:25-31`: `fields: BTreeMap<String, HashMap<ScalarValue, BTreeSet<(usize, usize)>>>`); per-path id queries (`p3/source/support.rs:211-249`, *(lead)*) | Demand-driven keyed compilation (RCA §6 row 1): one tracked function per (template, structural configuration), i.e. RCA's *specialization*; instance tree as a petgraph `StableGraph` | Tracked per specialization; instances share the specialization's result |
| P3 product/domain ancestry | Recursive CTE with a visited-array cycle guard (`p3/products/ancestry.rs:166`, *(lead)*) | petgraph traversal over the containment projection | Same tracked result as the instance tree |
| P4 features and predicates | Datalog SQL rules (semi-naive, `pse-rules/src/strata`); predicate tuples via one cross-join query **per source × predicate × instance** (`p4/predicates.rs:286`: `inventory.tuples(&domains, ctx.cancel).await?` inside the per-instance loop) | Set-oriented rules: **DataFusion (keep)**. Per-instance predicate evaluation: Rust over typed domain members, since `evaluate_predicate` is already a Rust interpreter (`predicates.rs:423`, *(lead)*). | Tracked per specialization |
| P5 containment closure | SQL rule iterated to fixpoint (`p5-containment-transitive.sql`: `JOIN "inferred"."instances" AS "children" ON ("paths"."descendant_id" = "children"."parent_instance_id")`), materialising every ancestor/descendant pair | petgraph tree plus on-demand `Dfs`/`has_path_connecting`; **no full closure** unless a consumer requires it (RCA §6 DEFAULT) | Tracked with the instance tree |
| P5 port walks, topology | DataFusion frontier loop, ≥4 executions per step (`p5/ports/walk.rs:43-60`) | Rust walk over typed port-binding steps; topology as a typed multigraph projection (§4.3) | Tracked per flowsheet scope |
| P6 demand closure | Recursive SQL rule plus DataFusion cursor loops (`p6/framing/dependencies.rs:239`, *(lead)*) | Monotone worklist (RCA §6 row 4) in Rust; the rule stays relational if it is set-oriented at scale † | Tracked per demand scope |
| P7 realization | Rust `pse_templates::instantiate` into one `ExprGraph` (`p7/expressions.rs:24,292`, *(lead)*): already the right backend | Unchanged; the input arrives from Salsa instead of about 40 decoding queries (`p7/inventory.rs:216-269`, *(lead)*) | Tracked per specialization/instance |
| P8–P10 math | Each pass reloads the whole graph from relations and re-emits it (`p8/graph.rs:67`, *(lead)*) | Pass owned `ExprGraph` results between passes; emit relations once, at publication | Tracked per law/method/canonical unit |
| P9 descendants | DataFusion BFS with two materialisations per level (`p9.rs:234-255`, read) | petgraph `Bfs` or rustworkx-core `descendants` on the instance tree | Shared tracked projection |
| P9 kernel order | O(n²) selection-sort topological order (`p9/kernels.rs:59-65`, read) | petgraph `toposort`, or rustworkx-core `lexicographical_topological_sort` for a semantic-ID tie-break | — |
| Rule stratification | Strata are hand-written `u16`s that are only checked (`pse-schema/src/checks.rs:175-191`, read) | Derived: rule-dependency graph → `tarjan_scc` → condensation → topological layers; reject a negative edge inside an SCC | Computed at registry build |
| Structural analysis (§15.3) | Not implemented (`pse-structural/src/lib.rs`) | petgraph `tarjan_scc`/`condensation`; rustworkx-core `lexicographical_topological_sort`; own Hopcroft–Karp kernel (§7 F6) | Tracked per problem; compilation dependency (RCA §7 MUST) |
| Numeric evaluation | DataFusion `Expr` → `PhysicalExpr` stages plus a hand-written arena, symbolic differentiation and scalar interpreter (`pse-numerics`, *(lead)*) | Specialised kernel lowering from `ExprGraph`; DataFusion `PhysicalExpr` only for batch/UDF evaluation † | Compiled program is a compilation artifact; values are runtime |
| Case, scenario and results | DataFusion/Delta | **Keep** | Execution-result reuse, separate from compilation reuse (RCA §7 MUST) |

### 4.2 Computation contracts (RCA §9), for the stages that carry findings

Cells marked † are undecided in the blueprint.

| Field | P3 specialization | Containment / topology projection | Structural analysis (BTD) | Reuse (all stages) |
|---|---|---|---|---|
| Semantic output and equality | normalized bindings per specialization; structural `PartialEq` † | adjacency plus ID map; equality by source revision | blocks (`compiled.blocks`, `kind = scc`, `order`) with ties broken by semantic ID (§15.3) | Salsa backdating on `PartialEq` |
| Backend / version / features | Rust + Salsa 0.28.4 † | petgraph 0.8.3 `stable_graph` | petgraph + rustworkx-core 0.18.1 † + kernel | Salsa default features; `persistence` optional † |
| Projection, ID mapping, configuration | n/a | direction `parent → child`; parallel edges retained for ports †; isolates kept † | bipartite incidence of active equalities × free variables (IDAES `include_inequality=False, include_fixed=False`); explicit dense remap † | n/a |
| Tracked boundary | (template, structural configuration) | flowsheet scope | problem | per stage |
| Observed dependencies incl. absence | template rows, child bindings, feature flags; member lookups through tracked indices † | instance membership (additions and removals) | incidence rows including zero-coefficient elimination (§15.3 `standard_repn` rule) | Salsa reads |
| Structural vs runtime inputs | a parameter changing topology, shape or code selection is structural (RCA §4); today's §14.4 table is compatible | structural only | structural; value-dependent zero-coefficient elision is a dependency (§14.4) | — |
| Cycles / termination | containment cycle → typed error with witness † | cycle → witness via `tarjan_scc` | an SCC is a block, not a solution (RCA §6 MUST) | fixpoint queries bounded (probe B024) |
| Exactness / reproducibility | exact | exact over the stated projection | exact; deterministic tie order | — |
| Cardinality and cost | per specialization, not per instance † | O(V+E); no closure | Hopcroft–Karp O(E√V); SCC O(V+E) | memo metadata bounded with `lru` (probe B014) † |
| Ownership / cancellation / publication | owned by the database; published only at the boundary | rebuilt, not mutated | same | Salsa cancellation unwinds; solver work stays outside tracked queries (RCA §7) |
| Departure from defaults | — | — | own matching kernel: no permissive Hopcroft–Karp exists (F6) | — |

### 4.3 Relationship structures (DM-34, RCA §5)

The code handles at least nine distinct relationship kinds, each with its own traversal:

| Relationship kind | Shape | Handled today by |
|---|---|---|
| Package dependency | DAG | `p0.rs` |
| Template child composition | DAG | P3 `instances.rs` |
| Instance containment | tree | SQL closure, P9 BFS, `targets/native.rs` per-hop queries |
| Port/arc connectivity | **directed multigraph**; ports and parallel arcs are meaningful | P5 |
| Expression | DAG | `pse-mathir/view.rs` |
| Rule dependency | general digraph | hand-assigned strata |
| Kernel dependency | DAG | `p9/kernels.rs` |
| Unit dependency | DAG | P3 `support.rs` *(lead)* |
| Incidence | bipartite | planned |

Directions follow RCA §5 (`prerequisite → dependent`; prerequisite discovery uses incoming edges,
`petgraph::visit::Reversed`).

The recommendation is **one small typed projection module**, not a universal graph framework
(RCA §5.2 DEFAULT). It provides one `GraphProjectionSpec` per kind above. Each spec produces:

- a petgraph container: `StableGraph` where removal-stable indices matter, and never `GraphMap`
  for connectivity, because it collapses parallel edges;
- an explicit `SemanticId ↔ NodeIndex` map;
- the handful of algorithms each kind needs.

**Provider selection and limitations.**

| Library | What it gives | Limitations |
|---|---|---|
| petgraph 0.8.3, already pinned with `stable_graph` and `graphmap` | `tarjan_scc`, `condensation`, `toposort`, `is_cyclic_directed`, `has_path_connecting`, `Dfs`/`Bfs`/`DfsPostOrder`, `Reversed` | Topological ties are broken by insertion order, not a key. `maximum_matching` is general-graph Gabow, O(V³). |
| rustworkx-core 0.18.1 | Zero-copy over the same containers (rust-graphs skill, interop `&container`): `lexicographical_topological_sort`, `layers`, `ancestors`, `descendants`, `cycle_basis`, `johnson_simple_cycles`, `find_cycle` | Declares no features, so rayon, ndarray, hashbrown 0.17 and indexmap are **always compiled**. Its sets are hashbrown 0.17, not std or petgraph's 0.15 (rust-graphs probe C005). Adopt only when `lexicographical_topological_sort` or `layers` is actually used. |

**Not recommended now** (RCA §5.2 DEFAULT: add a specialist only for a concrete capability):
- **leiden-rs, graphops, graphina:** community and centrality analytics have no consumer here.
- **raphtory:** dynamic simulation time is a *math* domain, not a temporal graph; RCA §5.4's
  temporal semantics do not arise.
- **rust-igraph:** its `maximum_bipartite_matching` could serve as a **test-only** differential
  oracle for the matching kernel, noting register R-31 for GPL-2.0-or-later (REFERENCE §6 T3).

## 5. Representative journeys

### Ordinary extension: add a flash unit template

**Today.** A reference-package YAML template, plus SQL rules only if new inference is needed. The
compiler, generated relations and engine are unchanged; this data-only extension path is a real
strength (DM-16, DM-17).

**Target.** The same. Salsa tracked functions are keyed on template and specialization data, so a
new template needs no new orchestration code, only model data (RCA §2 DEFAULT). **Preserve this in
the migration.**

### Meaningful change: edit one heater's outlet-temperature specification (runtime value), then a child-template binding (structural)

**Today.**
- **Inside one process,** reuse depends on the same `Arc` owners still being alive
  (`reuse.rs:222-235`: `Arc::ptr_eq(a, &b)`).
- **In a new process** — every test, CLI run or Python session — every opaque implementation gets
  a fresh UUIDv7 generation, so "a cold reconstruction must execute freshly even when every display
  name is unchanged" (`artifact.rs:228-230`). P3–P10 recompute from scratch.

The durable dependency receipts in Delta `commitInfo`, and the CDF-diff comparison
(`consumption.rs:186-196`), therefore only ever validate reuse inside a process that already holds
the in-memory memo.

**Target.**
- **The value edit** sets one LOW-durability input; P3–P12 structural results are revalidated,
  not recomputed. HIGH durability on reference packages lets Salsa skip walking them (probe B004).
  This is RCA §9's required check that nonstructural edits preserve structural artifacts.
- **The binding edit** invalidates only that specialization and its consumers; backdating (probe
  B001) stops propagation where a result is unchanged.
- **Cross-process warm starts** need either:
  - Salsa `persistence`, with these caveats: accumulators are not persisted (B026); a restored
    query re-runs once an interned dependency is garbage-collected (B036); every input a persisted
    query reads must itself be persisted (B027 panics otherwise); or
  - published compiled artifacts keyed by the release manifest plus the stable implementation
    version (§2).

  Choosing between them is a decision for the author (§10 D3).

### Boundary: tracked result → Arrow → Pyomo/NL

Salsa results are typed Rust. The generated `Builder`s turn them into Arrow relations **once, at
publication** (RCA §7). The `CanonicalMathProblem` bundle crossing into Python is unchanged (D12,
ADR-0075). This is the loss-relevant boundary; the generated builders' field checks remain the
validator (DM-42).

### Interruption: cancel a compile mid-P7; an Ipopt run is in flight

**Salsa side.** A write on one handle cancels queries on another (probe B012); the cancelled
handle's next call runs normally (B023). A fixpoint query ignores local cancellation (B024), so
fixpoint work must be bounded.

**Ipopt side.** Ipopt callbacks `catch_unwind` (blueprint §18.3). ADR-0020's objection —
"unwind-based cancellation cannot cross the Ipopt boundary" — is met by RCA §7's default: solver
execution is external orchestration over an owned executable specification, **never inside a
tracked query**. Publication stays an explicit effect outside Salsa (RCA §3 MUST input ownership).

## 6. Acceptance gates

| Gate | Result | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Unresolved** | Blueprint D10/§3.3.3 ("native DataFusion plans … implement all product data operations") and D14/§14.3 conflict with RCA §1–§2 (REFERENCE §6 T1, T2, T4). An interim rule exists (REFERENCE §6 "Until resolved"), so this is not an unreconciled conflict today, but the direction requested here has no authority record. Blueprint §14.3 still prescribes "stage hints" and `provenance.pass_records`; no crate contains either (grep), ADR-0068 deleted them, and §3.3.3's blanket "replace conflicting … prescriptions" leaves the reader to work out which sentences survive. | ADR (with review) superseding D10's allocation rule and D14's mechanism; blueprint `design:` PR (§11 C1). Adopting Salsa **beside** the existing descriptors without retiring them would turn this into a **Fail** (T1). |
| G2 — Semantic fidelity | **Unresolved** | No `GraphProjectionSpec` exists for topology, containment, incidence or rule dependency (RCA §5.1 MUST). Today's one graph site drops weights by design (`tears.rs:173`), which ADR-0075 corrects. | Declare projection specs (F4, F6) before any graph result becomes a compilation dependency. |
| G3 — Validity | **Unresolved (target)** | Moving P3–P10 internals from relations to typed Rust relocates enforcement: today field checks run at `Builder`/`FieldCheckedBatch` construction and relational obligations in DataFusion. The target must restate which invariants are enforced by typed construction, which by the publication builders, and which remain relational. Not re-examined for current code. | Stage table of enforcement boundaries in the implementing plan. |
| G4 — Hidden behaviour | **Pass (current, scoped) / conditional (target)** | The UUIDv7 generation read (`factory.rs:319-321`) is a declared, opaque implementation identity whose only effect is to suppress reuse. No mutation was found in the inspected inspection paths. In the target, any clock, random or configuration read inside a tracked function would be a hidden input (probe B003). | Salsa `project-` lint rules from the salsa skill, plus a governance test that tracked functions do not read `uuid::now_v7`, `SystemTime`, env or statics. |
| G5 — Consistency and recovery | **Pass (asserted)** | The control row is committed after its members (`publish.rs:52`, `admit_parent` at `:387`). Not attacked here. | Keep the manifest boundary unchanged in the target; Salsa state is never a published authority. |
| G6 — Transformation and reuse | **Pass (current: conservative) / Unresolved (target)** | Pointer-identity reuse only produces false negatives, so no stale reuse was found. The target's soundness depends on complete observed reads and sound `PartialEq`. The existing `incremental_equals_clean_p0_p3` test *(lead)* covers P0–P3 only. | RCA §9 clean-versus-incremental suite over P0–P10 (V3), with Salsa events as the reuse oracle (V4). |
| G7 — Truthful capability claims | **Pass (scoped)** | `pse-structural` says "no implementation yet"; §14.1 labels its table a design inventory; ADR-0075 removed the false §12.5 tear claims. | Keep new graph/Salsa claims labelled *Proposed* until V1–V6 exist. |

**F0 and the gates.** F0 violates an RCA §6 MUST, which REFERENCE §1b maps to G2 and G5. I have
not failed either gate for it. No wrong, partial or inconsistent result is published: the run
simply does not finish. The MUST gap decides §11 directly instead.

## 7. Principle findings

Ordered by severity (charter calibration): MUST-level and authority defects first, then
duplication and extension cost, then measured cost.

| # | Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| F0 | **The whole compiler is composed as one lazy DataFusion plan across passes. Model compilation time is then dominated by re-walking that expanded plan tree during plan construction, and there is no resource bound or diagnostic when it runs away.** | RCA §6 MUST ("preserve sharing through backend lowering rather than expanding a compact IR into duplicated plans"; "define termination and resource-failure behaviour"), RCA §7 MUST/DEFAULT (compiled plan versus materialisation; materialise where repeated work justifies it) · DM-39, DM-30, DM-38 | `native/model.rs:27`: "This builds a graph; it neither runs algorithms nor creates intermediate publications". The M9 stack path `p9.rs:54` → `p9/selections.rs:177` → `native_rows.rs:235` `join_detailed` → `LogicalPlan::using_columns` (8/8 and 3/3 samples, §1.3). M9: all five engineering workflows over 900 s; the heater took 293 s on 2026-09-15 (`.config/nextest.toml`). | The supported source→canonical-graph workflow does not complete for a single heater within the suite's diagnostic budget. Every new pass or rule multiplies the expanded tree that later passes re-walk, so cost grows super-linearly with compiler size rather than model size. The run neither fails with `ResourcesExhausted` nor reports progress; it is killed externally. This is the "enormous compile time" the maintainer reported. | Introduce an explicit **materialisation boundary at every pass output** (RCA §7 DEFAULT). Each pass consumes its predecessors' *materialised* relations (typed Rust or `MemTable`), never their plans. Plan composition stays *inside* a pass, where fusion pays (RCA §5). In the target this boundary is the Salsa tracked result (§4.1). As an interim step, the existing `retain` helper (`native_construction.rs:361`, *(lead)*) or `DataFrame::cache` can be applied at pass outputs. Add a planning-time budget with a typed refusal. | M9 re-run: each engineering workflow completes, with time recorded per pass; a governance check that no pass's input `TableScan` resolves to another pass's unmaterialised plan |
| F1 | **The backend-allocation rule in the authoritative design contradicts the adopted computation standard, and §14.3 describes deleted machinery.** D10/§3.3.3 make DataFusion the executor of "all product data operations", including "dependency analysis and incremental computation" and "semantic compilation"; RCA §1 assigns those to graph backends, Salsa and kernels. | DM-02, DM-05, DM-23 · RCA §1, §2 · G1 | Blueprint §2 D10 responsibility table rows "Semantic compilation", "Dependency analysis and incremental computation"; §3.3.3 ¶1; §14.3 "Prepared work and stage hints bind …" and "Complete terminal attempt records (revision 31)"; no `stage_hint`/`pass_records` in `crates/` | Two implementers get opposite instructions for the same pass: one writes a DataFusion frontier loop (as `p9.rs:234` does), the other a petgraph BFS. Reviews must apply REFERENCE §6's interim rule instead of the design, and each new pass re-litigates placement. | ADR "Allocate computation mechanisms per operation (RCA §1)" with review. It supersedes ADR-0065's D10 amendment and the reuse-mechanism parts of ADR-0068 and ADR-0074, and closes R-01 in favour of Salsa. A `design:` PR rewrites D10 as a per-operation allocation table (§4.1 of this review), D14 as "Salsa-observed dependencies for compilation; declared descriptors for durable artifacts", §1.3, §3.3.1–§3.3.3, §14.3–§14.4 and §15.3, and deletes the stale stage-hint and pass-record text. | `adr-lint`; REFERENCE §6 T1/T2/T4 rows deleted; a grep test that §14 names no mechanism absent from the code |
| F2 | **Reuse is keyed on allocation identity and a random per-process generation, spread over about six independent memo tables. Durable reuse is therefore unreachable across processes, and the machinery that validates it has no consumer.** | DM-32, DM-58, DM-02 · RCA §1 (no second invalidation graph), §4 MUST (no allocation identity as semantic equality) · G6 (cost) | `factory.rs:319-321` `Uuid::now_v7()`; `reuse.rs:222-235` `Arc::ptr_eq(a, &b)`; `artifact.rs:228-230` "a cold reconstruction must execute freshly"; CDF diff `consumption.rs:186-196`. Memo inventory *(lead)*: `session/cache.rs`, `cache/model.rs`, `operation/ports.rs`, `session/obligation.rs`, `cache/admission.rs`, `factory.rs`, about 13k lines including catalog `cache_service` | Every test, CLI run and Python session recompiles P3–P10 from nothing, even for an unchanged release. The Delta-receipt path (`delta/attempt.rs`, `delta/dependencies.rs`, `artifact/consumption.rs`, *(lead)*) and its CDF diff are exercised only where an in-memory memo already answers. Six validity predicates each re-decide relevance, so adding a new dependency aspect needs six edits (charter §E). | Salsa database for P0–P10. Stable implementation versions (§2). Configuration as inputs (B003). Reference packages at HIGH durability (B004). `lru` on large per-instance memos (B014). **Delete** the compile-path witness, epoch, obligation and model-result caches in the same cut (a T1 requirement). Keep DataFusion's own caching only for **execution-result** reuse (RCA §7 MUST). Repurpose the CDF code as the RCA §3 *ingestion adapter* (release N → N+1 row diff → Salsa input setters). | V3, V4, V5 |
| F3 | **Traversals and lookups run as per-entity or per-iteration DataFusion executions, and one pass reimplements a query engine to escape that cost.** | DM-38, DM-26, DM-56 · RCA §5 DEFAULT (no execution per entity or traversal step), §2 DEFAULT (typed internals) | `targets/native.rs:344-373` one filtered query per ancestor hop; `p9.rs:234-255` join, anti-join and two `retain` materialisations per BFS level; `p5/ports/walk.rs:43-60` frontier with ≥4 executions per step; `p4/predicates.rs:286` a cross-join query per source × predicate × instance; `p3/config/lookup.rs:25-31` `HashMap<ScalarValue, BTreeSet<(usize, usize)>>` over decoded batches, re-interpreting `Expr` equality and `InList` (`:154-166`, *(lead)*) | Compile time grows with query count × plan/optimise/execute overhead rather than data size (M9). Per-level materialisation makes the P9 BFS O(depth) executions. The P3 `Lookup` is a second, partial expression evaluator whose `Eq`/`InList` semantics can drift from DataFusion's (null, NaN, type coercion) without any test tying them. | Decode each relation once into typed Rust (it already becomes `BTreeMap`s: `passes/native_rows.rs:121-144`, *(lead)*). Do traversals in petgraph or plain Rust inside tracked functions. **Delete** `p3/config/lookup.rs`. Keep genuinely set-oriented work fused in DataFusion: rules, validation, and batched lookups as one join over all keys rather than a loop. | V6 (per-pass query-execution count via `pse-testkit` capture), M9 re-measured |
| F4 | **At least nine relationship kinds are traversed by separate hand-rolled or relational routines, with no typed projection and inconsistent cycle and tie-break semantics.** | DM-34, DM-22, DM-47, DM-40 · RCA §5 DEFAULT, §5.1 MUST, §6 MUST/DEFAULT | O(n²) topological sorts: `p0.rs:136-170`, `p9/kernels.rs:59-65`, P3 `instances.rs:63-138` *(lead)*. Full transitive closure materialised by `p5-containment-transitive.sql`. Strata hand-assigned and only checked (`checks.rs:175-191`). Separate DFS cycle checks in P3/P4/P7/P8/P10, mathir and templates *(lead)* | P0 reports "cyclic package dependency" with no members, so the author cannot find the cycle (DM-47). Tie order differs per site, which matters for §15.3's deterministic order. `instance_reachability` stores O(depth·n) pairs that most consumers answer by walking a parent chain (RCA §6: memoisation cannot remove an intrinsically large output). A new rule needs its stratum number chosen by hand, and a wrong choice is rejected rather than computed. | Typed projection module (§4.3): one `GraphProjectionSpec` per kind, petgraph containers, explicit ID maps. `toposort` plus `tarjan_scc` witnesses. Strata **derived** by SCC and condensation of the rule-dependency graph. Replace the closure relation with on-demand traversal unless a consumer needs all pairs (decision §10 D4). rustworkx-core `lexicographical_topological_sort` wherever a deterministic semantic-ID order is required. | Graph fixtures per RCA §9: asymmetry, parallel arcs, self-loops, isolates, index holes; cycle-witness tests |
| F5 | **The crate graph puts DataFusion and Delta under crates that do no data processing, and a 506k-line generated crate plus a 31k-line engine sit serially on the build's critical path.** | DM-57, DM-58, DM-39 · RCA §1 (separate layers), §2 DEFAULT (Arrow as boundary) | `pse-diagnostics` → `datafusion-common` with `parquet, object_store` (`lib.rs:10` `pub use datafusion_common::DataFusionError as NativeError`), so an error-code crate has 193 dependencies (M8). `pse-ids` → `datafusion-execution` for `MemoryReservation` (`resource.rs:7`). `pse-quantity`, `pse-material` and `pse-mathir` reach 247–253 dependencies only through those two. `pse-compiler` → `pse-catalog` → `deltalake-core` → rustls/aws-lc (`cargo tree -i aws-lc-rs`). `pse-relations/src/generated` is 20 MB. M2 critical path. | Every cold, feature-flip or worktree build pays the full DataFusion/Delta cone before any pse domain crate compiles. The four serial pse units (131 s) cannot start until DataFusion's metadata exists (M1, M2). A registry edit regenerates and recompiles the 506k-line crate (M5: 16.6 s incremental; 43.8 s cold). | **Layer the crates.** L0 domain (ids, diagnostics, quantity, material, mathir, templates, structural, the Salsa compiler core) has no arrow, datafusion, tokio or deltalake. L1 relations (arrow only). L2 engine and rules (DataFusion). L3 catalog (Delta). L4 runtime and py. Diagnostics own their error enum; the DataFusion bridge moves to L2. Memory accounting becomes a small L0 trait with a DataFusion-pool adapter in L2. The compiler core drops `pse-catalog`. **Shrink generated code:** one generic `TypedBatch<R: RelationRow>`/`RowBuilder<R>` replaces the per-relation pass-throughs (about 297 fixed lines × 443, *(lead)*); nested payload types are emitted once per distinct shape (the 7 `*_expr_nodes` trees, about 52k lines, *(lead)*); `contracts.rs` (118k lines, a frozen registry copy) becomes a fingerprint check; serde derives only on the ~92 relations that are document DTOs *(lead)*; relations split by namespace if consumers allow. | V7: M1/M2/M6 re-measured; `cargo tree` dependency ceilings per layer asserted in `tests/governance` |
| F6 | **Structural analysis (§15.3), the one place where graph results become compilation dependencies, has no projection specification, and the library choice is not recorded against the adopted libraries.** | DM-22, DM-43 · RCA §5 MUST (SCC region), §5.1 MUST, §5.2 MUST (algorithm identity), §6 MUST | `pse-structural/src/lib.rs` (stub). §15.3 specifies Hopcroft–Karp as "own implementation", Tarjan SCC and a semantic-ID tie order, but not the region scope (whole problem or requested outputs), the isolate policy (unmatched rows and columns), or the index-mapping contract. | Without a stated region, an implementer may restrict to the outputs' ancestors and miss a cycle through a non-ancestor, splitting one coupled block into sequential singletons (REFERENCE §3 worked example). The solve then converges elsewhere or fails with no diagnostic. | Specify `IncidenceProjection` (active equalities × free variables, linear-only zero-coefficient rule, whole problem scope) and `BlockAnalysis` (algorithm, deterministic order, exact). Use petgraph `tarjan_scc`/`condensation` and rustworkx-core `lexicographical_topological_sort(key = semantic id)`. Keep Hopcroft–Karp as a specialised kernel (RCA §1): no permissive crate provides it (rust-graphs coverage: only rust-igraph `maximum_bipartite_matching`, GPL-2.0-or-later; petgraph `maximum_matching` is Gabow O(V³); `pathfinding::kuhn_munkres` is a dense O(n³) weighted assignment, per context7). | Differential tests against scipy `maximum_bipartite_matching`/`structural_rank` (already in §15.3). A cycle-leaving-the-ancestor-set fixture asserting one block of size two. |
| F7 | **Mathematics passes through three representations with two independent hash-consers, and P8–P10 round-trip the whole graph through relations each pass.** | DM-21, DM-37, DM-56 · RCA §2 (Arrow as boundary; expression identity), §4 DEFAULT | Hash-consing in `pse-mathir/src/graph.rs:59-64` (`by_structure: BTreeMap<Vec<u8>, NodeId>`) and in `pse-numerics/src/arena.rs:19-25` (`lookup: HashMap<Expr, usize>`, keyed on DataFusion `Expr`). `p8/graph.rs:67` reloads the full graph *(lead)*. The generated `mathir_sink.rs` has 7 near-identical `Family` arms *(lead)*. | Two identity rules for one expression (byte key versus `Expr` structural hash): a canonicalisation difference, e.g. a literal's float encoding, makes nodes shared in one representation and duplicated in the other. Each pass pays a full decode and encode of the math graph, so cost is O(passes × graph). | `ExprGraph` is the single in-compiler expression representation, carried between passes as a tracked result. Relations are emitted at publication. The numeric tape lowers from `ExprGraph` directly; DataFusion `Expr` is used only where a batch `PhysicalExpr` evaluation is actually wanted. Amend D6 to "relational at publication; `ExprGraph` in compilation". | Equality of published `compiled.math_*` before and after; per-pass decode time (V6) |
| F8 | **Every non-evidence algorithm output of a full publication is written as its own Delta member,** where RCA assigns Delta to authoritative model data and *selected* durable artifacts. | DM-58, DM-36 · RCA §1 (Delta row), §7 DEFAULT (materialise where justified), §8 | `compose` inserts each non-`__` result into `outputs` (`native/model.rs:213-214`, *(lead)*), and `prepare_publication` pushes a `MemberWrite` per output (`artifact.rs:~433`, `Member::Write(MemberWrite {` read). | Publication cost and table count scale with the declared relation inventory (443 relation kinds) rather than with what consumers read. Intermediate normalized/inferred relations are reproducible from the release plus the compiler version. | Publish the authored release, the compiled problem bundle, diagnostics and runtime results. Make intermediate relations an **inspection mode** that materialises on request (§10 D5). | Publication member count per test; M9 |
| F9 | **Compiled-in capability exceeds used capability; "eligible" (ADR-0066) is being read as "enabled".** | DM-58 · RCA §1 | `datafusion = "=55.1.0"` with default features (compression, crypto, regex, unicode, encoding, nested, parquet, sql). The vendored delta-rs pins `rustls` on the kernel engine (`vendor/delta-rs/Cargo.toml:26`), which pulls aws-lc-sys (16.5 s C build, M3) and reqwest with no network use. | **Observation.** These units are off the cold critical path (aws-lc-sys finishes at 24 s, before sqlparser at 35 s; M2), so the wall-clock gain is small; the CPU gain (M1: 44 CPU-minutes) is real on smaller machines and in CI. `sql` must stay: 130 rules are SQL (`pse-schema/src/catalog/native_rules/`). | Enable only the features in use; record that eligibility is not enablement. Change the vendored kernel feature when the delta-rs pin next moves (no `[patch]`, per governance). | `cargo tree -e features` snapshot in governance |

**Applicability.**
- **Groups carrying findings:**
  - 6 and 8 (execution and resource behaviour): F0
  - 1 (authority): F1
  - 5 (lowering and representations): F7
  - 7 (dependencies and reuse): F2, F4
  - 8 (execution mechanism and layout): F3, F5, F8
  - 9 (capability exposure): F6
  - 12 (proportionality): F2, F5, F8, F9
- **Groups 3 (identity) and 6 (effects):** touched through the Salsa identity rules (§2) and the
  Ipopt boundary (§5), with no defect found.
- **Groups 2, 4, 10 and 11:** not re-examined. The proposal does not change schemas, templates,
  provenance or migrations; generation (group 11) enters only as build cost (F5).
- **RCA sections:** §1–§8 all apply. §5.3 (heuristics) and §5.4 (temporal) apply **only as
  reasons not to adopt** community, centrality or temporal backends now. §9 drives §9 of this
  review.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Current baseline** (DataFusion everywhere; whole-compiler lazy plan; pointer-identity reuse) | Nine relationship kinds with nine traversals; six memo predicates; a private query engine in P3; three expression representations | Reuse is sound but conservative; no cross-process reuse; stale blueprint §14.3; compile runs unbounded (F0) | About 13k lines of reuse machinery *(lead)*; 506k generated lines | M1–M9: engineering workflows over 900 s | Rejected: conflicts with the adopted standard and carries the measured cost |
| **Proposed**: RCA allocation + Salsa compiler core + petgraph projections + layered crates | One projection module; one reuse mechanism; one in-compiler expression IR; DataFusion only for set-oriented and batch work | New risks: Salsa identity pitfalls (B003, B018, B019/B021, B024), float equality for backdating, persistence caveats (B026, B036). Each has a named control (§3, §9). | A large migration touching every pass, but it **deletes** more than it adds: reuse machinery, `lookup.rs`, frontier loops, O(n²) sorts, most generated boilerplate | Build: F5 correction removes DataFusion from L0's cone (M8). Runtime: hypothesis until M9 is re-measured. | **Selected**, subject to §10 D1 |
| **Simpler viable alternative**: RCA allocation + petgraph projections + layered crates, **no incremental framework**. Recompute P0–P10 per request and delete the reuse machinery outright. | Same as proposed, minus Salsa. Reuse is simply absent (RCA §1 *Conditionality*: omit a boundary when avoided work is small). | Lowest: no memo, so no stale-reuse class at all. Loses the §14.4 promise that value edits reuse P0–P12 within a session. | Smallest: deletions plus the typed-Rust rewrite; no new framework | Today's behaviour already recomputes P3–P10 on every new process (F2), so for tests and batch runs this alternative **loses nothing measurable** | **Wins if** in-session incremental recompilation (interactive authoring, sweeps over structural parameters) is not a supported workload. It is also the first half of the proposed design, so choosing it does not foreclose Salsa. See §10 D1. |

**Abstractions justified by current needs.**
- **Kept:**
  - the registry and code generator (one declaration per relation, DM-52), but shrunk (F5);
  - the rule compiler (130 real rules);
  - DataFusion sessions for validation, rules, scenarios and results;
  - Delta for releases and runtime results.
- **Added:**
  - one typed projection module (§4.3);
  - the Salsa database, if D1 selects it.
- **Removed:**
  - the compile-path memo tables;
  - the P3 `Lookup` engine;
  - the durable-receipt reuse check, if D3 selects published artifacts plus the manifest instead.

**What remains ordinary code.**
- Hopcroft–Karp and Dulmage–Mendelsohn reachability: a specialised kernel of a few hundred lines,
  using petgraph visitors for the reachability.
- Template instantiation (`pse_templates::instantiate`).
- MathIR canonicalisation.
- The numeric tape and differentiation.

None of these needs a DSL, a DataFusion node or a Salsa query below its natural unit (RCA §4:
iterations stay inside the kernel).

### 8.1 Library capability map: what to leverage, what to leave

| Need in pse-arrow | Use | Specific API / feature | Evidence |
|---|---|---|---|
| Semantic-compilation reuse | salsa 0.28.4 | `#[salsa::input]` (documents, packages, case values) with `Durability::HIGH` for reference packages; `#[salsa::tracked]` functions per specialization, with `returns(ref)`, backdating and `lru = N`; `#[salsa::interned]` for qualified names and quantity-type keys (not for semantic IDs, whose identity is external); database clones for parallelism (`par_map` was removed after 0.24); `CancellationToken`; `heap_size` for memory reporting; the event stream (`WillExecute`, `DidValidateMemoizedValue`) as the test oracle | salsa skill: option matrix; probes B001–B041; obsolete-API table (`par_map`) |
| Cross-process warm start (optional) | salsa `persistence` **or** published artifacts | `persist` on tracked functions and structs (serde required); not accumulators (B026) | B025–B027, B034–B036 |
| Monotone recursive inference (P6 demand) | Rust worklist; Salsa `cycle_fn`/`cycle_initial` only if monotone and bounded | — | RCA §6; B010, B011, B024, B029 |
| Topology, containment, dependency DAGs | petgraph 0.8.3 | `StableGraph`/`Graph` (not `GraphMap` for arcs), `toposort`, `tarjan_scc`, `condensation`, `has_path_connecting`, `Dfs`/`Bfs`/`DfsPostOrder`, `visit::Reversed`, `dag_transitive_reduction_closure` only if a closure is truly needed | rust-graphs coverage and capability matrix (cargo-check proved) |
| Deterministic order, layers, ancestors | rustworkx-core 0.18.1 | `lexicographical_topological_sort`, `layers`, `ancestors`, `descendants`, `find_cycle`, `cycle_basis` | rust-graphs: zero-copy interop; C005 (hashbrown 0.17 sets); always-on rayon |
| Bipartite matching, DM, BTD | Own Hopcroft–Karp kernel plus petgraph SCC; rust-igraph only as a dev-only oracle (R-31) | — | rust-graphs coverage "matching"; context7 `pathfinding` |
| Rules, validation, joins, scenarios, results | DataFusion 55.1 | SQL rules; one batched join instead of per-key loops; `to_recursive_query` only where recursion is relational and bounded; prepared plans with parameter values instead of rebuilding per entity; `MemTable`/`DataFrame::cache` where a materialisation boundary is justified (RCA §7) | datafusion skill |
| Release reads, change sets, publication | Delta (dev commit) | Versioned `DeltaTableBuilder::with_version`; **CDF as the Salsa ingestion adapter**; application transactions for idempotent publication; `CommitProperties` metadata for the release manifest | deltalake skill `delta.cdf`, `delta.commit` (table-scoped transactions only, hence the manifest) |
| Sparse numerics (future Newton, homotopy, structural rank) | faer (context7: `SparseColMat::try_new_from_triplets`, `sp_lu`, symbolic analysis) | — | **Observation only:** not needed for this review's scope |
| **Not now** | leiden-rs, graphops, graphina, raphtory, rust-igraph (runtime) | — | No consumer. RCA §5.2 DEFAULT; RCA §5.3/§5.4 semantics do not arise. Register a trigger (§11 C9). |

## 9. Verification and measurement plan

| # | Claim or risk | Evidence label now | Test / analysis / benchmark | Conditions and expected result | Current result or gap |
|---|---|---|---|---|---|
| V1 | Layered crates remove DataFusion from L0 | Proposed | `tests/governance`: `cargo metadata` asserts no `arrow*`/`datafusion*`/`deltalake*`/`tokio` in the normal dependency closure of L0 crates | Holds for ids, diagnostics, quantity, material, mathir, templates, structural and the compiler core | Fails today (M8) |
| V2 | Build-time gain | Hypothesis (DM-39) | Repeat M1, M2 and M6 with the same flags and hardware | Cold wall time and the serial pse segment both drop; report with conditions | Baseline recorded (M1–M8) |
| V3 | Incremental equals clean (RCA §9) | Proposed | For each edit class — value edit, structural binding edit, template add/remove, failed-lookup-then-add, topology edge add/remove, rename — compare the Salsa result with a fresh database over the same inputs | Byte-equal published relations | `incremental_equals_clean_p0_p3` *(lead)* covers only P0–P3 on today's mechanism |
| V4 | Reuse actually happens | Proposed | Salsa event log: after a value edit, no `WillExecute` for structural queries; after a binding edit, only the affected specialization executes | Event sets equal the expected sets | — |
| V5 | No hidden inputs in tracked code | Proposed | salsa skill `project-` ast-grep rules plus a governance grep for clock, random, env and static reads under tracked functions | Zero hits | — |
| V6 | Query-per-entity eliminated | Proposed | `pse-testkit` capture counts DataFusion executions per pass on the engineering fixtures | O(1) executions per relation per pass, independent of instance count | Not measured |
| V7 | Graph semantics (RCA §9 graph additions) | Proposed | Fixtures: asymmetric direction, parallel arcs between the same ports, self-loops, isolates, `StableGraph` holes, deterministic ties, cycles with witnesses | Specified outputs | — |
| V8 | Model-compile runtime | Measured baseline (M9) | M9 re-run after F3 and F7 | Report per test with conditions | Baseline each of 5 tests > 900 s (terminated) |

**Cost accounting.** The material categories are:

- **Build:** M1–M8.
- **Model-compile time:** M9. This is dominated by cross-pass plan construction (F0), then
  per-entity query planning and execution (F3), then graph encode/decode per pass (F7).
- **Publication writes:** F8.
- **Salsa memo metadata:** RCA §8 notes that LRU does not bound metadata (salsa `memory` brief).
  Measure `heap_size` totals on the largest fixture before promising bounded retention.

## 10. Exceptions and unresolved decisions

**D1 — Is in-session incremental recompilation a supported workload?**
- **Principle IDs:** DM-33, DM-58; RCA §1 *Conditionality*.
- **Scope:** P0–P10.
- **Options:** Salsa (proposed), or the simpler alternative with no incremental framework (§8).
- **Evidence:** blueprint §14.4 promises value-edit reuse. No measured interactive workload
  exists, and today's reuse never survives a process (F2).
- **Owner:** paul-heyse.
- **Revisit trigger:** an interactive authoring or sweep workload with measured recompilation cost.
- **Recommendation:** do C0 (F0) immediately; it is independent of D1. Then execute the
  shared first half (F1 authority, F3, F4, F5, F7). Adopt Salsa in the same programme only if D1 is "yes"; otherwise delete the reuse machinery and leave
  R-01 open with that trigger.

**D2 — Resolve the authority conflicts T1, T2 and T4.** This needs an ADR, a design review and a
`design:` PR (F1). Until they land, reviews keep following REFERENCE §6's interim rule. This review
**recommends RCA's side** on all three, which is the maintainer's stated direction.

**D3 — Durable reuse route.** Choose between:
- Salsa `persistence` (serde on every persisted key and value; caveats B026, B036); or
- published compiled artifacts keyed by (release manifest, stable implementation version).

The second composes with Delta and needs no new serialisation surface. It is recommended unless
cold-start cost is measured to matter.

**D4 — Closure relations.** Does any consumer need all ancestor/descendant pairs
(`inferred.instance_reachability`)? If not, withdraw the relation and traverse on demand (RCA §6).

**D5 — Publication scope.** Which intermediate namespaces are inspection products, published only
on request, rather than always-written artifacts (F8)?

**SHOULD-level departures accepted with reasons:**
- Own Hopcroft–Karp kernel: no permissive library exists (F6).
- DataFusion kept for rule evaluation: set-oriented, 130 existing rules (RCA §5 DEFAULT satisfied,
  not departed from).

## 11. Decision and implementation changes

**Decision: Revise.** The current architecture is not unsafe; the gates found no stale-reuse or
consistency defect (G4–G7 pass or pass-scoped). But two things block acceptance.

**F0 is a violated RCA §6 MUST on the supported compile workflow.** Cross-pass lazy plan
composition expands shared structure. There is no resource bound, and the engineering workflows
do not complete (M9).

**G1, G2 and G3 are unresolved on in-scope behaviour:**

- the authoritative design contradicts the adopted computation standard;
- no graph projection is specified;
- the target moves validation boundaries without restating them.

F2–F5 are measured or line-cited cost and duplication defects that the standard names directly.
The target design above is **Accept-scoped** once C1 lands and D1 is decided.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| C0 (MUST; do first; valid under every §10 option) | Materialisation boundary at every pass output: passes consume materialised predecessor relations, never predecessor plans. Plan-construction budget with a typed refusal. Per-pass timing in the engineering tests. | RCA §6, §7 · DM-39, DM-30 | M9: all five engineering workflows complete, with per-pass times recorded | Governance: no cross-pass `LogicalPlan` inputs; a nextest timeout set from the new measurement |
| C1 (authority) | ADR plus review: per-operation mechanism allocation (RCA §1). Supersedes D10's allocation rule (ADR-0065 amendment), the reuse mechanism of ADR-0068/ADR-0074, and closes or retargets R-01/R-22. The `design:` PR updates D6, D10, D14, §1.3, §3.3.1–§3.3.3, §14.3–§14.4 and §15.3, and deletes stale §14.3 text. | DM-02, DM-23 · RCA §1–§2 | `adr-lint`; REFERENCE §6 T1/T2/T4 removed | Grep test: §14 names only existing mechanisms |
| C2 (graph semantics) | Typed projection module with a spec per relationship kind; `SemanticId ↔ NodeIndex` maps; cycle witnesses; derived rule strata | DM-34, DM-22, DM-47 · RCA §5, §5.1, §6 | V7 fixtures | Governance: no hand-rolled topological sort outside the module (ast-grep rule) |
| C3 (structural analysis) | `pse-structural`: incidence projection, own Hopcroft–Karp, petgraph SCC and condensation, rustworkx-core lexicographic order | DM-43 · RCA §5 MUST, §5.2 | scipy/Pyomo differential tests (§15.3); cycle-outside-ancestors fixture | CI parity job |
| C4 (build structure) | Crate layering L0–L4; diagnostics and ids off DataFusion; compiler core off catalog; generated-code shrink (generic `TypedBatch<R>`, shape-deduplicated payloads, `contracts.rs` → fingerprint, serde only on document DTOs) | DM-57, DM-58 | V1; V2 re-measured against M1–M8 | V1 dependency ceilings in governance |
| C5 (per-entity queries) | Decode once; traversals in Rust or petgraph; delete `p3/config/lookup.rs`; batch lookups as single joins | DM-38, DM-26 · RCA §5 | V6; M9 re-measured | V6 execution-count ceiling per pass |
| C6 (one math IR in compilation) | `ExprGraph` carried between P7–P10; relations emitted at publication; numeric tape from `ExprGraph` | DM-21, DM-37 · RCA §2 | Published `compiled.math_*` equality | Snapshot tests |
| C7 (reuse), if D1 = yes | Salsa database for P0–P10 with the §3/§5 controls. Delete the compile-path memo machinery in the same cut. The CDF ingestion adapter feeds inputs. | DM-32, DM-31 · RCA §1, §3, §4 | V3, V4, V5 | V3 in `just test`; salsa lint rules |
| C7′ (reuse), if D1 = no | Delete the compile-path memo machinery; recompute per request; record R-01 with its trigger | DM-58 · RCA §1 *Conditionality* | Tests unchanged in value; M9 | — |
| C8 (publication scope) | Publish the release, the problem bundle, diagnostics and results; intermediate relations on request (D5) | DM-58 · RCA §7, §8 | Member count per publication | Publication contract test |
| C9 (library hygiene) | Enable only the DataFusion features in use; the kernel TLS feature at the next delta-rs pin move; a register row for community, centrality and temporal backends with the trigger "a diagnostic or decomposition consumer exists" | DM-58 · RCA §5.2 | `cargo tree -e features` snapshot | Governance snapshot |

**Final check.**
- **Claims match the evidence.** Library claims are *Interface-checked* from the probe-backed
  skills; costs are *Measured* with conditions; the target is *Proposed*.
- **The scope is stated** (§1).
- **Extensions keep a data-only path** (§5). The Salsa decision is isolated as D1, so the first half
  of the migration is valid under either answer.
