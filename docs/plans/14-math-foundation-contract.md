---
title: Plan 14 mathematical foundation contracts and decision amendment
status: in-progress
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
---

# Mathematical foundation and native execution after M00–M21

This document records the implemented foundation and the concrete decision amendment
prepared for the ADR/design PR. It does not amend the accepted blueprint by implication.
The [execution inventory](14-execution-inventory.md) owns package status; the
[main plan](14-library-owned-process-simulator.md) owns the remaining target.

## Settled decisions and supported scope

The maintainer's U1–U5 decisions in the original math review remain selected:

| Decision | Implementation consequence |
|---|---|
| U1: real algebra on an admitted domain | Physical checks and domain barriers precede CAS. Floating operation order and bitwise equivalence with the old evaluator are not the mathematical contract. No approximate or finite-difference fallback. |
| U2: Symbolica starts at the typed definition | The authored AST lowers directly through `BodyBuilder`; no P10/MathIR intermediate. |
| U3: specialization-local bodies | `BodySpec`, `CompilerWorkspace`, `CasePlan`, `ArtifactRequest`, `InstanceBinding`, `CaseStructure`, `CaseValues` and evaluator scratch have separate owners. A known shape reuses its compiled evaluator; a genuinely changed shape can require compilation. |
| U4: direct Ipopt C plus POUNCE for NLP | Implemented adapters share `NlpOracle`; KINSOL roots, HiGHS coefficients and Clarabel cones have distinct representations under one runtime lifecycle. Diffsol owns the admitted fixed-mass ODE/index-1 profile. |
| U5: personal-use deployment | The existing multi-core hobby license is provisioned locally through `SYMBOLICA_LICENSE`. Initialization supplies the optional environment value to the public license API once; the native test runner forwards it by variable name. No key or distribution framework is committed. |

The implemented scope is finite, physically typed scalar/indexed algebra, explicit
instance/value bindings, guarded library values and exact first/second derivatives.
It includes finite sums/products, filtered membership, ragged tuples, aliases, affine
unit connections, and separate objective/constraint/gradient/Jacobian/Hessian demands.
Symbolica/Numerica own arithmetic and derivative composition; faer owns numerical
sparse structure and multiplication. Unsupported function spellings fail admission.
Free-variable switching remains value-only. Fixed/parameter-only guards can be
differentiated with respect to other coordinates. `sqrt(0)` remains value-valid and
derivative-ineligible. Output-specific obligation dependencies survive simplification.

M06 implements methane/ethane/propane PC-SAFT with DIPPR ideal caloric contributions,
identified source coefficients and explicit zero binary interactions. Independent
coordinates are `[T, molar density, x_methane, x_ethane]`; propane is the complement.
Outputs are pressure, total molar enthalpy/entropy and the three logarithmic fugacity
coefficients. FeOS NPT initialization is separate and does not promise vapor/liquid
branch selection. The declared caloric convention is FeOS's 298.15 K integration
reference without formation enthalpy. An explicit operating envelope is enforced for temperature, density, pressure and
all three fractions, including NPT initialization. Its provenance and bounds enter
provider identity. This declared window is not empirical certification; independent
physical reference comparisons remain M22.

M08 supplies explicit row accumulation, sparse Jacobians/Lagrangian Hessians,
coefficient and native oracle views, and validated explicit cone data. All-fixed
models evaluate as constants. M15–M16 connect those adapters to qualified library
presolve, generated model declarations, immutable revisions, public Rust/Python jobs,
physical result tables and explicit exact publication. The selected heater/flash
simulator is **not scientifically qualified**: M22 still owns the installed process
journeys and reference comparisons. M17–M18 implement native dynamics and fitting
within the profiles below.
M09–M10 add complete selected-case structural analysis, Salsa semantic reuse and
bounded runtime artifact/worker ownership. Dynamics uses a fixed diag(I,0)
ODE/index-1 mass-matrix profile. General MINLP, GPU, distributed execution, JIT and SIMD are not admitted.

## Library profile and composition evidence

Pins remain authoritative in workspace dependencies and `Cargo.lock`; this table
records why the resolved profiles are selected, rather than acting as another resolver.

| Library/profile | Selected use and limitation |
|---|---|
| Symbolica 3.0.0, defaults off, `integer-gmp,float-mpfr` | Atoms, built-in normalization/differentiation, Numerica hyperdual vectorization and multi-output evaluators. Host tracing remains untouched. No library allocator override, runtime code generation or native-CPU flags. Optimizer concurrency is explicit; normal parallel units and a two-core optimizer control pass with the local license. |
| faer 0.24.4, `std,sparse-linalg` | Owned sparse representations, canonical duplicate accumulation, stable refill maps and sparse matrix-vector multiplication. Alias-Hessian control includes the repeated-global-variable contribution. Native solver factorization stays with its library; M18 uses bounded faer pivoted LU/SVD for regular implicit responses and local observation rank. |
| FeOS/feos-core 0.10.1, PC-SAFT; num-dual 0.14.2; quantity 0.14.0; nalgebra 0.35.0 | One compatible provider composition. The production package computes requested value/first/full-second outputs from one coherent state; a higher-order cached result can serve reordered output subsets. No cross-version dual-number ABI is assumed. |
| Ipopt generated C ABI 3.14.20 | Existing digest-pinned solver image and `pse-ipopt-sys/link`; 32-bit indices, f64 scalars and real C symbol relocations tested inside that runtime. M11 implements direct C callbacks, exact/limited-memory Hessians, primal/dual starts, typed scaling, protected options, native diagnostics and RAII. The pinned MUMPS profile is serial. |
| HiGHS 2.4.0 / highs-sys 1.14.3 | Bundled native HiGHS 1.14.0; 32-bit `HighsInt`, native create/destroy/option symbols linked. M14 implements checked native upload/update, LP/MILP and certified continuous convex QP, binary/semi domains, bases/starts, full option readback, available metrics, rays/IIS/ranging and separate relaxation diagnostics. Scheduler reset is exclusive and blocking; pinned QP interruption is time-limit only. |
| Clarabel 0.11.1, `serde`, defaults off; optional `sdp-netlib` | M14 implements explicit cone execution, native settings/info/results, source-space quality, upper-column svec PSD packing and bound-row maps. The full native profile selects serial LP64 netlib SDP. `faer-sparse` stays off. Reusable-data mode disables presolve/chordal decomposition/zero dropping and checks native update eligibility. No external iterate warm API is advertised. |
| sundials-sys 0.6.2, `kinsol,klu,build_libraries,static_libraries`; suitesparse_sys 0.1.4 | M13 implements SUNDIALS 7.1.1 serial vectors, KLU/dense/analytic-SPGMR, Newton/line search and explicitly declared fixed-point/Picard/Anderson profiles. SuiteSparse 7.7.0 KLU is built from the pinned crate vendor source, static/PIC without CHOLMOD/CUDA/OpenMP. Sign constraints are admitted exactly; arbitrary boxes and constrained fixed-point/Picard are refused. |
| POUNCE 0.12.0 plus pounce-feral 0.12.0 | Native TNLP/application, FERAL, interior point and explicit active-set SQP; library restoration, full statistics/timing and linear summaries, applicable starts and compatible application retention. Admitted local Rayon pools own parallel FERAL. M15 uses the shared qualified pounce-presolve TNLP wrappers; retained application does not imply retained factors. |
| Diffsol 0.16.2, `faer`, defaults off | M17 implements BDF, consistent initialization, finite roots/resets/input changes, interpolation and smooth forward sensitivities through explicit native operators. Sequential faer context, fixed diag(I,0), known sparsity and typed failure containment; analytical contract controls pass, while physical trajectory qualification remains M22. |

The production derivative seam invokes fallible FeOS/num-dual workers outside
Symbolica callbacks. Symbolica differentiates formal provider compositions to produce
lift evaluators; Numerica supplies arithmetic-region jets. One layout converts Taylor
coefficients to raw derivatives, including diagonal factorials. Numerica hyperduals
and FeOS duals are intentionally private to their library boundary; neither faer nor
FeOS is forced to implement an incompatible scalar trait. Interpreted, guarded profiles
are tested. Vectorized evaluators must not call `optimize_stack()` after vectorization:
the pinned planning probe aborted. JIT/SIMD need separate ownership and equivalence
qualification before admission.

Oximo, diffsol-nl, IDA/IDAS, additional presolve adapters and further graph libraries are
not added without their assigned target consumer. Native convergence, cross-backend numerical comparisons and native scheduler/resource stress are M22 evidence obligations.

## Ownership and computation contracts

| Stage | Authority, inputs and equality | Mechanism, bounds and effects |
|---|---|---|
| Physical source admission | Actual selected Arrow relations and exact quantity/reference declarations; missing input differs from present-empty | `PhysicalInventory::load` uses the retained native session and checked batches once per inventory; retains allocation ownership. It does not execute mathematics or infer units from numeric values. |
| Typed local preparation | Parsed definition, formal layout, finite membership/filter choices, actual group tuples, physical revision, provider contracts, numerical policy | Typed Rust physical inference precedes Atom construction. Source spans remain diagnostic metadata. Finite expansion is local; no per-tuple SQL or model-wide expression graph. Unknown ragged tuples fail. |
| Evaluator construction | `BodySpec` semantic key plus admitted order and optimizer options; no Atom handles or printed strings | Symbolica owns arithmetic and optimization. Obligation/branch/provider regions are project control flow only. Library symbol discovery validates stage dependencies and single assignment. No custom arithmetic interpreter or AD tape. |
| Body reuse | Private Salsa inputs over actual physical inventories, immutable provider descriptors, selected definitions/membership/bindings and consumed assumptions | Tracked negative lookups, semantic backdating, bounded LRU values and generation reconstruction. `BodyStore` and the caller-asserted physical-hash preparation route are deleted. Diagnostic spans refresh independently of semantic products. No persistent Salsa store. |
| Structural analysis | Complete admitted equality rows/free columns, all-branch support and objective/inequality coupling | pounce-presolve matching/DM/components/BTF; semantic result validation and rustworkx deterministic topological tie breaking. Partial inventories cannot establish complete results. Recursive matching is limited to 100,000 rows on a 32 MiB stack. |
| Artifact retention | Compiler-issued demand/profile/source/build/ABI identity; explicit output/coordinate ordering and evaluation limits | DataFusion `DefaultCache` inside the shared runtime native cache component. Typed completion-owned single flights, epoch fences and per-allocation leases. Failures are retryable; abandoned native work retains its key until actual exit. |
| Instance/case binding | Stable semantic source/row identities, fixed/free structure, canonical formal ports; ordinary numeric parameters are separate | Repeated source slots are aliases, not duplicate independent variables. Unit conversion requires matching kind/basis/reference/scale; composition conversion needs an explicit physical operation. Fixed/free edits change the case layout, not the body. |
| Trial evaluation | Finite ordered inputs, admitted derivative-neighborhood requirement, exact provider phase/data/spec, cancellation | A cloned library evaluator and caller-owned provider workers have private scratch. One-trial caches include complete local inputs and derivative/output demand. Failed evaluation clears its cache; only a complete finite result is returned. Failure never returns the preceding trial's outputs. Instance evaluation wraps the
original typed source failure with the bound instance identity. No Arrow/Delta writes occur. |

`PreparedBody`, `CompiledBody` and `Worker` separate semantic preparation, effect-owned
compilation and mutable scratch. The unchecked public stage/value-artifact API is
removed. Compilation settings are excluded from `BodySpec` and included in artifact
identity. Only consumed provider contracts enter the semantic key. Identical provider
calls coalesce within the active region, while output-specific effects preserve
canceled calls without making unrelated callbacks execute them.

`CasePlan` admits `z = Gx + Bp + c`, explicit row contributions and mapped
Lagrangian weights. faer establishes sparse ordering once; a mechanical refill map
accumulates values into retained buffers. Both local cross-partials accumulate when
an alias lands on a global diagonal; off-diagonal entries are not doubled. The full
selected row/free-variable inventories survive, including isolates and numerical
zeros. Identical local body/output/coordinate artifacts share immutable programs
through shared runtime retention; each attempt owns its mutable evaluators and provider map.
`CaseAssembly` binds completed artifacts to the immutable plan. Graph-only planning
constructs no numeric evaluators. Registry edits re-admit representation conversions.
Coefficient queries observe only consumed fixed/parameter values; free trial values
do not invalidate structural plans or value-independent programs.

Coefficient extraction first uses Symbolica derivatives to establish affine/degree-two
eligibility, then uses library expansion and polynomial conversion. Original domain
obligations must be proved by constant evaluation or direct coordinate bounds; other
proofs are rejected. M15 adds independently useful affine-row proofs and loss-aware
FBBT tapes; it does not turn arbitrary domain obligations into proofs. Snapshots retain structure
and consumed fixed/parameter identity, objective constants/sense, explicit variable
domains and shifted row bounds. Quadratic degree does not establish convexity:
`GramCertificate` checks `sign*Q = Rᵀ diag(w) R` using exact represented rationals
through Symbolica/Numerica, requires nonnegative weights, and rejects stale matrices
or objective orientation. Explicit Clarabel cones receive library CSC-format checks
and profile-specific parameter/dimension checks. No arbitrary cone recognition is
claimed. NLP callbacks are separate demands; NLE requires finite equality rows,
square structure and no objective, and supplies an assembled Jacobian/faer JVP.
These problem views now feed the five native adapters through the shared Rust lifecycle. M17 adds physical dynamic roles and pure Salsa algebraic-partition admission.

Resource policies refuse excess rather than truncate: at most 4,096 reusable formal
symbols/slots, 16,384 local construction occurrences/control stages, depth 128 and
integral power degree 1,024. Positive integral powers materialize their bases before
CAS to avoid constructing enormous exact constants during admission. Provider
registrations have independent port/component limits. Case scalar, instance, row,
slot and body capacities are explicit. Optimizer budgets bound cores (1–64), Horner
search, common-pair work and caches; cancellation reaches the library builder and
each runtime region. Derivative component/operation/scratch budgets and case contribution/native-index/aggregate-worker budgets are explicit. The opaque-support profile refuses local widths above 256; this is not a global model-size limit. Coefficient and Gram proofs have explicit term/operation bounds. These logical limits do not claim interception of every foreign
allocation or a measured process RSS cap. M10 adds shared-pool admission and
completion-owned worker lifetimes; M22 must measure actual foreign/TLS/RSS behavior.

## Evidence-based reuse and deletion

| Mechanism and current consumer | Disposition and evidence |
|---|---|
| `pse-quantity` inference/registry; `BodyBuilder` and `SlotBinding` | Retain physical semantics, independent of the deleted arithmetic IR. Point subtraction, datum mismatch and cross-unit connection controls exercise the new boundary. Physical operation vocabulary does not advertise numerical availability. |
| `pse-ids`; BodySpec/provider/source identities | Retain framed semantic identity. Body tests distinguish physical/structural changes and exclude whitespace, instance values and library registration order. |
| Source binding contracts; runtime `authoring_driver/document/binding.rs` | Rehost in `pse-compiler::source_binding`; retain source ownership/rename semantics without the old lowering engine. |
| Package dependencies; runtime authoring P0 | Retain complete directed package projection and library topological order, including isolates and missing-dependency refusal. |
| Rule execution; `pse-rules::strata` | Retire the orphaned executor, exclusive schema contracts, tests and benchmark callers under ADR-0086. Production declares no rules. Retain `pse-rules::invariants` for inspection and physical fixture validation. |
| Old incidence/DM, containment, kernel/conversion graphs, tear helpers | Delete adapters and tests without current target consumers. Remove the rust-igraph dependency. M09 introduces matching/DM/BTF from library support and new bindings; M14 adds target tear selection; they do not inherit this old implementation. |
| Engine sessions, checked relation construction, Delta publication | Retain for physical/source input and data/inspection consumers. Rehost a generic collection ownership test onto authored symbol expressions. Retired mathematical transports stay deleted. The current native workflow publishes generated source/result relations through the retained exact control-last path; end-to-end qualification remains M22. |
| Compiler Salsa products and old release-update reader | Delete MathIR-dependent products and their journeys. M10 establishes new-target inputs and queries in `CompilerWorkspace`. Generic relational cache ownership remains with its real consumers. |
| `pse-mathir`, `pse-numerics`, `pse-templates`, NL/Pyomo stubs | Delete crates, exports, callers and dependent fixtures. `pse-math` contains new library integration, not renamed legacy arithmetic. |
| Expression schemas/generators and legacy P3 declaration | Delete expression-node/predicate/equation/compiled math transports, generated sinks/loaders and unsupported derived outputs. Regenerate Rust, Python, docs and concrete invariant fixtures through their owners. |
| Old native driver and public compile/solve routes | Delete under the approved hard-cut scope. Keep the raw Ipopt ABI and new class-specific native adapters/contracts. No compatibility API or placeholder success path. |
| Production Pyomo/Pint and old campaign tooling | Remove product dependencies and adapter paths; parity retains its isolated reference environment. Plan 14 rejects historical seals and carried mappings. M20 supplies complete target workloads; performance requires current functional qualification. |

## M09–M10 ownership and resource limits

`ResourceBudget.math` is a finite policy in the existing deployment pool. It bounds
artifact retention, foreign allocation allowances, worker storage, workspace capacity,
thread stacks, live jobs and flights. The same CPU semaphore serves data and math
work; each optimizer reserves all requested cores once. `with_worker` constructs,
uses and destroys providers/evaluators on the owning thread. A join supervisor keeps
permits and reservations through thread-local destruction after caller cancellation.

Project inventory/cardinality checks, conservative container-capacity admission, Salsa
LRU, query-key/revision generation rotation, and Salsa metadata/structural `heap_size`
observations constrain retained state. There are no escaping database clones or retiring
Salsa generations. Runtime prepared results reserve their own allowance; they can
outlive the workspace. Compiler publication is single-writer and validated before setters.
Foreign/GMP/Symbolica storage uses an explicit admission estimate, not an allocator
interceptor or proven RSS ceiling. `scratch_bytes` reports numeric scratch only.
Unobservable pinned/foreign extents remain unknown in reports rather than fabricated.

Complete physical inventory identity conservatively re-admits bodies after any registry
change. Definition/provider lookups backdate unrelated edits. A finer consumed-physical
fingerprint is not required for correctness and remains a measured optimization choice.
The [M09–M10 packet](14-m09-m10-execution.md) owns targeted acceptance and deviations.

## Concrete decision amendment prepared for the design PR

These are the proposed replacement contracts under ADR-0082–0084. Accepted ADR text
remains immutable. Apply the blueprint amendment with a revision row and inline
decision markers through the repository's ADR/design PR process; no PR was requested
or opened by this implementation task.

| Governed text | Replacement contract |
|---|---|
| D6, §7, §6.15.4, §14; ordered-IR parts of ADR-0047 | Authored process structure and full physical types are authoritative. Symbolica atoms/evaluators are derived library artifacts. Real-algebra equivalence is conditional on explicit original-domain obligations and the selected numerical profile. Source occurrence identity is independent of library-local symbols. |
| D10/D11, §3.2, §6.11, §18.2; relevant ADR-0037/0068/0076/0078 portions | Math uses specialization-local library artifacts with explicit bindings; native solvers own algorithms. DataFusion/Arrow retain actual relational/data work. Add `pse-math`, retire the five removed crates, remove expression relation transport and old compiler-query products. |
| D9, §9, §13.6; relevant ADR-0022/0043 portions | Executable provider registration includes complete physical ports, data/component identity, selected phase, implemented derivatives, smoothness and typed trial/terminal failures. Derivative capability does not establish phase regularity. The initial dynamic claim is ODE/index-1 mass matrix. |
| D12, §18–§21; ADR-0015 and affected ADR-0028/0038 portions | Production execution is native and class-specific. Python is authoring/results convenience; reference parity is separate. Remove Pyomo/NL production routes. Future result contracts separate solver termination from recomputed original-model quality. |
| Proposed ADR-0075 and R-32 | Retire the proposed Pyomo tear route. M14 decides the native heuristic/exact route from actual process-structure consumers; no current tear capability is advertised. |
| Pending ADR-0076–0081 and old plan acceptance | Preserve only meanings with consumers listed above. Target M20/M22 owns new coverage and acceptance. Old source seals and failed/incomplete outcomes remain historical. |

Formal supersession, blueprint acceptance, the ADR index and final governance/doc
qualification remain part of the decision PR/final Plan 14 closure. The implementation
does not claim those governance actions have already occurred.

## M10–M14 solver lifecycle and supported boundaries

`SolverProfile` separates intent, auto/explicit backend, finite controls, physical
acceptance tolerances and native settings. Automatic routing is deterministic and
returns explicit unavailable/ineligible errors; it never relaxes integrality, invents
an objective or converts arbitrary boxes to root sign constraints. All-fixed models
are evaluated directly. Coefficient-only requests avoid evaluator construction.

`MathService::prepare_solve` and `prepare_conic` admit immutable representations.
`solve` executes a finite `SolveSequence` on one admitted worker, with compatible
Ipopt C allocation, POUNCE application, KINSOL allocation, HiGHS model and Clarabel
data reuse. Reuse uses semantic source/physical coordinates, framed sparse patterns,
typed profile bits and consumed numerical assumptions. Native starts are separate
from native allocation reuse. No long-lived interactive native model service exists.

Salsa `problem_facts`, flow projection and conditional block queries observe complete
membership and physical inputs. BTF block plans bind unselected variables as fixed,
remove unrelated rows/objective demands, and produce ordinary library artifacts.
`prepare_initialization`/`initialize` use predecessor order, explicit physical scales,
finite supplied continuation and successful-block commits. `solve_declared_root`
constructs an explicit map or Picard splitting on the same admitted owner thread.
`CausalMap` evaluates declared causal units over the independently witnessed residual
DAG and propagates physical conversions; KINSOL owns all fixed-point iteration.
The unit interface requires complete input/output roles and exact external inputs.
The typed Rust services retain these advanced construction contracts. M16 supplies
the generated algebraic model frontdoor; M17–M18 add generated dynamic/fitting
declarations and the selected durable FeOS factory. Arbitrary flow maps and cones
remain typed Rust construction.

`prepare_flow`/`select_tears` share the same runtime lifecycle. Connection occurrences,
node isolates, decision groups/costs and mandatory/forbidden policy survive projection.
HiGHS solves order-variable feedback-edge MILP; petgraph separately checks acyclicity.
The explicit unweighted greedy route reports heuristic assurance. An available MIP
incumbent, best bound, gap and optimality claim remain distinct observations.

`SolveReport` retains raw native termination, assurance, optional candidate/certificates,
physical row/bound violations, dimensionless aggregate ratios, available metrics,
explicit options/defaults, provenance and bounded events. Missing data never means zero.
Postsolve callback failure preserves the native outcome and clears assurance; optional
HiGHS diagnostic failure does not replace the original solve. Source IDs and authored
objective sense/constant survive native layouts. M15 implements native transformed-coordinate
recovery with independent original observations and qualified multipliers; M16 encodes
these observations into retained Arrow batches and explicit Delta publication.

Runtime CPU, stack and foreign allowances survive native destruction, scheduler reset,
thread join and TLS destruction. Result envelopes retain a shared allocation lease when
extracted or cloned; explicit caller-made data copies are outside this ownership domain.
Progress bounds include event count and per-event retained extent; oversized events
increment an omitted count. Native/foreign temporary allocations still use conservative
allowances, not allocator interception or a hard RSS guarantee. Long native calls can
only cancel at library-supported checkpoints. M22 must measure these practical limits.

The full optional profile is `pse-runtime/native-solvers`; individual features are
`solver-ipopt`, `solver-pounce`, `solver-kinsol`, `solver-highs`, `solver-sdp` and
`solver-diffsol`.
Clarabel's non-SDP route is available in the default profile. The package check recipes
own exact compiler flags, native prefixes, container execution and force-validation.
The KLU build profile is keyed by `klu-profile-v1`; changes to that profile require a
new key. It builds only upstream vendored source and serializes construction with a lock.


## M15–M16 declarations, transformations and public ownership

The [execution packet](14-m15-m16-execution.md) owns completion evidence and
[workflow guide](../dev/native-workflow.md) describes the callable surface.
`authored.computation_models` owns algebraic model declarations. Typed
builders and package documents pass the same field/semantic admission. Physical
contexts retain real admitted source batches; there is no implicit fixture catalog.
Each immutable revision records its full declaration, physical and native provider
identity. Atomic revision preparation publishes selected inputs under the compiler
lock, so a shared Salsa workspace cannot mix two revisions during preparation.

`presolve_facts` observes source structure and consumed fixed/parameter values,
proves affine rows individually through Symbolica and emits the bounded native
FbbtTape vocabulary. Unsupported expressions remain opaque; guards remain owned
by the original evaluator. The native pipeline qualifies fixed upstream thresholds,
normalizes affine constants and wraps original TNLP → PresolveTnlp →
LinearEqElimTnlp → native scaling. Libraries own interval propagation, elimination,
derivative transport, warm projection and recovery. The project records effects
and inverse source attribution, not a second executable transformation IR.

A transformed candidate receives fresh original objective/constraint/bound checks.
Maximization multipliers use the documented minimization Lagrangian convention.
Native failure, candidate availability, physical feasibility and KKT observations
remain separate. The pinned library stands down on all-column elimination; an
authored all-fixed model instead follows direct evaluation. Numerical scaling is
not a unit conversion. Full expression/map/scale identity controls reuse, including
cases the native library fingerprint cannot distinguish.

Public RunHandle waiters share one immutable terminal result; the existing native
supervisor owns cancellation, destruction and join. The PyO3 async bridge borrows
the process Tokio executor. Blocking waits detach/check signals; async waiter
cancellation requests native stop without dropping its completion owner. Generated
result contracts expose original variables/parameters, physical violations and
qualified multipliers, complete native metrics/diagnostics and provenance. Checked
Arrow batches retain buffer leases through the final array reader.

Publication consumes an immutable result through existing member writes, conditional
parent and control-last settlement. Preparation performs no writes. A one-use
attempt retains identifiers for unresolved-effect inspection; there is no automatic
write retry or solver replay. Stored declarations support explicit rebuild;
the selected generated FeOS factory can reconstruct its declared registration.
Mutable evaluators, factors and integrator state are never durable authorities.

M19 cleanup and M20 acceptance implementation are complete. M21 final closure and
M22 qualification remain open, including whole-plan Clippy/static closure, formal
decisions, full installed native/publication journeys, execution against the independent
scientific references and resource/performance measurements.


## M17–M18 dynamics and fitting

The [execution packet](14-m17-m18-execution.md) defines the implemented profiles and
targeted evidence. Generated dynamic/provider/fitting declarations join model, dataset
and observation rows in revision identity, document/edit roundtrip and retained source
publication. `CasePlan::functions` and bounded Salsa queries select outputs and explicit
variable/parameter coordinates; compiler-issued artifacts use the same runtime cache.
Algebraic partition matching is pure compiler work. Trial integrations and fits are not.

Native dynamics uses seconds and physically registered time-derivative quantities.
State normalization, algebraic residual scaling and native tolerances have distinct
contracts. Initial functions cannot depend on dynamic states. Diffsol owns consistency,
BDF, interpolation, root location and smooth forward sensitivities. A contained private
Rust unwind bridges its infallible operators to recorded typed failure; the native
solver is discarded before any C/Python boundary. Events rewind to native root time,
reset/reinitialize before coincident sampling and preserve completed output on failure.
General implicit/high-index DAE and hybrid derivatives are not admitted.

Fitting uses the ordinary native NLP and library presolve pipeline. Shared parameters
precede experiment-specific steady unknowns; original constraints remain physical.
Transient blocks integrate inline with native forward sensitivities and require
limited-memory Hessians. The fixed weighted squared loss uses faer products and
compiled local partials. Exact steady Hessians include the residual second-derivative
term. All-fixed fits directly evaluate with truthful physical quality.

Local physical response requires a feasible regular square equality closure; faer
pivoted LU solves the implicit response and bounded SVD reports scaled observation
rank. Neither native convergence nor rank implies covariance or global identifiability.
Generated fit state/constraint rows preserve original source IDs, units and tolerances.
Native telemetry, public joined jobs, final-buffer ownership and control-last
publication use their existing owners. Full process recovery, scientific comparisons,
publication fault tests and measured resource/performance claims remain M22.

## M19–M20 surviving boundaries and acceptance

Empty `pse-plans`/`pse-kernels-ext` crates, unused compiled spatial families and obsolete
expression sharing generation are removed. Finite authored domains and actual
compiler/provider owners remain. syn owns structural path traversal and strum owns
leaf enum parsing/display/iteration; codes and wire spelling remain explicit.
Compact generated quantity fixtures retain exact float bits and ordinary admission.

Explicit codecs and fallible allocation/final-buffer leases remain because the proposed
serde_arrow and Arrow pool adapters do not satisfy all surviving ID, metadata, physical
admission and allocation contracts. Confirmed unused direct dependencies are gone;
shared-fixture imports, macro consumers and type-universe anchors remain.

The [acceptance packet](14-m19-m20-execution.md) binds implemented tests, independent
references, complete process workloads and evidence profiles to the current target.
Discovery proves compiled identities, not convergence. The shared source fixture is
public Rust/Python input; offline reference generation is isolated from production.

## M21 final contract alignment

The [M21 packet](14-m21-execution.md) and its [implementation review](../design_review/reviews/design_review_m21-design-closure_2026-09-24.md)
record the final consumer/deletion dispositions. Source models no longer accept an
arbitrary policy hash; the compiler owns guarded-real semantics and their identity.
The supported function vocabulary derives from `strum`; removed grammar has no
numerical fallback. Convert/Broadcast retain actual authoring/inspection consumers
and remain unavailable for numerical execution. The broader physical inference
vocabulary in `pse-quantity` describes physical compatibility, not evaluator support.
Obsolete specialized ID constructors, the unused runtime host relation and the
active Pyomo probe/regeneration route are deleted. Isolated reference parity is
still a deliberate nonproduction consumer.

`authored.physical_balances` owns contribution source IDs, typed roles, internal
transfer pairs, optional mode selection, conserved-state binding, explicit tolerances,
event impulses and provenance. Transient compiler projection derives each balance
row from these terms; a duplicate authored equation is rejected. Mode expansion is
charged before allocation. Raw original output observations produce separate
`runtime.physical_checks`; solver status and mathematical feasibility do not imply
physical closure. Missing observations have null acceptance and an error.

The vessel constructor uses complete typed ports and generated values directly.
Diffsol's integrated output equations, output tolerances and interpolation own flux
integration. Segment carry and explicitly declared reset impulses preserve physical
accounting across events; no host quadrature is introduced. Dynamic closure is
canonical accumulation change minus native integrated flux minus declared impulses.
Fitting reports physical checks per experiment/sample through the same result family.

Root admission consumes complete original-equation matching and rejects unmatched
rows/columns with semantic IDs. NLP requires matched equalities and permits genuine
optimization degrees of freedom. Fixed-point/Picard admission observes original
residual support. Existing dynamic algebraic-partition analysis and native
coefficient/conic presolve retain their respective meanings. Structural matching
never certifies numerical rank. Implicit response solves use faer in scaled coordinates
and verify normwise backward error before returning physical sensitivities.

The development barrier authenticates actual unit runner reports, exact test identities,
current source and current native binary/library bytes. It does not establish process
validity or supply independent reviews. The M22 review set is G1–G8 plus PS-G1–PS-G3;
none of these independent decisions is supplied by the implementation self-review.
