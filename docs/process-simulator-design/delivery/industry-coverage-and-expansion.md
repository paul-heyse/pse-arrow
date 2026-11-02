---
title: "Industry coverage and expansion strategy"
notion_source: "https://app.notion.com/p/40b141b583698188810fd7f19710de19"
notion_last_edited: "2026-11-01T23:34:01.406Z"
---

# Industry coverage and expansion strategy

Source: [Notion](https://app.notion.com/p/40b141b583698188810fd7f19710de19). Imported 2026-11-02.

[Plan detailing and industry coverage assessment](../rationale/plan-detailing-assessment.md)

**Status: Adopted product direction; candidate expansion increments** · 2026-11-01. The semantic direction and scope distinctions are incorporated into process-domain v1.1 and the implementation plan. The industry archetypes and six expansion slices remain candidates for explicit finite selection; this adoption does not claim implemented capabilities or executed qualification. The existing IDAES 2.13 equivalence program retains its own acceptance population.

## I08.1 — Define the ambition in engineering terms

The adopted product horizon is **a native process-modeling environment capable of expressing, composing, studying and qualifying the major classes of industrial process models**. Process models, material descriptions, physical methods, operating procedures and studies remain the organizing structure. Industry names identify library profiles and qualification campaigns; they do not create separate simulation architectures.

The scope cannot be closed by listing every industrial unit operation. A refinery, a fermentation plant and a mineral-processing circuit expose different gaps in the same underlying contracts. The useful high-level plan therefore has two linked documentation views over existing source, case and evidence owners:

- **Capability families:** material characterization, thermodynamics, transformations, transport, distributions, operations, networks, economics and evidence.
- **Industry archetypes:** bounded engineering problems that exercise combinations of those families, with their own data, assumptions, ranges and expected observables.

“Full range” is a continuing product ambition. An implementation milestone must still identify which families, configurations, operating domains and workflows are delivered. One successful demonstration is evidence for that case and its qualified domain; it cannot establish coverage of an entire industry.

The sources below establish that these demands exist in process modeling. Vendor capability descriptions are scope evidence, not independent proof of accuracy or specifications for reproducing proprietary implementations. The design constraints below are adopted: selected packages, authoring contracts and execution routes must preserve these distinctions. They do not automatically add a scientific family, adapter, industry case or numerical gate to P0–P7. Candidate expansion work becomes delivery scope only through the existing plan owner, with a finite configuration/case population and evidence requirements.

## I08.2 — Cross-cutting capability families

### A. Characterized materials, not only named molecules

Petroleum characterization generates pseudocomponents from bulk measurements or distillation curves. DWSIM documents C7+ characterization, ASTM/TBP curves and optional density, viscosity and molecular-weight information. These are materially different inputs from selecting pure chemicals from a database. [S1](https://dwsim.org/help/main/06-simulation.html)

**Target contract:** a material description can contain molecular species, characterized fractions, empirical lumps and structured populations. Preserve the original assay, measurement basis, cut scheme, estimation method and uncertainty alongside the generated components. A regenerated characterization is a new revision with an explicit mapping to the previous one. Do not silently manufacture an elemental formula or assume that every empirical lump has a meaningful molecular weight.

This supports crude cuts, biomass fractions, ore grades, coal, food ingredients and recycled feedstocks without assigning them a fictitious common chemistry. Libraries supply the appropriate characterization and transformation rules.

### B. More than one meaning of “component” and “balance”

WaterTAP ASM1 gives a decisive counterexample to a molecular-only domain. It represents soluble and particulate organic matter, biomass, combined nitrogen measures and alkalinity; it declares a liquid phase while using COD-based quantities and a separate alkalinity concentration. [S2](https://watertap.readthedocs.io/en/stable/technical_reference/property_models/ASM1.html)

**Target contract:** distinguish chemical identity, model component, phase membership and measurement/balance basis. A particulate population need not imply a thermodynamic solid phase in a lumped model. COD-equivalent concentration is not ordinary species mass concentration, and alkalinity cannot be converted using a generic molecular-weight rule.

Each selected package must declare the quantities it can balance and the conversions it justifies: mass, elements, charge, selected constituents, COD equivalents or other calibrated aggregates. Internal transformations expose residuals for the checks they claim. Missing composition data limits which physical checks can be claimed; it must not be converted into invented composition or an unsupported elemental-closure guarantee.

### C. Electrolytes and reacting multiphase mixtures across domains

OLI distinguishes mixed-solvent electrolyte models from their hydrocarbon/acid-gas extensions, with different phase and applicability considerations. This illustrates why a single “electrolyte supported” flag is insufficient. [S3](https://support.olisystems.com/hc/en-us/articles/40538954395923-Thermodynamic-Framework-Limits-in-OLI-Software)

**Target contract:** record true/apparent species conventions, chemical speciation, charge, concentration scales, solvent mixtures, possible precipitates and validated temperature/pressure/composition ranges. Select equilibrium and transport methods independently where appropriate. Couple precipitation, dissolution and acid/base reactions to the balances without forcing equilibrium on a kinetically limited unit.

Chemicals/fertilizers, brines, minerals recovery, wastewater and acid-gas treatment share this need. Separate reference packages and datasets should qualify their materially different chemistry.

### D. Particle populations and product structure

A published BHET crystallization study models nucleation, growth/dissolution and attrition through a population balance and connects crystal-size behavior to downstream filtration and process economics. Its particle-size coordinate is not a physical spatial mesh. [S4](https://pmc.ncbi.nlm.nih.gov/articles/PMC11378152/)

**Target contract:** associate a distribution with a material or phase population, identify its internal coordinates and physical meaning, and declare whether it is represented by bins, moments or another closure. Preserve the relationship between distribution measures and conserved mass/volume. Transformations may change number, size or morphology while retaining appropriate physical balances.

This enables crystallization, milling, classification, agglomeration, granulation, drying and selected emulsions. Implement concrete reusable population contracts through cases rather than designing a universal distribution solver before any case exists.

Polymers add another structured-material requirement. Aspen documents molecular-weight distributions, branching, composition and polymer-specific kinetic and property models. [S5](https://home.aspentech.com/en/products/pages/aspen-polymers) When a selected polymer model requires segment/chain descriptors or moments, its kinetics, transport and product-quality calculations must be able to use them with declared meaning and closure assumptions. A single fixed molecular weight and a scalar conversion are not an adequate universal representation.

### E. Biochemical transformations and quality attributes

**Target contract:** support calibrated yields and kinetics for growth, substrate utilization, maintenance, decay, inhibition and product formation, with declared dependencies on operating conditions. Biomass and substrate definitions must retain their basis and parameter provenance. Allow measured quality attributes to be model variables or observations when they influence the equations, rather than treating all quality information as a report annotation.

When a bioprocess increment is selected, define its bounded engineering question and the upstream/downstream, utility and waste boundaries needed to answer it. Detailed cell metabolism or binding models may be supplied by specialized packages when that question requires them; they are not mandatory dependencies of every fermentation model.

The same pattern serves food and agricultural processing: moisture, composition, recovery and selected quality kinetics can govern process decisions. VTT BALAS explicitly covers food, biochemical and pulp/paper processes and links site balances to water reuse and heat integration. [S6](https://www.vttresearch.com/en/knowledge-base/vtt-balas-conceptual-process-design-optimise-resource-efficiency) Product-specific quality models require their own experimental evidence.

### F. Batch, semi-batch, continuous and campaign operation

Equipment topology and an operating recipe are separate objects. Intelligen’s scheduling documentation distinguishes operation timing, shared equipment and resource conflicts; it also explicitly distinguishes detecting a proposed schedule’s conflicts from automatically optimizing a schedule. [S7](https://docs.intelligen.com/spd/SPDWebHelp/Bk7/Chapter7/Chapter7.htm)

**Target contract:** represent ordered or concurrent operations, conditions/events, state carryover, transfers, equipment assignment, cleaning/changeover, finite storage and time-dependent utility demands. A procedure may operate an existing unit in several modes. A campaign instantiates procedures for lots/products and adds resource/calendar constraints. A scheduling study chooses assignments or times; a dynamic study integrates the physical trajectory. Neither replaces the other.

Begin with authored schedules and deterministic conflict checking, then add optimization where demanded. This avoids promising a complete production scheduler merely because batch simulation works.

### G. Networks, utilities and site integration

EPANET demonstrates that network process behavior includes pressure/flow solutions, storage, variable demands, rule-based control and transported water-quality states. [S8](https://www.epa.gov/water-research/epanet) A utility or distribution network is consequently more than a drawing of preassigned stream flows.

**Target contract:** allow network topology with pressure/flow closure, storage and dynamic demand, including justified direction changes. Separate utility supply conditions, physical exchange and economic charges. Preserve the ability to connect heat, electrical power, mechanical work and material boundaries without representing all of them as material streams.

A selected site-level case declares which shared steam/power/cooling/water services and treatment/recycle networks fall inside its boundary. Detailed electrical transients, reservoir flow or atmospheric dispersion remain specialist integrations when the engineering question requires them; their availability does not dictate the process domain.

### H. Economics and environmental accounting as explicit studies

BioSTEAM’s life-cycle tutorial changes allocation methods and system boundaries while evaluating the same sugarcane process. [S9](https://biosteam.readthedocs.io/en/latest/tutorial/Life_cycle_assessment.html) These choices are analysis assumptions, not intrinsic thermodynamic properties.

**Target contract:** keep the physical inventory distinct from prices, economic scenarios, functional units, emissions factors, geography/time, allocation rules and system boundaries. Include direct process emissions and imported/exported utility inventories without double counting. Link costing to equipment configuration and operating history, including uptime, campaigns, consumables and replacement.

Results must expose the assumptions and contributors, so users can inspect why an option is cheaper or has a lower reported footprint. A computed footprint is a qualified study result, not universal environmental certification.

## I08.3 — Candidate industry qualification archetypes

These remain uncommitted candidate targets for planning. Adoption of the design constraints does not select these flowsheets, add them to P1 or change the IDAES acceptance population. A selected increment must define its own inputs, supported configurations, scientific fidelity and qualifying evidence; a single flowsheet does not qualify a sector.

| Industry | Suggested archetype | Distinctive capability combination |
| --- | --- | --- |
| Oil/gas, refining, petrochemicals | Characterized crude or gas-condensate separation with a conversion/blending extension | Assays, real and pseudo components, phase behavior, yield/kinetic transformations, contaminants and product specifications |
| Chemicals and fertilizers | Integrated synthesis/recycle plus acid/solution recovery and solid product finishing | Reaction equilibrium/kinetics, electrolytes, purification, precipitation/granulation, utility integration |
| Pharma and biopharma | Fed-batch production or batch synthesis followed by recovery, crystallization/filtration or chromatography | Biochemical/chemical kinetics, quality/purity attributes, local transport, recipe switching and shared-equipment campaigns |
| Food and agriculture | Ingredient or biomass processing with concentration/drying and coproduct recovery | Variable feed characterization, moisture/quality models, phase and particle behavior, seasonal scenarios, energy/water recovery |
| Water and wastewater | Biological treatment coupled to digestion, nutrient recovery or membrane/brine treatment | Aggregate component bases, ionic chemistry, biomass kinetics, solids/water splits, network and resource accounting |
| Minerals, metals and cement | Ore/recycled-feed preparation followed by leaching/extraction/precipitation or thermal conversion | Mineralogical/grade characterization, size distributions, slurry handling, reactive separation, condensed-phase thermochemistry |
| Pulp and paper | Mill water/chemical/energy circuits linked to drying and recovery | Fibrous/lumped materials, dissolved/colloidal substances, contaminant accumulation, recycle closure, thermal utility systems |
| Polymers | Polymerization with recovery/finishing and a grade-transition study | Segment/chain descriptors, molecular-weight distributions, nonlinear kinetics/properties, changing product specifications |
| Energy, hydrogen and CCUS | Integrated conversion/electrolysis/capture/storage with variable dispatch | Existing IDAES families plus impurity/speciation control, cyclic operation, utilities, degradation/availability scenarios and economic/environmental boundaries |

The minerals and thermal-processing rows have direct complementary evidence: PrOMMiS documents leaching, solvent extraction, precipitation, per-component slurry size distributions, crushing and recovery flowsheets; VTT’s KilnSimu work spans multiphase thermochemical rotary-kiln applications, including lime recovery and cement-related use. [S10](https://prommis.readthedocs.io/en/latest/api.html) [S11](https://cris.vtt.fi/en/publications/advanced-thermochemical-process-model-of-rotary-kilns-kilnsimu/) These sources justify additional model families without implying that either project already covers all minerals or cement operations.

Choose archetypes for the contracts they exercise. Track which capabilities are reused and which remain specific to a library. A new industry should usually add materials, constitutive methods, procedures and evidence—not duplicate the model/case/run architecture.

## I06/I08 — Native process capabilities and specialist coupling

The adopted boundary follows the engineering question and ownership of the equations. Local distributed process models remain eligible native models; this does not commit the product to a general CFD, DEM or specialist spatial platform.

| Native process-model responsibility | Specialist integration when needed |
| --- | --- |
| Composable balances, constitutive equations and empirical/surrogate units | CFD/DEM resolution of mixing, combustion or particle interactions |
| Local distributed reactors, columns, membranes, chromatography, kilns and heat exchangers | Geometric meshing, full-field visualization and specialist spatial solvers |
| Plant equipment performance, process controls, utilities and operational scenarios | Mechanical integrity, detailed structural design, grid transients or reservoir/geological simulation |
| Process-specific biochemical and quality models | Detailed molecular/cell models, pharmacokinetics or end-product performance beyond plant operation |

A selected coupled model must declare inputs/outputs, units/bases, physical meaning, validity, initialization, failure behavior and versioned assets. Dynamic exchange additionally declares time/event semantics; an optimization use declares its supported derivative or derivative-free route. Any claimed native mass/energy qualification includes the coupling boundary and identifies limits caused by unavailable observables.

FMI supplies standardized Model Exchange, Co-Simulation and Scheduled Execution interfaces, but leaves the importer’s coupling/solution algorithm outside the standard. [S12](https://fmi-standard.org/docs/3.0.2/) Therefore “supports FMUs” would be an interoperability milestone, not automatic scientific equivalence, conservation or optimization capability. Do not make every future solver implement a universal simulation ontology before useful process integration is possible.

## I01/I08 — Turning the horizon into implementation increments

### Protect a small set of extension points now

Retain the adopted authored-model authority and shared identities. Extend contracts only where a concrete industry case requires them:

1. Material characterization and component/balance-basis declarations.
2. Population and structured-material descriptors with checked conversions.
3. Constitutive-package capabilities, validity and parameter provenance.
4. Procedure/event/campaign definitions separate from physical models and studies.
5. Resource/network and accounting boundaries.
6. Coupling contracts and qualification evidence.

These are obligations for libraries and preparation/validation behavior, not a request to invent all future schema fields upfront. Avoid a closed industry enum or a mandatory molecular representation. Reject an unsupported basis conversion or model combination explicitly.

### I08 — Select complementary capability increments

Continue the existing P0–P7 IDAES program. Use the following candidate cases to examine current contract choices, and select finite non-IDAES implementation increments through the existing plan owner. Independent scientific families can then proceed concurrently when their shared contracts are settled:

- Assay characterization and pseudo-component separation.
- ASM-style aggregate biochemical components and a basis-aware translator.
- A population-balance crystallization/filtration case.
- A polymer-moment reaction case.
- A batch/campaign case with shared equipment and utility conflicts.
- A site water/energy/accounting case with alternate study boundaries.

These six slices remain candidate starting points for capability exploration. They are not a new diagnostic audit, automatic P1 gate or final industry coverage suite. When one is selected as a delivery increment, its scope includes the reusable native contract/library feature, complete authored case, observables and physical checks, a reference or experimental comparison, and its supported domain. Subsequent expansion follows the same finite selection rule.

### I07/I08 — Make coverage an inspectable claim

Document each capability's supported configuration, required inputs, property/kinetic data, mathematical route, limits, diagnostics and qualifying cases at its existing owner. An industry archetype references those capabilities and the engineering decisions it supports. Use the repository's existing evidence vocabulary and actual conformance/attempt outcomes; avoid a new status taxonomy or global industry-readiness flag.

For each selected capability, qualification must address the applicable original physical balances, population/basis consistency, event/state carryover, parameter/data fit and independent prediction. Choose ranges and adverse cases that challenge that capability, such as changing assays, absent trace components, near-solubility limits, zero/low flow, campaign conflicts, variable loads or distribution resolution. Pin the actual empirical models and datasets. When evidence supports an approximate lumped model, report that fidelity; solver success does not establish a stronger scientific claim.

A useful horizon gives the major requirement classes a conceptual home, a delivery route and an evidence obligation; further domain research will expand the finite populations. Implementation completion remains a finite, versioned claim about delivered capabilities and qualified tasks.

## Primary evidence register

S1. **DWSIM petroleum characterization.** Official help explains characterization from bulk or distillation data into pseudocomponents. [Source](https://dwsim.org/help/main/06-simulation.html).

S2. **WaterTAP ASM1 property package.** Official technical documentation defines lumped components, liquid phase, COD-based quantities and alkalinity. [Source](https://watertap.readthedocs.io/en/stable/technical_reference/property_models/ASM1.html).

S3. **OLI thermodynamic framework limits.** Provider documentation distinguishes MSE and MSE-SRK chemistry/phase applicability. These are provider claims, not universal guarantees. [Source](https://support.olisystems.com/hc/en-us/articles/40538954395923-Thermodynamic-Framework-Limits-in-OLI-Software).

S4. **BHET crystallization for PET recycling.** Primary experimental/modeling research uses population balances, parameter estimation and batch/continuous process evaluation. [Source](https://pmc.ncbi.nlm.nih.gov/articles/PMC11378152/).

S5. **Aspen Polymers.** Official product documentation identifies distribution/structural attributes and polymer kinetics/property families. [Source](https://home.aspentech.com/en/products/pages/aspen-polymers).

S6. **VTT BALAS.** Research-organization documentation demonstrates pulp/paper, food/biochemical and site resource integration scope. [Source](https://www.vttresearch.com/en/knowledge-base/vtt-balas-conceptual-process-design-optimise-resource-efficiency).

S7. **SuperPro scheduling.** Official manual distinguishes operation/campaign timing, resource conflicts and schedule optimization. [Source](https://docs.intelligen.com/spd/SPDWebHelp/Bk7/Chapter7/Chapter7.htm).

S8. **US EPA EPANET.** Official capabilities include hydraulic/storage/control and transported water-quality behavior. [Source](https://www.epa.gov/water-research/epanet).

S9. **BioSTEAM life-cycle assessment.** Official tutorial demonstrates alternative allocations and system boundaries in a biorefinery. [Source](https://biosteam.readthedocs.io/en/latest/tutorial/Life_cycle_assessment.html).

S10. **PrOMMiS API/model reference.** Official documentation identifies hydrometallurgy, precipitation, size-distribution and recovery models. [Source](https://prommis.readthedocs.io/en/latest/api.html).

S11. **VTT KilnSimu research.** Primary conference publication record describes multiphase thermochemical rotary-kiln modeling. [Source](https://cris.vtt.fi/en/publications/advanced-thermochemical-process-model-of-rotary-kilns-kilnsimu/).

S12. **FMI 3.0.2 specification.** The standard defines interface types and responsibilities, including coupling-algorithm boundaries. [Source](https://fmi-standard.org/docs/3.0.2/).
