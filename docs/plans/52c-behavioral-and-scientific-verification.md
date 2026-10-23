---
title: "Plan 52c: Behavioral and scientific verification"
status: draft
date: 2026-10-23
adrs: []
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
foundations. Test Assessment1.0 shapes expected-result independence and plausible-bad controls.
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
