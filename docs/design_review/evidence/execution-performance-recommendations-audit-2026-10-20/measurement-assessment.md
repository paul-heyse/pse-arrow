# Measurement assessment for the execution-performance recommendation audit

Supporting analysis for the [principal audit](../../reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md).
Inspected implementation: `1ee81798a9b884f3054b685ee9629e2c8d124498`.
This document interprets existing observations; it does not establish an implemented
speedup or create a second disposition owner.

## Sources actually inspected

- The original campaign's selection, selected-case summary, original invocation/native
  JSONL records, paired summary, publication index and failure notes under
  `build/external-pipeline-review-20261019/`.
- R2's measurement document and the retained driver, debugger sampler, GDB helper,
  five raw stack-sample files, seven root replay records and registry breakpoint
  output under `build/claude-review-scratch/`.
- The production diagnostic default, native control mappings and the source mechanisms
  cited in the principal review.

[Retained-observation extraction](retained-observations.json) records the source paths,
JSON pointers and selected values. It was recomputed from existing files during this
audit. No simulation, compiler, debugger, profiler or service was launched again.
Reported rerun wall times for which only the review's description remains are not
relabeled independently measured by this audit.

R1 explicitly lacked the raw campaign bundle. Its source-established findings remain
valuable, but its campaign consequences were briefing evidence. R2 had local outputs
and performed additional runs. Neither evidence base makes a proposed remedy an
implemented or measured improvement.

## Corrections to mode and attribution

### Unset diagnostics is not disabled diagnostics

R2's driver copies a retained condition and removes `PSE_DIAGNOSTICS` when it is null.
`pse-runtime::observation::process_diagnostics` enables ordinary collection unless the
value is exactly `off`. Thus the described environment does not establish diagnostics-off
runs: it is consistent with ordinary observation enabled, without selected source detail,
optional capture or review export. The driver also does not itself sanitize every
inherited diagnostic-request variable.

This corrects experimental labeling and the interpretation of callback/publisher cost;
it does not erase the observed registry or admission work. The original controlled
observer cohort explicitly selected modes before import and remains the appropriate
off/ordinary/detail comparison. Future reruns need only record the actual effective
mode and explicitly select it, not introduce environment attestation machinery.

### Debugger stacks are not scheduler-qualified CPU samples

The LLDB helper periodically stops the process and records every thread's stack.
For example, the small banded-root file contains 476 thread-stack rows across 187 stop
timestamps; the large root has 3,003 rows across 611 timestamps. Parked threads are
present in the raw files. Filtering recognizable blocked top frames is useful, but
does not determine which remaining threads were actually running on a CPU.

Debugger stops and stack walking also perturb different phases by different amounts.
The measurement report acknowledges roughly 10–30× wall-time inflation. Its percentages
are best treated as conditional stack-occupancy evidence supporting source investigation,
not precise CPU shares robustly transferable to the uninstrumented run. Registry and
modeling frames are strongly represented; exact percentages and derived savings remain
unqualified.

The perf driver selects `cpu-clock:u`, not a hardware cycle counter. Consequently
the reported libc percentage is a self share of sampled userspace CPU-clock events,
not a measured percentage of hardware cycles or time attributable solely to allocation.
Copying, comparison and allocation/freeing are different operations. The unusable
DWARF call chains limit causal attribution. The interrupted Callgrind run describes
its captured prefix, not the instruction distribution of a complete normal execution.

The GDB output does independently contain `ASSEMBLE_SECONDS 0.937`. This is one
finish-breakpoint observation of registry assembly, rather than a cold/warm distribution
or a demonstrated saving from a replacement.

### Whole-closure minus invocation is not a preparation span

The retained plain root records show about 1.697 seconds for the small harness closure
versus 0.0111 seconds for its invocation, and 53.543 versus 0.8021 seconds at 256
unknowns. This convincingly separates the invocation from the much larger enclosing
lifecycle. The difference includes work before and after invocation, including registry,
source admission, runtime work and teardown. It is not by itself exclusive preparation
time, and native interval sums require overlap/identity checks.

Source inspection independently establishes repeated broad lexical work. The empirical
exponent depends on a fitted fixed baseline and a few sizes; it is not a universal
complexity proof for every package. Extrapolating a 1,000-variable duration or demanding
an exponent at most 1.2 is unnecessary for the architectural judgment.

## Accepted starts already show distinct native effects

The parent harness contains a cold solve followed by a seeded solve, but its invocation
records preserve them separately. It is incorrect to say this design prevents observing
native-work differences. All ten accepted-start configurations were decoded; representative
records are:

| Configuration | Cold native work | Seeded native work |
| --- | --- | --- |
| Ipopt, 2 and 16 variables, exact | 4 iterations, 33 evaluations | 0 iterations, 13 evaluations |
| Ipopt, 2 and 16 variables, limited memory | 4 iterations, 29 evaluations | 0 iterations, 13 evaluations |
| POUNCE, 2 and 16 variables, exact | 6 factors, 36 evaluations | 1 factor, 16 evaluations |
| POUNCE, 2 and 16 variables, limited memory | 6 factors, 37 evaluations | 7 factors, 39 evaluations |
| Uno, 2 and 16 variables, limited memory | 5 iterations, 31 evaluations | 1 iteration, 15 evaluations |

POUNCE iteration counts are null in these records; factorization counts are not renamed
iterations. Uno's selected fixtures still fail their receipt assertions. Their numerical
improvement neither fixes the false flags nor justifies calling the entire fixture passed.

The seeds are produced at the same problem's solution. These observations refute a
blanket discarded-primal/nearly-cold diagnosis, but do not establish a universal
2–5× benefit after parameter perturbations, changed active sets or new scaling. Both
positive and adverse cases must remain visible when selecting restart policy.

## Consequences for priorities and acceptance

Preparation is a major opportunity for the measured fresh-process/package cases.
That does not establish that numerical methods are universally the third lever for
warm edit/re-solve, reused studies, implicit actions, stiff trajectories or coupled
systems. Use separate cold-first-result, warm-edit, accepted-throughput and requested
result-availability views.

The study-width examples do not isolate the coordinator as their cause: cheap science,
fixed setup and per-member admission can mask concurrency. Reservations of 13–22 GB
versus roughly 0.29 GB parent RSS identify an admission question; they do not establish
an invalid charge or a safe smaller bound. Native storage, stacks, uncommitted products
and foreign allocations have different lifetimes from observed resident pages.

R2's millisecond, callback, exponent, iteration-ratio and 0.8-times-core targets remain
possible experiment aims. They are not requirements derived from the functional target.
An accepted optimization needs an identified work reduction, preserved scientific and
lifecycle contracts, and a representative complete-operation comparison when a speed
claim or policy selection depends on timing. Five observations give descriptive ranges,
not stable tail latency or automatic statistical significance.

No full campaign rerun was warranted for this audit: source and retained records settled
the corrections above. The principal review names the bounded experiments needed to
choose unresolved methods, resets, defaults and layouts.
