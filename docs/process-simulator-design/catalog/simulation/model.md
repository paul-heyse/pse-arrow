---
title: "Simulation concepts — Model"
---

# Simulation concepts — Model

<a id="source-40a141b58369811aa574de81fa39182b"></a>

## Alternative

Source: [Notion](https://app.notion.com/p/40a141b58369811aa574de81fa39182b).

**Refers to:** A realization, or a template plus configuration

**Rules:** Siblings are compatible with the slot's declared contract, directly or through an explicit adapter. They need not have identical internal variables or physics; equivalence or approximation claims require separate evidence.

**Multiplicity:** 1..n

**Owns:** [Realization](model.md#source-40a141b5836981b2b65ef168ca05ff15)

**Definition:** One versioned candidate for a typed scientific-model, formulation or component slot, with its assumptions, applicability and external contract.

**Layer:** Model

**Owned by:** [Slot](model.md#source-40a141b58369812d9cdbdcdcd83b1c95)

**Concept:** Alternative

<a id="source-40a141b5836981158657fc14e089b107"></a>

## Boundary region

Source: [Notion](https://app.notion.com/p/40a141b5836981158657fc14e089b107).

**Refers to:** Nothing

**Rules:** Coverage, overlap, periodic pairing and compatibility of boundary conditions are checked for the selected formulation. Boundaries are not universally required to form a disjoint partition; any overlapping or uncovered conditions must have an explicit meaning.

**Multiplicity:** 0..n

**Definition:** A named selection of a domain boundary, such as an inlet, outlet, wall, symmetry plane or initial-time surface, with its orientation and role.

**Layer:** Model

**Owned by:** [Continuous domain](model.md#source-40a141b58369816782e0cdb1908fba28)

**Concept:** Boundary region

<a id="source-40a141b5836981ff892accf4daf66122"></a>

## Capability descriptor

Source: [Notion](https://app.notion.com/p/40a141b5836981ff892accf4daf66122).

**Refers to:** Nothing

**Rules:** Applicability is evaluated for the selected formulation, provider, state/branch, derivative demand and numerical route. Capability labels alone are insufficient; exact, approximate, conditional and unavailable products are distinguished and unsupported required products are refused.

**Multiplicity:** 1 (for opaque and sub-problem realizations)

**Definition:** A versioned, context-dependent contract for an implementation: supported values, derivative orders and directions, implicit or adjoint products, batch behavior, domains/branches, determinism, failures and resource estimates.

**Layer:** Model

**Owned by:** [Realization](model.md#source-40a141b5836981b2b65ef168ca05ff15)

**Concept:** Capability descriptor

<a id="source-40a141b5836981cda600cc7d0e61b5b2"></a>

## Component

Source: [Notion](https://app.notion.com/p/40a141b5836981cda600cc7d0e61b5b2).

**Refers to:** Component template, its configuration values

**Rules:** Owns physical relations, declared configuration and defaults; operating-case bindings, task roles and objectives retain their separate owners. Physical interfaces may be acausal or explicitly directed according to their contract. A path or diagram position is not instance identity.

**Multiplicity:** 1 (the root) under Model; 0..n under a parent component

**Owns:** [Discrete index set](model.md#source-40a141b5836981848941f36fdd728c5a); [Continuous domain](model.md#source-40a141b58369816782e0cdb1908fba28); [Quantity](model.md#source-40a141b5836981209c1ac20d70995188); [Mode](model.md#source-40a141b5836981029a6ef944e6f5cd46); [Constitutive model use](model.md#source-40a141b58369812fadb6eff665bc0353); [Port](model.md#source-40a141b5836981f0a3eadfe461f20d17); [Connection](model.md#source-40a141b58369813e85ffe7bb8c566a2f); [Relation](model.md#source-40a141b5836981f58c82efa7e5287583); [Slot](model.md#source-40a141b58369812d9cdbdcdcd83b1c95); [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Definition:** An owned instance of a reusable definition at a place in a composition hierarchy. In the process profile it represents a unit, composite unit, subflowsheet or other explicitly declared process role; its external interface is separate from its internal realization.

**Layer:** Model

**Owned by:** [Model](model.md#source-40a141b58369817a8e18f1c03bc9bcc7); [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Component

<a id="source-40a141b58369813e85ffe7bb8c566a2f"></a>

## Connection

Source: [Notion](https://app.notion.com/p/40a141b58369813e85ffe7bb8c566a2f).

**Refers to:** Ports

**Rules:** Connection laws follow the selected contract, not a universal equal-potential/sum-flow rule. Mixing, splitting, pressure loss, phase redistribution and unlike state/material/reference conversion are explicit unit or translator models. Distributed/lumped couplings require a declared mapping, and sign conventions and port incidence are checked.

**Multiplicity:** 0..n (in a parent component)

**Owns:** [Relation](model.md#source-40a141b5836981f58c82efa7e5287583); [Port mapping](model.md#source-40a141b5836981f3a6cfebdf144c376e)

**Definition:** An explicit coupling occurrence between compatible port contracts. A direct material connection binds independent state coordinates and retains transported-material/energy consistency; conservative junctions, signal bindings and energy exchanges each use their own declared laws.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Connection

<a id="source-40a141b58369812fadb6eff665bc0353"></a>

## Constitutive model use

Source: [Notion](https://app.notion.com/p/40a141b58369812fadb6eff665bc0353).

**Refers to:** Constitutive model, index sets, quantities

**Rules:** Admission checks material/phase coverage, basis and reference conventions, assumptions and available capabilities in the actual use context. Different scientific models are not interchangeable merely because they expose the same signature.

**Multiplicity:** 0..n

**Owns:** [Validity domain](model.md#source-40a141b58369818a9fc7c2df012ac3cc)

**Definition:** A binding of a selected scientific model and parameterization to a component's material scope, state contract, domains and quantities, producing physical relations with retained source attribution.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Constitutive model use

<a id="source-40a141b58369816782e0cdb1908fba28"></a>

## Continuous domain

Source: [Notion](https://app.notion.com/p/40a141b58369816782e0cdb1908fba28).

**Refers to:** Quantities that set its extent (length, end time), geometry

**Rules:** Physical extent and coordinate maps belong to model meaning and may depend on admitted design quantities. Numerical nodes, mesh/order and discretization choices remain separate. A spatially distributed unit remains a flowsheet component; dynamic use additionally requires an explicit temporal inventory closure.

**Multiplicity:** 0..n

**Owns:** [Boundary region](model.md#source-40a141b5836981158657fc14e089b107); [Geometry](model.md#source-40a141b583698111b23fd65cc7d5f0c3)

**Definition:** A physical coordinate domain such as time, axial position or an internal coordinate (size, age or chain length), with units, orientation and boundary semantics. General multidimensional fields are possible extension profiles, not a commitment to first-party CFD.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Continuous domain

<a id="source-40a141b5836981848941f36fdd728c5a"></a>

## Discrete index set

Source: [Notion](https://app.notion.com/p/40a141b5836981848941f36fdd728c5a).

**Refers to:** Chemical system members, other index sets

**Rules:** Finite; derived sets are recomputable from their sources.

**Multiplicity:** 0..n

**Definition:** A finite set of labels. Variants: enumerated; ordered range (stages 1 to N, for neighbour relations); drawn from the Vocabulary (species, phases, reactions of a chemical system); derived (product, subset, union, filter).

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Discrete index set

<a id="source-40a141b583698111b23fd65cc7d5f0c3"></a>

## Geometry

Source: [Notion](https://app.notion.com/p/40a141b583698111b23fd65cc7d5f0c3).

**Refers to:** Shape-parameter quantities

**Rules:** Physical geometry and its source representation are distinguished from a numerical approximation mesh. Changing mesh resolution does not silently change physical dimensions, constitutive assumptions or model fidelity.

**Multiplicity:** 0..1

**Definition:** The physical shape and dimensions relevant to a model, represented by declared primitives or versioned geometry data and parameterized by quantities.

**Layer:** Model

**Owned by:** [Continuous domain](model.md#source-40a141b58369816782e0cdb1908fba28)

**Concept:** Geometry

<a id="source-40a141b5836981029a6ef944e6f5cd46"></a>

## Mode

Source: [Notion](https://app.notion.com/p/40a141b5836981029a6ef944e6f5cd46).

**Refers to:** Relations

**Rules:** Each active configuration has an unambiguous composed relation set. Exclusivity, transition guards, reset effects and same-layout requirements are declared by the selected profile; shared relations need not belong to only one mode.

**Multiplicity:** 0..n

**Definition:** An explicitly selected group of additional or replacement relations, such as a valve operating mode. Shared base relations may remain active across multiple modes; process phase presence retains its own scientific meaning.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Mode

<a id="source-40a141b58369817a8e18f1c03bc9bcc7"></a>

## Model

Source: [Notion](https://app.notion.com/p/40a141b58369817a8e18f1c03bc9bcc7).

**Refers to:** Nothing

**Rules:** Owns one root composition view; nested components, connectivity and equation dependencies remain distinct relationships. Prepared numerical forms and solved values never overwrite authored model authority.

**Multiplicity:** 1

**Owns:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2); [Assessment requirement](cross-cutting.md#source-40b141b58369819db846e3d9a5250500)

**Definition:** An immutable authored domain-model revision, presented through a root component and its hierarchy, typed couplings and physical relations. A flowsheet is the process-domain view of these same declarations and instance identities.

**Layer:** Model

**Owned by:** [SimulationProblem](problem-root.md#source-40a141b58369813f9bc6dcfae91d96c7)

**Concept:** Model

<a id="source-40a141b5836981f0a3eadfe461f20d17"></a>

## Port

Source: [Notion](https://app.notion.com/p/40a141b5836981f0a3eadfe461f20d17).

**Refers to:** Port type, slices, boundary region

**Rules:** Members resolve to existing physical declarations or explicitly declared derived expressions. Independent coordinates, reconstruction and transported observations remain distinct; ownership, direction/incidence limits and conventions are explicit.

**Multiplicity:** 0..n

**Definition:** A typed interface occurrence on a component boundary, aliasing an existing local state specification or declared coordinates and transported observations. A distributed port also identifies its boundary domain; a port does not own a duplicate state.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Port

<a id="source-40a141b5836981f3a6cfebdf144c376e"></a>

## Port mapping

Source: [Notion](https://app.notion.com/p/40a141b5836981f3a6cfebdf144c376e).

**Refers to:** Boundary region, expressions

**Rules:** Preserve the actual transported quantities and conventions required by the coupling. An average temperature or composition is not automatically conservative. Information loss, zero-flow cases, admissible profiles and approximation/error evidence are explicit; the reverse mapping is not presumed unique.

**Multiplicity:** 0..1

**Definition:** A declared projection or reconstruction between boundary representations, such as integrating component and enthalpy fluxes from a distributed boundary or imposing a chosen lumped-to-distributed profile.

**Layer:** Model

**Owned by:** [Connection](model.md#source-40a141b58369813e85ffe7bb8c566a2f)

**Concept:** Port mapping

<a id="source-40a141b5836981209c1ac20d70995188"></a>

## Quantity

Source: [Notion](https://app.notion.com/p/40a141b5836981209c1ac20d70995188).

**Refers to:** Quantity kind and complete physical type; semantic index sets and continuous domains; derivative or inventory relations

**Rules:** Differential/algebraic structure is established for a selected formulation and temporal realization, not fixed as a universal intrinsic enum. Bounds retain their purpose, owner and provenance: physical domain, model validity, operating/design limit or numerical safeguard. Basis, datum, scale, subject and index identities are checked alongside dimensions and units.

**Multiplicity:** 0..n

**Definition:** An indexed physical declaration with stable identity, complete physical type, value type and domain. Declaration nature (variable, parameter or defined expression), dependence on coordinates, and participation in inventory or derivative relations are separate from a task's Specified, Unknown, Decision or Estimand roles.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Quantity

<a id="source-40a141b5836981b2b65ef168ca05ff15"></a>

## Realization

Source: [Notion](https://app.notion.com/p/40a141b5836981b2b65ef168ca05ff15).

**Refers to:** Expression, registry entry, or problem template

**Rules:** A common signature does not establish physical equivalence. Symbolic and opaque implementations retain validity and derivative contracts. Implicit differentiation is available only under its stated regularity, branch and solver conditions; nested solves may fail or return a qualified subset of products.

**Multiplicity:** 1

**Owns:** [Capability descriptor](model.md#source-40a141b5836981ff892accf4daf66122); [Validity domain](model.md#source-40a141b58369818a9fc7c2df012ac3cc); [ProblemTemplate](problem-root.md#source-40a141b5836981688530f7b5250f2695)

**Definition:** An implementation of an explicitly selected relation or formulation: symbolic expressions, an opaque provider with a versioned typed contract, or a nested problem whose qualified result supplies the relation. Scientific model choice, mathematical reformulation and numerical discretization remain separate choices.

**Layer:** Model

**Owned by:** [Alternative](model.md#source-40a141b58369811aa574de81fa39182b)

**Concept:** Realization

<a id="source-40a141b5836981f58c82efa7e5287583"></a>

## Relation

Source: [Notion](https://app.notion.com/p/40a141b5836981f58c82efa7e5287583).

**Refers to:** Slices, modes, its slot

**Rules:** Complete physical types and index domains are checked. Active relations and scoped inner decisions are explicit. Nested extrema do not acquire a global-optimality proof from stationarity conditions, and a catalogued relation form does not imply a supported execution profile.

**Multiplicity:** 0..n

**Owns:** [Slot](model.md#source-40a141b58369812d9cdbdcdcd83b1c95)

**Definition:** An authored physical or mathematical statement over a declared index selection: equation, inequality, derivative/inventory relation, complementarity, mode-dependent law, event/reset or an explicitly supported nested/stochastic formulation. Its semantic role, such as conservation, constitutive closure, equilibrium or connection, is separate from its numerical form.

**Layer:** Model

**Owned by:** [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2); [Connection](model.md#source-40a141b58369813e85ffe7bb8c566a2f)

**Concept:** Relation

<a id="source-40a141b58369812d9cdbdcdcd83b1c95"></a>

## Slot

Source: [Notion](https://app.notion.com/p/40a141b58369812d9cdbdcdcd83b1c95).

**Refers to:** Its alternatives

**Rules:** Alternatives satisfy the declared external contract, possibly through an explicit adapter, and declare differences in assumptions or approximation. A selection is resolved for each admitted case/structure or represented by an explicitly supported structural decision. Interface compatibility does not prove scientific equivalence.

**Multiplicity:** 0..n per relation or component

**Owns:** [Alternative](model.md#source-40a141b58369811aa574de81fa39182b)

**Definition:** A typed place for an explicit alternative in an authored definition. Each slot states whether the choice changes the scientific model, selects a mathematical formulation or chooses a structural component variant; numerical execution settings have their own owner.

**Layer:** Model

**Owned by:** [Relation](model.md#source-40a141b5836981f58c82efa7e5287583); [Component](model.md#source-40a141b5836981cda600cc7d0e61b5b2)

**Concept:** Slot

<a id="source-40a141b58369818a9fc7c2df012ac3cc"></a>

## Validity domain

Source: [Notion](https://app.notion.com/p/40a141b58369818a9fc7c2df012ac3cc).

**Refers to:** Slices

**Rules:** Static conditions are checked when knowable; state-dependent conditions are assessed during evaluation and reported with their coverage. Unknown evidence is not validity. Any allowed extrapolation is explicit and does not become empirical accuracy certification.

**Multiplicity:** 0..1

**Definition:** The stated applicability conditions and supporting evidence for a scientific model or implementation: input ranges, material/phase membership, composition, parameterization, assumptions, branch and extrapolation policy.

**Layer:** Model

**Owned by:** [Realization](model.md#source-40a141b5836981b2b65ef168ca05ff15); [Constitutive model use](model.md#source-40a141b58369812fadb6eff665bc0353)

**Concept:** Validity domain
