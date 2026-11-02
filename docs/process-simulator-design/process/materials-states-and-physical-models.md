---
title: "Materials, states, and physical models"
notion_source: "https://app.notion.com/p/40b141b583698139a01ecae762a132a6"
notion_last_edited: "2026-11-01T23:34:03.011Z"
---

# Materials, states, and physical models

Source: [Notion](https://app.notion.com/p/40b141b583698139a01ecae762a132a6). Imported 2026-11-02.

**Version 1.1** · 2026-11-01 · [Flowsheet Process Domain Model](index.md)

## Material identity before numerical coordinates

A component denotes a scientifically identified material constituent in a model. It may be a molecular species, an ion, a lumped petroleum fraction, a solid constituent or another explicitly characterized component. Its identity is independent of a display label, array order or a particular property package. A material system selects components and phases for a specific modeling context; one process may contain several compatible or explicitly translated material systems.

Distinguish the material subject from the characterization evidence selected for a calculation and from the representation used by a particular model. An assay, lot-specific measurement or fitted characterization can define the properties of a petroleum fraction or biomass lump without creating a universal molecular identity. Similar labels do not make fractions from different assays or cut rules interchangeable. Exact coordinate/unit conversion, aggregation, estimated characterization and replacement of a physical model are different operations; each mapping declares its domain, preserved quantities and information loss. Refining a representation preserves only the observables its contract establishes.

Component data distinguishes declared composition knowledge from unavailable information. A pseudocomponent may have empirical molecular weight and other properties without a complete elemental formula; the selected source and applicability of that molecular weight must be explicit before mass–mole conversion uses it. Elemental and charge claims require the relevant compatible evidence. Sparse missing coefficients are zero only under an admitted completeness claim, not merely because a material record exists.

The inspected chemistry contract permits unknown species but requires complete participant composition and charge for its exact `apparent`, `lumped` and reaction mappings. Keep those guarantees. An empirical assay/yield transformation with weaker knowledge has an explicitly qualified scientific contract; it must not borrow exact elemental or charge guarantees from the word `lumped`. [Current chemistry contract](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/physical/models/chemistry.pse).

Electrolyte models need explicit relationships between true species, apparent components and their composition representations. Total mass, total moles, elemental amounts and apparent-component totals are different conserved or reported quantities. A model must identify the basis of each state, reaction and transport term. Molecular-weight conversion requires the relevant mapping and data; a string suffix such as `mass` or `mol` is not sufficient evidence of the contract. IDAES's state framework and Gibbs reactor demonstrate the need for alternative coordinates and elemental representation. [State definitions](https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/state_definition.html); [Gibbs reactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/gibbs_reactor.html).

## Population and material structure representations

Chemical constituents, thermodynamic phases and population coordinates have distinct meanings. A particle may have size, composition, shape, moisture and internal structure; a polymer may require chain-length/composition distributions or selected moments. An internal-coordinate bin is not automatically a new chemical species. Joint distributions and marginal distributions need not carry the same information.

A representation declares its coordinates, measure and normalization: for example, number-, mass- or volume-weighting, density versus amount per bin, and relevant units. Moments are selected observations with a stated closure; they do not imply recovery of the full distribution. Breakage or aggregation can conserve material mass while changing particle count. Rebinning or moment reduction states which quantities it preserves and which approximation it introduces. The scientific population model remains distinct from its numerical resolution, while an empirical grouping rule can itself change scientific meaning.

These contracts first use authored sets, domains, functions and physical relationships. They do not prescribe a universal particle/polymer hierarchy or population solver. A concrete scientific case determines whether an existing generic owner needs extension.

## Phases and membership

A phase definition identifies an admissible thermodynamic phase and the constitutive models that describe it. A material system owns its allowed component-phase membership relation; membership need not be a complete Cartesian product. A candidate phase is not necessarily present in the solved state. Phase existence, zero phase amount and the validity of a phase-specific correlation remain explicit concerns of the selected physical model.

The domain does not force every phase or region to share one temperature, pressure or composition state. Equilibrium formulations may impose selected equalities and chemical-potential relationships. Rate-based contactors may retain distinct bulk and interface states with finite transport resistance. A coupled gas-solid reactor may use separate gas and solid temperatures. Those are model assumptions expressed through relationships, not consequences of the word `mixture`.

## State contract

A material state represents local material conditions at a boundary or within a domain. Its state representation selects independent coordinates and declares dependent reconstruction and transported observations. Temperature-pressure-flow-composition is one representation; enthalpy-pressure or phase-component-flow representations may be more suitable elsewhere. State identity survives a different derived display of the same quantities.

The state contract includes:

- Material-system and semantic component/phase membership.
- Independent coordinate roles with complete physical types and valid index sets.
- Required reconstruction relationships, such as dependent composition closure.
- Transported component/element quantities and energy observations with declared tolerances.
- Supported property requirements, phase assumptions and validity limits.
- Responsibility for supplied-state consistency versus receiving-state reconstruction.

Properties requested by a unit are built or evaluated only where needed. A package definition and its shared parameters are distinct from each local state occurrence. IDAES uses shared physical-parameter blocks and local state blocks for this purpose. pse-arrow already retains independent state coordinates, reconstruction rows and transported observations in its process specialization. [IDAES property packages](https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html); [pse-arrow state structures](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).

## Inventory is separate from flowing state

An inventory is a stored extensive quantity owned by a balance domain: component amount, internal energy, total adsorbed amount, charge or another declared subject with an explicit balance. Specific loading, concentration and density are local state or constitutive quantities; they become an inventory only through the required capacity, mass, volume or other extensive basis. The relation between inventory and state may involve phase allocation, porosity or adsorption. A port describes transport across a boundary; it does not own the receiving inventory.

Holdup and dynamic analysis are orthogonal. A steady model may need vessel volume and inventory for residence-time or costing relations. A memoryless mixer may remain algebraic inside a dynamic flowsheet. Dynamic storage adds an inventory evolution law and the required initial or periodic conditions. Bulk composition, volume fraction and flow fraction cannot be interchanged without a stated physical relation.

A material lot identifies provenance or custody and may select characterization evidence; it is neither a species nor the complete physical state. Mixing/splitting must preserve the lot information demanded by the application. Documentary labels need not affect scientific identity, but selected lot-specific properties and observations do. Storage requires its actual mixing/segregation or residence-time model. Instantaneous boundary flow multiplied by an arbitrary duration does not define vessel contents, and a closed batch can evolve without an active material stream.

The current process bundle intentionally narrows stored component balances to declared single-phase allocations and rejects unsupported stored projections. Its 1D control volume explicitly rejects dynamic use until a separate time-accumulation closure is supplied. The target domain must express broader combinations while retaining those honest implementation limits. [Current composition limits](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md); [current 1D model](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/control-volume-1d.pse).

## Balances and constitutive closure

A balance domain identifies the region and conserved subject over which accounting is performed. A conservation relationship combines original inflows, outflows, generation, consumption, inter-region transfer and accumulation. An observation can check closure without adding a redundant equation; an accounting total reports a quantity without asserting conservation. These distinctions already exist in the repository's accumulator contract. [Conservation and accounting](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).

A material/state mapping identifies each extensive quantity it claims to preserve and the evidence available on both sides. Mapping followed by the target conservation projection must agree with the source projection over the declared domain; approximate or incomplete mappings state their allowed error and unsupported claims. This is a contract for the authored relationships, not a mandatory new matrix API. Mass-only agreement does not establish elemental or charge closure. Dry/wet basis, standard-volume conditions, catalyst-based rates, particle measures and quality indices retain their full quantity conventions; equal dimensions alone do not make them interchangeable.

Constitutive closure provides the additional physics: pressure drop, heat transfer, mass transfer, equation of state, reaction rate, equilibrium, geometry, phase allocation or equipment performance. It identifies its required states and parameters, applicability conditions, basis and output contracts. A single generic `equation` remains useful mathematically; the process-facing classification records physical intent and enables meaningful diagnostics.

Internal transfer has one identity and paired contributions. The same interfacial heat or material transfer cancels in a balance enclosing both sides, subject to explicitly modeled interfacial storage or reaction. A sign change alone is insufficient if the sides use different units, reference datums or component representations; compatible conversion must precede cancellation.

## Thermodynamics, reactions and reference conventions

A property package combines a material system, selected property models and parameterization with explicit capability and validity contracts. It is not required to calculate every conceivable property. Requested enthalpy, density, viscosity, derivatives or phase-equilibrium behavior must be supported for the chosen phases and state region.

Substitution is relative to the requested use: required outputs, material representation, phase/root convention, validity, datum, accuracy, derivatives, initialization and effects must be compatible. A detailed unit and surrogate may share a process boundary while differing in internal observations or transient memory. Replacing an EOS changes the selected scientific model even if the interface is unchanged; replacing an implementation of the same model instead needs the declared equivalence for the demanded outputs. Value, derivative and stronger certification capabilities remain separate.

A reaction system declares species participation and conservation expectations. A reaction law chooses kinetics, specified conversion, equilibrium conditions or another admitted formulation. Gibbs-equilibrium models may instead use elemental constraints and thermodynamic potentials without an enumerated mechanism. Heterogeneous reactions may span several regions; their rate basis must say whether it is per catalyst mass, phase volume, total volume or interface area. [IDAES contactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html); [Gibbs reactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/gibbs_reactor.html).

Empirical conversion or yield models declare source accounting and only the conservation evidence their characterization supports. They do not weaken the exact checked-reaction contract or introduce arbitrary component-source callbacks in its place. Any empirical transformation, thermal convention and unsupported elemental/charge claim remains visible in the authored package and resulting assessment.

Enthalpy and internal-energy conventions must preserve reference datums and the treatment of chemical formation energy. Heat of reaction cannot be added twice through both species enthalpies and a separate source term. A property transition must preserve the selected energy meaning or explicitly transform it. The same care applies to absolute versus difference temperature, pressure conventions, concentration bases and electrical quantities. [Physical semantics](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md).

## Checks that remain independent

Dimensional/type admission, material-system compatibility, equation closure, original physical conservation, property-domain validity and numerical convergence answer different questions. A converged numerical point can violate a physical assumption or an independent transport agreement. Every reported acceptance should identify the particular obligations checked, their tolerances and any unavailable evidence. These checks reuse original authored quantities and relationships; they must not be inferred solely from scaled solver residuals.

See [Cases, specifications, and studies](cases-specifications-and-studies.md) for how these quantities are specified or observed and [Repository mapping and acceptance workflows](repository-mapping-and-acceptance.md) for the implementation boundary and worked exchanger example.
