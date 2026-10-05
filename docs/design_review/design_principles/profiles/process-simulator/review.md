# Process-simulator review additions

**Version 1.5 · 2026-10-05** · What the [process-simulator profile](principles.md) adds
to each slot of the [core review template](../../core/design-review-template.md). The additions
sit within the core slots; no slot is added or removed. Everything here applies only where the
subject touches the behaviour concerned. Architectural foundations and G9 remain visible;
the numerical detail does not replace decomposition or change-scenario analysis.

| Core slot | Profile addition |
|---|---|
| 1 Scope | Name the analysis modes in scope (square simulation, optimization, dynamics, estimation) and the simulator workloads considered (principles: *Functional target*), including material scale, sparsity/stiffness, case count, concurrent use and resource envelope. |
| 2 Decomposition | Distinguish authored physics, property integration, analysis policy, workflow orchestration and result/persistence responsibilities where they occur. |
| 3 Contracts and authority | A **physical-semantics table** (below), linked to the owned operations for formulation, evaluation and outcome interpretation. |
| 3 Contracts | A **well-posedness statement** (below). |
| 5 Mechanisms and execution | The **numerical stage columns** (below) for every stage that formulates, evaluates or solves. |
| 4 Change scenarios | The **simulator journeys** (below), selected by relevance. |
| 6 Gates | Rows PS-G1, PS-G2, PS-G3; qualitatively assess AP-07 through existing G9 separately from scientific adequacy, under the core's assessment scope. |
| 8 Library fit | Consider derivative, sparse algebra, property, structural-analysis and solver capabilities, their composed preparation/evaluation/solve capabilities, integration owners, optimizer/native batch visibility, movement, state lifetimes and testing/upgrade costs. |
| 9 Alternatives | Optional **reference-practice** note: how established simulators handle the same problem, read for behaviour only. |
| 10 Verification | Where relevant, how the touched unit or property models' correctness is established (PS-13), and any reference used. |

## Physical-semantics table (slot 3)

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|

## Well-posedness statement (slot 3)

State how variable roles are declared (PS-04), where degree-of-freedom and structural analysis
run, what is rejected before a solver runs, and how diagnostics name model elements.

## Numerical stage columns (slot 5)

Add to each formulating, evaluating or solving stage:

| Formulation policy (guards, smoothing, complementarity) | Derivative source and order | Scaling | Problem class · solver capability used | Status → outcome mapping | Tolerances (scaled / unscaled) · post-solve check |
|---|---|---|---|---|---|

For material stages, assess necessary versus repeated assembly/evaluation, sparse and dense
intermediates, language/device/store crossings, and live workspace/factor/trajectory sizes.
Compare a complete native operation with application loops while preserving PS-09's library-owned
iteration, globalization and factors and its bounded scientific composition. Scheduling boundaries
need not mirror unit/model ownership. Assess case/solver thread-pool contention, admitted/queued
work, backpressure and cancellation/drain. Preparation, durable progress and visible scientific
results can have distinct lifetimes; short coherent publication still requires complete outcomes
and an interruption argument. Reuse validity only while unchanged, preserving PS-10's independent
checks of each new numerical state. No mandatory benchmark or new numerical stage column follows.

## Simulator journeys (slot 4)

Classify each change as an instance, binding, composition, policy, domain concept or mechanism.
Assess both model adequacy and authoritative behavior under AP-04/G9 using the relevant
physical definitions, operations and consumers. A new case should reuse its definition;
genuinely new physics may require a contract migration. Investigate only as far as needed
for the scoped judgment; journeys do not require complete traces.

| Journey | What to assess |
|---|---|
| **Add a unit operation or property model** | Declarations needed; balances from contributions; common checks inherited; derivatives and envelope supplied; places meaning is re-expressed |
| **Replace an implementation** | For an existing consumed capability, identify adapter, policy and conformance changes; detect incidental backend details in consumers |
| **Test admission or policy locally** | Required input and capability facts; whether unrelated native or storage startup is necessary |
| **Compose a new analysis workflow** | Model and preparation primitives reused; genuinely new behavior; duplicated end-to-end orchestration |
| **Edit → re-solve** | A value change and a structural change on one flowsheet: which prepared artifacts survive, which rebuild, and whether the warm start is a recorded dependency |
| **Study over many cases** | Sweep or estimation: prepare once, bind per case, reuse evaluators; necessary work as case count grows, live cases/workspaces and nested pools; failed points do not contaminate others |
| **Recycle that will not converge** | Tear selection, convergence policy and history; what the user sees; that the specification is intact afterwards |
| **Infeasible or ill-posed problem** | Structural rejection before solving versus numerical infeasibility after; diagnostics in model terms |
| **Out-of-envelope property evaluation** | An iterate leaves a correlation's range: rejection or selected extrapolation, and how the result records it |
| **Dynamic start and event** | Consistent initialization, index handling, an event or discontinuity, and publication of partial trajectories |
| **Boundary round trip** | Units, basis and identities through authoring, storage and language bridges; selective hydration and compact consumer views without a second authority |
| **Grow or interrupt a workload** | Larger sparse structure, skewed blocks, longer trajectories or concurrent solves; necessary work versus repeated assembly, resource/transaction lifetimes, drain and bounded retry scope |

## Calibration: adequate finding shapes

| Shape | What makes it evidence | Principles · gate |
|---|---|---|
| **Status read as success** | The code that treats a solver return as converged, and a status (iteration limit, acceptable-level stop, restoration exit) that reaches publication as a solution | PS-10 · PS-G3 |
| **Silent finite difference** | A provider or kernel without derivatives, the fallback path that substitutes a difference or a zero, and the "exact" claim it contradicts | PS-07, DP-15 · PS-G3 |
| **Guard lost to simplification** | The authored domain restriction, the rewrite that removed it, and an iterate that evaluates outside the domain | PS-06, DP-08 · PS-G3 |
| **Initialization leak** | A temporary fix or deactivation, and the failure path that returns without restoring it | PS-08, DP-05 · G1, G5 |
| **Basis confusion** | Two interfaces exchanging a flow or property with different bases or reference states and no conversion | PS-01 · PS-G1 |
| **Topology used as solve order** | A stream graph traversed as though it were a dependency order, skipping a cycle or an information link | PS-05, DP-07 · PS-G2 |
| **Structure rebuilt per case** | The per-case path that recompiles or re-derives what depends only on structure | PS-11, DP-10 · G6 |
| **Unfit assembled solve route** | Supported case growth multiplies incidental rebuilds/crossings, live workspaces or nested queues; compare a simpler conforming native composition and distinguish source evidence from measured speed | AP-07, DP-10, DP-20 · G9 |
| **Reimplemented solver machinery** | Own Newton step, line search, tear convergence or factorization where a qualified solver provides it, with no stated reason | PS-09, DP-13 · G8 |
