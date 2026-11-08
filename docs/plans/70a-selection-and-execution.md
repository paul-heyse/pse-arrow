---
title: "Plan70a: Selection and execution"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [design, validation, preparation]
review_sources: [docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md]
---

# Selection and execution

**Proposed.** Realize [R01](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md#r01)
at the compiler, native runner and fixture admission boundaries. Plan69c already owns ordinary
root admission, exact failed rerun, and the result and release consumers. This companion supplies
stronger selected-requirement correspondence, actual runner identity and named specialist routes.
[Coordinator](70-test-assurance-architecture.md) · [Plan69c](69c-ordinary-testing-and-managed-results.md)

### <a id="a01"></a>A01 — Verify the actual runner and compiler closure

Resolve the generic nextest executable once against the configured tool pin and verify the
executable selected for launch. Share the existing pin interpretation with managed discovery.
Where the runner claims captured-image identity, retain the authenticated copy and verify the
executable used. Refuse a version mismatch or substitution. Retain generic Cargo as a
fixture-free route. Use `cargo_metadata` 0.23.1 `Target` for declared target kinds, test/doctest
eligibility and required features. Use `--unit-graph -Z unstable-options` on the checkout's pinned
Cargo nightly for actual roots, enabled features and host/target dependency edges. Target metadata
supplies candidates. The correctness adapter checks the admitted graph for explicit Arrow
validation in the actual selected normal, build and dev units. Exercise pure closure, Arrow closure and an
Arrow-consuming doctest with an omitted leaf. The doctest control must distinguish leaf
validation from ordinary source compilation and nextest listing.
[Target API](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.Target.html) ·
[Cargo unit graph](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#unit-graph) ·
[Toolchain pin](../../rust-toolchain.toml) · [Correctness adapter](../../crates/pse-host/src/correctness.rs) ·
[Discovery](../../crates/pse-host/src/discovery.rs)

### <a id="a02"></a>A02 — Bind selected requirements and truthful outcomes

Use Cargo for eligible roots and nextest for actual selection. Associate fixture prerequisites
with the captured identities before invoking the owner that performs the effect. Keep intended
selection, admitted compiler graph, collected cases, readiness, test verdict and settlement
distinct. The physical owner rejects missing or mismatched prerequisites before effects. Where
only the target owner can observe setup, retain an attributable typed setup result and the
assertion-not-reached distinction. Receive that outcome in the existing Plan69 managed
result contract. Migrate callers and remove duplicated rule interpretations.

Use nextest 0.9.146 native filters and dispatch. In `nextest-metadata` 0.15.0,
`RustTestCaseSummary::filter_match` exposes the case's filter result without effective group
membership. Use native `group(=…)` queries where per-run membership is required and scope them to
relevant contexts. PSE still matches fixture roles to actual cases and owns setup and settlement.
[E02](70e-preparation-and-verification-scope.md#e02) owns the binary and helper listing contract.
A renamed required case, stale role, wrong runner, empty selection and exit zero with
missing evidence must each remain observable as distinct nonpasses. Unrelated pure cases need no
fixture. Exclude unmarked park/foreground subprocess helper entries from assurance counts. Their
marked parent controls must affirm that the helper reached its intended stage.
[Pinned case summary](https://docs.rs/nextest-metadata/0.15.0/nextest_metadata/struct.RustTestCaseSummary.html) ·
[Filtersets](https://nexte.st/docs/filtersets/reference/) · [Managed runner](../../crates/pse-host/src/test_workflow/execute/runner.rs)

### <a id="a03"></a>A03 — Find the actual shared exclusion boundary

Inspect ownership of the named scientific groups and store effects under two independent
invocations. Nextest's named groups enforce scheduling within a run. Cross-invocation exclusion,
where required, belongs to the owner of the shared effect. Compare existing owner admission with
a narrow owner-held claim if the actual shared effect can overlap. Trace claimant identity, acquisition, cancellation, original drain
and safe release. The deciding cases are two runs contending for the same resource, unrelated
pure and private work, and a stopped run whose drain is unresolved. The same-resource case
must serialize where the underlying effect requires it, unrelated work remains concurrent,
and a stopped owner retains the claim until settlement. Use an existing guard where it
establishes the whole contract. Where it does not, specify enforcement at the actual effect owner
and schedule that change here. Where no shared constraint exists, retain per-run nextest policy.
Keep ordinary host width and retain native manager ownership. The inquiry must settle the
scope before adding any enforcement and use [B02](70b-fixtures-and-process-ownership.md#b02) for
receiving verification. [Nextest test groups](https://nexte.st/docs/configuration/test-groups/) ·
[Pinned group implementation](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/nextest-runner/src/runner/imp.rs)

### <a id="a04"></a>A04 — Make specialist routes reachable and discriminating

Wire the four parity interpreter selectors to the explicitly prepared locked interpreter and
D02's actual solver admission. Exercise each selected exchange with clean caller variables and
verify a missing input refuses. Name separate selections for reference-only controls and controls
that consume source-produced native contracts. Use `cargo miri test` on the pinned nightly for
the interpreter control, with explicit Arrow validation on its selected relation closure. Replace
the unsupported adapter invocation. Keep doctests on `cargo test --doc` through the correctness
adapter so rustdoc's real selection receives A01's graph checks. Add an exact
feature route for the arbitrary AST test. Bind qualification-module and long-hold tests to their
own receipt and readiness, separate from production-module borrowing. Migrate ordinary compiler
and modeling capsule consumers to E01's selected authored closure. Keep staged-receipt cases
on their explicit capture route. Repair obsolete binding and validation guide command references.
[Miri CI route](../../.github/workflows/rust-scheduled.yml) · [Cargo test target selection](https://doc.rust-lang.org/cargo/commands/cargo-test.html#target-selection) ·
[Recipes](../../justfile) · [Fixture declarations](../../crates/pse-operations/Cargo.toml) ·
[Qualification guide](../dev/validation-assessment.md)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [A01](#a01) | planned | Current Cargo adapter and tool pin | Actual launched nextest verified, including substitution refusal, and scoped unit-graph and doctest controls. |
| [A02](#a02) | planned | A01 runner identity and existing managed result receiver | Exact selected requirement correspondence, explicit verdict and settlement, and named missing/wrong/unmarked controls. |
| [A03](#a03) | planned | A02 requirement ownership for any enforcement, B02 drain contract for receiving checks | Record the resource owner and observed contention boundary. Where needed enforce it and prove independent overlap and cancellation behavior. |
| [A04](#a04) | planned | A01/A02 route semantics, D02 reference admission for parity, E01 for source-consuming routes, B02 for deadline-sensitive qualification | Each named specialist route selects the intended case and prerequisites. Remove superseded invocation and stale route descriptions. |
