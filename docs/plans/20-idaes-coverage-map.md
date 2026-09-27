---
title: idaes-pse capability coverage map for the target design
status: draft
date: 2026-09-26
parent: docs/plans/20-idaes-capability-target.md
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
---

# idaes-pse capability coverage map

**Evidence level: Proposed.** This map is the completeness check for the target design. It
places every IDAES process block with a static model record and every modular-property method
class in the target: its realization mechanism, its owner or package, and the delivery wave in
[Plan 20](20-idaes-capability-target.md#plan). The non-block capabilities follow in hand-written
tables.

- **Sources.** The generated tables are derived from the `pyomo-and-solvers` skill's static
  records of idaes-pse 2.13.0: `content/idaes/models/*.json` (156 declaration-scope model
  records) and `content/idaes/methods/*.json` (70 method classes). Detail for any row is at
  `python3 .codex/skills/pyomo-and-solvers/scripts/reference.py show idaes-model:<class>`.
- **"comps".** The count is the construction events a record holds, adding its own, delegated
  and method-built events.
- **"oracles".** The count is the upstream test assertions from tests that construct the
  block. Blocks every test builds, such as `FlowsheetBlock`, are therefore over-linked. Oracles
  are reference values for parity, not observations made here.
- **Scope decision.** This map assumes that `models_extra`, surrogates and the applications
  are in scope. That is a required authority change: blueprint §0.2 and the relationship
  document place them out of scope today (review slot 11).

## Waves

| Wave | Content | Rows here |
|---|---|---|
| W0 | Foundations: `pse-modeling` and `pse-properties` crates; template mechanisms (package parameters and facts, material domains, state-block children, multiplicity, delegated ports, expression symbols, interface slots, presets); indexed law engine; property-kind registry; identity rules; smoothing and complementarity primitives; template syntax | 8 |
| W1 | Core thermodynamics and steady core units: ideal-gas and pure-liquid forms; ideal and cubic EoS with cubic-root and EO closures; FTPx, FcTP, FPhx, FcPh and FpcTP states; SmoothVLE and complementarity VLE; bubble/dew; CV0D; feed, product, mixer, separator, heater, flash, heat exchangers (0D, NTU), pressure changers, valve, junction, translator, scaler | 73 |
| W2 | Reactions and remaining steady units; activity models; transport; initialization knowledge; derived-nominal scaling; diagnostics catalogue | 37 |
| W3 | Discretization; CV1D; distributed units; dynamics as an analysis mode (integrated and simultaneous); PID | 7 |
| W4 | Advanced thermodynamics: native PC-SAFT (FeOS production provider deleted), multiparameter Helmholtz and pure-fluid closures, nested flash, electrolytes and eNRTL | 19 |
| W5 | Costing, surrogates, studies (sweeps, convergence, sensitivity, covariance, NMPC/MHE, multiperiod), `models_extra` libraries, applications | 82 |

## Capabilities that are not process blocks

| IDAES capability | Source (idaes-pse 2.13.0) | Target realization | Owner | Wave |
|---|---|---|---|---|
| `declare_process_block_class`, `ProcessBlockData`, CONFIG | `core/base/process_block.py`, `process_base.py` | Typed templates, parameters and features; presets; admission invariants | `pse-modeling`, `pse-authoring` | W0 |
| `useDefault` and flowsheet flag inheritance | `process_base.py`, `flowsheet_model.py` | Static scope defaults recorded in specialization; `dynamic` becomes the analysis mode | `pse-modeling` | W0 |
| Ports and arcs (Pyomo network) | `unit_model.py`, `pyomo.network` | Typed ports with member roles; equality expansion; explicit splitters and mixers | `pse-modeling` (admission kept) | W0 |
| Property metadata (`StandardPropertySet`, `ElectrolytePropertySet`) | `core/base/property_set.py`, `property_meta.py` | Registry property kinds with index shape and complete quantity type | `pse-schema`, `pse-properties` | W0 |
| On-demand property construction | `property_base.py` (`build_on_demand`) | Salsa-tracked property resolution with lineage | `pse-properties` | W1 |
| Balance-type enums and control-volume options | `control_volume_base.py` | Registry enums (IDAES names kept) and CV template parameters | registry, `pse.core` | W0 |
| Units of measurement, currency units | `core/util/units_of_measurement.py`, `costing_base.register_idaes_currency_units` | `pse-quantity` (existing) plus cost-index conversion rules | `pse-quantity` | W5 |
| Smooth math (`smooth_max`, `smooth_min`, `smooth_abs`, `safe_log`, `safe_sqrt`) | `core/util/math.py` | DSL formulation primitives with typed ε and domain obligations | `pse-quantity`, `pse-math` | W0 |
| `cbrt` and `x_over_exp_x_minus_one` external functions | idaes-ext `functions.so` | DSL functions with obligations | `pse-quantity`, `pse-math` | W1 |
| `cubic_root_l`/`cubic_root_h` (and `_ext`/`_nan`) | idaes-ext `cubic_roots.so` (`show idaes-ext:cubic_root_l`) | Cubic-root closure algorithm with branch typing and implicit derivatives; EO complementarity alternative | `pse-thermo` | W1 |
| General Helmholtz external functions (238 registrations) | idaes-ext `general_helmholtz_external.so` | Helmholtz model forms + identities + pure-fluid inversion closure | `pse-properties`, `pse-thermo` | W4 |
| Initializers (15 defaults across records; the `InitializerBase` lifecycle) | `core/initialization/*`, unit modules | Generic transactional engine + start estimators + relaxation stages + homotopy policy | `pse-structural`, runtime strategies, package data | W2 |
| `propagate_state`, `fix_state_vars`, `revert_state_vars`, `initialize_by_time_element` | `core/util/initialization.py` | Structural predecessor order; overlays; simultaneous or integrated dynamic starts | existing engine | W2/W3 |
| Homotopy | `core/solvers/homotopy.py` | Adaptive continuation policy | runtime strategies | W2 |
| Scalers (21 defaults; about 50 `CustomScalerBase` subclasses), `AutoScaler`, `ScalingProfiler`, nominal tools, legacy `iscale` | `core/scaling/*`, `core/util/scaling.py` | Declared hints plus derived nominals from term magnitudes, IDAES scheme names; scaling profile study | `pse-math::numerics`, package data | W2 |
| `DiagnosticsToolbox` (31 methods), `SVDToolbox`, `DegeneracyHunter`, ill-conditioning certificate, `IpoptConvergenceAnalysis`, evaluation-error walker, constraint-term analysis | `core/util/diagnostics_tools/*` | Diagnostics catalogue with threshold profile `idaes-2.13` | `pse-structural`, `pse-backend-native::quality`, `pse-math`, HiGHS, faer | W2 |
| `model_statistics` (98 functions) | `core/util/model_statistics.py` | Inspection queries over the specialized model | `pse-modeling` + DataFusion inspection | W2 |
| `get_solver`, `idaes.cfg` solver defaults, `SolverWrapper` | `core/solvers/*`, `config.py` | Named solver profile `idaes-2.13` over class-specific adapters | `pse-backend-native` | W1 |
| `ipopt_l1` | `core/solvers/ipopt_l1.py` | POUNCE ℓ1 route for whole-model explanation (ADR-0109); elastic overlays for selected constraints | `pse-backend-native` POUNCE method; `pse-modeling` / analysis policy | W2 |
| PETSc SNES/TS/TAO, PETSc DAE stepping | `core/solvers/petsc.py`, idaes-ext `petsc` | Not reproduced; covered by KINSOL, Diffsol/IDAS and Ipopt/POUNCE | — | — |
| `replace_variables`, `simple_equality_eliminator` transformations | `core/plugins/*` | Presolve (pounce-presolve, existing) and template aliasing; no model-rewriting transformation | existing | — |
| `dyn_utils` (15 DAE helpers) | `core/util/dyn_utils.py` | Analysis-mode generation; time-indexed result projections | `pse-modeling`, results | W3 |
| `parameter_sweep`, convergence evaluation and search | `core/util/parameter_sweep.py`, `core/util/convergence/*` | Study runner over `case_sets` | runtime studies | W5 |
| Parameter estimation (parmest usage) | examples | Fitting (existing) + covariance and profile-likelihood studies | runtime fitting | W5 |
| Uncertainty propagation (`sens.py`) | `apps/uncertainty_propagation` | Parametric NLP sensitivity (POUNCE sensitivity / `pounce-sens-core` with FERAL, ADR-0107) + covariance propagation | runtime studies | W5 |
| Surrogates: `SurrogateBlock`, ALAMO, PySMO, Keras/ONNX via OMLT, sampling, metrics, plotting | `core/surrogate/*` | Surrogate data packages embedded as forms or providers; training and sampling studies | `pse-modeling`, studies | W5 |
| Stream tables, performance contents, tags | `core/util/tables.py`, `tags.py`, unit `_get_*_contents` | Result projections (Arrow) over ports and report symbols | results | W1 |
| Model serializer (`to_json`/`from_json`) | `core/util/model_serializer.py` | Immutable revisions + publication (existing; stronger) | existing | — |
| `structfs` flowsheet runner | `core/util/structfs/*` | Studies and workflows over prepared cases | runtime | W5 |
| Utility minimization (pinch) | `core/util/utility_minimization.py` | Optional heat-cascade reference template | `pse.units.core` | W5 |
| CLI (`idaes get-extensions`, `config-*`, `convergence-*`) | `commands/*` | Not reproduced as such; repository CLI and Python boundary | — | — |
| UI (`FlowsheetBlock.visualize` → idaes-ui) | `idaes-ui` | Out of the modeling target; result relations are the integration surface for a visualizer | — | — |
| DMF | `core/dmf` (stub) | Out of scope; publication and lineage cover provenance | — | — |
| Grid integration (bidder, tracker, coordinator, forecaster, multiperiod) | `apps/grid_integration/*` | Multiperiod templates + market workflows; Prescient via the Python boundary | studies, `pse.apps.grid` | W5 |
| NMPC/MHE (`caprese`, `nmpc`) | `apps/caprese`, `apps/nmpc` | Rolling-horizon workflow over simultaneous dynamics | studies | W5 |
| MatOpt | `apps/matopt` | Discrete-domain MILP templates (ADR-0103, ADR-0104) on HiGHS | `pse.apps.matopt` | W5 |

## Process blocks (generated from model records)

### `idaes.apps` (3)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `grid_integration.pricetaker.design_and_operation_models.DesignModel` | Multiperiod design/operation/storage templates | pse.apps.grid | W5 | 6 comps, 98 oracles |
| `grid_integration.pricetaker.design_and_operation_models.OperationModel` | Multiperiod design/operation/storage templates | pse.apps.grid | W5 | 9 comps, 22 oracles |
| `grid_integration.pricetaker.design_and_operation_models.StorageModel` | Multiperiod design/operation/storage templates | pse.apps.grid | W5 | 8 comps, 34 oracles |

### `idaes.core.base` (22)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `base.components.Anion` | Species role in material system (pse-material) | pse-material | W4 | 2 comps, 7 oracles |
| `base.components.Apparent` | Species role in material system (pse-material) | pse-material | W4 | 4 comps, 5 oracles |
| `base.components.Cation` | Species role in material system (pse-material) | pse-material | W4 | 2 comps, 7 oracles |
| `base.components.Component` | Species role in material system (pse-material) | pse-material | W1 | 3 comps, 29 oracles |
| `base.components.Ion` | Species role in material system (pse-material) | pse-material | W4 | 2 comps, 2 oracles |
| `base.components.Solute` | Species role in material system (pse-material) | pse-material | W1 | 4 comps, 4 oracles |
| `base.components.Solvent` | Species role in material system (pse-material) | pse-material | W1 | 4 comps, 4 oracles |
| `base.control_volume0d.ControlVolume0DBlock` | CV0D reference template over the indexed law engine | pse.core | W1 | 40 comps, 357 oracles |
| `base.control_volume1d.ControlVolume1DBlock` | CV1D reference template + discretization transformation | pse.core | W3 | 51 comps, 438 oracles |
| `base.control_volume_base.ControlVolume` | Control-volume interface (template kind) | pse-modeling | W0 | 0 comps, 0 oracles |
| `base.costing_base.FlowsheetCostingBlock` | Costing binding + flowsheet accounting laws over cost contributions | pse-modeling / pse.costing | W5 | 13 comps, 0 oracles |
| `base.costing_base.UnitModelCostingBlock` | Costing binding + flowsheet accounting laws over cost contributions | pse-modeling / pse.costing | W5 | 0 comps, 398 oracles |
| `base.extended_control_volume0d.ExtendedControlVolume0DBlock` | CV0D reference template over the indexed law engine | pse.core | W1 | 41 comps, 11 oracles |
| `base.extended_control_volume1d.ExtendedControlVolume1DBlock` | CV1D reference template + discretization transformation | pse.core | W3 | 52 comps, 9 oracles |
| `base.flowsheet_model.FlowsheetBlock` | Flowsheet template with static scope defaults | pse-modeling | W0 | 2 comps, 11303 oracles |
| `base.phases.AqueousPhase` | Phase type in material system | pse-material | W1 | 1 comps, 0 oracles |
| `base.phases.LiquidPhase` | Phase type in material system | pse-material | W1 | 1 comps, 0 oracles |
| `base.phases.Phase` | Phase type in material system | pse-material | W1 | 1 comps, 10 oracles |
| `base.phases.SolidPhase` | Phase type in material system | pse-material | W1 | 1 comps, 88 oracles |
| `base.phases.VaporPhase` | Phase type in material system | pse-material | W1 | 1 comps, 0 oracles |
| `base.process_base.ProcessBaseBlock` | Template kinds (process/unit interfaces) | pse-modeling | W0 | 0 comps, 88 oracles |
| `base.unit_model.UnitModelBlock` | Template kinds (process/unit interfaces) | pse-modeling | W0 | 9 comps, 373 oracles |

### `idaes.core.util` (4)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `util.testing.PhysicalParameterTestBlock` | Conformance fixture packages | tests/conformance | W0 | 8 comps, 1935 oracles |
| `util.testing.ReactionBlock` | Conformance fixture packages | tests/conformance | W0 | 1 comps, 0 oracles |
| `util.testing.ReactionParameterTestBlock` | Conformance fixture packages | tests/conformance | W0 | 2 comps, 637 oracles |
| `util.testing.StateBlockForTesting` | Conformance fixture packages | tests/conformance | W0 | 19 comps, 22 oracles |

### `idaes.models.control` (1)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `control.controller.PIDController` | Signal-port controller template (dynamic mode) | pse.control | W3 | 35 comps, 31 oracles |

### `idaes.models.costing` (2)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `costing.QGESS.QGESSCosting` | Costing correlation forms + enums as typed parameters | pse.costing | W5 | 189 comps, 89 oracles |
| `costing.SSLW.SSLWCosting` | Costing correlation forms + enums as typed parameters | pse.costing | W5 | 86 comps, 176 oracles |

### `idaes.models.properties` (18)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `properties.activity_coeff_models.BTX_activity_coeff_VLE.BTXParameterBlock` | Excess-Gibbs model forms (NRTL/Wilson) + example packages | pse.properties.activity | W2 | 30 comps, 1776 oracles |
| `properties.activity_coeff_models.activity_coeff_prop_pack.ActivityCoeffParameterBlock` | Excess-Gibbs model forms (NRTL/Wilson) + example packages | pse.properties.activity | W2 | 7 comps, 0 oracles |
| `properties.activity_coeff_models.activity_coeff_prop_pack.ActivityCoeffStateBlock` | Excess-Gibbs model forms (NRTL/Wilson) + example packages | pse.properties.activity | W2 | 96 comps, 0 oracles |
| `properties.activity_coeff_models.methane_combustion_ideal.MethaneParameterBlock` | Excess-Gibbs model forms (NRTL/Wilson) + example packages | pse.properties.activity | W2 | 25 comps, 83 oracles |
| `properties.general_helmholtz.helmholtz_functions.HelmholtzParameterBlock` | Multiparameter Helmholtz forms + pure-fluid inversion closure | pse.properties.helmholtz / pse-thermo | W4 | 16 comps, 318 oracles |
| `properties.general_helmholtz.helmholtz_state.HelmholtzStateBlock` | Multiparameter Helmholtz forms + pure-fluid inversion closure | pse.properties.helmholtz / pse-thermo | W4 | 144 comps, 0 oracles |
| `properties.iapws95.Iapws95ParameterBlock` | Multiparameter Helmholtz forms + pure-fluid inversion closure | pse.properties.helmholtz / pse-thermo | W4 | 16 comps, 1026 oracles |
| `properties.iapws95.Iapws95StateBlock` | Multiparameter Helmholtz forms + pure-fluid inversion closure | pse.properties.helmholtz / pse-thermo | W4 | 144 comps, 21 oracles |
| `properties.interrogator.properties_interrogator.InterrogatorStateBlock` | Property-demand inspection (resolver demand report) | pse-properties | W1 | 6 comps, 0 oracles |
| `properties.interrogator.properties_interrogator.PropertyInterrogatorBlock` | Property-demand inspection (resolver demand report) | pse-properties | W1 | 8 comps, 59 oracles |
| `properties.interrogator.reactions_interrogator.InterrogatorReactionBlock` | Property-demand inspection (resolver demand report) | pse-properties | W1 | 5 comps, 0 oracles |
| `properties.interrogator.reactions_interrogator.ReactionInterrogatorBlock` | Property-demand inspection (resolver demand report) | pse-properties | W1 | 3 comps, 23 oracles |
| `properties.modular_properties.base.generic_property.GenericParameterBlock` | Property package + state-definition + resolver | pse-properties | W1 | 24 comps, 4282 oracles |
| `properties.modular_properties.base.generic_property.GenericStateBlock` | Property package + state-definition + resolver | pse-properties | W1 | 130 comps, 0 oracles |
| `properties.modular_properties.base.generic_reaction.GenericReactionBlock` | Reaction package | pse-properties | W2 | 7 comps, 0 oracles |
| `properties.modular_properties.base.generic_reaction.GenericReactionParameterBlock` | Reaction package | pse-properties | W2 | 8 comps, 420 oracles |
| `properties.swco2.SWCO2ParameterBlock` | Multiparameter Helmholtz forms + pure-fluid inversion closure | pse.properties.helmholtz / pse-thermo | W4 | 16 comps, 18 oracles |
| `properties.swco2.SWCO2StateBlock` | Multiparameter Helmholtz forms + pure-fluid inversion closure | pse.properties.helmholtz / pse-thermo | W4 | 144 comps, 0 oracles |

### `idaes.models.unit_models` (31)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `unit_models.cstr.CSTR` | Single-CV reactor template (CV0D + rate reactions) | pse.units.core | W2 | 57 comps, 142 oracles |
| `unit_models.equilibrium_reactor.EquilibriumReactor` | Single-CV reactor template (equilibrium reactions) | pse.units.core | W2 | 55 comps, 135 oracles |
| `unit_models.feed.Feed` | Boundary template (state-block child + outlet port) | pse.units.core | W1 | 10 comps, 56 oracles |
| `unit_models.feed_flash.FeedFlash` | Boundary template with package VLE formulation | pse.units.core | W1 | 42 comps, 130 oracles |
| `unit_models.flash.Flash` | CV0D + package VLE formulation + phase-split outlets | pse.units.core | W1 | 57 comps, 178 oracles |
| `unit_models.gibbs_reactor.GibbsReactor` | Element law + Gibbs stationarity (resolved chemical potentials) | pse.units.core | W2 | 42 comps, 115 oracles |
| `unit_models.heat_exchanger.HeatExchanger` | Two CV0D children + ΔT interface slot (LMTD/AMTD/Underwood/smooth) | pse.units.core | W1 | 62 comps, 311 oracles |
| `unit_models.heat_exchanger_1D.HeatExchanger1D` | Two CV1D children + discretization policy | pse.units.core | W3 | 122 comps, 573 oracles |
| `unit_models.heat_exchanger_lc.HeatExchangerLumpedCapacitance` | HX + lumped wall capacitance (dynamic-capable) | pse.units.core | W3 | 75 comps, 79 oracles |
| `unit_models.heat_exchanger_ntu.HeatExchangerNTU` | Two CV0D children + NTU/effectiveness form | pse.units.core | W1 | 104 comps, 103 oracles |
| `unit_models.heater.Heater` | CV0D child + heat contribution | pse.units.core | W1 | 54 comps, 283 oracles |
| `unit_models.mixer.Mixer` | Indexed inlet children + mixing laws + pressure equality/smooth-min | pse.units.core | W1 | 25 comps, 194 oracles |
| `unit_models.mscontactor.MSContactor` | Indexed stream×stage children + interaction map + heterogeneous reactions | pse.units.core | W2 | 52 comps, 14 oracles |
| `unit_models.pipe.Pipe` | CV0D (or CV1D) + friction/pressure-drop slot | pse.units.core | W2 | 67 comps, 58 oracles |
| `unit_models.plug_flow_reactor.PFR` | CV1D + reactions + discretization policy | pse.units.core | W3 | 69 comps, 68 oracles |
| `unit_models.pressure_changer.Compressor` | CV0D + thermodynamic-assumption slot; Turbine/Compressor/Pump presets; performance-curve slot | pse.units.core | W1 | 73 comps, 9 oracles |
| `unit_models.pressure_changer.IsentropicPerformanceCurve` | CV0D + thermodynamic-assumption slot; Turbine/Compressor/Pump presets; performance-curve slot | pse.units.core | W1 | 2 comps, 0 oracles |
| `unit_models.pressure_changer.PressureChanger` | CV0D + thermodynamic-assumption slot; Turbine/Compressor/Pump presets; performance-curve slot | pse.units.core | W1 | 73 comps, 151 oracles |
| `unit_models.pressure_changer.Pump` | CV0D + thermodynamic-assumption slot; Turbine/Compressor/Pump presets; performance-curve slot | pse.units.core | W1 | 73 comps, 21 oracles |
| `unit_models.pressure_changer.Turbine` | CV0D + thermodynamic-assumption slot; Turbine/Compressor/Pump presets; performance-curve slot | pse.units.core | W1 | 73 comps, 27 oracles |
| `unit_models.product.Product` | Boundary template (state-block child + inlet port) | pse.units.core | W1 | 10 comps, 53 oracles |
| `unit_models.separator.Separator` | Indexed outlet children + split basis/energy split enums + ideal separation | pse.units.core | W1 | 26 comps, 394 oracles |
| `unit_models.shell_and_tube_1d.ShellAndTube1D` | Two CV1D children + wall + discretization | pse.units.core | W3 | 130 comps, 343 oracles |
| `unit_models.skeleton_model.SkeletonUnitModel` | User template: declared ports + equation or surrogate slot | pse.units.core | W2 | 0 comps, 53 oracles |
| `unit_models.solid_liquid.sl_separator.SLSeparator` | Separator variant over solid/liquid packages | pse.units.core | W2 | 15 comps, 74 oracles |
| `unit_models.solid_liquid.thickener.Thickener0D` | Solid-liquid template (settling correlation forms) | pse.units.core | W2 | 43 comps, 38 oracles |
| `unit_models.statejunction.StateJunction` | State-block child exposed on inlet/outlet ports | pse.units.core | W1 | 10 comps, 62 oracles |
| `unit_models.stoichiometric_reactor.StoichiometricReactor` | CV0D + extent-specified reactions | pse.units.core | W2 | 55 comps, 48 oracles |
| `unit_models.stream_scaler.StreamScaler` | Extensive-member scaling template | pse.units.core | W1 | 14 comps, 40 oracles |
| `unit_models.translator.Translator` | Two state-block children + explicit translation equations | pse.units.core | W1 | 11 comps, 22 oracles |
| `unit_models.valve.Valve` | CV0D + valve-characteristic slot (linear/equal-%/quick) + pressure-flow slot | pse.units.core | W1 | 74 comps, 144 oracles |

### `idaes.models_extra.co2_capture_and_utilization` (1)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `co2_capture_and_utilization.unit_models.membrane_1d.Membrane1D` | Membrane CV1D template | pse.units.membrane | W5 | 23 comps, 48 oracles |

### `idaes.models_extra.column_models` (9)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `column_models.MEAsolvent_column.MEAColumn` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 221 comps, 0 oracles |
| `column_models.condenser.Condenser` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 59 comps, 164 oracles |
| `column_models.plate_heat_exchanger.PlateHeatExchanger` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 139 comps, 55 oracles |
| `column_models.reboiler.Reboiler` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 61 comps, 72 oracles |
| `column_models.solvent_column.PackedColumn` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 139 comps, 52 oracles |
| `column_models.solvent_condenser.SolventCondenser` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 60 comps, 105 oracles |
| `column_models.solvent_reboiler.SolventReboiler` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 60 comps, 35 oracles |
| `column_models.tray.Tray` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 30 comps, 403 oracles |
| `column_models.tray_column.TrayColumn` | Column template (indexed trays / CV1D pairs + mass transfer) | pse.units.columns | W5 | 49 comps, 47 oracles |

### `idaes.models_extra.gas_distribution` (5)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `gas_distribution.properties.natural_gas.NaturalGasParameterBlock` | Natural-gas property package | pse.properties.gas_distribution | W5 | 13 comps, 186 oracles |
| `gas_distribution.properties.natural_gas.NaturalGasStateBlock` | Natural-gas property package | pse.properties.gas_distribution | W5 | 0 comps, 0 oracles |
| `gas_distribution.unit_models.compressor.IsothermalCompressor` | Gas-network template (1D momentum/friction) | pse.units.gas_distribution | W5 | 18 comps, 31 oracles |
| `gas_distribution.unit_models.node.PipelineNode` | Gas-network template (1D momentum/friction) | pse.units.gas_distribution | W5 | 29 comps, 66 oracles |
| `gas_distribution.unit_models.pipeline.GasPipeline` | Gas-network template (1D momentum/friction) | pse.units.gas_distribution | W5 | 49 comps, 140 oracles |

### `idaes.models_extra.gas_solid_contactors` (16)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `gas_solid_contactors.properties.methane_iron_OC_reduction.gas_phase_thermo.GasPhaseParameterBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 24 comps, 801 oracles |
| `gas_solid_contactors.properties.methane_iron_OC_reduction.gas_phase_thermo.GasPhaseStateBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 36 comps, 0 oracles |
| `gas_solid_contactors.properties.methane_iron_OC_reduction.hetero_reactions.HeteroReactionParameterBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 10 comps, 659 oracles |
| `gas_solid_contactors.properties.methane_iron_OC_reduction.hetero_reactions.ReactionBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 8 comps, 0 oracles |
| `gas_solid_contactors.properties.methane_iron_OC_reduction.solid_phase_thermo.SolidPhaseParameterBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 21 comps, 756 oracles |
| `gas_solid_contactors.properties.methane_iron_OC_reduction.solid_phase_thermo.SolidPhaseStateBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 21 comps, 0 oracles |
| `gas_solid_contactors.properties.oxygen_iron_OC_oxidation.gas_phase_thermo.GasPhaseParameterBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 25 comps, 441 oracles |
| `gas_solid_contactors.properties.oxygen_iron_OC_oxidation.gas_phase_thermo.GasPhaseStateBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 36 comps, 0 oracles |
| `gas_solid_contactors.properties.oxygen_iron_OC_oxidation.hetero_reactions.HeteroReactionParameterBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 10 comps, 340 oracles |
| `gas_solid_contactors.properties.oxygen_iron_OC_oxidation.hetero_reactions.ReactionBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 8 comps, 0 oracles |
| `gas_solid_contactors.properties.oxygen_iron_OC_oxidation.solid_phase_thermo.SolidPhaseParameterBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 21 comps, 399 oracles |
| `gas_solid_contactors.properties.oxygen_iron_OC_oxidation.solid_phase_thermo.SolidPhaseStateBlock` | Gas/solid property or heterogeneous reaction package | pse.properties.gas_solid | W5 | 21 comps, 0 oracles |
| `gas_solid_contactors.unit_models.bubbling_fluidized_bed.BubblingFluidizedBed` | Two-material-system reactor template (0D/1D, multi-region) | pse.units.gas_solid | W5 | 277 comps, 638 oracles |
| `gas_solid_contactors.unit_models.fixed_bed_0D.FixedBed0D` | Two-material-system reactor template (0D/1D, multi-region) | pse.units.gas_solid | W5 | 30 comps, 93 oracles |
| `gas_solid_contactors.unit_models.fixed_bed_1D.FixedBed1D` | Two-material-system reactor template (0D/1D, multi-region) | pse.units.gas_solid | W5 | 103 comps, 97 oracles |
| `gas_solid_contactors.unit_models.moving_bed.MBR` | Two-material-system reactor template (0D/1D, multi-region) | pse.units.gas_solid | W5 | 151 comps, 190 oracles |

### `idaes.models_extra.power_generation` (43)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `power_generation.costing.power_plant_capcost.QGESSCosting` | Power-plant costing forms | pse.costing.power | W5 | 107 comps, 258 oracles |
| `power_generation.costing.power_plant_costing.PowerPlantCosting` | Power-plant costing forms | pse.costing.power | W5 | 221 comps, 266 oracles |
| `power_generation.properties.flue_gas_ideal.FlueGasParameterBlock` | Property package (ideal flue gas) | pse.properties.power | W5 | 19 comps, 98 oracles |
| `power_generation.properties.flue_gas_ideal.FlueGasStateBlock` | Property package (ideal flue gas) | pse.properties.power | W5 | 37 comps, 7 oracles |
| `power_generation.unit_models.balance.BalanceBlock` | Power-generation unit template | pse.units.power | W5 | 55 comps, 0 oracles |
| `power_generation.unit_models.boiler_fireside.BoilerFireside` | Power-generation unit template | pse.units.power | W5 | 116 comps, 21 oracles |
| `power_generation.unit_models.boiler_heat_exchanger.BoilerHeatExchanger` | Power-generation unit template | pse.units.power | W5 | 141 comps, 0 oracles |
| `power_generation.unit_models.boiler_heat_exchanger_2D.HeatExchangerCrossFlow2D_Header` | Power-generation unit template | pse.units.power | W5 | 332 comps, 8 oracles |
| `power_generation.unit_models.cpu.CarbonProcessingUnit` | Power-generation unit template | pse.units.power | W5 | 61 comps, 10 oracles |
| `power_generation.unit_models.cross_flow_heat_exchanger_1D.CrossFlowHeatExchanger1D` | Power-generation unit template | pse.units.power | W5 | 206 comps, 0 oracles |
| `power_generation.unit_models.downcomer.Downcomer` | Power-generation unit template | pse.units.power | W5 | 70 comps, 12 oracles |
| `power_generation.unit_models.drum.Drum` | Power-generation unit template | pse.units.power | W5 | 80 comps, 11 oracles |
| `power_generation.unit_models.drum1D.Drum1D` | Power-generation unit template | pse.units.power | W5 | 155 comps, 12 oracles |
| `power_generation.unit_models.feedwater_heater_0D.FWH0D` | Power-generation unit template | pse.units.power | W5 | 20 comps, 3 oracles |
| `power_generation.unit_models.feedwater_heater_0D.FWHCondensing0D` | Power-generation unit template | pse.units.power | W5 | 64 comps, 0 oracles |
| `power_generation.unit_models.feedwater_heater_0D_dynamic.FWH0DDynamic` | Power-generation unit template | pse.units.power | W5 | 20 comps, 6 oracles |
| `power_generation.unit_models.feedwater_heater_0D_dynamic.FWHCondensing0D` | Power-generation unit template | pse.units.power | W5 | 73 comps, 0 oracles |
| `power_generation.unit_models.heat_exchanger_3streams.HeatExchangerWith3Streams` | Power-generation unit template | pse.units.power | W5 | 169 comps, 13 oracles |
| `power_generation.unit_models.heater_1D.Heater1D` | Power-generation unit template | pse.units.power | W5 | 112 comps, 0 oracles |
| `power_generation.unit_models.helm.compressor.HelmIsentropicCompressor` | Helmholtz-package unit template | pse.units.power | W5 | 63 comps, 4 oracles |
| `power_generation.unit_models.helm.condenser_ntu.HelmNtuCondenser` | Helmholtz-package unit template | pse.units.power | W5 | 65 comps, 2 oracles |
| `power_generation.unit_models.helm.mixer.HelmMixer` | Helmholtz-package unit template | pse.units.power | W5 | 16 comps, 9 oracles |
| `power_generation.unit_models.helm.phase_separator.HelmPhaseSeparator` | Helmholtz-package unit template | pse.units.power | W5 | 18 comps, 7 oracles |
| `power_generation.unit_models.helm.pump.HelmPump` | Helmholtz-package unit template | pse.units.power | W5 | 62 comps, 4 oracles |
| `power_generation.unit_models.helm.splitter.HelmSplitter` | Helmholtz-package unit template | pse.units.power | W5 | 15 comps, 9 oracles |
| `power_generation.unit_models.helm.turbine.HelmIsentropicTurbine` | Helmholtz-package unit template | pse.units.power | W5 | 64 comps, 2 oracles |
| `power_generation.unit_models.helm.turbine_inlet.HelmTurbineInletStage` | Helmholtz-package unit template | pse.units.power | W5 | 75 comps, 10 oracles |
| `power_generation.unit_models.helm.turbine_multistage.HelmTurbineMultistage` | Helmholtz-package unit template | pse.units.power | W5 | 40 comps, 0 oracles |
| `power_generation.unit_models.helm.turbine_outlet.HelmTurbineOutletStage` | Helmholtz-package unit template | pse.units.power | W5 | 79 comps, 9 oracles |
| `power_generation.unit_models.helm.turbine_stage.HelmTurbineStage` | Helmholtz-package unit template | pse.units.power | W5 | 69 comps, 2 oracles |
| `power_generation.unit_models.helm.valve_steam.HelmValve` | Helmholtz-package unit template | pse.units.power | W5 | 62 comps, 0 oracles |
| `power_generation.unit_models.soc_submodels.channel.SocChannel` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 80 comps, 4 oracles |
| `power_generation.unit_models.soc_submodels.conductive_slab.SocConductiveSlab` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 47 comps, 5 oracles |
| `power_generation.unit_models.soc_submodels.contact_resistor.SocContactResistor` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 23 comps, 4 oracles |
| `power_generation.unit_models.soc_submodels.porous_conductive_slab.PorousConductiveSlab` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 89 comps, 4 oracles |
| `power_generation.unit_models.soc_submodels.solid_oxide_cell.SolidOxideCell` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 54 comps, 6 oracles |
| `power_generation.unit_models.soc_submodels.solid_oxide_module_simple.SolidOxideModuleSimple` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 24 comps, 21 oracles |
| `power_generation.unit_models.soc_submodels.triple_phase_boundary.SocTriplePhaseBoundary` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 43 comps, 6 oracles |
| `power_generation.unit_models.soec_design.SoecDesign` | Electrochemical template (Nernst/Butler–Volmer, 1D/2D domains) | pse.units.power | W5 | 43 comps, 0 oracles |
| `power_generation.unit_models.steamheater.SteamHeater` | Power-generation unit template | pse.units.power | W5 | 117 comps, 11 oracles |
| `power_generation.unit_models.waterpipe.WaterPipe` | Power-generation unit template | pse.units.power | W5 | 75 comps, 28 oracles |
| `power_generation.unit_models.watertank.WaterTank` | Power-generation unit template | pse.units.power | W5 | 69 comps, 10 oracles |
| `power_generation.unit_models.waterwall_section.WaterwallSection` | Power-generation unit template | pse.units.power | W5 | 157 comps, 10 oracles |

### `idaes.models_extra.temperature_swing_adsorption` (1)

| IDAES block | Target realization | Owner / package | Wave | Records |
|---|---|---|---|---|
| `temperature_swing_adsorption.fixed_bed_tsa0d.FixedBedTSA0D` | Adsorption template (isotherm forms, cyclic formulation) | pse.units.adsorption | W5 | 236 comps, 95 oracles |

## Modular-property method classes

| IDAES method class | Family | Target realization | Owner / package | Wave |
|---|---|---|---|---|
| `coolprop.coolprop_wrapper.CoolPropExpressionError` | external_property_wrapper | Reference oracle (CoolProp); optional pure-fluid provider | tests / pse-thermo | W4 |
| `coolprop.coolprop_wrapper.CoolPropPropertyError` | external_property_wrapper | Reference oracle (CoolProp); optional pure-fluid provider | tests / pse-thermo | W4 |
| `coolprop.coolprop_wrapper.CoolPropWrapper` | external_property_wrapper | Reference oracle (CoolProp); optional pure-fluid provider | tests / pse-thermo | W4 |
| `eos.ceos.Cubic` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ceos.MixingRuleA` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ceos.MixingRuleB` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ceos_common.CubicThermoExpressions` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ceos_common.CubicType` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ceos_common._ExternalFunctionSpecs` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.enrtl.ENRTL` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W4 |
| `eos.enrtl_parameters.ConstantAlpha` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W4 |
| `eos.enrtl_parameters.ConstantTau` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W4 |
| `eos.enrtl_reference_states.Symmetric` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W4 |
| `eos.enrtl_reference_states.Unsymmetric` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W4 |
| `eos.eos_base.EoSBase` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ideal.Ideal` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `eos.ideal.IdealScaler` | equation_of_state | Helmholtz or excess-Gibbs model form + identities | pse.properties.eos | W1 |
| `phase_equil.bubble_dew.IdealBubbleDew` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.bubble_dew.IdealBubbleDewScaler` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.bubble_dew.LogBubbleDew` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.bubble_dew.LogBubbleDewScaler` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.forms.FugacityScaler` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.forms.LogFugacityScaler` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.forms.fugacity` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.forms.log_fugacity` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.henry.ConstantH` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.henry.HenryType` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.smooth_VLE.SmoothVLE` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.smooth_VLE.SmoothVLEScaler` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.smooth_VLE_2.CubicComplementarityVLE` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `phase_equil.smooth_VLE_2.CubicComplementarityVLEScaler` | phase_equilibrium | Phase-equilibrium form or VLE formulation policy | pse-properties | W1 |
| `pure.ChapmanEnskog.ChapmanEnskogLennardJones` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W2 |
| `pure.ChungPure.ChungViscosityPure` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W2 |
| `pure.ConstantProperties.Constant` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W1 |
| `pure.Eucken.Eucken` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W2 |
| `pure.NIST.NIST` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W1 |
| `pure.Perrys.Perrys` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W1 |
| `pure.RPP3.RPP3` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W1 |
| `pure.RPP4.RPP4` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W1 |
| `pure.RPP5.RPP5` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W1 |
| `pure.electrolyte.relative_permittivity_constant` | pure_component_method | Ideal-gas / pure-liquid / transport model forms | pse.properties.methods | W2 |
| `reactions.dh_rxn.ConstantEnthalpyRxnScaler` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.dh_rxn.constant_dh_rxn` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_constant.ConstantKeq` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_constant.ConstantKeqScaler` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_constant.GibbsEnergyScaler` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_constant.gibbs_energy` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_constant.van_t_hoff` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_forms.LogPowerLawEquilScaler` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_forms.PowerLawEquilScaler` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_forms.log_power_law_equil` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_forms.log_solubility_product` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_forms.power_law_equil` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.equilibrium_forms.solubility_product` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.rate_constant.arrhenius` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `reactions.rate_forms.power_law_rate` | reaction_form | Rate / equilibrium / heat-of-reaction forms | pse.properties.reactions | W2 |
| `state_definitions.FPhx.FPhx` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FPhx.FPhxScaler` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FTPx.FTPx` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FTPx.FTPxScaler` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FcPh.FcPh` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FcPh.FcPhScaler` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FcTP.FcTP` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FcTP.FcTPScaler` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FpTPxpc.FpTPxpc` | state_definition | State-definition template | pse.properties.states | W4 |
| `state_definitions.FpcTP.FpcTP` | state_definition | State-definition template | pse.properties.states | W1 |
| `state_definitions.FpcTP.FpcTPScaler` | state_definition | State-definition template | pse.properties.states | W1 |
| `transport_properties.no_method.NoMethod` | transport_property | Transport mixing-rule forms | pse.properties.transport | W2 |
| `transport_properties.thermal_conductivity_wms.ThermalConductivityWMS` | transport_property | Transport mixing-rule forms | pse.properties.transport | W2 |
| `transport_properties.viscosity_wilke.ViscosityWilke` | transport_property | Transport mixing-rule forms | pse.properties.transport | W2 |
