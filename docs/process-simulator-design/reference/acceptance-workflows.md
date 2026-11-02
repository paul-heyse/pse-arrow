---
title: "Acceptance Workflows"
---

# Acceptance Workflows

<a id="source-40b141b583698134a615ce21514f3e6e"></a>

## Composite solid oxide cell and replaceable provider

Source: [Notion](https://app.notion.com/p/40b141b583698134a615ce21514f3e6e).

**Contracts:** Local multidimensional physics can sit inside a process-facing unit; external interfaces remain stable across compatible detailed or reduced models.

**Repository evidence:** [IDAES 2.13 solid oxide cell](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/models_extra/power_generation/unit_models/soc_submodels/solid_oxide_cell.py) demonstrates the two-dimensional local reference model. No native pse-arrow SOC or replacement-provider parity is claimed.

**Priority:** P3

**Acceptance:** Expose fuel/oxygen/material and electrical/thermal meaning; conserve cross-region transfers and reaction/charge conventions; disclose differences in derivative, dynamic and validity capabilities when changing provider/fidelity.

**Status:** Design case

**Workflow:** Composite solid oxide cell and replaceable provider

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981dea2a7ec9fdabd84d4"></a>

## Cyclic adsorption with batch step

Source: [Notion](https://app.notion.com/p/40b141b5836981dea2a7ec9fdabd84d4).

**Contracts:** Operating mode/step distinct from study and solver stage; changing port activity, retained inventory, periodic closure and explicit transitions.

**Repository evidence:** [IDAES TSA](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/models_extra/temperature_swing_adsorption/fixed_bed_tsa0d.html) supplies concrete behavior. Existing [temporal workflow owners](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/workflows-and-results.md) do not by themselves qualify this generalized process cycle.

**Priority:** P2

**Acceptance:** Represent heating, closed cooling, pressurization and adsorption; preserve or account for storage across transitions; assess cyclic state closure and product accounting over a full cycle.

**Status:** Design case

**Workflow:** Cyclic adsorption with batch step

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b58369813692c4cbb8a02cd537"></a>

## Distributed reactor with mesh refinement

Source: [Notion](https://app.notion.com/p/40b141b58369813692c4cbb8a02cd537).

**Contracts:** Physical coordinate and geometry distinct from mesh; local material states, spatial flux, sources and boundary conditions.

**Repository evidence:** [Current 1D control volume](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volume-1d.pse) and [reactors](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/reactors.pse) are source evidence; the explicit dynamic refusal is part of the present limit.

**Priority:** P1

**Acceptance:** Preserve physical identity across mesh refinement; check independent integrated source/flux closure and boundary meaning; quantify numerical change after execution. Dynamic spatial use must refuse until time-accumulation closure is implemented.

**Status:** Partial source evidence

**Workflow:** Distributed reactor with mesh refinement

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981b0a225c49a02503f92"></a>

## Dynamic vessel with pressure control

Source: [Notion](https://app.notion.com/p/40b141b5836981b0a225c49a02503f92).

**Contracts:** Storage separate from material state and flow; physical initial conditions, input disturbances and controller state distinct from numerical starts.

**Repository evidence:** [Inventory structures](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs), [stored control volumes](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volumes.pse) and [documented process library](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md) provide partial source evidence. No new closed-loop run was performed.

**Priority:** P1

**Acceptance:** Check consistent initial conditions and algebraic equations, dynamic inventory-versus-flux closure, control signal semantics and declared actuator limits; retain precise event/disturbance behavior and rejected unsupported storage bases.

**Status:** Partial source evidence

**Workflow:** Dynamic vessel with pressure control

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981f8afc4eda0e207da27"></a>

## Electrolyte or apparent-to-true species case

Source: [Notion](https://app.notion.com/p/40b141b5836981f8afc4eda0e207da27).

**Contracts:** Material-system scope, true/apparent composition mapping, sparse phase membership and charge/element meaning.

**Repository evidence:** [Physical semantics](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md) and [selected equilibrium interfaces](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/thermodynamics/models/equilibrium.pse) identify integration boundaries only. They do not establish electrolyte or arbitrary-speciation support.

**Priority:** P2

**Acceptance:** Map inlet/reporting components to true species explicitly; verify permitted conservation laws and property capabilities; reject unavailable charge/composition data and ambiguous basis conversions.

**Status:** Design case

**Workflow:** Electrolyte or apparent-to-true species case

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b58369813a9839db842ee15157"></a>

## Explicit translator between material/state conventions

Source: [Notion](https://app.notion.com/p/40b141b58369813a9839db842ee15157).

**Contracts:** Compatible direct connections versus explicit species, basis, datum or state-coordinate translation.

**Repository evidence:** [Connection and translator contract](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md) and [state specialization](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs) provide partial infrastructure evidence. No concrete new translator pair was executed.

**Priority:** P0

**Acceptance:** Use a genuinely incompatible pair; demonstrate complete mapping and original mass/element/energy meaning where defined; reject missing mappings and mismatched datums; state information loss for aggregation.

**Status:** Partial source evidence

**Workflow:** Explicit translator between material/state conventions

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b58369814fa509ef635cd462e4"></a>

## Gas-solid reactive contactor

Source: [Notion](https://app.notion.com/p/40b141b58369814fa509ef635cd462e4).

**Contracts:** Distinct gas and solid states, phase-specific temperatures, solid inventory and heterogeneous reaction/transfer basis.

**Repository evidence:** [Generic composition and conservation](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md) provide the target integration route. No complete pse-arrow gas-solid workflow is asserted or executed.

**Priority:** P2

**Acceptance:** Conserve declared elements and energy over both regions; retain solid loading and gas flow as distinct quantities; check interphase transfer cancellation and required material characterization.

**Status:** Design case

**Workflow:** Gas-solid reactive contactor

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981de8392d00689e55d2c"></a>

## Heat exchanger across rating and sizing

Source: [Notion](https://app.notion.com/p/40b141b5836981de8392d00689e55d2c).

**Contracts:** Stable physical quantities under different specification roles; two fluid submodels and one paired internal energy exchange.

**Repository evidence:** [Current exchanger](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse) composes two heater children and a thermal exchange. The five-use worked example on the mapping page is target design, not a qualification report.

**Priority:** P0

**Acceptance:** Rate with fixed geometry and size with a permitted geometry degree of freedom; verify both original side balances, internal cancellation, approach/validity requirements and structural closure; diagnose conflicting duty/outlet-temperature specifications.

**Status:** Partial source evidence

**Workflow:** Heat exchanger across rating and sizing

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981698573f23e64a86a22"></a>

## Multi-stream contactor with side feed

Source: [Notion](https://app.notion.com/p/40b141b5836981698573f23e64a86a22).

**Contracts:** Units can own custom indexed states/balances without standard CV ancestry; inter-stream transfer, side ports and heterogeneous rate basis are explicit.

**Repository evidence:** [IDAES MSContactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html) is the reference witness. No concrete pse-arrow implementation or execution evidence for this complete case is asserted.

**Priority:** P2

**Acceptance:** Represent co/countercurrent streams and a side feed/draw; ensure transfer cancels in the enclosing system; declare reaction extent basis and heat attribution; distinguish physical stages from approximation elements.

**Status:** Design case

**Workflow:** Multi-stream contactor with side feed

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981bd931ccc2f4c571c62"></a>

## Reactive equilibrium and Gibbs formulation

Source: [Notion](https://app.notion.com/p/40b141b5836981bd931ccc2f4c571c62).

**Contracts:** Kinetic/stoichiometric reaction models and element-based Gibbs equilibrium fit one compositional domain without requiring the same internal mechanism.

**Repository evidence:** [Authored reactor definitions](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/reactors.pse) are partial source evidence. [IDAES Gibbs reactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/gibbs_reactor.html) supplies a reference behavior, not evidence of pse-arrow Gibbs parity.

**Priority:** P1

**Acceptance:** Check declared species/phase membership, elemental and charge conservation where known, thermodynamic assumptions and energy convention; incomplete elemental composition must refuse an elemental-balance claim.

**Status:** Partial source evidence

**Workflow:** Reactive equilibrium and Gibbs formulation

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b5836981ca9f29d157d286a5a8"></a>

## Recycle flowsheet with flash and purge

Source: [Notion](https://app.notion.com/p/40b141b5836981ca9f29d157d286a5a8).

**Contracts:** Hierarchical process assembly, directed material ports, phase split, recycle identity and solution tears separate from physical topology.

**Repository evidence:** [Authored recycle wrapper](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/campaign/models/recycle-flash.pse) and [flow projection](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-structural/src/flowsheet.rs) are partial source evidence. Complete selected workflow and reference results must be attached after execution.

**Priority:** P0

**Acceptance:** Express one complete feed/mix/heat/react/separate/recycle flow; preserve component and energy closure; retain connection identities through any tear strategy; reject incompatible species/port bindings. No new execution is claimed.

**Status:** Partial source evidence

**Workflow:** Recycle flowsheet with flash and purge

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.


<a id="source-40b141b583698126a709caa6a0e9816e"></a>

## Shared-parameter exchanger fitting and optimization

Source: [Notion](https://app.notion.com/p/40b141b583698126a709caa6a0e9816e).

**Contracts:** One process model serves multiple experiments and engineering questions with explicit shared unknowns, observations and objectives.

**Repository evidence:** [Runtime case preparation](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs) and [study/fitting owners](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/workflows-and-results.md) are partial source evidence. Complete exchanger multi-use qualification remains to be performed.

**Priority:** P1

**Acceptance:** Fit correlation parameters across cases; distinguish observation residuals from physical closure; diagnose U-area confounding; preserve case-specific feeds, shared parameter identity and truthful local/global optimization claims.

**Status:** Partial source evidence

**Workflow:** Shared-parameter exchanger fitting and optimization

Part of [Flowsheet Process Domain Model](../process/index.md), version 1, 2026-11-01.

See [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md) for the pinned baseline, acceptance procedure and implementation sequence.

#### Evidence rule

No new simulator or parity run was performed during this design review. “Partial source evidence” means related definitions or contracts were inspected, with the limitations in Repository evidence. “Design case” means the workflow remains a proposed acceptance witness. Priority is the proposed implementation sequence, not a claim that a feature is missing or complete.

When execution evidence becomes available, retain the exact model and reference revisions, assumptions, specifications, numerical route, observed outputs, tolerances, original physical checks and exclusions with the result.
