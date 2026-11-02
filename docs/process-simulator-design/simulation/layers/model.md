---
title: "Model: what is true"
notion_source: "https://app.notion.com/p/40a141b58369813d86b0e4faa46deb37"
notion_last_edited: "2026-11-01T20:37:20.283Z"
---

# Model: what is true

Source: [Notion](https://app.notion.com/p/40a141b58369813d86b0e4faa46deb37). Imported 2026-11-02.

**Version 3** · 2026-11-01

The Model contains the selected physical structure and assumptions. Component instances own their local quantities, relations, index sets, domains, ports, and child components. Connections form a separate graph and may contain recycles.

## Reusable physics and supported capabilities

Balance equations are acausal where appropriate. Their reusable definition does not depend on whether the active task fixes duty or outlet temperature. An external evaluator may have narrower causal capabilities, which it must declare rather than pretending to supply an arbitrary residual interface.

A quantity declares its physical type, coordinate dependence, and whether it is a variable, parameter, or defined expression. Differential/algebraic structure follows the selected formulation and temporal realization; task roles and discrete mode selection remain separate. Physical domains and model admissibility belong here. Equipment ratings and constitutive validity retain their sources and are not confused with arbitrary optimizer bounds.

Steady, dynamic, and periodic use require declared model capabilities. Transient storage behavior requires declared inventory/accumulation physics; memoryless units can participate algebraically in a dynamic flowsheet.

## Connections and internal physics

The connection contract specifies which relationships are generated. Equating potentials and summing flows is appropriate for some interfaces, but is not a universal rule for material mixing. Advected composition/enthalpy, orientation, reverse-flow capability, and explicit mixing or translation must have defined semantics.

Boundary relationships such as Danckwerts conditions are physical relations. Boundary values and initial data are supplied by Specification. Their consistency may require numerical initialization.

## Alternatives

Slots support alternative realizations and component configurations. The shared public contract lists compatible material scope, units/bases, observables, validity, and evaluation capabilities. It does not imply that all alternatives have identical internal quantities or predict the same answer.

Physical model alternatives, equivalent mathematical formulations, and numerical approximations are distinguished. A caller requesting a distributed field cannot silently receive a lumped quantity when a component alternative changes.

The process-domain design specializes these general concepts into unit operations, thermodynamic states, control/balance regions, and typed interfaces.

## Concepts

[Catalog](../../catalog/simulation-concepts.md)
