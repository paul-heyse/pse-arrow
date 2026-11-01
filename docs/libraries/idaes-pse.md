# IDAES-PSE — Comprehensive Python Library Overview

Research date: November 1, 2026 | Stable release: 2.13.0 (September 16, 2026) | Python 3.10–3.14 | License: BSD

IDAES-PSE (Institute for the Design of Advanced Energy Systems — Process Systems Engineering) is an extensible, equation-oriented process modeling and optimization framework built on Pyomo.

Unlike conventional process simulators that primarily execute predefined equipment and property calculations, IDAES exposes the mathematical structure of process systems as interconnected algebraic and differential-algebraic models. Engineers can simultaneously represent physical behavior, equipment design, process connectivity, operational decisions, economic objectives, and optimization constraints.

Its most important architectural characteristic is the separation of process topology, unit-operation physics, thermodynamic properties, reaction chemistry, and numerical solution. These elements can be independently developed, configured, and combined while participating in one mathematical optimization model.

IDAES supports steady-state simulation, dynamic simulation, parameter estimation, data reconciliation, process optimization, dynamic optimization, and control-oriented modeling. Its model libraries extend into power generation, gas–solid systems, carbon capture, electrochemical systems, and process economics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



Official resources

- Documentation: `https://idaes-pse.readthedocs.io/en/stable/`
- Repository: `https://github.com/IDAES/idaes-pse`
- Package: `https://pypi.org/project/idaes-pse/`
- Examples: `https://idaes-examples.readthedocs.io/`
- Releases: `https://github.com/IDAES/idaes-pse/releases`

The overview targets IDAES-PSE 2.13.0, the latest verified stable release. A `2.14.0.dev0` prerelease exists but is not treated as stable functionality.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 1. Capability landscape

| Capability family                  | Principal functionality                                                                                                              |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Equation-oriented process modeling | Hierarchical flowsheets, interconnected unit operations, explicit conservation equations, configurable boundary conditions           |
| Thermophysical modeling            | Modular property packages, equations of state, phase equilibrium, component properties, transport models, custom state formulations  |
| Reaction engineering               | Reaction kinetics, equilibrium reactions, heterogeneous systems, configurable stoichiometry and reaction models                      |
| Equipment and process models       | Reactors, separators, heat exchangers, pressure-changing equipment, columns, contactors, membranes, power-cycle components           |
| Dynamic systems                    | Material and energy accumulation, time-dependent differential equations, spatially distributed models, discretization                |
| Optimization                       | Process design, operating-point optimization, constrained nonlinear optimization, economic objectives                                |
| Numerical infrastructure           | Initialization strategies, variable and constraint scaling, structural diagnostics, Jacobian analysis, continuation methods          |
| Surrogate modeling                 | Data-driven approximations, sampling, surrogate training, neural-network and polynomial surrogates, embedding in optimization models |
| Parameter estimation               | Estimation from experimental or operational data, data reconciliation, uncertainty propagation                                       |
| Process economics                  | Equipment capital costs, operating costs, costing packages, process-level economic calculations                                      |
| Advanced process systems           | Dynamic control, moving-horizon estimation, multi-period optimization, energy-market integration                                     |
| Integration and extensibility      | Custom physical and reaction models, Pyomo optimization infrastructure, external property engines, visualization and reporting       |

A crucial distinction is that IDAES is not simply an equipment-model library. Its deeper value comes from the contracts and composition mechanisms that allow physical models, conservation laws, numerical tools, and optimization machinery to work together.

This creates architectural opportunities that would otherwise require substantial custom implementations: reusable balance-equation generation, thermodynamic interfaces, initialization orchestration, process connectivity, numerical conditioning analysis, and optimization-ready model construction.

## 2. Core modeling architecture

IDAES extends Pyomo's block-based algebraic modeling system with process-engineering abstractions and standardized interfaces.

A simplified hierarchy is:

FlowsheetBlock

Process hierarchy · time domain · unit connectivity

UnitModelBlock

Equipment behavior and performance

Parameter Blocks

Thermophysical and reaction definitions

ControlVolume

Material, energy, and momentum balances

StateBlock / ReactionBlock

Local property and reaction calculations

Pyomo algebraic model

Variables · constraints · expressions · objectives · derivative and solver infrastructure

Conceptual composition, not a strict class-inheritance diagram. State and reaction blocks are normally constructed within units or control volumes using their associated parameter packages.

### 2.1 ProcessBlock and extensible model construction

The foundational abstraction is `ProcessBlock`, built around `ProcessBlockData` and the `declare_process_block_class` decorator.

This provides a standardized method of constructing Pyomo-compatible models, including:

- `build()` lifecycle methods for assembling variables, constraints, expressions, and sub-blocks.
- Declarative configuration through Pyomo `ConfigBlock` and `ConfigValue`.
- Indexed blocks with per-instance configuration and index-to-configuration mappings.
- Inheritance and extension of existing unit models and modeling components.

The benefit is not merely object-oriented code reuse. The framework standardizes how physical submodels are instantiated and incorporated into Pyomo's symbolic model, making them available to the same optimization and diagnostic infrastructure.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/process_block.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 2.2 FlowsheetBlock and process hierarchy

`FlowsheetBlock` is the primary container for process topology and modeling context.

Its important responsibilities include:

- Establishing a time domain for steady-state or dynamic calculations.
- Supporting nested flowsheets with inherited time domains.
- Providing default property packages to descendant unit models.
- Organizing interconnected equipment into reusable process sections.
- Supporting both standalone process models and embedded subsystems.

A particularly useful feature is hierarchical property-package inheritance. Unit models can obtain the first applicable default property package from an ancestor flowsheet without every instance having to specify one individually.

Dynamic flowsheets use a Pyomo `ContinuousSet` as the time domain; steady-state flowsheets use an ordered `Set`. A steady-state subflowsheet can reside within a dynamic flowsheet, but a dynamic subflowsheet cannot be nested within a steady-state parent under the documented configuration rules.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/flowsheet_block.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 2.3 Control volumes and automatic conservation equations

The `ControlVolume` architecture is among IDAES's most consequential capabilities.

Rather than having each equipment model independently implement conservation equations, control-volume classes provide reusable construction methods for:

- Total and component material balances.
- Phase-specific material balances.
- Elemental balances.
- Total and phase energy balances.
- Enthalpy balances.
- Momentum and pressure balances.
- Material and energy accumulation.
- Mass-transfer, heat-transfer, and reaction source terms.

This makes it possible to develop specialized equipment by combining generic conservation equations with equipment-specific transport and performance relations.

Two main spatial abstractions are supported:

|                         | `ControlVolume0DBlock`                          | `ControlVolume1DBlock`                            |
| ----------------------- | ----------------------------------------------- | ------------------------------------------------- |
| Physical representation | Lumped, well-mixed volume                       | Variation along one flow-direction coordinate     |
| Spatial discretization  | Not required                                    | Required                                          |
| Typical uses            | Tanks, mixers, stirred reactors, simple heaters | Plug-flow reactors, pipes, distributed exchangers |
| States                  | Usually inlet and outlet state blocks           | State blocks distributed along the spatial domain |
| Dynamics                | Optional holdup and accumulation                | Optional time- and space-dependent balances       |
| Numerical complexity    | Generally smaller                               | Depends on spatial and temporal resolution        |

The spatial and temporal dimensions can be combined to form differential-algebraic models. The discretization process uses the Pyomo modeling infrastructure rather than a separate IDAES numerical PDE engine.

A meaningful boundary is that the generic 1D control volume is not an arbitrary multidimensional computational-fluid-dynamics framework. More complex geometries and transport phenomena require specialized models.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/control_volume_0d.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/control_volume_1d.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



### 2.4 Process connectivity and property translation

IDAES uses Pyomo `Port` and `Arc` mechanisms to connect unit-model stream interfaces. Connections become explicit mathematical equality constraints through the Pyomo network transformation.

Not every connection must use the same internal property model. Two especially useful unit abstractions are:

- `Translator`: bridges streams represented by different property packages or state formulations, with application-specific mapping relationships.
- `StateJunction`: provides a connection between state representations without introducing equipment behavior.

This separation matters for heterogeneous processes. The interface between two property packages can be modeled explicitly rather than requiring a single thermodynamic package to describe every subsystem.

However, a translator does not automatically establish a physically valid mapping between arbitrary chemical systems. Species correspondence, conservation relationships, basis conversions, and missing information must be addressed in the translation model.

Relevant unit-model reference:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 3. Thermophysical property modeling

IDAES's property architecture is designed for equation-oriented simulation and optimization, rather than exclusively for numerical property lookup.

This distinction affects both its strengths and its costs: thermophysical relationships may become explicit symbolic constraints and variables inside the optimization model, making them available for simultaneous optimization, but potentially increasing model size and nonlinearity.

### 3.1 PhysicalParameterBlock and StateBlock

The principal abstractions are `PhysicalParameterBlock` and `StateBlock`.

A physical parameter block describes the material system, its supported phases and components, property calculation methods, parameters, and associated metadata.

State blocks represent the local thermodynamic state and calculate properties at particular time or spatial locations. They may expose such quantities as temperature, pressure, composition, enthalpy, entropy, density, fugacity, and transport properties.

Property packages supply standardized methods for retrieving quantities required by balance equations, such as material flow, enthalpy flow, density, and material-flow basis.

Importantly, these abstractions distinguish the definition of a property system from the evaluation of that system at a particular state.

A single parameter package can therefore support many state-block instances throughout a process model.

### 3.2 Modular property package construction

`GenericParameterBlock` provides a configurable framework for assembling a thermophysical package from constituent methods.

Its major configuration dimensions are:

| Dimension              | Capabilities                                                                           |
| ---------------------- | -------------------------------------------------------------------------------------- |
| Components             | Chemical species, molecular properties, elemental composition, phase eligibility       |
| Phases                 | Vapor, liquid, solid, phase-specific species lists                                     |
| Equations of state     | Phase-specific thermodynamic formulations                                              |
| Pure-component methods | Correlations for heat capacity, vapor pressure, enthalpy, density and other properties |
| State definition       | Choice of independent flow, composition and thermodynamic variables                    |
| Phase equilibrium      | Phase-pair definitions and equilibrium equations                                       |
| Transport              | Viscosity, thermal conductivity and mixture relationships                              |
| Units                  | Base-unit definitions and conversion from correlation-specific units                   |
| Reference states       | Pressure and temperature references for relative thermodynamic quantities              |

Prebuilt property correlations can be combined with custom modules, allowing a property package to be developed without implementing every correlation and supporting class from scratch.

For optimization and parameter-estimation applications, many nominally fixed model parameters are represented as fixed Pyomo variables. These can subsequently be unfixed and estimated.

Limit: The upstream documentation explicitly describes the generic modular property framework as functional but still evolving. It does not cover all chemical systems or specialized thermodynamic models; custom implementations remain important for unusual or computationally demanding systems.

Source: `https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 3.3 Thermodynamic formulations

IDAES includes several important classes of property models.

Ideal and cubic equations of state. The modular framework supports ideal-gas/liquid methods and cubic equations of state, including Peng–Robinson and Soave–Redlich–Kwong.

The cubic implementation incorporates pure-component critical parameters, acentric factors, binary interaction parameters, mixture calculations, and thermodynamic residual contributions. An external cubic-root function provides the appropriate roots and derivatives needed for optimization.

Activity-coefficient models. The dedicated vapor–liquid-equilibrium property model supports ideal vapor behavior with ideal, NRTL, or Wilson liquid activity-coefficient formulations. These provide options for nonideal liquid mixtures that are inadequately represented by ideal-mixture assumptions.

Helmholtz-energy formulations. IDAES also provides specialized pure-fluid Helmholtz equation-of-state support. These formulations are especially relevant to high-accuracy pure-fluid thermodynamic calculations in energy systems.

Transport correlations. Supported modular correlations include Chapman–Enskog and Chung viscosity methods, Eucken thermal conductivity, Wilke mixture viscosity, and Wassiljewa–Mason–Saxena gas-mixture thermal conductivity.

These are distinct alternatives, not interchangeable sources of equivalent property accuracy. Their applicable species, pressure and temperature ranges, parameter requirements, and treatment of nonideality differ.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/eos/cubic.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/property_models/activity_coefficient.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/property_models/helmholtz.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/transport_properties/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+3



### 3.4 Alternative thermodynamic state formulations

The modular framework supports multiple choices of independent state variables, including `FTPx`, `FcTP`, `FPhx`, `FcPh`, and `FpcTP`.

These represent different combinations of flow, composition, temperature, pressure, and enthalpy variables.

This is consequential because state selection affects equation structure, numerical conditioning, and suitability for particular physical specifications.

For example, a temperature-based representation can be convenient for a temperature-specified unit, while an enthalpy-based representation may be more natural for some energy-balance or flashing calculations.

The availability of a state definition does not imply that every equation of state and phase-equilibrium formulation supports every combination.

Other important details:

- State-variable bounds include lower, initial/nominal, and upper values.
- Bounds can be supplied with units and converted automatically.
- Thermodynamic reference states must be configured consistently.
- Phases and eligible phase-component combinations are defined ahead of model construction.

Source: `https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/state_definition.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 3.5 Phase equilibrium and smooth formulations

Phase equilibrium is difficult in equation-oriented models because phases can appear or disappear as temperature, pressure, or composition changes.

IDAES provides several mechanisms to handle this:

- Configurable phase pairs and phase-component participation.
- Fugacity-based and other equilibrium relations.
- Bubble-point and dew-point relationships.
- `SmoothVLE` formulations using a smooth equilibrium-state representation.
- `CubicComplementarityVLE` formulations for cubic-EoS systems.

A notable technique is to introduce a separate equilibrium state so that equilibrium calculations remain well-defined even when a phase is effectively absent from the physical material state.

This can mitigate the discontinuities associated with conventional phase-switching logic and facilitate derivative-based optimization.

Nevertheless, smoothing and complementarity formulations introduce approximation parameters and additional nonlinear relationships. They do not eliminate the need to examine phase stability, critical-region behavior, and model conditioning.

Source: `https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/phase_equilibrium.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 3.6 Property metadata and extensibility contracts

Property packages expose machine-readable metadata through `PropertyClassMetadata`, `UnitSet`, and `PropertySet`.

Metadata can describe:

- Supported and required properties.
- Units and dimensionality.
- Property construction methods.
- Indexed and unindexed properties.
- Property validity ranges.
- Dependencies on other properties.

This provides an important degree of semantic interoperability. Unit models and reaction packages can interrogate a property package rather than relying entirely on implicit assumptions about its available calculations.

In architectural terms, it provides much of the foundation for building a reusable, typed interface between independently developed physical models.

However, metadata describes the provided model interfaces; it is not a universal chemical ontology, thermodynamic database, or guarantee of physical consistency between packages.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/property_metadata.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.12.0



## 4. Reaction modeling and chemical equilibrium

IDAES separates reaction chemistry from general material properties through `ReactionParameterBlock` and `ReactionBlock`.

The Generic Reaction Package Framework constructs reaction models using modular definitions for stoichiometry, kinetic expressions, equilibrium conditions, reaction parameters, and reaction enthalpy.

### 4.1 Rate-based chemistry

The framework includes support for configuring rate-controlled reactions using:

- Stoichiometric coefficients indexed by phase and component.
- Arrhenius rate constants.
- Power-law rate expressions.
- Reaction orders, including user-defined non-elementary orders.
- Multiple concentration bases.
- Heat-of-reaction relationships.

For example, the supported power-law formulation has the general form:

\\[ r=k(T)\prod_j C_j^{n_j} \\]

This can be used in steady-state and dynamic reactor models by coupling local reaction rates with the corresponding control-volume generation and consumption terms.

### 4.2 Equilibrium chemistry

Available reaction equilibrium methods include:

- Van 't Hoff equilibrium-constant relationships.
- Gibbs-energy-derived equilibrium constants.
- Power-law equilibrium expressions.
- Logarithmic equilibrium expressions.
- Solubility-product formulations for precipitation.

An important advanced capability is the treatment of solid precipitation through complementarity formulations. These can represent the distinction between undersaturation with no solid present and saturation with a precipitated solid phase.

The implementation includes smoothed complementarity relationships, allowing precipitation constraints to participate in nonlinear optimization.

The log-form equilibrium methods can improve the representation of certain concentration-product calculations, although positivity requirements and numerical behavior must still be respected.

### 4.3 Reaction/property-package coupling

Each reaction package depends on a compatible thermophysical property package for its state information.

This dependency includes agreement on relevant units, material-flow basis, component/phase definitions, and the availability of properties needed by the reaction expressions.

The reaction package should not be thought of as a stand-alone chemical solver: reaction relationships are incorporated into the overall Pyomo model and solved together with the remaining process constraints.

The generic reaction framework remains extensible, and its built-in library of reaction forms is not exhaustive. Specialized mechanisms, complex heterogeneous kinetics, or unusual reaction thermodynamics may require custom reaction modules.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general_reactions/index.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general_reactions/rate_form.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general_reactions/equil_form.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+3



## 5. Equipment and process-model libraries

IDAES provides several layers of reusable models, from generic equipment templates to specialized, application-specific process systems.

An important distinction is that the presence of a unit model does not imply a single predetermined level of physical fidelity. Many models expose alternative balance formulations, optional holdup, pressure-change relationships, and configurable performance equations.

### 5.1 General-purpose equipment

| Model family        | Representative models and capabilities                                                                                     |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Reactors            | `CSTR`, `PFR`, `EquilibriumReactor`, `GibbsReactor`, `StoichiometricReactor`                                               |
| Separation          | `Flash`, `Separator`, solid–liquid separators, `Thickener0D`                                                               |
| Heat transfer       | `Heater`, `HeatExchanger`, `HeatExchangerNTU`, `HeatExchanger1D`, shell-and-tube exchangers, lumped-capacitance exchangers |
| Pressure and flow   | `PressureChanger`, `Pump`, `Compressor`, `Turbine`, `Valve`                                                                |
| Flow distribution   | `Mixer`, stream splitters and junctions                                                                                    |
| Interphase transfer | `MSContactor` with multiple interacting streams                                                                            |
| Model interfaces    | `Feed`, `Product`, `StateJunction`, `Translator`, `StreamScaler`, `SkeletonUnitModel`                                      |

Three distinctions are particularly significant.

First, reactor families embody different specifications. A Gibbs reactor determines equilibrium through Gibbs-energy minimization, an equilibrium reactor imposes reaction-equilibrium relations, and a stoichiometric reactor uses specified conversion or reaction extents. These approaches do not require identical chemical information and need not give identical results outside their physical assumptions.

Second, heat exchangers offer multiple levels of spatial resolution. An NTU or lumped 0D exchanger may provide an efficient representation of overall performance. A 1D exchanger introduces separate distributed control volumes, flow-direction relationships, heat-transfer constraints, and configurable spatial discretization. Co-current and counter-current arrangements are supported.

Third, many simple equipment models act as extensible foundations. A `SkeletonUnitModel` permits the author to construct an unconventional interface, while a general heater or pressure changer can sometimes be extended with relatively few additional equations.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



### 5.2 Multi-stream contactors

`MSContactor` deserves particular attention because it provides a more general transfer-oriented abstraction than a conventional two-inlet, two-outlet equipment template.

It supports:

- Multiple interacting material streams with independently configured property packages.
- Co-current or counter-current flow.
- A configurable series of finite elements.
- Inter-stream mass and energy transfer.
- Optional pressure changes, external heating, side feeds, and side draws.
- Rate-based, equilibrium, inherent, and heterogeneous reactions.
- Material and energy holdup for dynamic models.
- Custom transfer correlations connecting the exposed transfer variables to physical driving forces.

Applications can include membrane separations, extraction, absorption, and other contact processes.

The model directly constructs the relevant conservation equations and transfer variables, but generally does not provide all the closure equations necessary to describe a specific device. For example, membrane permeance, interfacial mass transfer, and reaction-extent relationships still need to be supplied when appropriate.

Unusually, this model does not use the standard `ControlVolume` objects internally. Instead, it constructs state blocks and reaction blocks for its finite elements, with balances written at the unit-model level.

Its heterogeneous-reaction interface is explicitly described as beta functionality, without a finalized base class in the 2.13.0 documentation.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 5.3 Specialized model libraries

Power generation

IDAES includes equipment tailored to steam and power cycles, including feedwater heaters, turbine stages, boiler heat exchangers, waterwalls, steam drums, downcomers, water pipes, and solid-oxide electrolysis-related equipment.

It also provides larger flowsheet examples for supercritical and subcritical coal-fired power generation. These models can illustrate how complex thermal subsystems are assembled and initialized.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/power_generation/unit_models/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



Gas–solid processing

The gas–solid contactor library includes bubbling fluidized beds, 0D and 1D fixed beds, and moving-bed reactors.

These models can couple separate gas and solid property descriptions with heterogeneous chemical reactions, interphase transfer, and bed-specific conservation relationships.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/gas_solid_contactors/unit_models/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



Additional separations and heat-transfer models

Other documented models include plate heat exchangers, fixed-bed temperature-swing adsorption for carbon capture, a one-dimensional membrane model for CO₂ separation, and a dedicated solid–liquid processing library.

The solid–liquid models use distinct solid and liquid property packages and stream interfaces, rather than assuming that both phases can always be represented through one conventional fluid-state formulation.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/models_extra/index.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/solid_liquid/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



These specialized libraries demonstrate a useful architectural pattern: standard process interfaces can coexist with highly specialized internal physics. However, their validation ranges, physical assumptions, maturity, and initialization requirements must be assessed separately from those of the general IDAES framework.

## 6. Dynamic simulation and control

IDAES represents dynamic process behavior through differential-algebraic equations (DAEs), combining transient conservation laws with algebraic thermodynamic, hydraulic, transport, and equipment relationships.

### 6.1 Time-dependent physical models

A typical dynamic balance has the form

\\[ \frac{dM}{dt} = F\_{\mathrm{in}}-F\_{\mathrm{out}}+G-T \\]

where \\(M\\) is accumulated material, \\(G\\) denotes generation by reactions, and \\(T\\) represents net transfer terms.

Dynamic models introduce state trajectories and their derivatives through Pyomo DAE constructs such as `ContinuousSet` and `DerivativeVar`.

This enables:

- Transient vessel inventories and energy holdup.
- Time-varying reaction and heat-transfer behavior.
- Dynamic spatially distributed systems.
- Equipment start-up, shutdown, and disturbance response.
- Dynamic operating constraints and control decisions.

Time discretization can use Pyomo's finite-difference or orthogonal-collocation transformations. After discretization, the resulting algebraic system is solved using an appropriate optimization solver.

The distinction between a physical DAE and its discretized nonlinear system is important: the chosen mesh, scheme, integration horizon, and initial conditions affect accuracy, numerical stability, and optimization complexity.

### 6.2 Dynamic-model utilities

IDAES includes utilities specifically for manipulating time-indexed models.

Examples include:

- `copy_values_at_time`: transfers corresponding state values between time points, potentially across different flowsheets.
- `copy_non_time_indexed_values`: transfers values of variables that do not vary with time.
- `copy_values_from_point`: distributes an initialized state across a time domain.
- `get_derivatives_at`: locates derivative variables at selected time points.
- `find_comp_in_block_at_time`: matches corresponding components across models or discretizations.
- Utilities to fix time-independent variables, deactivate selected constraints, and inspect active/fixed states.

These are useful when transferring a converged steady-state solution into a dynamic model, constructing repeated control problems, or managing state trajectories during initialization.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/dyn_utils.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 6.3 PID control

IDAES provides a `PIDController` model for embedding feedback control relationships directly within a dynamic process model.

The implementation supports configurable proportional, integral, and derivative control behavior, including treatment of manipulated-variable bounds.

Because the controller is part of the same equation-oriented model, the controlled process and controller response can be solved together.

This is particularly useful for studying physical interactions between control laws and nonlinear equipment behavior, although it is different from providing a production industrial-control runtime.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/control/pid.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 6.4 Nonlinear model predictive control: Caprese

`idaes.apps.caprese` extends the dynamic-modeling framework toward optimization-based control.

Its `NMPCSim` abstraction accepts a plant model, controller model, their time domains, manipulated inputs, measured variables, and sampling interval.

The general concept is to repeatedly solve a finite-horizon optimization problem using a dynamic process model, apply the resulting manipulated inputs, and update the controller at the next sampling point.

IDAES provides machinery for matching plant and controller variables and managing model-based control simulations.

Maturity distinction: The official 2.13.0 documentation exposes `NMPCSim` but leaves part of the detailed optimization-problem explanation unfinished. Caprese should therefore be treated as an advanced, specialized interface whose implementation and examples warrant inspection, rather than as a fully documented turnkey control platform.

The documentation also lists moving-horizon estimation, but its MHE page explicitly states that the module is forthcoming. Consequently, a complete released MHE implementation should not be inferred from its appearance in the documentation navigation.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/caprese/nmpc.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/caprese/mhe.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



## 7. Optimization, multi-period modeling, and electricity-market integration

### 7.1 Simulation and optimization share the same model

Because IDAES models are built from Pyomo variables and constraints, a converged process simulation can provide the foundation for an optimization problem.

The distinction is primarily in how the model is specified:

- In a simulation, sufficient variables are fixed to yield a determined system.
- In an optimization, selected operating or design variables are left free, with an objective and appropriate constraints.
- In dynamic optimization, trajectories of decision variables can be optimized subject to discretized physical dynamics.

Typical objectives include energy consumption, production cost, profit, equipment size, emissions, resource utilization, and economic performance.

This supports simultaneous optimization of interconnected equipment rather than requiring a process simulator to be repeatedly called as an opaque black box.

However, optimization guarantees come from the mathematical formulation and selected solver. A local nonlinear optimizer does not automatically produce a globally optimal design, especially for nonconvex thermodynamic systems.

### 7.2 Multi-period operational models

IDAES's multi-period tooling connects individual process models across a sequence of operating periods.

The core principle is to represent the process separately at each period while introducing inter-period relationships, such as inventory conservation, operating limits, or changes in dispatch.

This is distinct from continuous-time dynamic simulation. A multi-period model may use steady-state process physics at each time point and connect those states with discrete operational constraints.

The framework can therefore represent economic operations over long horizons without necessarily carrying all the differential-algebraic detail of an equivalent fully dynamic flowsheet.

### 7.3 Price-taker optimization

`PriceTakerModel` provides a specialized abstraction for energy resources that accept externally determined electricity prices.

Its capabilities include:

- Repeated process-operation blocks across time periods.
- Capacity and minimum-operating-level constraints.
- Commitment and operating-mode relationships.
- Ramp-rate and temporal operating constraints.
- Economic objectives based on time-dependent market prices.
- Representative-day or scenario-based economic evaluation.

This is particularly relevant for energy-intensive industrial processes, flexible generation, storage, and hybrid power-and-chemical systems.

The price-taker assumption is consequential: prices are exogenous to the modeled resource. It is not equivalent to jointly clearing an electricity market.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/apps/grid_integration/multiperiod/Price_Taker.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 7.4 Double-loop market simulation

IDAES also contains an advanced grid-integration application for connecting detailed process models with wholesale electricity-market simulations.

Its principal abstractions are:

| Component     | Role                                                                        |
| ------------- | --------------------------------------------------------------------------- |
| `Forecaster`  | Supplies market forecasts and uncertainty information                       |
| `Bidder`      | Optimizes bids based on process economics and operating constraints         |
| `Tracker`     | Computes operating schedules that track dispatch instructions               |
| `Coordinator` | Exchanges information between the process-side models and market simulation |
| Prescient     | External production-cost and grid-market simulation engine                  |

The framework models day-ahead bidding and clearing alongside real-time dispatch, operational tracking, and settlement.

The bidder can formulate stochastic optimization problems for market participation. The coordinator then communicates with Prescient, which is a separate project, rather than a native solver implemented by IDAES itself.

This is a powerful example of compositional modeling: detailed process physics can influence decisions at a larger infrastructure and market scale.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/apps/grid_integration/Introduction.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/apps/grid_integration/Implementation.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/apps/grid_integration/Bidder.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



## 8. Numerical solution and initialization infrastructure

IDAES's numerical capabilities deserve as much attention as its physical-model library. For large equation-oriented models, constructing the equations is frequently easier than obtaining stable, reproducible solutions.

### 8.1 Solver independence and solver configuration

IDAES primarily delegates mathematical optimization to Pyomo's solver interfaces.

`idaes.core.solvers.get_solver()` provides a convenience interface for obtaining the configured solver, and IDAES adds solver-configuration, logging, and capability-checking utilities.

For nonlinear process models, IPOPT is a common choice. Other solver categories—including linear, mixed-integer, and mixed-integer nonlinear optimization—can be accessed through appropriate Pyomo interfaces when their capabilities match the formulation.

An important utility is `ipopt_has_linear_solver()`, which checks availability of linear-solver backends such as MA27, MA57, MA97, MUMPS, PARDISO, and SPRAL.

This matters because the sparse linear algebra inside a nonlinear solver can materially affect the behavior of large process models.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/solvers.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 8.2 PETSc integration for nonlinear equations and DAEs

One easily overlooked capability is IDAES's interface to PETSc.

IDAES exposes `petsc_snes` for nonlinear equation systems and `petsc_ts` for time integration through an AMPL-compatible interface.

The `petsc_dae_by_time_element()` utility can:

- Integrate a Pyomo DAE model between selected time points.
- Solve or reuse initial conditions.
- Transfer integrated states into the Pyomo model.
- Save, load, and interpolate trajectories using `PetscTrajectory`.
- Support implicit integration methods such as backward Euler, Crank–Nicolson, and generalized-alpha schemes.

This introduces an alternative to solving every transient calculation as one large, simultaneously discretized NLP. It also creates opportunities to generate dynamic initialization states and examine discretization accuracy.

The boundaries are important. The documented PETSc wrappers support zero-degree-of-freedom simulation problems, not general process optimization. Only implicit TS methods are supported through the interface; parallel methods, IMEX integration, and PETSc TAO optimization remain listed as future functionality.

PETSc binaries are optional and platform-dependent.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/solvers.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 8.3 Reusable initialization objects

The modern initialization framework uses `Initializer` objects with an `initialize()` method.

Important strategies include:

- `BlockTriangularizationInitializer`: decomposes a model into sequentially solvable blocks based on equation-variable incidence.
- `FromDataInitializer`: uses supplied or previously saved state information.
- `SingleControlVolumeUnitInitializer`: supports units built around a single control volume.
- Specialized initializers for modular properties, Helmholtz properties, mixers, separators, heat exchangers, pressure changers, and contactors.
- `InitializerBase` and `ModularInitializerBase` as extension foundations.

The block-triangularization strategy is particularly useful when a large nonlinear system contains exploitable structural dependencies. It can solve smaller subsystems in an order derived from the incidence graph, with strongly connected components treated as coupled blocks.

Not every model benefits from such decomposition. Strongly coupled phase-equilibrium systems, recycle loops, and highly nonlinear process sections may still require carefully selected guesses, temporary specifications, or dedicated initialization sequences.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/initialization/index.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/initialization/bt_initializer.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/initialization/specific_initializers.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



### 8.4 Adaptive homotopy and continuation

IDAES includes `idaes.core.solvers.homotopy.homotopy()`.

This meta-solver starts from an existing feasible solution and gradually moves selected fixed variables toward new target values.

Rather than solving the final condition in one large step, it:

1. Interpolates toward the target condition.
2. Attempts an IPOPT solve.
3. Accepts successful steps and adjusts the step size.
4. Reduces the step size and restores the previous state following failed steps.

The implementation uses adaptive step sizes informed by the nonlinear solver's iteration count.

This is useful for changing pressure, composition, heat duty, or other boundary specifications when direct convergence is difficult.

It is a continuation mechanism, not a guarantee of convergence or a remedy for structurally incorrect equations.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/homotopy.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 9. Scaling, structural analysis, and numerical diagnostics

IDAES provides unusually substantial infrastructure for analyzing mathematical-model correctness and numerical conditioning.

### 9.1 Scaling toolbox

The newer scaling infrastructure distinguishes two complementary approaches:

| Approach              | Mechanism                                                                            | Principal limitation                               |
| --------------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------- |
| `AutoScaler`          | Uses current variable values and Jacobian information to estimate scaling factors    | Depends on the current state and can overscale     |
| `CustomScalerBase`    | Allows model-specific scaling based on engineering knowledge and expected magnitudes | Requires domain-informed rules                     |
| `ArcConstraintScaler` | Scales constraints associated with stream connections                                | Cannot correct poor scaling inside connected units |

These mechanisms can propagate expected scales through a flowsheet, use known feed and operating magnitudes, and normalize constraint relationships.

The distinction between scaling variables and scaling equations is significant. Numerical problems arise from both the magnitudes of variable values and the derivatives in the constraint Jacobian.

The toolbox supports use of Pyomo scaling suffixes so compatible solvers can receive scaling information.

`ScalingProfiler` provides a further capability: it can compare scaling methods by examining Jacobian condition numbers and solver behavior on initialized and perturbed models.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/explanations/scaling_toolbox/scaling_toolbox.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/scaling_toolbox/scaling_workflow.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/scaling/scaling_profiler.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



### 9.2 DiagnosticsToolbox

`DiagnosticsToolbox` unifies structural and numerical model inspection.

Its documented capabilities include:

Structural diagnostics

- Degrees-of-freedom and square-system assessment.
- Structural singularity analysis.
- Dulmage–Mendelsohn partitioning to locate overconstrained or underconstrained submodels.
- Identification of constraints with no free variables.
- Unit-consistency checks.
- Detection of potentially invalid mathematical operations.

Numerical diagnostics

- Constraint residual inspection.
- Variables outside or near bounds.
- Extreme variable magnitudes and missing values.
- Large or small Jacobian coefficients.
- Near-parallel Jacobian rows or columns.
- Terms with extreme magnitude differences or numerical cancellation.

Infeasibility analysis

`compute_infeasibility_explanation()` attempts to identify conflicting constraints and bounds by relaxing constraints and investigating minimal infeasible systems.

This differs from ordinary solver failure reporting: it attempts to explain which portion of the mathematical model may be responsible for the failure.

The toolbox also provides assertion methods that can turn certain diagnostic warnings into automated checks.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/diagnostics/diagnostics_toolbox.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 9.3 SVD and irreducible degeneracy analysis

For more difficult cases, IDAES provides separate advanced tools.

`SVDToolbox` performs singular-value decomposition of the constraint Jacobian to investigate nearly dependent equations and poorly determined variables. It can use dense or sparse SciPy-based methods and accepts a customizable SVD callback.

`DegeneracyHunter` identifies irreducible degenerate sets using mixed-integer optimization, with SCIP documented as its default MILP solver.

These target different problems:

- SVD examines local numerical rank and small singular values.
- Degeneracy Hunter seeks particular sets of constraints responsible for degeneracy.
- Dulmage–Mendelsohn analysis identifies structural under- and overdetermination without requiring a numerical solution.

Used together, these methods can distinguish a model that is structurally incomplete from one that is square but locally ill-conditioned or redundant.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/diagnostics/svd_toolbox.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/diagnostics/degeneracy_hunter.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



## 10. Surrogate modeling and reduced-order representations

IDAES provides a common interface for constructing surrogate models from datasets and embedding their mathematical representations inside a flowsheet.

This is more than a collection of regression tools. The key capability is converting fitted relationships into constraints that participate in process simulation or optimization.

### 10.1 Surrogate model families

| Integration     | Model families                                         | Important characteristics                                                                  |
| --------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| PySMO           | Polynomial regression, radial basis functions, kriging | Python-based surrogate generation with configurable basis functions and fitting methods    |
| ALAMO / ALAMOPY | Algebraic surrogate models                             | Searches for parsimonious algebraic expressions; depends on an external ALAMO installation |
| OMLT / Keras    | Neural-network surrogates                              | Embeds trained neural networks into Pyomo-compatible optimization formulations             |

The interfaces distinguish training from deployment.

A trainer receives input/output datasets, variable labels, bounds, and algorithm-specific settings. The resulting surrogate exposes a mapping that can be connected to a process model through `SurrogateBlock`.

For example, a surrogate can replace an expensive equipment relationship with a fitted approximation while leaving the rest of the flowsheet equation-oriented.

### 10.2 Flowsheet integration through SurrogateBlock

`SurrogateBlock` creates an explicit relationship between Pyomo input and output variables using a surrogate model.

This enables several useful patterns:

- Replacing computationally expensive property or equipment calculations.
- Constructing optimization problems around experimentally measured response surfaces.
- Embedding neural-network approximations within mathematical programming models.
- Combining first-principles conservation laws with data-driven constitutive relationships.
- Using surrogate equations for process design and operating-point optimization.

The integration mechanism varies by surrogate family. Polynomial and radial-basis formulations differ considerably from neural-network embeddings, particularly regarding differentiability, nonlinear constraints, and the possible introduction of discrete variables.

OMLT is a separate optimization-and-machine-learning toolkit; IDAES provides the integration layer rather than implementing all neural-network mathematical reformulations itself.

### 10.3 Sampling, persistence, and diagnostics

The surrounding surrogate infrastructure includes:

- Latin hypercube sampling.
- Full-factorial sampling.
- Halton and Hammersley low-discrepancy sequences.
- Centroidal Voronoi tessellation sampling.
- Training and validation datasets.
- Parity and residual plots.
- Two- and three-dimensional surrogate visualizations.
- Model serialization and reloading, with formats dependent on the model family.

These features make the surrogate facilities useful even when a surrogate is trained separately from the eventual optimization model.

Critical limitations: The supported input bounds and training domain matter. An optimizer can exploit surrogate extrapolation errors and produce an apparently optimal but physically implausible solution. Validation error, constraint feasibility, extrapolation control, and derivatives of the surrogate must therefore be considered separately.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/surrogate/api/index.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/surrogate/api/pysmo/index.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/surrogate/api/omlt-keras/index.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/surrogate/sampling/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+3



## 11. Process costing and heat integration

### 11.1 Costing as part of the mathematical model

IDAES's costing framework makes equipment and process economics part of the same symbolic model as the physical flowsheet.

Its central abstractions are `FlowsheetCostingBlock` and `UnitModelCostingBlock`.

The costing framework can represent:

- Individual equipment capital costs.
- Aggregate capital costs.
- Fixed operating costs.
- Variable operating costs.
- Material and utility costs.
- Cost flows and economic accounting periods.
- Process-wide economic relationships.
- Cost indices and currency-basis conversion.

A costing package defines its global parameters, individual equipment costing methods, and process-level aggregation relationships.

For instance, a unit's heat-transfer area, capacity, or operating duty can feed directly into a capital-cost expression. The resulting cost can then participate in an optimization objective along with energy consumption and process-performance constraints.

This permits simultaneous economic and physical optimization instead of treating economics strictly as a postprocessing step.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/costing/costing_framework.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 11.2 Available costing libraries

IDAES contains multiple domain-specific costing libraries.

SSLW equipment costing incorporates correlations based on Seider, Seader, Lewin, and Widagdo, covering several classes of conventional process equipment, including vessels, exchangers, compressors, pumps, and related equipment.

NETL power-plant costing provides correlations and costing structures for power-generation technologies, with equipment capital-cost and operating-cost calculations intended for relevant process systems.

The framework is also designed to accept custom equipment-cost methods and costing packages.

These libraries should be treated as engineering cost-estimation models, not universal current-market quotations. Their cost bases, equipment categories, correlations, scaling ranges, and technology assumptions must be appropriate to the process being analyzed.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/costing/process_costing_sslw.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/power_generation/costing/power_plant_costing_netl.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



### 11.3 Heat integration and minimum-utility targeting

A less prominent but technically important capability is `idaes.core.util.utility_minimization`.

Its `min_utility()` function implements a formulation based on Duran–Grossmann simultaneous process optimization and heat integration.

It creates hot- and cold-utility variables and associated inequality constraints for a set of heating and cooling duties, using a specified minimum temperature approach.

This allows utility requirements to be constrained or minimized within the process optimization problem, rather than requiring an entirely separate pinch-analysis calculation.

The module also provides `heat_ex_data()` and `generate_curves()` for extracting heat-exchange information and plotting composite curves.

The distinction is important: utility targeting does not itself synthesize a complete, physically detailed heat-exchanger network. Network topology, individual exchanger feasibility, area, pressure drops, and costs may require additional modeling.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/utility_minimization.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 12. Parameter estimation, reconciliation, and uncertainty propagation

IDAES's equation-oriented models can be used not only to predict process behavior but also to estimate uncertain physical parameters from measured data.

### 12.1 Parameter estimation and data reconciliation

Parameter-estimation problems commonly introduce measured observations and an objective minimizing deviations between observations and model predictions.

The parameters to estimate might include reaction-rate constants, heat-transfer coefficients, equilibrium-model parameters, or equipment performance coefficients.

In an IDAES context, these may be represented as Pyomo variables subject to physical equations and shared across multiple experimental conditions.

Data reconciliation uses a similar mathematical structure, but its focus is different: it adjusts measured or estimated process states so that the resulting values satisfy conservation and process constraints.

An important boundary is that the documented general workflow makes substantial use of Pyomo's `parmest`. IDAES supplies the physical models and integration mechanisms; it does not replace all of Pyomo's parameter-estimation machinery with a separate estimation engine.

Source: `https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 12.2 Uncertainty propagation

IDAES additionally provides `idaes.apps.uncertainty_propagation`, which connects parameter-estimation uncertainty to downstream process-model outputs.

Its two principal operations are:

- `propagate_uncertainty()`: propagates an existing parameter covariance matrix through a process or optimization model.
- `quantify_propagate_uncertainty()`: estimates uncertain parameters and covariance from data, then propagates that uncertainty through a subsequent model.

Suppose model parameters \\(p\\) have covariance matrix \\(\Sigma_p\\), and a solved model produces an output \\(y(p)\\). The local first-order approximation is:

\\[ \operatorname{Var}(y)\approx \frac{dy}{dp}\\, \Sigma_p\\, \left(\frac{dy}{dp}\right)^T \\]

In constrained optimization, the required total derivative also includes how the optimal decision variables change as the parameters change.

IDAES uses the external `k_aug` solver for the sensitivity calculations, together with Pyomo's parameter-estimation support when covariance must first be estimated.

This is particularly useful because it extends uncertainty analysis beyond direct property evaluations to the optimal process outcome.

Its limitations are equally important:

- It is based on local sensitivity and first-order propagation, not a general probabilistic-programming system.
- Highly nonlinear behavior, phase transitions, discontinuities, active-set changes, and large uncertainty can undermine the approximation.
- It requires the external `k_aug` executable.
- The documented interface imposes specific treatment of uncertain parameters, including representing them as variables rather than ordinary fixed Pyomo parameters.

Source: `https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/uncertainty_propagation/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 13. Additional specialized capabilities

### 13.1 MatOpt: mathematical optimization for nanomaterial design

`idaes.apps.matopt` is a distinct modeling extension for designing atomic and crystalline structures through mathematical optimization.

Rather than representing macroscopic process streams, MatOpt represents candidate material structures using lattice sites, atomic occupancy, neighborhood relationships, and structural descriptors.

Its capabilities include:

- Crystalline nanoparticles, nanowires, surfaces, and periodic bulk structures.
- Discrete assignment of atoms or material species to candidate sites.
- Local coordination and neighborhood descriptors.
- Constraints on composition and structural features.
- Optimization of material structures against user-defined objectives.

The mathematical model can be formulated using mixed-integer optimization techniques.

This is architecturally different from IDAES flowsheeting, but shares the broader philosophy of constructing explicit optimization models from domain-level abstractions.

MatOpt should not be confused with an atomistic molecular-dynamics simulator or density-functional-theory engine. Its objectives and structural relationships must be supplied through appropriate models or data.

Source: `https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/matopt/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 13.2 Variable-like expressions

`VarLikeExpression` is a small but useful modeling abstraction.

It allows an ordinary Pyomo expression to serve in a context where users might expect a variable. Unlike a true `Var`, it cannot be independently fixed or assigned a value.

IDAES provides variable-like method handling that produces more informative errors when such operations are attempted.

One use is in ideal separation models, where outlet states can be expressed directly in terms of inlet conditions and split fractions rather than adding unnecessary independent variables and equality constraints.

This can reduce algebraic model size, although it also removes degrees of freedom that would exist if the quantities were independently represented.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/var_like_expr.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 14. Operational tooling, persistence, and extension boundaries

### 14.1 Structured flowsheet execution

IDAES includes a structured flowsheet runner through `idaes.core.util.structfs`.

`FlowsheetRunner` organizes a process-model lifecycle into named operations such as:

`build` → `set_operating_conditions` → `set_scaling` → `initialize` → `solve_initial` → `add_costing` → `solve_optimization` → `check_model_numerics`

The complete supported sequence also includes solver selection, structure checking, and costing initialization.

Steps are registered through decorators, and the runner supports actions before and after individual steps or groups of steps.

This is useful for making flowsheet execution inspectable and repeatable, particularly when instrumentation, diagnostics, or reporting should be attached consistently to model lifecycle events.

The runner is an orchestration interface, not an independent solver, workflow scheduler, or distributed-computing engine.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/structfs/structfs.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 14.2 Model state serialization

IDAES supplies `to_json()` and `from_json()` model-state utilities.

They can serialize and restore selected properties of an existing Pyomo model, including variable values, fixed states, and component activity according to the chosen serialization configuration.

This can support:

- Reusing converged initialization states.
- Transferring numerical solutions between model instances.
- Saving states for analysis and later continuation.
- Comparing the effects of different specifications.

This is not a complete executable-model serialization format. The model's structural definitions and Python construction code are still needed. Restoring state data into a model with incompatible structure can produce missing or mismatched assignments.

Source: `https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/model_serializer.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 14.3 Reporting, tags, and logging

`ModelTag` and `ModelTagGroup` provide an abstraction for assigning human-readable labels, units, and formatting behavior to model quantities.

They support consistent presentation of values, including conversion into reporting units and organization into tabular outputs.

IDAES also provides tagged logging for framework, flowsheet, unit, control-volume, property, and reaction operations. Logging and solver-output behavior can be configured globally or in a running Python session.

These mechanisms are useful for exposing engineering-level information without requiring downstream reporting systems to understand every Pyomo variable's internal name.

IDAES's reporting abstractions are not a replacement for a dedicated database, distributed telemetry platform, or full observability stack.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/util/tags.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/configuration.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



### 14.4 External libraries and compiled extensions

IDAES's capabilities span several dependency boundaries.

| Component      | Relationship to IDAES                                                                     |
| -------------- | ----------------------------------------------------------------------------------------- |
| Pyomo          | Underlying symbolic model, constraints, variables, transformations, and solver interfaces |
| IPOPT          | External nonlinear optimization solver commonly used with IDAES                           |
| PETSc          | Optional external nonlinear-equation and DAE solver                                       |
| k_aug          | External sensitivity solver for uncertainty propagation                                   |
| CoolProp       | Optional thermophysical-property integration                                              |
| OMLT           | External mathematical optimization formulations for machine-learning models               |
| ALAMO          | External algebraic surrogate-generation software                                          |
| Prescient      | External electricity-market simulation engine                                             |
| IDAES UI       | Optional companion interface for working with flowsheet models                            |
| IDAES Examples | Separately distributed examples and notebook materials                                    |

The documented pip extras include `ui`, `grid`, `omlt`, and `coolprop`. Some integrations also require non-Python binaries or independently installed applications.

The `idaes get-extensions` command retrieves supported platform-specific compiled extensions, including solver executables and property-function libraries. The PETSc binaries can be requested with `--extra petsc`.

Important CLI commands include `idaes --version`, `idaes environment-info`, `idaes get-extensions`, `idaes bin-directory`, and `idaes data-directory`.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/how_to_guides/opt_dependencies.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/commands/get_extensions.html`

`https://idaes-pse.readthedocs.io/en/stable/tutorials/getting_started/linux.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



### 14.5 Developing new model components

The framework provides documented extension interfaces for custom:

- Unit operations.
- Property packages.
- Reaction packages.
- Initialization strategies.
- Scaling methods.
- Costing packages.
- Surrogate models and external calculations.

The most consequential extension points are those that preserve the contracts of the existing framework. A custom model can participate in generic conservation equations, scaling, diagnostics, and optimization if its variables, property methods, configuration, and initialization behavior are compatible.

Custom models can also bypass some of those abstractions and construct Pyomo equations directly. This offers flexibility, but may sacrifice the automatic interoperability provided by standardized IDAES interfaces.

The main development guides are:

`https://idaes-pse.readthedocs.io/en/stable/how_to_guides/custom_models/index.html`

`https://idaes-pse.readthedocs.io/en/stable/reference_guides/core/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 15. Important boundaries, maturity distinctions, and architectural implications

### 15.1 Released versus historical functionality

Older IDAES documentation describes capabilities that should not be assumed available in version 2.13.0.

| Feature                                                             | 2.13.0 interpretation                                                                                  |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| RIPE                                                                | Archived; removed from the current codebase                                                            |
| HELMET                                                              | Archived; removed from the current codebase                                                            |
| Data Management Framework (DMF)                                     | No longer supported beginning with IDAES 2.6                                                           |
| Caprese MHE                                                         | Documentation states the module is forthcoming; not established as a complete supported implementation |
| MSContactor heterogeneous-reaction integration                      | Documented beta interface                                                                              |
| Surrogate models, costing, diagnostics, and uncertainty propagation | Documented released functionality, subject to their dependencies and limitations                       |

The distinction between archived HELMET and the current Helmholtz property-model infrastructure is particularly important. Helmholtz-based property calculations remain available; the historical HELMET regression extension has been removed.

Sources:

`https://idaes-pse.readthedocs.io/en/stable/archived_features/index.html`

`https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html`

`https://idaes-pse.readthedocs.io/en/stable/explanations/modeling_extensions/caprese/mhe.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



### 15.2 Fundamental technical tradeoffs

| Strength                                            | Corresponding limitation                                                               |
| --------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Simultaneous equation-oriented process optimization | Large nonlinear systems can be difficult to initialize and solve                       |
| Physically meaningful modular model interfaces      | Package compatibility and chemical semantics must still be established                 |
| Flexible thermodynamic formulations                 | Coverage and physical validity remain dependent on the chosen property package         |
| Dynamic and distributed modeling                    | Discretization can greatly increase equation count and numerical stiffness             |
| Surrogate integration                               | Approximation error can be amplified by optimization                                   |
| Structural and numerical diagnostics                | Diagnostic findings do not automatically provide a physically correct formulation      |
| Broad access to Pyomo optimization features         | Solver support, optimality guarantees, and computational costs depend on problem class |
| Specialized equipment and energy-system models      | Individual models vary in fidelity, applicability, and maintenance maturity            |

Three distinctions deserve special emphasis.

Equation-oriented versus sequential-modular simulation. IDAES supports sequential initialization, but its primary modeling paradigm is simultaneous solution of interconnected equations. This opens more direct optimization opportunities while imposing stronger requirements on initialization, scaling, and nonlinear solver behavior.

Property calculation versus property-model integration. A thermodynamic library that numerically evaluates a property at a state is not automatically interchangeable with an IDAES property package. IDAES unit and control-volume models expect particular symbolic properties, derivative behavior, state-variable specifications, and physical interfaces. Wrapping an external property engine may therefore require additional equations or derivative-compatible interfaces.

Model capability versus physical fidelity. Providing a reactor, separator, or heat exchanger abstraction does not guarantee that the chosen correlations, assumptions, and discretization describe a particular process accurately. Verification against suitable experimental or reference data remains separate from correct software construction.

### 15.3 Particularly valuable capability combinations

Several combinations illustrate where IDAES can eliminate substantial custom mathematical-modeling infrastructure.

| Combination                                             | Architectural opportunity                                                                            |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Property packages + reaction packages + control volumes | Build physically consistent reactor models without independently implementing every balance equation |
| Generic equipment + ports/arcs + nested flowsheets      | Assemble reusable process subsystems with explicit process connectivity                              |
| Costing + equipment sizing + Pyomo objectives           | Optimize process configuration and economics in one mathematical problem                             |
| Dynamic balances + PID + Caprese                        | Study nonlinear process-control interactions using the physical flowsheet                            |
| SurrogateBlock + process models                         | Create hybrid first-principles/data-driven optimization models                                       |
| Initializers + scaling + diagnostics + homotopy         | Establish reusable mechanisms for difficult nonlinear-solution workflows                             |
| Multi-period models + grid integration                  | Link equipment performance to time-varying operating economics                                       |
| Parameter estimation + k_aug uncertainty propagation    | Connect measured-data uncertainty to downstream optimal process outcomes                             |

These are capabilities and potential combinations, not guarantees that the resulting system will be numerically tractable. The value of any composition depends on the quality of its physical models, available solvers, dimensionality, and required accuracy.

## 16. Overall assessment

IDAES-PSE is best understood as a composable process-systems engineering framework built over a mathematical optimization language.

Its major differentiation is not any particular piece of equipment or thermodynamic correlation. It is the integration of domain-specific models with reusable mathematical infrastructure:

- Conservation laws and configurable physical properties.
- Chemical kinetics and equilibrium.
- Hierarchical equipment and flowsheet representations.
- Steady-state, dynamic, and multi-period formulations.
- Physical, economic, and control optimization.
- Advanced initialization and numerical diagnostics.
- Data-driven surrogate and uncertainty-analysis tools.
- Standardized extension mechanisms.

For sophisticated engineering systems, the most valuable less-obvious features are arguably the control-volume framework, property-package contracts, block-triangular initialization, scaling and degeneracy diagnostics, PETSc integration, surrogate embedding, and integrated costing.

Together, they offer a path to systems in which specialized physical models remain independently maintainable while sharing a common foundation for conservation equations, numerical analysis, and optimization.

The principal cost of that flexibility is that an IDAES application remains a carefully formulated mathematical model. Its correctness, physical validity, convergence, and performance cannot be delegated entirely to the framework.

Research basis: Official IDAES 2.13.0 documentation, current release information, and project-maintained model and extension references. This is a documentation-based capability assessment; no local model execution, solver validation, or performance benchmarking was performed.

# IDAES-PSE 2.13.0 — Capability Reference Addendum

Research date: November 1, 2026 | Target: IDAES-PSE 2.13.0, the latest verified stable release

This addendum supplements the original IDAES-PSE library overview with functionality that was omitted or insufficiently developed. It concentrates on additional modeling abstractions, thermodynamic capabilities, mathematical and numerical utilities, advanced optimization mechanisms, specialized equipment, and consequential integration distinctions.

The additions are organized by capability rather than by API inventory. Existing descriptions of the core flowsheet architecture, standard equipment, general solvers, and basic initialization are not repeated.

## 1. Property and reaction requirement interrogation

IDAES provides a property-requirement discovery mechanism that allows a flowsheet's thermophysical and reaction-property dependencies to be identified before implementing complete property and reaction packages.

The main interfaces are:

- `idaes.models.properties.interrogator.PropertyInterrogatorBlock`
- `idaes.models.properties.interrogator.ReactionInterrogatorBlock`

These substitute for conventional physical and reaction parameter blocks during provisional flowsheet construction.

Instead of building executable property expressions, they record requests for thermophysical and reaction quantities made by unit models as the flowsheet is assembled.

### 1.1 Dependency discovery

Both interrogator blocks expose:

| Method                                      | Functionality                                              |
| ------------------------------------------- | ---------------------------------------------------------- |
| `list_required_properties()`                | Enumerates requested properties across the flowsheet       |
| `list_models_requiring_property(prop)`      | Identifies models that request a particular property       |
| `list_properties_required_by_model(model)`  | Identifies the properties requested by a particular unit   |
| `print_required_properties()`               | Produces a human-readable requirements summary             |
| `print_models_requiring_property(prop)`     | Reports consumers of a property                            |
| `print_properties_required_by_model(model)` | Reports the requirements of a unit model                   |
| `required_properties`                       | Stores property names mapped to the requesting model names |

Property and reaction requirements are recorded independently. The physical interrogator identifies requests ordinarily satisfied through `StateBlock` interfaces; the reaction interrogator identifies requests ordinarily satisfied through `ReactionBlock` interfaces.

The mechanism can be used to construct a property dependency map relating unit models to the physical quantities they consume.

For instance, a proposed reactor-separator-exchanger network may require phase enthalpy, component density, fugacity, reaction rates, and equilibrium constants. The interrogator can identify which unit models request which properties without requiring a complete thermodynamic implementation.

### 1.2 Architectural significance and limitations

This creates an additional modeling stage between defining equipment structure and implementing thermodynamic relationships.

A reusable property package can be developed against discovered requirements rather than relying solely on manual examination of each unit model.

The important limitation is that interrogation is construction-time dependency recording, not full static or dynamic program analysis.

A property request made only through an optional or subsequently introduced modeling path may not be observed. Custom constraints created outside the interrogated construction path can introduce additional requirements. Interrogator flowsheets are intentionally not solvable.

Consequently, the recorded dependencies are evidence of observed property requirements, not a proof of interface completeness or thermodynamic consistency.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



Source — Property Interrogator Tool:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/property_models/interrogator.html`

## 2. Electrolyte thermodynamics and chemical-species representations

The modular property framework includes explicit support for electrolyte systems, with a distinction between true chemical species and apparent chemical components.

This capability is significant because electrolyte chemistry can require two different views of the same material state.

An aqueous salt solution, for example, may be specified in terms of neutral compounds while its equilibrium and activity calculations depend on dissolved ions.

### 2.1 True and apparent species

The electrolyte framework distinguishes:

| Representation               | Meaning                                                                                                           |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| True species                 | Species considered physically present in the thermodynamic model, including ions, solvents, and molecular solutes |
| Apparent species             | Chemical components used to represent material composition before accounting for dissociation                     |
| True phase-component set     | Valid phase/species combinations using the true representation                                                    |
| Apparent phase-component set | Corresponding combinations using the apparent representation                                                      |

`GenericParameterBlock` exposes the `state_components` option, which selects `StateIndex.true` or `StateIndex.apparent`.

This selection affects which species index the state-variable and material-flow interfaces expose.

For electrolyte systems, the framework constructs additional sets and relationships for converting between representations.

This is not merely a naming convention. True and apparent composition can have different meanings for conservation, reaction stoichiometry, property calculations, and stream interfaces.

For example, an apparent component may represent one formula unit of a salt, while the corresponding true representation contains separately modeled cations and anions.

The distinction can allow different equipment formulations to use chemically appropriate representations without having to independently implement every conversion.

### 2.2 Electrolyte component and phase definitions

The generic framework recognizes component and phase types relevant to electrolyte chemistry, including:

- `Solvent`
- `Solute`
- `Cation`
- `Anion`
- `Apparent`
- `AqueousPhase`

The component definitions establish chemically distinct roles, and the package constructs electrolyte-specific species sets from those declarations.

An aqueous phase also triggers a compatibility check against the selected equation-of-state implementation. The equation of state must explicitly advertise electrolyte support.

This matters because conventional ideal or cubic equation-of-state classes cannot simply be assumed suitable for aqueous ions.

### 2.3 Electrolyte property metadata

`ElectrolytePropertySet` extends the standard property metadata with electrolyte-specific quantities.

These include:

| Property                 | Meaning                                                         |
| ------------------------ | --------------------------------------------------------------- |
| `ionic_strength`         | Ionic strength on a molality basis                              |
| `log10_molality`         | Logarithmic molality                                            |
| `log10_act_coeff`        | Logarithmic activity coefficient                                |
| `log_act_phase_solvents` | Solvent activity                                                |
| `pH`, `pOH`              | Logarithmic hydrogen- and hydroxide-ion activity measures       |
| `log10_k_eq`, `pK`       | Equilibrium-constant representations                            |
| `saturation_index`       | Logarithmic ion-activity product relative to solubility product |

The metadata distinguishes properties that have different physical definitions despite superficially similar units or names.

A particularly important distinction is that a property declared in the metadata is not automatically a corresponding thermodynamic calculation. Actual support depends on the selected property and equation-of-state modules.

### 2.4 Inherent reactions and equilibrium constraints

The modular physical-property framework supports inherent reactions: reactions defined as part of the material-property system rather than attached only through a separate reaction package.

These are appropriate when chemical equilibrium forms part of the definition of the local thermodynamic state.

The source implementation also distinguishes initialization behavior for inherent-equilibrium constraints, including cases where the constraints cannot be solved independently at the state-block level and require surrounding conservation equations.

This creates an important interaction between thermodynamic equilibrium, reaction chemistry, and control-volume balances.

Electrolyte support therefore introduces additional requirements for model solvability, charge and species consistency, phase behavior, and initialization. IDAES supplies infrastructure for these requirements, but does not guarantee that arbitrary electrolyte systems are thermodynamically complete or readily solvable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



Sources — Electrolyte Property Set and generic property implementation:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/electrolyte_property_set.html`

`https://idaes-pse.readthedocs.io/en/2.13.0/_modules/idaes/models/properties/modular_properties/base/generic_property.html`

## 3. Extended state-variable architecture

### 3.1 Additional `FpTPxpc` state formulation

IDAES 2.13.0 includes an additional modular state-definition implementation, `FpTPxpc`, absent from the previous overview's list.

Its independent state variables include:

- `flow_mol_phase`: phase-specific molar flows.
- `mole_frac_phase_comp`: composition within each phase.
- `temperature`: state temperature.
- `pressure`: state pressure.

This differs from formulations using overall flow and overall composition.

The implementation derives total flow through an expression such as

\\[ F=\sum_p F_p \\]

and relates overall composition to phase compositions through

\\[ Fz_j=\sum_p F_p x\_{p,j} \\]

where \\(F_p\\) is the molar flow of phase \\(p\\), \\(z_j\\) is overall mole fraction, and \\(x\_{p,j}\\) is phase-specific composition.

This representation exposes phase quantities directly rather than obtaining every phase quantity through an overall-composition flash calculation.

That can be useful when phase flows and compositions are themselves natural equipment or process specifications.

The implementation includes interfaces for material-flow terms, enthalpy-flow terms, density terms, default balance types, ports, initialization, and scaling.

However, the tagged 2.13.0 source contains an explicit TODO for flash initialization. The existence of the state formulation therefore should not be interpreted as evidence that it provides a fully developed flash-initialization procedure for every multiphase application.

The functionality is present in the released source, even though the state-definition documentation page does not list it alongside all older formulations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source — Tagged 2.13.0 implementation:

`https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/models/properties/modular_properties/state_definitions/FpTPxpc.py`

### 3.2 State definitions are full behavioral interfaces

An IDAES state-definition module does considerably more than choose independent variables.

It also supplies relationships needed by the broader framework, including:

- `define_state_vars()`: identifies independent state variables.
- `define_port_members()`: determines quantities exposed through stream ports.
- `define_display_vars()`: identifies quantities suitable for reporting.
- `get_material_flow_terms()`: supplies phase/component material flows to control volumes.
- `get_enthalpy_flow_terms()`: supplies energy-flow terms.
- `get_material_density_terms()` and `get_energy_density_terms()`: support accumulation calculations.
- `get_material_flow_basis()`: identifies molar or mass basis.
- Default material- and energy-balance type methods.
- `state_initialization()`: establishes initial values for supporting state relationships.

These methods constitute an interface contract between the property package, process connectivity, and conservation-equation generation.

A custom state formulation must therefore preserve more than mathematical thermodynamic sufficiency. It must provide the methods needed by downstream unit and control-volume implementations.

This is an important extensibility distinction: independent thermodynamic state variables and the public process-model interface are related, but are not identical.

Source — Developing State Definitions:

`https://idaes-pse.readthedocs.io/en/2.13.0/explanations/components/property_package/general/developer/state_definition_dev.html`

## 4. Helmholtz thermodynamics: expression-level access and configurable phase presentation

The Helmholtz implementation has several specialized capabilities beyond its use as a conventional pure-fluid property package.

### 4.1 Property evaluation without a StateBlock

`HelmholtzThermoExpressions` provides direct access to Helmholtz property relationships as Pyomo expressions.

It can attach the necessary external functions to a Pyomo block and construct expressions from supported combinations of state variables.

These include:

| Input representation         | State variables |
| ---------------------------- | --------------- |
| Enthalpy–pressure            | \\(h,P\\)       |
| Internal energy–pressure     | \\(u,P\\)       |
| Entropy–pressure             | \\(s,P\\)       |
| Entropy–temperature          | \\(s,T\\)       |
| Temperature–vapor quality    | \\(T,x\\)       |
| Pressure–vapor quality       | \\(P,x\\)       |
| Temperature–pressure–quality | \\(T,P,x\\)     |

The expression writer can calculate properties such as enthalpy, entropy, density, heat capacity, saturation quantities, and other fluid properties supported by the selected Helmholtz model.

This interface is different from constructing a full `HelmholtzStateBlock`.

A state block participates in the IDAES stream and control-volume architecture, with independent state variables and standardized material and energy interfaces. The expression writer can instead supply an individual property relationship for a custom equation, constraint, or calculation.

For example, a specialized equipment correlation may require the saturated-liquid enthalpy at a particular pressure without needing an additional stream state. The expression writer can provide the required thermodynamic expression without constructing another full state block.

These remain Pyomo expressions using compiled external thermodynamic functions, rather than independently expanded algebraic representations of every Helmholtz relationship.

### 4.2 Phase presentation is independent of internal phase calculations

`HelmholtzParameterBlock` supports four phase-presentation configurations.

| Mode            | Model-facing representation                              |
| --------------- | -------------------------------------------------------- |
| `PhaseType.MIX` | One mixed phase, potentially containing liquid and vapor |
| `PhaseType.LG`  | Separate liquid and vapor phases                         |
| `PhaseType.L`   | Liquid-only representation                               |
| `PhaseType.G`   | Vapor-only representation                                |

`MIX` can reduce the size and complexity of equipment equations when explicit phase separation is unnecessary.

Crucially, a mixed presentation does not eliminate the internal phase-equilibrium calculation. Vapor fraction and phase-specific properties can still be evaluated.

However, a separator requiring independently modeled liquid and vapor streams cannot obtain that separation directly from a single model-facing mixed phase.

The `LG` presentation exposes separate phases but can introduce additional balance-equation requirements. The documentation generally recommends total material balances where appropriate, because the Helmholtz property package already determines phase equilibrium.

This makes phase presentation a mathematical modeling choice, not merely a reporting option.

### 4.3 Fluid registry and extensible substance definitions

The implementation also includes a registry for available pure-fluid models.

Relevant interfaces include:

- `registered_components()`
- `component_registered()`
- `viscosity_available()`
- `thermal_conductivity_available()`
- `surface_tension_available()`
- `eos_reference()` and corresponding transport-model reference methods
- `get_parameter_path()` and `set_parameter_path()`

Additional Helmholtz substances can be introduced through compatible parameter and expression files.

The main equation-of-state model is required, while viscosity, thermal conductivity, and surface-tension models are optional.

This means an available fluid model is not necessarily a complete source of every transport property.

The expression writer also distinguishes mass and molar bases through `AmountBasis`. It accepts SI-based state inputs and selected conversion options, but its documentation explicitly limits arbitrary automatic unit conversion because of model-construction cost.

Architectural significance: IDAES exposes two complementary thermodynamic interfaces: a process-compatible state-block interface and a more direct property-expression interface. The latter can be valuable when only individual thermodynamic relationships are needed.

The compiled function library must be available on the target platform, and supported differentiation and numerical behavior remain dependent on the underlying implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+2



Sources — Helmholtz property package and expression writer:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/property_models/helmholtz.html`

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/property_models/helmholtz_expressions.html`

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/property_models/helmholtz_parameters.html`

## 5. Smooth mathematical operators and differentiable approximations

`idaes.core.util.math` supplies mathematical expressions designed for equation-oriented models in which nonsmooth functions or singularities can destabilize nonlinear solution methods.

These are reusable modeling primitives rather than complete physical models.

| Function                          | Mathematical role                         |
| --------------------------------- | ----------------------------------------- |
| `smooth_abs(a, eps)`              | Smooth absolute value                     |
| `smooth_min(a, b, eps)`           | Smooth approximation to a minimum         |
| `smooth_max(a, b, eps)`           | Smooth approximation to a maximum         |
| `smooth_minmax(a, b, eps, sense)` | Configurable smooth minimum or maximum    |
| `smooth_bound(val, lb, ub, ...)`  | Smoothly limits a quantity to an interval |
| `smooth_heaviside(x, k)`          | Logistic approximation to a step function |
| `safe_log(a, eps)`                | Smoothly protected logarithm              |
| `safe_sqrt(a, eps)`               | Smoothly protected square root            |

### 5.1 Common smoothing formulation

The smooth absolute-value approximation is

\\[ |a|\_\epsilon=\sqrt{a^2+\epsilon^2}. \\]

The smooth maximum is then constructed as

\\[ \operatorname{smax}(a,b) =\frac{a+b+\sqrt{(a-b)^2+\epsilon^2}}{2}. \\]

An analogous expression gives the smooth minimum.

As \\(\epsilon\\) decreases, the approximation approaches the original nonsmooth operator, while its derivatives become more sharply varying near the transition.

These functions preserve a smooth symbolic structure suitable for derivative-based nonlinear optimization.

### 5.2 Bounds and domain protection

`smooth_bound()` composes smooth minimum and maximum operations to approximate clipping a value to a specified interval.

A representative application is a controller whose unconstrained output can exceed the physical operating range of an actuator.

Similarly, `safe_log()` and `safe_sqrt()` are intended to avoid transient evaluation failures when nonlinear iterations pass through states outside the ordinary mathematical domain.

For example, `safe_log(a, eps)` approximates the logarithm of a smoothly bounded version of \\(a\\), providing a finite expression when a trial iterate would otherwise evaluate a logarithm at a nonpositive argument.

This is a numerical robustness mechanism, not a replacement for physical domain constraints.

If the actual solution lies near or outside the protected domain, the smoothed formulation may materially change the model. A suitably chosen smoothing parameter should reflect both physical validity and solver conditioning.

Architectural significance: Common differentiable approximations do not need to be independently reimplemented. Their consistent use can also simplify numerical scaling and derivative analysis across independently developed equations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



Source — Mathematical Utility Methods:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/util/math.html`

## 6. Advanced numerical introspection and solver-convergence regression

The earlier overview covered high-level structural and Jacobian diagnostics. Several additional utilities support deeper investigation of individual expressions, numerical rank, infeasibility, and changes in solver performance.

### 6.1 Constraint-term analysis

`ConstraintTermAnalysisVisitor` traverses Pyomo expression trees to identify problematic terms in constraint expressions.

It focuses on two particular phenomena:

Mismatched term magnitudes. A constraint may contain additive terms differing by many orders of magnitude. Such differences can create floating-point sensitivity even when the overall residual is small.

Numerical cancellation. Large positive and negative contributions may nearly cancel, producing a result whose accuracy is sensitive to finite precision.

The visitor exposes configurable tolerances including:

- `term_mismatch_tolerance`
- `term_cancellation_tolerance`
- `term_zero_tolerance`
- `max_canceling_terms`
- `max_cancellations_per_node`

This differs from merely inspecting constraint residuals.

A residual answers whether the current variables approximately satisfy an equation. Constraint-term analysis investigates whether the arithmetic used to evaluate that equation is potentially ill-conditioned.

It can therefore identify expressions that appear to be feasible but may be numerically fragile.

Source:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/util/diagnostics/constraint_analysis.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 6.2 Ill-conditioning certificates

`compute_ill_conditioning_certificate()` examines the constraint Jacobian to identify rows or columns that may contribute to numerical ill-conditioning.

It supports `direction="row"` for constraint-oriented analysis and `direction="column"` for variable-oriented analysis.

The method also exposes tolerance controls, including `target_feasibility_tol` and `ratio_cutoff`.

Its role is distinct from singular-value decomposition.

SVD identifies nearly dependent numerical directions and their singular values. An ill-conditioning certificate seeks a compact explanation involving particular constraints or variables, using an auxiliary mathematical formulation.

This can help localize problematic equations or variable relationships.

The identified contributors should be interpreted as diagnostics rather than proof that the corresponding physical equations are unnecessary or incorrect.

Source:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/util/diagnostics/ill_conditioning.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.12.0



### 6.3 Expression evaluation, bounds, and solver failures

The diagnostic utilities also include specialized approaches for:

- Examining variables close to or violating their bounds.
- Identifying invalid expression-evaluation domains.
- Investigating IPOPT function-evaluation errors.
- Inspecting problematic constraint terms and extreme Jacobian entries.
- Identifying constraints with no free variables.
- Examining unused or unconstrained variables.

These tools distinguish several failure modes that can produce superficially similar solver messages.

For example, a nonlinear solver's failure may originate from an undefined logarithm, a genuine infeasibility, an incorrectly fixed variable, or a badly scaled but mathematically valid system.

The distinction matters because the appropriate corrective action differs in each case.

Source — Model Diagnostic Functions:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/util/model_diagnostics.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 6.4 IPOPT convergence-regression analysis

`IpoptConvergenceAnalysis` extends numerical testing beyond evaluating whether a single model instance solves successfully.

The tool performs parameter sweeps and records convergence behavior across multiple operating conditions.

Its capabilities include:

| Method                                 | Functionality                                                        |
| -------------------------------------- | -------------------------------------------------------------------- |
| `run_convergence_analysis()`           | Executes an analysis using a configured parameter-sweep runner       |
| `run_convergence_analysis_from_dict()` | Loads and executes an analysis specification from a dictionary       |
| `run_convergence_analysis_from_file()` | Executes a specification loaded from JSON                            |
| `report_convergence_summary()`         | Summarizes observed convergence results                              |
| `compare_convergence_to_baseline()`    | Compares a new run with recorded baseline behavior                   |
| `assert_baseline_comparison()`         | Raises an assertion when results differ beyond configured tolerances |
| `to_json_file()`                       | Serializes analysis specification and results                        |

The default runner is `SequentialSweepRunner`, although the class accepts other compatible IDAES parameter-sweep implementations.

The important architectural distinction is between physical-model correctness testing and numerical-behavior regression testing.

A modified model may continue to produce the expected result but require substantially more IPOPT iterations. The convergence-analysis utility can surface this deterioration across a representative set of operating conditions.

Conversely, a reduced iteration count does not establish that the physical model remains correct.

Results must be interpreted in the context of the solver configuration, initialization strategy, hardware environment, and tolerances used.

Source:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/util/diagnostics/convergence_analysis.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 6.5 Model-statistics and structural inspection

`idaes.core.util.model_statistics` exposes a broad set of functions for examining Pyomo model components.

Beyond `degrees_of_freedom()` and `report_statistics()`, these include interfaces for identifying active and inactive constraints, equality and inequality constraints, fixed and unfixed variables, unused variables, and variables appearing only in particular kinds of constraints.

These can be used to construct programmatic descriptions of model size and structure without independently traversing every Pyomo block.

One important semantic qualification is that `degrees_of_freedom()` performs a structural count, not a full numerical rank calculation.

It excludes inequalities and certain unused or disconnected variables from the count.

A reported degree-of-freedom value of zero therefore does not prove that a model has a unique solution, is feasible, or has a nonsingular Jacobian.

Combined with Dulmage–Mendelsohn analysis, constraint-term diagnostics, and Jacobian inspection, these utilities offer several independent views of a mathematical model's structure.

Source:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/core/util/model_statistics.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 7. Advanced multi-period optimization and electricity-market operations

The `idaes.apps.grid_integration.pricetaker` package provides substantially more than repeated flowsheet construction. It contains abstractions for separating equipment design, operating decisions, storage, time-series economics, and market-participation constraints.

### 7.1 DesignModel, OperationModel, and StorageModel

Three complementary abstractions support multi-period systems:

| Abstraction      | Responsibility                                                                      |
| ---------------- | ----------------------------------------------------------------------------------- |
| `DesignModel`    | Represents non-time-varying equipment design decisions and associated relationships |
| `OperationModel` | Represents the operating behavior of equipment within a time period                 |
| `StorageModel`   | Represents inventory-dependent equipment and relationships between periods          |

`PriceTakerModel.build_multiperiod_model()` constructs repeated flowsheet instances from a user-provided function.

When storage models are present, it can automatically generate inter-period linking constraints and, optionally, periodic constraints connecting the beginning and end of the modeled horizon.

The distinction between these constraints is consequential.

Linking constraints preserve continuity between consecutive periods, such as relating storage inventory at the end of one period to inventory at the beginning of the next.

Periodic constraints impose a cyclic boundary condition over the entire horizon, such as requiring the final inventory to equal the initial inventory.

Periodic operation is appropriate for some repeating schedules, but can be physically inappropriate for processes with net inventory accumulation or depletion.

This architecture allows an optimization problem to share equipment design variables across many periods while retaining distinct operating variables and inventories.

It also creates an alternative to a fully continuous-time DAE model when the desired time resolution and physical assumptions justify discrete-period operation.

### 7.2 Representative-day selection and price-series reduction

`PriceTakerModel` includes functionality for reducing long electricity-price histories into representative operating periods.

`append_lmp_data()` accepts locational marginal price data and can apply k-means clustering to create representative days.

`get_optimal_representative_days()` evaluates alternative cluster counts, including silhouette-based selection and optional elbow plots.

These methods reduce the number of operating periods included in an optimization model, potentially reducing the number of variables and constraints.

The benefit depends on what the clustering preserves. Representative-day selection based on price similarity does not necessarily preserve extreme events, long-duration storage needs, or chronological relationships between days.

Furthermore, `update_operation_params()` explicitly does not support the representative-days case, an important restriction when considering repeated solves with updated operating data.

### 7.3 Commitment, ramping, and operating-state constraints

The package includes methods for generating operating restrictions that would otherwise require substantial custom mixed-integer modeling.

| Method                                          | Mathematical functionality                                        |
| ----------------------------------------------- | ----------------------------------------------------------------- |
| `add_capacity_limits()`                         | Enforces operating capacity and minimum stable output             |
| `add_startup_shutdown()`                        | Enforces minimum up/down times and startup/shutdown relationships |
| `add_ramping_limits()`                          | Enforces startup, shutdown, and normal-operation ramp limits      |
| `add_linking_constraints()`                     | Connects values across consecutive periods                        |
| `add_periodic_constraints()`                    | Connects final and initial horizon states                         |
| `fix_operation_var()` / `unfix_operation_var()` | Applies or releases repeated-period specifications                |

Version 2.12 added support for multiple startup types, allowing more detailed representation of different startup conditions.

These features extend multi-period simulation into mixed-integer process optimization. Consequently, solver requirements may be materially different from those of an ordinary continuous nonlinear flowsheet.

### 7.4 Integrated cash flow and economic objectives

`add_hourly_cashflows()` aggregates operating-period revenue and cost expressions, including multiple revenue streams.

A process may, for example, receive revenue from electricity and chemical production while incurring fuel, electricity-consumption, carbon, and other operating costs.

`add_overall_cashflows()` introduces lifetime, discount-rate, taxation, and annualization relationships.

`add_objective_function()` supports the documented objective types:

| Objective      | Meaning                      |
| -------------- | ---------------------------- |
| `npv`          | Annualized net present value |
| `net_profit`   | Net operating profit         |
| `lifetime_npv` | Lifetime net present value   |

The package also provides methods for retrieving design-variable results, operational profiles, startup and shutdown counts, and price-signal visualizations.

This allows detailed process performance and financial objectives to participate in a unified optimization problem, subject to the limitations of the chosen economic assumptions.

Source — Price Taker:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/apps/grid_integration/multiperiod/Price_Taker.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



### 7.5 Alternative market-bidding strategies

The broader grid-integration package includes three distinct bidder implementations.

`Bidder` formulates a two-stage stochastic optimization problem to construct market bids, using uncertain price scenarios and associated operating decisions.

`SelfScheduler` uses stochastic optimization to determine self-scheduled resource operation rather than constructing the same kind of price-quantity offer curve.

`PEMParametrizedBidder` constructs parameterized bids for a renewable generator coupled with a polymer electrolyte membrane electrolyzer. Its formulation accounts for reserving renewable electricity for hydrogen production and the associated opportunity cost.

The PEM-specific implementation is not a general-purpose bidder for arbitrary process models. Its economic relationships encode particular assumptions about renewable generation and hydrogen co-production.

The documentation also describes separate day-ahead and real-time bidding problems, including real-time underbidding penalties.

The surrounding `Forecaster`, `Tracker`, and `Coordinator` abstractions manage distinct responsibilities: uncertainty forecasts, operational tracking, and interaction with the external electricity-market simulator.

Source — Bidder:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/apps/grid_integration/Bidder.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0



## 8. ONNX neural-surrogate import and optimization embedding

The earlier surrogate-modeling discussion identified Keras and OMLT but did not explicitly describe ONNX model support.

IDAES added ONNX surrogate support through OMLT before version 2.13.0. Version 2.12 subsequently expanded its ONNX parser to accept Softplus activation functions.

This matters because a neural-network surrogate need not originate from the Keras/TensorFlow ecosystem.

### 8.1 Interchange and optimization representation

The relevant architecture has three distinct layers.

| Layer                             | Responsibility                                                                             |
| --------------------------------- | ------------------------------------------------------------------------------------------ |
| Neural-network training framework | Constructs and trains the original network                                                 |
| ONNX and OMLT                     | Represents a supported network and converts it into an optimization-compatible formulation |
| IDAES surrogate integration       | Connects the resulting surrogate to process-model variables and constraints                |

The distinction between importing a neural network and embedding it into optimization is important.

An ONNX model may be executable by a general-purpose inference runtime without being supported by an optimization parser.

OMLT must understand the network structure and provide a mathematical representation of its operations.

Depending on the architecture and formulation, this may introduce nonlinear constraints or mixed-integer relationships.

### 8.2 Consequential restrictions

Supported ONNX operators, activation functions, network structures, and model representations must be checked against the installed OMLT and IDAES versions.

Softplus support in IDAES 2.12 illustrates why operator-level compatibility matters: an activation function may be straightforward to evaluate numerically while requiring additional work to embed in symbolic optimization.

Input scaling, output scaling, and valid training-domain bounds remain essential. An optimizer can exploit inaccurate surrogate behavior, especially outside the domain represented by its training data.

ONNX import also has a dependency advantage: when using a supported ONNX representation, TensorFlow need not necessarily be installed solely to embed the network. Keras-based workflows may have additional TensorFlow and Python-version constraints.

Source — OMLT/Keras surrogate integration:

`https://idaes-pse.readthedocs.io/en/2.13.0/explanations/modeling_extensions/surrogate/api/omlt-keras/index.html`

Source — Version 2.12 release notes:

`https://github.com/IDAES/idaes-pse/releases/tag/2.12.0`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.12.0+1



## 9. Additional specialized unit models and physical representations

Several specialized models deserve recognition because they extend the supported physical representations beyond standard lumped reactors, separators, and heat exchangers.

These models differ in geometry, transport resolution, and governing equations. Their value is not simply that they add more equipment names to the library.

### 9.1 Additional equipment capabilities

| Model                       | Distinctive functionality                                            |
| --------------------------- | -------------------------------------------------------------------- |
| `Pipe`                      | General pipe unit model introduced in version 2.11                   |
| `BoilerHeatExchanger2D`     | Distributed heat transfer with radial wall-conduction representation |
| `Drum1D`                    | Spatially resolved steam-drum behavior                               |
| `CrossFlowHeatExchanger1D`  | One-dimensional exchanger representation for cross-flow arrangements |
| Three-stream heat exchanger | Heat transfer involving three coupled process streams                |
| Dynamic feedwater heater    | Transient feedwater-heating equipment                                |
| `FeedFlash`                 | Feed boundary with an integrated flashing calculation                |
| Solid-oxide-cell models     | Specialized electrochemical, transport, and thermal representations  |

The 2.13.0 release also changed the radial domain representation in `Drum1D` to a dimensionless formulation, a version-specific detail relevant to model internals.

### 9.2 Two-dimensional boiler heat exchange

`BoilerHeatExchanger2D` provides more detailed thermal representation than a simple heat-duty relationship.

Its documented modeling includes tube-wall conduction, temperature distributions, heat-transfer correlations, pressure losses, and configurable treatment of additional heat-transfer effects.

The radial temperature variation of a tube wall is significant when the wall's thermal resistance and temperature distribution cannot be neglected.

A generic lumped exchanger may capture overall inlet/outlet energy conservation, whereas a distributed exchanger can represent local thermal behavior and its relationship to geometry and operating conditions.

This additional fidelity introduces spatial discretization, more variables, and a potentially more challenging nonlinear system.

Source — Boiler Heat Exchanger 2D:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/power_generation/unit_models/boiler_heat_exchanger2D.html`

### 9.3 Three-stream and cross-flow exchangers

Three-stream exchangers introduce thermal interactions among three different streams, potentially with distinct temperatures, flow directions, and heat-transfer relationships.

This is not necessarily equivalent to placing two independent two-stream exchangers in series. Heat-transfer coupling and intermediate thermal relationships may affect the mathematical behavior of the equipment.

Cross-flow exchanger models similarly address geometric arrangements not represented by the basic counter-current or co-current 1D templates.

Whether these specialized models are appropriate depends on the physical arrangement and the fidelity required.

Source — Power-generation unit-model library:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/power_generation/unit_models/index.html`

### 9.4 Solid-oxide-cell modeling hierarchy

IDAES's solid-oxide-cell models expose additional detail beyond a simplified electrolyzer efficiency or power-consumption relationship.

The relevant model family includes representations of cells and their physical substructures, with varying levels of detail for electrochemical behavior, heat transfer, material transport, and flow channels.

The architectural distinction is between a compact system-level conversion model and a detailed cell-level physical model.

Detailed electrochemical representations can capture internal conditions that are invisible to a simple algebraic efficiency model, but at a corresponding numerical and parameterization cost.

Availability of the model classes does not imply that electrochemical performance parameters or validation data are provided for arbitrary cell materials and operating regimes.

### 9.5 Feed with integrated flashing

`FeedFlash` combines a feed boundary with a phase-equilibrium calculation.

This can be useful when the boundary conditions are naturally expressed as an overall material state, but downstream equipment requires an equilibrium vapor/liquid state.

The model differs from an ordinary `Feed` because it incorporates phase-equilibrium behavior instead of merely exposing specified state variables.

Its validity remains dependent on the selected property package and phase-equilibrium formulation.

Source — Generic unit-model library:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/unit_models/index.html`

Release source — Pipe addition and subsequent model changes:

`https://github.com/IDAES/idaes-pse/releases`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 10. Initialization, scaling, and costing extensions

Several recent releases refined the mechanisms used to initialize, scale, and economically evaluate process models.

### 10.1 Block-triangular initialization without a final full-model solve

`BlockTriangularizationInitializer` includes a `skip_final_solve` configuration option introduced in version 2.10.

The initializer structurally decomposes the system, solves individual blocks or strongly coupled subsystems, and ordinarily may perform a final complete-model solve.

`skip_final_solve` permits omitting that final step.

This provides a useful distinction between generating a plausible initialized state through decomposition and confirming that the entire assembled system satisfies the final solver requirements.

Skipping the final solve can avoid redundant computation in some situations, but successful subsystem solves do not themselves guarantee complete-model convergence.

Source — Block Triangularization Initializer:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/initialization/bt_initializer.html`

### 10.2 Additional model-specific initializers

The framework has continued to extend its reusable initialization strategies.

Version 2.10 added material-holdup initialization support for `MSContactor`.

Version 2.12 introduced a dedicated solvent-reboiler initializer.

These additions matter because specialized dynamic and separation equipment may have coupled algebraic states, inventories, and equilibrium relationships that cannot be reliably initialized by a generic sequence alone.

A reusable initializer can encode the ordering and temporary specifications required by a particular model family, while remaining compatible with the broader `Initializer` framework.

### 10.3 Property and reaction default scalers

Version 2.12 added default scaler objects for physical and reaction property packages.

This extends the newer scaling architecture by allowing suitable package-specific scaling behavior to be associated with the underlying property or reaction implementation.

A model-specific scaler can use knowledge of the expected physical magnitudes or equation structure rather than relying exclusively on generic Jacobian-based heuristics.

This is significant because property equations frequently influence the conditioning of many equipment models sharing the same package.

Default scaling does not establish that every operating condition is well-scaled. Extreme compositions, phase transitions, unusual units, and poorly specified initial states may still require additional analysis.

### 10.4 ArcConstraintScaler

Version 2.13 added `ArcConstraintScaler`.

Its purpose is to scale equality constraints generated from stream connections, using the scaling information associated with connected variables.

This addresses a gap between scaling individual unit models and scaling the complete connected flowsheet.

Even when every unit is individually scaled, equality constraints connecting streams can introduce inconsistent numerical magnitudes.

`ArcConstraintScaler` provides a reusable mechanism for addressing that interface.

Its role is limited: scaling an arc constraint does not correct incorrectly scaled physical equations within the connected unit models.

Source — Arc Constraint Scaler:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/scaling/arc_constraint_scaler.html`

### 10.5 Location-factor costing

Version 2.13 added location-factor support to the costing infrastructure.

Location factors provide a means of adjusting supported capital-cost estimates for differences in geographic construction and installation conditions.

This extends the costing framework's treatment of equipment economics beyond equipment size and cost-index adjustments.

The key distinction is between cost escalation over time and cost adjustment across locations. These are separate influences and should not be conflated.

Location factors are empirical costing assumptions, not intrinsic properties of the equipment. Their appropriate use depends on the geographic basis, cost methodology, and data underlying the factors.

### 10.6 QGESS costing reorganization

Version 2.12 reorganized QGESS costing implementation.

This is principally an API and organizational change affecting the costing family, rather than evidence of an entirely new mathematical costing method.

Its importance is compatibility and discoverability: code written against earlier module organization may require examination when moving between versions.

Release sources:

`https://github.com/IDAES/idaes-pse/releases/tag/2.12.0`

`https://github.com/IDAES/idaes-pse/releases/tag/2.13.0`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



## 11. Model-interface distinctions and optional ecosystem boundaries

### 11.1 StateJunction versus Translator

The previous overview grouped these models together as stream-connection utilities. Their semantics require a sharper distinction.

`StateJunction` contains a single state block shared by an inlet port and an outlet port. It does not introduce an independent thermodynamic transformation or additional equipment equations.

It is useful as an explicit connection point in alternative flowsheet arrangements.

`Translator` instead exposes distinct inlet and outlet state representations, potentially associated with different property packages.

The translation relationships required to map between them must be defined for the particular application.

It is therefore not a generic automatic converter between arbitrary chemical models.

The distinction is important for preserving component balances, thermodynamic references, material-flow bases, and the meanings of corresponding state variables.

Sources:

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/unit_models/statejunction.html`

`https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/unit_models/translator.html`

### 11.2 Python compatibility versus optional model-stack compatibility

IDAES-PSE 2.13.0 added Python 3.14 support at the package level.

That does not establish that every optional integration supports Python 3.14.

Keras/TensorFlow, compiled thermodynamic extensions, external numerical solvers, and auxiliary optimization libraries have their own distribution and compatibility requirements.

The package's supported Python range and the availability of a complete application-specific dependency stack must therefore be evaluated separately.

ONNX/OMLT integration can sometimes reduce dependence on a particular neural-network training framework, but it does not eliminate the need for a compatible OMLT and optimization environment.

### 11.3 CoolProp version qualifications

The 2.13.0 release record contains two entries with apparently conflicting implications:

- A change restricting the CoolProp version to below 8.
- A change updating the implementation to support CoolProp 8.

Both appear in the same release notes.

This makes the effective packaged dependency constraint more authoritative than either isolated release-note description.

The correct interpretation is that CoolProp is an optional integration with version-sensitive compatibility. Support for a given CoolProp version should be verified against the actual 2.13.0 package metadata and the installed integration, rather than inferred solely from one change entry.

Source — Release 2.13.0:

`https://github.com/IDAES/idaes-pse/releases/tag/2.13.0`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 11.4 Companion-package boundaries

Several related projects expand the wider process-systems engineering ecosystem but should remain distinct from functionality shipped in `idaes-pse`.

| Related project | Relationship                                                                    |
| --------------- | ------------------------------------------------------------------------------- |
| Pecos           | Data-quality control, monitoring, and fault-detection functionality             |
| DISPATCHES      | Integrated energy-system models and electricity-market-oriented applications    |
| IDAES UI        | Companion visualization and user-interface tooling                              |
| PrOMMiS         | Domain-specific processing models and utilities in a separate project           |
| WaterTAP        | Water-treatment process models and optimization built on related infrastructure |

DISPATCHES, for example, supplies additional renewable generation, storage, and hydrogen-system models used in grid-integration applications. Those models should not automatically be described as native IDAES-PSE unit classes.

Pecos addresses data monitoring rather than the primary process-equation modeling abstractions.

The distinction is particularly important when describing dependencies: one project may provide a physical model, another an optimization interface, and a third the external simulation or solver runtime.

Sources — Related packages and UI:

`https://idaes-pse.readthedocs.io/en/2.13.0/explanations/related_packages/index.html`

`https://idaes-pse.readthedocs.io/en/2.13.0/explanations/components/ui/index.html`

`https://dispatches.readthedocs.io/en/main/models/renewables/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://idaes-pse.readthedocs.io\&sz=32)

IDAES v2.13.0+1



## 12. Capability synthesis

The additional mechanisms expand IDAES-PSE's architectural value in several ways that were not fully apparent from the initial reference.

| Combination                                                                    | Additional architectural capability                                                                                           |
| ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| Property interrogators + state-definition interfaces                           | Discover physical-property requirements and develop compatible packages without manually reconstructing every unit dependency |
| Electrolyte species representations + inherent reactions                       | Represent aqueous chemical equilibria while distinguishing process-facing components from internal ionic species              |
| Helmholtz expression writer + compiled property functions                      | Introduce selected thermodynamic relationships without constructing unnecessary process state blocks                          |
| Smooth mathematical utilities + scaling + expression diagnostics               | Build differentiable formulations and identify numerical problems at both expression and system levels                        |
| Initialization decomposition + convergence-regression analysis                 | Separate construction of a converged initial state from automated monitoring of numerical robustness                          |
| Design/operation/storage blocks + representative periods + economic objectives | Build integrated process-design and operational models across multiple time periods                                           |
| ONNX + OMLT + SurrogateBlock                                                   | Incorporate supported externally trained neural networks into constrained process optimization                                |
| Specialized distributed equipment + general flowsheet interfaces               | Combine detailed physical submodels with reusable process topology and conservation infrastructure                            |

The most consequential additions are the property-interrogation and electrolyte domain-modeling mechanisms, the deeper numerical diagnostics and solver-regression tools, and the multi-period operational-modeling abstractions.

These capabilities reinforce an important architectural distinction: IDAES is not exclusively a repository of equations and equipment models. It also supplies mechanisms for discovering modeling requirements, integrating heterogeneous physical representations, managing mathematical conditioning, and composing physical and economic decision models.

The boundaries remain consequential. Interrogation is not proof of property completeness; electrolyte metadata is not a complete aqueous thermodynamic engine; smoothing is not exact physical behavior; solver diagnostics do not guarantee convergence; and advanced process or market models remain dependent on their mathematical assumptions and external solvers.

Source and verification scope: This addendum is based on IDAES-PSE 2.13.0 documentation, tagged source where relevant, and upstream release history through November 1, 2026. It covers released capabilities and explicitly identifies selected limitations. No independent numerical execution or performance benchmarking was performed.