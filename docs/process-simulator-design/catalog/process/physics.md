---
title: "Process concepts — Physics"
---

# Process concepts — Physics

<a id="source-40b141b5836981529c2dec31e97e78a0"></a>

## Balance domain

Source: [Notion](https://app.notion.com/p/40b141b5836981529c2dec31e97e78a0).

**Cardinality:** Zero or more domains per unit; zero or more states, boundaries and inventories per domain.

**Invariants:** A standard control-volume class is optional; domain identity is independent of mesh resolution.

**Area:** Physics

**Implementation:** Existing control volumes and generic composition; target general process-facing region view.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html

**Definition:** A defined physical region over which one or more balances and local constitutive relationships apply.

**Owner:** Unit or subordinate physical model

**Relationships:** May use a reusable control-volume builder or directly authored balances and local interfaces.

**Concept:** Balance domain

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html).


<a id="source-40b141b5836981eeb0eaddc0b0ff23dd"></a>

## Conservation balance

Source: [Notion](https://app.notion.com/p/40b141b5836981eeb0eaddc0b0ff23dd).

**Cardinality:** One balance occurrence per subject/index; zero or more attributed contributions.

**Invariants:** Conservation, closure observation and ordinary accounting are distinct; physical checks use original terms. A declared constituent or aggregate may have generation/consumption; its balance does not assert that the quantity is universally conserved. Quality/COD/cost/impact metrics retain their own meanings and cannot borrow physical conservation guarantees. Mapping or rebinning checks cover only the quantities justified by the selected evidence.

**Area:** Physics

**Implementation:** Existing generic accumulator and conservation contracts.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** A relationship between original signed transport, source and accumulation contributions for a conserved subject.

**Owner:** Balance domain or enclosing composition

**Relationships:** Uses inventories, ports, reaction sources and paired internal transfers; emits independent physical obligations.

**Concept:** Conservation balance

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).


<a id="source-40b141b58369817fad9cef58ecbba007"></a>

## Constitutive closure

Source: [Notion](https://app.notion.com/p/40b141b58369817fad9cef58ecbba007).

**Cardinality:** Zero or more per model; each has declared required inputs, outputs and validity conditions.

**Invariants:** Assumptions and applicability are explicit; numerical convenience does not replace the physical relation.

**Area:** Physics

**Implementation:** Existing authored expressions/function slots; target process intent and applicability projections.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md

**Definition:** An authored physical relationship completing balances, such as pressure drop, heat transfer or equipment performance.

**Owner:** Physical submodel, unit or interface

**Relationships:** Binds original quantities, state properties, geometry and parameters.

**Concept:** Constitutive closure

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md).


<a id="source-40b141b583698153ba30de91a10b6f3d"></a>

## Internal interface and transfer

Source: [Notion](https://app.notion.com/p/40b141b583698153ba30de91a10b6f3d).

**Cardinality:** One interface joins two or more declared sides; each elementary transfer has paired participating sides.

**Invariants:** Internal exchange cancels once in an enclosing balance; external loss or interface storage remains explicit.

**Area:** Physics

**Implementation:** Existing exchange/reflection in exchanger and contribution semantics; target broader material/energy/charge cases.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse

**Definition:** An identified coupling between physical regions with explicit orientation and compatible exchanged quantities.

**Owner:** Common containing model or interface submodel

**Relationships:** Contributes to region balances; may include its own storage, reaction or constitutive law.

**Concept:** Internal interface and transfer

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse).


<a id="source-40b141b58369819e86d2c0121d04abaf"></a>

## Physical geometry

Source: [Notion](https://app.notion.com/p/40b141b58369819e86d2c0121d04abaf).

**Cardinality:** Zero or more dimensions and measures; may be fixed or exposed as admissible design quantities.

**Invariants:** Physical design variables are distinct from numerical mesh controls; consistency equations are retained.

**Area:** Physics

**Implementation:** Existing typed geometry quantities in authored units; target consistent exposure across studies.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse

**Definition:** Dimensions and region/interface measures that determine equipment and constitutive behavior.

**Owner:** Unit or local domain definition

**Relationships:** Used by inventories, area/volume-based rates, transfer closures and costing.

**Concept:** Physical geometry

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse).


<a id="source-40b141b5836981799828f3b425105859"></a>

## Property model or package

Source: [Notion](https://app.notion.com/p/40b141b5836981799828f3b425105859).

**Cardinality:** One selected configuration per local binding; shared by many state occurrences.

**Invariants:** Only supported requested properties are admitted; model validity and reference conventions are retained. Substitution is use-specific: material/state meaning, requested outputs, phase/root convention, domain, datum, accuracy, derivatives, initialization and effects must fit. Compatible ports or equal dimensions alone do not prove scientific equivalence.

**Area:** Physics

**Implementation:** Existing ThermoPackage and EquilibriumThermoPackage interfaces with selected capability scope.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/thermodynamics/models/equilibrium.pse

**Definition:** Selected thermophysical and transport relationships for a material system and state representation.

**Owner:** Authored physical-model package

**Relationships:** Uses parameters, references, phase models and required capabilities; supplies state properties/derivatives. Consumes selected Material characterization and Population and structure representation only when explicitly supported.

**Concept:** Property model or package

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/thermodynamics/models/equilibrium.pse).


<a id="source-40b141b5836981ce958ef22c37ab2f5f"></a>

## Property parameterization

Source: [Notion](https://app.notion.com/p/40b141b5836981ce958ef22c37ab2f5f).

**Cardinality:** Zero or more declared parameter records per property model; many states may reference the same parameterization.

**Invariants:** Provenance, units, datum and validity survive binding; immutable constants are not silently fitted variables. Measured, fitted, predicted and correlated records retain their distinct method, uncertainty and applicability. Missing values remain distinct from fitted or predicted zeros. Calibration and independent prediction evidence remain distinguishable.

**Area:** Physics

**Implementation:** Existing authored data and physical context; target explicit capability and study-role exposure.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html

**Definition:** Shared constants, correlations and reference conventions used by a chosen physical model.

**Owner:** Authored package and admitted declared data

**Relationships:** Connects property-model definition to material-specific data and permitted fitting roles. Material characterization may select these existing scientific records; it does not create another authoritative parameter bank.

**Concept:** Property parameterization

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html).


<a id="source-40b141b58369811d9868e3379adcb62b"></a>

## Reaction law

Source: [Notion](https://app.notion.com/p/40b141b58369811d9868e3379adcb62b).

**Cardinality:** One or more laws for selected reactions; each declares its participating states and the rate or extent basis wherever rates or extents are used.

**Invariants:** Rate per volume, catalyst mass or area is not interchangeable; chemical heat is not double-counted. A biochemical yield, empirical conversion, polymer-moment law or population transformation declares its representation, basis, scope and evidence. Approximate agreement does not establish a fully specified chemical mechanism.

**Area:** Physics

**Implementation:** Existing authored reactions; broader heterogeneous and electrolyte laws require specific qualification.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/reactors.pse

**Definition:** Kinetic, conversion, equilibrium or other admitted closure that determines reaction behavior.

**Owner:** Reaction submodel or unit definition

**Relationships:** Uses property states and parameters; contributes generation/consumption and energy under declared conventions.

**Concept:** Reaction law

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/reactors.pse).


<a id="source-40b141b583698139a525d5293c614e84"></a>

## Reaction system

Source: [Notion](https://app.notion.com/p/40b141b583698139a525d5293c614e84).

**Cardinality:** Zero or more reactions or equilibrium constraints; one or more participating states/regions.

**Invariants:** Stoichiometry, basis and conservation expectations are explicit; Gibbs equilibrium need not enumerate a mechanism. Exact chemistry requires the declared participant composition and charge evidence. Empirical yield or biochemical aggregate transformations use explicitly qualified scientific contracts; they do not weaken the exact reaction/mapping contract.

**Area:** Physics

**Implementation:** Existing authored reactor models; target clear mechanisms-versus-equilibrium contract.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/gibbs_reactor.html

**Definition:** Declared reactive constituents, stoichiometry or conserved elemental structure and permitted reaction behavior.

**Owner:** Authored reaction/material definition

**Relationships:** References components, phases, conserved elements/charge and reaction laws. References selected Material characterization and measurement/balance bases when empirical aggregates participate.

**Concept:** Reaction system

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/gibbs_reactor.html).
