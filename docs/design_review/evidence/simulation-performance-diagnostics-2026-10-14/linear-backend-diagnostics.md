---
title: Sparse backend diagnostics and native timer extraction
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
doc_retention: while-dependent
---

# Sparse-backend diagnostics and native timer extraction

Supporting **Interface-checked** evidence for the [principal review](../../reviews/design_review_simulation-performance-diagnostics_2026-10-14.md), baseline `a08f5fa09` plus the concurrent dirty checkout on 2026-10-14. This supplements [the library catalog](library-capabilities.md); it is not the overall review verdict, a runtime measurement or a selected remedy. The researcher authored only this scratch file; the coordinator published it here. No builds, installs, image pulls, services or probes were run.

The native-solver and mathematical-library skills were already loaded. Context7 resolution was attempted for MUMPS and SPRAL and returned unrelated libraries; its oneMKL result was queried but was insufficiently specific. Exact MUMPS 5.9.1 captured source/manual, vendored KLU, Ipopt 3.14.20 source, Clarabel 0.11.1 source, available solver headers and primary upstream documents were used instead. Relevant enduring authority remains blueprint §7, §14, §19 and §23.1–§23.2.

## MUMPS 5.9.1

Exact evidence: `.codex/skills/native-solver-libraries/content/corpus/mumps@5.9.1/doc/userguide_5.9.1.pdf`, manual §5.9, §5.25 and §7.1–§7.2; `include/{dmumps_c.h,dmumps_struc.h}`. The PDF was read through text extraction, preserving section/page context; no document was edited. MUMPS indices below are the manual's **one-based** indices; C arrays use index minus one. The native image's version declaration is not a proof of which linked factor backend a given attempt actually used.

The local C/Fortran structures contain `INFO/INFOG` arrays of 80 integers and `RINFO/RINFOG` arrays of 40 doubles. Some introductory manual text still says dimension 20 for real arrays, while the same section discusses later fields and headers declare 40; use exact header layout for ABI. `INFO`/`RINFO` are process-local; `INFOG`/`RINFOG` are aggregated global outputs. Global maximum and global sum are deliberately different observations.

### Available stage outputs

| Stage | Fields | Meaning and diagnostic use |
|---|---|---|
| Every completed job | `INFO(1:2)`, `INFOG(1:2)` | Status/warning and associated detail. Global status selects/combines process outcomes. Save/restore success describes that job, not the prior factorization's scientific success. |
| Analysis | `RINFO(1)`, `RINFOG(1)` | Estimated elimination flops, local/global. A prediction rather than measured elapsed cost. |
| Analysis | `INFOG(7)`, `(32)`, `(23)`, `(24)`, `(33)` | Actual ordering; sequential/parallel analysis; effective preprocessing, pivot-order and scaling settings. Scaling may still be deferred until factorization. |
| Analysis | `INFO(5:6)`, `INFOG(5:6)`, `INFOG(8)` | Estimated largest front, elimination-tree node count, structural symmetry; symmetry may be `-1` when not computed. |
| Analysis | `INFO(3:4)`, `(7:8)`, `(24)`; `INFOG(3:4)`, `(20)` | Estimated factor real/integer storage, workarray requirements and factor entry count. Storage entries differ from factor nonzeros, especially for symmetric factors. |
| Analysis | `INFO(15)`, `(17)`; `INFOG(16:17)`, `(26:27)` | In-core/out-of-core full-rank memory estimates in decimal MB; global pairs are maximum/sum. |
| Analysis, compressed modes | `INFO(29:38)`, `INFOG(36:47)` | Conditional low-rank factor/contribution-block workarray and memory estimates. Depend on compression assumptions and settings; not actual memory. |
| Factorization | `RINFO(2:4)`, `RINFOG(2:3)`, `(14)` | Assembly flops, full-rank/theoretical elimination flops and effective elimination flops; BLR can make effective work smaller. |
| Factorization | `INFO(9:11)`, `(27:28)`; `INFOG(9:11)`, `(29)`, `(35)` | Actual factor storage, largest processed front, full-rank-equivalent factor entries and BLR-compressed entries. Enables predicted/actual fill comparisons. |
| Factorization | `INFO(12:14)`, `(18)`, `(23)`, `(25)`, `(40)`; `INFOG(12:14)`, `(25)`, `(28)`, `(50)` | Off-diagonal or negative pivots, delayed eliminations, memory compressions, null pivots/deficiency, eliminated pivots, static tiny-pivot perturbations and negative null pivots. Meaning depends on symmetry and null/rank-revealing controls. A variable delayed repeatedly is counted repeatedly. |
| Factorization | `INFO(16)`, `(21:22)`, `(39)`; `INFOG(18:19)`, `(21:22)` | Allocated versus effectively used memory and workarray occupancy. Allocated data excludes caller WK_USER; effective-used memory includes its used portion. These are not interchangeable with RSS. |
| Solve | `INFOG(15)`, `INFO(26)`, `INFOG(30:31)` | Per-solve refinement steps and actual solve working-memory maximum/sum. Not counts of outer nonlinear iterations or factor calls. |
| Optional solve error analysis | `RINFOG(4:8)` | Matrix/solution infinity norms, scaled residual and two componentwise backward-error terms. Requires eligible error-analysis/refinement settings. |
| Expensive solve error analysis | `RINFOG(9:11)` | Forward-error bound and condition estimates for the linear system. Enabled by `ICNTL(11)=1`, which the manual says can cost substantially more than the solve itself. |
| Out-of-core / save | `RINFO(5:8)`, `RINFOG(15:18)` | Estimated/actual disk use and save/structure sizes, conditional on the operation. |
| Factor pivot diagnostics | `RINFOG(19:21)` | Smallest pivot including/excluding selected null/static pivots and largest pivot; interpretation uses preprocessing and pivot controls. Only `(21)` is marked experimental in the examined section. |
| Accelerators | `RINFO(9:10)`, `RINFOG(22:23)` | Offloaded flop count and communication-inclusive accelerator time (local or global sum/average as documented). Irrelevant/uncollected for an unactivated accelerator path. |

Integer storage/count outputs can encode large values as a **negative value whose absolute value is in millions**. Decode each documented field rather than interpreting all negatives as failure or all integers as raw counts. Decimal MB is not MiB. Inertia can exclude a parallel root with default `ICNTL(13)` or exclude null pivots; exact negative-eigenvalue claims need those preconditions and warnings. This checkout normally uses sequential MUMPS, but runtime identity must establish that for the attempt.

Error analysis is disabled by default (`ICNTL(11)=0`) and is incompatible with several distributed/multiple-RHS/forward-elimination/null-space modes; zero outputs do not establish zero error. Refinement can already produce backward errors without full error analysis. Enabling expensive diagnostics alters computational cost and is not passive inspection.

### Material live capability

Manual §5.25 explicitly documents **factorization progress**: `RINFO(3)` is updated during factorization and can be compared with local estimated `RINFO(1)`. Actual theoretical flops may exceed the estimate because numerical pivoting increases work. This is local per MPI process and, with BLR, theoretical full-rank work rather than effective compressed flops. It is not elapsed time, a bounded fraction guaranteed to stay under one, or global progress without aggregation.

The same section documents setting `ICNTL(50)=1` during numerical phases to request cancellation: factorization returns `-80`, solve returns `-81`, with process rank detail; the caller must reset the control before reuse. Non-working host mode requires a working process to receive the request.

This is a native documented avenue stronger than a blanket “factorization is opaque.” It **does not establish safe Rust/C concurrent reads/writes** of the current linked Fortran structure: synchronization, visibility, lifetime, compiler/ABI behavior and instance ownership still need qualification. The current Ipopt C++ linear interface privately owns MUMPS state; the repository sequence ABI does not export that instance/progress/control. No concurrent accessor is prescribed here.

### Timing and current Ipopt forwarding

The standard scalar statistics above do not supply general analysis/factor/solve wall-time arrays. Accelerator time is a conditional exception; Ipopt's phase timers provide another route. Native textual timing output is not a reviewed structured API. `IpMumpsSolverInterface.cpp` currently reads status, effective preprocessing/ordering, factor storage and inertia, prints selected data and handles memory retry/error; this does not imply the repository final metrics expose all MUMPS arrays.

## SPRAL SSIDS 2025.09.18

Exact primary sources: [pinned C documentation](https://raw.githubusercontent.com/ralna/spral/v2025.09.18/docs/C/ssids.rst), [pinned execution source](https://raw.githubusercontent.com/ralna/spral/v2025.09.18/src/ssids/ssids.f90), plus the available solver generation's `include/spral_ssids.h`. The inspected generation prefix is `/home/paul/.cache/pse-arrow/native/solver/cda1ac69f0b060964311cd8d94b9d944839caf67057985e60530a101da0e2188/`; possession is not proof that it is the active linked generation.

Header-visible `spral_ssids_inform` supplies `flag`, input duplicates/missing-diagonal/out-of-range counts, rank, tree depth, largest front/supernode, delayed pivots, `num_factor`, `num_flops`, negative and 2x2 pivots, supernodes, allocation status and CUDA/cuBLAS errors. `num_factor` means factor **entries**, not factorization calls. Analysis estimates omit pivoting; factorization reflects it. Delays count repeated propagation. No general timer or memory-byte field is in this inspected C structure. C docs mention CPU/GPU flop members absent from that header; treat their C ABI exposure as unresolved rather than assuming availability.

`spral_ssids_analyse[_coord/_topology/_ptr32]` returns `akeep` and may return actual ordering; `factor[_ptr32]` returns `fkeep`, scaling and inform; `solve/solve1` consumes factors with explicit RHS count/job. `enquire_posdef/indef` supplies pivot data after an eligible successful factor. Reuse of akeep/fkeep is possible, but a call/reuse census needs owner instrumentation. Source copies factor inform into solve inform: retained factor fields are not new solve work. Errors may leave partial/default fields; status and phase are indispensable. No asynchronous monitor or in-call read contract was established.

The exact Ipopt SPRAL interface times symbolic/factor/backsolve operations and prints delays/entries/flops/front, uses rank/inertia/error and delayed-pivot information in refresh/scaling decisions. These native observations are available inside that owner but not all forwarded by the repository ABI. No extra solve/evaluation is needed to copy existing inform; enquiring pivots still has nonzero inspection/copy cost.

## Intel oneMKL Pardiso

Available inspected header `include/mkl_version.h` identifies 2026.1.0 (`INTEL_MKL_VERSION=20260100`); `mkl_pardiso.h` declares `pardiso`, `pardisoinit`, 64-bit variants, diagonal/export and handle persistence APIs. This exact **oneMKL** `pardiso` signature has `iparm` and `error`, **no `dparm` argument**. Do not transfer standalone Pardiso's dparm statistics or index conventions. Header generation alone does not establish loaded runtime identity.

The official [2026 C iparm reference](https://www.intel.com/content/www/us/en/docs/onemkl/developer-reference-c/2026-0/pardiso-iparm-parameter.html) defines these zero-based outputs:

| Index | Diagnostic |
|---|---|
| `[6]` | Actual refinement steps during solve. |
| `[13]` | Perturbed pivots after eligible factorization. |
| `[14:16]` | Symbolic peak/permanent and factor-storage memory, kB, computed during analysis. Peak formula: `max([14], [15]+[16])`. |
| `[17]` | Factor nonzeros, enabled by negative input. |
| `[18]` | Factor work in millions of flops; negative input enables calculation and increases ordering cost. |
| `[19]` | CG/CGS completed iterations or encoded failure/recomputation outcome. |
| `[21:22]` | Positive/negative inertia for eligible indefinite matrices. |
| `[29]` | Failing equation for detected zero/negative pivot, not a general count. |
| `[62]` | Additional minimum out-of-core memory; conditional mode. |

Order/matching/scaling, refinement limit and in-core/out-of-core settings provide causal context. Reporting is activation-dependent; iteration and matrix-type meanings differ. No native wall-time counter, general factor-call census or safe asynchronous getter was established in this ABI.

[Pardiso API reference](https://www.intel.com/content/www/us/en/docs/onemkl/developer-reference-c/2026-0/pardiso.html) and exact header separate analysis, numeric factorization and solve phases. Owner-observed phase calls/time can distinguish repeated work without extra scientific evaluations; `maxfct/mnum` and retained handles are identity/context, not proof of successful reuse. Existing repository `mkl.rs` owns threading/linked-loader identity, not a complete Pardiso metric collector. Clarabel/PETSc/Ipopt wrappers may own/private-copy iparm; extraction must be established per wrapper rather than assumed from C visibility.

## KLU

Exact source now verified directly in `vendor/suitesparse-sys/vendor/KLU/Include/klu.h`: KLU 2.3.3 bundled with SuiteSparse 7.7.0 through vendored sys 0.1.4, subject to selected build/host profile. Earlier cached header used for discovery agreed but is not the ownership authority. Use `klu_l_*` for current 64-bit-index adapters.

| Native structure / call | Available facts and limits |
|---|---|
| Symbolic | Symmetry, estimated factor flops and estimated L/U entries are computed for AMD ordering; block count/largest block and chosen ordering/BTF are general. Structural rank requires BTF. Estimation is not actual numeric factor work. |
| Numeric | Actual `lnz/unz`, maximum per-block L/U entries, blocks, permutations/scaling and off-diagonal storage. Factor entries may include diagonals; summing L/U needs a stated convention. |
| Common | `status`, factor-storage reallocations, structural/numerical rank, original singular column, off-diagonal pivots, BTF work, current/peak allocated bytes. Rank/singular/noffdiag use `-1` for not computed; numerical rank is first zero-pivot location, not a general rank estimate. |
| Optional calculations | `klu_l_flops`, `klu_l_rcond`, `klu_l_condest`, `klu_l_rgrowth` fill flops/condition/growth fields. These are additional native calculations, not guaranteed prepopulated outputs; condest/growth have work costs. |
| `klu_l_refactor` | Updates factors for the same structural pattern using previous pivot/order/storage. With halt-if-singular, failure leaves partially defined numeric values. Never expose that object as usable merely because retained memory exists. |

Current `dynamics/linear.rs::Klu::refactor` already performs rcond and conditional condest, rejects failed/nonfinite/invalid estimates and falls back to full factorization. Its full-factor count is local/test-oriented. The existing reasoning distinguishes no factor, pattern incompatibility, failed refactor, conditioning rejection and new full factor, but does not publish a complete per-cause stream. These existing diagnostic calls may be observed; no new calls should be silently added to inspection. KINSOL/SUNDIALS KLU has its own native setup policy; a shared backend name does not prove identical lifecycle accounting.

`mempeak` is owned allocator history, not process peak RSS or per-attempt peak unless reset/instance lifetime proves it. No built-in per-call timing, safe concurrent common/factor read, or universal symbolic/numeric reuse counter was established. Publish completed-owner copies/checkpoints; external sampling can describe an opaque operation without assigning its scientific meaning.

## Ipopt 3.14.20 structured TimingStatistics

Exact source: skill corpus `ipopt@3.14.20/src/Algorithm/{IpTimingStatistics.hpp,IpTimingStatistics.cpp,IpIpoptData.hpp,IpIpoptAlg.cpp}`, `src/Common/IpTimedTask.hpp`, `src/Interfaces/IpIpoptApplication.{hpp,cpp}`; [pinned upstream](https://github.com/coin-or/Ipopt/tree/releases/3.14.20/src).

There is a direct **structured C++ route**, not only print parsing: `IpoptApplication::IpoptDataObject()` gives `IpoptData`, whose `TimingStats()` gives `TimingStatistics`. Task accessors expose `TimedTask`, with CPU/system/wall totals plus `IsEnabled` and `IsStarted`. The repository bridge already owns the application and receives data during intermediate callbacks, but has not exported detailed task timing in its result ABI.

Task coverage includes overall algorithm, problem statistics, initialization, Hessian update, output iteration, barrier update, search direction, acceptable-trial computation, acceptance, convergence, primal-dual system total/single solve, residual computation, augmented-system multisolve, linear scaling, symbolic factor, numeric factor, backsolve, structure conversion/initialization, quality search, corrector and objective/gradient/equality/inequality/Jacobian/Hessian evaluations. Generic `Task1`–`Task6` are internal placeholders whose meanings require actual call-site evidence.

`timing_statistics` defaults false and disables detailed tasks while leaving overall time. `print_timing_statistics=yes` sets `timing_statistics=yes` and prints after native execution. Activation must be recorded before zero timing is interpreted. `IsFunctionEvaluationTimeEnabled` and aggregate evaluation CPU/system/wall accessors exist. Native cache hits need not call user evaluations, so native function time and adapter-inclusive work can differ.

**Live restriction:** `TimedTask::Total*Time` asserts the task has ended; start getters assert it is running. Totals accumulate at End/EndIfStarted, not continuously. At an intermediate callback, overall algorithm and enclosing output/other tasks can be active. A blanket sample of every total violates the API precondition even on the owner thread. Owner-safe snapshots require activation and running/completed state handling; do not stop/restart timers merely to inspect them. C++ source exposes many task accessors only non-const, so pointer constness and callback ownership are integration details to establish without unsafe cast assumptions.

Error exits have task cleanup paths, but serious exits can leave absent statistics/data; check actual availability and ended state rather than fabricating final totals. Timing categories overlap hierarchically; CPU/system/wall are not additive. Reading/copying existing timer state need not evaluate the model, but adds observer cost. Structured access does not authorize another thread to read an active application.

## Clarabel 0.11.1 timer extraction: bounded conclusion

Exact source: skill corpus `clarabel@0.11.1/src/{lib.rs,timers/timers.rs,solver/core/solver.rs}`; [pinned timers](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/timers/timers.rs).

`timers` is a public module; `Timers` is public and Debug/Default, but its stack/subtimer map and inner timers are private. Public observation methods are `total_time` and `print`; there is no examined typed iterator/per-key duration getter, serialization or Clone. Debug/printed text exists but is not a selected stable extraction interface. Control methods reset/start/stop/suspend/resume mutate instrumentation and are not passive readers.

The solver's `timers: Option<Timers>` is public. `solve` takes it out, runs named timers locally, then restores it at completion; a termination callback receives `Info`, not this local timer tree. Internal `InnerTimer::elapsed` returns accumulated duration and excludes the current in-flight interval until stopped/suspended. Named scopes include solve/default start/IP iteration, cone scaling, KKT update, KKT solve and post-processing; `notimeit` explicitly excludes selected output/callback overhead. This is a useful source-visible decomposition, but neither public live phase-map extraction nor a full inclusive wall-time census is established.

Current adapter exports `DefaultInfo.solve_time` and iteration/convergence/linear-info facts, not this timer hierarchy. Further structured extraction requires a deliberate exact-interface decision; the review should not mistake existing Debug/print support for a stable typed API or choose parsing/patching in advance.

## Decisions these facts support, without selecting remedies

- Reclassify MUMPS in-factor work and cancellation as **available native facilities with an unqualified integration/synchronization path**, rather than genuinely unavailable.
- Reclassify Ipopt detailed phase time as **structured C++ capability not currently exported**, with activation and active-task preconditions; current ABI absence is not library absence.
- Keep Clarabel nested timer extraction **source-visible but public typed access unresolved**; its public total is insufficient for live phase questions.
- Distinguish sparse analysis estimates, returned numerical work, optional diagnostic recomputation, allocator history and wall time. Each answers a different cause of expensive/repeated work.
- Preserve partial status, mode, native instance/generation, stage and availability for every field. No absent/unactivated field becomes zero. No concurrent native getter or extra scientific evaluation is justified by this evidence.

Remaining bounds: no runtime synchronization proof, native runtime version/linkage probe, per-plugin census, wrapper patch or observer-overhead measurement was performed. Primary source contracts identify feasible review questions; all **Tested/Measured** and whole-product conclusions remain outside this supporting inquiry.
