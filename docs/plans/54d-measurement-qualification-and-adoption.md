---
title: "Plan 54d: Measurement, qualification and adoption"
status: in-progress
date: 2026-10-24
adrs: []
review_sources: [docs/design_review/reviews/design_review_rust-build-reuse_2026-10-24.md, docs/functional_reviews/rust-build-reuse-tooling-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/54-rust-build-reuse.md
doc_topics: [current-work, preparation, validation, performance, documentation]
---

# Plan 54d: Measurement, qualification and adoption

## Target and foundation

**Proposed:** measurements exercise the same relevant compiler inputs as supported workflows,
with source/artifact isolation and truthful attribution. Joined acceptance distinguishes
correctness, useful end-to-end benefit and externally limited opportunities. The
[coordinator](54-rust-build-reuse.md) owns scenarios S01–S09 and overall completion;
this companion owns measurement changes, assembled acceptance and adoption evidence.

**Interface-checked:** scripts/build_measurements.py already copies an exact working-source
snapshot, runs from that source, retains preparation/build/execution phases and supports
artifact recovery and a second snapshot. Its explicit campaign target variables defeat
some reuse it seeks to measure. Existing scripts/build_cache_probe.py is a disposable
canary, not an end-to-end performance or checkout-relocation qualification.

## D01 — Make each measurement snapshot own its default Cargo target

Use the copied checkout's default target, rather than exporting an absolute
CARGO_TARGET_DIR or PSE_CARGO_TARGET_DIR for the campaign:

| Role | Working source | Owned build target |
|---|---|---|
| Primary snapshot | output/source | output/source/target |
| Second snapshot | output/second/source | output/second/source/target |

Both ordinary Cargo measurement and nested execute_workload calls already use cwd=source.
This allows nested just, maturin, stub generation and native contract consumers to select
the correct isolated target through their normal checkout rules. Keep original-source
inventory checks, isolated Python environments, native capability selection, owned cache
endpoints and report destinations. Do not turn off isolation to gain reuse.

Normalize the campaign environment after build_environment.configure: preserve the selected
encoded compiler/frontend flags, but remove the campaign's target override (including the
frontend experiment's target path) and inherited PSE_CARGO_TARGET_DIR before invoking the
snapshot. Explicitly report the campaign's owned target selection instead of silently
pretending a caller's target override was honored. This is local measurement policy;
ordinary build-frontend and explicit developer target-directory behavior remain owned by
their current routes. Do not scrub arbitrary CARGO_* variables inside the rustc wrapper.

Pass the resolved owned target as an explicit path to measurement, timing-report, receipt
and recovery helpers. Remove their dependence on env['CARGO_TARGET_DIR']; no environment
value is needed to remember a target already known by the campaign. Record the actual
target choice alongside the source in retained measurement output. Check effective Cargo
target selection against the snapshot-owned target and refuse an inherited config/override
that would escape it before compiling or deleting anything. Do not weaken installer guards
or write a target-dir override into authored repository configuration.

Recovery may delete only the authenticated selected snapshot's default target. Validate
resolved ownership/path boundaries and the exact expected target before removing it;
refuse original-checkout, external, symlink-escaping or shared targets. Losing artifacts
and rebuilding at the same path is a separate case from switching source/target locations.
The second-snapshot phase uses its own default target without modifying the first. Reports
and source snapshots are retained according to the existing experiment owner, not discarded
as part of artifact recovery. No general cleanup feature is added.

**Acceptance, proposed:** scripts/tests/test_build_measurements.py exercises direct and
nested workflow environment without either target variable, retained frontend flags,
primary/second target identity, timing-copy paths, report attribution and refusal of unsafe
recovery destinations. Use the real harness with --recovery and --second-worktree on a
selected package, observing Cargo's actual artifact destinations and a compatible populated
cache. An eligible fixture without OUT_DIR must be able to reuse across campaigns; remaining
real-workspace OUT_DIR misses stay separately attributed. Preserve the existing workflow
selection and correctness proof. Update old tests of env-based routing to protect the new
owned-target behavior; do not retain the obsolete path for compatibility.

## D02 — Qualify the combined target and measure useful benefit

After selected functional packages and bounded inquiry decisions are integrated, run one
deliberate joined campaign. A01/A02 make tool/mode trustworthy, A03/D01 remove accidental
caller differences, B01/B02 change invalidation/graph behavior, and any selected C changes
extend the footprint. Select the following journeys; they establish this build subsystem's
supported consumers, not whole-platform scientific qualification.

| Journey | Required observation |
|---|---|
| Same-checkout no-op and private/public edits | Fresh no-op avoids rustc; genuine edits produce correct current behavior; incremental edited-crate latency remains visible |
| Identical-byte touch/restore | Cargo freshness avoids the affected Rust recompile; build-script mtime exceptions remain explicit |
| Real and generated input changes | Same-length source edits, changed includes, declared build-script files/env and changed dependency inputs invalidate appropriately |
| Fresh campaign and second worktree | Effective target and caller inputs are accounted for; cache is populated under matching settings before expected hits |
| Same-path artifact loss | Only owned artifacts are removed; restored outputs are correct and safe; no old-target artifact is mistaken for a new source |
| Agent/terminal and nested recipes | Refreshed equivalent callers reuse eligible work without unintended flag/env differences |
| Selected Rust and native correctness | Actual Arrow leaves remain validated in relevant target/host/build/dev closure; native conditional removals are compiled |
| Interactive/correctness Python | Maturin installation, stubs and loaded extension behave correctly with new freshness/selection; requested validation distinctions persist |
| Logging/configuration/interruption | Explicit cache requests refuse incompatible mode; ordinary fallback and actual caller-owned drain remain truthful |

Use existing recipes and tests. Package-local Python script controls run through the shared
environment and pinned .venv interpreter. Future command selections include:

    scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_build_environment scripts.tests.test_compiler_cache scripts.tests.test_build_measurements scripts.tests.test_installation_context scripts.tests.test_audit_tools
    just build-cache-probe
    just check
    just check-package pse-tests-conformance --features force-validate
    just check-native-package pse-runtime native-solvers
    just unit-package pse-diagnostics all()
    just doctest
    just py-sync
    just py-sync-correctness
    just py-unit

The filterset all() is passed as a quoted positional argument in a shell. Choose affected
backend/optional canonical/native target compilations based on B02's actual changes; normal
workspace --all-targets does not enable them all. Where native-dependent Python is affected,
use the native correctness sync and selected public installed-extension tests instead of
claiming that a raw cdylib Rust harness exercised them. A real rust-analyzer session checks
the repository configuration path; report any unavailable interactive observation explicitly.

For measured comparisons, use the existing just bench-builds harness and build-uncached
controls. Compare the same selected workload before/after using each revision's correct
isolated target routing; hold effective compiler/profile/validation/native choices constant
where they are not the variable under test. Keep production-equivalent timing and explicitly
validated correctness observations distinguishable. Do not disable validation to obtain a
correctness speed result or run a new all-solver/parity campaign for tooling alone.

Measure cold compiler-cache plus fresh target, warm compiler-cache plus fresh target, warm
Cargo artifacts, real edit/revert and same-path recovery separately. Use at least three
warm/edit repetitions (the current harness default) and report median and observed dispersion;
one cold run remains a single observation unless the decision actually needs more. Run
comparative timings without overlapping heavy owned work and describe unavoidable host
competition. Existing admission/reference allocations govern timing isolation; do not add a
second concurrency governor. Relative results within noise remain inconclusive, not gains.

**Maintainer scope amendment — 2026-10-24:** skip the final comparative performance
campaign. Its remaining purpose was speed quantification; no implementation/adoption decision
depended on it. Existing real correctness, invalidation, publication/recovery and consumer
controls remain required. Retain their evidence and the actual completed reference observations;
do not substitute a small benchmark or claim an unmeasured end-to-end speedup. The configured
50 GiB measurement guard remains unchanged, and original-checkout artifact cleanup is not
authorized or needed for this amendment.

Report complete wall time and preparation/build/install/execution costs relevant to each
claim, with attributable eligible hits/misses, non-cacheable activity and cache errors.
Cargo --timings unit durations show elapsed compiler-invocation intervals and critical-path
relationships; their sum is not CPU consumption and not wall-time saved. Actual CPU usage
requires process accounting. Cache-hit percentage, removed manifest edges and individually
timed doctests are not additive end-to-end speed estimates. Identify relocation's supported
subset rather than requiring 100% hits from intentionally incremental/linking work.

At functional scope end, run just hygiene and repair selected failures, plus the relevant
manual just governance, just docs, native-contract and feature-combination checks affected
by the actual change. Reuse existing feature/validation qualification and retained evidence
where its premises still apply; do not invent a fresh blanket product campaign for these
plans. Run the mandatory narrow static recipes once as the repository policy requires,
and rerun failures rather than repeating all checks after documentation edits. Document
actual command, mode, scope, zero baseline, result and exclusions.

## D03 — Adopt the demonstrated system and publish its limits

Update docs/dev/build-performance.md for the selected binary, logging/client-side contract,
shared cache scope, snapshot-local measurement targets, content freshness and actual
cross-location limits. Update the existing environment/dependency/qualification guides and
command descriptions where changed behavior requires it. Explain direct fallback versus
explicit refusal, caller overrides, build-script mtime limits, meaningful validation/profile
variants and why misses can be correct. Keep the library reference repository-agnostic and
unchanged by this implementation series.

For adopted remapping or storage settings, document supported roots/layouts, source mapping,
activation and future upgrade checks. For no-adoption decisions, retain the actual capability
gap and concrete revisit condition without implying an upstream PR will inevitably close it.
Carry enduring contracts through their existing architecture/ADR route only if they change.
No new ADR is required merely for tooling pins or the selected within-contract fixes.

Reconcile coordinator source finding/inquiry dispositions and companion package states with
the executed outcomes. A package can complete without a measurable speedup when it delivers
the required correctness or maintainability change; a benefit claim requires its own measured
conditions. Explicitly separate Implemented, Tested and Measured. Record deliberate deviations
and any mistake actually made and corrected when writing the closing Outcome; do not invent
one during planning. Retire completed artifacts through the existing lifecycle only after
their enduring owners and remaining readers are reconciled. Other plans remain independent.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| D01 | complete | 54d | Snapshot-local default targets, explicit report paths and guarded artifact recovery |
| D02 | in progress | 54d | Joined selected-consumer acceptance, zero-target scope-end checks and honest comparisons |
| D03 | in progress | 54d | Actual supported behavior published and series dispositions reconciled |

D01 can begin early and supplies trustworthy measurements to 54c. D02 consumes completed
selected functional changes and inquiry dispositions; D03 follows actual acceptance. No
performance campaign or product qualification was executed while authoring this plan series.

## Execution checkpoint

D01 passed focused routing/recovery controls, including real pinned-Cargo refusal of
external intermediate build directories before artifact deletion. The corrected 0.19
reference campaign completed cold, three warm/edit/touch repetitions, same-path recovery
and a second source snapshot with original-source inventory unchanged. Each used its own
default target. Its cache statistics proved incomplete for oversized publications; it is
retained as the actual pre-change reference, not relabeled as full eligible-hit recovery.

Joined qualification is running. Default workspace, selected native runtime and optional
xtask canonical/native compilation passed; external macro behavior, content invalidation,
managed large-entry publication/recovery and real analyzer reload/navigation/save controls
passed. The analyzer adapter's unsupported check.enable key was corrected to checkOnSave;
its actual saved-document control now emits no flycheck while retaining compile-time deps.
Analyzer stderr startup/cycle/slow-loop warnings remain explicitly outside a clean-editor claim.

The first hygiene pass found 15 Python lint findings and a stale generated capability
reference. The lint repairs passed the affected 90 host controls and just lint-py; just codegen
regenerated the reference and the full governance bundle passed. All other hygiene leaves,
including Clippy and rustdoc, passed. Remaining real doctests and repaired native contract
compilation passed. Interactive and correctness dev Python installation/stubs/import admission
passed, but the subsequent full dev unit run failed: 218 passed and 31 failed against zero.
Fourteen cases require linked native features; 17 concern separate mock-effect, protected
diagnostic and parity-fixture contracts. Independent triage found no stale-extension evidence.
The native correctness extension subsequently built, generated stubs and passed import
admission with force-validate/native-solvers/solver-highs. Its full unit run had 232 passes
and the same 17 separate contract/fixture failures; all 14 native-required cases passed.
Neither full Python run is a zero-failure qualification. The managed native CMake launcher
and exact selected-Clang resource argument now have explicit closed-context support and
actual compiler-context capture passed; arbitrary external programs/flags still refuse.
Feature qualification is running under the repaired context. The final analyzer reload,
navigation and no-flycheck save controls passed with the declared frame environment.

The feature matrix was restarted with explicit CARGO_INCREMENTAL=0 to avoid retaining
incremental query caches for one-off qualification variants; ordinary workspace build
policy is unchanged. The earlier interrupted run's 173 passes are not promoted to complete
matrix evidence or reused across the changed compiler context. Stopping the authenticated
campaign controller drained its component and preserved the interrupted receipt. An initial
signal to the outer launcher alone did not cascade to that separately grouped controller;
the existing generic launcher forwarding limitation is outside the direct compiler-caller
interruption control and is not claimed qualified here.

The maintainer skipped the final matched performance campaign: it would quantify speedup
without deciding adoption. No smaller benchmark or original-checkout artifact cleanup will
be substituted. The dry-run-only inventory identified 16.6 GiB of pse-py artifacts; these and
the separately installed native extension remain intact. Final publication and completed
feature qualification remain active obligations. An initial docs attempt exhausted light admission; retry after
an authenticated idle superseded daemon stopped passed, publishing 654 chapters.
No unrelated worktree, source, cache or running compiler was stopped.

The maintainer subsequently authorized purging historical build cohorts before October 19
and investigating obsolete caches across this checkout. Cleanup reclaimed 68.246 GiB by
unique-inode allocation accounting: 1,619 historical build cohorts and three dormant nested
Cargo targets. The deletion/accounting receipt is
`/tmp/pse-historical-artifact-cleanup-20261024.json`; the conservative cutoff was
2026-10-19T00:00:00Z. Current qualification outputs, the root target, installed extension, shared
compiler/native caches and canonical state were retained. This storage accounting does not
establish a build-time speedup or authorize recurring automatic eviction.

### Background campaign handoff — 2026-10-24

The maintainer requested end-of-turn updates and an end to active work, explicitly leaving
the feature campaign running. The active invocation is `CARGO_INCREMENTAL=0 just features-powerset`,
with log `/tmp/pse-plan54-features-nonincremental.log` and aggregate output
`build/assessment/20261024T215756.070343Z-features-powerset-84389-9a511f`. Its
`features-combinations/feature-campaign.json` owns individual results; the aggregate proceeds
to `features-no-default` afterward. The combinations receipt remains incomplete, with no
failed combinations observed at handoff. Partial passes do not establish matrix acceptance.

Leave the campaign and its owned processes running. On the maintainer's next review request,
inspect the aggregate and both component results, final source/compiler-context equivalence
and exit statuses before updating D02/D03. The 17 separate Python contract/fixture failures
remain recorded above; final publication and series acceptance remain pending. No further
benchmark, implementation or investigation is started by this handoff.
