# Design review: target architecture for full idaes-pse modeling capability

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | The **target design** for recreating the full idaes-pse 2.13.0 modeling capability in Rust. It is set out in [Plan 20](../../plans/20-idaes-capability-target.md) and its companion target documents: [thermodynamics](../../plans/20-target-thermodynamics.md), [modeling and flowsheets](../../plans/20-target-modeling-and-flowsheets.md), [numerical strategies](../../plans/20-target-numerical-strategies.md) and [coverage map](../../plans/20-idaes-coverage-map.md). The **current architecture** is assessed as the baseline for that target. Boundary: the modeling stack (materials, properties, templates, control volumes, laws, units, flowsheets, analysis modes), the numerical knowledge it needs (initialization, scaling, diagnostics), studies and applications. Suppliers and consumers read: the authoritative sections §0–§26, `pse-kernels`, `pse-runtime::workflow`, `pse-compiler::typed_math`, `pse-math::typed`, `pse-structural`, `pse-backend-native` dynamics and initialization. |
| Standard | Core **3.0** (AP-01–AP-06, DP-01–DP-24, G1–G9); process-simulator profile **1.1** (PS-01–PS-13, PS-G1–PS-G3); binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (binding default). Authority text that blocks the target is recorded as a required change in slot 11. |
| Reviewer / date | Agent review in the maintainer's session, 2026-09-26. The same agent authored the target documents, so this is an **author review, not an independent review**. |
| Decisions | Baseline fitness for the target: **G9 fail**; behaviour adequate within the current declared scope. Target design: architectural fitness **satisfied at the Proposed level**; behavioural adequacy **adequate by design with named measurement items**; overall **Accept-scoped (design direction, Proposed)**, see slot 12. |
| Disposition owner | [Plan 20 *Finding dispositions*](../../plans/20-idaes-capability-target.md#finding-dispositions) (draft; proposed owner only, and nothing is scheduled). |

**Functional target.** Everything idaes-pse 2.13.0 lets an engineer model:

- property frameworks, modular properties, the Helmholtz, cubic, ideal and activity models,
  electrolytes, reactions and transport;
- flowsheets, ports and arcs, control volumes 0D and 1D, the core unit library, and
  `models_extra` (power generation, columns, gas-solid, gas distribution, adsorption,
  membranes);
- controllers, costing, surrogates;
- initialization, scaling, diagnostics, dynamics, parameter estimation, sensitivity, sweeps and
  the applications (grid integration, NMPC/MHE, MatOpt).

All of this is to be delivered through a design that satisfies the core foundations and the
profile better than IDAES does. The maintainer stated that FeOS need not remain part of the
solution and that the design should be grounded in the best target, only loosely in the current
implementation.

**Evidence base.** IDAES is characterized statically in the `pyomo-and-solvers` skill, with the
full source trees for idaes-pse 2.13.0, idaes-ext 3.4.2, idaes-examples 2.10.0 and idaes-ui
0.25.10 and these records:

- 15,804 declarations;
- 257 generated process-block classes, 156 of them declaration-scope model records with
  configuration, guarded components, delegated control-volume construction and routines;
- 70 modular-property method classes;
- 1,989 function semantics;
- 23,371 upstream test oracles from 7,273 tests;
- 2,559 documentation equations;
- an idaes-ext inventory of 770 functions, 246 ASL registrations and 19 ELF binaries.

The pinned FeOS 0.10.1, num-dual 0.14.2, feos-core and candidate crates were read from the
Cargo registry. The current source was read at the seams listed in slot 2.

**Analysis modes and workloads in scope.** Square simulation, optimization, dynamics (integrated
and simultaneous), parameter estimation and sensitivity. Workloads: edit → re-solve, studies,
recycles, dynamics, extension (profile functional target).

**Not examined.** Runtime behaviour of any proposed mechanism; measured expression sizes and
preparation cost; convergence robustness of equation-oriented VLE versus nested flash; the
publication layer (unaffected); the Python boundary beyond noting that it consumes the same
declarations; any UI, DMF or CLI capability (out of target).

**Variation axes.** New model forms (correlations, equations of state, gᴱ models); new closure
algorithms; new unit templates and presets; new formulation policies (VLE, ΔT, valve
characteristic); new analysis modes and studies; library replacement at integration owners;
parameter data sets.

## 2. Decomposition, ownership and dependencies

**Baseline** (current source):

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State / effect owner | Local test setup |
|---|---|---|---|---|---|
| `pse-kernels` providers (`lib.rs`, `feos.rs`, `valve.rs`) | Model and algorithm together: FeOS PC-SAFT/DIPPR parameters, outputs, derivatives, envelope | `ProviderSpec` (scalar ports only, `lib.rs:190`; one fixed `Phase`); `Provider::evaluate(&[f64])` | Foundation → kernels | Per-trial cache in the worker | Unit tests in-crate |
| `pse-runtime::workflow::sources::factory` | Provider selection by string (`kind == "feos-pcsaft-dippr"`, `sources.rs:267`) and FeOS-specific translation from rows | `native_providers` relation, whose columns are FeOS-specific (`pcsaft_cas`, `ideal_gas_cas`) | Runtime → kernels | — | Requires runtime crate |
| `workflow::composition` + `composition/lower.rs` (1,726 lines) | Template selection, parameter and feature typing, guards, domains, symbol expansion, case targeting, equation rewriting, law lowering, submodels, connections, tears | Authored template relations → computation definitions and balances | Runtime → compiler | None (pure logic in an effectful crate) | Requires runtime crate (DataFusion, Tokio in the closure) |
| `workflow::balances`, `reactions`, `vessel`, `dynamics` | Scalar law projection; homogeneous rate projection; hand-coded vessel recipe with fixed CAS numbers; dynamic declaration preparation | Balance rows, reaction applications, dynamic cases | Runtime → compiler, backend | — | Requires runtime |
| `pse-compiler` (Salsa) | Body admission, plans, structure, artifacts | `Inputs` from the frozen revision | Compiler → math | Salsa workspace | Crate-local |
| `pse-math`, `pse-structural`, `pse-backend-native` | Library mathematics; matching, DM, BTF; native execution and qualification | Bodies, plans; oracles; outcomes | Well-layered (§1.1) | Attempt-owned workers | Crate-local |

**Target** (Plan 20 diagram): the model semantics move to pure crates, and the runtime keeps only
effects.

| Component / responsibility | Decision hidden | Contract | Dependency direction | Local test setup |
|---|---|---|---|---|
| `pse-properties` | Property kinds; model-form typing; identity rules; package admission and facts; resolution with lineage; closure formulations | `resolve(package, state_def, context, demand) → PropertyProgram` | → `pse-quantity`, `pse-material`, `pse-math`, `pse-authoring` | Symbolica only; no runtime or native solvers |
| `pse-thermo` | Closure algorithms (cubic roots, density, flash, pure-fluid inversion) over compiled programs | Generalized closure contract (thermodynamics document §4.9) | → `pse-properties`, `pse-math`, faer | Compiled model programs; no flowsheet |
| `pse-modeling` | Specialization, presets and slots, package-fact guards, indexed law engine, reaction and cost index maps, discretization, analysis-mode generation | Specialized definitions consumed by compiler queries | → `pse-properties`, `pse-authoring`, `pse-quantity` | Declarations only |
| `pse-compiler` | Tracked specialization, resolution, admission and plans | Unchanged outward | → `pse-modeling`, `pse-math` | Crate-local |
| `pse-runtime` | Admission boundary, strategies, studies, jobs, results, publication | Unchanged outward | Composition root | Integration tests |

The composition root stays `pse-runtime`. What varies independently: model forms (data), closure
algorithms (registered implementations), templates (data), formulation policies (package or
analysis data), library backends (integration owners).

## 3. Contracts, authority and constraints

| Meaning / contract | Authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|
| Thermodynamic model (potential or correlation) | Model-form declaration in a package (data), admitted by `pse-properties` | Consumers request property kinds, never a model's internals | Physical typing, domain obligations, envelope and parameter-axis completeness at admission; refusal with source identity | Identity-expanded expressions, compiled programs (derived, keyed) |
| Explicit property override (parity formulations) | The model form that declares it | Recorded in lineage; consistency checks report it as a declared inconsistency | Admission requires the override's kinds to be ones the identity would otherwise provide | — |
| Property package | Package declaration | Templates bind packages by capability type | Capability checked at binding; missing method → refusal naming kind and index | Package facts (typed) |
| Property demand and derivation | Resolver (Salsa) from consumer demand plus package rules | Consumers name kinds; lineage is published with results | Cycle → typed refusal with the chain | `PropertyProgram` (derived, keyed on package, state definition, context, demand) |
| Closure formulation | Package or analysis policy | Result records the formulation, ε and route | Unsupported closure/model pair refused at preparation (DP-15) | Equations, or a provider call with a declared derivative source |
| Unit or control-volume structure | Template (data), preset (binding data), slot implementation (template) | Instances bind parameters and slots; no code | Admission of guards, facts, multiplicity and ports; IDAES `ConfigurationError` cases become admission invariants | Specialized definitions (derived) |
| Balance | Law instance over contributions with index maps | New unit adds contributions, not balances | Every active contribution has one law; index maps typed; closure per member | Balance rows, closure checks |
| Discretization | Discretization policy (instance or analysis) | Continuous domains never reach the compiler undiscretized | Unsupported scheme/domain refused; index check after generation | Finite domains with mapping back to `(symbol, coordinate)` |
| Analysis mode | Case / analysis declaration | Same templates for every mode | Mode-generated accumulation only where holdup exists; dynamic without holdup refused | Mode-specific rows |
| Initialization knowledge | Estimators, stages and homotopy policy declared on packages and templates | Starts are inputs with provenance; stages are overlays | Transactional engine (existing); failed stage leaves the specification intact | `InitializationReport` |
| Numerical magnitudes | Resolved policy with sources: analysis, case, model, template/package hints, property defaults, derived nominals, quantity nominal, fallback | Adapters never reinterpret | Equal-rank conflict refused (existing) | Normalization (existing) |

**Physical-semantics table.** The property-side table is
[thermodynamics document §5](../../plans/20-target-thermodynamics.md#5-physical-semantics-table-profile-slot-3).
Model-side additions:

| Quantity or model element | Dimension and unit | Basis | Reference / convention | Validity | Authority |
|---|---|---|---|---|---|
| Material flow term `[p, j]` | mol/s or kg/s | Package flow basis (typed) | — | Nonnegative unless reversal is declared | State-definition term contract |
| Enthalpy flow term `[p]` | W | — | Package enthalpy datum | — | State-definition term contract |
| Heat, work, enthalpy transfer | W | — | Sign: into the control volume is positive (§10.2) | — | Law contribution role |
| Pressure change `deltaP` | Pa (difference scale) | — | Difference, never a point | — | CV template |
| Reaction extent contribution | mol/s | Reaction rate basis | Stoichiometric sign convention | Rate-form envelope | Reaction package + index map |
| Axial derivative terms | per normalized length | — | `x ∈ [0, 1]`, flow direction declared | — | CV1D template + discretization |
| Cost | currency / year basis | — | Cost-index year (typed unit) | Correlation range | Costing forms + conversion rules |

**Well-posedness statement.**

- **Variable roles.** Roles stay explicit (§7.6): fixed specification, free unknown, parameter,
  start. Start estimators produce *starts*, never fixes. Case specifications target template
  symbols by identity.
- **Degree-of-freedom and structural analysis** run before any solver, over the complete
  specialized case, which now includes resolved property programs and discretized domains
  (§15.2 kept).
- **What is rejected before a solver runs:** structural singularity, over- or under-specification,
  a dynamic index above 1 without a declared reduced formulation, an unsupported closure/model
  pair, an envelope-less extrapolation, a cycle in property derivation.
- **Diagnostics** name units, ports, law members, property kinds and discretization points,
  through the mapping kept by each transformation (PS-04, PS-05).

## 4. Change scenarios and composition

The scenarios refine binding seeds [PSE-S01](../design_principles/binding/pse-arrow.md#pse-s01)–PSE-S06.

| Scenario / stimulus and conditions | Expected response and change boundary | Edit / composition path (target) | Observed baseline impact | Acceptance and evidence |
|---|---|---|---|---|
| <a id="s01"></a>S01 Add a pure-component correlation (for example RPP5 cp) | One model-form declaration and focused formula tests (§E) | Package data. The resolver picks it up through method selection; identities integrate h and s from cp with the declared datum | The `methods` package declares it but nothing executes it (§9.3). Each consuming model re-authors the formula as a computation definition (F03) | The form admits and resolves; the formula test against the source publication passes. Proposed |
| <a id="s02"></a>S02 Add a unit from existing physics (IDAES `Flash`) | One template file plus a preset; no code | CV0D child with phase-equilibrium binding, two phase-split outlet ports, VLE policy from the package | Not expressible: package child (`lower.rs:109`), material domains (:507), indexed laws (:1267), delegated ports (:1564) are refused (F04, F06) | Admits and specializes; conformance against `idaes-oracle:` values for the IDAES flash tests with the same parameters. Proposed |
| <a id="s03"></a>S03 Add an EoS family (SRK or SAFT-VR Mie) | One Helmholtz model form; all properties from identities | Package data plus existing closures | A new Rust provider hand-coding each output, plus a `sources.rs` string-match edit and FeOS-specific relation columns (F01) | Identity-consistency and reference checks pass for the new form. Proposed |
| <a id="s04"></a>S04 Replace a thermodynamic implementation (FeOS PC-SAFT → native form; or upgrade FeOS) | Integration owner absorbs it; consumers unchanged | Swap the package's model selection; FeOS becomes a test oracle | The FeOS model is embedded in the provider and the `native_providers` columns; the num-dual family is coupled to FeOS (R-05) (F01, F13) | PC-SAFT conformance versus FeOS oracle; FeOS production code deleted in the same change. Proposed |
| <a id="s05"></a>S05 Switch a flowsheet from steady to dynamic | Change the case's analysis mode, horizon and initial conditions | Law engine generates accumulation; integrated or simultaneous route | Templates refuse derivative symbols (`lower.rs:839`); dynamics authored separately as dynamic cases or the vessel recipe (F08) | Same revision identity for templates; dynamic closure per law member. Proposed |
| <a id="s06"></a>S06 Distributed PFR or HX1D with collocation | Instance discretization policy | CV1D template + discretization transformation | Continuous domains refused (`lower.rs:471`); §13.4 retired (F09) | Discretization residuals at the solution; refinement study converges. Proposed |
| <a id="s07"></a>S07 Change a package's VLE formulation (smooth → complementarity → nested flash) | Package policy change only | Closure registry selects the formulation; units unchanged | Phase equilibrium refused (§9.5); DSL refuses smoothing names (F05) | Same flowsheet solves under each; post-solve stability check passes. Proposed |
| <a id="s08"></a>S08 Edit → re-solve (value) and a sweep of 1,000 points | Structure prepared once; value bindings only | Salsa-tracked specialization and resolution reuse; study runner | A sweep re-freezes each revision, re-running untracked lowering in the runtime (F07) | Per-point preparation work is value-only (measure). Proposed |
| <a id="s09"></a>S09 Flash-heavy recycle flowsheet from default guesses | Estimators and relaxation stages give a start; transactional engine; tears | Package and template declarations composed by the initialization policy | No model-supplied starts; initializer templates refused (`lower.rs:78`); `initialize_npt` has no caller (F10) | Convergence study over sampled feeds; specification intact on failure. Proposed |
| <a id="s10"></a>S10 An iterate leaves a correlation's range | Recoverable envelope failure or selected extrapolation recorded | Envelopes compiled as obligations for authored forms | Envelopes enforced for providers only; correlation intervals are unchecked data (§9.10) | Envelope rejection test per form. Proposed |
| <a id="s11"></a>S11 Test property resolution, law expansion or discretization locally (PSE-S05) | Pure crate tests with declarations only | `pse-properties` / `pse-modeling` unit tests | Composition and law lowering live in `pse-runtime` (F07) | Tests run without runtime, DataFusion or native solvers. Proposed |
| <a id="s12"></a>S12 Add costing to a flowsheet | Costing bindings + one flowsheet costing template | Cost contributions over accounting laws; cost-index conversions | No costing relation exists; only `LawFamily::cost` and enum names (F04) | Aggregation closure; SSLW correlation tests. Proposed |
| <a id="s13"></a>S13 Over-specified flash or ill-conditioned column | Structural refusal naming units and members; numerical diagnostics catalogue | Structural admission (exists) + named analyses | Structural refusal exists (§15.2). No SVD, degeneracy or near-parallel analyses (§15.5) (F12) | Diagnostics name model elements; IDAES diagnostics oracle cases. Proposed |
| <a id="s14"></a>S14 Parity validation of a unit (PS-13) | Oracle values with the same parameters, references and tolerances | Conformance suites keyed by `idaes-oracle:` IDs | Parity harness exercises names only (§21) | Per-unit conformance reports. Proposed |

## 5. Mechanisms and execution

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|
| Property resolution (`pse-properties`, Salsa) | Demand-driven derivation with a fixed selection order; lineage | Package, state definition, context, demand | Pure | Keyed on complete inputs; backdates unchanged packages | Proposed |
| Identity expansion (`pse-properties` via `pse-math`) | Symbolic differentiation of authored potentials; explicit overrides | Model forms | Pure | Exact (real algebra on the admitted domain); expression size unmeasured (R1) | Proposed; R1 open |
| Closure algorithms (`pse-thermo`) | Nested roots, flash, inversion with implicit derivatives; residual-checked convergence | Compiled model programs | Attempt-owned workers; internal warm start declared as a determinism class | Piecewise-smooth by regime; regime observable | Proposed; the FeOS defect shows why residual checks are mandatory |
| Specialization (`pse-modeling`, Salsa) | Templates, presets, slots, facts, multiplicity, ports | Authored declarations, packages | Pure | Incremental by instance | Proposed |
| Law engine (`pse-modeling`) | Indexed subjects with contribution index maps | Specialized templates | Pure | Closure per member | Proposed |
| Discretization (`pse-modeling`) | FD and collocation with an identity mapping | Continuous domains, policy | Pure | Consistency order *k*; Symbolica-certified nodes | Proposed |
| Initialization (runtime strategies, existing engine) | Estimators → stages → blocks → homotopy | Prepared case, declarations | Transactional overlays (existing) | Starts recorded as dependencies | Engine Implemented; knowledge Proposed |
| Derived nominals (`pse-math`) | Term magnitudes at nominals; IDAES scheme names | Compiled bodies | Pure artifact | Ranked source with provenance | Proposed |
| Diagnostics (structural, quality, faer, HiGHS) | Named catalogue with a threshold profile | Prepared case, candidate | Opt-in bounded analyses | Never change outcomes | Structural Implemented; rest Proposed |

**Numerical stage columns** (profile) are in
[numerical strategies document §8](../../plans/20-target-numerical-strategies.md#8-numerical-stage-columns-profile-slot-5).
The load-bearing points:

- the derivative source is recorded per link (Symbolica exact, or implicit from converged
  closures);
- no silent finite difference exists anywhere;
- status maps to outcome through the existing envelope;
- post-solve checks add VLE stability and closure per law member.

## 6. Architectural assessment and gates

### 6.1 Baseline architecture against the target

| Foundation | Scenario and evidence | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | S08, S11: model semantics (specialization, laws, reactions, recipes) sit in the effectful `pse-runtime`. `lower.rs` and `freeze` mix unrelated responsibilities (F07). The FeOS provider mixes model and algorithm (F01) | **violated** | F07, F01 |
| AP-02 Stable contracts | S03, S04: the provider contract is scalar and single-phase; the registry's `native_providers` columns expose FeOS internals (`pcsaft_cas`); selection is a string match (F01) | **violated** | F01 |
| AP-03 Composition | S01, S02, S12: each unit must hand-wire property calls. Composition lacks package children, multiplicity, delegated ports and slots. Laws are scalar only (F02, F04, F06) | **violated** | F02, F04, F06 |
| AP-04 Authoritative meaning | S01: method packages declare correlations that execute only when re-authored per model (F03) | **violated** | F03 |
| AP-05 Explicit structure | Every unsupported construct is refused at a named site with source identity; admission accounting is explicit | **satisfied** (strength) | Keep |
| AP-06 Local reasoning / testability | S11: law and composition tests need the runtime crate (F07). Property methods cannot be tested because they do not execute (F03) | **violated** | F07, F03 |

| Gate | Result | Evidence | Required action |
|---|---|---|---|
| G1 Authority | **fail (latent)** | F03: two writable definitions of one correlation, with no reconciliation | P0.5 |
| G2 Semantic fidelity | pass | Complete quantity types; explicit absence (§7.6) | — |
| G3 Validity | pass | Refusals at admission (slot 2 sites) | — |
| G4 Hidden behaviour | pass | Tracked queries pure; diagnostics non-mutating | — |
| G5 Consistency and recovery | pass | Transactional initialization; truthful completion | — |
| G6 Transformation and reuse | pass within scope | Keys complete; lowering outside Salsa costs reuse, not correctness (F07) | — |
| G7 Truthful capability claims | pass | §11.1 and §25 state that the reference unit and state templates are non-executable; refusals are explicit | — |
| G8 Library leverage | pass | Library-owned math and solvers; the FeOS density defect is contained by the pressure-closure check (`feos.rs`) | — |
| G9 Architectural fitness | **fail** | AP-01, AP-02, AP-03, AP-04 and AP-06 violated for S01–S04, S08, S11, S12 | Plan 20 |
| PS-G1 Physical consistency | pass within scope | Typing, datum and envelope rules | — |
| PS-G2 Well-posedness | pass | Structural admission before solving (§15.2) | — |
| PS-G3 Numerical integrity | pass within scope | Outcome envelope, original-space qualification | — |

The baseline is behaviourally adequate for its declared scope and **architecturally unfit for the
target**. This is not a defect in its supported outputs. The ordinary target extensions cannot be
expressed without changing the core contracts that the findings name.

### 6.2 Target design (Proposed)

| Foundation | Scenario and evidence | Verdict | Required action |
|---|---|---|---|
| AP-01 | Models, closure algorithms, resolution, composition, numerical knowledge and effects each have one owner (slot 2). S01 and S03 touch only packages. A new closure algorithm touches only `pse-thermo` | satisfied (design) | — |
| AP-02 | Consumers depend on property kinds and package capability types, never on model or closure internals. Closure selection is by declared capability; capability differences such as derivative source and regime behaviour stay visible (thermodynamics document §4.9) | satisfied (design) | — |
| AP-03 | Presets, interface slots, law contributions, package-fact guards and indexed children let every IDAES subclass and callback variant be expressed as data (modeling document §3.2) | satisfied (design) | — |
| AP-04 | One potential defines its derived properties; overrides are declared; presets are binding data; template syntax is a projection of the same declarations | satisfied (design) | Keep the override rule strict (only kinds the identity would provide) |
| AP-05 | Package facts, formulation policies, analysis modes, discretization policies and closure regimes are declared and inspectable | satisfied (design) | — |
| AP-06 | Pure crates; conformance suites run without the runtime | satisfied (design) | — |

| Gate | Result | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 | pass (design) | One authority per meaning (slot 3) | — |
| G2 | pass (design) | Index shapes, bases and reference states on every property kind | — |
| G3 | pass (design) | Admission invariants replace IDAES construction exceptions | — |
| G4 | pass (design), with a verification item | Nested closures' internal warm starts could influence results; the design requires a declared determinism class | Define the class when `pse-thermo` lands |
| G5 | pass (design) | Studies isolate failed points; initialization transactional | — |
| G6 | pass (design) | Identity expansion (exact), discretization (declared order), derived nominals (ranked source) and ε policies all carry contracts and keys | — |
| G7 | pass (design) | Every coverage-map row has an implementation route; all claims are Proposed | — |
| G8 | pass (design) | Libraries placed (slot 8); bespoke pieces bounded with reasons | — |
| G9 | pass (design) | All six foundations satisfied at the Proposed level | — |
| PS-G1 | pass (design) | Datums, bases, envelopes (including correlation validity), closure per member | — |
| PS-G2 | pass (design) | Pre-solve analysis over the complete specialized case, with a mapping back through resolution and discretization | — |
| PS-G3 | pass (design) | Declared formulation and derivative source; residual-checked closures; post-solve stability checks | R2/R3 measurement before fixing defaults |

## 7. Findings

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | **The provider contract fuses a thermodynamic model with its algorithm and exposes it as scalar, single-phase ports selected by string.** | AP-02, AP-01, DP-15, PS-02; G9; S03, S04 | `ProviderSpec::validate` refuses non-scalar ports (`pse-kernels/src/lib.rs:190`); one fixed `Phase` per spec; `FeosPackage` hard-codes outputs P, H, S, ln φ (`feos.rs:511-531`); `factory` string match (`sources.rs:267`); `native_providers` columns `pcsaft_cas`, `ideal_gas_cas` (`native_math.rs:680-745`) | Every new EoS is a Rust provider re-implementing each output. Multiphase closures cannot be expressed. Replacing FeOS changes a registry contract | Separate model forms (data) from closure algorithms (thermodynamics document §4.3, §4.9); generalize the contract (index shapes, phase sets, regime, derivative source); select by declared capability | S03 and S04 edit paths touch only packages or `pse-thermo` |
| <a id="f02"></a>F02 | **Property demand has no derivation model**, so each consumer wires each property call explicitly. | AP-03, DP-06, D8; G9; S01, S02 | §9.6: "global demand fixed point is not part of the current design". A kernel call selects one output per `native_providers` row (`model.rs:63-68`, `vessel.rs:149-155`) | IDAES models consume about 100 derived property kinds (99 on-demand builders on `GenericStateBlock`); hand-wiring them per unit multiplies authoring | Salsa-tracked acyclic resolution with lineage (thermodynamics document §4.7); amend D8 at the consumer boundary | A unit names kinds only; the resolver's lineage covers every property |
| <a id="f03"></a>F03 | **Method packages declare correlations that never execute; executing one means re-authoring it per model.** | AP-04, DP-01; G1; S01 | §9.3: "To execute a correlation today, author it as an ordinary typed computation definition". `method_specs`, `method_selections` and `property_packages` have no runtime consumer (source survey) | Two writable definitions of the same correlation can diverge. The tested formula (xtask) is not the executed one | Execute method forms through the resolver; delete the per-model copies | One definition per correlation, with tests against it |
| <a id="f04"></a>F04 | **The law engine is scalar-only.** Indexed subjects, element projection, default balances and cost subjects are refused. | AP-03, PS-03, D7; G9; S02, S12 | `lower.rs:1267-1285` refuses indexed, element and default-balance laws; reference laws (`units/laws/balances.yaml`) are all indexed | No multicomponent or multiphase control volume or costing aggregation can execute | Indexed law engine with contribution index maps (modeling document §4.2) | CV0D componentPhase, componentTotal and elementTotal expansions with closure per member |
| <a id="f05"></a>F05 | **No formulation primitives or policies exist for phase equilibrium and other nonsmooth physics.** | PS-06, DP-08; G7; S07 | Phase equilibrium refused (`sources.rs:273`, `feos.rs:141`); the DSL refuses `smooth_max`/`smooth_min`/`smooth_abs` by test (`pse-math/src/functions.rs:14-24`); only hard `min`/`max` (value-only) | IDAES SmoothVLE and complementarity VLE, Mixer pressure minimization and smooth LMTD cannot be expressed | Declared smoothing and complementarity primitives with typed ε; package VLE policies (thermodynamics document §4.8) | Positive and negative function controls; VLE formulations switchable per S07 |
| <a id="f06"></a>F06 | **Template composition cannot express the IDAES unit library**: no package-typed parameters or facts, material domains, state-block children, multiplicity, delegated ports, expression or reference symbols, or interface slots. | AP-03, DP-06, D2; G9; S02, S06 | Refusals at `lower.rs:109, :124, :340, :507, :1438, :1564`; the shipped `pse.units` and `states` templates hit them | The ~90-unit library cannot be data; IDAES callbacks and subclasses have no counterpart | Mechanisms in modeling document §3.2 | `pse.units` Heater, Mixer and FTPx admit (P0.3) |
| <a id="f07"></a>F07 | **Model semantics live in the effectful runtime and outside Salsa.** | AP-01, AP-06, DP-09, DP-17; G9; S08, S11 | `composition/lower.rs` 1,726 lines of mixed responsibilities; `model.rs::freeze` mixes selection, provider registration, a valve dimension check, lowering and admission; `vessel.rs` hard-codes species | Composition tests need the runtime; a value sweep re-runs untracked lowering per point | `pse-modeling` pure crate consumed by tracked queries (modeling document §11) | S11 tests run without runtime; S08 value-only preparation |
| <a id="f08"></a>F08 | **Steady and dynamic behaviour are authored separately.** | PS-11, DP-06; S05 | Templates refuse derivative symbols (`lower.rs:839`); dynamics declared through `authored.dynamic_cases` state and RHS rows or the `vessel` recipe | Switching mode means re-authoring; IDAES models (all with `dynamic`/`has_holdup`) have no single-model route | Analysis-mode generation of accumulation (modeling document §7) | S05 without template edits |
| <a id="f09"></a>F09 | **No continuous domains or discretization transformation.** | PS-11, DP-08; G7 (target scope); S06 | Continuous domains refused (`lower.rs:471`); §13.4 retired | PFR, HX1D, ShellAndTube1D, CV1D, columns, beds, pipelines and simultaneous dynamic optimization are all inexpressible | Typed discretization transformation (modeling document §6) | S06 |
| <a id="f10"></a>F10 | **Initialization knowledge has no home.** | PS-08, DP-05; S09 | Starts only from template defaults and case values (`lower.rs:991, 1065`); initializer templates refused (`lower.rs:78`); `initialize_npt` has no caller | Flash, recycle and column flowsheets depend on hand-supplied guesses; IDAES robustness knowledge is lost | Start estimators, relaxation stages, adaptive homotopy (numerical strategies document §2) | Convergence study (S09) |
| <a id="f11"></a>F11 | **Magnitudes are constants only; no derived nominals.** | PS-07; S08 | `pse-math/src/numerics.rs` sources are declared constants; §16.3 retired | Balances over large flows get row scales from quantity nominals only; IDAES scaler knowledge (21 default scalers) has no counterpart | Package and template hints plus derived term-magnitude nominals (numerical strategies document §3) | Scaling profile study |
| <a id="f12"></a>F12 | **The numerical diagnostics catalogue is incomplete.** | PS-04, PS-10, DP-21; S13 | §15.5: no SVD over general Jacobians, degeneracy search, infeasibility explanation or term analysis | Ill-posed or ill-conditioned models fail late without an explanation in model terms | Diagnostics catalogue (numerical strategies document §4) | S13 diagnostics name model elements |
| <a id="f13"></a>F13 | **FeOS cannot own the target property scope, and its density iteration reports unconverged roots as success.** | DP-15, PS-10; G8 (library fit); S04 | FeOS survey: no excess-Gibbs models; PR only; N > 2 TP flash f64-only; PcSaft transport f64-only; pure-only multiparameter; `density_iteration.rs` `iterations == maxiter + 1` never true | Keeping FeOS central yields two property architectures; the defect must be contained at every call | Native potential framework; FeOS as test oracle and algorithm reference (thermodynamics document §7) | S04 |

**Strengths to keep:**

- complete physical typing with basis and reference;
- library-owned mathematics;
- explicit, source-attributed refusal;
- structural admission before solving;
- class-specific solvers with truthful outcome envelopes and original-space qualification;
- transactional initialization;
- immutable revisions and Salsa preparation.

Each already exceeds the corresponding IDAES mechanism and is kept unchanged by the target.

## 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Bespoke code removed / recommendation |
|---|---|---|---|---|---|
| Symbolic algebra, derivatives, evaluators for model forms and identities | `pse-math` | Symbolica/Numerica (adopted) | Exact derivatives; CSE; jets. Expression size for multicomponent SAFT unmeasured | Already the math authority | **Adopt** (extends D6) |
| Helmholtz and SAFT property engine | `pse-properties` + `pse-thermo` | FeOS 0.10.1 | See F13. Multiparameter covers all 11 IDAES fluids but is pure-only, has no transport, and treats `tc`/`rhoc` as reducing values | Couples num-dual (R-05) | **Oracle and algorithm reference**, test-only |
| Pure-fluid reference data | packages | teqp's bundled CoolProp-format fluid files (MIT), IAPWS release values, NIST | Complete coefficient sets | Data provenance | **Adopt as data**; IAPWS and NIST as data-independent oracles |
| CoolProp | tests (Python group) | `coolprop-sys` / `rfluids` | Global lock; no composition derivatives | Conflicts with attempt workers | **Oracle only** |
| Cubic roots | `pse-thermo` | `roots` (values only) | No derivatives or branch typing | — | **Build bounded** (with implicit derivatives) |
| Flash and stability algorithms | `pse-thermo` | feos-core phase equilibria (adapt, with attribution); `vle-thermo` (reject: num-dual 0.11) | FeOS algorithms proven; must generalize to N components with derivatives | Oracle independence lost for adapted algorithms | **Adapt** |
| Activity models (NRTL, Wilson, eNRTL; UNIQUAC/UNIFAC beyond parity) | package data | none suitable | — | Group-data provenance | **Build as model forms** |
| Collocation nodes and weights | `pse-modeling` via `pse-math` | Symbolica `isolate_real_roots`/`refine_root_interval` | Certified roots | None new | **Adopt** |
| Discretization transformation | `pse-modeling` | none (Pyomo DAE is Python; DiffSL is a second language) | — | — | **Build bounded** |
| Index check | `pse-structural` | pounce-presolve matching (adopted) | Structural index-1 check exists for the algebraic partition | — | **Adopt**; no index reduction |
| Smallest singular values | runtime diagnostics | faer (`partial_svd` gives largest only) | Dense under budget or shift-invert adapter | Small adapter | **Adopt + adapter** |
| Degeneracy hunter, certificate LP, LP IIS | backend | HiGHS (adopted; IIS already wrapped) | — | — | **Adopt**; bespoke MILP formulations |
| NLP infeasibility explanation | runtime | none; Pyomo `mis` pattern | Elastic deletion filter | — | **Build bounded** orchestration |
| MPCC | DSL primitives + policies | POUNCE ℓ1 penalty (explicit route only) | Not recommended by its README for MPCC | No fallback | Smoothing primitives **built**; POUNCE ℓ1 optional explicit |
| Sampling | studies | `egobox-doe` (LHS), `sobol_burley` | ndarray duplicate (not a pinned family) | Seeds recorded | **Adopt** |
| Kriging | studies | `egobox-gp` | Fitted parameters private | Export coupling | Adopt or bespoke faer + Ipopt fit |
| NN import | studies | `tract-onnx` (layer translation) | Values only | — | **Adopt** for translation to authored forms |
| Global MINLP | — | SCIP, BARON | — | — | Out of the design target (§25) |

## 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | S05 partially (separate authoring), S08 partially | Strong admission; composition in runtime | Narrow modeling slice | Correct within scope | Not selected: cannot reach the target (F01–F13) |
| **Proposed design** (potential-based property system + template mechanisms + law engine + analysis modes + declared numerical knowledge) | S01–S14 each local to one owner | Pure crates; declared contracts | One math authority; data-driven library | Expression-size and EO robustness unmeasured (R1/R2) | **Selected.** Revisit the default closure routes after R1/R2 measurement |
| Library-owned alternative (FeOS-centred thermodynamics, extended with CoolProp) | S03 only for SAFT and multiparameter families; S01 and S07 not served | Two property architectures (FeOS models plus bespoke others) | num-dual AD beside Symbolica | Density defect; f64-only N-component flash; global lock (CoolProp) | Rejected. Revisit if FeOS gains excess-Gibbs models, N-component differentiable flash and a model-as-data interface |
| Fork FeOS as the architecture | S03 for Helmholtz families | Models remain Rust code | Second AD authority | Maintenance of a fork | Rejected as architecture; algorithms adapted (slot 8) |
| Simplest viable: imperative Rust builders per unit and property (port IDAES `build()` to Rust over `ModelBuilder`) | S02 served by code; S01, S03 and S07 need code; S08/S11 unaffected | Every variant is code; callbacks return as closures | Duplicated construction logic across ~90 units, like IDAES | Fast to start; violates D2, AP-03 and AP-04 at scale | Rejected: reproduces the IDAES maintenance surface the design exists to remove |
| Embed IDAES/Pyomo | Parity trivially | Violates D12 | Python in production | — | Rejected (D12, clean room) |

**Reference practice.** IDAES, gPROMS, Aspen Plus and Modelica-based tools each either hand-code
property methods per model or derive from Helmholtz or Gibbs potentials; FeOS and teqp show the
second is maintainable at scale. Equation-oriented simulators pair declarative models with
declared initialization knowledge, as proposed here. This is read for behaviour only.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| Baseline refusal sites and seams (slot 2, F01–F12) | Interface-checked | Source read at the cited lines | — | Established |
| FeOS capability and defects (F13) | Interface-checked | Pinned source read (`feos-0.10.1`, `feos-core-0.10.1`, `num-dual-0.14.2`) | — | Established; no FeOS code executed |
| IDAES target inventory | Interface-checked | Static skill records and full source | — | Established; nothing executed |
| Target foundations satisfied (slot 6.2) | Proposed | Design reasoning over the scenarios | — | Needs implementation evidence |
| R1 expression size | Proposed | Measure PC-SAFT (10/20 components) and IAPWS-95 preparation time and memory | Bounded preparation cost | Open |
| R2 EO VLE versus nested flash robustness | Proposed | Convergence study on IDAES flash and column oracle cases | Comparable or better convergence with estimators | Open |
| Unit conformance (S14) | Proposed | Oracle suites with the same parameters and references | Agreement within the IDAES test tolerances | Per packet |

## 11. Authority changes, exceptions and disposition

| Blocking authority text | Required change | Route |
|---|---|---|
| §0.2 scope table and [relationship to IDAES](../../relationship-to-idaes.md) (`models_extra`, surrogates, apps out of scope) | Bring them into the functional target (Plan 20 A1) | ADR + `design:` PR |
| ADR-0003 parity pin `idaes-pse==2.12.0` | Move to 2.13.0 to match the characterization (A2) | Short ADR (parity-pin rule) |
| D8, §9.6 (no demand derivation); D9 and §9.4 (scalar single-phase providers); §9.5 (phase equilibrium refused); §9.8 (FeOS as provider); ADR-0084/0088 parts | Potential-based property system, generalized closure contract, FeOS as oracle (A3) | ADR + design review |
| §3.2 crate table | Add `pse-properties`, `pse-modeling`, `pse-thermo`; move semantics out of `pse-runtime` (A4) | ADR + design review |
| D2 (kept) and §10.5, §11.1, §11.4 (no inheritance; refusals), §12.1 (local ports only), §22.3 | Template mechanisms and template syntax (A5) | ADR + design review |
| §10.1 scalar executable boundary | Indexed law engine (A6, within D7/ADR-0010) | ADR |
| §7.2 removed smoothing names | Formulation primitives (A7) | Short ADR |
| §13 (no discretization; separate dynamic authoring), retired §13.4 | Analysis modes and discretization (A8) | ADR + design review |
| §16.2 sources; retired §16.3, §16.4, §17.2, §17.3 | Declared hints, derived nominals, estimators, stages, homotopy (A9) | ADR |
| §15.5, §19.3–§19.8 | Diagnostics catalogue and studies (A10) | ADR |

No SHOULD exception is recorded. No MUST gap is accepted: the baseline's G9 failure is scoped to
the target and does not narrow the current supported scope. Dispositions: every finding links to
the [Plan 20 table](../../plans/20-idaes-capability-target.md#finding-dispositions) with
disposition `open` and a proposed owner. Nothing is scheduled.

## 12. Decision

- **Behavioural / semantic adequacy.**
  - The baseline is adequate within its declared scope: all core and profile gates pass there,
    and G1 fails only latently (F03).
  - The target design is adequate by design at the Proposed level, with measurement items R1–R3
    before the default closure routes are fixed.
- **Architectural fitness.**
  - The baseline fails G9 for the target (AP-01, AP-02, AP-03, AP-04 and AP-06 violated).
  - The target design satisfies all six foundations at the Proposed level.
- **Overall: Accept-scoped** for the target design as the **design direction**. It is scoped to a
  Proposed architecture that needs:
  - the ADR set A1–A10;
  - R1–R3 measurements;
  - per-packet implementation evidence.

  Acceptance does not schedule work, close a finding, or certify implementation.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| 1 | ADRs A1–A5 (scope, parity pin, property system, crates, template mechanisms) | F01–F03, F06, F07, F13 | Review re-run on the ADR set | Plan 20 P0.1 |
| 2 | `pse-modeling` extraction; template mechanisms; indexed law engine | F04, F06, F07; S02, S08, S11 | Packets P0.2–P0.4 tests | Plan 20 |
| 3 | `pse-properties` core with ideal and cubic forms; formulation primitives | F01–F03, F05; S01, S03, S07 | P0.5–P0.6 and W1 conformance | Plan 20 |
| 4 | Initialization knowledge, derived scaling, diagnostics | F10–F12; S09, S13 | W2 | Plan 20 |
| 5 | Discretization and dynamics as a mode | F08, F09; S05, S06 | W3 | Plan 20 |
| 6 | Native PC-SAFT, Helmholtz, nested flash, electrolytes; FeOS production removal | F13; S04 | W4 | Plan 20 |
| 7 | Costing, surrogates, studies, `models_extra`, applications | S12, S14 | W5 | Plan 20 |

**What would falsify the design's central claims:**

- identity expansion that cannot be prepared within bounded cost even through the nested route
  (R1);
- an IDAES unit whose behaviour needs imperative construction that no mechanism in modeling
  document §3.2 expresses;
- a property kind whose derivation requires a genuine fixed point rather than an acyclic
  closure.

**Coverage.**

- *Established by reading source and records:* the full IDAES target through its static
  records; the baseline seams cited above.
- *Not examined:* runtime behaviour of any proposed mechanism.
- **Artifact:** this file, with the target documents under `docs/plans/20-*.md`.
