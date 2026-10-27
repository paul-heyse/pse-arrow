# Mathematics, compilation and structural analysis

This page covers `pse-math` (Symbolica/Numerica integration), `pse-compiler` (Salsa preparation),
`pse-structural` and `pse-kernels`. Contracts are owned by
[mathematics and compilation](../../authoritative_design/sections/mathematics-and-compilation.md) and
[§15–§16](../../authoritative_design/sections/numerical-execution.md#section-15). The baseline is on the
[entry page](README.md).

**System picture.** Library use here is the strongest in the workspace.

- **Symbolica / Numerica.**
  - Supplies coefficient extraction, derivatives, `replace_multiple`, certified real-root
    isolation and refinement, univariate polynomials, evaluator optimisation and `HyperDual`
    jets.
  - `jets.rs` states that the code is "never a project differentiation engine".
  - Collocation nodes and weights come from Symbolica Jacobi roots (`collocation.rs`).
- **Numerical libraries.**
  - faer owns canonical CSC structure, sparse LU and the self-adjoint EVD.
  - `root1d::toms748` does scalar bracketing.
- **Graph and structure libraries.**
  - pounce-presolve does Hopcroft–Karp, Dulmage–Mendelsohn (DM), block-triangular form (BTF) and
    interval FBBT.
  - petgraph and rustworkx-core do SCC, topological order, greedy FAS and cycles.
  - Weighted tear selection is a HiGHS MILP (`pse-backend-native/src/tears.rs`).
- **Salsa.** Salsa is used with durability tiers, interned keys, per-query `lru` and
  `heap_size`, `Cancelled::catch` and event hooks.

The following are legitimately custom:
- original-residual verification after every accelerated root;
- sealed exact-rational evidence;
- identity hashing;
- FMA-free native compilation flags;
- guarded branch semantics;
- the provider envelope.

<a id="cbo-021"></a>

## CBO-021 — Bounded and cancellable Symbolica operations through version-coupled mirrors

- **Operation and consumers.**
  - Every Symbolica call must run under byte, operation and time budgets and honour
    cancellation: `derivative`, `replace`, `coefficient_list`, `vectorize`, `evaluator()`, root
    isolation and export.
  - Consumers are admission, class evidence, the cubic accelerator, native compilation and
    collocation.
- **Current mechanism.**
  - `library/admission.rs` carries Dualizer expansion cost formulas "from
    `Dualizer::map_instruction` in Symbolica 3.0.1".
  - `native_compiled.rs` (`export_bound`) bounds export source size from Symbolica's
    `export.rs`.
  - `class_evidence.rs` wraps each call in `allowance.charge(get_byte_size())` followed by
    `checkpoint`.
  - `library.rs` binds a thread-local `CURRENT_ABORT` because "Symbolica 3.0.1 stores the
    builder's abort callback in its evaluator settings" with no setter.
  - `implicit_cubic.rs` polls cancellation between root-isolation steps.
  - The pinned version is still 3.0.1, so these premises are current.
- **Why it matters.**
  - The cost models and abort plumbing depend on private behaviour of one release.
  - Any of the following would replace the mirror and close the drift risk:
    - a library budget or limit;
    - an abort hook;
    - a size estimator;
    - an upstream change.
- **Patterns.** [LOP-X001](../assessment-patterns.md#lop-x001) (primary), LOP-10, LOP-11.
- **Footprint.**
  - Four sites inspected.
  - Suspected through `typed*.rs`, `factorable.rs` and `assembly*.rs`.
- **Preservation constraints.**
  - Deterministic typed refusal (`MathError::Limit` / `WorkLimit`).
  - Charging before allocation.
  - Cancellation observed before results are retained.
- **Counterevidence.**
  - The models may be intentionally conservative.
  - Symbolica may offer nothing better yet.
- **Status.** Current promise (bounded construction).
- **Research questions.**
  - Does a later Symbolica release expose settings or abort setters, operation limits or size
    estimators?
  - Would upstream accept such hooks?

<a id="cbo-022"></a>

## CBO-022 — Prepared-mathematics reuse held in four layers beside Salsa

- **Operation and consumers.** Avoid repeating Symbolica evaluator preparation and native
  compilation across cases, rebinds and sessions. Consumers are compiler preparation, runtime
  retention and native evaluation.
- **Current mechanism.**
  - **Salsa first, then an out-of-Salsa body cache.** Salsa's `semantic_body` (`lru=64`) falls
    through to an out-of-Salsa `body_retention` cache keyed by identity
    (`executable/grouped.rs`, runtime `math/retention.rs`).
  - **Scalar cache.** An in-process `ScalarCache` is keyed by hashed atom lists (`admission.rs`,
    plus `reverse.rs`, `jets.rs` and `execution.rs`).
  - **Disk cache.** A disk cache for compiler output, `native_compiled/cache.rs` (496 lines),
    has its own entry, byte and scan limits under `$TMPDIR`.
  - **Manual maps beside Salsa inputs.** `state.sources` and `state.requests` maps sit beside the
    Salsa inputs (`modeling.rs`).
  - **Private Salsa API.** `granular.rs` reaches into `salsa::plumbing` (`AsId`, `ZalsaDatabase`)
    to label queries.
- **Why it matters.**
  - Invalidation lives in four mechanisms with four different keys: identity hash, atom hash,
    source hash and Salsa revision.
  - Use of the private `plumbing` API is an upgrade hazard.
- **Patterns.** LOP-07 (primary), LOP-10.
- **Footprint.** All four caches were inspected.
- **Preservation constraints.**
  - Identity follows the immutable dependency closure.
  - `semantic_identity() == input.identity`.
  - Compiled bytes carry headers and checksums.
  - Reuse across processes.
- **Counterevidence.**
  - Salsa memos do not outlive the process, so the disk layer serves a distinct lifetime.
  - Atoms are expensive to hash and compare.
  - The runtime retention layer was not read in depth.
- **Status.** Current promise.
- **Research questions.**
  - Is there a public Salsa API for query labels?
  - Could Salsa durability own the evaluator product for stable bodies?
  - Would a content-addressed cache crate (`cacache` or similar, fit unknown) carry the disk
    layer?

<a id="cbo-023"></a>

## CBO-023 — Structural decomposition and dependency closure owned in several places

- **Operation and consumers.** Bipartite matching, DM, BTF, block DAGs and transitive dependency
  closure. Consumers are structural analysis, initialization ordering, implicit reconstruction and
  executable dependency planning.
- **Current mechanism.**
  - `pse-structural/src/incidence.rs` runs the pounce sequence, then lexicographic topological
    sort. It also guards pounce recursion with a dedicated large-stack thread and
    row/stack limits.
  - `pse-math/src/implicit_reconstruction.rs` rebuilds an `EqualityIncidence` by hand. It runs
    `hopcroft_karp`, DM and BTF again without those guards, then walks the blocks to a
    transitive closure in a `BTreeSet` loop.
  - `initialization.rs` builds another block DAG with `kosaraju_scc`.
  - `pse-compiler` `executable/dependencies.rs` and `implicit.rs` build separate `DiGraph`
    reachability.
- **Why it matters.**
  - The same decomposition has several owners, each with its own limits.
  - The recursion guard exists in only one of them.
  - Transitive-closure loops duplicate what the graph libraries provide.
- **Patterns.** LOP-09 (primary), LOP-03.
- **Footprint.** Four to five graph builders inspected.
- **Preservation constraints.**
  - Tie-breaking by minimum `SemanticId`.
  - Deterministic order.
  - Conservative support across all branches.
  - Cancellation.
- **Counterevidence.**
  - The reconstruction path works on raw indices, by design, at a lower level.
  - The recursion bound is a property of pounce.
- **Status.** Current promise.
- **Research questions.**
  - Does pounce-presolve offer an iterative matcher, or a decomposition object that carries
    closure?
  - Do rustworkx-core or petgraph provide deterministic transitive closure or reduction (for
    example `dag_to_toposorted_adjacency_list` or `tred`)?

<a id="cbo-024"></a>

## CBO-024 — One expression lowered through several local IRs with a repeated operator vocabulary

- **Operation and consumers.** Solver-facing projections: global solvers, FBBT, cone recognition
  and class evidence.
- **Current mechanism.**
  - **Atoms to staged bodies.** Symbolica atoms become a staged `PreparedBody`/`Stage`
    (`guarded.rs`, `execution.rs`).
  - **Staged bodies to a factorable DAG.** That becomes a local `factorable::Node` DAG with its
    own interner (`factorable.rs`, about 3.9k lines).
  - **DAG to interval and FBBT views.** That becomes a `pounce_nlp::FbbtTape` (`presolve.rs`)
    and interval evaluation (`curvature.rs`).
  - **Class evidence re-reads the atoms.** `class_evidence.rs` re-reads the original atoms
    through `derivative` and `replace_multiple`.
  - **Operator vocabulary repeated per lowering.** exp, log, abs, pow, sin and cos are spelled
    out in each lowering.
- **Why it matters.**
  - Each lowering is a place for semantic drift.
  - A new operator touches every IR.
- **Patterns.** LOP-06 (primary), LOP-02, [LOP-X003](../assessment-patterns.md#lop-x003).
- **Footprint.** `factorable.rs`, `presolve.rs`, `curvature.rs` and `class_evidence.rs`.
- **Preservation constraints.**
  - `Exact`/`Relaxed` fidelity.
  - The obligation versus auxiliary distinction.
  - Identity of branches to `abs`.
  - A library-neutral global-solver vocabulary.
  - Cancellation.
- **Counterevidence.**
  - The DAG carries semantics that atoms lack: obligations, fidelity, provider calls and
    selects.
  - Each IR serves a different consumer.
- **Status.** Current promise.
- **Research questions.**
  - Could Symbolica's exported evaluator instruction stream serve as a common lowering source?
  - Could the pounce tape operator set be declared once?

<a id="cbo-025"></a>

## CBO-025 — Exact rational LDLᵀ and a cubic-refinement fallback ladder carried locally

- **Operation and consumers.** Convexity certificates and quadratic Gram checks need an exact
  decision. Cubic implicit roots need a refined interval.
- **Current mechanism.**
  - `convexity.rs` contains a hand-written sparse exact LDLᵀ over
    `Vec<BTreeMap<usize, Rational>>` with row-length pivoting, a budget and cancellation.
  - `implicit_cubic.rs` has a bespoke refinement ladder because the public refiner needs
    endpoints that are not roots.
- **Why it matters.** Generic exact linear algebra and refinement logic would be carried by a
  library if one fits.
- **Patterns.** LOP-01, LOP-11.
- **Footprint.** About 100–200 lines each.
- **Preservation constraints.**
  - The decision is exact.
  - The diagnostic reasons are kept (`Indefinite`, `GramFactors`).
- **Counterevidence.**
  - The kernels are small and specialized: symmetric pivoting and the 2×2 negative-direction
    witness.
  - This is a weak opportunity.
- **Status.** Current promise.
- **Research questions.**
  - Does `symbolica::domains::matrix` or a rational linear-algebra crate offer symmetric-pivoted
    exact LDLᵀ?
  - Does a later Symbolica release relax the refiner's endpoint requirement?
