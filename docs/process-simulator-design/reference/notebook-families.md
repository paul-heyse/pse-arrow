---
title: "Reference case register"
notion_source: "https://app.notion.com/p/40b141b5836981db9a84dbcab1b77bc9"
notion_last_edited: "2026-11-01T23:34:09.831Z"
---

# Reference case register

Source: [Notion](https://app.notion.com/p/40b141b5836981db9a84dbcab1b77bc9). Imported 2026-11-02.

[Implementation design and IDAES equivalence plan](../delivery/index.md)

**Status: Source-inspected planning inventory** · 2026-11-01. The 55 documented notebook families below were inspected at the frozen modern examples revision `eb64e38ac429b7e3ae4f8db071f8d8cad2eef675`. None was executed in this review. The held notebook was also inspected; archive entries were identified from the source tree only.

## How to use this register

The table of contents contains 55 documented families. The repository contains 266 notebook files, grouped into 79 normalized path families after removing generated documentation, test, user, exercise, solution and source suffixes. The other families are one held, 20 archive, two developer and one unlisted empty dummy notebook. These are source-organization counts, not counts of unique scientific problems or accepted configurations. [Frozen documentation contents](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/_toc.yml); [frozen examples tree](https://github.com/IDAES/examples/tree/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples).

Each linked title below identifies an exact upstream file. The observations identify direct dependencies or a qualification focus, not a complete transitive runtime closure. P0 must resolve local helper modules, datasets, fitted artifacts, initializations and external prerequisites; expand meaningful scientific variants; and associate native case declarations and retained evidence through the existing owners. A tutorial concerning diagnostics may require an expected finding/refusal rather than a solved numerical candidate.

The adopted [Cross-industry domain contracts](../process/cross-industry-contracts.md) govern how selected native cases preserve meaning. Candidate non-IDAES archetypes in [Industry coverage and expansion strategy](../delivery/industry-coverage-and-expansion.md) are not entries in this frozen reference population. If an expansion is selected, retain its separate source/configuration scope and evidence through the same existing owners; do not reinterpret these 55 source families as an industry-readiness count.

The examples snapshot identifies itself as package 2.10.0 and does not pin its optional IDAES dependency. Compatibility with the chosen core 2.13.0 reference environment remains to be qualified. [Examples dependency declarations](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/pyproject.toml).

## Documented families

All rows in this section have status **Contents inspected; not executed**. Row numbers are navigation only, not new runtime identities.

### Tutorials — 5 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 1 | [Flash Unit Model Tutorial](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/tut/core/flash_unit.ipynb) | Flash authoring, specification, initialization, solve and physical-result inspection. |
| 2 | [HDA Flowsheet Simulation and Optimization](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/tut/core/hda_flowsheet.ipynb) | Custom HDA thermo/reaction configuration; reactor, flash, recycle and separation with simulation/optimization. |
| 3 | [Introduction to IDAES](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/tut/introduction.ipynb) | Introductory process authoring, execution and inspection workflow. |
| 4 | [Introduction to IDAES (short)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/tut/introduction_short.ipynb) | Condensed introductory authoring/execution workflow; retain as a separate documented entry point. |
| 5 | [Flowsheet Visualizer Tutorial](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/tut/ui/visualizer_tutorial.ipynb) | Companion visualization workflow; native UI parity concerns capabilities, not Python call names. |

### Diagnostics and scaling — 5 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 6 | [Degeneracy Hunter Examples](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/diagnostics/degeneracy_hunter.ipynb) | Diagnostics on constructed problematic systems; upstream SCIP test helper. |
| 7 | [IDAES Model Diagnostics Toolbox Tutorial](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/diagnostics/diagnostics_toolbox.ipynb) | Structural/numerical diagnostics and attributable findings on the demonstrated models. |
| 8 | [Structural singularity in a dynamic moving bed model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/diagnostics/structural_singularity.ipynb) | Local gas-solid moving-bed model with intentionally problematic structure; acceptance includes diagnostic findings. |
| 9 | [Advanced Scaling Techniques](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/scaling/advanced_scaling_techniques.ipynb) | MEA solvent/vapor packages and solvent reboiler; Jacobian conditioning and scaling. |
| 10 | [How to Create Scaler Objects in IDAES](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/scaling/scaler_workshop.ipynb) | Saponification property/reaction model, equilibrium reactor and custom scaling behavior. |

### Parameter estimation — 2 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 11 | [Parameter Estimation Using the NRTL State Block](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/param_est/parameter_estimation_nrtl_using_state_block.ipynb) | BTX NRTL state-block fitting through parmest. |
| 12 | [Parameter Estimation Using Flash Unit Model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/param_est/parameter_estimation_nrtl_using_unit_model.ipynb) | BTX NRTL flash-unit fitting through parmest. |

### DAE and control — 2 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 13 | [PETSc Time-stepping Solver -- Chemical Akzo Nobel Example](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/dae/petsc_chem.ipynb) | Core Chemical Akzo Nobel DAE example; PETSc time stepping. |
| 14 | [PETSc Time-stepping Solver -- PID Control and Steam Example](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/dae/petsc_pid.ipynb) | Local steam-tank/PID module; PETSc time stepping; notebook metadata skips the generated test target. |

### Integrated flowsheets — 4 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 15 | [HDA Flowsheet Costing](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/flowsheets/hda_flowsheet_with_costing.ipynb) | HDA local flowsheet builder and SSLW equipment costing; includes flash and distillation alternatives. |
| 16 | [HDA Flowsheet Simulation and Optimization](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/flowsheets/hda_flowsheet_with_distillation.ipynb) | TrayColumn, BTX activity coefficients, custom HDA property/reaction packages, recycle initialization. |
| 17 | [Methanol Synthesis Flowsheet Example](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/flowsheets/methanol_synthesis.ipynb) | Local methanol flowsheet builder, modular property/reaction configurations and SSLW costing. |
| 18 | [TSA Adsorption Cycle for Carbon Capture](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/flowsheets/temperature_swing_adsorption/temperature_swing_adsorption.ipynb) | Core fixed-bed TSA model and its initialization/reporting; cycle formulation. |

### Generic operations — 7 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 19 | [Compressor Unit Model with Span-Wagner Property Package for supercritical CO2](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/compressor.ipynb) | Span–Wagner CO2 properties and pressure-changer thermodynamic modes. |
| 20 | [Heat Exchanger 0D Unit Model with Ideal & IAPWS Property Package](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/heat_exchanger_0d.ipynb) | Two-sided heat exchanger with ideal-mixture and IAPWS packages; additional modular BT-ideal configuration. |
| 21 | [Heater Unit Model with Ideal Property Package](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/heater.ipynb) | Ideal-property heater configuration, operating specifications and heat/pressure results. |
| 22 | [Mixer Unit Model with Ideal Property Package](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/mixer.ipynb) | Ideal-property mixing configuration, feed states and original balances. |
| 23 | [Pump Unit Model with IAPWS Property Package](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/pump.ipynb) | IAPWS steam/water properties and pump model. |
| 24 | [IDAES Skeleton Unit Model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/skeleton_unit.ipynb) | Open custom-unit equations, specification and initialization workflow. |
| 25 | [Turbine Unit Model with IAPWS Property Package](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/operations/turbine.ipynb) | IAPWS steam/water properties and turbine model. |

### Reactors — 5 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 26 | [Flowsheet Continuous Stirred Tank Reactor (CSTR) Simulation and Optimization of Ethylene Glycol Production](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/reactors/cstr.ipynb) | Ethylene-glycol process simulation/optimization with modular kinetics and thermo. |
| 27 | [Flowsheet Equilibrium Reactor Simulation and Optimization of Steam Methane Reforming](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/reactors/equilibrium_reactor.ipynb) | Steam-methane reforming with reaction-equilibrium formulation. |
| 28 | [Flowsheet Gibbs Reactor Simulation and Optimization of Steam Methane Reforming](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/reactors/gibbs_reactor.ipynb) | Steam-methane reforming with Gibbs minimization and elemental conservation. |
| 29 | [Flowsheet Plug Flow Reactor (PFR) Simulation and Optimization of Ethylene Glycol Production](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/reactors/plug_flow_reactor.ipynb) | Ethylene-glycol process simulation/optimization with spatially distributed reaction. |
| 30 | [Flowsheet Stoichiometric Reactor Simulation and Optimization of Ethylene Glycol Production](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/reactors/stoichiometric_reactor.ipynb) | Ethylene-glycol process with specified conversion/stoichiometry. |

### Custom units — 3 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 31 | [Creating Custom Unit Model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/custom_unit_models/creating_unit_model.ipynb) | Custom aqueous property and unit-model extension contract. |
| 32 | [Custom Compressor Unit Model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/custom_unit_models/custom_compressor.ipynb) | Custom unit equations and methanol property package. |
| 33 | [Custom Heater Unit Model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/custom_unit_models/custom_heater.ipynb) | Custom unit equations and methanol property package. |

### Properties and reactions — 4 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 34 | [Introduction to Property Packages in IDAES](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/properties/custom/custom_physical_property_packages.ipynb) | Custom property-package construction, state requirements and metadata; retain the demonstrated scientific definitions. |
| 35 | [Reaction Property Packages in IDAES](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/properties/custom/custom_reaction_property_packages.ipynb) | Custom reaction-package construction, parameters and its unit-model consumer. |
| 36 | [Create a dictionary and use Txy diagrams](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/properties/dictionary_txy_diagrams.ipynb) | Modular ideal EOS, FTPx, smooth VLE and component correlations; phase-envelope calculation. |
| 37 | [Using Parameter Estimation with Modular Property Packages](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/properties/parameter_estimation_pr.ipynb) | Modular CO2–bmimPF6 Peng–Robinson package, flash unit and parmest experiment fitting. |

### Surrogates — 14 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 38 | [Autothermal Reformer Flowsheet Optimization with ALAMO Surrogate Object](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/alamo/alamo_flowsheet_optimization.ipynb) | ALAMO training/evaluation and autothermal-reformer flowsheet embedding; optional external trainer. |
| 39 | [ML/AI Best Practices: "Selecting Surrogate Model Form/Size for Optimization"](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/best_practices_optimization.ipynb) | Local autothermal-reformer training utility; surrogate form/size choices and optimization. |
| 40 | [Autothermal Reformer Flowsheet Optimization with OMLT (TensorFlow Keras) Surrogate Object](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/omlt/keras_flowsheet_optimization.ipynb) | TensorFlow/Keras model training and OMLT equation embedding. |
| 41 | [PySMO Tutorial](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/pysmo/pysmo_basics.ipynb) | Polynomial, radial-basis and kriging training plus sampling. |
| 42 | [Autothermal Reformer Flowsheet Optimization with PySMO Surrogate Object](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/pysmo/pysmo_flowsheet_optimization.ipynb) | PySMO algebraic surrogate fitting and autothermal-reformer flowsheet embedding. |
| 43 | [Supercritical CO2 Property Surrogate with ALAMO Surrogate Object - Training Surrogate (Part 1)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/alamo/alamo_training.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 44 | [Supercritical CO2 Property Surrogate with ALAMO Surrogate Object - flowsheet\_optimization (Part 3)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/alamo/flowsheet_optimization.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 45 | [Supercritical CO2 Property Surrogate with ALAMO Surrogate Object - Embedding Surrogate (Part 2)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/alamo/surrogate_embedding.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 46 | [Supercritical CO2 Property Surrogate with OMLT Surrogate Object - flowsheet\_optimization (Part 3)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/omlt/flowsheet_optimization.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 47 | [Supercritical CO2 Property Surrogate with OMLT Surrogate Object - Training Surrogate (Part 1)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/omlt/keras_training.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 48 | [Supercritical CO2 Property Surrogate with OMLT Surrogate Object - Embedding Surrogate (Part 2)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/omlt/surrogate_embedding.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 49 | [Supercritical CO2 Property Surrogate with PySMO Surrogate Object - flowsheet\_optimization (Part 3)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/pysmo/flowsheet_optimization.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 50 | [Supercritical CO2 Property Surrogate with PySMO Surrogate Object - Training Surrogate (Part 1)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/pysmo/pysmo_training.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |
| 51 | [Supercritical CO2 Property Surrogate with PySMO Surrogate Object - Embedding Surrogate (Part 2)](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/surrogates/sco2/pysmo/surrogate_embedding.ipynb) | Three-step dependency chain: training, property-package embedding, flowsheet optimization; uses saved model/data artifacts. |

### Power and electrochemistry — 4 families

| # | Upstream family and exact source | Observed dependencies / qualification focus |
| --- | --- | --- |
| 52 | [NGCC Baseline and Turndown](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/power_gen/ngcc/ngcc.ipynb) | Local NGCC module; baseline and turndown operating cases. |
| 53 | [Solid oxide cell PID control](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/power_gen/solid_oxide_cell/soc_pid_control.ipynb) | Local SOC dynamic flowsheet; distributed electrochemical/thermal submodels, PETSc, PID antiwindup/bounds and operating-condition CSV data. |
| 54 | [Supercritical Power Plant Flowsheet Example](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/power_gen/supercritical/supercritical_power_plant.ipynb) | Core SCPC full-plant flowsheet; steam/water and flue-gas packages; heat-transfer and boiler closure. |
| 55 | [Supercritical Steam Cycle Example](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/power_gen/supercritical/supercritical_steam_cycle.ipynb) | Core supercritical steam-cycle module; Helmholtz steam/water properties. |

## Held family

This scientifically relevant case is outside the documented contents. Its environment and status require an explicit scope decision and qualification; presence in the tree does not establish current support.

| Upstream family | Inspection and dependency status |
| --- | --- |
| [CO2 Adsorption Desorption simulation example with a 1D Fixed Bed model](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/held/flowsheets/CO2_adsorption_desorption/CO2_Adsorption_Desorption_1DFixedBed.ipynb) | Contents inspected; not executed. Held example: core FixedBed1D, local NETL-32D gas/solid/adsorption packages and PETSc. |

## Archived path families

All 20 entries below have status **Tree entry inspected; contents not inspected; not executed**. Their paths include historical duplicates and an embedded DMF workspace copy. They are not automatically additional mandatory current cases. P0 should retain their scientific behavior where required, identify duplicates explicitly and avoid reviving unsupported infrastructure solely to reproduce a notebook.

| Archived family and exact source | Source grouping |
| --- | --- |
| [boiler flowsheet recon](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/data_reconciliation/boiler_flowsheet_recon_src.ipynb) | Archive / data\_reconciliation |
| [econ parmest](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/data_reconciliation/econ_parmest_src.ipynb) | Archive / data\_reconciliation |
| [econ recon](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/data_reconciliation/econ_recon_src.ipynb) | Archive / data\_reconciliation |
| [data management framework](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/dmf/data_management_framework.ipynb) | Archive / dmf |
| [data management framework](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/dmf/my_workspace/files/928385c3acda4a449412c5bfbbaa83b5/data_management_framework.ipynb) | Archive / dmf |
| [bifunctional surface design](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/matopt/bifunctional_surface_design_src.ipynb) | Archive / matopt |
| [bimetallic nanocluster design](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/matopt/bimetallic_nanocluster_design_src.ipynb) | Archive / matopt |
| [metal oxide bulk design](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/matopt/metal_oxide_bulk_design_src.ipynb) | Archive / matopt |
| [monometallic nanocluster design](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/matopt/monometallic_nanocluster_design_src.ipynb) | Archive / matopt |
| [nanowire design](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/matopt/nanowire_design_src.ipynb) | Archive / matopt |
| [surface design](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/matopt/surface_design_src.ipynb) | Archive / matopt |
| [data quality control](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/pecos/data_quality_control_src.ipynb) | Archive / pecos |
| [ngfc flowsheet](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/ngfc/ngfc_flowsheet_src.ipynb) | Archive / power\_gen |
| [rsofc soec](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/rsofc/rsofc_soec_src.ipynb) | Archive / power\_gen |
| [soec](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/soec/soec.ipynb) | Archive / power\_gen |
| [sofc](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/sofc/sofc.ipynb) | Archive / power\_gen |
| [sofc with soec](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/sofc_soec/sofc_with_soec.ipynb) | Archive / power\_gen |
| [subcritical boiler](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/subcritical/subcritical_boiler_src.ipynb) | Archive / power\_gen |
| [supercritical power plant](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/supercritical/supercritical_power_plant_src.ipynb) | Archive / power\_gen |
| [supercritical steam cycle](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/archive/power_gen/supercritical/supercritical_steam_cycle_src.ipynb) | Archive / power\_gen |

## Developer and empty-file exclusions

Two developer notebook families exercise the examples documentation/authoring machinery. One unlisted dummy notebook is empty. They are not additional process-science cases in this reviewed reference population.

| Entry | Disposition |
| --- | --- |
| [cache](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/_dev/notebooks/cache.ipynb) | Tree entry inspected; contents not inspected |
| [notebook tags example](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/_dev/notebooks/ex/notebook_tags_example.ipynb) | Tree entry inspected; contents not inspected |
| [foo](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/power_gen/testnew/foo.ipynb) | Unlisted dummy notebook with zero cells; not a scientific example. |

## Required coverage beyond this notebook register

The capability-coverage chapter adds core non-notebook examples, generic and specialized model configurations, custom extension workflows, compatibility API cases, independent property verification and robustness tests. The inspected core tree contains 344 Python test modules, but their parameterized instances have not been enumerated by this review. The compatibility tree has 34 versioned API example/tutorial test modules with additional scientific-verification and robustness populations.

P0 must turn those sources into a finite configuration/case population. The final equivalence claim requires every mandatory obligation in that reviewed population to have accepted evidence. Completing these 55 documented source families alone cannot establish full release coverage.
