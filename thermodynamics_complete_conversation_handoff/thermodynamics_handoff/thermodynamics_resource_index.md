# Thermodynamics implementation resources

These are public upstream entry points and focused reading locations, checked against official project documentation during this handoff. Moving documentation is not a qualified release manifest. The original research's source pins and limitations remain in B2/B3, and the full-package validation gaps remain in B9. No engine is installed, benchmarked, or newly approved by this index.

## R-01. DWSIM: complete process workflow and configured-package reference

**Project:** <https://dwsim.org/>  
**Source:** <https://github.com/DanWBR/dwsim10>  
**Earlier source tree:** <https://github.com/DanWBR/dwsim>  
**Method selection:** <https://dwsim.org/tutorials/en/reference/property-packages-guide.html>

The `dwsim10` tree separates simulator code under `engine/`, UI under `ui/`, and tests under `tests/`. Start with thermodynamics, material streams, property-package configuration, flash dispatch, unit operations, and the flowsheet solver. B2 records exact inspected paths and its commit. Do not transpose old repository paths onto a different release without checking.

Use DWSIM for complete workflow coverage, sample flowsheets, and a candidate whole-package adapter. Its official project and repository identify GPLv3 licensing. Check the particular edition, optional integrations, data, and distribution terms. An external worker is a technical boundary, not an automatic licensing exemption. The property guide distinguishes core and paid features.

## R-02. ChEDL `thermo`: assembled conventional property packages

**Source:** <https://github.com/CalebBell/thermo>  
**Documentation:** <https://thermo.readthedocs.io/>  
**Data/model separation:** <https://thermo.readthedocs.io/chemical_package_tutorial.html>  
**Phase interface:** <https://thermo.readthedocs.io/thermo.phases.html>  
**Flash families:** <https://thermo.readthedocs.io/thermo.flash.html>  
**Inspectable VLN source:** <https://thermo.readthedocs.io/_modules/thermo/flash/flash_vln.html>

Start with `ChemicalConstantsPackage`, `PropertyCorrelationsPackage`, phase objects, `FlashVL`, `FlashVLN`, `FlashPureVLS`, bulk-property settings, and structured equilibrium results. The useful boundary is explicit phase/caloric/data/flash assembly. The documented separation of data from algorithms supports frozen, reproducible configurations. A many-liquid interface is not unrestricted multicomponent-solid capability. B3 records the bounded source-level limitations.

## R-03. ChEDL `chemicals`: data, correlations, and numerical building blocks

**Source:** <https://github.com/CalebBell/chemicals>  
**Documentation:** <https://chemicals.readthedocs.io/>  
**Allocation utilities:** <https://chemicals.readthedocs.io/chemicals.rachford_rice.html>

Use this separately from `thermo`: it supplies property data, correlations, and numerical utilities. A Rachford–Rice routine operating on supplied K-values is an inner allocation calculation, not a complete EOS, stability solver, or energy-complete flash. Preserve source and data-use terms along with correlation definitions. B9 exercised one extracted residual function, not the installed package.

## R-04. Clapeyron.jl: model composition and an alternative coherent engine

**Source:** <https://github.com/ClapeyronThermo/Clapeyron.jl>  
**Documentation:** <https://clapeyronthermo.github.io/Clapeyron.jl/stable/>  
**Composite methods:** <https://clapeyronthermo.github.io/Clapeyron.jl/stable/eos/misc/>  
**Flash compatibility:** <https://clapeyronthermo.github.io/Clapeyron.jl/stable/properties/flash/>

Inspect `CompositeModel`, ideal contributions, activity/EOS composition, parameter sources, reference conventions, and the exact flash method matrix. Model availability and algorithm/initializer coverage are separate. The project is Julia-based and the documentation also identifies `pyclapeyron`; verify a chosen binding against the actual operation rather than assuming interface parity. Its official documentation states MIT licensing.

## R-05. CoolProp: specialized fluid and utility calculations

**Source:** <https://github.com/CoolProp/CoolProp>  
**Documentation:** <https://coolprop.org/>  
**Reusable backend state:** <https://coolprop.org/coolprop/LowLevelAPI.html>  
**Input/derivative conventions:** <https://coolprop.org/coolprop/HighLevelAPI.html>

Start with a selected backend and `AbstractState`, not a generic claim about everything behind the common interface. Qualify saturation/quality, reference-state lifetime, mixture input pairs, and derivative semantics for that backend/build. A CoolProp interface to a separately obtained external engine does not provide that engine or its data. B3's source observations and any installed release must be compared explicitly.

## R-06. Reaktoro: coherent chemical-system calculations

**Source:** <https://github.com/reaktoro/reaktoro>  
**Documentation:** <https://reaktoro.org/>  
**Source/build dependencies:** <https://reaktoro.org/installation/installation-using-cmake.html>

The official project describes a C++/Python framework for equilibrium and kinetic chemically reactive processes, including aqueous speciation and mineral dissolution/precipitation. Inspect chemical-system, state, property, specification/condition/restriction, and sensitivity interfaces. Choose database, species/phase set, activity models, and caloric conventions explicitly. Do not substitute a narrow DWSIM bridge's coverage for the upstream interface or assume a full absorber/crystallizer unit comes with a chemistry calculation.

## R-07. ThermoPack: focused EOS and state/property engine

**Source:** <https://github.com/thermotools/thermopack>  
**Documentation:** <https://thermotools.github.io/thermopack/>  
**Core interface:** <https://thermotools.github.io/thermopack/vcurrent/thermo_methods.html>  
**Cubic configuration:** <https://thermotools.github.io/thermopack/vcurrent/cubic_methods.html>

The official project describes a Fortran core, C/C++ access, a Python wrapper, and Apache 2.0 licensing. Use version-matched documentation. Qualify actual exposed flashes, parameter construction, total calorics, and session activation; `vcurrent` and a selected binary are not interchangeable evidence. B3 records the runtime-pseudocomponent/caloric distinction and the limits of its source-level session assessment.

## R-08. FeOS: potential-based thermodynamics and differentiated states

**Source:** <https://github.com/feos-org/feos>  
**Documentation:** <https://feos-org.github.io/feos/>  
**Framework paper:** <https://doi.org/10.1021/acs.iecr.2c04561>

Inspect ideal/residual model composition, parameter records, local states, equilibrium, and derivative interfaces. Rust implementations and Python bindings are useful, but native language does not establish general process scope. The paper and project describe a scientifically substantial framework; use it for suitable model families rather than treating its current surface as the universal material model. Parameter formats and inverse preconditions require version-specific checking.

## R-09. IDAES and CAPE-OPEN: process and contract precedents

**IDAES source:** <https://github.com/IDAES/idaes-pse>  
**Property architecture:** <https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html>  
**Modular methods:** <https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/general/index.html>  
**CAPE-OPEN specifications:** <https://www.colan.org/specifications/>

IDAES separates shared parameter definitions from local property states and physical properties from reaction properties. Study state construction, phase/component eligibility, unit coupling, initialization/scaling, and translator constraints. Use the abstraction, not a requirement to port every Pyomo object. CAPE-OPEN is a useful reference for material/property/equilibrium authority and interoperability; it supplies neither a model database nor automatic conservative translation. B3 identifies the exact thermodynamics and separate reaction documents it consulted.

## R-10. Additional handoff pointer: `reaktoro-pse`

**Source and project documentation:** <https://github.com/watertap-org/reaktoro-pse>

The maintainers describe an existing Reaktoro gray-box integration for Pyomo/IDAES/WaterTAP. It does not replace Reaktoro or automatically select appropriate databases and activity models. This is a newly noted implementation reference for constrained-chemistry block integration, not a dependency approved or tested by the nine-stage study.

## R-11. Solver contracts, not mandatory solver selection

**Ipopt interface:** <https://coin-or.github.io/Ipopt/INTERFACES.html>  
**Ipopt derivative/approximation features:** <https://coin-or.github.io/Ipopt/SPECIALS.html>

B8 records these sources and versioned Pyomo reduced/gray-box interfaces as formulation references. Inspect actual Jacobian/Hessian, coordinate, sparsity, scaling, and differentiability obligations of whichever solver is selected. These links are not a decision to require Ipopt or Pyomo.

## How to use the supplied executable material

Open [the original Step-9 README](expanded/09_validation/README.md) and work in a copy of that extracted directory. Its `scripts/` directory contains source-kernel/formulation probes, a bounded publication checker, an optional full-package runner, and an artifact validator. `fixtures.json`, `source_manifest.json`, and `results/` retain assumptions, attribution, observed outcomes, and dependency blockers.

The Step-7/8 reference and structural scripts are also preserved in that package's `baselines/` and in their own extracted bundles. Historical result files are evidence, not outputs recomputed during this packaging task. Inspect script version requirements and prerequisites before use; package-dependent worker branches were blocked in the recorded Step-9 environment.

Do not use a moving documentation page, the existence of a binding, or a small extracted function as a production manifest. For each enabled configuration, capture the exact code/build/binding/data and the selected methods, conventions, operation, phase/chemistry policy, and numerical arrangement. Data and optional dependencies require their own availability and terms review.
