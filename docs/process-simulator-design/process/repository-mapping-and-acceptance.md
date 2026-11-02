---
title: "Repository mapping and acceptance workflows"
notion_source: "https://app.notion.com/p/40b141b5836981d5a803cdcbc8d1d278"
notion_last_edited: "2026-11-01T23:34:05.270Z"
---

# Repository mapping and acceptance workflows

Source: [Notion](https://app.notion.com/p/40b141b5836981d5a803cdcbc8d1d278). Imported 2026-11-02.

**Version 1.1** · 2026-11-01 · [Flowsheet Process Domain Model](index.md)

## Evidence baseline

This mapping uses repository revision `926655517f6eac4ab52ae7a817a7b72168afdc25`, inspected on 1 November 2026, and the repository's IDAES 2.13.0 parity target. It describes source evidence and proposed work. No simulator, test campaign or parity run was executed for this design review. Existing qualification claims retain their original conditions and exclusions. [Pinned README](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/README.md); [validation owner](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md).

The next implementation should strengthen process-facing contracts over the current authored model system. It should not introduce another equipment builder, independent model database or alternate source of executable equations.

**Supplementary amendment evidence.** The original Version 1 inspection baseline remains recorded above. The contract clarifications additionally use source inspection at `19e7b49a67df24c1bf73005a6186e145e930d53a` on 2026-11-01. The strict chemistry mappings, state/transport identities, fixed-layout temporal scope, provider effects and equipment-costing applicability were inspected at that revision; none of these additions establishes new execution or parity qualification. [Cross-industry domain contracts](cross-industry-contracts.md) records the distinctions and supporting sources.

## Mapping to current owners

| Design responsibility | Existing source evidence | Required direction |
| --- | --- | --- |
| Authored process definitions and composition | [Models and composition](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md) describes generic definitions, interfaces, children, connections and package authoring | Express new process semantics through these definitions; change the kernel only for a demonstrated generic gap |
| Local material state and boundary meaning | [Process specialization](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs) contains `StateSpecification`, `MaterialPort`, transported observations and inventory balances | Expose process views using those same identities; make material-system capability and translation rules explicit |
| Physical type and reference conventions | [Physical semantics](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md) owns physical typing and scientific knowledge | Extend declared vocabulary and package contracts rather than bypassing full physical compatibility |
| Flow topology and recycle | [Flowsheet projection](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-structural/src/flowsheet.rs) and [connectivity contract](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md) retain explicit selected topology | Keep topology, equation incidence and numerical tear choices separate |
| Cases and preparation | [Runtime cases](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs) separates starts, overrides and solve preparation; [analysis](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/analysis.rs) owns analysis interpretation | Add process-facing roles without changing the underlying quantity identity or making initialization permanent |
| Unit library | [Exchanger](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse), [reactors](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/reactors.pse), [control volumes](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volumes.pse) and [recycle wrapper](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/campaign/models/recycle-flash.pse) are authored models | Use representative workflows to refine interfaces and fill specific scientific gaps |
| Distributed models | [1D control volume](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volume-1d.pse) contains domains, discretization, flux balances and independent closure checks | Retain physical-domain versus mesh distinction; dynamic 1D requires a new accumulation closure and qualification |
| Thermodynamic package boundary | [Equilibrium definitions](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/thermodynamics/models/equilibrium.pse) expose selected thermodynamic interfaces and an explicit liquid/vapor equilibrium pairing | Generalize by concrete material/phase requirements; do not infer arbitrary phase or electrolyte support |

The current composition documentation explicitly narrows stored control-volume configurations. The current 1D model explicitly refuses dynamic use. Those are useful, attributable boundaries. The new domain design should make the required extension visible without relabeling present source as complete capability. [Current limits](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md); [1D admission requirement](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volume-1d.pse).

## Extension owners for the clarified contracts

| Contract question | Existing owner to inspect before implementation |
| --- | --- |
| Characterization, empirical constituent data and exact versus empirical mapping | [Chemistry](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/physical/models/chemistry.pse), [scientific-record selection](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/domain/models/properties.pse) and quantity basis/datum contracts |
| Non-molar, population or polymer representation at a material boundary | [Authored state definitions](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/thermodynamics/models/equilibrium.pse), [state/transport/inventory specialization](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-modeling/src/specialize/process.rs) and the consuming authored unit |
| Physical procedures, campaign continuity and state transfer | [Temporal and workflow contracts](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/workflows-and-results.md); existing mode, event, staged, study and native dynamic owners retain their distinct responsibilities |
| Residual exposure, input/output evaluation or external time advance | [Provider boundary](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-kernels/src/provider.rs), compiler demand/physical admission and runtime/native capability owners |
| Costing/environmental boundary, allocation and period | [Authored costing](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/process/models/costing.pse), existing accumulators, physical result selection and authored assessment relationships |

The table identifies where an agent must inspect actual consumers; it does not prescribe new tables, classes or a framework for each contract. Scientific data and equations remain in their existing authored owners. A generic gap is demonstrated before extending the kernel or generated product contracts.

## Worked example: one exchanger, five engineering uses

Consider a countercurrent exchanger joining a hot process stream and a cold utility stream. Its reusable model owns hot and cold fluid submodels, four material ports, a heat-transfer interface, geometry, the selected driving-force relation and the relevant pressure-drop/transport closures. Optional wall and fluid inventories belong to an explicitly supported dynamic formulation.

The current authored exchanger provides a concrete starting point: hot and cold heater children, area and coefficient quantities, a selected temperature-driving-force child, a paired thermal exchange and co-current/countercurrent endpoint relationships. This establishes source-level composition. It does not by itself establish a qualified dynamic wall model, arbitrary phase-change behavior or every use below. [Current exchanger](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse).

### Rating

The case specifies both inlet states, geometry/area, the applicable heat-transfer coefficient or correlation parameters, and the required pressure conditions. It solves outlet states and heat duty. The case must avoid fixing both duty and both outlet temperatures when they already follow from the balances and transfer law.

Acceptance checks the original energy balance on each side, cancellation of the internal heat exchange, all required property-domain conditions, endpoint/approach assumptions of the driving-force formulation, and the remaining original equations. A converged LMTD expression alone does not certify physical feasibility.

### Sizing

Keep the same physical quantity identities. Replace the fixed-area specification with an engineering target, such as a cold outlet temperature, while retaining enough inlet, pressure and transfer information to determine area. The model/study must permit area to be a solved coordinate. If a selected correlation depends on geometry, that dependence remains active rather than being replaced with an assumed constant coefficient.

Acceptance checks the target and both balances, ensures geometry lies in its valid range, and demonstrates that the sizing case is structurally closed. The implementation should report the changed specification roles directly.

### Optimization

Expose permitted geometry, flow or operating decisions and add an objective with declared economic relationships. Constraints can include pressure drop, approach temperature, product temperature and utility availability. Material/energy relationships remain the same physical model; the objective does not replace conservation.

Acceptance distinguishes numerical termination, feasibility in original physical quantities and any optimality statement supported by the chosen route. A local solution must not become a global-optimum claim. Cost parameter units, currency basis and time basis remain explicit.

### Parameter fitting

Use several operating cases with distinct inlet conditions and measured outlet temperatures or duties. Select shared correlation parameters, such as a heat-transfer multiplier or fouling resistance, with experiment-specific nuisance quantities only where justified. Observation mappings identify the measured side, state, time/averaging meaning and uncertainty.

Not every parameterization is identifiable. A single observation of the product `U × A` cannot generally distinguish free `U` and free `A` without additional information. The study should diagnose that ambiguity rather than treating a small residual as proof that both parameters were recovered. Acceptance includes observation residuals, physical closure, sensitivity/rank evidence appropriate to the claim and uncertainty interpretation.

### Dynamics

Select a formulation with explicit fluid and/or wall inventories and their constitutive relations. Define initial conditions consistent with the algebraic constraints, then apply time-dependent inlets or controls. Memoryless pressure or transfer submodels may remain algebraic. Spatial distribution adds physical boundary conditions and a separately selected mesh.

Acceptance checks instantaneous original relations where applicable and integrated storage-versus-flux balance over the simulated interval. It preserves event/step transitions and does not replace initial-condition consistency with a warm-start guess. This use requires additional authored storage and possibly distributed accumulation support beyond the inspected steady exchanger; it remains a design case until executed and qualified.

Across all five uses, model quantities retain identity, case/study roles differ, numerical choices remain separate and each result points back to the original physical obligations. That is the practical test of a well-factored process domain.

## Acceptance workflow

For each named coverage case, first write a behavior brief: required inputs, physical assumptions, exposed outputs, independent checks and unsupported combinations. Identify the specific authored definitions and the exact reference behavior. Then construct the smallest composition that exercises the missing contract. Inspect type, membership, reference and incidence admission before numerical execution.

After implementation, execute the selected numerical routes and assess original physical checks independently of solver convergence. When comparing IDAES, align scientific assumptions, property parameterization, specifications and tolerances; matching equipment names is insufficient. Retain the exact source revisions, conditions, comparison outputs and limitations with the result.

Use adverse cases matched to the contract risk: swapped species order or incompatible datums; unsupported empirical-to-elemental claims; rebinning or moment reduction that loses a demanded quantity; double-counted internal transfers or shared utilities; lost inventory at a mode transition; incomplete interval observations; or an external evaluation that advances hidden state. Retain the existing over-specified duty/temperature, unavailable phase-property and inconsistent-initial-condition cases where relevant. A rejection identifies the authored occurrence and reason; not every increment must execute every adverse case.

## Sequencing

First stabilize reusable identities, state/port/connection contracts and case roles through a recycle flowsheet and exchanger rating/sizing. Next exercise material translation, reactive equilibrium and a dynamic inventory/control example. Then validate staged/distributed and multi-stream composition, followed by solids, cyclic operation and the specialized two-dimensional SOC boundary.

This original witness sequence is retained as design history; the adopted implementation roadmap below owns delivery order. Neither sequence is evidence that its workflows have executed. Existing owners and the coverage catalog decide what evidence exists for each case. The domain is sufficiently specified when these workflows can be expressed coherently, not when every possible unit or physical phenomenon has its own catalog row.

The implementation plan's P0–P7 sequence is the adopted high-level delivery sequence. Industry examples refine the relevant slice's meaning and acceptance boundary; they do not automatically move all population, campaign, coupling or assessment implementation into P1. Source/design examples suffice until an executable probe is needed to resolve a material uncertainty. Existing qualification plans retain their status and acceptance owners.

Return to [Flowsheet Process Domain Model](index.md) for the concept catalog and coverage cases, or [Unified Simulation Domain Model](../simulation/index.md) for the generic mathematical and execution architecture.
