# DWSIM workflow reverse-engineering report

**Document:** THERMO-DWSIM-002  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 2, reverse-engineer DWSIM from complete workflows  
**Predecessor:** THERMO-SCOPE-001 v0.1, `thermodynamics_simulation_behavior_scope_v0_1.md`  
**Status:** Source-backed conceptual analysis. No DWSIM build, numerical experiment, or runtime conformance test was executed.

## 0. Findings and decision

DWSIM is a useful primary reference because its thermodynamic functionality participates in complete process-model workflows: selecting and changing materials; assembling configured packages; resolving stream specifications; evaluating temporary states inside equipment; distributing material to products; coordinating reaction, column, and recycle iterations; preserving externally calculated states; and restoring a saved model with its shared definitions. The architectural value is in those responsibilities and interactions, not simply its catalog of property methods. [D01–D18]

The principal recommendation is to **retain the user-facing configured property package and flowsheet-oriented workflow breadth, while separating definition preparation, physical assumptions, numerical execution, result ownership, and model lifecycle more explicitly**.

This investigation also corrects an oversimplification in the earlier discussion. DWSIM is not exclusively a collection of property methods that mutate a stream. The inspected interfaces include explicit equilibrium-result objects, headless construction, internal solver-input objects, snapshots, dirty status, and persisted solutions. These are useful existing boundaries. They coexist with mutable material context, shared parameter objects, fixed phase slots, application-wide settings, and unit-specific result publication. [D01,D04,D06–D08,D15,D17]

### 0.1 Research boundary

The primary source is `DanWBR/dwsim10`, pinned to commit:

`a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3`

The commit metadata reports 25 September 2026, 12:41:04 UTC. This is a source snapshot, not a claim about the latest stable binary release. The older `DanWBR/dwsim` repository is archived; historical descriptions were not substituted for the active source. Search results were used to locate files, then relevant code was fetched at the pinned commit. [D00]

The audit follows selected paths through current engine code, the Fluent API, and one current desktop configuration helper. It is not an exhaustive audit of every unit, package, editor, solver, or platform. In particular, detailed petroleum-characterization algorithms, every chemical-equilibrium implementation, the complete CAPE-OPEN adapter, and every staged-column solver were not traced end to end.

### 0.2 Evidence conventions

- **Observed, code:** behavior directly visible in the inspected implementation.
- **Observed, documentation:** a statement in the pinned project's own documentation, not independently exercised here.
- **Inference:** a consequence suggested by multiple observed paths; runtime behavior remains to be tested where stated.
- **Design recommendation:** a proposed requirement for our simulator, not a description of DWSIM.
- **Not assessed:** no capability conclusion should be inferred from this audit.

References [D00]–[D18] identify the pinned sources and inspected regions in Section 12. Step-1 scenario identifiers are preserved. Source inspection does not upgrade any scenario to Executable or Validated under THERMO-SCOPE-001.

## 1. The actual responsibility and information map

### 1.1 Objects and responsibilities observed

| Responsibility | DWSIM location in the inspected paths | Information held or consumed | Important interaction |
|---|---|---|---|
| Simulation-wide configuration | `FlowsheetBase`, its options and collections | Selected compounds, configured packages, reactions, reaction sets, units, scripts, stored solutions, assays | Shared definitions are referenced by streams and units; save/load reconstructs these relationships. [D04] |
| Chemical/property definition | `ConstantProperties` objects in selected-compound collections | Identity, constants, property and characterization information | Multiple stream-phase compound objects point to the same selected definition. [D04,D05] |
| Local composition and quantities | `Compound` objects inside `Phase` objects | Fractions and amounts/flows at a material location | Material values are local; constant-property definitions are shared. [D04,D06] |
| Material state and specification | `MaterialStream` | State specification, phase dictionary, quantities, package binding, calculated properties, state flags | The stream selects equilibrium versus property-only behavior and coordinates many property calculations. [D06] |
| Configured thermodynamic package | `PropertyPackage` and subclasses | Method choices, phase policies, model parameters, flash settings, overrides, current material context | A package is much more than an EOS. Some setup work occurs when its material context is assigned. [D07,D08,D10] |
| Equilibrium orchestration | `UniversalFlash` and concrete flash algorithms | Requested specification, composition, phase policy, numerical settings, guesses | Selects algorithms, calls thermodynamic functions through the package, and may invoke a configured fallback. [D09] |
| Typed calculation result | `IFlashCalculationResult` and equilibrium entry points | Calculated state and phase-allocation results, result exception | Used directly by units such as valves and conversion reactors. [D07,D08,D11,D14] |
| Unit physical model | Valve, vessel, exchanger, reactor, column | Operating specification, balance relationships, connected streams, internal working data | Determines the problem to solve and how results are assigned to products. [D11–D16] |
| Nested unit solution | Reactor iteration and column solver inputs | Trial compositions, temperatures, stage flows, reaction extents, tolerances | Not all thermodynamics is a standalone stream flash. [D14–D16] |
| Plant-wide execution | `FlowsheetSolver` | Calculation order, queue, recycle convergence, cancellation, exception collection | Repeats local solves until the outer flowsheet conditions are met. [D17] |
| Model editing and recovery | Configuration helpers, snapshots, undo/redo | Before/after definitions, local values, dirty flags | Changes must repair or invalidate multiple dependent objects. [D04,D05] |
| Persistence and reconstruction | `SaveToXML`, `LoadFromXML`, snapshot/process-data restoration | Definitions, package data, object state, connections, reactions, assays, results | Restoring values is insufficient without restoring shared references and implementations. [D04] |
| External chemistry service | Documented Reaktoro integration contract | Chemical-system definition, species ordering, equilibrium/property requests | The integration exposes a smaller capability surface than the underlying library. [D18] |

### 1.2 Three different kinds of state are interwoven

The inspected paths distinguish some, but not all, of the following:

**Definition state:** compound constants, package parameters, selected methods, reactions, assay information.

**Problem and working state:** stream specifications, connected inputs, guesses, internal material clones, stage arrays, flash iterates.

**Result state:** evaluated properties, phase allocation, unit products, recycle convergence, stored solutions and calculation status.

For example, a material stream contains specifications and calculated quantities; a property package contains both configuration and a changing `CurrentMaterialStream`; a column owns numerical arrays for internal locations; the flowsheet stores both the model and saved solutions. These combinations are directly visible in the source. [D04,D06,D08,D15]

**Design recommendation:** preserve the relationships but do not require the same object to be the authority for all three kinds of state. A result should identify the definition and problem revisions under which it was obtained. A numerical workspace may be mutable without making the published material definition or accepted result mutable.

### 1.3 What “phase” means in this implementation

The common stream/package mapping uses fixed slots: overall mixture, aggregate liquid, vapor, individual liquid slots, aqueous, and solid. The inspected mapping includes mixture at 0, aggregate liquid at 1, vapor at 2, liquid instances at 3–5, aqueous at 6, and solid at 7. Flash algorithms also return positional arrays whose meanings vary with the specification; the package translates those values into the stream representation. [D06,D08]

The important distinction is that **aggregate liquid and overall mixture are not additional physical phases**. Likewise, a vapor outlet of a separator can have a prescribed phase allocation without requiring another independent equilibrium calculation. [D08,D12]

**Design recommendation:** distinguish phase category, phase instance, aggregate reporting view, and permitted phase participation. Keep provider-specific integers and positional arrays inside an adapter. This is a semantic requirement, not an expectation that every provider can solve an arbitrary number of phases.

## 2. Complete workflow traces

Each card identifies the observed route, the state mutations, its failure or completion boundary, and the implication for our conceptual architecture. A “trace” means static control/data-flow reconstruction, not an executed example. Branches outside the inspected regions are explicitly limited.

### WF-01. Create a flowsheet, select compounds, configure a package, and add a feed

**Scope links:** all scenarios; particularly SC-01, SC-05, SC-08 and X08, X14, X17.

**Initiator and inputs:** user or automation selects compound names, a package, package settings, and a feed specification.

**Observed sequence:** The Fluent API creates or wraps a flowsheet; `WithCompounds` delegates compound addition to the flowsheet. Package selection goes through package identification and creation, with a Plus-access check on that API surface for identified restricted integrations. A stream builder obtains a material-stream simulation object. Flowsheet helpers populate phases with local compound entries referring to shared constant-property definitions. Object creation also creates a graphical object and its connectors, including in headless creation paths. [D01,D03,D04]

**Ownership:** selected definitions and configured packages are flowsheet-level resources; material fractions and flows belong to the stream's phase-compound records. The connector representation participates in operational connectivity, not only rendering. [D04]

**Completion boundary:** successful construction means the configuration objects exist. It does not establish complete data, a supported calculation, a converged state, or a physically valid model. The flowsheet solver separately checks for selected compounds and at least one package. [D17]

**Design recommendation:** retain a convenient configured-package choice and a reusable selected-material definition. Separate package discovery, entitlement/availability, configuration validity, and calculation readiness. Make connectivity a domain model that the visual editor displays, rather than making graphical objects the authoritative location of ports.

**Limit:** the catalog's complete name-resolution and every package factory were not audited. The observed access gate does not imply that the underlying independently distributed library is restricted in the same way.

### WF-02. Specify or modify feed composition and flow

**Scope links:** SC-01 and X01–X04, X16, X21.

**Observed sequence:** The Fluent material builder exposes scalar state and flow setters. Its composition-setting operation replaces overall composition, whereas component-flow operations distinguish newly created feeds from existing ones. Newly created streams can initially contain equal component fractions; the builder tracks newly created streams so setting component flows does not unintentionally retain those defaults. Existing or loaded streams preserve unmentioned component flows in the relevant patch-style operation. [D03]

**Mutation:** specification values, component fractions and/or flows, and the inferred flow basis change on the same material object.

**Why this matters:** the semantic difference between “replace composition” and “edit one component” is real. It should not depend on an undocumented history of object construction. The inspected implementation contains explicit work to manage that difference. [D03]

**Design recommendation:** define distinct actions for replacing a complete composition, patching selected component amounts, changing a total-flow basis, and rescaling total amount without changing composition. Do not let default initial guesses acquire the status of user input. A change operation should declare the quantities preserved and the quantities derived.

**Required future checks:** zero-flow composition retention; whole-composition replacement versus partial edit; mass/molar consistency; reordering of components; equivalent behavior for GUI, loaded case, and fresh headless case.

### WF-03. Associate a material context and prepare missing parameters

**Scope links:** SC-08 and X08–X10, X16–X18.

**Observed sequence:** Assigning `PropertyPackage.CurrentMaterialStream` can cache compound definitions, invoke a post-association routine, and check compounds. In the NRTL package, the post-association routine can estimate missing binary interaction parameters when automatic estimation is enabled. The base package's inspected default enables automatic NRTL/UNIQUAC estimation. [D08,D10]

For the NRTL estimation branch inspected, DWSIM builds temporary binary material/package objects, computes activity coefficients with a group-contribution model at selected compositions and temperature, and fits NRTL parameters. Other special-case branches also exist. When estimation fails, the inspected catch path sets very small interaction values and warns that the pair will be treated as ideal. This is **not a silent failure in that path**, but it is a change to the physical description rather than a purely numerical retry. [D10]

**Mutation:** package interaction data can change as a side effect of attaching material context. The routine also uses shared control state to avoid recursive estimation. [D10]

**Completion boundary:** a package can become numerically usable through an estimate or substitute, but that does not mean the original requested model was evaluated using established data.

**Design recommendation:** make data resolution and estimation explicit preparation actions. Record measured, imported, fitted, estimated, and substituted values separately, along with the derivation and applicable domain. An evaluated request should use a resolved configuration revision. A fallback that treats a pair as ideal should yield an explicitly qualified model outcome.

**Inference requiring a probe:** saving a package can temporarily associate a material stream [WF-14]. In conjunction with this hook, that creates a possible save-time preparation side effect under suitable conditions. The combination is visible statically; its occurrence in a particular saved case was not tested.

### WF-04. Resolve a stream specification and calculate its properties

**Scope links:** SC-01–SC-14; X01–X07, X09–X11, X14, X23.

**Observed sequence:** `MaterialStream.Calculate` decides whether equilibrium must be recalculated or only properties refreshed. It accounts for an externally calculated-equilibrium marker and an option to preserve already defined equilibrium results in certain paths. The detailed calculation then validates inputs, resolves mass/molar/volumetric flow relationships, associates the package's current material context, and dispatches the selected state specification to equilibrium calculation when needed. It subsequently calculates present-phase properties, clears absent-phase properties, evaluates aggregate properties and flow conversions, and records solution-input information at the successful tail. [D06]

The inspected stream code contains a context-dependent special case for a single-component TP specification downstream of a unit, where PH may be selected with a warning. This is an example of a specification interpretation influenced by the surrounding workflow rather than by the two scalar values alone. [D06]

**Mutation and failure:** the mutable package equilibrium path resets equilibrium status and clears old phase values before solving. An exception can therefore occur after portions of the working state have changed. This does not establish the absence of all recovery mechanisms, but the inspected path does not provide an atomic accepted-result replacement contract. [D08]

**Properties versus equilibrium:** these are distinct operations. Calculating viscosity, density, or enthalpy for supplied phases should not necessarily replace their allocation. Some additional property calculations have local failure handling rather than being mandatory for every state. [D06,D08]

**Design recommendation:** separate a requested state specification, a candidate resolved state, a property request, and an accepted result. Define whether a request is allowed to change phase allocation. Validate independent specifications explicitly instead of relying on unit-type heuristics as the fundamental domain rule.

### WF-05. Select an equilibrium algorithm, run it, and recover from failure

**Scope links:** SC-05, SC-08, SC-10–SC-14, SC-24, SC-34; X05–X07, X09, X23.

**Observed sequence:** `UniversalFlash.Flash_PT` validates composition and scalar inputs, checks a user-defined override, and selects an equilibrium calculation type. Under its default selection behavior it uses heuristics involving solid participation, liquid splitting, and forced-solid choices. It routes to VLE, VLLE, SVLE, SVLLE, or no-flash behavior and selects corresponding numerical implementations, including special immiscible-water handling. [D09]

The algorithm receives the configured package, allowing equilibrium iteration to request relevant thermodynamic evaluations. The outer package API exposes typed results even though some inner numerical routes return positional arrays. [D07–D09]

**Failure path observed:** a configured fail-safe mode can retry rigorous VLE with the same package, use an ideal Raoult package, return a no-flash state, or rethrow the initial exception. The inspected code records the fail-safe choice in the calculation inspector; ordinary warning calls in those particular branches are commented out. The audit does not establish which mode every newly created package defaults to. [D09]

**Semantic consequences:** retrying VLE after a broader multiphase request can restrict the permitted equilibrium, even when the package is unchanged. Ideal-model substitution changes thermodynamics. No-flash behavior changes the operation being performed. None is automatically equivalent to a same-problem retry.

**Design recommendation:** classify retries as same-model numerical recovery, altered-phase-policy recovery, alternate-model calculation, or incomplete/degraded result. Require requested and actually used methods, data, phases, and algorithm to be inspectable. A heuristic decision is not itself a global-stability certificate.

### WF-06. Calculate a pressure-reducing valve and publish its outlet

**Scope links:** SC-06; X02, X06, X11, X15, X23.

**Observed sequence:** The valve validates connections or uses explicitly provided inlet/outlet objects, binds its package to the inlet, and reads inlet temperature, pressure, mass enthalpy, and mass flow. The unit's operating mode determines outlet pressure. The isenthalpic target is the inlet enthalpy. An outlet-temperature estimate can reuse an earlier outlet value. The valve then calls the typed pressure–enthalpy equilibrium interface and checks the returned temperature and enthalpy. [D11]

**Publication:** the inspected normal path writes outlet pressure, calculated temperature, target enthalpy, total flow, and composition. It sets the stream specification to PH and marks `AtEquilibrium = False`. It does not simply install the complete phase allocation from the returned flash result. The outlet-stream calculation later completes or refreshes that material state. [D11]

**Failure and invalidation:** connection and state checks can throw. `DeCalculate` clears selected outlet values and composition information and marks the graphic uncalculated. A unit-level numerical calculation, publication of outlet constraints, and final stream-property completion are therefore distinct events. [D11]

**Design recommendation:** retain the separation of the valve's process rule from thermodynamic resolution. Make the product publication contract explicit: does the unit publish constraints, a fully evaluated state, or an externally authoritative allocation? Reusing an evaluated result should be possible where valid, but avoiding duplicate work must not erase the distinction between target values and accepted state.

### WF-07. Mix feeds, resolve a separator's contents, and route phases to products

**Scope links:** SC-01, SC-05, SC-10, SC-11, SC-23; X03, X05, X11, X14, X22, X23.

**Observed sequence:** The separator creates an internal `MixedStream`, populates its material definitions, validates connected feeds, aggregates component mass and enthalpy flows, and determines an operating pressure using the selected pressure behavior. It then selects a calculation route based on its operating mode. In the inspected adiabatic branch, a single feed can be copied with its phase information without a new flash; multiple feeds trigger a PH calculation. Temperature/pressure overrides can select a TP calculation, with heat accounting handled by the unit. Other modes use heat-adjusted enthalpy and, in one inspected route, an outer pressure search around a PH calculation. [D12]

**Routing is separate:** after determining the internal state, the vessel distributes vapor and liquid material to connected products. In the inspected multi-outlet path, solid material is apportioned between liquid outlets according to their liquid mass ratio. The code evaluates liquid densities to decide the outlet ordering. These are equipment/product-routing rules layered over the equilibrium state, not additional equilibrium identities. [D12]

**Publication:** the vapor outlet is assigned pressure, enthalpy, flow, and composition, with vapor allocation marked as already determined. Liquid outputs can likewise be given prescribed allocation when the relevant conditions apply. Empty outlets receive specific handling. [D12]

**Design recommendation:** distinguish mixing, equilibrium/state resolution, and product allocation. A phase instance must be independent of a light/heavy product port. Solids carryover, phase entrainment, and density-based routing belong in explicit unit rules. An absent product should not be represented by a falsely present phase with invented composition.

**Limit:** this trace covers selected steady-state branches. It does not certify arbitrary multiphase cases or every separator operating mode.

### WF-08. Couple two thermodynamic regions through a heat exchanger

**Scope links:** SC-02, SC-12, SC-28; X09–X11, X14, X22, X23.

**Observed sequence:** The exchanger validates its two feeds, identifies hot and cold sides, and reads each side's state and flow. It uses material clones for hypothetical property evaluations and the respective inlet streams' packages for side-specific calculations. In the inspected thermal-efficiency branch, the unit determines heat exchange, updates each side's target enthalpy using its own mass flow and heat-loss convention, and calls separate PH equilibrium calculations for the hot and cold sides. LMTD, area, and other equipment calculations remain in the unit. [D13]

**An instructive boundary case:** a hypothetical bounding state used to estimate maximum exchange can lie outside a side's property-package coverage even when the real process states are acceptable. The inspected cold-side bounding calculation has local failure handling so that its failure need not be treated as failure of every possible exchanger calculation. This source path motivates distinguishing essential state requests from auxiliary estimates; it does not establish a universal policy for every exchanger branch. [D13]

**State ownership:** there is no material conversion between the two sides. The coupling is heat. Internal clones are not necessarily flowsheet connections. The dynamic source additionally shows per-cell material states using the respective side packages, although the time-stepping algorithm was not validated in this audit. [D13]

**Design recommendation:** retain separate thermodynamic bindings for each material region. Do not require a shared component slate or identical absolute enthalpy reference merely to exchange heat. Require consistent within-side energy differences. Define optional reporting calculations separately from essential balances and prevent their failures from silently replacing required physical calculations.

**Limit:** selected side-property and energy-resolution branches were traced, not the complete set of exchanger sizing/rating modes or every final-output assignment.

### WF-09. Configure reactions, transform composition, and resolve reactor energy and products

**Scope links:** SC-17–SC-19, with partial relevance to SC-20–SC-25; X04, X07, X11, X12, X14, X23.

**Configuration observed:** flow-level reaction constructors distinguish conversion, equilibrium, kinetic, and heterogeneous-catalytic definitions. They hold stoichiometry, reference reactant, reaction phase, and calculation basis. Kinetic definitions separately carry reaction orders and forward/reverse expressions or Arrhenius parameters. Equilibrium definitions can use a supplied expression or a Gibbs-based option. [D04]

**Conversion-reactor sequence observed:** the unit clones the inlet into an internal material, binds its package, resolves reaction groups, and evaluates specified conversions. The inspected optimization seeks requested conversions subject to material availability, so the resulting conversions can be limited rather than blindly enforced. The unit updates species flows and composition, calculates reaction heat contributions, and then resolves the resulting thermal state. Adiabatic operation uses a PH request; isothermal or specified-outlet-temperature operation uses TP behavior and heat accounting. A subsequent equilibrium result is used for product-phase information and outlet assignment. [D14]

**Responsibility boundary:** reaction orchestration and heat accounting are not all hidden inside the property package. The reactor uses reaction definitions and thermodynamic services together. In this path, grouping and sequencing are also process-model choices. [D14]

**Design recommendation:** separate reaction definitions, conversion/kinetic/equilibrium assumptions, transformation results, thermal constraints, and phase resolution. A requested conversion not achieved because of availability must be reported as such. Make the formation-property/reaction-heat convention explicit so it is neither omitted nor counted twice when a different provider is substituted.

**Limit:** the conversion path was traced in detail. The presence of equilibrium and kinetic reaction constructors is not an audit of all equilibrium, Gibbs, CSTR, or PFR execution paths. Coupled reactive separation and electrolyte energy balances remain unverified.

### WF-10. Prepare internal column states and couple thermodynamics to a staged solver

**Scope links:** SC-07–SC-10 and SC-26; X06, X09, X14, X15, X19, X23.

**Observed sequence:** column preparation gathers stage pressures, feeds, side draws, and interstage heat inputs. It builds arrays for stage temperatures, liquid/vapor flows, compositions, K-values, and related data. It uses user estimates or generated estimates, obtains K-values or initial phase information through the package, and assembles a `ColumnSolverInputData` object containing physical inputs, estimates, specifications, and numerical controls. More than one estimate-generation routine is present in the inspected source. [D15]

**Key architectural evidence:** internal locations are not represented only as flowsheet stream connectors. A unit can have a substantial mathematical representation whose property requests use stage-indexed numerical data and temporary package material context. A general simulator cannot force all local thermodynamics through “create a stream and flash it.” [D15]

**Rate-based qualification:** the inspected `RigorousColumnRateBased` implementation retains equilibrium-stage MESH equations and iterates component/stage efficiencies derived from transfer correlations. It rates a solution, updates efficiencies, and solves again until the change meets a tolerance or a pass limit is reached. If the pass limit is reached, it records that the efficiencies did not settle and retains the last set. [D16]

This is useful functionality but does not by itself demonstrate SC-26's intended full nonequilibrium formulation with separately modeled bulk/interface states and potentially separate bulk temperatures.

**Design recommendation:** retain the separation between unit mathematical structure, initialization, thermodynamic evaluation, and numerical solver. Distinguish fixed physical inputs, initial guesses, and calculated stage results even when a solver API groups them. Describe the actual model formulation, not just a feature label such as “rate-based.” Model nested convergence and its qualified outcomes explicitly.

**Limit:** column setup and the rate-correction loop were inspected; every main solver, final product write, and convergence strategy was not audited end to end.

### WF-11. Solve a flowsheet and close recycles

**Scope links:** integrated journeys J01–J07; X14–X18, X23.

**Observed sequence:** the flowsheet solver verifies basic configuration, determines calculation order by traversing connector relationships, and creates a queue of simulation-object calculations. Explicit recycle objects affect ordering. A traversal guard reports a loop problem when the graph cannot be ordered as expected. The execution path invokes before/after specifications and scripts, evaluates streams and units, updates statuses, collects exceptions, and respects cancellation/settings. [D17]

**Outer convergence:** the inspected route repeatedly queues the ordered objects, marks stream equilibrium status and calculation flags for refresh, processes the queue, then checks recycle convergence. Optional global Broyden behavior updates recycle variables. Local unit success is therefore not synonymous with plant-wide convergence. The inspected code also uses shared solver/calculator settings and shared event/inspector machinery. [D17]

**Publication and failure:** objects and outlets can be updated during iterations; exceptions can be collected and surfaced after parts of a pass have run. The source does not establish an atomic whole-flowsheet result publication contract. This is a description of the inspected route, not a claim that DWSIM has no stored solutions or recovery features. [D04,D17]

**Design recommendation:** distinguish execution order, numerical iteration structure, and accepted model state. A process graph should not require users to model a numerical tear as a physical transformation, although an explicit recycle block can remain a useful user interface. Associate output validity with the convergence level reached. Shared settings and mutable package contexts create isolation requirements to test, not proof that every parallel route is unsafe.

### WF-12. Add or remove a selected compound

**Scope links:** SC-01, SC-15, SC-21, SC-30; X01, X04, X08, X16–X18.

**Observed sequence:** the inspected current desktop helper snapshots compound state before changes. Addition delegates to the flowsheet and ensures the compound exists in every material stream's phase dictionary with the selected shared definition. Removal changes the authoritative selected-compound collection, subtracts the component's carried quantities from stream totals, removes phase entries, clears calculated properties, normalizes remaining compositions, and marks the affected streams uncalculated. It also removes the compound from package forced-solid lists. [D05]

The helper deliberately avoids mutating a sorted copy of the selected-compound dictionary. In the flowsheet implementation, nondefault ordering modes can return a reordered dictionary while retaining the shared definition objects. [D04,D05]

**Semantic consequence:** removing a component is not merely deleting a row in a catalog. It is a choice about how material quantities change, what is renormalized, and which dependent configurations must be repaired.

**Design recommendation:** make this a reusable domain action with an impact preview and explicit policy. Depending on intent, removing a component might preserve total flow and renormalize, preserve the remaining component flows, remove the material physically, or be rejected because active reactions require it. These must be distinct, named choices. All front ends should use the same change semantics.

**Limit:** the inspected helper demonstrates these specific mutations. The complete behavior of every calling editor, automation path, reaction repair, and subsequent solve trigger was not audited; no interface-consistency defect is claimed without testing.

### WF-13. Change package configuration, invalidate results, and undo the change

**Scope links:** all families; X08, X10, X11, X16–X18.

**Observed configuration mutation:** the Fluent package configuration writes flash choices and settings, and edits model-specific interaction tables. The inspected PR/SRK path assigns pair coefficients; NRTL has directed parameters; Wilson uses a different compound-key convention. Generic configuration hooks can reach additional package state. These edits demonstrate that parameter identity and pairing conventions are model-specific. [D02]

**Observed invalidation:** `ResetCalculationStatus` marks every simulation object dirty and clears numerical and graphical calculated flags. Compound snapshot restoration updates existing selected-definition instances, repairs the selected set, and invokes reset behavior. Undo/redo captures a current snapshot, restores a prior one, and maintains the opposite stack while refreshing the interface. [D04]

**What is not established:** a uniform, dependency-complete invalidation transaction after every possible arbitrary package mutation was not demonstrated by the inspected setters. The audit also does not prove that all changes force full recalculation; there are multiple execution paths.

**Design recommendation:** retain undo/redo and dirty-state concepts but give model changes explicit revisions and dependency effects. Separate changing parameter values, changing phase eligibility, changing numerical tolerances, and editing display units. A physical-model change must invalidate thermodynamic results; a display-only change should not. A restored historical result should not automatically become current under a different parameter revision.

### WF-14. Save the model, reload it, and restore the relationships needed to calculate

**Scope links:** SC-15, SC-30, SC-32; X08, X16–X18, X23.

**Save sequence observed:** the flowsheet writes build/version information, simulation objects, settings, graphics/connectivity, package data, compound definitions, reaction sets, reactions, stored solutions, dynamic data, assays, particle-size distributions, and results. Petroleum assays are persisted separately from the compound records generated or used by the model. [D04]

When a configured package has no current material context, the inspected save path creates a temporary material stream, populates compounds, associates it with the package for serialization, then detaches it. This is a concrete dependency of serialization on runtime context. [D04]

**Load sequence observed:** the loader applies compatibility transformations, restores settings and compound definitions, resolves package implementations by saved type/name, reconstructs simulation and graphic objects, and reconnects phase-compound entries to shared selected constant-property objects. Reactions, assays, and stored solutions are restored separately. Missing package implementations produce collected errors; the loader can report a partially loaded model rather than simply fail before reconstructing anything. [D04]

**Restoration is not only values:** process-data and snapshot restoration also repair definition references. Some snapshot restoration intentionally updates existing definition objects to preserve those shared references. [D04]

**Design recommendation:** serialize resolved configuration without requiring a working evaluation context. Retain both characterization inputs and resulting material definitions. Separate “document loaded for inspection,” “dependencies resolved,” “model ready to calculate,” and “saved result valid for this revision.” Record provider and data versions as part of reproducibility. Preserve an unresolved model for inspection without accidentally permitting calculation with a substitute package.

**Limit:** no save/reload equivalence experiment was performed. The source's migration branches and stored fields establish behavior to test, not numerical reproducibility by themselves.

### WF-15. Preserve an external result and constrain an external library's usable surface

**Scope links:** SC-18, SC-21, SC-22, SC-29, SC-30; X05, X07, X09, X11–X14, X17, X23.

**Material-result ownership observed in code:** a material stream can carry a one-shot indication that equilibrium was calculated externally. The inspected calculation branch then refreshes properties without replacing the externally supplied split. The comments identify external/CAPE-OPEN calculations as a motivation, including sensitivity around saturation. [D06]

**External-chemistry surface observed in documentation:** the pinned Reaktoro contract identifies four DWSIM call sites: TP equilibrium, repeated equilibrium during a bubble-temperature loop, activity evaluation at supplied composition without equilibrium resolution, and an element-driven Gibbs-reactor/system/database pathway. It explicitly says those call sites do not request kinetics, transport, surfaces, or sensitivity derivatives. The contract also specifies species ordering, lifecycle ownership, and error behavior. [D18]

**Consequence:** an upstream library's full capability catalog cannot be treated as the capability of the DWSIM adapter, nor can a successful activity evaluation be treated as an equilibrium solve. Different integrations may expose different subsets or meanings of the same upstream software.

**Design recommendation:** define provider capabilities at the operation-and-semantics level. Identify who owns species amounts, phase allocation, caloric quantities, and their reference conventions. A property-only operation must preserve the quantities the caller has declared authoritative, but the system must still be able to assess whether the supplied state and requested property model are compatible.

**Limit:** the Reaktoro integration's own contract was read, not every external native implementation or deployed runtime. This is not a validation of energy-consistent reactive flowsheet execution.

## 3. Reconstructed information flows

### 3.1 Configuration flow

```text
User/automation choices
  -> flowsheet selected material definitions
  -> configured property package and reaction definitions
  -> local stream/unit bindings
  -> parameter checks and, in some packages, context-triggered estimation
  -> state and unit specifications
  -> execution readiness checks
```

The final two transitions are not equivalent. Existing objects do not establish an executable model; obtaining estimates does not establish validated data. [D01–D04,D08,D10,D17]

### 3.2 Ordinary state-resolution flow

```text
State specification and composition
  -> MaterialStream.Calculate
  -> decide equilibrium calculation versus property-only refresh
  -> assign material context and resolve quantity conventions
  -> package equilibrium interface
  -> flash selection / numerical iteration / property callbacks
  -> phase allocation and resolved intensive state
  -> phase properties and aggregate properties
  -> stream quantities and calculation bookkeeping
```

The typed-result interface is an alternate callable boundary used by unit models. Some units publish outlet constraints afterward rather than copying the complete returned equilibrium state. [D06–D09,D11]

### 3.3 Equipment-internal and plant-wide flow

```text
Accepted or current feed values + unit operating constraints
  -> local workspaces (material clones or stage/reaction arrays)
  -> repeated thermodynamic evaluations
  -> unit balance / stage / reaction solution
  -> product specification or state publication
  -> downstream stream/property completion
  -> downstream units
  -> recycle comparison and another flowsheet pass, when necessary
```

This is a conceptual consolidation of the observed paths, not one universal DWSIM function that performs every step. [D11–D17]

### 3.4 Change and restoration flow

```text
Change request
  -> optional snapshot
  -> change shared definitions and/or local specifications
  -> repair dependent references, compositions, and settings
  -> clear or mark affected results
  -> recalculate
  -> new result / qualified failure

Saved document
  -> compatibility handling
  -> definitions and provider implementations
  -> objects and connections
  -> reference rebinding
  -> readiness assessment
  -> calculation under reconstructed configuration
```

The audit found concrete examples of each major responsibility but not a single uniform transaction encompassing every editor and arbitrary configuration mutation. [D04,D05,D17]

## 4. Assumptions and recovery policies that must become explicit

The following table distinguishes observed DWSIM behavior from proposed requirements. It is not a defect ranking.

| Observed choice | Source | Why its meaning matters | Recommended explicit representation |
|---|---|---|---|
| Package selects more than an EOS: caloric, density, transport, vapor-fugacity, phase, and estimation choices coexist | D08 | A package name does not identify a complete model configuration | Resolved configured package with all consequential choices |
| Associating material can trigger interaction-parameter estimation | D08,D10 | Evaluation context can affect the definition being evaluated | Explicit parameter-preparation action and resulting data revision |
| Failed NRTL estimation can install near-ideal parameters with a warning | D10 | Calculation may proceed under a substitute description | Qualified substitution record, not ordinary same-model success |
| Universal flash uses phase heuristics | D09 | Chosen phase scope is not just a convergence tolerance | Requested phase policy, selected phase search, and stability evidence |
| Fail-safe can restrict to VLE, change to an ideal package, or omit equilibrium | D09 | Recovery can change the physical problem | Distinct same-model retry versus alternate-problem result |
| Stream calculations can preserve external equilibrium allocations | D06 | Property evaluation must not always take ownership of phase amounts | Explicit authority for allocation and permitted state mutations |
| A valve writes PH constraints after obtaining a flash result | D11 | Unit completion and stream completion are separate | Product publication kind and convergence level |
| Separator routes liquid products using density and distributes solids by a mass-ratio rule | D12 | Product assignment is an equipment closure, not equilibrium | Named routing and carryover policies |
| Column preparation includes generated guesses and dedicated stage arrays | D15 | Not every intermediate location is a standalone stream | Internal location/state model and separate initialization data |
| Rate-based efficiency passes can retain the last set after a limit | D16 | A returned model can have an unresolved nested iteration | Nested convergence diagnostics and acceptance rules |
| Compound removal changes quantities and normalizes the remainder | D05 | Catalog editing can change process material | Explicit removal policy and affected-model preview |
| Save may construct a temporary material context | D04 | Serialization can depend on runtime association behavior | Side-effect-free serialization of resolved definitions |
| Missing package implementations can yield a partially restored model | D04 | Successful inspection is not calculation readiness | Separate import/readiness/result-validity states |
| Shared current contexts and global solver settings appear in the execution path | D08,D17 | Isolation cannot be assumed from object count | Defined session/workspace isolation plus runtime tests |

## 5. Lifecycle and change-impact requirements derived from the traces

### 5.1 Status needs several independent dimensions

The source contains `Calculated`, `AtEquilibrium`, dirty state, stored input information, recycle convergence, exception lists, snapshots, and stored solutions. Those mechanisms already recognize multiple aspects of completion. [D04,D06,D17]

For our design, use the following **conceptual dimensions**, not one linear status enum:

| Dimension | Questions it must answer |
|---|---|
| Definition readiness | Are materials, methods, parameters, and required implementations resolved? Were estimates/substitutions used? |
| Specification readiness | Is the state intentionally partial, independently specified, overdetermined, or inconsistent? |
| Calculation outcome | Was a request not attempted, completed, failed, cancelled, or completed under changed assumptions? |
| Physical authority | Which actor supplied or solved composition, phase allocation, intensive state, and energy quantities? |
| Convergence level | Did the property request, equilibrium problem, unit model, and outer flowsheet each converge? |
| Validity | Is the result supported by the chosen model/data range and the acceptance checks? |
| Currency | Does the result match the present definitions and specifications, or is it stale/historical? |
| Publication | Is it a private trial, inspectable candidate, accepted local result, or accepted flowsheet result? |

These dimensions can coexist. A locally converged phase state may belong to an unconverged recycle iteration. A numerically successful result can be outside a model's validated domain. A historical result can remain valuable while no longer being current.

### 5.2 Change-impact matrix for the later specification

This is a proposed requirement matrix informed by the observed edit/restore paths, not a claim that DWSIM implements these exact dependency rules.

| Change | Definitions affected | Results to invalidate or reassess | Additional obligations |
|---|---|---|---|
| Feed temperature, pressure, flow, or composition | Local specification revision | Local state and dependent units/recycles | Preserve explicit input intent; distinguish replacing versus patching composition |
| Add/remove a selected component | Material representation and compatible models/reactions | Compositions, ordering-dependent arrays, parameters, states, units | Declare conservation/removal policy; validate active reactions and mappings |
| Replace constants or pure-property correlations | Parameter/data revision | All dependent property results and unit balances | Track provenance and validity, not only the new scalar values |
| Change binary interactions or mixing rule | Package/model revision | Equilibrium, caloric terms, phase identities, dependent equipment | Reinitialize when old phase/branch guesses are incompatible |
| Restrict phases or declare forced solids | Physical-assumption revision | Equilibrium and downstream phase routing | Never classify as a purely numerical change |
| Change flash algorithm/tolerances | Numerical-policy revision | Convergence evidence and potentially selected branch | State whether prior physical result remains acceptable; record used algorithm |
| Change reaction set, ordering, kinetics, or basis | Chemical/process-model revision | Species, reaction heat, energy balance, downstream states | Recheck species/element/charge and unit requirements |
| Change a package assignment | Thermodynamic-region binding | State, property and energy compatibility | Treat material boundaries separately from heat-only coupling |
| Restore a snapshot | Restored definition/specification revision | All mismatched results and cached data | Rebind identities safely; preserve historical versus current distinction |
| Change display units or graphical position | Presentation state only, unless an input value changes | No physical invalidation should be needed | Do not conflate layout with process topology or numerical semantics |
| Upgrade a provider or import an old case | Implementation/dependency revision | Reproducibility and readiness assessment | Expose migrations, missing dependencies, and numerical revalidation needs |

## 6. Retain / separate / adapt / exclude assessment

### 6.1 Retain

**Configured property packages as an engineering concept.** A user should select and maintain a coherent property-method configuration rather than manually wire every internal primitive. The breadth of options in DWSIM demonstrates the usefulness of that concept. [D08]

**Shared material definitions with local state values.** Reusing definitions across streams and internal locations is appropriate. Preserve identity and consistency without requiring reference aliasing to be the only mechanism. [D04,D06]

**Property-only and equilibrium operations as distinct behaviors.** Both are necessary, particularly when an external unit owns a phase split. [D06]

**Typed numerical results and internal solver input structures.** DWSIM already has these useful boundaries; build on the distinction rather than assuming all useful modularity must be invented. [D07,D15]

**Internal material locations, initialization, nested solvers, diagnostics, and recovery.** Real flowsheets require all of these. Retain the functionality even where the ownership changes. [D11–D17]

**Snapshots, undo/redo, explicit invalidation, migrations, and saved solutions.** They belong to model semantics, not merely the graphical application. [D04,D05]

### 6.2 Separate

Separate package definition from runtime material context and cached workspace. Separate parameter preparation/estimation from evaluation. Separate physical phase policy from numerical algorithm choice. Separate equipment balance/transfer/routing rules from material constitutive relationships. Separate authoritative process connectivity from graphical rendering. Separate diagnostic reporting requests from essential calculation requirements. Separate local iteration outputs from accepted whole-flowsheet results.

These are design recommendations arising from the concrete combinations traced above, not a requirement to make every concept a separately deployed service.

### 6.3 Adapt

Adapt fixed phase slots into explicit phase instances and aggregate views. Adapt positional numerical outputs into stable semantic results. Adapt mixed component-key conventions into material identity mappings with explicit ordering. Adapt application-wide settings into scoped calculation policies. Adapt warning-only alternate-model outcomes into structured qualified results. Adapt current dirty flags and snapshots into revision-aware result currency. Adapt unit-specific finalization rules into explicit publication contracts.

A provider adapter may keep its own stateful implementation internally. The goal is reliable boundaries, not a prohibition on numerical mutability.

### 6.4 Exclude from the new domain core

Do not reproduce dependence on graphical connector objects as the authoritative process graph. Do not make model preparation a hidden effect of reading or attaching a calculation context. Do not let arbitrary extension properties substitute for specified semantics needed by supported scenarios. Do not use magic phase indexes or an undifferentiated “calculated” flag as the universal meaning of state. Do not classify ideal substitution or phase suppression as an ordinary same-problem success. Do not infer a provider's usable capability from a menu entry or upstream library name alone.

These exclusions concern the new domain core. They do not imply deleting useful user interfaces, legacy-file adapters, stateful provider wrappers, or optional pragmatic engineering approximations.

## 7. Mapping the findings back to THERMO-SCOPE-001

### 7.1 Scenario evidence coverage

**T:** a representative execution path was traced in source.  
**P:** a relevant interface, internal boundary, or partial path was inspected.  
**N:** the scenario was not assessed sufficiently to characterize DWSIM behavior.

Neither T nor P means numerically executable or validated in this work.

| Step-1 scenarios | Audit status | Evidence and implication |
|---|---|---|
| SC-01 mixing/splitting | P | Feed editing and separator feed aggregation traced; the dedicated mixer/splitter pair was not audited. [D03,D12] |
| SC-02 sensible heating / heat exchange | T | Selected two-sided exchanger energy/property path traced; not every mode or dedicated heater. [D13] |
| SC-03 pump/pipe; SC-04 compression/expansion | N | Central property/flash interfaces are relevant, but these unit workflows were not traced. |
| SC-05 cooling and VLE separation; SC-06 valve flashing | T | Separator state resolution/product routing and valve PH calculation traced. [D11,D12] |
| SC-07 staged distillation | P | Stage preparation, estimates, and solver-input boundary traced; not all solvers/output paths. [D15] |
| SC-08 nonideal separation | P | NRTL configuration, estimation, and flash framework inspected; no azeotrope fixture solved. [D08–D10] |
| SC-09 physical absorption | P | Relevant column setup and property boundaries present in inspected paths, not a complete absorption test. [D15,D16] |
| SC-10 LLE; SC-11 VLLE | P | Phase-policy dispatch, extractor initialization, and separator phase routing inspected; no numerical equilibrium validation. [D09,D12,D15] |
| SC-12 steam/water; SC-13 pure refrigerant; SC-14 mixed refrigerant | P | State-specification and phase-preservation mechanisms inspected; specialized model behavior not audited. [D06,D13] |
| SC-15 assay pseudocomponents | P | Assay and definition persistence inspected; characterization generation/regression not traced. [D04] |
| SC-16 black oil | N | No adequate model execution trace. |
| SC-17 conversion reaction | T | Definition, internal composition update, thermal resolution, and product-phase path traced. [D04,D14] |
| SC-18 chemical equilibrium | P | Reaction definition and external-chemistry contract inspected; general Gibbs/equilibrium solver not audited. [D04,D18] |
| SC-19 kinetics | P | Reaction-definition semantics inspected; full CSTR/PFR solution not traced. [D04] |
| SC-20 reactive separation | N | Ordinary column and reactor paths must not be combined into an unsupported capability claim. |
| SC-21 electrolyte; SC-22 reactive aqueous absorption | P | External chemistry's scoped contract inspected; energy coupling and full unit workflow remain unverified. [D18] |
| SC-23 inert solids in fluids | P | Solids distribution in separator products inspected; generalized slurry/solid property support not established. [D12] |
| SC-24 crystallization; SC-25 reactive gas-solid | P | Solid-phase flash selection / chemistry boundary identified; complete process workflows not traced. [D09,D18] |
| SC-26 full nonequilibrium contacting | P, formulation mismatch | The inspected rate-based column uses iterated efficiencies with equilibrium-stage equations; this does not establish the full requested formulation. [D16] |
| SC-27 adsorption/membranes | N | Outside the inspected integration surface; no conclusion on other DWSIM components. |
| SC-28 heat-only package coupling | T | Side-specific package usage in exchanger traced. [D13] |
| SC-29 material package transition; SC-30 representation translation | P | Assignment, external-result ownership, and species-order contracts are relevant; a general conservative translator was not established. [D06,D18] |
| SC-31 empirical mass-only materials | N | No adequate material-model execution trace. |
| SC-32 polymers/distributions | P | Distribution persistence observed; polymer thermodynamic behavior not audited. [D04] |
| SC-33 inventory/dynamic compatibility | P | Internal accumulation/cell states sampled; time integration and inventory semantics not validated. [D13,D14] |
| SC-34 constrained/metastable states | P | Forced-phase/no-flash policy identified; physical branch/stability guarantees not demonstrated. [D08,D09] |

### 7.2 Cross-cutting requirements sharpened by direct evidence

| Scope obligation | What Step 2 adds |
|---|---|
| X01 units and bases | The state pipeline, reactor, and column exchange different amount/energy conventions; conversion must be a declared boundary operation, not inferred from a field name. [D06,D14,D15] |
| X02 specification independence | Stream specification interpretation can depend on context; represent partial states and independent inputs explicitly. [D06] |
| X03/X04 empty, trace, and reacting material | Builder defaults, empty separator outlets, and reacting product amounts require distinct semantics. [D03,D12,D14] |
| X05/X07 phases and equilibrium scope | Phase slots, density-based routing, external ownership, and fallback phase restriction are different concepts. [D06,D08,D09,D12] |
| X08–X10 data, partial capability, validity | Estimation/substitution and optional reporting-property failures must carry meaning beyond a number or exception. [D10,D13] |
| X11/X12 energy and reaction accounting | Units combine material enthalpy with reaction heat and energy streams; provider substitution requires an explicit convention. [D11–D14] |
| X13/X14 representations and internal locations | Species ordering is an integration contract; columns and reactors require internal states not reducible to public stream edges. [D14,D15,D18] |
| X15 nested convergence | Recycle closure and column efficiency iteration each have their own completion conditions. [D16,D17] |
| X16/X17 invalidation and reproduction | Shared definitions, snapshots, model migrations, and dependency resolution are essential to interpreting saved results. [D04,D05] |
| X18 isolation | Shared material context and solver state require explicit session isolation and tests; static inspection alone is not a race diagnosis. [D08,D17] |
| X19 derivatives | Flags/interfaces alone do not establish derivative coordinates or validity. Dedicated provider research remains necessary. [D07,D08,D18] |
| X20 chemical exchanges; X21 standard reporting | Not sufficiently resolved in this audit; remain explicit requirements for the next provider investigations. |
| X22 effective properties | Mixture aggregation and unit-level solids routing demonstrate that effective properties and transport closures require named assumptions. [D08,D12] |
| X23 failures | Failed preparation, altered-model recovery, failed nested convergence, and partial document load require different outcomes. [D04,D09,D10,D16,D17] |
| X24 time and signed flow | Dynamic internal states were sampled, not a complete dynamic conformance result. [D13,D14] |

## 8. The resulting conceptual architecture, without implementation prescriptions

This is a refined responsibility hypothesis for the later blueprint, not a physical module layout, schema, or Rust interface.

| Conceptual area | Owns | Must not silently own |
|---|---|---|
| Material definition and representation | Identity, selected slate, species/lump meaning, characterization lineage, alternate-representation mappings | Current stream flow or a solver's temporary composition |
| Data resolution and parameter preparation | Data selection, estimation, fitting, substitution approval, provenance, validity and resolved revisions | Ordinary property evaluation's hidden mutation of the model |
| Configured thermodynamic package | Compatible physical methods, caloric conventions, phase policies, parameter selections and declared capabilities | Shared current state of every material using the package |
| Material specification and evaluated state | Location, quantities, composition, phase instances, authoritative inputs, evaluated outputs and result currency | Equipment geometry or plant-wide convergence strategy |
| Property evaluation | Properties and derivatives of an admissible supplied state with declared coordinates | Unrequested phase/species redistribution |
| Equilibrium and speciation operations | Defined equilibrium problem, candidate solution, stability/convergence evidence and used assumptions | Product-port assignment or undeclared chemistry exchange |
| Unit transformation and coupling | Balances, reaction/transfer/routing assumptions, internal locations, physical constraints and product publication | Hidden redefinition of reusable material identity |
| Numerical execution and initialization | Workspaces, guesses, algorithm choice, retries, cancellation and diagnostics | Unreported alternate physical assumptions or overwriting accepted results |
| Flowsheet coordination | Process connectivity, dependency scheduling, nested solve policy and accepted whole-model result | Thermodynamic definition by graphical layout or calculation-order accident |
| Model lifecycle and reproduction | Edits, revisions, invalidation, snapshots, imports, dependency manifests and historical results | Treating a partially reconstructed document as a complete executable model |
| Provider integration | Supported operation semantics, mapping, ordering, limitations, lifecycle and error translation | Assuming every upstream capability is actually exposed |

The crucial interaction is between the configured package, the material state, the operation request, and the unit's physical constraints. These should be independently understandable even if one provider implements several together.

### 8.1 A new first-class requirement: calculation authority

Every operation should declare which information it accepts as authoritative and which information it may change. Examples:

- A PH valve outlet request fixes pressure, composition, amount, and the specified caloric target; it solves temperature and permitted phase allocation.
- A property-only refresh on an externally supplied split preserves allocation but computes compatible requested properties.
- A separator takes an internal phase state and applies a product-routing/carryover rule.
- A conversion reactor changes species amounts under a declared reaction model before or during thermal/phase resolution.
- A column owns internal stage variables and asks for properties during its coupled solution.
- A package-boundary translator, still to be researched, must state its conservation and recalculation contract explicitly.

These examples consolidate the observed workflows [D06,D11–D16]. The explicit authority contract is our recommendation, not an existing single DWSIM abstraction.

### 8.2 Granularity of reuse must be evidence-led

DWSIM provides several possible boundaries: complete configured packages, typed equilibrium results, mutable property routines, flash algorithms, primitive models, and external chemistry adapters. [D07–D10,D18]

The safest conceptual integration boundary is not automatically the smallest. Reusing a complete configured package can preserve its internal caloric, phase, and parameter conventions. Reassembling primitives from different providers requires separate compatibility evidence. This audit therefore does not select DWSIM, ThermoPack, or another library as the numerical foundation and does not imply arbitrary mixing of their submodels.

## 9. Actions to carry into the functional specification

The following is a candidate action catalog derived from the traces. It is not an implementation API.

| Action | Required semantic distinction |
|---|---|
| Resolve material identity | Identity versus display name, alias, provider identifier, or pseudocomponent definition |
| Select or change a material slate | Complete replacement versus addition/removal, with explicit quantity and dependency policy |
| Characterize a material | Preserve raw characterization and generated definitions; detailed algorithms remain to be investigated |
| Resolve package data | Use existing values, estimate, fit, reject, or approve a substitution |
| Configure a coherent package | Physical method, reference convention, phase policy, parameter revision and capability declaration |
| Bind a package to a material region | Definition binding, not mutation of an implicit global current stream |
| Specify a material state | User constraints versus guesses versus derived values |
| Replace or patch material quantities | Amount conservation and normalization intent |
| Evaluate supplied phases | Property-only behavior with permitted mutations explicitly stated |
| Resolve equilibrium or speciation | Physical problem, supported specification, candidate phases/species, convergence and stability diagnostics |
| Initialize a unit or local state | Numerical estimates that do not redefine the physical problem |
| Execute a unit transformation | Unit-owned process constraints, internal evaluations and product results |
| Allocate material to product ports | Phase routing, carryover/entrainment and empty-product semantics |
| Retry a calculation | Same-problem numerical recovery versus alternate physical calculation |
| Accept or reject a candidate result | Acceptance checks and required convergence level |
| Change a model definition | Revision creation, impact assessment and invalidation |
| Restore a snapshot or saved document | Values plus identities, dependencies, compatibility and readiness |
| Request auxiliary reporting properties | Optional calculations that cannot silently weaken essential validation |
| Translate an external result | Species/phase mapping, authority and compatible caloric conventions |
| Reproduce a calculation | Exact definitions, data, assumptions, implementation versions and policy |

## 10. Runtime probes required before treating the design lessons as integration guarantees

No probe below has been executed. They are targeted verification tasks, not additional user scope commitments or claims of defects.

| Probe | Trigger and observation | Question resolved |
|---|---|---|
| P01 Context association | Bind streams of different slates sequentially to a package; record data/cache changes | Does binding affect resolved data or leave incompatible cached state? |
| P02 Save purity | Save a package with no current material and missing interaction data; compare parameter state before/after | Does the observed save/association path perform preparation in that fixture? |
| P03 Failure atomicity | Force a flash failure after an accepted stream state exists | Which values and statuses change, and what can safely remain published? |
| P04 Recovery semantics | Exercise each configured fail-safe on a multiphase request | Are used model, allowed phases and degradation discoverable without parsing free text? |
| P05 External split preservation | Install a supported external phase split then request property refresh | Is allocation preserved through the complete caller/solver path? |
| P06 Valve finalization | Count local PH evaluation and downstream stream evaluation | Are results equivalent, and where is a second solve required versus avoidable? |
| P07 Phase identity and routing | Reverse provider phase order or cross liquid densities | Does physical identity remain separate from product-port identity? |
| P08 Feed-edit equivalence | Compare replace/patch actions on new, loaded, and existing streams | Are input semantics deterministic and independent of creation history? |
| P09 Compound-change completeness | Remove a compound used by a reaction, forced-solid policy and saved result | Which dependencies are repaired, rejected, or left requiring user action? |
| P10 Parameter-change invalidation | Edit the same interaction value through supported UI/API routes | Do all affected results become stale and recalculate under the same data? |
| P11 Roundtrip reconstruction | Save/load a case with assays, custom parameters, reactions and multiple packages | Are definitions, bindings, input intent and numerical outcomes preserved? |
| P12 Missing provider | Load a case with an unavailable implementation | Can inspection proceed while calculation is reliably prevented or qualified? |
| P13 Interleaved/concurrent cases | Alternate independent slates and settings, then test supported parallel paths | Are contexts, options, errors and results isolated? |
| P14 Nested convergence | Force recycle or column-efficiency pass exhaustion | Are local results distinguishable from accepted unit/flowsheet solutions? |
| P15 Caloric consistency | Couple a reaction model and packages with stated reference conventions | Is reaction/formation energy accounted for once and consistently? |
| P16 External capability boundary | Match advertised provider operations to exercised adapter calls | Which upstream capabilities are actually usable through the chosen adapter? |
| P17 Optional-property failure | Fail an auxiliary bounding/report property while required state properties remain available | Does the outcome correctly distinguish optional omission from physical failure? |
| P18 Component-order invariance | Change compound ordering without changing physical input | Are all positional values remapped consistently? |

These probes belong in the later provider and conformance work. A source pattern can motivate a requirement without demonstrating that a particular production case currently fails.

## 11. Completion and handoff

### What Step 2 has established

This report supplies a pinned-source responsibility map; fifteen workflow cards; information-flow reconstructions; observed physical-policy and lifecycle decisions; retain/separate/adapt/exclude assessments; and explicit links to the scope baseline. Representative valve, separator, reactor, flowsheet/recycle, model-edit, and persistence pathways were traced at the source level. Exchanger, column, and external-provider boundaries were investigated with their coverage limits stated.

### What remains open

No numerical correctness, performance, thread safety, save/reload equivalence, or independent validation claim has been established. No current product-wide feature certification is made. Some specialized scenario families and some unit execution branches remain untraced. Publicly exposed source and the behavior of a particular distributed build can differ, especially where optional integrations, native dependencies, or product-edition gates apply.

### Consequence for the next research stage

The next provider comparisons should answer the concrete questions uncovered here: whether configuration preparation is separate from evaluation; what state and phase ownership the interface assumes; how caloric conventions and species ordering are represented; which operations and derivative meanings are actually exposed; whether failures can change physical assumptions; and what must be persisted to reproduce a calculation.

The architecture should preserve DWSIM's breadth of useful engineering actions while making those actions' data, assumptions, authority, and outcomes explicit. It should not copy its object graph literally, and it should not discard its existing modular boundaries merely because some paths are stateful.

## 12. Pinned source register

All source-code references below use the same commit. Listed regions identify inspected portions, not a claim that every line in the file was audited. Overlapping regions are consolidated for readability. Where a connector response truncated an extended region, only visible content supported the findings; method names are supplied to aid retrieval. Repository-relative links are pinned and directly reusable in later research.

### D00. Repository and commit identity

[Pinned commit metadata](https://api.github.com/repos/DanWBR/dwsim10/commits/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3). [Active repository metadata](https://api.github.com/repos/DanWBR/dwsim10). [Historical repository metadata](https://api.github.com/repos/DanWBR/dwsim). Repository metadata was retrieved during this investigation; metadata endpoints are not immutable, whereas all code links below identify the fixed commit.

### D01. Flowsheet construction and headless orchestration

**File:** [`engine/DWSIM.FluentAPI/Flowsheet.cs`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/Flowsheet.cs)

**Inspected regions:** [lines 1–260](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/Flowsheet.cs#L1-L260); [lines 500–810](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/Flowsheet.cs#L500-L810).

**Evidence used:** Create/Wrap/Load/Save, WithCompounds, WithPropertyPackage, Solve/TrySolve and solver dispatch.

### D02. Package configuration actions

**File:** [`engine/DWSIM.FluentAPI/PropertyPackageConfig.cs`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/PropertyPackageConfig.cs)

**Inspected regions:** [lines 1–270](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/PropertyPackageConfig.cs#L1-L270).

**Evidence used:** Flash configuration; model-specific interaction-parameter updates; generic configuration escape hatch.

### D03. Material input actions

**File:** [`engine/DWSIM.FluentAPI/Builders/MaterialStreamBuilder.cs`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/Builders/MaterialStreamBuilder.cs)

**Inspected regions:** [lines 1–260](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FluentAPI/Builders/MaterialStreamBuilder.cs#L1-L260).

**Evidence used:** Composition versus component-flow updates; new-stream default handling; scalar state/flow setters.

### D04. Flowsheet definitions, lifecycle and persistence

**File:** [`engine/DWSIM.FlowsheetBase/FlowsheetBase.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb)

**Inspected regions:** [lines 1–420](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L1-L420); [lines 750–960](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L750-L960); [lines 1400–1630](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L1400-L1630); [lines 2600–3060](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L2600-L3060); [lines 3120–3510](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L3120-L3510); [lines 4800–5020](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L4800-L5020); [lines 5500–5740](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L5500-L5740); [lines 5900–6140](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetBase/FlowsheetBase.vb#L5900-L6140).

**Evidence used:** Shared collections and compound association; object creation; ResetCalculationStatus; reaction constructors; SaveToXML/LoadFromXML; LoadProcessData; selected snapshot restoration and undo/redo paths. Portions of large responses were truncated; claims use visible code, not inferred unseen bodies.

### D05. Compound selection changes in the current desktop setup helper

**File:** [`ui/DWSIM.UI.Desktop.Avalonia/SimulationSetupShared.cs`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/ui/DWSIM.UI.Desktop.Avalonia/SimulationSetupShared.cs)

**Inspected regions:** [lines 1–105](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/ui/DWSIM.UI.Desktop.Avalonia/SimulationSetupShared.cs#L1-L105).

**Evidence used:** CompoundSelection.Add/Remove, phase-compound repair, quantity changes, normalization, clearing and forced-solid updates.

### D06. Material state calculation and assignment

**File:** [`engine/DWSIM.Thermodynamics/MaterialStream/MaterialStream.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/MaterialStream/MaterialStream.vb)

**Inspected regions:** [lines 1–240](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/MaterialStream/MaterialStream.vb#L1-L240); [lines 740–1530](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/MaterialStream/MaterialStream.vb#L740-L1530).

**Evidence used:** State and phase storage; equilibrium-ownership flags; Calculate paths; flow conversions; phase/aggregate properties; successful-tail bookkeeping; Assign/AssignProps.

### D07. Public property-package interface

**File:** [`engine/DWSIM.Interfaces/IPropertyPackage.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Interfaces/IPropertyPackage.vb)

**Inspected regions:** [lines 1–260](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Interfaces/IPropertyPackage.vb#L1-L260).

**Evidence used:** Configured package identity and context; equilibrium-result entry points; other package-facing functions.

### D08. Common property-package implementation

**File:** [`engine/DWSIM.Thermodynamics/PropertyPackages/Base/PropertyPackage.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/Base/PropertyPackage.vb)

**Inspected regions:** [lines 1–910](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/Base/PropertyPackage.vb#L1-L910); [lines 2180–2440](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/Base/PropertyPackage.vb#L2180-L2440); [lines 2750–2960](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/Base/PropertyPackage.vb#L2750-L2960); [lines 3100–3390](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/Base/PropertyPackage.vb#L3100-L3390).

**Evidence used:** Method options and defaults; current material context; phase maps; cloning; aggregate properties; typed equilibrium dispatch; mutable equilibrium preparation and selected PH paths.

### D09. Flash selection and fail-safe behavior

**File:** [`engine/DWSIM.Thermodynamics/FlashAlgorithms/UniversalFlash.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/FlashAlgorithms/UniversalFlash.vb)

**Inspected regions:** [lines 1–240](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/FlashAlgorithms/UniversalFlash.vb#L1-L240).

**Evidence used:** Flash_PT validation, heuristic phase/algorithm dispatch and configured recovery branches.

### D10. NRTL preparation and substitution behavior

**File:** [`engine/DWSIM.Thermodynamics/PropertyPackages/NRTL.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/NRTL.vb)

**Inspected regions:** [lines 1–350](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.Thermodynamics/PropertyPackages/NRTL.vb#L1-L350).

**Evidence used:** RunPostMaterialStreamSetRoutine, EstimateMissingInteractionParameters, group-model fitting and failed-estimation warning/substitution.

### D11. Valve state resolution and outlet publication

**File:** [`engine/DWSIM.UnitOperations/UnitOperations/Valve.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/Valve.vb)

**Inspected regions:** [lines 1300–1540](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/Valve.vb#L1300-L1540); [lines 1700–1920](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/Valve.vb#L1700-L1920).

**Evidence used:** Selected steady-state Calculate setup, PH result use, product specification publication and DeCalculate.

### D12. Separator internal state and product allocation

**File:** [`engine/DWSIM.UnitOperations/UnitOperations/Vessel.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/Vessel.vb)

**Inspected regions:** [lines 1100–1780](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/Vessel.vb#L1100-L1780).

**Evidence used:** Selected Calculate branches: MixedStream creation, feed aggregation, operating modes, phase/solids routing and outlet state writes.

### D13. Two-sided exchanger property and heat coupling

**File:** [`engine/DWSIM.UnitOperations/UnitOperations/HeatExchanger.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/HeatExchanger.vb)

**Inspected regions:** [lines 1050–1270](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/HeatExchanger.vb#L1050-L1270); [lines 1450–1865](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/HeatExchanger.vb#L1450-L1865).

**Evidence used:** Sampled per-cell dynamic states; maximum-exchange helper; selected steady-state setup, side-specific hypothetical states and thermal-efficiency PH calls.

### D14. Conversion reaction, thermal calculation and product handling

**File:** [`engine/DWSIM.UnitOperations/Reactors/Conversion.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/Reactors/Conversion.vb)

**Inspected regions:** [lines 350–590](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/Reactors/Conversion.vb#L350-L590); [lines 670–900](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/Reactors/Conversion.vb#L670-L900).

**Evidence used:** Reaction group execution, constrained conversions, composition update, reaction-heat/thermal modes and selected product-phase path.

### D15. Column internal states and solver preparation

**File:** [`engine/DWSIM.UnitOperations/UnitOperations/RigorousColumn.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/RigorousColumn.vb)

**Inspected regions:** [lines 4000–4220](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/RigorousColumn.vb#L4000-L4220); [lines 4400–4620](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/RigorousColumn.vb#L4400-L4620); [lines 5400–5480](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/RigorousColumn.vb#L5400-L5480).

**Evidence used:** Selected estimate generation, stage data collection, K-value/flash use and ColumnSolverInputData construction. An additional extended response showed part of the robust preparation routine; not all of that body was visible or audited.

### D16. Rate-based column efficiency iteration

**File:** [`engine/DWSIM.UnitOperations/UnitOperations/RigorousColumnRateBased.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/RigorousColumnRateBased.vb)

**Inspected regions:** [lines 1–220](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.UnitOperations/UnitOperations/RigorousColumnRateBased.vb#L1-L220).

**Evidence used:** Documented formulation plus RunRateBasedPasses and initial ComputeRateBasedEfficiencies implementation.

### D17. Flowsheet ordering, queue coordination and recycle iteration

**File:** [`engine/DWSIM.FlowsheetSolver/FlowsheetSolver.vb`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetSolver/FlowsheetSolver.vb)

**Inspected regions:** [lines 1–250](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetSolver/FlowsheetSolver.vb#L1-L250); [lines 1050–1300](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetSolver/FlowsheetSolver.vb#L1050-L1300); [lines 1440–1680](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/engine/DWSIM.FlowsheetSolver/FlowsheetSolver.vb#L1440-L1680).

**Evidence used:** Selected object dispatch and ordering, SolveFlowsheet readiness/shared settings, queue construction and outer recycle convergence.

### D18. External Reaktoro contract

**File:** [`docs/reaktoro-contract.md`](https://github.com/DanWBR/dwsim10/blob/a443463f2ace79cdaff75ecd0974c7b6b9d7c4d3/docs/reaktoro-contract.md)

**Evidence used:** Pinned project documentation describing four call sites, operation distinctions, species ordering, lifecycle/error handling and native-runtime versioning. Documentation evidence, not an independent execution test.

---

**End of THERMO-DWSIM-002 v0.1.** This document is an input to the cross-library responsibility comparison and the later language-independent functional blueprint. It does not select a numerical backend or prescribe Rust implementation structures.
