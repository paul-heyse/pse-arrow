---
title: "Vocabulary: what things are"
notion_source: "https://app.notion.com/p/40a141b5836981378fa3ffe1a736b7b4"
notion_last_edited: "2026-11-01T20:36:16.900Z"
---

# Vocabulary: what things are

Source: [Notion](https://app.notion.com/p/40a141b5836981378fa3ffe1a736b7b4). Imported 2026-11-02.

**Version 3** · 2026-11-01

Vocabulary holds reusable definitions and the versions a problem references. Model holds their instances. The definitions can be stored once and referenced by immutable identity rather than cloned into every live problem; the submitted snapshot records exactly which definitions were used.

## Responsibilities

| Definition | Contract to make explicit |
| --- | --- |
| Unit and quantity kind | Dimension, conversion, physical kind, and applicable basis distinctions |
| Chemical system and Species | Material scope, component/species identity, phase membership, charges or formulae where meaningful |
| Reaction | Stoichiometric or declared lumped transformation and its basis/location; kinetic behavior is supplied separately |
| Port type | Public members, material/state conventions, directionality where applicable, and connection semantics |
| Constitutive model | Supported scope, required parameters, validity, observables, and evaluation capabilities |
| Component template | Reusable quantities, relations, interfaces, internal structure, and configuration semantics |

Chemical identity can be shared across representations where a defensible mapping exists. A petroleum lump, apparent electrolyte component, adsorbate, and fully specified molecule are not automatically interchangeable merely because they share a name. Cross-scale identity and state mapping are explicit evidence-backed relationships.

A reusable definition may reference other definitions. It must not depend secretly on one caller's current operating case or optimization objective. Parameter values and source records may be supplied through case bindings, while fixed versioned parameterizations may be referenced as named data sets.

Physical models such as equations of state and transport correlations remain central. Quantum levels of theory and turbulence closures remain representable extensions, not requirements for implementing the current flowsheet frontend.

## Concepts

[Catalog](../../catalog/simulation-concepts.md)
