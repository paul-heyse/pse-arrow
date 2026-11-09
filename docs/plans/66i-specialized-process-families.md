---
title: "Plan66i: Specialized process families"
status: planned
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [current-work, design, validation]
---

# Specialized process families

## Ownership and present foundation

This companion of [Plan66](66-integrated-simulation-target.md), the sole go-forward
series, owns remaining specialized authored physics and complete family consumers.
It preserves every required configured family and its ordinary input closure while
allowing ready independent families to proceed. [66l](66l-required-population-and-assembled-acceptance.md)
owns the exact case denominator and reference readiness; this document owns scientific
implementation and family-local acceptance. Source enumeration is not family qualification.

**Interface-checked baseline:** main `66341514d` contains reusable states, property and
reaction definitions, generic process/control-volume/equipment source, selected steam
equipment and native mathematical/workflow owners. Their existence does not establish
all steam, bed, capture, adsorption, electrochemical or network configurations. Carry
only missing family science, actual integrations, migrations and receiving evidence.
Do not replace working generic mechanisms or rerun all delivered controls as new work.
The source census at `git:66341514d:docs/plans/64h-required-process-population.md`
is the immutable provenance for 66l's transferred selectors, helper closures and input
roles, including held required cases and conditional archived entry points.

The target applies combined-review I01/I02/I03/I09/I10 and scientific S01–S04/S07.
Scientific laws remain authored in the existing [reference packages](../../packages/reference/);
class-specific native libraries perform mathematical iteration. There is no family-name
Rust dispatcher, universal ontology prerequisite or second external-worker domain model.

## Complete configured population

| Family | Required scientific scope and consequential configured variation |
|---|---|
| Steam/power and pure-fluid equipment | Subcritical boiler/plant, supercritical steam cycle and complete SCPC; NGCC gas turbine/HRSG/steam-turbine subflowsheets and design/load sweeps; selected sCO2/reformer consumers; matching property, work/heat, sizing and costing roles |
| Power unit population | Fireside zones/platen/roof and surrogate dictionary; inline/staggered, liquid/vapor, radiation and side-flow exchangers; 2D/header/radial meshes; feedwater-heater drain mixer/desuperheat/drain-cooling choices; waterpipe end effects, watertank geometries, waterwall/steamheater laws; three-stream exchanger, heater/drum1D; downcomer/drum/CPU/balance and Helmholtz pump/compressor/mixer/splitter/phase separator/condenser; multistage HP/IP/LP/inlet/outlet/split/throttle selections and steam-valve laws |
| Gas-solid and chemical looping | Dynamic/holdup fixed-bed 0D and TGA; fixed-bed 1D forward/reverse flow, reaction-free/reactive, simple/Ergun pressure drop and energy options; moving-bed countercurrent transport with opposing discretization directions; bubbling-fluidized-bed and chemical-looping configurations, distinct gas/solid states, inventories and heterogeneous reaction/transfer |
| Columns and capture | Equilibrium tray/feed/side-draw/condenser/reboiler/column with selected tray count/feed position, pressure/heat and bubble-point/custom-temperature specifications; solvent absorber/stripper, MEA enhancement, condenser/reboiler and vapor-flow/heat-duty freedoms; actual liquid/vapor speciation, transport and reaction heat |
| Membrane and coupled separation | Co/countercurrent, sweep/no-sweep, energy/pressure and mesh choices; CO2/H2O/N2 steam-rich-sweep solved consumer plus independently required other configurations; permeance/selectivity, geometry and coupled mass/enthalpy closure; selected plate-exchanger passes/channels/divider-plate behavior |
| TSA and held adsorption | All six shipped TSA0D configured cases, not only default zeolite; zeolite_13x, mmen_mg_mof_74 and polystyrene_amine loading laws, fixed/calculated bed allocation, selected compressor and steam approximations; heating/desorption/cooling/repressurization/adsorption/collection; the held NETL-32D 1D adsorption/desorption case remains a distinct required model |
| SOC/electrochemical | Selected SOC submodel/cell/stack and process benchmarks, controller examples and SOEC design; regional material/energy/charge transfer, reaction/property/geometry and heat-transfer choices; applicable dynamic/local-domain settings, current/voltage/thermal conventions and original ROM/artifact roles |
| Gas networks | Selected steady gas network and upstream transient linepack configurations; gas property basis, pressure/flow coupling, pipe/compressor/boundary data, network connectivity and actual compressible inventory/time closure |

For all rows retain effective inherited dynamic/holdup/balance defaults, admissibility
guards, state/property/reaction package choices and initialization policy. A finite
set of selected stage counts/meshes/geometry envelopes replaces an unbounded Cartesian
product; no scientific configuration is excluded merely because it needs data or a
reference optional dependency. Gas-solid reverse-flow collocation and incompatible
upwind choices remain attributable refusals. TSA's normalized within-stage time does
not establish generic parent-time dynamics; parent `dynamic=False` does not mean
stage inventory carryover is optional. Adsorbing CO2/N2 and externally exposed H2O/O2
membership have different meanings.

NGCC compressed initialization snapshots and serializer compatibility are reference
execution dependencies, not original physical truth. Keep nested helper imports,
property/correlation tables, gas/solid kinetic/isotherm parameters, ordinary geometry
and transient profiles separate from generated CSVs, plots and solver transcripts.
Modern SOC/NGCC requirements remain mandatory; archived NGFC/SOFC/SOEC/combined variants,
archived plant wrappers, MatOpt and PECOS have their explicit conditional/source
dispositions at 66l. A wrapper retirement never excludes underlying required science.

## Shared realization and extension boundaries

[66g](66g-properties-equipment-and-steady-workflows.md) supplies only the actual required
property/reaction/equipment slice. [66h](66h-inventory-distributed-and-cyclic-physics.md)
supplies only the consumed inventory/spatial/time/operating-program contract. Neither
whole document is a prerequisite. A ready steady power or equilibrium-column increment
can proceed independently of distributed adsorption and electrolyte completion.

Preserve original physical quantities, datum and membership through
[66a](66a-scientific-operations-and-product-semantics.md). Each family declares supported
value/derivative/regularity products and actual phase/regime/validity refusal. In
[66d](66d-mathematical-representation-and-numerical-execution.md), preserve linear,
square nonlinear, constrained optimization, residual/differential and mixed-discrete
structure rather than lowering every family to an opaque objective. Family laws and
discretization do not become solver-selection policy in generated adapters.

For this companion's family and receiving packages (I02–I07), consume
[D's shared products](66d-mathematical-representation-and-numerical-execution.md#symbolica-shared-products)
for repeated constitutive evaluation only on an actual compatible region. Beds, capture and
separation, electrochemistry, power and networks share preparation where domains, ordered
layouts and requested products agree. Preserve transport direction, ragged membership and
constitutive limits; exercise a reordered region or a differing validity premise before
qualifying reuse. Retain exact elimination guards. [Precision](66d-mathematical-representation-and-numerical-execution.md#symbolica-precision)
is demand-specific; precision agreement alone is no certified physical error bound.
faer 0.24.4 symbolic/numeric factor reuse and `MatRef`/`MatMut` support compact native
blocks; `LinOp`/`BiLinOp` and `lsmr` are bounded large-response alternatives when an
actual bed/network consumer justifies them. Native batch layout may use ndarray 0.16.1
views/Zip while preserving ragged regional species maps. Concrete source and applicability
are in [S02–S04](../plan_improvement_reviews/plan64-65-scientific-extensibility-analysis-2026-11-06.md).
Do not introduce tensor/graph/property engines merely because a library offers them;
an actual missing complete consumer contract must justify additional machinery.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| I01 | planned | 66l configured family/input selections and current source | Each mandatory variant has its actual physics/data/route/consumer or verification-only disposition, including held prerequisites and admissibility refusals |
| I02 | planned | Selected 66g pure-fluid/pressure/heat closure; required 66h storage only | Missing power/steam equipment and integrated plant/design/load consumers deliver original matched outputs, input provenance and family evidence |
| I03 | planned | Selected gas/solid properties, heterogeneous reaction/transfer and required 66h accumulation | Fixed/moving/bubbling-bed and chemical-looping cases preserve bases, direction, inventories, pressure/energy and original global conservation |
| I04 | planned | Selected 66g equilibrium/speciation/transport and required 66h distribution | Columns, MEA/solvent, membrane and coupled separation configurations deliver their own demanded simulation/design products and invalid-case behavior |
| I05 | planned | Selected adsorption/electrochemical science and required 66h stage/local-domain closure | All required TSA/SOC and held adsorption consumers retain physical carryover/regional balances and supported provider products |
| I06 | planned | Selected gas-property/pressure coupling and 66h linepack inventory | Steady and transient network consumers deliver complete boundary/flow/inventory outputs with explicit mesh/time and failure behavior |
| I07 | planned | Selected I02–I06 producers and actual 66b/66e/66f receiving paths | All configured family clients/readers/exports and assessment/control consumers migrate; replaced interpretation paths/fixtures are deleted and 66l receives independent family acceptance |

I01 produces family-local ready closures, not a new serial source-research program.
Begin I02 with the already selected HEOS Water equipment closure. Develop I03/I04
in parallel when their actual data and spatial/property contracts work; distinguish
a source helper already inspected from an executable scientific input still missing.
For I05, first close the selected zeolite physical cycle, then its other required
adsorbent/compressor/steam cases without claiming they are identical. SOC can proceed
through an independently ready local-domain contract instead of waiting for all TSA.

Every family increment carries native admission/execution, diagnostics, protected
selected reading, retained/exported original interpretation and its real engineering
consumer. [66j](66j-inference-uncertainty-design-and-control.md) and
[66k](66k-engineering-assessments-and-native-views.md) consume family products without
reconstructing physics. Reuse 66b contracts, 66c performing-owner/drain and 66e/66f
data/client mechanisms; don't defer all consumers to a final unbounded migration phase.
Delete legacy family mappings and duplicated equations only after the corresponding
selected callers move and independent controls distinguish the replacement.

## Verification and completion

**Planned verification:** each selected family has native force-validating local
controls and independently matched expectations, including material/element/energy
and where applicable charge closure, operating-envelope limits, original datum,
initialization/feasibility, supported derivatives and deliberate unsupported variants.
Spatial/time refinement, stage continuity and boundary perturbation test different
claims. A successful nominal solved fixture cannot qualify a different flow direction,
adsorbent, phase, mesh scheme, pressure law or power operating mode.

Use independent original scientific records and isolated IDAES 2.13.0 comparisons where
the selected formulation is comparable; distinguish intentional native alternatives.
Test wrong output, missing output, reordered regions/species and a different datum so
the comparator can fail meaningfully. Preserve failed/held cases in 66l without a
failure waiver. At scope end 66l assembles complete family/client/readback and selected
worker-placement acceptance; pure-fluid/provider component controls remain narrower.
No tests, numerical claims or performance measurements are established by authoring
this document. Completion requires the finite configured population and complete
consumers, not a family catalogue or an isolated scalar law.
