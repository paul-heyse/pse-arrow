> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, A3 (code-mapper): structure, decomposition and evaluation capability, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

# A3 review evidence: structure, decomposition and evaluation capability

Everything below was read from the working tree and nothing was run, so no claim is *Tested*. Claims are *Interface-checked* unless marked *Proposed*. Paths use BN, RT and PM as defined in the assignment.

## Summary

- **Structural products.** Structure is computed once per case, in compiler preparation. `StructuralAnalysis` holds a maximum matching, the Dulmage–Mendelsohn over/under/square parts, and BTF blocks ordered prerequisite-first, each with a `coupling` class. It holds no linearity, degree or per-block type information. The `ProbeView` handed to pounce-presolve passes `linearity: None` (`incidence.rs:362`).
- **Block routing.** Per-block routing ignores linearity. `oracle_facts` hard-codes `affine_rows = vec![false; …]` (`routing.rs:675`). Every square block therefore routes to the SquareRoot class and KINSOL, with the default `LineSearch + Klu` (`settings/kinsol.rs:125-140`). Nothing sizes, linearises or tears a block.
- **No assignment or tearing.** No symbolic "solve for variable", explicit-assignment detection or causalization exists in compiler, math, structural or runtime source. No Modelica-style equation tearing exists inside a nonlinear block. The only tearing is flowsheet tearing: a HiGHS MILP or a greedy feedback-arc heuristic over `FlowGraph`.
- **Block execution.** Two routes exist.
  - The automatic ladder's `Block` mechanism runs a serial BTF sequence on one backend (`PreparedBlocks.backend`). That is the P-C4 code.
  - Staged initialization resolves one `Route` per block (`PreparedInitialization::strategies`), but all blocks share one `SolveSettings`.
  - Each block restarts from authored guesses, with `warm: None` and "Allocation retention never substitutes a previous solved root".
- **Recycles.** A recycle is a KINSOL fixed-point map with optional Anderson acceleration and damping. Units are opaque causal maps, so the map has no Jacobian and units are refused when a sensitivity is requested. Newton and Broyden on tear variables are impossible by construction. No design-spec-over-recycle layer (nested vs simultaneous) exists. A unit with its own residuals and unknowns is solved inside the sweep (conditional realization). Anything else is an explicit map. A simultaneous alternative is the ordinary Direct solve.
- **Decomposition.** `automatic_separator` is not a graph partitioner. It selects columns shared by more than one instance, plus columns that cross BTF blocks through execution dependencies. It reaches POUNCE only for `Backend::Pounce` with `CompositionPolicy::Auto`. The shooting and fitting NLP oracles do not set a separator, so they never use the Schur path. This is *Interface-checked* by a search of `with_solve_separator`.
- **Evaluation capability.**
  - Derivative orders are `Value`, `First`, `Second`.
  - A Hessian is an assembled sparse lower triangle; no Hessian-vector product exists.
  - First directional actions (one Taylor axis) are demanded only by KINSOL Krylov routes. This closes F16 for KINSOL: the matrix-free Krylov Newton route needs no assembled Jacobian. IDAS `jtimes` still assembles the partials.
  - Evaluators are interpreted f64 (`no-jit;no-simd`), keyed per template body hash, with instances bound to slots.
- **Studies.** Continuation edges are authored. Start proposals use Zeroth, Root-tangent, KKT, Activity, QP and Secant mechanisms, each screened. There is no adaptive step bisection or automatic case ordering in studies. Adaptive step halving exists only in staged homotopy.
- **Probes.** Section 7 lists candidates for every class.

## 1. Structural analysis

### What is computed and when

- `CaseIncidence::analyze` (`pse-structural/src/incidence.rs:302-575`) runs on a dedicated 32 MiB-stack thread. It builds `EqualityIncidence` from a `ProbeView`, then runs Hopcroft–Karp matching. `DulmageMendelsohnPartition::from_matching` follows, then `SquareComponents` and `BlockTriangularForm::of_component`. Each library result is cross-validated: matching consistency, partition coverage, and BTF order against `inc.neighbors`.
- Final block order is `rustworkx_core::lexicographical_topological_sort` over block dependencies, with ties broken by minimum row id.
- Products (`StructuralAnalysis`, `incidence.rs:92-109`):
  - `matching`: one witness only, documented as "not canonical".
  - `over`, `under`, `square`: `Part` values holding row and column semantic ids.
  - `blocks`: `Block { id, members: Part, coupling }`. `coupling` is `AuxiliaryCouplingClass`, one of PureEquality, ObjectiveCoupled, InequalityCoupled, ObjectiveAndInequalityCoupled.
  - `contributions`: original instance/output incidence.
- Block size is only `members.rows.len()`.
- The `coupling` field has no consumer in `pse-backend-native`, `pse-runtime` or `pse-compiler`. A grep found only the producer in `incidence.rs` and test fixtures.
- Callers that use block membership:
  - the Block ladder rung;
  - the KINSOL `BlockFactor` preconditioner, which uses `analysis.blocks[].members` (`kinsol/block.rs:84-123`);
  - the separator in `automatic_separator`.

### Timing

- The compiler builds the whole-case analysis during case preparation. It is stored in `PreparedCase.structure` (`pse-compiler/src/workspace.rs:~327`, with a conditional-partition variant at ~378).
- The runtime carries it with `AlgebraicOracle::with_structural_analysis` (`solves.rs:~4628`).
- Native admission re-checks it with `crate::structural::check` (a class-aware refusal, with `Mode` Roots / Nlp / NativeFeasibility) before solving.
- Callback oracles with no witness trigger a fresh analysis (`structural::oracle_structure_with_cancel`).
- The block schedule (`automatic_alternatives`, `workspace.rs:601-658`) is derived lazily on the runtime worker. It is `Unavailable` if the case has native constraints, an objective, or any non-equality row.

### Initialization `Plan`

- `Plan::from_analysis` (`initialization.rs:124`) refuses any over or under part and any non-square block.
- It computes predecessor `inputs` per block and checks that BTF order respects dependencies.
- `Plan::with_execution_dependencies` (`initialization.rs:32-122`) adds authored execution and validity edges, computes `kosaraju_scc` over blocks, and merges each cycle into one block with `members` re-sorted. It reuses the unchanged numerical witness; the unit test says "No invented numerical matching/rank".
- A block therefore carries `{id, members, inputs}` only.
- This merge is control-cycle handling. It is not a tear and not a nonlinear decomposition.

### Symbolic solve, assignment detection, tearing

- *Interface-checked absence.* I searched `pse-math`, `pse-compiler`, `pse-structural`, `pse-runtime` and `pse-backend-native` for `explicit.?assignment`, `solve_for`, `causaliz`, `tearing`, `solve_linear` and `linear_solve(`. Hits were unrelated: `resolve_for` in `dynamics.rs`, and comments.
- Plan 36c P-C4 states the intent: "A scalar implicit equation is not an explicit assignment; direct execution requires an actual explicit expression/proof" (`36c…md`, P-C4). I found no code for the direct path.
- What does exist is form-recognizing solving of authored implicit functions (`PM/implicit*.rs`), not of generic blocks:
  - `implicit_affine.rs`: exact affine elimination by faer sparse LU.
  - `implicit_cubic.rs`: Symbolica coefficient extraction with certified roots.
  - `implicit_accelerators.rs`: `Accelerator::admit` is a registration trait that recognizes a residual form and returns an `InnerSolver`.
  - `implicit_selection.rs`: `graph_equivalence` returns NondegenerateAffine / RestrictedSquareRoot, with a scalar-only restriction (`unknowns != 1` returns `Unestablished`).
  - These are the only places where structure picks a method. They use Symbolica form recognition and IFT derivatives (`implicit.rs:462 derivatives`).
- Library presolve (pounce-nlp) runs auxiliary-block elimination and affine linear-equality elimination, but pse refuses the `Aggressive` auxiliary-coupling policy (`presolve.rs:234`). Only the Auto/Explicit policy is selectable.
- Row-level affine proofs already exist (`pse_math::presolve::Facts.affine: Vec<Option<AffineRow>>`). They are not joined to the BTF blocks. `PreparedBlock` carries `{class_proof_work, boundary, plan, structure, artifacts}` (`workspace.rs:1022-1033`) and no facts.

### Proposed (not implemented)

- *Proposed.* Attach a per-block fact record: size, all-rows-affine (from `Facts.affine`), block-triangular depth, and coupling class. A single-row, single-variable block with a proved affine or cubic form would use the existing `implicit` accelerators. All-affine blocks could use a direct sparse LU. Neither exists today.

## 2. Block execution

### Automatic-ladder `Block` mechanism (P-C4 code)

- Eligibility (`blocks.rs:123-163` `automatic_block_count`) requires all of the following:
  - `StartPolicy::NoPriorStart`;
  - no sensitivity request;
  - `ConvexityPolicy::Exact`;
  - no explicit presolve;
  - a Roots or Nlp representation;
  - an algebraic source;
  - `schedule.blocks.len() > 1`.
- `prepare_blocks` (`blocks.rs:173-316`) builds one `PreparedSolve` per block with `profile.presolve = Off`, `intent = Initialize` and `reuse = Fresh`. It calls `prepare_conditional(…, Route::Native(backend), …)`, where `backend` is the original's single backend.
- `blocks_step` (`blocks.rs:322-…`) runs each block with `run_step`, calls `commit_block` after each (independent original-quality check), and clears `retained`. After all blocks it does a fresh full-original `constant` assessment. A candidate refusal aborts the whole attempt as an `EffectFailure::component`, with the failed component's observation attached.
- Per-block reports are stored in `report.components`.

### Staged initialization

- `PreparedInitialization::strategies` (`math/initialization.rs:~48-118`) builds a `Vec<Route>` by calling `Requirements::select(solver.selection)` per block. The facts are the block's contract (variable count, bounds shape, derivative order) and its own structure witness.
- For an Auto selection, the problem class is SquareRoot for every well-posed block, so routing is always KINSOL. It differs per block only through an explicit adapter choice, a capability mismatch such as bound shape, or the settings check.
- `validate_profile` (`:~126-210`) requires that "typed backend settings must belong to every block's route". There is one `solver.backend` setting for all blocks, so one method choice applies to all blocks.
- `ConditionalUnitSession::evaluate` (`:727-805`) calls `execution::roots` with `warm: None` and `initial: &self.initial`. Its inline comment says allocation retention never substitutes a previous solved root. Per-iteration warm starting across recycle sweeps does not exist.

### KINSOL block use

- `BN/kinsol/block.rs` is not a block solver. It builds `Blocks` from the structural analysis and factors each BTF diagonal block with `feral::SparseLu::factor_markowitz`. It then serves `Preconditioner::BlockFactor` as a right preconditioner for a Krylov solve (`solve` applies `ftran` per block). Header: "library sparse LU, used only as a right preconditioner".
- The whole-system Krylov iteration (GMRES, FGMRES, BiCGStab, TFQMR) therefore supplies the coupling between blocks. This is the one place the BTF structure steers a linear-algebra choice inside a single solve.
- The memory estimate for this route is conservative (a cubic term per block, `checked_extent`).

### What decides per-block strategy today (§17.1)

- The route is static and class-based: `problem_classes` → adapter `automatic_classes` and rank (`routing.rs:1198-1260`).
- The ladder order is fixed in `automatic_operations` (`solves.rs:~650-790`): prepared rungs, then Causal MapsAnderson, then ReducedSpace, then Block (if count > 1), then Direct.
- `next_automatic` (`strategy.rs:~75-165`) takes the first unattempted admissible candidate and does not score cost.
- An alternative decomposition is also in the ladder: the PETSc `NonlinearAdditiveSchwarz` over the same BTF blocks (`solves/petsc.rs:825` `prepare_petsc_blocks`, `BN/petsc/blocks.rs` `BlockOracle`).
- The ReducedSpace rung is created only when `automatic_reconstruction_accuracy()` is present. For complete reconstruction it needs no NLP. Otherwise it selects an NLP adapter and forces `HessianMode::LimitedMemory` when the Hessian is Auto (`solves.rs:~758`).

## 3. Flowsheet recycles

### Tear selection

- `FlowGraph` (`flowsheet.rs`) is a physical graph: nodes, connection occurrences and decision groups carrying `cost` and `Policy` (Free / Mandatory / Forbidden).
- `unweighted_heuristic` (`:489`) uses petgraph `greedy_feedback_arc_set`, or a forbidden-backbone toposort if any decision is Forbidden.
- `BN/tears.rs` `compile` is the HiGHS MILP (ADR-0179). It has SCC-local order variables and rows, globally shared decision columns, and an objective of authored `cost` (not stream size or Jacobian size). `solve` and the heuristic both run an independent petgraph acyclicity witness.
- `RecycleRequest::from_model` (`requests.rs:75-97`) takes explicit tears or, if absent, the unweighted heuristic. The MILP route is not the default there.

### Convergence

- `CausalMap::new` (`recycle.rs:142`) validates the unit inventory against the graph witness. It orders units, propagates non-tear connections with a scale and offset conversion, and builds the tear-to-tear map. The tear residual is predicted − x.
- `CausalMap` implements `FixedPointOracle`. The doc on line 3 says no recycle iteration algorithm lives here; KINSOL owns it.
- Control surface:
  - `RecycleRequest { tears, units, anderson: usize, damping: Fraction }`.
  - KINSOL fixed-point settings: Anderson history, orthogonalization, delay (`anderson_delay`), `max_newton_step` and damping. Newton-step options are refused for the map.
- Fixed-point iterations refuse bounded inputs.
- The original Jacobian pattern is declared conservatively (opaque unit support, `recycle.rs:~290`), but the iteration is Picard/Anderson only.

### Specs, nesting, sensitivities

- *Interface-checked absence.* I searched `RT/workflow/strategies`, `RT/math/flows.rs`, `recycle.rs`, `tears.rs` and the conditional modeling code for `design.spec`, `nested`, `wegstein` and `broyden`. Hits were only unrelated nested-disjunction and nested-implicit tests. No outer loop around a recycle exists.
- A design specification currently has two forms. It is either an equation and unknown inside a unit (`CausalUnitRealization::Conditional { residuals, unknowns }`), solved in the inner sweep, or part of a whole-flowsheet simultaneous Direct solve. The causal-map admission refuses undeclared parent coupling.
- `admit_unit_profile` (`math/initialization.rs:~810`) refuses `profile.sensitivity.is_some()`. Per-unit sensitivities are therefore unavailable inside the sweep.
- A Root-solve IFT exists, but at whole-solve level only: `square_response` / `root_response` (`solves.rs:4751`) and `fresh_root_predictor_at`. It gives parameter sensitivities and a sparse factor (P-C5) for a regular square closure.
- *Proposed.* A unit's JVP via the implicit-function theorem is feasible from a conditional unit's own Root solve (the unit session already has a KINSOL oracle with a Jacobian). That would enable Newton or Broyden on tear variables. It is not wired.

### Conditional units

- Retention in `conditional.rs` / `math/initialization.rs` `ConditionalUnitSession` is allocation retention only. It never carries a numeric solution forward.
- Plan 36c P-C4 prohibits silent warm starts: "Previous committed roots become declared recorded starts only when existing uniqueness/branch policy admits them".

## 4. Decomposition for optimization and estimation

### Separator

- `PreparedCase::automatic_separator` (`workspace.rs:530-595`):
  - It starts with free variables whose slots appear in more than one instance.
  - It adds, from `plan.dependencies`, any free variable used by a row or objective when the owning BTF blocks differ.
  - It returns `Part { rows, columns }`, with rows being dependency targets touching the separator. The doc says it is "a performance projection, never a numerical rank or elimination proof".
  - It does not minimise the separator and does not balance block sizes.
- Reaching the solver: `solves.rs:4629-4666` adds `SolveSeparator` only when `run.adapter.backend() == Backend::Pounce` and `profile.composition.policy == CompositionPolicy::Auto`.
  - `BN/presolve/pipeline.rs:659` carries the separator through the presolve transport.
  - `BN/pounce/observed.rs` maps it to POUNCE and records observed metrics: `linear.schur.actual_use`, `linear.schur.final_path`, `linear.schur.path_reason`, and per-path `factor_attempts` for `monolithic`, `schur.f` and `schur.s`.
  - `schur_tests.rs` shows an unsuitable separator or failed factorization falls back to monolithic.
- A search for `solve_separator` found implementations only in generic wrappers (transform, structural, callback, presolve, assembled, pounce) and none in `RT/workflow/fitting` or `shooting.rs`.

### Fitting

- `RT/workflow/fitting` composes `PreparedExperiments` (Steady and Transient) into a single sparse NLP. `sparse::Layout` holds fixed response, constraint and Hessian assembly matrices with per-experiment `Mapping`, and shares parameter columns across experiments by addend.
- A Gauss–Newton Gram `JᵀWJ` is built with faer (`GramWorker`, `HessianMode::GaussNewton`). Transient experiments take gradients from forward sensitivities and from the adjoint (test: `adjoint_gradient_equals_forward_on_transient_fit`).
- Steady experiments are solved as implicit closures (`steady_response_solves_the_compiled_implicit_closure`).
- Multi-experiment independence is not exploited as a decomposition: the experiments sit in one NLP and POUNCE Schur is not set.

### Multiple shooting

- `workflow/shooting.rs` (header lines 3-16): single and multiple shooting. Multiple shooting integrates one window per node interval, anchored at its start state, with differential states at inner nodes as variables closed by continuity rows. Gradients are forward sensitivities or the adjoint (a documented 'one adjoint per window'). The NLP uses `HessianMode::LimitedMemory`. The block-angular structure is not passed to POUNCE Schur.

### Reduced space / nonlinear elimination

- `ReducedSpace` is `MechanismKind::ReducedSpace` / `FamilyKind::Reduced { reconstruction }` (`pse-math/src/derived.rs`). It is a derived family with a complete-reconstruction contract.
- Authored implicit functions (`PM/implicit.rs`) provide nested elimination with IFT derivatives. They give a reduced-space formulation only when the author declares an implicit unknown set.

### Conic

- There is a chordal decomposition test: `clarabel_chordal_matches_undecomposed` in `clarabel_tests.rs`. I did not read its implementation.
- No MINLP or LP block decomposition (Benders, Dantzig–Wolfe) was found in the files I searched.

## 5. Evaluation capability

### Derivative orders

- `DerivativeOrder { Value, First, Second }` (`pse-kernels/src/lib.rs:172`). No third order exists.
- `CasePlan` holds an assembled sparse Jacobian and lower-triangle Hessian (`hessian: Arc<AssemblyMatrix>`, `assembly.rs:98`) plus `jacobian_terms` / `hessian_terms` addend lists.
- Order upgrade is demand-driven (`prepare_order`; P-C2).

### Products without forming J or H

- `RootOperations { jacobian, jacobian_product }` (`lib.rs:480`) and `NleOracle::jacobian_product`.
- *Interface-checked.* `CaseWorker::jacobian_product` (`assembly.rs:2285`) branches on `has_directional_actions()`.
  - If true, it runs a directional program: one Taylor axis (`compile_directional`, `execution.rs:680`), compiled per body. The demand is the `Directional` group (`assembly.rs:879 with_directional_actions`).
  - Otherwise it assembles the Jacobian and multiplies.
- Runtime demand: `consumes_jvp()` is true when linear is Krylov and the strategy is Newton or LineSearch (`settings/kinsol.rs:145`). `solves.rs:~3297` then calls `prepare_directional_actions`. `consumes_directional_only()` skips Second/First assembly entirely when there is no preconditioner.
- F16 therefore holds for KINSOL only when the Krylov JVP and the directional program are used. `Preconditioner::Jacobi` or `BlockFactor` still needs an assembled First (`routing.rs` test `krylov_product_preparation_is_distinct…`). The KINSOL default (`Klu`) assembles the Jacobian.
- No Hessian-vector product exists. A search of `crates` for `hessian_vector|hvp|HessianProduct|hessian_product` found nothing. Limited-memory (`HessianMode::LimitedMemory`, quasi-Newton) is the only Hessian-free option for Ipopt and POUNCE; Uno has `limited_memory`.
- IDAS `jtimes` (`idas.rs:612`) calls `c.partials(t, &x)` and multiplies, so it forms the Jacobian first. It is not a Symbolica directional product.

### Template specialization

- `CasePlan.bodies: BTreeMap<ContentHash, Arc<PreparedBody>>`. Artifact keys (`workspace.rs:~440 artifact_requests`) hash `body`, order, outputs, coordinates, directional flag, optimization and evaluation limits, and the string `interpreted-f64;numerica-jets;real-algebra;no-jit;no-simd`. Programs are therefore per template body, with instances bound by slot. Outputs and coordinates are part of the key, so support is specialized per demand.

### Callback cost shape

- `CaseWorker::constraints` (`assembly.rs:2162`) loops over all instances and their contributions, allocates `vec![0.0; rows]` each call, and ends with a `finite` pass.
- `BN/callback.rs` is the classification and accounting shim (`classify`, `evaluation_unit`). It does no caching. Caching lives in the worker's trial state (`invalidate_trial`, `EvalInputs`).
- The evaluator is interpreted. There is no JIT and no SIMD in the identity string.
- Cost therefore scales with instance count, not with the number of distinct templates.
- *Proposed.* Measure with the native statistics (§7), because I cannot state a per-instance cost.

## 6. Studies and sequences

- Start vocabulary (§17.4 / `pse_model::study::StartPolicy`): `Fresh`, `Explicit { role, seed }`, `Continuation(edge)`. The edge's predecessor is a study point; edges are authored, not derived.
- `continuation_start` (`RT/workflow/study_execution.rs:1085`):
  - If recovery allows `StartOrigin::Predicted` and the start policy is not Explicit, the proposal comes from `available_prediction(older, target, branch, execution)` (`modeling/results.rs:2186`). The proposal is screened (`screen_start`) before use.
  - Otherwise it uses the predecessor's seed.
  - Proposal mechanisms (`math/prediction.rs:416-440`): Zeroth, Root (tangent via the sparse Root factor), Kkt, Activity, Qp, Secant (`SecantHistory`).
- Retention (`ContinuationRetention`, `:339`, `:1297`) keeps at most the selected predecessor and one older, to support secant extrapolation.
- *Interface-checked absences.* A search of `study*.rs` and `pse-model/src/study.rs` for `halv|bisect|adaptive` found nothing. The study scheduler does not reorder cases or bisect a step.
- Adaptive halving lives in staged homotopy (`staged.rs`, growth 1.5; spec §17.5: "a retryable homotopy failure halves the step, down to the minimum step").
- Studies run points as `FuturesUnordered` with a dependency graph, so continuation chains are ordered but fan-out is parallel.

## 7. Probe candidates

Command patterns come from the justfile (`unit-native-package pkg features filter`, line ~1317; `unit-package`, line 480). I did not check each test's feature gate or runtime, so treat the feature list as *Proposed*. Backend features are named in `pse-backend-native/Cargo.toml` (`kinsol`, `ipopt`, `pounce`, `highs`, `scip`, `idas`, `diffsol`, `sdp`).

| Class | Test (path, name) | Pattern |
|---|---|---|
| Square nonlinear (KINSOL) | `BN/kinsol.rs`: `kinsol_krylov_variants_solve`, `kinsol_dense_spgmr_picard_solve` | `just unit-native-package pse-backend-native kinsol kinsol_krylov_variants_solve` |
| KINSOL + BTF preconditioner | `BN/kinsol/block.rs`: `actual_kinsol_block_factors_solve_coupled_original_and_rebuild_changed_sources` | same, filter `actual_kinsol_block_factors` |
| Blocks (ladder) | `RT/math/solves/derived_tests.rs`: `automatic_blocks_execute_complete_nonport_coupled_original_and_keep_actual_components` | `just unit-native-package pse-runtime solver-kinsol,… automatic_blocks_execute` |
| Recycle / tear | `BN/tears.rs`: `tear_component_native_optimum_charges_shared_group_once`; `BN/recycle.rs`: `causal_map_uses_declared_tears_and_recomputes_original_connection_residual`; `RT/workflow/strategies/conditional/integrated_journey.rs`: `indexed_mixer_holdup_separator_executes_conditional_recycle_and_external_closure` | same pattern, `pse-backend-native highs,kinsol` and `pse-runtime` |
| Smooth NLP, Ipopt | `BN/ipopt.rs`: `native_ipopt_preserves_tight_rows_with_independent_kkt_budgets` | `… ipopt …` |
| Smooth NLP, POUNCE + Schur | `BN/pounce/schur_tests.rs`: `actual_schur_assembled_profiles_and_typed_mc64_quality_reach_factors` | `… pounce …` (reports `linear.schur.*`) |
| LP / QP (HiGHS) | `BN/highs.rs`: `highs_qp_with_linear_columns_verifies_its_upload` | `… highs …` |
| MILP | `BN/highs/mip_tests.rs`: `highs_incumbents_streamed`, `mip_node_budget_independent` | `… highs …` |
| Conic | `BN/clarabel_tests.rs`: `clarabel_lp_matches_highs`, `clarabel_chordal_matches_undecomposed` | `… sdp …` |
| MINLP | `BN/scip_tests.rs`: `small_synthesis_minlp_optimal` | `… scip …` |
| Dynamics | `BN/dynamics/adjoint_tests.rs`: `idas_adjoint_gradient_equals_forward_and_differences`, `diffsol_adjoint_gradient_equals_forward_and_differences` | `… idas,diffsol …` |
| Fitting | `RT/workflow/fitting/p09_tests.rs`: `gauss_newton_fit_admits_transient`; `RT/workflow/fitting/oracle.rs`: `nlp_runner_serves_solve_initialize_fit` | `just unit-native-package pse-runtime … gauss_newton_fit` |

Benchmark route: `benches/benches/native_process.rs` is driven by the `PSE_PROCESS_COST_SPEC` and `PSE_PROCESS_COST_OUTPUT` environment variables. Operations are heater, flash, vessel, fit, k4-study, k4-restart and k4-accuracy. The compile check is `just check-native-process-bench`; I did not find a recipe that runs it. Treat it as a secondary source because its spec shape needs reading first.

### Probes I did not run (to settle open doubts)

1. **Does Krylov Newton avoid assembling the Jacobian?** Run `kinsol_krylov_variants_solve` with default Krylov and `Preconditioner::None`, with native statistics at maximum verbosity. *Expected:* a nonzero JVP count and zero Jacobian refills. If Jacobian refills are nonzero, the directional program was not demanded.
2. **Does a linear BTF block waste Newton work?** Run `automatic_blocks_execute_complete_nonport_coupled_original_and_keep_actual_components`. Check each component's iterations and factorizations. *Expected:* at least one block iteration for a trivially affine block, which would support the "linearity is ignored" claim.
3. **Does recycle warm start exist at all?** Run `indexed_mixer_holdup_separator_executes_conditional_recycle_and_external_closure`. Compare per-sweep unit iteration counts. *Expected:* flat counts across sweeps, as "no warm start" predicts.
4. **Does the Schur path pay off?** Run `actual_schur_assembled_profiles_and_typed_mc64_quality_reach_factors`. Check `linear.schur.actual_use`, `schur.f.factor_attempts` and `schur.s.factor_attempts` against the monolithic path.

## Caveats

- I did not read these, so the following claims stay open:
  - the dynamics integrators' own structure handling (IDAS/Diffsol index handling or sparse decomposition);
  - the Clarabel chordal implementation;
  - the Uno backend;
  - `RT/math/prediction.rs` beyond its mechanism enum;
  - whether `Block.coupling` is consumed outside the three crates searched.
- Which ladder rung is chosen at runtime also depends on `strategy.rs` admission and `StartRules` (A1/A2 scope). I recorded only the static order.

## File index

- `crates/pse-structural/src/incidence.rs`
- `crates/pse-structural/src/initialization.rs`
- `crates/pse-structural/src/flowsheet.rs`
- `crates/pse-runtime/src/math/solves/blocks.rs`
- `crates/pse-runtime/src/math/initialization.rs`
- `crates/pse-runtime/src/math/solves.rs`
- `crates/pse-runtime/src/math/strategy.rs`
- `crates/pse-runtime/src/math/prediction.rs`
- `crates/pse-runtime/src/workflow/strategies/requests.rs`
- `crates/pse-runtime/src/workflow/strategies/conditional.rs`
- `crates/pse-runtime/src/workflow/study_execution.rs`
- `crates/pse-runtime/src/workflow/shooting.rs`
- `crates/pse-runtime/src/workflow/fitting/oracle.rs`
- `crates/pse-compiler/src/workspace.rs`
- `crates/pse-backend-native/src/kinsol/block.rs`
- `crates/pse-backend-native/src/recycle.rs`
- `crates/pse-backend-native/src/tears.rs`
- `crates/pse-backend-native/src/routing.rs`
- `crates/pse-backend-native/src/pounce/observed.rs`
- `crates/pse-math/src/assembly.rs`
- `crates/pse-math/src/execution.rs`
- `crates/pse-math/src/implicit.rs`
- `docs/plans/36c-mathematical-and-native-execution.md`
- `docs/authoritative_design/sections/numerical-execution.md`