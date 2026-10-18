# Execution-speed target design for pse-arrow

**Baseline:** 5a009dccc7023e0142784892665c58bd20fbc305  
**Review date:** 2026-10-18  
**Scope:** End-to-end source preparation, mathematical execution, native solvers, studies/fitting, dynamics, observation and result publication.  
**Review type:** Design tier; target purpose.  
**Evidence level:** Inspected production source and interfaces, reconciled Plans42–45, and checked primary library documentation. No new build, numerical test or performance benchmark was executed.  
**Suggested repository location if adopted:** docs/design_review/reviews/design_review_execution-speed-target_2026-10-18.md

## Executive assessment

The strongest target is a simulator that prepares reusable mathematical structure once, computes only the numerical product actually consumed, and runs it inside retained private workers under one explicit resource budget. Original physical assessment remains the definition of a successful result.

The present architecture already supplies much of that foundation: typed source handoff, incremental preparation, immutable mathematical templates, private native state, sparse assembly, exact reuse conditions, selected derivative products, native solver composition, bounded study concurrency and durable result contracts. The remaining opportunities are concentrated where a narrow request expands into broader work, or a retained result still incurs repeated preparation and data movement.

The principal recommendations, in order of implementation emphasis, are:

1. Carry selected derivative coordinates, directions and output cotangents through implicit-provider boundaries. Preserve the full physical model and sparse state factorization while avoiding unnecessary complete parameter jets.
2. Make large sparse first-derivative execution compact before final CSC assembly, and investigate Hessian actions whose preparation does not enumerate the complete supported Hessian.
3. Establish a credible large-system linear algebra tier: reusable preconditioner workspaces, stronger problem-appropriate preconditioning, and an optional inner-parallel root route.
4. Replace initialization wave barriers and ephemeral block workers with a prepared dependency graph and completion-driven retained execution lanes.
5. Bind immutable provider descriptors once and use reusable result storage; remove proven warm-path traversal, gathering and copying.
6. Complete the scalable response/sensitivity path: retained response symbolics, explicitly named sparse regularity products, direct dynamic actions and memory-aware adjoint replay.
7. Pipeline selected result encoding/publication within the existing bounded ownership and acknowledgement contracts.

This is a target architecture and source-backed prioritization. It does not establish that any proposed method will outperform every alternative, or quantify a speedup. The numerical route must be selected against complete time to an equivalently qualified solution, and the parallel route against a fixed total resource envelope.

## 1. What the recent plans have already accomplished

The review uses the implemented state at the specified commit, including Plan45's reopened follow-up dispositions. Earlier limitations in the background plans are not automatically current defects. [P42] [P43] [P44] [P45]

| Area | Foundation to preserve | Remaining target |
|---|---|---|
| Source and preparation | Typed direct handoff, shared local source indexes, completion-owned preparation, exact historical reuse and consumed physical premises on eligible paths | Avoid processing every unchanged declaration during large incremental imports where the source contract permits changed-object admission |
| Mathematical execution | Shared immutable templates, private worker scratch, active local axes, selected products, same-point reuse and canonical sparse refill maps | Keep derivative demand compact across all providers and intermediate representations |
| Sparse native execution | Qualified fixed-pattern KLU symbolic retention is implemented | Improve preconditioning and remaining response-factor lifetimes |
| Implicit providers | Numerical reuse observes effectful anchors/restrictions | Avoid complete parameter jets for narrow derivative consumers and avoid owned projection copies |
| Dynamics and fitting | Selected sensitivity axes, eligible combined forward quadrature, corrected IDAS quadrature history, output sampling independent of integration-step selection | Direct Diffsol actions, stronger DAE preconditioners, long-horizon checkpoint efficiency |
| Workflow execution | Bounded completion-driven study execution and explicit continuation ownership | Remove barriers within block initialization; address skew in fitting experiments |
| Results and observation | Selected/full read contracts, grouped publication, explicit backpressure and terminal ownership | Reduce synchronous per-sample construction and parallelize sufficiently large independent reads |

Plan45f records one 80-state, five-time-point execution with total wall time 19.275325 seconds, source-to-analysis time 9.602189 seconds, preparation time 4.017421 seconds, and inclusive simulation/publication time 1.104101 seconds. These scopes overlap and are not an exclusive timing decomposition. The run is useful evidence that interactive latency includes substantial work outside native integration; it is not a repeatable speedup estimate or a universal bottleneck ranking. [P45F]

The exact-counter growth fixture also stopped and joined an independent periodic diagnostics writer after observing a dropped update. Its exact counters do not establish ordinary telemetry overhead or universal zero-loss behavior. The next qualification should distinguish observer-disabled, ordinary and detailed collection modes. [P45F]

## 2. Target architecture and responsibility boundaries

The mathematical layer should continue to own mathematical meaning, support, derivative demand, assembly and reconstruction. Native libraries should continue to own nonlinear iteration, time stepping, sparse factorization and established preconditioning algorithms. The runtime should own readiness, resource admission, cancellation and publication. Arrow and database representations belong at source/result boundaries; the current mathematical layer already excludes Arrow, DataFusion and Delta dependencies. [A1] [A2] [A3]

The target has five consequential lifetimes:

| Lifetime | Retained content | Invalidated by |
|---|---|---|
| Model structure | Canonical equations, conservative support, matching/BTF, execution dependencies, reconstruction, selected product descriptions | Consumed structural or semantic changes |
| Prepared numerical layout | Compiled expressions, sparse scatter maps, immutable provider-demand descriptors, compatible symbolic factor plans | Layout, support, ordering, realization or capability changes |
| Attempt/worker | Bound providers, evaluator scratch, native sessions, result buffers, block workspaces, admitted capacity | Attempt ownership/reset, incompatible profile or explicit lifecycle end |
| Current numerical point | Residuals, derivative values, numerical factors, completed assembled products | Exact point, fixed-input revision, weights/direction or validity-premise changes |
| Result publication | Selected rows, bounded batches, exact event ordinals and durable identity | Completion of publication or explicit cancellation/recovery contract |

A prepared provider handle is a local checked binding, not a new global registry. A retained block worker is an existing MathService-owned lane, not another scheduler platform. A retained symbolic factor is an immutable structure product, not permission to reuse point-dependent factors. These distinctions keep each optimization attached to an existing owner.

For one simulation, the intended execution is: admit only changed source meaning; obtain a compatible prepared layout; bind private workers; execute the ready mathematical work and native iterations; independently assess the original model; publish demanded outputs through bounded stages. Iteration callbacks should usually refill numerical values in prepared layouts. They should not rediscover immutable identities, construct set-based demand checks, rewalk already initialized stages or allocate temporary result ownership merely to copy it away.

### Workload objectives

Three separate objectives determine the winning realization:

- **Interactive latency:** cold import, unchanged replay, same-structure parameter edit, consumed physical edit and structural edit to a usable qualified result.
- **Large-case latency:** one substantial coupled root, stiff dynamic simulation or optimization to original-model acceptance, including preparation and response work requested by the user.
- **Fixed-resource throughput:** studies and fitting across many cases/experiments, including tails and failures, under the same cores, memory and output demand.

No universal solver, derivative mode or thread count optimizes all three.

## 3. Numerical design: reduce necessary work before tuning its execution

### F01 — Carry derivative demand through implicit providers

**Evidence: source-established effort amplification. Priority: highest for wide implicit providers.**

ResidualProblem::derivatives_at_verified_point factors the local state Jacobian sparsely, then constructs derivatives across all provider input coordinates. First order solves an n-by-p right-hand-side block; second order constructs all p(p+1)/2 coordinate pairs and eventually expands state Hessians over n-by-p-by-p entries. This remains broader than a consumer that requests a few parameter columns, one direction or one weighted output. Existing sparse factorization and batched right-hand-side solves are strengths. [S01]

For local equations r(z,p)=0, implicit differentiation gives

\[
r_z z_v=-r_p v.
\]

One requested parameter direction v can therefore use one combined right-hand side and one state solve. For a scalar consumed output q(z,p),

\[
r_z^\mathsf{T}\lambda=q_z,\qquad
\frac{dq}{dp}=q_p-r_p^\mathsf{T}\lambda.
\]

This uses a transposed state solve and contracted derivatives. It avoids constructing every state-by-parameter sensitivity merely to contract it later. Hessian actions should likewise differentiate the consumed contraction where supported.

**Correction.** Extend the existing provider-demand contract to express selected coordinate columns, JVP, VJP and weighted second-order actions. Keep all physical input values and all local states needed for a valid solve. Reuse the state factorization for batched demanded right-hand sides. Retain the present complete-jet route when a consumer actually needs the complete jet.

Sparse state equations do not guarantee sparse implicit response: local elimination can make z_p dense. The optimization is demand reduction, not an assumption that all response matrices remain sparse. Output subsets alone also do not always reduce the required state solve.

**Verification.** Vary total input count independently from consumed directions/output cotangents. Count RHS columns, derivative entries, provider evaluations, storage and complete solve time. Include full demand, singular neighborhoods, branch changes, failed trials, effectful anchors/restrictions and order upgrades. A narrower request must not silently weaken neighborhood or original-space derivative checks.

### F02 — Keep sparse first derivatives compact before assembly

**Evidence: source-established representation cost. Priority: high for large sparse local bodies.**

Worker::evaluate_seeded uses a main frame sized by body slots times jet width, and returns first derivatives over outputs times requested coordinates, including structural zeros. CaseWorker::jacobian later extracts selected terms into the canonical sparse matrix. Compilation already narrows individual arithmetic regions to active axes; the remaining issue is the whole-worker/result representation. [S02] [S03]

**Correction.** Let support-indexed local products feed precomputed canonical scatter maps directly. Where library capabilities fit, compare compressed directional seeds/coloring or another library-owned derivative realization. Retain dense jets for small or genuinely dense blocks. Preserve conservative branch support and structural zero semantics.

Authoring should not require splitting a sparse model into thousands of tiny bodies solely to escape dense intermediate storage; that can trade memory cost for occurrence and dispatch overhead.

**Verification.** Compare a large flat sparse body with an equivalent many-instance body. Vary support density, coordinate count and output count. Measure live frame entries, returned entries, scalar operations, gathering/scattering and complete Jacobian callback time.

### F03 — Build Hessian actions from contractions when that reduces preparation

**Evidence: source-established cold construction pattern. Priority: high for dense curvature with narrow consumption.**

PreparedSupport::compile_direct_product builds a union of second-order support pairs and differentiates the weighted expression twice for those pairs before constructing the HVP expressions. The published product is action-shaped, but its symbolic preparation can still track the supported Hessian. [S04]

**Correction.** Compare a library-owned evaluation of the directional derivative of the weighted gradient with the current pairwise construction. An equivalent mathematical target is the gradient of the directional derivative of the weighted scalar function. Share the expression DAG and respect provider/domain/branch boundaries.

Symbolica's public automatic-differentiation and evaluator APIs support directional algebra and vectorized realizations. They do not establish that a dedicated reverse-mode HVP API is already a drop-in replacement for this integration. The exact pinned release and demanded orders need verification. [U1] [U2]

A sparse pairwise construction can be preferable for a small supported Hessian. Ipopt's exact-Hessian interface consumes sparse Hessian entries; an HVP alone does not satisfy it. Its limited-memory mode is a separate algorithm with different iteration/storage tradeoffs. [U3]

**Verification.** Compare preparation plus the expected repeated evaluations on dense reductions, sparse chains, shared subexpressions and provider-heavy programs. Count generated operations and retained code/scratch as well as time.

### F04 — Treat formulation and initialization as numerical choices

**Evidence: existing mechanisms plus a proposed extension. Priority: high on connection-heavy and difficult-start workloads.**

The code already has incidence matching, structural decomposition, execution-dependency merging and authored regular implicit reduced suppliers. The root runner checks structure, normalizes, executes and recovers original coordinates; it does not apply the NLP presolve pipeline. The NLP pipeline withholds certain bound-tightening passes because original multiplier recovery is unavailable. [S05] [S33] [S06] [S07] [S08]

**Correction.** Keep one transformation/reconstruction authority, but qualify transformations against the operation's actual obligations. Root finding or initialization may admit suitable alias/affine elimination and bound contraction when complete primal reconstruction is established, without demanding optimizer multiplier recovery that its consumer does not use. Optimization retains original multiplier/KKT obligations.

Compare simultaneous solution, causal/BTF execution, coalesced tiny blocks and regular implicit suppliers as alternative realizations. Use accepted warm states, continuation and tangent predictions only under declared start and branch validity. A scalar structural block alone does not establish a continuous bracket or a unique physical root.

Smaller formulations can increase fill, worsen conditioning or multiply nested property solves. Condensation is beneficial only when the reduced solve plus reconstruction and derivative work wins.

**Verification.** Use connection-heavy flowsheets, large coupled recycle components, many tiny blocks, difficult starts and branch-sensitive suppliers. Compare original outcomes and complete property/factorization work. Preserve all original bounds, balances and reconstruction failures.

### F05 — Establish a scalable preconditioning and linear-solver tier

**Evidence: source-established capability limits; replacement performance remains proposed. Priority: highest for large coupled systems.**

The KINSOL block preconditioner is BTF diagonal-block Jacobi. Off-block lower coupling remains in the true JVP. Each setup creates fresh FERAL Markowitz factors; each application allocates a global result vector and per-block RHS vectors. Its conservative admission bound includes 2048n² + 256n³ + 4096n bytes plus object overhead for a block of size n. At n=1,000, the cubic term alone is 256 GB in decimal units. This is an admission allowance derived from the pinned implementation's possible retained history, not a measured actual allocation. [S09]

**Correction, immediate.** Reuse preconditioner application scratch and immutable block/scatter maps under the existing session owner. Separate symbolic structure from numerical factors where the library contract supports it. Preserve complete true-operator coupling and failure-safe publication.

**Correction, scale-enabling.** Qualify an established library implementation of stronger block-triangular, ILU, overlapping or Schur preconditioning for the relevant matrix families. A large strongly connected component does not by itself provide a useful internal split. Physical roles and structural couplings must justify the split.

For

\[
J=\begin{bmatrix}A&B\\C&D\end{bmatrix},
\qquad S=D-CA^{-1}B,
\]

apply the inverse action through solves; do not form an explicit inverse. Local thermodynamic states, network variables and differential/algebraic roles can suggest useful partitions when the actual equations support them. PETSc PCFIELDSPLIT provides additive, multiplicative and Schur composition, including implicit Schur actions. This is a candidate library capability, not an already integrated parallel route. [U4]

| Matrix/workload character | Target candidate | Principal tradeoff |
|---|---|---|
| Small dense local system | Dense direct solve | Low overhead can dominate asymptotic advantages |
| Sparse process network with manageable fill | KLU with qualified symbolic retention | Excellent reuse opportunity; serial factorization limits one-case scaling |
| Large system with costly sparse direct factors | Threaded unsymmetric direct route | Factor latency versus memory and reduced case concurrency |
| Large strongly coupled system | Exact JVP with strong admitted preconditioning | Setup/application work and convergence are inseparable |
| Many small independent cases | Serial or modestly threaded inner solves, broad outer concurrency | Maximize total completed work within the same core budget |

Current KINSOL execution requires one thread. Current PETSc execution has a serial process owner. Existing threaded SPRAL/Pardiso options for Ipopt KKT systems do not provide a threaded unsymmetric root binding. SUNDIALS 7.1.1 documents sparse direct interfaces including SuperLU_MT and SuperLU_Dist, offering narrow library-adapter options if the workload justifies them. [S10B] [S11] [U5]

KINSOL already exposes native line search, modified/inexact Newton setup controls, Eisenstat–Walker forcing and Anderson settings. The target is to calibrate and compose these existing capabilities, not add another Newton implementation. FGMRES is a fitting option when the applied preconditioner changes, but it cannot compensate for an ineffective preconditioner. [S10] [U5]

**Verification.** Use weak/strong coupling, dominant large components, nonnormal matrices and representative process sparsity. Measure setup, applications, numerical factors, backsolves, Krylov work, nonlinear recovery, peak memory and complete qualified time. An improved iteration count can lose if each iteration costs more.

### F06 — Make dynamic actions and DAE preconditioning match their consumers

**Evidence: source-established route distinction. Priority: high for large stiff dynamics.**

Diffsol Operator::product and transpose_product obtain assembled sparse partials and multiply them. Some sparse scatter paths also repeat stable entry lookups. IDAS jtimes already consumes a direct RHS action, while its exposed Krylov preconditioner choices are currently None/Jacobi and reject BlockFactor. These routes should not be described as having identical gaps. [S12] [S13]

**Correction.** Route JVP/VJP demand to prepared action programs when that avoids building a matrix. Retain CSC multiplication when a complete current Jacobian is already required or cheaper. Prepare stable offset maps for sparse scattering.

For an index-1 DAE, the relevant linear operator includes the current integration shift and mass structure, such as F_y + c_j F_ydot or the equivalent c_j M − J under the residual convention. Preconditioning must treat the algebraic block and current step coefficients. Reusing a root preconditioner without those distinctions can regress integration.

Keep selected forward/adjoint axes and functionals. For few consumed parameter axes, forward sensitivities are often attractive; for few scalar objectives over many parameters, adjoints become attractive. Compare the whole route, including replay and derivative preparation.

The current adjoint checkpoint policy uses fixed spacing and refuses when its admitted maximum is exceeded. A memory/recomputation schedule or qualified library checkpoint policy is a long-horizon target. Checkpoint identity must include events, schedules and replay state; changed parameters do not justify carrying old multistep history into a fresh solve. IDAS has a native adjoint/checkpoint lifecycle that should guide any adapter extension. [S14] [U6]

**Verification.** Use stiff reactions, algebraic pressure/phase constraints, transport discretizations and long-horizon fitting. Count rejected steps, consistent initialization, derivative calls, factorization, replay and live checkpoint memory. Preserve the supported fixed diag(I,0), ODE/index-1 scope. General implicit or higher-index dynamics would be a separate capability change.

### F07 — Complete scalable response products without changing rank meaning

**Evidence: source-established repeated work and scaling boundary. Priority: high where large public responses are required.**

The sparse square-response factor constructor performs symbolic LU analysis for each new response; its action groups allocate fresh scratch and RHS/solution/residual matrices. Separately, the public spectrum product materializes an n-by-n scaled dense matrix for SVD under explicit memory admission. This is requested response work, not an unconditional cost of every root solve. [S15] [S16]

**Correction.** Retain exact compatible symbolic response plans and attempt-owned action scratch; keep fresh point-specific numerical factors. Preserve batching of RHS columns.

Offer a separately named sparse regularity/rank assessment when that satisfies a consumer's scientific claim. SuiteSparseQR is an established candidate, and its source is present in the vendored SuiteSparse tree; a production binding and its semantics still need qualification. QR rank at a pivot tolerance, SVD rank at a relative singular-value cutoff and an iterative singular-value estimate are distinct products. A successful LU or low backward error is not a full numerical-rank certificate. [S17]

Retain bounded dense SVD for consumers requiring its current meaning. Keep useful sparse prediction/actions independently available where their own validity is established.

**Verification.** Include scaled near-dependencies, exact rank loss, nonnormal systems, changed support/ordering and singular-to-regular transitions. Record the method, scaling, cutoff, memory and uncertainty with the result.

## 4. Callback and code-efficiency target

The following findings concern work visible in source. Their runtime importance depends on whether actual cases are arithmetic-, factorization- or host-overhead-dominated.

### F08 — Bind providers once and fill reusable storage

Provider execution currently rehashes immutable specifications, looks them up in a BTreeMap, compares bound specifications and repeats request validation. ProviderRequest::validate and ProviderValues::validate each build a set to check output uniqueness. The provider interface returns owned value/Jacobian/Hessian vectors; evaluation scatters their contents into the worker frame and drops them. Nested::project creates fresh vectors even for a successful numerical cache projection. [S01] [S02] [S18]

**Correction.** Bind the actual fresh provider instance to a prepared slot and checked demand descriptor once. Precompute immutable identity and exact result extents. Fill caller-owned admitted buffers or expose a safe internal borrowed result view. Retain a checked public boundary for arbitrary callers.

Per-trial finiteness, input envelopes, neighborhood, cancellation, deadlines, effectful anchors/restrictions and failure behavior remain current checks. Equal numerical inputs do not establish provider purity or permit reordering. A provider registration/capability change invalidates the checked binding.

**Acceptance.** On ordinary warm calls, stable descriptor hashing, duplicate ordinal-set allocation and projection-buffer allocation disappear. Failures after partial fill never expose stale or partially valid outputs.

### F09 — Reuse the completed callback product and avoid repeated warm preparation

Worker::prepare_order traverses instantiate_stages even after the order already exists; that traversal includes nested scopes and both branch arms. Most warm visits only discover an already initialized worker or an interpreted no-op. Track successful instantiated state and invalidate it on reset. [S02]

CaseWorker local result reuse still gathers/converts/bits-compares occurrence inputs, and constraints/gradient/Jacobian paths refill assembled products on repeat requests. A point revision can qualify reuse of the completed assembled product, including exact free-point bits, fixed-input revision, derivative order, weights and direction. Native “new point” hints are usable only where the actual native interface guarantees their meaning. Ipopt also performs its own caching, so real callback traces must establish the remaining duplication. [S03] [U7]

Sparse derivatives also cross multiple complete buffers between assembly, oracle and shielded callback publication. Choose one failure-safe completed staging owner, then publish once. Preserve original accumulation order and failure atomicity. [S19] [S19B]

**Acceptance.** Measure stage visits, gathered scalars, refilled entries, copied bytes and actual callback time. Include order upgrades, reset/retry, changed fixed values at identical free x, changed weights, late failures and independent original assessment. Retaining a completed native callback result never licenses skipping the final original-model assessment.

### F10 — Reduce frame traffic with compiler-owned write and liveness facts

The evaluator poisons the current-order frame, gathers region arguments, zeroes/scatters output jets and copies requested results out. Those operations include meaningful unassigned/nonfinite protection. [S02]

Use prepared write sets and lifetime facts to initialize only necessary components, reuse dead slots and move contiguous ranges where safe. Let established library evaluators consume direct views when their ownership contracts support it. Do not remove checks merely because successful cases overwrite the buffers.

Warm preparation lookups can also allocate owned scalar keys before checking a cache and repeat map lookups. Borrowed keys or exact prepared scalar handles can remove such work without a second cache authority. [S20]

For many pure independent instances of one template, investigate structure-of-arrays batches using library SIMD evaluation. Provider effects, branch divergence, source failure order and cancellation remain explicit boundaries. This is a targeted realization experiment, not an argument for a new automatic-differentiation engine.

## 5. Parallel execution and orchestration

### F11 — Use completion-driven retained block workers

MathService::prepare_blocks and blocks_step repeatedly traverse readiness waves. ready_wave reconstructs producer information and scans boundaries; execution clones the complete scalar Values map for each block. Parallel waves spawn scoped operating-system threads whose Retained state is fresh and cleared before joining. Every child in the wave completes before successors are released. [S21] [S22]

**Target.** Prepare successor lists, predecessor counters and compact block input bindings once. Dispatch eligible work to existing bounded MathService lanes, keeping compatible private evaluator/native allocations associated with a lane or block. Release a successor when its actual prerequisites have committed.

This changes critical-path behavior when a short block unlocks a chain while an unrelated sibling runs much longer. Coalescing tiny blocks may be helpful when dispatch and boundary work exceed numerical work. Use the compiled execution dependency graph, including provider/control dependencies, rather than deriving a scheduler from a visible flowsheet stream graph.

Scheduling, start policy and numerical reuse remain separate. Neutral allocations and immutable preparation can survive; solved roots, stale numerical factors and acceleration history cannot cross into another solve without the existing scientific start/reuse authority. No incomplete stage becomes an accepted initialization because a sibling later fails.

The current code divides a total iteration allowance across a wave and subtracts actual work. A completion scheduler must define an equally bounded aggregate policy and deterministic visible outcome/error selection. Giving every new task the full original allowance would change behavior and resource semantics.

**Acceptance.** A skewed independent-block fixture demonstrates that an eligible successor starts without waiting for an unrelated long block. Count readiness scans, full-map copies, threads and worker/session construction. Verify dependencies, cancellation/drain, aggregate iteration budgets, failure ordering and memory retention.

### Preserve study concurrency; improve fitting balance

Study execution already uses a completion-driven in-flight collection with bounded preparation and execution. Continuation chains have explicit ownership. That scheduler is a foundation, and independent-study execution should not be redesigned as if it were serial. [S23]

Fitting experiments are assigned to retained lanes by static round-robin stride. A few expensive experiments can leave other lanes idle. Compare cost/affinity-aware assignment or dynamic dispatch of unstarted independent experiments, while retaining each experiment's private state and bounded storage. Avoid replicating every experiment across every worker merely to obtain movable work. [S24] [S25]

Native process constraints also matter. PETSc currently has a serial process gate; other adapters have their own qualified lifecycle constraints. Time spent waiting on a library gate should be distinguished from CPU computation. Isolated native processes are a conditional option only when gate serialization dominates enough work to justify IPC, startup and additional residency.

### One total parallelism budget

The target is approximately bounded by admitted outer concurrency times admitted inner solver/evaluator teams, with memory and any process gates considered as well. Separate phases can reuse capacity; independently maximizing every nested pool causes oversubscription.

Choose outer parallelism first for many small cases; qualify inner factorization or Krylov parallelism for large individual cases. Preserve deterministic visible ordering and reductions where promised. A thread pool may retain allocation/workspace capacity, but retained capacity is still owned and charged.

## 6. Streaming, persistence and interactive preparation

### F12 — Pipeline sample handling within explicit output semantics

Live simulation currently constructs rows, a collection and checked Arrow representation on the native sample path. A full bounded channel causes retry/wait behavior. The supervisor awaits subscriber delivery and publication before acknowledging an event and progressing through that path. This makes encoding and downstream readiness relevant to observed simulation latency. [S26] [S27]

**Target.** Transfer compact selected numerical samples into admitted buffers; encode bounded microbatches on a dedicated existing publication stage. Flush at declared event/schedule/attempt boundaries and enforce a maximum latency and memory allowance. Preserve exact acknowledgement ordinals and terminal ownership.

The result append queue already groups files and metadata publication. Extending upstream batching should compose with that mechanism. It should not create a second unrelated publication queue or redefine “durable” to mean “enqueued.” [S28]

An unread subscriber currently has defined backpressure semantics. Introducing lossy monitoring, detachment or acknowledgement before durability would be an explicit product contract change. Internal batching can preserve existing meaning, but its effect on visibility latency still belongs in the comparison.

**Acceptance.** Compare sparse and dense selected output at unchanged integrator tolerances/steps. Record encoding time, queue occupancy, wait, batches, bytes and ACK latency. Preserve no sample loss, duplicate/reordered events, premature terminal success or cancellation leaks.

Large analytical reads have another conditional opportunity: the snapshot provider exposes one streaming partition and serialized decode access. Independent file decoding could help sufficiently large result scans. Retain full integrity checks and truthful filter/projection pushdown; ordinary small simulation latency is not evidence that parallel reads are needed. [S29]

### Source edits and cache capacity

Typed source handoff is already improved, yet canonical::persist still processes the candidate population before filtering unchanged objects, including generated declaration admission and canonical serialization. On very large repeated source edits, the authoritative source owner could accept typed changed-object sets and reuse exact per-object canonical bytes for unchanged objects. A complete import remains a complete operation. Identity, deletions, references, retry semantics and trust-boundary admission remain exact. [S30]

Plan45 also documents an optional retention boundary: a complete set of protection/release grants for the growth fixture exceeded the available optional cache budget. Such a retained product may legitimately be declined. Granular reuse or a different payload layout is worth considering if measurements show repeated work, but undercharging retained ownership or blindly enlarging the default cache is not a performance design. [P45F]

## 7. Library strategy and deployment

The main proposals fit the current library composition. Add only the missing native capabilities that a representative workload actually requires.

| Capability | Existing foundation | Proposed integration/qualification |
|---|---|---|
| Residual and derivative evaluation | Symbolica 3.0.1; existing prepared products | Directional/contracted construction, compact sparse products and eligible batches |
| Local/sparse linear algebra | faer 0.24.4 and existing factor owners | More exact symbolic retention, caller/attempt-owned scratch |
| Nonlinear roots | KINSOL; PETSc; native globalization and Krylov | Stronger preconditioners; optional large-case threaded root adapter |
| NLP | Ipopt/POUNCE and original KKT assessment | Compare current exact Hessian versus native limited-memory where appropriate; preserve consumer-specific derivatives |
| Dynamics | Vendored Diffsol 0.16.2; IDAS via SUNDIALS 7.1.1 | Direct action adapters, DAE-aware preconditioning and qualified checkpoint policies |
| Sparse rank/regularity | Existing bounded dense SVD; vendored SPQR source | Explicit separate sparse product and production binding if required |
| LP/QP/conic/MIP | HiGHS, Clarabel, SCIP and existing routing | Retain problem-class dispatch; avoid converting every solve to generic NLP |

The SUNDIALS sys-crate version is not the numerical C-library version. PETSc build metadata pins 3.24.0 while current upstream documentation inspected here describes 3.26.0. Current Diffsol documentation can describe 0.17.1 while this repository vendors 0.16.2. Upstream capability establishes a candidate; the pinned ABI, callback, cancellation and memory behavior must be verified before adoption. [S31] [S32] [U4] [U5]

The present release profile uses optimization level 2, ThinLTO and 16 codegen units; the dist profile already uses FatLTO and one codegen unit. Compare representative PGO and existing profile settings only after identifying the actual CPU-dominant kernels. Cargo explicitly recommends measurement because optimization level 3 can lose to level 2. [S31] [U8] [U9]

Plan45 investigated retained Symbolica sub-evaluators and alternate JIT construction. Required second-order realization and lifecycle/CPU-contract issues kept those alternatives from being adopted. Reopen that investigation when the exact library or deployment premise changes. “Enable JIT” is not a sufficient new target. [P45B]

SIMD across pure homogeneous instances has a clearer near-term fit than indiscriminate offloading. A GPU target would require enough device-resident numerical work or homogeneous batch work to amortize transfers and launches. It is conditional on workload and deployment, not an automatic consequence of using a faster accelerator. Any optional ISA, arithmetic or precision profile must state its numerical and portability contract.

## 8. Implementation sequence and acceptance

Prioritize demonstrated work amplification and scale limits. Keep numerical replacement choices provisional until the comparison establishes a winner.

| Stage | Deliverable | Completion criteria |
|---|---|---|
| A: remove direct amplification | Selected implicit derivative contract; reusable provider/preconditioner storage; skip warm stage-instantiation traversal | Work scales with demanded directions/columns where mathematically possible; stable warm metadata work and avoidable allocations disappear |
| B: preserve sparsity and readiness | Compact local derivative layout; contracted HVP experiment; prepared completion-driven block execution | Sparse cases avoid dense intermediate growth; HVP choice wins complete-use comparisons; eligible successors proceed without unrelated wave barriers |
| C: support large coupled cases | Strong root/DAE preconditioner candidates; optional inner-parallel root route; retained response factor plans and explicitly named sparse regularity tier | Large representative cases complete within admitted memory with equivalent original acceptance; one-case latency and fixed-core throughput are both reported |
| D: optimize long trajectories and data paths | Direct dynamic actions, checkpoint policy, fitting balance, upstream sample batching, large-edit/read improvements where justified | Full-trajectory and workflow comparisons show improvements; replay/output/ACK semantics and capacity remain intact |
| E: choose deployment realizations | Family profiles, optional SIMD/PGO/JIT/native-team choices with exact build identities | Recommended settings have repeatable workload-specific evidence, ordinary defaults remain supported, regressions and limits are documented |

Stages can overlap when changes are independent. Provider-demand design should precede separate buffer APIs so the buffer extent is driven by the actual product. Compact derivative layouts should share existing sparse maps. Block scheduling and retained ownership should be designed together so a queue change does not preserve repeated worker construction under a new name.

The response rank tier belongs in Stage C wherever large public responses or fitting qualification are required. It is not mere diagnostic polish: the current dense product has an honest scale limit.

### Representative comparison families

| Scenario | Variation that settles the design | Primary evidence |
|---|---|---|
| S01: repeated model edits | Cold, unchanged, changed consumed/unconsumed values, structural changes | End-to-end latency, source/admission/preparation work, exact reuse |
| S02: sparse local mathematics | Flat versus many-instance; width, density, output count | Frame/product entries, arithmetic work, assembly/copy time |
| S03: implicit properties | Wide input space; few columns/directions/cotangents; full demand | RHS count, factor reuse, jets, provider calls and validity |
| S04: coupled roots | Weak versus strong coupling, large component, fill, difficult starts | Factors, preconditioner setup/applications, Krylov/nonlinear work, acceptance |
| S05: skewed independent work | Long sibling plus short dependent chain; uneven experiments | Critical path, idle capacity, worker construction and bounded ownership |
| S06: stiff dynamics | Differential/algebraic coupling, step rejection, events and schedules | Total steps, factors, direct actions, initialization and original accuracy |
| S07: long-horizon sensitivities | Parameters versus functionals; memory pressure | Forward/adjoint/replay work, checkpoints, derivative validity |
| S08: dense output/analysis | Output density and file population with identical science | Encoding, queues, durable ACK latency, decode throughput and integrity |
| S09: library/realization change | Existing versus candidate backend/profile under exact identity | Complete qualified time, memory, failures, reproducibility and capability refusal |

Add realistic cases from the team's actual model population. These families discriminate mechanisms; they are not a new benchmark-platform requirement.

## 9. Measurement that can choose between competing designs

Use the existing diagnostics and performance infrastructure. The measurement unit is an end-to-end operation that produces the same requested scientific result.

Report three timing views separately: user-visible critical path; nonoverlapping stage durations where the scopes genuinely permit them; and nested numerical/callback work. Inclusive source, prepare, native and publication spans must not be summed into a false total. Record queue and process-gate waits separately from compute.

The smallest useful explanatory counters are:

- Requested and completed residual/Jacobian/JVP/VJP/HVP evaluations; actual coordinate directions, sparse/dense entries and implicit RHS columns.
- Provider calls and repeated inner solves, including cache hits, misses and state/effect barriers.
- Symbolic analyses, numerical factors, refactors, backsolves, preconditioner setups/applications and Krylov work.
- Nonlinear iterations, line-search/domain retries, rejected integration steps, restarts and adjoint replay.
- Worker/thread/session construction, readiness scans, gathered/refilled/copied entries and allocated bytes where relevant.
- Admitted/live/retained memory, capacity waits, result batches/bytes, queue occupancy and acknowledgement latency.

Collect enough repetitions to distinguish the proposed benefit from ordinary variance; the required count depends on the noise and effect size. Use cold/warm distinctions and fixed builds/hardware/profiles/output demand. Include failure and refusal populations, not just successful short runs. Separate ordinary observer cost from detailed collection.

Compare original physical residuals, balances, bounds, derivative validity and requested optimization KKT or dynamic error criteria. Keep work tolerance, physical acceptance, integration error and sensitivity/KKT tolerances distinct. The ordinary engineering relative target does not imply that all numerical controls should share the same scalar value. A speed gain from weakening the requested result is a different product decision.

Useful acceptance statements name eliminated work: zero additional symbolic analysis for an exactly compatible retained layout; no immutable descriptor-set construction in a warm provider call; directional implicit work proportional to demanded directions rather than all parameter pairs; no readiness wait on unrelated siblings. Wall time then establishes whether removing that work matters for the intended workload.

## 10. Architectural judgment and proposed rule impacts

The review applies the repository's Core 3.4 design principles, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 profile and pse-arrow binding. The verdict is scoped to the inspected execution responsibilities and declared high-performance target. It is not a complete scientific certification of the repository. [R1] [R2] [R3]

| Foundation | Scoped verdict | Reason |
|---|---|---|
| AP-01: separate concerns | Satisfied in inspected baseline | Mathematical, native, workflow and result owners are explicit; proposals preserve them |
| AP-02: explicit contracts | Unresolved for proposed extensions | Selected provider actions, new sparse regularity and scheduling semantics need precise capability contracts |
| AP-03: composition | Satisfied in inspected baseline | Established native libraries own numerical algorithms; narrow adapters are preferred |
| AP-04: authoritative domain model | Satisfied in inspected baseline | Physical meaning, reconstruction and original assessment remain authoritative |
| AP-05: constraints as design inputs | Satisfied in inspected admission paths | Memory, derivative validity and supported problem classes have explicit refusal; improved execution must preserve them |
| AP-06: local reasoning | Satisfied with bounded changes required | Immutable binding, attempt state and current-point state can evolve at existing owners; effects must remain visible |
| AP-07: execution fit | Violated for the specified large/narrow-demand target scenarios | Full implicit parameter expansion, dense local first products, large-block admission growth and wave/worker repetition materially oppose that target |

**Behavioral/semantic adequacy:** existing checks and ownership are supported by source inspection; runtime qualification of proposed replacements is unresolved. G1–G6 and Process Simulator physical/well-posedness/numerical gates are preservation requirements, not newly certified passes. G7 truthful claims is satisfied by separating inspected and proposed behavior and withholding speedup claims. G8 library fit is interface-checked for candidates; exact production integration remains open.

**Architectural fitness:** revise the target through F01–F12. The strongest changes remove work at the owner that causes it and retain existing semantic authority. A new generic cache, orchestration engine or custom numerical library would require its own evidence and is not part of this recommendation.

**Overall decision:** the current architecture is a strong implementation base, but the declared best-in-class execution target is not yet met in the identified scenarios. Adopt the target direction as a proposal and qualify each selected realization before describing it as implemented or faster.

### Authority changes to make explicit during implementation

| Proposed change | Rule/contract impact |
|---|---|
| Provider actions and reusable fill storage | Extend mathematical/provider demand, effect, extent and failure-publication contracts |
| Completion-driven block execution | Update readiness/commit ordering, aggregate iteration allocation, retained ownership and deterministic failure rules |
| Root-specific presolve eligibility | State primal versus multiplier/KKT reconstruction obligations without weakening optimizer requirements |
| Sparse regularity/rank product | Add a separately named method/scaling/cutoff contract; preserve existing SVD semantics |
| Threaded root or broader PETSc execution | Extend native team/process scope, ABI/build identity, admission and lifecycle qualification |
| Live sample microbatching | Preserve or explicitly revise visibility latency, backpressure and acknowledgement boundaries |
| Optional ISA/JIT/precision realization | Declare portability, arithmetic, construction/cancellation and equivalence requirements |

No code, repository rule, default solver setting or scientific tolerance was changed by this review. Follow-up disposition can be owned by the next performance plan; adoption of this document alone does not close implementation work.

## Source index

All repository links below are pinned to the reviewed commit. Symbol names in the findings identify the inspected implementation without relying on mutable branch content.

### Plans and architecture

[P42]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/plans/42-post-plan41-simulation-efficiency-and-diagnostics.md
[P43]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/plans/43-solver-diagnostics-and-performance.md
[P44]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/plans/44-pipeline-performance-optimization.md
[P45]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/plans/45-pipeline-execution-optimization.md
[P45F]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/plans/45f-short-run-qualification-and-publication.md
[P45B]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/plans/45b-mathematical-program-preparation-and-execution.md
[A1]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/authoritative_design/sections/architecture-overview.md
[A2]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/authoritative_design/sections/mathematics-and-compilation.md
[A3]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/authoritative_design/sections/numerical-execution.md

- [P42] — Plan42: simulation efficiency and diagnostics.
- [P43] — Plan43: solver diagnostics and performance.
- [P44] — Plan44: pipeline performance.
- [P45] — Plan45: current completion and follow-up dispositions.
- [P45F] — Plan45f: short-run qualification and publication.
- [P45B] — Plan45b: evaluator realization and preparation investigations.
- [A1] — Architecture overview.
- [A2] — Mathematical authority, compilation and execution contracts.
- [A3] — Native execution, physical acceptance, responses and dynamics.

### Production source

[S01]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-math/src/implicit.rs
[S02]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-math/src/execution.rs
[S03]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-math/src/assembly.rs
[S04]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-math/src/products.rs
[S05]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-structural/src/incidence.rs
[S06]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-compiler/src/workspace/modeling/conditional.rs
[S07]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/execution/runner.rs
[S08]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/presolve.rs
[S09]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/kinsol/block.rs
[S10]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/settings/kinsol.rs
[S10B]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/kinsol.rs
[S11]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/petsc.rs
[S12]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/dynamics/integrator.rs
[S13]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/dynamics/idas.rs
[S14]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/dynamics/integrator/adjoint.rs
[S15]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/square_response/actions.rs
[S16]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/square_response.rs
[S17]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/vendor/suitesparse-sys/vendor/SPQR/README.txt
[S18]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-kernels/src/lib.rs
[S19]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/assembled.rs
[S19B]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-backend-native/src/uno/callback.rs
[S20]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-math/src/library/admission.rs
[S21]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/math/solves/blocks.rs
[S22]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/math/initialization.rs
[S23]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/study_execution.rs
[S24]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/fitting/experiments.rs
[S25]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/fitting/profile.rs
[S26]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/live_simulation.rs
[S27]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/run.rs
[S28]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/result_projection.rs
[S29]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/result_snapshot.rs
[S30]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-runtime/src/workflow/modeling/canonical.rs
[S31]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/Cargo.toml
[S32]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/vendor/sundials-sys/vendor/CMakeLists.txt
[S33]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/crates/pse-structural/src/initialization.rs

- [S01] — Implicit factorization, full parameter jets, effect-aware nested reuse and projections.
- [S02] — Evaluator frames, order preparation, stage instantiation and provider execution.
- [S03] — Occurrence binding, same-point reuse and canonical sparse assembly.
- [S04] — Direct derivative-product compilation.
- [S05], [S33] — Structural analysis and execution-dependency merging.
- [S06] — Qualified regular implicit suppliers.
- [S07], [S08] — Root execution and presolve qualification.
- [S09], [S10], [S10B] — KINSOL block preconditioning, settings and native execution restrictions.
- [S11] — PETSc process ownership.
- [S12], [S13], [S14] — Diffsol/IDAS actions and adjoint checkpoints.
- [S15], [S16], [S17] — Sparse response factors, bounded dense spectrum and SPQR vendor source.
- [S18], [S19], [S19B] — Provider and native callback output contracts.
- [S20] — Scalar preparation cache admission.
- [S21], [S22], [S23] — Block and study orchestration.
- [S24], [S25] — Fitting experiment lanes and shared team sizing.
- [S26], [S27], [S28], [S29] — Live samples, supervision, grouped publication and result scans.
- [S30] — Typed candidate persistence and unchanged-object filtering.
- [S31], [S32] — Rust/native versions and build profiles.

### Primary upstream references

[U1]: https://symbolica.io/docs/automatic_differentiation.html
[U2]: https://symbolica.io/docs/numerical_evaluation.html
[U3]: https://coin-or.github.io/Ipopt/SPECIALS.html
[U4]: https://petsc.org/release/manualpages/PC/PCFIELDSPLIT/
[U5]: https://sundials.readthedocs.io/en/v7.1.1/kinsol/Mathematics_link.html
[U6]: https://sundials.readthedocs.io/en/v7.1.1/idas/Usage/ADJ.html
[U7]: https://coin-or.github.io/Ipopt/INTERFACES.html
[U8]: https://doc.rust-lang.org/cargo/reference/profiles.html
[U9]: https://doc.rust-lang.org/rustc/profile-guided-optimization.html

- [U1], [U2] — Symbolica AD and numerical evaluation; candidate realization capabilities.
- [U3], [U7] — Ipopt Hessian alternatives and callback contracts.
- [U4] — PETSc block/Schur preconditioner composition; current docs differ from the repository pin.
- [U5], [U6] — SUNDIALS 7.1.1 KINSOL mathematics and IDAS adjoint lifecycle.
- [U8], [U9] — Cargo profiles and Rust profile-guided optimization.

### Review standard

[R1]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/design_review/design_principles/standard.toml
[R2]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/design_review/design_principles/core/design-principles.md
[R3]: https://github.com/paul-heyse/pse-arrow/blob/5a009dccc7023e0142784892665c58bd20fbc305/docs/design_review/design_principles/profiles/process-simulator/review.md

- [R1] — Selected versions and binding.
- [R2] — Core foundations and acceptance rules.
- [R3] — Process Simulator review obligations.



