---
title: "Plan64h required process population"
status: in-progress
date: 2026-11-03
doc_role: reference
doc_owner: docs/plans/64h-reference-coverage-and-qualification.md
doc_topics: [current-work, design, validation]
---

# Required process population

## Scope, sources and evidence boundary

**Interface-checked, source only:** this investigation supplies concrete reference
selections and prerequisite distinctions for D01/E01/F01/G01. Its native baseline is
`aa98d039e`; the local `external/idaes-pse` reading copy has HEAD
`84532d7f99a9f976b05170948147afdd69cfa5f6`. The exact modern examples and
compatibility source archives were acquired separately into temporary storage;
neither the external reading copy nor any product source was modified. No reference
environment, model, notebook, solver, test or measurement was executed. The
[64h owner](64h-reference-coverage-and-qualification.md) owns execution and H-package
state; this document owns the investigated source/configuration crosswalk, not a
second qualification ledger.

The fixed source roots are [C: core 2.13.0][core],
[X: modern examples][examples] and [V: compatibility][compat]. Paths and test/class
selectors below are relative to their indicated root. Test selectors identify
scientific source cases; their presence is not evidence that they pass. Examples
declares version 2.10.0 and an unpinned optional `idaes-pse` dependency in its
[package metadata][examples-project]. Compatibility's `apis/v2_0_0` directory is
an API lineage label, not proof of compatibility with the selected core. Missing
external functions, trainers, solvers or data block execution of a required case;
they do not remove it from the population.

The existing [notebook register](../process-simulator-design/reference/notebook-families.md)
remains the owner of its documented/held/archive entry list. The present crosswalk
preserves every mandatory family in the
[adopted scope](../process-simulator-design/delivery/idaes-scope-and-capability-coverage.md).
It does **not** establish a closed total of scientific configurations or parameterized
test instances: inherited settings, callable configurations, operating envelopes,
full test parameters and transitive input closures are not all resolved. The
specific remaining enumeration work is stated below. No completion percentage can
be calculated from this investigation.

## Population rules

| Disposition | Meaning in this crosswalk |
| --- | --- |
| Required | Released process capability or adopted worked scientific workflow; its configured cases, ordinary inputs and independent evidence remain obligations. Every D–G row below is required unless expressly identified otherwise. |
| Required, held execution | Scientifically selected behavior whose pinned environment or data closure is unresolved; a hold is not an exclusion. |
| Conditional supplemental | Whole-distribution or archived behavior outside the accepted process-only claim; retain its source and selection question. The coordinator owns any adoption/exclusion decision. |
| Excluded source wrapper | Documentation/test/user/exercise/solution rendering or exact duplicate entry point; scientific differences must still be represented once. No exclusion of an underlying scientific case follows. |
| Excluded non-science machinery | Examples' developer cache/tag demonstrations, empty `testnew/foo.ipynb`, and obsolete DMF workspace mechanics are not process-science requirements. This does not exclude scientific data formerly stored there. |
| Native extension | Required native outcome when selected by its owning plan, with independent evidence; it must not be attributed to released IDAES behavior. MHE and broader uncertainty methods use this disposition. |

A population member is a scientific model plus the configuration that changes its
equations, meanings or demanded products, its specifications and input closure.
Changing a notebook suffix, Python initializer API or plot style does not add a
scientific member. Simulation, design optimization, parameter fitting and physical
time operation remain separate case roles even when they reuse one model. Different
property/reaction packages are distinct when they change physics or conventions.
Continuous temperatures, pressures, dimensions and mesh counts require an envelope
and motivated interior/boundary/refinement cases; they are not an unlimited Cartesian
product and cannot be replaced by one unnamed nominal point.

## Shared finite configuration axes

The axes identify demands to map to actual admissible models; they are **not** an
assertion that every combination is supported. Read construction guards and actual
case specifications before adding a member. An enum value can select a refusal.

| Source owner | Categorical population exposed by the pinned source | Required meaning / remaining decision |
| --- | --- | --- |
| [C control-volume types][cv-types] and [0D/1D owners][cv-base] | Material: none, component-phase, component-total, element-total, total; energy: none, enthalpy-phase/total, energy-phase/total, isothermal; momentum: none, pressure-total/phase, momentum-total/phase; explicit/default selection; forward/backward direction | Resolve effective defaults against the selected property package; retain dynamic/steady, holdup, phase equilibrium, reaction/transfer/heat/work/pressure terms and property-supported bases. Enum membership alone grants no implemented balance. |
| [C modular states][states] | `FTPx`, `FcTP`, `FPhx`, `FcPh`, `FpcTP`, `FpTPxpc`; defined/receiving state and phase-component membership | Coordinate reconstruction, phase fractions, flow/enthalpy transport and disappearance/zero-flow behavior. `test_FpTPxpc.py` alone includes one/two/three-phase construction with defined/undefined state and bounds choices; construction fixtures are not independent fluid-validation cases. |
| [C Helmholtz][helmholtz] | `PH`, `PS`, `PU`, `TPX`; `MIX`, `LG`, `L`, `G`; mass/mole basis; per-fluid registration | A supported external-function/data closure is required. Shipped JSON parameter files do not establish that executable `.nl` expressions or their transport functions are installed; inspect [parameter registration][helmholtz-parameters]. Resolve datum, phase/root, saturation and demanded derivatives separately. |
| [C activity coefficients][activity] | Ideal, NRTL, Wilson; selected vapor/liquid presentation and state coordinates | Component interactions, reference state, phase equilibrium and temperature-dependence are part of the model, not replaceable labels. |
| [C cubic EOS][cubic] and [equilibrium methods][phase-equil] | PR/SRK; selected phase root, equilibrium formulation, bubble/dew and Henry-law methods | Freeze EOS options and interaction data; distinguish SmoothVLE/SmoothVLE2 and cubic-complementarity behavior. A local stationary root is not a stable-equilibrium certificate. |
| [C electrolyte EOS][enrtl] | True/apparent states; symmetric/unsymmetric reference-state mechanisms and selected electrolyte chemistry | Preserve charge/speciation, solvent reference conventions and apparent-to-true mappings. Inspect actual example/test configurations before declaring their combinations admissible. |

## D — Scientific assets and generic equipment

`D` owners below refer to [64d](64d-scientific-assets-and-equipment.md). Native
source starting points are [properties.pse](../../packages/reference/domain/models/properties.pse),
[equilibrium.pse](../../packages/reference/thermodynamics/models/equilibrium.pse),
[reactions.pse](../../packages/reference/thermodynamics/models/reactions.pse),
[chemistry.pse](../../packages/reference/physical/models/chemistry.pse), the authored
[process models](../../packages/reference/process/models/) and
[process specialization](../../crates/pse-modeling/src/specialize/process.rs).
These names establish owners to extend, not parity for any row. In the native
column, **starting surface** means source exists but this investigation establishes
no configuration-specific execution, derivative or readback support.

| Demand / owner | Exact upstream model and shipped case selectors | Native source state and prerequisite / ordinary-data / reference gap |
| --- | --- | --- |
| State coordinates, ideal and cubic mixtures — D02 | [C modular examples][property-examples]: `BT_ideal.py`, `BT_PR.py`, `ASU_PR.py`, `HC_PR.py`, `HC_PR_vap.py`, `CO2_H2O_Ideal_VLE.py`, `CO2_bmimPF6_PR.py`; all six state modules above | Starting surfaces: `ideal-caloric-state.pse`, `peng-robinson.pse`, `pr-equilibrium.pse`, `equilibrium.pse`. Exact correlation/interaction/datum records and coordinate-specific consumer checks must be matched; no blanket SRK or six-state support claim. |
| Component caloric and transport methods — D02/D03 | [C pure methods][pure]: ConstantProperties, NIST, Perrys, RPP3/4/5, ChapmanEnskog, ChungPure, Eucken; [mixture transport][transport]: Wilke viscosity, WMS conductivity, explicit no-method | Property starting surface only. Each selected viscosity/conductivity consumer needs phase/component parameters and derivative/domain support; a missing method must remain unavailable. Trace original parameter provenance rather than publish oracle fixture values as ordinary data. |
| Steam, CO2 and general pure-fluid properties — D02 | C `iapws95.py`, `swco2.py`, [general Helmholtz][helmholtz]; base parameter JSONs for butane, CO2, water, isobutane, ammonia, propane, R1234ze, R125, R134a, R227ea and R32; V [IAPWS95/SWCO2 verification][verification] has separate liquid, vapor, saturation and supercritical CSV/test sources | `helmholtz.pse` is a starting surface, not an IAPWS95/SWCO2 implementation claim. Need exact fluid expressions, datum, units/basis, derivatives, native realization and independent ordinary records; reference-only CSVs retain their validation role. JSON presence does not establish executable component registration or transport availability. |
| Electrolyte/speciation and capture properties — D03 | C `enrtl_H2O_NaCl_KCl.py`, `enrtl_NaBr_mixed_solvent.py`; V `eNRTL/test_enrtl_verification_{NaCl,KCl,NaCl_KCl,2_solvent}.py`; [C MEA solvent/vapor][mea-properties] | `aqueous.pse`, chemistry and reaction projection are starting surfaces. Exact reference conventions, electrolyte interactions, true/apparent conservation and MEA transport/kinetic data remain ordinary-source and scientific implementation obligations. |
| Reaction mechanisms — D03 | [C reaction methods][reaction-methods]: Arrhenius, power-law rates; constant/van't Hoff/Gibbs equilibrium constants; power-law/log and solubility-product/log-solubility forms; constant reaction enthalpy; `examples/reactions/reaction_example.py` | `reactions.pse`/`chemistry.pse` starting surfaces. Select basis, participating species/phases and formation-versus-sensible energy once. Precipitation and inherent/equilibrium/rate reactions cannot be merged into specified-conversion evidence. |
| Boundary and representation units — D04 | [C generic unit source][units]: feed/product, feed_flash, statejunction, stream_scaler, translator, skeleton_model. [Tests][unit-tests]: `test_feed_flash.py::{TestBTXIdeal,TestBTIdealModular,TestIAPWS}`, `test_translator.py::TestTranslate`, `test_skeleton_unit_model.py::{TestSkeletonDefault,TestSkeletonCustom}` | `units.pse`/`homogeneous-units.pse` starting surfaces. Feed flash is isothermal/isenthalpic; translator uses independently selected inlet/outlet packages, defined outlet state and phase-equilibrium options. Ordinary inputs and preserved/lost quantities must be declared. These upstream boundary units explicitly forbid dynamic/holdup configurations. |
| Mixing — D04 | C `mixer.py`; tests `test_mixer.py::{TestBTX,TestIAPWS,TestSaponificationScalerObject,TestComponentPhaseWithInherentReactions}` | `mixer.pse` starting surface. Energy mixing none/extensive; pressure none/minimum/equality/switchable minimum-and-equality; inlet list/count, phase equilibrium and inherent reaction cases remain distinct; upstream steady/no-holdup limitation stays explicit. |
| Separation and flash — D04 | C `separator.py`, `flash.py`; tests `test_separator.py::{TestSaponification,TestBTXIdeal,TestIAPWS,TestBTX_IdealSep}`, inherent-reaction split tests; `test_flash.py::{TestBTXIdeal,TestBTIdealModular,TestIAPWS}` | `separator.pse`/`flash.pse` starting surfaces. Total/phase/component/phase-component split and ideal split maps; energy none/equal-temperature/equal-molar-enthalpy/enthalpy-split; pressure/heat/phase-equilibrium choices need valid combinations and exact property closures. No upstream dynamic flash claim. |
| Heater, pressure changer and valve — D04 | C `heater.py`, `pressure_changer.py`, `valve.py`; tests `test_heater.py::{TestBTX,TestIAPWS,TestSaponification,TestBT_Generic}`, `test_pressure_changer.py::{TestBTX_isothermal,TestIAPWS,TestTurbine,TestCompressor,TestPump,TestTurbinePerformance}` and valve linear/equal-percentage/quick-opening classes | `pressure-changer.pse`, `control-volumes.pse` starting surfaces. Isothermal/isentropic/pump/adiabatic, compression/expansion and optional performance curves are separate demands. Curve/callback data, steam/CO2 properties, work sign and pressure-flow laws are prerequisites. |
| HX0D and wall storage — D04/E02 | C `heat_exchanger.py`, `heat_exchanger_lc.py`; tests `test_heat_exchanger.py::{TestBTX_cocurrent,TestIAPWS_countercurrent,TestSaponification_crossflow,TestBT_Generic_cocurrent}`, LMTD/smooth/AMTD/Underwood callback tests; `test_heat_exchanger_lc.py::{TestHXRegression,TestHXLCGeneric}` | `heat-exchanger.pse`/`delta-t.pse` starting surfaces. Co/counter/crossflow, different side properties, rating/design freedoms and wall-only heat accumulation are distinct. Wall heat capacity/resistance/fouling and delta-T behavior need ordinary data and independent evidence; H02 first-case HX is only one selected slice. |
| HX1D, shell-and-tube and NTU — D04/E03 | C `heat_exchanger_1D.py`, `shell_and_tube_1d.py`, `heat_exchanger_ntu.py`; tests `test_heat_exchanger_1D.py::{TestBTX_cocurrent,TestBTX_countercurrent,TestBTX_lagrange_radau,TestBTX_lagrange_legendre,TestIAPWS_cocurrent,TestIAPWS_countercurrent}`, shell/tube analogous classes, `test_hx_ntu.py::TestHXNTU` | Heat/geometry starting surfaces only. Resolve side balance/pressure/phase changes, finite-difference/collocation and shell-hot/cold configuration. HX1D rejects crossflow, different side transformation methods, missing collocation scheme and mismatched collocation schemes; these are refusal obligations, not dropped rows. |
| Reactor families — D03/D04/E02/E03 | C `stoichiometric_reactor.py`, `equilibrium_reactor.py`, `gibbs_reactor.py`, `cstr.py`, `plug_flow_reactor.py`; generic tests saponification, `test_gibbs.py::{TestGibbsInerts,TestMethane}`; X notebook register rows 26–30 | `reactors.pse` starting surface. Specified extent/conversion, equilibrium reaction, Gibbs/element/inert species and kinetic CSTR/PFR simulation/optimization are separate cases. X ethylene-glycol thermo/reaction helper files and steam-methane-reforming reactions must be resolved. Equilibrium/Gibbs reactor upstream dynamic/holdup refusals must remain explicit. |
| Pipe and multistream contacting — D04/E03 | C `pipe.py`, `mscontactor.py`; tests `test_pipe.py::TestSaponification`, `test_mscontactor.py::{TestBuild,TestReactions,TestToyProblem,TestMSContactorInitializerNoHoldup,TestMSContactorInitializerWithHoldup,TestLiCoDiafiltration}` | `control-volume-1d.pse` starting surface explicitly refuses dynamic use without time accumulation. Contactor has per-stream forward/backward direction, feed/side streams, reaction/energy/pressure terms, interacting-stream map and heterogeneous reactions. Toy/dummy property blocks are structural evidence only; Li/Co diafiltration is a distinct scientific consumer. |
| Solid/liquid separation and thickening — D04 | [C solid-liquid source/tests][solid-liquid]: `sl_separator.py`, `thickener.py`; `test_sl_separator.py::TestSLSeparatorBasic`, `test_thickener.py::TestThickener0DBasic`; independent solid/liquid property packages, material/momentum balance and energy-split choices; both steady/no holdup | Generic separator starting surface is not proof of settling-law/solids support. Required settling constitutive law, liquid-recovery freedom, solid/liquid bases, geometry and actual parameter data remain ordinary scientific and consumer obligations. The thickener test's locally defined property blocks are not independent physical-data validation. |
| HDA, methanol and custom authoring — D05/G06 | X register rows 2, 15–17, 24, 31–35; [HDA/methanol modules][example-modules], notebook-local `hda_flowsheets_for_costing_notebook.py`, `methanol_flowsheet.py`, `methanol_flowsheet_w_recycle.py`, `liquid_extraction/{aqueous_property,organic_property,liquid_liquid_extractor,liq_liq_extractor_flowsheet}.py` | Existing composition/property/reaction owners remain authority. Preserve HDA modular-versus-legacy properties, flash-versus-distillation, simulation/design/costing and methanol with/without recycle. Custom compressor/heater and aqueous/organic extraction are user-extension obligations. Helper closure and original coefficient provenance precede ordinary source publication and matched references. |

## E — Initial physical cases and dynamic population

The following are concrete **Proposed first increments** for
[64e](64e-inventory-distributed-and-cyclic-physics.md), not a reduction of its later
configured families. Their source specifications are available without a new
generic simulator framework. Freeze physical inputs and requested output grids at
the existing case/source owners before implementation/reference execution.

| First increment / prerequisite owners | Exact upstream case and physical demand | Native source gap and evidence needed |
| --- | --- | --- |
| Lumped reactive vessel — E02/D03/G05 | [C Caprese CSTR][cstr-model] `make_model(steady=False, bounds=False)` versus steady initialization and bounded controller model. Aqueous-enzyme property/reaction classes come from `core/util/tests/test_initialization.py`; component-total and enthalpy-total balances, no momentum balance, holdup and reaction heat, fixed vessel volume, nonzero initial component/energy holdup; Radau time collocation | `vessels.pse`, `reactors.pse`, `temporal` specialization and `workflow/horizon.rs` are starting owners. Retain the source's actual inventory and controlled-feed meaning; test helper origin does not authorize those coefficients as ordinary data. Need published ordinary science, consistent initial-condition evidence, physical flux/inventory closure, complete trajectories and failure correspondence. |
| Distributed reactive inventory — E03/F03 | [C fixed-bed 1D source][fixed-bed] and `unit_models/tests/test_FB1D.py::{TestIronOC,TestIronOC_reverse_flow,TestIronOC_NoReaction,TestIronOC_conservation_with_reaction}`. Gas transport plus stationary solid inventory, methane/iron reduction, pressure drop, time and length domains; compare reaction-free and reactive conservation | `control-volume-1d.pse` explicitly requires a separate dynamic accumulation closure; existing source alone cannot serve this case. D03 heterogeneous chemistry/transport and F03 bed correlations are prerequisites. Freeze boundary/initial conditions, pressure law, spatial/time discretization, requested physical locations and independent grid refinement. |
| Physical adsorption cycle — E04/F05 | [C TSA source/tests][tsa]. First zeolite case: `TestTsaZeolite`, 120 beds, no compressor or steam calculation, Radau within-stage time discretization (`finite_elements=20`, `collocation_points=6`); heating/desorption, closed cooling, feed repressurization and adsorption/collection. Other adsorbents remain required below | Existing `vessels.pse`, `control.pse` and horizon/study owners are starting surfaces; an ordinary study occurrence is not physical carryover. Need actual bed inventory/isotherm/thermal laws, collection windows, stage transitions, closure and complete initial/final inventory accounting; no cycle qualification from the source model's `FlowsheetBlock(dynamic=False)`. |

Remaining required dynamic consumers include HX wall storage; CSTR and PFR
configurations; moving/fixed/bubbling beds; steam tanks/PID; SOC local thermal and
gas storage; gas-pipeline linepack; the Chemical Akzo Nobel DAE; and held NETL-32D
adsorption/desorption. Their E02/E03/E04 prerequisites are distinct. The Chemical
Akzo Nobel case in X `docs/dae/petsc_chem.ipynb` consumes C
`core/solvers/features.py::dae`; steam PID consumes X
`mod/dae/petsc/pid_steam_tank.py`, IAPWS95, heaters, valves and controller semantics.
The PID notebook's generated-test skip metadata is a reference-execution gap, not
permission to remove the workflow. Event/reset sensitivity and arbitrary hybrid
changes are not established by these smooth-path source examples.

## F — Specialized configured families

`F` owners refer to [64f](64f-specialized-process-families.md). Their current native
support state is **starting surfaces only** at the authored scientific/equipment
owners named above; this inspection supplies no complete specialized native model
or reference qualification. D02/D03 supply exact properties, reactions and transport;
E02/E03/E04 supply only the storage/distribution/cycle capability each row consumes.

| Required scientific demand / owner | Exact source configurations and shipped consumers | Data / route / independent-reference prerequisite |
| --- | --- | --- |
| Steam/power equipment and integrated plant — F02 | [C power source][power]: `unit_models/{boiler_fireside,boiler_heat_exchanger,boiler_heat_exchanger_2D,heater_1D,cross_flow_heat_exchanger_1D,heat_exchanger_3streams,drum,drum1D,downcomer,waterwall_section,waterpipe,watertank,steamheater,feedwater_heater_0D,feedwater_heater_0D_dynamic,cpu,balance}.py`; `unit_models/helm` turbine inlet/stage/outlet/multistage, pump/compressor/mixer/splitter/steam valve/phase separator/condenser | Source-tree coverage is recorded, but per-unit categorical/test-instance expansion remains open. Preserve subcritical boiler/plant, supercritical steam cycle and SCPC full plant as distinct integrated cases, not a steam property evaluator witness. Requires correlation/surrogate data, geometry, steam phase presentation, pressure and work/heat sign, operating envelope and costing inputs. |
| NGCC — F02 | X `docs/power_gen/ngcc/ngcc.ipynb` → `mod/power_gen/ngcc.py` → `gas_turbine.py`, `hrsg.py`, `steam_turbine.py`; C natural-gas PR/flue-gas and Helmholtz steam units | Shipped `ngcc_init.json.gz`, `gas_turbine_init.json.gz`, `hrsg_init.json.gz`, `steam_turbine_init.json.gz` are reference initialization dependencies, not ordinary scientific truth. Notebook load sweep and generated tabulations are separate from initial design. Need exact serializer/core compatibility, load specifications and full subflowsheet/plant outputs. |
| Fixed bed 0D — F03/E02 | C `gas_solid_contactors/unit_models/fixed_bed_0D.py`, `tests/test_FB0D.py::{TestIronOC,TestIronOC_EnergyBalanceType}` and `flowsheets/dyn_TGA_example.py` | Upstream requires dynamic=True and holdup=True. Gas/solid packages, reaction basis, initial solid inventory and enthalpy-total versus omitted-energy behavior need separate evidence; this is not a generic steady vessel. |
| Fixed bed 1D — F03/E03 | [C fixed bed][fixed-bed]; forward/reverse flow, simple/Ergun pressure-drop law, heat/phase/reaction/energy options; `test_FB1D.py` selectors in E table | Reverse flow with collocation is rejected upstream. Finite-difference flow direction requires compatible upwind scheme; wrong direction/scheme is a refusal. Required native dynamic accumulation, reaction/no-reaction cases and original integrated conservation are not established by this investigation. |
| Moving bed — F03/E03 | C `gas_solid_contactors/unit_models/moving_bed.py`; `tests/test_MB.py::{TestIronOC,TestIronOC_EnergyBalanceType,TestIronOC_TransformationMethod,TestBidirectionalSpatialDiscretization}`; `flowsheets/ss_MB_methane_combustion.py` | Countercurrent is the declared flow type; gas/solid schemes can differ for opposing directions. Simple/Ergun pressure drop and energy-free/enthalpy configurations need exact hydrodynamic and transport closure. Diagnostic moving-bed example uses separate X local model/property modules and cannot be treated as an alias of this scientific case. |
| Bubbling fluidized bed — F03/E03 | C `bubbling_fluidized_bed.py`; `tests/test_BFB_methane_oc.py` and `test_BFB_oxygen_oc.py`, each with IronOC, energy-balance and transformation-method classes; `flowsheets/ss_BFB_methane_combustion.py`, `ss_BFB_OC_oxidation.py` | Separate methane/iron reduction and oxygen/iron oxidation gas/solid/reaction packages, co/countercurrent settings, discretization, bubble/emulsion transfer and inventory source accounting. Ordinary transport/hydrodynamic coefficients and full combined balances must not be inferred from a successful bed shell. |
| Equilibrium column and auxiliaries — F04/D04 | [C columns][columns]: tray/feed/side draws, total/partial condenser and bubble-point/custom temperature, reboiler boilup/heat specifications, TrayColumn; tests include `test_tray_column.py::{TestBTXIdealFcTP,TestBTXIdealFTPz,TestBTXIdealGeneric}` and separate tray/condenser/side-draw tests | Steady/no-holdup constraints remain explicit. Number/feed-tray position and pressure/heat/side-draw settings change the case. Preserve HDA distillation's property/port/specification mapping; different state coordinates do not eliminate model-consumer checks. |
| Rate-based solvent/MEA capture — F04/D03 | C `solvent_column.py`, `MEAsolvent_column.py`, `solvent_condenser.py`, `solvent_reboiler.py`, enhancement-factor model and `properties/MEA_{solvent,vapor}.py`; `test_solvent_column.py::{TestAbsorberColumn,TestStripperColumn}`, `test_MEAsolvent_column.py::TestAbsorber`; solvent-reboiler parameterization has vapor-flow/heat-duty specification × old/new initialization API | Scientific specifications produce two demands; API wrappers do not double their scientific count. Need exact liquid/vapor, enhancement, transport, reaction heat and true/apparent mappings. X advanced scaling loads `advanced_scaling_failed_solution.json`; expected diagnostics and solved physical outputs are distinct evidence. |
| Plate exchanger and membrane — F04 | C `column_models/plate_heat_exchanger.py` passes/channels/divider plates; [membrane source/test][membrane] co/countercurrent, sweep/no-sweep, energy/pressure balances and finite elements; `TestMembrane` uses CO2/H2O/N2 ideal gas, countercurrent steam-rich sweep | Test config checks include both directions, but the inspected solved fixture is countercurrent. Permeance/selectivity, feed/sweep states, geometry and coupled material/enthalpy balances are required. Cocurrent or no-sweep behavior cannot be qualified by that fixture. |
| TSA zeolite — F05/E04 | [C TSA][tsa] `TestTsaZeolite`: zeolite_13x, fixed bed count 120, no compressor, steam=none; X TSA notebook | Isotherm/thermal/source coefficients and cycle operation are required ordinary scientific inputs. Compare product streams, recovery/purity, stage times, pressure drop, cycle inventories and energy; notebook presence does not qualify the cycle. |
| TSA MgMOF — F05/E04 | C `TestTsaMgmof`: mmen_mg_mof_74, bed count calculated (`None`), compressor=False, simplified steam | A different adsorbent law, bed-sizing freedom and steam approximation; not an additional rendering of zeolite. Record ordinary isotherm provenance, heat conditions, sizing role and requested steam products. |
| TSA polystyrene — F05/E04 | C `TestTsaPolystyrene`: polystyrene_amine, one bed, compressor=True with flue-gas properties, rigorous steam with IAPWS95 | Adds compressor work/efficiency and steam-heater property closure; no substitution by simplified steam. DAC electric-boiler/NGCC-retrofit costing cases depend on separate JSON coefficient files in `temperature_swing_adsorption/costing`. |
| Solid oxide cell/module/SOEC — F05/E03/G05 | [C SOC source][soc]: conductive/porous slabs, channel, triple-phase boundary, contact resistor, cell and simple module; `soec_design.py`; X `soc_pid_control.ipynb` → `mod/power_gen/soc_dynamic_flowsheet.py` | Preserve thin/resolved fuel electrode, oxygen electrode, electrolyte/interconnect; co/counterflow, gas holdup, contact resistance, heat loss, custom voltage and kinetic modifier choices. Thin region plus its x-grid is rejected upstream. X operating scenarios consume `soec_flowsheet_operating_conditions.csv` and `max_production.json.gz`/`power_mode.json.gz`; other shipped initializations must be dispositioned. Full per-option/test population and ordinary electrochemical data remain open. |
| Gas network — F06/E03 | [C gas distribution][gas-network]: natural-gas properties, compressor/node/pipeline; `test_pipeline.py::TestSolvePipelineSquare` nominal/sweep, `TestSolveDynamicPipeline` inlet-pressure/outlet-flow simulation and optimization; `test_pipeline_compressor.py` dynamic coupling; node supply/demand and inlet/outlet counts | Exact gas composition/conventions, isothermal/compression law, pressure-flow boundary roles and temporal/spatial discretization are required. Forward/backward/Radau/Legendre construction tests do not all establish transient solutions. Negative signed flow alone grants no composition/enthalpy reversal semantics. Linepack and network balance require E03. |

The pinned power-unit declarations expose the following additional finite axes.
They refine F02's required source population; compatibility with inherited settings
and complete shipped operating cases still needs case-level reconciliation.

| Power unit owner under C `models_extra/power_generation/unit_models` | Declared scientific configurations to preserve |
| --- | --- |
| `boiler_fireside.py` | Zone count, platen/roof superheater presence, calculated primary/secondary-air flows, selected surrogate dictionary; dynamic/holdup settings require actual construction support. |
| `boiler_heat_exchanger.py`, `boiler_heat_exchanger_2D.py`, `cross_flow_heat_exchanger_1D.py` | Inline/staggered tubes, liquid/vapor water, radiation on/off; 2D co/countercurrent, header on/off and wall/header radial mesh; crossflow shell-hot/cold. Geometry and numerical discretization have different meanings. |
| `feedwater_heater_0D.py`, `feedwater_heater_0D_dynamic.py` | Drain mixer, desuperheat and drain-cooling sections independently selected; condensing/desuperheating/cooling side closures; actual dynamic wall/fluid inventory is separately qualified. |
| `waterpipe.py`, `watertank.py`, `waterwall_section.py`, `steamheater.py` | Pipe-end none/contraction/expansion and liquid/vapor; simple/rectangular/vertical-cylindrical/horizontal-cylindrical tank; rigorous boiling on/off; single-sided heat exposure on/off; selected heat/pressure and holdup settings. |
| `heat_exchanger_3streams.py`, `heater_1D.py`, `drum1D.py` | Independent side-2/side-3 co/counterflow; heater tube arrangement and fluid-holdup refusal; wall radial geometry/discretization and pressure/heat settings for drum1D. |
| `helm/turbine_multistage.py`, `helm/valve_steam.py` | HP/IP/LP and parallel-inlet stage counts, splits/disconnections/outlet counts and throttle curve; steam valve liquid/vapor and linear/equal-percentage/quick-opening law. Stage counts and split locations require finite authored case selections, not all integers. |
| `soec_design.py` | Selected reaction EOS and heat-transfer inclusion; configured reaction/property mapping and actual design specifications remain necessary. |

## TSA0D configured census and input closure — 2026-11-05

**Interface-checked, source only:** this slice reads C's entire
`models_extra/temperature_swing_adsorption` Python/test/costing directory, its
inherited `UnitModelBlockData`/`ProcessBlockData` configuration, the consumed
flue-gas/steam/costing owners, and X's four TSA notebook renderings at the fixed
revisions above. Native correspondence was inspected at `85868b3b4`. It closes
the source selection and shipped-case inputs below, not TSA implementation,
reference execution, ordinary-data admission or the rest of H01. All rows remain
required. The held NETL-32D 1D adsorption/desorption case is a separate model.

### Effective configuration and refusals

| Axis / actual source owner | Effective choices and construction boundary |
| --- | --- |
| Inherited `dynamic`, `has_holdup` | Both start as `useDefault`; dynamic inherits the parent and holdup resolves to dynamic. A dynamic unit in a steady parent and dynamic=True/holdup=False are refused by the base. The shipped TSA cases use a steady parent, hence False/False. TSA's heating/cooling have their own normalized time domains and retained stage states; these exist even when the parent is steady. TSA does not add a generic parent-time accumulation law: its inlet composition/pressure coupling samples the parent's final time. True/True and steady/holdup=True inheritance therefore do not establish additional transient scientific cases or native support. |
| Adsorbent | Default `zeolite_13x`; alternatives `mmen_mg_mof_74`, `polystyrene_amine`. Each builds a different loading law/parameter set. All use CO2/N2 adsorption; external ports additionally expose H2O/O2. Port membership does not make them adsorbing species. |
| Bed specification | Default integer 120 fixes adsorption-bed allocation; `None` selects calculated allocation with inlet velocity free and an independently fixed pressure drop. It is not an enumeration of every integer or an integer bed-design optimization. The reported total bed count also includes regeneration duty; it is distinct from this configured adsorption-bed count. |
| Stage discretization | Actual defaults are collocation/Radau, 20 finite elements and 6 collocation points. Explicit `useDefault` resolves to backward finite difference or Radau collocation. Finite difference permits backward/forward; collocation permits Radau only. Other method/scheme pairs are refusals. In particular finite difference with an omitted scheme retains the actual Radau default and is refused; the scheme's descriptive default text is not authoritative. Missing element/point values are checked in the transformation owner; those checks follow transformation invocation, so do not promise a particular first error. Zero/negative mesh sizes have no TSA-specific admissibility proof. |
| Compressor | Default False. True requires an explicit property package; the shipped package has only N2/CO2 and feeds an isentropic pressure changer. Pressure rise offsets the TSA pressure loss; efficiency is a separate fixed specification. An explicitly supplied incompatible package is not certified by the presence-only guard. |
| Steam | Default `none`; `simplified` builds a fixed algebraic steam-demand correlation without a trainer/artifact; `rigorous` requires an explicit steam package and builds a heater. Shipped rigorous cases use IAPWS95, inlet 510000 Pa and 52465.8 J/mol, zero pressure change, and an outlet vapor fraction of 1e-9. Missing rigorous properties are refused; no guard proves arbitrary supplied packages compatible. |
| Balance / flow policies | There is no user-selectable material/energy/momentum balance enum, reversed-bed direction, sweep or axial grid for this shortcut. Heating exchanges matter and heat; cooling is closed; repressurization uses feed; adsorption ends at breakthrough. Their fixed equations must be preserved rather than acquiring unrelated control-volume axes. |

These are factorized categorical choices with guards, not a claim that their
Cartesian product has been physically validated. Uninstantiated cross-combinations,
forward-difference numerical behavior and inherited parent-time settings retain
separate readiness/acceptance questions; the six concrete configurations below
do not discharge them.

### Shipped specifications, numerical variants and assessment roles

All flows are mol/s, pressures Pa, temperatures K and lengths m. All six
configurations fix inlet H2O=O2=0 and pressure=100000. The temperature tuple is
(inlet, adsorption, desorption, heating utility, cooling utility); utility
temperatures are not the stage terminal state. Unless overridden, the mesh is
20-element/6-point Radau collocation. No `pytest.mark.parametrize` expansion is
present in the three TSA test modules; scientific differences are fixture classes,
the UI override and the notebook specifications.

| Exact selector / case role | Complete independent operating/design specifications | Additional products / initialization selection |
| --- | --- | --- |
| C `tests/test_fixed_bed_tsa0d.py::TestTsaZeolite` | Zeolite; 120 beds; CO2=40, N2=99960; temperatures (303.15,310,470,500,300); diameter=4, height=8; compressor off; steam none | Cycle and stream products; heating/cooling initial time guesses 70000/110000 s. |
| C same file `TestTsaMgmof` | MgMOF; beds=None; CO2=0.001152, N2=0.008448; temperatures (300,310,430,440,300); diameter=0.03, height=1.2; pressure drop=-4000; compressor off; steam simplified | Calculated bed allocation/velocity and steam flow; time guesses 2000/900 s, initializer constraint tolerance 1e-4. |
| C same file `TestTsaPolystyrene` | Polystyrene; 1 bed; same feed rates as MgMOF; temperatures (300,440,445,450,435); diameter=0.1, height=1.2; compressor on, N2/CO2 flue-gas package, efficiency=0.8; rigorous IAPWS95 steam | Compressor work/pressure rise and steam flow/heater states; time guesses 700/700 s. |
| X `docs/flowsheets/temperature_swing_adsorption/temperature_swing_adsorption.ipynb`; C UI base/solve/output tests | Zeolite; 1 bed; MgMOF feed rates and temperatures; diameter=0.03, height=1.2; compressor off; steam none | Distinct from the 120-bed fixture. Notebook uses user scaling and Ipopt tolerance 1e-6; initializer default time guesses 1000/500 s. UI has the same physical specification and default mesh, but retains its own solver settings. |
| C `tests/test_fixed_bed_tsa0d_ui.py::test_build_with_finite_elements` | Same UI physical specification; backward finite difference with 10 elements; collocation-point option remains 6 but is not a finite-difference order | A construction/refinement configuration, not a demonstrated transient solve. Forward difference remains an exposed numerical configuration without a shipped solved fixture here. |
| C `tests/test_fixed_bed_tsa0d_costing.py::{TestElectricBoilerCosting,TestRetrofitNgccCosting}` | Zeolite; 600 beds; 120-bed fixture feed, temperatures and geometry; compressor on with N2/CO2 flue-gas package and efficiency=0.8; rigorous IAPWS95 steam | One physical configuration, two distinct assessment roles: `electric_boiler` and `retrofit_ngcc`. Both initialize at default time guesses; final solver sets bound_push=1e-6. Preserve each role's capital, fixed/variable O&M, annualized cost and cost-of-capture products. Unknown costing-case strings are refused. |

The X base and `_test` code cells have the same parsed statements. After removing
assertions, `_doc`/`_usr` differ only by absent assertion-only pytest and
termination-condition imports. Thus these four wrappers supply one scientific
specification, while retaining their distinct evidence/rendering roles. The SVG
and directory index are documentation, not coefficient inputs. The UI's graphical
processor is not required to reconstruct the native scientific problem; executing
its adapter tests separately requires `idaes_flowsheet_processor.api`. Upstream
skip/importorskip behavior is not an accepted native qualification outcome.

### Recursive prerequisite closure and input roles

| Selected consumer | Complete case-specific source/artifact dependency and unresolved readiness |
| --- | --- |
| All six configurations | C `fixed_bed_tsa0d.py` plus `initializer.py`, base units/core scaling/math/serializer and Pyomo DAE/NLP support; `util.py` adds pandas/matplotlib for summaries/profiles. General wall capacity, gas viscosity, molecular weights, sphericity and fluidization voidage are inline parameters. Adsorbent builders supply voidages, solid heat capacity/density, heat transfer, particle diameter, adsorption heats and reference temperatures, plus their distinct isotherm coefficients. No case reads an external dataset, trained weight, trainer executable or serialized starting solution. Initializer serializer dictionaries are created/restored in memory; its optional JSON initial-guess API is not selected by these fixtures. |
| Adsorbent law/data | Zeolite builder supplies saturation/affinity/heterogeneity and their temperature dependence; MgMOF supplies lower/upper capacities and affinities, step width/pressure/enthalpy and weighting; polystyrene supplies saturation, affinity and Toth parameters. Original-source leads below identify scientific provenance. All coefficient values, conventions and units still need independently sourced ordinary records at B/D/F owners; reading these reference coefficients does not grant ordinary production eligibility. Expected values in tests/notebook remain oracle-only. |
| Compressor configurations | C `power_generation/properties/flue_gas_ideal.py` and generic `pressure_changer.py` with ideal flue-gas coefficients, state/enthalpy/entropy and isentropic pressure/work closure; no external cubic-root or trainer artifact is selected. Only the explicit N2/CO2 composition is selected here. Original ordinary coefficient provenance remains D02/F05 work. |
| Rigorous steam configurations | C `iapws95.py` → general Helmholtz parameter/state/expression owners → registered Water component, its parameter JSON and generated `h2o_expressions_eos.nl` with expression metadata, plus loadable `general_helmholtz_external` library. The registration path and exact installed artifacts must be frozen and checked before execution. Water JSON alone is insufficient. This heater demands thermodynamic saturation/enthalpy/mass flow, not a blanket qualification of Water viscosity, conductivity or surface tension. |
| Both costing roles | C `costing/dac_costing.py` reads its matching `costing_params_dac_electric_boiler.json` or `costing_params_dac_retrofit_ngcc.json`. It consumes QGESS `power_plant_capcost.py`, `costing_dictionaries.py`/`BB_costing_data.json`, and `generic_ccs_capcost_custom_dict.py`/`generic_ccs_costing_data.json`. The sCO2 dictionary loader is imported but not called by these DAC cost-account routes. Capacity/utilization, USD_2018 prices/currency conversions, inline resource/emissions correlations and labor/land/account assumptions are assessment inputs, not adsorption material data. Neither role trains these correlations. |
| Actual execution environment | Core remains IDAES2.13.0 with Pyomo6.10.1/Pint selected by `pyproject.toml` and its lock; initializer explicitly selects legacy `ipopt`, not `ipopt_v2`. Pin exact interpreter, Ipopt executable/options and selected Helmholtz binary/data identity at execution. No environment was prepared or changed for this census. X2.10.0 metadata is not compatibility evidence for this notebook against core2.13.0; its source inputs are resolved but that environment remains unqualified. |

### Native handoff and remaining evidence

E04/F05 must author the adsorbent loading/thermal laws and explicit heating,
closed-cooling, repressurization and adsorption states using the existing temporal,
inventory and operating-program owners. `vessels.pse::HomogeneousInventory`/`Vessel`,
`control-volumes.pse`, `control.pse` and `workflow/horizon.rs` are enabling surfaces;
inspection of these process declarations and specialization owners found no TSA
science or complete cycle case. This bounded search is not a whole-tree absence
proof. D02 supplies the exact compressor/Water closure; G06 consumes physical
outputs for each costing role. No native TSA declaration/case/output identity is
yet available for a matched reference exchange; that is a concrete blocker rather
than permission to substitute a generic vessel or an independent study sequence.

Each case must map all four port streams (component flows, temperature, pressure),
the full `get_var_dict` summary population and, when selected, compressor/steam
products. Retain stage endpoint composition/loading/temperature/pressure,
heating/cooling profiles at requested physical times, all four durations, total
cycle time, purity/recovery/productivity, per-bed/total heat and captured amount,
bed allocation, fluidization/pressure loss and annual throughput/emissions. Summary
pressure loss is the negative of the signed equation variable. Reported annual
quantities require the same cycle/utilization convention. Original component and
energy integrals, stage continuity and full cycle closure remain required checks
beyond the upstream scalar assertions.

The shipped cases select operating points, not certified continuous validity
envelopes. Before native acceptance, D/F must admit source-backed loading/thermal
domains containing each selected temperature/partial-pressure path, and select
boundary/interior and independently refined stage-time meshes. Do not infer a
global pressure/temperature or zero-feed claim from variable domains. Wrong
adsorbent/steam formulation, missing stage output, incompatible property package,
invalid mesh policy and broken closed-cooling inventory require discriminating
refusals or failures. Forward finite difference, additional admissible categorical
combinations, ordinary coefficient admission, actual 2.13.0 reference runs and
native physical/assessment acceptance remain open. This slice resolves six shipped
physical/numerical configurations and two assessment roles; it does not close the
family's full configuration qualification or H01.

## Gas-network configured census and input closure — 2026-11-05

**Interface-checked, source only:** C `models_extra/gas_distribution` contains one
pseudo-component property owner and compressor, pipeline and node owners. This
slice inspected all their source/test files, the inherited unit/CV1D settings and
the actually consumed `apps/nmpc` cost/input helpers. It expands E03/F06/G05's
finite cases without claiming a native gas network, a solved reference or a full
configuration denominator. Paths in the following tables are relative to that
core directory, at the fixed C revision. No pytest parameter decorators add hidden
cases here; loops and helper defaults produce the enumerated selections.

### Scientific configuration and inherited boundaries

| Owner / axis | Effective configuration and guard/refusal obligations |
| --- | --- |
| Shared unit inheritance | `dynamic` inherits the parent; `has_holdup` defaults to the resolved dynamic flag. Base guards reject dynamic-in-steady-parent and dynamic without holdup. Pipeline passes its complete config to CV1D, including an explicit property package; compressor/node create time-indexed state blocks directly and add no storage law even when their parent is dynamic. Their inherited holdup flag is not evidence of compressor or junction inventory. |
| Property/phase compatibility | All three require one phase named `Vap`; pipeline/node additionally check pressure metadata. These guards do not certify all subsequently demanded density, sound-speed, heat-capacity or enthalpy-flow terms. The shipped `NaturalGasParameterBlock` has one empirical pseudo-component, not chemically exact methane or a resolved natural-gas composition. The compressor test's methane name does not change its actual property selection. |
| Pipeline equations | Always component-phase material balance, pressure/momentum equation and spatially isothermal temperature; no selectable energy/reaction/heat-transfer law. Dynamic parent adds material storage and mass-flow time derivative. The dynamic momentum decision reads parent dynamic, so a unit-level override is not independent proof of consistent transient behavior. Geometry is diameter/area and physical length; fixed roughness default is 0.025 mm and friction depends on diameter. Preserve the signed flow-times-absolute-flow resistance without inferring reversible port composition or thermal semantics. |
| Spatial approximation | Defaults: finite difference, FORWARD, 1 element, collocation_points=None. Source's dynamic endpoint policy admits FORWARD, BACKWARD and LAGRANGE-RADAU, deactivating the appropriate initial-time boundary momentum equation; CENTRAL and LAGRANGE-LEGENDRE are explicitly refused for dynamic pipelines. Actual method/scheme compatibility also goes through CV1D/Pyomo. Radau construction fixture supplies 2 elements/2 points. A steady Legendre/CENTRAL construction is not an inspected solved scientific case. |
| Compressor science | Always isothermal with component-phase inlet/outlet equality, pressure boost, heat-capacity-ratio exponent and compression-power relation. Boost and power are bounded nonnegative. There is no efficiency option, turbine/expansion configuration or compressor material/energy storage in this owner. This is different science from the isentropic TSA pressure changer. |
| Node topology | Defaults: 1 inlet, 1 outlet, 0 supply, 0 demand; explicit counts create indexed ports. Algebraic pressure/temperature/composition coupling, total/component flow and enthalpy mixing apply at each time. Connection to an out-of-range port or a second pipeline at an occupied port is refused. Integer configuration domains do not establish every integer as a valid/physically meaningful network. Counts and topology below are actual finite cases, not an arbitrary-topology guarantee. |

### Executable scientific selections and actual finite sweeps

All selected scientific cases use pseudo-component mole fraction=1. Pipeline
geometry is diameter=0.92 m, length=300 km and temperature=293.15 K unless the
construction-only table states otherwise. Time is in hours. Spatial FORWARD
finite difference is the default; dynamic solved cases use 4 spatial elements
and backward time difference on [0,20] with 20 time elements (21 output times).
`fix()` without a numeric argument below consumes values from the independently
solved steady initialization; it does not supply zero or a new arbitrary default.

| Exact shipped selector | Finite operating/input population and specification freedoms | Required result correspondence |
| --- | --- | --- |
| `unit_models/tests/test_compressor.py::TestCompressorValues::test_calculate_power` | 48 explicit combinations: inlet pressures {30,40,50,60} bar × boosts {2,5,10} bar × standard-volume flows {300000,400000,500000,600000} m3/h; T=293.15 K. Standard-volume→mass uses 0.72 kg/m3, then divide by 18 kg/kmol for the molar input. | Power at every combination, pressure rise, component flow and temperature equality. The inline expected-power table is oracle evidence, not an ordinary property source. |
| `unit_models/tests/test_pipeline.py::TestSolvePipelineSquare::test_pipeline_steady_forward_nominal` | 1 spatial element; inlet pressure=57 bar and flow=300000 kg/h fixed | Nominal feasibility/termination, outlet pressure/flow and full spatial original balance. This flow is distinct from the sweep below. |
| Same class `test_pipeline_steady_forward_sweep` | 27 explicit combinations: spatial elements {1,2,5} × inlet pressures {40,57,70} bar × inlet mass flows {180000,270000,450000} kg/h. The last axis comes from {6,9,15} million standard m3/day using 24 h/day and 0.72 kg/m3. | All outlet pressure/flow pairs, infeasibility/termination and mesh-specific correspondence; the inline target dictionaries must remain oracle-only. These finite pressure/flow limits are selected samples, not proof of their whole enclosing rectangle. |
| Same file `TestSolveDynamicPipeline::test_sim_inlet_pressure_outlet_flow_nominal` | Steady seed at inlet 57 bar/300000 kg/h; `has_holdup=True` in seed. Dynamic boundary fixes inlet pressure/composition/T and outlet flow; initial pressure fixed except inlet and initial flow except outlet. Load outlet=360000 kg/h on (4,20] h; earlier times including 4 h retain seed values. | Inlet-flow and outlet-pressure series at all 21 times, plus linepack and original space/time material/momentum checks. The left-open/right-closed boundary follows the helper's actual loading semantics, not interpolation from the two supplied points. |
| Same class `test_opt` | Same 300000 kg/h seed; separately solved target at inlet 57 bar/500000 kg/h. Fix initial pressure and flow at every space point; inlet composition/T fixed. Optimize inlet pressure with 2 h piecewise-constant intervals and unrestricted outlet-flow decisions on the 1 h grid. | Both endpoint pressure and flow trajectories; sum of weighted squared errors at grid times, weights 1e-2 for pressure and 1e-10 for mass flow. It is a discrete tracking sum, not an integrated continuous-time objective or a general constrained NMPC demonstration. |
| `unit_models/tests/test_pipeline_compressor.py::TestSolveDynamicPipelineCompressor::test_sim` | Upstream compressor→pipeline, same geometry/grid/boundary policy. Seed fixes compressor inlet=50 bar, boost=7 bar and pipeline inlet flow=300000 kg/h. Inlet T/composition fixed; boost remains 7 bar; outlet-flow change is 360000 kg/h on (4,20] h. | Same two pipeline trajectories plus compressor power at all 21 times, arc consistency, and the selected initial boundary momentum deactivation. The upstream pressure/power products make this a distinct configured consumer. |

The helper `make_steady_model` defaults (2 spatial elements, FORWARD and explicit
holdup=True) are overridden to 4 elements by all three dynamic scientific cases;
the optimization additionally overrides target mass flow. The initial state and
target are computed by the independent reference, flattened to in-memory values
and loaded at every dynamic time; no shipped initialization file is consumed.
Time inputs use `dynamic_data/interval_data.py`: right-endpoint interval value,
exclude interval's left boundary and include its right boundary. A missing grid
endpoint makes that helper skip loading; native acceptance must verify actual
boundary coverage instead of accepting an unchanged-input trajectory.

### Structural configurations and required topology controls

These source cases intentionally establish construction, units, degrees of freedom
or matching, rather than additional accepted scientific trajectories. Their exact
fixtures remain available to the native contract/refusal owners. Unspecified
`fix()` values in these construction tests are source initializers, not newly
admitted scientific operating records.

| Source/test selectors | Complete distinct construction selection |
| --- | --- |
| `test_compressor.py::TestSimpleCompressor::test_compressor_methane` | Actual pseudo-gas package, steady; inlet 300 K/20 bar/1000 kmol/h, boost 10 bar, mole fraction=1. No solve is invoked. |
| `test_pipeline.py::TestConstructPipeline::{test_geometry,test_momentum_balance_steady,test_steady_forward}` | Default steady geometry link (area fixed at 5 m2), steady momentum incidence, then 2-element FORWARD steady pipeline with diameter 0.92 m, length **300 m**, T=300 K, inlet=57 bar/300000 kg/h. Preserve the short physical length rather than silently changing it to the scientific cases' 300 km. |
| Same class `{test_dynamic_forward,test_dynamic_inlet_pressure_outlet_flow}` | 4 spatial elements, [0,20] h with 5 backward time elements; respectively inlet pressure+flow boundaries versus inlet-pressure/outlet-flow boundaries, with corresponding independent initial states. Both are structural checks; the former is not the shipped stable solved transient. |
| Same class `{test_dynamic_dof_forward_space,test_dynamic_dof_backward_space,test_dynamic_dof_radau_space,test_dynamic_dof_legendre_space}` | [0,20] h/2 backward time elements, 2 spatial elements; FORWARD, BACKWARD, or Radau with 2 collocation points and inlet-pressure/outlet-flow boundary roles; Legendre 2-point request is the explicit dynamic refusal control. |
| `test_pipeline_compressor.py::TestConstructPipelineCompressorFlowsheet::{test_steady,test_dynamic}` | One compressor→one pipe; 2 spatial elements steady versus 4 spatial elements/[0,20] h/5 backward time elements dynamic. Fix geometry and boundary/initial freedoms; neither is an extra solved compressor transient. |
| `test_node.py::TestConstructNode::{test_construct_node_with_pipelines,test_pipeline_with_two_nodes,test_multiple_inlets_outlets}` | Steady selections: pipe→default 1-in/1-out node→pipe; single pipe between supply-only node (0 in/1 out/1 supply/0 demand) and demand-only node (1/0/0/1); and a node with (3 in,4 out,1 supply,2 demands) attached to seven pipes. Pipes have 2 spatial elements. |
| Same class `{test_dynamic_two_nodes,test_dynamic_multiple_inlet_outlet}` | The latter two topologies above, [0,20] h/4 backward time elements and 2 spatial elements; boundary/initial pressure and flow roles are fixed per topology. Algebraic node mixing is not node inventory. |
| `test_flowsheet.py::TestConstructFlowsheets::{test_four_nodes_linear,test_two_supply_one_demand,test_one_supply_two_demand}` | Each has four nodes/three pipes, 2 spatial elements, [0,20] h/2 backward time elements. Linear chain has supplies at nodes0/2, demands1/3 and compressors upstream of pipes0/2. Converging topology has supplies0/1→junction2→demand3, compressor upstream pipe2. Diverging topology has supply0→junction1→demands2/3, compressors upstream pipes1/2. They test topology/structural matching, not solved market/network operation. |
| `properties/tests/test_natural_gas.py::TestNaturalGasPropertyPackage` | Build/property surface, nominal density, compressibility and reference-temperature controls plus printable state; these do not add independent multicomponent physical cases. |

### Recursive inputs, native correspondence and remaining work

The selected reference closure is C's three unit modules and `properties/natural_gas.py`,
core unit/CV1D geometry/material/state infrastructure, Pyomo transformations/network
arcs/incidence/parameter sweeper, and, for dynamic cases, `apps/nmpc/cost_expressions.py`
and `dynamic_data/interval_data.py`. All scientific coefficients, expected tables,
boundary arrays and topology are inline. The inspected owners perform no external
dataset, trained-model, JSON starting-state or trainer load; no PETSc, Helmholtz,
cubic-root or external property function is selected. Scientific solves use Ipopt;
the exact executable/interpreter/locked environment remains to be frozen and run.
Core2.13.0/Pyomo6.10.1 pins remain unchanged. No claim about all X/V gas-related
sources or archived applications follows from this core-directory closure.

Ordinary property inputs must preserve empirical identity: molecular weight
18 kg/kmol, standard density 0.72 kg/m3, compressibility 0.80, constant mass heat
capacities 2.34/1.85 kJ/(kg K), reference temperature 298.15 K, sound-speed
convention and pipe roughness/friction law. The source identifies Zavala's 2014
natural-gas-network control study as its scientific lead. These numeric facts
freeze matched reference selection; independent provenance and ordinary admission
remain B/D02/F06 obligations. Node enthalpy-flow mixing uses the package's sensible
reference convention even though the property metadata does not offer a general
enthalpy property. Do not replace the pseudo-gas with methane, PR or another EOS
and call it matched physics.

E03/F06 must extend the existing distributed/temporal/conservation owners with
this isothermal pressure/storage/momentum closure and genuine coupled node and
compressor consumers; G05 owns the configured tracking objective/hold policy.
Inspection of `packages/reference/process/models`, process specialization and
`workflow/horizon.rs` found no native gas-network/pipeline declaration or configured
case corresponding to these sources; that bounded search does not prove whole-tree
absence. Existing `control-volume-1d.pse` is an enabling spatial owner with its
dynamic refusal, not evidence that linepack already executes. Exact native
source/case/output identities and independent ordinary records therefore still
block native/reference comparison.

Required outputs include mapped endpoint and spatial pressure, signed mass/molar
and component flow, density/linepack at requested physical locations/times, T and
composition, compressor pressure rise/power, junction supply/demand mixing and
tracking products. Original network-wide mass inventory/flux closure, momentum
residuals, pressure/temperature domains and temporal/spatial refinement accompany
differential comparisons; endpoint agreement alone is insufficient. Preserve
invalid phase/property and mesh requests, occupied/out-of-range node ports,
zero/negative-flow domain questions and complete input-profile coverage as explicit
refusal/validation obligations. There is no arbitrary flow-reversal or thermal
network support claim. The 48/27 sweeps and three dynamic scientific roles are
finite source selections; native implementation, property validity envelopes,
additional admissible configuration combinations, reference execution and H01
completion remain open.

## Modular state-coordinate census and input closure — 2026-11-05

**Interface-checked, source only:** this slice covers C's six
`modular_properties/state_definitions` modules, shared electrolyte projection,
all twelve coordinate test modules, generic-property/state/bounds configuration,
the five BT-ideal coordinate example test modules and their inherited property
harness. It consumes existing native coordinate declarations rather than proposing
a second state mechanism. These are coordinate/initialization configurations;
their existence does not qualify the chosen EOS, equilibrium, phase selection,
electrolyte science or numerical solve. All required coordinate families remain
in D02/D03/H04; the counts below describe named source populations only.

### Effective coordinate contracts and inherited choices

| C state definition | Independent coordinates / bounds keys | Actual reconstructed or solved quantities / flash policy |
| --- | --- | --- |
| `FTPx` | Total molar flow, T, P, bulk component mole fractions; bounds keys flow_mol/T/P | Phase flows/compositions and phase fractions; `always_flash=True`. |
| `FcTP` | Component molar flows, T, P; bounds keys flow_mol_comp/T/P | Total flow is their sum; bulk fractions satisfy component-flow reconstruction; phase flows/compositions/fractions; always flash. |
| `FPhx` | Total molar flow, P, bulk mole fractions, total molar enthalpy; bounds keys flow_mol/P/enth_mol and supporting T | T is solved with the generic phase-enthalpy mixing equation; always flash. A T initialization/bound is not a separately fixed temperature coordinate. |
| `FcPh` | Component molar flows, P, total molar enthalpy; bounds keys flow_mol_comp/P/enth_mol and supporting T | Total flow/bulk fractions and T are reconstructed/solved; always flash. |
| `FpcTP` | Phase-component molar flows on actual phase membership, T, P; active bounds keys flow_mol_phase_comp/T/P | Total/component/phase flows are sums. Bulk fraction and multi-phase fraction contain total-flow ratios; phase composition is constrained against phase flow (or fixed to one for a phase having one component). `always_flash=False`. The accepted bounds-key list also contains enth_mol, but define_state does not read that key or make enthalpy an independent state. |
| `FpTPxpc` | Phase molar flows, T, P and phase-component mole fractions; bounds keys flow_mol_phase/T/P | Total flow is the phase sum; component flows and bulk fractions are reconstructed, phase fractions constrained; `always_flash=False`. A supplied phase split/composition is not required to be an equilibrium feed. |

T/P in bounds-key cells mean the literal `temperature`/`pressure` keys. All six
return molar material flow basis, component-total default material balance and
enthalpy-total default energy balance; explicit unit balance selection remains
the unit owner's configuration. Material, enthalpy-flow, material-density and
internal-energy-density methods demand actual phase property products. Coordinate
selection alone does not supply those products or their derivatives.

`GenericParameterBlock` has no default state_definition; omission is refused.
Explicit components, phases, per-phase EOS, pressure_ref and temperature_ref are
required. State components default to true species; apparent/true indexing is a
separate electrolyte configuration. `defined_state` defaults False in the base
state; it changes normalization and generic equilibrium construction. For phase
pairs, generic equilibrium is constructed when the state is receiving or
always_flash is True. Thus a defined FTPx state still flashes, whereas a defined
FpcTP/FpTPxpc feed can preserve a supplied nonequilibrium split. `has_phase_equilibrium`
on the base state is not an independent promise that a generic configuration has
an EOS, phase-pair method or component fugacity form.

Actual phase-component membership comes from component phase validity or explicit
phase component lists; unknown members, phase-incompatible members and a component
absent from every phase are refused. Electrolytes maintain separate true/apparent
membership. No full Cartesian phase/component membership is implied by the state
name. Generic flags also retain formation-enthalpy inclusion (default True); an
enthalpy coordinate requires the same datum choice in its independent reference.

Bounds use (lower, initial, upper) in base units or a fourth explicit unit converted
by `base/utility.py::get_bounds_from_config`; absent entries yield no bound/initial
value. Bounds are not empirical validity envelopes. Mole fractions have the source's
automatic 1e-20/1.001 numerical bounds; fraction-bound inputs are ignored with a
warning for FTPx/FPhx/FpTPxpc, while the component-flow definitions reject unexpected
keys. Neither those numerical bounds nor positive nominal initialization establish
zero-total/zero-phase-flow regularity. FpcTP total ratios and reconstruction from
flows require explicit domain handling in native acceptance.

### Finite structural, electrolyte and initialization population

| Exact source selection under `state_definitions/tests` | Distinct source configuration / evidence role |
| --- | --- |
| Each `test_{FTPx,FcTP,FPhx,FcPh,FpcTP,FpTPxpc}.py::Test{1,2,3}PhaseDefinedStateFalseNoBounds` and `Test{1,2,3}PhaseDefinedStateTrueWithBounds` | Six paired construction fixtures per definition, 36 total: c1/c2/c3 in one/two/three dummy-EOS phases, receiving/no-bounds or supplied/with-bounds. These are paired fixtures, not the full bounds×defined×phase-count product. Each bounded fixture uses coordinate-flow (0,100,200), T (290,345,400) K and P (1e5,3e5,5e5) Pa; FcPh instead uses T initial=300 and P initial=1e5. Enthalpy definitions add (0,500,1000) J/mol. The single-phase supplied FpTPxpc fixture explicitly passes has_phase_equilibrium=False. No physical fluid is selected by the dummy EOS. |
| Each same module `TestInvalidBounds`, `TestCommon` | Bad-key refusal, mole-fraction-bound warning or refusal as applicable, unit conversion, flow/density/energy projections, normalization, scaling and cloning. Common fixtures use c1/c2/c3 in two dummy phases a/b, coordinate-flow bounds (0,0.1,0.2) kmol/s, T (522,621,720) degR, P (1,3,5) bar and h (0,0.5,1) kJ/mol when applicable; FcPh uses T initial=540 degR and P initial=1 bar. These are conversion/contract cases, not six extra chemical systems or ragged-membership validation. |
| Each `test_*_electrolyte.py::{TestApparentSpeciesBasisNoInherent,TestApparentSpeciesBasisInherent,TestTrueSpeciesBasisNoInherent,TestTrueSpeciesBasisInherent}`; FTPx class names add scaling suffix | Six coordinate definitions × true/apparent basis × inherent/no-inherent chemistry. FTPx runs both legacy/new scaling and adds apparent/true inherent configurations with Ideal in place of DummyEoS. Scaling implementation is not a new scientific member. These sources use the exact shared `electrolyte_testing_config.py`, not a validated industrial carbonate package. |
| Shared electrolyte no-inherent configuration | H2O/CO2/KHCO3/K+/HCO3-/N2, aqueous dummy EOS + ideal vapor; KHCO3 dissociates to K+/HCO3-. True/apparent sets and allowed phase memberships differ; preserve both projection directions. Dummy enthalpy and density functions are test inputs, never ordinary scientific coefficients. |
| Shared electrolyte inherent configuration | One aqueous phase; apparent H2O/KHCO3/K2CO3/KOH and true H+/K+/OH-/HCO3-/CO3-- species, with the two named water-ionization/carbonate reaction laws. True/apparent totals differ through dissociation/inherent extent. Apparent states include equilibrium internally even at defined-state boundaries; the control-volume inherent-reaction flag is suppressed to avoid counting it twice. This is a mapping/reaction-ownership control, not independent physical equilibrium validation. |
| `test_FTPx.py::TestModifiedRachfordRice::test_flash` | 21 initialization inputs: seven shared/liquid-only/vapor-only component partitions × eps {default=1e-5,0,0.01}, bulk composition (1/3,1/3,1/3), K(a,b,c)=(0.5,1,3). Partitions are ({abc},{},{}), ({bc},{a},{}), ({ab},{},{c}), ({b},{a},{c}), ({c},{ab},{}), ({},{abc},{}), ({},{},{abc}), ordered as (shared,liquid-only,vapor-only). Negative K and unphysical fractions have separate diagnostic controls. These synthetic ratios are not a 21-fluid validation set. |
| FTPx shared initializer / no-VLE controls | FcTP/FPhx/FcPh reuse FTPx state_initialization. Negative bulk mole fraction raises; more than one liquid-vapor pair raises. No VLE, liquid-only/vapor-only species, bubble/dew guesses and Henry-law branches retain their actual fallback behavior. Missing saturation methods and unsupported Henry types can fall back during initialization; a fallback guess is not a successful thermodynamic solve. |
| `test_FpcTP.py::{test_phase_equilibrium_legacy_initialization,test_phase_equilibrium_initializer_object}` and `test_FpTPxpc.py::test_phase_equilibrium_initializer_object` | H2O/CO2, Ideal aqueous/vapor, Perrys/NIST coefficients in each module's `thermo_config_no_rxn`; receiving state at 298 K/101325 Pa. FpcTP starts phase-component flows VapCO2=0.005, LiqCO2=1e-8, VapH2O=1e-8, LiqH2O=9.995 mol/s. FpTPxpc starts phase flows (Vap,Liq)=(0.005,9.995) mol/s with corresponding fractions including the 1e-8 cross-phase traces. Both deliberately leave two equilibrium residuals unsolved; initializer-object checks accept infinite constraint tolerance. Retain these as partial-state initialization evidence, never a converged flash reference. |
| `test_FpcTP.py::{test_legacy_initialization_dof_error,test_initializer_object_single_phase}` | Vapor-only H2O/CO2 after removing liquid/pair configuration; 398 K/101325 Pa, VapCO2=0.005 and VapH2O=0.0005 mol/s. The legacy path fixes incomplete state freedoms and expects an error; the new initializer performs partial state initialization. A failure control and a successful initialization API call are distinct roles of this same scientific input. |

There are no pytest parameter decorators in these twelve coordinate test modules;
the explicit fixture classes and Rachford–Rice loop are the finite expansions.
Additional property/EOS/phase-equilibrium test populations remain with those owners,
including the later electrolyte verification and reaction/solubility consumers.

### BT ideal consumers and scientific input closure

All five example modules use benzene/toluene and liquid/vapor Ideal EOS with
SmoothVLE, component fugacity equality and IdealBubbleDew; they demand Perrys
liquid density/caloric and RPP4 ideal-gas caloric/vapor-pressure data. Their
independent numeric declarations stay at their exact source owners; the
separately embedded FPhx/FcTP/FcPh config_dicts must not be presumed identical to
`examples/BT_ideal.py::configuration` from matching names alone.

| Source under `modular_properties/examples/tests` | Exact selected scientific specification |
| --- | --- |
| `test_BTIdeal.py::TestStateBlock{LegacyScaling,ScalerObject}` | Defined FTPx state: flow=1 mol/s, T=368 K, P=101325 Pa, benzene/toluene fractions=0.5/0.5. |
| `test_BTIdeal_FcTP.py::TestStateBlock{LegacyScaling,ScalerObject}` | Defined FcTP state: each component flow=0.5 mol/s, T=368 K, P=101325 Pa. |
| `test_BTIdeal_FPhx.py::TestStateBlock{LegacyScaler,ScalerObject}` | Defined FPhx state: flow=1 mol/s, h=47297 J/mol, P=101325 Pa, bulk fractions=0.5/0.5; T is solved. The rounded h input is not an assertion of exact equivalence to fixed T=368 K. |
| `test_BTIdeal_FcPh.py::TestStateBlock{LegacyScaling,ScalerObject}` | Defined FcPh state: each component flow=0.5 mol/s, h=47297 J/mol, P=101325 Pa; T is solved. |
| `test_BTIdeal_FpTPxpc.py::TestBTIdeal_FpTPxpc` | FeedFlash: T=368 K, P=101325 Pa, each incoming phase flow=0.5 mol/s, each component fraction in each incoming phase=0.5. Inlet is supplied split; receiving outlet flashes. This is a unit consumer, not a property-state alias. |

BT configs bound T to [273.15,450] K with initial 300 K, pressure [5e4,1e6] Pa
with initial 1e5, coordinate flows [0,1000] mol/s with initial 100; enthalpy
configs bound h to [1e4,2e5] J/mol with initial 5e4. Reference T/P are 300 K/1e5 Pa.
These bounds are numerical case settings, not a source-certified validity domain.
Both scaling classes exercise one scientific specification per coordinate; each
module's inherited `PropertyTestHarness` also exercises construction, property
demands, initialization/failure, release-state and 0D CV integration. Do not count
inherited methods as additional operating points. Solved examples select
`ipopt_v2`; selected FpcTP legacy initialization also retains its legacy solver
route. Exact executable and locked environment identity remain execution work.

The closure consists of the selected state module, shared electrolyte helper when
needed, generic_property/base utility, the selected Ideal/SmoothVLE/fugacity/
IdealBubbleDew and Perrys/RPP4 or NIST methods, the explicit inline coefficient
dictionary, and the property harness/FeedFlash consumer. No dataset, trained
artifact, trainer or saved initialization file is loaded by these coordinate
fixtures. Original BT source leads are Reid et al. (4th ed.), Perry's (7th ed.)
and the specific formation-enthalpy source declared in BT_ideal.py. This is a
provenance lead, not ordinary admission. Preserve original units, reference state,
formation-versus-sensible convention and coefficient provenance before B/D02
publication; inline expected values remain oracle-only. H02's ordinary BT science
is separately selected and must not inherit these reference-only records.

### Native correspondence and remaining acceptance

The existing `thermodynamics/models/state-coordinates.pse` maps the six choices to
TotalFlowTemperature, ComponentFlowTemperature, TotalFlowEnthalpy,
ComponentFlowEnthalpy, PhaseComponentFlowTemperature and PhaseFlowTemperature.
`ideal-caloric-state.pse` adds selected caloric closures;
`seed-data/models/state-coordinate-fixtures.pse` supplies analytic transport,
reconstruction and zero/disappearing-state controls. These are concrete native
owners to extend, not a missing state universe and not the BT equilibrium law.
Current native coordinate interfaces admit every selected component in every
selected phase; ragged upstream membership and true/apparent electrolyte projection
therefore still need actual mapped native contracts and consumers. Native
component-flow states refuse undefined zero-total composition; supplied total-flow
coordinates can retain composition at zero flow. Phase-flow coordinates retain
supplied composition for a disappearing phase, while the caloric phase-component
closure requires positive phase flow to reconstruct intensive properties. Preserve
these exact native domains rather than borrowing upstream numerical bounds or
claiming an unimplemented zero-ratio convention.

D02/D03 acceptance still requires exact native cases/data and full transported
component/phase flow, composition, T/P/h/energy, equilibrium and derivative
products for each admitted scientific case; supplied/receiving and ragged/species
mapping cases must remain separate. Analytic coordinate fixtures do not qualify
BT, electrolyte chemistry or arbitrary phase counts. Original normalization,
material/energy/charge conservation, phase disappearance and datum controls
accompany matched comparisons. Scientific coefficient admission, validity
envelopes, configuration-specific native/reference solves and the remaining
property/reaction configuration population remain open. No run was performed in
this census slice and no initializer tolerance was adopted as native acceptance.

## Pure-fluid Helmholtz census and input closure — 2026-11-05

**Interface-checked, source only:** this slice resolves C's
`general_helmholtz` parameter/state/expression/registration/generation owners,
their six test modules, all eleven fluid parameter families, IAPWS95/SWCO2
convenience wrappers, and V's eight Water/CO2 verification CSVs and drivers.
Tracked-file inspection confirms that the fixed C revision includes generated
EOS `.nl` files and expression metadata, not just source JSON. Their presence in
the reading copy is not installation, shared-library compatibility or accepted
execution. Native Water status remains owned by 64d/64f/64k; this census changes
no scientific provider or reference pin.

### Fluid and executable-artifact population

All eleven have `<fluid>.json`, the corresponding parameter-generation Python
owner, `<fluid>_parameters.json`, `<fluid>_expressions_eos.nl` and
`<fluid>_expressions_st.nl` tracked under C
`general_helmholtz/components/parameters`. `st`, `visc`, `tcx` denote surface
tension, viscosity and conductivity. The table describes the tracked executable
closure, not every correlation that appears in generator source or bibliography.

| Registration key / fluid | Tracked transport expression families | EOS scientific source lead retained by its parameter JSON |
| --- | --- | --- |
| h2o / Water | st, visc, tcx | IAPWS95 2016 release and Wagner–Pruß 2002 |
| co2 / carbon dioxide | st, visc, tcx | Span–Wagner 1996 |
| butane / n-butane | st only | Bücker–Wagner butane/isobutane formulation |
| isobutane | st only | Bücker–Wagner butane/isobutane formulation |
| nh3 / ammonia | st only | Gao et al. 2023 associating ammonia EOS |
| propane | st, visc, tcx | Lemmon–McLinden–Wagner 2009 |
| r1234ze / trans R1234ze(E) | st, visc, tcx | Thol–Lemmon 2016 |
| r125 | st only | Lemmon–Jacobsen 2005 |
| r134a | st, visc, tcx | Tillner-Roth–Baehr 1994 |
| r227ea | st only | Lemmon–Span R227ea formulation, cited DOI acs.jced.5b00684 |
| r32 | st only | Tillner-Roth–Yokozeki 1997 |

Ammonia's Python generator contains viscosity/conductivity definitions, but the
tracked metadata has both availability flags False and no matching tracked
transport `.nl` files. Regenerating that source would change the artifact
population; do not claim those methods executable from source awareness alone.
Similarly an EOS/transport bibliographic reference is not numerical coefficient
admission. Missing visc/tcx are explicit unavailable products for the selected
artifacts, not a reason to remove their fluids from required scope.

The active registration owner scans the configured parameter path for EOS and
transport expression filenames; default path is the packaged parameter directory,
and changing it clears/rebuilds the registry. Component lookup is case-insensitive.
The IAPWS95/SWCO2 classes specialize the component default to Water/CO2, respectively;
they do not add a second scientific fluid. Construction requires a loadable
`general_helmholtz_external` binary, an existing parameter path and a registered
component. A negative availability result or unregistered component is a required
failure, not a skipped native acceptance run. Exact binary ABI/version, parameter
path and file identities must be retained with an executed reference.

### Effective state/phase/basis and numeric domains

Defaults are pure_component=None (not an executable fluid), state PH, mole amount
basis and MIX phase presentation. PH/PS/PU use pressure and specific
enthalpy/entropy/internal energy plus the corresponding mass/molar flow. TPX uses
T/P and, only for MIX/LG, vapor fraction; L/G have T/P without quality. MIX exposes
one framework phase while retaining private liquid/vapor thermodynamics, LG exposes
both phases, and L/G impose a single branch. These four presentations are not four
different EOSs or blanket stable-phase certificates. In particular single-phase
presentation can report its imposed phase even for inputs outside the intended
single-phase region; native branch/domain checks remain necessary.

Base `defined_state=False` and has_phase_equilibrium=True remain inherited state
settings. TPX MIX/LG receiving states add the smoothed saturation-complementarity
condition; defined states omit it. A separate saturation constraint is initially
inactive. PH/PS/PU obtain intensive products through external EOS inversions.
The new Helmholtz initializer returns success without solving equations; neither
that status nor inherited harness initialization establishes numerical convergence.
The legacy initializer's fix/release behavior remains a separate state-management
control. `htpx`/related helpers require exactly two of T, P, quality and check their
selected input bounds; the general expression writer has its own admissible input
combinations, including explicitly selected phase evaluation. Preserve the actual
selected interface rather than treating every T/P/x triple as one helper call.

Each fluid's `<fluid>_parameters.json::param` freezes R/MW, reducing/critical/triple
constants, density bound, T_min/max, P_min/max and reference_state_offset. Those
are executable-package bounds, not a certified scientific validity envelope.
For example metadata T ranges include Water [235,1300] K, CO2 [216,1400] K and
R134a [168,460] K; original EOS validity may differ. Preserve both the source
validity and actual artifact setting when choosing case boundaries. CO2 has a
nonzero ideal-Helmholtz reference offset; ammonia metadata has an empty offset
list; the other nine store [0,0]. Retain those exact declarations and establish
the effective enthalpy/entropy datum independently instead of interpreting every
empty/zero list as the same physical reference. Metadata is dimensioned through
the EOS library interface (including pressure conversion), never copied as
unitless SI values into ordinary native science.

### Shipped configured consumers and finite scientific inputs

| Source selector under C `general_helmholtz/tests` | Actual finite configurations and required evidence role |
| --- | --- |
| `test_with_harness.py::TestBasic*` | 28 explicit Water harness configurations: PH/PS/PU × MIX/LG/L/G × mole/mass, plus TPX × MIX/LG/L/G on mass basis. TPX mole is not present in this harness matrix; its separate state/heater controls remain below. Inherited PropertyTestHarness tests are construction, demanded-product, state-management and CV integration controls, not additional fluids or operating envelopes. |
| `test_stateblock.py::{test_scaler_object_mole_basis,test_scaler_object_mass_basis,test_scaler_object_mole_basis_TPX,test_misc_methods,test_set_not_fixed}` | PH MIX mole/mass and TPX MIX mole scaling/state-interface and state-management selections. Scaling is not a separate fluid problem. |
| `test_with_heater.py::test_heater_ph{_mixed_byphase,mixed_mixed_total,_lg_total,_lg_phase}` | Water PH mole, 100 mol/s, inlet h=4000 J/mol/P=101325 Pa, heat=2 MW; MIX or LG, default/component-total/component-phase material balance as each selector declares. The first case also checks hold/release-state before fixing its inlet. |
| Same file `test_heater_ph_l_phase` and `test_heater_ph_l_phase_two` | PH liquid-only, component-phase material balance, 100 mol/s, h_in=3000 J/mol/P=101325 Pa, heat=0.1 MW. Parsed bodies after their differing docstrings are identical: one scientific specification with two source wrappers, despite the latter's different descriptive intent. |
| Same file `test_heater_ph_g_phase`, `test_heater_tpx_g_phase` | Vapor-only/component-phase, 100 mol/s/P=101325 Pa/heat=1 MW; respectively h_in=50000 J/mol or T_in=422.60419933276177 K. Keep rounded enthalpy versus fixed temperature specifications distinct. |
| Same file `test_heater_tpx_lg{_total,_total_2,_phase}` | TPX LG mole feed T=326.1667075078748 K, quality=0, P=101325 Pa, 100 mol/s. Heat=2 MW/component-total, 5 MW/component-total or 5 MW/component-phase; distinct duty and balance choices. |
| `test_functions_unit.py::test_htpx_{mass,mole}`, `test_expression_writer_{mass,mole,sat}` | Water input interfaces at 300 K and saturation P=3536.8 Pa, quality endpoints; pressure-side probes at 3536/3537 Pa. Mass inverse coordinates include h or u=112560 J/kg, h=2549900 J/kg, u=2411600 J/kg and s=393.09 or 8517.4 J/(kg K), with their exact selected products and explicit mass/mole result-basis conversions in source. Mole counterparts use MW=18.015268 g/mol. Preserve input/output/unit flags and saturation/liquid/vapor selections; repeated method calls are product demands, not extra fluids. Other argument/error controls in these functions must not be promoted into physically admissible nominal states. |
| `test_functions_unit.py::test_{h2o,co2,r134a,r1234ze,propane}_transport` and `test_r134a_thermo` | Water viscosity at saturation 300/640 K (liquid/vapor), conductivity 300 K; CO2 viscosity liquid240/300 K and vapor220/300 K, conductivity vapor220 K. R134a viscosity170/370 K, conductivity170/270 K, surface tension270 K; saturation thermodynamics at 280 K or 372710 Pa. R1234ze transport selects vapor (250 K,50000 Pa), (300 K,1e5 Pa), (300 K,0.001 Pa), liquid (250/300 K,20 MPa) and (300 K,10 MPa). Propane transport uses 320 K at 0.1/2.5/4 MPa with specified vapor/liquid branch. Every exact product and result basis remains at its named test owner; no cross-product of all products/points is implied. |
| `test_parameter_util.py::test_{h2o,nh3,co2,propane,r125,r134a,r32,butane,isobutane}` | Each has three inline saturation records (near triple, interior, near critical) for pressure/enthalpy/entropy and approximate saturation-density products. The Python generation entry is called with dry_run=True; tests probe the source-built expressions without publishing regenerated artifacts. Parameter-generation numerical agreement is distinct from installed-library state/equipment agreement. |
| Same file `test_r1234ze` | Six declared (T,density) pressure records but dictionary key4 repeats: the 383 K record shadows the earlier 360 K record, leaving five realized records. Record the shadowed 360 K/2039.103 kPa declaration as an unresolved source case, not a passing sixth point. `r227ea_main` is imported but there is no test_r227ea in this module; its registered fluid remains required. |
| `test_register.py`, `test_functions_unit.py::{test_available,test_available2,test_add_function,test_initialize_param_block,test_errors,test_plot_no_exception,test_default_initializer,test_HelmholtzEoSInitializer}` | Registration/availability, malformed state/input, nonexistent fluid, helper bounds, display and no-op-initializer controls. These are contract/refusal or presentation roles, not extra physical operating cases. |

There are no pytest parameter decorators in these six Helmholtz test modules;
the harness classes and inline numerical tables are the concrete expansions.
The three coverage gaps above (uninstantiated basis/state tests,
R227ea parameter test and ammonia transport artifacts) remain explicit coverage
questions, not exclusions of their scientific families.

### Independent Water/CO2 verification data and historical omissions

V `verification/properties/IAPWS95/test_iawps_verification.py` and
`SWCO2/test_swco2_verification.py` consume one CSV each for liquid, vapor,
supercritical and saturation. Both select PH/MIX mass basis, defined state and
flow_mass=1 kg/s. Each CSV row supplies P in MPa and h in kJ/kg as reference inputs;
T, density, internal energy, entropy, heat capacities, sound speed and selected
transport are independent expected outputs. Preserve BOM-aware CSV headers and
per-property units, especially CO2 viscosity in microPa s versus Water in Pa s.

| Family / CSV scientific row count (header excluded) | Driver behavior that limits its historical evidence |
| --- | --- |
| Water liquid: 5002 | Driver omits one row satisfying P=22 MPa and T>646.85 K. |
| Water vapor: 5263 | Omits one row satisfying P=22 MPa and T<647 K. |
| Water supercritical: 5731 | No whole-row omission in the inspected driver. |
| Water saturated: 1200 | Omits row index0 and both phase rows at P=21.899 MPa/T=646.47 K, three rows total. Phase is selected from each row's liquid/vapor tag. |
| CO2 liquid/vapor/supercritical/saturated: 9/157/101/4 | No whole-row omission; vapor entropy suppresses (1 MPa,476.59 K) and (7.3589 MPa,684.13/704.13 K); vapor Cv/Cp/sound-speed suppresses (7.3589 MPa,304.13 K). Supercritical entropy suppresses (7.3773 MPa,684.13/704.13 K); one supercritical row has Cv=ND and its Cv/Cp/sound-speed comparisons are omitted together. Current driver H_OFFSET/S_OFFSET are both zero; commented alternative datums are not selected. |

These are 17196 Water and 271 CO2 source rows, not 17467 demonstrated passes.
Historical guards do not authorize native exclusions. Retain the five Water
whole-row omissions and every per-property missing/suppressed expectation for
explicit source/near-critical/reference investigation; admit or hold the exact
case/property with its reason rather than silently shrinking scope. Thermal
conductivity is not asserted by these drivers; CO2 viscosity is also not asserted,
and some CSV columns such as Joule–Thomson coefficient are not consumed. File
presence cannot qualify those outputs. All CSVs and inline expected values remain
oracle/independent-validation material, never ordinary property coefficients.

### Recursive closure and native handoff

The executable reference needs C's selected fluid source/metadata/EOS and demanded
transport `.nl` files, parameter-path registration, external binary and linked
dependencies, Pyomo external-function/DAE/unit machinery and the exact locked
core/interpreter environment. Parameter regeneration additionally consumes the
per-fluid Python transport definitions and `helmholtz_parameters.py::WriteParameters`;
do not regenerate merely to make a missing declared product appear. Harness and
heater cases additionally consume their existing unit/harness owners and selected
Ipopt interface. V adds pandas and its exact eight CSVs. No training dataset,
trainer, fitted neural artifact or saved initialization is selected. Installation,
ABI/loadability and actual numeric runs remain unperformed for this census.

Native `helmholtz.pse` owns a general residual-Helmholtz formulation, while the
existing CoolProp Water factory/steam declarations and 64k worker own the selected
liquid/vapor/supercritical Water realization and declared numerical products.
Their current targeted status is linked from 64f/64k; it does not establish wet
PH/PS/PU/TPX inversions, all Water transport or any of the other ten fluids. Match
effective EOS constants, enthalpy/entropy datum, phase/root and input/output basis
before constructing each native/reference case; a shared provider direct/worker
comparison alone is not independent scientific validation. Ordinary scientific
sources and coefficients require B/D02 admission separately from using the IDAES
reference artifacts. D02 supplies those pure-fluid/property closures; F02/F05 and
generic equipment consume exact phases/derivatives/transport rather than the
Helmholtz package name. All missing native configurations, source-backed validity
envelopes, near-critical/refusal products and independent output comparisons stay
required under D02/H04. This resolves source/artifact selection and finite cases,
not broad pure-fluid implementation or qualification.

## Solid/liquid separation and thickener census — 2026-11-05

**Interface-checked, source only:** C `unit_models/solid_liquid/{sl_separator,
thickener}.py`, both test modules and their actual generic-Separator/property
dependencies resolve the following D04 selections. These units replace inherited
unit configuration with their own ConfigBlock: dynamic=False and has_holdup=False
are the only allowed values. Neither a parent dynamic flowsheet nor a time-indexed
port grants storage or transient settling. No parameter decorators/operating-point
sweep occurs in the two test modules.

| Actual categorical configuration | Effective meaning / guard |
| --- | --- |
| Independent solid/liquid property packages and argument dictionaries | Each defaults to useDefault, but these builders call the selected package directly and do not themselves resolve a parent default. Both shipped fixtures supply explicit packages. Inlet state blocks force defined_state=True and has_phase_equilibrium=False. A package label is insufficient: flow, density, viscosity, temperature/pressure or enthalpy products must exist where consumed. |
| Material balance default useDefault | Each internal Separator resolves the actual property's default. Exposed alternatives componentPhase/componentTotal/total/none are implemented by that owner; elementTotal is explicitly refused. A none balance constructs less physics and is not a closed separation claim. |
| Momentum default pressureTotal | Internal splitters equate outlet and inlet pressure; none omits that equation. pressurePhase/momentumTotal/momentumPhase are accepted enum values but explicitly unimplemented/refused downstream. |
| Energy default equal_temperature | Internal splitters also expose none, equal_molar_enthalpy and enthalpy_split through the actual EnergySplittingType enum. Molar-enthalpy equality demands enth_mol even for a mass-basis package; not every fixture supports it. enthalpy_split works with the units' fixed totalFlow splitting and requires actual enthalpy-flow terms. An energy policy changes equations and cannot be inferred from a default test pass. |
| Fixed splitter structure | Both use nonideal, total-flow splitters with named outlets, no phase equilibrium and explicit inlet mixed-state blocks. Phase/component/ideal split axes of the general Separator are not exposed choices on these units. Property inherent reactions, where actually present, retain the Separator's reaction-generation coupling; the shipped fixtures do not select them. |

### Exact shipped scientific/structural selections

| Source fixture | Input specification, actual science and fixed/free roles |
| --- | --- |
| `tests/test_sl_separator.py::TestSLSeparatorBasic` | Same `SaponificationParameterBlock` is supplied to both sides; the nominal solid side is actually liquid-only. Solid-side feed: volumetric flow=10 m3/s, concentrations H2O=55388, NaOH=100, ethyl acetate=100, sodium acetate=ethanol=1e-6 mol/m3; T=303.15 K/P=101325 Pa. Liquid feed: 20 m3/s, H2O=55388, NaOH=ethyl acetate=1e-6, sodium acetate=ethanol=50 mol/m3; T=320 K/P=200000 Pa. Liquid recovery fixed at0.7. Default material resolves componentTotal, energy equal-temperature, pressureTotal. Solids share one inlet/outlet state, while liquid splits recovered/retained with the same composition, T and P. No solid constitutive law or physical solid-property validation follows from this fixture. |
| `tests/test_thickener.py::TestThickener0DBasic` | Locally authored mass-basis synthetic solid a/b and liquid c/d packages. Solid mass flow=0.01225 kg/s, fractions (0.2,0.8), T=303.15 K/P=101325 Pa, density=2500 kg/m3. Liquid mass flow=0.0021 kg/s, fractions (0.3,0.7), T=310 K/P=150000 Pa, density=1000 kg/m3 and viscosity=0.001 Pa s. Total feed volume=7e-6 m3/s with solid volume fraction0.7. Fix underflow volume=5e-6 m3/s, area=1 m2, v0=1.18e-4 m/s, v1=1e-5 m/s, exponent C=5 and maximum solid volume fraction0.9. Particle size and both phase splits are solved, not additional fixed inputs. Default material resolves componentTotal on each mass-basis package; pressure/energy stay separate for the solid/liquid streams. |

The thickener has two total-flow splitters, each with underflow/overflow. Its
scalar area, overflow/underflow volume fractions and flux-density equations close
solids continuity/conservation. The selected hindered-settling law is piecewise:
inside zero-to-maximum solids fraction it combines Stokes and Darcy-type terms,
outside it returns zero; outlet fractions are also bounded above by the maximum.
Stokes closure connects v0 and particle diameter to feed density difference,
viscosity and gravity. There is no selectable settling-law enum, axial/time grid,
moving interface or retained inventory. The governing leads are Bürger et al.
(2006, DOI 10.1016/j.mcm.2005.11.008) and Barton–Li–Spencer (1992), with inline
fixture coefficients. They do not identify a validated real suspension.

Both tests exercise build, DOF, units, structural/numerical diagnostics,
BlockTriangularizationInitializer, solve, detailed solution/stream outputs and
conservation; SLSeparator adds scaling controls. These are roles of two fixed
fixtures rather than new configurations per test method. SLSeparator's legacy
initialize API explicitly refuses; initializer objects are selected. Numerical
tests select ipopt_v2. Preserve initialization solver/options and the real
Ipopt/locked environment at execution; no solver was invoked for this census.

### Input closure, native correspondence and remaining configurations

SLSeparator's recursive inputs are its unit plus generic `separator.py`,
`statejunction`-style aliasing via the shared solid state, the saponification
example property owner and core state/initialization/scaling/reporting methods.
Thickener additionally consumes its test-local two property classes, fixed
densities/viscosity, sensible-enthalpy test functions and inline settling
coefficients. No CSV, trainer, serialized starting state or external property
function is loaded by these fixtures. The property inputs are test/reference
material; copying their dummy chemistry or settling constants does not admit an
ordinary solid/liquid scientific package. D02/D04 must source actual phase
density/viscosity/caloric data, particle/settling parameters and empirical validity
for an ordinary consumer. A fixed Stokes velocity and a measured particle-size
input are different specification roles; do not overfix both without a compatible
closure/design problem.

Native `process/models/separator.pse` provides total/component/phase split owners,
and the existing property/state/source owners provide typed domains and input
eligibility. A bounded search of native reference packages, process specialization
and runtime modeling owners found no SLSeparator/Thickener/Kynch/Stokes consumer
matching these units; generic splitter presence is not complete physical support.
D04 must compose the actual two-property streams and settling closure at those
owners. Required reports map all five separator streams or six thickener phase
streams, full component/mass/volume flows, T/P, liquid recovery, area, feed and
outlet solid fractions, v0/v1/C/maximum fraction, particle size and fluxes. Original
solid/liquid component and total conservation, energy conventions and dimensional
Stokes/flux residuals accompany numerical comparison.

The exact two fixtures and exposed categorical/refused choices are resolved;
nondefault compatible property/energy/balance configurations, zero/maximum-volume
fraction and zero-density-difference domains, settling-law boundary smoothness,
rating versus area/particle sizing, ordinary data and native/reference execution
remain required decisions/evidence. Do not declare every exposed balance×energy
combination physically supported from these defaults. The source search of X's
`mod` tree and V's source tree found no named SLSeparator/Thickener consumers;
that does not establish absence from notebook code cells or unseen archive
content. No held or required case is excluded by this bounded finding.

## Membrane and plate-exchanger configured census — 2026-11-05

**Interface-checked, source only:** F04's exact core sources are
`co2_capture_and_utilization/unit_models/membrane_1d.py` and
`column_models/plate_heat_exchanger.py`, their test modules, and their consumed
MSContactor/NTU/property owners. Neither test module has a parameter decorator or
an operating sweep. The two physics families remain distinct; the X skeleton
pervaporation notebook is a third custom-unit scientific problem, not a wrapper
for this gas membrane.

### Gas membrane: choices, full fixture and input issues

Membrane defaults are countercurrent, sweep=True and 5 finite elements. Feed
always goes forward; cocurrent sweep goes forward and countercurrent sweep goes
backward. Sweep=False removes the sweep inlet/feed and retains a permeate outlet.
Each side has an explicit property package/arguments plus energy/pressure-balance
booleans, both default True. It does not expose arbitrary reaction, pressure-drop,
heterogeneous chemistry or side-stream options from the underlying MSContactor.
Shared components form the transfer population. MSContactor requires interacting
components (or separately selected heterogeneous chemistry) and matching stream
flow bases; membrane transfer accepts only mass or molar basis.

Outer dynamic/holdup flags are inherited. Construction passes streams and element
count to MSContactor, whose dynamic default inherits the containing unit; it does
not explicitly pass the outer holdup selection. Child storage/volume allocation
and transient initial/boundary conditions therefore need an actual E03 case rather
than inference from an inherited flag. Shipped membrane tests use steady/no holdup.
Separate no-sweep, cocurrent solved, mass-basis, nondefault balance and transient
configurations remain required selections/readiness questions, not established
by the one nominal solve.

| Exact selector under the membrane test directory | Finite input specification / evidence role |
| --- | --- |
| `tests/test_membrane_1d.py::{test_config_countercurrent,test_config_cocurrent}` | Three cells, sweep=True, explicit steady setting and the same Ideal vapor CO2/H2O/N2 property package. Two direction-construction checks; neither supplies a solved operating specification. |
| Same file `TestMembrane::membrane` | Countercurrent/three cells/sweep=True; area=100 cm2, length=10 cm; permeance CO2=1500 GPU, H2O=N2=60 GPU in every cell. Feed=100 mol/s,365 K,120000 Pa with (N2,CO2,H2O)=(0.76,0.13,0.11). Sweep=0.01 mol/s,300 K,51325 Pa with (H2O,CO2,N2)=(0.9986,0.0003,0.0001). Both sides retain energy/pressure defaults. Block-triangular initialization uses constraint tolerance2e-5, followed by the selected default solver. Source tests demand both product-stream states and original material/enthalpy balances. |

The sweep input fractions sum to **0.999**, not one. Preserve the exact source
problem and its source issue; do not silently normalize or label the fixture
native-valid. A corrected input is a separately identified ordinary/reference
selection with a reason and corresponding expected products. This mismatch must
be reconciled before accepting normalized native state correspondence.

The actual cell thermal equation equates feed and sweep temperatures at each
cell. It does not state that each side's temperature is constant along the whole
unit; inlet temperatures are different in the fixture. Actual material transfer
uses local feed density, area per cell, the explicit GPU conversion factor
10e-8/13333.2239 m/(s Pa), permeance and cross-membrane partial-pressure difference,
with signed transfer and opposite-stream accounting. Density/partial-pressure
and flow-basis conversion are part of the matched formulation; a different
standard-volume permeance interpretation is not an implementation-only change.

The property closure is `power_generation/properties/natural_gas_PR.py::get_prop`
with components CO2/H2O/N2, phases=[Vap], EOS=IDEAL and scaled=False. It produces
FTPx, NIST-based component caloric/entropy data and its explicit pure/mixture
methods, reference T/P=298.15 K/101325 Pa. The helper's default PR/all-component
choice is overridden; an imported cubic module does not select PR for this case.
The unit, MSContactor, generic property/Ideal/pure-method declarations and inline
coefficient/permeance inputs are the recursive scientific closure. No dataset,
trainer or saved initialization is selected. Ordinary component and membrane
parameter provenance/validity remain B/D02/F04 work.

### Plate exchanger: inherited NTU/MEA closure and finite fixture

PlateHeatExchanger inherits two NTU 0D control volumes, each with property and
material/energy/momentum balance choices; defaults resolve from the actual
property (component-total/enthalpy-total here) with pressure-total. It forces
has_pressure_change=True on both sides; False is a configuration refusal, not a
constant-pressure alternative. Inherited dynamic/holdup flags are not a
qualification of transient plate/fluid/wall storage. There is no independent
co/countercurrent selector: effectiveness is selected by pass parity. Passes and
channels are positive integers, divider plates nonnegative; defaults are4/12/0.
Positive channel gap/active plate count, compatible property products and valid
correlation domains remain physical constraints beyond those integer domains.

The only fully specified fixture is
`column_models/tests/test_plate_heat_exchanger.py::TestPHE::phe` (also used by
the configuration test): four passes, twelve channels/pass and two dividers,
steady parent, separate identical aqueous-MEA packages. Hot inlet is
60.54879 mol/s,392.23 K,202650 Pa with (CO2,H2O,MEA)=(0.0158,0.8747,0.1095);
cold inlet is63.01910 mol/s,326.36 K,202650 Pa with
(0.0414,0.8509,0.1077). `fix()` selects the actual initialized geometry:
plate length1.6925 m, width0.6135 m, thickness0.0006 m, compressed pack length
0.381 m, port diameter0.2045 m, plate conductivity16.2 W/(m K), total area114.3 m2.
Outlet states, heat duty, overall coefficient/effectiveness, channel/port flow and
pressure losses are solved; area is rating input, not an unknown design variable
in this fixture.

The four-pass branch supplies countercurrent-style effectiveness. The odd-pass
branch references `pass_num`, while the inspected derived/base owners declare
`number_of_passes` and no `pass_num`; no odd-pass test is shipped here. Preserve
odd-pass science as a source/execution-readiness question rather than treating
the declared positive-integer domain as demonstrated cocurrent support. Plate
geometry determines active plate count, gap/enlargement and hydraulic diameter;
Reynolds/Prandtl heat-transfer and pressure-loss correlations consume inlet liquid
density, viscosity, conductivity and heat capacity. Their scientific lead is
Akula et al. 2019's solvent-capture plate-exchanger parameter-estimation study.

The actual property input is `column_models/properties/MEA_solvent.py::configuration`:
one aqueous Ideal phase on apparent basis, FTPx state with H2O/MEA/CO2 and true
MEA+/carbamate/bicarbonate species. Carbamate and bicarbonate inherent equilibria,
Henry/N2O analogy, apparent-to-true mapping and solution caloric/density/viscosity/
conductivity correlations are required even though the equipment has no separate
reaction-package argument. The property uses its declared sensible/absorption
enthalpy convention; a generic formation-enthalpy replacement would be different
science. Those classes and coefficients are inline in the MEA owner, consuming
generic-property/Ideal/reaction and unit helpers; no external dataset, trainer or
saved solution is selected. The available but undemanded diffusivity/surface-tension
methods are not qualified by the exchanger fixture.

The plate test's initializer/solve/solution methods carry a non-strict historical
xfail associated with older Pyomo/platform behavior. The selected core/Pyomo pins
remain unchanged, and an xfail or unavailable solver cannot satisfy native H04.
Default solver executable/options and actual MEA/property derivatives must be
frozen and exercised for the chosen reference. No test ran in this census slice.

### Native handoff and exact remaining obligations

Existing native heat-exchanger, state, chemistry and distributed/transfer owners
are enabling surfaces. Inspection of native process/thermodynamic model packages
did not find the membrane permeance or MEA plate-correlation consumer above; this
bounded source observation is not an absence proof for every runtime module.
D02/D03 supply gas/MEA scientific records and demanded products, E03 supplies any
admitted storage/distributed contract and F04 supplies the configured unit
closures. Reuse those owners; generic contactor/exchanger presence alone cannot
supply physical membrane or plate behavior.

Membrane correspondence requires all inlet/outlet component flows/T/P, cell
composition/density/pressure/temperature, permeances and transfer/enthalpy products
on declared physical positions, original cross-stream material/energy cancellation
and spatial refinement. Plate correspondence requires both complete outlets,
heat/material balances, pressure losses and correlation/geometry products with
rating/design freedoms explicit. Ordinary membrane/MEA/plate inputs, valid
correlation envelopes, the sweep-normalization and odd-pass issues, uninstantiated
categorical variants and actual native/reference cases remain open. The custom
pervaporation notebook's separate empirical flux, glycol-water feeds and design
role remain required under D05/G06; this gas-membrane slice cannot discharge them.

## Custom glycol/water pervaporation closure — 2026-11-05

**Interface-checked, source only:** X
`notebooks/docs/unit_models/operations/skeleton_unit.ipynb` and its `_doc`,
`_test`, `_usr` versions have identical full parsed Python statements, including
assertions. They supply one authored custom unit with two scientific roles:
fixed-feed simulation and inverse feed-composition optimization. Its only local
import is the adjacent `eg_h2o_ideal.py`; recursively inspect that exact helper
and `operations/tests/test_eg_h2o_ideal.py`, not a same-named unrelated property
example. The notebook's literature diagram describes a recycle process, but the
actual executable topology is two feeds→mixer→custom pervaporation→two products,
with no recycle arc. Native correspondence follows the constructed model.

| Role / exact selected input | Scientific meaning and fixed/free specification |
| --- | --- |
| Custom unit / baseline simulation | Steady skeleton, one liquid representation and water/ethylene-glycol molar component flows. Water feed: water=0.34, glycol=1e-6 mol/s; glycol feed: water=1e-6, glycol=0.66 mol/s. Both at318.15 K/101325 Pa. Membrane area=6 m2; permeate vacuum=1300 Pa. Thus total mixed input is1.000002 mol/s, retaining the two trace flows. |
| Empirical component flux | Water/glycol activation energies=51000/53000 J/mol and preexponentials=5611320/22358.88 mol/(s m2). Each permeate rate is area times its preexponential and activation exponential at inlet T. There is no composition, partial-pressure, density or vacuum dependence in this selected empirical flux law. Retentate is inlet minus permeate component flow. Do not replace it with the gas membrane's driving-force law. |
| Thermal/pressure representation | Retentate and permeate share inlet temperature; retentate keeps inlet pressure, permeate uses fixed vacuum. Positive reported condensation-duty magnitude is the sum of permeate molar rates times fixed latent heats40660/56900 J/mol. Product property blocks remain liquid-only: no wet-vapor/VLE/condenser thermodynamics is constructed. Keep this simplified assessment boundary distinct from a full energy-conserving phase-change unit. |
| Inverse design/optimization | Free the dominant water-feed water rate and glycol-feed glycol rate; constrain their sum to1 mol/s while retaining both1e-6 trace flows. Maximize mixed-inlet water mole fraction with separation factor≥100; all temperatures, pressures, area and empirical coefficients remain fixed. Separation factor is the permeate water/glycol mole ratio divided by the inlet ratio. This is feed design, not an area optimum or a transient recycle. |
| Custom initialization / authoring role | Notebook attaches a user initializer, fixes supplied states, calculates custom variables from their constraints, solves the unit, then releases inlet states and solves the assembled flowsheet. That custom authored law/port/initialization workflow remains required D05/G06 behavior; reproducing Python callback syntax is not required. Solver availability and an actual initialization/assembled solve remain execution prerequisites. |

The scientific lead is Du–Chakma–Feng 2008 (DOI
10.1016/j.seppur.2008.08.004), with the chosen empirical coefficients inline in
the notebook. Their relation to the measured membrane, operating envelope and
temperature dependence needs original-source ordinary admission. The selected
temperature is a single operating point, not certification across all positive
temperatures or feed compositions. Native acceptance must explicitly address
nonnegative retained flows, composition/ratio singularities and reachable design
specifications; unconstrained algebraic variables do not establish those domains.

The helper selects FpcTP, one liquid Ideal phase and water/ethylene-glycol
components; Perrys liquid density/caloric and RPP4 ideal-gas caloric/vapor-pressure
coefficients are inline. Reference T/P are298.15 K/1e5 Pa; numeric state bounds
are flow[0,1000] mol/s, T[273.15,450] K, P[1e3,1e6] Pa. Formation enthalpies,
component parameters and units remain those exact declarations; NIST/Reid/Perry
leads identify original coefficient provenance, not ordinary permission. The
single-liquid consumer does not activate a liquid/vapor equilibrium merely
because the component dictionaries contain vapor-pressure methods.

### Adjacent helper validation population and recursive prerequisites

The helper test's `TestStateBlock` is a separate FpcTP property witness: water
and glycol each100 mol/s at300 K/101325 Pa. Its inherited PropertyTestHarness
adds API/CV/state-management controls. The numeric pure-method tests explicitly
parameterize component {water,ethylene_glycol} × test_point {0,1}, four inputs
per role:

| Exact class/method role | Water / glycol selected temperature pairs and interpretation |
| --- | --- |
| `TestPerrysProperties::test_liquid_densities` | Water273.16/333.15 K; glycol260.15/719.7 K; pure-component flow100 mol/s/P101325 Pa. The glycol point and other out-of-bound temperatures are present source inputs, not implicitly admitted by the helper's450 K upper state bound. |
| `TestPerrysProperties::test_liquid_heat_capacities_enthalpy` | Water273.16/533.15 K; glycol260.15/493.15 K; independent enthalpy-difference checks reference298 K for both components, with the exact expected Cp/reference values at this test owner. |
| `TestRPP4Properties::test_vapor_heat_capacities_enthalpy` | Water545/632 K; glycol500/600 K; switch the pure-component phase to ideal vapor, pure flow100 mol/s/P101325 Pa; enthalpy-difference references500/400 K respectively. |
| `TestRPP4Properties::test_saturation_pressures` | Water300.25/350.16 K; glycol387/473 K; same explicitly selected pure vapor property construction. Vapor-pressure evaluation is distinct from the pervaporation unit's empirical flux. |

Thus there are sixteen parameterized pure-method inputs plus the fixed mixture
property witness; they are not sixteen membrane operating cases. Numeric
reference arrays, rounded heat-capacity/enthalpy conventions and out-of-bounds
requests remain exact oracle inputs requiring admissibility/evidence assessment;
they cannot certify the ordinary package over their enclosing range.

The complete selected closure is the notebook's inline custom functions and
parameters, adjacent property helper, generic property/FpcTP/Ideal/Perrys/RPP4,
Feed/Mixer/Product/SkeletonUnitModel, core state propagation/arc expansion and
constraint-calculation helpers, plus its selected solver and notebook runner.
The process image is explanatory, not a coefficient artifact. No CSV, trained
model, external trainer or saved initial state is loaded. The X2.10.0 declaration
still does not establish compatibility with core2.13.0; local import resolution,
helper validity and solver/notebook execution must be qualified at the existing
reference owner without changing the core pin silently.

Native ordinary authored relations, ports/state/initialization, optimization and
assessment owners are the target extension workflow. The inspected process and
thermodynamic native models contain no pervaporation consumer matching this
empirical law; this bounded search is not a whole-tree absence claim. D05/G06
must supply the real authored custom case and fixed/free roles, with independent
ordinary inputs from B/D02 and native/reference output identities. Compare every
component product, inlet/permeate fraction, separation factor, latent-duty report,
optimized feed and physical domain/material residual. An analytic calculation of
the empirical law is a useful independent check, not proof of real-membrane
validity. Both roles, helper scientific controls, original-data admission and
matched execution remain required; none ran in this census slice.

## Enzyme CSTR inventory and NMPC source closure — 2026-11-05

**Interface-checked, source only:** E02/G05's concrete core family is
`apps/caprese/examples/{cstr_model,cstr_nmpc,cstr_reduced}.py`, with
`core/util/tests/test_initialization.py` owning the actual property/reaction
classes. Their origin in a test helper neither excludes the selected process nor
admits its data as ordinary physical thermochemistry. The reduced-controller
example is a distinct approximation/control source variant; successful full-model
control cannot qualify its altered equations. No model or controller ran here.

### Physical/normalized model, categorical settings and initial inputs

`cstr_model.make_model` exposes steady/dynamic × bounds off/on; no guard removes
bounded steady construction, but the shipped top-level selections are dynamic
unbounded plant, dynamic bounded controller and unbounded steady initializer.
Defaults are horizon6 min,60 finite elements,2 Radau points, inlet_E=11.91 and
inlet_S=12.92 in the helper's kmol/m3 convention. Time/mesh arguments are ignored
by the steady construction; they are not extra steady scientific configurations.
The two inlet concentration arguments change scientific forcing and require a
declared envelope rather than every possible real input.

The CSTR explicitly selects holdup=True, component-total/enthalpy-total balances,
no momentum balance and reaction heat. The two-input mixer has component-total
material balance and no pressure-mixing equation. All species are in one aqueous
phase: substrate S, enzyme E, complex C, product P and solvent. Reactions are
S+E→C, C→S+E, C→P+E; first reaction is second order in S/E and the others first
order in C. Actual Arrhenius inputs are preexponentials
(3.36e6,1.80e6,5.79e7), activation energies(8000,9000,10000) kcal/kmol and
gas constant1.987 kcal/(kmol K), with reference temperature300 K retained in the
parameter block. Heat coefficients are(1000,1000,5000)/(900×0.231) in the
helper's normalized thermal convention, not independently admitted SI reaction
enthalpies. Source leads are Christofides–Daoutidis1996 and Heineken et al.1967;
original formulation/parameter correspondence remains ordinary-source work.

Property transport uses concentration×volumetric flow. Receiving solvent
concentration is fixed at1. Thermal density is temperature and thermal flow is
volumetric flow×temperature: the nominal energy storage literal300 is normalized
storage, **not300 J**. State/reaction scalar variables mostly carry no Pyomo units
despite documented kmol/min/m3 conventions and metadata. Native physical typing
must declare the normalization/caloric scale explicitly before comparing energy
or reaction-heat products. This simplified model cannot establish general fluid
enthalpy, pressure or caloric-data validity.

| Selected baseline input | Actual specification |
| --- | --- |
| Feed composition | Enzyme feed E=11.91 kmol/m3; substrate feed S=12.92 kmol/m3. Other reactive concentrations on each feed are0; solvent concentration=1 on both. |
| Feed flow and temperature | Enzyme0.1 m3/min at290 K; substrate2.1 m3/min at310 K. No feed-pressure coordinate or pressure balance is invented. |
| Dynamic storage | Volume fixed at t=0 to1 m3; initial material holdup for S/E/C/P each0.001 kmol, solvent excluded from this fixation. Initial normalized thermal storage=300. Dynamic inlet/outlet volume flows are constrained equal; solvent storage/concentration participates in constant-volume behavior. Empty initial material inventory is not the selected case. |
| Bounded construction | Enzyme feed flow[0.01,1], substrate flow[0.5,5] m3/min; material holdups nonnegative, S≤20/E≤1/C≤5/P≤5 in helper amount units. These limits constrain the controller problem; they do not establish the validity of the kinetic/thermal law throughout those bounds. |

### Explicit scientific/numerical workflow variants

| Exact core selector | Finite configuration, specifications and required products |
| --- | --- |
| `cstr_model.py` top-level constructors | Three canonical construction roles above: dynamic(6 min,60×2), bounded dynamic(same mesh), and steady. The steady model retains holdup variables but no time accumulation; it is not a transient run. |
| `apps/caprese/tests/test_util.py::test_initialize_by_element_in_range` | Unbounded dynamic2 min,20 elements/default2 points. Solve intervals[0,1] then[1,2], retaining state continuity and the same original equations. Expected S/C/E/P concentrations at1/2 min and original equality residual checks remain exact oracle inputs at this owner. `_test_add_noise_at_time` in the same file is not collected by pytest and cannot supply a passing noise control. |
| `apps/caprese/tests/test_categorize_cstr.py::test_categorize_1` | Dynamic1 min,5×2, original differential/measurement partition; structural categorization, not an accepted trajectory. |
| Same file `test_categorize_2` | Same model/mesh, replace initial energy holdup by T=300, initial P holdup by P concentration=0, and initial C holdup by C accumulation=0. These are different initial-condition freedoms, not test aliases. |
| Same file `test_categorize_3` | Same mesh, hold normalized thermal storage=300 throughout and deactivate its accumulation discretization: isothermal/reduced-thermal variant. |
| Same file `test_categorize_4`, `test_categorize_error` | Fix thermal and E accumulations at0, release their initial holdups and deactivate corresponding discretization for the pseudo-steady approximation; separate error selector checks invalid categorization. Structural matching does not establish approximation error or physical-time accuracy. |
| `cstr_nmpc.main` | Controller3 min,30×2 bounded; plant0.5 min,5×2 unbounded; sample period0.5 min. Inputs are substrate/enzyme feed volume flows. Measurements are C/E/S/P concentrations, outlet T and volume. Setpoint targets P=0.4,S=0, normalized thermal storage=300,E flow=0.1,S flow=2.0,volume=1, all setpoint weights1. Release conflicting initial holdup/volume constraints for the steady-target solve, then form tracking weights1 on differential states and inputs, piecewise-constant inputs, and fix only initial control inputs. Preserve steady-target and rolling-control roles separately. |
| `cstr_reduced.main` | Same controller/plant meshes and hold period but different equations. Fix solvent holdup and deactivate total-volume-flow closure, categorize states, form derivative/algebraic Jacobians through PyomoNLP/SciPy sparse factorization and eigenanalysis, select modes above half the largest eigenvalue magnitude and state participation above0.5, then set selected fast derivatives0, release their initial conditions and deactivate their discretization. Rebuild controller categories; use four concentration measurements. Targets differ from full controller: S=0.1 instead of0; other target fields/weights stay as declared. Approximation error against the unreduced physical plant is required. |

The full script declares simulation_horizon=60 and derives a sample list, but the
actual requested work is one initial half-minute plant propagation plus a loop
`range(1,11)` of ten more windows. Do not report a60-minute closed-loop campaign
from that unused declaration. Both examples need consistent initial-condition
solves, steady-target optimization, first controller solve, plant propagation,
measurement/input loading and horizon advancement with retained failure outcomes.

Full-model random seed=246; measurement noise parameters for(S,E,C,P,T,volume)
are(0.2,0.05,0.1,0.05,5,0.05), input parameters for(S flow,E flow)=(0.02,0.001).
Reduced seed=100; assigned parameters are(0.01,0.005,0.01,0.005,1,0.05), but only
the four selected concentration measurements consume that observation noise.
Although source variables are named variance, `set_variance` stores them unchanged
and `apply_noise_with_bounds` passes them directly to `random.gauss`; their
implemented role is sigma, without a square root. Preserve that selected law or
explicitly identify changed statistics. Bounds default to rejecting and resampling
out-of-bound draws, at most six attempts (`max_number_discards=5`); exhaustion
raises. A clipped or silently accepted failed draw would change the problem.

### Recursive closure, native mapping and remaining evidence

Selected source closure comprises the three example files; the two helper classes
and their state/reaction blocks; CSTR/Mixer/core control-volume and Pyomo DAE/
consistent-initial-condition/arc owners; Caprese NMPCSim, DynamicBlock,
ControllerBlock, categorization/variable/vector/rolling and bounded-noise helpers.
Full plots consume pandas/matplotlib; reduction adds NumPy/SciPy sparse linear
algebra and PyomoNLP/PyNumero's compiled ASL interface. No CSV, trainer, trained
weights or saved-state file is selected; initial states are constructed in memory.
Both control examples explicitly ask Ipopt for ma57, tol1e-6,bound_push1e-8 and
halt_on_ampl_error=yes. The constructor's separate solver uses legacy Ipopt with
tol1e-6,mu_init1e-8,bound_push1e-8. Missing ma57/PyNumero prerequisites hold the
selected execution; a different solver configuration is an attributed selection,
not silently equivalent preparation. Exact executable, Python/NumPy/SciPy/ASL
versions and the reduced example's current API compatibility remain to be frozen
and exercised against core2.13.0.

Native `reactors.pse::CSTR`, aqueous/reaction/inventory/temporal owners and
`workflow/horizon.rs` are actual starting contracts. They do not yet supply this
enzyme family's ordinary coefficients, normalized thermal meaning, complete
initial-state allocation or reduced controller. E02/D03/G05 must map exact
source/case/output identities, physical-time/hold semantics and required/optional
trajectory products there. Required products include all species/complex and
enzyme/substrate-pool inventories/fluxes, reaction rates, normalized-to-physical
thermal accounting, inlet/outlet flow/T, complete trajectories, setpoint solve,
controls/measurements/noise outcomes, reduced-state identity and approximation
error. Exact elemental composition is not inferred for generic enzyme/substrate
labels; conserved pools need their own declared projection. Ordinary kinetic/
caloric records, positivity and operating envelopes, independent conservation,
trajectory/refinement/error checks and closed-loop/reduced native/reference
execution remain open. This finite source closure supplies E01/G01 input decisions
and does not establish E02/G05/H04 acceptance or native MHE.

## Methane/iron fixed-bed inventory closure — 2026-11-05

**Interface-checked, source only:** C `gas_solid_contactors` supplies distinct
0D reservoir-gas/solid-inventory and 1D flowing-gas/stationary-solid models,
their `test_FB0D.py`/`test_FB1D.py` fixtures and `flowsheets/dyn_TGA_example.py`.
Both consume the exact `properties/methane_iron_OC_reduction/{gas_phase_thermo,
solid_phase_thermo,hetero_reactions}.py` closure. These source families supply
E02/E03/F03 inputs; no reference solve, kinetic acceptance or native distributed
bed follows from this census. Oxygen/iron oxidation, moving/bubbling beds and
NETL-32D adsorption/desorption remain distinct required selections.

### Categorical configurations and guards

| Actual owner / choices | Effective construction/scientific boundary |
| --- | --- |
| FixedBed0D | Own ConfigBlock requires dynamic=True and has_holdup=True. Energy defaults enthalpyTotal; its broad EnergyBalanceType enum is accepted, but implementation branches only on none versus non-none, not separate phase/total/internal-energy laws. Only enthalpyTotal/none are selected by these fixtures. Independent gas/solid property packages and reaction package/arguments are explicit. It holds evolving solid mass/composition/porosity/temperature against externally fixed gas composition/T/P, without a gas-throughflow/control-volume material balance. It is not a generic steady vessel or a0D approximation that preserves all1D transport. |
| FixedBed0D energy none | Adds an isothermal-solid constraint at later times rather than merely dropping temperature closure. EnthalpyTotal instead evolves solid energy through the selected reaction heat. Both retain solid material storage; no dynamic/steady switch removes it. |
| FixedBed1D dynamic/holdup | Outer flags inherit UnitModel defaults, but the gas CV is explicitly constructed dynamic=True/holdup=True and solid material accumulation always exists. Parent/unit consistency therefore requires a dynamic enclosing problem; a nominal enum/default is not a supported steady bed. Solid state has no solid-flow transport; stationary inventory is indexed over time/length. |
| FixedBed1D discretization | Actual defaults are10 spatial elements, initial normalized grid[0,1], finite difference, unspecified scheme,3 collocation points. Forward flow resolves BACKWARD difference; reverse flow resolves FORWARD difference; forward collocation resolves Radau. Reverse collocation, opposite upwind choices and non-Radau collocation are refused. The config's descriptive20-element default is stale; the constructor selects10. Mesh/order are numerical settings separate from physical bed dimensions. |
| FixedBed1D balances/reactions | Defaults componentTotal/enthalpyTotal/pressureTotal. Gas property and optional homogeneous reaction package are separate from solid property and optional heterogeneous package. Reaction-package presence selects gas reaction/heat or cross-phase transfer; has_equilibrium_reactions defaults False and needs actual supported package behavior if enabled. Exposed generic balance enum values pass into CV construction; neither an arbitrary enum nor a package name establishes all equations/products used by the bed's specialized solid/transfer closure. |
| Pressure and thermal law | Actual has_pressure_change default=True with Ergun; alternatives simple correlation or no pressure change remain separate. The simple law uses its explicit0.2/s coefficient and density difference; Ergun uses particle size, voidage, viscosity and gas velocity/density. Non-none energy includes interphase gas/solid heat and transport correlations; energy none instead ties gas/solid temperatures to initial solid temperature with appropriate initial/inlet exceptions. No-pressure/no-energy cases must preserve those actual equations rather than losing closure. |

`test_FB1D.py::test_config_validation` retains five direction/scheme refusals
alongside valid default construction: reverse collocation, reverse/BACKWARD
difference, forward/FORWARD difference and collocation with BACKWARD or FORWARD.
Other uninstantiated balance/reaction/thermal/pressure combinations remain
required configuration investigations, not demonstrated science.

### Shared exact science and operating inputs

Gas species are CH4/CO2/H2O on molar basis; solid species are Fe2O3/Fe3O4/Al2O3
on mass basis. The heterogeneous reaction is
CH4+12Fe2O3→CO2+2H2O+8Fe3O4, with inert Al2O3. Its source reaction enthalpy is
136584.3 J/mol_rxn. The selected grain conversion/rate model uses grain radius
2.6e-7 m, reactive volume fraction0.28, activation energy49000 J/mol,
preexponential0.0008 in the order-dependent units, order1.3 and grain molar
density32811 mol/m3. These fixed parameters are not an independently inferred
mechanism from stoichiometry alone. The source grain/conversion/smoothing laws,
porosity and basis conversions remain part of the matched formulation.

Shared solid-property defaults include particle diameter0.0015 m, bed voidage0.35
and particle conductivity12.3 W/(m K). Minimum-fluidization quantities are also
declared by the property owner, but their presence does not make a fixed bed a
fluidized bed. The gas/solid source dictionaries contain exact molecular weights,
formation/caloric/Shomate, density/porosity and gas viscosity/conductivity inputs;
the unit demands the particular subset its equations consume. Reference and
sensible/reaction-heat conventions must remain consistent across phase bases.

| Exact finite fixture / source | Complete selected specification and numerical role |
| --- | --- |
| `test_FB1D.py::build_model` → `TestIronOC` and performance-class wrapper | Forward, reactive, Ergun and full energy;360 s with120 s backward time steps (3 elements),10 BACKWARD spatial elements. Diameter9 m/height5 m. Gas boundary20 mol/s,723.15 K,156500 Pa and fractions(CH4,CO2,H2O)=(0.975,0.02499,0.00001) at all times. Initial gas away from inlet has same fractions/20 mol/s but1273.15 K. Initial solids at every position:1273.15 K,particle porosity0.20,mass fractions(Fe2O3,Fe3O4,Al2O3)=(0.45,0,0.55). The performance wrapper repeats this scientific case; it is not new chemistry. |
| `TestIronOC_reverse_flow::iron_oc` | Same360 s/3 time elements, geometry, boundary and initial chemistry; reverse direction with10 FORWARD spatial elements. Inlet is normalized x=1, so initial gas specifications exclude that endpoint rather than x=0. Reference locations/direction must map physically, not by a common vector ordinal. |
| `TestIronOC_NoReaction::iron_oc` | Forward360 s/3 time elements/10 spatial elements, same geometry/composition and solid initial state; no reaction package and inlet T=1273.15 K, matching initial gas/solid T. Preserve this reaction-free inventory/heat/conservation control separately from the cold-feed reacting case. |
| `TestIronOC_conservation_with_reaction::iron_oc` | Forward reactive/full energy, same geometry/compositions and initial solid/gas states; inlet T=1273.15 K,3600 s/30 backward time elements and20 spatial elements. This longer/higher-resolution conservation case is not a wrapper of the360 s cold-feed run. |
| `test_FB0D.py::TestIronOC::iron_oc` | Reactive enthalpyTotal,3600 s/100 backward time elements. Diameter=height=1 m; initial solids1273.15 K,porosity0.20 and mass fractions(0.45,0,0.55). Reservoir gas at every time1273.15 K/101325 Pa with fractions(CO2,H2O,CH4)=(0.4,0.5,0.1). No gas feed-flow value is selected because the gas acts as an imposed reservoir. |
| Same class `iron_oc_unscaled` | Same physical reservoir/solid inputs and enthalpyTotal, but360 s/10 time elements; a distinct horizon/mesh initialization/solve selection, not only a scaling label. |
| `TestIronOC_EnergyBalanceType::{iron_oc,iron_oc_unscaled}` and `flowsheets/dyn_TGA_example.py::main` | Same0D reservoir and solid inputs, energy none/isothermal solid,3600 s/100 time elements. Scaled/unscaled paths and the worked TGA entry point share this scientific/numerical configuration; retain their initialization/presentation roles without multiplying chemistry. |

No parameter decorators add hidden cases in the two unit test modules. Initial
Fe3O4=0 is explicit selected zero-species behavior; porosity and densities determine
actual mass/volume allocation rather than assigning an invented solid inventory.
Initial gas pressure is constrained through the chosen pressure profile, not
fixed independently at every cell. These details belong to exact native initial
and boundary-condition mapping.

### Initialization, input custody and native acceptance

1D source tests select staged unit initialization, block triangularization and
time-element initialization as separate numerical routes, check restored original
fixed/active states and deliberately overfix an outlet for an initialization
failure. Defaults for selected solves include legacy Ipopt,tol1e-5,
bound_push1e-22,user-scaling; triangular constraint calculation uses eps1e-5.
Retain actual per-route solver settings and full failure reports. 0D similarly
has scaled/unscaled/staged routes and an overfixed-volume refusal. A successful
initialization method does not replace a final original-problem solve.

Conservation fixtures integrate gas boundary mass/enthalpy and gas/solid storage
over the selected time/space meshes. Their Pyomo Integral calculations use the
selected discretization; a finite integral check is not a grid-independent
continuous conservation proof. Required native products include complete gas
outlet/position/time states, stationary solid inventory/composition/porosity/T,
cross-phase reaction/heat, pressure loss, gas/solid storage and original component,
element and energy balances. Chemical element/oxygen transfer must use actual
Fe/C/H/O/Al stoichiometry and molecular weights; gas molar versus solid mass basis
cannot be compared without the authoritative conversion.

All selected coefficients are inline in the three property/reaction modules;
the unit/test/TGA closure additionally consumes core CV/state/initialization,
Pyomo DAE/Integral and the chosen solver. No CSV, trained artifact, external
property binary or saved initialization file is selected by these fixed-bed
fixtures. Original scientific data/formulation references and partly empirical
hydrodynamic/kinetic parameters require B/D02/D03/F03 ordinary-source admission;
the upstream test arrays remain oracle evidence. A different kinetic, transport
or pressure law must be identified as changed science, even if numerically easier.

Native inventory, reaction/state/chemistry and `control-volume-1d.pse` are enabling
owners. The latter's dynamic refusal remains until its actual E03 accumulation
closure exists; these source fixtures do not remove it. The inspected native
process model collection did not contain this Fe2O3/Fe3O4 grain-kinetic bed;
that bounded search is not a whole-tree absence proof. E02/E03/F03 must deliver
native identities, ordinary inputs and complete configured cases, then compare
independent space/time refinements and appropriate interior/boundary operating
states. Source inventories now distinguish seven concrete0D/1D physical/numerical
configurations; missing ordinary records, nondefault categorical closure,
continuous validity, matched execution and all moving/fluidized/adsorption
families remain required and open.

## Held NETL-32D adsorption/desorption closure — 2026-11-05

**Interface-checked, source only:** X's held
`flowsheets/CO2_adsorption_desorption/CO2_Adsorption_Desorption_1DFixedBed*.ipynb`
selects one coupled two-stage experiment, with four wrapper entry points. The
source closure is X's `mod/co2_adsorption_desorption/` gas and solid property
modules, adsorption reaction module and `simulation_utilities.py`, C's
`gas_solid_contactors/unit_models/fixed_bed_1D.py`, and
`core/solvers/petsc.py`. Required, held execution remains its disposition. This
material is neither the zeolite TSA0D model nor the methane/iron reactive bed.

| Stage/role | Actual fixed inputs and transferred state |
|---|---|
| Adsorption | Dynamic bed, diameter9 m, height1 m, 50 backward finite-difference spatial elements. Time set is fixed at0,27000,54000,81000,108000 s: four time elements. Inlet3.544 mol/s,303.15 K,124520 Pa, gas fractions CO2 .0408/H2O .0875/N2 .7517/O2 .12, fixed throughout the stage. Initial gas flow, temperature and composition match that inlet at every spatial point; interior initial pressure is not prescribed by this helper. Initial solids are442 kg/m³,303.15 K, fractions H2O_s1e-8/Car1e-8/SiO1 at every spatial point. |
| Desorption | New dynamic bed of identical geometry/spatial mesh, time set0,7200 s, one time element. Inlet10 mol/s,470 K,106525 Pa, H2O1 and CO2/N2/O2 each1e-8, fixed throughout. Initial gas flow/temperature/composition are replaced by that feed. Every spatial solid density and component fraction is populated from the adsorption trajectory's last stored time; solid temperature is reset to470 K. These values seed all desorption time points, then the initial-time values are fixed. Gas inventory is not transferred. |
| Initial-state/domain guards | Degrees of freedom must be zero. At each stage's initial inlet the helper requires ideal-gas inlet velocity no greater than the fixed minimum-fluidization velocity .0091 m/s, using solid temperature, R8.314 and pi3.14. This is a fixed-bed admissibility check, not a fluidized-bed capability. |

Both stages inherit forward gas flow, component-total material and total-pressure
balances with Ergun pressure loss; no reverse-flow selection is supplied.
`energy_balance_type=none` constructs gas temperatures tied to the local initial
solid temperature away from inlet/initial points and solids tied to their own
initial temperatures. Thus each stage is isothermal at its imposed temperature;
neither heating/cooling dynamics nor an energy-conserving temperature jump is
solved. The wrapper's helper `ntfe`/`nxfe` are mesh inputs, not an enumerated
scientific parameter sweep. The base/test wrappers divide the scalar `horizon`
by10 **after** constructing the adsorption time set and never use that changed
scalar before redefining it for desorption. Doc/user wrappers omit that statement;
all four therefore retain the same actual108000 s adsorption experiment.

The gas helper is an ideal-gas N2/O2/CO2/H2O package with inline molecular masses
and formation enthalpies,32 Shomate coefficients,16 viscosity coefficients,
16 conductivity coefficients and4 diffusion volumes. These named tables in the
fixed source are the coefficient closure; there is no runtime NIST/Perry query.
Their source attributions are leads for ordinary-data provenance, not an admitted
temperature envelope: a single Shomate range per component is used, without
piecewise range switching. The solid helper's fixed parameters are particle
diameter1.5e-4 m, minimum-fluidization voidage .5, bed voidage .45, nominal
density442 kg/m³, heat capacity1.13 **J/(kg·K)**, conductivity1.36 W/(m·K), and
reference temperature298.15 K. Solid molecular masses are .018/.044/.06 kg/mol
for H2O_s/Car/SiO; SiO is a surrogate material with a placeholder molecular mass.
The declared heat-capacity unit/value must be resolved scientifically rather
than silently interpreted as kJ/(kg·K). Actual density is a state inventory,
not merely a constant-density parameter.

Two heterogeneous molar reactions transfer one gas H2O to H2O_s and one gas
CO2 to Car. Respectively their fixed reaction enthalpies are−52100/−64700 J/mol,
entropies−78.5/−174.6 J/(mol·K), activation energies28200/57700 J/mol, and
pre-exponential parameters56234.13/100 in the source's declared mol/(m³·s)
units. The rate-constant law also multiplies by smoothed temperature, floored
at298.15 K; dimensional treatment of that extra temperature factor belongs to
ordinary formulation/data resolution. Equilibrium depends on pressure and
temperature. CO2 uptake includes squared amine-site availability with fixed
amine loading1900.46 mol/m³; water uptake and both reverse terms depend on
solid loading/density. This is a kinetic two-sorbate model, not a pure CO2
equilibrium isotherm. Initial solid fractions sum to1.00000002 and desorption
feed fractions to1.00000003; adsorption feed sums to one. Preserve these exact
reference inputs and establish an explicit composition allowance/native
admissibility disposition rather than normalizing them implicitly.

Initialization uses block triangularization, Ipopt with ma27, maximum100
iterations, user scaling and variable-calculation epsilon1e-5. Each stage then
uses C's ASL PETSc executable interface with backward Euler, initial step200 s,
time relative tolerance20, Krylov relative tolerance1e-10, trust-region Newton,
up to1000 SNES failures, and saved trajectory. Initial conditions are solved
again (`skip_initial=False`, default `petsc_snes`). The executable and compatible
PETSc trajectory reader (`PetscBinaryIOTrajectory`, directly importable or in a
`petscpy` directory alongside PETSc/IDAES binaries) are prerequisites in addition
to Ipopt/ma27 and the pinned Pyomo/core environment. Generated solver/trajectory
files are outputs, not preexisting saved initialization or scientific input
artifacts. The four helpers load no CSV, external parameter file, trainer or
serialized model; pandas/matplotlib/NumPy serve trajectory reporting/plots.

`simulation_utilities.py` derives spatial/time integrals of reaction heat,
adsorbed-water/carbamate inventories, inlet/outlet CO2, recovery, purity,
productivity, annual capture and thermal duties. Its actual metric definitions
need separate disposition before comparison: water desorbed subtracts the same
final inventory from itself; preheating and precooling both multiply by
303.15−470 K; productivity uses1e-6 times kg for an adsorbent quantity labelled
tonnes; annualization assumes8760 hours and omits heating/cooling time. Both
stages still flow forward despite a helper's interpretation of desorption heat
sign as flow reversal. The final spatial plotting call also repeats adsorption
instead of selecting desorption. These are source limitations, not permission
to adopt corrected outputs without a documented reference distinction. Require
independent component/gas-solid inventory and thermal/sign/unit controls, with
chosen reported metrics and allowances stated explicitly.

Bounded inspection of native reference declarations, modeling and horizon/dynamic
owners found no NETL-32D case. E04/F02/F05 still require native solid uptake,
distributed gas storage/transport, explicit stage transfer and matched outputs;
the existing spatial dynamics refusal remains. Ordinary coefficient/provenance
and physical-domain admission belong to B; these fixed external dictionaries
are oracle inputs only until that work closes. PETSc availability, exact
examples/core compatibility, metric repairs/dispositions and numerical/native
qualification remain open. An ephemeral AST comparison established the wrapper
difference and the inline table cardinalities without importing models or
executing any solver; this supplies source census evidence only.

## Condensing feedwater-heater population — 2026-11-05

**Interface-checked, source only:** C's power
`unit_models/{feedwater_heater_0D,feedwater_heater_0D_dynamic}.py`, their
`tests/test_feedwater_heater*.py`, and `helm/{condenser_ntu.py,tests/test_condenser_ntu.py}`
define the following finite first power-equipment population. Their shared
property dependency is the already identified IAPWS95/general-Helmholtz water
closure, including saturation, liquid heat capacity/density and two-phase
enthalpy/energy derivatives, not just liquid-pump pressure work.

Each feedwater-heater owner has three independent Boolean section switches,
all defaulting true. In `(desuperheat,drain_cooling,drain_mixer)` order the
declared topology selections are000,001,010,011,100,101,110,111. Condensation
is present in every selection; optional branches connect steam→desuperheater→
mixer→condenser→drain cooler as selected, and feedwater traverses selected
sections in the opposite order. These are eight source topology demands per
owner, not eight executed qualification results. Section areas and heat-transfer
coefficients, drain feed, pressure/balance policies and valid properties still
need well-posed fixed/free specifications for each selection.

The steady owner uses generic0D countercurrent LMTD exchangers by default.
Its condensing section constrains outlet enthalpy to saturated-liquid enthalpy
at the **hot outlet** pressure plus fixed `enth_sub` (default zero). A positive
value therefore raises outlet enthalpy in the actual equation; the variable's
name is not sufficient authority for a subcooling convention. Steam extraction
flow is normally free, solved from condensation and heat transfer. Drain mixer
outlet pressure equals the steam inlet pressure; no inequality enforcing a
higher drain pressure is added. Section property packages inherit the overall
selection unless explicitly overridden. Dynamic/holdup flags inherit the base
unit/exchanger contracts; the steady test below does not establish transient
support for this owner or dynamic mixer storage.

The dynamic owner's condensing section instead derives from
`HelmNtuCondenser`: cold liquid inlet flow×liquid heat capacity supplies the
capacity rate, NTU=UA/capacity rate, effectiveness=1−exp(−NTU), and the driving
temperature is hot-inlet saturation temperature minus cold-inlet temperature.
Its outlet enthalpy is fixed to saturated-liquid enthalpy at the **hot inlet**
pressure. This differs from the steady owner when pressure changes. The shell
liquid volume follows the horizontal circular-segment level geometry, scaled
by shell-volume fraction and section length; hot-side pressure change is a
positive gravity×level×outlet-liquid-density head. The arcsine requires level
between zero and diameter, with positive diameter/length and physically valid
shell fraction/volume. Source variables do not enforce these bounds. It also
unconditionally accesses hot-side volume and deltaP, so a default declaration
is not proof of a usable no-holdup/no-pressure-change configuration. Preserve
these prerequisites/refusals explicitly. The mixer is forced algebraic
(`dynamic=False`); the selected desuperheat/cooler below are algebraic too.
Fluid inventories in the condenser are not wall thermal storage or spatial
resolution.

| Source selector | Scientific inputs, roles and required observations |
|---|---|
| `tests/test_feedwater_heater.py::test_fwh_model` | Steady111, water. Steam inlet P201325 Pa/h60000 J/mol; flow seed100 mol/s remains free. Drain1 mol/s at201325 Pa/h20000. Feedwater400 mol/s at101325 Pa/h3000. Each section area1000 m²; U condense100/desuperheat10/cooler10 W/(m²·K). Initialization max_iter50; source checks extracted steam flow98.335±.01 mol/s and zero DOF. Preserve all section duties/outlet states and whole-unit mass/energy controls beyond this scalar. The separate unit-consistency constructor has no fixed scientific experiment and does not double the solved population. |
| `tests/test_feedwater_heater_dynamic.py::build_unit` initial role | Dynamic111, time0–60 s with2 backward time elements (0,30,60). Condenser both sides have pressure change and fluid holdup; cooler/desuperheater are algebraic, cooler no holdup. Same inlet P/h/drain/feedwater as steady case; steam flow100 seed is unfixed. Areas condense600/desuperheat85/cooler100 m²; respective U1010/145/675. Condenser cold deltaP0, initial/fixed level .275 m, diameter1.4 m, shell fraction .675, section length6.4 m, cold-side volume1.3 m³. Initial material/energy accumulations are fixed zero; other-time accumulations receive zero guesses. Initialization establishes the initial regime, with source drain flow67.20607±.01 mol/s. |
| `test_feedwater_heater_dynamic.py::test_fwh_model` transient role | From that initialized regime, fix cooler hot-outlet drain flow to .95 times initial flow for t≥30 s, free condenser level except its fixed initial .275 m. Thus changes apply at both30/60 s; they are not an open-left interval. Solver tol1e-7,ma27,max_iter50. Source checks level .2759±.01 m at60 s and zero DOF. Freeze full shell/tube inventories, heat, pressure head and initial/trajectory conservation, not only this loose terminal level comparison. |
| `helm/tests/test_condenser_ntu.py::test_condenser_steady` | Algebraic, no side pressure change; steam100 mol/s,h30000 J/mol, pressure9000 Pa is only a seed and remains free. Cooling water1200 mol/s,101325 Pa,h1800 J/mol; area1000 m²,U500 W/(m²·K). Initialization `unfix="pressure"` solves shell pressure; source target14036.3360 Pa with1% relative allowance. |
| `test_condenser_dynamic` in the same file | Same fixed inputs and algebraic condenser (`dynamic=False`) on a dynamic parent time0–3 s with4 backward elements. It checks the same pressure at every time. This is repeated algebraic evaluation, not a condenser-storage transient. Stream-table test separately exercises default unsolved state reporting, not a third physical experiment. |

`HelmNtuCondenser.initialize` exposes exactly three solve specifications:
free hot flow (default), free cold flow, or free hot pressure; other strings
are explicitly refused. The pressure fixture above supplies one role; the
dynamic feedwater heater uses free hot flow. Free cold-flow and the remaining
section configurations stay required without an invented fixture or unsupported
claim. Initializer snapshots/restores fixed values, fixed/free flags and active
equations using in-memory serializer dictionaries; no supplied JSON file,
CSV, trainer or saved initialization is read by these isolated tests. Its
inherited balance choices still require the selected property methods; arbitrary
energy/material enums are not automatically usable with the NTU/saturation law.

Actual integrated consumers preserve additional topology selections and cannot
be collapsed into111: C `supercritical_steam_cycle/supercritical_steam_cycle.py`
selects001 for fwh1,111 for fwh2/3/6/7 and110 for fwh4/8. C
`subcritical_power_plant/steam_cycle_flowsheet.py` selects the dynamic owner
with001/011/010/111/110 for fwh1/2/3/5/6 respectively. Those complete plant
boundary, geometry, initialization, controller and boiler/turbine closures
remain separate F02 work. X `mod/power_gen/steam_turbine.py` additionally
selects an algebraic NTU main condenser with cooling-water200000 mol/s,
500000 Pa,h1800 J/mol, area5000 m² and U15000; X `ngcc.py` later changes
its cooling-water enthalpy to1260 and area to1000 during plant initialization.
The NGCC saved-initialization closure and final assembled specification have
not been replaced by this equipment census.

No matched condensing feedwater-heater case was found in bounded native reference
declarations. Existing sensible-heat exchanger and liquid-pump witnesses do not
qualify wet-steam heat transfer, saturation-location differences, level geometry
or dynamic inventories. D02/E01/E02/F02 need explicit native case/data/output
correspondence and independent conservation checks. Ordinary water/caloric data
and geometric/heat-transfer provenance remain separate from the source's fixed
oracle inputs/expected scalar answers. Ephemeral AST inspection confirmed14/23/14
fixed-input calls across the three test files and no hidden parameterization,
and resolved the seven/five integrated heater selections without model imports
or reference execution. Upstream skip guards do not waive required prerequisites
in the local fail-on-missing reference route.

## SOC replication inputs, variants and cache roles — 2026-11-05

**Interface-checked, source only:** C's
`power_generation/unit_models/soc_submodels/tests/test_herring_replication{,_thin,_interconnect,_heat_loss}.py`
provides a concrete electrochemical benchmark population. The recursive model
closure is `solid_oxide_cell.py`, `solid_oxide_module_simple.py`, channel,
porous/conductive slab, contact resistor, triple-phase boundary and `common.py`.
Stacks additionally use explicit **ideal** natural-gas property configurations
for H2/H2O/N2 and O2/N2; importing `natural_gas_PR.get_prop` does not select PR
in these cases. IAPWS95 saturation functions convert measured dew points into
steam feed fractions, so the water external-function/artifact closure remains
required even for an otherwise gas-only cell benchmark.

The actual input CSV `tests/data_cache/herring-et-al-data.csv` has five rows:

| Case | N2/H2 standard cm³/min for ten cells | Dew point/inlet °C | Maximum stack voltage |
|---|---|---|---|
| 1 | 1011/205 | 48.5/800 | 12.5 V |
| 2 | 2017/411 | 70.4/800 | 13 V |
| 3 | 1017/410 | 83.8/800 | 16 V |
| 4 | 2018/411 | 82.9/800 | 16 V |
| 5 | 2018/411 | 83.2/830 | 16 V |

Every case uses85910 Pa, ten cells, and oxygen feed3500 standard cm³/min for
the stack at .21 O2/.79 N2. Source standard-volume conversion uses101300 Pa,
273.15 K and R8.31; do not substitute another standard condition. Fuel H2O
fraction is saturation pressure at the measured dew point divided by85910 Pa;
the measured dry N2/H2 flows then determine wet total flow and fractions.
Single-cell flow is one tenth of the stack flow. The first voltage is computed
from the source enthalpy/entropy Nernst expression (F96485,R8.31,reference
pressure101300 Pa); each case then solves11 evenly spaced cell voltages through
0.9×maximum-stack-voltage/10. This is five configured feeds ×11 actual voltage
points per replication variant, not an unbounded parameter catalog.

| Variant/source role | Actual physical/numerical selection and expected-data source |
|---|---|
| Base replication | Steady cell, cocurrent,10 axial finite volumes from11 uniform faces; one through-thickness volume per resolved fuel electrode, oxygen electrode and electrolyte. Solid holdup constructed (`has_holdup=True`) but parent dynamic=False and gas holdup defaultsFalse. Temperature variation across thickness and all four contact resistances included; no interconnect flux. Reads `case_1.csv`…`case_5.csv`. |
| `_thin` replication | Same feeds/axial mesh, both electrodes replaced by thin resistance elements, no electrode x-grids. Resolved electrolyte remains. Fuel/oxygen electrode area resistance is the corresponding base resistivity×23/21 µm thickness, with contact fraction1 and thermal exponent0. Electrode diffusion, thermal mass and spatial resolution are not established by a matching electrical resistance. Reads `case_1_thin.csv`…`case_5_thin.csv`. |
| `_interconnect` replication | Resolved electrodes/electrolyte, no holdup, adds periodic interconnect flux and one through-thickness interconnect volume. Reads `case_1_interconnect.csv`…`case_5_interconnect.csv`. |
| `_heat_loss` replication | Same interconnect configuration with a heat-loss port, but fixture fixes total heat loss to zero. It actually reads the **interconnect** cache files. `case_1_heat_loss.csv`…`case_5_heat_loss.csv` are shipped but not consumed by these replication functions. Do not infer a nonzero-loss sweep from their filenames. |
| Custom voltage-drop extension | Interconnect configuration with six extra drops (fuel/oxygen electrodes, electrolyte, interconnect and both reaction boundaries), all fixed zero. Repeats the55-point sweep against the interconnect caches. This checks the zero extension, not a degradation law or nonzero drop response. |
| Exchange-current modifier extension | Heat-loss-port configuration, total heat loss zero, both reaction-boundary logarithmic exchange-current modifiers fixed zero. Repeats the55 points against the interconnect caches. This does not qualify a nonzero modifier/degradation law. |

Across these six declared regression entry points there are330 source solve
requests; zero extensions/zero heat loss deliberately repeat baseline science.
This count describes only this bounded replication population, not independent
scientific experiments or all SOC configurations. All four original replication
functions have identical ASTs: actual differences enter through model/parameter
configuration and selected cache. Each run uses Ipopt, user scaling,tol1e-6,
max_iter100 and checks optimal/status-ok plus `model_check` at every voltage.

Shared fixed cell geometry is .08×.08 m active plane,1.09e-3 m fuel/oxygen
channels and100 W/(m²·K) channel heat-transfer coefficients. Fuel electrode
thickness23 µm, porosity .37,tortuosity3, solid heat capacity450 J/(kg·K),
density3210 kg/m³,conductivity2.16 W/(m·K),resistivity8.856e-6 Ω·m with
zero thermal exponent; oxygen electrode21 µm,.37,3,430,3030,2.16 and
1.425e-4 Ω·m respectively. Electrolyte thickness140 µm,heat capacity470,
density5160,conductivity2.16,resistivity pre-exponential1.07e-4 Ω·m and
temperature exponent dividend7237 K. Included interconnect is460 µm,630,
7700,25 and1.2e-6 Ω·m with zero thermal exponent. Contact pre-exponentials
are .55e-4 at the two electrode/mesh interfaces and .15e-4 at the two
mesh/interconnect interfaces, with fraction1 and zero thermal exponent.
Fuel/oxygen reaction-boundary exchange-current pre-exponentials5.5e8/7e8,
activation energies100/110 kJ/mol, both charge-transfer coefficients1, and
composition powers H2 .5,H2O .1/O2 .5 are fixed, not trained by these tests.
Stoichiometry includes gas species, vacancy, oxide ion and electrons; preserve
its per-electron convention and the negative-current electrolysis sign.
`common.py` owns inline gas caloric/entropy, elemental and collision/diffusion
tables; its15 admitted gas species are not15 experimentally exercised feeds.

There are20 cached case CSVs, each11 rows with14 physical/report columns plus
index: voltage, average current density/axial temperature, and fuel/oxygen
outlet flows,temperatures,pressures and compositions. They are outputs of the
upstream model, compared at relative tolerance .003; they are **regression
oracles**, not independent measurements or ordinary coefficient data.
`sweep_5_kazempoor_replication.csv` is a separate44-point, headerless
current-density/voltage curve from a literature-model replication. Only case5
compares interpolated curve current against minus modeled current, with3%
relative/50 A/m² absolute allowance. It is distinct from both the five operating
rows and the cached cell results. The selected cocurrent, zero-loss model does
not reproduce a crossflow furnace experiment without model discrepancy; the
Herring/Kazempoor scientific attributions in the pinned tests and explicit allowances retain that
boundary. No trainer, saved initialization, ROM or external CFD executable is
loaded by these replication tests. Ordinary calibration/provenance and test-data
independence still need their own admission.

Initialization supplies a separate fixed benchmark at1.288 V,1103.15 K,
85910 Pa,fuel .000484863 mol/s per cell with H2 .0630491/H2O .6273812/
N2 .3095697, and the same oxygen feed above. Each of the four variants has
single-cell and10-cell-module initialization roles; module fuel flow is
.00484863 mol/s. These check spatial current and outlet states, then release
11 input degrees of freedom and verify initializer preservation. Heat-loss
tests separately fix1.5 W/cell or15 W/10-cell module and check the changed
current/compositions/outlet temperatures using `ipopt_v2` with scaled writing.
Those are the nonzero heat-loss cases, not a hidden55-point loss sweep.

The cell declarations additionally expose independent thin/resolved electrolyte
and, when selected, interconnect choices; contact on/off; co/counterflow;
gas/solid holdup; custom voltage and exchange-current modifiers. Source refusals
exclude gas holdup without other holdup, heat loss without interconnect flux,
crossflow, thickness grids supplied for thin regions, nonzero/nonunit mesh
endpoints or non-increasing grids, and `include_temperature_x_thermo=False`.
Module/cell heat-loss configuration must agree. These constraints bound
admissible configurations without treating the Boolean Cartesian product as
qualified science. `test_solid_oxide_cell.py` adds six steady structural
fixtures: counterflow/contact/holdup; cocurrent/no-contact/no-holdup; each with
interconnect; and no-contact interconnect heat-loss or exchange-current ports.
Their axial/electrode/electrolyte/interconnect meshes and species differ from
the replication benchmark; these build/unit checks are not solved voltage
curves. Remaining submodel thermal/material-boundary, gas-holdup/dynamic,
SOEC-design and X controller/operating-table/saved-initialization populations
remain required and unresolved here.

No native matched SOC declaration/case was found in bounded process/reference
inspection. E03/F05/G05 still require ordinary electrochemical/transport data,
physical domain and charge/element/energy closure, native layer/port/work sign,
state/pool correspondence and independent numerical evidence. Ephemeral
AST/CSV checks confirmed the five×eleven sweep constants, identical sweep
functions, actual cache readers,20×11 cached rows and44 literature-curve
points. No model import, reference solve, measured result or compatibility
claim follows from those source checks.

## Modern SOC controller schedule and initialization closure — 2026-11-05

**Interface-checked, source only:** X
`docs/power_gen/solid_oxide_cell/soc_pid_control*.ipynb` imports
`mod/power_gen/soc_dynamic_flowsheet.py::SocStandaloneFlowsheet`. The four
notebook wrappers share one scientific construction; base/test add zero-DOF
and terminal assertions omitted by doc/user. No training workflow is called.
This transient plant/control family is separate from the steady Herring
replication, including different cell/material data and a condensing recycle.

| Configuration/consumer | Actual selection and boundary |
|---|---|
| Notebook physical configuration | Dynamic parent, thin oxygen electrode and electrolyte, resolved fuel electrode, resolved interconnect,10 axial finite volumes. Countercurrent cell,400000 cells,.2345×.2345 m active plane,873 µm channels,1 mm fuel electrode with porosity .326/tortuosity3. Solid holdup present; gas holdup absent. No contact-resistance or heat-loss port selected. |
| Model dynamics | Cell solid energy, three exchanger wall inventories and two electric-heater wall inventories are dynamic. Exchanger fluids, heater fluids, blower, mixers/splitters and condenser flash are algebraic. Three countercurrent crossflow exchangers use6 Legendre elements×1 collocation point, hot shell; sweep exchanger inline tubes, fuel hot/medium exchangers staggered. Each heater uses2 Radau elements×2 points, inline tubes. These spatial and wall-storage configurations do not establish gas residence/inventory dynamics. |
| Gas/condensation properties | Air/sweep O2 .2074/H2O .0099/CO2 .0003/N2 .7732/Ar .0092 and fuel H2/H2O/Ar/N2 use ideal gas packages. The condenser uses liquid+vapor PR with Stryjek–Vera alpha, water kappa1−.0665 and H2/N2/Ar kappa1 zero, with custom alpha/first/second temperature derivatives. This is not default PR or ideal condensation. Core cubic-root external functions, natural-gas coefficient tables and IAPWS95 dependencies remain required. |
| Fixed fresh-feed and hardware inputs | Makeup378.15 K/120000 Pa,Ar .0008/N2 .0002; H2/H2O set by schedule. Blower feed288.15 K/101300 Pa with the sweep composition above, isentropic efficiency .85, discharge120000 Pa. Flash deltaP0. `_set_initial_inputs` supplies inline tube diameters/thicknesses, wall/fouling/heat-transfer parameters and bundle dimensions for all five thermal units; `_define_cell_params` supplies this plant's electrode/electrolyte/interconnect/kinetic parameters. These exact named method bodies plus the selected value overlay below close the fixed parameter inputs; substituting Herring parameters changes the model. |
| Inherited optional configurations | Helper defaults are resolved oxygen electrode/electrolyte, no interconnect, no heat loss, actual dynamics when parent dynamic and quasi_steady_state=False; grid defaults to11 uniform faces. Notebook overrides thin regions and interconnect. Heat loss requires interconnect, as in C; its helper branch fixes heat loss zero. `quasi_steady_state=True` forces unit algebraic dynamics. These alternative configurations remain required selections/refusals, not variants executed by this notebook. |

Two actual operating scenarios are exposed, with `maximum_production` selected
in every shipped wrapper and `power_mode` selectable in its explicit branch.
Both use six knots0,3600,3900,11100,11400,22200 s: hold1 h,ramp5 min,
hold2 h,ramp5 min,hold3 h. Maximum-production schedule uses rows
`maximum_H2,maximum_H2,power,power,maximum_H2,maximum_H2`; power-mode schedule
reverses those two row roles. Unknown scenarios raise. The CSV has three rows
and24 numeric columns; `minimum_H2` exists but is not selected by either
schedule. It is not a third executed scenario.

| Selected endpoint quantities from `soec_flowsheet_operating_conditions.csv` | `maximum_H2` | `power` |
|---|---|---|
| Net H2 target kg/s; voltage reference V; fuel-outlet H2 target | 2;1.321508935;.748999939 | −.9191892;.885541324;.620805129 |
| Fresh makeup/sweep flow references mol/s | 1324.539114/2261.014098 | 630.603772/7006.177366 |
| Fuel/sweep heater duty references W | 1000001.329/1000004.824 | 100000/100000 |
| Fuel inner/outer temperature references K | 942.2212333/985.3387465 | 952.1497399/963.1956808 |
| Sweep inner/outer temperature references K | 968.0334026/985.9509337 | 936.9772859/1012.960812 |
| Stack-core temperature K | 992.9985544 | 992.3074377 |
| Fuel/sweep/vapor-gas recycle ratios | 1/1/.00010001 | .631647422/.999999718/4.999999555 |
| Condenser vapor outlet K; fresh H2/H2O fractions; H2 controller proportional gain | 323.15;1e-14/.999;200 | 323.15;.969/.03;−100 |

Recycle **ratio** is recycle fraction/(1−fraction); ratio1 means fraction .5,
not complete recycle. The19 consumed CSV columns are endpoint policy inputs,
not measured time series. Five columns are not scheduled: `current`,
`oxygen_out`, `hydrogen_in`, `condenser_heat_duty` and
`h2_production_rate_controller_gain_i`. Source constructs derivative-equals-
fixed-rate equations for ramped references, with backward discretization at
the six knots and right-endpoint rates. Voltage reference/target, fresh H2/H2O
fractions, H2 production target and its proportional gain instead **step** to
the row at each knot (notably3900 and11400 s). Initial ramp equations are
deactivated, initial ramped values fixed. Preserve that exact mixed ramp/step
policy and its times, including fresh-composition jumps; interpolation of all
columns would be a different controller experiment.

Seven controllers are constructed with integral initialization calculation
disabled. Fuel/sweep inlet temperature use inner PI on heater electric duties:
Kp100000/200000, integral time900 s, bounds0–10 MW, smoothing1000 W and
back-calculation gain .5/900 s⁻¹. Outer P controllers on fuel/sweep outlet
temperature change inner setpoints with Kp .75 and no output bounds. A PI
controls fuel-outlet H2 with voltage: Kp−2,integral time240 s,bounds .7–1.6 V,
smoothing .01 V,back-calculation gain .5/240 s⁻¹. Stack-core temperature uses
P sweep-flow control,Kp−50,bounds1500–10000 mol/s,smoothing10. H2 net
production uses P makeup-flow control,bounds1–1500 mol/s,smoothing1, and the
scheduled gain above. Declared but unused integral/derivative time scalars
do not turn these P/PI controllers into PID. Generic helper performance bounds
and steady-target constraints are **not called** by this notebook; their lower
heater-power/voltage limits are not this controller's acceptance envelope.

| Reference artifact | Actual role and unresolved compatibility obligation |
|---|---|
| `max_production.json.gz` | Selected maximum-production initial values, serializer format4, dated2025-09-20, metadata reports23011 serialized components. |
| `power_mode.json.gz` | Selected power-mode initial values, format4, dated2025-09-20, metadata reports23013 components. |
| `min_production.json.gz`, `neutral.json.gz` | Shipped but not loaded by these two branches. Minimum snapshot is format4 from2025-09-20; neutral is format4 from2024-04-02 and contains an older flowsheet class spelling. Keep their other consumers/disposition open; neither is an implied fallback initializer. |
| `mod/power_gen/soc_dynamic_template.svg` | Reporting template read by `write_pfd`; generated SVG and PETSc trajectories are outputs. Template is not scientific input data. |

Both selected snapshots contain steady time-zero values and no controllers,
not complete transient trajectories. Notebook loads `StoreSpec.value()`, which
overlays variable values without importing fixed/active flags or bounds and
defaults to ignoring missing entries in the selected core serializer. It then
fixes initial fuel/interconnect solid energy and mixed-gradient state, heat
holdup of three exchangers/two heaters, and PI integrals; initial mean-cell
temperature equations are activated, selected wall-temperature equations
deactivated. Thus silent component/index mismatches must be detected during
readiness; a file parse or matching format number does not establish the
required initial-state coverage. Parameter values in these overlays must also
be reconciled with authored inline inputs before independent ordinary data
admission. CSV and saved values are oracle operating/initial-state inputs, not
ordinary parameter provenance or validation truth.

Direct dynamic `initialize_build` is explicitly unsupported; the notebook uses
the saved-value route and a subsequent initial solve. `fix_initial_conditions`
refuses the non-thin oxygen/electrolyte configuration; steady helper
`make_performance_constraints` refuses multiple time points. The separate
steady initializer's existing-file `load_from` branch references `init_log`
before assigning it: resolve that compatibility/readiness defect if selecting
that route, without claiming it was exercised here.

The actual dynamic solve is ASL PETSc backward Euler, initial dt .1 s,
rtol .001,step bounds .001–3600 s,trust-region Newton,max25 SNES failures,
max50 nonlinear iterations,absolute residual1e-6 and relative/step residual
tolerances zero. It saves visualization trajectories, keeps files and solves
initial conditions with Ipopt/ma57 (user scaling,tol/constr_viol_tol1e-8,
max_iter300). Compatible PETSc executable/trajectory reader, ma57, cubic-root
and water functions, and exact Pyomo/core/examples serializer compatibility
are prerequisites. No additional trainer/ROM/parameter file is loaded by this
selected helper/notebook path.

Every solver result must terminate optimally. Base/test terminal assertions
belong only to the default maximum-production return: current−191441360.747 A
at .1% relative allowance; fuel inlet/outlet941.622/985.483 K and oxygen
inlet/outlet968.256/985.767 K at .1 K absolute; heater duties790423.648/
1078951.237 W at .1% relative. The alternate power-return branch has no matching
separate endpoint assertions in these wrappers. Full trajectories additionally
report H2 production/consumption, power, inlet/outlet flows/compositions,
recycles, temperatures/gradients, wall states, currents, controller states and
condensation duty. Neither plots nor endpoint agreement alone closes charge,
element/energy, initial-inventory, ramp/step continuity, actuator saturation
or antiwindup controls. Native SOC/storage/controller mapping remains absent
at this matched scope; ordinary domain/data, alternative configurations and
independent reference/native execution stay required. Ephemeral AST/CSV/gzip
inspection established wrapper assertion differences, actual files/metadata,
time-zero snapshot structure and scheduled-column roles without model import
or a solver run.

## Grid market and multiperiod input population — 2026-11-05

**Interface-checked, source only:** C `apps/grid_integration` owns bidder,
self-scheduler, tracker, coordinator, forecasters, generator data and period
builders. The following source cases distinguish market optimization from
generator physics and actual clock advance. No market/provider simulation was
executed, and none of the examples selects a thermodynamic power-plant model.

| Actual selector/model role | Configured experiment and required distinctions |
|---|---|
| `tests/util.py::ExampleModel` thermal fixture | Generator10_STEAM,bus5,power30–76 MW,constant marginal cost30, with cost expression30×power. Model data carries min-down4 h/min-up8 h,120 MW/h ramps,startup/shutdown30 MW,initial status9/initial output42 MW and four cost bid points30/45.3/60.7/76; startup costs are7355.42/10488.35/11383.41 after4/10/12 h. The actual fixture only constructs bounded power and linear cost: no commitment/ramp constraints. Updating its `pre_P_T` value cannot qualify absent dynamics. |
| `tests/test_tracker.py::tracker_object` | Four-period horizon,implement1 hour, CBC. Dispatch[30,40,50,70] MW at2021-07-26,17:00. Source checks all four tracked powers and last delivered power30 MW. Objective sums cost plus under/over-delivery penalties:10000 for implemented time0,10000/3 for remaining periods. Soft deviation variables mean an infeasible dispatch need not make the optimization infeasible; acceptance needs actual deviation/feasibility policy. |
| `tests/test_bidder.py::bidder_object` | DA and RT horizons4,three equal scenario copies, synthetic DA forecasts constant29 then31 against marginal cost30,RT forecasts identically zero. DA low/high-price bid curves are distinct roles. RT has realized DA prices29 and dispatch0 in every period,DA forecast29. Both `include_default_p_cost=True` andFalse are tested: omission leaves only the minimum-load cost point and changes final capacity/bid curve. The fixture mutates shared model data for that role, requiring deliberate state isolation in a local reference. |
| `tests/test_coordinator.py::coordinator_object` | Explicit fixture parameterization thermal/renewable (two roles),DA48/RT4/tracking4,three scenarios,one implemented hour. Renewable fixture has0–200 MW/zero marginal cost. Tests assemble static parameters, signals and callback handoffs; they do not run an independent market/plant. |
| `tests/test_PEM_Parameterized_bidder.py` | Algebraic wind/PEM bidder,DA/RT6 h,renewable200 MW,PEM100 MW,marginal value30. Synthetic2020-01-01 capacity factors DA[.1,.1,.1,.8,.8,.8],RT[1,.2,.2,.75,.75,.75],DA/RT prices10/20. Realized DA dispatch[20,20,20,120,120,180] covers surplus above PEM capacity,within capacity,and wind below commitment. Source asserts every piecewise marginal→total cost curve. It uses a dummy `OptSolver`, not a solved electrolyzer; PEM300 MW against renewable200 is an explicit refusal. |
| `examples/thermal_generator.py::ThermalGenerator` | Separate mixed-integer thermal dispatch/commitment model with four production segments,minimum up/down history,startup/shutdown events,ramping and prior power. It derives heat-rate/fuel/startup cost and60-minute ramps from bundled generator/bus data; no heat/mass balance or temperature model is present. Before updates, internal `pre_on_off` defaults1 and prior power=Pmin while exported model data has initial status−1/output0. Freeze actual initial/history handoff explicitly; source metadata alone does not resolve that difference. |

The bidder objective sums DA revenue,RT deviation revenue,weighted model cost
and underbid penalty(default10000) over periods and scenarios. There are no
supplied nonuniform scenario probabilities in these fixtures; summing identical
scenario contributions is not a statistically validated forecast. `SelfScheduler`
has its own nonanticipativity/schedule path and `fixed_to_schedule` policy;
the integration selection below sets it true. `PEMParametrizedBidder` is a
different algebraic bid construction and does not use scenario optimization.
Integer-positive scenario/implemented-hour checks,required model methods/
attributes and compatible solver checks are actual constructor refusal roles;
do not invent a process capability from successful adapter construction.

`tests/test_integration.py::TestDoubleLoopIntegration` supplies two separate
Prescient runs, bidder and self-scheduler. Both use `ExampleModel`, DA48 h,
RT/tracker4 h,ten synthetic scenarios,one implemented hour, and generator
10_STEAM. They simulate two days starting07-10-2020,SCED frequency60 min/
horizon4,RUC horizon48,CBC for dispatch/commitment,RUC MIP gap .01,zero reserve
factor,LMP day-ahead pricing,settlements enabled,out-of-sample/persistent
forecast errors enabled. CBC RUC options are `feas=off,DivingF=on`; price/
contingency/reserve thresholds1000/100/5. The selected test checks existence
of15 output CSVs for each run, **not** their power balance,prices,profit,
commitment/network constraints or actual state-transfer correctness. Those
remain native/reference acceptance products.

The recursively referenced `idaes/tests/prescient/5bus` input closure is all11
CSV files: `bus.csv`(5 buses),`branch.csv`(7 branches),`gen.csv`(8 generators),
`initial_status.csv`,`reserves.csv`,`simulation_objects.csv`,
`timeseries_pointers.csv`,and four load/renewables files. The16 time-series
pointers resolve only to the four shipped data files. Each DA file has8784
hourly2020 rows; each RT file105408 five-minute2020 rows. Load columns are
areas1/2; renewable columns include1_HYDRO,4_WIND,2_RTPV,10_PV. Reserve table
has a header but no requirements. Preserve raw metadata/parser readiness:
`simulation_objects.csv` declares DAY_AHEAD dates at2020-01-01 and REAL_TIME
dates at2020-12-31 despite the full-year data and selected July simulation;
one initial-status row has an extra empty trailing field. Actual Prescient
interpretation must be established, not silently repaired or inferred from
the source file count. These operating/network/cost inputs need ordinary
data provenance independently of their oracle/provider role.

The actual thermal-generator Prescient example uses the same five-bus files
and48/4/4 h horizons/ten scenarios/one implemented hour, but replaces
`ExampleModel` with the commitment model above. Its `PlaceHolderForecaster`
reads four inline24-value arrays of DA/RT mean/standard-deviation prices from
`examples/utils.py`, tiles by hour modulo24, samples NumPy independent normal
draws and clips negatives to zero. No seed is set in the selected plugin;
freeze random-stream/sample custody before matched execution. The synthetic
test forecaster instead returns constant DA predictions and zero RT prices;
it is not this stochastic daily-price model. Forecast arrays are reference
assumptions,not fitted/validated market forecasts.

Prescient,Egret transmission utilities and CBC are explicit executable
prerequisites; the pinned core's `gridx-prescient>=2.2.3` optional dependency
is only an upstream lower bound,not a frozen compatible local environment.
Exact reference/provider versions and import/API closure must be selected
without moving core2.13.0. Upstream skip/importorskip guards do not waive
the local fail-on-missing contract. No trainer,serialized generator model or
saved model initialization is read by these selected grid fixtures; generated
market/bid/tracker CSVs are outputs.

`multiperiod/tests/test_multiperiod.py` independently selects eight categorical
layouts:5 time points × optional4 days × optional3 years × optional3 scenarios.
For no-days/no-years,days only,years only,and both, the respective block counts
are5/20/15/60 without scenarios and15/60/45/180 with three scenarios. A ninth
manual builder uses five periods. The process is only nonnegative x,y with
x+y=1; initialization fixes y=.5/guesses x=.5,then releases y and links
adjacent periods' y values. Day/year labels are repetition indices,not elapsed
durations or inter-day/inter-year inventory laws; no cross-scenario
nonanticipativity or probability model is supplied by these fixtures. Period
links occur within each day/year/scenario; the stated DOFs1/4/3/12 (tripled
for scenarios) retain those independent groups.

Additional explicit controls build two periods without initialization,
unfixing,or linking and expect warnings; stochastic periodic callbacks are
warned about and **not applied**,not enforced cyclic closure. A three-period
per-time-options dictionary bypasses initialization; a two-period declaration
with that same three-entry dictionary produces a warning. Contradictory
initialization x+y=1 andx+y=2 must raise `InitializationError`. Rolling
`advance_time` in the nonstochastic builder deactivates an old block,appends a
new one and rebuilds links/optional endpoint periodic constraint; it does not
integrate a differential inventory law. Preserve these missing-link/failed-
initialization outcomes and independently qualify actual physical period
coupling where a native application consumes it.

Native horizon/study owners are G05 starting points,not matched bidder,
market-network or thermal-commitment acceptance. Original market feasibility,
time/settlement conventions,history state custody,scenario reproducibility,
failure treatment and ordinary data remain open. Price-taker clustering,
design/commitment,backcaster/perfect-forecaster detailed populations remain
separate required closure. Ephemeral AST/CSV inspection resolved actual
two-role coordinator fixture,period layouts,input-pointer closure and raw
annual data cardinalities; no CBC,Prescient,Pyomo model or native scientific
execution occurred.

## Grid price-taker, clustering and forecast cases — 2026-11-05

**Interface-checked, source only:** the remaining selected C grid owners are
`pricetaker/{clustering,price_taker_model,design_and_operation_models,unit_commitment}.py`
and their four test modules,plus `forecaster.py` and
`tests/{test_backcaster,test_perfectforecaster,test_model_data}.py`. Their
constructors,expression checks and surrogate inputs are required in their
actual roles; none is a solved physical generator qualification by itself.

| Price-signal/case selector | Exact data and configuration |
|---|---|
| `test_clustering.py::sample_data_fixture` | Sole file input `pricetaker/tests/lmp_data.csv`,8760 rows with time1…8760 and `BaseCase_2030` price column;24-value periods produce365 days. The fixture opens bare `lmp_data.csv`,so reference working-directory/path resolution must be explicit. Source naming is not ordinary price provenance. |
| `dummy_data_fixture` |24 inline values representing twelve2-value samples around three clusters: four corners each near(.5,.5),(10.5,.5),(5.5,10.5). `cluster_lmp_data` uses3 clusters,seed42,expects those centers with weight4 each. These are synthetic clustering inputs,not a market year. |
| Cluster-count selection | Full file uses elbow search k4…30 inclusive,seed42,expected optimum7. Twelve-sample synthetic data uses silhouette k2…7,expected2; `PriceTakerModel` silhouette k2…6 on the same data expects3. Preserve the different search windows and actual scoring outcome; do not substitute a single universal cluster count. |
| Price-taker full-vector and representative cases | Source `test_mp_model_full_year` actually uses only the24-value synthetic vector,one24-step period,weight1. Other fixtures select3 representative periods×2 steps (weights4 each),3×4 for update-error cases,2×12 for adjacent-period links/ramp/commitment,3×6 for periodic links,and1×12 for startup counts. An append-only configuration test uses[−4,5,2,50,−3.2],representatives3,horizon2,seed30; negative prices are admitted. These are actual selected configurations,not8760-step plant solutions. |

Daily-data conversion explicitly refuses a horizon longer than the vector;
nonmultiple lengths warn and discard trailing values (range8/horizon3 drops2).
K-means uses `random_state=seed` but does not explicitly pin algorithm/init/
`n_init` in its constructor; exact scikit-learn version/defaults and label
correspondence therefore belong to reference readiness. Count search requires
2≤kmin<kmax<number of samples and recognizes only silhouette/elbow; degenerate
elbow/normalization behavior has separate diagnostic tests. Core SciPy spline
interpolation,NumPy,pandas,matplotlib and scikit-learn are the transitive
analysis dependencies. There is no external trainer executable or saved
clustering artifact in these cases.

The period callback `simple_flowsheet_func` constructs x+y=1 and dummy
power/binary operation variables; its links even connect prior power to next
binary mode in a structural test. These are expression/index witnesses,not
meaningful physical port coupling. Representative periods link adjacent steps
only within each selected representative; periodic tests connect each period's
first/last power. Weighting represents repeated samples without restoring
chronology or inter-period inventory by itself.

The design/operation fixture selects capacity300–400,installation binary,
capex1000,fixed annual O&M30,operating power0–400,cost23×power+50×on,
startup/shutdown cost5/4,revenue price×power plus fixed100,and fixed cost50.
The initialized readback fixture uses constant price5 over24 steps,assigns
powers1…24 and synthetic startup/shutdown patterns without solving feasibility.
Separate design variants use fixed capacity650/capex40/fom35 or variable
power150–650 with polynomial capex coefficients[4,5,6,7],fom[2,3] and
constant auxiliary ng_flow42. Operation variants include explicit callbacks,
no automatically declared operation/LMP variables,default single startup
versus named hot/warm/cold startup times4/8/12,and polynomial surrogates.
Polynomial constant terms multiply operation status; coefficient lists are
inline declared formulas,not fitted coefficients or a trainer/data closure.
The tests also distinguish explicit auxiliary variables/equalities from
direct expression outputs.

| Commitment/storage consumer | Source configurations and boundary controls |
|---|---|
| `test_unit_commitment.py` capacity/ramp cases |10 steps,capacity650 or variable capacity150–600,minimum operating fraction .3. Ramp case startup/shutdown .3,up .2,down .15 fractions of capacity per period; first-period ramp inequalities are absent. Scalar-vs-variable capacity are distinct construction roles. |
| Startup/shutdown cases |10 steps with min-up3/min-down4 and no startup categories;10 steps with same minima and hot2/warm5/cold8 transitions;5 steps with min-up/down2 and transitionNone or empty dictionary;8 steps with min-up2/min-down3 and a single hot2 category;10 steps with min-up2/min-down4 and hot3/warm6. Lower-level UC accepts that single category while `OperationModel`'s startup-type validator requires at least two named categories. Preserve that API boundary rather than declaring a global refusal. First binary-transition/min-up/min-down constraints start only where their local history fits; prior physical state is not supplied by these structural cases. |
| Price-taker wrappers |Capacity100/minimum fraction .3 with full24 or3×2 layout; design-capacity ramp startup .3/shutdown .4/up-down .2 with full24 or2×12; min-up3/min-down4 with/without installation variable and full24/2×12. Duplicate additions,missing operation blocks and inconsistent UC data are explicit failure roles. |
| Automatic power-cycle/storage fixture |Inline polynomial ng_flow[1,2],vom5; unit named ngcc has capacity500,min-up/down3,minimum fraction .3,startup/shutdown .3,up/down .5. A unit named coal_pp only adds commitment; a default storage block is linked/periodic when requested. These names do not instantiate NGCC or coal thermodynamics. |
| Fixed storage |Time interval .14,charge efficiency1,discharge .8,inventory50.5–100,charge≤10.5,discharge≤20. Inventory change=(charge−discharge/.8)×.14. |
| Variable storage design |Time interval .5,charge/discharge efficiencies .9/.8,inventory .2×capacity…capacity,charge≤design charge limit,discharge≤.8×that limit. Variable bounds become inequalities; they are not evaluated as fixed numeric limits. No battery material/electrochemical law or loss calibration is selected. |

Overall cashflow defaults are30-year life,8% discount,20% corporate tax,
straight-line depreciation,optional annualization factor and cash-inflow scale1.
Objective selection exposes annualized `npv`,lifetime_npv and net_profit;
only net_profit objective construction is explicitly selected in its positive
test. Representative-day weights feed cashflow aggregation. Missing hourly
cost/revenue/design callbacks warn and produce zero corresponding terms;
missing overall cashflows/unknown objective names refuse. These synthetic
financial formulas need explicit periods,units and ordinary economic provenance
before an assessment result is scientific acceptance. Construction/string
equality is not an independently optimized cost/profit witness.

Forecast cases have finite,distinct time semantics:

- `test_backcaster.py` supplies three24-hour DA days valued1/2/3 and RT
  days10/20/30 at `test_bus`,default history limit10 days. RT request
  2022-05-11,hour18,horizon4,three samples chooses newest-to-oldest days;
  DA request hour0,horizon48,two samples wraps across the retained price
  vector,producing3→1 and2→3 day pairs. Forecast date does not select actual
  dated weather/market history in this implementation. This is deterministic
  cycling,not random resampling.
- History cap1 drops oldest two days; hourly stats append a completed24-hour
  day of price15 then clear its partial buffer. Eight further synthetic days
  exercise retention at10 days; DA callbacks similarly retain the last10.
  Missing bus raises `ForecastError`. Input guards require nonempty dicts of
  lists,at least24 values in multiples of24; invalid max-days values are
  types `"10"`,`"ten"`,[10] and values−10,0,−0.0 (six parameter rows).
- `test_perfectforecaster.py` supplies one hourly2020-01-01 DataFrame at
  `Caesar`/`303_WIND_1`: DA prices0…23,RT1…24,DA capacity0… .23,RT .01… .24.
  DA24-step and RT4-step reads are exact data access,not forecast validation;
  scenario-count argument is unused. DataFrame or CSV path are allowed,
  ndarray/list are two explicit rejected input forms. At exhaustion the source
  appends values from the beginning **once**,rather than arbitrary cyclic
  repetition or a missing-coverage refusal. Long/out-of-range windows need
  explicit admissibility controls before adoption.

Generator-data validation separately has23 parameter rows:8 nonnumeric
fields,5 negative values,3 capacity/startup/shutdown values below Pmin,
2 empty bid lists,2 invalid initial statuses and3 inconsistent initial-output
roles. Individual guards additionally cover missing Pmin/Pmax bid endpoints,
invalid startup/cost curves,fixed commitment and generator/bus names. These
refusals constrain accepted reference inputs; they are not extra physical
experiments. The selected forecast/price-taker tests load no additional data
file beyond `lmp_data.csv` and no saved fit/initialization. Native G05/G06
still require matched period/history/stock/commitment,weighted assessment,
incomplete-coverage and failed-horizon controls plus independently sourced
ordinary inputs. Exact provider versions,numerical cluster execution and
native/reference qualification remain open. Ephemeral AST/CSV inspection
confirmed8760 price rows,the24-value synthetic vector and the23 parameter
rows; no clustering,market,optimization or scientific reference was run.

## sCO2 surrogate pipeline and input closure — 2026-11-05

**Interface-checked, source only:** X
`docs/surrogates/sco2/{alamo,pysmo,omlt}` each exposes training,property
embedding and a file named `flowsheet_optimization`,with local `properties.py`
and `flowsheet_optimization.py`. These remain three required pipelines with
three requested stage roles each; actual source gaps below do not remove the
optimization demand. Full/reduced/ReLU/ONNX and reformer/PySMO-basics families
are separate scope,not exercised by this smooth sCO2 neural configuration.

All three pipelines read the shared `500_Points_DataSet.csv`: exactly500
rows,eight columns. Only the first four columns are selected: pressure,
temperature,`CO2SM.CO2_Enthalpy`,`CO2SM.CO2_Entropy`. Other enthalpy/entropy
columns are not interchangeable targets; they have different numeric values.
All status/error fields are zero; source performs no additional filtering.
Observed input range is7.460891–34.993814 MPa and306.215965–999.971989 K;
selected H range−403924.714779…−360255.753654 and S−67.269005…20.819994.
Local packages label outputs kJ/kmol andkJ/(kmol·K),numerically J/mol and
J/(mol·K); flow iskmol/s andpressure MPa. Data-generating property formulation,
reference datum and provenance are not established by these labels or the
zero status column. Admit ordinary data independently from this oracle dataset.

| Training role | Exact selected options and reproducibility boundary |
|---|---|
| ALAMO | Shuffle all500 rows with pandas random_state0,then400/100 split with seed500. Fit both H/S on the400 rows using constant/linear functions,two-variable powers1/2,monomials2/3,ratio power1,maximum10 terms per output; write `alamo_run.alm` and `alamo_surrogate.json`. Validation rows feed only plots. Executable availability is queried,but the unavailable branch only prints and later uses an undefined fit; missing trainer remains an execution failure,not excluded training. |
| PySMO polynomial | Shuffle500 without an explicit initial random_state,then400/100 split with seed500. Maximum degree5,multinomialsTrue,internal training split .8,cross-validations10,and five extra terms: P×T²,P²×T²,P²×T,P/T,T/P. Core's selected default least-squares route is Pyomo; internal cross-validation reseeds NumPy by fold index. The initial unseeded permutation still affects training membership. Save `pysmo_poly_surrogate.json` with declared P7–40,T306–1000 bounds. |
| Keras/OMLT | Seed NumPy46,Python1342,TensorFlow62; shuffle500,then form400/100 plotting partitions with seed500. However,the neural fit/scalers use **all500 rows**,not just the400-row partition. Min/max normalizers for inputs/outputs therefore use the full dataset; Keras `validation_split=.2` supplies its own100-row holdout. Network2→20→20→20→20→2,tanh hidden/linear output,Adam,MSE loss,MAE/MSE metrics,250 epochs with unspecified batch default. A best-validation checkpoint `.mdl_co2.keras` is written,but not reloaded: exported `sco2_keras_surr/sco2_keras_model.keras` uses the final in-memory model. `PYTHONHASHSEED`/CUDA environment settings occur inside the already running process after TensorFlow import; they do not prove process hash/device/determinism settings. |

ALAMO training saves its fitted object **before** reconstructing an in-memory
object with wider P7–40/T306–1000 bounds. The shipped JSON therefore actually
declares P7.460891–34.993814 andT306.215965–999.971989. Plotting the widened
object does not change that saved artifact. PySMO/neural metadata instead
declare the wider range. Neural input offsets/factors are(7.460891,306.215965)/
(27.532923,693.756024),output offsets/factors
(−403924.714779,−67.269005)/(43668.961125,88.088999). The shipped `.keras`
archive contains config,weights andmetadata identifying Keras3.4.1 saved
2024-07-17; this is artifact provenance,not the currently qualified reference
runtime. Record the actual version/scaler/weights for each retraining run
without overwriting the immutable shipped oracle.

Each local property package creates one component CO2 and one phase labelled
Liq,with independent fitted H(P,T)/S(P,T) equations. That label neither supplies
phase equilibrium nor proves valid liquid/supercritical behavior. The state
variables are flow,P,T; no density,heat-capacity,potential-derived integrability
or general transport closure is supplied. Core `SurrogateBlock` intersects
the state bounds P7.38–40/T304.2–1033.15 with actual artifact input bounds.
The effective ALAMO domain is thus its narrower saved domain,whereas PySMO/
neural effective domains areP7.38–40/T306–1000. Evaluation plots and bounded
embedding are distinct domain controls. A fit to H/S does not establish their
thermodynamic consistency or derivatives over an enlarged envelope.

Embedding notebooks principally declare classes; their existence is not a
constructed/solved property probe. Four training wrappers and four flowsheet
wrappers per family have identical scientific ASTs. Embedding wrapper
differences include an ALAMO future-division import,an extra unused PySMO os
import,and a real PySMO user-wrapper typo: default unit key `temperatureo`
instead of `temperature`. Preserve that readiness defect rather than
deduplicating it as a proved compatible wrapper. The separate local
`properties.py` is what the flowsheet imports by bare name,so execution must
bind the intended family directory; an unrelated module of that name cannot
satisfy the dependency. ALAMO/PySMO load their matching JSON; neural loads
both `.keras` and `idaes_info.json` and embeds OMLT **FULL_SPACE smooth**
equations. No ReLU MIP/complementarity or standalone ONNX evaluation is selected.

The three same-topology flowsheet scripts instantiate fixed-input steady
rating cases,not a design optimization: seven Heater units (boiler,two
pseudo-shell/tube recuperator pairs,two coolers),three isentropic pressure
changers,two splitters andone mixer,with14 arcs. The HTR-tube outlet is **not
connected back to the boiler inlet**,which remains an independently fixed
boundary. Thus a closed-cycle material/thermal carryover claim is unsupported.
Both pseudo recuperator shell/tube pairs impose equal/opposite heat duty;
they have no area,UA,pinch or exchanger feasibility law. No objective,
free-design search or constrained optimum is defined despite the filenames.

| Fixed rating inputs | Selected values |
|---|---|
| Boiler boundary |121.1 kmol/s,685.15 K,34.51 MPa; outlet893.15 K,deltaP−.21 MPa. The commented760°C alternative is not a second constructed case. |
| Turbine |Pressure ratio1/3.68,isentropic efficiency .927. |
| Recuperator pseudo hot sides |HTR outlet489.15 K/LTR354.15 K,eachdeltaP−.07 MPa. LTR cold deltaP0,HTR cold−.07 MPa; paired duties equal/opposite. |
| Flow/pressure/cooling |Bypass fraction .25; main/bypass compressor efficiencies .85 andpressure ratios3.8; CO2 cooler outlet308.15 K/deltaP−.07 MPa; FG-cooler fraction .046,outlet483.15 K/deltaP−.06 MPa. |

Scripts initialize sequentially,expand arcs,print DOF,call Ipopt once,report
unit states andnet turbine-minus-compressor work. They do not assert solver
termination,physical closure,error bounds or an optimal result. Upstream
notebook outputs are historical reports,not current execution evidence.
Required optimization therefore needs an explicit bounded case/native mapping,
fixed/free decisions,objective,feasibility and artifact-domain obligations at
G04/F02; it cannot be closed by replaying this rating file. Even the rating
requires all inlet/outlet H/S/P/T/flows,duties/work and original conservation
checks,not just the printed work scalar.

Runtime input closure is the shared CSV,three local property/flowsheet owners,
two JSON fits or neural `.keras`+metadata,andcore trainer/surrogate/scaling/
plotting/embedding APIs. `CO2_flowsheet.png` is presentation input; generated
PDFs,ALAMO control/trace/checkpoint files andtrainer outputs are not additional
ordinary physical data. Exact ALAMO executable/version is required only for
retraining,not separately selected JSON evaluation/embedding; it remains
required scope. PySMO requires its selected least-squares solver stack; neural
requires exact TensorFlow/Keras,OMLT andloadable archived weights; all rating
cases require Ipopt. No external property-EOS provider is consulted by the
fitted H/S package itself. Remaining installed-version,ordinary dataset/datum,
independent predictive/derivative/error,closed-cycle and actual optimization
work remains open. Native fitting is a starting workflow,not evidence of
these matched trainer/artifact/property/consumer stages. Ephemeral AST/CSV/
JSON/ZIP reads confirmed500 rows,artifact bounds/scalers/layers,wrapper
differences,14 arcs andabsence of any objective; no trainer,model,network
evaluation,reference solver or native scientific case was executed.

## Reformer surrogate inputs, family selection and optimization — 2026-11-05

**Interface-checked, source only:** X
`docs/surrogates/{alamo/alamo_flowsheet_optimization,pysmo/pysmo_flowsheet_optimization,omlt/keras_flowsheet_optimization,best_practices_optimization}*.ipynb`
and `mod/surrogates/AR_training_methods.py` close the selected reformer source
paths. Each four-wrapper group has one scientific AST after assertion removal.
Training,artifact evaluation,embedding and bounded optimization remain separate
G04 demands; no helper availability set is an execution/qualification result.

`reformer-data.csv` has2800 rows,15 columns. Inputs are Bypass_Fraction
.1–.8 andNG_Steam_Ratio .8–1.2. Ordered targets are Steam_Flow,
Reformer_Duty,AR,C2H6,**C3H8,C4H10**,CH4,CO,CO2,H2,H2O,N2,O2.
These exact output labels/order,flow/duty units,composition basis and generating
process/datum need explicit native/ordinary-data correspondence. A picture
`AR_PFD.png` is presentation input,not a process declaration or conservation
closure. The fitted examples themselves instantiate independent output Vars,
not reformer reaction/equilibrium/energy laws or normalized composition
constraints; nonnegativity and sum-to-one are not enforced on fitted fractions.

| Training family | Actual selected100-row experiment and artifact |
|---|---|
| Standalone ALAMO |Unseeded pandas sample100 from2800,then80/20 split seed100. Fit13 outputs using constant/linear,multi2power[1,2],monomial[2,3],ratio[1,2],max10 terms/output. Requires ALAMO executable; FileNotFound catches a specific unavailable-trainer case and guards later stages. Local reference still fails missing required training. Save fitted JSON before constructing a wider-bound in-memory object. Shipped subdirectory artifact's bypass bounds .110144928–.8 differ from root artifact .110144928–.789855072. |
| Standalone PySMO polynomial |Same unseeded sample100/80–20 seed100;degree6,multinomialsTrue,internal .8 split,10 cross-validations,default Pyomo least squares. Saves/loads `pysmo_poly_surrogate.json` in its working directory with declared .1–.8/.8–1.2 bounds. A shipped file named `pysmo/pysmo_poly_surr.json` is a different artifact and is truncated after `input_labels`; it fails JSON parsing and cannot be an implicit substitute. |
| Standalone Keras |NumPy46/Python1342/TensorFlow62 seeds,100 sampled rows,80/20 seed100 plotting split. Fit/scalers use all100 selected rows with Keras validation_split .2;outer20-row plot set is not independent of that fit/normalization. Network2→40→40→13,tanh hidden/linear output,Adam,MSE/MAE metrics,1000 epochs. Best-loss checkpoint `.mdl_wts.keras` is written but not reloaded; final weights are exported. Process hash/CUDA environment assignments occur after TensorFlow import,so runtime determinism/device setup remains to establish. |
| Shared helper additional families |Polynomial as above; Gaussian RBF with Pyomo solution andregularizationTrue; kriging with numerical_gradientsTrue/regularizationTrue; neural same2×40 tanh/1000-epoch route; ALAMO same basis/term choices. All use the shared sample100,80/20 seed100; comments saying seed2800 do not change the actual seed. Numerical-gradient kriging selects the core's SciPy gradient fitting route; no external CFD/property source is consulted. |

The helper imports TensorFlow andall five trainer families eagerly,andsets
global seeds on import. `train_load_surrogates(retrain=False)` still reads/
samples the dataset even when fits exist; it trains absent artifacts. Its
filename/control behavior must be resolved explicitly:

- Existing root ALAMO JSON bypasses training but is **not added** to the
  returned type set; freshly trained ALAMO is added. Missing ALAMO is caught
  and omitted. Therefore a comparison can silently lose required ALAMO scope;
  the local declaration must retain it as held,not interpret set membership
  as the required denominator.
- The helper requests `pysmo_kriging_surrogate.json`,but the shipped root
  file is `pysmo_krig_surrogate.json`. Default non-retrain mode therefore
  still requires kriging training. Polynomial/RBF filenames do match their
  shipped root artifacts; all three PySMO types are added to the returned
  set whether trained or merely found.
- Root and`omlt/keras_surrogate/keras_model.keras` archives both identify
  Keras3.4.1 andthe2→40→40→13 network. **Neither folder contains the required
  `idaes_info.json`** with labels,bounds andnormalizing scalers. Core loading
  requires that file. The helper only tests for `.keras`,skips retraining
  andadds KERAS to its set; that does not make either shipped pair executable.
  Standalone training can generate a new complete artifact,but it is a new
  fit,not recovery of missing metadata for the shipped weights.

All three standalone notebooks embed outputs in positional order with
**C4H10 before C3H8**,opposite to dataset/artifact output labels. Core maps
by position,so the named hydrocarbon outputs are exchanged. The shared
best-practices functions use the correct C3H8-before-C4H10 order. Do not copy
the swapped mapping or silently repair oracle outputs; retain an explicit
reference-distinction/correctness control before native comparisons.

The standalone physical-input probe fixes bypass .5 andNG/steam ratio1,
solves with Ipopt andreports all13 outputs. It then frees both inputs,
maximizes fitted H2 andconstrains fitted N2≤.34 within fitted/input bounds.
Source asserts |H2−.33|≤.01 but only N2≤.4+1e-8,looser than its actual
constraint; no independent global optimum or solver termination proof is
provided by those scalars. ALAMO reloads its saved narrower-bound artifact,
so in-memory widened training plots do not establish the optimization domain.
Neural embedding explicitly selects full-space smooth OMLT,not ReLU or
global mixed-integer optimization.

Best-practices calls the shared helper,then explicitly removes KERAS from
the returned trainer set. With shipped root ALAMO present,the actual returned
comparison families are therefore the three PySMO types; ALAMO participates
only if freshly trained/added. The notebook randomly selects10 dataset cases
after helper training/loading; exact case identities depend on preceding
training paths andtheir random-state effects. Each selected family gets ten
fixed-input embedded solves,reporting relative steam/duty errors andabsolute
C2H6/CH4/H2/O2 errors,then one H2-maximizing,N2≤.34 solve. The helper type
order comes from a set,not a declared stable campaign order. Bare exception
handlers retry solves once but do not provide a qualified failure policy,
optimal termination check or original physical-domain acceptance. Plots/
displayed tables alone do not establish error tolerances or held-out accuracy.
All five families andtheir requested stages remain required despite these
selected-loop omissions.

Recursive reference inputs are the shared2800-row CSV,selected root/family
JSON orcomplete neural archive+metadata,helper source,andcore ALAMO/PySMO/
scaling/sampling/surrogate/OMLT/plotting owners. GeneratedALAMO controls/traces,
PDFs,checkpoints andnew fits are outputs,not original physical data; historical
model files require exact filenames andformat readiness,not name-based aliasing.
Reference placement must isolate generated fits from immutable shipped
artifacts andbind bare working-directory file paths. Exact ALAMO,TensorFlow/
Keras,OMLT,SciPy/NumPy/pandas andIpopt versions remain to freeze with core2.13.0;
Python imports or Keras archive metadata do not establish this environment.
Native G04/data/physical-output mapping still needs explicit implement/reuse/
refuse selections for each stage; existing fitting/surrogate machinery is not
a matched full pipeline. PySMO-basics/benchmark helpers,core parameterized
sampling/regression andneural/ONNX representation cases remain separate
required closure. Ephemeral AST/CSV/JSON/ZIP checks established2800×15 data,
wrapper correspondence,actual file gaps/truncation andlayer metadata; no fit,
artifact evaluation,optimization or native scientific case was executed.

## Core neural and ONNX formulation population — 2026-11-05

**Interface-checked, source only:** C
`core/surrogate/tests/{test_keras_surrogate,test_onnx_surrogate}.py` selects
four Keras artifacts andone supplied ONNX artifact,plus five synthetic ONNX
activation cases. Their owners are `keras_surrogate.py`,`onnx_surrogate.py`,
`omlt_base_surrogate_class.py`,`surrogate_block.py` andscaling utilities.
These configurations preserve exact evaluation/representation/solver roles,
not a claim that every architecture/activation/formulation combination works.

Keras artifact basenames are `T_data_1_10_10_2_{sigmoid,relu}` and
`PT_data_2_10_10_2_{sigmoid,relu}` under `tests/data/keras_models`.
Every model has two10-node hidden layers with the named activation andtwo
linear outputs EnthMol/VapFrac. ZIP metadata identifies Keras3.2.1,saved
2024-04-18; it does not establish current compatibility. The actual loader
`load_keras_json_hd5` reads `.keras` **andthen `.weights.h5`**,overlaying
weights. Same-basename architecture JSONs are shipped but not read by this
loader. Freeze both executable artifacts per case rather than assuming the
`.keras` weights alone determine the reference. Input/output labels,scalers
andbounds come from the test's inline declarations,not a missing sidecar.

| Exact Keras case | Inputs,formulation and actual witness |
|---|---|
| `test_keras_evaluate`,four models |T-only points365/370/375 K; PT points(360 K,1.05 atm),(370 K,1.10 atm),(380 K,1.15 atm),where1 atm=101325 Pa. Two predicted outputs at every point are compared to fixed upstream network outputs with rtol/atol1e-4. These are four artifact-evaluation configurations,not independent thermodynamic measurements. |
| Auto-created variables,sigmoid |Each of T/PT uses full-space smooth and reduced-space smooth OMLT,Ipopt;fixed365 K andPT pressure111457.5 Pa. Four formulation cases. |
| Auto-created variables,ReLU |Each of T/PT uses ReLU complementarity with Ipopt andReLU big-M with GLPK. Four more cases; NLP/complementarity andMILP are separate backend demands. No claim of globally solved nonlinear design follows from fixed-input feasibility. |
| Supplied-variable mapping |PT sigmoid/full-space/Ipopt at365 K,111457.5 Pa repeated through five mappings: scalar inputs,indexed input group,scalar outputs,indexed output group,andboth supplied inputs/indexed outputs. These are distinct positional/ownership controls for the same science. |
| Round-trip |PT sigmoid save/load with generated `idaes_info.json`,then identical three-point evaluation andthe fixed365 K/111457.5 Pa full-space solve. Files are temporary outputs of this test,not independent training inputs. |
| No scaler |PT sigmoid uses raw network inputs(.5,.5),labels still Temperature_K/Pressure_Pa,bounds[−3,3] each,with no physical scaling. Full-space Ipopt matches raw network evaluation. This is a representation control,not .5 K/.5 Pa thermodynamics. |

There are15 explicit Keras solver sites in these selected functions (eight
automatic formulations,five supplied-variable mappings,round-trip,no-scaler).
They compare embedded outputs to direct Keras predictions andassert optimal
termination. A constant objective is used; none optimizes a physical process.
Constructor refusals cover mismatched input/output scaler labels,bounds keys
andunknown formulation string. Model format/layer/activation restrictions
must also be established against exact installed TensorFlow/Keras/OMLT.

Physical-domain mapping uses T360–380 K andPT pressure101325–121590 Pa.
T-only input offset/factor369.873737/5.776343; output offsets
(59519.621464,.556033),factors(14805.365235,.432934). PT input offsets
(369.983611,111421.319811),factors(5.836047,5917.954504); output offsets
(54599.629980,.403307),factors(14654.226615,.430181). Exact order and
per-column scaling are mandatory. OMLT scales declared input bounds before
formulation; big-M bounds are not optional cosmetic metadata. Expected
sigmoid predictions include VapFrac−.001982489 and1.002572366 for PT and
1.000048027 for T-only. Preserve network correspondence while separately
dispositioning physical fraction validity; clipping/renormalizing is a model
change andcannot silently become oracle behavior.

`tests/data/create_keras_models.py` supplies the separate training source
closure: `T_data.csv`100 rows at101325 Pa,T360–380 K,and`PT_data.csv`10000
rows across101325–121590 Pa/T360–380 K. Six CSV columns include liquid/vapor
enthalpies,but only mixture EnthMol/VapFrac are trained. Script sets Keras
random seed42,first80/20 data split,mean/std scalers from the80% training/
validation partition,then Keras validation_split .2 andbatch16. Thus T uses
64 train/16 internal validation/20 test; PT6400/1600/2000. T sigmoid/ReLU
epochs1000/500;PT200/50,Adam/MSE. Best-validation checkpoints are not reloaded
before saving final models/weight overlays. Data-generating science,original
units/provenance,current trainer reproducibility andthe relation of a new
fit to frozen inline scaler constants remain separate obligations; no
retraining occurs during the artifact/embedding tests themselves.

The supplied ONNX case is `tests/data/onnx_models/net_Calcite_ST.onnx` plus
`net_Calcite_ST_idaes_info.json`. Twelve ordered inputs are feed_pH,
pressure_bar_feed,Na,Cl,Ca,Mg,HCO3,SO4,K,Sr,Ba,HCl;only pressure's unit is
explicit in its label,so chemical-concentration/output units andthe original
calibration domain still need source admission. Metadata bounds are pH4–12,
pressure1–406.32,Na0–135.99,Cl0–180,Ca/Mg/HCO3/Sr0–10,SO4 0–100,
K0–40,Ba0–.1,HCl0–2000. Input offsets are pH4/pressure1/others0;
factors are those respective span widths. Output Calcite_ST offset0/factor98.19.

Both manual construction andsave/load round-trip embed **reduced-space**
OMLT andsolve with Ipopt at pH9.5,pressure1.01325,Na .230858556000748,
Cl .106701648328453,Ca .0245274696499109,Mg .0311348703689873,
HCO3 .430482141673564,SO4 .182204065,K .000500561,Sr .000761853,
Ba .000025,HCl10. They assert Calcite_ST40.4270772546529 with .1%
relative allowance. This is one physical-input probe with two construction/
artifact-custody routes,not a trained geochemical equilibrium acceptance.
`ONNXSurrogate.evaluate_surrogate` explicitly raises `NotImplementedError`;
import/embedding must not be reported as standalone ONNX evaluation support.
Exact graph/opset/layer compatibility andnative chemical-domain interpretation
remain readiness/implementation work.

Five parameterized construction cases synthesize a1→2→1 ONNX graph at
opset13 with fixed weights/biases andinput[−1,1]: ReLU/big-M,Sigmoid/reduced,
Tanh/reduced,Softplus/reduced,andLogSoftmax/reduced expected to refuse.
They **patch OMLT's private activation allow-list in an autouse test fixture**;
these cases therefore do not establish the unmodified production parser's
support set. They construct formulations without numerical solves,anddo not
exercise the supplied calcite graph's other declared formulations. Full-space,
big-M andcomplementarity remain exposed ONNX roles whose actual compatible
cases/backends must be selected rather than inferred from enum membership.

Input closure comprises fourpaired Keras model/overlay artifacts,inline
scalers/reference predictions,two optional training CSVs,the ONNX binary+
metadata,andcore/OMLT/TensorFlow/Keras/ONNX/Ipopt/GLPK runtime dependencies.
No ONNX Runtime is selected by the implementation's evaluation path; adding
one would be a separate stage decision. Ordinary data,valid fraction/chemical
domain,native representation/solver-class support andindependent numerical
acceptance remain open G04 work. Upstream missing-dependency skip guards do
not remove required cases. Ephemeral AST/CSV/JSON/ZIP checks confirmed4 layer
artifacts,100/10000 training rows,15 Keras/two ONNX solve sites andfive
activation parameter rows; no network import/evaluation,training orsolver run
was performed.

## PySMO basics and analytic benchmark closure — 2026-11-05

**Interface-checked, source only:** X `docs/surrogates/pysmo/pysmo_basics*.ipynb`
has four identical scientific ASTs. It first creates25 Hammersley points in
the numeric rectangle[273,373]×[1,50]; this demonstrates sample creation,not
thermodynamic model execution. It then reads the30-row,three-column
`brainin_30.txt` andselects three separate fit/evaluation families:

| Basics stage | Actual selection |
|---|---|
| Polynomial |Same30 rows for original/regression data,degree4,multinomialsTrue,internal split .8,10 cross-validations,extra terms x1²×x2²,exp(x1),exp(x2);core default Pyomo/Ipopt least squares. Generate an expression,predict at(5,8),(−3,10),(−2,3),andplot parity/residuals. |
| RBF |Same30 rows,Gaussian basis,default **algebraic** solution route,regularization defaultTrue. Predict the same three points andreport training R²/RMSE. Source prose claiming a Pyomo default is superseded by the selected constructor's actual algebraic assignment. |
| Kriging |Same30 rows,numerical gradients defaultTrue,regularization defaultTrue (SciPy gradient optimizer). Predict the same three points,report training R²/RMSE andgenerate a Pyomo expression. |

No independent fit-validation dataset or numerical accuracy threshold is
specified by this notebook. Core serialization writes default `solution.pickle`
outputs; each selected overwrite=True fit can replace an earlier fit's file.
These are generated model outputs,not preexisting input artifacts or proof
that old pickles are compatible. Match fitted family,actual coefficients,
prediction/error products andgenerated-equation behavior separately. No
standalone optimization is constructed by the basics notebook.

Six separate X `mod/surrogates/pysmo/` helpers extend this bounded source
population; they are not imported orrun by `pysmo_basics`:

| Helper basename | Actual analytic/data/trainer/consumer closure |
|---|---|
| `polyregression_branin_function.py` |Inline Branin function with source pi3.1417;NumPy seed100;**25** random training and100 distinct test draws over[−5,10]×[0,15]. Degree4,multinomialsTrue,.8 split,5 cross-validations;extra x1²x2,x1x2²,x1²x2²,exp(x1),exp(x2). Report held-out R² andgenerated expression. Comments mentioning20 samples do not change25. |
| `kriging_branin_function.py` |`branin_5625.txt`5625 rows;uniform9×9 nearest-data selection gives up to81 unique training rows,not the descriptive49. Kriging numerical_gradientsFalse,regularizationTrue selects stochastic basin hopping. Evaluate **all** data rows including training matches. Subsequent Ipopt minimization uses x1[−5,0],x2[10,15],not the full data rectangle;initial(2.5,7.5) lies outside both declared bounds. Retain this actual restricted-domain/start behavior for readiness. |
| `kriging_six_hump_function.py` |`six_hump_data_2400.txt`actually2501 rows,61×41 grid over[−3,3]×[−2,2]. Scale features to[0,1],retain unscaled target;Hammersley nearest-data selection requests**50**,not descriptive100,rows. Kriging numerical_gradientsFalse,default regularizationTrue;evaluate all2501 rows (including training matches),reshape errors61×41. No independent unseen-point guarantee orminimization is selected. |
| `polyregression_matyas_function.py` |Legacy binary Excel `matyas_function.xls`;pandas read_excel,30 Latin-hypercube-selected data rows,degree8,multinomialsTrue,default .75 internal split/3 cross-validations/Pyomo fit;generate expression. Compatible legacy-XLS reader andactual sheet/column admission remain explicit prerequisites. |
| `polyregression_six_hump_function.py` |`six_hump_function_data.tab`,10201 data rows plus header,index column removed;30 Latin-hypercube selections,degree8,multinomialsTrue,default split/CV/Pyomo fit,overwriteTrue;generate expression. |
| `polyregression_three_hump_function.py` |`three_humpback_data_v4.csv`,10201 data rows plus header,index removed;30 Latin-hypercube selections,degree8,multinomialsTrue,default split/CV/Pyomo fit;generate expression. |

Every helper imports obsolete `idaes.surrogate.pysmo` paths; the selected
core source owns `idaes.core.surrogate.pysmo` andhas no corresponding old
package tree. Five helpers resolve data relative to their own module directory,
but no `mod/surrogates/pysmo/data_files/` exists in the fixed examples archive.
Their required files are instead shipped under notebook
`docs/surrogates/pysmo/data_files/`. These exact import/path correspondences
are unresolved execution prerequisites,not permission to copy an upstream
implementation orsilently select another dataset. Preserve old entry points
as held required source selections pending explicit compatibility disposition.
The inline Branin helper has no data-file dependency but shares the import
problem. No helper selects external ALAMO orneural trainers.

Latin-hypercube selection is stochastic with no explicit helper seed; basin
hopping also has no explicit helper seed. Core sampling finds nearest existing
rows anddeduplicates,so requested count is not a guaranteed count of unique
selected rows; freeze actual sample identities/cardinality before fitting.
Polynomial internal fold seeding does not by itself freeze initial sample
selection. NumPy/SciPy/pandas/Pyomo/Ipopt andXLS-reader versions,working
directories,generated pickle custody andfailure treatment remain to establish.

Additional shipped cardinal_sine(100 rows),cardinal_sine_2500(2500),
griewank_data(10201) andexponential_function_data(10201 plus header) files
are not consumed by these seven selected entry points. Their remaining
benchmark consumers/source selection stay open,not excluded oradded to a
training population merely because files exist. C's full polynomial/RBF/
kriging/sampling test parameterizations remain a separate unresolved expansion.
No native matched benchmark/training/equation/optimization case has been
established;G04 retains each selected stage andordinary analytic-function/
process-data distinctions. Ephemeral AST/text/file inspection established
four-wrapper equivalence,actual constants,data cardinalities/imports and
module-relative missing paths. No sampler,fit,Excel parser,model import,
pickle load orsolver was executed.

## Moving and bubbling bed configuration closure — 2026-11-05

**Interface-checked, source only:** C
`gas_solid_contactors/unit_models/{moving_bed,bubbling_fluidized_bed}.py`,
`tests/{test_MB,test_BFB_methane_oc,test_BFB_oxygen_oc}.py` andthe three
`flowsheets/ss_{MB_methane_combustion,BFB_methane_combustion,BFB_OC_oxidation}.py`
select the following distinct experiments. Scaled/unscaled initialization
wrappers preserve the same physical feeds where shown; constructor/config,
dynamic-index andfailed-initialization checks do not add solved transients.

| Scientific fixture | Actual geometry,feed andnumerical selection |
|---|---|
| Moving bed `TestIronOC` |Countercurrent gas0→1/solid1→0,steady,D6.5 m/H5 m,10 backward spatial elements,full enthalpy balance,simple pressure-drop law. Gas128.20513 mol/s,298.15 K,200000 Pa,CH4 .975/CO2 .02499/H2O1e-5. Solid591.4 kg/s,1183.15 K,porosity .27,Fe2O3 .45/Fe3O4 1e-9/Al2O3 .55. |
| Moving bed `TestIronOC_EnergyBalanceType` |Same geometry/flow/compositions,energy none,andgas inlet raised to1183.15 K to match the solid feed. Gas/solid temperatures are constrained to the **solid inlet** temperature away from their inlet boundaries. This is an imposed isothermal case,not an energy-conserving hot-feed alternative. |
| Moving bed `TestIronOC_TransformationMethod` |Same hot1183.15 K gas/solid feeds,full enthalpy balance,10 elements×3 Radau collocation points. It is not the cold-feed baseline with only a mesh change. |
| Methane BFB `TestIronOC` |Steady,cocurrent,D6.5 m/H5 m,5 backward elements,full enthalpy. Distributor `number_orifice` fixed2500 **m⁻²**,not a dimensionless total hole count. Gas272.81 mol/s,373 K,186000 Pa,CH4 .4582/CO2 .4772/H2O .0646. Solid1422 kg/s,1186 K,porosity .27,.45 Fe2O3/1e-9 Fe3O4/.55 Al2O3. |
| Methane BFB energy-none |Same flow/composition/geometry,gas inlet changed to1186 K. Bubble/gas-emulsion temperatures equal gas inlet;solid-emulsion temperature equals solid inlet. This model's no-energy closure differs from moving-bed semantics. |
| Methane BFB collocation |Same cold373 K/hot1186 K feeds as its full-energy baseline,5×3 Radau spatial mesh,full enthalpy. |
| Oxygen BFB `TestIronOC` |Same geometry/distributor/5-backward mesh,cocurrent,full enthalpy. Gas1567.79 mol/s,373 K,186000 Pa,O2 .2095/N2 .7808/CO2 .0004/H2O .0093. Solid1230.865 kg/s,1173.9 K,porosity .27,Fe2O3 .244162011502/Fe3O4 .201998299487/Al2O3 .553839689011. |
| Oxygen BFB energy-none |Same feeds including **unequal** gas373 K/solid1173.9 K. Each phase retains its own imposed inlet temperature;no gas-solid sensible heating orreaction heat balance is solved. |
| Oxygen BFB collocation |Same cold/hot feeds,full enthalpy,5×3 Radau mesh. |

The three standalone steady flowsheets repeat their full-energy baseline
inputs/configurations; they add operating/reporting/initialization entry roles,
not a different reaction/bed case. None loads saved state orCSV/trainer data.
Methane fixtures use the already resolved three inline methane/iron reduction
property/reaction owners;trace Fe3O4 makes inlet solid fractions sum to
1.000000001,requiring explicit composition tolerance rather than normalization.

Oxygen fixtures have a **separate** three-module closure under
`properties/oxygen_iron_OC_oxidation/`: ideal O2/N2/CO2/H2O caloric/transport
data (4 molecular masses/formation enthalpies,32 Shomate,16 viscosity,
16 conductivity coefficients,4 diffusion volumes),andthree-solid mass/
density/caloric data (24 Shomate coefficients). Fixed particle diameter
.0015 m,minimum-fluidization voidage .45,bed voidage .35,solid conductivity
12.3 W/(m·K) accompany state-dependent density/porosity. Reaction consumes
O2+4Fe3O4 andproduces6Fe2O3,heat−469443.2 J/mol reaction;grain radius
2.6e-7 m,molar solid density22472 mol/m³,available volume .28,activation
energy14000 J/mol,reaction order1,andpre-exponential3.1e-4 m/s. It cannot
be deduplicated into methane reduction orNETL adsorption. All coefficients
are inline reference inputs;original ordinary kinetic/caloric/transport
provenance andvalidity remain B/D03/F03 obligations.

Moving-bed source configuration permits only countercurrent flow. Simple
pressure loss defaults to .2/s times gas velocity×solid-minus-gas density;
Ergun instead uses relative gas-plus-opposing-solid superficial velocity,
viscosity,particle diameter andbed voidage. These laws are not interchangeable.
Full-energy gas-solid heat transfer requires its particle Reynolds/Prandtl/
Nusselt andconductivity closure. No-energy omits those heat-transfer equations
andimposes the temperature policy above. Material/energy enums expose broader
generic declarations,but actual participating phase properties/laws still
bound admissibility;one accepted enthalpy fixture does not qualify every enum.

Spatial defaults are moving10/BFB5 elements,initial normalized domain[0,1],
finite difference defaulting BACKWARD,orcollocation defaulting Radau3. Both
owners allow explicit FD FORWARD/BACKWARD andreject a non-Radau collocation
scheme in their branch checks. Moving-bed independent gas/solid schemes are
declared only for finite difference andmust be paired,without an explicitly
shared scheme. Actual guard **order** needs readiness verification: automatic
scheme defaulting andpair checks share an `if/elif` chain,so a defaulting
branch can bypass later pair checks. Preserve three explicit test error
selections (collocation+gas scheme,only gas scheme,shared+both phase schemes)
without claiming all such inputs were executed/refused. Different upwind
phase schemes create separate gas/solid spatial domains;their coordinate/
transport/interpolation correspondence is required native work.

`TestBidirectionalSpatialDiscretization` constructs the same dynamic300 s
model twice for build/index checks:three100 s time elements×3 Radau points,
three spatial elements,gas BACKWARD/solid FORWARD,holdupTrue,Ergun pressure
loss. It checks discretization variables/equations andderivative index
dependencies,but fixes no feed,geometry orinitial inventories andruns no
transient solve. It therefore establishes no physical dynamic benchmark.
Native distributed accumulation/initial-state/time correspondence remains
required beyond this constructor footprint.

BFB exposes co/countercurrent flow;only cocurrent is selected in its six
steady fixtures above (the three moving cases are countercurrent). It creates
bubble gas,emulsion gas andsolid-emulsion control volumes with configured
dynamic/holdup flags,but only emulsion gas has the selected pressure balance.
Hydrodynamics closes bubble growth/velocity,bubble/emulsion fractions,
minimum-fluidization behavior,gas/solid superficial velocities andgravity
pressure loss;species diffusion/bulk transfer andreaction source terms,
plus full-energy bubble/emulsion/solid heat exchange,are separate required
laws. No user pressure-drop correlation enum exists here. Countercurrent,
no-pressure-change,dynamic holdup andhomogeneous/equilibrium-reaction
variants remain explicit pending configured-case/admissibility closure.
Fractional powers/smoothed transfer laws andlargely unbounded Reals variables
do not by themselves establish physical domains below fluidization orat
zero/inverted driving forces.

Each steady class has scaled initialization,solution,combined gas-solid
mass/stoichiometric controls andan overfixed outlet-flow1 mol/s
`InitializationError` role. Full-energy cases also check energy including
the selected reaction-heat/datum convention;energy-none cases cannot claim
adiabatic energy conservation. Methane stoichiometric Fe2O3/CH4 ratio12
andoxygen Fe3O4/O2 ratio4 are independent required observations. Retain
all outlet phase/component flows,pressure drop,temperatures,conversion and
original integrated balances;scalar outlet matches are insufficient for
native transport/hydrodynamic acceptance. The separate X diagnostic moving
bed/local property modules remain required diagnostic consumers,not aliases.
Ephemeral AST/source inspection confirmedthree fixture classes perreaction/
bed owner,no hidden parameterization,andtwo dynamic structural constructors;
no model/reference/native solver run was performed. Native gas-solid physics,
ordinary data,full alternative configuration population andindependent
numerical/mesh evidence remain open F03/E03 work.

## Equilibrium column, tray and auxiliary configurations — 2026-11-05

**Interface-checked, source only:** C `models_extra/column_models/{tray,
tray_column,condenser,reboiler,util}.py` andthe eight matching test modules
(`test_conventional_tray`,`test_feed_tray`,`test_tray_side_liq_draw`,
`test_tray_side_vap_draw`,`test_total_condenser`,`test_partial_condenser`,
`test_reboiler`,`test_tray_column`) supply17 explicit operating fixtures.
Fourteen are seven scientific inlet/specification roles with two state
interfaces;three are assembled columns. This is not17 distinct fluids or
a closed count of all categorical column settings.

All isolated auxiliaries/trays select legacy
`properties/activity_coeff_models/BTX_activity_coeff_VLE.py::BTXParameterBlock`
with benzene/toluene,Liq+Vap andIdeal activity. FTPz uses total molar flow/
overall fractions;FcTP uses per-component molar flows. Their inline critical/
vapor-pressure/caloric/formation-reference tables andbase activity-property
equilibrium methods are recursive inputs,not a separately downloaded dataset.
There is no selected xylene stream merely because the package is named BTX.
The third column instead selects `modular_properties/examples/BT_ideal.py`
with FTPx,Ideal phase EOS,SmoothVLE,bubble/dew,fugacity equality,andPerrys
liquid/RPP4 ideal-gas/saturation methods. It is a distinct property definition,
not a label-only coordinate conversion of the legacy package.

| Isolated physical case,each with FTPz/FcTP interfaces | Exact fixed inputs andspecifications |
|---|---|
| Conventional tray |Liquid1 mol/s at369 K,vapor1 mol/s at372 K,both101325 Pa,benzene/toluene .5/.5 each. Heat andpressure-change terms enabled butfixed duty0/deltaP0. FcTP supplies .5 mol/s ofeach component per inlet. |
| Feed tray |Same two tray inlets plus1 mol/s,.5/.5 feed at369 K/101325 Pa;duty0/deltaP0. Third feed flash/mixing is an additional equation/input role,not a conventional tray alias. |
| Liquid side draw |Conventional inputs,liquid draw split .5. Draw andremainder retain the selected liquid-phase composition/enthalpy. |
| Vapor side draw |Conventional inputs,vapor draw split .5;distinct vapor-phase port mapping/enthalpy consumer. |
| Total condenser,bubble point |Inlet1 mol/s,.5/.5,375 K/101325 Pa;condenser pressure101325 Pa,reflux ratio1. Condenser temperature follows the liquid bubble point,heat duty solved. Reflux/distillate are liquid products;no vapor product port. |
| Partial condenser,custom temperature |Same inlet,P andreflux ratio;distillate temperature fixed369 K. Liquid reflux/distillate plus vapor outlet require separate component/energy products. Reflux ratio is not a direct vapor split. |
| Reboiler,boilup-ratio role |Inlet1 mol/s,.5/.5,363 K/101325 Pa;boilup ratio1 (vapor/liquid),heat duty solved. Vapor boilup andliquid bottoms are distinct phase products. |

Three assembled column fixtures each have10 equilibrium trays,feed at5,
total bubble-point condenser,reflux ratio1.4,condenser pressure101325 Pa,
boilup ratio1.3,no tray heat/pressure-change terms,andno side draws:

- `TestBTXIdealFcTP`:20 mol/s benzene+20 mol/s toluene,368 K/101325 Pa.
- `TestBTXIdealFTPz`:40 mol/s,.5/.5,368 K/101325 Pa,same legacy science
  with a different port/state representation.
- `TestBTXIdealGeneric`:100 mol/s,.5/.5,368 K/101325 Pa,modular Ideal
  definition above. Different feed scale andproperty law both matter:
  expected distillate fraction/temperature differ,not only extensive flow.

`build_model_btx_ftpz` is reused by the performance initializer test andFTPz
fixture;it does not add another physical column. Each column source checks
optimal solve anddistillate/bottoms flows,compositions,temperatures andpressure.
The legacy total-flow source targets about18.978/21.021 mol/s distillate/
bottoms,whereas generic targets49.28/50.711;do not rescale one oracle into
the other. Native acceptance also requires complete tray profiles,duties,
phase/component closure andwhole-column energy at the matched property datum.

Tray/column flags explicitly require dynamic=False/has_holdup=False. A tray
independently selects feed,liquid/vapor side draws,heat andpressure terms;
source permits simultaneous options butthe four tray fixtures above exercise
only their stated selections. Side-draw initializer requires each selected
split fixed. Column side-draw flags apply to **all trays**,not a user-supplied
tray subset. Number/feed location are integer configuration fields;the source
requires a non-None tray count butdoes not thereby establish physical range/
edge-tray admissibility. Selected10/5 topology is the concrete fixture;other
counts/feed placements andheat/pressure/side-draw configurations remain
required enumeration/admissibility work,not an inferred supported Cartesian
product. No efficiency/non-equilibrium mass-transfer/hydraulic law is selected
by these equilibrium-stage fixtures.

Condenser configurations expose total/bubble,total/custom andpartial/custom;
missing temperature selection andpartial/bubble are explicit build refusals.
Total/custom occurs only in construction tests here. Its initializer
unconditionally accesses `eq_total_cond_spec` for a total condenser while
that constraint is created only for bubble-point selection:retain the
readiness issue before claiming the custom branch executable. Custom-temperature
initialization otherwise requires zero DOF,including the fixed outlet
temperature. Bubble-point construction takes the first equilibrium pair;
arbitrary multiphase/multiple-pair packages need a separate mapping/refusal.
Reflux fraction is ratio/(1+ratio). Reboiler boilup is vapor/liquid phase
flow andrequires recognized molar-phase orphase-component flow methods;
default `has_boilup_ratio=False` is only a config constructor in these tests.
A heat-duty-specified reboiler remains a distinct required role. The base
condenser/reboiler control-volume flags do not inherit the tray's explicit
no-dynamics guarantee;their actually selected fixtures remain steady and
provide no dynamic inventory evidence.

`make_phase_split` adapts total/phase/component,molar/mass flows andenthalpy
port expressions;its synthetic utility tests include unsupported/missing
flow/fraction/enthalpy method errors. Native phase membership,coordinate
conversion andsplit basis must retain those refusal controls;matching one
legacy spelling is not the local scientific vocabulary authority. The source
conservation tests sum every phase product,including side draws andpartial
condenser vapor;an omitted product cannot be hidden by an outlet-only match.
Initializers use local fixed/free/active staging andIpopt;noCSV,trainer,
external-function EOS orsaved initialization file is loaded by this ideal
BT selection. Exact package/solver compatibility,ordinary coefficients/
envelopes andnative equilibrium-stage/column correspondence remain open
D02/D04/F04 obligations. The existing native separator is a starting surface,
not an assembled column qualification. HDA distillation andMEA/rate-based
columns retain their independent populations. Ephemeral AST inspection
confirmed17 explicit fixtures andno hidden parameter decorators;no property,
column,initializer orsolver was executed.

## MEA rate-based column and solvent auxiliary closure — 2026-11-05

**Interface-checked, source only:** C `models_extra/column_models/{solvent_column,
MEAsolvent_column,solvent_condenser,solvent_reboiler}.py`, their four matching
test modules, `properties/{MEA_solvent,MEA_vapor}.py` and
`enhancement_factor_model_pseudo_second_order_explicit.py` resolve three
packed-column operating cases and four auxiliary specification roles. The
reboiler's old/new initialization APIs double its two scientific roles into
four fixtures; they do not supply four different experiments. These rate-based
columns are independent of the equilibrium BT tray/column population above.

| Case | Geometry, feed and selected transfer/specification inputs |
|---|---|
| Generic absorber,10 spatial elements | Diameter .65 m,length15 m. Vapor22 mol/s,318 K,107650 Pa,CO2/H2O/N2/O2 .12/.09/.74/.06;these fractions sum **1.01**,an unresolved source-input inconsistency,not permission to normalize. Liquid37 mol/s,320 K,same pressure,CO2/H2O/MEA .01/.87/.12. Liquid holdup .01;fixed gas-side CO2/H2O coefficients3e-7/4e-7 mol/(Pa m² s),interfacial area200 m²/m³,heat-transfer coefficient7100 W/(m² K). |
| Generic stripper,10 spatial elements | Diameter .64135 m,length12.1 m. Wet CO2 vapor17.496 mol/s,396.6 K,183430 Pa,CO2/H2O .0145/.9855. Liquid84.48 mol/s,382.15 K,same pressure,CO2/H2O/MEA .0331/.8547/.1122. Holdup .01,gas-side coefficients3e-5/4e-5,same area/heat coefficient as absorber. Liquid CO2 absorption enthalpy is explicitly changed from−84000 to−97000 J/mol;that changes the property science. |
| Correlation-based MEA absorber,40 spatial elements | Diameter18 m,length21.6 m. Vapor19000 mol/s,313.15 K,100000 Pa,CO2/H2O/N2/O2 .041/.074/.763/.122. Liquid24000 mol/s,same temperature/pressure,CO2/H2O/MEA .022/.868/.11. Transfer,interfacial area,holdup andenhancement are calculated from the selected correlations,not the generic absorber's fixed coefficients. Property flow bounds are widened to[0,1e6] mol/s with initial value1. |
| Solvent condenser,reflux-flow role | Wet-CO2 inlet1.1117 mol/s,339.33 K,184360 Pa,CO2/H2O .8817/.1183;reflux liquid flow fixed .1083 mol/s,heat duty solved. The source class is named `TestStripperVaporFlow`,butthe actual specification is reflux flow. |
| Solvent condenser,heat-duty role | Same inlet;heat duty fixed−6264 W,reflux/vapor split solved. The initializer's failure control additionally fixes reflux flow100 mol/s while retaining the heat specification;preserve its over-specified disposition. |
| Solvent reboiler,vapor-flow role | Liquid83.89 mol/s,392.5 K,183700 Pa,CO2/H2O/MEA .0326/.8589/.1085;vapor flow fixed9.56 mol/s,duty solved. Vapor package is four-component flue gas,not the condenser's two-component wet CO2. Source targets duty420983 W andbottoms74.33 mol/s at393.773 K. |
| Solvent reboiler,heat-duty role | Same inlet;duty430610 W,vapor flow solved. Source targets9.7852 mol/s vapor and74.1048 mol/s bottoms at393.810 K. Each specification is parameterized over legacy andnew initializer APIs with the same physical initial guesses. |

Packed-column configuration forces steady operation;holdup may still be
constructed with `has_holdup=True`,which does not grant a dynamic experiment.
The implemented flow topology is countercurrent,vapor0→1 with backward
differences andliquid1→0 with forward differences. The default mesh is20,
not either selected10/40 test mesh. The exposed length-domain list/element
count andpressure-change Boolean do not define a tested Cartesian product;
alternative meshes,nonzero pressure drop andholdup constructors retain
required admissibility/qualification work. The vapor uses total-pressure
momentum balance;the liquid has no momentum balance. Packages must have exactly
Vap andLiq phases respectively,atleast one common component andmatching material
flow bases. Geometry/packing defaults are void fraction .97,specific area250
m²/m³ andchannel size .1 m;hydraulic diameter follows the first two inputs.
Neither the channel-size default norinterfacial area is the physical column
diameter. Conservation must include phase/component transfer andinterphase
enthalpy,not only outlet CO2 removal.

The aqueous property input is apparent H2O/MEA/CO2 with true ionic species
MEA_+,MEACOO_- andHCO3_-,Ideal liquid EOS on the apparent property basis,
FTPx state,andcarbamate/bicarbonate inherent equilibria on true molarity.
CO2 uses the N2O-analogy Kpc Henry relation on the **true** basis;the absorber's
default absorption enthalpy is−84000 J/mol. Vapor is Ideal FTPx,apparent
components,with four-component flue gas or two-component wet CO2 as selected.
Inline molecular weights,caloric/formation references,vapor pressure,Henry,
liquid volume/density,viscosity,surface tension,diffusivity andvapor
conductivity/Cp methods andcoefficients form recursive scientific inputs.
Their tracked presence permits oracle reproduction only;ordinary native
admission still needs original provenance,units,reference datum andenvelope.
An Ideal EOS label does not remove reactive speciation or transport laws.

`MEAColumn` adds Billet–Schultes transfer/area/holdup andheat-transfer
correlations,log property relations andflooding calculations. Its construction
fixes area parameters1.43914/.12,holdup11.4474/.6471 andtransfer constants
Cv=.357/Cl=.5. The selected enhancement callback is
`PseudoSecondOrderExplicit` with default Putta kinetics;Luo is a separate
exposed alternative,not exercised by this absorber fixture. The MEA/H2O
activation-temperature/pre-exponential pairs are4936.6 K/3173.2 and3900 K/108.82
for Putta versus4742 K/20030 and3110 K/4.147 for Luo,with pre-exponentials
in m⁶/(mol² s). True-species concentrations/diffusion,liquid CO2 transfer and
Hatta/enhancement relations must follow that choice. An unrecognized kinetics
branch returns a ValueError object rather than explicitly raising it;retain
the build/refusal readiness question instead of claiming a checked rejection.

The initializer consumes seven inline31-point profile lists:two gas transfer,
one liquid transfer,heat coefficient,interfacial area andtwo mode-specific
enhancement lists. These are reference **initial guesses**,not experimental
targets or ordinary material data. Absorber/stripper select different
enhancement profiles;unknown mode raises. Mesh changes interpolate the lists;
the requested grid is `i/(nfe+1)` for i=0..nfe,so its last point is not1.
Preserve that actual initialization behavior separately from the physical
mesh. No CSV,serialized initialization,trainer or external EOS file is loaded
by these selected owners. Source-only AST inspection confirmed the list sizes
andabsence of direct file-loading calls;it did not initialize a property or
run a solver.

Auxiliaries explicitly refuse dynamics/holdup andrequire single matching
Vap/Liq phase packages andatleast one common component. Condenser supports
only molar flow basis;reboiler has molar/mass branches,butits noncondensable
zero-flow parameter is declared in molar units even in the mass branch.
Selected molar cases set nonshared components to1e-8 mol/s,not mathematically
zero:the condenser's liquid-only MEA andreboiler's vapor-only N2/O2 have this
role. Preserve epsilon-component accounting andbasis refusals. Native
rate-based transport/reactive liquid data,auxiliary phase mapping,complete
profiles/duties/species/energy closure andboth specification roles remain
D02/D04/F04 obligations. Source Ipopt/default-solver skip decorators do not
carry into the fail-never-skip parity policy;exact pinned-environment and
independent numerical execution remain open.

## Legacy activity-property configuration closure — 2026-11-05

**Interface-checked, source only:** C
`models/properties/activity_coeff_models/{activity_coeff_prop_pack,
BTX_activity_coeff_VLE}.py` andthe four `tests/test_ideal_{ideal_FTPz,
ideal_FcTP,NRTL,Wilson}.py` resolve the legacy BT activity-property selection.
The parameter domain is exactly three liquid models Ideal/NRTL/Wilson,two
state interfaces FTPz/FcTP andfour phase presentations Liq,Vap,(Vap,Liq),
(Liq,Vap):24 syntactic parameter selections. The two phase tuple orders have
the same selected physics;they do not add separate fluids. Defaults are
Ideal,FTPz,(Vap,Liq). Parameter constructor acceptance alone does not establish
that every lazy property,initializer andconsumer works for every selection.

The actual source population has twelve parameter selections:Ideal with both
state interfaces andthree physical phase choices,plus NRTL/Wilson with FTPz
andthe same three phase choices. Each has defined-state andreceiving-state
fixtures,24 roles in total. NRTL/Wilson FcTP,reverse phase ordering andother
operating points remain required explicit dispositions. Ideal FTPz repeats
these roles in both class fixtures andnew-initializer functions;Ideal FcTP
has a separate six-role initializer class. Those API paths are evidence
routes,not extra scientific experiments.

All selected state points are368 K/101325 Pa,benzene/toluene .5/.5 and1 mol/s.
FcTP fixes .5 mol/s ofeach component. Defined FTPz states fix both fractions;
receiving FTPz states fix benzene .5 andsolve toluene from normalization.
Single-phase selections explicitly disable phase equilibrium;requesting it
with only Liq orVap raises a configuration error. A two-phase package still
constructs flash equations when its state flag says no phase equilibrium;
that flag alone is not a no-flash scientific selection. Single-phase
NRTL/Wilson tests verify absence ofactivity-coefficient equations anddo not
exercise nonideal liquid fugacity or equilibrium calculations there.

Ideal cases actually initialize/solve andcompare phase fractions orcomponent
phase flows. The two-phase source targets liquid/vapor benzene fractions
.4121/.6339 (absolute tolerance1e-3);FcTP targets benzene phase flows
.2488/.2512 mol/s. These are different coordinate products ofthe selected
point,not separately fitted data. The flash uses smooth bubble/dew clamping
with eps1=.01 K/eps2=.0005 K,fugacity equality andan equilibrium temperature;
physical state temperature andequilibrium temperature must not be conflated.
No global stability/supercritical certificate follows from the smooth solve.

The NRTL/Wilson modules contain construction,input/DOF andunit-consistency
checks,not numerical initializer/solve orfitted-property assertions. NRTL
two-phase fixtures fix alpha at its initial .3 andtau at1;Wilson fixes molar
volumes at1 m³/mol andtau at1. These are synthetic defaults,not accepted BT
interaction/density data. Receiving two-phase states retain six NRTL orfour
Wilson DOF because those interaction parameters remain free. NRTL tau here is
a dimensionless direct interaction variable,with no selected temperature-law
coefficient fit. Fitting/uncertainty notebooks with measured VLE andshared
parameters are a separate G02/G03 population;their dataset andestimation
conditions cannot be inferred from these constructors.

Only benzene andtoluene are actual component objects/equilibrium members.
The additional o-xylene master-table entries do not select a ternary fluid.
Inline critical data,molecular weights,saturation coefficients,phase caloric
polynomials andformation enthalpy/entropy use the package's298.15 K/101325 Pa
reference. Vapor density is ideal-gas;liquid molar density is explicitly a
placeholder11100 mol/m³ with a source warning. Native inventory/pressure-drop
orheat-exchanger density science cannot be accepted from that placeholder;
ordinary density admission andmatched alternative science remain explicit
owner decisions. The lazy vapor internal-energy method also references
`temperature_ref` while this BT parameter owner declares
`temperature_reference`;retain this property-readiness gap separately from
tested phase-split behavior. Property metadata lists pressure bubble/dew
methods butthis owner defines only temperature bubble/dew builders;metadata
is not executable coverage.

There is no selected data file,trainer or external-function EOS input in
these four core tests. Ordinary coefficients andvalidity envelopes still
need original provenance. Reference Ipopt/ipopt_v2 compatibility,full caloric/
derivative/density products,single-phase nonideal properties,remaining
parameter selections andnative NRTL/Wilson correspondences remain open.
AST/source reads established selections andtest roles only;no property model
or solver was executed andno upstream skip policy is adopted.

## NRTL fitting and local uncertainty input closure — 2026-11-05

**Interface-checked, source only:** X `docs/param_est/
parameter_estimation_nrtl_using_{state_block,unit_model}.ipynb` andtheir
solution/doc/test renderings select two model-consumer routes for one fitted
BT interaction definition. C `apps/uncertainty_propagation/examples/
{NRTL_model_scripts,uncertainty_propagation_NRTL,rooney_biegler,
uncertainty_propagation_rooney,simple_opt_problem}.py`, `uncertainties.py`,
`sens.py` and`tests/test_uncertainties.py` add parameter covariance andlocal
optimized-output sensitivity products. These products remain separate from
an ordinary forward state solve or a Bayesian/robust inference claim.

X `docs/param_est/BT_NRTL_dataset.csv`,C example CSV andC test CSV have exactly
equal parsed records,despite different file bytes. Each contains50 rows with
columns temperature,liq_benzene,vap_benzene:temperature365.5–371.1470588 K,
liquid benzene .28914901–.480953385,vapor benzene .484824184–.692441237.
Thus this is one input population used by three consumers,not150 independent
experiments. Both phase compositions in a row are observations at the same
temperature. The notebook labels them experimental data,butthe read sources
do not establish original measurement/noise provenance. Retain them as
oracle/fit inputs until B separately admits ordinary scientific provenance;
no noise sampler orgenerated uncertainty is implied by their scatter.

Both notebook routes use legacy BT NRTL,FTPz,two-phase,flow1 mol/s,
pressure101325 Pa andoverall benzene/toluene .5/.5. Diagonal alpha/tau are0;
off-diagonal alpha isfixed .3. Two dimensionless off-diagonal tau values are
shared across all50 experiments,initialized−.9/1.4 andbounded[−5,5]. They are
constant parameters over the dataset's temperature range,not a fitted
temperature-dependent law. Initialization first uses368 K with tau fixed;
the experiment then fixes its actual temperature andthe estimation interface
selects the two unknowns. The state-block route uses phase compositions
directly;the flash route adds fixed zero duty/deltaP andconsumes flash outlet
phase ports. It must retain energy andport closure even when fitted tau and
composition targets correspond.

Each row becomes one Experiment. Its two benzene outputs are labeled;
measurement_error isNone andthe selected Estimator objective isSSE.
The state route wraps outputs in singleton indexed Expressions;the flash
route labels outlet variables directly. Unknown-parameter identity is
tau[benzene,toluene] then tau[toluene,benzene]. The source targets objective
5.07496e-4,tau−.89876/1.41048. State objective comparison has relative1e-3;
flash objective comparison has absolute .1,which is much larger than the
target. Do not adopt that loose guard as native scientific acceptance.
Four-bootstrap resampling with seed542 is commented out;it is an exposed
extension request,not an actually executed bootstrap population orconfidence
region. No holdout dataset ornoise variance is supplied by these notebooks.

The base/solution/usr notebooks contain an incomplete exercise function
followed by the complete function that replaces it. In the unit-model
exercise the temperature setter incorrectly points to `state_block`;the
complete function selects `flash.inlet` correctly. Doc/test renderings retain
the complete function;exercise rendering retains the incomplete one. Select
the complete scientific route for execution rather than counting these
instructional variants as distinct science orpromoting the exercise to an
independent reference. Full normalized wrapper equivalence outside these
inspected functions remains separate from this scoped correspondence.

C NRTL covariance uses the same50 rows andphysical flash,butinitializes
off-diagonal tau .1690/−.1559 before estimation. Its residual sum for both
benzene phase observations ismultiplied by10000,andits objective target is
5.0749685783 rather than the notebook's5.07496e-4. The source covariance target
is[[.01194738,−.02557055],[−.02557055,.05490639]] in that tau order. Record the
objective normalization andcovariance sample count explicitly:the owner
defaults covariance_n to50 rows,not100 scalar observations. It calls
parmest's legacy callback-based Estimator and`theta_est(calc_cov=True)`;
compatibility with pinned Pyomo6.10.1 remains execution-readiness work,
independent ofthe notebooks' newer Experiment interface.

The uncertain optimization model fixes the same368 K,adiabatic flash point,
leaves tau free for sensitivity machinery,andminimizes
exp(−.3*tau[toluene,benzene]);the first tau has zero direct objective
coefficient. Propagation evaluates at fitted tau bysetting equal lower/upper
bounds andforms local total derivatives through the solved equations. The
source targets objective-gradient component−.19649493 andpropagated objective
**variance** .00211994998,not a standard deviation or Monte Carlo interval.
Constraint covariance isanother output role. Parameter order,derivative sign,
state/parameter columns,reference point andactive constraints must remain
matched. The source uses Ipopt NL/ASL,k_aug anddot_sens andtheir generated
Jacobian/row/column files. Those are execution intermediates,not frozen
scientific inputs;exact executable versions andisolated working-directory
ownership remain prerequisites. No missing-executable skip is admissible in
the local parity route.

`NRTL_model_opt_infeasible` isnot an established infeasible case:its test is
xfail with an explicit source note that the model isfeasible. It retains the
same physics/objective andomits an initialization call. Preserve this as a
failed historical failure control;G01/G03 still need a genuine,declared
infeasible control with appropriate diagnosis instead of inheriting its name.

The companion Rooney–Biegler fit is a separate analytic benchmark:response
A*(1−exp(−k*h));six(h,y) records(1,8.3),(2,10.3),(3,19),(4,16),(5,15.6),(7,19.8),
initial A15/k.5,andtwo shared unknowns withoutselected bounds. The downstream
output isresponse at h10. Source targets fit objective4.3317112137,A19.1425753,
k.531091377,covariance[[6.30579403,−.4395341],[−.4395341,.04193591]] andoutput
variance5.4543933775. The standalone quadratic sensitivity example uses
p1=10/p2=5,links x1+x2=p1 andx2+x3=p2,objective p1*x1+p2*x2²+p1*p2,
three unbounded primal variables andparameter covariance diag(2,1) in p1/p2
order;its commented0..10 bounds are **not** actual constraints.
It is an analytic derivative/propagation control,not a process experiment.
Remaining API/refusal tests,covariance definiteness/identifiability,ordinary
data admission,native case/output mapping andindependent execution remain
open. CSV/JSON/AST reads here ran no estimator,property initializer orsolver.

## Modular PR mixture and phase-membership closure — 2026-11-05

**Interface-checked, source only:** C `models/properties/modular_properties/
examples/{BT_PR,ASU_PR,HC_PR,HC_PR_vap,CO2_bmimPF6_PR}.py` provides six
configuration dictionaries;`examples/tests/test_BT_PR_legacy_SmoothVLE.py`
adds a seventh complete BT definition. Matching seven test modules and
`eos/{ceos,ceos_common}.py` resolve their selected inputs,phase membership and
external-root dependency. This is seven authored definitions,not a closed
count ofevery cubic EOS/mixture/equilibrium combination.

| Definition | Actual state/property/phase selection andinteraction input |
|---|---|
| BT_PR | Benzene/toluene,FTPx,Liq+Vap,PR inboth phases,CubicComplementarityVLE,LogBubbleDew andlog fugacity equality. Four kappa entries are0. RPP4 ideal-gas caloric/entropy/saturation methods;298.15 K/101325 Pa reference. Flow bounds0..1000 mol/s(initial100),T273.15..500 K(initial300),P50000..1e6 Pa(initial1e5). |
| Legacy SmoothVLE BT | Complete BT dictionary in its test module selects SmoothVLE rather than cubic complementarity. Same selected component/PR/caloric family isnot proof that smoothing,phase-transition orderivative behavior isidentical. Six fixed operating probes below correspond;retain separate formulation acceptance. |
| ASU,standard | Nitrogen/argon/oxygen,FTPx,Liq+Vap,PR,SmoothVLE/LogBubbleDew/log fugacity. RPP4 ideal-gas caloric/entropy,NIST saturation;critical pressures3.4e6/4.898e6/5.043e6 Pa andomega .037/.001/.025 inthat component order. Symmetric off-diagonal kappa N2–Ar−.0026,N2–O2−.0119,Ar–O2 .0104;diagonals0. |
| ASU,Dowling2015 | Same topology/interaction matrix butRPP3 saturation,critical pressures3394387.5/4873732.5/5045985 Pa andomega .04/−.004/.021. This is a different scientific parameter/method definition,not a second name for standard ASU. Both useT10..350 K/P50000..1e7 Pa bounds,initial300 K/1e5 Pa,andreference298.15 K/101325 Pa. |
| HC_PR | Thirteen components:hydrogen,methane,ethane,propane,nbutane,ibutane,ethylene,propene,butene,pentene,hexene,heptene,octene. Hydrogen/methane areVap-only;the other eleven haveLiq+Vap,so phase-component set has24 members,not26. FTPx,PR,SmoothVLE/LogBubbleDew/log fugacity;all169 ordered kappa entries0. RPP4 ideal-gas caloric/entropy,with inline Perrys liquid/RPP5 saturation methods for applicable components. T273.15..1500 K/P50000..1e7 Pa bounds. |
| HC_PR_vap | Same13-component vapor-only property consumer;no selected liquid phase/flash equilibrium. All169 kappa entries0,FTPx/PR,same state bounds/reference. Its retained component coefficient/method tables do not select liquid outputs. Need vapor caloric/fugacity/density correspondence independently ofthe two-phase root. |
| CO2–bmimPF6 | FTPx,PR,Liq+Vap,SmoothVLE/LogBubbleDew/log fugacity. bmimPF6 isLiq-only;CO2 hasboth phases,so phase set has3 members. Ordered kappa[bmimPF6,CO2]=−.4071 andkappa[CO2,bmimPF6]=.0206,diagonals0;do not symmetrize them. RPP4 ideal-gas caloric/entropy,NIST CO2 saturation. BoundsT10..500 K andP.0005..1e10 Pa areconstructor limits,not a demonstrated validity envelope. Reference298.15 K/101325 Pa. |

The six fixed BT probes all use100 mol/s. Five havebenzene/toluene .5/.5:
(T K,P Pa)=(350,1e5),(350,5e5),(450,1e5),(450,5e5),(368,1e5).
The sixth uses376 K/1e5 Pa and.2/.8. Both current complementarity scaling
paths repeat these inputs;scaler choice isnot a new fluid. Current eps_t/eps_z
are1e-4 foraccuracy probes. Additional temperature **optimization** sweeps
select8 pressures logspace(4.8,5.9),freeT within an explicitly widened600 K
upper bound,andobjective(T−510)²;current formulation first solves at eps10
thenrefines1e-4 andasserts liquid flow≤1e-5. Legacy SmoothVLE instead scales
the objective by1/10000 anduses≤1e-4 liquid-flow guard. This trajectory
crosses the original500 K declared bound deliberately;it doesnot extend the
ordinary property validity envelope.

Pressure sweeps fix T at370,395,420,445,470,495 K andstep pressure1e5..1e6 Pa
by1e5,with an extra initial solve at1e5. Current complementarity uses eps1e-2
for this continuation. These are60 prescribed(T,P) probes per formulation/
scaling route plus repeated initial solves,not60 independent parameter
definitions. Solver success isnot the only required physical check:preserve
root/phase membership,component/energy closure,caloric datum,density,fugacity
andderivative products. Exact successful/failure trajectories remain
unexecuted;do not derive a numerical robustness rate from the loop bounds.

Standard ASU's selected state is1 mol/s,85 K/101325 Pa,equal thirds N2/Ar/O2.
Dowling's three states are1 mol/s at77.5295 K/101000 Pa with
.9823/.00803/.00967,300 K/353140 Pa with.78/.01/.21 (SF0),and92.88 K/353140 Pa
with.6653/.014/.3207 (SFIL3). Source compares two-phase nitrogen fractions/
vapor fraction at the first andsingle-phase vapor/liquid limits andenthalpy
at the latter two. Standard andDowling state probes aredistinct;these are
not a direct same-input validation oftheir coefficient changes.

Both HC definitions select1 mol/s,295 K/1e5 Pa,first twelve components each
.077 andoctene .076 (sum1). Two-phase source targets vapor fraction .77026,
vapor hydrogen .09996 andliquid/vapor propene .01056/.09681,absolute1e-4.
The performance helper reuses this same model/input. Vapor-only construction/
initialization andcaloric products do not reproduce those two-phase split
targets. Inline elemental composition andall selected method coefficients
are recursive data inputs;RPP/Perrys table presence isnot ordinary admission
or a claim that every configured method isconsumed by the cubic EOS.

CO2–IL state/scaling fixtures use1 mol/s,298.15 K,1214713.75 Pa,CO2/IL .2/.8.
The separate flash uses1 mol/s,200 K/101325 Pa,.5/.5,noheat/no pressure-change
terms:it isadiabatic,not a flash with outlet temperature fixed200 K. Source
targets liquid CO2 .3119,vapor CO2≈1,vapor/liquid molar-flow ratio .37619
(absolute1e-4). Requiring IL vapor would violate the selected phase
membership. X PR parameter-fitting CSV/flash is a separate required input/
shared-parameter/estimation consumer,not covered by this fixed fit.

The shared Cubic domain supports PR andSRK with only the default a/b mixing
rules currently declared;all seven definitions above selectPR. Same-EOS
options must agree across phases;unrecognized EOS/mixing options,non-liquid/
vapor phases andHenry components in a cubic phase areexplicit refusals.
Soave alpha andits first/second temperature derivatives areselected by the
owner,unless a consuming model deliberately replaces them (the SOC source
above does). Full SRK,alternative pure-method/coordinate/equilibrium and
mixed-EOS selections remain required separate census/acceptance work.

`ceos_common` locates andloads `cubic_roots`;the reference bindings select
`cubic_root_l`/`cubic_root_h` forliquid/vapor root values andderivatives.
Four synthetic general-polynomial controls in `eos/tests/test_cubic_root.py`
check low/high root,residual,gradient andpacked Hessian for(b,c,d)=
(−3,.5,−1),(−3,.5,1),(1,.5,−1),(1,.5,2);onlythe second ismarked three-root.
This numerical kernel evidence isseparate from a phase-stability certificate.
Exact shared-library/build provenance andIpopt/ipopt_v2 remain prerequisites;
availability skips upstream do not relax the local fail-never-skip policy.
Inherited `properties/tests/test_harness.py` adds property hooks,initialization,
fixed/free restoration,failure andcontrol-volume construction atdefault
states;BT/ASU explicitly set `has_density_terms=False`,so those inherited
integration checks do not qualify transient density inventories. No dataset,
trainer orsaved initializer isloaded by these fixed core mixture cases.
All source tables remain oracle inputs until ordinary provenance/envelopes
areadmitted,andnative ragged phases/methods/output mapping andindependent
scientific execution remain open. AST/source reads executed no EOS orsolver.

## Ionic-liquid PR fitting and T–x–y consumer closure — 2026-11-05

**Interface-checked, source only:** X `docs/properties/
parameter_estimation_pr{,_doc,_test,_usr}.ipynb`, `CO2_IL_298.csv` and
`dictionary_txy_diagrams{,_doc,_test,_usr}.ipynb` resolve two distinct
scientific consumers. The first fits ordered PR interactions forCO2–bmimPF6;
the second generates an Ideal BT phase diagram. They share modular property
infrastructure,not fluid physics or scientific data. C `core/util/
phase_equilibria.py` is the diagram's recursive computation/plotting helper.

The PR CSV has **18** records,six each at283.15,298.15 and323.15 K,not the
nineteen claimed in notebook prose or a single298 K dataset implied by its
filename. Columns aretemperature,pressure,x_carbon_dioxide,x_bmimPF6.
Pressure spans202299.43..1214713.75 Pa,liquid CO2 .020368132..246765331;
each liquid pair sums exactly1 inparsed values. These are temperature,
liquid-composition andpressure observations,not an inlet/outlet stream table.
Original measurement/noise/source provenance isnot established by the CSV;
retain it asreference fitting input until ordinary admission isresolved.

Despite flash wording/imports,the complete `PR_model` builds a **state block**
with core `CO2_bmimPF6_PR.configuration`,FTPx,1 mol/s,Liq-only IL andCO2
inboth phases. It initially fixes datasetT/P andoverall CO2=x_liquid_CO2+.5,
IL=1−that value. Diagonal kappa are0,ordered IL→CO2/CO2→IL initialize−.047/.002
(different from the core fixed-fit−.4071/.0206). It initializes,then fixes
the dataset'stwo liquid compositions andoverall IL toits observed liquid
fraction,unfixes pressure andoverall CO2,andbounds the two ordered unknowns
at[−5,5]. Preserve this actual fixed/free construction;do not substitute a
feed-at-measured-composition flash orsymmetrize kappa. Unknown order is
kappa[IL,CO2] thenkappa[CO2,IL].

One Experiment per row labels **pressure alone** asoutput,with
measurement_error=None;liquid fractions andtemperature aretreated asexact
inputs. The shared two interactions areconstant across allthree temperatures.
Estimator selectsSSE and`theta_est()`;there isno covariance,bootstrap,
holdout or fitted-parameter numerical assertion inthese four wrappers.
The printed objective multiplies rawSSE by1e-7 only atdisplay time;that isnot
an objective weight used by the estimator. A separate test dict at298 K,
812323 Pa,CO2/IL .14/.86 builds/solves a0-DOF model,butthe pressure isfreed
by the same helper. Its observed pressure is an initial value,not a hard
constraint on the resulting bubble-pressure calculation. Source asserts
zero DOF only;it doesnot assert optimal termination orpressure residual.
Reference Ipopt,cubic-root library andPyomo Experiment-interface compatibility,
parameter identifiability/residual units,native constrained-equilibrium fit
mapping andindependent numerical qualification remain open.

The T–x–y notebook authors its own two-component BT dictionary:FTPx,Ideal
liquid/vapor,SmoothVLE,IdealBubbleDew andfugacity equality;Perrys liquid
density/enthalpy andRPP4 ideal-gas enthalpy/saturation. Reference is300 K/
100000 Pa,unlike core BT's298.15 K/101325 Pa. Temperature bounds150..450 K,
initial300 K;pressure bounds50000..1e6 Pa. Its liquid Cp coefficients are
declared J/(kmol Kⁿ),e.g.benzene129 andtoluene140 for theconstant term,
rather than J/(mol Kⁿ). Preserve theactual factor1000 caloric distinction
before admitting ordinary data orclaiming whole-property equivalence;the
diagram's bubble/dew result doesnot qualify its caloric values.

The actual diagram call requests25 binary compositions at101325 Pa,flow1
mol/s,initial/state temperature150.15 K,solver tolerance1e-6,nofigure-name,
legend on andpressure legend off. C `Txy_data` builds adefined state,
initializes atbenzene .99/toluene .01,thenuses25 linearly spaced benzene
fractions **descending .99→.01**,with toluene1−benzene. Temperature remains
fixed150.15 K whilebubble/dew auxiliary temperatures arecalculated;it isnot
a vapor/liquid coexistence stream at150.15 K. The requested grid excludes
pure endpoints. Generic extra components,whenpresent,arefixed1e-5 each and
subtract from the second component;that is a separate trace-mixture behavior,
not a true binary diagram or a selected fixture here.

`Txy_diagram` passes its default solver=None to`Txy_data`,overriding the
latter's direct-call default ipopt_v2;freeze theactual selected solver/version
for reproduction. The helper tests optimality perpoint,but**omits failed
points** from returned arrays andplots. Local qualification must retain all25
requested inputs,their outcomes andfailed/missing-point dispositions;an
attractive shortened curve isnot completed denominator evidence. The four
notebook wrappers have no numerical diagram assertions. Bubble/dew branch
availability,property/phase validity,complete curve correspondence andnative
point/plot product mapping remain required. The CSV,inline property table,
initial guesses andgenerated plot/fit outputs retain separate roles;noother
datafile,trainer orsaved initialization isloaded by these consumers. Reads
andstdlib CSV/JSON inspection executed no estimator,EOS,diagram orsolver.

## Electrolyte reference-state and verification-input closure — 2026-11-05

**Interface-checked, source only:** C `modular_properties/examples/
enrtl_{H2O_NaCl_KCl,NaBr_mixed_solvent}.py`,the NaCl/KCl core example test,
`eos/{enrtl,enrtl_parameters,enrtl_reference_states}.py` andV
`verification/properties/eNRTL/test_enrtl_verification_{NaCl,KCl,NaCl_KCl,
2_solvent}.py` resolve these selected activity populations. All useFTPx,
true-species state coordinates,one aqueous Liq phase andexplicit apparent
salt dissociation;this doesnot qualify alternate apparent-coordinate,
multiphase,caloric orprocess-unit consumers.

C NaCl/KCl definesapparent NaCl/KCl andtrueH2O/Na+/K+/Cl−,unit ion charges,
water molar density1000/.018 mol/m³,MW .018 kg/mol andpermittivity78.54.
Symmetric reference isexplicit,referenceT300 K/P1e5 Pa;its constant ordered
water→NaCl/NaCl→water tau are9.0234/−4.5916 andwater→KCl/KCl→water
8.1354/−4.1341. These ion-pair interactions arenot molecule-only NRTL tau.
Core'sactual50-point NaCl limit test changes Na tau to8.885/−4.549,sets
T298.15 K andleaves K+ at1e-12;do not compare it against unmodified default
NaCl/KCl data orcall KCl exactlyabsent.

C mixed-solvent NaBr definesH2O/MeOH/EtOH solvents,apparentNaBr→Na++Br−,
same unit charges,defaultSymmetric reference andtrue coordinates. Solvent
densities are1000/.018,792/.032,789.45/.046 mol/m³,respectively;permittivities
78.54/32.6146/24.113. Function comments swap the alcohol names,butthe actual
keyed methods use these values. Three symmetric solvent–solvent alpha values
are.3031(H2O/EtOH),.2994(H2O/MeOH),.3356(MeOH/EtOH);solvent–salt alpha
.2/.2/.1. Ordered solvent tau pairs areH2O↔MeOH1.4265/−.42864,
H2O↔EtOH2.2485/−.18514,MeOH↔EtOH−.04394/.02147;water,methanol,ethanol
→NaBr/reverse are9.527/−4.790,5.910/−3.863,6.118/−4.450. No solvent
temperature dependence isselected by these constant methods.

| Exact V class/family | Input/reference/target population |
|---|---|
| SingleNaCl,Symmetric andUnsymmetric | T298.15 K,watertrue fractionx andeachion(1−x)/2. The same four digitized tables have47 water-ln-gamma points(x .0166772823..999998),48 long-range ionic points(.0102202097..9945432987),51 local-composition ionic points(.0064291094..9926673601),50 total ionic points(.0162094136..9855967963). Tau8.885/−4.549;water/NaCl constant molar-volume method18e-6 m³/mol. Pure-water limits retain1e-12 ions;Symmetric also testsnear-pure salt withwater1e-12 andions .5/.5. |
| SingleKCl,Unsymmetric | T298 K,tau8.064/−4.107.28 molality/gamma pairs,m=.001..4.9908 mol/kg;water w=1000/18 mol on1 kg basis. True fractions waterw/(w+2m),eachionm/(w+2m). Compare natural-log ionic gamma minusln(1+.036m) againstln(observed molal gamma),rel.03/abs.006. |
| MixedNaCl/KCl,zeroKCl limits | Symmetric andUnsymmetric at298.15 K,Na tauoverridden8.885/−4.549. Reuse the four singleNaCl dictionaries exactly;K+ remains1e-12. These are species-embedding/reference-state checks,not new independent digitized data. |
| MixedNaCl/KCl,zeroNaCl limit | Unsymmetric at298 K,Ktauoverridden8.064/−4.107. Reuses singleKCl's28 molality pairs;Na+ remains1e-12. |
| Mixedsalts,tau0 | Unsymmetric298.15 K,totalmolality4,waterw=1000/18. K=x*4,Na=(1−x)*4,Cl=4;normalize by w+8. Default water–salt tau retained;both ordered salt–salt tau0. Separate NaCl/KCl target curves41/44 points. |
| Mixedsalts,tau1 | SameT/molality/composition map;ordered KCl→NaCl tau−1,NaCl→KCl+1. Separate NaCl/KCl curves36/38 points. |
| Mixedsalts,tau2 | Sameinputs;ordered KCl→NaCl+1,NaCl→KCl−1. Separate NaCl/KCl curves50/46 points. These sign-reversed interactions aredistinct scientific definitions. |
| NaBr,binary-solvent saturation curves | DefaultSymmetric,298.15 K;H2O/EtOH44,H2O/MeOH50,MeOH/EtOH31 digitized(x,g) pairs from the citedfigure. x isfirst-solvent mass percent ofsalt-free solvent,g issalt mass on100 total-mass basis. Convert saltg/.1029,solvent masses(100−g)*x/100 and(100−g)*(1−x/100) bytheir MW,andnormalize withtwice saltmoles. Compareln(activityNa*activityBr) to−7.157,rel .02/.025/.023 respectively. No solid phase orsolubility solver isselected. |

The NaCl four tables areparsed-equal across symmetric/unsymmetric andmixed
zero-K embedding roles;the core50-point total-gamma table matches themtoo.
Count data origins separately from repeated method/reference-state checks.
NaBr solventx rangesare1.90023165..97.80665581,6.82452237..98.90094264 and
2.71536367..97.33767574 percent intheroworder above. The unused thirdsolvent
is1e-12,so each isatrace-containing limit rather than exactlybinary.
The digitized targets andupstream fitted interactions areoracle verification
inputs;ordinary material parameters require original-source admission and
digitization allowances. Target dictionaries arenot ordinary EOS coefficient
tables simply because they areinlinePython.

For symmetric NaCl comparisons,the long-range target adds.971,the local
target subtracts2.414 andtotal subtracts1.462. Unsymmetric ionic comparisons
use the unshifted targets. These offsets align referenceconventions;do not
treat themasfree regression knobs orremove themfrom comparison allowances.
Water gamma andapparent-salt gamma areadditional products,not substitutes
forindividual true-ion activities. Mixed-salt curves comparelog10 meanionic
gamma onmolal basis:half thesum ofionicln-gamma,divided byln10,minus
log10((w+8)/w). Watertrue fraction .874125874,Cltruefraction/ionicstrength
.06293706294 areseparate normalization controls.

ENRTL defaults toSymmetric,ConstantAlpha/ConstantTau andclosest-approach14.9.
Alpha is symmetric,default .3 formolecule/molecule and.2 otherwise;conflicting
reversealpha andinvalid pair keysrefuse. Tau isordered,default0 withinvalid
pairsrefused. Customalpha/tau/reference callbacks areopen methodcontracts,
not afinite arbitrary-callback catalog. Symmetric normalizes referenceions
toafused-salt composition,withreference solvent0;Unsymmetric normalizes
reference molecularspecies andusesionEPS1e-20. Those referenceEPS andstate
trace1e-12 have differentroles. Exactzero ion/solvent limits canmake logarithms
orcomposition denominatorssingular;neither nonnegative statebounds nor
the shownnear-limit controls establish arbitraryzero-state admissibility.

V fixtures often use `dict(configuration)` thenmutate nestedphase options
toUnsymmetric;this shallowcopy also changes theshared configuration.
Reference execution mustfreeze percase options andtestorder rather than
inferreproducibility fromaclassname. The sources assignproperty composition
values andevaluateexpressions withoutinitializing/solving acharge-balanced
flowsheet. No measured processstream,CSV,trainer,savedstate ornative solver
isloaded bythese selected verificationowners. Native D03 stillowes admitted
electrolyte inputs,true/apparent andchargeconservation,referenceconversion,
complete property/derivative validity,remainingmethod/state selections and
independent numerical evidence. Source/AST dictionary inspection here didnot
evaluate activityexpressions orqualify thepinned referenceenvironment.

## Henry forms and noncondensable-water configuration closure — 2026-11-05

**Interface-checked, source only:** C `modular_properties/phase_equil/
henry.py`,its `tests/test_henry.py`,the Henry-validation branch in
`base/generic_property.py`,and`examples/CO2_H2O_Ideal_VLE.py` withits matching
test resolve twodistinct required roles:Henry-method/form support anda
condenser property definition withCO2 excluded fromliquid. They arenot
interchangeable dissolved-CO2 models.

Henry forms areexactlyHcp=c/p,Kpc=p/c,Hxp=x/p,Kpx=p/x,withmolar concentration
c,molefractionx andpressurep. Units arerespectivelymol/(m³ Pa),Pa m³/mol,
Pa⁻¹ andPa. Pressure helperdivides concentrationbyH ormultipliesbyK;log helper
uses correspondinglog concentration±log coefficient. Electrolyte inputs must
explicitlyselecttrue orapparent concentrationbasis;nonelectrolyte usesnormal
phasecomposition. A concentrationform's vapor/liquid equilibriumratio needs
the liquid molardensity atthesame composition;one coefficient value cannot
be reinterpreted asa molefractioncoefficient withoutthat density andunits.

The genericbuilder validatesphase existence/liquidtype,method,type enum and,
forelectrolytes,basis presence. With `phases_in_equilibrium` configured,it
**refuses everyform exceptKpx**. The non-Kpx helper branches therefore do not
establish fullphase-equilibrium support. MEA'strue-basis Kpc callback above
isusedbyits separate single-liquid reactiveproperty/mass-transfer consumer;
it isnot evidence forarbitrary Kpc fullVLE. CubicEOS Henry refusal remains
independent ofthegeneric Kpx allowance. Invalidtype Dummy isatest refusal,
not afifth scientificform. Customcoefficient methods remain source-specific
callbacks,not afinite universal modelcatalog.

`ConstantH` returns thephase'sconfigured `henry_ref` unchanged withzeroT
derivative. Its constructiontest usesdummyH2O,Kpx=86 Pa,reference300 K/1e5 Pa.
The four equilibriumratio helpertests instead usecoefficient7 intheform's
ownunits,liquiddensity3 mol/m³,pressure5 Pa andliquidfraction .5,withdummy
EOS. Expectedratios are3/35,21/5,1/35 and7/5 inHcp/Kpc/Hxp/Kpx order.
These arbitrarysmall syntheticvalues checkunits/conversion,not scientific
waterHenrydata oraphysical equilibrium solve. No temperaturelaw,dataset,
trainer orexternal-functionEOS isselected bythese methodtests.

The CO2–water definitionhasthree phase-component members:H2O inLiq/Vap and
CO2 inVap only. BothphaseEOS areIdeal,FTPx,SmoothVLE/IdealBubbleDew with
waterfugacity equality;noHenrylaw,finiteaqueous CO2 orelectrolyte speciation
isselected. InlinewaterPerrys liquiddensity/Cp/enthalpy andNIST vapor
Shomate/saturation,plusCO2 NIST vaporcaloric data,usethe298.15 K/101325 Pa
reference. Waterliquid formationenthalpy is−285830 J/mol;vapors aredeclared0
atthereference. Preserve thatdatum inheatduty comparison. Flowbounds0..20
mol/s(initial10),T273.15..1000 K(initial323.15),P50000..1e7 Pa(initial108900)
areconstructor limits,not validatedproperty envelopes. Sourcecited water
density range273.16..333.15 K,liquidCp273.16..533.15 K,saturation255.9..373 K,
vaporShomate500..1700 K,andCO2 Shomate298..1200 K differ;selectedlow-temperature
vaporcaloric/saturation extrapolations needexplicitordinary permissions and
comparison allowances ratherthanasingle combined boundsclaim.

The isolatedstate point is10 mol/s,323.15 K/108900 Pa,H2O/CO2 .5/.5. Source
targets liquidwater1,vaporwater .11501/vaporCO2 .88499 andvaporfraction .56498
(absolute1e-4). Aseparate cooledflash hasfeed9.89433124673833 mol/s,
396.79057912844183 K,183430 Pa,CO2/H2O .13805801934749645/.8619419806525035,
deltaP0 andoutletT initially313.15 K;duty issolved. The sweeprequests50
linearly spacedoutlet temperatures313..396 K inclusive,two50-entry target
lists andthesamefeed. Vaporflow/feedfraction targetsrange .14388→1;duty
targets−396276.55508→−275.30623 W. Everypoint checksoptimal termination,
fractionabsolute1e-4 anddutyrelative1e-4;retain completephase/component/energy
closure anddatum beyondthose selectedassertions. This isacooling/duty-design
experiment,not anadiabatic outlet-temperature solve.

The full source/CSV-free closure isinlineproperty coefficients,methods and
50-point referenceoutput lists. Ordinarycoefficient provenance/envelope and
theoracle comparisonroles remain separate. Native D02/D04 needraggedphase
membership,waterproperty/datum,coolingfixed/free correspondence,Henry-form
permissions andinvalid-combination refusals. Independentpinned-environment
initialization/solve/derivative evidence remainsopen;AST listinspection
confirmed targetlistsizes only andexecuted noEOS,Henry expression orsolver.

## Helmholtz steam-turbine configured population — 2026-11-05

**Interface-checked, source only:** C `power_generation/unit_models/helm/
{turbine,turbine_stage,turbine_inlet,turbine_outlet,turbine_multistage}.py`,
their four dedicated test modules and`turbine` in`tests/test_compare_to_generic.py`
resolve the steam-turbine stage/train cases. Recursive owners include
`balance.py`,Helm splitter/mixer/steam valve,generic Heater,model serializer
andIAPWS95/Helmholtz external functions. The multistage fixture hasno saved
file;its in-memory specification snapshots arenot shipped initialization
data. Integrated NGCC/SCPC/subcritical/steam-cycle plant inputs remain separate.

All stages force dynamic=False,has_holdup=False,pressure/work transfer on,
heat transfer off. Time indexing under a dynamic parent is**pseudo-steady**,
not turbine fluid/metal/shaft inventory. The property guard requires an actual
Helmholtz parameter block,PH state andphase presentation MIX,L orG;LG
presentation andother state coordinates fail assertions. Selected tests use
defaultIAPWS95 water,PH/MIX,molar basis. Permitting L/G atconstruction isnot
an independent liquid-turbine/gas-turbine scientific qualification. Enthalpy
atconstant inletentropy/outletpressure definesisentropic work;actual outlet
enthalpy follows efficiency. Work/shaftpower use the energy-balance signed
work,negativeforgeneration;native positive-generation reporting needsan
explicit signmapping. Base efficiency isfixed .9 unlessa specializedstage
replaces it;mechanical efficiency defaults1.

| Actual fixture/role | Inputs andfixed/free/acceptance meaning |
|---|---|
| Regular stage | Inletinitial values15000 mol/s,70000 J/mol,8e6 Pa;ratio .7 andisentropic efficiency .8 fixed. Initialization mustleave3 inletDOF free andsatisfy everyactive equality toabsolute1e-4. It isnot a fullyfixed operating-point solve. Shaftspeed60 s⁻¹/mechanicalefficiency1 arefixed defaults;speed feeds a specific-speed expression,not a shaftdynamic law. |
| Inlet stage,rating | Inletinitial values6500 mol/s,24.233e6 Pa,h from880 K atthatpressure. Fixed defaultflowcoefficient1.053/3600 kg K½/(Pa s),blade reaction .9,velocity110 m/s,nozzle efficiency .95,mechanical efficiency1;isentropic efficiency andratio solved fromvelocity/flow correlations. Initializer restores3 inletDOF. |
| Inlet stage,coefficient calculation | Sameinlet;ratioinitial .6,initialize(calculate_cf=True) temporarilyfixes ratio andfrees coefficient,thenretains thecalculated coefficient while restoring originalspecification. Subsequentfixed-inlet solvechecks ratio .6/zeroDOF. This iscoefficient calibration,not independent validation ofthedefault coefficient. |
| Inlet time-indexed cases | Parenttime0..4 s,4 backwardfinite elements;constantinlet asabove atall5 points,coefficientfixed. Defaultcoefficient andcalculate_cf=True/.6 ratio variants check residual/DOF restoration. Nochangingboundary,carryover orinitial inventory. |
| Outlet stage,coefficient calculation | Inletinitial15000 mol/s,47115 J/mol,80000 Pa;outlet40000 Pa fixed. Both theordinary initialization test andexplicitcalculate_cf test **fitcoefficient**,because its initializer defaultsTrue. Firstretains2 DOF;second fixesinleth/P andchecks solvedflow15000/zeroDOF. |
| Outlet time-indexed calibration | Dynamicparent'sdefault timehorizon,4 backwardelements;constantinleth47115/P80000 andoutlet40000 fixed,flow guesses15000,coefficientfixed beforecalculate_cf=True initialization. Residual andflow/DOF checks remainpseudo-steady coefficient behavior. Explicitcalculate_cf=False default-coefficient rating isanother requiredrole. |
| Base turbine/generic comparison | BothgenericTurbine andHelmIsentropicTurbine get1000 mol/s,h from500 K/1e6 Pa,outlet700000 Pa,efficiency .9. Sourcecompares outletT andsignedwork relative1e-7 afterinitialization. SharedIAPWS95 means this isformulation correspondence,not an independent propertyoracle. |
| Multistage,reheater rating andcoefficient-calibration routes | Both use4 parallelthrottle/inletstages,HP7/IP14/LP11 regularstages plus1 outletstage. HPsplitafter4/7,IPafter5/14,LPafter4/7/9/11;IP14 has3 outlets,otherextractions2. HP7 disconnectinsertsone genericreheater beforeIP1 withoutletT880 K. Two routes sharetopology/steam butdifferent coefficient/initialization specifications below. |

Outlet science addsStodola flow,exhaust-volume fifth-order loss andwetness
efficiency. Defaultcoefficient .0333 kg K½/(s Pa),dry efficiency .87,
design exhaust6000 m³/s andmechanicalefficiency1 arefixed;losscoefficients
c0..c5=[6400,−32800,63800,−54200,22000,−3500] J/mol arefixed. Wetnessuses
outletvaporfractionx andfactor x*(1−.65*(1−x)),withloss/isentropic-enthalpy
correction. Theseinline empirical inputs andtheir domainmustbe admitted
ordinarily before usingthem asnative plantphysics. A waterproperty witness
doesnot qualify anexhaust-loss/partial-admission correlation.

The multistage inlet isinitiallyfixed26000 mol/s,24.233e6 Pa,h from880 K.
RegularHP/IP/LP stage ratios arefixed .88/.85/.82 andefficiencies .9.
Extractionfractions .03 atbothHPsplits,.04 atIP5/IP14 outlet2,.15 atIP14
outlet3,and.04 ateachLPsplit arefixed. Defaultlinear vapor throttlevalves
haveCv .1 mol/(s Pa)/opening1,subsequentlyfixed;inletmixer uses equalpressure.
Totalinletflow isthenfreed,so26000 isnot a persistentflow boundary intherating
route. Ratinginitializes disconnectedIP1 at7.802e6 Pa/880 K/25220 mol/s,
temporarilyfixes thatinlet,thenreconnects viareheater andadds880 K outlet
constraint. Calibration insteadstartsIP enthalpyfrom1.4e6 Pa/880 K,setsinlet
stage ratio .6 andrequests calculate_inlet_cf/calculate_outlet_cf pluscopy
disconnectedflow/pressure. It asserts recoveredtotalinletflow26000 aftersolve.
Bothfinallyfix outletstage pressure **atitsinitialized value**,not anexplicit
user-supplied condenserpressure inthisfixture. Freeze thatrealizedvalue in
anexecution record before claiming anexactrating reference;do notinvent one.

The multistage defaults aredifferent:4 parallel,HP2/IP10/LP5,noextractions/
disconnections,twooutlets whenasplit isselected. Number/split/disconnection
fields areinteger/list/dict domains,not a validatedall-integer catalog.
Sourceindexes first/laststages directly;zero/negativecounts,out-of-range or
duplicate splitindices,mismatched outletmaps andunconnectedexternal ports
needexplicit admissibility/refusal controls. Otherfinite planttopologies,
liquid/vapor valvephase,linear/quick-opening/equal-percentage andcustom
callback selections remainrequired. A customvalve selection withoutcallback
refuses;noncustom callbackwarns andusesselectedbuilt-in law. The authored
train selectslinear/Vap only. Thermodynamicpower sumsallstages;extraction
streams andreheater dutybelong inwholetrain component/energy accounting.

Source tests emphasizeequality residuals1e-4 andDOF restoration;multistage
doesnot supplyanindependent outputpower/efficiency oracle merely bysolving.
Ipopt with20-iteration multistage limit,externalIAPWS/Helmholtz andexact
serializer behavior remainreference prerequisites. Missingexternal functions
cannot becomea localparityskip. Ordinary empirical/data provenance,native
stage/train topology/fixed-free/products,remainingcategorical controls and
independent numerical acceptance remainopen F02/D02/D04 obligations. Source
andAST readsran noenthalpy evaluation,initializer,turbine orsolver.

## NGCC plant, serialized specification and load-series closure — 2026-11-05

**Interface-checked, source only:** X `docs/power_gen/ngcc/ngcc.ipynb` and
`mod/power_gen/{ngcc,gas_turbine,hrsg,steam_turbine}.py` resolve the default
baseline andreduced capture/load-series experiments. Allfour helpers are
required local source,withC natural-gas PR/reaction/flue-gas properties,
generic feed/product/translator/mixer/splitter/reactor/turbomachinery/HX,
boilerHX andHelmholtz steam units. No trainer orexperimental CSV isloaded.
Inlinegeometry,transport/performance coefficients andsaved modelstate are
separate reference inputs;theirpresence doesnot supplyordinary provenance.

The actual plantissteady. GT usesoneaircompressor,complete stoichiometric
combustion offourfuelhydrocarbons,three fittedexpansion stages andthree
cooling-air branches/mixtures,plusfuelpreheater. HRSG hasthreepressurelevels,
16 BoilerHeatExchanger banks (LP economizer/superheater;IP2 economizers/
3 superheaters;HP5 economizers/4 superheaters),3 generic evaporators,
phase separator,pumps,mixers,steam/flue splits andHP evaporatorvalve.
Steamtrain has**one** parallel inlet,HP7/IP10/LP11 plusinlet/outlet stages,
HP7/IP10 disconnects,reheat andLP steamaddition,three-way LP turbine/reboiler/
SOEC split,NTU maincondenser,hotwell,condensate isentropiccompressor andreturn
mixer. This differsfrom thecore4-parallel/7-14-11 trainfixture above.
Notebookimports neitherphysicalMEA absorber/stripper norcompleteCO2
compressiontrain;captureisrepresented bylumped steam/power correlations.

GT airspecies areCO2/Ar/H2O/O2/N2;combustionaddsCH4/C2H6/C3H8/C4H10;
fluespecies returntofiveaircomponents. Allthree callnatural_gas_PR.get_prop
withVap-only/defaultPR,FTPx,zero orderedkappa,reference298.15 K/101325 Pa,
withNIST/RPP caloric andChapman–Enskog/Eucken/Wilke/WMS transport inputs.
get_rxn selectsfourcomplete combustionstoichiometries withfuelconversion1,
not equilibrium dissociation,NOx formation orfinitekinetics. Natural-gas
helper'sadditionalIDEAL/Liq/scaled/component selections remain separate
required admissibility work;theNGCC doesnot selectallof them.

FreshGT inputssetair at288.15 K/101047 Pa,compositionO2 .2074,H2O .0099,
CO2 .0003,N2 .7732,Ar .0092;flow34875.9 mol/s isa guess ratherthanfixed
finalboundary. Fuel1348.8 mol/s at310.928 K/3.10264e6 Pa hasCH4 .931,
C2H6 .032,C3H8 .007,C4H10 .004,CO2 .01,N2 .016 andO2/H2O/Ar1e-19 each.
Itsmajor fractions sum1,thetraces remain explicit. Initialcompressor
efficiency .81/ratio19.075,vane deltaP−100 Pa,coolingflows2315/509/354 mol/s
andpreheater5000 m²/U100 W/(m² K) areconstruction/initialization inputs,
notnecessarily thesavedbaseline'sfinalspecification. GTstageefficiency
curves arethree distinctfifth-order polynomials of.896*inletvolumetricflow,
eachmultiplied1.02;headcurves arenegativecubicpolynomials ofitsnaturallog.
Their sixinlinecoefficient arrays andpositive-flow logdomain aremandatory
empirical inputs atthesourceowner;no trainingdata/file orcurvefit isexecuted.

HRSG steamusesIAPWS95;fluegas usesC FlueGasParameterBlock withN2/O2/CO2/H2O.
The planttranslator preserveseachofthosefourcomponent molarflows,T andP,
but**dropsAr**. It doesnot normalize remainingfractions orconserve omitted
argonmass/enthalpy. Native acceptance needsanexplicit approximated/discarded
boundary andenergy/element accounting ratherthana falselyclosed translator.
The capturepower/duty formulas useCO2 fromGTstage2 output,not finalstack
species;preserve thatselected measurementlocation andits approximation.
InlineBoilerHX bankgeometry/corrections,fluegas caloric/transport,
evaporatorLMTD/phase assumptions andHelmholtz waterartifacts formrecursive
scientific inputs. They donot becomequalified fromanassembled powernumber.

| Shipped input inX `docs/power_gen/ngcc/` | Actual serialization/consumer role |
|---|---|
| `ngcc_init.json.gz` | Format4,dated2024-07-25,rootfs. Completeplantstate/specification loaded bydefault;thisbypasses freshGT/HRSG/ST initialization andthefresh-branch fitted-parameter assignments. |
| `gas_turbine_init.json.gz` | Format4,dated2024-06-07,rootgt. Usedbythefreshwhole-plant fallback'sGT initializer;directGT loadreturnssubflowsheet state. |
| `hrsg_init.json.gz` | Format4,dated2024-06-07,roothrsg. CorrespondingHRSG fallback/directinitialization input. |
| `steam_turbine_init.json.gz` | Format4,dated2024-06-07,rootst. Correspondingsteamfallback/directinitialization input. |
| `gas_turbine_template.svg`,`hrsg_template.svg`,`steam_turbine_template.svg` | Required onlyfortheactualnotebook'sPFD display/write stage,openedrelative toworkingdirectory. Theyarepresentationtemplates,notphysicsparameters. Existing`data_pfds/*_baseline.svg` aregeneratedreference outputs. |

The fourgzip fileswereparsed withstdlib withoutloadingmodels. C StoreSpec
withsuffix=False stillloadsvalues,fixedflags,bounds,mutableparameters and
activeconstraints/blocks;itsdefaultignore_missing=True canignoremissing
components/attributes. This isa **serializedspecification**,not avalue-only
initialguess orreconstructedmodel. Rootcomponent names/oldclassstrings,
fullcomponent/indexcoverage,missing/newcomponents andintendedfixed/active
sets mustbe reconciled againstIDAES2.13/Pyomo6.10.1 beforeclaimingcompatibility.
Examples2.10 metadata andsuccessfulJSON decodeareinsufficient. Keepallfour
files asreference inputs evenwhenthedefaultbaseline consumesonlythewhole
snapshot;loadingone cannotestablish freshinitialization orsubflowsheet fits.

The wholesnapshot explicitlyfixesnetpower646 MW,capturefraction .90,
specificreboilerduty2.7e6 J/kg,specificCO2 compression274800 J/kg,
additionalCO2/reboilerduty0,fuelLHV47.2e6/HHV52.3e6 J/kg andfuelprice4.42
USD2018/MBtu. Inspectedsavedoverrides agreewiththefreshfit branch:compressor
efficiency .840187,outletsteamflowcoefficient .1099345,evaporatorHP/IP/LP
areas15000/11000/14000 m²,HP4 HTCcorrection .65939 andmaincondenserarea1000 m².
Thefreshbranch additionallysetsIP3 HTC1.372068,HP1economizer7.863405,
IP1economizer .6/LPeconomizer20,LPfluesplit .5,condensercoolinginleth1260
J/mol andHPevaporatorvalve deltaP−7e6 Pa;thefrozencase mustuseactualsnapshot
specification,not casuallymixfreshdefaults andloadedvalues. Steamoutlet
pressure is10kPa infreshsteamsetup;theintegratedpath laterfreesit anduses
condenserclosure. Mainsteam/reheat temperatures andturbineflowcoefficients
participate inintegrated sizing;aninletseed isnot afinalfree/fixed contract.

Baseline notebookselectsIpopt/ma57,user-scaling,automaticMA57 scaling,
pivtol1e-5/pivtolmax .1. Itloads snapshot,solvesandcomparesNETL B31B-style
aggregates:net646 MW,grosssigned−690 MW,GTsigned−477 MW,LHVefficiency52.8%
(abs .1 percentagepoint),variable/fuel/otherrates37.2799/31.6462/5.63373
USD2018/MWh(relative .01 fortherates). Gross/GT checksuse .001relative;
netusespytestdefaultcomparison. Net includespumps,fan,combustion,capture,
compression,transformer andmiscauxiliaries. OtherO&M isalumpedHHVenergy-rate
coefficient7.457044934107763e-10 USD2018/J;fuelcostusesHHV whileefficiency
usesLHV. Thoseassessment bases arenot interchangeable anddonot constitute
completecapitalcost/LCA qualification. Sourcebaseline checksdonot establish
wholeplant component/energybalance,bankprofiles orcaptureprocess science.

`run_series=False` disables theseparate100-request series:650→160 MW by5
(99 gridmembers),with646 insertedsecond. Ifselected,itsets97% captureand
2.4e6 J/kgspecificduty,Ipopt tolerance1e-6/max_iter50. Eachpoint eitherloads
`data/ngcc_<integerMW>.json.gz` withfullStoreSpec orfixespower/solves/saves;
afailedsolvebreakstheloop,soasummaryCSV cancontainonlyaprefix. Cachedpoint
loads donot independentlyverifytheloadedpower/specification. Trackall100
requested outcomes,capture-change consistency,cachecoverage/fixed-active
specifications andfailures;neverdeduplicate646 inthis97% experimentwiththe
90% baseline. StreamCSV outputsatthe650 MW point,seriesCSV,plots andnew
snapshots aregeneratedresults,not originalscientific inputs. Thoseper-load
caches areabsent fromthetrackedinputtree. Thefourownerload/savebranches,
wholeplantclosure,nativescientificsource/case/productmapping andnumerical
referenceexecution remainrequired F02/D02/D04/G06/H01 work. Noenvironment,
propertyfunction,serializerrestore,plantor solverwasexecuted forthiscensus.

## Boiler fireside dictionary and operating-input closure — 2026-11-05

**Interface-checked, source only:** C `power_generation/unit_models/
boiler_fireside.py`, `tests/test_boilerfireside.py` and its imported
`tests/datadictionary.py` resolve one solved operating fixture and a second
construction-only air-specification role. The test dictionary contains
**constant output values**, not a trained heat-transfer surrogate. The
subcritical plant's `generic_surrogate_dict.py` remains a separate required
correlated-input definition; neither this constant fixture nor NGCC qualifies it.

Both authored constructors select 12 waterwall zones, platen and roof,
steady operation and default six-component FlueGasParameterBlock
(H2O/CO2/N2/O2/SO2/NO). The first fixes primary/secondary component flows;
the second sets `calculate_PA_SA_flows=True` but only checks construction and
19 remaining DOF. It supplies no operating inputs, initialization or solved
air-demand reference. Defaults instead select 16 zones, both superheaters
and externally supplied air. For a fixed zone count the two superheater
Booleans and two air-specification modes expose eight selections, of which
only both-superheaters/explicit-air is numerically exercised by this source.
Other zone counts need a matching dictionary and explicit authored cases;
they are not an all-integer qualification population.

The required dictionary keys are zone indices 1..N, optional `pl`/`roof`,
and `flyash`/`NOx`. Missing dictionary refuses; entry count must equal
N+platen+roof+2, then each selected key is accessed. Matching length alone
does not establish valid key identity or expression domain. The test's 16
string entries evaluate to constants: zone duties in MW are
[20,10,10,10,15,10,12,30,25,20,18,10], platen50, roof65; flyash unburned
organic mass fraction .0001; NOx140 ppm. These are **duties in W**, not heat
fluxes per unit area despite source labels. Equations set duty times the
correction equal to the dictionary value. With both corrections .95, duty
is divided by .95; multiplying the prediction by .95 changes the science.
Roof uses the waterwall correction. The source executes expressions through
Python eval; native scientific declarations need the selected mathematical
meaning and input domains, not adoption of an arbitrary string evaluator.

The solved fixture supplies raw coal44 kg/s, moisture mass fraction .20,
vaporized moisture fraction .60, temperature335.15 K and dry HHV25.81e6 J/kg.
Dry ultimate-analysis mass fractions are C .6300,H .0412,O .1300,N .0127,
S .0284,ash .1577 (sum1). Dry/ash-free and dry/as-received bases must remain
distinct. Mill-vaporized moisture is **added** to primary-air H2O; the
post-mill coal retains the remaining moisture. Primary moist air is reset
to coal temperature in the furnace energy balance, which therefore does
not independently account for a complete mill energy balance at the
333.15 K inlet temperature.

Nominal combined air is21290.6999 mol/s, allocated .34 primary/.66 secondary.
Each component flow equals its allocation times H2O .0078267,CO2 .000337339,
N2 .783994,O2 .20784,SO2 .00001,NO .00001. These factors sum1.000018039;
they are supplied component flows rather than a normalized composition
constraint. Preserve their actual totals. Both inlet pressures are101325 Pa;
primary/secondary temperatures333.15/650.15 K. Waterwall temperatures are
690 K,platen750 K,roof650 K; with constant dictionary outputs they do not
create a temperature-dependent radiation/transfer experiment. Lower-furnace
stoichiometric ratio1,waterwall/platen corrections .95 and pressure loss
1000 Pa are fixed. Positive deltaP is subtracted from secondary-air pressure
in this owner; native signed-pressure-change mapping must be explicit.

The explicit-air initializer fixes component flows and frees PA/coal ratio,
total stoichiometric ratio and dry O2 percentage. The calculated-air branch
instead frees air component flows and requires its ratio specifications to
leave zero DOF. Its default air table supplies all non-N2 species while N2
closes the remaining flow. Initialization therefore changes fixed/free state;
do not assume the generic fixed-state-restoration contract without checking
these deliberate owner effects. Time/holdup flags inherit broad Boolean
domains, but the body constructs time-indexed algebraic state/balance laws
without accumulation derivatives. Dynamic-parent construction cannot qualify
coal/gas/solid thermal inventory or waterwall dynamics.

NOx input becomes a mole fraction of NO by dividing ppm by1e6; separate
lb/MMBtu reporting converts NO mass to NO2 equivalent. The unburned flyash
term includes dry ash-free organic elements, not carbon alone. Atomic masses,
coal enthalpy/HHV conversions,ash sensible heat referenced to298.15 K,
unburned-organic caloric terms and flue-gas thermodynamics are recursive
ordinary-input obligations, separate from the constant oracle dictionary.
The source target is flue outlet1236.780228 K (absolute1e-3), plus total mass
residual absolute.01 kg/s and energy residual absolute.001 W, including
all zone/platen/roof duties,ash and unburned fuel. Native acceptance must
retain complete elemental/products/energy accounting and each solid outlet.

There is no CSV, fitted JSON, trainer, saved initialization or external
water-property call consumed by this selected model. Its upstream test
nevertheless has an IAPWS availability skip decorator; that incidental skip
must not become a scientific prerequisite or relax local fail-never-skip
policy. Exact flue-gas/Ipopt reference compatibility, calculated-air and
optional-section cases, actual correlated plant dictionaries, ordinary coal/
coefficient provenance and native fireside implementation/qualification
remain required. AST/dictionary reads confirmed the constant values and
input sums only; no expression, initializer or solver was executed.

## Subcritical recirculation and correlated-boiler input closure — 2026-11-05

**Interface-checked, source only:** C `power_generation/flowsheets/
subcritical_power_plant/{subcritical_boiler,subcritical_boiler_flowsheet,
generic_surrogate_dict}.py`, `flowsheets/test/test_subcritical_boiler.py` and
`subcritical_power_plant/tests/test_subcritical_flowsheets.py` resolve two
distinct boiler constructions and their selected steady/dynamic experiments.
Full `subcritical_power_plant.py` steam-cycle/controller coupling remains a
separate required closure; a circulation loop is not the whole boiler or plant.

The **ten-zone recirculation** model is steady, with no holdup in Drum,
Downcomer or WaterwallSection; drum/downcomer/ten waterwalls form one closed
water/steam circulation path with feedwater and steam boundaries. IAPWS95
water is consumed; the constructed FlueGasParameterBlock supplies no gas-side
combustion experiment in this loop. Drum diameter1.2 m,length15.3256 m,level
.6 m,6 downcomer connections ofdiameter .38 m and zero heat loss are fixed.
The separate downcomer hasdiameter .3 m,height40.1 m,6 parallel pipes and
zero heat duty. Preserve the different connection/pipe diameters.

Allten waterwalls have610 tubes,diameter .047 m,wall .0035 m,fin .00455 m,
fin length .0115 m,slag .001 m andpressure-drop correction1.2. Ordered heights
are[6.15,3.15,1.5,1.45,1.35,1.25,1.15,1.35,3.25,3.45] m;projected areas
[320,150.3,70.8,70,58.6,58.6,50.1,65.6,145.6,165.5] m². Fresh initialization
fixes zone duties[23e6,15e6,6.8e6,12e6,12e6,12e6,10e6,9.9e6,**2.1**,20e6] W.
Zone9 is literally2.1 W,not21 MW. Drum initialization seeds recirculating
water/steam199470.7831 mol/s,10903981.9107 Pa,26585.3483 J/mol andfeedwater
4630.6098 mol/s,samepressure,22723.907 J/mol. Final rating fixes feedwater
flow/enthalpy and frees feedwater pressure;internal port seeds are released.
The first solve creates `subcritical_boiler_init.json.gz`;an existing file
instead restores the complete model specification using default StoreSpec.
Core tests run in a temporary path,so their intended default route is fresh
initialization,not automatic reuse ofthe archived file.

Core `run_sensitivity()` adds20 heat multipliers1.1..3.0 by.1. It uses the
same ten duty pattern except **zone9=21e6*multiplier W**,frees feedwaterflow
and fixes its pressure/enthalpy. This is a change ofspecification andheat
pattern,not simply a10–200% perturbation ofthe2.1 W baseline. It records
vapor fraction/feedwater flow even if a solve fails;printed failure text
does not prevent stale values entering plots. Retain every requested input,
solve disposition andrealized pressure fixed fromthe preceding solution.
Core rating tests separately check optimality,positive downcomer deltaP,
recirculation component-flow agreement andfeedwater=steam flow toabsolute
.001 mol/s. Those are selected loop controls,not complete boiler energy/
wall-profile qualification.

X archived `power_gen/subcritical/subcritical_boiler_src.ipynb` calls these
same core create/set/initialize functions. Its shipped gzip isformat4,
dated2022-09-16,root`unknown`;parsed zone9 duty isalso2.1 W. The archive
addsthree uniform slag thicknesses .001/.002/.003 m,thenreloads its complete
snapshot andadds the20-point heat sweep withzone9=21 MW. Slag effects and
theheat-specification change are real additional consumer roles,not wrapper
duplicates. It appends steam/vapor/slag results after failed solves too.
The PNG is a presentation schematic. This content comparison resolves the
candidate's meaning;conditional archival entry-point adoption remains with
A01 ratherthan silently adding/dropping it. Original source experiments
already required by the core loop retain their status. Old snapshot/class/
component compatibility andordinary geometry/slag provenance remain open.

The **twelve-zone correlated boiler** selects a different coal,geometry and
topology:steady fireside withcalculated air;Drum1D with4 radial elements;
12 waterwalls;downcomer,roof/platen,waterpipe,attemperator,primary-air mixer,
3-stream air preheater and4 crossflow2D/header banks RH1/RH2/PSH/ECON with
4/2/6/5 elements. Radiation ison inRH1/RH2/PSH andoff inECON. Water/metal
holdup isconstructed where selected. Dynamic parent time is0..60 s withtwo
backward elements (0,30,60);drum/waterwalls/selected crossflow metal inherit
dynamic behavior,whilefireside,downcomer,roof,platen,pipe,mixers andair
preheater explicitlyremain steady. A common time index doesnot give each
unit the same accumulation law. Its initial-condition methods andcopied
steady state require state-by-state native inventory correspondence.

The correlated dictionary hasexactly16 outputs:12 zone duties,platen,roof,
flyash andNOx. Each expression uses aselected subset ofthe12 wall
temperatures,platen/roof temperatures,rawcoal flow,moisture,SR,lower-furnace
SR,secondary-air temperature andPA/coal ratio. Inline polynomial,cross,
logarithmic andexponential terms make its domain different fromtheconstant
dictionary;logged flow/moisture/ratios/temperatures requirestrict positivity.
AST inspection found no file access ordivision denominators inthese
expressions;no trainer,dataset,fit options,validation split ortraining
envelope is supplied. Preserve every authored coefficient atitsimmutable
reference owner for oracle reproduction;ordinary adoption requires the
original correlated-model provenance,domain andphysical-output meaning.

Its drycoal compositionC/H/O/N/S/ash is
.717259/.0506301/.0789829/.0140639/.0282403/.110824,HHV30.5052e6 J/kg,
rawmoisture .1112,rawcoal29 kg/s,mill temperature338.7 K andmoisture
vaporization .6. AirO2/N2/CO2/H2O/SO2/NO factors are
.206201/.777811/.0003346/.0156532/1e-7/1e-7. Drum geometry is16 m long,
1.8 m innerdiameter,.13 m wall,level .9 m,8 downcomers of .375 m,
ambient300 K/insulation .15 m;separate downcomer .375 m×45 m×8.
Waterwalls use660 tubes,.055 m diameter,.0055 m wall,.005 m fin,
.013 m finlength,.001 m slag andpressure correction1.2. Their12 heights are
[7,4.8,2.6,2.6,2.65,2.7,2.7,2.7,2.75,3.5,5.5,5.5] m andareas
[359.738,249.6,135.2,135.2,137.8,140.4,140.4,140.4,143,183.81,179.3,179.3] m².
Roof/platen/bank wall,fin,slag,fouling/elastic/transport andheader geometry
remain explicit inline inputs in`set_inputs`,not values recoverable from
theten-zone loop. No scientific input file isloaded by this fullboiler route.

Initialization seeds wall temperatures
[641,664,722,735,744,747,746,729,716,698,681,665] K,platen790 K/roof643 K,
SR1.19/SR_lf .97/PAcoal2;these wall values andduties arethenfreed andcoupled
toactual slag-boundary temperatures/heattransfer. Final economizer water
inletfixes14e6 Pa/18335.7 J/mol withflowfree;reheat inletfixes3.1e6 Pa/
55879 J/mol withflow=.9*mainsteamflow. Blowdown=.02*economizerinletflow.
Airpreheater coefficients,heated-primary fraction,PA/coal anddryO2 are
loadpolynomials;their comments swap PA/SA labels,so useactualside2/side3
port identities. Atcoalq kg/s,side2 UA=−150q²+9400q+65000,side3UA=30000q+100000;
heated-primary fraction=.0003q²−.0175q+.5,PAcoal=.0018q²−.11q+3.45,
dryO2%=−.00075q³+.067q²−2q+22.95. These empirical laws need admitted
domains andcannot beextrapolated asuniversal controls.

Steady source targets furnaceexit1399.96 K,waterwall329144395 W andtotal
418839867 W,relative1e-5,plus selectedsteam massbalance absolute.01 mol/s.
Dynamic experiment firstcopies steady values intoalltimes,solves aconstant
steady trajectory,thenstepscoal+2.5% at**t≥30 s**,withtime0 unchanged.
At60 s ittargets1408.9636 K,waterwall334508402 W,total426313190 W.
This isone two-interval step experiment,not a converged time/mesh study or
closed-loop plant ramp. Exact Ipopt/ma57,Helmholtz andheat-transfer/metal
methods,ordinary surrogate/coal/geometry provenance,native differential
states andfull profile/element/energy acceptance remain required. Full
steamcycle/powerplant/controller andarchive scope dispositions remainopen.
Allreads here were source/AST/gzip inspection;no surrogate,initializer,
inventory model,propertyfunction orsolver wasexecuted.

## Subcritical steam-cycle and full-plant control closure — 2026-11-05

**Interface-checked, source only:** C `subcritical_power_plant/
{steam_cycle_flowsheet,subcritical_power_plant}.py`,the shared controller
owner and `tests/test_subcritical_flowsheets.py` resolve standalone steam,
connected steady plant,standalone pressure step androlling load-demand ramp.
They consume the twelve-zone correlated boiler above andIAPWS95 water,
Helm steam/valve/pump/condenser anddynamic heater/tank owners. No CSV,
saved model snapshot or trainer isread by these two helpers. Inlinegeometry,
correlation data,controller initial states androlling state transfer remain
scientific inputs even without an external file.

The steam turbine isone parallel inlet,HP14/IP9/LP5 plusinlet/outlet stages;
HP14 isdisconnected forreheat. Extractions occurafterHP14,IP6/IP9 andLP2/4/5;
IP9 hasthree outlets. Its custom throttle fixesCv1 anduses openingo with
function .028904o³−.033497o²+.014514o−.0014533. This empirical valvefunction
isnot thelinear NGCC orcore multistage fixture. Full train also includes
boiler-feed-pump turbine/front/outlet stages,main/aux condensers,hotwell and
deaerator tanks,condensate/booster/feedwater/drain pumps,valves,six feedwater
heater roles andspray attemperation. Selected dynamic heater sections were
enumerated above;the whole loop must preserve their drain,condensate,spray,
makeup andsteam-extraction boundaries rather than sum onlymain turbine ports.

Standalone steam fixesmain inlet61958 J/mol,12.5e6 Pa andletsflow bepressure
driven (11000 mol/s isaguess);reheat inlet63754 J/mol,2.8e6 Pa withflow
linked toHP exhaust. The initialization startscondenser pressure6000 Pa,
thenfrees turbine backpressure forcondenser closure;it doesnot retain6000 Pa
asafinaluniversal boundary. Source standalone steady targetsmainflow
12473.27146 mol/s andgross power266.8806 MW,relative1e-5. The separate
connected steady plant targets61634.3740 J/mol,14908.39189 mol/s and320 MW.
Those aredifferent fixed/free/consumer cases,not rescaling ofone oracle.
Five arcs connectmainsteam,cold/hotreheat,feedwater andspray;standalone
reheat-flow/feedwater constraints aredeactivated andallcoupled ports freed.
Plant slidingpressure is14.4+.02*(grossMW−320) MPa. Gross heatrate and
efficiency usedrycoal HHV;auxiliarypower isaseparate empirical multi-input
polynomial withmixed engineering-unit conversions,not native unit-free
arithmetic orfull equipment-wise electrical accounting.

Dynamic standalone steam uses0..60 s,2 backward elements,initial steady
state copiedtoalltimes,thenstepsIP1 inletpressure+5% att≥30 s. Seven
controllers remainactive;full sourceplots/printed outputs arenot terminal
numerical assertions. Its actual test `test_dynamic_steam_cycle` calls
get_model(dynamic=True) andchecks0 DOF;it doesnot callmain_dynamic's pressure
step orassert trajectory physics. Full-plant dynamic construction defaults
totimepoints[0,20,200] s,nstep2;these pre-existing points formanonuniform
grid,not anassumed100-second uniform mesh. `init=False` buildtests expect
157 dynamic or−5 steady DOF undercurrent PIDinitial-equation semantics;
neither isasolved physicalacceptance case.

| Controller,allcalculate_initial_integral=False | PV→MV,type,gains andfixed setpoint/bias beforesteady-bias reset |
|---|---|
| FWH2/FWH3 level | Condenselevel→corresponding outletvalve,PI,Kp−.1/Ki−.02,setpoint .275 m,bias .5. |
| FWH5/FWH6 level | SamePV/MVrole,PI,Kp−.1/Ki−.1,setpoint .275 m,bias .5. |
| Deaerator level | Deaerator tanklevel→condensate valve,PI,.1/.1,setpoint2.75 m,bias .5. |
| Hotwell makeup | Hotwell tanklevel→makeup valve,PI,.01/.001,setpoint .7 m,bias .35;smoothMV bounds .05..1,epsilon1e-4. |
| Mainsteam spray | Mainsteamtemperature→spray valve,PID,Kp−.05/Ki−1e-4/Kd−1e-4,setpoint810 K,bias .25;smoothMV bounds .05..1,epsilon1e-4. Derivative isonnegativeprocess-variable derivative,not error derivative. |
| Drum master | Drumlevel→feedwater-demand adjustment,PI,40000/100,setpoint .9 m,bias0. |
| Drum slave | Feedwater pumpoutlet molarflow→pump-turbine valve,PI,.02/.0002,bias .5;setpoint=drum adjustment+mainsteamflow+blowdown. |
| Turbine master | GrosspowerMW→throttleopening,PI,.0005/.0005,bias .6;setpoint isdemand profile. |
| Boiler master | MainsteampressureMPa→coal kg/s,PI,10/.25,bias29;setpoint=sliding-demandpressure+.02*(demandMW−actualMW). |

Onlymakeup/spray selectcontroller MVbounds;the othernine default tounbounded
controller outputs evenifthecorresponding physical valveisexpected between
0 and1. Alluse defaultantiwindupNONE,not conditional integration orback
calculation. Initialintegral components arefixed0 beforesteadystate bias
reset;spray derivative stateinitial0. Warmstart replaces controllerbiases
withrealized valve/coal states,initializesbounded integral terms fromtheir
reference expressions,andfixes initial heater/tank/drum levels andselected
coal/throttle/spray states. This isacomplete controller/state preparation
contract,not just forwarding aPIgain pair. Preserve controller-equation
behavior attime0;current sourcetests explicitlyaccount foritschanged DOF.

The fullplant ramp selects71 consecutive60-second periods usingthesame
model object withtwo backward intervals (nodes0,30,60 s). Thedeclared
second120-second modeliscommentedout/None,andnoactive periodselectsit.
Demand isrelative tosolvedsteady grosspowerx0:holdx0 for0≤t<60;ramp linearly
tox0/2 over60..660;holdhalf over660..2460;rampback over2460..3060;holdx0
through4260 s. That is5% ofinitial load perminute,not 5% ofcurrentload or
netpower. Outputjoining skipsrepeated windowstarts,soacomplete runhas143
unique30-second nodes,not213 independentstates or71 newphysical plants.
Allrequesttimes/holds andfinal inventory mustremain explicit.

Eachnew windowcopies theprevious terminaltime's values toinitial/guess
times withcopy_fixed=True,excepttheunchanged terminalnode ofthereusedmodel.
Differential inventories,controller integrals/derivative states andalgebraic
seeds therefore havedistinct carryover/initializationroles. Betweenwindows,
source manuallylimitsspray integral to.3 whenunboundedMV isbelowitslower
bound andintegral>.3,or−1 whenaboveupperbound andintegral<−1. This selective
reset changes thecontrol law;it isnot thecontroller'sbuilt-in antiwindup.
Native control/evolution mustpreserve orseparately select thatpolicy.
DynamicdryO2 additionally gets .05*(spraytemperature setpoint−actualT)
ontop ofthecoal-load polynomial;the steady dryO2 equation isdeactivated.

`run_dynamic` setsdemandateachglobaltime,solves,thenunconditionallyappends
coal/air/dryO2,power/duty,drum/heater/tanklevels,steam/feedwater/spray flows,
temperatures/pressures,valve/controller andwall/thermal products. It doesnot
checkoptimality orstopbeforeappending stale values. Trackevery71-window
outcome and143-node demand/trajectory completeness;`case_5pct_result.txt`,
plots andPFD SVGs aregeneratedoutputs,not reference inputtruth. Full-plant
dynamic sourcetest explicitlydeclines thesimulation becauseofitsruntime;
construction/checkDOF doesnot establish theramp. Standalone/connectedsteady
tests checkselectedgross products andsteam massbalance only. Exactnative
inventory/derivative/controller semantics,ordinary empiricalcoefficients,
solver/initialization compatibility,fullconservation andindependent complete
trajectory qualification remainrequired E02/F02/G05/H04. Source/AST reads
ran nosteady plant,controller,rollingcopy,modelinitialization orsolver.

## Supercritical steam-cycle and SCPC configuration closure — 2026-11-05

**Interface-checked, source only:** C `flowsheets/supercritical_steam_cycle/
supercritical_steam_cycle.py`, `supercritical_power_plant/{
boiler_subflowsheet_build,SCPC_full_plant}.py` and `test/{test_scsc,
test_scpc_plant}.py`, plus X `docs/power_gen/supercritical/
supercritical_{steam_cycle,power_plant}.ipynb`, resolve three assembled
case roles. Standalone steam-cycle andboiler-network construction are
dependencies oftheconnectedplant,not evidence thattheir specifications or
outputs remainunchanged aftercoupling. Allselected experiments aresteady.

Steamcycle usesIAPWS95 PH/MIX andfourparallel inletstages,HP7/IP10/LP11,
HPsplits4/7,IP5/10,LP4/8/10/11,HP7 reheatdisconnect andthreeoutlets atIP10.
It includescondenser,hotwell,condensate/boiler-feed/drain pumps,feedwater
heater/extraction train andboiler-feed-pump turbine. Mainsteam fixes24.23e6
Pa/62710 J/mol,flowpressure-driven (21000 mol/s isaguess),inletcoefficient
4.6e-4,outletcoefficient .32. Fourlinear throttlevalves haveCv .0009 and
opening .9 each. HP/IP/LP regularstages fixratio .8/.78/.76 andefficiency .9.
Reheat haszero pressurechange,866 K outlet andlinkedHP/IP flow. Standalone
boilerpressureloss fraction .1 closesfeedwater-to-mainsteam pressure;the
detailed boiler laterreplaces it.

Condenser coolingwater2.5e6 mol/s,1700 J/mol,500000 Pa,area13000 m² and
U15000 W/(m² K);makeuph2500 J/mol/P101325 Pa,condensatepump efficiency .8/
deltaP1e6 Pa arefixed. Heaterareas,Uvalues,pump efficiencies andextraction
initial guesses areinline source inputs intheexistingfeedwater-heater
population. Initialization seedsbackpressure3500 Pa andspecifiedextraction
fractions,thenreleases extractions/backpressure totheheater/condenser
closure. Source steadygrosspower target622.38 MW (absolute .01 MW),zeroDOF
andallactive equalityresiduals<5e-4. The valvechange sets**onlyparallelvalve1**
opening .25;otherthree remain .9 andsource targets594.66 MW. This ispartial
admission,not a uniform throttle-to25% experiment. The testdoesnot assert
optimaltermination forthechangedpoint;retainthatindependent obligation.

`main(initialize_from_file=None,store_initialization=None)` performsfresh
initialization bydefault. Optionalfileloading restoresdefaultStoreSpec and
skipsinitialization;there isno selectedshipped scientificsnapshot inthese
modern wrappers. Optionalnew snapshots areoutputs. PFDtemplate isC
`supercritical_steam_cycle.svg`;modernnotebooks also displayX
`Boiler_scpc_PFD.svg`. Templates arepresentationinputs,not thermaldata.
Stream tables/PFD results anddisplayedgrosspower arenot additional numerical
acceptance. Bothmodern notebookfamilies invoke theexistingC owners without
addinganotherparameter dataset orscientifictrainer.

Boilernetwork usesfourcountercurrent,in-line BoilerHeatExchanger units
ECON/PrSH/FSH/RH,watercold/fluegashot,pressurechange onboth sides,noholdup.
ECON selectsLiq water/no radiation;otherthree selectVap/radiation. Waterwall
andplaten aregenericHeater units withimposed duties,not coal/fireside or
waterwall/metal transportmodels. FSHfluegas splits .75 toreheater/.25
toprimarysuperheater,recombines andentersECON;steam runsECON→waterwall→
primary→platen→finishing→spraymixer. This selectedsplit differsfromsome
independent initializationflow guesses (.85/.18) thatarefreed onconnection.

Suppliedfluegas nominal21290.6999 mol/s hasH2O .0869,CO2 .1449,N2 .7434,
O2 .0247,NO .0006,SO2 .002. Factors sum**1.0025**;actualcomponentflows
mustnot besilently normalized. FSHgasinlet1300.335 K/100145 Pa isphysical
networkboundary;ECON682.335 K,PrSH/RH1180.335 K areinitializationstates
laterfreed atconnectedports. ECONfeedwater initial24194.177 mol/s,
26922222.222 Pa,14990.97 J/mol;RHinitial21235.27 mol/s,3677172.33638 Pa,
53942.7569 J/mol. Waterwallheat751 MW andplaten55 MW arefixed initially.
The fourbank geometries,cold-phase correlations,emissivity .7,fouling and
HTC/pressure-drop corrections areinline requiredinputs. Native coal/fireside
science cannotbeinferred fromtheprescribedfluegas andheaterduties.

Othersteam initialstates useIAPWS enthalpy evaluatedatPrSH773.15 K/
24865516.722 Pa,FSH823.15 K/24790249.01 Pa,platen798.15 K/24790249.01 Pa,
waterwall588.15 K/25449000 Pa,andattemperator875.15 K/24865516.722 Pa.
Sprayinitial flow .001 mol/s has**122e6 Pa** pressurefixed whileitsenthalpy
iscomputedfrom563.15 K/**25.449e6 Pa**. PreservetheactualindependentP/h
specification;do not re-evaluateenthalpy at122 MPa orcallthisthesameT/P
point. Couplednetwork inletseeds arefreed selectivelybyunfix_inlets;
spray/FSHphysicalboundaries remainexplicitconsumerroles.

Standaloneboilertest fixesATMP1 outlet62710.01 J/mol andfrees sprayflow,
checksoptimality andECONoutlet521.009 K. Its `pytest.approx(521.009,1)` uses
relative1 (100%),so itdoesnot supplyaprecise temperatureacceptance bound.
The networkconstructor byitself doesnot initialize/solve;source testorder
prepares itthroughinitialize/unfix/ratingspecification. Reproduction needs
thatcomplete boundedprocedure andphysicalcheck products.

ConnectedSCPC firstsolves thesteamcycle,builds/initializes boilernetwork,
solvesdisconnected,thenconnectsFWH8→ECON,ATMP1→HP,HP7→RH andRH→IP1.
Itdeactivates lumpedboiler pressure/flowclosure andreheatflow/P/T equations,
freescoupled ports,thenfixesmainsteampressure24.23e6 Pa andattemperator
outlet62710.01 J/mol while**freeingplatenheat**. Mainsteamenthalpy control
thereforechangesaduty-designrole,not simplya622.38 MW cycleextension.
Hotwellmakeup lowerbound isrelaxed to−1 mol/s,andfinalsolve strips**all
variablebounds** withareversibletransformation withoutrestoring thembefore
return. Thoseareactualreference allowances;native state/flow/efficiency/
property-validity acceptance mustbechecked independently ratherthan adopted
asanordinaryfeasibility waiver. FinalIpopt selectstol1e-6/ma27/max_iter40.
Coreconnectedplant testassertsoptimaltermination only;nofullplantpower,
component/element/energy,bankprofile ororiginal coalbalanceoracle follows.

The selectedclosure containsinlineproperty/geometry inputs,Helmholtz
waterexternalartifacts,fluegas caloric/transport andpresentationtemplates;
noCSV,coaltrainer,saved-stateinput orphysicaltime trajectory isloaded by
default. Original empirical/geometry/fluegas provenance,envelopes andsigned
work/duty/boundary accounting remainordinary-source obligations. Native
F02/D02/D04 needeachstandalone/coupled specification role,partialadmission,
profile/conservation andproperdomain products. Archivedsupercritical wrapper
comparison andremainingunitcategorical cases stillremainrequiredcensuswork.
Source/AST readsran noenthalpyfunction,network,serializer,initializer orsolver.

## Carbon purification surrogate configuration/input closure — 2026-11-05

**Interface-checked, source only:** C `power_generation/unit_models/cpu.py`
and `tests/test_cpu.py` resolve one CarbonProcessingUnit experiment. It is
an empirical cryogenic-purification surrogate with one inlet and three
outlets named pureco2,water,vent. It does not construct a cryogenic column,
compressor train,property package or energy balance. Five component identities
are fixed:Ar,CO2,O2,H2O,N2; arbitrary species subsets/orderings are not a
configuration family in this owner.

The actual function argument order is flow mol/s,Ar,CO2,O2,H2O,N2 fractions.
Eighteen active surrogate relations specify cooling duty,compressor work,
pure-product totalflow/T/P/four componentflows,water totalflow/T/P/four
componentflows,andventT/P. Product/water N2 flows close from their totalflow
andfraction normalization;vent componentflows close from five unit component
balances,then vent totalflow/fractions close. Eight other fitted helper
functions (productN2,waterN2,venttotal andfive ventcomponents) exist but are
**not called by the model**. Do not compare native balanced vent products
against those unused fits or assume ventH2O=0 because its unused helper
returns zero. Pure-product H2O is actively fixed to0 by its fit.

The coefficient functions are inline affine combinations/constants;there is
no fitted JSON,CSV,trainer executable,saved initialization or external EOS
input. Work,cooling,totalproduct andproductCO2 depend on flow andCO2 fraction;
waterflow/H2O fits depend on **O2** fraction (argument4),not inletH2O
(argument5). Remaining component/T fits consume their actual selected
arguments. Preserve that mapping;substituting a scientifically more plausible
water-input coefficient would be changed science. The source describes
ALAMO fitting from300 Latin-hypercube samples of a rigorous model,but ships
no sample population,bounds for each composition,split,seed,trainer options
or rigorous-process input/model. Those remain held required training/
provenance obligations,not established by direct evaluation of shipped fits.

Source stated flow range10850..11850 lbmol/h (expected11350) converts to
1367.077..1493.075 mol/s (expected1430.076),using453.59237 mol/lbmol. The
only test fixes **1260 mol/s**,below that stated training range,with
Ar .005,CO2 .900,H2O .05,N2 .025,O2 .02 (sum1). It initializes withIpopt
tol1e-7,ma27,max_iter40,release_state=False,then solves. Source targets
pure/water/vent flows1103.3/37.1/119.6 mol/s (absolute .1),CO2 purity>.99,
cooling35262510 W andwork21402861.6 W (absolute1). Both duty andwork are
positive reported empirical products;there is no enthalpy-based energy
equation establishing their sign/boundary relation.

Source training assumptions specify inlet100°F/14.7 psia,about310.928 K/
101353 Pa,but actual inletT/P variables are unused by the equations and
unfixed by the test (initial values110 K/17 Pa). Their port presence does
not grant thermal/pressure response. The DOF statistic excludes unused
variables;zero DOF after fixing flow/five fractions does not resolve those
physical inlet states. There is no inlet-fraction-sum equation;component
balances imply totaloutflow=Fin*sum(inletfractions),so the physical totalflow
closure requires normalized admitted inlet fractions. Flow/fraction variables
have no positivity/simplex bounds. Negative residual species/outputs outside
the fit domain therefore need native admissibility and failure controls,
not acceptance solely from optimal termination.

The build method bypasses the parent UnitModel build and creates algebraic
variables/constraints at parent times;no storage or dynamic accumulation is
present. Its initializer always usesIpopt despite its solver argument,
fixes onlytime0 flow/fractions,andwith defaultrelease_state=True unfixes all
six afterward rather than restoring their original fixedness. The selected
test deliberately usesFalse. Other release/time/invalid-input configurations
retain explicit execution/refusal work;this isnot an inventory controller
or general multi-time initialization witness.

Native F02/G04 requires selected artifact mathematics andargument mapping,
three complete outlet products,component/element closure,fit/training domain,
andseparate ordinary provenance for an empirical process replacement.
Reference evaluator/embedding evidence must not become ordinary rigorous
cryogenic/separation/energy science. ExactIpopt compatibility,source-domain
extrapolation permission,training closure andindependent numerical
qualification remain open. Source/AST andunit-conversion arithmetic here
executed no fitted helper,model initializer or solver.

## Original scientific source leads

These links identify scientific formulation or parameter provenance to investigate
at B/D/F owners. They do not admit a complete ordinary coefficient package, establish
all numeric transcription/convention choices, or authorize a newer formulation to
replace the pinned reference problem. Original source, upstream parameterization,
matched differential evidence and independent validation remain distinct roles.

| Consuming demand | Original source and pinned-source correspondence |
| --- | --- |
| Water/steam — D02/F02 | [IAPWS formulation release](http://iapws.org/relguide/IAPWS95-2016.pdf), cited by C `general_helmholtz/components/parameters/h2o.json`; [Wagner–Pruß 2002](https://doi.org/10.1063/1.1461829). The [2018 release](https://iapws.org/public/documents/k3PSK/IAPWS95-2018.pdf) explains subsequent revisions; choosing it for ordinary science must preserve or explicitly distinguish the selected reference constants/datum. |
| CO2 — D02/F02 | [Span–Wagner 1996, original paper at NIST](https://srd.nist.gov/jpcrdreprint/1.555991.pdf), cited by C `components/parameters/co2.json`. A later CO2 EOS is different science, not evidence for this selected EOS. |
| PR and electrolyte activity — D02/D03 | [Peng–Robinson 1976](https://pubs.acs.org/doi/10.1021/i160057a011); [Song–Chen 2009 symmetric electrolyte NRTL](https://pubs.acs.org/doi/10.1021/ie9004578), cited by C `eos/enrtl_reference_states.py`. Variant, reference-state and interaction choices must still match the exact upstream configuration. |
| TSA cycle formulation — E04/F05 | [Joss et al. 2015](https://doi.org/10.1021/ie5048829), identified by C `fixed_bed_tsa0d.py`. Its equilibrium shortcut is the selected physical-model class; do not infer an axially resolved adsorption transport model from its bed name. |
| TSA adsorbent parameters — F05 | Zeolite: [Hefti et al. 2015](https://doi.org/10.1016/j.micromeso.2015.05.044); phase-change adsorbent: [Hefti et al. 2016](https://doi.org/10.1039/C6FD00040A); amine-functionalized polystyrene: [Elfving et al. 2017](https://doi.org/10.1016/j.jcou.2017.10.010). These correspond to the source's three parameter-building branches. Bibliographic checks corrected the zeolite publication year and the phase-change title/author order relative to source shorthand; full parameter/table correspondence remains open. |

## G — Studies, inference, control and assessment

`G` owners refer to [64g](64g-analyses-studies-and-applications.md). Native starting
surfaces are `crates/pse-runtime/src/workflow/{fitting,uncertainty,horizon,controls,study}.rs`
and their existing modules, `completion.rs`, `local_analysis.rs` and
`result_inspection.rs`. Existing source is not evidence for the following complete
configured workflows. Shared product permission remains C05's owner; G01 selects
each consumer's originating point/step/coverage and required-versus-optional products.

| Required demand / owner | Exact upstream case/configuration and dependency closure | Native/data/reference decision needed |
| --- | --- | --- |
| NRTL and PR inference — G02/D02 | X register rows 11/12 NRTL state-block/flash using `docs/param_est/BT_NRTL_dataset.csv`; row 37 CO2–bmimPF6 PR state-block fitting (despite flash prose) using `docs/properties/CO2_IL_298.csv`; C `apps/uncertainty_propagation/examples/NRTL_model_scripts.py` | Shared parameters across experiments, experimental state/basis/noise mapping, bounds and covariance/identifiability demands must be explicit. The datasets have different scientific roles from feed data. Native fitting starting surfaces do not establish either complete parameter recovery or appropriate uncertainty. |
| Local uncertainty/sensitivity — G03/G01 | [C uncertainty examples][uncertainty]: Rooney–Biegler and NRTL fits, optimized output andthe NRTL control named infeasible (source says feasible); `sens.py`/`uncertainties.py` | Exact covariance/derivative convention and optimized reference point are prerequisites. Preserve infeasible/ill-conditioned outcomes;the named feasible control does not establish them. This source does not establish general Bayesian, robust or Monte Carlo equivalence; selected broader native methods have extension evidence. |
| Robustness population — G03/H04 | [V robustness][robustness] CSTR, flash, Gibbs/equilibrium reactors, HX0D, isothermal pressure changer, each with JSON baseline/source configuration; e.g. flash declares 200 Latin-hypercube samples | Imported JSON baseline statistics are historical observations, not a zero-failure waiver. Need actual realized input population, bounds/sampler/version, initialization and failed-case policy; a sampling count is not a configuration count. Do not silently drop failed samples. |
| PySMO training/evaluation/embedding — G04 | C polynomial/RBF/kriging and sampling tests; X `pysmo_basics`, reformer flowsheet, best-practices utility `mod/surrogates/AR_training_methods.py`; X `docs/surrogates/reformer-data.csv` and PySMO `data_files/` | Preserve fitted family, terms/kernel/options, scaling, train/validation split, domains, persisted JSON, evaluation outputs, equation embedding and optimization separately. Inspect remaining benchmark/helper variants before closing the population. |
| ALAMO — G04 | X reformer pipeline plus sCO2 ALAMO training → embedding → optimization; `alamo_surrogate.json`, run/control artifacts; C `core/surrogate/alamopy.py` | External trainer executable/version is required for trainer reproduction; a shipped fitted JSON can support separately selected artifact evaluation/embedding evidence. It cannot prove the training stage. ALAMO absence holds required execution. |
| Keras/ONNX/OMLT — G04 | [C Keras/ONNX source/tests][neural]: full/reduced smooth-space, ReLU big-M and ReLU complementarity. Core test artifacts include sigmoid/ReLU Keras models and `tests/data/onnx_models/net_Calcite_ST.onnx` plus its IDAES metadata; X reformer and sCO2 Keras pipelines | TensorFlow/Keras/ONNX/OMLT versions, supported layers/activations, bounds/scalers and solver class must be selected exactly. ONNX standalone evaluation is not granted by import/embedding. Smooth NLP and discrete/complementarity routes need separate native backend/support decisions; no library substitution is adopted here. |
| sCO2 pipeline closure — G04/D02/F02 | X `docs/surrogates/sco2/{alamo,pysmo,omlt}` each has training, property embedding and flowsheet optimization; shared `500_Points_DataSet.csv`; local `properties.py` and `flowsheet_optimization.py`; JSON or `.keras`/`idaes_info.json` artifacts | These are three pipelines with three demanded stages each; do not count nine unrelated fluids, nor drop training or optimization by deduplicating them. Each property package must preserve the fitted output/datum/domain and downstream compressor/exchanger meaning. Shipped fits and retrained artifacts have different evidence. |
| PID/NMPC and native MHE — G05/E02 | C `models/control/controller.py`; X steam PID and SOC PID; [C Caprese CSTR][cstr-nmpc] with steady target, plant/controller horizons, initialization and tracking cost | Need physical-time/hold/initial-state/observation and constrained-horizon meaning, feasibility and failure policy. MHE remains a native extension: core documentation [states the module is forthcoming][mhe]. Controller availability is not full state-estimation evidence. |
| Multiperiod/grid — G05 | [C grid][grid]: `multiperiod/tests/test_multiperiod.py`; thermal-generator and Prescient plugin examples; bidder, tracker, coordinator/backcaster/forecaster, price-taker commitment/clustering/design-operation tests with `pricetaker/tests/lmp_data.csv` | Period linking/ramping, market boundary, prices/scenarios and commitment decisions need exact mapped scientific products. Prescient/integration dependencies remain environment prerequisites where consumed; mocked construction tests are not a full market simulation. Grid period coupling does not provide physical time integration. |
| Costing, environmental/utility assessment — G06/D05 | [C SSLW][sslw], power QGESS/NETL costing and JSON coefficient files, TSA DAC costing; [C utility minimization][utilities]; X HDA/methanol/SCPC assessment consumers | `costing.pse` starting surface. Freeze capacity/basis/year/currency, operating period/utilization, physical contributor, allocation and coefficient provenance. Released cost/utility methods and selected native environmental assessments have separate scope; general LCA is not established by a costing module. |
| Diagnostics, scaling, inspection and extension — G01/G06/D05 | X notebook register rows 1–10, 24, 31–35 and visualizer; diagnostic moving-bed local modules; C diagnostics/initialization/scaling tests | Expected structural/numerical finding, initialization failure report, zero-state analysis and qualified point products must remain separate. Native source/case/conformance and inspection/result owners carry identities; no new result database or universal scanner is needed. Full GUI imitation is not a P1 prerequisite, while selected native visualization/reporting remains a workflow obligation. |

## Input closure, duplicates and source-category dispositions

| Source category / exact population entry | Disposition and closure result |
| --- | --- |
| Modern documented notebook families | Required scientific behavior; the existing 55-family register retains exact entry points. This investigation read code-cell imports, configuration/data accesses and the named helper sources above. Generated `_doc`, `_test`, `_usr`, exercise/solution/source renderings do not add members; meaningful science changes still require explicit variants. The full normalized-wrapper content equivalence has not been established. |
| Modern HDA/custom/methanol helpers | Required with the consuming cases. Local `mod/hda`, `mod/methanol`, `mod/properties`, reactor `egprod_ideal.py`, `egprod_reaction.py`, `msr_reaction.py` and custom liquid-extraction modules must accompany the reference source. Modular and legacy HDA formulations cannot be declared duplicates without a scientific correspondence. |
| Modern power helpers and saved initialization | Required with NGCC/SOC cases. Nested helper imports identified above and shipped compressed initialization are reference prerequisites. Generated stream CSVs, plots and solver transcripts are outputs, not original physical input data. Serializer compatibility and all recursively loaded files remain execution-readiness work. |
| Modern surrogate helper/data/artifacts | Required selected training/evaluation/embedding/optimization stages. `AR_training_methods.py` itself imports TensorFlow and multiple trainer families; a helper import can require more than the notebook's chosen fitted family. Reformer/sCO2 datasets, JSON/`.keras` metadata/scaling and local property code are separate input artifacts; obsolete pickle compatibility is not assumed. |
| Held NETL-32D fixed-bed notebook | Required, held execution: `held/flowsheets/CO2_adsorption_desorption/CO2_Adsorption_Desorption_1DFixedBed.ipynb` → `mod/co2_adsorption_desorption/{NETL_32D_gas_phase_thermo,NETL_32D_solid_phase_thermo,NETL_32D_adsorption_reactions,simulation_utilities}.py` plus core fixed bed/PETSc. This is not zeolite TSA0D and cannot be deduplicated into that row. Resolve adsorption/desorption input profiles, inventory transfer, parameters and PETSc compatibility. |
| Archived reconciliation: boiler, economizer parmest/reconciliation | Conditional supplemental source selection remains open; scientific reconciliation is already mandatory G02 behavior. Read/archive-specific inputs, covariance/noise and boiler/economizer models before deciding whether these add cases to the existing required workflow. Tree presence alone neither selects nor excludes them. |
| Archived NGFC, rSOFC/SOEC, SOEC, SOFC, combined SOFC/SOEC, subcritical boiler | Conditional archived entry-point selection; released core power/SOC behavior remains required above. Models/ROMs/helper data may differ from modern cases; content equivalence and all nested prerequisites have not been established. No blanket duplicate/exclusion declaration. |
| Archived supercritical plant/steam-cycle entry points | Candidate duplicates of modern entry points; retain pending content/scientific comparison. Core/modern plant cases remain required even if archived wrappers retire. |
| Six archived MatOpt design families and C `apps/matopt` | Conditional supplemental whole-distribution track, outside the process-only equivalence claim. Retain materials-design scope for any later whole-release claim; no deletion or rejection of the library. |
| Archived PECOS data quality | Conditional supplemental workflow; do not claim its application-specific data-quality behavior from native process inspection. |
| Archived DMF and embedded workspace copy | Excluded obsolete infrastructure mechanics; retain any scientific artifact selected by an actual required consumer. The copied workspace notebook is a source duplicate candidate, not an extra scientific workflow. |
| Developer cache/tag notebooks and empty dummy | Excluded non-science source machinery as stated above. These exclusions do not change any mandatory process model. |
| Core tests, V API examples and V independent verification | Required behavioral/scientific evidence sources selected by the case mapping, rather than a promise to reproduce Python test APIs. V state-junction, translator, feed-flash, NTU/LC/HX1D/valve and custom-model cases supplement modern notebooks. Independent verification remains oracle/validation material unless separately admitted with original scientific provenance. |

## Next closure work and consumers

H01 remains open for the following concrete work; these are missing denominator
and readiness products, not missing test passes to be replaced by a source-count claim:

1. Expand the inherited per-model admissible balance/dynamic/holdup configurations,
   callable policies and actual parameterized scientific fixtures, including full
   remaining fluid/configuration selections, solid-liquid/thickener, power units, SOC, grid and
   surrogate benchmarks. Record physically invalid/unsupported combinations as
   refusals. The tables preserve their mandatory owners while enumeration proceeds.
2. Inspect all remaining archive contents and compare candidate wrappers scientifically.
   Keep held required behavior separate from optional archived entry points. Reconcile
   any actual scope change with A01/the coordinator before dropping a demand.
3. Resolve complete helper/data/external-artifact closures for each first executable
   case; freeze actual Python/solver/external-function/trainer versions. Source
   accessibility does not qualify the examples/core environment. Original scientific
   parameter provenance and ordinary-versus-oracle permission are B's source-owner work.
4. Map the selected native declaration/case, fixed/free roles, envelope, physical
   outputs, derivative/statistical products, comparison allowances and original
   conservation/feasibility checks at the existing typed case/conformance/reference
   owners. H02's ordinary recycle/HX correspondence remains its independent first-case
   owner; this inventory does not replace it.

D01 can use the scientific/consumer groups immediately. E01 can settle the three
first physics increments without awaiting all power/grid cases. F01 has separate
bed/capture/TSA/SOC/network variants and real prerequisite distinctions. G01 has
specific originating-product, statistical, artifact and control demands; G04 still
owes the selected native backend/reuse/refusal decision. None of these handoffs
closes D–G runtime delivery or H04/H07 qualification.

[core]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6
[examples]: https://github.com/IDAES/examples/tree/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples
[compat]: https://github.com/IDAES/idaes-compatibility/tree/8f1ae7a3ca358625128a78722bc2fdb75753a441/src/idaes_compatibility
[examples-project]: https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/pyproject.toml
[cv-types]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/core/base/control_volume_base.py
[cv-base]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/core/base
[states]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/state_definitions
[helmholtz]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/general_helmholtz/helmholtz_functions.py
[helmholtz-parameters]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/general_helmholtz/components/parameters/__init__.py
[activity]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/activity_coeff_models/activity_coeff_prop_pack.py
[cubic]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/eos/ceos.py
[phase-equil]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/phase_equil
[enrtl]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/eos/enrtl.py
[property-examples]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/examples
[pure]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/pure
[transport]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/transport_properties
[verification]: https://github.com/IDAES/idaes-compatibility/tree/8f1ae7a3ca358625128a78722bc2fdb75753a441/src/idaes_compatibility/verification/properties
[mea-properties]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/column_models/properties
[reaction-methods]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/properties/modular_properties/reactions
[units]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/unit_models
[unit-tests]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/unit_models/tests
[solid-liquid]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/unit_models/solid_liquid
[example-modules]: https://github.com/IDAES/examples/tree/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/mod
[cstr-model]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/apps/caprese/examples/cstr_model.py
[fixed-bed]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/gas_solid_contactors/unit_models/fixed_bed_1D.py
[tsa]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/temperature_swing_adsorption/tests/test_fixed_bed_tsa0d.py
[power]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/power_generation
[columns]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/column_models
[membrane]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/co2_capture_and_utilization/unit_models/tests/test_membrane_1d.py
[soc]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/power_generation/unit_models/soc_submodels
[gas-network]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models_extra/gas_distribution
[uncertainty]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/apps/uncertainty_propagation/examples
[robustness]: https://github.com/IDAES/idaes-compatibility/tree/8f1ae7a3ca358625128a78722bc2fdb75753a441/src/idaes_compatibility/robustness/unit_models
[neural]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/core/surrogate
[cstr-nmpc]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/apps/caprese/examples/cstr_nmpc.py
[mhe]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/docs/explanations/modeling_extensions/caprese/mhe.rst
[grid]: https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/apps/grid_integration
[sslw]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/models/costing/SSLW.py
[utilities]: https://github.com/IDAES/idaes-pse/blob/84532d7f99a9f976b05170948147afdd69cfa5f6/idaes/core/util/utility_minimization.py
