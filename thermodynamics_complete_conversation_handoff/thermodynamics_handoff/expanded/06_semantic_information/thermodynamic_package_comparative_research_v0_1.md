# Comparative thermodynamic-framework and library research

**Document:** THERMO-LIBRARIES-003  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 3, compare the other packages against the DWSIM workflow questions  
**Status:** Evidence-backed conceptual research baseline; not a runtime validation or backend selection

**Inputs:** THERMO-SCOPE-001 v0.1 and THERMO-DWSIM-002 v0.1. Their 34 scenarios, 24 cross-cutting requirements and calculation-authority distinction remain the baseline. This document does not silently expand or reduce those commitments.

## 0. Executive findings

The packages do not implement the same abstraction. CAPE-OPEN specifies interactions; IDAES constructs equation-oriented property and reaction submodels; ChEDL and Clapeyron assemble methods and equilibrium algorithms; ThermoPack and CoolProp provide numerically substantial but interface-specific state/property services; FeOS organizes potential-based models, states and derivatives; Reaktoro organizes chemically reactive systems and their constrained calculations. The DWSIM baseline remains the reference for the surrounding flowsheet lifecycle. [CO01,ID01,CL01,CH01,TP02,CP01,FE01,RK01]

**Recommendation:** retain one user-facing configured thermodynamic package, but permit several internal integration granularities. A package must declare which material description, physical assumptions, required data, property operations, equilibrium operations and calculation lifecycle it actually supports. It must not be identified merely by an EOS name or an upstream library.

The most important revisions to the initial hypotheses are:

1. **ChEDL `thermo` is more than a data/correlation source.** Its modern phase and flash framework is a serious candidate for assembled conventional property packages. Its older `Stream` convenience interface should not define our assessment. [CH01,CH04–CH06]
2. **Clapeyron's model composition is directly relevant to mainstream process packages**, not just advanced EOS research. Its explicit flash compatibility matrix is equally important: broad model and algorithm catalogs do not imply every combination works. [CL01,CL02]
3. **ThermoPack remains a useful EOS and fluid-state candidate, not a predetermined general-process foundation.** Runtime cubic pseudocomponents exist, but the inspected example expressly lacks their ideal-gas heat capacities. Its documented ordinary wrapper flashes are two-phase. Internal multiphase capability does not automatically give a general externally exposed multi-liquid PH service. [TP02–TP05]
4. **IDAES has actual eNRTL implementation**, not merely electrolyte-aware placeholders. Its equation and eligibility contracts are the primary lesson; usable configured chemical systems still need their data and closure choices. [ID01,ID03]
5. **Current CoolProp source contains mixture PH/PS/PU solving paths.** The older blanket claim that mixtures only support TP/PQ/TQ is not valid for the inspected source. This is source evidence, not a guarantee for a particular binary release, backend or mixture. [CP03]
6. **FeOS has real PH/PS and implicit-equilibrium derivative implementations.** Their two-phase initialization preconditions are material integration constraints. Its scientific architecture is useful even though its present workflow scope should not define a general simulator's boundary. [FE01,FE02]
7. **Reaktoro is broader than the DWSIM adapter inspected in Step 2.** Upstream exposes generalized constraints, restrictions, equilibrium sensitivities and kinetics. These belong to a coherent chemical-system provider, not an unqualified drop-in replacement for any fluid flash. [RK01–RK05]
8. **CAPE-OPEN clarifies authority and energy semantics.** Phase equilibrium does not imply that every property is already populated, and reaction energy correction is not simply another name for the whole heat of reaction. [CO02,CO03]

## 1. Method, evidence levels and limits

The research applies the same questions to eight reference families. ChEDL `thermo` and `chemicals` are one family but remain separate dependencies. Source snapshots for eight repositories are pinned below. Official documentation supplements those snapshots and is labeled separately because `stable`, `vcurrent`, generated API pages and repository default branches can describe different points in development.

Evidence labels in this report:

| Label | Meaning | Does not establish |
|---|---|---|
| **S** | A relevant source implementation or public interface was inspected at the recorded commit. | Successful build, tested execution, correct numerics, complete feature coverage or thread safety. |
| **D** | The behavior or limitation is documented by the project or standards body. | An exercised integration or proof that every shipped build matches the page. |
| **U** | The specific behavior has not been established by this investigation. | Absence of the feature from the project. |
| **P** | A proposed architectural treatment, acceptance rule or verification task. | An existing package capability. |

For CAPE-OPEN, normative text is specification evidence rather than code evidence. An interface argument, method name or abstract base class is not treated as a completed implementation. Negative claims are limited to explicit restrictions or inspected branches.

**No numerical probes were executed.** The available runtime did not contain the candidate Python packages or Julia; the attempted package-index preflight failed name resolution. Public documentation and source retrieval remained available. No installation failure is attributed to a library. Accordingly, this work does not promote any Step-1 scenario to Executable or Validated. The probe plan in Section 11 specifies the remaining empirical checks.

This is not an exhaustive audit of every model, parameter database, test suite, API binding, native dependency, release artifact or failure path. Where a workflow question is unresolved, the dossier states the boundary rather than converting incomplete evidence into a feature claim.

### 1.1 The common questions

| ID | Question applied to each family | Required architectural answer |
|---|---|---|
| Q01 | What is the reusable unit: standard, phase model, configured package, state service, chemical system or equation submodel? | Identify a safe integration boundary, not just the language or model name. |
| Q02 | How are compounds, species, composition coordinates, extensive amounts and alternate representations described? | Separate identity, ordering, basis and conservation semantics. |
| Q03 | How are phases declared, searched, constrained, identified and reported? | Separate candidate phases, present phases, aggregates and numerical placeholders. |
| Q04 | How are data loaded, combined, estimated, overridden and retained? | Establish resolved-data provenance and the meaning of missing values. |
| Q05 | How are equilibrium, ideal/caloric, transport and reference-state models made coherent? | Identify what a complete process package requires beyond phase splitting. |
| Q06 | What operations are actually exposed, with what authority over the supplied state? | Distinguish evaluation, redistribution, reaction and equation construction. |
| Q07 | What initialization, stability, phase-search and retry assumptions apply? | Make solver eligibility and branch policy explicit. |
| Q08 | How are chemical equilibrium, reactions, kinetics and open exchanges represented? | Identify conserved quantities and division from equipment models. |
| Q09 | Which derivatives exist and what do they hold fixed or re-equilibrate? | Separate phase derivatives, path derivatives and solved-state sensitivities. |
| Q10 | What is mutable, cached, global or reusable between calls? | Define session isolation and lifecycle tests. |
| Q11 | What happens on failure, parameter change, save/reload and result reuse? | Define acceptance, currency, reconstruction and compatibility responsibility. |
| Q12 | What maturity and public-availability evidence exists, and what remains unverified? | Distinguish documented design provisions, tested coverage and usable distribution. |

### 1.2 Source snapshots

All commit selections use a common reference cutoff of 25 September 2026, 18:57 UTC. They are research snapshots, not selected production releases.

| Family/repository | Commit | Commit date used |
|---|---|---|
| `IDAES/idaes-pse` | `5e51567b315a25a4ed0057b4a23afe02f1109482` | 24 September 2026 |
| `thermotools/thermopack` | `d68c794c7342bfc6938eb424a1fbb88b7780b738` | 26 February 2026 |
| `ClapeyronThermo/Clapeyron.jl` | `4319d4fb7288aacc7969ca212e0792a691c9564e` | 25 September 2026 |
| `CalebBell/thermo` | `2bb466e98439c2395a004e7095d07f9b97a5a0f0` | 13 July 2026 |
| `CalebBell/chemicals` | `e79047588b30cfabc564c79fb26d760c746877d7` | 3 August 2026 |
| `CoolProp/CoolProp` | `afce86ff977552663ca3a78d8ea318cc64dcbdfd` | 24 September 2026 |
| `feos-org/feos` | `1b956cf7c55631b9be73abf2806058c85797bb04` | 8 September 2026 |
| `reaktoro/reaktoro` | `f587235e692b7168faa272c8892a4c079a1f7dd9` | 2 February 2026 (committer date) |

The DWSIM comparison remains THERMO-DWSIM-002's `DanWBR/dwsim10` commit `a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3`. CAPE-OPEN references are the Thermodynamics 1.1 specification and errata plus the separately revised Chemical Reactions specification. Do not mix the latter's semantics into an assertion that all existing Thermodynamics 1.1 clients implement them.

## 2. Responsibility-level map

The following are recommended roles, not mutually exclusive layers or final module boundaries.

| Reference family | Most informative responsibility | Candidate runtime boundary | Main qualification |
|---|---|---|---|
| DWSIM, prior baseline | Full flowsheet context, engineering package configuration, units, product publication, lifecycle | Complete configured package or selected validated operation | Stateful context and optional integration/product boundaries remain explicit. |
| CAPE-OPEN | Interoperability, material authority, property/equilibrium and reaction contracts | Standards adapter | Supplies neither model equations nor parameter data. |
| IDAES | Equation-oriented local states, shared parameters, unit/property/reaction interaction | Property/reaction mathematical submodel | Requires model construction, specified constraints, initialization and solution machinery. |
| ChEDL thermo/chemicals | Explicit data/correlation assembly, phase evaluation, conventional flashes and result objects | Complete configured phase-and-flash service; separately, data/correlations | Flash families differ; multicomponent solids are not supplied by FlashVLN. |
| Clapeyron | Model composition, parameter structure and algorithm eligibility | Configured composite model plus approved solver; selected kernels | Initialization and model/flash compatibility need operation-specific checks. |
| ThermoPack | EOS computations, properties, derivatives and selected fluid-state algorithms | Configured EOS session and supported state operations | High-level ordinary flashes inspected are two-phase; pseudo caloric and global-state limits matter. |
| CoolProp | Backend/state lifecycle and specialized fluid properties | Complete selected backend and associated data | Backend and mixture-specific operation limits; public core does not include separately licensed REFPROP. |
| FeOS | Potential-based properties, local states and differentiated equilibrium | Complete supported EOS/ideal-model combination and phase operations | Mostly two-phase algorithms; total caloric and initialization prerequisites. |
| Reaktoro | Chemical-system definition, constrained speciation/equilibrium and chemistry sensitivity | Complete chemical-system calculation | Species mapping, caloric compatibility, transport and flowsheet embedding need explicit contracts. |

Sources: DWSIM baseline; [CO01–CO03,ID01–ID06,CH01–CH06,CL01–CL06,TP01–TP05,CP01–CP03,FE01–FE03,RK01–RK05].

**Interpretation:** a standard property request can be implemented by a cohesive external package, a model assembled inside another library, or equations embedded in the process problem. Those execution arrangements need not share internal objects. They do need compatible semantics for inputs, authoritative quantities, outcomes and limitations.

## 3. Comparative dossiers

### L01. CAPE-OPEN: contract reference, not a computational engine

**Definition, representation and composition (Q01–Q05).** The Thermodynamics 1.1 conceptual model separates material objects, compound/phase discovery, property routines, equilibrium routines and material context. These are interface responsibilities, not a mandatory internal class hierarchy. The specification is a strong check that our system can support both a material passing through a flowsheet port and material used inside a unit operation. It is not a universal specification for detailed petroleum characterization, polymers or arbitrary nonconventional solids. Those scope limitations must not be erased merely because an interface has a phase list. [CO01]

The separately revised Chemical Reactions specification adds compound slates and chemical/reaction interactions. A slate can distinguish alternative representations of the same underlying material, including representations needed by reactive systems. This should inform the relationship between reporting components and actual species, but does not provide an automatic inverse for an information-losing mapping. Every mapping still needs its own conservation and reversibility claims. [CO03,CO04]

**Operations, authority, initialization and chemistry (Q06–Q09).** The errata clarify a useful postcondition: completing an equilibrium operation establishes the prescribed state/phase-allocation outputs; it does not guarantee that all derived properties have also been evaluated. A subsequent enthalpy or transport request remains a separate responsibility. Specification checking should identify whether a provider accepts an operation, rather than treating every numerical failure as unsupported functionality. The standard does not give the client a universal solver, initialization strategy or proof of stable equilibrium. [CO02]

The reactions document also distinguishes the correction required to reconcile the material enthalpy convention with a reaction balance from the entire heat of reaction. This matters when some formation-energy contribution is already inside the material enthalpy. Our design must not blindly add a reaction heat on top of an independently defined caloric package. A reference convention, a species mapping and the reaction-balance convention belong together. [CO03]

**Context and lifecycle (Q10–Q11).** Material association is an explicit interaction: the context must be refreshed if the material object or its relevant component set changes. This is not an instruction to reproduce DWSIM's ambient current-stream pointer in our domain. A scoped adapter can translate between our explicit request context and an external provider's required material association. Nor does compliance imply that the provider supports concurrent sessions or supplies a revision-aware history. [CO02]

**Maturity evidence and safe role (Q12).** Normative separation of responsibilities and subsequent errata demonstrate that lifecycle and authority edge cases have been considered. They do not certify the implementation of any component. Recommended role: use the standard as a compatibility checklist and potential external adapter boundary; keep the internal model richer where our P2/P3 scenarios require it. Whether to implement full runtime CAPE-OPEN compliance remains open.

**Carry into the blueprint (P):** explicit operation postconditions; context lifetime; compound/phase discovery; property availability separate from equilibrium; representation/slate maps; reaction-energy compatibility. **Still unverified (U):** support of the newer reaction interfaces by particular providers, runtime platform constraints and adapter conformance.

### L02. IDAES: shared definitions and local equation-oriented states

**Definition, representation and composition (Q01–Q05).** The inspected `PhysicalParameterBlock` holds shared property-package parameters and creates linked state blocks. It constructs a phase-component eligibility set from each phase's admitted components, rather than assuming every component belongs in every phase. This is directly useful for internal column, reactor or exchanger locations. The state is not inherently an entire stream, and the parameter definition is not the state. [ID01]

The modular framework lets component properties, phase behavior, state definitions and other methods be assembled into a configured package. Its phase set is declared when building the model. Phase amount may subsequently vanish, but that is different from dynamically inventing a new phase object during a numerical search. The required model structure and candidate phase list must therefore be known to the equation formulation. [ID04,ID05]

The pinned `enrtl.py` contains an actual electrolyte-NRTL implementation with ion-pair indexing, interaction rules and a reference-state choice. Thus, describing IDAES as having only electrolyte placeholders would be wrong. Conversely, the existence of this method does not establish a complete ready-to-run parameterization, transport model and reactive energy balance for every aqueous process. [ID03]

**Operations, authority, initialization and derivatives (Q06–Q09).** IDAES property calculations become expressions, variables and constraints in a mathematical model. A unit can request local properties and supply its own balance constraints; initialization temporarily fixes or activates parts of the problem as needed. The modular framework documents staged initialization rather than assuming that a generic monolithic flash always initializes a process state. Shared model parameters can be represented in a way that permits parameter estimation. [ID04]

The translator is particularly useful evidence for authority. Its inlet and outlet use separate packages; the outlet can be declared already defined or asked to enforce equilibrium, but the inspected constructor rejects the incompatible combination of a fully defined outlet and requested phase-equilibrium enforcement. The translator creates state blocks and ports, not an automatically correct map between arbitrary chemical representations. Application-specific linking constraints are still required. [ID02]

Derivatives of model expressions support an equation-oriented solver, but they must not be confused with total sensitivities after a separate equilibrium re-solve. The latter require differentiating the complete constrained solution. That distinction is a blueprint requirement, not a criticism of symbolic differentiation.

**Context, failure and persistence (Q10–Q11).** Model objects, fixed-variable status and activation state are mutable. The model-serialization utility restores attributes into an existing model; it does not reconstruct the defining algebra. Consequently, saving numerical values is not enough to reproduce the thermodynamic problem. Preserve the construction recipe, method identities, parameter values, phase declarations and reference conventions alongside numerical state. [ID06]

**Maturity evidence and safe role (Q12).** Shared parameter/state separation, phase eligibility, initialization, scaling hooks and translation authority are concrete provisions for maintaining complex process models. The project license is a three-clause BSD-style license with the additional contribution language in its own text. No test suite or installed solver build was exercised here. [ID01,ID02,ID07]

**Carry into the blueprint (P):** a property/reaction equation-submodel boundary; local state locations; property demand; structural phase eligibility; initialization responsibilities; explicit translator constraints. **Still unverified (U):** exact ready-configured model/data coverage for the target mixtures, solver dependencies, equation scaling and robustness across phase transitions, and persistence of full model recipes.

### L03. Clapeyron: composing thermodynamic methods without assuming one universal formulation

**Definition, representation and composition (Q01–Q05).** The inspected `CompositeModel` supports a fluid EOS, separate fluid correlations, or an activity model coupled to a compatible fluid description. It can include a solid description and an explicit mapping between solid and fluid components. It rejects an activity-only configuration in a context requiring a fluid model. This is a useful example of composability with semantic checks, rather than a flat enumeration treating NRTL and PR as interchangeable objects. [CL01]

Its parameter conventions distinguish single-component, pair and association data. Submodels can have their own overrides. Missing pair parameters may invoke model-specific combining rules; activity parameters have different requirements, and missing association data can mean absent interactions unless an association combining rule is selected. A universal rule that every missing coefficient is zero would therefore change model semantics. [CL04]

Caloric completion is another distinct requirement. The documented `BasicIdeal` model has constant ideal-gas heat capacity of 5R/2. That is a deliberate ideal contribution, not a fitted heat-capacity description of every real compound. A flowsheet package intended for heating, compression or reaction-energy accounting should explicitly choose and verify its ideal contribution. A successful saturation calculation is insufficient evidence that its energy predictions are fit for a process duty. [CL05]

**Operations, authority, initialization and derivatives (Q06–Q09).** The pinned flash documentation provides an unusually useful compatibility matrix. Multiphase TP search, two-phase flashes, generalized X–Y flashes and activity/electrolyte models do not all share the same eligibility. `MultiPhaseTPFlash` documents automatic phase search with activity models but not electrolyte models. The generalized X–Y formulation can express a broader problem than the library can automatically initialize: initial points are limited to two phases for some specification pairs. This is a numerical preparation limitation distinct from conceptual expressibility. [CL02]

The same documentation distinguishes active phases from zero-fraction entries that retain K-value information. Its older `tp_flash` return form also differs from the structured `tp_flash2` interface. A future adapter must normalize result meaning, not merely rename fields or count every returned object as a present physical phase. [CL02]

Clapeyron documents implicit differentiation of solved properties, avoiding differentiation through every numerical iteration. The documentation also explains why an earlier shortcut did not correctly deliver arbitrary higher-order derivatives. That reinforces the need to specify derivative order and path for the selected implementation. No universal smoothness or correctness at phase appearance is inferred. [CL06]

**Context and lifecycle (Q10–Q11).** Models can be built from supplied parameter tables and submodels; parameter/reference changes should produce a new resolved configuration in our host even where an implementation permits in-place changes. This report does not establish a complete built-in flowsheet persistence or dependency-invalidation contract. Treat such responsibilities as host-owned until a specific provider serialization route is verified.

**Maturity evidence and safe role (Q12).** The model/algorithm eligibility matrix, composite-model validation and documented data conventions are more useful maturity evidence than the size of the EOS catalog. The snapshot identifies version 0.6.28, while the repository also describes the project as actively evolving. Recommended role: model-composition reference and candidate whole configured model-plus-solver service. Numerical suitability for a selected conventional package needs the same conformance cases as the alternatives. [CL01–CL04,CL07]

**Carry into the blueprint (P):** explicit compatible submodel composition; structured parameter types; operation-specific solver eligibility; initializable versus theoretically formulable problems; candidate/active phase distinction. **Still unverified (U):** complete transport and industrial characterization coverage, generic reactive flowsheet behavior, reproducible model reconstruction, and robustness of the exact target model/flash combinations.

### L04. ThermoPack: substantial EOS service with interface-specific boundaries

**Definition, representation and composition (Q01–Q05).** ThermoPack is an EOS-centered numerical library, with configurable component lists, EOS parameters, numerical bounds and methods. Its public materials describe a long-lived Fortran core and C/C++ and Python interfaces. The source model container combines the mixture, EOS and solver options; the wrapper is not itself a complete flowsheet material model. [TP01,TP03]

The important correction is pseudocomponent support. A runtime cubic initialization route accepts critical properties, acentric factors and optional molecular weights for pseudocomponents. Thus, it is too strong to say that every new engineering component requires rebuilding the database. However, the pinned example explicitly states that ideal-gas heat capacities are not implemented for those pseudocomponents and that only residual caloric properties are available in the described route. PVT participation is not a complete petroleum heating/flash package. The missing ideal contribution needs a justified and tested completion strategy. [TP04,TP05]

The source also contains apparent-composition infrastructure. This is evidence against claiming that the project cannot express any nontrivial component mapping, but it does not establish a general electrolyte speciation and material-translation contract comparable to our full scope. [TP03]

**Operations, authority, initialization and derivatives (Q06–Q09).** The documented ordinary Python state operations include explicit two-phase TP, PH, PS and UV flashes, alongside property/derivative, saturation and envelope calculations. These are useful process primitives. The project's broader multiphase description does not establish that every desired solid or three-phase energy-constrained operation is available through the same high-level interface. Record the particular entry point and supported model, rather than making a package-wide claim. [TP01,TP02]

Pressure/temperature property functions and temperature/volume formulations use different natural independent variables and amount conventions. Derivative labels must preserve those coordinates and whether the property is total or molar. A numerical solver temperature bound is not automatically a scientific validity bound. The source stores limits used by solvers; our information model must keep those separate from fitted-data and model validity. [TP02,TP03]

**Context, failure and persistence (Q10–Q11).** The inspected source has a model registry and an active-model pointer in module state. It also contains per-thread EOS storage for internal OpenMP use. These are not equivalent guarantees. The presence of multiple model instances and internal parallel support does not prove that unrelated host threads can independently activate different models and evaluate them concurrently. Until a specific build is tested, an adapter should conservatively isolate or serialize the complete activation-plus-calculation sequence. This is a risk-control recommendation, not a claim that ThermoPack never supports concurrency. [TP03]

No portable full-session save/reload or universal nonfatal-error behavior was established here. Preserve the recipe and resolved parameters in the host; exercise error and interleaving paths before accepting a session pool.

**Maturity evidence and safe role (Q12).** The property/derivative interfaces and configurable numerical machinery make ThermoPack a meaningful candidate. Apache 2.0 code availability is documented. The limitations above are reasons to qualify its role, not dismiss it. Recommended role: a cohesive EOS session supplying supported properties and flashes; extend through separately qualified interfaces only when their required data and return semantics are known. [TP01–TP05]

**Carry into the blueprint (P):** operation-scoped capabilities; ideal/residual caloric completeness; component-order maps; numerical versus scientific bounds; explicit session activation and isolation. **Still unverified (U):** general high-level VLLE/solid PH coverage, complete pseudocomponent caloric completion, cross-model reentrancy, and recoverability of all native errors.

### L05. ChEDL thermo and chemicals: data separation plus an assembled process-property framework

**Definition, representation and composition (Q01–Q05).** `chemicals` and `thermo` should not be treated as the same layer. The former supplies data, correlations and numerical utilities; for example its Rachford–Rice routines allocate phases for supplied K-values and form an inner step of a larger equilibrium algorithm. They do not themselves supply a complete consistent thermodynamic model. [CH02]

Modern `thermo` assembles chemical constants, property correlations, phase-model objects, flash algorithms and bulk-property settings. The data-package documentation explicitly contrasts this with an earlier design that looked up data during calculations and was difficult to override. It also cautions that collected data are not a critically evaluated recommendation set. For our design, this supports a preparation step that resolves and records actual data rather than allowing future database changes to redefine saved results. [CH03]

The phase/flash API is the relevant surface, not the legacy convenience `Stream` class. The project itself recommends newer flash and `EquilibriumStream` interfaces for configurable thermodynamics. Phase objects can be evaluated directly without running an equilibrium flash; the documented style creates a new phase state from an existing model rather than editing its physical state in place. This is a design convention, not a claim that every internal cache or Python object is immutable. [CH04,CH06]

**Operations, authority, initialization and derivatives (Q06–Q09).** The three flash families differ materially. `FlashPureVLS` is for a pure component with suitable liquid/solid models and data; `FlashVL` handles a gas and one liquid; `FlashVLN` handles a gas and multiple liquid candidates. The pinned `FlashVLN` constructor rejects supplied solids and requires a gas model, even when a particular solution has no present vapor. The maximum searched liquid count comes from the configured candidate list. Therefore the mere existence of a `solids` argument or an N-phase label is not evidence of multicomponent precipitation support. [CH01,CH05]

Its model composition can support conventional cubic or activity-based workflows, but compatible caloric and phase descriptions remain essential. The phase tutorial explicitly warns that combining inconsistent phase models can introduce discontinuities or missing solutions. Library flexibility is not certification of arbitrary combinations. [CH04]

Bulk-property choices are separate configuration. That is valuable for effective viscosity and other aggregate relationships, which should not be confused with unique equilibrium properties. Property derivatives are available through phase methods; this investigation does not establish that every flash, every bulk closure or every changing phase set supplies the same differentiated-solution contract.

**Context, failure and persistence (Q10–Q11).** Phase JSON serialization and separate model/state hashes provide useful precedents. They can support reconstruction and cache identity, but the host should not assume an undocumented hash is a permanent cross-version scientific identifier. Store semantic configuration and dependency versions too. Warm starts and stability-skipping options must retain their qualification; a result obtained without a requested stability check must not be reported as a newly certified stable solution. [CH04]

**Maturity evidence and safe role (Q12).** Explicit data assembly, focused flash families, controllable solver constants, branch qualifications, phase-only access and serialization address practical framework concerns. These features justify promotion from a data-reference role to a serious assembled-package candidate. The inspected source files carry MIT license headers. No general electrolyte-reaction engine, full petroleum assay workflow or unrestricted multicomponent solid flash is inferred. [CH01–CH06]

**Carry into the blueprint (P):** separate data and algorithms; declared candidate count; phase-only evaluation; effective-property closures; functional state/result concepts; independent data-quality status. **Still unverified (U):** robust multiphase PH/PS on the selected mixtures, parameter/reference coherence across mixed methods, current serialization compatibility, and complete electrolyte/empirical material workflows.

### L06. CoolProp: backend/state lifecycle and bounded specialized packages

**Definition, representation and composition (Q01–Q05).** The low-level `AbstractState` interface constructs a selected backend/fluid system, sets composition, updates a state and reads properties. Reusing this object avoids treating every property call as an independent reconstruction. The common interface sits over different backends; it must not be interpreted as a guarantee that all backends support every composition basis, input pair or derivative. [CP01,CP04]

Mixture data and interaction parameters matter independently of available compound names. A named mixture or accepted component list is not enough if the required interactions are missing or estimated inadequately. The host should preserve whether parameters were supplied, inherited or estimated. Specialized pure-fluid, water, refrigerant and incompressible services remain valuable even when they are not general reacting process packages. [CP02]

**Operations, authority, initialization and derivatives (Q06–Q09).** A material correction to older guidance is supported by inspected code: the HEOS mixture path for specified pressure plus enthalpy, entropy or internal energy solves for temperature through inner PT state calculations. It attempts saturation-based narrowing, recovers usable bounds and checks the final property residual. Consequently, the audit does not label current CoolProp mixtures categorically incapable of PH or PS. This is an implemented source path, not a claim about old releases, every backend or all target mixtures. [CP03]

That final residual check is a useful architectural precedent: a root-finding procedure reaching a boundary or a phase discontinuity is not necessarily a solved thermodynamic specification. The provider checks the requested observable; the host should independently test its accepted state and balances. [CP03]

Phase imposition and derivative semantics require care. An imposed phase is an assertion/restriction on evaluation and can give an inappropriate result if supplied wrongly. Homogeneous-state derivatives, saturation-path derivatives and supported two-phase derivatives are distinct requests. They are not one boolean capability. [CP01]

**Context, failure and persistence (Q10–Q11).** The low-level documentation explains that reference-state changes do not retroactively alter an already created state instance. This is a concrete reason to bind a provider session to a resolved configuration revision, not reuse it after arbitrary global settings changes. Operations can update internal state while calculating; this investigation does not establish that a failed update leaves an object unchanged. Read outputs only after successful completion and use separate working instances where necessary. [CP01,CP03]

A saved handle is not a saved model. Preserve backend identity, fluid/composition and data/reference choices and reconstruct a fresh session. Cross-thread use of the same handle, different parameter overrides and global library settings needs explicit testing.

**Maturity evidence and safe role (Q12).** A reusable state protocol, documented backend variation, reference-state lifecycle and explicit inverse-specification residual checking are substantive integration provisions. Developer documentation includes release and testing guidance, but no numerical coverage audit was performed here. Publisher metadata reports MIT licensing; a REFPROP interface does not supply the separately obtained REFPROP engine or its data. [CP01,CP03–CP05]

**Carry into the blueprint (P):** reusable backend sessions; operation-scoped eligibility; phase restriction semantics; reference/version-bound session lifetime; verified inverse-state postconditions. **Still unverified (U):** installed-release parity with the pin, target mixture robustness, binding-specific coverage and recoverable concurrent-session behavior.

### L07. FeOS: coherent potential-based states with explicit calculation prerequisites

**Definition, representation and composition (Q01–Q05).** FeOS organizes EOS parameters, local thermodynamic states and phase-equilibrium results around thermodynamic potentials. Its documentation includes parameter records and JSON, ideal/residual model contributions, state properties, transport examples and density-functional/interface functionality. These are scientifically substantial abstractions, although they do not constitute the full material characterization and lifecycle of a general flowsheet simulator. [FE03,FE04]

The pinned phase-equilibrium container separates individual phase states and fractions, with optional total moles. A request for a total extensive quantity can fail when only an intensive state is known. This is precisely the distinction our scope requires between state, material amount and flow. Total enthalpy functionality is constrained to a total-caloric EOS description rather than being inferred from residual-only phase behavior. [FE01]

**Operations, authority, initialization and derivatives (Q06–Q09).** Most methods in the inspected equilibrium abstraction target two-phase vapor/liquid or liquid/liquid cases, with additional three-phase representations and specialized calculations. A representational parameter for phase count is not evidence that every operation solves arbitrary multiphase systems. A single-phase answer can also inhabit a two-entry result representation; the adapter must consider amounts and status rather than interpret the container size as a physical phase count. [FE01]

PH and PS algorithms are implemented. Their documented precondition is unusually important for flowsheet use: an initial temperature is required and the system must be two-phase at that initial temperature. The implementation first runs a TP flash and then solves the energy/entropy-constrained system, including implicit differentiation of the converged solution. A general-purpose valve or compressor service therefore needs a separately verified policy for locating initial states and handling single-phase regions, not just a call to a method named `ph_flash`. [FE02]

The interface and source show why we should separate residual thermodynamics, full caloric properties, initialization eligibility and solved-state sensitivities. These capabilities are related but not interchangeable. Interfacial or entropy-scaling methods likewise require their own parameters and models; their existence is not universal transport coverage. [FE01–FE03]

**Context, failure and persistence (Q10–Q11).** Parameter serialization is useful, while result/error types make incomplete amounts and unsuccessful calculations visible. The inspected interfaces alone do not establish whole-case persistence, cross-model cache invalidation or a universal parallel-sharing guarantee. Our host should snapshot semantic parameters and preserve accepted results independently of provider working state.

**Maturity evidence and safe role (Q12).** Optional extensive state, total-caloric eligibility, explicit PH/PS preconditions and implicit derivative implementation are concrete engineering/scientific provisions. Treating this solely as an amateur EOS collection would obscure them. For the user's breadth objective, however, FeOS remains a selected-model provider and abstraction reference, not the top-level definition of conventional, petroleum and reactive material scope. The project reports dual MIT/Apache-2.0 licensing. [FE01–FE04]

**Carry into the blueprint (P):** potential-based providers as one allowed formulation; optional total amounts; conditional total-caloric capability; preconditioned flashes; independent equilibrium sensitivity. **Still unverified (U):** broad single-/two-phase inverse-state dispatch, generic multi-liquid/solid reactive flowsheets, petroleum characterization, and complete provider result persistence.

### L08. Reaktoro: the chemical system as the coherent computational boundary

**Definition, representation and composition (Q01–Q05).** Reaktoro starts from a chemical system containing database-backed species, permitted phases and their thermodynamic/activity models. A `ChemicalState` stores material amounts and intensive conditions, not an intrinsically flowing stream. Property evaluation and chemical equilibration are distinct operations. For flowsheet use, amount-to-flow conversion must be explicit; a basis-sized chemical calculation is not automatically a reactor inventory. [RK01,RK02,RK04]

This is a richer composition problem than a nonreactive flash. Species quantities can change while the relevant conserved totals, including explicitly modeled exchanges, are maintained. Apparent feed descriptions, chemical formulas and the actual solver species list must be mapped rather than treated as the same coordinates. Standard thermodynamic data and activity corrections should remain a coherent combination. An equilibrium composition cannot simply be paired with an unrelated enthalpy model without a defined reconciliation.

**Operations, authority, initialization and derivatives (Q06–Q09).** The public solver interface separates declared specification kinds, actual condition values, reactivity restrictions, a mutable initial/final state and a returned outcome. It also exposes equilibrium sensitivity calculations. Restrictions can limit species reactivity, rather than forcing all species into full equilibrium. [RK01]

Generalized constraints include more than fixed T/P. The official fixed-volume/internal-energy tutorial computes temperature, pressure and equilibrium species amounts while maintaining volume, energy and the material accounting of a closed system. This is a direct architectural reference for SC-33 and reactive energy constraints, although the tutorial's example is not evidence that every electrolyte database provides all needed caloric data. [RK03]

Open systems require explicit exchanges. A constraint involving a reservoir or an added substance must report what entered or left the process material. The host must not silently create a titrant or gas supply merely to satisfy a chemical condition. [RK02]

Upstream kinetics is also broader than the DWSIM bridge traced in Step 2. The kinetics tutorial couples a kinetic mineral transformation with an aqueous subset treated as equilibrated, and checks the solver result on each step. Surface area and rate-law assumptions are part of that kinetic problem; equipment geometry, hydrodynamics and bulk flow integration do not disappear into thermodynamics. [RK05]

**Context, failure and persistence (Q10–Q11).** The source explicitly describes the state argument as both an initial guess and an output. Thus, the host should solve on a working copy and adopt it only after checking the result and conservation. The existence of copy construction does not establish exception atomicity or arbitrary simultaneous access to a solver. Persist the system/database identity, model selection, species/phase set, restrictions, conditions and initial material, not only the final species array. [RK01]

**Maturity evidence and safe role (Q12).** Separate specification/condition/restriction abstractions, direct equilibrium sensitivities, property evaluation and coupled kinetic/equilibrium workflows are relevant framework provisions. The upstream license is LGPL 2.1-or-later. The exact database and native dependency distribution still require assessment. The DWSIM C bridge is a separate adapter/fork surface; its capability must not be substituted for the whole upstream C++/Python interface. [RK01–RK06; DWSIM baseline]

**Carry into the blueprint (P):** coherent chemical-system providers; conserved-basis maps; open-exchange accounting; independent reaction participation; outcome/state separation; chemistry sensitivities. **Still unverified (U):** target aqueous caloric completeness, transport closure, species mapping in the chosen adapter, arbitrary cross-package reference translation and recoverability/concurrency of actual distributed builds.

## 4. Common-question comparison at the architecture boundary

This table condenses the dossiers without converting documented behavior into executed capability. “Host” means our future simulator must own or verify that responsibility; it does not assert that the library has no supporting utility.

| Reference | Definition and data authority | State/equilibrium authority | Phase/specification qualification | Lifecycle lesson |
|---|---|---|---|---|
| CAPE-OPEN | Discovery/material/slate interfaces; actual database supplied by component | Property evaluation and equilibrium have distinct postconditions | Supported operations and phases are implementation-specific | Explicit association; interoperability does not supply host history or atomicity. [CO01–CO03] |
| IDAES | Shared parameter block and construction configuration | Unit/property equations jointly define unknown states | Phase/component sets declared in model structure; translator authority checked | Save model recipe separately from state; initialization status matters. [ID01–ID06] |
| Clapeyron | Composite models and submodel-specific parameter tables | Direct property evaluation or chosen flash solver | Compatibility matrix depends on algorithm/model/specification; initialization can be narrower than formulation | Preserve resolved parameters, selected methods and active-phase interpretation. [CL01–CL06] |
| ThermoPack | EOS/mixture model, component data and solver settings | Property calls or exposed flash operation | Wrapper TP/PH/PS/UV route is two-phase; pseudo caloric completion is a separate problem | Active-model registry requires a verified session-isolation policy. [TP02–TP05] |
| ChEDL | Constants, correlations, phase models, bulk and solver settings | Phase-only calls, flashes and structured results | Pure/VL/VLN differ; VLN constructor rejects solids | Separate data preparation; JSON/hash utilities do not replace host versioning. [CH01–CH06] |
| CoolProp | Backend plus fluid/interaction/reference configuration | Update-selected state and property readback | Current source supports mixture PY, but not an all-backend guarantee | Reference changes and session construction are linked; failure-state recovery must be tested. [CP01–CP03] |
| FeOS | EOS parameter records plus ideal/residual contributions | Local states and selected differentiated equilibria | Most algorithms two-phase; PH/PS have explicit initialization precondition | Amount presence and total-caloric eligibility should be explicit. [FE01–FE03] |
| Reaktoro | Chemical system, database, activity models and permitted species | Property evaluation or constrained reactive/kinetic solution | Specs, actual conditions, restrictions and open exchanges are separate | Mutable working state and returned outcome must be adopted deliberately. [RK01–RK05] |

### 4.1 Maturity is multidimensional

This review identifies **maturity provisions**, not a validated ranking of numerical reliability. Four independent questions must remain visible:

**Process breadth:** can the surrounding framework characterize materials, assign methods, calculate internal states, route products, close unit/flowsheet balances and restore the case? DWSIM remains the primary reference from Step 2. A smaller library should not be penalized for intentionally leaving process orchestration to its host.

**Thermodynamic composition:** are methods, ideal/caloric contributions, phase eligibility, interactions and references organized coherently? Clapeyron, ChEDL and IDAES are especially useful design references for different parts of this question. This is an architectural inference from their inspected boundaries, not a performance result. [CL01,CH03,ID01]

**Numerical behavior:** does a selected operation have appropriate initialization, stability/branch logic, physical residual checks, failure reporting and derivatives? The concrete provisions differ: CoolProp's inspected inverse residual check, FeOS's explicit inverse preconditions and derivative implementation, and Clapeyron's compatibility matrix all answer parts of the question. None proves all target cases robust. [CP03,FE02,CL02]

**Operational reproducibility:** can resolved data, model construction, reference conventions and session configuration be reconstructed and isolated? ChEDL serialization, IDAES's state-only serializer and ThermoPack's activation state illustrate different responsibilities, not one equivalent “save” capability. [CH04,ID06,TP03]

The final provider selection should be profile-specific. We have enough evidence to choose architectural references now, but not enough to declare one numerical engine the most robust for the entire P1/P2 scenario set.

## 5. Eight cross-package workflow comparisons

Each workflow below states the common engineering action, how the packages map to it, and the proposed blueprint consequence. These are source/documentation walkthroughs, not executed simulations.

### W01. Construct and qualify a configured package

**Journey:** select a material representation → supply chemical/characterization data → select compatible phase methods → complete caloric and required transport information → choose candidate phases and algorithms → check eligibility → freeze a resolved configuration.

DWSIM demonstrates the complete engineer-facing preparation context. ChEDL separates constants and correlations from phase/flash construction; Clapeyron composes methods and applies submodel-specific parameter rules; IDAES builds shared parameters and phase eligibility. ThermoPack resolves a configured EOS model, CoolProp creates a backend/fluid system, FeOS constructs a potential-based EOS and Reaktoro creates a chemical system. These are different implementations of configuration, not interchangeable object types. [ID01,CL01,CL04,TP03,CH03,CP01,FE03,RK02]

**Important distinctions:** specified zero, missing value, predicted combining-rule value, regression result and fallback substitution are not equivalent. The host must know whether the package can perform the actual downstream request: component critical data alone do not establish usable enthalpy, entropy, diffusivity or reaction reference data.

**Blueprint consequence (P):** retain an explicit preparation/qualification result with unresolved needs. Never infer “ready for all unit operations” from successful model construction. Trace: SC-02,04,08,15,21,31; X08–X11; C05,C06,C11.

### W02. Evaluate phase properties without reallocating material

**Journey:** a unit has specified local phase compositions and states → request density, enthalpy, viscosity or equilibrium driving-force quantities → return values without changing phase amounts or forcing a bulk flash.

CAPE-OPEN separates property from equilibrium routines. ChEDL exposes phase-only evaluation, IDAES supplies local property submodels, and Reaktoro distinguishes chemical-property evaluation from a new equilibrium solve. ThermoPack, CoolProp and FeOS expose lower-level state/property operations, but any imposed phase, root or state constructor must be qualified in its own terms. [CO01,CH04,ID01,RK04,TP02,CP01,FE01]

**Blueprint consequence (P):** a request must say whether the provider may resolve density/root, redistribute phases, alter species or only evaluate the supplied state. Creating a state object is not necessarily a flash, and completing a flash is not permission to overwrite every independently supplied phase state. Trace: SC-07,09,26; X07,X14,X19; C03,C04,C09.

### W03. Resolve PH/PS states for heaters, valves and compressors

**Journey:** unit balances establish pressure and energy/entropy conditions → initialize the thermodynamic problem → solve → verify target property and conservation → publish a qualified outlet result.

IDAES can express the target as a constraint within a larger equation model. ChEDL offers inverse-flash families with their required phase/caloric data. Clapeyron selects an eligible inverse algorithm according to model family and initialization capability. ThermoPack's documented wrapper supplies explicit two-phase PH/PS. Current CoolProp HEOS source brackets temperature using repeated PT calculations and checks the final target residual. FeOS's PH/PS implementation requires a two-phase initial state. Reaktoro can constrain energy or entropy while allowing chemical redistribution when the chemical system asks for it; that is not automatically the same nonreactive flash. [ID04,CH05,CL02,TP02,CP03,FE02,RK02,RK03]

**Blueprint consequence (P):** capability must describe the full inverse problem, including fixed or reacting composition, complete caloric methods, initial-region requirements and allowed phases. A compressor also needs a distinction between an ideal PS reference state and its actual energy-constrained outlet. Trace: SC-02,04,06,12–14; X02,X06,X11,X19,X23; C03,C07,C11.

### W04. Support staged and multiple-liquid unit models

**Journey:** initialize stage states → evaluate phase quantities repeatedly → allow the selected unit formulation to couple stages → search or constrain local phases as appropriate → route products independently of provider ordering.

IDAES's declared phase-component sets suit equation-oriented internal states. ChEDL's VLN candidate list and Clapeyron's multiphase TP algorithm expose bounded/searchable phase possibilities. Their representations do not imply universal multi-liquid PH or reactive-separation support. FeOS's mostly two-phase methods and ThermoPack's ordinary wrapper flashes cannot be promoted to arbitrary VLLE merely from broader project descriptions. CAPE-OPEN lets an adapter report phases but does not provide their solver. [ID01,ID05,CH01,CL02,FE01,TP02,CO01]

A returned zero-amount phase can contain useful incipient or numerical information. It must not become an actual product stream merely because its array entry exists. Sorting liquids by density can support port routing, but it does not provide persistent physical phase identity near changing densities or branches.

**Blueprint consequence (P):** separate structural candidates, solver candidates, active physical phases, numerical placeholders and routing roles. The unit owns whether stage states are in equilibrium and whether transfer is efficiency-based or a full nonequilibrium formulation. Trace: SC-07–11,20,26; X05–X07,X14; C03,C04,C08.

### W05. Couple reactions, electrolytes and precipitation to energy accounting

**Journey:** represent feed and conserved quantities → establish species/phase eligibility and reaction participation → solve chemistry or rates under the declared constraints → account for exchanges → calculate compatible energy and requested transport → map into process outputs.

Reaktoro provides the most directly inspected generalized chemical-system boundary. IDAES supplies actual electrolyte methods and equation/reaction integration. CAPE-OPEN's reaction/slate concepts provide interface semantics, including energy corrections. Clapeyron includes electrolyte model/algorithm combinations, but the pinned flash matrix prevents treating all of them as eligible for generic multiphase or inverse algorithms. Aqueous property correlations in a data library are not, by themselves, a reacting-speciation system. [RK01–RK05,ID03,CO03,CL02,CH03]

The property package may include formation contributions already. Reaction energy must therefore be reconciled to the chosen caloric convention, not added mechanically. Mineral precipitation also needs explicit admitted solid phases and compatible standard-state data. A solid component merely being transported is a different process model.

**Blueprint consequence (P):** distinguish phase transfer, chemical equilibrium, kinetic evolution, empirical conversion, frozen species and external reservoirs. Define conservation in the representation that justifies it. Trace: SC-17–25,30; X11–X13,X20; C01,C07,C08,C10.

### W06. Cross thermodynamic regions

**Journey A, heat-only coupling:** evaluate each circuit with its own coherent package and couple through heat duty. No compound translation is implied.

**Journey B, material transfer:** establish source and target representations, energy references and which quantities are preserved; map the material; recalculate under explicit target constraints; report information loss and model disagreement.

IDAES's translator makes linking constraints explicit. CAPE-OPEN's material/slate interfaces help identify representation and authority. Neither provides a universal physical reconciliation between arbitrary packages. ChEDL's warning about inconsistent model combinations reinforces that interoperability is not thermodynamic consistency. [ID02,CO03,CH04]

**Blueprint consequence (P):** never try to preserve temperature, pressure, composition and incompatible enthalpy predictions simultaneously by hiding a heat correction. A legitimate reference offset and a physical-model discrepancy are separate facts. Trace: SC-28–30; X11,X13; C07,C10.

### W07. Change data, reuse sessions, save/reload and recover failure

**Journey:** modify an interaction parameter or reference convention → identify affected results and provider sessions → reconstruct or rebind as needed → recalculate on working state → adopt only validated local outputs → preserve historical results with their original configuration.

IDAES serializes state into existing algebra rather than saving its construction. ChEDL provides phase serialization and hashes. CoolProp documents reference changes that do not propagate into already created state objects. ThermoPack has model activation state. Reaktoro takes a mutable initial/final state. These differences require explicit adapter-specific lifecycle logic, not a generic assumption of immutable or stateless providers. [ID06,CH04,CP01,TP03,RK01]

**Blueprint consequence (P):** preserve semantic construction and resolved data separately from caches, guesses and handles. Test complete call sequences, not only individual functions, for isolation. Failure outcomes must distinguish unsupported operations, unresolved data, infeasible specifications, nonconvergence and failed optional properties. Trace: all scenarios; X08,X16–X18,X23; C06,C11,C12.

### W08. Add an unfamiliar material or a limited-property engineering representation

**Journey:** import an assay, pseudocomponent or empirical material → declare what information is known → determine which operations can be justified → permit those operations without inventing molecular detail.

ThermoPack demonstrates partial cubic pseudocomponent support but explicitly incomplete ideal-caloric data in the inspected example. ChEDL and Clapeyron accept custom constants/parameters, but that is not a complete assay-characterization workflow. FeOS can express intensive states without total amounts; Reaktoro requires meaningful chemical identity/conservation data for its chemical operations. None of those facts alone demonstrates SC-31 mass-only empirical-material support across the whole requested flowsheet. [TP04,CH03,CL04,FE01,RK02]

**Blueprint consequence (P):** capability attaches to material representation plus operation, not only to provider. A mass-based heat-capacity model can support heating without supporting molar fractions, fugacity or chemical equilibrium. Do not fabricate molecular weight or critical constants to satisfy a uniform fluid interface. Trace: SC-15,16,23,27,31–33; X01,X09,X13; C01,C02,C06,C13.

## 6. Coverage against all 34 scope scenarios

“Candidate” means an architectural or numerical route merits testing. It does not mean the scenario was executed. Each row includes unit-level requirements in addition to thermodynamics, so no library gets an automatic pass from a property API. DWSIM remains the previously researched flowsheet reference; this table focuses on the newly compared families.

| Scenario | Candidate contribution from this research | Evidence and unresolved qualification |
|---|---|---|
| SC-01 Mixing/splitting | ChEDL/Clapeyron configured phases; IDAES balance-linked states; suitable EOS services | Property/state routes S/D; mixing and mechanical splitting remain unit operations. W01,W02,W06. |
| SC-02 Heating/exchange | ChEDL, Clapeyron, CoolProp, ThermoPack; IDAES formulation | Caloric completion, inverse branch and optional rating properties still to test. W03. |
| SC-03 Pumps/pipes | Phase-property service appropriate to material; IDAES local states | Hydraulic model is external; viscosity and phase admissibility need request-specific qualification. W02. |
| SC-04 Compression/expansion | ChEDL/Clapeyron/CoolProp/ThermoPack state routes; IDAES constraints | PS reference plus actual PH; FeOS inverse precondition prevents unqualified assignment. W03. |
| SC-05 VLE separation | ChEDL/Clapeyron/ThermoPack/FeOS and selected CoolProp systems | Candidate set and stability evidence required; equipment routing separate. W03,W04. |
| SC-06 PH valve | Same energy-complete fluid systems; IDAES inverse state | Explicit initialization and final enthalpy residual, not merely returned temperature. W03. |
| SC-07 Distillation | IDAES equation states; ChEDL/Clapeyron or suitable phase kernels | Repeated local evaluation and caloric consistency; unit solver not supplied by a standalone flash. W02,W04. |
| SC-08 Nonideal/azeotropic separation | ChEDL activity phases, Clapeyron composite model, IDAES activity methods | Parameter coverage and ideal/excess/caloric choices must be pinned. W01,W04. |
| SC-09 Physical absorption/humidification | Suitable gamma–phi/Henry or specialized property descriptions | Exact solubility, caloric and transport requirements U for target cases. W02,W04. |
| SC-10 LLE extraction | ChEDL VLN, Clapeyron multiphase/two-liquid methods, IDAES declared phases | Liquid candidates and identity/routing must be qualified; none executed. W04. |
| SC-11 VLLE | ChEDL VLN and Clapeyron multiphase TP strongest directly documented routes here | PH/PS, global stability and target mixtures U; no general N-phase implication. W04. |
| SC-12 Water/steam | CoolProp; ChEDL IAPWS phase/flash interfaces; IDAES pure-fluid routes from baseline | Saturation specification independence and quality basis required. W03. |
| SC-13 Pure refrigeration | Complete pure-fluid phase/state package | Loop convergence and component states need actual process integration. W03,W07. |
| SC-14 Mixed refrigeration | Suitable mixture package among ChEDL/Clapeyron/ThermoPack/CoolProp | Exact interaction data, inverse flashes and phase glide must be tested. W03. |
| SC-15 Petroleum pseudocomponents | ThermoPack cubic pseudo route; ChEDL/Clapeyron custom data | Partial support S; complete assay and caloric construction U. W08. |
| SC-16 Black oil | No complete alternative workflow established | DWSIM baseline investigation and a dedicated representation/provider remain needed. W08. |
| SC-17 Conversion reaction | IDAES reaction/unit approach or host conversion plus compatible property provider | Conversion semantics and reaction-energy convention required. W05. |
| SC-18 Equilibrium reaction | Reaktoro chemical system; IDAES equation/reaction package | Data completeness and energy/basis compatibility still to qualify. W05. |
| SC-19 Kinetic reaction | Reaktoro kinetics; IDAES/host kinetic model plus properties | Distinguish rate law, chemical substep and actual reactor transport/residence-time model. W05. |
| SC-20 Reactive separation | Coupled host/IDAES formulation or qualified chemical provider in stage model | Whole reactive column workflow not established by this research. W04,W05. |
| SC-21 Electrolytes/neutralization | Reaktoro; actual IDAES eNRTL framework | Apparent/true species, databases, reaction participation and caloric completeness. W05. |
| SC-22 Reactive gas absorption | Reaktoro/IDAES chemistry combined with contactor model | Gas/liquid equilibrium, reaction, transport and heat cannot be assumed from speciation alone. W05. |
| SC-23 Inert carried solids | Host material/solids representation plus conditional properties | No standalone equilibrium solver is a general slurry carrying/routing model. W08. |
| SC-24 Precipitation/crystallization | Reaktoro mineral phases; selected Clapeyron solid descriptions; IDAES configured model | Thermo FlashVLN explicitly rejects solids; kinetics/size model and full caloric fixture separate. W05. |
| SC-25 Gas–solid transformation | Reaktoro or appropriate IDAES/host chemistry | Specific species/phase thermodynamics and reactor closure U for fixtures. W05. |
| SC-26 Nonequilibrium contacting | Phase-only properties from suitable providers; IDAES local-state formulation | Full bulk/interface transfer formulation remains host responsibility, not automatic flash capability. W02,W04. |
| SC-27 Adsorption/membrane extension | FeOS surface/pore concepts; chemical potentials from applicable providers | P3 representability only; no complete membrane/adsorber workflow certified. W08. |
| SC-28 Heat-only package coupling | Any two internally coherent region packages | Independently referenced enthalpy differences; no material map across wall. W06. |
| SC-29 Material package boundary | IDAES translator and CAPE-OPEN contract concepts | No universal automatic energy/model reconciliation. W06. |
| SC-30 Representation translation | CAPE-OPEN slates, IDAES translator, Reaktoro species system | Conservation and information loss must be defined by the mapping. W05,W06. |
| SC-31 Mass-only empirical material | No complete direct provider workflow established | Conditional-property host model required; molecular data cannot be fabricated. W08. |
| SC-32 Polymer distributions | Model-specific parameters provide only partial analogies | Full distribution transfer/reduction semantics U; P3 retained. W08. |
| SC-33 Inventory/UV state | Reaktoro closed UV example; ThermoPack UV route; FeOS optional amounts | Total versus molar/specific variables and material amount explicit; time integration separate. W03,W08. |
| SC-34 Restricted/metastable requests | Reaktoro restrictions; CoolProp phase imposition; provider-specific phase constraints | Restrictions are not all physically equivalent; P3 scope labels remain mandatory. W02,W05. |

The table deliberately does not award DWSIM, IDAES or any library all downstream unit scenarios merely because it can be extended with custom code. Extension feasibility and implemented operation coverage remain separate.

## 7. The conceptual architecture implied by the comparison

The following is a synthesis to carry into the later functional specification, not a final object model or implementation schema.

### 7.1 Package the problem in seven cooperating responsibilities

| Conceptual responsibility | What it owns | Primary design references |
|---|---|---|
| **Material description** | Meaning of compounds/species/lumps, phases/material domains, composition coordinates, amount basis and conserved quantities | DWSIM breadth; CAPE-OPEN slates; IDAES eligibility; Reaktoro chemical system; FeOS intensive/extensive distinction. |
| **Resolved thermodynamic definition** | Approved models, ideal/caloric completion, parameter values, references, transport closures, phase/reaction policy and provenance | ChEDL data separation; Clapeyron composition; IDAES shared parameters. |
| **Calculation problem** | Specified quantities, unknowns, allowed changes, required outputs and requested derivative semantics | CAPE-OPEN postconditions; IDAES defined/equilibrium state; Reaktoro specs/conditions/restrictions. |
| **Execution preparation** | Operation eligibility, initial states, branch policy, numerical bounds, provider sessions and workspaces | Clapeyron algorithm matrix; FeOS inverse preconditions; ThermoPack and CoolProp sessions; IDAES initialization. |
| **Thermodynamic execution** | Selected numerical operations or equation contribution, coherent within its advertised contract | Whole configured packages or qualified kernels from the numerical libraries; IDAES equation submodels. |
| **Outcome and publication** | Candidate values, required checks, diagnostics, requested-versus-used physics, acceptance and result lineage | DWSIM publication boundary; CAPE-OPEN operation separation; CoolProp residual check; Reaktoro explicit result. |
| **Flowsheet integration and lifecycle** | Unit constraints, internal locations, heat/material coupling, translation, revision propagation, save/rebuild and nested convergence | DWSIM complete workflows; IDAES unit/translator model; host-owned orchestration. |

This arrangement does not demand that every provider be split into seven separately callable interfaces. A complete external package can implement several responsibilities behind an adapter. Conversely, a data provider can implement only one. The host's semantic boundaries are more important than forcing external code into identical object structures.

### 7.2 Preserve coherent thermodynamics before maximizing interchangeable parts

A gamma–phi package is not assembled just by connecting an NRTL coefficient vector to an arbitrary vapor density function. It needs compatible liquid standard-state and saturation/Henry behavior, vapor fugacity, caloric contributions, references, required pure properties and the chosen equilibrium algorithm. The composite-model checks in Clapeyron and the phase-compatibility warning in ChEDL demonstrate why the construction must be qualified. [CL01,CH04]

A potential-based provider may derive many properties from a common formulation. A correlation-based engineering package may supply those properties through several functions. Both should fit the architecture. Do not require every material to expose a Helmholtz potential or every property function to be symbolic. Instead, declare the compatibility and consistency conditions appropriate to the configured package.

The same rule applies to reactive systems. An equilibrium composition, a reaction enthalpy correction and a stream enthalpy are not three independently interchangeable values. Their conventions and material representation must be reconciled. [CO03,RK01–RK03]

### 7.3 Treat initialization as a capability, not just a performance hint

A mathematically expressible flash may have no supported initializer for the requested phase count. A library may need a two-phase initial point or a prior envelope. Some operations may return an incipient phase for continuation. Those are information and action requirements in the blueprint.

Therefore distinguish: the problem can be formulated; the provider can evaluate its equations; an admissible starting point can be prepared; a numerical solution was obtained; and the accepted result satisfies the requested physical constraints. This is a refinement of the existing R/E/V distinction, not a replacement for it. [CL02,FE02,CP03]

### 7.4 Separate a property from its derivative and from a total solved-state sensitivity

The same symbol can have different meanings depending on whether composition, density, phase amounts or chemical reaction is held fixed. Phase-property derivatives in ThermoPack or CoolProp, equation derivatives in IDAES, implicit flash derivatives in FeOS/Clapeyron and constrained chemical sensitivities in Reaktoro should not be collapsed into one “automatic differentiation available” flag. [TP02,CP01,FE02,CL06,RK01]

A derivative request should carry its output observable, input coordinate, amount basis, fixed quantities, reaction/phase response, derivative order, and validity at the current state. This describes meaning, not a code signature.

### 7.5 Configuration completeness has several levels

A useful qualification order is:

**Identity/representation completeness → parameter completeness → phase-model completeness → caloric completeness → requested transport/derivative completeness → algorithm eligibility → initialization readiness → process-boundary compatibility.**

A failed later qualification need not make earlier limited operations unusable. The ThermoPack pseudocomponent example is a concrete reminder: a PVT-capable cubic material is not automatically calorically complete. The same principle permits useful mass-only empirical heating models without granting them molecular or reactive capabilities. [TP04; SC-31 baseline]

## 8. Proposed requirements and decisions to carry forward

These are proposed requirements derived from the research. Their final wording, ownership and detailed acceptance criteria belong to Step 4 and the information/action blueprint.

| ID | Proposed requirement | Evidence or motivating distinction | Scope trace |
|---|---|---|---|
| CR-01 | A package configuration identifies coherent method/data/reference choices independently of its backend session. | IDAES shared parameters, ChEDL composition, CoolProp lifetime. | C05,C06,C12; X16–X18 |
| CR-02 | A request declares authority over intensive state, phase allocation and species transformation separately. | CAPE-OPEN, IDAES translator, Reaktoro restrictions. | C03,C04,C08; X07,X14 |
| CR-03 | Phase and property outputs have operation-specific postconditions, not an all-properties-populated assumption. | CAPE-OPEN errata; phase-only providers. | C09,C11; X09,X23 |
| CR-04 | Capability is indexed by material representation, resolved data, model, phase policy, operation and derivative demand. | All dossiers; explicit restricted constructors/algorithms. | All scenarios; C11 |
| CR-05 | Model declaration, initializer coverage and solver coverage are distinct. | Clapeyron generalized XY; FeOS PH/PS precondition. | C03,C11; X02,X06 |
| CR-06 | Ideal/residual/caloric/formation contributions are explicitly qualified before energy-constrained use. | ThermoPack pseudo, Clapeyron ideal model, FeOS total bound, CAPE energy correction. | C07; X11,X12 |
| CR-07 | A missing parameter's meaning is method-specific and records approved estimation/substitution. | Clapeyron combining/association conventions; DWSIM baseline. | C06; X08 |
| CR-08 | Candidate, active, incipient, aggregate and numerical phase entries remain distinguishable. | Clapeyron zero-fraction entries; FeOS container; ChEDL max candidates. | C04; X05–X07 |
| CR-09 | Heat coupling, material transfer and representation conversion are different actions. | IDAES translator and CAPE slates; DWSIM exchanger. | C10; SC-28–30 |
| CR-10 | Reactive calculations preserve the justified conserved totals and account for every modeled external exchange. | Reaktoro constraints and CAPE reaction representation. | C01,C08; X12,X20 |
| CR-11 | Effective bulk/transport properties identify the added closure and required phase pair or material domain. | ChEDL BulkSettings; phase-specific services. | C09; X22 |
| CR-12 | Provider lifetime and sharing are explicit; no concurrency is inferred from multiple instances or internal parallelism. | ThermoPack activation; CoolProp reference lifecycle; mutable Reaktoro state. | C12; X18 |
| CR-13 | Failure and numerical retry do not silently change physical assumptions or replace an accepted result with an unqualified trial. | DWSIM fallbacks; mutable provider state; CoolProp target checks. | C11,C12; X07,X23 |
| CR-14 | Save/reload preserves the semantic recipe and resolved data, not only provider handles or solved values. | IDAES serializer boundary; ChEDL hashes; session models. | C06,C12; X17 |
| CR-15 | Derivative requests identify coordinates, held-fixed quantities and phase/chemical response. | FeOS, Clapeyron, Reaktoro, CoolProp derivative differences. | C11; X19 |
| CR-16 | Partial material/property definitions remain usable for justified operations without invented molecular data. | Pseudo caloric limits and the SC-31 baseline. | C01,C02,C11; X01,X09 |

No Rust traits, physical storage schemas, framework dependencies or default numerical backend are selected by these requirements.

## 9. Safe integration boundaries and candidate profiles

### 9.1 Candidate profiles, not a single universal winner

| Profile | Leading candidates to exercise | What must be demonstrated before adoption |
|---|---|---|
| **Conventional assembled fluid package** | DWSIM complete configured package; ChEDL phase/flash assembly; Clapeyron composite model | Required cubic/activity methods, data selection, caloric properties, PH/PS, phase handling and reproducible results. |
| **Focused EOS numerics** | ThermoPack; selected Clapeyron or FeOS implementations | Actual property/flash/derivative interfaces, data completeness, initialization and session isolation. |
| **Utilities and specialized fluids** | CoolProp; relevant ChEDL or IDAES pure-fluid routes | Saturation, quality and inverse-state behavior for the chosen fluid/backend and required transport properties. |
| **Reactive aqueous/mineral system** | Reaktoro; configured IDAES electrolyte/reaction model | Species/phase definitions, energy data, constraints/exchanges, reporting maps and process transport demands. |
| **Equation-oriented process integration** | IDAES as the main structural reference; qualified numerical providers behind external operations | Local states, required equations/properties, scaling, initialization, derivative semantics and host solver compatibility. |
| **Interoperability and representation** | CAPE-OPEN with IDAES translator and Reaktoro representation concepts | Explicit operation authority, compound/slate maps and caloric compatibility; not automatic full standard implementation. |

These are recommended evaluation groups based on the inspected architecture. They are not rankings of numerical fidelity or maturity and do not imply all candidates in a row support identical features.

### 9.2 Whole provider versus primitive integration

Prefer a complete configured package where it already bundles compatible phase behavior, ideal/caloric models, data and robust state algorithms. Reusing only its EOS while rebuilding its flashes can discard much of its engineering value. Prefer a primitive boundary where the provider has a narrow, clear contract and the host genuinely needs a shared solver or equation representation.

For Reaktoro, a whole chemical-system boundary is often the safer initial interpretation because standard-state data, activity corrections, species constraints and energy conventions are coupled. For ThermoPack and CoolProp, a configured numerical session is a natural implementation boundary. For IDAES, the reusable contribution can be a mathematical submodel rather than a numerical snapshot. These are design inferences from the dossiers, not mandatory implementation choices.

## 10. Public availability and operational prerequisites

This is a dependency-screening record, not legal advice or a complete distribution review. Code licenses do not automatically cover optional databases, external engines, fitted parameter collections, wrappers or other dependencies.

| Family | Publicly reported or inspected main-code terms | Important separate check |
|---|---|---|
| IDAES | Three-clause BSD-style project license, including its own additional contribution text. [ID07] | Pyomo/solvers, compiled property helpers and data used by selected packages. |
| Clapeyron | MIT reported by its repository. | Data-file provenance, optional model data and extension/runtime dependencies. |
| ThermoPack | Apache 2.0. [TP01] | Compiler/runtime artifacts, actual wrapper exports and any added fluids/data. |
| thermo / chemicals | MIT headers inspected. [CH01,CH02] | Source and redistribution conditions of imported property data and optional dependencies. |
| CoolProp | MIT publisher metadata. [CP05] | REFPROP is separately obtained; qualify actual backend data and build. |
| FeOS | MIT OR Apache-2.0 reported by repository. [FE04] | Model-specific parameter files and actual enabled functionality. |
| Reaktoro | LGPL 2.1-or-later. [RK01,RK06] | Exact database, native dependency closure, wrapper/fork terms and runtime packaging. |
| CAPE-OPEN | Public specifications, not a single software library license. [CO01–CO04] | Terms and platform support of the chosen implementation, type libraries and adapters. |

The source pins are intentionally distinct from a deployable bill of materials. That later record needs exact released artifacts, dependency versions, parameter/data versions and configuration. No performance numbers, binary compatibility or “works on all platforms” claim is made by this audit.

## 11. Twenty unexecuted verification probes

These complement the 18 DWSIM probes from Step 2. They are deliberately bounded witnesses rather than a benchmark contest. The exact material, parameter file, operating points and tolerances must be fixed from suitable evidence before execution. None was run here.

| ID | Target | Stimulus | Required witness | Gate |
|---|---|---|---|---|
| PV-01 | Every numerical candidate | Build the same specified fluid state twice from frozen data, once after a new session. | Stable values within declared tolerance; recorded model/data/reference identity. | Reproduction |
| PV-02 | ChEDL | Build an explicit phase/caloric package without database lookups; save/reconstruct it. | Identical resolved inputs and consistent properties; no silent default-data substitution. | Data/lifecycle |
| PV-03 | ChEDL VLN | A reference two-liquid/three-phase TP case, then reorder candidate phase objects. | Conserved amounts; phase identity resolved independently of position; stability scope recorded. | Phase semantics |
| PV-04 | ChEDL VLN | Supply solid candidates and compare with a suitable pure-solid route. | Expected unsupported multicomponent-solid result; no false feature promotion from parameter signature. | Capability |
| PV-05 | Clapeyron | Exercise selected gamma–phi TP, PH and PS combinations using explicitly chosen algorithms. | Each pair either succeeds with residual checks or rejects precisely; no catalog-wide inference. | Algorithm eligibility |
| PV-06 | Clapeyron | Compare a generalized XY problem with and without a valid two-phase initial state. | Initialization requirement distinguished from unsupported equations or physical infeasibility. | Initialization |
| PV-07 | Clapeyron | Replace BasicIdeal with a justified compound caloric model for an ordinary heater case. | Caloric provenance and change in energy prediction recorded; no claim that phase behavior alone validates duty. | Caloric completeness |
| PV-08 | ThermoPack | Built-in mixture followed by the documented cubic pseudo reconstruction. | PVT equivalence where intended; exact unavailable/available ideal and total caloric operations identified. | Pseudocomponent scope |
| PV-09 | ThermoPack | Interleave two differently configured EOS models; then attempt controlled concurrency. | No activation/configuration contamination under the chosen isolation policy. | Session safety |
| PV-10 | ThermoPack | Compare its exposed two-phase flash route with any proposed multiphase/solid route. | Exact component/phase/request outputs and derivative/error contract recorded; no inferred wrapper parity. | Provider boundary |
| PV-11 | CoolProp | Mixture TP state → H/S extraction → matching PH/PS calls across selected phase regions. | Final observable agrees with target; phase behavior and build identity recorded. | Inverse solve |
| PV-12 | CoolProp | Change reference convention before versus after state construction, using isolated sessions. | Existing/new-instance behavior documented; no reuse under the wrong resolved revision. | Reference lifecycle |
| PV-13 | FeOS | PH/PS starting inside versus outside the documented two-phase initialization region. | Exact supported region and failure behavior; separate initializer requirement. | Inverse solve |
| PV-14 | FeOS/Clapeyron | Compare implicit equilibrium sensitivities with carefully chosen finite perturbations away from transitions. | Matching coordinates/order; separately flag phase-boundary nonsmoothness and conditioning. | Derivative semantics |
| PV-15 | IDAES | Construct a small unit with local state blocks and prescribed candidate phases; initialize, release and solve. | No unintended overconstraint; property demand, scaling and state-fixing lifecycle recorded. | Equation integration |
| PV-16 | IDAES | Save state, alter/rebuild the model recipe, then restore only numerical attributes. | Detect incompatible construction; demonstrate why state serialization is not a full model archive. | Reconstruction |
| PV-17 | Reaktoro | Closed UV material with pinned compatible data; separately evaluate supplied state properties. | Conserved quantities/UV verified; property evaluation does not re-equilibrate by accident. | Chemistry authority |
| PV-18 | Reaktoro | A constrained open-system or kinetic/equilibrium example with explicit reservoir/rate assumptions. | Every material exchange accounted for; species restrictions and outcome checked. | Reactive scope |
| PV-19 | Cross-package | Transfer one material between two packages under two declared boundary policies. | Preserve specified quantities; distinguish energy reference offset from model discrepancy; no hidden heat. | Translation |
| PV-20 | Every adapter / CAPE-OPEN where applicable | Fail a primary calculation, then an optional property, then reuse the session. | Accepted state protected; outcomes distinguished; post-equilibrium property promises honored. | Failure/publication |

Not all probes must precede drafting functional requirements. They do precede upgrading a provider/scenario to Executable or Validated and any strong claim about runtime reliability, derivative correctness or session isolation.

## 12. Open questions and handoff

The provider investigation narrows the design uncertainty, but leaves explicit issues:

**Highest-impact unresolved coverage:** petroleum characterization with complete caloric information; mass-only empirical/nonconventional materials; bounded multicomponent solid and electrolyte energy calculations; full nonequilibrium contacting; reactive separation; and consistent transfer between different thermodynamic representations. No single library's model catalog closes those questions.

**Version/documentation drift:** CoolProp's older limited-mixture guidance is superseded by implemented source paths at the selected pin. Clapeyron documentation from different tracks describes different defaults; this report relies on the pinned compatibility matrix rather than one universal default. ThermoPack `vcurrent` explicitly denotes a beta documentation track. IDAES introductory EOS lists do not exhaust the actual source, as the eNRTL implementation demonstrates. These are reasons to pin provider profiles, not reasons to distrust all documentation.

**Unassessed operational guarantees:** full native error recoverability, detailed concurrent-session behavior, exact CI/test coverage, numeric performance, serialization across releases, parameter-data redistribution and binary binding parity. Source inspection supplies test hypotheses; it does not answer these empirically.

### Step-3 handoff

This report supplies eight comparative dossiers, a shared question framework, eight complete conceptual workflow comparisons, an all-scenario coverage map, sixteen proposed requirements and twenty explicit probes. The source register separates fixed source evidence from moving documentation.

The next functional-specification step should turn each proposed requirement into observable behavior with ownership, inputs, postconditions, failure outcomes and an acceptance witness. It should preserve whole-package integration as an option and should not choose a default backend merely because one language binding is convenient.

**Final design thesis:** use DWSIM to preserve the breadth of the engineer's flowsheet workflow; use CAPE-OPEN to clarify interface authority; use IDAES to structure local mathematical states; use ChEDL and Clapeyron to organize coherent methods and resolved data; and let ThermoPack, CoolProp, FeOS and Reaktoro contribute through their actual, qualified computational boundaries. The simulator owns representation meaning, compatibility, publication and reproducibility across those boundaries.

## Source register

Accessed 25 September 2026. Source-code entries use the fixed commits in Section 1.2; line ranges are source-file ranges, not tool-output line numbers. Documentation entries are the actual consulted public pages. Source descriptions identify the limited claim they support; they do not certify the report's proposed architecture. Open-source examples are evidence of intended behavior, not numerical runs performed in this research.

### [CO01] CAPE-OPEN Thermodynamics and Physical Properties 1.1, specification 3.11

**Evidence:** Official documentation; moving URL unless version stated.  
Conceptual objects and separate material/property/equilibrium interfaces; scope exclusions. Dated 10 May 2011; conceptual relationship figure on PDF page 14 inspected.  
Source: <https://www.colan.org/wp-content/uploads/2015/05/CO_Thermo_1.1_Specification_311.pdf>

### [CO02] Thermodynamics 1.1 Errata, 2.0.0009

**Evidence:** Official documentation; moving URL unless version stated.  
SetMaterial context refresh; post-equilibrium outputs; CheckEquilibriumSpec. November 2012. Textual clauses used; no inaccessible figures inferred.  
Source: <https://www.colan.org/wp-content/uploads/2016/06/Errata_1.1_2.0.0009.pdf>

### [CO03] Chemical Reactions interface specification

**Evidence:** Official documentation; moving URL unless version stated.  
Approved version 1.1, revision 1 December 2024: compound slates, reaction interfaces, and enthalpy-of-reaction balance correction. Introductory and energy-accounting textual clauses, including printed pages 15–18.  
Source: <https://www.colan.org/wp-content/uploads/2024/10/Chemical-Reactions-Interface-Specification.pdf>

### [CO04] CO-LaN Chemical Reactions specification landing page

**Evidence:** Official documentation; moving URL unless version stated.  
Specification status and relationship to Thermodynamics 1.1; standard availability is not implementation coverage.  
Source: <https://www.colan.org/specifications/chemical-reactions-2/>

### [ID01] IDAES: idaes/core/base/property_base.py

**Evidence:** Pinned source, inspected lines 75–240.  
Shared parameter block, state-block construction, phase-component eligibility, scaler references.  
Source: <https://github.com/IDAES/idaes-pse/blob/5e51567b315a25a4ed0057b4a23afe02f1109482/idaes/core/base/property_base.py#L75-L240>

### [ID02] IDAES: idaes/models/unit_models/translator.py

**Evidence:** Pinned source, inspected lines 1–245.  
Separate inlet/outlet packages and authority flags; caller-provided mapping constraints; initializer boundary.  
Source: <https://github.com/IDAES/idaes-pse/blob/5e51567b315a25a4ed0057b4a23afe02f1109482/idaes/models/unit_models/translator.py#L1-L245>

### [ID03] IDAES: idaes/models/properties/modular_properties/eos/enrtl.py

**Evidence:** Pinned source, inspected lines 1–150.  
Actual liquid-electrolyte NRTL implementation, ion-pair sets, alpha/tau rules, reference-state choices.  
Source: <https://github.com/IDAES/idaes-pse/blob/5e51567b315a25a4ed0057b4a23afe02f1109482/idaes/models/properties/modular_properties/eos/enrtl.py#L1-L150>

### [ID04] IDAES modular property framework

**Evidence:** Official documentation; moving URL unless version stated.  
Modular configuration, fixed parameters usable for estimation, hierarchical initialization.  
Source: <https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/index.html>

### [ID05] IDAES phase definitions

**Evidence:** Official documentation; moving URL unless version stated.  
A priori phase declarations and phase-specific component eligibility; introductory catalog is not exhaustive source inventory.  
Source: <https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/phase_def.html>

### [ID06] IDAES model serializer

**Evidence:** Official documentation; moving URL unless version stated.  
State/attribute restoration requires an already constructed model; not reconstruction of its algebra or objects.  
Source: <https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/model_serializer.html>

### [ID07] IDAES: LICENSE.md

**Evidence:** Pinned source, inspected lines 1–90.  
Project license text; additional dependency and data terms remain separate.  
Source: <https://github.com/IDAES/idaes-pse/blob/5e51567b315a25a4ed0057b4a23afe02f1109482/LICENSE.md#L1-L90>

### [TP01] ThermoPack project overview

**Evidence:** Official documentation; moving URL unless version stated.  
EOS-centered numerical library, Fortran and wrappers, Apache 2.0; overview says multiphase, which does not certify every wrapper operation.  
Source: <https://thermotools.github.io/thermopack/>

### [TP02] ThermoPack core Python interface

**Evidence:** Official documentation; moving URL unless version stated.  
Explicit two-phase TP/PH/PS/UV operations; thermodynamic property/derivative interfaces and reference setters; vcurrent is marked beta.  
Source: <https://thermotools.github.io/thermopack/vcurrent/thermo_methods.html>

### [TP03] ThermoPack: src/thermopack_var.f90

**Evidence:** Pinned source, inspected lines 1–220.  
Model registry, active-model pointer, model parameters and numerical limits, internal OpenMP-specific EOS arrays, apparent-composition infrastructure.  
Source: <https://github.com/thermotools/thermopack/blob/d68c794c7342bfc6938eb424a1fbb88b7780b738/src/thermopack_var.f90#L1-L220>

### [TP04] ThermoPack: addon/pyExamples/pseudocomponents.py

**Evidence:** Pinned source, inspected lines 1–180.  
Runtime cubic pseudocomponent example and explicit missing ideal-gas heat-capacity limitation; residual caloric properties only in described route.  
Source: <https://github.com/thermotools/thermopack/blob/d68c794c7342bfc6938eb424a1fbb88b7780b738/addon/pyExamples/pseudocomponents.py#L1-L180>

### [TP05] ThermoPack cubic methods

**Evidence:** Official documentation; moving URL unless version stated.  
Cubic initialization, pseudocomponents, alpha/mixing choices and parameter adjustment.  
Source: <https://thermotools.github.io/thermopack/vcurrent/cubic_methods.html>

### [CL01] Clapeyron: src/models/CompositeModel/CompositeModel.jl

**Evidence:** Pinned source, inspected lines 1–225.  
Explicit composite fluid/activity/correlation/solid construction, model validation and solid-to-fluid mapping.  
Source: <https://github.com/ClapeyronThermo/Clapeyron.jl/blob/4319d4fb7288aacc7969ca212e0792a691c9564e/src/models/CompositeModel/CompositeModel.jl#L1-L225>

### [CL02] Clapeyron: docs/src/properties/flash.md

**Evidence:** Pinned source, inspected lines 1–180.  
Algorithm/model/phase/specification compatibility matrix; initialization limits; active versus zero-fraction phases; tp_flash2 structured output.  
Source: <https://github.com/ClapeyronThermo/Clapeyron.jl/blob/4319d4fb7288aacc7969ca212e0792a691c9564e/docs/src/properties/flash.md#L1-L180>

### [CL03] Clapeyron: Project.toml

**Evidence:** Pinned source, inspected lines 1–95.  
Source snapshot identifies version 0.6.28; not a statement that it is the selected production release.  
Source: <https://github.com/ClapeyronThermo/Clapeyron.jl/blob/4319d4fb7288aacc7969ca212e0792a691c9564e/Project.toml#L1-L95>

### [CL04] Clapeyron user-defined parameters

**Evidence:** Official documentation; moving URL unless version stated.  
Single/pair/association data, submodel-specific overrides, default combining rules and absent association semantics.  
Source: <https://clapeyronthermo.github.io/Clapeyron.jl/stable/tutorials/user_defined_parameters/>

### [CL05] Clapeyron ideal models

**Evidence:** Official documentation; moving URL unless version stated.  
BasicIdeal heat capacity and alternative ideal-gas models; ideal contribution must be consciously selected for caloric work.  
Source: <https://clapeyronthermo.github.io/Clapeyron.jl/stable/eos/ideal/>

### [CL06] Clapeyron implicit differentiation

**Evidence:** Official documentation; moving URL unless version stated.  
Implicit differentiation of root solutions; implementation scope and historical higher-order caveat, not a guarantee of smooth phase transitions.  
Source: <https://clapeyronthermo.github.io/Clapeyron.jl/stable/implicit_differentiation/>

### [CL07] Clapeyron project overview

**Evidence:** Official documentation; moving URL unless version stated.  
Model families and project description.  
Source: <https://clapeyronthermo.github.io/Clapeyron.jl/stable/>

### [CH01] Thermo: thermo/flash/flash_vln.py

**Evidence:** Pinned source, inspected lines 1–235.  
FlashVLN constructor, required gas model, maximum liquid candidates, unsupported solids, algorithm controls and example.  
Source: <https://github.com/CalebBell/thermo/blob/2bb466e98439c2395a004e7095d07f9b97a5a0f0/thermo/flash/flash_vln.py#L1-L235>

### [CH02] Chemicals: chemicals/rachford_rice.py

**Evidence:** Pinned source, inspected lines 1–125.  
Rachford-Rice inner loop, fixed-K composition allocation, numerical edge cases; not a complete thermodynamic package.  
Source: <https://github.com/CalebBell/chemicals/blob/e79047588b30cfabc564c79fb26d760c746877d7/chemicals/rachford_rice.py#L1-L125>

### [CH03] Thermo data-package tutorial

**Evidence:** Official documentation; moving URL unless version stated.  
Constants/correlations separate from algorithms; five configuration sites; data quality caveat.  
Source: <https://thermo.readthedocs.io/chemical_package_tutorial.html>

### [CH04] Thermo phase and flash tutorial

**Evidence:** Official documentation; moving URL unless version stated.  
Phase-only evaluation, functional state changes, model/state hashes and JSON; explicit warning about incompatible phase-model mixtures.  
Source: <https://thermo.readthedocs.io/tutorial_phases_and_flash.html>

### [CH05] Thermo flash API

**Evidence:** Official documentation; moving URL unless version stated.  
FlashPureVLS, FlashVL and FlashVLN have different scope and data requirements; no uniform solids capability.  
Source: <https://thermo.readthedocs.io/thermo.flash.html>

### [CH06] Thermo stream API

**Evidence:** Official documentation; moving URL unless version stated.  
Legacy Stream limitations; recommendation to use newer flash/EquilibriumStream interfaces.  
Source: <https://thermo.readthedocs.io/thermo.stream.html>

### [CP01] CoolProp low-level API

**Evidence:** Official documentation; moving URL unless version stated.  
Reusable backend/state lifecycle, phase imposition, derivative distinctions, reference changes and existing instances.  
Source: <https://coolprop.org/coolprop/LowLevelAPI.html>

### [CP02] CoolProp mixtures

**Evidence:** Official documentation; moving URL unless version stated.  
Interaction-data dependence, phase envelope and mixture limitations; guidance has varied by documentation version.  
Source: <https://coolprop.org/fluid_properties/Mixtures.html>

### [CP03] CoolProp: src/Backends/Helmholtz/FlashRoutines.cpp

**Evidence:** Pinned source, inspected lines 3375–3730.  
Mixture P-H/P-S/P-U solve via inner PT updates, bracketing, fallback and final specification residual check; not the old limited-mixture-interface claim.  
Source: <https://github.com/CoolProp/CoolProp/blob/afce86ff977552663ca3a78d8ea318cc64dcbdfd/src/Backends/Helmholtz/FlashRoutines.cpp#L3375-L3730>

### [CP04] CoolProp developer documentation

**Evidence:** Official documentation; moving URL unless version stated.  
Backend architecture, release/testing documentation. No CI run or quantitative coverage audit performed.  
Source: <https://coolprop.org/develop/index.html>

### [CP05] CoolProp publisher package metadata

**Evidence:** Official documentation; moving URL unless version stated.  
Publisher-reported MIT licensing; versions and classifiers are not independent robustness validation.  
Source: <https://pypi.org/project/CoolProp/>

### [FE01] FeOS: crates/feos-core/src/phase_equilibria/mod.rs

**Evidence:** Pinned source, inspected lines 1–235.  
Phase-equilibrium state container, mostly two-phase functionality, phase fractions, optional total amount and total-caloric bound.  
Source: <https://github.com/feos-org/feos/blob/1b956cf7c55631b9be73abf2806058c85797bb04/crates/feos-core/src/phase_equilibria/mod.rs#L1-L235>

### [FE02] FeOS: crates/feos-core/src/phase_equilibria/px_flashes.rs

**Evidence:** Pinned source, inspected lines 1–200.  
Actual PH/PS implementations, two-phase initial-temperature precondition, total EOS requirement, implicit differentiation.  
Source: <https://github.com/feos-org/feos/blob/1b956cf7c55631b9be73abf2806058c85797bb04/crates/feos-core/src/phase_equilibria/px_flashes.rs#L1-L200>

### [FE03] FeOS documentation

**Evidence:** Official documentation; moving URL unless version stated.  
Parameter records/JSON, EOS/state/property/phase operations, entropy-scaling transport and surface examples. Features require model-specific data.  
Source: <https://feos-org.github.io/feos/>

### [FE04] FeOS repository overview

**Evidence:** Official documentation; moving URL unless version stated.  
Project scope and published scientific framework; dual MIT/Apache licensing. Primary repository overview, not a numerical audit.  
Source: <https://github.com/feos-org/feos>

### [RK01] Reaktoro: Reaktoro/Equilibrium/EquilibriumSolver.hpp

**Evidence:** Pinned source, inspected lines 1–245.  
Mutable in/out ChemicalState; separate specs, conditions, restrictions, result and equilibrium sensitivity interfaces; LGPL header.  
Source: <https://github.com/reaktoro/reaktoro/blob/f587235e692b7168faa272c8892a4c079a1f7dd9/Reaktoro/Equilibrium/EquilibriumSolver.hpp#L1-L245>

### [RK02] Reaktoro constrained equilibrium

**Evidence:** Official documentation; moving URL unless version stated.  
Chemical system, declared specification types, condition values, open components and reservoir exchanges.  
Source: <https://reaktoro.org/tutorials/equilibrium/equilibrium-specifying-constraints.html>

### [RK03] Reaktoro fixed-volume/internal-energy equilibrium

**Evidence:** Official documentation; moving URL unless version stated.  
Example using NASA data, unknown T/P, fixed conserved amounts, UV constraints and bounds; tutorial result, not a run by this audit.  
Source: <https://reaktoro.org/tutorials/equilibrium/equilibrium-with-fixed-volume-internal-energy.html>

### [RK04] Reaktoro chemical-property evaluation

**Evidence:** Official documentation; moving URL unless version stated.  
ChemicalProps can evaluate a supplied state without solving a new equilibrium; phase/system properties.  
Source: <https://reaktoro.org/tutorials/basics/computing-chemical-properties.html>

### [RK05] Reaktoro kinetics basics

**Evidence:** Official documentation; moving URL unless version stated.  
Kinetic mineral dissolution coupled to equilibrated aqueous species, rate-model assumptions and success checks.  
Source: <https://reaktoro.org/tutorials/kinetics/kinetics-basics.html>

### [RK06] Reaktoro license

**Evidence:** Official documentation; moving URL unless version stated.  
LGPL 2.1-or-later; database and dependency distribution not assessed here.  
Source: <https://reaktoro.org/general/license.html>

### Additional public repository licensing checks

The repository overview for Clapeyron reports MIT licensing: <https://github.com/ClapeyronThermo/Clapeyron.jl>. FeOS workspace metadata reports `MIT OR Apache-2.0`: <https://github.com/feos-org/feos/blob/1b956cf7c55631b9be73abf2806058c85797bb04/Cargo.toml>. These screening observations are not complete license or data-provenance audits.

### Project baselines

THERMO-SCOPE-001 v0.1, *Thermodynamics simulation-behavior scope baseline*, supplied in this conversation as `thermodynamics_simulation_behavior_scope_v0_1.md`.

THERMO-DWSIM-002 v0.1, *DWSIM workflow reverse-engineering*, supplied in this conversation as `dwsim_workflow_reverse_engineering_v0_1.md`. Its static audit and deferred probes are not reclassified as executed evidence here.
