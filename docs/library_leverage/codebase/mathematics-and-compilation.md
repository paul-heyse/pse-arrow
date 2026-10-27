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
- **Library assessment correction (2026-10-27).** Symbolica's abort truncates optimisation rather than failing: `build()` can return `Ok` with a less-optimised evaluator. The local rule that an observed abort means cancelled is therefore required. Since 3.0.1, upstream `main` has rewritten `dual.rs` and `export.rs`, so the mirrored formulas will need re-deriving at the next release. The repository moved to `symbolica-dev/symbolica`, and its unreleased `community` branch adds a thread-local `ResourceBudget` measured by RSS and wall time. That budget is not prepaid or deterministic. See [symbolica](../libraries/symbolica/README.md).
- **Comparative review correction (2026-10-27).** The mirrors are predict-then-reconcile: `bounded_evaluator` refuses with `Contract("library Taylor expansion exceeded admitted bound")` after `vectorize`, and the compiled path refuses when `export_cpp_str` exceeds `export_bound`. Drift therefore fails loudly; the residual exposure is the transient allocation inside the call. There is no `symbolica` entry in `[workspace.metadata.pse.pins]`. See the [comparative review](../reviews/mathematics-preparation.md#cbo-021).

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
- **Library assessment correction (2026-10-27).** Salsa's `heap_size` closures return the local `retained_bytes()`, so Salsa reports the local measure rather than running a second accounting system. The private-API footprint is four `salsa::plumbing` uses in `granular.rs` and one in `workspace.rs`; only the query-name half has a public replacement (`ingredient_debug_name`). See [salsa](../libraries/salsa/README.md) and [cacache](../libraries/cacache/README.md).
- **Comparative review correction (2026-10-27).** The four mechanisms are four lifetimes, each with its natural key, not parallel caches. `ScalarCache` is construction-scoped and frozen into immutable `ScalarProducts`, which is product composition. `ModelingBodyRetention` is the runtime cross-workspace cache that §14.4 requires, with an identity and physical check on every hit (`executable/grouped.rs` `semantic_body`). The disk layer is ADR-0212 (proposed; implemented and tested 2026-10-20). `state.sources`/`requests` are Salsa's input-handle registry. The `workspace.rs` plumbing use is `current_revision` (retention snapshot), which Salsa 0.28.5 exports only under `salsa::plumbing`. See the [comparative review](../reviews/mathematics-preparation.md#cbo-022).

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
- **Library assessment correction (2026-10-27).** `MathPolicy::validate` already requires math workers to have at least `MATCHING_STACK`, so the recursion half of the "missing guard" may already be covered; which thread runs `prepare_contract_inner` was not traced. The compiler already uses petgraph `Bfs`, reversed `Dfs` and `toposort`, so the duplication lies in graph construction and limits. The vendored SuiteSparse already ships BTF, CXSparse and SPEX. See [suitesparse](../libraries/suitesparse/README.md) and [pounce](../libraries/pounce/README.md).
- **Comparative review correction (2026-10-27).** The open question is answered: both guards hold transitively. The only production caller is `pse-runtime/src/math/solves/derived.rs` (`prepare_reduced_supplier` in `job_retained_scoped`); the `pse-backend-native` call sites are tests. Math jobs spawn with `.stack_size(service.policy.stack_bytes)` (`math/jobs.rs`), `MathPolicy::validate` refuses a stack below `MATCHING_STACK`, and `CaseLimits` caps a case at 100,000 scalars. The real gap is that implicit reconstruction runs no result validation, so it would mis-propagate silently if block order were wrong. See the [comparative review](../reviews/mathematics-preparation.md#cbo-023).

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
- **Comparative review correction (2026-10-27).** The premise is rejected: there is one project-owned factorable projection. Curvature consumes the factorable DAG directly, FBBT uses POUNCE's own format, and class evidence is atom-level by design; each consumer's per-operator rules are its own semantics, forced by exhaustive matches. The LOP-X003 citation does not fit, because no generator is involved. Side observation, not traced to a conclusion: `presolve.rs::tape` lowers constants through `Constant::value()`, which rounds to nearest rather than outward. §15.4's tolerance-expanded confirmation appears to cover infeasibility claims; whether tightened bounds are also confirmed was not checked. See the [comparative review](../reviews/mathematics-preparation.md#cbo-024).

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
- **Library assessment correction (2026-10-27).** The "non-root endpoints" rationale is directionally right but understates the hazard. A probe of Symbolica 3.0.1 `refine_root_interval` ([evidence](../../design_review/evidence/library-leverage-symbolica-root-refinement-2026-10-27/README.md)) found:
- A left-endpoint root panics in debug and in release returns a narrow interval with no root and no error.
- A right-endpoint root panics in debug when p(a) > 0.
- An interval symmetric about 0 panics with "Division by 0" in debug and release.
- An asymmetric interval straddling a root at 0 did not return within 20 s.

`isolate_real_root_intervals` can itself emit intervals whose endpoint is an adjacent exact root. The local p(a) = 0 / p(b) = 0 screen is therefore a correctness guard, not a precision one. The zero-symmetric and zero-straddling cases are not covered by that screen. Whether the isolator can emit them is unsettled, and this is worth a product-side check independent of library leverage. See [symbolica](../libraries/symbolica/README.md).

- **Comparative review correction (2026-10-27).** The open isolator question is settled, and the live hazard is a different one. `isolate_real_root_intervals` confines each interval to one closed half-line, so zero-symmetric and zero-straddling intervals are unreachable on the isolate→refine path. For repeated-root cubics, however, an isolating interval can strictly contain another root, which `refine_root_interval_until_disjoint` does not detect and the p(a)/p(b) screen does not catch. For (3x − 8)(2x − 3)² the ladder panics in debug, and in release returns `[1.5, 1.5]`: 8/3 is lost and a valid unique root is falsely refused. Over (αx − β)²(γx − δ) with integers in −12..=12 (8,556 root pairs), no interval straddled zero, 90 pairs showed containment, and 20 lost their simple root in release. The `IsolatedRoot` fallback is the common path (5,651 of the 8,556 release pairs), mostly because the binary64 agreement check fails for non-dyadic roots, so the unbounded process-global `RootCache` is populated on most solves in this family (*Tested*, [evidence](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)). See the [comparative review](../reviews/mathematics-preparation.md#cbo-025).

