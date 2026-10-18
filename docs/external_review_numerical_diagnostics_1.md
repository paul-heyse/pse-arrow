---
title: External review of numerical diagnostics
---

# External review of numerical diagnostics

## My assessment

**I would build the next stage around a consistently joined record of numerical work, execution cost, and solution quality for each complete problem execution.** Your code already has a substantial diagnostic foundation. The biggest opportunity is to make that evidence explain *why* a change is faster, and to expose a few expensive operations that currently sit inside broader timing categories.

I reviewed the repository at commit [`9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c`](https://github.com/paul-heyse/pse-arrow/commit/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c), tracing the compiler and mathematical representation, structural transformations, implicit calculations, native backends, dynamics, fitting and shooting, runtime scheduling, result publication, diagnostics, and benchmarks. This is a source and architecture review; **I have not run your model benchmarks, so the optimization candidates below are hypotheses supported by code inspection, not measured speedups.**

My strongest recommendations are:

1. **Join timings, work counts, problem structure, and outcomes for the same individual execution.**
2. **Measure local derivative construction and implicit derivative contraction explicitly.** Sparse global matrices can conceal substantial dense local work.
3. **Investigate the IDAS output-sampling driver.** It currently makes every requested output time a hard integration stop.
4. **Separate oracle mathematics from callback transport, allocation, and diagnostic overhead.**
5. **Preserve the reasons for repeated integrations in fitting, shooting, and sensitivity calculations.**
6. **Use the existing numerical portfolio in controlled comparisons by problem family.** You already support many of the methods that a generic review would recommend adding.

The following is how I would make those recommendations concrete.

## 1. What you already have—and should build upon

Your architecture already separates authored definitions, immutable prepared mathematical products, and attempts/results. Symbolica/Numerica and the native libraries own numerical execution; Arrow/DataFusion serve the data and query boundaries, with canonical persistence handled separately. That is a useful foundation for keeping instrumentation out of the mathematical semantics. [Architecture overview](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/docs/authoritative_design/sections/architecture-overview.md)

The existing diagnostics are more developed than ordinary application tracing:

| Area | Existing capability | Implication |
|---|---|---|
| Runtime observation | Operation identities, relationships, phases, typed values, units, availability, counter generations, bounded histories and explicit loss | Extend these contracts for missing facts. |
| Timeline and CPU capture | Standard Perfetto output, owned capture sessions, Linux `perf`, and Samply conversion | The primary profiling foundation is already present. |
| Numerical work accounting | An inclusive work ledger with charging-owner deduplication and propagation of unknown counts | Preserve this as the authority for charged work. |
| Native solve evidence | Callback totals, backend statistics, structured progress events, bounded histories and native timing in several backends | Improve coverage and joins before introducing another event system. |
| Mathematical execution | Implicit solve, factorization, backsolve, proof/refinement and reuse observations | Add detail where dense construction or contraction remains hidden. |
| Data boundaries | DataFusion operator observations and canonical query statistics | Include their cost in complete execution comparisons. |

The main source anchors are the [diagnostics guide](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/docs/dev/simulation-diagnostics.md), [observation model](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-diagnostics/src/observation.rs), [capture implementation](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/capture.rs), [strategy work ledger](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/math/strategy.rs), and [native progress contracts](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/solve.rs).

**The next deliverable should be a set of reproducible comparison views over this evidence.** For a slow solve, those views should answer:

- Did it perform more mathematical work?
- Did each unit of work become more expensive?
- Did a transformation move work into another layer?
- Did scheduling, memory pressure, or publication dominate?
- Did the run achieve the same requested numerical outcome?

## 2. Make the complete execution the unit of comparison

### Measure two completion endpoints

I would retain separate elapsed times for:

**Time to a result qualified against the original problem.** This includes preparation, transformations, every attempted strategy, native execution, recovery, and the required original-problem assessment.

**Time to the requested result being available to the caller.** This additionally includes the requested analysis products, canonical publication, result materialization, and any reopening/querying that the actual workflow requires.

This distinction is especially relevant in your implementation. The NLP runner performs presolve, native execution, recovery and quality assessment, followed by requested sensitivity/KKT/output-accuracy work. Those activities need identifiable phase boundaries so that their costs can be compared under the same requested outputs. Existing presolve pass timings are useful, but do not by themselves explain the full lifecycle. [NLP runner](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/execution/runner.rs), [presolve pipeline](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/presolve/pipeline.rs)

### Preserve ownership and timing semantics

Each comparison record should reference the existing operation, native attempt, semantic source and work-charging identities. These identities describe different things; joining them explicitly is more useful than collapsing them into one generic “solve ID.”

For each metric, preserve:

- **Scope:** operation, attempt, callback, native session, thread, process or server statement.
- **Meaning:** count, cumulative counter, gauge, elapsed duration, estimate or reservation.
- **Coverage:** complete, sampled, coalesced, truncated, unavailable or unknown.
- **Numerical coordinates:** original or transformed/scaled problem, norm definition, units and tolerance.
- **Counter generation:** the baseline needed to distinguish new work from inherited native state.

Your work ledger already rejects duplicate inclusive charges and preserves unknown observations. Keep those properties when producing performance summaries. An allowance is not measured work, and a known subtotal does not make an unknown total known. [Work ledger](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/math/strategy.rs)

Likewise, **do not sum nested or overlapping durations to obtain total elapsed time**. The runtime already distinguishes compiler admission, resource admission, native worker execution and join waits. A join wait can overlap the native work it awaits. Ipopt’s native timing code explicitly documents overlapping tasks. Use direct endpoint timings for totals, and timeline relationships to explain the critical path. [Runtime jobs](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/math/jobs.rs), [Ipopt timing](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/ipopt/sequence.rs)

### Fix the benchmark pairing before drawing finer conclusions

There is a specific opportunity in the current process benchmark.

`native_process.rs` accumulates observations across benchmark closure invocations, including warmup, and reports phase averages using that invocation count. `case_measure.py` separately reads Criterion’s measured samples and combines the two summaries. The scopes are disclosed, so this is not an invalid benchmark. However, **there is no direct pairing between the work and timing of each individual solve**. Aggregate counters cannot explain a particular slow sample, and their denominator must not silently become Criterion’s measured-only denominator. [Process benchmark](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/benches/benches/native_process.rs), [comparison script](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/scripts/case_measure.py)

I would adopt the existing K4 pattern more broadly: retain elapsed time, phase/preparation observations, numerical work and outcome together for each operation. Keep Criterion’s statistical output alongside those records. If measurement-phase membership is needed, capture it through a verified harness contract rather than guessing from invocation order. [K4 accuracy benchmark](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/benches/benches/k4/accuracy.rs)

This is my **first implementation priority**, because it improves the reliability of every subsequent experiment.

## 3. The most promising code-specific investigations

### A. Measure the amplification between sparse global derivatives and local derivative work

This is one of the most consequential findings.

`JetLayout::new` creates all first-order components and all \(k(k+1)/2\) second-order pairs for the selected local coordinates. Each derivative shape is represented by a vector of length \(k\). Consequently, second-order shape metadata grows cubically with local coordinate width, subject to the existing allocation limits. Other execution paths expand triangular second derivatives into dense output Hessians. [Jet layout](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-math/src/jets.rs), [mathematical execution](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-math/src/execution.rs)

There is already support-aware selection and narrowing inside arithmetic stages. The issue is more specific: a sparse assembled Jacobian or Hessian does not establish that the *selected local representation* is cheap.

For each expensive mathematical body and requested derivative order, collect:

- Selected coordinate count and output count.
- Actual first- and second-order support.
- Taylor width, frame slots and retained/result bytes.
- Preparation, evaluation, expansion and assembly time.
- Requested order versus the order actually evaluated.

A useful diagnostic is the relationship between **local coefficients constructed** and **coefficients ultimately consumed**. Keep both operands visible, especially when support is empty or repeated contributions exist.

There is a second source-level opportunity in `case_patterns`: Hessian construction scans local coordinate pairs and tests support membership before emitting sparse terms. Instrument candidate pairs, emitted terms, unique nonzeros and preparation time. If the scan is material, support-driven enumeration could avoid visiting absent pairs. That change must preserve coordinate aliasing, contribution multiplicity and global triangle conventions. [Assembly pattern construction](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-math/src/assembly.rs)

The likely experiments are narrower local projections, more effective grouping, directional products where the consumer supports them, and alternative derivative representations provided by the mathematical libraries. These should be driven by measured width and demand, not by global matrix density alone.

### B. Separate implicit derivative contraction from sparse linear algebra

The implicit derivative path already factors a sparse unknown Jacobian and solves multiple right-hand sides. That is a strong implementation choice.

However, `derivatives_at_verified_point` also consumes dense residual jets, constructs second-order right-hand sides by scanning the full combined unknown/parameter width twice, and verifies derivative solves using dense Jacobian scans. The sparse factorization can therefore be only one part of the cost. [Implicit derivatives](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-math/src/implicit.rs)

I would add a decomposition covering:

1. Residual jet evaluation.
2. Sparse matrix extraction.
3. Factorization.
4. First- and second-order right-hand-side construction.
5. Backsolves.
6. Derivative residual verification.
7. Result expansion.

You already observe many solve, factor, proof and reuse activities. The additions should expose the contraction and materialization work around them. [Implicit observations](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-math/src/implicit_observation.rs)

This matters when comparing full and reduced formulations. Eliminating variables can reduce the outer system while increasing inner solves, derivative width or qualification work. Measure the complete cost of the reduction, including the work required to establish the original result.

### C. Investigate IDAS hard stops at every output sample

In `Session::segment`, the IDAS driver selects the next requested sample and calls:

```rust
IDASetStopTime(self.mem, target)
```

before solving to that target. The Diffsol driver instead stops at the actual segment boundary and samples crossed output times through dense output. [IDAS driver](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/dynamics/idas.rs), [Diffsol driver](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/dynamics/integrator.rs)

The pinned SUNDIALS 7.1.1 documentation says that `IDA_NORMAL` can step past the requested output time and interpolate back to it. A hard stop restricts that freedom. This makes the current driver a plausible source of extra work when output samples are dense. [User Documentation for SUNDIALS documentation](https://sundials.readthedocs.io/en/v7.1.1/idas/Usage/SIM.html?utm_source=chatgpt.com)

I would run a controlled comparison with:

- Identical equations, initial conditions, tolerances and physical events.
- A fixed first requested output time.
- Increasing output-grid density.
- Current per-sample stops versus stops at genuine integration boundaries.

Measure internal steps, residual/Jacobian evaluations, linear setups, error-test failures, output/interpolation time and total elapsed time. Check trajectory accuracy, conservation and sensitivity results.

The implementation would need to preserve root handling, resets, consistent initialization, post-transition sample semantics and the distinction between native internal time and returned output time. Its budget logic must also distinguish **taking another step** from **serving a sample from already computed interpolation data**.

This is a particularly good early experiment because the mechanism is concrete and the comparison can be tightly controlled. It is not yet evidence of a speedup.

### D. Measure callback transport and diagnostic overhead independently

The shared native NLP callback path allocates result vectors for several derivative demands. PETSc’s callbacks also perform copying and vector/matrix publication around the oracle call. Those costs may matter when individual equations or property evaluations are cheap. [Shared NLP callbacks](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/uno/callback.rs), [PETSc bridge](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/petsc.rs)

The current callback timer also has a particular scope: it includes scoped observation and some admission work, but ends before much of the metric publication. It is therefore not a pure oracle timer or a complete callback-boundary timer. [Callback accounting](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/callback.rs)

For focused investigations, separate:

- Argument gathering and native/Rust conversion.
- Oracle execution.
- Derivative scatter and native matrix/vector publication.
- Allocation and buffer growth.
- Observation publication.

Then compare observation disabled, ordinary observation and detailed capture on both cheap and expensive callbacks. Disabling the observation hub still leaves some final-report accounting in place, so label the baseline accurately.

The observation hub uses `try_lock` and records loss; it does not simply block every producer on a mutex. Nevertheless, name formatting, metric lookup, timing and publication have costs. If allocation profiling shows a meaningful contribution, reusable worker-owned buffers and cheaper metric handles are sensible candidates. Preserve the existing ownership and failure semantics. [Observation update path](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-diagnostics/src/observation.rs)

### E. Preserve the reasons for repeated integrations

Fitting and shooting need a more informative unit than “optimizer evaluation.”

The fitting oracle already distinguishes new trials, exact cache hits and derivative upgrades. Its final identifiability calculation can switch a gradient-only fit to response mode and run forward sensitivities at the candidate. This is legitimate additional work, but it should not be attributed to the optimizer’s ordinary gradient evaluations. [Fitting oracle](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/workflow/fitting/oracle.rs)

Shooting likewise has separate value/forward-sensitivity products and integral-objective adjoints. A mixed objective can require several integration products at the same optimizer vector. [Shooting oracle](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/workflow/shooting.rs)

Record integrations by reason: primal trial, derivative upgrade, forward response, adjoint, curvature, original assessment, and rank/covariance assessment. Report work per distinct parameter vector as well as per callback.

For adjoints, retain the existing forward-recording, replay and backward roles. Diffsol already handles inherited counter baselines carefully when restoring checkpoint solvers. Preserve those deltas alongside checkpoint count, bytes and spacing so that a memory-versus-replay experiment remains interpretable. [Adjoint implementation](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/dynamics/integrator/adjoint.rs)

### F. Measure compilation and reuse economics

Your compiled scalar evaluator is an explicit, conservative option for **Value arithmetic blocks**. It does not make the full provider/derivative pipeline native compiled code. Horner and common-pair optimization controls already exist. [Evaluator policy](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-math/src/library.rs)

Measure the eligible arithmetic fraction separately from input preparation, domain checks, provider calls, result validation and derivative composition. The relevant break-even condition is:

\[
N\left(t_{\text{interpreted eligible arithmetic}}
-t_{\text{compiled eligible arithmetic}}\right)
>
t_{\text{incremental compile, link and load}}.
\]

Use the artifact’s actual lifetime and reuse count for \(N\).

Across the pipeline, distinguish reuse of compiled artifacts, buffers, symbolic factorization, numerical factors, native solver objects, trajectories and permitted primal/dual starts. The native implicit-session cache would benefit from production hit/miss, compatibility, eviction and retained-byte observations. A session being reused is not evidence that numerical factors were reused. [Native implicit sessions](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/implicit.rs)

## 4. Fill native diagnostic gaps selectively

Backend coverage is uneven in useful, identifiable ways.

| Backend | What is already strong | Next useful extension |
|---|---|---|
| **Ipopt** | Intermediate convergence information and completed native initialization, search, trial, scaling, symbolic-factor, numeric-factor and backsolve timings | Add private linear-solver detail only where an experiment needs it; retain overlap semantics. |
| **POUNCE/FERAL** | Native statistics, detailed timers, factor counts, pattern reuse, fill, pivots, inertia, and actual monolithic/Schur events | Use this as a rich reference path; expose retry/refinement detail where currently opaque. |
| **KINSOL** | Nonlinear/function/linear/preconditioner/JVP information and retained sessions | Obtain direct-factor setup/refactor/solve evidence from the actual native owner. Current KLU subphases are explicitly unknown. |
| **PETSc** | SNES information, aggregate linear work, last KSP result and callback observations | Enable event/stage logging through the typed bridge and attribute individual KSP/preconditioner costs. |
| **Diffsol / IDAS** | Step/order/failure information, sensitivity roles and substantial linear-solver observations | Complete factor visibility where native internals remain opaque; join it to driver and integration-purpose information. |
| **Uno** | Oracle, aggregate iteration/subproblem and materialization statistics | Add a bounded structured iteration/subproblem hook; the current bridge explicitly lacks one. |

These gaps are stated directly in the [KINSOL](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/kinsol.rs), [PETSc](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/petsc.rs), and [Uno](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/uno.rs) adapters; the richer POUNCE evidence is in its [statistics projection](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/pounce.rs).

For PETSc, upstream stages/events and logging are directly relevant. Integrate against your pinned headers and expose the controls through your profile; the current wrapper does not make arbitrary upstream command-line options automatically available. [PETSc 3.26.0 documentation](https://petsc.org/release/manual/profiling/?utm_source=chatgpt.com)

For SUNDIALS 7.1.1, `SUNDIALS_BUILD_WITH_PROFILING` enables its native profiler, and `ENABLE_CALIPER` adds Caliper support. This is a useful focused diagnostic build. It improves timing visibility; exact factor/refactor counts still require observations at the responsible linear-solver operations. [User Documentation for SUNDIALS documentation](https://sundials.readthedocs.io/en/v7.1.1/sundials/Profiling_link.html?utm_source=chatgpt.com)

## 5. How I would evaluate numerical methods

Your code already supports a broad portfolio. The selection question should be: **which admitted formulation and method produces the requested result fastest for this problem family, including failed attempts and qualification?**

I would start with this matrix.

| Problem family | Controlled comparisons using existing capabilities | Evidence that should decide |
|---|---|---|
| **Smooth continuous NLP** | Ipopt/POUNCE exact versus limited-memory curvature; eligible native sparse linear solvers and orderings; Uno first-order profiles where appropriate | Derivative cost, trial/restoration work, factor fill/time, convergence and original feasibility/stationarity |
| **Structured NLP** | Presolve settings; full versus eligible reduced formulations; actual monolithic versus Schur execution | Preparation/recovery cost, dimensions, separator size, fill, inner work, fallback reasons and qualified total time |
| **Regular square equations** | KINSOL analytic sparse direct versus analytic JVP/Krylov; forcing terms and admitted preconditioners | Residual/Jacobian/JVP time, nonlinear and linear work, setup cost, memory and original residual |
| **Declared recycle/fixed-point maps** | Damping, Anderson depth/orthogonalization and explicit restart schedules | Raw map residual, map calls, nested solve work, rejected restarts and original equation qualification |
| **Stiff ODE / supported DAE** | Eligible Diffsol/IDAS profiles; sparse linear solvers or admitted JVP routes; output-grid policy | Native steps, error/nonlinear failures, setup/refactor behavior, events, conservation, accuracy and total time |
| **Dynamic derivatives** | Forward sensitivities versus checkpointed adjoints across parameter/output widths; checkpoint spacing; admitted exact versus approximate curvature | Forward/replay/backward work, memory, derivative checks and complete workflow time |

The actual settings and admission limits are visible in [KINSOL settings](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/settings/kinsol.rs), [Ipopt settings](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/settings/ipopt.rs), [POUNCE settings](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/settings/pounce.rs), and [dynamics admission](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/dynamics.rs).

Several distinctions are essential to those comparisons:

**Compare actual execution with requested execution.** A requested Schur configuration may fall back. A requested reuse policy may rebuild. Preserve those outcomes.

**Keep eligibility explicit.** PETSc’s current root profile does not enforce arbitrary finite bounds. Uno is not an exact-Hessian contender. Adjoint routes have event/reset and output restrictions; exact transient Hessians currently require the IDAS route. These are different capability sets.

**Separate allocation reuse from starting-point quality.** Compare fresh allocation/cold start, retained allocation/cold start, and retained allocation/explicit permitted seed independently. Observe numerical refactorization separately.

**Study scaling and tolerances through original-problem acceptance.** Useful diagnostics include scale distributions, the sources of dominant residuals, native convergence measures, and original physical violations. Similar-looking solver tolerances do not necessarily represent the same norm or transformed problem. Ipopt’s interface documentation, for example, distinguishes internal scaled quantities from original-problem quantities. [coin-or.github.io](https://coin-or.github.io/Ipopt/INTERFACES.html?utm_source=chatgpt.com)

**Keep expensive convergence probes in dedicated studies.** True linear residuals and condition estimates can explain failure or poor preconditioning, but have their own cost. PETSc specifically distinguishes true-residual monitoring for convergence studies from timing runs. [PETSc 3.26.0 documentation](https://petsc.org/release/manual/ksp/?utm_source=chatgpt.com)

Once the records are consistent, build performance profiles by problem family and success criterion. Dolan–Moré profiles are useful for displaying the fraction of problems solved within a factor of the best eligible method while retaining failures. [arxiv.org](https://arxiv.org/abs/cs/0102001?utm_source=chatgpt.com)

## 6. The tooling stack I would use

### First priority: deepen your existing stack

**Perfetto plus Trace Processor.** Keep your existing timeline format and add repeatable SQL analyses: expensive operations, derivative-demand costs, failed-attempt overhead, admission delays, reuse economics and publication latency. Current Trace Processor supports exporting static tables as standard Arrow files, which fits your existing analytical boundary well. Runtime-created tables are not included in that export, so persist derived query results deliberately. [Perfetto Tracing Docs](https://perfetto.dev/docs/reference/trace-processor-cli?utm_source=chatgpt.com)

For example, with a compatible Trace Processor version:

```bash
trace_processor query trace.pftrace \
  "SELECT ts, dur, name FROM slice ORDER BY dur DESC LIMIT 30"

trace_processor export arrow_tar -o tables.tar trace.pftrace
```

Treat those tables and summaries as derived diagnostics linked to the existing run identities.

**`perf` and Samply.** Continue using the integrated CPU capture for native/Rust stack attribution. Add `perf stat` campaigns for cycles, instructions, task-clock, context switches, migrations and faults, followed by CPU-specific events when there is a concrete question. Preserve event availability and multiplexing information. Cache-miss counters alone do not establish memory-bandwidth saturation. [GitHub](https://github.com/mstange/samply?utm_source=chatgpt.com)

Your existing optimized profiling build with full debug information is the right starting point. Preserve the repository’s floating-point and target policy when comparing builds. [Build profiles](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/Cargo.toml)

**Heaptrack.** This would be my first additional allocation tool. It can identify allocation volume, churn and allocation stacks across the Rust/native boundary where allocations use intercepted functions. Use it to investigate callback vectors, dense jets, temporary contractions and cache retention. Custom pools need their own accounting. [GitHub](https://github.com/KDE/heaptrack?utm_source=chatgpt.com)

### Add focused tools when the evidence calls for them

| Tool/library | Question it answers | How I would use it here |
|---|---|---|
| **`dhat`** | Which Rust allocations survive or churn within a selected workload? | Small isolated allocation studies; native allocations bypassing the Rust allocator require other coverage. [Rust](https://docs.rs/dhat/latest/dhat/?utm_source=chatgpt.com) |
| **BCC / `bpftrace`** | Where is wall time lost to scheduling, blocking, I/O or kernel activity? | Investigate admission/native-thread waits or publication stalls after the timeline identifies them. Ubuntu packages include `bpftrace` and `bpfcc-tools`. [Ubuntu Server documentation](https://ubuntu.com/server/docs/how-to/debugging/ebpf/?utm_source=chatgpt.com) |
| **`perf` off-CPU recording** | Where are threads descheduled or blocked? | A dedicated profile: current BPF off-CPU stack capture has frame-pointer requirements, including native code. [Linux manual page](https://man7.org/linux/man-pages/man1/perf-record.1.html?utm_source=chatgpt.com) |
| **SUNProfiler / Caliper** | Which SUNDIALS regions consume time? | A bounded native diagnostic build, aligned to the vendored version. [User Documentation for SUNDIALS documentation](https://sundials.readthedocs.io/en/v7.1.1/sundials/Profiling_link.html?utm_source=chatgpt.com) |
| **Gungraun / Valgrind tools** | Did a small kernel’s instruction or allocation behavior regress? | Supplement Criterion for stable, isolated kernels. Gungraun is the successor name to Iai-Callgrind. [GitHub](https://github.com/gungraun/gungraun?utm_source=chatgpt.com) |
| **Tokio Console** | Are async tasks spending time queued, blocked or repeatedly polled? | Runtime orchestration investigations using a dedicated instrumented build. [Rust](https://docs.rs/console-subscriber/latest/console_subscriber/?utm_source=chatgpt.com) |
| **AMD uProf / IBS** | Which instructions or memory accesses limit an AMD host? | Deeper microarchitecture studies after a hot native or evaluator region is identified. [AMD](https://www.amd.com/en/developer/uprof/uprof-performance-analysis.html?utm_source=chatgpt.com) |
| **LLVM optimization remarks / `llvm-mca`** | Why was a hot loop not optimized, or what constrains its instruction throughput? | Small confirmed hotspots; static throughput analysis complements runtime measurement. [LLVM Machine Code Analyzer](https://llvm.org/docs/CommandGuide/llvm-mca.html?utm_source=chatgpt.com) |

On a Linux profiling host, the practical starting set is a kernel-compatible `perf`, Samply, Perfetto Trace Processor and Heaptrack. Add Valgrind and the BPF tools for their specific investigations. Native debug symbols are as important as the tool installation.

### You can start with the current commands

Your existing diagnostic interface already supports:

```bash
just diagnostics list

just diagnostics inspect "$checkpoint" "$operation" \
  --window 60 --related-limit 16 --json

just diagnostics capture "$checkpoint" "$operation" \
  --kind timeline --output /tmp/pse-timeline-new

just diagnostics capture "$checkpoint" "$operation" \
  --kind cpu --output /tmp/pse-cpu-new --duration 10
```

Use the discovered checkpoint/operation values and a fresh output directory for each capture. [Diagnostic command guide](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/docs/dev/simulation-diagnostics.md)

For short benchmarks, arrange profiling around the launched workload. Attaching CPU capture to an already active operation cannot recover CPU samples from its earlier preparation.

## 7. Keep diagnostic overhead and incomplete evidence visible

I would support three operating modes using the existing infrastructure:

**Ordinary execution:** bounded counters, coarse phases, structure metadata, outcome and coverage. Accumulate cheap facts locally and publish at safe boundaries.

**Selected numerical study:** richer iteration histories, derivative decomposition, setup reasons, reuse decisions and integration-purpose records.

**Focused system profile:** CPU sampling, allocation tracing, off-CPU tracing or expensive numerical probes, with their overhead recorded.

The current per-second observation summaries deliberately coalesce activity. They are useful for operational inspection but cannot reconstruct every nonlinear iteration. You already have progress taps, bounded native histories and canonical progress coverage; use those when the study needs finer sequencing. [Progress handling](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-backend-native/src/solve.rs), [workflow progress](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/workflow/progress.rs)

Two attribution details also deserve explicit treatment:

- Process RSS/high-water memory and process CPU are shared by concurrent work. They are not automatically private operation peaks.
- Canonical server statement duration is not client request wall time. Pair it with client latency, serialization/materialization and publication phases when explaining end-to-end cost.

Those distinctions are already reflected in the [runtime observation code](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-runtime/src/observation.rs) and [canonical transport](https://github.com/paul-heyse/pse-arrow/blob/9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c/crates/pse-operations/src/canonical_transport.rs). Preserve them in the resulting dashboards and comparisons.

## 8. Recommended implementation order

### First: make comparisons trustworthy

Extend the existing benchmark record to pair each operation’s elapsed time, work, structure, actual execution choices and outcome. Add clear lifecycle timings around preparation, recovery, original assessment, requested analysis and publication.

Record the build/host/native-library configuration, worker and native thread counts, start policy, cache state, requested outputs and diagnostic mode. Retain failed attempts and their costs.

Establish representative cases covering cheap callbacks, expensive properties, wide derivatives, difficult nonlinear convergence, structured sparse systems and dynamic fitting. Measure both isolated latency and concurrent throughput.

### Second: run the highest-value source-directed experiments

I would prioritize:

1. **Local derivative width and implicit contraction cost.**
2. **IDAS output-grid driving.**
3. **Callback allocation/copying and observer overhead.**
4. **Fitting/shooting derivative upgrades and adjoint replay.**
5. **Compiled evaluator and session-cache amortization.**

Each experiment should explain the changed mechanism as well as the changed wall time.

### Third: close the native gaps needed for method selection

Add PETSc event/KSP attribution, focused SUNDIALS profiling and precise native factor hooks where the first campaigns show uncertainty. Extend Uno’s structured observations if it remains a serious contender for the relevant workloads.

Then compare numerical configurations by family, using the same original-problem acceptance and requested products. Only after those results are repeatable would I encode stronger automatic selection policies.

**The most valuable outcome is a comparison that can say:** this formulation reduced factorization work but increased implicit derivative contraction; this derivative policy reduced optimizer iterations but added integration passes; this implementation reduced allocation cost with unchanged numerical work; or this method improved isolated latency while reducing concurrent throughput.

Your current architecture already contains much of the machinery needed to produce those explanations. The next investment should make that evidence complete, consistently scoped and easy to compare—then use it to decide which mathematical and implementation changes actually deserve to become the default.