# Conceptual packaging of thermodynamics

**Document:** THERMO-PACKAGING-005  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 5, define the conceptual packaging of thermodynamics  
**Status:** Proposed language-independent responsibility architecture. No implementation, provider integration, numerical validation or independent R/E/V certification is claimed.  
**Predecessors:** THERMO-SCOPE-001, THERMO-DWSIM-002, THERMO-LIBRARIES-003 and THERMO-FUNCTIONAL-004, each version 0.1.

## 0. Decision and intended use

Retain one coherent **engineer-facing thermodynamic package**, while separating the responsibilities that make it meaningful, executable and reliable inside a flowsheet. A package is neither just an equation of state nor a live material stream. It is a resolved choice of compatible material descriptions, methods, data, conventions and bounded capabilities. Its actual numerical realization may remain a cohesive external engine.

The proposed architecture has **ten conceptual responsibility packages**. Every one of the 94 functional requirements has exactly one primary package owner and named supporting participants. Eighteen information products have a single semantic owner. Fourteen handoff walkthroughs cover the 34 scenario routes, including the four deferred extension scenarios. Eight assembled-package examples and a mapping to the prior library research test whether the boundaries accommodate different modeling styles.

These are not ten services, ten databases, ten threads or ten Rust crates. They are explicit boundaries of meaning, responsibility and permitted behavior. Several can share code and deployment; a provider can realize several internally. The architecture does not prescribe a numerical backend, solver, storage technology, API signatures or record layouts.

**The central decision is to separate four authorities:** the authority to define a physical problem; the authority to perform numerical work; the authority to assess what the result establishes; and the authority to publish that result as current for a model revision. They cooperate through explicit handoffs rather than through a mutable global material object.

### Evidence boundary

This stage synthesizes the supplied fixed baselines. B1 remains the scope authority; B4 remains the normative functional baseline. B2/B3 supply research evidence with their recorded limitations. All library mappings below mean “candidate role supported by the prior research,” not a new current-release or numerical capability claim. No fresh library/version research was needed to allocate the already specified responsibilities.

The original 65 host-wide, 25 conditional-capability and four P3-representability obligations remain unchanged. The 14 P1, 16 P2 and four P3 scenarios remain unchanged. In particular, a provider returning unsupported does not close an intended P1/P2 delivery gap, and a conceptual handoff does not by itself establish numerical execution or independent physical validation. [B4 §§0–1; FD-01,FD-02,FD-09,FD-10,FD-12]

### Reading routes

Read §§1–3 for the packaging model; §4 for each package’s charter; §§5–8 for ownership, dependencies and consequential boundary decisions; §9 for complete handoffs; §§10–11 for library and assembled-package mappings. The requirement and scenario allocations in §§12–13 are the implementation-independent review checklist. The JSON companion is an architecture-management register, not the future thermodynamic data schema.

## 1. From the original seven domains to ten responsibility packages

The original plan’s seven areas remain intact. The refinement is primarily in its broad final area, “flowsheet binding, execution and results,” which contains responsibilities with different authority and lifetimes. A second refinement separates state information from the operation that is allowed to determine or change it.

| Original planning area | Proposed allocation | Reason for refinement |
| --- | --- | --- |
| A. Material identity and characterization | P01 material meaning; P02 characterization evidence/recipes; P10 coordinated adoption | Generating a cut definition and establishing its material meaning are related but distinct responsibilities. |
| B. Property data and model parameters | P02, with compatibility assessed by P03 | Evidence selection and numerical fitting must not silently occur during ordinary property evaluation. |
| C. Methods and configured packages | P03 | Keep the coherent package, but separate its realization, local binding and workspace. |
| D. Material states and state specifications | P04 descriptions; P05 completeness and calculation authority | A stored or partial state does not by itself define which changes a solver is permitted to make. |
| E. Property, stability and equilibrium operations | P05 contracts, realized by P08 and assessed by P09 | Do not confuse operation meaning, numerical implementation and accepted postconditions. |
| F. Reaction and speciation coupling | P06 definitions; P03 convention reconciliation; P05 combined problems | Separate chemistry meaning without requiring an independent chemistry-then-flash solver sequence. |
| G. Flowsheet binding, execution and results | P07 coordination, P08 execution, P09 qualification, P10 revision/publication | Local convergence, process completion, acceptance evidence and currentness need separate authorities. |

The new package map is not a one-to-one renaming of the thirteen functional-requirement families. For example, local material quantities move into P04; the interpretation of state specifications moves into P05; effective location bindings move into P07; and obsolete-result protection moves into P10. The original functional accountability labels remain in the allocation register as source context. Their normative statements and acceptance tests are not changed.

## 2. Disambiguate what “package” means

Several very different objects are routinely described using the same word. The blueprint should preserve the distinctions below even when one library combines them internally.

| Concept | Meaning | Authority |
| --- | --- | --- |
| Method description | What a particular physical/property method means, its required data, compatible conventions and available observables. It is not a local state. | P03, using P02 parameter/evidence descriptions |
| Reusable package recipe | An engineer-facing assembly or named whole-provider choice with unresolved data or defaults still visible. | P03 |
| Resolved package revision | The consequential selected methods, material/data references, conventions and physical defaults under which evaluation is to occur. | P03; P10 commits the approved revision |
| Effective location binding | Which resolved package applies at a stream/stage/region and which allowed local restrictions or defaults specialize it. | P07; P10 commits binding changes |
| Physical calculation problem | Captured state, constraints, allowed changes and required outputs under that effective binding. | P05 |
| Provider realization | The concrete engine/build/binding and data realization of the package or selected contribution, with exposed restrictions. | P08 |
| Runtime session/workspace | A disposable or reusable numerical context with an explicit compatible configuration and lifetime. | P08 |
| Qualified result | Values plus evidence about the original request, actual methods, checks and achieved scope. | P09 |
| Current published binding | A qualified result adopted for a matching model/location revision and permitted run/scope. | P10 |

### Identity and sharing rules

A reusable package may serve many streams, stages, interfaces and runs. Sharing its approved definition must not share those locations’ mutable state. A region binding may provide inherited choices, but its effective resolution must be discoverable and independent of evaluation order.

A local frozen-reaction or phase-suppression choice changes the **effective physical problem** even when the base reusable package is unchanged. Conversely, selecting another numerical root algorithm need not change the base physical definition, although it changes execution evidence and can change initialization or branch behavior. Neither distinction licenses silent changes in what is actually solved. [B4:FR-CFG-01,FR-CFG-05,FR-CFG-06,FR-RUN-05]

Not every external engine exposes independently selectable internal data or method components. A whole-provider realization may identify an inseparable model/data bundle by a pinned, attributable artifact and its documented conventions. Unknown provenance or scientifically relevant choices must be recorded as limits, not reverse-engineered by guessing. Reproduction claims are bounded by the manifest actually available. This preserves whole-package reuse without pretending that opaque internals are fully characterized. [B4:FR-GOV-02,FR-GOV-04,FR-RES-07,FR-LIF-04]

A resolved definition is not the same as a validated definition. Explicitly authorized estimates, reduced fidelity and extrapolation may remain usable within a qualified engineering workflow. Readiness, acceptance and independent physical validation remain separate assessments.

## 3. Overall conceptual arrangement

The conceptual organization has three cooperating parts, not a mandatory processing pipeline.

**Engineering definitions:** P01 material meaning, P02 data/preparation, P06 chemistry meaning and P03 coherent package assembly.

**Local calculation semantics and realization:** P04 local material descriptions, P05 allowed operations/problems and P08 provider/numerical execution.

**Flowsheet reliability and coordination:** P07 process integration, P09 evidence/acceptance and P10 revisions/publication/reconstruction.

A normal numerical use case reads as follows:

```text
Material + data + optional chemistry definitions
    -> coherent resolved package
    -> explicit binding at a process location
    -> local description + original constraints + permitted changes
    -> initialization and eligible provider execution on working state
    -> candidate values and execution evidence
    -> required checks at the requested local/unit/flowsheet scope
    -> revision/run-guarded publication
```

This is an information/authority flow. An equation-oriented process model need not call a complete standalone flash at every local point. It can use qualified property or equation contributions while its own coupled solver coordinates the unknowns. Parameter fitting and recycle convergence introduce legitimate feedback loops under explicit trial/run contexts.

### What remains outside the thermodynamics core

The host process-model system owns equipment geometry and physical operating laws, authoritative process connectivity, global equation/graph construction, the overall numerical process solver and time integration. P07 exposes the thermodynamic boundary of those responsibilities; it is not a replacement implementation of all of them. Presentation and editing clients invoke the same domain actions and display the same status meanings but do not define thermodynamics by graphical placement.

P07/P09/P10 may therefore be shared facilities of the larger simulator. Their thermodynamic obligations are specified here so the numerical core is not forced to own them. They do not have to be duplicated inside every property package. [B4 §2; FR-FLW-01,FR-FLW-07,FR-LIF-08]

## 4. Package charters

### P01. Material semantics and representations

**Purpose:** Give every material description a defensible meaning before choosing a calculation method.

**Why these responsibilities belong together:** Identity, representation, coordinate ordering and conservation belong together because an amount has no stable physical meaning without them.

**Owns:**

- Constituent identity distinct from names, registry aliases, provider identifiers and numerical ordering.
- Meaning of molecular mixtures, apparent/true descriptions, assay-derived cuts, empirical constituents, and distributed or surface-associated material descriptions.
- Composition coordinates, permitted basis conversions, justified conserved quantities, and reusable representation-map definitions.
- Domain meaning for bulk phases, surface/site inventories, and distribution-valued material attributes; these meanings do not automatically confer numerical support.

**Consumes:**

- User-supplied or imported identity assertions and their source references.
- Characterization proposals expressed as candidate constituent descriptions, not an instruction to change the live material list.
- Requirements for a representation map, including conserved quantities and known information loss.

**Produces:**

- Versionable material/representation definitions and validity-of-meaning assessments.
- Explicit coordinate maps, basis relationships and conserved-quantity declarations.
- Reusable mapping definitions with stated inverse/reconstruction limitations.

**Actions:**

- Resolve identity and reject ambiguous aliases.
- Declare or extend a material representation.
- Validate a proposed constituent or domain description.
- Define and assess a representation mapping or distribution reduction.

**Must not own:**

- Live stream quantities, phase-search results, or unit product assignments.
- Invented molecular weight, elemental formula or chemical identity solely to satisfy a backend.
- A universal inverse for an aggregating map.
- A thermodynamic method or global phase-equilibrium policy.

**Boundary justification:** Local amounts change with a process calculation; identity and representation should not. Assay fitting creates candidate descriptions but must not become the identity authority.

**Lifecycle:** Draft/imported description -> semantically assessed proposal -> revision accepted through P10. No calculation or serialization silently upgrades an unresolved identity.

**Primary requirements (6):** FR-MAT-01, FR-MAT-02, FR-MAT-07, FR-MAT-08, FR-EXT-01, FR-EXT-02.

**Evidence/requirement basis:** B4:FR-MAT-01, B4:FR-MAT-02, B4:FR-MAT-07, B4:FR-MAT-08, B4:FR-EXT-01, B4:FR-EXT-02, B3:W08.

### P02. Property evidence, parameters and characterization

**Purpose:** Produce attributable data and characterized-material proposals without silently redefining the model during evaluation.

**Why these responsibilities belong together:** Characterization and parameter fitting are both evidence-to-definition preparation processes: they produce traceable candidates and may need thermodynamic evaluations.

**Owns:**

- Source observations, property correlations as documented evidence, parameter-set values and override precedence.
- Parameter interpretation records: specified zero, absent value, omitted interaction, rule-derived value, units, convention, validity and provenance.
- Estimation and regression problem definitions, candidate parameter trials, fit evidence and approval proposals.
- Original assays, cut-generation/characterization recipes and derivation lineage linking raw descriptions to proposed constituent/property records.

**Consumes:**

- P01 material identities and composition/mapping semantics.
- Method-specific parameter needs and interpretation rules supplied as explicit descriptors; no live configured-package query is required.
- Measured/imported evidence and numerical evaluation outcomes provided by the coordinating use case.

**Produces:**

- Resolved parameter snapshots, unresolved-needs reports and source/override lineage.
- Candidate fitted/estimated/substituted data with their method and limitations.
- Characterization proposals containing raw-assay linkage, proposed material descriptors for P01, and associated property evidence.

**Actions:**

- Resolve and compare data alternatives.
- Construct an estimation, fitting or characterization task.
- Interpret returned trial/fit evidence and prepare an approval proposal.
- Export evidence and recipe information without executing hidden thermodynamics.

**Must not own:**

- Approved live-model revision changes or provider-session state.
- Implicit estimation triggered by attaching a stream or saving a case.
- Authority to declare a numerical provider available.
- Final physical suitability of a composed package; P03 assesses compatibility and P09 retains validation evidence.

**Boundary justification:** P02 does not call a package that recursively resolves its own missing parameters. The coordinator supplies a fixed trial configuration to P08, returns evidence to P02, and commits the selected proposal through P10.

**Lifecycle:** Imported evidence remains attributable. Trials and estimated alternatives are separate from selected data. A fit may produce a local assessment without independent physical validation. Numerical fitting and characterization remain conditional capabilities.

**Primary requirements (7):** FR-DAT-01, FR-DAT-02, FR-DAT-03, FR-DAT-04, FR-DAT-05, FR-DAT-06, FR-DAT-07.

**Evidence/requirement basis:** B4:FR-DAT-01, B4:FR-DAT-02, B4:FR-DAT-03, B4:FR-DAT-04, B4:FR-DAT-05, B4:FR-DAT-06, B4:FR-DAT-07, B2:WF-03, B3:W01, B3:W08.

### P03. Thermodynamic methods and configured packages

**Purpose:** Define a coherent, reusable engineering property package independently of where it is used and which live session executes it.

**Why these responsibilities belong together:** Model choices, resolved data and references must be qualified as one coherent package. Splitting their ownership across independently authoritative objects would permit incompatible physics to appear valid.

**Owns:**

- Method descriptions and their property, standard-state, caloric, phase-eligibility and data obligations.
- The configured package recipe and its resolved, consequential method/data/reference choices.
- Compatibility treatment for phase methods, caloric contributions, transport/effective-property closures and optional chemical-system definitions.
- Physical defaults and allowed local specializations, plus the semantic contract for request-specific readiness; numerical policy and implementation binding have separate identity.
- The package-side energy convention, including compatibility between material formation contributions and the chemistry correction.

**Consumes:**

- P01 material representation and domain definitions.
- P02 resolved data, interpretation rules, source limitations and characterization outputs.
- P06 chemistry definitions and reference/property requirements where the package includes reactive species.
- Declarative operation/capability assessments supplied by the coordinator in a P03-owned qualification vocabulary; these are not calls to live execution or publication services.

**Produces:**

- Resolved package definition with referenced data and coherent method composition.
- Physical eligibility/default-policy description and operation-specific qualification conditions.
- Compatibility and caloric-completeness assessments, including unsupported and unassessed combinations.

**Actions:**

- Assemble a whole-provider or composed-method package.
- Compare two configurations by their consequential choices.
- Qualify caloric and convention compatibility.
- Assess a requested capability against supplied readiness evidence; unknown prerequisites remain unknown.

**Must not own:**

- A mutable current material stream, accepted process result or numerical work buffer.
- Reactor residence time, equipment geometry, outlet routing or global solve order.
- The assumption that every package exposes a potential, symbolic expression, arbitrary phase count or derivative.
- A promise that matching property names alone makes submodels interchangeable.

**Boundary justification:** A reusable package can be shared by many different locations and physical requests. Per-location constraints and run-specific algorithms must not leak into its approved definition.

**Lifecycle:** Template or draft recipe -> resolved revision with bounded qualification -> use through explicit bindings. A new consequential model/data choice produces a new effective definition. A numerical-policy or realization change is separately recorded and can require reassessment without pretending the base physics changed.

**Primary requirements (9):** FR-GOV-04, FR-CFG-01, FR-CFG-02, FR-CFG-03, FR-CFG-04, FR-CFG-05, FR-CFG-07, FR-PRP-07, FR-CHM-05.

**Evidence/requirement basis:** B4:FR-CFG-01, B4:FR-CFG-02, B4:FR-CFG-03, B4:FR-CFG-04, B4:FR-CFG-05, B4:FR-CFG-07, B4:FR-PRP-07, B4:FR-CHM-05, B3:W01, B3:CR-01, B3:CR-06.

### P04. Material states, quantities and local descriptions

**Purpose:** Represent material at a location without confusing it with its provider, solver workspace, flow connection or acceptance status.

**Why these responsibilities belong together:** Local values, their basis and provenance roles must travel together so a guessed temperature cannot become a specified temperature or a reported aggregate an additional physical phase.

**Owns:**

- Local composition/amount descriptions, intensive coordinates, flows, inventories and their distinct bases.
- Value roles: specified constraint, observation, numerical guess, supplied allocation and calculated value.
- Physical phase-instance descriptions, aggregate views, zero-amount incipient information and ambiguous cross-result correspondence.
- Meaningful local domain descriptions for separate bulk/interface states and inventory contexts.
- Explicit composition replace/patch/rescale transformations and reporting-basis descriptions; these transformations return proposals rather than directly committing live model edits.

**Consumes:**

- P01 material, coordinate, basis and mapping definitions.
- Location/configuration references and role-tagged values supplied by a caller.
- Normalized candidate values or externally supplied states; a provider buffer is not the authoritative representation.

**Produces:**

- State/specification descriptions and candidate-state values with clear amounts and roles.
- Proposed quantity edits and derived reporting views without hidden material changes.
- Inventory or zero-flow descriptions that retain what is known and mark what is undefined.

**Actions:**

- Create a local description independent of graphical streams.
- Replace a full composition or patch/rescale selected quantities under a named preservation rule.
- Describe physical and incipient phases and assess correspondence without inventing continuity.
- Provide explicit reporting-basis conversions when the required relationships/data are available.

**Must not own:**

- Selection of an equilibrium algorithm or authority to alter species/phase allocation.
- A library-wide current-state pointer.
- Equipment-owned port identity as physical phase identity.
- Current-result adoption or global convergence; those are P09/P10 and P07 responsibilities.

**Boundary justification:** A material description can be intentionally partial, supplied by an external unit, or numerically unaccepted. P05 decides which operation is admissible; P09/P10 determine acceptance and currentness.

**Lifecycle:** Each description is associated with explicit references and roles. Provider working mutations remain in P08. A proposed state or input edit is not made current until the appropriate P10 publication/change action succeeds.

**Primary requirements (11):** FR-MAT-03, FR-MAT-04, FR-MAT-05, FR-MAT-06, FR-STA-01, FR-STA-04, FR-STA-05, FR-STA-06, FR-STA-07, FR-STA-08, FR-EXT-03.

**Evidence/requirement basis:** B4:FR-MAT-03, B4:FR-MAT-04, B4:FR-MAT-05, B4:FR-MAT-06, B4:FR-STA-01, B4:FR-STA-04, B4:FR-STA-05, B4:FR-STA-06, B4:FR-STA-07, B4:FR-STA-08, B4:FR-EXT-03, B2:WF-04, B3:W02.

### P05. Thermodynamic problems and operation contracts

**Purpose:** Define what each calculation means, what it may determine, and what it must establish, without choosing a particular numerical library or forcing separate phase and chemistry solvers.

**Why these responsibilities belong together:** Properties, state constraints, equilibrium relations and sensitivities share the same authority and postcondition language. Keeping them within one problem boundary allows a provider to solve coupled phase/chemistry constraints coherently.

**Owns:**

- Operation semantics for phase properties, phase-pair properties, effective closures, TP/PH/PS and saturation resolution, stability and branch questions, chemical equilibrium, coupled reactive multiphase problems and derivative requests.
- The problem-specific unknowns, authoritative quantities, permitted root/density resolution, phase/species changes, physical restrictions and required outputs.
- Completeness/consistency assessment appropriate to a standalone solve versus an intentionally partial coupled formulation.
- The constitutive/equilibrium contribution contract for values, residuals, equations and derivative/sensitivity information, with actual availability distinguished.
- Definitions of problem-specific postconditions and check demands, including the original inverse target and actual stability/phase-search scope.

**Consumes:**

- P03 resolved package and eligible physical scope.
- P04 state descriptions, quantities and input roles.
- P06 reaction definitions, participation restrictions and exchange constraints where relevant.
- P07-supplied operating constraints and required result scope, expressed through operation contracts rather than unit-object dependencies.

**Produces:**

- An explicit calculation problem and allowed-change description.
- A qualification failure or intentionally partial formulation with unresolved prerequisites.
- An operation/contribution contract, mandatory/optional property demands, and postcondition definitions for P08 and P09.
- Problem-normalized candidate semantics, not automatic publication.

**Actions:**

- Prepare property-only, state-resolution, equilibrium, chemistry or derivative requests.
- Check conflicts between supplied authority and requested transformations.
- Specify a combined reacting multiphase problem or a declared splitting approximation.
- Describe equation/residual/callback participation without demanding every provider supply the same representation.

**Must not own:**

- Equipment balance choices, reactor geometry, product routing, graphical connectivity or flowsheet iteration strategy.
- A second independent chemistry solver that repeatedly invalidates an independently authoritative phase solver.
- Provider handles, algorithm internals or approval of altered physics as ordinary numerical success.
- Approval of current process state.

**Boundary justification:** The operation contract is not a promise to implement a universal host flash. Numerical execution may remain inside a whole package, or the host mathematical system may assemble approved contributions.

**Lifecycle:** Original physical problem is retained through initialization and retries. Local policy overlays form part of the effective physical problem; they need not mutate the reusable base package. A physically altered recovery creates a distinguishable alternative problem.

**Primary requirements (18):** FR-STA-02, FR-STA-03, FR-EQL-01, FR-EQL-02, FR-EQL-03, FR-EQL-04, FR-EQL-05, FR-EQL-06, FR-EQL-07, FR-EQL-08, FR-PRP-01, FR-PRP-02, FR-PRP-03, FR-PRP-04, FR-PRP-05, FR-PRP-06, FR-CHM-03, FR-EXT-04.

**Evidence/requirement basis:** B4:FR-STA-02, B4:FR-STA-03, B4:FR-EQL-01, B4:FR-EQL-02, B4:FR-EQL-03, B4:FR-EQL-08, B4:FR-PRP-01, B4:FR-PRP-05, B4:FR-CHM-03, B4:FR-EXT-04, B3:W02, B3:W03, B3:W05.

### P06. Chemistry definitions and participation

**Purpose:** Define chemical transformation semantics and their property/energy needs independently of a reactor geometry and independently of where the numerical solve occurs.

**Why these responsibilities belong together:** Reaction definitions, participation and conserved-basis semantics must remain consistent even when the same chemistry is used in different unit formulations.

**Owns:**

- Reaction identity, stoichiometry, extent/conversion meaning, rate/equilibrium-law conventions and applicable units.
- Kinetic, equilibrated and frozen participation rules, with admissible species and justified conserved-quantity requirements.
- Chemical-system mapping semantics, including apparent versus true species and non-duplicating material accounts.
- Reservoir/exchange requirements and their signs/bases; the process model decides whether such an exchange is permitted.
- Chemical reference/formation requirements that P03 must reconcile with its chosen caloric convention.

**Consumes:**

- P01 species/representation/mapping and conservation meanings.
- P02 thermochemical data, reaction parameters, source conventions and limitations.
- Explicit unit-supplied extent or participation choices, expressed as chemistry policy rather than equipment objects.

**Produces:**

- Reaction or chemical-system definitions and participation declarations.
- Strict conversion feasibility assessments and transform proposals on a stated quantity basis.
- Property/rate requirements and reference/exchange conditions for P03 and P05.
- Reporting/species reconciliation information without creating a second inventory.

**Actions:**

- Define and validate reactions, rates and conserved-basis assumptions.
- Assess strict extent feasibility, without silently optimizing it.
- Declare frozen/kinetic/equilibrated subsets and open exchanges.
- Define species/reporting maps and reaction-energy requirements.

**Must not own:**

- The complete process energy balance, hydrodynamics, interfacial area, residence-time model or time integrator.
- A mandatory separate chemistry service that must run before/after every flash.
- Permission to add titrant or reservoir material not authorized by the process request.
- Power to replace a finite-rate model with equilibrium because the available provider cannot evaluate rates.

**Boundary justification:** Numerical phase and chemical equilibrium may be solved together in P08 under one P05 problem. P06 supplies the chemistry meaning; P03 reconciles the material energy convention; P09 checks the combined outcome.

**Lifecycle:** Chemical definitions and policy revisions are distinct from numerical iterates and local equipment settings. Changing residence time need not change a reaction definition or package; changing participation changes the effective physical problem.

**Primary requirements (5):** FR-CHM-01, FR-CHM-02, FR-CHM-04, FR-CHM-06, FR-CHM-07.

**Evidence/requirement basis:** B4:FR-CHM-01, B4:FR-CHM-02, B4:FR-CHM-04, B4:FR-CHM-06, B4:FR-CHM-07, B2:WF-09, B3:W05.

### P07. Flowsheet integration and use-case coordination

**Purpose:** Connect material/property semantics to engineering actions without turning a property package into the process simulator.

**Why these responsibilities belong together:** The orchestration of engineering use cases needs process context and explicit handoffs. It should be above reusable thermodynamic definitions and providers.

**Owns:**

- Thermodynamic region/location bindings and their explicit inheritance/override rules.
- Unit-mode property demands and process-imposed constraints, including local stages, segments, bulk/interface locations and inventories.
- Mixing, mechanical splitting, separation/routing, heat-only coupling and material/representation-boundary action semantics.
- Use-case sequencing for authoring, preparation, unit calculations, coupled mathematical contributions, parameter-fit campaigns and restoration.
- Hierarchy of process completion scopes and the required relation between local, unit and flowsheet convergence.

**Consumes:**

- P01–P06 definitions and semantic contracts.
- Unit/flowsheet mathematical models and process topology from the host process-model system, not graphical objects.
- P08 capability/execution results, P09 assessment results and P10 revision/publication outcomes.

**Produces:**

- Explicit bound operation requests, provider-preparation requests and equation-contribution demands.
- Unit-local and whole-flowsheet candidate collections, process residuals and publication proposals.
- Definition/change proposals and prerequisite evidence supplied to their semantic owners.
- A user-visible account of unresolved work at the appropriate process scope.

**Actions:**

- Resolve effective bindings for a unit’s locations.
- Coordinate configuration preparation, local calculations and validation demands.
- Apply process routing/mapping constraints and request required state/property resolution.
- Drive fit/characterization use cases without embedding an optimizer inside the data catalog.
- Submit accepted-scope publication and change proposals to P10.

**Must not own:**

- Thermodynamic equations, parameter meaning, physical identity or provider-local caches.
- A new universal unit solver or mandatory sequential-modular execution model.
- Independent mutable copies of every package’s authoritative data.
- A shortcut around P09 checks or P10 revision/cancellation guards.

**Boundary justification:** P07 is an application-facing boundary, not a monolithic database or numerical engine. The surrounding process system still owns geometry, equipment laws, topology compilation, global mathematical solution and time integration.

**Lifecycle:** Every coordinated attempt captures its inputs and required completion scope. A local successful request can feed an ongoing unit/recycle iteration without prematurely becoming the published whole-flowsheet solution.

**Primary requirements (10):** FR-CFG-06, FR-FLW-01, FR-FLW-02, FR-FLW-03, FR-FLW-04, FR-FLW-05, FR-FLW-06, FR-FLW-07, FR-FLW-08, FR-RUN-08.

**Evidence/requirement basis:** B4:FR-CFG-06, B4:FR-FLW-01, B4:FR-FLW-02, B4:FR-FLW-03, B4:FR-FLW-04, B4:FR-FLW-05, B4:FR-FLW-06, B4:FR-FLW-07, B4:FR-FLW-08, B4:FR-RUN-08, B2:WF-06, B2:WF-07, B2:WF-10, B2:WF-11.

### P08. Provider realization and numerical execution

**Purpose:** Realize declared operations through native methods or external engines while containing implementation-specific ordering, state and failures.

**Why these responsibilities belong together:** The usable adapter surface, initializer, session model and error behavior form one operational boundary. Assessing any one in isolation can give a false integration guarantee.

**Owns:**

- Provider/build/binding/dependency identity and the capability actually exposed by each adapter.
- Implementation realization of package recipes, including provider-owned opaque but attributable method/data bundles where necessary.
- Numerical algorithm and initializer realization, workspaces, session activation, cache lifetime and isolation policy.
- Actual execution traces, used algorithm/phase search, returned values, failure translation and session recovery decisions.
- Cancellation/budget response and prevention of further useful reuse when a session’s state is uncertain.

**Consumes:**

- P03 configuration and declarative eligibility contracts.
- P04/P05 explicit states/problems, allowed changes and required contribution/outputs.
- P06 chemical semantics through the resolved problem, not an independent second request that can silently override it.
- Captured revision/run tokens and numerical policies supplied by P07/P10.

**Produces:**

- Adapter-specific capability/availability attestations in consumer-owned declaration contracts.
- Initialization assessments, candidate values or mathematical contributions with achieved scope.
- Requested-versus-used execution evidence and precise unsupported, failed, cancelled or quarantined outcomes.
- Reusable, reconstructed or retired session status.

**Actions:**

- Discover an exposed capability without claiming all upstream functionality.
- Construct or reuse a compatible session and isolate the full activation/evaluation sequence.
- Execute value, flash, chemistry or equation-contribution operations.
- Apply same-problem retries, or propose a separately authorized alternative when physics would change.
- Normalize native results and errors, retire uncertain sessions, and withhold interrupted work from publication.

**Must not own:**

- User material identity, semantic package approval, equipment operating decisions or independent current-result writes.
- Automatic validation claims based on its own numerical success flag.
- Silent model/data/phase substitution or fabricated unavailable quantities.
- Requirement that a whole provider reveal every internal algorithm as a separate service.

**Boundary justification:** P08 supplies candidates and evidence. Semantic meaning remains P01–P06; P09 assesses admissibility and P10 guards publication. Stateful engines are allowed, but their side effects cannot redefine host inputs.

**Lifecycle:** Session creation -> compatible use -> reuse assessment -> reset/reconstruction/retirement. A stale revision, unsafe failure, changed reference convention or unsupported concurrent context cannot be repaired by relabeling the same handle.

**Primary requirements (9):** FR-GOV-02, FR-GOV-03, FR-RUN-01, FR-RUN-02, FR-RUN-03, FR-RUN-04, FR-RUN-05, FR-RUN-06, FR-RUN-07.

**Evidence/requirement basis:** B4:FR-GOV-02, B4:FR-GOV-03, B4:FR-RUN-01, B4:FR-RUN-02, B4:FR-RUN-03, B4:FR-RUN-04, B4:FR-RUN-05, B4:FR-RUN-06, B4:FR-RUN-07, B3:W07, B3:CR-12.

### P09. Result qualification and coverage evidence

**Purpose:** Determine what a result establishes, independently of whether a solver returned success and independently of which revision is currently displayed.

**Why these responsibilities belong together:** Acceptance, lineage and coverage are linked evidence questions: they identify the claim actually justified, rather than merely retain numbers.

**Owns:**

- Qualified outcome meaning, per-property completion, postcondition evidence and acceptance-scope decisions.
- Conservation/thermal/authority check assessment under the justified bases and predeclared tolerances.
- Scientific-admissibility and numerical-conformance assessments, with independent empirical validation retained separately.
- Result lineage and requested-versus-used assumptions, including local versus larger-scope completion evidence.
- Scenario/configuration coverage claims and explicit open scope gaps.

**Consumes:**

- Original P05 problem and P03 convention/compatibility information.
- P04 normalized candidate values, captured inputs, and producer-supplied execution evidence.
- P06 conserved/chemical requirements and P07 supplied unit/flowsheet residual/closure evidence.
- Currentness/cancellation/publication facts provided by P10 for reporting; these do not change the historic scientific assessment.

**Produces:**

- Accepted-for-stated-scope, rejected or not-assessed candidate assessments with per-check evidence.
- Additional evaluation demands when mandatory evidence is missing; P07 can obtain it through P08 under the original conventions.
- Attributable result/validation records and bounded R/E/V claims.
- Publication-eligibility proposal, not an unconditional current-result write.

**Actions:**

- Classify candidate outcomes and assess mandatory/optional postconditions.
- Evaluate material, energy, chemical and authority residual evidence.
- Distinguish local acceptance from unit or flowsheet acceptance.
- Record validation and retain explicit scenario coverage gaps.
- Present a combined view of independent quality dimensions without overwriting their separate authorities.

**Must not own:**

- Thermodynamic model creation, undocumented numerical tolerances, or changing the request to pass a check.
- Invented independence of a re-evaluation that uses the same model/data.
- Provider-session recovery or current model revision.
- Authority to publish a stale or cancelled completion as current merely because its numeric checks passed.

**Boundary justification:** P09 can decide that a revision-A local result meets its requested criteria; it cannot decide that A is still the current model. P10 alone governs current revision/result bindings. A process-scope verdict also needs P07’s process criteria and residual evidence.

**Lifecycle:** Candidate -> required evidence assessment -> qualified outcome -> eligible publication proposal. History preserves original assessments. New data or independent validation adds evidence without retroactively changing what configuration produced the original values.

**Primary requirements (10):** FR-GOV-01, FR-GOV-05, FR-CHM-08, FR-RES-01, FR-RES-02, FR-RES-03, FR-RES-04, FR-RES-06, FR-RES-07, FR-RES-08.

**Evidence/requirement basis:** B4:FR-GOV-01, B4:FR-GOV-05, B4:FR-CHM-08, B4:FR-RES-01, B4:FR-RES-02, B4:FR-RES-03, B4:FR-RES-04, B4:FR-RES-06, B4:FR-RES-07, B4:FR-RES-08, B3:CR-13.

### P10. Model revisions, publication and reconstruction

**Purpose:** Keep approved definitions and current-result bindings coherent across edits, failed work, persistence and restoration.

**Why these responsibilities belong together:** Currentness, coordinated edits and restoration require one revision authority. Separating the revision check from the actual current-result update creates a race between acceptance and publication.

**Owns:**

- Current model revision and dependency/change-impact records.
- Atomic logical acceptance of coordinated changes to material, data, package, chemistry, state specifications and bindings.
- The authoritative current-result binding and revision/run/cancellation guard at publication.
- Reconstructable semantic archives, dependency manifests, migration decisions, snapshots and history relationships.
- Reproduction-versus-changed-model comparison status and interface-independent change semantics.

**Consumes:**

- Candidate semantic changes validated by their domain owners, plus captured old and proposed references.
- P09 acceptance/publication eligibility and P07’s requested location/scope.
- Explicit run cancellation/supersession facts and current model context.
- Exported domain snapshots and provider/dependency manifests, not live numerical handles.

**Produces:**

- Committed revisions or rejected/unresolved change proposals.
- Stale/needs-reassessment facts for affected results and sessions.
- Guarded accepted-current-result references, or stale/cancelled/obsolete publication rejection with historical retention.
- Inspectable reconstructed documents and separate dependency/readiness/restored-result status.

**Actions:**

- Assess and commit consequential changes conservatively when exact impact is unknown.
- Validate a result’s captured context at the same logical commit point that updates its current binding.
- Restore snapshots and semantic archives before considering numerical values current.
- Record migrations and reproducibility limits without substituting different physics.
- Apply common change semantics across interactive and automated callers.

**Must not own:**

- A second authority for thermodynamic or reaction laws, fit quality or numerical convergence.
- Provider memory ownership or caches masquerading as persistent semantic definitions.
- Inference that a document successfully loaded must be executable.
- Restoring an old numeric snapshot into different model meaning without explicit compatibility assessment.

**Boundary justification:** Logical atomicity does not prescribe a database, transaction engine or distributed architecture. P10 commits references and validated proposals; the semantic owners remain responsible for each content domain.

**Lifecycle:** Preparation and numerical work do not directly mutate approved state. An A-linked result can remain a valid historical result after revision B, but is not auto-published as B. Undo restores relationships and reassesses currentness, not merely values.

**Primary requirements (9):** FR-RES-05, FR-LIF-01, FR-LIF-02, FR-LIF-03, FR-LIF-04, FR-LIF-05, FR-LIF-06, FR-LIF-07, FR-LIF-08.

**Evidence/requirement basis:** B4:FR-RES-05, B4:FR-LIF-01, B4:FR-LIF-02, B4:FR-LIF-03, B4:FR-LIF-04, B4:FR-LIF-05, B4:FR-LIF-06, B4:FR-LIF-07, B4:FR-LIF-08, B2:WF-13, B2:WF-14, B3:W07.

## 5. Authoritative information products

These are conceptual products exchanged between responsibilities, not proposed tables, class fields or API return types. Each product has one owner of its meaning and invariants. P10 commits approved model revisions/current bindings; it does not become the author of every domain rule. Copies, snapshots and provider representations are permissible when their role and lineage remain clear.

A proposal is not a second authoritative version of the live model. Likewise, P08’s normalized candidate can use P04’s material language without taking ownership of that language, and P09 can retain the candidate in an accepted evidence record without taking ownership of P10’s current model revision.

| ID | Information product | Semantic owner | Meaning | Main consumers |
| --- | --- | --- | --- | --- |
| A01 | Material and representation definition | P01 | Meaning of identified constituents, lumps, chemical/reporting coordinates, nonbulk domains and justified bases. | P02, P03, P04, P05, P06, P07, P08, P09 |
| A02 | Representation map and reduction definition | P01 | A declared forward mapping with conserved quantities and information-loss/inverse limits. | P04, P06, P07, P08, P09 |
| A03 | Property evidence and resolved parameter snapshot | P02 | Actual selected observations/coefficients or attributable provider data bundle, precedence and limitations. | P03, P06, P08, P09, P10 |
| A04 | Characterization/fit proposal | P02 | Original assay or fitting evidence, recipe, trials and proposed changes to A01/A03. | P01, P03, P07, P09, P10 |
| A05 | Chemical definition and participation description | P06 | Reactions, rate/equilibrium conventions, admitted/frozen/kinetic subsets, reservoirs and conserved basis. | P03, P05, P07, P08, P09, P10 |
| A06 | Configured thermodynamic package revision | P03 | Resolved coherent methods, referenced material/data/chemistry, physical defaults, compatibility and caloric conventions. | P04, P05, P07, P08, P09, P10 |
| A07 | Region/location binding | P07 | Effective assignment and local policy overlay, distinct from a runtime current-stream pointer. | P04, P05, P08, P09, P10 |
| A08 | Local material description | P04 | Amounts, intensive coordinates, roles, actual/incipient phases, configuration references and possible unresolved quantities. | P05, P07, P08, P09, P10 |
| A09 | Calculation problem and authority contract | P05 | Original constraints, unknowns, permitted changes, required/optional outputs and declared physical scope. | P07, P08, P09, P10 |
| A10 | Readiness assessment | P03 | Request-scoped aggregation criteria with referenced constituent assessments, not a package-wide ready flag. | P07, P08, P09, P10 |
| A11 | Provider realization/capability attestation | P08 | Concrete build/binding/data identity, exposed operation subset, initialization and dependency limits. | P03, P05, P07, P09, P10 |
| A12 | Execution plan and numerical workspace | P08 | Algorithms, initializer guesses and temporary iterates of one captured request; not approved material state. | P07 |
| A13 | Provider candidate and execution evidence | P08 | Normalized P04 values plus actually used methods, residuals, errors, search and convergence facts. | P07, P09 |
| A14 | Qualification and acceptance evidence | P09 | Assessment of a candidate against the original request and a declared local/unit/flowsheet scope. | P07, P10 |
| A15 | Current-result publication binding | P10 | Association of an accepted result with a matching current location/model revision and permitted run/scope. | P04, P07, P09 |
| A16 | Change and reconstruction record | P10 | Committed revision, dependency impacts, archive manifests, migrations and restored-readiness status. | P01, P02, P03, P04, P06, P07, P08, P09 |
| A17 | Coverage and independent validation record | P09 | Bounded R/E/V and conformance claims with absent evidence explicit. | P03, P07, P08, P10 |
| A18 | Unit demands, coupling and routing description | P07 | Process-owned balances, phase-to-product roles, heat/material boundaries and required completion hierarchy. | P05, P08, P09, P10 |

### Boundary decisions where dual ownership would otherwise arise

**Characterization:** P02 owns the assay, method, fit and derivation record. P01 owns the meaning of generated constituent identities. P03 owns whether those generated properties complete a requested thermodynamic package. P10 accepts the coordinated proposal as one coherent change. No generated pseudocomponent silently appears in a live stream halfway through a failed characterization.

**Material state and result:** P04 owns quantity and phase meaning; P08 owns its working memory and raw candidate production; P09 owns the acceptance assessment and evidence; P10 owns the association that says this result is current for this model/location. This is four different facts, not four competing state stores.

**Caloric/chemical conventions:** P02 retains source data and their stated convention. P06 expresses chemical formation/reference requirements. P03 reconciles the selected material caloric convention and any required chemical energy correction. P07 constructs the actual equipment energy balance, including genuine heat/work/material exchange. P09 checks that balance. No component adds a generic heat of reaction independently of that agreement.

**Capability:** P08 attests to the exposed adapter/build/binding and its restrictions. P03 interprets compatibility and readiness under a resolved definition. P09 owns evidence-backed scenario claims. P07 owns unit demands and gathers the necessary assessments. Availability, scientific suitability, initializability, solved postconditions and validated coverage are not one flag.

**Reporting:** P04 owns the quantity/basis description and pure coordinate conversions. P05 defines any new property or reference-condition evaluation required to obtain the reporting value; P08 performs it. A standard-volume report is not always a simple unit conversion. It must not invent reference conditions or a density model.

## 6. Permitted dependencies and explicit feedback

### 6.1 The declared contract-dependency graph

The table gives the **logical dependency of a package’s public semantic contracts on lower-level contracts**. It is not a runtime call graph, a graph of every identifier reference, or a Rust dependency graph. A package may receive a producer’s evidence as values conforming to a consumer-owned declaration without importing that producer’s implementation.

For example, P03 defines what a capability/readiness attestation must say. P08 produces such an attestation and P07 supplies it to P03. P03 does not depend on a live P08 session merely to define package meaning. Similarly, P09 can receive a currentness fact in its reporting vocabulary from P10 without controlling the current revision itself. These exchanges do not require reverse implementation dependencies.

| Consumer | Permitted contract dependencies | Restriction |
| --- | --- | --- |
| P01, Material semantics and representations | Shared vocabulary only | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P02, Property evidence, parameters and characterization | P01 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P03, Thermodynamic methods and configured packages | P01, P02, P06 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P04, Material states, quantities and local descriptions | P01 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P05, Thermodynamic problems and operation contracts | P01, P02, P03, P04, P06 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P06, Chemistry definitions and participation | P01, P02 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P07, Flowsheet integration and use-case coordination | P01, P02, P03, P04, P05, P06, P08, P09, P10 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P08, Provider realization and numerical execution | P03, P04, P05, P06 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P09, Result qualification and coverage evidence | P01, P03, P04, P05, P06 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |
| P10, Model revisions, publication and reconstruction | P01, P02, P03, P04, P05, P06, P09 | No live upstream mutation; use explicit snapshots, attestations, proposals or operation contracts. |

Verified topological order of the declared graph: **P01 -> P02 -> P04 -> P06 -> P03 -> P05 -> P08 -> P09 -> P10 -> P07**.

The graph was checked for invalid references, self-dependencies and cycles. That structural result establishes only the consistency of the graph written here. It does not establish that an implementation already has those dependencies or that every semantic question has been settled.

The small shared vocabulary contains only cross-boundary references, quantity/basis identities, revision/run/scope identities, provenance references and agreed outcome/assertion terminology. It must not become an untyped universal property bag, hidden service locator or second material model. Detailed content contracts belong to their named semantic owners in Step 6.

### 6.2 Five apparent cycles and their treatment

| Apparent cycle | Permitted treatment | What is prohibited |
| --- | --- | --- |
| Parameter fitting needs thermodynamics, whose model needs parameters | P07 freezes each trial parameter/configuration proposal, sends evaluation to P08 under a P05 contract, returns trial evidence to P02, and later commits approved values through P10. Trial data can be numerically complete without being approved production data. | P02 recursively triggering evaluation that silently re-estimates the same unresolved parameters. |
| Chemical equilibrium changes phase allocation and phase equilibrium changes chemistry | Describe one P05 problem containing P06 chemistry and compatible P03 methods; P08 can solve it together or use an explicitly declared approximation; P09 checks the combined result. | Two independently authoritative solvers overwriting each other and claiming combined equilibrium from two standalone successes. |
| Verification needs additional property calculations | P09 emits an explicit missing-evidence demand or uses a qualified evaluator contract; P07 obtains P08 evidence under the original P05 context and returns it to P09. | P09 mutating the requested target, accepted state or physical model to make verification pass. |
| A flowsheet recycle needs results from a later unit | P07 and the host solver manage a captured run/iteration view. Local accepted candidates can feed later trials inside that view without publishing a whole-flowsheet result. P09 checks the appropriate completion level. | Treating the numerical tear as a mandatory physical transformation or calling every locally completed trial a converged flowsheet. |
| A result passes checks while the model changes | P09 qualifies the captured result. P10 performs a matching revision/run/scope check at the same logical commit point as the current-result update. | A time-separated revision check followed by an unconditional current-result write. |

Numerical iteration, reverse information delivery and dependency references are therefore allowed. Unexplained upward ownership mutation is not. The system need not be globally acyclic: process networks and nonlinear equations often contain loops. What must remain acyclic here is the declared **contract-construction dependency**, with orchestration and feedback made explicit.

### 6.3 Forbidden shortcuts

An evaluator must not fetch its current material from whichever stream was associated last. A fitting call must not rewrite approved data. A chemistry call must not silently add a reservoir. An adapter must not publish directly to the live flowsheet. A property call must not route separator products. A graphical edit must not alter physical values without a domain change. An acceptance check must not invent absent data or declare a local numerical success to be a current plant result. [B4:FR-CFG-06,FR-DAT-06,FR-CHM-07,FR-RES-05,FR-FLW-03,FR-LIF-03]

## 7. Consequential boundary resolutions

### 7.1 The configured package is a coherent bundle, not a monolith and not a bag of methods

P03 retains the useful engineering concept that a user can select and maintain one package. Its semantic content includes phase behavior, necessary caloric description, references, required pure/interaction data, physical eligibility and any required transport/effective-property choices. Those choices can be composed explicitly or realized by one provider.

The system does **not** require every model family to derive from one potential or expose all the same low-level primitives. A potential-based engine, an activity-based assembly, a steam formulation, a reactive chemical system and a mass-only empirical model can supply different operation subsets. Compatibility requirements follow the actual physical relationships being combined, not merely uniform software signatures. [B3 §§7.1–7.5; B4:FR-GOV-04,FR-CFG-02,FR-CFG-04]

There is also no requirement to expose every internal submodel when a whole provider is the coherent boundary. A native flash using its own compatible phases and data can be preferable to reassembling primitives from unrelated engines. Interchangeability is subordinate to retained meaning and demonstrated compatibility.

### 7.2 Chemistry is a definition domain, not a mandatory standalone solver

P06 separates reaction/speciation meaning from equipment closure. It does not assert that “chemistry happens first, then thermodynamics.” Phase allocation, chemical species, energy and restrictions can form one coupled P05 problem. A whole Reaktoro chemical system or an equation-oriented formulation can preserve that coherence inside P08. A nonreactive flash instead keeps species totals fixed because its authority contract says so.

A sequential chemistry/flash procedure is permissible only with a stated approximation or convergence strategy and corresponding acceptance checks. If the flash invalidates the chemical conditions and they are not reconciled, the result cannot be labeled fully coupled equilibrium. Strict conversion remains distinct from reactant-limited optimization, and finite-rate/frozen participation remains distinct from equilibrium. [B4:FR-CHM-02–FR-CHM-08]

### 7.3 Data preparation may use numerical work without owning execution

Raw observations, inferred parameter values, characterization recipes, trial coefficients and approved datasets have different meanings. P02 preserves those distinctions. It can define the fitting problem and interpret the fit. P07 coordinates a fitting campaign and P08 performs the selected numerical realization. P03’s trial package may deliberately use provisional coefficients, but evaluation cannot change them except through a new explicit trial.

After review, P10 commits the selected parameter/material/package proposal. A failed numerical trial does not invalidate the approved production definition; it remains evidence about that trial. Scientific uncertainty estimation and independent validation are capabilities/evidence to qualify, not automatically granted by a successful fit. [B4:FR-DAT-03,FR-DAT-04,FR-DAT-05,FR-DAT-06]

### 7.4 State, specification and numerical trial remain different

P04 can hold an intentionally partial local description; P05 determines whether it is sufficient for the requested standalone result or legitimate as part of a coupled process model. A specified number, observation, previous result and initial guess retain their roles. Root/density selection is not automatically phase redistribution, and phase redistribution is not automatically chemical transformation.

A single-component saturated TP observation can support endpoint questions while not specifying intermediate phase amounts. Two separate bulk phases in a rate-based contactor can retain different temperatures/compositions and ask for interface quantities. No universal “two state numbers” rule or “always flash the stream” rule defines the architecture. [B4:FR-STA-02–FR-STA-04,FR-EQL-04,FR-FLW-08]

### 7.5 Thermodynamic transport is not equipment transport

A phase viscosity or diffusivity is requested under P05 and supplied by P08 using qualified P03 methods/data. An effective slurry viscosity needs an additional named bulk closure, also qualified and traceable. An interfacial property identifies the relevant phase/domain pair. A mass-transfer coefficient, interfacial area or hydraulic pressure drop belongs to the unit’s physical model exposed through P07, even when it consumes those properties.

The location of a useful correlation in a library does not by itself determine domain ownership. An adapter can call a provider utility while the simulator still attributes the resulting equipment or effective-property assumption to the right conceptual responsibility. [B4:FR-PRP-03,FR-PRP-04,FR-FLW-01,FR-FLW-08]

### 7.6 Three forms of phase scope must remain visible

P03 defines the physical candidates and permitted constituents. P05 defines the particular request’s structural/physical candidates and any authorized restrictions. P08 records what its algorithm actually searched or retained. P09 limits the stability/coverage claim to the evidence supplied.

P04 separately distinguishes present phase instances, incipient or numerical entries and aggregates. A numerical container size does not establish physical phase count. Across results, correspondence can be ambiguous; the architecture does not promise a unique continuous label at coalescence. P07’s product routing can change while the internal state’s identity remains independently described. [B4:FR-CFG-07,FR-STA-05,FR-STA-06,FR-EQL-06,FR-FLW-03]

### 7.7 Acceptance has local, process and currentness dimensions

A P09 assessment can establish that a local PH result satisfies the original target and component balances. That does not establish the surrounding compressor efficiency relation, column balances or recycle convergence: P07 must supply the relevant process conditions/evidence. Even a checked whole-flowsheet result may be obsolete if the underlying revision changed before publication.

P10 is the sole authority for the current-result binding. It receives the requested publication scope from P07, qualification evidence from P09, and the captured revision/run/cancellation context. The binding update and context check must be one logical operation. This is a correctness requirement, not a selection of database technology.

The current revision can be updated while numerical work continues. Such work may finish as attributable history, but cannot silently replace the new current result. An uninterruptible native call does not excuse late publication after cancellation. [B4:FR-RES-02–FR-RES-07,FR-RUN-06,FR-RUN-08,FR-LIF-01]

### 7.8 Equation-oriented integration is a first-class realization, not an exception

P07’s host mathematical model owns global unit equations and unknowns. P05 describes the thermodynamic relationships and local authority required at each location. P08 may supply explicit equations/residuals or value callbacks with qualified derivative support. The architecture does not require every provider to emit symbolic expressions, nor does it treat numerical differentiation as automatically acceptable for any solver demand.

Initialization that fixes variables, relaxes constraints or changes an active phase set must identify what changed and restore or explicitly reconcile it before accepting the original problem. The equations’ derivative is not necessarily the total sensitivity of a re-equilibrated flash; both need their own P05 request meaning and P09 evidence. [B4:FR-FLW-07,FR-PRP-05,FR-PRP-06,FR-RUN-02]

### 7.9 Readiness is a coordinated assessment, not a service cycle

P03 defines the selected package’s compatibility and readiness interpretation. P05 assesses the request’s specification and authority. P08 reports actual exposed capability, initialization and session restrictions. P07 declares the unit’s requirements and supplies those attestations to the readiness assessment. P09 reports whether supporting validation/coverage evidence exists.

A missing diffusivity may block a rating or rate-based calculation while leaving enthalpy evaluation available. A mathematically expressible inverse problem may lack an initializer. A wrapper may be available while its external engine is absent. The aggregate assessment must retain these reasons separately. No package calls “up” into a flowsheet to fill an unannounced default. [B4:FR-GOV-02,FR-GOV-03,FR-CFG-03,FR-PRP-02,FR-RUN-01]

### 7.10 Defined semantics, bounded scope and maintainability

An extension is acceptable when it supplies its material/domain meaning, quantity basis, supported operation meaning, mutation/authority rules, evidence and persistence information to the relevant owners. It is not acceptable solely because arbitrary metadata can be attached. At the same time, the core need not implement every specialist operation. Unsupported numerical requests remain explicit while P3 representations remain meaningful.

This avoids both extremes: a universal molecular-fluid abstraction that forces fake data, and a catch-all extension bag that bypasses balances and lifecycle rules. The architectural requirement is to accommodate known variation with named boundaries, not to predict every possible future physical theory. [B4:FR-MAT-02,FR-EXT-01–FR-EXT-04]

## 8. Handoff and authority summary

A change, calculation, check and publication are separate actions even when a caller experiences them as one command. The table below identifies the semantic decision, not an implementation call signature.

| Handoff | Producer / responsible party | Receiver | What is transferred | Not transferred |
| --- | --- | --- | --- | --- |
| Material/parameter proposal | P01/P02/P06/P03 under P07 coordination | P10 | Validated domain proposals and their dependency/quantity effects | Permission to mutate approved definitions during a calculation |
| Effective binding | P07 | P05/P08 through captured request | Resolved location, package revision and authorized local overlays | Ambient current-stream authority |
| Operation demand | P05 using P07 context | P08 | Original constraints, allowed changes, necessary output/check scope | Authority to simplify the physical problem silently |
| Numerical candidate | P08 | P09, coordinated by P07 | Normalized values plus actual execution evidence and unresolved outputs | An assertion that all process or physical checks passed |
| Missing check evidence | P09 | P07/P08 through P05 request | A bounded additional evaluation under the captured convention and context | Permission to change the original target or approve defaults |
| Local acceptance | P09 | P07 | Assessment at a named local scope; candidate remains tied to captured inputs | Whole-unit/flowsheet convergence |
| Unit or flowsheet publication proposal | P07 and P09 | P10 | Required scope, matched candidate collection, check evidence and run/revision identity | Unconditional currentness |
| Guarded publication | P10 | Consumers including P07/P09 | Current-result binding or explicit rejected/stale/cancelled publication decision | A revised thermodynamic or numerical validity claim |
| Change invalidation | P10 | P07/P08/P09 | New revision and affected or conservatively invalidated objects/results/sessions | Silent relabeling of old values under new definitions |
| Archive restoration | P10 with domain snapshots | P07/P08/P09 | Reconstructed meaning, missing dependency facts and restored evidence | Automatic approval of results under changed equations or providers |

### Internal use of local candidates

A coupled run needs local results before the whole run converges. The coordinator may retain a run-local, revision-bound view of candidates and locally checked states for subsequent calculations. That view is not the same as the published accepted whole-flowsheet result. Local acceptance, final unit acceptance and final plant acceptance have distinct scopes, so the architecture does not deadlock waiting for global convergence before it can use any local values.

Related outlet values should be proposed as a coherent publication set at the required unit/flowsheet scope. Partial diagnostic results can still be visible as diagnostics; they must not silently replace only half of a previously accepted coupled result.

## 9. Complete conceptual handoff walkthroughs

These walkthroughs demonstrate where each responsibility acts and which shortcuts are forbidden. They allocate the already defined behavior; they do not execute fixtures or complete the detailed information/action specifications of Steps 6–7.

### PW-01. Prepare, estimate and approve an ordinary package

**Scope:** SC-01, SC-08, SC-09.

**Sequence:**

1. **P07**: Collect material intent, required unit modes and chosen preparation policy.
2. **P01**: Resolve constituent identities and representation semantics.
3. **P02**: Resolve available data; issue a proposal for missing-parameter estimation rather than silently estimating on association.
4. **P03**: Describe a fixed trial configuration and its permitted evaluation scope.
5. **P08**: Under P07 coordination, evaluate the fixed trial or run the selected fitting/estimation realization; return candidate evidence.
6. **P02**: Assess fit/estimation evidence and prepare the selected parameter proposal with provenance.
7. **P03**: Assess method, caloric and operation-specific compatibility using supplied capability/initializer evidence.
8. **P10**: Commit the validated material/data/package proposal and binding under one consistent revision.

**Forbidden shortcut:** A property call must not choose new interaction parameters while appearing to evaluate the old definition. Fitting evaluations cannot recursively trigger the same unresolved data preparation.

**Requirements:** FR-DAT-01, FR-DAT-03, FR-DAT-04, FR-DAT-06, FR-CFG-01, FR-CFG-03. **Basis:** B2:WF-03, B3:W01.

### PW-02. Heater, pump, compressor and PH valve state resolution

**Scope:** SC-02, SC-03, SC-04, SC-05, SC-06, SC-12, SC-13, SC-14.

**Sequence:**

1. **P07**: Resolve the local binding and selected equipment mode; establish process constraints and property demands, including a separate ideal PS reference where needed.
2. **P04**: Capture feed values, target values and guesses without changing their roles.
3. **P05**: Prepare a TP/PH/PS/saturation or property request with independent constraints, allowed phase response and original target.
4. **P08**: Check initializer/session eligibility, execute on isolated working state, and return candidate values plus used assumptions.
5. **P09**: Check the original target, balances, authority and promised property completion; ask for missing required evidence if necessary.
6. **P07**: Apply the unit-owned efficiency/work/energy relation and coordinate any further outlet request; declare whether publication is constraints or a resolved state.
7. **P10**: Publish only the accepted scope for the captured current revision and active run.

**Forbidden shortcut:** A successful PS reference calculation is not automatically the real compressor outlet; saturated TP cannot invent phase quality; missing pump rating transport cannot be silently replaced with duty-only success.

**Requirements:** FR-EQL-02, FR-EQL-03, FR-EQL-04, FR-FLW-01, FR-STA-03, FR-RES-02, FR-RES-04. **Basis:** B2:WF-06, B3:W03.

### PW-03. Multiphase separator and product routing

**Scope:** SC-01, SC-05, SC-10, SC-11, SC-23.

**Sequence:**

1. **P07**: Mix compatible feed quantities and energy under explicit pressure/heat rules.
2. **P05**: Describe the required shared phase-allocation problem and permitted solid participation.
3. **P08**: Execute an eligible multiphase operation; return actual and incipient entries without hiding searched scope.
4. **P04**: Describe physical phase instances and any ambiguous correspondence separately from positional output.
5. **P09**: Qualify the internal result against conservation, requested phase scope and available stability evidence.
6. **P07**: Apply the declared light/heavy routing, entrainment and solids-carryover rules to form product candidates, including empty outputs.
7. **P09**: Check product reconciliation at the unit scope, not only the internal flash.
8. **P10**: Publish the coherent accepted product set or retain the prior accepted set.

**Forbidden shortcut:** An array slot is not a phase identity or a port role; a numerical phase placeholder must not create positive product material.

**Requirements:** FR-STA-05, FR-STA-06, FR-EQL-08, FR-FLW-02, FR-FLW-03, FR-RES-03. **Basis:** B2:WF-07, B3:W04.

### PW-04. Internal column or nonequilibrium contacting formulation

**Scope:** SC-07, SC-08, SC-09, SC-10, SC-20, SC-26.

**Sequence:**

1. **P07**: Declare stage/segment locations, independent bulk/interface descriptions, transfer/routing laws and required coupled unknowns.
2. **P04**: Represent local values as partial, guessed or supplied without turning them into graphical streams.
3. **P03**: Qualify phase/property/chemistry and transport conventions for the chosen formulation.
4. **P05**: Supply the operation/contribution contracts: phase values, equilibrium relations, rates, derivatives or value callbacks, with authority over each local state explicit.
5. **P08**: Realize only supported contributions and initialization, keeping temporary fixing or relaxed problems identifiable.
6. **P07**: The host mathematical solver coordinates the coupled unit and restores the original constraints before final acceptance.
7. **P09**: Assess bulk/interface transfer, chemistry where present, stage/unit balances and achieved nested convergence.
8. **P10**: Publish accepted unit or whole-flowsheet results only at the established scope.

**Forbidden shortcut:** A bulk-phase property refresh cannot homogenize the two bulk states; an efficiency-corrected equilibrium-stage result cannot claim the full SC-26 formulation.

**Requirements:** FR-FLW-07, FR-FLW-08, FR-PRP-01, FR-PRP-05, FR-RUN-02, FR-RUN-08. **Basis:** B2:WF-10, B3:W02, B3:W04.

### PW-05. Conversion or kinetic reactor with energy

**Scope:** SC-17, SC-19, SC-25.

**Sequence:**

1. **P06**: Declare balanced stoichiometry, strict extent or rate conventions, participation and frozen species.
2. **P03**: Reconcile the selected caloric/formation convention with reaction-energy requirements.
3. **P07**: Supply unit-owned residence time, thermal/mechanical constraints and permitted material exchanges.
4. **P06**: Assess strict conversion feasibility or define the rate/property subproblem without replacing kinetics with equilibrium.
5. **P05**: Prepare the combined or explicitly split phase/thermal/chemistry problem with its actual approximation.
6. **P08**: Execute supported contributions and retain used physical restrictions.
7. **P09**: Check achieved conversion/rates, conserved quantities and energy convention at the declared scope.
8. **P10**: Publish only the checked current result; a separately authorized limited-conversion alternative keeps a different problem identity.

**Forbidden shortcut:** No negative-amount clipping or hidden conversion optimization; do not add the full reaction heat to formation-inclusive material enthalpy a second time.

**Requirements:** FR-CHM-01, FR-CHM-02, FR-CHM-04, FR-CHM-05, FR-RES-03. **Basis:** B2:WF-09, B3:W05.

### PW-06. Reactive aqueous absorption and precipitation

**Scope:** SC-18, SC-20, SC-21, SC-22, SC-24, SC-25.

**Sequence:**

1. **P01**: Define apparent feed, true species and admissible solid identities with justified conserved bases.
2. **P06**: Declare chemical-system participation, constraints and any authorized reservoir exchanges.
3. **P03**: Resolve compatible standard-state/activity/caloric methods, with transport required only when the unit needs it.
4. **P05**: Form one reacting multiphase problem, or a named coupling approximation, with original thermal and exchange constraints.
5. **P08**: Use a coherent chemical-system provider or coupled mathematical realization; amounts and phase state remain candidates.
6. **P09**: Verify chemical, phase, thermal and conservation conditions together, including every allowed exchange and any omitted/unchecked stability evidence.
7. **P07**: Reconcile reporting coordinates without duplicate inventory and apply equipment product/transfer rules.
8. **P10**: Commit accepted unit results only after coupled and process checks succeed.

**Forbidden shortcut:** A speciation result cannot be labeled energy-balanced without caloric completion, and a later flash cannot invalidate chemistry while leaving a fully coupled success claim.

**Requirements:** FR-CHM-03, FR-CHM-05, FR-CHM-06, FR-CHM-07, FR-CHM-08, FR-CFG-04. **Basis:** B3:W05.

### PW-07. Heat-only coupling across packages

**Scope:** SC-02, SC-12, SC-28.

**Sequence:**

1. **P07**: Bind each material region independently and declare heat/loss/work relations; no cross-wall component mapping is requested.
2. **P05**: Prepare side-specific state/property requests under each side’s coherent caloric convention.
3. **P08**: Evaluate side candidates and separately report an unavailable optional hypothetical bound.
4. **P09**: Check within-side energy differences and required property postconditions.
5. **P07**: Assemble the unit heat balance and side outlet candidates without exchanging compounds.
6. **P09**: Qualify the coupled exchanger scope.
7. **P10**: Publish both sides consistently for the captured model revision.

**Forbidden shortcut:** No requirement for identical absolute enthalpy zeroes or a common molecular slate merely to transfer heat. Optional-report failure does not waive a necessary energy check.

**Requirements:** FR-FLW-04, FR-PRP-02, FR-RES-03. **Basis:** B2:WF-08, B3:W06.

### PW-08. Material transition and information-losing translation

**Scope:** SC-16, SC-21, SC-29, SC-30.

**Sequence:**

1. **P07**: Declare source/target bindings, preserved boundary quantities and allowed physical exchanges.
2. **P01**: Assess coordinate mapping, justified conservation and whether a reverse map loses information.
3. **P03**: Supply package conventions and a known reference relation where available; do not invent one.
4. **P05**: Construct the target state problem using only the chosen consistent constraints.
5. **P08**: Evaluate the target description under its qualified realization.
6. **P09**: Separate reference adjustment, model discrepancy, mapping residual and an actually modeled heat/work exchange.
7. **P07**: Prepare target material/reporting outputs with information loss or unresolved boundaries visible.
8. **P10**: Publish the accepted translation or retain an explicitly incomplete boundary.

**Forbidden shortcut:** Preserving T,P and composition cannot also preserve inconsistent corrected enthalpy by hiding a duty; one lump cannot be uniquely un-lumped without new assumptions.

**Requirements:** FR-FLW-05, FR-FLW-06, FR-CHM-06, FR-MAT-07, FR-RES-03. **Basis:** B3:W06.

### PW-09. Petroleum characterization, reduced oil and empirical material

**Scope:** SC-15, SC-16, SC-23, SC-31.

**Sequence:**

1. **P02**: Retain the original assay or empirical evidence and prepare explicit cut/correlation or reduced-model proposals.
2. **P01**: Validate the proposed material meaning; reject invented registry identities or molecular data.
3. **P03**: Qualify requested operations separately: PVT, caloric completion, inverse state and supported reporting.
4. **P10**: Commit compatible material/data/package revisions while preserving raw and derived lineage.
5. **P07**: Request only the unit behavior justified by the chosen material representation.
6. **P08**: Execute an eligible complete provider or limited correlation realization without hidden molecular placeholders.
7. **P09**: Check balances on justified bases; missing elemental data is not an elemental-balance pass.
8. **P10**: Publish current results or retain explicit unresolved P2 coverage.

**Forbidden shortcut:** Creating cubic pseudocomponents is not itself an assay workflow or complete energy package. An empirical mass-only heating model does not thereby expose fugacity.

**Requirements:** FR-DAT-05, FR-CFG-04, FR-MAT-02, FR-RES-03, FR-GOV-05. **Basis:** B3:W08.

### PW-10. Shared definition edit, cancellation and late completion

**Scope:** Cross-cutting lifecycle behavior across applicable scenarios.

**Sequence:**

1. **P10**: Record an accepted revision A and issue a captured-context token for a run.
2. **P08**: Execute the A request using isolated workspaces.
3. **P07**: Submit an authorized edit or cancellation through the shared lifecycle path.
4. **P10**: Commit revision B or revoke publication for the run; mark dependent results/sessions stale or needing reassessment.
5. **P08**: Return a late A candidate or a cancelled/provider-failed outcome without writing current material.
6. **P09**: Assess any retained A candidate for its original request only, not as evidence for B.
7. **P10**: At the publication commit point, reject the A/cancelled current binding; retain attributable history if useful.

**Forbidden shortcut:** Checking revision before lengthy verification is insufficient unless checked again atomically with current-result update. Cancellation cannot rely solely on immediate interruptibility of native code.

**Requirements:** FR-RUN-06, FR-RES-05, FR-LIF-01, FR-LIF-08. **Basis:** B3:W07.

### PW-11. Save, inspect, reconstruct and reproduce

**Scope:** Cross-cutting lifecycle behavior across applicable scenarios.

**Sequence:**

1. **P07**: Coordinate the archive or restoration use case.
2. **P10**: Capture semantic snapshots, resolved data/manifests, bindings, policies and historical-result references; no live handle is the definition.
3. **P01**: On restore, re-establish material identity and mapping meaning from supplied snapshots.
4. **P02**: Re-establish source data and characterization lineage without re-fitting as a load side effect.
5. **P03**: Reconstruct coherent package definitions and explicit missing prerequisites.
6. **P08**: Report dependency availability and realize compatible sessions only when available.
7. **P09**: Assess restored evidence and any rerun as same-configuration reproduction or changed-model comparison.
8. **P10**: Keep inspection load, readiness and restored-current-result authority separate.

**Forbidden shortcut:** Loading final arrays into changed equations does not restore original meaning; absent providers must not cause silent substitution.

**Requirements:** FR-DAT-06, FR-LIF-04, FR-LIF-05, FR-LIF-06, FR-LIF-07. **Basis:** B2:WF-14, B3:W07.

### PW-12. P3 surface/selective-transfer material

**Scope:** SC-27.

**Sequence:**

1. **P01**: Describe bulk domains and separate site/surface loading, including its site/area/sorbent-mass basis and constituent identity.
2. **P04**: Represent bulk and stored surface quantities as distinct local descriptions, not fictitious liquid flow.
3. **P07**: Declare the sorbent amount, membrane sides or area as process-owned context and the permitted transfer action.
4. **P05**: Describe a site-exchange/selective-transfer property or constrained subproblem; unsupported numerics remain explicit.
5. **P09**: Walk through accounting: an authorized amount delta removed from the bulk appears in the receiving surface or opposite-side account on the declared conversion basis.
6. **P10**: Retain the extension definition and limited capability without claiming numerical E/V.

**Forbidden shortcut:** A loading scalar with no denominator or relationship to conserved material is not sufficient representability. Sorbent geometry and throughput are not thermodynamic identity.

**Requirements:** FR-EXT-01, FR-MAT-03, FR-MAT-08, FR-FLW-08. **Basis:** B4:FR-EXT-01.

### PW-13. P3 distribution-valued material

**Scope:** SC-32.

**Sequence:**

1. **P01**: Define the attribute distribution, support coordinate and mass/number or other basis; a mean is a reduction, not the full object.
2. **P04**: Associate a local distribution with its actual material amount.
3. **P07**: For a mass-basis mixture, combine compatible distributions with material mass weights; translate incompatible coordinates only under a declared map.
4. **P01**: If an approved reduced representation is requested, record retained moments and the information lost.
5. **P09**: Check that material accounting and the declared reduction are coherent; reject a unique inverse from insufficient moments.
6. **P10**: Retain original/reduced lineage and explicit unimplemented thermodynamic operations.

**Forbidden shortcut:** Attaching a polymer label and mean molecular weight does not preserve a chain-length distribution; mixing number-basis distributions as mass-basis distributions changes meaning.

**Requirements:** FR-EXT-02, FR-FLW-06, FR-DAT-05. **Basis:** B4:FR-EXT-02.

### PW-14. P3 closed inventory and constrained branch

**Scope:** SC-33, SC-34.

**Sequence:**

1. **P04**: Represent a nonempty inventory with known material amounts/composition, total U and total V; T and P may remain unknown. No rate or implicit duration is created.
2. **P03**: Describe a compatible material/caloric definition and permitted physical scope.
3. **P05**: Express a UV problem or restricted branch request, retaining excluded phases/reactions and distinguishing a starting guess.
4. **P08**: Report the actual available realization; absent numerical support leaves an explicit representability-only description.
5. **P09**: For a conceptual empty inventory, record that composition/intensive fluid state is not uniquely defined by absent material; restrictions limit any stability claim.
6. **P10**: When a restriction is lifted, mark the prior restricted result inapplicable to the new problem without discarding its historical meaning.

**Forbidden shortcut:** No dummy kg/s converts inventory to flow, no total U,V-only unique state without justified material data, and no restricted result is certified as unrestricted stable equilibrium.

**Requirements:** FR-EXT-03, FR-EXT-04, FR-MAT-04, FR-EQL-06. **Basis:** B4:FR-EXT-03, B4:FR-EXT-04.

### What the P3 walkthroughs establish here

PW-12–PW-14 provide an allocated semantic route for site-associated storage, distribution reduction, inventory-state problems and physical restrictions. They do not award an independent R pass. Step 6 must complete the information concepts and invariants, and Step 7 must complete the actions before those representability witnesses are reviewed as fulfilled. Their numerical E/V scope remains deferred exactly as in B4.

## 10. Mapping the responsibilities back to the researched libraries

The following uses only the fixed B2/B3 findings. It is an allocation of candidate integration roles, not a refreshed feature inventory or backend recommendation based on unexecuted tests. The P-numbers identify semantic responsibilities a provider can help realize internally; they do not say the library already implements our contracts.

### DWSIM

**Candidate integration boundary:** Whole configured package through an adapter; selected methods only after separate qualification.

**Responsibilities informed or realized:** P02, P03, P04, P05, P06, P08.

Use complete flowsheet workflows to assess P07/P09/P10 needs. Its current-stream representation and package internals may remain inside P08, rather than becoming P04’s canonical state.

**Host obligations and limits:** Host material meaning, cross-provider bindings, allowed-change contract, result qualification and currentness remain explicit. Ordinary defaults and optional product/adapter availability cannot silently establish coverage.

**Evidence basis:** B2; B3 §2.

### ChEDL thermo

**Candidate integration boundary:** Configured constants/correlations + phase models + eligible flash algorithm.

**Responsibilities informed or realized:** P02, P03, P04, P05, P08.

A candidate coherent conventional property-and-flash realization, not merely a source of constants. Phase-only and effective-property semantics inform P04/P05.

**Host obligations and limits:** The inspected flash families have different bounds; do not infer multicomponent precipitation from an N-phase name. Host supplies process coordination and qualified lifecycle.

**Evidence basis:** B3 L05; W01–W04.

### ChEDL chemicals

**Candidate integration boundary:** Data/correlations or selected numerical primitives, separately qualified.

**Responsibilities informed or realized:** P02, P05, P08.

Supports evidence preparation and qualified calculation kernels; an inner allocation routine using supplied K-values is not a complete equilibrium package.

**Host obligations and limits:** Model assembly, caloric/standard-state compatibility, equilibrium authority and adoption remain elsewhere.

**Evidence basis:** B3 L05; CH02.

### Clapeyron

**Candidate integration boundary:** Configured composite thermodynamic model with a compatible algorithm, or selected qualified model contributions.

**Responsibilities informed or realized:** P02, P03, P05, P08.

Principal reference for method composition, parameter meaning, initializer eligibility and solved-state derivative distinctions.

**Host obligations and limits:** Require exact model/algorithm/specification/data/initialization compatibility. Do not inherit all catalog combinations or generic transport/characterization coverage.

**Evidence basis:** B3 L03; W01–W04.

### IDAES

**Candidate integration boundary:** Equation-oriented property/reaction contribution or a qualified solved submodel.

**Responsibilities informed or realized:** P03, P04, P05, P06, P08.

Shared parameters/local states, declared eligibility, initialization and translator constraints inform P03–P07.

**Host obligations and limits:** The host process mathematical model and its solver remain peers of P07/P08. Retain the construction recipe, not merely state serialization. A symbolic submodel is not automatically a flash sensitivity.

**Evidence basis:** B3 L02; W02–W06.

### ThermoPack

**Candidate integration boundary:** Configured EOS session and the actual supported property/state operations.

**Responsibilities informed or realized:** P02, P03, P05, P08.

Candidate focused EOS/value/derivative/flash engine; keep activation and any wrapper-specific ordering in P08.

**Host obligations and limits:** Qualify exposed multiphase/inverse routes, pseudocomponent caloric completion, native errors and full-sequence isolation. Internal parallelism is not assumed to prove cross-case reentrancy.

**Evidence basis:** B3 L04; W03; W07–W08.

### CoolProp

**Candidate integration boundary:** Selected backend/fluid/mixture session with its actual data, reference and operation support.

**Responsibilities informed or realized:** P03, P04, P05, P08.

Candidate specialized-fluid realization and session/reference-lifetime precedent; internal state does not become the accepted host state.

**Host obligations and limits:** Support varies by actual backend/build/binding and mixture. Session reuse after reference/configuration changes must be qualified; optional external engines remain distinct dependencies.

**Evidence basis:** B3 L06; W03; W07.

### FeOS

**Candidate integration boundary:** Coherent potential-based model with required ideal/caloric contributions and qualified state/equilibrium methods.

**Responsibilities informed or realized:** P02, P03, P04, P05, P08.

Reference for intensive versus extensive descriptions and implicit equilibrium derivatives; valid selected-model numerical candidate.

**Host obligations and limits:** Do not infer arbitrary phase count, generic inverse initialization, complete petroleum characterization or mass-only material support from the model abstraction.

**Evidence basis:** B3 L07; W03; W08.

### Reaktoro

**Candidate integration boundary:** Coherent chemical-system realization, including database, admitted species/phases, models and constraints.

**Responsibilities informed or realized:** P02, P03, P05, P06, P08.

Chemistry and phase behavior can remain one numerical problem. Upstream system/rate/sensitivity contracts and the narrower DWSIM adapter are different realizations.

**Host obligations and limits:** Host owns inventory/flow conversion, process exchanges and reporting meaning. Verify caloric/transport and adapter coverage before claiming an energy-balanced reactive unit.

**Evidence basis:** B3 L08; W05.

### CAPE-OPEN

**Candidate integration boundary:** Contract reference; an implemented standards adapter would sit in P08.

**Responsibilities informed or realized:** P01, P04, P05, P06, P07.

Informs material/slate meaning, property versus equilibrium postconditions, contextual association and reaction-energy compatibility.

**Host obligations and limits:** Not a numerical thermodynamic engine. No full standards-compliance or new reaction-interface implementation commitment is introduced here.

**Evidence basis:** B3 L01.

### Whole-provider and composed-model integration are both permitted

**Whole-provider route:** an engineer-facing P03 package is realized by a cohesive DWSIM, ChEDL, Clapeyron, specialized-fluid or chemical-system implementation. P08 maps explicit host inputs into the provider’s accepted context, runs its eligible calculation and returns normalized candidate evidence. The host does not rebuild the provider’s internal flash merely to make the architecture appear uniform.

**Composed-model route:** the host deliberately assembles qualified phase/property contributions. P03 owns compatibility and caloric completion; P05 owns the physical problem; P08 realizes numerical or equation contributions. The host accepts the additional burden of stability, initialization, derivative, reference and failure consistency. Reusing a small kernel is useful only where that burden is justified by the required process formulation.

A single flowsheet may mix these routes across regions. A single request must still have one coherent description of the physical problem it actually solves. Two providers producing similarly named numbers do not automatically become a compatible combined package. [B3 §9.2; B4:FR-GOV-04,FR-CFG-02]

## 11. Eight engineer-facing package examples

These examples test whether the same responsibility map accommodates mainstream breadth. They are semantic bundles to qualify, not approved numerical packages or implementation workarounds.

### EB-01. Conventional hydrocarbon package

**Semantic composition:** Identified species; qualified cubic phase model; complete caloric and needed transport choices; TP/PH/PS and bounded phase policy.

**Candidate realization:** DWSIM, thermo or Clapeyron coherent package; ThermoPack/other focused realization when complete for the request.

**Ownership test:** P07 owns pumps/compressors/valves and recycle closure. A PS reference is separate from actual outlet energy.

### EB-02. Nonideal one-/two-liquid package

**Semantic composition:** Activity-based liquid method plus compatible vapor, standard-state/saturation/Henry and caloric descriptions; bounded candidate liquids.

**Candidate realization:** DWSIM, thermo or Clapeyron; IDAES contribution where the process formulation uses its semantics.

**Ownership test:** Eligibility includes phase count and inverse initializer support. Density-based product routing remains P07.

### EB-03. Water or refrigerant package

**Semantic composition:** Exact pure, pseudo-pure or mixture identity; backend data/reference convention; saturation and inverse-state qualification.

**Candidate realization:** CoolProp or relevant coherent DWSIM/thermo/IDAES routes recorded in B3.

**Ownership test:** No new stream type for steam. Pure saturation ambiguity and quality basis remain P04/P05 obligations.

### EB-04. Reactive aqueous/mineral package

**Semantic composition:** Species/database/activity/standard-state definitions, chemical participation, caloric completion and authorized exchanges.

**Candidate realization:** Reaktoro chemical system or a qualified IDAES construction; another adapter only for its actual exposed surface.

**Ownership test:** P06 defines chemistry, P05 one coupled problem, P07 unit constraints, P09 combined checks. Speciation alone is not a full absorber.

### EB-05. Assay-characterized petroleum package

**Semantic composition:** Raw assay and characterization recipe; generated cuts and property data; declared missing/estimated values and complete required caloric route.

**Candidate realization:** DWSIM workflow reference; eligible configured custom-data route subject to B3’s explicit completion gaps.

**Ownership test:** No candidate is awarded a complete assay/energy workflow here. P02/P01 changes commit together and preserve raw evidence.

### EB-06. Reduced black-oil package

**Semantic composition:** Empirical material coordinates and standard-condition conventions distinct from a compositional molecular mixture.

**Candidate realization:** No complete alternative was established in B3; retain a named unresolved provider/profile requirement.

**Ownership test:** An unsupported-profile response is honest per provider but does not close SC-16 delivery coverage.

### EB-07. Empirical mass-only package

**Semantic composition:** Mass-defined constituents with justified density/caloric correlations; explicit unsupported molecular quantities.

**Candidate realization:** A limited host/provider correlation realization; no complete general library route assumed.

**Ownership test:** P04/P05 permit heating without molecular weight; P09 checks justified mass/energy only.

### EB-08. Nonequilibrium contacting configuration

**Semantic composition:** Separate coherent bulk-phase evaluations, interface conditions, needed transport and permitted phase/species transfer.

**Candidate realization:** Suitable phase/property providers; IDAES/local contribution architecture; chemistry provider where needed.

**Ownership test:** Not one automatic bulk flash. P07 owns transfer laws/area and coupled bulk/interface balance structure.

The absence of a proven complete alternative for black oil or mass-only empirical material remains visible. The architecture provides a home for that functionality, but a home is not an implemented capability. This is particularly important for the breadth-first objective: selecting an excellent fluid engine must not erase P2 materials and operations that require additional work.

## 12. Requirement-to-package allocation

Each B4 requirement has exactly one primary accountable package below. Collaborators are not co-owners of the same invariant; they supply semantic inputs, execution, evidence or lifecycle actions. The JSON companion preserves the original B4 functional-accountability label, obligation class, source basis and acceptance-test identifiers. No normative requirement text is changed.

Cross-family allocation is deliberate. For example, FR-CHM-05 goes to P03 because its actual obligation is reconciliation of the selected package’s energy convention; FR-CHM-08 goes to P09 because its obligation is combined-result verification. P06 remains responsible for chemistry definitions and requirements in both cases.

| Requirement | Title | Class | Primary | Supporting packages |
| --- | --- | --- | --- | --- |
| FR-GOV-01 | Qualify coverage at the scenario and configuration level | H | P09 | P03, P08 |
| FR-GOV-02 | Describe the operation actually exposed by an adapter | H | P08 | P03, P05, P09 |
| FR-GOV-03 | Separate availability from model suitability | H | P08 | P03, P10 |
| FR-GOV-04 | Admit cohesive packages and qualified submodels | H | P03 | P05, P08 |
| FR-GOV-05 | Preserve declared scope and explicit coverage gaps | H | P09 | P07, P08 |
| FR-MAT-01 | Resolve identity without conflating labels | H | P01 | P02, P08 |
| FR-MAT-02 | Declare the material representation and justified operations | H | P01 | P03, P05 |
| FR-MAT-03 | Keep intensive state, flow, and inventory distinct | H | P04 | P01, P07 |
| FR-MAT-04 | Distinguish zero flow, empty material, and missing composition | H | P04 | P01, P05, P07 |
| FR-MAT-05 | Replace a complete composition explicitly | H | P04 | P01, P07, P10 |
| FR-MAT-06 | Patch component quantities with a preservation rule | H | P04 | P01, P07, P10 |
| FR-MAT-07 | Map composition coordinates consistently | H | P01 | P04, P08, P09 |
| FR-MAT-08 | Declare justified conserved quantities | H | P01 | P05, P06, P09 |
| FR-DAT-01 | Resolve parameter values with provenance | H | P02 | P01, P03, P10 |
| FR-DAT-02 | Interpret missing parameters according to the method | H | P02 | P03, P08 |
| FR-DAT-03 | Authorize and record estimation or substitution | H | P02 | P03, P07, P09, P10 |
| FR-DAT-04 | Keep fitted parameter trials separate from approved data | C | P02 | P03, P07, P08, P09, P10 |
| FR-DAT-05 | Retain characterization inputs and generated materials | C | P02 | P01, P03, P07, P08, P10 |
| FR-DAT-06 | Prepare and serialize definitions without hidden evaluation effects | H | P02 | P03, P08, P10 |
| FR-DAT-07 | Preserve separate data and model validity information | H | P02 | P03, P05, P09 |
| FR-CFG-01 | Identify a coherent configured package | H | P03 | P01, P02, P06, P08, P10 |
| FR-CFG-02 | Validate composed-method compatibility | H | P03 | P02, P05, P06, P08, P09 |
| FR-CFG-03 | Establish operation-specific readiness | H | P03 | P05, P07, P08, P09 |
| FR-CFG-04 | Qualify complete caloric behavior before energy-constrained use | H | P03 | P02, P05, P06, P09 |
| FR-CFG-05 | Separate physical policy from numerical policy | H | P03 | P05, P06, P08, P10 |
| FR-CFG-06 | Bind configurations to named material regions and local locations | H | P07 | P01, P03, P04, P10 |
| FR-CFG-07 | Declare candidate domains and phase/species eligibility | H | P03 | P01, P05, P06, P08 |
| FR-STA-01 | Create a thermodynamic state at any required local location | H | P04 | P01, P03, P07 |
| FR-STA-02 | Check specification completeness and consistency | H | P05 | P03, P04, P07, P08 |
| FR-STA-03 | Declare calculation authority per quantity family | H | P05 | P04, P07, P08, P09 |
| FR-STA-04 | Preserve specified values separately from guesses and derived values | H | P04 | P05, P07, P08 |
| FR-STA-05 | Identify phase instances and aggregate views without positional meaning | H | P04 | P01, P07, P08, P09 |
| FR-STA-06 | Qualify absent and incipient phase information | H | P04 | P05, P08, P09 |
| FR-STA-07 | Keep flow direction separate from physical composition | H | P04 | P01, P07, P09 |
| FR-STA-08 | Define reporting bases and reference conditions | H | P04 | P01, P03, P05 |
| FR-EQL-01 | Resolve a supported TP equilibrium problem | C | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-02 | Resolve PH with a verified enthalpy target | C | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-03 | Resolve PS and distinguish reference from actual equipment state | C | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-04 | Distinguish saturation endpoints from complete phase allocation | C | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-05 | Return only the promised post-equilibrium properties | H | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-06 | Qualify stability and searched phase scope | H | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-07 | Identify branches and honor branch-selection policy | H | P05 | P03, P04, P06, P08, P09 |
| FR-EQL-08 | Expose bounded multiple-liquid and solid-equilibrium coverage | C | P05 | P03, P04, P06, P08, P09 |
| FR-PRP-01 | Evaluate a supplied phase without unrequested redistribution | C | P05 | P03, P04, P06, P08, P09 |
| FR-PRP-02 | Distinguish primary and optional property demands | H | P05 | P07, P08, P09 |
| FR-PRP-03 | Provide phase transport and phase-pair properties with explicit scope | C | P05 | P03, P04, P06, P08, P09 |
| FR-PRP-04 | Name the closure behind an effective multiphase property | C | P05 | P03, P04, P06, P08, P09 |
| FR-PRP-05 | Define derivative meaning before computation | H | P05 | P03, P04, P06, P08, P09 |
| FR-PRP-06 | Qualify derivative method, conditioning, and nonsmoothness | C | P05 | P03, P04, P06, P08, P09 |
| FR-PRP-07 | Keep caloric, standard-state, and chemical-potential conventions explicit | H | P03 | P02, P05, P06, P09 |
| FR-CHM-01 | Declare reaction definitions independently of reactor closure | C | P06 | P01, P02, P03, P07 |
| FR-CHM-02 | Honor strict specified-conversion feasibility | C | P06 | P01, P04, P07, P09 |
| FR-CHM-03 | Resolve chemical equilibrium under declared constraints | C | P05 | P03, P04, P06, P08, P09 |
| FR-CHM-04 | Preserve finite-rate and frozen-chemistry assumptions | C | P06 | P03, P05, P07, P08, P09 |
| FR-CHM-05 | Reconcile reaction and formation energy exactly once | H | P03 | P02, P06, P07, P09 |
| FR-CHM-06 | Map apparent and true species without duplicate inventory | C | P06 | P01, P04, P05, P07, P09 |
| FR-CHM-07 | Account explicitly for chemically constrained external exchanges | C | P06 | P01, P05, P07, P09 |
| FR-CHM-08 | Verify combined reaction and phase conditions | C | P09 | P03, P05, P06, P07, P08 |
| FR-FLW-01 | Expose the unit’s property demands and physical constraints | H | P07 | P01, P03, P04, P05, P06, P08, P09, P10 |
| FR-FLW-02 | Preserve mixing and mechanical-splitting semantics | C | P07 | P01, P03, P04, P05, P09 |
| FR-FLW-03 | Separate internal phase state from product allocation | C | P07 | P01, P04, P05, P09 |
| FR-FLW-04 | Couple disjoint material regions through heat | C | P07 | P03, P04, P05, P09 |
| FR-FLW-05 | Translate a material across package boundaries under explicit constraints | C | P07 | P01, P03, P04, P05, P08, P09 |
| FR-FLW-06 | Translate representations with declared conservation and information loss | C | P07 | P01, P04, P06, P09 |
| FR-FLW-07 | Support equation-oriented participation without requiring symbolic providers | C | P07 | P03, P04, P05, P08, P09 |
| FR-FLW-08 | Preserve bulk/interface separation in nonequilibrium contacting | C | P07 | P03, P04, P05, P08, P09 |
| FR-RUN-01 | Qualify initialization independently of formulation support | H | P08 | P03, P05, P07 |
| FR-RUN-02 | Preserve the original problem through initialization | H | P08 | P05, P07, P09 |
| FR-RUN-03 | Bind reusable sessions to compatible configuration revisions | H | P08 | P03, P10 |
| FR-RUN-04 | Isolate complete calculation sequences across cases | H | P08 | P04, P10 |
| FR-RUN-05 | Separate same-problem retry from an alternate physical calculation | H | P08 | P03, P05, P07, P09, P10 |
| FR-RUN-06 | Support cancellation without publishing interrupted trials | H | P08 | P07, P09, P10 |
| FR-RUN-07 | Contain provider failures and qualify subsequent reuse | H | P08 | P09, P10 |
| FR-RUN-08 | Record nested convergence independently | H | P07 | P05, P08, P09 |
| FR-RES-01 | Return structured, distinguishable operation outcomes | H | P09 | P03, P04, P05, P06, P07, P08, P10 |
| FR-RES-02 | Adopt results only after declared acceptance checks | H | P09 | P03, P05, P07, P08 |
| FR-RES-03 | Apply conservation and thermal checks on justified bases | H | P09 | P01, P03, P05, P06, P07 |
| FR-RES-04 | Publish only the authorized result scope | H | P09 | P04, P05, P07, P10 |
| FR-RES-05 | Protect accepted results from failed or obsolete work | H | P10 | P04, P07, P08, P09 |
| FR-RES-06 | Expose independent result-quality dimensions | H | P09 | P07, P08, P10 |
| FR-RES-07 | Retain result lineage and requested-versus-used assumptions | H | P09 | P02, P03, P05, P08, P10 |
| FR-RES-08 | Distinguish numerical conformance from physical validation | H | P09 | P03, P08 |
| FR-LIF-01 | Create a revision and impact assessment for consequential changes | H | P10 | P01, P02, P03, P04, P06, P07, P08, P09 |
| FR-LIF-02 | Change a material slate with an explicit quantity/dependency policy | H | P10 | P01, P02, P03, P04, P06, P07, P08, P09 |
| FR-LIF-03 | Separate presentation changes from physical and numerical changes | H | P10 | P04, P07, P09 |
| FR-LIF-04 | Serialize the semantic model and resolved data | H | P10 | P01, P02, P03, P04, P06, P07, P08, P09 |
| FR-LIF-05 | Reconstruct definitions before restoring result authority | H | P10 | P01, P02, P03, P04, P06, P07, P08, P09 |
| FR-LIF-06 | Restore snapshots without losing lineage or dependency consistency | H | P10 | P01, P02, P03, P04, P06, P07, P08, P09 |
| FR-LIF-07 | Reproduce cases within declared tolerances and manifest limits | H | P10 | P07, P08, P09 |
| FR-LIF-08 | Apply the same change semantics through every interface | H | P10 | P01, P02, P03, P04, P07, P09 |
| FR-EXT-01 | Represent surface and selective-transfer domains meaningfully | R3 | P01 | P04, P05, P07 |
| FR-EXT-02 | Represent distributed material attributes and their reductions | R3 | P01 | P02, P04, P07, P10 |
| FR-EXT-03 | Represent inventory-based state specifications without a flow workaround | R3 | P04 | P01, P05, P07 |
| FR-EXT-04 | Represent restricted-equilibrium and metastable requests explicitly | R3 | P05 | P03, P04, P06, P08, P09 |

### Allocation counts

| Package | Primary requirements |
| --- | --- |
| P01 Material semantics and representations | 6 |
| P02 Property evidence, parameters and characterization | 7 |
| P03 Thermodynamic methods and configured packages | 9 |
| P04 Material states, quantities and local descriptions | 11 |
| P05 Thermodynamic problems and operation contracts | 18 |
| P06 Chemistry definitions and participation | 5 |
| P07 Flowsheet integration and use-case coordination | 10 |
| P08 Provider realization and numerical execution | 9 |
| P09 Result qualification and coverage evidence | 10 |
| P10 Model revisions, publication and reconstruction | 9 |

### Explanations for important cross-family allocations

| Requirement | Why this primary owner |
| --- | --- |
| FR-GOV-04 | P03 defines coherent semantic packaging; P08 realizes it at the chosen granularity. |
| FR-MAT-03 | P01 defines basis meaning; P04 owns the local quantity distinctions and conversions that use it. |
| FR-MAT-05 | P04 owns replacement transformation semantics; P10 commits a requested live edit. |
| FR-DAT-05 | P02 owns characterization recipe/evidence; P01 validates generated material meaning; P10 accepts their coordinated revision. |
| FR-DAT-06 | P02 guarantees no hidden preparation effects; P10 owns archive/change orchestration and P08 isolates providers. |
| FR-CFG-03 | P03 owns readiness criteria and interpretation; P07 gathers state, initialization and boundary attestations from their owners. |
| FR-CFG-06 | Location assignment and inheritance are process-context facts, not hidden state on a package. |
| FR-STA-02 | Specification completeness depends on the requested operation or coupled formulation, not only the stored state values. |
| FR-STA-03 | P05 defines permissible changes and conflicts; P08 obeys them and P09 verifies resulting authority. |
| FR-PRP-07 | Conventions are part of the approved package definition; operations and chemistry consume them. |
| FR-CHM-03 | Chemical equilibrium is one P05 problem realized by P08; P06 supplies chemistry definitions, not a mandatory separate solve. |
| FR-CHM-05 | P03 reconciles the actual selected material caloric convention with P06 chemical energy requirements before use. |
| FR-CHM-08 | P09 owns the combined acceptance verdict; P05/P06 define the conditions and P07 supplies unit-level evidence. |
| FR-RUN-08 | P07 owns process nesting and required completion level; P08 only knows the numerical subproblem it executed. |
| FR-RES-04 | P09 determines admissible publication kind and scope; P10 performs the guarded current-binding commit. |
| FR-RES-05 | Protection against obsolete adoption requires the current-revision check and binding update to share one logical P10 authority. |
| FR-EXT-03 | Inventory is a local material-quantity description, not a disguised steady-flow stream. |
| FR-EXT-04 | Restrictions and branch requests modify the physical problem; they are not mere initialization metadata. |

## 13. Scenario coverage by responsibility and handoff

The listed packages own the distinctive primary requirements of each B4 scenario acceptance contract. All applicable host requirements still apply; this is not an exemption list. P07 coordinates process integration, P09 qualifies the required result scope, and P10 guards current publication in applicable executable use cases.

| Scope / acceptance | Scenario | Profile | Primary-requirement owners | Handoff route |
| --- | --- | --- | --- | --- |
| SC-01 / SA-01 | Blending and splitting without reaction | P1 | P01, P04, P07, P09 | PW-01, PW-03 |
| SC-02 / SA-02 | Sensible heating and heat exchange | P1 | P03, P05, P07 | PW-02, PW-07 |
| SC-03 / SA-03 | Liquid pumping and pressure-loss calculations | P1 | P03, P05, P07 | PW-02 |
| SC-04 / SA-04 | Gas compression, expansion and intercooling | P1 | P03, P04, P05, P07 | PW-02 |
| SC-05 / SA-05 | Cooling and vapor–liquid separation | P1 | P04, P05, P07, P09 | PW-02, PW-03 |
| SC-06 / SA-06 | Pressure reduction with flashing | P1 | P05, P08, P09, P10 | PW-02 |
| SC-07 / SA-07 | Conventional equilibrium-stage distillation | P1 | P04, P05, P07, P08, P09 | PW-04 |
| SC-08 / SA-08 | Nonideal-liquid separation and azeotropic behavior | P1 | P02, P03, P05, P08 | PW-01, PW-04 |
| SC-09 / SA-09 | Physical absorption, humidification and gas dissolution | P1 | P03, P04, P05, P07 | PW-01, PW-04 |
| SC-10 / SA-10 | Liquid–liquid extraction and decanting | P2 | P04, P05, P07 | PW-03, PW-04 |
| SC-11 / SA-11 | Vapor–liquid–liquid separation | P2 | P03, P05, P09 | PW-03 |
| SC-12 / SA-12 | Water and steam through saturation | P1 | P04, P05 | PW-02, PW-07 |
| SC-13 / SA-13 | Pure-fluid refrigeration loop | P1 | P01, P05, P07, P10 | PW-02 |
| SC-14 / SA-14 | Mixed-refrigerant phase change | P2 | P01, P03, P04, P05 | PW-02 |
| SC-15 / SA-15 | Assay-derived petroleum pseudocomponents | P2 | P01, P02, P03, P10 | PW-09 |
| SC-16 / SA-16 | Black-oil or other reduced petroleum representation | P2 | P01, P02, P03, P04, P07 | PW-08, PW-09 |
| SC-17 / SA-17 | Specified-conversion reaction | P1 | P01, P03, P05, P06, P09 | PW-05 |
| SC-18 / SA-18 | Chemical-equilibrium reaction | P1 | P01, P03, P05, P06 | PW-06 |
| SC-19 / SA-19 | Kinetically controlled reaction | P2 | P04, P05, P06 | PW-05 |
| SC-20 / SA-20 | Reactive separation | P2 | P03, P05, P07, P09 | PW-04, PW-06 |
| SC-21 / SA-21 | Electrolyte mixing and neutralization | P2 | P01, P03, P05, P06, P07 | PW-06, PW-08 |
| SC-22 / SA-22 | Reactive gas absorption into aqueous liquid | P2 | P03, P06, P07, P09 | PW-06 |
| SC-23 / SA-23 | Inert solids carried with fluid | P2 | P01, P03, P05, P07 | PW-03, PW-09 |
| SC-24 / SA-24 | Crystallization and precipitation | P2 | P03, P04, P05, P09 | PW-06 |
| SC-25 / SA-25 | Gas–solid chemical transformation | P2 | P03, P06, P09 | PW-05, PW-06 |
| SC-26 / SA-26 | Rate-based nonequilibrium contacting | P2 | P05, P07 | PW-04 |
| SC-27 / SA-27 | Adsorption and membrane state extensions | P3 | P01, P03, P04, P05 | PW-12 |
| SC-28 / SA-28 | Heat exchange between different property packages | P1 | P03, P05, P07, P09 | PW-07 |
| SC-29 / SA-29 | Material transfer across a property-package boundary | P2 | P03, P05, P07, P09 | PW-08 |
| SC-30 / SA-30 | Translation between material representations | P2 | P01, P06, P07, P10 | PW-08 |
| SC-31 / SA-31 | Mass-based empirical and nonconventional materials | P2 | P01, P03, P04, P05 | PW-09 |
| SC-32 / SA-32 | Polymer distributions and material attributes | P3 | P01, P04, P07, P10 | PW-13 |
| SC-33 / SA-33 | Inventory-based state and dynamic compatibility | P3 | P04 | PW-14 |
| SC-34 / SA-34 | Restricted-equilibrium and metastable-state requests | P3 | P03, P05, P08 | PW-14 |

All eight integrated journeys and eight synthetic fixtures from B4 remain applicable without being redefined. Their primary requirement IDs resolve through the same allocation above. For example, the synthetic obsolete-completion fixture now has P08 workspace/cancellation behavior, P09 original-request assessment and P10 currentness commit as separate responsibilities. The reference-offset versus model-discrepancy fixture now has P03 convention reconciliation, P07 preserved-boundary constraints, P05 target problem and P09 residual assessment.

No additional thermodynamic acceptance-test passes are claimed. In particular, an architecture scenario route does not imply that the relevant native engine, parameter data, complete unit solver or numerical fixture exists.

## 14. Decision register

**PD-01 | Ten responsibility packages, not ten runtime services.** Use the ten boundaries in this report. They can coexist in one application and can be realized partly inside a cohesive provider. Do not derive a deployment or Rust-crate layout from this count. **Requirement basis:** FR-GOV-04.

**PD-02 | Retain the user-facing configured package.** A user chooses a coherent property-method configuration. The internal architecture separately identifies method families, selected data, physical policy, realization, workspace and result. **Requirement basis:** FR-CFG-01, FR-GOV-04.

**PD-03 | Allow inseparable whole-provider realizations.** A provider may keep phase, caloric, data and flash internals together. Its attributable manifest, observable conventions, eligible operations and limitations must still be exposed. The host does not require arbitrary mixing of internal primitives. **Requirement basis:** FR-GOV-02, FR-GOV-04, FR-CFG-02.

**PD-04 | Preparation is an explicit use case.** Estimation/fitting/characterization produces candidate definitions; no ordinary evaluation/context/serialization call alters approved definitions. P07 coordinates evaluations and P10 commits the selected change. **Requirement basis:** FR-DAT-03, FR-DAT-04, FR-DAT-05, FR-DAT-06.

**PD-05 | Separate stable base definition from local physical problem.** A binding can specialize allowed phase/chemistry policies at a location. The effective physical problem records base revision plus local overlays and constraints. Changing a local restriction need not rewrite an unchanged reusable package, but must change the effective problem identity. **Requirement basis:** FR-CFG-01, FR-CFG-05, FR-CFG-06, FR-STA-03.

**PD-06 | One coupled problem may contain chemistry and phase equilibrium.** P06 owns chemistry meaning; P05 describes the combined physical problem; P08 can solve it as one chemical system or coupled formulation; P09 checks all required conditions together. No mandatory independent chemistry-then-flash sequence. **Requirement basis:** FR-CHM-03, FR-CHM-08, FR-FLW-07.

**PD-07 | State is not a stream and not a workspace.** P04 describes material at a location with optional flow or inventory. A numerical buffer remains P08-private. Acceptance and currentness are additional facts, not automatically attributes inferred from a populated state. **Requirement basis:** FR-MAT-03, FR-STA-01, FR-STA-04, FR-RES-05.

**PD-08 | Calculation authority is request-specific.** P05 states which quantities are fixed, observed, unknown or allowed to change. A property-only call can resolve density without being allowed to redistribute phases or species. **Requirement basis:** FR-STA-02, FR-STA-03, FR-PRP-01.

**PD-09 | Thermodynamics does not route equipment products.** P07 owns physical mixing/splitting, entrainment and product roles. P05 resolves material states; a separator phase and its outlet port are not identical entities. **Requirement basis:** FR-FLW-02, FR-FLW-03.

**PD-10 | Heat and material boundaries are different contracts.** Heat-only coupling uses within-side caloric differences. Material translation declares identity/reference transformations and preserved constraints. Irreversible representation maps retain information-loss labels. **Requirement basis:** FR-FLW-04, FR-FLW-05, FR-FLW-06.

**PD-11 | One currentness commit authority.** P09 supplies acceptance evidence for a captured request; P10 atomically checks matching revision/run/scope at current-result update. P07 proposes the scope, P08 supplies the candidate. A valid A result cannot overwrite revision B. **Requirement basis:** FR-RES-02, FR-RES-04, FR-RES-05, FR-LIF-01.

**PD-12 | Runtime cycles are not circular definition dependencies.** Equilibrium iterations, reaction/phase coupling, fitting and recycle loops are allowed under captured problems. Consumers use exported contracts and snapshots; a provider does not reach up to mutate the flowsheet to resolve its own inputs. **Requirement basis:** FR-FLW-07, FR-RUN-02, FR-RUN-08.

**PD-13 | Readiness is coordinated from several owned assessments.** P03 owns package/readiness interpretation, P05 owns specification authority, P08 owns exposed capability and initializer/session facts, P07 owns process-boundary requirements, and P09 owns evidence quality. P07 assembles these through explicit calls/records. **Requirement basis:** FR-CFG-03, FR-GOV-02, FR-RUN-01.

**PD-14 | Equation contributions share semantics, not mandatory implementation.** A provider can supply values, residuals, equations or sensitivities only as qualified. The host process formulation owns global unknowns and balancing equations; initialization must restore the original problem. **Requirement basis:** FR-FLW-07, FR-RUN-02, FR-PRP-05.

**PD-15 | Deferred scenarios enter the real domains.** Surface/site, distribution, inventory and restriction concepts belong in P01/P04/P05/P06 and process coupling, not an opaque extensions bag. Numerical P3 capability remains deferred. **Requirement basis:** FR-EXT-01, FR-EXT-02, FR-EXT-03, FR-EXT-04.

**PD-16 | Provider choice remains open.** Use the fixed B2/B3 evidence for candidate mappings. No package count, model menu, wrapper name or source path establishes executed or validated scope. P09 retains open P1/P2 gaps. **Requirement basis:** FR-GOV-01, FR-GOV-05, FR-RES-08.

## 15. Structural audit and completion assessment

The companion audit was computed from the architecture register and the actual supplied B4 JSON. It verifies allocation integrity and the declared dependency graph; it does not run thermodynamic operations or certify design correctness.

| Check | Result |
| --- | --- |
| 94 requirements preserved | PASS |
| exactly one primary owner per requirement | PASS |
| supporting packages valid and nonduplicated | PASS |
| original obligation classes preserved | PASS |
| original accountable roles preserved as source fields | PASS |
| 34 scenarios preserved | PASS |
| profiles preserved | PASS |
| all scenarios have walkthrough route | PASS |
| all 94 requirements referenced by package primary lists | PASS |
| no empty primary package | PASS |
| contract dependency graph acyclic | PASS |
| no invalid contract dependency | PASS |
| information product single semantic owner | PASS |
| information product requirements valid | PASS |
| walkthrough owners and requirements valid | PASS |
| decision requirement refs valid | PASS |
| source files verified present | PASS |
| prior source hashes match B4 register | PASS |
| P3 only representability not numerical claim | PASS |
| functional test ids retained | PASS |

### Counts

| Artifact or relationship | Count |
| --- | --- |
| packages | 10 |
| requirements | 94 |
| scenarios | 34 |
| information products | 18 |
| decisions | 16 |
| walkthroughs | 14 |
| library mapping rows | 10 |
| package exemplars | 8 |
| open items | 10 |
| functional acceptance test links | 188 |

### Interpretation of the Step-5 completion gate

**Every requirement has a home:** satisfied as a catalog allocation. Each of the 94 IDs has one primary owner, collaborators and a retained link to its original witnesses.

**No unexplained circular dependency:** the declared contract graph is acyclic, and the five major numerical/information feedback loops have explicit orchestration and no required upward ownership mutation. This is a property of the proposed architecture, not a verified property of code.

**No duplicated authority:** eighteen information products have one semantic owner; current model revision and current-result adoption share P10 authority, while P09 retains acceptance evidence. The semantic-owner/producer/persistence distinction explains records represented in multiple places.

**Bounded unresolved decisions:** phase matching, concrete reference transformations, detailed equation contracts and provider lifecycle policies still need specification and testing. These are recorded below; the architecture does not conceal them as implementation details already solved.

No numerical execution, native-library installation, concurrency experiment, completed independent representability review or physical validation was performed. Structural checks must not be reported as scenario R/E/V passes. [B4:FR-RES-08]

## 16. Open questions with owners

| ID / issue | Primary / supporting | Resolution required | Gate |
| --- | --- | --- | --- |
| PO-01 Phase correspondence | P04 / P01, P05, P07, P08, P09 | Resolve within-result identities and cross-result matching/ambiguity, including coalescence and equal-density changes. | Step 6 before claiming continuous phase identity; no matching algorithm chosen. |
| PO-02 Reference and reactive energy compatibility | P03 / P02, P06, P07, P09 | Define actual reference transformations and formulation-specific formation/reaction correction; do not assume one scalar offset works for all compositions. | Steps 6–8 before accepting a reactive or cross-package energy profile. |
| PO-03 Coupled equation contribution detail | P05 / P03, P07, P08, P09 | Specify local variables, residual/equation/property demands, derivative paths, temporary fixing and evidence extraction. | Steps 6–8; no library or solver choice forced. |
| PO-04 Session containment | P08 / P07, P09, P10 | Choose tested reentrancy, cancellation, failure recovery and realization-isolation policy for exact builds. | Before claiming operationally safe reusable/concurrent execution. |
| PO-05 Dependency invalidation granularity | P10 / P03, P07, P08, P09 | Define precise dependency payloads or retain correct conservative invalidation, including external data artifacts and effective overlays. | Steps 6–7; optimization must not weaken currentness. |
| PO-06 Petroleum and empirical completion | P02 / P01, P03, P07, P08, P09 | Close actual assay, black-oil and mass-only caloric/provider routes. | Step 8/9 qualification before SC-15/16/31 numerical coverage claims. |
| PO-07 Reactive transport completion | P03 / P02, P05, P06, P07, P08, P09 | Identify actual chemistry, phase, caloric and transport combinations for SC-20–26. | Before executed/validated reactive-separation or nonequilibrium profiles. |
| PO-08 Acceptance evidence and check independence | P09 / P05, P07, P08 | Specify required check evidence, cost/optional status, scaling, independent references and the difference between same-model reevaluation and external validation. | Steps 6–9; do not invent one universal accuracy percentage. |
| PO-09 Provider manifests and opaque internals | P08 / P02, P03, P09, P10 | Record exact attributable data/method/build choices even when a provider keeps them inseparable; mark any missing provenance and public-availability limits. | Before reproducibility and distribution/readiness claims. |
| PO-10 Extension information semantics | P01 / P04, P05, P06, P07, P10 | Complete site/loading, distribution reductions, inventory-empty behavior and restricted-branch information dictionaries. | Step 6 followed by Step 7 action contracts; P3 numerical execution remains deferred. |

## 17. Handoff to the information and action blueprint

Step 6 should elaborate the eighteen conceptual information products and the underlying domain concepts. For each, define identity, meaning, cardinality, amount/unit basis, roles, permitted variants, invariants, source/reference semantics, lifecycle, owner and relationships. That is the semantic information dictionary, still not a physical storage schema.

The highest-leverage concept groups are:

- Material identity, representation and coordinate/conservation mappings, including empirical, surface and distribution-valued cases.
- Method/data/reference definitions and effective physical-package bindings, with explicit caloric and chemistry convention reconciliation.
- Local state/specification/authority, physical versus incipient phases and declared derivative response.
- Candidate execution evidence, acceptance scope, currentness guards and reconstructable revision lineage.

Step 7 should turn the handoffs into granular actions: parameter preparation; configuration qualification; binding resolution; property-only evaluation; state/chemistry resolution; coupled equation participation; product/representation translation; acceptance; publication; invalidation; save/rebuild. Define successful, partial, rejected, stale, cancelled and provider-failed paths without requiring one universal numerical algorithm.

Steps 8–9 must resolve actual provider compatibility and run the existing witness plans. Only after those semantics are stable should Rust type boundaries, data representation, solver integration and library choices be derived. The present proposal makes those later choices easier without predetermining them.

**Design thesis:** a flowsheet-driven thermodynamic framework should preserve coherent material/method meaning while allowing heterogeneous numerical realizations. Its fundamental boundaries are approved definition, local problem, numerical work, qualified evidence and current publication. DWSIM supplies the breadth reference; the other frameworks supply complementary precedents. The host makes their interactions explicit rather than inheriting one provider’s object graph.

## 18. Source manifest and references

The source files below were read from the supplied attachments and hashed in this environment. All are version 0.1. The B4 register’s recorded predecessor hashes match the actual B1/B2/B3 files. Source findings retain their earlier evidence level and version pins. References such as B4:FR-STA-03 or B3:W05 point to those documents, not to a claim of new external verification.

| Key | Document/file | SHA-256 |
| --- | --- | --- |
| B1 | [thermodynamics_simulation_behavior_scope_v0_1.md](thermodynamics_simulation_behavior_scope_v0_1.md) | `1d12284bbca6569f198caaae86327788bac8fbaf3946625d99d971ef3602d028` |
| B2 | [dwsim_workflow_reverse_engineering_v0_1.md](dwsim_workflow_reverse_engineering_v0_1.md) | `55e77b6fddab4547280e4e40ca5c742fac3fd322e45534767ddae7f5f6ba94ab` |
| B3 | [thermodynamic_package_comparative_research_v0_1.md](thermodynamic_package_comparative_research_v0_1.md) | `c0f9216c450405360b8532771e6c4beca6536154345ea5adf00130a9c4e73875` |
| B4 | [thermodynamics_functional_requirements_v0_1.md](thermodynamics_functional_requirements_v0_1.md) | `7b9c3c4e2110531cb5767fb00f9ba11dec160294b8cafd909c5b04984a87b5c2` |
| B4-register | [thermodynamics_functional_requirements_v0_1_register.json](thermodynamics_functional_requirements_v0_1_register.json) | `1777f2c60962526252aa863dc2cb673f218b4ed58fe9c108f9721264f9fde825` |

**B1:** THERMO-SCOPE-001 supplies SC-01–SC-34, X01–X24, concept obligations and P1/P2/P3 commitments.

**B2:** THERMO-DWSIM-002 supplies WF-01–WF-15 and the pinned DWSIM workflow evidence, including definition preparation, stream/property authority, unit product publication, nested convergence and reconstruction.

**B3:** THERMO-LIBRARIES-003 supplies L01–L08, W01–W08 and CR-01–CR-16. It also contains the detailed upstream source register and versioned research limitations underlying the candidate library mappings.

**B4:** THERMO-FUNCTIONAL-004 is the normative behavior source. Its original IDs, obligation classes, scenarios and test witnesses are preserved in this allocation. The B4-register companion is the machine-readable baseline used for structural cross-checks.

The complete bundle includes unchanged predecessor documents/register so these references can be inspected together. No upstream source code, library binaries or font files are redistributed by this deliverable.
