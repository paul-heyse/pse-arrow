---
title: "Cases, specifications, and studies"
notion_source: "https://app.notion.com/p/40b141b58369818e80d5d58cd8bf0119"
notion_last_edited: "2026-11-01T23:34:04.596Z"
---

# Cases, specifications, and studies

Source: [Notion](https://app.notion.com/p/40b141b58369818e80d5d58cd8bf0119). Imported 2026-11-02.

**Version 1.1** · 2026-11-01 · [Flowsheet Process Domain Model](index.md)

## Keep the process reusable

A process model defines physical relationships and admissible configurations. An operating case selects conditions and specifications for one use of a checked model revision. A study defines the engineering question and any relationship among cases. A numerical plan chooses an admitted preparation and solution route. A run records one actual attempt and its outcomes.

The same exchanger can be rated, sized, optimized and fitted without changing the identity of its heat-transfer area. The roles attached to that quantity change. A fitted quantity must nevertheless be exposed as an admissible model parameter or decision coordinate; a study cannot silently turn an immutable declaration into a variable. If the required role is not supported, an explicit model revision or interface refinement is necessary.

The repository already separates case interpretation, source of starting values, temporary overrides and solve preparation. A warm start, a parameter replacement and a temporary initialization fix have distinct ownership. This design extends that separation into a process-facing vocabulary. [Case interpretation](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs); [workflows and results](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/workflows-and-results.md).

## Mapping to the overall formulation contract

These process terms specialize the overall model; they do not create a second Specification, Query, Study, or execution authority.

| Process-facing term | Overall contract |
| --- | --- |
| Checked process-model revision | Versioned Vocabulary and instantiated Model |
| Operating case | Specification: bindings, input/initial data, observations, selected realizations, and sources of operating requirements |
| Individual operating/design specification | A binding or active Query condition as its meaning requires; reusable physical laws remain Model relations |
| One atomic engineering task | One SimulationProblem with an explicit Query over the selected model and case |
| Process study composing atomic tasks | The existing overall Study, containing problem nodes and explicit data or iteration relationships |
| Numerical plan and run | The existing Execution plan and Run contracts |

A Study may contain one atomic task. Conversely, several experiments jointly estimating shared parameters can belong to one coupled Infer problem with multiple scenarios; they must not be split into independent fits merely because the cases have different names. A decomposition into multiple problem nodes requires explicit coupling and convergence semantics.

## Operating-case contract

Each case references exactly one checked process-model revision and its selected configuration. It contains zero or more explicit assignments and specifications, with source provenance and units. Shared defaults are resolved visibly, and conflicting assignments to the same active occurrence are rejected. An inherited case is an authored derivation whose effective content is explicit; it is not mutable ambient state.

| Kind | Meaning | Consequence |
| --- | --- | --- |
| Fixed value | A selected quantity is held at a stated value | Removes the corresponding freedom through the admitted binding mechanism |
| Design or operating equation | A relationship such as product purity or approach temperature | Adds a physical/engineering condition with an explicit owner |
| Bound or admissible region | A permitted range or feasibility relationship | Constrains the solution without pretending to be an observation |
| Initial condition | A physical condition at the start of a temporal domain | Must be consistent with the algebraic system and storage contract |
| Input profile | A specified function or sampled trajectory over time | Carries interpolation, discontinuity and time-basis meaning |
| Observation | A measured quantity with mapping and uncertainty/weight | Supports fitting or checking; is not automatically a hard equation |
| Numerical start | A guess or prior solution used for preparation | Does not alter the original physical problem |

Engineering specifications can span units, such as a recycle ratio, product recovery, plant utility limit or control relation. They belong at the smallest composition scope containing the referenced quantities. IDAES likewise distinguishes internal unit constraints from flowsheet-level operating or control constraints. [IDAES flowsheet model](https://idaes-pse.readthedocs.io/en/stable/explanations/components/flowsheet/index.html).

## Structural closure and specification roles

Before solving, determine whether the selected case is structurally appropriate for the intended study. A rating calculation typically closes the remaining degrees of freedom. An optimization intentionally leaves declared decision freedoms and adds objectives and feasibility constraints. An estimation study couples observations and shared unknowns across cases.

Equation and variable counts are a useful diagnostic, not proof of independence or solvability. Retain the responsible specification identities so an overdetermined or underdetermined result can explain which conditions conflict or remain absent. Structural rank, numerical conditioning, physical-domain checks and original-equation checks complement counting.

The UI may call the same scalar `input`, `output`, `design variable` or `estimated parameter` in different contexts. Those labels describe a case/study role. They do not redefine the physical type or force causal execution. A solved outlet temperature can be specified in a sizing study while area becomes the unknown.

## Time, operating steps and control

A temporal domain states its physical unit, extent and relevant boundary conditions. An operating mode selects active physical relationships or boundary behavior. An operating step applies one mode over a time interval or until a declared transition condition. A transition identifies continuity, permitted instantaneous transfer/reset and any required reinitialization.

An operating procedure composes physical steps and transitions for a unit or train. A production campaign links physical batches or procedure executions through equipment availability, retained material, transfers, schedules and any required lot custody. These are operating intent within the model/case. A Study may simulate or optimize them, but computational dependency edges alone do not define physical continuity or material transfer. This distinction requires explicit contracts, not a separate general-purpose recipe or scheduling framework.

These concepts are different from numerical initialization stages, solver iterations or an outer study loop. A recycle initialization step does not represent a plant operation. A parameter-sweep point is not a point in physical time. A finite physical stage is not a collocation point.

Keep physical process time, sampling clocks, cycle-relative coordinates, acquisition timestamps and economic periods distinguishable. Numerical multirate stepping/interpolation belongs to realization; equipment sequencing and hold times belong to operating intent. An observation collected over an interval requires its stated integration or averaging map. A cyclic result needs an explicit cycle-closure criterion rather than visual similarity of successive endpoints.

For a batch or cyclic process, step boundaries may change active feed/product connections while inventories persist. Continuity is the default only for quantities whose physical definition supports it; explicit discharge, charge, switching or idealized instantaneous events require their own balance accounting. Cyclic steady state adds a periodic relationship between the end and beginning of the cycle. The IDAES TSA example demonstrates multiple operating steps, including a closed batch cooling step, while the PID example demonstrates time-dependent disturbances and control. These examples motivate the contracts without establishing that every generalized hybrid route is already implemented in pse-arrow. [TSA](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/models_extra/temperature_swing_adsorption/fixed_bed_tsa0d.html); [PID](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/control/pid.html).

The existing dynamic contract preserves conserved-subject identity through modes and independently assesses original flux and permitted event transfers. Its admitted modes share a fixed state/parameter layout, with explicit restrictions on algebraic resets, simultaneous actionable events and event-bearing simultaneous realization. Changing laws or connections must not erase retained inventory. A physically changing state space requires an explicit state mapping and admitted execution contract; the existence of modes does not establish that capability. [Current temporal contracts](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/workflows-and-results.md).

A controller is a physical/operational submodel connecting measured quantities, internal state and manipulated quantities. Sensor dynamics, actuator bounds and control laws are explicit where modeled. A control signal does not acquire material or energy conservation semantics merely because it uses the same generic port mechanism.

## Studies as compositions of questions

Useful study profiles include steady or dynamic simulation, equipment rating, sizing, constrained optimization, sensitivity, uncertainty propagation, data reconciliation and parameter estimation. Profiles can overlap; admissibility depends on concrete required capabilities, not a universal enum of five answer kinds.

A study references one or more cases and identifies their coupling. In a multi-experiment fit, geometry or correlation parameters may be shared while each experiment has distinct feeds and observations. In a design optimization, economic relationships and environmental constraints are model extensions over original process quantities. In a sensitivity study, the chosen perturbations and response definitions are explicit. IDAES's costing and estimation documentation support this separation. [Costing](https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/costing/costing_framework.html); [estimation and reconciliation](https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html).

Observation mappings identify the measured quantity, material/phase or population representation, location, time or aggregation interval, basis, datum and error model. A measured bulk composition cannot silently stand for a particular phase composition, and a collected average cannot silently become a pointwise state value. Measured values, fitted parameters, predictions, correlations and applicability evidence keep distinct roles and provenance. Missing observations, censored values and unavailable uncertainty remain explicit. Lot or sensor identifiers refer to selected material/model occurrences; acquisition timestamps are mapped to physical process time rather than treated as interchangeable.

## Engineering assessment scope

An engineering assessment consumes selected physical quantities/results and explicit economic, environmental or other engineering assumptions. Declare its system boundary, included equipment and flows, period or campaign, utilization, currency/base date where relevant, factor provenance, allocation rule and functional unit. Capital cost, operating cost per period, physical emissions and characterized environmental impact have different meanings. Annualized throughput is not the same as a stored process inventory.

Assessment models may supply objectives or constraints through the existing Query and authored-equation mechanisms. Changing allocation or an accounting boundary must not change the underlying physical conservation or double-count internal transfers, shared utilities and intermediate products. Prices and impact factors do not become conserved material identity. The current equipment-costing example remains a scoped correlation with its applicability limits; broader assessment support requires its own selected data and evidence. These contracts do not require a second analysis engine, general financial platform or life-cycle database. [Current costing model](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/process/models/costing.pse); [assessment choices in BioSTEAM](https://biosteam.readthedocs.io/en/latest/tutorial/Life_cycle_assessment.html).

## Numerical plans and evidence

Numerical choices include spatial/time approximation, initialization, continuation, scaling, solver selection and requested derivatives. They are bound to a concrete model/case/study and checked against available capabilities. They cannot weaken an original conservation obligation or replace a physical initial condition with a convenient start value.

A result retains the original request, model revision, effective specifications, numerical selection, actual execution status and checked physical outcomes. Solver success, physical acceptance, complete output publication and reference agreement remain separate. An unexecuted design case, a source-inspected implementation and a successfully qualified workflow must never share an ambiguous `supported` label.

The coverage catalog on [Flowsheet Process Domain Model](index.md) records design witnesses and the current source evidence. [Repository mapping and acceptance workflows](repository-mapping-and-acceptance.md) explains how these contracts attach to the existing implementation and how the exchanger exercises their separation.
