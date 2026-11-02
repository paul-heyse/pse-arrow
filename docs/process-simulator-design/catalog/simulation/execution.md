---
title: "Simulation concepts — Execution"
---

# Simulation concepts — Execution

<a id="source-40a141b5836981c69c35c755377f1b5f"></a>

## Accuracy model

Source: [Notion](https://app.notion.com/p/40a141b5836981c69c35c755377f1b5f).

**Refers to:** Settings

**Rules:** Absence of a general error model does not prohibit execution supported by the required original-equation/physical assessments. A kernel cannot claim an unsupported accuracy guarantee; reported evidence states whether it is proved, estimated, assumed or unavailable.

**Multiplicity:** 0..1

**Definition:** A kernel's declared relationship between method settings and error evidence: certified bounds, asymptotic estimates, empirical comparisons, residual checks or statistical estimates, with assumptions and coverage.

**Layer:** Execution

**Owned by:** [Kernel declaration](execution.md#source-40a141b5836981bd834fd7f6fd6aa9b6)

**Concept:** Accuracy model

<a id="source-40a141b58369816a9236c03ab2986b9e"></a>

## Capability registry

Source: [Notion](https://app.notion.com/p/40a141b58369816a9236c03ab2986b9e).

**Refers to:** Forms, kernels

**Rules:** A registry entry or graph path alone does not establish contextual applicability, scientific validity or guaranteed success. Admission checks the selected closure and distinguishes supported, unsupported and unresolved conditions.

**Multiplicity:** 1

**Owns:** [Form](execution.md#source-40a141b5836981e692d4d74944acec0b); [Kernel](execution.md#source-40a141b583698183b221eb7f64a353db)

**Definition:** The available, versioned implementations and their form interfaces, context requirements, product capabilities and evidence. A graph of admissible transitions is a derived planning view of these contracts.

**Layer:** Execution

**Owned by:** [Execution environment](execution.md#source-40a141b5836981b88948f4c66d859124)

**Concept:** Capability registry

<a id="source-40a141b5836981b88948f4c66d859124"></a>

## Execution environment

Source: [Notion](https://app.notion.com/p/40a141b5836981b88948f4c66d859124).

**Refers to:** Nothing

**Rules:** One per deployment; several may serve one formulation side.

**Multiplicity:** Root

**Owns:** [Capability registry](execution.md#source-40a141b58369816a9236c03ab2986b9e)

**Definition:** The root of execution: its capability registry, planners and runs.

**Layer:** Execution

**Concept:** Execution environment

<a id="source-40a141b583698181a474da42aa85b717"></a>

## Execution plan

Source: [Notion](https://app.notion.com/p/40a141b583698181a474da42aa85b717).

**Refers to:** Problem, kernels

**Rules:** Ordinary dependencies are acyclic; any iterative procedure has an explicit convergence/stop policy, state ownership and completion contract. A plan is derived from a semantic request and method selection, not a replacement domain model.

**Multiplicity:** Root, keyed to a problem's semantic hash

**Owns:** [Plan step](execution.md#source-40a141b583698168bfddf63954d5d738); [Run](execution.md#source-40a141b5836981198e93c8c05502f226)

**Definition:** A graph of numerical operations and artifact dependencies for a selected task, with declared reconstruction mappings and any bounded iterative procedures such as initialization, continuation or accuracy refinement.

**Layer:** Execution

**Concept:** Execution plan

<a id="source-40a141b5836981e692d4d74944acec0b"></a>

## Form

Source: [Notion](https://app.notion.com/p/40a141b5836981e692d4d74944acec0b).

**Refers to:** Nothing

**Rules:** Kernels agree on the exact form version and context they exchange. Class labels such as NLP or DAE do not by themselves prove convexity, index, feasibility, differentiability or implementation support.

**Multiplicity:** 1..n

**Definition:** A versioned mathematical interface in an extensible catalog, including its variables, relations, domains, required products, physical/source mappings and relevant structural assumptions.

**Layer:** Execution

**Owned by:** [Capability registry](execution.md#source-40a141b58369816a9236c03ab2986b9e)

**Concept:** Form

<a id="source-40a141b58369817baec0e8ee63ba79cf"></a>

## Form instance

Source: [Notion](https://app.notion.com/p/40a141b58369817baec0e8ee63ba79cf).

**Refers to:** Form, naming map

**Rules:** Carries its source/naming map and derivation context. Prepared forms do not become a competing authored model authority, and a solver or analysis step need not produce another form.

**Multiplicity:** 0..n produced or consumed per plan step

**Owns:** [Naming map](execution.md#source-40a141b5836981e9b66de73444d0da22)

**Definition:** A concrete derived mathematical artifact for a selected model/case/task, with retained assumptions, physical conventions, source mapping and consumed dependencies.

**Layer:** Execution

**Owned by:** [Plan step](execution.md#source-40a141b583698168bfddf63954d5d738)

**Concept:** Form instance

<a id="source-40a141b583698183b221eb7f64a353db"></a>

## Kernel

Source: [Notion](https://app.notion.com/p/40a141b583698183b221eb7f64a353db).

**Refers to:** Forms

**Rules:** Admission uses the actual implementation, form, context and requested products, not a family label alone. Kernel capabilities are extensible and do not determine the scientific domain model.

**Multiplicity:** 1..n

**Owns:** [Kernel declaration](execution.md#source-40a141b5836981bd834fd7f6fd6aa9b6)

**Definition:** A versioned execution capability with a declared input/output contract. Lowering, solver, evaluation, analysis and planner are the initial capability families; an implementation may expose several separately declared capabilities.

**Layer:** Execution

**Owned by:** [Capability registry](execution.md#source-40a141b58369816a9236c03ab2986b9e)

**Concept:** Kernel

<a id="source-40a141b5836981bd834fd7f6fd6aa9b6"></a>

## Kernel declaration

Source: [Notion](https://app.notion.com/p/40a141b5836981bd834fd7f6fd6aa9b6).

**Refers to:** Forms, capability descriptors

**Rules:** Separate pre-execution conditions that are checkable from conditional assumptions and runtime/post-run assessments. Unknown applicability is not a proved capability; a requested guarantee is claimed only with its supporting evidence. Failure and unavailable-product behavior are part of the contract.

**Multiplicity:** 1

**Owns:** [Settings](execution.md#source-40a141b5836981ed8fa0f2a39595c624); [Accuracy model](execution.md#source-40a141b5836981c69c35c755377f1b5f)

**Definition:** A versioned contract identifying accepted input forms and contexts, produced artifacts/results, required evaluation products, settings, resource policy and the scope of any accuracy or correctness evidence.

**Layer:** Execution

**Owned by:** [Kernel](execution.md#source-40a141b583698183b221eb7f64a353db)

**Concept:** Kernel declaration

<a id="source-40a141b5836981d88e2ae4c186763cc8"></a>

## Lift

Source: [Notion](https://app.notion.com/p/40a141b5836981d88e2ae4c186763cc8).

**Refers to:** Naming maps

**Rules:** Compose only the products and validity conditions each step supports. Eliminations, approximations, relaxations, smoothing and discretization may lose information or change guarantees; unsupported reconstructions are withheld with reasons, and original-space assessment remains separate.

**Multiplicity:** 1 per lowering plan step

**Definition:** A qualified reconstruction/projection from a derived result to an earlier semantic representation. It may recover values, selected duals or derivatives and translate diagnostics or error evidence; it is not generally the inverse of a lowering.

**Layer:** Execution

**Owned by:** [Plan step](execution.md#source-40a141b583698168bfddf63954d5d738)

**Concept:** Lift

<a id="source-40a141b5836981e9b66de73444d0da22"></a>

## Naming map

Source: [Notion](https://app.notion.com/p/40a141b5836981e9b66de73444d0da22).

**Refers to:** Slices, relations

**Rules:** Every produced artifact has a declared origin or derivation; generated auxiliaries are identified as such rather than assigned fictitious original variables. Report mappings retain units and transformations and state when an original value or dual cannot be recovered.

**Multiplicity:** 1

**Definition:** A source/derivation map relating numerical coordinates, rows and generated auxiliaries to authored identities, index selections and transformation steps. The relationship may be one-to-many or many-to-many.

**Layer:** Execution

**Owned by:** [Form instance](execution.md#source-40a141b58369817baec0e8ee63ba79cf)

**Concept:** Naming map

<a id="source-40a141b583698168bfddf63954d5d738"></a>

## Plan step

Source: [Notion](https://app.notion.com/p/40a141b583698168bfddf63954d5d738).

**Refers to:** Kernel, settings, form instances, realizations

**Rules:** Check knowable admission conditions before effects and retain runtime/post-run findings separately. A step may produce no candidate or only partial products; its failure or cancellation does not imply scientific infeasibility.

**Multiplicity:** 1..n

**Owns:** [Form instance](execution.md#source-40a141b58369817baec0e8ee63ba79cf); [Lift](execution.md#source-40a141b5836981d88e2ae4c186763cc8); [Settings](execution.md#source-40a141b5836981ed8fa0f2a39595c624)

**Definition:** One admitted kernel operation over named input artifacts with explicit settings and requested products; it retains the selected source/context and any conditional assumptions.

**Layer:** Execution

**Owned by:** [Execution plan](execution.md#source-40a141b583698181a474da42aa85b717)

**Concept:** Plan step

<a id="source-40a141b583698141bd2ed18322efd79d"></a>

## Raw result

Source: [Notion](https://app.notion.com/p/40a141b583698141bd2ed18322efd79d).

**Refers to:** Form instance

**Rules:** Native coordinates may be inspected with their form/source mapping. They are not presented as qualified semantic results without supported reconstruction and assessment; a failed step may produce no candidate.

**Multiplicity:** 0..n candidate payloads; termination evidence is retained independently

**Definition:** Native outputs and termination evidence from an execution step before scientific reconstruction and original-space assessment.

**Layer:** Execution

**Owned by:** [Run](execution.md#source-40a141b5836981198e93c8c05502f226)

**Concept:** Raw result

<a id="source-40a141b5836981198e93c8c05502f226"></a>

## Run

Source: [Notion](https://app.notion.com/p/40a141b5836981198e93c8c05502f226).

**Refers to:** Plan steps

**Rules:** Cancellation, native termination, candidate availability, scientific qualification and recording completion are distinct. Provenance supports reproducibility investigation; identical bitwise results or successful repeat execution are not universal guarantees.

**Multiplicity:** 0..n

**Owns:** [Run event](execution.md#source-40a141b5836981f398f2cb78c08514f3); [Raw result](execution.md#source-40a141b583698141bd2ed18322efd79d)

**Definition:** One execution attempt of an admitted plan and selected immutable inputs, retaining its actual starts, settings, environment and completion evidence.

**Layer:** Execution

**Owned by:** [Execution plan](execution.md#source-40a141b583698181a474da42aa85b717)

**Concept:** Run

<a id="source-40a141b5836981f398f2cb78c08514f3"></a>

## Run event

Source: [Notion](https://app.notion.com/p/40a141b5836981f398f2cb78c08514f3).

**Refers to:** Plan steps, slices, relations

**Rules:** Timestamped.

**Multiplicity:** 0..n

**Definition:** Progress, warnings and checkpoints during a run, translated through naming maps where possible.

**Layer:** Execution

**Owned by:** [Run](execution.md#source-40a141b5836981198e93c8c05502f226)

**Concept:** Run event

<a id="source-40a141b5836981ed8fa0f2a39595c624"></a>

## Settings

Source: [Notion](https://app.notion.com/p/40a141b5836981ed8fa0f2a39595c624).

**Refers to:** Nothing

**Rules:** Settings satisfy the selected kernel schema and admission policy. Scientific model/parameterization and physical formulation choices remain separately identified; changing a numerical setting does not silently change process assumptions.

**Multiplicity:** 1 schema per kernel declaration; 1 value per plan step

**Definition:** Typed numerical method controls such as mesh resolution/order, integration/transcription method, tolerances, sampling count and resource allowances, with exact versions and defaults retained.

**Layer:** Execution

**Owned by:** [Kernel declaration](execution.md#source-40a141b5836981bd834fd7f6fd6aa9b6); [Plan step](execution.md#source-40a141b583698168bfddf63954d5d738)

**Concept:** Settings
