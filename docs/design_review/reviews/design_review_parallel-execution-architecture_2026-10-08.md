---
title: Parallel execution architecture
date: 2026-10-08
tier: design
purpose: target
standard: core-3.4
profiles: [process-simulator-1.5]
binding: pse-arrow
reviewer: independent-design-reviewer
decision: revise
disposition_owner: docs/plans/28-surrealdb-unified-substrate.md
---

# Parallel execution architecture

The architecture has useful foundations for concurrent scientific work: immutable selected admissions and compiled products, private compiler generations, independent evaluator and provider workers, typed study occurrences, original-model assessment, and resource owners retained through native teardown. These foundations should be preserved.

The complete route does not yet satisfy the reliable sixteen-worker target. Existing end-to-end controls fail with definite canonical-store transaction conflicts. Independently, some preparation operations reserve their maximum configured capacity as immediate working memory, and persistent native teams lack an explicit reservation for their additional configured stacks. Ordinary independent study frontiers also retain serial execution outside the specialized native batch. Backend-specific process constraints need an explicit execution strategy before the broad target can be accepted.

The decision is **Revise**. This does not establish that SurrealDB, Salsa, immutable artifact sharing, or the native session mechanism needs replacement. The findings identify distinct coordination and lifecycle corrections. A universal guarantee of sixteen simultaneous native solves for every backend is neither required nor supported by the inspected evidence.

## 1. Scope, target and evidence boundary

This is a **DESIGN-tier, TARGET-purpose** review of the composed parallel execution architecture. It applies Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5, and the pse-arrow binding.

The functional target is reliable execution of sixteen independent end-to-end cases on sixteen physical cores, with internal library parallelism coordinated under finite CPU and memory resources. Independent cases should run concurrently where their operations permit it. Required scientific validation, exact occurrence identity and result ordering, source protection, coherent publication, recovery, and cancellation followed by native drain remain constraints.

The inspected baseline is `fbbf718dc175dea773c468db1f14deba88f7042d` plus the current uncommitted Plan 28 implementation. Plans 28 and 28a–28h supply the current implementation context. Previous reviews were investigation leads, not acceptance criteria. The independent reviewer changed no files and ran no tests, builds, measurements, or campaigns. The coordinator publishes this review and its plan handoff; documentation checks do not qualify product execution.

The boundary includes:

- Construction and canonical source publication.
- Protected source selection, selected admission, Salsa compilation, durable descriptions, and native artifacts.
- Native preparation, evaluator/provider workers, staged initialization, solving, and original-model assessment.
- In-process studies, durable occurrence policy, worker discovery, claims, execution, recovery, and result publication.
- Progress streaming, connected results, and historical serving.
- Cargo and test orchestration, assessed separately from product execution.

Square simulation, optimization, nested property evaluation, initialization and studies are directly relevant. Dynamics and fitting are assessed through their shared preparation, worker, lifecycle and publication boundaries; their numerical algorithms and scientific fidelity are not requalified here. Adding physical models and replacing an implementation are assessed as change scenarios.

The failed control log is `/tmp/plan28-parallel-typed-owner-controls.log`. It records:

```text
just unit-native-capability-package 'pse-runtime'
'pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-relations/force-validate'
'solver,klu,isolation,uno,petsc'
'test(conformance_parallel_sixteen_)' --profile local
```

The executed Nextest selection retained explicit force-validation. Against the zero-failure target, **2 tests ran: 0 passed, 2 failed, 637 skipped**.

The first control reported ten native entries rather than sixteen, with canonical `Some(TransactionConflict)` failures among cases that did not enter. The staged initializer control reported sixteen fixtures, nine results, eleven initializations and seven failures, also including typed canonical transaction conflicts. These observations establish failure of the exercised complete route. They do not identify the precise failing SQL operation, prove a native concurrency ceiling of ten, or establish the memory findings as their cause.

The controls use a **2 GiB pool and their fixture math policy**. The broader reference execution profile uses a **128 GiB pool and 16 GiB worker capacity**. Those conditions must not be conflated.

## 2. Responsibilities and composition

The execution architecture separates several responsibilities effectively.

| Responsibility and owner | Consumed contract and hidden decision | State, effects and local reasoning |
|---|---|---|
| Authored model and physical typing: `pse-modeling`, `pse-quantity`, `pse-math` | Physical meaning, variable roles, guards, provider demands and mathematical operations | Scientific definitions govern construction; scheduling must not reinterpret them |
| Canonical source and product ownership: `pse-operations` | Exact immutable identities, revision membership, protection, publication and acknowledgment | Surreal transactions and retry/recovery behavior remain inside this integration |
| Selected preparation: runtime modeling canonical owner | Complete selected dependencies and admitted scientific revision | Shared immutable admissions; each subsequent canonical compiler workspace is fresh |
| Compiler: `CompilerWorkspace` | Structural/value dependencies and prepared products | Workspace-local Salsa database and mutex; no global compiler mutex was established |
| Math execution: `MathService` | Jobs, CPU admission, allocation owners, flights and artifacts | Shared service controls; native work and resource ownership survive caller cancellation |
| Native adapters: `BackendExecution` and adapter implementations | Problem representation, numerical capability, settings, status, retained state and native scope | Libraries own fitting iteration and factorization; adapters own their consumed ABI and lifecycle |
| Staged scientific workflow and assessment | Original specification, temporary stage bindings, starts and original-model obligations | Private mutable workers; assessment is evaluated on the step’s worker |
| Study policy: `pse-model::study` and `pse-operations::study_policy` | Occurrences, dependency edges, starts, lifecycle and scientific availability | Pure policy is shared by execution adapters; equal bindings do not merge occurrences |
| Durable worker and publication | Discovery, claims, fences, progress ingestion, terminal acknowledgment | Durable effects remain distinct from scientific results and caller completion |
| Connected result reader | Exact terminal selection, relation and recorded coverage | Bounded independently decoded blocks; escaped Arrow arrays retain allocation ownership |

The important positive distinction is **shared immutable preparation versus private mutable execution**. `CompiledBody::worker_scoped` clones evaluator programs and constructs an independent frame. `CaseAssembly::worker_scoped` creates independent workers, provider maps, caches, and matrix refills. Sharing compiled products therefore does not force a shared evaluator lock.

Likewise, runtime `canonical_workspace` wraps a clone of the selected read in a new mutex, then constructs a new `CompilerWorkspace`. The body-retention mutex and compiler mutex are local to that workspace. Their synchronous RPC crossings matter to resource occupancy, but describing them as a global compiler serialization mechanism would be incorrect.

## 3. Contracts and semantic authority

The scientific model is sufficiently explicit for the inspected parallelization boundary. Parallel execution is a realization of existing scientific operations, rather than a second mathematical authority.

The operational model distinguishes requested occurrence, native attempt, prepared product, retained result, lifecycle, scientific availability, cancellation, and unattempted work. `PointAction` carries an exact occurrence and expected revision; study conclusion keeps scientific availability separate from lifecycle. Those distinctions constrain scheduling and publication.

Resource semantics need particular care: an upper capacity, an immediate reservation, a retained allocation, an active CPU team, and a persistent native scope have different lifetimes. F02 and F04 concern their physical realization.

| Contract | Authority and consuming behavior | Assessment |
|---|---|---|
| Selected scientific meaning | Canonical revision/dependencies and Rust admission; positive, absent-name, membership and interpretation premises govern reuse | Strong foundation; protection and publication coordination are the concern in F01 |
| Prepared and compiled products | Compiler/math owners; source context and selected demand govern identity | Immutable sharing and private workers are appropriate |
| Native operation | Adapter capability, admitted settings, original contract and execution scope | Internal thread capability is explicit; cross-case process coexistence needs the strategy in F03 |
| Study occurrence | Study definition and pure occurrence policy | Equal case bindings preserve distinct occurrence and attempt identities |
| Result permission | Original-model assessment and typed completion | Solver status alone does not establish a scientifically usable solution |
| Cancellation and drain | Cancellation owner plus native supervisor/session close | CPU, job and allocation owners are retained through native completion and join |
| Durable visibility | Claims/fences, bounded ingestion, exact terminal acknowledgment | Stream completion, result ingestion and terminal publication have distinct lifetimes |
| Historical serving | Protected terminal selection and recorded coverage | Missing ranges remain missing; bounded retrieval does not fabricate complete results |

### Physical semantics

This review changes no quantity definition or numerical convention.

| Quantity/model element | Dimension and unit | Basis/reference/convention | Validity | Authority |
|---|---|---|---|---|
| Authored scalar, parameter and variable | Registry-admitted quantity and canonical unit | Declared physical type includes consequential basis/reference distinctions | Construction and model constraints | Quantity/modeling owners; blueprint §7.3 |
| Equation, contribution and observation | Admitted mathematical/physical output type | Original model identity and coordinate mapping | Authored guards and applicability | Math/modeling owners; blueprint §7.3–§7.5 |
| Provider input/output | Declared provider port quantities and units | Provider declaration and configuration | Envelope, smoothness and derivative demand | Provider registration/admission |
| Candidate and assessment | Original physical quantities and model identities | Original-coordinate interpretation | Current residuals, bounds, domains and required closure | Runtime modeling assessment/results |
| Stored result or trajectory | Registry schema and recorded identities | Persisted interpretation and coverage | Recorded completion and scientific availability | Canonical result publication and connected reader |

**Well-posedness.** Original structural admission remains distinct from numerical solving. `execution/runner.rs` invokes structural checking and validates retained assessments against the original inventory and equality/bounds contract. Source topology is not treated as a substitute solve order. The reviewed concurrency mechanisms must retain these checks and their model attribution. This is Interface-checked preservation of the route, not a new qualification of every model.

## 4. Representative scenarios

| ID | Stimulus and kind | Expected behavior and boundary | Current assessment |
|---|---|---|---|
| **S01** | Sixteen independent authored cases; instance/concurrency change | Complete preparation, execution, original checks and results; stable discovery-order report | Exercised controls fail; F01. No native ceiling inferred |
| **S02** | Small cases invoke nested-provider preparation or value rebind under a generous per-case capacity; growth/policy change | Admit actual needs or pace temporary work; preserve legitimate oversized-work refusal | Maximum-capacity reservation can refuse ordinary concurrent work; F02 |
| **S03** | Retained sessions coexist with transient preparation and original validation; composition | Job, memory and CPU lifetimes compose without relying on accidental spare slots | Thirty-two jobs supply known headroom for the selected sixteen-session route; broader stage composition requires explicit policy |
| **S04** | Direct HiGHS, Uno, PETSc, Ipopt or a multithreaded backend coexist; mechanism/binding | Select a documented thread/process strategy; backend exclusion and waiting do not masquerade as active CPU work | Native constraints are real; complete strategy remains unresolved; F03 |
| **S05** | Staged session switches to a multithreaded POUNCE scope; mechanism/lifecycle | Admit all known team stacks before creation and retain their charge until scope teardown | One-stack session reservation misses the additional persistent team extent; F04 |
| **S06** | Cancel, hit a finite cap, fail a fixture or interrupt publication; failure journey | Stop issuance where appropriate, drain issued work, preserve partiality and exact acknowledgment semantics | Important mechanisms are implemented; full concurrent controls are not newly established |
| **S07** | Study contains repeated equal bindings and continuation edges; composition | Preserve every occurrence, dependency and chosen start; reduce only in the declared order | Explicit occurrence policy preserves identities and starts, but the ordinary ephemeral frontier uses one staged session and serial member execution outside the specialized native batch. Durable concurrency depends on the selected managed-worker deployment; F05 |
| **S08** | Add a unit/provider or substitute a compatible implementation; domain extension/mechanism | Scientific owners absorb new meaning; adapter owns capability differences; shared checks and workflows are reused | Existing seams are suitable. No new registry or general scheduler framework is required |
| **S09** | Serve historical results during retirement or interruption; representation/failure | Exact protected terminal selection, recorded missing coverage, bounded decoding and retained escaped buffers | Connected reader is a strength; no current contrary result established |
| **S10** | Run sixteen ordinary test processes or one test containing sixteen runtime workers; tooling | Outer concurrency remains distinct from product concurrency and inner library teams | Nextest’s exclusive sixteen-slot parallel controls avoid a sixteen-by-sixteen multiplication |

## 5. Complete-operation execution

### Preparation and reuse

`checked_selection` protects the revision, rechecks candidate dependencies, otherwise resolves selected inputs and uses shared selected-admission flights. A fresh workspace subsequently prepares the demanded product. This avoids confusing shared immutable admission with a mutable cross-case compiler.

Canonical body retention can reuse admitted meaning, but a memory hit still calls publication to establish durable reachability for the current revision. Durable lookup/publication uses `Handle::block_on` while a preparation job is active. The selected-read mutex is private; the canonical guards can be shared across cases. Consequently, waiting for store completion can occupy a job and CPU permit even where the mathematical computation has finished. This contributes to the coordination concerns but does not itself prove globally serialized compilation.

Artifact flights are appropriate for equivalent construction. Artifact jobs admit optimizer cores and construction scratch; sharing a completed artifact does not merge case occurrence or result identity.

### Native lifetime

A native session reserves a job slot, stack, foreign allowance and inner-session capacity until its thread joins. Worker storage draws from the shared pool as it is created. Each request separately acquires CPU permits, and the request retains them through its work.

Cancellation of a request signals native work and awaits its reply. Closing a session drops its sender and awaits the join witness. Detached supervision preserves resource ownership if the caller stops waiting. These choices match the fact that started blocking/native work cannot generally be aborted by dropping an async handle. Tokio documents that limitation for [`spawn_blocking`](https://docs.rs/tokio/1.53.2/tokio/task/fn.spawn_blocking.html).

The persistent adapter scope is a different lifetime from the CPU request. `serve` can remain inside an adapter scope while waiting for the next request. F04 concerns the known team state retained by that scope.

### Scientific stages

| Stage | Formulation and derivative contract | Scaling/class/status/assessment |
|---|---|---|
| Construction and specialization | Physical admission, authored guards, selected dependencies and provider demand remain authoritative | Structural/value products remain distinct |
| Evaluator and provider preparation | Symbolic/provider derivative source and demanded order govern compiled products | Immutable product; private numeric/provider workers |
| Initialization and staged execution | Temporary scientific bindings and declared strategy are separate from the original specification | Retained native state is compatible only within its admitted scope |
| Native solve | Adapter consumes admitted class, representation, derivative capability and settings | Backend maps native status to typed report; no universal cross-backend concurrency assumption |
| Original-model assessment | Current candidate evaluated against original rows, domains and required closure | Current tolerances and independent acceptance remain required |
| Durable publication and serving | Publishes actual completion, recorded coverage and scientific availability | Lifecycle completion alone is not a scientific solution |

### Studies, publication and results

Conformance uses bounded `FuturesUnordered`, continuously reuses active slots, compacts completed reports and merges them by discovery prefix. It preserves unattempted inventory after a cap or cancellation. Fatal work stops further issuance and continues polling issued futures before return. This is preferable to confusing deterministic report order with an execution barrier.

Conformance’s bounded concurrent futures do not characterize ordinary study execution. The ephemeral study driver prepares frontier cases sequentially and routes eligible fresh cases through one staged session. Generic batch execution remains sequential unless the specialized homogeneous coefficient/cone batch applies. F05 requires independent frontier execution while preserving genuine continuation dependencies.

A durable worker handle processes one action at a time. Multiple managed workers/processes supply a separate concurrency mechanism. An in-process durable study loop calling `work_once` does not by itself supply sixteen simultaneous workers.

Progress is bounded. The native producer clones the sender under its lock, releases that lock, and waits only within the configured bound when the queue is full. Failure marks overflow and requests cancellation. Durable finish awaits the streamer, ingests result tables and seeds, and terminates while heartbeat ownership remains live. These are useful backpressure and settlement mechanisms. The producer may still hold its native CPU permit while waiting; the complete deployment strategy must account for that occupancy.

Connected results stream independently decoded blocks under an exact protected terminal selection. Coverage coordinates and missing ranges remain explicit. No finding requires replacing this route with full-result materialization.

## 6. Foundations and gates

### Architectural foundations

| Foundation | Verdict | Reason |
|---|---|---|
| **AP-01 Separation of concerns** | **Satisfied** in the inspected scope | Scientific meaning, canonical effects, compiler preparation, native integration, study policy and results have identifiable owners. The remedies should retain these boundaries |
| **AP-02 Stable contracts** | **Unresolved** | The native contract describes internal thread capability, but the complete cross-case thread/process strategy and exclusion behavior needed by S04 remain unsettled; F03 |
| **AP-03 Composition** | **Violated** | Ordinary independent study cases are composed through the single-sequence execution owner, retaining serial member execution even where no scientific dependency requires it; F05. Native/deployment composition under F03 also remains unresolved |
| **AP-04 Domain model and semantic authority** | **Satisfied** in the inspected scope | Scientific definitions govern construction and assessment; occurrence, attempt, scientific availability and lifecycle remain distinct. No second scientific authority was established |
| **AP-05 Explicit structure and constraints** | **Violated** | Additional persistent POUNCE team stacks have no explicit matching session-scope reservation; F04. Backend coexistence also needs the F03 strategy |
| **AP-06 Local reasoning and testability** | **Satisfied** in the inspected scope | Private compiler/worker state and shared pure study policy support local reasoning. Necessary store/native integration tests remain necessary |
| **AP-07 Execution fits workload** | **Violated** | Broad guard coupling and demonstrated complete-route conflicts obstruct S01; maximum-capacity reservations amplify concurrent live admission in S02; persistent team extent is incompletely admitted in S05. Ordinary independent study frontiers retain avoidable serial preparation/execution outside the specialized native batch; F05 |

G9 follows these individual judgments without averaging.

### Core gates

| Gate | Judgment | Evidence and limitation |
|---|---|---|
| **G1 Authority** | **Pass, scoped** | Authored scientific meaning and pure study policy remain identifiable authorities; caches and products are derived |
| **G2 Semantic fidelity** | **Pass, scoped** | Occurrence/attempt identity, original coordinates, ordering and missing coverage are explicit. No silent merging or reinterpretation was established |
| **G3 Validity** | **Unresolved for expanded parallel scope** | Scientific admission boundaries are present; full sixteen-case operation and the corrected persistent-team envelope remain unestablished |
| **G4 Hidden behavior** | **Pass, scoped** | Canonical retention is an explicitly attached effect boundary; transient refusals unwind tracked evaluation rather than becoming scientific cached results |
| **G5 Consistency and recovery** | **Fail for resource coordination; otherwise scoped strengths** | F04 leaves a known persistent native-team resource outside matching explicit admission. No observed partial output was shown to masquerade as a committed solution |
| **G6 Transformation and reuse** | **Unresolved for the complete target** | Immutable reuse/private worker mechanisms are inspected, but selected concurrent original-result/order controls fail before establishing the required complete comparison |
| **G7 Truthful capability claims** | **Unresolved for sixteen-worker acceptance** | The target remains unqualified. Native process constraints require explicit supported strategies; pending plan status should remain pending |
| **G8 Library leverage** | **Pass, scoped** | Established libraries own mathematics, solvers, factors, async polling, memory-pool mechanisms and native teams. No fitting generic solver machinery was shown to need bespoke replacement |
| **G9 Architectural fitness** | **Fail** | AP-03, AP-05 and AP-07 are violated; AP-02 has material unresolved premises |

### Process-simulator gates

| Gate | Judgment | Evidence and limitation |
|---|---|---|
| **PS-G1 Physical consistency** | **Pass for inspected mechanism preservation** | Physical typing, provider contracts and original closure checks remain on the route. This is not fresh numerical qualification of all models |
| **PS-G2 Well-posedness** | **Pass for inspected mechanism preservation** | Original structural admission and retained-inventory validation remain before solving; no topology/solve-order substitution was established |
| **PS-G3 Numerical integrity** | **Unresolved for full concurrent qualification** | Typed outcomes and original-model assessment are present. The failed controls do not establish complete concurrent scientific equivalence, nor do they demonstrate a misclassified native success |

## 7. Findings

### <a id="f01"></a>F01 — Independent scientific work shares a broad retention conflict domain

**Principles:** AP-07, DP-20; H21–H24.  
**Scenarios:** S01, S03, S06.  
**Evidence:** Implemented structure; Tested failure under the named existing control.

`PROTECTED_BEGIN` reads `retention:$problem FOR UPDATE`. Protection/release, product admission and staging operations update that retention guard. Therefore, selected reads within one problem participate in commit-time conflict detection against otherwise scientifically independent protection and publication activity.

The implementation evidence is at the [canonical protected/retry/publication owner](../../../crates/pse-operations/src/canonical.rs), [protection and retirement owner](../../../crates/pse-operations/src/canonical_retention.rs), [staging transactions](../../../crates/pse-operations/src/canonical_staging.rs), [runtime selected preparation](../../../crates/pse-runtime/src/workflow/modeling/canonical.rs), and [canonical body retention](../../../crates/pse-runtime/src/math/retention.rs).

SurrealDB’s `FOR UPDATE` registers records for commit-time conflict detection; it is not a PostgreSQL-style reader mutex. Whole-transaction retry follows a conflict. Its documentation recommends short transactions touching the fewest necessary keys. [SurrealDB SELECT documentation](https://surrealdb.com/docs/reference/query-language/statements/select), [transaction documentation](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions)

The selected SurrealDB 3.3.0 gRPC source corroborates concurrent multiplexed request dispatch; its ignored capacity argument does not establish a request-concurrency limit. The examined [pinned revision](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/src/engine/remote/grpc.rs) is `238bfeb11f5725bebed370167656748df8067595`. Optimistic conflict validation and broad application guard coupling must therefore be distinguished from a claim that the transport or whole database serializes all requests.

The existing complete controls show definite conflicts escaping to case preparation or start-to-solve failures. The precise failing SQL operation is not recorded in that log. Accordingly, the broad guard is a demonstrated structural contention mechanism and a strong corrective lead, not a proven attribution of every observed refusal.

Current bounded retries are necessary protection. Increasing attempts and adding jitter does not establish that the assembled route is reliable or that its coordination work is proportionate.

**Correction direction.** The canonical integration owner should provide a reliable complete-operation route that reduces unnecessary shared guard activity while preserving reclamation and publication safety. Candidates include sharing an operation’s immutable protection where its lifetime and completeness permit it, coalescing equivalent reachability publication, or narrowing conflict domains to the protection/root/reclamation effects actually coupled. A short guarded-decision coordinator is also eligible if it simplifies this local deployment without serializing scientific computation.

Do not remove the retention premise without replacing its protection against reclamation and concurrent change. Named absence/set premises still require appropriate conflict coverage. Distinct problems must retain independent domains except where a shared immutable-version lifecycle genuinely requires coordination.

**Verification.** Identify the failing operation class if needed to choose the correction. Show sixteen complete cases, exact report order and original checks, concurrent protection/publication/retirement safety, and acknowledgment behavior after uncertain delivery. A reader-only control or higher retry count alone does not close F01.

### <a id="f02"></a>F02 — Some preparation routes charge the maximum configured capacity as immediate demand

**Principles:** AP-07, DP-20, PS-11; H11, H17, H21.  
**Scenarios:** S02, S03.  
**Evidence:** Implemented source structure; no measured capacity or performance claim.

`modeling_inner_providers`, term diagnostics, and some rebind routes submit `self.policy.worker_bytes` to `job_retained`. The job immediately grows a shared-pool reservation by that supplied extent plus its known overhead. Thus a generous per-job maximum becomes an immediate full reservation even when the current operation’s actual required state is much smaller.

The implementation evidence is at [modeling preparation/rebind submissions](../../../crates/pse-runtime/src/math/modeling.rs), [immediate job reservation and admission](../../../crates/pse-runtime/src/math/jobs.rs), [policy and workspace ownership](../../../crates/pse-runtime/src/math.rs), and [retained session admission](../../../crates/pse-runtime/src/math/staged.rs).

A conservative upper-bound reservation can be justified for opaque preparation. The defect is its composition with immediate refusal and the finite supported workload: ordinary small operations can be refused because maximum allowances overlap. The remedy may retain conservative reservations while safely pacing temporary stages. It need not introduce a generic cost estimator, duplicate the memory-pool authority, or admit sixteen maximum-sized allocations simultaneously.

In the broader target profile, sixteen operations each reserving a 16 GiB maximum cannot simultaneously fit in a 128 GiB pool, before other retained state and overhead. This is reservation arithmetic, not an assertion that sixteen maximum-sized cases should fit, and not the cause established by the 2 GiB control log.

Native session job slots and transient preparation jobs are also distinct. The selected staged control’s thirty-two job slots provide explicit headroom for sixteen retained sessions plus their preparation/validation work. Simply equating fixture width, CPU permits and job slots would discard that distinction.

**Correction direction.** The math execution owner should distinguish capacity from current reservation and pace temporary work so ordinary legitimate cases complete. Use known operation extents where available, incremental drawing where the library permits safe admission before growth, or a bounded preparation width/queue where conservative native allowance is necessary. Retained scientific state and foreign allowances remain charged at their actual ownership lifetimes.

No requirement follows to admit sixteen genuinely oversized jobs. A case that cannot fit individually must still refuse honestly. Queued preparation must not wait behind persistent owners whose own progress requires that same capacity, and must retain the original cancellation/deadline semantics.

**Verification.** Exercise small concurrent nested-provider/rebind cases under a generous cap; distinguish their actual demand from the cap. Show completion without increasing the deployment budget or weakening scientific checks. Also retain oversized refusal, bounded waiting, cancellation and retained-owner pressure controls. Moving the same full reservation to another field does not close F02.

### <a id="f03"></a>F03 — The complete parallel strategy does not yet resolve native process coexistence and exclusion

**Principles:** AP-02, AP-03, AP-05, AP-07, DP-15, DP-20, PS-09.  
**Scenario:** S04.  
**Evidence:** Interface-checked/Implemented native constraints; complete scheduling and liveness remain unresolved.

The adapter `parallel` capability describes an adapter consuming more than one admitted native thread. It does not describe every backend’s cross-case coexistence constraints.

Inspected mechanisms differ materially:

- PETSc retains a process admission mutex for its native operation.
- Direct HiGHS retains a shared scheduler lifecycle read guard with its native model.
- Uno requires an exclusive HiGHS scheduler scope and checks cancellation while waiting.
- The same-session HiGHS-to-Uno adapter explicitly clears retained state before taking that exclusive scope. A same-session deadlock is therefore **not established**.
- Other sessions’ retained HiGHS models can postpone Uno’s exclusive acquisition.
- Ipopt’s sequential MUMPS integration has guarded factorization/solve sections. This does not serialize every Ipopt callback or the entire scientific case.
- POUNCE can use an admitted local Rayon team.

General native CPU admission occurs before these adapter-specific waits. Blocking on an exclusive process resource can therefore occupy a CPU permit while another native operation is the one actually computing. A cross-session deadlock was not proven. The structural waiting and coexistence differences are sufficient to require an explicit strategy; they are insufficient to claim all sixteen cases can execute every backend simultaneously within one process.

**Correction direction.** The runtime/native integration owners should select and describe supported thread/process execution strategies for the required case classes. Put process exclusion and team sizing into that selection/admission responsibility rather than making scientific consumers discover it through backend internals. Eligibility and finite deadlines must remain explicit.

For thread-safe independent instances, prefer shared immutable products and private workers. For process-constrained backends, serialized native phases or bounded isolated processes may be appropriate. Process isolation must share a deployment envelope; creating a fresh full CPU/memory budget per process is not coordination.

This requires no universal backend-neutral scheduler trait. Existing typed settings, adapter admission functions and deployment policy may be sufficient.

**Verification.** Establish the supported backend/profile combinations and their coexistence behavior, including mixed retained sessions, cancellation while awaiting exclusion, and teardown. Verify the selected sixteen-case route. Do not infer universal concurrency from `parallel: true`, backend names, or outer worker count.

### <a id="f04"></a>F04 — Persistent POUNCE teams lack an explicit matching stack reservation

**Principles:** AP-05, AP-07, DP-19, DP-20, PS-11; H17, H21, H23.  
**Scenario:** S05.  
**Evidence:** Implemented source structure; no process-RSS or touched-stack measurement.

`NativeSession::session_on` reserves:

```text
one stack + foreign allowance + inner-session capacity
```

`serve` enters adapter scopes around the retained session loop, including idle `recv`. POUNCE’s scope calls `with_threads`; for more than one thread, it constructs a scoped Rayon pool with `num_threads(threads)` and the configured `stack_size(stack)`. That team can survive between requests.

The session request owns CPU permits but has no matching explicit reservation for those additional configured team stacks. In contrast, the one-shot job path explicitly charges a coordinator plus the admitted team’s stacks.

The fixed foreign allowance is separately documented from explicit native stacks. It does not establish admission of a thread-count-dependent, project-created team extent. This finding does not demand that the cooperative pool become a general allocator or claim it bounds process RSS.

The implementation evidence is at [session reservation and the persistent scope loop](../../../crates/pse-runtime/src/math/staged.rs), [POUNCE scoped team creation](../../../crates/pse-backend-native/src/pounce.rs), [POUNCE adapter scope](../../../crates/pse-backend-native/src/execution/pounce.rs), and [one-shot team-stack accounting](../../../crates/pse-runtime/src/math/jobs.rs).

**Correction direction.** The session/native scope owner should reserve the known additional team extent before team creation and retain that reservation through scope teardown, including idle retention. Alternatively, end the team scope between steps if the library’s retained-state contract permits it. A CPU permit released at step completion cannot be the allocation owner of a team that remains alive.

Scope changes, failed entry, panic, cancellation and session close must release exactly the matching team owner after destruction. Serial sessions should retain their simpler one-stack path.

**Verification.** Show pool reservation changes with actual admitted team size, remains while an idle team exists, and returns after scope teardown/join. Show refusal before creation when the known team extent cannot fit. No RSS benchmark is required to establish the accounting correction.

### <a id="f05"></a>F05 — Ordinary study orchestration serializes independent frontier cases

**Principles:** AP-03, AP-07, DP-20, PS-11; H21, H22.  
**Scenarios:** S07, S03.  
**Evidence:** Implemented source structure; no new execution or throughput measurement.

The ephemeral study route sends independent cases through one staged sequence and one retained native session:

- `study_inner` opens one `Staged`, prepares frontier members through awaited sequential calls, and calls `staged.batch` for eligible fresh declared cases. [Study execution](../../../crates/pse-runtime/src/workflow/study_execution.rs)
- Workflow `Staged::batch` prepares assessment and result ownership sequentially, then submits the members to that session. [Workflow staging](../../../crates/pse-runtime/src/workflow/staged.rs)
- `NativeSession::batch` awaits each `self.step` sequentially when the composition is not eligible for the direct batch route. Its direct route executes one scoped session request. [Math staging](../../../crates/pse-runtime/src/math/staged.rs)
- Inside that request, `MathService::execute_batch` uses a native batch only for homogeneous eligible coefficient/cone representations whose adapter declares `batch`. Otherwise it calls `run_step` sequentially for admitted members. [Solve execution](../../../crates/pse-runtime/src/math/solves.rs)
- The default backend batch implementation is sequential. POUNCE-convex supplies the inspected specialized parallel override. Ordinary Ipopt, KINSOL and direct HiGHS study cases do not gain independent-case concurrency merely by entering this method. [Backend contract](../../../crates/pse-backend-native/src/execution.rs), [POUNCE-convex adapter](../../../crates/pse-backend-native/src/execution/pounce_convex.rs)

A batch-shaped interface therefore does not establish parallel execution of ordinary independent study cases. Replacing a process-constrained native implementation with a suitable independent-instance implementation would not remove this orchestration serialization.

The durable route has a related boundary: `study_durable` invokes `work_once` sequentially, and each worker handle processes one action at a time. External managed workers provide separate concurrency, but the selected two-worker deployment does not establish the sixteen-worker study target. The local caller can process an additional action; neither fact supplies the required deployment strategy by itself.

**Consequence.** A study frontier containing sixteen independent fresh cases can retain serial preparation and serial native execution despite available case workers. This is a distinct orchestration defect, rather than another instance of F03’s native process exclusion.

Continuation edges and explicit seed dependencies can legitimately require ordering. The specialized native batch is also a valid alternative when it supplies the complete consumed contract. Neither exception justifies serializing every ordinary independent frontier.

**Correction direction.** The study execution owner should compose bounded independent-case execution from existing preparation, session, assessment and publication primitives. Use independent workers/sessions for independent sequences, or a fitting native bulk operation when its capability, thread extent and member semantics permit it. Retain serial reuse within genuinely dependent sequences.

Preserve every occurrence and attempt identity, explicit start provenance, dependency readiness, deterministic result placement, failed-point isolation, original-model assessment, finite admission, and cancellation followed by drain. Completion order must not redefine occurrence order. Preparation may be paced independently when F02’s resource constraints require it; sixteen workers do not require sixteen temporary maximum reservations.

For durable execution, select and account for the required managed-worker deployment instead of treating repeated local `work_once` calls as a parallel scheduler. Coordinate that deployment with F03’s native strategy and the existing memory envelope.

**Verification.** Exercise one ordinary study containing sixteen independent fresh non-batching cases through the selected public execution route. Establish actual concurrent case execution, complete occurrence inventory, stable result placement and original scientific checks. Retain continuation-order, repeated-equal-binding, failed-point isolation, finite-memory and cancel/drain cases. A specialized POUNCE-convex batch or sixteen independent conformance fixtures does not by itself qualify the ordinary study route.

### Relationships between the findings

F01 concerns store conflict scope; F02 concerns admission demand and pacing; F04 concerns a persistent native allocation lifetime. Their closure obligations differ. Increasing budgets or worker counts would not resolve all three.

F03 governs how the target composes native capabilities. Its solution must accommodate F02’s temporary versus retained ownership and F04’s persistent teams. Removing store contention may expose native admission problems previously hidden by preparation failures; that would not invalidate F01.

F05 is independent of F03: backend coexistence can be suitable while the study consumer still dispatches cases sequentially. Its correction consumes F02’s admission behavior and F04’s native-scope ownership. Preserve the specialized native batch rather than replacing every bulk operation with individual tasks.

## 8. Library fit and alternatives

| Capability | Current fit and limits | Recommendation |
|---|---|---|
| **SurrealDB 3.3.0** | Suitable canonical transaction/persistence boundary. Named-record optimistic protection requires explicit conflict coverage and application recovery. gRPC sharing does not prove serialized requests or universal store serialization | Retain as a viable substrate; resolve F01 within the canonical owner before considering replacement |
| **Salsa 0.28.4** | Private compiler generations and tracked inputs support local incremental preparation. Shared immutable admission/products supply reuse outside those generations | Retain; do not replace private workspace mutexes to solve a store contention problem |
| **Symbolica 3.0.1** | Immutable compiled products plus private evaluator stacks fit independent cases. Optimizer teams differ from numerical evaluation workers | Coordinate construction cores separately from solve teams; do not force a shared evaluator lock |
| **faer and sparse factors** | Caller-owned matrices/factors/scratch fit private workers and valid reuse. Explicit sequential paths are appropriate under outer case concurrency | Preserve exact factor refresh and derivative checks; verify effective features/global parallelism where it affects a selected route |
| **Native solvers** | Libraries own fitting numerical iteration, factors and model management. PETSc, HiGHS/Uno and MUMPS impose distinct coexistence constraints | Preserve adapters; settle F03 rather than reimplementing solver machinery |
| **SUNDIALS 7.1.1 through `sundials-sys=0.6.2`: KINSOL and enabled IDAS** | The pinned wrapper vendors 7.1.1. KINSOL constructs a separate `SUNContext` and serial vectors per session. IDAS owns its context, integrator memory, vectors, matrices, linear solvers and callback state, destroying them in dependency order. [KINSOL owner](../../../crates/pse-backend-native/src/kinsol.rs), [IDAS owner](../../../crates/pse-backend-native/src/dynamics/idas.rs), [feature selection](../../../crates/pse-backend-native/Cargo.toml) | **Implemented / Interface-checked:** private instances support independent cases; serial vectors differ from outer case concurrency. Sixteen complete scientific journeys are not established. The existing KINSOL parallel control failed on the composed preparation route |
| **Diffsol 0.16.2, `diffsol-la` 0.1.1 and faer 0.24.4** | [Pins/features](../../../Cargo.toml). The integrator owns instance-local oracle, parameter, mode, seed and failure state; `FaerContext` selects `Par::Seq`. [Integrator](../../../crates/pse-backend-native/src/dynamics/integrator.rs). However, custom `FaerLu` captures `faer::get_global_parallelism()` during construction/reset. [Linear integration](../../../crates/pse-backend-native/src/dynamics/linear.rs) | **Implemented / Interface-checked:** private state and sequential operator context fit outer concurrency. That context does not establish sequential factorization everywhere. Effective faer features/global parallelism remain a narrow F03 premise; no feature-resolution or concurrency probe was executed |
| **Clarabel 0.11.1: QDLDL and feature-selected MKL Pardiso** | QDLDL admits one thread and sets `max_threads=1`. MKL Pardiso sets `max_threads=0`, leaving the count to the owning worker’s scoped oneMKL-local setting from admitted controls. [Conic admission/settings](../../../crates/pse-backend-native/src/conic.rs), [Clarabel adapter](../../../crates/pse-backend-native/src/execution/clarabel.rs) | **Implemented / Interface-checked:** private instances differ from a factorization team. The MKL route requires its profile enabled and linked. Settings inspection establishes neither sixteen-instance conformance nor throughput |
| **SCIP 10.0.2 through `scip-sys=0.1.28`, declared `THREADSAFE=ON`, `TPI=tny` build** | [Build recipe](../../../docker/solvers/build.sh), [pins](../../../Cargo.toml). The adapter reserves `lp/threads=1`; for controls requesting more threads, it sets the deterministic concurrent portfolio’s minimum/maximum counts and disables central presolving before that portfolio. [SCIP integration](../../../crates/pse-backend-native/src/scip.rs) | **Implemented / Interface-checked:** internal portfolio and independent outer instances are separate axes. Build configuration alone establishes neither actual loaded-build identity nor complete independent-instance behavior. Keep sixteen-case claims scoped to execution evidence |
| **Tokio** | Async supervision, owned permits and join handling fit the effect boundary. Started blocking work cannot be forcibly cancelled | Preserve native drain; obtain relevant resource admission before work and retain it through completion |
| **FuturesUnordered** | Fits bounded independent future polling and completion-order collection. It does not itself create CPU workers | Retain conformance orchestration and deterministic merge |
| **Rayon** | Local pools fit fitting native team scopes. They do not turn foreign lock waiting into async admission | Retain where consumed; account team lifetime under F04 |
| **DataFusion memory pools** | Supply finite cooperative reservation/refusal and shared accounting; they are not a wait queue or general native allocator | Retain; fix current demand/pacing in the math owner rather than treating pool refusal as workload feasibility |
| **Task tracking utilities** | Can simplify bookkeeping if existing supervisors need it, but do not supply CPU/memory brokerage or forced native stop | Optional; no adoption is required to close these findings |

These rows identify the consumed configurations and private-state boundaries. They do not establish uniform cross-case concurrency across libraries. Serial numerical components can coexist with concurrent cases; internally parallel components consume admitted teams. Process-global constraints and actual loaded configuration remain part of the selected execution strategy.

Official library contracts supporting these distinctions include [Rayon thread pools](https://docs.rs/rayon/1.12.0/rayon/struct.ThreadPool.html), [FuturesUnordered](https://docs.rs/futures/0.3.32/futures/stream/struct.FuturesUnordered.html), [PETSc CPU/OpenMP guidance](https://petsc.org/release/manual/getting_started/#cpu-openmp-parallelism), [HiGHS C API lifecycle guidance](https://ergo-code.github.io/HiGHS/dev/interfaces/c_api/), and the [Ipopt 3.14.20 MUMPS integration](https://github.com/coin-or/Ipopt/blob/releases/3.14.20/src/Algorithm/LinearSolvers/IpMumpsSolverInterface.cpp).

Capability-skill versions were compared with the current lockfile. In particular, the math skill’s Symbolica 3.0.0 baseline is not interchangeable with current 3.0.1 evidence. Exact current-source evidence was used for the relevant evaluator/optimizer distinctions.

## 9. Alternative complete realizations

**Current realization.** Bounded concurrent case futures, immutable sharing, private native sessions, optimistic guarded canonical operations, and distinct durable worker processes provide the right major pieces. Complete-operation reliability and resource composition are not yet established.

**Simplest viable correction.** Keep those pieces. Reduce/coalesce unnecessary canonical guard activity; pace conservative temporary preparation; charge actual known retained native-team extent; select backend-appropriate thread/process strategies. This is the preferred direction because it preserves scientific owners and removes demonstrated obligations without introducing another execution authority.

**Existing Ipopt alternatives.** The declared Ipopt 3.14.20 build includes sequential MUMPS 5.9.1, SPRAL 2025.09.18 with OpenMP, and oneMKL 2026.1.0 Pardiso. These are existing typed selections. [Build recipe](../../../docker/solvers/build.sh), [typed linear settings](../../../crates/pse-backend-native/src/settings/ipopt.rs). MUMPS admits one thread and retains the previously described protected factorization/solve sections. SPRAL and PardisoMkl can consume admitted internal threads; current admission checks their linked availability and selected runtime prerequisites, including SPRAL’s OpenMP cancellation/binding requirements and Pardiso’s pinned CBWR branch with dynamic threading disabled. [Runtime admission](../../../crates/pse-backend-native/src/ipopt/settings.rs), [scoped native thread controls](../../../crates/pse-backend-native/src/mkl.rs). They are eligible alternatives when choosing F03’s case-versus-factorization strategy. Source inspection does not establish better speed, complete independent-instance safety or sixteen-case scientific equivalence. Selecting a different factorization can affect numerical behavior and must retain original acceptance checks. No default change is proposed by this review, and changing the factorization would not resolve F05’s serial study dispatch.

**Library-owned executor alternative.** A Rayon or Tokio blocking executor can host suitable bounded CPU work, but cannot eliminate canonical contention, repair capacity-as-demand reservations, or make process-constrained native libraries coexist. It must preserve worker-local native state, explicit teardown and drain. Adoption earns its place only if it removes existing supervision machinery while honoring those contracts.

**Bounded process alternative.** Processes are credible for native integrations requiring isolation. They impose artifact/source reconstruction, IPC, supervision and separate retained state. Their benefit is isolation of process-global library constraints; they do not automatically reduce work or supply a shared CPU envelope. Select them per supported capability rather than for every case.

**Store replacement.** Another transactional store could change conflict behavior, but a backend replacement would still need exact codecs, named/set premises, retained-history protection, idempotent publication, acknowledgment and recovery. The current evidence does not establish that this larger change is needed.

The recommendation changes if the canonical owner cannot preserve safety with credible contention under the required workload, or a native library cannot supply the required independent-instance contract in any practical supported strategy. Those are observable reopen conditions, not current conclusions.

## 10. Verification and uncertainty

| Claim/risk | Evidence | Result or limit |
|---|---|---|
| Sixteen complete cases and original result/order equivalence | **Tested**, existing named control log, local native capability profile with force-validation | **Failed: 0/2 passed, 2 failed**. No new run |
| Precise operation causing each transaction conflict | **Unresolved** | Log preserves typed conflicts but not exact SQL attribution |
| Private compiler/evaluator/provider state | **Implemented / Interface-checked**, inspected constructors and worker paths | Supports independent instances; does not qualify the complete route |
| Broad retention conflict domain | **Implemented**, inspected protected/read/write guards | Structural mechanism established; each logged failure’s causal attribution remains open |
| Maximum-capacity reservation amplification | **Implemented**, inspected job and modeling routes | Static feasibility concern under broad profile; not a measured memory failure |
| Persistent team stack admission gap | **Implemented**, inspected session/scoped-team lifecycle | Known extent/lifetime mismatch established; no RSS claim |
| Ordinary independent study concurrency | **Implemented**, inspected study/staged/solve dispatch | Serial generic route established; specialized native-batch exception identified. No new concurrent study test or measurement |
| Mixed backend liveness | **Interface-checked**, process/scheduler guards | Restrictions established; deadlock not established |
| Cancellation/native teardown and durable settlement | **Implemented**, inspected supervisor, session and durable finish paths | Mechanism strengths; new concurrent execution evidence not obtained |
| Historical result exactness | **Implemented / Interface-checked**, connected reader | No contradictory result established; broad historical qualification not rerun |
| Throughput or capacity improvement from proposed remedies | **Proposed** | No measurements |
| Whole scientific model conformance and parity | **Not run** | No unit/property model requalification or reference comparison performed |

The relevant conformance suite already contains shared checks and parallel controls. The two selected controls are failed evidence, not positive conformance. No unit or property definition is changed by this review. Remedies must retain original tolerances, domain/envelope checks, derivative obligations, default-start behavior and physical closure.

Cargo/build parallelism is a separate concern. Existing outer worker limits and exclusive sixteen-slot parallel tests are appropriate protections against multiplication. Faster compilation, sixteen Nextest processes, or a server with several runtime threads cannot establish sixteen product cases completing correctly. Full build/test qualification remains outside this read-only review.

Python’s default `-n16 --dist=worksteal` starts separate test processes. The inspected session fixture declares a 64 GiB pool and two threads per process, so those processes have independent resource owners. Summing configured ceilings does not establish actual RSS or an OOM. Aggregate test placement and memory-heavy admission still require explicit reasoning; sixteen configured xdist workers do not qualify sixteen application-native cases.

## 11. Rule impacts and disposition

Findings have one current disposition owner: **[Plan 28](../../plans/28-surrealdb-unified-substrate.md)**. Packet documents may own implementation work and evidence, but should not create a second finding ledger.

| Rule impact | Required decision or change | Route |
|---|---|---|
| <a id="rc01"></a>**RC01 — Canonical guard topology, if narrowed** | F01 may change ADR-0164’s named retention-guard realization and its blueprint protection/publication description. The safety obligations remain. Coalescing within the existing contract may need no authority change | Operator confirms the selected design during plan creation; use the decision/design route if the protected contract changes |
| <a id="rc02"></a>**RC02 — Temporary admission policy, if contention becomes bounded waiting** | F02 may change the current refusal/queue behavior described by blueprint §14.3.2 and the math execution admission policy. Keep genuinely oversized refusal and original clocks/cancellation | Plan 28b/28f and execution owners; amend enduring owner text if behavior changes. No ADR is implied for an ordinary implementation correction within current contracts |
| <a id="rc03"></a>**RC03 — Supported native execution strategy** | F03 requires an explicit backend/profile thread/process strategy and a correctly scoped deployment envelope. Existing backend choices and qualified native-strategy descriptions may need revision | Plan 28 and native/runtime owners; apply the decision route only where an accepted contract is altered |
| <a id="rc04"></a>**RC04 — Study execution strategy** | Replace the current generic single-session/in-turn realization for independent frontier cases with bounded independent workers or a fitting native batch. Preserve dependent sequence order and occurrence semantics. Select the durable worker deployment required by the target | Plan 28c owns study execution; 28f/native runtime owners supply shared preparation/session integration. Update enduring execution-owner text where its sequential realization changes; apply the decision route only if an accepted contract is altered |
| **No rule change required by F04’s accounting correction** | Explicit native stack admission already exists as a contract. Charging the additional persistent team implements it | Session/native scope implementation and focused verification |

No recommendation changes an accepted ADR or architecture owner through publication of this review. No SHOULD exception is proposed. MUST gaps remain open.

For each finding, Plan 28 should record its finding/scenario reference, disposition, decision/work owner and evidence or revisit trigger. A higher retry limit, accepted design, scheduled packet or isolated test pass is not resolution of the complete finding.

## 12. Decision

**Behavioral/semantic adequacy:** The inspected scientific and occurrence contracts have important strengths. The complete sixteen-worker behavior remains unqualified and the existing selected controls fail. Native strategy and corrected resource admission are material unresolved premises.

**Architectural fitness:** **Not acceptable as the complete target**, because AP-03, AP-05 and AP-07 are violated and AP-02 remains unresolved.

**Overall:** **Revise**.

| Priority | Required change | Finding/scenario | Acceptance distinction |
|---|---|---|---|
| High | Reliable complete canonical preparation/publication route under sixteen cases | F01 / S01, S06 | Complete-case success plus retained protection and acknowledgment safety |
| High | Admit/predictably pace temporary preparation according to current demand and retained pressure | F02 / S02, S03 | Small legitimate concurrent cases complete without merely enlarging budgets |
| High | Charge known persistent native team extent through scope teardown | F04 / S05 | Reservation follows real team size and lifetime; refusal precedes creation |
| High | Compose concurrent ordinary independent study cases and select the durable worker deployment | F05 / S07, S03 | One ordinary study demonstrates actual concurrent non-batching case execution with exact outcomes, scientific checks and drain |
| Medium, prerequisite for broad acceptance | Select supported native thread/process strategies and exclusion behavior | F03 / S04 | Explicit supported combinations and examined waiting/cancel/drain behavior |

The next consequential decision is how to compose the existing canonical protection/publication owner and native admission owner for the required sixteen-case route. Local corrections are the preferred starting point. The stopped reference campaign must remain stopped until the user-authorized prerequisites are satisfied; this review supplies no permission or positive evidence to resume it.

Principal artifact: `docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md`.
