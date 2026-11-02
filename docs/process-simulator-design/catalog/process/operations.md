---
title: "Process concepts — Operations"
---

# Process concepts — Operations

<a id="source-40b141b583698137af0ffce422514ee6"></a>

## Control model

Source: [Notion](https://app.notion.com/p/40b141b583698137af0ffce422514ee6).

**Cardinality:** Zero or more measured and manipulated quantities with declared control law.

**Invariants:** Signal coupling carries no implicit material conservation; actuator limits and internal state are explicit.

**Area:** Operations

**Implementation:** Existing authored PID noted in composition owner; broader control behavior requires specific evidence.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/control/pid.html

**Definition:** A submodel relating measured signals, controller state and manipulated process quantities.

**Owner:** Unit/subflowsheet composition

**Relationships:** Uses typed signal ports, input profiles, actuator/sensor models and dynamic state where needed.

**Concept:** Control model

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/control/pid.html).


<a id="source-40b141b5836981078640d2a0a286f368"></a>

## Initial condition

Source: [Notion](https://app.notion.com/p/40b141b5836981078640d2a0a286f368).

**Cardinality:** As many independent conditions as the admitted dynamic formulation requires.

**Invariants:** Initial conditions must be consistent with original equations; numerical starts cannot substitute for them.

**Area:** Operations

**Implementation:** Existing original inventory initial-condition structures and selected temporal routes.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs

**Definition:** A physical condition on stored states or relevant coordinates at the start of a temporal domain.

**Owner:** Dynamic operating case or operating step

**Relationships:** References inventories, algebraic consistency and temporal domain; distinct from numerical seed.

**Concept:** Initial condition

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).


<a id="source-40b141b5836981d49e7af311fc28c3ec"></a>

## Input profile

Source: [Notion](https://app.notion.com/p/40b141b5836981d49e7af311fc28c3ec).

**Cardinality:** Zero or more profiles; one target quantity or declared indexed set per profile.

**Invariants:** Time, units, interpolation, extrapolation and discontinuities are explicit; scenario index is not time.

**Area:** Operations

**Implementation:** Existing temporal workflows; target clear process-boundary profile contract.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/control/pid.html

**Definition:** A prescribed time-dependent boundary or operating quantity with explicit time and interpolation meaning.

**Owner:** Operating case or step

**Relationships:** Supplies dynamic cases and controller setpoints; may include declared discontinuities.

**Concept:** Input profile

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/control/pid.html).


<a id="source-40b141b5836981dc967cf359c5cfa2d3"></a>

## Material lot

Source: [Notion](https://app.notion.com/p/40b141b5836981dc967cf359c5cfa2d3).

**Cardinality:** Zero or more lot references per operating context; splits and merges explicitly identify predecessor/successor relationships where needed.

**Invariants:** Lot identity is not chemical species, phase or thermodynamic state. Lineage does not create physical mass or replace transfer equations. Conceptual design cases need no fictitious production lot or installed asset.

**Area:** Operations

**Implementation:** Adopted v1.1 design contract. Add lineage at existing source/case/result owners when a selected operating consumer needs it; no manufacturing execution system is required.

**Source URL:** https://app.notion.com/p/40b141b58369818e80d5d58cd8bf0119

**Definition:** An identifiable material quantity or batch with custody and lineage across production, transfers, splitting and blending, linked to its selected characterization and physical inventory.

**Owner:** Operating procedure/campaign material lineage referencing model occurrences

**Relationships:** Material characterization; Inventory; Operating case; Operating procedure; Production campaign; Observation mapping.

**Concept:** Material lot

Adopted design contract in [Flowsheet Process Domain Model](../../process/index.md). This is a semantic role over existing authored identities; it does not prescribe a one-to-one runtime class or persistence record. The structured properties contain the definition and invariants.

Owning narrative: [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md).

Implementation and scientific qualification remain separate from documentation adoption.


<a id="source-40b141b5836981b4b822c1cc24c0cdba"></a>

## Operating case

Source: [Notion](https://app.notion.com/p/40b141b5836981b4b822c1cc24c0cdba).

**Cardinality:** Exactly one model revision; zero or more assignments, specifications and initial/input conditions.

**Invariants:** Defaults and overrides resolve explicitly; conflicting active specifications reject without mutating the model. Changing selected scientific records, public demand or free-variable layout is not merely a numerical-value edit. Retained physical state across procedures is explicit.

**Area:** Operations

**Implementation:** Existing case binding and solve preparation; target process-facing role vocabulary.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs

**Definition:** An immutable selection of conditions and specifications for one checked process-model revision.

**Owner:** Case authoring and runtime interpretation

**Relationships:** Referenced by studies and runs; binds original quantity identities. May select Material characterization, Material lot, Operating procedure, Production campaign and Engineering assessment assumptions through existing owners.

**Concept:** Operating case

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs).


<a id="source-40b141b58369811ba6b8e0c90df13cf9"></a>

## Operating mode and step

Source: [Notion](https://app.notion.com/p/40b141b58369811ba6b8e0c90df13cf9).

**Cardinality:** One or more ordered/linked steps for a sequence; zero or more transition conditions.

**Invariants:** Operating steps are distinct from solver/initialization stages; resets and transfer accounting are declared. A physical step is distinct from thermodynamic phase. State carryover, boundary switching, additions/removals and event transfers are explicit. Existing fixed-layout/event routes do not imply arbitrary changing state spaces.

**Area:** Operations

**Implementation:** Target generalized process contract; existing event/temporal infrastructure does not imply all hybrid workflows.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/models_extra/temperature_swing_adsorption/fixed_bed_tsa0d.html

**Definition:** A selected physical configuration or boundary behavior applied over a temporal interval or until transition.

**Owner:** Operational model or case

**Relationships:** Controls active laws/connections, state continuity and explicit event transfers. Operating procedure composes steps; Production campaign links procedure/lot occurrences and required resource constraints.

**Concept:** Operating mode and step

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/models_extra/temperature_swing_adsorption/fixed_bed_tsa0d.html).


<a id="source-40b141b58369811680b3e6c2685a63a0"></a>

## Operating procedure

Source: [Notion](https://app.notion.com/p/40b141b58369811680b3e6c2685a63a0).

**Cardinality:** One or more ordered or concurrent operating steps; zero or more event conditions, resource uses and transfer/reset declarations.

**Invariants:** A physical operating step is distinct from a thermodynamic phase, solver stage and Study node. Retained inventories and declared additions/removals remain accountable across transitions; closed batches require no artificial steady flow. Actual route limitations remain explicit.

**Area:** Operations

**Implementation:** Adopted v1.1 design contract over authored models, temporal/event and case owners. Existing fixed-layout and event support does not imply arbitrary changing layouts, cyclic qualification or campaign scheduling.

**Source URL:** https://app.notion.com/p/40b141b58369818e80d5d58cd8bf0119

**Definition:** An authored physical program composing operating steps, permitted transitions, transfers, collection and state carryover over selected equipment.

**Owner:** Reusable operational definition selected by a case

**Relationships:** Operating mode and step; Input profile; Initial condition; Inventory; Material lot; Unit model; Production campaign.

**Concept:** Operating procedure

Adopted design contract in [Flowsheet Process Domain Model](../../process/index.md). This is a semantic role over existing authored identities; it does not prescribe a one-to-one runtime class or persistence record. The structured properties contain the definition and invariants.

Owning narrative: [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md).

Implementation and scientific qualification remain separate from documentation adoption.


<a id="source-40b141b5836981bd93f0f3cb433f769c"></a>

## Production campaign

Source: [Notion](https://app.notion.com/p/40b141b5836981bd93f0f3cb433f769c).

**Cardinality:** One or more procedure or lot occurrences; zero or more equipment, calendar, storage and resource constraints.

**Invariants:** Successive batches require explicit physical carryover and transfers. Independent Study scenarios do not automatically form a campaign. Simulating a prescribed schedule, checking its conflicts and optimizing a schedule are distinct capabilities.

**Area:** Operations

**Implementation:** Adopted v1.1 design contract. Begin with selected procedures and explicit schedules through current owners; a generic production scheduler is deferred until a concrete scoped consumer requires it.

**Source URL:** https://app.notion.com/p/40b141b58369818e80d5d58cd8bf0119

**Definition:** A physical execution context for multiple procedure/lot occurrences with equipment assignment, material carryover, storage, time and shared-resource constraints.

**Owner:** Authored operating/campaign context referenced by cases and studies

**Relationships:** Operating procedure; Material lot; Inventory; Unit model; Input profile; Operating case; Study; Engineering assessment.

**Concept:** Production campaign

Adopted design contract in [Flowsheet Process Domain Model](../../process/index.md). This is a semantic role over existing authored identities; it does not prescribe a one-to-one runtime class or persistence record. The structured properties contain the definition and invariants.

Owning narrative: [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md).

Implementation and scientific qualification remain separate from documentation adoption.


<a id="source-40b141b58369811ab7cefc9e7030264b"></a>

## Specification

Source: [Notion](https://app.notion.com/p/40b141b58369811ab7cefc9e7030264b).

**Cardinality:** Zero or more specifications per case; explicit target occurrences and applicability.

**Invariants:** A fixed value, bound and observed value are different; structural count alone does not prove closure.

**Area:** Operations

**Implementation:** Existing case bindings and authored equations; target explainable specification ownership.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/explanations/components/flowsheet/index.html

**Definition:** An individual process specification item: a hard assignment, engineering condition or admissible region for a case/task; not a replacement for the overall Specification container.

**Owner:** Case or smallest relevant composition scope

**Relationships:** Uses the overall binding/Query-condition contracts according to meaning; references original quantities and records degrees-of-freedom responsibility. Reusable physical laws remain Model relations.

**Concept:** Specification

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/explanations/components/flowsheet/index.html).
