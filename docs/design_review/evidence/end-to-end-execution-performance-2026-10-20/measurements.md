---
title: End-to-end execution performance — measurements and attribution
doc_role: evidence
doc_topics: [design, performance, preparation, numerical-execution]
doc_owner: docs/design_review/reviews/design_review_end-to-end-execution-performance-target_2026-10-20.md
---

# End-to-end execution performance — measurements and attribution

This document supports the
[end-to-end execution performance target review](../../reviews/design_review_end-to-end-execution-performance-target_2026-10-20.md).
It records what was re-executed, with which tools and under which conditions. It also records
the attribution that the review relies on. The principal review owns the judgment. This file
owns only the observations.

## 1. Baseline and conditions

- **Source:** `main` at `1ee81798a`. This is the campaign baseline `106aef5ec` plus one
  documentation-only commit. The working tree was clean apart from the untracked external
  review `docs/design_review_execution-performance-target_2026-10-20.md`.
- **Binaries:** the campaign's own profiling-profile harnesses, reused without rebuilding:
  - `target/profiling/build/pse-benches/9108121f6a2eff95/out/native_process-9108121f6a2eff95`
    (built 2026-10-19 21:13, 4.2 GB with full debug info);
  - `…/98e0cafb7f5a0549/out/modeling_preparation-…`.
  - The profiling profile inherits release (`opt-level` 2 workspace / 3 dependencies, thin LTO).
    These harnesses carry explicit Arrow `force_validate` and the compiled-evaluator capability,
    as the campaign labels them.
- **Host:** AMD Ryzen 9 9950X3D (16 cores / 32 threads) with 188 GB RAM, on the maintainer's
  workstation.
  - The host was not quiesced. Load average was 3.8–5.7 from unrelated work. The single-threaded
    attributions below are robust to that load; small wall-clock differences are not.
- **Case inputs:** each run reused the retained `case-run.json` from
  `build/external-pipeline-review-20261019/`. Specifically:
  - the exact `workload` (optionally with a declared override), passed through
    `PSE_PROCESS_COST_SPEC`;
  - the campaign thread pins (`OMP/MKL/OPENBLAS_NUM_THREADS=1`, `MKL_CBWR=COMPATIBLE`, …);
  - `PSE_DIAGNOSTICS` unset;
  - **no recording request, no capture, no review export.**
  - Runs used `scripts/pse-env --native -- …`.
- **Store:** only ephemeral (non-canonical) cases were re-run. `cold-small-1` and the four
  preparation cases need a canonical fixture.
  - A private SurrealDB state was set up for them. Its test-resource registry then refused with
    "requires explicit fenced migration recovery". Recovery of the maintainer's resource registry
    was not attempted.
  - Canonical cases are therefore represented only by the campaign's own timelines.
  - The private service was stopped afterwards.
- **Repetition:** one observation per configuration unless stated. These are descriptive single
  observations, not a statistical comparison.

## 2. Tools and their limits

| Tool | Use | Limits |
|---|---|---|
| Plain re-run | Process wall time and the harness's own `*-observations.jsonl` (workflow invocation intervals, strategy ledger, compiler phases) | Single observation; harness `seconds` covers the whole closure |
| `gdb` finish breakpoint on a symbol-only copy (`objcopy --strip-debug`, same build-id) | Wall time of `pse_schema::catalog::assemble` | Measures one call; breakpoint overhead is negligible relative to 0.9 s |
| `lldb` stop-the-world stack sampler (SB API; interval 10 ms, or 100 ms for long runs) | Inclusive attribution of on-CPU stacks: 188–842 on-CPU samples per case | Stopping perturbs wall time (runs take 10–30× longer under the sampler). The proportion of on-CPU samples still estimates CPU distribution. Threads parked in futex/epoll/syscall are excluded. |
| `valgrind --tool=callgrind` | Instruction attribution | Crashed inside the diagnostics publisher thread (`rustix` vDSO initialisation under valgrind) after the registry build; the 14.3 G instructions it captured are almost all `RegistryBuilder::build` |
| `perf record --call-graph dwarf` | DSO-level self time | DWARF unwinding stops after one or two frames on this binary, so call stacks are unusable. Self-time-by-DSO and leaf symbols remain valid. |

Under `perf_event_paranoid=2`, `samply` refuses to run. That is why the lldb sampler was used.

## 3. Small cases: where a 1.7 s process spends its time

Plain wall time, no diagnostics:

| Case | Plain process wall (s) | Campaign harness `seconds` (cohort) |
|---|---:|---:|
| `compact-root-banded-4` | 1.72 | 3.86 (pilots-repaired) |
| `compact-idas-ode-5` | 1.71 | 4.55 (pilots-repaired) |
| `compact-recycle-coupled` | 1.80 | 4.61 (breadth) |
| `review-nlp-ipopt-2-cold-exact` | 1.77 | 3.64 (breadth) |
| `review-nlp-ipopt-16-accepted-exact` | 1.76 | 5.03 (breadth) |
| `review-pounce-convex-1-1w` | 1.79 | 1.58 (breadth-final) |
| `review-study-4w-16p-compact-independent-nonlinear` | 1.83 | 2.22 (breadth-final) |
| `compact-implicit-derivatives-8-2` (direct math) | 0.93 | 1.03 (breadth-final) |
| `review-pounce-convex-8-1w` | 6.12 | 5.58 (breadth-final) |
| `compact-reference` | 6.00 | 8.18 (breadth) |
| `compact-property-value` | 6.98 | 8.38 (breadth) |

**How the campaign column was recorded.** Every campaign case used `selected` recording with a
review export, and some also captured CPU and timeline from the start. The breadth and pilot
cohorts partly overlapped compilation and other host work.

**How to read the comparison.**
- The plain/campaign comparison is indicative only. The breadth-final cohort, which ran
  after compilation, is close to the plain figures.
- It does not establish a diagnostics overhead. The controlled observer comparison in §6 does
  that.

Within `compact-root-banded-4`, the workflow invocation is the staged modeling execution plus
original assessment. It took **11.1 ms** end to end, including six native worker executions
totalling about 1.4 ms. `pse_schema::catalog::assemble` took **0.937 s** (gdb).

**CPU attribution (lldb samples).** Each on-CPU sample is assigned to the first matching
category in this order: registry → modeling compiler → authored document admission → canonical
persist → math → native → workflow.

| Case | On-CPU samples | Registry assembly | Authored document admission | Modeling compiler (`pse_modeling::check`/Salsa) | Math + native solver | Other |
|---|---:|---:|---:|---:|---:|---:|
| `compact-root-banded-4` | 188 | 53% | 41% | 1% | 2% | 3% |
| `review-nlp-ipopt-2-cold-exact` | 223 | 52% | 38% | 1% | <1% | 8% |
| `review-pounce-convex-8-1w` | 732 | 14% | 79% | 1% | 3% | 3% |
| `compact-property-value` | 842 | 12% | 33% | 31% | 2% | 22% (modeling-revision job, mostly `selected_source` resolution) |
| `review-root-banded-256` | 611 | 2% | 1% | 94% | <1% | 3% |

**Leading inclusive frames.**

- **Registry:**
  - `fingerprint::semantic_product` 36%
  - `arrow::bind_field` / `bind_type` 29%
  - `native_field::canonical_json` 22%
  - `builder::native::BOOTSTRAP` / `materialize` 16–17%
- **Document admission:**
  - `from_documents_with_receipt`
  - `pse_relations::validate::predicates::field_local` (15–24%)
  - `validate::field::validate_field`
  - `pse_engine::session::admission::{restore_semantic_fields, admit_field, admit_expression}`
  - DataFusion `TreeNode` / `create_physical_expr` / `replace_children_if_necessary`
  - `arrow_select::filter::filter_record_batch`
- **Modeling compiler:**
  - `expression::check_declaration` → `declaration_environment` → `CheckedPackage::named_types` → `resolve_with_types` (60% of all samples at N=256)
  - `phases::signature` → `resolve` → `named_types_with` (35% of compiler samples)

**perf self time by DSO** (`compact-root-banded-4`):

- `libc.so.6` 53.8% (malloc, `_int_free`, `_int_malloc`, memmove, memcmp, `malloc_consolidate`)
- main binary 45.9%
- `ld-linux` 0.27%
- MKL 0.03%

Dynamic loading of the 4.2 GB binary is therefore not material.

**`review-pounce-convex-8-1w`.** Eight members take 6.1 s against 1.8 s for one member. The extra
≈0.6 s per member ((6.12 − 1.79)/7) is authored-package admission (79% of samples). The POUNCE-convex solves
themselves are about 2% of samples. Admitted native worker count cannot change that serial
admission share.

## 4. Size growth: banded root sweep

The workload is `review-root-banded-256` with `blocks` overridden. The authored rows form an
analytic cubic-plus-linear dependency chain (`x_i = i+1`), solved with KINSOL. "Invocation" is
the harness's own workflow-invocation terminal time.

| Unknowns N | Plain wall (s) | Invocation (s) | of which native worker execution (s, count) | Strategy iterations / evaluations | Outcome |
|---:|---:|---:|---:|---:|---|
| 4 | 1.72 | 0.011 | 0.0014 (6) | — | pass |
| 16 | 2.95 | 0.109 | 0.0042 (18) | 344 / 1,822 | pass |
| 32 | 2.25 | 0.072 | 0.0058 (34) | 424 / 2,297 | pass |
| 64 | 3.30 | 0.103 | 0.0120 (63) | 425 / 2,321 | **fail**: `KIN_STEP_LT_STPTOL` → `Limit`, candidate unqualified |
| 128 | 8.67 | 0.235 | 0.0233 (63) | 464 / 2,539 | pass |
| 192 | 24.35 | 0.377 | 0.0425 (63) | 477 / 2,534 | pass |
| 256 | 53.68 | 0.802 | 0.0675 (63) | 507 / 2,798 | pass (campaign: interrupted at the 60 s budget with diagnostics on) |

**Fixed baseline.** Above a ≈2.1 s fixed baseline, local exponents are:

| Interval | Time above baseline | Exponent |
|---|---|---:|
| 64 → 128 | 1.2 s → 6.6 s | ≈2.5 |
| 128 → 192 | 6.6 s → 22.3 s | ≈3.0 |
| 192 → 256 | 22.3 s → 51.6 s | ≈2.9 |

The exponents rest on these intervals and the assumed baseline. The source establishes the
cubic structure (`named_types_with`).

At N=256, 94% of on-CPU samples are in `pse_modeling::check::check_with_phases`. The
compiler-phase totals recorded by the harness (`pse.case.*`) stay at 0.03–0.12 s, so the
growth is in modeling admission, not in body/order construction.

**Inside the invocation.** Non-native orchestration (invocation minus native) is 0.105, 0.066,
0.091, 0.212, 0.335 and 0.735 s for N = 16, 32, 64, 128, 192 and 256.

- Up to N = 192 it grows about N^1.1–1.2. From 192 to 256 it grows ≈N^2.7.
- Native time grows roughly linearly (4 → 67 ms) and is 4–12% of the invocation.

**The failing N=64 candidate.**

- The harness prints at most eight component-local coordinates:
  `[1.999999998, 38.0, 37.0, 17.0000041, 64.0, 51.0, 43.0, 35.0]`.
- These are analytic `i+1` values in permuted order; the worst is within 4.1e-6.
- The candidate was refused with `infeasible; unqualified; native_outcome`.
- The empty "failed original checks: []" list does not establish that the original checks
  passed. It is a filtered view, and it is also empty when no original checks ran on an
  unqualified candidate.
- The full candidate's residual is not reported.

**Evaluations per iteration.** The ledger shows about 5.3–5.5 residual evaluations per
iteration and 8–19 iterations per native execution, from the 0.5 start. Ledger evaluations
may include fallback and assessment evaluations, so this is a lead, not a diagnosis.

## 4b. Long campaign cases: invocation versus harness time

This section reads the campaign's own `*-observations.jsonl` (selected recording). Workflow
invocation endpoints are measured from invocation establishment to response-ready. Native time
is the sum of `native.*` intervals inside the invocation.

| Case | Harness `seconds` | Invocation `response_ready_seconds` | Native inside |
|---|---:|---:|---:|
| `fit-curved-second-cold-1` | 59.32 | 5.13 | 1.33 |
| `mixed-scale-1-1` | 59.47 | 6.44 | <0.01 |
| `qp-1-1` | 50.80 | 3.34 | <0.01 |
| `shooting-single-idas-cold-1` | 40.38 | 3.13 | 0.21 |
| `sparse-fit-12-1` | 35.30 | 2.38 | 0.02 |
| `review-sparse-fit-2-1` | 12.73 | 1.26 | 0.01 |
| `review-sparse-fit-8-1` | 11.90 | 1.15 | 0.01 |

## 5. Study width

The workload is `review-study-4w-16p-compact-independent-nonlinear`. The harness admits only
`4|16` points and `1|2|4` workers.

| Points | Workers/threads | Plain wall (s) |
|---:|---:|---:|
| 16 | 1 | 2.02 |
| 16 | 4 | 2.31 |

The campaign's ordinary-mode paired pipeline (five fresh processes per side) is consistent:

- four-point Compact, 1 vs 4 workers: median 1.68 vs 1.62 s;
- eight POUNCE-convex members, 1 vs 4 workers: 5.55 vs 5.62 s.

`pool_peak_bytes` was 13.3–22.3 GB (accounted reservation) while process peak RSS was
0.28–0.29 GB.

## 6. Campaign observations reused (not re-executed)

| Source | Observation used |
|---|---|
| `paired-observer-repaired/paired-summary.json` | Python KINSOL/IDAS/POUNCE probes, off vs ordinary vs selected detail. `prepare_start_join_wall` stays 0.26–0.28 s in every mode. `child_before_runtime_close_wall` is 1.27/1.28/1.30 s off vs 1.40/1.44/1.55 s ordinary. The difference is post-run inspection (45–72 ms), export (59–115 ms) and analysis (12–33 ms), not science. |
| `breadth/cold-small-1` timeline | `audit.source_admission` 58.2 s, `runtime_admission` 1.15 s; one 44.4 s `modeling.prepare`; 52,822 queued timeline packets dropped; about 2,500 `log event` instants per second |
| `preparation-final/{pr-inline,pr-nested,pcsaft-02}` timelines | `modeling.prepare` 36.6 / 40.0 / 41.1 s. Inside it, `math.job`/`compiler.publish_modeling` covers about 2 s, plus never-closed `compiler.query.{typed,resolved,complete,checked_expressions}` spans. Cold/structural/value/warm preparation after admission: 1.65 / 0.019 / 0.019 / 0.019 s (PR inline). |
| `preparation-final/k4-scalar-preparation` timeline | Whole cold/warm/value preparation in tens of milliseconds |
| `managed-repaired/k4-study-managed-durable-16` timeline | 8.3 s fixture/service start, 0.8 s source admission, 19.3 s dispatch/execution/results for 16 trivial points: one 14.0 s activity, 92 result reopen/access spans totalling 3.0 s, native worker time ≈7 ms. Then a 2.0 s receiver drain. |
| `REVIEW.md` matrix | Interrupted at the 60 s budget: `cold-small-1`, `warm-small-1`, `specialization-medium-1`, `fit-gradient-cold-1`, `conic-1-1`, `dynamic-rebind-4-1`, `evented-fit-1-1`, `p44-results-admitted-retained-80-25`, `review-root-banded-256`. Long completions: `fit-curved-second-cold-1` 59 s, `mixed-scale-1-1` 59 s, `qp-1-1` 51 s, `shooting-single-idas-cold-1` 40 s, `sparse-fit-12-1` 35 s. |

**Trace reliability.** The open timeline spans cannot be read as exclusive time:

- `Node::update` drops updates on `try_lock` contention;
- the detail ring holds 256 events and open phases are capped at 128;
- lowering/projection/specialization carry no phases.

The CPU attribution in §3–§4 does not depend on those spans.

## 7. Reproduction

The probes used a small driver. It reads a retained `case-run.json` and re-executes its exact
command, workload and conditions. It optionally applies `RUNCASE_OVERRIDE` (JSON merged into
the workload) and wraps the command with `gdb`, `lldb` (stop-sampling script), `valgrind` or
`perf`. The driver and sampler lived in the ignored scratch directory
`build/claude-review-scratch/` and are not part of the repository.

The sampler works as follows:

- launch through `SBTarget.Launch` in async mode;
- every interval, `SBProcess.Stop()`, record every thread's frames (function name, depth ≤ 120),
  then `Continue()`;
- run on a symbol-only copy of the binary so lldb loads quickly.

**Equivalent ad hoc commands:**

```bash
# Plain run of one retained case
PSE_PROCESS_COST_CASE=<id> PSE_PROCESS_COST_OUTPUT=<dir> \
PSE_PROCESS_COST_SPEC="$(jq -c .workload <bundle>/<cohort>/<id>/case-run.json)" \
scripts/pse-env --native -- <campaign native_process binary> --test

# Registry assembly wall time
objcopy --strip-debug <binary> <copy>
gdb -batch -ex 'python ... FinishBreakpoint on pse_schema::catalog::assemble ...' -ex run --args <copy> --test
```
