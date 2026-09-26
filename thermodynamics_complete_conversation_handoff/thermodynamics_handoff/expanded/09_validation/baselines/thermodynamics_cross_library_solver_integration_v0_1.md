# Cross-library compatibility and solver integration blueprint

**Document:** THERMO-INTEGRATION-008  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 8, resolve cross-library and solver integration rules  
**Status:** Proposed language-independent integration decisions and provisional provider-profile routes. Selected official documentation was checked and authored algebra/reference guards were executed. No native thermodynamic integration, production qualification or independent scenario R/E/V promotion is claimed.

## 0. Decisions and what this stage completes

Use **curated, operation-qualified thermodynamic configurations**, not a pool of nominally interchangeable property functions. Retain complete external packages when they already contain coherent methods, data, calorics and equilibrium algorithms. Permit smaller contributions when their meaning is clear and the host deliberately accepts the additional modeling, derivative, initialization and validation responsibilities.

The principal decisions are:

1. **Separate the reuse boundary from the solution arrangement.** A whole package may serve a direct calculation or a qualified condensed block in a coupled model. A phase model may serve an explicit host formulation. Neither boundary dictates sequential-modular versus equation-oriented process solution.
2. **Freeze the actual function being evaluated for a captured solve.** Method definitions, fixed data, references, physical restrictions and provider realization remain attributable. State variables and parameters deliberately declared as estimation/decision coordinates may change within their captured formulation; they are not hidden edits of shared approved data. Native fallback, data substitution or regional provider switching cannot silently change the function while its old derivatives or old identity remain in use.
3. **Qualify caloric and reactive consistency at the point of composition.** An EOS capable of phase splitting is not automatically a complete energy package. A post hoc energy addition does not repair an inverse solve performed with the wrong enthalpy function.
4. **Distinguish mathematical solver compatibility from numerical feature availability.** A successful flash is not automatically a smooth optimization callback. Derivatives must correspond to the actual local or solved-response observable, with its coordinate chart, branch and required order.
5. **Allow deliberate engineering approximations.** The architecture must support useful conventional correlation-based packages, rather than requiring every material to derive from one thermodynamic potential. It must also retain what each approximation does and does not establish.
6. **Treat heat coupling, material translation and representation mapping as different compositions.** Cross-package exchanges require explicit preserved quantities and conventions; neither an interface standard nor equal units creates physical compatibility.
7. **Keep provider selection profile-specific.** Conventional fluid, utility, reactive aqueous/mineral, petroleum, empirical-material and coupled-contacting routes have different qualification burdens. Unresolved P2 capability remains a delivery gap, not an optional omission disguised as an unsupported-provider response.

This stage makes decisions about the admissible integration patterns, rejection conditions, initial qualification targets and required evidence. It does **not** select a universal backend, choose a Rust boundary, promise that an untested provider meets our contracts, or replace the later execution of numerical and process fixtures.

### Baselines and new evidence

B1 controls scope; B4 controls functional behavior; B5 controls responsibility ownership; B6 controls information meaning; B7 controls action/lifecycle behavior. B2/B3 remain the fixed research baselines, including their source pins and stated limitations. No predecessor is edited.

The additional E01–E14 references are selected official solver/library documentation consulted for this stage. Their tracks are identified individually, including versioned Pyomo pages and moving `stable`/`vcurrent` pages. They are **not** a refresh of every upstream source file or proof that a distributed runtime matches either those pages or the earlier source pins. No native package was executed here.

The record called an **integration profile** below is a coordinated view of existing IC-20/23/24/34/35/39/40/41/53/58/64/70/72 and their related data. It is not a new seventy-third material concept, a physical storage layout, or a required API object.

### Reading routes

Read §§1–2 for the decision model and the two axes; §§5–7 for solver and thermodynamic composition; §8 for concrete candidate profiles; §9 for the qualification gates and experiments. The detailed rule and composition catalogs in §§3–4 and traceability in §10 make these choices actionable against the existing AC/IC/FR records.

## 1. What is being qualified

A named library is too coarse a unit of qualification. The relevant subject is a **particular use of a particular material definition and physical configuration through a particular exposed realization and solver arrangement**.

| Profile facet | Required meaning | Existing information home |
| --- | --- | --- |
| Material and domain | Species/components/lumps, account basis, justified conserved quantities, phase/nonbulk domain meanings | IC-01–09, IC-25–33 |
| Resolved definition | Actual phase, ideal/residual/excess or correlation contributions, parameter values or attributable bundle, standard states and references | IC-12–24 |
| Physical problem | Operation, fixed constraints, unknowns, allowed phase/chemical changes, required outputs and checks | IC-34–47 |
| Process use | Local locations, unit mode, preserved boundary quantities, transfer/routing and completion scope | IC-48–52, IC-71 |
| Numerical realization | Engine/build/adapter/data, actual native coordinates, algorithm and initializer, private session and error/isolation behavior | IC-53–58, IC-72 |
| Solver contract | Local versus total solved derivatives, variable ownership, sparsity, scaling, active-branch treatment and inner error policy | IC-39–41, IC-54/57/72 |
| Qualification and authority | Evidence, limitations, captured dependencies, permitted run and coherent publication scope | IC-59–70 |

### Four independent quality questions

**Numerical regularity:** Is the actual evaluated function suitable for the chosen algorithm? Is it differentiable on the used branch, can trial points be evaluated, and is inner noise controlled?

**Thermodynamic consistency:** Which identities and standard-state relationships does the assembled model preserve? Is an intentionally approximate caloric/phase combination declared as such?

**Engineering applicability:** Are the selected material/data/conditions and requested outputs within the accepted use or explicitly authorized extrapolation? Do they address the process question rather than only a standalone property?

**Evidence:** What has been inspected, executed, checked against the contract, compared with reference data, and reproduced? None of these questions is answered solely by a positive answer to another.

The difference is operationally important. A smooth but physically poor model can be easy to optimize. A physically useful phase model may have a nonsmooth transition unsuitable for an unmodified smooth solver. An engineering caloric approximation can be valuable for a bounded duty study without satisfying every exact thermodynamic identity. A mathematically compatible assembly can still lack adequate parameter data.

### Admission is specific, directed and not generally transitive

Compatibility of A and B for one property does not establish their compatibility for phase equilibrium, energy or derivatives. A directed representation map does not establish a unique inverse. Two pairwise-approved submodel combinations need not form one jointly coherent configuration.

A profile may be ready for **limited evaluation**, ready for **initialization trials**, or eligible for a **particular accepted process operation**. Unassessed required prerequisites block the corresponding claim, not necessarily every use of the material. Explicit provisional research contexts are allowed without production-publication authority. [B4:FR-CFG-03; B6:IC-24; B7:AC-09/45]

### What gets fixed versus what stays open

This document fixes compatibility/rejection rules and names the first profile routes to qualify. Exact real-material fixtures, data artifacts, stable binary/build manifests and numerical/scientific tolerances must be instantiated before the Step-9 witnesses run. Those absent details remain visible in each candidate template rather than being fabricated to make it look deployment-ready.

## 2. Two independent integration axes

### 2.1 Reuse boundary

The first axis asks **how much coherent functionality is retained from a library**. A larger coherent boundary is often more reusable than an isolated EOS when the surrounding library already supplies compatible calorics, parameter handling and flashes. A smaller boundary can be appropriate when the host needs shared unknowns, special physics or a directly assembled mathematical problem.

| Boundary | What is retained | Additional host responsibility | Examples |
| --- | --- | --- | --- |
| RB-01 Data/correlation source | Evidence, parameter interpretation and limited observable meaning | Selection, coherent assembly, inverse solving, derivative and acceptance contract where not supplied | chemicals data/correlations; selected characterized or fitted data |
| RB-02 Phase/property bundle | Coherent constitutive properties and local phase state | Interphase/chemical problem, unit constraints and allowed derivative assembly | thermo phase model; qualified Clapeyron, ThermoPack, CoolProp or FeOS property surface |
| RB-03 Complete nonreactive property package | Compatible phase/caloric/data descriptions plus eligible flashes | Material identity, request authority, unit/routing/lifecycle and provider qualification | DWSIM configured package; thermo phase/flash assembly; Clapeyron model plus selected algorithm |
| RB-04 Complete chemical-system calculation | Species/phases, thermochemical/activity data, restrictions and coupled equilibrium or kinetic subset | Flowsheet amount/energy exchanges, unit transport/geometry, publication and exact adapter coverage | Reaktoro coherent system; qualified IDAES chemical formulation |
| RB-05 Mathematical contribution | Declared physical equations/residuals and their semantic prerequisites | Global variable/constraint assembly, scaling and solve; no inferred derivative through opaque calls | IDAES-style state/reaction contribution; host model with qualified external functions |


### 2.2 Where the unknowns are solved

The second axis asks **who determines the unknown state and how that determination participates in the process solve**. The five solver modes below are different contracts, not a ranking from immature to advanced.

| Mode | Who solves local unknowns | Main use |
| --- | --- | --- |
| [SM-01](#sm-01) Direct solved-state service | Provider solves requested local state; host unit/process model owns its external balances and any recycle. | Heaters, valves, flash drums, utility states and coherent reactive equilibrium blocks in a sequential or hierarchical flowsheet. |
| [SM-02](#sm-02) Supplied-phase constitutive callback | Host fixes/supplies local coordinates; provider may resolve only expressly permitted dependent coordinates such as density/root. | Column phase properties, rate-based bulk/interface states, rate-law inputs and host residual construction. |
| [SM-03](#sm-03) Condensed solved block | Provider eliminates internal y from g(u,y)=0; host solves remaining process residuals in u. | Embedding expensive flashes or chemical equilibria into a coupled process solve while reusing their native solvers. |
| [SM-04](#sm-04) Exposed local residual block | Host includes local phase/chemical variables and solves their residuals with unit equations. | Strongly coupled stages, reactive separation and formulations for which hiding equilibrium would produce difficult nested solves. |
| [SM-05](#sm-05) Explicit equation contribution | Host compiler/formulation owns declared algebraic variables and constraints; provider contributes physical relationships. | Native equation-oriented process compilation, parameter estimation, scaling and sparse solver integration. |


For example, a complete nonreactive package can run as SM-01 for a valve and, with separately qualified total derivatives, as SM-03 inside a larger optimizer. The same backend might also offer an SM-02 phase-only surface. A library that only returns a solved state cannot be assumed to expose SM-04 residuals or SM-05 equations.

The host should not force every location through a completed stream flash. Conversely, obtaining explicit host equations is not automatically worth discarding a mature complete solver. The choice is made per problem based on actual exposed functionality, coupling, derivative quality and robustness. [B3 §§7–9; B5:PD-03/14; B7:AC-22/26/43]


## 3. Integration-rule catalog

These are proposed design admission and rejection rules. Their positive/negative witnesses are specifications for actual integration tests unless separately identified in the authored reference script. They do not amend the original FR obligations.

| Rule | Decision | Accountable package |
| --- | --- | --- |
| [IR-01](#ir-01) | Qualify a complete directed integration profile | P03 |
| [IR-02](#ir-02) | Preserve a coherent constitutive and phase description | P03 |
| [IR-03](#ir-03) | Bind parameters to equations and ordered roles | P02 |
| [IR-04](#ir-04) | Map identity, quantity and every tensor axis | P01 |
| [IR-05](#ir-05) | Complete calorics inside the state problem | P03 |
| [IR-06](#ir-06) | Permit named engineering approximations without false consistency claims | P03 |
| [IR-07](#ir-07) | Augment properties without taking state authority | P05 |
| [IR-08](#ir-08) | Keep reactive equilibrium, participation and exchanges coherent | P06 |
| [IR-09](#ir-09) | Reconcile energy references and chemical standards separately | P03 |
| [IR-10](#ir-10) | Distinguish heat-only coupling, representation mapping and material transition | P07 |
| [IR-11](#ir-11) | Differentiate the actual observable in the declared coordinates | P05 |
| [IR-12](#ir-12) | Qualify reduced solved blocks and their inner error | P05 |
| [IR-13](#ir-13) | Match the solver derivative order and structural contract | P05 |
| [IR-14](#ir-14) | Treat phase and branch changes as mathematical events | P05 |
| [IR-15](#ir-15) | Qualify initialization and restore the target formulation | P08 |
| [IR-16](#ir-16) | Give each unknown and constitutive relation one authority | P05 |
| [IR-17](#ir-17) | Freeze the realized function and distinguish alternative problems | P08 |
| [IR-18](#ir-18) | Contain the full session sequence and uncertain native failure | P08 |
| [IR-19](#ir-19) | Report trial-domain errors without silently changing coordinates | P08 |
| [IR-20](#ir-20) | Require complete original-problem checks and qualified optional work | P09 |
| [IR-21](#ir-21) | Preserve run-local coherence and guarded current publication | P10 |
| [IR-22](#ir-22) | Archive exact reconstructable meaning and qualify actual dependencies | P10 |
| [IR-23](#ir-23) | Keep limited and nonbulk materials useful without invented physics | P01 |
| [IR-24](#ir-24) | Promote evidence only at the tested profile and scenario scope | P09 |

<a id="ir-01"></a>
### IR-01. Qualify a complete directed integration profile

**Accountability:** P03. **Applies when:** A library, algorithm or model is selected, replaced or combined.

**Decision:** Admit an operation only for its complete material/data/method/physical-policy/realization/solver-mode tuple; compatibility is purpose- and direction-specific.

**Admission evidence:** Captured IC-20/35/53; observable and amount conventions; phase/chemistry authority; mandatory output and initializer needs; exposed support evidence. Preserve supported, unsupported and unassessed distinctions.

**Rejection or bounded-use rule:** A catalog label, successful constructor or pairwise compatibility graph is not sufficient. A compatible A-to-B reporting map need not make B-to-A reconstruction valid, nor does A/B and B/C compatibility establish A/B/C coherence.

**Positive witness:** A direct TP profile is admitted while its same-material PH variant waits for caloric and initializer evidence.

**Negative/boundary witness:** A generic supports-flash flag cannot authorize reactive VLLE PH or an exact Hessian.

**Actions:** [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07), [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08), [AC-09](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-09), [AC-19](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-19), [AC-20](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-20), [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21), [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27). **Information:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23), [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53). **Basis:** B4:FR-CFG-03, B3:CR-04, [E11](#e11).

<a id="ir-02"></a>
### IR-02. Preserve a coherent constitutive and phase description

**Accountability:** P03. **Applies when:** A whole package or a combination of phase/property methods is configured.

**Decision:** For each phase/equilibrium/caloric assembly, state the selected constitutive relationships and their compatible standard states before treating the assembly as a package.

**Admission evidence:** Vapor fugacity, liquid standard-state/activity or EOS convention, saturation/Henry treatment, reference data, caloric relationships and selected phase problem. Whole-provider internals may remain bundled when attributable.

**Rejection or bounded-use rule:** Matching names or signatures is not compatibility. A phase model that cannot represent the material or required root cannot be repaired by arbitrary data from another package.

**Positive witness:** A named gamma–phi configuration records its solvent and dilute-solute conventions and the associated thermal model.

**Negative/boundary witness:** An NRTL coefficient vector and unrelated enthalpy/density values do not establish an energy-consistent flash.

**Actions:** [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07), [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08), [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22), [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23), [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24). **Information:** [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38). **Basis:** B3:W01, B4:FR-CFG-02, [E08](#e08), [E09](#e09).

<a id="ir-03"></a>
### IR-03. Bind parameters to equations and ordered roles

**Accountability:** P02. **Applies when:** Data are imported, fitted, transferred between implementations or supplied to a submodel.

**Decision:** Translate parameters only after their formula, ordered subjects, units, temperature dependence, logarithm convention and missing-value meaning are matched.

**Admission evidence:** Exact parameter interpretation; original evidence and selected values; conversions of all coefficients and independent variables; explicit missing/zero/omitted/estimated status.

**Rejection or bounded-use rule:** Name equality such as A12, a value matching at one temperature, or automatic symmetrization is not sufficient. Incompatible or unknown formula meaning leaves the data unresolved.

**Positive witness:** A directed interaction table and its temperature derivatives are reproduced across an explicit formula conversion.

**Negative/boundary witness:** Two tau functions agreeing at 300 K but with different temperature slopes cannot be treated as equivalent caloric models.

**Actions:** [AC-04](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-04), [AC-05](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-05), [AC-06](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-06), [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08), [AC-28](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-28). **Information:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-11](thermodynamics_semantic_information_dictionary_v0_1.md#ic-11), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-13](thermodynamics_semantic_information_dictionary_v0_1.md#ic-13), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17). **Basis:** B4:FR-DAT-01, B4:FR-DAT-02, B6:SI-11, B3:L03.

<a id="ir-04"></a>
### IR-04. Map identity, quantity and every tensor axis

**Accountability:** P01. **Applies when:** A provider or process representation differs from the canonical semantic description.

**Decision:** Use explicit material/phase/coordinate mappings for every input and output, including derivative axes and parameter subject roles; preserve the distinction between physical accounts and reporting views.

**Admission evidence:** Constituent mapping, normalized composition chart, total/molar/mass/reference-amount interpretation, native phase roles and disjoint partition declaration. Map forward and reverse separately.

**Rejection or bounded-use rule:** Array order, names, aggregate liquids and apparent/true aliases cannot create material. Missing molecular information blocks only conversions that actually need it.

**Positive witness:** A constituent permutation maps compositions, property vectors and both Jacobian axes while preserving original material totals.

**Negative/boundary witness:** Composition is reordered but the Jacobian, activity parameter matrix or elemental basis remains in the old order.

**Actions:** [AC-01](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-01), [AC-02](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-02), [AC-03](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-03), [AC-12](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-12), [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13), [AC-14](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-14), [AC-15](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-15), [AC-16](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-16), [AC-17](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-17), [AC-18](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-18), [AC-28](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-28), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33). **Information:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-30](thermodynamics_semantic_information_dictionary_v0_1.md#ic-30), [IC-31](thermodynamics_semantic_information_dictionary_v0_1.md#ic-31), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). **Basis:** B6:SI-01, B6:SI-05, B6:SI-06, B4:FR-MAT-07.

<a id="ir-05"></a>
### IR-05. Complete calorics inside the state problem

**Accountability:** P03. **Applies when:** Residual, ideal, excess, formation or empirical caloric pieces are combined.

**Decision:** Every energy/entropy-constrained solve shall use the complete selected caloric function during the solve and in final verification, not add a missing contribution afterward.

**Admission evidence:** Declared decomposition or indivisible complete function; consistent reference and amount basis; required H,S,U and derivative dependencies; complete function supplied to inverse equations.

**Rejection or bounded-use rule:** Residual-only PH followed by a post hoc ideal enthalpy addition is not a total-PH solution. Adding a second ideal contribution to a provider total double counts it.

**Positive witness:** A host inverse using h_total(T)=h_part1(T)+h_part2(T) reaches the original total target.

**Negative/boundary witness:** A partial model solves 50 kJ/kg at 325 K, but the completed model gives 125 kJ/kg at that point; the candidate fails.

**Actions:** [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07), [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08), [AC-09](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-09), [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23), [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-41](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-41), [AC-42](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-42). **Information:** [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41). **Basis:** B4:FR-CFG-04, B3:TP04, B3:CL05, B3:FE01.

<a id="ir-06"></a>
### IR-06. Permit named engineering approximations without false consistency claims

**Accountability:** P03. **Applies when:** Correlation-based calorics, approximate phase coupling, smoothing, reduced materials or fitted replacement functions are selected.

**Decision:** Allow useful engineering approximations as explicit package/formulation variants, while separating numerical regularity, thermodynamic identity consistency, operating applicability and independent validation.

**Admission evidence:** Which equalities/consistency identities the approximation preserves or relinquishes; intended operations; valid evidence domain; the actual function and derivative to be solved; declared acceptance obligations.

**Rejection or bounded-use rule:** Neither exact-potential consistency for every useful model nor silent inconsistency is imposed. An approximation cannot claim identities it does not satisfy or use a different function for derivatives.

**Positive witness:** A declared approximate caloric mode can support a bounded duty study with its own checks and labels.

**Negative/boundary witness:** Its phase Cp is reported as dh/dT of a different enthalpy function or its approximation label disappears in optimization results.

**Actions:** [AC-05](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-05), [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07), [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08), [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38), [AC-54](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-54). **Information:** [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15), [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61). **Basis:** B4:FD-07, B3:W01, [E08](#e08), [E06](#e06).

<a id="ir-07"></a>
### IR-07. Augment properties without taking state authority

**Accountability:** P05. **Applies when:** A separate library supplies viscosity, diffusivity, surface tension, reporting properties or an effective mixture relationship.

**Decision:** Attach supplemental transport, interfacial or effective-property calculations only to a declared supplied state, preserving protected phase/species quantities and naming any additional closure.

**Admission evidence:** Required state values; phase pair or diffusion frame; composition and solvent basis; extra closure data; mandatory-versus-optional demand; state dependence for coupled derivatives.

**Rejection or bounded-use rule:** The supplement may not reflash or respeciate by default. A pure-solvent correlation cannot silently stand in for arbitrary concentrated solution behavior. Viscosity is not a transfer coefficient.

**Positive witness:** An approved liquid viscosity method evaluates a captured liquid phase while leaving the flash allocation unchanged.

**Negative/boundary witness:** A viscosity query rewrites aqueous speciation or erases a second liquid; authority checks reject its use.

**Actions:** [AC-17](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-17), [AC-20](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-20), [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22), [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-55](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-55). **Information:** [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-31](thermodynamics_semantic_information_dictionary_v0_1.md#ic-31), [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50). **Basis:** B4:FR-PRP-03, B4:FR-PRP-04, B4:FR-FLW-08, B7:AR-03.

<a id="ir-08"></a>
### IR-08. Keep reactive equilibrium, participation and exchanges coherent

**Accountability:** P06. **Applies when:** Reaction or speciation is coupled to phase allocation and energy.

**Decision:** Use a coherent chemical-system or explicitly coupled formulation for reacting multiphase calculations; retain strict conversion, kinetic, frozen and equilibrium meanings and account for every authorized exchange exactly once.

**Admission evidence:** Species and conserved basis; thermochemical/activity data; participation restrictions; extent/rate convention; thermal conditions; external exchange permissions and realized transfers; final combined checks.

**Rejection or bounded-use rule:** Fixed-species flash is not chemical equilibrium. Sequential successes are insufficient if the final state violates earlier chemistry. A retry cannot consume material again or optimize an infeasible strict conversion silently.

**Positive witness:** A mixed kinetic-solid/equilibrated-aqueous profile checks final quantities under its declared restricted chemistry.

**Negative/boundary witness:** A closed request reaches a reservoir target only by undeclared titrant or gas addition.

**Actions:** [AC-10](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-10), [AC-11](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-11), [AC-12](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-12), [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37), [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43). **Information:** [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43), [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-46](thermodynamics_semantic_information_dictionary_v0_1.md#ic-46), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). **Basis:** B4:FR-CHM-02, B4:FR-CHM-08, B7:AR-14, [E13](#e13).

<a id="ir-09"></a>
### IR-09. Reconcile energy references and chemical standards separately

**Accountability:** P03. **Applies when:** Material enthalpy, reaction heat, chemical potentials or standard states cross a boundary.

**Decision:** Before combining reactive or cross-package energy values, define the reference transformation and corresponding formation/reaction correction; do not assume a bookkeeping energy offset preserves arbitrary chemical-equilibrium conventions.

**Admission evidence:** Known transformation including composition dependence; chemistry basis and reference requirements; which formation terms are already in material H; genuine exchange/work; dimensionless equilibrium conventions.

**Rejection or bounded-use rule:** Unknown transforms cannot be guessed. A constant species offset cancels in nonreactive conserved-component balances but may change reaction-energy differences; arbitrary per-species chemical-potential shifts may change equilibrium.

**Positive witness:** Equivalent formation-inclusive and correction-based formulations produce the same selected duty after an explicit reference adjustment.

**Negative/boundary witness:** Adding the full reaction heat twice or removing a model discrepancy as if it were reference energy fails the accounting checks.

**Actions:** [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08), [AC-10](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-10), [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24), [AC-28](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-28), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-41](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-41), [AC-42](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-42). **Information:** [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43), [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-51](thermodynamics_semantic_information_dictionary_v0_1.md#ic-51), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). **Basis:** B4:FR-CHM-05, B6:IC-21, B6:SI-14, B3:CO03.

<a id="ir-10"></a>
### IR-10. Distinguish heat-only coupling, representation mapping and material transition

**Accountability:** P07. **Applies when:** Flowsheet regions share heat or a material is represented by a different package or slate.

**Decision:** Use different boundary contracts for heat exchange, coordinate/reporting mapping and actual material re-description under another property package.

**Admission evidence:** For heat: within-side coherent differences and process heat balance. For mapping: conserved basis and information loss. For material transition: preserved independent quantities, known reference relation and target-state calculation.

**Rejection or bounded-use rule:** A wall is not a component translator. An irreversible lumping map has no unique inverse. Incompatible T,P,z and corrected H cannot all be preserved through hidden heat.

**Positive witness:** Steam and a hydrocarbon circuit exchange duty through separate packages with no cross-wall species map.

**Negative/boundary witness:** A package switch preserves incompatible T and H by fabricating an unrequested heat input.

**Actions:** [AC-03](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-03), [AC-12](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-12), [AC-17](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-17), [AC-39](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-39), [AC-41](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-41), [AC-42](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-42). **Information:** [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-51](thermodynamics_semantic_information_dictionary_v0_1.md#ic-51), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). **Basis:** B4:FR-FLW-04, B4:FR-FLW-05, B4:FR-FLW-06, [E07](#e07).

<a id="ir-11"></a>
### IR-11. Differentiate the actual observable in the declared coordinates

**Accountability:** P05. **Applies when:** A solver or sensitivity caller requests any first, second or directional derivative.

**Decision:** Derivative evaluation shall refer to the exact function, parameters, references, independent chart and phase/chemical response used for values, with all required chain-rule terms.

**Admission evidence:** Value function identity; derivative input/output meaning, held-fixed quantities, amount/chart, temperature-dependent parameter and reference contributions, method and branch.

**Rejection or bounded-use rule:** A homogeneous derivative is not a derivative through a flash. A derivative from a different engine is not accepted merely because values agree at one point. Automatic differentiation outside an opaque call does not reveal its internals.

**Positive witness:** A derivative changes appropriately when a normalized composition chart or nonlinear coordinate transform is used.

**Negative/boundary witness:** Use cp with phase allocation frozen as the derivative of total enthalpy after re-equilibration.

**Actions:** [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-28](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-28), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36). **Information:** [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58). **Basis:** B4:FR-PRP-05, B6:SI-05, B3:CR-15.

<a id="ir-12"></a>
### IR-12. Qualify reduced solved blocks and their inner error

**Accountability:** P05. **Applies when:** An inner phase/chemistry solve eliminates local unknowns before an outer solver evaluates residuals.

**Decision:** A condensed external solve used inside a coupled solver must specify branch regularity, total solved-state response and an inner error policy commensurate with outer residual and derivative demands.

**Admission evidence:** Inner equations or an independently qualified total-response route; fixed active branch or explicit transition treatment; nonsingular local system when implicit differentiation is used; residual evidence and sensitivity-conditioned tolerance.

**Rejection or bounded-use rule:** Small inner step size is not an error bound. Differentiating only outer direct terms misses implicit response. If inner equations are unavailable, qualify full re-solve finite differences or restrict solver mode instead of claiming implicit derivatives.

**Positive witness:** For y²-u=0 on y>0 and F=y³+u, the derivative includes dy/du and equals 4 at u=4.

**Negative/boundary witness:** Returning the frozen-y derivative 1 for that reduced function is rejected.

**Actions:** [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23), [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24), [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-29](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-29), [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43). **Information:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). **Basis:** B4:FR-FLW-07, [E04](#e04), [E05](#e05).

<a id="ir-13"></a>
### IR-13. Match the solver derivative order and structural contract

**Accountability:** P05. **Applies when:** A problem is passed to a sparse nonlinear equation solver or optimizer.

**Decision:** Supply the exact derivative order, multiplier weighting, dimensions, sparsity and scaling required by the selected solver mode; do not infer structural independence from a numerical zero.

**Admission evidence:** Solver-mode declaration; Jacobian availability; full Lagrangian-Hessian or an explicit compatible approximation; variable/residual chart and scaling; a stable structural layout during the solve.

**Rejection or bounded-use rule:** A library using second derivatives internally has not thereby exposed output Hessians. Limited-memory Hessian approximation does not create a missing Jacobian or smooth a phase discontinuity.

**Positive witness:** A first-derivative-qualified profile uses a solver Hessian approximation explicitly, retaining correct values/Jacobian.

**Negative/boundary witness:** The number or ordering of callback outputs changes when a phase appears mid-solve, or an at-start zero is deleted from required sparsity.

**Actions:** [AC-20](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-20), [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-29](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-29), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43). **Information:** [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). **Basis:** B4:FR-PRP-06, B6:IC-40, [E01](#e01), [E02](#e02), [E03](#e03), [E11](#e11).

<a id="ir-14"></a>
### IR-14. Treat phase and branch changes as mathematical events

**Accountability:** P05. **Applies when:** A solve spans appearance/disappearance, saturation, coalescence or alternate roots.

**Decision:** Choose an explicit branch/phase-transition strategy for the formulation, and keep physical candidates, solver layout, searched scope and verified stability separate.

**Admission evidence:** Fixed branch/domain for a local callback, a qualified fixed-superset formulation, or explicit outer restructuring/restart; endpoint/quality treatment for saturation; branch-selection and stability evidence.

**Rejection or bounded-use rule:** A sign change across a discontinuity is not proof of a scalar root. A fixed number of numerical slots is not physical phase count. Smooth local NLP success is not unrestricted stable equilibrium.

**Positive witness:** A pure-fluid saturation profile resolves a permitted quality/enthalpy specification without pretending TP alone fixes phase amounts.

**Negative/boundary witness:** A two-liquid request is silently reduced to one liquid to preserve an old callback layout.

**Actions:** [AC-16](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-16), [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21), [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23), [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-40](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-40). **Information:** [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-30](thermodynamics_semantic_information_dictionary_v0_1.md#ic-30), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56). **Basis:** B4:FR-EQL-04, B4:FR-EQL-06, B6:SI-09, [E06](#e06), [E11](#e11).

<a id="ir-15"></a>
### IR-15. Qualify initialization and restore the target formulation

**Accountability:** P08. **Applies when:** A library or coupled process formulation needs a seed, phase search, envelope, fixed variables or relaxed equations.

**Decision:** Record supported initial-region requirements and temporary interventions separately from the original problem; restore original conditions before its acceptance.

**Admission evidence:** Initializer target, compatible seed provenance, permitted interventions, restoration/reconciliation plan, final original-target checks and bounded attempts.

**Rejection or bounded-use rule:** Failure to find a start is not physical infeasibility. A guessed temperature is not an extra unit constraint. A phase policy used for initialization cannot become accepted physics unnoticed.

**Positive witness:** A two-phase-only initializer is attempted only with compatible seed evidence, and the original equations are checked afterward.

**Negative/boundary witness:** An initialization fixation remains active and overconstrains the supposed original coupled model.

**Actions:** [AC-09](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-09), [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-29](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-29), [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31), [AC-34](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-34), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43). **Information:** [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). **Basis:** B4:FR-RUN-01, B4:FR-RUN-02, B3:FE02, [E11](#e11).

<a id="ir-16"></a>
### IR-16. Give each unknown and constitutive relation one authority

**Accountability:** P05. **Applies when:** A whole flash or property block is embedded in a unit or global equation system.

**Decision:** Explicitly allocate local unknowns and equation responsibilities between host and provider; choose either elimination or exposure of the same local constraints, not independent conflicting enforcement.

**Admission evidence:** Host inputs/unknowns; provider inputs/unknowns; eliminated vs exposed equations; fixed allocation and reaction permissions; unit-owned balances; required local contribution type.

**Rejection or bounded-use rule:** Fixing arbitrary phase fractions while also requesting incompatible unrestricted equilibrium is overconstraint, not interoperability. A full phase solve cannot overwrite separate nonequilibrium bulk states.

**Positive witness:** A column owns interstage balances while a local contribution supplies phase enthalpies and compatible equilibrium relations.

**Negative/boundary witness:** The provider and host both independently prescribe the same phase split under different targets.

**Actions:** [AC-20](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-20), [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21), [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22), [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24), [AC-26](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-26), [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43). **Information:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50). **Basis:** B4:FR-STA-03, B4:FR-FLW-07, B4:FR-FLW-08, [E04](#e04).

<a id="ir-17"></a>
### IR-17. Freeze the realized function and distinguish alternative problems

**Accountability:** P08. **Applies when:** Provider selection, fallback, surrogate/tabulation, cache reuse or an upgrade could change evaluated values.

**Decision:** Pin the function definition, fixed data and realization conventions for a captured solve; state and explicitly declared parameter decision coordinates may vary under that definition. Record numerical attempt changes and require a distinct authorized problem for physical replacements.

**Admission evidence:** Realization identity, exact method/data bindings, cache dependency key, branch/numerical context and any approximation policy. A deliberately piecewise/composite function needs its own definition and solver qualification.

**Rejection or bounded-use rule:** Do not switch engines by region or by individual property merely to obtain successful callbacks. Rounded cache keys or table interpolation cannot masquerade as exact evaluation and derivatives.

**Positive witness:** A compatible warm start changes the numerical path without changing the prescribed function.

**Negative/boundary witness:** Function values use one EOS and gradients use another, or failure switches to ideal thermodynamics inside the same accepted objective.

**Actions:** [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27), [AC-28](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-28), [AC-29](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-29), [AC-30](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-30), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-34](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-34), [AC-47](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-47), [AC-54](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-54). **Information:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). **Basis:** B4:FR-RUN-05, B7:AR-11, B6:SI-21.

<a id="ir-18"></a>
### IR-18. Contain the full session sequence and uncertain native failure

**Accountability:** P08. **Applies when:** A backend has mutable contexts, module globals, opaque caches or uncertain failure behavior.

**Decision:** Choose a conservative, build-specific session policy covering activation, all input writes, evaluation, derivative/readback and cleanup; revoke publication independently of native termination.

**Admission evidence:** Tested reentrancy or whole-sequence serialization; process isolation when required by fatal-error risk; active-call lifetime; session quarantine/reconstruction rules; nested-callback/reentrancy policy.

**Rejection or bounded-use rule:** Several handles or internal parallelism do not prove independent cross-case safety. A caught exception cannot contain a fatal in-process crash. A cancelled but running session cannot be freed/reused.

**Positive witness:** An uncertain failed session is retired and the next request reconstructs from recorded data while accepted host state is retained.

**Negative/boundary witness:** Only the activation call is locked, allowing another model activation before the first property readback.

**Actions:** [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27), [AC-30](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-30), [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33), [AC-35](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-35), [AC-46](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-46). **Information:** [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70). **Basis:** B4:FR-RUN-04, B4:FR-RUN-07, B7:AR-08, [E12](#e12), [E13](#e13), B3:TP03.

<a id="ir-19"></a>
### IR-19. Report trial-domain errors without silently changing coordinates

**Accountability:** P08. **Applies when:** Iterative methods propose negative trial amounts, unavailable roots, invalid logarithm arguments or out-of-policy states.

**Decision:** When a solver trial is not evaluable, return a precise recoverable evaluation failure where the host contract supports it; do not clip, renormalize or project inputs without an explicit differentiated formulation.

**Admission evidence:** Physical domain vs numerical search bounds; known cause; recoverable callback semantics; any intentional coordinate transform and its derivatives; no mutation of original trial meaning.

**Rejection or bounded-use rule:** An interior numerical trial need not be accepted material, but unsupported evaluation is not permission to invent values. A hidden projection makes function/Jacobian inconsistent.

**Positive witness:** A host line search receives an evaluation failure and selects a new admissible point.

**Negative/boundary witness:** Negative trial fractions are clipped for function values while derivatives assume the original unconstrained values.

**Actions:** [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13), [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21), [AC-29](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-29), [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31), [AC-32](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-32), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33), [AC-35](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-35). **Information:** [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15), [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). **Basis:** B4:FR-STA-07, B4:FR-RES-01, [E02](#e02).

<a id="ir-20"></a>
### IR-20. Require complete original-problem checks and qualified optional work

**Accountability:** P09. **Applies when:** A provider returns success, partial values or failure after coupled primary/optional calculation.

**Decision:** Accept only the scope supported by original-target, conservation, authority and declared stability/derivative checks; protect trustworthy primary results from separable optional work.

**Admission evidence:** Original IC-41 criteria; correctly based residuals and scales; required/conditional output evaluation; actual used methods; independence of check evidence.

**Rejection or bounded-use rule:** Missing mandatory evidence is not pass. A failed shared native call with no trustworthy detached primary output cannot be reclassified as primary success just because the failing property was optional.

**Positive witness:** A phase split and enthalpy pass while separately requested optional viscosity is unavailable and explicitly labeled.

**Negative/boundary witness:** A small inner iteration step or a convergence flag replaces an original PH target check.

**Actions:** [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37), [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38), [AC-55](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-55), [AC-56](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-56). **Information:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). **Basis:** B4:FR-RES-02, B4:FR-RES-03, B7:AR-04, B7:AR-13.

<a id="ir-21"></a>
### IR-21. Preserve run-local coherence and guarded current publication

**Accountability:** P10. **Applies when:** Results feed a coupled solver, are cached, are received late or are proposed as current.

**Decision:** Reuse or publish results only under compatible actual input lineage, captured dependencies, completion scope and live run/ancestor authority; logical group adoption must not mix obsolete iterates.

**Admission evidence:** Exact or conservative dependency capture; run-local input lineage; required unit/flowsheet residuals; current target-slot generation; duplicate-command outcome and coherent publication group.

**Rejection or bounded-use rule:** Matching base package revision alone is insufficient. A fitting or replayed candidate cannot acquire production authority. A rejected group updates no completed-group current binding.

**Positive witness:** A locally accepted flash feeds another iterate but the unit remains unaccepted until its coupled residuals pass.

**Negative/boundary witness:** A stale same-revision stage or a child of a cancelled run overwrites the current unit result.

**Actions:** [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13), [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21), [AC-33](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-33), [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43), [AC-44](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-44), [AC-45](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-45), [AC-46](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-46), [AC-48](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-48), [AC-49](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-49), [AC-56](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-56). **Information:** [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70). **Basis:** B4:FR-RES-05, B7:AR-06, B7:AR-10.

<a id="ir-22"></a>
### IR-22. Archive exact reconstructable meaning and qualify actual dependencies

**Accountability:** P10. **Applies when:** A package is prepared for use, saved, restored, upgraded or shared.

**Decision:** Retain semantic definitions, construction recipe, selected data and the actual realization manifest separately from runtime state; treat dependency availability and distribution conditions as explicit unresolved prerequisites when not established.

**Admission evidence:** Exact engine/binding/data artifacts, optional modules and relevant build options; opaque-bundle attribution limits; semantic migration record; no current aliases in historical provenance.

**Rejection or bounded-use rule:** Public wrapper source does not supply an absent or separately required engine/database. Loading old values into new equations is comparison, not reproduction. Saving must not execute hidden fitting.

**Positive witness:** A missing-provider model remains inspectable while its affected operations and reproduction claims are blocked.

**Negative/boundary witness:** A saved session handle or new-default database is treated as the original model.

**Actions:** [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27), [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38), [AC-47](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-47), [AC-48](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-48), [AC-50](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-50), [AC-51](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-51), [AC-52](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-52), [AC-53](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-53). **Information:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69). **Basis:** B4:FR-GOV-03, B4:FR-LIF-04, B4:FR-LIF-07, B7:AR-16.

<a id="ir-23"></a>
### IR-23. Keep limited and nonbulk materials useful without invented physics

**Accountability:** P01. **Applies when:** Petroleum, carried solids, mass-only material or P3 extensions enter a flowsheet.

**Decision:** Qualify operations against the information actually justified by empirical, lumped, surface, distribution and inventory descriptions; numerical gaps remain explicit rather than forcing all materials into a molecular-fluid interface.

**Admission evidence:** Supported bases/conserved quantities; thermal/transport functions where known; distribution or loading denominator and reduction/transfer rules; account purpose; required extra data for each requested operation.

**Rejection or bounded-use rule:** No fictitious molecular weight, atoms, vapor pressure, unique distribution inverse or one-second flow conversion. Provider rejection does not close a P2 delivery gap.

**Positive witness:** A mass-only heater uses an explicit h(T) relation without requiring fugacity or molar flow.

**Negative/boundary witness:** A PVT pseudocomponent route is advertised as complete assay characterization and reactive energy support.

**Actions:** [AC-02](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-02), [AC-03](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-03), [AC-06](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-06), [AC-09](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-09), [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13), [AC-18](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-18), [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23), [AC-42](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-42), [AC-50](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-50). **Information:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-08](thermodynamics_semantic_information_dictionary_v0_1.md#ic-08), [IC-09](thermodynamics_semantic_information_dictionary_v0_1.md#ic-09), [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-32](thermodynamics_semantic_information_dictionary_v0_1.md#ic-32), [IC-33](thermodynamics_semantic_information_dictionary_v0_1.md#ic-33), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). **Basis:** B4:FR-MAT-02, B4:FR-EXT-01, B4:FR-EXT-02, B4:FR-EXT-03, B3:W08.

<a id="ir-24"></a>
### IR-24. Promote evidence only at the tested profile and scenario scope

**Accountability:** P09. **Applies when:** A candidate profile is proposed, checked, tested, reused for a new solver mode or compared across libraries.

**Decision:** Separate design admissibility, source/documentation support, executable adapter behavior, contract conformance, physical consistency and independent validation; release claims require evidence for the exact instantiated profile.

**Admission evidence:** Witness manifest, exact materials/data/operating points, predeclared numerical and scientific tolerances, independence, process-integration scope and unresolved gaps.

**Rejection or bounded-use rule:** Synthetic algebra, matching method names, a phase-table entry or agreement of shared models/data cannot establish real-fluid accuracy or all downstream units. Preserve P1/P2/P3 commitments.

**Positive witness:** One executed TP fixture adds only its bounded evidence; derivative/PH/reactive variants remain unassessed.

**Negative/boundary witness:** All scenario rows have candidates, so the whole simulator is labeled validated.

**Actions:** [AC-09](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-09), [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27), [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36), [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37), [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38), [AC-53](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-53), [AC-56](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-56). **Information:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69). **Basis:** B4:FR-GOV-01, B4:FR-GOV-05, B4:FR-RES-08, B7:AR-18.



## 4. Concrete composition decisions

A row describes one proposed composition and its missing proof obligations, not a blanket compatibility assertion for two projects. All rows remain unexecuted native profiles.

<a id="cm-01"></a>
### CM-01. DWSIM complete configured cubic package

**Disposition:** Admissible boundary for qualification.

**Admission contract:** Retain actual component data, cubic/mixing choices, caloric convention and explicit flash/fallback settings; adapt detached results and scoped state context.

**Do not:** Do not inherit mutable stream authority or alternate-model fail-safe as ordinary success.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03). **Validation route:** [IE-01](#ie-01), [IE-02](#ie-02), [IE-03](#ie-03), [IE-04](#ie-04), [IE-13](#ie-13), [IE-14](#ie-14), [IE-15](#ie-15), [IE-17](#ie-17). **Basis:** B2:WF-03, B2:WF-05, B2:WF-06.

<a id="cm-02"></a>
### CM-02. thermo explicit constants/correlations + compatible PR phases + eligible flash

**Disposition:** Admissible candidate assembly.

**Admission contract:** Freeze constants, correlations, ideal contributions, interactions and reference convention; select the flash family according to phase count.

**Do not:** A built phase alone does not qualify a complete inverse package or multicomponent solids.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03). **Validation route:** [IE-01](#ie-01), [IE-02](#ie-02), [IE-03](#ie-03), [IE-04](#ie-04), [IE-05](#ie-05), [IE-06](#ie-06), [IE-13](#ie-13), [IE-17](#ie-17). **Basis:** B3:L05, [E09](#e09), [E10](#e10).

<a id="cm-03"></a>
### CM-03. Clapeyron activity/fluid composite + explicit eligible flash

**Disposition:** Admissible candidate assembly.

**Admission contract:** Specify activity, gas, saturation/Henry and caloric components; consult exact model/algorithm/specification and initializer eligibility, not a moving default.

**Do not:** Do not select every generalized inverse or multiphase algorithm merely because the activity model constructs.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03). **Validation route:** [IE-01](#ie-01), [IE-02](#ie-02), [IE-03](#ie-03), [IE-04](#ie-04), [IE-06](#ie-06), [IE-08](#ie-08), [IE-09](#ie-09). **Basis:** B3:L03, [E11](#e11).

<a id="cm-04"></a>
### CM-04. Residual EOS properties + externally completed ideal/caloric model

**Disposition:** New composed package, conditional.

**Admission contract:** Reconcile basis/reference and construct complete H,S,U where required; the inverse solver and derivatives use those full functions; update package identity and tests.

**Do not:** Never append missing ideal heat after a residual-only PH solve or add ideal energy twice.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-02](#ie-02), [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08), [IE-16](#ie-16). **Basis:** B3:TP04, B4:FR-CFG-04.

<a id="cm-05"></a>
### CM-05. Phase allocation from engine A + unrelated enthalpy from engine B

**Disposition:** Blocked until explicitly qualified.

**Admission contract:** Define the resulting composite model, standard-state/phase/caloric consistency or deliberate approximation, and solve inverse targets using its actual energy function.

**Do not:** Value matching at one point does not prove identity or authorize call-by-call substitution.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03). **Validation route:** [IE-02](#ie-02), [IE-03](#ie-03), [IE-06](#ie-06), [IE-10](#ie-10), [IE-12](#ie-12). **Basis:** B3:W01, [E09](#e09).

<a id="cm-06"></a>
### CM-06. Named approximate caloric/phase assembly

**Disposition:** Admissible only as a declared approximation.

**Admission contract:** Identify relaxed consistency identities, intended use and original numerical checks; derivatives follow the implemented approximating function.

**Do not:** Do not require exact potential structure for all use, or falsely claim identities the approximate assembly relinquishes.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-02](#ie-02), [IE-06](#ie-06), [IE-07](#ie-07), [IE-09](#ie-09), [IE-18](#ie-18). **Basis:** [E08](#e08), B4:FD-07.

<a id="cm-07"></a>
### CM-07. Supplemental liquid transport after a coherent flash/chemical solve

**Disposition:** Conditional non-state-changing augmentation.

**Admission contract:** Evaluate the supplied phase/composition on an eligible correlation with solvent basis, domain and diffusion/interfacial convention; include state dependence when needed by coupled solver.

**Do not:** No reflash/respeciation or unqualified pure-solvent substitution; transfer coefficients remain equipment closures.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-05](#ie-05), [IE-06](#ie-06), [IE-11](#ie-11), [IE-14](#ie-14). **Basis:** B4:FR-PRP-03, B4:FR-FLW-08.

<a id="cm-08"></a>
### CM-08. CoolProp water/steam on one exchanger side; general mixture package on the other

**Disposition:** Admissible heat-only composition.

**Admission contract:** Use each side’s coherent energy differences, correct phase behavior and the host heat balance; no shared material map across the wall.

**Do not:** Do not equate absolute enthalpies or move species across the wall to reconcile energy.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-12](#ie-12), [IE-15](#ie-15). **Basis:** B2:WF-08, B3:W06.

<a id="cm-09"></a>
### CM-09. Same material moves between different property packages

**Disposition:** Conditional directed translation.

**Admission contract:** Declare constituent map, reference transformation, preserved independent targets, resulting model discrepancy and target solver.

**Do not:** Do not simultaneously preserve inconsistent T,P,z,H through hidden heat or call a model switch a unit conversion.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-12](#ie-12), [IE-17](#ie-17). **Basis:** [E07](#e07), B6:IC-51.

<a id="cm-10"></a>
### CM-10. Detailed material is lumped and subsequently reconstructed

**Disposition:** Forward map conditional; inverse generally nonunique.

**Admission contract:** Define conservation and information loss; reverse calculation requires retained detail or explicitly new assumptions and its own result lineage.

**Do not:** Do not invent a unique detailed composition from aggregate mass alone.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-12](#ie-12), [IE-16](#ie-16), [IE-17](#ie-17). **Basis:** B4:FR-FLW-06, B6:IC-07.

<a id="cm-11"></a>
### CM-11. Reaktoro coherent chemical system in a flowsheet block

**Disposition:** Admissible candidate chemical boundary.

**Admission contract:** Keep species/activity/standard thermodynamics and constraints coherent; map reference amounts to process flows/inventories; qualify energy and actual exposed sensitivities.

**Do not:** Do not inherit upstream functionality through a narrower adapter or detach species from their energy conventions.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03). **Validation route:** [IE-01](#ie-01), [IE-05](#ie-05), [IE-06](#ie-06), [IE-07](#ie-07), [IE-10](#ie-10), [IE-13](#ie-13), [IE-17](#ie-17). **Basis:** B3:L08, [E13](#e13).

<a id="cm-12"></a>
### CM-12. Independent speciation followed by an unrelated fluid flash

**Disposition:** Blocked as a claim of fully coupled equilibrium.

**Admission contract:** Permit only a defined approximation or iterative coupled procedure that preserves compatible methods and passes final combined chemistry, phase and energy checks.

**Do not:** Two successful standalone calls do not certify the final coupled material.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-10](#ie-10), [IE-12](#ie-12), [IE-15](#ie-15). **Basis:** B4:FR-CHM-08, B7:AC-24.

<a id="cm-13"></a>
### CM-13. Value-only flash wrapped by automatic differentiation

**Disposition:** Not derivative-qualified by wrapping.

**Admission contract:** Qualify a total-response derivative implementation separately; use original inner equations/sensitivities or full re-solve finite differences where admissible.

**Do not:** AD on surrounding host arithmetic cannot infer derivatives through an opaque native solve.

**Possible solver arrangements after qualification:** [SM-03](#sm-03). **Validation route:** [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08). **Basis:** B4:FR-PRP-05, [E04](#e04).

<a id="cm-14"></a>
### CM-14. Two different engines supply value and derivative of a nominally same method

**Disposition:** Blocked until equivalence is established.

**Admission contract:** Match equations, parameters, references, density roots, derivative chart/order and active branch over the required domain, or use one coherent realized function.

**Do not:** Do not infer derivative equality from same PR/NRTL name or agreement at a single test point.

**Possible solver arrangements after qualification:** [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-01](#ie-01), [IE-06](#ie-06), [IE-08](#ie-08), [IE-17](#ie-17). **Basis:** B6:IC-39, B3:W03.

<a id="cm-15"></a>
### CM-15. Rate-based unit with independently supplied bulk phases and interface properties

**Disposition:** Admissible formulation contract; full implementation remains open.

**Admission contract:** Host owns transfer laws, area, boundary conditions and separate local temperatures/compositions; properties and interface equilibrium evaluated only under assigned authority.

**Do not:** An efficiency-corrected equilibrium-stage model cannot claim full nonequilibrium coverage by label.

**Possible solver arrangements after qualification:** [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-05](#ie-05), [IE-06](#ie-06), [IE-08](#ie-08), [IE-09](#ie-09), [IE-10](#ie-10), [IE-11](#ie-11), [IE-15](#ie-15). **Basis:** B2:WF-10, B4:FR-FLW-08.

<a id="cm-16"></a>
### CM-16. Empirical mass-only solid plus ordinary fluid package

**Disposition:** Admissible limited material composite.

**Admission contract:** Carry disjoint amounts and justified calorics; name effective-property/routing rules; reserve chemical potentials and atom balances for information actually supplied.

**Do not:** No fake molecular weight or vapor pressure to fit the ordinary fluid interface.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-02](#ie-02), [IE-11](#ie-11), [IE-16](#ie-16), [IE-18](#ie-18). **Basis:** B4:FR-MAT-02, B3:W08.

<a id="cm-17"></a>
### CM-17. CAPE-OPEN adapter around any listed engine

**Disposition:** Conditional interoperability boundary.

**Admission contract:** Map operation postconditions, material/slate conventions and lifecycle; qualification remains specific to the implemented adapter and installed dependencies.

**Do not:** Interface compliance does not certify cross-engine thermodynamic equivalence, new reaction-interface support or numerical breadth.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03). **Validation route:** [IE-01](#ie-01), [IE-03](#ie-03), [IE-05](#ie-05), [IE-12](#ie-12), [IE-13](#ie-13), [IE-17](#ie-17). **Basis:** B3:L01, B2:WF-15.

<a id="cm-18"></a>
### CM-18. Temperature-dependent automatic provider switching or tabulation

**Disposition:** New declared composite/surrogate, not transparent fallback.

**Admission contract:** Specify the actual switching/interpolation function, approximation/error domain, continuity/derivatives and parameter provenance; qualify the chosen solver mode separately.

**Do not:** An unnoticed jump or rounded cache returning a derivative from another point cannot enter an exact smooth callback.

**Possible solver arrangements after qualification:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05). **Validation route:** [IE-01](#ie-01), [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08), [IE-09](#ie-09), [IE-14](#ie-14), [IE-17](#ie-17). **Basis:** B4:FR-CFG-05, B6:IC-72, [E01](#e01).



## 5. Solver-integration contracts in detail

The solver arrangements must expose the same material, reference and authority semantics, but they do not need the same internal algorithms. A mathematical contribution is not a permission for the property subsystem to own equipment geometry, interstage balances or plant targets.

<a id="sm-01"></a>
### SM-01. Direct solved-state service

**Unknown ownership:** Provider solves requested local state; host unit/process model owns its external balances and any recycle.

**Contribution contract:** Return a complete or explicitly partial candidate for the exact state/chemistry contract; outer host need not request derivatives for plain sequential evaluation.

**Derivative demand:** Only those explicitly demanded by an outer algorithm. No automatic optimization-ready claim.

**Phase/branch treatment:** Provider handles permitted search with scope and branch evidence; caller must still qualify phase support and stability.

**Use and candidate boundaries:** Heaters, valves, flash drums, utility states and coherent reactive equilibrium blocks in a sequential or hierarchical flowsheet. Candidates or references: DWSIM; thermo; Clapeyron; ThermoPack; CoolProp; FeOS; Reaktoro.

**Gate:** A robust direct solve is not automatically a deterministic smooth callback over all solver trial points. **Governing rules:** [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-14](#ir-14), [IR-15](#ir-15), [IR-18](#ir-18), [IR-20](#ir-20), [IR-21](#ir-21).

<a id="sm-02"></a>
### SM-02. Supplied-phase constitutive callback

**Unknown ownership:** Host fixes/supplies local coordinates; provider may resolve only expressly permitted dependent coordinates such as density/root.

**Contribution contract:** Evaluate h, fugacity/activity, transport or other named quantities at caller-owned state without independent bulk redistribution.

**Derivative demand:** Derivatives of these actual local functions in host coordinates; numerical derivatives only when explicitly eligible.

**Phase/branch treatment:** Branch and phase ownership fixed by caller; inactive/incipient evaluations qualified separately.

**Use and candidate boundaries:** Column phase properties, rate-based bulk/interface states, rate-law inputs and host residual construction. Candidates or references: IDAES-style contributions; thermo phases; Clapeyron properties; ThermoPack properties; CoolProp backend properties; FeOS states; Reaktoro properties; qualified DWSIM phase calls.

**Gate:** Cannot secretly flash or chemically equilibrate to make an out-of-domain supplied state evaluable. **Governing rules:** [IR-04](#ir-04), [IR-07](#ir-07), [IR-11](#ir-11), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19).

<a id="sm-03"></a>
### SM-03. Condensed solved block

**Unknown ownership:** Provider eliminates internal y from g(u,y)=0; host solves remaining process residuals in u.

**Contribution contract:** Return the solved observable and a qualified total response through all eliminated state/phase/chemistry variables.

**Derivative demand:** Implicit response when local regularity and needed inner Jacobians exist; otherwise qualified full-resolve perturbation. Hessians need total second response or a compatible outer approximation.

**Phase/branch treatment:** One regular branch per local smooth contract, or explicit event/restart/regularized formulation. Phase transitions are not hidden.

**Use and candidate boundaries:** Embedding expensive flashes or chemical equilibria into a coupled process solve while reusing their native solvers. Candidates or references: Qualified whole-package callbacks; FeOS/Clapeyron solved sensitivities where exposed; Reaktoro equilibrium sensitivities where exposed; Pyomo reduced-model pattern as reference.

**Gate:** Reject frozen-phase partial derivatives as total response; quantify inner residual amplification and repeatability. **Governing rules:** [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-16](#ir-16), [IR-17](#ir-17), [IR-20](#ir-20).

<a id="sm-04"></a>
### SM-04. Exposed local residual block

**Unknown ownership:** Host includes local phase/chemical variables and solves their residuals with unit equations.

**Contribution contract:** Provider supplies local residual evaluations and coordinate/bounds/derivative meaning without independently fixing the same unknowns.

**Derivative demand:** Residual Jacobian and, when demanded, multiplier-weighted second derivatives or qualified approximation.

**Phase/branch treatment:** Structural candidate superset or outer rebuild strategy; fixed variable/constraint layout for each solver invocation.

**Use and candidate boundaries:** Strongly coupled stages, reactive separation and formulations for which hiding equilibrium would produce difficult nested solves. Candidates or references: Qualified residual providers; IDAES-style property/reaction formulations; Host assembly from phase functions.

**Gate:** A library exporting only a flash result does not expose residuals by inference; host must implement and validate any missing equations. **Governing rules:** [IR-02](#ir-02), [IR-08](#ir-08), [IR-11](#ir-11), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-16](#ir-16), [IR-19](#ir-19).

<a id="sm-05"></a>
### SM-05. Explicit equation contribution

**Unknown ownership:** Host compiler/formulation owns declared algebraic variables and constraints; provider contributes physical relationships.

**Contribution contract:** Export supported equations/expressions or equivalent qualified mathematical content with units, references and structural dependencies.

**Derivative demand:** Host symbolic/automatic differentiation only through represented expressions and explicit qualified external derivative contracts.

**Phase/branch treatment:** Declared fixed structural phases plus tested phase-presence formulation or outer restructuring; auxiliary equilibrium coordinates remain distinct from actual state.

**Use and candidate boundaries:** Native equation-oriented process compilation, parameter estimation, scaling and sparse solver integration. Candidates or references: IDAES as architecture reference; Future host/native thermodynamic models; Explicitly exposing libraries where verified.

**Gate:** No automatic extraction of symbolic equations from arbitrary compiled engines; opaque callbacks remain separately qualified contributions. **Governing rules:** [IR-02](#ir-02), [IR-03](#ir-03), [IR-11](#ir-11), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-16](#ir-16), [IR-22](#ir-22).



### 5.1 A condensed block must return the solved response

Let the host supply variables \(u\). An inner thermodynamic solve determines local unknowns \(y\) from

\[
g(u,y)=0.
\]

The host evaluates a residual or objective contribution

\[
F(u)=r(u,y(u)).
\]

On a locally differentiable branch where \(g_y\) is square and nonsingular, differentiation gives

\[
g_y\frac{dy}{du}=-g_u,
\qquad
\frac{dF}{du}=r_u-r_y g_y^{-1}g_u.
\]

The notation shows the relation; an implementation should solve the corresponding linear systems rather than construct a dense inverse merely to follow the notation. A full exposed block with Jacobian blocks \(r_u,r_y,g_u,g_y\) has the same local condensed derivative through its Schur complement under these assumptions.

The host must not substitute \(r_u\), which holds \(y\) fixed, for the total derivative. Eliminated quantities may include density roots, phase fractions, phase compositions, reaction extents and chemical species. All responses authorized by the original problem matter.

Pyomo's documented external reduced-model formulation is a useful precedent for this separation; it is not a decision to require Pyomo in the eventual implementation. [E04]

**Authored example:** take \(g=y^2-u\), the positive branch \(y=\sqrt{u}\), and \(r=y^3+u\). At \(u=4,y=2\),

\[
\frac{dy}{du}=\frac14,
\qquad
F'(u)=4,
\qquad
F''(u)=\frac38.
\]

The frozen-\(y\) derivative is only 1. It is the wrong derivative for the reduced function even though it is a legitimate partial derivative of the uncondensed expression.

At a singular \(g_y\), a phase switch or a nonunique branch, this local formula does not establish a smooth response. A constrained chemical minimization can require differentiating a qualified active-set/KKT system instead of an unconstrained species residual. Regularity and active-set assumptions then form part of the profile. No generic inverse formula certifies arbitrary phase transitions.

### 5.2 Opaque internals do not require invented derivatives

A provider may expose a solved sensitivity directly, expose inner residual/Jacobian information, or expose only values. These are different usable surfaces.

A value-only engine can still be reused for a direct operation. For a coupled mode, possible routes include a qualified full-re-solve finite difference, an explicit local approximation/surrogate with a separate model identity, or a host formulation that does not require the missing derivative. The selected route must be disclosed and tested. It cannot be described as analytical or implicit differentiation merely because the outside wrapper uses automatic differentiation.

Automatic differentiation of surrounding arithmetic sees only represented operations and supplied external derivative rules. It does not infer the solution map hidden in an arbitrary native function. Likewise, a numerical algorithm that internally needs second derivatives of an EOS need not expose a Hessian of its final solved outputs. The Clapeyron algorithm table is useful for algorithm eligibility, but its internal derivative column must not be promoted to a universal external sensitivity contract. [E11; B4:FR-PRP-05/06]

### 5.3 Inner convergence is part of outer accuracy

Suppose an approximate inner state \(\tilde y\) has residual \(\epsilon=g(u,\tilde y)\). In the same locally regular neighborhood,

\[
\tilde y-y^*\ \approx\ g_y^{-1}\epsilon,
\qquad
r(u,\tilde y)-r(u,y^*)\ \approx\ r_y g_y^{-1}\epsilon.
\]

These are first-order local estimates, not unconditional error bounds. The sign changes if the expression is written as the correction to the approximate state rather than its error. The design consequence is that a universal inner residual threshold is not enough: the effect of the inner error on the outer observables and derivatives must be bounded or empirically qualified for the intended profile.

For the authored example above, the local magnitude multiplier is \(r_y/g_y=3\). An inner residual magnitude of \(10^{-6}\) can therefore contribute approximately \(3\times10^{-6}\) to the outer value at that point, under the linearization assumptions.

For a central difference with step \(\delta\), if each computed function value has absolute error at most \(\eta\), the value-error contribution to the derivative can be as large as \(\eta/|\delta|\). Shrinking the step indefinitely while retaining the same inner solve noise can make the derivative worse. Truncation error, machine precision, branch changes and solver noise all require consideration; no universal step or inner tolerance is selected here.

The profile must therefore state how inner tolerances are chosen relative to demanded outer residual/derivative accuracy, what conditioning or repeatability evidence is available, and what happens when the desired accuracy is not attainable. Failed qualification may restrict the mode to direct use, require stronger initialization, expose the residual block, or reject that operation. It must not silently relax the original physical acceptance condition. [IR-12; B7:AC-29/31/36]

### 5.4 A solver may need more than a property Hessian

For a smooth constrained optimizer, a second-order contribution can mean a Hessian of the **Lagrangian**,

\[
\nabla^2\mathcal L=\sigma\nabla^2 f+\sum_j\lambda_j\nabla^2 c_j,
\]

not an arbitrary Hessian of enthalpy. Objective factors, constraint multipliers, index ordering and scaling must be mapped correctly. Ipopt's standard interface illustrates this requirement and requires fixed structural nonzero patterns through one solve. Pyomo's grey-box documentation likewise treats multiplier-weighted second-derivative contributions explicitly. [E02,E05]

Three operational contracts are valid when actually supported:

| Derivative route | Required qualification | What is not granted |
| --- | --- | --- |
| Full first and required second derivatives | Actual function/chart/branch, correct weights and full second chain rule | An arbitrary local-phase Hessian does not become a solved-state Hessian. |
| First derivatives plus a solver-approved second-order approximation | Accurate required gradients/Jacobian and explicit solver approximation | Missing first derivatives or nonsmooth functions are not repaired automatically. |
| Qualified numerical derivatives | Repeated evaluability, controlled noise, branch behavior, perturbation/chart policy and verification | No analytical/automatic derivative label or arbitrary phase-boundary guarantee. |

Ipopt documents a limited-memory approximation that can avoid an exact Hessian callback. Its smooth-NLP assumptions remain relevant; this option does not make every value-only flash suitable for an unconstrained range of optimizer trial points. The design preserves a solver-class contract rather than choosing Ipopt as the mandatory final solver. [E01,E03]

### 5.5 Coordinate and reference transformations are differentiated too

For nonlinear coordinates \(x=x(z)\), the second derivative of a scalar \(f\) is

\[
\nabla_z^2 f
=J^T\nabla_x^2 f J
+\sum_i\frac{\partial f}{\partial x_i}\nabla_z^2 x_i.
\]

Omitting the second term is wrong in general. In the authored example \(x=z^2\), \(f=x^2\), at \(z=2\), the first term is 32 and the missing coordinate-curvature term is 16. The correct Hessian is 48.

A composition chart, a mass/molar transformation using mixture molecular weight, a temperature-dependent parameter formula, and a composition-dependent reference transformation can all contribute chain-rule terms. A simple permutation requires corresponding axis permutations; it is not permission to omit nonlinear transformations elsewhere.

For scaling \(x=x_0+S_x\hat x\) and \(\hat r=S_r^{-1}r\),

\[
\frac{\partial\hat r}{\partial\hat x}=S_r^{-1}J S_x.
\]

The physical residual and its original units must remain recoverable. Numerical scaling must not redefine the accepted physical constraint or its error budget. A zero Jacobian value at one state is not proof of structural independence: \(r=x y\) has zero gradient at the origin, but not elsewhere.

### 5.6 Unknown ownership and sparsity must be explicit

In SM-03, the inner solver owns the eliminated variables; the host receives their qualified response. In SM-04/05, those variables and their defining equations become part of the host formulation. Mixing both arrangements without a deliberate formulation can duplicate or conflict with constraints.

A host may, for example, fix total composition and pressure and solve phase allocation. It must not simultaneously fix an arbitrary split and request a different unrestricted equilibrium. An observed temperature retained as a check is not an extra equation. The host's global equation solver remains responsible for the combined system's structure and solvability; a parameter count or property-block construction alone is not a rank/feasibility proof. [B6:IC-36/37/40; B7:AC-21/26/43]

## 6. Phase transitions, stability and solver regularity

### 6.1 Choose a phase strategy, not an implicit hope of smoothness

| Strategy | What is fixed or varied | Qualification burden |
| --- | --- | --- |
| Local phase/root branch | Evaluate a declared homogeneous or fixed-allocation branch | Branch admissibility, local derivatives and out-of-domain behavior; no unrestricted stability inference. |
| Condensed whole equilibrium solve | Provider can choose permitted phases internally; host sees a solved map | Total response on regular regions, deterministic branch policy, transition detection and the solver's treatment of nonsmooth points. |
| Fixed structural candidate superset | Host allocates a stable variable/constraint layout containing allowed phase slots | A correct phase-presence formulation, inactive variables/regularity/scaling, physically meaningful zero-amount and incipient outputs. |
| Outer event/restructure/restart | Host changes active structure between, not secretly during, solver invocations | Preserved original physical scope, branch/seed mapping and explicit restart/acceptance evidence. |
| Qualified smoothed or regularized formulation | Smooth auxiliary equations approximate an original relationship | Recorded regularization, auxiliary-versus-actual states, and declared physical residual/approximation limits. |

These are alternatives with different tradeoffs, not a promise that every library supports all of them. A fixed supersystem is not automatically well-conditioned; absent-phase variables can introduce degeneracy. A continuation seed is not a present phase. If an outer algorithm expects a fixed layout, native output ordering cannot redefine that layout mid-iteration.

IDAES's documented smooth vapor-liquid treatment illustrates formulation-specific auxiliary equilibrium temperatures and smoothing. The useful lesson is to distinguish the auxiliary variable from actual stream temperature and record the approximation/control parameters. It is not evidence that a generic smooth-NLP wrapper solves every multiphase or electrolyte problem. [E06]

### 6.2 Pure-fluid saturation is a specific inverse problem

At coexistence, a pure-fluid TP pair does not by itself determine the fraction allocated to liquid and vapor. A TP API choosing one endpoint can yield a discontinuous enthalpy-versus-temperature return path. Wrapping that path in a scalar temperature root finder is not automatically a general PH flash.

The authored test uses an intentionally simple return function: enthalpy 0 below a threshold and 100 above or at it. For a target of 50, bracket endpoints have opposite residual signs, but the return function has no root. The final target check must reject the apparent numerical solution. A separately specified phase-mixture representation with equal amounts of the declared endpoints can represent the target; this is a different, explicitly supported allocation calculation.

Accordingly, a PH or PS profile must state whether homogeneous inversion, coexistence allocation, multiphase inverse resolution and branch selection are internally supported or supplied by the host. A temperature search is admissible only over a function and domain for which it actually resolves the required state. [B4:FR-EQL-02/04; B6:SI-10; IR-14]

### 6.3 Phase ordering, physical presence and stability remain independent

All outputs are normalized into snapshot-local phase/domain descriptions. Physical phase count follows amounts and qualified presence meaning, not native container size. Correspondence across snapshots may be a merge, split or ambiguity. Product routing uses unit-owned intent, not library liquid1/liquid2 labels.

A convergence result for an admitted phase set is not necessarily a global-stability certificate over every physically allowed candidate. Record what was declared, what the algorithm examined, what checks ran and what remains unassessed. A branch-constrained result remains useful under that restriction; it cannot be relabeled unrestricted merely because the solver converged.

### 6.4 Trial points and hidden repairs

Coupled solvers may propose trial coordinates outside an evaluator's domain. A profile must distinguish invalid accepted material from an unevaluable numerical trial. Where the solver contract permits, the evaluator reports failure so the host can backtrack or choose another point. It must not silently clip negative fractions, renormalize a trial composition, select a different physical model or return a leftover value.

Explicit variable transformations, safeguarded evaluation, finite regularizations and surrogate extrapolation can be chosen deliberately. Their actual functions, domains and derivatives must then be part of the formulation. A hidden projection used only for values but not derivatives creates a different mathematical problem. [IR-19; B7:AC-32/35]

## 7. Physical composition and flowsheet boundary agreements

### 7.1 Gamma–phi assembly needs more than an activity coefficient

For a declared conventional fugacity convention, a vapor term might be expressed as \(f_i^V=y_i\phi_i P\), while a liquid term has the form \(f_i^L=x_i\gamma_i f_i^\circ(T,P)\). This schematic representation does not specify the entire model: \(f_i^\circ\), saturation or Henry convention, pressure corrections, vapor treatment and the associated data still have to be selected coherently.

Solvent and dilute-solute standards need not be the same. Activity parameters fitted to one convention cannot be substituted into another by name alone. Supporting pure-component behavior and energy functions must refer to the same intended material and admitted domains.

The selected caloric model may use an ideal/residual or ideal/excess decomposition, empirical correlations, or a bundled provider formula. No single decomposition is compulsory. What is compulsory is a clear interpretation of the complete function used for the requested energy/entropy calculation and its relationship to the phase model. [B3:W01; IR-02/03/05]

### 7.2 Do not turn consistency requirements into a fidelity bottleneck

The researched `thermo` documentation explicitly distinguishes caloric and equilibrium choices and discusses a commonly used caloric approximation that is not fully thermodynamically consistent. That is a reason to expose a deliberate engineering choice, not to forbid every package outside a fundamental potential formulation. [E08]

For each approximation, state which operations remain intended, which consistency identities are not claimed, the actual h/s/rate/transport functions used in balances, and what numerical derivatives mean. Acceptance checks remain tied to the stated approximation and its original process constraints. Independent validation can still compare its predictions to appropriate data; the result must retain the approximate configuration identity.

For example, if an enthalpy correlation and a separately supplied Cp do not satisfy Cp=dh/dT under the requested fixed conditions, the solver callback derivative must be the derivative of the implemented enthalpy function or the discrepancy must block that derivative claim. Returning an unrelated Cp as its derivative is not an acceptable way to preserve a familiar property label.

### 7.3 Completing partial calorics changes the whole inverse calculation

The authored example deliberately uses easy arithmetic:

\[
h_1(T)=2(T-300),\qquad h_2(T)=3(T-300),
\]

in kJ/kg. For a total target of 50, a solve using only \(h_1\) gives 325 K. Adding \(h_2\) afterward produces 125 kJ/kg at that temperature, not the target. Solving the full function \(h_1+h_2\) gives 310 K and meets 50 kJ/kg.

A proposed external ideal-caloric supplement to a residual EOS must therefore do one of the following: enter a provider-supported complete-property/inverse interface, or enter a host inverse formulation using complete properties throughout. It creates a separately qualified composed package. The original native partial inverse cannot be treated as complete merely because the final report has more fields.

The earlier ThermoPack pseudocomponent observation motivates this rule but does not establish that every current pseudocomponent path has the same limitation or that the proposed host completion already exists. [B3:TP04; IP-07; IR-05]

### 7.4 Reference offsets, chemical standards and reaction energy

For an explicitly established energy bookkeeping transformation

\[
H_B=H_A+c^T n,
\]

a balanced nonreactive component transfer can cancel constant component offsets across its boundary. For a reaction \(\Delta n=\nu\xi\), the reference contribution changes by \(c^T\nu\xi\). A separate correction-based energy formulation must account for that change with the corresponding opposite correction if it is to represent the same physical energy balance. It must not add the full heat of reaction again to already formation-inclusive enthalpy.

This bookkeeping does not make arbitrary chemical-potential shifts innocuous. For a constrained chemical minimization with conserved basis \(A n=b\), a shift \(\mu' = \mu+A^T\lambda\), for constant \(\lambda\) in the stated chemical state problem, changes total Gibbs energy by the constant \(\lambda^T b\). It leaves the minimizing composition unchanged under those constraints. A general component shift \(c\) need not have this form. In reaction coordinates, \(\nu^T c\neq0\) can change the reaction driving-force convention unless the rest of the chemical formulation is transformed consistently.

These are algebraic qualifications of a declared transformation, not authorization to alter real chemical data arbitrarily. Formation enthalpy references, entropy references, activity standard states and equilibrium constants must each be reconciled as required by the selected physical equations. A single user-facing reference label cannot hide those differences. [B6:IC-21; B3:CO03; IR-09]

### 7.5 Three boundary agreements

| Boundary | Quantities/actions governed | Explicitly not implied |
| --- | --- | --- |
| Heat-only link | Side-specific energy differences, real heat/loss/work convention, coherent coupled acceptance | Material transfer, shared composition or equal absolute energy zeroes |
| Representation link | Directed map, conserved basis, altered reporting/species coordinates, information loss and inverse prerequisites | New physical inventory, real material exchange or unique reverse reconstruction |
| Material/package transition | Source account, representation and known reference map, jointly chosen preserved targets, target-state calculation and model discrepancy | Simultaneous preservation of incompatible predictions or hidden corrective heat |

The source and target can each be internally qualified and still disagree. A translator needs its own directed qualification. IDAES's translator documentation makes application-specific constraints explicit; neither constructing the interface nor sharing a standard supplies a universal conversion. [E07]

The retained synthetic comparison uses \(h_A=T-300\) and \(h_B=1.2(T-300)+100\). At 350 K, subtracting the known 100 kJ/kg offset from B leaves 60 kJ/kg versus A's 50. A TP-preserving link reports the remaining 10 kJ/kg model discrepancy. A corrected-PH-preserving link instead obtains approximately 341.67 K in B. Neither choice is automatically the uniquely correct plant representation; the model author selects the intended boundary contract.

### 7.6 Cross-library verification cannot silently change the question

A second library can be used for an independent-model comparison only if the report states the model/data/reference differences. It is not an exact numerical oracle merely because both method menus say PR, NRTL or IAPWS.

A re-evaluation using the same model can be useful for detecting wrong roots, target residuals, coordinate errors or inconsistent returned fields. It is not independent validation of the model's physical accuracy. A property-only check must preserve the candidate's protected state; reflashing it and comparing the resulting different state can conceal the very error being checked. [B7:AC-36/38/55; IR-20/24]


## 8. Provisional scenario-to-provider profiles

These are **screened profile templates**, not instantiated production configurations. Each specifies the physical composition, candidate reuse boundary, intended solver mode, missing data, initializer and derivative contract, first witness and stopping conditions. No template has an assigned validated numerical realization in this work.

### 8.1 Initial qualification order

For ordinary nonreacting fluid packages, the first comparative route is **DWSIM as a complete configured package versus an explicitly assembled `thermo` configuration**, with **Clapeyron** as a further composition/algorithm candidate. This is an evaluation order based on the B2/B3 workflow evidence, not a numerical ranking. It prioritizes coherent conventional functionality rather than rebuilding a flash around an isolated EOS.

For utilities and pure refrigeration, start with a **specific CoolProp backend** and compare only explicitly matching formulations. For chemical equilibrium and aqueous/mineral systems, start from a **coherent Reaktoro chemical system**; a configured **IDAES-style electrolyte/reaction formulation** is a separate alternative. For directly exposed local properties and EOS calculations, keep **ThermoPack and FeOS**, as well as eligible surfaces from the other libraries, available for focused qualification. Their native implementation language does not determine the domain model.

For coupled equations, use **IDAES and the explicit/reduced external-model patterns as architecture references**, not a requirement that the production host call Pyomo. Data/correlation functions from `chemicals` can help preparation or selected properties, but do not by themselves supply a coherent phase/energy problem. **CAPE-OPEN** remains an interoperability contract reference and a possible adapter boundary, not a numerical provider or compatibility certificate. [B3 §§2–9; E04,E05,E07]

The petroleum, black-oil, empirical-material and full reactive/nonequilibrium contacting cases retain explicit host and provider-completion work. A list of extensible classes does not close them. Selecting an exact parameter/data manifest is a prerequisite to the corresponding native experiment, not something the report silently fills with defaults.

| Template | Target | Initial qualification candidates |
| --- | --- | --- |
| [IP-01](#ip-01) | Conventional nonreacting hydrocarbon cubic package | First comparison: DWSIM whole configured package versus thermo explicit phase/flash assembly; Alternative: Clapeyron configured cubic plus selected algorithm; Focused EOS alternative: ThermoPack after interface and session qualification |
| [IP-02](#ip-02) | Nonideal liquid gamma–phi package | First comparison: DWSIM NRTL package versus thermo GibbsExcessLiquid with compatible gas/flash; Alternative: Clapeyron activity/fluid composite; Equation alternative: configured IDAES property/reaction formulation |
| [IP-03](#ip-03) | Multiple-liquid and vapor–multiple-liquid package | DWSIM configured multiphase route; thermo FlashVLN for its supported fluid scope; Clapeyron explicitly selected multiphase TP route; Solids or reactive variants use IP-11/12/13 instead of assuming generic extension |
| [IP-04](#ip-04) | Pure water and steam utility package | Initial utility route: selected CoolProp water backend; Comparison: relevant IDAES, thermo or DWSIM water package with explicit formulation |
| [IP-05](#ip-05) | Pure-fluid refrigeration package | Initial candidate: exact CoolProp pure-fluid backend; Alternative: qualified whole-provider pure-fluid route in DWSIM, thermo or Clapeyron |
| [IP-06](#ip-06) | True-mixture refrigerant package | Compare exposed mixture routes in thermo, Clapeyron, ThermoPack and a selected CoolProp backend; DWSIM whole configured mixture route remains a candidate |
| [IP-07](#ip-07) | Assay-characterized compositional petroleum | First workflow investigation: DWSIM characterization plus its configured package; Alternative custom-data route: host characterization with thermo or Clapeyron; ThermoPack cubic pseudocomponent route only with explicitly completed and qualified calorics |
| [IP-08](#ip-08) | Reduced black-oil or bulk petroleum representation | DWSIM dedicated reduced-model route is the first investigation target; No complete alternative workflow established in B3; retain a visible coverage gap |
| [IP-09](#ip-09) | Mass-based empirical material and carried inert solids | Host semantic/limited-property implementation using justified supplied correlations; Any listed data/phase library may supply a qualified component correlation; no complete existing route is presumed |
| [IP-10](#ip-10) | Specified conversion and finite-rate chemistry with a compatible property package | Host conversion or rate contribution + IP-01/02/09 property service as appropriate; IDAES-style reaction/property formulation; Reaktoro kinetic subset only when its exposed meaning matches the unit request |
| [IP-11](#ip-11) | Coherent chemical equilibrium, aqueous speciation and mineral phases | Initial chemical-system candidate: Reaktoro upstream qualified interface; DWSIM Reaktoro adapter assessed as its own narrower realization; IDAES alternative treated as IP-12, not interchangeable data by default |
| [IP-12](#ip-12) | Equation-oriented electrolyte/reaction alternative | IDAES physical-parameter/state/reaction construction as initial formulation reference; Equivalent host equations only after method/data/derivative qualification |
| [IP-13](#ip-13) | Staged, reactive and full nonequilibrium contacting | Host coupled formulation + thermo/Clapeyron or other eligible phase functions; IDAES-style local property/reaction structure; Qualified Reaktoro subblocks where authority and total responses fit the formulation |
| [IP-14](#ip-14) | Deferred surface, distribution, inventory and restricted-state extensions | Use existing P01/P04/P05/P06 contracts for independent representability review; Potential future numerical contributions from FeOS surfaces, Reaktoro or ThermoPack inventory operations, or provider-specific restricted phases only after exact qualification |


### 8.2 Concrete profile cards

<a id="ip-01"></a>
#### IP-01. Conventional nonreacting hydrocarbon cubic package

**Scope connections:** SC-01, SC-02, SC-03, SC-04, SC-05, SC-06, SC-07. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** A specific PR or SRK configuration, not a generic cubic switch: fixed pure-component data, alpha/mixing rules and interactions, compatible ideal/residual calorics and explicit vapor/liquid search. PR and SRK are separate variants.

**Candidate realization route:** First comparison: DWSIM whole configured package versus thermo explicit phase/flash assembly; Alternative: Clapeyron configured cubic plus selected algorithm; Focused EOS alternative: ThermoPack after interface and session qualification.

**Reuse boundary:** RB-03 by default; RB-02 only when the host intentionally owns equilibrium/residuals. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04).

**Required actual data:** Actual component slate; critical/acentric and ideal caloric data; interaction values and their temperature convention; reference conditions; density/transport data required by selected unit modes.

**Initialization contract:** TP and PH/PS starts plus homogeneous/two-phase routing. Require a bounded initializer and final original-target residual; no silent phase suppression.

**Derivative contract:** Direct simulation can start without outer derivatives. SM-02 needs local derivatives; SM-03 needs total solved response. Any SM-04 host phase problem needs separately qualified equations.

**Limits and missing work:** A catalog cubic implementation is not tested compressor/column coverage. Do not presume phase stability, transport, arbitrary mixtures or same-data numerical equivalence.

**First native/formulation witness:** Use one fully documented light-hydrocarbon mixture and a calorically complete pure-fluid check. Fix data first, then TP→H/S→PH/PS roundtrips, a valve and a compressor reference/actual outlet sequence.

**Experiment routes:** [IE-01](#ie-01), [IE-02](#ie-02), [IE-03](#ie-03), [IE-04](#ie-04), [IE-05](#ie-05), [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08), [IE-12](#ie-12), [IE-13](#ie-13), [IE-14](#ie-14), [IE-15](#ie-15), [IE-17](#ie-17), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-02](#ir-02), [IR-03](#ir-03), [IR-04](#ir-04), [IR-05](#ir-05), [IR-11](#ir-11), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-18](#ir-18), [IR-20](#ir-20). **Basis:** B3:L03, B3:L04, B3:L05, B2:WF-06, [E10](#e10), [E14](#e14). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-02"></a>
#### IP-02. Nonideal liquid gamma–phi package

**Scope connections:** SC-08, SC-09, SC-20. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** A named NRTL-based liquid plus explicit vapor fugacity and saturation/Henry standard states. Ideal vapor and cubic vapor are different approved variants. Caloric/excess choices and activity parameter temperature dependence are part of the package.

**Candidate realization route:** First comparison: DWSIM NRTL package versus thermo GibbsExcessLiquid with compatible gas/flash; Alternative: Clapeyron activity/fluid composite; Equation alternative: configured IDAES property/reaction formulation.

**Reuse boundary:** RB-03; RB-02/05 for an intentional coupled formulation. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** Directional NRTL coefficients, alpha and exact tau formula; pure/solvent saturation and gas-solute Henry conventions; ideal heat capacities and caloric treatment; solvent identity and validity evidence.

**Initialization contract:** Admissible phase starts; explicitly selected eligible TP/inverse algorithm; independent azeotrope/topology evidence for the eventual fixture; no near-ideal fallback hidden as success.

**Derivative contract:** Differentiate actual selected enthalpy and fugacity/activity functions, including temperature-dependent coefficients. Approximate calorics remain an explicitly different function and evidence claim.

**Limits and missing work:** Reactive separation in this row requires IP-10/11/12/13 coupling; ordinary gamma–phi alone is not SC-20 coverage. Missing Henry or temperature data is not zero interaction.

**First native/formulation witness:** A documented polar-mixture fixture with its own parameter set: phase relation and original heat duty checked separately; then compare a deliberately allowed caloric approximation without relabeling it exact.

**Experiment routes:** [IE-01](#ie-01), [IE-02](#ie-02), [IE-03](#ie-03), [IE-05](#ie-05), [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08), [IE-11](#ie-11), [IE-12](#ie-12), [IE-13](#ie-13), [IE-14](#ie-14), [IE-17](#ie-17), [IE-18](#ie-18).

**Rules:** [IR-02](#ir-02), [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-07](#ir-07), [IR-09](#ir-09), [IR-11](#ir-11), [IR-14](#ir-14), [IR-16](#ir-16), [IR-20](#ir-20). **Basis:** B3:L02, B3:L03, B3:L05, [E08](#e08), [E09](#e09), [E11](#e11). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-03"></a>
#### IP-03. Multiple-liquid and vapor–multiple-liquid package

**Scope connections:** SC-10, SC-11, SC-20, SC-24. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** One shared physical phase-allocation problem with explicit liquid candidates/eligibility, permitted vapor and any separately supported solids. No duplication of a one-liquid solution to mimic LLE.

**Candidate realization route:** DWSIM configured multiphase route; thermo FlashVLN for its supported fluid scope; Clapeyron explicitly selected multiphase TP route; Solids or reactive variants use IP-11/12/13 instead of assuming generic extension.

**Reuse boundary:** RB-03 for eligible whole flash; RB-02/05 for an intentional host phase formulation. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** Reference partially miscible system; liquid interaction data; vapor and caloric conventions; parameter validity; explicit physical phase count and candidate search policy.

**Initialization contract:** Phase-creation/incipient information and bounded stability search. Qualify TP separately from PH/PS. An energy-constrained outer T root is conditional and must handle branch/saturation discontinuities.

**Derivative contract:** Fixed-domain phase derivatives differ from total multiphase response. Cross-phase-count derivatives require explicit transition or fixed-superset treatment.

**Limits and missing work:** Pinned B3 thermo VLN rejects supplied solids; the Clapeyron compatibility matrix is algorithm-specific. Automatic phase search does not supply every inverse/chemistry route or certify global stability.

**First native/formulation witness:** A documented two-liquid TP state and a separate VLL state. Permute output entries, test absent phases and routing, then test inverse variants only with complete energy and initialization data.

**Experiment routes:** [IE-03](#ie-03), [IE-04](#ie-04), [IE-05](#ie-05), [IE-06](#ie-06), [IE-07](#ie-07), [IE-09](#ie-09), [IE-13](#ie-13), [IE-14](#ie-14), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-02](#ir-02), [IR-04](#ir-04), [IR-05](#ir-05), [IR-08](#ir-08), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-16](#ir-16), [IR-20](#ir-20). **Basis:** B3:CH01, B3:CL02, B2:WF-07, [E10](#e10), [E11](#e11). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-04"></a>
#### IP-04. Pure water and steam utility package

**Scope connections:** SC-02, SC-03, SC-12, SC-28. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** One pure-water formulation with a fixed reference convention. IAPWS-95 and IF97 are explicit alternatives, not interchangeable implementations assumed identical.

**Candidate realization route:** Initial utility route: selected CoolProp water backend; Comparison: relevant IDAES, thermo or DWSIM water package with explicit formulation.

**Reuse boundary:** RB-03 complete pure-fluid service, or RB-02 local property surface. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** Backend/formulation identity, reference choices, required thermal/transport domain, endpoint and quality bases. Comparison to another formulation records model differences.

**Initialization contract:** Saturated endpoints, subcooled/superheated states and quality/PH/PS independence checked separately. TP on coexistence cannot invent phase fraction.

**Derivative contract:** Homogeneous, saturation-path and two-phase responses each separately qualified. An endpoint density/enthalpy derivative is not arbitrary mixture sensitivity.

**Limits and missing work:** SM-04/05 only through an exposed/implemented qualified formulation, not by assuming a numerical water backend emits equations. Actual valid range follows the selected backend, not this template.

**First native/formulation witness:** A water loop with documented reference states; test saturated TP ambiguity and endpoint/quality resolution, then heat-only coupling to IP-01.

**Experiment routes:** [IE-01](#ie-01), [IE-04](#ie-04), [IE-09](#ie-09), [IE-12](#ie-12), [IE-13](#ie-13), [IE-15](#ie-15), [IE-17](#ie-17), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-04](#ir-04), [IR-05](#ir-05), [IR-09](#ir-09), [IR-10](#ir-10), [IR-11](#ir-11), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-18](#ir-18), [IR-20](#ir-20). **Basis:** B3:L02, B3:L05, B3:L06, [E12](#e12), [E06](#e06). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-05"></a>
#### IP-05. Pure-fluid refrigeration package

**Scope connections:** SC-13. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** A selected pure working fluid with coherent phase/caloric/transport treatment, fixed reference and supported compression/condensation/expansion operations.

**Candidate realization route:** Initial candidate: exact CoolProp pure-fluid backend; Alternative: qualified whole-provider pure-fluid route in DWSIM, thermo or Clapeyron.

**Reuse boundary:** RB-03; local property callbacks available only where exposed and qualified. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03).

**Required actual data:** Exact fluid identity and backend/data reference; caloric and phase saturation definition; demanded transport and derivative data.

**Initialization contract:** PH expansion, PS ideal compression reference and actual PH outlet; near-saturation branch policy and valid starts; no pure-fluid success promotion to mixtures.

**Derivative contract:** Separate local gas/liquid derivatives, saturation path and solved cycle response; branch events must be explicit for optimization.

**Limits and missing work:** Complete loop work/heat agreement requires the unit/flowsheet model in addition to provider outputs. No arbitrary phase-transition smoothness claim.

**First native/formulation witness:** One selected refrigeration-fluid loop with input data frozen; compare separate PS reference and actual compressor outlet and verify the cycle balance.

**Experiment routes:** [IE-01](#ie-01), [IE-04](#ie-04), [IE-09](#ie-09), [IE-13](#ie-13), [IE-15](#ie-15), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-05](#ir-05), [IR-09](#ir-09), [IR-11](#ir-11), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-20](#ir-20), [IR-21](#ir-21). **Basis:** B3:L06, B4:SC-13, [E12](#e12). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-06"></a>
#### IP-06. True-mixture refrigerant package

**Scope connections:** SC-14. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** A declared composition with separate liquid/vapor compositions and actual mixture interactions. A pseudo-pure approximation is a separate configuration and not a hidden replacement.

**Candidate realization route:** Compare exposed mixture routes in thermo, Clapeyron, ThermoPack and a selected CoolProp backend; DWSIM whole configured mixture route remains a candidate.

**Reuse boundary:** RB-03 or configured EOS session; do not infer common capability across backends. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03).

**Required actual data:** Component identities, mixture interaction data, ideal/caloric models, saturation definition, molar/mass quality and reference convention.

**Initialization contract:** Qualify the exact TP/PH/PS and saturation pairs with mixture phase behavior; tests must include distinct endpoint compositions when applicable.

**Derivative contract:** Composition-chart and total-mixture response derivatives require exact exposed evidence. An implemented inverse source path is not a verified derivative surface.

**Limits and missing work:** No preferred numerical winner yet; prior CoolProp source findings remain version-specific. Mixture capability cannot be inherited from IP-05.

**First native/formulation witness:** Pin a documented blend, check composition preservation and endpoint behavior, then matching TP→H/S→PH/PS on actual supported phase regions.

**Experiment routes:** [IE-02](#ie-02), [IE-04](#ie-04), [IE-09](#ie-09), [IE-13](#ie-13), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-03](#ir-03), [IR-04](#ir-04), [IR-05](#ir-05), [IR-11](#ir-11), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-18](#ir-18), [IR-20](#ir-20), [IR-24](#ir-24). **Basis:** B3:L04, B3:L05, B3:L06, B3:SC-14, [E14](#e14), [E12](#e12). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-07"></a>
#### IP-07. Assay-characterized compositional petroleum

**Scope connections:** SC-15, SC-29, SC-30. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** Original assay plus an explicit cut-generation recipe yields identified pseudocomponent definitions and complete properties for the intended thermal/separation calculations.

**Candidate realization route:** First workflow investigation: DWSIM characterization plus its configured package; Alternative custom-data route: host characterization with thermo or Clapeyron; ThermoPack cubic pseudocomponent route only with explicitly completed and qualified calorics.

**Reuse boundary:** P02/P01 coordinated preparation feeding RB-03 or a newly qualified RB-02 composition. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04).

**Required actual data:** Raw assay and basis/test conventions; cut boundaries; estimated MW/critical/acentric data where justified; full ideal/caloric and density correlations; characterization fit evidence; parameter provenance.

**Initialization contract:** Recharacterization creates new identities/coordinates and invalidates seeds as appropriate; qualify thermal inverse only after caloric completion.

**Derivative contract:** Derivatives after characterization distinguish fixed cuts/data from sensitivity to assay or cut recipe; no automatic differentiability through discrete cut changes.

**Limits and missing work:** This remains an unresolved P2 numerical workflow. B3 documents runtime cubic pseudocomponents but incomplete ideal calorics in the inspected ThermoPack example. No silent PVT-to-energy promotion.

**First native/formulation witness:** Choose an actual accessible assay with its cut recipe and reference evidence. Save/rebuild generated definitions, check assay reconciliation, then a heater/flash and an explicitly lossy lumping boundary.

**Experiment routes:** [IE-01](#ie-01), [IE-02](#ie-02), [IE-12](#ie-12), [IE-16](#ie-16), [IE-17](#ie-17), [IE-18](#ie-18).

**Rules:** [IR-03](#ir-03), [IR-04](#ir-04), [IR-05](#ir-05), [IR-06](#ir-06), [IR-10](#ir-10), [IR-11](#ir-11), [IR-15](#ir-15), [IR-22](#ir-22), [IR-23](#ir-23), [IR-24](#ir-24). **Basis:** B2:WF-14, B3:TP04, B3:W08, B4:FR-DAT-05. **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-08"></a>
#### IP-08. Reduced black-oil or bulk petroleum representation

**Scope connections:** SC-16, SC-29, SC-30. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** An explicit empirical oil/gas/water representation with standard-condition conventions and model-specific inputs; it is not an unobserved detailed compositional mixture.

**Candidate realization route:** DWSIM dedicated reduced-model route is the first investigation target; No complete alternative workflow established in B3; retain a visible coverage gap.

**Reuse boundary:** A qualified complete reduced-property service plus host representation/translation contracts. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03).

**Required actual data:** Actual empirical method inputs and fit/source ranges; standard pressure/temperature and oil/gas/water definitions; supported thermal properties or an explicit lack of them.

**Initialization contract:** Use only the empirical operation surface; a molecular flash initializer is not automatically meaningful.

**Derivative contract:** Differentiate the declared empirical functions if needed and valid; do not request species chemical-potential Jacobians without molecular data.

**Limits and missing work:** Numerical provider route, caloric completion and runtime behavior remain unestablished. Unsupported detailed chemistry is legitimate, but the P2 surface-separation requirement remains open.

**First native/formulation witness:** Pin one black-oil convention and an actual supported surface-separation fixture; verify standard/actual quantities and reject a unique detailed reverse map.

**Experiment routes:** [IE-12](#ie-12), [IE-16](#ie-16), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-04](#ir-04), [IR-05](#ir-05), [IR-06](#ir-06), [IR-10](#ir-10), [IR-11](#ir-11), [IR-22](#ir-22), [IR-23](#ir-23), [IR-24](#ir-24). **Basis:** B3:SC-16, B3:W08, B4:FR-FLW-06. **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-09"></a>
#### IP-09. Mass-based empirical material and carried inert solids

**Scope connections:** SC-02, SC-23, SC-31. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** Disjoint fluid and inert/empirical solid accounts with explicitly supported mass-based h,Cp,density and named effective-property/carryover assumptions.

**Candidate realization route:** Host semantic/limited-property implementation using justified supplied correlations; Any listed data/phase library may supply a qualified component correlation; no complete existing route is presumed.

**Reuse boundary:** RB-01/02 limited contributions composed with a coherent fluid package and host material accounting. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** Mass basis, supported correlations and reference; solid participation (carried versus reactive); unknown MW/element data stay unknown; particle attributes only if used by a declared closure.

**Initialization contract:** A mass-based heater can initialize independently of a molecular EOS. Slurry/thermal mixed state and mechanical separation need explicit unit assumptions.

**Derivative contract:** Local derivatives of actual empirical functions where defined. Effective-property derivatives follow the selected closure, not a guessed universal mixture rule.

**Limits and missing work:** No fugacity, molar quantities, elemental balance or reactive-solid coverage is inferred from supported heating. Actual field data and effective closures remain qualification tasks.

**First native/formulation witness:** Start with the existing authored mass-only heat balance, then acquire a real material correlation fixture. Test mechanical solids routing without giving inert material phase-equilibrium authority.

**Experiment routes:** [IE-02](#ie-02), [IE-11](#ie-11), [IE-16](#ie-16), [IE-18](#ie-18).

**Rules:** [IR-04](#ir-04), [IR-05](#ir-05), [IR-06](#ir-06), [IR-07](#ir-07), [IR-08](#ir-08), [IR-11](#ir-11), [IR-16](#ir-16), [IR-20](#ir-20), [IR-23](#ir-23), [IR-24](#ir-24). **Basis:** B4:FR-MAT-02, B4:FR-PRP-04, B3:W08. **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-10"></a>
#### IP-10. Specified conversion and finite-rate chemistry with a compatible property package

**Scope connections:** SC-17, SC-19, SC-25. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** Host/P06 transformation or rates with explicit stoichiometry/normalization/participation; compatible fluid/solid calorics and a declared reactor thermal constraint. Strict extent and limited-conversion optimization remain distinct.

**Candidate realization route:** Host conversion or rate contribution + IP-01/02/09 property service as appropriate; IDAES-style reaction/property formulation; Reaktoro kinetic subset only when its exposed meaning matches the unit request.

**Reuse boundary:** RB-02/03/05 plus host transformation, or RB-04 when the requested kinetic/equilibrium subset is coherent. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** Balanced stoichiometry and justified conserved basis; fixed extent or kinetic law parameters; activities/concentration basis; formation data and correction; solid definitions when reactive.

**Initialization contract:** Check strict extent before candidate creation; finite-rate solver initialization must preserve frozen species and original unit residence/transport assumptions.

**Derivative contract:** Derivative of rates includes the actual property and basis conversions. A frozen-chemistry derivative is different from an equilibrium sensitivity.

**Limits and missing work:** DWSIM observed reactant-limited conversion is not a pass for strict extent without explicit change of problem. Gas–solid and kinetic unit behavior remains profile-specific.

**First native/formulation witness:** Existing synthetic strict A→B feasibility and equivalent energy formulations, followed by one actual documented reaction with complete data and host unit balance.

**Experiment routes:** [IE-10](#ie-10), [IE-15](#ie-15), [IE-18](#ie-18).

**Rules:** [IR-04](#ir-04), [IR-05](#ir-05), [IR-08](#ir-08), [IR-09](#ir-09), [IR-11](#ir-11), [IR-12](#ir-12), [IR-15](#ir-15), [IR-16](#ir-16), [IR-20](#ir-20), [IR-21](#ir-21), [IR-24](#ir-24). **Basis:** B2:WF-09, B4:FD-03, B3:W05, [E13](#e13). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-11"></a>
#### IP-11. Coherent chemical equilibrium, aqueous speciation and mineral phases

**Scope connections:** SC-18, SC-21, SC-22, SC-24, SC-25, SC-30, SC-33. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** A single declared chemical system with compatible standard thermodynamics/activity models, admitted species/phases, conserved totals, restrictions and thermal/exchange conditions.

**Candidate realization route:** Initial chemical-system candidate: Reaktoro upstream qualified interface; DWSIM Reaktoro adapter assessed as its own narrower realization; IDAES alternative treated as IP-12, not interchangeable data by default.

**Reuse boundary:** RB-04 complete chemical system; phase property queries only when explicitly exposed. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03).

**Required actual data:** Exact database and model selection; species/solid forms and true/apparent map; H/S/U completeness for energy requests; reservoir and charge conventions; separately demanded transport.

**Initialization contract:** Chemical state as private initial/final work; supported generalized constraints separately tested; all exchanges authorized and checked; UV remains P3 unless a separate numerical claim is intentionally added.

**Derivative contract:** Exposed sensitivities qualified against selected specs/restrictions and active phases. No automatic arbitrary higher-order Hessian or transport derivative.

**Limits and missing work:** Do not infer complete absorber/crystallizer kinetics or transport from speciation. Species result plus unrelated calorics/reflash is not accepted without coherent composition.

**First native/formulation witness:** One closed TP chemical fixture with defined conserved quantities, then its energy-constrained variant and separately an authorized titrant/reservoir example. Mineral tests state allowed solids explicitly.

**Experiment routes:** [IE-01](#ie-01), [IE-05](#ie-05), [IE-06](#ie-06), [IE-07](#ie-07), [IE-10](#ie-10), [IE-11](#ie-11), [IE-12](#ie-12), [IE-13](#ie-13), [IE-14](#ie-14), [IE-17](#ie-17), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-04](#ir-04), [IR-05](#ir-05), [IR-07](#ir-07), [IR-08](#ir-08), [IR-09](#ir-09), [IR-11](#ir-11), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-18](#ir-18), [IR-20](#ir-20), [IR-24](#ir-24). **Basis:** B3:L08, B2:WF-15, [E13](#e13). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-12"></a>
#### IP-12. Equation-oriented electrolyte/reaction alternative

**Scope connections:** SC-18, SC-20, SC-21, SC-22, SC-24, SC-25. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** A configured IDAES-style phase/property/reaction mathematical system, including eNRTL only where its exact parameter and reference configuration applies.

**Candidate realization route:** IDAES physical-parameter/state/reaction construction as initial formulation reference; Equivalent host equations only after method/data/derivative qualification.

**Reuse boundary:** RB-05 explicit equation or residual contribution; not an assumed turnkey numerical package. **Solver arrangements:** [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** True/apparent species and eligibility, ionic/interaction parameters, standard-state thermochemistry, reaction constraints, complete caloric data and all construction options.

**Initialization contract:** Staged initialization with fixing/release and scale evidence; structural candidate phases declared; restore original host constraints before acceptance.

**Derivative contract:** Derivatives of expressed equations plus any compiled external functions; total solved sensitivities separately qualified; Hessian demand depends on selected host solver mode.

**Limits and missing work:** An implemented eNRTL source file does not supply every ready-parameterized aqueous process, precipitation law or nonlinear solver dependency. No guaranteed parity with Reaktoro.

**First native/formulation witness:** Small configured reaction/state unit with explicit degrees of freedom and scaling; verify original constraints after initialization and compare only genuinely equivalent conventions/data.

**Experiment routes:** [IE-01](#ie-01), [IE-08](#ie-08), [IE-09](#ie-09), [IE-10](#ie-10), [IE-17](#ie-17), [IE-18](#ie-18).

**Rules:** [IR-01](#ir-01), [IR-02](#ir-02), [IR-03](#ir-03), [IR-05](#ir-05), [IR-08](#ir-08), [IR-09](#ir-09), [IR-11](#ir-11), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-16](#ir-16), [IR-19](#ir-19), [IR-20](#ir-20). **Basis:** B3:L02, B3:ID03, [E06](#e06), [E07](#e07). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-13"></a>
#### IP-13. Staged, reactive and full nonequilibrium contacting

**Scope connections:** SC-07, SC-09, SC-20, SC-22, SC-26. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** Host stage/segment/bulk/interface formulation, with explicit transfer/routing and thermal assumptions, using qualified phase, chemistry and transport contributions.

**Candidate realization route:** Host coupled formulation + thermo/Clapeyron or other eligible phase functions; IDAES-style local property/reaction structure; Qualified Reaktoro subblocks where authority and total responses fit the formulation.

**Reuse boundary:** RB-02/04/05 assembled at the process-model boundary, not a pretend standalone flash service. **Solver arrangements:** [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** All required phase properties, diffusion/interfacial conventions and rate inputs; bulk/interface locations; energy/chemical standards; geometry and transfer closures supplied by unit model.

**Initialization contract:** Consistent stage seeds, branch/phase strategy, authority separation and active-set restoration. Matching package revision is not enough for consistent final iterate lineage.

**Derivative contract:** Differentiate the actual coupled functions; choose condensed chemical blocks versus exposed variables explicitly. Transport state dependence and matrix frames must follow host coordinates.

**Limits and missing work:** Full SC-26 bulk/interface behavior is not an efficiency-corrected equilibrium-stage calculation. No listed library by itself closes the whole unit or parent recycle requirement.

**First native/formulation witness:** One nonreactive staged unit with property-only callbacks, then a separate full bulk/interface fixture. Add chemistry only after those contracts and the coupled energy/species checks are demonstrated.

**Experiment routes:** [IE-05](#ie-05), [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08), [IE-09](#ie-09), [IE-10](#ie-10), [IE-11](#ie-11), [IE-15](#ie-15), [IE-18](#ie-18).

**Rules:** [IR-02](#ir-02), [IR-07](#ir-07), [IR-08](#ir-08), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15), [IR-16](#ir-16), [IR-19](#ir-19), [IR-20](#ir-20), [IR-21](#ir-21), [IR-24](#ir-24). **Basis:** B2:WF-10, B4:FR-FLW-08, B3:W04, [E04](#e04), [E06](#e06). **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.

<a id="ip-14"></a>
#### IP-14. Deferred surface, distribution, inventory and restricted-state extensions

**Scope connections:** SC-27, SC-32, SC-33, SC-34. These connections can be contributions to a scenario; they are not independent claims that a provider supplies the complete unit.

**Physical definition:** Four separately qualified semantic subprofiles: surface/selective transfer, distributed attributes, inventory-state constraints, and restricted/metastable branch requests. This grouping is for review, not one universal numerical engine.

**Candidate realization route:** Use existing P01/P04/P05/P06 contracts for independent representability review; Potential future numerical contributions from FeOS surfaces, Reaktoro or ThermoPack inventory operations, or provider-specific restricted phases only after exact qualification.

**Reuse boundary:** Semantic/accounting contribution first; no numerical reuse boundary mandated. **Solver arrangements:** [SM-01](#sm-01), [SM-02](#sm-02), [SM-03](#sm-03), [SM-04](#sm-04), [SM-05](#sm-05).

**Required actual data:** Loading denominator/site meaning, distribution weighting/reduction, actual inventory amount and total U/V, or exact physical restriction/branch. Each subprofile has separate requirements.

**Initialization contract:** No dummy flow, invented unique distribution, false surface phase or silent unrestricted branch. Numeric initialization remains unassigned where execution is deferred.

**Derivative contract:** Only actually supplied derivative meaning is claimed; no smoothness across phase appearance, distribution reduction or loading convention changes.

**Limits and missing work:** P3 representability only remains the commitment. Mode entries describe possible future realization, not required or working implementations. No source API promotes these scenarios to E/V.

**First native/formulation witness:** Complete the four B7 conceptual transfer/change/result walkthroughs and independently review them; retain blocked numerical requests. Later numeric experiments require an explicit scope/evidence decision.

**Experiment routes:** [IE-18](#ie-18).

**Rules:** [IR-04](#ir-04), [IR-08](#ir-08), [IR-10](#ir-10), [IR-11](#ir-11), [IR-14](#ir-14), [IR-16](#ir-16), [IR-21](#ir-21), [IR-22](#ir-22), [IR-23](#ir-23), [IR-24](#ir-24). **Basis:** B4:FR-EXT-01, B4:FR-EXT-02, B4:FR-EXT-03, B4:FR-EXT-04, B7:WF7-16, B7:WF7-17. **Status:** screened template; exact production realization unselected; native execution and independent validation not performed.



### 8.3 Unit demands determine the minimum numerical surface

| Process use | Essential thermodynamic demand | Additional demand only for the chosen mode | Inadmissible shortcut |
| --- | --- | --- | --- |
| Duty heater or cooler | Complete enthalpy at known states and eligible inverse state when outlet T is unknown | Transport/rating correlations if geometry/area is being rated | Treating a density-capable package as energy-complete |
| Valve | Correct upstream energy convention and downstream PH problem | Total solved response if embedded in a differentiable outer solve | Adding missing enthalpy after solving partial PH |
| Compressor/expander | PS reference state and actual outlet energy relation using complete H/S | Qualified process/flash derivatives for coupled optimization | Reporting the ideal reference as the actual machine outlet |
| Equilibrium separator | Eligible shared phase allocation plus calorics required by its thermal constraint | Phase correspondence, interfacial property or carryover closure as required by unit | Treating a port label or numerical slot as a physical phase |
| Equilibrium-stage column | Repeated phase properties, equilibrium relations and complete calorics at local trial states | Jacobian/Hessian/residual contribution according to selected global solver | Forcing a new full stream flash that overwrites host stage unknowns |
| Reactive unit | Strict transformation or coherent equilibrium/rate relations and energy convention | Chemistry sensitivities, transport, mineral/solid models only as required | Two independent unqualified chemistry/flash successes |
| Full nonequilibrium contactor | Separate bulk/interface phase/property inputs with transfer conventions | Multicomponent diffusion, rates and differentiated coupling per formulation | Replacing it by an efficiency-corrected equilibrium-stage model |
| Heat exchanger across packages | Coherent side-specific energy differences and actual heat balance | Optional hypothetical limits and rating properties independently qualified | Requiring equal absolute h or material translation across a wall |
| Empirical material heating | Justified mass-based thermal relationship and actual flow/inventory basis | Molecular/chemical properties only if independently supplied | Invented MW or one-second dummy flow |

The outer solver only obtains the derivatives required by its selected mode. This avoids making exact Hessians or universal symbolic models a prerequisite for simple process simulation while preventing unsupported derivative claims when an optimization mode is requested.

## 9. Qualification gates and bounded experiments

### 9.1 Gate interpretation

A gate states what evidence is needed for a particular claim. It is not a single linear score from immature to mature, and not every property request needs every scientific test. A production energy or reactive claim, however, cannot waive a mandatory prerequisite merely because a narrower density or TP calculation works.

Readiness checks can establish that a bounded experiment is safe and meaningful to attempt. They do not establish successful execution or validation. A deliberately approximate engineering profile may be approved for a named operational use with its limitations, while the independent-validation dimension remains absent or bounded.

| Gate | Owner | Required evidence | Blocking distinction |
| --- | --- | --- | --- |
| <a id="qg-01"></a>QG-01 Instantiate an attributable profile | P03 | Concrete material and use, selected coherent definition, exact realization/data or explicit unresolved dependencies, solver mode and desired scope. IC-20/35/53 and profile resolution record | An uninstantiated template cannot claim production readiness. |
| <a id="qg-02"></a>QG-02 Resolve representation and parameter semantics | P01 | Identity/coordinate/basis maps, parameter formula conversions, justified conserved quantities and input authority. IC-03/04/05/12/14/37/58 | Unknown formula, missing molecular conversion or duplicate accounts blocks affected use. |
| <a id="qg-03"></a>QG-03 Qualify physical and caloric composition | P03 | Phase/standard/caloric/chemistry compatibility, intentional approximations, total inverse function, unit/boundary convention. IC-21/22/23/24/45/51 | Do not qualify energy from residual-only or incompatible standard-state data. |
| <a id="qg-04"></a>QG-04 Qualify the exact exposed operation | P08 | Actual adapter supports required state pair, allowed phases/species, outputs, initialization and build dependencies. IC-34/53/57 | Source existence or upstream catalog is not executed adapter support. |
| <a id="qg-05"></a>QG-05 Qualify solver-mode mathematics | P05 | Unknown ownership, branch/structure, original constraints, derivative chart/order/scale and inner error requirements. IC-35/36/39/40/41/72 | A direct flash is not a smooth or differentiated callback by default. |
| <a id="qg-06"></a>QG-06 Demonstrate session and failure containment | P08 | Isolated complete call sequences, cleanup/error recovery, cancellation and deterministic context; actual build-specific tests. IC-54/55/56/70 | Unknown or fatal in-process behavior requires stronger containment or an explicit block. |
| <a id="qg-07"></a>QG-07 Demonstrate numerical contract conformance | P09 | Executed positive/negative state and derivative fixtures with original-target/conservation/authority checks; source and exact manifest retained. IC-41/59/60/62 | Structural tables and authored mathematical tests do not pass a native contract. |
| <a id="qg-08"></a>QG-08 Demonstrate process-boundary and lifecycle behavior | P07 | Executed unit/coupled tests, translation, routing, immutable input, run-local coherence, save/rebuild and guarded publication. IC-50/51/52/64/66/67/68/69 | Local phase success cannot certify whole-unit/recycle or coherent current results. |
| <a id="qg-09"></a>QG-09 Bound independent physical-validation claims | P09 | Independent reference evidence and predeclared scientific uncertainty/tolerances over a stated material/operation envelope. IC-10/15/61 | Shared model/data agreement is not independent validation; absent evidence leaves claim limited. |
| <a id="qg-10"></a>QG-10 Approve a specific operational use or retain a gap | P10 | Relevant gate results, deliberate approved approximations and evidence limits, exact available dependencies and active publication authority. IC-24/60/61/64/70 | No blanket package-wide production/validated claim; requested use determines mandatory gates and evidence. |


P03 owns the scientific interpretation of package compatibility and readiness. P05 owns the mathematical/authority contract. P08 supplies actual build/interface/session facts. P09 owns numerical conformance and scientific evidence. P07 supplies unit/boundary completion requirements. P10 commits the permitted use and current references. This does not introduce another authority outside the ten packages.

### 9.2 Eighteen executable-investigation specifications

The following tests are planned for actual candidate libraries and process formulations. **None was executed in this stage.** They preserve and organize all 18 B2 and 20 B3 native probe obligations rather than treating catalog links as passes.

A test instance must first fix its materials, actual data sources, operating points, branch conditions, native build/binding and predeclared numerical/scientific tolerances. A synthetic fixture may be used to test the host contract; it is not substituted for real-fluid accuracy evidence. Some listed scenarios require separate plausible fixtures rather than one implausible all-feature plant.

<a id="ie-01"></a>
#### IE-01. Freeze data and reconstruct a package

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-04](#ip-04), [IP-05](#ip-05), [IP-07](#ip-07), [IP-11](#ip-11), [IP-12](#ip-12). **Question:** Are visible definitions and actual data stable across sessions and reload?

**Procedure:** Capture exact methods, data/artifacts and conventions; construct fresh sessions twice and reload semantic archive without fresh data selection.

**Required witness:** Matched definitions and tolerance-qualified outputs with no hidden estimation; actual artifacts recorded.

**Reject/control case:** Change a database or lose a required backend and verify no silent substitution.

**Inherited probes:** B2:P01, B2:P02, B2:P11, B3:PV-01, B3:PV-02, B3:PV-16, B2:P16. **Rules:** [IR-03](#ir-03), [IR-17](#ir-17), [IR-22](#ir-22). **Status:** not executed.

<a id="ie-02"></a>
#### IE-02. Caloric completion and inverse consistency

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-06](#ip-06), [IP-07](#ip-07), [IP-09](#ip-09). **Question:** Does the selected PH/PS function include all intended caloric terms?

**Procedure:** Evaluate complete forward H/S, resolve matching inverse requests, and compare with deliberate partial-caloric variants. Include temperature-dependent interaction data.

**Required witness:** Original complete target residuals pass; partial route explicitly rejected or labeled limited.

**Reject/control case:** Post hoc ideal addition after residual-only solve must fail.

**Inherited probes:** B3:PV-07, B3:PV-08, B3:PV-11. **Rules:** [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06). **Status:** not executed.

<a id="ie-03"></a>
#### IE-03. One-liquid and multiple-liquid authority

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03). **Question:** Are candidate search, presence, identity and routing distinct?

**Procedure:** Run referenced VLE/LLE/VLL fixtures, reorder native outputs and test phase disappearance; verify properties at fixed returned phases without reflash.

**Required witness:** Account reconstruction, explicit stability scope and correct product matching or ambiguity.

**Reject/control case:** A numerical placeholder cannot create a positive outlet; VLE cannot pass required VLLE.

**Inherited probes:** B2:P05, B2:P07, B3:PV-03, B3:PV-04, B3:PV-10. **Rules:** [IR-04](#ir-04), [IR-07](#ir-07), [IR-14](#ir-14). **Status:** not executed.

<a id="ie-04"></a>
#### IE-04. Inverse initializer and saturation eligibility

**Profiles:** [IP-01](#ip-01), [IP-03](#ip-03), [IP-04](#ip-04), [IP-05](#ip-05), [IP-06](#ip-06). **Question:** Which inverse requests can actually be initialized and solved?

**Procedure:** Test chosen TP→H/S→inverse cases in homogeneous and two-phase regions, pure saturation with/without quality, and required versus absent seeds.

**Required witness:** Initializable/unsupported/underdetermined outcomes differentiated and final physical targets checked.

**Reject/control case:** Scalar root sign change at saturation jump cannot count as solved PH.

**Inherited probes:** B3:PV-05, B3:PV-06, B3:PV-11, B3:PV-13, B2:P16, B3:PV-10. **Rules:** [IR-05](#ir-05), [IR-14](#ir-14), [IR-15](#ir-15). **Status:** not executed.

<a id="ie-05"></a>
#### IE-05. Phase-property callback preservation

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03), [IP-11](#ip-11), [IP-13](#ip-13). **Question:** Can properties be evaluated without taking phase/species authority?

**Procedure:** Supply deliberately separate bulk phases or an externally owned split; ask for selected phase properties and verify every protected quantity afterward.

**Required witness:** Required phase/chemical authority invariants hold; permitted density/root choices recorded.

**Reject/control case:** A property refresh that equilibrates the bulk or consumes frozen species is rejected.

**Inherited probes:** B2:P05, B3:PV-17, B3:PV-20. **Rules:** [IR-07](#ir-07), [IR-16](#ir-16). **Status:** not executed.

<a id="ie-06"></a>
#### IE-06. Total reduced response versus local derivative

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03), [IP-11](#ip-11), [IP-13](#ip-13). **Question:** Does a condensed block return the derivative of its actual solved output?

**Procedure:** Away from transitions, compare exposed total sensitivities or an implicit construction against controlled full re-solves in the same chart/branch; retain local phase derivatives separately.

**Required witness:** Coordinate-matched derivative agreement within declared inner/outer error budgets, plus conditioning evidence.

**Reject/control case:** Frozen-y partial derivative and an AD wrapper around a value-only black box fail.

**Inherited probes:** B3:PV-14, B3:PV-17. **Rules:** [IR-11](#ir-11), [IR-12](#ir-12). **Status:** not executed.

<a id="ie-07"></a>
#### IE-07. Inner-error and finite-difference budget

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03), [IP-11](#ip-11), [IP-13](#ip-13). **Question:** Can inner solve error meet outer residual/derivative demands?

**Procedure:** Vary inner tolerances and perturbation scales on fixed branch; measure actual repeated-evaluation noise and effect on host residual/Jacobian.

**Required witness:** A bounded policy with sensitivity and noise evidence; ill-conditioned cases are restricted or escalated.

**Reject/control case:** Shrinking the FD step while maintaining noisy inner solves is not accepted as higher accuracy.

**Inherited probes:** B3:PV-14. **Rules:** [IR-11](#ir-11), [IR-12](#ir-12), [IR-20](#ir-20). **Status:** not executed.

<a id="ie-08"></a>
#### IE-08. Hessian, chart and sparse-structure contract

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-12](#ip-12), [IP-13](#ip-13). **Question:** Is the selected coupled mode compatible with the chosen solver interface?

**Procedure:** Check local-to-host coordinate/scaling maps, structural nonzeros, Jacobian and requested weighted Hessian. Exercise a declared first-order plus Hessian-approximation alternative.

**Required witness:** No missing chain-rule terms or changing callback dimensions; mode-specific derivative obligations explicit.

**Reject/control case:** Observed numerical zeros cannot remove potential structural dependencies.

**Inherited probes:** B3:PV-14, B3:PV-15. **Rules:** [IR-11](#ir-11), [IR-13](#ir-13), [IR-16](#ir-16). **Status:** not executed.

<a id="ie-09"></a>
#### IE-09. Phase-transition and branch strategy

**Profiles:** [IP-03](#ip-03), [IP-04](#ip-04), [IP-05](#ip-05), [IP-06](#ip-06), [IP-12](#ip-12), [IP-13](#ip-13). **Question:** What happens when the active phase structure changes?

**Procedure:** Traverse neighboring physical states with the chosen outer restart or fixed-superset/smoothed strategy; compare original physical checks and auxiliary state meaning.

**Required witness:** Explicit branch event and layout handling; smoothing/regularization limitations retained.

**Reject/control case:** A mid-solve reordered or resized phase output violates the solver contract.

**Inherited probes:** B2:P07, B3:PV-03, B3:PV-06. **Rules:** [IR-13](#ir-13), [IR-14](#ir-14), [IR-15](#ir-15). **Status:** not executed.

<a id="ie-10"></a>
#### IE-10. Coupled chemistry, reaction heat and open exchange

**Profiles:** [IP-10](#ip-10), [IP-11](#ip-11), [IP-12](#ip-12), [IP-13](#ip-13). **Question:** Do final chemistry, phase and energy constraints hold together?

**Procedure:** Use pinned balanced chemistry and compatible standard data; compare equivalent reference formulations; exercise closed and explicitly open constraints plus frozen/kinetic subsets.

**Required witness:** All original balances and chemical/phase/thermal criteria pass and exchange amounts appear once.

**Reject/control case:** Infeasible strict extent, undeclared reservoir or sequentially invalidated speciation fails.

**Inherited probes:** B2:P15, B3:PV-17, B3:PV-18. **Rules:** [IR-08](#ir-08), [IR-09](#ir-09), [IR-20](#ir-20). **Status:** not executed.

<a id="ie-11"></a>
#### IE-11. Supplemental transport and effective properties

**Profiles:** [IP-02](#ip-02), [IP-09](#ip-09), [IP-11](#ip-11), [IP-13](#ip-13). **Question:** Can a separate correlation be added safely and meaningfully?

**Procedure:** Use captured phase states and explicit solvent/diffusion/closure conventions; evaluate primary and optional variants with fault injection.

**Required witness:** No phase/species mutation; correct subject/units; state derivatives qualified for coupled modes.

**Reject/control case:** Missing diffusivity cannot be replaced by an arbitrary mass-transfer coefficient or silently pure-water value.

**Inherited probes:** B2:P17, B3:PV-20. **Rules:** [IR-07](#ir-07), [IR-11](#ir-11), [IR-20](#ir-20). **Status:** not executed.

<a id="ie-12"></a>
#### IE-12. Heat-only and directed material translation

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-04](#ip-04), [IP-07](#ip-07), [IP-08](#ip-08), [IP-11](#ip-11). **Question:** Does boundary treatment preserve exactly the intended quantities?

**Procedure:** Test independent exchanger references, then actual source-target material cases with documented reference relation and either TP or corrected-PH preservation; test lossy mapping separately.

**Required witness:** Heat balance independent of arbitrary consistent side offsets; transformations and model discrepancies separately visible.

**Reject/control case:** Simultaneous inconsistent T,H preservation through hidden heat or unique unlumping from a total fails.

**Inherited probes:** B3:PV-19. **Rules:** [IR-04](#ir-04), [IR-09](#ir-09), [IR-10](#ir-10). **Status:** not executed.

<a id="ie-13"></a>
#### IE-13. Sessions, interleaving and native failure

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03), [IP-04](#ip-04), [IP-05](#ip-05), [IP-06](#ip-06), [IP-11](#ip-11). **Question:** Is the actual complete provider-call sequence isolated and recoverable?

**Procedure:** Alternate differently configured cases, then test only approved concurrent paths; inject errors, uninterruptible calls and cleanup uncertainty under the chosen containment.

**Required witness:** No configuration/buffer contamination; running contexts not reused; recovery/reconstruction evidence build-specific.

**Reject/control case:** Several instance handles are not accepted as a thread-safety proof; fatal crash risk is not handled by catch alone.

**Inherited probes:** B2:P03, B2:P13, B3:PV-09, B3:PV-12, B3:PV-20. **Rules:** [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19). **Status:** not executed.

<a id="ie-14"></a>
#### IE-14. Original inputs, alternative recovery and optional failure

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03), [IP-11](#ip-11). **Question:** Are retries truthful and are trustworthy primary results preserved?

**Procedure:** Force numeric failure, an attempted phase/model substitute, optional-property failure and a missing mandatory check; track candidate/problem identities.

**Required witness:** Same-problem retries retain targets; alternatives need authorization and new physical context; optional work does not falsify primary success.

**Reject/control case:** A failed native compound call cannot recover imaginary primary data from a stale buffer.

**Inherited probes:** B2:P04, B2:P17, B3:PV-20. **Rules:** [IR-17](#ir-17), [IR-19](#ir-19), [IR-20](#ir-20). **Status:** not executed.

<a id="ie-15"></a>
#### IE-15. Process integration, iteration lineage and publication

**Profiles:** [IP-01](#ip-01), [IP-04](#ip-04), [IP-05](#ip-05), [IP-10](#ip-10), [IP-13](#ip-13). **Question:** Do local candidates form a coherent accepted unit/flowsheet?

**Procedure:** Exercise valve, compressor, exchanger and recycle; race edits/cancellation with publication, repeat command delivery, and test old-stage/new-inlet combinations.

**Required witness:** Required process residuals and coherent candidate sets; no late/group-partial/replayed overwrite.

**Reject/control case:** Equal package revision with stale iterative inputs cannot pass final unit publication.

**Inherited probes:** B2:P06, B2:P14, B2:P10, B3:PV-20, B2:P08. **Rules:** [IR-16](#ir-16), [IR-20](#ir-20), [IR-21](#ir-21). **Status:** not executed.

<a id="ie-16"></a>
#### IE-16. Petroleum, reduced and empirical scope

**Profiles:** [IP-07](#ip-07), [IP-08](#ip-08), [IP-09](#ip-09). **Question:** Are broad-process material routes actually complete?

**Procedure:** Run a pinned assay characterization and thermal fixture, separately a reduced black-oil convention and a mass-only real-correlation heater; retain raw-to-derived lineage.

**Required witness:** Justified quantity/energy operations work without fabricated molecular data; all remaining P2 gaps explicit.

**Reject/control case:** A partial EOS custom-component example cannot certify complete petroleum or empirical unit scope.

**Inherited probes:** B2:P11, B3:PV-08. **Rules:** [IR-05](#ir-05), [IR-22](#ir-22), [IR-23](#ir-23), [IR-24](#ir-24). **Status:** not executed.

<a id="ie-17"></a>
#### IE-17. Exact restoration and upgrade comparison

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-04](#ip-04), [IP-07](#ip-07), [IP-11](#ip-11), [IP-12](#ip-12). **Question:** Can the profile and result meaning be reconstructed without current defaults?

**Procedure:** Save/rebuild with frozen data, temporarily missing backend, changed reference, coordinate ordering and changed build; inspect exact vs changed-model records.

**Required witness:** No hidden fitting, replayed transfers or resurrected run permission; manifest differences block exact reproduction claim.

**Reject/control case:** Restored arrays inserted into changed equations cannot inherit original acceptance.

**Inherited probes:** B2:P02, B2:P11, B2:P12, B2:P18, B3:PV-01, B3:PV-02, B3:PV-16, B2:P09. **Rules:** [IR-04](#ir-04), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22). **Status:** not executed.

<a id="ie-18"></a>
#### IE-18. Independent coverage review including P3

**Profiles:** [IP-01](#ip-01), [IP-02](#ip-02), [IP-03](#ip-03), [IP-04](#ip-04), [IP-05](#ip-05), [IP-06](#ip-06), [IP-07](#ip-07), [IP-08](#ip-08), [IP-09](#ip-09), [IP-10](#ip-10), [IP-11](#ip-11), [IP-12](#ip-12), [IP-13](#ip-13), [IP-14](#ip-14). **Question:** What evidence can legitimately be claimed?

**Procedure:** Review executed fixture manifests against scenario and solver-mode scope; compare genuinely independent reference data where available; independently review all four P3 conceptual action paths.

**Required witness:** Bounded separate R/E/V/conformance claims; accepted engineering approximations retain limitations; no new numerical P3 commitment.

**Reject/control case:** A passing synthetic guard or inherited source catalog cannot promote a real-fluid scenario.

**Inherited probes:** B3:PV-01, B3:PV-14. **Rules:** [IR-23](#ir-23), [IR-24](#ir-24). **Status:** not executed.



### 9.3 Session admission and native-failure policy

Until an exact build is shown to support independent sharing, assume no concurrent use of the same mutable session or ambient activation context. Serialize the **complete** activation/input/evaluation/readback/cleanup sequence or isolate it at a stronger boundary suitable for the native library. Where fatal native behavior would defeat the required host-state protection, qualification must establish appropriate process-level containment or block that deployment arrangement. Exception translation alone cannot prove containment.

An engine's internal parallel evaluation is separate from host-level reentrancy. Likewise, one handle per material does not prove independence if the implementation consults globals. A callback that reenters an already leased provider, or a derivative routine that makes nested calls, must be included in the actual qualification sequence. These are admission policies and test requirements, not claims that all listed libraries are unsafe.

Logical cancellation revokes publication permission immediately under the B7 contract; it need not immediately stop the native numerical operation. Still-running contexts retain ownership until safe termination/cleanup, then are reconstructed or reused only on demonstrated evidence. Native failure never gives a stale output buffer authority as a new result. [B7:AC-30/35/46; B3:TP03; E12,E13]

### 9.4 Repeatability, caching and parameter estimation

For a callback used by an outer solver, repeated evaluation at the same explicit inputs and captured configuration must agree within its declared repeatability/error policy. A warm start can accelerate the same branch calculation; it must not silently choose a different accepted physical branch. Unqualified result caching by only T/P or component names is insufficient.

A cache key or its guarded dependency record must account for all consequential material, parameter, reference, physical-policy, realization and coordinate choices. Approximate key rounding, tabulation or interpolation is permitted only as a declared numerical/model approximation whose error and derivative behavior are qualified. It is not exact evaluation merely because the user sees a familiar property name.

Parameter estimation does not violate function freezing when the parameters are explicitly declared coordinates of the fitting problem. Each evaluation uses its specified trial values and records them; fixed production data remain unchanged. The optimizer's coordinate chart, trial model and derivative with respect to those coordinates must be coherent. Approval of a fitted result occurs later through the P02/P03/P10 proposal path, not by modifying a shared live package during a function call.

### 9.5 Transition to a new solver mode is a new qualification

A direct-service profile can later gain a local-property callback, reduced sensitivity or exposed-residual realization. Existing evidence may be reused where its assumptions actually match, but the new mode adds obligations for trial-domain coverage, unknown ownership, derivative meaning, sparsity, regularity and accuracy. No direct-flash pass automatically certifies optimization mode.

Similarly, an exposed model's first derivative, an implicit solved sensitivity and a Hessian of the complete outer contribution are different evidence subjects. The gate record states which is supplied and which remains unavailable. A solver that can operate with a declared derivative approximation can use that route without falsely upgrading the provider's capability.

## 10. Traceability into the existing blueprint

### 10.1 What is unchanged

All 94 functional IDs, original obligation classes, primary packages, primary actions and inherited acceptance-test links are retained. All 56 actions, 72 information concepts, 18 A-products, 108 relationship contracts, 28 semantic rules, 12 operation variants and 34 scenario profiles remain attached through the B7 register. These are references to the existing semantics, not replacement contracts or new schema choices.

The integration register adds governing IR rules, candidate IP profiles, solver modes and IE experiments. A requirement's primary action remains the B7 allocation. The IR relation identifies the integration decisions relevant to that action/family, not a transfer of semantic ownership.

| Source item | Count | Treatment |
| --- | --- | --- |
| Functional requirements | 94 | Original owner, class, primary action, information owner and witnesses retained |
| Actions | 56 | Every action has governing integration rules |
| Information concepts | 72 | Original action meaning preserved through unchanged B7 coverage |
| Information products | 18 | No new owner or mandatory physical representation |
| Operation specializations | 12 | New solver-mode qualification does not change physical intent |
| Scenarios | 34 | Original 14 P1 / 16 P2 / 4 P3 preserved |
| Inherited native probes | 38 | Every probe has an IE route; none promoted to executed |


### 10.2 Scenario-to-profile and acceptance routing

| Scenario / original profile | Candidate templates | Integration qualification or unresolved boundary |
| --- | --- | --- |
| SC-01 / P1 Blending and splitting without reaction | [IP-01](#ip-01) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-02 / P1 Sensible heating and heat exchange | [IP-01](#ip-01), [IP-04](#ip-04), [IP-09](#ip-09) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-03 / P1 Liquid pumping and pressure-loss calculations | [IP-01](#ip-01), [IP-04](#ip-04) | Pump/pipe hydraulic laws stay host-owned; demanded viscosity and phase restrictions must be qualified. |
| SC-04 / P1 Gas compression, expansion and intercooling | [IP-01](#ip-01) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-05 / P1 Cooling and vapor–liquid separation | [IP-01](#ip-01) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-06 / P1 Pressure reduction with flashing | [IP-01](#ip-01) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-07 / P1 Conventional equilibrium-stage distillation | [IP-01](#ip-01), [IP-13](#ip-13) | A property provider is not a complete column; stage equations and convergence require SM-02/04/05 or explicitly qualified SM-03 blocks. |
| SC-08 / P1 Nonideal-liquid separation and azeotropic behavior | [IP-02](#ip-02) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-09 / P1 Physical absorption, humidification and gas dissolution | [IP-02](#ip-02), [IP-13](#ip-13) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-10 / P2 Liquid–liquid extraction and decanting | [IP-03](#ip-03) | Requires a liquid-split model and reference data, not two identical liquid entries. |
| SC-11 / P2 Vapor–liquid–liquid separation | [IP-03](#ip-03) | Qualify TP phase search independently from any inverse PH/PS variant. |
| SC-12 / P1 Water and steam through saturation | [IP-04](#ip-04) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-13 / P1 Pure-fluid refrigeration loop | [IP-05](#ip-05) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-14 / P2 Mixed-refrigerant phase change | [IP-06](#ip-06) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-15 / P2 Assay-derived petroleum pseudocomponents | [IP-07](#ip-07) | Complete raw-assay-to-thermal workflow remains open, not closed by custom cubic component construction. |
| SC-16 / P2 Black-oil or other reduced petroleum representation | [IP-08](#ip-08) | Numerical black-oil provider and caloric surface not established; program-level P2 gap retained. |
| SC-17 / P1 Specified-conversion reaction | [IP-10](#ip-10) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-18 / P1 Chemical-equilibrium reaction | [IP-11](#ip-11), [IP-12](#ip-12) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-19 / P2 Kinetically controlled reaction | [IP-10](#ip-10) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-20 / P2 Reactive separation | [IP-02](#ip-02), [IP-03](#ip-03), [IP-12](#ip-12), [IP-13](#ip-13) | Use a combined reactive/staged contract; no standalone gamma–phi plus chemistry success promotion. |
| SC-21 / P2 Electrolyte mixing and neutralization | [IP-11](#ip-11), [IP-12](#ip-12) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-22 / P2 Reactive gas absorption into aqueous liquid | [IP-11](#ip-11), [IP-12](#ip-12), [IP-13](#ip-13) | Energy and transport requirements remain mandatory for the selected absorber formulation. |
| SC-23 / P2 Inert solids carried with fluid | [IP-09](#ip-09) | Inert carried material does not enter equilibrium merely because it is labeled solid. |
| SC-24 / P2 Crystallization and precipitation | [IP-11](#ip-11), [IP-12](#ip-12) | Equilibrium amount does not predict crystal size, nucleation kinetics or actual yield dynamics. |
| SC-25 / P2 Gas–solid chemical transformation | [IP-10](#ip-10), [IP-11](#ip-11), [IP-12](#ip-12) | Requires actual instantiated model/data and selected solver-mode evidence plus its original unit and lifecycle witnesses. |
| SC-26 / P2 Rate-based nonequilibrium contacting | [IP-13](#ip-13) | Full nonequilibrium bulk/interface model remains host-owned and not yet numerically qualified. |
| SC-27 / P3 Adsorption and membrane state extensions | [IP-14](#ip-14) | Independent representability review only; numerical execution remains deferred. |
| SC-28 / P1 Heat exchange between different property packages | [IP-01](#ip-01), [IP-04](#ip-04), [IP-05](#ip-05) | CM-08/IR-10 couples separate profiles by heat; no shared absolute enthalpy zero or material map required. |
| SC-29 / P2 Material transfer across a property-package boundary | [IP-01](#ip-01), [IP-02](#ip-02), [IP-04](#ip-04), [IP-07](#ip-07), [IP-08](#ip-08), [IP-11](#ip-11) | CM-09/IR-09/10 directed source-target convention/constraint agreement is mandatory. |
| SC-30 / P2 Translation between material representations | [IP-07](#ip-07), [IP-08](#ip-08), [IP-11](#ip-11) | CM-10/IR-04/10 mapping records conservation and lost information; source/target profiles are illustrative pairs. |
| SC-31 / P2 Mass-based empirical and nonconventional materials | [IP-09](#ip-09) | Mass-only thermal capability has no implied molecular/fugacity/elemental completion. |
| SC-32 / P3 Polymer distributions and material attributes | [IP-14](#ip-14) | P3 distribution accounting/reduction only; no invented unique inverse. |
| SC-33 / P3 Inventory-based state and dynamic compatibility | [IP-14](#ip-14) | P3 inventory semantics retained even if IP-11 or another provider can later supply a UV candidate. |
| SC-34 / P3 Restricted-equilibrium and metastable-state requests | [IP-14](#ip-14) | P3 restricted/metastable scope; a forced numerical phase is not a stability certificate. |


### 10.3 Functional requirements and integration-rule navigation

| Requirement | Original owner / class | Primary action retained | Governing integration rules |
| --- | --- | --- | --- |
| [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01) | P09 / H | [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38) | [IR-01](#ir-01), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02) | P08 / H | [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27) | [IR-01](#ir-01), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03) | P08 / H | [AC-27](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-27) | [IR-01](#ir-01), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04) | P03 / H | [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07) | [IR-01](#ir-01), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-GOV-05](thermodynamics_functional_requirements_v0_1.md#fr-gov-05) | P09 / H | [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38) | [IR-01](#ir-01), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01) | P01 / H | [AC-01](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-01) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02) | P01 / H | [AC-02](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-02) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03) | P04 / H | [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04) | P04 / H | [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05) | P04 / H | [AC-14](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-14) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06) | P04 / H | [AC-15](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-15) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07) | P01 / H | [AC-02](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-02) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08) | P01 / H | [AC-02](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-02) | [IR-04](#ir-04), [IR-21](#ir-21), [IR-23](#ir-23) |
| [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01) | P02 / H | [AC-04](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-04) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02) | P02 / H | [AC-04](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-04) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03) | P02 / H | [AC-05](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-05) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04) | P02 / C | [AC-06](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-06) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05) | P02 / C | [AC-06](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-06) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06) | P02 / H | [AC-06](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-06) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07) | P02 / H | [AC-04](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-04) | [IR-03](#ir-03), [IR-05](#ir-05), [IR-06](#ir-06), [IR-22](#ir-22) |
| [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01) | P03 / H | [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02) | P03 / H | [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03) | P03 / H | [AC-09](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-09) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04) | P03 / H | [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05) | P03 / H | [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06) | P07 / H | [AC-19](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-19) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07) | P03 / H | [AC-07](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-07) | [IR-01](#ir-01), [IR-02](#ir-02), [IR-05](#ir-05), [IR-06](#ir-06), [IR-09](#ir-09), [IR-14](#ir-14), [IR-17](#ir-17) |
| [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01) | P04 / H | [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02) | P05 / H | [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03) | P05 / H | [AC-21](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-21) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04) | P04 / H | [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05) | P04 / H | [AC-16](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-16) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06) | P04 / H | [AC-16](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-16) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07) | P04 / H | [AC-15](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-15) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08) | P04 / H | [AC-17](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-17) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-16](#ir-16), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01) | P05 / C | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02) | P05 / C | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03) | P05 / C | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04) | P05 / C | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05) | P05 / H | [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06) | P05 / H | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07) | P05 / H | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08) | P05 / C | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-01](#ir-01), [IR-05](#ir-05), [IR-08](#ir-08), [IR-12](#ir-12), [IR-14](#ir-14), [IR-15](#ir-15), [IR-20](#ir-20) |
| [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01) | P05 / C | [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02) | P05 / H | [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03) | P05 / C | [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04) | P05 / C | [AC-22](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-22) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05) | P05 / H | [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06) | P05 / C | [AC-25](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-25) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07) | P03 / H | [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08) | [IR-07](#ir-07), [IR-11](#ir-11), [IR-12](#ir-12), [IR-13](#ir-13), [IR-20](#ir-20) |
| [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01) | P06 / C | [AC-10](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-10) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02) | P06 / C | [AC-11](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-11) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03) | P05 / C | [AC-24](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-24) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04) | P06 / C | [AC-10](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-10) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05) | P03 / H | [AC-08](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-08) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06) | P06 / C | [AC-12](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-12) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07) | P06 / C | [AC-10](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-10) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08) | P09 / C | [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37) | [IR-04](#ir-04), [IR-08](#ir-08), [IR-09](#ir-09), [IR-16](#ir-16), [IR-20](#ir-20) |
| [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01) | P07 / H | [AC-20](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-20) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02) | P07 / C | [AC-39](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-39) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03) | P07 / C | [AC-40](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-40) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04) | P07 / C | [AC-41](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-41) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05) | P07 / C | [AC-42](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-42) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06) | P07 / C | [AC-42](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-42) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07) | P07 / C | [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08) | P07 / C | [AC-43](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-43) | [IR-07](#ir-07), [IR-10](#ir-10), [IR-12](#ir-12), [IR-13](#ir-13), [IR-16](#ir-16), [IR-21](#ir-21) |
| [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01) | P08 / H | [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02) | P08 / H | [AC-31](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-31) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03) | P08 / H | [AC-30](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-30) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04) | P08 / H | [AC-30](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-30) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05) | P08 / H | [AC-34](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-34) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06) | P08 / H | [AC-35](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-35) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07) | P08 / H | [AC-35](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-35) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08) | P07 / H | [AC-44](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-44) | [IR-12](#ir-12), [IR-15](#ir-15), [IR-17](#ir-17), [IR-18](#ir-18), [IR-19](#ir-19), [IR-21](#ir-21) |
| [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01) | P09 / H | [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02) | P09 / H | [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03) | P09 / H | [AC-36](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-36) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04) | P09 / H | [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05) | P10 / H | [AC-49](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-49) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06) | P09 / H | [AC-56](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-56) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07) | P09 / H | [AC-37](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-37) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08) | P09 / H | [AC-38](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-38) | [IR-20](#ir-20), [IR-21](#ir-21), [IR-22](#ir-22), [IR-24](#ir-24) |
| [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01) | P10 / H | [AC-48](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-48) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02) | P10 / H | [AC-47](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-47) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03) | P10 / H | [AC-47](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-47) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04) | P10 / H | [AC-50](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-50) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05) | P10 / H | [AC-51](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-51) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06) | P10 / H | [AC-52](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-52) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07) | P10 / H | [AC-53](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-53) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08) | P10 / H | [AC-48](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-48) | [IR-03](#ir-03), [IR-17](#ir-17), [IR-21](#ir-21), [IR-22](#ir-22) |
| [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01) | P01 / R3 | [AC-03](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-03) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-23](#ir-23), [IR-24](#ir-24) |
| [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02) | P01 / R3 | [AC-03](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-03) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-23](#ir-23), [IR-24](#ir-24) |
| [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03) | P04 / R3 | [AC-13](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-13) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-23](#ir-23), [IR-24](#ir-24) |
| [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04) | P05 / R3 | [AC-23](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md#ac-23) | [IR-04](#ir-04), [IR-14](#ir-14), [IR-23](#ir-23), [IR-24](#ir-24) |


### 10.4 Inherited native probes remain unexecuted

| Prior probe | New experiment routes | Status |
| --- | --- | --- |
| B2:P01 | [IE-01](#ie-01) | Not executed |
| B2:P02 | [IE-01](#ie-01), [IE-17](#ie-17) | Not executed |
| B2:P03 | [IE-13](#ie-13) | Not executed |
| B2:P04 | [IE-14](#ie-14) | Not executed |
| B2:P05 | [IE-03](#ie-03), [IE-05](#ie-05) | Not executed |
| B2:P06 | [IE-15](#ie-15) | Not executed |
| B2:P07 | [IE-03](#ie-03), [IE-09](#ie-09) | Not executed |
| B2:P08 | [IE-15](#ie-15) | Not executed |
| B2:P09 | [IE-17](#ie-17) | Not executed |
| B2:P10 | [IE-15](#ie-15) | Not executed |
| B2:P11 | [IE-01](#ie-01), [IE-16](#ie-16), [IE-17](#ie-17) | Not executed |
| B2:P12 | [IE-17](#ie-17) | Not executed |
| B2:P13 | [IE-13](#ie-13) | Not executed |
| B2:P14 | [IE-15](#ie-15) | Not executed |
| B2:P15 | [IE-10](#ie-10) | Not executed |
| B2:P16 | [IE-01](#ie-01), [IE-04](#ie-04) | Not executed |
| B2:P17 | [IE-11](#ie-11), [IE-14](#ie-14) | Not executed |
| B2:P18 | [IE-17](#ie-17) | Not executed |
| B3:PV-01 | [IE-01](#ie-01), [IE-17](#ie-17), [IE-18](#ie-18) | Not executed |
| B3:PV-02 | [IE-01](#ie-01), [IE-17](#ie-17) | Not executed |
| B3:PV-03 | [IE-03](#ie-03), [IE-09](#ie-09) | Not executed |
| B3:PV-04 | [IE-03](#ie-03) | Not executed |
| B3:PV-05 | [IE-04](#ie-04) | Not executed |
| B3:PV-06 | [IE-04](#ie-04), [IE-09](#ie-09) | Not executed |
| B3:PV-07 | [IE-02](#ie-02) | Not executed |
| B3:PV-08 | [IE-02](#ie-02), [IE-16](#ie-16) | Not executed |
| B3:PV-09 | [IE-13](#ie-13) | Not executed |
| B3:PV-10 | [IE-03](#ie-03), [IE-04](#ie-04) | Not executed |
| B3:PV-11 | [IE-02](#ie-02), [IE-04](#ie-04) | Not executed |
| B3:PV-12 | [IE-13](#ie-13) | Not executed |
| B3:PV-13 | [IE-04](#ie-04) | Not executed |
| B3:PV-14 | [IE-06](#ie-06), [IE-07](#ie-07), [IE-08](#ie-08), [IE-18](#ie-18) | Not executed |
| B3:PV-15 | [IE-08](#ie-08) | Not executed |
| B3:PV-16 | [IE-01](#ie-01), [IE-17](#ie-17) | Not executed |
| B3:PV-17 | [IE-05](#ie-05), [IE-06](#ie-06), [IE-10](#ie-10) | Not executed |
| B3:PV-18 | [IE-10](#ie-10) | Not executed |
| B3:PV-19 | [IE-12](#ie-12) | Not executed |
| B3:PV-20 | [IE-05](#ie-05), [IE-11](#ie-11), [IE-13](#ie-13), [IE-14](#ie-14), [IE-15](#ie-15) | Not executed |


## 11. Open items, resolved policy and remaining evidence

| Inherited item / owner | Step-8 decision | Remaining evidence |
| --- | --- | --- |
| PO-01 / P04 Phase correspondence | IR-04, IR-14; IP-03/13. Choose a profile-specific correspondence/layout strategy. Ambiguity blocks only dependent routing/continuation claims; cross-phase-count solver layout changes require explicit restructuring or qualified fixed-superset formulation. | Actual matching thresholds, physical phase witnesses and near-coalescence behavior remain to test. |
| PO-02 / P03 Reference and reactive energy compatibility | IR-05, IR-09, IR-10; CM-04/09/12. Complete calorics must be inside inverse equations. Directed reference transformation, reaction correction and genuine energy exchange are separate; chemical gauges are not arbitrary property offsets. | Actual data/standard-state transformations for each source-target pair remain uninstantiated. |
| PO-03 / P05 Coupled equation contribution detail | SM-01–05; IR-11–16. Fix unknown ownership and local/total derivative semantics, implicit response, chart/Hessian/scaling rules, inner accuracy and stable per-solve layout. | Exact executable contributions, derivative support, active-set regularity and chosen host solver still need qualification. |
| PO-04 / P08 Session containment | IR-18/19; IE-13. Default to no unverified concurrent sharing; lease full sequences; fatal native failure needs real containment; cancellation and native lifetime remain distinct. | Exact build reentrancy, containment mechanism, nested calls and recovery tests remain required. |
| PO-05 / P10 Dependency invalidation granularity | IR-17/21/22; IE-15/17. Capture full function and input lineage, not just package name; distinguish explicit variable coordinates from fixed coefficients. Approximate caches are declared approximations; currentness remains guarded. | Dependency-minimal implementation and atomic persistence mechanism remain future choices; conservative invalidation is permitted. |
| PO-06 / P02 Petroleum and empirical completion | IP-07/08/09; IE-16. DWSIM characterization/reduced workflows are explicit investigation targets; partial pseudocomponent and mass-only capabilities do not imply full energy or chemistry. | Real assays, black-oil route, empirical property data and complete native numerical workflow remain open P2 gates. |
| PO-07 / P03 Reactive transport completion | IR-07/08/16; IP-10–13; IE-10/11. Coherent reacting problem plus phase-only transport augmentation and host bulk/interface equations; reject separate incompatible chemistry/flash outputs as coupled success. | Actual caloric/speciation/diffusion/rate models, phase forms and unit datasets remain unqualified. |
| PO-08 / P09 Acceptance evidence and check independence | IR-12/20/24; QG-07–09. Require original residuals, branch/authority checks and inner-error/derivative budgets; same-model consistency checks remain distinct from independent physical validation. | Real-fluid tolerances, independence and measured repeatability/conditioning still need fixed evidence. |
| PO-09 / P08 Provider manifests and opaque internals | IR-01/22; IC-53/58/68. Keep earlier source pins, current document tracks and eventual build/data manifests separate. Opaque attributable bundles permitted with bounded claims. | Actual runtime artifacts, optional data availability and distribution conditions still require verification before use. |
| PO-10 / P01 Extension information semantics | IR-23; IP-14; IE-18. Extensions retain concrete account/transfer/reduction semantics and possible future solver arrangements without forcing molecular placeholders. | Independent P3 representability review remains pending; no numerical commitment or automatic R/E/V promotion added. |


The completion criterion for Step 8 is met as an integration design: each proposed composition has an explicit admission contract, forbidden shortcut and validation route; solver modes state unknown/derivative/branch responsibilities; each scenario has a provisional profile route or an explicit numerical gap. It is **not** a claim that the actual runtime prerequisites in the rightmost column have already been demonstrated.

No additional Rust types, FFI signatures, storage design, mandatory numerical solver, module graph or deployment topology is chosen. Those should be derived from the qualified profiles and the already established semantic/action boundaries, not from whatever wrapper happens to be easiest to call.

## 12. Authored checks, catalog audit and evidence limits

The included `reference_integration_tests.py` uses only the Python standard library. It evaluates authored algebraic examples and selected sequential compatibility/lifetime/publication guards. It is not the simulator, an actual native adapter, a nonlinear solver or a concurrent implementation. All examples are synthetic unless explicitly described as inherited research.

**Structural result:** 42 of 42 catalog/link checks passed. **Authored reference result:** 43 of 43 passed.

| Deliverable | Count |
| --- | --- |
| Integration rules | 24 |
| Reuse boundaries | 5 |
| Solver modes | 5 |
| Composition assessments | 18 |
| Profile templates | 14 |
| Qualification gates | 10 |
| Native/formulation experiment specifications | 18 |
| Requirements preserved | 94 |
| Actions covered | 56 |
| Scenarios retained | 34 |
| Authored reference checks executed | 43 |
| Native numerical tests executed | 0 |
| Scenario R/E/V promotions | 0 |


| Check | Expected / obtained | Result |
| --- | --- | --- |
| RC8-01 Implicit state response | 1/4 / 1/4 | PASS |
| RC8-02 Reduced derivative includes inner response | 4 / 4 | PASS |
| RC8-03 Reduced second derivative | 3/8 / 3/8 | PASS |
| RC8-04 Frozen-y derivative rejected | False / False | PASS |
| RC8-05 Reduced derivative matches full re-solve central perturbation | 4.0 / 3.9999999999196234 | PASS |
| RC8-06 Linearized outer sensitivity to inner residual | 3 / 3 | PASS |
| RC8-07 Inner error magnification example | 3/1000000 / 3/1000000 | PASS |
| RC8-08 Noise amplification for tiny perturbation | 100 / 100 | PASS |
| RC8-09 Hessian chart first term alone | 32 / 32 | PASS |
| RC8-10 Hessian chart curvature contribution | 16 / 16 | PASS |
| RC8-11 Full nonlinear chart Hessian | 48 / 48 | PASS |
| RC8-12 Scaled Jacobian | 30 / 30 | PASS |
| RC8-13 Zero at a point is not structural independence | True / True | PASS |
| RC8-14 Partial caloric solve temperature | 325 / 325 | PASS |
| RC8-15 Completed enthalpy at wrong partial solution | 125 / 125 | PASS |
| RC8-16 Total caloric inverse temperature | 310 / 310 | PASS |
| RC8-17 Original complete target satisfied | 50 / 50 | PASS |
| RC8-18 Discontinuous bracket has opposite residual signs | True / True | PASS |
| RC8-19 Discontinuous endpoint has nonzero target residual | 50 / 50 | PASS |
| RC8-20 Explicit phase fraction resolves synthetic endpoint mixture | 50 / 50 | PASS |
| RC8-21 Parameter match at one temperature | 1 / 1 | PASS |
| RC8-22 Parameter slope differs | 1/300 / 1/300 | PASS |
| RC8-23 Equimolar reference shift | 25 / 25 | PASS |
| RC8-24 Unequal-composition reference shift | 35/2 / 35/2 | PASS |
| RC8-25 Reaction reference contribution | 12 / 12 | PASS |
| RC8-26 Compensating reaction correction preserves accounting | 100 / 100 | PASS |
| RC8-27 Conserved-basis chemical gauge leaves reaction affinity unchanged | 0 / 0 | PASS |
| RC8-28 Arbitrary species offset changes reaction affinity | 30 / 30 | PASS |
| RC8-29 Model discrepancy after known offset | 10 / 10 | PASS |
| RC8-30 Temperature for corrected enthalpy match | 1025/3 / 1025/3 | PASS |
| RC8-31 Both Jacobian axes permuted | [[4, 3], [2, 1]] / [[4, 3], [2, 1]] | PASS |
| RC8-32 Disjoint phase material count | 10 / 10 | PASS |
| RC8-33 Lossy map lacks unique inverse | True / True | PASS |
| RC8-34 Required second liquid blocked | False / False | PASS |
| RC8-35 Local derivative does not qualify total response | False / False | PASS |
| RC8-36 Value request can use same eligible profile | True / True | PASS |
| RC8-37 Partial caloric PH is blocked | False / False | PASS |
| RC8-38 Running session not reused after logical cancellation | False / False | PASS |
| RC8-39 Quarantined session not reused | False / False | PASS |
| RC8-40 Same revision with revoked parent blocks publication | False / False | PASS |
| RC8-41 Same revision with stale iterative input blocks publication | False / False | PASS |
| RC8-42 Fully matching synthetic publication eligibility | True / True | PASS |
| RC8-43 Initializer not restored cannot claim original model | False / False | PASS |


The structural validator checks IDs, required content, mapping endpoints, baseline ownership, source hashes, scope/profile preservation, probe coverage and local document links. It does not prove universal thermodynamic sufficiency, real implementation atomicity, numerical smoothness, provider reentrancy or physical accuracy.

A numeric native fixture has not been executed merely because its test plan is complete. The existing positive/negative witnesses, provider probes and R/E/V statuses retain their previous state. The next validation stage can now instantiate and test a concrete profile against these rules without redefining material or operation semantics per library.

## 13. Source register and artifact use

### 13.1 Unchanged project baselines

| Key | Exact unchanged file | SHA-256 |
| --- | --- | --- |
| B1 | [thermodynamics_simulation_behavior_scope_v0_1.md](thermodynamics_simulation_behavior_scope_v0_1.md) | `1d12284bbca6569f198caaae86327788bac8fbaf3946625d99d971ef3602d028` |
| B2 | [dwsim_workflow_reverse_engineering_v0_1.md](dwsim_workflow_reverse_engineering_v0_1.md) | `55e77b6fddab4547280e4e40ca5c742fac3fd322e45534767ddae7f5f6ba94ab` |
| B3 | [thermodynamic_package_comparative_research_v0_1.md](thermodynamic_package_comparative_research_v0_1.md) | `c0f9216c450405360b8532771e6c4beca6536154345ea5adf00130a9c4e73875` |
| B4 | [thermodynamics_functional_requirements_v0_1.md](thermodynamics_functional_requirements_v0_1.md) | `7b9c3c4e2110531cb5767fb00f9ba11dec160294b8cafd909c5b04984a87b5c2` |
| B4-register | [thermodynamics_functional_requirements_v0_1_register.json](thermodynamics_functional_requirements_v0_1_register.json) | `1777f2c60962526252aa863dc2cb673f218b4ed58fe9c108f9721264f9fde825` |
| B5 | [thermodynamics_conceptual_packaging_v0_1.md](thermodynamics_conceptual_packaging_v0_1.md) | `85602e0a750add6a1168cb4f4294998feb5e90e35b37cef5f0b4e63c5f1076f1` |
| B5-register | [thermodynamics_conceptual_packaging_v0_1_register.json](thermodynamics_conceptual_packaging_v0_1_register.json) | `d46ce687dee20cb7e677082039f349a38d8fd71101a659079d47bb8d57d80612` |
| B6 | [thermodynamics_semantic_information_dictionary_v0_1.md](thermodynamics_semantic_information_dictionary_v0_1.md) | `58c33ae37009c2c07485a46ae95fdd51f0475504aaa75cb85ef7a23a8484dee9` |
| B6-register | [thermodynamics_semantic_information_dictionary_v0_1_register.json](thermodynamics_semantic_information_dictionary_v0_1_register.json) | `ab459185e0536ce5ef721c572010ed8d1279f89f8d6ad632b32f927b03f21f8d` |
| B7 | [thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md](thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md) | `d7624eb730cf3ccc6d65ec2bf21658319ad0d44e0366a4182cecdc29f0448121` |
| B7-register | [thermodynamics_action_workflow_lifecycle_blueprint_v0_1_register.json](thermodynamics_action_workflow_lifecycle_blueprint_v0_1_register.json) | `ddcd4239f165ee7425d8c668aa2b5b824ffee12edfaf73ded0f2748df74627f6` |


B1 supplies SC/X/C and P1/P2/P3 scope. B2/B3 supply earlier pinned workflow/library evidence. B4 is the normative behavior baseline, B5 the responsibility allocation, B6 the semantic dictionary, and B7 the action/lifecycle contracts. The bundle includes their unchanged documents and available registers so the cross-references can be inspected together.

### 13.2 Targeted official-documentation checks

The following were read for bounded interface/formulation facts on 25 September 2026. The source track is part of the citation. Moving documentation does not identify a tested binary or retroactively update a historical source pin. A compatibility proposal in this document is our design inference, not a statement that the upstream library already implements our IC/AC/IR contracts.

<a id="e01"></a>
#### E01. Ipopt problem assumptions

**Track:** Moving official documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** The stated NLP formulation expects twice continuously differentiable objective and constraint functions; a local solver is not a certificate of global thermodynamic stability.

**Source:** [Ipopt problem assumptions](https://coin-or.github.io/Ipopt/)

<a id="e02"></a>
#### E02. Ipopt standard interfaces

**Track:** Moving official documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** The standard interface requests function values, gradients/Jacobian and a weighted Lagrangian Hessian unless a quasi-Newton option is used. Its structural nonzero pattern must remain fixed during a solve.

**Source:** [Ipopt standard interfaces](https://coin-or.github.io/Ipopt/INTERFACES.html)

<a id="e03"></a>
#### E03. Ipopt special features

**Track:** Moving official documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Documents derivative checking and limited-memory Hessian approximation. This is not evidence that an arbitrary value-only or nonsmooth external flash satisfies the solver contract.

**Source:** [Ipopt special features](https://coin-or.github.io/Ipopt/SPECIALS.html)

<a id="e04"></a>
#### E04. Pyomo external reduced model

**Track:** Explicit version 6.9.5. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Describes eliminating external variables through inner equations and evaluating reduced residuals and derivatives. It is a formulation reference, not a recommendation to make Pyomo the runtime.

**Source:** [Pyomo external reduced model](https://pyomo.readthedocs.io/en/6.9.5/api/pyomo.contrib.pynumero.interfaces.external_pyomo_model.ExternalPyomoModel.html)

<a id="e05"></a>
#### E05. Pyomo external grey-box model

**Track:** Explicit version 6.9.2. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Defines external input/output/equality interfaces and multiplier-weighted second-derivative contributions when supplied.

**Source:** [Pyomo external grey-box model](https://pyomo.readthedocs.io/en/6.9.2/api/pyomo.contrib.pynumero.interfaces.external_grey_box.ExternalGreyBoxModel.html)

<a id="e06"></a>
#### E06. IDAES smooth vapor-liquid formulation

**Track:** Moving stable documentation; page identifies v2.13.0. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Uses auxiliary equilibrium temperatures and smoothing in a vapor-liquid formulation. A formulation-specific auxiliary state must remain distinct from an actual material temperature.

**Source:** [IDAES smooth vapor-liquid formulation](https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/pe/smooth_flash.html)

<a id="e07"></a>
#### E07. IDAES translator

**Track:** Moving stable documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Translation between property descriptions requires application-specific linking constraints; construction of two state blocks does not supply universal physical reconciliation.

**Source:** [IDAES translator](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/translator.html)

<a id="e08"></a>
#### E08. thermo phase models

**Track:** Moving documentation; page identifies 0.6.1. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** GibbsExcessLiquid distinguishes equilibrium and caloric conventions. Documentation also identifies an Hvap caloric approximation as inconsistent but commonly used; approximation must not be conflated with exact consistency.

**Source:** [thermo phase models](https://thermo.readthedocs.io/thermo.phases.html)

<a id="e09"></a>
#### E09. thermo phase and flash tutorial

**Track:** Moving documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Warns that incompatible phase-model combinations can produce discontinuities or missing solutions. Flexible assembly is not automatic compatibility certification.

**Source:** [thermo phase and flash tutorial](https://thermo.readthedocs.io/tutorial_phases_and_flash.html)

<a id="e10"></a>
#### E10. thermo flash families

**Track:** Moving documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Pure, single-liquid and multiple-liquid flash families have different intended scope. Existing B3 pinned-source restrictions remain authoritative only for their inspected versions.

**Source:** [thermo flash families](https://thermo.readthedocs.io/thermo.flash.html)

<a id="e11"></a>
#### E11. Clapeyron flash compatibility

**Track:** Moving stable documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Model family, phase count, specification pair and initializer coverage vary by selected algorithm. An algorithm-internal derivative requirement is not automatically an exposed derivative of its solved output.

**Source:** [Clapeyron flash compatibility](https://clapeyronthermo.github.io/Clapeyron.jl/stable/properties/flash/)

<a id="e12"></a>
#### E12. CoolProp low-level interface

**Track:** Moving official documentation; page identifies 8.0.0. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Reusable backend objects have phase/derivative restrictions. Reference changes do not retroactively affect existing state instances; actual build and session behavior require qualification.

**Source:** [CoolProp low-level interface](https://coolprop.org/coolprop/LowLevelAPI.html)

<a id="e13"></a>
#### E13. Reaktoro equilibrium solver

**Track:** Generated official API documentation, not a pinned deployment build. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** The interface distinguishes conditions, restrictions, mutable initial/final state and equilibrium sensitivities. The coherent chemical-system boundary is the candidate integration unit.

**Source:** [Reaktoro equilibrium solver](https://reaktoro.org/api/classReaktoro_1_1EquilibriumSolver.html)

<a id="e14"></a>
#### E14. ThermoPack state/property methods

**Track:** Moving vcurrent beta documentation. **Evidence:** official documentation, not executed here. **Accessed:** 2026-09-25.

**Bounded use:** Documents selected property and two-phase flash operations. No parity of every native and wrapper surface or general solid/multi-liquid inverse capability is inferred.

**Source:** [ThermoPack state/property methods](https://thermotools.github.io/thermopack/vcurrent/thermo_methods.html)



**Companions:** [integration register](thermodynamics_cross_library_solver_integration_v0_1_register.json), [structural audit](thermodynamics_cross_library_solver_integration_v0_1_audit.json), [authored reference checks](thermodynamics_cross_library_solver_integration_v0_1_reference_checks.json), [reference test script](reference_integration_tests.py), [structural validator](validate_integration_blueprint.py).

**Design thesis:** preserve coherent thermodynamic packages; compose only through explicit quantity, reference and physical contracts; select solver arrangements by their actual mathematical demands; and qualify the resulting configuration, not the upstream library name. That gives a broad flowsheet simulator leverage from mature engines without inheriting their hidden state, weakening energy accounting or promising derivatives and phase behavior they do not expose.
