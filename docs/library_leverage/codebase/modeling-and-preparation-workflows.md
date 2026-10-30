# Modeling and preparation workflows

This page covers the runtime workflow layer that turns an authored modeling package into prepared and executed
scientific operations (`crates/pse-runtime/src/workflow/modeling.rs` and `workflow/modeling/`) and the runtime
effect boundary around the compiler and mathematical products (`crates/pse-runtime/src/math.rs` and `math/`).
Contracts are owned by [models and composition](../../authoritative_design/sections/models-and-composition.md) and
[numerical execution](../../authoritative_design/sections/numerical-execution.md). The baseline is on the
[entry page](README.md). Mechanisms below are Interface-checked by static reading; every value claim is Proposed,
and nothing was built, run or measured.

**System picture.** The modeling workflow layer is about 68.6k lines, about 41.5k production and about 27.2k test.
It covers source ingress and exact-parent deltas (`canonical*`), project comparison and merge, case resolution
(`cases`, `implicit`, `views`), dynamic preparation and trajectories, staged initialization, diagnostics,
data-authored conformance and result projection. Consumers are the workflow operations, the CLI and native host,
fitting, studies and shooting. The runtime math layer is `MathService`: preparation lifetimes (artifact, scalar,
frontier and basis flights, retained products, rebind), admission and residency (a DataFusion `MemoryPool`,
semaphores, retained native lanes), solve preparation, the numerical strategy driver, and original-point scientific
assessment. It does not implement solver iteration.

Library use is deep where it exists. Result projection goes through registry-generated builders and cursors rather
than hand Arrow. Source revisions are `rpds` persistent maps with cheap branching. Reload delegates incremental
parsing to `pse_admission`. Linear diagnostics call HiGHS IIS, rays and ranging directly; infeasibility
certification uses SCIP; point uncertainty uses IBEX with library-reported workspace. The caches use DataFusion's
cache traits and `quick_cache`. Cancellation is completion-owned, and ordered conformance merge is careful and
tested. Opportunities cluster in four places: a hand-built dependency tracker beside the compiler's Salsa database
(CBO-087); long orchestration bodies that interpret a compiler product by shape (CBO-088, CBO-089, CBO-090);
stable preparation rebuilt at the wrong lifetime (CBO-096, CBO-097, CBO-098); and mechanisms with no production
consumer (CBO-099).

**Coverage on this page.** Read closely: `dynamics.rs` production (`prepare_simulation_mode`, `state_signs`,
`integrate_sample_attempt`), `dynamics_accuracy.rs`, `dynamics_checks.rs`, `dynamics_events.rs`, `trajectory.rs`,
`cases.rs`, `implicit.rs`, `views.rs`, `canonical.rs` and `canonical/{delta,reload,local,rebind}.rs`,
`project_comparison.rs`, the linear and nonlinear diagnostics, and in the runtime math layer `staged/driver.rs`,
`solves/paths.rs`, `solves/{causal,multistart,starts}.rs`, the accuracy files, `strategy/admission.rs` and
`native_child.rs`. Read in part: `conformance.rs` (report projections outlined), `declared_requests.rs` (first part), `results.rs`, `diagnostics.rs` (second half unread), `solves.rs`, `solves/petsc.rs` (block preparation and
setup partly unread), `portable.rs`, `retention*`, `modeling.rs` and `surrogate*`. Skimmed or outlined only:
`project_preparation*`, `documents.rs`, `knowledge.rs`, `engagement.rs`, `declared.rs`, `engines.rs`,
`pure.rs`, `flows.rs` and `settings.rs`. Test files were classified by name and sample, not read. The numerical
accuracy logic in `dynamics_accuracy.rs` beyond intervals, hash field completeness and `force_validate` activation
were not checked, and no performance was measured.

<a id="cbo-099"></a>

## CBO-099 — The durable body tier and child route have no production consumer

- **Operation and consumers.** Publish an admitted mathematical body to the canonical store and reconstruct it in
  another process, so a restarted runtime or a study worker avoids parse, admission, specialization,
  differentiation and evaluator construction for a model compiled elsewhere. A private child process solving a root
  from the same recipe is the second consumer.
- **Current mechanism.**
  - In `math/portable.rs`, every reference outside the file to `publish_body`, `publish_unqualified_body`,
    `reuse_body`, `reuse_body_current`, `describe_body`, `product_candidate_presence` and `qualify_replay` is in a
    `#[cfg(test)]` module. `workflow/canonical.rs` imports only `ReplayAdmission`.
  - `publish_body` and `reuse_body` are called only from test modules of `math/portable.rs`. `pse-worker` and
    `pse-project` compose `CanonicalDeployment::new(.., None)`; `CanonicalDeployment::producer()` has one caller,
    `workflow/study_tests.rs`.
  - Production still pays the capture: `pse-compiler` `typed_math/direct.rs` `Request::admit` runs
    `portable::capture` on every admitted body and keeps the payload on `AdmittedBody.portable`, falling back to
    `None` at a byte limit.
  - `native_child::solve_portable_root` has no caller outside `native_child*.rs` in `pse-runtime/src` and `pse-host`.
    `Key::EncodedBody` retention serves only this tier. By a method-call search, `MathService::prepare_path` and
    `prepare_multistart` have no production caller, and `prepare_surrogate` only tests.
- **Why it matters.** Either cross-process reuse of compiled mathematics is an intended activity that needs a
  consumer, or about 1.7k production lines in `portable.rs`, the operations-side selection, the eager capture and
  the child route serve no path. A second lead sits beside it: `ScalarProducts` hold exact evaluators that a fresh
  process rebuilds through Horner and CSE, and the supplier declares serde features on its evaluator types that the
  manifest does not select.
- **Patterns.** LOP-X020, LOP-12, LOP-07, LOP-11.
- **Evidence.** Searches over `crates/` and `xtask` `*.rs` for the portable symbols; `pse-worker.rs` composition,
  `canonical.rs`, `direct.rs` `admit`, `qualify_replay` and `reuse_body_current` read. (Interface-checked.)
- **Footprint.** Inspected as named. `pse-operations/src/canonical_selection.rs` by search only.
- **Preservation constraints.** ADR-0164 authority (`from_canonical_product`, unsafe mint), producer attestation,
  strict re-admission in `reconstruct_portable`, D14 (a digest never substitutes for semantic admission), the symbol
  registration-order universe (§14.3), and no native handle in durable state.
- **Counterevidence and uncertainty.** This is an absence claim by name search, limited to `crates/` and `xtask`;
  indirect dispatch and external embedders of a `pub` API are invisible. Tests exercise the tier thoroughly, so it is
  implemented and unconsumed. Reconstruction repeats evaluator construction, so the saving may be small, and no
  compile timing was read.
- **Status.** Current promise in the architecture and in Plan 28b's Outcome, with the implementation unconsumed in production. The overview says the substrate stores portable compilation descriptions, §18.8 refers to "the implemented child route", and Plan 28b records "consumer migration" as implemented. That conflict between owners and code belongs to the plan-completion route as well as to leverage.
- **Research questions.** What would a warm restart save against reconstruction cost, and is the store the right
  layer? Can the supplier's exported instruction forms or serialized evaluators be persisted and reloaded, and how
  does Salsa persistence stand at the pinned version?

<a id="cbo-088"></a>

## CBO-088 — Dynamic preparation interprets the compiler's product in orchestration

- **Operation and consumers.** `ModelingPackage::prepare_simulation_mode`, consumed by simulation, shooting,
  consistent initialization and fitting, plus the numerical-requirement derivations in `cases.rs` shared with the
  steady route.
- **Current mechanism.**
  - `prepare_simulation_mode` is about 1,500 lines. It decides endpoint contradiction and time-axis checks, the
    differential and algebraic partition (scanning `ModelingOutput::DynamicRate`, subtracting `used`, zip-assigning
    algebraic rows), the output-row rule, function roles and actions per integrator and sensitivity mode, row-scale
    formulas, post-compile structure rejection, and inventory and event-guard agreement.
  - `cases.rs` holds about 800 lines (`lower_authored_accuracy`, `lower_closure_requirements`, `contextual_scales`,
    `physical_row_requirement`, `conservation_row_requirements`, `state_reconstruction_requirements`) that use no
    runtime service and evaluate nothing beyond passed values.
  - The `AccuracyGoal*` hint set is spelled four times across `dynamics.rs` and `cases.rs`. `resolve_numerics`
    mutates `numerical.declarations` and then purges its own producers by id.
- **Why it matters.** Most of this is a function of the compiled product and a method selection. A compiler-owned or
  `pse-math::numerics` plan (partition, roles, row sets, scales, refusals) would give one owner, make the facts
  testable without a runtime or solver, and leave the runtime to bind values and observe. A sibling route already
  gets steady facts from a lower owner (`ViewPolicy::roles`, `NumericalAnnotations`). Today a new output kind needs
  edits inside this body, and several `dynamics.rs` tests run the full pipeline to reach structural rules.
- **Patterns.** LOP-02, LOP-09.
- **Evidence.** `dynamics.rs` `prepare_simulation_mode` read in full; `cases.rs` production bodies; `resolve_numerics`;
  `pse-compiler` `executable/grouped.rs` `ModelingOutput`. (Interface-checked.)
- **Footprint.** Inspected: the production bodies. Suspected: `workflow::dynamics` also interprets `Function`, and
  `DynamicProgram` construction may overlap.
- **Preservation constraints.** Physical coordinate identity, frozen allowance semantics, contract and profile
  identity, route selection with `resolve_for_retained`, every refusal diagnostic and stage, and "fixture schedules
  are the authority".
- **Counterevidence and uncertainty.** `actions` and the selected-Krylov decision depend on a native profile the
  compiler should not know, so the split would be structural facts versus method demand. `resolve_starts` and
  observations need evaluation, so a two-phase shape may remain. The two-phase policy resolution is required by the
  meaning of derived scales and is not a defect.
- **Status.** Current mechanism.
- **Research questions.** Which derivations depend only on the product, which on method choice, which on values?
  Could the lower owner publish a typed plan that the orchestrator binds? No library applies.

<a id="cbo-087"></a>

## CBO-087 — Source selection carries a hand-built query engine beside Salsa

- **Operation and consumers.** Select the minimal source closure for a root (`select_local`, consumed by
  `selected_revision`, `checked_selection` and `prepare`), and on a delta recompute only the declarations whose
  resolved references could have changed (`apply_delta_parent_ordered_mode`, consumed by edits, reload, merge and
  study variants).
- **Current mechanism.**
  - `canonical.rs` holds `SourceIndex` (rows, name buckets, memo tables, completeness sets), `Lookup::Pending` with
    `requests` as an acquisition fixed point, `probing` as a recursion cut, and a `member_memo` cleared wholesale on
    any new row and used only for root, non-traced lookups.
  - `ReferenceTrace` records reads per declaration and `Products.dependencies` stores them. The delta loop tests
    `reads.is_disjoint(&invalidated)` and early-cuts when references are unchanged.
  - `canonical/local.rs` adds a third piece: `LocalSourcePremises` records every positive and absent lookup and
    `matches_scoped` re-verifies them against another revision, with `Arc::ptr_eq` as a shortcut.
  - A complete index is built three ways: `SourceRevision::open`, `persist_with_project` and `admit_replay_source`.
- **Why it matters.** Dependency recording, early cutoff, cycle handling and memoization are what Salsa provides, and
  `pse-compiler` already uses it (`granular.rs`). Two incremental systems share one semantic chain and are reasoned
  about separately. The wholesale memo clear is a visible limit.
- **Patterns.** LOP-X019, LOP-01, LOP-03, LOP-07, LOP-09. Related: [CBO-107](mathematics-and-compilation.md#cbo-107),
  [CBO-108](mathematics-and-compilation.md#cbo-108).
- **Evidence.** `canonical.rs` (`Request`, `ReferenceTrace`, `SourceIndex`, `select_local`, `parsed_fields`),
  `canonical/delta.rs`, `canonical/local.rs`, `granular.rs` to the query heads. (Interface-checked.)
- **Footprint.** Inspected as named. Unread: how `documents.parsed_fields` flows into granular inputs.
- **Preservation constraints.** Exact premises for checked selection (A-B-A retention), per-object leases charged
  before decode, cancellation between items, declaration-order bucket contracts, Pending versus Absent, and no
  database handle escaping publication. Replay's recomputation is deliberate assurance ("self-consistent hashes
  alone do not establish a complete source closure") and must stay independent of stored references.
- **Counterevidence and uncertainty.** Salsa's revision model is a mutable database, whereas each revision here is an
  immutable shareable value with branching. Acquisition is effectful and charged, so Pending would have to become an
  input-demand protocol. Salsa memory is charged differently from prepaid leases. This may legitimately stay custom.
- **Status.** Current mechanism.
- **Research questions.** Can a branchable dependency graph be charged before computing? Do Salsa cycle recovery or
  forking fit immutable branching revisions? Would moving the index into `pse-modeling` or `pse-compiler` without
  Salsa be a smaller step?

<a id="cbo-090"></a>

## CBO-090 — Observation crossings and demand discovered in stages

- **Operation and consumers.** Preparing a case or simulation: observing rows at a point, and deciding derivative
  order and provider demand.
- **Current mechanism.** Two manifestations, kept distinct.
  - **Crossings.** `prepare_simulation_mode` issues separate `observe_registered` calls for initial conditions,
    original initial projections, nominal rows, accuracy rows and signs, each preparing or reusing a program and
    acquiring a worker with fresh reservations. `resolve_starts` is a layered fixed point: scan `instances` for
    ready hints, call full `observe` per layer, `retain`, repeat. `conformance_envelope` observes per range and sample.
  - **Staged demand.** `inner_registrations` runs at order Value, again at `order`, again for the parametric program,
    and again in `finish_case` when `solve.required_order()` exceeds the plan, which is known only after routing.
    Fitting's `declared_case` runs a full `declared_execution` and keeps only bindings and case.
- **Why it matters.** The intent "observe these rows at this point" is known up front. Start order is a graph fact a
  lower owner could compute once, and demand reaches the compiler in pieces because backend choice, order and
  providers depend on each other but are decided in sequence.
- **Patterns.** LOP-04, LOP-05, LOP-07, LOP-01.
- **Evidence.** `dynamics.rs`; `cases.rs` `observe`, `observe_registered`, `resolve_starts`, `finish_case`;
  `views.rs` `observation_program`; `implicit.rs` `inner_registrations`; `declared.rs` `declared_case`. (Interface-checked.)
- **Footprint.** `variable_states` and `resolve_numerics` by call site only.
- **Preservation constraints.** Per-layer value dependence where one start reads another, differing provider sets
  per crossing, per-observation failure attribution and `Domain` refusal, the `allow_missing_free` partial result,
  provider preparation before bound case identity, and `observation_structure` as the compiler-owned reuse key.
- **Counterevidence and uncertainty.** `resolve_numerics` crosses three different points, so they cannot all merge,
  and on the steady path `variable_states` is already one crossing. A joint program can refuse where a smaller one
  would not, so coalescing needs per-row outcomes. Programs are cached by structure and the early return makes the
  common case free; no duplicate work was measured. Staging may be essential because hints affect structure.
  `TrialHints::resolve` is deliberately repeated per trial.
- **Status.** Current mechanism.
- **Research questions.** Can routing be decided before provider preparation? Would a single topological evaluation
  subsume the layered loop, with Salsa tracked structs or accumulators expressing final demand as one query?

<a id="cbo-091"></a>

## CBO-091 — Revision comparison and diff implemented per operation

- **Operation and consumers.** Deciding what changed between source states, for comparison, merge, edits, reload and
  publication; CLI and agent inspection of revisions.
- **Current mechanism.**
  - **Comparison hydration.** `project_snapshot` decodes the delta parent, packages and fits into typed objects with
    a `retained*3` charge for both sides, even when a paged observation returns `limit` items. `project_merge_preview`
    snapshots three packages; `project_publish_merge` snapshots three, then base and proposal again in the manifest
    check loop. Equality is on decoded objects. `impacts` declares only numeric-literal initializers value-only.
    Merge is whole-object three-way.
  - **Diff written per site.** Object or row comparison is separate in `persist_with_project`,
    `apply_delta_parent_ordered_mode`, `publish_admitted_project`, `reload::make_edit`,
    `project_comparison::{compare, merge_objects, delta_to}` and `AdmittedModelRevision::apply_inputs`. The same
    six-field object equality is written at three of these. `record_study_source` and `record_study_variant` rebuild
    `ObjectEdit` although `LocalSourceObject::edit` exists.
- **Why it matters.** Revisions are persistent maps with structural sharing, yet each operation walks every entry.
  One structural diff would give one meaning of "changed" and work proportional to the change. Admitted objects carry
  a version per logical key, so unchanged ones could be excluded without decode. Compiler knowledge of structure keys
  could give a real value-versus-structural impact answer and feed an impacted-operations query
  ([CBO-145](capability-gaps.md#cbo-145)).
- **Patterns.** LOP-05, LOP-06, LOP-09, LOP-03, LOP-04, LOP-12.
- **Evidence.** `canonical.rs`, `canonical/{delta,local,reload}.rs`, `project_comparison.rs` to
  `project_publish_merge`. (Interface-checked.)
- **Footprint.** Inspected. Unread: `project_merge_*` bodies in full.
- **Preservation constraints.** Deterministic edit order, references compared with payload, span and package-hash
  normalization in `same_merge_object` (a comparison policy that differs by operation), bounded observation size,
  refusal across physical contexts, and explicit resolution of every conflict.
- **Counterevidence and uncertainty.** The operations compare different things (rows, objects, normalized objects),
  so only the object-level ones can share a mechanism. The comparison module deliberately excludes expression
  canonicalization, so compiler-derived impact changes that contract. Span normalization makes version equality
  stricter than semantic equality, so a version filter excludes only the identical case. Edit sizes may be small.
- **Status.** Current mechanism for hydration and per-operation diff; Proposed for compiler-derived impact and
  field-level merge.
- **Research questions.** Does the persistent-map implementation expose shared-node skipping for a diff, or does an
  alternative ordered map do so? Ask before assuming any gain.

<a id="cbo-096"></a>

## CBO-096 — Runtime numerics rebuild stable preparation at the wrong lifetime

- **Operation and consumers.** Path corrections, path curvature and original-point assessment, consumed by paths,
  events and the engineering, KKT and certified accuracy routes.
- **Current mechanism.** Three manifestations.
  - **Path correctors.** `correct` and `correct_target` each call `path.supplier`, making a new worker (charged
    anew), cloning values and the contract, rebuilding `AssemblyMatrix`, re-running structural analysis and
    `routing::Requirements::select`, and bypassing the scratch lane that `case_worker` uses. `localize_event` adds a
    `correct` per bisection. `run_path` also keeps a long-lived tangent supplier.
  - **Path curvature.** `CurvatureSupplier::action` computes `d' H_i d` by calling `oracle.hessian` once per row,
    with a pattern clone per action. `prepare_path_curvature` rejects directional actions.
  - **Assessment.** `selected_coordinate_accuracy` makes a fresh `SquareScope::admit` without `reuse_symbolic_from`,
    which `fresh_root_partials_at` and fitting's oracle do use. `native::assembled::contract(plan)` appears 12 times,
    row-bounds vectors are rebuilt at about nine sites, and `BlockAssessor::evaluate` rebuilds tolerances and a worker.
- **Why it matters.** The plan has fixed structure that consumers re-derive, and `root_layout` shows symbolic reuse is
  valued but applied on some paths only. A path-lifetime owner of route, structure and worker, a square view cached
  with the plan, or a collective second-directional pass could change repeated work to once.
- **Patterns.** LOP-07, LOP-09, LOP-03, LOP-04.
- **Evidence.** `paths.rs` `correct`, `correct_target`, `supplier`, `run_path`, `CurvatureSupplier::action`;
  `solves.rs`; `square_response.rs` `SquareScope::admit`; `initialization.rs` conditional unit session. (Interface-checked.)
- **Footprint.** Inspected as named. Native session reuse under `retained` was not traced and may already reuse state.
- **Preservation constraints.** Worker scope and cancellation binding, work-count attribution, per-correction
  callback ownership, bounds from the bound values, `SymbolicPlan` compatibility by exact support and order, and the
  charge-before-factor order.
- **Counterevidence and uncertainty.** By a method-call search, `MathService::prepare_path` has no production caller
  (see [CBO-099](#cbo-099)), so the path half has value only if paths become consumed. Worker construction cost is
  unmeasured. The per-correction comment cites diagnostics, though the catalogue could be scoped without a new
  evaluator. Bounds change under rebind, so plan-level caching must key on the actual plan.
- **Status.** Current mechanism.
- **Research questions.** Can the jets or Taylor layer give a one-pass second directional derivative of the residual?
  Does `WeightedHessianAction` with a per-row action suffice as a cheaper intermediate?

<a id="cbo-089"></a>

## CBO-089 — Assessment reinterprets outputs at every observed point

- **Operation and consumers.** `assess_observations`, called per candidate, per trajectory sample
  (`SampleCheckWorker::check` in `dynamics_checks.rs`) and in fitting.
- **Current mechanism.** Each call scans `admitted.outputs` and annotations, looks up closures and expectations by
  map, does a linear `outputs.find` and derived `row_id()` per annotation, and a linear `numerics.targets.find` for
  tolerances. The sample worker rebuilds a `BTreeMap` of observed rows per sample and does a linear equation find for
  each `Row` target. `assessment_units` is recomputed by each caller. `ModelingCheck` is a generated row of about 30
  fields with no `Default`; the same 20-plus `None` literal lines appear five times here and once each in
  `conformance.rs`, `dynamics_checks.rs`, `fitting/profile.rs` and `numerics.rs`.
- **Why it matters.** The row map is value-independent, so it could be prepared once per product, with row
  positions, tolerances and kinds in one declaration, probably owned by the compiler product. Sample counts multiply
  obligations. A constructor or generated builder would remove the literal repetition, which stands regardless.
- **Patterns.** LOP-07, LOP-02, LOP-04.
- **Evidence.** `results.rs` `assessment_units`, `assess_observations`; `dynamics_checks.rs`; `assessment.rs` call
  site; `grouped.rs` `row_id`; the generated `pse-model/src/generated/runtime/modeling_checks.rs` derive list. (Interface-checked.)
- **Footprint.** Unread: `product.assess_expectations_for` and `assess_closures` in lower crates. A generator change
  sits outside this scope.
- **Preservation constraints.** Check order, scope filtering, duplicate-report refusal, certified-bound substitution,
  missing-value refusal, and identical semantics across candidate, sample and fit.
- **Counterevidence and uncertainty.** Typical output counts may be small enough that scans are immaterial; nothing
  was profiled. Per-sample interpretation is shown, its cost is not.
- **Status.** Current mechanism.
- **Research questions.** Where would a prepared assessment plan live so that candidate, sample and fit share it?
  Can the generator emit a builder or `Default` for the row?

<a id="cbo-097"></a>

## CBO-097 — Identity hashes recomputed from full data at every use

- **Operation and consumers.** `PreparedSolve::original_identity`, `preparation_identity`, `strategy_profile` and
  `profile_key`, about 131 calls in non-test `math/` by search. Uses include `decision_key` per driver iteration,
  `direct_batch_candidate`, refined-step comparison and trace construction.
- **Current mechanism.** `original_identity` walks every scalar, provider and, for conic problems, every matrix index
  and value through `FramedHasher`. `preparation_identity` hashes artifacts and rungs recursively. Neither memoizes,
  and `PreparedSolve` is cloned and mutated through builder methods. `step_initial` computes
  `step.preparation_identity()?` each loop pass; `batch` and `step_bound` hash each member and input point; the
  multistart paths recompute `original_identity` at five sites.
- **Why it matters.** Sweeps over many cases repeat framed hashing in the coordinating path, and memoized identities
  would simplify signatures that return `Result`.
- **Patterns.** LOP-07.
- **Evidence.** `solves.rs` `original_identity`, `preparation_identity`; `driver.rs` `step_initial`. By search for
  callers. (Interface-checked.)
- **Footprint.** Not all callers read.
- **Preservation constraints.** Identity must follow every builder mutation (`with_start`, `with_strategy`, profile
  changes). A stale identity would be a scientific provenance error.
- **Counterevidence and uncertainty.** Hashing small cases is cheap beside a native solve; large conic or coefficient
  problems are the credible cost. Invalidation risk is real and would need a frozen-versus-builder split. Nothing
  measured.
- **Status.** Current mechanism.
- **Research questions.** Can identity be computed once on a frozen type (for example behind a once-cell) without a
  path that mutates after caching? A faster hash is not the question, because the framing is contractual.

<a id="cbo-098"></a>

## CBO-098 — PETSc callbacks evaluate through a map-keyed point

- **Operation and consumers.** `solves/petsc.rs` `LocalBlock::{point, residual, jacobian}` and
  `CompiledDomain::check`, called from inside PETSc on every callback.
- **Current mechanism.** Each callback inserts every coordinate, ghosts by `binary_search`, into
  `CaseValues.scalars: BTreeMap<SemanticId, f64>`, then calls `worker.constraints(&values)` or
  `worker.jacobian(&values)`. `check` evaluates objective, constraints and Jacobian each time. `jacobian` allocates a
  temporary per call and remaps through `jacobian_slots`. A compact slot path exists elsewhere:
  `native::assembled::AlgebraicOracle::compact`, `refill_inputs` and `assembly.bind_inputs`.
- **Why it matters.** Block callbacks are the hottest path in this layer, and each pays map work plus an allocation.
  Slot-bound inputs could remove both.
- **Patterns.** LOP-06, LOP-09.
- **Evidence.** The named functions read; `assembled.rs` `refill_inputs` by signature. (Interface-checked.)
- **Footprint.** `petsc.rs`; block preparation and part of `execute_petsc` setup read only partly.
- **Preservation constraints.** Validate the whole output before writing, finite checks, ghost selection by the
  compiler, and the domain check's full-demand meaning (applicability claims are evaluated by the Jacobian too).
- **Counterevidence and uncertainty.** The worker may bind inputs by map once per call anyway, so removing the insert
  alone may gain little. Evaluator cost is unknown and nothing was measured.
- **Status.** Current mechanism.
- **Research questions.** Can the PETSc callbacks bind slots once per block and refill in place, as the compact
  oracle does?

<a id="cbo-092"></a>

## CBO-092 — Project ancestry walked one guarded round trip per revision

- **Operation and consumers.** `Runtime::project_is_ancestor`, used by merge decision, undo and publication, with two
  ancestry checks per decision.
- **Current mechanism.** A client-side depth-first search with a `seen` set and a bound of 8,192 calls
  `store.revision(key)` and `store.project_revision_parents(&revision)` per node. The second runs a guarded SurrealQL
  script per call (`pse-operations/src/canonical_projects.rs`). Publication also re-validates ancestry by traversal.
- **Why it matters.** The intent is one question, whether A is an ancestor of B, and the store holds the data
  (`parent` on revisions, `canonical_project_ancestry`, branch `origin_revision`). One bounded server-side traversal
  could remove up to thousands of crossings per check. This is the same shape as the per-point round trips in
  [CBO-073](persistence-and-transport.md#cbo-073).
- **Patterns.** LOP-04, LOP-05.
- **Evidence.** `project_comparison.rs` `project_is_ancestor`; `canonical_projects.rs` `project_revision_parents`,
  which belongs to a sibling scope. (Interface-checked.)
- **Footprint.** Caller side read; store schema not read beyond that script.
- **Preservation constraints.** The availability and issuer guards inside the script apply per revision; bounded
  work; a cancellation checkpoint between steps; refusal when a source is unavailable.
- **Counterevidence and uncertainty.** Typical history depth is unknown and probably small on a workstation. Parent
  links are record fields plus an ancestry table rather than graph edges, so a recursive query may need a schema view.
- **Status.** Current mechanism.
- **Research questions.** Can recursive record links, or a bounded loop inside one guarded script, run the per-step
  guard server-side under the same transaction?

<a id="cbo-094"></a>

## CBO-094 — One requested operation in several serialized request languages

- **Operation and consumers.** Durable studies, scientific replay, reproducers and review tooling.
- **Current mechanism.** `StudyOperation` and `OperationRequest` (`StoredPoint` JSON) drive reconstruction in workers.
  `ReplayConstructor`, `ReplayCaseConstructor` and `OriginalCaseInputs` capture constructor arguments for rerun. The
  canonical run row carries `request`, `source_selection` and `attestation` byte fields, and `reproducer.rs` extracts
  from replay documents. In-process rerun reconstructs, compares `replay_declaration` and `same_inventory`, and on
  mismatch substitutes the original graph. `ReplayCaseConstructor` refuses portable replay when implicit capabilities
  are present, while the study route re-admits them.
- **Why it matters.** A new option must be placed in each language, and each has different reach: a study point is
  portable, a replay document may refuse the same case. `OriginalCaseInputs::capture` has hand extent arithmetic per
  field, a typical drift point.
- **Patterns.** LOP-02, LOP-09, LOP-X007.
- **Evidence.** `workflow/replay.rs` (`OriginalCaseInputs::{capture, document}`, `rerun_with_cancel`,
  `modeling_request_facts`), `study_operations.rs`, `operation_preparation.rs` (`RuntimeOperationRequest`),
  `reproducer.rs` head, `study.rs` `definition_from_parts`. (Interface-checked.)
- **Footprint.** Unread: `ReplayDocument::reconstruct_body`, `review_context.rs`.
- **Preservation constraints.** A replay is new work, never occurrence recovery. Exact values and start bits
  (`bits`/`unbits`); screening is not re-granted by history; a study operation excludes occurrence identity; refusals
  stay explicit.
- **Counterevidence and uncertainty.** The languages sit at different altitudes on purpose: intent versus the
  resolved constructor that records what ran. `RuntimeOperationRequest` shows some unification is done. Merging could
  erase a real distinction.
- **Status.** Current mechanism.
- **Research questions.** Which fields are truly shared across the languages? Could a serde-derived field accounting
  produce the extent counts without a second spelling?

<a id="cbo-093"></a>

## CBO-093 — Logical keys and identities cross boundaries as formatted strings

- **Operation and consumers.** Object identity inside admitted revisions, and claims, recovery, seeds, block keys and
  operation receipts.
- **Current mechanism.**
  - **Source keys.** About 62 `format!("fit:{..}")`, `data:`, `document:`, `record:` and `physical_scope_document:`
    sites plus `starts_with` filters, and `o.kind == "data"`, `"package"` and `"modeling:import"` literals across
    `canonical.rs`, `canonical/{delta,reload}.rs` and `project_comparison.rs`.
  - **Operation identities.** `claim.fence.run().strip_prefix("run:")` plus `parse_hex` occurs three times. The
    attempt-lineage hash `"scientific.attempt.lineage.v1"` is derived in both `worker.rs` and `durable.rs`. Idempotency
    keys are ad hoc strings (`study-claim:`, `study-recover:`, `study-summary:`, `export-close:`), and `StudyId` is
    parsed back from the study key in `finalize_canonical_study`.
- **Why it matters.** A prefix filter stands in for a typed key, so a family mismatch ("merge logical identity
  changed object family") is representable. A duplicate derivation is a silent identity split if one moves.
- **Patterns.** LOP-02, LOP-06.
- **Evidence.** Counts by `rg`; `worker.rs` `work_study_candidate`, `durable.rs` `recover_with_study_point`,
  `study_execution.rs` (two `strip_prefix` sites). (Interface-checked.)
- **Footprint.** Representative, with no census. `canonical/local.rs` inventory form and `pse-operations` canonical
  storage were not read.
- **Preservation constraints.** Persisted logical strings and stored key ordering are identity; operation-key replay
  semantics must hold.
- **Counterevidence and uncertainty.** Largely a local refactor of small value. `result_batch_key` and `result_set_key`
  show typed constructors already exist for some keys.
- **Status.** Current mechanism.
- **Research questions.** Would the store's typed record identifier serve as the boundary key, given that stored
  strings must stay stable?

<a id="cbo-095"></a>

## CBO-095 — Nonlinear explanation runs a sequential deletion filter

- **Operation and consumers.** `explain_nonlinear`, an infeasibility diagnosis for modelers.
- **Current mechanism.** One attempt per outer equation in row order, one at a time. Each trial sets
  `Formulation.omitted` and goes through `prepare_analysis_attempt`, then the POUNCE ell-1 method, seeded only with
  base values. The budget is `maximum_attempts`. `certify_infeasibility` delegates global IIS to SCIP, which is the
  strong route.
- **Why it matters.** This is a known conflict-isolation family, and the trials are independent solves on a
  many-lane workstation. Parallel trials, a divide-and-conquer order, or reuse of the prior candidate could reach a
  smaller candidate within the same budget. The omission structure changes per trial, so structural preparation is
  repeated.
- **Patterns.** LOP-01, LOP-07, LOP-12.
- **Evidence.** `diagnostic_nonlinear.rs` read in full. (Interface-checked.)
- **Footprint.** Preparation cost not measured.
- **Preservation constraints.** Results stay local evidence, never a global verdict; deterministic attempt order in the
  report; a shared deadline; retained attempts; omission stays a formulation over original rows.
- **Counterevidence and uncertainty.** A deletion filter's order determines which subset is retained, so parallelism
  needs a rule to keep it deterministic. Consumer demand for faster explanations is assumed, not evidenced.
- **Status.** Proposed.
- **Research questions.** Does a conflict-isolation algorithm such as QuickXplain fit the formulation-omission
  interface? What shares across trials besides the base values?

## Examined and not retained

- **Conformance repeating declared execution.** `conform_fixture` needs stages that execution hides (route assessment
  before attempt, degrees of freedom from structural inventory, derivative sampling), so most of its length is
  legitimate. The residue is small: the envelope status mapping written twice, `admission_tables` export three times,
  and numerics re-derived in `conformance_derivatives`.
- **Delta ingress validation by building and discarding columnar batches.** About nine sites build a batch only to run
  row-to-column validation. One validation authority is the point, and a row-level validator would be a second
  implementation unless generated; no library lead.
- **The rung protocol as a closed enum.** `PreparedRung` has about 15 same-shaped matches and a 300-line closure in
  `step_bound`, which is local machinery with a small library role.
- **Private child IPC framing.** `native_child` serializes each message twice to bound it, and `portable.rs` has a
  separate frame. Low consequence and one-shot per child, and the route has no production caller (see CBO-099).
- **The four-counter work ledger.** `strategy/admission.rs` writes the same four counters in a dozen places. A
  counter-indexed local type would shrink it; explicit fields are exhaustive and the unit set has been stable.
- **`futures::StreamExt::buffered` for `conform`.** A slow head stalls the window, which the contract rejects;
  the owned `FuturesUnordered` and ordered fatal-drain logic are justified.
- **Text merge libraries for project merge.** Merge operates on typed authored objects, and a line merge would bypass
  typed identity.
