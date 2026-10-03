---
title: Flowsheet compilation and capability-driven solver execution
date: 2026-10-02
tier: design
purpose: target
standard: core-3.3
profile: process-simulator-1.3
baseline: ec45d4c8640f1dcbc04e0eb8a552e146f4d3ac39
evidence: Implemented
decision: Revise
---

# Flowsheet compilation and capability-driven solver execution

**Revise the current compilation/routing target.** The user's hypothesis is supported: full problem compilation should compose owned domain operations, and backend execution should follow contextual capability contracts. The implementation already has much of that scientific model and adapter structure. The principal gaps concern when those meanings govern preparation, what contextual eligibility promises, and how failed execution is interpreted. A larger inventory of unconditional solver flags would not repair these interactions.

The recommended target is **Proposed**: retain the checked process model, separate semantic selection from executable preparation, assess candidate requirements before choosing artifact demand, and qualify every result in original physical terms. This is a correction of existing ownership boundaries, not a recommendation to replace authoring with another language or implement numerical algorithms in the simulator.

Architectural fitness **fails G9**. Behavioral/semantic adequacy also requires revision: missing feasibility evidence can be presented as `infeasible`, and resource exhaustion can be classified as trial rejection. These paths still refuse candidate use; the finding is misleading interpretation, not a falsely accepted solution. Complete PR/PFR scientific adequacy and integrated qualification remain unresolved.

## Review boundary, baseline and evidence

The review applies [Core 3.3](../design_principles/core/design-principles.md), its [template](../design_principles/core/design-review-template.md), [ProcessSimulator 1.3](../design_principles/profiles/process-simulator/principles.md) and the [repository binding](../design_principles/binding/pse-arrow.md). It is a subsystem **design-tier, target-purpose** review, including adjacent consumers needed to assess full case compilation and execution.

The inspected boundary is authored unit/property/connection and indexed/conditional definitions → specialization and case/formulation binding → mathematical facts and artifact preparation → solver selection/admission → attempt and original-space qualification. Square roots, optimization, dynamics, fitting, initialization and studies are considered where their requests change that boundary. All nine current adapter contracts were inspected; this is not exhaustive numerical qualification of every backend or the whole durable store.

Investigation began on `a7e9a4e4` with the preserved Plan 25 working changes. The maintainer committed that tree during review as the baseline above; it was clean before these review-document edits. Production files were not edited by the reviewers. Accepted ADRs and architecture sections were not changed. Proposed ADRs retain their existing status.

The coordinator owns this principal document and receipt verification. A fresh independent design reviewer assessed compilation and the combined boundary, including a follow-up on failure interpretation; a library researcher independently assessed native capabilities using pinned sources and Context7 primary documentation. The coordinator inspected decisive source paths and reconciled those assessments. No product tests, native campaigns or performance measurements were run for this review: static evidence settles the reported stage and interpretation defects. Prior execution receipts are identified below with their narrower conditions.

Current adopted finding status belongs to the [Plan 25 coordinator](../../plans/25-design-remediation.md#finding-dispositions); [25k](../../plans/25k-integrated-qualification-and-closure.md) owns qualification readiness. This review recommends work owners; it does not claim its recommendations are scheduled, implemented or accepted through an ADR.

### Functional drivers and revealing changes

| Scenario | Stimulus and intended observation | Evidence and limit |
|---|---|---|
| <a id="s01"></a>S01 — Add a unit/property model | Existing physical concepts, indexed ports and contributions should extend their semantic owners; topology and a value observation should not require an unrelated Hessian | Source inspection of specialization, topology, projection and body admission; actual extension performance unmeasured |
| <a id="s02"></a>S02 — Substitute a consumed solver | A bounded square problem must expose each candidate's actual representation and preparation requirements, without changing scientific bounds or inventing a dummy objective | CSTR/PR routing receipts and adapter source; substitution is conditional on supported guarantees |
| <a id="s03"></a>S03 — Prepare a large case selectively | Repeated PFR instances and a PR flowsheet need complete dependencies and original checks, with preparation following requested outputs and derivative order | PR/PFR preparation receipts plus support-construction source; no complete multimesh scaling measurement |
| <a id="s04"></a>S04 — Compose an analysis | Roots, fitting, dynamics and studies reuse model semantics while retaining distinct sensitivity, event, initialization and outcome requirements | Declared analysis, fitting and dynamic admission source; no new enclosing scientific campaign |
| <a id="s05"></a>S05 — Exercise policy locally | Route planning and completion interpretation should accept typed model/capability/evidence inputs without starting unrelated native or storage services | Existing pure routing and completion seams; new demand separation remains Proposed |
| <a id="s06"></a>S06 — Fail or stop execution | No candidate, unavailable quality, exhausted resources and evaluated infeasibility remain distinguishable; feasible incumbents keep their declared use policy | Source plus PR resource-failure receipt; preservation tests specified below |

S01/S02/S04/S05 refine the binding's PSE-S01/PSE-S02/PSE-S03/PSE-S05 seeds. They include a domain extension, a mechanism substitution and a local test scenario without treating incompatible solvers as interchangeable.

## Existing model, responsibilities and preservation constraints

The system already models more than equations. `pse-modeling::SpecializedModel` retains instantiated ownership, state specifications, material ports, connection occurrences, physical inventories, formulations, implicit selections and provenance. `pse-quantity` owns complete physical contracts and operation admission. The compiler derives mathematical bodies and case structures; libraries own arithmetic, sparse assembly, structural algorithms and native iteration.

| Responsibility | Owned meaning and consumed contract | State/effect and test boundary |
|---|---|---|
| Authored/checking/specialization owners | Definition and instance binding; physical/indexed context; contribution, connection and formulation meaning | Immutable checked revisions and specialized inventory; semantic admission can be tested with actual physical declarations |
| Compiler/math preparation | Body-local mathematics, instance attribution, selected case roles, structural facts, representations and artifact demands | Salsa owns semantic derivation; runtime owns effectful artifact construction and retention; values rebind through existing views |
| Structural/flow owners | Directed physical topology, equation incidence, conditional locality and explicit solve/tear decisions | Distinct immutable projections; petgraph and established numerical/structural libraries supply algorithms |
| Native adapters and policy | Class, representation, settings, method/build requirements, native sessions and status interpretation | Each adapter hides library details; contextual admission precedes execution; attempt state remains worker-owned |
| Workflow qualification | Candidate permission, physical checks, endpoint/closure/applicability evidence and analysis validity | Shared completion policy consumes typed evidence; persistence/transport project its meaning |

These distinctions are constraints on the correction:

- `FlowGraph` retains isolated units, directed and parallel connection occurrences, declared tear groups, physical binding conversions and actual quantity types. `PreparedModeling::flow_graph` checks selected membership and rejects cut declared connections. A stream graph must not become equation incidence or solve order (PS-05).
- `CaseIncidence` and `StructuralAnalysis` retain the complete selected rows and free columns, contribution provenance, matching, DM partitions and BTF. Structural rank is not a claim about numerical rank. Conditional-unit preparation checks actual ownership and external coupling; neither BTF nor a tear set alone licenses independent unit execution.
- `ImplicitMeaning`, `ImplicitSelection` and `ImplicitAlgorithm` distinguish a relation, unique graph, restricted branch, operational selection and minimum-score meaning from the mechanism executing it. The original operational density selector and explicitly authored relational density formulation remain different contracts. A successful solver is not proof of uniqueness or branch equivalence.
- Body-local checked-member tokens and total `InstanceBinding::checked_members` now separate shared mathematics from each instance's attribution. The repaired identity/projection lead is a preservation constraint, not another open finding. Old results must retain their original attribution.
- Existing case/observation/parametric caching, value rebinding, allocation ownership and separate mutable workers are useful seams. Staging must not create another cache authority or move escaped allocations outside accounting.

### Physical semantics and well-posedness

This table records contract distinctions rather than duplicating authored numerical quantities. Exact units, bounds, nominal magnitudes and property data remain in their current declarations and physical inventory.

| Model element | Dimension/unit and basis | Reference/convention | Validity | Authority |
|---|---|---|---|---|
| Material stream/port | Declared mass/molar flow and indexed species/phase contract; conversion only at a checked boundary | Declared direction; explicit connection occurrence | Complete composition and selected applicability evidence | Physical semantics §8–§9; specialized material ports and `FlowGraph::admit` |
| Temperature/pressure | Declared canonical quantity unit; point differs from difference | Temperature datum; absolute differs from gauge pressure | Original property envelopes and domain obligations survive lowering | `QuantityType`, physical operation admission; blueprint §8 |
| Energy contributions/inventory | Declared extensive/intensive basis and accumulation dimensions | Reference state; heat/work and transfer signs | Independent boundary/temporal closure, including event transfers | Authored contributions and closure declarations; blueprint §12 |
| Indexed/temporal state | Ordered physical axes and realized membership | Initial condition versus fixed specification versus guess | Finite realization and required dynamic initial conditions | Checked occurrence/index and temporal analysis owners |
| Density/property selection | Declared property quantity and phase bounds | Relation versus operational selected function | Envelope, branch and derivative evidence are distinct | Authored property definitions and admitted implicit meaning; ADR-0144 remains proposed |

The well-posedness mechanism is **Implemented** in the inspected slice: case variable roles are explicit, original selected rows/free columns survive preparation, and `structural::admit` consumes mode-qualified structural witnesses before native solving. Conditional initialization additionally checks local boundaries and external coupling. Complete well-posedness across every selected route remains unqualified; a successful scalar/root control cannot establish full flowsheet closure or dynamics consistency.

## Findings and their relationships

| Finding | Consequence | Priority and proposed owner |
|---|---|---|
| [F01](#f01) — Consumer demand arrives after executable/support admission | Topology, selected observations and Value/First requests inherit unrequested executable/Second work | High architectural/operational consequence; compiler/modeling/math preparation owners |
| [F02](#f02) — Preparation and contextual route eligibility do not compose | A backend assumption restricts artifacts before model-dependent selection; generic candidates can later fail contextual admission | High capability/architecture consequence; routing/adapter and analysis preparation owners jointly |
| [F03](#f03) — Missing assessment and resource failure are misinterpreted | Safe refusal carries misleading scientific and diagnostic reasons | High semantic fidelity consequence; shared completion and diagnostic projection owners |

F01 and F02 meet at the preparation contract but have different closure obligations. Selecting a better backend does not remove eager body work; separating support products does not choose a suitable solver. F03 is independent of numerical preparation, although accurate failures make the other work diagnosable. Priority follows consequences; prerequisite order is discussed in the decision.

### <a id="f01"></a>F01 — Consumer demand arrives after executable/support admission

**Implemented source path.** `ModelingPackage::prepare_flow` calls complete `prepare` before topology projection. `MathService::prepare_modeling_revision` invokes `CompilerWorkspace::prepare_modeling_cancellable`, which includes projection and executable admission. In compiler `workspace/modeling/executable.rs`, `projection` adds observable identity outputs for runtime inputs and retained expression members, alongside equations and mandatory observations. `executable/grouped.rs::admit` admits these expressions before `bound_structure` or `observation_structure` selects the consumer's executable view.

`pse-math::typed::BodyBuilder::prepare` calls `execution::PreparedBody::new_with_allowance`. Its `analyze` pass constructs both first and second support before a requested derivative order is supplied; it invokes library differentiation for retained expressions and composes Hessian pairs. `dense_second` materializes `n(n+1)/2` pairs. This work consumes the body occurrence allowance. Requested order arrives later in `PreparedModeling::prepare_view` and `CasePlan::prepare`.

**Consequence.** A physically checked topology or lawful value/first-order observation can be refused because of arithmetic output or second-order support work it never consumes. Repeated cases can consequently spend resources before their relevant representation is known. The PR/PFR receipts establish real support refusals, but they do not isolate what proportion came from this cause; unavoidable requested work or an insufficient explicit allowance may also remain after correction.

**Principles:** AP-01/AP-03/AP-04/AP-05/AP-06, DP-08/DP-10/DP-20, PS-07; S01/S03/S05. The violated model is the *compilation operation's request and dependency contract*, not a claim that all physical types or authored models are inadequate.

**Correction, Proposed.** Keep semantic/physical admission and finite specialization as a product usable by topology. Select executable outputs and their mandatory dependency closure before body/program preparation. Separate value/control/domain/effect facts from derivative support and artifact products, keyed by requested outputs, coordinate sets and order. Reuse existing compiler/math owners and library algorithms.

Three obligations must be demonstrated independently: topology must not need executable arithmetic; unrelated observations must not be prepared merely because they are expressible; Value/First requests must not construct unrequested Second support. Larger allowances or shared fact snapshots alone do not establish these boundaries.

**Legitimate countercase.** A zero derivative does not erase a domain guard, property envelope, applicability requirement, physical check or effect dependency. Value admission must still reject a physical/type error and retain the original obligations of selected outputs and required closure/checks. A lawful exact-Hessian request may still exhaust its finite budget; that failure must not invalidate a previously admitted Value product or be memoized as permanent mathematical incapability.

**Verification.** Exercise topology, value, first and second preparation separately on the same definition. Include a value/first request with second-support work exceeding its allowance, an unrelated observation that should not be built, and an invalid physical/domain declaration that must still fail at the proper boundary. Compare clean and incremental selected products; retain complete rows, coordinates, original checks, identity and diagnostic attribution. Establish large-case cost only through later named measurements.

### <a id="f02"></a>F02 — Preparation and contextual route eligibility do not compose

**Implemented source path.** `SolverProfile::derivative_order` in runtime `math/solves.rs` assumes KINSOL for automatic Root/Initialize and Ipopt for other automatic algebraic requests. `workflow/modeling/declared.rs` assigns that order to the analysis. `resolve_case` prepares inner registrations at the maximum of caller order and this speculative demand, and the bound case at caller order. Final `routing::admit` tests the minimum of `ProblemFacts.derivatives` and `prepared_derivatives` against each adapter's demand.

The facts already distinguish scientific availability from prepared artifacts; preserve that distinction. The missing operation is contextual candidate assessment that can request an available artifact before concluding that the candidate is unusable. For a boxed root, KINSOL is excluded by bounds while exact Ipopt/POUNCE require Second. The PR automatic receipt selected SCIP with Ipopt/POUNCE marked `derivatives`, demonstrating that initial preparation influenced the available route set. It does not prove that a Second artifact will fit the budget or that another native solve will converge.

Contextual representation/method checks are also later than generic selection. `MathService::prepare_resolved` chooses a route before `factorable_program` and `execution::admit_program`. Ipopt settings admit the selected linear solver's actual linkage, threads and process prerequisites. Cone subtypes/build profiles and trajectory methods have further typed checks. Dynamic `Profile::resolved_method` chooses Auto from recoverable-trial policy alone, then `validate` checks directional events, sensitivities and method-specific algebraic support. An Auto Diffsol request with a directional event can therefore be rejected even when a linked IDAS route could meet that request.

These checks are **safe preflight boundaries**, not evidence that unsupported inputs reach native execution. Nevertheless, the currently consumed generic eligibility contract does not fully support preparation planning or automatic selection of a contextually suitable candidate. Published class capability is not a promise that every instance of that class is representable by the selected method/build.

**Principles:** AP-01/AP-02/AP-03/AP-04/AP-05, DP-02/DP-08/DP-15/DP-17, PS-07/PS-09/PS-11; S02/S04/S05.

**Correction, Proposed.** Derive solver-independent case/structural/scientific facts first. Assess candidates through the existing representation, adapter settings and dynamic owners, with an explicit runtime/build snapshot. Return contextual impossibility, unresolved evidence, or a supported candidate with pending representation/kernel demands. Apply explicit selection or the existing automatic class/rank policy to those assessments, prepare the selected demands, and verify readiness before execution.

Here “first” means supplying the evidence needed by the consuming decision, not assuming
every structural or scientific fact is available without preparation. Current case structural
analysis consumes support from `CasePlan`. Assessment must be able to request that evidence:
an early structural-analysis demand can legitimately prepare conservative First support
without Second support or native evaluators. Return pending structural/capability evidence
when needed and reassess its consumers after preparation. The operations form explicit
evidence dependencies rather than a rigid single-pass pipeline; structural admission must
still be complete before native execution.

The adapter should own conditional requirements; a central metadata table must not independently reimplement its rules. Inventory should derive from those contracts. Keep algebraic derivative requirements distinct from dynamic sensitivity capabilities. Cancellation latency class, external starts, retained allocation/factor/search state and diagnostic support need typed contracts when callers actually make decisions about them; the current informational prose is not itself a defect.

**Legitimate countercases.** Explicit selection still refuses an incompatible backend. Exact versus limited-memory or finite-difference behavior remains an authored policy choice; Auto must not silently change it. A base solve may legitimately succeed while optional sensitivities are withheld. A conditional root factory that expressly consumes a KINSOL-specific map is not interchangeable with an arbitrary NLP oracle. Class-specific contracts remain necessary even under shared planning.

**Verification.** On a boxed square case with proven C2 availability but only First artifacts, candidate assessment must distinguish pending preparation from absent scientific derivatives. Genuine C1 and unavailable native-build cases must produce different reasons. Check method-dependent threading, cone subtypes, directional events, dynamic sensitivities and exact transient Hessians. Verify deterministic Auto ranking, explicit-only candidates, optional quantity withholding, retained-state compatibility and original-space outcome checks. The same assessment must be exercisable locally with typed capability/runtime facts.

### <a id="f03"></a>F03 — Missing assessment and resource failure are misinterpreted

**Implemented source path and retained execution evidence.** `workflow/numerics.rs::native_use` adds `CandidateRefusal::Infeasible` when `quality.as_ref().is_some_and(Quality::feasible)` is false. `None` and evaluated nonfeasibility therefore receive the same reason. Missing candidates separately receive `NoCandidate`. `workflow/modeling/results.rs::qualification_diagnostic` maps only TimeLimit/IterationLimit to ResourceLimit; ResourceExhausted falls through to TrialRejected. The PR automatic receipt contains native `resource_exhausted`, public `trial_rejected`, and `no_candidate; infeasible`.

`CandidateDecision::refuse` keeps these cases Unusable, and native termination is retained. The scientific/public explanation is misleading even though use safety remains enforced. NodeLimit, SolutionLimit and generic Limit also use the wildcard; their intended public classes need explicit decisions rather than incidental coverage by another category.

**Principles:** AP-02/AP-04, DP-02/DP-03/DP-21, G2/G7, PS-10/PS-12; S06.

**Correction, Proposed.** Completion consumes candidate presence, actual original-space assessment, native termination and use policy separately. Evaluated violations justify `Infeasible`; unavailable assessment requires its own typed reason, and evaluation failures retain their cause. One shared projection of termination/assessment meaning should govern public failure categories without replacing adapter-owned native status classification or weakening incumbent policy. A focused registry-backed refusal distinction may suffice; a universal assessment schema is not required.

**Verification.** Cover no-candidate/no-quality resource exhaustion, candidate with unavailable quality, evaluated infeasibility, callback/validation failure and a feasible limited incumbent. The first two must remain unusable without an invented feasibility verdict. Preserve violations and causes in the latter cases, and check every declared native category against its intended public projection, including contradiction handling.

## Solver characterization: contracts, conditions and library fit

This is an **Interface-checked** characterization of exposed adapters and their important restrictions. It is not an exhaustive catalog of intrinsic library capabilities, host availability, convergence, cancellation latency or sensitivity accuracy. Rust pins remain owned by `Cargo.toml`/`Cargo.lock`; native image sources and build observations remain under their existing owners. The linked table alone cannot establish which optional routine or linear solver a process can use.

| Adapter / inspected pin | Mathematical inputs and preparation | Conditional execution and lifecycle |
|---|---|---|
| Ipopt / native 3.14.20 | Continuous NLP, general variable/row bounds, values/gradient/sparse Jacobian; supplied Hessian needs Second, explicitly selected limited memory needs First | Selected linear solver determines linkage/thread/process requirements. Primal/dual starts and compatible retained C problem; callback/intermediate interruption |
| POUNCE / 0.12.0 | Smooth NLP/TNLP, general bounds, conditional supplied/limited-memory Hessian demand; L1 realization requires the appropriate method | Primal/dual/working-set starts and compatible retained application; library thread pool/FERAL; callback interruption and its own status semantics |
| KINSOL / sundials-sys 0.6.2, bundled SUNDIALS 7.1.1 | Square roots, analytic Jacobian or JVP; one-sided bounds become shifted signs, two-sided boxes refused | KLU/dense/Krylov and method-specific inputs; Picard/fixed-point need declared splitting/map and cannot use sign constraints. Serial, primal starts, compatible allocations and evaluation interruption |
| HiGHS / 2.4.0 and highs-sys 1.15.0 | Sparse LP/MILP and continuous convex QP coefficients; general bounds; no derivative callbacks; native linear/mixed-linear lexicographic route | Basis/primal/dual/partial-MIP starts and compatible data updates. Interrupt callbacks for simplex/IPM/MIP; QP/PDLP rely on native time limit. Adapter does not supply MIQP/nonconvex QP |
| Clarabel / 0.11.1 | Convex coefficient/cone data with evidence; SOC, exponential, power and generalized power; PSD requires `sdp` | No external warm payload. Data reuse restrictions include preprocessing, structural-zero dropping and chordal decomposition. Linear backend affects threads; iteration interruption and cone/certificate evidence |
| POUNCE-convex / 0.12.0 | Explicit-only LP/convex-QP/cone route with general boxes; translated cone ordering; generalized power refused | Independent parallel batches and internally retained compatible solution starts; no retained native factorization. No running interrupt; cancellation waits for the solve-wide deadline, which may overshoot by one factorization |
| SCIP / native 10.0.2, scip-sys 0.1.28 | Factorable LP/MILP/QP/MIQP/MINLP/NLP and native handlers; global certification depends on export/domain/bound evidence | Incumbents, concurrency and event interruption. Ordinary attempts recreate state; reoptimization has restricted edits. Exact mode restricts concurrency/reoptimization/IIS; global bounds do not certify an opaque callback |
| Diffsol / 0.16.2 | Exposed ODE/semi-explicit index-1 path, fixed diagonal 0/1 mass and First partials; implicit methods or mass-free Tsit45 | Directional events and recoverable typed trials refused. Forward reset sensitivities and their declared approximations differ from adjoints, which cannot cross events/resets/quadratures. Serial/cooperative stop |
| IDAS / same SUNDIALS baseline | Exposed residual `M*y' - f` with fixed diagonal 0/1 mass, consistent initialization and First partials; native library accepts broader residual DAEs | Signs, directional roots and recoverable trials. Forward sensitivities cannot cross events; adjoints exclude events/resets/quadratures and steady starts. Exact transient Hessian requires IDAS forward-over-adjoint, Second partials and bounded directions/checkpoints |

Source evidence is in `pse-backend-native/src/execution.rs::Capability/adapter` and `execution/{ipopt,pounce,kinsol,highs,clarabel,pounce_convex,scip,dynamics}.rs`, with conditional checks in `ipopt/settings.rs`, `kinsol.rs`, `conic.rs` and `dynamics.rs`. Structural policy, lexicographic degradation, native forms and formulation requirements remain distinct contracts. `batch` governs batch execution; `sensitivities` is an automatic selection preference followed by independent validity/withholding, not a guarantee. External `warm` and internally reused starts are different. The inspected routing consumers do not parse `reuse`, `cancellation` or `diagnostics` strings as policy.

The native-solver-libraries skill supplied pinned-source routes; Context7 corroborated primary [KINSOL constraints](https://github.com/llnl/sundials/blob/main/doc/kinsol/guide/source/Usage/index.rst), [Ipopt Hessian options](https://github.com/coin-or/Ipopt/blob/stable/3.14/doc/special.dox), [HiGHS problem classes](https://github.com/ERGO-Code/HiGHS/blob/master/docs/src/index.md), [Clarabel 0.11.1 data-update restrictions](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/solver/implementations/default/data_updating.rs), [Diffsol mass-matrix documentation](https://github.com/martinjrobins/diffsol/blob/main/book/src/primer/the_mass_matrix.md) and [SCIP nonlinear interfaces](https://scipopt.org/doc/html/cons__nonlinear_8h_source). Current documentation is discovery/corroboration; pinned source and actual adapter code govern version transfer. Newer Diffsol API syntax was not imported into 0.16.2 reasoning. POUNCE resolution returned unrelated Context7 matches, so its 0.12.0 source supplied that evidence.

Symbolica already owns arithmetic/differentiation; faer owns sparse layouts/algebra; existing structural and native integrations own matching/decomposition, presolve, tear optimization and iteration. Their integration owners should absorb representation and release changes. Demand coordination and physical/formulation policy belong to the simulator, but another symbolic derivative engine, Newton loop, generic graph framework or parallel capability authority is not justified. No dependency or license exclusion motivates the recommendation.

## Recommended target and alternatives

### Owned operations and data flow — Proposed

| Operation | Consumes and decides | Produces / consumer obligation |
|---|---|---|
| Check/select the semantic model | Checked revision, instance/static bindings, physical inventory, finite indexed realization; validates declared meaning and relationships | Specialized semantic inventory, topology and original obligations, without executable derivative artifacts |
| Bind/formulate the case | Analysis purpose, fixed/free roles, values, formulation selections and required observations/closure | Original rows/objectives, selected dependency closure and available structural/scientific facts; missing evidence has explicit preparation dependencies |
| Assess candidate realizations | Those facts plus adapter/method/settings, explicit build/runtime snapshot, guarantees and resource policy | Contextual refusal, evidence still needed, or supported representation with explicit artifact requirements; policy chooses among lawful candidates |
| Prepare demanded artifacts | Selected outputs, mandatory obligations, coordinates, derivative order, representation and finite construction limits | Immutable admitted value/derivative/coefficient/cone/factorable/trajectory products, complete identity and attribution, attributable preparation outcome |
| Admit/execute/qualify | Ready products, case values, native settings, explicit starts/reuse and attempt controls | Worker-owned execution, native outcome and independent original-space/physical evidence; completion decides use and derived-quantity validity |

These are operation contracts; existing functions and types can realize them. They do not require one new type per row, another crate, a serializable compiler instruction language or a universal graph IR. Static/indexed specialization remains a domain operation. Topology, incidence and solve order remain separate projections. Values bind to prepared structure; occurrence attribution is separate from reusable mathematical content.

The rows describe responsibility boundaries, not an unconditional one-way execution order.
Structural incidence or capability proof may require a demanded support product before
candidate selection can finish. Such evidence preparation belongs to its existing owner,
has explicit dependencies and finite limits, and must not accidentally demand all stronger
orders. Final execution requires both its structural witness and representation readiness;
pending evidence is neither structural acceptance nor scientific incapability.

For numerical stages, retain the profile's obligations explicitly:

| Stage | Formulation and derivative contract | Scaling, class and outcome obligations |
|---|---|---|
| Semantic/case admission | Authored domains, physical conventions and formulation choices before rewriting; no derivative artifact assumed | Declared roles/nominals; complete original rows and structural assessment; typed invalid/unknown/refused outcomes |
| Candidate assessment | Exact/approximate provider evidence and conditional kernel demand; algebraic order differs from dynamic sensitivity mode | Solver-independent class facts plus contextual adapter requirements; unsupported versus pending preparation versus unavailable runtime |
| Artifact preparation | Selected coordinate/output support and library derivative source/order; guards/effects never omitted because derivatives vanish | Budget before costly work; no native status or convergence claim; resource/cancel/infrastructure outcomes remain distinct |
| Native execution | Prepared callback/coefficient/cone/factorable/trajectory contract and selected method | Model-owned resolved normalization and tolerance budgets; adapter maps native termination; private attempt state |
| Completion | Original residuals/bounds/domains/checks, physical closure and endpoint obligations independent of native success | Scaled/native tolerances do not replace unscaled physical acceptance; missing assessment differs from failed assessment; optional analyses retain validity conditions |

Use **staged candidate assessment followed by bounded, selected preparation** as the preferred direction. Reuse existing class order and automatic ranks, including explicit-only capabilities. Resolve adapter/settings/representation conditions through their owners. Establish a typed pending demand rather than allowing the currently prepared order to define all future eligibility.

If preparation reveals a previously unknown mathematical incompatibility, any reconsideration must use the same case snapshot and an explicit deterministic policy. A resource, cancellation or infrastructure failure is not evidence of mathematical incompatibility and must not silently trigger another solver. No numerical attempt failure gains an automatic retry/fallback route. Approximation, relaxed formulation, branch choice and result-use guarantees remain explicitly selected.

Preparation identity must include consumed model/provider/physical dependencies, selected outputs and coordinates, representation/order/policy and relevant build contracts. Runtime observation is an explicit input to assessment, not an ambient read inside a supposedly pure query. An upgrade to stronger artifacts must preserve weaker products and their attribution; failure must not contaminate them. Existing cache/retention ownership should implement reuse rather than introducing another mechanism.

| Alternative | Assessment and selection condition |
|---|---|
| Current pipeline with larger budgets or explicit fixture backends | May restore individual cases and retains valid checks, but leaves topology/Value coupling and order-dependent Auto routing. Useful controls, insufficient remedy |
| Prepare the maximum requirements of all candidate solvers | Simple readiness rule but forces Second/factorable work on cases served by First/coefficient routes and can wrongly make optional work a prerequisite. Not the default for the observed failures |
| Tighten existing owners with staged demand/candidate assessment | Preferred smallest target: preserve scientific definitions and libraries while making consumed purpose/requirements govern expensive work. Benefits are Proposed until tested/measured |
| Replace authoring/compiler with a new generic DSL or optimizer framework | No evidence that this is needed for the reported defects; adds interpretation, migration and ownership cost. Revisit only if the existing semantic operations cannot express a required variation |
| Adopt library-owned representations and algorithms at existing seams | Retain/use where semantic fit is established. A library representation does not replace physical/formulation/permission policy; source-specific translation and conformance remain owned |

## Verification and independent judgments

### Retained execution evidence and coverage

These are prior runs read during review, not new validation of the committed tree. Native/Python development-profile execution and recipe-owned memory limits belong to those campaigns; the Arrow files do not independently seal their exact invocation or source revision. The observations below support their named cases only. All failure accounting is against **zero**.

| Evidence | Observation | Meaning and limit |
|---|---|---|
| **Tested**, selected CSTR native modeling-conformance receipt `build/seed-conformance-cstr-ipopt` | One fixture passed, complete selected coverage, 82 checks; 46 original free variables/equalities; 232 derivative comparisons with no reported issues. Explicit Ipopt/NLP route; KINSOL excluded by bounds | Suitable bounded CSTR route exists with original acceptance checks. This does not characterize C1 limitations or qualify other cases; sampling is not proof of exact derivatives |
| **Tested**, selected PR relational receipt `build/seed-conformance-pr-relational` | Auto Root selected SCIP/factorable; Ipopt/POUNCE derivative exclusions, KINSOL bounds exclusion. Native ResourceExhausted, no qualified candidate; public TrialRejected and `no_candidate; infeasible` | Supports F02/F03 manifestations; does not establish scientific infeasibility or a successful alternative solve |
| **Tested**, selected PR explicit-Ipopt receipt `build/seed-conformance-pr-ipopt` | One inconclusive fixture, two recorded checks; support construction required 2678 operations with 575 remaining, Taylor width 0. No route/structure rows were produced | Preparation failed before routing/solve; selecting Ipopt alone does not solve F01 or establish numerical adequacy |
| **Tested**, earlier selected PFR receipt `build/seed-conformance-pfr-reuse` | One inconclusive fixture; derivative support allowance exhausted before route/structure rows | Historical preparation boundary, preceding immutable fact-sharing repair. Not a current scaling measurement or proof that all cost is unrequested Second work |
| **Tested**, initial `just assessment` receipt `build/assessment/25k-integrated` | 42/42 checks attempted, six unsuccessful recipes; source changed during run. Default Rust 1988 passed/359 failed, native 2622/83, Python 180 passed/12 failures/1 error | A failed historical assessment, not a clean final gate. Environmental, fixture and implementation repairs since then cannot be inferred from these counts |

Static inspection establishes the **Implemented** causes in F01–F03 and the **Interface-checked** adapter contracts. No claim here is Measured or Formally established. The support failure counters are retained diagnostics, not an end-to-end performance comparison.

The conformance mechanism is shared and data-authored; a new model obtains common DOF, derivative, envelope and expectation checks through that harness. CSTR has the bounded passing receipt above; the selected PR/PFR families do not yet have complete positive receipts. Indexed connected/recycle, dynamics, implicit refusal, fitting and lifecycle controls in earlier packets are leads for later verification, not blanket acceptance of this broader target.

### Architectural foundations

| Foundation | Verdict and basis |
|---|---|
| AP-01 Separation of concerns | **Violated:** F01/F02 make topology/value consumers depend on stronger arithmetic work and preparation on an assumed backend |
| AP-02 Stable contracts | **Violated:** contextual readiness is incomplete for the planning consumer; F03 changes unavailable assessment into negative evidence. Existing body/binding/worker seams remain strengths |
| AP-03 Composition | **Violated:** consumer selection and backend requirements do not compose before complete preparation; S01/S02/S04 |
| AP-04 Domain model and authority | **Violated in preparation/routing/completion operations:** scientific concepts are substantially adequate, but demand/readiness and assessment absence do not sufficiently govern behavior |
| AP-05 Explicit structure | **Violated:** relevant consumer demands and contextual requirements become decisive after broad preparation or initial route selection |
| AP-06 Local reasoning/testability | **Violated:** topology/Value-only responsibilities require unrelated executable/support admission; pure route/completion policy seams should be preserved |

### Behavioral and profile gates

| Gate | Judgment | Evidence or settling limit |
|---|---|---|
| G1 Authority | **Unresolved for the combined scope** | Useful single scientific/adapter authorities inspected; no new competing mutable authority demonstrated, but complete supplier/consumer breadth was not audited |
| G2 Semantic fidelity | **Fail** | F03 conflates missing feasibility evidence with negative feasibility; native safety remains intact |
| G3 Validity | **Unresolved for the full case family** | Physical/structural/adaptation enforcement exists; complete PR/PFR, dynamics and conditional boundary behavior is not qualified |
| G4 Hidden behavior | **Unresolved for the combined scope** | Explicit runtime/job ownership inspected; proposed pure contextual assessment must receive process observations explicitly |
| G5 Consistency/recovery | **Unresolved for the combined scope** | Resource failures are explicit and candidate use is refused; entire retention/interruption/durable lifecycle not requalified |
| G6 Transformation/reuse | **Unresolved** | Useful body/instance/value separation; complete staged equivalence, reuse and full-case behavior need the specified controls |
| G7 Truthful capability claims | **Fail in public failure interpretation** | F03; full scientific capability qualification is also incomplete. F02 does not imply native admission is absent |
| G8 Library leverage | **Pass in the inspected math/structure/solver slice** | Libraries own arithmetic, sparse algebra and native iteration; no clearly redundant generic solver implementation identified. Not an exhaustive bespoke-code census |
| G9 Architectural fitness | **Fail** | Individual foundation violations above are not offset by passing numerical cases |
| PS-G1 Physical consistency | **Unresolved for full cases** | Complete physical distinctions and original checks exist; PR/PFR and connected temporal closure not comprehensively established |
| PS-G2 Well-posedness | **Unresolved across combined routes** | Sound inspected selected structural/locality enforcement; no evidence here of topology substituted for incidence |
| PS-G3 Numerical integrity | **Fail for public status classification; broader scope unresolved** | F03 misclassifies ResourceExhausted in the public failure projection; preserved native termination and Unusable permission prevent a false solution |

These judgments assess the current implementation. Proposed corrections do not make it pass. Unexamined breadth is a coverage limit rather than an invented finding.

## Decision, authority route and qualification handoff

**Behavioral/semantic adequacy: Revise. Architectural fitness: Revise. Overall: Revise.** The recommended staged direction is supported at **Proposed** evidence, informed by Implemented source causes and Interface-checked library contracts. It has not been accepted as an implemented or numerically qualified target.

First settle the shared semantic-case/candidate-demand contract in F01/F02: the compiler supplies solver-independent meaning and pending scientific evidence; adapter/representation owners supply contextual requirements; policy selects; the compiler/math service prepares required products. That contract enables the urgent Auto-routing correction without installing another backend assumption. Topology/selected-output and derivative-stage separation are distinct F01 obligations. F03 can be corrected independently and should accompany this work so later failures remain interpretable.

| Required follow-up | Proposed owner and route | Acceptance before closure |
|---|---|---|
| F01/F02 target and dependency boundaries | Compiler/modeling/math and routing/adapter owners jointly; subsequent authorized implementation plan | Agreed consumed contracts; mandatory obligation preservation; local demand/candidate controls and actual scientific journeys |
| New compiler pass/kernel contracts or changed routing policy | Short ADR under the existing decisions if appropriate; broader D1–D14/semantic contract changes follow ADR plus design-review route | ADR precedes code; accepted architecture amended through `design:` route with revision row, not this review |
| F03 assessment and generated public vocabulary | Completion/diagnostic owners; registry owns any new refusal value; evaluate durable/Python boundary evolution under ADR-0146/0151 and the existing decision rules | No-candidate/quality/status matrix; preserving recorded contracts; regenerate declarations rather than editing generated output |
| Enduring explanation | Blueprint §14.1/§14.3 and §18.7; relevant §19/§23.2 owners | Separate semantic admission, pending preparation and full contextual admission; update only through authorized design route |
| Integrated scientific/performance acceptance | Existing 25k owner | Corrected original-oracle cases, selected integrated journeys, final composite qualification and later dev-profile counts/timings/resource evidence |

No SHOULD exception or MUST waiver is proposed. Current allowance increases and explicit fixture route choices remain controls, not architectural closure. Do not narrow claimed full flowsheet support solely in a verdict while leaving the same broad capability advertised.

Qualification remains paused at the maintainer's review boundary. The next consequential decision is adoption of these contracts and findings into the existing work owner and an authorized follow-up implementation plan. Resume broad qualification after that decision and the relevant corrections/targeted controls, or after the maintainer explicitly directs a different bounded qualification scope. Later scientific runs preserve authored bounds, applicability permissions, original residual/closure checks and independent reference oracles. Measurement retains the selected Linux dev profile, bounded scheduling and native memory limits; no speedup or successful full-case scaling is claimed by this review.
