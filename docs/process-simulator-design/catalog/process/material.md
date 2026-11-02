---
title: "Process concepts — Material"
---

# Process concepts — Material

<a id="source-40b141b5836981eabe60d479859041bf"></a>

## Composition representation

Source: [Notion](https://app.notion.com/p/40b141b5836981eabe60d479859041bf).

**Cardinality:** One chosen basis per quantity; zero or more explicit mappings to other representations.

**Invariants:** Conversions require justified data and preserve only their declared quantities. Exact conversion, estimated characterization, aggregation and scientific-model replacement are distinct. Mapping validity, uncertainty and information loss are explicit; missing molecular composition does not license fabricated mass–mole or elemental conversion.

**Area:** Material

**Implementation:** Existing complete physical types; target explicit process-facing mapping contracts.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md

**Definition:** A declared coordinate and quantity-basis view of material composition, including mass/molar, true/apparent, empirical aggregate and measurement-specific forms.

**Owner:** Material-system/state contract

**Relationships:** Relates component amounts/fractions, elemental totals, molecular weights and translators. References selected Material characterization, aggregate measurement bases and explicit translation; population structure is separately represented.

**Concept:** Composition representation

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md).


<a id="source-40b141b5836981e28638e6770bdaf323"></a>

## Inventory

Source: [Notion](https://app.notion.com/p/40b141b5836981e28638e6770bdaf323).

**Cardinality:** Zero or more inventories per domain; zero or one active temporal evolution per conserved occurrence.

**Invariants:** Transported flow and stored amount are distinct; holdup does not imply a dynamic route. A specific loading, concentration or density needs its extensive basis to define stored amount. Physical transitions preserve declared inventories or account for transfers/resets; deactivating a mode cannot erase material. A closed batch requires no fictitious continuous stream.

**Area:** Material

**Implementation:** Existing InventoryBalance and selected stored control volumes; broader storage bases remain explicit gaps.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volumes.pse

**Definition:** A stored extensive physical quantity whose constitutive relation and evolution are explicitly modeled.

**Owner:** Balance domain or storage submodel

**Relationships:** Related to local state, geometry, phase allocation, signed boundary flux and initial conditions. Material lot records lineage; Operating procedure and Production campaign declare physical carryover. Population and structure representation may contribute stored observables with explicit meaning.

**Concept:** Inventory

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volumes.pse).


<a id="source-40b141b5836981118666ebbc13bc4097"></a>

## Material characterization

Source: [Notion](https://app.notion.com/p/40b141b5836981118666ebbc13bc4097).

**Cardinality:** Zero or more characterization records per material subject; each modeling use selects its effective records and mappings.

**Invariants:** Subject identity, measured characterization and selected model representation are distinct. Measured, fitted and predicted values retain method, basis, uncertainty, provenance and applicability. Unknown composition or molecular weight remains unknown. A characterization is not a duplicate parameter store.

**Area:** Material

**Implementation:** Adopted v1.1 design contract. Extend existing physical-context/scientific-record and authored package owners through a concrete consumer; complete assay and aggregate-material workflows are not established by this record.

**Source URL:** https://app.notion.com/p/40b141b583698139a01ecae762a132a6

**Definition:** A selected, revisioned body of evidence describing a material subject for a modeling use: assay, grade, composition, empirical fractions, measurements and justified derived properties.

**Owner:** Authored scientific records and the case/material selection that references them

**Relationships:** Material system; Species or component; Composition representation; Property parameterization; Material lot; Observation mapping.

**Concept:** Material characterization

Adopted design contract in [Flowsheet Process Domain Model](../../process/index.md). This is a semantic role over existing authored identities; it does not prescribe a one-to-one runtime class or persistence record. The structured properties contain the definition and invariants.

Owning narrative: [Materials, states, and physical models](../../process/materials-states-and-physical-models.md).

Implementation and scientific qualification remain separate from documentation adoption.


<a id="source-40b141b5836981b2aa44f6533f71da8c"></a>

## Material state

Source: [Notion](https://app.notion.com/p/40b141b5836981b2aa44f6533f71da8c).

**Cardinality:** One state representation and material context per occurrence; many time/space-indexed occurrences.

**Invariants:** State is neither a flowsheet edge nor an inventory; required local assumptions remain explicit. A material lot is not a thermodynamic state; a population or quality measurement is included only with its selected scientific representation and observation meaning.

**Area:** Material

**Implementation:** Existing StateSpecification structures; broaden packages through concrete workflow requirements.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs

**Definition:** A local material condition at a boundary or within a physical domain, with independent and derived quantities.

**Owner:** Unit, balance domain or local physical submodel

**Relationships:** Binds property package, coordinate quantities, reconstructions and transported observations; ports alias it.

**Concept:** Material state

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).


<a id="source-40b141b5836981149d8edf3a016b26ce"></a>

## Material system

Source: [Notion](https://app.notion.com/p/40b141b5836981149d8edf3a016b26ce).

**Cardinality:** One or more components; zero or more explicitly admissible phases and mappings.

**Invariants:** No universal global species vector, molecular/molar basis or two-phase state is imposed. Component, thermodynamic phase and population membership remain distinct. Cross-system mapping is explicit and qualified.

**Area:** Material

**Implementation:** Target consolidated process-facing concept over authored sets and property interfaces.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html

**Definition:** A selected set of chemical/model components, thermodynamic phases, memberships, composition/structure representations and justified balance bases for a modeling context.

**Owner:** Authored package or local physical-model configuration

**Relationships:** Used by property packages, states, reaction systems and translators. Selects Material characterization and, where required, Population and structure representation.

**Concept:** Material system

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html).


<a id="source-40b141b5836981aebda5faefb9c62f0a"></a>

## Phase

Source: [Notion](https://app.notion.com/p/40b141b5836981aebda5faefb9c62f0a).

**Cardinality:** Zero or more phases per material system; many local state occurrences.

**Invariants:** Candidate phase membership does not assert positive phase amount; interphase equality requires a law. An operational step is not a thermodynamic phase. A particulate or aggregate model component does not, by itself, require a separately represented thermodynamic solid phase.

**Area:** Material

**Implementation:** Existing selected liquid/vapor equilibrium interfaces; broader phase combinations remain capability-specific.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/thermodynamics/models/equilibrium.pse

**Definition:** An admissible thermodynamic phase with selected constitutive behavior and local state meaning.

**Owner:** Material system or property model definition

**Relationships:** References component membership, phase property models and optional equilibrium/transfer relations.

**Concept:** Phase

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/thermodynamics/models/equilibrium.pse).


<a id="source-40b141b5836981f8bae1fbfa088ad815"></a>

## Phase-component membership

Source: [Notion](https://app.notion.com/p/40b141b5836981f8bae1fbfa088ad815).

**Cardinality:** Many-to-many, potentially sparse; every admitted pair has semantic identity.

**Invariants:** No implicit Cartesian product; absent membership is distinct from a present component with zero amount.

**Area:** Material

**Implementation:** Target first-class process projection over existing declared membership mechanisms.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html

**Definition:** The explicit relation specifying which components are admitted in which phases.

**Owner:** Material-system definition

**Relationships:** Indexes phase state, property, reaction and transfer quantities.

**Concept:** Phase-component membership

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html).


<a id="source-40b141b5836981829c7dc37c710d69b9"></a>

## Population and structure representation

Source: [Notion](https://app.notion.com/p/40b141b5836981829c7dc37c710d69b9).

**Cardinality:** Zero or more representations per relevant material population; each declares its coordinates, weighting, normalization and closure.

**Invariants:** Material internal coordinates are distinct from species identity, thermodynamic phase and physical spatial coordinates. Number/mass/volume weighting, density versus bin amount, and moment closure are explicit. Rebinning or reduction states conservation and information-loss limits; a product-quality attribute needs defined scientific meaning.

**Area:** Material

**Implementation:** Adopted v1.1 design contract. Start with authored axes, distributions, mappings and real consumers; no universal population framework or current polymer/particle solver support is implied.

**Source URL:** https://app.notion.com/p/40b141b583698139a01ecae762a132a6

**Definition:** A selected mathematical representation of material structure, such as particle size/shape distributions, population moments, or polymer chain/segment attributes used by properties, kinetics, transport or product quality.

**Owner:** Authored material/state/constitutive definitions; case-selected representation

**Relationships:** Material system; Material state; State representation; Index domain; Inventory; Composition representation; Constitutive closure.

**Concept:** Population and structure representation

Adopted design contract in [Flowsheet Process Domain Model](../../process/index.md). This is a semantic role over existing authored identities; it does not prescribe a one-to-one runtime class or persistence record. The structured properties contain the definition and invariants.

Owning narrative: [Materials, states, and physical models](../../process/materials-states-and-physical-models.md).

Implementation and scientific qualification remain separate from documentation adoption.


<a id="source-40b141b5836981ab84fef57ecc6831b4"></a>

## Species or component

Source: [Notion](https://app.notion.com/p/40b141b5836981ab84fef57ecc6831b4).

**Cardinality:** Referenced by zero or more material systems, phases and reactions.

**Invariants:** Names and vector positions are not identity. Subject identity, characterization and representation are distinct. COD/nitrogen/alkalinity and other aggregate quantities retain their declared bases; they are not automatically molecular species or generic mass concentrations. Unknown composition or molecular weight remains unknown.

**Area:** Material

**Implementation:** Existing authored chemistry/material vocabulary; target complete capability-specific characterization contract.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md

**Definition:** A scientifically identified constituent of a selected material representation: a chemical species, ion, characterized pseudocomponent or explicitly defined model aggregate. Chemical identity and model-component identity need not coincide.

**Owner:** Authored material definition

**Relationships:** Carries identity and available molecular/elemental/charge characterization; participates in composition mappings. Material characterization and Composition representation define the selected scientific use; Population and structure representation is a separate meaning.

**Concept:** Species or component

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md).


<a id="source-40b141b58369812fa7ffd98a17cfc333"></a>

## State representation

Source: [Notion](https://app.notion.com/p/40b141b58369812fa7ffd98a17cfc333).

**Cardinality:** One representation per admitted state occurrence; zero or more compatible projections.

**Invariants:** Receiving reconstruction is emitted once; supplied states retain independent consistency checks. The selected contract declares component/phase coverage, quantity basis/datum, phase/root conventions and demanded observations. Receiving-state compatibility does not prove scientific substitutability. Generic state meaning does not require the seed molecular/molar liquid–vapor coordinates.

**Area:** Material

**Implementation:** Existing state specialization and material-connection semantics.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs

**Definition:** The contract selecting independent coordinates, dependent reconstruction and transported observations.

**Owner:** Authored state declaration

**Relationships:** References semantic slots, component/phase indices, physical types and closure obligations. May use Population and structure representation and empirical component bases when the consuming model explicitly supports them.

**Concept:** State representation

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Materials, states, and physical models](../../process/materials-states-and-physical-models.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).
