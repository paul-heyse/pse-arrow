Yes. Combining the earlier six-gap resource set with the broader thermodynamics atlas, and excluding IDAES-PSE and FeOS as requested, I would build the corpus below.

I would classify access this way:

**OSS-P** = permissive open source, generally easiest to inspect, translate, or incorporate; **OSS-C** = copyleft open source, inspectable but redistribution/integration implications matter; **PUBLIC** = freely accessible specification/data, but do not assume unrestricted redistribution; **FREE-R** = free with registration, scope, or noncommercial restrictions; **COMM-API** = commercial, but explicitly offers an SDK/API/property-package/redistribution path; **COMM** = commercial closed data/software.

For your immediate goal of **learning the full thermodynamic domain model and lowering it into your common mathematical representation**, OSS-P, OSS-C and PUBLIC resources are the most valuable because you can actually inspect how they encode the science. Commercial systems are still extremely useful as **coverage references**.

## 1. General fluid thermodynamics, EOS, activity models and equilibrium

| Resource | Access | What you get / why I would pull it |
|---|---|---|
| **Clapeyron.jl** [Project/docs](https://clapeyronthermo.github.io/Clapeyron.jl/stable/?utm_source=chatgpt.com) | **OSS-P, MIT** | Probably the broadest open *thermodynamic-model taxonomy* to ingest. Cubics, alpha functions, volume translations, mixing rules, SAFT/PC-SAFT/CPA, activity models, multiparameter models, flash/stability methods, corresponding states, electrolyte-related work, etc. Excellent for decomposing models into reusable contributions rather than monolithic packages. :chatgpt-content-reference{index="1"} |
| **ThermoPack** [GitHub](https://github.com/thermotools/thermopack?utm_source=chatgpt.com) | **OSS-P, Apache-2.0** | Mature compiled Fortran thermodynamics with C/C++/Python interfaces: cubics, CPA, PC-SAFT, SAFT-VR Mie, multiparameter EOS, flashes, critical points and envelopes. Its technical memos are unusually valuable for bridging equations and numerical implementations. :chatgpt-content-reference{index="3"} |
| **CoolProp** [Project](https://coolprop.org/?utm_source=chatgpt.com) | **OSS-P, MIT** | High-quality multiparameter Helmholtz EOS, mixture models, transport, ancillaries and fluid JSON definitions. Especially valuable for understanding declarative EOS-term representation and property derivation. :chatgpt-content-reference{index="5"} |
| **teqp** [GitHub](https://github.com/usnistgov/teqp?utm_source=chatgpt.com) | **OSS-P / NIST open source** | Very strong source for multifluid Helmholtz EOS, GERG-like models, cubics, PC-SAFT, CPA, critical calculations and high-order derivatives. Particularly attractive for your IR because model terms and reducing functions are relatively exposed. :chatgpt-content-reference{index="7"} |
| **thermo** [GitHub](https://github.com/CalebBell/thermo?utm_source=chatgpt.com) | **OSS-P, MIT** | Huge practical collection of EOS, excess-Gibbs/activity models, flash algorithms, parameter handling and phase/property objects. Excellent implementation reference for NRTL, Wilson, UNIQUAC, UNIFAC families and generic flash architecture. :chatgpt-content-reference{index="9"} |
| **chemicals** [GitHub](https://github.com/CalebBell/chemicals?utm_source=chatgpt.com) | **OSS-P, MIT** | Large library of explicit physical-property correlations and equation forms. Particularly good source for building a typed `CorrelationFamily → Parameters → ValidityDomain` layer. :chatgpt-content-reference{index="11"} |
| **PhasePy** [GitHub](https://github.com/gustavochm/phasepy?utm_source=chatgpt.com) | **OSS-P, MIT** | Cubic EOS, activity models, MHV/Wong-Sandler mixing, VLE/LLE/VLLE and square-gradient interfacial thermodynamics. Small enough to understand end-to-end. :chatgpt-content-reference{index="13"} |
| **NeqSim** [GitHub](https://github.com/equinor/neqsim?utm_source=chatgpt.com) | **OSS-P, Apache-2.0** | Exceptionally broad petroleum/process thermodynamics: dozens of EOS variants, CPA, GERG, PVT, phase envelopes, hydrates, petroleum characterization, transport and process calculations. Very useful as a coverage benchmark. :chatgpt-content-reference{index="15"} |
| **DWSIM Thermodynamics** [GitHub](https://github.com/DanWBR/dwsim?utm_source=chatgpt.com) | **OSS-C, GPLv3** | One of the best open references for how a full process simulator organizes property packages, petroleum characterization, UNIFAC variants, cubics, PC-SAFT, GERG and flash infrastructure. Excellent to inspect; GPL implications matter if code is copied into a differently licensed product. :chatgpt-content-reference{index="17"} |
| **NIST AGA8 / GERG code** [GitHub](https://github.com/usnistgov/AGA8?utm_source=chatgpt.com) | **PUBLIC / NIST source** | Reference implementations for AGA8 and GERG-2008. Very valuable for pinning exact gas-mixture formulations and validation cases. :chatgpt-content-reference{index="19"} |

For the six initial gaps, this first group already gives you an extraordinary amount of the **actual mathematical ontology**: potential-based models, contribution models, mixing rules, group contributions, association, parameter matrices, stability, implicit equations, flashes and critical calculations.

## 2. Reactive fluids, electrolytes, minerals and chemical equilibrium

| Resource | Access | Why it matters |
|---|---|---|
| **Reaktoro** [Project](https://reaktoro.org/?utm_source=chatgpt.com) | **OSS-C, LGPL-2.1+** | One of the most important additions. Species, elements, phases, reactions, activities, standard states, chemical potentials, constrained equilibrium/minimization, kinetics and interfaces to multiple thermochemical databases. A fundamentally broader chemical-system ontology than a conventional EOS package. :chatgpt-content-reference{index="21"} |
| **ThermoFun** [Project](https://thermohub.org/thermofun/thermofun/?utm_source=chatgpt.com) | **OSS-C, LGPL-2.1** | Standard-state thermodynamic properties of substances and reactions as functions of \(T,P\), JSON datasets and derivatives. Excellent source for separating standard-state models from mixture models. :chatgpt-content-reference{index="23"} |
| **ThermoHub** [Portal](https://thermohub.org/?utm_source=chatgpt.com) | **Mixed/open portal; dataset-specific terms** | Thermochemical datasets supporting ThermoFun/Reaktoro-style workflows. Useful as a schema and data-provenance corpus; check individual dataset licenses before redistribution. |
| **PHREEQC** [USGS download](https://www.usgs.gov/software/phreeqc-version-3?utm_source=chatgpt.com) | **PUBLIC / USGS, effectively public-domain/CC0 government code/data** | Huge conceptual value: aqueous species, master species, reaction definitions, log-K functions, minerals, gases, surface complexation, ion exchange, redox, SIT and Pitzer. Also comes with multiple databases. :chatgpt-content-reference{index="26"} |
| **GEMS3K / xGEMS** [GitHub](https://github.com/gemshub/GEMS3K?utm_source=chatgpt.com) | **OSS-C, LGPL-3+** | General Gibbs-energy minimization and one of the richest solution-model collections: Pitzer, SIT, extended UNIQUAC, Debye-Hückel variants, NRTL/Wilson, cubic fluids and complex solid solutions. :chatgpt-content-reference{index="28"} |
| **OLI Engine / OLI databases** [OLI Cloud APIs](https://olisystems.com/software/oli-cloud-apis/?utm_source=chatgpt.com) | **COMM-API** | One of the major commercial electrolyte references. OLI exposes REST APIs around its Engine, process, chemistry, corrosion and scaling calculations. Its database claims 6,000+ species and broad multicomponent electrolyte coverage. You can integrate calculations, but the model/database contents themselves are proprietary. :chatgpt-content-reference{index="30"} |

**OLI is worth adding even though it is commercial.** It gives you an industrial benchmark for what “complete electrolyte thermodynamics” means beyond eNRTL alone.

## 3. CALPHAD, solids, molten phases and high-temperature thermochemistry

| Resource | Access | Why it matters |
|---|---|---|
| **pycalphad** [GitHub](https://github.com/pycalphad/pycalphad?utm_source=chatgpt.com) | **OSS-P, MIT** | Essential architecture reference for sublattices, constituents, site fractions, endmembers, vacancies, piecewise Gibbs functions, interaction parameters and multiphase equilibrium. :chatgpt-content-reference{index="32"} |
| **ESPEI** [Project](https://espei.org/?utm_source=chatgpt.com) | **OSS-P, MIT** | Complements pycalphad with parameter inference/fitting and uncertainty. Particularly useful for your `Evidence → Parameterization → Model` representation. :chatgpt-content-reference{index="34"} |
| **OpenCalphad** [GitHub](https://github.com/sundmanbo/opencalphad?utm_source=chatgpt.com) | **OSS-C, GPLv3** | Independent implementation of CALPHAD/TDB thermodynamics and equilibrium. Good second executable reference. :chatgpt-content-reference{index="36"} |
| **Thermochimica** [GitHub](https://github.com/ORNL-CEES/thermochimica?utm_source=chatgpt.com) | **OSS-P, BSD-3-Clause** | Thermochemical Gibbs minimization designed explicitly as an embeddable numerical library for multiphysics applications. :chatgpt-content-reference{index="38"} |
| **SGTE PURE5** [Thermo-Calc resources](https://thermocalc.com/support/resources/?utm_source=chatgpt.com) | **FREE/PUBLIC download** | Free unary/reference-element CALPHAD database. Very useful for understanding reference-state and piecewise Gibbs-function representation. :chatgpt-content-reference{index="40"} |
| **Thermo-Calc Educational Package** [Thermo-Calc documentation](https://thermocalc.com/support/documentation/?utm_source=chatgpt.com) | **FREE-R** | Free package requiring no commercial license, but intentionally limited chemistry/databases. Useful for inspecting CALPHAD workflows and formats. :chatgpt-content-reference{index="42"} |
| **Thermo-Calc full + TC-Python** [TC-Python documentation](https://thermocalc.com/support/documentation/tc-python-help/?utm_source=chatgpt.com) | **COMM-API** | Full commercial CALPHAD/database ecosystem with Python SDK and very broad materials databases. TC-Python itself requires a license. :chatgpt-content-reference{index="44"} |
| **FactSage** [Product site](https://factsage.com/?utm_source=chatgpt.com) | **COMM** | One of the key industrial high-temperature thermochemistry platforms: slags, metals, oxides, molten salts, minerals, gases and solution phases. |
| **ChemApp Light** [Download/license](https://gtt-technologies.de/download/chemapp-light-for-microsoft-visual-c/?utm_source=chatgpt.com) | **FREE-R** | Free restricted equilibrium library. Private/noncommercial and evaluation uses are permitted subject to detailed restrictions; it is not a general free commercial runtime. :chatgpt-content-reference{index="47"} |
| **ChemApp regular** [Documentation](https://python.gtt-technologies.de/doc/chemapp/introduction/main.html?utm_source=chatgpt.com) | **COMM-API** | Specifically designed as an embeddable equilibrium library. Development and Redistributable editions exist, making this much more relevant to your architecture than FactSage's GUI alone. :chatgpt-content-reference{index="49"} |

ChemApp is especially relevant because its commercial model explicitly recognizes the distinction between **developer library** and **redistributable runtime**, which is exactly the question you will face with external proprietary property engines.

## 4. Reaction thermochemistry, gas-phase equilibrium and combustion

| Resource | Access | Why pull it |
|---|---|---|
| **Cantera** [Project](https://cantera.org/?utm_source=chatgpt.com) | **OSS-P, BSD-3-Clause** | Species, phases, NASA thermochemistry, reaction stoichiometry, kinetics, transport and equilibrium behind a clean YAML-oriented data model. :chatgpt-content-reference{index="51"} |
| **NASA CEA** [GitHub](https://github.com/nasa/CEA?utm_source=chatgpt.com) | **OSS-P, Apache-2.0 in current NASA implementation** | Chemical equilibrium with thousands of gaseous/condensed species and multiple state specifications. Very useful free-energy-minimization reference. :chatgpt-content-reference{index="53"} |
| **RMG-Py / Arkane** [RMG GitHub](https://github.com/ReactionMechanismGenerator/RMG-Py?utm_source=chatgpt.com) | **OSS-P, MIT** | Especially valuable for automatically constructing thermochemistry: group-additivity estimation, quantum-chemistry processing, NASA polynomial fitting and reaction thermodynamics. :chatgpt-content-reference{index="55"} |
| **RMG Database** [GitHub](https://github.com/ReactionMechanismGenerator/RMG-database?utm_source=chatgpt.com) | **Open data/source ecosystem** | Group-additivity trees, thermochemical libraries and kinetics knowledge. Architecturally interesting because parameters live in hierarchical chemical-structure rules rather than simply by component ID. |

## 5. High-accuracy fluid standards and reference EOS

| Resource | Access | Notes |
|---|---|---|
| **IAPWS-95 / IAPWS-IF97 releases** [IAPWS releases](https://iapws.org/relguide/?utm_source=chatgpt.com) | **PUBLIC specifications** | The authoritative formulations themselves are freely downloadable. IAPWS does not principally distribute one canonical OSS implementation, so implementation licensing varies. Extremely useful for regional/piecewise/backward-equation modeling. :chatgpt-content-reference{index="58"} |
| **REFPROP** [NIST REFPROP](https://www.nist.gov/srd/refprop?utm_source=chatgpt.com) | **COMM-API / DATA-LICENSE** | Current NIST page lists a paid single-user product, site licenses and explicit **Distributor Agreements** for integrating REFPROP into third-party products. This is the important distinction: proprietary, but there is a formal product-integration route. :chatgpt-content-reference{index="60"} |
| **NIST AGA8/GERG** [GitHub](https://github.com/usnistgov/AGA8?utm_source=chatgpt.com) | **PUBLIC source** | Keep separately from REFPROP because you can directly inspect the natural-gas equations/reference implementation without purchasing REFPROP. |

For your project, I would use **CoolProp/teqp/NIST AGA8 as extractable scientific sources**, while keeping REFPROP as a high-authority licensed validation/runtime option.

## 6. Molecularly predictive thermodynamics and COSMO-type models

| Resource | Access | Notes |
|---|---|---|
| **NIST COSMO-SAC** [GitHub](https://github.com/usnistgov/COSMOSAC?utm_source=chatgpt.com) | **Mixed** | Implementation code is MIT, but some associated sigma-profile datasets carry more restrictive academic/noncommercial conditions. Treat source-code licensing and descriptor-data licensing separately. :chatgpt-content-reference{index="63"} |
| **Open PySCF COSMO-SAC sigma profiles** [GitHub](https://github.com/Danny-Taehyun-Kim/PySCF-Sigmaprofile-COSMO-SAC?utm_source=chatgpt.com) | **OSS/data-open: MIT code, CC-BY-4.0 dataset** | Large generated sigma-profile corpus. Very useful for designing molecule-descriptor-backed thermodynamics; because it is a newer research resource, I would independently validate scientific coverage before relying on it. :chatgpt-content-reference{index="65"} |
| **Published UNIFAC parameters at DDBST** [Published parameters](https://www.ddbst.com/published-parameters-unifac.html?utm_source=chatgpt.com) | **PUBLIC published subset** | Publicly accessible parameter tables and references. Useful baseline for original/modified UNIFAC. :chatgpt-content-reference{index="67"} |
| **UNIFAC Consortium** [Consortium](https://www.ddbst.com/unifac-consortium.html?utm_source=chatgpt.com) | **COMM / consortium membership** | Expanded current parameter tables are restricted to consortium members; membership is fee-based and parameters cannot simply be passed to third parties. Use as a commercial data option, not as corpus material you can redistribute. :chatgpt-content-reference{index="69"} |

## 7. Petroleum characterization, PVT and flow assurance

| Resource | Access | Notes |
|---|---|---|
| **DWSIM petroleum characterization** [Source tree](https://github.com/DanWBR/dwsim/tree/windows/DWSIM.Thermodynamics/PetroleumCharacterization?utm_source=chatgpt.com) | **OSS-C, GPLv3** | TBP/distillation curves, pseudo-component generation, Riazi/Lee-Kesler-style correlations, fitting and quality checks. Excellent source model, licensing matters for direct reuse. |
| **NeqSim characterization/PVT** [GitHub](https://github.com/equinor/neqsim?utm_source=chatgpt.com) | **OSS-P, Apache-2.0** | TBP fractions, plus fractions, lumping, fluid characterization, petroleum PVT and specialized oil/gas models. |
| **CSMGem** [CSM Hydrates software](https://hydrates.mines.edu/software/?utm_source=chatgpt.com) | **LICENSE-BY-REQUEST** | Specialized gas-hydrate equilibrium/cage-occupancy software. Colorado School of Mines directs users to contact them regarding licensing/use. :chatgpt-content-reference{index="73"} |
| **PVTsim Nova** [Calsep](https://calsep.com/?utm_source=chatgpt.com) | **COMM** | Major petroleum PVT/characterization/flow-assurance reference including hydrates, wax, asphaltenes and salts. |
| **Calsep Flash API** [Flash API](https://calsep.com/calsep-cloud/flash-api/?utm_source=chatgpt.com) | **COMM-API** | Licensed HTTP API plus Python/C# SDK. Uses characterized PVTsim fluids exported in JSON, which is particularly interesting for your external-backend abstraction. :chatgpt-content-reference{index="76"} |
| **Multiflash** [KBC Multiflash](https://www.kbc.global/process-optimization/technology/simulation-software/multiflash-simulation-software/?utm_source=chatgpt.com) | **COMM-API** | Major commercial multiphase property engine, especially strong in oil/gas and flow assurance; supports external integration/CAPE-OPEN and programmatic uses. |

## 8. Polymers and statistical-mechanical fluids

| Resource | Access | Notes |
|---|---|---|
| **PolyKin** [GitHub](https://github.com/HugoMVale/polykin?utm_source=chatgpt.com) | **OSS-P, MIT** | Flory-Huggins, Poly-NRTL, Scatchard-Hildebrand, Sanchez-Lacombe and molecular-weight-distribution utilities. :chatgpt-content-reference{index="79"} |
| **flory** [GitHub](https://github.com/qiangyicheng/flory?utm_source=chatgpt.com) | **OSS-P, MIT** | Clean implementation of free-energy-density-based phase coexistence using volume fractions, size ratios and interaction matrices. Valuable for composition-coordinate typing. :chatgpt-content-reference{index="81"} |
| **NIST pyPRISM** [GitHub](https://github.com/usnistgov/pyPRISM?utm_source=chatgpt.com) | **OSS-P / permissive NIST license** | Polymer Reference Interaction Site Model calculations, pair correlations and structure factors. Pushes your ontology beyond ordinary bulk free-energy models into liquid-state statistical mechanics. :chatgpt-content-reference{index="83"} |

## 9. Interfaces, adsorption and spatial thermodynamics

| Resource | Access | Notes |
|---|---|---|
| **SurfPack** [Package page](https://pypi.org/project/surfpack/?utm_source=chatgpt.com) | **OSS-P, MIT** | ThermoTools classical DFT implementation for surfaces/interfaces. Introduces Helmholtz **functionals**, density fields, weighted densities and convolutions. :chatgpt-content-reference{index="85"} |
| **PhasePy SGT** [GitHub](https://github.com/gustavochm/phasepy?utm_source=chatgpt.com) | **OSS-P, MIT** | Square-gradient interfacial theory, useful as a simpler continuum-interface representation alongside full DFT. |
| **KineticGas** [GitHub](https://github.com/thermotools/KineticGas?utm_source=chatgpt.com) | **OSS-P, MIT** | Kinetic-theory transport calculations, collision integrals, diffusion matrices, thermal diffusion, viscosity and conductivity. Good source for separating equilibrium thermo from transport constitutive models. :chatgpt-content-reference{index="88"} |
| **pyGAPS** [GitHub](https://github.com/pauliacomi/pyGAPS?utm_source=chatgpt.com) | **OSS-P, MIT** | Adsorption isotherm models, enthalpy methods, IAST and multicomponent adsorption. :chatgpt-content-reference{index="90"} |
| **NIST ISODB / Adsorbent Registry** [Database](https://adsorption.nist.gov/?utm_source=chatgpt.com) | **PUBLIC database/API** | Experimental adsorption data downloadable/accessible through JSON/XML/CSV interfaces. Useful for adsorption evidence and validation schema. :chatgpt-content-reference{index="92"} |

## 10. Experimental and evaluated thermodynamic databases

This category deserves special treatment because **accessing values is different from being allowed to redistribute those values in your own product**.

| Resource | Access | What I would use it for |
|---|---|---|
| **NIST ThermoML Archive** [Archive](https://www.nist.gov/mml/acmd/trc/thermoml/thermoml-archive?utm_source=chatgpt.com) | **PUBLIC download** | One of the best sources you can ingest immediately. Experimental thermo/transport data in standardized XML, now also distributed as JSON archives. NIST explicitly describes the archive data as public. Includes uncertainties, conditions, methods and bibliographic provenance. :chatgpt-content-reference{index="94"} |
| **ThermoML schema** [Standard overview](https://www.nist.gov/mml/acmd/trc/thermoml?utm_source=chatgpt.com) | **PUBLIC standard** | I would ingest the schema even before the data. It is a first-class model for `Measurement → Property → Conditions → Uncertainty → Method → Citation`. :chatgpt-content-reference{index="96"} |
| **NIST Chemistry WebBook** [WebBook](https://webbook.nist.gov/chemistry/?utm_source=chatgpt.com) | **PUBLIC web access** | Thermochemistry, phase-change properties, spectra and references. Excellent validation/provenance source, though bulk redistribution should follow NIST terms rather than being assumed from free web access. :chatgpt-content-reference{index="98"} |
| **NIST-JANAF Tables** [JANAF](https://janaf.nist.gov/?utm_source=chatgpt.com) | **PUBLIC web/PDF** | High-quality standard-state thermochemical functions for roughly 1,700 substances. Excellent for species-property/reference-state testing. :chatgpt-content-reference{index="100"} |
| **Active Thermochemical Tables (ATcT)** [ATcT](https://atct.anl.gov/?utm_source=chatgpt.com) | **PUBLIC web** | Network-evaluated gas-phase thermochemistry with uncertainty and provenance. Architecturally useful because thermochemical values arise from a coupled evidence network rather than isolated measurements. :chatgpt-content-reference{index="102"} |
| **DIPPR 801 sample/student DB** [Register](https://dippr.aiche.org/Account/Register?utm_source=chatgpt.com) | **FREE-R** | Free one-year sample/student access after registration, explicitly noncommercial/test/educational in scope. Useful for examining database organization. :chatgpt-content-reference{index="104"} |
| **DIPPR 801 full** [Pricing/licensing](https://www.aiche.org/dippr/dippr-pricing-ordering?utm_source=chatgpt.com) | **COMM / DATA-LICENSE** | Evaluated pure-component properties and correlations. Annual licenses are published; AIChE also explicitly offers **secondary-distribution licenses**, making it possible to negotiate incorporation into downstream software. :chatgpt-content-reference{index="106"} |
| **Dortmund Data Bank (DDB)** [DDBST](https://www.ddbst.com/ddb.html?utm_source=chatgpt.com) | **COMM / DATA-LICENSE** | Massive mixture/property corpus: VLE/LLE/SLE, excess data, electrolytes, polymers, transport, hydrates, adsorption, etc. Public UNIFAC subsets exist, but the main databank is commercially licensed. |
| **DETHERM** [DETHERM online](https://detherm.cds.dechema.de/?utm_source=chatgpt.com) | **COMM / restrictive data license** | Aggregates DDB plus additional thermophysical collections including electrolyte, pure-component, transport and phase-equilibrium data. DECHEMA explicitly states contents are protected and distributed under licenses restricting copying and distribution. :chatgpt-content-reference{index="109"} |
| **NIST ThermoData Engine 103b** [TDE 103b](https://www.nist.gov/mml/acmd/trc/thermodata-engine/srd-nist-tde-103b?utm_source=chatgpt.com) | **COMM** | Critically evaluated pure, mixture and reaction data built atop NIST's SOURCE archive. Current NIST listing shows a substantial commercial license price, so this is more appropriate as an evaluation/parameter-development environment than something to casually bundle. :chatgpt-content-reference{index="111"} |
| **OECD/NEA Thermochemical Database** [TDB project](https://www.oecd-nea.org/jcms/pl_37223/electronic-database-of-the-tdb-project?utm_source=chatgpt.com) | **FREE-R / registration conditions** | High-quality critically reviewed formation/reaction/SIT data, particularly useful for geochemistry. Selected electronic data access involves registration/eligibility conditions; reports are openly published. :chatgpt-content-reference{index="113"} |
| **Cemdata18 / CemGEMS data** [Cemdata overview](https://cemgems.org/cemdata/about-cemdata/?utm_source=chatgpt.com) | **Generally free/research distribution; verify individual dataset terms** | Specialized cement/mineral thermodynamics, but an excellent stress test for aqueous + mineral + solid-solution representations. |

## 11. Interoperability standards and commercial property servers

These are particularly useful to your **domain-model/type-system work** because they reveal what mature systems need to exchange with a process simulator.

| Resource | Access | Notes |
|---|---|---|
| **CAPE-OPEN Thermodynamics specifications** [Specifications](https://www.colan.org/specifications/?utm_source=chatgpt.com) | **PUBLIC specification** | Material objects, phases, compounds, property packages, property calculations and equilibrium calls. I would map your internal model against CAPE-OPEN explicitly, even if your representation is richer. |
| **Simulis Thermodynamics** [CAPE-OPEN description](https://www.colan.org/process-modeling-component/simulis-thermodynamics-2/?utm_source=chatgpt.com) | **COMM-API / property server** | Commercial thermodynamic server with CAPE-OPEN Property Packages, activity/EOS/specialized models and DIPPR integration. Useful example of a property system explicitly designed for embedding. :chatgpt-content-reference{index="117"} |
| **Aspen Properties** [AspenTech product page](https://www.aspentech.com/en/products/engineering/aspen-properties?utm_source=chatgpt.com) | **COMM** with integration surfaces | One of the key industrial coverage benchmarks: Aspen advertises ~37,000 components, 127 property packages and 5M+ data points/interaction parameters. It exposes property calculations to Excel and Aspen has CAPE-OPEN support, but this is a proprietary property/database ecosystem rather than inspectable source. :chatgpt-content-reference{index="119"} |
| **gPROMS Properties** [Siemens gPROMS Process](https://www.siemens.com/en-us/products/gproms/process/?utm_source=chatgpt.com) | **COMM** | Particularly useful as a capability benchmark because its unified package spans ordinary fluids, solids, reactive systems, associating compounds, polymers and electrolytes and includes SAFT-γ Mie. :chatgpt-content-reference{index="121"} |

These commercial products are less useful for **equation extraction**, but very useful for asking:

> “Is there a thermodynamic system type or workflow our ontology still cannot represent?”

---

# How I would actually assemble the corpus

I would not treat all of these equally. For your specific purpose, I see **four acquisition tiers**.

| Tier | Resources | What to do with them |
|---|---|---|
| **Tier A — ingest source + models now** | Clapeyron, ThermoPack, CoolProp, teqp, thermo, chemicals, PhasePy, NeqSim, Reaktoro, ThermoFun, PHREEQC, GEMS3K, pycalphad, ESPEI, Thermochimica, Cantera, NASA CEA, RMG, KineticGas, SurfPack, PolyKin, flory, pyPRISM, pyGAPS | Clone repositories; inventory model families, schemas, equation forms, parameter records, phase/component types and algorithms. These should drive your domain model. |
| **Tier B — ingest open standards/data** | ThermoML, NIST AGA8/GERG, IAPWS, NIST-JANAF, ATcT, WebBook, NIST ISODB, SGTE PURE5, public UNIFAC parameters, open sigma profiles | Build parsers and normalize them into your provenance/evidence/parameter ontology. |
| **Tier C — obtain restricted/free evaluation access** | DIPPR sample, Thermo-Calc Educational, ChemApp Light, OECD/NEA TDB, CSMGem where available | Use to validate that your representation covers their semantics; do not assume redistribution rights. |
| **Tier D — commercial reference/integration** | REFPROP, DDB, DETHERM, full DIPPR, TDE, Thermo-Calc/TC-Python, FactSage/ChemApp, OLI, PVTsim/Flash API, Multiflash, Simulis, Aspen Properties, gPROMS Properties | Treat primarily as high-authority validation/data/runtime systems. Negotiate data redistribution or runtime integration only where it materially expands production coverage. |

The important point is that **Tier A + Tier B alone already gives you enough diversity to design an extremely general thermodynamic domain model without relying on proprietary software**.

You would encounter essentially all of the major mathematical representation classes:

\[
\begin{aligned}
&\text{explicit empirical correlations}\\
&\text{fundamental Helmholtz potentials}\\
&\text{residual/excess contributions}\\
&\text{group-contribution models}\\
&\text{association-site models}\\
&\text{standard-state species models}\\
&\text{reaction/log-K representations}\\
&\text{electrolyte interaction models}\\
&\text{sublattice/endmember Gibbs models}\\
&\text{pseudo-component characterization}\\
&\text{implicit constitutive systems}\\
&\text{constrained Gibbs minimization}\\
&\text{phase stability/phase discovery}\\
&\text{spatial free-energy functionals}\\
&\text{adsorption free-energy/isotherm models}\\
&\text{transport constitutive models}\\
&\text{piecewise/regional/backward equations}\\
&\text{experiment → regression → parameter models}.
\end{aligned}
\]

That is broad enough that I would expect the **canonical thermodynamic type system you derive from it to survive almost any later addition**, rather than being implicitly structured around the conventional `component + phase + EOS + flash` worldview.

One licensing principle is worth making architectural now: **store model provenance and redistribution rights independently from the mathematical representation.** The same canonical `BinaryInteractionParameter`, for example, could originate from an MIT-licensed source, an openly published paper, DIPPR, DDB, or an internal regression. The mathematical object can be identical while the rights to persist, expose, redistribute, or compile it into a product are completely different. This should therefore be metadata on every imported model/parameter/data artifact, not an external spreadsheet you reconcile later.