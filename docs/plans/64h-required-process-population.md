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
