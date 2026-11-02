---
title: "Simulation concepts — Vocabulary"
---

# Simulation concepts — Vocabulary

<a id="source-40a141b5836981fba346ed20911673c6"></a>

## Chemical system

Source: [Notion](https://app.notion.com/p/40a141b5836981fba346ed20911673c6).

**Refers to:** Nothing

**Rules:** Each selected model states which material and phase subsets it supports. Reaction participants and mappings resolve by semantic identity. Incomplete composition is retained and only operations requiring the missing information are refused or left unassessed.

**Multiplicity:** 1..n

**Owns:** [Species](vocabulary.md#source-40a141b583698109b919eb17c8572fae); [Phase kind](vocabulary.md#source-40a141b5836981538897db34cac0f8b2); [Reaction](vocabulary.md#source-40a141b58369814ba984f568cbfcf375)

**Definition:** A named material scope declaring participating material entities, allowed phase identities and permitted reaction knowledge, with any true/apparent, mass/molar or pseudo-component mappings made explicit. Its members can index the Model.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Chemical system

<a id="source-40a141b5836981d2b474d62a33996eb3"></a>

## Component template

Source: [Notion](https://app.notion.com/p/40a141b5836981d2b474d62a33996eb3).

**Refers to:** Port types, constitutive models, other templates

**Rules:** Specialization is deterministic for the complete selected configuration and dependency closure. The template states its interface, assumptions and supported configurations; no closed equipment taxonomy or universal acausality is required.

**Multiplicity:** 0..n

**Owns:** [Configuration parameter](vocabulary.md#source-40a141b5836981cdaf26d4b721b2efa2)

**Definition:** A reusable authored definition or interface declaring configuration inputs, index domains, physical quantities, relations, ports, typed choices and child definitions. Specialization produces instances while retaining source and occurrence identities.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Component template

<a id="source-40a141b5836981cdaf26d4b721b2efa2"></a>

## Configuration parameter

Source: [Notion](https://app.notion.com/p/40a141b5836981cdaf26d4b721b2efa2).

**Refers to:** Chemical system

**Rules:** A structural choice is fixed for an admitted specialization. Optimizing it requires an explicit supported alternative or parameterized structural family, with each candidate's meaning and validity retained; numerical mesh settings remain a separate realization choice.

**Multiplicity:** 0..n

**Definition:** An input selected for specialization that determines structure or formulation, such as material scope, number of stages, allowed phases or enabled physical balances. It is distinct from an operating value varied within a fixed prepared structure.

**Layer:** Vocabulary

**Owned by:** [Component template](vocabulary.md#source-40a141b5836981d2b474d62a33996eb3)

**Concept:** Configuration parameter

<a id="source-40a141b58369816f8951f4741582ce6d"></a>

## Constitutive model

Source: [Notion](https://app.notion.com/p/40a141b58369816f8951f4741582ce6d).

**Refers to:** Chemical system, quantity kinds

**Rules:** Physical conventions and required parameters are explicit. Selecting a different scientific model or parameter dataset is a semantic choice. Defaults and versioned parameter data may accompany a model, but their selected values and provenance remain explicit.

**Multiplicity:** 0..n

**Definition:** A reusable versioned scientific model with relations, parameter contracts, assumptions, material/phase scope and validity evidence. Process examples include equations of state, activity and transport models, rate laws, transfer correlations and costing models; unrelated physics profiles are deferred.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Constitutive model

<a id="source-40a141b5836981b09dc4eef803ffc343"></a>

## Element

Source: [Notion](https://app.notion.com/p/40a141b5836981b09dc4eef803ffc343).

**Refers to:** Nothing

**Rules:** Unique symbol.

**Multiplicity:** 0..n

**Definition:** A chemical element or isotope, used in element balances and formulas.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Element

<a id="source-40a141b5836981538897db34cac0f8b2"></a>

## Phase kind

Source: [Notion](https://app.notion.com/p/40a141b5836981538897db34cac0f8b2).

**Refers to:** Nothing

**Rules:** Phase category, material membership, phase instance and active/present status remain distinct. Compatibility does not follow from label spelling, and multiple phases may share one kind.

**Multiplicity:** 1..n

**Definition:** A physical phase category, such as vapor, liquid, solid or adsorbed. Distinct phase identities and occurrences, such as L1 and L2 or separate solid phases, are represented by the process domain and are not different phase kinds merely because their labels differ.

**Layer:** Vocabulary

**Owned by:** [Chemical system](vocabulary.md#source-40a141b5836981fba346ed20911673c6)

**Concept:** Phase kind

<a id="source-40a141b5836981ae98e4cb7916eaa213"></a>

## Port type

Source: [Notion](https://app.notion.com/p/40a141b5836981ae98e4cb7916eaa213).

**Refers to:** Quantity kinds, chemical system

**Rules:** The connector declares equality, signed conservation, advection and causality where applicable. A material-stream temperature, composition or enthalpy is not generically an equalized potential; mixing and translation require explicit physical models.

**Multiplicity:** 0..n

**Definition:** A typed boundary contract describing members, physical conventions, semantic indices and their connection roles: independent state coordinates, transported or advected quantities, potential/through quantities, signals or explicit boundary fields. Material, thermal, work and signal interfaces have different laws.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Port type

<a id="source-40a141b5836981e799b9dcca2095365e"></a>

## Quantity kind

Source: [Notion](https://app.notion.com/p/40a141b5836981e799b9dcca2095365e).

**Refers to:** Unit

**Rules:** Equal dimensions do not imply interchangeable meanings. Complete physical compatibility is checked before composition; changing chemical basis or thermodynamic reference is an explicit scientific conversion, not merely unit formatting.

**Multiplicity:** 1..n

**Definition:** A named physical meaning with dimension and value shape, used in a complete physical type that also records basis, datum/reference convention, scale kind and subject as applicable. Examples include temperature, component molar flow, mass fraction and molar enthalpy.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Quantity kind

<a id="source-40a141b58369814ba984f568cbfcf375"></a>

## Reaction

Source: [Notion](https://app.notion.com/p/40a141b58369814ba984f568cbfcf375).

**Refers to:** Species, phase kinds

**Rules:** Element, charge and site balance are checked where the required composition is known. Missing composition produces an explicit unavailable check, not an invented proof. Approximate yield/conversion models declare their different contract; reaction energy conventions must avoid double counting formation or reaction enthalpy.

**Multiplicity:** 0..n

**Definition:** A stoichiometric relation over identified material or site entities with explicit location, basis and reversibility. Kinetics, equilibrium laws and heat conventions are separate selected physical models.

**Layer:** Vocabulary

**Owned by:** [Chemical system](vocabulary.md#source-40a141b5836981fba346ed20911673c6)

**Concept:** Reaction

<a id="source-40a141b583698109b919eb17c8572fae"></a>

## Species

Source: [Notion](https://app.notion.com/p/40a141b583698109b919eb17c8572fae).

**Refers to:** Elements

**Rules:** Known descriptors must be internally consistent; absent or incomplete composition is explicit. Do not invent atom counts or silently equate species across packages. A balance or transformation requiring elemental, charge or site composition must have the necessary information or state that the check is unavailable.

**Multiplicity:** 0..n in a generic vocabulary; a process material contract states its required nonempty scope

**Definition:** A named material entity with stable identity and whichever descriptors are known: formula, structure, charge, isotope, site identity or characterization data. It may be a molecule, ion, radical, adsorbate, surface site or characterized pseudo-component.

**Layer:** Vocabulary

**Owned by:** [Chemical system](vocabulary.md#source-40a141b5836981fba346ed20911673c6)

**Concept:** Species

<a id="source-40a141b58369811e9f28cf36ab5d54b0"></a>

## Unit

Source: [Notion](https://app.notion.com/p/40a141b58369811e9f28cf36ab5d54b0).

**Refers to:** Nothing

**Rules:** Conversions state their exactness and affine/ratio convention. A unit conversion does not silently change mass/mole basis, composition meaning or thermodynamic reference state.

**Multiplicity:** 0..n

**Definition:** A unit of measure with dimension and a declared conversion to a coherent representation, including affine temperature units where applicable.

**Layer:** Vocabulary

**Owned by:** [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451)

**Concept:** Unit

<a id="source-40a141b583698143b08bf5061e91c451"></a>

## Vocabulary

Source: [Notion](https://app.notion.com/p/40a141b583698143b08bf5061e91c451).

**Refers to:** Nothing

**Rules:** Dependencies and versions are explicit and acyclic where required by admission. Referencing a shared definition does not copy its authority; instantiated occurrences belong to the Model.

**Multiplicity:** 1

**Owns:** [Unit](vocabulary.md#source-40a141b58369811e9f28cf36ab5d54b0); [Quantity kind](vocabulary.md#source-40a141b5836981e799b9dcca2095365e); [Element](vocabulary.md#source-40a141b5836981b09dc4eef803ffc343); [Chemical system](vocabulary.md#source-40a141b5836981fba346ed20911673c6); [Port type](vocabulary.md#source-40a141b5836981ae98e4cb7916eaa213); [Constitutive model](vocabulary.md#source-40a141b58369816f8951f4741582ce6d); [Component template](vocabulary.md#source-40a141b5836981d2b474d62a33996eb3)

**Definition:** The immutable package closure of type declarations, units, named scientific entities, reusable templates and constitutive definitions referenced by a task. The process domain owns the process-specific meanings; the generic execution registry does not become a scientific catalog.

**Layer:** Vocabulary

**Owned by:** [SimulationProblem](problem-root.md#source-40a141b58369813f9bc6dcfae91d96c7)

**Concept:** Vocabulary
