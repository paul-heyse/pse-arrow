# Execution performance target for pse-arrow

**Design review · 20 October 2026 · target purpose**

## 1. Decision and scope

The strongest target is a sparse, demand-driven numerical execution system in which prepared structure, current linearizations and compatible native storage survive exactly as long as their premises permit. The existing ownership model is a suitable foundation. The largest source-established opportunities concern the amount and granularity of work between those owners: dense reconstruction inside sparse implicit actions, repeated parent scans during block preparation, one native lifecycle per very small block, repeated IVP allocation, and orchestration that sometimes hides independent work behind a serial group or result projection.

The recommendation is to evolve those boundaries, retain the class-specific native libraries, and make original physical acceptance the fixed endpoint. Selecting a different universal solver, increasing every worker count, or compiling every expression cannot by itself address these mechanisms.

**Architectural fitness: revise.** Source inspection establishes execution-fit defects under supported growth and repeated-solve scenarios. **Behavioral adequacy: not qualified by this review.** The original-space acceptance paths inspected are important strengths; two adapter contracts and one receipt need correction, but this review does not establish that an invalid result was published as a valid solution. **Overall decision: revise the current execution design toward the target below; no implemented speedup or whole-product scientific qualification is claimed.**

### Baseline and available evidence

The inspected repository is [paul-heyse/pse-arrow at 1ee81798a9b884f3054b685ee9629e2c8d124498][baseline]. Its parent, 106aef5eca10f194bf1481cfd49ab255627d592d, is the campaign baseline. The intervening change affects only the external review context document, so the implementation reviewed matches that campaign.

The supplied [external review context][context] was read locally. The directory /home/paul/pse-arrow/build/external-pipeline-review-20261019 and its sibling tar archive are absent from the execution environment available to this review. Consequently:

- **Implemented / source-inspected** identifies paths traced in the repository.
- **Interface-checked** identifies capabilities checked against pinned or explicitly qualified upstream documentation/source.
- Campaign counts and failures are **briefing evidence**, not independently decoded diagnostic evidence.
- Every replacement mechanism and settling experiment below is **proposed**. No build, simulation, benchmark, speed multiplier or solver ranking was produced.

The review covers square simulation, nonlinear and conic optimization, implicit property mathematics, selected fitting and shooting paths, ODE/index-1 DAE execution, local and durable studies, result retention and diagnostics. It considers many cheap cases, fine triangular blocks, large coupled sparse systems, repeated parameter trials, skewed result sizes and cancellation. It does not certify all unit models, all thermodynamic regimes, higher-index DAEs, global optimization performance or industrial flowsheet capacity.

The selected standard is Core 3.4, Efficient Architecture Heuristics 1.0 and ProcessSimulator 1.5, with the pse-arrow binding. The accountable reviewer is ChatGPT, integrating independent focused reviews of mathematics/preparation, numerical execution and workflow. No simulator implementation was changed. This document is the sole principal assessment; proposed follow-up owners are given in Section 9. Upon adoption, the owning implementation plan should hold dispositions, without a second parallel backlog.

## 2. What the current pipeline already does well

The architecture has meaningful separations that should survive optimization:

| Responsibility | Current authoritative boundary | Preservation constraint |
|---|---|---|
| Physical meaning | Authored typed definitions, roles, domains and provider contracts | A fast evaluator cannot redefine validity, units, basis or conservation. |
| Structural preparation | Compiler, immutable admitted products and Salsa dependencies | Reuse structure after value changes; rebuild after consumed structural changes. |
| Mathematical realization | Symbolica/Numerica integration, demanded derivative products, sparse layouts | Preserve exact demand, source checks and provider effects. |
| Numerical iteration | Class-specific native adapters and admitted profiles | Libraries own iteration, globalization, factors and model management. |
| Scientific outcome | Original-coordinate residual, bound, domain and conservation assessment | Native status is evidence, not scientific acceptance. |
| Execution lifetime | Attempt-owned workers, native sessions and finite resource admission | Mutable native state remains private; cancellation joins real work. |
| Result representation | Arrow/DataFusion selection, retention and durable publication owners | Detail, continuation seeds and durable acknowledgement have distinct lifetimes. |

These are current contracts and executing paths, rather than a proposed new layering. The source already contains incremental preparation, compilation sharing across repeated mathematical instances, demand-specific products, worker scratch reuse, exact affine primal reduction, KLU symbolic reuse, KINSOL matrix-free actions and preconditioning, selected forward and adjoint sensitivities, fitting-team concurrency, completion-driven study/block scheduling, store-free local studies, Compact retention and bounded selected output. [Architecture][architecture], [assembly][assembly], [native/session execution][staged], [studies][study].

In particular, the local scheduler already refills on completion; replacing a wave barrier is not the opportunity. Compact already releases full reports once selected capture completes; retaining every report for the entire study is not the diagnosed problem. Ordinary diagnostic writes use a nonblocking try-lock and record lost observations; they are not a blocking mutex around all solver work. [Study execution][study], [selected retention][demand], [observation implementation][observation].

### Physical and numerical semantics

| Element | Units, basis and convention | Execution interpretation and authority |
|---|---|---|
| Model variables and equation contributions | Authored physical dimensions, material basis and reference conventions | Compiler/model admission defines roles and mappings; native arrays are derived coordinates. |
| Normalized variable | x = S z, with declared positive nominal scales | Normalization improves numerical behavior; original x and its per-variable budget remain authoritative. |
| Residual and constraint budgets | Original physical row units, mapped through declared row scales | Independent assessment uses the original requirements. A method-specific relative norm is not automatically the same requirement. |
| Provider evaluation | Declared physical input context, envelope, branch and derivative capability | A changed complete context or regime invalidates dependent numerical state even when outer input numbers coincide. |
| Dynamic states | Declared differential/algebraic roles, initial conditions and event semantics | Consistent initialization and segment/event handling precede valid continuation; old integration history is not a generic cache. |
| Cones | Declared cone coordinates and common scaling within nonlinear cone blocks | The original reconstructed b − Ax is assessed geometrically. Native slack alone is insufficient. |

The engineering target of 0.001 must not be loosened for speed. Rank thresholds, derivative verification, integrator error weights, linear-solve accuracy and native stagnation safeguards are distinct controls, not interchangeable uses of that number. The current normalized transport and original assessment provide the right owners for expressing those distinctions. [Numerical policy][numerical-design], [transport][transport].

**Well-posedness.** Variable roles are authored rather than inferred from the presence of values. Structural matching, degree-of-freedom analysis and decomposition belong to preparation; they are distinct from numerical rank and convergence. A structural block is useful dependency information, not proof of a unique nonlinear root, a nonsingular numeric block or a desirable native invocation size. The proposed execution grouping must retain diagnostics in original model terms. [Numerical execution §§15–17][numerical-design].

## 3. The target architecture

### 3.1 Separate six lifetimes

The implementation need not add six new public types. These are the ownership distinctions that should govern existing types. Their dependency scopes overlap: a bound or fixed-role change can also invalidate structural admission, presolve or derived products even when equation incidence is unchanged.

| Lifetime | Retained contents | Invalidation |
|---|---|---|
| Model/structure | Equation topology, incidence, block DAG, row/column ordinals, sparse patterns and source mappings | Consumed physical definition, topology, coordinate or capability changes |
| Executable mathematics | Already-demanded evaluation functions and immutable compiled modules | Mathematical body, selected outputs, product/layout, evaluator policy, library/ABI or supported ISA changes |
| Case binding | Values, bounds, nominals, budgets, selected regime and actual start provenance | Changes to those case inputs |
| Current linearization | Sparse state matrix, factor/preconditioner and complete provider linearization association | Point, coefficients, provider context, branch, relevant scaling or method state changes |
| Execution lane/attempt | Private evaluator scratch, compatible native allocation capacity and callback storage | Compatibility or owner change; explicit full numerical reset for a new problem |
| Result/commit | Mandatory scientific facts, minimal continuation product, selected detail and pending durable effects | Consumer completion, retention policy and acknowledged publication/drain |

The main performance rule is: **changing an action vector should not invalidate a reusable state matrix; changing a case value should not invalidate unrelated structure; changing result detail should not independently determine numerical preparation.** The converse is equally important: unchanged array dimensions are insufficient to preserve numeric factors, branch validity or history.

Prepared plans should retain sparse layouts and indexed views. Native lanes should bind current values and invoke sufficiently large useful work. Evaluation should expose the exact operations consumers need—values, selected Jacobian entries, Jv, transpose products, weighted Hessians or Hessian-vector actions—without materializing a larger product merely to discard it. A single CPU/memory admission policy should allocate resources across cases, blocks, experiments, shooting windows and native teams.

Original assessment remains on the scientific critical path. Selected formatting, encoding and durable commit can have separate bounded execution lifetimes where the public completion contract permits it. They must retain their producers and drain before their own completion is reported.

### 3.2 Workload-dependent numerical routes

“Best in class” is a portfolio selected by actual problem structure and requested assurance. Existing class/capability routing and advisory observations are a suitable owner; another universal policy framework is unnecessary.

| Workload | Preferred target to qualify | Main decision |
|---|---|---|
| Affine or safely reducible equations | Existing library-owned exact elimination and original reconstruction | Is the reduction exact, guarded and cheaper than keeping the full system? |
| Fine triangular nonlinear chain | Group inexpensive semantic blocks; compare one fixed-pattern sparse solve | Native setup and boundary crossings versus useful nonlinear work |
| Large coupled smooth root | Sparse direct Newton or analytic Jv with a strong sparse/block preconditioner | Factor fill/memory versus Krylov iteration and preconditioner cost |
| Smooth sparse NLP | Native optimizer with exact sparse derivatives when economical | Total preparation/evaluation/factorization time, not iteration count alone |
| Curvature with expensive fill or expression growth | Qualified limited-memory Hessian mode; direct HVP where the native method consumes it | Exact curvature benefits versus whole-solve cost |
| True least-squares estimation | Existing fitting NLP route with sparse response derivatives and appropriate Gauss–Newton structure | Do not reinterpret a general objective as least squares |
| Repeated dynamics | Selected forward sensitivities or adjoints according to input/output demand; compatible native storage | Total trajectory, sensitivity and checkpoint cost |
| Multiple shooting | Independent fixed-trial windows on bounded private lanes | Available window parallelism, checkpoint memory and outer-team admission |
| Convex QP/conic batches | Lawful native batch/update operations with individual original checks | Actual profile compatibility, thread extent and update restrictions |
| Global/certified work | Explicit assurance-preserving global route | A fast local solve cannot replace a requested global certificate |

Ipopt documents the relevant tradeoff between exact second derivatives and limited-memory approximation: fewer iterations need not mean less total time when Hessian computation/factorization is costly. This is a qualification choice, not a recommendation to switch all NLPs to L-BFGS. [Ipopt special features][ipopt].

## 4. Highest-value numerical findings

### F01 — Implicit derivative actions reconstruct a dense state Jacobian before sparse factor reuse

**Diagnosis: implemented/source-inspected. Priority: high for nested properties and repeated actions.**

In Problem::demanded_derivatives, the action route allocates/resizes an n by n tangent matrix, issues n unit-seed residual JVPs, copies every n-vector into it, then extracts the entries in the known sparse pattern and calls factorize_ift. This applies to the action route, not every Value, CompleteJets or SelectedColumns path. [Implicit product construction][implicit-products].

The existing factor reuse test occurs after this reconstruction. Consequently a factor hit can still pay for dense assembly and n residual traversals. For k directions, the inspected path performs n + k first-order residual products for forward or transpose actions; its weighted-second route performs n + 3k + 1 first/second residual products. These counts exclude the nonlinear root solve, value checks and factorization. They establish unnecessary work, not elapsed speedup.

For F(y,p) = 0, the required operations are:

$$
F_y\,\delta y=-F_p\,\delta p,
\qquad
F_y^T\lambda=w,
\qquad
J^T w=-F_p^T\lambda.
$$

The state matrix and factor depend on the current qualified state. Direction vectors, cotangents and weights determine right-hand sides and actions. Those lifetimes should be separate.

**Correction.** Retain a sparse F_y and its complete association at the implicit-state owner. Prefer a directly requested unknown-coordinate sparse Jacobian; qualify seed coloring only where useful. Bind each new action against that state and use library-owned forward/transposed linear solves. Do not cache an unbounded set of Krylov directions.

The existing identities support this separation: the root numerical key excludes action seeds; current product matching includes exact demand/action bits; factor association includes actual point, sparse coefficient bits and ordered child linearization tokens. [Implicit state and factors][implicit].

**Safety boundary.** Provider-free equal-premise/equal-point residuals can skip the n probes directly. Residuals containing providers currently re-solve and validate their effects even at identical input. Preserve that behavior. A current structurally First traversal may establish complete child tokens before matrix reuse, but zero-direction visitation, winner changes and output subsets must be verified. Missing or changed tokens require reconstruction/refusal. Ordinary input equality cannot substitute for current regime validation.

**Settling experiment.** Diagonal and banded n = 8, 32, 128 implicit systems; change only action directions, then change one provider context, branch winner or point. Record residual traversals, matrix bytes and symbolic/numeric factor counts. The correction should remove repeated n-probe construction without allowing stale child completion.

### F02 — Sparse HVP preparation hard-codes a complete-support cutoff for directional construction

**Diagnosis: implemented/source-inspected. Priority: high where sparse curvature preparation is large.**

compile_direct_product uses a directional-gradient route only when the supported Hessian pair count is exactly the complete triangle. Otherwise it constructs supported second partials and combines them with the direction. A nearly dense support with one missing pair can therefore take a qualitatively different preparation route from complete support. [Direct mathematical products][products].

**Correction.** Make direct action construction a candidate for sparse support too: differentiate the weighted gradient in the supplied direction through the existing mathematical library. Retain entry construction where genuinely cheaper or where a consumer needs actual entries. Choose from expression/instruction growth and complete preparation-plus-evaluation evidence, not from the Boolean property “all pairs exist.”

This is a mathematical-product integration change, not a request to implement a second automatic-differentiation engine. Guards, provider capabilities, action binding identity and exact-versus-approximate derivative claims remain unchanged.

**Settling experiment.** Compare diagonal, banded, dense and one-edge-missing curvature under identical demanded HVPs. Measure symbolic work, executable size, preparation time and repeated action time against analytic results. No runtime complexity advantage is asserted solely from symbolic call counts.

### F03 — Block preparation repeatedly scans the parent problem, and semantic blocks become overly fine native work

**Diagnosis: implemented/source-inspected. Priority: high for decomposable models.**

Conditional capability/allocation analysis and row selection repeatedly visit parent structures per block. Runtime block preparation also reconstructs views by scanning parent row/column collections. With B blocks and R rows, these inventory passes can require B × R work; a chain with B proportional to R exposes quadratic preparation work before numerical difficulty is considered. [Mathematical assembly][assembly], [compiler workspace][workspace], [block execution][blocks].

The block route then prepares each conditional problem and runs a native lifecycle for each block. Its initialization profile requests fresh reuse. The ready-block scheduler already supports concurrency, but it cannot remove the overhead of a long chain of tiny dependent solves.

**Correction, in two independent parts.**

1. Build block views using retained row/column ordinals and existing contribution indexes. Preserve source order, multiplicity, boundary-fixed variables and allocation/capability checks. Construct parent-to-request and consumer maps once for derivative-order upgrades instead of repeated searches across all instances.
2. Keep semantic BTF decomposition separate from physical execution grouping. Coarsen inexpensive connected blocks or use the complete sparse direct route where better. Preserve meaningful large coupled regions and expose independent groups to the existing bounded scheduler.

Coarsening must preserve declared root/branch selection, initial-start semantics and applicable per-block controls. Qualify changed solve trajectories and basin behavior; algebraic equivalence plus a small final residual does not establish the same selected implicit solution.

A scalar nonlinear equation does not automatically justify bracketing or nonlinear elimination. Such a route still needs continuity, domain and root-selection premises. Exact affine reduction already exists and should be used where eligible.

The briefing reports that the 256-variable banded case stopped in preparation before native execution. That makes preparation evidence valuable; it does not prove these scans caused that stop. [Campaign briefing][context].

**Settling experiment.** Independent diagonal blocks, triangular chains at 64/256/1,024, a large coupled SCC and skewed mixed blocks. Compare indexed preparation, coarsened groups and complete fixed-pattern sparse execution. Count examined parent entries, compiled requests, native setups and actual critical-path intervals, retaining identical original checks.

### F04 — KINSOL's relative step safeguard is described as an absolute physical step test

**Diagnosis: implemented/source-inspected and pinned native-source checked. Priority: high for reliable convergence policy.**

The adapter states that its scale construction makes the native step test equivalent to the physical step budget. Vendored SUNDIALS 7.1.1 computes a different quantity. [Project adapter][kinsol], [vendored KINScSNorm][kinsol-native].

Let x = S z, let o be the normalized one-sided-bound shift, and let native coordinates be u = x/S − o. The production path sets native variable scale to q S_i/t_i, where q is the minimum normalized budget and t_i is the original variable budget. Let a be the effective scaled-step tolerance. The native condition is equivalent to:

$$
|\Delta x_i|
\leq
(a/q)t_i+a|x_i-S_i o_i|.
$$

In the ordinary a = q case this becomes t_i + a|x_i − S_i o_i|, not t_i. It depends on coordinate magnitude and bound shift. The existing scale-times-deviation test does not exercise this denominator.

This does **not** establish false successful publication: KIN_STEP_LT_STPTOL maps to a Limit outcome, and independent original assessment remains. It does not establish the cause of the briefing's 64-variable failure. It does establish an incorrect method-control interpretation and a plausible source of early stagnation/fallback in affected coordinates.

**Correction.** Treat native stagnation independently from physical residual acceptance. Derive a conservative method-specific threshold, evaluate roundoff-based native defaults as a candidate, and allow only finite recorded refinement within the original work allowance. Fix the documentation and native-facing test. Keep physical budgets unchanged. SUNDIALS itself documents separate residual and step controls and the ambiguity of a step-small return. [KINSOL 7.1.1 usage][kinsol-doc].

**Settling experiment.** F(x) = exp(x − A) − 2 from x = A, varying A, sensible versus identity normalization, and one-sided shift. Inspect actual native status and original residual under different step safeguards. This is a proposed probe, not a measured failure.

### F05 — Clarabel working tolerance lacks a guaranteed relation to absolute original cone acceptance

**Diagnosis: implemented/source-inspected and exact pinned-source checked. Priority: high for successful accepted conic execution.**

The adapter assigns accuracy.feasibility to tol_feas. Clarabel 0.11.1, tag v0.11.1 at commit 25540f559592068d0c8a80e46ded1b21760212a1, uses relative primal and dual residuals. Undoing equilibration/homogenization, before reversal of other transformations, its primal measure is:

$$
r_p=
\frac{\|A\bar x+\bar s-b\|_2}
{\max(1,\|b\|_\infty+\|\bar x\|_2+\|\bar s\|_2)}.
$$

Its Solved test requires relative primal/dual feasibility plus an absolute-or-relative objective gap condition; the absolute gap option is not an absolute feasibility test. [Project cone adapter][conic], [pinned Clarabel info][clarabel-info], [pinned problem data][clarabel-data].

The original assessment correctly computes b − Ax and checks its actual cone violation. The briefing's SOC 0.004418 and PSD 0.001919 violations exceed 0.001 and must remain failures. Native slack substitution would conceal the error. These reported failures are consistent with the norm distinction; the original attempts are needed for complete causal attribution. [Briefing][context].

**Correction.** Translate physical cone requirements into a justified working tolerance where magnitude and recovery bounds exist. Otherwise use finite, recorded tightening/re-solve and fresh original acceptance. A denominator from a previous solution can guide refinement but is not a bound on the next solution. Presolve/chordal recovery, block scaling, native slack cone error and floating-point error must be included.

For direct normalized SOC/PSD coordinates, cone-feasible native slack and a certified denominator bound permit conservative residual-to-geometric bounds: SOC violation is at most sqrt(2) times the equality-residual 2-norm, and PSD minimum-eigenvalue violation is at most that norm under the svec isometry. This is conditional mathematical reasoning, not a universal formula for every cone or preprocessing route.

**Settling experiment.** Scale coefficient and solution magnitudes while holding original physical requirements fixed. Compare native residuals, actual b − Ax violations, refinement counts and accepted end-to-end time. Test transformed and direct routes separately.

### F06 — Repeated IVPs recreate compatible native storage, while callbacks allocate temporary arrays

**Diagnosis: implemented/source-inspected. Priority: high for short repeated trajectories; conditional for long stiff solves.**

Shooting retains a DynamicWorker, but on the IDAS path each integrate_window invocation reaches a new IDAS Session and later frees its native context, vectors and solver storage. Event/segment reinitialization within a trajectory already exists; the gap is across compatible windows/trials. [Integrated workflow][integrated], [IDAS implementation][idas], [shooting][shooting].

Residual and sensitivity callbacks additionally copy native state/rate vectors, construct residual Vecs and allocate sparse-times-dense inputs/results. These allocations are visible in source; their share of total integration time is unmeasured.

**Correction.** A bounded lane should own compatible IDAS allocation capacity and flat callback scratch. Reset initial conditions, consistent initialization, numeric setup, integration history, sensitivity layout, checkpoint state, callback context and counters for each new trial. Reinitialize only within the native compatibility contract. IDAReInit supports same-size restart without reallocating its internal memory and deletes the previous solution history. It does not automatically reset every external option or sensitivity/adjoint owner. Shape-changing or unsupported sensitivity/adjoint layouts must rebuild. If reset or update fails after mutation, discard the uncertain native owner. [IDAS 7.1.1][idas-doc].

Use checked borrowed native slices for the synchronous callback where aliasing permits, and reusable buffers for required gathers/products. Preserve the faer/library algebra rather than introducing bespoke sparse multiplication.

**Settling experiment.** Many short compatible windows and long stiff trajectories separately; record session construction, callback allocations, setup work, integration work and total time. Include mode changes, inconsistent initial conditions, failure followed by retry, cancellation and checkpoint drain.

### F07 — Independent multiple-shooting windows execute serially at a fixed optimizer trial

**Diagnosis: implemented/source-inspected. Priority: high when window work is significant.**

The current evaluation loops windows sequentially with one WindowWorker; the integral adjoint loop is also sequential. At a fixed NLP trial x, each window's anchor is supplied by x. The later continuity constraints connect their results, so these integrations can be independent at that trial. This does not apply to ordinary single shooting or sequential initial-guess construction. [Shooting evaluation and adjoints][shooting].

**Correction.** Reuse the bounded fitting experiment-team pattern for independent shooting windows. Each lane owns provider/evaluator/native state. Reduce results in declared order, install a new point cache only when the required evaluation is complete, and preserve the previous valid cache on refusal/failure as appropriate. The outer optimizer must lend CPU budget rather than multiplying nested pools; checkpoint memory can bound width before CPU does.

**Settling experiment.** 1/4/16/64 anchored windows under fixed total CPU and memory budgets, with skew and failure. Verify actual native overlap, derivative agreement, bounded live checkpoints and joined cancellation.

### Capability expansion C01 — Strong preconditioning for coupled DAEs

The current IDAS matrix-free routes offer GMRES/FGMRES with None/Jacobi, plus the sparse-direct alternative; coupled BlockFactor preconditioning is explicitly refused. KINSOL already has a stronger block factor option, so “add preconditioning” would be an inaccurate whole-project diagnosis. [Dynamic capability admission][dynamics], [IDAS][idas], [KINSOL][kinsol].

For large coupled index-1 systems, qualify an approximate block-triangular, sparse ILU or Schur preconditioner for the actual Newton operator J = c_j M − f_y. Its refresh must account for c_j, current state and regime. Structural partitions do not establish invertibility. The exact Newton operator must retain differential/algebraic coupling. Qualify whether the approximation captures enough coupling to control Krylov work; dropped couplings are a performance tradeoff, not a change to the original equations.

Use existing native callback interfaces and library-owned sparse factors. PETSc 3.24-family field-split/Schur facilities are another library capability to evaluate, but they are not presently exposed by this project's PETSc PC vocabulary, and nonlinear NASM is a different operation. Upstream 3.24.6 documentation is family evidence, not qualification of the project's 3.24.0 build. [PETSc field splitting][petsc-fieldsplit].

## 5. Code generation, scheduling and result execution

### F08 — Incompatible native bulk groups can serialize independent cases

**Diagnosis: implemented/source-inspected. Priority: high for heterogeneous cheap studies.**

The local scheduler groups eligible ready cases by backend. The native batch owner subsequently requires matching strategy profiles, threads and direct-composition legality. If any member disqualifies the group, it executes the members serially through one session. [Outer grouping][study], [native batch legality/fallback][staged].

This is reachable through public per-point operations. For example, otherwise identical fresh POUNCE-convex cases can have history = 256 and history = 257. That public reporting control participates in the complete controls/profile identity; the group can pass backend-only grouping and fail inner compatibility. This is a source-traced example requiring both preparations to be ready in the grouping turn, not an executed reproducer. Standard homogeneous sweeps clone the same settings and normally avoid this particular mismatch. The finding does not explain the campaign's homogeneous worker contrast without its returned profile/batch evidence. [Controls][solve], [profile construction][solves], [study admission][study-model].

**Correction.** Expose the numerical owner's admitted batch compatibility before outer grouping. Partition incompatible members into independent futures. Start with the complete effective profile, thread extent and current composition restrictions; refine reporting-only identity separately if useful. Keep final native safety checks and individual assessment. Do not duplicate strategy policy in the workflow or parallelize mutable operations inside a session whose contract is serial.

**Settling experiment.** A compatible pair should use actual batch execution; a legitimate incompatible pair at width two should retain independent dispatch. Record native intervals and member counts, not configured width alone.

### F09 — The local study coordinator repeatedly scans the pending frontier

**Diagnosis: implemented/source-inspected. Priority: medium/high for many cheap cases.**

The frontier initially contains every point. Each scheduling turn copies remaining candidates and evaluates readiness before checking free capacity. Ready candidates left over after capacity is full remain in the frontier for the next scan. Fixed-width independent studies can therefore perform quadratic-in-case-count pending inspections. The immediate-predecessor policy itself is appropriately scoped. [Study scheduler][study].

**Correction.** Keep an ordered dispatchable queue and an indexed waiting/dependency set. Enqueue on admission, relevant predecessor changes, retries or cancellation policy changes; consume only enough ready members to fill capacity. Preserve current continuation-lane priority and stable placement. A queue and membership index are sufficient; another scheduling framework is unnecessary.

**Settling experiment.** Count candidate/policy visits over 100/1,000/10,000 cheap independent occurrences and sparse DAGs at fixed width. Expected work should track occurrences plus changed edges. Four- or sixteen-point timings alone cannot establish this growth behavior.

### F10 — Compact extraction runs on the sole completion coordinator

**Diagnosis: implemented/source-inspected. Priority: medium, increasing with selected detail.**

After a lane completes, the main loop awaits record_execution, which awaits Compact capture and exhausts selected relation cursors before processing other completed lanes and refilling. Full retention largely keeps an Arc at that point. Other native work can continue, but other futures in the same FuturesUnordered are not polled during the capture await. [Study completion loop][study], [capture_detail][demand].

**Minimal correction.** Move selected capture into each existing finite lane future. Retain its Arc of RunResult, source/preparation scope and charged producer owners through terminal cursor exhaustion. Return a completed payload containing mandatory facts, continuation product and retained result. The coordinator can poll other lanes while that lane projects, provided projection retains bounded work per poll, with no new task pool or queue.

A cursor is a borrowed, polled producer, not an independently running export service. A final nonempty chunk can precede a terminal producer error. Do not drop its parent, infer success from that last chunk, or report the whole study finished while issued capture is undrained. Optional detail failure must remain distinct from scientific completion. [Result cursors][results], [producer exhaustion][result-export].

Parking native capacity before capture and a separate bounded commit/retention stage are further optimizations, requiring explicit continuation ownership and bounds on uncommitted full reports. They are not prerequisites for the smaller lane-local fix.

**Settling experiment.** Suspend one selected producer after its last chunk but before terminal return, including an injected terminal error. Other lanes should complete/refill; the first must not report falsely complete detail; cancellation must drain every issued owner.

### F11 — Original-quality row recovery performs repeated linear ID searches

**Diagnosis: implemented/source-inspected. Priority: high-confidence small correction.**

The transport recovery path searches the complete contract row list by semantic ID for every quality row. For a complete m-row report this entails approximately m(m + 1)/2 comparisons. It adds serial quadratic work after a potentially sparse solve. [Quality coordinate recovery][transport].

**Correction.** Prepare a semantic-ID-to-ordinal map or validate positional correspondence once and use it directly. Preserve subset/reordered reports and explicit unknown-ID errors. Independent original checks remain; only the incidental lookup work changes.

**Settling experiment.** Compare large full and subset/reordered quality reports; count ID comparisons and verify identical recovered rows. This is a useful independent improvement without waiting for a numerical redesign.

### F12 — Each scalar compiled evaluator pays its own compiler/linker lifecycle

**Diagnosis: implemented/source-inspected. Priority: high where preparation produces many small native functions.**

Repeated semantic instances already share artifacts. Arithmetic-region construction also performs joint optimization across eligible independent siblings. Compilation is demand-driven across selected routes, eager within an admitted executable request, and lazy for mutable worker instantiation. It would be incorrect to describe all expression compilation as per-instance or per-callback. [Assembly requests][assembly], [math service][math-service], [mathematical execution][math-execution].

For a missing compiled artifact, however, each evaluator exports one function into its own temporary directory and invokes separate compiler and linker children. M such modules create 2M compiler-driver invocations. Every requested artifact is awaited during executable assembly. [Native compiled owner][native-compiled].

**Correction.** Collect already-demanded arithmetic entrypoints into bounded multi-function modules and attach named handles to existing stages. Preserve host execution order, guard checks, branch selection, providers, cancellation, compiler-child drain and publish-after-success. Module ownership should follow co-used executable families so one surviving function does not retain unrelated code indefinitely.

This does not require evaluation fusion. Symbolica 3.0.1 supports loading another named function from a shared library with independent evaluation storage. Its scalar BatchEvaluator implementation is a loop over scalar evaluations; adopting that API alone does not remove scalar arithmetic/FFI calls. [Symbolica compiled evaluator][symbolica-compiled], [Symbolica source][symbolica-source].

**Separate expansion C02.** Qualify provider-free tiled arithmetic and actual SIMD/JIT only after the same semantics are preserved. Include layout packing, action lanes, tail cases and executable/scratch memory in the comparison. Keep explicit conservative floating-point/compiler flags: upstream defaults must not silently authorize fast math or host-native ISA. Current §14.5 explicitly excludes JIT/SIMD/GPU/distributed execution; changing that scope is a rule impact, not an already available production feature. GPU execution is worth evaluating only for workloads with enough coherent resident work to amortize transfer and launch costs; this campaign cannot settle that choice.

**Settling experiment.** Identical demanded programs under scalar modules, packed modules and separately qualified vector realization. Compare compiler children, preparation span, code bytes, warm evaluation and complete simulation latency, including provider/guard errors. Cold and warm results must remain separate.

### Native concurrency limits are part of the target

More admitted case lanes do not imply more simultaneously executing native solvers.

| Path | Inspected constraint | Target implication |
|---|---|---|
| PETSc MPIUNI | One-thread adapter and a process-wide admission guard | Preserve required exclusion; consider qualified isolated worker processes/concurrent-capable build only if this workload justifies it. |
| Uno and direct HiGHS | Uno takes the exclusive shared scheduler owner; direct HiGHS takes shared ownership | Uno serializes against these owners; direct HiGHS is not universally serial. |
| SCIP concurrent mode | Process-static TPI pool gated for multi-threaded mode | Independent serial instances and internally concurrent cases have different parallelism contracts. |
| POUNCE-convex | Multi-member batch uses its admitted Rayon pool; members factor serially inside it | Compare across-case allocation with within-case allocation; member count does not establish worker count. |
| Other teams | CPU extent and scoped native thread settings are admitted centrally | Avoid multiplying case width by fully allocated inner teams. |

These are source limitations, not reasons to remove protective gates. A future process-isolated lane must preserve cancellation, complete native destruction, identity, resource accounting and bounded transfer of prepared inputs/results. Warm processes could amortize startup, but benefit is unmeasured. [PETSc owner][petsc], [Uno][uno], [HiGHS lifecycle][highs-lifecycle], [SCIP][scip], [POUNCE-convex][pounce-convex], [native sessions][staged].

## 6. Diagnostics and durable execution

### F13 — Ordinary observation performs several shared updates per callback

With ordinary collection enabled, a callback publishes call/phase events, then completion count, elapsed duration, cumulative time, CPU availability and phase exit. Each observation update reaches shared state through try_lock and a clock read. Disabled and detailed modes are already distinct. There is no source basis for calling this a blocking global solver lock. [Callback path][callback], [observation state][observation].

**Correction.** Retain exact attempt-local counters and predeclared fact slots, then publish coalesced snapshots at defined safe boundaries. Keep mandatory work-limit admission exact and separate from optional observation. Detailed per-call records remain an explicit diagnostic mode. If ordinary live views promise per-return freshness today, coalescing intentionally changes that contract and needs a documented cadence/loss meaning.

The existing 45 observer comparisons are the right starting evidence when available. Do not assign an overhead percentage from the number of updates alone. A rich callback can amortize this work; a tiny scalar callback may not.

### F14 — Uno's start receipt confuses primal initialization with the optional native warm payload

The primal seed goes through pipeline.initial, is copied into Uno's primal buffer, and is passed to its native bridge. The receipt sets start_submitted from warm.is_some(), while primal-only starts may legitimately lack that payload. [Start pipeline][start-pipeline], [Uno adapter][uno].

Correct the receipt to represent the start actually supplied, preserving its provenance. Do not invent dual seeds, diagnose a cold fallback, or claim a speed improvement from correcting the flag. This matters because otherwise warm-start performance comparisons are misinterpreted.

### Capability expansion C03 — Shared bulk dispatch and bounded durable commit

The local execution path can dispatch multiple eligible cases through the native bulk surface. Durable workers currently claim/start one occurrence at a time and hold the lane through joined publication and bookkeeping. Required durable finish drains progress, projects tables, stores seeds and records the terminal effect. [Durable worker][worker], [study operation][study-operations], [run finalization][run], [durable finish][durable].

The target should let local and durable orchestration consume one numerical bulk-admission capability, while retaining per-occurrence fences, leases, outcomes and acknowledgement. A bounded commit stage may free compatible preparation/native capacity while results await actual durable completion. It must bound full products awaiting commit, maintain heartbeats and recover uncertain acknowledgements without blindly rerunning science.

This is an architectural extension, not a claim that publication is unnecessary or that it dominates the briefing's roughly 35-second durable example. Start with shared compatibility/dispatch; only separate commit ownership where observed latency warrants it.

A narrower reuse refinement is also available: the full StudyOperation, including output demand, participates in preparation identity. Public local studies have one shared output demand, so alternating Full/Compact within such a study is not a valid reproducer. The legitimate opportunity concerns a durable lane serving consecutive/interleaved scientifically identical studies with different retention demand. Exclude only nonconsumed presentation detail from scientific preparation identity; dynamic demands that alter requested products remain real dependencies. Shared lower caches may already avoid recompilation, so the gain is unmeasured. [Study operation identities][study-operations], [public study request][study-model].

## 7. What the campaign does and does not establish

The briefing describes an unusually useful qualification corpus, but it must be interpreted at the level of its actual subjects:

| Briefing evidence | Correct interpretation for this review |
|---|---|
| 99 science configurations: 85 completed, 5 assertion failures, 9 interrupted | Coverage and failure leads; “completed” is not a blanket best-in-class performance result. |
| 4 preparation configurations completed | Preparation has dedicated evidence; inspect both cold and reused work. |
| 36 Rust and 5 Python controls passed | Behavioral controls, not 41 representative industrial simulations. |
| 45 observer comparisons and 40 pipeline comparisons, five fresh-process repetitions | Descriptive contrasts are available; inspect individual pairs/attempts and median/range. |
| 139 indexed packs and 686 decoded Arrow streams | Readback coverage; not proof that every intended scientific configuration passed. |
| 169 receipts | Includes original, repaired and repeated attempts; not 169 independent models. |
| All 11 native adapters had execution/segment evidence | Exercised adapters, not an equal-workload solver ranking. |
| 64-variable banded root failed; 256 stopped during preparation | Different execution boundaries; no single “sparse solver problem” is established. |
| SOC/PSD native success but failed original acceptance | Preserve those failures and investigate working-accuracy translation. |
| Durable success, stack-overflow retry and separate exit 137 incident | Inspect each incident's evidence; neither is a universal diagnosis of scheduling or memory. |

All counts in this table come from the [supplied context][context], not direct readback in this review.

The scientific harness used a release-derived profiling build with force_validate; Python/worker correctness used different dev conditions. Some compilation overlapped, and the host was not a controlled benchmark machine. Inclusive preparation/solver/callback/projection clocks overlap and cannot be summed into a wall-time decomposition. Cooperative stop plus drain can exceed a nominal timeout. A configured worker count is not observed native concurrency; a new allocation ID is not necessarily semantic rebuilding; retained pool capacity is not RSS.

Many timeline packets were lost, CPU-sample loss was not fully established, and the bundle does not retain exact historical ELFs or a separately portable symbol sidecar, so arbitrary re-symbolization of old raw perf data is not established. These limitations affect detailed speed attribution, not the source-established repeated-work findings.

The next evidence pass should start with the bundle's REVIEW/readme, qualification review-guide summary, repaired paired-comparison summaries and raw trials. Then inspect the preparation-stop and banded-root attempt observations, conic acceptance failures, implicit derivative controls, provider/PC-SAFT preparation, durable runs and the observer contrasts. Preserve original and repaired attempts rather than selecting only successful later receipts.

The retained fixtures emphasize smaller synthetic structures; a banded dependency chain is not a large coupled sparse stress test. After resolving the concrete mechanisms above, qualify representative flowsheets with coupled thermodynamics, regime changes, poorly conditioned recycles, sparse NLP fill, long stiff trajectories and skewed studies. Their purpose is to choose routes and limits that small fixtures cannot distinguish.

## 8. Priorities and evidence that should decide implementation

Priorities below reflect consequence, source evidence and breadth of benefit. They are not a measured ranking or a staffing schedule. Independent small corrections should proceed without waiting for larger designs.

| Order | Recommended work | Why it belongs here | Proposed owner |
|---|---|---|---|
| Immediate independent corrections | F11 indexed quality recovery; F04 native step-control contract; F14 truthful Uno start receipt | Removes demonstrated incidental work and repairs interpretation needed for reliable comparisons | Native transport, KINSOL and Uno integrations |
| First major numerical work | F01 qualified sparse implicit linearization; F02 action-based curvature alternatives | Removes unnecessary mathematical work in repeated derivative consumption | pse-math implicit/product owners |
| First preparation work | F03 indexed conditional views and request-consumer maps; qualify physical block grouping | Prevents problem size from multiplying unrelated preparation and lifecycle work | Compiler, math assembly and numerical composition owners |
| Repeated dynamics | F06 compatible storage/callback reuse; F07 bounded independent windows | Targets repeated integrations and a concrete parallel region | IDAS integration and shooting composition |
| Study execution | F08 admitted batch partition; F09 event-local readiness; F10 lane-local capture | Restores exposed concurrency and keeps representation work from blocking the coordinator | Numerical binder and study executor |
| Larger numerical scope | C01 coupled DAE preconditioning; workload-specific direct/Krylov and curvature choices | Can change solve complexity when coupling/fill dominates | Native capabilities and numerical policy |
| Preparation and operating refinement | F12 module packing; F13 observation coalescing; C03 bulk/commit unification | Reduces repeated external crossings and downstream coupling | Native mathematical compilation, diagnostics, durable runtime |
| Qualified expansion | True SIMD/JIT, process-isolated native lanes, GPU/distributed execution where justified | Requires workload and platform evidence beyond the present campaign | Explicitly admitted execution capabilities |

A high-priority mechanism need not be first in dependency order. F01 requires a complete current linearization association before removing probes; that validity work enables the optimization. F07 benefits from F06 but can be qualified separately. F08's conservative compatibility partition can land without a broad profile-identity redesign. F10's lane-local correction does not require a new commit service. F12 module packing can precede and remain independent of SIMD.

### Measure the complete outcome

Keep four distinct user-visible measures:

1. Cold time to the first physically accepted result.
2. Warm edit/re-solve latency under a specified valid preparation/start state.
3. Accepted-case throughput at a fixed CPU and memory envelope.
4. Time until the requested detail or durable result is actually available.

Include unsuccessful attempts, fallback work and required drain in the corresponding operational measure. Preserve a separate successful-solve breakdown so failure costs remain explainable. Never improve reported throughput by weakening the physical target, dropping an assurance requirement or silently excluding hard cases.

For each change, use its specific settling experiment above before broad qualification. The important discriminator is a mechanism observation coupled to the same scientific result:

| Mechanism | Required discriminating evidence |
|---|---|
| Sparse implicit reuse | Fewer repeated state-Jacobian probes; provider/regime changes still invalidate; derivative/linear residual checks agree |
| HVP realization | Lower complete preparation/evaluation cost for the chosen support family; correct analytic actions |
| Indexed/coarsened blocks | Parent scans and native setups fall as intended; selected branch/start semantics and original checks preserved |
| Native tolerances | Native stopping and original acceptance are jointly explained across scale/shift/magnitude changes |
| IDAS reuse/windows | Fewer constructions, legitimate overlap, bounded checkpoints and complete reset/drain |
| Mixed batch/frontier | Actual parallel entries where legal; policy inspections track real events rather than all pending work |
| Compact retention | Other lanes progress; terminal producer error is preserved; live reports and bytes remain bounded |
| Module packing | Fewer compiler children with unchanged host-stage behavior and conservative code-generation flags |
| Observation changes | Off/ordinary/detail comparison, including loss/freshness semantics and exact work accounting |
| Durable overlap | True acknowledgement, bounded pending products and correct recovery under slow/uncertain publication |

Five fresh-process repetitions support medians/ranges and an initial direction. They do not alone establish stable tail latency or a statistical significance claim. Match the shipped build/profile, solver settings, evaluator availability, hardware, total team budget and scientific requirements when making comparative claims. Existing release and dist configurations differ, so a profile change must be reported as part of the experiment. [Cargo profiles][cargo].

## 9. Formal architectural judgment and rule impacts

### Representative change scenarios

These refine the binding's PSE-S01–S07 scenarios without changing the product requirements.

| Scenario | Change and expected locality | Main findings |
|---|---|---|
| S01 Repeated property action | Change a direction at one state; mathematical action changes while qualified state matrix survives | F01, F02 |
| S02 Larger decomposable flowsheet | Grow declarations/blocks; preparation tracks selected structure, native grouping remains a numerical choice | F03, F11 |
| S03 Repeated dynamic trial | Rebind initial values/parameters; private compatible allocations survive, numerical history resets | F06, F07 |
| S04 Mixed or skewed study | Change per-case profile or selected output size; compatible work overlaps and bounded detail drains | F08–F10, C03 |
| S05 New provider or regime | Existing physical concepts extend their owner; full context invalidates dependent numerical state | F01 preservation constraint |
| S06 Library realization substitution | Replace scalar module realization or preconditioner capability at its integration owner | F12, C01/C02 |
| S07 Shifted/scaled or interrupted solve | Numerical controls retain their meanings; original checks, start provenance and complete drain remain truthful | F04, F05, F14 and execution constraints |

The model remains adequate for these distinctions: physical meaning, derivative demand, solver method, state validity, occurrence identity and result completeness already have explicit owners. Several defects arise because physical work does not yet follow those distinctions closely enough. Adding a new unit/property model should not require a new scheduler policy; changing the compiled realization should not modify physical definitions; a numerical batch should not merge durable occurrence identity.

### Foundations

Verdicts concern the inspected execution boundary, at source/interface evidence level.

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 Separation of concerns | Satisfied in scope | Physics, mathematical realization, native iteration and output owners are distinct. Proposed corrections remain at those owners. |
| AP-02 Stable contracts | Violated | F04's physical-equivalence promise differs from the native operation; F14's receipt conflates two start concepts. |
| AP-03 Composition | Violated for the examined bulk seam | Backend-only outer grouping discards inner compatibility and can turn available independence into serial fallback. |
| AP-04 Domain model and semantic authority | Satisfied in inspected paths | Explicit physical roles, original acceptance, demand and effect identities govern the traced operations; no claim about unexamined physics coverage. |
| AP-05 Explicit structure | Satisfied in scope | Prepared plans, sparse layouts, dependency/continuation ownership and typed outcomes are visible. |
| AP-06 Local reasoning/testability | Satisfied for proposed change boundaries | Mathematics, adapter controls, transport and pure study policy have explicit local inputs; those corrections do not require production storage semantics. Runtime qualification remains separate. |
| AP-07 Execution fits workload | Violated | F01/F03/F09/F11 show work amplification; F06/F08/F10/F12 expose repeated setup/crossing or inappropriate scheduling/lifetime granularity. |

### Gates

| Gate | Judgment | Scope and follow-up |
|---|---|---|
| G1 Authority | Pass, source-inspected | Keep physical model and outcome owners authoritative. |
| G2 Semantic fidelity | Fail for stated method-control contract | Correct the absolute/relative KINSOL interpretation; retain original acceptance. |
| G3 Validity | Pass for inspected existing association mechanisms | Proposed F01/F06 reuse is not qualified until complete invalidation/reset behavior is exercised. |
| G4 Hidden behavior | Pass in inspected boundary | Native/evaluation effects have explicit owners; no proposed hidden background work. |
| G5 Consistency and recovery | Pass at inspected contract/path level | Drain and publication witnesses exist; proposed decoupling needs its own failure qualification. |
| G6 Transformation and reuse | Pass for inspected baseline validity; efficiency defects remain | Do not equate excessive recomputation with an established stale-cache bug. |
| G7 Truthful capability claims | Fail in the narrow start receipt | F14 reports no start submission despite primal submission; wider capability execution remains unqualified here. |
| G8 Library use | Pass at source/interface level | Native libraries own numerical iteration and factors; target continues that division. |
| G9 Architectural fitness | Fail | AP-02/AP-03/AP-07 defects require revision independently of current passing controls. |
| PS-G1 Physical consistency | Unresolved for whole scoped workload | Unit/property/balance qualification was not independently executed or decoded. No new violation was established. |
| PS-G2 Well-posedness | Pass in inspected admission contract | Structural and numerical questions are separated; grouping must preserve the distinction. |
| PS-G3 Numerical integrity | Unresolved for campaign-wide behavior | Original checks and correct step-limit mapping were inspected. Raw failed/interrupted attempts and implementation qualification are unavailable. |

These judgments do not reject the whole simulator or establish failure of its independent physical checker. They reject a claim that the present implementation already meets the requested execution-performance target.

### Rule impacts and disposition

This review proposes changes; it does not supersede rules or schedule work. If adopted, each finding should be linked from the appropriate owning plan's disposition table with its scenario and closing evidence.

| ID | Current rule/location | Proposed change and dependent recommendation | If retained |
|---|---|---|---|
| RC01 | KINSOL scale/step equivalence in adapter documentation, read with numerical policy §16 | Distinguish native relative stagnation control from absolute physical acceptance; F04 | The incorrect promise must still be corrected; conservative native tuning can remain within current capabilities. |
| RC02 | Mathematics §14.5 excludes JIT/SIMD/GPU/distributed execution; compiled artifact identity fixes scalar realization | Admit separately qualified vector/JIT/platform realizations with explicit identity and FP behavior; C02 | Implement F12 module packing and scalar work removal; do not claim vector/device execution. |
| RC03 | Callback owner currently publishes ordinary cumulative facts at every safe return | Define bounded coalesced publication and truthful freshness/loss semantics; F13 | Preserve per-return publication, limiting changes to local allocation/update efficiency. |
| RC04 | Dynamics capability admission explicitly refuses the coupled BlockFactor route; current PETSc PC vocabulary is bounded | Add a qualified coupled preconditioner contract at the native integration owner; C01 | Compare existing KLU and None/Jacobi Krylov routes without claiming new Schur support. |
| RC05 | PETSc MPIUNI and process-wide native scheduler ownership contracts | Qualify an isolated-process or other concurrency-capable deployment only if selected | Preserve gates and account for serialized capacity; optimize other parallel regions. |

No recommendation requires weakening 0.001 physical acceptance, dropping source/domain checks, replacing the model authority, writing a bespoke nonlinear solver or changing durable acknowledgement meaning. Changes to native accuracy interpretation, reporting freshness and newly admitted capabilities need explicit design decisions in the normal plan/ADR route.

## 10. Source map

Repository links below are pinned to the inspected implementation. Source anchors are supplied for the most consequential paths; symbol names in the findings describe the exact operation. The campaign context is the source of reported campaign facts. Upstream references are primary documentation/source, with version qualifications stated in the text.

The outstanding evidence request is the actual external-pipeline-review-20261019 directory or its tar archive. With that available, the next pass can replace provisional consequence-based priorities with measured attribution, identify whether these mechanisms dominate the retained cases, and select concrete route/width/cutoff defaults. The source-established work reductions above remain actionable independently of that ranking.

[baseline]: https://github.com/paul-heyse/pse-arrow/commit/1ee81798a9b884f3054b685ee9629e2c8d124498
[context]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/docs/external-pipeline-performance-review-context.md
[architecture]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/docs/authoritative_design/sections/architecture-overview.md
[numerical-design]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/docs/authoritative_design/sections/numerical-execution.md
[assembly]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-math/src/assembly.rs
[staged]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/math/staged.rs
[study]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/study_execution.rs
[demand]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/study_demand.rs
[observation]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-diagnostics/src/observation.rs#L1067
[transport]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/transport.rs#L399
[implicit-products]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-math/src/implicit_products.rs#L619
[implicit]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-math/src/implicit.rs
[products]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-math/src/products.rs#L260
[workspace]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-compiler/src/workspace.rs#L1367
[blocks]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/math/solves/blocks.rs
[kinsol]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/kinsol.rs
[kinsol-native]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/vendor/sundials-sys/vendor/src/kinsol/kinsol.c#L2465
[kinsol-doc]: https://sundials.readthedocs.io/en/v7.1.1/kinsol/Usage/index.html
[conic]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/conic.rs
[clarabel-info]: https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/src/solver/implementations/default/info.rs
[clarabel-data]: https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/src/solver/implementations/default/problemdata.rs
[integrated]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/integrated.rs
[idas]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/dynamics/idas.rs
[shooting]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/shooting.rs
[idas-doc]: https://sundials.readthedocs.io/en/v7.1.1/idas/Usage/SIM.html
[dynamics]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/dynamics.rs
[petsc-fieldsplit]: https://web.cels.anl.gov/projects/petsc/vault/petsc-3.24/docs/manualpages/PC/PCFIELDSPLIT.html
[solve]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/solve.rs
[solves]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/math/solves.rs
[study-model]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/study.rs
[results]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/results.rs
[result-export]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/result_export.rs
[math-service]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/math.rs#L855
[math-execution]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-math/src/execution.rs
[native-compiled]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-math/src/native_compiled.rs#L540
[symbolica-compiled]: https://docs.rs/symbolica/latest/symbolica/evaluate/struct.CompiledRealEvaluator.html
[symbolica-source]: https://docs.rs/symbolica/latest/src/symbolica/evaluate/backend.rs.html
[petsc]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/petsc.rs#L66
[uno]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/uno.rs
[highs-lifecycle]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/highs_lifecycle.rs
[scip]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/scip.rs
[pounce-convex]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/execution/pounce_convex.rs
[callback]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/callback.rs#L405
[start-pipeline]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-backend-native/src/presolve/pipeline.rs
[worker]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/worker.rs
[study-operations]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/study_operations.rs
[run]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/run.rs
[durable]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/crates/pse-runtime/src/workflow/durable.rs
[cargo]: https://github.com/paul-heyse/pse-arrow/blob/1ee81798a9b884f3054b685ee9629e2c8d124498/Cargo.toml#L443
[ipopt]: https://coin-or.github.io/Ipopt/SPECIALS.html




