---
title: Short pipeline case observations and interpretation limits
date: 2026-10-17
doc_role: evidence
doc_topics: [numerical-execution, diagnostics, performance]
---

# Short pipeline case observations and interpretation limits

These notes support the [principal review](../../reviews/design_review_pipeline-performance-optimization_2026-10-17.md).
They describe its bounded execution and library checks, not a separate design judgment or
implementation backlog. The initially clean source baseline was
`b8c535888075d84bbd9ef54767aacfb283601d5e`. Production source remained unchanged.
The [selected observations](case-observations.json) contain 37 process/case observations;
multiple invocations within a case remain separate records.

## Conditions and controls

**Measured:** 26 selected observations completed and eight passed. Two larger implicit
fixtures returned typed derivative-eligibility refusals. One two-worker study violated
an experimental preparation-count expectation after all four scientific results passed.
The immediate cancellation control intentionally produced no usable solution. The largest
selected whole-driver elapsed observation was 4.8064 seconds; these values include imports,
fixture construction and observation work. They are not comparative latency estimates or
statistical performance qualification.

The existing editable native extension was used after the Rust target directory had been
cleaned. `pse._build.compiled_features()` returned `compiled-evaluator`, `force-validate`,
`native-solvers` and `solver-highs`. No extension installation or Rust rebuild was needed.
Commands selected the linked native environment explicitly:

```bash
PSE_RESOURCE_CLASS=functional scripts/pse-env --native=solver,klu -- \
  .venv/bin/python docs/design_review/evidence/pipeline-performance-optimization-2026-10-17/short-cases.py \
  root /tmp/pipeline-root-example --detail
```

Use a fresh output directory. The [short driver](short-cases.py) accepts `root`, `edits`,
`sparse`, `dense`, `guard`, `mixed`, `qp`, `nlp`, `implicit`, `ode`, `dae`, `events`, `fit`,
`reference` and `cancel`, with backend, size, parameter-count and selected-detail options.
The [workflow driver](other-cases.py) covers conic, recycle, discrete, independent and
continuation studies, and result reads:

```bash
PSE_RESOURCE_CLASS=functional scripts/pse-env --native=solver,klu -- \
  .venv/bin/python docs/design_review/evidence/pipeline-performance-optimization-2026-10-17/other-cases.py \
  study-independent /tmp/pipeline-study-example --workers 2 --memory-gib 16
```

Each process owns a fresh ephemeral runtime and closes it in `finally`; these probes start
no canonical store or managed worker. Ordinary settings use two engine threads, a 16 GiB
logical memory pool, 4 GiB engine working cache and a fresh spill directory. The study
intervention changes only the declared pool to 64 GiB; this does not allocate that much
resident memory. Modeling retention has its separate 8 GiB policy. Peak process RSS in
the matched study observations was about 381 MB, and was not substituted for operation
charges or constructor allowances.

Scientific controls retain the ordinary engineering relative fraction **0.001**, original
model-domain checks and case-owned assertions. Final event localization also uses 0.001.
The unchanged 1e-8 rank threshold classifies rank; it is not a tightened solve tolerance.
Dynamic fixtures use short physical horizons, independently of wall time. Final fitting
controls hold the added fixed inputs at zero, preserving the same fitted value and response.
All observations use the actual native production APIs; Python does not implement iteration.

Where public handles exist, the short driver requests cancellation after a 60-second
observation interval and waits for real completion. No selected case approached that
interval. Synchronous strategy/study APIs expose no timer-cancellable public handle; the
workflow driver makes no hard-timeout claim. Neither driver force-kills native work.

## Selected observations and interpretation

The JSON records the exact raw directory of every selected observation, its status and
elapsed value, available phases, invocation documents and selected counters. Raw live
snapshots and full detail/profile files remain under their original `/tmp` directories;
they are disposable local outputs, not a provenance or environment-proof requirement.
The retained JSON and principal source analysis carry the published conclusions.

| Group and local output | Selected controls and useful facts |
| --- | --- |
| `/tmp/pse-pipeline-review-root-03` | Ordinary scalar cold, repeated, accepted-primal and equal-preparation invocations. Default KINSOL setup interval gives 16 function/2 Jacobian evaluations for the cold solve; accepted primal gives 2/1. |
| `/tmp/pse-pipeline-review-breadth-01/` | Four/eight-variable diagonal and coupled roots; mixed scales; guarded logarithm; QP; Ipopt/POUNCE NLP; immediate cancellation. These small controls passed their intended accepted/cancelled assertions. |
| `/tmp/pse-pipeline-review-edits-final` | Repeated/equal preparations, accepted seed, value overlay, value source edit and structural edit. Overlay candidate 3 is checked through its actual native result. |
| `/tmp/pse-pipeline-review-final-01/` | IDAS index-1 DAE, Diffsol fit with zero/four fixed inputs, IDAS fit with four fixed inputs, KINSOL setup interval 1. Its cold root uses 5 function/4 Jacobian evaluations; accepted start uses 1/0. |
| `/tmp/pse-pipeline-review-reference-01` | Sixteen actual reference packages: 119 files, 1,010,520 source bytes and 3,829 declarations. This is admission, not a thermodynamic-property numerical workload. |
| `/tmp/pse-pipeline-review-implicit-branch-01` | Eligible scalar positive-branch implicit solve; actual inner/factor/IFT stages observed. |
| `/tmp/pse-pipeline-review-controls-01/` | Four/eight-unknown implicit refusals, Diffsol/IDAS 5/25-output ODE controls, events, hub-off and ordinary hub-on roots. Both ODE backends retain 16 steps across the two output densities. Diagnostic selection preserves the cold root's 16/2 work counts. |
| `/tmp/pse-pipeline-other-20261017-*` | Clarabel conic, declared recycle, integer Count MILP, serial/two-worker independent and continuation studies, repeated projections, and 16/64 GiB study intervention. Exact suffixes are retained per case in JSON. |

The final matched Diffsol fit keeps 27 steps in the surviving forward experiment while
linear solves rise from 54 to 162 and initial callbacks from three to seven with four
unconsumed fixed-input sensitivity axes. These are experiment counters, not a whole-fit sum.
Complete selected `workflow.integration.v1` records contain eleven integrations per final
fit: five primal trials, four derivative upgrades, one value-only original assessment
and one forward identifiability integration. IDAS final fitting also completes with the
same split final purposes. Ordinary retained snapshots have explicit partiality; completed
experiment operations can still have a `started` producer because of the close/publication
ordering diagnosed in F08. Raw and retained observations must not be silently interchanged.

Four study roots remain scientifically usable in both pool interventions. Two-worker
16 GiB execution constructs four views and ten observation programs; 64 GiB constructs
two views and six programs, with two rebinds. Final modeling-cache entries are five versus
27, and reported bypass counts are zero. Eviction chronology, pinned bytes and in-flight
reuse are unavailable rather than zero. The larger-pool observation did **not** establish
a latency improvement; its measured study interval was longer. This is a work-count and
source-supported reuse finding. The experimental `views <= workers` assertion is not
promoted into an existing universal product contract.

### Selected CPU capture

The existing public launch-capture route ran the short reference admission case:

```bash
PSE_RESOURCE_CLASS=functional scripts/pse-env --native=solver,klu -- \
  .venv/bin/python -m pse.diagnostics capture-launch \
  --output /tmp/pse-pipeline-review-reference-cpu-01 --duration 10 -- \
  .venv/bin/python docs/design_review/evidence/pipeline-performance-optimization-2026-10-17/short-cases.py \
  reference /tmp/pse-pipeline-review-reference-01
```

The child exited normally. Capture v2 reported Stopped, no errors and 5.2312 seconds of
observed duration. The output stem is `314339-0000000000880442`; `.perf.data`,
`.perf.symbols.txt`, `.perf.profile.json.gz` and tool logs remain in that output directory.
The launch identity initially names the pre-exec shell; it must not be interpreted as a
separate scientific owner. The document has no related operation, unknown clock alignment,
unknown sampling loss, source partiality and 299 resolved/nine unresolved frames. Output
was not exhausted and no queue refusals were reported. These facts do not establish a
complete profile or an exclusive phase attribution.

Loader/PyArrow initialization, Arrow DataType cloning, Serde sequence/field handling and
allocation/free stacks supply leads for O06. A phase-aligned or narrow allocation probe is
needed before recommending an admission/serialization redesign. No timeline or allocation
capture was required to settle the present numerical-work findings.

## Pilot repairs and exclusions

Pilot setup mistakes were repaired rather than classified as production defects: a conic
identity needed the typed codec's unstructuring; the discrete model needed physical Count
rather than Scalar integers; a DAE needed its consistent algebraic start; and edited-result
checks needed the accepted native result rather than an unrelated export relation. Final
scientific assertions remained intact. Initial nonzero fixed-input fit fixtures are not
the final matched zero-input comparison used for F01.

The larger implicit fixtures were **not** repaired into an unsupported derivative claim:
their smooth-first capability refusals are retained as limits. They do not measure sparse
IFT verification growth. Likewise, the smaller logical-pool study's work expectation remains
an explicit violation, although its scientific answers passed.

No shooting, curvature, managed/durable execution, large result publication, global MINLP,
optional PETSc/Uno route, property-heavy numerical solve, long stiff trajectory or long
opaque cancellation latency was measured. Source inspection covered their relevant adjacent
contracts selectively. This review does not reproduce the historical long heater run,
establish a universal backend ranking, or qualify all Plan39/43 scientific scope.

## Library capability checks

The native-solver and Symbolica/faer capability skills routed exact-source inspection.
Reviewers used Context7 `/llnl/sundials` for selected forward sensitivities and native
quadrature-sensitivity capability. The [LLNL IDAS forward-sensitivity guide](https://github.com/llnl/sundials/blob/main/doc/idas/guide/source/Usage/FSA.rst)
documents `Ns`, selected parameters and supplied sensitivity callbacks. Applicability to
this checkout was checked against its pinned SUNDIALS 7.1.1 source, including
[`IDASensInit`](../../../../vendor/sundials-sys/vendor/src/idas/idas.c) and the
[host IDAS adapter](../../../../crates/pse-backend-native/src/dynamics/idas.rs).
An API capability is **Interface-checked**, not a qualified selected-axis implementation.

For O02, the same [forward-sensitivity guide](https://github.com/llnl/sundials/blob/main/doc/idas/guide/source/Usage/FSA.rst)
and [native forward-sensitivity example](https://github.com/llnl/sundials/blob/main/doc/idas/idas_ex_fwd.tex)
describe quadrature sensitivities. The pinned `IDAQuadSensInit`/`IDAGetQuadSens` source
requires active forward sensitivity analysis and initial quadrature-sensitivity vectors.
A null derivative callback selects native finite differences; an exact derivative route
needs its explicit callback. Scheduled restarts require correct reinitialization and
accumulated sensitivities. The [adjoint guide](https://github.com/llnl/sundials/blob/main/doc/idas/guide/source/Usage/ADJ.rst)
documents the retained alternative. These upstream links target main; pinned native source
settles applicability here. Adapter composition, event/reset support and benefit remain
unqualified.

The KLU diagnosis follows exact native initialization/setup in
[`kinsol.c`](../../../../vendor/sundials-sys/vendor/src/kinsol/kinsol.c),
[`kinsol_ls.c`](../../../../vendor/sundials-sys/vendor/src/kinsol/kinsol_ls.c) and
[`sunlinsol_klu.c`](../../../../vendor/sundials-sys/vendor/src/sunlinsol/klu/sunlinsol_klu.c).
It does not establish a supported replacement or measured sparse-setup benefit.
For F05, the faer 0.24.4 capability contract exposes
[`sparse_dense_matmul`](https://docs.rs/faer/0.24.4/faer/sparse/linalg/matmul/fn.sparse_dense_matmul.html);
deterministic sparse support traversal remains an alternative. The independently checked
physical backward-error rule stays authoritative whichever implementation is selected.
The exact Diffsol 0.16.2 host/source realization still needs investigation for selected
directions. Neither stale library-utilization records nor capability maps were consulted.

## Review-support verification

**Tested:** the following command passed **28 tests, zero failures**, against a zero-failure
baseline. It exercises the existing runner plus compact selection, finite deadline rejection,
cooperative stop, preserved child outcome and awaited cleanup; it does not execute the Rust
benchmark binary or qualify a numerical remedy.

```bash
scripts/pse-env -- python -m unittest \
  scripts.tests.test_case_audit \
  scripts.tests.test_validation.ValidationTests.test_k4_smoke_selectors_build_and_dispatch_only_the_preparation_target \
  scripts.tests.test_validation.ValidationTests.test_case_selection_preserves_default_and_refuses_unknown_identity
```

Pinned Ruff checked the changed runner/tests and both evidence drivers with
`--no-force-exclude`: zero findings. The explicit option is necessary because repository
defaults exclude `docs/`. Drivers retain narrow documented exemptions for scientific
assertions and their standalone console/import roles. Readability cleanup preserved 60
exact generated-source comparisons across the short driver's supported scientific shapes;
no scientific settings changed during that cleanup.

The runner now admits compact declared selectors, including the 12-parameter sparse-fit
rank-refusal control, and accepts `--audit-deadline-seconds` only with explicit diagnostic
observation. That optional interval requests stop; it neither overrides the scientific
outcome nor establishes cleanup. The public launch-profiler remains separate: the runner's
environment/output routing and forceful launch-handle cancellation do not compose safely
with cooperative fixture ownership merely by substitution.

**Tested:** scoped documentation validation passed for the four changed Markdown pages:
strict metadata parsing, 73 existing local source targets, three explicitly delivered
evidence assets, actual mdBook rendering, local rendered anchors and Pagefind indexing.
The scoped site is `/tmp/pse-pipeline-review-docs-t6emteee/html`. `git diff --check`
passed. `just adr-index` and `just fmt` completed successfully; index regeneration
introduced no index change.

The full `just docs` command failed with one publication error against the zero-error
baseline: unchanged `docs/external_review_numerical_diagnostics_1.md` lacks a scalar title
or first H1. The `just turn-end` and diagnostic `just ready` wrappers each failed before
their checks because the existing resource registry requires explicit fenced migration
recovery. The separate `adr-index` and `fmt` constituents above completed without changing
that registry. Neither unrelated condition was repaired or bypassed by resetting shared
state. These failures do not turn the scoped documentation check into a full-site pass.

No full native rebuild, broad Clippy/hygiene campaign, environment/provenance proof or
production optimization was performed for this documentation/tooling review.
