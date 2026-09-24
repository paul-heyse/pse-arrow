# Process-simulator review additions

**Version 1.0 · 2026-09-24** · What the [process-simulator profile](principles.md) adds
to each slot of the [core review template](../../core/design-review-template.md). The additions
sit within the core slots; no slot is added or removed. Everything here applies only where the
subject touches the behaviour concerned.

| Core slot | Profile addition |
|---|---|
| 1 Scope | Name the analysis modes in scope (square simulation, optimization, dynamics, estimation) and the simulator workloads considered (principles: *Functional target*). |
| 2 Authority map | A **physical-semantics table** (below). |
| 3 Contracts | A **well-posedness statement** (below). |
| 4 Derivation and execution | The **numerical stage columns** (below) for every stage that formulates, evaluates or solves. |
| 5 Journeys | The **simulator journeys** (below), selected by relevance. |
| 6 Gates | Rows PS-G1, PS-G2, PS-G3. |
| 8 Library ledger | Consider derivative, sparse linear algebra, property, structural-analysis and solver capabilities wherever own code performs them. |
| 9 Alternatives | Optional **reference-practice** note: how established simulators handle the same problem, read for behaviour only. |
| 10 Verification | Where relevant, how the touched unit or property models' correctness is established (PS-13), and any reference used. |

## Physical-semantics table (slot 2)

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|

## Well-posedness statement (slot 3)

State how variable roles are declared (PS-04), where degree-of-freedom and structural analysis
run, what is rejected before a solver runs, and how diagnostics name model elements.

## Numerical stage columns (slot 4)

Add to each formulating, evaluating or solving stage:

| Formulation policy (guards, smoothing, complementarity) | Derivative source and order | Scaling | Problem class · solver capability used | Status → outcome mapping | Tolerances (scaled / unscaled) · post-solve check |
|---|---|---|---|---|---|

## Simulator journeys (slot 5)

| Journey | What to trace |
|---|---|
| **Add a unit operation or property model** | Declarations needed; balances from contributions; common checks inherited; derivatives and envelope supplied; places meaning is re-expressed |
| **Edit → re-solve** | A value change and a structural change on one flowsheet: which prepared artifacts survive, which rebuild, and whether the warm start is a recorded dependency |
| **Study over many cases** | Sweep or estimation: prepare once, bind per case, reuse evaluators; how failed points are reported without contaminating the rest |
| **Recycle that will not converge** | Tear selection, convergence policy and history; what the user sees; that the specification is intact afterwards |
| **Infeasible or ill-posed problem** | Structural rejection before solving versus numerical infeasibility after; diagnostics in model terms |
| **Out-of-envelope property evaluation** | An iterate leaves a correlation's range: rejection or selected extrapolation, and how the result records it |
| **Dynamic start and event** | Consistent initialization, index handling, an event or discontinuity, and publication of partial trajectories |
| **Boundary round trip** | Units, basis and identities through authoring, storage and language bridges |

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
| **Reimplemented solver machinery** | Own Newton step, line search, tear convergence or factorization where a qualified solver provides it, with no stated reason | PS-09, DP-13 · G8 |
