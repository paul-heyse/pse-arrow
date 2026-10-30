# Capability gaps

This page covers activities that are absent from the product or hard to compose from what exists, as
opposed to inefficient realizations of capabilities already delivered. The architecture owners are
[§25 Supported scope and recorded limits](../../authoritative_design/sections/scope-and-open-design.md#section-25)
and the "Capability coverage against IDAES" appendix in the same page, with the risks in §26 and the
scope list in [§0.2](../../authoritative_design/sections/architecture-overview.md#section-0-2). Every
entry here is a proposal for investigation, not accepted scope; where an entry also records a promise or
an accepted plan, the status line says so separately. The baseline is on the [entry page](README.md).

**System picture.** pse-arrow is a linked-Rust process simulator. Its authority is authored `.pse`
definitions, executed by library mathematics and native solvers, with results retained in one canonical
store. Its users are a Rust application author calling `pse_runtime::workflow`, a model author writing
`.pse`, the maintainer and agents using `pse` and `pse-project`, and isolated scientific reference
drivers. There is no end-user product shell.

**The §25 table lags the source.** A search of §25 and the coverage appendix for multistart, pseudo-arclength
continuation, IBEX root isolation, Latin hypercube designs, study cohorts, reducers and reproducers, and
project packs found no mention, yet each exists in source: designs (`workflow/sampling.rs`), multistart seeds
(`math/solves/multistart.rs`), continuation (`math/solves/paths.rs`, `backend-native/kkt/path/arclength.rs`),
PETSc products (`math/solves/petsc.rs`), IBEX isolation (`root_isolation.rs`), cohorts over DataFusion
(`workflow/cohort_provider.rs`, `study_workbench.rs`), `workflow/reduction.rs` and `reproducer.rs`, and
`project_portability.rs`. The appendix row "Surrogates … Not implemented" coexists with a retained egobox actor.
So a gap read from the documents alone is often false: "no multistart" and "no continuation" are not gaps.

The real pattern is different ([LOP-X022](../assessment-patterns.md#lop-x022)): many mechanisms stop short of a
retained product. A strong, library-backed numerical mechanism ends as an auxiliary start, a proposal, an
indicator or a Rust-only handle, and the activity it could serve for an engineer stays unavailable. Most entries
here ask for that last step, and the libraries involved are often already linked. In each case the confinement
may be the scientific-honesty contract (a surrogate start grants no result permission), so the question is the
smallest retained product and the validity statement it needs.

**Three statuses, kept apart.** *Promised* means in the target scope or recorded as a limit, with no delivery
commitment ("Scope is not a delivery commitment", §0.2): transport and Helmholtz properties, surrogates, utility
minimization and the global-solving envelope contract. *Accepted planned* means a plan or accepted decision
schedules it. Reading the active-plan index, plans 59, 58 (with 58a, d, e, f, g, h), 54, 53, 40, 32 and 28 by topic
found none that schedules a scientific entry below; the one accepted-planned item among the gaps examined is
application backup and restore, an obligation of Plan 58h before non-regenerable data, homed at
[CBO-074](persistence-and-transport.md#cbo-074). Plan 57d (sampling and statistical workflows) is superseded and
authorizes nothing. *Proposed* is everything else.

**Gaps homed elsewhere.** Surrogate products ([CBO-135](numerical-workflows.md#cbo-135)), non-local uncertainty
([CBO-034](numerical-workflows.md#cbo-034)), fit quality and estimated variance
([CBO-036](numerical-workflows.md#cbo-036)), experiment design and reconciliation
([CBO-137](numerical-workflows.md#cbo-137)), language tooling for `.pse` ([CBO-113](authoring-and-identity.md#cbo-113)),
and backup ([CBO-074](persistence-and-transport.md#cbo-074)) were first found as gaps and live on those pages.

**Coverage on this page.** Read closely: §25, the coverage appendix and §26; the provider route from `.pse`
`external` through `modeling_package_registered` to `prepare_selected_providers`; `workflow/uncertainty.rs`,
`workflow/sampling.rs`, `study_workbench.rs::aggregate`; `canonical_explanations.rs`; the solver paths for
bounded roots and the adjoint profile (`execution/petsc.rs`, `dynamics.rs::admit_adjoint`, `integrator/adjoint.rs`,
`dynamics/idas.rs`, `anchored.rs`). Representative: continuation (`paths.rs`, `arclength.rs` heads and
`PathRequest`), `scip.rs` expression vocabulary, `kinsol/block.rs`, authoring and `.pse` tooling. Skimmed: the
global-certification bodies behind `grep -il ibex`, `diagnostic_request.rs` and `normalization.rs`, and unit-model
breadth, which is document-level only and not source-verified. Not read: active plan bodies beyond the topic hits,
`project_portability.rs` in full, `pse-math::continuation`, and the IDAES algorithms beyond signatures and docstrings;
parmest bootstrap lies outside this checkout. Absence claims rest on case-insensitive `rg` over `crates/` and do not
prove absence under another name. Several statements about demand are inferred from IDAES behaviour, not recorded.

<a id="cbo-141"></a>

## CBO-141 — Transport, Helmholtz and high-accuracy property backends

- **Operation and consumers.** A process engineer sizing equipment needs viscosity, conductivity, diffusivity and
  surface tension; a steam, refrigerant or carbon-dioxide cycle designer needs reference-quality fluid properties;
  a model author composing non-seed unit models needs both. The model author is the consumer of the provider seam;
  a Rust application author is the only one who can register code today.
- **Current mechanism.** The coverage appendix marks transport "Not implemented" and Helmholtz and CoolProp
  "Not implemented"; §0.2 lists Helmholtz in scope as providers. No source hit for viscosity, thermal conductivity,
  diffusivity or surface tension in `crates` or `packages`. The seam is built: `ProviderSpec`, envelopes, derivative
  and smoothness contracts, the `.pse` form `external <implementation> revision <r> data <d> output … derivatives`,
  and `Runtime::modeling_package_registered(rows, physical, providers)`, documented as "source names load no code".
  Outside tests, `impl Provider` is wrappers and implicit-block factories only, `pse-kernels/src/feos/` is empty,
  FeOS is reference-only, and no reference package uses `external`. The seam has never carried a scientific external
  function in production code.
- **Why it matters.** Sizing-oriented unit models need transport properties, and this is a promised property gap. Real fluids are otherwise inaccessible. Two routes are distinct: a provider for Helmholtz fluids, and
  authored correlation-style transport (mixing rules, DIPPR-class forms) as `.pse` laws plus data documents that need
  no new library. Neither covers the other.
- **Patterns.** LOP-12, LOP-X022.
- **Evidence.** Read §0.2, the coverage appendix, §9 provider text, `pse-kernels/src/lib.rs`, a listing of
  `pse-kernels/src/feos`, a grep of `impl Provider`, and IDAES `general_helmholtz` and transport modules at docstring
  level. Interface-checked; absence by search.
- **Footprint.** How an external worker process obtains a registration is unresolved: no `registered` or `providers`
  hit in `worker.rs`, `native_worker.rs`, `durable.rs` or `native_deployment.rs`, so durable studies through a
  registered provider are not established.
- **Preservation constraints.** D9 provider contracts; scientific knowledge lives in authored bundles and a provider
  must not smuggle species meaning into the host; no silent fallback; per-quantity coordinates, phase-boundary
  smoothness, derivative source and validity envelope declared; data identity for a library's embedded parameter
  files; thread-confined state under per-worker factories. In-tree registration needs no firing of register R-04.
- **Counterevidence and uncertainty.** Registers R-04 and R-05 record deliberate deferral. Global routes refuse
  provider outputs without an envelope ([CBO-142](#cbo-142)). The maintainer records the thermodynamic knowledge
  repository as far-off, so data curation is excluded here.
- **Status.** Declared target, undelivered, with no accepted plan found. The mechanism choice is Proposed.
- **Research questions.** Which library exposes per-state analytic or automatic derivatives with reusable
  thread-confined state? Can candidate libraries be built hermetically in the local solver prefix?

<a id="cbo-143"></a>

## CBO-143 — Engineer-readable outputs: stream tables, diagrams, equations

- **Operation and consumers.** An engineer or reviewer wants the stream table of a solved flowsheet, a drawing of its
  topology and recycle tears, and a readable listing of a specialized model's equations. An agent or the maintainer
  diagnosing a model would read the same outputs. IDAES provides these through `tables.py` and its flowsheet visualizer.
- **Current mechanism.** The coverage appendix marks `tables` Implemented through "checked Arrow results". No hit for
  stream tables, Graphviz, DOT or Mermaid in `crates`; petgraph is declared without its `dot` feature; Symbolica has
  `PrintMode::Latex` in vendor source and `crates` has no `latex` hit. The review pack (`tooling/review_pack.rs`,
  `review_analysis/render.rs`) renders bounded Markdown and static HTML, over review questions, not flowsheets.
  `pse-structural::flowsheet::FlowGraph` holds topology, but a heading search of generated relations found none that
  retains connection topology, and the `solve_variables` row keys on `run_id, step, symbol_id` with quantity, unit,
  value, bounds and duals but no port, stream or instance field. A stream table needs a join from connection to port
  to symbol to retained value that no owner supplies. IDAES resolves each arc to its state block's declared variables;
  the equivalent here is an authored display declaration on a state binding, which does not exist.
- **Why it matters.** Verifying that a flowsheet is what its author meant currently means reading relations or `.pse`.
  The renderers are cheap; the stream-table definition (variables, units, phase handling, from authored port metadata)
  is the real work.
- **Patterns.** LOP-12, LOP-03, LOP-X022.
- **Evidence.** Read `physical_inspection.rs`, the encoders in `simulation_results.rs` and `modeling_results.rs`,
  `review_analysis/render.rs` header, Cargo.toml petgraph features, IDAES `core/util/tables.py`. Interface-checked.
- **Footprint.** `workflow/modeling/results.rs` not read in full.
- **Preservation constraints.** Rendering is a read-only projection with no authority; bounded review rendering stays
  bounded; no second result path.
- **Counterevidence and uncertainty.** A user interface is a later port and this is a linked-library product with no
  shell. Agents may prefer structured output over drawings. Whether Symbolica LaTeX works on expressions as retained,
  or only at compile time, is unchecked.
- **Status.** Proposed. The `tables` cell of the coverage appendix is generous and is arguably a current promise.
- **Research questions.** Which retained fact would carry a stream's membership? Does a pure-Rust layout crate meet
  the need without a Graphviz install?

<a id="cbo-140"></a>

## CBO-140 — Retained branch diagrams and steady-state stability

- **Operation and consumers.** A reactor or recycle engineer tracing a steady-state branch through a parameter and
  looking for folds (ignition and extinction in a stirred-tank reactor), and asking whether a converged point is
  dynamically stable. §26 records "Local convergence only; multiple roots depend on starts" as a current risk.
- **Current mechanism.** Sparse bordered pseudo-arclength continuation with turning events exists (`solves/paths.rs::
  PathRequest`, `kkt/path/arclength.rs`). Its contract is that auxiliary path points remain starts for the original
  completion owner and numeric turning indicators do not certify folds; the path is a derived strategy rung, not a
  retained branch relation. No hit for eigen or spectral analysis of a system Jacobian; eigen uses found are
  reduced-Hessian, convexity and covariance PSD checks, and `square_response.rs` holds rank only.
- **Why it matters.** The costly numerics exist and a multiplicity answer is unreachable though the corrector and
  tangent machinery could produce it.
- **Patterns.** LOP-12, LOP-X022.
- **Evidence.** Read `paths.rs` head and `PathRequest`, `arclength.rs` head, the `Path` variant in `derived.rs`, greps
  across `workflow`, `math` and `backend-native/dynamics`. Interface-checked.
- **Footprint.** `EventPolicy` read partly; `pse-math::continuation` suspected and unread.
- **Preservation constraints.** An auxiliary point grants no result permission; fold and Hopf labels are estimated, not
  certified; a stability statement carries validity under PS-12 and the index-1 restriction; the branch relation needs
  its own schema and identity, and each point may need original assessment at a cost.
- **Counterevidence and uncertainty.** The restriction may be deliberate, to keep one completion owner. No consumer is
  named. Tracing a branch up to a turning point is possible today by seeded continuation, so only the fold-passing and
  stability parts are absent.
- **Status.** Proposed (the underlying risk is recorded in §26).
- **Research questions.** Does a generalized eigen routine on the fixed `diag(I,0)` pencil suit small and medium sizes?
  For large sparse pencils, see [CBO-119](native-solver-integration.md#cbo-119) on densification of sparse operators.

<a id="cbo-142"></a>

## CBO-142 — Provider envelopes and global solving through providers

- **Operation and consumers.** A user of the explicit `certify` intent, or of MINLP, whose model contains a property
  provider inside a nonlinear term. Thermodynamics-bearing models are the main workload, so global claims are mostly
  unavailable to them.
- **Current mechanism.** §25 refuses global solving where a provider output without a declared envelope enters a
  nonlinear term, noting "no production provider declares one yet". `ProviderFactory::envelope` is a declaration;
  nothing derives one, and an opaque worker cannot be interval-evaluated. `scip.rs` maps a fixed vocabulary
  (`SCIPcreateExpr{Sum,Product,Pow,Exp,Log,Sin,Cos,Abs,Value,Var}`); no `SCIPincludeExprhdlr` appears in `crates`.
  IBEX is linked and used for root isolation.
- **Why it matters.** Without a way to supply envelopes or estimators, providers ([CBO-141](#cbo-141)) and surrogates
  ([CBO-135](numerical-workflows.md#cbo-135)) can never enter certified routes. With no production provider there is
  no present consumer, so the entry is dependent.
- **Patterns.** LOP-12, LOP-05.
- **Evidence.** Read §18.9, §18.10.1, §25, §9 envelope text, `scip.rs` API list and `execution/factorable.rs` head; the
  IBEX bodies were not read. Interface-checked.
- **Footprint.** `pse-math::factorable` unread. Interval work is on [CBO-118](native-solver-integration.md#cbo-118).
- **Preservation constraints.** The ADR-0120 envelope contract; enforcement at every evaluation; the envelope frames
  provider identity; readback to 10⁻⁹; sound relaxations only; `Relaxed` labelling for dropped rows; original
  requalification. A sampled bound is not an envelope and must not be presented as one.
- **Counterevidence and uncertainty.** Sound provider estimators are the hard part and are modeling work, not a library
  capability. A proved bound for a real equation of state may be unreachable; the Peng-Robinson instability case already
  fails to close in bounded time. Lifting closed-form property models through factorable expansion may be preferable.
  The priority of global claims is unknown.
- **Status.** Current promise (global certification over finite boxes is implemented, and provider envelopes are an
  accepted contract under ADR-0120 with no production declaration). Proposed: any mechanism that derives, attests or
  estimates.
- **Research questions.** Can a symbolic surrogate be interval-evaluated through the Symbolica route? Who attests an
  envelope, on what evidence? Does SCIP's custom-expression extension accept evaluation, interval and estimator callbacks
  fit for a provider?

<a id="cbo-148"></a>

## CBO-148 — Adjoints stop at events and resets

- **Operation and consumers.** Fitters and shooting users needing parameter gradients through hybrid dynamics (authored
  mode switches). A large-parameter fit of a hybrid model has only forward sensitivities, on one backend, and their
  cost grows with parameter count.
- **Current mechanism.** `Profile::admit_adjoint` refuses any contract with events, on the stated grounds that Diffsol
  has no mass-matrix reset adjoint and keeps reset metadata private, and IDAS has no reset sensitivities. IDAS events
  also run without forward sensitivities. §25 records the refusal. Only Diffsol forward sensitivities cross resets
  (`apply_reset_with_sens`, `apply_reset_with_sens_mass`).
- **Why it matters.** Mode switches are authored, supported features, and the cost of the missing route scales with
  parameter count. For the ODE case the barrier is smaller than the comment says: `apply_reset_with_adjoint` is a public
  trait method in Diffsol 0.16.2 and 0.18.0, and the repository's `Passes::run` already drives segments and knows its
  root index. The real limit is mass matrices: `State::apply_reset_with_adjoint` errors when `eqn.mass()` is set, and
  `Passes::record` refuses `RootFound` on the forward pass. Process DAEs need the mass route. Diffsol 0.18.0 adds
  `StepVjp::step_vjp` for reverse-mode derivatives of accepted steps on DAEs; IDAS has `restart_backward` and `jump` as a
  mechanical place for a reset jump. Neither is a library call that discharges the derivation and event-time term.
- **Patterns.** LOP-12, LOP-03, LOP-X022.
- **Evidence.** Read `dynamics.rs::admit_adjoint`, `integrator.rs::reset_sens`, `integrator/adjoint.rs`, the vendored
  Diffsol 0.16.2 adjoint and state sources, and 0.18.0 `adjoint.rs`, `state.rs`, `step_ad.rs` at module-doc level.
  Interface-checked, not run.
- **Footprint.** The 21,000-line Diffsol 0.16.2 to 0.18.0 diff was not read line by line. IDAS reset sensitivities were not
  derived. Supplier-move questions are [CBO-129](native-solver-integration.md#cbo-129).
- **Preservation constraints.** Bounded checkpoints and replay budgets; sample jumps; the refusal of a fixed-time
  observation that coincides with a parameter-dependent jump; index-1 algebraic handling.
- **Counterevidence and uncertainty.** Most process DAEs have mass matrices, so the ODE-only path covers a minority. A
  discretize-then-optimize hybrid adjoint would be a new route with different accuracy and a saved state per step unless
  thinned. Demand is not recorded in the scope read.
- **Status.** Current mechanism (stated refusal in §25); Proposed (the route).
- **Research questions.** Can `step_vjp` plus a transposed reset deliver DAE hybrid gradients at acceptable memory under
  bounded checkpointing, and how would they compare with forward sensitivities on the event models that exist?

<a id="cbo-152"></a>

## CBO-152 — Structural DAE index analysis is absent

- **Operation and consumers.** A dynamic-model author whose DAE is not index-1, who today meets an opaque integrator or
  initialization failure.
- **Current mechanism.** The declared dynamics profile is ODE or index-1 with fixed `diag(I,0)`. A search of `crates/`
  and the owner sections for Pantelides, index reduction and DAE index found only unrelated hits. `pse-structural` has
  matching and the math stack has symbolic derivatives.
- **Why it matters.** Structural index detection could turn opaque failures on high-index models into authored, located
  diagnostics. Index reduction would widen the admitted class.
- **Patterns.** LOP-12, LOP-01.
- **Evidence.** The search above and the §25 Dynamics row. Interface-checked; `continuous.rs`, `temporal.rs` and the runtime
  dynamics path were not read.
- **Footprint.** Absence limited to Rust source in `crates/` and the architecture `sections/`.
- **Preservation constraints.** Index-1 profile refusals; one model across analyses; authored equations remain the authority,
  so any differentiated equation needs lineage.
- **Counterevidence and uncertainty.** Two obligations differ. Detection is a diagnostic inside the current exclusion.
  Reduction conflicts with a recorded exclusion: §25 states that general higher-index DAEs "are not part of the design
  target, not merely unimplemented", so reduction would need a design decision and not an implementation. No demand is
  recorded, and another name may cover an equivalent check.
- **Status.** Proposed (detection). Reduction is a recorded exclusion, not a promise.
- **Research questions.** Does any Rust or C library offer Pantelides or sigma-method analysis, or should detection build
  on the existing matching?

<a id="cbo-147"></a>

## CBO-147 — Bounded nonlinear roots have no native bounded-root method

- **Operation and consumers.** Root and initialization intents with two-sided boxes, a common physical condition; model
  authors who bound mole fractions or temperatures.
- **Current mechanism.** KINSOL expresses one-sided bounds as shifted signs, and §25 lists two-sided boxes on KINSOL as
  refused, routing two-sided roots to a constrained NLP adapter. `execution/petsc.rs::admit_contract` refuses any finite
  variable bound; its methods are Newton trust region, pseudo-transient and NASM. The prepared PETSc prefix ships
  `SNESVINEWTONRSLS`, `SNESVINEWTONSSLS` and `SNESVISetVariableBounds` in `petscsnes.h`, and the bridge already owns SNES
  profiles.
- **Why it matters.** A square system with box domains would get a method matching its class rather than an
  interior-point reformulation with dual state and barrier behaviour, and PETSc's block and flow compositions could then
  accept bounded models.
- **Patterns.** LOP-12, LOP-03.
- **Evidence.** Read `execution/petsc.rs`, `settings/petsc.rs` head, `pse_petsc.h`, §18.7. Interface-checked.
- **Footprint.** Bridge bodies not read.
- **Preservation constraints.** A solution of a variational inequality at an active bound is not a root of the original
  system; original residual acceptance must refuse rather than relabel it. No hidden fallback engine; explicit settings;
  typed native reasons; domain callback semantics.
- **Counterevidence and uncertainty.** ADR-0215 may keep the profile small deliberately. Ipopt's feasibility route is
  proven here and may suffice. The need is inferred from bounded initialization; no failing workload was seen.
- **Status.** Proposed.
- **Research questions.** Does the variational-inequality family compose with the project's DM-shell NASM and domain errors
  at the pinned PETSc?

<a id="cbo-145"></a>

## CBO-145 — Transitive impact query over retained lineage

- **Operation and consumers.** A scientist who has corrected a parameter bank or an authored law and asks which retained
  fits, runs and analyses derive from it, including through fit-to-propagation or seed chains; the maintainer facing stale
  derived work in long-lived history.
- **Current mechanism.** `canonical_explanations.rs` anchors are `StudyPoint`, `Run`, `AnalysisNode` and `Artifact`. A source
  revision is not an anchor. `Run` supports only `RecordedSources`, going from run to revision; the forward question has no
  scope. Indexes exist for run to revision on the primary revision, and `canonical_run_sources` is indexed on `(run, key)` and
  `(run, revision)` with no reverse index on revision alone. The type comment says no transitive or all-path completeness is
  implied. `dependency_analysis` is numerical incidence, not result impact. `pse-project` `LifecycleExplain` and retention
  roots address reachability for retirement and are the closest neighbour, not traced.
- **Why it matters.** A store with exact lineage could answer this directly; today the honest answer is manual traversal.
- **Patterns.** LOP-12, LOP-05.
- **Evidence.** Read `canonical_explanations.rs` (`GraphAnchor`, `GraphScope`, `sql`, `graph_explanation`), generated
  `surreal.surql` index definitions, `analyses.rs::dependency_analysis` header, §19.2. Interface-checked.
- **Footprint.** Data-document and parameter-bank edges (`canonical_edges`, `research`) not traced; research publication
  ancestry may supply part of the chain.
- **Preservation constraints.** Exact scan budgets for bounded reads and admission gates; immutability, since "affected"
  must not rewrite or invalidate immutable records; no scientific certification by derivation; bounded output.
- **Counterevidence and uncertainty.** Recursive graph queries can break the index-entry budget the design relies on, and an
  offline analysis may suit better. Retained history is young. Which edge kinds propagate (does a seed taint a result?) is a
  policy question not yet answered.
- **Status.** Proposed.
- **Research questions.** Can recursive traversal over existing edges respect the scan budget, or does client-side reachability
  over a frozen page set serve better?

<a id="cbo-146"></a>

## CBO-146 — Jacobian-derived scaling proposals

- **Operation and consumers.** A modeller with a badly scaled flowsheet asking for suggested nominals, as the IDAES
  autoscaler and scaling toolbox offer. The coverage appendix marks scaling "Partial".
- **Current mechanism.** Resolved policy, nominals and reversible normalization (`pse-math::normalization`); an Ipopt
  gradient-based option; conic Ruiz equilibration inside solvers; bounded Jacobian SVD diagnostics. No hit for a suggestion
  surface in `conditioning.rs`, `diagnostics.rs` or `workflow/numerics.rs`. Diagnostics describe poor scaling; nothing
  proposes authored nominals back.
- **Why it matters.** Scaling is a top practical failure source, and the Jacobian and SVD are already computed on the
  diagnostic path. IDAES scales variables by current magnitude and constraints by Jacobian norm; the algorithm is small,
  so the value lies in the proposal surface.
- **Patterns.** LOP-12.
- **Evidence.** Greps; §16 numeric text; the coverage appendix; IDAES `AutoScaler` at docstring level. Interface-checked.
- **Footprint.** `diagnostic_request.rs` and `normalization.rs` not read; search limited to the stated words.
- **Preservation constraints.** One resolved policy with recorded precedence; original-space acceptance; a proposed nominal
  is advice to the author, never silently applied, and relaxing hard guards is refused.
- **Counterevidence and uncertainty.** This may belong to small generic custom code with little library value, and may be
  better done as an authored-knowledge feature. The HSL equilibration routine is excluded by register R-34, though simple
  geometric equilibration needs no library.
- **Status.** Proposed.
- **Research questions.** Which retained Jacobian is the right input, and where would an accepted proposal land in the
  authored scaling declaration?

<a id="cbo-144"></a>

## CBO-144 — Solver-neutral problem export for reports and comparison

- **Operation and consumers.** The maintainer reporting a solver defect upstream, or comparing against a solver not linked
  here. Register R-41 records "reports not filed" and R-52 holds one hand-saved CIP file.
- **Current mechanism.** `workflow/reproducer.rs` extracts executable reproducers, but they need this stack. A search of the
  backend and PETSc bridge found no `SCIPwrite*`, `Highs_writeModel`, `write_model`, `SNESView` or `PetscViewer`, though the
  solver prefix ships `bin/{scip,ipopt,…}` and `SCIPreadProb` is used. The one CIP fixture,
  `tests/fixtures/scip/bt-pr-liquid-tpd.cip`, is referenced from `scip_tests.rs`. The docs record native defects repeatedly
  (a SCIP superindicator crash, a cardinality copy, a PETSc NASM setup failure).
- **Why it matters.** An upstream maintainer can act on a standalone CIP, MPS or LP file, not a Rust workspace. Comparing a
  suspect result with another solver gets cheap. §25's refusal of production NL routes concerns execution, not an opt-in output.
- **Patterns.** LOP-12, LOP-03.
- **Evidence.** Greps over `crates/pse-backend-native` and `crates/pse-petsc-sys/bridge`; §18.10.1 readback text; R-41, R-52.
  Interface-checked; adapters not read in detail.
- **Footprint.** How the existing CIP file was produced was not traced.
- **Preservation constraints.** Never an execution route and no identity effect; opt-in and bounded; labelled as a
  native-projected, normalized model, not the original; no secrets or licence in artifacts; numeric formatting must round-trip
  if it is to reproduce a defect.
- **Counterevidence and uncertainty.** Small scale, one maintainer, low frequency. Callback NLPs have no file form without
  evaluation (POUNCE has an NL crate; an NL writer for Ipopt would be custom). A debug build of a local solver command-line
  tool may be enough.
- **Status.** Proposed.
- **Research questions.** What do the native write functions emit for a transformed problem, and which prepared program is the
  right one to export?

<a id="cbo-150"></a>

## CBO-150 — Sparse preconditioning is thin

- **Operation and consumers.** Krylov solves of large sparse roots and DAEs, the alternative when direct factor admission
  refuses ([CBO-125](native-solver-integration.md#cbo-125)); users with large stiff flowsheet DAEs.
- **Current mechanism.** IDAS offers Jacobi or banded block-diagonal preconditioning (difference-based bands, restricted to
  SPGMR). `IdasLinear` with `Preconditioner::BlockFactor` returns "IDAS block factor preconditioning is not supplied", and
  `idas/coupled_probe.rs` is a test-only inquiry with no production route. KINSOL offers Jacobi or block factor, and
  `kinsol/block.rs::Blocks::setup` calls `feral::SparseLu::factor_markowitz` afresh per block at each setup. There is no
  incomplete factorization or algebraic multigrid route. PETSc serves roots (`petsc.rs`) but not dynamics.
- **Why it matters.** Flowsheet DAEs are stiff and sparse with block structure, and a structural block-triangular witness already
  exists for roots. Better preconditioning would make the Krylov routes usable at scale.
- **Patterns.** LOP-12, LOP-07, LOP-X001.
- **Evidence.** Read `layout.rs::linear_bytes`, `idas.rs::linear_solver`, `kinsol/block.rs`, the `coupled_probe.rs` header.
  Interface-checked; the POUNCE checkout's `vendor/feral/src/lu/sparse_factor.rs` shows refactor counters whose use was not read.
- **Footprint.** `petsc.rs` read at its head only.
- **Preservation constraints.** Analytic Jacobian-vector products stay exact and approximate preconditioners stay separate;
  admission before factors; no stale factor across applications; IDAS supports left preconditioning only here.
- **Counterevidence and uncertainty.** The test-only `coupled_probe` suggests the idea was investigated and not promoted, for
  reasons unknown. Demand at scale is proposed; current workloads may be small enough for the direct sparse factor.
- **Status.** Proposed.
- **Research questions.** Could a sparse-linear-algebra or PETSc preconditioner supply numeric-refresh block factors or
  incomplete factors behind the SUNDIALS preconditioner hooks?

<a id="cbo-149"></a>

## CBO-149 — Integral objectives become extra states rather than native quadrature adjoints

- **Operation and consumers.** Gradients of integral objectives in shooting and fitting.
- **Current mechanism.** `admit_adjoint` refuses declared quadratures ("adjoint gradients observe sampled outputs only"), and
  §25 lists adjoints over declared quadratures as refused. `Anchored::new(inner, observe_quadratures)` therefore integrates each
  quadrature as an extra differential state observed as an output, with layout growth, an `atol` extension and
  `out_rtol = None`, so quadrature error control becomes state error control. Both libraries natively support integral
  functionals: Diffsol documents a backward pass for `∫u(y(t))dt` with `integrate_out`, and IDAS has backward quadrature, already
  used for parameter gradients here. `QuadratureFlux`-as-`out` is unused on the Diffsol adjoint route because the profile refuses it.
- **Why it matters.** The state dimension grows, enlarging the Newton matrix and error control.
- **Patterns.** LOP-03, LOP-05.
- **Evidence.** Read `dynamics.rs::admit_adjoint`, `anchored.rs` and `anchored/layout.rs` production code, the use in
  `workflow/dynamics.rs`, the Diffsol adjoint documentation. Interface-checked.
- **Footprint.** `workflow/shooting.rs` callers seen by search only. Related preparation costs are
  [CBO-128](native-solver-integration.md#cbo-128).
- **Preservation constraints.** The conservation report, which `Anchored::new` refuses to combine with observation; quadrature
  error control; selected parameter layout; a cotangent callback that reads samples rather than a declared functional.
- **Counterevidence and uncertainty.** The wrapper's main job is anchors as parameters and it is carefully done. The observed-state
  form gives one "functional of sampled outputs" contract across both backends and multiple shooting, a real simplification, and
  error-controlled integrated states may be what the tolerance contract wants. A native integral route needs a new cotangent shape.
- **Status.** Current mechanism; Proposed (native alternative).
- **Research questions.** What does a mixed functional of samples plus integrals require from each library's backward route?

<a id="cbo-151"></a>

## CBO-151 — IDAS controls and native diagnostics are a narrow subset

- **Operation and consumers.** Scientists tuning stiff runs, and developers diagnosing failed ones.
- **Current mechanism.** `IdasSettings` has no route to `IDASetMaxOrd`, `IDASetMaxStep`, `IDASetMinStep`, `IDASetNonlinConvCoef`,
  `IDASetMaxNonlinIters`, `IDASetEpsLin`, `IDASetLSNormFactor` or step and eta controls; `IDASetSuppressAlg` appears in a test only.
  Diffsol, by contrast, exposes its complete option struct through serde remotes. The `build.rs` lines read define no logging level
  or profiling option, so `SUNLogger` per-step output is presumably unavailable. Diagnostics are hand-listed counters with
  hand-coded "is this getter current" rules (`statistics`, `solve_application`), the `local_error` capture and the KLU observer patch.
- **Why it matters.** Maximum step is a common control for sharp inputs and sparse sampling. When Newton fails, step-level history is
  what a user needs and currently requires local instrumentation.
- **Patterns.** LOP-12, LOP-03.
- **Evidence.** Search of crate `src` for those option names; `vendor/sundials-sys/build.rs` (`.define` listing, head only);
  `idas.rs::statistics`; `kinsol.rs::solve_application`; `local_error.rs`. Interface-checked.
- **Footprint.** `build.rs` was searched, not read in full; an option could be reachable through a name the list missed.
- **Preservation constraints.** Getter currency rules are real (stale linear counters; `IDAGetLastStep` exposing the artificial
  initial-condition step); bounded, non-stderr output; profile identity must cover any new control.
- **Counterevidence and uncertainty.** Few controls may be deliberate: library defaults are robust and each control adds profile
  surface. Logging is compile-time and file-oriented, and its text is not a typed record; the custom diagnostics are probably more
  useful for typed reports.
- **Status.** Proposed.
- **Research questions.** Can a memory-stream logger feed bounded diagnostics in a dedicated build, and what would a profiling build
  show relative to the KLU observer?

## Examined and not retained

- **Utility minimization and composite curves.** §19.6 and the coverage appendix record it as not implemented, and §0.2 lists it in
  scope, so it is a promise with a small footprint and a plain consumer (a process engineer targeting minimum utilities). It is an
  algebraic overlay over equipment heat duties, expressible as authored laws and a smooth operator, so library leverage is minimal
  and it reads as a domain-knowledge port; composite curves would want the rendering in [CBO-143](#cbo-143).
- **Ad hoc SQL over retained results.** Plan 28 retired SQL convenience deliberately, keeping DataFusion for necessary capabilities,
  and a frozen study cohort already exports to IPC for any external engine. Only the fixed-aggregate point remains, folded into
  [CBO-034](numerical-workflows.md#cbo-034).
- **Store evolution.** The mechanism exists: `canonical_migration.rs` (`SchemaMaintenance`, ordered edges, activation receipt,
  drain-before-apply), frozen declarations under `pse-operations/src/migrations/`, and the §20 forward rollout (ADR-0182). The
  leverage question is hand-frozen steps against a schema tool, which belongs to
  [CBO-069](persistence-and-transport.md#cbo-069).
- **Delivered but understated.** Multistart, continuation in general, adaptive surrogate, infeasible-subsystem analysis, structural
  diagnostics, sensitivities, project packs and reducers exist; the documents understate them.
- **Not scheduled and without named demand.** Electrolytes, eNRTL, broader reactions and costing tables are authored-knowledge ports
  needing no library beyond what is linked, and the thermodynamic data work is recorded as far-off. A live plant connection for
  rolling-horizon control (the horizon has a `Measured` contract) and stochastic or multi-period problems have hypothetical demand;
  composition through children and indexes was not verified for the latter, so no absence is claimed.
