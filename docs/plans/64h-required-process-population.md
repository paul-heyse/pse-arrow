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
| NRTL and PR inference — G02/D02 | X register rows 11/12 NRTL state-block/flash using `docs/param_est/BT_NRTL_dataset.csv`; row 37 CO2–bmimPF6 PR flash fitting using `docs/properties/CO2_IL_298.csv`; C `apps/uncertainty_propagation/examples/NRTL_model_scripts.py` | Shared parameters across experiments, experimental state/basis/noise mapping, bounds and covariance/identifiability demands must be explicit. The datasets have different scientific roles from feed data. Native fitting starting surfaces do not establish either complete parameter recovery or appropriate uncertainty. |
| Local uncertainty/sensitivity — G03/G01 | [C uncertainty examples][uncertainty]: Rooney–Biegler and NRTL fits, optimized output and intentionally infeasible NRTL optimization; `sens.py`/`uncertainties.py` | Exact covariance/derivative convention and optimized reference point are prerequisites. Preserve infeasible/ill-conditioned outcomes. This source does not establish general Bayesian, robust or Monte Carlo equivalence; selected broader native methods have extension evidence. |
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
   pure-fluid registration, solid-liquid/thickener, power units, SOC, grid and
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
