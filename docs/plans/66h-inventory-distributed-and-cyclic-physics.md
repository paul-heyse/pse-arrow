---
title: "Plan66h: Inventory, distributed and cyclic physics"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [current-work, design, validation]
---

# Inventory, distributed and cyclic physics

## Remaining target and baseline

This companion of the sole go-forward [Plan66](66-integrated-simulation-target.md)
owns missing physical storage/allocation, distributed temporal accumulation and actual
cyclic operating programs, plus the receiving evidence for delivered slices. It consumes
the exact required configuration/input population from
[66l](66l-required-population-and-assembled-acceptance.md); no earlier plan remains
a prerequisite or parallel execution owner. Review I01/I02/I03/I09/I10 and scientific
S01–S04/S07 motivate complete physical consumers instead of a generic dynamics switch.

**Interface-checked baseline:** main `66341514d` already has homogeneous vessel integrated
and simultaneous formulations, native Diffsol/IDAS routes, temporal specialization,
indexed stock projections and spatial control-volume source. These are implemented
foundations, not new infrastructure tasks. The element-stock/projection source and
selected two-phase transfer controls do not establish all allocated energy or
phase-equilibrium trajectories. Existing [spatial control-volume source](../../packages/reference/process/models/control-volume-1d.pse)
explicitly refuses dynamics without time accumulation; no new E03-style distributed
temporal implementation exists at this baseline. Preserved source investigations are
not numerical acceptance.

The enduring authorities are [physical semantics](../authoritative_design/sections/physical-semantics.md),
[composition](../authoritative_design/sections/models-and-composition.md) and
[workflows/results](../authoritative_design/sections/workflows-and-results.md).
Extend their existing authored accumulators, [temporal specialization](../../crates/pse-modeling/src/temporal.rs)
and native formulation owners. New kernel/pass/relation contracts follow the existing
decision route before implementation; do not create a second physical time graph.

## Physical behavior and mandatory populations

Inventory is an extensive physical quantity with an actual constitutive allocation;
EOS normalization amount, stream flow and conserved stock are not interchangeable.
Component, phase-component, element and total material projections retain membership,
amount basis and initial origin. Energy storage declares internal/enthalpy and formation/
sensible conventions, pressure/volume/work treatment and wall/fluid ownership so source
terms do not double count. Initial states and constraints must be consistent with the
selected differential/algebraic formulation, not invented from the first sampled output.

| Population | Physical/input/configuration obligations |
|---|---|
| Lumped vessel and CSTR | Steady initializer versus dynamic physical model; fixed volume, aqueous component inventory, reaction heat, component-total and enthalpy-total demand where selected; nonzero initial component/energy holdup, bounds/controller variants and actual feed/control schedules |
| Multiphase and wall/fluid allocation | Required phase/component/element/total storage projections; equilibrium and transfer closures where consumed; distinct phase appearance/disappearance; exchanger wall-only storage versus fluid inventory, wall heat capacity/resistance/fouling and independent initial conditions |
| Distributed inventory | Indexed cells/mesh, physical volume/length/area and flow units; initial profile, inlet/outlet/side boundaries, direction/upwind/collocation compatibility; temporal accumulation of the actual field and conservative source/flux accounting |
| Reactive/transfer consumers | Fixed-bed gas transport plus stationary solid inventory, methane/iron reduction, reaction-free/reactive balance and pressure drop; required PFR/pipe, exchanger and contacting cases with their actual chemistry, heat and spatial closures |
| Cyclic/batch behavior | TSA heating/desorption, closed cooling, repressurization and adsorption/collection; carried material/energy and stage boundary/transfer meaning; initial/final inventory, collection windows and periodic closure; distinct adsorbent laws remain 66i obligations |
| Program and numerical products | Scheduled switches, state-triggered events, resets and periodic boundary conditions distinguished; physical intervals/holds and requested output grid; supported sensitivities, errors and regularity scoped to the actual route and trajectory |

The immutable source basis is the E/F configured census and case specifications at
`git:66341514d:docs/plans/64h-required-process-population.md`, transferred into 66l.
Its concrete lumped enzyme-CSTR inputs originate in reference test helpers: helper
availability does not confer ordinary scientific provenance. The iron-oxide fixed-bed
and the zeolite TSA source are separate downstream consumers, not interchangeable
generic dynamic examples. Missing chemistry, isotherms or thermal inputs remain visible.

The enzyme CSTR still needs ordinary coefficients and conserved pools, a physical thermal-stock
map, and the original nonzero initialization. The iron-bed control additionally needs its
gas/solid and pressure/heat transfer data and a supported temporal route; the current general
control volume refuses dynamics. The TSA cycle needs the selected isotherms and thermal inputs
from 66i together with actual stage transfer and periodic closure. The existing aqueous holds
do not supply those case prerequisites.

## Representation, numerical route and reuse

Preserve indexed physical quantities through temporal specialization rather than
flattening mesh and species to an anonymous vector at the public boundary. Native
solver vectors can be compact while their original coordinate map remains owned by
[66a](66a-scientific-operations-and-product-semantics.md). Actual mesh/phase membership
changes invalidate structure; compatible coefficients or boundary values can reuse
immutable preparation where its validity conditions still hold. Mutable integrator,
factor and provider scratch remains attempt-owned under
[66c](66c-preparation-execution-and-service-lifecycles.md).

[66d](66d-mathematical-representation-and-numerical-execution.md) owns whether a selected
case uses existing integrated/simultaneous native routes or needs a bounded residual-DAE
extension. Inspect actual index, algebraic constraints, initialization and demanded
products first. Existing Diffsol/IDAS availability does not imply arbitrary hybrid or
high-index support. Event-sensitive derivatives need their real contract; a smooth
trajectory or provider observation cannot synthesize saltation/stability evidence.

Use Symbolica 3.0.1 shared multi-output evaluation and selective coefficient precision
remapping where demanded, preserving branch/domain assumptions. faer 0.24.4 sparse
symbolic reuse already supports native linear work; `LinOp`/`BiLinOp` with `lsmr` are
conditional large-response candidates, not a replacement for integrator state or an
automatic conditioning certificate. ndarray 0.16.1 views/Zip and faer borrowed matrices
can evaluate cell/field batches without Arrow round trips. Retain ragged phase/material
membership and charge actual factors, high-precision coefficients and scratch to their
native lifetime. Exact maintained source links and limits are in
[S02–S04](../plan_improvement_reviews/plan64-65-scientific-extensibility-analysis-2026-11-06.md).

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| H01 | in-progress | 66l selected case/input roles; existing stock/temporal source | The released wall-stock input is identified and authored; the enzyme CSTR still lacks admitted ordinary inputs. Resolve the remaining cohorts' allocation, law, boundary, route and independent expected products. |
| H02 | in-progress | Selected H01 lumped inputs; relevant 66g property/reaction slice | The standalone PH, zero-rate wall stock, capacity perturbation and prescribed two-phase transfer passed native receiving. Remaining lumped/multiphase cases and protected trajectory receiving stay open. |
| H03 | in-progress | H01 distributed case; selected native stock/spatial/temporal owners and 66d route | Finite one/four-cell integrated and simultaneous aqueous trajectories plus original protected export are integrated with independent boundary/stock/identity controls. Eight selected native controls pass cleanly; interior-cohort refinement and strict zero-initial scheme/domain refusal pass two targeted controls. The new site-varying initial stock passed targeted native receiving against the independent per-site solution. Original whole-member native and protected trajectory use passed main receiving. Broader physical families, point products and specialized consumers remain open. No general spatial-control-volume dynamics refusal is removed by this slice. |
| H04 | in-progress | Working H02/H03 storage law; selected ordinary operating program and 66i constitutive inputs where the case requires them | The first authored aqueous feed/wash/feed hold program passed targeted native receiving through the existing scheduler and stock owner. Physical stage transfers, state-triggered/periodic behavior and required adsorption-law consumers remain open. |
| H05 | planned | Changed H02–H04 route and demanded 66a/66d product contract | Selected trajectory/response/error/regularity products are independently checked and scoped; unsupported derivative or hybrid claims remain withheld |
| H06 | planned | Selected physical producers plus 66b/66e/66f receiving paths | Original fixture/program, native execution, retained/read/exported trajectories and family consumers migrate coherently; matching old adapters/fixtures are deleted and 66l acceptance is supplied |

H01 is case-local readiness, not an all-family gate. A narrow H03 first increment is
constant-density aqueous component transport with explicit cell volumes, flow/boundary
composition and isothermal energy treatment. Compare its analytic Erlang response as
an exact finite-volume/cell-cascade discretization reference. Independently accumulate
global input/output/storage balance. This does not qualify a continuum pipe, reactive
fixed bed or non-isothermal family. H03 then exposes the actual additional inventory,
energy, constitutive and solver needs of the iron-oxide consumer; H04 can separately
use a ready lumped cycle without waiting for all distributed families.

### First H03 delivery — November6

Main now includes [the authored aqueous cascade](../../packages/reference/process/models/aqueous-inventory.pse),
three original integrated fixtures and their
[native controls](../../crates/pse-runtime/src/workflow/modeling/aqueous_inventory_tests.rs).
One parent physical timeline evolves actual `PhaseInventory` cell members. Each interior cell
owns its original `V/N` material stock, incoming/outgoing flux and initial conditions; the inlet
is a boundary, not stored material. Original constructor geometry remains distinct from numerical
initial guesses. The selected uniform backward spatial scheme admits forward/zero flow;
non-isothermal, reactive, multiphase and alternative spatial schemes are not qualified here.

**Tested, targeted in the owning checkout:** `scripts/pse test pse-runtime --lib --features
solver-ipopt,solver-kinsol,solver-root-isolation,solver-diffsol -E 'test(aqueous_integrated_) |
test(aqueous_geometry)'` passed four controls, zero failures/pending/build warnings, with the
refreshed original scientific input capsule and complete capture/settlement. One/four cells,
nonzero original clock, nonzero/zero tracer stock and zero-flow operation are compared with
the independent finite-cell/Erlang response. Independent prefix closure consumes actual native
coordinate bindings and boundary quadrature IDs at every requested time; wrong-component joins
and a missing intermediate time fail the comparator. Authored whole-domain integral checks and
reports are terminal-only under the existing contract; the test does not claim earlier authored
closure observations. Constructor/derived geometry overrides refuse through native admission.
Main receiving remains separate work; the protected independent trajectory delivery below
adds a bounded original-result consumer.

The simultaneous one/four-cell 61-point cases are now integrated too. Their actual seven-control
scientific child reported seven passes, zero scientific failures/build warnings and complete
original capture, including nonzero physical clock and zero-flow inventory. Its native wrapper
remains **incomplete**: the manager connection closed during cleanup after child exit zero.
Kernel evidence independently identified global OOM killing the user D-Bus service; this is
separate from the earlier finite-placement launcher defect. The original scope is now absent,
but that observation does not rewrite the failed historical settlement receipt. A subsequent
clean native selection passed the same seven scientific controls plus the protected-delivery
consumer below:8/8, zero failures/pending/build warnings and complete capture/settlement.
The next two targeted native controls now pass with complete capture/settlement and zero failures
or build warnings: actual target-keyed numerical requirements expose a zero-initial Radau
scheme/domain conflict, retained as an original infeasible/unqualified diagnostic control, while
a separate interior-initialized cohort passes the unchanged eightfold coarse/fine common-time
error discriminator. Both preserve the original physical bounds and acceptance allowances.
[66d's numerical-demand section](66d-mathematical-representation-and-numerical-execution.md#numerical-demand-exposed-by-temporal-refinement)
owns the independent faer stage-equation check, production Symbolica stencil mechanism and
precise evidence limits. This is a selected temporal comparison, not general positivity or
convergence-order qualification.

**Implemented — spatially varying initial stock:** The same authored cascade now accepts an
initial concentration profile on the realized normalized site and species. Every actual
cell's original initial equation applies its physical `V/N` weight to that profile; integrated
terminal closure independently sums the original site stock specifications. Existing uniform
inputs use the same profile contract. A new four-cell integrated fixture at an original `5 s`
start supplies distinct initial tracer concentrations and independently specified outlet,
cell-stock response and boundary quadrature balance controls. The [November 9 checkpoint](66-integrated-simulation-target.md#66ah-execution-checkpoint--november-9)
records the targeted native pass. The profile does not widen the spatial scheme, flow, chemistry or
thermal admission.

**Implemented — H04 scheduled stock carryover:** The aqueous cascade now uses one independent
concentration input per original species for both the boundary and cumulative supply. The
authored single-cell feed/wash/feed fixture schedules only the tracer over `[5,6)`, `[6,8)` and
`[8,10] s` through the existing integration fixture schedule (ADR-0119 Outcome 2). The test
checks that both species retain their original stocks at the two settled changes. It checks the
named physical input, exact hold values, no reset and the analytical mixed-cell stock and
independent cumulative inlet and outlet fluxes at every original sample. **Tested, selected:**
the hold-program and original protected aqueous export controls passed on the default-profile,
force-validated runtime selection in the [November 9 checkpoint](66-integrated-simulation-target.md#66ah-execution-checkpoint--november-9).
This exercises a prescribed operating program; state-triggered
transfers, periodic closure and TSA constitutive inputs remain open.

The integrated protected-delivery consumer submits the original typed Study `SimulationOperation`,
retains its actual integrated fixture and checks the accepted native result at the nonzero clock.
Expected values come from original native samples plus admitted output/member/quantity/unit
contracts, independently of the canonical row mapper. It drops producing handles, clears the
preparation cache, reads protected canonical rows, exports bounded IPC and checks framing,
lineage, raw Arrow values/exact bits and dynamic Protobuf context. Foreign revision refuses;
Proto3 default zero and field presence are checked separately. Current `simulation_samples`
scientific usability was initially **Unknown**. A's original whole-member owner now supplies the
native grant and protected recorded decision with exact source/attempt/coordinate correspondence.
Main's 23-control selected receive includes this updated actual journey: complete original rows
are usable, one-row/inspection selections remain partial, and decoded completion has no live proof.
Foreign source/physical/preparation/profile/member/run and output/quantity/unit/time controls
withhold correspondence. This closes the selected original whole-member use gap; point/derivative
authority, other families and L02's wider failure/assembled population remain open.

D corrected the unrelated full-curvature estimate used for selected affine proof and repeated
retention charging of actual shared source/snapshot owners. All distinct support-copy and
consumer-attribution costs remain. Semantic identities, configured retention guards and physical
allowances were not changed to clear that refusal. These are owned-allocation accounting
corrections, separate from native allocation/RSS measurement.

**Selected H02 wall-stock delivery:** The inherited E02/D04 `TestHXLCGeneric` selection owns a
wall-only thermal stock with two no-holdup water sides at `100 mol/s` and `101325 Pa`, source
inlet enthalpies `4000/3000 J/mol`, `100 W/K` film conductances, `1000 m²` area and actual
`1 J/K` wall capacity. The existing steady exchanger owns an isolated pair of fluid duties;
wall storage needs separate side-to-wall exchanges and an independently initialized stock.
The authored [wall fixtures](../../packages/reference/seed-data/models/steam-fixtures.pse) now
translate the original IAPWS95 enthalpies into the stock ideal-gas datum and solve the two
selected PH liquid states through the existing HEOS temperature/density equations. The wall
model has separate side fluxes and a simultaneous backward step from the original zero-rate
equilibrium. A separate initial perturbation distinguishes the declared capacity and checks
wall stock against both fluid enthalpy-flow changes. Both simultaneous wall-stock controls
passed in the [November 9 force-validated runtime selection](66-integrated-simulation-target.md#66ah-receiving-and-physics-follow-up--november-9).
The separate PH-only control also passed against the two original native component roots.
Protected delivery of the wall trajectory remains open.

The authored `inventory_fixtures.closed_transfer` now has a focused native receiving control
for prescribed, two-phase benzene/toluene transfer. It checks both endpoints, all original
phase stocks and independent component, element and total closures. The same focused selection
passed after reading the exact typed coordinates from the original inventory equations. This
establishes prescribed transfer and does not qualify phase-equilibrium configurations.

For each increment retain immutable original initial/profile/schedule specifications
separately from mutable preparation overlays and actual applied inputs. The existing
original-fixture observation source needs receiving adapter/readback evidence where
changed; do not reimplement it. Program observation does not confer execution or
derived-product permission. A Study sequence supplies orchestration, while the physical
state transfer and time advance remain explicit native operations.

## Consumer cutover and planned acceptance

Each physical case must reach typed admission/execution, complete original-coordinate
trajectory inspection, independent export and the real specialized-family or control
consumer. [66b](66b-contracts-generation-and-interpretation.md) projects existing native
meaning; [66e](66e-protected-data-and-result-delivery.md) retains selected original results;
[66f](66f-developer-operations-clients-and-evidence.md) supplies client/failure journeys.
Remove a dynamics guard only for its now-supported configuration. Delete replaced
flattened interpretation, duplicated allocation and obsolete fixture routes after
selected callers move; retain scientific negative controls at the surviving owner.

For H03–H06, use [D's shared derivative products](66d-mathematical-representation-and-numerical-execution.md#symbolica-shared-products)
only for the selected cascade stencil and its demanded derivatives. Preserve independent initial
stocks, conservation, original-clock trajectories, and exact carryover at each feed/wash/feed
hold. Keep the strict-zero Radau refusal and the positive-cohort refinement independent:
higher arithmetic precision does not repair a stage-domain conflict. Fixed-time holds do not
qualify state-triggered or periodic transitions. If a selected event derivative changes,
use an independent moving-event control: from `x=0`, evolve `x'=p>0` until `x=1`, reset to zero,
then evolve `x'=2`; at fixed `T>1/p` the sensitivity is `2/p²`. A local post-event derivative
alone misses it. Keep transition and consistency ownership and typed refusal in the native route.

**Planned verification:** native force-validating controls cover nonzero initial clock,
initial algebraic consistency, reordered species/mesh identity, zero/reverse flow,
wrong units/datum, local and independently summed global conservation, reaction-free
and reactive limits, phase changes and unsupported energy/events. Use spatial and
temporal refinement independently, and interval/hold/event counterexamples. Compare
integrated/simultaneous outcomes only within matched formulations. Failed trajectory
members and unsupported response products remain readable with truthful partiality.

Exercise cancellation/observer departure and actual native drain through the selected
66c/66f owner without adding another execution framework. At scope end 66l owns assembled
native/reference trajectories and physical carryover evidence, including finite locations
and output times. Performance/accuracy expansion follows representative consumer
measurements; no numerical or benefit claim is made by this plan-authoring change.
