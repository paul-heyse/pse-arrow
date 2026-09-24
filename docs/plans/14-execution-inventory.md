---
title: Library-owned simulator execution inventory
status: in-progress
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
---

# Plan 14 execution inventory

Only this target owns these requirements. Plan 13 receipts are historical evidence.

| ID | Responsibility | Status | Evidence |
|---|---|---|---|
| M00 | Decisions and scope | complete |  |
| M01 | Library compositions | complete |  |
| M02 | Semantic capability contracts | complete |  |
| M03 | Typed Symbolica compilation and hard cut | complete |  |
| M04 | Indexed structure and binding | complete |  |
| M05 | Guarded values and smoothness | complete |  |
| M06 | Thermodynamic provider | complete |  |
| M07 | Derivative artifacts | complete |  |
| M08 | Sparse problem views | complete |  |
| M09 | Structural analysis | complete |  |
| M10 | Incremental artifacts | complete |  |
| M11 | Ipopt | complete | [execution packet](14-m10-m14-execution.md) |
| M12 | POUNCE | complete | same packet |
| M13 | Initialization | complete | same packet |
| M14 | Linear conic and tears | complete | same packet |
| M15 | Presolve and postsolve | complete | [execution packet](14-m15-m16-execution.md) |
| M16 | Public orchestration | complete | same packet |
| M17 | Dynamics | complete | [execution packet](14-m17-m18-execution.md) |
| M18 | Parameter fitting | complete | same packet |
| M19 | Final cleanup | complete | [execution packet](14-m19-m20-execution.md) |
| M20 | Acceptance inventory | complete | same packet; process tests compiled/discovered, not qualified |
| M21 | Implementation closure | complete | [M21 packet](14-m21-execution.md) |
| M22 | Full qualification | planned |  |

| ID | Displaced mechanism | Status |
|---|---|---|
| X01 | Custom expression graph | complete |
| X02 | Custom differentiation and evaluation | complete |
| X03 | Old math crate identities | complete |
| X04 | Math relation transport | complete |
| X05 | Scalar case preparation | complete |
| X06 | Custom graph/root/presolve algorithms | complete |
| X07 | Production Pyomo | complete |
| X08 | Empty backend routes | complete |
| X09 | Old native callback contracts | complete |
| X10 | MathIR query/cache products | complete |
| X11 | Duplicate math authorities | complete |
| X12 | Historical execution authorization | complete |

## Current implementation boundary

M00–M21 is complete at the implementation scope defined by the execution packets:
[foundation](14-m00-m05-execution.md), [M06–M08](14-m06-m08-execution.md),
[M09–M10](14-m09-m10-execution.md), [M10–M14](14-m10-m14-execution.md),
[M15–M16](14-m15-m16-execution.md), [M17–M18](14-m17-m18-execution.md) and
[M19–M20](14-m19-m20-execution.md) and [M21](14-m21-execution.md).
Legacy deletions remain complete; no prior math or Pyomo implementation is retained
as a fallback. M19 closes empty crates, unused compiled spatial declarations,
duplicate generation and confirmed unused direct dependencies. M20 supplies concrete
native acceptance bodies, independent references and current-profile evidence tools.

The [foundation contract](14-math-foundation-contract.md) records exact consumers,
profiles, owners and limits. The [native solver review](../design_review/reviews/design_review_unified-native-solvers_2026-09-24.md)
assesses the present contract scope. ADR-0082–0084 stay proposed; formal acceptance,
blueprint/index reconciliation and full governance qualification remain open.

Salsa prepares mathematical facts, conditional initialization blocks and complete
physical flow projections. Runtime admission owns Ipopt/POUNCE NLP, KINSOL roots/maps,
HiGHS LP/MILP/convex QP and Clarabel cones/SDP, finite allocation reuse/warm seeds,
physical quality, bounded reports and source-attributed failures. Native uploads,
callbacks, library construction, ABI, graph witnesses and ownership have targeted
unit evidence; actual convergence and scientific acceptance remain M22.

M15 adds native library transformations, independent physical postsolve observations
and qualified dual/warm handling. M16 adds generated source/model/result contracts,
immutable revisions, blocking/async Python jobs, retained Arrow output and explicit
control-last publication. The [workflow guide](../dev/native-workflow.md) states the
public API/profile boundary. Advanced cone/graph construction remains typed Rust
functionality; generated dynamic, fitting and selected provider-factory declarations
are available in Rust/Python. Full process/Python/provider journeys still need M22 evidence.

M17 adds Diffsol BDF on a fixed diag(I,0) profile, consistent starts, finite events,
physical samples and smooth native forward sensitivities. M18 adds simultaneous
steady and single-shooting transient/mixed fitting, original physical results and
qualified local response/rank diagnostics. Both reuse the existing compiler, native
admission, Python job and exact publication owners.

M21 closes policy/envelope/conservation contracts, original structural admission,
scaled response checks and the remaining obsolete surfaces. Its [review](../design_review/reviews/design_review_m21-design-closure_2026-09-24.md)
records library/consumer dispositions. The development collector binds actual unit
observations to source/native identity; it does not qualify complete process cases.

Start next with **M22**: full functional/scientific/native/Python/publication and
repository qualification, then measurements and independent G1–G8 plus PS-G1–PS-G3
reviews. Formal ADR/blueprint reconciliation remains part of that final stage.

Targeted evidence is in the relevant execution packets. The M15–M16 exploratory
Clippy run is not clean; M22 owns whole-plan static/format/governance closure as well as runtime qualification.
The zero-finding target is unchanged.
