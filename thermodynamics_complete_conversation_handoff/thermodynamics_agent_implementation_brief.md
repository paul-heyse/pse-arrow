# Thermodynamics and flowsheet modeling: implementation handoff

**Audience:** the programming agent designing and implementing the Process simulator.  
**Purpose:** preserve the research-intensive decisions, their rationale, the useful external implementations, and the work the host must still own.  
**Status:** architectural synthesis of the nine-stage conversation, not a new thermodynamic validation or a production backend approval. Sources and entry points are in [the resource index](thermodynamics_resource_index.md).

## 1. The conclusion to implement

Build a **flowsheet-oriented, provider-independent modeling core around coherent thermodynamic configurations**, rather than building the simulator around a particular thermodynamic library's stream/state objects. Start as a modular application with explicit internal responsibilities, not a collection of separately deployed services. The intended Rust implementation should own process meaning and orchestration; numerical thermodynamics can initially remain in mature non-Rust engines.

The product goal is breadth of conventional process engineering: ordinary fluids, nonideal mixtures, multiple liquids, water/steam and refrigeration, petroleum representations, reactions, aqueous chemistry, solids, and useful limited-property materials. Exceptional accuracy for a narrow advanced EOS family is not the organizing objective. The conventional fluid foundation is necessary but insufficient: the broader cases must influence the contracts before they are numerically implemented. P3 surface/distribution/inventory/restricted-state extensions retain their explicitly deferred execution scope. [B1; B4 §1]

The core separation is:

> **Material meaning + resolved methods/data → location-specific physical problem → numerical realization → candidate → scoped qualification → current-result publication.**

Do not translate every concept sheet into a class, every action into an RPC, or every responsibility package into a crate. Those catalogs specify semantics and accountability. They are not a demand for the maximum number of software abstractions. [B5 §§0–3; B6 §1; B7 §1]

The early suggestion that ThermoPack should necessarily be the default backend was superseded. The final position is profile-specific qualification: **DWSIM, `thermo`, and Clapeyron are serious coherent-package candidates; CoolProp is a specialized-fluid candidate; Reaktoro is a coherent chemistry candidate; ThermoPack and FeOS are useful focused numerical providers.** IDAES and CAPE-OPEN supply important process/formulation and interoperability precedents. None received full production approval in this conversation. [B3 §§0,9; B8 §8; B9 §0]

## 2. What a thermodynamic package means

Retain an engineer-facing property package: users should be able to select and maintain one coherent configuration. Internally, distinguish the physical roles it contains. PR and NRTL are not interchangeable enum alternatives: an EOS and a liquid activity model answer different questions and may cooperate in one package.

A nonideal-mixture package can require vapor fugacity, liquid activity, liquid standard states, saturation or Henry relationships, caloric contributions, density/volume methods, transport models, interaction data, and permitted phases. Which pieces are independent depends on the formulation. A potential-based model may derive several properties together; a correlation-based package may assemble them from several relationships. Both are legitimate.

The organizing rule is **compatible physics, not uniform function signatures**. Do not join an activity vector, arbitrary vapor model, and unrelated enthalpy function and call the result a complete package. Document the reference/standard-state agreement, caloric approximation, operation eligibility, and data requirements of the combination. Conversely, do not dismantle a library's internally coherent package merely to force identical low-level interfaces across providers. [B3 §7; B5 §7.1; B8 IR-02/05/06]

Four things need separate identity: the reusable recipe; its resolved method/data/reference revision; its binding and allowed local policy at a process location; and its actual provider/build/numerical realization. Sessions are runtime resources, not any of these definitions. A package can serve hundreds of locations without sharing their mutable physical state.

Resolve data during explicit preparation. An estimate, regression, combining-rule value, intentional absence, and specified zero have different meanings. A parameter is qualified by its equation, ordered subjects, units, temperature dependence, and standard-state convention, not just a label such as `A12`. Preserve original assay/measurement evidence and the generated parameter/material proposal. Approval of a fit is not independent validation. [B2 WF-03; B6 IC-10–24; B7 AC-04–09]

**Completeness is operation-specific.** A material can support PVT calculations but lack total enthalpy; a package can solve TP but lack an admissible PH initializer; a unit can support duty calculation but not rating because transport data are absent. Preserve those useful subsets without pretending they close the program's broader coverage gaps.

## 3. Flowsheet structure and thermodynamics have complementary authority

The host process model owns equipment laws, connectivity, operating modes, physical boundaries, interstage balances, geometry, transfer/carryover assumptions, and the global solution formulation. Thermodynamics owns compatible material relationships. A property package should not need a graphical flowsheet or a particular equipment class to evaluate a phase.

Represent material at **locations**, not only on external stream connectors: a stage vapor, stage liquid, exchanger segment, reactor contents, interface state, or vessel inventory can all need thermodynamics. IDAES's shared-parameter/local-state pattern is a useful precedent, while DWSIM supplies the full workflows that expose how these states are used. [B2 WF-04/07–11; B3 L02; B5 P04/P07]

A unit should declare the properties and constraints its selected formulation actually needs. A heater's energy balance and a rating calculation have different demands. A compressor has a PS reference state and a distinct actual outlet determined by its efficiency/work relationship. A separator resolves or accepts an internal state and then applies product-routing assumptions. An equilibrium-stage column and a full nonequilibrium contactor are different formulations, not interchangeable features with similar names.

Keep the **physical process graph** distinct from its **numerical solution representation**. The host may derive equation incidence, local blocks, tear variables, and iteration schedules from the same physical model. A numerical tear need not become a physical transformation; the GUI's connector representation need not be the authoritative process topology. This is a proposed implementation organization of the established process/solver separation, not a commitment to a specific graph algorithm.

Permit local candidates inside an ongoing coupled calculation. Waiting for a converged plant before using any local result would prevent iteration. But those candidates remain tied to the actual input/iteration view and achieved completion scope. Two results with the same package revision can still be inconsistent because they used different upstream iterates. Final unit/flowsheet acceptance requires the coupled residuals and input lineage, not merely successful children. [B7 AR-09/10; B8 IR-21]

## 4. The data distinctions that prevent architectural dead ends

### Material identity, representation, and account

A chemical species, assay cut, empirical lump, or distribution-valued constituent has identity and justified information. A representation defines the coordinates used to describe material. A local material account identifies what physical material is being counted at a location/snapshot. These are different concepts.

Apparent electrolyte feed components and true ionic species can be two descriptions of one account; do not add them as two inventories. Two actual disjoint liquid portions can be summed. An overall-liquid reporting aggregate cannot then be added again. For reacting material, conserve the justified element/mass/charge/site quantities under the selected model, not necessarily each species or the total number of species moles. Maps may be aggregating, state-dependent, or noninvertible. [B6 IC-01–09,25–33,47; SI-06/07/15]

Do not manufacture molecular weight, elemental composition, critical properties, or fugacity for a mass-only empirical material. Its supported density or thermal correlation can remain useful. An unavailable molecular conversion blocks the operation that needs it, not every mass/energy operation. Petroleum characterization must preserve both the original assay and generated cut definitions; accepting custom EOS components is not the whole characterization workflow.

### State, amount, role, and availability

An intensive state, material flow, inventory, and reference amount used by a numerical calculation are not equivalent. A one-mole solver basis is not one mole of physical holdup. An empty inventory does not gain a unique fluid state by normalizing zeros. A zero-flow feed can retain independently specified composition.

Every value must retain observable meaning, subject, amount basis, denominator, units, required reference convention, and role. A target, observation, guess, externally supplied allocation, and calculated value can be numerically identical while carrying different authority. Known zero, unresolved, physically undefined, unsupported, deferred, and failed must not collapse into one sentinel. Reuse common semantic descriptors instead of repeating large metadata blocks in every hot numeric value. [B6 §§1.3–1.4; IC-25–28,38]

### Phase category, instance, and correspondence

A phase category is not a phase instance. Allowed candidates, structural solver slots, present physical portions, incipient information, numerical placeholders, and aggregates all differ. A liquid's array position or a light/heavy outlet label is not its physical identity.

Use snapshot-local phase identities and explicit cross-snapshot correspondence. Appearance, disappearance, split, merge, and ambiguity are legitimate. Do not require false continuous one-to-one labels through coalescence. Product routing consumes phase evidence plus unit rules; it does not define equilibrium identity. [B6 IC-29–31; B7 AC-16/40]

### Definitions, workspaces, and results

Approved definitions and captured inputs must be stable in meaning. Provider sessions, arrays, and solver iterates may be mutable internally. Detach or otherwise safely retain candidate payloads before they become accepted evidence. Separate the candidate, check results, accepted scope, and current association. This is an authority boundary, not an insistence on copying every large array or persisting every iterate. [B5 §§5,7.7; B6 IC-53–70]

## 5. Calculation authority should drive the interfaces

A numerical engine's capability is broader than its permission in one request. A supplied-phase property evaluation may determine density or select an allowed root without changing species totals or phase allocation. A nonreactive flash may redistribute phases but not react the material. A chemical-equilibrium problem may change species while preserving declared conserved totals and accounting for authorized exchanges. A provider that can do more must still honor the narrower request.

Capture the original constraints, unknowns, phase/chemistry restrictions, branch intent, required outputs, and check obligations. Preserve that physical problem through initialization and same-problem retries. A changed phase policy, ideal-model substitution, or frozen chemistry is a different problem unless it is an explicitly temporary initialization intervention subsequently restored. A successful response describing underdetermination is not a successfully resolved physical state. [B6 IC-34–42; B7 AR-03/04/11]

Do not require every call to be a complete flash. Phase-only evaluation is essential for externally owned allocations, nonequilibrium contacting, and host-controlled stage variables. Completion of equilibrium also does not promise every transport or reporting property. Optional work can fail without destroying trustworthy independent primary results, but a failed combined native call cannot justify invented primary success.

## 6. Reuse boundary and solver arrangement are independent choices

Support several arrangements rather than choosing one universal abstraction: direct solved-state calls; supplied-phase property callbacks; condensed solved blocks; exposed local residual blocks; and explicit equation contributions. A whole package can serve a valve directly and a coupled optimizer through a separately qualified solved-response interface. A value-only backend does not automatically expose residuals or symbolic equations. [B8 §2]

For a condensed block `g(u,y)=0` and outer observable `F(u)=r(u,y(u))`, on a regular branch:

`dy/du = -g_y^(-1) g_u`

`dF/du = r_u - r_y g_y^(-1) g_u`.

A frozen-`y` derivative is not the derivative of the solved block. Internal variables can include phase fractions, compositions, density roots, and chemical species. Derivative identity also includes the independent composition chart, constraints held fixed, amount basis, branch, and order. Automatic differentiation around an opaque native call does not supply its missing mathematical derivative. Hessian and sparsity contracts need equally explicit meaning when the chosen outer solver demands them.

The Step-9 authored two-phase fixture made this distinction tangible: frozen-allocation heat capacity was 100 J/mol/K, while the re-equilibrated enthalpy response was about 2,659.24 J/mol/K. That is evidence about the synthetic formulation, not certification of a provider derivative. [B9 §4.1]

Complete calorics must enter **inside** a PH/PS/UV solve and its verification. Solving a residual-only enthalpy target and adding the ideal contribution afterward answers the wrong inverse problem. A sign-changing bracket through a saturation discontinuity does not prove a root. Likewise, all active `K=1` makes the fixed-K Rachford–Rice allocation undetermined; a full problem with a separate energy/quality constraint may still be resolvable. Distinguish incomplete specification, conditioning, branch events, and solver failure. [B8 IR-05/12/14; B9 §§3–4]

Admit useful engineering approximations explicitly. Do not demand a universal fundamental potential or every exact identity from a deliberately limited correlation package. Do require derivatives of the actual selected function and appropriately bounded applicability, consistency, and validation claims.

## 7. Chemistry and energy cannot be reconciled by loose post-processing

Chemistry is a separate **definition responsibility**, not a mandatory separate solve before or after a flash. A coherent reactive multiphase thermal problem can be solved in one chemical-system engine or a coupled host formulation. Sequential splitting is allowed only with a declared approximation/convergence strategy and final checks of the combined conditions. Two successful substeps do not establish equilibrium if the second invalidates the first. [B5 §7.2; B7 AC-24; B8 IR-08]

Distinguish strict conversion, equilibrium, kinetics, and frozen participation. Reject infeasible strict consumption; a reactant-limited alternative is a new authorized problem. A constraint that requires gas, titrant, or another reservoir must expose the exchange in material and applicable energy balances. Retries evaluate a candidate from captured inputs; they do not apply physical exchange repeatedly.

Separate heat-only coupling, representation mapping, and actual material transition. Two exchanger circuits can have separate internally consistent enthalpy references without translating compounds across the wall. A material moving between packages needs an explicit preserved-quantity policy and any justified reference transformation. After reference alignment, differing predicted enthalpies are model disagreement, not a license to invent heat.

A justified transformation can be composition-dependent, e.g. `H_B = H_A + c^T n`. Reactions change its contribution by `c^T nu xi`; formation/reaction bookkeeping must reconcile it exactly once. This energy relation does not authorize arbitrary changes of chemical-potential standards or equilibrium constants. Preserve source caloric/standard-state conventions and distinguish correction, discrepancy, and real exchange. [B6 IC-21/43/51/71; B8 §7]

## 8. Library strategy: reuse the accumulated engineering, not only formulas

The following is the practical recommendation, not an already tested production ranking. Public entry points and focused reading locations are in [the resource index](thermodynamics_resource_index.md).

| Resource | Role in this implementation | Why and limits |
|---|---|---|
| **DWSIM** | Primary whole-flowsheet reference; coherent-package candidate; comparative process fixtures. | Its value includes package preparation, internal unit states, routing, reactions, recycles, and restoration. Keep its mutable stream/context internals inside an adapter. GPLv3 and separate edition/dependency availability require an explicit distribution review; a process boundary is not an assumed licensing exemption. |
| **`thermo` + `chemicals`** | First practical conventional-package integration spike, compared with DWSIM; separate correlation/data support. | `thermo` already separates constants/correlations, phase models, flash families, and results. Use its modern phase/flash interface, not a convenience stream as canonical state. `chemicals` supplies lower-level data/correlations/numerical utilities, not a complete interchangeable package. Pure, VL, VLN and solids coverage are distinct. |
| **Clapeyron.jl** | Serious alternative whole-model/solver candidate and principal composition reference. | Composite methods and algorithm eligibility are more useful than a catalog count. Verify model × algorithm × specification × initializer compatibility; do not confuse a Julia runtime boundary with an architectural incompatibility. |
| **CoolProp** | First specialized utility/pure-fluid backend candidate. | Reuse an explicitly selected backend and coherent fluid/reference configuration. Do not infer all-mixture/all-input-pair support from its shared interface or old documentation. Its REFPROP interface does not supply REFPROP itself. |
| **Reaktoro** | First chemical-system candidate for aqueous speciation, reactive equilibrium, and mineral phases. | Keep its database, species/phases, activity models, constraints, and caloric requirements together. Transport, unit geometry, full absorber behavior, and safe host integration are separate work. |
| **ThermoPack** | Focused EOS/property/flash provider, attractive for native integration where its exact surface fits. | Established Fortran numerical core and C/C++/Python access are useful. Qualify actual exposed phase counts/input pairs, complete pseudocomponent calorics, and activation/session behavior. Do not preselect it for every conventional or reactive material. |
| **FeOS** | Selected potential-based/SAFT provider and derivative architecture reference. | Native Rust is useful but not the selection criterion. Its scientific framework is substantial; its scope is not the complete process-material and lifecycle envelope. Qualify ideal/caloric completion and inverse-operation prerequisites. |
| **IDAES** | Main equation-oriented process/property/reaction reference; optional Python validation formulation. | Borrow shared definitions/local states, explicit unit-property coupling, initialization, and translator constraints. Do not reproduce Pyomo object structure mechanically or mistake an implemented model for a ready parameterized industrial package. |
| **CAPE-OPEN** | Interoperability and calculation-authority reference, optional adapter target. | Defines interaction semantics rather than physical data or algorithms. Keep a richer internal model where the process scope requires it; standards compatibility does not automatically provide a conservative translator. |

**Suggested first execution environment:** qualify an explicit `thermo` conventional package and a CoolProp utility package, with DWSIM used for comparative configured cases; qualify Reaktoro separately for chemistry. Keep Clapeyron a real alternative where its composition or phase algorithms better fit. Introduce ThermoPack or FeOS where their demonstrated numerics add value. This order minimizes new runtime integrations while confronting both general fluids and specialized cases; it is not evidence that the first candidate is numerically superior.

**Additional current research pointer, not covered by the nine-stage validation:** WaterTAP's `reaktoro-pse` already supplies a Reaktoro gray-box integration for Pyomo/IDAES. Inspect its treatment of constrained chemistry, output derivatives, and variable mapping before designing a comparable condensed boundary. It explicitly does not replace Reaktoro or select appropriate chemistry automatically. Neither its existence nor this handoff validates the Rust equivalent. [R-10]

## 9. What to build and what not to rebuild

Build the host's material/representation model, explicit location bindings, unit/property demand boundary, original-problem authority, adapter normalization, qualification, and revision-safe publication. These are the semantics no external EOS package can supply for our whole heterogeneous simulator. Also own explicitly chosen host unit formulations and any missing limited-property material/characterization workflow.

Reuse complete property configurations, flash implementations, standard pure-fluid models, chemical-system engines, available correlations and data, and existing numerical dependencies where they meet those contracts. Do not start by rewriting broad databases, all unit thermodynamics, or a universal multiphase solver in Rust. Adding a backend should require mapping and qualification, not replacing material meaning throughout the application.

A sensible initial deployment is a **modular Rust host with coarse-grained provider adapters**, using local worker processes where runtime or containment needs justify them and in-process calls where a suitable qualified interface exists. Send a complete flash/chemical problem or a batch of phase evaluations across a costly boundary, not one request per scalar property or inner iteration. Reuse compatible sessions; measure before choosing a permanent transport. Do not make each responsibility an independently deployed service.

Preserve the possibility of faster integration without making native ABI convenience the first scientific decision. The canonical semantics should survive moving a provider from a worker to an in-process adapter or replacing it with native equations.

For the broader project's data fabric, keep durable/queryable definitions, parameters, provenance, bindings, and evidence distinct from derived process/equation topology and hot numerical arrays. This fits the prior Arrow/DataFusion, graph, and incremental-compilation direction without making those technologies thermodynamic contracts. Ordinary solver iterates should not require a relational query or a durable revision per arithmetic operation. Resolve a compact numerical context once, calculate efficiently, and attach the captured semantic references to the resulting evidence.

## 10. Reliability without architectural bureaucracy

Keep definition proposal, private calculation, assessment, and current adoption distinct. All current adoption goes through one logical authority that checks matching dependencies, live run/ancestor permission, scope, and coherent target generation at commit. This need not imply a distributed transaction system or microservices. [B7 AC-45–49]

Same command plus same payload returns the prior decision; it must not apply an incremental transfer twice or reinstall an old result over a newer one. A logical cancellation revokes future publication, not necessarily the native call's execution. A still-running or uncertain session cannot be pooled or freed prematurely. Verify the whole activation/input/evaluation/readback sequence, not just one function. If fatal native failure would defeat required host-state protection, use real containment rather than assuming an exception wrapper is sufficient. [B7 AR-05–10; B8 IR-18/21]

Archive semantic reconstruction inputs, not only final arrays or native handles. A successfully loaded case can be inspectable yet not executable. Changing a valid input leaves the model edited even if the subsequent solve fails; prior outputs become stale rather than quietly reverting the input. Save/restore must not rerun hidden estimation, revive cancelled authority, or replay physical effects.

Start with correct conservative invalidation and straightforward local ownership. Optimize dependency granularity, caching, session pooling, and parallel execution after the semantic tests pass. The catalogs do not require expensive instrumentation of every arithmetic operation.

## 11. Evidence and the next concrete implementation sequence

The strongest completed evidence is an authored scenario/ownership review, synthetic thermodynamic and unit formulations, two extracted ChEDL numerical functions, and a bounded sequential publication model with deliberately unsafe controls. Complete candidate packages were not executable in the earlier environment. The included source-kernel tests are not full `thermo`/`chemicals` tests, the synthetic materials are not real-fluid validation, and the sequential guard model is not a concurrency proof. Those boundaries are recorded in B9 and must remain attached to the evidence.

Begin with a thin vertical implementation that captures a configured package, one local material problem, an actual provider result, original-target checks, and protected result adoption. Then exercise a mixer/heater/PH valve/separator route with an explicit conventional package. Add a pure-water saturation and utility case through a second provider before treating the adapter abstraction as adequate. Test a mass-only caloric material and an irreversible representation map early so the host does not accidentally become molecular-only.

Next qualify nonideal and multiple-liquid fixtures, then a coherent chemical system with energy/exchanges, then the actual staged/nonequilibrium formulation required. Petroleum/black-oil and real empirical data remain explicit work, not something supplied by the existence of custom-component constructors. Advance coupled optimization only when the chosen arrangement's total derivatives, branch behavior, conditioning, and error budgets have been demonstrated.

Pin a real material, actual selected parameters, backend/build/adapter, operation, phase/chemistry policy, solver arrangement, and acceptance thresholds **before** promoting a fixture. Structural links or a successful source import are not evidence of whole-scenario execution. The supplied full-package runner is a starting scaffold whose package-dependent branches were not exercised in B9; inspect and adapt it to the chosen manifest rather than assuming it is a certified integration.

**The implementation thesis:** own physical meaning and process composition; reuse coherent scientific engines; never confuse a convenient numerical return with fulfillment of the original physical request; and keep enough attributable context to explain, change, reproduce, and safely replace every accepted flowsheet result.

## Sources and how to use the archive

B1 through B9 are the numbered reports listed in [START_HERE.md](START_HERE.md). In particular, B4 is the normative behavior source; B5 fixes ownership; B6 fixes information meaning; B7 fixes actions/lifecycle; B8 fixes integration admission; B9 supplies the bounded evidence overlay. B2/B3 explain why the decisions were made. This brief is a design synthesis, not a silent revision of those baselines.

Read this brief and the resource index first, then consult B8 for a chosen integration profile and B9 for its evidence limits. Use B5/B6/B7 selectively while implementing the relevant boundary, and the registers to resolve exact identifiers and witnesses. The source reports contain pinned implementation references; moving documentation in the resource index is navigation, not a replacement for those pins or an instruction to install an unqualified latest release.
