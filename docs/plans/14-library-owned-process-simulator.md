---
title: Library-owned process simulator hard pivot
status: in-progress
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
evidence: Implemented — M00-M21 and hard-cut replacements; targeted evidence in execution packets; M22 qualification remains open
---

# Library-owned process simulator hard pivot

## Context

This plan combines the [original library-owned mathematics review][R1] and the
[native-library target follow-up][R2]. It replaces the custom mathematical pipeline
and production Pyomo integration with a small process-modeling core composed with
library-owned mathematics, thermodynamics and native solvers.

**This plan supersedes Plan 13's execution scope in full.** The maintainer's
2026-09-24 clarification governs how the reviews are combined: no unfinished
Plan 13 package, campaign, acceptance ID or historical obligation is inherited as
a matter of course. This also overrides the reviews' suggestions to carry W19/W20
and older acceptance ledgers wholesale. Existing code and tests enter this plan
only through an evidenced connection to the new target. Plan 13 and its receipts
remain historical records; supersession does not turn their incomplete acceptance
into success.

**Current state (2026-09-24):** M00–M21 is implemented under the approved
[foundation](14-m00-m05-execution.md), [M06–M08](14-m06-m08-execution.md),
[M09–M10](14-m09-m10-execution.md), [M10–M14](14-m10-m14-execution.md),
[M15–M16](14-m15-m16-execution.md), [M17–M18](14-m17-m18-execution.md) and
[M19–M20](14-m19-m20-execution.md) and [M21](14-m21-execution.md) packets.
Typed Symbolica/Numerica derivatives, FeOS properties and sparse problem views feed
Ipopt, POUNCE, KINSOL, HiGHS and Clarabel through one admitted Rust solver lifecycle.
Salsa owns immutable facts, conditional block programs and complete physical flow
projections; native models and convergence remain worker-local. Finite sequences,
compatible native reuse/starts, initialization, explicit recycle maps, weighted tears,
semi domains and PSD cones are implemented. The legacy engine/Pyomo routes remain
deleted. The [foundation contract](14-math-foundation-contract.md) and
[scoped review](../design_review/reviews/design_review_unified-native-solvers_2026-09-24.md)
record supported scope. Shared library presolve/postsolve, typed public model revisions,
blocking/async native Python jobs, physical Arrow results and explicit exact Delta
publication are implemented. M17 adds Diffsol BDF for the declared ODE/index-1
profile; M18 adds steady, smooth transient and mixed fitting through the same native
NLP lifecycle. Generated source/result contracts retain physical units and provenance.
M19 removes residual empty crates and unused generated families and compacts physical
fixtures. M20 implements the Q01–Q18 tests, references, profiles and evidence tooling;
the native process bodies are compiled/discovered but unqualified. ADR-0082–0084
remain proposed. M21 closes the remaining source-policy, property-envelope,
conservation, structural-admission and implementation-evidence contracts; its
[bounded review](../design_review/reviews/design_review_m21-design-closure_2026-09-24.md)
records current library/consumer dispositions. M22 is next and alone establishes full
scientific, runtime, performance and whole-plan static qualification.

### Intended outcome and completion boundary

The required target has:

- physically typed, indexed process templates and explicit instance/case bindings;
- Symbolica-owned arithmetic, simplification, differentiation and evaluation;
- one useful, library-owned thermodynamic package, plus a complete extension
  contract for additional property/reaction/transport providers;
- direct native Ipopt C and POUNCE NLP execution, library nonlinear initialization,
  direct HiGHS/Clarabel routes for admitted problem classes, and native tear selection;
- a bounded first native dynamic-simulation profile and a native parameter-fitting
  workflow built from the same physical model and solver capabilities;
- interpretable physical results, errors, scaling, applicable duals and provenance;
- no legacy compiler/evaluator fallback, production Pyomo dependency, custom
  differentiation engine or relational expression-node transport.

The first useful vertical slice is an indexed steady-state heater/flash model with
physical conversions and a real property provider, compiled through Symbolica and
executed through Ipopt. It is an implementation milestone, not the plan's completion
boundary. Completion requires every mandatory M00–M21 package and its paired
deletions, then the single final qualification stage M22. Candidate or explicitly
deferred features do not become supported by implication.

The initial acceptance environment is the repository's Linux CPU development/native
solver environment. Other platforms, distributed execution, GPU math and general
global MINLP are not implied. Preserve useful Python authoring/results convenience;
the production mathematical and solver pipeline is native. IDAES/Pyomo may remain
in a separately isolated parity environment when a selected physical comparison
requires them.

### How the two reviews are reconciled

| Topic | Selected plan direction | Source / disposition |
|---|---|---|
| Algebra and identity | Real-algebra equivalence on the admitted domain; physical typing and obligations precede CAS; identity comes from typed definitions and complete semantic inputs | R1/U1–U2, F4/F11/F12; R2/N01/N05/N06 |
| Compilation granularity | Per-specialization library bodies; instance and case binding stay separate; shape changes may create new bodies | R1/U3/F7; R2/N04 qualifies the original zero-recompile claim |
| NLP | One numerical NLP oracle, direct Ipopt C and POUNCE adapters, exact derivatives where the whole provider chain supports them | R1/U4/F8/F9; R2 §4.4/§4.6 |
| Root solving | A library owns iterations/globalization; KINSOL is the preferred candidate, `diffsol-nl` the bounded pure-Rust alternative | R2/N08 replaces R1's faer-based custom Newton direction |
| LP/MILP/QP/conic | Direct HiGHS and Clarabel adapters for existing coefficient/cone representations | R2/N09 refines R1's mandatory Oximo layer; Oximo/export is conditional |
| Properties | First-class FeOS provider where physically suitable; num-dual/provider derivatives and an explicit fallible multi-output bridge | R2/N03/N10 expands R1's kernel registration sketch |
| Graphs | Reuse semantic projection contracts only when they serve a target consumer; qualify pounce-presolve for matching/DM/BTF; preserve algorithm-specific guarantees | R1 §8.3 plus R2 §4.6; neither retain nor drop rust-igraph merely because a previous plan selected it |
| Presolve | Use library reductions only for supported expressions, declared tolerances and complete postsolve/reuse contracts | R2/N11; a TNLP callback alone does not enable FBBT |
| Dynamics and fitting | Implement one stated dynamic profile and ordinary native parameter fitting; distinguish simulation, transcription, sensitivity and uncertainty claims | R2/N12 and §4.7; broader features remain explicit decisions |
| Legacy plan scope | Re-derive requirements and acceptance from this target; old artifacts are evidence inputs only | Maintainer clarification overrides automatic carry-forward language in R1/R2 |
| License/deployment premise | Preserve R1/U5's already-decided personal-use arrangement; no new licensing/distribution framework | R1/U5; no key material in source, fixtures or logs |
| Optional cleanup | Delete displaced math transport first; adopt other libraries only for surviving demonstrated consumers | R1/F13 and §8.3, narrowed by R2 §8 |

## Decisions

### D00 — Hard replacement, with target-derived scope

Do not preserve `ExprGraph`, both differentiators, scalar instruction evaluation,
Pyomo routing or old artifacts behind compatibility APIs. A new implementation may
exist briefly before its callers move, but never as a permanent parallel production
mode. A package is incomplete until its replacement's targeted tests pass, callers
move, and the displaced mechanism, tests and fixtures are removed. Retain a test's
physical or mathematical obligation only when this target requires it; do not port
tests that merely memorialize a deleted implementation.

Do not restart Plan 13 W19 to establish a prerequisite baseline. Do not copy its
acceptance manifest, pass counts, source seals or feature campaigns into Plan 14.
Historical measurements can be comparison data only when workload, meaning and
measurement conditions still match. Any old failure relevant to a selected new
contract receives a new target-specific regression; its old campaign identity does
not create a requirement.

### D01 — Evidence-based reuse assessment

The following table records pre-cutover candidate evidence, including paths since
deleted. Actual M00 dispositions and surviving consumers are in the
[foundation contract](14-math-foundation-contract.md#evidence-based-reuse-and-deletion).
Each disposition follows: **target
consumer → semantic contract → current source evidence → adaptation/deletion → new
verification**. No consumer means no automatic port. Mechanisms encountered during
implementation receive the same assessment; this is not a complete audit of all
Plan 13 code.

| Historical candidate evidence | New-target reason | Initial disposition and proving task |
|---|---|---|
| `pse-structural/src/projection.rs`: `Scope::{Whole, Independent, Partial}`, semantic edge IDs, `GraphLimits`, duplicate/dangling checks, sorted construction preserving isolates/parallel occurrences | Correct flowsheet topology, containment and complete incidence require explicit source coverage and ID/index maps | Retain useful contracts, adapt layouts to actual consumers; M09 tests scope, multiplicity, ordering and limits |
| `pse-structural/src/incidence.rs`: rust-igraph matching, project alternating reachability/DM/block assembly; algorithm identity names that backend | Initialization and diagnostics need matching/DM/BTF, but generic algorithms should come from libraries | Replace algorithm composition with qualified pounce-presolve functionality where it covers the contract; retain only necessary semantic mapping/certification glue; M09 |
| `pse-compiler/src/case.rs`: incidence consumes `NumericalTopology::derivatives()`; `case/prepare.rs` requires scalarized math | New structure must come from library-derived support and actual bindings, including aliases and indexed models | Replace this source path; it cannot be retained by changing a dependency name; M04/M08/M09 |
| `pse-compiler/src/topology.rs`: complete-node/edge request, heuristic tear groups, Salsa memo; endpoint groups are the output grain | A recycle strategy needs a defined flowsheet projection and declared heuristic/exact tear policy | Reassess edge versus endpoint-group identity, weights and consumer needs; keep useful projection semantics, replace policy/output as needed; M09/M14 |
| `pse-compiler/src/containment.rs` and runtime containment/port-path callers | Indexed authoring and connection binding need ownership/path resolution | Retain only the physical-model semantics actually consumed by M03/M04/M16; no automatic retention of every old traversal/output |
| `pse-compiler/src/incremental.rs`: Salsa DB, tracked definitions and structural queries, cancellation; query variants include `CanonicalGraph`/`NumericalTopology` | Repeated model edits need trustworthy per-body/structure reuse | Reuse suitable database/input-lifecycle primitives; replace MathIR query products, identities and dependencies; M10. Old memo equality or `no_eq` settings are not assumed correct |
| `pse-runtime/src/compiler_driver/session/case.rs`: numerical preparation requests compiled math relations and reconstructs a request; `compiler.rs` exposes old numerical products | Target compilation needs a public orchestration boundary, not a relation round trip | Replace the math-dependent handoff and outputs; M07/M16; retain no transport for compatibility |
| `pse-runtime/src/release_update.rs`: exact endpoint staging; authored release and selected result catalog boundaries described in R1/R2 | Exact authored inputs and coherent native results remain actual product requirements | Reuse only the admission/publication facilities consumed by M16; optional CDF acceleration and maintenance campaigns are not inherited |
| `pse-runtime/src/budget.rs`, `cancel.rs`: shared thread/memory configuration, solver allowance, cancellation token | Library compilation and native work must remain bounded and safely owned | Adapt the smallest useful controls to actual library allocations/worker lifetimes; M10/M16. Existing allowance formulas are not proofs of new native memory bounds |
| Quantity rules, diagnostic/semantic IDs and generated boundary mechanics identified by R1/F12/F13 | Physical meaning, source attribution and public result contracts survive the pivot | Reuse those meanings; rebuild math-specific registries/adapters and remove obsolete generated families; M02/M03/M19 |
| Plan 13 campaigns, seals and Wxx/Lxx completion assertions | No new functional requirement follows merely from their existence | Historical evidence only. M20 defines a fresh capability/acceptance inventory |

Graph selection is particularly narrow: containment, process connectivity, mathematical
incidence, matching-oriented block order and tear selection have different meanings.
Communities, rankings, generic analytics and historical graph output tables are not
required unless a concrete target consumer demonstrates the need. Numeric derivative
support and guard/provider dependency graphs must also remain distinct.

### D02 — Ownership and dependency shape

Use existing owners where their responsibilities still fit. The selected shared math
boundary is a replacement **`pse-math`** crate: library integration and immutable body/
binding contracts shared by compiler and native consumers. It replaces the relevant
roles of `pse-mathir` and `pse-numerics`; it is not a renamed custom IR or a general
numeric framework. M00 recorded the crate change in ADR-0082; the workspace compiles
with the new acyclic boundary. The table describes target responsibilities; later
provider, derivative and solver interfaces are not implied to be implemented.

| Owner | Target responsibility | Boundary |
|---|---|---|
| `pse-authoring`, `pse-quantity` | Authored syntax, binders, domain IDs, physical typing, quantity/basis/reference rules | Physical semantics before algebra; no solver state |
| `pse-compiler` | Typed specialization, domain/smoothness requirements, membership/binding derivation and pure tracked preparation | No live providers, native workspaces or async effects in tracked queries |
| `pse-math` (replacement) | Symbolica bodies and derivative/evaluation specifications; narrow library artifact builders; local support, problem binding contracts and mechanical numerical adapters | No independent opcode interpreter, differentiator, generic sparse algebra or model authority; no Arrow/DataFusion math transport |
| `pse-kernels` | Property-provider contracts and concrete native implementations, including the selected FeOS package | Provider data/phase/derivative semantics explicit; provider-native types remain private; no dependency back into the compiler |
| `pse-structural` | Target-specific graph projection, IDs, library adapters and interpretable analysis results | No dependence on the deleted MathIR representation; no custom matching/DM algorithm where a qualified library supplies it |
| `pse-backend-native`, `pse-ipopt-sys` | Native problem-specific adapters, profiles, callbacks and attempt results | Shared NLP oracle only for NLP; root, conic and DAE interfaces keep their own contracts; native solvers own iteration/factorization |
| `pse-runtime`, useful engine/catalog facilities | Artifact construction scheduling, admission, native worker ownership, actual relational work and publication | Mutable evaluators/property/solver workspaces are attempt/worker-owned; select reused facilities by D01 |
| `pse-model`, `pse-schema`, `pse-relations`, `pse-codegen`, `pse-py`, `python/pse` | Authored and public boundary declarations, generated adapters and thin Python convenience | Generate only required surviving contracts; production Python does not construct or solve Pyomo models |

Do not create one new project crate per third-party library. Separate additional
crates only if a concrete cyclic dependency, linking requirement or API boundary
justifies it and M00's decision is amended. Large native dependencies belong behind
real consumer features, not empty advertised placeholder features.

### D03 — Library and execution decisions

Use the reviews' exact library research as the starting evidence, not a second set
of version declarations. M01 records a resolved feature/profile and locks it through
the normal workspace dependencies. Reference [R2 §4.2 and §9.2][R2] for inspected
candidate versions and known incompatibilities.

| Capability | Selected direction | Must resolve before claiming support |
|---|---|---|
| Symbolic math | Symbolica; local multi-output evaluators, built-in optimization and optional library AD/JIT | Safe lazy guards, output-demand groups, numeric policy, process-global registration, explicit allocator/tracing/JIT features |
| Sparse storage/diagnostics | faer matrices, constructor ordering and library linear algebra | Duplicate accumulation, index order, rank/conditioning and finite residual checks; solver KKT factorization stays native |
| NLP | Existing Ipopt C boundary plus POUNCE, through the same mathematical oracle | Full Lagrangian Hessian, sparse ABI, profile/status/warm-start mapping, panic and error containment |
| Nonlinear roots | KINSOL preferred; qualify `diffsol-nl` as the smaller challenger and select one default | Native ABI/build where applicable, scaling/domain failures, square-system limits and convergence behavior; no custom Newton fallback |
| LP/MILP/QP | Direct HiGHS | Convex continuous-QP restrictions, integrality, coefficient conventions, basis/solution ownership and actual cancellation capability |
| Conic | Direct Clarabel | Explicit supported cone set, matrix conventions, update restrictions and solver-owned linear algebra; no MILP claim |
| Properties | FeOS first where its models/data fit; num-dual or provider derivatives for necessary kernels | FeOS 0.10.1 uses num-dual 0.14, not automatically the proposal's 0.15 trait; isolate types or qualify aligned versions; typed failure and derivative order |
| Structural analysis | petgraph for suited graph operations; qualified pounce-presolve matching/DM/BTF | Full inventory/isolate semantics, index validation, exact versus heuristic output and no accidental edge collapse |
| Numeric presolve | Library reductions/FBBT only for a qualified subset | ExpressionProvider/tape availability, tolerances, invalidation inputs and original-space solution recovery |
| Dynamics | First supported Diffsol native equation/mass-matrix profile | Shape/index/consistent-start/event/sensitivity contract; supply known sparsity instead of trusting control-flow-sensitive discovery |
| Broader implicit DAE | IDA/IDAS candidate if an actual selected dynamic model needs it | Concrete native ABI, residual and index-reduction requirements; not an automatic second dynamics implementation |
| Oximo/export | Optional derived projection with a demonstrated consumer | Must remove actual adapter/transform work and preserve admitted semantics; not a mandatory second modeling layer |

KINSOL/Ipopt/HiGHS and other C/C++/Fortran libraries invoked from Rust are native
execution. All-Rust internals are not a requirement. Conversely, language choice is
not evidence that a solver or binding is mature or correct.

### D04 — Authority amendments and sequencing

The user's pivot direction is settled. Existing architecture text that contradicts
it must change through the decision/design route; it is not a reason to preserve
the old engine. M00 prepares scoped decisions before the governed implementation.
M01's bounded interface experiments may supply evidence while those records remain
proposed. Formal acceptance follows a scoped review resolving the relevant gates.

| Decision scope | Records/text to reconcile, as identified by the reviews | Required result |
|---|---|---|
| Math ownership, equality and crate boundary | Blueprint D6/D10/D11, §6.11, §6.15.4, §7, §14, §18.2 and crate map; ADR-0047, ADR-0037, relevant pending ADR-0054/0068/0076/0078 text; RCA identity wording | Typed physical definitions plus library arithmetic; real-algebra/admissibility policy; no authoritative expression-node relations or ordered-float derivative engine |
| Native backend and Python boundary | D12/ADR-0015, §18–§21 and relevant ADR-0028/0038 text | Native class-specific pipeline; no production Pyomo; first/second derivatives and status semantics explicit |
| Property and dynamics contracts | D9, §9, §13.6 and related kernel decisions including ADR-0022/0043 | Physical provider/branch/data/derivative contract; bounded initial dynamic profile |
| Tear selection | Proposed ADR-0075 and R-32 | Native heuristic/exact selection as justified by target consumers; remove the proposed Pyomo route |
| Reuse, publication and resources | Relevant portions of pending ADR-0076–0081 only where D01 proves their new purpose | Keep useful semantics, replace MathIR-dependent parts, remove unsupported broad preservation mandates |
| Plan/status and public claims | Plan indexes, AGENTS, README/developer docs, generated capability declarations | Plan 14 is the execution direction; old incomplete acceptance stays historical; new claims follow new evidence |

Accepted ADRs are superseded, not rewritten. Proposed records are reconciled in
their decision workflow. Blueprint edits require the designated design amendment
with a revision row. Creating this plan does none of those amendments silently.

## Plan

M00–M05 executes under [its approved detailed packet](14-m00-m05-execution.md);
M06–M08 executes under [the thermodynamics/derivative/assembly packet](14-m06-m08-execution.md).
The maintainer selected a hard removal of old public math/solve routes; necessary
X01–X05 and X07–X11 deletions move into that cut, while replacement solver work
remains in its later packages. See [current inventory](14-execution-inventory.md).

### Execution rhythm and dependency order

M00–M21 are implementation/deletion packages. Their local acceptance is compilation,
targeted unit controls, and required generation when a declaration changes. Use
`just check-package <pkg>` / `just check`, `just unit-package <pkg> <filter>` and
`just codegen` or its appropriate pure-contract recipe. Rust correctness tests retain
explicit `pse-relations/force-validate` through the recipes. Do not run full
integration, component, solver, Python or performance campaigns between packages.
M22 is the single final qualification stage, with functional, measurement and
documentation substeps. Failures there are repaired and affected gates rerun to
zero; they are not baselined away.

M01's small library experiments establish composition contracts, not product
acceptance: compile native interfaces, exercise callback/value/derivative contracts
in isolation, and keep full actual solver/process journeys for M22. No permanent
second backend or bespoke algorithm is introduced just to compare candidates.

| Package | Deliverable | Prerequisites | Initial state |
|---|---|---|---|
| M00 | Decisions, target scope and evidence-based reuse dispositions | This plan and R1/R2 | complete |
| M01 | Resolved library profiles and critical composition controls | M00 | complete |
| M02 | Physical/function/provider/problem contracts | M00, M01 | complete |
| M03 | Typed templates and Symbolica mathematical core | M02 | complete |
| M04 | Finite indices, reductions and instance/case binding | M03 | complete |
| M05 | Safe domains, guards and differentiability admission | M03, M04 | complete |
| M06 | Real thermodynamic provider and fallible derivative bridge | M02, M05 | complete |
| M07 | Library derivative/evaluator artifacts and optional JIT | M03, M05, M06 | complete |
| M08 | Sparse assembly and class-specific problem views | M04, M07 | complete |
| M09 | Target graph projection and library structural analysis | M08 | complete |
| M10 | Semantic reuse and bounded artifact/worker ownership | M07–M09 | complete |
| M11 | Direct Ipopt NLP and faithful callback/results contract | M08, M10 | complete at targeted-unit scope |
| M12 | POUNCE over the same NLP oracle | M11 | complete at targeted-unit scope |
| M13 | Library-owned nonlinear initialization and recycle strategy | M09–M11 | complete at targeted-unit scope |
| M14 | Direct HiGHS/Clarabel and native tear selection | M08, M09, M10 | complete at targeted-unit scope |
| M15 | Qualified presolve, scaling, postsolve and warm starts | M11–M14 | complete |
| M16 | Public native orchestration, Python boundary and selected results | M10–M15 | complete |
| M17 | First native dynamic simulation profile | M06–M10, M13, M16 | complete; [packet](14-m17-m18-execution.md) |
| M18 | Native parameter fitting and scoped sensitivity reporting | M11, M16, M17 for dynamic sensitivities | complete; [packet](14-m17-m18-execution.md) |
| M19 | Final generated/dependency cleanup and optional-leverage dispositions | M03–M18 | implemented; targeted units pass |
| M20 | Target-derived acceptance inventory and tooling | M00 scope; finalize after M19 | implemented; native acceptance compiled/discovered |
| M21 | Implementation/deletion closure and capability review | M19, M20 | complete |
| M22 | Full qualification, measurements and final documentation | M21 | planned |

Dependencies describe required contracts, not a mandate for parallel agent work.
Owners below are responsibility boundaries. Independent details may be developed
concurrently only when shared contracts and file ownership are settled.

### M00 — Decisions, new scope and evidenced reuse

**Owners:** architecture/compiler; plans, decision records, capability declarations
and the scope-selection portions of existing validation tooling.

1. Prepare D04's math, native-backend/Python and provider/dynamics decisions. Record
   U1–U5 and resolve scope-specific review gates; do not turn R2's unresolved gates
   into an all-target acceptance statement.
2. Turn D01 into an implementation inventory of mechanisms actually touched. For
   each reuse, cite its source contract and target caller; identify what must change
   and which new test proves applicability. Unneeded graph algorithms, query
   variants and publications have explicit deletion dispositions.
3. Allocate only this target's implementation, deletion and acceptance identifiers.
   Define a new manifest interface if the existing runner requires one. Update
   active-plan selection without importing Plan 13's `carried` entries, campaign
   requirements or source seal. A Plan 13 seal never authorizes Plan 14 work.
4. Record supported initial process/model classes, provider data requirements,
   deployment profile and explicitly deferred features. Bind exact future ADR IDs
   into this plan when allocated.

**Delete/replace:** active pointers and default runner assumptions that would resume
the old math plan; unsupported blanket Pyomo/NL/all-model capability promises.
Preserve historical documents and receipts with their original outcomes.

**Targeted acceptance:** isolated scope/manifest units reject a borrowed old seal,
an unclassified legacy test mapping and an unsupported advertised route. The scope
review can trace every planned capability to the new target rather than an old ID.

### M01 — Library profiles and critical composition controls

**Owners:** math/provider/native-runtime; workspace dependencies, consumer features,
reproducible small library fixtures and existing native build configuration.

1. Resolve the reviewed candidate versions and feature graph. Keep Symbolica's
   allocator/tracing behavior under host control; record JIT target/flags and
   optimizer thread policy. Do not introduce implicit native-CPU assumptions.
2. Resolve FeOS/num-dual trait compatibility explicitly. Either align a qualified
   version or keep provider types private and exchange numeric/derivative results.
   Do not assume Clarabel's optional faer backend shares the project's faer line.
3. Qualify native ABI/linking/error/index-width assumptions for the chosen Ipopt,
   KINSOL, HiGHS and dynamic profiles. A Rust package version alone does not identify
   a system-linked native library.
4. Establish small controls for lazy guarded library evaluation, fallible property
   bridging, multi-output reuse, exact local second derivatives, evaluator cloning,
   faer duplicate ordering and the repeated-global-variable Hessian case.
5. Compile native adapter signatures and inspect support for stop/status/options.
   KINSOL is provisional default where its contract fits; select `diffsol-nl` only
   for an evidenced advantage or blocking fit/build gap. Record the decision.

**Delete/replace:** incorrect compatibility commentary and no-consumer placeholder
features as concrete consumers land. No production dependency is added solely to
keep every candidate available.

**Targeted acceptance:** locked composition builds and named isolated contract
fixtures, preserving source/lock/profile information. Unsupported branch/JIT/kernel
combinations are explicit refusals. A failed seam chooses another library-supported
composition or narrows a mode; it does not authorize a new custom math engine.

### M02 — Shared semantic and capability contracts

**Owners:** authoring, quantity, compiler, kernels, native backend and schema owners.

Define typed contracts for formal slots and source attribution; structural versus
runtime parameters; finite domains and reduction policies; guarded obligations;
property data, phases and derivative order; local support; instance bindings;
backend profiles; original-model result quality; and resource/attempt identity.
Distinguish value validity from C1/C2 suitability, physical branch selection,
implicit regularity and numerical conditioning. Normalize quantities only after
their basis/reference semantics are established.

Replace the independent operator/lowering lists with one function/provider
registration contract whose generated capability view is enforced at admission.
Library built-ins do not need duplicated project derivative rules. Define distinct
NLE, NLP, linear/quadratic/conic and DAE problem views; only the two NLP backends
share the NLP oracle. Use backend-specific option schemas plus a small shared
attempt policy, rather than a lowest-common-denominator solver configuration.

**Delete/replace:** authoritative DataFusion math opcodes, unsupported backend enum
entries and duplicated derivative/lowering capability declarations.

**Targeted acceptance:** invalid physical signatures, missing derivative order,
unsupported classes and malformed profiles fail before execution; generated
capabilities cannot advertise a registration-only or placeholder implementation.

### M03 — Typed templates and library-owned arithmetic

**Owners:** `pse-authoring`, `pse-quantity`, `pse-compiler`, replacement
`pse-math`; former `pse-mathir` consumers.

Re-host physical inference over authored typed structure, preserving distinctions
such as point subtraction before Symbolica normalizes arithmetic. Build library
atoms through typed APIs, never print/reparse expressions. Keep authored spans,
binders, physical types and domain requirements separate from the library's
arithmetic representation. Construct one body per relevant specialization.

Define body identity from typed definition, structural specialization, provider
registrations and numerical policy. Printed atoms, pointer identity, random
assembly IDs and Symbolica process handles are not durable/reuse identities.
Bound formal-slot and function registration independently of body-cache size.

**Delete/replace:** `ExprGraph`, custom canonicalization/hash-consing/folding,
project arithmetic topology walkers, arithmetic relation loaders and corresponding
ordered-IR APIs as their callers move. Transfer useful quantity tests to the actual
new physical boundary; delete tests whose purpose was the removed ordered engine.
Keep only new library-integration contracts in `pse-math`, not renamed old modules.

**Targeted acceptance:** physically typed algebra cases, quantity/reference errors,
source attribution, normalization and identity controls; unrelated symbol
registration order does not change semantic identity.

### M04 — Indexed model structure and instance/case binding

**Owners:** compiler/templates and runtime authored-input adapter.

Implement finite-domain membership, filters, empty reductions, multiplicity,
ordering, ragged shapes, shared component sets and cross-unit connection equations.
Use reusable bounded scalar/block templates and declared shape specialization;
avoid model-wide expression expansion as an expedient. Bind formal slots to global
variables, parameters and offsets with stable semantic row/variable maps.

Separate ordinary value changes from structural changes such as fixed/free status,
membership, selected phase/provider or an exponent/branch parameter actually consumed
at specialization. Composition-dependent physical conversion remains a nonlinear
model/provider operation, not an affine slot factor. Bound expansion cardinality,
distinct specializations and local body width with explicit refusal.

**Delete/replace:** scalar-only preparation gates and legacy P12/MathIR assumptions;
per-tuple DataFusion planning where the target operation is a direct binding or one
set-oriented operation. Do not retain an expanded legacy representation for fallback.

**Targeted acceptance:** empty/filtered/ragged sums, duplicate binding errors,
cross-unit coupling, fixed/free edits and repeated instances. Adding an already
known shape reuses its body; a new shape rebuilds what it actually changes.

### M05 — Guarded domains and solver smoothness

**Owners:** compiler and `pse-math`; function/provider admission.

Extract admissibility obligations before CAS normalization, including those a
rewrite would erase. Give obligation evaluation its own dependency order so checking
`log(1/x)` cannot divide before validating `x`. Preserve lazy branch semantics
through the enabled library optimization/evaluation modes. Qualify guarded
library-compiled blocks if a monolithic public API cannot meet that contract;
do not recreate a general interpreter.

Distinguish a valid value from a valid smooth neighborhood and available derivative
order. Define variable-guard, phase-boundary, `abs`/`min`/`max`, fractional-power,
non-finite and kernel regularity policies. Explicit approximation profiles are
permitted only with truthful semantics; finite differences or smoothing are never
silent fallbacks.

**Delete/replace:** ordered-float guard/folding machinery and unsupported tests
requiring its operation order. Keep test-only analytic/high-precision references
for selected invariants, not a maintained duplicate production arithmetic engine.

**Targeted acceptance:** `x/x`, `exp(log x)`, `log(1/x)`, invalid untaken branches,
`sqrt(0)` derivative admission, nested guards, NaN and phase-boundary controls.

### M06 — Thermodynamic provider and fallible derivatives

**Owners:** `pse-kernels`, quantity/provider binding, `pse-math` integration.

Implement a concrete FeOS-backed package for selected components/models with
qualified parameter data. Record state variables, units/bases/reference states,
phase selection/stability, valid ranges, supported outputs and derivative orders.
Provider objects and workspaces are local to a valid evaluation context; one
computed state supplies its multiple requested outputs.

Use provider-native derivatives or num-dual for necessary custom kernel bodies.
For implicit properties, expose residual equations or recover derivatives at a
library-solved regular root using qualified library helpers. Check root/conditioning
and singular failure behavior; do not differentiate uncontrolled convergence loops
or copy implicit-differentiation mathematics into another generic engine.

Complete the M01 bridge: Symbolica's infallible scalar callback type does not itself
preserve typed provider errors. Any evaluation-scoped error channel has explicit
ownership and reset behavior; no global mutable side channel. Qualify first/second
derivatives, multi-output reuse, cloning and every enabled JIT/branch mode.

**Delete/replace:** kernel/conversion stubs and unavailable-in-production advertised
bindings; bespoke thermodynamic implementations where this selected library
provider replaces them. No assumption that FeOS covers all reaction/transport models.

**Targeted acceptance:** provider data/units/phase validation, counted state reuse,
analytic derivative cases, singular/unconverged roots and attributable recoverable
versus terminal provider failures. Independent physical end-to-end comparisons are
M22 work.

### M07 — Derivatives, evaluator artifacts and demand

**Owners:** `pse-math` and runtime artifact construction.

Build local residual/objective, Jacobian and Lagrangian-Hessian evaluation products
using Symbolica differentiation and/or its qualified library AD route. Include
objective factor and constraint multipliers in Hessian inputs and numeric cache
identity. Use num-dual/provider derivatives only at the contracted kernel boundary.

Group outputs by actual callback demand; share multi-output work within groups
without computing all second derivatives on every value callback. Use built-in
optimization, merge/vectorization and JIT capabilities when their qualified profiles
apply. Bound derivative expansion, optimization effort, code memory and threads
using library settings and explicit cancellation.

Pure preparation yields immutable specifications/support; effect-owned construction
produces library artifacts. Each worker owns mutable evaluator state and scratch.
JIT is a supported optimization only after it meets the interpreted library path's
semantic contract; no new symbolic-JIT or custom AD subsystem runs beside Symbolica.

**Delete/replace:** both project differentiators, `NumericalTopology`, scalar program/
opcode interpreter, dead DataFusion `Expr` evaluation path and numerical function
identity via UDFs. Remove `pse-numerics` when its last caller moves.

**Targeted acceptance:** analytic derivatives, multiplier/objective-factor changes
at fixed `x`, output-demand counts, optimizer abort, evaluator isolation and
interpreted/JIT equivalence under the declared numerical policy.

### M08 — Sparse assembly and solver-class views

**Owners:** `pse-math`, compiler case binding and native problem adapters.

Specify assembly as local gather/row maps: `z_i = G_i x + B_i p + c_i`,
`J = sum(A_i J_i G_i)`, and `H = sum(G_i^T H_i G_i)` with correctly mapped
Lagrangian weights. Sum duplicate contributions, including repeated formal inputs
bound to the same global variable, before applying the backend's triangular
convention. Preserve nonlinear conversions as equations/provider calls.

Let faer own sparse construction/order. Use its symbolic/numeric constructors and
ordering result where appropriate; maintain only the mechanical stable refill map
needed by callbacks. Derive support across every admitted branch/value of the
compiled shape, never from zeros at one trial point. Guard-only dependencies remain
in invalidation even when absent from the numeric Jacobian.

Derive class-specific views from library mathematics or known typed formulations:
affine coefficients, quadratic terms, explicit cones, NLP and NLE contracts.
Convexity, integrality and DAE suitability require separate declared evidence;
degree two does not establish positive semidefiniteness. Reclassify when an
assumption-sensitive parameter changes.

**Delete/replace:** duplicate sparse patterns and old topology-to-COO builders;
unproven coefficient/zero-elision/classification shortcuts.

**Targeted acceptance:** `a*b` with `a=b=x` has Hessian 2; repeated rows, permutations,
scaling and index overflow; branch support and numeric-zero controls; nonconvex and
unsupported class rejection; no per-callback structure rebuild.

### M09 — Graph scope and library-owned structural analysis

**Implemented and targeted-tested:** see [M09–M10 execution](14-m09-m10-execution.md).

**Owners:** `pse-structural`, compiler structure adapter and initialization consumers.

Apply D01 to each graph use. Retain or simplify semantic projection contracts for
actual containment, connectivity, incidence and block consumers. Build incidence
from M08's complete selected equality/free-variable inventory and support, including
isolates. Never reuse the old `NumericalTopology` incidence source or mistake a
partial inspection for a whole problem.

Qualify pounce-presolve matching/DM/BTF behind checked index maps and result contracts.
Use petgraph for suited SCC/order/traversal operations. Replace custom generic DM/
block logic where the library supplies it; preserve only semantic mapping and
necessary result validation. A perfect structural matching is not numerical rank.

Reassess the rust-igraph dependency after its target consumers are known. Remove it
when all are replaced, not because a historical microbenchmark ranked a matcher.
Keep version/algorithm/projection interpretation in derived analysis provenance.
Heuristic directed feedback-arc results never acquire minimum-cost semantics.

**Delete/replace:** old-math incidence adapters, redundant graph representations,
unconsumed analysis products and superseded matching/DM glue/dependencies.

**Targeted acceptance:** complete/partial scope, omitted coupling, isolates,
parallel edges, invalid indices, alternative valid maximum matchings, invariant DM
partitions, acyclic block order and structural-versus-numerical singularity.

### M10 — Incremental reuse and native artifact lifetime

**Implemented and targeted-tested:** see [M09–M10 execution](14-m09-m10-execution.md).

**Owners:** compiler Salsa integration and runtime scheduling/resource ownership.

Rebuild tracked queries around typed bodies, dependencies, support and binding
structure. Reuse existing input transaction/lifetime controls only after their new
consumers and inputs are explicit. Observe membership, absence, provider/data/phase
changes, structural options, classification assumptions and registry revisions.
Do not memoize attempts, provider convergence state or mutable evaluators.

Use the existing runtime where it fits for artifact single-flight/construction and
worker admission. Key JIT/native artifacts by library/profile/target and semantic
inputs; exclude case values only where truly irrelevant. Bound body caches,
metadata/key generations, global registrations, artifact/code memory, in-flight
work and nested threads separately. An LRU does not bound Symbolica's global state.

Construct `!Send` native objects on their worker. Keep owned buffers, callbacks and
permits until actual native exit, even after cancellation is requested. State which
native allocation limits are enforced and which are admission estimates measured
at qualification. Avoid a new cache service or scheduling framework.

**Delete/replace:** MathIR/numerical query variants, identity-only stale memo keys,
shared mutable evaluator state and obsolete reservation wrappers without consumers.

**Targeted acceptance:** reuse versus clean preparation after each semantic edit;
value-only reuse, generation bounds, compile abort, cancellation before/after worker
entry, concurrent workspace separation and delayed native teardown controls.

### M11 — Direct Ipopt C NLP path

**Implemented and targeted-tested:** [M10–M14 execution](14-m10-m14-execution.md).

The approved [M10–M14 execution packet](14-m10-m14-execution.md) details the shared
solver lifecycle and extensions to M10, native warm/reuse foundations from M15,
semi-variable domains and PSD cones. Its scope governs this implementation sequence.

**Owners:** `pse-backend-native`, `pse-ipopt-sys`, runtime native adapter.

Connect the library-backed NLP oracle to the existing contained C boundary. Preserve
buffer lifetimes and panic barriers while removing dependencies on the old numerical
program and DataFusion math opcodes. Supply sparse structure and exact Lagrangian
Hessian where all providers admit it; limited-memory mode is explicit, not disguised
as exact second derivatives.

Move hard-coded options into a validated backend profile. Record recoverable trial
failures as events and derive terminal outcome from actual native status and final
physical quality. Distinguish optimal, acceptable, feasible-only, limited,
restoration/numerical failure, cancellation and panic. Do not let one failed trial
brand a subsequently converged attempt as an evaluation failure.

**Delete/replace:** old evaluator/problem adapter, limited-memory-only Hessian stub,
latched-error termination override and status collapsing.

**Targeted acceptance:** directly exercised callback ABI/structure/value/Hessian,
null structure-query pointers, index conventions, unwind containment, profile
validation and status/event mapping. Actual native solves belong to M22.

### M12 — POUNCE using the same mathematical oracle

**Implemented and targeted-tested:** [M10–M14 execution](14-m10-m14-execution.md).

**Owners:** native backend and feature/build owners.

Implement a direct POUNCE TNLP adapter preserving M11's mathematical contract, with
its own options, supported features, workspace and result mapping. Do not route this
through Oximo's unrelated derivative/model interface. Preserve POUNCE's native
algorithm and linear algebra; no project KKT solver or emulated Ipopt status.
Keep its thread-local state on the owning worker and expose actual cancellation
and termination limits truthfully.

**Delete/replace:** any attempted second model construction or silent finite-
difference/dense-Jacobian substitution for the shared sparse oracle.

**Targeted acceptance:** adapter callback equivalence on analytic inputs, Hessian
weights, sparse mapping, profile refusals and lifecycle isolation. M22 compares
both solvers against independent mathematics/physical evidence, not just each other.

### M13 — Nonlinear initialization and recycle convergence

**Implemented and targeted-tested:** [M10–M14 execution](14-m10-m14-execution.md).

**Owners:** native root adapter and process initialization strategy.

Use the selected library root solver for square nonlinear systems and recycle
convergence. The library owns Newton/fixed-point updates, globalization and
convergence mechanics. The project selects physical starts, block/tear order,
continuation policy and scales. Connect M09 structural diagnostics and preserve
original equation/variable identities through block solves.

Respect backend limits: KINSOL sign constraints are not general box constraints;
bounded/inequality problems may need the declared NLP route. Do not clamp trial
points or silently convert an infeasible root problem into a successful least-
squares problem. A numerical singularity remains distinct from structural rank.

**Delete/replace:** any existing or newly proposed custom faer-based Newton,
line-search, convergence or recycle fixed-point engine covered by the chosen library.

**Targeted acceptance:** residual/Jacobian/scale mappings, initialization plan order,
capability refusals, failure translation and block recovery. Poor-start/recycle/
singularity convergence journeys run in M22.

### M14 — HiGHS, Clarabel and native tears

**Implemented and targeted-tested:** [M10–M14 execution](14-m10-m14-execution.md).

**Owners:** native optimization adapters and flowsheet tear strategy.

Feed admitted sparse coefficient models directly to HiGHS for LP/MILP/convex
continuous QP. Preserve the quadratic convention, integrality, bounds, native
options, solution/basis distinctions and stopping state. Feed explicit conic
forms to Clarabel, including only qualified cone types and feature profiles.
Respect its matrix-update restrictions instead of promising universal warm reuse.

Implement a native exact tear-selection MILP when the selected process strategy
requires minimum-cost tears. Define graph projection, occurrence/group grain,
weights, cycle-breaking witness, incumbent, gap and optimality status. A heuristic
route remains explicitly heuristic. No pinned graph routine is assumed to provide
an exact minimum directed feedback arc set.

**Delete/replace:** production Pyomo tear/model construction and mandatory Oximo
matrix-to-model rebuilding. Remove unsupported Clarabel MILP/HiGHS MIQP or nonconvex
QP claims; do not retain empty route variants.

**Targeted acceptance:** coefficient/cone/ordering transformations, unsupported-class
refusals, parameter-sensitive convexity invalidation and tiny exhaustive tear
oracles. Solver-class-specific execution cases are part of M22.

### M15 — Presolve, scaling, postsolve and warm starts

**Implemented:** the [M15–M16 packet](14-m15-m16-execution.md) supplies the shared
library TNLP pipeline, per-row Symbolica proofs/FBBT tapes, native option admission,
qualified automatic passes, scaling and library recovery. Fresh original-model
observations and complete transformation identities qualify duals and starts.
The following obligations describe the implemented package; whole-model numerical
qualification remains M22.

**Owners:** native backend/problem preparation and result interpretation.

Use qualified library presolve where it reduces project code and preserves the
selected problem's meaning. POUNCE FBBT requires a supported expression-provider
tape; build a loss-aware derived view for that subset or report the pass disabled.
Do not silently advertise a pass that received only numerical callbacks.

Record coefficient/equality tolerances, expression-only invalidation, applied
reductions and the distinction between raw contradiction detection and a valid
infeasibility certificate. Use library primal/multiplier recovery where available;
always verify the original model after postsolve. Every scale/permutation/fixed-
variable/reduction transformation has an explicit inverse or a typed unsupported
dual-reporting result.

Warm starts include compatible structure, scale, backend and primal/dual/basis
identity. A reused body does not prove factorization, basis or multiplier reuse.
No project-wide custom presolver, interval engine or independent transformation IR
is introduced to fill library gaps.

**Delete/replace:** ad hoc reduction/recovery and stale cache assumptions covered
by the library contracts; reduced-space result publication as physical results.

**Targeted acceptance:** near-zero coefficients, changed expression tapes,
unsupported operations, affine elimination/recovery, scaled duals, stale warm
starts and original-model residual checks on small analytic cases.

### M16 — Public native workflow and publication

**Implemented:** see the [execution packet](14-m15-m16-execution.md) and
[workflow guide](../dev/native-workflow.md) for exact API, profile and test boundaries.

**Owners:** runtime, needed engine/catalog facilities, `pse-py`, `python/pse`, boundary
schema/generator owners.

Replace the math-dependent compile/case handoff with typed preparation, artifact
construction and native attempts. Reuse exact authored admission and selected
publication only where D01 establishes their new role. Connect the public Rust and
thin Python APIs; no Python numerical kernel, Pyomo construction or hidden backend
fallback is required to initialize or solve.

Publish original physical variables, evaluated constraints/bounds, violations,
applicable duals, status, provider/profile/source provenance and useful diagnostics.
Raw `g(x)` is not a residual. Canonical library text is inspection material, not a
durable restore format. Reject incompatible old compiled artifacts and rebuild
from admitted definitions; do not implement legacy readers or migrations.

Keep explicit coherent result publication and typed failed/limited/cancelled
outcomes. Do not inherit every intermediate table, catalog maintenance workflow or
old campaign solely because it once existed.

**Delete/replace:** `pse-backend-pyomo`, `python/pse/adapters/pyomo`, production Pyomo
dependencies and Pyomo-only transitive requirements without surviving consumers;
`pse-backend-nl` stub unless a real export consumer is scheduled. Remove old math
transport/query output variants and regenerate required public boundary contracts.

**Targeted acceptance:** public API/profile/result construction, dependency/import
boundary checks, publication-state/result-mapping controls and incompatible-artifact
refusal. Actual install/compile/solve/publish/reopen journeys are M22.

### M17 — Native dynamic simulation

**Implemented:** see the [M17–M18 packet](14-m17-m18-execution.md) for the admitted
fixed-mass profile, source/compiler/runtime boundaries and targeted evidence.

**Owners:** compiler dynamic formulation, native integrator and physical model owners.

Implement one explicit ODE/index-1 mass-matrix process profile using Diffsol's native
equation interfaces and the shared mathematical products. Let the library own time
stepping, local error control, events and relevant sensitivity mechanics. Supply
known support and derivatives rather than treating NaN-based discovery as complete
behind control flow. Define state/algebraic partitions, mass matrix dependence,
consistent starts, time-dependent inputs, event ordering/reset and result sampling.

Select IDA instead, or add a separately justified profile, only if a chosen physical
model needs the broader implicit residual form and the integration cost is justified.
No automatic dual integrator stack and no claim of general high-index support.
Spatial/discrete stencils reuse M04 where actually required. Simultaneous dynamic
optimization needs its own transcription and is not implied by a time integrator.

**Delete/replace:** dynamic placeholder claims, Pyomo DAE/runtime routes and any
custom stepper covered by the selected library. Do not add DiffSL as a second
authoring/derivative pipeline without a separate demonstrated need.

**Targeted acceptance:** derivative/mass/support mapping, consistent-start contract,
event/reset construction, unsupported-DAE refusal and snapshot identity. Actual
dynamic vessel conservation, error and event journeys run in M22.

### M18 — Native fitting and truthful sensitivity scope

**Implemented:** see the [M17–M18 packet](14-m17-m18-execution.md) for native steady,
smooth transient and mixed experiments, library derivatives/factors and local-rank limits.

**Owners:** physical workflow/model library and native result interpretation.

Provide one ordinary parameter-estimation workflow as a typed physical objective
and constraints over observations, using the existing native NLP pipeline. Keep
data/units/weights, free/fixed parameter identity and objective convention explicit;
do not recreate a general modeling/optimization package to replace `parmest`.

Use qualified library/provider derivatives and the selected integrator's sensitivity
facilities for scoped reports. State the conditions for sensitivity and statistical
interpretation; convergence is not identifiability, and a derivative is not a
confidence interval. Broader covariance/uncertainty, robust optimization, GDP and
global MINLP belong to the explicit deferred ledger below unless separately selected.

**Delete/replace:** production Python estimation routes and unimplemented broad
feature promises. Preserve necessary ordinary physical workflow logic, not Pyomo
object construction or private-data-model adapters.

**Targeted acceptance:** synthetic objective/derivative truth, observation unit and
weight validation, fixed/free parameter behavior and unsupported report refusal.
M22 fits a known physical/synthetic parameter case and checks the scoped sensitivity
against an independent oracle.

### M19 — Generated families and final dependency cleanup

**Owners:** schema/codegen, surviving semantic/columnar boundaries, workspace and
Python dependency owners.

Close residual deletion work after each earlier package has removed its own replaced
mechanism. Remove normalized/inferred/compiled expression-node relation families,
mathir sink/source/value generators and their produced contracts where no target
boundary needs them. Keep authored definitions and selected compiled/result tables.
Fix generators and regenerate; never edit generated trees manually.

Compact quantity fixture generation without changing the physical cases it proves.
Remove remaining `pse-mathir`/`pse-numerics` crate registrations, obsolete transport
dependencies, unused native feature edges and Python-only dependencies without target
consumers. Inspect normal/transitive dependency closure rather than optional flags
alone. Use the one-universe check when the pinned Arrow-family graph changes.

After deletion, disposition R1's `serde_arrow`, Arrow pool and `strum` suggestions
against the remaining work: adopt an existing-library mechanism only if a concrete
consumer and contract justify it; otherwise defer explicitly. No broad codec or
allocation-framework rewrite is a prerequisite for this pivot.

**Targeted acceptance:** pure generation/registry controls, concrete consumer
compilation, no obsolete declarations/exports and compact fixture equivalence.
Full quality and generation gates wait for M22.

### M20 — Fresh acceptance inventory and qualification tooling

**Owners:** test/fixture and existing validation-tooling owners.

Maintain `14-acceptance-cases.toml` and the minimum necessary inventory/receipt support
from the Qxx requirements below. M20's implementation is recorded in its execution
packet; declarations and discovery are not qualification. Map each admitted capability to actual tests and
conditions; identify deliberate unsupported/refusal cases. Extend existing recipes
for Plan 14 only where needed; avoid a new generic evidence platform.

For any reused test, record the new requirement it proves and why its oracle still
applies. Never import an old test or acceptance ID simply to preserve counts or
ancestry. Replace old-mechanism expectations with target-specific physical/semantic
oracles, or delete the test. Historical campaign manifests stay historical and do
not constrain this manifest.

**Delete/replace:** active default references to old manifests and unsupported
placeholder acceptance; no stale source seal or previous passing receipt may
satisfy a new target case.

**Targeted acceptance:** isolated manifest/tool units enforce implemented-versus-
qualified status, named source/test ownership, unsupported-class refusals and
source/profile identity. Listing the scope performs no product execution.

### M21 — Implementation and deletion closure

**Owners:** architecture/integration; all package owners close their own rows.

Review the actual target against R1/R2 and the new capability surface. Close all
mandatory implementation and deletion rows, including no remaining old production
route, math relation transport, Python numerical dependency or silent unsupported
feature. Recheck every retained Plan 13 mechanism against its D01 rationale and
current caller. Delete orphaned remnants rather than keeping them “for evidence.”

Ensure the new acceptance inventory is executable and complete for the selected
target. Any source manifest/barrier records this implementation's scope; it is not
a quality result and must not inherit a Plan 13 seal. Keep source sealing minimal
and reuse applicable tooling instead of building a new governance subsystem.

**Targeted acceptance:** code/consumer/manifest inspection and isolated controls
show the final paths and deletion closure. Product acceptance remains not_run until
M22; a narrow fixture or source seal cannot turn it green.

### M22 — One final qualification stage

**Owners:** integration/physical validation, native runtime and documentation.

Run the full Qxx matrix and all applicable repository gates for this target once
M21 closes. Within this stage, finish functional/physical correctness first, then
measure cold/warm performance and resource use, then finalize independent G1–G8 and PS-G1–PS-G3
verdicts and documentation. Repair failures and rerun affected gates until the
required zero baseline is met. Do not lower scope merely because a case fails.

Qualify the public native workflow, indexed/thermodynamic/recycle process model,
both NLP backends, selected root solver, actual linear/conic classes, dynamic
profile and fitting workflow. Include current authored-input/result publication
only as used by the new target. Old Plan 13 campaigns remain unrun/incomplete as
historical records; their status is not the completion condition here.

Update current architecture/public capability documents, usage examples, ADR/design
status, plan index and AGENTS from actual results. Record what remains unsupported
or deferred. Complete Outcome with evidence labels, conditions, commands, zero-
baseline counts, a real corrected mistake and deliberate deviations. No integrated
speedup, universal model coverage or numerical IDAES equivalence is claimed without
the corresponding evidence.

## Verification

### Required target cases

**Implemented, unqualified.** These are new Plan 14 obligations, not renamed inherited cases.
M20 maps each to actual tests/fixtures/recipes; no integrated row is Tested by this document.
M22 runs the integrated matrix in the declared feature/native build modes.

| ID | Claim and required conditions | Independent oracle / expected failure controls | Owner |
|---|---|---|---|
| Q01 | Physically typed indexed definitions, reductions, bindings and conversions preserve authored meaning | Analytic finite sums; empty/ragged/filter and shared-component cases; quantity/basis/reference errors | M03/M04 |
| Q02 | Real-algebra normalization preserves admitted meaning and unsafe branches stay lazy | Domain-erasing rewrites, nested/untaken invalid operations, high-precision difficult values; interpreted and each enabled JIT/vector mode | M05/M07 |
| Q03 | Local and global first/second derivatives are correct | Exact polynomials, `a=b` Hessian=2, repeated rows, scaling/permutations, directional checks with several step sizes; changed multipliers at fixed `x` | M07/M08 |
| Q04 | A real property provider preserves physical meaning and derivative/error behavior | Independent property references, stable phase selection, counted multi-output state reuse, singular/unconverged state and first/second derivative controls | M06 |
| Q05 | Structural results describe the complete admitted mathematical problem | Omitted cross-coupling, isolates, invalid indices, repeated edges, independent tiny matching/DM oracles; full matching with numerically singular Jacobian | M09 |
| Q06 | Reuse equals clean preparation for the same target compiler | Values, membership/absence, shape, fixedness, provider/data/phase, numerical policy and target/profile edits; bounded body rebuilds | M10 |
| Q07 | Native NLP callbacks and outcomes remain faithful through both backends | Analytic NLP plus physical case; recoverable trial then convergence, feasible-only/limited exit, Hessian modes, callback panic and terminal failure | M11/M12 |
| Q08 | Library initialization solves the declared process problem | Scaled recycle from poor starts, domain-rejected trials, singular block and constrained-root refusal; original physical residuals | M13 |
| Q09 | Linear/quadratic/conic capability routing is truthful | LP/MILP/convex-QP on eligible routes; declared Clarabel cones; negative MIQP/nonconvex/opaque-conic and Clarabel-MILP cases | M14 |
| Q10 | Native tear selection preserves graph policy and solver assurance | Tiny exhaustive weighted cases and acyclicity witnesses; heuristic versus optimal/limited status; parallel-connection grain | M09/M14 |
| Q11 | Presolve/scaling/warm starts preserve original-model meaning | Unsupported tape, expression-only edit, near-zero coefficients, eliminated variables, dual/scaling recovery and incompatible-start refusal | M15 |
| Q12 | Public Rust/Python operation is genuinely native | Install/import without production Pyomo; authored indexed heater/flash/separation with conversion, recycle, initialization, optimization and physical result access | M16 |
| Q13 | Authored input and selected native results remain coherent | Exact selected source, failed/limited/cancelled attempts, interrupted publication and reopen; reject incompatible compiled artifacts | M16 |
| Q14 | The first dynamic profile has correct physical/time semantics | Dynamic vessel with algebraic constraints, consistent start, valve event/reset, balance/error checks and unsupported higher-index refusal | M17 |
| Q15 | Native fitting and advertised sensitivities mean what they claim | Known-parameter case, observation units/weights, non-identifiability distinction, independent derivative/sensitivity reference | M18 |
| Q16 | Resource/cancellation/lifetime limits survive actual library work | Compile abort, callback stop, slow factorization/teardown, concurrent attempts, global registration bounds, nested thread budget and retained buffers | M10/M16 |
| Q17 | Replaced production mechanisms and obsolete claims are absent | Caller/dependency/export/generated-family inventory; no legacy math/Pyomo fallback or dormant advertised feature | M19/M21 |
| Q18 | End-to-end benefit and final design claims have evidence | Cold/warm complete process cost, representative shape diversity, measured peak memory, reproducibility profile and independent G1–G8 and PS-G1–PS-G3 assessment | M22 |

Two solvers sharing a compiler are not independent mathematical oracles. Use analytic
models, conservation identities, independently sourced physical data and appropriate
high-precision/directional checks. Do not define correctness as agreement with the
deleted engine. R1's scratch probes and R2's interface evidence identify hypotheses
and controls; they are not freshly executed receipts for this target.

### Mandatory deletion ledger

The ledger is target-derived. No Plan 13 Lxx row is carried automatically. Delete
paired mechanisms as soon as their replacement is proven and callers have moved;
M19/M21 catch residuals rather than postponing all deletion until the end.

| ID | Remove or replace | Replacement owner | Closure condition |
|---|---|---|---|
| X01 | `ExprGraph`, custom ordered canonicalization/hash/fold/topology and relation loaders | M03/M05 | Physically typed definitions lower directly to Symbolica; no old arithmetic graph dependency |
| X02 | Both bespoke derivative engines, scalar interpreter, `NumericalTopology`, dead DataFusion evaluation path, UDF/opcode math binding | M07/M08 | Only qualified library derivative/evaluator paths serve native callbacks |
| X03 | `pse-mathir` and `pse-numerics` crate/API identities after their roles move | M03/M07/M19 | `pse-math` is a narrow replacement boundary; no old crate exports or shim |
| X04 | Intermediate math relation families, sink/source/value generators, generated adapters and unused output tables | M03/M16/M19 | No expression-node encode/decode between preparation stages; surviving boundary contracts regenerated |
| X05 | Scalar-only case preparation, old topology incidence source, per-tuple math/binding query execution | M04/M08/M09/M16 | Indexed bindings and library support drive preparation/analysis directly |
| X06 | Generic custom matching/DM/block/root/presolve algorithms replaced by selected libraries; unnecessary graph dependencies | M09/M13/M15 | Actual target callers use qualified library algorithms; remaining glue has a semantic mapping purpose |
| X07 | `pse-backend-pyomo`, Python Pyomo adapter and production-only dependency chain; Pyomo tear/estimation routes | M14/M16/M18 | Public native workflow installs/runs without Pyomo; parity environment isolated |
| X08 | Empty `pse-backend-nl`/dynamic/backend feature routes and unsupported capability promises | M02/M16/M17/M19 | An advertised feature has a concrete qualified path, or is explicitly unsupported; export only for a chosen consumer |
| X09 | Old numerical callback/status/profile mapping, stale trial-failure latch, reduced-space result publication | M11/M12/M15/M16 | Faithful native termination and original physical values/quality/duals |
| X10 | MathIR-specific Salsa queries, incomplete reuse keys, shared mutable evaluator state and obsolete cache/resource wrappers | M10 | New semantic dependencies and owned artifact/attempt lifetimes are the only route |
| X11 | Duplicated operator/derivative/lowering capability authorities and generated code without a surviving contract | M02/M19 | One declaration per surviving meaning; no manufactured feature support |
| X12 | Active Plan 13 runner/seal/acceptance assumptions and old-mechanism fixtures/tests | M00/M20/M21 | New target inventory and verification govern; historical records remain historical |

### Qualification commands and evidence conditions

The existing recipe surface supplies compilation, selected Rust units, generation,
native solver execution, Python tests and repository quality. M00/M20 must wire
Plan 14 scope selection before any end-to-end runner is used. A future invocation
such as `just assessment <new-output> --plan 14` is **planned**, not claimed to work
today. `just assessment-list --plan 14` must list the new target cases without
executing them once that support exists.

At M22, use the recipes appropriate to the actual implemented consumers: Rust
tests/doctests in required profiles, native solver and compiler/solver tests, fresh
editable Python build and selected unit/component/integration groups, declared
physical/parity comparisons, feature combinations for the admitted native profiles,
and the Qxx process/measurement cases. Tests use explicit force-validation. A
production performance run without force-validation has separate feature/build
identity and is not a correctness test.

The closure quality surface includes formatting/lint, Python `quality`, governance,
dependency-family consistency, complete generation checks, ADR/design/index checks,
agent configuration and documentation. Revise obsolete checks that assert the
deleted architecture; do not suppress a real surviving invariant. Run these at plan
close, not after each document edit. Distribution builds, cloud qualification and
new platforms are not automatic prerequisites for this local simulator target.

Record commands, modes, native library/profile versions, source identity, physical
inputs, seed/thread policy and result conditions. Keep pass/fail, unsupported,
cancelled/interrupted, advisory and not_run outcomes distinct. The failure baseline
is zero; a warning or failed required test remains open until repaired. Avoid
per-command receipt logs in this plan; detailed evidence belongs with the relevant
test/campaign artifacts, while checkpoints record state and next steps.

### Cost and design acceptance

Measure template typing/library preparation, optimization/JIT, binding/assembly,
graph analysis, native conversions, property-state work, initialization, callback
groups, solver iterations, original-model validation and publication separately and
end to end. Include cold and warm runs, repeated instances and genuinely new shapes,
ordinary and difficult starting points, cancellation and peak memory. Report build/
dependency cost separately from runtime cost. Historical data is comparable only
where the workload and numerical/feature conditions actually match.

| Gate | Required target evidence at M22 |
|---|---|
| G1 — Authority | Physical meaning has one authored owner; Symbolica/providers own derived mathematics; no competing old engines or model layers |
| G2 — Fidelity | Guarded domains, alias assembly, property derivatives, scaling and original-model results preserve declared meaning |
| G3 — Validity | Physical domain, smoothness, regularity, convexity and problem-class admission reject incompatible requests before execution |
| G4 — Effects | Registration/target inputs, compilation artifacts, provider effects, native workspaces and nondeterminism are explicit |
| G5 — Recovery | Trial versus terminal failure, cancellation/teardown, postsolve and coherent publication have tested outcomes |
| G6 — Reuse | Clean versus reused results agree across all relevant structural/runtime/provider edits, within declared numerical equivalence |
| G7 — Claims | Every advertised route maps to a qualified case; candidate/deferred/unsupported behavior remains visible |
| G8 — Library leverage | Generic capabilities use applicable library built-ins; remaining domain adapters have an evidenced purpose |
| PS-G1 — Physical consistency | Declared property windows are enforced; quantity/basis/reference conventions and independent conservation close |
| PS-G2 — Well-posedness | Complete original structure and analysis-mode admission reject ill-posed cases with model identities |
| PS-G3 — Numerical integrity | Derivative, domain, scaling, convergence, tolerance and result claims agree with actual native behavior |

No gate is offset by performance, code deletion or another passing gate. Do not mark
the plan done while a mandatory gate is unresolved. Narrower implementation
milestones can be described truthfully without representing this whole plan as
complete.

### Review-to-work traceability

The complete finding sets are accounted for here; the plan deliberately refines
proposals where R2 or the maintainer's scope clarification supersedes R1.

| Review finding | Plan disposition |
|---|---|
| R1/F1 — unsupported indexed/kernel/conversion/Hessian/model routes | M02–M08, M11/M16/M17; Q01–Q04/Q07/Q12/Q14; claims match selected target rather than all historical blueprint functions |
| R1/F2 — duplicate divergent derivatives | M07; X02; Q03 |
| R1/F3 — relational math transport/per-tuple work | M03/M04/M16/M19; X04/X05; Q18 measures complete effect |
| R1/F4 — ordered-float contract blocks CAS | M00/M03/M05; D04; Q02 |
| R1/F5 — dead DataFusion math/bespoke interpreter | M07/M19; X02/X03 |
| R1/F6 — recovered trial becomes terminal failure | M11/M12/M16; Q07 |
| R1/F7 — whole-problem numerical preparation | M03/M04/M07/M10; Q06; shape changes explicitly qualify reuse |
| R1/F8 — solver options/status/Hessian contract | M02/M11/M12/M15; Q07/Q11 |
| R1/F9 — Oximo loses required derivative contract | M11/M12/M14; direct adapters; optional export disposition below; no Clarabel MILP parity claim |
| R1/F10 — Pyomo/NL architecture and dependency claims | M00/M14/M16/M18/M19; X07/X08 |
| R1/F11 — global symbols/printed ordering/identity | M03/M07/M10; Q02/Q06/Q16 |
| R1/F12 — physical typing must precede normalization | M02/M03/M06; Q01/Q04 |
| R1/F13 — generated volume/duplicate mechanics | M19 after math deletion; optional codec/macro choices below |
| R1/F14 — Symbolica process-wide defaults/JIT flags | M01/M07/M10; Q16 |
| R1/F15 — faer singularity/determinism limits | M08/M09/M13; native solver-owned factors; Q05/Q08 |
| R2/N01 — obligation schedule | M05/M07; Q02 |
| R2/N02 — aliasing/symmetry | M08; Q03 |
| R2/N03 — incomplete property integration | M01/M02/M06/M07; Q04 |
| R2/N04 — bindings are not index semantics | M04; Q01/Q06 |
| R2/N05 — validity versus solver suitability | M02/M05/M08; Q02/Q04/Q09 |
| R2/N06 — artifact/reuse lifetime | M07/M10; Q06/Q16 |
| R2/N07 — inverse transformations/result meaning | M11/M15/M16; Q07/Q11/Q13 |
| R2/N08 — faer leaves custom nonlinear algorithm | M01/M13; selected library owns iterations |
| R2/N09 — mandatory Oximo layer | M14; conditional adapter/export only |
| R2/N10 — incompatible candidate versions/features | M01; exact locked consumer profile |
| R2/N11 — presolve input/postsolve requirements | M15; Q11 |
| R2/N12 — broader native functionality | M17/M18 and explicit deferred capability ledger; no automatic import of the whole old simulator backlog |
| R2/N13 — unsupported/all-gates-pass claims | M00/M02/M20/M22; independent gate table above |
| R2/N14 — independent oracles/end-to-end costs | Q01–Q18; M20/M22 |
| R2/N15 — hard deletion of competing ownership | D00 and X01–X12; M21 |

## Open items

### Bounded decisions within the mandatory plan

| Decision | Starting position | Owner / deciding package | Failure response |
|---|---|---|---|
| Precise property bridge and JIT eligibility | Library arithmetic plus one real fallible multi-output provider | Math/provider, M01/M06 | Qualify guarded library-block composition or restrict the unsupported execution mode; do not resurrect an interpreter |
| FeOS/num-dual feature/version combination | Use exact reviewed candidates as evidence; isolate provider types where sensible | Dependency/provider, M01 | Deliberate aligned pin or numeric boundary; no assumed trait compatibility |
| Root solver and native build | KINSOL provisional; bounded `diffsol-nl` comparison where justified | Native runtime, M01/M13 | Choose the library that meets the declared profile; no custom Newton fallback |
| Structural algorithm composition | pounce-presolve matching/DM/BTF with semantic guards, petgraph suited operations | Structural, M09 | Use another qualified existing library for a demonstrated gap; justify any retained rust-igraph consumer |
| Numeric presolve subset | Only supported expressions/policies with tested original-space recovery | Native runtime, M15 | Pass is explicitly disabled/refused for unsupported cases; no silent no-op claim |
| Dynamic profile | One Diffsol-compatible physical model first | Dynamics, M17 | Select IDA if the actual required formulation needs it; explicitly narrow unsupported high-index/discrete scope |
| Retained infrastructure | Only D01 consumers/contracts supported by current source evidence | Owning package, M00 onward | Adapt, replace or delete; Plan 13 provenance is not a retention argument |

### Explicitly conditional or deferred recommendations

Every item mentioned by the reviews receives a disposition. Deferral is not a
production stub or a requirement to implement it before M22. If an item becomes
necessary for a mandatory capability, promote it into an owned package and its
acceptance before claiming completion.

| Item | Disposition / reason | Revisit trigger and owner |
|---|---|---|
| Oximo model transforms and ASCII NL/LP/MPS export | Optional derived consumer; no native solver depends on this path | A named external consumer requires an export/transform and a loss-aware adapter demonstrably reduces work; optimization owner qualifies it |
| General implicit/high-index DAE, a second integrator, dynamic-optimization transcription | Beyond the first stated M17 profile; a time integrator is not transcription/index reduction | A selected physical model or optimization workflow requires it; dynamics owner supplies formulation and acceptance |
| SCIP/russcip/scip-sys, GDP/global MINLP | Candidate native extension, not a foundational dependency or promise of global optimization from black-box callbacks | A concrete discrete nonlinear process problem with structural expression/bound information; optimization owner researches/qualifies exact APIs |
| Robust optimization/PyROS replacement, broad covariance/uncertainty toolkit | No automatic replacement of every historical Pyomo utility | A specified physical/statistical product requirement beyond M18; model/statistics owner defines meaning and library route |
| Further property/reaction/transport libraries | FeOS is not a complete process-property catalog | Selected components/physics fall outside the initial provider; physical-model owner qualifies the next library rather than porting a full catalog preemptively |
| `serde_arrow` row codecs | Reassess after removing math transport; no general codec rewrite bundled by default | Measured surviving duplication or conversion cost with registry schemas/metadata preserved; columnar owner |
| Arrow pool / DataFusion `ArrowMemoryPool` integration | Reassess concrete ownership after native math removal; no speculative accounting replacement | Measured escaping-buffer or allocation-accounting need in a retained boundary; resource owner qualifies exact pinned built-ins |
| `strum` for remaining closed enums | Small M19 choice only where a surviving enum has repeated mechanics | Concrete remaining generator/macro duplication; schema owner |
| egglog / separate symjit / another algebra IR | Not selected; Symbolica owns arithmetic | A new physical transformation or changed library requirement cannot be met by the selected engine; architecture review before adoption |
| Plan 13 optional CDF, graph analytics, catalog maintenance and broad campaigns | No automatic scope; only actual new target consumers can justify them | D01 evidence of a specific required operation; corresponding owner |
| New platforms, GPU or distributed solves | Outside the initial qualified deployment | Explicit deployment/workload requirement; native runtime owner |

## Implementation outcome — M00–M05

**Implemented:** direct physically typed Symbolica preparation, explicit finite and
case bindings, guarded value evaluation, executable provider contracts, class-specific
native problem interfaces and locked composition profiles. Required hard-cut deletions
were pulled forward and completed; the inventory distinguishes deletion completion
from later replacement capabilities.

**Tested:** the [execution packet](14-m00-m05-execution.md#verification) records the
focused commands, counts, conditions and zero failure baseline. Workspace compilation
and required contract/fixture regeneration succeed. No full simulator, Python,
performance or M22 qualification claim is made.

**Mistake corrected:** checking domains only after constructing an Atom could eagerly
normalize an invalid constant in an untaken branch. Materializing the original operand
and guarding it before substitution fixes that ordering; the short-circuit regression
covers `log(-1)` in an untaken branch. Formal unit admission was also tightened so a
body cannot receive noncanonical coordinates under an otherwise compatible type.

**Deliberate deviations:** remove old public math/solve routes and unsupported generated
outputs immediately, rather than retaining the later deletion sequencing. Use scalar
interpreted guarded library blocks; JIT/SIMD and full derivative execution remain
unavailable. Only package/rule graph consumers survive this cut. Initial local power
and construction limits refuse work beyond the selected bounded profile.

**Remaining after M00–M05 (historical checkpoint):** M06 production thermodynamics, M07 derivatives, M08 sparse assembly,
M09 structure, M10 incremental artifacts, M11–M18 native workflows, M19 final target
cleanup, M20 acceptance inventory, M21 closure and M22 full qualification. Formal
ADR/design acceptance and blueprint supersession are prepared, not claimed complete.

## Outcome — M06–M08

**Implemented:** identified FeOS PC-SAFT/DIPPR mixture data, coherent fallible AD,
output-specific obligations, Symbolica/Numerica derivative artifacts, full provider
keys, separate semantic/artifact identity, explicit row/gather assembly, persistent
faer sparse buffers, coefficient/Gram/conic admission and split native oracle views.
The old public unchecked stage/value artifact and its dependent probe are deleted.

**Tested:** see [packet verification](14-m06-m08-execution.md#verification) for the
combined targeted-unit command, zero baseline and compile evidence. Independent
thermodynamic comparisons, actual native solves and full qualification remain M22.

**Mistake corrected:** initial output pruning preserved all property calls regardless
of callback demand. Explicit typed output effects now retain canceled obligations
while excluding calls belonging only to another output. Raw/Taylor factorials and
alias-to-diagonal accumulation have direct regression controls.

**Deliberate deviations:** use explicit density and pressure equations; NPT is only
initialization. Numerica stays on the Symbolica side and num-dual on the FeOS side.
Use interpreted evaluators; JIT/SIMD remain unqualified. General domain proofs and
cone inference are excluded rather than guessed. Exact Gram evidence supplies the
initial convex-QP proof; rank/eigenvalue diagnostics belong to later consumers.

**Remaining:** M09 graph analysis; M10 semantic incremental reuse and whole-attempt
resource ownership; M11–M18 native workflows; M19 cleanup; M20 acceptance cases;
M21 closure; M22 full qualification and formal ADR/blueprint reconciliation.


[R1]: ../design_review/reviews/design_review_library-owned-math-pipeline_2026-09-23.md
[R2]: ../design_review/reviews/design_review_native-library-math-target_2026-09-24.md

## Outcome — M10–M14

**Implemented:** one class-routed, completion-owned native solver system over five
library adapters, with semi-variable/PSD profiles, finite reuse, faithful reports,
Salsa-prepared conditional initialization and native/library tear/recycle strategies.
See the [packet outcome](14-m10-m14-execution.md#outcome) for corrected mistakes and
deliberate scope boundaries. Earlier outcomes above are historical checkpoints.

**Tested:** `just unit-native-contracts` passes 95 selected units, zero failures,
58 excluded, in the pinned container/full native/force-validation profile. Compilation
passes for full native and default runtime profiles. M15–M22 remain; this does not
establish native convergence, public process workflows or full qualification.


## Outcome — M15–M16

**Implemented:** library-owned qualified transformations and original-model recovery
are shared across Ipopt/POUNCE. Registry-generated authoring and result contracts
connect immutable revisions, Salsa preparation, the existing native supervisor,
blocking/async Python jobs, owned Arrow results and explicit exact Delta publication.
The [packet outcome](14-m15-m16-execution.md#outcome) records corrected mistakes,
public construction boundaries, actual evidence and the remaining Clippy obligation.

**Tested:** `just unit-native-contracts` passes 105 selected units, zero failures,
58 excluded, in the full native/force-validation profile. Public Python unit evidence
and generated-boundary checks are recorded in the packet. Compilation passes for
default runtime and full native Python profiles. This is targeted package completion;
M17–M22 and the selected process/scientific acceptance journeys remain open.


## Outcome — M17–M18

**Implemented:** native Diffsol BDF, consistent initialization, finite events/sampling
and smooth forward sensitivities consume shared Symbolica/Numerica functions and
Salsa structural admission. Steady/transient/mixed fitting shares native NLP, presolve,
faer response/rank calculations and the public joined-result/publication lifecycle.
Generated declarations retain exact physical source contracts. The
[packet](14-m17-m18-execution.md#outcome) records refinements and corrected mistakes.

**Tested:** targeted commands, feature profiles, zero baseline and final counts are
in the [packet verification](14-m17-m18-execution.md#verification). This completes
M00–M18 implementation at the package boundary; earlier remaining-work statements
are historical checkpoints. M19–M21 and the full M22 campaign remain open. No actual
vessel/parameter-recovery, installed-publication or performance qualification is implied.

## Outcome — M19–M20

**Implemented:** residual crate/generated/dependency cleanup, compact bit-exact physical
fixtures, library enum/AST mechanisms, and concrete native acceptance/measurement
bodies. The v3 manifest binds 29 case groups and 135 exact witnesses to current profiles
and Q01–Q18. Independent teqp/Decimal references, shared Rust/Python declarations and
strict source/native/artifact evidence replace placeholder acceptance.

**Tested:** targeted commands, conditions, zero baseline and counts are in the
[packet verification](14-m19-m20-execution.md#verification). Native discovery compiled
109 Rust tests and collected three Python tests without executing process journeys.
The [packet outcome](14-m19-m20-execution.md#outcome) records corrected mistakes and
deliberate library dispositions. M00–M20 implementation is complete; M21 closure and
M22 full qualification remain open. Earlier remaining-work statements are historical.


## M21 implementation outcome

**Implemented:** [M21](14-m21-execution.md) closes all implementation/deletion rows.
Physical balances derive from signed source contributions; property windows are
explicit and enforced; original-equation structure and scaled response residuals
control native admission. Diffsol owns integrated conservation fluxes. Compiler
policy, generated contracts, source consumers and exact unit-evidence tooling now
match this target. No production Pyomo/legacy math fallback is retained.

**Tested:** targeted unit/profile evidence and its zero baseline are recorded in the
M21 packet. Source/native seals authorize only this implementation checkpoint.
Earlier remaining-work paragraphs are historical checkpoints. M22 remains open for
complete functional/scientific/runtime/static qualification, measurement, independent
reviews and formal ADR/blueprint reconciliation. No end-to-end speedup or scientific
qualification is claimed by closing M21.
