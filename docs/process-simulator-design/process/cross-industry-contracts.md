---
title: "Cross-industry domain contracts"
notion_source: "https://app.notion.com/p/40b141b5836981bca17dde73c867160c"
notion_last_edited: "2026-11-01T23:34:00.651Z"
---

# Cross-industry domain contracts

Source: [Notion](https://app.notion.com/p/40b141b5836981bca17dde73c867160c). Imported 2026-11-02.

[Plan detailing and industry coverage assessment](../rationale/plan-detailing-assessment.md)

**Status: Adopted domain contracts** · 2026-11-01. Incorporated into process-domain v1.1, its concept catalog and the implementation plan. This page supplies detailed rationale and representative design cases. Source evidence remains pinned to `19e7b49a67df24c1bf73005a6186e145e930d53a`; no builds, simulations or tests were run. [Flowsheet Process Domain Model — v1.1](../rationale/version-records.md#source-40b141b5836981958b9ad5a14519520f).

## Adopted cross-industry design constraints

The following constraints govern the relevant authoring, scientific-package, preparation and runtime changes in P0–P7 and in later selected increments. They preserve the existing owners and source authority. Adoption establishes design obligations, not implementation or qualification evidence, and does not commit the candidate cases below.

The adopted domain is a sound foundation beyond IDAES. It already distinguishes definitions from instances, states from inventories, direct connections from translators, physical operating steps from solver stages, and conservation from accounting. Its flexible unit composition and open scientific packages avoid the most consequential restrictions. The implementation plan also explicitly rejects a second equipment graph and reserves generic changes for demonstrated gaps.

**Current seed formulations must not become universal authoring contracts.** Molecular/molar descriptions, the particular liquid–vapor representation, fixed-layout dynamic modes and the current costing correlation remain scoped supported formulations. The relevant public contracts must preserve material representation, basis, operating continuity, capability and assessment distinctions, with generic changes justified by a concrete consumer. Current principles support this approach: model only the distinctions that govern behavior, keep each fact with its owner, and prove extension locality through concrete changes.[1](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/core/design-principles.md) [2](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/models-and-composition.md)

## I04.1 — Material identity, characterization and representation

Retain the existing concept of a material constituent, but require authors to distinguish the identified subject, the characterization evidence selected for this calculation, and its numerical/scientific representation. A petroleum fraction derived from one assay and cut rule is not interchangeable with a similarly named fraction from another assay. Refining a representation may preserve selected observables while changing others; it is not automatically a reversible coordinate conversion. DWSIM's documented petroleum characterization illustrates the distinction between bulk/curve evidence, generated pseudocomponents and the represented material stream.[3](https://dwsim.org/help/main/06-simulation.html)

Material mappings must declare their domains, preserved quantities and information loss. Exact changes of coordinates or units remain distinct from aggregation, estimated characterization and a replacement physical model. A pseudocomponent may carry empirical molecular weight, density and boiling information without an invented complete formula. Bind mass–mole conversion only to an admitted existing authority for the required empirical data; where that authority is insufficient, resolve it in the selected package or owning generic contract before claiming the conversion.

**Concrete source pressure:** `chemistry.species` permits unknown composition, but its derived molecular mass relies on complete composition and atomic weights. The named `apparent` and `lumped` kinds require complete participant composition and charge, and enforce exact element/charge mapping. Preserve these strong contracts. An assay-based empirical lump or yield model with weaker evidence must not acquire their guarantees by being called “lumped”; express the weaker scientific contract explicitly in packages.[4](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/physical/models/chemistry.pse)

## I04.2 — Constituents, populations and quality measurements

A particle-size class is not generally a chemical species. A particle can have composition, size, shape, moisture and internal structure; their joint distribution may matter even when each marginal distribution is known. Polymer chain-length distributions present a related pressure. Published BHET crystallization research provides a concrete example in which an internal particle-size distribution, kinetic fitting and downstream filtration affect the engineering result.[5](https://pmc.ncbi.nlm.nih.gov/articles/PMC11378152/)

The minimal commitment is that material descriptions may contain declared population coordinates and distributions alongside chemical composition. Their measures and normalizations must be explicit: number, mass or volume weighting, density versus per-bin amount, and the relevant coordinate units. Moments are selected observations, not a promise to reconstruct the complete distribution. Rebinning must state the invariants it preserves and its approximation. Breakage can conserve mass while changing particle count.

Use existing authored entity sets, axes, functions, balances and physical types first. Do not introduce a universal particle hierarchy or fixed list of internal coordinates. A selected milling, crystallization or other population-bearing case establishes whether the present continuous/indexed machinery is sufficient before a generic extension is introduced. Record the scientific population model separately from numerical binning, while recognizing that an empirical grouping rule can itself change scientific meaning.

## I04.3 — Conservation mappings and scoped evidence

Translator and balance admission must answer: **which extensive quantities can both sides actually identify and preserve?** For a linear representation map y = Mx, and conservation projections C\_source and C\_target, require C\_target Mx = C\_source x over its declared domain for each claimed conserved quantity. This is a mathematical contract for authored mappings, not a prescribed new matrix API. Nonlinear mappings need the corresponding explicitly checked relation. Missing composition information must not become zero or be replaced by an unqualified mass-only claim about elemental closure.

The existing quantity owner already distinguishes kind, basis, datum, subject and indexed axes; conservation, observation and accounting are distinct accumulator modes. Preserve that machinery. Extend authored quantities when needed for dry versus wet basis, standard-volume conditions, catalyst-based rates, particle measures, biomass lumping or COD-like characterization. Equal dimensions do not establish interchangeable meanings. A reported quality index, environmental impact indicator or product purity is not automatically additive or conserved.[6](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/physical-semantics.md) [7](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-modeling/src/specialize/process.rs)

Likewise, empirical conversion/yield models need declared source accounting and the conservation evidence their data supports. Current `reactions.Projection` rightly derives coefficients from an admitted reaction rather than arbitrary callbacks. Do not weaken its exact reaction contract to accommodate every industrial yield model; determine whether an additional authored transformation contract suffices.[4](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/physical/models/chemistry.pse) [8](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/thermodynamics/models/reactions.pse)

## I05.1 — Flow, stored inventory and identifiable lots

A stream view describes transfer through a boundary. Inventory describes retained extensive physical state. A lot or batch identifier describes material lineage and possibly selected characterization; it is neither a new species nor the thermodynamic state itself. Mixing and splitting need explicit treatment of any lineage information demanded by the application. A label that does not affect the scientific calculation can remain documentary; selected lot-specific properties and measurements are scientific inputs with their usual provenance.

Do not derive vessel contents by multiplying an instantaneous stream by an arbitrary duration. Specify the storage/mixing model: well-mixed tank, segregated compartments, residence-time distribution or another declared law. A zero-flow material boundary can coexist with changing inventory and reaction. A closed batch process must remain representable without inventing continuous product flow.

Reversible transport also deserves an explicit capability boundary. Negative flow on a directed port does not supply upstream composition/enthalpy selection, zero-flow behavior or junction mixing. Modelica's stream specification establishes why transported quantities require additional semantics for reversal. Retain the current directed contract where it applies; add reversal through a concrete network or transfer case.[9](https://specification.modelica.org/maint/3.6/stream-connectors.html)

## I05.2 — Operating programs and computational studies

The domain already contains operating modes and steps. Each selected operating program declares active laws/connections, duration or transition condition, initial/terminal requirements, retained inventories, allowed transfers/resets and observations. Distinguish a physical cycle from a solver iteration and a campaign's successive batches from independent study occurrences. CADET-Process demonstrates useful cases where timed events alter flow, connectivity and collection behavior within a cycle.[10](https://cadet-process.readthedocs.io/en/v0.11.1/user_guide/process_model/process.html)

Current dynamics already preserves conserved-subject identity across modes and independently checks inventory change against original flux and event transfers. Its admitted modes share one state/parameter layout; algebraic resets, simultaneous actionable events and event-bearing simultaneous realization have explicit restrictions. These are exact starting limits, not evidence that operating programs are absent.[11](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/workflows-and-results.md)

Different physical time scales do not require different notions of physical time. Keep process time, sampling clocks, cycle-relative coordinates, acquisition timestamps and economic periods distinguishable. Numerical multirate stepping and interpolation are realization choices; equipment sequencing and hold times are operating intent. A periodic result needs a stated cycle-closure criterion, not merely a last trajectory that appears settled. Defer a general production scheduler or recipe standard until a selected campaign requires it; shared-equipment and material-transfer constraints can initially be ordinary authored equations and operation dependencies.

## I03 — Substitutability for the requested use

Two implementations are substitutes only for a stated contract and use: outputs, material representation, phase/root convention, validity region, reference datum, accuracy, derivatives, initialization requirements and effects. Equal ports do not guarantee that a detailed contactor and a surrogate expose the same internal fields, optimization curvature or transient memory. A compatible interface permits composition; it does not establish scientific equivalence.

Use the existing demanded-property and native-capability mechanisms. Value-only eligibility, first derivatives, second-order products and global-certification eligibility must remain separate. A change from one EOS to another is a selected scientific-model change even if both implement the same interface. Conversely, a qualified implementation replacement of the same model should preserve the declared observations within its contract, without requiring identical internal variables or algorithms.[6](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/physical-semantics.md) [12](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-kernels/src/provider.rs)

## I06 — Equation exposure, evaluation and time advance

Retain equation-oriented composition as the principal source of scientific meaning, while allowing explicit procedural or external implementations where a consumer needs them. Distinguish three operations: contributing residual equations, evaluating a requested input-to-output relation, and advancing a stateful model over time. A legacy simulator, reduced-order model or external detailed unit must declare which it supplies.

The present provider boundary already owns mutable attempt bindings, derivative demands and effect-sensitive completion; it must not be described as universally pure. But a provider call alone does not establish time-advance, rollback, event synchronization or conservative co-simulation. FMI's separate Model Exchange, Co-Simulation and Scheduled Execution interfaces demonstrate those distinct responsibilities.[12](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-kernels/src/provider.rs) [13](https://fmi-standard.org/docs/3.0.2/)

A time-advancing external model needs declared input interpolation/hold, accepted time interval, output coverage, state custody, restart/restore capability, events, algebraic-loop participation and failure behavior. Repeated nonlinear residual evaluation must not accidentally advance it. Coupling error and conservation assessments are separate from each component's internal solver success. FMI and CAPE-OPEN are useful candidate adapter standards with different scopes; their interfaces do not establish equivalent thermodynamics or conservative dynamic coupling. Select their actual versions and capabilities for a concrete consumer. Defer a universal co-simulation coordinator and mandatory adapters. [CAPE-OPEN thermodynamics 1.1](https://www.colan.org/specifications/thermodynamics-and-physical-properties-interface-specification-v1-1/). First select one real coupling and extend the existing runtime/provider owner only for the contract it demonstrably needs; keep its effects outside pure compiler reuse.

## I07 — Economic and environmental accounting scopes

Selected costing and environmental assessments are authored models consuming physical results, with explicit coupling when they are optimization objectives or constraints. Each declares its system boundary, included equipment/flows, period or campaign, utilization, currency/base date, factor provenance, allocation and functional unit as applicable. Capital and annual cost, physical emission and characterized impact, and operating inventory and annualized throughput remain distinct.

Current exchanger costing supplies a sourced equipment correlation with explicit applicability; it does not define a universal economic model. BioSTEAM's published biorefinery example illustrates that identical physical outputs can produce different allocated impacts under energy, revenue and displacement conventions. Those are explicit assessment choices, not changes to physical conservation.[14](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/process/models/costing.pse) [15](https://biosteam.readthedocs.io/en/latest/tutorial/Life_cycle_assessment.html)

Avoid double-counting internal heat/material transfers, shared utilities or intermediate coproducts when changing the assessment boundary. Do not put prices or impact factors into the conserved material identity. Defer jurisdiction-specific reporting frameworks, comprehensive finance and life-cycle databases; retain enough declared scope and provenance for a later consumer to reproduce the calculation.

## I08 — Discriminating process cases

These remain uncommitted candidate cases for resolving a concrete contract question or for a later selected delivery increment. They are not claims of current support, new P1 gates or additions to the frozen IDAES reference population. Each exposes a different failure of an overly narrow domain; selection and finite scope remain with the existing plan owner.

| Case and industry | Distinction it must exercise | Decisive evidence |
| --- | --- | --- |
| Assay-characterized crude separation and blending; refining | Real components plus empirical fractions; mass/mole/volume bases; alternative cut resolution | Reconcile assay totals and selected product observables; state mapping loss; reject unsupported elemental claims |
| Mill–classifier–silo recycle; minerals/cement | Joint size/composition/moisture distributions; holdup; segregation | Mass and selected composition closure despite changing particle count; conservative rebinning; correct storage response |
| Reactive crystallizer, filter and dryer; specialty/pharmaceutical chemicals | Dissolved species, solid population, nucleation/growth and wet/dry products | Solute/solvent and energy accounting across phases; particle-measure consistency and drying basis |
| Fed-batch fermentation with downstream recovery; biotech/food | Empirical biomass, yield/rate models, oxygen transfer, additions and harvest | Closed inventory accounting over the entire batch; explicit composition-knowledge limits; no invented steady throughput |
| Load–wash–elute chromatography or cyclic adsorption; biopharma/separation | Operating sequence, connection switching, retained adsorbate, product collection | Cycle inventory closure, integrated yield/purity, and explicit cyclic-stationarity criterion |
| Electrolytic treatment with precipitation; water/metallurgy | Apparent/true species, electroneutrality, charge transport and solids | Consistent mass/element/charge mappings and electrical/chemical energy accounting |
| Polymerization and product-grade transition; polymers | Segment/chain moments, distribution-dependent properties, changing specifications | Monomer/segment and energy closure; justified moment closure and comparison to product-distribution evidence |
| Steam/thermal network with storage and reversal; power/utilities | Reversing advective transport, junctions, state collapse and multiple response times | Correct composition/enthalpy transport through zero flow; inventory/energy closure; truthful derivative availability |
| External detailed furnace or custom unit inside a recycle; steel/process integration | Stateful versus residual/evaluation coupling, initialization, optional rollback | Repeatable agreed calls, no hidden advance, converged coupling and original boundary conservation |
| Biorefinery with shared utilities and coproducts; renewables/pulp/food | Physical boundary versus economic/environmental allocation and time basis | Same physical run reused under declared assessment choices; no duplicated utilities or inconsistent functional units |

## I07 — Scientific data and observations

A selected method, characterization, calibration or assessment must retain its actual input records and provenance. Keep measured values, fitted parameters, predictions, correlations and applicability evidence distinct. Observation mappings need quantity meaning, basis, sampling/aggregation interval, uncertainty or error-model assumptions and missing-data treatment. A measured average over a collection interval is not automatically a pointwise state value.

Use current physical-context, scientific-record, case, fitting and result owners. Cross-experiment fits retain shared parameter identity; fit and independent prediction evidence keep their distinct roles. When later integrating plant data, acquisition timestamps map explicitly to process time, while equipment tags and sensor identifiers remain external references to selected model occurrences. A design-stage model need not identify an installed asset.

NIST ThermoML preserves measurement methods, constraints, uncertainty and bibliographic information at data-point level; NIST separately states that representation checking does not constitute critical evaluation. This supports the data contract, without requiring a particular import implementation now. [NIST ThermoML](https://www.nist.gov/mml/acmd/trc/thermoml).

## Decisions before the relevant coding slice

Before implementing the relevant public authoring contract, settle identity and mapping meaning, the quantities actually conserved, supplied/receiving-state responsibilities, temporal continuity and the requested capability. These are adopted constraints on the relevant P0–P7 coding slice. They do not require every industry's package design to be settled before P1 and do not replace the implementation sequence.

| Choice | Implementation alternatives and adopted starting rule |
| --- | --- |
| Material mapping | Ordinary authored functions/relations keep science local; a new generic mapping construct enables shared discovery/enforcement but adds language obligations. Start authored; promote only when a concrete consumer needs the missing contract. |
| Operating configurations | A fixed superset of states/laws fits current mode machinery; changing layouts can represent genuinely changing state spaces more directly but requires explicit state mapping and new native admission. Use the fixed layout where faithful, never erase inventory through deactivation. |
| External model | Exposed residuals support coupled equation solving; input/output evaluation accommodates an internal solve; co-simulation preserves an external integrator. The latter choices need declared derivatives, effects and coupling limits. Select by the actual unit and analysis; no universal wrapper makes them interchangeable. |

Defer industry-wide taxonomies, a universal population solver, campaign scheduler, adapter ecosystem and assessment database. Defer implementation choice, not the distinction needed to prevent silent information loss, uncontrolled time advance or false conservation claims.

## Guidance for implementation and owner inspection

Keep P1's existing recycle/exchanger delivery. Before freezing broad authoring contracts, use selected source/design examples to check an empirical material mapping, a population-bearing inventory and a physical sequence. Implement an executable probe only when it resolves a consequential uncertainty; these examples do not automatically add scientific families or tests to P1. The external-coupling case can initially end in an explicit supported/refused contract. These probes should resolve semantic uncertainty, not initiate a new generic framework or full industry library.

For each selected increment, the coding agent states the scientific question, the distinctions that must survive, the selected existing owner, the admitted configurations and the smallest discriminating example. It inspects that owner's actual consumer before choosing a package-only addition or a generic extension. Maintain source identities and one scientific authority; the conceptual catalog must not become parallel persistence.

| Question requiring exact source inspection | Start with these current owners |
| --- | --- |
| Empirical material identity, partial characterization, exact versus empirical mappings | `packages/reference/physical/models/chemistry.pse`; property-record selection in `packages/reference/domain/models/properties.pse`; quantity basis/datum owners |
| Non-molar or population-bearing state and transfer | `packages/reference/thermodynamics/models/equilibrium.pse`; `crates/pse-modeling/src/specialize/process.rs`; authored control volumes and physical operation contracts |
| Batch continuity, modes and external time advance | `crates/pse-modeling/src/specialize/continuous.rs`; `crates/pse-runtime/src/workflow/dynamics.rs`; existing native dynamic profile, events, study and staged owners |
| Value/derivative substitution and executable boundary | `crates/pse-kernels/src/provider.rs`; function specialization; compiler demand/physical admission; native capability routing |
| Assessment boundary and annualization | `packages/reference/process/models/costing.pse`; current accumulator and result-selection owners; authored assessment packages |

Do not weaken exact chemistry, physical types or original-space acceptance to make a broad example fit. A broader ambition should increase the range of explicitly modeled science while keeping each supported claim narrow enough to verify.
