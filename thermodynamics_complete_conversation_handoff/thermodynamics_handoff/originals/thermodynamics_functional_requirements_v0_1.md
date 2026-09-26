# Thermodynamics functional requirements and acceptance baseline

**Document:** THERMO-FUNCTIONAL-004  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 4, convert scenarios and research into functional requirements  
**Status:** Proposed functional baseline. Normative statements define intended behavior, not implemented capability.  
**Predecessors:** THERMO-SCOPE-001 v0.1; THERMO-DWSIM-002 v0.1; THERMO-LIBRARIES-003 v0.1.

## 0. Executive decision and use

This baseline contains **94 individually identified requirements**, **34 scenario acceptance contracts**, **8 integrated acceptance journeys**, and **8 synthetic contract-fixture specifications**. Every requirement has an accountable role, initiating actor, trigger, inputs/preconditions, normative behavior, outputs/postconditions, unsuccessful outcome, source trace, and positive and negative/boundary acceptance witnesses. Every B1 scenario, cross-cutting behavior and concept obligation, every B3 CR requirement, and the B2/B3 workflow and probe catalogs have navigation mappings.

The central requirement is **explicit calculation authority**: distinguish evaluating a supplied phase, resolving a state, redistributing material across phases, transforming species, routing products, translating representations, and publishing accepted results. A common property-package concept remains useful, but the provider may implement it as a cohesive engine, a qualified collection of methods, a chemical system, or a mathematical contribution. This is a synthesis of B2 §8.1 and B3 §§5–9, not a mandate for a class hierarchy.

Use this document to review externally observable behavior and to generate later information/action specifications. Use the JSON companion as a requirements-management register; it is **not a proposed material schema or Rust API**. The requirements are independently readable through explicit common rules. A scenario’s listed primary requirements highlight its distinctive obligations; applicable host requirements and cross-cutting rules still apply.

### What was and was not done

This work transformed the three supplied fixed research baselines. It did not conduct new package research, select a backend, inspect the user’s implementation, define thermodynamic storage layouts, choose solvers, or run numerical thermodynamics. Earlier source findings retain their original evidence limitations. Structural catalog checks were performed on IDs, required fields, source references, crosswalks, and profile preservation. No requirement conformance or scenario R/E/V pass follows from those checks.

## 1. Normative interpretation and evidence precedence

“Shall” states an observable obligation under its stated applicability and preconditions. “May” describes an allowed alternative. The logical owner is accountable for the behavior, not a required software service, module, class, thread, or deployment. Several responsibilities can be implemented by one coherent provider. Mutable workspaces and conservative invalidation remain permitted when their observable behavior satisfies the requirements.

B1 establishes scope and acceptance intent. B2 and B3 supply design evidence and limitations, not new scope authority or guarantees about provider behavior. Where evidence and scope differ, the scope wins unless an explicit decision revises it. In particular, a limited-conversion result observed in DWSIM is not a pass for B1’s strict specified-conversion request. The decisions below make this conflict explicit.

| Class | Meaning | Count |
| --- | --- | --- |
| H | Mandatory host behavior wherever its preconditions apply. | 65 |
| C | Behavior mandatory for a numerical/formulation capability the simulator claims; individual providers may reject unsupported profiles. | 25 |
| R3 | Concrete representability obligation for the four P3 extension scenarios; numerical execution deferred. | 4 |

**H does not mean every provider implements every property.** The host must expose a precise unavailable/unsupported result. **C does not make P1/P2 optional.** The overall simulator’s target remains selected executed and validated fixtures across P1 and P2. No single backend must supply all those capabilities, but an unresolved gap must remain visible and cannot be closed by returning “unsupported.” R3 requires later concrete representability walkthroughs, not numerical implementations.

| Profile from B1 | Scenarios | Target retained |
| --- | --- | --- |
| P1 | 14 | Representative conventional steady-state R/E/V fixtures; not full general-purpose breadth on its own. |
| P2 | 16 | Selected bounded broad-process R/E/V fixtures, including petroleum, solids, chemistry, multiple liquids and empirical materials. |
| P3 | 4 | Concrete representability for SC-27 and SC-32–SC-34; numerical E/V deferred. |

### Evidence is not one status ladder

Keep three independent assessments: (1) specification/catalog coverage, (2) implementation conformance to functional contracts, and (3) scenario Representable/Executable/Validated evidence. The present document supplies the first and plans witnesses for the others. Model preparation, provider evaluation capability, initializer readiness, convergence, checked physical postconditions and independent validation are separate facts. A partial or historical result can remain useful without becoming a current accepted process state.

### Common unsuccessful-outcome rules

For every requirement, preserve the original request and accepted result unless an explicit successful change or publication action replaces them. Report the affected operation/property/location and the reason that is actually known. Retain candidate diagnostics only with their unaccepted status. Do not infer physical infeasibility from generic nonconvergence, invent a missing value as zero, or change the physical problem merely to obtain a successful return. Unknown evidence stays unknown. Mandatory-check failure blocks the affected acceptance level; optional-property failure does not retroactively destroy an independent accepted primary result.

## 2. Accountable roles and requirement navigation

| Family | Logical accountability | Requirements |
| --- | --- | --- |
| GOV | Coverage and dependency qualification | 5 |
| MAT | Material definition and quantity semantics | 8 |
| DAT | Data preparation and characterization | 7 |
| CFG | Thermodynamic configuration qualification | 7 |
| STA | State/request interpretation | 8 |
| EQL | Phase-state and equilibrium resolution | 8 |
| PRP | Property and derivative evaluation | 7 |
| CHM | Chemistry and reaction-property coordination | 8 |
| FLW | Process-model integration | 8 |
| RUN | Execution and provider-session control | 8 |
| RES | Result qualification and publication | 8 |
| LIF | Model change and reconstruction | 8 |
| EXT | Extension semantics | 4 |

The process-integration role owns unit operating constraints, internal locations, separation/routing assumptions and the agreement between local and whole-flowsheet publication. Thermodynamics does not own equipment geometry, global process targets, time integration or graphical layout. The provider-control role owns mapping/isolation/error translation at the boundary. Result qualification owns admission to accepted state, independently of a solver’s success flag. These are responsibility assignments for review, not implementation boundaries.

| Requirement | Title | Class |
| --- | --- | --- |
| [FR-GOV-01](#fr-gov-01) | Qualify coverage at the scenario and configuration level | H |
| [FR-GOV-02](#fr-gov-02) | Describe the operation actually exposed by an adapter | H |
| [FR-GOV-03](#fr-gov-03) | Separate availability from model suitability | H |
| [FR-GOV-04](#fr-gov-04) | Admit cohesive packages and qualified submodels | H |
| [FR-GOV-05](#fr-gov-05) | Preserve declared scope and explicit coverage gaps | H |
| [FR-MAT-01](#fr-mat-01) | Resolve identity without conflating labels | H |
| [FR-MAT-02](#fr-mat-02) | Declare the material representation and justified operations | H |
| [FR-MAT-03](#fr-mat-03) | Keep intensive state, flow, and inventory distinct | H |
| [FR-MAT-04](#fr-mat-04) | Distinguish zero flow, empty material, and missing composition | H |
| [FR-MAT-05](#fr-mat-05) | Replace a complete composition explicitly | H |
| [FR-MAT-06](#fr-mat-06) | Patch component quantities with a preservation rule | H |
| [FR-MAT-07](#fr-mat-07) | Map composition coordinates consistently | H |
| [FR-MAT-08](#fr-mat-08) | Declare justified conserved quantities | H |
| [FR-DAT-01](#fr-dat-01) | Resolve parameter values with provenance | H |
| [FR-DAT-02](#fr-dat-02) | Interpret missing parameters according to the method | H |
| [FR-DAT-03](#fr-dat-03) | Authorize and record estimation or substitution | H |
| [FR-DAT-04](#fr-dat-04) | Keep fitted parameter trials separate from approved data | C |
| [FR-DAT-05](#fr-dat-05) | Retain characterization inputs and generated materials | C |
| [FR-DAT-06](#fr-dat-06) | Prepare and serialize definitions without hidden evaluation effects | H |
| [FR-DAT-07](#fr-dat-07) | Preserve separate data and model validity information | H |
| [FR-CFG-01](#fr-cfg-01) | Identify a coherent configured package | H |
| [FR-CFG-02](#fr-cfg-02) | Validate composed-method compatibility | H |
| [FR-CFG-03](#fr-cfg-03) | Establish operation-specific readiness | H |
| [FR-CFG-04](#fr-cfg-04) | Qualify complete caloric behavior before energy-constrained use | H |
| [FR-CFG-05](#fr-cfg-05) | Separate physical policy from numerical policy | H |
| [FR-CFG-06](#fr-cfg-06) | Bind configurations to named material regions and local locations | H |
| [FR-CFG-07](#fr-cfg-07) | Declare candidate domains and phase/species eligibility | H |
| [FR-STA-01](#fr-sta-01) | Create a thermodynamic state at any required local location | H |
| [FR-STA-02](#fr-sta-02) | Check specification completeness and consistency | H |
| [FR-STA-03](#fr-sta-03) | Declare calculation authority per quantity family | H |
| [FR-STA-04](#fr-sta-04) | Preserve specified values separately from guesses and derived values | H |
| [FR-STA-05](#fr-sta-05) | Identify phase instances and aggregate views without positional meaning | H |
| [FR-STA-06](#fr-sta-06) | Qualify absent and incipient phase information | H |
| [FR-STA-07](#fr-sta-07) | Keep flow direction separate from physical composition | H |
| [FR-STA-08](#fr-sta-08) | Define reporting bases and reference conditions | H |
| [FR-EQL-01](#fr-eql-01) | Resolve a supported TP equilibrium problem | C |
| [FR-EQL-02](#fr-eql-02) | Resolve PH with a verified enthalpy target | C |
| [FR-EQL-03](#fr-eql-03) | Resolve PS and distinguish reference from actual equipment state | C |
| [FR-EQL-04](#fr-eql-04) | Distinguish saturation endpoints from complete phase allocation | C |
| [FR-EQL-05](#fr-eql-05) | Return only the promised post-equilibrium properties | H |
| [FR-EQL-06](#fr-eql-06) | Qualify stability and searched phase scope | H |
| [FR-EQL-07](#fr-eql-07) | Identify branches and honor branch-selection policy | H |
| [FR-EQL-08](#fr-eql-08) | Expose bounded multiple-liquid and solid-equilibrium coverage | C |
| [FR-PRP-01](#fr-prp-01) | Evaluate a supplied phase without unrequested redistribution | C |
| [FR-PRP-02](#fr-prp-02) | Distinguish primary and optional property demands | H |
| [FR-PRP-03](#fr-prp-03) | Provide phase transport and phase-pair properties with explicit scope | C |
| [FR-PRP-04](#fr-prp-04) | Name the closure behind an effective multiphase property | C |
| [FR-PRP-05](#fr-prp-05) | Define derivative meaning before computation | H |
| [FR-PRP-06](#fr-prp-06) | Qualify derivative method, conditioning, and nonsmoothness | C |
| [FR-PRP-07](#fr-prp-07) | Keep caloric, standard-state, and chemical-potential conventions explicit | H |
| [FR-CHM-01](#fr-chm-01) | Declare reaction definitions independently of reactor closure | C |
| [FR-CHM-02](#fr-chm-02) | Honor strict specified-conversion feasibility | C |
| [FR-CHM-03](#fr-chm-03) | Resolve chemical equilibrium under declared constraints | C |
| [FR-CHM-04](#fr-chm-04) | Preserve finite-rate and frozen-chemistry assumptions | C |
| [FR-CHM-05](#fr-chm-05) | Reconcile reaction and formation energy exactly once | H |
| [FR-CHM-06](#fr-chm-06) | Map apparent and true species without duplicate inventory | C |
| [FR-CHM-07](#fr-chm-07) | Account explicitly for chemically constrained external exchanges | C |
| [FR-CHM-08](#fr-chm-08) | Verify combined reaction and phase conditions | C |
| [FR-FLW-01](#fr-flw-01) | Expose the unit’s property demands and physical constraints | H |
| [FR-FLW-02](#fr-flw-02) | Preserve mixing and mechanical-splitting semantics | C |
| [FR-FLW-03](#fr-flw-03) | Separate internal phase state from product allocation | C |
| [FR-FLW-04](#fr-flw-04) | Couple disjoint material regions through heat | C |
| [FR-FLW-05](#fr-flw-05) | Translate a material across package boundaries under explicit constraints | C |
| [FR-FLW-06](#fr-flw-06) | Translate representations with declared conservation and information loss | C |
| [FR-FLW-07](#fr-flw-07) | Support equation-oriented participation without requiring symbolic providers | C |
| [FR-FLW-08](#fr-flw-08) | Preserve bulk/interface separation in nonequilibrium contacting | C |
| [FR-RUN-01](#fr-run-01) | Qualify initialization independently of formulation support | H |
| [FR-RUN-02](#fr-run-02) | Preserve the original problem through initialization | H |
| [FR-RUN-03](#fr-run-03) | Bind reusable sessions to compatible configuration revisions | H |
| [FR-RUN-04](#fr-run-04) | Isolate complete calculation sequences across cases | H |
| [FR-RUN-05](#fr-run-05) | Separate same-problem retry from an alternate physical calculation | H |
| [FR-RUN-06](#fr-run-06) | Support cancellation without publishing interrupted trials | H |
| [FR-RUN-07](#fr-run-07) | Contain provider failures and qualify subsequent reuse | H |
| [FR-RUN-08](#fr-run-08) | Record nested convergence independently | H |
| [FR-RES-01](#fr-res-01) | Return structured, distinguishable operation outcomes | H |
| [FR-RES-02](#fr-res-02) | Adopt results only after declared acceptance checks | H |
| [FR-RES-03](#fr-res-03) | Apply conservation and thermal checks on justified bases | H |
| [FR-RES-04](#fr-res-04) | Publish only the authorized result scope | H |
| [FR-RES-05](#fr-res-05) | Protect accepted results from failed or obsolete work | H |
| [FR-RES-06](#fr-res-06) | Expose independent result-quality dimensions | H |
| [FR-RES-07](#fr-res-07) | Retain result lineage and requested-versus-used assumptions | H |
| [FR-RES-08](#fr-res-08) | Distinguish numerical conformance from physical validation | H |
| [FR-LIF-01](#fr-lif-01) | Create a revision and impact assessment for consequential changes | H |
| [FR-LIF-02](#fr-lif-02) | Change a material slate with an explicit quantity/dependency policy | H |
| [FR-LIF-03](#fr-lif-03) | Separate presentation changes from physical and numerical changes | H |
| [FR-LIF-04](#fr-lif-04) | Serialize the semantic model and resolved data | H |
| [FR-LIF-05](#fr-lif-05) | Reconstruct definitions before restoring result authority | H |
| [FR-LIF-06](#fr-lif-06) | Restore snapshots without losing lineage or dependency consistency | H |
| [FR-LIF-07](#fr-lif-07) | Reproduce cases within declared tolerances and manifest limits | H |
| [FR-LIF-08](#fr-lif-08) | Apply the same change semantics through every interface | H |
| [FR-EXT-01](#fr-ext-01) | Represent surface and selective-transfer domains meaningfully | R3 |
| [FR-EXT-02](#fr-ext-02) | Represent distributed material attributes and their reductions | R3 |
| [FR-EXT-03](#fr-ext-03) | Represent inventory-based state specifications without a flow workaround | R3 |
| [FR-EXT-04](#fr-ext-04) | Represent restricted-equilibrium and metastable requests explicitly | R3 |

## 3. Functional requirement cards

Each card’s positive and negative/boundary witnesses are specifications of future tests. None has been executed against an implementation. H witnesses can often start with a mock provider; C numerical witnesses require a pinned real or explicitly synthetic formulation plus the scenario’s process integration. R3 witnesses require a concrete conceptual walkthrough, not an opaque extension field.

### GOV. Coverage claims and usable dependencies

<a id="fr-gov-01"></a>
#### FR-GOV-01: Qualify coverage at the scenario and configuration level

**Applicability:** H. **Accountable owner:** Coverage and dependency qualification. **Initiator:** Model author or coverage reviewer.

**Trigger:** A capability claim is created, queried, or exported.

**Inputs and preconditions:** Scenario identifier; representation, material set, configuration, operation, physical policy, operating envelope, and evidence records.

**Requirement:** The system shall report Representable, Executable, and Validated evidence separately for the specific claimed configuration and scenario.

**Outputs and postconditions:** A bounded claim with evidence provenance and explicit unassessed dimensions; no library-wide promotion from a method name.

**Failure/boundary behavior:** An absent execution or validation record leaves the corresponding claim unassessed, not passed.

**AT-GOV-01-P:** Given only a successful representation walkthrough, querying coverage returns R evidence and no E/V pass.

**AT-GOV-01-N:** Given one executed TP example, requesting an E/V claim for a reactive PH unit is rejected as unsupported by that evidence.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-05, B1:X09, B1:C11, B3:CR-04, B3:W01.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-gov-02"></a>
#### FR-GOV-02: Describe the operation actually exposed by an adapter

**Applicability:** H. **Accountable owner:** Coverage and dependency qualification. **Initiator:** Unit model or provider registrar.

**Trigger:** A provider operation is registered or selected.

**Inputs and preconditions:** Exact provider/build/binding identity; exposed operation; supported material/model/phase combinations; derivative and initialization restrictions.

**Requirement:** The system shall qualify the exposed adapter operation rather than inherit the upstream library catalog as its capability statement.

**Outputs and postconditions:** Discoverable supported, unsupported, and unassessed combinations, including any extra host composition needed.

**Failure/boundary behavior:** Unknown combinations remain unassessed; unsupported combinations do not enter a production solve.

**AT-GOV-02-P:** A mock adapter exposing TP only reports TP and explicitly unavailable PH even when its upstream description mentions PH.

**AT-GOV-02-N:** A solids argument with an explicitly rejecting implementation cannot register a working precipitation capability.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-24, B1:X09, B1:C11, B2:WF-15, B3:CR-04, B3:PV-04, B3:PV-10.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-gov-03"></a>
#### FR-GOV-03: Separate availability from model suitability

**Applicability:** H. **Accountable owner:** Coverage and dependency qualification. **Initiator:** Model author or loader.

**Trigger:** A configuration is prepared or restored.

**Inputs and preconditions:** Required engine, binding, data, optional modules, and recorded access/availability conditions for the chosen profile.

**Requirement:** The system shall report dependency availability independently of thermodynamic suitability and numerical readiness.

**Outputs and postconditions:** A dependency resolution report identifying missing or restricted resources without substituting another model.

**Failure/boundary behavior:** An unavailable required dependency blocks the affected operation but does not prevent inspecting its saved definition.

**AT-GOV-03-P:** A saved model with an absent engine opens for inspection and names the unavailable dependency.

**AT-GOV-03-N:** An available wrapper pointing to an absent external engine does not make the calculation executable or satisfy public-dependency coverage.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X09, B1:X17, B1:C11, B2:WF-01, B2:WF-14, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-gov-04"></a>
#### FR-GOV-04: Admit cohesive packages and qualified submodels

**Applicability:** H. **Accountable owner:** Coverage and dependency qualification. **Initiator:** Provider integrator or model author.

**Trigger:** An implementation is bound to a thermodynamic configuration.

**Inputs and preconditions:** Declared integration boundary: whole configured package, chemical system, phase/property model, data source, or equation contribution.

**Requirement:** The system shall permit different integration granularities while requiring the same applicable observable request, authority, and outcome semantics.

**Outputs and postconditions:** An integration declaration identifying which responsibilities are inside the provider and which are supplied by the host.

**Failure/boundary behavior:** A fragment with no declared completion path cannot be advertised as a complete property-and-equilibrium package.

**AT-GOV-04-P:** A cohesive external flash service and a host-assembled equivalent both satisfy the same mock request contract without exposing identical internals.

**AT-GOV-04-N:** A supplied-K-value phase-allocation utility alone fails qualification as a complete thermodynamic package.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:C05, B1:X09, B2:WF-05, B3:W01, B3:CR-01, B3:CR-04.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-gov-05"></a>
#### FR-GOV-05: Preserve declared scope and explicit coverage gaps

**Applicability:** H. **Accountable owner:** Coverage and dependency qualification. **Initiator:** Coverage reviewer.

**Trigger:** A profile, requirement, or provider assignment is revised.

**Inputs and preconditions:** B1 scenario/profile baseline; proposed change; coverage and dependency evidence.

**Requirement:** The system shall retain an explicit disposition for every in-scope scenario instead of silently dropping scenarios not served by the selected provider.

**Outputs and postconditions:** Versioned scope status distinguishing targeted, deferred by baseline, unresolved, and explicitly revised commitments.

**Failure/boundary behavior:** A provider limitation alone is not approval to demote a P2 scenario or claim a P3 numerical capability.

**AT-GOV-05-P:** Selecting a fluid-only provider leaves SC-21 and SC-31 visible as unresolved coverage obligations.

**AT-GOV-05-N:** A change marking SC-31 out of scope without a recorded scope decision fails the coverage review.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-31, B1:SC-32, B1:C11, B3:W08, B3:CR-16.

**Status:** Implementation not assessed; witnesses not executed.

### MAT. Material identities, representations, and quantities

<a id="fr-mat-01"></a>
#### FR-MAT-01: Resolve identity without conflating labels

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Model author or adapter.

**Trigger:** A compound or material constituent is selected or mapped.

**Inputs and preconditions:** Authoritative identity or characterization record; aliases and provider identifiers; optional registry identifiers.

**Requirement:** The system shall distinguish material identity from display names, provider names, and numerical position.

**Outputs and postconditions:** A resolved identity or an explicit ambiguity; pseudocomponents need no invented registry identity.

**Failure/boundary behavior:** Ambiguous aliases require resolution and cannot silently select the first match.

**AT-MAT-01-P:** Renaming a display label preserves physical identity and all existing balances.

**AT-MAT-01-N:** Two provider records sharing an alias do not merge until an explicit identity mapping is supplied.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-13, B1:SC-15, B1:X13, B1:C01, B2:WF-01, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-02"></a>
#### FR-MAT-02: Declare the material representation and justified operations

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Model author.

**Trigger:** A material description is created or imported.

**Inputs and preconditions:** Representation kind and basis; species/components/lumps; available molecular, assay, charge, or empirical information.

**Requirement:** The system shall record what a material representation means and qualify operations using only the information it actually supplies.

**Outputs and postconditions:** An identifiable molecular, apparent-species, compositional-petroleum, reduced-petroleum, or empirical representation with bounded operation support.

**Failure/boundary behavior:** Unsupported molecular or chemical requests fail specifically without inventing molecular weight, atoms, or critical constants.

**AT-MAT-02-P:** A mass-only empirical material remains eligible for its supplied heat-capacity calculation.

**AT-MAT-02-N:** The same material is refused a molar-fugacity request when no defensible molecular description exists.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-16, B1:SC-31, B1:X01, B1:X09, B1:C01, B3:CR-16, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-03"></a>
#### FR-MAT-03: Keep intensive state, flow, and inventory distinct

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Unit model or model author.

**Trigger:** A quantity is supplied, converted, or requested.

**Inputs and preconditions:** Quantity meaning, units, amount basis, and any molecular data, density, duration, or reference conditions needed for conversion.

**Requirement:** The system shall convert quantities only through a valid declared basis relationship and shall not treat flow, inventory, and intensive state as interchangeable.

**Outputs and postconditions:** Converted values with their bases and required context; unavailable totals remain unavailable.

**Failure/boundary behavior:** Missing conversion information produces a missing-basis diagnostic rather than an arbitrary normalization or time interval.

**AT-MAT-03-P:** A 10 kg inventory and a 2 kg/s stream remain different while mass-to-mole conversion succeeds only when justified.

**AT-MAT-03-N:** A total volume without material amount does not yield density; a flow does not become an inventory without a stated operation.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-01, B1:SC-31, B1:SC-33, B1:X01, B1:X24, B1:C02, B3:W08, B3:CR-16.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-04"></a>
#### FR-MAT-04: Distinguish zero flow, empty material, and missing composition

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Unit model or model author.

**Trigger:** A zero or near-zero amount/flow is supplied.

**Inputs and preconditions:** Explicit total and component quantities; supplied composition, if any; retention convention; empty-inventory status.

**Requirement:** The system shall preserve the semantic difference between zero flow with a specified composition, undefined composition at zero amounts, and an empty inventory.

**Outputs and postconditions:** Zero extensive quantities with qualified retained intensives or explicit undefined values; no fabricated present phase.

**Failure/boundary behavior:** No division by zero or arbitrary equal-fraction composition may establish an accepted physical state.

**AT-MAT-04-P:** Setting flow to zero retains an explicitly specified blend as a reusable specification while reporting zero carried material.

**AT-MAT-04-N:** An empty inventory with no composition cannot be reported as a uniquely resolved fluid state.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-01, B1:SC-33, B1:X03, B1:C02, B2:WF-02, B2:WF-07, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-05"></a>
#### FR-MAT-05: Replace a complete composition explicitly

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Model author.

**Trigger:** The caller requests complete composition replacement.

**Inputs and preconditions:** Target material revision; component identities; basis; supplied values; explicit normalization policy.

**Requirement:** The system shall treat complete replacement as replacing the whole declared composition, with unnamed constituents set to zero under that operation.

**Outputs and postconditions:** A new specification with preserved/derived totals identified and the exact normalization, if authorized, recorded.

**Failure/boundary behavior:** Unknown identities, negative physical fractions, or a missing required normalization choice block acceptance.

**AT-MAT-05-P:** Replacing A/B/C by A=0.2 and B=0.8 yields C=0 for both newly constructed and loaded materials.

**AT-MAT-05-N:** A non-normalized replacement is not silently interpreted as a partial patch or as mole fractions when supplied on a mass basis.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-01, B1:X01, B1:X04, B1:X16, B1:C02, B2:WF-02, B3:W01.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-06"></a>
#### FR-MAT-06: Patch component quantities with a preservation rule

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Model author.

**Trigger:** The caller edits selected component quantities or rescales a total.

**Inputs and preconditions:** Existing quantities; edited constituents; declared preserve-remaining, preserve-total, or rescale policy and required basis.

**Requirement:** The system shall apply component edits according to the explicitly chosen quantity-preservation policy, independently of creation or load history.

**Outputs and postconditions:** Reconciled totals/compositions and an impact record naming the quantities changed.

**Failure/boundary behavior:** An underdetermined fraction patch or conflicting total specification is rejected instead of silently changing unmentioned material.

**AT-MAT-06-P:** For A=2 and B=3 kg/s, patching A to 4 kg/s while preserving B returns total 7 kg/s.

**AT-MAT-06-N:** Requesting the same patch while fixing total at 5 kg/s and B at 3 kg/s reports inconsistency rather than accepting all three.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-01, B1:X01, B1:X02, B1:X16, B1:C02, B2:WF-02, B2:P08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-07"></a>
#### FR-MAT-07: Map composition coordinates consistently

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Provider adapter or equation integration.

**Trigger:** Data cross an identity/order/coordinate boundary.

**Inputs and preconditions:** Ordered source and target identities; amount/composition basis; mapping, including phase and derivative coordinates.

**Requirement:** The system shall transform all positional material data consistently with the declared identity and coordinate mapping.

**Outputs and postconditions:** Mapped values and corresponding parameter, residual, and derivative associations with reversible permutation metadata where applicable.

**Failure/boundary behavior:** Duplicate, missing, or incompatible identities block the affected mapping; coordinates are not inferred from array length.

**AT-MAT-07-P:** Permuting component order and remapping all inputs reproduces the same labeled properties and balances.

**AT-MAT-07-N:** A derivative vector left in the old component order is rejected by the mapping conformance test.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-30, B1:X13, B1:X19, B1:C01, B2:WF-15, B2:P18, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-mat-08"></a>
#### FR-MAT-08: Declare justified conserved quantities

**Applicability:** H. **Accountable owner:** Material definition and quantity semantics. **Initiator:** Chemistry or process model.

**Trigger:** A transformation or balance is configured.

**Inputs and preconditions:** Material representation; stoichiometric or empirical balance description; phases; external exchanges.

**Requirement:** The system shall identify the quantities that the selected representation and physical operation justify conserving.

**Outputs and postconditions:** Nonreactive component balances, reactive mass/element/charge balances where defined, or explicitly limited empirical balances.

**Failure/boundary behavior:** Unknown elemental composition prevents an elemental-closure claim; total molecular moles are not a universal conserved quantity.

**AT-MAT-08-P:** A molecular reaction is checked on elements and mass while permitting a change in total molecular moles.

**AT-MAT-08-N:** An empirical lump without elemental characterization cannot receive a passed carbon-balance result.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-17, B1:SC-21, B1:SC-31, B1:X12, B1:C01, B1:C08, B3:CR-10.

**Status:** Implementation not assessed; witnesses not executed.

### DAT. Resolved data, estimation, and characterization

<a id="fr-dat-01"></a>
#### FR-DAT-01: Resolve parameter values with provenance

**Applicability:** H. **Accountable owner:** Data preparation and characterization. **Initiator:** Model author or preparation coordinator.

**Trigger:** Data are selected for an executable configuration.

**Inputs and preconditions:** Source candidates; identities; method and coefficient conventions; units; versions; overrides; stated validity and uncertainty if available.

**Requirement:** The system shall produce an attributable resolved parameter set with explicit source and override precedence.

**Outputs and postconditions:** Actual selected values, model-specific meaning, source/version, and measured/fitted/estimated/user-specified status; unknown quality remains unknown.

**Failure/boundary behavior:** Conflicting values without a selection policy and incompatible units/conventions prevent readiness.

**AT-DAT-01-P:** An approved user override wins over a database value and both the selected value and override lineage are recoverable.

**AT-DAT-01-N:** A data source with unspecified uncertainty is not automatically labeled validated or high quality.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-08, B1:SC-15, B1:X08, B1:X10, B1:C06, B2:WF-03, B3:CR-07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-dat-02"></a>
#### FR-DAT-02: Interpret missing parameters according to the method

**Applicability:** H. **Accountable owner:** Data preparation and characterization. **Initiator:** Preparation coordinator.

**Trigger:** A required parameter is absent or a combining rule applies.

**Inputs and preconditions:** Parameter kind; selected model; explicit absence semantics and authorized combining rules.

**Requirement:** The system shall distinguish missing, specified-zero, not-applicable, omitted-interaction, and rule-derived parameters.

**Outputs and postconditions:** A method-specific completeness result and any derived parameter with its derivation.

**Failure/boundary behavior:** A generic zero default cannot satisfy an unknown required interaction parameter.

**AT-DAT-02-P:** A permitted pair combining rule yields a derived coefficient while an explicitly supplied zero remains a supplied value.

**AT-DAT-02-N:** Removing an essential activity-model pair parameter produces a data gap rather than silently invoking ideal behavior.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-08, B1:X08, B1:C06, B3:CR-07, B3:L03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-dat-03"></a>
#### FR-DAT-03: Authorize and record estimation or substitution

**Applicability:** H. **Accountable owner:** Data preparation and characterization. **Initiator:** Model author or authorized preparation policy.

**Trigger:** A data gap can be filled only by estimation or an alternate physical description.

**Inputs and preconditions:** Missing values; proposed estimator or substitution; derivation inputs; authorization and applicability policy.

**Requirement:** The system shall create a distinguishable resolved-data revision when an estimation or substitution is accepted.

**Outputs and postconditions:** Qualified values, derivation, authorization, limitations, and requested-versus-used model/data differences.

**Failure/boundary behavior:** Failed estimation leaves the gap unresolved unless a separately authorized substitute is selected; no hidden near-ideal replacement.

**AT-DAT-03-P:** An approved group-contribution estimate is recorded as estimated and changes the resolved-data revision.

**AT-DAT-03-N:** An estimator failure followed by an unapproved ideal substitution cannot produce ordinary same-model success.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-08, B1:X07, B1:X08, B1:C06, B2:WF-03, B3:CR-07, B3:CR-13.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-dat-04"></a>
#### FR-DAT-04: Keep fitted parameter trials separate from approved data

**Applicability:** C. **Accountable owner:** Data preparation and characterization. **Initiator:** Parameter-estimation workflow.

**Trigger:** A parameter fit is requested or a fitted set is promoted.

**Inputs and preconditions:** Observations and their conventions; fit variables/bounds; objective and weighting; initial values; model recipe; promotion policy.

**Requirement:** When fitting is supported, the system shall retain trial parameter values and fit results separately from approved parameter sets.

**Outputs and postconditions:** A candidate fit with objective/residual evidence, fit domain, parameter values, and explicit approval status.

**Failure/boundary behavior:** Fit failure or insufficient information cannot modify the approved set or be presented as independent physical validation.

**AT-DAT-04-P:** A fit changes trial values repeatedly while an unrelated calculation continues to reference its original approved set.

**AT-DAT-04-N:** Cancelling a fit does not leave its last iterate installed as the current production parameter set.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:X08, B1:X16, B1:C06, B2:WF-03, B3:W01, B3:L02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-dat-05"></a>
#### FR-DAT-05: Retain characterization inputs and generated materials

**Applicability:** C. **Accountable owner:** Data preparation and characterization. **Initiator:** Material-characterization workflow.

**Trigger:** An assay or engineering material is characterized.

**Inputs and preconditions:** Original assay and conventions; cuts; characterization methods/options; ancillary measurements; approved estimates.

**Requirement:** For supported characterization, the system shall preserve both the original description and the generated constituent/property definitions with derivation lineage.

**Outputs and postconditions:** A reproducible characterized representation and reconciliation against the stated assay quantities.

**Failure/boundary behavior:** Incomplete assay information or failed reconciliation yields an unresolved/candidate characterization rather than an accepted compositional material.

**AT-DAT-05-P:** Changing cut boundaries creates a new representation revision while preserving the original assay and prior cuts.

**AT-DAT-05-N:** Saving only generated pseudocomponent names without their characterization inputs fails the reconstruction witness.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-15, B1:X08, B1:X17, B1:X21, B1:C06, B2:WF-14, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-dat-06"></a>
#### FR-DAT-06: Prepare and serialize definitions without hidden evaluation effects

**Applicability:** H. **Accountable owner:** Data preparation and characterization. **Initiator:** Preparation, query, or persistence caller.

**Trigger:** A definition is inspected, bound to context, or serialized.

**Inputs and preconditions:** Resolved configuration and data revision; requested read/bind/save action.

**Requirement:** The system shall not change approved physical definitions as an implicit effect of ordinary inspection, context association, or serialization.

**Outputs and postconditions:** The same definition revision before and after the action; internal caches may change only without altering semantic values.

**Failure/boundary behavior:** Preparation needs encountered during a read/save are reported separately instead of silently fitting or replacing parameters.

**AT-DAT-06-P:** Saving a configuration with an unresolved data gap preserves that gap and the same parameter revision.

**AT-DAT-06-N:** A provider context hook that estimates data is isolated or rejected so it cannot mutate the saved approved definition.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X08, B1:X16, B1:X17, B1:C06, B1:C12, B2:WF-03, B2:WF-14, B2:P02, B3:CR-14.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-dat-07"></a>
#### FR-DAT-07: Preserve separate data and model validity information

**Applicability:** H. **Accountable owner:** Data preparation and characterization. **Initiator:** Qualification or results reviewer.

**Trigger:** A request is checked against recorded applicability ranges.

**Inputs and preconditions:** Data-fit domain; model applicability; validation envelope; numerical bounds; unknown-range markers; extrapolation policy.

**Requirement:** The system shall report known domain limits separately and apply the configured extrapolation policy without treating convergence as domain validity.

**Outputs and postconditions:** Per-request qualification with inside/outside/unknown assessments and any approved extrapolation.

**Failure/boundary behavior:** An out-of-policy request is blocked; absent scientific bounds are reported unknown rather than inferred from solver temperature limits.

**AT-DAT-07-P:** A state inside numerical bounds but outside a fitted-parameter interval is labeled extrapolated when policy permits it.

**AT-DAT-07-N:** A converged extrapolated state cannot acquire a within-validation-envelope claim.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-08, B1:SC-16, B1:X10, B1:C11, B3:W01, B3:L04.

**Status:** Implementation not assessed; witnesses not executed.

### CFG. Coherent package configuration

<a id="fr-cfg-01"></a>
#### FR-CFG-01: Identify a coherent configured package

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Model author.

**Trigger:** A package configuration is created or changed.

**Inputs and preconditions:** Material representation; compatible phase/property models; data revisions; caloric and standard-state conventions; physical and numerical policy identifiers.

**Requirement:** The system shall identify a configured thermodynamic package by its consequential method, data, and reference choices independently of a runtime session.

**Outputs and postconditions:** A versioned semantic definition with a separately identified provider binding and numerical policy.

**Failure/boundary behavior:** An EOS/library name alone cannot be treated as a complete reproducible configuration.

**AT-CFG-01-P:** Two packages using the same EOS but different interaction data or caloric contributions have distinct definitions.

**AT-CFG-01-N:** Discarding an ideal caloric contribution while preserving the same complete-package identity fails definition comparison.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:C05, B1:C06, B1:X17, B2:WF-01, B3:CR-01, B3:W01.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-cfg-02"></a>
#### FR-CFG-02: Validate composed-method compatibility

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Preparation coordinator.

**Trigger:** Methods from one or more providers are assembled.

**Inputs and preconditions:** Phase models; standard states; saturation/Henry conventions; references; caloric contributions; algorithm compatibility evidence.

**Requirement:** The system shall require an explicit compatibility treatment for a composed package before qualifying it for the requested operation.

**Outputs and postconditions:** Qualified combinations, identified unassessed combinations, or a specific incompatibility report.

**Failure/boundary behavior:** Matching function signatures or unit labels is insufficient to certify thermodynamic consistency.

**AT-CFG-02-P:** An approved gamma–phi package names the liquid activity standard, vapor treatment, supporting data, and caloric completion.

**AT-CFG-02-N:** An activity-coefficient vector connected to an unrelated enthalpy method without convention reconciliation remains unqualified.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-08, B1:SC-22, B1:X11, B1:C05, B3:CR-01, B3:CR-06, B3:W01.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-cfg-03"></a>
#### FR-CFG-03: Establish operation-specific readiness

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Unit model or request validator.

**Trigger:** A unit requests a particular property or state operation.

**Inputs and preconditions:** Required primary/optional properties; material and data; selected methods; algorithm and initializer coverage; process-boundary needs.

**Requirement:** The system shall report readiness for the requested operation rather than use one package-wide ready flag.

**Outputs and postconditions:** A readiness assessment separating representation, data, method, caloric, transport/derivative, algorithm, initialization, and boundary prerequisites.

**Failure/boundary behavior:** A failed later prerequisite blocks only the affected operation unless it invalidates shared mandatory assumptions.

**AT-CFG-03-P:** A density-capable package remains usable for density while a PH request reports missing caloric completion.

**AT-CFG-03-N:** Successful model construction cannot authorize compressor work when entropy or the actual outlet calculation is unavailable.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-04, B1:SC-31, B1:X09, B1:C11, B3:CR-04, B3:CR-05, B3:CR-16.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-cfg-04"></a>
#### FR-CFG-04: Qualify complete caloric behavior before energy-constrained use

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Unit model or package qualifier.

**Trigger:** Enthalpy, entropy, internal energy, or a related inverse calculation is required.

**Inputs and preconditions:** Applicable ideal/residual/excess/correlation contributions; reference conventions; formation treatment; material representation.

**Requirement:** The system shall verify the caloric contributions required by the selected formulation before accepting an energy- or entropy-constrained operation.

**Outputs and postconditions:** A caloric completeness assessment that does not require every provider to use the same mathematical decomposition.

**Failure/boundary behavior:** Residual-only properties or a generic ideal contribution without justified applicability cannot silently pass as a complete process-energy model.

**AT-CFG-04-P:** A pseudocomponent package passes PVT readiness but fails PH readiness until its missing caloric contribution is supplied and qualified.

**AT-CFG-04-N:** An EOS capable of predicting phase split is not accepted as a validated heater-duty model on that evidence alone.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-02, B1:SC-04, B1:SC-15, B1:SC-22, B1:X11, B1:C07, B3:CR-06, B3:PV-07, B3:PV-08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-cfg-05"></a>
#### FR-CFG-05: Separate physical policy from numerical policy

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Model author or execution coordinator.

**Trigger:** Phase, reaction, model, or solver settings are chosen or changed.

**Inputs and preconditions:** Candidate species/phases; participation restrictions; mixing/reference choices; algorithm, tolerances, guesses, and search controls.

**Requirement:** The system shall identify changes to the physical problem separately from changes to the numerical procedure used to solve it.

**Outputs and postconditions:** Distinct physical and numerical policy records, with heuristics and search restrictions qualified by their actual effects.

**Failure/boundary behavior:** Phase suppression, frozen chemistry, or ideal substitution cannot be recorded only as a convergence-tolerance change.

**AT-CFG-05-P:** Changing a root solver preserves the physical problem identity while selecting frozen chemistry creates a different physical configuration.

**AT-CFG-05-N:** A retry that drops an allowed second liquid cannot retain an unrestricted multiphase success label.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-34, B1:X07, B1:X16, B1:C08, B2:WF-05, B3:CR-13.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-cfg-06"></a>
#### FR-CFG-06: Bind configurations to named material regions and local locations

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Process-model author.

**Trigger:** A package is assigned or inherited within a flowsheet.

**Inputs and preconditions:** Region/location identity; explicit assignment or default rule; package revision; component/species compatibility.

**Requirement:** The system shall make the effective thermodynamic binding at each location discoverable and independent of call order or an ambient current-stream pointer.

**Outputs and postconditions:** Resolved bindings and explicit material-boundary checks when neighboring locations differ.

**Failure/boundary behavior:** An ambiguous inheritance or incompatible assignment blocks the affected calculation rather than choosing the last-used package.

**AT-CFG-06-P:** Two exchanger sides retain separate discoverable package bindings during interleaved calculations.

**AT-CFG-06-N:** Reordering evaluation calls cannot change a stage’s effective package.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-07, B1:SC-28, B1:SC-29, B1:X14, B1:X18, B1:C10, B2:WF-08, B3:W06.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-cfg-07"></a>
#### FR-CFG-07: Declare candidate domains and phase/species eligibility

**Applicability:** H. **Accountable owner:** Thermodynamic configuration qualification. **Initiator:** Model author or equation builder.

**Trigger:** A physical system or an equilibrium problem is prepared.

**Inputs and preconditions:** Candidate phase/material-domain identities; admitted constituents; phase/reaction transfer restrictions; provider numerical limits.

**Requirement:** The system shall distinguish declared physical candidates, structural candidates required by a formulation, and candidates actually examined by a solver.

**Outputs and postconditions:** Eligibility information usable by numerical and equation-oriented paths, with actual assessed phase scope in results.

**Failure/boundary behavior:** A missing required candidate or incompatible provider phase count blocks the requested coverage; enumeration alone does not certify stability.

**AT-CFG-07-P:** A two-liquid problem declares both liquid instances and their admitted constituents independently of product-port labels.

**AT-CFG-07-N:** A two-phase-only adapter cannot accept a required vapor–two-liquid problem by ignoring one candidate.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-10, B1:SC-11, B1:SC-24, B1:X05, B1:X07, B1:C04, B3:CR-08, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

### STA. Local states, specifications, and authority

<a id="fr-sta-01"></a>
#### FR-STA-01: Create a thermodynamic state at any required local location

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Unit model or process formulation.

**Trigger:** A stream, stage, segment, reactor location, or inventory context requests state information.

**Inputs and preconditions:** Location identity; applicable representation/configuration; available quantities and unresolved variables; caller authority.

**Requirement:** The system shall support local thermodynamic descriptions without requiring a graphical flowsheet stream or a fictitious material flow.

**Outputs and postconditions:** Distinct local state contexts sharing definitions where appropriate, with explicit location and amount semantics.

**Failure/boundary behavior:** One local evaluation cannot overwrite another location’s specification through shared mutable context.

**AT-STA-01-P:** Two column stages share a package but retain different temperature/composition specifications and independent results.

**AT-STA-01-N:** Creating an internal inventory state does not require adding a graphical connector or assigning a dummy flow.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-07, B1:SC-26, B1:SC-33, B1:X14, B1:C03, B2:WF-10, B3:CR-02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-02"></a>
#### FR-STA-02: Check specification completeness and consistency

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Unit model or model author.

**Trigger:** A state is submitted for full resolution or retained in a coupled formulation.

**Inputs and preconditions:** Supplied quantities and provenance; declared independent constraints and unknowns; configuration/formulation; requested completion level.

**Requirement:** The system shall distinguish intentionally partial, sufficiently specified, over-specified, inconsistent, and not-yet-assessed state specifications.

**Outputs and postconditions:** A readiness/diagnostic result identifying unresolved or conflicting quantities; redundant compatible observations can be marked as checks rather than extra constraints.

**Failure/boundary behavior:** Dependent inputs do not establish uniqueness; unresolved numerical rank or feasibility is reported unassessed rather than asserted.

**AT-STA-02-P:** A partial stage state remains a legitimate equation-model input but cannot be published as fully resolved.

**AT-STA-02-N:** Supplying mutually inconsistent pressure, temperature, and enthalpy as fixed constraints produces a conflict instead of discarding one silently.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-06, B1:SC-12, B1:X02, B1:C03, B2:WF-04, B3:CR-05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-03"></a>
#### FR-STA-03: Declare calculation authority per quantity family

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Unit model, external component, or request validator.

**Trigger:** An operation is requested or an external state is adopted.

**Inputs and preconditions:** Authoritative intensive coordinates, species totals, phase allocation, material amounts, and external exchanges; permitted unknowns/changes.

**Requirement:** The system shall reject an operation whose permitted state changes conflict with the caller’s declared authoritative quantities.

**Outputs and postconditions:** A calculation-authority record distinguishing root/density resolution, phase redistribution, species transformation, and property evaluation.

**Failure/boundary behavior:** An external allocation cannot be silently re-equilibrated by a property-only request; conflicting authority is an explicit error.

**AT-STA-03-P:** A density request resolves its permitted density coordinate while preserving supplied composition, phase amounts, temperature, and pressure.

**AT-STA-03-N:** A request simultaneously fixing an arbitrary phase split and requiring an incompatible unrestricted flash fails authority validation.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-26, B1:SC-29, B1:X07, B1:X14, B1:C03, B1:C08, B2:WF-15, B3:CR-02, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-04"></a>
#### FR-STA-04: Preserve specified values separately from guesses and derived values

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Initializer or process model.

**Trigger:** A local calculation is initialized, updated, or inspected.

**Inputs and preconditions:** User/process constraints; observations; prior results; numerical guesses; calculated values and their source.

**Requirement:** The system shall retain the role of each value so an initial estimate or previous result cannot silently become an authoritative specification.

**Outputs and postconditions:** Recoverable input intent and separate candidate values; numerical work may revise guesses without changing the physical request.

**Failure/boundary behavior:** A missing specification cannot be filled from a stale prior result without an explicit choice that changes the request.

**AT-STA-04-P:** Changing a column temperature guess leaves the fixed pressure and product specification unchanged.

**AT-STA-04-N:** A stale outlet temperature supplied for initialization is not reported as a user-specified outlet temperature.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-04, B1:SC-07, B1:X02, B1:X06, B1:C03, B2:WF-06, B2:WF-10, B3:W03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-05"></a>
#### FR-STA-05: Identify phase instances and aggregate views without positional meaning

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Adapter or results consumer.

**Trigger:** Phase results are received, compared, or routed.

**Inputs and preconditions:** Phase category/domain; composition and amount; provider labels; matching context; aggregate definitions.

**Requirement:** The system shall distinguish physical phase instances from aggregate reporting views and numerical entries, without treating array order as physical identity.

**Outputs and postconditions:** Physical phase identities or an explicit unresolved correspondence; aggregate mixture/liquid views do not increase physical phase count.

**Failure/boundary behavior:** Near coalescence or ambiguous matching, phase correspondence is reported ambiguous rather than assigning false continuity.

**AT-STA-05-P:** Reversing two liquid output entries preserves the labeled physical solution after valid matching.

**AT-STA-05-N:** An overall-liquid aggregate is not published as a third actual liquid phase.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-10, B1:SC-11, B1:X05, B1:C04, B2:WF-07, B3:CR-08, B3:PV-03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-06"></a>
#### FR-STA-06: Qualify absent and incipient phase information

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Equilibrium or continuation caller.

**Trigger:** A phase amount reaches zero or a provider returns placeholder phases.

**Inputs and preconditions:** Phase amount and tolerances; physical presence decision; incipient/stability/initialization information; output role.

**Requirement:** The system shall label zero-amount incipient and numerical phase information separately from properties and compositions of present physical phases.

**Outputs and postconditions:** Explicit presence and information-role status; optional continuation data may be retained without creating material.

**Failure/boundary behavior:** An absent phase cannot yield an unqualified actual-phase composition or positive product flow from a placeholder.

**AT-STA-06-P:** A zero-amount incipient liquid is retained for continuation while total liquid product flow remains zero.

**AT-STA-06-N:** Counting every slot of a two-entry single-phase provider result as a present phase fails result qualification.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-05, B1:SC-11, B1:X03, B1:X05, B1:C04, B3:CR-08, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-07"></a>
#### FR-STA-07: Keep flow direction separate from physical composition

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Process connection or inventory model.

**Trigger:** A flow changes direction or a time-associated state is represented.

**Inputs and preconditions:** Connection orientation; signed transfer convention; nonnegative material amounts/fractions; location and time context if applicable.

**Requirement:** The system shall associate direction with the transfer convention rather than make physical composition fractions or inventories negative.

**Outputs and postconditions:** Direction-qualified transfer values and meaningful physical states at identified locations/time points.

**Failure/boundary behavior:** Negative accepted physical amounts are invalid; numerical trial coordinates may be nonphysical only when explicitly labeled and confined to the solver.

**AT-STA-07-P:** Reversing a connection changes the signed transfer direction but leaves its material’s valid fractions nonnegative.

**AT-STA-07-N:** A negative component fraction cannot be accepted merely because the connection flow is reversed.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-01, B1:SC-33, B1:X04, B1:X24, B1:C02, B2:WF-11.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-sta-08"></a>
#### FR-STA-08: Define reporting bases and reference conditions

**Applicability:** H. **Accountable owner:** State/request interpretation. **Initiator:** Reporting consumer or model author.

**Trigger:** A standard-volume, wet/dry, molality, or phase-fraction quantity is requested.

**Inputs and preconditions:** Observable definition; amount basis and denominator; reference temperature/pressure; carrier/solvent identity; conversion convention.

**Requirement:** The system shall return a reporting quantity only with the basis and reference conditions needed to interpret it.

**Outputs and postconditions:** A labeled derived quantity traceable to physical amounts and the declared convention.

**Failure/boundary behavior:** Missing standard conditions or denominator produce an incomplete-request diagnostic instead of assuming a universal standard.

**AT-STA-08-P:** Wet-to-dry gas conversion changes the reported ratio without changing actual water or carrier amounts.

**AT-STA-08-N:** A vapor-fraction number lacking mass/mole basis cannot silently determine a complete two-phase state.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-09, B1:SC-14, B1:SC-16, B1:X01, B1:X21, B1:C02, B3:W03.

**Status:** Implementation not assessed; witnesses not executed.

### EQL. State resolution and phase equilibrium

<a id="fr-eql-01"></a>
#### FR-EQL-01: Resolve a supported TP equilibrium problem

**Applicability:** C. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Unit model or state caller.

**Trigger:** A fully qualified temperature–pressure equilibrium request is submitted.

**Inputs and preconditions:** Temperature, pressure, material quantities/composition, admitted phases/species, fixed-or-reactive chemistry policy, and required outputs.

**Requirement:** For a supported TP profile, the system shall produce a candidate equilibrium satisfying the declared state and conservation constraints or a qualified unsuccessful outcome.

**Outputs and postconditions:** Resolved phase amounts/compositions and assessed equilibrium scope, with specified and calculated quantities distinguished.

**Failure/boundary behavior:** Unallowed phase/species changes, underdetermination, nonconvergence, and unsupported configurations remain distinct.

**AT-EQL-01-P:** A pinned nonreactive TP fixture reconstructs every feed component from the returned phases within its predeclared tolerances.

**AT-EQL-01-N:** An adapter restricted to VLE cannot mark a requested required-VLLE fixture solved by omitting the other liquid.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-05, B1:SC-11, B1:X07, B1:X12, B1:C08, B2:WF-04, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-02"></a>
#### FR-EQL-02: Resolve PH with a verified enthalpy target

**Applicability:** C. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Valve, heater, or energy-balance model.

**Trigger:** The process model establishes a pressure–enthalpy outlet request.

**Inputs and preconditions:** Pressure; enthalpy target and basis/reference; material; chemistry policy; phase policy; caloric completeness; admissible initial data.

**Requirement:** For a supported PH profile, the system shall resolve the outlet and verify its calculated enthalpy against the original target before accepting state-resolution success.

**Outputs and postconditions:** Temperature, phase/species allocation permitted by the request, target residual, conservation checks, and used configuration.

**Failure/boundary behavior:** A solver success flag with an unacceptable target residual yields CHECK_FAILED, not successful resolution.

**AT-EQL-02-P:** A forward state evaluated to H and then resolved at the same P,H recovers a compatible branch and closes the enthalpy residual.

**AT-EQL-02-N:** A mock solver that stops at a temperature bound with the wrong enthalpy is rejected while the prior accepted state is preserved.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-02, B1:SC-06, B1:X02, B1:X11, B1:X23, B1:C07, B2:WF-06, B3:W03, B3:PV-11.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-03"></a>
#### FR-EQL-03: Resolve PS and distinguish reference from actual equipment state

**Applicability:** C. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Compressor or expander model.

**Trigger:** A pressure–entropy reference calculation is requested.

**Inputs and preconditions:** Pressure; entropy target and basis/reference; composition/amount; phase and chemistry policy; required caloric data.

**Requirement:** For a supported PS profile, the system shall return a verified pressure–entropy state separately from any actual equipment outlet calculated using an efficiency or work relation.

**Outputs and postconditions:** Reference-state identity and entropy residual; actual outlet, if calculated by the unit, has separate constraints and result identity.

**Failure/boundary behavior:** No entropy support means the reference operation is unsupported; a TP capability does not substitute for it.

**AT-EQL-03-P:** A compressor fixture retains its ideal-reference PS state and actual energy-constrained state as distinct results.

**AT-EQL-03-N:** An actual outlet with a different entropy cannot be reported as the solved isentropic reference state.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-04, B1:SC-13, B1:X09, B1:X11, B1:C03, B1:C07, B3:W03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-04"></a>
#### FR-EQL-04: Distinguish saturation endpoints from complete phase allocation

**Applicability:** C. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Utility/refrigeration unit or property caller.

**Trigger:** A bubble/dew, saturation, or phase-fraction request is submitted.

**Inputs and preconditions:** Pure/pseudo-pure/mixture representation; independent specifications; selected endpoint/phase; quality basis and denominator.

**Requirement:** For a supported saturation profile, the system shall distinguish endpoint properties, a fully allocated state, and a non-unique allocation from dependent inputs.

**Outputs and postconditions:** Endpoint or allocated-state result with phase-composition and fraction semantics appropriate to the representation.

**Failure/boundary behavior:** Pure-fluid saturation pressure and temperature alone do not justify an invented intermediate vapor fraction.

**AT-EQL-04-P:** A pure-water P,quality fixture with stated mass basis resolves while an endpoint query reports only its promised endpoint properties.

**AT-EQL-04-N:** A saturated pure-fluid TP request for a complete allocation reports underdetermination unless additional valid information is supplied.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-12, B1:SC-13, B1:SC-14, B1:X02, B1:C03, B3:W03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-05"></a>
#### FR-EQL-05: Return only the promised post-equilibrium properties

**Applicability:** H. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** State caller or unit model.

**Trigger:** Equilibrium resolution completes and derived properties are consumed.

**Inputs and preconditions:** Operation postconditions; requested primary and optional properties; provider result availability.

**Requirement:** The system shall distinguish determined phase allocation from individually evaluated derived properties after equilibrium.

**Outputs and postconditions:** Explicit availability for each promised/requested property; additional requests occur only as required by the caller’s contract.

**Failure/boundary behavior:** A missing enthalpy or diffusivity cannot be returned as zero or marked available merely because equilibrium converged.

**AT-EQL-05-P:** A successful phase split lacking viscosity remains a valid allocation result while viscosity is requested separately or reported unavailable.

**AT-EQL-05-N:** A heater requiring enthalpy cannot be completed using a phase-split-only result.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-02, B1:SC-26, B1:X09, B1:X23, B1:C11, B3:CR-03, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-06"></a>
#### FR-EQL-06: Qualify stability and searched phase scope

**Applicability:** H. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Equilibrium caller or acceptance reviewer.

**Trigger:** A result carries a stability or equilibrium claim.

**Inputs and preconditions:** Admitted candidates, actually searched candidates, assessment method, restrictions, and returned diagnostics.

**Requirement:** The system shall report the scope and strength of stability evidence without upgrading restricted or unchecked results to unrestricted stable equilibrium.

**Outputs and postconditions:** Assessed-stable, unstable, not-assessed, or indeterminate status relative to a specified candidate set.

**Failure/boundary behavior:** Unmet required stability checks prevent the requested acceptance level, while lower-qualified diagnostic results may remain inspectable.

**AT-EQL-06-P:** A result obtained with solids suppressed is labeled restricted and records that solid stability was not assessed.

**AT-EQL-06-N:** A heuristic phase choice or local numerical convergence alone cannot establish a global-minimum certificate.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-05, B1:SC-24, B1:SC-34, B1:X06, B1:X07, B1:C08, B2:WF-05, B3:CR-08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-07"></a>
#### FR-EQL-07: Identify branches and honor branch-selection policy

**Applicability:** H. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Initializer, continuation caller, or result reviewer.

**Trigger:** Multiple admissible roots/phase solutions or differing starts are encountered.

**Inputs and preconditions:** Branch hints; prior qualified state; search policy; root/candidate diagnostics; evidence of ambiguity.

**Requirement:** The system shall preserve branch-selection information and report unresolved ambiguity rather than claim universal uniqueness.

**Outputs and postconditions:** Selected branch and rationale, or a set/description of unresolved candidates; repeated-run comparison uses the same declared policy.

**Failure/boundary behavior:** A prior result may be a guess but cannot override newly required stability or physical constraints.

**AT-EQL-07-P:** Two admissible branch candidates under an explicit continuation rule yield a recorded selection rather than an unlabeled swap.

**AT-EQL-07-N:** Different initial guesses producing different admissible outcomes cannot be silently merged into one unique-state claim.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-06, B1:SC-08, B1:X05, B1:X06, B1:C04, B2:WF-10, B3:CR-05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-eql-08"></a>
#### FR-EQL-08: Expose bounded multiple-liquid and solid-equilibrium coverage

**Applicability:** C. **Accountable owner:** Phase-state and equilibrium resolution. **Initiator:** Separator or crystallization model.

**Trigger:** A multi-liquid or solid-containing equilibrium operation is requested.

**Inputs and preconditions:** Actual phase and solid-form candidates; compatible parameters; supported specification pair and initializer; material balances.

**Requirement:** For a claimed multiphase/solid profile, the system shall solve the shared permitted allocation problem or an explicitly qualified equivalent procedure.

**Outputs and postconditions:** Jointly consistent phase/species amounts and target residuals for the declared candidates, not unrelated independent binary answers.

**Failure/boundary behavior:** A pure-solid method, a constructor argument, or an N-phase label cannot stand in for a supported multicomponent precipitation operation.

**AT-EQL-08-P:** A three-phase fixture closes one shared component balance and applicable interphase conditions across all products.

**AT-EQL-08-N:** Separately successful flashes that double-count a component fail joint allocation acceptance.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-10, B1:SC-11, B1:SC-24, B1:X05, B1:X12, B1:C04, B3:W04, B3:PV-04.

**Status:** Implementation not assessed; witnesses not executed.

### PRP. Property evaluation, transport, and derivatives

<a id="fr-prp-01"></a>
#### FR-PRP-01: Evaluate a supplied phase without unrequested redistribution

**Applicability:** C. **Accountable owner:** Property and derivative evaluation. **Initiator:** Unit model or external component.

**Trigger:** Properties of an already specified phase are requested.

**Inputs and preconditions:** Phase state/model; composition; authoritative allocation; allowed density/root unknowns; requested property set.

**Requirement:** For a supported phase-property profile, the system shall evaluate the requested properties without changing authoritative phase or species quantities.

**Outputs and postconditions:** Values tied to the supplied phase and declared property convention, with any permitted root resolution identified.

**Failure/boundary behavior:** An incompatible phase state or unsupported property produces a targeted diagnostic rather than a substitute bulk flash.

**AT-PRP-01-P:** A density/enthalpy call on two separately specified bulk phases preserves their original distinct compositions.

**AT-PRP-01-N:** A provider that re-equilibrates the whole material during a phase-only call fails the authority test.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-07, B1:SC-26, B1:X07, B1:X14, B1:C09, B3:CR-02, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-prp-02"></a>
#### FR-PRP-02: Distinguish primary and optional property demands

**Applicability:** H. **Accountable owner:** Property and derivative evaluation. **Initiator:** Unit model or report caller.

**Trigger:** A set of properties is requested.

**Inputs and preconditions:** Per-property observable, state/domain, basis, and required-versus-optional role for this operation.

**Requirement:** The system shall report success and failure at the requested-property level and shall not let an optional-property failure masquerade as failure or success of an unrelated primary calculation.

**Outputs and postconditions:** Available values and targeted omissions; primary operation status reflects only its actual necessary conditions.

**Failure/boundary behavior:** A property necessary to a unit balance cannot be reclassified optional after failure to obtain a successful result.

**AT-PRP-02-P:** Failing an optional speed-of-sound report leaves an already accepted primary enthalpy calculation intact.

**AT-PRP-02-N:** Missing liquid enthalpy blocks an energy balance even when density and vapor fraction are available.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-02, B1:X09, B1:X23, B1:C11, B2:WF-08, B2:P17, B3:CR-03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-prp-03"></a>
#### FR-PRP-03: Provide phase transport and phase-pair properties with explicit scope

**Applicability:** C. **Accountable owner:** Property and derivative evaluation. **Initiator:** Rating or contacting model.

**Trigger:** Viscosity, conductivity, diffusivity, or interfacial properties are needed.

**Inputs and preconditions:** Specific phase or phase pair; observable/transport convention; local state and compatible transport parameters.

**Requirement:** For supported transport profiles, the system shall identify the phase/domain and transport convention for each returned property.

**Outputs and postconditions:** Intrinsic properties qualified by method/data and pair identity, separate from geometry-dependent coefficients.

**Failure/boundary behavior:** Missing diffusivity data block the relevant rate-based request rather than being replaced by an arbitrary transfer coefficient.

**AT-PRP-03-P:** An interfacial-tension request explicitly distinguishes the two liquid instances from a liquid–vapor pair.

**AT-PRP-03-N:** A viscosity value is not returned as a heat-transfer coefficient just because the unit needs one.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-03, B1:SC-10, B1:SC-26, B1:X09, B1:X22, B1:C09, B3:CR-11.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-prp-04"></a>
#### FR-PRP-04: Name the closure behind an effective multiphase property

**Applicability:** C. **Accountable owner:** Property and derivative evaluation. **Initiator:** Unit model or reporting consumer.

**Trigger:** An aggregate viscosity, conductivity, density, or heat capacity is requested.

**Inputs and preconditions:** Contributing phase properties/amounts; declared aggregation or physical closure; intended use; additional required material information.

**Requirement:** For a supported effective-property profile, the system shall associate the result with its explicit closure and inputs rather than present it as a unique consequence of equilibrium.

**Outputs and postconditions:** A closure-qualified bulk value and applicability; ordinary additive balance quantities retain their correct algebraic meaning.

**Failure/boundary behavior:** No appropriate closure means the effective property is unavailable even when constituent phase properties exist.

**AT-PRP-04-P:** Two approved slurry-viscosity closures can yield different named results from the same phase state without changing its equilibrium allocation.

**AT-PRP-04-N:** A requested equilibrium heat-capacity derivative is not replaced silently by a phase-weighted frozen-allocation heat capacity.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-23, B1:X22, B1:C09, B2:WF-07, B3:CR-11, B3:L05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-prp-05"></a>
#### FR-PRP-05: Define derivative meaning before computation

**Applicability:** H. **Accountable owner:** Property and derivative evaluation. **Initiator:** Equation solver or sensitivity caller.

**Trigger:** Any property derivative or solved-state sensitivity is requested.

**Inputs and preconditions:** Output and input observables; independent composition coordinates; units/bases; held-fixed constraints; phase/chemical response; derivative order.

**Requirement:** The system shall distinguish local phase derivatives, path derivatives, parameter derivatives, and sensitivities of a re-solved constrained state.

**Outputs and postconditions:** A fully specified derivative request and operation-specific supported/unsupported/unassessed status.

**Failure/boundary behavior:** An ambiguous derivative symbol or a generic automatic-differentiation flag cannot satisfy an incompletely specified request.

**AT-PRP-05-P:** At fixed composition, a single-phase temperature derivative is stored separately from a temperature sensitivity after re-equilibration.

**AT-PRP-05-N:** A second derivative or chemical-equilibrium sensitivity is not inferred from a supported first phase-property derivative.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-04, B1:SC-07, B1:X19, B1:C11, B3:CR-15, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-prp-06"></a>
#### FR-PRP-06: Qualify derivative method, conditioning, and nonsmoothness

**Applicability:** C. **Accountable owner:** Property and derivative evaluation. **Initiator:** Equation solver or sensitivity caller.

**Trigger:** A supported derivative is evaluated.

**Inputs and preconditions:** Complete derivative request; state; selected method and perturbation policy if numerical; phase/reaction branch and diagnostic evidence.

**Requirement:** For a claimed derivative profile, the system shall return the derivative together with method and relevant validity or conditioning qualifications.

**Outputs and postconditions:** Analytic, automatic, implicit, or finite-difference method; active branch; known limitations and successful/failed verification evidence.

**Failure/boundary behavior:** At unassessed transitions or ill-conditioned solved states, return a qualified/unsupported outcome instead of an unverified smooth derivative.

**AT-PRP-06-P:** Away from transitions, a derivative fixture matches an independent perturbation check with the same held-fixed quantities and declared tolerance.

**AT-PRP-06-N:** A perturbation crossing a phase boundary cannot be reported as a verified homogeneous derivative without qualification.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:X05, B1:X19, B1:C11, B3:CR-15, B3:PV-14.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-prp-07"></a>
#### FR-PRP-07: Keep caloric, standard-state, and chemical-potential conventions explicit

**Applicability:** H. **Accountable owner:** Property and derivative evaluation. **Initiator:** Property caller or package integrator.

**Trigger:** A caloric or chemical-equilibrium observable crosses a component boundary.

**Inputs and preconditions:** Enthalpy/entropy references; chemical-potential/activity/fugacity standard states; basis; formation-energy accounting and permitted transformations.

**Requirement:** The system shall preserve the conventions required to combine property values in phase, chemical, and energy relationships.

**Outputs and postconditions:** Convention-qualified quantities and explicit compatible conversion, or an incompatibility diagnosis.

**Failure/boundary behavior:** Equal units do not establish equal reference conventions; unavailable offsets cannot be fabricated.

**AT-PRP-07-P:** A known constant reference shift is applied consistently to both sides of a compatible nonreactive balance.

**AT-PRP-07-N:** Values from incompatible standards are not compared as chemical-equilibrium residuals merely because both use joules per mole.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-08, B1:SC-17, B1:SC-29, B1:X11, B1:C07, B3:CR-06, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

### CHM. Reactions, speciation, and chemical energy

<a id="fr-chm-01"></a>
#### FR-CHM-01: Declare reaction definitions independently of reactor closure

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Reaction/process-model author.

**Trigger:** A conversion, equilibrium, or kinetic reaction is configured.

**Inputs and preconditions:** Species identities; stoichiometry; reaction mode; phase participation; activity/concentration/pressure basis; rate or equilibrium expression; units.

**Requirement:** For supported chemistry, the system shall preserve reaction-definition semantics separately from residence time, geometry, and other equipment assumptions.

**Outputs and postconditions:** A reaction definition with justified balances, units, participation, and required property inputs.

**Failure/boundary behavior:** Unbalanced declared chemistry or incompatible rate/equilibrium units block the corresponding reaction calculation.

**AT-CHM-01-P:** Changing reactor residence time leaves the referenced reaction definition and property-package identity unchanged.

**AT-CHM-01-N:** A kinetic expression with an incompatible concentration basis cannot be used silently with mole-fraction inputs.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-17, B1:SC-18, B1:SC-19, B1:X12, B1:C08, B2:WF-09, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-02"></a>
#### FR-CHM-02: Honor strict specified-conversion feasibility

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Conversion reactor model.

**Trigger:** A fixed conversion or extent is imposed.

**Inputs and preconditions:** Feed amounts; balanced stoichiometry; conversion/extent basis; simultaneous/sequential reaction policy; thermal constraints.

**Requirement:** For a strict specified-conversion request, the system shall reject infeasible consumption instead of clipping negative quantities or silently limiting the requested conversion.

**Outputs and postconditions:** Achieved amounts satisfying the actual specified conversion and relevant balances, or an infeasibility result with the conflicting quantities.

**Failure/boundary behavior:** An explicitly authorized limiting/optimization alternative is a different problem and must not pass the original strict-conversion fixture.

**AT-CHM-02-P:** A feasible synthetic A→B extent of 0.4 from 1.0 amount of A returns 0.6 A and 0.4 B on a stated compatible basis.

**AT-CHM-02-N:** An extent of 1.2 with only 1.0 A yields infeasibility, not a clipped or silently optimized successful conversion.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-17, B1:X04, B1:X12, B1:C08, B2:WF-09, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-03"></a>
#### FR-CHM-03: Resolve chemical equilibrium under declared constraints

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Equilibrium reactor or speciation caller.

**Trigger:** Species transformation to chemical equilibrium is requested.

**Inputs and preconditions:** Coherent chemical system; admitted species/phases; justified conserved totals; equilibrium equations/potentials; thermal conditions; restrictions.

**Requirement:** For a supported chemical-equilibrium profile, the system shall resolve species amounts under the declared chemical and phase constraints and verify the applicable balances.

**Outputs and postconditions:** Species/phase results, equilibrium scope, thermal target checks, and conserved-quantity residuals.

**Failure/boundary behavior:** A fixed-species phase flash does not satisfy chemical-equilibrium capability; absent required data remain a readiness failure.

**AT-CHM-03-P:** A supported equilibrium fixture changes individual species amounts while preserving closed-system conserved quantities.

**AT-CHM-03-N:** Adding an excluded species during a solver retry cannot be reported as the same constrained chemical problem.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-18, B1:SC-24, B1:X07, B1:X12, B1:C08, B3:CR-10, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-04"></a>
#### FR-CHM-04: Preserve finite-rate and frozen-chemistry assumptions

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Kinetic reactor or coupled chemistry model.

**Trigger:** A rate or mixed kinetic/equilibrium subproblem is evaluated.

**Inputs and preconditions:** Rate laws and units; local state; kinetic/frozen/equilibrated species sets; unit-owned time/residence/transport context.

**Requirement:** For supported kinetic coupling, the system shall supply or evaluate only the authorized reaction/property relationships while retaining declared finite-rate and frozen-species restrictions.

**Outputs and postconditions:** Rates and qualified chemical/property outputs with participation rules; process evolution remains owned by the declared unit/time model.

**Failure/boundary behavior:** No supported kinetics means the requested kinetics is unavailable, not replaced by full equilibrium.

**AT-CHM-04-P:** A mixed kinetic-mineral/equilibrated-aqueous fixture retains the mineral’s rate constraint while solving the allowed aqueous subset.

**AT-CHM-04-N:** A rate request that unexpectedly consumes frozen species fails the authority and conservation checks.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-19, B1:SC-25, B1:SC-34, B1:X07, B1:X14, B1:C08, B3:W05, B3:PV-18.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-05"></a>
#### FR-CHM-05: Reconcile reaction and formation energy exactly once

**Applicability:** H. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Reaction/energy-model integrator.

**Trigger:** Reaction and material caloric contributions enter an energy balance.

**Inputs and preconditions:** Material enthalpy convention; formation contributions already included; reaction-energy correction; representation maps; heat/work/exchange definitions.

**Requirement:** The system shall qualify and record an energy-accounting convention that prevents omission or double counting of reaction and formation contributions.

**Outputs and postconditions:** An interpretable accounting statement showing included material energy and any necessary reaction correction.

**Failure/boundary behavior:** Unknown convention compatibility blocks a claimed energy-balanced reactive result even when composition equilibrium succeeds.

**AT-CHM-05-P:** Two equivalent synthetic formulations, one using formation-inclusive enthalpy and one using a corresponding correction, give the same physical duty.

**AT-CHM-05-N:** Adding the full reaction heat again to already formation-inclusive enthalpy fails the energy-accounting witness.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-17, B1:SC-18, B1:SC-22, B1:X11, B1:X12, B1:C07, B3:CR-06, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-06"></a>
#### FR-CHM-06: Map apparent and true species without duplicate inventory

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Electrolyte feed/reporting or chemistry caller.

**Trigger:** A reporting composition is related to the internal species description.

**Inputs and preconditions:** Apparent/true representation definitions; conversion constraints; elemental/charge information; activity/pH convention; thermal data when required.

**Requirement:** For a supported electrolyte profile, the system shall reconcile reporting components with internal species while maintaining one physical material account.

**Outputs and postconditions:** Speciation and reporting values with explicit mapping and conventions; balances include charge where the model imposes it.

**Failure/boundary behavior:** Inconsistent charge constraints or an unsupported reverse mapping are reported, not repaired by unexplained normalization.

**AT-CHM-06-P:** Converting an apparent salt feed to ions preserves its justified mass/element totals without summing both representations as separate material.

**AT-CHM-06-N:** A report that adds apparent salt mass to already represented ionic mass fails duplicate-inventory checks.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-21, B1:SC-30, B1:X12, B1:X13, B1:C01, B3:CR-10, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-07"></a>
#### FR-CHM-07: Account explicitly for chemically constrained external exchanges

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Open-system equilibrium or contacting model.

**Trigger:** A chemical constraint requires matter or energy exchange with a reservoir.

**Inputs and preconditions:** Reservoir/titrant identity; permitted exchange; amount and energy convention; imposed pH/fugacity/other constraint; closed/open boundary.

**Requirement:** For supported open chemistry, the system shall include every modeled exchange required to satisfy the constraint in the result and process balance.

**Outputs and postconditions:** Signed exchanged quantities, reservoir identities, applicable energy contribution, and combined conservation residuals.

**Failure/boundary behavior:** A constraint requiring an undeclared source/sink cannot be solved by silently adding material.

**AT-CHM-07-P:** A pH-controlled fixture reports the titrant amount and includes it in downstream conserved quantities and the applicable energy balance.

**AT-CHM-07-N:** A closed-system request cannot satisfy a reservoir fugacity constraint by hidden gas addition.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-18, B1:SC-22, B1:X20, B1:C08, B3:CR-10, B3:PV-18.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-chm-08"></a>
#### FR-CHM-08: Verify combined reaction and phase conditions

**Applicability:** C. **Accountable owner:** Chemistry and reaction-property coordination. **Initiator:** Reactive separation or reactive-solids unit.

**Trigger:** Phase transfer and chemical transformation are solved at one process location.

**Inputs and preconditions:** Compatible models/data; reaction mode; admitted phases/solid forms; coupling formulation; thermal constraints; acceptance criteria.

**Requirement:** For a claimed reactive multiphase profile, the system shall verify the combined reaction, phase, thermal, and conservation conditions under the stated coupling approximation.

**Outputs and postconditions:** A coupled or explicitly approximate result with residuals for all required conditions and actual restrictions.

**Failure/boundary behavior:** Sequential substeps that invalidate each other cannot be labeled fully coupled equilibrium.

**AT-CHM-08-P:** A reactive-separation fixture closes stage chemistry, phase conditions, and energy together rather than only each standalone call.

**AT-CHM-08-N:** A chemically equilibrated composition that becomes chemically inconsistent after a separate flash fails coupled-result acceptance.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-20, B1:SC-22, B1:SC-24, B1:SC-25, B1:X07, B1:X11, B1:C08, B3:W05.

**Status:** Implementation not assessed; witnesses not executed.

### FLW. Participation in unit models and flowsheets

<a id="fr-flw-01"></a>
#### FR-FLW-01: Expose the unit’s property demands and physical constraints

**Applicability:** H. **Accountable owner:** Process-model integration. **Initiator:** Unit-model author or process formulation.

**Trigger:** A unit is prepared to calculate.

**Inputs and preconditions:** Selected unit formulation and operating mode; feed/internal/output locations; balance constraints; primary and optional properties.

**Requirement:** The system shall expose the thermodynamic requirements of the selected unit formulation without assigning equipment geometry or process operating decisions to the property package.

**Outputs and postconditions:** A unit demand/constraint declaration usable for readiness checking and local request preparation.

**Failure/boundary behavior:** Missing mandatory properties identify the affected unit/location; a different unit formulation requires explicit selection.

**AT-FLW-01-P:** A duty-only heater requires appropriate enthalpy while its rating variant additionally requests transport properties.

**AT-FLW-01-N:** Missing rating viscosity cannot be hidden by claiming the full rating mode succeeded as a duty-only calculation.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-02, B1:SC-03, B1:X09, B1:X14, B1:C09, B2:WF-08, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-02"></a>
#### FR-FLW-02: Preserve mixing and mechanical-splitting semantics

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Mixer or splitter model.

**Trigger:** Compatible feeds are combined or a composition-preserving split is requested.

**Inputs and preconditions:** Feed quantities/states; compatible references and representations; unit pressure/heat rules; split fractions.

**Requirement:** For the conventional mixing/splitting profile, the system shall support the selected material and energy balances without confusing mechanical splitting with equilibrium separation.

**Outputs and postconditions:** Mixed-state requests and product quantities consistent with the declared unit rules.

**Failure/boundary behavior:** Incompatible representations or energy conventions require boundary treatment before mixing; invalid split fractions fail.

**AT-FLW-02-P:** A mechanical split preserves composition and intensive state, and recombining its products recovers the original material balance.

**AT-FLW-02-N:** A mixer cannot replace its energy balance by an unqualified arithmetic temperature average.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-01, B1:X01, B1:X11, B1:C02, B2:WF-07, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-03"></a>
#### FR-FLW-03: Separate internal phase state from product allocation

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Separator or solids-handling model.

**Trigger:** An evaluated internal material is routed to unit outlets.

**Inputs and preconditions:** Internal phases and quantities; product-port roles; separation/carryover/entrainment rules; empty-product policy.

**Requirement:** For a supported separation profile, the system shall apply named product-routing assumptions independently of phase identity and equilibrium determination.

**Outputs and postconditions:** Product quantities/compositions and routing lineage with material/energy reconciliation.

**Failure/boundary behavior:** Absent phases and unspecified solids routing cannot acquire arbitrary positive outlet flows.

**AT-FLW-03-P:** Changing a permitted solids-carryover rule changes products while leaving the referenced internal equilibrium state identifiable.

**AT-FLW-03-N:** Swapping provider liquid order must not silently swap light/heavy product intent or phase identity.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-05, B1:SC-10, B1:SC-23, B1:X05, B1:X22, B1:C04, B2:WF-07, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-04"></a>
#### FR-FLW-04: Couple disjoint material regions through heat

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Heat-exchanger or utility model.

**Trigger:** Heat crosses between two independently modeled materials.

**Inputs and preconditions:** Separate side configurations/states/flows; unit heat/work/loss convention; required within-side caloric properties.

**Requirement:** For heat-only coupling, the system shall reconcile side energy changes without requiring material translation or equality of absolute enthalpy zeroes.

**Outputs and postconditions:** Side-specific outlet calculations and a common heat-balance residual with no compound transfer across the wall.

**Failure/boundary behavior:** A side’s missing necessary caloric property blocks the coupled balance; optional bounding/report failures remain separately qualified.

**AT-FLW-04-P:** A known constant reference shift applied consistently to one nonreacting side does not alter predicted exchanged heat.

**AT-FLW-04-N:** A heat-only connection cannot transfer species or invoke an automatic composition translator.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-02, B1:SC-28, B1:X11, B1:C10, B2:WF-08, B3:CR-09, B3:W06.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-05"></a>
#### FR-FLW-05: Translate a material across package boundaries under explicit constraints

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Process-model author or boundary component.

**Trigger:** The same physical material crosses to a different thermodynamic description.

**Inputs and preconditions:** Source/target configurations; identity map; known reference conversion; chosen preserved quantities; model-discrepancy policy.

**Requirement:** For supported package translation, the system shall preserve the declared boundary constraints and distinguish a reference offset from disagreement between property models.

**Outputs and postconditions:** Mapped material, target state, reference transformation, residual model discrepancy, and any explicitly physical heat/work exchange.

**Failure/boundary behavior:** Incompatible simultaneous constraints or unknown reference relations block translation or yield a clearly qualified incomplete result, not hidden heat.

**AT-FLW-05-P:** Holding T,P,composition across a synthetic package boundary reports the remaining enthalpy-model difference after the known reference correction.

**AT-FLW-05-N:** Holding the same T,P,composition and incompatible corrected H cannot pass by inserting an undeclared heat duty.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-29, B1:X11, B1:X13, B1:C10, B2:WF-15, B3:CR-09, B3:PV-19.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-06"></a>
#### FR-FLW-06: Translate representations with declared conservation and information loss

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Representation boundary or reporting caller.

**Trigger:** Detailed, lumped, apparent, or true-species descriptions are mapped.

**Inputs and preconditions:** Source/target representations; mapping; justified conserved quantities; reversible/aggregating character; reconstruction assumptions.

**Requirement:** For supported representation translation, the system shall state what is conserved, recalculated, and lost, and shall not invent a unique inverse for an irreversible mapping.

**Outputs and postconditions:** Mapped quantities and conservation checks with information-loss and lineage records.

**Failure/boundary behavior:** Missing information for reverse reconstruction requires explicit additional assumptions or rejection.

**AT-FLW-06-P:** Combining two detailed constituents into one lump preserves their declared total mass and records the lost split.

**AT-FLW-06-N:** Unlumping that result without extra information cannot return a unique detailed composition as observed fact.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-15, B1:SC-21, B1:SC-30, B1:X13, B1:C01, B1:C10, B3:CR-09, B3:W06.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-07"></a>
#### FR-FLW-07: Support equation-oriented participation without requiring symbolic providers

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Equation compiler or coupled unit solver.

**Trigger:** A thermodynamic definition participates in a coupled process formulation.

**Inputs and preconditions:** Shared definitions; local variables/constraints; property requirements; external callback or equation contribution; initialization and derivative needs.

**Requirement:** For a claimed coupled formulation, the system shall preserve the same physical request semantics while exposing only the equation, residual, or callback capabilities actually supported.

**Outputs and postconditions:** A discoverable contribution contract identifying unknowns, fixed constraints, scaling/bounds requirements, initialization obligations, and available derivatives.

**Failure/boundary behavior:** No symbolic equations or derivatives are inferred from a value-only provider; unsupported derivative demands must be handled by an explicitly selected valid method or rejected.

**AT-FLW-07-P:** A local algebraic property model and a qualified external value callback can each participate without changing material identity or authority semantics.

**AT-FLW-07-N:** Temporary initialization fixing that remains active and overconstrains the accepted process model fails the formulation test.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-07, B1:X02, B1:X14, B1:X19, B1:C03, B2:WF-10, B3:PV-15, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-flw-08"></a>
#### FR-FLW-08: Preserve bulk/interface separation in nonequilibrium contacting

**Applicability:** C. **Accountable owner:** Process-model integration. **Initiator:** Rate-based contacting model.

**Trigger:** Separate bulk phases and interface relationships are evaluated.

**Inputs and preconditions:** Bulk states, possibly distinct temperatures/compositions; interface assumptions; flux conventions; unit-owned transport/area closure.

**Requirement:** For the SC-26 formulation, the system shall evaluate bulk and interface thermodynamics without forcing the entire location into bulk equilibrium.

**Outputs and postconditions:** Distinct bulk/interface values and interface-transfer quantities suitable for the unit’s coupled balance checks.

**Failure/boundary behavior:** An efficiency-corrected equilibrium-stage model may be a different declared formulation but cannot pass the full SC-26 requirement by label alone.

**AT-FLW-08-P:** A contactor fixture retains distinct bulk temperatures/compositions while satisfying its selected interface and transfer relationships.

**AT-FLW-08-N:** A provider refresh that replaces both bulk states with a common equilibrium state fails SC-26 authority preservation.

**Verification method:** Pinned numerical/formulation fixture plus required process integration and negative/boundary tests.

**Source/requirement basis:** B1:SC-26, B1:X07, B1:X14, B1:C09, B2:WF-10, B3:W02, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

### RUN. Preparation, numerical execution, and sessions

<a id="fr-run-01"></a>
#### FR-RUN-01: Qualify initialization independently of formulation support

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Execution coordinator or initializer.

**Trigger:** An operation is prepared for numerical execution.

**Inputs and preconditions:** Model/operation eligibility; required initial phase region; available guesses, envelope, bounds, or prior state; initialization policy.

**Requirement:** The system shall report initializer requirements and readiness independently of the ability to formulate or evaluate the thermodynamic problem.

**Outputs and postconditions:** A usable starting context or INITIALIZATION_REQUIRED/INITIALIZATION_FAILED with unmet prerequisites.

**Failure/boundary behavior:** Failure to find a start is not proof of physical infeasibility or blanket model unsupportedness.

**AT-RUN-01-P:** A provider needing a two-phase initial temperature reports that precondition and proceeds when an admissible start is supplied.

**AT-RUN-01-N:** A single-phase starting guess for that profile is not silently accepted as a supported initialization.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-06, B1:SC-07, B1:X06, B1:C11, B3:CR-05, B3:PV-06, B3:PV-13.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-02"></a>
#### FR-RUN-02: Preserve the original problem through initialization

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Initializer or coupled solver.

**Trigger:** Initialization temporarily fixes variables, changes activation, or uses simplified work states.

**Inputs and preconditions:** Original physical constraints; temporary numerical adjustments; intended restoration and convergence checks.

**Requirement:** The system shall restore or explicitly reconcile temporary initialization changes before accepting a solution of the original problem.

**Outputs and postconditions:** Initialization record and a final check against the original constraint/authority set.

**Failure/boundary behavior:** An initializer’s relaxed model or fixed guess cannot silently become the production model.

**AT-RUN-02-P:** Temporarily fixed local variables are released as required before the coupled-model acceptance check.

**AT-RUN-02-N:** A simplified ideal initialization that is never restored cannot pass as the requested nonideal calculation.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-07, B1:X02, B1:X07, B1:C03, B3:CR-05, B3:PV-15.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-03"></a>
#### FR-RUN-03: Bind reusable sessions to compatible configuration revisions

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Provider-session controller.

**Trigger:** A runtime session is created, reused, rebound, or retired.

**Inputs and preconditions:** Semantic configuration revision; provider/build; component map; parameter/reference settings; verified lifecycle rules.

**Requirement:** The system shall use a provider session only with the configuration and context for which its state and caches are valid.

**Outputs and postconditions:** Validated session reuse, reconstruction, or explicit rebinding with recoverable provenance.

**Failure/boundary behavior:** A reference/parameter/component change invalidates incompatible session reuse; a stale handle is not a current model.

**AT-RUN-03-P:** Changing a reference convention reconstructs or rebinds affected sessions before the next result is accepted.

**AT-RUN-03-N:** An old handle retaining prior reference settings cannot label its results with the new revision.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X16, B1:X17, B1:X18, B1:C12, B3:CR-01, B3:CR-12, B3:PV-12.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-04"></a>
#### FR-RUN-04: Isolate complete calculation sequences across cases

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Execution coordinator.

**Trigger:** Independent cases or local calculations are interleaved or run concurrently.

**Inputs and preconditions:** Sessions; model activation steps; global/provider state; errors/caches; explicitly tested sharing policy.

**Requirement:** The system shall prevent one calculation context from changing another’s physical configuration, material state, or diagnostics through shared provider state.

**Outputs and postconditions:** Isolated outcomes under a declared serial, locked, replicated, or otherwise verified execution policy.

**Failure/boundary behavior:** Unverified concurrent use is not advertised as supported; internal provider parallelism alone does not establish host reentrancy.

**AT-RUN-04-P:** Interleaving differently parameterized cases reproduces each serial reference under the chosen isolation policy.

**AT-RUN-04-N:** Activating model B between activation and evaluation of model A cannot contaminate A’s accepted result.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X18, B1:C12, B2:P13, B3:CR-12, B3:PV-09.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-05"></a>
#### FR-RUN-05: Separate same-problem retry from an alternate physical calculation

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Execution coordinator or authorized user policy.

**Trigger:** A numerical operation fails or requests recovery.

**Inputs and preconditions:** Original request/configuration; allowed numerical retries; separately authorized physical alternatives; failure reason.

**Requirement:** The system shall distinguish a same-problem numerical retry from an alternate-model, restricted-phase, or omitted-equilibrium calculation.

**Outputs and postconditions:** Attempt history and requested-versus-used physical/numerical choices; altered-physics candidates cannot satisfy the original request’s success status.

**Failure/boundary behavior:** Absent authorization, physical substitution is not attempted as recovery; failure remains visible.

**AT-RUN-05-P:** Changing only a compatible root-solving method yields a same-problem retry with its own numerical record.

**AT-RUN-05-N:** Retrying a required-VLLE request as ideal VLE produces an alternate-problem result, never an ordinary VLLE success.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X07, B1:X23, B1:C11, B2:WF-05, B2:P04, B3:CR-13.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-06"></a>
#### FR-RUN-06: Support cancellation without publishing interrupted trials

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** User or process coordinator.

**Trigger:** Cancellation or execution-budget exhaustion occurs.

**Inputs and preconditions:** Active request; cancellation signal or configured budget; provider interruptibility; current working state.

**Requirement:** The system shall distinguish cancellation/budget exhaustion from infeasibility and prevent interrupted trial values from becoming accepted results.

**Outputs and postconditions:** A cancellation or budget outcome with any inspectable diagnostic candidate clearly unaccepted.

**Failure/boundary behavior:** If a native call cannot be interrupted immediately, late results are not auto-published and cancellation remains visible; no hard real-time guarantee is implied.

**AT-RUN-06-P:** Cancelling after several local iterates leaves the prior accepted result and its original provenance intact.

**AT-RUN-06-N:** A late callback after cancellation cannot overwrite the accepted process state.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X23, B1:C12, B2:WF-11, B3:CR-13, B3:PV-20.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-07"></a>
#### FR-RUN-07: Contain provider failures and qualify subsequent reuse

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Provider-session controller.

**Trigger:** An external provider returns an error, crashes, or leaves uncertain working state.

**Inputs and preconditions:** Native outcome/error; session lifecycle contract; accepted host state; reconstruction policy.

**Requirement:** The system shall translate provider failure into a qualified host outcome and reuse the affected session only when recovery is established.

**Outputs and postconditions:** Provider/build and stage-specific error information plus retired, reconstructed, or demonstrably reusable session status.

**Failure/boundary behavior:** Uncertain provider state is quarantined rather than trusted for the next result; failure is not reduced to a missing-property zero.

**AT-RUN-07-P:** After an injected provider failure, a fresh valid request succeeds through a reconstructed session without changing prior accepted state.

**AT-RUN-07-N:** A provider’s retained output buffer from the failed call cannot be published as the next request’s result.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X18, B1:X23, B1:C12, B2:P03, B3:CR-12, B3:PV-20.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-run-08"></a>
#### FR-RUN-08: Record nested convergence independently

**Applicability:** H. **Accountable owner:** Execution and provider-session control. **Initiator:** Unit or flowsheet coordinator.

**Trigger:** A property, equilibrium, stage, unit, or recycle iteration completes or exhausts limits.

**Inputs and preconditions:** Convergence criteria/residuals for each participating level; maximum iterations; parent request and publication requirements.

**Requirement:** The system shall keep local numerical completion distinct from required unit and whole-flowsheet convergence.

**Outputs and postconditions:** Level-specific outcomes and residuals, with local candidates inspectable but not promoted beyond the convergence achieved.

**Failure/boundary behavior:** A recycle or efficiency-loop pass limit cannot be hidden by successful local property calls.

**AT-RUN-08-P:** A converged flash inside an unconverged recycle is labeled locally completed and whole-flowsheet unaccepted.

**AT-RUN-08-N:** Exhausting a column correction loop cannot mark the specified coupled column solution converged merely because the last stage calculations returned values.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-07, B1:SC-13, B1:X15, B1:C12, B2:WF-11, B2:P14, B3:W04.

**Status:** Implementation not assessed; witnesses not executed.

### RES. Outcomes, acceptance, and publication

<a id="fr-res-01"></a>
#### FR-RES-01: Return structured, distinguishable operation outcomes

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Any calculation consumer.

**Trigger:** A requested operation terminates or is rejected.

**Inputs and preconditions:** Request identity; execution stage; values and diagnostics; provider error when applicable; required output contract.

**Requirement:** The system shall distinguish unsupported operation, unresolved data, incompatible methods, invalid/incomplete specification, initialization failure, nonconvergence, failed checks, provider failure, and cancellation.

**Outputs and postconditions:** A qualified outcome with stage/cause and inspectable candidate or per-property status where appropriate.

**Failure/boundary behavior:** Unknown failure causes remain unknown; nonconvergence is not automatically physical infeasibility.

**AT-RES-01-P:** Injecting missing data and then numerical nonconvergence yields two different classified outcomes linked to the same requested operation.

**AT-RES-01-N:** A solver exception cannot be converted to a successful zero enthalpy or a universal unsupported-model claim.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X09, B1:X23, B1:C11, B2:WF-04, B3:CR-13, B3:PV-20.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-02"></a>
#### FR-RES-02: Adopt results only after declared acceptance checks

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Result acceptance controller.

**Trigger:** A candidate calculation is proposed for acceptance.

**Inputs and preconditions:** Original physical request; candidate values; required checks; predeclared absolute/relative tolerances and scales; available residual evidence.

**Requirement:** The system shall evaluate the required postconditions against the original request before marking an operation accepted.

**Outputs and postconditions:** Pass/fail/not-assessed evidence per required check, separate from provider convergence and empirical validation.

**Failure/boundary behavior:** A failed or unavailable mandatory check blocks that acceptance level; explicitly optional checks may remain unassessed.

**AT-RES-02-P:** A mock converged PH result is accepted only after its target enthalpy and applicable conservation residuals meet the declared criteria.

**AT-RES-02-N:** A small solver update with a large physical balance residual is rejected even when the provider reports success.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-06, B1:X12, B1:X23, B1:C11, B3:CR-13, B3:PV-11.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-03"></a>
#### FR-RES-03: Apply conservation and thermal checks on justified bases

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Unit or equilibrium acceptance controller.

**Trigger:** A material transformation or energy exchange is checked.

**Inputs and preconditions:** Declared conserved quantities; external flows/reservoirs; energy conventions; candidate state; fixed tolerances and physical scales.

**Requirement:** The system shall evaluate conservation and thermal residuals using the justified quantities and all declared exchanges without hiding errors through cancellation-based scaling.

**Outputs and postconditions:** Absolute and scaled residuals, used scales/tolerances, and pass/fail/not-assessed conclusions.

**Failure/boundary behavior:** Unknown elemental data cannot produce an elemental pass; invalid zero-scale comparisons use the declared absolute floor.

**AT-RES-03-P:** Equal and opposite large energy streams are checked against gross physical scales rather than only their nearly zero net value.

**AT-RES-03-N:** A reaction balance fails when a declared titrant exchange is omitted, even if the internal species solver converged.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-17, B1:SC-22, B1:X11, B1:X12, B1:C07, B3:CR-10.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-04"></a>
#### FR-RES-04: Publish only the authorized result scope

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Unit model, external component, or flowsheet coordinator.

**Trigger:** A calculation publishes outputs to a shared process state.

**Inputs and preconditions:** Publication kind: constraints, authoritative allocation, evaluated local state, or accepted flowsheet result; authority and required convergence.

**Requirement:** The system shall publish results according to the declared completion and authority contract, keeping target specifications distinct from resolved states.

**Outputs and postconditions:** A published object/result with its scope, completion level, unresolved work, and source revisions identifiable.

**Failure/boundary behavior:** Publishing PH outlet constraints does not mark all phase properties completed; external allocation does not authorize unrelated species changes.

**AT-RES-04-P:** A valve publishes PH constraints explicitly as pending state completion, or publishes a checked resolved state under a different declared contract.

**AT-RES-04-N:** A stream awaiting phase resolution cannot be labeled a fully evaluated outlet solely because its unit completed.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-06, B1:X15, B1:X23, B1:C12, B2:WF-06, B2:WF-15, B3:CR-03.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-05"></a>
#### FR-RES-05: Protect accepted results from failed or obsolete work

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Execution/publication controller.

**Trigger:** A calculation runs, fails, or completes after its inputs have changed.

**Inputs and preconditions:** Accepted result revision; captured input/configuration revisions; candidate result; current model revisions.

**Requirement:** The system shall prevent working-state mutations and obsolete completions from silently replacing an accepted current result.

**Outputs and postconditions:** Prior results preserved with original lineage; candidate adoption only for matching authority and revisions, otherwise stale/historical or rejected.

**Failure/boundary behavior:** Failure, cancellation, or intervening edits do not relabel an old candidate as current.

**AT-RES-05-P:** A calculation launched under parameter revision A and finishing after revision B is retained as A-linked history, not B’s current result.

**AT-RES-05-N:** A provider overwriting a work array before throwing cannot alter the independently retained accepted result.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X16, B1:X23, B1:C12, B2:P03, B3:CR-13, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-06"></a>
#### FR-RES-06: Expose independent result-quality dimensions

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** User, unit model, or report consumer.

**Trigger:** A result is queried or summarized.

**Inputs and preconditions:** Readiness, outcome, convergence level, authority, scientific validity, revision currency, and publication evidence.

**Requirement:** The system shall report result quality without collapsing numerical completion, physical validity, currentness, and publication into one calculated flag.

**Outputs and postconditions:** An interpretable multi-dimensional status with unknown/not-assessed values retained.

**Failure/boundary behavior:** A stale validated historical state is not current; a converged extrapolated result is not automatically validated.

**AT-RES-06-P:** A locally converged but globally unconverged trial reports both facts without contradiction.

**AT-RES-06-N:** A presentation layer that shows unqualified success for an alternate-model or stale result fails status conformance.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X10, B1:X15, B1:X16, B1:C11, B1:C12, B2:WF-11, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-07"></a>
#### FR-RES-07: Retain result lineage and requested-versus-used assumptions

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Results or audit consumer.

**Trigger:** A result is stored, compared, or reproduced.

**Inputs and preconditions:** Request and definition revisions; provider/build/binding; actual methods/data; numerical policy; initial/branch context where consequential; substitutions.

**Requirement:** The system shall retain enough lineage to explain what physical problem and implementation actually produced the result.

**Outputs and postconditions:** An attributable result record with requested-versus-used differences and links to necessary reproducibility inputs.

**Failure/boundary behavior:** A result missing required lineage is marked non-reproducible or incompletely attributable, not silently completed with current defaults.

**AT-RES-07-P:** An approved estimated parameter remains visible in the lineage of a downstream heat-duty result.

**AT-RES-07-N:** Replacing a provider version cannot retroactively relabel a historical result as produced by the new version.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X08, B1:X17, B1:C06, B1:C12, B3:CR-13, B3:CR-14.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-res-08"></a>
#### FR-RES-08: Distinguish numerical conformance from physical validation

**Applicability:** H. **Accountable owner:** Result qualification and publication. **Initiator:** Validation reviewer.

**Trigger:** A scenario or provider gains evidence or a claim is reviewed.

**Inputs and preconditions:** Executed fixture and manifest; predicted outputs; independent reference where available; uncertainty; numerical and model/data tolerances.

**Requirement:** The system shall distinguish adapter/implementation agreement, physical-consistency checks, reference-data agreement, and operational reproducibility evidence.

**Outputs and postconditions:** A validation record bounded by configuration and envelope; absent independent evidence lowers the claim instead of being hidden.

**Failure/boundary behavior:** Agreement between libraries sharing a model or dataset does not alone establish independent physical validation.

**AT-RES-08-P:** A synthetic contract test can pass while the corresponding real-fluid model remains unvalidated.

**AT-RES-08-N:** A structural check that every scenario has a requirement cannot be reported as an executed thermodynamic scenario.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X10, B1:X17, B1:C11, B3:CR-04, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

### LIF. Changes, invalidation, and reconstruction

<a id="fr-lif-01"></a>
#### FR-LIF-01: Create a revision and impact assessment for consequential changes

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Model author or change coordinator.

**Trigger:** Composition, data, method, phase/reaction policy, or package binding changes.

**Inputs and preconditions:** Old and proposed definitions/specifications; dependency relationships or conservative affected-region set; change intent.

**Requirement:** The system shall identify consequential model changes and mark dependent results and execution contexts stale or requiring reassessment.

**Outputs and postconditions:** New semantic revision, affected results/sessions, retained historical evidence, and required reinitialization/requalification.

**Failure/boundary behavior:** A targeted-invalidation implementation that cannot determine impact must invalidate conservatively rather than leave potentially stale results current.

**AT-LIF-01-P:** Changing a binary parameter invalidates dependent equilibrium and unit energy results while independent configurations retain their identity.

**AT-LIF-01-N:** An old cached result with the former parameter set cannot be returned as current without reassessment.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X08, B1:X16, B1:C12, B2:WF-13, B3:CR-01, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-02"></a>
#### FR-LIF-02: Change a material slate with an explicit quantity/dependency policy

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Model author.

**Trigger:** A constituent is added or removed from an existing model.

**Inputs and preconditions:** Affected material and reaction/phase/mapping definitions; preserve-total or preserve-remaining-quantities choice; impact preview.

**Requirement:** The system shall make material-slate changes through an explicit domain action that validates quantity and dependency effects before acceptance.

**Outputs and postconditions:** Updated representations and reconciled quantities, or a rejected change with affected dependencies; semantics independent of interface.

**Failure/boundary behavior:** Removing a required reaction species cannot leave an apparently ready reaction; removed material is not silently redistributed.

**AT-LIF-02-P:** Removing A while preserving B’s component flow updates totals and invalidates dependent state calculations.

**AT-LIF-02-N:** Removing A from a still-active reaction referencing A is rejected or leaves that reaction explicitly unresolved, never ready.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-01, B1:SC-21, B1:SC-30, B1:X13, B1:X16, B1:C01, B2:WF-12, B2:P09.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-03"></a>
#### FR-LIF-03: Separate presentation changes from physical and numerical changes

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Model editor or automation caller.

**Trigger:** Display units, labels, graphical layout, or numerical controls change.

**Inputs and preconditions:** Change semantics; actual physical values; process topology; numerical policy and branch/acceptance effects.

**Requirement:** The system shall distinguish display-only changes, physical-value/topology changes, and numerical-policy changes when reassessing result currency.

**Outputs and postconditions:** No physical invalidation from pure representation/layout changes; explicit reassessment for numerical evidence or branch-policy changes.

**Failure/boundary behavior:** Moving an icon cannot change process topology or thermodynamics unless an explicit connection action occurs.

**AT-LIF-03-P:** Changing a displayed temperature unit while preserving the physical value leaves the physical configuration unchanged.

**AT-LIF-03-N:** Relaxing a convergence tolerance cannot silently convert a previously failed physical residual into validated accuracy.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X01, B1:X16, B1:C12, B2:WF-13, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-04"></a>
#### FR-LIF-04: Serialize the semantic model and resolved data

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Persistence caller.

**Trigger:** A case or configured package is saved for reconstruction.

**Inputs and preconditions:** Material and characterization definitions; data values/references; package recipe; bindings; reaction/physical policies; specifications; result lineage; dependency manifest.

**Requirement:** The system shall preserve the semantic inputs required to reconstruct the thermodynamic problem rather than only runtime handles, caches, or solved values.

**Outputs and postconditions:** A reconstructable archive or explicit unresolved-dependency record, with saved results separate from model definitions and runtime state.

**Failure/boundary behavior:** Unavailable dependencies or nonserializable semantic extensions are reported; caches cannot stand in for authoritative data.

**AT-LIF-04-P:** Saving a case retains the assay, generated cuts, selected caloric model, parameter values, and package assignments.

**AT-LIF-04-N:** An archive containing only an in-memory provider handle and final species array fails reconstruction completeness.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:SC-15, B1:X17, B1:C06, B1:C12, B2:WF-14, B3:CR-14, B3:PV-02.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-05"></a>
#### FR-LIF-05: Reconstruct definitions before restoring result authority

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Case loader or migration coordinator.

**Trigger:** A saved case is loaded or migrated.

**Inputs and preconditions:** Archive schema/version; semantic recipe; required dependencies; identity relationships; stored values/results; approved migration rules.

**Requirement:** The system shall distinguish inspection load, semantic reconstruction, dependency resolution, calculation readiness, and validity of restored results.

**Outputs and postconditions:** Rebuilt identities/bindings and explicit migration/readiness status; missing dependencies remain visible.

**Failure/boundary behavior:** Restoring values into a different model recipe does not confer result validity; unavailable models cannot be silently substituted.

**AT-LIF-05-P:** A missing provider permits inspection of the case while blocking affected calculations and marking restored results historical/unresolved.

**AT-LIF-05-N:** Applying a state snapshot to changed equations without compatibility assessment fails the restore-authority check.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X17, B1:C12, B2:WF-14, B2:P12, B3:CR-14, B3:PV-16.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-06"></a>
#### FR-LIF-06: Restore snapshots without losing lineage or dependency consistency

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Undo/redo or recovery caller.

**Trigger:** A model snapshot is restored.

**Inputs and preconditions:** Snapshot definitions/specifications/results; current revision; shared material relationships; dependency compatibility.

**Requirement:** The system shall restore semantic relationships and reassess result/session currency rather than merely copy numerical values.

**Outputs and postconditions:** A restored configuration with preserved historical identity and explicit revalidation/recalculation needs.

**Failure/boundary behavior:** Snapshot restoration cannot make a result current under a different unresolved parameter or provider revision.

**AT-LIF-06-P:** Undoing a material change restores its original mapping and checks related package/reaction bindings before reuse.

**AT-LIF-06-N:** An undo that repairs a component list but leaves solver coordinates in the changed order fails restoration conformance.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X13, B1:X16, B1:X17, B1:C12, B2:WF-13, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-07"></a>
#### FR-LIF-07: Reproduce cases within declared tolerances and manifest limits

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** Reproduction or comparison caller.

**Trigger:** A stored result is rerun or compared after an upgrade.

**Inputs and preconditions:** Saved semantic inputs; actual provider/data/build identities; numerical/branch policy; declared reproduction tolerance; known nondeterminism.

**Requirement:** The system shall distinguish a reproduction under the recorded configuration from a comparison using changed dependencies or methods.

**Outputs and postconditions:** Reproduction/comparison record with matching and changed inputs, outcome differences, and tolerance-based assessment.

**Failure/boundary behavior:** Bitwise identity is not assumed; an unavailable original dependency prevents an unqualified exact-configuration reproduction claim.

**AT-LIF-07-P:** Rebuilding a compatible session from recorded inputs yields values within the fixture’s declared reproduction tolerance.

**AT-LIF-07-N:** A rerun with a changed parameter database is labeled a changed-model comparison rather than a reproduction of the original case.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X17, B1:C12, B3:CR-14, B3:PV-01, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-lif-08"></a>
#### FR-LIF-08: Apply the same change semantics through every interface

**Applicability:** H. **Accountable owner:** Model change and reconstruction. **Initiator:** GUI, automation, import, or batch caller.

**Trigger:** An equivalent supported domain operation is invoked from different interfaces.

**Inputs and preconditions:** Same semantic input operation and configuration; interface-specific presentation only.

**Requirement:** The system shall apply equivalent material-edit, configuration, readiness, and publication semantics independently of the caller interface.

**Outputs and postconditions:** Equivalent semantic revisions and qualified outcomes within the declared numerical reproducibility conditions.

**Failure/boundary behavior:** A frontend bypass that omits required validation or invalidation does not satisfy the operation contract.

**AT-LIF-08-P:** The same complete-composition replacement from a GUI and an automation client produces identical named material quantities.

**AT-LIF-08-N:** A loaded-case edit retaining unnamed defaults while a new-case edit removes them fails interface-equivalence testing.

**Verification method:** Contract inspection plus positive/negative mock or fault-injection tests; actual integration where applicable.

**Source/requirement basis:** B1:X16, B1:X17, B1:C12, B2:WF-02, B2:WF-12, B2:P08, B3:W07.

**Status:** Implementation not assessed; witnesses not executed.

### EXT. Explicit P3 extension obligations

<a id="fr-ext-01"></a>
#### FR-EXT-01: Represent surface and selective-transfer domains meaningfully

**Applicability:** R3. **Accountable owner:** Extension semantics. **Initiator:** Domain-model reviewer.

**Trigger:** SC-27 is walked through in the architecture blueprint.

**Inputs and preconditions:** Bulk and nonbulk domains; surface/site/loading bases where required; membrane-side identities; permitted transfer/storage operations.

**Requirement:** The blueprint shall describe surface/selective-transfer material and conserved quantities without representing every stored surface amount as an ordinary liquid phase.

**Outputs and postconditions:** A complete conceptual state-and-transfer walkthrough with explicit unsupported numerical requests.

**Failure/boundary behavior:** An untyped metadata field alone does not meet representability; no adsorption/membrane numerical implementation is implied.

**AT-EXT-01-P:** An adsorption walkthrough accounts separately for material in the bulk and on sites and reconciles a transfer between them.

**AT-EXT-01-N:** A surface-loading number with no site/area/mass basis or conservation relation fails the walkthrough.

**Verification method:** Conceptual state/action/conservation walkthrough.

**Source/requirement basis:** B1:SC-27, B1:X01, B1:X13, B1:C13, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-ext-02"></a>
#### FR-EXT-02: Represent distributed material attributes and their reductions

**Applicability:** R3. **Accountable owner:** Extension semantics. **Initiator:** Domain-model reviewer.

**Trigger:** SC-32 is walked through in the architecture blueprint.

**Inputs and preconditions:** Distribution or moment definition; basis; material association; mixing/transfer/reduction rules; known information loss.

**Requirement:** The blueprint shall preserve or explicitly reduce distribution-valued material characterization with defined operation semantics.

**Outputs and postconditions:** A conceptual polymer/distributed-material walkthrough identifying conserved material and retained/lost attribute information.

**Failure/boundary behavior:** An arbitrary mean molecular weight or opaque attachment cannot be claimed a lossless distribution representation.

**AT-EXT-02-P:** A mixing/transfer walkthrough preserves the declared mass-based distribution or records a defined moment reduction.

**AT-EXT-02-N:** Reverse reconstruction from one mean into a unique full distribution is rejected as information not supplied.

**Verification method:** Conceptual state/action/conservation walkthrough.

**Source/requirement basis:** B1:SC-32, B1:X01, B1:X13, B1:C13, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-ext-03"></a>
#### FR-EXT-03: Represent inventory-based state specifications without a flow workaround

**Applicability:** R3. **Accountable owner:** Extension semantics. **Initiator:** Domain-model reviewer.

**Trigger:** SC-33 is walked through in the architecture blueprint.

**Inputs and preconditions:** Material inventories; total internal energy and volume or another admissible specification; time/location identity; empty-inventory policy.

**Requirement:** The blueprint shall describe inventory-state resolution independently of steady-flow quantities and without prescribing a time integrator.

**Outputs and postconditions:** A conceptual nonempty and empty inventory walkthrough with explicit intensive/extensive relationships and unknowns.

**Failure/boundary behavior:** No fictitious flow or implicit duration may be used to make an inventory appear compatible; numerical UV execution remains deferred unless separately claimed.

**AT-EXT-03-P:** A closed inventory with stated material amounts,U,V can express T,P as unresolved outputs with no mass-flow field required.

**AT-EXT-03-N:** Total U,V alone without justified material amount/composition cannot be treated as a complete unique state.

**Verification method:** Conceptual state/action/conservation walkthrough.

**Source/requirement basis:** B1:SC-33, B1:X03, B1:X24, B1:C02, B1:C03, B3:PV-17, B3:W08.

**Status:** Implementation not assessed; witnesses not executed.

<a id="fr-ext-04"></a>
#### FR-EXT-04: Represent restricted-equilibrium and metastable requests explicitly

**Applicability:** R3. **Accountable owner:** Extension semantics. **Initiator:** Domain-model reviewer.

**Trigger:** SC-34 is walked through in the architecture blueprint.

**Inputs and preconditions:** Excluded phases/reactions or requested branch; rationale; validity limits; requested property/equilibrium operation.

**Requirement:** The blueprint shall distinguish a restriction on the physical problem, a metastable branch request, and a numerical starting guess.

**Outputs and postconditions:** A conceptual request/result walkthrough retaining the restriction and the limited equilibrium/stability claim.

**Failure/boundary behavior:** Removing a restriction invalidates the earlier result for the new problem; numerical metastable support is not inferred.

**AT-EXT-04-P:** A suppressed-solid walkthrough retains the suppression label and records that unrestricted solid stability is not certified.

**AT-EXT-04-N:** A phase-constrained value returned as unrestricted stable equilibrium fails representability acceptance.

**Verification method:** Conceptual state/action/conservation walkthrough.

**Source/requirement basis:** B1:SC-34, B1:X06, B1:X07, B1:C08, B3:CR-02, B3:W02.

**Status:** Implementation not assessed; witnesses not executed.

## 4. Scenario acceptance contracts

SA identifiers are acceptance decompositions of existing B1 SC scenarios, not new scope. For every P1/P2 scenario, acceptance ultimately requires both suitable local thermodynamic witnesses and at least one actual process-integration witness for its family. The listed positive test is conjunctive: all required conditions must hold. A standalone provider call is insufficient evidence of a completed unit scenario. The corresponding negative test must demonstrate the prescribed limitation/failure behavior. Applicable H requirements and the mapped X rules remain inherited even when not repeated in the primary list.

Numerical fixture inputs, published references, applicable ranges and tolerances remain to be pinned; that is a fixture gate, not permission to weaken the normative postcondition. P3 acceptance is a later domain/action walkthrough with explicit state, transfer, conservation, change and failure semantics. No numerical support is inferred.

<a id="sa-01"></a>
### SA-01 / SC-01: Blending and splitting without reaction

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-01.

**Process trigger:** Two independently specified feeds → mixer → composition-preserving splitter.

**Objective:** Combine and partition material without introducing hidden chemical or equilibrium assumptions.

**Inputs/assumptions:** Compatible material identities; independent feed states; explicit flow bases; outlet pressure rule and heat/work assumptions supplied by the unit model.

**Required observable behavior:** Resolve the outlet thermal state from the specified balance constraints. Distinguish a mechanical split from an equilibrium separation. Permit a property request independent of total flow.

**Primary requirements:** [FR-MAT-05](#fr-mat-05), [FR-MAT-06](#fr-mat-06), [FR-MAT-08](#fr-mat-08), [FR-FLW-02](#fr-flw-02), [FR-RES-03](#fr-res-03).

**Cross-cutting scope rules:** X01, X03, X11, X14, X23.

**AT-SA-01-P:** Run two-feed mixing under declared pressure/heat rules, then a composition-preserving split and recombination. Check component flows and energy; splitting changes quantities, not the material’s intensive state.

**AT-SA-01-N:** Supply incompatible reference conventions or an invalid split; reject or require explicit boundary treatment instead of averaging temperatures or dropping components.

**Failure obligation inherited from scope:** Reject incompatible component mappings or enthalpy conventions rather than silently averaging temperatures or dropping components.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-02"></a>
### SA-02 / SC-02: Sensible heating and heat exchange

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-02.

**Process trigger:** Single-phase liquid or gas → heater/cooler; repeat as one side of an exchanger.

**Objective:** Support both a specified outlet temperature with computed duty and a specified duty with computed outlet state.

**Inputs/assumptions:** Feed state and amount flow; pressure or pressure-drop rule; one thermal specification; a compatible caloric method.

**Required observable behavior:** Provide enthalpy and heat capacity where required. For equipment rating, provide the requested conductivity and viscosity separately; the unit supplies geometry and heat-transfer correlations.

**Primary requirements:** [FR-CFG-04](#fr-cfg-04), [FR-EQL-02](#fr-eql-02), [FR-PRP-02](#fr-prp-02), [FR-PRP-03](#fr-prp-03), [FR-FLW-01](#fr-flw-01), [FR-FLW-04](#fr-flw-04).

**Cross-cutting scope rules:** X02, X09, X19, X22.

**AT-SA-02-P:** For one pinned fluid, solve temperature-specified duty and duty-specified outlet modes and verify they agree on the selected branch. Add rating demands separately.

**AT-SA-02-N:** Remove a required caloric method, then an optional rating/report property. Distinguish the blocked energy calculation from a qualified omitted optional result.

**Failure obligation inherited from scope:** Reject an over-specified thermal request, or identify the unsupported rating property without inventing it.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-03"></a>
### SA-03 / SC-03: Liquid pumping and pressure-loss calculations

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-03.

**Process trigger:** Liquid feed → pump → pipe segment → outlet.

**Objective:** Use liquid thermophysical properties in pressure-changing equipment without moving equipment physics into thermodynamics.

**Inputs/assumptions:** Liquid state; flow basis; pump efficiency or work relationship; pipe geometry and hydraulic closure supplied by the unit.

**Required observable behavior:** Supply density, enthalpy and viscosity; supply compressibility or additional properties only when the selected unit formulation requires them. Report the actual phase policy and validity.

**Primary requirements:** [FR-PRP-01](#fr-prp-01), [FR-PRP-03](#fr-prp-03), [FR-CFG-03](#fr-cfg-03), [FR-FLW-01](#fr-flw-01), [FR-EQL-06](#fr-eql-06).

**Cross-cutting scope rules:** X07, X09, X10, X14.

**AT-SA-03-P:** Use declared local density, enthalpy, and viscosity in a selected pump/pipe formulation; close its work and pressure relations and assess allowed phase behavior.

**AT-SA-03-N:** Cross the liquid-only operating assumption into vapor formation; reject or explicitly qualify a separately chosen approximation rather than continuing as single liquid.

**Failure obligation inherited from scope:** Do not continue a liquid-only hydraulic calculation as though vapor formation were impossible; return a policy-appropriate diagnostic.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-04"></a>
### SA-04 / SC-04: Gas compression, expansion and intercooling

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-04.

**Process trigger:** Gas feed → compressor/expander → intercooler → optional subsequent stage.

**Objective:** Support ordinary pressure-changing gas equipment and its ideal reference state.

**Inputs/assumptions:** Feed state; outlet pressure; unit-selected efficiency/work relation; declared phase policy.

**Required observable behavior:** Support the pressure–entropy reference-state calculation and the actual energy-constrained outlet. Keep the ideal reference calculation distinct from the actual equipment result.

**Primary requirements:** [FR-EQL-03](#fr-eql-03), [FR-EQL-02](#fr-eql-02), [FR-CFG-04](#fr-cfg-04), [FR-STA-04](#fr-sta-04), [FR-PRP-05](#fr-prp-05), [FR-FLW-01](#fr-flw-01).

**Cross-cutting scope rules:** X09, X10, X14, X19.

**AT-SA-04-P:** Resolve an ideal-reference PS state and an actual efficiency/work-constrained outlet; retain both identities and verify individual unit and intercooler energy balances.

**AT-SA-04-N:** Remove entropy support or use an invalid inverse initializer; the reference calculation is unavailable/not initialized even if TP evaluation still works.

**Failure obligation inherited from scope:** A provider lacking a usable entropy method cannot claim this scenario merely because it supports temperature–pressure flashes.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-05"></a>
### SA-05 / SC-05: Cooling and vapor–liquid separation

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-05.

**Process trigger:** Multicomponent hydrocarbon feed → cooler → equilibrium separator → vapor and liquid products.

**Objective:** Establish the basic multicomponent phase-split behavior used by a flowsheet.

**Inputs/assumptions:** Feed composition and flow; separator pressure and thermal specification; permitted phases and a pinned property configuration.

**Required observable behavior:** Determine phase amounts and phase compositions; evaluate properties required for balances; distinguish material redistribution from chemical reaction.

**Primary requirements:** [FR-EQL-01](#fr-eql-01), [FR-EQL-06](#fr-eql-06), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-FLW-03](#fr-flw-03), [FR-RES-03](#fr-res-03).

**Cross-cutting scope rules:** X04, X05, X06, X07.

**AT-SA-05-P:** Resolve the cooler/separator internal state, route vapor and liquid under declared rules, and reconstruct feed component amounts and the thermal constraint.

**AT-SA-05-N:** Return a phase-restricted or unchecked solution while claiming unrestricted stability; reject that claim without confusing it with numerical nonconvergence.

**Failure obligation inherited from scope:** Do not report an unverified unstable or restricted-phase solution as unrestricted stable equilibrium.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-06"></a>
### SA-06 / SC-06: Pressure reduction with flashing

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-06.

**Process trigger:** Pressurized feed → adiabatic valve → downstream equilibrium state.

**Objective:** Support energy-constrained state resolution when pressure changes and phases may appear.

**Inputs/assumptions:** Feed material and state; outlet pressure; valve assumptions excluding shaft work and material kinetic/potential energy changes for the reference fixture.

**Required observable behavior:** Resolve a pressure–enthalpy outlet request with an explicit enthalpy basis and reference convention. Preserve the input accepted state during unsuccessful trials.

**Primary requirements:** [FR-EQL-02](#fr-eql-02), [FR-STA-03](#fr-sta-03), [FR-RES-02](#fr-res-02), [FR-RES-04](#fr-res-04), [FR-RES-05](#fr-res-05), [FR-RUN-01](#fr-run-01).

**Cross-cutting scope rules:** X02, X05, X06, X23.

**AT-SA-06-P:** Apply the valve’s declared adiabatic/no-work pressure change, resolve PH, and check original enthalpy and component constraints before publishing outlet state or explicit pending constraints.

**AT-SA-06-N:** Inject a converged-looking result with a failed enthalpy residual, and separately a failed call; neither may overwrite the prior accepted state.

**Failure obligation inherited from scope:** An infeasible request, unsupported specification, ambiguous branch or numerical failure must remain distinguishable.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-07"></a>
### SA-07 / SC-07: Conventional equilibrium-stage distillation

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-07.

**Process trigger:** Feed → multistage column with condenser and reboiler → products.

**Objective:** Demonstrate that thermodynamic states exist inside equipment, not only on flowsheet connections.

**Inputs/assumptions:** Material definition; stage locations; pressure profile and column specifications owned by the unit; compatible phase and caloric methods.

**Required observable behavior:** Evaluate separate vapor/liquid states and equilibrium relations at each stage. Provide the same physical relationships to repeated numerical evaluation or a coupled process formulation without prescribing the global solver.

**Primary requirements:** [FR-STA-01](#fr-sta-01), [FR-STA-04](#fr-sta-04), [FR-PRP-01](#fr-prp-01), [FR-FLW-07](#fr-flw-07), [FR-RUN-02](#fr-run-02), [FR-RUN-08](#fr-run-08), [FR-RES-04](#fr-res-04).

**Cross-cutting scope rules:** X14, X15, X19, X23.

**AT-SA-07-P:** Evaluate staged vapor/liquid states in an actual column formulation; verify accepted local phase relations, stage balances, whole-column balance, and required outer convergence.

**AT-SA-07-N:** Remove a needed property at one stage or leave temporary initialization constraints active; identify the location and do not publish converged products.

**Failure obligation inherited from scope:** Local property success must not imply stage, column or flowsheet convergence.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-08"></a>
### SA-08 / SC-08: Nonideal-liquid separation and azeotropic behavior

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-08.

**Process trigger:** Polar liquid feed → flash or equilibrium-stage separation; alcohol/water is a candidate fixture family.

**Objective:** Prevent a hydrocarbon-only model architecture while keeping phase behavior and caloric behavior explicitly compatible.

**Inputs/assumptions:** Liquid nonideality method, vapor-phase convention, compatible standard states, required interaction data, and a fixture with independently characterized phase behavior.

**Required observable behavior:** Represent a configured vapor/liquid method combination without treating every method as the same kind of equation of state. Resolve the declared equilibrium and heat duties.

**Primary requirements:** [FR-DAT-02](#fr-dat-02), [FR-DAT-03](#fr-dat-03), [FR-CFG-02](#fr-cfg-02), [FR-CFG-04](#fr-cfg-04), [FR-EQL-07](#fr-eql-07), [FR-RUN-05](#fr-run-05).

**Cross-cutting scope rules:** X06, X08, X10, X11.

**AT-SA-08-P:** Use pinned nonideal phase data to verify the fixture’s required qualitative phase topology as well as quantitative tolerances and compatible energy predictions.

**AT-SA-08-N:** Remove an essential interaction parameter or force ideal fallback. The original nonideal fixture must not pass from solver convergence alone.

**Failure obligation inherited from scope:** Missing interaction parameters, inappropriate method combinations and unsupported extrapolation remain visible; a numerical fallback must not silently replace the physical model.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-09"></a>
### SA-09 / SC-09: Physical absorption, humidification and gas dissolution

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-09.

**Process trigger:** Gas and liquid feeds → nonreactive contact stage → gas and liquid outlets.

**Objective:** Cover dissolved gases and water transfer without making every gas–liquid operation chemically reactive.

**Inputs/assumptions:** Distinct phase compositions; declared transferable components; appropriate phase partition convention; gas reporting basis such as wet gas, dry gas or per unit dry carrier where used.

**Required observable behavior:** Evaluate physical partitioning and phase-specific caloric properties. Preserve phase eligibility and any nontransferring carrier assumptions.

**Primary requirements:** [FR-CFG-02](#fr-cfg-02), [FR-CFG-07](#fr-cfg-07), [FR-PRP-01](#fr-prp-01), [FR-PRP-07](#fr-prp-07), [FR-STA-08](#fr-sta-08), [FR-FLW-01](#fr-flw-01).

**Cross-cutting scope rules:** X01, X08, X11, X21.

**AT-SA-09-P:** Resolve a nonreactive gas/liquid contacting fixture with declared transfer eligibility and dilute-solute conventions; reconcile actual amounts/energy and wet/dry reports.

**AT-SA-09-N:** Introduce chemical redistribution without a reaction definition or omit the solubility/reference convention; reject the altered or incomplete problem.

**Failure obligation inherited from scope:** An undefined Henry-law/reference convention or missing gas–solvent data must not be hidden as ideal partitioning.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-10"></a>
### SA-10 / SC-10: Liquid–liquid extraction and decanting

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-10.

**Process trigger:** Feed and solvent → contacting stage → decanter → two liquid products.

**Objective:** Require multiple instances of one phase category and phase-pair-specific behavior.

**Inputs/assumptions:** A documented partially miscible system; compatible parameters; permitted liquid phases; temperature, pressure and material amounts.

**Required observable behavior:** Determine distinct liquid phase compositions and amounts. Associate an interfacial property with a particular phase pair when requested. Do not use output-array position as physical identity.

**Primary requirements:** [FR-EQL-08](#fr-eql-08), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-PRP-03](#fr-prp-03), [FR-FLW-03](#fr-flw-03).

**Cross-cutting scope rules:** X05, X06, X07, X22.

**AT-SA-10-P:** Resolve a supported two-liquid fixture and rerun with reordered provider candidates. Preserve physical state after valid matching and attach interfacial properties to the correct pair.

**AT-SA-10-N:** A duplicated single liquid or an ambiguous phase correspondence cannot become two unique accepted liquid products.

**Failure obligation inherited from scope:** A method unable to represent the required liquid split must be rejected or declared unsupported, not patched by duplicating one liquid phase.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-11"></a>
### SA-11 / SC-11: Vapor–liquid–liquid separation

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-11.

**Process trigger:** Gas/organic/aqueous feed → three-phase separator → gas and two liquid outlets.

**Objective:** Exercise simultaneous competing phase allocations rather than separate unrelated binary flashes.

**Inputs/assumptions:** A documented three-phase fixture; candidate phase set; material composition and thermal/mechanical specifications.

**Required observable behavior:** Allocate all components over a shared equilibrium problem or a validated equivalent solution procedure. Identify absent and present phases explicitly.

**Primary requirements:** [FR-CFG-07](#fr-cfg-07), [FR-EQL-01](#fr-eql-01), [FR-EQL-08](#fr-eql-08), [FR-EQL-06](#fr-eql-06), [FR-RES-03](#fr-res-03).

**Cross-cutting scope rules:** X04, X05, X06, X07.

**AT-SA-11-P:** Resolve vapor plus two liquids in a shared material problem and verify component/thermal residuals, phase relations, and adjacent phase-appearance behavior.

**AT-SA-11-N:** Reject a two-phase-only replacement, or an assembly of independent flashes that double-counts material, as a solution of the original three-phase request.

**Failure obligation inherited from scope:** A two-phase-only provider cannot claim success by ignoring an allowed aqueous or organic liquid phase.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-12"></a>
### SA-12 / SC-12: Water and steam through saturation

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-12.

**Process trigger:** Condensate → pump → boiler → expansion or heat delivery → condenser.

**Objective:** Support a specialized pure-fluid configuration while exercising saturation and thermal balances.

**Inputs/assumptions:** Pure water identity; compatible package; subcooled, saturated, two-phase and superheated fixture states; explicit phase-fraction basis.

**Required observable behavior:** Distinguish saturated-liquid/vapor endpoint queries, a complete two-phase state and an underdetermined request. Retain the same reference convention around the loop.

**Primary requirements:** [FR-EQL-04](#fr-eql-04), [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03), [FR-STA-02](#fr-sta-02), [FR-STA-08](#fr-sta-08).

**Cross-cutting scope rules:** X02, X05, X11, X19.

**AT-SA-12-P:** Exercise subcooled, endpoint, allocated two-phase, and superheated water states under one caloric convention; verify the selected utility loop balances.

**AT-SA-12-N:** At pure-fluid saturation, TP alone cannot select an intermediate phase fraction; additional valid specification is required for complete allocation.

**Failure obligation inherited from scope:** Ambiguous saturation inputs receive an underdetermination diagnostic, not an arbitrary phase selection presented as a unique solution.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-13"></a>
### SA-13 / SC-13: Pure-fluid refrigeration loop

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-13.

**Process trigger:** Evaporator → compressor → condenser → expansion valve → evaporator.

**Objective:** Combine specialized fluid properties with the same flowsheet request semantics used elsewhere.

**Inputs/assumptions:** Selected pure working fluid; pressure levels; thermal boundary conditions and compressor relation; consistent reference conventions.

**Required observable behavior:** Support evaporation/condensation, pressure–enthalpy and pressure–entropy requests, and the separate unit energy accounting.

**Primary requirements:** [FR-MAT-01](#fr-mat-01), [FR-EQL-04](#fr-eql-04), [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03), [FR-RUN-08](#fr-run-08), [FR-LIF-07](#fr-lif-07).

**Cross-cutting scope rules:** X01, X11, X15, X17.

**AT-SA-13-P:** Run a pinned pure-fluid evaporator/compressor/condenser/valve loop and verify every unit balance and the loop heat/work balance under declared branch policy.

**AT-SA-13-N:** Do not generalize this success to arbitrary mixtures or conceal a failed recycle behind individually completed unit states.

**Failure obligation inherited from scope:** A supported pure-fluid loop must not be advertised as support for arbitrary mixtures of that fluid.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-14"></a>
### SA-14 / SC-14: Mixed-refrigerant phase change

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-14.

**Process trigger:** Specified refrigerant blend → evaporating/condensing loop or exchanger section.

**Objective:** Prevent pure-fluid quality or saturation assumptions from defining all mixture behavior.

**Inputs/assumptions:** Blend composition, actual component identities or explicitly declared pseudo-pure approximation, candidate phases, and mixture-capable data.

**Required observable behavior:** Represent different liquid and vapor compositions and distinct saturation endpoints where the selected fixture requires them. Distinguish mole-based from mass-based phase fraction.

**Primary requirements:** [FR-MAT-02](#fr-mat-02), [FR-STA-08](#fr-sta-08), [FR-EQL-04](#fr-eql-04), [FR-EQL-02](#fr-eql-02), [FR-CFG-02](#fr-cfg-02).

**Cross-cutting scope rules:** X01, X02, X05, X11.

**AT-SA-14-P:** For a true mixture fixture, preserve overall composition while resolving distinct phase compositions and the documented endpoint/glide behavior.

**AT-SA-14-N:** A pseudo-pure approximation or a changed mass/mole quality basis is a different configuration, not an unnoticed shortcut to the original result.

**Failure obligation inherited from scope:** Do not interpret a mixture as a pure fluid merely to obtain a convenient flash call.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-15"></a>
### SA-15 / SC-15: Assay-derived petroleum pseudocomponents

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-15.

**Process trigger:** Assay or boiling-curve description → characterization → blend/heating/separation flowsheet.

**Objective:** Treat characterized materials as reproducible model inputs rather than unidentified chemicals.

**Inputs/assumptions:** Original assay, stated test/basis conventions, cut definitions, characterization methods, estimated properties and provenance.

**Required observable behavior:** Create a reproducible component representation and use it in selected thermal/separation calculations. Retain the link to the originating assay and all estimation assumptions.

**Primary requirements:** [FR-DAT-05](#fr-dat-05), [FR-DAT-01](#fr-dat-01), [FR-CFG-04](#fr-cfg-04), [FR-MAT-01](#fr-mat-01), [FR-LIF-04](#fr-lif-04), [FR-LIF-07](#fr-lif-07).

**Cross-cutting scope rules:** X08, X10, X16, X17, X21.

**AT-SA-15-P:** Characterize a pinned assay into declared cuts, reconcile its quantities, perform supported thermal/separation use, and save/reconstruct the assay and generated property definitions.

**AT-SA-15-N:** A PVT-only pseudo route cannot pass heating/PH coverage; missing molecular structure does not authorize invented exact atom/registry identities.

**Failure obligation inherited from scope:** Do not invent a registry identity, atom balance or exact molecular structure for a cut; do not claim support for an uncharacterized assay.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-16"></a>
### SA-16 / SC-16: Black-oil or other reduced petroleum representation

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-16.

**Process trigger:** Bulk oil/gas/water description → pressure/temperature changes → supported surface-separation calculation.

**Objective:** Permit a useful engineering representation without falsely promising molecular-resolution equilibrium.

**Inputs/assumptions:** Model-specific bulk quantities; standard-condition conventions; empirical correlations; declared supported outputs and operating envelope.

**Required observable behavior:** Keep this representation distinct from an assay-characterized compositional mixture. Expose only calculations justified by its data and correlations.

**Primary requirements:** [FR-MAT-02](#fr-mat-02), [FR-STA-08](#fr-sta-08), [FR-CFG-03](#fr-cfg-03), [FR-DAT-07](#fr-dat-07), [FR-FLW-06](#fr-flw-06).

**Cross-cutting scope rules:** X09, X10, X13, X21.

**AT-SA-16-P:** Run a pinned reduced petroleum model using its actual/standard-condition conventions and explicitly limited partition/property operations.

**AT-SA-16-N:** Detailed molecular chemical potentials or a unique detailed composition remain unsupported unless additional justified characterization and mapping are supplied.

**Failure obligation inherited from scope:** No silent conversion from bulk oil characterization to an invented detailed molecular composition.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-17"></a>
### SA-17 / SC-17: Specified-conversion reaction

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-17.

**Process trigger:** Reactant feeds → conversion reactor → cooling/separation.

**Objective:** Separate process-imposed reaction extent from equilibrium prediction.

**Inputs/assumptions:** Stoichiometry, selected conversion or extent, phase participation, feed composition and an explicit heat-accounting convention.

**Required observable behavior:** Calculate resulting amounts and thermal requirements under the unit-imposed reaction extent. Treat conversion as an input assumption, not a computed equilibrium state.

**Primary requirements:** [FR-CHM-01](#fr-chm-01), [FR-CHM-02](#fr-chm-02), [FR-CHM-05](#fr-chm-05), [FR-MAT-08](#fr-mat-08), [FR-EQL-02](#fr-eql-02), [FR-RES-03](#fr-res-03).

**Cross-cutting scope rules:** X04, X08, X11, X12.

**AT-SA-17-P:** Apply a feasible fixed conversion with explicit reaction sequence and thermal mode; verify stoichiometry, mass/elements where defined, and exactly-once reaction energy.

**AT-SA-17-N:** An infeasible fixed conversion must fail. A separately authorized reactant-limited alternative is qualified and cannot pass the original strict fixture.

**Failure obligation inherited from scope:** Reject impossible conversion or unbalanced specified chemistry rather than clipping negative product quantities.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-18"></a>
### SA-18 / SC-18: Chemical-equilibrium reaction

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-18.

**Process trigger:** Specified reactant mixture → isothermal or energy-constrained equilibrium reactor → outlet.

**Objective:** Support changes in chemical species in addition to redistribution among phases.

**Inputs/assumptions:** Admissible species, conserved quantities, reaction-equilibrium or chemical-potential formulation, phase policy, and thermal constraints.

**Required observable behavior:** Resolve equilibrium under declared closed/open-system conditions. Keep phase equilibrium, chemical equilibrium and reaction inhibition separately identifiable.

**Primary requirements:** [FR-CHM-03](#fr-chm-03), [FR-CHM-05](#fr-chm-05), [FR-CHM-07](#fr-chm-07), [FR-MAT-08](#fr-mat-08), [FR-CFG-07](#fr-cfg-07).

**Cross-cutting scope rules:** X07, X11, X12, X20.

**AT-SA-18-P:** Run a pinned isothermal or energy-constrained chemical-equilibrium fixture; verify species/phase restrictions, relevant conserved totals, and thermal conditions.

**AT-SA-18-N:** A fixed-species VLE result is not chemical equilibrium; undisclosed external exchange or expanded species scope fails the original problem.

**Failure obligation inherited from scope:** A vapor–liquid flash with fixed chemical species totals is not accepted as chemical-equilibrium capability.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-19"></a>
### SA-19 / SC-19: Kinetically controlled reaction

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-19.

**Process trigger:** Feed → stirred or distributed reactor → outlet.

**Objective:** Allow finite-rate chemistry to consume thermophysical properties without the property service imposing equilibrium.

**Inputs/assumptions:** Rate law, basis and units, participating phases, material state, and unit-owned residence time/geometry/transport assumptions.

**Required observable behavior:** Supply activities, concentrations or other declared rate-law inputs, as well as compatible thermal properties. Evaluate local states at required internal locations.

**Primary requirements:** [FR-CHM-01](#fr-chm-01), [FR-CHM-04](#fr-chm-04), [FR-PRP-01](#fr-prp-01), [FR-PRP-05](#fr-prp-05), [FR-STA-01](#fr-sta-01).

**Cross-cutting scope rules:** X09, X12, X14, X19.

**AT-SA-19-P:** Use matched rate/property bases in a selected reactor formulation, evaluate required internal states, and close accepted balances as residence time changes.

**AT-SA-19-N:** A finite-rate request cannot be replaced with equilibrium, and an unqualified concentration conversion cannot feed the rate expression.

**Failure obligation inherited from scope:** Do not replace a finite-rate reaction by equilibrium because that is the only supported backend operation.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-20"></a>
### SA-20 / SC-20: Reactive separation

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-20.

**Process trigger:** Reacting liquid/vapor system → reactive contacting or equilibrium-stage separation → products.

**Objective:** Test that chemical and phase transformations can coexist in one local material description.

**Inputs/assumptions:** A documented compatible reacting-mixture fixture; declared reaction mode, stage assumptions, phase-transfer eligibility and energy conventions.

**Required observable behavior:** Represent simultaneous or explicitly approximated reaction/phase-equilibrium coupling. Keep the closure choice visible and compatible with the unit formulation.

**Primary requirements:** [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-EQL-08](#fr-eql-08), [FR-FLW-07](#fr-flw-07), [FR-RUN-08](#fr-run-08).

**Cross-cutting scope rules:** X07, X11, X12, X14.

**AT-SA-20-P:** Run one compatible reactive stage/separation formulation and jointly verify phase, chemistry, energy, and stage/whole-unit conditions.

**AT-SA-20-N:** Separate chemistry and flash successes that do not satisfy the coupled constraints do not establish reactive-separation success.

**Failure obligation inherited from scope:** Do not silently sequence incompatible chemistry and flash models and label the result fully coupled equilibrium.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-21"></a>
### SA-21 / SC-21: Electrolyte mixing and neutralization

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-21.

**Process trigger:** Acid/base/salt feeds → mixer or neutralizer → aqueous outlet.

**Objective:** Support reporting composition and internal species composition as different legitimate representations.

**Inputs/assumptions:** Apparent feed description, true species definition, charge and elemental information, thermodynamic data and reference conventions.

**Required observable behavior:** Resolve speciation under the declared constraints; expose reporting quantities without summing apparent and true species as separate physical inventories.

**Primary requirements:** [FR-CHM-06](#fr-chm-06), [FR-CHM-03](#fr-chm-03), [FR-CHM-05](#fr-chm-05), [FR-MAT-08](#fr-mat-08), [FR-FLW-06](#fr-flw-06).

**Cross-cutting scope rules:** X01, X08, X11, X12, X13.

**AT-SA-21-P:** Map an apparent electrolyte feed to true species, resolve the selected neutralization/speciation problem, and report pH/activity conventions without duplicate material.

**AT-SA-21-N:** Charge inconsistency, double-counted apparent/true species, or unsupported caloric completion prevents the affected claimed result.

**Failure obligation inherited from scope:** Inconsistent charge or incomplete reaction/species data produces a specific diagnostic rather than an unexplained normalization.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-22"></a>
### SA-22 / SC-22: Reactive gas absorption into aqueous liquid

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-22.

**Process trigger:** Gas feed plus aqueous solvent → absorber → gas and loaded-liquid products.

**Objective:** Combine gas transfer, dissolved-species chemistry and heat effects in a flowsheet context.

**Inputs/assumptions:** Gas and aqueous definitions, speciation/reaction rules, transfer conventions, thermal properties and a chosen equilibrium-stage or rate-based unit closure.

**Required observable behavior:** Account for gas entering or leaving the liquid and for changes in dissolved species. Keep gas reservoirs or added titrants explicit when constrained chemistry requires them.

**Primary requirements:** [FR-CHM-07](#fr-chm-07), [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-CFG-04](#fr-cfg-04), [FR-FLW-08](#fr-flw-08).

**Cross-cutting scope rules:** X09, X11, X12, X13, X20.

**AT-SA-22-P:** Combine gas transfer and aqueous reaction under a selected contacting formulation, account for every declared gas/titrant exchange, and close energy across representations.

**AT-SA-22-N:** A speciation-only result cannot pass an energy-balanced absorber fixture; no hidden gas reservoir or implicit titrant is allowed.

**Failure obligation inherited from scope:** A speciation-only result with no defensible caloric method does not qualify as an energy-balanced absorber calculation.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-23"></a>
### SA-23 / SC-23: Inert solids carried with fluid

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-23.

**Process trigger:** Liquid/gas plus inert solids → heating/transport → mechanical separation.

**Objective:** Represent a solid-bearing material without forcing the solids into a chemical-equilibrium calculation.

**Inputs/assumptions:** Solid identities or empirical material descriptors, amounts, phase-specific caloric/density data, participation policy and optional particle attributes.

**Required observable behavior:** Carry the solids through balances; include their thermal contribution when required; distinguish mechanical separation from precipitation. Let equipment models own settling/filtering closures.

**Primary requirements:** [FR-MAT-02](#fr-mat-02), [FR-MAT-08](#fr-mat-08), [FR-CFG-07](#fr-cfg-07), [FR-PRP-04](#fr-prp-04), [FR-FLW-03](#fr-flw-03).

**Cross-cutting scope rules:** X01, X07, X09, X22.

**AT-SA-23-P:** Carry equilibrium-inactive solid material through fluid heating and mechanical separation, including its justified caloric contribution and any declared bulk closure.

**AT-SA-23-N:** An inert solid cannot disappear into an equilibrium routine or be assigned fictitious vapor pressure/molar identity to fit its API.

**Failure obligation inherited from scope:** Do not assign a fictitious vapor pressure or molar basis to an empirical solid merely to fit a fluid API.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-24"></a>
### SA-24 / SC-24: Crystallization and precipitation

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-24.

**Process trigger:** Solution → cooling/evaporation or reagent addition → crystal-bearing material → separation.

**Objective:** Distinguish equilibrium solid formation from inert-solid transport and from crystal growth kinetics.

**Inputs/assumptions:** Dissolved species/components, admissible solid identities, compatible solution/solid data, thermal constraints and phase policy.

**Required observable behavior:** Determine equilibrium solid quantities or report the selected constrained calculation. Keep particle-size distribution and nucleation/growth modeling external unless explicitly added.

**Primary requirements:** [FR-EQL-08](#fr-eql-08), [FR-EQL-06](#fr-eql-06), [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-STA-06](#fr-sta-06).

**Cross-cutting scope rules:** X05, X07, X11, X12, X22.

**AT-SA-24-P:** Resolve a bounded compatible precipitation fixture with named solid forms, quantify present solids, check conserved totals and thermal targets, and assess absent candidates as declared.

**AT-SA-24-N:** A saturation index is not crystal size or kinetic yield; unsupported multicomponent solid capability remains a specific rejection.

**Failure obligation inherited from scope:** Do not treat a saturation calculation alone as a prediction of crystal size or kinetic yield.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-25"></a>
### SA-25 / SC-25: Gas–solid chemical transformation

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-25.

**Process trigger:** Gas and reactive solid → contactor/reactor → gas and solid products.

**Objective:** Ensure reactive solids are not represented as inert carrier mass or dissolved fluid by default.

**Inputs/assumptions:** Solid reactant/product identities, gas species, reaction mode, compatible solid/gas caloric data and unit-owned contact/transport assumptions.

**Required observable behavior:** Evaluate the requested reaction-equilibrium or kinetic property relationships for the declared phases. Permit unreacted and product solids to coexist as distinct entities.

**Primary requirements:** [FR-CHM-01](#fr-chm-01), [FR-CHM-04](#fr-chm-04), [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-CFG-07](#fr-cfg-07).

**Cross-cutting scope rules:** X07, X08, X11, X12.

**AT-SA-25-P:** Run a bounded gas–solid reaction formulation with distinct reactant/product solids, required thermal data, and explicit equilibrium or kinetic limits.

**AT-SA-25-N:** Unsupported polymorph/solid-solution behavior cannot silently become an ideal-liquid approximation or inert carrier.

**Failure obligation inherited from scope:** Unsupported solid-solution or polymorph behavior remains explicit rather than being approximated as an ideal liquid without authorization.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-26"></a>
### SA-26 / SC-26: Rate-based nonequilibrium contacting

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-26.

**Process trigger:** Gas/liquid feeds → spatially resolved contactor → separate outlets.

**Objective:** Support phase-specific bulk states and local interface relationships without automatically equilibrating the entire unit.

**Inputs/assumptions:** Separate phase temperatures and compositions where the chosen unit requires them; pressure/mechanical convention; transfer and interface assumptions.

**Required observable behavior:** Evaluate each bulk phase independently, plus interfacial equilibrium or driving-force properties as requested. The unit supplies interfacial area and transfer coefficients or correlations.

**Primary requirements:** [FR-FLW-08](#fr-flw-08), [FR-STA-03](#fr-sta-03), [FR-PRP-01](#fr-prp-01), [FR-PRP-03](#fr-prp-03), [FR-RUN-08](#fr-run-08).

**Cross-cutting scope rules:** X07, X09, X14, X19, X22.

**AT-SA-26-P:** Keep distinct bulk-phase states and required interface relations within a selected rate-based unit; verify transfer/energy reconciliation and property coverage.

**AT-SA-26-N:** An automatic bulk reflash or efficiency-corrected equilibrium-stage model does not pass the full specified SC-26 formulation.

**Failure obligation inherited from scope:** An equilibrium flash is not substituted for a rate-based contactor or used to overwrite its phase states.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-27"></a>
### SA-27 / SC-27: Adsorption and membrane state extensions

**Profile:** P3. **Accountable owner:** Extension semantics. **Source:** B1 §5, SC-27.

**Process trigger:** Bulk feed → adsorber or membrane contacting location → separate material domains.

**Objective:** Reserve a concrete extension path for surface-bound material and selective transfer.

**Inputs/assumptions:** Bulk material states; optional adsorbed/surface inventory, loading basis and site definition; membrane-side identities and applicable chemical-potential conventions.

**Required observable behavior:** Represent relevant bulk and nonbulk states without labeling every surface inventory as an ordinary liquid phase. Keep isotherms, permeation laws and equipment transport relationships explicitly assigned.

**Primary requirements:** [FR-EXT-01](#fr-ext-01), [FR-MAT-03](#fr-mat-03), [FR-MAT-08](#fr-mat-08), [FR-STA-03](#fr-sta-03), [FR-PRP-07](#fr-prp-07).

**Cross-cutting scope rules:** X07, X13, X14, X22.

**AT-SA-27-P:** Walk through bulk, surface storage, and selective-transfer domains with stated loading/site/basis conventions and a conserved-material transfer.

**AT-SA-27-N:** A label or arbitrary metadata field with no state, transfer, and conservation meaning fails representability; numerical support remains unclaimed.

**Failure obligation inherited from scope:** A generic untyped metadata attachment is insufficient evidence that this scenario is representable.

**Evidence gate:** Concrete representability walkthrough; E/V deferred.

**Status:** Specified, not exercised.

<a id="sa-28"></a>
### SA-28 / SC-28: Heat exchange between different property packages

**Profile:** P1. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-28.

**Process trigger:** Process-fluid circuit ↔ heat exchanger ↔ independent water/refrigerant circuit.

**Objective:** Allow different thermodynamic configurations in one flowsheet without implying material translation.

**Inputs/assumptions:** Independent material definitions and packages per circuit; separate reference conventions; declared heat-loss/work conventions.

**Required observable behavior:** Compute each circuit’s enthalpy changes independently and couple them through heat transfer. Do not require common component sets or equal numerical enthalpy zeroes.

**Primary requirements:** [FR-FLW-04](#fr-flw-04), [FR-CFG-06](#fr-cfg-06), [FR-PRP-02](#fr-prp-02), [FR-PRP-07](#fr-prp-07), [FR-RES-03](#fr-res-03).

**Cross-cutting scope rules:** X01, X11, X14, X18.

**AT-SA-28-P:** Couple two independent material packages through a heat exchanger, preserving within-side energy differences and no species transfer across the wall.

**AT-SA-28-N:** Differing absolute reference zeroes cannot by themselves trigger material translation; a required caloric failure cannot be hidden as optional.

**Failure obligation inherited from scope:** Do not invoke a chemical component translator merely because two connected equipment sides use different packages.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-29"></a>
### SA-29 / SC-29: Material transfer across a property-package boundary

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-29.

**Process trigger:** Material stream described by package A → explicit boundary treatment → region using package B.

**Objective:** Handle different models for the same physical material without a hidden energy or identity discontinuity.

**Inputs/assumptions:** Common or mapped material identities; both package conventions; chosen quantities to preserve; known reference offsets and explicit discrepancy policy.

**Required observable behavior:** Separate a pure reference-state conversion from disagreement between physical models. Define which state is recomputed and under which constraints.

**Primary requirements:** [FR-FLW-05](#fr-flw-05), [FR-PRP-07](#fr-prp-07), [FR-STA-03](#fr-sta-03), [FR-RES-07](#fr-res-07).

**Cross-cutting scope rules:** X02, X11, X13, X20.

**AT-SA-29-P:** Test a declared T/P-preserving boundary and a pressure/corrected-enthalpy-preserving boundary separately; report reference corrections and residual model disagreement.

**AT-SA-29-N:** Do not simultaneously enforce incompatible T,P,composition,H by hidden heat or by relabeling model discrepancy as a constant reference offset.

**Failure obligation inherited from scope:** Do not promise simultaneous preservation of incompatible temperature, pressure, composition and enthalpy predictions from different models.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-30"></a>
### SA-30 / SC-30: Translation between material representations

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-30.

**Process trigger:** Detailed/apparent material description → explicit representation map → lumped/true-species description.

**Objective:** Permit heterogeneous modeling fidelity while making information loss and conserved quantities explicit.

**Inputs/assumptions:** Source and target representations; mapping rules; feasible conserved quantities; assumptions needed for any reverse reconstruction.

**Required observable behavior:** Translate without treating an irreversible aggregation as a bijection. Preserve provenance and distinguish reporting conversion from physical re-equilibration.

**Primary requirements:** [FR-FLW-06](#fr-flw-06), [FR-MAT-07](#fr-mat-07), [FR-MAT-08](#fr-mat-08), [FR-CHM-06](#fr-chm-06), [FR-LIF-06](#fr-lif-06).

**Cross-cutting scope rules:** X01, X12, X13, X17.

**AT-SA-30-P:** Translate a reversible identity/order case and an irreversible aggregation case; verify the appropriate balances and document information lost.

**AT-SA-30-N:** An irreversible aggregation has no unique inverse without extra declared information; apparent-to-true mapping cannot double-count material.

**Failure obligation inherited from scope:** Missing information for unlumping is not synthesized as a unique composition.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-31"></a>
### SA-31 / SC-31: Mass-based empirical and nonconventional materials

**Profile:** P2. **Accountable owner:** Process-model integration, with result qualification. **Source:** B1 §5, SC-31.

**Process trigger:** Assay-described biomass, sludge, polymer granules or other engineering material → heating/mixing/separation.

**Objective:** Support useful limited-property process models without pretending every material is a known molecular mixture.

**Inputs/assumptions:** Declared mass basis; empirical density/heat-capacity/enthalpy relationships as available; composition or assay descriptors and their validity.

**Required observable behavior:** Answer only supported requests. Permit mass/energy calculations where justified; require additional characterization before molar, elemental or chemical-potential calculations.

**Primary requirements:** [FR-MAT-02](#fr-mat-02), [FR-MAT-03](#fr-mat-03), [FR-CFG-03](#fr-cfg-03), [FR-PRP-02](#fr-prp-02), [FR-PRP-04](#fr-prp-04).

**Cross-cutting scope rules:** X01, X08, X09, X13, X22.

**AT-SA-31-P:** Complete a mass/energy fixture using only justified empirical properties, with no assumed molecular weight; show its restricted property readiness.

**AT-SA-31-N:** Reject an unsupported molecular or elemental request specifically while keeping its supported empirical heating calculation usable.

**Failure obligation inherited from scope:** Do not use arbitrary molecular weight, critical constants or zero-valued missing properties to satisfy a universal fluid contract.

**Evidence gate:** Pinned numerical/data fixture and actual unit/process integration before E; bounded physical/reference/operational evidence before V.

**Status:** Specified, not exercised.

<a id="sa-32"></a>
### SA-32 / SC-32: Polymer distributions and material attributes

**Profile:** P3. **Accountable owner:** Extension semantics. **Source:** B1 §5, SC-32.

**Process trigger:** Polymer/solvent feed → mixing/devolatilization location → material with retained distribution descriptors.

**Objective:** Reserve support for materials whose characterization is not exhausted by one molecular identity and mole fraction.

**Inputs/assumptions:** A documented distribution or moment representation; basis; polymer/solvent descriptors; model-specific parameters and permitted reductions.

**Required observable behavior:** Associate distributions and derived averages with their material definition. Distinguish repeat units, molecules, segments and mass bases when relevant.

**Primary requirements:** [FR-EXT-02](#fr-ext-02), [FR-MAT-02](#fr-mat-02), [FR-MAT-03](#fr-mat-03), [FR-FLW-06](#fr-flw-06), [FR-LIF-04](#fr-lif-04).

**Cross-cutting scope rules:** X01, X09, X13, X22.

**AT-SA-32-P:** Walk through transfer/mixing of a declared polymer distribution or moment description, preserving or explicitly reducing its attributes and material basis.

**AT-SA-32-N:** A single average molecular weight cannot be claimed to preserve a full distribution, and numerical detailed rheology/equilibrium remains unclaimed.

**Failure obligation inherited from scope:** A single arbitrary mean molecular weight is not accepted as a lossless representation of an entire distribution.

**Evidence gate:** Concrete representability walkthrough; E/V deferred.

**Status:** Specified, not exercised.

<a id="sa-33"></a>
### SA-33 / SC-33: Inventory-based state and dynamic compatibility

**Profile:** P3. **Accountable owner:** Extension semantics. **Source:** B1 §5, SC-33.

**Process trigger:** Vessel inventory at an identified time/location → material-state evaluation under volume/energy constraints.

**Objective:** Keep future dynamics possible without including a time integrator in the present thermodynamics scope.

**Inputs/assumptions:** Component/species inventories or another declared material amount basis; total internal energy and total volume, or another admissible independent specification.

**Required observable behavior:** Represent inventory separately from flow. Permit a state request based on total quantities with the required amount information; the process model owns accumulation balances, temporal evolution and events.

**Primary requirements:** [FR-EXT-03](#fr-ext-03), [FR-MAT-03](#fr-mat-03), [FR-MAT-04](#fr-mat-04), [FR-STA-01](#fr-sta-01), [FR-STA-07](#fr-sta-07).

**Cross-cutting scope rules:** X01, X02, X03, X14, X24.

**AT-SA-33-P:** Walk through a nonempty inventory state with material amounts and total U,V, and an empty-inventory variant with explicit undefined/retained intensive semantics.

**AT-SA-33-N:** Do not require a fictitious flow or infer density/unique state from total volume and energy without adequate material information.

**Failure obligation inherited from scope:** Do not infer density from total volume alone without material amount or assume a steady-flow enthalpy request is always sufficient.

**Evidence gate:** Concrete representability walkthrough; E/V deferred.

**Status:** Specified, not exercised.

<a id="sa-34"></a>
### SA-34 / SC-34: Restricted-equilibrium and metastable-state requests

**Profile:** P3. **Accountable owner:** Extension semantics. **Source:** B1 §5, SC-34.

**Process trigger:** Process location with an explicit phase or reaction restriction → constrained property/state evaluation.

**Objective:** Separate user-imposed physical approximations from solver heuristics and global-equilibrium claims.

**Inputs/assumptions:** Named excluded phases/reactions or requested branch; rationale; validity restrictions; expected diagnostics.

**Required observable behavior:** Represent frozen chemistry, suppressed solids or a metastable branch where a provider supports it. Keep this distinct from an initial guess and from equilibrium over all allowed candidates.

**Primary requirements:** [FR-EXT-04](#fr-ext-04), [FR-CFG-05](#fr-cfg-05), [FR-EQL-06](#fr-eql-06), [FR-EQL-07](#fr-eql-07), [FR-RUN-05](#fr-run-05).

**Cross-cutting scope rules:** X05, X06, X07, X16.

**AT-SA-34-P:** Walk through frozen chemistry, suppressed solids, or a supported metastable branch as differently qualified requests with their restrictions retained.

**AT-SA-34-N:** Removing a restriction invalidates the old result for the new request; altered-phase retry cannot acquire unrestricted equilibrium status.

**Failure obligation inherited from scope:** A fallback that disables a phase or reaction without changing the declared problem is forbidden.

**Evidence gate:** Concrete representability walkthrough; E/V deferred.

**Status:** Specified, not exercised.

## 5. Cross-cutting, concept, research, and workflow traceability

These crosswalks are navigational mappings, not proof of implementation. They are generated from the same register as the requirement cards. Reverse links are included in the JSON. A primary map identifies the most direct requirements; applicable H clauses are inherited. Source observations are not retroactively changed into normative source requirements.

### 5.1 Scenario → primary functional requirements

| B1 SC | Primary functional requirements |
| --- | --- |
| SC-01 | [FR-MAT-05](#fr-mat-05), [FR-MAT-06](#fr-mat-06), [FR-MAT-08](#fr-mat-08), [FR-FLW-02](#fr-flw-02), [FR-RES-03](#fr-res-03) |
| SC-02 | [FR-CFG-04](#fr-cfg-04), [FR-EQL-02](#fr-eql-02), [FR-PRP-02](#fr-prp-02), [FR-PRP-03](#fr-prp-03), [FR-FLW-01](#fr-flw-01), [FR-FLW-04](#fr-flw-04) |
| SC-03 | [FR-PRP-01](#fr-prp-01), [FR-PRP-03](#fr-prp-03), [FR-CFG-03](#fr-cfg-03), [FR-FLW-01](#fr-flw-01), [FR-EQL-06](#fr-eql-06) |
| SC-04 | [FR-EQL-03](#fr-eql-03), [FR-EQL-02](#fr-eql-02), [FR-CFG-04](#fr-cfg-04), [FR-STA-04](#fr-sta-04), [FR-PRP-05](#fr-prp-05), [FR-FLW-01](#fr-flw-01) |
| SC-05 | [FR-EQL-01](#fr-eql-01), [FR-EQL-06](#fr-eql-06), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-FLW-03](#fr-flw-03), [FR-RES-03](#fr-res-03) |
| SC-06 | [FR-EQL-02](#fr-eql-02), [FR-STA-03](#fr-sta-03), [FR-RES-02](#fr-res-02), [FR-RES-04](#fr-res-04), [FR-RES-05](#fr-res-05), [FR-RUN-01](#fr-run-01) |
| SC-07 | [FR-STA-01](#fr-sta-01), [FR-STA-04](#fr-sta-04), [FR-PRP-01](#fr-prp-01), [FR-FLW-07](#fr-flw-07), [FR-RUN-02](#fr-run-02), [FR-RUN-08](#fr-run-08), [FR-RES-04](#fr-res-04) |
| SC-08 | [FR-DAT-02](#fr-dat-02), [FR-DAT-03](#fr-dat-03), [FR-CFG-02](#fr-cfg-02), [FR-CFG-04](#fr-cfg-04), [FR-EQL-07](#fr-eql-07), [FR-RUN-05](#fr-run-05) |
| SC-09 | [FR-CFG-02](#fr-cfg-02), [FR-CFG-07](#fr-cfg-07), [FR-PRP-01](#fr-prp-01), [FR-PRP-07](#fr-prp-07), [FR-STA-08](#fr-sta-08), [FR-FLW-01](#fr-flw-01) |
| SC-10 | [FR-EQL-08](#fr-eql-08), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-PRP-03](#fr-prp-03), [FR-FLW-03](#fr-flw-03) |
| SC-11 | [FR-CFG-07](#fr-cfg-07), [FR-EQL-01](#fr-eql-01), [FR-EQL-08](#fr-eql-08), [FR-EQL-06](#fr-eql-06), [FR-RES-03](#fr-res-03) |
| SC-12 | [FR-EQL-04](#fr-eql-04), [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03), [FR-STA-02](#fr-sta-02), [FR-STA-08](#fr-sta-08) |
| SC-13 | [FR-MAT-01](#fr-mat-01), [FR-EQL-04](#fr-eql-04), [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03), [FR-RUN-08](#fr-run-08), [FR-LIF-07](#fr-lif-07) |
| SC-14 | [FR-MAT-02](#fr-mat-02), [FR-STA-08](#fr-sta-08), [FR-EQL-04](#fr-eql-04), [FR-EQL-02](#fr-eql-02), [FR-CFG-02](#fr-cfg-02) |
| SC-15 | [FR-DAT-05](#fr-dat-05), [FR-DAT-01](#fr-dat-01), [FR-CFG-04](#fr-cfg-04), [FR-MAT-01](#fr-mat-01), [FR-LIF-04](#fr-lif-04), [FR-LIF-07](#fr-lif-07) |
| SC-16 | [FR-MAT-02](#fr-mat-02), [FR-STA-08](#fr-sta-08), [FR-CFG-03](#fr-cfg-03), [FR-DAT-07](#fr-dat-07), [FR-FLW-06](#fr-flw-06) |
| SC-17 | [FR-CHM-01](#fr-chm-01), [FR-CHM-02](#fr-chm-02), [FR-CHM-05](#fr-chm-05), [FR-MAT-08](#fr-mat-08), [FR-EQL-02](#fr-eql-02), [FR-RES-03](#fr-res-03) |
| SC-18 | [FR-CHM-03](#fr-chm-03), [FR-CHM-05](#fr-chm-05), [FR-CHM-07](#fr-chm-07), [FR-MAT-08](#fr-mat-08), [FR-CFG-07](#fr-cfg-07) |
| SC-19 | [FR-CHM-01](#fr-chm-01), [FR-CHM-04](#fr-chm-04), [FR-PRP-01](#fr-prp-01), [FR-PRP-05](#fr-prp-05), [FR-STA-01](#fr-sta-01) |
| SC-20 | [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-EQL-08](#fr-eql-08), [FR-FLW-07](#fr-flw-07), [FR-RUN-08](#fr-run-08) |
| SC-21 | [FR-CHM-06](#fr-chm-06), [FR-CHM-03](#fr-chm-03), [FR-CHM-05](#fr-chm-05), [FR-MAT-08](#fr-mat-08), [FR-FLW-06](#fr-flw-06) |
| SC-22 | [FR-CHM-07](#fr-chm-07), [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-CFG-04](#fr-cfg-04), [FR-FLW-08](#fr-flw-08) |
| SC-23 | [FR-MAT-02](#fr-mat-02), [FR-MAT-08](#fr-mat-08), [FR-CFG-07](#fr-cfg-07), [FR-PRP-04](#fr-prp-04), [FR-FLW-03](#fr-flw-03) |
| SC-24 | [FR-EQL-08](#fr-eql-08), [FR-EQL-06](#fr-eql-06), [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-STA-06](#fr-sta-06) |
| SC-25 | [FR-CHM-01](#fr-chm-01), [FR-CHM-04](#fr-chm-04), [FR-CHM-08](#fr-chm-08), [FR-CHM-05](#fr-chm-05), [FR-CFG-07](#fr-cfg-07) |
| SC-26 | [FR-FLW-08](#fr-flw-08), [FR-STA-03](#fr-sta-03), [FR-PRP-01](#fr-prp-01), [FR-PRP-03](#fr-prp-03), [FR-RUN-08](#fr-run-08) |
| SC-27 | [FR-EXT-01](#fr-ext-01), [FR-MAT-03](#fr-mat-03), [FR-MAT-08](#fr-mat-08), [FR-STA-03](#fr-sta-03), [FR-PRP-07](#fr-prp-07) |
| SC-28 | [FR-FLW-04](#fr-flw-04), [FR-CFG-06](#fr-cfg-06), [FR-PRP-02](#fr-prp-02), [FR-PRP-07](#fr-prp-07), [FR-RES-03](#fr-res-03) |
| SC-29 | [FR-FLW-05](#fr-flw-05), [FR-PRP-07](#fr-prp-07), [FR-STA-03](#fr-sta-03), [FR-RES-07](#fr-res-07) |
| SC-30 | [FR-FLW-06](#fr-flw-06), [FR-MAT-07](#fr-mat-07), [FR-MAT-08](#fr-mat-08), [FR-CHM-06](#fr-chm-06), [FR-LIF-06](#fr-lif-06) |
| SC-31 | [FR-MAT-02](#fr-mat-02), [FR-MAT-03](#fr-mat-03), [FR-CFG-03](#fr-cfg-03), [FR-PRP-02](#fr-prp-02), [FR-PRP-04](#fr-prp-04) |
| SC-32 | [FR-EXT-02](#fr-ext-02), [FR-MAT-02](#fr-mat-02), [FR-MAT-03](#fr-mat-03), [FR-FLW-06](#fr-flw-06), [FR-LIF-04](#fr-lif-04) |
| SC-33 | [FR-EXT-03](#fr-ext-03), [FR-MAT-03](#fr-mat-03), [FR-MAT-04](#fr-mat-04), [FR-STA-01](#fr-sta-01), [FR-STA-07](#fr-sta-07) |
| SC-34 | [FR-EXT-04](#fr-ext-04), [FR-CFG-05](#fr-cfg-05), [FR-EQL-06](#fr-eql-06), [FR-EQL-07](#fr-eql-07), [FR-RUN-05](#fr-run-05) |

### 5.2 Cross-cutting behavior → functional requirements

| B1 X | Primary functional requirements |
| --- | --- |
| X01 | [FR-MAT-03](#fr-mat-03), [FR-MAT-05](#fr-mat-05), [FR-STA-08](#fr-sta-08), [FR-PRP-07](#fr-prp-07) |
| X02 | [FR-STA-02](#fr-sta-02), [FR-STA-04](#fr-sta-04), [FR-EQL-04](#fr-eql-04) |
| X03 | [FR-MAT-04](#fr-mat-04), [FR-STA-06](#fr-sta-06), [FR-EXT-03](#fr-ext-03) |
| X04 | [FR-MAT-05](#fr-mat-05), [FR-MAT-08](#fr-mat-08), [FR-STA-07](#fr-sta-07), [FR-CHM-02](#fr-chm-02) |
| X05 | [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-EQL-07](#fr-eql-07) |
| X06 | [FR-RUN-01](#fr-run-01), [FR-EQL-07](#fr-eql-07), [FR-PRP-06](#fr-prp-06) |
| X07 | [FR-CFG-05](#fr-cfg-05), [FR-STA-03](#fr-sta-03), [FR-EQL-06](#fr-eql-06), [FR-CHM-04](#fr-chm-04), [FR-RUN-05](#fr-run-05) |
| X08 | [FR-DAT-01](#fr-dat-01), [FR-DAT-02](#fr-dat-02), [FR-DAT-03](#fr-dat-03), [FR-DAT-04](#fr-dat-04), [FR-DAT-05](#fr-dat-05) |
| X09 | [FR-GOV-02](#fr-gov-02), [FR-CFG-03](#fr-cfg-03), [FR-EQL-05](#fr-eql-05), [FR-PRP-02](#fr-prp-02), [FR-MAT-02](#fr-mat-02) |
| X10 | [FR-DAT-07](#fr-dat-07), [FR-RES-06](#fr-res-06), [FR-RES-08](#fr-res-08) |
| X11 | [FR-CFG-04](#fr-cfg-04), [FR-PRP-07](#fr-prp-07), [FR-CHM-05](#fr-chm-05), [FR-FLW-04](#fr-flw-04), [FR-FLW-05](#fr-flw-05) |
| X12 | [FR-MAT-08](#fr-mat-08), [FR-CHM-02](#fr-chm-02), [FR-CHM-03](#fr-chm-03), [FR-CHM-05](#fr-chm-05), [FR-RES-03](#fr-res-03) |
| X13 | [FR-MAT-01](#fr-mat-01), [FR-MAT-07](#fr-mat-07), [FR-CHM-06](#fr-chm-06), [FR-FLW-06](#fr-flw-06), [FR-LIF-02](#fr-lif-02) |
| X14 | [FR-STA-01](#fr-sta-01), [FR-CFG-06](#fr-cfg-06), [FR-FLW-07](#fr-flw-07), [FR-FLW-08](#fr-flw-08) |
| X15 | [FR-RUN-08](#fr-run-08), [FR-RES-04](#fr-res-04), [FR-RES-06](#fr-res-06) |
| X16 | [FR-LIF-01](#fr-lif-01), [FR-LIF-02](#fr-lif-02), [FR-LIF-03](#fr-lif-03), [FR-RES-05](#fr-res-05), [FR-RUN-03](#fr-run-03) |
| X17 | [FR-RES-07](#fr-res-07), [FR-LIF-04](#fr-lif-04), [FR-LIF-05](#fr-lif-05), [FR-LIF-06](#fr-lif-06), [FR-LIF-07](#fr-lif-07) |
| X18 | [FR-RUN-03](#fr-run-03), [FR-RUN-04](#fr-run-04), [FR-RUN-07](#fr-run-07) |
| X19 | [FR-PRP-05](#fr-prp-05), [FR-PRP-06](#fr-prp-06), [FR-FLW-07](#fr-flw-07) |
| X20 | [FR-STA-03](#fr-sta-03), [FR-CHM-07](#fr-chm-07), [FR-FLW-01](#fr-flw-01) |
| X21 | [FR-STA-08](#fr-sta-08), [FR-MAT-03](#fr-mat-03), [FR-DAT-05](#fr-dat-05) |
| X22 | [FR-PRP-03](#fr-prp-03), [FR-PRP-04](#fr-prp-04), [FR-FLW-03](#fr-flw-03) |
| X23 | [FR-RUN-06](#fr-run-06), [FR-RUN-07](#fr-run-07), [FR-RES-01](#fr-res-01), [FR-RES-02](#fr-res-02), [FR-RES-05](#fr-res-05), [FR-PRP-02](#fr-prp-02) |
| X24 | [FR-MAT-03](#fr-mat-03), [FR-STA-07](#fr-sta-07), [FR-EXT-03](#fr-ext-03) |

### 5.3 Concept obligation → functional requirements

| B1 C | Primary functional requirements |
| --- | --- |
| C01 | [FR-MAT-01](#fr-mat-01), [FR-MAT-02](#fr-mat-02), [FR-MAT-07](#fr-mat-07), [FR-MAT-08](#fr-mat-08), [FR-FLW-06](#fr-flw-06) |
| C02 | [FR-MAT-03](#fr-mat-03), [FR-MAT-04](#fr-mat-04), [FR-STA-07](#fr-sta-07), [FR-STA-08](#fr-sta-08) |
| C03 | [FR-STA-01](#fr-sta-01), [FR-STA-02](#fr-sta-02), [FR-STA-03](#fr-sta-03), [FR-STA-04](#fr-sta-04), [FR-FLW-07](#fr-flw-07) |
| C04 | [FR-CFG-07](#fr-cfg-07), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-FLW-03](#fr-flw-03) |
| C05 | [FR-GOV-04](#fr-gov-04), [FR-CFG-01](#fr-cfg-01), [FR-CFG-02](#fr-cfg-02), [FR-CFG-03](#fr-cfg-03) |
| C06 | [FR-DAT-01](#fr-dat-01), [FR-DAT-02](#fr-dat-02), [FR-DAT-03](#fr-dat-03), [FR-DAT-04](#fr-dat-04), [FR-DAT-05](#fr-dat-05), [FR-DAT-06](#fr-dat-06) |
| C07 | [FR-CFG-04](#fr-cfg-04), [FR-PRP-07](#fr-prp-07), [FR-CHM-05](#fr-chm-05), [FR-FLW-05](#fr-flw-05) |
| C08 | [FR-CFG-05](#fr-cfg-05), [FR-EQL-06](#fr-eql-06), [FR-CHM-03](#fr-chm-03), [FR-CHM-07](#fr-chm-07), [FR-CHM-08](#fr-chm-08) |
| C09 | [FR-PRP-03](#fr-prp-03), [FR-PRP-04](#fr-prp-04), [FR-FLW-01](#fr-flw-01), [FR-FLW-08](#fr-flw-08) |
| C10 | [FR-CFG-06](#fr-cfg-06), [FR-FLW-04](#fr-flw-04), [FR-FLW-05](#fr-flw-05), [FR-FLW-06](#fr-flw-06) |
| C11 | [FR-GOV-01](#fr-gov-01), [FR-GOV-02](#fr-gov-02), [FR-CFG-03](#fr-cfg-03), [FR-DAT-07](#fr-dat-07), [FR-PRP-05](#fr-prp-05), [FR-RES-01](#fr-res-01), [FR-RES-02](#fr-res-02), [FR-RES-08](#fr-res-08) |
| C12 | [FR-RUN-03](#fr-run-03), [FR-RUN-04](#fr-run-04), [FR-RES-05](#fr-res-05), [FR-RES-06](#fr-res-06), [FR-RES-07](#fr-res-07), [FR-LIF-01](#fr-lif-01), [FR-LIF-04](#fr-lif-04), [FR-LIF-07](#fr-lif-07) |
| C13 | [FR-EXT-01](#fr-ext-01), [FR-EXT-02](#fr-ext-02) |

### 5.4 Step-3 CR proposal → functional requirements

| B3 CR | Primary functional requirements |
| --- | --- |
| CR-01 | [FR-CFG-01](#fr-cfg-01), [FR-DAT-06](#fr-dat-06), [FR-RUN-03](#fr-run-03), [FR-LIF-04](#fr-lif-04) |
| CR-02 | [FR-STA-03](#fr-sta-03), [FR-PRP-01](#fr-prp-01), [FR-RES-04](#fr-res-04), [FR-CHM-04](#fr-chm-04) |
| CR-03 | [FR-EQL-05](#fr-eql-05), [FR-PRP-02](#fr-prp-02), [FR-RES-01](#fr-res-01) |
| CR-04 | [FR-GOV-01](#fr-gov-01), [FR-GOV-02](#fr-gov-02), [FR-CFG-03](#fr-cfg-03), [FR-GOV-05](#fr-gov-05) |
| CR-05 | [FR-STA-02](#fr-sta-02), [FR-RUN-01](#fr-run-01), [FR-RUN-02](#fr-run-02) |
| CR-06 | [FR-CFG-04](#fr-cfg-04), [FR-PRP-07](#fr-prp-07), [FR-CHM-05](#fr-chm-05), [FR-FLW-05](#fr-flw-05) |
| CR-07 | [FR-DAT-01](#fr-dat-01), [FR-DAT-02](#fr-dat-02), [FR-DAT-03](#fr-dat-03) |
| CR-08 | [FR-CFG-07](#fr-cfg-07), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-EQL-06](#fr-eql-06) |
| CR-09 | [FR-FLW-04](#fr-flw-04), [FR-FLW-05](#fr-flw-05), [FR-FLW-06](#fr-flw-06) |
| CR-10 | [FR-MAT-08](#fr-mat-08), [FR-CHM-03](#fr-chm-03), [FR-CHM-06](#fr-chm-06), [FR-CHM-07](#fr-chm-07), [FR-RES-03](#fr-res-03) |
| CR-11 | [FR-PRP-03](#fr-prp-03), [FR-PRP-04](#fr-prp-04) |
| CR-12 | [FR-RUN-03](#fr-run-03), [FR-RUN-04](#fr-run-04), [FR-RUN-07](#fr-run-07) |
| CR-13 | [FR-RUN-05](#fr-run-05), [FR-RES-01](#fr-res-01), [FR-RES-02](#fr-res-02), [FR-RES-05](#fr-res-05) |
| CR-14 | [FR-DAT-06](#fr-dat-06), [FR-LIF-04](#fr-lif-04), [FR-LIF-05](#fr-lif-05), [FR-LIF-07](#fr-lif-07) |
| CR-15 | [FR-PRP-05](#fr-prp-05), [FR-PRP-06](#fr-prp-06), [FR-MAT-07](#fr-mat-07) |
| CR-16 | [FR-MAT-02](#fr-mat-02), [FR-MAT-03](#fr-mat-03), [FR-CFG-03](#fr-cfg-03), [FR-DAT-05](#fr-dat-05) |

### 5.5 DWSIM workflow → functional requirements

| B2 WF | Primary functional requirements |
| --- | --- |
| WF-01 | [FR-GOV-03](#fr-gov-03), [FR-CFG-01](#fr-cfg-01), [FR-CFG-06](#fr-cfg-06), [FR-DAT-01](#fr-dat-01) |
| WF-02 | [FR-MAT-05](#fr-mat-05), [FR-MAT-06](#fr-mat-06), [FR-MAT-04](#fr-mat-04), [FR-LIF-08](#fr-lif-08) |
| WF-03 | [FR-DAT-01](#fr-dat-01), [FR-DAT-02](#fr-dat-02), [FR-DAT-03](#fr-dat-03), [FR-DAT-06](#fr-dat-06) |
| WF-04 | [FR-STA-02](#fr-sta-02), [FR-STA-03](#fr-sta-03), [FR-EQL-05](#fr-eql-05), [FR-RES-02](#fr-res-02) |
| WF-05 | [FR-CFG-05](#fr-cfg-05), [FR-EQL-06](#fr-eql-06), [FR-RUN-05](#fr-run-05), [FR-RES-01](#fr-res-01) |
| WF-06 | [FR-EQL-02](#fr-eql-02), [FR-RES-04](#fr-res-04), [FR-RES-05](#fr-res-05) |
| WF-07 | [FR-FLW-02](#fr-flw-02), [FR-FLW-03](#fr-flw-03), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06) |
| WF-08 | [FR-FLW-04](#fr-flw-04), [FR-PRP-02](#fr-prp-02), [FR-CFG-06](#fr-cfg-06) |
| WF-09 | [FR-CHM-01](#fr-chm-01), [FR-CHM-02](#fr-chm-02), [FR-CHM-05](#fr-chm-05), [FR-CHM-08](#fr-chm-08) |
| WF-10 | [FR-STA-01](#fr-sta-01), [FR-STA-04](#fr-sta-04), [FR-FLW-07](#fr-flw-07), [FR-FLW-08](#fr-flw-08), [FR-RUN-08](#fr-run-08) |
| WF-11 | [FR-RUN-08](#fr-run-08), [FR-RES-04](#fr-res-04), [FR-RES-06](#fr-res-06) |
| WF-12 | [FR-LIF-02](#fr-lif-02), [FR-MAT-06](#fr-mat-06), [FR-LIF-08](#fr-lif-08) |
| WF-13 | [FR-LIF-01](#fr-lif-01), [FR-LIF-03](#fr-lif-03), [FR-LIF-06](#fr-lif-06) |
| WF-14 | [FR-DAT-06](#fr-dat-06), [FR-LIF-04](#fr-lif-04), [FR-LIF-05](#fr-lif-05), [FR-LIF-07](#fr-lif-07) |
| WF-15 | [FR-GOV-02](#fr-gov-02), [FR-STA-03](#fr-sta-03), [FR-MAT-07](#fr-mat-07), [FR-FLW-05](#fr-flw-05) |

### 5.6 Comparative workflow → functional requirements

| B3 W | Primary functional requirements |
| --- | --- |
| W01 | [FR-DAT-01](#fr-dat-01), [FR-DAT-02](#fr-dat-02), [FR-CFG-01](#fr-cfg-01), [FR-CFG-02](#fr-cfg-02), [FR-CFG-03](#fr-cfg-03), [FR-CFG-04](#fr-cfg-04) |
| W02 | [FR-STA-03](#fr-sta-03), [FR-PRP-01](#fr-prp-01), [FR-PRP-05](#fr-prp-05), [FR-EQL-05](#fr-eql-05) |
| W03 | [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03), [FR-EQL-04](#fr-eql-04), [FR-CFG-04](#fr-cfg-04), [FR-RUN-01](#fr-run-01) |
| W04 | [FR-STA-01](#fr-sta-01), [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-EQL-08](#fr-eql-08), [FR-FLW-08](#fr-flw-08) |
| W05 | [FR-CHM-01](#fr-chm-01), [FR-CHM-03](#fr-chm-03), [FR-CHM-05](#fr-chm-05), [FR-CHM-06](#fr-chm-06), [FR-CHM-07](#fr-chm-07), [FR-CHM-08](#fr-chm-08) |
| W06 | [FR-FLW-04](#fr-flw-04), [FR-FLW-05](#fr-flw-05), [FR-FLW-06](#fr-flw-06) |
| W07 | [FR-RUN-03](#fr-run-03), [FR-RUN-04](#fr-run-04), [FR-RUN-07](#fr-run-07), [FR-LIF-04](#fr-lif-04), [FR-LIF-05](#fr-lif-05), [FR-LIF-07](#fr-lif-07), [FR-RES-05](#fr-res-05) |
| W08 | [FR-MAT-02](#fr-mat-02), [FR-DAT-05](#fr-dat-05), [FR-CFG-04](#fr-cfg-04), [FR-EXT-01](#fr-ext-01), [FR-EXT-02](#fr-ext-02), [FR-EXT-03](#fr-ext-03) |

## 6. Acceptance, numerical tolerances, and evidence gates

### 6.1 Reusable contract verification

Start with identity and basis transformations, mock capability descriptors, invalid input, hidden substitution, failure atomicity, stale-completion handling, and save/reconstruct semantics. These tests can demonstrate host behavior without selecting a thermodynamic engine. Where a whole external package cannot expose a demanded diagnostic, its declared acceptance profile must be narrowed or the host must obtain justified evidence another way; missing evidence is not a passed check.

For the synthetic fixtures below, all values and laws are deliberately invented contract oracles. They do not describe real chemical data and cannot validate a real thermodynamic model. The examples are already specific enough to implement future tests. They were not exercised against an implementation here.

#### SF-01: Complete replacement versus component-flow patch

**Setup:** A and B are synthetic mass constituents. Initial flows are A=2 kg/s, B=3 kg/s. No thermodynamic model is needed.

**Stimulus:** Patch A to 4 kg/s under preserve-unmentioned-component-flows; separately request total=5 kg/s while holding B=3 and A=4.

**Expected witness:** First operation: total=7 kg/s, mass fractions A=4/7 and B=3/7. Second operation: inconsistent constraints, no accepted change.

**Requirement trace:** [FR-MAT-06](#fr-mat-06), [FR-LIF-08](#fr-lif-08). **Status:** Specified only.

#### SF-02: Absent versus zero property

**Setup:** A mock phase reports density=1000 kg/m3; enthalpy is unavailable, not zero. An unrelated accepted result already exists.

**Stimulus:** Request density only, then a primary enthalpy calculation, then an optional viscosity report.

**Expected witness:** Density can succeed; primary enthalpy fails specifically; optional viscosity omission does not erase prior accepted primary results.

**Requirement trace:** [FR-CFG-03](#fr-cfg-03), [FR-PRP-02](#fr-prp-02), [FR-EQL-05](#fr-eql-05). **Status:** Specified only.

#### SF-03: Reference offset versus physical-model discrepancy

**Setup:** Deliberately synthetic constant-cp packages: h_A=T−300 and h_B=1.2(T−300)+100, in kJ/kg with T in K. The known pure reference offset from B is 100 kJ/kg. Pressure and composition are fixed.

**Stimulus:** At T=350 K, translate by preserving T,P; separately preserve P and reference-corrected h=50 kJ/kg.

**Expected witness:** At equal T: h_A=50, h_B=160, corrected h_B=60, residual model discrepancy=10 kJ/kg. At corrected h=50: T_B=341.6666666667 K. There is no hidden heat source and the two boundary policies are not equivalent.

**Requirement trace:** [FR-FLW-05](#fr-flw-05), [FR-PRP-07](#fr-prp-07). **Status:** Specified only.

#### SF-04: Useful mass-only caloric material

**Setup:** An empirical material has mass flow 2 kg/s and h=2(T−300) kJ/kg, with no molecular weight or elemental description.

**Stimulus:** Heat it from 300 K to 325 K with no other work/loss; then request molar enthalpy and fugacity.

**Expected witness:** Thermal fixture duty=100 kW. Molecular requests fail specifically; no fictional molecular weight is supplied. This is a contract oracle, not a claim about any real material.

**Requirement trace:** [FR-MAT-02](#fr-mat-02), [FR-MAT-03](#fr-mat-03), [FR-CFG-03](#fr-cfg-03). **Status:** Specified only.

#### SF-05: Converged-looking inverse result with failed target

**Setup:** A mock PH provider returns a success flag and a candidate whose recomputed h=20 kJ/kg; the requested target is 50 kJ/kg. The acceptance tolerance is 1e−6 kJ/kg for this synthetic case.

**Stimulus:** Submit the candidate for state acceptance.

**Expected witness:** Absolute enthalpy residual=30 kJ/kg, therefore CHECK_FAILED. The prior accepted result is unchanged.

**Requirement trace:** [FR-EQL-02](#fr-eql-02), [FR-RES-02](#fr-res-02), [FR-RES-05](#fr-res-05). **Status:** Specified only.

#### SF-06: Strict conversion versus approved alternative

**Setup:** Synthetic equal-formula A and B use A→B with initial amounts A=1 mol and B=0 mol; the example is only a stoichiometric contract test.

**Stimulus:** First request extent 0.4 mol, then strict extent 1.2 mol.

**Expected witness:** Feasible result: A=0.6 mol, B=0.4 mol. Strict 1.2 mol is infeasible. A reactant-limited alternative, if separately authorized, cannot pass the original fixed-extent request.

**Requirement trace:** [FR-CHM-02](#fr-chm-02), [FR-MAT-08](#fr-mat-08), [FR-RUN-05](#fr-run-05). **Status:** Specified only.

#### SF-07: Phase permutation and placeholder semantics

**Setup:** A synthetic result has L-a=1 mol with x=(0.8,0.2), L-b=3 mol with x=(0.1,0.9), and one zero-amount vapor placeholder. This is not an equilibrium benchmark.

**Stimulus:** Reverse the provider phase order and remap by declared identity; calculate total component amounts.

**Expected witness:** Total material=4 mol; component totals=(1.1,2.9) mol; two present liquids, zero present vapor. No extra physical phase arises from an aggregate or placeholder.

**Requirement trace:** [FR-STA-05](#fr-sta-05), [FR-STA-06](#fr-sta-06), [FR-MAT-07](#fr-mat-07). **Status:** Specified only.

#### SF-08: Obsolete and cancelled completion

**Setup:** A mock request captures model revision A and an accepted result exists. The model changes to B before A completes; a second variant cancels the request before completion.

**Stimulus:** Deliver A’s late success callback.

**Expected witness:** Keep A’s candidate historical/stale or rejected, not current under B. For cancellation, do not auto-publish the late candidate. Preserve prior accepted lineage.

**Requirement trace:** [FR-RES-05](#fr-res-05), [FR-RUN-06](#fr-run-06), [FR-LIF-01](#fr-lif-01). **Status:** Specified only.

### 6.2 Real-material fixture gate

Before a numerical acceptance run, its dossier shall identify: fixture/scenario versions; actual material representation and component data; provider/build/binding; resolved parameter and reference conventions; allowed phases/species and reaction/transfer policy; input values and bases; independent constraints and unresolved variables; unit assumptions; primary/optional outputs; initializer and branch policy; numerical and scientific domains; relevant conserved quantities and external exchanges; absolute/relative tolerances and physical scales; reference evidence and uncertainty if available; expected negative cases; and the publication level being claimed. An incomplete dossier is not an executed validation fixture.

### 6.3 Residual rules

For a checked scalar residual r, use a predeclared criterion such as **|r| ≤ a + b·S**, with a nonnegative absolute allowance a, relative allowance b, and declared physical scale S. Use a physically meaningful gross scale or floor rather than only a nearly cancelling net value. Report residual, scale and tolerance. Material/element/charge residuals and energy/specification residuals each require suitable units and scales. Numerical solver tolerances, host acceptance tolerances, model/data error and experimental uncertainty are different quantities. This is an acceptance pattern, not a universal mandated solver norm or an invented universal accuracy level. [B1 §8.3]

A selected profile must state which checks are mandatory and which are unsupported, optional, or merely diagnostic. A provider success flag is necessary only where its contract requires it and is never by itself sufficient. Stability evidence is relative to the actually assessed phase/species/restriction set. No universal proof of a global equilibrium minimum is required. At phase transitions, any derivative/branch claim needs its stated validity; failure to assess smoothness is not evidence of smoothness.

### 6.4 Separate acceptance from validation

Contract conformance asks whether the promised action occurred and failed correctly. Physical consistency asks whether the requested constraints and justified balances hold. Implementation comparison asks whether an adapter agrees with its provider under matching definitions. Independent reference validation asks whether the selected model predicts appropriate evidence within its declared uncertainty/tolerances and envelope. Operational tests ask whether edits, reload, failure, isolation and repeated runs preserve meaning. These dimensions can disagree and must be reported separately. [B1 §§1,8; B3 §4.1]

### 6.5 Integrated acceptance journeys

#### IJ-01 / B1 J01

**Journey:** Gas mixing → compression → cooling → separation → pressure reduction, with an explicit recycle.

**Scope:** SC-01, SC-04, SC-05, SC-06.

**Acceptance witness:** Verify unit material/energy balances, reference-versus-actual compressor states, PH targets, phase allocation, and recycle convergence under one captured model revision.

**Negative/boundary witness:** Exhaust the recycle after successful local flashes: retain inspectable iterates but do not accept the whole-flowsheet result.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-02 / B1 J02

**Journey:** A chemically justified polar separation/column case plus extraction/decanter and solvent recycle; test the full nonequilibrium variant separately.

**Scope:** SC-07, SC-08, SC-09, SC-10, SC-11, SC-26.

**Acceptance witness:** Verify local-state independence, nonideal phase behavior, multiple-liquid mapping, stage/whole-unit balances, and declared contacting formulation.

**Negative/boundary witness:** A fixture not physically supporting all sections cannot demonstrate them by juxtaposition; an efficiency model cannot certify SC-26.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-03 / B1 J03

**Journey:** Water/steam or refrigerant utility coupled through heat to a different process material; use separate justified fluid variants where required.

**Scope:** SC-02, SC-03, SC-12, SC-13, SC-14, SC-28.

**Acceptance witness:** Verify within-side caloric consistency, quality semantics, inverse state operations, no species exchange across the wall, and utility-loop closure.

**Negative/boundary witness:** Change one side’s legitimate constant energy reference and require invariant physical heat/work; isolate optional bounding-property failures.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-04 / B1 J04

**Journey:** Reaction, cooling, phase separation, and recycle using separately declared conversion, equilibrium, kinetic, and reactive-separation variants.

**Scope:** SC-17, SC-18, SC-19, SC-20.

**Acceptance witness:** Verify that each variant retains its own species authority, reaction-energy convention, required coupled conditions, and outer convergence.

**Negative/boundary witness:** An impossible fixed conversion must not be converted to a limited or equilibrium solution without a new authorized problem.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-05 / B1 J05

**Journey:** Aqueous preparation → reactive gas contact → precipitation → solids separation.

**Scope:** SC-21, SC-22, SC-23, SC-24, SC-26, SC-30.

**Acceptance witness:** Track true/apparent species, external exchanges, energy, inert versus precipitating solids, and qualified contacting formulation across all boundaries.

**Negative/boundary witness:** Remove an essential caloric or transfer property: speciation may remain useful but the complete energy/rate-based process claim must fail.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-06 / B1 J06

**Journey:** Assay characterization → compositional processing, with a separately declared reduced-model/package-boundary test.

**Scope:** SC-15, SC-16, SC-29, SC-30.

**Acceptance witness:** Reconstruct original assay/cuts and complete required caloric data; verify translation constraints and report irreversible aggregation/model discrepancy.

**Negative/boundary witness:** PVT-only pseudocomponents or an invented inverse lumping map cannot satisfy the thermal/detailed-composition fixtures.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-07 / B1 J07

**Journey:** Empirical solid-bearing material → heating → mechanical separation, with a separately qualified reactive-solid variant.

**Scope:** SC-23, SC-25, SC-31.

**Acceptance witness:** Execute justified mass/energy operations without fictitious molecular properties; require additional explicit chemistry before the reactive extension.

**Negative/boundary witness:** Request elemental balance/fugacity from a mass-only material: reject only the unsupported claim and retain supported thermal use.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

#### IJ-08 / B1 J08

**Journey:** Conceptual surface, distribution, inventory, and restricted-state walkthroughs, each followed by a transfer/change/result operation.

**Scope:** SC-27, SC-32, SC-33, SC-34.

**Acceptance witness:** Show meaningful representation, quantity/basis and conservation semantics, permitted actions, and qualified unavailable numerical operations.

**Negative/boundary witness:** Untyped metadata or a fictitious flow/unique distribution inverse cannot satisfy representability; no numerical P3 pass is implied.

**Status:** Not executed. The families may require multiple separately justified fixtures, not one physically implausible all-feature plant.

## 7. Inherited probe disposition

The 18 B2 probes and 20 B3 probes remain unexecuted. They are now connected to requirements rather than left as a detached research list. These mappings do not prescribe adopting the probed provider. A probe can qualify an adapter, reveal a required host wrapper, or disqualify a claimed provider profile without changing the host requirement.

| B2 DWSIM probe | Requirements whose evidence it informs |
| --- | --- |
| P01 | [FR-DAT-06](#fr-dat-06), [FR-RUN-03](#fr-run-03), [FR-RUN-04](#fr-run-04) |
| P02 | [FR-DAT-06](#fr-dat-06), [FR-LIF-04](#fr-lif-04) |
| P03 | [FR-RES-05](#fr-res-05), [FR-RUN-07](#fr-run-07) |
| P04 | [FR-RUN-05](#fr-run-05), [FR-CFG-05](#fr-cfg-05) |
| P05 | [FR-STA-03](#fr-sta-03), [FR-PRP-01](#fr-prp-01), [FR-RES-04](#fr-res-04) |
| P06 | [FR-EQL-02](#fr-eql-02), [FR-RES-04](#fr-res-04) |
| P07 | [FR-STA-05](#fr-sta-05), [FR-FLW-03](#fr-flw-03) |
| P08 | [FR-MAT-05](#fr-mat-05), [FR-MAT-06](#fr-mat-06), [FR-LIF-08](#fr-lif-08) |
| P09 | [FR-LIF-02](#fr-lif-02), [FR-CFG-07](#fr-cfg-07) |
| P10 | [FR-LIF-01](#fr-lif-01), [FR-LIF-08](#fr-lif-08) |
| P11 | [FR-LIF-04](#fr-lif-04), [FR-LIF-05](#fr-lif-05), [FR-LIF-07](#fr-lif-07), [FR-DAT-05](#fr-dat-05) |
| P12 | [FR-GOV-03](#fr-gov-03), [FR-LIF-05](#fr-lif-05) |
| P13 | [FR-RUN-03](#fr-run-03), [FR-RUN-04](#fr-run-04) |
| P14 | [FR-RUN-08](#fr-run-08), [FR-RES-06](#fr-res-06) |
| P15 | [FR-CHM-05](#fr-chm-05), [FR-PRP-07](#fr-prp-07) |
| P16 | [FR-GOV-02](#fr-gov-02), [FR-GOV-04](#fr-gov-04) |
| P17 | [FR-PRP-02](#fr-prp-02), [FR-EQL-05](#fr-eql-05) |
| P18 | [FR-MAT-07](#fr-mat-07), [FR-LIF-06](#fr-lif-06) |

| B3 comparative probe | Requirements whose evidence it informs |
| --- | --- |
| PV-01 | [FR-LIF-07](#fr-lif-07), [FR-RES-07](#fr-res-07) |
| PV-02 | [FR-DAT-01](#fr-dat-01), [FR-DAT-06](#fr-dat-06), [FR-LIF-04](#fr-lif-04) |
| PV-03 | [FR-STA-05](#fr-sta-05), [FR-EQL-08](#fr-eql-08) |
| PV-04 | [FR-GOV-02](#fr-gov-02), [FR-EQL-08](#fr-eql-08) |
| PV-05 | [FR-CFG-02](#fr-cfg-02), [FR-CFG-03](#fr-cfg-03), [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03) |
| PV-06 | [FR-RUN-01](#fr-run-01), [FR-STA-02](#fr-sta-02) |
| PV-07 | [FR-CFG-04](#fr-cfg-04), [FR-DAT-01](#fr-dat-01) |
| PV-08 | [FR-DAT-05](#fr-dat-05), [FR-CFG-04](#fr-cfg-04), [FR-MAT-02](#fr-mat-02) |
| PV-09 | [FR-RUN-04](#fr-run-04), [FR-RUN-03](#fr-run-03) |
| PV-10 | [FR-GOV-02](#fr-gov-02), [FR-EQL-08](#fr-eql-08) |
| PV-11 | [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03), [FR-RES-02](#fr-res-02) |
| PV-12 | [FR-RUN-03](#fr-run-03), [FR-LIF-01](#fr-lif-01) |
| PV-13 | [FR-RUN-01](#fr-run-01), [FR-EQL-02](#fr-eql-02), [FR-EQL-03](#fr-eql-03) |
| PV-14 | [FR-PRP-05](#fr-prp-05), [FR-PRP-06](#fr-prp-06) |
| PV-15 | [FR-FLW-07](#fr-flw-07), [FR-RUN-02](#fr-run-02) |
| PV-16 | [FR-LIF-04](#fr-lif-04), [FR-LIF-05](#fr-lif-05) |
| PV-17 | [FR-EXT-03](#fr-ext-03), [FR-CHM-03](#fr-chm-03), [FR-STA-03](#fr-sta-03) |
| PV-18 | [FR-CHM-04](#fr-chm-04), [FR-CHM-07](#fr-chm-07) |
| PV-19 | [FR-FLW-05](#fr-flw-05), [FR-FLW-06](#fr-flw-06) |
| PV-20 | [FR-PRP-02](#fr-prp-02), [FR-RES-05](#fr-res-05), [FR-RUN-07](#fr-run-07) |

## 8. Decision rules and explicit reconciliations

| Decision | Topic | Rule for this baseline | Basis |
| --- | --- | --- | --- |
| FD-01 | Normative scope | B1 remains the scope authority. All 34 scenarios, 24 cross-cutting behaviors, and P1/P2/P3 assignments are retained. New wording refines behavior; no backend is selected. | B1 §§1–4; B3 §12 |
| FD-02 | Applicability | H requirements govern the host’s behavior wherever applicable; C requirements govern claimed numerical/formulation capabilities. P1/P2 still require selected end-to-end execution and validation eventually. Unsupported per-provider results do not satisfy P1/P2 program-wide delivery. | B1 §§1–3,8; B3 CR-04 |
| FD-03 | Strict conversion | B1 SC-17 wins over DWSIM’s observed limiting behavior. Reject an infeasible strict conversion. A separately authorized limited-conversion alternative is recorded as a different problem and cannot pass the strict fixture. | B1 SC-17; B2 WF-09 |
| FD-04 | Alternate-model recovery | Same-model numerical retries are permitted under policy. Any changed phase/species scope, ideal substitution, or omitted equilibrium must remain an alternate problem with explicit authorization and distinct result qualification. | B1 X07; B2 WF-05; B3 CR-13 |
| FD-05 | Readiness versus broad capability | The qualification chain is request-specific. PVT support, caloric completion, transport support, derivatives, algorithm coverage, and initialization are not synonyms. | B3 §7.5; CR-04–CR-06 |
| FD-06 | Defaults and input intent | A default can initialize a workspace or be accepted as a named model choice; it cannot silently become a user specification. Complete replacement, patching, and rescaling have different effects. | B2 WF-02,WF-04,WF-10 |
| FD-07 | Calibration and science | Known extrapolation or estimated data can remain usable under authorized engineering policy. Such use is not automatically within validated scope. Numerical acceptance is not independent physical validation. | B1 §§2,8; B3 W01,W07 |
| FD-08 | Authority and publication | A property call, equilibrium result, unit outlet specification, accepted local state, and accepted flowsheet state carry different authority and completion claims. | B2 §8.1; B3 CR-02,CR-03 |
| FD-09 | No imposed implementation architecture | Logical owners identify accountability, not services/classes. Coherent external packages, scoped mutable sessions, and equation contributions all remain admissible. | B2 §8.2; B3 §§7,9 |
| FD-10 | P3 representability | Only concrete state/action/conservation walkthroughs satisfy P3 representability. The four R3 records do not make numerical surface/polymer/UV/metastable implementations mandatory in P1/P2. | B1 SC-27,SC-32–SC-34; §8.1 |
| FD-11 | Acceptance thresholds | Real-fluid numerical fixtures require independently specified absolute/relative tolerances, scales, data and model conventions before execution. No universal accuracy percentage is invented. | B1 §8.3 |
| FD-12 | Requirements evidence | B2/B3 source observations motivate requirements but do not make them capabilities of this simulator. This stage performs catalog consistency checks only, not thermodynamic execution or validation. | B2 §0.2; B3 §1 |

### Outcome categories, not an implementation enum

The following vocabulary fixes distinctions that callers must be able to observe. It is not a required exception hierarchy or storage schema. One result can have several independent qualifications; an alternate-model candidate, for example, is not an exception category by itself and does not erase the original request’s failure.

| Semantic category | Meaning |
| --- | --- |
| UNKNOWN_IDENTITY / AMBIGUOUS_IDENTITY | A constituent or provider identifier cannot be resolved uniquely. |
| INVALID_INPUT / INCONSISTENT_SPECIFICATION | Values, bases, or jointly imposed constraints are invalid or conflicting. |
| UNDER_SPECIFIED / OVER_SPECIFIED / COMPLETENESS_UNASSESSED | A complete-state request lacks independent information, adds unresolved extra constraints, or has not been assessed. Intentionally partial model states are not errors by themselves. |
| AUTHORITY_CONFLICT | The requested changes conflict with caller-owned intensives, phase allocations, or species quantities. |
| UNSUPPORTED_OPERATION / CAPABILITY_UNASSESSED | The exact adapter/profile rejects the operation, or its support has not been established. |
| DEPENDENCY_UNAVAILABLE / MISSING_DATA | Required engine, binding, parameter data, or separately required resource is unavailable. |
| METHOD_INCOMPATIBLE / REFERENCE_INCOMPATIBLE | The selected method or energy/standard-state combination cannot be coherently applied. |
| OUTSIDE_ALLOWED_DOMAIN | The request violates its selected data/model/extrapolation policy; record numerical bounds separately. |
| INITIALIZATION_REQUIRED / INITIALIZATION_FAILED | The provider needs an admissible start or preparation failed. This is not a proof of physical infeasibility. |
| INFEASIBLE / FEASIBILITY_UNASSESSED | The constraints are demonstrated incompatible, or feasibility has not been established. Do not infer infeasibility from generic failure. |
| BRANCH_AMBIGUOUS / PHASE_CORRESPONDENCE_AMBIGUOUS | A unique branch or cross-run phase match is not justified. |
| NONCONVERGENCE / BUDGET_EXHAUSTED | Required numerical convergence was not achieved under the stated policy. |
| CHECK_FAILED / REQUIRED_CHECK_UNASSESSED | A mandatory physical postcondition fails or lacks required evidence despite any solver success flag. |
| PROVIDER_FAILURE | An external failure with known or unknown native cause; identify affected stage/session. |
| CANCELLED / OBSOLETE_CONTEXT | The caller cancelled or the captured input/configuration revision is no longer current. |
| OPTIONAL_PROPERTY_UNAVAILABLE | An optional property failed; primary operation status must be assessed independently. |

### Change-impact treatment

| Change | Required semantic treatment |
| --- | --- |
| Feed state, amount or composition | New specification revision; affected local/unit/recycle results become stale. Preserve replace/patch intent. |
| Add/remove or remap constituent | Impact preview; explicit quantity policy; repair or invalidate phase, reaction, coordinate and parameter dependencies. |
| Parameter/correlation/characterization change | New resolved-data/material revision; reassess caloric/phase capabilities and all affected cached results. |
| Reference-state change | New convention revision; reconcile affected energy relationships and rebuild/rebind sessions where required. |
| Allowed phases, reactions or physical approximation | New physical-problem identity. Prior results are not solutions of the unrestricted/new problem. |
| Algorithm, tolerance or initialization policy | New numerical-policy evidence; reassess branch and acceptance implications without pretending physics automatically changed. |
| Provider/build/binding upgrade | New implementation manifest; require qualification and reproduce/compare explicitly. |
| Display units, label or icon position | No physical change when the actual input and connectivity are unchanged; presentation-only revision. |
| Snapshot/load/migration | Reconstruct semantics and identities first; retain historical lineage and reassess currency/readiness. |
| Result completes after an intervening edit | Publish only to the captured revision as qualified history, or reject; never relabel as current. |

Correct conservative invalidation is permitted. The later blueprint must decide whether and how to make it more selective; this document does not mandate graph algorithms, caches, storage engines, or an incremental-computation library.

## 9. Open items and implementation handoff

| Item | Topic | Resolution required | Gate |
| --- | --- | --- | --- |
| OP-01 | Provider and data selection | Select actual supported profiles with exact builds, bindings, parameter sources and public availability evidence. | Before a numerical fixture can claim Executable; host rejection tests can use mocks now. |
| OP-02 | Numerical fixture definition | Pin real material compositions, operating points, boundary cases, independent references and error/uncertainty budgets. | Before thermodynamic fixture execution or a Validated claim. |
| OP-03 | Phase correspondence | Define a matching policy and explicit ambiguity treatment at coalescence, disappearance and equal-density crossings. | Before claiming cross-run phase continuity; order-invariance tests remain mandatory. |
| OP-04 | Cross-package convention reconciliation | Determine exact reference transformations and meaningful preserved quantities for each material boundary; do not assume a universal offset. | Before accepting each package-boundary profile. |
| OP-05 | Empirical and petroleum completion | Identify a complete caloric/characterization route for SC-15/16/31 rather than relying on the most convenient EOS API. | Before P2 broad-coverage claims. |
| OP-06 | Chemistry and transport completion | Qualify actual speciation, heat, phase, transport and rate data for aqueous/reactive/solid scenarios. | Before accepting SC-20–26 numerical profiles. |
| OP-07 | Session and failure containment | Select and demonstrate isolation, cancellation and recovery policies for actual native providers. | Before concurrent/reusable production session claims. |
| OP-08 | Equation/formulation contract | Resolve full information ownership, constraint boundaries, scaling and initializer restoration in the conceptual blueprint. | Steps 5–8 before code-interface design; no solver/library selection implied here. |
| OP-09 | Change dependencies and persistence | Choose a correct conservative or precise invalidation strategy and semantic migration rules in the later blueprint. | Before accepting change/reconstruction capabilities; precision/efficiency is not assumed. |
| OP-10 | Operational performance | Establish workload-based latency/throughput/cancellation targets after representative calls and problem sizes are known. | Before performance commitments; correctness does not depend on invented performance numbers. |

### Conditions for the next architecture step

Step 5 should assign each requirement to conceptual responsibility boundaries and identify the information needed to satisfy it. Each proposed domain concept must have meaning, authority, lifecycle and motivating requirements; each proposed action must consume valid inputs and produce the promised outcome or a specified failure. Internal data structures and Rust interfaces should be derived only after those relationships and workflows are resolved. No implementation task is authorized merely by selecting a preferred library name.

The next blueprint should demonstrate at minimum the valve PH publication path; two-liquid identity versus product routing; calorically complete versus partial pseudocomponent use; strict reaction conversion and exactly-once heat accounting; apparent/true species with explicit exchanges; heat-only versus material package boundaries; nonequilibrium bulk/interface evaluation; mass-only empirical heating; and configuration-change/failure/reload lifecycle. These are consequences of this requirement set, not additional scope families.

### Catalog completion audit

| Check | Result |
| --- | --- |
| Unique functional requirements | 94 |
| Requirement classes | H: 65, C: 25, R3: 4 |
| Positive plus negative/boundary requirement witnesses | 188 |
| Scenario contracts / scenario witnesses | 34 / 68 |
| Integrated journeys | 8 |
| Synthetic contract-fixture specifications | 8 |
| B1 scenario / X / concept mappings | 34 / 24 / 13 |
| B2 workflow / probe mappings | 15 / 18 |
| B3 CR / workflow / probe mappings | 16 / 8 / 20 |
| Missing mandatory card fields | 0 |
| Duplicate IDs or broken requirement crosslinks | 0 |
| Numerical or implementation-conformance tests executed | 0 |
| Scenario R/E/V promotions | 0 |

A passing structural audit establishes only that the catalog is connected and well formed. It does not establish that the requirements are semantically sufficient for every future material, that a provider implements them, or that a numerical result is correct. The QA companion records the source hashes and counts used for this audit.

## 10. Source baseline register and citation convention

This specification is a transformation of the supplied project baselines. It does not independently re-verify their external library assertions. Requirement statements and synthetic fixtures are proposed design content; references identify motivation and inherited obligations, not scientific proof of the proposed implementation. The existing external source pins and their limitations remain in B2/B3. No source-library code is reproduced or selected for reuse.

### B1: THERMO-SCOPE-001 v0.1

**File:** `thermodynamics_simulation_behavior_scope_v0_1.md`  
**SHA-256:** `1d12284bbca6569f198caaae86327788bac8fbaf3946625d99d971ef3602d028`

**Used:** §§1–3 scope/evidence/boundary rules; §5 all 34 scenario cards; §6 X01–X24; §7 J01–J08; §8 acceptance rules; §9 C01–C13; §§10–12 boundaries and handoff.

### B2: THERMO-DWSIM-002 v0.1

**File:** `dwsim_workflow_reverse_engineering_v0_1.md`  
**SHA-256:** `55e77b6fddab4547280e4e40ca5c742fac3fd322e45534767ddae7f5f6ba94ab`

**Used:** §2 WF-01–WF-15; §§4–6 recovery/lifecycle/design distinctions; §8.1 calculation authority; §9 actions; §10 P01–P18. Observations are pinned-source findings, not executed tests.

### B3: THERMO-LIBRARIES-003 v0.1

**File:** `thermodynamic_package_comparative_research_v0_1.md`  
**SHA-256:** `c0f9216c450405360b8532771e6c4beca6536154345ea5adf00130a9c4e73875`

**Used:** §3 L01–L08 provider dossiers; §5 W01–W08; §§7–9 synthesis and CR-01–CR-16; §11 PV-01–PV-20; §12 unresolved coverage. Library observations retain the documented source and runtime limitations.

Cross-references such as `B1:SC-17`, `B2:WF-09`, and `B3:CR-06` refer to the named sections/identifiers of these exact baseline versions. `AT-*` identifies a witness specification in this document. `SA-*` refines a scope scenario without adding one. `SF-*` is synthetic and cannot support a real-fluid accuracy claim. Full predecessors are included under `baselines/` in the downloadable bundle.

**Final requirement thesis:** preserve material meaning and coherent model/data/reference choices; declare authority and prerequisites for each operation; protect accepted results from unqualified work; and retain enough information to explain, change, reconstruct and validate a flowsheet calculation across its intended breadth.
