# Mathematics preparation and structure

**Group G5 of the [first comparative review](README.md), 2026-10-27.** This page
covers [CBO-021](../codebase/mathematics-and-compilation.md#cbo-021) to
[CBO-025](../codebase/mathematics-and-compilation.md#cbo-025). It also gives the
mathematics-specific input to [CBO-001](../codebase/cross-cutting-mechanisms.md#cbo-001),
whose joint judgment G4 owns.

| Item | Value |
| --- | --- |
| Source inspected | `main` at `3e9a2525b`. `git diff dbc4c9436 HEAD` shows no change under `crates/pse-math`, `pse-compiler`, `pse-structural`, `pse-kernels` or `pse-runtime/src/math*`, so the audit baseline is still current for this group. No dirty product source was used. |
| Library realizations | `symbolica`/`numerica` 3.0.1, installed source. `salsa` 0.28.5. `pounce-presolve` 0.12.0 at fork revision `7f5e2e9`. `petgraph` 0.8.3 and `rustworkx-core` 0.18.1. The vendored `suitesparse_sys` 0.1.4 (SuiteSparse 7.7.0). Upstream Symbolica PR and issue metadata for #103, #115, #127, #129 and #130 was read on 2026-10-27. |
| Consumed accounts | The codebase account (first audit, 2026-10-27) with its library-assessment corrections. The library accounts for symbolica, numerica, salsa, cacache, sccache, pounce, suitesparse, petgraph-and-rustworkx-core, rug-and-malachite and retained-size-crates. [Additional patterns](../assessment-patterns.md) revision 2. |
| Standard | Core 3.4 and process-simulator profile 1.5 via `standard.toml`, with the pse-arrow binding. Architecture owners: [§7](../../authoritative_design/sections/mathematics-and-compilation.md#section-7), [§14](../../authoritative_design/sections/mathematics-and-compilation.md#section-14) and [§15](../../authoritative_design/sections/numerical-execution.md#section-15), plus ADR-0082 and ADR-0212. |
| Evidence | Every target is *Proposed*. Mechanisms are *Interface-checked* by static reading of the installed library source and repository source. The reviewer ran no probes or builds; the coordinator-run [containment probe](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md) then *Tested* the CBO-025 containment trace. The [root-refinement probe](../../design_review/evidence/library-leverage-symbolica-root-refinement-2026-10-27/README.md) is the only *Tested* input. |

## Group summary

**Joint decision.** This subsystem already gives mathematics to libraries more thoroughly
than any other part of the workspace. The comparison shows that most of the remaining local
machinery is the residue the libraries leave. Retaining it is correct, with one exception:

- the cubic root-refinement ladder of
  [CBO-025](#cbo-025), where reading the library source exposed a correctness hazard that the
  local screen does not cover;
- two smaller consolidations that only make existing obligations explicit, in [CBO-023](#cbo-023)
  and the CBO-001 input.

| CBO | Disposition | Target in one line |
| --- | --- | --- |
| [021](#cbo-021) | Retain current, with a declared hold. U1 changes the route. | Keep the predict-then-reconcile mirrors and the abort binding. Declare the Symbolica coupling once as a pin hold. If U1 is yes, contribute a deterministic pre-allocation limit and a per-call abort upstream. |
| [022](#cbo-022) | Premise rejected; retain current | The four layers have four lifetimes, each with its natural key. The disk layer is ADR-0212's tested decision. |
| [023](#cbo-023) | Recommend (narrow) | One checked dense-decomposition owner in `pse-structural`, reused by implicit reconstruction. Stack ownership stays with callers and is stated. Iterative matching is deferred to growth. |
| [024](#cbo-024) | Premise rejected; retain current | One project-owned factorable projection with library-format consumers. Each consumer's operator rules are its own semantics. |
| [025](#cbo-025) | Split | Exact LDLᵀ: retain current. Cubic ladder: recommend isolating on the square-free part, a local exact bisection, and deleting the `refine_root_interval` and `IsolatedRoot` fallback. |

**Interactions with other groups.**

- **G4 (CBO-001).** This group supplies [the mathematics input](#cbo-001-math). The Symbolica
  mirrors already reconcile their predictions against the built product. The genuine
  inconsistency is std `BTreeMap` geometry, re-derived differently at four or more sites.
- **G2 (cancellation and context).** The Symbolica abort binding (`CURRENT_ABORT`) is a
  library-held thread-local carrier in the LOP-X002 sense. It reads the same atomic flag that
  ADR-0195's notifying cancellation owner lends to scientific callbacks. Any G2 change to that
  carrier must keep a plain `Arc<AtomicBool>` readable from inside a foreign callback.
- **G6 (native solvers).** `MATCHING_STACK` is reused as a stack policy anchor outside
  structural analysis: `kinsol/block.rs`, `native_child.rs` (`NATIVE_CHILD_STACK_BYTES`) and the
  runtime `MathPolicy` minimum. An iterative matcher would therefore not by itself release
  those stack reservations. SuiteSparse BTF is already vendored for KLU under G6's features.
- **G1 (host processes and durable files).** The compiled-evaluator cache (ADR-0212) uses
  `tempfile` persist, std `File::try_lock` and a pidfd/process-group compiler child
  (`native_compiled.rs`, `CompilerChild`). A G1 recommendation that changes process
  containment or atomic file publication should be adopted there as a consumer. G5 has no
  separate preference.
- **G7 (numerical workflows).** Collocation nodes come from `IsolatedRoot`
  (`collocation.rs`), which populates Symbolica's process-global root cache. That is harmless
  only because the polynomial population (collocation orders) is small. It is named here so G7
  does not widen it.

**Shared prerequisites, reinforcing and competing choices.**

- *Shared prerequisite.* There is none for the retained items. CBO-023's consolidation needs a
  new internal dependency edge, `pse-math → pse-structural`. That edge is acyclic, and
  `pse-structural`'s dependencies are a subset of `pse-math`'s plus petgraph and rustworkx-core.
- *Reinforcing.* One Symbolica pin hold should name every private-behaviour coupling across
  CBO-021 and CBO-025:
  - the Dualizer expansion model;
  - the export source model;
  - abort retention in evaluator settings;
  - `refine_root_interval` preconditions;
  - the root cache.

  Under U1 the upstream contributions form one coherent package for the same library.
- *Competing.* There are none. SuiteSparse BTF and an iterative pounce matcher are alternative
  *growth* routes for CBO-023 and are not current choices.

**Priority by consequence**, kept separate from prerequisite order:

1. **CBO-025 cubic ladder.** This is a reachable correctness hazard: a debug panic and, in
   release, a false refusal of a valid unique root. The fallback also retains results without
   bound in a process-global library cache. The fix is small, and the [containment probe](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md) confirmed both failure modes.
2. **CBO-021 hold declaration.** It is cheap and turns an implicit upgrade hazard into a
   deliberate re-derivation. The upstream package (U1) is valuable but has no deadline.
3. **CBO-023 consolidation.** It extends existing result-invariant validation to a consumer
   that currently trusts block order silently.
4. **The CBO-001 BTree helper.** G4 decides.
5. **CBO-022 and CBO-024.** No action.

There is no prerequisite order between items 1–4.

<a id="cbo-021"></a>

## CBO-021 — Bounded and cancellable Symbolica operations through version-coupled mirrors

### Responsibility and target intent

Every library construction must be admitted before it allocates, refuse deterministically
with a typed `Limit`/`WorkLimit`, and observe cancellation before a result is retained.
§14.3.2 promises that refusal is typed and checked "before the allocation they protect",
and that "derivative admission bounds the pinned Numerica Taylor convolution … from the source
operation counts before vectorization, then reconciles the actual built operations". The
promised behaviour is **predict, admit, build, reconcile**.

Which library internals are mirrored is incidental. The local rule that an observed abort
means cancelled is promised, because Symbolica's abort is cooperative truncation: `build()`
can return `Ok` with a less-optimised evaluator.

The mechanism has a better shape than the opportunity account suggests:

- **Predictions take public observables as inputs.** The Taylor model in
  `library/admission.rs` (`expansion`, `expansion_with_products`) multiplies the scalar
  evaluator's public `count_operations()` by a model of `Dualizer::map_instruction`.
  `export_bound` in `native_compiled.rs` walks the public `export_instructions()` with a
  per-operand byte model of the C++ emitter.
- **Both are reconciled against the actual product.** `bounded_evaluator` refuses with
  `Contract("library Taylor expansion exceeded admitted bound")` when the built program counts
  more operations than predicted. The compiled path refuses when `export_cpp_str` output
  exceeds the bound.
- **Drift is therefore a loud typed failure, not a silent under-admission.** The residual
  exposure is transient. If a new release under-predicts, the library call allocates beyond the
  admitted amount before reconciliation rejects the product.
- **The abort binding** (`library.rs`: `AbortBinding`, `DetachAbort`, `scoped_abort`,
  `CURRENT_ABORT`) is the best available use of 3.0.1:
  - The evaluator retains the build-time closure, and no setter exists.
  - A bincode round trip clears it, which would make later passes unabortable.
  - The closure therefore consults the producer flag during `build()` and a scoped
    thread-local afterwards.

### Credible complete options

1. **Retain current.** Keep the mirrors with post-hoc reconciliation, the abort binding and the
   inter-step polling in `implicit_cubic.rs` and `class_evidence.rs`.
2. **Retain, and declare the coupling once.** Add a `[workspace.metadata.pse.pins]` hold for
   `symbolica` (none exists today). It should name each mirrored private behaviour:
   - the Dualizer expansion;
   - the export source model;
   - abort retention in `OptimizationSettings`;
   - `refine_root_interval` preconditions, until CBO-025 removes that coupling;
   - the root cache.

   It should also name the reconciliation tests that detect drift. A bump then becomes a
   deliberate re-derivation. `dual.rs` and `export.rs` already changed on upstream `main`, so
   the next release will need one.
3. **Upgrade route (unreleased).** Upstream `main` has `EvaluatorComposer::finish(outputs,
   OptimizationSettings)`, which takes per-call settings, and `from_instructions`. Neither is
   released. Neither removes the Taylor model, and `from_instructions` builds with default
   settings, which means no abort.
4. **Upstream contribution (U1).** A small package with this priority:
   1. A per-call abort, either an evaluator settings setter or an `abort_check` argument on
      `vectorize` and later passes. This retires the thread-local and the detach logic.
   2. A deterministic, pre-allocation limit on `vectorize`, for example a `max_operations`
      that the Dualizer checks with its own exact count before allocating and returns as a
      structured error. The repository would reserve the admitted cap, pass it, and release
      the excess after reconciliation. That keeps charge-before-allocate with the model owned
      by the algorithm's author.
   3. Deterministic checkpoints in `derivative`, `replace` and `coefficient_list`.

   Upstream already accepts this shape. Open PR #103 adds checked series operations with
   structural limits that return a structured error and publish no partial result.
5. **The community `ResourceBudget` (unreleased, `community` branch).** It is RSS and
   wall-time based and polled only by some numerical kernels. It cannot replace a
   deterministic prepaid refusal. At most it is a backstop (LOP-X001 countercondition).

### Comparative argument

- **Semantic fit and domain authority.**
  - Options 1 and 2 keep admission policy with its owner (`MathPolicy`, `EvaluationLimits`)
    and keep the library on what it measures exactly.
  - Option 4.2 would move the *model* to the library author while the policy stays local.
    That is the correct split, but it is not available.
  - Option 5 would put a non-deterministic budget in the place of a reproducible refusal,
    which violates the preservation constraint.
- **Change locality.**
  - The couplings sit in three places: `library/admission.rs`, `export_bound` and the abort
    binding in `library.rs`.
  - An upgrade breaks them in a detectable way. The exhaustive `Instruction` matches fail to
    compile when the `Fun` tag type changes, and reconciliation refuses when expansion shapes
    change.
  - Option 2 adds only a declaration. It lets the person bumping the pin find the obligation
    without reading the code.
- **Failure and recovery.** The current design fails closed. After an under-predicting
  upgrade, valid work is refused with a typed error until the models are re-derived. Nothing
  is admitted silently.
- **Total machinery.** Option 2 adds one manifest entry. Option 4 would remove about 80 lines
  of abort plumbing and the Taylor model, but only after a release.

### Revealing case

A Symbolica release improves CSE so that `count_operations()` on the scalar evaluator falls,
while the Dualizer expands the same constructs more. The prediction falls and the built
program does not:

- current code refuses with a `Contract` error after `vectorize` has already allocated;
- in tests, this is caught by the reconciliation units at the bump;
- in production, it is a loud refusal of valid work, not memory admitted under a false
  prediction.

This case does not overturn retention. It shows why the hold (option 2) is worth having and
why a library-side pre-allocation check (4.2) is the real end state.

The community budget (option 5) fails this case. It would let the allocation proceed until
RSS crossed a process-wide line, so the outcome would depend on concurrent work.

### Disposition

**Retain current**, with the hold declaration (option 2) as a clear-win minor addition.

- **U1 = yes.** Pursue option 4 as one upstream package, in the priority order above. Until it
  is released, treat it as not available. When released, delete `CURRENT_ABORT`,
  `AbortBinding`'s producer swap and `expansion*`. Keep the post-hoc reconciliation as an
  independent check: it costs one `count_operations()` and guards the library's own promise.
- **U1 = no.** Retain. At the next Symbolica release, re-derive `expansion*` and
  `export_bound` from the release source. Check whether `EvaluatorComposer::finish` with
  per-call settings can retire the thread-local for composed evaluators.

### Rule impacts

None of these need an ADR or design review:
- a pin hold is dependency policy (ADR-0165);
- local re-derivation at a bump is a fix within contract;
- an upstream contribution is no local decision.

### Evidence labels

- *Interface-checked:*
  - `library.rs` (`exact_evaluator`, `AbortBinding`, `scoped_abort`);
  - `library/admission.rs` (`expansion`, `bounded_evaluator` reconciliation);
  - `native_compiled.rs` (`export_bound` and its use before `export_cpp_str`);
  - `Cargo.toml` pins table (no `symbolica` hold);
  - upstream PR #103 and #129/#130 bodies.
- *Proposed:* the hold and the upstream package.

<a id="cbo-022"></a>

## CBO-022 — Prepared-mathematics reuse held in four layers beside Salsa

### Responsibility and target intent

Avoid repeating evaluator preparation and native compilation across rebinds, workspace
generations, lanes and processes, without ever letting a cache define meaning. §14.4 makes the
boundaries promises:
- "Salsa memoizes pure semantics, never native construction";
- "Salsa handles remain local";
- rotation "discards memos, not correctness";
- "there is no persistent Salsa store";
- the shared `MathService` retains bodies and views "in a byte-bounded `quick_cache`
  weighted cache".

ADR-0164 makes Salsa "an optional local accelerator" and assigns cross-restart reuse to
persisted dependencies. ADR-0212 (proposed; implemented and tested 2026-10-20) selected the
bounded disposable disk cache for compiled scalar modules.

### What the layers actually are

The account's "four layers, four keys" are four lifetimes. Each uses the identity natural to
that lifetime:

| Layer | Lifetime / sharing scope | Key | Why it is not redundant |
| --- | --- | --- | --- |
| Salsa `semantic_body` (`lru=64`) | One workspace generation, dependency-tracked | Salsa revision and interned `BodyKey` | Gives backdating and revalidation; discarded on rotation |
| `ModelingBodyRetention` (runtime `math/retention.rs` behind the trait in `modeling.rs`) | Process; shared across lanes, rotations and historical workspaces | Complete semantic dependency hash, checked against `semantic_identity()` and physical spec on every hit | Salsa databases cannot share memos. §14.4 requires reuse across rotation. |
| `ScalarCache` → frozen `ScalarProducts` (`library/admission.rs`) | One body's construction, then immutable inside the body | Hashed atom lists | Product composition: shares scalar sub-evaluators across derivative orders of the same body, not a parallel cache |
| `native_compiled/cache.rs` | Across processes | Complete generated source, profile, compiler, exports and dimensions | ADR-0212. Private copies survive eviction. Hits never restore scientific state. |

`state.sources`/`state.requests` are Salsa's ordinary external-key → input-handle registry
(inputs are created once and mutated through setters). They are not a cache.

### Credible complete options

1. **Retain current.**
2. **Salsa owns more.**
   - Durability-tiered memos for evaluator products are ruled out by §7.1/§14.4: evaluators are
     native construction effects, outside tracked queries.
   - The `persistence` feature is a prototype: a whole-database image keyed by Salsa `Id`s, not
     content-addressed, with no multi-process access and no coverage of native artifacts. It is
     ruled out by ADR-0164's placement of durable reuse.
3. **`cacache` 13.1.0 for the disk layer.** It would carry content addressing and verified
   reads. The rest remains local:
   - the entry, byte and scan bounds, and eviction (no GC or size API);
   - the nonblocking publication lock;
   - the private-copy rule.
4. **sccache as compiler wrapper.** It caches only the compile phase, not the linked module.
   It is a daemon outside per-call admission, which the persistence premise rejects.
5. **Public Salsa labels.** Replace the four `fn_ingredient_(..).database_key_index(input.as_id())`
   uses in `granular.rs` with an event-side `ingredient_debug_name` family map plus an
   `Id → SemanticId` map. `AsId` remains plumbing, and so does `current_revision`
   (`workspace.rs`; it is only re-exported under `salsa::plumbing` at 0.28.5).

### Comparative argument

- **Options 2–4 remove no local responsibility.** The bounded, owner-preserving, cross-scope
  parts are the local contract, and no candidate carries them.
  - The local disk cache is about 380 lines plus tests.
  - It has std `try_lock`, `tempfile` persist and checksummed records.
  - It is a fresh, ADR-backed design whose fresh-process, corruption, concurrent-writer and
    eviction controls passed.
  - `cacache` would add an append-oriented index whose growth and eviction remain unproven
    for a bounded temp cache.
- **Option 5 trades a compile-time failure for a runtime one.**
  - Today a Salsa upgrade that renames the generated ingredient accessor breaks the build
    loudly.
  - The public route keys a family table on `ingredient_debug_name` text, which is documented
    as not semver-guaranteed. Without a dedicated test, a rename would silently drop
    attribution.
  - It still uses two plumbing items, so the upgrade hazard shrinks without disappearing.
  - That is not a clear win.

### Revealing case

A workspace exceeds `WorkspaceLimits` and rotates to a fresh generation, or a second lane
admits the same definition. If Salsa owned the body product (option 2), both lose reuse and
pay native construction again. The retention layer keeps it, with an identity check on every
hit. The four-layer arrangement is the design working as specified.

### Disposition

**Premise rejected; retain current.** The layers are not fragmented reuse of one lifetime.

- **U1 = yes.** Optionally ask Salsa for a public `AsId`/`current_revision`, or user labels on
  events. This is low value.
- **U1 = no.** Nothing changes.

### Rule impacts

None.

### Evidence labels

- *Interface-checked:*
  - `granular.rs` (four `plumbing` uses, `parsed_heap`, `members_heap`);
  - `workspace.rs` (`current_revision`);
  - `executable/grouped.rs` (`semantic_body`, `body_heap`);
  - `modeling.rs` (`ModelingBodyRetention`);
  - `library/admission.rs` (`ScalarCache::freeze`);
  - `native_compiled/cache.rs` (`MAX_*`, `try_lock`, `persist`);
  - Salsa 0.28.5 `lib.rs` plumbing re-exports;
  - ADR-0212.
- The runtime `math/retention.rs` body was not read in depth.

<a id="cbo-023"></a>

## CBO-023 — Structural decomposition and dependency closure owned in several places

### Responsibility and target intent

Matching, DM and BTF answer whether equations can determine unknowns and give block order. In
this subsystem they serve two consumers:

- **Case structural analysis** (`pse-structural::incidence::analyze`). This is the checked
  semantic boundary of §15.3. It runs on a dedicated 32 MiB stack under the 100,000-row
  qualification, with cancellation between stages. It validates every pounce result against
  the declared dimensions, equality coverage, partition completeness and block order before
  converting to semantic IDs, and orders blocks by SemanticId through rustworkx's
  `lexicographical_topological_sort`.
- **Selected implicit reconstruction** (`pse-math::implicit_reconstruction`,
  `prepare_contract_inner`). It derives, per residual branch, which retained parameters each
  eliminated unknown structurally depends on. It runs the same pounce sequence on a dense
  local incidence. It then performs a *source-restricted* propagation along BTF block order
  (`dependencies[column]` unions). It requires a complete matching and BTF coverage, but
  performs no result-invariant validation.

The other graph builders serve different graphs with different meanings:

- `pse-structural::initialization` (`kosaraju_scc` over initialization blocks);
- the compiler's `executable/dependencies.rs` and `implicit.rs` (petgraph `Bfs`, reversed
  `Dfs`, `toposort` over compiler items).

They already use library algorithms. Their shared part is constructing a graph from domain
data, which is inherently local.

**Premise corrections from source.** The recursion guard is not missing in production:

- The only production caller of reconstruction is `pse-runtime/src/math/solves/derived.rs`
  (`prepare_reduced_supplier`, inside `job_retained_scoped`). The `pse-backend-native` call
  sites are tests.
- Math jobs spawn threads with `.stack_size(service.policy.stack_bytes)` (`math/jobs.rs`).
- `MathPolicy::validate` refuses `stack_bytes < MATCHING_STACK`.
- `CaseLimits` caps a case at 100,000 scalars, so a local block cannot exceed `MATCHING_ROWS`.

Both guards therefore hold *transitively*. Neither is stated or checked at the call. The
genuine gap is that reconstruction trusts pounce's block order without the validation that
§15.3 applies.

That matters because propagation reads `dependencies[column]` for columns of *earlier* blocks.
If block order were wrong, a dependency set would be empty at its use. The derivative support
would silently miss parameters, a PS-07 derivative-chain error that no numerical check flags.

### Credible complete options

1. **Retain current.**
2. **One checked dense-decomposition owner (recommended).** Extract from `analyze_inner`, in
   `pse-structural`, a function over a dense `EqualityIncidence` that does the following:
   - runs `hopcroft_karp` → `DulmageMendelsohnPartition` → `SquareComponents` →
     `BlockTriangularForm`;
   - applies the `MATCHING_ROWS` refusal and the existing result-invariant checks, including
     the block-order check;
   - takes the cancellation flag between stages;
   - returns validated blocks plus the inter-block edges already built for the lexicographic
     sort.

   `incidence::analyze` keeps its semantic mapping, thread spawn and SemanticId ordering.
   Implicit reconstruction calls the same function and keeps its complete-matching refusal
   and its source-restricted propagation. The kernel does **not** spawn. It states that the
   caller must run on a qualified stack (≥ `MATCHING_STACK`). That avoids a second, uncharged
   32 MiB reservation inside a math job that already carries one.
3. **SuiteSparse BTF/CXSparse.** This is the vendored `suitesparse_sys` with `btf`, and
   `cxsparse` if DM is needed.
   - `btf_maxtrans` with `maxwork` is non-recursive and work-capped.
   - `cs_dmperm` gives coarse and fine DM.
   - It would remove the stack thread and the row cap, but makes a pure-Rust semantic-tier
     crate build native C unconditionally. It also re-does the DM and BTF that pounce already
     supplies.
   - It is valuable only if case limits rise above 100,000 rows.
4. **An iterative Hopcroft–Karp in pounce.** The `dfs` in `matching.rs` is recursive.
   - It can be contributed upstream to `jkitchin/pounce` (U1), or carried in the maintainer's
     fork, which the workspace already pins.
   - It keeps one library and identical matchings (same neighbour order).
   - Like option 3, it is a growth route.
5. **petgraph `tred` for the propagation.** The assessment already rules this out. The full
   O(V²) closure costs more than the source-restricted walk, and no library builds a block DAG
   from BTF output.

### Comparative argument

- **Semantic fit.** Option 2 puts the §15.3 contract around every use of the library sequence.
  The library still owns the algorithms. Reconstruction's propagation remains local because it
  is a different question: parameter support, not closure.
- **Isolated reasoning and change locality.** A pounce upgrade or a change in the fork today
  must be re-validated in two places, one of which validates nothing. With option 2 it is one
  owner.
- **Execution.** Unchanged:
  - reconstruction runs at preparation time, per branch;
  - the validation is linear in the incidence;
  - no thread is added.
- **Growth.** Options 3 and 4 matter only when case limits exceed 100,000. Option 2 makes that
  future change land in one place, and the refusal (`MATCHING_ROWS`) then protects
  reconstruction too.
- **Machinery.** The kernel extraction shares the pounce sequencing and adds about a dozen
  validation lines to the reconstruction path. It requires a new internal edge,
  `pse-math → pse-structural`. That edge is acyclic, adds no new external library to
  `pse-math`'s build closure beyond petgraph and rustworkx-core (already in the workspace),
  and introduces no crate.

### Revealing case

Suppose the shared kernel spawned its own 32 MiB thread, like `analyze` does. Reconstruction
inside a math job would then hold two qualified stacks:

- math jobs charge one `policy.stack_bytes` per worker stack (`jobs.rs`);
- structural-analysis jobs separately charge `analyze`'s own thread
  (`ConstructionEstimate::Known(MATCHING_STACK)` in `modeling.rs`);
- the derived-solve job that hosts reconstruction has no charge for a second matching thread.

That would silently under-charge.

The target therefore leaves stack ownership to the caller and states the premise. That is the
honest form of what already holds transitively.

A second case: a future implicit block with more than 100,000 unknowns, after case limits are
raised. Today reconstruction would recurse past its stack. Under option 2 it receives a typed
refusal.

### Disposition

**Recommend (narrow): option 2.** The premise is partly rejected: the recursion guard holds
transitively, and the closure loop is a legitimate source-restricted propagation.

- **Bounded target.**
  - *Mechanism:* one `pse-structural` function over `pounce_presolve::EqualityIncidence`.
  - *Responsibility boundary:* the kernel owns limits, stage cancellation and result
    invariants. Callers own the stack, semantic mapping, ordering and propagation.
  - *Domain rules retained:*
    - SemanticId tie-breaking stays in `analyze`;
    - complete-matching refusal and source-restricted propagation stay in reconstruction;
    - conservative all-branch support is unchanged, because it belongs to pattern
      construction.
  - *Behaviour change:* reconstruction gains typed refusals on invariant violation and on
    more than `MATCHING_ROWS` rows.
- **Growth branch.** When case limits are raised past 100,000 rows:
  - U1 = yes: contribute an iterative `dfs` to pounce and adopt it through the fork.
  - U1 = no: carry the same change in the maintainer's fork, or use SuiteSparse `btf_maxtrans`
    behind the shared kernel.

  In both cases the shared kernel is the single place to switch. Releasing `MATCHING_STACK`
  as a policy anchor for G6's consumers is a separate decision.

### Rule impacts

- No ADR: this is a refactor within an accepted contract.
- §15.3's text names `pse-structural::incidence` as the checked boundary. Extending its consumer
  set to implicit reconstruction warrants a small architecture amendment through the
  design-edit route (revision row), so that §15.3 and the reconstruction contract in §7 agree.

### Evidence labels

- *Interface-checked:*
  - `pse-structural/src/incidence.rs` (`MATCHING_ROWS`, `MATCHING_STACK`, `analyze`,
    `analyze_inner`);
  - `pse-math/src/implicit_reconstruction.rs` (`prepare_contract_inner`);
  - `pse-runtime/src/math/jobs.rs` stack sizing and `math.rs` policy validation;
  - `pse-runtime/src/math/solves/derived.rs` caller;
  - `pse-structural/Cargo.toml` and `pse-math/Cargo.toml`;
  - the pounce `matching.rs` recursion (library account).
- *Proposed:* the target.

<a id="cbo-024"></a>

## CBO-024 — One expression lowered through several local IRs with a repeated operator vocabulary

### Responsibility and target intent

Give global solvers, presolve/FBBT and convexity recognition a library-neutral projection that
preserves:
- per-row fidelity (`Exact`/`Relaxed`);
- the distinction between obligations and auxiliaries;
- provider envelopes;
- guarded selections;
- the exact absolute-value branch identity.

ADR-0105 and ADR-0120 decide this projection: `FactorableProgram`.

### What the IRs actually are

There is one project-owned projection, the `factorable::Node` DAG, which is hash-consed with
children before parents. Its consumers are:

- **Curvature** (`curvature.rs`) reads `Node`s directly, with no further IR.
- **FBBT** (`presolve.rs::tape`) is a mechanical lowering into POUNCE's own library format
  (`FbbtOp`), whose closed operator set is declared by the library.
- **The SCIP binding** lowers the same nodes in `pse-backend-native`.
- **Class evidence** (`class_evidence.rs`) does not consume an IR. It proves affine
  coefficients at the atom level through Symbolica `derivative` and `replace_multiple`, which
  is correct for an algebraic proof.

Each consumer's handling of an operator is that consumer's semantics:
- an interval rule;
- a curvature rule;
- a native expression handler.

Exhaustive `match` over `Node` makes a new operator a compile error at each consumer, which is
the desired forcing function.

### Credible complete options

1. **Retain current.**
2. **Symbolica `export_instructions()` as the common lowering source**
   (symbolica/evaluator-instruction-interchange). It would discharge n-ary arithmetic, powers
   and functions for the arithmetic core. It loses:
   - fidelity;
   - obligation and auxiliary roles;
   - provider identity;
   - guarded selects;
   - `abs` branch identity.

   Its post-optimisation (Horner, CSE) shape changes FBBT enclosure tightness. Its `Fun` tag
   type changes on upstream `main`.
3. **Numerica balls for interval views.** These are not certified beyond basic arithmetic, and
   the evaluator domain is unreleased.

### Comparative argument

- **Option 2 would introduce a second projection** for the arithmetic core beside the one that
  must still exist for the semantic attributes. That increases the number of lowerings. The
  factorable builder already shares stage results as single nodes, so it gains nothing from
  library CSE. It would also couple FBBT tightness to optimiser settings.
- **Option 3 is not sound** for transcendental functions.
- **The pattern citations do not fit.** The account's "operator vocabulary repeated per
  lowering" is per-consumer semantics. LOP-X003 (generation pipeline) does not fit, because no
  generator is involved.

### Revealing case

Adding a `tanh` primitive requires all of these, and no shared stream would remove any of
them:
- a curvature rule (S-shaped, neither convex nor concave);
- an FBBT decision (`Opaque`, or a composite through `exp`);
- a SCIP expression (through `exp`);
- a factorable fidelity decision.

### Disposition

**Premise rejected; retain current.** There is no U1 dependency.

One observation for a later inquiry, not inspected to conclusion: `tape` lowers
`Node::Const` through `Constant::value()`, which rounds to nearest. Inexact rationals such as
1/3 therefore enter the FBBT tape unenclosed. §15.4 confirms presolve infeasibility proofs on
tolerance-expanded original facts, which appears to absorb this at the ulp scale. Whether
tightened bounds (rather than infeasibility claims) are also confirmed was not traced.

### Rule impacts

None.

### Evidence labels

*Interface-checked:*
- `factorable.rs` (module contract, `Node`, `Fidelity`);
- `presolve.rs` (`tape`);
- `curvature.rs` (`Node` matches);
- `class_evidence.rs` (atom-level `derivative`/`replace_multiple`);
- the library accounts for the instruction stream and balls.

<a id="cbo-025"></a>

## CBO-025 — Exact rational LDLᵀ and a cubic-refinement fallback ladder carried locally

The two kernels share only their pattern labels, so they are judged separately.

### Exact LDLᵀ (`convexity.rs`)

**Responsibility.** ADR-0121 requires an exact convexity decision and Gram factors. The
factors are consumed by `curvature.rs` cone recognition and `pse-backend-native`
`conic/lowering.rs`. The kernel (`ldlt`, about 80 lines) does the following:
- symmetric, fewest-entries pivoting over `Symbolica Rational`;
- decides positive semidefinite including the zero-pivot (semidefinite) case;
- detects negative curvature from a negative or zero-with-coupling diagonal;
- spends a deterministic multiply-add budget (`Exhausted` → `Inconclusive`);
- polls cancellation per pivot.

**Options.**
- Retain current.
- Numerica `Matrix`/`SparseMatrix`: no LDLᵀ, inertia or definiteness.
- SPEX 3.2 LDL: an upgrade of the vendored SuiteSparse plus GMP linkage. It cannot decide
  semidefinite matrices (`SPEX_ZERODIAG`), gives no witness and has no budget or cancellation.
- rug/malachite: scalar arithmetic only.
- faer: floating point, so not exact.
- Sylvester leading minors through Numerica `det`: they cannot decide semidefiniteness and give
  no factors.

**Judgment.** Every library route either cannot reach the promised decision (semidefinite with
factors) or needs the local kernel as a fallback anyway. The kernel is small, exact,
deterministic, budgeted and cancellable.

**Disposition: Retain current.** There is no U1 dependency.

**Evidence.** *Interface-checked:* `convexity.rs` (`GramCertificate::certify`, `ldlt`) and
consumers (`curvature.rs`, `conic/lowering.rs`, `workflow/strategies.rs`); the numerica,
suitesparse and rug-and-malachite accounts.

### Cubic refinement ladder (`implicit_cubic.rs`)

#### Responsibility and target intent

The `cubic_roots` accelerator is an explicitly registered acceleration (§7.2). It must
enumerate *every* real root of an exact rational polynomial of degree at most 3. The
coefficients are binary64 values made exact. For each root it must produce a binary64 point.
It then accepts only if exactly one root passes original-residual verification inside the
declared branch interval (`CubicWorker::solve`).

The uniqueness claim is the selection authority, so the enumeration must be complete. The
refinement only needs binary64 resolution, because verification against the original residual
follows.

The current ladder (`real_points`) works in three stages:
1. It takes `isolate_real_root_intervals()`.
2. For each interval:
   - a degenerate interval is used as is;
   - otherwise it screens p(a) = 0 or p(b) = 0;
   - it calls `refine_root_interval` with relative tolerance 2⁻⁵⁴;
   - it checks that both ends round to the same binary64.
3. On any screen failure it falls back to `isolate_real_roots()` and `IsolatedRoot::refined`.

#### What the library source settles

Reading the 3.0.1 source (`poly/univariate.rs`: `isolate_real_root_intervals` for
`IntegerRing`, `isolate_real_roots_square_free`, `max_real_root_bound`;
`poly/univariate/roots.rs`: `refine_root_interval`, `refine_root_interval_until_disjoint`,
`RootCache`; `poly/factor.rs`: `square_free_factorization_0_char`) settles the open question
and exposes a different one.

**1. The isolator cannot emit zero-symmetric or zero-straddling intervals.**

- Positive roots are isolated by VAS-CF from the Möbius map
  `(a·y + b)/(c·y + d)` with non-negative integers `a, b, c, d` and positive `c, d`. Every
  endpoint is therefore ≥ 0.
- Negative roots come from the same procedure on `f(−x)`, mirrored, so their endpoints are ≤ 0.
- An exact zero root is emitted as `(0, 0)`, and the negative pass skips its duplicate.
- The cross-factor disjointness step only bisects within an interval, so it stays in the same
  closed half-line.

Every non-degenerate emitted interval therefore satisfies 0 ≤ a < b or a < b ≤ 0, so a + b ≠ 0.
`(0, 0)` reaches the local `a == b` branch before any refinement. The probe's b1–b3 hazards are
unreachable on the isolate→refine path at 3.0.1. This settles the account's open question,
statically.

**2. A different precondition violation is reachable: strict containment across square-free
factors.**

`isolate_real_root_intervals` isolates each square-free factor separately. Yun's algorithm
lists factors by increasing multiplicity. The method then calls
`refine_root_interval_until_disjoint` for each earlier/later pair whose interval overlaps.
That function's overlap test (both at entry and as its loop condition) asks only whether an
*endpoint of the earlier interval* lies in the later one. When the earlier interval strictly
contains the later one, the test reports "disjoint". The earlier interval then contains two
distinct roots of the polynomial with no sign change of its square-free part between the
endpoints.

For degree ≤ 3 this needs a rational double root:
- the polynomial is (αx−β)²(γx−δ);
- both factors are linear;
- their local-max root bounds differ by a factor ≥ 4;
- the wider interval comes first.

Worked example (static trace): p = 12x³ − 68x² + 123x − 72 = (3x − 8)(2x − 3)².

1. Yun gives `[(3x−8, 1), (2x−3, 2)]`.
2. The local-max bounds are 16 and 4, so the intervals are (0, 16) and (0, 4).
3. Lockstep bisection gives (0, 8)/(0, 2), then (0, 4)/(1, 2).
4. The test now sees neither endpoint of (0, 4) inside (1, 2), and exits.
5. The emitted intervals are (0, 4) and (1, 2). (0, 4) contains both 3/2 and 8/3.

`real_points` then handles (0, 4) as follows:
- p(0) = −72 and p(4) = 100, so the endpoint screen passes;
- the square-free part has the same sign at both ends;
- **debug build:** the `debug_assert` in `refine_root_interval` panics;
- **release build:** the first midpoint 2 separates the roots; bisection lands exactly on 3/2
  and converges there;
- (1, 2) also yields 3/2;
- the result is `[3/2, 3/2]`, and 8/3 is never produced.

The consequence is a false refusal:
- if the branch interval contains only 3/2, two verified copies give "exactly one … root"
  refusal for a valid unique root;
- if the branch contains only 8/3, nothing verifies.

In the containment geometries I could derive for degree ≤ 3, release returns a duplicate of
the inner root rather than a non-root, so no wrong acceptance follows. That is not proved
exhaustively.

**3. The fallback retains without bound.** `isolate_roots`, `isolate_real_roots` and
`IsolatedRoot::refined` populate `root_cache()`. That is a process-global
`LazyLock<RootCache>` of `RwLock<HashMap<Vec<coefficients>, …>>`, with no eviction and no clear
API at 3.0.1. Each cubic evaluated at new parameter values is a new key. A long sweep that keeps
reaching the fallback therefore grows a library-owned map outside every admission. §14.3.2's
"declared conservative allowance" for foreign Symbolica storage is per operation, so it cannot
bound this. `refined` also takes an *absolute* radius, which gives weak relative accuracy for
tiny roots. The original-residual check mitigates that.

#### Credible complete options

1. **Retain current.** It has the hazard in 2 and the unbounded retention in 3.
2. **Retain the library refiner, and complete the precondition screen.**
   - Compute the square-free part u = p / gcd(p, p′) (Symbolica `gcd`, `derivative`, exact
     `Div`).
   - Require u(a)·u(b) < 0 before `refine_root_interval`.
   - For degree ≤ 3 that check is complete: an interval with no sign change holds an even
     number of roots of u.

   It still needs a fallback when the screen fails, and the only library fallback is the
   caching `IsolatedRoot`.
3. **Always `IsolatedRoot`.** This is the certified route. It makes the unbounded global
   retention the normal path and keeps the absolute-radius semantics. Ruled out.
4. **Isolate on the square-free part, then refine by local exact bisection (recommended).**
   - Call `u.isolate_real_root_intervals()` on u alone. A single square-free factor never
     enters the cross-factor step. The only separate factor Symbolica's integer path can
     produce is a monomial x, and its `(0, 0)` is listed first and resolved correctly.
   - Every non-degenerate interval then has exactly one root of u in its *open* interior,
     though an endpoint may be a different root.
   - Refine each interval by exact rational bisection on u:
     - take the side sign from u(a), or from u′(a) when u(a) = 0 (u is square-free, so
       u′(a) ≠ 0);
     - return an exact midpoint root immediately;
     - poll cancellation and the existing time limit each step;
     - stop when the bracket lies within one binary64 rounding cell, then **certify** the
       returned double d: evaluate the sign of u exactly at d's two rounding half-way points
       (d ± ulp/2), with ties to even when the root lies exactly on a half-way point. A bracket
       whose ends merely convert to the same double is not a certificate. *(Revised after the numerical and solver methods challenge.)*
     - convert rationals to binary64 with an exact, correctly rounded conversion. Never use
       Numerica 3.0.1 `Rational::to_f64`: when numerator and denominator fit in i64 or i128 it
       computes `n as f64 / d as f64`, which rounds twice once either exceeds 2^53. Bisection
       midpoints with non-dyadic denominators land in exactly that range; an emulation misrounded
       71,124 of 200,000 such values and found brackets whose ends both convert to one double
       while the enclosed root rounds to its neighbour. The current ladder shares this defect.
     - apply a deterministic step cap derived from the bracket, ⌈log₂(b − a)⌉ + 1,076 plus a
       margin, or bisect on the binary exponent first. A fixed cap near the ordinary step count
       (about 60, for example 3/7 on (0, 4)) would refuse roots near the subnormal range, which
       take 1,074–1,075 steps (`x − 2⁻¹⁰⁷⁴` on (0, 1)) and which today's fallback returns.
     - map roots that round below the smallest subnormal to ±0 by IEEE rounding, and refuse typed
       a root that rounds beyond `f64::MAX`.
   - Delete `refine_root_interval`, the p(a)/p(b) screen and the `IsolatedRoot` fallback.
5. **Numerical refinement on the exact bracket.** This could use the existing `root1d::toms748`
   or the unreleased `solve::nsolve_bracketed` (PR #127). Binary64 sign evaluation near
   near-critical roots cannot respect an exact isolating bracket, which is exactly where the
   current fixtures already live (roots ±2⁻³⁰). Not preferred.
6. **Upstream fixes (U1).**
   - A containment-aware overlap test in `refine_root_interval_until_disjoint`.
   - A fallible `refine_root_interval` that checks its precondition, handles both endpoint
     cases and offers absolute tolerance and abort.
   - A bounded or clearable `RootCache`, or a non-caching isolation entry.

#### Comparative argument

- **Domain authority.** Option 4 keeps certified root *counting and isolation* with Symbolica.
  It keeps the scientific acceptance (residual verification and uniqueness in the branch)
  local. It owns only exact bisection on a certified bracket, about 20 lines with no domain
  meaning.
- **The stated reason for owning generic machinery (PS-09):**
  - the library's interval refiner checks its precondition only with `debug_assert`, with
    profile-dependent failure modes;
  - it uses a relative stop rule;
  - it has no cancellation;
  - its certified alternative retains without bound in process-global state.
- **Failure and recovery.**
  - Option 4 has no build-profile-dependent behaviour and no hidden retention.
  - Every remaining refusal is typed: step cap, non-finite rounding, uniqueness.
  - Option 2 leaves a fallback that re-introduces the cache.
- **Machinery.**
  - Removed: one library call with an unchecked contract, the screen, the f64-agreement
    fallback trigger, and the second isolation path through factorisation and complex roots.
  - Added: a short bisection and one `gcd`/`Div`.
  - Execution per solve stays close to the current primary path: one gcd of degree ≤ 3,
    isolation, and about 60 rational bisection steps per ordinary root, up to about 1,075 for
    roots near the subnormal range unless the exponent is bisected first.
- **Determinism.** Exact rational arithmetic with binary64 rounding at one boundary, so the
  result is FMA-independent. The point may differ from today's in the last ulp, so the solver
  identity string (`symbolica.cubic-roots.v2`) should advance.

#### Revealing case

Two distinct legitimate cases challenge option 4:

- **Repeated root inside the branch.** For a cubic (x − r)²(x − s), the target enumerates r
  once and s once (distinct roots of u), as today. A double root that is the only root in the
  branch is still accepted as a *value*. The existing IFT refusal keeps it from becoming a
  regular jet (`repeated_cubic_root_is_a_value_but_never_a_regular_ift_jet`). No distinction is
  erased.
- **Very small roots** (the `x³ − 2⁻⁶⁰x` fixture). Isolation on u gives `(0, 0)` and the
  half-line intervals for ±2⁻³⁰. Bisection with the derivative-side rule reaches binary64
  resolution in a bounded number of steps without relative-tolerance division.

#### Disposition

**Recommend option 4.** This doubles as the product-side fix that the codebase account asked
to be checked. The [probe outcome](#probe-outcome) confirmed the containment trace, so it is high priority.

- **Retained domain rules:**
  - original-residual verification of every point;
  - "exactly one verified root in the branch";
  - nondegenerate-polynomial refusal;
  - cancellation and time-limit polling;
  - coefficient-evaluator admission.
- **Behaviour changes:**
  - repeated-root cubics in the containment geometry stop panicking (debug) and stop refusing
    valid unique roots (release);
  - the root-cache growth path disappears;
  - points may move by at most an ulp, so advance the solver identity.
- **U1 = yes:** also contribute option 6 upstream. It benefits collocation, which keeps
  `IsolatedRoot`, and any future caller. Do not switch back to the library refiner merely
  because it becomes fallible: a 20-line exact bisection on a certified bracket is not a
  maintenance burden.
- **U1 = no:** option 4 stands. Record the three library conditions in the Symbolica account
  and pin hold, together with `Rational::to_f64`'s double rounding, which also affects
  coefficient conversion in `library/admission.rs` (`map_coeff(|c| c.re.to_f64())`).

#### Rule impacts

- No ADR: this is a bug fix and refactor within §7.2's registered-acceleration contract.
- The solver identity bump is an ordinary versioned identity, not a hashing-contract change.

#### Evidence labels

- *Interface-checked:* the Symbolica 3.0.1 source paths above, and `implicit_cubic.rs`
  (`real_points`, `CubicWorker::solve`, tests).
- *Tested:* the hazards b1–b3, c1–c2 and d2 in isolation (the probe record).
- *Tested:* the containment reachability and its consequences, in the [containment probe](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)
  (see [Probe outcome](#probe-outcome)).
- *Proposed:* the target.

<a id="cbo-001-math"></a>

## CBO-001 (mathematics input for G4's joint judgment)

- **Symbolica mirrors follow a predict-then-reconcile discipline.** See [CBO-021](#cbo-021).
  - Predictions use library-public observables (`count_operations`, `export_instructions`,
    `get_byte_size`) through private-algorithm models.
  - Actual products are reconciled with typed refusal.
  - Residual exposure is transient over-allocation inside the library call when a new release
    under-predicts.
  - Every library route that could prepay is unreleased or non-deterministic. Under U1, a
    library-side pre-allocation limit is the real replacement.
- **Std B-tree geometry is the genuine unconsolidated mirror.** The code mirrors std's private
  `BTreeMap` node layout (B = 6: 11 key/value slots, 12 edges) with **different formulas at each
  site**:
  - `pse-structural/src/flowsheet.rs` charges (n + 1)·(11·w + 16 words);
  - `pse-compiler/.../executable/implicit.rs` (`implicit_node_bytes`) charges 16·K + 16·V + 32
    words per entry;
  - `pse-math/src/convexity.rs` (`construction_allocation_bound`) charges 4 entries + 256 bytes
    per cell and 3 maps + 512 bytes per row;
  - measured `retained_bytes` in `pse-math/src/execution.rs` uses len·(8 + 96) per `BTreeSet`
    and +96 per `SemanticId` entry.

  `pse_model::HeapUsage` has no `BTreeMap`/`BTreeSet` implementation. No library predicts B-tree
  extent before construction. get-size2's `BTreeMap` measure sums keys and values only, so it
  under-counts against the conservative direction.

  Suggested target for G4: one conservative B-tree extent helper beside `HeapUsage` in
  `pse-model`, used for both predictions and measurements. It would be derived once from the std
  node layout and carry its own pin-style note that it mirrors a private std constant. It is a
  local LOP-09 consolidation, with no library involved.
- **Salsa `heap_size` is a consumer, not a second system.** `parsed_heap`, `members_heap`,
  `body_heap` and `body_key_heap` return local `retained_bytes()`. `memory_usage` is a post-hoc
  census. Nothing to change.
- **Retained-size derives have little reach in `pse-math`.** There are about 36 hand-written
  `retained_bytes`/`heap_bytes` bodies and 54 `get_byte_size` uses. Most math products hold
  Symbolica atoms or evaluators, which would need a `size_fn` per field. Measurement is not the
  math prepay sites' problem; prediction is.

## Pattern sufficiency

The working set framed this group adequately. Three refinements would have sped recognition,
and one framing is new. The texts are in [additional patterns](../assessment-patterns.md) revision 3:

- **LOP-X001: distinguish reconciled from unreconciled mirrors.** A mirror whose prediction
  is checked against the built product with typed refusal fails loudly on drift. Its residual
  risk is the transient allocation inside the call. An unreconciled mirror drifts silently.
- **LOP-X009: cover unchecked API preconditions,** not only hostile input. Some libraries
  check a precondition only with `debug_assert!`, so failure depends on the build profile. In
  some cases the *same library's companion producer* can emit inputs that violate it. Prefer
  a complete cheap precondition, or restructuring the input (here: square-free), over
  screening for observed failures.
- **New [LOP-X014](../assessment-patterns.md#lop-x014): library-held process-global retention outlives caller ownership.**
  Symbolica's `RootCache` is the prompting case. The formal-symbol pool in §7.1 is the
  repository's existing managed answer to the same shape for Symbolica's symbol registry.

## Probe outcome

The coordinator-run containment probe answered this page's request
([evidence](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md); *Tested*, Symbolica 3.0.1, one run per case, debug and release profiles).

- **The worked example reproduces exactly.** For (3x − 8)(2x − 3)², that is 12x³ − 68x² + 123x − 72,
  `isolate_real_root_intervals` returns `(0, 4)` with multiplicity 1 and `(1, 2)` with
  multiplicity 2. Both roots lie strictly inside `(0, 4)`. The debug ladder panics on the
  `refine_root_interval` sign assertion (`roots.rs:2202`). The release ladder returns `[1.5, 1.5]`:
  the simple root 8/3 is lost and the double root duplicated.
- **The sweep.** Over (αx − β)²(γx − δ) with integers in −12..=12 (8,556 distinct root pairs),
  90 pairs show containment, every one of them panics in debug, and 20 lose their simple root in
  release without any refusal. The other 70 are rescued, 69 of them through the `IsolatedRoot`
  fallback. No interval straddles zero, which confirms condition 1 empirically.
- **The fallback is the common path.** The ladder took the `IsolatedRoot` fallback on 5,651 of the
  8,556 release pairs, mostly because `refine_root_interval` stops within about one ulp and the
  binary64 agreement check then fails for non-dyadic roots. The process-global `RootCache` is
  therefore populated on most solves in this family.

The first outcome branch of the request holds: the cubic target is a product defect fix with high
priority, and the Symbolica account gains the containment condition.

## Limits

- Not read in depth: runtime `math/retention.rs`, the SCIP lowering of `Node`, `curvature.rs`
  rules, `typed*.rs` and `assembly*.rs` (the suspected CBO-021 footprint), and the Salsa
  event-handling consumer of `label_query`.
- Symbolica `main` and the `community` branch were taken from the library account and PR
  bodies, not re-read.
- The cacache `index` module was not read.
- The probe exercised Symbolica directly, not `real_points` inside a `pse-math` unit fixture; the
  ladder was reproduced in the probe crate.
- No measurements were taken. Every performance remark is qualitative.
