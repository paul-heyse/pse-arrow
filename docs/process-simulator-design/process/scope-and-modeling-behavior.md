---
title: "Scope and modeling behavior"
notion_source: "https://app.notion.com/p/40b141b583698147a81ad9b713d87152"
notion_last_edited: "2026-11-01T23:34:02.458Z"
---

# Scope and modeling behavior

Source: [Notion](https://app.notion.com/p/40b141b583698147a81ad9b713d87152). Imported 2026-11-02.

**Version 1.1** · 2026-11-01 · [Flowsheet Process Domain Model](index.md)

> This v1 defines the process-facing domain for pse-arrow: reusable process models, their physical interfaces, and the operating questions posed about them. It is a target design informed by the repository at `926655517f6eac4ab52ae7a817a7b72168afdc25` and IDAES 2.13.0 behavior. Source inspection is evidence of an existing contract or implementation fragment; it is not a new execution or parity qualification.

## Product boundary

The organizing object is a process model: one or more equipment or process submodels connected through explicit physical or informational interfaces. A complete plant flowsheet, a reusable train, a single reactor, and a standalone property calculation can use the same composition mechanisms. Each presents the quantities and relationships needed for its intended process question.

**Contract amendment — breadth of process use.** The long-term domain spans refining and petrochemicals, polymers, minerals and metals, food and biotechnology, pharmaceuticals, water and environmental processes, and energy systems. This is a direction for scientific coverage, not a claim that every family is implemented. IDAES replication remains a concrete acceptance programme within that broader ambition. Selected industrial cases also test material characterization, population and polymer representations, physical operating sequences, external model coupling and engineering assessments before a narrow seed formulation becomes a public contract.

The model must support both network-level behavior and the local physics that make equipment predictive. A packed column can contain distinct gas and liquid states; a heat exchanger can contain fluid and wall regions; a particle model can be a reactor submodel. Flowsheet scope is therefore an engineering composition boundary, not a maximum spatial dimension. IDAES's solid oxide cell already contains two-dimensional local submodels, and its multi-stream contactor deliberately does not use the standard control-volume classes. These are concrete requirements against which the abstractions should be tested. [IDAES solid oxide cell](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/models_extra/power_generation/unit_models/soc_submodels/solid_oxide_cell.py); [multi-stream contactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html).

A general CFD environment, arbitrary geometric meshing, and a universal classification of all scientific simulations are outside the initial product scope. Specialized equipment may expose such calculations through an explicit model-provider interface. Native support can expand when a named process workflow needs it and the existing mathematical owners can admit it. Merely naming a future capability does not create executable support.

## Modeling behavior to preserve

| Behavior | Required domain meaning | Initial implementation emphasis |
| --- | --- | --- |
| Steady process calculation | Algebraic balances, property relations and operating specifications | Full process assembly and selected IDAES workflows |
| Equipment rating and sizing | The same physical quantity can be specified or solved under different cases | Clear specification ownership and structural closure |
| Process optimization | Objectives, bounds and operating constraints refer to original process quantities | Reuse admitted numerical infrastructure |
| Dynamics and control | Inventories, constitutive relations, initial conditions, input trajectories and controls | Qualified local storage and supported temporal profiles |
| Batch, semibatch and cyclic operation | Step-specific boundaries and laws, state continuity and declared transfers | Explicit domain contracts before broad event execution |
| Distributed equipment | Coordinates, domains, local states, boundary conditions and inter-region transfer | Standard 1D first; specific additional dimensions when required |
| Reactive and multiphase systems | Component and phase membership, reaction basis, equilibrium or transport assumptions | Capability-specific package admission |
| Estimation and reconciliation | Observations, shared fitted quantities, noise assumptions and multiple cases | Existing fit/study machinery with process identities |
| Modular and mixed-fidelity models | A declared external contract with substitutions qualified for the requested quantities, conditions and analysis | Authored composition; explicit approximation, derivative, state and effect limitations |

These rows are behavior profiles, not mutually exclusive subclasses. A dynamic, reactive, two-phase column can also be an optimization model. No single five-way or similar label should determine all admissibility, mathematical structure or returned results. Capabilities are checked from the chosen model, case, study and numerical route together.

## Separation of concerns

The domain has seven useful areas: Structure, Material, Physics, Connectivity, Operations, Analysis and Evidence. These are ownership and navigation areas. They are not seven independent stores or a new hierarchy that every equation must traverse.

Structure says which model instances exist and how they are composed. Material says what substances, phases and local states mean. Physics supplies balances and constitutive behavior. Connectivity couples declared boundaries. Operations supplies conditions and operating intent. Analysis supplies the question and numerical selection. Evidence records what was admitted, executed, checked and observed.

The overarching architecture remains responsible for generic authored definitions, typed mathematics, preparation, execution and results. This domain specializes those mechanisms through process contracts. It does not replace the mathematical representation or create a second solver authority. See [Unified Simulation Domain Model](../simulation/index.md).

## One authoring authority

Authored `.pse` packages, together with their admitted declared data and case documents, remain the source of executable model meaning. A process catalog, UI, graph, database projection or future API must resolve to the same declaration and instance identities. Such a consumer can help create or inspect authored content; it cannot maintain an independently executable equipment record that can drift from the package.

Material characterization, population/structure representations, lots, procedures, production campaigns and engineering assessments are semantic roles or views over those same sources and identities. Their introduction does not require a new Rust class, table or independent lifecycle for every concept. Existing scientific-data, authoring, case, operation and result owners remain authoritative; generic machinery changes only when a concrete consumer demonstrates a missing contract.

The distinction is already reflected in the repository: reusable definitions are checked and specialized by the modeling layer; prepared arithmetic and runtime layouts are derived products. The current process state, material-port and inventory structures retain semantic identity and original physical expressions. [Models and composition](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md); [process specialization](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).

## Fundamental invariants

1. Every executable physical relationship has an authored owner and a traceable specialized occurrence.
2. Every connection exposes its compatibility, incidence and conservation obligations explicitly; visual proximity creates no coupling.
3. Material state, transported flow and stored inventory retain distinct meanings.
4. Model structure, operating specifications, numerical choices and results retain separate lifecycles and identities.
5. A unit may contain zero, one or many balance domains and submodels. Standard control volumes are reusable building blocks, not mandatory ancestry.
6. Species identity, basis, reference datum and sign convention survive every boundary and transformation.
7. Numerical convergence, original physical acceptance, publication and parity evidence are separate claims.
8. Unsupported combinations refuse with an attributable reason; they do not silently become a simpler model.

## How breadth is earned

Coverage is established with examples chosen to expose distinct contracts, followed by broader replication of IDAES behavior. The initial witnesses include a recycle flowsheet, a two-sided exchanger, a reacting system, a property translator, a dynamic inventory with control, and a staged or distributed contactor. Cyclic adsorption, solid processing and the solid oxide cell test the intended extension boundary.

The broader witnesses include assay-based blending, particle processing, polymer grade transition, a batch/cyclic process, reversible transport, a deliberately bounded external coupling and a multi-product assessment. Choose source/design examples first; implement an executable probe only when it resolves a consequential uncertainty. These witnesses complement the IDAES reference population and do not automatically add all industry families to the first delivery. Their detailed distinctions are recorded in [Cross-industry domain contracts](cross-industry-contracts.md).

A new scientific variation belongs in an authored model when existing semantics suffice. A genuine generic semantic gap belongs in the existing owning kernel, with a focused synthetic example and a concrete process witness. This keeps extensions useful without turning every new equipment type into a Rust-level category. The repository documents the same division between authored scientific variants and generic semantic gaps. [Models and composition](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).

Continue with [Process structure and connectivity](structure-and-connectivity.md), [Materials, states, and physical models](materials-states-and-physical-models.md), [Cases, specifications, and studies](cases-specifications-and-studies.md) and [Repository mapping and acceptance workflows](repository-mapping-and-acceptance.md). The concept and coverage catalogs on [Flowsheet Process Domain Model](index.md) are structured indexes of this design, not additional executable authorities.
