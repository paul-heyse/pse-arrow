---
title: M22 local Linux qualification and Plan 14 closure
status: done
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084, ADR-0085, ADR-0086, ADR-0087]
phase: 1
---

# M22 local Linux qualification

Execute the approved final packet of [Plan 14](14-library-owned-process-simulator.md).
The maintainer defines closure as local Linux scientific, runtime, performance,
repository and independent design qualification. The final scope clarification keeps
this a design-stage milestone: release-profile, coverage and exhaustive feature
campaigns, dependency inventories and the known upstream `proc-macro-error2` warning
do not block it. Selected default/native profiles remain required. Remote CI, Windows/macOS, wheel/sdist
builds, publication and GitHub merges do not block this closure. This does not claim
those environments passed or change remote repository protections.

## Execution order

| Packet | Deliverable | State |
|---|---|---|
| M22.0 | Repair native environment and record local completion boundary | complete |
| M22.1 | Complete exact Q01–Q17 scientific/runtime witnesses | complete |
| M22.2 | Case-cost measurements and measurement/review continuation | complete |
| M22.3 | Complete local functional qualification and affected-gate repairs | complete |
| M22.4 | Measure qualified case preparation/rebuilding and complete operations | complete |
| M22.5 | Independent G1–G8 and PS-G1–PS-G3 reviews | complete |
| M22.6 | Reconcile local decisions/blueprint, evidence and final claims | complete |

Use the existing checkout, pinned toolchain, stable Cargo target and compiler caches.
No cargo clean, cold Rust compilation benchmark, uncached build campaign, new worktree
or toolchain experiment is part of M22. Application compilation is untimed setup.
Cold means a fresh process-case/runtime/compiler owner in an already-built application.
Measurements use the cached `dev` Cargo profile for this design-stage comparison;
they are not release-build performance claims.
Warm means retaining the revision/compiler owner. No new M21 source seal is required.

## Functional acceptance

The current manifest owns exact witnesses; counts are not acceptance. Cover Q01–Q03
physical/indexed meaning, guards and independent derivatives; Q04–Q06 independent
properties, structure and semantic reuse; Q07–Q11 actual native backend, initialization,
class routing, tear and presolve/postsolve/warm-start behavior; Q12–Q13 public native
jobs/results and exact publication; Q14–Q15 conserved dynamics and parameter fitting;
Q16–Q17 actual resource/cancellation/teardown limits and deletion closure.

Map existing runtime single-flight, compile failure, admission and delayed thread-local
destructor controls explicitly. Add a physically valid full-matching numerical-rank
deficiency case, actual recoverable-trial convergence and synchronized after-entry
cancellation. Inspect completed native presolve/postsolve/warm starts and advertised
Python async/dynamic/fitting paths. Use independent analytic, exhaustive, conservation,
Decimal and frozen thermodynamic oracles. Do not change references to fit a failure.

Run `just bootstrap` for the reported environment failure, then `just py-sync-native`
before provenance capture. Validate the manifest, list both phases and discover exact
tests. Run `just architecture-acceptance build/plan14/m22-functional --phase functional`.
The local design-stage aggregate owns default tests/doctests, selected native and
Python journeys, static/governance/generation/family/ADR/docs checks. Do not duplicate it
with ci-pr. Repair owners and resume in fresh directories with explicit affected gates
and change reasons. All correctness runs use force validation; baseline failures are zero.

## Case measurements and evidence interfaces

Retain the twelve cold/warm heater scenarios (1/8/32 blocks, 1/4 process threads).
Add structural case edits and new specializations at medium size with 1/4 threads;
one-thread flash cold/warm/difficult-start, conserved vessel/transient fitting,
exact publication/reopen and synchronized in-flight cancellation workloads.
Measure phase and end-to-end costs, using existing backend metrics and bounded timing
observations. Report absent submetrics as unavailable, not zero. Interpreted execution
does not have a JIT cost. Native nested threads remain one. Each scenario has its own
process, raw samples and confidence intervals (at least ten samples for normal timing).
Track pool peaks separately from process RSS; include attempt teardown. Compare clean
and reused preparation under identical physical inputs and numerical policies.
Do not impose an invented speedup or absolute time threshold.

Extend existing runner/xtask entrypoints with `--stop-after plan14-measure`: measurements
are retained while reviews remain explicitly pending and qualification incomplete.
Resume with authenticated prior evidence into a new directory. Replace the heater-only
measurement artifact/validator together; no historical artifact qualifies by compatibility.
Test partial checkpoints, full completion, tampering, stale/native changes, exact witness
coverage and documentation-only continuation.

After current functional qualification, run the performance campaign with its
`--functional-from` directory and the measurement checkpoint. Performance correctness
failures return to functional repair. Rust compilation/build cost is excluded.

## Independent review and local closure

Three non-implementing agents review architecture/runtime (G1/G4/G5/G6), scientific
integrity (G2/G3/PS-G1/PS-G2/PS-G3), and claims/library ownership (G7/G8). Reviews inspect
actual source, functional reports and measurements under the layered standard. Each
gate requires independent authored acceptance, evidence paths, source identity and no
open mandatory finding. Accept-scoped cannot discard mandatory M22 requirements.

Reconcile ADR-0082–0084 and relevant ADR-0085/0086 contracts, blueprint sections and
revision markers, obsolete proposed decisions and register entries. Accepted arguments
remain immutable; supersession is explicit. Record the scoped local-closure decision.
Update active capabilities, examples, inventory/index, README and AGENTS from evidence.
Regenerate indexes/contracts through owners. No remote CI or merge is required.

Final documentation changes use one explicit continuation: preserve the original
measurement/source identity; refresh affected document/decision checks; retain evidence
only for unchanged executable inputs/native bytes; independently reissue reviews against
the final digest. Do not relabel older runs as current execution. Final closure requires
complete authenticated Q01–Q18, all required local gates, all eleven independent verdicts,
consistent authority documents, no open mandatory finding and no task-owned process.

## Resource policy

The maintainer's 192 GB, 16-core/32-thread workstation supports larger declared
allowances. Shared workflow fixtures use a 64 GiB pool; native mathematics defaults
to 8 GiB retained artifacts, 2 GiB per worker and 4 GiB per workspace. Compiler input
capacity is 2 GiB with the existing 64-query cache retained. Two engineering tests may
run concurrently, giving a 128 GiB aggregate declared ceiling. These are admission
allowances, not claims that foreign allocations or process RSS are capped. Explicit
small-budget refusal controls remain small and must fail as designed.

## Current state

Functional Q01–Q17 and all 23 case measurements are complete on local Linux.
All eleven independent gates accept the scoped target with no open MUST finding.
ADR-0082–0087 and blueprint revision 51 reconcile the current authority.
Detailed logs remain in the authenticated campaign artifacts named below.

## Design-stage static boundary

The maintainer prioritizes functional target proof and forward design progress.
Strict workspace Clippy cleanup is separate work: the attempted run exposed
documentation and style findings across earlier implementation packets and is not
claimed clean. Normal compilation, functional tests, schema/governance tests,
Python quality, generation, ADR and book checks remain required. The full workspace
test run already includes governance tests; a second package-only governance run
is redundant and would build another dependency-feature graph. Ordinary merge
and release quality requirements remain unchanged.


## Verification

**Tested — local Linux, baseline zero:**
`just architecture-acceptance build/plan14/m22-functional-cost-input --phase functional`
(with the explicit continuation/affected-gate arguments preserved in `checks.json`)
qualifies all 40 required checks and Q01–Q17. The executed evidence comprises:

| Command / mode | Result |
|---|---|
| `just test --profile ci --success-output final`, explicit `pse-relations/force-validate` | 1,697 passed; zero failed/skipped; 108.5 s; retained workspace observation |
| `just doctest`, explicit force validation | passed; retained separate doctest observation |
| `just plan14-native --profile ci --success-output final`, full selected native solver/force-validation profile | 127 passed; zero failed/skipped |
| `just assessment-python-unit`, current editable native extension | 116 passed; zero failed/skipped |
| `just assessment-python-component`, fresh exact Delta fixture | 18 passed; zero failed/skipped |
| `just assessment-python-integration`, two admitted readers | 4 passed; zero failed/skipped |
| `just plan14-python`, native process/async/dynamic/fitting journeys | 4 passed; zero failed/skipped |
| `just plan14-tools`, manifest-selected evidence controls | 6 passed; zero failed/skipped |
| `.venv/bin/python -m unittest scripts.tests.test_validation` | 28 passed; continuation controls including authenticated reuse of reviewed observations |

The workspace result is retained under explicit maintainer-authorized impact review
of concurrent formatting/style edits. Its original source drift, origin, logs and
artifact hashes remain recorded. It is not relabeled as fresh execution on final
bytes. Subsequent functional changes reran their affected native, fixture, Python
or tooling gates. The tip's `source_unchanged` describes that continuation only.

**Measured — local Linux, cached Cargo `dev` profile:**
`just architecture-acceptance build/plan14/m22-costs --phase performance
--functional-from build/plan14/m22-functional-cost-input --stop-after plan14-measure`
executes 23 independent workload processes. Each has ten flat Criterion samples,
250 ms warmup and a one-second target extended for slow operations. Process threads
are one/four as declared, native nested threads one; force validation is enabled.
Rust compilation and Cargo-lock waits are untimed setup, with stable checkout/target
paths and existing compiler caches preserved. No clean rebuild was requested.
Raw samples, intervals, phases, pool peaks, VmHWM and binary/native identity are in
`build/plan14/m22-costs/plan14-measure.json` and its `process-cost/` children.
These are workstation development costs, not release or isolated-host guarantees.

| Workload | Mean complete operation (ms) | Pool peak (MiB) | Process peak RSS (MiB) |
|---|---:|---:|---:|
| cold-small-1 | 176.965 | 16812.4 | 157.4 |
| cold-medium-1 | 276.688 | 16812.6 | 163.5 |
| cold-large-1 | 742.853 | 16813.2 | 166.2 |
| warm-small-1 | 48.003 | 16812.6 | 157.7 |
| warm-medium-1 | 173.971 | 16812.9 | 159.5 |
| warm-large-1 | 630.860 | 16813.9 | 166.2 |
| structure-medium-1 | 182.150 | 16812.8 | 160.4 |
| specialization-medium-1 | 187.831 | 17644.9 | 159.9 |
| cold-small-4 | 178.251 | 16812.4 | 159.4 |
| cold-medium-4 | 282.076 | 16812.6 | 159.7 |
| cold-large-4 | 752.596 | 16813.2 | 166.5 |
| warm-small-4 | 46.798 | 16812.6 | 157.9 |
| warm-medium-4 | 196.133 | 16812.9 | 159.8 |
| warm-large-4 | 979.559 | 16813.9 | 166.6 |
| structure-medium-4 | 266.875 | 16812.8 | 160.3 |
| specialization-medium-4 | 214.941 | 17644.9 | 160.4 |
| flash-cold-1 | 2579.687 | 17068.6 | 161.6 |
| flash-warm-1 | 1216.655 | 17068.8 | 161.0 |
| flash-difficult-1 | 1010.728 | 17068.6 | 164.6 |
| vessel-cold-1 | 149.982 | 21830.7 | 158.5 |
| fit-cold-1 | 648.856 | 21830.8 | 164.1 |
| publication-cold-1 | 1854.987 | 16812.4 | 225.5 |
| cancellation-cold-1 | 175.920 | 16813.2 | 174.3 |

The `flash-difficult-1` workload multiplies both initial liquid and vapor density by
1.02; all other initial coordinates remain at the reference. This setting was selected
after the original 1.15 density multiplier failed local convergence. That earlier run
reported `Infeasible_Problem_Detected`, no feasibility/assurance acceptance, and a
conservation closure of approximately -0.0784. Its failed observation is preserved in
`build/plan14/m22-measurements/process-cost/flash-difficult-1/process.log`. This is a
local starting-point limitation, not proof of global infeasibility. The qualified 1.02
case keeps all physical references and tolerances unchanged; no earlier sample is
relabeled, and no robustness to arbitrary initial guesses is claimed.

Compiler spans are inclusive observations; phase averages include warmup/calibration
operations, whereas Criterion means use its timed samples. Native conversion and
property-state construction are included but have no separate clocks; JIT is disabled.
Pool peaks are reservations, separately reported from process-lifetime RSS. No speedup
or process-wide memory cap is inferred from an admission allowance.

## Outcome

**Implemented and Tested:** the library-owned process target has actual native
convergence, independent physical/derivative references, class-aware refusals,
presolve/postsolve recovery, conserved dynamics/fitting, Python jobs, exact publication,
cancellation and retained-buffer witnesses. Larger workstation allowances preserve
explicit tiny-budget refusal controls. Legacy math/Pyomo and orphaned rule execution
remain removed; no compatibility engine was restored.

**Mistakes corrected:** the KINSOL Jacobian callback now restores CSC structure after
native clearing; the measurement harness enters its existing Tokio runtime before
starting jobs; the stale DSL `Call.named` test field is removed; vessel providers
use the current kernel syntax; dynamic structural refusals preserve offending IDs;
exact inspection fixtures publish checked registry reflection. Python ownership tests
now distinguish active cache-reader pins from independent exported buffer leases and
prove actual last-array release without residency. Full workspace tests were repeated
unnecessarily during qualification; their successful result is retained thereafter.

**Deliberate deviations:** strict Clippy documentation/style cleanup remains separate
by explicit maintainer decision, with no lint-clean claim. The upstream
`proc-macro-error2` warning is accepted without repair. Release/coverage/exhaustive
feature, remote CI, other-platform, distribution and IDAES parity campaigns are outside
this design-stage local milestone. Empirical property certification, global stability,
general implicit/higher-index DAE, hybrid gradients, global identifiability/covariance,
JIT/SIMD and general MINLP remain unsupported. See ADR-0087 and blueprint §0.5.


**Independently reviewed:** [runtime](../design_review/reviews/design_review_m22-runtime_2026-09-24.md)
(G1/G4/G5/G6), [scientific](../design_review/reviews/design_review_m22-scientific_2026-09-24.md)
(G2/G3/PS-G1/PS-G2/PS-G3), and [claims/library](../design_review/reviews/design_review_m22-local-qualification_2026-09-24.md)
(G7/G8) accept the local scope with zero open MUST findings. The final functional
continuation is `build/plan14/m22-closure-qualified`; the measurement/review
continuation is `build/plan14/m22-closed`. They preserve original executed evidence
and measurement identity while refreshing affected documentation and review gates.
