# Preparation-family capability gaps: Symbolica 3.0.1 (with Numerica and Graphica), Salsa 0.28.4, pounce-presolve, petgraph, rustworkx-core, faer, rpds, quick_cache, typed-index-collections

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Capability-gap inventory: Symbolica 3.0.1, Salsa 0.28.4 and structural libraries.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

Baseline: HEAD `0530dcaa1` (source `a791c533e`). Evidence comes from Context7 (Symbolica and Salsa docs, which describe pre-3.0 Symbolica signatures), the pinned sources in `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`, the pounce checkout `7f5e2e9`, and the repository source. Nothing was built or run. Every claim below is *Interface-checked*, or *Implemented* where it describes current source. All cost and speed claims are *Proposed*. I wrote no files.

**One correction to the census (§3).** It lists Symbolica `native_code_generation` as off. That is true of the default build only. `pse-math/compiled-evaluator` turns it on (`crates/pse-math/Cargo.toml:18-22`), and so do `pse-runtime` and `pse-py` (`crates/pse-runtime/Cargo.toml:31`, `crates/pse-py/Cargo.toml:33`). That feature gates `export_cpp`/`CompiledRealEvaluator` and `jit_compile` (`symbolica-3.0.1/src/evaluate.rs:7-45`).

---

## 1. Capability surface at the pins

### Symbolica 3.0.1 (+ Numerica domains)

**Symbols and global state**
- `SymbolBuilder` offers `with_attributes`, `with_normalization_function`, `with_derivative_function`, `with_evaluation_info`, `with_user_data` (`src/atom.rs:897-1180`). `SymbolAttribute` includes `Real` and `Positive` (`atom.rs:718-737`).
- Symbols live in one process-global, append-only table. Registration order defines canonical term order, which `crates/pse-math/src/library.rs:37-41` explicitly guards against.
- `Indeterminate::Function(Symbol, Atom)` (`atom.rs:2312-2317`): a function atom such as `f(7)` can be a derivative variable or an evaluator parameter. Symbolica's own `Dualizer` builds its parameters this way, as `function!(dual_arg, index)` (`src/evaluate/dual.rs:558-560`). So integer-indexed function atoms are a library-native idiom for positional formals that needs no per-slot symbol registration.

**Evaluator construction** (`EvaluatorBuilder`, `src/evaluate/function_map.rs:289-480`)
- Options: `cores`, `horner_iterations`, `cpe_iterations`, `hot_start` (reuse a Horner scheme), `abort_check`/`abort_level`, `max_horner_scheme_variables`, `max_common_pair_*`, `direct_translation`.
- `build()` takes the direct translator when `direct_translation` is set or any function is not inlined. Otherwise it builds an `EvalTree` (`to_eval_tree_multiple`), optimizes it (Horner, CSE via `common_subexpression_elimination`, `tree.rs:3385`) and linearizes.
- `evaluator_multiple` optimizes all outputs jointly: common subexpressions across outputs are computed once (symbolica.io "Multiple expressions").

**FunctionMap** (`function_map.rs:94-262`)
- Registration: `add_function[_with_options]`, `add_tagged_function` (leading tag arguments; one body per tag), and `add_aliases`. Aliases are argument-free and "always inline in the scope where they are used" (`:156-176`).
- `InliningPolicy` (`:34-47`):
  - `Always` (the default);
  - `Never`, which makes a separate sub-evaluator;
  - `Auto`, which currently behaves like `Always`.
- An alias inherits the caller's bindings and expression cache (`tree.rs:974-990`). The direct translator checks that cache before emitting (`tree.rs:1013`), so a repeated alias reference is linearized once. On the tree path, an alias body is expanded in place at every use (`tree.rs:4013-4025`) and only deduplicated later by CSE.
- The tree has an `IfElse` node (`tree.rs:1540`). The repository deliberately does not use it; see §5.

**Evaluator operations**
- `merge(other, cpe_rounds)` requires an equal parameter count, concatenates outputs, then runs common-pair elimination and `optimize_stack` (`src/evaluate/optimize.rs:239-495`).
- Also available: `optimize_stack` (`:504`) and `set_real_params` (`:84`).
- Introspection: `count_operations` (`evaluator.rs:745`), `export_instructions` (`instruction.rs:219`), `map_coeff`/`map_coeff_with_prec` (`evaluator.rs:653-690`).

**Derivatives**
- `vectorize(&Dualizer)` performs Taylor-mode expansion over `HyperDual` shapes. `zero_components_per_parameter` declares structural zeros (`dual.rs:127`, `:521-536`). `InstructionList::is_zero`/`is_one` (`instruction.rs:510-529`) suggest that zero components fold away; not verified.
- A sub-evaluator, which is what `InliningPolicy::Never` produces, cannot be vectorized. The repository's own test asserts this (`crates/pse-math/src/library/admission.rs:1009`, `:1048`).
- Symbolic `derivative(Indeterminate)` is also available.

**Backends**
- `export_cpp[_str]` with `CompiledRealEvaluator::load` and `load_new_function`.
- `jit_compile` (symjit 2.26.0, resolved), with `JITCompilationSettings { direct_translation, optimization_level (default 3), options }` (`src/evaluate/backend.rs:415-540`). It targets `f64`, `f64x4` and complex; `BatchEvaluator` is provided.

**Disabled features**
- `serde`/`bincode` would make evaluators and `FunctionMap` serializable. `bincode` decoding requires a state map (`HasStateMap`).
- `faster_alloc`, `python_*` and `c_api` are also off.

**Pattern work.** `replace(...).with(...)`, `replace_multiple` and `Transformer` exist, but they operate on post-admission atoms only.

**Graphica** offers graph canonization and isomorphism (`graphica-3.0.1/src/lib.rs:1383`, `:1777`). No preparation obligation needs it, because dispatch grouping already identifies equal structures.

### Salsa 0.28.4 (enabled: `macros`, `inventory`, `salsa_unstable`)

**Inputs**
- `#[salsa::input]` accepts `heap_size`, `singleton`, `debug`, `constructor` and field `#[default]` (`salsa-macros-0.28.4/src/input.rs:36-86`; `HEAP_SIZE: true` at `:68`).
- `Input::new` takes `&Db`, so creating an input needs no `&mut` (`salsa-macro-rules-0.28.5/src/setup_input_struct.rs:254`).
- Builder durability: `*_durability(Durability::…)`.

**Tracked functions** accept `returns(...)`, `specify`, `no_eq`, `lru`, `heap_size` and `cycle_fn`/`cycle_initial`/`cycle_result` (`tracked_fn.rs:30-64`). Backdating is the default unless `no_eq` is set.

**Tracked structs**
- Untracked fields form the identity. `#[tracked]` fields carry their own dependency revisions, so a reader of one tracked field is not invalidated by a change to another (`tracked_struct.rs:74`; the runtime `tracked_field` reports a per-field read, docs.rs `tracked_struct.rs`).
- They are created inside a tracked function, and identity is disambiguated per creator.

**Interned structs** accept `revisions`. The default is 3 reusable revisions before reclamation; `IMMORTAL` is opt-in (`salsa-0.28.4/src/interned.rs:31`, `:260`).

**Handles and cancellation**
- `Storage: Clone` creates a new handle onto the shared `Zalsa` with its own local query stack (`storage.rs:259-266`). Concurrent handles share memos and block on in-progress identical queries (`EventKind::WillBlockOn`).
- Any `&mut` write (a setter) calls `cancel_others`. That sets the cancellation flag and waits until every other handle drops (`storage.rs:160-190`).

**Memory reporting.** `<dyn Database>::memory_usage()` returns per-ingredient `IngredientInfo` and `heap_size_of_fields` (`database.rs:407-511`, gated by `salsa_unstable`). Heap is reported only where a `heap_size` callback is declared.

**Disabled features.**
- `persistence` serializes the database: every persisted ingredient needs serde.
- `accumulator` carries side outputs.
- The `rayon` feature in 0.28.4 adds only a `SalsaValue` impl for `Either` (`salsa_value.rs:175`); there is no `par_map` in this release.

### Structural and support libraries

**pounce-presolve** (`crates/pounce-presolve/src/lib.rs:63-103`)
- Structure: `matching::hopcroft_karp`, `DulmageMendelsohnPartition`, `SquareComponents`, `BlockTriangularForm`, `coupling::classify_block`, `licq_check`, `find_redundant_rows`.
- `fbbt::{forward_pass, reverse_pass, run_fbbt}`.
- Reduction: `linear_eq_plan`, `ReductionFrame`/`ReductionStack`, `warm`.

**petgraph 0.8.3** (`stable_graph`, `graphmap`; `rayon` off): `toposort`, `kosaraju_scc`, `has_path_connecting`, `greedy_feedback_arc_set`, `visit::{Dfs, Bfs, Reversed, NodeFiltered}`, `algo::tred` (transitive reduction and closure on toposorted DAGs), `condensation`, `dominators`.

**rustworkx-core 0.18.1**: `dag_algo::{lexicographical_topological_sort, layers, collect_runs}`, `connectivity::find_cycle`, plus traversal.

**faer 0.24.4** sparse: `SymbolicSparseColMat::try_new_from_indices` returns a symbolic pattern and an `Argsort`; `SparseColMat::new_from_argsort` refills values against that symbolic pattern (`src/sparse/csc/mod.rs:1302`, `:2355`).

**rpds 1.2.1**: `RedBlackTreeMapSync`, `HashTrieMapSync`; O(1) clone with structural sharing.

**quick_cache 0.6.24**
- `sync::Cache`: `get_or_insert_with`/`get_or_insert_async` (per-key placeholder single-flight), `get_value_or_guard[_async]`, `entry`.
- `Lifecycle` (`is_pinned`, `before_evict`), `memory_used()` (`src/sync.rs:565-816`).

**typed-index-collections 3.5.0**: `TiVec`/`TiSlice` dense typed index spaces.

---

## 2. How the repository uses them (verified)

- **Positional slot pool.**
  - `Pool` registers `pse_math::slot_{k}` and `pse_math::function_{k}` in 8192-slot chunks under a `RwLock`, with a compatibility check on re-registration (`crates/pse-math/src/library.rs:18`, `:37-128`).
  - `BodyBuilder::bind` and `independent` materialize every non-trivial `let` value into a fresh slot through `Stage::Block{expressions: vec![atom], outputs: vec![slot]}` (`crates/pse-math/src/typed.rs:611-630`, `:763-774`).
  - Positional slots are what make alpha-equivalent bodies share one atom and one program across instances.
- **Regions.**
  - `arithmetic_region` fuses consecutive `Block` stages only when they are *independent*. It stops at the first block that reads a sibling's output (`crates/pse-math/src/execution.rs:4802-4870`, test at `:4833`).
  - So every dependent `let` is a separate Symbolica evaluator. Its slot becomes a parameter of the next evaluator, and no Horner or CSE optimization crosses that boundary.
  - Guards (`Require`, `Domain`, `Applicability`), `Branch` and `Provider` are host stages (`crates/pse-math/src/guarded.rs:76-140`).
- **Evaluator calls.**
  - `exact_evaluator` calls `Atom::evaluator_multiple` with an empty `FunctionMap`, a bounded optimizer and an abort hook (`library.rs:257-293`).
  - Taylor products come from `vectorize(Dualizer)` with `zero_components`, then `map_coeff` to `f64` (`library/admission.rs:587-710`).
  - Admission reads `export_instructions` and `count_operations` (`library.rs:330-380`). Exact preparation is shared through a `ScalarCache` (`admission.rs:440-545`).
  - C++ export runs under `compiled-evaluator`. JIT appears only in a probe test.
- **Per-output bodies.**
  - The compiler builds one `BodyInput` per scalar output, with `expressions: std::slice::from_ref(&expression)` (`crates/pse-compiler/src/workspace/modeling/executable/grouped.rs:956`).
  - Its identity frames rendered text: `ModelingConsumerBodyV5` plus `dsl::render_expr` and formal path strings (`grouped.rs:988-1040`).
  - Transitive members are inlined as `let` (`grouped.rs:525-650`, renames to `input_<n>`/`member_<n>`).
  - Yet `typed_math::Request::admit_modeling_outputs` already accepts `outputs: &[QuantityTypeId]`, one per expression (`crates/pse-compiler/src/typed_math.rs:336-345`, arity check `:457`). `Stage::Block` is documented as "Safe multi-output arithmetic" (`guarded.rs:79-87`).
- **Shared bodies discarded.** `DispatchBody { coordinates, expressions, equations }` (`crates/pse-modeling/src/specialize/group.rs:7-15`) is a per-definition, multi-output, member-sharing body over local coordinates. Its only consumer, `bound_bodies`, expands it back per instance (`group.rs:229-276`; called at `executable.rs:401`).
- **Package-wide `Catalog` input.**
  - `#[salsa::input] Catalog { checked: Arc<CheckedPackage> }` (`crates/pse-compiler/src/workspace/modeling.rs:178-181`).
  - `selected(catalog, root)` would backdate, but `projection` reads the input directly: `catalog.checked(db).context().scope` (`executable.rs:459`) and `model.function_contracts(catalog.checked(db))` (`executable.rs:1033`). Any package edit therefore re-executes `projection`.
- **Whole-root specialization with demand in the key.**
  - `Request { root, instance, bindings, limits }`, where `Bindings.demand: Vec<String>` (`modeling.rs:182-189`; `crates/pse-modeling/src/specialize.rs:121-135`).
  - `specialized` is one tracked query per root (`modeling.rs:323-342`).
  - Demand is applied as a post-pass after `instantiate` (`specialize.rs:641-648`), so it is separable in principle.
  - `BasisKey` repeats `bindings.demand` and hand-lists hashed fields (`crates/pse-runtime/src/math/preparation.rs:23-70`).
- **Defaults inside structure.**
  - `Symbol.initial` ("kept outside the body identity", `specialize.rs:177`) lives inside the `SpecializedModel` value. It is read only by `PreparedModeling::case_values` (`crates/pse-compiler/src/workspace/modeling/executable/solve.rs:110-120`).
  - A default-only edit therefore changes the `specialized` value and re-runs `projection`.
- **Push-style granular checking.** The checker sets per-declaration `#[salsa::input]`s between tracked calls (`crates/pse-compiler/src/workspace/modeling/granular.rs:13-40`, `:159-170`).
- **Hand heap estimators.**
  - The inputs declare no `heap_size`. `State::known_bytes` re-derives their heap by hand ("These input declarations supply no heap_size callback", `granular.rs:183-260`).
  - Tracked and interned `heap_size` callbacks call hand `retained_bytes` estimators (for example `body_key_heap`, `grouped.rs:57-97`).
  - `retention_usage` adds Salsa's `memory_usage` to that census (`crates/pse-compiler/src/workspace.rs:1754-1770`).
- **Compiler sharing across lanes.**
  - `CompilerDatabase: Clone` (`workspace.rs:204-211`), but no production clone exists.
  - A lineage's lanes share `Arc<Mutex<CompilerWorkspace>>` (`crates/pse-runtime/src/math.rs:557-563`; `math/modeling.rs:1617-1630`), so lanes of one lineage serialize on that lock.
  - Cross-lane reuse runs beside Salsa (DA-030): `MathService` with a weighted cache, `selected_flights`, `frontier_flights`, `basis_flights` and `scalar_flights` (`math.rs:365-402`), plus `quick_cache::unsync::Cache` behind a `Mutex` (`math/retention/weighted.rs:5-60`).
- **Name-keyed type environments.**
  - Each contribution × coordinate rebuilds `BTreeMap<String, Type>` from all symbols (`specialize.rs:2576-2582`). The `TypeLookup` trait already abstracts lookup (`crates/pse-modeling/src/types/environment.rs:8-16`).
  - Elsewhere, rpds is used for exactly this kind of persistent typing (`crates/pse-modeling/src/check/phases.rs:10`, `:589`).
- **Structural libraries.** HK/DM/BTF go through pounce-presolve (`crates/pse-structural/src/incidence.rs:6-10`; again at `crates/pse-math/src/implicit_reconstruction.rs:1473-1492`). petgraph and rustworkx are used for SCCs, toposort and cycles (`flowsheet.rs:470-543`). The compiler's member graph builds its edges by parsing `s_<hex>` names (`executable.rs:532-541`), and dynamic-row detection is a per-row fixed point over names (`executable.rs:614-655`).

---

## 3. Unused and under-used capabilities, mapped to obligations

### C1. Multi-output bodies per dispatch group, with members computed once — Symbolica `evaluator_multiple` CSE, local-coordinate slots and `zero_components`

- **Capability.**
  - Admit one body per `DispatchBody` rather than per scalar output:
    - its `equations` (and term or contribution outputs) are the output list;
    - its `expressions` (members) are shared intermediates;
    - its `coordinates` map directly to positional formals `slot(k)`.
  - One `evaluator_multiple` over all outputs CSEs shared members across outputs. `Dualizer` then gives all outputs' derivatives in one Taylor program, with `zero_components` declaring which coordinates each parameter does not carry.
  - The typed path and the stage model already accept multi-output requests (`typed_math.rs:336`; `guarded.rs:79`).
  - Per-instance binding stays `SlotBinding`/`CaseStructure`. One admitted program serves every instance of the group, as positional sharing does today.
- **(i) Obligations owned.**
  - **DA-028:** a member read by k outputs is computed once per instance evaluation, including its derivatives.
  - **DA-027, single body identity:** the dispatch group's content hash becomes the one body identity. Today `group_bodies` mints that hash and `ModelingConsumerBodyV5` mints a second one.
  - **DA-027, re-expansion:** expansion and re-identification disappear.
  - **DA-029:** removes the per-output re-normalization and per-output physical prepass, the per-output `rows.iter().find` and `position` scans, and the accumulator-term scan per contribution.
  - **DA-001** in part: no `input_<n>`/`member_<n>` renaming and no rendered-text framing for bodies.
- **(ii) Bespoke code removed.**
  - `bound_bodies` (`group.rs:229-276`).
  - Per-output `normalize`, `BodyInput`, `ModelingConsumerBodyV5` framing and `body_key_heap` (`grouped.rs:53-97`, `:525-1040`).
  - `MemberReads` (`executable.rs:1276+`).
  - Most of the expression copies that DA-027 counts live during preparation.
- **(iii) Fit and limits.**
  - Taylor width grows to the union of a group's coordinates. Without zero folding the cost would be 1+2n (first order) or 1+3n+2n² (second order) per instruction (`admission.rs:31-40`). Symbolica tracks structural zeros (`zero_components`, `InstructionList::is_zero`), so emitted work should follow actual dependence. That is unmeasured (probe Q1).
  - Admission budgets (`EvaluationLimits.derivative_components = 4096`, `operations`) then apply per group, so a large group may need splitting by output clusters. A policy cut, for example by connected component of the member-read graph, keeps groups within limits.
  - Work no longer depends on which instance (the cross-instance sharing granularity is unchanged).
  - Guard stages split the group into regions as today (see C2).
  - Conservation rows that are skipped as accumulators (`grouped.rs:924-936`) keep their own path.
- **(iv) Reverse check.**
  - Guard survival: preserved. `Require`, `Branch`, `Provider` and `Domain` stay host stages, and the region rule is unchanged.
  - Library-owned derivatives: preserved (`Dualizer` and symbolic `derivative`).
  - Conservation by construction: preserved. Closures are still assessed from original terms (`specialize.rs:3015-3083`), and contributions are outputs of the same body.
  - Value rebind: preserved. Bodies are value-free.
  - Premise-checked reuse: the physical prepass runs once per group instead of once per output, with the same `PhysicalPremises` capture.
  - Lane parallelism: unaffected.
  - Open regression risk: a mixed-demand case, where only some outputs are needed (observation-only members), pays for the whole group unless outputs are selected. `ProductKind::SelectedPartials` already selects outputs (`crates/pse-math/src/jets.rs:31-42`), so selection must happen at evaluation rather than at construction (Q1).

### C2. Fused let chains — Symbolica `FunctionMap::add_aliases` with `direct_translation`

- **Capability.** Inside one barrier-free run of dependent `Block`s, register the intermediate slots as aliases (`slot_k -> expr`) on the builder instead of materializing each one as a separate evaluator.
  - On the direct path, aliases share the caller's expression cache, so each is linearized once (`tree.rs:974-990`, `:1013`).
  - Slots that later host stages read (a `Require` argument, a `Provider` input, a `Branch` comparison) stay explicit outputs of the fused evaluator.
- **(i) Obligations owned.**
  - **DA-028**, the within-body half: Horner and CSE optimization across a body's `let` members.
  - Fewer evaluators per body, which feeds the DA-030 product counts.
- **(ii) Bespoke code removed.** The dependency break in `arithmetic_region` (`execution.rs:4831-4835`) becomes alias registration. Per-let `ScalarCache` entries collapse into one.
- **(iii) Fit and limits.**
  - The tree path expands aliases at every use (`tree.rs:4013-4025`), so a deep `let` DAG with heavy reuse grows exponentially before CSE. Use `direct_translation(true)`, accepting its weaker tree-level Horner optimization, or keep materialization above a reuse-depth threshold (Q2).
  - `native_compiled.rs:108-125` assumes an empty `FunctionMap`. Inlined aliases leave no sub-evaluators, so `export_cpp` should be unaffected; verify (Q2).
  - Never use `InliningPolicy::Never` here: a sub-evaluator cannot be vectorized (`admission.rs:1048`).
- **(iv) Reverse check.**
  - Guard order is unchanged: only `Block`→`Block` adjacency is fused, and every barrier still splits regions.
  - Taylor work admission recounts the fused export (`admission.rs:695-708`).
  - Nothing else is affected.

### C3. Function-atom formals in place of a registered symbol pool — `Indeterminate::Function`

- **Capability.** Use `slot(k)` (one symbol with an integer argument) as the formal for slot k, and `lift(k, args…)` (a tagged function) for provider lifts. Symbolica already parameterizes its own Dualizer this way (`dual.rs:558-560`).
- **(i) Obligations owned.**
  - The global-state and ordering limit that the Pool exists to manage. Canonical order of `slot(k)` follows the integer argument, independent of registration order or timing.
  - `body_slots` stops implying registration work, so the 16 384 registered symbols per chunk (measured at 5–6 ms per chunk, `library.rs:14-17`) go away.
- **(ii) Bespoke code removed.** `Pool` and `Symbols::cover`, chunk growth under a write lock, `FORMAL_CHUNK`, and the re-registration compatibility checks (`library.rs:18-128`).
- **(iii) Fit and limits.**
  - Function atoms are larger than variables, so admission budgets based on `get_byte_size` shift.
  - Body identities and printed output change once.
  - `max_horner_scheme_variables` and the parameter mapping operate on parameter positions after `to_eval_tree`, so they should be unaffected.
  - Derivative, `Dualizer`, `export_cpp` and JIT acceptance of function parameters must be probed (Q5).
- **(iv) Reverse check.** Alpha-normalized sharing is preserved, because the formals are still positional. No other strength is touched.

### C4. Per-instance, demand-free incremental specialization — Salsa tracked structs with `#[tracked]` fields, tracked functions keyed by them, and backdating

- **Capability.**
  - `specialized` becomes a small root query that creates tracked `InstanceSpec` structs. Each is identified by `(definition, path or instance id)` and has `#[tracked]` fields for the inherited environment, presets and selections.
  - A tracked `specialize_instance(InstanceSpec)` produces that instance's dispatch-group binding, rows and members. Unchanged instances backdate.
  - Demanded members become `demanded(model, path)` queries outside `Request` (the post-pass at `specialize.rs:641-648`).
  - Defaults move to a separate `#[tracked]` field or query (`defaults(InstanceSpec)`), so structure readers do not depend on them.
- **(i) Obligations owned.**
  - DA-030 granularity: a structural edit re-specializes only the affected instances.
  - Demand-free specialization: an observation demand no longer changes the specialization key.
  - Defaults vs structure: a default-only edit leaves the `projection` inputs equal, so downstream queries backdate.
- **(ii) Bespoke code removed.** `Bindings.demand` in `Request` and `BasisKey` (`preparation.rs:23-70`), and the `request_hash` fields for demand (`modeling.rs:186-208`).
- **(iii) Fit and limits.**
  - Cross-instance coupling must be expressed as query dependencies, not engine state: shared accumulators and closures, exchanges, connections, connectivity checks and replicated equations (`specialize.rs:638-660`).
  - Accumulators and conservation closure stay a root-level assembly query over instance outputs. That keeps conservation by construction intact, but the engine's single-pass `Engine` state (`specialize.rs:560-622`) has to be split. This is the costliest change in the family and a design-level item, not a mechanical one.
  - Salsa tracked-struct identity is process-local. Persisted identities stay with `pse-ids`.
- **(iv) Reverse check.**
  - Value rebind: preserved, because values stay `CaseValues` and are never Salsa inputs. Making study values Salsa inputs would regress rebind; do not.
  - Lane parallelism: unaffected.
  - Guard and derivative strengths: untouched.

### C5. A firewall for the package-wide `Catalog` — tracked accessor queries

- **Capability.** Route `projection`'s two direct `catalog.checked(db)` reads (`executable.rs:459`, `:1033`) through tracked functions keyed by the selected package, for example `physical_scope(selected)` and `function_contracts(selected)`. Equal results backdate, so `projection` re-runs only when what it consumes changes.
  - Done fully, `Catalog` would split into per-declaration tracked outputs. The granular phase queries already exist (`granular.rs:42-160`), and `selected(root)` would become a pull over them.
- **(i) Obligations owned.** DA-030: unrelated edits stop re-running projection and DA-029's scans for every live root.
- **(ii) Bespoke code removed.** None by itself. It makes the hand caches less necessary (C7).
- **(iii) Fit and limits.** The minimal form is two accessor queries. The full pull form conflicts with the push-style setter driver (`granular.rs:283-420`) and would need the checker phases to become tracked functions over inputs.
- **(iv) Reverse check.** None. This is pure dependency narrowing.

### C6. Memory reporting through Salsa `heap_size` on inputs, plus library sizes

- **Capability.** Declare `heap_size = …` on every `#[salsa::input]` (`Source`, `TypeInput`, `CompleteInput`, `OccurrenceInput`, `MemberPhase`, `ExpressionPhase`, `ReferencePhase`, `InventoryInput`, `Catalog`, `Request`, `DataDocumentInput`). Input heap then shows up in `memory_usage()` (`input.rs:68`; `database.rs:428-437`). Symbolica's `get_byte_size()` and `ExportedInstructions` stay the measured sizes for library objects.
- **(i) Obligations owned.** DA-022's reporting half: one reporting path. It does not own the size *models*.
- **(ii) Bespoke code removed.** The `State::known_bytes` census and its comment (`granular.rs:183-260`), and the census half of `retention_usage` (`workspace.rs:1754-1770`).
- **(iii) Limits.**
  - Callbacks still call hand `retained_bytes` estimators. In-family libraries cannot measure project structures.
  - A derive-based sizer (get-size2) is not in `Cargo.lock` (searched by name). It stays a lead outside this family's scope.
  - `memory_usage` is `salsa_unstable` and already pinned for that reason (`Cargo.toml:598`).
- **(iv) Reverse check.** None.

### C7. Same-lineage lanes sharing one Salsa database through cloned handles

- **Capability.** For one published revision, give each lane a `CompilerDatabase` clone. Lanes create their own `Request` inputs with `&db`, run `specialized`/`projection`/`semantic_body` concurrently, share memos, and block on in-flight identical queries. Publishing a new revision on that lineage goes through `&mut`, which cancels and joins the reader handles (`storage.rs:160-190`).
- **(i) Obligations owned.**
  - DA-030: "fewer runtime cross-lane caches" within a lineage. Salsa's blocking replaces `selected_flights`, `frontier_flights` and `basis_flights` for compiler products.
  - The hand equality and hashing of `BasisKey` and `SelectedRequest` (pointer-address keys, `canonical.rs:1845-1863`) for those products.
- **(ii) Bespoke code removed.** The per-lineage `Mutex<CompilerWorkspace>` serialization, and the flights for same-database products.
- **(iii) Fit and limits.**
  - Writers cancel every reader, and the granular checker publishes by setters (`granular.rs:400-420`). So checking must happen on a handle while no other handle is live, which is the current exclusive candidate model.
  - Cross-lineage and cross-database reuse (`MathService` body retention by `SemanticBodyHash`, `grouped.rs:99-130`) and durable reuse remain. Salsa cannot share across databases.
  - The interned `requests` bucket map in `State` (`modeling.rs:210-215`) would need synchronization.
- **(iv) Reverse check.**
  - "Cancellation wins" is consistent: a revision publication cancelling stale readers is correct for live lanes.
  - Durable and historical lanes stay detached databases (`historical` lanes, `math/modeling.rs:1581-1590`). Without that they would be cancelled by a head publication, which would regress P1 lane parallelism.
  - **Remedy open** if pinned-revision durable lanes must share one database with head publication. Salsa offers no reader-isolated write.

### C8. Lifecycle-aware single-flight caches — `quick_cache::sync::Cache`

- **Capability.** `get_or_insert_async` gives per-key placeholder single-flight with weighting, `Lifecycle::is_pinned` and `before_evict`, and `memory_used`. That could fold the separate `Flights` table and the `Mutex<unsync::Cache>` of `WeightedCache` into one structure for each cross-database product.
- **(i) Obligations owned.** DA-030: "same product found through more than one mechanism, each with its own eviction" collapses to one per product kind.
- **(iii) Fit and limits.**
  - `pse_columnar::Flights` completes "independently of caller futures" (`crates/pse-columnar/src/flight.rs:83-86`). quick_cache's loader is the caller: a dropped guard hands loading to the next waiter.
  - That changes cancellation and drain semantics. It is not a drop-in, and fits only where the caller-owned loader is acceptable.
- **(iv) Reverse check.** Guards the cancellation-drain strength: compute must be released only after the native thread joins (`jobs.rs:1428-1490`). Flights over native work must keep detached completion, so apply C8 only to pure preparation products.

### C9. Libraries that cannot own DA-001, -002, -003, -006 or -031

- **DA-001, resolved IR.** No in-scope library owns the *pre-admission* resolved IR. Symbolica atoms cannot carry physical types, binders or host guard semantics, and admission must precede Symbolica (`typed.rs:811-905`, the guard-survival strength).
  - What the libraries can own is the representation around a project IR: `typed-index-collections` `TiVec<SymbolIx, Symbol>` for an identity-indexed symbol table, rpds for a persistent scoped environment (C10), and C3 for post-admission formals.
- **DA-002, derived identities.** Salsa interning is process-local and unsuitable for persisted derived identities. The owner stays `pse-ids` catalogued frames plus relationships recorded in the model; no library.
- **DA-003, DA-006 and DA-031.** These are registry, codegen or compiler-ownership questions. Salsa can host the single numerical-policy lowering (DA-006) and a compiler-published "view choice" output (DA-031) as tracked queries, but the decisive work is project-owned.

### C10. Small library mechanisms for DA-029

- **rpds `HashTrieMapSync<SemanticId, Type>`** (or `RedBlackTreeMapSync`), maintained incrementally as specialization inserts symbols and exposed through `TypeLookup` (`environment.rs:8-16`). It replaces the per-site full rebuild at `specialize.rs:2576-2582`, with O(1) snapshots and O(log n) inserts. The checker already uses rpds the same way (`phases.rs:10`, `:589`).
- **petgraph reverse reachability.** One `Bfs`/`Dfs` over `Reversed` from the derivative-rate nodes of the member graph replaces the per-row fixed point (`executable.rs:614-655`). For the per-output member closure, use `algo::tred::dag_transitive_reduction_closure` on the toposorted DAG, or the existing `MemberReads` index.
- **Edges by identity.** Member-graph edges should come from resolved identities rather than parsed names (`executable.rs:532-541`); that waits on the DA-001 IR.
- **Reverse check.** None.

---

## 4. Clear-win improvements no DA finding names

1. **`hot_start` on evaluator rebuilds for the same atoms at a higher order or under a new optimizer profile** (`function_map.rs:415`). It reuses the Horner scheme found last time. *Proposed*, a construction-time win; it needs `Expression` access to the previous scheme.
2. **Use `count_operations_with_subexpression` and joint `evaluator_multiple` counts as the admission estimate before vectorization**, instead of the summed `source_operations` walk (`admission.rs:86-118`). That would make the budget CSE-aware and avoid refusing bodies that CSE would shrink. The repository's own comment notes that Symbolica 3.0.1's `count_operations` omits `Fun` nodes (`:104-107`), so it is an addition to that walk, not a replacement.
3. **`optimize_stack` after `map_coeff`** (`optimize.rs:504`), if exports show unreused temporaries. That reduces `numeric_entries` charged to worker scratch (`library.rs:362-367`). Probe with `ExportedInstructions::temporary_count` before and after.
4. **faer: keep `(SymbolicSparseColMat, Argsort)` once and refill with `new_from_argsort`** where per-call `try_new_from_triplets` appears in result and dynamics paths (`crates/pse-runtime/src/workflow/modeling/results.rs:672-704`; `dynamics.rs:597`, `:758`). Assembly already fixes its Jacobian pattern at plan time (`crates/pse-math/src/assembly_entries.rs:809-818`), so this is outside the hot path; minor.
5. **One structural route for HK/DM/BTF.** It runs in both `pse-structural::incidence` and `pse-math::implicit_reconstruction` (`:1473-1492`). If both analyze the same incidence, one call through `pse-structural` serves both. Unverified whether the inputs coincide.
6. **rustworkx `dag_algo::layers` over the member and body DAG** gives independent construction waves for bounded parallel body admission. This only matters if C1 makes bodies fewer but larger. *Proposed*.

---

## 5. Considered and not recommended, remedy open, and leads

- **Symbolica `IfElse` and conditional evaluators.** Moving `Branch` into Symbolica would evaluate both arms eagerly inside one program, which defeats lazy guarded evaluation and branch-continuity admission. **Rejected:** regresses guard survival.
- **`InliningPolicy::Never` and tagged-function sub-evaluators for shared members.** Incompatible with `Dualizer` (`admission.rs:1048`). **Rejected** while derivatives come from Taylor vectorization. Symbolic `derivative` per output followed by one evaluator would avoid the limit, but it changes the derivative route. Treat as an alternative to compare in Q1, not a default.
- **`merge(other, cpe_rounds)` to join per-output evaluators.** It requires an identical parameter count and layout. Per-output bodies use first-occurrence `input_<n>` layouts, so they would first need re-parameterizing to instance slots. It also runs only common-pair elimination, not full CSE, and does not join vectorized programs cleanly. It is weaker than C1; keep it only as a fallback if C1 group sizes break budgets.
- **Salsa `persistence`** for cross-process warm starts (P2). It needs serde on every value, including `AdmittedBody` with Symbolica atoms (`serde`/`bincode` off, and decoding needs the global state map). It would also be a second reuse store beside the durable record. Under P3 it could only be a tactical cache. That is a major addition with tradeoffs, so **not recommended** here; recorded as a lead.
- **Salsa `accumulator`.** No preparation obligation needs side outputs; receipts are already captured into the body (`typed_math.rs:355-378`). Not recommended.
- **Symbolica JIT (`jit_compile`, symjit).** It could replace g++ subprocess compilation (`native_compiled.rs:23-34`) for latency. However, symjit's floating-point contraction and FMA behaviour is unverified against the "never FMA / `-ffp-contract=off`" invariant, and the option keys are opaque (`backend.rs:458-469`). **Lead** until probe Q7.
- **`SymbolAttribute::Real` on formals.** It may simplify normalization (`conj`, `sqrt(x²)`), but it changes canonical forms and therefore every identity, with an unclear gain. **Unresolved**; not proposed.
- **Graphica canonization.** No obligation needs it.
- **Remedy open: C7 with durable lanes.** Stated in C7(iv).

---

## 6. Unresolved questions and probes (described only, none run)

| # | Question | Probe (command and expected observation) | Settles |
|---|---|---|---|
| Q1 | Per-output vs multi-output vs merge cost | In a scratch test in `pse-math`, take one PR-equilibrium instance (`packages/reference/thermodynamics/models/pr-equilibrium.pse`) and the `projection_tests.rs` replica fixture. Build: (a) today's per-output bodies, summing `count_operations()` after `Dualizer` at orders 1 and 2; (b) one `evaluator_multiple` over all group outputs with members substituted, then `Dualizer` with per-parameter `zero_components`; (c) per-output evaluators re-parameterized to instance slots, then `merge(…, Some(1))`; (d) symbolic `derivative` per output, then one `evaluator_multiple`. Report scalar and Taylor op counts, `temporary_count` and build time. Run via `scripts/pse-env -- cargo t -p pse-math <test>`. Expect (b) below (a) whenever members are shared; the Taylor-width effect stays to be measured. | C1 adoption and grouping policy; whether (d) beats Taylor for wide groups; DA-028 magnitude |
| Q2 | Alias-fused let chains | Build a deep `let` DAG with reuse (k uses per level, depth d). Compare `add_aliases` on the tree path vs `direct_translation(true)` vs today's materialized regions: construction time, peak atom bytes, op count. Also check that `export_cpp` accepts the fused evaluator (`compiled-evaluator`). Expect exponential growth on the tree path only. | C2 policy and threshold |
| Q3 | Demand-free specialization diff | Specialize one root with `demand=[]` and with `demand=[p]` and diff `SpecializedModel`. Expect only `symbols`, `paths`, lineage demand entries and `source_occurrences` to differ. | C4 split point; whether demand can leave `Request` |
| Q4 | Catalog firewall | The audit's probe: edit a declaration the live root does not reach, then count `compiler.query.*` and `WillExecute` events for `projection` (`workspace.rs:1511-1520`). Repeat with tracked accessors at `executable.rs:459` and `:1033`. Expect re-execution today and backdating with the accessors. | C5 |
| Q5 | Function-atom formals | Replace `formal(k)` with `function!(slot, k)` in a scratch body. Check that the evaluator export is identical up to parameter order, that `Dualizer`, `derivative(Indeterminate::Function)` and `export_cpp` accept it, and record `get_byte_size` deltas. | C3 |
| Q6 | Input `heap_size` | Add `heap_size` to the granular inputs in a scratch branch. Compare `retention_usage()` with today's `known_bytes` on `granular_tests.rs:739`. Expect equal or closer bytes and no census. | C6 |
| Q7 | symjit floating-point semantics | Disassemble a JIT `a*b+c` (`JITCompiledEvaluator::as_bytes`) and look for `vfmadd`. Check symjit's `Config` options for a contraction switch. | JIT lead |
| Q8 | Shared Salsa handles | Two threads on cloned `CompilerDatabase` handles run `specialized` for distinct `Request`s on one revision; a third handle sets `Catalog`. Expect shared memos and `WillBlockOn` on overlap, with the setter cancelling both readers. | C7 fit and its open remedy |
| Q9 | Zero-component folding | Count Taylor instructions for an output depending on 2 of 50 coordinates, with and without `zero_components`. | C1 width cost |

**Search limits.**
- "Unused" here means not found by `rg` in `crates/*/src` (from the census, spot-checked) for the named APIs.
- I did not read `pse-math/src/assembly.rs` `CaseWorker` internals beyond the audit's citation.
- I did not read dynamics or fit preparation, so the preparation side of DA-014 is covered only by this: `PreparedModeling::rebind` with `values_match` premises (`solve.rs:463-550`) and `IntegratedExperiment::bind` are the value-only mechanisms. No library in this family adds to them, and value binding must stay outside Salsa.