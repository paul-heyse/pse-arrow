---
title: "Process-simulator target design review"
date: 2026-11-02
tier: design
purpose: target
focus: holistic
standard: "Core 3.4; efficiency heuristics 1.0; ProcessSimulator 1.5; test-assessment 1.2"
source_baseline: "03765c22e34db1d89f12f1a21a87e5b3e83a95ab"
audit_publication: "cf6822d10"
decision: revise
---

# Process-simulator target design review

The proposed direction is sound: develop a native, reusable process-science system by completing the existing authored scientific definitions, exposing coherent engineering operations over them, and qualifying complete cases. Retain that architecture. A second equipment graph, universal simulation AST, compulsory control-volume superclass or general autonomous planner would add substantial obligations without improving the selected engineering outcomes.

**Decision: Revise, with a bounded correction rather than a replacement architecture.** Two catalog contracts contradict otherwise sound guarantees: execution-plan identity is described too narrowly, and structural analysis products appear to require a solution member even when no state solution exists. The public process interface must also preserve existing initialization reports and producer-owned scientific permission across the current implementation’s diagnostic and result-selection boundaries. The refreshed audit identifies those implementation defects; their remedies must accompany the consumers that would expose them.

The distinction matters. The design already recognizes missing distributed accumulation, multiphase storage, property breadth and specialized workflows. Those are future scientific delivery obligations, not evidence that today’s narrowly admitted formulations are incorrect. Conversely, source inspection of existing models and numerical owners does not establish that the proposed complete journeys work.

## 1. Scope, criteria and evidence

This is an independent, holistic, design-tier, target-purpose assessment of `docs/process-simulator-design/`, covering the overall formulation contract, process-domain model, catalogs, task and answer contracts, scientific sourcing proposal, P0–P7 program and concrete A–D first delivery.

The functional target is reusable, physically meaningful process models serving edit/re-solve, simulation, rating/design, optimization, dynamics, estimation and studies, with useful failures and qualified results. IDAES 2.13.0 supplies an important behavioral and scientific comparison population. Its Python architecture is not the target architecture. Cross-industry cases stress the sufficiency of the contracts without committing all their implementations.

The selected standard is Core 3.4, efficiency heuristics 1.0, ProcessSimulator 1.5 and test-assessment 1.2 through the pse-arrow binding. Current repository mechanisms and policies describe the subject and its transition constraints; they do not limit the target judgment.

**Interface-checked baseline:** source `03765c22e34db1d89f12f1a21a87e5b3e83a95ab`; refreshed diagnostic publication `cf6822d10`, particularly [the scoped process-simulator account](../audit/process-simulator-target.md). Its preceding whole-system account inspected `19e7b49a67df24c1bf73005a6186e145e930d53a` and was published at `91f255e4d`. The scoped refresh establishes applicability of the named scientific/runtime observations; it does not refresh every historical finding or the separate test-audit population.

Reading covered the package’s principal narratives and delivery contracts, concept and task catalogs, representative examples, current state/port/inventory specialization, declared execution, source-edit, completion and connected-result owners. Scientific research examined relevant tagged IDAES behavior and candidate library contracts. No numerical or operational execution was undertaken.

Fifteen dirty library-leverage documents present at engagement start were preserved. They supply leads rather than independently established adoption recommendations.

Representative workload conditions are repeated numeric edits, structural edits, sparse coupled models, failed recycles, multi-experiment inference, growing case populations, longer trajectories and concurrent/interrupted work. This review establishes no capacity, latency, speedup or memory claim.

**Disposition owner:** subsequent authorized implementation planning must assign accepted findings to the existing owning plans. This review is evidence and proposed design; it does not schedule repairs, change active-plan status or authorize implementation.

## 2. Retain the core architecture and make its operations concrete

The useful decomposition follows reasons for change:

| Responsibility | Meaning and operation owned | Existing realization and boundary |
|---|---|---|
| Scientific definitions and data | Material identities, formulations, parameter conventions, states, reactions, equipment and supported configurations | Authored packages and admitted scientific records; no equipment-name dispatch in generic Rust |
| Source admission and revision | Interpret text or typed declarations, resolve dependencies, reject incompatible meaning, create immutable revisions | `pse-authoring`, `pse-admission`, `pse-modeling`, `ModelingPackage` and source-delta services |
| Process-facing inspection/editing | Present selected engineering structure; translate an authorized edit into the existing source/case operation | Derived declaration and semantic-instance views; no independently mutable process graph |
| Formulation and preparation | Specialize selected science, establish physical and structural obligations, derive numerical representations and mappings | Existing modeling/compiler/math owners |
| Numerical integration | Bind actual library capabilities, attempt state, callback contracts and termination evidence | Existing native backend/provider owners |
| Case and workflow composition | Select task intent, conditions, starts, policies and relationships between occurrences | Existing case, initialization, study, fitting and dynamic workflows |
| Scientific outcome and publication | Assess original obligations and product permission; retain and publish the resulting evidence | Existing completion, result, research/publication and canonical owners |
| Scientific discovery | Explain available definitions, missing dependencies and evidence for a selected use | Derived views over exact admitted revisions |

These are responsibilities, not instructions to create one trait, class, table, transaction or task per row.

### Proposed process-facing contract

The first interface should expose a small set of understandable operations over existing identities:

- Inspect a source draft, admitted declaration selection or selected instantiated model, stating which was inspected.
- Create, compose, connect and edit through source/declaration operations against an explicit parent revision.
- Bind or change an operating case without changing the reusable physical definition.
- Explain the selected task’s missing inputs, incompatible conventions, structural issues and unavailable numerical capabilities.
- Prepare the selected operation without executing it.
- Execute locally or through an explicitly selected recorded route.
- Inspect physical outputs, full failures, original assessments and product coverage.

A source-valid model, an inspectable underdetermined case, a numerically admissible square solve and a scientifically usable result are different states. Expose those distinctions through existing diagnostics and operation products. Do not add a global “ready” state machine or calculate readiness from a screen’s available values.

Choose the first editing representation deliberately. `ModelingPackage::with_declarations` already computes declaration-ID additions, replacements and removals, and stops treating old text as authority for the edited IR. Text editing and typed declaration editing can share admission without promising two synchronized authoritative representations.

Preserve the implemented topology route. `prepare_flow` performs semantic preparation independently of the selected numerical incidence. A property-panel request should use the smallest sufficient source or semantic view. It need not prepare an entire numerical problem.

### Proposed task admission

Solve, Extremize, Infer, Expect and Analyze should remain extensible public profiles mapped onto existing operations, not a replacement universal enum.

Admission consumes the selected model/configuration, effective case, requested products, enforcement choices and actual implementation capabilities. It produces attributable findings, a supported operation/preparation path, or an explicit refusal. Optional unavailable products remain unavailable; mandatory unsupported products refuse the request. Structural analysis itself must remain available for unsolved or ill-posed subjects.

Keep genuinely late inputs late: numeric values, placement and execution selection do not all belong in the compiler. However, late binding must consume prepared meanings rather than reconstructing conservation attribution, bound purpose or scale policy in each workflow.

## 3. Scientific authority and physical semantics

The package correctly distinguishes physical models, mathematical formulations, numerical approximations and task/evidence requirements. Preserve that distinction through all operations and discovery results.

| Element | Dimension/unit and basis | Convention or validity | Authority and consumed operation |
|---|---|---|---|
| Material boundary | Declared independent state coordinates plus component/energy transport observations | Species/phase membership, datum, supplied/receiving responsibility and incidence | Authored state contract; specialization and connection admission |
| Heat/work exchange | Physical transfer quantity and orientation | Paired contributions cancel only under compatible conventions; explicit interface storage/reaction changes closure | Authored exchange and contribution semantics |
| Inventory | Stored extensive quantity, with explicit capacity/phase/population relation | Original signed flux, event transfers and initial/periodic conditions | Authored inventory law and retained original-space checks |
| Thermodynamic property | Demanded output’s complete physical type | Exact formulation/data variant, phase/root convention, envelope, derivative availability | Selected scientific records and property realization |
| Reaction source | Declared extent/rate basis and participating subjects | Element/charge claims depend on admitted composition; formation-energy convention prevents double counting | Checked reaction projection or explicitly weaker empirical transformation |
| Empirical characterization | Declared assay, aggregate or measurement basis | Unknown formula or molecular weight stays unknown; mappings preserve only their stated quantities | Existing scientific-record and material-selection owners |
| Population/structure | Internal-coordinate measure, weighting and normalization | Number need not be conserved; moments need not reconstruct a distribution | Authored population/state/closure definitions |
| Observation | Quantity, location and sampling/aggregation interval | Noise, dependence, missingness, acquisition/process-time alignment | Observation mapping and inference/assessment contract |
| Engineering assessment | Physical contributor plus economic/environmental unit and period | Boundary, currency/base date, allocation, utilization and factor provenance | Authored assessment over selected physical results |

These are proposed contract obligations; their presence in documentation does not establish execution evidence.

### Well-posedness

Effective roles are explicit at scalar/indexed occurrence level. A supplied value is not automatically a fixed specification, and a guess is not a physical initial condition or branch selection.

Before a square solve, the owning preparation route must perform the applicable degree-of-freedom and structural analyses and attribute missing or conflicting conditions to model elements. Optimization, inference and explicitly underdetermined feasibility requests require their own structural contracts; positive freedom is not universally invalid. Matching and counts do not prove numerical rank, existence, uniqueness or physical validity.

A diagnosis-only Analyze task must accept the malformed or underdetermined structure it is intended to examine, without first requiring successful solve admission. This exposes the catalog correction in F06.

### Scientific assets and source-to-use closure

Retain the proposed native sourcing direction. The useful product is an exact, explainable scientific dependency closure for a consuming case:

> selected demand → existing native coverage → exact source and interpretation → admitted declaration/data revision → selected implementation → use-specific evidence.

A catalog item is not automatically executable, ordinary-input eligible or qualified. Preserve source occurrence, interpretation, implementation and evidence as distinct facts. Joint fits, directional interactions, formulation-bound constants and missing-versus-default parameter values must survive extraction.

Use SurrealDB for canonical content, exact selections and discovery; prepare selected native objects for repeated numerical evaluation. Do not issue a database lookup per residual or scalar property. A discovery query should identify its covered revisions and selection universe so that “not found” does not become “scientifically unavailable.”

Scientific publication already has source-delta preview and guarded evidence owners. Extend them for concrete missing formats or queries. A second accepted-knowledge application or qualification database is unnecessary.

## 4. Scenarios that distinguish the design

| ID | Scenario and classification | Required boundary and revealing evidence |
|---|---|---|
| S01 | Add an authored exchanger closure or compatible state implementation; scientific binding/composition | Reuse side balances and engineering interface. Preserve demanded transport, datum, validity and derivative contracts. A different EOS remains changed science. |
| S02 | Rename a label, change feed temperature, then change rating to design; presentation/value/structural edits | Preserve identity for labels; rebind admitted numeric demand; revisit layout for released/fixed variables. Incremental and clean preparation agree in admitted meaning. |
| S03 | Recycle initialization fails before final solve; policy/failure | Return original stage and attempt evidence; temporary overlays never become source facts. No candidate means unavailable assessments, not physical infeasibility. |
| S04 | Export a failed or qualified result, then request sensitivity/covariance; representation/product extension | Producer permission and local mathematical validity remain separate and are combined once. Callers cannot upgrade usability by labelling a selection. |
| S05 | Analyze an unsolved underdetermined model; task composition | Return structural products and diagnostics without inventing a root or state member. |
| S06 | Repeated cases, concurrent solves or interruption; workload growth | Reuse compatible preparation, isolate mutable attempt state, coordinate existing pools and retain charges until native work drains. |
| S07 | Dynamic inventory and distributed transfer; new physical composition | Supply accumulation and phase/transfer closure before removing current refusals; retain initial/boundary and integrated conservation meaning. |
| S08 | Joint experiments or posterior transfer; inference composition | Shared parameters remain shared; observation intervals and covariance/dependence survive alignment. Separate fitting evidence from prediction evidence. |
| S09 | Empirical material, population inventory and physical cycle; semantic stress | Do not invent exact chemistry, treat particle bins as species, or replace physical carryover with Study edges. Design examples suffice until an execution question requires a probe. |
| S10 | External detailed unit or implementation replacement; mechanism substitution | Distinguish residual exposure, input/output evaluation and time advance. Declare derivative, branch, state/effect and coupling limitations. |

S09 does not add every industry family to P1. S10 does not make FMI or a universal co-simulation coordinator a prerequisite.

## 5. Numerical stages and execution fit

| Stage | Formulation and derivatives | Scaling/class and capability | Outcome, tolerances and assessment | Physical realization |
|---|---|---|---|---|
| Admission/specialization | Capture original domains and physical obligations before rewriting; resolve selected science | Roles, contribution meanings and model scales have authoritative definitions | Attributable invalid/unsupported/unresolved findings | Select sufficient closure; reuse established immutable validity |
| Structural preparation | Preserve topology/incidence distinction and origin mappings | Analyze the effective task; do not infer class from the profile name | Counts/matching/decomposition with explicit limits | Reuse structure across compatible value changes |
| Property/provider evaluation | State actual values and derivative order, branch and smoothness; no silent derivative fallback | Demand only consumed products; validate actual provider context | Typed evaluation/domain failures; selected extrapolation stays qualified | Attempt-owned mutable workers; batch native operations; no hot-path store calls |
| Initialization/recycle | Explicit overlays, staged simplifications, starts and branch/path policy | Compatible native root/continuation machinery | Full stage history and failure; restored-original acceptance | Reuse preparation without sharing mutable attempt state |
| Root/optimization | Original guards survive transformations; exactness follows the complete derivative chain | Derived class, sparse structure and fitting native solver | Native termination distinct from original residual, bounds, domains and closure | Native iteration/globalization/factors; compact mappings and workspaces |
| Dynamics/discretization | Inventory, initial conditions, events and boundary physics precede numerical approximation | Supported ODE/DAE/index/event/derivative contract | Coverage, event-side values, integrated closure and error evidence | Retain only necessary trajectories; reuse compatible preparation |
| Derived analysis | Original point/branch/active-set and mathematical regularity | Actual sensitivity, curvature, information or statistical product | Local validity combined with governing permission; unavailable products remain explicit | Compute requested products without repeating unrelated preparation |
| Publication/readback | Supported reconstruction only; no invented inverse or physical interpretation | Existing canonical/document schemas | Coherent coverage and producer-owned permission | Separate long computation from short publication; hydrate detail on demand |

AP-07 does not require a new cost model, scheduler or performance campaign. Necessary coupled solves and global structural work are legitimate. Incidental rebuilding per case, per-scalar crossings, all-case retention and independently maximized pools need a concrete benefit.

The package already specifies the right preparation/hot-evaluation separation. The main contract correction is to prevent semantic-question identity from being mistaken for preparation identity.

## 6. Findings and proposed corrections

### <a id="f01"></a>F01 — Preserve initialization evidence across the public operation boundary

**High consequence for S03. Diagnosis: Interface-checked. Remedy: Proposed.** Inherited cause: [DA-039](../audit/structure-and-invariants.md#da-039). AP-02, AP-05, DP-02, DP-21, PS-08 and PS-10.

`execute_declared` receives a full initialization report, then converts an incomplete report into debug text through `contract`, yielding `WorkflowError::Input` and losing staged report structure. `initialize_declared` already returns the full report. The first-delivery brief correctly selects that route.

The process interface must consume a defined initialization outcome contract: actual completion/failure, stage and attempt evidence, candidate availability and original assessments. A failed numerical initialization must not become invalid input merely because a convenience wrapper was used.

Use the full-report owner for P1. If a common execution operation is exposed, its typed result/error must preserve that report or a stable owned reference to it. Do not build another rollback or initialization implementation.

**Correction discriminator:** the intentional failure reaches its intended stage and remains machine-classifiable with stage/attempt evidence through local and selected recorded readback. A setup failure cannot satisfy it. Existing restoration and original scientific checks remain independently exercised.

### <a id="f02"></a>F02 — Result export must derive scientific usability from its producer

**High consequence for S04. Diagnosis: Interface-checked. Remedy: Proposed.** Inherited cause: [DA-049](../audit/semantic-model-and-authority.md#da-049). AP-02, AP-04, DP-01, DP-03 and PS-12.

`PackScientificState` promises producer-recorded usability, but `pack_selection` accepts the state from its caller. Validation reconstructs using the supplied state without deriving the producer permission it claims.

Make the selected result owner resolve usability from the exact completion/permission scope. A caller may select requested content or impose a stricter eligibility filter; it cannot assert a stronger scientific state. Preserve diagnostic-only access without presenting it as usable science. Absence of permission remains unknown/unavailable, never success.

This is an enforcement correction at an exported meaning boundary. No incorrect published scientific result was reproduced.

**Correction discriminator:** changing only a caller-supplied label cannot upgrade an unusable or unknown result; exact coordinates, source revision, qualifiers and coverage still reconcile.

### <a id="f03"></a>F03 — Derived-product permission needs one composition owner across product families

**Material consequence for S04/S08. Diagnosis: Interface-checked. Remedy: Proposed.** Inherited cause: [DA-048](../audit/semantic-model-and-authority.md#da-048). AP-04, DP-01, DP-05 and PS-12.

Completion combines permission for some fit-derived products, while non-fit analysis consumers separately interpret original permission and mathematical validity. Neither fact is redundant: a local derivative can be mathematically valid at a scientifically unusable point.

Extend the existing completion/product-permission responsibility to compose these facts for actually exposed product families. Preserve their separate evidence and qualifiers. Lazy products gain their combined permission when produced; an earlier immutable invocation may truthfully report them unavailable.

This does not call for a new permission service or repeating physical solves for every reader.

**Correction discriminator:** a new derivative/statistical consumer reads a completed decision and its governing evidence without reimplementing how original permission combines with local validity. Diagnostic visibility survives withheld use permission.

### <a id="f04"></a>F04 — New process workflows must consume formulation obligations rather than reconstruct them

**Architectural consequence for S01/S07/S08. Diagnosis: Interface-checked. Remedy: Proposed.** Inherited cause: [DA-006](../audit/semantic-model-and-authority.md#da-006). AP-01, AP-03, AP-04, PS-03 and PS-07.

Runtime case, sequence and dynamic preparation compose formulation meanings already established by scientific/modeling/compiler owners. Adding another process facade without settling the consumed contract would add another interpretation point.

Have the scientific/modeling/preparation owner expose the selected obligation meaning and source attribution needed by the consuming operation. Runtime binds actual late values and numerical/execution choices. Keep interfaces proportional to concrete consumers; do not move every runtime decision into the compiler or serialize the entire specialized model.

[DA-003](../audit/semantic-model-and-authority.md#da-003) is a related change-coupling consideration for source selection, not evidence that conservative discovery must become exact checking. Shared traversal and deliberate superset selection are preservation constraints.

**Correction discriminator:** changing a closure convention or physical allowance changes its authority and necessary binding, rather than independent interpretations in each steady, dynamic and process-facing workflow.

### <a id="f05"></a>F05 — Execution-plan catalog identity conflicts with the stated reuse contract

**Bounded document/API ambiguity for S02/S06. Diagnosis: Interface-checked. Remedy: Proposed.** AP-02, AP-05, AP-07, DP-09 and G6.

`catalog/simulation/execution.md` describes an Execution plan as “Root, keyed to a problem’s semantic hash.” Elsewhere, MethodRequest/Header and the execution narrative correctly require consumed method/preparation dependencies and permit structural reuse across changed case values.

Read literally as a complete key, the catalog could reuse a plan after a required method change or rebuild structure for every value-only case. The stronger surrounding contracts are contrary evidence; this review does not claim an implemented cache defect.

Change that row to distinguish association with a semantic request from identity of a prepared plan. Derive the latter from its complete consumed structural, method, evaluator/version and capability dependencies. Keep submission, attempt and executed-result identities separate. Do not introduce a second cache mechanism.

**Correction discriminator:** one semantic question with incompatible required numerical settings does not collide, while a compatible numeric edit can retain prepared structure. Existing complete-dependency ownership supplies the mechanism.

### <a id="f06"></a>F06 — Analysis products must not require a state-solution member

**Bounded semantic contradiction for S05. Diagnosis: Interface-checked. Remedy: Proposed.** AP-02, AP-04, DP-02 and G2.

`catalog/simulation/solution.md` places Analysis result under Solution member, whose definition is a candidate/qualified branch. The Analyze profile expressly permits successful structural analysis of an unsolved or singular model with zero state-solution members.

Allow task-level analysis products under the retained outcome, with optional association to a qualified member/reference point where required. A failed solve may also carry useful structural findings. Do not invent a numeric member or reinterpret numerical failure as successful simulation to carry those products.

**Correction discriminator:** an underdetermined unsolved model returns counts/matching findings with no fabricated root, while point-based sensitivity still identifies its actual candidate and assumptions.

### Existing acceptance and lifecycle constraints

Two earlier findings matter to the complete target but are not newly refreshed diagnoses here:

- [DA-062](../audit/execution-fit.md#da-062): ordinary interval acceptance differs between numerical routes. A correction should share the original tolerance-aware interval meaning while preserving strict domain/sign guards and fixed specifications. Do not relax every bound.
- [DA-063](../audit/execution-fit.md#da-063): pending capture can obstruct unrelated renewal/cancellation. Preserve selector completeness and recovery fencing while separating unrelated liveness operations. Do not replace the gate with a snapshot of only currently matching rows.

Their existing owners retain disposition. Broader P6/P7 claims must consume applicable resolution/qualification rather than silently assuming these concerns disappeared.

## 7. Architectural assessment and gates

These judgments concern the package as an implementation-design baseline. A Proposed remedy does not make the current implementation pass.

| Foundation | Package judgment | Reason |
|---|---|---|
| AP-01 | Satisfied at Proposed design level | Scientific, admission, numerical, workflow and publication responsibilities are coherent; F04 constrains realization |
| AP-02 | Unresolved | F05/F06 leave contradictory consumed contracts; F01/F02 identify current boundary defects |
| AP-03 | Satisfied at Proposed design level | Cases and analyses compose existing capabilities; no universal builder or compulsory control-volume ancestry |
| AP-04 | Unresolved | Main semantic model is adequate; F06 and current F02/F03 handoffs prevent an unqualified authority judgment |
| AP-05 | Unresolved | Most constraints are explicit; plan identity and outcome ownership need the bounded corrections |
| AP-06 | Satisfied at Proposed design level | Admission/policy can use actual bounded inputs; independent science and integration evidence retain distinct roles |
| AP-07 | Unresolved | Credible native preparation/evaluation route is specified, but F05 must resolve reuse meaning; assembled benefit remains unmeasured |

| Gate | Target baseline | Current realization/evidence limit |
|---|---|---|
| G1 Authority | Pass at Proposed design level | One authored scientific authority; F02/F03 remain implementation ownership obligations |
| G2 Semantic fidelity | Unresolved | F06; DA-039 also demonstrates current diagnostic loss |
| G3 Validity | Pass at Proposed contract level | Required rejection and assessment boundaries are specified; F02 is a current enforcement gap |
| G4 Hidden behavior | Pass at Proposed contract level | Local/recorded effects and provider/time-advance distinctions are explicit |
| G5 Consistency/recovery | Pass at Proposed contract level | Immutable inputs, overlays and publication owners are retained; no new execution/crash qualification |
| G6 Transformation/reuse | Unresolved | F05; reconstruction and approximation limits are otherwise sound |
| G7 Truthful capability claims | Pass with stated scope | Adopted documentation is not implementation evidence; catalogs and deferred examples explicitly limit claims |
| G8 Library leverage | Pass at Proposed design level | Native/library ownership is retained; no generic solver replacement is recommended |
| G9 Architectural fitness | Unresolved | Follows individual foundation judgments without averaging |
| PS-G1 Physical consistency | Contract specified; implementation qualification not established here | Physical conventions and independent checks are explicit; no numerical execution |
| PS-G2 Well-posedness | Unresolved at outcome contract boundary | F06 must preserve diagnosis without requiring successful Solve; no universal solvability proof is requested |
| PS-G3 Numerical integrity | Contract specified; current integration needs revision | F01–F03 and applicable DA-062 obligations; no new solver/derivative qualification |

Overall **Revise** follows the material unresolved contract boundaries and current integration defects. The scientific direction is retained.

## 8. Library fit and alternatives

The supporting scientific research is **Interface-checked**, not adoption or product qualification.

| Need | Mechanism and design role | Limits and ownership consequence |
|---|---|---|
| Discover demanded properties | IDAES `PropertyInterrogatorBlock` / `ReactionInterrogatorBlock`, `list_required_properties`, `list_models_requiring_property` demonstrate explainable construction-time demand discovery | Interrogator dummy values cannot solve or prove complete optional configuration coverage. Native specialization should explain its real demanded closure; do not add a parallel stub model architecture. [IDAES documentation](https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/property_models/interrogator.html) |
| Reusable balances and state behavior | IDAES state `get_material_flow_terms`, density, enthalpy and energy terms demonstrate the consumed contract; MSContactor demonstrates composition without ControlVolume | Reuse behavioral separation, not mutable Python builders. Rate basis and missing closure remain authored science. [MSContactor](https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/unit_models/mscontactor.html) |
| Native external properties | CoolProp `AbstractState`, `update`, keyed outputs and supported first/second partial derivatives | Reuse one attempt-owned state for multiple products; freeze backend/data/datum. General partial derivatives have homogeneous-phase limits; separate routines do not imply arbitrary flash Hessians. [Low-level API](https://coolprop.org/coolprop/LowLevelAPI.html) |
| Multifluid Helmholtz capabilities | teqp `build_multifluid_model`, `get_Arxy`/related derivative products and `pure_VLE_T` | Residual Helmholtz capabilities require compatible ideal/caloric contributions for complete enthalpy/entropy use. Ancillary densities are starts, not equilibrium evidence. [Multifluid](https://teqp.readthedocs.io/en/latest/models/multifluid.html), [derivatives](https://teqp.readthedocs.io/en/latest/derivs/derivs.html) |
| Residual DAE integration | IDA `IDAInit`, `IDASetId`, `IDACalcIC`, `IDASolve` are candidates when their exact residual/index contract fits | Consistent-initialization numerics do not provide arbitrary high-index reduction or hybrid sensitivity semantics. Exact linked version/ABI remains a delivery decision. [SUNDIALS 7.1.1](https://sundials.readthedocs.io/en/v7.1.1/ida/Usage/index.html) |
| Discrete/nonlinear applications | HiGHS for genuinely linear/MILP formulations; SCIP nonlinear expression constraints for fitting factorable MINLP | Preserve structure needed by nonlinear handlers and bounds. An opaque value/gradient provider does not establish global capability. [SCIP 10.0.2 nonlinear constraints](https://scipopt.org/doc-10.0.2/html/cons__nonlinear_8h.php) |

Context7 did not yield a relevant IDAES corpus; tagged 2.13.0 primary source and official documentation supplied consequential evidence. Missing search coverage is not capability absence.

Compare complete options:

1. **Retain authored native science, extend its existing integration owners — recommended default.** Best continuity with physical types, derivatives, source identity and reuse. New scientific content and qualification remain substantial work.
2. **Use a native external implementation for a selected demanded contract — valid alternative.** Can remove substantial EOS, property or numerical machinery; adapter state, datum, derivatives, branch behavior and build/data identity become explicit ownership costs.
3. **Mirror a reference simulator’s object hierarchy — reject.** It duplicates the current scientific model lifecycle without guaranteeing native semantic or numerical equivalence.
4. **Build a universal ontology/planner before useful process delivery — reject.** The catalog’s breadth does not justify implementing every conceptual noun or mathematical form.
5. **Use complete fixed-input collection rather than elaborate streaming — valid for small cases.** Larger studies/trajectories must justify their live-state route; streaming is not mandatory merely because it is available.

## 9. P0–P7 handoff

These are design groupings for later planning, not newly scheduled work.

- **P0:** freeze the finite scientific/configuration/reference population and distinguish matched equivalence from improved science and beyond-release extensions. Repair the scoped authority conflicts listed below. Do not use notebook counts as scientific coverage.
- **A:** settle ordinary inputs and exact scientific dependency closure for the recycle and exchanger. Retain oracle/fixture eligibility separately. Add the first bounded import/discovery path only where it supplies an actual missing input.
- **B:** deliver one process inspection/edit path over existing identities and revisions. Resolve F05/F06 before their catalog rows guide public contracts. Keep source, instantiated semantic and numerical scopes distinct.
- **C:** integrate full native journeys with F01/F04 obligations. Preserve initialization reports, original checks, explicit local/recorded effects and typed unavailable products.
- **D:** matched independent references and complete mapped outputs; preserve analytic/hand-derived controls. Include F02 in any selected exported-result/readback path. F03 follows each exposed derived product rather than becoming an excuse to implement all analysis families.
- **P2/P3:** grow scientific assets and reusable equipment through consumer closures. Derivative/envelope and configuration evidence accompany the selected units. No whole-family readiness framework is needed.
- **P4/P5:** add actual storage, phase allocation, distributed accumulation, transfer and operating-cycle physics. Current refusals are removed only with those contracts and evidence. Select external/native integrations by complete consumed capability.
- **P6:** compose inference, uncertainty, control, discrete and assessment operations with shared parameter and physical-time meaning. Carry producer permission and applicable acceptance/liveness corrections through the exposed products.
- **P7:** assembled qualification and release claims close the actual finite denominator. Required missing prerequisites fail selected runs; unrun scope remains unattempted. No source-level review substitutes for this evidence.

Priority follows consequence: misleading scientific usability and lost failure meaning are more urgent than catalog wording. Prerequisite order differs: the small catalog corrections should precede freezing affected APIs.

## 10. Verification and conformance status

**Not run:** all builds, tests, solver runs, parity invocations, campaigns, benchmarks and runtime preparation. This review has no new failure count or passing product baseline.

Source-inspected recycle and exchanger campaigns contain real authored cases and independent checks. Existing process-contract tests include species reordering, indexed/parallel connections and receiving reconstruction. Existing initialization evidence includes restoration and wrong-class/wrong-member controls. Those are preservation constraints; reading their bodies does not create new passing receipts.

No touched unit/property model gains new conformance status from this review. Broader steam, electrolyte, transport, multiphase inventory, distributed dynamics and specialized-family qualification remains with its selected delivery owners.

Useful acceptance evidence is already largely specified by A–D: ordinary-input eligibility; identity/order and incremental/clean correspondence; intended failure attribution; original residual/domain/bounds and independent conservation checks; exact retained readback; complete finite mapped references and meaningful wrong/missing-output controls.

Performance questions should distinguish cold preparation, value-only reuse, structural edits, case populations and trajectories. Measure only when making the corresponding benefit or capacity claim.

## 11. Rule impacts and consequential unknowns

| ID | Existing rule/location | Proposed change and dependent recommendation | If retained |
|---|---|---|---|
| RC01 | Scope exclusions and retired reference/product descriptions in `docs/relationship-to-idaes.md`; affected architecture scope owners | Adopt a finite expanded released process-modeling target, with explicit supplemental non-process obligations and exact independent reference revisions | P0–P7 cannot claim the expanded equivalence program under the existing exclusions |
| RC02 | Python authoring/result-boundary wording in the review binding | Reconcile the touched description with the current native product and isolated scientific-reference boundary | Treat it as stale descriptive context; do not restore a Python product boundary to satisfy it |
| RC03 | Execution-plan multiplicity/identity row in `catalog/simulation/execution.md` | Replace semantic-hash-only wording with F05’s association versus complete preparation-dependency distinction | The affected public preparation identity remains unresolved |
| RC04 | Analysis-result ownership in `catalog/simulation/solution.md` | Permit task-level analysis products with optional member/reference-point association, as F06 requires | Diagnosis-only Analyze outcomes lack a coherent stated home |

Bug fixes restoring declared diagnostic and producer-permission contracts require no new policy exception. The Rust/SurrealDB sourcing direction is already recorded in the package; this review does not propose changing it.

Consequential unknowns are bounded:

- **Ordinary scientific inputs:** which exact records can support A without fixture/oracle permissions? Settle through the selected source/record closure.
- **Reference compatibility:** which pinned example configurations execute faithfully against IDAES 2.13.0? Settle per mandatory case, not by package metadata.
- **Derived-product scope:** which non-fit products does the first interface actually expose? That determines the immediate F03 surface.
- **Dynamic breadth:** does a selected case fit current layout/index/event contracts? If not, specify the physical and numerical gap before selecting integration machinery.
- **Empirical/population sufficiency:** can existing authored quantities, axes and relations express the selected stress case without false conservation? A concrete counterexample, not an industry label, triggers a generic extension.
- **External substitution:** are demanded values, derivatives, branches, datums and effects supplied together by the exact candidate? A library name or installed binary cannot settle this.
- **Assembled execution:** source establishes credible routes and specific amplification concerns, not the achieved latency, robustness or capacity.

## 12. Decision

Retain the proposed native authored-science architecture, compositional equipment model, explicit task profiles, targeted scientific sourcing and case-driven P0–P7 sequence.

Revise the two contradictory catalog contracts and carry the diagnostic, formulation and producer-permission corrections into the public operations that consume them. Do not expand this into a universal modeling framework or unrelated numerical rewrite.

The resulting target remains **Proposed** until adopted through the existing planning/decision routes. Existing implementation, required qualification and measured benefit retain separate status.
