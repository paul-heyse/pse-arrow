---
title: "Process structure and connectivity"
notion_source: "https://app.notion.com/p/40b141b58369812c9eb8c101ff49bb4b"
notion_last_edited: "2026-11-01T23:34:03.862Z"
---

# Process structure and connectivity

Source: [Notion](https://app.notion.com/p/40b141b58369812c9eb8c101ff49bb4b). Imported 2026-11-02.

**Version 1.1** · 2026-11-01 · [Flowsheet Process Domain Model](index.md)

## Definitions, instances and boundaries

A model definition describes a reusable set of quantities, parameters, relationships, interfaces and optional children. An instance selects a definition, binds its allowed configuration and gives each resulting occurrence a stable identity. A model revision identifies an immutable authored package closure; a run never edits that revision.

At a selected process root, every contained instance has exactly one containment owner. It may own zero or more children, and a child may expose selected internal boundaries through the parent's interface. Containment is acyclic after finite specialization. Shared parameter or property definitions are references to shared meaning, not children with multiple containment parents. Reusing a definition creates separate instance quantities unless the author explicitly shares a parameter or aliases a member.

A subflowsheet is a composed model with a process-facing boundary. It is not a different execution universe. Global equation assembly, block initialization and explicit recycle calculation may each operate on the same hierarchy. DWSIM's embedded-flowsheet unit is a useful interface precedent; pse-arrow should retain its own equation and runtime ownership rather than inheriting a mandatory solve-and-copy algorithm. [DWSIM subflowsheet](https://dwsim.org/help/api/custom/flowsheet-uo.html); [current composition contract](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).

## Unit composition

Every unit instance owns zero or more state submodels, balance domains, constitutive closures, internal interfaces and subordinate units. It exposes zero or more ports and can publish additional quantities for specification, measurement or reporting. There is no universal requirement for exactly one inlet, outlet or control volume.

Examples explain the cardinalities:

- A translator owns two state representations and mapping equations, with no required material holdup.
- A heater uses a fluid balance domain and an energy exchange boundary.
- A heat exchanger can compose two fluid domains, a wall domain, transfer laws and four material ports.
- A mixer has a configurable set of inlet states and one mixed state.
- A batch vessel can have no active material flow during a particular step while retaining heat exchange and inventory.
- A multi-stream contactor can own its indexed states and balance equations directly.

IDAES's 1D exchanger uses two control-volume blocks; its multi-stream contactor explicitly bypasses those classes. The pse-arrow exchanger already composes hot and cold heater children and an explicit thermal exchange. These are source witnesses for compositional design, not evidence of complete parity. [IDAES exchanger](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/heat_exchanger_1D.html); [IDAES contactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html); [pse-arrow exchanger](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse).

## Port and connection contract

A port is a named boundary view over existing typed quantities. It does not create duplicate storage or an independent material state. Its declaration states the permitted coupling role, physical contract, incidence policy and owning model occurrence. A material port aliases one admitted material-state specification and its transported observations. Scalar ports may expose temperature, heat rate, shaft quantities, electrical quantities or signals through package-defined contracts; the generic kernel need not own a closed list.

A connection is one explicit occurrence between compatible endpoints. It retains its own identity even when parallel connections join the same units. Admission resolves actual active ports, validates semantic slot coverage and complete physical compatibility, and checks incidence limits. Semantic species identity determines correspondence; list position and display names do not. The current repository already retains these distinctions. [Connectivity contract](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md); [state and port structures](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).

Direct material connection is valid when the independent-state and transported contracts agree. Different property-package identities do not by themselves prohibit connection. Conversely, identical variable names or units do not establish agreement if basis, species membership, datum, reconstruction or transported meaning differs. An authored translator supplies the needed mapping and its physical checks. A translation that discards components or aggregates pseudocomponents must declare the information loss and conservation basis; it cannot claim a full reversible conversion. [IDAES translator](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/translator.html).

Compatibility is checked for the actual representation and transported quantities demanded at the boundary. An approximate or information-losing translator does not establish unrestricted replacement of one material system by another. It identifies the downstream uses for which its retained information and physical checks are sufficient; see [Materials, states, and physical models](materials-states-and-physical-models.md) for characterization, population and conservation mappings.

## What a stream means

The ordinary flowsheet stream is a process-facing representation of a connection and its material observations. It is not automatically a second independently specified thermodynamic object. A stream table can display total flow, phase split, composition and enthalpy derived from the endpoint contract.

A stream is also distinct from retained inventory, a material lot and a collection interval. Its observations may include population or structure quantities when the selected boundary contract supports them. Collected product amount or average quality is defined by the stated interval and aggregation operation; it is not silently substituted for an instantaneous stream state.

When a connection needs pressure loss, heat leakage, storage, transit delay, dispersion or a device law, represent that behavior as an explicit link model or unit with its own equations. A line on a diagram must not conceal physical behavior. Mixing and splitting likewise require declared conservation and closure equations; broadcasting a state or signal is not a material splitter.

## External and procedural implementations

A process boundary may be implemented by authored equations, a procedural calculation or an external model, but the operation must be explicit: contributing residual equations, evaluating a requested input/output relation, or advancing a stateful model over a physical interval. These operations are not interchangeable merely because they expose similar ports.

The boundary declares material/quantity conventions, required inputs and outputs, validity, approximation, derivatives, initialization and typed failure behavior. A time-advancing implementation additionally declares input hold/interpolation, time and coverage, state custody, event behavior, restart/restore capabilities and permitted algebraic-loop participation. Repeated residual evaluation must not accidentally advance physical state. Component solver success does not establish coupled convergence or original boundary conservation.

Current provider bindings already own mutable attempt state and effect-sensitive completion. A provider call is not automatically pure, and it does not by itself establish co-simulation, rollback or event synchronization. Use existing authored, provider, runtime and capability owners; a concrete consumer selects an adapter or additional coupling contract. This domain does not require a universal co-simulation coordinator or a second physics graph. [Current provider owner](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-kernels/src/provider.rs); [FMI interface distinctions](https://fmi-standard.org/docs/3.0.2/).

## Multiple graphs, one model

The containment graph, material-flow graph, energy/control coupling graphs, equation-incidence graph and execution dependency graph answer different questions. They share original identities and may be derived from the same checked model, but one is not substituted for another.

Material flow direction does not establish mathematical evaluation order. Pressure specifications, energy integration and recycle can create dependencies in either direction. A tear is an initialization or solution choice over selected connections, not a physical equipment attribute. The current `ModelingFlowSelection` and flow-graph admission preserve explicit selected topology separately from all model equations. [Flow projection](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-structural/src/flowsheet.rs); [composition and recycle semantics](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).

For initial IDAES-style workflows, directed material connections are sufficient where their assumptions hold. A future reversible connection must declare how transported composition and enthalpy change with flow direction and near zero flow. Permitting negative flow numerically is not sufficient. Modelica's stream semantics are evidence of this physical issue, not a requirement to adopt Modelica as the implementation language. [Modelica stream connectors](https://specification.modelica.org/maint/3.6/stream-connectors.html).

## Internal transfer and exposed boundaries

An internal transfer occurrence identifies the participating sides and one consistent orientation. Material, heat or work sent out of one side enters the other under the same physical convention. When a larger balance encloses both sides, the internal contribution cancels once. External loss, storage within an interface or interfacial reaction is represented explicitly and changes that enclosing balance.

Exposing a child port at a parent boundary is an alias or declared mapping. It preserves the original state and connection identities and cannot erase incidence limits, create a second reconstruction equation or count a transfer twice. The same principle applies when a user expands or collapses a subflowsheet in a UI.

## Stages, coordinates and geometry

Physical stages are equipment structure: changing the number of trays changes the model of the equipment. Mesh points are numerical approximation: refining a coordinate mesh should preserve the modeled physical domain and its boundary conditions. The two may share generic index machinery while retaining different declared meaning.

Geometry describes physical length, area, volume and interface extent. Discretization describes how continuous relationships are approximated. A unit may choose a geometry-dependent constitutive law, but a mesh resolution is not an equipment design parameter by default. Local domains can carry distinct phase states or solid fields and communicate through explicit interfaces. See [Materials, states, and physical models](materials-states-and-physical-models.md) for the resulting state and conservation contracts and [Repository mapping and acceptance workflows](repository-mapping-and-acceptance.md) for acceptance examples.
