---
title: Expand the petgraph skill into rust-graphs
status: done
date: 2026-09-22
adrs: []
phase: 0
---

# Expand the petgraph skill into rust-graphs

## Context

`.codex/skills/petgraph/` is an offline, reproducible reference for petgraph 0.8.3. Its generated
`content/` is built by a staged pipeline (`acquire` → `run_probes` → `build` → `verify`). Its
centrepiece is an algorithm × container compatibility matrix, and `cargo check` re-proves every
cell of it. The skill answers "will this compile, and will the answer be right?"; it does not
answer "does this exist?".

The skill should cover **graph analytics in Rust**, not only petgraph. It gains six libraries:
rustworkx-core, leiden-rs, graphops, graphina, rust-igraph and raphtory. The goal is that an
agent coding a graph task can find the right capability, in the right library, together with the
cost of reaching it from its own graph. That cost includes type and version compatibility,
licence, features and determinism.

Status of this document: *Proposed*. The library facts below are *Interface-checked* (crates.io,
docs.rs and library-enrichment, 2026-09-22); none are *Tested* yet.

## Decisions

Maintainer decisions, 2026-09-22:

- **Rename** `.codex/skills/petgraph` to `.codex/skills/rust-graphs`. The directory is untracked and
  nothing outside it refers to the name, so no compatibility alias is kept.
- **Uniform full depth.** Every library gets the treatment petgraph has: two captures, a full
  corpus, behaviour probes with controls, a complexity index, release history and retained licences.
- **petgraph is the hub.** Every other library is reached through an explicit seam, and
  each seam is proved by compile probes.
- **The skill is not a dependency.** The pinned crates never enter the workspace `Cargo.toml`
  or `Cargo.lock`. Probe crates build in temporary directories. No ADR is required: this is
  tooling (see AGENTS.md, "When an ADR is required").

## Plan

### Pinned subjects and the facts that shape the design

| Library | Pin | Licence | Graph model → how a petgraph 0.8.3 graph gets in | Hazard to encode |
|---|---|---|---|---|
| petgraph | `0.8.3` | MIT/Apache | the hub | (existing) |
| rustworkx-core | `0.18.1` | Apache-2.0 | **generic over petgraph `visit` traits; no copy** (re-exports `rustworkx_core::petgraph`, `petgraph ^0.8`) | no features, so rayon and ndarray are always on; `ContractNodesDirected` exists only on `StableGraph`/`GraphMap`; `max_weight_matching` returns a HashSet; `parallel_threshold` arguments |
| leiden-rs | `0.8.1` | MIT/Apache | own CSR `GraphData`; `convert::from_petgraph` takes **`Graph` only**, with `E: Into<f64>`, behind the `petgraph` feature | docs.rs builds only the default features (`cli`, `rayon`, `gryf`), so `from_petgraph` is **not on docs.rs**; recommend `default-features=false` |
| graphops | `0.5.1` | MIT/Apache | its own adapter traits `Graph`, `WeightedGraph`, `GraphRef`, `WeightedGraphRef`; the optional adapter targets **petgraph 0.6** | a second petgraph in the build; the functions named `leiden*` are actually connectivity-refined Louvain; walks only, with no skip-gram trainer |
| graphina | `=0.4.0-alpha.6` | MIT/Apache | `BaseGraph` wraps a private `StableGraph`; `as_petgraph()` is read-only and there is no conversion back | every algorithm module is behind a feature and **no feature is on by default**; it forces the petgraph features `graphmap`, `stable_graph`, `matrix_graph`, `serde-1` and `rayon` on through feature unification; it has exact `=` pins (nalgebra, bincode, rand) |
| rust-igraph | `0.7.0` | **GPL-2.0-or-later** | its own `Graph` with `u32` ids and a runtime directedness flag, and weights as `&[f64]`; the graph must be copied in | deleting vertices renumbers them; the graph is `!Sync`; there is no parallelism; the project is very young |
| raphtory | `0.17.0` (crates.io; GitHub is at 0.18.5) | **GPL-3.0** | temporal multigraph with `GID` node ids, layers and windows; the graph must be copied in or loaded from Parquet | always pulls in **arrow 56 / datafusion 50**, which conflicts with a consumer on another Arrow major (a one-type-universe hazard); always compiles in `axum`; MSRV 1.91; only 19.6% of items documented; the `python` feature brings in pyo3 0.25; `label_propagation`'s `_seed` argument looks unused |

Repository facts from the exploration pass:
- The skill directory is untracked (about 44 MB), so renaming it loses no history.
- Nothing outside the directory refers to it. `.claude/skills` and `.agents/skills` are symlinks to
  `.codex/skills`, and `just lint-agents` does not check skill frontmatter.
- `REUSE.toml` gives all of `.codex/**` the licence MIT OR Apache-2.0, which would mislabel the GPL corpora.

### Architecture: petgraph is the hub, and each library is reached through an explicit seam

Five reader-facing layers are added, alongside the existing eight:

1. **Library choice**: `content/libraries/00-map.md` plus one page per library. Like the
   container pages, each library page's central section is *what it cannot do* and *what it costs
   you to use it*: how to get data in, the licence, features, a version conflict, determinism, and
   maturity.
2. **Coverage matrix**: `content/index/coverage.tsv` plus `content/catalogs/coverage.md`. It is
   keyed on capability family × library, with columns `family · capability · library · access_path
   · input_model · weighted · directed · parallel · seeded · verdict`. This answers "who can do
   betweenness, Leiden, node2vec or temporal motifs at all, and in what form". The rows are
   generated from the captures together with an authored family map (`authoring/families.tsv`);
   they are never written by hand.
3. **Interop matrix**: `content/index/interop.tsv`. It is keyed on library × petgraph container,
   with verdicts `zero-copy | convert-fn | adapter | copy | version-mismatch | n/a`. Each non-`n/a`
   verdict is proved by a compile probe with a control. For example, graphops' `petgraph` feature
   does **not** accept a petgraph 0.8 `Graph`, and the newtype adapter does. leiden-rs
   `from_petgraph` accepts `Graph` and rejects `StableGraph`.
4. **The existing compatibility matrix gains a `library` column.** rustworkx-core's generic
   functions go into the same algorithm × container matrix, because their bounds are petgraph
   `visit` traits. rustc re-proves those cells exactly as it does petgraph's, which is where
   "seamless" is actually demonstrated. The matrix grows from about 43 to about 140 generic
   algorithms.
5. **Seams**: the existing `content/seams/` gains five pages:
   - `petgraph-version` (graphops 0.6; how a second petgraph shows up as E0308/E0277)
   - `arrow-universe` (raphtory's arrow 56 / datafusion 50; exchange data through Parquet or IPC
     bytes, or keep raphtory in a separate process)
   - `licences` (GPL-2+ for rust-igraph, GPL-3 for raphtory; what linking implies; no refusal,
     only a statement)
   - `features` (graphina's modules are all off by default and it forces petgraph features on;
     leiden-rs's docs.rs gap; rustworkx-core's rayon cannot be switched off)
   - `determinism` (seed conventions per library; hash-ordered returns; rayon thread counts;
     defaults of seed 0 versus `None`)

The reading order in `SKILL.md` becomes: **task → capability family → library ladder → container /
interop → contract**. The library ladder is authored judgment and is stated once, in
`authoring/libraries.json`:
1. petgraph
2. rustworkx-core (same graph, no copy)
3. leiden-rs (communities)
4. graphops (walks, similarity and kernels, through an adapter)
5. graphina (link prediction and approximation; alpha)
6. rust-igraph (the broadest library; GPL; requires a copy)
7. raphtory (temporal graphs; GPL; separate Arrow universe)

### Decision briefs (`authoring/capabilities/*.json`)

**Update the 12 existing briefs** so that their `choices` include cross-library candidates:
- shortest-path: rustworkx `all_shortest_paths`, `distance_matrix`, `negative_cycle_finder`
- connectivity: `core_number`, `chain_decomposition`, igraph's vertex and edge connectivity
- isomorphism: igraph's LAD, BLISS and automorphisms
- flow: igraph's Gomory–Hu tree and `all_st_mincuts`; rustworkx's Stoer–Wagner
- spanning-tree: graphina's Borůvka
- acyclic: rustworkx's `lexicographical_topological_sort`, `layers` and `longest_path`
- traversal: rustworkx's event visitors

**Add about 14 briefs:**
- `graph.library-choice` (the meta brief: which library, and what reaching it costs)
- `graph.interop` (getting a petgraph graph into each library and results back)
- `graph.centrality` (betweenness, closeness, eigenvector, Katz, HITS, PageRank; normalization
  conventions differ between libraries)
- `graph.community` (Leiden with CPM or modularity versus Louvain versus Infomap versus label
  propagation; resolution; seeds)
- `graph.partition-quality` (modularity, NMI/ARI/VI, conductance)
- `graph.embedding` (node2vec walks versus FastRP versus spectral embedding)
- `graph.similarity-link-prediction`
- `graph.matching` (split out from flow; covers Gabow, weighted blossom, igraph bipartite and
  Hungarian `lsap`)
- `graph.coloring` (DSatur, greedy with strategies, edge colouring, Misra–Gries)
- `graph.cycles` (cycle basis, Johnson simple cycles, feedback arc and vertex sets)
- `graph.cuts` (Stoer–Wagner, Gomory–Hu, disjoint paths)
- `graph.dag-analysis`
- `graph.generators`
- `graph.layout`
- `graph.temporal` (windows, rolling views, temporal reachability, motifs)

Topic axes grow from 10 to about 15: centrality, communities, embeddings and similarity,
temporal, layout and generators. `router.json` gains questions for each new area, and each
question names its wrong answer. Examples: "graphops `leiden` for Leiden", "raphtory in-process
beside arrow 59", "igraph centrality numbers compared directly with rustworkx without
normalization".

### Behaviour probes (each paired with a control that must come out the other way)

Probes that compare libraries against each other on the same fixture (karate club, and a small
DAG with ties):
- **Betweenness** from rustworkx, graphina, rust-igraph and graphops (through the adapter):
  records the normalization and endpoint conventions. Possible silent divergence.
- **Modularity of one fixed partition** from leiden-rs, graphops, graphina, igraph and raphtory:
  should agree to a tolerance, or the difference gets recorded.
- **graphops `leiden*` versus leiden-rs `Leiden`**: the partitions or connectivity guarantees
  differ.
- **Seed reproducibility**: the same seed gives the same partition for leiden-rs, graphops,
  graphina and igraph. raphtory `label_propagation` with two different seeds is expected to give
  the *same* result, which demonstrates that the seed is ignored.
- rustworkx `lexicographical_topological_sort`: tie-breaking by key, compared with petgraph
  `toposort`.
- rustworkx `max_weight_matching`: the result is a set with no order.
- igraph `delete_vertices`: renumbering (the control is `delete_vertices_map`).
- raphtory `window`: a view excludes out-of-window edges (the control is the unwindowed view).

Interop compile probes supply every `interop.tsv` cell. A **petgraph unification probe** runs
`cargo metadata` in each probe crate and asserts exactly one `petgraph` package resolved, except
for the graphops-feature probe, which must show two.

Probe workspaces are split so that each library is proved on its own:
- `probe-core`: petgraph, rustworkx-core, leiden-rs, graphina, graphops and rust-igraph
- `probe-raphtory`: isolated because of arrow 56, `axum` and MSRV 1.91, and tagged slow

### Pipeline generalisation (`build/`)

**Manifests.** `build/manifests/rust-graphs.json` is the umbrella: tools, toolchains, format
allowances and library order. Each library gets its own file, `build/manifests/libraries/<lib>.json`,
which moves petgraph's current manifest together with its comments. A per-library entry carries:

```
package, lib, version, role (hub | petgraph-generic | own-model | temporal), capture
[docs_rs, all_features], capture_features, corpora, integration_model, compatibility, boundary
```

`capture_features` exists because `--all-features` is not always wanted:
- raphtory is captured with `arrow,io,proto,search,storage` and without `python` or `vectors`.
  That profile is recorded as a boundary in `unresolved.tsv`.
- leiden-rs and graphina use `--all-features`.

**Code changes**, following the pattern `datafusion/build/build.py` already uses (loop over
`(package, version)` pairs):
- **Single-subject code to generalise:**
  - `build.py`: `build_models`, `write_features`, `write_coverage`, `write_feature_visibility`,
    `main`'s licence list and PROVENANCE `subject`, which becomes `libraries[]`.
  - `acquire.py`: `MANIFEST`, `capsule_root` (whose environment variable becomes
    `RUST_GRAPHS_SKILL_CAPSULE`), and `acquire_release_history`, which runs once per library.
  - `scripts/run_probes.py` and `verify.py` (manifest paths), `fetch.USER_AGENT`, and
    `capabilities.write`'s dependency list.
- **Cross-crate trait resolution in `surface.py`.** It already resolves the lattice and its
  blanket `&G` forwarding. It also has to resolve rustworkx-core's bounds, which name petgraph
  traits through `external_crates` / `rustworkx_core::petgraph`, onto the *same* lattice. It must
  also add foreign trait impls on petgraph containers, such as `ContractNodesDirected` on
  `StableGraph`. `load()` stops hard-coding `package="petgraph"`.
- **`algorithms.py`:**
  - collect from every library whose role is `petgraph-generic`, not only from the
    `petgraph::algo::` prefix
  - add a `library` column to `algorithms.tsv` and `capability-matrix.tsv`
  - for non-generic libraries, collect the public free functions and methods into a per-library
    `functions.tsv`
- **`matrix.py` and `probes.py`.** The per-library `[dependencies]` generation replaces the
  hard-coded `petgraph = "=0.8.3"`. A new `interop.py` builds `interop.tsv` together with its
  probe fixtures.
- **Complexity** becomes a per-library extractor:
  - petgraph keeps its template.
  - Other libraries' documented `O(...)` figures come from their docstrings with source
    `docstring`.
  - A library with no figure is marked `unstated`, never guessed.
- **New `coverage.py`** joins `authoring/families.tsv` (a capability family mapped to name
  patterns per library) with the symbol indexes to produce `coverage.tsv` and its catalog page.
  `verify.py` then refuses a family row that matches no symbol.
- **`integration.py`.** The `BOUNDARY` hard-coded dict is replaced by manifest data. New boundary
  crates are added only where their types leak into caller signatures:
  - `ndarray` (rustworkx `distance_matrix`)
  - `foldhash`/`indexmap` (`DictMap`)
  - `rand` (seed and `Rng` parameters, where exposed)
  - `sprs` and `nalgebra` (graphina spectral)
  - raphtory's arrow 56 types are listed as a boundary with a reason; they are not indexed
- **`queries.py` and `queries/`.** Each library gets an outline rule. New `project-` hint rules:
  graphops `petgraph` feature in a manifest; raphtory beside another Arrow major; `leiden-rs`
  default features; graphina used without features.
- **`licenses.py`** is already generic. `NOTICE.md` is rewritten per library. A
  **`REUSE.toml` override** annotates `.codex/skills/rust-graphs/content/corpus/{rust-igraph,raphtory}/**`
  and their licence directories with their real GPL licences.
- **Corpora** come from the **`.crate` archive of each pinned version**. That keeps one rule for
  every library: leiden-rs lives on GitCode, and raphtory's GitHub tag is ahead of crates.io.
  Upstream tests from git are added only where a tag matches the pin.
- **`verify.py` gains three checks:**
  - `interop`: set equality in both directions against the compile probes
  - `unification`: petgraph resolves to one version per probe crate
  - `coverage`: every family row resolves

  `counts.json` is reconciled last.
- **`reference.py`:**
  - `find --library`
  - `matrix --library`
  - a new `coverage --family <f> [--library]`
  - a new `interop --library <l> [--container]`
  - a new `libraries` summary
  - the `--container` help text is generated, not hard-coded
  - `canonical()` resolves paths across crates

**Transferability stays unchanged.** The skill contains nothing pse-arrow-specific, and briefs
stay generic. The structural-analysis needs this repository has (bipartite matching, SCC/BTD,
minimum feedback arc set, deterministic topological order) are served through the generic
`graph.matching`, `graph.cycles` and `graph.dag-analysis` briefs.

### Execution phases (each ends with `python3 build/verify.py` green)

0. **Record the plan and rename.** Copy this plan to `docs/plans/` with `just plan rust-graphs-skill`.
   Then:
   - `mv .codex/skills/petgraph .codex/skills/rust-graphs`
   - set frontmatter `name: rust-graphs`, with a description that names all seven libraries and
     the task areas
   - move the manifest to the umbrella and per-library layout
   - rename the environment variable and user agent
   - `just agent-config-sync` and `just lint-agents`

   **Gate:** a rebuild with petgraph as the only library reproduces today's `content/`
   byte-for-byte, apart from the intended rename strings and the new `library` column.
1. **Generalise the pipeline to N libraries**, still with only petgraph. Includes manifests,
   `surface.load`, the `library` columns, the reader CLI flags and `interop.py`/`coverage.py`
   running on empty inputs.
2. **rustworkx-core.** Two captures, the corpus, cross-crate lattice resolution, the extended
   matrix and its compile probes, and the centrality, coloring, dag-analysis, cycles and cuts
   briefs.
3. **leiden-rs and graphops.** Adapter lattice, interop probes (including the petgraph 0.6
   mismatch), the community, partition-quality, embedding and similarity briefs, and the
   modularity and seed probes.
4. **graphina and rust-igraph.** Feature maps, the copy interop probes, igraph renumbering, the
   GPL seam and REUSE override, and the layout, generator and matching briefs.
5. **raphtory.** Isolated probe workspace, captures with `capture_features`, the arrow-universe
   seam, the temporal brief and window probes.
6. **Reader surface.**
   - rewrite `SKILL.md` (the library ladder first, then the existing "will it compile" material)
   - update `reference.md` (the pinned table, index schemas and limits: per-library capture
     formats; what the coverage matrix does *not* claim, which is equivalence of results across
     libraries beyond the probed fixtures)
   - update `maintenance.md` ("adding a library" becomes a manifest recipe)
   - update `build/README.md`, the topic axes, `router.json` and `counts.json`

### Critical files

- `.codex/skills/petgraph/` → `.codex/skills/rust-graphs/`: `SKILL.md`, `reference.md`,
  `maintenance.md`, `NOTICE.md`
- `build/{acquire,build,surface,algorithms,matrix,probes,integration,featuremap,capabilities,router,queries,verify,fetch}.py`
- New modules: `build/{interop,coverage}.py`
- `build/manifests/*`, `build/router.json`, `build/probes_behaviour.json`,
  `build/fixtures/tree.json`, `build/counts.json`
- `authoring/{capabilities/*.json,crate-roles.tsv,contract-notes.json,libraries.json,families.tsv}`
- `scripts/{reference,run_probes,verify_bundle,qualify_transfer}.py`, `build/test_contracts.py`
- `REUSE.toml` (at the repository root; a GPL path override)
- A model to reuse: `datafusion/build/build.py` for the multi-crate pair loop and the
  `crate-roles.tsv` routing rule

## Verification

Evidence at completion, 2026-09-23. Every item below is *Tested*: each names the command that ran
and its outcome. The baseline is zero.

- `python3 scripts/run_probes.py` (stable 1.98.1, single-threaded tests): receipt `passed`.
  That covers 7 matrix probes, all confirmed (938 cells, 205 blocked); 7 interop routes over
  49 compiled cells, all agreeing in both directions; 5 compile probe/control pairs confirmed;
  15 behaviour probe/control pairs confirmed; and unification `U-core` and `U-raphtory`, each
  resolving exactly one petgraph 0.8.3.
- `python3 build/verify.py`: all 14 checks passed. They are determinism (a byte-for-byte
  rebuild), integrity, counts, recipes, router, probes, unification, coverage, contracts,
  rule_tests (17 passed, 0 failed), oracles, transferability, matrix (rustc re-run) and
  interop (rustc re-run).
- `python3 scripts/package.py reader` followed by `scripts/qualify_transfer.py` under strace:
  `passed`, with 0 network syscalls and 0 accesses outside the bundle. It exercised 9 reader
  lookups from `/`, including `libraries`, `coverage`, `interop` and
  `find --library leiden-rs`.
- `just lint-agents` and `just setup-test` (47 tests): passed.
- `just quality`: every step this work touched passes, including `lint-toml` after formatting
  `typos.toml`. One step, `fmt-py-check`, fails on `scripts/tests/test_setup.py`, which the
  maintainer was changing at the same time as the deny-rule fix; this work did not touch that
  file.

- `python3 build/acquire.py && python3 scripts/run_probes.py && python3 build/build.py && python3 build/verify.py`
  must report all checks passing. That includes determinism (a byte-for-byte rebuild), matrix and
  interop set equality, unification, coverage, counts and recipes. A `divergent` probe verdict is
  a failure.
- `python3 scripts/package.py reader --output <scratch>/rg.tar.gz`, then `verify_bundle.py` in
  the extracted tree: the reader runs sandboxed with no network, no subprocess and no file access
  outside the bundle.
- Spot-check that the reader routes correctly:
  - `reference.py find --task 'leiden community detection'` goes to `graph.community`, with
    leiden-rs first
  - `reference.py interop --library graphops` shows `version-mismatch` for the feature and
    `adapter` for the newtype
  - `reference.py matrix --library rustworkx-core --container StableGraph --blocked` shows the
    blocked cells
  - `reference.py coverage --family centrality` shows the betweenness providers and their
    normalization notes
- Repository gates: `just lint-agents`, `just setup-test` and `just quality`. REUSE runs inside
  `quality`, so the GPL override must pass `reuse lint`.

## Open items

- **REUSE annotation for GPL corpora.** `REUSE.toml` gives all of `.codex/**` the licence MIT OR Apache-2.0. The
  rust-igraph (GPL-2.0-or-later) and raphtory (GPL-3.0) corpora need a path override. This is a repo-root
  configuration change that has to pass `reuse lint`.
- **Corpus size.** The skill is about 44 MB with one library. The raphtory and rust-igraph corpora are large.
  Measure the size after phase 5 and decide whether the tests-from-git additions stay.
- **raphtory release lag.** crates.io carries 0.17.0 while GitHub is tagged 0.18.5. The pin follows crates.io.
  Revisit when `raphtory` 0.18 is published.
- **graphina is alpha** (`=0.4.0-alpha.6`). Treat every alpha bump as a re-acquisition that re-runs the probes.

## Outcome (recorded after implementation)

### What was built

`.codex/skills/rust-graphs/` is a local-only library skill (gitignored under the rule that
landed while this plan ran).

- *Implemented*: seven libraries are pinned and captured twice, from docs.rs and locally:
  petgraph 0.8.3, rustworkx-core 0.18.1, leiden-rs 0.8.1, graphops 0.5.1,
  graphina 0.4.0-alpha.6, rust-igraph 0.7.0 and raphtory 0.17.0. There are 1104 algorithm
  functions and 20557 operation contracts, and every library's source is in the corpus.
- *Implemented*: reader-facing layers.
  - A library ladder, with one page per library that leads with what it *cannot* do and what
    reaching it *costs*.
  - A coverage matrix of 247 rows across 21 families, generated from `authoring/families.tsv`.
  - Five new seams: two petgraphs, two Arrows, licences, features and determinism.
  - 27 briefs (15 new, 9 widened), 16 topic axes and 28 routed questions.
  - `reference.py` gains `libraries`, `coverage` and `interop`, and `--library` on `find` and
    `matrix`.
- *Tested* (`scripts/run_probes.py`, `build/verify.py` checks `matrix` and `interop`):
  - The capability matrix now also covers rustworkx-core's 91 generic functions: 938 cells,
    205 blocked, each re-proved by `cargo check`. Its `call` column gives the spelling that
    compiles: `&g`, `&&g` or `type`.
  - The interop matrix's 49 compiled cells agree with the lattice derivation in both directions.
- *Tested* (`scripts/run_probes.py`, behaviour and compile families, each paired with a
  control):
  - graphops's `leiden` returns its `louvain` partition (B010).
  - graphina's betweenness ignores the weights it requires (B008).
  - raphtory's `label_propagation` ignores its seed (B015).
  - rust-igraph renumbers vertices on delete (B013).
  - `from_petgraph` reads edge weights (B009).
  - Betweenness and modularity agree across the libraries that implement them (B007, B009).
  - raphtory 0.17.0 does not compile as published (C001).
  - graphina has no algorithms by default (C002).
  - leiden-rs's adapter needs its feature (C003).
  - graphops's petgraph feature is petgraph 0.6 (C004).
  - rustworkx-core returns hashbrown 0.17 sets (C005).
- *Tested* (matrix probe `M-Graph`): petgraph's `subgraph_isomorphisms_iter` compiles only as
  `&&g`.
- *Interface-checked* against the shipped source and its documentation: no library here
  computes an exact minimum directed feedback arc set. rust-igraph's `FasAlgorithm` has the
  single variant `EadesLinSmyth`, and petgraph's greedy heuristic documents that it ignores
  weights. This bears directly on blueprint §12.5's `mip` tear method.
- *Implemented*: four project hint rules and one outline for the new corpora. *Tested*: rule
  tests report 17 passed, 0 failed.

### A mistake made and corrected

- **The Phase 0 baseline was lost.** Phase 0's gate was a byte-for-byte comparison of petgraph's
  generated content before and after the rename. The pipeline was rewritten before the old
  `content/` tree was copied aside (it was untracked), so that comparison could no longer be
  made. It was replaced by three checks:
  - every registered petgraph number is equal: 46 algorithms, 43 generic, 301 cells,
    48 blocked, per-container blocks of 9/11/14/14, 36 documented complexities, 13 unpublished
    names and 6 non-re-exported paths;
  - all seven matrix probes are confirmed against the same cells;
  - the petgraph local capture has identical `index` and `paths`.

  The acquired captures *were* backed up.
- **Two model errors, each caught by rustc and fixed:**
  - graphops's own `GraphRef` trait was matched by leaf name against petgraph's, which
    classified 43 adapter-generic functions as petgraph-generic.
  - rustworkx-core's `line_graph` had its two graph parameters' bounds unioned.
- **A draft brief claimed an exact feedback-arc-set method in rust-igraph.** The shipped source
  (`FasAlgorithm` has one variant, and the parameter is spelled `_algo`) refuted it before the
  brief was published.

### Deviations from the plan, deliberate

- **The coverage index is named `coverage-matrix.tsv`,** because `coverage.tsv` already means
  per-capture feature coverage.
- **No `REUSE.toml` override.** Library skills became gitignored during this work, so REUSE never
  sees the GPL corpora. Their licences are retained in the skill instead. raphtory's archive
  ships no licence file, so its GPL-3.0 text is fetched from tag `v0.17.0`.
- **The manifest-level hint rules became compile probes C001–C004.** Those are the rules for
  graphops's feature, raphtory beside another Arrow, leiden-rs's defaults and graphina's empty
  defaults. ast-grep in this setup has no TOML grammar. Source-level hint rules were written
  instead for the silent failures B008, B010, B013 and B015.
- **Probe workspaces: four, not two** (`matrix`, `core`, `graphops-petgraph`, `raphtory`).
  Proving the petgraph 0.6 mismatch needs its own lockfile.
- **Behaviour probes follow what measuring found,** not the plan's list:
  - "max_weight_matching returns an unordered set" became C005, the hashbrown type.
  - graphina's ignored weights (B008) was added.
  - Seed behaviour was probed for leiden-rs, graphops and raphtory. For graphina, rust-igraph
    and rustworkx-core the determinism seam marks it "not probed".
- **Corpora come from each `.crate` archive only.** No git test trees were added for the six
  new libraries.
- **petgraph content changed deliberately in four places:**
  - the corpus moved to `corpus/petgraph/`;
  - the docs.rs-built feature count now includes the defaults docs.rs builds (3 → 8);
  - item-level feature gates are now read, so `parallel_johnson` is `default-off`;
  - the matrix gained `library` and `call` columns.
