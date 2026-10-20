# Context for the external pipeline performance review

This briefing accompanies the diagnostic campaign in
`build/external-pipeline-review-20261019/`, collected on 19–20 October 2026.
It explains what the observations represent and connects them to their production
owners. It is context for an independent review, not a diagnosis to accept or an
implementation specification. Paths below are relative to the repository root.

The bundle is generated material under ignored `build/`; an ordinary Git checkout
does not supply the measurements. The accompanying archive is
`build/external-pipeline-review-20261019.tar.gz`. Its internal paths remain usable
after extraction elsewhere; recorded original command/source paths are descriptive.

The review concerns end-to-end problem execution: avoiding unnecessary preparation
and evaluation, choosing effective formulations and numerical methods, reusing the
right products, exploiting parallelism, coordinating work, and making results
available efficiently. Diagnostics are valuable both as evidence for these questions
and as production functionality whose own cost and usefulness matter.

## Product and design context

`pse-arrow` is a Rust process-systems engineering core with a Python boundary. Typed
authored process definitions are the model authority. Symbolica/Numerica own the
mathematical representation and evaluators; class-specific native adapters execute
numerical methods. Arrow, DataFusion and canonical storage serve admission,
inspection, retained results and publication. Those data systems are not the numerical
evaluation engine.

The functional target includes interactive value edit → re-solve without rebuilding
structure; sweeps, sensitivities, optimization and fitting; recycles and tears;
dynamics from the same model definitions; and extension with new units, properties
and solvers. Small isolated mathematical fixtures are useful for separating mechanisms,
but are not representative industrial flowsheets by themselves.

The [architecture reading guide](authoritative_design/sections/reading-guide.md)
routes to the current contracts. Particularly relevant owners are:

- [Mathematics and compilation](authoritative_design/sections/mathematics-and-compilation.md),
  including preparation, demand, derivative products and reuse.
- [Numerical execution](authoritative_design/sections/numerical-execution.md), including
  structure, scaling, initialization, routing and original-space qualification.
- [Workflows and results](authoritative_design/sections/workflows-and-results.md), including
  studies, dynamics, fitting, retention and result access.
- [Operations and validation](authoritative_design/sections/operations-and-validation.md)
  and the [diagnostic operating guide](dev/simulation-diagnostics.md), including the
  distinction between live observations, frozen evidence, analysis and publication.

The applicable [design principles](design_review/design_principles/core/design-principles.md)
and [efficiency heuristics](design_review/design_principles/core/efficient-architecture-heuristics.md)
matter alongside raw speed: change locality, explicit ownership, truthful contracts,
scientific validity and the ability to compile/test a responsibility with its actual
dependencies remain part of the target. Historical plans explain the recent work, but
their completion labels do not establish that every implementation choice is optimal.

The current practical engineering accuracy is **0.001**. The campaign preserved it.
Derivative, rank and other mechanism controls retain their own appropriate assertions;
0.001 is not a universal replacement for every test tolerance. Improving speed by
weakening scientific acceptance would answer a different question. This remains a
design-phase, personal Linux project: there is no established timing SLA or need for
an environment/provenance attestation system.

## What the campaign contains

The selection combines 57 existing short configurations, 42 additional small
configurations and four preparation cases. It favors breadth of mechanisms over
large problem sizes or exhaustive Cartesian combinations.

| Population | Observed outcome | Meaning |
| --- | --- | --- |
| 99 science configurations | 85 completed, five failed case assertions, nine interrupted | Selected final attempts; completion is separate from diagnostic coverage |
| Four preparation configurations | Four completed | PR inline/nested, PC-SAFT and a small scalar preparation path |
| 36 Rust and five Python controls | All passed | Focused assertions about mechanisms and boundaries; not 41 additional industrial simulations |
| 45 observer comparisons | All completed | KINSOL, POUNCE and IDAS × off/ordinary/selected-detail × five fresh processes |
| 40 pipeline comparisons | All completed | Four matched contrasts × two sides × five fresh processes |
| 139 indexed packs | Offline readback succeeded; 686 available Arrow IPC streams decoded | Reader/publication checks, not proof of complete recording or scientific success |

All eleven adapters have actual returned execution or segment evidence: Ipopt,
POUNCE, KINSOL, HiGHS, Clarabel, Diffsol, IDAS, SCIP, POUNCE-convex, Uno and PETSc.
This establishes exercised paths, not equivalent coverage, a ranking or whole-library
qualification. Seven larger original configurations were deliberately excluded;
`campaign-setup/selection.json` records their exact identities.

### Case families and what they expose

| Family | Main distinctions represented |
| --- | --- |
| Authoring/preparation | Reference-package admission, inline versus nested definitions, cold/warm/value preparation and source reuse |
| Nonlinear optimization | Small sparse NLPs at two sizes; Ipopt/POUNCE exact versus limited-memory settings; cold and accepted starts; Uno's supported start role |
| Roots and structure | Diagonal, triangular banded and coupled cases; decomposition, fallback, KINSOL/PETSc paths; separate larger coupled native controls |
| Linear, mixed-integer and convex problems | LP/MILP/MINLP, coefficient preparation, POUNCE-convex batches, SOC and PSD cones |
| Dynamics and responses | Diffsol/IDAS ODE and index-1 DAE paths, smooth/stiff cases, scheduled/unscheduled sampling, simultaneous formulations, forward/adjoint and linear-solver controls |
| Derivatives and providers | Implicit solves/products, first/second-order demand, compiled actions, provider selection/effects and reuse |
| Studies, recycles and fitting | Full/Compact retention, independent/continuation policies, worker widths, coupled/stagnating recycles, sparse fitting and shooting |
| Durable execution and results | Managed dispatch/receivers, retained result selection, publication chains, Arrow/Python access and reopen behavior |

The additional NLP fixtures have independently checked analytic answers. The banded
root fixture is a dependency chain, not an all-to-all sparse-linear stress problem.
Configured dimensions, workers and methods are inputs; actual decomposition, native
work and active concurrency must come from the returned observations. Mechanism-test
fixtures and their source assertions are described in
`campaign-setup/supplement-selection.json`.

## How the files relate to an execution

The bundle has several views of the same work. They are not independent samples.

| Artifact, relative to the bundle | Role |
| --- | --- |
| `README.md` | Compact delivery overview and interpretation boundaries |
| `REVIEW.md` | Detailed selected-case matrix, native witnesses, controls, original evidence and coverage notices |
| `qualification/review-guide-summary.json` | Machine-readable selection and coverage summary |
| `campaign-setup/selection.json`, catalogues and supplement selection | Intended populations and configurations; declarations alone do not establish execution |
| Each `case-run.json` | Actual command, workload, conditions and collection/result status for that attempt |
| `process.log`, `*-observations.jsonl`, native reports and memory documents | Original process output, invocation/native facts and measured observations |
| `diagnostics/` | Recording requests/ACKs, checkpoints, inspections, detail, captures, frozen inputs, analysis requests and pack index |
| `review-results/` where present | Scientific result exports made while the owner/fixture was alive |
| Each review pack's `index.json` | Exact indexed members, formats, availability, context and publication state |
| `index.html`, `index.json`, `index.md`, `observations.csv` | Aggregate navigation and scalar extraction across retained attempts |
| `paired-observer-repaired/paired-summary.*` | Completed observer population and combined comparisons, referencing the original 40 pipeline trials in place |
| `qualification/` | Outcome notes, publication/readback checks, control records and scope-end check results |

The aggregate index retains 169 case receipts, including original and repaired
attempts and the 40 pipeline contrast trials. It is not a population of 169 distinct
scientific models. `REVIEW.md` selects final cohorts explicitly: a later failed
attempt does not disappear behind an earlier passing one. Initial observer children
that failed before importing `pse` remain historical setup failures and are excluded
from the completed 45-observation population.

The CSV is a large scalar extraction with source paths, JSON pointers and JSONL line
numbers. It includes configuration, identity, status and scientific values as well as
performance measures; not every number is a metric. IDs, timestamps, dimensions and
duplicate projections must not become performance samples. The paired summaries
apply a narrower metric allowlist and preserve original observations.

There are distinct evidence levels within one case:

1. The requested source selection and actual producer acknowledgement describe what
   recording was admitted, not what subsequently happened.
2. Source records, native reports and invocation/context documents describe observed
   work. Context joins preserve original identities; they are not reconstructed
   workflow executions.
3. Frozen evidence retains a supplied snapshot. Analysis is derived from that supplied
   scope; it does not silently perform another solve.
4. Arrow tables and exported packs are projections/publications of those facts.
   Several projections can describe one event.

Pack sealing, successful offline reading, scientific usability, source completeness,
native termination and cleanup are independent properties. Direct math and
preparation-only cases can legitimately lack a workflow `RunResult` or invocation
bridge. Ephemeral results do not acquire canonical protected exports merely because
diagnostics exist. These absences are stated rather than filled with synthetic context.

## Conditions that affect interpretation

The scientific harness used the existing release-derived **profiling** build with
symbols and explicit Arrow correctness validation. The Python extension and managed
worker used the fast editable **dev correctness** build and retain separate labels.
This is not a uniform release-build throughput benchmark. Cargo compilation was
setup rather than the intended science timer, but some initial breadth executions
overlapped compilation; other host workloads were not controlled.

Each breadth case normally supplied one original harness observation. A static
Criterion description mentioning ten samples does not mean ten measurements occurred
under the actual `--test` invocation. The repeated comparisons instead use five fresh
processes per condition, alternate order and retain their actual values/work checks.
They support descriptive medians/ranges, not a demonstrated p-value or solver winner.
Fresh processes also do not establish cold filesystem, compiler or store caches.

The four matched pipeline contrasts are Full versus Compact at four points/one worker,
Full versus Compact at sixteen points/four workers, one versus four workers for the
same four-point Compact study, and one versus four workers for eight POUNCE-convex
members. They use ordinary audit recording without optional source-detail/profiler
capture or review export. Observer comparisons use separate small probes; their
overhead is not automatically the overhead of every full workflow.

The nominal 60-second limit requests cooperative scientific stop, followed by actual
drain. Parent elapsed can exceed it through initialization, polling, publication and
teardown. A canonical-backed test often creates an isolated database/schema fixture;
that cost does not establish that production reinitializes schema for every solve.
The roughly 70-second Python lifecycle control includes its fixture and reopen path.

| Observation | Important scope distinction |
| --- | --- |
| Parent, phase, strategy and native wall times | Owners overlap; summing inclusive clocks creates fictitious exclusive time. Parallel work totals differ from critical-path latency. |
| Iterations, evaluations, factors and linear work | Native definitions and availability vary. Segment resets, attempts, components and fallbacks keep separate identities; unknown is not zero. |
| Preparation/admission/construction/reuse counts | Semantic reuse and allocation identity are different. A changed allocation ID alone does not prove structural rebuilding. |
| Callback/provider events and detail | A bounded selection can omit or sample records. Healthy execution need not trigger a callback-failure family. |
| Process peak RSS and retained/pool bytes | Process lifetime, accounted pool memory and retained object lifetimes are different scopes; none alone measures operation-private peak memory. |
| CPU samples and Timeline intervals | Resolution, unwind/symbol coverage, task scope, clock alignment and loss determine what attribution is possible. Configured width is not observed parallelism. |
| Retention, queries and publication | Result-policy costs, inspection/projection costs and numerical solve costs are distinct, even when included in the same parent total. |

Many Timeline captures are partial because packets were lost; CPU loss can be unknown.
The readable profiles remain useful within their reported coverage. Decoded Samply
gzip profiles are included, but exact historical ELFs and a separately portable symbol
sidecar are not: arbitrary re-symbolization of old raw perf data is not established.

Analysis has an existing 64-exact-subject limit. Where necessary, publication analyzed
an explicitly selected smaller supplied operation while retaining the full frozen
input. A valid smaller analysis is not a complete diagnosis of the root graph. Table,
analysis and document allowances also have different meanings; none is a whole-process
memory measurement.

## Source map

These are entry points into responsibilities, not an exhaustive symbol inventory.
The authored input, actual returned route and observed phase determine which apply.

| Responsibility | Principal source owners |
| --- | --- |
| Campaign declarations and execution/collection | `.config/process-cases.json`, `.config/preparation-cases.json`; `scripts/pipeline_review.py`, `scripts/case_audit.py`, `scripts/case_measure.py` |
| Scientific fixtures and independent assertions | `benches/benches/native_process.rs`; its `compact.rs`, `extended.rs`, `review_cases.rs`, `results_analysis.rs` and sibling modules; `benches/benches/modeling_preparation.rs`; `benches/benches/k4/studies.rs`; `tests/support/plan14.rs` |
| Authored/physical admission and selected preparation | `crates/pse-runtime/src/workflow/modeling/`; `crates/pse-modeling/src/`; `crates/pse-compiler/src/workspace.rs` and its modules |
| Mathematics, products and provider effects | `crates/pse-math/src/assembly.rs`, `products.rs`, `implicit.rs`, `implicit_products.rs` and execution modules; `crates/pse-kernels/src/` |
| Routing, transformation, start transport and native dispatch | `crates/pse-backend-native/src/routing.rs`, `presolve/pipeline.rs`, `execution/runner.rs`, `execution/` and individual adapter modules |
| Original-space quality and cone assessment | `crates/pse-backend-native/src/quality.rs`, `conic.rs`, `solve.rs` and runtime solve assessment |
| Runtime solve composition, decomposition and reuse | `crates/pse-runtime/src/math/solves.rs`, `math/solves/derived.rs`, `blocks.rs`, related strategy/product modules; `workflow/strategies/` |
| Studies, cancellation, dispatch and result retention | `crates/pse-runtime/src/workflow/study*.rs`, `workflow/run.rs`, `workflow/results.rs`, `workflow/replay.rs`; `crates/pse-operations/src/` and the worker boundary |
| Dynamics, sensitivities, fitting and shooting | `crates/pse-backend-native/src/dynamics/`, `derivative_dynamics.rs`; `crates/pse-runtime/src/workflow/dynamics.rs`, `fitting/`, `shooting.rs`, `horizon/` |
| Native observations and callback/linear attribution | `crates/pse-backend-native/src/observation.rs`, `solve/observations.rs`, `callback.rs`, `kinsol/linear_work.rs`, `pounce/observed.rs`, `petsc/logging.rs` and adapter-specific counters |
| Live evidence, joins, frozen analysis and packs | `crates/pse-runtime/src/observation.rs`, `invocation.rs`, `capture/`, `inspection.rs`, `frozen_inspection.rs`, `review_analysis/`, `review_query.rs`, `review_pack/`; `workflow/review_context/` and `review_export.rs` |
| Relational/Arrow instrumentation and schemas | `crates/pse-engine/src/session/`; `crates/pse-schema/src/catalog/diagnostic_evidence.rs`; generated diagnostic relations under `pse-model`/`pse-relations` |
| Python access and observer probes | `crates/pse-py/src/diagnostics.rs`; `python/pse/diagnostics.py`, `_diagnostic_query.py`; `python/pse/tests/test_review_observer_fit.py` and selected result/Arrow tests |

Dependency declarations and exact pins are in `Cargo.toml`, `Cargo.lock` and the Python
manifests. The local `native-solver-libraries`, `symbolica-faer-oximo` and other library
skills provide source-backed capability routes; pinned dependency source remains
important where adapter behavior or an upstream capability affects a conclusion.
Native-library capability, current adapter exposure and actual exercised configuration
are separate questions.

## Known outcomes worth keeping distinct

The retained notes are leads for independent assessment, not a closed optimization list.

**Uno accepted starts, two and sixteen variables.** The later runs reach two native
joins and successful numerical returns, but fail the accepted-start submission evidence
assertion. Source inspection traces supplied primal coordinates into `pipeline.initial`
while primal-only input does not populate `Pipeline.warm`; Uno derives its submission
flag from that optional warm value. The observations prove the seed exists and the
flags are false, not direct tracing of the native upload. This differs from both a
cold fallback and a failed numerical solve. The relevant chain is
`presolve/pipeline.rs` → `execution/runner.rs` → `execution/uno.rs` → `uno.rs` →
`solve.rs` start-receipt recording.

**SOC and PSD feasibility.** Clarabel returns `Solved`, but the independent original
primal cone residuals are approximately 0.004418 and 0.001919, above 0.001. The adapter
reconstructs slack as `b - A*x` and evaluates the actual cone violation. Native relative
residuals and native-reported slack are not interchangeable with that physical test.
Both cases remain failed; substituting the reported slack would hide the discrepancy.

**Banded root at 64 variables.** Captured records contain successful scalar components
followed by `KIN_STEP_LT_STPTOL` limit returns, including a fallback; the original
candidate remains unusable/unqualified. This is not a profiler/export failure or the
external 60-second stop. The 256-variable case instead stops during preparation with
no observed native join, so it does not establish a native convergence defect.

**Other interruptions.** Nine selected configurations stop within the bounded campaign.
Several do not reach native execution. Their phase history can identify preparation
cost or a missing observation boundary, but cannot support conclusions about the
unentered solver trajectory. Smaller completed cases and focused controls supply
additional coverage without making the interrupted originals pass.

**Managed execution and collection history.** The successful durable study completes
in about 35 seconds with an explicit receiver drain and protected result exports.
An earlier attempt overflowed the benchmark main stack; the retry used a task-local
64 MiB main stack and 16 MiB Rust thread stack. Separately, one completed sixteen-point
study was recovered after collector exit 137: its science/cleanup facts were retained,
but the child OS exit and collector termination cause remain unproved. Neither history
should be generalized into a proven production cause from these outputs alone.

Several preliminary failures were harness-contract mistakes, subsequently corrected:
missing explicit coefficient demand, missing complete derivative-product preparation,
an insufficient Compact detail allowance and an invalid Uno seed payload role. The
later retained attempts establish their current outcomes. Publication repairs were
performed from existing recordings, without repeating science. They include matching
Python readers to the existing 64 MiB pack-document maximum, bounded projection/analysis
allowances, exact detail-page projection and correct gzip capture encoding. Original
failures remain in versioned history.

The source copies and task patch in `campaign-setup/` explain these campaign changes.
The checkout was dirty: a Git HEAD alone does not identify every producing source
version, and current source must not be retroactively attributed to every preliminary
attempt. Actual commands, inputs, retained histories and the relevant source contract
together delimit a claim.

The overall evidence is deliberately strongest for identifying unnecessary work and
mechanism-level opportunities in short executions. It is weaker for asymptotic scaling,
industrial model representativeness, rare failures and long-horizon numerical behavior.
Those limits leave room for additional opportunities beyond the observed failure cases;
they do not imply that only recorded hot paths can benefit from review.
