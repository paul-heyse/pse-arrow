---
title: Native library mathematics and process simulation — follow-up target review
date: 2026-09-24
status: proposed
depth: deep
scope: >-
  Follow-up to the 2026-09-23 library-owned math pipeline review, covering native
  mathematics, thermodynamics, initialization, solver integration and dynamics.
evidence: >-
  Interface-checked public library APIs and exact published crate manifests;
  selected current implementation evidence; Proposed target and qualification.
decision: >-
  Endorse the hard pivot; revise the detailed target to resolve composition and
  capability gaps before treating it as an implementation-complete design.
---

# Native library mathematics and process simulation — follow-up target review

## 1. Decision and scope

**Decision: endorse the hard pivot; revise the proposed target before treating it as an
implementation-complete design.** Symbolica should replace the bespoke arithmetic IR,
differentiators and evaluator. Native solvers should replace production Pyomo integration.
Extend the same ownership principle to thermodynamics, nonlinear initialization and time
integration. The project should own process meaning and the composition of these capabilities,
with the mathematical algorithms owned by libraries.

**Reviewed proposal:** [Library-owned math pipeline, 2026-09-23][original], especially §§3–4,
§8.3, §9.3 and §11. **Reviewer:** Codex. **Evidence:** target recommendations are **Proposed**;
the named library contracts are **Interface-checked** where indicated. No new integrated
implementation, correctness test or performance measurement is claimed.

The prior review correctly identifies the central simplification: one library-owned mathematics
representation, reusable template bodies, native solver callbacks, and deletion of superseded
production machinery. Its five recorded maintainer decisions, U1–U5, remain the starting point:
real-algebra equivalence on an explicitly admissible domain; Symbolica after physical typing;
per-specialization evaluation; Ipopt C and POUNCE as the NLP targets; and the already-decided
Symbolica operating arrangement. This review does not reopen those decisions.

The original also already requires explicit nonsmooth policy, non-finite-result errors, solver
status fidelity and coherent publication. The additions below make their difficult composition
cases decidable; they should not be read as claims that those concerns were entirely absent.

The most consequential improvements are:

1. **Let libraries own complete algorithms.** Use a nonlinear solver for initialization and
   recycle convergence, a thermodynamic library for supported property models, and an ODE/DAE
   solver for dynamics. Selecting faer does not itself remove a custom Newton solver.
2. **Use direct problem-specific solver interfaces.** Keep a shared numerical NLP oracle for
   Ipopt and POUNCE. Send sparse coefficients directly to HiGHS and Clarabel when that is the
   problem representation already available. Oximo need not become a mandatory second model.
3. **Close the composition contracts.** Lazy guards, differentiability, indexed reductions,
   repeated-variable binding, implicit property derivatives, scaling and postsolve mapping are
   load-bearing parts of a simulator. A collection of individually capable libraries does not
   decide these contracts automatically.
4. **Qualify actual dependency combinations.** FeOS 0.10.1 and num-dual 0.15 are not one dual-number
   type universe; Clarabel's optional faer backend is not the proposed faer 0.24 line. Contain
   provider-native types and choose feature profiles deliberately.
5. **Make acceptance a simulator journey.** An indexed, thermodynamic flowsheet with a recycle,
   initialization, optimization and interpretable physical results is the first product target.
   Scalar NLP examples and synthetic evaluator timings are necessary evidence of much narrower
   claims.

### Method and coverage

Read the original review, the data-model charter, review template and RCA guidelines; inspected
the relevant blueprint contracts for math, initialization, dynamics, solver classes and Pyomo;
and refreshed selected current implementation evidence. In particular:

- `crates/pse-compiler/src/case/prepare.rs:132–220` still requires scalarized indices and literal
  guards before constructing the numerical request.
- `crates/pse-backend-native/src/driver/callbacks.rs:166–226` still has the limited-memory-only
  Hessian callback; the same module records callback failures in attempt state.
- `crates/pse-backend-native/src/native/output.rs:1–105` gives a recorded evaluation failure
  priority over successful native termination and groups native statuses 0, 1 and 6 together.
- `Cargo.toml` contains candidate-provider commentary rather than an implemented math-library
  dependency set. `pyproject.toml` currently includes production Pyomo-related dependencies.

Used the Symbolica/faer/Oximo, Salsa, rust-graphs and Context7 skills. Context7 supplied discovery
for Diffsol and HiGHS; its FeOS resolution was irrelevant and was discarded. Exact crate
manifests/public APIs and upstream documentation were used to resolve version-sensitive claims.
Some web-indexed `latest` rustdoc pages were stale; the num-dual 0.15.0 archive, for example,
establishes its current trait signatures rather than the indexed 0.13.2 page. Symbolica was
inspected through public documentation and the skill's public API records only, without reading
its implementation source.

This is a design follow-up, not a fresh audit of every earlier finding. The previous review's
runtime probes and timings were not rerun. No solver image, new Cargo combination, derivative
bridge, JIT kernel registration, DAE model or physical-property data set was qualified here.
Uninspected provider behavior is identified as a qualification item, not assumed safe. No source
code, dependency pins, blueprint or accepted ADR was changed by this review.

**Governing objective.** The user's requested pivot governs this assessment. Existing decisions
requiring relational math execution, an ordered floating-point IR or Pyomo are amendment targets,
not reasons to preserve those mechanisms. Data-model discipline remains useful for physical
meaning, provenance and coherent publication; it does not require every intermediate to be a
relation. The supported target is a process simulator, not a general mathematical-programming
platform assembled in advance of its consumers.

## 2. Authority and lifecycle map

| Meaning | Authority and identity | Revision boundary | Permitted change | Derived or attempt-owned representation |
|---|---|---|---|---|
| Unit operations, balances, indexed domains, connectivity and specifications | Authored process definitions and stable domain IDs | Model release | Authored edit | Typed templates, finite index bindings, connection equations |
| Dimensions, quantity kinds, bases and reference states | Physical type rules and selected property-package contract | Registry/provider revision | Explicit definition or provider change | Unit-normalized slots and conversions; no CAS inference of physical validity |
| Arithmetic and its derivatives | Symbolica body derived from a physically typed definition; provider functions own their declared mathematics | Specialization and function-provider revision | Regenerate from that definition | Library atoms, derivative evaluators, optional JIT artifacts |
| Property data and phase policy | Versioned property package, parameter data, component identities and phase-selection policy | Property-package revision | Explicit selection/data edit | FeOS or another qualified provider's native objects |
| Structural versus numeric inputs | Explicit specialization/binding contract | Structure revision versus case revision | Shape/fixedness changes rebuild affected structure; ordinary values refill inputs | Sparse support, slot mappings and case buffers |
| Problem classification | Derived capability analysis with assumptions | Body, bindings, bounds and relevant parameter assumptions | Reclassify when assumptions change | NLE, NLP, linear/quadratic/conic, DAE or unsupported requirement |
| Solver settings | Backend-specific validated profile plus a small shared attempt policy | Profile revision | Explicit option change | Native options, thread limits, tolerances and initialization policy |
| Evaluation and solve state | One attempt, never the model or Salsa database | Attempt ID | Native callback/solver progression | Mutable evaluators, property workspaces, factors, iterates and diagnostics |
| Result | Original-model IDs plus model/case/profile/provider provenance | Published attempt outcome | Append a coherent result | Physical values, residuals, duals when meaningful, termination and quality evidence |

The typed frontend retains only distinctions that arithmetic libraries do not own: binders,
physical types, source locations, branch meaning and domain obligations. It is not a replacement
arithmetic optimizer or a second executable scalar IR. Small source-to-library lowering code is
justified; another home-grown differentiation, tape optimization or linear algebra subsystem is
not.

Stable IDs identify process entities. Slot ordinals, Symbolica symbols, graph indices and solver
row numbers are local addresses. Renaming a unit must not change its equations' mathematics;
reordering must only change derived permutations. Changing component membership, phase policy,
fixed/free status or a reduction's shape may change the specialization or problem structure.
Canonical library text is useful for inspection, but is neither durable semantic identity nor a
cross-version serialization guarantee.

Opaque behavior is explicit: a property provider, native solver or JIT executable has a bounded
contract and owns its workspace. Its internal iterations need not be modeled as relations.
Provider-native objects never become an independent authority for model definitions.

## 3. Semantic contracts and invariants

| Contract | Representation and enforcement | Failure behavior | Required evidence |
|---|---|---|---|
| Physical meaning precedes algebra | Typed template plus explicit conversion/provider binding before Symbolica lowering | Reject incompatible quantity kind, basis, reference state or unresolved conversion | Unit/basis/reference-state cases, including composition-dependent conversions |
| Admissibility is distinct from simplification | Guarded, dependency-ordered obligations extracted before CAS normalization | Attributed domain error on the executed path; no evaluation of an invalid untaken branch | N01 cases in §5 |
| Value validity is distinct from differentiability | Function/provider metadata for domain and C1/C2 support, with relevant branch conditions | Refuse incompatible solver mode or select an explicitly declared approximation | Boundary and phase-transition cases; no silent finite-difference fallback |
| Index binding preserves equations | Finite domain membership, filters, reduction rules, row identity and ordered slot maps | Reject unresolved membership, invalid cardinality, duplicate ownership or resource exhaustion | Empty/ragged/coupled reductions and shape-changing edits |
| Sparse assembly is a linear transformation | Explicit gather, row mapping, scaling and scatter-add maps | Reject invalid indices and unsupported nonlinear slot conversion | Aliased-slot Jacobian/Hessian and permutation tests |
| Kernel errors retain meaning | Versioned, possibly multi-output provider contract with derivative order, branch policy and typed failure | Recoverable trial rejection versus terminal provider failure/panic | Error injection through interpreted, JIT-enabled and native solver paths |
| Solver class is a real capability | Problem properties combined with provider requirements and profile | Reject unsupported class, unsupported integer/conic structure or unproven convexity requirement | Negative routing tests as well as successful solves |
| Reuse observes all dependencies | Pure compilation identity and separate artifact/profile/attempt identities | Rebuild affected artifact; never reuse a numeric result as structure | Reuse versus clean compilation after additions, removals and failed lookups |
| Publication describes the original physical model | Explicit inverse of scaling, permutation, fixed-variable handling and presolve | Publish typed failure/partial outcome; never a partial successful solution | Original-model residual and dual consistency, interrupted publication |

Real-algebra equivalence is accepted on the stated admissible domain. It permits reassociation
and different floating-point rounding; it does not establish overflow safety, a finite result,
or a scientific accuracy bound. Domain tests and numerically difficult but admissible tests are
different obligations. Use per-quantity absolute and relative tolerances, dimensionless scaled
solver tolerances, and an independently evaluated high-precision reference where cancellation
or extreme magnitudes matter. The old ordered evaluator is not the new production authority.

Distinguish absent specification, unsupported capability, invalid input, singular derivative,
unconverged property state, rejected trial point, cancelled attempt, feasible point, acceptable
termination and verified local optimum. A status enum with only success/failure loses useful
solver meaning. A time or iteration limit may leave a useful incumbent; it must remain visibly
limited. No local NLP outcome implies global optimality.

## 4. Derivation and execution design

### 4.1 Recommended architecture

```text
authored process model + physical/property definitions
                 |
     typed templates, index binding, domain/smoothness obligations
                 |
        Symbolica arithmetic and local derivative bodies
                 |
     immutable problem structure + instance/case bindings
                 |
       +---------+-----------+------------+--------------+
       |                     |            |              |
 square equations       smooth NLP     LP/MILP/QP     conic problem
 KINSOL candidate       Ipopt C        HiGHS          Clarabel
 (or diffsol-nl)        POUNCE
       |
 dynamic models: Diffsol mass-matrix profile / SUNDIALS IDA profile

 property calls: FeOS where suitable; num-dual/provider derivatives
 structure: petgraph + qualified pounce-presolve operations
 sparse representation and diagnostics: faer; solver-owned factorizations
                 |
     physical results, diagnostics and provenance -> Arrow / Delta
```

These are problem-specific views of the same model. A DAE residual and a conic coefficient
matrix should not be squeezed into a TNLP-shaped universal trait. Share identifiers, physical
bindings, evaluation products, execution controls and publication contracts. Keep each solver's
native problem and option vocabulary available.

### 4.2 Library ownership and selection

Versions below are **inspected candidates**, not changes to workspace pins. They establish an
interface baseline, not a compiled compatible stack.

| Owner | Built-in functionality to use | Recommendation and boundary |
|---|---|---|
| Symbolica 3.0.0 | Atoms, differentiation, multi-output `ExpressionEvaluator`, `EvaluatorBuilder`, CSE/Horner optimization, `Dualizer`, optional JIT | Primary arithmetic engine. Use library differentiation/evaluation; compare symbolic derivatives and library AD for bounded local bodies. Keep physical typing and index semantics outside it. [Public evaluation APIs][symbolica-eval] |
| faer 0.24.4 | Sparse symbolic/numeric matrices, ordering and factorization, dense/sparse diagnostics | Own matrix representation and ordinary linear algebra. Avoid a custom Newton/KKT algorithm just because faer can factor its matrix. Native solvers choose their own linear solver. [CSC API/source][faer-csc] |
| Ipopt C, existing 3.14.20 project target | Native constrained NLP, exact Lagrangian Hessian or explicit limited-memory mode, warm starts and native status/options | First production NLP route; adapt the existing contained FFI boundary. Keep KKT inertia/globalization in Ipopt. [Interface][ipopt-api], [termination meanings][ipopt-output] |
| POUNCE 0.12.0 | Native Rust NLP and its TNLP interface | Second NLP route under U4. Qualify separately; a newer solver and a successful small example are not evidence of parity across process models. [Published crate][pounce] |
| SUNDIALS KINSOL, Rust integration candidate `sundials-sys` 0.6.2 | Newton with line search, fixed-point/Picard methods with Anderson acceleration, scaling, native direct/Krylov linear solver interfaces | Preferred maturity-oriented candidate for square initialization and recycle systems. Qualify the concrete C build and Rust wrapper. Sign constraints are not general box constraints. [Mathematics][kinsol-math], [API][kinsol-api] |
| `diffsol-nl` 0.1.1 | `NewtonNonlinearSolver` and `BacktrackingLineSearch` over library linear algebra | Smaller pure-Rust alternative to qualify if it serves the required initialization workloads. It already supplies globalization; do not reimplement it around faer. Select one default root solver rather than building two new backends immediately. [Published source][diffsol-nl] |
| HiGHS through `highs` 2.4.0 / its native bindings | LP, MILP, convex continuous QP, sparse matrices, solution guesses, native options/status | Direct route for extracted coefficients and exact tear-selection MILPs. Its QP route does not imply MIQP or arbitrary nonconvex QP support. [Rust model API][highs-model], [native scope][highs] |
| Clarabel 0.11.1 | Native conic solver, including SOC, exponential and power cones; PSD capability is feature-specific | Direct route for actual conic formulations. Preserve cone ordering and native matrix conventions. It is not a MILP solver. [Cone types][clarabel-cones] |
| FeOS / feos-core 0.10.1 | Supported EOS/property models, state calculations, phase equilibria and stability analysis | Add as a concrete property provider where model/data coverage fits. Use its thermodynamics rather than porting EOS, flash and derivative machinery. It does not establish complete IDAES property/reaction/transport coverage. [Crate][feos], [state][feos-state], [equilibrium][feos-equilibrium] |
| num-dual 0.15.0, or a deliberately aligned provider version | Generic dual-number derivatives, fallible explicit-function evaluation, implicit derivative helpers | Derivatives for project-specific kernels when no library property implementation exists. `ImplicitFunction` / `ImplicitDerivative` separate a real root solve from derivative recovery. Regularity and failure checks remain necessary. [Exact source][num-dual] |
| Diffsol 0.16.2 | Native Rust equation interfaces, BDF/Runge–Kutta families, mass-matrix problems, events and sensitivity facilities | First Rust dynamics candidate for supported ODE/index-1 mass-matrix models. Supply compiled sparsity; use native closures/traits instead of adding DiffSL as a second mathematical language. [API][diffsol] |
| SUNDIALS IDA/IDAS | General implicit DAE residual interface, consistent-initial-condition facilities, time integration; IDAS sensitivity facilities | Candidate when the model requires `F(t, y, ydot) = 0` beyond the chosen Diffsol profile. High-index models still need a justified reduction/formulation. [IDA mathematics][ida-math] |
| `pounce-presolve` 0.12.0 and petgraph 0.8.3 | Matching, DM/BTF, SCC/traversal; optional library presolve and postsolve facilities | Reuse defined structural algorithms. Keep topology, incidence and scheduling graphs distinct. Evaluate numeric presolve separately from exact structural analysis. [Presolve source][pounce-presolve] |
| Oximo 0.7.0 | Model construction, selected solver adapters and exports | Optional derived-model/export adapter when its actual API saves work. It is not required between an already-compiled coefficient matrix and a solver, and is not the authoritative nonlinear model. [Public solver contract][oximo-solver] |

Mature C/C++/Fortran solvers called through Rust are native execution. The design should prefer
the strongest practical library implementation, without imposing an all-Rust-internals rule.
Conversely, a new Rust crate is not automatically mature because it removes FFI. Qualification
must distinguish the algorithm, wrapper, feature set and deployment build.

### 4.3 Typed bodies, indexed structure and numerical compilation

Retain the original per-specialization design, but make its specialization rule explicit. A body
contains formal numeric slots and typed calls. Bindings expand finite domains and connect those
slots to global variables, parameters and row identities. An indexed sum needs semantics for
membership, filters, empty identity, duplicates and ordering; a binding table alone does not
supply them. Cross-unit equations and discretization stencils are reusable bodies too.

Prefer small scalar or bounded-block templates where repeated structure permits them. Where a
reduction genuinely changes the arithmetic shape, specialize on that shape or use a qualified
library reduction representation. Never silently emit one model-wide Symbolica tree to fill the
gap. Bound specialization count, body width, derivative output count and expansion size. The
claim that adding an instance recompiles zero bodies applies only when it adds no new shape or
provider specialization.

Extract obligations before normalization, but execute them in dependency order. For
`log(1/x)`, the check involving `1/x` must not run before the check on `x`. An untaken branch must
not evaluate its property call or an invalid arithmetic subexpression. The existence of a
Symbolica conditional facility does not prove this behavior survives CSE, vectorization and
JIT. Qualify each enabled execution mode. If the public API cannot preserve the required branch
semantics, compose guarded library-compiled blocks or reject that mode; do not resurrect a
general scalar interpreter.

Compile callback-demand groups: values, Jacobian, and Lagrangian Hessian. Multi-output evaluation
can share work within a group. Computing a Hessian on every line-search value callback can erase
the advantage of library compilation. `ExpressionEvaluator::merge` and `vectorize` are built-in
options to investigate, not reasons to write another optimizer or AD engine. JIT is an optional
execution optimization after the interpreted library path meets the same contract. [Evaluator
API][symbolica-eval], [Dualizer API][symbolica-dual]

### 4.4 Sparse assembly, aliasing and derivatives

For a local body, write the binding mathematically as `z_i = G_i x + B_i p + c_i`, with a row
map `A_i`. `G_i` may repeat a global column and may contain constant conversion/scaling factors.
The assembled Jacobian is the sum of `A_i J_i G_i`; the Lagrangian Hessian is the sum of
`G_i^T H_i G_i`, using the correctly mapped multipliers and objective factor. Composition-dependent
conversions are nonlinear provider/model operations, not constant entries in `G_i`.

The important adversarial case is `f(a,b) = a*b`, with both `a` and `b` bound to the same global
`x`. The result is `x^2` and its second derivative is 2. A scatter implementation that retains
only one local triangular entry and then collapses duplicate global coordinates can return 1.
Specify multiplicities, symmetry and triangularization before choosing the storage layout.
Likewise, repeated row contributions require summation, not last-write-wins assignment.

Use faer's `SymbolicSparseColMat::try_new_from_indices` and the returned `Argsort`, and
`SparseColMat::new_from_argsort`, where their construction contract fits. The 0.24.4 API already
owns index ordering. A stable refill/scatter map can still be necessary for allocation-free
callbacks, but it is a derived adapter, not another sparse-matrix implementation. Its exact
duplicate-sum behavior and steady-state allocation cost need qualification. [Exact CSC
implementation/API][faer-csc]

Structural sparsity must cover every admitted value and branch of that compiled body. A zero
coefficient or derivative at the initial iterate is not structural absence. Distinguish the
numeric Jacobian support used for matching from guard/provider dependencies used for validation
and invalidation. Active-branch observations cannot silently prune the former for all future
iterates.

Ipopt's exact Hessian is that of the **Lagrangian**, including its supplied objective factor and
constraint multipliers. Cache identity cannot be only the primal vector: multipliers and the
objective factor can change at the same `x`. Respect its lower-triangular structure query and
numeric callback contract. Limited-memory mode remains an explicit valid profile for supported
first-derivative models; claiming exact second derivatives requires every reachable kernel to
support them. [Ipopt callback contract][ipopt-api]

### 4.5 Thermodynamics and property kernels

Treat property packages as a first-class library integration, not a placeholder named
“num-dual kernel.” A package contract includes component/data identity, supported phases, state
variables, valid ranges, quantity/reference conventions, selected physical branch, derivative
order, failure taxonomy and workspace ownership. A single thermodynamic state may produce many
properties. Compute that state once per valid evaluation context; avoid a separate flash for
each scalar output or partial derivative.

FeOS is a useful concrete target because it owns substantial thermodynamic mathematics already.
Its state and phase-equilibrium APIs offer more than an EOS residual. Stability and phase
selection must be part of the provider contract: a converged algebraic root need not be the
physical equilibrium branch. Validate specific models and parameter data against independent
physical references. Keep unsupported reactions, transport laws and other packages explicit;
do not translate “has a thermodynamics library” into complete simulator coverage.
[State API][feos-state], [phase-equilibrium API][feos-equilibrium]

For an implicit kernel, either expose the underlying residual equations to the flowsheet solver,
or let a library solve the internal problem and recover derivatives at the converged regular
root. num-dual supplies `implicit_derivative`, `implicit_derivative_vec` and
`ImplicitDerivative`; do not create a new generic implicit-differentiation engine. These helpers
do not establish a well-conditioned unique root or safe singular behavior: the inspected vector
implementation contains an unchecked factorization unwrap. Check the regularity/failure
contract and test the singular case before using it behind FFI. Do not differentiate a variable
number of convergence iterations and assume that is the derivative of equilibrium. [Exact
num-dual 0.15.0 implicit implementation][num-dual-implicit]

There is an unresolved integration seam in the original proposal. Symbolica's public
`ExternalFunction<T>` returns `T`, not a typed `Result`; `ExpressionEvaluator::try_evaluate`
returning an evaluator error does not by itself translate a thermodynamic provider failure.
num-dual's fallible explicit-function support does not change that callback type. Before broad
implementation, demonstrate one fallible, multi-output property kernel through values, first
and second derivatives, branch suppression, evaluator cloning and every enabled JIT mode.
The adapter may need explicit evaluation-scoped error storage or guarded block composition;
its ownership and recovery semantics must be specified. Never hide errors in a global side
channel or report NaN as an ordinary property value. [External function API][symbolica-external],
[registration API][symbolica-registration], [num-dual explicit API][num-dual-explicit]

### 4.6 Initialization, presolve and solver lifecycle

The simulator chooses a block/continuation/tear strategy and supplies residuals, derivatives,
scales and starts. A library should own Newton updates, line searches, convergence tests and
linear solver interaction. KINSOL is the preferred candidate when its square-system contract
fits; `diffsol-nl` is a concrete pure-Rust alternative. Cases needing general bounds or inequality
constraints can use the constrained NLP route. Do not silently clamp trial points or turn an
infeasible root problem into an apparently solved least-squares minimization.

Separate exact structural analysis from optional numeric presolve. `pounce-presolve` exposes
more than matching/DM/BTF: affine elimination, bound tightening and solution-recovery machinery
may remove further bespoke code. However, FBBT requires its `ExpressionProvider`/`FbbtTape` view;
a numerical TNLP callback alone silently supplies no expression information. This is a derived,
loss-aware provider adapter for a supported subset, not a reason to adopt a second authoritative
math IR. Disable/refuse unsupported transformations and record which actually ran.
[Expression-provider contract][pounce-expression], [presolve wrapper][pounce-presolve]

A perfect structural matching does not prove numerical rank at a particular state. The
equations `x+y=0` and `2*x+2*y=0` have a full matching but a singular Jacobian. Use library
factorization/diagnostic facilities with explicit scaling, finiteness and residual checks;
do not report structural rank as numerical nonsingularity.

Its numeric elimination also has coefficient/equality tolerances, and its transformation
fingerprint does not cover every expression-tape change. Those policies and dependencies must
be explicit. Reuse its primal/multiplier recovery where qualified, while verifying the original
model after postsolve. Do not interpret a raw contradiction flag as a physical infeasibility
certificate; the inspected API distinguishes `certified_infeasible` from other detections.
[Elimination policy][pounce-elimination], [presolve API][pounce-presolve]

For tear selection, the graph skill confirms that neither the pinned petgraph nor rust-igraph
offers an exact minimum **directed** feedback arc set. Their applicable routines are heuristics.
Use a declared heuristic when sufficient, or a HiGHS MILP formulation when minimum-cost tears
matter. Preserve the edge projection, weights, cycle-breaking witness, incumbent and optimality
gap. Removing rust-igraph is reasonable only after its remaining consumers and required
semantics are replaced; a single matching benchmark is not that proof. [petgraph
algorithm][petgraph-fas], [rust-igraph algorithm][igraph-fas]

Solver state belongs to one attempt. Construct `!Send` solver objects on the worker that uses
them, keep callback storage alive until native teardown, contain panics at FFI boundaries, and
retain concurrency permits until the native work has actually exited. Cancellation requested,
cancellation acknowledged, trial evaluation rejected and terminal failure are different events.
Preserve the original review's recovered-trial-error repair, and also distinguish Ipopt's
optimal, acceptable and feasible-only statuses. Recompute physical residuals against the
original model before publishing solution quality. [HiGHS model auto traits][highs-model],
[Ipopt status definitions][ipopt-output]

Scaling, permutation, fixed-variable elimination and presolve each require an explicit inverse
for reported values and applicable duals. Warm starts carry the structure, scale and backend
identity they require, including primal/dual/basis distinctions. A reusable Symbolica body does
not imply a reusable solver factorization, basis or multiplier vector.
For a ranged constraint, retain its evaluated value and bounds and compute the physical bound
violation; the raw native `g(x)` value is not itself a residual or evidence of infeasibility.

### 4.7 Linear, conic, dynamic and advanced process problems

For affine/quadratic models, use Symbolica's library polynomial capabilities or known typed
template structure to derive coefficients. Feed those coefficients directly to HiGHS, using
its fallible builders/options and `try_pass_hessian`. Degree two is not proof of convexity, and
HiGHS does not fully certify positive semidefiniteness at that boundary. Require a valid
construction/assumption or choose a compatible nonconvex route. A parameter edit that invalidates
that assumption must invalidate routing. [HiGHS model/Hessian contract][highs-model]

For conic models, preserve the explicit cone formulation and use Clarabel directly. This exposes
exponential/power capabilities beyond a generic “SOCP” label. Its matrix-update facility is
conditional: dimensions, sparsity and cones must remain compatible, and the documented update
path excludes enabled presolve/chordal decomposition. Do not promise factorization/update reuse
for every case edit. Oximo is useful only where its model transformations or exports demonstrably
reduce adapter work; public `Solver::solve(&Model, ...)` is not an external derivative-oracle
interface. [Clarabel updates][clarabel-update], [Oximo contract][oximo-solver]

For dynamics, separate simulation from simultaneous dynamic optimization. Diffsol's native
equation interface can consume the same compiled residual/derivative products for its supported
mass-matrix form. Use its time-stepping, events and sensitivity machinery instead of writing
an integrator. Supply known sparsity: its documented NaN-based detection can miss behavior
behind control flow. Do not claim support for arbitrary implicit or high-index DAEs through
this route. IDA is the candidate for the broader implicit residual form; qualification must
cover consistent starts, algebraic constraints and events. Dynamic optimization additionally
needs a stated transcription/discretization strategy; a time integrator alone does not replace
that missing model transformation. [Diffsol API][diffsol], [IDA][ida-math]

Removing Pyomo also removes assumed access to GDP transformations, MINLP orchestration,
parameter-estimation utilities and robust-optimization workflows. Map each actual product
requirement to a native formulation or mark it unimplemented. Parameter estimation can first
use the existing native NLP path; sensitivities and uncertainty reports need their own
statistical and regularity contracts. Native SCIP, through a qualified Rust binding/FFI, is a
candidate for factorable MINLP and discrete logic. Its global methods need structural nonlinear
information and valid bounds, not merely a black-box local derivative oracle. This is a later
capability package, not a foundational dependency. [SCIP supported problem classes][scip]

Production Python remains a convenient model/result interface if desired, calling the Rust
compiler and solvers. It must not construct Pyomo models, invoke Python numerical kernels or
depend on Pyomo to initialize a flowsheet. Keep IDAES/Pyomo only in the isolated external parity
environment where the comparison itself requires them. NL/LP/MPS export is an optional
interchange consumer, not the native execution path or an early acceptance prerequisite.

### 4.8 Compilation, incremental reuse and resource ownership

Salsa should track semantic inputs, typed specializations, immutable library preparation
descriptions and structure analyses. It should not own mutable solver/evaluator workspaces.
Separate artifact construction with explicit resource ownership from pure semantic derivation
where JIT, native allocation or worker lifecycle is involved. This can use the existing runtime;
it does not require a new compilation service or plugin framework.

Artifact identity includes the exact library/profile, target features, numeric policy and
provider registrations that affect it. Process-global Symbolica registration must be bounded
and deterministic; slot reuse does not bound an unlimited stream of version-tagged kernel
symbols. An LRU on Salsa queries also does not reclaim that global registry. Use a finite
session/provider registration policy and explicit failure or session rotation at its bound.

Use Symbolica's existing `OptimizationSettings` controls, including `cores`, `abort_check`,
`horner_iterations`, `cpe_iterations` and common-pair cache limits, before adding bespoke
compiler scheduling machinery. Bound the total nested thread budget across compilation,
instance evaluation and native solvers. Large derivative expansion must fail explicitly rather
than silently truncate the model. [Optimization settings][symbolica-optimization]

### 4.9 Per-stage computation contracts (RCA §9)

The common rule is: semantic IDs survive publication; physical indices do not. Equality is
structural/semantic for preparation and tolerance-based for numerics. No solver attempt is a
tracked query. Cycles in a physical flowsheet are admissible; cycles in a purported acyclic
evaluation schedule are an error.

| Stage | Meaning, representation and backend | Dependencies and tracked boundary | Cycles, cost and limits | Ownership, cancellation, equality and publication |
|---|---|---|---|---|
| T1 — physical specialization | Typed body, obligations and source map; project domain logic | Salsa reads definition, membership/absence, units, phase policy, providers and structural parameters | Finite binders; bound shape, body width and specialization count | Immutable; cancellable preparation; semantic equality; inspectable summaries only |
| T2 — library derivation | Symbolica body, local derivative support and evaluation specification | Per-specialization query; exact library/registration/numeric profile | Library optimization with declared limits; no global instance expansion | Immutable specification; artifact equality separate from pointer identity; no durable Salsa state |
| T3 — artifact construction | Library evaluator/JIT for each demanded callback group | Derived specification plus target/profile; effect-owned cache outside mutable query state | Bound compile concurrency, optimization and code memory | Per-worker mutable evaluator; cancellation retains ownership until exit; tolerance-equivalent numerical modes |
| T4 — binding and assembly | Slot/row maps, faer pattern and graph projections | Structure query observes instance/domain membership, coupling, fixedness, support and ordering policy | Cost includes instances × slots and assembled nonzeros; validate indices and duplicate rules | Immutable structure; deterministic mapped IDs; publish derived incidence/blocks with spec/version |
| T5 — structure/initialization | Exact matching/DM/BTF; chosen tear strategy; library NLE | Structural analysis tracked; case-sensitive numeric presolve and starts are attempt inputs | SCCs represent genuine coupling; heuristic versus exact tear contract explicit; bounded solver work | Analysis immutable; initialization workspace attempt-local; publish convergence and projection diagnostics |
| T6 — solve/integrate | Ipopt/POUNCE, HiGHS, Clarabel, KINSOL/Diffsol/IDA according to class | No Salsa attempt memoization; reads case, full profile, provider data, warm start, capabilities | Iteration/time/event limits; DAE temporal semantics and phase events explicit | Native worker owns mutable state; cancellation acknowledged only after exit; typed termination and numeric quality |
| T7 — postsolve/publish | Original physical values, residuals, applicable duals and provenance | Reads exact transformations and attempt snapshot | Cost proportional to required validation/output; no partial-success truncation | Explicit coherent publication; retries follow existing publication contract; failed/partial outcomes distinguished |

Arrow/Delta remain useful at authored-input and result boundaries, and DataFusion for genuinely
set-oriented model/data work. These stages do not require relational transport of expression
nodes or solver iterations. That departure from the old execution design is deliberate.

## 5. Representative and adversarial journeys

| Journey | Expected target behavior | What it attacks |
|---|---|---|
| Add an indexed component balance and connect two units | Type once for the relevant shape; bind component membership and connection slots; assemble rows with stable IDs; solve and report each balance | An instance table being mistaken for complete index/reduction semantics |
| Reuse two formal inputs as one global variable | `a*b` with `a=b=x` produces value `x^2`, derivative `2*x`, Hessian 2; a row permutation changes no physical result | Duplicate scatter and premature triangularization |
| Evaluate `if x > 0 then log(x) else 0` at a negative value | Return the inactive branch value without invoking `log`; preserve this through CSE and enabled JIT/vector modes | A guard checked after an unsafe operation has already executed |
| Evaluate nested domains and derivative boundaries | `log(1/x)` at zero reports the first invalid dependency; `sqrt(x)` at zero has a valid value but cannot claim a finite smooth derivative | Conflating admissibility, obligation evaluation and solver smoothness |
| Add a FeOS-backed flash/property package | Bind component data/reference conventions, solve/select a physical state, reuse its multiple outputs, propagate required derivatives and failures | Repeated hidden property solves; assuming any converged root is the physical phase |
| Approach an implicit-kernel singularity or phase boundary | Report invalid/ill-conditioned derivative or explicit branch event; never unwind across native callbacks | AD helpers being treated as a conditioning or phase-selection guarantee |
| Change one value, then component membership, then a provider version | Value refill preserves valid structure; new shape and provider semantics invalidate exactly the necessary preparation; compare with clean compilation | Missing membership, absence or global-registration dependencies |
| Run a recycle flowsheet from a poor initial point | Library nonlinear globalization handles the iterations; domain-rejected trials are events; final physical feasibility is independently checked | A custom Newton loop hidden behind a faer adapter |
| Solve a quadratic model, then change a parameter so it is indefinite | Reclassify or reject the convex-QP route; never send unsupported curvature with an “optimal” promise | Degree-based routing and stale convexity assumptions |
| Presolve a model with eliminated variables and scaled equations | Recover full physical values and meaningful duals; reevaluate original bounds and balances; invalidate on expression changes | Reduced-space outputs being published as original-model results |
| Cancel a solve during a property callback or native factorization | Record request promptly; hold buffers and permits until work exits; publish one typed attempt outcome | Detached native work, unsafe teardown or premature capacity release |
| Simulate a dynamic vessel with algebraic constraints and a valve event | Consistent initialization, declared event/reset behavior, bounded error and preserved balances under the qualified DAE profile | Treating a generic ODE callback as complete process dynamics |
| Install and solve through the public Python interface without Pyomo | Python passes typed data to Rust; no Python modeling or numerical fallback; results preserve physical identity | A nominal native backend whose production workflow still needs Pyomo |

These are proposed tests and traces, not executed results from this review. In particular,
branch-laziness and external-function JIT compatibility have not been established by API
inspection alone.

## 6. Acceptance gates

These verdicts assess the **original proposed target**, not a new runtime implementation. A
design-level pass establishes an adequate stated mechanism; it does not mean that mechanism
has been implemented or tested. The follow-up recommendations name how to close each gap.

| Gate | Verdict | Independent evidence | Required action |
|---|---|---|---|
| G1 — Authority | **Pass at design level (Proposed)** | Original §§2 and 4 assign authored meaning to definitions, arithmetic to Symbolica, and remove both bespoke derivative engines rather than maintaining competing production routes | Preserve this ownership and deletion requirement; keep provider/export views derived |
| G2 — Semantic fidelity | **Unresolved** | Original §§3–4 require guarded obligations and scatter maps but do not decide safe obligation scheduling, aliased-slot Hessian assembly or fallible implicit-kernel composition | N01–N04; §3 and §4.3–§4.5 |
| G3 — Validity | **Unresolved** | Admission is proposed, but value-domain validity, C1/C2 suitability, implicit-root regularity and convex-class eligibility are not fully specified | N05; capability-specific admission and negative tests |
| G4 — Hidden behavior | **Unresolved** | Original S4 labels evaluator/JIT construction pure; its resource/target boundary and per-attempt external-function failure channel are not settled | N03, N06; declare ambient inputs, native artifact ownership and error effects |
| G5 — Consistency and recovery | **Unresolved** | Original F6 repairs stale evaluation failure, but recovery of scaled/presolved results and warm-start identities is not specified sufficiently for the enlarged target | N07, N11; original-model postsolve and lifecycle tests |
| G6 — Transformation and reuse | **Unresolved** | Original §9.3 proposes reuse tests; it does not settle shape-dependent specialization, parameter-sensitive classification, registry bounds or complete presolve invalidation | N04, N06, N11; reuse-versus-clean adversarial edits |
| G7 — Truthful capability claims | **Fail for one explicit claim; otherwise unresolved** | Original F9 calls for MILP parity across HiGHS and Clarabel, although Clarabel is not a MILP solver. Original §11 also treats every target gate as passed while the above composition decisions remain open | Correct class-specific verification; distinguish candidate/API support from admitted, implemented and qualified simulator support |

The result supports **Revise**, while fully supporting the hard-pivot direction. It does not
support retaining the custom engines until every future capability is implemented.

## 7. Principle findings

Each verdict applies to the cited requirement in the reviewed proposal. A recommended extension
is marked as a leverage opportunity rather than misrepresented as an existing correctness bug.

| Finding and verdict | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **N01 — Unresolved: obligations do not yet define a safe evaluation schedule** | DM-07, DM-22, DM-24; RCA §3, §7 | Original §3 gives guarded obligations and §9.3 lists branch tests, but S4 does not state whether CSE/evaluation can execute an unsafe subexpression before its guard | An untaken `log`/flash fails, or checking `log(1/x)` itself divides by zero before the intended diagnostic | Dependency-order obligations and qualify lazy guarded library evaluation per mode; §4.3 | Branch, nested-domain and CSE/JIT cases in §5 |
| **N02 — Unresolved: sparse assembly lacks an aliasing and symmetry contract** | DM-22, DM-24, DM-41 | Original S5/S6 name binding/scatter maps without the repeated-global-slot Hessian rule | `a*b`, with `a=b=x`, can lose one mixed derivative and produce Hessian 1 | Specify gather/scatter algebra and duplicate summation; let faer own storage ordering; §4.4 | Exact polynomial oracle, repeated slots/rows, permutation and scaling tests |
| **N03 — Unresolved: a kernel registration is not yet a complete property integration** | DM-04, DM-08, DM-29, DM-44 | Original §5.1 describes a num-dual body plus registration. Inspected Symbolica `ExternalFunction<T>` is infallible; implicit kernels also require a regular physical root and derivative-order policy | Provider errors lose attribution; a global error slot crosses attempts; an internal solve is repeated for every property; derivatives follow the wrong branch | One explicit provider contract; use FeOS/library derivatives and qualified implicit helpers; prove the fallible multi-output bridge first; §4.5 | Real property kernel with recoverable/terminal failures, first/second derivatives, cloning and branch/JIT cases |
| **N04 — Unresolved: binding tables do not replace all indexed/discretized model semantics** | DM-09, DM-17, DM-18, DM-31; RCA §4 | Original §4.4 says bindings replace missing P12 and adding a unit recompiles no body; no complete reduction/shape contract accompanies it | A new component changes a sum's arity without recompilation, or scalar expansion recreates a huge monolithic body | Finite-domain/reduction semantics, bounded shape specialization and reusable coupling/stencil bodies; §4.3 | Empty/ragged/filtered sums, new shape, shared components and cross-unit coupling |
| **N05 — Unresolved: solver admissibility needs more than function availability** | DM-07, DM-19, DM-40, DM-43 | Original registrations do not establish smooth neighborhoods or implicit regularity; original class routing does not identify a convexity proof/assumption | `sqrt(0)` reaches an exact-Hessian solver; an indefinite quadratic reaches a convex solver; a phase boundary appears smooth | Capability analysis includes C1/C2, curvature, integer/cone/DAE structure and provider conditions; §3, §4.7 | Boundary, singular-root, indefinite-QP and unsupported-class rejection tests |
| **N06 — Unresolved: artifact construction and reuse ownership remain underspecified** | DM-28, DM-29, DM-31, DM-32; RCA §4, §7, §8 | Original S4 is pure but returns mutable interpreted/JIT evaluators; fixed slot names do not bound arbitrary versioned kernel registrations | Target features or provider changes reuse an incompatible artifact; concurrent attempts share workspace; global registrations outlive memo eviction | Pure semantic specification, owned artifact construction and worker-local evaluation; bounded registration policy and built-in compiler limits; §4.8 | Concurrent attempts, library/profile changes, compile cancellation and registry-bound tests |
| **N07 — Unresolved: result meaning requires inverse transformations** | DM-08, DM-14, DM-30, DM-46 | Original §3 already requires status fidelity and F6 resolves the failure latch; §§4.7/5.4 do not define scaled/presolved dual recovery or warm-start mapping | A scaled multiplier is attributed to the physical balance, or a warm start uses old row order; the current 0/1/6 grouping also remains a concrete implementation repair | Preserve the proposed status fidelity and add original-model postsolve, quality and warm-start compatibility; §4.6 | Recovered trial, feasible-only/limited exits, scaled eliminated model and dual checks |
| **N08 — Violated as a library-first design objective: faer still leaves a generic nonlinear algorithm to us** | DM-38, DM-56, DM-58 | Original S7/§4.6 chooses faer for Newton; no library owns the iteration/globalization | The pivot deletes a scalar interpreter but creates another numerical algorithm and convergence-policy maintenance burden | Qualify KINSOL or `diffsol-nl`; keep only process-specific initialization strategy; §4.2/§4.6 | Difficult scaled recycle, poor start, singularity and recoverable domain failure against the chosen library |
| **N09 — Unresolved leverage: mandatory Oximo adds a model layer without a demonstrated consumer benefit** | DM-23, DM-41, DM-56, DM-58 | Original S7 routes all LP/MILP/QP/SOCP through Oximo; direct HiGHS/Clarabel APIs already accept the needed numeric representations | A coefficient model is rebuilt in another algebra layer and available native capabilities/options may be narrowed | Default to direct adapters for existing coefficient/cone forms; retain Oximo only for a qualified transformation/export consumer; §4.7 | Compare adapter surface, admitted capabilities and results on identical problems |
| **N10 — Unresolved: candidate version combinations are not integration contracts** | DM-41, DM-43, DM-59 | Exact manifests: feos-core 0.10.1 requires num-dual `^0.14`; Clarabel 0.11.1 optional faer-sparse uses faer `^0.21.9`; project commentary mentions FeOS with num-dual 0.15 | A generic `DualNum` or matrix adapter assumed to be shared fails to compose; unintended native/default features enter the build | Explicit provider feature profiles and type boundaries; §9.2 | Locked minimal composition builds, feature tree and concrete cross-boundary examples |
| **N11 — Unresolved leverage and validity: presolve can be reused only with complete inputs and postsolve** | DM-07, DM-22, DM-31, DM-42 | Original §8.3 considers matching/DM/BTF. Exact POUNCE APIs also offer reductions/FBBT but require expression tapes; tapes can be invisible to its fingerprint | Advertised FBBT silently does nothing, a stale reduction is reused, or reduced-space results are published | Use built-in presolve for a qualified subset, explicit policy/dependencies, and library postsolve plus original-model validation; §4.6 | Unsupported tape, expression-only edit, near-zero coefficient, full-space primal/dual recovery |
| **N12 — Unresolved: the functional replacement for Pyomo extends beyond an NLP callback** | DM-04, DM-43, DM-59 | Original removes Pyomo and sketches Diffsol later; blueprint §13.6, §18.7 and §19 also describe dynamic/discrete/estimation workflows | “Native process simulator” is declared complete after scalar steady-state solves while required dynamics or discrete features remain absent | Publish a capability roadmap with native routes and explicit unscheduled scope; §4.7 and §11 | The representative process journey plus distinct dynamics/discrete acceptance packages |
| **N13 — Violated: the blanket all-gates-pass and cross-class verification claims outrun the proposal** | DM-43, DM-53, DM-59 | Original §11 declares every target gate passed; original F9 specifies LP/MILP parity across HiGHS and Clarabel | Acceptance can be reported using a test matrix one backend cannot execute and contracts not yet decided | Replace with §6; compare only shared classes, and test refusals separately | Generated capability-to-test matrix with no unsupported advertised route |
| **N14 — Unresolved: a complete acceptance oracle must be independent enough to catch shared compiler errors** | DM-39, DM-48, DM-53, DM-54 | Original §9.3 uses derivatives/solver agreement and refers to synthetic/historical baselines; both native solvers would consume the same generated mathematics | Both solvers agree on an incorrectly assembled balance or Hessian; an evaluator speedup is mistaken for flowsheet speedup | Add analytic/physical oracles, metamorphic assembly and full cold/warm process cost; §9 | Known exact derivatives, independent physical references, conservation and complete cost receipts |
| **N15 — Satisfied direction: remove competing production math and Python modeling ownership** | DM-02, DM-23, DM-38, DM-56; RCA §1, §7 | Original §§2, 4 and 11 explicitly replace the bespoke IR/evaluators and Pyomo path, keeping physical typing before library algebra | Preserving the old path would retain contradictory derivatives and double the migration/maintenance surface | Keep the hard deletion boundary; do not add compatibility execution or a universal replacement IR | Deletion inventory, dependency/import checks and source-to-native-solve acceptance |

**Applicability.** Groups 1–3 govern physical meaning, authority and identity; groups 4–5 govern
templates, index binding and library lowering; groups 6–7 govern native workspaces and reuse;
groups 8–9 govern algorithm choice, numerical contracts and providers; groups 10–12 govern
traceability, evidence and architectural cost. DM-45 is relevant to accepted executable/provider
inputs, but remote plugin trust was not reviewed. DM-51 applies to explicit contract-version
changes and rejection of incompatible artifacts; the user has not requested legacy data
migration or compatibility execution, and this review recommends neither. RCA §§1–4, §5's
graph projection/algorithm contracts, §6's cycle semantics, and §§7–9 apply. Community detection,
ranking and temporal graph analytics do not serve these math stages and are not required.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness/operational risk | Cost and performance evidence | Decision |
|---|---|---|---|---|
| Current custom math plus planned Pyomo integration | Retains project arithmetic, derivative, representation and backend semantics | Existing scalar/index/kernel limitations and duplicated rules identified in the original review | No new baseline measured here; historical costs motivate change but do not certify a replacement | Reject as the go-forward design |
| Original Symbolica/faer/Ipopt/POUNCE/Oximo target | Deletes the largest custom math surface; preserves physical typing and local bodies | Leaves root algorithms, property composition, indexed assembly and some solver capabilities underspecified | Prior measurements concern specific probes; target integration remains unqualified | Retain the core direction, revise as above |
| **Simpler viable first delivery: Symbolica + existing Ipopt C + faer, one real property provider** | One native steady-state path, exact same intended source/binding model; no duplicate compatibility engine | Bounds and equation solving use the qualified NLP route initially; some initialization/dynamic capabilities remain explicitly absent | Smallest coherent production transition; does not promise this is the fastest NLE path | Preferred first vertical slice; adding KINSOL, POUNCE and other classes follows concrete consumers, with U4 preserved as the NLP target |
| **Recommended complete target: library-owned problem-specific algorithms** | Same typed frontend and local math; native root/NLP/linear/conic/dynamic interfaces; providers own thermodynamics | More integration boundaries, each with explicit capability and lifetime contracts | Greater library reuse; no end-to-end speed or memory claim until qualification | Select as the target, stage by simulator functionality |
| Oximo as universal model/solver frontend | One additional general model must represent all physical/lazy/kernel/derivative semantics | Its inspected public contract does not carry the required external NLP oracle/opaque operations | Removes adapter code only if its actual supported surface matches the consumer | Do not make universal; retain only justified optional uses |

The justified abstractions are a typed process frontend, reusable local arithmetic bodies,
explicit problem bindings, property packages, small native adapters and coherent result
publication. They exist because several concrete consumers share those meanings. A global
solver plugin marketplace, universal derivative IR, generic algebra interpreter, custom
autodiff engine, custom sparse factorizer and new cache service are not required.

Process-specific ordinary code remains: authoring validation, unit and basis rules, component
and phase binding, model templates, semantic assembly, initialization strategy, physical result
interpretation and diagnostics. The objective is to minimize generic mathematics we implement,
not to pretend process-specific engineering decisions can disappear into a library.

Do not couple this pivot to a broad Arrow row-codec rewrite or memory-pool framework. The prior
review's `serde_arrow` and allocator/pool suggestions can be reassessed after deleting math
transport, when the surviving workload is known. Likewise, do not add egglog or another rewrite
system beside Symbolica without a demonstrated physical-model transformation it must own.

## 9. Verification and measurement plan

### 9.1 Evidence required for the target

All rows are **Proposed** acceptance work unless explicitly marked Interface-checked. None is
reported as Tested or Measured by this follow-up review. Unit-level contract qualification and
the eventual full process/solver campaign are different scopes; avoid repeatedly running a
full campaign after each deletion package.

| Claim/risk | Evidence now | Check and conditions | Acceptance meaning |
|---|---|---|---|
| Public library routes exist | Interface-checked | APIs/manifests referenced in §4 and §9.2 | Suitable candidates for a composition spike, not executable integration proof |
| Guarded real-algebra lowering | Proposed | Negative branch/nested-domain cases plus admissible extreme-value cases, across interpreted and each enabled JIT/vector mode | No lost domain failure, unsafe untaken evaluation or unexplained non-finite result |
| Local/global derivatives | Proposed | Analytic polynomials including aliased slots, numeric directional checks at several step sizes, and backend derivative diagnostics where exposed | Correct values/Jacobian/Lagrangian Hessian; no universal fixed finite-difference tolerance masking conditioning |
| Provider integration | Proposed | One real flash/property case, implicit singularity, multi-output call counts, typed recoverable/terminal failure, first/second derivative modes | Library property mathematics survives the entire native callback boundary |
| Index/shape semantics | Proposed | Empty and ragged domains, filters, coupling, new specialization, fixed/free changes and discretization stencil | Compiled equations equal authored finite-model meaning, with bounded expansion |
| Classification | Proposed | LP/MILP on HiGHS; common LP/convex-QP cases across capable backends; cones on Clarabel; negative MIQP/nonconvex/opaque-conic cases | Each advertised class is implemented and unsupported cases fail before solving |
| Native status and postsolve | Proposed | Rejected trial then convergence, feasible-only exit, limited incumbent, singular initialization, scaling/presolve recovery | Correct termination and original-physical-model quality, including meaningful dual conventions |
| Incremental reuse | Proposed | Compare warm reuse with clean compilation after value, membership, absence, provider, phase and profile edits | Same semantic/physical result; expected body/structure rebuilds, with no required claim of zero recompiles after shape changes |
| Cancellation/resource limits | Proposed | Compile abort, callback cancellation, slow native teardown, concurrent attempts and global registration bound | No shared mutable state, stale success, dangling buffers, released-too-early permit or truncated model |
| First product acceptance | Proposed | Indexed heater/flash/separation flowsheet with property calls, mass/mole conversion, a recycle, initialization, optimization and publish/reload | Physical conservation and independently checked values; public Rust/Python route works without production Pyomo |
| Dynamics acceptance | Proposed | Index-1 constrained vessel, consistent start, valve event and sensitivity check under one qualified profile | Correct time/error/event semantics; no claim about untested high-index/discrete dynamics |
| No obsolete production path | Proposed | Source/dependency/generated-contract inventory after replacement, regenerated from declarations | No bespoke derivative/interpreter or Pyomo fallback hidden behind the new interface |
| End-to-end performance | Proposed | Cold and warm compile/bind/initialize/solve/postsolve/publish, fixed features/threads/CPU profile and representative scales | Reproducible phase costs, peak memory and useful results; evaluator timing alone is not acceptance |

Independent-looking solvers sharing one compiler are not independent mathematical oracles. Use
closed-form small models, physical balance identities and independent property data where
appropriate. Check local derivatives with analytic/AD/directional evidence suited to the
function, rather than defining correctness as agreement with the old engine. Finite differences
near singularities and phase boundaries need an explicit error/conditioning interpretation.

Capture build and runtime costs separately: library feature/build size, template preparation,
JIT, binding/scatter construction, native matrix conversion, property evaluations, initialization,
solver iterations, original-model validation and publication. Include both repeated identical
units and genuinely different shapes; cold/warm cases; finite versus difficult starting points;
and the actual thread budget. Store future probes, lockfiles, inputs and receipts in the
repository's evidence workflow. The original review's session scratch measurements remain
historical evidence; this review does not make them newly reproducible or end-to-end.

### 9.2 Exact dependency and feature issues found

| Inspected package/profile | Observed contract | Design consequence |
|---|---|---|
| feos-core 0.10.1 | Normal dependencies include num-dual `^0.14`, nalgebra `^0.35` and quantity `^0.14` | Correct the proposed num-dual 0.15 compatibility assumption. Either qualify an aligned version or keep provider types private and cross a numeric/derivative-result boundary. Two releases' `DualNum` traits are not interchangeable. [Manifest][feos-manifest] |
| num-dual 0.15.0 | Current `DualNum`/`ImplicitFunction` APIs differ from older indexed examples; vector/Hessian helpers are feature-sensitive | Compile against the exact crate and selected features. The 0.15 implicit helpers are not a fallible-root guarantee; §4.5. [Manifest/source][num-dual] |
| Diffsol 0.16.2 | Depends on `diffsol-la` and `diffsol-nl` 0.1.1-compatible lines, not identically numbered 0.16.2 crates | Pin/lock the actual family and test its resolved composition. [Manifest][diffsol-manifest] |
| diffsol-la 0.1.1 | Native target dependency uses faer `^0.24`; nalgebra `^0.35` also exists; feature disabling is not proof all alternative algebra dependencies disappear | The faer line is a promising fit, but inspect the resolved feature tree rather than asserting a faer-only dependency graph. [Manifest][diffsol-la] |
| Clarabel 0.11.1 `faer-sparse` | Optional faer `^0.21.9` and faer-traits `^0.21.5` | Prefer its ordinary solver-owned linear algebra unless an optional backend is justified. Do not force Clarabel matrices/factors into the project's faer 0.24 type universe. [Manifest][clarabel-manifest] |
| `sundials-sys` 0.6.2 | Build logic can discover a system SUNDIALS or build bundled libraries; module/features and detected native version matter | A Rust crate pin alone does not pin the actual C ABI. Qualify native version, scalar/index widths, modules, linking and diagnostics. [Build logic][sundials-build] |
| Symbolica 3.0.0 | The prior review identified process-wide allocator/tracing defaults and code-generation profile implications | Preserve explicit feature and initialization settings; independently qualify JIT target/flags and kernel support. Do not re-enable defaults through feature unification. [Original F14][original] |
| Native solver objects | The HiGHS model's public auto traits are `!Send`/`!Sync`; POUNCE integration also has thread-local ownership requirements | Send immutable problem data to a worker and construct native state there; do not assert `Send` to fit an async executor. [HiGHS API][highs-model], [POUNCE][pounce] |

This is not a requirement that every math library resolve to a single nalgebra/faer/dual-number
version. Shared concrete types must be compatible; private provider types may coexist behind
explicit value boundaries. The existing Arrow-family invariant remains a separate concern.
No compatibility of the combined target is claimed until a locked build and a meaningful
cross-boundary fixture succeed.

### 9.3 Review validation

This review changes documentation only. Validation on 2026-09-24 used a zero-failure baseline:

| Check / mode | Result | Limit |
|---|---|---|
| `just docs` — documentation build | 0 failures; 1 large-search-index warning | Book generation only; the warning remains open, and this is not a warning-free repository gate |
| `.venv/bin/typos` on this review and `docs/SUMMARY.md` — scoped spelling | 0 findings | Only those two documents |
| `git diff --check` for the summary and `git diff --no-index --check /dev/null` for the new review — whitespace | 0 whitespace diagnostics; the no-index command returns 1 for the new-file difference | Does not audit unrelated working-tree edits |
| Reference/structure script — this review | All 37 reference labels resolve; the relative target exists; sections 1–11 are present | Structural checks, not a proof of prose or library behavior |
| HTTP checks — 36 external references | 35 returned HTTP 200 after correcting two URLs; SCIP documentation was rate-limited (HTTP 429) | SCIP's official page was found through web search; its direct fetch remains unverified |

No solver or performance campaign was started, and no existing acceptance checkpoint was marked
complete. These documentation checks establish no runtime mathematics or simulator capability.

### 9.4 Source register

The links at the relevant claims are the evidence trail. Exact crate source links below denote
public APIs/manifests or source inspected for non-Symbolica libraries; Symbolica references are
public rustdoc. Context7 was a discovery aid, not authority for exact signatures. Upstream
SUNDIALS documentation establishes candidate capabilities; the actual native build remains a
qualification decision. These are research snapshots, not additional workspace pin declarations.

## 10. Authority changes, departures and unresolved decisions

### 10.1 Required authority changes

The user has authorized the direction; the remaining design decisions concern execution details.
No old principle is used here to require retaining bespoke mathematics or Pyomo. Apply the
repository's ADR/design amendment route when implementing these changes; this review does not
edit immutable authority in passing.

| Authority affected | Required go-forward change | Why / route |
|---|---|---|
| Blueprint D10 and the math-pass descriptions in §7/§14; earlier relational-execution decisions | Library-native arithmetic, in-memory preparation and problem-specific native solvers; relations at useful authored/result boundaries | Record the computational ownership change in the math ADR and design amendment |
| Ordered floating-point rewrite/hash policy, including ADR-0047 and the RCA wording identified by the original review | Accepted real-algebra/admissibility policy, numerical accuracy scope, typed-definition identity and explicit provider/target inputs | Superseding decision plus precise guard/smoothness contracts; no attempt to grandfather the old arithmetic engine |
| D9/kernel contracts | Versioned property packages, physical branch/reference meaning, derivative modes, fallibility and implicit-solve behavior | Amend the kernel/provider decision and generated capability contract |
| D11/native solving, §17 initialization and §18 solver classes | Native NLP plus library-owned root solving; class-specific profiles, status fidelity and original-model postsolve | Backend/initialization decision; library algorithm versus process strategy ownership stated explicitly |
| D12 and §21 Pyomo; GDP/MINLP/estimation references in §18–§19 | Remove production Pyomo authority and undocumented fallback promises; describe native routes and outstanding capability packages | Supersede affected decisions; update production dependency/import boundary and public claims |
| §13.6 dynamics and P11/P12 expectations | Explicit finite-index semantics and supported dynamic profile; distinguish time integration from dynamic-optimization transcription | Follow-up design amendment before dynamic capability is advertised |
| Crate and generated-family inventory | Delete replaced IR/evaluator/math-transport/Pyomo families and their callers/tests; regenerate outputs from surviving declarations | Crate/contract ADR where required; no hand editing generated trees |
| Plan 13 and related status/design documents | Separate completed reusable infrastructure from replaced math scope and still-open product acceptance; link the successor math plan when one is approved | Reconcile living plans during the pivot; do not mark W19/W20 or the native simulator complete merely because the target changed |

No legacy migration layer is proposed. Incompatible old compiled artifacts should be rejected
and rebuilt from supported authored definitions, with explicit version boundaries. Existing
authored inputs should be assessed for what the new compiler accepts; do not silently reinterpret
old serialized arithmetic under new semantics. Historical reviews/receipts remain identifiable
as historical evidence, not executable compatibility code.

### 10.2 Decisions to settle with bounded qualification

| Decision | Recommended starting position | Remaining evidence / owner and trigger |
|---|---|---|
| Root solver default | KINSOL for maturity and feature depth; `diffsol-nl` as the smaller pure-Rust challenger | Native-runtime owner qualifies the same difficult recycle/scaling cases before making one default; no parallel custom Newton implementation |
| Fallible property-to-Symbolica bridge | Prove one real multi-output provider, including required derivative order, before generalizing registrations | Math/provider owner settles callback error ownership and JIT/branch behavior in the first provider spike; if unsupported, use guarded library blocks or explicitly restrict that mode |
| Symbolic derivatives versus library AD | Symbolic local derivatives first, `Dualizer`/num-dual where their contracts reduce expansion or kernel work | Compiler owner compares bounded local workloads when derivative size or compilation cost becomes material; no new AD tape engine |
| Native presolve scope | Exact graph analysis first; enable numeric reductions only for qualified supported expressions/policies | Solver owner proves dependency completeness and original-space recovery before enabling each reduction |
| Dynamics profile | Diffsol for its supported native mass-matrix form; IDA when a concrete implicit-DAE requirement needs it | Dynamics owner supplies the first actual process model and consistency/event tests; do not implement both integrations speculatively |
| Oximo and export | Optional consumer-specific projection | Add only when a required export or transformation demonstrably removes work and passes conformance; no foundational export gate |
| Additional thermodynamics / discrete optimization | FeOS for compatible selected models; SCIP remains a candidate for later factorable MINLP | Process-model owner identifies missing physical/model-class requirements; choose/qualify providers from those requirements |

The meaningful departures are intentional: mathematical semantics are primarily owned by
libraries, and numerical reproducibility is tolerance-based under a declared policy. They reduce
bespoke decisions while retaining physical validity and interpretable results. They do not waive
domain checking, correct derivatives, failure attribution or truthful capability claims. There
is no proposed exception allowing a failed MUST-level contract to be called supported.

## 11. Decision and implementation changes

**Decision: Revise the detailed target; proceed with the library-owned hard pivot.** Preserve the
original proposal's strongest choices and extend library ownership further. Close the concrete
composition contracts before broad migration, then replace and delete by vertical slice. This
section gives dependency order for a subsequent implementation plan; it is not an implementation
performed by this review.

| Priority / dependency | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 — before broad implementation | Record the ownership/numerical/native-runtime decisions; correct the candidate feature/version assumptions; narrow unsupported claims | DM-02, DM-40, DM-43, DM-59 | Reviewed decisions plus locked minimal composition builds; §9.2 gaps settled for the first slice | Dependency/profile and capability checks |
| 2 — critical composition spikes | Guarded library evaluation; aliased sparse assembly; one fallible property provider with required derivatives; native callback status/scaling | DM-07, DM-22, DM-24, DM-30, DM-44 | Focused cases N01–N07, including the exact `a=b` Hessian oracle | Contract tests at the library boundaries |
| 3 — first coherent product slice | Typed indexed template → Symbolica → bindings → existing Ipopt C → original physical results; one supported property package | DM-17, DM-18, DM-23, DM-41, DM-46 | Indexed heater/flash with conversion, correct derivatives, public native solve and coherent publication | Source-to-solve fixture and physical oracles |
| 3 — paired with each replacement | Delete bespoke math IR/derivative/evaluator paths, relational math transport, Pyomo production integration and superseded callers/tests; regenerate surviving contracts | DM-02, DM-52, DM-56, DM-58 | Deletion inventory and clean generation; production dependency/import boundary without Pyomo | No fallback-route test or retained compatibility engine |
| 4 — after the shared numerical contract | Add the qualified library root solver and process initialization strategy; complete POUNCE under U4; qualify useful presolve/postsolve | DM-19, DM-22, DM-29, DM-30, DM-38 | Difficult recycle initialization and both NLP backends, with physical/status checks | Recovered trial, singularity, presolve and cancellation cases |
| 5 — concrete optimization consumers | Direct HiGHS/Clarabel routes; exact tear MILP when required; optional Oximo/export only for demonstrated consumers | DM-19, DM-41, DM-42, DM-43 | Class-specific positive/negative cases, compatible warm starts and meaningful status | Capability-to-fixture mapping, convexity/shape invalidation |
| 6 — process capability expansion | Qualified dynamics, estimation and later discrete/robust workflows, each with explicit functional scope | DM-04, DM-43, DM-48, DM-59 | Separate physical/model-class acceptance; no implication that a steady-state slice completes all simulator scope | Model-specific DAE/event/sensitivity/discrete regressions |
| 7 — qualification and documentation closure | Full representative process journeys, end-to-end cost and resource measurement, reconciliation of plans/designs/public claims | DM-39, DM-53, DM-54, DM-59, DM-60 | Evidence-labelled acceptance with exact commands, feature/build conditions, zero failure baseline and documented remaining scope | Maintained acceptance suite and discoverable current-state documentation |

The desired result is a small process-modeling and integration core around strong mathematical
libraries. Success is established when adding a physical model mainly adds equations, bindings,
property selection and physical tests, while differentiation, convergence, factorization and
integration continue to be supplied by libraries.

[original]: design_review_library-owned-math-pipeline_2026-09-23.md
[symbolica-eval]: https://docs.rs/symbolica/3.0.0/symbolica/evaluate/evaluator/struct.ExpressionEvaluator.html
[symbolica-dual]: https://docs.rs/symbolica/3.0.0/symbolica/evaluate/dual/struct.Dualizer.html
[symbolica-external]: https://docs.rs/symbolica/3.0.0/symbolica/evaluate/external/trait.ExternalFunction.html
[symbolica-registration]: https://docs.rs/symbolica/3.0.0/symbolica/atom/struct.EvaluationInfo.html
[symbolica-optimization]: https://docs.rs/symbolica/3.0.0/symbolica/evaluate/function_map/struct.OptimizationSettings.html
[faer-csc]: https://docs.rs/crate/faer/0.24.4/source/src/sparse/csc/mod.rs
[ipopt-api]: https://coin-or.github.io/Ipopt/INTERFACES.html
[ipopt-output]: https://coin-or.github.io/Ipopt/OUTPUT.html
[pounce]: https://docs.rs/crate/pounce-rs/0.12.0
[kinsol-math]: https://sundials.readthedocs.io/en/latest/kinsol/Mathematics_link.html
[kinsol-api]: https://sundials.readthedocs.io/en/latest/kinsol/Usage/index.html
[diffsol-nl]: https://docs.rs/crate/diffsol-nl/0.1.1/source/src/
[highs-model]: https://docs.rs/highs/2.4.0/highs/struct.Model.html
[highs]: https://github.com/ERGO-Code/HiGHS/blob/master/docs/src/index.md
[clarabel-cones]: https://clarabel.org/stable/api_cone_types/
[clarabel-update]: https://clarabel.org/stable/user_guide_data_updating/
[clarabel-manifest]: https://docs.rs/crate/clarabel/0.11.1/source/Cargo.toml
[feos]: https://docs.rs/feos/0.10.1/feos/
[feos-state]: https://docs.rs/feos-core/0.10.1/feos_core/struct.State.html
[feos-equilibrium]: https://docs.rs/feos-core/0.10.1/feos_core/struct.PhaseEquilibrium.html
[feos-manifest]: https://docs.rs/crate/feos-core/0.10.1/source/Cargo.toml
[num-dual]: https://docs.rs/crate/num-dual/0.15.0/source/src/lib.rs
[num-dual-implicit]: https://docs.rs/crate/num-dual/0.15.0/source/src/implicit.rs
[num-dual-explicit]: https://docs.rs/crate/num-dual/0.15.0/source/src/explicit.rs
[diffsol]: https://docs.rs/diffsol/0.16.2/diffsol/
[diffsol-manifest]: https://docs.rs/crate/diffsol/0.16.2/source/Cargo.toml
[diffsol-la]: https://docs.rs/crate/diffsol-la/0.1.1/source/Cargo.toml
[ida-math]: https://sundials.readthedocs.io/en/latest/ida/Mathematics_link.html
[sundials-build]: https://docs.rs/crate/sundials-sys/0.6.2/source/build.rs
[pounce-presolve]: https://docs.rs/crate/pounce-presolve/0.12.0/source/src/lib.rs
[pounce-expression]: https://docs.rs/crate/pounce-nlp/0.12.0/source/src/expression_provider.rs
[pounce-elimination]: https://docs.rs/crate/pounce-presolve/0.12.0/source/src/linear_eq_plan.rs
[petgraph-fas]: https://docs.rs/petgraph/0.8.3/petgraph/algo/feedback_arc_set/fn.greedy_feedback_arc_set.html
[igraph-fas]: https://docs.rs/crate/rust-igraph/0.7.0/source/src/algorithms/feedback_arc_set.rs
[oximo-solver]: https://docs.rs/oximo-solver/0.7.0/oximo_solver/solver/trait.Solver.html
[scip]: https://www.scipopt.org/doc/html/WHATPROBLEMS.php
