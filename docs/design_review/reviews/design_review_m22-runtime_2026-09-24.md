---
title: M22 runtime, ownership, publication and reuse review
date: 2026-09-24
status: reviewed
reviewer: /root/final_runtime_review
implementation-authors: [/root]
---

# M22 runtime review

## 1. Scope, purpose and coverage

| Field | Review scope |
|---|---|
| Subject | Plan 14 M22 implementation; compiler workspace, mathematical artifact cache, native job lifecycle, public jobs/results, exact result publication and their acceptance controls |
| Standard | Core 2.0, DP-01–DP-24 and G1–G8; process-simulator profile 1.0, PS-01–PS-13 and PS-G1–PS-G3; pse-arrow binding |
| Tier · purpose | Design · conformance, bounded to the assigned M22 runtime gates G1/G4/G5/G6 |
| Reviewer | `/root/final_runtime_review`, independent of implementation author `/root`; no product changes |
| Decision | **Accept-scoped** for G1/G4/G5/G6: 0 open runtime MUST findings; completed local Linux functional and measurement evidence supports the examined contracts |

**Outcome sought.** Establish that immutable process definitions remain authoritative,
reuse preserves their meaning, native attempts retain their resources until completion,
and partial or interrupted work cannot become an apparently committed solution.

**Supported scope.** Local Linux design-stage qualification under the M22 packet and
ADR-0087. Algebraic roots/optimization, dynamics and fitting share the reviewed lifecycle.
Cold/warm case preparation, source edits, bounded native jobs, cancellation, retained
results and exact publication are in scope. Release profiles, exhaustive feature sets,
coverage, Windows/macOS, distribution artifacts and remote CI are outside this review.
The acknowledged upstream `proc-macro-error2` warning is not reported as repaired.
Strict Clippy cleanup is outside the maintainer-selected functional closure; this review
does not claim either strict Clippy configuration passed.

**Method and coverage.** Read current source, relevant uncommitted changes, the M22 packet,
foundation contract, blueprint §5 and §14.3–§14.4, and the named test bodies. Used pinned
ripgrep 15.2.0 and ast-grep 0.45.3 to locate and structurally inspect worker construction,
publication calls and owner attachment. Source reasoning attacked stale cache insertion,
absent/missing dependencies, concurrent revision preparation, abandoned callers, native
thread-local destruction, result reuse, cancellation and partial publication. No broad
test campaign was rerun by this reviewer.

The scientific reviewer owns physical/numerical correctness and G2/G3/PS-G1/PS-G2/PS-G3;
the claims/library reviewer owns G7/G8 and the complete measurement interpretation.
Those gates are not silently approved here. This review does not requalify every Delta
maintenance or remote-store failure mode. It inspects the existing local publication
composition and settlement mechanism used by M22. No hardware-wide OOM guarantee or
bitwise numerical determinism is claimed.

## 2. Authority and identity map

| Fact | Type and owner | Revision/update boundary | Derived forms |
|---|---|---|---|
| Authored model, values and source declarations | Generated `ModelDeclaration`, `SourceDeclarations`; immutable `ModelRevision` | `ModelBuilder::freeze` validates the complete draft; `edit` copies it | Checked source batches, compiler `Inputs`, case plans |
| Physical interpretation | `PhysicalContext`, admitted `PhysicalInventory` | Exact quantity/precondition identity; admitted source rows retained | Compiler physical key and provider ports |
| Prepared mathematical program | Compiler-issued private `ArtifactRequest` | Body, demand, derivative order, build/source identity and numerical profile enter the key | Shared immutable compiled body; never a new authoring authority |
| Attempt and result | `RunHandle`, `RunResult`, unique run ID | One owned native attempt; terminal report after join | Once-encoded result tables and explicit publication command |
| Durable publication | `PublicationRoot { location, version }` plus generated control row | Member writes precede one conditional control commit | Exact member providers, descriptor and inspection session |

**Implemented:** `workflow/model.rs::freeze` frames model/source/physical/provider identity
and retains independent immutable revisions; `math.rs::prepare_revision` holds one compiler
lock across input installation and preparation. Model names, object addresses and solver
ordinals do not substitute for the content identity used by program reuse. Runtime pointer
equality in `Runtime::start` checks ownership compatibility only.

**Physical-semantics scope.** This review checks retention and routing of physical meaning,
not the correctness of the property model itself.

| Model element | Dimension/unit and basis | Convention/envelope authority | Runtime preservation |
|---|---|---|---|
| Variables, parameters and constraint observations | Generated quantity/unit IDs and admitted quantity registry | Original physical declarations; numerical qualification delegated to scientific review | Result rows preserve semantic IDs, quantity/unit IDs, bounds and explicit unavailable observations |
| Provider selection | Registered native specification and selected output | Exact provider specification identity and admitted source declaration | Identity enters model/compiler products; workers are attempt-local |
| Physical source support | Registry-declared relations and support closure | `PhysicalInventory::load` and `support_closure` | `physical_from_documents` retains support batches; publication rejects conflicting inventories |

## 3. Contracts and invariants

| Contract | Enforcement and observable failure | Evidence |
|---|---|---|
| Invalid edits cannot partially install compiler inputs | `CompilerWorkspace::publish` validates before setters; runtime serializes publication/preparation | **Implemented:** `workspace.rs:829`, `math.rs:273`; atomic-invalid-batch and clean-rebuild tests |
| Mutable solver/evaluator state belongs to its worker | `ExecutionWorker` owns its case and lease; dedicated thread executes the closure | **Implemented:** `math.rs:173`, `math/jobs.rs:50` |
| Cancellation does not release a running native allocation early | Detached supervisor retains slot, CPU permit and lease until `JoinHandle::join` returns | **Implemented:** `math/jobs.rs:70`; delayed TLS-destructor control inspected |
| Late cache build cannot repopulate invalidated retention | Epoch comparison and insertion occur under the same publication mutex as invalidation | **Implemented:** `math/artifacts.rs:121`, `:142` |
| Last readers retain accounted storage | Compiled bodies retain `ProgramOwner`; result arrays receive retained buffer owners | **Implemented:** `math/artifacts.rs:118`, `workflow/simulation_results.rs:239` |
| Incomplete publication cannot become an exact committed root | Native member versions feed the control-last command; commit requires one valid control-version receipt | **Implemented:** `workflow/publication.rs:34`, `delta/publication_plan.rs::compose` |
| Retry/uncertainty does not imply rollback or exactly-once by label | Native settlement distinguishes conflicts/unknown outcomes; reconciliation compares the complete prior control row | **Implemented:** `delta/publish.rs::reconcile`; public command is one-use and has no implicit retry |

**Outcomes.** Rejected/unattempted/constant/native outcomes are separate; native termination,
assurance, feasibility and candidate presence remain separate in result rows. A retained
candidate is not automatically a verified solution. `RunResult::tables` memoizes encoding
of the joined immutable report and cannot rerun a solver.

**Well-posedness boundary.** `MathService::prepare_solve` derives the route and applies
class-specific structural admission before creating solver state. Source roles, coefficients
and fixed/parameter assumptions are checked against prepared inputs. Numerical adequacy of
that structural admission belongs to the independent scientific review; this report does
not certify it from a call-site name.

## 4. Derivation and execution

| Stage | Inputs and reuse | Effects and ownership | Termination/equality |
|---|---|---|---|
| Draft admission and immutable revision | Generated declarations, physical/provider inventory, registry | Bounded reservation; no solve | Atomic rejection; exact current-format identity |
| Salsa preparation | Tracked input fields, selected definition/provider lookups including absence, structural and fixed/parameter assumptions | Serialized workspace; finite generations/LRU; cancellation remains transient | Clean-rebuild comparison; no external native evaluator constructed in a tracked query |
| Artifact construction | Compiler-issued complete key and selected optimizer/evaluator profile | Existing shared cache/flight service; explicit worker boundary and pooled allowance | Build errors retryable; invalidated completions not reinserted |
| Native attempt | Prepared case, values, solver policy, compatible start | CPU/job admission; attempt-local native objects; result owner retained | Explicit limits/failures/cancellation; return only after join |
| Result encoding/publication | Immutable report, exact source declarations and physical context | Once-only encoding; retained Arrow owners; control-last Delta command | Exact requested root/member versions; mismatched source inventories rejected |

**Numerical columns for the native stage.** Formulation, derivative order and solver class
arrive through prepared contracts and profiles; this review does not independently certify
their formulas. `prepare_solve` rejects stale fixed/parameter assumptions. Compatible warm
state is scoped to the finite sequence, cleared on failed steps or `Fresh` policy, and
guarded by layout/data/backend compatibility. Typed native reports carry post-solve quality
and assurance. Their scientific tolerance and derivative claims are reviewed separately.

The ownership boundary crosses with immutable programs, generated source data and owned
reports; mutable providers and native handles stay on the admitted worker. Explicit foreign
allowances are conservative admission policy, not measured heap usage or process RSS.

## 5. Journeys examined

| Journey | Mechanism and evidence |
|---|---|
| Edit and re-solve | `ModelRevision::edit` preserves the original; `prepare_revision` atomically selects the new inputs. Compiler tests compare reused/clean products after values, parameters, sources, physical context, membership and provider changes. |
| Parallel cases | Shared immutable artifacts, separate workers and attempt IDs; one pool and CPU budget. `independent_workspaces_use_one_pool_and_worker_thread` and `publication_resource` cover the relevant boundaries. |
| Cancel before admission | Cancelled waiter exits without executing its closure and releases unused admission; `cancellation_before_entry_releases_admission_without_starting`. |
| Abandon after worker execution | Test blocks a native TLS destructor, abandons the caller, and checks CPU/job/pool ownership remains until release and join. |
| Fail or cancel an actual native solve | `native_nlp_terminal_failure_panic_limit_and_after_entry_stop` injects after native trial entry and distinguishes evaluation/panic/limit/cancel outcomes. Scientific classification is independently reviewed. |
| Publish, fail, reopen | `publication_resource` exercises exact reopen, nonexistent version refusal, pre-cancel and cancellation after actual store write. Complete member/control composition is inspected directly; no remote-store recovery generalization. |
| Python wait and result retention | PyO3 async `CancelWait` requests cancellation while the native supervisor remains owner; the original public handle can await its terminal result again. |
| Last exported Arrow array | The component witness starts a fresh process with `resident_bytes=0`, retains a sliced array after closing readers/publication, verifies its contents, and observes the pool charge decrease only after releasing that final array. Cache pins and active loads are separately zero. |

The final inspection fixture admits `registry.schema_batches()` before composing its
durable artifact plan, so registry self-description uses the same publication route.
Python inspection fixtures explicitly select two threads, one target partition and two
concurrent loads. Removing additive cache-capacity arithmetic and exhausted-reader pin
assumptions preserves the ownership proof: reader pins describe active readers, while
exported buffers hold their own leases. The fresh-process witness measures the latter
directly; it does not require process-wide memory or every pool charge to become zero.

Adding a unit/property implementation, numerical recycle policy, difficult dynamic events,
and physical reference validation are covered by the scientific review. Here they inherit
the same reviewed ownership and publication interfaces, not a separate execution owner.

## 6. Gates

| Gate | Verdict | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **Pass** | Immutable source revisions, exact physical/provider identity, derived compiler/program/result state; functional Q01–Q17 complete with authenticated retained/rerun evidence | None in assigned scope |
| G2 Semantic fidelity | n.a. to assigned verdict | Independent scientific reviewer owns this gate; result/source preservation inspected as support | Collect separate gate decision |
| G3 Validity | n.a. to assigned verdict | Independent scientific reviewer owns this gate | Collect separate gate decision |
| G4 Hidden behaviour | **Pass** | Explicit native build/solve/store boundaries; inspection encodes joined outcomes; tracked compilation does not own a mutable native evaluator; final functional controls passed | None in assigned scope |
| G5 Consistency and recovery | **Pass** | Joined native lifetime, bounded admission, exact control-last publication, explicit uncertainty and final-array lease control; Q12/Q13/Q16 and cancellation/publication measurements complete | None in assigned scope |
| G6 Transformation and reuse | **Pass** | Complete compiler-issued artifact key; tracked missing/membership inputs; clean-rebuild controls; epoch fencing; Q06 and cold/warm/edit measurements complete | None in assigned scope |
| G7 Truthful capability claims | n.a. to assigned verdict | Independent claims reviewer owns aggregate evidence/performance claims | Collect separate gate decision |
| G8 Library leverage | n.a. to assigned verdict | Independent library reviewer owns complete scope; bounded runtime ledger below supports that review | Collect separate gate decision |
| PS-G1 Physical consistency | n.a. to assigned verdict | Independent scientific reviewer owns this gate | Collect separate gate decision |
| PS-G2 Well-posedness | n.a. to assigned verdict | Independent scientific reviewer owns this gate | Collect separate gate decision |
| PS-G3 Numerical integrity | n.a. to assigned verdict | Independent scientific reviewer owns this gate | Collect separate gate decision |

## 7. Findings and principle verdicts

| ID | Finding | Principles · gate | Evidence/gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| — | **No concrete MUST defect found in the examined runtime implementation** | Assigned G1/G4/G5/G6 | Source and named controls above | No defect-based recommendation | None | Completed functional and measured evidence below |

This is not an assertion that unexamined scientific or library behavior is correct.
Separate reviewers own those decisions; aggregate review collection is not a runtime defect.

| Applicable principle | Verdict at inspected code boundary | Reason |
|---|---|---|
| DP-01, DP-04, DP-05 | Satisfied | One immutable source revision; separate program, attempt and publication identities; editable drafts do not mutate prior results |
| DP-08, DP-09, DP-23 | Satisfied for runtime reuse | Compiler-issued keys, tracked membership/absence, value assumptions and incremental-versus-clean controls |
| DP-10, DP-17 | Satisfied | Stable compiler/program ownership; separate mutable workers; coarse Python interfaces |
| DP-11 | Satisfied for runtime contract preservation | Numerical policy/build/profile enter preparation identity; no new bitwise or speedup claim |
| DP-12, DP-19, DP-20 | Satisfied for owned execution | Finite jobs/history/result limits; explicit unattempted outcomes; joined teardown; coherent publication |
| DP-18 | Satisfied | Explicit build, execution and commit boundaries; result inspection has no solver replay |
| DP-21 | Satisfied for lifecycle provenance | Source/build/provider identity and typed errors survive runtime boundaries; full numerical diagnostic review delegated |
| DP-22 | Satisfied in assigned qualification scope | Functional receipt distinguishes retained observations and reruns; 23 measured cases retain provenance, limits and unavailable submetrics; no aggregate Q18 completion claim |
| PS-08, PS-11 | Satisfied for ownership and reuse only | Temporary execution state is worker-owned; failed steps clear warm state; profile and compatibility qualify reuse |
| PS-10, PS-12 | Unresolved outside this review's numerical scope | Typed outcome preservation is examined; full scientific adequacy requires separate verdicts |

Other principles are not independently rated: physical interpretation, domain rewriting,
mathematical library fitness and schema evolution are not changed by this bounded runtime
review. The mechanisms that carry weight are the native join owner, immutable revision
selection lock, complete artifact key, and exact control version: losing any would create
respectively early resource release, mixed revisions, stale programs or incoherent reads.

## 8. Library-leverage ledger

| Capability | Current composition | Fit and gap | Recommendation |
|---|---|---|---|
| Incremental semantic dependency tracking | Salsa 0.28.4 tracked queries, input fields, LRU and cancellation; `CompilerWorkspace` binds domain inputs | Project logic owns domain identity and admission, not another query engine | Retain |
| Completed artifact retention | DataFusion 55.1.0 `DefaultCache`, shared existing `Flights`, typed compiler request | Domain key/epoch/owner adaptation is necessary; no new parallel cache introduced by M22 | Retain |
| Native concurrency and join | Tokio 1.53.1 semaphore/oneshot/watch plus standard-library thread join | Native thread lifetime and TLS teardown require explicit supervision; no new executor algorithm | Retain |
| Publication | Native Delta write/conditional commit and DataFusion composition | Project code owns complete declared member selection and semantic reconciliation | Retain |
| Python async boundary | PyO3 0.29.2 and `pyo3_async_runtimes`; small cancellation guard | Native completion remains authoritative across cancelled Python waiters | Retain |

This ledger is **Interface-checked/Implemented** from local pinned declarations and current
callers. It is not a second full library review, nor a claim that every feature of a listed
library is qualified.

## 9. Alternatives

| Alternative | Meaning/ownership | Bespoke code and risk | Evidence and decision |
|---|---|---|---|
| Current baseline with M22 qualification additions | One source/compiler/runtime/publication route | Retains existing small adapters and explicit lifetime controls | Selected; named tests attack actual failure boundaries |
| Rebuild compiler/evaluator for every case | Retains source meaning but discards stable preparation | Simpler invalidation, additional work; does not remove native lifetime obligations | Rejected as a general route; clean rebuild remains the test oracle and cold baseline |
| Library-owned retention/concurrency | Salsa, DataFusion cache and Tokio/standard join | Coincides with current composition; project retains only domain keys and lifecycle policy | Selected |
| Simplest viable execution/publication | Immutable prepared case, owned worker, joined result, explicit one-use publication command | Coincides with current public route; avoids new retry/serialization framework | Selected; no additional mechanism proposed |

## 10. Verification and evidence limits

Baseline is **zero failures**. The reviewer inspected, and did not rerun, these commands.
Paths below normalize the recorded repository-root prefix to relative paths.
`build/plan14/m22-functional-cost-input/checks.json` records all 40 selected checks passed,
Q01–Q17 complete, `source_unchanged=true`, no provenance errors and full required-check
coverage. Its continuation records identify retained observations and the impact review
supporting them; this is not a claim that every command freshly executed on the final bytes.

| Evidence label | Exact command and mode | Current result | Limit |
|---|---|---|---|
| **Tested, retained** | `just test --profile ci --success-output final --config-file build/plan14/m22-functional-current/test-nextest.toml`; `rust-boundary`, explicit `pse-relations/force-validate` | 1,697 passed; 0 failed/errors/skipped/not-run; baseline 0 | Original execution from `m22-functional-current` retained with explicit impact review of intervening formatting/style edits; not a fresh final-byte run |
| **Tested** | `just plan14-native --profile ci --success-output final --config-file build/plan14/m22-functional-ready/plan14-native-nextest.toml`; `rust-native`, native force-validation, OMP/OPENBLAS/MKL threads each 1 | 127 selected tests passed; 0 failed/errors/skipped/not-run; baseline 0 | Selected native manifest; authenticated origin `m22-functional-ready` |
| **Tested** | `just plan14-python build/plan14/m22-functional-tested`; Python native journeys with force-validation extension | 4 passed; 0 failed/errors/skipped/not-run; baseline 0 | Four selected public process journeys |
| **Tested** | `just assessment-python-unit build/plan14/m22-functional-qualified`; Python unit | 116 passed; 0 failed/errors/skipped/not-run; baseline 0 | Authenticated origin `m22-functional-qualified` |
| **Tested** | `just assessment-python-component build/plan14/m22-functional-pass`; Python component | 18 passed; 0 failed/errors/skipped/not-run; baseline 0 | Includes final-array ownership witness; bounded fixture settings described above |
| **Tested** | `just assessment-python-integration build/plan14/m22-functional-pass`; Python integration | 4 passed; 0 failed/errors/skipped/not-run; baseline 0 | Local integration scope |
| **Measured** | `just architecture-acceptance build/plan14/m22-costs --phase performance --functional-from build/plan14/m22-functional-cost-input --stop-after plan14-measure`; `performance-native` | `plan14-measure` passed, exit 0; 23 cases complete; 0 failed cases against baseline 0; source unchanged, no provenance errors | `plan14-reviews` is deliberately not run at this checkpoint; Q18 and aggregate completion are not claimed here |

The completed measurement is `build/plan14/m22-costs/plan14-measure.json`, schema
`process-cost-v2`, source digest
`43a97300e00d83aa7e456f9b5a05fc65a4aaac2455833afe6e303892f3589aaf`.
The reviewer independently checked the digests of all 230 retained files across the
23 case records: **0 mismatches**. This authenticates the retained case evidence; it is
not a formal proof of the algorithms or a later documentation digest.

**Measured conditions.** Local x86_64 Linux, rustc 1.98.1, Cargo dev profile, features
`native-process,pse-relations/force-validate`; prebuilt benchmark binary
`target/debug/deps/native_process-37e0d778b3c4ff30`. Each case executes in a separate
process with 10 Criterion samples, flat sampling, 250 ms warmup and a 1 s target extended
for slow operations. Cases select one or four process threads and one nested native
thread. Compilation and build-lock waits are excluded. Cold creates a fresh case/runtime/
compiler owner in the prebuilt application; warm retains preparation across iterations
and applies the recorded tiny initial-value perturbation. These are case costs, not Rust
build benchmarks or claims about every possible unchanged input.

| Selected case | Mean ms | 95% confidence interval ms | Peak pool reservation MiB | Process high-water RSS MiB |
|---|---:|---:|---:|---:|
| `cold-small-1` | 176.965 | 174.927–179.228 | 16,812.442 | 157.449 |
| `warm-small-1` | 48.003 | 47.689–48.275 | 16,812.594 | 157.691 |
| `cold-medium-1` | 276.688 | 274.737–278.645 | 16,812.616 | 163.492 |
| `warm-medium-1` | 173.971 | 171.298–176.577 | 16,812.891 | 159.531 |
| `structure-medium-1` | 182.150 | 161.755–202.235 | 16,812.820 | 160.449 |
| `specialization-medium-1` | 187.831 | 175.112–199.677 | 17,644.949 | 159.852 |
| `cold-large-4` | 752.596 | 744.144–761.434 | 16,813.223 | 166.547 |
| `warm-large-4` | 979.559 | 791.995–1,193.960 | 16,813.919 | 166.602 |
| `publication-cold-1` | 1,854.987 | 1,840.906–1,869.588 | 16,812.442 | 225.523 |
| `cancellation-cold-1` | 175.920 | 174.806–176.959 | 16,813.223 | 174.328 |

**Measured interpretation.** Reuse is tested for semantic correctness; timing does not
establish that correctness. Small and medium warm cases are faster here, while the
four-thread large warm case is slower and variable, so there is no general speedup claim.
Reservations are conservative admission allowances, distinct from actual allocation and
process-lifetime RSS. The values do not establish an RSS cap. Phase summaries include
warmup/calibration operations and inclusive compiler spans, so they cannot be summed as
an exclusive decomposition of Criterion samples. JIT is not enabled; native conversion
and property-state construction are not independently clocked.

The publication case commits and reopens the exact returned control version. Its
publication/reopen phase averages **1,679.812 ms**. The cancellation case requires a
native progress event while no terminal result exists, then cancels and joins with
`Termination::Cancelled`; cancellation-to-join averages **2.206 ms**. Both phase means
cover 11 operations including calibration, rather than the 10 retained timing samples.
The end-to-end samples include case teardown; the retained warm runtime's final drop is
outside each warm iteration, so these numbers are not complete process-shutdown latency.

The selected difficult flash cost uses a **2%** initial density perturbation. The earlier
15% case failed locally with `Infeasible_Problem_Detected`, without feasibility/assurance,
and remains disclosed in the M22 packet and
`build/plan14/m22-measurements/process-cost/flash-difficult-1/process.log`.
Reference values and tolerances were not relaxed. Neither this selected successful case
nor the earlier local failure establishes global convergence or global infeasibility.

Relevant inspected runtime controls include
`actual_artifact_singleflight_profiles_epoch_and_retained_owners`,
`independent_workspaces_use_one_pool_and_worker_thread`,
`cancellation_before_entry_releases_admission_without_starting`,
`abandoned_caller_holds_pool_and_cpu_through_tls_destructor`,
`actual_compile_failure_is_retryable_and_admission_is_finite`,
`generous_workspace_preserves_retention_and_refuses_tiny_budget`, the compiler
incremental-versus-clean/absence/provider cases, `publication_resource`, and the native
after-entry failure/cancellation case. The complete manifest owns exact witness selection;
these names are not a substitute for the authenticated final acceptance report.

Review inspection commands included `git status --short`, scoped `git diff -- ...`,
`just --list`, `rg -n` over named source/manifest paths and these structural queries:

```bash
ast-grep run --lang rust --pattern 'std::thread::Builder::new()' crates/pse-runtime/src/math
ast-grep run --lang rust --pattern '$COMPILER.publish($INPUTS)' crates/pse-runtime/src/math
ast-grep run --lang rust --pattern '$VALUE.with_owner($OWNER)' crates/pse-runtime/src/math
```

These establish syntax locations, not resolved type identity; the enclosing code was read.
Physical unit/property conformance and independent reference oracles are assigned to the
scientific review. No such result is inferred from runtime or publication success.

## 11. Authority changes and exceptions

Followed binding K1 through Plan 14 D02–D04 and ADR-0082–0084 for math/native ownership;
K2 retains surviving generated contracts; K3 preserves the meaning of historical IDs without
editing accepted records. These conflict routes do not waive a runtime MUST.

ADR-0087 and the maintainer-selected local design-stage scope control the qualification
boundary. Excluding an extended gate is not evidence that it passed. No new runtime
exception, blueprint edit or accepted-record edit is proposed by this reviewer.

## 12. Decision

**Accept-scoped: G1, G4, G5 and G6 pass.** There are **0 open runtime MUST findings**
in this bounded implementation review. The completed functional evidence, explicit
retention/impact review, and all 23 measured cases support the examined authority,
ownership, publication and reuse contracts within the local Linux design-stage scope.

| Boundary | Remaining aggregation | Runtime finding | Evidence needed |
|---|---|---|---|
| Other gates | Collect the independent scientific and claims/library decisions | Outside assigned verdict | Separate reviews |
| Final source binding | Finish authority/status documentation and collect independently authored gate records for the final digest | Administrative evidence continuation, not a product defect | Final source-bound review collection and affected documentation checks |

No additional product change or test campaign is requested. This reviewer edited only
this review artifact. The review is evidence and does not itself accept an ADR, amend
the blueprint or assert that the pending Q18 aggregate has completed.
