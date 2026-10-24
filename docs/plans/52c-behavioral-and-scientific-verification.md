---
title: "Plan 52c: Behavioral and scientific verification"
status: in-progress
date: 2026-10-23
adrs: [ADR-0223]
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, validation, numerical-execution, diagnostics]
---

# Plan 52c: Behavioral and scientific verification

## Target and foundations

**Proposed:** give Codex revealing ways to challenge incorrect behavior, understand scientific
mismatches and retain smaller failures. Owns O08–O14. The
[coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns progress/I05/I06 and RC05.
No technique becomes a mandatory repository-wide score, campaign or new scientific authority.

Existing `affected.py`, cargo-llvm-cov/nextest, Proptest1.11, independent references,
`derivative_diagnostics.rs`, strict `workflow/reproducer.rs` and the pure-ID Miri recipe remain
foundations. Test Assessment1.2 shapes expected-result independence and plausible-bad controls.
A generator, a sanitizer, a high-precision result and a round trip establish different claims.

## Work packages

<a id="C01"></a>
### C01 — Changed-source coverage correspondence

Extend `affected` and coverage reporting with selected changed regions, branches/missing paths,
actual executed scope and exclusions. Existing LCOV is merged and excludes generated/test/bench/
xtask paths; preserve those limits. Use
[cargo-llvm-cov JSON/reporting](https://github.com/taiki-e/cargo-llvm-cov), checking exact support
for unstable branch instrumentation. Tie observations to content, compiler, features, generation
and instrumentation. Merged data cannot name an individual covering test; that needs separately
selected captures or explicit correspondence. Rust coverage says nothing about opaque native C/C++.

Acceptance: a real changed rejection branch is unexercised, then a selected test capture reaches
it; source/build drift makes old coverage stale. Dependency selection remains the conservative
fallback. Explain what the assertion challenges independently of coverage. No percentage gate or
automatic test omission based on stale execution history. A compact JSON/readable view feeds G01
and F01 without another test-selection service.

<a id="C02"></a>
### C02 — Semantic transformations and stateful challenges

Extend actual covariance, implicit-product, quantity and preparation test owners with independently
specified properties: coherent scaling/permutation (`C′ = D C Dᵀ`), analytic expected variance,
keyed coordinate permutation and edit→prepare→bind→evaluate sequences. Compare cold/reused paths
against the independent analytic result, not merely each other. Invalid, withheld and incomplete
states stay distinct; arbitrary nonlinear starts need not select the same branch.

Use existing Proptest and
[failure persistence](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html).
I05 evaluates the separate
[proptest-state-machine crate](https://github.com/proptest-rs/proptest/blob/main/book/src/proptest/state-machine.md)
versus ordinary sequence strategies. Add
[Hypothesis state machines](https://hypothesis.readthedocs.io/en/latest/stateful.html) only for
actual Python/Arrow lifetime behavior that Rust-owned generation does not exercise. Do not add
another dependency merely to rename an existing capability. Pin any selected addition exactly.

Acceptance: generate and shrink a stale-preparation/coordinate-association counterexample into
an interpretable short sequence; retain concrete inputs as well as seed/strategy identity.
Independently expected numeric values and a deliberately wrong association expose correlated
errors. Preserve malformed cases rather than repair them in builders. No universal property registry.

<a id="C03"></a>
### C03 — Assertion sensitivity through mutation and explicit faults

Challenge consequential acceptance/status/comparator predicates with plausible wrong behavior.
Use [cargo-mutants diff selection](https://mutants.rs/in-diff.html),
[nextest integration](https://mutants.rs/nextest.html),
[Cargo arguments](https://mutants.rs/cargo-args.html) and
[outcome classification](https://mutants.rs/using-results.html). Bootstrap/`mutants-file` presence
is not proof that native libraries, license, source placement or force-validation survive the
mutation workspace. Compose `native_tests` correctness/environment owners; qualify the unchanged
selected test there before interpreting mutations. Do not inherit an arbitrary two-job recipe
limit as a new machine policy; isolated native safety and actual tool semantics govern the run.

A test-only edit needs an explicit production target. Distinguish caught, missed, unviable,
timeout, equivalent and setup/environment failures. Acceptance: a meaningful wrong predicate
survives an insufficient assertion, then an independently derived counterexample rejects it.
Use explicit domain-specific fault injection if generic operators/native-copy integration obscure
the question. The base capability is a usable bounded sensitivity check, not an all-crate mutation
score or claims about untested predicates. Keep isolated tool copies owned and drained.

<a id="C04"></a>
### C04 — Pure-boundary coverage-guided exploration

Start one finite [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)/
[LibFuzzer](https://llvm.org/docs/LibFuzzer.html) target at a pure authored parser or frozen
inspection decoder, using authentic examples and malformed/version/size cases. Existing DSL
strategies can seed rendered inputs; Proptest and fuzz `Arbitrary` traits are different.
[Structure-aware mutation](https://github.com/google/fuzzing/blob/master/docs/structure-aware-fuzzing.md)
can improve reach beyond immediate parse refusal without repairing invalidity away.

Assert no unexpected panic, bounded refusal and independently defined selected meaning where
claimed. Parser/renderer agreement alone is weaker. Use corpus/failure minimization and promote
concrete reduced cases to ordinary tests. No server/native solver startup is a prerequisite.
Acceptance includes unsupported codec version, oversized input and a nontrivial decoder path,
with exact instrumentation and finite workload ownership. If toolchain compatibility blocks
coverage-guided execution, retain an explicitly narrower structured-generator route and record
the unsupported tool scope; do not call that coverage-guided qualification.

<a id="C05"></a>
### C05 — Numerical reference investigation

Compose existing native derivative diagnostics/dynamics, `plan14_reference.py` and independent
implicit-property/product tests into selected legal-step sweeps with units, coordinates, scaling,
completeness, rejected evaluations, conditioning clues and convergence/status visible. Export machine-readable samples
and standalone plots of error versus step. This is an investigation consumer, not another solver.

[SciPy adaptive Jacobians](https://docs.scipy.org/doc/scipy/reference/generated/scipy.differentiate.jacobian.html)
are conditional on the actual pinned API and valid perturbations. Existing fixed legal steps and
Decimal references are the simpler fallback. Add
[mpmath](https://mpmath.org/doc/current/contexts.html) only for a demonstrated precision/function
need; its experimental intervals are not certified enclosures. More precision/shared formulas do
not create an independent oracle. Differences of the production evaluator challenge its
derivatives, not its underlying physical formula.

Acceptance: an analytic case with independent value/derivative, a domain-limited nonlinear case
with central/inward steps and refused perturbations, and an intentionally wrong derivative or
coordinate that the report exposes. Include a sensitivity/roundoff-dominated step regime so the
consumer can distinguish numerical sensitivity from persistent derivative error; a universal
condition number or certified conditioning claim is not required. Preserve existing report meaning and no tolerance loosening
merely to pass. C06/E03 can consume the explicit issue/predicate evidence without using this as
a new execution engine.

<a id="C06"></a>
### C06 — Separate scientific reduction predicate and source-aware search

Follow FND00/RC05 before introducing a new runtime document contract. Preserve the original
reproducer and `OriginalFailureReproduced` unchanged. Define a separate reduction request/result
adjacent to `workflow/reproducer.rs`: selected original typed diagnostic, permitted reduction
operators, source/coordinate correspondence, preserved family/stage/rule, validity/domain/branch
premises and issue-specific conditions. A candidate is preserved, different, refused, incomplete
or inconclusive; a different failure is not successful reduction.

The first supported family is an authored numerical evaluation-domain failure, using the existing
negative-log-at-explicit-start reproducer as the semantic control. Remove irrelevant authored
siblings/operations while preserving the actual failing expression/start and typed source mapping;
then apply only admissible AST reductions whose predicate remains independently checked. Use
source-aware [delta debugging](https://www.debuggingbook.org/html/DeltaDebugger.html), finite owned
candidate execution and cancellation. Do not promise global minimality: report smaller or locally
irreducible under the selected moves/bounds. Provider/frozen cases are supported only when their
necessary correspondence and environment can be retained; otherwise explicit refusal, not guessing.

Acceptance: reduce a multi-operation case, reject a smaller parser error, unrelated singularity
and different domain violation, retain original/reduced artifacts and exact comparison, and run
one ordinary regression for the preserved issue. Cancellation/timeout/resource/dependency-unusable
failures remain outside a point-based numerical reducer. Manual subtraction is preferable for
already small cases. No generic scientific equivalence framework is required.

<a id="C07"></a>
### C07 — Focused lifetime and native instrumentation

Resolve I06's native branch at a small HiGHS callback/`CallbackBinding` lifecycle target, reusing
`highs_lifecycle.rs` and native build owners. PETSc is an alternative when a concrete risk points
there, not an automatic second mandatory solver. Qualify a selected
[AddressSanitizer build](https://doc.rust-lang.org/stable/unstable-book/compiler-flags/sanitizer.html)
with actual Rust/native instrumentation and shared-library identity before interpreting results.
ThreadSanitizer is a separate supported-build question, not an assumed flag addition.

Retain the existing pure-ID [Miri](https://github.com/rust-lang/miri/) route; it cannot execute
arbitrary solver FFI. Instrumentation stops at opaque native dependencies. Acceptance exercises
normal and interrupted callback teardown, with retained ownership until actual drain and a
sensitivity control for the relevant buffer/lifetime fault where feasible. If consistent native
instrumentation is unsupported, record the exact boundary and use existing GDB/rr plus ordinary
lifecycle tests as narrower diagnostics. No full solver-fleet safety claim or second production
implementation. Positive adoption requires one usable actual diagnostic route, not installation alone.

<a id="C08"></a>
### C08 — Controlled cancellation and publication schedules

Resolve I06's modeled branch independently of C07. Select one existing cancellation/drain or
publication transition and inject explicit clocks/scheduling hooks where sufficient. Consider
[Loom0.7.2](https://docs.rs/loom/0.7.2/loom/) only when replacement synchronization types can model
the real mechanism without duplicating its logic. `loom::model`, bounded preemption and checkpointed
failure paths are useful; ordinary synchronization, system calls/native FFI and parts of weak
memory behavior remain outside the model.

Acceptance: force cancellation on both sides of the selected transition and verify ownership/
drain semantics. Preserve production correspondence and report schedule/model bounds. A passing
simplified story is not evidence for unrelated production transitions. If credible correspondence
is too intrusive, select deterministic injected schedules in existing tests and record Loom
non-adoption with a concrete revisit trigger. Neither stress repetition nor rr's single recorded
schedule is presented as systematic interleaving coverage.

## Verification and sequencing

**Proposed:** C01–C05 can start independently at their current owners; C02/C03 reinforce each
other without a circular prerequisite. C06 requires its new meaning contract and selected typed
failure; C07/C08 are separate conditional diagnostic outcomes. F01 composes actual results and
E03 executes selected experiments. Techniques have targeted checks and finite representative
populations. New Arrow-consuming closures explicitly force validation. 52h selects assembled
Rust/Python/native journeys and scope-end checks after functional delivery; it does not turn
all candidate techniques into broad campaigns or promote tool success into scientific usability.

## Execution boundary — instrumentation

The maintainer accepted stopping deliberate invalid-memory sensitivity work. A subsequent
normal-only ASan task and the finite cargo-fuzz task were also rejected by automatic review
as possible cybersecurity risks; the review exposed no exact blocked command. Both remaining
owned compiler scopes were stopped and drained. This is an execution-policy limit, not evidence
of unsupported toolchain semantics, a solver defect or a successful instrumented run.

C07 continues through the existing ordinary GDB launch owner and actual normal/interrupted
CallbackBinding test. ASan execution and injected-fault sensitivity remain **not_run**. C04
uses ordinary bounded structured decoder controls for actual operation meaning, malformed,
version and size cases. The harness/finite wrapper remain an explicitly unqualified optional
route: no coverage-guided execution, feedback, finding minimization or sanitizer safety claim
is made. Required assembled qualification stays with 52h and must preserve these exclusions.

## Targeted execution Outcome — 2026-10-24

**Implemented:** C01–C08 functionality is integrated with its actual owners and public consumers.
C01/C02/C03/C05/C06/C08 meet their selected focused acceptance; C07 selects the ordinary GDB
diagnostic route. C04's ordinary structured fallback is **Tested**, and its optional harnesses are
**Interface-checked**, but coverage-guided acceptance remains blocked by the execution boundary
above. This companion therefore remains in progress; neither implementation nor compilation
silently closes the unexecuted campaign. No assembled Plan52 or performance/productivity claim
is made. The simulation-diagnostics and qualification guides own enduring operation.

| Package | Actual selected evidence against zero failures | Supported conclusion and limits |
|---|---|---|
| C01 | Actual `coverage_correspondence capture --branch` controls over `pse-ids/src/scientific_replay.rs`: changed rejection line43 had branch counts 0/0, then 1/1 in separately selected one-test captures. Source restoration and compiler-flag changes independently made correspondence stale; `affected --coverage` retained the same conservative selection. | **Tested** source-fresh selected-population/branch correspondence, not individual-test attribution from merged data, assertion strength or native coverage. |
| C02 / I05 | Executor's focused Arrow-validating Cargo selection across math, quantity, runtime and native diagnostics: 11 passed; strengthened actual-provider shrinking control: one passed. Existing Proptest sequences use independent analytic expectations; seed5202 shrinks stale provider input association to `[(0,1)]`, with concrete expected `[1/3,2/3]` versus stale `[0,0]`. | **Tested** covariance scaling/permutation, quantity point/difference behavior, cold/reused implicit and edit→prepare→bind→evaluate paths. Select ordinary existing Proptest strategies; no extra state-machine/Hypothesis dependency without an uncovered Python-owned lifetime behavior. |
| C03 | Actual pinned cargo-mutants `>`→`>=` mutation at the admitted maximum-source boundary: unchanged baselines passed; insufficient ordinary/oversize assertions MISSED it (tool exit2), independently legal exact-maximum counterexample CAUGHT it (exit0). `derivative_assertion_sensitivity_preserves_completeness_and_nonempty_evidence` also passed. | **Tested** revealing assertion sensitivity at those predicates. A missed mutant is retained evidence, not a tool/setup failure or a repository mutation score. Native mutation-copy integration remains unqualified. |
| C04 | Final Rust selection below passed the authentic-operation decoder control, including exact counters above 2^53/u64 maximum, 20 truncations, malformed/version999 and 64MiB+1 size refusal. Ordinary `arrow_validation check -p xtask --locked --features xtask/fuzz-inspection --bin frozen-inspection-fuzz --bin inspection-fuzz-seeds` passed with explicit validation, zero warnings/errors. | **Tested** finite structured boundary cases; **Interface-checked** optional binaries. Coverage-guided execution/feedback/minimization **not_run** after task denial; no actual finding population or clean campaign claimed. |
| C05 | Actual numerical consumer exported 85 dynamic and17 implicit samples; `scripts.tests.test_numerical_investigation`: three passed. Standalone SVG/PNG visually inspected. Cubic finite differences entered a roundoff regime (minimum error about3.96e-10 versus5.29 at the smallest step); deliberately wrong derivative retained error48. Domain central/inward/refused and partial implicit samples remain visible. | **Tested** the selected investigation consumer and independent analytic/100-digit Decimal controls, not physical-formula truth, certified conditioning or universal step selection. Existing legal steps/Decimal suffice; SciPy adaptive and mpmath additions remain unselected. |
| C06 | Final selected native-environment Rust command below: five passed, including source-aware reduction, forged/unrelated premises, quoted-name expression spans, aggregate move-payload refusal, cancellation/expired clock, strict original reproduction and the C04 decoder control. Actual public Python reduction CLI plus existing strict-reproducer regression: two passed. | **Tested** actual shrinking while retaining the selected negative-log issue/expression/start and refusing parser, unrelated singularity and different-domain changes. Original reproduction truth is unchanged. First support is deliberately bounded to the documented scalar-call/start family. |
| C07 / I06 native | `highs_lifecycle_diagnostic NEW_OUTPUT --mode gdb` used the actual linked HiGHS test executable and existing GDB launch owner: normal/interrupted CallbackBinding teardown test passed, inferior exited0, owned drain complete. Two driver qualification controls passed. | **Tested** the ordinary selected GDB/lifecycle route. ASan and fault-sensitivity execution **not_run**; no sanitizer, ThreadSanitizer or solver-fleet safety claim. A genuine supported instrumented route is a separate revisit, not an automatic retry of denied tasks. |
| C08 / I06 schedule | Actual scheduler hooks forced cancellation before gate acquisition and after the read guard but before compute resume; ordinary controls passed ownership/drain and successful-resume assertions. | **Tested** those actual production transitions. Loom is not adopted: the existing std RwLock/Condvar/native-reset path cannot credibly be modeled by replacing synchronization alone. Revisit for a concrete pure atomic publication protocol; no systematic whole-runtime interleaving claim. |

The final Rust command was:

```bash
scripts/pse-env --resource-class build --native=solver,klu -- python3 -m scripts.arrow_validation nextest run -p pse-runtime --lib --features pse-runtime/solver-kinsol --locked -E 'test(reduction::tests) | test(actual_text_numerical_domain_failure_reproduces_selected_typed_source_diagnostic) | test(structured_decoder_controls_keep_actual_operation_meaning_and_refuse_bad_boundaries)'
```

The actual Python command used the freshly installed linked force-validated extension:

```bash
scripts/pse-env --resource-class build --native -- .venv/bin/python -m scripts.python_tests --unit-only --maxfail=0 --continue-on-collection-errors python/pse/tests/test_diagnostics.py -k 'scientific_reduction_cli or reproducer_explicit_runner' --basetemp build/plan52cd-evidence-20261024/python-pytest-v5 -q
```

Its product invocation is `72ac8af0fc0943e4b9fd000fd08d117c`. `PSE_RESOURCE_CLASS=build just codegen`
and `PSE_RESOURCE_CLASS=build just py-sync-native-correctness` passed; stubs derive from the
actual compiled API. `PSE_RESOURCE_CLASS=light just ready` subsequently passed both checks,
including all selected tool versions and the installed native extension. The runner separately
deferred cleanup of three historical unsupported-profile resources;
those remain preserved, not counted as repaired or silently deleted by this scope.
Root integration unittests passed 37 coverage/sensitivity/context/campaign/watch/pressure controls and another25
driver/numerical/pin/source-context controls (overlapping families, not a unique-test total).

Selected actual artifacts are copied to `build/plan52cd-evidence-20261024/`: coverage/mutation
protocols and raw results, numerical JSON/SVG/PNG, GDB result/identity/drain, stopped instrumented
attempt diagnostics and the actual Python temporary workspace. These are retained historical
controls at their original source/build conditions, not fresh reusable evidence for arbitrary
later edits. Worktree compiler targets were not copied or relabeled as new builds.

Mistakes corrected: the numerical diagnostic producer had discarded its typed cause/source
attribution in a message-only envelope. It now projects the actual cause. Independent review
then exposed unbound anchors, quadratic/eager search, substring-derived edit locations and
unaccounted aggregate move text. Actual control attribution, admitted parser spans, indexed
parents, bounded source/declarations/moves/text and evidence reservations repair those issues;
the reviewer closed all findings. An initially incorrect dotted parser-field lookup was repaired.
Python integration exposed the missing explicit RecordedDocument registration and fixture
union/ID rendering errors; all were fixed before the passing consumer. Shared `/tmp/pytest-of-paul`
was root-owned and stopped collection; a fresh private basetemp solved the selected invocation
without changing that unrelated directory. Earlier failures remain in their native result owners.

Deliberate limits: sensitivity fault injection was stopped at the maintainer's direction; normal
ASan and fuzz followups were separately denied by automatic review and their remaining owned
compiler scopes stopped/drained. This does not establish a toolchain incompatibility. C04's
remaining obligation is a permitted actual finite coverage-guided campaign, with observed tool
feedback/completion and minimization/promotion of any real findings. It remains open rather
than being credited to the structured fallback. Full feature powerset, hygiene/manual checks
and assembled scientific/native/Python journeys remain 52h's independent scope.

All three executor branches are merged into main, their scoped conflicts resolved, selected
artifacts retained, and their fully merged worktrees removed. Unrelated operator edits remain
outside these commits. The ordinary build/diagnostic controls are drained; no service or shared
compiler cache was stopped as part of worktree retirement.
